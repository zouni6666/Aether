//! Gateway-backed tunnel end-to-end regressions.
//!
//! 这两个用例需要真实 Gateway 路由和 Tunnel 进程状态，因此放在独立
//! integration package，避免 Workspace Rest 的普通目标编译 Gateway。

use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::AtomicU64;
use std::sync::{Arc, Once};
use std::task::{Context, Poll};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use aether_contracts::tunnel::{
    sign_tunnel_relay_request, tunnel_relay_payload_digest, TUNNEL_RELAY_AUTH_NONCE_HEADER,
    TUNNEL_RELAY_AUTH_PAYLOAD_HEADER, TUNNEL_RELAY_AUTH_SENDER_HEADER,
    TUNNEL_RELAY_AUTH_SIGNATURE_HEADER, TUNNEL_RELAY_AUTH_TIMESTAMP_HEADER,
    TUNNEL_RELAY_OWNER_INSTANCE_HEADER,
};
use aether_gateway::{build_router_with_state, AppState as GatewayAppState};
use aether_tunnel::config::Config;
use aether_tunnel::registration::client::AetherClient;
use aether_tunnel::runtime::DynamicConfig;
use aether_tunnel::state::{
    AppState as TunnelAppState, ServerContext, TunnelMetrics, TunnelRequestMetrics,
};
use aether_tunnel::target_filter::DnsCache;
use aether_tunnel::tunnel::protocol;
use aether_tunnel::tunnel::run;
use aether_tunnel::upstream_client;
use arc_swap::ArcSwap;
use axum::Router;
use reqwest::StatusCode;
use tokio::sync::watch;

struct SessionTask<T>(tokio::task::JoinHandle<T>);

impl<T> SessionTask<T> {
    fn new(handle: tokio::task::JoinHandle<T>) -> Self {
        Self(handle)
    }
}

impl<T> Future for SessionTask<T> {
    type Output = Result<T, tokio::task::JoinError>;

    fn poll(mut self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.0).poll(context)
    }
}

impl<T> Drop for SessionTask<T> {
    fn drop(&mut self) {
        self.0.abort();
    }
}

#[tokio::test]
async fn tunnel_reconnects_after_gateway_restart() {
    ensure_rustls_provider();

    let gateway_port = reserve_local_port().expect("gateway port should reserve");
    let gateway_base_url = format!("http://127.0.0.1:{gateway_port}");
    let (gateway_state, mut gateway_handle) = start_gateway_on_port(gateway_port)
        .await
        .expect("gateway should start");

    let mut tunnel_config = sample_config(&gateway_base_url);
    tunnel_config.tunnel_security = aether_tunnel::config::TunnelSecurity::NonTlsRequired;
    tunnel_config.tunnel_encryption_key =
        Some("BwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwc=".to_string());
    let state = sample_state(tunnel_config);
    let server = sample_server(&state, "node-recovery");
    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    let tunnel_task = tokio::spawn({
        let state = Arc::clone(&state);
        let server = Arc::clone(&server);
        let (_drain_tx, drain_rx) = watch::channel(false);
        async move {
            run(&state, &server, 0, shutdown_rx, drain_rx).await;
        }
    });

    wait_until_relay_status(
        &gateway_base_url,
        "node-recovery",
        StatusCode::GATEWAY_TIMEOUT,
    )
    .await;

    gateway_handle.abort();
    let _ = (&mut gateway_handle).await;
    assert_eq!(gateway_state.force_close_all_tunnel_proxies(), 1);

    let (_restarted_gateway_state, restarted_gateway_handle) =
        start_gateway_on_port_retry(gateway_port)
            .await
            .expect("gateway should restart on fixed port");
    gateway_handle = restarted_gateway_handle;

    wait_until_relay_status(
        &gateway_base_url,
        "node-recovery",
        StatusCode::GATEWAY_TIMEOUT,
    )
    .await;

    assert!(server.tunnel_metrics.snapshot().connect_successes >= 2);
    let _ = shutdown_tx.send(true);
    tokio::time::timeout(Duration::from_secs(5), tunnel_task)
        .await
        .expect("tunnel task should stop")
        .expect("tunnel task should join");
    gateway_handle.abort();
}

async fn wait_until_relay_status(gateway_base_url: &str, node_id: &str, expected: StatusCode) {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(10);
    let mut last_observed = None::<String>;
    loop {
        if let Some((status, body)) = probe_relay_status(gateway_base_url, node_id).await {
            last_observed = Some(format!("{status} body={body}"));
            if status == expected {
                return;
            }
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "relay status did not become {expected} within timeout; last={:?}",
            last_observed
        );
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
}

async fn probe_relay_status(gateway_base_url: &str, node_id: &str) -> Option<(StatusCode, String)> {
    let response = relay_response(gateway_base_url, node_id, relay_probe_envelope()).await?;
    let status = response.status();
    let body = response.text().await.unwrap_or_default();
    Some((status, body))
}

async fn relay_response(
    gateway_base_url: &str,
    node_id: &str,
    payload: Vec<u8>,
) -> Option<reqwest::Response> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("test clock should be after epoch")
        .as_secs();
    let nonce = uuid::Uuid::new_v4().simple().to_string();
    let digest = tunnel_relay_payload_digest(&payload, &[]);
    let signature = sign_tunnel_relay_request(
        b"tunnel-reconnect-test-secret-at-least-32-bytes",
        "tunnel-reconnect-test-client",
        "tunnel-reconnect-test-gateway",
        node_id,
        "",
        false,
        timestamp,
        &nonce,
        &digest,
    );
    reqwest::Client::new()
        .post(format!(
            "{gateway_base_url}/api/internal/tunnel/relay/{node_id}"
        ))
        .header("content-type", "application/octet-stream")
        .header(
            TUNNEL_RELAY_AUTH_SENDER_HEADER,
            "tunnel-reconnect-test-client",
        )
        .header(
            TUNNEL_RELAY_OWNER_INSTANCE_HEADER,
            "tunnel-reconnect-test-gateway",
        )
        .header(TUNNEL_RELAY_AUTH_TIMESTAMP_HEADER, timestamp)
        .header(TUNNEL_RELAY_AUTH_NONCE_HEADER, nonce)
        .header(
            TUNNEL_RELAY_AUTH_PAYLOAD_HEADER,
            digest.encode_header_value(),
        )
        .header(TUNNEL_RELAY_AUTH_SIGNATURE_HEADER, signature)
        .body(payload)
        .send()
        .await
        .ok()
}

fn relay_probe_envelope() -> Vec<u8> {
    let meta = protocol::RequestMeta {
        provider_id: None,
        endpoint_id: None,
        key_id: None,
        method: "GET".to_string(),
        url: "http://127.0.0.1:80/blocked".to_string(),
        headers: std::collections::HashMap::new(),
        stream: false,
        request_timeout_ms: None,
        stream_first_byte_timeout_ms: None,
        timeout: 5,
        follow_redirects: None,
        http1_only: false,
        transport_profile: None,
    };
    let meta_json =
        serde_json::to_vec(&meta).expect("tunnel relay probe metadata should serialize");
    let mut envelope = Vec::with_capacity(4 + meta_json.len());
    envelope.extend_from_slice(&(meta_json.len() as u32).to_be_bytes());
    envelope.extend_from_slice(&meta_json);
    envelope
}

async fn start_gateway_on_port(
    port: u16,
) -> Result<(GatewayAppState, tokio::task::JoinHandle<()>), std::io::Error> {
    // The embedded gateway now fails closed when relay authentication is
    // not configured. Keep this integration fixture explicitly authenticated.
    static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    let state = {
        let _guard = ENV_LOCK.lock().unwrap();
        let previous_secret = std::env::var_os("AETHER_TUNNEL_RELAY_AUTH_SECRET");
        let previous_instance = std::env::var_os("AETHER_GATEWAY_INSTANCE_ID");
        std::env::set_var(
            "AETHER_TUNNEL_RELAY_AUTH_SECRET",
            "tunnel-reconnect-test-secret-at-least-32-bytes",
        );
        std::env::set_var(
            "AETHER_GATEWAY_INSTANCE_ID",
            "tunnel-reconnect-test-gateway",
        );
        let mut state = GatewayAppState::new().expect("gateway test state should build");
        aether_gateway::configure_test_tunnel_security(
            &mut state,
            "node-recovery",
            "test-generation-1",
            "BwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwc=",
        );
        restore_test_env("AETHER_TUNNEL_RELAY_AUTH_SECRET", previous_secret);
        restore_test_env("AETHER_GATEWAY_INSTANCE_ID", previous_instance);
        state
    };
    let router = build_router_with_state(state.clone());
    let handle = spawn_router_on_port(port, router).await?;
    Ok((state, handle))
}

#[tokio::test]
async fn negotiated_small_window_streams_large_responses_and_cancels_idle_upstream() {
    use axum::body::{Body, Bytes};
    use axum::routing::get;
    use futures_util::StreamExt;

    ensure_rustls_provider();
    let upstream_port = reserve_local_port().unwrap();
    let upstream = Router::new()
        .route(
            "/large",
            get(|| async { Body::from(vec![b'x'; 2 * 1024 * 1024]) }),
        )
        .route(
            "/idle",
            get(|| async {
                let first = futures_util::stream::once(async {
                    Ok::<_, std::io::Error>(Bytes::from_static(b"data: started\n\n"))
                });
                (
                    [("content-type", "text/event-stream")],
                    Body::from_stream(first.chain(futures_util::stream::pending())),
                )
            }),
        );
    let upstream_task =
        SessionTask::new(spawn_router_on_port(upstream_port, upstream).await.unwrap());
    let gateway_port = reserve_local_port().unwrap();
    let gateway_url = format!("http://127.0.0.1:{gateway_port}");
    let (_, gateway_task) = start_gateway_on_port(gateway_port).await.unwrap();
    let gateway_task = SessionTask::new(gateway_task);
    let mut config = sample_config(&gateway_url);
    config.tunnel_security = aether_tunnel::config::TunnelSecurity::NonTlsRequired;
    config.tunnel_encryption_key = Some("BwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwc=".into());
    config.tunnel_stream_initial_window_bytes = 512 * 1024;
    config.tunnel_drain_deadline_ms = 100;
    config.allow_private_targets = true;
    config.allowed_ports.push(upstream_port);
    let state = sample_state(config);
    let server = sample_server(&state, "node-recovery");
    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    let (_drain_tx, drain_rx) = watch::channel(false);
    let tunnel_task = SessionTask::new(tokio::spawn({
        let state = Arc::clone(&state);
        let server = Arc::clone(&server);
        async move {
            run(&state, &server, 0, shutdown_rx, drain_rx).await;
        }
    }));
    wait_until_relay_status(&gateway_url, "node-recovery", StatusCode::GATEWAY_TIMEOUT).await;

    let envelope = |path: &str| {
        let mut meta: protocol::RequestMeta =
            serde_json::from_slice(&relay_probe_envelope()[4..]).unwrap();
        meta.url = format!("http://127.0.0.1:{upstream_port}/{path}");
        meta.stream = true;
        meta.timeout = 10;
        meta.stream_first_byte_timeout_ms = Some(10_000);
        let encoded = serde_json::to_vec(&meta).unwrap();
        let mut result = (encoded.len() as u32).to_be_bytes().to_vec();
        result.extend(encoded);
        result
    };
    let response = relay_response(&gateway_url, "node-recovery", envelope("large"))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = tokio::time::timeout(Duration::from_secs(10), response.bytes())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(body.len(), 2 * 1024 * 1024);
    assert!(body.iter().all(|byte| *byte == b'x'));

    let mut response = relay_response(&gateway_url, "node-recovery", envelope("idle"))
        .await
        .unwrap();
    assert_eq!(
        response.chunk().await.unwrap().unwrap(),
        "data: started\n\n"
    );
    drop(response);
    tokio::time::timeout(Duration::from_secs(3), async {
        while server
            .active_connections
            .load(std::sync::atomic::Ordering::Acquire)
            != 0
        {
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("cancelled SSE must release the upstream handler");

    let mut response = relay_response(&gateway_url, "node-recovery", envelope("idle"))
        .await
        .unwrap();
    assert!(response.chunk().await.unwrap().is_some());
    shutdown_tx.send(true).unwrap();
    tokio::time::timeout(Duration::from_secs(3), tunnel_task)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        server
            .active_connections
            .load(std::sync::atomic::Ordering::Acquire),
        0
    );
    drop(response);
    drop(gateway_task);
    drop(upstream_task);
}

fn restore_test_env(key: &str, value: Option<std::ffi::OsString>) {
    if let Some(value) = value {
        std::env::set_var(key, value);
    } else {
        std::env::remove_var(key);
    }
}

async fn start_gateway_on_port_retry(
    port: u16,
) -> Result<(GatewayAppState, tokio::task::JoinHandle<()>), std::io::Error> {
    let mut attempts = 0usize;
    loop {
        match start_gateway_on_port(port).await {
            Ok(server) => return Ok(server),
            Err(err) => {
                attempts += 1;
                if attempts >= 20 {
                    return Err(err);
                }
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
        }
    }
}

async fn spawn_router_on_port(
    port: u16,
    app: Router,
) -> Result<tokio::task::JoinHandle<()>, std::io::Error> {
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", port)).await?;
    Ok(tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
        )
        .await
        .expect("gateway test server should run");
    }))
}

fn reserve_local_port() -> Result<u16, std::io::Error> {
    let listener = std::net::TcpListener::bind("127.0.0.1:0")?;
    let port = listener.local_addr()?.port();
    drop(listener);
    Ok(port)
}

fn sample_state(config: Config) -> Arc<TunnelAppState> {
    let config = Arc::new(config);
    let dns_cache = Arc::new(DnsCache::new(Duration::from_secs(60), 128));
    let upstream_client_pool =
        upstream_client::UpstreamClientPool::new(Arc::clone(&config), Arc::clone(&dns_cache));
    Arc::new(TunnelAppState {
        config,
        dns_cache,
        upstream_client_pool,
        tunnel_tls_config: Arc::new(aether_tunnel::tunnel::client::build_tls_config()),
        resource_monitor: Arc::new(aether_tunnel::hardware::RuntimeResourceMonitor::new()),
        stream_gate: None,
        distributed_stream_gate: None,
    })
}

fn sample_server(state: &Arc<TunnelAppState>, node_id: &str) -> Arc<ServerContext> {
    let config = Arc::clone(&state.config);
    Arc::new(ServerContext {
        server_label: "gateway-owned-tunnel".to_string(),
        aether_url: config.aether_url.clone(),
        management_token: config.management_token.clone(),
        tunnel_security: config.tunnel_security,
        tunnel_encryption_key: config.tunnel_encryption_key.clone(),
        node_name: config.node_name.clone(),
        node_id: Arc::new(std::sync::RwLock::new(node_id.to_string())),
        tunnel_generation: "test-generation-1".to_string(),
        aether_client: Arc::new(AetherClient::new(
            &config,
            &config.aether_url,
            &config.management_token,
        )),
        dynamic: Arc::new(ArcSwap::from_pointee(DynamicConfig::from_config(&config))),
        active_connections: Arc::new(AtomicU64::new(0)),
        metrics: Arc::new(TunnelRequestMetrics::new()),
        tunnel_metrics: Arc::new(TunnelMetrics::new()),
    })
}

fn sample_config(aether_url: &str) -> Config {
    Config {
        aether_url: aether_url.to_string(),
        management_token: "token".to_string(),
        public_ip: None,
        node_name: "tunnel-test".to_string(),
        tunnel_security: aether_tunnel::config::TunnelSecurity::Off,
        tunnel_encryption_key: None,
        node_region: None,
        heartbeat_interval: 1,
        allowed_ports: vec![80, 443],
        allow_private_targets: false,
        aether_request_timeout_secs: 10,
        aether_connect_timeout_secs: 2,
        aether_pool_max_idle_per_host: 8,
        aether_pool_idle_timeout_secs: 90,
        aether_tcp_keepalive_secs: 60,
        aether_tcp_nodelay: true,
        aether_http2: true,
        aether_outbound_proxy_url: None,
        aether_retry_max_attempts: 1,
        aether_retry_base_delay_ms: 50,
        aether_retry_max_delay_ms: 100,
        diagnostics_bind: None,
        max_concurrent_connections: None,
        max_in_flight_streams: None,
        distributed_stream_limit: None,
        distributed_stream_redis_url: None,
        distributed_stream_redis_key_prefix: None,
        distributed_stream_lease_ttl_ms: 30_000,
        distributed_stream_renew_interval_ms: 10_000,
        distributed_stream_command_timeout_ms: 1_000,
        dns_cache_ttl_secs: 60,
        dns_cache_capacity: 128,
        upstream_connect_timeout_secs: 30,
        upstream_pool_max_idle_per_host: 4,
        upstream_pool_idle_timeout_secs: 60,
        upstream_client_pool_capacity: aether_tunnel::config::DEFAULT_UPSTREAM_CLIENT_POOL_CAPACITY,
        upstream_tcp_keepalive_secs: 60,
        upstream_tcp_nodelay: true,
        upstream_proxy_url: None,
        upstream_proxy_remote_dns: false,
        legacy_redirect_replay_budget_bytes_ignored: None,
        emit_proxy_timing_header: true,
        log_level: "info".to_string(),
        log_destination: aether_tunnel::config::TunnelLogDestinationArg::Stdout,
        log_dir: None,
        log_rotation: aether_tunnel::config::TunnelLogRotationArg::Daily,
        log_retention_days: 7,
        log_max_files: 30,
        tunnel_reconnect_base_ms: 50,
        tunnel_reconnect_max_ms: 250,
        tunnel_ping_interval_ms: 1_000,
        tunnel_max_streams: Some(8),
        tunnel_profile: aether_tunnel::config::TunnelProfileArg::Lite,
        tunnel_stream_initial_window_bytes:
            aether_tunnel::config::DEFAULT_TUNNEL_STREAM_INITIAL_WINDOW_BYTES,
        tunnel_drain_deadline_ms: aether_tunnel::config::DEFAULT_TUNNEL_DRAIN_DEADLINE_MS,
        tunnel_connect_timeout_ms: 2_000,
        tunnel_ipv4_only: false,
        tunnel_ipv6_only: false,
        tunnel_tcp_keepalive_secs: 30,
        tunnel_tcp_nodelay: true,
        tunnel_stale_timeout_ms: 5_000,
        tunnel_connections: Some(1),
        tunnel_connections_max: Some(1),
        tunnel_scale_check_interval_ms: 1_000,
        tunnel_scale_up_threshold_percent: 70,
        tunnel_scale_down_threshold_percent: 35,
        tunnel_scale_down_grace_secs: 15,
    }
}

fn ensure_rustls_provider() {
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        let _ = rustls::crypto::ring::default_provider().install_default();
    });
}

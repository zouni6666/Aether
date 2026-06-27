use crate::tests::{
    any, build_router, build_router_with_state, start_server, AppState, Arc, Body, Mutex, Request,
    Router, StatusCode,
};
use aether_data::repository::oauth_providers::{
    InMemoryOAuthProviderRepository, StoredOAuthProviderConfig,
};

fn sample_identity_oauth_provider(provider_type: &str) -> StoredOAuthProviderConfig {
    StoredOAuthProviderConfig::new(
        provider_type.to_string(),
        "Linux DO".to_string(),
        "client-id".to_string(),
        "https://backend.example.com/oauth/callback".to_string(),
        "https://frontend.example.com/auth/callback".to_string(),
    )
    .expect("oauth provider config should build")
    .with_config_fields(
        None,
        Some("https://connect.linux.do/oauth2/authorize".to_string()),
        Some("https://connect.linux.do/oauth2/token".to_string()),
        Some("https://connect.linux.do/api/user".to_string()),
        Some(vec!["openid".to_string()]),
        Some(serde_json::json!({"email": "email"})),
        None,
        None,
        true,
    )
}

#[tokio::test]
async fn gateway_serves_oauth_public_providers_locally_without_hitting_upstream() {
    let upstream_hits = Arc::new(Mutex::new(0usize));
    let upstream_hits_clone = Arc::clone(&upstream_hits);
    let upstream = Router::new().route(
        "/{*path}",
        any(move |_request: Request| {
            let upstream_hits_inner = Arc::clone(&upstream_hits_clone);
            async move {
                *upstream_hits_inner.lock().expect("mutex should lock") += 1;
                (StatusCode::OK, Body::from("proxied"))
            }
        }),
    );

    let (upstream_url, upstream_handle) = start_server(upstream).await;
    let gateway = build_router().expect("gateway should build");
    let (gateway_url, gateway_handle) = start_server(gateway).await;

    let response = reqwest::Client::new()
        .get(format!("{gateway_url}/api/oauth/providers"))
        .send()
        .await
        .expect("request should succeed");

    assert_eq!(response.status(), StatusCode::OK);
    let payload: serde_json::Value = response.json().await.expect("json body should parse");
    assert_eq!(payload["providers"].as_array().map(Vec::len), Some(0));
    assert_eq!(*upstream_hits.lock().expect("mutex should lock"), 0);

    gateway_handle.abort();
    upstream_handle.abort();
}

#[tokio::test]
async fn gateway_accepts_oauth_authorize_device_id_header_without_hitting_upstream() {
    let upstream_hits = Arc::new(Mutex::new(0usize));
    let upstream_hits_clone = Arc::clone(&upstream_hits);
    let upstream = Router::new().route(
        "/{*path}",
        any(move |_request: Request| {
            let upstream_hits_inner = Arc::clone(&upstream_hits_clone);
            async move {
                *upstream_hits_inner.lock().expect("mutex should lock") += 1;
                (StatusCode::OK, Body::from("proxied"))
            }
        }),
    );

    let (upstream_url, upstream_handle) = start_server(upstream).await;
    let gateway = build_router().expect("gateway should build");
    let (gateway_url, gateway_handle) = start_server(gateway).await;

    let response = reqwest::Client::new()
        .get(format!("{gateway_url}/api/oauth/linuxdo/authorize"))
        .header("x-client-device-id", "device-123")
        .send()
        .await
        .expect("request should succeed");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let payload: serde_json::Value = response.json().await.expect("json body should parse");
    assert_eq!(payload["detail"], "OAuth Provider 不存在或已禁用");
    assert_eq!(*upstream_hits.lock().expect("mutex should lock"), 0);

    gateway_handle.abort();
    upstream_handle.abort();
}

#[tokio::test]
async fn gateway_blocks_configured_oauth_provider_when_oauth_module_disabled() {
    let upstream_hits = Arc::new(Mutex::new(0usize));
    let upstream_hits_clone = Arc::clone(&upstream_hits);
    let upstream = Router::new().route(
        "/{*path}",
        any(move |_request: Request| {
            let upstream_hits_inner = Arc::clone(&upstream_hits_clone);
            async move {
                *upstream_hits_inner.lock().expect("mutex should lock") += 1;
                (StatusCode::OK, Body::from("proxied"))
            }
        }),
    );

    let repository = Arc::new(InMemoryOAuthProviderRepository::seed(vec![
        sample_identity_oauth_provider("linuxdo"),
    ]));
    let data_state =
        crate::data::GatewayDataState::with_oauth_provider_repository_for_tests(repository)
            .with_system_config_values_for_tests(vec![(
                "module.oauth.enabled".to_string(),
                serde_json::json!(false),
            )]);

    let (upstream_url, upstream_handle) = start_server(upstream).await;
    let gateway = build_router_with_state(
        AppState::new()
            .expect("gateway should build")
            .with_data_state_for_tests(data_state),
    );
    let (gateway_url, gateway_handle) = start_server(gateway).await;

    let providers_response = reqwest::Client::new()
        .get(format!("{gateway_url}/api/oauth/providers"))
        .send()
        .await
        .expect("request should succeed");

    assert_eq!(providers_response.status(), StatusCode::OK);
    let providers_payload: serde_json::Value = providers_response
        .json()
        .await
        .expect("json body should parse");
    assert_eq!(
        providers_payload["providers"].as_array().map(Vec::len),
        Some(0)
    );

    let authorize_response = reqwest::Client::new()
        .get(format!("{gateway_url}/api/oauth/linuxdo/authorize"))
        .header("x-client-device-id", "device-123")
        .send()
        .await
        .expect("request should succeed");

    assert_eq!(authorize_response.status(), StatusCode::NOT_FOUND);
    let authorize_payload: serde_json::Value = authorize_response
        .json()
        .await
        .expect("json body should parse");
    assert_eq!(authorize_payload["detail"], "OAuth Provider 不存在或已禁用");
    assert_eq!(*upstream_hits.lock().expect("mutex should lock"), 0);

    gateway_handle.abort();
    upstream_handle.abort();
}

#[tokio::test]
async fn gateway_serves_configured_oauth_provider_when_oauth_module_enabled() {
    let upstream_hits = Arc::new(Mutex::new(0usize));
    let upstream_hits_clone = Arc::clone(&upstream_hits);
    let upstream = Router::new().route(
        "/{*path}",
        any(move |_request: Request| {
            let upstream_hits_inner = Arc::clone(&upstream_hits_clone);
            async move {
                *upstream_hits_inner.lock().expect("mutex should lock") += 1;
                (StatusCode::OK, Body::from("proxied"))
            }
        }),
    );

    let repository = Arc::new(InMemoryOAuthProviderRepository::seed(vec![
        sample_identity_oauth_provider("linuxdo"),
    ]));
    let data_state =
        crate::data::GatewayDataState::with_oauth_provider_repository_for_tests(repository)
            .with_system_config_values_for_tests(vec![(
                "module.oauth.enabled".to_string(),
                serde_json::json!(true),
            )]);

    let (upstream_url, upstream_handle) = start_server(upstream).await;
    let gateway = build_router_with_state(
        AppState::new()
            .expect("gateway should build")
            .with_data_state_for_tests(data_state),
    );
    let (gateway_url, gateway_handle) = start_server(gateway).await;

    let providers_response = reqwest::Client::new()
        .get(format!("{gateway_url}/api/oauth/providers"))
        .send()
        .await
        .expect("request should succeed");

    assert_eq!(providers_response.status(), StatusCode::OK);
    let providers_payload: serde_json::Value = providers_response
        .json()
        .await
        .expect("json body should parse");
    assert_eq!(
        providers_payload["providers"],
        serde_json::json!([{
            "provider_type": "linuxdo",
            "display_name": "Linux DO"
        }])
    );

    let authorize_client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("client should build");
    let authorize_response = authorize_client
        .get(format!("{gateway_url}/api/oauth/linuxdo/authorize"))
        .header("x-client-device-id", "device-123")
        .send()
        .await
        .expect("request should succeed");

    assert_eq!(authorize_response.status(), StatusCode::FOUND);
    let location = authorize_response
        .headers()
        .get(http::header::LOCATION)
        .and_then(|value| value.to_str().ok())
        .expect("location should exist");
    assert!(location.starts_with("https://connect.linux.do/oauth2/authorize?"));
    assert!(location.contains("client_id=client-id"));
    assert!(location.contains("redirect_uri=https%3A%2F%2Fbackend.example.com%2Foauth%2Fcallback"));
    assert_eq!(*upstream_hits.lock().expect("mutex should lock"), 0);

    gateway_handle.abort();
    upstream_handle.abort();
}

#[tokio::test]
async fn gateway_requires_auth_for_oauth_user_bindable_providers_without_hitting_upstream() {
    let upstream_hits = Arc::new(Mutex::new(0usize));
    let upstream_hits_clone = Arc::clone(&upstream_hits);
    let upstream = Router::new().route(
        "/{*path}",
        any(move |_request: Request| {
            let upstream_hits_inner = Arc::clone(&upstream_hits_clone);
            async move {
                *upstream_hits_inner.lock().expect("mutex should lock") += 1;
                (StatusCode::OK, Body::from("proxied"))
            }
        }),
    );

    let (upstream_url, upstream_handle) = start_server(upstream).await;
    let gateway = build_router().expect("gateway should build");
    let (gateway_url, gateway_handle) = start_server(gateway).await;

    let response = reqwest::Client::new()
        .get(format!("{gateway_url}/api/user/oauth/bindable-providers"))
        .send()
        .await
        .expect("request should succeed");

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let payload: serde_json::Value = response.json().await.expect("json body should parse");
    assert_eq!(payload["detail"], "缺少用户凭证");
    assert_eq!(*upstream_hits.lock().expect("mutex should lock"), 0);

    gateway_handle.abort();
    upstream_handle.abort();
}

#[tokio::test]
async fn gateway_requires_auth_for_oauth_user_bind_token_without_hitting_upstream() {
    let upstream_hits = Arc::new(Mutex::new(0usize));
    let upstream_hits_clone = Arc::clone(&upstream_hits);
    let upstream = Router::new().route(
        "/{*path}",
        any(move |_request: Request| {
            let upstream_hits_inner = Arc::clone(&upstream_hits_clone);
            async move {
                *upstream_hits_inner.lock().expect("mutex should lock") += 1;
                (StatusCode::OK, Body::from("proxied"))
            }
        }),
    );

    let (upstream_url, upstream_handle) = start_server(upstream).await;
    let gateway = build_router().expect("gateway should build");
    let (gateway_url, gateway_handle) = start_server(gateway).await;

    let response = reqwest::Client::new()
        .post(format!("{gateway_url}/api/user/oauth/linuxdo/bind-token"))
        .send()
        .await
        .expect("request should succeed");

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let payload: serde_json::Value = response.json().await.expect("json body should parse");
    assert_eq!(payload["detail"], "缺少用户凭证");
    assert_eq!(*upstream_hits.lock().expect("mutex should lock"), 0);

    gateway_handle.abort();
    upstream_handle.abort();
}

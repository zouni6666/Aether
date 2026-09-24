pub(super) use std::convert::Infallible;
pub(super) use std::sync::{Arc, Mutex};

pub(super) use axum::body::{to_bytes, Body, Bytes};
pub(super) use axum::response::Response;
pub(super) use axum::routing::any;
pub(super) use axum::{extract::Request, Json, Router};
pub(super) use http::header::{HeaderName, HeaderValue};
pub(super) use http::StatusCode;
pub(super) use serde_json::json;

mod ai_execute;
mod async_task;
mod audit;
mod concurrency;
mod control;
mod files;
mod frontdoor;
mod operational_auth;
mod proxy;
mod usage;
mod video;

pub(super) use super::async_task::VideoTaskTruthSourceMode;
pub(super) use super::constants::*;
pub(super) use super::fallback_metrics::{GatewayFallbackMetricKind, GatewayFallbackReason};
pub(super) use super::rate_limit::FrontdoorUserRpmConfig;
pub(super) use super::router::{attach_static_frontend, build_router, build_router_with_state};
pub(super) use super::state::{AppState, FrontdoorCorsConfig};
pub(super) use super::usage::UsageRuntimeConfig;

pub(super) async fn start_server(app: Router) -> (String, tokio::task::JoinHandle<()>) {
    let listener = crate::test_support::bind_loopback_listener()
        .await
        .expect("listener should bind");
    let addr = listener.local_addr().expect("local addr should resolve");
    let handle = tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
        )
        .await
        .expect("server should run");
    });
    (format!("http://{addr}"), handle)
}

/// 在独立的大栈线程中运行需要深调用栈的异步测试。
///
/// 这些测试仍保留 16 MiB 栈空间；这里只统一线程和 runtime 的启动逻辑，
/// 避免每个测试分区各自复制一份 helper，降低维护时误改测试执行语义的风险。
pub(crate) fn run_async_test_on_large_stack<F, Fut>(
    test_name: &'static str,
    stack_size: usize,
    make_future: F,
) where
    F: FnOnce() -> Fut + Send + 'static,
    Fut: std::future::Future<Output = ()> + 'static,
{
    run_async_test_on_large_stack_with_result(test_name, stack_size, make_future);
}

/// 与上面的 helper 相同，但允许深栈测试返回结果，供公共请求 helper 使用。
pub(crate) fn run_async_test_on_large_stack_with_result<F, Fut, R>(
    test_name: &'static str,
    stack_size: usize,
    make_future: F,
) -> R
where
    F: FnOnce() -> Fut + Send + 'static,
    Fut: std::future::Future<Output = R> + 'static,
    R: Send + 'static,
{
    let handle = std::thread::Builder::new()
        .name(test_name.to_string())
        .stack_size(stack_size)
        .spawn(move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("test runtime should build");
            runtime.block_on(make_future())
        })
        .expect("large-stack test thread should spawn");

    match handle.join() {
        Ok(result) => result,
        Err(payload) => std::panic::resume_unwind(payload),
    }
}

pub(super) const OPERATIONAL_ADMIN_DEVICE_ID: &str = "device-operational-admin";

pub(super) async fn start_authenticated_operational_server(
    state: AppState,
) -> (String, tokio::task::JoinHandle<()>, String) {
    let access_token =
        control::issue_shared_test_admin_access_token(&state, OPERATIONAL_ADMIN_DEVICE_ID).await;
    let (url, handle) = start_server(build_router_with_state(state)).await;
    (url, handle, access_token)
}

pub(super) fn authenticated_operational_client(access_token: &str) -> reqwest::Client {
    authenticated_operational_client_with_builder(reqwest::Client::builder(), access_token)
}

pub(super) fn authenticated_operational_client_with_builder(
    builder: reqwest::ClientBuilder,
    access_token: &str,
) -> reqwest::Client {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        reqwest::header::AUTHORIZATION,
        format!("Bearer {access_token}")
            .parse()
            .expect("operational authorization header should build"),
    );
    headers.insert(
        "x-client-device-id",
        OPERATIONAL_ADMIN_DEVICE_ID
            .parse()
            .expect("operational device header should build"),
    );
    builder
        .default_headers(headers)
        .build()
        .expect("operational client should build")
}

pub(super) async fn send_request(app: Router, mut request: Request) -> Response {
    use tower::ServiceExt;

    request
        .extensions_mut()
        .insert(axum::extract::ConnectInfo(std::net::SocketAddr::from((
            [127, 0, 0, 1],
            40000,
        ))));
    app.oneshot(request)
        .await
        .expect("router request should complete")
}

pub(super) fn build_router_with_execution_runtime_override(
    execution_runtime_override_base_url: impl Into<String>,
) -> Router {
    let state = build_state_with_execution_runtime_override(execution_runtime_override_base_url);
    build_router_with_state(state)
}

pub(super) fn build_state_with_execution_runtime_override(
    execution_runtime_override_base_url: impl Into<String>,
) -> AppState {
    AppState::new()
        .expect("gateway should build")
        .with_execution_runtime_override_base_url(execution_runtime_override_base_url)
}

pub(super) async fn wait_until(timeout_ms: u64, mut predicate: impl FnMut() -> bool) {
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_millis(timeout_ms);
    loop {
        if predicate() {
            return;
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "condition not met within {}ms",
            timeout_ms
        );
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
}

pub(crate) fn strip_sse_keepalive_comments(body: &str) -> String {
    body.replace(": aether-keepalive\n\n", "")
}

pub(crate) async fn next_non_keepalive_chunk(response: &mut reqwest::Response) -> Bytes {
    loop {
        let chunk = response
            .chunk()
            .await
            .expect("chunk should read")
            .expect("chunk should exist");
        if chunk.as_ref() != b": aether-keepalive\n\n" {
            return chunk;
        }
    }
}

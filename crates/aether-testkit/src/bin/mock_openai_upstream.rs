use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use axum::body::{Body, Bytes};
use axum::extract::State;
use axum::http::{header, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::Router;
use serde_json::json;

#[derive(Debug, Clone)]
struct Config {
    binds: Vec<SocketAddr>,
    chunks: u64,
    first_byte_delay: Duration,
    chunk_delay: Duration,
    payload_bytes: usize,
    status: StatusCode,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            binds: vec!["127.0.0.1:18181"
                .parse()
                .expect("default bind address should parse")],
            chunks: 8,
            first_byte_delay: Duration::from_millis(0),
            chunk_delay: Duration::from_millis(20),
            payload_bytes: 32,
            status: StatusCode::OK,
        }
    }
}

#[derive(Debug, Default)]
struct Metrics {
    requests_total: AtomicU64,
    completed_total: AtomicU64,
    in_flight: AtomicU64,
    max_in_flight: AtomicU64,
}

#[derive(Debug, Clone)]
struct App {
    config: Config,
    metrics: Arc<Metrics>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = parse_args(std::env::args().skip(1).collect())?;
    let app_state = App {
        config: config.clone(),
        metrics: Arc::new(Metrics::default()),
    };

    let app = Router::new()
        .route("/health", get(health))
        .route("/metrics", get(metrics))
        .route("/v1/chat/completions", post(chat_completions))
        .route("/chat/completions", post(chat_completions))
        .route("/v1/responses", post(responses))
        .route("/responses", post(responses))
        .with_state(app_state);

    serve_listeners(&config.binds, app).await?;
    Ok(())
}

async fn serve_listeners(
    binds: &[SocketAddr],
    app: Router,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut listeners = Vec::with_capacity(binds.len());
    for bind in binds {
        listeners.push((*bind, tokio::net::TcpListener::bind(bind).await?));
    }
    if listeners.len() == 1 {
        let (bind, listener) = listeners
            .into_iter()
            .next()
            .ok_or_else(|| std::io::Error::other("mock upstream listener set is empty"))?;
        eprintln!("mock OpenAI upstream listening on http://{bind}");
        axum::serve(listener, app).await?;
        return Ok(());
    }

    let mut servers = tokio::task::JoinSet::new();
    for (bind, listener) in listeners {
        let app = app.clone();
        eprintln!("mock OpenAI upstream listening on http://{bind}");
        servers.spawn(async move { axum::serve(listener, app).await });
    }
    if let Some(result) = servers.join_next().await {
        servers.abort_all();
        let serve_result = result
            .map_err(|err| std::io::Error::other(format!("mock listener task failed: {err}")))?;
        serve_result?;
    }
    Ok(())
}

async fn health() -> impl IntoResponse {
    (StatusCode::OK, "ok\n")
}

async fn metrics(State(app): State<App>) -> Response {
    let body = format!(
        concat!(
            "# HELP mock_upstream_requests_total Total requests accepted by the mock upstream.\n",
            "# TYPE mock_upstream_requests_total counter\n",
            "mock_upstream_requests_total {}\n",
            "# HELP mock_upstream_completed_total Total requests completed by the mock upstream.\n",
            "# TYPE mock_upstream_completed_total counter\n",
            "mock_upstream_completed_total {}\n",
            "# HELP mock_upstream_in_flight Current in-flight requests/streams.\n",
            "# TYPE mock_upstream_in_flight gauge\n",
            "mock_upstream_in_flight {}\n",
            "# HELP mock_upstream_max_in_flight Maximum in-flight requests/streams observed.\n",
            "# TYPE mock_upstream_max_in_flight gauge\n",
            "mock_upstream_max_in_flight {}\n"
        ),
        app.metrics.requests_total.load(Ordering::Acquire),
        app.metrics.completed_total.load(Ordering::Acquire),
        app.metrics.in_flight.load(Ordering::Acquire),
        app.metrics.max_in_flight.load(Ordering::Acquire),
    );
    (StatusCode::OK, body).into_response()
}

async fn chat_completions(State(app): State<App>, body: Bytes) -> Response {
    let stream = request_wants_stream(&body);
    record_request_started(&app.metrics);
    if app.config.status != StatusCode::OK {
        record_request_completed(&app.metrics);
        return (app.config.status, "mock upstream error\n").into_response();
    }
    if stream {
        return build_chat_sse_response(app);
    }
    let payload = json!({
        "id": "chatcmpl-mock",
        "object": "chat.completion",
        "created": current_unix_secs(),
        "model": request_model(&body).unwrap_or_else(|| "mock-model".to_string()),
        "choices": [{
            "index": 0,
            "message": {
                "role": "assistant",
                "content": mock_payload(app.config.payload_bytes)
            },
            "finish_reason": "stop"
        }],
        "usage": {
            "prompt_tokens": 1,
            "completion_tokens": app.config.chunks.max(1),
            "total_tokens": app.config.chunks.max(1) + 1
        }
    });
    record_request_completed(&app.metrics);
    axum::Json(payload).into_response()
}

async fn responses(State(app): State<App>, body: Bytes) -> Response {
    let stream = request_wants_stream(&body);
    record_request_started(&app.metrics);
    if app.config.status != StatusCode::OK {
        record_request_completed(&app.metrics);
        return (app.config.status, "mock upstream error\n").into_response();
    }
    if stream {
        return build_responses_sse_response(app);
    }
    let payload = json!({
        "id": "resp_mock",
        "object": "response",
        "created_at": current_unix_secs(),
        "model": request_model(&body).unwrap_or_else(|| "mock-model".to_string()),
        "output": [{
            "type": "message",
            "id": "msg_mock",
            "role": "assistant",
            "content": [{
                "type": "output_text",
                "text": mock_payload(app.config.payload_bytes)
            }]
        }],
        "usage": {
            "input_tokens": 1,
            "output_tokens": app.config.chunks.max(1),
            "total_tokens": app.config.chunks.max(1) + 1
        }
    });
    record_request_completed(&app.metrics);
    axum::Json(payload).into_response()
}

fn build_chat_sse_response(app: App) -> Response {
    let metrics = Arc::clone(&app.metrics);
    let config = app.config.clone();
    let stream = async_stream::stream! {
        if !config.first_byte_delay.is_zero() {
            tokio::time::sleep(config.first_byte_delay).await;
        }
        for index in 0..config.chunks {
            if index > 0 && !config.chunk_delay.is_zero() {
                tokio::time::sleep(config.chunk_delay).await;
            }
            let payload = json!({
                "id": "chatcmpl-mock",
                "object": "chat.completion.chunk",
                "created": current_unix_secs(),
                "model": "mock-model",
                "choices": [{
                    "index": 0,
                    "delta": {
                        "content": mock_payload(config.payload_bytes)
                    },
                    "finish_reason": serde_json::Value::Null
                }]
            });
            yield Ok::<Bytes, Infallible>(Bytes::from(format!("data: {payload}\n\n")));
        }
        yield Ok::<Bytes, Infallible>(Bytes::from(
            "data: {\"choices\":[{\"index\":0,\"delta\":{},\"finish_reason\":\"stop\"}]}\n\n",
        ));
        yield Ok::<Bytes, Infallible>(Bytes::from("data: [DONE]\n\n"));
        record_request_completed(&metrics);
    };
    sse_response(Body::from_stream(stream))
}

fn build_responses_sse_response(app: App) -> Response {
    let metrics = Arc::clone(&app.metrics);
    let config = app.config.clone();
    let stream = async_stream::stream! {
        if !config.first_byte_delay.is_zero() {
            tokio::time::sleep(config.first_byte_delay).await;
        }
        yield Ok::<Bytes, Infallible>(Bytes::from(
            "event: response.created\ndata: {\"type\":\"response.created\",\"response\":{\"id\":\"resp_mock\",\"status\":\"in_progress\"}}\n\n",
        ));
        for index in 0..config.chunks {
            if index > 0 && !config.chunk_delay.is_zero() {
                tokio::time::sleep(config.chunk_delay).await;
            }
            let payload = json!({
                "type": "response.output_text.delta",
                "item_id": "msg_mock",
                "output_index": 0,
                "content_index": 0,
                "delta": mock_payload(config.payload_bytes)
            });
            yield Ok::<Bytes, Infallible>(Bytes::from(format!("event: response.output_text.delta\ndata: {payload}\n\n")));
        }
        yield Ok::<Bytes, Infallible>(Bytes::from(
            "event: response.completed\ndata: {\"type\":\"response.completed\",\"response\":{\"id\":\"resp_mock\",\"status\":\"completed\"}}\n\n",
        ));
        yield Ok::<Bytes, Infallible>(Bytes::from("data: [DONE]\n\n"));
        record_request_completed(&metrics);
    };
    sse_response(Body::from_stream(stream))
}

fn sse_response(body: Body) -> Response {
    let mut response = Response::new(body);
    let headers = response.headers_mut();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/event-stream; charset=utf-8"),
    );
    headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-cache"));
    response
}

fn request_wants_stream(body: &[u8]) -> bool {
    serde_json::from_slice::<serde_json::Value>(body)
        .ok()
        .and_then(|value| value.get("stream").and_then(serde_json::Value::as_bool))
        .unwrap_or(false)
}

fn request_model(body: &[u8]) -> Option<String> {
    serde_json::from_slice::<serde_json::Value>(body)
        .ok()
        .and_then(|value| {
            value
                .get("model")
                .and_then(serde_json::Value::as_str)
                .map(str::to_string)
        })
}

fn mock_payload(bytes: usize) -> String {
    if bytes == 0 {
        return String::new();
    }
    "x".repeat(bytes)
}

fn record_request_started(metrics: &Metrics) {
    metrics.requests_total.fetch_add(1, Ordering::AcqRel);
    let in_flight = metrics.in_flight.fetch_add(1, Ordering::AcqRel) + 1;
    metrics.max_in_flight.fetch_max(in_flight, Ordering::AcqRel);
}

fn record_request_completed(metrics: &Metrics) {
    metrics.completed_total.fetch_add(1, Ordering::AcqRel);
    metrics.in_flight.fetch_sub(1, Ordering::AcqRel);
}

fn current_unix_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

fn parse_args(args: Vec<String>) -> Result<Config, Box<dyn std::error::Error>> {
    let mut config = Config::default();
    let mut binds_overridden = false;
    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--bind" => {
                if !binds_overridden {
                    config.binds.clear();
                    binds_overridden = true;
                }
                config.binds.push(next_value(&mut iter, "--bind")?.parse()?);
            }
            "--chunks" => config.chunks = next_value(&mut iter, "--chunks")?.parse()?,
            "--first-byte-delay-ms" => {
                config.first_byte_delay =
                    Duration::from_millis(next_value(&mut iter, "--first-byte-delay-ms")?.parse()?)
            }
            "--chunk-delay-ms" => {
                config.chunk_delay =
                    Duration::from_millis(next_value(&mut iter, "--chunk-delay-ms")?.parse()?)
            }
            "--payload-bytes" => {
                config.payload_bytes = next_value(&mut iter, "--payload-bytes")?.parse()?
            }
            "--status" => {
                let status = next_value(&mut iter, "--status")?.parse::<u16>()?;
                config.status = StatusCode::from_u16(status)?;
            }
            "--help" | "-h" => {
                print_usage();
                std::process::exit(0);
            }
            other => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("unknown argument: {other}"),
                )
                .into());
            }
        }
    }
    if config.binds.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "at least one --bind is required",
        )
        .into());
    }
    Ok(config)
}

fn next_value(
    iter: &mut impl Iterator<Item = String>,
    flag: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    iter.next().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("missing value for {flag}"),
        )
        .into()
    })
}

fn print_usage() {
    eprintln!(
        "usage: cargo run -p aether-testkit --bin mock_openai_upstream -- [--bind 127.0.0.1:18181]... [--chunks 8] [--first-byte-delay-ms 0] [--chunk-delay-ms 20] [--payload-bytes 32] [--status 200]"
    );
}

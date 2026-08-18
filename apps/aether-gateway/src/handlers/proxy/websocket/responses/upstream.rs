//! Physical upstream WebSocket binding and transport helpers.

use std::time::Duration;

use serde_json::Value;
use wreq::ws::message::Message as WreqWsMessage;

use super::adapter::ResponsesWebSocketProtocolAdapter;
use super::binding::{UpstreamBindingIdentity, UpstreamBindingIdentityError};
use super::redaction::ResponsesWebSocketRedactionRestorer;
use super::request::planned_response_create_event;
use super::state::{BoundResponsesConnection, ExhaustedResponsesWebSocketExclusions};
use super::turn_state::ResponsesTurnState;
use crate::ai_serving::{AiExecutionDecision, ResponsesWebSocketBodyNormalization};
use crate::handlers::proxy::websocket::session::RESPONSES_WEBSOCKET_SESSION_LIMITS;
use crate::handlers::proxy::websocket::transport::{
    close_upstream_socket, connect_upstream_websocket, send_upstream_message,
};

/// 上游 WebSocket 握手的默认绝对 deadline（30 秒）。
/// 覆盖 DNS → TCP connect → TLS → HTTP 101 Upgrade → 发送首条 event 的完整链路。
/// 如果 decision 配置了更短的 first_byte_ms 或 total_ms，取其与此值的较小者。
const DEFAULT_UPSTREAM_HANDSHAKE_DEADLINE_MS: u64 = 30_000;

/// 从 decision.timeouts 推导实际 handshake 绝对 deadline。
/// 取 first_byte_ms / total_ms / DEFAULT 三者中的最小正值。
pub(super) fn resolve_upstream_handshake_deadline(decision: &AiExecutionDecision) -> Duration {
    let mut deadline_ms = DEFAULT_UPSTREAM_HANDSHAKE_DEADLINE_MS;
    if let Some(timeouts) = decision.timeouts.as_ref() {
        if let Some(first_byte_ms) = timeouts.first_byte_ms.filter(|v| *v > 0) {
            deadline_ms = deadline_ms.min(first_byte_ms);
        }
        if let Some(total_ms) = timeouts.total_ms.filter(|v| *v > 0) {
            deadline_ms = deadline_ms.min(total_ms);
        }
    }
    Duration::from_millis(deadline_ms)
}

pub(super) async fn bind_responses_upstream(
    decision: &AiExecutionDecision,
    normalization: ResponsesWebSocketBodyNormalization,
    initial_event: &Value,
    adapter: &'static dyn ResponsesWebSocketProtocolAdapter,
) -> Result<BoundResponsesConnection, &'static str> {
    // 绝对 deadline：从此刻起必须在限定时间内完成握手 + 首条事件发送，
    // 防止慢 TLS / 慢 HTTP Upgrade 无限占用 connection permit。
    let handshake_deadline = resolve_upstream_handshake_deadline(decision);
    tokio::time::timeout(
        handshake_deadline,
        bind_responses_upstream_inner(decision, normalization, initial_event, adapter),
    )
    .await
    .map_err(|_| "responses_websocket_upstream_handshake_timeout")?
}

/// 实际执行握手 + 首条事件发送的内部函数，由外层 timeout 包裹。
async fn bind_responses_upstream_inner(
    decision: &AiExecutionDecision,
    normalization: ResponsesWebSocketBodyNormalization,
    initial_event: &Value,
    adapter: &'static dyn ResponsesWebSocketProtocolAdapter,
) -> Result<BoundResponsesConnection, &'static str> {
    let binding_identity =
        UpstreamBindingIdentity::from_decision(adapter, decision).map_err(|error| match error {
            UpstreamBindingIdentityError::MissingUpstreamUrl => {
                adapter.upstream_errors().upstream_url_missing
            }
            UpstreamBindingIdentityError::InvalidUpstreamUrl => {
                adapter.upstream_errors().upstream_url_invalid
            }
            UpstreamBindingIdentityError::InvalidHandshakeHeaders => {
                adapter.upstream_errors().headers_invalid
            }
        })?;
    let mut upstream = connect_upstream_websocket(
        decision,
        RESPONSES_WEBSOCKET_SESSION_LIMITS,
        adapter.upstream_errors(),
    )
    .await?;
    let first_event = planned_response_create_event(decision, initial_event)?;
    send_upstream_message(&mut upstream.socket, WreqWsMessage::text(first_event))
        .await
        .map_err(|_| "responses_websocket_initial_send_failed")?;

    let client_model = initial_event
        .get("model")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or("responses_websocket_model_missing")?
        .to_string();
    let provider_model = decision
        .provider_request_body
        .as_ref()
        .and_then(|body| body.get("model"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .or_else(|| {
            decision
                .mapped_model
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
        })
        .ok_or("responses_websocket_mapped_model_missing")?
        .to_string();

    Ok(BoundResponsesConnection {
        upstream: Some(upstream.socket),
        adapter,
        client_model,
        provider_model,
        decision_template: decision.clone(),
        body_normalization: normalization,
        binding_identity,
        // 首条 response.create 已经发出，但这一轮的 logical turn 和 attempt 由调用方
        // 通过 `ResponsesTurnState::begin` 装上：绑定本身不持有记账状态。
        turn_state: ResponsesTurnState::Idle,
        // 同理，这一轮的 mask session 也由调用方登记：绑定看不到脱敏链路。
        redaction_restorer: ResponsesWebSocketRedactionRestorer::default(),
        next_turn_index: 2,
        upstream_response_headers: upstream.response_headers,
        pending_adapter_drain: None,
        pending_adapter_observation: None,
        exhausted_exclusions: ExhaustedResponsesWebSocketExclusions::default(),
        pending_turn_finalization: None,
    })
}

pub(super) async fn receive_optional_upstream(
    upstream: &mut Option<wreq::ws::WebSocket>,
) -> Option<Result<WreqWsMessage, ()>> {
    match upstream.as_mut() {
        Some(upstream) => upstream.recv().await.map(|message| message.map_err(|_| ())),
        None => std::future::pending().await,
    }
}

pub(super) async fn close_bound_upstream(bound: &mut BoundResponsesConnection) {
    if let Some(mut upstream) = bound.upstream.take() {
        close_upstream_socket(&mut upstream, None).await;
    }
}

pub(super) fn decision_reuses_bound_upstream(
    bound: &BoundResponsesConnection,
    adapter: &'static dyn ResponsesWebSocketProtocolAdapter,
    decision: &AiExecutionDecision,
) -> bool {
    bound.upstream.is_some()
        && UpstreamBindingIdentity::from_decision(adapter, decision)
            .map(|identity| bound.binding_identity == identity)
            .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use aether_contracts::ExecutionTimeouts;

    use crate::ai_serving::AiExecutionDecision;

    use super::{resolve_upstream_handshake_deadline, DEFAULT_UPSTREAM_HANDSHAKE_DEADLINE_MS};

    fn sample_decision() -> AiExecutionDecision {
        AiExecutionDecision {
            action: "local".to_string(),
            decision_kind: None,
            execution_strategy: None,
            conversion_mode: None,
            request_id: None,
            candidate_id: None,
            provider_name: None,
            provider_type: Some("custom".to_string()),
            provider_id: None,
            endpoint_id: None,
            key_id: None,
            upstream_base_url: None,
            upstream_url: Some("https://example.test/v1/responses".to_string()),
            provider_request_method: None,
            auth_header: None,
            auth_value: None,
            provider_api_format: Some("openai:responses".to_string()),
            client_api_format: Some("openai:responses".to_string()),
            provider_contract: None,
            client_contract: None,
            model_name: None,
            mapped_model: Some("provider-model".to_string()),
            prompt_cache_key: None,
            extra_headers: std::collections::BTreeMap::new(),
            provider_request_headers: std::collections::BTreeMap::new(),
            provider_request_body: None,
            provider_request_body_base64: None,
            content_type: None,
            content_encoding: None,
            request_gzip: None,
            proxy: None,
            transport_profile: None,
            timeouts: None,
            upstream_is_stream: true,
            report_kind: None,
            report_context: None,
            auth_context: None,
        }
    }

    #[test]
    fn handshake_deadline_defaults_to_30s_without_configured_timeouts() {
        let decision = sample_decision();
        let deadline = resolve_upstream_handshake_deadline(&decision);
        assert_eq!(
            deadline,
            Duration::from_millis(DEFAULT_UPSTREAM_HANDSHAKE_DEADLINE_MS)
        );
    }

    #[test]
    fn handshake_deadline_uses_first_byte_ms_when_shorter_than_default() {
        let mut decision = sample_decision();
        decision.timeouts = Some(ExecutionTimeouts {
            first_byte_ms: Some(10_000),
            total_ms: Some(60_000),
            ..ExecutionTimeouts::default()
        });
        let deadline = resolve_upstream_handshake_deadline(&decision);
        assert_eq!(deadline, Duration::from_millis(10_000));
    }

    #[test]
    fn handshake_deadline_uses_total_ms_when_shorter_than_first_byte_and_default() {
        let mut decision = sample_decision();
        decision.timeouts = Some(ExecutionTimeouts {
            first_byte_ms: Some(25_000),
            total_ms: Some(8_000),
            ..ExecutionTimeouts::default()
        });
        let deadline = resolve_upstream_handshake_deadline(&decision);
        assert_eq!(deadline, Duration::from_millis(8_000));
    }

    #[test]
    fn handshake_deadline_ignores_zero_values() {
        let mut decision = sample_decision();
        decision.timeouts = Some(ExecutionTimeouts {
            first_byte_ms: Some(0),
            total_ms: Some(0),
            ..ExecutionTimeouts::default()
        });
        let deadline = resolve_upstream_handshake_deadline(&decision);
        assert_eq!(
            deadline,
            Duration::from_millis(DEFAULT_UPSTREAM_HANDSHAKE_DEADLINE_MS)
        );
    }

    #[test]
    fn handshake_deadline_does_not_exceed_default_even_with_larger_configured_values() {
        let mut decision = sample_decision();
        decision.timeouts = Some(ExecutionTimeouts {
            first_byte_ms: Some(120_000),
            total_ms: Some(600_000),
            ..ExecutionTimeouts::default()
        });
        let deadline = resolve_upstream_handshake_deadline(&decision);
        assert_eq!(
            deadline,
            Duration::from_millis(DEFAULT_UPSTREAM_HANDSHAKE_DEADLINE_MS)
        );
    }

    #[tokio::test]
    async fn bind_responses_upstream_times_out_against_stalled_server() {
        use super::bind_responses_upstream;
        use crate::ai_serving::ResponsesWebSocketBodyNormalization;
        use crate::handlers::proxy::websocket::responses::adapter::resolve_responses_websocket_adapter;
        use serde_json::json;

        // 启动一个接受 TCP 连接但永不完成 HTTP Upgrade 的 mock 服务器
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("mock listener should bind");
        let addr = listener.local_addr().expect("should have local addr");
        let _server = tokio::spawn(async move {
            loop {
                let (socket, _) = listener.accept().await.unwrap();
                // 接受连接但不发送任何 HTTP 响应，模拟 stalled handshake
                tokio::spawn(async move {
                    let _hold = socket;
                    tokio::time::sleep(Duration::from_secs(300)).await;
                });
            }
        });

        let mut decision = sample_decision();
        decision.upstream_url = Some(format!("http://{addr}/v1/responses"));
        // 设置极短的 deadline 以便测试快速完成
        decision.timeouts = Some(ExecutionTimeouts {
            first_byte_ms: Some(100),
            total_ms: Some(200),
            ..ExecutionTimeouts::default()
        });
        decision.provider_request_body = Some(json!({"model": "test-model"}));

        let adapter = resolve_responses_websocket_adapter(
            crate::orchestration::ResponsesWebSocketAdapter::Standard,
        );
        let result = bind_responses_upstream(
            &decision,
            ResponsesWebSocketBodyNormalization::for_tests("test-model"),
            &json!({"type": "response.create", "model": "test-model"}),
            adapter,
        )
        .await;

        assert_eq!(
            result.err().expect("bind should fail with timeout"),
            "responses_websocket_upstream_handshake_timeout"
        );
    }
}

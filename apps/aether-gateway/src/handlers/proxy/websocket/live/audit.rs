//! Session-level audit records for Codex Live transports.
//!
//! Frameless Bidi does not expose an authoritative token/cost usage object.
//! These records therefore capture exactly one bounded lifecycle summary per
//! connection and are explicitly void for billing. They never infer tokens,
//! audio duration, or cost from frame sizes.

use std::time::Duration;

use aether_ai_serving::AiStreamAttempt;
use aether_contracts::ExecutionPlan;
use aether_data_contracts::repository::usage::{
    LIVE_SESSION_METADATA_KEY, USAGE_AVAILABLE_METADATA_KEY, USAGE_PRICING_AVAILABLE_METADATA_KEY,
    WEBSOCKET_MODE_METADATA_KEY, WEBSOCKET_TRANSPORT_METADATA_KEY,
};
use aether_usage_runtime::build_usage_event_data_seed;
use serde_json::{json, Map, Value};
use tracing::warn;

use crate::usage::{UsageEvent, UsageEventData, UsageEventType};
use crate::AppState;

const LIVE_AUDIT_WRITE_WAIT: Duration = Duration::from_secs(5);
const LIVE_AUDIT_SCHEMA_VERSION: &str = "1";
const LIVE_AUDIT_LOG_TARGET: &str = "aether_gateway::handlers::proxy::codex_live";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum LiveAuditTransport {
    WebRtc,
    DirectWebSocket,
    Sideband,
}

impl LiveAuditTransport {
    const fn transport(self) -> &'static str {
        match self {
            Self::WebRtc => "webrtc",
            Self::DirectWebSocket => "websocket",
            Self::Sideband => "sideband",
        }
    }

    const fn mode(self) -> &'static str {
        match self {
            Self::WebRtc => "call_create",
            Self::DirectWebSocket => "direct",
            Self::Sideband => "sideband",
        }
    }

    const fn websocket_transport(self) -> Option<&'static str> {
        match self {
            Self::WebRtc => None,
            Self::DirectWebSocket => Some("codex_live_direct"),
            Self::Sideband => Some("codex_live_sideband"),
        }
    }
}

/// Marks the existing synchronous SDP call-create audit row as an unmetered
/// WebRTC control exchange. The media leg bypasses Aether after this request.
pub(super) fn mark_live_call_create_report_context(report_context: &mut Option<Value>) {
    attach_live_base_metadata(report_context, LiveAuditTransport::WebRtc);
}

fn attach_live_base_metadata(report_context: &mut Option<Value>, transport: LiveAuditTransport) {
    let object = report_context_object(report_context);
    object.insert(USAGE_AVAILABLE_METADATA_KEY.to_string(), Value::Bool(false));
    object.insert(
        USAGE_PRICING_AVAILABLE_METADATA_KEY.to_string(),
        Value::Bool(false),
    );
    object.insert(
        WEBSOCKET_MODE_METADATA_KEY.to_string(),
        Value::Bool(transport.websocket_transport().is_some()),
    );
    if let Some(websocket_transport) = transport.websocket_transport() {
        object.insert(
            WEBSOCKET_TRANSPORT_METADATA_KEY.to_string(),
            Value::String(websocket_transport.to_string()),
        );
    } else {
        object.remove(WEBSOCKET_TRANSPORT_METADATA_KEY);
    }
    object.insert(
        LIVE_SESSION_METADATA_KEY.to_string(),
        json!({
            "schema_version": LIVE_AUDIT_SCHEMA_VERSION,
            "transport": transport.transport(),
            "mode": transport.mode(),
            "usage_state": "unavailable",
        }),
    );
}

fn report_context_object(report_context: &mut Option<Value>) -> &mut Map<String, Value> {
    if !matches!(report_context, Some(Value::Object(_))) {
        let seed = report_context.take();
        let mut object = Map::new();
        if let Some(seed) = seed.filter(|value| !value.is_null()) {
            object.insert("seed".to_string(), seed);
        }
        *report_context = Some(Value::Object(object));
    }
    report_context
        .as_mut()
        .and_then(Value::as_object_mut)
        .expect("Live audit report context was normalized to an object")
}

pub(super) struct LiveSessionAudit {
    plan: ExecutionPlan,
    report_context: Option<Value>,
    transport: LiveAuditTransport,
}

impl LiveSessionAudit {
    pub(super) fn from_attempt(attempt: &AiStreamAttempt, transport: LiveAuditTransport) -> Self {
        let mut report_context = attempt.report_context.clone();
        attach_live_base_metadata(&mut report_context, transport);
        Self {
            plan: attempt.plan.clone(),
            report_context,
            transport,
        }
    }

    /// Persists one terminal lifecycle row. The spawned write remains alive if
    /// the bounded caller wait elapses, so closing a socket cannot silently
    /// cancel the only audit write for that connection.
    pub(super) async fn finish(self, state: &AppState, terminal: LiveSessionTerminal) {
        let request_id = self.plan.request_id.clone();
        let event = self.build_terminal_event(terminal);
        let usage_runtime = std::sync::Arc::clone(&state.usage_runtime);
        let usage_data = std::sync::Arc::clone(state.usage_lifecycle_data_state());
        let task = tokio::spawn(async move {
            usage_runtime
                .record_terminal_event_direct(usage_data.as_ref(), event)
                .await;
        });
        match tokio::time::timeout(LIVE_AUDIT_WRITE_WAIT, task).await {
            Ok(Ok(())) => {}
            Ok(Err(error)) => warn!(
                target: LIVE_AUDIT_LOG_TARGET,
                event_name = "codex_live_session_audit_task_failed",
                log_type = "ops",
                request_id,
                error = %error,
                "Codex Live session audit task failed"
            ),
            Err(_) => warn!(
                target: LIVE_AUDIT_LOG_TARGET,
                event_name = "codex_live_session_audit_write_slow",
                log_type = "ops",
                request_id,
                wait_ms = LIVE_AUDIT_WRITE_WAIT.as_millis() as u64,
                write_detached = true,
                "Codex Live stopped waiting for a slow session audit write"
            ),
        }
    }

    fn build_terminal_event(self, terminal: LiveSessionTerminal) -> UsageEvent {
        let mut data = build_usage_event_data_seed(&self.plan, self.report_context.as_ref());
        data.request_type = Some("live".to_string());
        data.is_stream = Some(self.transport != LiveAuditTransport::WebRtc);
        data.status_code = Some(terminal.status_code);
        data.response_time_ms = Some(terminal.elapsed_ms);
        data.first_byte_time_ms = terminal.first_upstream_frame_ms;
        data.input_tokens = None;
        data.output_tokens = None;
        data.total_tokens = None;
        data.cache_creation_input_tokens = None;
        data.cache_creation_ephemeral_5m_input_tokens = None;
        data.cache_creation_ephemeral_1h_input_tokens = None;
        data.cache_read_input_tokens = None;
        data.cache_creation_cost_usd = None;
        data.cache_read_cost_usd = None;
        data.total_cost_usd = None;
        data.actual_total_cost_usd = None;
        if terminal.disposition != LiveSessionDisposition::Completed {
            data.error_message = Some(terminal.termination.to_string());
            data.error_category = Some(terminal.disposition.error_category().to_string());
        }
        data.request_metadata =
            attach_terminal_metadata(data.request_metadata, self.transport, &terminal);
        UsageEvent::new(
            terminal.disposition.event_type(),
            self.plan.request_id,
            data,
        )
    }
}

fn attach_terminal_metadata(
    metadata: Option<Value>,
    transport: LiveAuditTransport,
    terminal: &LiveSessionTerminal,
) -> Option<Value> {
    let mut object = match metadata {
        Some(Value::Object(object)) => object,
        _ => Map::new(),
    };
    object.insert(USAGE_AVAILABLE_METADATA_KEY.to_string(), Value::Bool(false));
    object.insert(
        USAGE_PRICING_AVAILABLE_METADATA_KEY.to_string(),
        Value::Bool(false),
    );
    object.insert(
        WEBSOCKET_MODE_METADATA_KEY.to_string(),
        Value::Bool(transport.websocket_transport().is_some()),
    );
    if let Some(websocket_transport) = transport.websocket_transport() {
        object.insert(
            WEBSOCKET_TRANSPORT_METADATA_KEY.to_string(),
            Value::String(websocket_transport.to_string()),
        );
    }
    object.insert(
        LIVE_SESSION_METADATA_KEY.to_string(),
        json!({
            "schema_version": LIVE_AUDIT_SCHEMA_VERSION,
            "transport": transport.transport(),
            "mode": transport.mode(),
            "state": terminal.disposition.state(),
            "termination": terminal.termination,
            "elapsed_ms": terminal.elapsed_ms,
            "client_frames": terminal.client_frames,
            "client_bytes": terminal.client_bytes,
            "upstream_frames": terminal.upstream_frames,
            "upstream_bytes": terminal.upstream_bytes,
            "first_upstream_frame_ms": terminal.first_upstream_frame_ms,
            "usage_state": "unavailable",
        }),
    );
    Some(Value::Object(object))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum LiveSessionDisposition {
    Completed,
    Failed,
    Cancelled,
}

impl LiveSessionDisposition {
    const fn event_type(self) -> UsageEventType {
        match self {
            Self::Completed => UsageEventType::Completed,
            Self::Failed => UsageEventType::Failed,
            Self::Cancelled => UsageEventType::Cancelled,
        }
    }

    const fn state(self) -> &'static str {
        match self {
            Self::Completed => "closed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }

    const fn error_category(self) -> &'static str {
        match self {
            Self::Completed => "none",
            Self::Failed => "transport_error",
            Self::Cancelled => "client_cancelled",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) struct LiveSessionTerminal {
    pub(super) disposition: LiveSessionDisposition,
    pub(super) status_code: u16,
    pub(super) termination: &'static str,
    pub(super) elapsed_ms: u64,
    pub(super) first_upstream_frame_ms: Option<u64>,
    pub(super) client_frames: u64,
    pub(super) client_bytes: u64,
    pub(super) upstream_frames: u64,
    pub(super) upstream_bytes: u64,
}

impl LiveSessionTerminal {
    pub(super) const fn failure(
        status_code: u16,
        termination: &'static str,
        elapsed_ms: u64,
    ) -> Self {
        Self {
            disposition: LiveSessionDisposition::Failed,
            status_code,
            termination,
            elapsed_ms,
            first_upstream_frame_ms: None,
            client_frames: 0,
            client_bytes: 0,
            upstream_frames: 0,
            upstream_bytes: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use aether_contracts::{ExecutionTimeouts, RequestBody};

    use super::*;

    fn sample_attempt() -> AiStreamAttempt {
        AiStreamAttempt {
            plan: ExecutionPlan {
                request_id: "live-request".to_string(),
                candidate_id: Some("candidate-live".to_string()),
                provider_name: Some("Codex".to_string()),
                provider_id: "provider-live".to_string(),
                endpoint_id: "endpoint-live".to_string(),
                key_id: "key-live".to_string(),
                method: "GET".to_string(),
                url: "wss://example.test/v1/live".to_string(),
                headers: BTreeMap::new(),
                content_type: None,
                content_encoding: None,
                body: RequestBody {
                    json_body: None,
                    body_bytes_b64: None,
                    body_ref: None,
                },
                stream: true,
                client_api_format: "codex:live".to_string(),
                provider_api_format: "codex:live".to_string(),
                model_name: Some("gpt-live".to_string()),
                proxy: None,
                transport_profile: None,
                timeouts: Some(ExecutionTimeouts::default()),
            },
            report_kind: Some("openai_responses_stream".to_string()),
            report_context: Some(json!({
                "user_id": "user-live",
                "api_key_id": "gateway-key-live",
                "trace_id": "trace-live"
            })),
        }
    }

    #[test]
    fn direct_terminal_audit_is_opaque_unmetered_and_void_eligible() {
        let audit =
            LiveSessionAudit::from_attempt(&sample_attempt(), LiveAuditTransport::DirectWebSocket);
        let event = audit.build_terminal_event(LiveSessionTerminal {
            disposition: LiveSessionDisposition::Completed,
            status_code: 200,
            termination: "client_close_frame",
            elapsed_ms: 1234,
            first_upstream_frame_ms: Some(42),
            client_frames: 3,
            client_bytes: 128,
            upstream_frames: 5,
            upstream_bytes: 512,
        });

        assert_eq!(event.event_type, UsageEventType::Completed);
        assert_eq!(event.data.input_tokens, None);
        assert_eq!(event.data.total_cost_usd, None);
        let metadata = event.data.request_metadata.expect("metadata");
        assert_eq!(metadata[USAGE_AVAILABLE_METADATA_KEY], false);
        assert_eq!(metadata[USAGE_PRICING_AVAILABLE_METADATA_KEY], false);
        assert_eq!(metadata[WEBSOCKET_MODE_METADATA_KEY], true);
        assert_eq!(
            metadata[WEBSOCKET_TRANSPORT_METADATA_KEY],
            "codex_live_direct"
        );
        assert_eq!(metadata[LIVE_SESSION_METADATA_KEY]["client_frames"], 3);
        assert_eq!(
            metadata[LIVE_SESSION_METADATA_KEY]["usage_state"],
            "unavailable"
        );
    }

    #[test]
    fn call_create_is_webrtc_not_websocket() {
        let mut context = Some(json!({"trace_id": "trace-live"}));
        mark_live_call_create_report_context(&mut context);
        let context = context.expect("context");

        assert_eq!(context[USAGE_AVAILABLE_METADATA_KEY], false);
        assert_eq!(context[USAGE_PRICING_AVAILABLE_METADATA_KEY], false);
        assert_eq!(context[WEBSOCKET_MODE_METADATA_KEY], false);
        assert!(context.get(WEBSOCKET_TRANSPORT_METADATA_KEY).is_none());
        assert_eq!(context[LIVE_SESSION_METADATA_KEY]["transport"], "webrtc");
    }
}

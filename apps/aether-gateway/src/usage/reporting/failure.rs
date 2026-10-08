use aether_usage_runtime::{
    stream_report_missing_terminal_event, stream_report_represents_failure,
    sync_report_represents_failure, GatewayStreamReportRequest, GatewaySyncReportRequest,
};
use serde_json::{json, Map, Value};

pub(crate) fn execution_error_analytics_context(
    context: Option<&Value>,
    error: &aether_contracts::ExecutionError,
) -> Option<Value> {
    use aether_contracts::{ExecutionErrorKind as Kind, ExecutionPhase as Phase};
    let stage = match error.phase {
        Phase::Connect => "connect",
        Phase::Handshake => "handshake",
        Phase::Write => "request_write",
        Phase::FirstByte => "first_byte",
        Phase::StreamRead => "stream_read",
        Phase::Decode => "decode",
        Phase::Finalize => "finalize",
    };
    let (origin, reason) = match error.kind {
        Kind::ConnectTimeout => ("transport", "connect_timeout"),
        Kind::FirstByteTimeout => ("upstream", "first_byte_timeout"),
        Kind::ReadTimeout => ("transport", "read_timeout"),
        Kind::TlsError => ("transport", "tls_error"),
        Kind::ProxyError => ("transport", "proxy_error"),
        Kind::Upstream4xx | Kind::Upstream5xx => ("upstream", "upstream_response_error"),
        Kind::ProtocolError => ("upstream", "protocol_error"),
        Kind::Internal => ("gateway", "execution_internal_error"),
        Kind::Cancelled => ("unknown", "execution_cancelled"),
    };
    with_analytics_failure(context, origin, stage, reason)
}

pub(crate) fn with_analytics_failure(
    context: Option<&Value>,
    origin: &'static str,
    stage: &'static str,
    reason: &'static str,
) -> Option<Value> {
    let mut object = context
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    object.insert(
        "analytics_failure".into(),
        json!({
            "origin": origin, "stage": stage, "reason": reason, "schema_version": 1,
        }),
    );
    Some(Value::Object(object))
}

fn normalized_failure_context(context: Option<&Value>, failed: bool) -> Map<String, Value> {
    let mut object = context
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    if !failed {
        // A successful retry supersedes a previous candidate's failure classification.
        object.remove("analytics_failure");
        object.remove("error_flow");
        object.remove("transport_error");
    }
    object
}

fn classify_observed_failure(object: &Map<String, Value>, stage: &'static str) -> Option<Value> {
    if object
        .get("analytics_failure")
        .and_then(Value::as_object)
        .is_some()
    {
        return Some(Value::Object(object.clone()));
    }
    if object.get("transport_error").and_then(Value::as_bool) == Some(true) {
        return with_analytics_failure(
            Some(&Value::Object(object.clone())),
            "transport",
            stage,
            "upstream_transport_error",
        );
    }
    if object.get("error_flow").is_some_and(|flow| {
        flow.get("source").and_then(Value::as_str) == Some("upstream_response")
            && flow
                .get("status_code")
                .and_then(Value::as_u64)
                .is_some_and(|status| status >= 400)
    }) || object
        .get("upstream_response")
        .and_then(|value| value.get("status_code"))
        .and_then(Value::as_u64)
        .is_some_and(|status| status >= 400)
    {
        return with_analytics_failure(
            Some(&Value::Object(object.clone())),
            "upstream",
            stage,
            "upstream_response_error",
        );
    }
    None
}

pub(crate) fn sync_analytics_context(
    context: Option<&Value>,
    payload: &GatewaySyncReportRequest,
) -> Option<Value> {
    let failed = sync_report_represents_failure(payload, None);
    let object = normalized_failure_context(context, failed);
    if !failed {
        return Some(Value::Object(object));
    }
    classify_observed_failure(&object, "response").or(Some(Value::Object(object)))
}

pub(crate) fn stream_analytics_context(
    context: Option<&Value>,
    payload: &GatewayStreamReportRequest,
    downstream_cancelled: bool,
) -> Option<Value> {
    if downstream_cancelled {
        return with_analytics_failure(context, "client", "delivery", "downstream_disconnect");
    }
    let failed = stream_report_represents_failure(payload);
    let object = normalized_failure_context(context, failed);
    if !failed {
        return Some(Value::Object(object));
    }
    if let Some(classified) = classify_observed_failure(&object, "stream_read") {
        return Some(classified);
    }
    if payload
        .terminal_summary
        .as_ref()
        .is_some_and(|summary| summary.parser_error.is_some())
    {
        return with_analytics_failure(
            Some(&Value::Object(object)),
            "gateway",
            "decode",
            "response_decode_error",
        );
    }
    if stream_report_missing_terminal_event(payload) {
        return with_analytics_failure(
            Some(&Value::Object(object)),
            "upstream",
            "stream_read",
            "missing_terminal_event",
        );
    }
    Some(Value::Object(object))
}

pub(crate) fn gateway_error_analytics_context(
    context: Option<&Value>,
    error: &crate::GatewayError,
) -> Option<Value> {
    match error {
        crate::GatewayError::AdmissionTimeout { .. } => {
            with_analytics_failure(context, "gateway", "admission", "gateway_admission_timeout")
        }
        crate::GatewayError::LocalExecutionPlanningTimeout { .. } => {
            with_analytics_failure(context, "gateway", "routing", "planning_timeout")
        }
        crate::GatewayError::PlanUsageLimited(_) => {
            with_analytics_failure(context, "client", "admission", "quota_exceeded")
        }
        crate::GatewayError::UpstreamUnavailable { .. } => {
            with_analytics_failure(context, "upstream", "connect", "upstream_unavailable")
        }
        crate::GatewayError::ControlUnavailable { .. } => {
            with_analytics_failure(context, "gateway", "routing", "control_unavailable")
        }
        crate::GatewayError::Internal(_) => {
            with_analytics_failure(context, "gateway", "finalize", "internal_error")
        }
        _ => context.cloned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn sync_payload(status: u16) -> GatewaySyncReportRequest {
        GatewaySyncReportRequest {
            trace_id: "t".into(),
            report_kind: "openai_chat_sync_success".into(),
            report_context: None,
            status_code: status,
            headers: BTreeMap::new(),
            body_json: None,
            client_body_json: None,
            body_base64: None,
            telemetry: None,
        }
    }

    #[test]
    fn analytics_failure_preserves_upstream_auth_and_throttling_as_service_failures() {
        for status in [401, 429] {
            let context =
                json!({"error_flow": {"source": "upstream_response", "status_code": status}});
            let result = sync_analytics_context(Some(&context), &sync_payload(status)).unwrap();
            assert_eq!(result["analytics_failure"]["origin"], "upstream");
            assert_eq!(result["analytics_failure"]["schema_version"], 1);
            let unknown = sync_analytics_context(Some(&json!({})), &sync_payload(status)).unwrap();
            assert!(unknown.get("analytics_failure").is_none());
        }
    }

    #[test]
    fn analytics_failure_successful_retry_clears_previous_observations() {
        let context = json!({"request_id": "keep", "analytics_failure": {"origin": "upstream"}, "error_flow": {"source": "upstream_response"}, "transport_error": true});
        let result = sync_analytics_context(Some(&context), &sync_payload(200)).unwrap();
        assert_eq!(result["request_id"], "keep");
        assert!(result.get("analytics_failure").is_none());
        assert!(result.get("transport_error").is_none());
    }

    #[test]
    fn analytics_failure_transport_uses_observation_and_not_public_error_text() {
        let context =
            json!({"transport_error": true, "error_flow": {"source": "upstream_response"}});
        let result = sync_analytics_context(Some(&context), &sync_payload(502)).unwrap();
        assert_eq!(result["analytics_failure"]["origin"], "transport");
        assert_eq!(
            result["analytics_failure"]["reason"],
            "upstream_transport_error"
        );
    }

    #[test]
    fn analytics_failure_gateway_admission_is_not_client_quota() {
        let error = crate::GatewayError::AdmissionTimeout {
            trace_id: "t".into(),
            gate: "upstream",
            queue_budget_ms: 20,
        };
        let result = gateway_error_analytics_context(None, &error).unwrap();
        assert_eq!(result["analytics_failure"]["origin"], "gateway");
        assert_eq!(result["analytics_failure"]["stage"], "admission");
    }

    #[test]
    fn analytics_failure_structured_error_has_priority_over_http_diagnostic() {
        let context = json!({"error_flow": {"source": "upstream_response", "status_code": 502}});
        let error = aether_contracts::ExecutionError {
            kind: aether_contracts::ExecutionErrorKind::Internal,
            phase: aether_contracts::ExecutionPhase::Decode,
            message: "secret diagnostic".into(),
            upstream_status: None,
            retryable: false,
            failover_recommended: false,
        };
        let context = execution_error_analytics_context(Some(&context), &error).unwrap();
        let result = sync_analytics_context(Some(&context), &sync_payload(502)).unwrap();
        assert_eq!(result["analytics_failure"]["origin"], "gateway");
        assert_eq!(result["analytics_failure"]["stage"], "decode");
        assert!(!result.to_string().contains("secret diagnostic"));
    }

    #[test]
    fn analytics_failure_downstream_disconnect_is_distinct_from_unclassified_cancellation() {
        let payload = GatewayStreamReportRequest {
            trace_id: "t".into(),
            report_kind: "openai_chat_stream_error".into(),
            report_context: None,
            status_code: 499,
            headers: BTreeMap::new(),
            provider_body_base64: None,
            provider_body_state: None,
            client_body_base64: None,
            client_body_state: None,
            terminal_summary: None,
            telemetry: None,
        };
        let result = stream_analytics_context(None, &payload, true).unwrap();
        assert_eq!(result["analytics_failure"]["origin"], "client");
        assert_eq!(
            result["analytics_failure"]["reason"],
            "downstream_disconnect"
        );
        let unknown = sync_analytics_context(None, &sync_payload(499)).unwrap();
        assert!(unknown.get("analytics_failure").is_none());
    }
}

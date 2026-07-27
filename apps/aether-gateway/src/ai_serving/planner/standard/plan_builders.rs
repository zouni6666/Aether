use std::collections::BTreeMap;

use super::{
    augment_sync_report_context, build_ai_execution_plan_from_decision,
    resolve_ai_passthrough_sync_request_body, take_ai_decision_plan_core,
    take_ai_upstream_auth_pair, take_non_empty_string, AiExecutionPlanFromDecisionParts,
    AiStreamAttempt, AiSyncAttempt,
};
use crate::ai_serving::transport::{
    build_standard_plan_fallback_headers, StandardPlanFallbackAcceptPolicy,
    StandardPlanFallbackHeadersInput,
};
use crate::ai_serving::{
    generic_decision_missing_exact_provider_request,
    provider_adaptation_requires_eventstream_accept,
};
use crate::{AiExecutionDecision, GatewayError};

pub(crate) fn build_standard_sync_plan_from_decision(
    parts: &http::request::Parts,
    _body_json: &serde_json::Value,
    payload: AiExecutionDecision,
) -> Result<Option<AiSyncAttempt>, GatewayError> {
    let mut payload = payload;
    if generic_decision_missing_exact_provider_request(&payload) {
        return Ok(None);
    }
    let Some(core) = take_ai_decision_plan_core(&mut payload) else {
        return Ok(None);
    };
    let Some(url) = take_non_empty_string(&mut payload.upstream_url) else {
        return Ok(None);
    };
    let Some(auth_pair) = take_ai_upstream_auth_pair(&mut payload) else {
        return Ok(None);
    };
    let Some(provider_request_body_value) = payload.provider_request_body.take() else {
        return Ok(None);
    };
    let mut provider_request_headers =
        build_standard_plan_fallback_headers(StandardPlanFallbackHeadersInput {
            request_headers: &parts.headers,
            existing_provider_request_headers: std::mem::take(
                &mut payload.provider_request_headers,
            ),
            auth_header: auth_pair.as_ref().map(|pair| pair.header.as_str()),
            auth_value: auth_pair.as_ref().map(|pair| pair.value.as_str()),
            extra_headers: &BTreeMap::new(),
            content_type: payload.content_type.as_deref(),
            provider_api_format: core.provider_api_format.as_str(),
            client_api_format: core.client_api_format.as_str(),
            upstream_is_stream: payload.upstream_is_stream,
            build_from_request_when_empty: false,
            accept_policy: StandardPlanFallbackAcceptPolicy::TextEventStreamIfStreaming,
        });
    let content_type = payload
        .content_type
        .take()
        .or_else(|| Some("application/json".to_string()));
    let report_context = augment_sync_report_context(
        payload.report_context.take(),
        &provider_request_headers,
        &provider_request_body_value,
    )?;
    let request_body = resolve_ai_passthrough_sync_request_body(
        Some(provider_request_body_value),
        payload.provider_request_body_base64.take(),
    );
    let stream = payload.upstream_is_stream;
    let plan = build_ai_execution_plan_from_decision(
        &mut payload,
        AiExecutionPlanFromDecisionParts {
            core,
            method: "POST".to_string(),
            url,
            headers: std::mem::take(&mut provider_request_headers),
            content_type,
            body: request_body,
            stream,
        },
    );

    Ok(Some(AiSyncAttempt {
        plan,
        report_kind: payload.report_kind,
        report_context,
    }))
}

pub(crate) fn build_standard_stream_plan_from_decision(
    parts: &http::request::Parts,
    _body_json: &serde_json::Value,
    payload: AiExecutionDecision,
    _inject_stream_flag: bool,
) -> Result<Option<AiStreamAttempt>, GatewayError> {
    let mut payload = payload;
    if generic_decision_missing_exact_provider_request(&payload) {
        return Ok(None);
    }
    let Some(core) = take_ai_decision_plan_core(&mut payload) else {
        return Ok(None);
    };
    let Some(url) = take_non_empty_string(&mut payload.upstream_url) else {
        return Ok(None);
    };
    let Some(auth_pair) = take_ai_upstream_auth_pair(&mut payload) else {
        return Ok(None);
    };
    let Some(provider_request_body_value) = payload.provider_request_body.take() else {
        return Ok(None);
    };

    let envelope_name = payload
        .report_context
        .as_ref()
        .and_then(|context| context.get("envelope_name"))
        .and_then(serde_json::Value::as_str);
    let accept_policy = if payload.upstream_is_stream
        && provider_adaptation_requires_eventstream_accept(
            envelope_name,
            core.provider_api_format.as_str(),
        ) {
        StandardPlanFallbackAcceptPolicy::ProviderEventStreamIfMissing
    } else {
        StandardPlanFallbackAcceptPolicy::TextEventStreamIfStreaming
    };
    let mut provider_request_headers =
        build_standard_plan_fallback_headers(StandardPlanFallbackHeadersInput {
            request_headers: &parts.headers,
            existing_provider_request_headers: std::mem::take(
                &mut payload.provider_request_headers,
            ),
            auth_header: auth_pair.as_ref().map(|pair| pair.header.as_str()),
            auth_value: auth_pair.as_ref().map(|pair| pair.value.as_str()),
            extra_headers: &BTreeMap::new(),
            content_type: payload.content_type.as_deref(),
            provider_api_format: core.provider_api_format.as_str(),
            client_api_format: core.client_api_format.as_str(),
            upstream_is_stream: payload.upstream_is_stream,
            build_from_request_when_empty: false,
            accept_policy,
        });
    let content_type = payload
        .content_type
        .take()
        .or_else(|| Some("application/json".to_string()));
    let report_context = augment_sync_report_context(
        payload.report_context.take(),
        &provider_request_headers,
        &provider_request_body_value,
    )?;
    let request_body = resolve_ai_passthrough_sync_request_body(
        Some(provider_request_body_value),
        payload.provider_request_body_base64.take(),
    );
    let stream = payload.upstream_is_stream;
    let plan = build_ai_execution_plan_from_decision(
        &mut payload,
        AiExecutionPlanFromDecisionParts {
            core,
            method: "POST".to_string(),
            url,
            headers: std::mem::take(&mut provider_request_headers),
            content_type,
            body: request_body,
            stream,
        },
    );

    Ok(Some(AiStreamAttempt {
        plan,
        report_kind: payload.report_kind,
        report_context,
    }))
}

#[cfg(test)]
mod tests {
    use aether_contracts::{ExecutionResponseBodyMode, EXECUTION_RESPONSE_BODY_MODE_HEADER};
    use serde_json::json;

    use super::{
        build_standard_stream_plan_from_decision, build_standard_sync_plan_from_decision,
        AiExecutionDecision,
    };

    fn decision_with_raw_body(upstream_is_stream: bool) -> AiExecutionDecision {
        serde_json::from_value(json!({
            "action": if upstream_is_stream { "stream" } else { "sync" },
            "request_id": "req-raw",
            "provider_id": "provider-raw",
            "endpoint_id": "endpoint-raw",
            "key_id": "key-raw",
            "upstream_url": "https://api.anthropic.test/v1/messages",
            "provider_api_format": "claude:messages",
            "client_api_format": "claude:messages",
            "provider_request_headers": {
                "content-type": "application/json",
                (EXECUTION_RESPONSE_BODY_MODE_HEADER): ExecutionResponseBodyMode::PreserveBytes.as_str()
            },
            "provider_request_body": {
                "model": "claude-sonnet-4",
                "messages": []
            },
            "provider_request_body_base64": "eyAibW9kZWwiOiAiY2xhdWRlLXNvbm5ldC00IiwgIm1lc3NhZ2VzIjogW10gfQ==",
            "content_type": "application/json",
            "upstream_is_stream": upstream_is_stream
        }))
        .expect("decision should deserialize")
    }

    fn request_parts() -> http::request::Parts {
        http::Request::builder()
            .uri("http://localhost/v1/messages")
            .body(())
            .expect("request should build")
            .into_parts()
            .0
    }

    #[test]
    fn standard_sync_plan_prefers_exact_request_body_bytes() {
        let built = build_standard_sync_plan_from_decision(
            &request_parts(),
            &json!({}),
            decision_with_raw_body(false),
        )
        .expect("plan should build")
        .expect("plan should exist");

        assert!(built.plan.body.json_body.is_none());
        assert_eq!(
            built.plan.body.body_bytes_b64.as_deref(),
            Some("eyAibW9kZWwiOiAiY2xhdWRlLXNvbm5ldC00IiwgIm1lc3NhZ2VzIjogW10gfQ==")
        );
        assert_eq!(
            built
                .plan
                .headers
                .get(EXECUTION_RESPONSE_BODY_MODE_HEADER)
                .map(String::as_str),
            Some(ExecutionResponseBodyMode::PreserveBytes.as_str())
        );
    }

    #[test]
    fn standard_stream_plan_prefers_exact_request_body_bytes() {
        let built = build_standard_stream_plan_from_decision(
            &request_parts(),
            &json!({}),
            decision_with_raw_body(true),
            false,
        )
        .expect("plan should build")
        .expect("plan should exist");

        assert!(built.plan.body.json_body.is_none());
        assert!(built.plan.body.body_bytes_b64.is_some());
    }
}

//! Authenticated WebRTC call creation for Codex Live.

use std::collections::BTreeMap;
use std::net::SocketAddr;

use aether_contracts::{
    ExecutionPlan, ExecutionResponseBodyMode, ExecutionResult, EXECUTION_RESPONSE_BODY_MODE_HEADER,
};
use axum::body::{Body, Bytes};
use axum::http::{HeaderValue, Response, StatusCode};
use base64::Engine as _;
use serde_json::json;
use tracing::{info, warn};

use crate::ai_serving::build_standard_sync_plan_from_decision;
use crate::api::response::{
    build_client_response_from_parts, build_client_response_from_parts_with_mutator,
    build_local_auth_rejection_response, build_local_http_error_response_with_request_path,
};
use crate::control::{execution_plan_balance_capacity_rejection, GatewayPublicRequestContext};
use crate::execution_runtime::execute_execution_runtime_sync_plan_with_report_context;
use crate::handlers::proxy::websocket::responses::ResponsesWebSocketTurnAdmission;
use crate::{AppState, GatewayError};

use super::audit::mark_live_call_create_report_context;
use super::live_usage_accounting_is_safe;
use super::planner::{live_call_url, plan_live_candidate, LiveAuthMode, LivePoolLeaseGuard};
use super::protocol::{build_live_multipart, extract_call_id_from_location, parse_live_multipart};
use super::registry::{LiveCallBinding, LiveCallRegistry};

const MAX_LIVE_HTTP_BODY_BYTES: usize = 1024 * 1024;

pub(crate) async fn maybe_handle_live_http(
    state: &AppState,
    request_context: &GatewayPublicRequestContext,
    parts: &http::request::Parts,
    body: Option<&Bytes>,
    remote_addr: &SocketAddr,
) -> Result<Option<Response<Body>>, GatewayError> {
    if parts.method != http::Method::POST || request_context.request_path != "/v1/live" {
        return Ok(None);
    }
    let Some(control_decision) = request_context.control_decision.as_ref() else {
        return Ok(Some(local_live_error(
            request_context,
            StatusCode::NOT_FOUND,
            "Codex Live route is unavailable",
        )?));
    };
    if !live_usage_accounting_is_safe(control_decision) {
        return Ok(Some(local_live_error(
            request_context,
            StatusCode::NOT_IMPLEMENTED,
            "Codex Live is unavailable for finite-balance keys until Frameless usage settlement is supported",
        )?));
    }
    let Some(body) = body else {
        return Ok(Some(local_live_error(
            request_context,
            StatusCode::BAD_REQUEST,
            "Codex Live requires a multipart WebRTC offer",
        )?));
    };
    if body.len() > MAX_LIVE_HTTP_BODY_BYTES {
        return Ok(Some(local_live_error(
            request_context,
            StatusCode::PAYLOAD_TOO_LARGE,
            "Codex Live WebRTC offer exceeds the 1 MiB limit",
        )?));
    }
    let content_type = parts
        .headers
        .get(http::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();
    let offer = match parse_live_multipart(content_type, body.as_ref()) {
        Ok(offer) => offer,
        Err(error) => {
            return Ok(Some(local_live_error(
                request_context,
                error.status_code(),
                error.client_message(),
            )?))
        }
    };
    let Some(client_model) = offer
        .session
        .get("model")
        .and_then(serde_json::Value::as_str)
    else {
        return Ok(Some(local_live_error(
            request_context,
            StatusCode::BAD_REQUEST,
            "Codex Live session.model must be a non-empty model identifier",
        )?));
    };

    let Some(mut candidate) = plan_live_candidate(
        state,
        request_context.trace_id.as_str(),
        control_decision,
        &parts.headers,
        remote_addr,
        client_model,
        None,
    )
    .await?
    else {
        return Ok(Some(local_live_error(
            request_context,
            StatusCode::BAD_GATEWAY,
            "No eligible Codex Live provider mapping is available",
        )?));
    };
    let lease = LivePoolLeaseGuard::new(state, &candidate);
    let binding = LiveCallBinding::from_candidate(&candidate);
    let mut provider_session = offer.session.clone();
    provider_session
        .as_object_mut()
        .expect("validated Live session is a JSON object")
        .insert(
            "model".to_string(),
            serde_json::Value::String(candidate.provider_model.clone()),
        );
    let upstream_url = match live_call_url(&candidate) {
        Ok(url) => url,
        Err(error) => {
            lease.release().await;
            return Ok(Some(local_live_error(
                request_context,
                error.status_code(),
                error.client_message(),
            )?));
        }
    };

    let (provider_content_type, provider_body_base64) =
        build_live_call_provider_body(candidate.auth_mode, offer.sdp.as_str(), &provider_session)?;
    // The standard plan builder requires a JSON body marker even when the exact wire body is
    // carried as bytes. Keep only the mapped model here: retaining the SDP/session projection in
    // the decision would unnecessarily widen the surface for future logging or report changes.
    let provider_body_marker = json!({"model": candidate.provider_model.clone()});
    candidate.execution.upstream_url = Some(upstream_url);
    candidate.execution.provider_request_method = Some("POST".to_string());
    candidate.execution.provider_request_body = Some(provider_body_marker.clone());
    candidate.execution.provider_request_body_base64 = Some(provider_body_base64);
    candidate.execution.content_type = Some(provider_content_type.clone());
    candidate.execution.content_encoding = None;
    candidate.execution.request_gzip = None;
    candidate.execution.upstream_is_stream = false;
    prepare_live_call_request_headers(
        &mut candidate.execution.provider_request_headers,
        provider_content_type.as_str(),
    );
    candidate.execution.provider_request_headers.insert(
        EXECUTION_RESPONSE_BODY_MODE_HEADER.to_string(),
        ExecutionResponseBodyMode::PreserveBytes
            .as_str()
            .to_string(),
    );

    let Some(mut attempt) =
        build_standard_sync_plan_from_decision(parts, &provider_body_marker, candidate.execution)?
    else {
        lease.release().await;
        return Ok(Some(local_live_error(
            request_context,
            StatusCode::BAD_GATEWAY,
            "Codex Live provider request could not be built",
        )?));
    };
    // The synchronous SDP exchange has an ordinary request lifecycle, but it
    // does not contain the media leg's token/cost usage. Keep the existing row
    // while making that boundary explicit and non-billable.
    mark_live_call_create_report_context(&mut attempt.report_context);
    if let Some(rejection) = execution_plan_balance_capacity_rejection(
        state,
        control_decision,
        &attempt.plan,
        attempt.report_context.as_ref(),
    )
    .await?
    {
        lease.release().await;
        return Ok(Some(build_local_auth_rejection_response(
            request_context.trace_id.as_str(),
            Some(control_decision),
            &rejection,
        )?));
    }
    let admission = ResponsesWebSocketTurnAdmission::acquire(
        state,
        &attempt.plan,
        request_context.trace_id.as_str(),
    )
    .await?;
    let result = execute_execution_runtime_sync_plan_with_report_context(
        state,
        Some(request_context.trace_id.as_str()),
        &attempt.plan,
        attempt.report_context.as_ref(),
    )
    .await;
    // These guards intentionally cover only the synchronous call-creation exchange. The
    // WebRTC media leg bypasses Aether, and neither the two-hour routing binding nor a sideband
    // attachment proves that media is still alive. Holding either guard for a guessed lifetime
    // would leak capacity or release it early without an authoritative upstream close signal.
    admission.release().await;
    let pool_lease_healthy = lease.is_healthy();
    lease.release().await;
    let result = result?;
    if !(200..300).contains(&result.status_code) {
        let response_body = execution_result_body(&result)?;
        let downstream_headers =
            sanitized_live_response_headers(&result.headers, response_body.preserves_wire_encoding);
        warn!(
            event_name = "codex_live_call_upstream_failed",
            log_type = "ops",
            trace_id = %request_context.trace_id,
            provider_id = %attempt.plan.provider_id,
            endpoint_id = %attempt.plan.endpoint_id,
            key_id = %attempt.plan.key_id,
            status_code = result.status_code,
            elapsed_ms = result.telemetry.as_ref().and_then(|value| value.elapsed_ms),
            "Codex Live call creation failed upstream"
        );
        return Ok(Some(build_client_response_from_parts(
            result.status_code,
            &downstream_headers,
            Body::from(response_body.bytes),
            request_context.trace_id.as_str(),
            Some(control_decision),
        )?));
    }
    if !pool_lease_healthy {
        warn_live_call_orphaned(
            request_context,
            &attempt.plan,
            &result,
            "pool_lease_lost",
            None,
        );
        return Ok(Some(local_live_error(
            request_context,
            StatusCode::SERVICE_UNAVAILABLE,
            "Codex Live provider lease expired during call creation",
        )?));
    }
    let response_body = match execution_result_body(&result) {
        Ok(body) => body,
        Err(error) => {
            warn_live_call_orphaned(
                request_context,
                &attempt.plan,
                &result,
                "response_body_unavailable",
                None,
            );
            return Err(error);
        }
    };
    let Some(location) = header_value(&result.headers, "location") else {
        warn_live_call_orphaned(
            request_context,
            &attempt.plan,
            &result,
            "location_missing",
            None,
        );
        return Ok(Some(local_live_error(
            request_context,
            StatusCode::BAD_GATEWAY,
            "Codex Live upstream response did not include a call location",
        )?));
    };
    let call_id = match extract_call_id_from_location(location) {
        Ok(call_id) => call_id,
        Err(error) => {
            warn_live_call_orphaned(
                request_context,
                &attempt.plan,
                &result,
                "location_invalid",
                Some(error.code()),
            );
            return Ok(Some(local_live_error(
                request_context,
                StatusCode::BAD_GATEWAY,
                error.client_message(),
            )?));
        }
    };
    let Some(auth_context) = control_decision.auth_context.as_ref() else {
        warn_live_call_orphaned(
            request_context,
            &attempt.plan,
            &result,
            "auth_context_missing",
            None,
        );
        return Ok(Some(local_live_error(
            request_context,
            StatusCode::UNAUTHORIZED,
            "Codex Live requires an authenticated gateway API key",
        )?));
    };
    let registry = LiveCallRegistry::new(std::sync::Arc::clone(&state.runtime_state));
    if let Err(error) = registry
        .register(
            auth_context.user_id.as_str(),
            auth_context.api_key_id.as_str(),
            call_id.as_str(),
            &binding,
        )
        .await
    {
        warn_live_call_orphaned(
            request_context,
            &attempt.plan,
            &result,
            "binding_failed",
            Some(error.kind()),
        );
        return Ok(Some(local_live_error(
            request_context,
            StatusCode::SERVICE_UNAVAILABLE,
            "Codex Live sideband binding is temporarily unavailable",
        )?));
    }
    info!(
        event_name = "codex_live_call_created",
        log_type = "event",
        trace_id = %request_context.trace_id,
        provider_id = %attempt.plan.provider_id,
        endpoint_id = %attempt.plan.endpoint_id,
        key_id = %attempt.plan.key_id,
        client_model = %binding.client_model(),
        status_code = result.status_code,
        elapsed_ms = result.telemetry.as_ref().and_then(|value| value.elapsed_ms),
        usage_unavailable = true,
        "Codex Live created a bound WebRTC call"
    );
    let downstream_location = format!("/v1/live/{call_id}");
    let downstream_headers =
        sanitized_live_response_headers(&result.headers, response_body.preserves_wire_encoding);
    Ok(Some(build_client_response_from_parts_with_mutator(
        result.status_code,
        &downstream_headers,
        Body::from(response_body.bytes),
        request_context.trace_id.as_str(),
        Some(control_decision),
        |headers| {
            headers.insert(
                http::header::LOCATION,
                HeaderValue::from_str(downstream_location.as_str())
                    .map_err(|error| GatewayError::Internal(error.to_string()))?,
            );
            Ok(())
        },
    )?))
}

fn local_live_error(
    request_context: &GatewayPublicRequestContext,
    status: StatusCode,
    message: &str,
) -> Result<Response<Body>, GatewayError> {
    build_local_http_error_response_with_request_path(
        request_context.trace_id.as_str(),
        request_context.control_decision.as_ref(),
        Some("/v1/live"),
        status,
        message,
    )
}

fn remove_headers(headers: &mut BTreeMap<String, String>, names: &[&str]) {
    headers.retain(|candidate, _| {
        !names
            .iter()
            .any(|name| candidate.eq_ignore_ascii_case(name))
    });
}

fn header_value<'a>(headers: &'a BTreeMap<String, String>, name: &str) -> Option<&'a str> {
    headers
        .iter()
        .find(|(candidate, _)| candidate.eq_ignore_ascii_case(name))
        .map(|(_, value)| value.as_str())
}

fn prepare_live_call_request_headers(headers: &mut BTreeMap<String, String>, content_type: &str) {
    remove_headers(
        headers,
        &[
            "content-type",
            "content-length",
            "content-encoding",
            "accept",
            "accept-encoding",
        ],
    );
    headers.insert("content-type".to_string(), content_type.to_string());
    headers.insert("accept".to_string(), "application/sdp".to_string());
    headers.insert("accept-encoding".to_string(), "identity".to_string());
}

fn build_live_call_provider_body(
    auth_mode: LiveAuthMode,
    sdp: &str,
    session: &serde_json::Value,
) -> Result<(String, String), GatewayError> {
    let (content_type, bytes) = match auth_mode {
        LiveAuthMode::ApiKey => {
            let (content_type, bytes) = build_live_multipart(sdp, session);
            (content_type, bytes)
        }
        LiveAuthMode::ChatGptOauth => {
            let bytes = serde_json::to_vec(&json!({"sdp": sdp, "session": session}))
                .map_err(|error| GatewayError::Internal(error.to_string()))?;
            ("application/json".to_string(), bytes)
        }
    };
    Ok((
        content_type,
        base64::engine::general_purpose::STANDARD.encode(bytes),
    ))
}

fn sanitized_live_response_headers(
    headers: &BTreeMap<String, String>,
    preserves_wire_encoding: bool,
) -> BTreeMap<String, String> {
    let mut sanitized = headers.clone();
    remove_headers(&mut sanitized, &["location", "set-cookie", "set-cookie2"]);
    if !preserves_wire_encoding {
        remove_headers(&mut sanitized, &["content-length", "content-encoding"]);
    }
    sanitized
}

fn warn_live_call_orphaned(
    request_context: &GatewayPublicRequestContext,
    plan: &ExecutionPlan,
    result: &ExecutionResult,
    reason: &'static str,
    error_kind: Option<&'static str>,
) {
    warn!(
        event_name = "codex_live_call_orphaned",
        log_type = "ops",
        trace_id = %request_context.trace_id,
        provider_id = %plan.provider_id,
        endpoint_id = %plan.endpoint_id,
        key_id = %plan.key_id,
        status_code = result.status_code,
        elapsed_ms = result.telemetry.as_ref().and_then(|value| value.elapsed_ms),
        reason,
        error_kind = error_kind.unwrap_or("none"),
        "Codex Live upstream call succeeded but could not be safely exposed downstream"
    );
}

struct LiveResponseBody {
    bytes: Vec<u8>,
    preserves_wire_encoding: bool,
}

fn execution_result_body(result: &ExecutionResult) -> Result<LiveResponseBody, GatewayError> {
    let Some(body) = result.body.as_ref() else {
        return Ok(LiveResponseBody {
            bytes: Vec::new(),
            preserves_wire_encoding: false,
        });
    };
    if let Some(encoded) = body.body_bytes_b64.as_deref() {
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(encoded)
            .map_err(|error| GatewayError::Internal(error.to_string()))?;
        return Ok(LiveResponseBody {
            bytes,
            preserves_wire_encoding: true,
        });
    }
    let bytes = body
        .json_body
        .as_ref()
        .map(serde_json::to_vec)
        .transpose()
        .map(|body| body.unwrap_or_default())
        .map_err(|error| GatewayError::Internal(error.to_string()))?;
    Ok(LiveResponseBody {
        bytes,
        preserves_wire_encoding: false,
    })
}

#[cfg(test)]
mod tests {
    use aether_contracts::{ExecutionPlan, ExecutionResult, ResponseBody};
    use axum::body::to_bytes;

    use crate::control::{GatewayControlAuthContext, GatewayControlDecision};

    use super::*;

    #[test]
    fn preserved_wire_bytes_win_over_the_json_projection() {
        let result = ExecutionResult {
            request_id: "request".to_string(),
            candidate_id: None,
            status_code: 201,
            headers: Default::default(),
            response_observation: None,
            body: Some(ResponseBody {
                json_body: Some(json!({"projected": true})),
                body_bytes_b64: Some(
                    base64::engine::general_purpose::STANDARD.encode(b"raw-sdp-answer"),
                ),
            }),
            telemetry: None,
            error: None,
        };
        let body = execution_result_body(&result).unwrap();
        assert_eq!(body.bytes, b"raw-sdp-answer");
        assert!(body.preserves_wire_encoding);
    }

    #[test]
    fn live_call_bodies_are_bytes_only_and_do_not_enter_report_context() {
        let session = json!({
            "model": "provider-live-model",
            "instructions": "opaque private instructions",
            "future_capability": {"enabled": true}
        });
        let sdp = "v=0\r\no=private-live-offer";
        let provider_body_marker = json!({"model": "provider-live-model"});

        for auth_mode in [LiveAuthMode::ApiKey, LiveAuthMode::ChatGptOauth] {
            let (content_type, encoded) =
                build_live_call_provider_body(auth_mode, sdp, &session).unwrap();
            let report_context = aether_ai_serving::augment_sync_report_context(
                Some(json!({"trace_id": "trace-live"})),
                &BTreeMap::new(),
                &provider_body_marker,
            )
            .unwrap()
            .unwrap();
            assert!(report_context.get("provider_request_body").is_none());
            assert!(!report_context.to_string().contains("private-live-offer"));
            assert!(!report_context
                .to_string()
                .contains("opaque private instructions"));

            let plan_body = aether_ai_serving::resolve_ai_passthrough_sync_request_body(
                Some(provider_body_marker.clone()),
                Some(encoded.clone()),
            );
            assert!(plan_body.json_body.is_none());
            assert_eq!(plan_body.body_bytes_b64.as_deref(), Some(encoded.as_str()));
            let usage_plan = ExecutionPlan {
                request_id: "trace-live".to_string(),
                candidate_id: Some("candidate-live".to_string()),
                provider_name: Some("codex".to_string()),
                provider_id: "provider-live".to_string(),
                endpoint_id: "endpoint-live".to_string(),
                key_id: "key-live".to_string(),
                method: "POST".to_string(),
                url: "https://api.openai.com/v1/live".to_string(),
                headers: BTreeMap::new(),
                content_type: Some(content_type.clone()),
                content_encoding: None,
                body: plan_body,
                stream: false,
                client_api_format: "openai:responses".to_string(),
                provider_api_format: "openai:responses".to_string(),
                model_name: Some("provider-live-model".to_string()),
                proxy: None,
                transport_profile: None,
                timeouts: None,
            };
            let usage_seed = aether_usage_runtime::build_terminal_usage_context_seed(
                &usage_plan,
                Some(&report_context),
            );
            assert!(usage_seed.provider_request.is_none());
            assert!(!usage_seed
                .request_metadata
                .as_ref()
                .map(ToString::to_string)
                .unwrap_or_default()
                .contains("private-live-offer"));

            let wire = base64::engine::general_purpose::STANDARD
                .decode(encoded)
                .unwrap();
            match auth_mode {
                LiveAuthMode::ApiKey => {
                    let parsed = parse_live_multipart(content_type.as_str(), wire.as_slice())
                        .expect("API-key multipart should round-trip");
                    assert_eq!(parsed.sdp, sdp);
                    assert_eq!(parsed.session, session);
                }
                LiveAuthMode::ChatGptOauth => {
                    assert_eq!(content_type, "application/json");
                    let decoded: serde_json::Value = serde_json::from_slice(wire.as_slice())
                        .expect("OAuth JSON should round-trip");
                    assert_eq!(decoded["sdp"], sdp);
                    assert_eq!(decoded["session"], session);
                    assert_eq!(decoded["session"]["model"], "provider-live-model");
                    assert_eq!(
                        decoded["session"]["future_capability"],
                        json!({"enabled": true})
                    );
                }
            }
        }
    }

    #[test]
    fn live_call_request_headers_replace_stale_body_and_encoding_metadata() {
        let mut headers = BTreeMap::from([
            ("Content-Type".to_string(), "stale".to_string()),
            ("CONTENT-LENGTH".to_string(), "42".to_string()),
            ("Content-Encoding".to_string(), "gzip".to_string()),
            ("Accept".to_string(), "application/json".to_string()),
            ("ACCEPT-ENCODING".to_string(), "br, gzip".to_string()),
            ("x-future".to_string(), "opaque".to_string()),
        ]);
        prepare_live_call_request_headers(&mut headers, "multipart/form-data; boundary=live-test");

        assert_eq!(
            header_value(&headers, "content-type"),
            Some("multipart/form-data; boundary=live-test")
        );
        assert_eq!(header_value(&headers, "accept"), Some("application/sdp"));
        assert_eq!(header_value(&headers, "accept-encoding"), Some("identity"));
        assert_eq!(header_value(&headers, "content-length"), None);
        assert_eq!(header_value(&headers, "content-encoding"), None);
        assert_eq!(headers.get("x-future").map(String::as_str), Some("opaque"));
    }

    #[test]
    fn live_response_headers_never_expose_upstream_location_or_cookies() {
        let headers = BTreeMap::from([
            (
                "Location".to_string(),
                "https://upstream/v1/live/secret".to_string(),
            ),
            ("SET-COOKIE".to_string(), "session=secret".to_string()),
            ("Set-Cookie2".to_string(), "legacy=secret".to_string()),
            ("Content-Length".to_string(), "128".to_string()),
            ("Content-Encoding".to_string(), "gzip".to_string()),
            ("x-future".to_string(), "opaque".to_string()),
        ]);

        let sanitized = sanitized_live_response_headers(&headers, true);
        assert_eq!(header_value(&sanitized, "location"), None);
        assert_eq!(header_value(&sanitized, "set-cookie"), None);
        assert_eq!(header_value(&sanitized, "set-cookie2"), None);
        assert_eq!(header_value(&sanitized, "content-length"), Some("128"));
        assert_eq!(header_value(&sanitized, "content-encoding"), Some("gzip"));
        assert_eq!(header_value(&sanitized, "x-future"), Some("opaque"));
    }

    #[test]
    fn rebuilt_live_response_body_drops_stale_length_and_encoding() {
        let headers = BTreeMap::from([
            ("content-length".to_string(), "128".to_string()),
            ("content-encoding".to_string(), "gzip".to_string()),
            ("content-type".to_string(), "application/json".to_string()),
        ]);

        let sanitized = sanitized_live_response_headers(&headers, false);
        assert_eq!(header_value(&sanitized, "content-length"), None);
        assert_eq!(header_value(&sanitized, "content-encoding"), None);
        assert_eq!(
            header_value(&sanitized, "content-type"),
            Some("application/json")
        );
    }

    #[tokio::test]
    async fn finite_balance_post_live_fails_before_parsing_or_upstream_execution() {
        let mut decision = GatewayControlDecision::synthetic(
            "/v1/live",
            Some("ai_public".to_string()),
            Some("openai".to_string()),
            Some("codex_live".to_string()),
            Some("openai:responses".to_string()),
        );
        decision.auth_context = Some(GatewayControlAuthContext {
            user_id: "user-finite".to_string(),
            api_key_id: "key-finite".to_string(),
            username: Some("finite".to_string()),
            api_key_name: Some("finite".to_string()),
            balance_remaining: Some(1.25),
            access_allowed: true,
            user_rate_limit: None,
            api_key_rate_limit: None,
            api_key_is_standalone: false,
            admin_bypass_limits: false,
            local_rejection: None,
            allowed_models: None,
            ip_rules: None,
        });
        let request_context = GatewayPublicRequestContext {
            trace_id: "trace-live-finite".to_string(),
            request_method: http::Method::POST,
            request_path: "/v1/live".to_string(),
            request_query_string: None,
            request_content_type: None,
            host_header: None,
            control_decision: Some(decision),
        };
        let (parts, _) = http::Request::builder()
            .method(http::Method::POST)
            .uri("/v1/live")
            .body(())
            .unwrap()
            .into_parts();
        let response = maybe_handle_live_http(
            &AppState::new().expect("gateway state should build"),
            &request_context,
            &parts,
            None,
            &"127.0.0.1:65000".parse().unwrap(),
        )
        .await
        .unwrap()
        .expect("Live HTTP route must produce a local rejection");
        assert_eq!(response.status(), StatusCode::NOT_IMPLEMENTED);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        assert!(String::from_utf8_lossy(body.as_ref()).contains("finite-balance"));
    }
}

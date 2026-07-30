use super::super::errors::build_internal_control_error_response;
use super::super::provisioning::{
    provider_oauth_key_proxy_value, provision_provider_oauth_token_payload_for_provider,
};
use super::super::runtime::resolve_provider_oauth_runtime_endpoints;
use super::super::state::authorize_admin_provider_oauth_with_cookie;
use crate::handlers::admin::provider::shared::paths::admin_provider_oauth_cookie_provider_id;
use crate::handlers::admin::request::{AdminAppState, AdminRequestContext};
use crate::GatewayError;
use axum::{body::Body, http, response::Response};

struct ClaudeCookieAuthorizeRequest {
    session_key: String,
    name: Option<String>,
    proxy_node_id: Option<String>,
}

pub(super) async fn handle_admin_provider_oauth_cookie_authorize(
    state: &AdminAppState<'_>,
    request_context: &AdminRequestContext<'_>,
    request_body: Option<&axum::body::Bytes>,
) -> Result<Response<Body>, GatewayError> {
    if !state.has_provider_catalog_data_reader() {
        return Ok(super::super::state::build_admin_provider_oauth_backend_unavailable_response());
    }
    let Some(provider_id) = admin_provider_oauth_cookie_provider_id(request_context.path()) else {
        return Ok(build_internal_control_error_response(
            http::StatusCode::NOT_FOUND,
            "Provider 不存在",
        ));
    };
    let payload = match parse_claude_cookie_authorize_request(request_body) {
        Ok(payload) => payload,
        Err(response) => return Ok(response),
    };

    let Some(provider) = state
        .read_provider_catalog_providers_by_ids(std::slice::from_ref(&provider_id))
        .await?
        .into_iter()
        .next()
    else {
        return Ok(build_internal_control_error_response(
            http::StatusCode::NOT_FOUND,
            "Provider 不存在",
        ));
    };
    let provider_type = provider.provider_type.trim().to_ascii_lowercase();
    if provider_type != "claude_code" {
        return Ok(build_internal_control_error_response(
            http::StatusCode::BAD_REQUEST,
            "Cookie 授权仅支持 Claude Code Provider",
        ));
    }

    let endpoint_resolution =
        resolve_provider_oauth_runtime_endpoints(state, &provider, &provider_type).await?;
    let endpoints = endpoint_resolution.endpoints;
    let request_proxy = state
        .resolve_admin_provider_oauth_operation_proxy_snapshot(
            payload.proxy_node_id.as_deref(),
            &[
                endpoint_resolution
                    .runtime_endpoint
                    .as_ref()
                    .and_then(|endpoint| endpoint.proxy.as_ref()),
                provider.proxy.as_ref(),
            ],
        )
        .await;
    let key_proxy = provider_oauth_key_proxy_value(payload.proxy_node_id.as_deref());
    let token_payload = match authorize_admin_provider_oauth_with_cookie(
        state,
        payload.session_key,
        request_proxy.clone(),
    )
    .await
    {
        Ok(payload) => payload,
        Err(response) => return Ok(response),
    };

    provision_provider_oauth_token_payload_for_provider(
        state,
        &provider,
        &endpoints,
        &token_payload,
        payload.name,
        key_proxy,
        request_proxy,
        "cookie-authorize",
    )
    .await
}

fn parse_claude_cookie_authorize_request(
    request_body: Option<&axum::body::Bytes>,
) -> Result<ClaudeCookieAuthorizeRequest, Response<Body>> {
    let Some(request_body) = request_body else {
        return Err(bad_cookie_request("请求体必须是合法的 JSON 对象"));
    };
    let payload = serde_json::from_slice::<serde_json::Value>(request_body)
        .ok()
        .and_then(|value| value.as_object().cloned())
        .ok_or_else(|| bad_cookie_request("请求体必须是合法的 JSON 对象"))?;
    let cookie = ["cookie", "session_key", "sessionKey"]
        .into_iter()
        .find_map(|key| payload.get(key).and_then(serde_json::Value::as_str))
        .ok_or_else(|| bad_cookie_request("Cookie 不能为空"))?;
    let session_key = normalize_claude_session_key(cookie)
        .ok_or_else(|| bad_cookie_request("Cookie 中缺少有效的 sessionKey"))?;

    Ok(ClaudeCookieAuthorizeRequest {
        session_key,
        name: optional_trimmed_string(&payload, "name"),
        proxy_node_id: optional_trimmed_string(&payload, "proxy_node_id")
            .or_else(|| optional_trimmed_string(&payload, "proxyNodeId")),
    })
}

pub(super) fn normalize_claude_session_key(raw: &str) -> Option<String> {
    let raw = raw.trim();
    if raw.is_empty() || raw.contains(['\r', '\n']) {
        return None;
    }
    let cookie = raw
        .split_once(':')
        .filter(|(name, _)| name.trim().eq_ignore_ascii_case("cookie"))
        .map(|(_, value)| value.trim())
        .unwrap_or(raw);

    if !cookie.contains('=') {
        return valid_session_key_value(cookie).then(|| cookie.to_string());
    }

    let mut session_key = None;
    for segment in cookie.split(';') {
        let (name, value) = segment.trim().split_once('=')?;
        if !name.trim().eq_ignore_ascii_case("sessionKey") {
            continue;
        }
        if session_key.is_some() || !valid_session_key_value(value.trim()) {
            return None;
        }
        session_key = Some(value.trim().to_string());
    }
    session_key
}

fn valid_session_key_value(value: &str) -> bool {
    !value.is_empty()
        && !value.contains(['\r', '\n', ';'])
        && http::HeaderValue::from_str(value).is_ok()
}

fn optional_trimmed_string(
    payload: &serde_json::Map<String, serde_json::Value>,
    key: &str,
) -> Option<String> {
    payload
        .get(key)
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn bad_cookie_request(detail: &'static str) -> Response<Body> {
    build_internal_control_error_response(http::StatusCode::BAD_REQUEST, detail)
}

#[cfg(test)]
mod tests {
    use super::{normalize_claude_session_key, parse_claude_cookie_authorize_request};
    use axum::body::Bytes;
    use serde_json::json;

    #[test]
    fn normalizes_supported_claude_cookie_inputs() {
        for (input, expected) in [
            ("sk-ant-sid01-raw", "sk-ant-sid01-raw"),
            ("sessionKey=sk-ant-sid01-pair", "sk-ant-sid01-pair"),
            (
                "Cookie: other=value; sessionKey=sk-ant-sid01-header; theme=dark",
                "sk-ant-sid01-header",
            ),
        ] {
            assert_eq!(
                normalize_claude_session_key(input).as_deref(),
                Some(expected)
            );
        }
    }

    #[test]
    fn rejects_ambiguous_or_unsafe_claude_cookie_inputs() {
        for input in [
            "",
            "foo=bar",
            "sessionKey=one; sessionKey=two",
            "sessionKey=value\r\nx-leak: yes",
        ] {
            assert!(
                normalize_claude_session_key(input).is_none(),
                "input={input:?}"
            );
        }
    }

    #[test]
    fn accepts_authorize_body_and_session_key_above_previous_caps() {
        let session_key = "x".repeat(40 * 1024);
        let body = Bytes::from(json!({ "sessionKey": session_key }).to_string());
        assert!(body.len() > 32 * 1024);

        let parsed = parse_claude_cookie_authorize_request(Some(&body))
            .expect("large Cookie authorization payload should parse");
        assert_eq!(parsed.session_key.len(), 40 * 1024);
    }
}

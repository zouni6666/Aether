use super::{
    classified, classified_with_request_auth_channel, detect_claude_client_surface,
    is_gemini_cli_request, is_gemini_models_route, is_gemini_operation_route, ClassifiedRoute,
};
use crate::ai_serving::ApiOperation;

pub(super) fn classify_ai_public_route(
    method: &http::Method,
    normalized_path: &str,
    headers: &http::HeaderMap,
) -> Option<ClassifiedRoute> {
    if let Some(route) = classify_antigravity_v1internal_route(method, normalized_path) {
        Some(route)
    } else if method == http::Method::POST && normalized_path == "/v1/chat/completions" {
        Some(classified(
            "ai_public",
            "openai",
            "chat",
            "openai:chat",
            true,
        ))
    } else if method == http::Method::POST && normalized_path == "/v1/embeddings" {
        Some(classified(
            "ai_public",
            "openai",
            "embedding",
            "openai:embedding",
            true,
        ))
    } else if method == http::Method::POST && normalized_path == "/v1/rerank" {
        Some(classified(
            "ai_public",
            "openai",
            "rerank",
            "openai:rerank",
            true,
        ))
    } else if method == http::Method::GET
        && normalized_path == "/v1/realtime"
        && is_websocket_upgrade_request(headers)
    {
        Some(classified(
            "ai_public",
            "openai",
            "realtime",
            "openai:realtime",
            true,
        ))
    } else if (method == http::Method::POST && normalized_path == "/v1/live")
        || (method == http::Method::GET
            && (normalized_path == "/v1/live" || normalized_path.starts_with("/v1/live/"))
            && is_websocket_upgrade_request(headers))
    {
        // Codex Live has an independent wire contract and permission surface;
        // it must never be authorized as an OpenAI Responses request.
        Some(classified("ai_public", "codex", "live", "codex:live", true))
    } else if (method == http::Method::POST
        || (method == http::Method::GET
            && normalized_path == "/v1/responses"
            && is_websocket_upgrade_request(headers)))
        && matches!(normalized_path, "/v1/responses" | "/v1/responses/compact")
    {
        if normalized_path.ends_with("/compact") {
            Some(classified(
                "ai_public",
                "openai",
                "responses:compact",
                "openai:responses:compact",
                true,
            ))
        } else {
            Some(classified(
                "ai_public",
                "openai",
                "responses",
                "openai:responses",
                true,
            ))
        }
    } else if method == http::Method::POST && normalized_path == "/v1/alpha/search" {
        Some(classified(
            "ai_public",
            "openai",
            "search",
            "openai:search",
            true,
        ))
    } else if method == http::Method::POST
        && matches!(
            normalized_path,
            "/v1/images/generations" | "/v1/images/edits"
        )
    {
        Some(classified(
            "ai_public",
            "openai",
            "image",
            "openai:image",
            true,
        ))
    } else if method == http::Method::POST && normalized_path == "/v1/messages/count_tokens" {
        let request_auth_channel = claude_request_auth_channel(headers);
        Some(
            classified_with_request_auth_channel(
                "ai_public",
                "claude",
                "count_tokens",
                request_auth_channel,
                "claude:messages",
                true,
            )
            .with_client_surface(detect_claude_client_surface(headers))
            .with_api_operation(ApiOperation::ClaudeCountTokens),
        )
    } else if method == http::Method::POST && normalized_path == "/v1/messages" {
        let request_auth_channel = claude_request_auth_channel(headers);
        Some(
            classified_with_request_auth_channel(
                "ai_public",
                "claude",
                "messages",
                request_auth_channel,
                "claude:messages",
                true,
            )
            .with_client_surface(detect_claude_client_surface(headers))
            .with_api_operation(ApiOperation::ClaudeMessagesCreate),
        )
    } else if normalized_path.starts_with("/v1/videos") {
        Some(classified(
            "ai_public",
            "openai",
            "video",
            "openai:video",
            true,
        ))
    } else if method == http::Method::POST
        && matches!(normalized_path, "/v1/interactions" | "/v1beta/interactions")
    {
        Some(classified_with_request_auth_channel(
            "ai_public",
            "gemini",
            "interactions",
            "api_key",
            "gemini:interactions",
            true,
        ))
    } else if method == http::Method::POST && is_gemini_models_route(normalized_path) {
        if normalized_path.ends_with(":predictLongRunning") {
            Some(classified(
                "ai_public",
                "gemini",
                "video",
                "gemini:video",
                true,
            ))
        } else if normalized_path.ends_with(":embedContent")
            || normalized_path.ends_with(":batchEmbedContents")
        {
            Some(classified_with_request_auth_channel(
                "ai_public",
                "gemini",
                "embedding",
                "api_key",
                "gemini:embedding",
                true,
            ))
        } else if is_gemini_cli_request(headers) {
            Some(classified_with_request_auth_channel(
                "ai_public",
                "gemini",
                "generate_content",
                "bearer_like",
                "gemini:generate_content",
                true,
            ))
        } else {
            Some(classified_with_request_auth_channel(
                "ai_public",
                "gemini",
                "generate_content",
                "api_key",
                "gemini:generate_content",
                true,
            ))
        }
    } else if is_gemini_operation_method(method, normalized_path)
        && is_gemini_operation_route(normalized_path)
    {
        Some(classified(
            "ai_public",
            "gemini",
            "video",
            "gemini:video",
            true,
        ))
    } else if is_gemini_files_method(method, normalized_path) {
        Some(classified(
            "ai_public",
            "gemini",
            "files",
            "gemini:files",
            true,
        ))
    } else {
        None
    }
}

fn claude_request_auth_channel(headers: &http::HeaderMap) -> &'static str {
    if crate::headers::header_value_str(headers, "x-api-key").is_some()
        || crate::headers::header_value_str(headers, "api-key").is_some()
    {
        "api_key"
    } else if crate::headers::header_value_str(headers, http::header::AUTHORIZATION.as_str())
        .is_some_and(|value| value.trim().to_ascii_lowercase().starts_with("bearer "))
    {
        "bearer_like"
    } else {
        "api_key"
    }
}

fn is_websocket_upgrade_request(headers: &http::HeaderMap) -> bool {
    let has_upgrade_connection = headers
        .get(http::header::CONNECTION)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| {
            value
                .split(',')
                .map(str::trim)
                .any(|value| value.eq_ignore_ascii_case("upgrade"))
        });
    let has_websocket_upgrade = headers
        .get(http::header::UPGRADE)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.eq_ignore_ascii_case("websocket"));

    has_upgrade_connection && has_websocket_upgrade
}

fn is_gemini_operation_method(method: &http::Method, normalized_path: &str) -> bool {
    method == http::Method::GET
        || (method == http::Method::POST && normalized_path.ends_with(":cancel"))
}

fn is_gemini_files_method(method: &http::Method, normalized_path: &str) -> bool {
    (method == http::Method::POST && normalized_path == "/upload/v1beta/files")
        || ((method == http::Method::GET || method == http::Method::DELETE)
            && normalized_path.starts_with("/v1beta/files"))
}

fn classify_antigravity_v1internal_route(
    method: &http::Method,
    normalized_path: &str,
) -> Option<ClassifiedRoute> {
    if method != http::Method::POST {
        return None;
    }

    let action = normalized_path.strip_prefix("/v1internal:")?;
    let (route_kind, execution_runtime_candidate) = match action {
        "loadCodeAssist" => ("load_code_assist", false),
        "fetchAvailableModels" => ("fetch_available_models", false),
        "retrieveUserQuotaSummary" => ("retrieve_user_quota_summary", false),
        "fetchUserInfo" => ("fetch_user_info", false),
        "fetchAdminControls" => ("fetch_admin_controls", false),
        "setUserSettings" => ("set_user_settings", false),
        "listExperiments" => ("list_experiments", false),
        "recordCodeAssistMetrics" => ("record_code_assist_metrics", false),
        "writeTrajectoryAcls" => ("write_trajectory_acls", false),
        "streamGenerateContent" => ("stream_generate_content", true),
        _ => return None,
    };

    Some(classified_with_request_auth_channel(
        "ai_public",
        "antigravity",
        route_kind,
        "bearer_like",
        "antigravity:v1internal",
        execution_runtime_candidate,
    ))
}

#[cfg(test)]
mod tests {
    use axum::http::header::{CONNECTION, UPGRADE};
    use axum::http::{HeaderMap, HeaderValue, Method};

    use super::classify_ai_public_route;

    #[test]
    fn classifies_websocket_upgrade_on_responses_route() {
        let mut headers = HeaderMap::new();
        headers.insert(CONNECTION, HeaderValue::from_static("keep-alive, Upgrade"));
        headers.insert(UPGRADE, HeaderValue::from_static("websocket"));

        let route = classify_ai_public_route(&Method::GET, "/v1/responses", &headers)
            .expect("Responses WebSocket should be an AI public route");
        assert_eq!(route.route_class, "ai_public");
        assert_eq!(route.route_family, "openai");
        assert_eq!(route.route_kind, "responses");
        assert_eq!(route.auth_endpoint_signature, "openai:responses");
    }

    #[test]
    fn does_not_classify_plain_get_as_responses_websocket() {
        assert!(
            classify_ai_public_route(&Method::GET, "/v1/responses", &HeaderMap::new()).is_none()
        );
    }

    #[test]
    fn classifies_only_websocket_upgrade_on_realtime_route() {
        let mut headers = HeaderMap::new();
        headers.insert(CONNECTION, HeaderValue::from_static("keep-alive, Upgrade"));
        headers.insert(UPGRADE, HeaderValue::from_static("websocket"));

        let route = classify_ai_public_route(&Method::GET, "/v1/realtime", &headers)
            .expect("Realtime WebSocket should be an AI public route");
        assert_eq!(route.route_class, "ai_public");
        assert_eq!(route.route_family, "openai");
        assert_eq!(route.route_kind, "realtime");
        assert_eq!(route.auth_endpoint_signature, "openai:realtime");
        assert!(route.execution_runtime_candidate);

        assert!(
            classify_ai_public_route(&Method::GET, "/v1/realtime", &HeaderMap::new()).is_none()
        );
        assert!(classify_ai_public_route(&Method::POST, "/v1/realtime", &headers).is_none());
    }

    #[test]
    fn classifies_live_http_and_websocket_routes_as_codex_live() {
        let post = classify_ai_public_route(&Method::POST, "/v1/live", &HeaderMap::new())
            .expect("Live WebRTC call creation should be an AI public route");
        assert_eq!(post.route_family, "codex");
        assert_eq!(post.route_kind, "live");
        assert_eq!(post.auth_endpoint_signature, "codex:live");

        let mut headers = HeaderMap::new();
        headers.insert(CONNECTION, HeaderValue::from_static("Upgrade"));
        headers.insert(UPGRADE, HeaderValue::from_static("websocket"));
        for path in ["/v1/live", "/v1/live/rtc_opaque"] {
            let route = classify_ai_public_route(&Method::GET, path, &headers)
                .expect("Live WebSocket should be an AI public route");
            assert_eq!(route.route_family, "codex");
            assert_eq!(route.route_kind, "live");
            assert_eq!(route.auth_endpoint_signature, "codex:live");
        }

        assert!(
            classify_ai_public_route(&Method::GET, "/v1/live/rtc_opaque", &HeaderMap::new())
                .is_none()
        );
    }
}

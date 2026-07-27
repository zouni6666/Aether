use super::super::errors::build_internal_control_error_response;
use crate::handlers::admin::provider::shared::paths::{
    admin_provider_oauth_agent_identity_import_task_path,
    admin_provider_oauth_batch_import_task_path, admin_provider_oauth_cookie_task_path,
};
use crate::handlers::admin::request::{AdminAppState, AdminRequestContext};
use crate::handlers::admin::shared::attach_admin_audit_response;
use crate::GatewayError;
use axum::{
    body::Body,
    http,
    response::{IntoResponse, Response},
    Json,
};

const PROVIDER_AGENT_IDENTITY_IMPORT_KIND: &str = "agent_identity";
const PROVIDER_OAUTH_BATCH_IMPORT_KIND: &str = "oauth_batch";
const PROVIDER_COOKIE_AUTHORIZE_IMPORT_KIND: &str = "cookie_authorize";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ProviderOAuthTaskRouteKind {
    BatchImport,
    AgentIdentity,
    CookieAuthorize,
}

fn provider_oauth_import_task_matches_route(
    task_id: &str,
    payload: &serde_json::Value,
    route_kind: ProviderOAuthTaskRouteKind,
) -> bool {
    let has_agent_prefix = task_id.starts_with("agent-identity-");
    let has_cookie_prefix = task_id.starts_with("claude-cookie-");
    let import_kind = payload
        .get("import_kind")
        .and_then(serde_json::Value::as_str);
    match route_kind {
        ProviderOAuthTaskRouteKind::BatchImport => {
            !has_agent_prefix
                && !has_cookie_prefix
                && matches!(
                    import_kind,
                    None | Some("") | Some(PROVIDER_OAUTH_BATCH_IMPORT_KIND)
                )
        }
        ProviderOAuthTaskRouteKind::AgentIdentity => {
            has_agent_prefix && import_kind == Some(PROVIDER_AGENT_IDENTITY_IMPORT_KIND)
        }
        ProviderOAuthTaskRouteKind::CookieAuthorize => {
            has_cookie_prefix && import_kind == Some(PROVIDER_COOKIE_AUTHORIZE_IMPORT_KIND)
        }
    }
}

pub(super) async fn handle_admin_provider_oauth_batch_import_task_status(
    state: &AdminAppState<'_>,
    request_context: &AdminRequestContext<'_>,
) -> Result<Response<Body>, GatewayError> {
    handle_admin_provider_oauth_import_task_status(
        state,
        request_context,
        ProviderOAuthTaskRouteKind::BatchImport,
    )
    .await
}

pub(super) async fn handle_admin_provider_oauth_agent_identity_import_task_status(
    state: &AdminAppState<'_>,
    request_context: &AdminRequestContext<'_>,
) -> Result<Response<Body>, GatewayError> {
    handle_admin_provider_oauth_import_task_status(
        state,
        request_context,
        ProviderOAuthTaskRouteKind::AgentIdentity,
    )
    .await
}

pub(super) async fn handle_admin_provider_oauth_cookie_task_status(
    state: &AdminAppState<'_>,
    request_context: &AdminRequestContext<'_>,
) -> Result<Response<Body>, GatewayError> {
    handle_admin_provider_oauth_import_task_status(
        state,
        request_context,
        ProviderOAuthTaskRouteKind::CookieAuthorize,
    )
    .await
}

async fn handle_admin_provider_oauth_import_task_status(
    state: &AdminAppState<'_>,
    request_context: &AdminRequestContext<'_>,
    route_kind: ProviderOAuthTaskRouteKind,
) -> Result<Response<Body>, GatewayError> {
    let not_found_detail = match route_kind {
        ProviderOAuthTaskRouteKind::BatchImport => "批量导入任务不存在或已过期",
        ProviderOAuthTaskRouteKind::AgentIdentity => "Agent Identity 导入任务不存在或已过期",
        ProviderOAuthTaskRouteKind::CookieAuthorize => "Cookie 授权任务不存在或已过期",
    };
    let task_path = match route_kind {
        ProviderOAuthTaskRouteKind::BatchImport => {
            admin_provider_oauth_batch_import_task_path(request_context.path())
        }
        ProviderOAuthTaskRouteKind::AgentIdentity => {
            admin_provider_oauth_agent_identity_import_task_path(request_context.path())
        }
        ProviderOAuthTaskRouteKind::CookieAuthorize => {
            admin_provider_oauth_cookie_task_path(request_context.path())
        }
    };
    let Some((provider_id, task_id)) = task_path else {
        return Ok(build_internal_control_error_response(
            http::StatusCode::NOT_FOUND,
            not_found_detail,
        ));
    };
    let payload = match state
        .read_provider_oauth_batch_task_payload(&provider_id, &task_id)
        .await
    {
        Ok(Some(payload)) => payload,
        Ok(None) => {
            return Ok(build_internal_control_error_response(
                http::StatusCode::NOT_FOUND,
                not_found_detail,
            ));
        }
        Err(_) => {
            return Ok(build_internal_control_error_response(
                http::StatusCode::SERVICE_UNAVAILABLE,
                "provider oauth batch task redis unavailable",
            ));
        }
    };
    if !provider_oauth_import_task_matches_route(&task_id, &payload, route_kind) {
        return Ok(build_internal_control_error_response(
            http::StatusCode::NOT_FOUND,
            not_found_detail,
        ));
    }
    let status = payload
        .get("status")
        .and_then(serde_json::Value::as_str)
        .map(ToOwned::to_owned)
        .unwrap_or_default();
    let response = Json(payload).into_response();
    let (completed_event, failed_event, action, target_type) = match route_kind {
        ProviderOAuthTaskRouteKind::AgentIdentity => (
            "admin_provider_oauth_agent_identity_import_completed_viewed",
            "admin_provider_oauth_agent_identity_import_failed_viewed",
            "view_provider_agent_identity_import_terminal_state",
            "provider_agent_identity_import_task",
        ),
        ProviderOAuthTaskRouteKind::CookieAuthorize => (
            "admin_provider_oauth_cookie_task_completed_viewed",
            "admin_provider_oauth_cookie_task_failed_viewed",
            "view_provider_oauth_cookie_task_terminal_state",
            "provider_oauth_cookie_task",
        ),
        ProviderOAuthTaskRouteKind::BatchImport => (
            "admin_provider_oauth_batch_task_completed_viewed",
            "admin_provider_oauth_batch_task_failed_viewed",
            "view_provider_oauth_batch_task_terminal_state",
            "provider_oauth_batch_task",
        ),
    };
    Ok(match status.as_str() {
        "completed" => attach_admin_audit_response(
            response,
            completed_event,
            action,
            target_type,
            &format!("{provider_id}:{task_id}"),
        ),
        "failed" => attach_admin_audit_response(
            response,
            failed_event,
            action,
            target_type,
            &format!("{provider_id}:{task_id}"),
        ),
        _ => response,
    })
}

#[cfg(test)]
mod tests {
    use super::{provider_oauth_import_task_matches_route, ProviderOAuthTaskRouteKind};
    use serde_json::json;

    #[test]
    fn import_task_status_routes_are_bidirectionally_isolated() {
        let agent_payload = json!({ "import_kind": "agent_identity" });
        let batch_payload = json!({ "import_kind": "oauth_batch" });
        let cookie_payload = json!({ "import_kind": "cookie_authorize" });

        assert!(provider_oauth_import_task_matches_route(
            "agent-identity-task-1",
            &agent_payload,
            ProviderOAuthTaskRouteKind::AgentIdentity,
        ));
        assert!(!provider_oauth_import_task_matches_route(
            "agent-identity-task-1",
            &agent_payload,
            ProviderOAuthTaskRouteKind::BatchImport,
        ));
        assert!(provider_oauth_import_task_matches_route(
            "batch-task-1",
            &batch_payload,
            ProviderOAuthTaskRouteKind::BatchImport,
        ));
        assert!(!provider_oauth_import_task_matches_route(
            "batch-task-1",
            &batch_payload,
            ProviderOAuthTaskRouteKind::AgentIdentity,
        ));
        assert!(provider_oauth_import_task_matches_route(
            "claude-cookie-task-1",
            &cookie_payload,
            ProviderOAuthTaskRouteKind::CookieAuthorize,
        ));
        for route_kind in [
            ProviderOAuthTaskRouteKind::BatchImport,
            ProviderOAuthTaskRouteKind::AgentIdentity,
        ] {
            assert!(!provider_oauth_import_task_matches_route(
                "claude-cookie-task-1",
                &cookie_payload,
                route_kind,
            ));
        }
        for (task_id, payload) in [
            ("batch-task-1", &batch_payload),
            ("agent-identity-task-1", &agent_payload),
        ] {
            assert!(!provider_oauth_import_task_matches_route(
                task_id,
                payload,
                ProviderOAuthTaskRouteKind::CookieAuthorize,
            ));
        }
        assert!(!provider_oauth_import_task_matches_route(
            "claude-cookie-task-1",
            &batch_payload,
            ProviderOAuthTaskRouteKind::CookieAuthorize,
        ));
        assert!(provider_oauth_import_task_matches_route(
            "legacy-batch-task",
            &json!({}),
            ProviderOAuthTaskRouteKind::BatchImport,
        ));
    }
}

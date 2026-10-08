use aether_routing_core::RoutingGroupConfig;
use axum::{
    body::Body,
    http,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

use super::{
    build_auth_error_response, resolve_authenticated_local_user, AppState,
    GatewayPublicRequestContext,
};
use crate::routing::selection::routing_group_is_user_visible;

pub(super) async fn handle_users_me_routing_groups_get(
    state: &AppState,
    request_context: &GatewayPublicRequestContext,
    headers: &http::HeaderMap,
) -> Response<Body> {
    if let Err(response) = resolve_authenticated_local_user(state, request_context, headers).await {
        return response;
    }
    if !state.has_routing_group_data_reader() {
        return build_auth_error_response(
            http::StatusCode::SERVICE_UNAVAILABLE,
            "策略分组目录暂不可用",
            false,
        );
    }
    let groups = match state.list_routing_groups().await {
        Ok(groups) => groups,
        Err(error) => {
            return build_auth_error_response(
                http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("user routing group lookup failed: {error:?}"),
                false,
            )
        }
    };
    let items = groups
        .into_iter()
        .filter(|group| group.enabled && routing_group_is_user_visible(group))
        .filter_map(|group| {
            let config = serde_json::from_value::<RoutingGroupConfig>(group.config_json).ok()?;
            if !config.billing_multiplier.is_finite() || config.billing_multiplier < 0.0 {
                return None;
            }
            Some(json!({
                "id": group.id,
                "name": group.name,
                "billing_multiplier": config.billing_multiplier,
                "is_default": group.is_system_default,
            }))
        })
        .collect::<Vec<_>>();
    Json(json!({"total": items.len(), "items": items})).into_response()
}

mod monitoring;
mod routes;
mod stats;
mod usage;

pub(super) use self::monitoring::maybe_build_local_admin_monitoring_response;
pub(super) use self::routes::maybe_build_local_admin_observability_response;
pub(crate) use self::stats::{
    admin_stats_bad_request_response, maybe_build_local_admin_stats_response, parse_bounded_u32,
    round_to,
};
pub(crate) use self::stats::{AdminStatsTimeRange, AdminStatsUsageFilter};
pub(crate) use self::usage::maybe_build_local_admin_usage_response;

pub(crate) async fn resolve_usage_user_group_scope(
    state: &crate::handlers::admin::request::AdminAppState<'_>,
    query: Option<&str>,
    include_inactive: bool,
    exclude_admin: bool,
) -> Result<Result<Option<Vec<String>>, String>, crate::GatewayError> {
    let group_id = crate::handlers::admin::shared::query_param_value(query, "user_group_id");
    let Some(group_id) = group_id else {
        return Ok(Ok(None));
    };
    if crate::handlers::admin::shared::query_param_value(query, "user_id").is_some() {
        return Ok(Err(
            "user_id and user_group_id cannot be used together".to_string()
        ));
    }
    if !state.has_user_data_reader() {
        return Ok(Err("user group data is unavailable".to_string()));
    }

    match state
        .resolve_usage_user_group_member_ids(&group_id, include_inactive, exclude_admin)
        .await?
    {
        Some(user_ids) => Ok(Ok(Some(user_ids))),
        None => Ok(Err("user_group_id does not exist".to_string())),
    }
}

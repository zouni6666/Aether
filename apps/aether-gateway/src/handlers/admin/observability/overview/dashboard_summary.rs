use crate::handlers::admin::request::{AdminAppState, AdminRequestContext};
use crate::GatewayError;
use aether_admin::observability::analytics::{dashboard_summary_value, parse_dashboard_query};
use axum::{
    body::Body,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Json,
};

pub(super) async fn response(
    state: &AdminAppState<'_>,
    context: &AdminRequestContext<'_>,
) -> Result<Response<Body>, GatewayError> {
    let query = match parse_dashboard_query(context.query_string()) {
        Ok(query) => query,
        Err(detail) => return Ok(super::error(StatusCode::BAD_REQUEST, &detail)),
    };
    if !state.as_ref().has_usage_data_reader() {
        return Ok(super::error(
            StatusCode::SERVICE_UNAVAILABLE,
            "dashboard statistics are unavailable",
        ));
    }
    let snapshot = match tokio::time::timeout(
        std::time::Duration::from_secs(5),
        state.as_ref().data.query_dashboard_summary(&query),
    )
    .await
    {
        Ok(Ok(snapshot)) => snapshot,
        Ok(Err(error)) => return Err(GatewayError::Internal(error.to_string())),
        Err(_) => {
            return Ok(super::error(
                StatusCode::GATEWAY_TIMEOUT,
                "dashboard statistics exceeded their time budget",
            ))
        }
    };
    let mut value = dashboard_summary_value(&snapshot);
    value["concurrency"] = state
        .as_ref()
        .today_concurrency(&query.timezone)
        .map_err(GatewayError::Internal)?;
    Ok(([(header::CACHE_CONTROL, "private, no-store")], Json(value)).into_response())
}

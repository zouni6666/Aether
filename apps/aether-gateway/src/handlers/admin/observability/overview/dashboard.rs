use crate::handlers::admin::request::{AdminAppState, AdminRequestContext};
use crate::GatewayError;
use aether_admin::observability::analytics::{dashboard_value, parse_dashboard_query};
use axum::{
    body::Body,
    http::{self, StatusCode},
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
            "usage analytics is unavailable",
        ));
    }
    let snapshot = match tokio::time::timeout(
        std::time::Duration::from_secs(15),
        state.as_ref().query_dashboard_analytics(&query),
    )
    .await
    {
        Ok(result) => result?,
        Err(_) => {
            return Ok(super::error(
                StatusCode::GATEWAY_TIMEOUT,
                "dashboard query exceeded its time budget",
            ))
        }
    };
    let data = dashboard_value(&query, &snapshot).map_err(GatewayError::Internal)?;
    Ok((
        [(http::header::CACHE_CONTROL, "private, no-store")],
        Json(data),
    )
        .into_response())
}

pub(super) async fn total_response(
    state: &AdminAppState<'_>,
    context: &AdminRequestContext<'_>,
) -> Result<Response<Body>, GatewayError> {
    use crate::cache::OverviewTotalRead;
    use aether_data_contracts::repository::usage::UsageDashboardAnalyticsQuery;
    use serde_json::json;
    use std::sync::Arc;
    use std::time::{Duration, Instant};

    let query = match parse_dashboard_query(context.query_string()) {
        Ok(query) => query,
        Err(detail) => return Ok(super::error(StatusCode::BAD_REQUEST, &detail)),
    };
    if !state.as_ref().has_usage_data_reader() {
        return Ok(super::error(
            StatusCode::SERVICE_UNAVAILABLE,
            "usage analytics is unavailable",
        ));
    }
    let (cached, refresh) = state.as_ref().overview_total_cache.read(Instant::now());
    if let Some(refresh) = refresh {
        let app = state.as_ref();
        let data = if app.background_data.has_usage_reader() {
            Arc::clone(&app.background_data)
        } else {
            Arc::clone(&app.data)
        };
        // Lifetime boundaries do not depend on the viewer's timezone. Every
        // administrator shares one refresh, including after a page reload.
        tokio::spawn(async move {
            let query = UsageDashboardAnalyticsQuery {
                timezone: "UTC".into(),
            };
            let result = tokio::time::timeout(
                Duration::from_secs(185),
                data.query_dashboard_analytics(&query),
            )
            .await;
            let snapshot = match result {
                Ok(Ok(snapshot)) => Some(snapshot),
                Ok(Err(error)) => {
                    tracing::warn!(%error, "dashboard lifetime refresh failed");
                    None
                }
                Err(_) => {
                    tracing::warn!("dashboard lifetime refresh exceeded its time budget");
                    None
                }
            };
            refresh.finish(snapshot, Instant::now());
        });
    }
    let (status, body, retry_after) = match cached {
        OverviewTotalRead::Pending => {
            (StatusCode::ACCEPTED, json!({"status":"pending"}), Some("3"))
        }
        OverviewTotalRead::Failed => (
            StatusCode::SERVICE_UNAVAILABLE,
            json!({"status":"failed", "detail":"cumulative dashboard totals are temporarily unavailable; retry shortly"}),
            Some("10"),
        ),
        OverviewTotalRead::Ready { snapshot, stale } => {
            let mut value = dashboard_value(&query, &snapshot).map_err(GatewayError::Internal)?;
            (
                StatusCode::OK,
                json!({
                    "status":"ready", "total": value["total"].take(),
                    "history_complete": snapshot.history_complete, "stale": stale,
                }),
                None,
            )
        }
    };
    let mut response = (
        status,
        [(http::header::CACHE_CONTROL, "private, no-store")],
        Json(body),
    )
        .into_response();
    if let Some(retry_after) = retry_after {
        response.headers_mut().insert(
            http::header::RETRY_AFTER,
            http::HeaderValue::from_static(retry_after),
        );
    }
    Ok(response)
}

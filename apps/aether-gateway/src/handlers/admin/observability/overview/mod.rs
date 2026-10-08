mod dashboard;
mod dashboard_summary;
mod live;

use crate::handlers::admin::request::{AdminAppState, AdminRequestContext};
use crate::GatewayError;
use aether_admin::observability::analytics::{
    costs_value, dashboard_charts_value, envelope, export_csv, metrics_value, page_value,
    parse_dashboard_charts_query, parse_overview_query, performance_value, user_finance_value,
    user_payments_value,
};
use aether_data_contracts::repository::usage::{UsageAnalyticsGranularity, UsageAnalyticsView};
use axum::{
    body::Body,
    http::{self, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

pub(crate) async fn maybe_build_overview_response(
    state: &AdminAppState<'_>,
    context: &AdminRequestContext<'_>,
) -> Result<Option<Response<Body>>, GatewayError> {
    if context.route_family() != Some("overview_manage") || context.method() != http::Method::GET {
        return Ok(None);
    }
    let kind = context.route_kind().unwrap_or_default();
    if kind == "dashboard_summary" {
        return dashboard_summary::response(state, context).await.map(Some);
    }
    if kind == "dashboard_total" {
        return dashboard::total_response(state, context).await.map(Some);
    }
    if kind == "dashboard" {
        return dashboard::response(state, context).await.map(Some);
    }
    if matches!(kind, "operations_live" | "operations_resources") {
        return live::response(state, context).await.map(Some);
    }
    let view = match kind {
        "dashboard_charts" => UsageAnalyticsView::DashboardCharts,
        "summary" => UsageAnalyticsView::Summary,
        "timeseries" | "costs" => UsageAnalyticsView::Timeseries,
        "operations_performance" => UsageAnalyticsView::Performance,
        "users" | "user_detail" => UsageAnalyticsView::Users,
        "breakdown" => UsageAnalyticsView::Breakdown,
        "consumption" => UsageAnalyticsView::Consumption,
        _ => return Ok(None),
    };
    let parsed = if kind == "dashboard_charts" {
        parse_dashboard_charts_query(context.query_string())
    } else {
        parse_overview_query(context.query_string(), view)
    };
    let mut request = match parsed {
        Ok(value) => value,
        Err(detail) => return Ok(Some(error(StatusCode::BAD_REQUEST, &detail))),
    };
    if kind == "user_detail" {
        let encoded = context
            .path()
            .trim_end_matches('/')
            .rsplit('/')
            .next()
            .unwrap_or_default();
        let Ok(id) = percent_encoding::percent_decode_str(encoded).decode_utf8() else {
            return Ok(Some(error(
                StatusCode::BAD_REQUEST,
                "invalid user identifier",
            )));
        };
        let id = id.as_ref();
        if id.is_empty() || id.len() > 512 || id.contains('/') || id.chars().any(char::is_control) {
            return Ok(Some(error(
                StatusCode::BAD_REQUEST,
                "invalid user identifier",
            )));
        }
        if request
            .query
            .actor_user_id
            .as_deref()
            .is_some_and(|value| value != id)
            || request
                .query
                .credential_owner_id
                .as_deref()
                .is_some_and(|value| value != id)
        {
            return Ok(Some(error(
                StatusCode::BAD_REQUEST,
                "user filter conflicts with the requested employee",
            )));
        }
        if request.query.attribution_kind.as_deref() == Some("employee") {
            request.query.actor_user_id = Some(id.into());
        } else {
            request.query.credential_owner_id = Some(id.into());
        }
        request.query.limit = 1;
        request.query.offset = 0;
    }
    if matches!(
        view,
        UsageAnalyticsView::Timeseries | UsageAnalyticsView::Performance
    ) {
        request.query.limit = 10_000;
        request.query.offset = 0;
    }
    if kind == "costs" {
        request.query.granularity = UsageAnalyticsGranularity::Day;
    }
    if !state.as_ref().has_usage_data_reader() {
        return Ok(Some(error(
            StatusCode::SERVICE_UNAVAILABLE,
            "usage analytics is unavailable",
        )));
    }
    let snapshot = match tokio::time::timeout(
        std::time::Duration::from_secs(if request.csv { 30 } else { 15 }),
        state.as_ref().query_usage_analytics(&request.query),
    )
    .await
    {
        Ok(result) => result?,
        Err(_) => {
            return Ok(Some(error(
                StatusCode::GATEWAY_TIMEOUT,
                "report query exceeded its time budget; narrow the range or filters",
            )))
        }
    };
    if request.csv {
        return Ok(Some(match export_csv(&request, &snapshot) {
            Ok(csv) => (
                [
                    (http::header::CONTENT_TYPE, "text/csv; charset=utf-8"),
                    (
                        http::header::CONTENT_DISPOSITION,
                        "attachment; filename=overview.csv",
                    ),
                    (http::header::CACHE_CONTROL, "private, no-store"),
                ],
                csv,
            )
                .into_response(),
            Err(detail) => error(StatusCode::UNPROCESSABLE_ENTITY, &detail),
        }));
    }
    let data = match kind {
        "dashboard_charts" => dashboard_charts_value(&snapshot),
        "summary" => metrics_value(&snapshot.summary),
        "user_detail" => {
            let Some(user) = snapshot.users.first() else {
                return Ok(Some(error(StatusCode::NOT_FOUND, "employee not found")));
            };
            json!({
                "user": { "id": user.user_id, "username": user.username, "email": user.email, "is_active": user.is_active },
                "summary": metrics_value(&user.metrics),
                "finance": user_finance_value(user.finance.as_ref()),
                "payments": user_payments_value(snapshot.user_payments.as_ref()),
            })
        }
        "costs" => costs_value(&request, &snapshot),
        "timeseries" => {
            let mut page = page_value(&request, &snapshot);
            page["granularity"] = json!(request.query.granularity);
            page
        }
        "operations_performance" => performance_value(&request, &snapshot),
        _ => page_value(&request, &snapshot),
    };
    let mut response = Json(envelope(&request, &snapshot, data)).into_response();
    response.headers_mut().insert(
        http::header::CACHE_CONTROL,
        http::HeaderValue::from_static("private, no-store"),
    );
    Ok(Some(response))
}

fn error(status: StatusCode, detail: &str) -> Response<Body> {
    (status, Json(json!({"detail": detail}))).into_response()
}

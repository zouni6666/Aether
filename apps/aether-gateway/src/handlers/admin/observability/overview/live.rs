use super::error;
use crate::handlers::admin::request::{AdminAppState, AdminRequestContext};
use crate::GatewayError;
use aether_admin::observability::analytics::{envelope, metrics_value, OverviewRequest};
use aether_data_contracts::repository::usage::{UsageAnalyticsQuery, USAGE_ANALYTICS_VERSION};
use axum::{
    body::Body,
    http::{header, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

pub(super) async fn response(
    state: &AdminAppState<'_>,
    context: &AdminRequestContext<'_>,
) -> Result<Response<Body>, GatewayError> {
    if context
        .query_string()
        .is_some_and(|query| !query.is_empty())
    {
        return Ok(error(
            StatusCode::BAD_REQUEST,
            "live diagnostics do not accept historical filters",
        ));
    }
    let app = state.as_ref();
    let _ = app.metric_samples().await;
    let snapshot = app.metric_snapshot.read().await.clone();
    let captured = snapshot.as_ref().map(|(captured, _)| *captured);
    let now = chrono::Utc::now();
    let observed_at = captured
        .and_then(|captured| chrono::Duration::from_std(captured.elapsed()).ok())
        .map(|age| now - age);
    let mut unavailable = Vec::new();
    let (resilience_result, recent_result) = tokio::join!(
        tokio::time::timeout(
            std::time::Duration::from_secs(3),
            super::super::monitoring::overview_resilience_payload(state)
        ),
        tokio::time::timeout(
            std::time::Duration::from_secs(3),
            recent_activity(state, now)
        ),
    );
    let resilience = match resilience_result {
        Ok(Ok(value)) => Some(value),
        _ => {
            tracing::warn!("overview resilience snapshot unavailable");
            unavailable.push("resilience");
            None
        }
    };
    let recent_activity = match recent_result {
        Ok(Ok(value)) => Some(value),
        _ => {
            unavailable.push("recent_activity");
            None
        }
    };
    if captured.is_none() {
        unavailable.push("metrics");
    }
    let mut response = Json(json!({
        "meta": {
            "schema_version": 1, "metric_version": USAGE_ANALYTICS_VERSION, "scope": {"kind": "node"},
            "generated_at": now, "data_through": observed_at, "read_revision": observed_at.map(|value| value.timestamp_millis().to_string()),
            "coverage": {"status": if unavailable.is_empty() {"complete"} else {"partial"}},
        },
        "data": {
            "observed_at": observed_at, "window_seconds": null, "node_id": null,
            "scope": {"kind": "node", "node_ids": []},
            "metrics_text": snapshot.map(|(_, samples)| aether_runtime::metrics::render_prometheus_text(&samples)),
            "resilience": resilience, "recent_activity": recent_activity,
            "execution_activity": app.execution_activity.snapshot(),
            "unavailable_sections": unavailable,
        },
    })).into_response();
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("private, no-store"),
    );
    Ok(response)
}

async fn recent_activity(
    state: &AdminAppState<'_>,
    now: chrono::DateTime<chrono::Utc>,
) -> Result<serde_json::Value, GatewayError> {
    let to = now.timestamp_millis().max(60_000) as u64;
    let request = OverviewRequest {
        query: UsageAnalyticsQuery {
            from_unix_ms: to - 60_000,
            to_unix_ms: to,
            timezone: "UTC".into(),
            limit: 1,
            ..Default::default()
        },
        amount_basis: "billable".into(),
        csv: false,
    };
    let snapshot = state.as_ref().query_usage_analytics(&request.query).await?;
    let data = recent_activity_data(&snapshot);
    Ok(envelope(&request, &snapshot, data))
}

fn recent_activity_data(
    snapshot: &aether_data_contracts::repository::usage::StoredUsageAnalytics,
) -> serde_json::Value {
    let mut data = metrics_value(&snapshot.summary);
    data["requests_per_second"] = json!(snapshot.summary.request_count as f64 / 60.0);
    data["requests_per_minute"] = json!(snapshot.summary.request_count);
    data["tokens_per_minute"] = data["total_tokens"].clone();
    data["window_seconds"] = json!(60);
    data
}

#[cfg(test)]
mod tests {
    use super::*;
    use aether_data_contracts::repository::usage::{StoredUsageAnalytics, UsageAnalyticsMetrics};

    #[test]
    fn recent_activity_reports_one_minute_rates_without_inventing_missing_tokens() {
        let mut snapshot = StoredUsageAnalytics {
            summary: UsageAnalyticsMetrics {
                request_count: 120,
                usage_available_count: 120,
                total_tokens: 4200,
                ..Default::default()
            },
            ..Default::default()
        };
        let value = recent_activity_data(&snapshot);
        assert_eq!(value["window_seconds"], 60);
        assert_eq!(value["requests_per_second"], 2.0);
        assert_eq!(value["requests_per_minute"], 120);
        assert_eq!(value["tokens_per_minute"], 4200);
        snapshot.summary.usage_available_count = 0;
        assert!(recent_activity_data(&snapshot)["tokens_per_minute"].is_null());
        snapshot.summary = UsageAnalyticsMetrics::default();
        assert_eq!(recent_activity_data(&snapshot)["tokens_per_minute"], 0);
    }
}

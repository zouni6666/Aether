use super::amount;
use aether_data_contracts::repository::usage::{DashboardSummaryMetrics, StoredDashboardSummary};
use serde_json::{json, Value};

/// The homepage reads a durable aggregate, not the full historical report.
pub fn dashboard_summary_value(snapshot: &StoredDashboardSummary) -> Value {
    let total = metrics_value(&snapshot.total);
    json!({
        "stats_since": snapshot.stats_since,
        "generated_at": snapshot.generated_at,
        "timezone": snapshot.timezone,
        "activity_timezone": snapshot.activity_timezone,
        "today_from": snapshot.today_from,
        "window_seconds": snapshot.window_seconds,
        "today": metrics_value(&snapshot.today),
        "total": {
            "request_count": total["request_count"],
            "total_tokens": total["total_tokens"],
            "cache_read_tokens": total["cache_read_tokens"],
            "cache_input_tokens": total["cache_input_tokens"],
            "billable_amount": total["billable_amount"],
        },
        "users": snapshot.users,
        "active_days": snapshot.active_days,
        "consecutive_active_days": snapshot.consecutive_active_days,
        "activity_days": snapshot.activity_days,
    })
}

fn metrics_value(metrics: &DashboardSummaryMetrics) -> Value {
    let empty = metrics.request_count == 0;
    let tokens_available = empty || metrics.usage_available_count > 0;
    let billable = if empty {
        Some("0.00000000".to_string())
    } else {
        metrics.billable_amount.clone()
    };
    let average = |sum: f64, samples: u64| {
        (samples > 0 && sum.is_finite() && sum >= 0.0).then(|| sum / samples as f64)
    };
    json!({
        "request_count": metrics.request_count,
        "input_tokens": tokens_available.then_some(metrics.input_tokens),
        "output_tokens": tokens_available.then_some(metrics.output_tokens),
        "total_tokens": tokens_available.then_some(metrics.total_tokens),
        "billable_amount": amount(&billable, "billable", metrics.pricing_available_count == metrics.request_count),
        "active_users": metrics.active_users,
        "cache_read_tokens": tokens_available.then_some(metrics.cache_read_tokens),
        "cache_creation_tokens": tokens_available.then_some(metrics.cache_creation_tokens),
        "cache_input_tokens": (tokens_available && metrics.cache_input_tokens >= metrics.cache_read_tokens)
            .then_some(metrics.cache_input_tokens),
        "avg_first_byte_ms": average(metrics.first_byte_sum_ms, metrics.first_byte_sample_count),
        "avg_response_ms": average(metrics.response_sum_ms, metrics.response_sample_count),
        "stream_requests": metrics.stream_requests,
        "standard_requests": metrics.standard_requests,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compact_summary_preserves_unknown_cost_and_weighted_response_samples() {
        let mut snapshot = StoredDashboardSummary {
            stats_since: "2026-09-19T00:00:00Z".into(),
            active_days: 90,
            consecutive_active_days: 12,
            today: DashboardSummaryMetrics {
                request_count: 3,
                response_sum_ms: 600.0,
                response_sample_count: 2,
                first_byte_sum_ms: 80.0,
                first_byte_sample_count: 1,
                ..Default::default()
            },
            total: DashboardSummaryMetrics {
                request_count: 3,
                usage_available_count: 3,
                cache_read_tokens: 120,
                cache_input_tokens: 400,
                pricing_available_count: 1,
                billable_amount: Some("1.23456789".into()),
                ..Default::default()
            },
            ..Default::default()
        };
        let value = dashboard_summary_value(&snapshot);
        assert_eq!(value["active_days"], 90);
        assert_eq!(value["consecutive_active_days"], 12);
        assert_eq!(value["today"]["avg_response_ms"], 300.0);
        assert_eq!(value["today"]["avg_first_byte_ms"], 80.0);
        assert_eq!(value["today"]["billable_amount"]["status"], "unknown");
        assert!(value["today"]["total_tokens"].is_null());
        assert_eq!(
            value["total"]["billable_amount"]["status"],
            "known_subtotal"
        );
        assert_eq!(value["total"]["billable_amount"]["value"], "1.23456789");
        assert_eq!(value["total"]["cache_read_tokens"], 120);
        assert_eq!(value["total"]["cache_input_tokens"], 400);
        assert!(value["today"].get("latency_p95_ms").is_none());

        snapshot.total.cache_input_tokens = 100;
        assert!(dashboard_summary_value(&snapshot)["total"]["cache_input_tokens"].is_null());

        snapshot.total.usage_available_count = 0;
        let unknown = dashboard_summary_value(&snapshot);
        assert!(unknown["total"].get("cache_read_tokens").unwrap().is_null());
        assert!(unknown["total"]
            .get("cache_input_tokens")
            .unwrap()
            .is_null());
    }

    #[test]
    fn empty_collection_has_zero_cost_but_no_measured_latency() {
        let value = dashboard_summary_value(&StoredDashboardSummary::default());
        assert_eq!(value["consecutive_active_days"], 0);
        assert_eq!(value["today"]["billable_amount"]["value"], "0.00000000");
        assert_eq!(value["today"]["billable_amount"]["status"], "known");
        assert!(value["today"]["avg_response_ms"].is_null());
        assert!(value["today"]["avg_first_byte_ms"].is_null());
        assert_eq!(value["total"]["request_count"], 0);
        assert_eq!(value["total"]["cache_read_tokens"], 0);
        assert_eq!(value["total"]["cache_input_tokens"], 0);
    }
}

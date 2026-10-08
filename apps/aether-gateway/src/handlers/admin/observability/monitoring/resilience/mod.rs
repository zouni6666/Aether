mod history;
mod reset;
mod snapshot;
mod status;

pub(super) use history::build_admin_monitoring_resilience_circuit_history_response;
pub(super) use reset::build_admin_monitoring_reset_error_stats_response;
pub(super) use status::build_admin_monitoring_resilience_status_response;

pub(in super::super) async fn overview_resilience_payload(
    state: &crate::handlers::admin::request::AdminAppState<'_>,
) -> Result<serde_json::Value, crate::GatewayError> {
    let snapshot = snapshot::build_admin_monitoring_resilience_snapshot(state).await?;
    let from = (snapshot.timestamp - chrono::Duration::hours(24))
        .timestamp()
        .max(
            state
                .admin_monitoring_error_stats_reset_at()
                .unwrap_or_default() as i64,
        );
    Ok(serde_json::json!({
        "scope": {"kind": "installation"},
        "error_range": {"from": chrono::DateTime::from_timestamp(from, 0), "to": snapshot.timestamp},
        "timestamp": snapshot.timestamp, "health_score": snapshot.health_score,
        "status": snapshot.status, "error_statistics": snapshot.error_statistics,
        "recent_errors": snapshot.recent_errors, "recommendations": snapshot.recommendations,
    }))
}

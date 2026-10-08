//! Compatibility reads for installations whose retained daily totals predate
//! the incremental dashboard. Never replay or mutate the activation boundary.
use crate::error::SqlxResultExt;
use aether_data_contracts::{repository::usage::*, DataLayerError};
use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::{Postgres, Row, Transaction};

pub(super) struct DashboardHistory {
    pub since: DateTime<Utc>,
    pub metrics: Value,
    pub days: Vec<DashboardActivityDay>,
}

pub(super) async fn read_dashboard_history(
    tx: &mut Transaction<'_, Postgres>,
    activation: DateTime<Utc>,
    now: DateTime<Utc>,
) -> Result<Option<DashboardHistory>, DataLayerError> {
    // Both timestamp and integer dates exist in supported imported schemas.
    let state = sqlx::query(
        r#"
      WITH summary AS (
        SELECT CASE WHEN pg_typeof(cutoff_date) IN ('bigint'::regtype,'integer'::regtype)
          THEN to_timestamp(cutoff_date::text::double precision)
          ELSE cutoff_date::text::timestamptz END AS cutoff
        FROM stats_summary ORDER BY updated_at DESC, created_at DESC LIMIT 1
      ), daily AS (
        SELECT CASE WHEN pg_typeof(date) IN ('bigint'::regtype,'integer'::regtype)
          THEN to_timestamp(date::text::double precision)
          ELSE date::text::timestamptz END AS day
        FROM stats_daily WHERE total_requests>0
      ) SELECT cutoff, (SELECT min(day) FROM daily WHERE day<LEAST(cutoff,$1)) AS since
        FROM summary WHERE cutoff<=$2"#,
    )
    .bind(activation)
    .bind(now)
    .fetch_optional(&mut **tx)
    .await
    .map_postgres_err()?;
    let Some(state) = state else {
        return Ok(None);
    };
    let Some(since) = state
        .try_get::<Option<DateTime<Utc>>, _>("since")
        .map_postgres_err()?
    else {
        return Ok(None);
    };
    let cutoff: DateTime<Utc> = state.try_get("cutoff").map_postgres_err()?;
    // Canonical fact views expand several joins. JIT compilation can cost more
    // than the small recent range itself on upgraded installations.
    sqlx::query("SET LOCAL jit=off")
        .execute(&mut **tx)
        .await
        .map_postgres_err()?;
    // The published summary cutoff is advanced atomically with daily rollups.
    // Read that same snapshot and use disjoint [history, cutoff), [cutoff, now)
    // ranges. This keeps working after raw history is pruned or the cutoff moves.
    let row = sqlx::query(include_str!("queries/dashboard_history.sql"))
        .bind(cutoff)
        .bind(now)
        .fetch_one(&mut **tx)
        .await
        .map_postgres_err()?;
    let days: Value = row.try_get("days").map_postgres_err()?;
    Ok(Some(DashboardHistory {
        since,
        metrics: row.try_get("metrics").map_postgres_err()?,
        days: serde_json::from_value(days)
            .map_err(|error| DataLayerError::UnexpectedValue(error.to_string()))?,
    }))
}

pub(super) async fn complete_activation_day(
    tx: &mut Transaction<'_, Postgres>,
    today: &DashboardSummaryMetrics,
    today_start: DateTime<Utc>,
    activation: DateTime<Utc>,
    now: DateTime<Utc>,
) -> Result<Value, DataLayerError> {
    // Match dashboard_request_fact exactly for the missing part of today.
    // The actor UNION deduplicates users present on both sides of activation.
    let row = sqlx::query(include_str!("queries/dashboard_activation_day.sql"))
        .bind(today_start)
        .bind(activation)
        .bind(now)
        .bind(
            serde_json::to_value(today)
                .map_err(|error| DataLayerError::UnexpectedValue(error.to_string()))?,
        )
        .fetch_one(&mut **tx)
        .await
        .map_postgres_err()?;
    row.try_get("metrics").map_postgres_err()
}

use super::analytics::{
    analytics_additive_metrics_sql, dashboard_total_metrics_sql, push_analytics_filter,
};
use crate::error::SqlxResultExt;
use aether_data_contracts::{repository::usage::*, DataLayerError};
use chrono::{DateTime, Utc};
use sqlx::{Postgres, QueryBuilder, Row};

pub(super) async fn read_projection_coverage(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    query: &UsageAnalyticsQuery,
    read_enabled: bool,
) -> Result<UsageAnalyticsProjectionCoverage, DataLayerError> {
    let row = sqlx::query(r#"WITH rounded AS (
      SELECT date_trunc('hour',$1::timestamptz AT TIME ZONE 'UTC') AT TIME ZONE 'UTC' AS start_at,
        date_trunc('hour',LEAST($2::timestamptz,NOW()) AT TIME ZONE 'UTC') AT TIME ZONE 'UTC' AS end_at
    ), bounds AS (
      SELECT CASE WHEN start_at<$1 THEN start_at+INTERVAL '1 hour' ELSE start_at END AS start_at,end_at FROM rounded
    ), pending AS (
      SELECT e.bucket_start,bool_or(e.unrecoverable) AS unrecoverable
      FROM stats_overview_dirty_events e CROSS JOIN bounds b
      WHERE e.projection_version='overview-v2' AND e.granularity='hour'
        AND e.bucket_start>=b.start_at AND e.bucket_start<b.end_at GROUP BY e.bucket_start
    ), stored AS (
      SELECT s.* FROM stats_bucket_state s CROSS JOIN bounds b
      WHERE s.projection_version='overview-v2' AND s.granularity='hour'
        AND s.bucket_start>=b.start_at AND s.bucket_start<b.end_at
    ), states AS (
      SELECT COALESCE(s.bucket_start,p.bucket_start) AS bucket,
        COALESCE(s.source_revision,0) AS source_revision,COALESCE(s.built_revision,-1) AS built_revision,
        CASE WHEN p.unrecoverable THEN 'unrecoverable'
          ELSE COALESCE(s.coverage_status,'unbuilt') END AS coverage_status,
        p.bucket_start IS NOT NULL AS pending,
        COALESCE(s.source_revision=s.built_revision AND s.coverage_status='complete',false)
          AND p.bucket_start IS NULL AS clean
      FROM stored s FULL JOIN pending p USING(bucket_start)
    ), clean AS (
      SELECT bucket,row_number() OVER(ORDER BY bucket) AS position FROM states WHERE clean
    ) SELECT CASE WHEN end_at>start_at THEN start_at END AS projection_from,
      CASE WHEN end_at>start_at THEN COALESCE((SELECT max(bucket)+INTERVAL '1 hour' FROM clean
        WHERE bucket=start_at+(position-1)*INTERVAL '1 hour'),start_at) END AS projection_through,
      (SELECT count(*) FROM states WHERE (source_revision>built_revision OR pending) AND coverage_status<>'unrecoverable') AS dirty,
      GREATEST(EXTRACT(EPOCH FROM(end_at-start_at))::bigint/3600,0)-(SELECT count(*) FROM states) AS missing
      FROM bounds"#)
        .bind(DateTime::<Utc>::from_timestamp_millis(query.from_unix_ms as i64).expect("validated"))
        .bind(DateTime::<Utc>::from_timestamp_millis(query.to_unix_ms as i64).expect("validated"))
        .fetch_one(&mut **tx).await.map_postgres_err()?;
    Ok(UsageAnalyticsProjectionCoverage {
        projection_from: row
            .try_get::<Option<DateTime<Utc>>, _>("projection_from")
            .map_postgres_err()?
            .map(|value| value.to_rfc3339()),
        projection_through: row
            .try_get::<Option<DateTime<Utc>>, _>("projection_through")
            .map_postgres_err()?
            .map(|value| value.to_rfc3339()),
        dirty_bucket_count: row.try_get::<i64, _>("dirty").map_postgres_err()? as u64,
        missing_bucket_count: row.try_get::<i64, _>("missing").map_postgres_err()? as u64,
        read_enabled,
    })
}

pub(super) fn supports_projection(query: &UsageAnalyticsQuery) -> bool {
    query.status.is_none()
        && query.endpoint_kind.is_none()
        && query.is_stream.is_none()
        && query.has_format_conversion.is_none()
        && query.slow_threshold_ms.is_none_or(|value| value == 5000)
}

const ADDITIVE_COUNTS: &[&str] = &[
    "request_count",
    "successful_request_count",
    "failed_request_count",
    "cancelled_request_count",
    "in_flight_request_count",
    "input_tokens",
    "output_tokens",
    "total_tokens",
    "cache_read_input_tokens",
    "cache_creation_input_tokens",
    "usage_available_count",
    "pricing_available_count",
    "settled_count",
    "allocation_available_count",
    "trusted_attribution_count",
    "classified_failure_count",
    "latency_sample_count",
    "slow_request_count",
    "first_byte_sample_count",
    "output_tps_sample_count",
    "reported_usage_count",
    "estimated_usage_count",
    "mixed_usage_count",
    "unknown_usage_count",
    "cache_pricing_available_count",
];

const ADDITIVE_AMOUNTS: &[&str] = &[
    "rated_amount",
    "billable_amount",
    "quota_covered_amount",
    "wallet_consumed_amount",
    "wallet_debit_amount",
    "wallet_recharge_debit_amount",
    "wallet_gift_debit_amount",
    "wallet_overdraft_amount",
    "cache_read_cost_amount",
    "cache_creation_cost_amount",
    "cache_estimated_full_cost_amount",
];

/// Closed verified buckets and raw gaps are disjoint within the caller's read snapshot.
pub(super) async fn read_additive_summary(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    query: &UsageAnalyticsQuery,
) -> Result<UsageAnalyticsMetrics, DataLayerError> {
    read_projected_metrics(
        tx,
        query,
        &analytics_additive_metrics_sql(5000),
        ADDITIVE_COUNTS,
        &["latency_sum_ms", "first_byte_sum_ms", "output_tps_sum"],
        ADDITIVE_AMOUNTS,
        false,
    )
    .await
}

pub(super) async fn read_dashboard_total_summary(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    query: &UsageAnalyticsQuery,
) -> Result<UsageAnalyticsMetrics, DataLayerError> {
    read_projected_metrics(
        tx,
        query,
        dashboard_total_metrics_sql(),
        &[
            "request_count",
            "total_tokens",
            "usage_available_count",
            "pricing_available_count",
            "settled_count",
            "allocation_available_count",
        ],
        &[],
        &["billable_amount"],
        true,
    )
    .await
}

async fn read_projected_metrics(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    query: &UsageAnalyticsQuery,
    raw_metrics_sql: &str,
    counts: &[&str],
    float_sums: &[&str],
    amounts: &[&str],
    dashboard_total: bool,
) -> Result<UsageAnalyticsMetrics, DataLayerError> {
    // Normal writers create UTC-aligned buckets. Ignore any manually supplied
    // nonaligned state so the equality exclusion below still partitions facts.
    // A writer can commit after a builder's snapshot without touching bucket state.
    // Its event is committed atomically with the facts, so the same read snapshot
    // must exclude that bucket until the event has been merged and rebuilt.
    let mut builder = QueryBuilder::<Postgres>::new(
        "WITH valid AS MATERIALIZED (SELECT bucket_start FROM stats_bucket_state WHERE projection_version='overview-v2' AND granularity='hour' AND source_revision=built_revision AND coverage_status='complete' AND NOT EXISTS (SELECT 1 FROM stats_overview_dirty_events e WHERE e.projection_version=stats_bucket_state.projection_version AND e.granularity=stats_bucket_state.granularity AND e.bucket_start=stats_bucket_state.bucket_start) AND bucket_start = date_trunc('hour', bucket_start AT TIME ZONE 'UTC') AT TIME ZONE 'UTC' AND bucket_start >= ",
    );
    builder.push_bind(DateTime::<Utc>::from_timestamp_millis(query.from_unix_ms as i64).expect("validated"))
        .push(" AND bucket_start + INTERVAL '1 hour' <= ").push_bind(DateTime::<Utc>::from_timestamp_millis(query.to_unix_ms as i64).expect("validated"))
        .push(" AND bucket_start + INTERVAL '1 hour' <= NOW()), pieces AS (SELECT p.metrics FROM stats_overview_hourly p JOIN valid v USING(bucket_start) WHERE p.projection_version='overview-v2'");
    for (column, value) in [
        ("actor_user_id", &query.actor_user_id),
        ("credential_owner_id", &query.credential_owner_id),
        ("attribution_kind", &query.attribution_kind),
        ("api_key_id", &query.api_key_id),
        ("model", &query.model),
        ("provider_id", &query.provider_id),
        ("api_format", &query.api_format),
        ("request_type", &query.request_type),
    ] {
        if let Some(value) = value {
            builder
                .push(" AND p.dimensions->>'")
                .push(column)
                .push("' = ")
                .push_bind(value.clone());
        }
    }
    builder
        .push(" UNION ALL SELECT to_jsonb(raw) FROM (SELECT ")
        .push(raw_metrics_sql);
    if dashboard_total {
        super::dashboard::push_dashboard_total_filter(&mut builder, query);
    } else {
        push_analytics_filter(&mut builder, query);
    }
    builder.push(" AND NOT EXISTS (SELECT 1 FROM valid WHERE valid.bucket_start = date_trunc('hour', usage_analytics_facts_v1.created_at AT TIME ZONE 'UTC') AT TIME ZONE 'UTC')) raw) SELECT to_jsonb(m) AS metrics FROM (SELECT ");
    for (index, name) in counts.iter().enumerate() {
        if index > 0 {
            builder.push(",");
        }
        builder
            .push("COALESCE(sum((metrics->>'")
            .push(*name)
            .push("')::bigint),0)::bigint AS ")
            .push(*name);
    }
    for name in float_sums {
        builder
            .push(",COALESCE(sum((metrics->>'")
            .push(*name)
            .push("')::double precision),0)::double precision AS ")
            .push(*name);
    }
    for name in amounts {
        builder
            .push(",sum((metrics->>'")
            .push(*name)
            .push("')::numeric)::text AS ")
            .push(*name);
    }
    builder.push(" FROM pieces) m");
    let row = builder
        .build()
        .fetch_one(&mut **tx)
        .await
        .map_postgres_err()?;
    serde_json::from_value(row.try_get("metrics").map_postgres_err()?)
        .map_err(|error| DataLayerError::UnexpectedValue(error.to_string()))
}

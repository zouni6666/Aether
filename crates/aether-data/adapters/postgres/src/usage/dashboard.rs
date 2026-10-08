use super::{
    analytics::{dashboard_total_metrics_sql, read_analytics_metrics},
    projection_reader::{read_dashboard_total_summary, read_projection_coverage},
    SqlxUsageReadRepository,
};
use crate::error::SqlxResultExt;
use aether_data_contracts::{repository::usage::*, DataLayerError};
use chrono::{DateTime, Utc};
use sqlx::{Postgres, QueryBuilder, Row};

// This projection keeps the canonical billing token precedence while reading usage
// once. Joining the two public fact views reads and hashes the entire usage table
// twice. Extract both availability flags together so large JSON metadata is parsed
// once per row. Live database regressions compare these fields with the canonical view.
const DASHBOARD_TOTAL_FACTS_SQL: &str = r#"(
SELECT u.created_at, u.api_key_id, u.model, u.provider_id, u.api_format, u.endpoint_kind,
  u.request_type, u.status, u.is_stream, u.has_format_conversion, u.failure_origin,
  'request'::text AS record_kind,
  COALESCE(s.billing_status, u.billing_status) AS settlement_status,
  COALESCE(metadata.value->'usage_available', 'true'::jsonb) <> 'false'::jsonb AS usage_available,
  COALESCE(metadata.value->'usage_pricing_available', 'true'::jsonb) <> 'false'::jsonb
    AND (s.billing_total_cost_usd IS NOT NULL OR COALESCE(s.billing_status, u.billing_status) = 'settled') AS pricing_available,
  CASE WHEN COALESCE(metadata.value->'usage_available', 'true'::jsonb) <> 'false'::jsonb THEN
  GREATEST(
    COALESCE(
      CASE
        WHEN s.billing_effective_input_tokens IS NOT NULL
        THEN GREATEST(s.billing_effective_input_tokens, 0)
          + GREATEST(COALESCE(s.billing_output_tokens, u.output_tokens, 0), 0)
          + GREATEST(
              COALESCE(
                s.billing_cache_creation_tokens,
                CASE
                  WHEN s.billing_cache_creation_5m_tokens IS NOT NULL
                    OR s.billing_cache_creation_1h_tokens IS NOT NULL
                  THEN COALESCE(s.billing_cache_creation_5m_tokens, 0)
                    + COALESCE(s.billing_cache_creation_1h_tokens, 0)
                END,
                CASE
                  WHEN COALESCE(u.cache_creation_input_tokens, 0) = 0
                       AND (
                         COALESCE(u.cache_creation_input_tokens_5m, 0)
                         + COALESCE(u.cache_creation_input_tokens_1h, 0)
                       ) > 0
                  THEN COALESCE(u.cache_creation_input_tokens_5m, 0)
                    + COALESCE(u.cache_creation_input_tokens_1h, 0)
                  ELSE COALESCE(u.cache_creation_input_tokens, 0)
                END,
                0
              ),
              0
            )
          + GREATEST(
              COALESCE(
                s.billing_cache_read_tokens,
                u.cache_read_input_tokens,
                0
              ),
              0
            )
        WHEN s.billing_total_input_context IS NOT NULL
        THEN GREATEST(s.billing_total_input_context, 0)
          + GREATEST(COALESCE(s.billing_output_tokens, u.output_tokens, 0), 0)
      END,
      NULLIF(GREATEST(COALESCE(u.total_tokens, 0), 0), 0),
      CASE
        WHEN split_part(lower(COALESCE(COALESCE(u.endpoint_api_format, u.api_format), '')), ':', 1)
             IN ('openai', 'gemini', 'google')
        THEN GREATEST(COALESCE(u.input_tokens, 0), 0)
          + GREATEST(COALESCE(u.output_tokens, 0), 0)
        ELSE GREATEST(COALESCE(u.input_tokens, 0), 0)
          + GREATEST(COALESCE(u.output_tokens, 0), 0)
          + GREATEST(
              CASE
                WHEN COALESCE(u.cache_creation_input_tokens, 0) = 0
                     AND (
                       COALESCE(u.cache_creation_input_tokens_5m, 0)
                       + COALESCE(u.cache_creation_input_tokens_1h, 0)
                     ) > 0
                THEN COALESCE(u.cache_creation_input_tokens_5m, 0)
                  + COALESCE(u.cache_creation_input_tokens_1h, 0)
                ELSE COALESCE(u.cache_creation_input_tokens, 0)
              END,
              0
            )
          + GREATEST(COALESCE(u.cache_read_input_tokens, 0), 0)
      END,
      0
    ),
    0
  )::bigint END AS total_tokens,
  CASE WHEN COALESCE(metadata.value->'usage_pricing_available', 'true'::jsonb) <> 'false'::jsonb
    AND (s.billing_actual_total_cost_usd IS NOT NULL OR COALESCE(s.billing_status, u.billing_status) = 'settled')
    THEN public.usage_customer_billable_amount(metadata.value,
      COALESCE(s.billing_total_cost_usd::numeric, u.total_cost_usd::numeric),
      COALESCE(s.billing_actual_total_cost_usd::numeric, u.actual_total_cost_usd::numeric)) END AS billable_amount,
  s.allocation_status
FROM public.usage u
LEFT JOIN public.usage_settlement_snapshots s USING (request_id)
CROSS JOIN LATERAL (SELECT u.request_metadata::jsonb AS value OFFSET 0) metadata
WHERE NOT EXISTS (SELECT 1 FROM public.usage_attribution_snapshots a
  WHERE a.request_id=u.request_id AND a.record_kind='session')
) AS usage_analytics_facts_v1"#;

pub(super) fn push_dashboard_total_filter(
    builder: &mut QueryBuilder<'_, Postgres>,
    query: &UsageAnalyticsQuery,
) {
    super::analytics::push_analytics_source_filter(builder, query, DASHBOARD_TOTAL_FACTS_SQL);
}

/// Lifetime cards only need additive totals and their coverage. Performance and
/// active-user metrics remain exact in the separate today snapshot.
pub(super) async fn read_dashboard_total_metrics(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    query: &UsageAnalyticsQuery,
    projection_reads: bool,
) -> Result<UsageAnalyticsMetrics, DataLayerError> {
    // A materialized valid-bucket CTE prevents parallel aggregation of raw gaps.
    // Upgraded installations can have years of raw history and no built buckets;
    // use the parallel raw query directly until there is a bucket to combine.
    let has_projection = if projection_reads {
        sqlx::query_scalar::<_, bool>(r#"SELECT EXISTS (
          SELECT 1 FROM stats_bucket_state WHERE projection_version='overview-v2'
            AND granularity='hour' AND source_revision=built_revision AND coverage_status='complete'
            AND NOT EXISTS (SELECT 1 FROM stats_overview_dirty_events e
              WHERE e.projection_version=stats_bucket_state.projection_version
                AND e.granularity=stats_bucket_state.granularity
                AND e.bucket_start=stats_bucket_state.bucket_start)
            AND bucket_start = date_trunc('hour', bucket_start AT TIME ZONE 'UTC') AT TIME ZONE 'UTC'
            AND bucket_start >= $1 AND bucket_start + INTERVAL '1 hour' <= $2
            AND bucket_start + INTERVAL '1 hour' <= NOW())"#)
            .bind(DateTime::<Utc>::from_timestamp_millis(query.from_unix_ms as i64).expect("validated"))
            .bind(DateTime::<Utc>::from_timestamp_millis(query.to_unix_ms as i64).expect("validated"))
            .fetch_one(&mut **tx).await.map_postgres_err()?
    } else {
        false
    };
    if has_projection {
        return read_dashboard_total_summary(tx, query).await;
    }
    let mut builder = QueryBuilder::<Postgres>::new("SELECT to_jsonb(m) AS metrics FROM (SELECT ");
    builder.push(dashboard_total_metrics_sql());
    push_dashboard_total_filter(&mut builder, query);
    builder.push(") m");
    let row = builder
        .build()
        .fetch_one(&mut **tx)
        .await
        .map_postgres_err()?;
    serde_json::from_value(row.try_get("metrics").map_postgres_err()?)
        .map_err(|error| DataLayerError::UnexpectedValue(error.to_string()))
}

impl SqlxUsageReadRepository {
    pub async fn query_dashboard_analytics(
        &self,
        query: &UsageDashboardAnalyticsQuery,
    ) -> Result<StoredUsageDashboardAnalytics, DataLayerError> {
        query.validate()?;
        let mut tx = self.pool.begin().await.map_postgres_err()?;
        sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ, READ ONLY")
            .execute(&mut *tx)
            .await
            .map_postgres_err()?;
        sqlx::query("SET LOCAL statement_timeout='180s'")
            .execute(&mut *tx)
            .await
            .map_postgres_err()?;
        let state=sqlx::query(r#"SELECT date_trunc('milliseconds',NOW()) AS now,pg_current_snapshot()::text AS revision,
          (SELECT min(created_at) FROM usage WHERE created_at<NOW()) AS raw_from,
          (SELECT min(bucket_start) FROM (SELECT bucket_start FROM stats_bucket_state WHERE projection_version IN ('overview-v1','overview-v2') AND granularity='hour' UNION ALL SELECT bucket_start FROM stats_overview_dirty_events WHERE projection_version='overview-v2' AND granularity='hour') buckets WHERE bucket_start<NOW()) AS state_from,
          (SELECT CASE WHEN pg_typeof(min(date)) IN ('bigint'::regtype,'integer'::regtype)
            THEN to_timestamp(min(date)::text::double precision) ELSE min(date)::text::timestamptz END
            FROM stats_daily WHERE total_requests>0) AS legacy_from,
          (SELECT count(*) FROM users WHERE is_active AND NOT is_deleted) AS enabled_users"#)
            .fetch_one(&mut *tx).await.map_postgres_err()?;
        let now: DateTime<Utc> = state.try_get("now").map_postgres_err()?;
        let raw_from: Option<DateTime<Utc>> = state.try_get("raw_from").map_postgres_err()?;
        let state_from: Option<DateTime<Utc>> = state.try_get("state_from").map_postgres_err()?;
        let legacy_from: Option<DateTime<Utc>> = state.try_get("legacy_from").map_postgres_err()?;
        let total_from = raw_from.into_iter().chain(state_from).min();
        let today_from = query.today_start(now)?;
        let revision: String = state.try_get("revision").map_postgres_err()?;
        let enabled_users = state
            .try_get::<i64, _>("enabled_users")
            .map_postgres_err()? as u64;
        let mut snapshots = Vec::with_capacity(2);
        for (from, lifetime) in [(today_from, false), (total_from.unwrap_or(now), true)] {
            let range = UsageAnalyticsQuery {
                from_unix_ms: from.timestamp_millis().max(0) as u64,
                to_unix_ms: now.timestamp_millis().max(0) as u64,
                timezone: query.timezone.clone(),
                limit: 1,
                ..Default::default()
            };
            let mut summary = if lifetime {
                read_dashboard_total_metrics(&mut tx, &range, self.overview_projection_reads)
                    .await?
            } else {
                read_analytics_metrics(&mut tx, &range, self.overview_projection_reads).await?
            };
            summary.enabled_users = enabled_users;
            let unrecoverable=sqlx::query_scalar::<_,i64>("SELECT count(DISTINCT bucket_start) FROM (SELECT bucket_start FROM stats_bucket_state WHERE projection_version IN ('overview-v1','overview-v2') AND granularity='hour' AND coverage_status='unrecoverable' UNION SELECT bucket_start FROM stats_overview_dirty_events WHERE projection_version='overview-v2' AND granularity='hour' AND unrecoverable) pending WHERE bucket_start<$2 AND bucket_start+INTERVAL '1 hour'>$1")
                .bind(from).bind(now).fetch_one(&mut *tx).await.map_postgres_err()? as u64;
            if summary.request_count == 0 && unrecoverable == 0 {
                summary.rated_amount = Some("0.00000000".into());
                summary.billable_amount = Some("0.00000000".into());
            }
            let coverage =
                read_projection_coverage(&mut tx, &range, self.overview_projection_reads).await?;
            snapshots.push(StoredUsageAnalytics {
                total: summary.request_count,
                summary,
                read_revision: revision.clone(),
                generated_at: now.to_rfc3339(),
                unrecoverable_bucket_count: unrecoverable,
                coverage,
                ..Default::default()
            });
        }
        let mut total = snapshots.pop().expect("total");
        let today = snapshots.pop().expect("today");
        let known_gap = total.unrecoverable_bucket_count > 0
            || legacy_from.is_some_and(|legacy| {
                raw_from.is_none_or(|raw| legacy.date_naive() < raw.date_naive())
            });
        if known_gap && total.summary.request_count == 0 {
            total.summary.rated_amount = None;
            total.summary.billable_amount = None;
        }
        tx.commit().await.map_postgres_err()?;
        Ok(StoredUsageDashboardAnalytics {
            today,
            total,
            today_from: today_from.to_rfc3339(),
            total_from: total_from.map(|value| value.to_rfc3339()),
            to: now.to_rfc3339(),
            history_complete: known_gap.then_some(false),
        })
    }
}

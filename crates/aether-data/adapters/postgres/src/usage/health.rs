use super::SqlxUsageReadRepository;
use crate::error::SqlxResultExt;
use aether_data_contracts::repository::usage::*;
use aether_data_contracts::DataLayerError;
use chrono::{DateTime, Utc};
use sqlx::Row;
use std::collections::BTreeMap;

impl SqlxUsageReadRepository {
    pub async fn summarize_health_observations(
        &self,
        query: &HealthObservationQuery,
    ) -> Result<HealthObservationSummary, DataLayerError> {
        if query.from_unix_ms >= query.to_unix_ms
            || query.to_unix_ms - query.from_unix_ms > 31 * 86_400_000
            || query.segments == 0
            || query.segments > 96
            || query.to_unix_ms > 253_402_300_799_000
        {
            return Err(DataLayerError::InvalidInput(
                "invalid health observation window".into(),
            ));
        }
        let (request_object, attempt_object) = match query.object_kind {
            HealthObservationObjectKind::ApiFormat => ("u.api_format", "u.api_format"),
            HealthObservationObjectKind::Model => ("u.model", "u.model"),
            HealthObservationObjectKind::Provider => ("u.provider_id", "c.provider_id"),
        };
        let sql = format!(
            r#"
WITH observations AS (
 SELECT {request_object}::text AS object_value, u.created_at AS event_at, 'request' AS kind,
   u.status, u.failure_origin, u.failure_stage, u.failure_reason, u.response_time_ms
 FROM usage_analytics_facts_v1 u WHERE u.created_at >= $1 AND u.created_at < $2 AND u.record_kind <> 'session'
 UNION ALL
 SELECT {attempt_object}::text AS object_value, c.created_at AS event_at, 'attempt' AS kind,
   c.status::text, NULL, NULL, NULL, NULL::bigint
 FROM request_candidates c LEFT JOIN usage_analytics_facts_v1 u ON u.request_id = c.request_id
 WHERE c.created_at >= $1 AND c.created_at < $2
   AND (c.status IN ('streaming','success','failed','cancelled') OR (c.status = 'pending' AND c.started_at IS NOT NULL))
), scoped AS (
 SELECT *, LEAST($4 - 1, FLOOR(EXTRACT(EPOCH FROM (event_at - $1)) * 1000 / $5)::integer) AS segment,
   (status IN ('failed','cancelled') AND ((status = 'cancelled' AND failure_origin = 'client') OR (failure_origin = 'client' AND
       (failure_stage IN ('authentication','admission') OR failure_reason IN ('invalid_input','invalid_credentials','quota_exceeded','policy_rejection'))))) IS TRUE AS excluded
 FROM observations WHERE object_value IS NOT NULL AND ($3::text[] IS NULL OR object_value = ANY($3))
), aggregates AS (
 SELECT object_value, segment, GROUPING(object_value) AS all_objects, GROUPING(segment) AS all_segments,
 count(*) FILTER (WHERE kind = 'request')::bigint AS request_count,
 count(*) FILTER (WHERE kind = 'request' AND status = 'completed')::bigint AS succeeded_count,
 count(*) FILTER (WHERE kind = 'request' AND status = 'failed')::bigint AS failed_count,
 count(*) FILTER (WHERE kind = 'request' AND status NOT IN ('completed','failed','cancelled'))::bigint AS in_progress_count,
 count(*) FILTER (WHERE kind = 'request' AND status = 'cancelled')::bigint AS cancelled_count,
 count(*) FILTER (WHERE kind = 'request' AND status = 'completed')::bigint AS service_succeeded_count,
 count(*) FILTER (WHERE kind = 'request' AND status IN ('failed','cancelled') AND NOT excluded AND failure_origin IN ('gateway','upstream','transport'))::bigint AS service_failed_count,
 count(*) FILTER (WHERE kind = 'request' AND excluded)::bigint AS excluded_count,
 count(*) FILTER (WHERE kind = 'request' AND status IN ('failed','cancelled') AND NOT excluded AND (failure_origin IS NULL OR failure_origin NOT IN ('gateway','upstream','transport')))::bigint AS unknown_failure_count,
 count(*) FILTER (WHERE kind = 'attempt' AND status = 'success')::bigint AS attempt_succeeded_count,
 count(*) FILTER (WHERE kind = 'attempt' AND status = 'failed')::bigint AS attempt_failed_count,
 count(*) FILTER (WHERE kind = 'attempt' AND status IN ('pending','streaming'))::bigint AS attempt_in_progress_count,
 count(*) FILTER (WHERE kind = 'attempt' AND status = 'cancelled')::bigint AS attempt_cancelled_count,
 COALESCE(sum(response_time_ms) FILTER (WHERE kind = 'request'),0)::double precision AS latency_sum_ms,
 count(response_time_ms) FILTER (WHERE kind = 'request')::bigint AS latency_sample_count,
 (EXTRACT(EPOCH FROM max(event_at) FILTER (WHERE kind = 'request')) * 1000)::bigint AS last_request_at_unix_ms
 FROM scoped GROUP BY GROUPING SETS ((object_value,segment),(object_value),(segment),())
) SELECT object_value, segment, all_objects, all_segments,
 to_jsonb(aggregates) - 'object_value' - 'segment' - 'all_objects' - 'all_segments' AS metrics FROM aggregates
"#
        );
        let mut tx = self.pool.begin().await.map_postgres_err()?;
        sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ, READ ONLY")
            .execute(&mut *tx)
            .await
            .map_postgres_err()?;
        sqlx::query("SET LOCAL statement_timeout = '15s'")
            .execute(&mut *tx)
            .await
            .map_postgres_err()?;
        let width = (query.to_unix_ms - query.from_unix_ms).div_ceil(u64::from(query.segments));
        let rows = sqlx::query(&sql)
            .bind(
                DateTime::<Utc>::from_timestamp_millis(query.from_unix_ms as i64)
                    .expect("validated timestamp"),
            )
            .bind(
                DateTime::<Utc>::from_timestamp_millis(query.to_unix_ms as i64)
                    .expect("validated timestamp"),
            )
            .bind(query.object_values.as_ref())
            .bind(query.segments as i32)
            .bind(width as f64)
            .fetch_all(&mut *tx)
            .await
            .map_postgres_err()?;
        let make_timeline = || {
            (0..query.segments)
                .filter_map(|segment| {
                    let from = query.from_unix_ms + u64::from(segment) * width;
                    (from < query.to_unix_ms).then(|| HealthObservationBucket {
                        from_unix_ms: from,
                        to_unix_ms: (from + width).min(query.to_unix_ms),
                        metrics: Default::default(),
                    })
                })
                .collect::<Vec<_>>()
        };
        let mut result = HealthObservationSummary {
            timeline: make_timeline(),
            ..Default::default()
        };
        let mut objects: BTreeMap<String, HealthObservationObject> = BTreeMap::new();
        for row in rows {
            let metrics: HealthObservationMetrics =
                serde_json::from_value(row.try_get("metrics").map_postgres_err()?)
                    .map_err(|error| DataLayerError::UnexpectedValue(error.to_string()))?;
            let all_objects: i32 = row.try_get("all_objects").map_postgres_err()?;
            let all_segments: i32 = row.try_get("all_segments").map_postgres_err()?;
            let segment: Option<i32> = row.try_get("segment").map_postgres_err()?;
            if all_objects == 1 {
                if all_segments == 1 {
                    result.overall = metrics;
                } else if let Some(bucket) =
                    segment.and_then(|segment| result.timeline.get_mut(segment as usize))
                {
                    bucket.metrics = metrics;
                }
            } else {
                let value: String = row.try_get("object_value").map_postgres_err()?;
                let object =
                    objects
                        .entry(value.clone())
                        .or_insert_with(|| HealthObservationObject {
                            object_value: value,
                            metrics: Default::default(),
                            timeline: make_timeline(),
                        });
                if all_segments == 1 {
                    object.metrics = metrics;
                } else if let Some(bucket) =
                    segment.and_then(|segment| object.timeline.get_mut(segment as usize))
                {
                    bucket.metrics = metrics;
                }
            }
        }
        result.objects = objects.into_values().collect();
        tx.commit().await.map_postgres_err()?;
        Ok(result)
    }
}

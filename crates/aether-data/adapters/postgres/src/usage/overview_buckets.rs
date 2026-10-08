use super::{analytics::analytics_additive_metrics_sql, SqlxUsageReadRepository};
use crate::error::SqlxResultExt;
use aether_data_contracts::DataLayerError;
use chrono::{DateTime, Utc};
use sqlx::Row;

impl SqlxUsageReadRepository {
    /// Move committed writer-owned events into bucket state in one transaction.
    /// Readers exclude both queued and merged dirtiness in their fact snapshot.
    /// Never acknowledge a high-water mark: transaction IDs can commit out of order.
    pub async fn merge_overview_dirty_events(&self) -> Result<u64, DataLayerError> {
        let mut tx = self.pool.begin().await.map_postgres_err()?;
        sqlx::query("SET LOCAL statement_timeout = '15s'")
            .execute(&mut *tx)
            .await
            .map_postgres_err()?;
        let acquired: bool = sqlx::query_scalar(
            "SELECT pg_try_advisory_xact_lock(hashtextextended('overview-dirty-merge', 19))",
        )
        .fetch_one(&mut *tx)
        .await
        .map_postgres_err()?;
        if !acquired {
            tx.rollback().await.map_postgres_err()?;
            return Ok(0);
        }
        let count: i64 = sqlx::query_scalar(
            r#"WITH selected AS MATERIALIZED (
              SELECT transaction_id,projection_version,granularity,bucket_start
              FROM stats_overview_dirty_events
              WHERE projection_version='overview-v2'
              ORDER BY transaction_id,projection_version,granularity,bucket_start
              LIMIT 10000 FOR UPDATE SKIP LOCKED
            ), removed AS (
              DELETE FROM stats_overview_dirty_events e USING selected s
              WHERE (e.transaction_id,e.projection_version,e.granularity,e.bucket_start)
                = (s.transaction_id,s.projection_version,s.granularity,s.bucket_start)
              RETURNING e.*
            ), merged AS (
              INSERT INTO stats_bucket_state(projection_version,granularity,bucket_start,
                source_revision,coverage_status,last_error)
              SELECT projection_version,granularity,bucket_start,count(*),
                CASE WHEN bool_or(unrecoverable) THEN 'unrecoverable' ELSE 'unbuilt' END,
                CASE WHEN bool_or(unrecoverable) THEN 'retained usage facts were deleted' END
              FROM removed GROUP BY projection_version,granularity,bucket_start
              ON CONFLICT (projection_version,granularity,bucket_start) DO UPDATE SET
                source_revision=stats_bucket_state.source_revision+EXCLUDED.source_revision,
                coverage_status=CASE WHEN EXCLUDED.coverage_status='unrecoverable'
                  THEN 'unrecoverable' ELSE stats_bucket_state.coverage_status END,
                last_error=CASE WHEN EXCLUDED.coverage_status='unrecoverable'
                  THEN EXCLUDED.last_error ELSE stats_bucket_state.last_error END
              RETURNING 1
            ) SELECT count(*) FROM removed"#,
        )
        .fetch_one(&mut *tx)
        .await
        .map_postgres_err()?;
        tx.commit().await.map_postgres_err()?;
        Ok(count as u64)
    }

    /// Rebuild buckets marked dirty by normal writes without backfilling historical facts.
    pub async fn rebuild_overview_buckets(
        &self,
        target: DateTime<Utc>,
        budget: usize,
    ) -> Result<usize, DataLayerError> {
        let budget = budget.min(48);
        if budget == 0 {
            return Ok(0);
        }
        let merged_events = self.merge_overview_dirty_events().await?;
        let rows = sqlx::query(
            r#"SELECT granularity,bucket_start FROM stats_bucket_state
          WHERE projection_version='overview-v2' AND source_revision > built_revision
            AND source_revision > 0
            AND coverage_status <> 'unrecoverable'
            AND (last_failed_at IS NULL OR last_failed_at < NOW() - INTERVAL '10 minutes')
            AND bucket_start + ('1 ' || granularity)::interval <= $1
          ORDER BY last_failed_at NULLS FIRST,bucket_start,granularity LIMIT $2"#,
        )
        .bind(target)
        .bind(budget as i64)
        .fetch_all(&self.pool)
        .await
        .map_postgres_err()?;
        let mut published = 0;
        for row in rows {
            let granularity: String = row.try_get("granularity").map_postgres_err()?;
            let bucket: DateTime<Utc> = row.try_get("bucket_start").map_postgres_err()?;
            let started = std::time::Instant::now();
            match self
                .rebuild_merged_overview_bucket(&granularity, bucket)
                .await
            {
                Ok(true) => published += 1,
                Ok(false) => {}
                Err(error) => {
                    let error_detail = error.to_string().chars().take(500).collect::<String>();
                    tracing::warn!(
                        event_name = "overview_bucket_rebuild_failed",
                        log_type = "ops",
                        projection_version = "overview-v2",
                        granularity = %granularity,
                        bucket_start = %bucket,
                        elapsed_ms = started.elapsed().as_millis() as u64,
                        retry_after_secs = 600,
                        error = %error_detail,
                        "overview bucket rebuild failed; retry deferred"
                    );
                    sqlx::query("UPDATE stats_bucket_state SET last_error=$3,last_failed_at=NOW() WHERE projection_version='overview-v2' AND granularity=$1 AND bucket_start=$2")
                        .bind(&granularity).bind(bucket).bind(error_detail)
                        .execute(&self.pool).await.map_postgres_err()?;
                }
            }
        }
        // A busy current hour has no closed bucket to publish yet. Still report
        // progress so the bounded maintenance catch-up loop drains the next batch.
        Ok(published.max(usize::from(merged_events > 0)))
    }

    #[cfg(test)]
    pub(super) async fn rebuild_overview_bucket(
        &self,
        granularity: &str,
        bucket: DateTime<Utc>,
    ) -> Result<bool, DataLayerError> {
        self.merge_overview_dirty_events().await?;
        self.rebuild_merged_overview_bucket(granularity, bucket)
            .await
    }

    async fn rebuild_merged_overview_bucket(
        &self,
        granularity: &str,
        bucket: DateTime<Utc>,
    ) -> Result<bool, DataLayerError> {
        let (table, duration) = match granularity {
            "hour" => ("stats_overview_hourly", chrono::Duration::hours(1)),
            "day" => ("stats_overview_daily", chrono::Duration::days(1)),
            _ => {
                return Err(DataLayerError::InvalidInput(
                    "invalid overview bucket granularity".into(),
                ));
            }
        };
        let mut tx = self.pool.begin().await.map_postgres_err()?;
        sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ")
            .execute(&mut *tx)
            .await
            .map_postgres_err()?;
        sqlx::query("SET LOCAL statement_timeout = '15s'")
            .execute(&mut *tx)
            .await
            .map_postgres_err()?;
        sqlx::query("SET LOCAL lock_timeout = '1s'")
            .execute(&mut *tx)
            .await
            .map_postgres_err()?;
        // These bounded aggregates have many expressions but run only once per
        // bucket. JIT compilation consumes a significant part of their timeout.
        // Keep the setting transaction-local so other pool users retain theirs.
        sqlx::query("SET LOCAL jit = off")
            .execute(&mut *tx)
            .await
            .map_postgres_err()?;
        let acquired: bool =
            sqlx::query_scalar("SELECT pg_try_advisory_xact_lock(hashtextextended($1, 19))")
                .bind(format!("overview-v2:{granularity}:{}", bucket.timestamp()))
                .fetch_one(&mut *tx)
                .await
                .map_postgres_err()?;
        if !acquired {
            tx.rollback().await.map_postgres_err()?;
            return Ok(false);
        }
        let revision: Option<i64> = sqlx::query_scalar("SELECT source_revision FROM stats_bucket_state WHERE projection_version='overview-v2' AND granularity=$1 AND bucket_start=$2")
            .bind(granularity).bind(bucket).fetch_optional(&mut *tx).await.map_postgres_err()?;
        let Some(revision) = revision else {
            // Another merger may be running, or a bounded batch has not reached this bucket yet.
            tx.rollback().await.map_postgres_err()?;
            return Ok(false);
        };
        let lost: bool = sqlx::query_scalar("SELECT coverage_status='unrecoverable' FROM stats_bucket_state WHERE projection_version='overview-v2' AND granularity=$1 AND bucket_start=$2")
            .bind(granularity).bind(bucket).fetch_one(&mut *tx).await.map_postgres_err()?;
        if lost {
            tx.rollback().await.map_postgres_err()?;
            return Ok(false);
        }
        sqlx::query(&format!(
            "DELETE FROM {table} WHERE projection_version='overview-v2' AND bucket_start=$1"
        ))
        .bind(bucket)
        .execute(&mut *tx)
        .await
        .map_postgres_err()?;
        let sql = format!(
            r#"INSERT INTO {table}(projection_version,bucket_start,dimensions,metrics)
          SELECT 'overview-v2',$1,dimensions,to_jsonb(m) - 'dimensions'
          FROM (SELECT dimensions, {} FROM (
            SELECT *, jsonb_build_object('attribution_kind',attribution_kind,'actor_user_id',actor_user_id,
              'credential_owner_id',credential_owner_id,'api_key_id',api_key_id,'model',model,
              'provider_id',provider_id,'api_format',api_format,'request_type',request_type,'record_kind',record_kind) dimensions
            FROM usage_analytics_facts_v1 WHERE created_at >= $1 AND created_at < $2 AND record_kind <> 'session'
          ) facts GROUP BY dimensions) m"#,
            analytics_additive_metrics_sql(5000)
        );
        sqlx::query(&sql)
            .bind(bucket)
            .bind(bucket + duration)
            .execute(&mut *tx)
            .await
            .map_postgres_err()?;
        let published = sqlx::query("UPDATE stats_bucket_state SET built_revision=$3,coverage_status='complete',built_at=NOW(),last_error=NULL,last_failed_at=NULL WHERE projection_version='overview-v2' AND granularity=$1 AND bucket_start=$2 AND source_revision=$3")
            .bind(granularity).bind(bucket).bind(revision).execute(&mut *tx).await.map_postgres_err()?.rows_affected();
        if published != 1 {
            tx.rollback().await.map_postgres_err()?;
            return Ok(false);
        }
        tx.commit().await.map_postgres_err()?;
        Ok(true)
    }
}

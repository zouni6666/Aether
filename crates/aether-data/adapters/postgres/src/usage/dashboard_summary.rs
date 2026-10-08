use super::dashboard_history::{complete_activation_day, read_dashboard_history};
use super::SqlxUsageReadRepository;
use crate::error::SqlxResultExt;
use aether_data_contracts::{repository::usage::*, DataLayerError};
use chrono::{DateTime, NaiveDate, Timelike, Utc};
use serde_json::Value;
use sqlx::Row;

fn decode_metrics(mut value: Value) -> Result<DashboardSummaryMetrics, DataLayerError> {
    let count = value
        .get("request_count")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let priced = value
        .get("pricing_available_count")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    value["billable_amount"] = if count == 0 {
        Value::String("0.00000000".into())
    } else if priced == 0 {
        Value::Null
    } else {
        // The SQL converts NUMERIC to text before JSON serialization; no f64 step.
        value.get("billable_amount").cloned().unwrap_or(Value::Null)
    };
    serde_json::from_value(value)
        .map_err(|error| DataLayerError::UnexpectedValue(error.to_string()))
}

impl SqlxUsageReadRepository {
    pub async fn query_dashboard_summary(
        &self,
        query: &UsageDashboardAnalyticsQuery,
    ) -> Result<StoredDashboardSummary, DataLayerError> {
        query.validate()?;
        let mut tx = self.pool.begin().await.map_postgres_err()?;
        sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ, READ ONLY")
            .execute(&mut *tx)
            .await
            .map_postgres_err()?;
        sqlx::query("SET LOCAL statement_timeout='5s'")
            .execute(&mut *tx)
            .await
            .map_postgres_err()?;
        let state = sqlx::query("SELECT stats_since, CURRENT_TIMESTAMP AS now, (SELECT count(*) FROM users WHERE NOT is_deleted) AS users FROM dashboard_stats_state WHERE singleton")
            .fetch_one(&mut *tx).await.map_postgres_err()?;
        let since: DateTime<Utc> = state.try_get("stats_since").map_postgres_err()?;
        let now: DateTime<Utc> = state.try_get("now").map_postgres_err()?;
        let today_start = query.today_start(now)?;
        let history = read_dashboard_history(&mut tx, since, now).await?;
        let today_from = if history.is_some() {
            today_start
        } else {
            today_start.max(since)
        };
        let minute_from = today_start
            .max(since)
            .with_second(0)
            .and_then(|v| v.with_nanosecond(0))
            .expect("valid minute");
        let metrics_rows = sqlx::query(r#"
          WITH selected AS (
            SELECT 'total' AS period, metrics FROM dashboard_stats_total
            UNION ALL
            SELECT 'today', metrics FROM dashboard_stats_minute WHERE bucket_start >= $1 AND bucket_start <= $2
          ), sums AS (
            SELECT period, key, sum(value::numeric) AS amount FROM selected
            CROSS JOIN LATERAL jsonb_each_text(metrics) GROUP BY period,key
          ) SELECT period, jsonb_object_agg(key, CASE WHEN key='billable_amount'
            THEN to_jsonb(round(amount,8)::text) ELSE to_jsonb(amount) END) AS metrics
          FROM sums GROUP BY period"#)
            .bind(minute_from).bind(now).fetch_all(&mut *tx).await.map_postgres_err()?;
        let mut today = decode_metrics(serde_json::json!({}))?;
        let mut total = today.clone();
        for row in metrics_rows {
            let metrics = decode_metrics(row.try_get("metrics").map_postgres_err()?)?;
            match row
                .try_get::<String, _>("period")
                .map_postgres_err()?
                .as_str()
            {
                "today" => today = metrics,
                _ => total = metrics,
            }
        }
        today.active_users = sqlx::query_scalar::<_,i64>("SELECT count(DISTINCT a.actor_user_id) FROM dashboard_actor_minute a JOIN users u ON u.id=a.actor_user_id AND NOT u.is_deleted WHERE a.bucket_start >= $1 AND a.bucket_start <= $2 AND a.request_count > 0")
            .bind(minute_from).bind(now).fetch_one(&mut *tx).await.map_postgres_err()? as u64;
        if history.is_some() && since > today_start {
            today = decode_metrics(
                complete_activation_day(&mut tx, &today, today_start, since, now).await?,
            )?;
        }
        // Most UTC hours belong to one local day. Only hours straddling a local
        // midnight (e.g. Kathmandu) need their minute counts read. Lifetime
        // activity therefore scans narrow hourly counters, not metric JSON.
        let activity_timezone = if history.is_some() {
            "UTC"
        } else {
            query.timezone.as_str()
        };
        let mut days = if history.is_none() {
            sqlx::query(r#"WITH hours AS MATERIALIZED (
            SELECT bucket_start,shard,request_count,
              (bucket_start AT TIME ZONE $1)::date AS day,
              ((bucket_start+INTERVAL '1 hour'-INTERVAL '1 microsecond') AT TIME ZONE $1)::date AS last_day
            FROM dashboard_activity_hour WHERE request_count>0 AND bucket_start <= $2
          ), daily AS (
            SELECT day, request_count AS requests FROM hours WHERE day=last_day
            UNION ALL
            SELECT (m.bucket_start AT TIME ZONE $1)::date, m.request_count
            FROM hours h JOIN LATERAL (
              SELECT bucket_start,request_count FROM dashboard_activity_minute
              WHERE shard=h.shard AND bucket_start>=h.bucket_start
                AND bucket_start<h.bucket_start+INTERVAL '1 hour' AND bucket_start<=$2
              UNION ALL
              SELECT w.bucket_start,COALESCE((w.metrics->>'request_count')::bigint,0)
              FROM dashboard_stats_minute w
              WHERE w.shard=h.shard AND w.bucket_start>=h.bucket_start
                AND w.bucket_start<h.bucket_start+INTERVAL '1 hour' AND w.bucket_start<=$2
                AND NOT EXISTS(SELECT 1 FROM dashboard_activity_minute n
                  WHERE n.bucket_start=w.bucket_start AND n.shard=w.shard)
              OFFSET 0
            ) m ON TRUE WHERE h.day<>h.last_day
          ) SELECT day AS date, sum(requests)::bigint AS requests FROM daily
            GROUP BY day HAVING sum(requests)>0 ORDER BY day"#)
            .bind(activity_timezone).bind(now).fetch_all(&mut *tx).await.map_postgres_err()?
            .into_iter().map(|row| Ok(DashboardActivityDay {
                date: row.try_get::<NaiveDate, _>("date").map_postgres_err()?.to_string(),
                requests: row.try_get::<i64, _>("requests").map_postgres_err()?.max(0) as u64,
            })).collect::<Result<Vec<_>, DataLayerError>>()?
        } else {
            Vec::new()
        };
        let mut display_since = since;
        if let Some(history) = history {
            display_since = history.since;
            total = decode_metrics(history.metrics)?;
            days = history.days;
        }
        let local_today = now
            .with_timezone(
                &activity_timezone
                    .parse::<chrono_tz::Tz>()
                    .map_err(|_| DataLayerError::InvalidInput("invalid timezone".into()))?,
            )
            .date_naive();
        let first_heatmap_day = local_today - chrono::Duration::days(364);
        let active_days = days.len() as u64;
        let dates: Vec<NaiveDate> = days
            .iter()
            .map(|row| {
                NaiveDate::parse_from_str(&row.date, "%Y-%m-%d")
                    .map_err(|error| DataLayerError::UnexpectedValue(error.to_string()))
            })
            .collect::<Result<_, _>>()?;
        let consecutive_active_days =
            dashboard_consecutive_active_days(dates.iter().copied(), local_today);
        let mut activity_days = Vec::new();
        for (row, date) in days.into_iter().zip(dates) {
            if date >= first_heatmap_day {
                activity_days.push(DashboardActivityDay {
                    date: date.to_string(),
                    requests: row.requests,
                });
            }
        }
        let events = sqlx::query("SELECT COALESCE(sum(created_count),0)::bigint AS created, COALESCE(sum(deleted_count),0)::bigint AS deleted FROM dashboard_user_events_minute WHERE bucket_start >= $1 AND bucket_start <= $2")
            .bind(minute_from).bind(now).fetch_one(&mut *tx).await.map_postgres_err()?;
        let summary = StoredDashboardSummary {
            stats_since: display_since.to_rfc3339(),
            generated_at: now.to_rfc3339(),
            timezone: query.timezone.clone(),
            activity_timezone: activity_timezone.to_string(),
            today_from: today_from.to_rfc3339(),
            window_seconds: (now - today_from).num_milliseconds().max(0) as f64 / 1000.0,
            today,
            total,
            users: DashboardUserCounts {
                total: state.try_get::<i64, _>("users").map_postgres_err()?.max(0) as u64,
                created_today: events
                    .try_get::<i64, _>("created")
                    .map_postgres_err()?
                    .max(0) as u64,
                deleted_today: events
                    .try_get::<i64, _>("deleted")
                    .map_postgres_err()?
                    .max(0) as u64,
            },
            active_days,
            consecutive_active_days,
            activity_days,
        };
        tx.commit().await.map_postgres_err()?;
        Ok(summary)
    }
}

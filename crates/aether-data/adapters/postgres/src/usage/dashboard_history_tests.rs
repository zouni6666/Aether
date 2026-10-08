use super::SqlxUsageReadRepository;
use aether_data_contracts::repository::usage::*;
use chrono::{DateTime, Duration, Utc};
use futures_util::FutureExt;
use std::{collections::BTreeMap, panic::AssertUnwindSafe};

#[tokio::test]
#[ignore = "requires local AETHER_TEST_DATABASE_URL with temporary database creation"]
async fn live_dashboard_restores_legacy_history_without_replaying_or_double_counting() {
    let options = std::env::var("AETHER_TEST_DATABASE_URL")
        .unwrap()
        .parse::<sqlx::postgres::PgConnectOptions>()
        .unwrap();
    let admin = sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .connect_with(options.clone())
        .await
        .unwrap();
    let database = format!("dashboard_history_{}", uuid::Uuid::new_v4().simple());
    sqlx::query(&format!("CREATE DATABASE {database}"))
        .execute(&admin)
        .await
        .unwrap();
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(4)
        .connect_with(options.database(&database))
        .await
        .unwrap();

    let outcome = AssertUnwindSafe(async {
        crate::POSTGRES_MIGRATOR.run(&pool).await.unwrap();
        let repo = SqlxUsageReadRepository::new(pool.clone());
        let query = UsageDashboardAnalyticsQuery {
            timezone: "Asia/Kathmandu".into(),
        };
        let now: DateTime<Utc> = sqlx::query_scalar("SELECT CURRENT_TIMESTAMP")
            .fetch_one(&pool)
            .await
            .unwrap();
        let utc_today = now.date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc();
        let local_today = query.today_start(now).unwrap();
        let activation = local_today
            + Duration::microseconds((now - local_today).num_microseconds().unwrap() / 2);
        let before_activation = local_today
            + Duration::microseconds((activation - local_today).num_microseconds().unwrap() / 2);
        let after_activation = activation
            + Duration::microseconds((now - activation).num_microseconds().unwrap() / 2);
        let oldest = utc_today - Duration::days(400);
        let recent_history = utc_today - Duration::days(3);
        let cutoff = utc_today - Duration::days(2);
        sqlx::query("UPDATE dashboard_stats_state SET stats_since=$1")
            .bind(activation)
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO users(id,username,email_verified,is_deleted) VALUES ('shared','shared',false,false),('before-only','before-only',false,false),('deleted','deleted',false,true)")
            .execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO stats_summary(id,cutoff_date) VALUES('legacy',$1)")
            .bind(cutoff).execute(&pool).await.unwrap();

        // The oldest day's request rows have already expired. Its exact NUMERIC
        // cost and lifetime activity must survive outside the visible heatmap.
        for (id, date, requests, input, output, creation, read, cost) in [
            ("expired", oldest, 10, 1000_i64, 200_i64, 30_i64, 40_i64, "123456789.12345678"),
            ("recent", recent_history, 3, 300, 60, 9, 12, "0.30000003"),
            ("at-cutoff", cutoff, 1, 100, 20, 3, 4, "0.10000001"),
        ] {
            sqlx::query("INSERT INTO stats_daily(id,date,total_requests,input_tokens,output_tokens,cache_creation_tokens,cache_read_tokens,actual_total_cost) VALUES($1,$2,$3,$4,$5,$6,$7,$8::text::numeric)")
                .bind(id).bind(date).bind(requests).bind(input).bind(output)
                .bind(creation).bind(read).bind(cost).execute(&pool).await.unwrap();
        }
        // Retained detail below the cutoff must not be counted a second time;
        // a request exactly at the cutoff must still be included once.
        sqlx::query("INSERT INTO usage(id,request_id,model,provider_name,status,billing_status,input_tokens,output_tokens,total_tokens,cache_creation_input_tokens,cache_read_input_tokens,actual_total_cost_usd,created_at) VALUES('overlap','overlap','m','p','completed','settled',999,999,1998,0,0,999,$1),('boundary','boundary','m','p','completed','settled',100,20,127,3,4,0.10000001,$2)")
            .bind(recent_history + Duration::hours(12)).bind(cutoff)
            .execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO usage(id,request_id,model,provider_name,status,billing_status,input_tokens,output_tokens,total_tokens,actual_total_cost_usd,created_at) VALUES('pending','pending','m','p','pending','pending',999,999,1998,999,$1),('unknown','unknown','m','unknown','completed','settled',999,999,1998,999,$1)")
            .bind(cutoff + Duration::hours(12)).execute(&pool).await.unwrap();
        for (id, user, at) in [
            ("before-shared", "shared", before_activation),
            ("before-unique", "before-only", before_activation),
            ("before-deleted", "deleted", before_activation),
            ("after-shared", "shared", after_activation),
        ] {
            sqlx::query("INSERT INTO usage(id,request_id,user_id,model,provider_name,status,billing_status,input_tokens,output_tokens,total_tokens,actual_total_cost_usd,total_cost_usd,created_at,first_byte_time_ms,response_time_ms,upstream_is_stream) VALUES($1,$1,$2,'m','p','completed','settled',100,20,120,0.25,0.25,$3,100,800,true)")
                .bind(id).bind(user).bind(at).execute(&pool).await.unwrap();
        }
        // Today's missing prefix must use final settlement facts, just like
        // the already-active projection, instead of stale capture counters.
        sqlx::query("INSERT INTO usage_settlement_snapshots(request_id,billing_status,billing_input_tokens,billing_effective_input_tokens,billing_output_tokens,billing_cache_read_tokens,billing_cache_creation_tokens,billing_total_input_context,billing_total_cost_usd,billing_actual_total_cost_usd) VALUES('before-shared','settled',100,100,30,40,10,150,0.12345678,0.12345678)")
            .execute(&pool).await.unwrap();

        let restored = repo.query_dashboard_summary(&query).await.unwrap();
        assert_eq!(restored.total.request_count, 18);
        assert_eq!(restored.total.total_tokens, 2318);
        assert_eq!(restored.total.billable_amount.as_deref(), Some("123456790.39691360"));
        assert_eq!(restored.stats_since, oldest.to_rfc3339());
        assert_eq!(restored.timezone, query.timezone);
        assert_eq!(restored.activity_timezone, "UTC");
        assert_eq!(restored.today_from, local_today.to_rfc3339());
        assert_eq!(restored.today.request_count, 4);
        assert_eq!(restored.today.total_tokens, 540);
        assert_eq!(restored.today.billable_amount.as_deref(), Some("0.87345678"));
        assert_eq!(restored.today.active_users, 2, "users on both sides of activation count once; deleted users do not count");
        assert_eq!(restored.today.first_byte_sample_count, 4);
        assert_eq!(restored.today.response_sample_count, 4);
        assert_eq!(restored.today.response_sum_ms, 3200.0);
        assert_eq!(restored.today.stream_requests, 4);

        let mut expected_days = BTreeMap::from([
            (recent_history.date_naive().to_string(), 3_u64),
            (cutoff.date_naive().to_string(), 1),
        ]);
        *expected_days.entry(before_activation.date_naive().to_string()).or_default() += 3;
        *expected_days.entry(after_activation.date_naive().to_string()).or_default() += 1;
        assert_eq!(restored.active_days, expected_days.len() as u64 + 1);
        assert_eq!(restored.activity_days.iter().map(|day| (day.date.clone(), day.requests)).collect::<BTreeMap<_, _>>(), expected_days);
        assert_eq!(sqlx::query_scalar::<_, DateTime<Utc>>("SELECT stats_since FROM dashboard_stats_state WHERE singleton")
            .fetch_one(&pool).await.unwrap(), activation, "reading history must not change the projection activation boundary");
        assert_eq!(sqlx::query_scalar::<_, i64>("SELECT count(*) FROM dashboard_request_contributions")
            .fetch_one(&pool).await.unwrap(), 1, "old requests are read, not replayed into projection ledgers");

        // The daily aggregator advances its exclusive boundary over the same
        // request. Existing daily rows above the old cutoff were not duplicates.
        sqlx::query("UPDATE stats_summary SET cutoff_date=$1 WHERE id='legacy'")
            .bind(cutoff + Duration::days(1)).execute(&pool).await.unwrap();
        sqlx::query("DELETE FROM usage WHERE request_id='boundary'")
            .execute(&pool).await.unwrap();
        let advanced = repo.query_dashboard_summary(&query).await.unwrap();
        assert_eq!(advanced.total.request_count, restored.total.request_count);
        assert_eq!(advanced.total.total_tokens, restored.total.total_tokens);
        assert_eq!(advanced.total.billable_amount, restored.total.billable_amount);
        assert_eq!(advanced.today, restored.today);
        assert_eq!(advanced.activity_days, restored.activity_days);
        assert_eq!(advanced.active_days, restored.active_days);

        // New daily rollups retain customer charges independently after detail
        // expires; older NULL daily charges retain their original legacy cost.
        sqlx::query("UPDATE stats_daily SET billing_cost=1.5 WHERE id='recent'")
            .execute(&pool).await.unwrap();
        sqlx::query("DELETE FROM usage WHERE request_id='overlap'")
            .execute(&pool).await.unwrap();
        let billed_history = repo.query_dashboard_summary(&query).await.unwrap();
        assert_eq!(billed_history.total.billable_amount.as_deref(), Some("123456791.59691357"));
        assert_eq!(billed_history.total.request_count, restored.total.request_count);
        assert_eq!(billed_history.today, restored.today);
        let provider_cost: String = sqlx::query_scalar("SELECT actual_total_cost::text FROM stats_daily WHERE id='recent'")
            .fetch_one(&pool).await.unwrap();
        assert_eq!(provider_cost, "0.30000003");

        // The pre-activation live prefix applies the same composite snapshot
        // to its finalized base amount, independently of procurement cost.
        sqlx::query("UPDATE usage SET request_metadata=$1 WHERE request_id='before-shared'")
            .bind(serde_json::json!({"billing_multiplier_snapshot": {"version": 1, "factors": {"routing_group": 2, "user_group": 0.75}, "multiplier": 1.5}}))
            .execute(&pool).await.unwrap();
        let billed_prefix = repo.query_dashboard_summary(&query).await.unwrap();
        assert_eq!(billed_prefix.total.billable_amount.as_deref(), Some("123456791.65864196"));
        assert_eq!(billed_prefix.today.billable_amount.as_deref(), Some("0.93518517"));

        // A summary cutoff without legacy daily history must leave the normal
        // future-only projection and its requested calendar unchanged.
        sqlx::query("DELETE FROM stats_daily").execute(&pool).await.unwrap();
        let future_only = repo.query_dashboard_summary(&query).await.unwrap();
        assert_eq!(future_only.stats_since, activation.to_rfc3339());
        assert_eq!(future_only.activity_timezone, query.timezone);
        assert_eq!(future_only.total.request_count, 1);
        assert_eq!(future_only.today.request_count, 1);
        assert_eq!(future_only.today_from, activation.to_rfc3339());
    })
    .catch_unwind()
    .await;

    pool.close().await;
    sqlx::query(&format!("DROP DATABASE {database} WITH (FORCE)"))
        .execute(&admin)
        .await
        .unwrap();
    admin.close().await;
    if let Err(error) = outcome {
        std::panic::resume_unwind(error);
    }
}

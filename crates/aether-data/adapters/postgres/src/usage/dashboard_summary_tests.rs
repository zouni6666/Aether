use super::SqlxUsageReadRepository;
use aether_data_contracts::repository::usage::*;
use chrono::{Duration, Utc};

#[tokio::test]
#[ignore = "requires local AETHER_TEST_DATABASE_URL with temporary database creation"]
async fn live_future_dashboard_is_incremental_idempotent_and_survives_retention() {
    use futures_util::FutureExt;
    use std::panic::AssertUnwindSafe;
    let options = std::env::var("AETHER_TEST_DATABASE_URL")
        .unwrap()
        .parse::<sqlx::postgres::PgConnectOptions>()
        .unwrap();
    let admin = sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .connect_with(options.clone())
        .await
        .unwrap();
    let database = format!("future_dashboard_{}", uuid::Uuid::new_v4().simple());
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
        let query = UsageDashboardAnalyticsQuery { timezone: "Asia/Kathmandu".into() };
        let empty = repo.query_dashboard_summary(&query).await.unwrap();
        assert_eq!(empty.total.request_count,0);
        assert_eq!(empty.consecutive_active_days,0);
        assert_eq!(empty.total.billable_amount.as_deref(),Some("0.00000000"));
        let since = chrono::DateTime::parse_from_rfc3339(&empty.stats_since).unwrap().with_timezone(&Utc);
        sqlx::query("INSERT INTO users(id,username,email_verified,is_active) VALUES('future-user','future-user',false,false)").execute(&pool).await.unwrap();
        let mut tx = pool.begin().await.unwrap();
        for (id,at) in [("old",since-Duration::days(500)),("new",Utc::now()),("second",Utc::now())] {
            sqlx::query("INSERT INTO usage(id,request_id,user_id,model,provider_name,status,billing_status,input_tokens,output_tokens,total_tokens,actual_total_cost_usd,total_cost_usd,created_at,request_metadata,first_byte_time_ms,response_time_ms,upstream_is_stream) VALUES($1,$1,'future-user','m','p','completed','settled',100,20,120,0.25,0.25,$2,'{\"analytics_attribution\":{\"is_standalone\":false},\"upstream_is_stream\":true}',100,800,true)")
                .bind(id).bind(at).execute(&mut *tx).await.unwrap();
        }
        tx.commit().await.unwrap();
        let first = repo.query_dashboard_summary(&query).await.unwrap();
        assert_eq!(first.total.request_count,2);
        assert_eq!(first.total.total_tokens,240);
        assert_eq!(first.today.active_users,1);
        assert_eq!(first.today.first_byte_sample_count,2);
        assert_eq!(first.today.response_sample_count,2);
        assert_eq!(first.today.stream_requests,2);
        assert_eq!(first.users.total,1,"inactive but nondeleted accounts count");
        assert_eq!(first.users.created_today,1);
        assert_eq!(first.active_days,1);
        assert_eq!(first.consecutive_active_days,1);
        assert_eq!(first.activity_days.iter().map(|d| d.requests).sum::<u64>(),2);
        sqlx::query("UPDATE usage SET output_tokens=999 WHERE request_id='old'").execute(&pool).await.unwrap();
        let mut rollback = pool.begin().await.unwrap();
        sqlx::query("UPDATE usage SET total_tokens=999 WHERE request_id='second'").execute(&mut *rollback).await.unwrap();
        rollback.rollback().await.unwrap();
        assert_eq!(repo.query_dashboard_summary(&query).await.unwrap().total,first.total);
        sqlx::query("INSERT INTO usage_settlement_snapshots(request_id,billing_status,billing_input_tokens,billing_effective_input_tokens,billing_output_tokens,billing_cache_read_tokens,billing_cache_creation_tokens,billing_total_input_context,billing_total_cost_usd,billing_actual_total_cost_usd) VALUES('new','settled',100,100,30,40,10,150,0.12345678,0.12345678)").execute(&pool).await.unwrap();
        let settled = repo.query_dashboard_summary(&query).await.unwrap();
        assert_eq!(settled.total.request_count,2);
        assert_eq!(settled.total.total_tokens,300);
        assert_eq!(settled.today.cache_read_tokens,40);
        assert_eq!(settled.today.cache_creation_tokens,10);
        assert_eq!(settled.today.cache_input_tokens,250);
        assert_eq!(settled.total.billable_amount.as_deref(),Some("0.37345678"));
        sqlx::query("UPDATE usage_settlement_snapshots SET billing_actual_total_cost_usd=0.12345678 WHERE request_id='new'").execute(&pool).await.unwrap();
        assert_eq!(repo.query_dashboard_summary(&query).await.unwrap().total,settled.total);
        // Independent usage and settlement transactions converge on the final
        // canonical fact regardless of which deferred projection acquires its shard first.
        let usage_update = async {
            let mut tx = pool.begin().await.unwrap();
            sqlx::query("UPDATE usage SET response_time_ms=900 WHERE request_id='new'")
                .execute(&mut *tx).await.unwrap();
            tx.commit().await.unwrap();
        };
        let settlement_update = async {
            let mut tx = pool.begin().await.unwrap();
            sqlx::query("UPDATE usage_settlement_snapshots SET billing_output_tokens=40 WHERE request_id='new'")
                .execute(&mut *tx).await.unwrap();
            tx.commit().await.unwrap();
        };
        tokio::time::timeout(std::time::Duration::from_secs(5), async {
            tokio::join!(usage_update,settlement_update);
        }).await.expect("concurrent writes finish without deadlock");
        let concurrent = repo.query_dashboard_summary(&query).await.unwrap();
        assert_eq!(concurrent.total.total_tokens,310);
        assert_eq!(concurrent.today.response_sum_ms,1700.0);
        // A correction followed by purge in one transaction captures the final fact.
        let mut purge = pool.begin().await.unwrap();
        sqlx::query("UPDATE usage_settlement_snapshots SET billing_output_tokens=50 WHERE request_id='new'").execute(&mut *purge).await.unwrap();
        sqlx::query("DELETE FROM usage WHERE request_id='new'").execute(&mut *purge).await.unwrap();
        purge.commit().await.unwrap();
        let retained = repo.query_dashboard_summary(&query).await.unwrap();
        assert_eq!(retained.total.request_count,2);
        assert_eq!(retained.total.total_tokens,320);
        assert_eq!(retained.total.billable_amount,settled.total.billable_amount);
        assert_eq!(sqlx::query_scalar::<_,i64>("SELECT count(*) FROM dashboard_request_contributions WHERE request_id='new'").fetch_one(&pool).await.unwrap(),0,"purged requests retain totals without retaining correction ledgers");
        sqlx::query("UPDATE users SET is_deleted=true WHERE id='future-user'").execute(&pool).await.unwrap();
        sqlx::query("DELETE FROM users WHERE id='future-user'").execute(&pool).await.unwrap();
        let deleted = repo.query_dashboard_summary(&query).await.unwrap();
        assert_eq!(deleted.users.total,0);
        assert_eq!(deleted.users.deleted_today,1,"soft deletion then purge counts one removal");
        assert_eq!(deleted.total.request_count,2);
        assert_eq!(sqlx::query_scalar::<_,i64>("SELECT count(*) FROM dashboard_stats_pending").fetch_one(&pool).await.unwrap(),0);
        // Kathmandu midnight splits a UTC hour. Only that hour's minute
        // counters must supply the two separate local activity dates.
        let midnight = query.today_start(Utc::now()).unwrap()-Duration::days(1);
        sqlx::query("UPDATE dashboard_stats_state SET stats_since=$1")
            .bind(midnight-Duration::minutes(2)).execute(&pool).await.unwrap();
        for (id, at) in [("before-midnight",midnight-Duration::minutes(1)),("after-midnight",midnight+Duration::minutes(1))] {
            sqlx::query("INSERT INTO usage(id,request_id,model,provider_name,status,billing_status,created_at) VALUES($1,$1,'m','p','completed','pending',$2)")
                .bind(id).bind(at).execute(&pool).await.unwrap();
        }
        let boundary = repo.query_dashboard_summary(&query).await.unwrap();
        let tz: chrono_tz::Tz = query.timezone.parse().unwrap();
        for at in [midnight-Duration::minutes(1),midnight+Duration::minutes(1)] {
            let day = at.with_timezone(&tz).date_naive().to_string();
            assert_eq!(boundary.activity_days.iter().find(|entry| entry.date==day).unwrap().requests,1);
        }
        assert_eq!(boundary.active_days,3);
        assert_eq!(boundary.consecutive_active_days,3);
        assert_eq!(boundary.total.request_count,4);

        // Upgrade compatibility: old installations contain wide minutes but
        // no narrow activity minutes. Keep local-calendar history accurate
        // during partial compaction, including midnight splitting a UTC hour.
        let historic_midnight = query.today_start(Utc::now()).unwrap()-Duration::days(40);
        sqlx::query("UPDATE dashboard_stats_state SET stats_since=$1")
            .bind(historic_midnight-Duration::days(1)).execute(&pool).await.unwrap();
        for (id,at) in [("historic-before",historic_midnight-Duration::minutes(1)),("historic-after",historic_midnight+Duration::minutes(1))] {
            sqlx::query("INSERT INTO usage(id,request_id,user_id,model,provider_name,status,billing_status,created_at,response_time_ms) VALUES($1,$1,'retained-user','m','p','completed','pending',$2,10)")
                .bind(id).bind(at).execute(&pool).await.unwrap();
        }
        sqlx::query("INSERT INTO dashboard_stats_minute(bucket_start,shard,metrics) SELECT date_trunc('minute',created_at),(hashtextextended(request_id,0)&15)::smallint,metrics FROM dashboard_request_contributions WHERE request_id LIKE 'historic-%'").execute(&pool).await.unwrap();
        sqlx::query("DELETE FROM dashboard_activity_minute WHERE bucket_start < now()-INTERVAL '35 days'").execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO dashboard_actor_minute(bucket_start,shard,actor_user_id,request_count) VALUES($1,0,'old-actor',1)").bind(historic_midnight).execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO dashboard_user_events_minute(bucket_start,shard,created_count) VALUES($1,0,1)").bind(historic_midnight).execute(&pool).await.unwrap();
        let legacy = repo.query_dashboard_summary(&query).await.unwrap();
        assert_eq!(legacy.active_days,boundary.active_days+2);
        assert_eq!(legacy.total.request_count,boundary.total.request_count+2);
        // A delayed correction seeds the old narrow minute before applying its
        // delta; a later compaction must not overwrite or double its count.
        sqlx::query("UPDATE usage SET response_time_ms=20 WHERE request_id='historic-before'").execute(&pool).await.unwrap();
        let corrected = repo.query_dashboard_summary(&query).await.unwrap();
        assert_eq!(corrected.activity_days,legacy.activity_days);
        repo.maintain_dashboard_projection(Utc::now(),1).await.unwrap();
        assert_eq!(repo.query_dashboard_summary(&query).await.unwrap().activity_days,legacy.activity_days,"partially compacted history remains complete");
        let old_correction = async {
            sqlx::query("UPDATE usage SET response_time_ms=40 WHERE request_id='historic-after'").execute(&pool).await.unwrap();
        };
        let compaction = async {
            repo.maintain_dashboard_projection(Utc::now(),1).await.unwrap();
        };
        tokio::time::timeout(std::time::Duration::from_secs(5),async {
            tokio::join!(old_correction,compaction);
        }).await.expect("old corrections and retention use compatible lock orders");
        // A busy shard is deliberately deferred to the next maintenance pass.
        repo.maintain_dashboard_projection(Utc::now(),1).await.unwrap();
        assert_eq!(sqlx::query_scalar::<_,i64>("SELECT count(*) FROM dashboard_stats_minute WHERE bucket_start < now()-INTERVAL '35 days'").fetch_one(&pool).await.unwrap(),0);
        assert_eq!(sqlx::query_scalar::<_,i64>("SELECT count(*) FROM dashboard_actor_minute WHERE bucket_start < now()-INTERVAL '35 days'").fetch_one(&pool).await.unwrap(),0);
        assert_eq!(sqlx::query_scalar::<_,i64>("SELECT count(*) FROM dashboard_user_events_minute WHERE bucket_start < now()-INTERVAL '35 days'").fetch_one(&pool).await.unwrap(),0);
        let compacted = repo.query_dashboard_summary(&query).await.unwrap();
        let mut expected_total = corrected.total.clone();
        expected_total.response_sum_ms += 30.0;
        assert_eq!(compacted.total,expected_total);
        assert_eq!(compacted.activity_days,legacy.activity_days);
        assert_eq!(compacted.consecutive_active_days,legacy.consecutive_active_days);
        sqlx::query("DELETE FROM usage WHERE request_id LIKE 'historic-%'").execute(&pool).await.unwrap();
        let purged_history = repo.query_dashboard_summary(&query).await.unwrap();
        assert_eq!(purged_history.total,compacted.total);
        assert_eq!(purged_history.activity_days,compacted.activity_days);
        assert_eq!(sqlx::query_scalar::<_,i64>("SELECT count(*) FROM dashboard_request_contributions WHERE request_id LIKE 'historic-%'").fetch_one(&pool).await.unwrap(),0);

        // Orphan ledgers left by the old trigger are removed in bounded PK
        // pages. Live rows before the orphans must not starve later pages.
        for id in ["zz-orphan-1","zz-orphan-2","zz-orphan-3"] {
            sqlx::query("INSERT INTO dashboard_request_contributions(request_id,created_at,metrics) VALUES($1,now(),'{}')").bind(id).execute(&pool).await.unwrap();
        }
        sqlx::query("UPDATE dashboard_stats_state SET contributions_cleanup_cursor=NULL").execute(&pool).await.unwrap();
        repo.maintain_dashboard_projection(Utc::now(),1).await.unwrap();
        assert_eq!(sqlx::query_scalar::<_,i64>("SELECT count(*) FROM dashboard_request_contributions WHERE request_id LIKE 'zz-orphan-%'").fetch_one(&pool).await.unwrap(),3);
        for _ in 0..12 { repo.maintain_dashboard_projection(Utc::now(),1).await.unwrap(); }
        assert_eq!(sqlx::query_scalar::<_,i64>("SELECT count(*) FROM dashboard_request_contributions WHERE request_id LIKE 'zz-orphan-%'").fetch_one(&pool).await.unwrap(),0);
        assert_eq!(repo.query_dashboard_summary(&query).await.unwrap().total,purged_history.total);

        // A correction may decide a bucket is still inside retention while a
        // concurrent cleaner crosses the next minute cutoff. Hold its narrow
        // row to suspend that correction after it locks the total shard, then
        // prove maintenance skips the shard instead of locking wide and waiting
        // on narrow (which would invert the correction's lock order).
        let edge_now=Utc::now();
        let edge_at=edge_now-Duration::days(35)+Duration::minutes(1);
        sqlx::query("INSERT INTO usage(id,request_id,model,provider_name,status,billing_status,created_at,response_time_ms) VALUES('retention-edge','retention-edge','m','p','completed','pending',$1,10)")
            .bind(edge_at).execute(&pool).await.unwrap();
        let edge_shard:i16=sqlx::query_scalar("SELECT (hashtextextended('retention-edge',0)&15)::smallint").fetch_one(&pool).await.unwrap();
        let mut blocker=pool.begin().await.unwrap();
        sqlx::query("SELECT 1 FROM dashboard_activity_minute WHERE bucket_start=date_trunc('minute',$1::timestamptz) AND shard=$2 FOR UPDATE")
            .bind(edge_at).bind(edge_shard).execute(&mut *blocker).await.unwrap();
        let correction_pool=pool.clone();
        let edge_correction=tokio::spawn(async move {
            sqlx::query("UPDATE usage SET response_time_ms=30 WHERE request_id='retention-edge'")
                .execute(&correction_pool).await.unwrap();
        });
        tokio::time::timeout(std::time::Duration::from_secs(2),async {
            loop {
                let available=sqlx::query_scalar::<_,i16>("SELECT shard FROM dashboard_stats_total WHERE shard=$1 FOR UPDATE SKIP LOCKED")
                    .bind(edge_shard).fetch_optional(&pool).await.unwrap();
                if available.is_none() { break; }
                tokio::time::sleep(std::time::Duration::from_millis(5)).await;
            }
        }).await.expect("correction reached its shard lock");
        tokio::time::timeout(std::time::Duration::from_secs(1),repo.maintain_dashboard_projection(edge_now+Duration::minutes(2),1000))
            .await.expect("retention skips a correction's busy shard at the cutoff boundary").unwrap();
        blocker.rollback().await.unwrap();
        edge_correction.await.unwrap();
        let edge_corrected=repo.query_dashboard_summary(&query).await.unwrap();
        repo.maintain_dashboard_projection(edge_now+Duration::minutes(2),1000).await.unwrap();
        assert_eq!(repo.query_dashboard_summary(&query).await.unwrap().total,edge_corrected.total);
        assert_eq!(sqlx::query_scalar::<_,i64>("SELECT count(*) FROM dashboard_stats_minute WHERE bucket_start=date_trunc('minute',$1::timestamptz) AND shard=$2")
            .bind(edge_at).bind(edge_shard).fetch_one(&pool).await.unwrap(),0);
    }).catch_unwind().await;
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

use super::*;
use aether_data_contracts::repository::usage::{UsageAnalyticsQuery, UsageAnalyticsView};
use chrono::{TimeZone, Utc};

#[tokio::test]
async fn overview_dirty_events_keep_independent_usage_writes_concurrent_and_reads_fresh() {
    let Some(server) = ManagedPostgresServer::try_start()
        .await
        .expect("overview PostgreSQL should start or skip")
    else {
        return;
    };
    let pool = PgPool::connect(server.database_url()).await.unwrap();
    prepare_and_apply_clean_postgres_database(&pool).await;
    let at = Utc.with_ymd_and_hms(2026, 1, 2, 3, 0, 0).unwrap();
    let repo = aether_data_postgres::SqlxUsageReadRepository::new(pool.clone());
    let insert = "INSERT INTO usage(id,request_id,model,provider_name,status,billing_status,created_at,response_time_ms) VALUES($1,$1,'dirty-event-test','test','completed','settled',$2,100)";
    let second_id: String = query_scalar(
        "SELECT 'second-' || n FROM generate_series(1,100) n WHERE (hashtextextended('second-' || n,0) & 15) <> (hashtextextended('first',0) & 15) LIMIT 1",
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    // A stays open while B writes the same hour/day. Before the migration,
    // B times out on the shared stats_bucket_state day row despite a distinct request.
    let mut first = pool.begin().await.unwrap();
    sqlx::query(insert)
        .bind("first")
        .bind(at)
        .execute(&mut *first)
        .await
        .unwrap();
    let mut second = pool.begin().await.unwrap();
    sqlx::query("SET LOCAL lock_timeout='500ms'")
        .execute(&mut *second)
        .await
        .unwrap();
    sqlx::query(insert)
        .bind(&second_id)
        .bind(at)
        .execute(&mut *second)
        .await
        .unwrap();
    second.commit().await.unwrap();
    assert_eq!(
        repo.rebuild_overview_buckets(at + chrono::Duration::days(1), 8)
            .await
            .unwrap(),
        2
    );
    let query = UsageAnalyticsQuery {
        from_unix_ms: at.timestamp_millis() as u64,
        to_unix_ms: (at + chrono::Duration::hours(1)).timestamp_millis() as u64,
        timezone: "UTC".into(),
        view: UsageAnalyticsView::Summary,
        limit: 100,
        ..Default::default()
    };
    assert_eq!(
        repo.query_usage_analytics(&query)
            .await
            .unwrap()
            .summary
            .request_count,
        1
    );

    // Commit order differs from transaction-ID order. Consuming B must never
    // acknowledge A, and A must invalidate the already-published clean projection.
    first.commit().await.unwrap();
    let pending = repo.query_usage_analytics(&query).await.unwrap();
    assert_eq!(pending.summary.request_count, 2);
    assert_eq!(pending.coverage.dirty_bucket_count, 1);
    let queued: i64 = query_scalar("SELECT count(*) FROM stats_overview_dirty_events")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(queued, 2);
    assert_eq!(
        repo.rebuild_overview_buckets(at + chrono::Duration::days(1), 8)
            .await
            .unwrap(),
        2
    );
    let rebuilt = repo.query_usage_analytics(&query).await.unwrap();
    assert_eq!(rebuilt.summary, pending.summary);
    assert_eq!(rebuilt.coverage.dirty_bucket_count, 0);
    assert_eq!(
        query_scalar::<_, i64>("SELECT count(*) FROM stats_overview_dirty_events")
            .fetch_one(&pool)
            .await
            .unwrap(),
        0
    );

    let mut rolled_back = pool.begin().await.unwrap();
    sqlx::query(insert)
        .bind("rollback")
        .bind(at)
        .execute(&mut *rolled_back)
        .await
        .unwrap();
    rolled_back.rollback().await.unwrap();
    assert_eq!(
        query_scalar::<_, i64>("SELECT count(*) FROM stats_overview_dirty_events")
            .fetch_one(&pool)
            .await
            .unwrap(),
        0
    );

    // Repeated mutations in one transaction deduplicate, but deleting facts
    // retains the irreversible-loss marker until it is merged into state.
    let mut deleted = pool.begin().await.unwrap();
    sqlx::query("UPDATE usage SET response_time_ms=200 WHERE request_id='first'")
        .execute(&mut *deleted)
        .await
        .unwrap();
    sqlx::query("DELETE FROM usage WHERE request_id='first'")
        .execute(&mut *deleted)
        .await
        .unwrap();
    deleted.commit().await.unwrap();
    assert_eq!(
        query_scalar::<_, i64>(
            "SELECT count(*) FROM stats_overview_dirty_events WHERE unrecoverable"
        )
        .fetch_one(&pool)
        .await
        .unwrap(),
        2
    );
    let lost = repo.query_usage_analytics(&query).await.unwrap();
    assert_eq!(lost.summary.request_count, 1);
    assert_eq!(lost.unrecoverable_bucket_count, 1);
    assert_eq!(
        repo.rebuild_overview_buckets(at + chrono::Duration::days(1), 8)
            .await
            .unwrap(),
        1
    );
    let merged = repo.query_usage_analytics(&query).await.unwrap();
    assert_eq!(merged.unrecoverable_bucket_count, 1);
    assert_eq!(merged.summary, lost.summary);
    assert_eq!(
        query_scalar::<_, i64>("SELECT count(*) FROM stats_overview_dirty_events")
            .fetch_one(&pool)
            .await
            .unwrap(),
        0
    );
    pool.close().await;
}

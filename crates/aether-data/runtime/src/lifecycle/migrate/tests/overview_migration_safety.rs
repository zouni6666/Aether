use super::*;
use aether_data_contracts::repository::usage::{UsageAnalyticsQuery, UsageAnalyticsView};
use chrono::{TimeZone, Utc};

const OVERVIEW_START: i64 = 20260911000000;
const BILLING_INDEX: i64 = 20260918000000;
const INSERT_USAGE: &str = "INSERT INTO usage(id,request_id,model,provider_name,status,billing_status,created_at) VALUES($1,$1,'overview-migration-test','test','completed','settled',$2)";

async fn apply_through_overview(connection: &mut PgConnection) {
    connection.ensure_migrations_table().await.unwrap();
    for migration in POSTGRES_MIGRATOR
        .iter()
        .filter(|migration| migration.version <= OVERVIEW_START)
    {
        connection.apply(migration).await.unwrap();
    }
}

async fn assert_independent_same_bucket_writes(pool: &PgPool, prefix: &str) {
    let at = Utc.with_ymd_and_hms(2026, 1, 2, 3, 0, 0).unwrap();
    let mut first = pool.begin().await.unwrap();
    query(INSERT_USAGE)
        .bind(format!("{prefix}-first"))
        .bind(at)
        .execute(&mut *first)
        .await
        .unwrap();
    let mut second = pool.begin().await.unwrap();
    query("SET LOCAL lock_timeout='500ms'")
        .execute(&mut *second)
        .await
        .unwrap();
    query(INSERT_USAGE)
        .bind(format!("{prefix}-second"))
        .bind(at)
        .execute(&mut *second)
        .await
        .expect("another request in the same hour/day must not wait for the first transaction");
    second.commit().await.unwrap();
    first.commit().await.unwrap();
}

async fn is_stamped(pool: &PgPool, version: i64) -> bool {
    query_scalar("SELECT EXISTS(SELECT 1 FROM _sqlx_migrations WHERE version=$1 AND success)")
        .bind(version)
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn queue_and_state(pool: &PgPool) -> (String, String) {
    sqlx::query_as(
        r#"
SELECT
  (SELECT COALESCE(jsonb_agg(to_jsonb(e) ORDER BY transaction_id,projection_version,granularity,bucket_start),'[]')::text FROM stats_overview_dirty_events e),
  (SELECT COALESCE(jsonb_agg(to_jsonb(s) ORDER BY projection_version,granularity,bucket_start),'[]')::text FROM stats_bucket_state s)
"#,
    )
    .fetch_one(pool)
    .await
    .unwrap()
}

#[tokio::test]
async fn overview_queue_is_safe_from_its_first_installation() {
    let Some(server) = ManagedPostgresServer::try_start().await.unwrap() else {
        return;
    };
    let mut connection = PgConnection::connect(server.database_url()).await.unwrap();
    apply_through_overview(&mut connection).await;
    let pool = PgPool::connect(server.database_url()).await.unwrap();

    // The initial installation must be safe before any subsequent migration,
    // including the potentially long concurrent index build, has completed.
    assert_independent_same_bucket_writes(&pool, "first-install").await;
    let (queued, obsolete, direct): (i64, i64, i64) = sqlx::query_as(
        "SELECT (SELECT count(*) FROM stats_overview_dirty_events), (SELECT count(*) FROM stats_overview_dirty_events WHERE projection_version <> 'overview-v2'), (SELECT count(*) FROM stats_bucket_state)",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!((queued, obsolete, direct), (4, 0, 0));
    assert!(!is_stamped(&pool, BILLING_INDEX).await);
    pool.close().await;
}

#[tokio::test]
async fn overview_queue_survives_later_index_failure_and_retry_without_losing_data() {
    let Some(server) = ManagedPostgresServer::try_start().await.unwrap() else {
        return;
    };
    let mut connection = PgConnection::connect(server.database_url()).await.unwrap();
    apply_through_overview(&mut connection).await;
    let pool = PgPool::connect(server.database_url()).await.unwrap();
    let at = Utc.with_ymd_and_hms(2026, 1, 2, 3, 0, 0).unwrap();
    query(INSERT_USAGE)
        .bind("before-index-failure")
        .bind(at)
        .execute(&pool)
        .await
        .unwrap();
    let repo = aether_data_postgres::SqlxUsageReadRepository::new(pool.clone());
    assert_eq!(
        repo.rebuild_overview_buckets(at + chrono::Duration::days(1), 8)
            .await
            .unwrap(),
        2
    );
    let original_state = queue_and_state(&pool).await.1;

    let mut blocker = pool.begin().await.unwrap();
    query("LOCK TABLE usage_settlement_snapshots IN SHARE MODE")
        .execute(&mut *blocker)
        .await
        .unwrap();
    let error = super::super::run_migrations(&pool)
        .await
        .expect_err("the concurrent index must encounter the held relation lock");
    assert!(error.to_string().contains("lock timeout"), "{error}");
    assert!(is_stamped(&pool, OVERVIEW_START).await);
    assert!(!is_stamped(&pool, BILLING_INDEX).await);

    // A failed later migration must leave the safe trigger committed and able
    // to accept independent business writes until the upgrade can be retried.
    assert_independent_same_bucket_writes(&pool, "failed-upgrade").await;
    let after_failure = queue_and_state(&pool).await;
    assert_eq!(after_failure.1, original_state);
    assert_eq!(
        query_scalar::<_, i64>("SELECT count(*) FROM stats_overview_dirty_events")
            .fetch_one(&pool)
            .await
            .unwrap(),
        4
    );
    blocker.rollback().await.unwrap();
    super::super::run_migrations(&pool).await.unwrap();
    assert_eq!(queue_and_state(&pool).await, after_failure);
    assert!(super::super::pending_migrations(&pool)
        .await
        .unwrap()
        .is_empty());

    assert_eq!(
        repo.rebuild_overview_buckets(at + chrono::Duration::days(1), 8)
            .await
            .unwrap(),
        2
    );
    let result = repo
        .query_usage_analytics(&UsageAnalyticsQuery {
            from_unix_ms: at.timestamp_millis() as u64,
            to_unix_ms: (at + chrono::Duration::hours(1)).timestamp_millis() as u64,
            timezone: "UTC".into(),
            view: UsageAnalyticsView::Summary,
            limit: 100,
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(result.summary.request_count, 3);
    assert_eq!(result.coverage.dirty_bucket_count, 0);
    pool.close().await;
}

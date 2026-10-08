use super::*;

const BILLING_VERSION: i64 = 20261007000000;

#[tokio::test]
async fn customer_billing_upgrade_preserves_history_and_aggregates_new_days() {
    let Some(server) = ManagedPostgresServer::try_start().await.unwrap() else {
        return;
    };
    let mut connection = PgConnection::connect(server.database_url()).await.unwrap();
    connection.ensure_migrations_table().await.unwrap();
    for migration in POSTGRES_MIGRATOR
        .iter()
        .filter(|migration| migration.version < BILLING_VERSION)
    {
        connection.apply(migration).await.unwrap();
    }
    let pool = PgPool::connect(server.database_url()).await.unwrap();
    sqlx::raw_sql(
        r#"
INSERT INTO stats_daily(id,date,total_requests,actual_total_cost,is_complete)
VALUES ('history','2026-07-17 00:00:00+00',1,0.5,true);
INSERT INTO usage(id,request_id,model,provider_name,status,billing_status,
  total_cost_usd,actual_total_cost_usd,created_at,request_metadata)
VALUES ('history','history','m','p','completed','settled',2,0.5,
  '2026-07-17 12:00:00+00','{"routing_group_billing_multiplier":2}');
"#,
    )
    .execute(&pool)
    .await
    .unwrap();
    let history_before: serde_json::Value =
        query_scalar("SELECT to_jsonb(d) FROM stats_daily d WHERE id='history'")
            .fetch_one(&pool)
            .await
            .unwrap();
    let usage_before: serde_json::Value =
        query_scalar("SELECT to_jsonb(u) FROM usage u WHERE request_id='history'")
            .fetch_one(&pool)
            .await
            .unwrap();
    let migration = POSTGRES_MIGRATOR
        .iter()
        .find(|migration| migration.version == BILLING_VERSION)
        .unwrap();
    connection.apply(migration).await.unwrap();

    // Even retained requests with captured factors must not rewrite old daily totals.
    let history_after: serde_json::Value =
        query_scalar("SELECT to_jsonb(d) - 'billing_cost' FROM stats_daily d WHERE id='history'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(history_after, history_before);
    let usage_after: serde_json::Value =
        query_scalar("SELECT to_jsonb(u) FROM usage u WHERE request_id='history'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(usage_after, usage_before);
    let legacy_cost: (Option<String>, String) = sqlx::query_as(
        "SELECT billing_cost::text, COALESCE(billing_cost,actual_total_cost::numeric)::text FROM stats_daily WHERE id='history'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(legacy_cost, (None, "0.50000000".to_string()));

    sqlx::raw_sql(
        r#"
INSERT INTO usage(id,request_id,model,provider_name,status,billing_status,
  total_cost_usd,actual_total_cost_usd,created_at,request_metadata)
VALUES ('new-billed','new-billed','m','p','completed','settled',4,1,
  '2026-07-18 12:00:00+00',
  '{"billing_multiplier_snapshot":{"version":1,"factors":{"routing_group":2,"user_group":0.75},"multiplier":1.5}}'),
  ('new-legacy','new-legacy','m','p','completed','settled',2,0.5,
  '2026-07-18 13:00:00+00','{}');
"#,
    )
    .execute(&pool)
    .await
    .unwrap();
    let new_day = historical_stats_day() + chrono::Duration::days(1);
    let backend = postgres_backend(server.database_url());
    let summary = backend
        .aggregate_stats_daily(&crate::StatsDailyAggregationInput {
            target_day_utc: new_day,
            aggregated_at: new_day + chrono::Duration::days(1),
        })
        .await
        .unwrap()
        .unwrap();
    assert_eq!(summary.day_start_utc, new_day);
    assert_eq!(summary.total_requests, 2);
    let new_costs: (String, String) = sqlx::query_as(
        "SELECT billing_cost::text, actual_total_cost::text FROM stats_daily WHERE date=$1",
    )
    .bind(new_day)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        new_costs,
        ("6.50000000".to_string(), "1.50000000".to_string())
    );
    assert!(query_scalar::<_, bool>(
        "SELECT billing_cost IS NULL FROM stats_daily WHERE id='history'",
    )
    .fetch_one(&pool)
    .await
    .unwrap());
    backend.pool().close().await;
    pool.close().await;
}

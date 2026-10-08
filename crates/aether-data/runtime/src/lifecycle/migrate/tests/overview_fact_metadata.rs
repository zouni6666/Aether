use super::*;
use serde_json::Value;

const OPTIMIZATION_VERSION: i64 = 20261004000000;

async fn read_facts(pool: &PgPool) -> Value {
    query_scalar(
        "SELECT jsonb_agg(to_jsonb(f) ORDER BY request_id) FROM usage_analytics_facts_v1 f",
    )
    .fetch_one(pool)
    .await
    .unwrap()
}

async fn stored_rows(pool: &PgPool) -> Vec<Value> {
    let mut rows = Vec::new();
    for table in [
        "usage",
        "usage_settlement_snapshots",
        "usage_attribution_snapshots",
        "stats_overview_dirty_events",
        "stats_bucket_state",
        "stats_overview_hourly",
        "stats_overview_daily",
        "dashboard_request_contributions",
        "dashboard_stats_total",
        "dashboard_activity_minute",
    ] {
        rows.push(
            query_scalar(&format!(
                "SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY to_jsonb(t)), '[]') FROM {table} t"
            ))
            .fetch_one(pool)
            .await
            .unwrap(),
        );
    }
    rows
}

#[tokio::test]
async fn overview_fact_metadata_optimization_preserves_facts_without_backfill() {
    let Some(server) = ManagedPostgresServer::try_start().await.unwrap() else {
        return;
    };
    let mut connection = PgConnection::connect(server.database_url()).await.unwrap();
    connection.ensure_migrations_table().await.unwrap();
    for migration in POSTGRES_MIGRATOR
        .iter()
        .filter(|migration| migration.version < OPTIMIZATION_VERSION)
    {
        connection.apply(migration).await.unwrap();
    }
    let pool = PgPool::connect(server.database_url()).await.unwrap();
    sqlx::raw_sql(
        r#"
INSERT INTO users(id,username,email_verified) VALUES('owner','owner',false);
INSERT INTO api_keys(id,user_id,key_hash,is_standalone)
VALUES('employee-key','owner',repeat('e',64),false),
      ('standalone-key','owner',repeat('f',64),true);
"#,
    )
    .execute(&pool)
    .await
    .unwrap();

    // Preserve distinctions between absent/null flags, booleans and strings,
    // malformed nested shapes, key overrides and pre-attribution legacy rows.
    let metadata_cases = [
        None,
        Some("null"),
        Some("[]"),
        Some("false"),
        Some("\"metadata\""),
        Some("{}"),
        Some(r#"{"usage_available":false,"usage_pricing_available":false}"#),
        Some(r#"{"usage_available":null,"usage_pricing_available":null}"#),
        Some(r#"{"usage_available":"false","usage_pricing_available":"false"}"#),
        Some(
            r#"{"analytics_attribution":{"is_standalone":true},"analytics_measurement":{"source":"reported"}}"#,
        ),
        Some(
            r#"{"analytics_attribution":{"is_standalone":false},"analytics_measurement":{"source":"estimated"}}"#,
        ),
        Some(
            r#"{"analytics_attribution":{"is_standalone":"true"},"api_key_is_standalone":false,"analytics_measurement":{"source":"mixed"}}"#,
        ),
        Some(
            r#"{"analytics_attribution":[],"api_key_is_standalone":true,"analytics_measurement":{"source":false}}"#,
        ),
        Some(
            r#"{"analytics_attribution":null,"api_key_is_standalone":"true","analytics_measurement":[]}"#,
        ),
        Some(
            r#"{"usage_available":true,"usage_available":false,"analytics_attribution":{"is_standalone":false},"analytics_attribution":{"is_standalone":true}}"#,
        ),
    ];
    for (index, metadata) in metadata_cases.into_iter().enumerate() {
        for key in [None, Some("employee-key"), Some("standalone-key")] {
            let request = format!("metadata-{index}-{}", key.unwrap_or("legacy"));
            query(
                r#"INSERT INTO usage(id,request_id,user_id,api_key_id,model,provider_name,
                  status,billing_status,input_tokens,output_tokens,total_tokens,
                  cache_read_input_tokens,cache_creation_input_tokens,total_cost_usd,
                  actual_total_cost_usd,response_time_ms,first_byte_time_ms,is_stream,
                  created_at,request_metadata)
                VALUES($1,$1,'owner',$2,'test','test','completed','settled',100,10,110,
                  20,5,0.12345678,0.11111111,1000,100,true,'2026-01-01 00:00:00+00',$3::json)"#,
            )
            .bind(&request)
            .bind(key)
            .bind(metadata)
            .execute(&pool)
            .await
            .unwrap();
            if index % 2 == 0 {
                query(
                    r#"INSERT INTO usage_settlement_snapshots(request_id,billing_status,
                      billing_input_tokens,billing_effective_input_tokens,billing_output_tokens,
                      billing_cache_read_tokens,billing_cache_creation_tokens,
                      billing_total_cost_usd,billing_actual_total_cost_usd,input_price_per_1m,
                      billing_cache_read_cost_usd,billing_cache_creation_cost_usd,
                      quota_covered_amount_usd,wallet_consumed_amount_usd,wallet_debit_amount_usd,
                      wallet_recharge_debit_usd,wallet_gift_debit_usd,wallet_overdraft_usd,
                      allocation_status)
                    VALUES($1,'settled',200,175,20,15,10,0.3,0.25,2,0.00001,0.00002,
                      0.05,0.2,0.2,0.1,0.1,0,'complete')"#,
                )
                .bind(&request)
                .execute(&pool)
                .await
                .unwrap();
            }
            if key.is_none() {
                query("DELETE FROM usage_attribution_snapshots WHERE request_id=$1")
                    .bind(&request)
                    .execute(&pool)
                    .await
                    .unwrap();
            }
        }
    }
    // Keep a large nested payload so parity also covers toasted JSON metadata.
    query("UPDATE usage SET request_metadata=json_build_object('usage_available',true,'payload',repeat('metadata payload ',4096)) WHERE request_id='metadata-5-legacy'")
        .execute(&pool).await.unwrap();
    let before_facts = read_facts(&pool).await;
    let before_rows = stored_rows(&pool).await;

    // The definition-only upgrade must not touch historical projections/queues.
    let mut blocked_history = pool.begin().await.unwrap();
    query("LOCK TABLE stats_bucket_state,stats_overview_dirty_events,stats_overview_hourly,stats_overview_daily IN ACCESS EXCLUSIVE MODE")
        .execute(&mut *blocked_history).await.unwrap();
    query("SET lock_timeout='500ms'")
        .execute(&mut connection)
        .await
        .unwrap();
    let migration = POSTGRES_MIGRATOR
        .iter()
        .find(|migration| migration.version == OPTIMIZATION_VERSION)
        .unwrap();
    connection.apply(migration).await.unwrap();
    blocked_history.rollback().await.unwrap();

    assert_eq!(read_facts(&pool).await, before_facts);
    assert_eq!(stored_rows(&pool).await, before_rows);

    // The maintained bootstrap fragment must install the same read model.
    let bootstrap =
        include_str!("../../../../schema/bootstrap/postgres/190_overview_analytics.sql");
    let view_start = bootstrap
        .find("CREATE OR REPLACE FUNCTION public.usage_customer_billable_amount(")
        .unwrap();
    sqlx::raw_sql(&bootstrap[view_start..])
        .execute(&pool)
        .await
        .unwrap();
    assert_eq!(read_facts(&pool).await, before_facts);
    assert_eq!(stored_rows(&pool).await, before_rows);
    pool.close().await;
}

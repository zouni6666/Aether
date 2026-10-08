use super::*;
use crate::lifecycle::postgres_test_support::ManagedPostgresServer;
use sqlx::PgPool;

async fn migrate(pool: &PgPool) {
    crate::lifecycle::migrate::prepare_database_for_startup(pool)
        .await
        .unwrap();
    crate::lifecycle::migrate::run_migrations(pool)
        .await
        .unwrap();
}

async fn request(pool: &PgPool, id: &str, input: i32) {
    sqlx::query("INSERT INTO usage(id,request_id,user_id,api_key_id,provider_name,model,status,billing_status,created_at,input_tokens,output_tokens,total_tokens) VALUES ($1,$1,'backup-user','backup-key','test','test','completed','settled',clock_timestamp(),$2,3,$2+3)")
        .bind(id).bind(input).execute(pool).await.unwrap();
}

async fn dashboard_rows(pool: &PgPool) -> BTreeMap<String, Vec<Value>> {
    let mut result = BTreeMap::new();
    for table in AUXILIARY_TABLES
        .iter()
        .filter(|table| table.name.starts_with("dashboard_"))
    {
        let mut rows =
            sqlx::query_scalar::<_, Value>(&format!("SELECT to_jsonb(t) FROM {} t", table.name))
                .fetch_all(pool)
                .await
                .unwrap();
        rows.sort_by_key(Value::to_string);
        result.insert(table.name.to_owned(), rows);
    }
    result
}

async fn total(pool: &PgPool) -> i64 {
    sqlx::query_scalar("SELECT COALESCE(sum((metrics->>'request_count')::bigint),0)::bigint FROM dashboard_stats_total").fetch_one(pool).await.unwrap()
}

async fn tokens(pool: &PgPool) -> i64 {
    sqlx::query_scalar("SELECT COALESCE(sum((metrics->>'total_tokens')::bigint),0)::bigint FROM dashboard_stats_total")
        .fetch_one(pool).await.unwrap()
}

#[tokio::test]
async fn postgres_dashboard_snapshot_roundtrip_preserves_purged_totals_and_future_updates() {
    let Some(server) = ManagedPostgresServer::try_start().await.unwrap() else {
        return;
    };
    let source = sqlx::postgres::PgPoolOptions::new()
        .max_connections(2)
        .connect(server.database_url())
        .await
        .unwrap();
    migrate(&source).await;
    sqlx::raw_sql("INSERT INTO users(id,username,email_verified) VALUES('backup-user','backup-user',false); INSERT INTO api_keys(id,user_id,key_hash) VALUES('backup-key','backup-user',repeat('e',64));").execute(&source).await.unwrap();
    request(&source, "backup-kept", 11).await;
    request(&source, "backup-purged", 23).await;
    sqlx::query("DELETE FROM usage WHERE request_id='backup-purged'")
        .execute(&source)
        .await
        .unwrap();
    // Simulate the retention worker dropping a contribution whose raw request is gone.
    sqlx::query("DELETE FROM dashboard_request_contributions WHERE request_id='backup-purged'")
        .execute(&source)
        .await
        .unwrap();
    assert_eq!(total(&source).await, 2);
    assert_eq!(tokens(&source).await, 40);
    let expected = dashboard_rows(&source).await;
    let export = export_postgres_core_jsonl(&source, 1_800_000_000)
        .await
        .unwrap();
    let plan = build_import_plan(&export).unwrap();
    assert!(plan.manifest.dashboard_snapshot.is_some());
    assert!(!plan
        .rows(ExportDomain::Auxiliary)
        .iter()
        .any(|row| row.payload["__table"] == "dashboard_stats_pending"));
    sqlx::query("CREATE DATABASE dashboard_restore_test")
        .execute(&source)
        .await
        .unwrap();
    let target_url = server
        .database_url()
        .strip_suffix("/postgres")
        .unwrap()
        .to_owned()
        + "/dashboard_restore_test";
    let target = sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .connect(&target_url)
        .await
        .unwrap();
    migrate(&target).await;
    // The gateway initializes one verified local admin and a zero-value unlimited
    // wallet before the operator can run restore. Preserve that account while
    // replacing its one initialization event with the source statistics.
    sqlx::raw_sql("INSERT INTO users(id,username,email_verified,role,auth_source,is_active) VALUES('bootstrap-admin','custom-root-name',true,'admin','local',true); INSERT INTO wallets(id,user_id,limit_mode,currency,status,created_at,updated_at) VALUES('bootstrap-wallet','bootstrap-admin','unlimited','USD','active',clock_timestamp(),clock_timestamp());")
        .execute(&target).await.unwrap();
    sqlx::query("UPDATE wallets SET balance=1,total_recharged=1 WHERE id='bootstrap-wallet'")
        .execute(&target)
        .await
        .unwrap();
    let error = import_postgres_jsonl(&target, &export).await.unwrap_err();
    assert!(
        error
            .to_string()
            .contains("conflicts with existing statistics"),
        "{error}"
    );
    sqlx::query("UPDATE wallets SET balance=0,total_recharged=0 WHERE id='bootstrap-wallet'")
        .execute(&target)
        .await
        .unwrap();
    import_postgres_jsonl(&target, &export).await.unwrap();
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM users WHERE id='bootstrap-admin'")
            .fetch_one(&target)
            .await
            .unwrap(),
        1,
        "the initialized admin account is not removed"
    );
    assert_eq!(dashboard_rows(&target).await,expected,"restore includes exact activation timestamp, narrow history, user events and purged request counts");
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM usage")
            .fetch_one(&target)
            .await
            .unwrap(),
        1
    );
    import_postgres_jsonl(&target, &export).await.unwrap();
    assert_eq!(
        dashboard_rows(&target).await,
        expected,
        "repeated import must not count users or requests twice"
    );
    let mut invalid_source = decode_jsonl(&export).unwrap();
    for record in &mut invalid_source {
        if let DataExportRecord::Row {
            domain: ExportDomain::Users,
            payload,
            ..
        } = record
        {
            payload["auth_source"] = Value::String("invalid-auth-source".into());
        }
    }
    assert!(
        import_postgres_jsonl(&target, &encode_jsonl(&invalid_source).unwrap())
            .await
            .is_err()
    );
    assert_eq!(
        dashboard_rows(&target).await,
        expected,
        "failure after the restore guard and deletes rolls the entire transaction back"
    );
    let guard: Option<String> =
        sqlx::query_scalar("SELECT current_setting('aether.dashboard_restore',true)")
            .fetch_one(&target)
            .await
            .unwrap();
    assert_ne!(guard.as_deref(), Some("on"));
    sqlx::query("UPDATE usage SET input_tokens=17,total_tokens=20 WHERE request_id='backup-kept'")
        .execute(&target)
        .await
        .unwrap();
    assert_eq!(
        total(&target).await,
        2,
        "existing restored contribution updates by delta"
    );
    assert_eq!(tokens(&target).await, 46);
    request(&target, "backup-next", 31).await;
    assert_eq!(
        total(&target).await,
        3,
        "future writes resume ordinary aggregation"
    );
    assert_eq!(tokens(&target).await, 80);
    let before_conflict = dashboard_rows(&target).await;
    let error = import_postgres_jsonl(&target, &export).await.unwrap_err();
    assert!(
        error
            .to_string()
            .contains("conflicts with existing statistics"),
        "{error}"
    );
    assert_eq!(
        dashboard_rows(&target).await,
        before_conflict,
        "conflicting snapshots must roll back without overwriting accumulated data"
    );
    let mut truncated = decode_jsonl(&export).unwrap();
    truncated.retain(|row| !matches!(row,DataExportRecord::Row { payload,.. } if payload["__table"]=="dashboard_stats_total" && payload["shard"]==serde_json::json!(1)));
    let error = import_postgres_jsonl(&target, &encode_jsonl(&truncated).unwrap())
        .await
        .unwrap_err();
    assert!(error.to_string().contains("missing, truncated"), "{error}");
    // A legacy file has neither aggregate rows nor the optional snapshot manifest.
    // Its ordinary source import must still trigger aggregation.
    let mut legacy = decode_jsonl(&export).unwrap();
    if let DataExportRecord::Manifest { manifest } = &mut legacy[0] {
        manifest.dashboard_snapshot = None;
    }
    legacy.retain(|row| !matches!(row,DataExportRecord::Row { payload,.. } if payload["__table"].as_str().is_some_and(|name| name.starts_with("dashboard_"))));
    sqlx::query("CREATE DATABASE dashboard_legacy_restore_test")
        .execute(&source)
        .await
        .unwrap();
    let legacy_url = server
        .database_url()
        .strip_suffix("/postgres")
        .unwrap()
        .to_owned()
        + "/dashboard_legacy_restore_test";
    let legacy_target = sqlx::postgres::PgPoolOptions::new()
        .max_connections(2)
        .connect(&legacy_url)
        .await
        .unwrap();
    migrate(&legacy_target).await;
    sqlx::query("UPDATE dashboard_stats_state SET stats_since='2000-01-01'")
        .execute(&legacy_target)
        .await
        .unwrap();
    let legacy_export = encode_jsonl(&legacy).unwrap();
    import_postgres_jsonl(&legacy_target, &legacy_export)
        .await
        .unwrap();
    assert_eq!(total(&legacy_target).await, 1);
    import_postgres_jsonl(&legacy_target, &legacy_export)
        .await
        .unwrap();
    assert_eq!(total(&legacy_target).await, 1);
    legacy_target.close().await;
    target.close().await;
    source.close().await;
}

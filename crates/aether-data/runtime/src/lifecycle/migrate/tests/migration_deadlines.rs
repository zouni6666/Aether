use super::*;
use sqlx::postgres::PgPoolOptions;
use std::time::{Duration, Instant};

const OVERVIEW_MIGRATION: i64 = 20260911000000;

async fn legacy_connection(server: &ManagedPostgresServer) -> PgConnection {
    let mut connection = PgConnection::connect(server.database_url()).await.unwrap();
    connection.ensure_migrations_table().await.unwrap();
    for migration in POSTGRES_MIGRATOR
        .iter()
        .filter(|migration| migration.version < OVERVIEW_MIGRATION)
    {
        connection.apply(migration).await.unwrap();
    }
    connection
}

async fn wait_for_settlement_ddl(connection: &mut PgConnection) {
    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            let waiting: bool = query_scalar(
                "SELECT EXISTS (SELECT 1 FROM pg_locks WHERE relation='public.usage_settlement_snapshots'::regclass AND mode='AccessExclusiveLock' AND NOT granted)",
            )
            .fetch_one(&mut *connection)
            .await
            .unwrap();
            if waiting {
                return;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("migration should reach the blocked settlement ALTER TABLE");
}

async fn assert_overview_migration_rolled_back(pool: &PgPool) {
    assert!(
        !column_exists(pool, "usage", "failure_origin")
            .await
            .unwrap(),
        "the earlier ALTER TABLE must roll back with the blocked statement"
    );
    let stamped: bool =
        query_scalar("SELECT EXISTS (SELECT 1 FROM public._sqlx_migrations WHERE version=$1)")
            .bind(OVERVIEW_MIGRATION)
            .fetch_one(pool)
            .await
            .unwrap();
    assert!(
        !stamped,
        "a failed migration must not receive a success stamp"
    );
}

#[tokio::test]
async fn migration_deadlines_release_queued_usage_work_and_roll_back_before_retry() {
    let Some(server) = ManagedPostgresServer::try_start().await.unwrap() else {
        return;
    };
    let mut observer = legacy_connection(&server).await;
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .after_connect(|connection, _| {
            Box::pin(async move {
                query("SET statement_timeout='30s'")
                    .execute(&mut *connection)
                    .await?;
                query("SET lock_timeout='3s'")
                    .execute(&mut *connection)
                    .await?;
                Ok(())
            })
        })
        .connect(server.database_url())
        .await
        .unwrap();
    let original_pid: i32 = query_scalar("SELECT pg_backend_pid()")
        .fetch_one(&pool)
        .await
        .unwrap();
    let mut business = PgConnection::connect(server.database_url()).await.unwrap();
    let mut business_transaction = business.begin().await.unwrap();
    query("LOCK TABLE public.usage_settlement_snapshots IN ACCESS SHARE MODE")
        .execute(&mut *business_transaction)
        .await
        .unwrap();
    let mut queued_business = PgConnection::connect(server.database_url()).await.unwrap();

    // The usage ALTERs take their table exclusively, then settlement ALTER
    // waits behind an existing reader. Business writes wait on the usage lock
    // already held by this transaction and must resume when it rolls back.
    let started = Instant::now();
    let (migration_result, ()) = tokio::join!(super::super::run_migrations(&pool), async {
        wait_for_settlement_ddl(&mut observer).await;
        let acquired_first_lock: bool = query_scalar(
            "SELECT EXISTS (SELECT 1 FROM pg_locks WHERE relation='public.usage'::regclass AND mode='AccessExclusiveLock' AND granted)",
        )
        .fetch_one(&mut observer)
        .await
        .unwrap();
        assert!(
            acquired_first_lock,
            "the migration must have already changed the first table"
        );
        let queued_write = query("INSERT INTO public.usage(id,request_id,model,provider_name,status,billing_status,created_at) VALUES ('migration-live-request','migration-live-request','test','test','completed','settled',NOW())")
            .execute(&mut queued_business);
        tokio::pin!(queued_write);
        tokio::select! {
            result = &mut queued_write => panic!("the business write should initially queue behind the DDL: {result:?}"),
            () = tokio::time::sleep(Duration::from_millis(100)) => {}
        }
        let written = tokio::time::timeout(Duration::from_secs(4), queued_write)
            .await
            .expect("queued business work must resume after the migration's lock timeout")
            .unwrap();
        assert_eq!(written.rows_affected(), 1);
    });
    let error =
        migration_result.expect_err("busy settlement table must defer this upgrade attempt");
    assert!(
        error.to_string().contains("lock timeout"),
        "the failure should identify the bounded lock wait: {error}"
    );
    assert!(started.elapsed() < Duration::from_secs(5));
    assert_overview_migration_rolled_back(&pool).await;

    let (new_pid, statement_timeout, lock_timeout): (i32, String, String) = sqlx::query_as(
        "SELECT pg_backend_pid(), current_setting('statement_timeout'), current_setting('lock_timeout')",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_ne!(
        new_pid, original_pid,
        "failed migration connection must be discarded"
    );
    assert_eq!(statement_timeout, "30s");
    assert_eq!(lock_timeout, "3s");

    business_transaction.rollback().await.unwrap();
    super::super::run_migrations(&pool).await.unwrap();
    assert!(super::super::pending_migrations(&pool)
        .await
        .unwrap()
        .is_empty());
    let success: bool =
        query_scalar("SELECT success FROM public._sqlx_migrations WHERE version=$1")
            .bind(OVERVIEW_MIGRATION)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(
        success,
        "a later quiet retry must succeed without manual stamp repair"
    );
    assert_eq!(
        query_scalar::<_, i64>(
            "SELECT count(*) FROM public.usage WHERE request_id='migration-live-request'"
        )
        .fetch_one(&pool)
        .await
        .unwrap(),
        1,
        "the resumed business write must survive the upgrade retry"
    );
    let settings: (String, String) = sqlx::query_as(
        "SELECT current_setting('statement_timeout'), current_setting('lock_timeout')",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(settings, ("30s".into(), "3s".into()));

    // Startup preparation also takes SQLx's advisory migration lock. Another
    // upgrade process must not leave startup waiting indefinitely for it.
    observer.lock().await.unwrap();
    let preparation =
        tokio::time::timeout(Duration::from_secs(4), prepare_database_for_startup(&pool))
            .await
            .expect("startup preparation must bound its advisory-lock wait");
    let preparation_error = preparation.expect_err("another migration owns the advisory lock");
    assert!(preparation_error.to_string().contains("lock timeout"));
    observer.unlock().await.unwrap();
    assert!(prepare_database_for_startup(&pool)
        .await
        .unwrap()
        .is_empty());
    pool.close().await;
}

#[tokio::test]
async fn migration_deadlines_caller_cancellation_releases_ddl_locks_and_rolls_back() {
    let Some(server) = ManagedPostgresServer::try_start().await.unwrap() else {
        return;
    };
    let mut connection = legacy_connection(&server).await;
    sqlx::raw_sql(
        r#"
CREATE FUNCTION public.test_cancelled_migration_ddl() RETURNS event_trigger LANGUAGE plpgsql AS $$
BEGIN PERFORM pg_sleep(8); END $$;
CREATE EVENT TRIGGER test_cancelled_migration_ddl ON ddl_command_end
  WHEN TAG IN ('ALTER TABLE') EXECUTE FUNCTION public.test_cancelled_migration_ddl();
"#,
    )
    .execute(&mut connection)
    .await
    .unwrap();
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(server.database_url())
        .await
        .unwrap();
    // Keep this future in an inner scope: leaving it actually drops the entire
    // migration operation, rather than merely dropping a pinned reference.
    {
        let migrate = super::super::run_migrations(&pool);
        tokio::pin!(migrate);
        tokio::select! {
            result = &mut migrate => panic!("the caller must cancel before the slow migration finishes: {result:?}"),
            () = async {
                tokio::time::timeout(Duration::from_secs(4), async {
                    loop {
                        let running: bool = query_scalar(
                            "SELECT EXISTS (SELECT 1 FROM pg_locks l JOIN pg_stat_activity a USING(pid) WHERE l.relation='public.usage'::regclass AND l.mode='AccessExclusiveLock' AND l.granted AND a.wait_event='PgSleep')",
                        )
                        .fetch_one(&mut connection)
                        .await
                        .unwrap();
                        if running {
                            return;
                        }
                        tokio::time::sleep(Duration::from_millis(10)).await;
                    }
                }).await.expect("migration must acquire its first DDL lock before caller cancellation");
            } => {}
        }
    }
    tokio::time::timeout(
        Duration::from_secs(4),
        query_scalar::<_, i64>("SELECT count(*) FROM public.usage").fetch_one(&pool),
    )
    .await
    .expect("dropping the caller future must stop the server-side statement and release DDL locks")
    .unwrap();
    assert_overview_migration_rolled_back(&pool).await;
    sqlx::raw_sql(
        "DROP EVENT TRIGGER test_cancelled_migration_ddl; DROP FUNCTION public.test_cancelled_migration_ddl()",
    )
    .execute(&mut connection)
    .await
    .unwrap();
    super::super::run_migrations(&pool).await.unwrap();
    assert!(super::super::pending_migrations(&pool)
        .await
        .unwrap()
        .is_empty());
    pool.close().await;
}

#[tokio::test]
async fn migration_deadlines_bound_the_whole_transaction_not_only_each_statement() {
    let Some(server) = ManagedPostgresServer::try_start().await.unwrap() else {
        return;
    };
    let mut connection = legacy_connection(&server).await;
    // Each ALTER finishes within the ten-second statement limit, but the
    // entire migration exceeds it. A per-statement timeout alone is insufficient.
    sqlx::raw_sql(
        r#"
CREATE FUNCTION public.test_slow_migration_ddl() RETURNS event_trigger LANGUAGE plpgsql AS $$
BEGIN PERFORM pg_sleep(3); END $$;
CREATE EVENT TRIGGER test_slow_migration_ddl ON ddl_command_end
  WHEN TAG IN ('ALTER TABLE') EXECUTE FUNCTION public.test_slow_migration_ddl();
"#,
    )
    .execute(&mut connection)
    .await
    .unwrap();
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(server.database_url())
        .await
        .unwrap();
    let started = Instant::now();
    let result = tokio::time::timeout(Duration::from_secs(15), super::super::run_migrations(&pool))
        .await
        .expect("the whole migration deadline must fire before all slow statements finish");
    assert!(result.is_err(), "the over-budget migration must fail");
    assert!(
        started.elapsed() >= Duration::from_secs(9),
        "upgrade failed before its deadline: {result:?}"
    );
    assert!(started.elapsed() < Duration::from_secs(15));
    // The backend can still be unwinding the statement when its socket closes.
    // Reading the first locked table proves cancellation released the DDL lock.
    tokio::time::timeout(
        Duration::from_secs(4),
        query_scalar::<_, i64>("SELECT count(*) FROM public.usage").fetch_one(&pool),
    )
    .await
    .expect("DDL locks must be released when the migration connection closes")
    .unwrap();
    assert_overview_migration_rolled_back(&pool).await;
    sqlx::raw_sql(
        "DROP EVENT TRIGGER test_slow_migration_ddl; DROP FUNCTION public.test_slow_migration_ddl()",
    )
    .execute(&mut connection)
    .await
    .unwrap();
    super::super::run_migrations(&pool).await.unwrap();
    assert!(super::super::pending_migrations(&pool)
        .await
        .unwrap()
        .is_empty());
    pool.close().await;
}

use super::*;
use aether_data_contracts::repository::usage::UsageDashboardAnalyticsQuery;

const ANONYMIZATION_VERSION: i64 = 20261001000000;
const INSERT_USER: &str = "INSERT INTO users(id,username,email_verified) VALUES($1,$1,false)";
const INSERT_USAGE: &str = "INSERT INTO usage(id,request_id,user_id,model,provider_name,status,billing_status,created_at) VALUES($1,$1,$2,'anonymization','test','completed','settled',clock_timestamp())";

async fn assert_anonymous(pool: &PgPool, user: &str) {
    let counts: (i64, i64) = sqlx::query_as(
        "SELECT (SELECT count(*) FROM dashboard_actor_minute WHERE actor_user_id=$1), (SELECT count(*) FROM dashboard_request_contributions WHERE actor_user_id=$1)",
    )
    .bind(user)
    .fetch_one(pool)
    .await
    .unwrap();
    assert_eq!(counts, (0, 0), "retained identity for {user}");
}

#[tokio::test]
async fn dashboard_user_anonymization_preserves_totals_without_migration_backfill() {
    let Some(server) = ManagedPostgresServer::try_start().await.unwrap() else {
        return;
    };
    let mut connection = PgConnection::connect(server.database_url()).await.unwrap();
    connection.ensure_migrations_table().await.unwrap();
    for migration in POSTGRES_MIGRATOR
        .iter()
        .filter(|migration| migration.version < ANONYMIZATION_VERSION)
    {
        connection.apply(migration).await.unwrap();
    }
    let pool = PgPool::connect(server.database_url()).await.unwrap();
    query(INSERT_USER)
        .bind("legacy-deleted")
        .execute(&pool)
        .await
        .unwrap();
    query(INSERT_USAGE)
        .bind("legacy-request")
        .bind("legacy-deleted")
        .execute(&pool)
        .await
        .unwrap();
    query("DELETE FROM usage WHERE request_id='legacy-request'")
        .execute(&pool)
        .await
        .unwrap();
    query("DELETE FROM users WHERE id='legacy-deleted'")
        .execute(&pool)
        .await
        .unwrap();
    let actor_before: String =
        query_scalar("SELECT jsonb_agg(to_jsonb(a))::text FROM dashboard_actor_minute a")
            .fetch_one(&pool)
            .await
            .unwrap();

    // An upgrade must succeed even while historical projection tables are
    // inaccessible: install definitions without reading or rewriting their rows.
    let mut blocked_history = pool.begin().await.unwrap();
    query("LOCK TABLE dashboard_actor_minute, dashboard_request_contributions IN ACCESS EXCLUSIVE MODE")
        .execute(&mut *blocked_history).await.unwrap();
    query("SET lock_timeout='500ms'")
        .execute(&mut connection)
        .await
        .unwrap();
    let migration = POSTGRES_MIGRATOR
        .iter()
        .find(|migration| migration.version == ANONYMIZATION_VERSION)
        .unwrap();
    connection.apply(migration).await.unwrap();
    blocked_history.rollback().await.unwrap();
    let actor_after: String =
        query_scalar("SELECT jsonb_agg(to_jsonb(a))::text FROM dashboard_actor_minute a")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(
        actor_before, actor_after,
        "migration must not rewrite old actors"
    );

    let repo = aether_data_postgres::SqlxUsageReadRepository::new(pool.clone());
    let summary_query = UsageDashboardAnalyticsQuery {
        timezone: "UTC".into(),
    };
    let legacy = repo.query_dashboard_summary(&summary_query).await.unwrap();
    assert_eq!(
        legacy.today.active_users, 0,
        "old orphan actors must be excluded without backfill"
    );
    assert_eq!(legacy.total.request_count, 1);

    for (user, purge, soft) in [
        ("hard-live", false, false),
        ("hard-purged", true, false),
        ("soft-live", false, true),
        ("soft-purged", true, true),
    ] {
        query(INSERT_USER).bind(user).execute(&pool).await.unwrap();
        query(INSERT_USAGE)
            .bind(user)
            .bind(user)
            .execute(&pool)
            .await
            .unwrap();
        if purge {
            query("DELETE FROM usage WHERE request_id=$1")
                .bind(user)
                .execute(&pool)
                .await
                .unwrap();
        }
        let before = repo.query_dashboard_summary(&summary_query).await.unwrap();
        let sql = if soft {
            "UPDATE users SET is_deleted=true WHERE id=$1"
        } else {
            "DELETE FROM users WHERE id=$1"
        };
        query(sql).bind(user).execute(&pool).await.unwrap();
        assert_anonymous(&pool, user).await;
        let after = repo.query_dashboard_summary(&summary_query).await.unwrap();
        assert_eq!(
            after.total, before.total,
            "deletion must preserve request totals"
        );
        assert_eq!(after.today.active_users, 0);
        if !purge {
            query("UPDATE usage SET response_time_ms=200 WHERE request_id=$1")
                .bind(user)
                .execute(&pool)
                .await
                .unwrap();
            assert_anonymous(&pool, user).await;
        }
        if soft {
            query("DELETE FROM users WHERE id=$1")
                .bind(user)
                .execute(&pool)
                .await
                .unwrap();
            assert_anonymous(&pool, user).await;
        }
    }

    // Exercise both deferred-event orders, plus explicitly immediate constraint
    // triggers. A captured deleted_fact must never recreate a deleted actor.
    for (user, user_first, immediate) in [
        ("deferred-request-first", false, false),
        ("deferred-user-first", true, false),
        ("immediate-user", true, true),
    ] {
        query(INSERT_USER).bind(user).execute(&pool).await.unwrap();
        let mut tx = pool.begin().await.unwrap();
        if immediate {
            query("SET CONSTRAINTS ALL IMMEDIATE")
                .execute(&mut *tx)
                .await
                .unwrap();
        }
        query(INSERT_USAGE)
            .bind(user)
            .bind(user)
            .execute(&mut *tx)
            .await
            .unwrap();
        if user_first {
            query("DELETE FROM users WHERE id=$1")
                .bind(user)
                .execute(&mut *tx)
                .await
                .unwrap();
        }
        query("DELETE FROM usage WHERE request_id=$1")
            .bind(user)
            .execute(&mut *tx)
            .await
            .unwrap();
        if !user_first {
            query("DELETE FROM users WHERE id=$1")
                .bind(user)
                .execute(&mut *tx)
                .await
                .unwrap();
        }
        tx.commit().await.unwrap();
        assert_anonymous(&pool, user).await;
    }

    // A concurrent writer can still see a user after its deletion statement
    // but before that transaction commits. Check both commit orders: the new
    // actor must be rejected after the shard lock, or removed by the deleter.
    for (user, writer_first) in [
        ("concurrent-delete-first", false),
        ("concurrent-writer-first", true),
    ] {
        query(INSERT_USER).bind(user).execute(&pool).await.unwrap();
        let mut deletion = pool.begin().await.unwrap();
        query("DELETE FROM users WHERE id=$1")
            .bind(user)
            .execute(&mut *deletion)
            .await
            .unwrap();
        let mut writer = pool.begin().await.unwrap();
        query(INSERT_USAGE)
            .bind(user)
            .bind(user)
            .execute(&mut *writer)
            .await
            .unwrap();
        tokio::time::timeout(std::time::Duration::from_secs(5), async {
            if writer_first {
                writer.commit().await.unwrap();
                deletion.commit().await.unwrap();
            } else {
                deletion.commit().await.unwrap();
                writer.commit().await.unwrap();
            }
        })
        .await
        .expect("concurrent deletion and usage must not deadlock");
        assert_anonymous(&pool, user).await;
    }
    let final_summary = repo.query_dashboard_summary(&summary_query).await.unwrap();
    assert_eq!(final_summary.total.request_count, 10);
    assert_eq!(final_summary.today.active_users, 0);
    let invalid: (i64, i64, i64) = sqlx::query_as("SELECT (SELECT count(*) FROM dashboard_actor_minute WHERE request_count < 0), (SELECT count(*) FROM dashboard_stats_pending), (SELECT count(*) FROM dashboard_user_anonymization_pending)")
        .fetch_one(&pool).await.unwrap();
    assert_eq!(invalid, (0, 0, 0));
    pool.close().await;
}

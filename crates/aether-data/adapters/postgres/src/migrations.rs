use std::collections::{HashMap, HashSet};
use std::future::Future;
use std::pin::Pin;

use sqlx::{
    migrate::{AppliedMigration, Migrate, MigrateError, Migrator},
    query, query_scalar, Connection, PgConnection, PgPool,
};
use tracing::{error, info, warn};

use aether_data_contracts::PendingMigrationInfo;

mod cancellation;
mod timeouts;
use cancellation::MigrationAbortGuard;
use timeouts::{with_deadline, MigrationTimeouts};

pub static POSTGRES_MIGRATOR: Migrator = sqlx::migrate!("./migrations");
const MIGRATIONS_TABLE_EXISTS_SQL: &str =
    "SELECT to_regclass('public._sqlx_migrations') IS NOT NULL";
const INVALID_CONCURRENT_INDEX_EXISTS_SQL: &str = r#"
SELECT EXISTS (
    SELECT 1
    FROM pg_catalog.pg_class AS index_relation
    JOIN pg_catalog.pg_namespace AS index_namespace
      ON index_namespace.oid = index_relation.relnamespace
    JOIN pg_catalog.pg_index AS index_state
      ON index_state.indexrelid = index_relation.oid
    WHERE index_namespace.nspname = 'public'
      AND index_relation.relname = $1
      AND NOT index_state.indisvalid
)
"#;

pub type BootstrapFuture<'a> = Pin<Box<dyn Future<Output = Result<(), MigrateError>> + 'a>>;

pub trait PostgresMigrationBootstrap: Send + Sync {
    fn apply_snapshot<'a>(
        &self,
        conn: &'a mut PgConnection,
        migrator: &'static Migrator,
    ) -> BootstrapFuture<'a>;
}

#[derive(Debug, Clone, Copy, Default)]
struct NoopBootstrap;

impl PostgresMigrationBootstrap for NoopBootstrap {
    fn apply_snapshot<'a>(
        &self,
        _conn: &'a mut PgConnection,
        _migrator: &'static Migrator,
    ) -> BootstrapFuture<'a> {
        Box::pin(async { Ok(()) })
    }
}

pub async fn run_migrations(pool: &PgPool) -> Result<(), MigrateError> {
    run_migrations_with_bootstrap(pool, &NoopBootstrap).await
}

pub async fn run_migrations_with_bootstrap(
    pool: &PgPool,
    bootstrap: &dyn PostgresMigrationBootstrap,
) -> Result<(), MigrateError> {
    let timeouts = MigrationTimeouts::from_env()?;
    let mut conn = crate::pool::acquire_postgres_migration_connection(pool).await?;
    let mut abort_guard = MigrationAbortGuard::new(pool, &mut conn).await?;
    let result = async {
        with_deadline(timeouts.transaction_ms, None, async {
            timeouts.apply(&mut conn, false).await?;
            if POSTGRES_MIGRATOR.locking {
                conn.lock().await?;
            }
            prepare_database_for_startup_locked(&mut conn, bootstrap).await?;
            Ok(())
        })
        .await?;
        run_migrations_locked(&mut conn, timeouts).await?;
        if POSTGRES_MIGRATOR.locking {
            conn.unlock().await?;
        }
        Ok(())
    }
    .await;
    // A dropped SQLx future can leave an active/aborted transaction behind. Do
    // not issue more SQL or return this connection to the pool on any error.
    if result.is_err() {
        abort_guard.abort().await;
        let _ = tokio::time::timeout(
            std::time::Duration::from_secs(1),
            conn.detach().close_hard(),
        )
        .await;
    } else {
        abort_guard.disarm();
    }
    result
}

pub async fn pending_migrations(pool: &PgPool) -> Result<Vec<PendingMigrationInfo>, MigrateError> {
    let mut conn = pool.acquire().await?;
    pending_migrations_locked(&mut conn).await
}

pub async fn prepare_database_for_startup(
    pool: &PgPool,
) -> Result<Vec<PendingMigrationInfo>, MigrateError> {
    prepare_database_for_startup_with_bootstrap(pool, &NoopBootstrap).await
}

pub async fn prepare_database_for_startup_with_bootstrap(
    pool: &PgPool,
    bootstrap: &dyn PostgresMigrationBootstrap,
) -> Result<Vec<PendingMigrationInfo>, MigrateError> {
    let timeouts = MigrationTimeouts::from_env()?;
    let mut conn = crate::pool::acquire_postgres_migration_connection(pool).await?;
    let mut abort_guard = MigrationAbortGuard::new(pool, &mut conn).await?;
    let result = with_deadline(timeouts.transaction_ms, None, async {
        timeouts.apply(&mut conn, false).await?;
        if POSTGRES_MIGRATOR.locking {
            conn.lock().await?;
        }
        let pending = prepare_database_for_startup_locked(&mut conn, bootstrap).await?;
        if POSTGRES_MIGRATOR.locking {
            conn.unlock().await?;
        }
        Ok(pending)
    })
    .await;
    if result.is_err() {
        abort_guard.abort().await;
        let _ = tokio::time::timeout(
            std::time::Duration::from_secs(1),
            conn.detach().close_hard(),
        )
        .await;
    } else {
        abort_guard.disarm();
    }
    result
}

async fn run_migrations_locked(
    conn: &mut PgConnection,
    timeouts: MigrationTimeouts,
) -> Result<(), MigrateError> {
    // Snapshot SQL and legacy migrations may change session settings. Restore
    // the limits before inspecting or applying the next migration.
    timeouts.apply(conn, false).await?;
    if let Some(version) = conn.dirty_version().await? {
        error!(version, "database migration state is dirty");
        return Err(MigrateError::Dirty(version));
    }

    let applied_migrations = conn.list_applied_migrations().await?;
    validate_applied_migrations(&applied_migrations)?;
    let applied_migrations_by_version = applied_migrations
        .into_iter()
        .map(|migration| (migration.version, migration))
        .collect::<HashMap<_, _>>();
    let pending_migrations = POSTGRES_MIGRATOR
        .iter()
        .filter(|migration| migration.migration_type.is_up_migration())
        .filter(|migration| !applied_migrations_by_version.contains_key(&migration.version))
        .collect::<Vec<_>>();
    let total_migrations = POSTGRES_MIGRATOR
        .iter()
        .filter(|migration| migration.migration_type.is_up_migration())
        .count();
    let applied_count = total_migrations.saturating_sub(pending_migrations.len());

    if pending_migrations.is_empty() {
        info!(
            total_migrations,
            applied_migrations = applied_count,
            pending_migrations = 0,
            "database migrations already up to date"
        );
        return Ok(());
    }

    info!(
        total_migrations,
        applied_migrations = applied_count,
        pending_migrations = pending_migrations.len(),
        "database migrations pending"
    );
    for (index, migration) in pending_migrations.iter().enumerate() {
        let current = index + 1;
        info!(
            current,
            total = pending_migrations.len(),
            version = migration.version,
            description = %migration.description,
            lock_timeout_ms = timeouts.lock_ms,
            migration_timeout_ms = timeouts.execution_ms(migration.no_tx),
            "applying database migration"
        );
        let elapsed = with_deadline(
            timeouts.execution_ms(migration.no_tx),
            Some(migration.version),
            async {
                timeouts.apply(conn, migration.no_tx).await?;
                repair_invalid_concurrent_index(conn, migration.version).await?;
                conn.apply(migration).await
            },
        )
        .await?;
        info!(
            current,
            total = pending_migrations.len(),
            version = migration.version,
            description = %migration.description,
            elapsed_ms = elapsed.as_millis() as u64,
            "applied database migration"
        );
    }
    info!(
        total_migrations,
        applied_migrations = total_migrations,
        pending_migrations = 0,
        "database migrations complete"
    );
    Ok(())
}

async fn repair_invalid_concurrent_index(
    conn: &mut PgConnection,
    migration_version: i64,
) -> Result<(), MigrateError> {
    // Only these fixed, trusted identifiers may be interpolated into DROP INDEX.
    let index_name = match migration_version {
        20260715000000 => "idx_usage_legacy_body_ref_cleanup_created_at",
        20260715130000 => "idx_usage_settlement_dashboard_cover",
        20260720000000 => "idx_usage_stale_pending_created_request",
        20260918000000 => "idx_usage_settlement_dashboard_cover_v2",
        20260920000000 => "idx_payment_orders_status_credited_user",
        20260921020000 => "ix_usage_attribution_owner_request",
        20260921020100 => "ix_usage_analytics_actor_metadata",
        _ => return Ok(()),
    };

    let invalid_index_exists: bool = query_scalar(INVALID_CONCURRENT_INDEX_EXISTS_SQL)
        .bind(index_name)
        .fetch_one(&mut *conn)
        .await?;
    if !invalid_index_exists {
        return Ok(());
    }

    warn!(
        migration_version,
        index = index_name,
        "dropping invalid index left by an interrupted concurrent migration"
    );
    query(&format!(
        "DROP INDEX CONCURRENTLY IF EXISTS public.{index_name}"
    ))
    .execute(&mut *conn)
    .await?;
    Ok(())
}

async fn prepare_database_for_startup_locked(
    conn: &mut PgConnection,
    bootstrap: &dyn PostgresMigrationBootstrap,
) -> Result<Vec<PendingMigrationInfo>, MigrateError> {
    conn.ensure_migrations_table().await?;
    bootstrap.apply_snapshot(conn, &POSTGRES_MIGRATOR).await?;
    pending_migrations_locked(conn).await
}

async fn pending_migrations_locked(
    conn: &mut PgConnection,
) -> Result<Vec<PendingMigrationInfo>, MigrateError> {
    if !migrations_table_exists(conn).await? {
        return Ok(all_up_migrations());
    }
    if let Some(version) = conn.dirty_version().await? {
        error!(version, "database migration state is dirty");
        return Err(MigrateError::Dirty(version));
    }
    let applied_migrations = conn.list_applied_migrations().await?;
    validate_applied_migrations(&applied_migrations)?;
    Ok(pending_migrations_from_applied(&applied_migrations))
}

async fn migrations_table_exists(conn: &mut PgConnection) -> Result<bool, MigrateError> {
    Ok(query_scalar(MIGRATIONS_TABLE_EXISTS_SQL)
        .fetch_one(&mut *conn)
        .await?)
}

pub fn all_up_migrations() -> Vec<PendingMigrationInfo> {
    POSTGRES_MIGRATOR
        .iter()
        .filter(|migration| migration.migration_type.is_up_migration())
        .map(|migration| PendingMigrationInfo {
            version: migration.version,
            description: migration.description.to_string(),
        })
        .collect()
}

pub fn pending_migrations_from_applied(
    applied_migrations: &[AppliedMigration],
) -> Vec<PendingMigrationInfo> {
    let applied_versions = applied_migrations
        .iter()
        .map(|migration| migration.version)
        .collect::<HashSet<_>>();
    POSTGRES_MIGRATOR
        .iter()
        .filter(|migration| migration.migration_type.is_up_migration())
        .filter(|migration| !applied_versions.contains(&migration.version))
        .map(|migration| PendingMigrationInfo {
            version: migration.version,
            description: migration.description.to_string(),
        })
        .collect()
}

fn validate_applied_migrations(
    applied_migrations: &[AppliedMigration],
) -> Result<(), MigrateError> {
    if POSTGRES_MIGRATOR.ignore_missing {
        return Ok(());
    }
    let known_versions = POSTGRES_MIGRATOR
        .iter()
        .map(|migration| migration.version)
        .collect::<HashSet<_>>();
    for applied_migration in applied_migrations {
        if !known_versions.contains(&applied_migration.version) {
            error!(
                version = applied_migration.version,
                "applied database migration is missing from embedded migrations"
            );
            return Err(MigrateError::VersionMissing(applied_migration.version));
        }
    }
    for migration in POSTGRES_MIGRATOR
        .iter()
        .filter(|migration| migration.migration_type.is_up_migration())
    {
        if let Some(applied_migration) = applied_migrations
            .iter()
            .find(|applied_migration| applied_migration.version == migration.version)
        {
            if migration.checksum != applied_migration.checksum {
                warn!(
                    version = migration.version,
                    description = %migration.description,
                    "database migration checksum mismatch (ignored: version-only validation)"
                );
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        all_up_migrations, pending_migrations_from_applied, validate_applied_migrations,
        POSTGRES_MIGRATOR,
    };

    #[test]
    fn historical_account_attribution_migration_remains_valid() {
        let checksum = "16210b169c8fc1e428de0336b836170652014a39a53e966bc3a25d5080c5b7e2532c7aed085b7e186c4c9a0d7ea0ea2f";
        let historical = sqlx::migrate::AppliedMigration {
            version: 20260917000000,
            checksum: (0..checksum.len())
                .step_by(2)
                .map(|offset| u8::from_str_radix(&checksum[offset..offset + 2], 16).unwrap())
                .collect::<Vec<_>>()
                .into(),
        };
        validate_applied_migrations(std::slice::from_ref(&historical)).unwrap();
        let embedded = POSTGRES_MIGRATOR
            .iter()
            .find(|migration| migration.version == historical.version)
            .unwrap();
        assert_eq!(embedded.checksum, historical.checksum);
        assert!(!pending_migrations_from_applied(&[historical])
            .iter()
            .any(|migration| migration.version == 20260917000000));
    }

    #[test]
    fn unknown_applied_migrations_still_block_startup() {
        let unknown = sqlx::migrate::AppliedMigration {
            version: 20990101000000,
            checksum: Vec::new().into(),
        };
        assert!(matches!(
            validate_applied_migrations(&[unknown]),
            Err(sqlx::migrate::MigrateError::VersionMissing(20990101000000))
        ));
    }

    #[test]
    fn embeds_ordered_postgres_migration_sources() {
        let versions = POSTGRES_MIGRATOR
            .iter()
            .map(|migration| migration.version)
            .collect::<Vec<_>>();
        assert!(!versions.is_empty());
        assert!(versions.windows(2).all(|pair| pair[0] < pair[1]));
        assert_eq!(pending_migrations_from_applied(&[]), all_up_migrations());
    }

    #[test]
    fn pending_migrations_preserve_gaps_and_do_not_repeat_completed_index_builds() {
        let applied = POSTGRES_MIGRATOR
            .iter()
            .filter(|migration| migration.version != 20260918000000)
            .map(|migration| sqlx::migrate::AppliedMigration {
                version: migration.version,
                checksum: migration.checksum.clone(),
            })
            .collect::<Vec<_>>();
        let pending = pending_migrations_from_applied(&applied);
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].version, 20260918000000);
    }

    #[test]
    fn embeds_scoped_codex_live_permission_migration() {
        let migration = POSTGRES_MIGRATOR
            .iter()
            .find(|migration| migration.version == 20260821000000)
            .expect("Codex Live permission migration should be embedded");
        let sql = migration.sql.as_ref();

        for required_fragment in [
            "UPDATE public.users",
            "UPDATE public.user_groups",
            "UPDATE public.api_keys",
            "UPDATE public.provider_api_keys",
            "provider.provider_type",
            "openai:responses",
            "codex:live",
        ] {
            assert!(
                sql.contains(required_fragment),
                "Codex Live permission migration is missing {required_fragment}"
            );
        }
    }

    #[test]
    fn concurrent_index_migrations_opt_out_of_transactions() {
        for version in [
            20260715000000,
            20260715130000,
            20260720000000,
            20260918000000,
            20260918000100,
            20260920000000,
            20260921020000,
            20260921020100,
        ] {
            let migration = POSTGRES_MIGRATOR
                .iter()
                .find(|migration| migration.version == version)
                .expect("concurrent index migration should be embedded");
            assert!(
                migration.no_tx,
                "migration {version} must run without a transaction"
            );
        }

        let analyze_tuning = POSTGRES_MIGRATOR
            .iter()
            .find(|migration| migration.version == 20260715130100)
            .expect("analyze tuning migration should be embedded");
        assert!(!analyze_tuning.no_tx);
    }

    #[test]
    fn tunnel_generation_backfill_does_not_require_pgcrypto() {
        let migration = POSTGRES_MIGRATOR
            .iter()
            .find(|migration| migration.version == 20260831010000)
            .expect("tunnel generation migration should be embedded");
        let sql = migration.sql.as_ref();

        // Existing installations may not have the optional pgcrypto extension.
        // Legacy rows only need an opaque epoch marker; new registrations use a
        // CSPRNG in the application layer.
        assert!(sql.contains("SET tunnel_generation = md5("));
        assert!(sql.contains("ctid::text"));
        assert!(!sql.contains("gen_random_uuid"));
    }

    #[test]
    fn gemini_file_mapping_metadata_migration_expands_legacy_columns() {
        let migration = POSTGRES_MIGRATOR
            .iter()
            .find(|migration| migration.version == 20260901000000)
            .expect("Gemini file mapping metadata migration should be embedded");
        let sql = migration.sql.as_ref();

        for expected in [
            "file_name TYPE character varying(512)",
            "display_name TYPE character varying(512)",
            "mime_type TYPE character varying(255)",
        ] {
            assert!(sql.contains(expected), "migration is missing {expected}");
        }
    }
}

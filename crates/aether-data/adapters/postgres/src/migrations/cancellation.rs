use std::time::Duration;

use sqlx::{migrate::MigrateError, Connection, PgConnection, PgPool};
use tokio::task::JoinHandle;
use tracing::warn;

use super::timeouts::with_deadline;

// A client-side timeout does not cancel PostgreSQL's running statement. Use a
// separate connection to terminate only our own migration session, including
// when the caller drops the migration future. Never borrow from the pool here:
// it can have a single connection, currently held by the migration itself.
pub(super) struct MigrationAbortGuard {
    control: Option<PgConnection>,
    backend: Option<(i32, String)>,
}

impl MigrationAbortGuard {
    pub async fn new(pool: &PgPool, conn: &mut PgConnection) -> Result<Self, MigrateError> {
        let (backend, control) = with_deadline(3_000, None, async {
            let backend = sqlx::query_as::<_, (i32, String)>(
                "SELECT pid, backend_start::text FROM pg_stat_activity WHERE pid = pg_backend_pid()",
            )
            .fetch_one(conn)
            .await?;
            // Reserve the cancellation connection before taking any migration
            // locks. A full server must fail preparation, not prevent cleanup.
            let control = PgConnection::connect_with(pool.connect_options().as_ref()).await?;
            Ok((backend, control))
        })
        .await?;
        Ok(Self {
            control: Some(control),
            backend: Some(backend),
        })
    }

    pub fn disarm(&mut self) {
        self.backend = None;
        self.control = None;
    }

    pub async fn abort(&mut self) {
        if let Some(task) = self.start_abort() {
            // Dropping this await leaves the cleanup task running.
            let _ = task.await;
        }
    }

    fn start_abort(&mut self) -> Option<JoinHandle<()>> {
        let (pid, started_at) = self.backend.take()?;
        let mut control = self.control.take()?;
        let runtime = match tokio::runtime::Handle::try_current() {
            Ok(runtime) => runtime,
            Err(error) => {
                warn!(pid, %error, "runtime unavailable to terminate migration session");
                return None;
            }
        };
        Some(runtime.spawn(async move {
            let result = tokio::time::timeout(Duration::from_secs(3), async {
                // backend_start guards PID reuse. Same role + same database
                // guards against ever signalling an unrelated database user.
                // The second argument waits at most one second for termination.
                let terminated = sqlx::query_scalar::<_, bool>(
                    "SELECT pg_terminate_backend(pid, 1000) FROM pg_stat_activity \
                     WHERE pid = $1 AND backend_start = $2::text::timestamptz \
                     AND usename = current_user AND datname = current_database()",
                )
                .bind(pid)
                .bind(started_at)
                .fetch_optional(&mut control)
                .await?;
                if terminated == Some(false) {
                    warn!(
                        pid,
                        "migration session did not terminate within the cleanup deadline"
                    );
                }
                control.close_hard().await
            })
            .await;
            match result {
                Ok(Ok(())) => {}
                Ok(Err(error)) => warn!(pid, %error, "failed to terminate migration session"),
                Err(error) => warn!(pid, %error, "migration session cleanup timed out"),
            }
        }))
    }
}

impl Drop for MigrationAbortGuard {
    fn drop(&mut self) {
        // SQLx close_on_drop still discards the data connection; the independent
        // task makes cancellation release server-side DDL locks promptly too.
        let _ = self.start_abort();
    }
}

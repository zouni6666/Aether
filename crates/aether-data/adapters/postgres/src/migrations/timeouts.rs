use std::{future::Future, io, time::Duration};

use sqlx::{migrate::MigrateError, PgConnection};

const LOCK_TIMEOUT_ENV: &str = "AETHER_POSTGRES_MIGRATION_LOCK_TIMEOUT_MS";
const TRANSACTION_TIMEOUT_ENV: &str = "AETHER_POSTGRES_MIGRATION_TIMEOUT_MS";
const CONCURRENT_TIMEOUT_ENV: &str = "AETHER_POSTGRES_MIGRATION_CONCURRENT_TIMEOUT_MS";

#[derive(Clone, Copy)]
pub(super) struct MigrationTimeouts {
    pub lock_ms: u32,
    pub transaction_ms: u32,
    concurrent_ms: u32,
}

impl MigrationTimeouts {
    pub fn from_env() -> Result<Self, MigrateError> {
        Ok(Self {
            lock_ms: read_timeout(LOCK_TIMEOUT_ENV, 1_000)?,
            transaction_ms: read_timeout(TRANSACTION_TIMEOUT_ENV, 10_000)?,
            concurrent_ms: read_timeout(CONCURRENT_TIMEOUT_ENV, 900_000)?,
        })
    }

    pub fn execution_ms(self, no_transaction: bool) -> u32 {
        if no_transaction {
            self.concurrent_ms
        } else {
            self.transaction_ms
        }
    }

    pub async fn apply(
        self,
        conn: &mut PgConnection,
        no_transaction: bool,
    ) -> Result<(), MigrateError> {
        sqlx::query(
            "SELECT set_config('lock_timeout', $1, false), \
                    set_config('statement_timeout', $2, false)",
        )
        .bind(format!("{}ms", self.lock_ms))
        .bind(format!("{}ms", self.execution_ms(no_transaction)))
        .execute(conn)
        .await?;
        Ok(())
    }
}

fn read_timeout(name: &str, default: u32) -> Result<u32, MigrateError> {
    match std::env::var(name) {
        Ok(value) => parse_timeout(name, &value),
        Err(std::env::VarError::NotPresent) => Ok(default),
        Err(error) => Err(MigrateError::Source(Box::new(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("{name}: {error}"),
        )))),
    }
}

fn parse_timeout(name: &str, value: &str) -> Result<u32, MigrateError> {
    value
        .trim()
        .parse::<u32>()
        .ok()
        .filter(|value| (1..=i32::MAX as u32).contains(value))
        .ok_or_else(|| {
            MigrateError::Source(Box::new(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("{name} must be milliseconds in 1..=2147483647; migration timeouts cannot be disabled"),
            )))
        })
}

pub(super) async fn with_deadline<T>(
    milliseconds: u32,
    version: Option<i64>,
    operation: impl Future<Output = Result<T, MigrateError>>,
) -> Result<T, MigrateError> {
    match tokio::time::timeout(Duration::from_millis(u64::from(milliseconds)), operation).await {
        Ok(result) => result,
        Err(_) => {
            let error = sqlx::Error::Io(io::Error::new(
                io::ErrorKind::TimedOut,
                format!("database migration exceeded {milliseconds}ms; connection will be closed; retry during a quieter period"),
            ));
            Err(match version {
                Some(version) => MigrateError::ExecuteMigration(error, version),
                None => MigrateError::Execute(error),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::parse_timeout;

    #[test]
    fn migration_limits_cannot_be_disabled_or_overflow_postgres() {
        for invalid in ["0", "-1", "2147483648", "500ms", "", " "] {
            assert!(parse_timeout("TEST_TIMEOUT", invalid).is_err(), "{invalid}");
        }
        assert_eq!(parse_timeout("TEST_TIMEOUT", " 1000 ").unwrap(), 1_000);
        assert_eq!(
            parse_timeout("TEST_TIMEOUT", "2147483647").unwrap(),
            i32::MAX as u32
        );
    }
}

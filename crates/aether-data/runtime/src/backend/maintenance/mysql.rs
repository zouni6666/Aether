use crate::backend::MysqlBackend;
use crate::error::SqlResultExt;
use crate::{DataLayerError, DatabaseMaintenanceSummary};

use super::maintenance_identifier;

impl MysqlBackend {
    pub async fn run_table_maintenance(
        &self,
        table_names: &[&str],
    ) -> Result<DatabaseMaintenanceSummary, DataLayerError> {
        let mut summary = DatabaseMaintenanceSummary::default();
        for table_name in table_names {
            let table_name = maintenance_identifier(table_name)?;
            summary.attempted += 1;
            let statement = format!("ANALYZE TABLE `{table_name}`");
            if sqlx::query_as::<_, (String, String, String, String)>(&statement)
                .fetch_all(self.pool())
                .await
                .map_sql_err()
                .is_ok_and(|rows| mysql_analyze_succeeded(&rows))
            {
                summary.succeeded += 1;
            }
        }
        Ok(summary)
    }
}

fn mysql_analyze_succeeded(rows: &[(String, String, String, String)]) -> bool {
    !rows.is_empty()
        && rows.iter().any(|(_, _, message_type, message)| {
            message_type.eq_ignore_ascii_case("status") && message.eq_ignore_ascii_case("ok")
        })
        && rows
            .iter()
            .all(|(_, _, message_type, _)| !message_type.eq_ignore_ascii_case("error"))
}

#[cfg(test)]
mod tests {
    use super::mysql_analyze_succeeded;

    fn row(message_type: &str, message: &str) -> (String, String, String, String) {
        (
            "aether.usage".to_string(),
            "analyze".to_string(),
            message_type.to_string(),
            message.to_string(),
        )
    }

    #[test]
    fn analyze_requires_an_explicit_ok_status() {
        assert!(mysql_analyze_succeeded(&[row("status", "OK")]));
        assert!(!mysql_analyze_succeeded(&[]));
        assert!(!mysql_analyze_succeeded(&[row("note", "skipped")]));
    }

    #[test]
    fn analyze_rejects_error_rows_even_when_an_ok_row_is_present() {
        assert!(!mysql_analyze_succeeded(&[
            row("Error", "Table does not exist"),
            row("status", "OK"),
        ]));
    }
}

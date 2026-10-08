use super::SqlxBillingReadRepository;
use crate::error::SqlxResultExt;
use aether_data_contracts::{repository::billing::*, DataLayerError};
use sqlx::Row;

const SELECT_FIELDS: &str = "id, client_request_id, provider_id, provider_name, kind, amount::text AS amount, currency, (EXTRACT(EPOCH FROM paid_at)*1000)::bigint AS paid_ms, (EXTRACT(EPOCH FROM period_start)*1000)::bigint AS period_start_ms, (EXTRACT(EPOCH FROM period_end)*1000)::bigint AS period_end_ms, note, external_reference, created_by, (EXTRACT(EPOCH FROM created_at)*1000)::bigint AS created_ms, (EXTRACT(EPOCH FROM voided_at)*1000)::bigint AS voided_ms, voided_by";
fn row_record(row: &sqlx::postgres::PgRow) -> Result<ProviderExpenseRecord, DataLayerError> {
    Ok(ProviderExpenseRecord {
        id: row.try_get("id").map_postgres_err()?,
        entry: ProviderExpenseInput {
            client_request_id: row.try_get("client_request_id").map_postgres_err()?,
            provider_id: row.try_get("provider_id").map_postgres_err()?,
            provider_name: row.try_get("provider_name").map_postgres_err()?,
            kind: row.try_get("kind").map_postgres_err()?,
            amount: row.try_get("amount").map_postgres_err()?,
            currency: row.try_get("currency").map_postgres_err()?,
            paid_at_unix_ms: row.try_get::<i64, _>("paid_ms").map_postgres_err()? as u64,
            period_start_unix_ms: row
                .try_get::<Option<i64>, _>("period_start_ms")
                .map_postgres_err()?
                .map(|v| v as u64),
            period_end_unix_ms: row
                .try_get::<Option<i64>, _>("period_end_ms")
                .map_postgres_err()?
                .map(|v| v as u64),
            note: row.try_get("note").map_postgres_err()?,
            external_reference: row.try_get("external_reference").map_postgres_err()?,
            created_by: row.try_get("created_by").map_postgres_err()?,
        },
        created_at_unix_ms: row.try_get::<i64, _>("created_ms").map_postgres_err()? as u64,
        voided_at_unix_ms: row
            .try_get::<Option<i64>, _>("voided_ms")
            .map_postgres_err()?
            .map(|v| v as u64),
        voided_by: row.try_get("voided_by").map_postgres_err()?,
    })
}
fn datetime(ms: u64) -> chrono::DateTime<chrono::Utc> {
    chrono::DateTime::from_timestamp_millis(ms as i64).expect("validated expense timestamp")
}
impl SqlxBillingReadRepository {
    pub(super) async fn expense_page(
        &self,
        query: &ProviderExpenseQuery,
    ) -> Result<Option<ProviderExpensePage>, DataLayerError> {
        query.validate()?;
        let mut tx = self.pool.begin().await.map_postgres_err()?;
        sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ, READ ONLY")
            .execute(&mut *tx)
            .await
            .map_postgres_err()?;
        let from = datetime(query.from_unix_ms);
        let to = datetime(query.to_unix_ms);
        let totals = sqlx::query(r#"
            SELECT currency, sum(amount)::text AS amount,
                COALESCE(sum(amount) FILTER (WHERE kind='recharge'),0)::text AS recharge_amount,
                COALESCE(sum(amount) FILTER (WHERE kind='subscription'),0)::text AS subscription_amount,
                COALESCE(sum(amount) FILTER (WHERE kind='other'),0)::text AS other_amount,
                count(*)::bigint AS entry_count
            FROM provider_expenses
            WHERE voided_at IS NULL AND paid_at >= $1 AND paid_at < $2
            GROUP BY currency ORDER BY currency
        "#)
            .bind(from)
            .bind(to)
            .fetch_all(&mut *tx)
            .await.map_postgres_err()?
            .iter()
            .map(|r| Ok(ProviderExpenseTotals {
                currency: r.try_get("currency").map_postgres_err()?,
                amount: r.try_get("amount").map_postgres_err()?,
                recharge_amount: r.try_get("recharge_amount").map_postgres_err()?,
                subscription_amount: r.try_get("subscription_amount").map_postgres_err()?,
                other_amount: r.try_get("other_amount").map_postgres_err()?,
                entry_count: r.try_get::<i64,_>("entry_count").map_postgres_err()? as u64,
            }))
            .collect::<Result<Vec<_>,DataLayerError>>()?;
        let providers = sqlx::query(
            r#"
            SELECT provider_id,
                (array_agg(provider_name ORDER BY paid_at DESC,id DESC))[1] AS provider_name,
                currency, sum(amount)::text AS amount, count(*)::bigint AS entry_count
            FROM provider_expenses
            WHERE voided_at IS NULL AND paid_at >= $1 AND paid_at < $2
            GROUP BY provider_id,currency ORDER BY provider_id,currency
        "#,
        )
        .bind(from)
        .bind(to)
        .fetch_all(&mut *tx)
        .await
        .map_postgres_err()?
        .iter()
        .map(|r| {
            Ok(ProviderExpenseProviderTotal {
                provider_id: r.try_get("provider_id").map_postgres_err()?,
                provider_name: r.try_get("provider_name").map_postgres_err()?,
                currency: r.try_get("currency").map_postgres_err()?,
                amount: r.try_get("amount").map_postgres_err()?,
                entry_count: r.try_get::<i64, _>("entry_count").map_postgres_err()? as u64,
            })
        })
        .collect::<Result<Vec<_>, DataLayerError>>()?;
        let items = sqlx::query(&format!(
            r#"
            SELECT {SELECT_FIELDS} FROM provider_expenses
            WHERE voided_at IS NULL AND paid_at >= $1 AND paid_at < $2
            ORDER BY paid_at DESC,id DESC LIMIT $3 OFFSET $4
        "#
        ))
        .bind(from)
        .bind(to)
        .bind(i64::from(query.limit))
        .bind(query.offset as i64)
        .fetch_all(&mut *tx)
        .await
        .map_postgres_err()?
        .iter()
        .map(row_record)
        .collect::<Result<Vec<_>, _>>()?;
        tx.commit().await.map_postgres_err()?;
        Ok(Some(ProviderExpensePage {
            items,
            total: totals.iter().map(|r| r.entry_count).sum(),
            totals,
            providers,
        }))
    }
    pub(super) async fn insert_expense(
        &self,
        input: &ProviderExpenseInput,
    ) -> Result<AdminBillingMutationOutcome<ProviderExpenseRecord>, DataLayerError> {
        if let Err(detail) = input.validate() {
            return Ok(AdminBillingMutationOutcome::Invalid(detail));
        }
        let row = sqlx::query(&format!(
            r#"
            INSERT INTO provider_expenses (
                id,client_request_id,provider_id,provider_name,kind,amount,currency,
                paid_at,period_start,period_end,note,external_reference,created_by
            ) VALUES ($1,$2,$3,$4,$5,$6::text::numeric,$7,$8,$9,$10,$11,$12,$13)
            ON CONFLICT(client_request_id) DO NOTHING RETURNING {SELECT_FIELDS}
        "#
        ))
        .bind(uuid::Uuid::new_v4().to_string())
        .bind(&input.client_request_id)
        .bind(&input.provider_id)
        .bind(&input.provider_name)
        .bind(&input.kind)
        .bind(&input.amount)
        .bind(&input.currency)
        .bind(datetime(input.paid_at_unix_ms))
        .bind(input.period_start_unix_ms.map(datetime))
        .bind(input.period_end_unix_ms.map(datetime))
        .bind(&input.note)
        .bind(&input.external_reference)
        .bind(&input.created_by)
        .fetch_optional(&self.pool)
        .await
        .map_postgres_err()?;
        if let Some(row) = row {
            return Ok(AdminBillingMutationOutcome::Applied(row_record(&row)?));
        }
        let row = sqlx::query(&format!(
            "SELECT {SELECT_FIELDS} FROM provider_expenses WHERE client_request_id=$1"
        ))
        .bind(&input.client_request_id)
        .fetch_one(&self.pool)
        .await
        .map_postgres_err()?;
        let record = row_record(&row)?;
        Ok(if record.entry.same_request_as(input) {
            AdminBillingMutationOutcome::Applied(record)
        } else {
            AdminBillingMutationOutcome::Invalid(
                "client_request_id was already used for another expense".into(),
            )
        })
    }
    pub(super) async fn void_expense(
        &self,
        id: &str,
        operator: Option<&str>,
    ) -> Result<AdminBillingMutationOutcome<ProviderExpenseRecord>, DataLayerError> {
        let row = sqlx::query(&format!(
            r#"
            UPDATE provider_expenses
            SET voided_by = CASE WHEN voided_at IS NULL THEN $2 ELSE voided_by END,
                voided_at = COALESCE(voided_at,NOW())
            WHERE id=$1 RETURNING {SELECT_FIELDS}
        "#
        ))
        .bind(id)
        .bind(operator)
        .fetch_optional(&self.pool)
        .await
        .map_postgres_err()?;
        row.as_ref().map(row_record).transpose().map(|r| {
            r.map(AdminBillingMutationOutcome::Applied)
                .unwrap_or(AdminBillingMutationOutcome::NotFound)
        })
    }
}

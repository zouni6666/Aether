use super::SqlxUsageReadRepository;
use crate::error::SqlxResultExt;
use aether_data_contracts::{repository::usage::UsageAttributionSnapshot, DataLayerError};

impl SqlxUsageReadRepository {
    pub async fn correct_usage_attribution(
        &self,
        snapshot: &UsageAttributionSnapshot,
        expected_revision: u64,
    ) -> Result<bool, DataLayerError> {
        snapshot.validate()?;
        if snapshot.attribution_revision <= expected_revision {
            return Err(DataLayerError::InvalidInput(
                "attribution revision must increase".into(),
            ));
        }
        let result = sqlx::query(r#"UPDATE usage_attribution_snapshots SET actor_user_id=$2,
          credential_owner_id=$3,attribution_kind=$4,attribution_source=$5,
          record_kind=$6,parent_request_id=$7,schema_version=$8,attribution_revision=$9,recorded_at=NOW()
          WHERE request_id=$1 AND attribution_revision=$10
            AND ($2::text IS NULL OR EXISTS (SELECT 1 FROM users WHERE id=$2 AND NOT is_deleted))
            AND ($3::text IS NULL OR EXISTS (SELECT 1 FROM users WHERE id=$3 AND NOT is_deleted))"#)
            .bind(&snapshot.request_id).bind(&snapshot.actor_user_id).bind(&snapshot.credential_owner_id)
            .bind(&snapshot.attribution_kind).bind(&snapshot.attribution_source)
            .bind(&snapshot.record_kind).bind(&snapshot.parent_request_id).bind(snapshot.schema_version as i32)
            .bind(snapshot.attribution_revision as i64).bind(expected_revision as i64)
            .execute(&self.pool).await.map_postgres_err()?;
        Ok(result.rows_affected() == 1)
    }
}

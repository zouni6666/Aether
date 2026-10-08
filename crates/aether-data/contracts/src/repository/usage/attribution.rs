use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UsageAttributionSnapshot {
    pub request_id: String,
    pub actor_user_id: Option<String>,
    pub credential_owner_id: Option<String>,
    pub attribution_kind: String,
    pub attribution_source: String,
    pub record_kind: String,
    pub parent_request_id: Option<String>,
    pub schema_version: u32,
    pub attribution_revision: u64,
}

impl UsageAttributionSnapshot {
    pub fn validate(&self) -> Result<(), crate::DataLayerError> {
        if self.request_id.is_empty()
            || self.attribution_revision == 0
            || self.schema_version != 1
            || !matches!(
                self.attribution_kind.as_str(),
                "employee" | "standalone" | "unknown"
            )
            || !matches!(
                self.attribution_source.as_str(),
                "user_account" | "standalone_key" | "unknown"
            )
            || (self.attribution_kind == "employee") != self.actor_user_id.is_some()
            || (self.attribution_kind == "employee"
                && (self.actor_user_id != self.credential_owner_id
                    || self.attribution_source != "user_account"))
            || (self.attribution_kind == "standalone"
                && (self.credential_owner_id.is_none()
                    || self.attribution_source != "standalone_key"))
            || (self.attribution_kind == "unknown" && self.attribution_source != "unknown")
        {
            return Err(crate::DataLayerError::InvalidInput(
                "invalid usage attribution snapshot".into(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::UsageAttributionSnapshot;

    #[test]
    fn account_attribution_requires_matching_owner_and_key_classification() {
        let mut snapshot = UsageAttributionSnapshot {
            request_id: "request".into(),
            actor_user_id: Some("member".into()),
            credential_owner_id: Some("member".into()),
            attribution_kind: "employee".into(),
            attribution_source: "user_account".into(),
            record_kind: "request".into(),
            parent_request_id: None,
            schema_version: 1,
            attribution_revision: 2,
        };
        assert!(snapshot.validate().is_ok());
        snapshot.actor_user_id = Some("another-member".into());
        assert!(snapshot.validate().is_err());
        snapshot.actor_user_id = None;
        snapshot.attribution_kind = "standalone".into();
        snapshot.attribution_source = "standalone_key".into();
        assert!(snapshot.validate().is_ok());
        snapshot.credential_owner_id = None;
        assert!(snapshot.validate().is_err());
        snapshot.attribution_kind = "unknown".into();
        snapshot.attribution_source = "unknown".into();
        assert!(snapshot.validate().is_ok());
    }
}

use aether_data_contracts::repository::usage::HealthObservationObjectKind;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub(super) const PUBLICATION_KEY: &str = "health_publication_v1";

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct HealthPublication {
    pub enabled: bool,
    pub objects: Vec<PublishedHealthObject>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PublishedHealthObject {
    pub public_id: String,
    pub kind: HealthObservationObjectKind,
    pub value: String,
    pub display_name: String,
}

impl HealthPublication {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.objects.len() > 200 {
            return Err("At most 200 public health objects may be published");
        }
        let mut ids = BTreeSet::new();
        let mut values = BTreeSet::new();
        for object in &self.objects {
            if object.public_id.is_empty()
                || object.public_id.len() > 80
                || !object
                    .public_id
                    .bytes()
                    .all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_')
            {
                return Err(
                    "Public IDs must contain 1 to 80 letters, digits, hyphens or underscores",
                );
            }
            if object.kind == HealthObservationObjectKind::Provider {
                return Err("Internal providers cannot be published as public health objects");
            }
            if object.value.trim().is_empty()
                || object.value.len() > 256
                || object.display_name.trim().is_empty()
                || object.display_name.len() > 120
            {
                return Err("Every public object requires a bounded source value and display name");
            }
            if !ids.insert(object.public_id.clone())
                || !values.insert(format!("{:?}:{}", object.kind, object.value))
            {
                return Err("Public IDs and health source objects must be unique");
            }
        }
        Ok(())
    }
}

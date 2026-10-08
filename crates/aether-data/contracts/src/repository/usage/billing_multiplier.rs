use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::DataLayerError;

use super::ROUTING_GROUP_BILLING_MULTIPLIER_METADATA_KEY;

pub const BILLING_MULTIPLIER_SNAPSHOT_METADATA_KEY: &str = "billing_multiplier_snapshot";

/// Immutable customer pricing factors. Provider Key rates belong to upstream cost,
/// not this snapshot. Add future factors (for example `user_group`) at admission.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BillingMultiplierSnapshot {
    version: u32,
    factors: BTreeMap<String, f64>,
    multiplier: f64,
}

impl BillingMultiplierSnapshot {
    pub fn from_factors(factors: BTreeMap<String, f64>) -> Result<Self, DataLayerError> {
        if factors.len() > 16
            || factors.iter().any(|(name, value)| {
                name.is_empty()
                    || name.len() > 64
                    || !name
                        .bytes()
                        .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
                    || !value.is_finite()
                    || *value < 0.0
            })
        {
            return Err(invalid_snapshot());
        }
        let multiplier = if factors.values().any(|value| *value == 0.0) {
            0.0
        } else {
            factors.values().product::<f64>()
        };
        if !multiplier.is_finite() {
            return Err(invalid_snapshot());
        }
        Ok(Self {
            version: 1,
            factors,
            multiplier,
        })
    }

    pub fn validate(&self) -> Result<(), DataLayerError> {
        let expected = Self::from_factors(self.factors.clone())?;
        if self.version != 1 || self.multiplier != expected.multiplier {
            return Err(invalid_snapshot());
        }
        Ok(())
    }

    pub fn multiplier(&self) -> f64 {
        self.multiplier
    }

    pub fn cost(&self, base_cost: f64) -> Result<f64, DataLayerError> {
        self.validate()?;
        let cost = base_cost * self.multiplier;
        if !base_cost.is_finite() || base_cost < 0.0 || !cost.is_finite() {
            return Err(DataLayerError::InvalidInput(
                "customer billing cost must be finite and non-negative".to_string(),
            ));
        }
        // Match wallet storage and usage-policy cost units (eight decimals).
        // Scaling a finite large amount must not introduce infinity by itself.
        let scaled = cost * 100_000_000.0;
        Ok(if scaled.is_finite() {
            scaled.round() / 100_000_000.0
        } else {
            cost
        })
    }
}

fn invalid_snapshot() -> DataLayerError {
    DataLayerError::InvalidInput("invalid billing multiplier snapshot".to_string())
}

/// None preserves legacy charging. A malformed captured snapshot is an error,
/// never an instruction to silently charge a different rate.
pub fn billing_multiplier_snapshot(
    metadata: Option<&Value>,
) -> Result<Option<BillingMultiplierSnapshot>, DataLayerError> {
    let Some(metadata) = metadata.and_then(Value::as_object) else {
        return Ok(None);
    };
    if let Some(value) = metadata.get(BILLING_MULTIPLIER_SNAPSHOT_METADATA_KEY) {
        let snapshot: BillingMultiplierSnapshot =
            serde_json::from_value(value.clone()).map_err(|_| invalid_snapshot())?;
        snapshot.validate()?;
        return Ok(Some(snapshot));
    }
    if let Some(value) = metadata.get(ROUTING_GROUP_BILLING_MULTIPLIER_METADATA_KEY) {
        let multiplier = value.as_f64().ok_or_else(invalid_snapshot)?;
        return BillingMultiplierSnapshot::from_factors(BTreeMap::from([(
            "routing_group".to_string(),
            multiplier,
        )]))
        .map(Some);
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn composes_customer_factors_without_provider_cost_and_freezes_them() {
        let snapshot = BillingMultiplierSnapshot::from_factors(BTreeMap::from([
            ("routing_group".to_string(), 2.0),
            ("user_group".to_string(), 0.75),
        ]))
        .unwrap();
        assert_eq!(snapshot.multiplier(), 1.5);
        assert_eq!(snapshot.cost(10.0).unwrap(), 15.0);
        assert_eq!(snapshot.cost(0.123456789).unwrap(), 0.18518518);
        let metadata = json!({"billing_multiplier_snapshot": snapshot, "routing_group_billing_multiplier": 99, "rate_multiplier": 0.1});
        assert_eq!(
            billing_multiplier_snapshot(Some(&metadata)).unwrap(),
            Some(snapshot)
        );
        assert_eq!(billing_multiplier_snapshot(None).unwrap(), None);
    }

    #[test]
    fn rejects_corrupt_overflowing_snapshots_and_accepts_zero_rates() {
        for factors in [
            BTreeMap::from([("routing_group".into(), -1.0)]),
            BTreeMap::from([("routing_group".into(), f64::INFINITY)]),
            BTreeMap::from([
                ("routing_group".into(), f64::MAX),
                ("user_group".into(), 2.0),
            ]),
        ] {
            assert!(BillingMultiplierSnapshot::from_factors(factors).is_err());
        }
        let zero = BillingMultiplierSnapshot::from_factors(BTreeMap::from([
            ("routing_group".into(), 0.0),
            ("user_group".into(), 2.0),
        ]))
        .unwrap();
        assert_eq!(zero.cost(10.0).unwrap(), 0.0);
        for invalid in [
            Value::Null,
            json!({"version": 2, "factors": {}, "multiplier": 1}),
            json!({"version": 1, "factors": {"routing_group": 2}, "multiplier": 1}),
        ] {
            assert!(billing_multiplier_snapshot(Some(
                &json!({"billing_multiplier_snapshot": invalid})
            ))
            .is_err());
        }
        let doubled = BillingMultiplierSnapshot::from_factors(BTreeMap::from([(
            "routing_group".into(),
            2.0,
        )]))
        .unwrap();
        assert!(doubled.cost(f64::MAX).is_err());
    }
}

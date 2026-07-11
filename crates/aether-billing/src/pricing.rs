use std::collections::BTreeSet;

use aether_data_contracts::repository::{
    billing::StoredBillingModelContext, usage::normalize_provider_service_tier,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BillingPricingSource {
    ProviderOverride,
    GlobalDefault,
}

impl BillingPricingSource {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ProviderOverride => "provider_override",
            Self::GlobalDefault => "global_default",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BillingPricingResolution {
    pub requested_processing_tier: Option<String>,
    pub actual_processing_tier: Option<String>,
    pub billing_processing_tier: Option<String>,
    pub tiered_pricing: Option<Value>,
    pub tiered_pricing_source: Option<BillingPricingSource>,
    pub price_per_request: Option<f64>,
    pub price_per_request_source: Option<BillingPricingSource>,
}

impl BillingPricingResolution {
    pub fn requires_actual_processing_tier(&self) -> bool {
        self.billing_processing_tier.is_none()
    }

    pub fn pricing_source(&self) -> &'static str {
        match (self.tiered_pricing_source, self.price_per_request_source) {
            (Some(tiered), Some(request)) if tiered != request => "mixed",
            (Some(source), _) | (_, Some(source)) => source.as_str(),
            (None, None) => "unpriced",
        }
    }

    pub fn bills_standard_processing_tier(&self) -> bool {
        self.billing_processing_tier
            .as_deref()
            .is_some_and(processing_tier_is_standard)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BillingModelPricingSnapshot {
    pub provider_id: String,
    pub provider_billing_type: Option<String>,
    pub provider_api_key_id: Option<String>,
    pub provider_api_key_rate_multipliers: Option<Value>,
    pub provider_api_key_cache_ttl_minutes: Option<i64>,
    pub global_model_id: String,
    pub global_model_name: String,
    pub global_model_config: Option<Value>,
    pub default_price_per_request: Option<f64>,
    pub default_tiered_pricing: Option<Value>,
    pub model_id: Option<String>,
    pub model_provider_model_name: Option<String>,
    pub model_config: Option<Value>,
    pub model_price_per_request: Option<f64>,
    pub model_tiered_pricing: Option<Value>,
}

impl BillingModelPricingSnapshot {
    pub fn resolve_pricing(
        &self,
        requested_processing_tier: Option<&str>,
        actual_processing_tier: Option<&str>,
    ) -> BillingPricingResolution {
        let requested_processing_tier = normalize_processing_tier(requested_processing_tier);
        let actual_processing_tier = normalize_processing_tier(actual_processing_tier);
        let billing_processing_tier = actual_processing_tier
            .as_deref()
            .map(canonical_processing_tier)
            .or_else(|| {
                requested_processing_tier.as_deref().map_or(
                    Some("standard".to_string()),
                    |requested| {
                        processing_tier_is_standard(requested).then(|| "standard".to_string())
                    },
                )
            });

        let (tiered_pricing, tiered_pricing_source) = billing_processing_tier
            .as_deref()
            .and_then(|tier| self.resolve_tiered_pricing(tier))
            .map_or((None, None), |(pricing, source)| {
                (Some(pricing.clone()), Some(source))
            });
        let (price_per_request, price_per_request_source) = self
            .resolve_price_per_request()
            .map_or((None, None), |(price, source)| (Some(price), Some(source)));

        BillingPricingResolution {
            requested_processing_tier,
            actual_processing_tier,
            billing_processing_tier,
            tiered_pricing,
            tiered_pricing_source,
            price_per_request,
            price_per_request_source,
        }
    }

    pub fn resolve_authorization_pricing_candidates(
        &self,
        requested_processing_tier: Option<&str>,
    ) -> Option<Vec<BillingPricingResolution>> {
        let requested_processing_tier = normalize_processing_tier(requested_processing_tier);
        let requested_billing_tier = requested_processing_tier
            .as_deref()
            .map(canonical_processing_tier)
            .unwrap_or_else(|| "standard".to_string());
        let requested_resolution = self.authorization_pricing_for_tier(
            requested_processing_tier.clone(),
            Some(requested_billing_tier.clone()),
        );
        if !processing_tier_is_standard(&requested_billing_tier)
            && requested_resolution.tiered_pricing.is_none()
        {
            return None;
        }

        let mut billing_tiers = BTreeSet::from(["standard".to_string(), requested_billing_tier]);
        for pricing in [
            self.model_tiered_pricing.as_ref(),
            self.default_tiered_pricing.as_ref(),
        ]
        .into_iter()
        .flatten()
        {
            let Some(processing_tiers) = pricing.get("processing_tiers").and_then(Value::as_object)
            else {
                continue;
            };
            billing_tiers.extend(processing_tiers.keys().filter_map(|tier| {
                normalize_processing_tier(Some(tier)).map(|tier| canonical_processing_tier(&tier))
            }));
        }

        let candidates = billing_tiers
            .into_iter()
            .filter_map(|billing_tier| {
                let resolution = self.authorization_pricing_for_tier(
                    requested_processing_tier.clone(),
                    Some(billing_tier),
                );
                (resolution.bills_standard_processing_tier() || resolution.tiered_pricing.is_some())
                    .then_some(resolution)
            })
            .collect::<Vec<_>>();
        (!candidates.is_empty()).then_some(candidates)
    }

    fn authorization_pricing_for_tier(
        &self,
        requested_processing_tier: Option<String>,
        billing_processing_tier: Option<String>,
    ) -> BillingPricingResolution {
        let (tiered_pricing, tiered_pricing_source) = billing_processing_tier
            .as_deref()
            .and_then(|tier| self.resolve_tiered_pricing(tier))
            .map_or((None, None), |(pricing, source)| {
                (Some(pricing.clone()), Some(source))
            });
        let (price_per_request, price_per_request_source) = self
            .resolve_price_per_request()
            .map_or((None, None), |(price, source)| (Some(price), Some(source)));

        BillingPricingResolution {
            requested_processing_tier,
            actual_processing_tier: None,
            billing_processing_tier,
            tiered_pricing,
            tiered_pricing_source,
            price_per_request,
            price_per_request_source,
        }
    }

    fn resolve_tiered_pricing(
        &self,
        processing_tier: &str,
    ) -> Option<(&Value, BillingPricingSource)> {
        if processing_tier_is_standard(processing_tier) {
            return self
                .model_tiered_pricing
                .as_ref()
                .filter(|value| has_pricing_data(value))
                .map(|value| (value, BillingPricingSource::ProviderOverride))
                .or_else(|| {
                    self.default_tiered_pricing
                        .as_ref()
                        .filter(|value| has_pricing_data(value))
                        .map(|value| (value, BillingPricingSource::GlobalDefault))
                });
        }

        self.model_tiered_pricing
            .as_ref()
            .and_then(|pricing| processing_tier_overlay(pricing, processing_tier))
            .map(|value| (value, BillingPricingSource::ProviderOverride))
            .or_else(|| {
                self.default_tiered_pricing
                    .as_ref()
                    .and_then(|pricing| processing_tier_overlay(pricing, processing_tier))
                    .map(|value| (value, BillingPricingSource::GlobalDefault))
            })
    }

    fn resolve_price_per_request(&self) -> Option<(f64, BillingPricingSource)> {
        self.model_price_per_request
            .map(|price| (price, BillingPricingSource::ProviderOverride))
            .or_else(|| {
                self.default_price_per_request
                    .map(|price| (price, BillingPricingSource::GlobalDefault))
            })
    }

    pub fn is_free_tier(&self) -> bool {
        self.provider_billing_type
            .as_deref()
            .map(|value| value.eq_ignore_ascii_case("free_tier"))
            .unwrap_or(false)
    }

    pub fn rate_multiplier_for_api_format(&self, api_format: Option<&str>) -> f64 {
        let Some(api_format) = api_format.map(str::trim).filter(|value| !value.is_empty()) else {
            return 1.0;
        };
        let normalized = api_format.to_ascii_lowercase();
        let Some(mapping) = self
            .provider_api_key_rate_multipliers
            .as_ref()
            .and_then(Value::as_object)
        else {
            return 1.0;
        };
        mapping
            .get(&normalized)
            .and_then(|value| value.as_f64())
            .unwrap_or(1.0)
    }
}

impl From<&StoredBillingModelContext> for BillingModelPricingSnapshot {
    fn from(context: &StoredBillingModelContext) -> Self {
        Self {
            provider_id: context.provider_id.clone(),
            provider_billing_type: context.provider_billing_type.clone(),
            provider_api_key_id: context.provider_api_key_id.clone(),
            provider_api_key_rate_multipliers: context.provider_api_key_rate_multipliers.clone(),
            provider_api_key_cache_ttl_minutes: context.provider_api_key_cache_ttl_minutes,
            global_model_id: context.global_model_id.clone(),
            global_model_name: context.global_model_name.clone(),
            global_model_config: context.global_model_config.clone(),
            default_price_per_request: context.default_price_per_request,
            default_tiered_pricing: context.default_tiered_pricing.clone(),
            model_id: context.model_id.clone(),
            model_provider_model_name: context.model_provider_model_name.clone(),
            model_config: context.model_config.clone(),
            model_price_per_request: context.model_price_per_request,
            model_tiered_pricing: context.model_tiered_pricing.clone(),
        }
    }
}

impl From<StoredBillingModelContext> for BillingModelPricingSnapshot {
    fn from(context: StoredBillingModelContext) -> Self {
        Self {
            provider_id: context.provider_id,
            provider_billing_type: context.provider_billing_type,
            provider_api_key_id: context.provider_api_key_id,
            provider_api_key_rate_multipliers: context.provider_api_key_rate_multipliers,
            provider_api_key_cache_ttl_minutes: context.provider_api_key_cache_ttl_minutes,
            global_model_id: context.global_model_id,
            global_model_name: context.global_model_name,
            global_model_config: context.global_model_config,
            default_price_per_request: context.default_price_per_request,
            default_tiered_pricing: context.default_tiered_pricing,
            model_id: context.model_id,
            model_provider_model_name: context.model_provider_model_name,
            model_config: context.model_config,
            model_price_per_request: context.model_price_per_request,
            model_tiered_pricing: context.model_tiered_pricing,
        }
    }
}

fn normalize_processing_tier(value: Option<&str>) -> Option<String> {
    value.and_then(normalize_provider_service_tier)
}

fn canonical_processing_tier(value: &str) -> String {
    if processing_tier_is_standard(value) {
        "standard".to_string()
    } else {
        value.to_string()
    }
}

fn processing_tier_is_standard(value: &str) -> bool {
    matches!(value, "auto" | "default" | "standard")
}

fn processing_tier_overlay<'a>(pricing: &'a Value, tier: &str) -> Option<&'a Value> {
    pricing
        .get("processing_tiers")
        .and_then(Value::as_object)
        .and_then(|tiers| tiers.get(tier))
        .filter(|value| has_pricing_data(value))
}

fn has_pricing_data(value: &Value) -> bool {
    value
        .get("tiers")
        .and_then(Value::as_array)
        .is_some_and(|tiers| !tiers.is_empty())
        || value
            .get("image_output_price_default")
            .and_then(Value::as_f64)
            .is_some()
        || [
            "image_output_prices",
            "image_output_price_ranges",
            "image_output_price_per_image",
            "image_output_price_matrix",
            "image_prices",
        ]
        .iter()
        .any(|key| value.get(key).is_some_and(value_has_entries))
}

fn value_has_entries(value: &Value) -> bool {
    value.as_object().is_some_and(|object| !object.is_empty())
        || value.as_array().is_some_and(|items| !items.is_empty())
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{BillingModelPricingSnapshot, BillingPricingSource};

    fn snapshot(
        model_tiered_pricing: Option<serde_json::Value>,
        default_tiered_pricing: Option<serde_json::Value>,
    ) -> BillingModelPricingSnapshot {
        BillingModelPricingSnapshot {
            provider_id: "provider-1".to_string(),
            provider_billing_type: None,
            provider_api_key_id: None,
            provider_api_key_rate_multipliers: None,
            provider_api_key_cache_ttl_minutes: None,
            global_model_id: "global-model-1".to_string(),
            global_model_name: "gpt-5".to_string(),
            global_model_config: None,
            default_price_per_request: None,
            default_tiered_pricing,
            model_id: Some("model-1".to_string()),
            model_provider_model_name: Some("gpt-5-upstream".to_string()),
            model_config: None,
            model_price_per_request: None,
            model_tiered_pricing,
        }
    }

    #[test]
    fn empty_provider_tiered_pricing_inherits_global_default() {
        let default_pricing =
            json!({"tiers":[{"up_to":null,"input_price_per_1m":3.0,"output_price_per_1m":15.0}]});
        let pricing = snapshot(Some(json!({})), Some(default_pricing.clone()));

        let resolution = pricing.resolve_pricing(None, None);
        assert_eq!(resolution.tiered_pricing, Some(default_pricing.clone()));
        assert_eq!(
            resolution.tiered_pricing_source,
            Some(BillingPricingSource::GlobalDefault)
        );

        let pricing = snapshot(Some(json!({"tiers": []})), Some(default_pricing.clone()));

        let resolution = pricing.resolve_pricing(None, None);
        assert_eq!(resolution.tiered_pricing, Some(default_pricing));
        assert_eq!(
            resolution.tiered_pricing_source,
            Some(BillingPricingSource::GlobalDefault)
        );
    }

    #[test]
    fn populated_provider_tiered_pricing_overrides_global_default() {
        let provider_pricing =
            json!({"tiers":[{"up_to":null,"input_price_per_1m":1.0,"output_price_per_1m":2.0}]});
        let default_pricing =
            json!({"tiers":[{"up_to":null,"input_price_per_1m":3.0,"output_price_per_1m":15.0}]});
        let pricing = snapshot(Some(provider_pricing.clone()), Some(default_pricing));

        let resolution = pricing.resolve_pricing(None, None);
        assert_eq!(resolution.tiered_pricing, Some(provider_pricing));
        assert_eq!(
            resolution.tiered_pricing_source,
            Some(BillingPricingSource::ProviderOverride)
        );
    }

    #[test]
    fn explicit_nonstandard_request_requires_actual_tier() {
        let pricing = snapshot(
            None,
            Some(json!({
                "tiers": [{"up_to": null, "input_price_per_1m": 3.0}],
                "processing_tiers": {
                    "priority": {"tiers": [{"up_to": 272000, "input_price_per_1m": 6.0}]}
                }
            })),
        );

        let resolution = pricing.resolve_pricing(Some("Priority"), None);

        assert!(resolution.requires_actual_processing_tier());
        assert_eq!(
            resolution.requested_processing_tier.as_deref(),
            Some("priority")
        );
        assert_eq!(resolution.billing_processing_tier, None);
        assert_eq!(resolution.tiered_pricing, None);
    }

    #[test]
    fn actual_tier_selects_exact_catalog_and_source() {
        let pricing = snapshot(
            Some(json!({
                "processing_tiers": {
                    "priority": {"tiers": [{"up_to": 272000, "input_price_per_1m": 9.0}]}
                }
            })),
            Some(json!({
                "tiers": [{"up_to": null, "input_price_per_1m": 3.0}],
                "processing_tiers": {
                    "flex": {"tiers": [{"up_to": null, "input_price_per_1m": 1.5}]}
                }
            })),
        );

        let flex = pricing.resolve_pricing(Some("priority"), Some("flex"));
        assert_eq!(flex.billing_processing_tier.as_deref(), Some("flex"));
        assert_eq!(
            flex.tiered_pricing_source,
            Some(BillingPricingSource::GlobalDefault)
        );
        assert_eq!(
            flex.tiered_pricing
                .as_ref()
                .and_then(|value| value.pointer("/tiers/0/input_price_per_1m"))
                .and_then(serde_json::Value::as_f64),
            Some(1.5)
        );

        let standard = pricing.resolve_pricing(Some("priority"), Some("Default"));
        assert_eq!(standard.actual_processing_tier.as_deref(), Some("default"));
        assert_eq!(
            standard.billing_processing_tier.as_deref(),
            Some("standard")
        );
        assert_eq!(
            standard.tiered_pricing_source,
            Some(BillingPricingSource::GlobalDefault)
        );
    }

    #[test]
    fn tiered_and_fixed_price_sources_are_recorded_independently() {
        let mut pricing = snapshot(
            None,
            Some(json!({"tiers": [{"up_to": null, "input_price_per_1m": 3.0}]})),
        );
        pricing.model_price_per_request = Some(0.02);

        let resolution = pricing.resolve_pricing(None, None);

        assert_eq!(
            resolution.tiered_pricing_source,
            Some(BillingPricingSource::GlobalDefault)
        );
        assert_eq!(
            resolution.price_per_request_source,
            Some(BillingPricingSource::ProviderOverride)
        );
        assert_eq!(resolution.pricing_source(), "mixed");
    }

    #[test]
    fn authorization_candidates_include_requested_catalog_without_inventing_actual_tier() {
        let pricing = snapshot(
            None,
            Some(json!({
                "tiers": [{"up_to": null, "input_price_per_1m": 3.0}],
                "processing_tiers": {
                    "priority": {"tiers": [{"up_to": 272000, "input_price_per_1m": 6.0}]}
                }
            })),
        );

        let candidates = pricing
            .resolve_authorization_pricing_candidates(Some("Priority"))
            .expect("authorization catalogs should resolve");
        let resolution = candidates
            .iter()
            .find(|resolution| resolution.billing_processing_tier.as_deref() == Some("priority"))
            .expect("priority catalog should be included");

        assert_eq!(
            resolution.requested_processing_tier.as_deref(),
            Some("priority")
        );
        assert_eq!(resolution.actual_processing_tier, None);
        assert_eq!(
            resolution.billing_processing_tier.as_deref(),
            Some("priority")
        );
        assert_eq!(
            resolution
                .tiered_pricing
                .as_ref()
                .and_then(|value| value.pointer("/tiers/0/input_price_per_1m"))
                .and_then(serde_json::Value::as_f64),
            Some(6.0)
        );
        assert!(candidates.iter().any(|resolution| {
            resolution.billing_processing_tier.as_deref() == Some("standard")
        }));
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BillingUsageInput {
    pub task_type: String,
    pub api_format: Option<String>,
    #[serde(default)]
    pub requested_processing_tier: Option<String>,
    #[serde(default)]
    pub actual_processing_tier: Option<String>,
    pub request_count: i64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cache_creation_tokens: i64,
    pub cache_creation_ephemeral_5m_tokens: i64,
    pub cache_creation_ephemeral_1h_tokens: i64,
    pub cache_read_tokens: i64,
    pub image_count: i64,
    pub image_size: Option<String>,
    pub image_quality: Option<String>,
    pub image_output_format: Option<String>,
    pub cache_ttl_minutes: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BillingAuthorizationEstimateInput {
    pub task_type: String,
    pub api_format: Option<String>,
    pub requested_processing_tier: Option<String>,
    pub input_tokens: i64,
    pub max_output_tokens: Option<i64>,
}

impl BillingAuthorizationEstimateInput {
    pub fn new(task_type: impl Into<String>, input_tokens: i64) -> Self {
        Self {
            task_type: task_type.into(),
            api_format: None,
            requested_processing_tier: None,
            input_tokens: input_tokens.max(0),
            max_output_tokens: None,
        }
    }
}

impl BillingUsageInput {
    pub fn new(task_type: impl Into<String>) -> Self {
        Self {
            task_type: task_type.into(),
            api_format: None,
            requested_processing_tier: None,
            actual_processing_tier: None,
            request_count: 1,
            input_tokens: 0,
            output_tokens: 0,
            cache_creation_tokens: 0,
            cache_creation_ephemeral_5m_tokens: 0,
            cache_creation_ephemeral_1h_tokens: 0,
            cache_read_tokens: 0,
            image_count: 0,
            image_size: None,
            image_quality: None,
            image_output_format: None,
            cache_ttl_minutes: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BillingComputation {
    pub cost_result: crate::CostResult,
    pub actual_total_cost: f64,
    pub rate_multiplier: f64,
    pub is_free_tier: bool,
    pub pricing_resolution: BillingPricingResolution,
}

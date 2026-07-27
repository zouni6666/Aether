use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::snapshot::GatewayProviderTransportSnapshot;

/// Provider-side compatibility applied to otherwise same-format Anthropic requests.
///
/// Native Anthropic endpoints should remain transparent. The legacy Claude Code
/// profile is opt-in, except for the existing `claude_code` provider type where it
/// remains the backwards-compatible default. Endpoint config takes precedence over
/// provider config. The canonical field is `anthropic.compatibility_profile`;
/// explicitly Anthropic-namespaced legacy spellings remain accepted.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AnthropicCompatibilityProfile {
    #[default]
    #[serde(
        alias = "native",
        alias = "anthropic",
        alias = "none",
        alias = "transparent"
    )]
    NativeTransparent,
    #[serde(
        alias = "claude_code",
        alias = "claude-code",
        alias = "legacy",
        alias = "same_format_compat"
    )]
    ClaudeCodeLegacy,
}

impl AnthropicCompatibilityProfile {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NativeTransparent => "native_transparent",
            Self::ClaudeCodeLegacy => "claude_code_legacy",
        }
    }

    pub const fn uses_claude_code_compatibility(self) -> bool {
        matches!(self, Self::ClaudeCodeLegacy)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("unknown Anthropic compatibility profile")]
pub struct AnthropicCompatibilityProfileConfigError;

/// Validate the optional Anthropic compatibility fields in a provider or
/// endpoint config without resolving provider defaults or mutating config.
pub fn validate_anthropic_compatibility_profile_config(
    config: Option<&Value>,
) -> Result<(), AnthropicCompatibilityProfileConfigError> {
    match profile_from_config(config) {
        ConfiguredProfile::Absent | ConfiguredProfile::Valid(_) => Ok(()),
        ConfiguredProfile::Invalid => Err(AnthropicCompatibilityProfileConfigError),
    }
}

pub fn resolve_anthropic_compatibility_profile(
    transport: &GatewayProviderTransportSnapshot,
    provider_api_format: &str,
) -> AnthropicCompatibilityProfile {
    if !aether_ai_formats::api_format_alias_matches(provider_api_format, "claude:messages") {
        return AnthropicCompatibilityProfile::NativeTransparent;
    }

    for (scope, resolution) in [
        (
            "endpoint",
            profile_from_config(transport.endpoint.config.as_ref()),
        ),
        (
            "provider",
            profile_from_config(transport.provider.config.as_ref()),
        ),
    ] {
        match resolution {
            ConfiguredProfile::Valid(profile) => return profile,
            ConfiguredProfile::Invalid => {
                tracing::warn!(
                    event_name = "anthropic_compatibility_profile_invalid",
                    log_type = "ops",
                    provider_id = %transport.provider.id,
                    endpoint_id = %transport.endpoint.id,
                    config_scope = scope,
                    "invalid Anthropic compatibility profile; using native transparent behavior"
                );
                return AnthropicCompatibilityProfile::NativeTransparent;
            }
            ConfiguredProfile::Absent => {}
        }
    }

    if transport
        .provider
        .provider_type
        .trim()
        .eq_ignore_ascii_case("claude_code")
    {
        AnthropicCompatibilityProfile::ClaudeCodeLegacy
    } else {
        AnthropicCompatibilityProfile::NativeTransparent
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConfiguredProfile {
    Absent,
    Valid(AnthropicCompatibilityProfile),
    Invalid,
}

fn profile_from_config(config: Option<&Value>) -> ConfiguredProfile {
    let Some(config) = config.and_then(Value::as_object) else {
        return ConfiguredProfile::Absent;
    };

    for (container, fields) in [
        (
            config.get("anthropic").and_then(Value::as_object),
            &["compatibility_profile", "compatibilityProfile", "profile"][..],
        ),
        (
            config
                .get("anthropic_compatibility")
                .and_then(Value::as_object),
            &["profile", "compatibility_profile", "compatibilityProfile"][..],
        ),
        (
            config
                .get("anthropicCompatibility")
                .and_then(Value::as_object),
            &["profile", "compatibilityProfile", "compatibility_profile"][..],
        ),
    ] {
        let Some(container) = container else {
            continue;
        };
        for &field in fields {
            if let Some(profile) = container.get(field).and_then(parse_profile_value) {
                return ConfiguredProfile::Valid(profile);
            }
            if container.contains_key(field) {
                return ConfiguredProfile::Invalid;
            }
        }
    }

    for field in [
        "anthropic_compatibility_profile",
        "anthropicCompatibilityProfile",
    ] {
        if let Some(value) = config.get(field) {
            return parse_profile_value(value)
                .map(ConfiguredProfile::Valid)
                .unwrap_or(ConfiguredProfile::Invalid);
        }
    }

    let Some(claude_code_advanced) = config
        .get("claude_code_advanced")
        .and_then(Value::as_object)
    else {
        return ConfiguredProfile::Absent;
    };
    for field in ["compatibility_profile", "compatibilityProfile"] {
        if let Some(value) = claude_code_advanced.get(field) {
            return parse_profile_value(value)
                .map(ConfiguredProfile::Valid)
                .unwrap_or(ConfiguredProfile::Invalid);
        }
    }
    ConfiguredProfile::Absent
}

fn parse_profile_value(value: &Value) -> Option<AnthropicCompatibilityProfile> {
    if let Some(object) = value.as_object() {
        return ["kind", "name", "profile"]
            .into_iter()
            .find_map(|field| object.get(field).and_then(parse_profile_value));
    }
    serde_json::from_value(value.clone()).ok()
}

#[cfg(test)]
mod tests {
    use super::{
        resolve_anthropic_compatibility_profile, validate_anthropic_compatibility_profile_config,
        AnthropicCompatibilityProfile,
    };
    use crate::snapshot::{
        GatewayProviderTransportEndpoint, GatewayProviderTransportKey,
        GatewayProviderTransportProvider, GatewayProviderTransportSnapshot,
    };
    use serde_json::json;

    fn sample_transport(provider_type: &str) -> GatewayProviderTransportSnapshot {
        GatewayProviderTransportSnapshot {
            provider: GatewayProviderTransportProvider {
                id: "provider-1".to_string(),
                name: "provider".to_string(),
                provider_type: provider_type.to_string(),
                website: None,
                is_active: true,
                keep_priority_on_conversion: false,
                enable_format_conversion: false,
                concurrent_limit: None,
                max_retries: None,
                proxy: None,
                request_timeout_secs: None,
                stream_first_byte_timeout_secs: None,
                config: None,
            },
            endpoint: GatewayProviderTransportEndpoint {
                id: "endpoint-1".to_string(),
                provider_id: "provider-1".to_string(),
                api_format: "claude:messages".to_string(),
                api_family: Some("claude".to_string()),
                endpoint_kind: Some("chat".to_string()),
                is_active: true,
                base_url: "https://api.anthropic.com".to_string(),
                header_rules: None,
                body_rules: None,
                max_retries: None,
                custom_path: None,
                config: None,
                format_acceptance_config: None,
                proxy: None,
            },
            key: GatewayProviderTransportKey {
                id: "key-1".to_string(),
                provider_id: "provider-1".to_string(),
                name: "key".to_string(),
                auth_type: "api_key".to_string(),
                is_active: true,
                api_formats: None,
                auth_type_by_format: None,
                allow_auth_channel_mismatch_formats: None,
                allowed_models: None,
                capabilities: None,
                rate_multipliers: None,
                global_priority_by_format: None,
                expires_at_unix_secs: None,
                proxy: None,
                fingerprint: None,
                upstream_metadata: None,
                decrypted_api_key: "secret".to_string(),
                decrypted_auth_config: None,
            },
        }
    }

    #[test]
    fn validates_configured_profile_without_resolving_provider_defaults() {
        assert!(validate_anthropic_compatibility_profile_config(None).is_ok());
        assert!(
            validate_anthropic_compatibility_profile_config(Some(&json!({
                "anthropic_compatibility": {"profile": "native_transparent"}
            })))
            .is_ok()
        );
        assert!(
            validate_anthropic_compatibility_profile_config(Some(&json!({
                "anthropic": {"compatibility_profile": "claude_code_legacy"}
            })))
            .is_ok()
        );

        let error = validate_anthropic_compatibility_profile_config(Some(&json!({
            "anthropic_compatibility": {"profile": "claude_cod_typo"}
        })))
        .expect_err("unknown profile should be rejected");
        assert_eq!(error.to_string(), "unknown Anthropic compatibility profile");
    }

    #[test]
    fn ignores_generic_compatibility_fields_owned_by_other_transports() {
        let config = json!({
            "compatibility_profile": "strict",
            "compatibility": {"profile": "v2"},
            "adaptation": {"profile": "legacy"}
        });

        assert!(validate_anthropic_compatibility_profile_config(Some(&config)).is_ok());
        let mut transport = sample_transport("custom");
        transport.provider.config = Some(config);
        assert_eq!(
            resolve_anthropic_compatibility_profile(&transport, "claude:messages"),
            AnthropicCompatibilityProfile::NativeTransparent
        );
    }

    #[test]
    fn native_anthropic_is_transparent_by_default() {
        let transport = sample_transport("custom");

        assert_eq!(
            resolve_anthropic_compatibility_profile(&transport, "claude:messages"),
            AnthropicCompatibilityProfile::NativeTransparent
        );
    }

    #[test]
    fn claude_code_keeps_legacy_compatibility_by_default() {
        let transport = sample_transport("claude_code");

        assert_eq!(
            resolve_anthropic_compatibility_profile(&transport, "claude:messages"),
            AnthropicCompatibilityProfile::ClaudeCodeLegacy
        );
    }

    #[test]
    fn endpoint_profile_overrides_provider_and_legacy_defaults() {
        let mut transport = sample_transport("claude_code");
        transport.provider.config = Some(json!({
            "anthropic": {"compatibility_profile": "claude_code_legacy"}
        }));
        transport.endpoint.config = Some(json!({
            "anthropic_compatibility": {
                "profile": "native_transparent"
            }
        }));

        assert_eq!(
            resolve_anthropic_compatibility_profile(&transport, "claude:messages"),
            AnthropicCompatibilityProfile::NativeTransparent
        );
    }

    #[test]
    fn provider_profile_can_explicitly_enable_legacy_compatibility() {
        let mut transport = sample_transport("custom");
        transport.provider.config = Some(json!({
            "anthropic": {
                "compatibility_profile": "claude_code"
            }
        }));

        assert_eq!(
            resolve_anthropic_compatibility_profile(&transport, "claude:messages"),
            AnthropicCompatibilityProfile::ClaudeCodeLegacy
        );
    }

    #[test]
    fn anthropic_profile_is_ignored_for_non_anthropic_formats() {
        let mut transport = sample_transport("claude_code");
        transport.endpoint.config = Some(json!({
            "anthropic": {"compatibility_profile": "claude_code_legacy"}
        }));

        assert_eq!(
            resolve_anthropic_compatibility_profile(&transport, "openai:chat"),
            AnthropicCompatibilityProfile::NativeTransparent
        );
    }

    #[test]
    fn invalid_explicit_profile_fails_closed_instead_of_using_legacy_default() {
        let mut transport = sample_transport("claude_code");
        transport.endpoint.config = Some(json!({
            "anthropic_compatibility": {"profile": "native_transparnt"}
        }));
        transport.provider.config = Some(json!({
            "anthropic_compatibility": {"profile": "claude_code_legacy"}
        }));

        assert_eq!(
            resolve_anthropic_compatibility_profile(&transport, "claude:messages"),
            AnthropicCompatibilityProfile::NativeTransparent
        );
    }
}

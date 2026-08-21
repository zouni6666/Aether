use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::snapshot::GatewayProviderTransportSnapshot;

pub const CODEX_FINGERPRINT_CONFIG_NAMESPACE: &str = "codex";
pub const CODEX_FINGERPRINT_ENABLED_CONFIG_KEY: &str = "fingerprint_convergence_enabled";

#[derive(Debug, Clone, PartialEq, Eq)]
struct CodexConvergedFingerprint {
    installation_id: String,
    session_id: String,
    thread_id: String,
    turn_id: String,
    window_id: String,
    turn_started_at_unix_ms: u64,
}

pub fn codex_fingerprint_convergence_enabled(
    provider_type: &str,
    provider_config: Option<&Value>,
) -> bool {
    provider_type.trim().eq_ignore_ascii_case("codex")
        && provider_config
            .and_then(Value::as_object)
            .and_then(|config| config.get(CODEX_FINGERPRINT_CONFIG_NAMESPACE))
            .and_then(Value::as_object)
            .and_then(|config| config.get(CODEX_FINGERPRINT_ENABLED_CONFIG_KEY))
            .and_then(Value::as_bool)
            .unwrap_or(false)
}

pub fn apply_codex_oauth_fingerprint_convergence(
    transport: &GatewayProviderTransportSnapshot,
    provider_api_format: &str,
    original_client_session_id: Option<&str>,
    provider_request_headers: &mut BTreeMap<String, String>,
    provider_request_body: &mut Value,
) -> bool {
    let is_responses = aether_ai_formats::is_openai_responses_format(provider_api_format);
    let is_live = aether_ai_formats::api_format_alias_matches(provider_api_format, "codex:live");
    if !transport
        .provider
        .provider_type
        .trim()
        .eq_ignore_ascii_case("codex")
        || !transport.key.auth_type.trim().eq_ignore_ascii_case("oauth")
        || crate::agent_identity::is_codex_agent_identity_transport(transport)
        || (!is_responses && !is_live)
        || is_responses
            && aether_ai_formats::openai_responses_request_operation(
                provider_api_format,
                provider_request_body,
            ) == Some(aether_ai_formats::OPENAI_RESPONSES_OPERATION_COMPACT)
        || !codex_fingerprint_convergence_enabled(
            transport.provider.provider_type.as_str(),
            transport.provider.config.as_ref(),
        )
        || !provider_request_body.is_object()
    {
        return false;
    }

    let auth_identity = aether_ai_formats::parse_codex_auth_identity(
        transport.key.decrypted_auth_config.as_deref(),
    );
    let account_seed = auth_identity
        .account_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(transport.key.id.as_str());
    let fingerprint = resolve_converged_fingerprint(account_seed, original_client_session_id);

    apply_converged_headers(provider_request_headers, &fingerprint);
    // Live uses the converged identity on the WebSocket/call-control headers.
    // Its event/session payload is an independent opaque protocol and must not
    // receive Responses-only `client_metadata` fields.
    if is_responses {
        apply_converged_client_metadata(provider_request_body, &fingerprint);
    }
    true
}

fn resolve_converged_fingerprint(
    account_seed: &str,
    original_client_session_id: Option<&str>,
) -> CodexConvergedFingerprint {
    let installation_id =
        derive_stable_uuid_v4(&format!("aether:codex-installation-id:v1:{account_seed}"));
    let session_id = derive_stable_uuid_v4(&format!("aether:codex-session-id:v1:{account_seed}"));
    let original_client_session_id = original_client_session_id
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let thread_id = original_client_session_id
        .map(|client_session_id| {
            derive_stable_uuid_v4(&format!(
                "aether:codex-thread-id:v1:{account_seed}:{client_session_id}"
            ))
        })
        .unwrap_or_else(|| session_id.clone());
    let window_id = format!("{thread_id}:0");
    let turn_started_at_unix_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis().min(u64::MAX as u128) as u64)
        .unwrap_or(0);

    CodexConvergedFingerprint {
        installation_id,
        session_id,
        thread_id,
        turn_id: Uuid::now_v7().to_string(),
        window_id,
        turn_started_at_unix_ms,
    }
}

fn derive_stable_uuid_v4(seed: &str) -> String {
    let digest = Sha256::digest(seed.as_bytes());
    let mut bytes = [0_u8; 16];
    bytes.copy_from_slice(&digest[..16]);
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    Uuid::from_bytes(bytes).to_string()
}

fn apply_converged_headers(
    headers: &mut BTreeMap<String, String>,
    fingerprint: &CodexConvergedFingerprint,
) {
    set_header(
        headers,
        "x-codex-installation-id",
        fingerprint.installation_id.clone(),
    );
    set_header(headers, "x-codex-window-id", fingerprint.window_id.clone());
    set_header(
        headers,
        "x-client-request-id",
        fingerprint.thread_id.clone(),
    );
    set_header(headers, "session-id", fingerprint.session_id.clone());
    set_header(headers, "session_id", fingerprint.session_id.clone());
    set_header(headers, "thread-id", fingerprint.thread_id.clone());
    // Codex Live/Realtime uses `x-session-id` for the thread-scoped session
    // identity on the WebSocket upgrade request. Keep it aligned with the
    // converged thread identity instead of the account-scoped session value.
    set_header(headers, "x-session-id", fingerprint.thread_id.clone());
    rewrite_header_turn_metadata(headers, fingerprint);
}

fn apply_converged_client_metadata(body: &mut Value, fingerprint: &CodexConvergedFingerprint) {
    let Some(body) = body.as_object_mut() else {
        return;
    };
    let metadata = body
        .entry("client_metadata".to_string())
        .or_insert_with(|| Value::Object(Map::new()));
    if !metadata.is_object() {
        *metadata = Value::Object(Map::new());
    }
    let Some(metadata) = metadata.as_object_mut() else {
        return;
    };

    metadata.insert(
        "x-codex-installation-id".to_string(),
        Value::String(fingerprint.installation_id.clone()),
    );
    metadata.insert(
        "session_id".to_string(),
        Value::String(fingerprint.session_id.clone()),
    );
    metadata.insert(
        "thread_id".to_string(),
        Value::String(fingerprint.thread_id.clone()),
    );
    metadata.insert(
        "turn_id".to_string(),
        Value::String(fingerprint.turn_id.clone()),
    );
    metadata.insert(
        "x-codex-window-id".to_string(),
        Value::String(fingerprint.window_id.clone()),
    );
    rewrite_embedded_turn_metadata(metadata, fingerprint);
}

fn rewrite_header_turn_metadata(
    headers: &mut BTreeMap<String, String>,
    fingerprint: &CodexConvergedFingerprint,
) {
    let Some((name, raw)) = find_header(headers, "x-codex-turn-metadata") else {
        return;
    };
    let Ok(mut metadata) = serde_json::from_str::<Map<String, Value>>(&raw) else {
        return;
    };
    apply_turn_metadata_fields(&mut metadata, fingerprint);
    let Ok(rebuilt) = serde_json::to_string(&metadata) else {
        return;
    };
    headers.remove(&name);
    headers.insert("x-codex-turn-metadata".to_string(), rebuilt);
}

fn rewrite_embedded_turn_metadata(
    metadata: &mut Map<String, Value>,
    fingerprint: &CodexConvergedFingerprint,
) {
    let Some(raw) = metadata
        .get("x-codex-turn-metadata")
        .and_then(Value::as_str)
    else {
        return;
    };
    let Ok(mut turn_metadata) = serde_json::from_str::<Map<String, Value>>(raw) else {
        return;
    };
    apply_turn_metadata_fields(&mut turn_metadata, fingerprint);
    let Ok(rebuilt) = serde_json::to_string(&turn_metadata) else {
        return;
    };
    metadata.insert("x-codex-turn-metadata".to_string(), Value::String(rebuilt));
}

fn apply_turn_metadata_fields(
    metadata: &mut Map<String, Value>,
    fingerprint: &CodexConvergedFingerprint,
) {
    metadata.insert(
        "installation_id".to_string(),
        Value::String(fingerprint.installation_id.clone()),
    );
    metadata.insert(
        "session_id".to_string(),
        Value::String(fingerprint.session_id.clone()),
    );
    metadata.insert(
        "thread_id".to_string(),
        Value::String(fingerprint.thread_id.clone()),
    );
    metadata.insert(
        "turn_id".to_string(),
        Value::String(fingerprint.turn_id.clone()),
    );
    metadata.insert(
        "window_id".to_string(),
        Value::String(fingerprint.window_id.clone()),
    );
    metadata.insert(
        "turn_started_at_unix_ms".to_string(),
        Value::from(fingerprint.turn_started_at_unix_ms),
    );
}

fn set_header(headers: &mut BTreeMap<String, String>, name: &str, value: String) {
    let matching_names = headers
        .keys()
        .filter(|candidate| candidate.eq_ignore_ascii_case(name))
        .cloned()
        .collect::<Vec<_>>();
    for matching_name in matching_names {
        headers.remove(&matching_name);
    }
    headers.insert(name.to_string(), value);
}

fn find_header(headers: &BTreeMap<String, String>, name: &str) -> Option<(String, String)> {
    headers
        .iter()
        .find(|(candidate, _)| candidate.eq_ignore_ascii_case(name))
        .map(|(name, value)| (name.clone(), value.clone()))
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::snapshot::{
        GatewayProviderTransportEndpoint, GatewayProviderTransportKey,
        GatewayProviderTransportProvider,
    };

    fn sample_transport() -> GatewayProviderTransportSnapshot {
        GatewayProviderTransportSnapshot {
            provider: GatewayProviderTransportProvider {
                id: "provider-1".to_string(),
                name: "Codex".to_string(),
                provider_type: "codex".to_string(),
                website: None,
                is_active: true,
                keep_priority_on_conversion: false,
                enable_format_conversion: true,
                concurrent_limit: None,
                max_retries: None,
                proxy: None,
                request_timeout_secs: None,
                stream_first_byte_timeout_secs: None,
                config: Some(json!({
                    "codex": {"fingerprint_convergence_enabled": true},
                    "unrelated": {"kept": true}
                })),
            },
            endpoint: GatewayProviderTransportEndpoint {
                id: "endpoint-1".to_string(),
                provider_id: "provider-1".to_string(),
                api_format: "openai:responses".to_string(),
                api_family: None,
                endpoint_kind: None,
                is_active: true,
                base_url: "https://chatgpt.com/backend-api/codex".to_string(),
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
                name: "OAuth".to_string(),
                auth_type: "oauth".to_string(),
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
                decrypted_api_key: "access-token".to_string(),
                decrypted_auth_config: Some(json!({"account_id": "account-1"}).to_string()),
            },
        }
    }

    #[test]
    fn provider_config_switch_is_opt_in_and_codex_only() {
        assert!(!codex_fingerprint_convergence_enabled("codex", None));
        assert!(!codex_fingerprint_convergence_enabled(
            "codex",
            Some(&json!({"codex": {"fingerprint_convergence_enabled": false}}))
        ));
        assert!(codex_fingerprint_convergence_enabled(
            "CODEX",
            Some(&json!({"codex": {"fingerprint_convergence_enabled": true}}))
        ));
        assert!(!codex_fingerprint_convergence_enabled(
            "openai",
            Some(&json!({"codex": {"fingerprint_convergence_enabled": true}}))
        ));
    }

    #[test]
    fn convergence_rewrites_headers_and_body_with_one_identity_set() {
        let transport = sample_transport();
        let mut headers = BTreeMap::from([
            ("Session-Id".to_string(), "client-session".to_string()),
            (
                "X-Session-Id".to_string(),
                "client-live-session".to_string(),
            ),
            (
                "x-codex-turn-metadata".to_string(),
                json!({
                    "installation_id": "client-installation",
                    "session_id": "client-session",
                    "thread_source": "cli"
                })
                .to_string(),
            ),
        ]);
        let mut body = json!({
            "model": "gpt-5.4",
            "client_metadata": {
                "session_id": "client-session",
                "x-codex-turn-metadata": json!({
                    "installation_id": "client-installation",
                    "sandbox": "workspace-write"
                }).to_string()
            }
        });

        assert!(apply_codex_oauth_fingerprint_convergence(
            &transport,
            "openai:responses",
            Some("client-session"),
            &mut headers,
            &mut body,
        ));

        let session_id = headers.get("session-id").expect("session header");
        let thread_id = headers.get("thread-id").expect("thread header");
        let installation_id = headers
            .get("x-codex-installation-id")
            .expect("installation header");
        assert_eq!(
            Uuid::parse_str(session_id)
                .expect("session UUID")
                .get_version_num(),
            4
        );
        assert_eq!(
            Uuid::parse_str(thread_id)
                .expect("thread UUID")
                .get_version_num(),
            4
        );
        assert_eq!(
            Uuid::parse_str(installation_id)
                .expect("installation UUID")
                .get_version_num(),
            4
        );
        assert_eq!(headers["session_id"], *session_id);
        assert_eq!(headers["x-client-request-id"], *thread_id);
        assert_eq!(headers["x-session-id"], *thread_id);
        assert_eq!(headers["x-codex-window-id"], format!("{thread_id}:0"));
        assert_eq!(
            headers
                .keys()
                .filter(|name| name.eq_ignore_ascii_case("x-session-id"))
                .count(),
            1
        );
        assert_eq!(body["client_metadata"]["session_id"], *session_id);
        assert_eq!(body["client_metadata"]["thread_id"], *thread_id);
        assert_eq!(
            body["client_metadata"]["x-codex-installation-id"],
            *installation_id
        );

        let header_metadata: Value =
            serde_json::from_str(&headers["x-codex-turn-metadata"]).expect("header metadata");
        let body_metadata: Value = serde_json::from_str(
            body["client_metadata"]["x-codex-turn-metadata"]
                .as_str()
                .expect("embedded metadata"),
        )
        .expect("body metadata");
        assert_eq!(
            header_metadata["turn_id"],
            body["client_metadata"]["turn_id"]
        );
        assert_eq!(body_metadata["turn_id"], body["client_metadata"]["turn_id"]);
        assert_eq!(header_metadata["thread_source"], "cli");
        assert_eq!(body_metadata["sandbox"], "workspace-write");
        assert_eq!(
            Uuid::parse_str(
                body["client_metadata"]["turn_id"]
                    .as_str()
                    .expect("turn id")
            )
            .expect("turn UUID")
            .get_version_num(),
            7
        );
    }

    #[test]
    fn stable_account_identity_and_per_client_thread_are_deterministic() {
        let first = resolve_converged_fingerprint("account-1", Some("client-a"));
        let second = resolve_converged_fingerprint("account-1", Some("client-a"));
        let other_client = resolve_converged_fingerprint("account-1", Some("client-b"));
        let other_account = resolve_converged_fingerprint("account-2", Some("client-a"));

        assert_eq!(first.installation_id, second.installation_id);
        assert_eq!(first.session_id, second.session_id);
        assert_eq!(first.thread_id, second.thread_id);
        assert_ne!(first.turn_id, second.turn_id);
        assert_ne!(first.thread_id, other_client.thread_id);
        assert_eq!(first.session_id, other_client.session_id);
        assert_ne!(first.installation_id, other_account.installation_id);
    }

    #[test]
    fn live_convergence_sets_the_websocket_identity_without_mutating_the_payload() {
        let transport = sample_transport();
        let original_body = json!({"model": "gpt-live", "future_live_field": true});
        let mut body = original_body.clone();
        let mut headers = BTreeMap::new();

        assert!(apply_codex_oauth_fingerprint_convergence(
            &transport,
            "codex:live",
            Some("client-live-session"),
            &mut headers,
            &mut body,
        ));

        assert_eq!(body, original_body);
        assert_eq!(headers.get("x-session-id"), headers.get("thread-id"));
        assert!(headers.contains_key("x-codex-installation-id"));
        assert!(headers.contains_key("x-codex-window-id"));
    }

    #[test]
    fn disabled_or_out_of_scope_requests_are_unchanged() {
        let mut transport = sample_transport();
        let original_headers = BTreeMap::from([("session-id".to_string(), "client".to_string())]);
        let original_body = json!({"model": "gpt-5.4"});

        transport.provider.config = None;
        let mut headers = original_headers.clone();
        let mut body = original_body.clone();
        assert!(!apply_codex_oauth_fingerprint_convergence(
            &transport,
            "openai:responses",
            Some("client"),
            &mut headers,
            &mut body,
        ));
        assert_eq!(headers, original_headers);
        assert_eq!(body, original_body);
        transport.provider.config = Some(json!({
            "codex": {"fingerprint_convergence_enabled": true}
        }));

        for api_format in [
            "openai:responses:compact",
            "openai:chat",
            "openai:search",
            "openai:image",
        ] {
            let mut headers = original_headers.clone();
            let mut body = original_body.clone();
            assert!(!apply_codex_oauth_fingerprint_convergence(
                &transport,
                api_format,
                Some("client"),
                &mut headers,
                &mut body,
            ));
            assert_eq!(headers, original_headers);
            assert_eq!(body, original_body);
        }

        let mut compact_v2_headers = original_headers.clone();
        let mut compact_v2_body = json!({
            "model": "gpt-5.4",
            "input": [{"type": "compaction_trigger"}]
        });
        let original_compact_v2_body = compact_v2_body.clone();
        assert!(!apply_codex_oauth_fingerprint_convergence(
            &transport,
            "openai:responses",
            Some("client"),
            &mut compact_v2_headers,
            &mut compact_v2_body,
        ));
        assert_eq!(compact_v2_headers, original_headers);
        assert_eq!(compact_v2_body, original_compact_v2_body);

        for auth_type in ["api_key", "bearer"] {
            transport.key.auth_type = auth_type.to_string();
            let mut headers = original_headers.clone();
            let mut body = original_body.clone();
            assert!(!apply_codex_oauth_fingerprint_convergence(
                &transport,
                "openai:responses",
                Some("client"),
                &mut headers,
                &mut body,
            ));
            assert_eq!(headers, original_headers);
            assert_eq!(body, original_body);
        }
    }

    #[test]
    fn agent_identity_oauth_transport_is_unchanged() {
        let mut transport = sample_transport();
        transport.key.decrypted_auth_config = Some(
            json!({
                "auth_mode": "agentIdentity",
                "agent_identity": {
                    "agent_runtime_id": "runtime-1",
                    "agent_private_key": "private-key"
                }
            })
            .to_string(),
        );
        let original_headers = BTreeMap::from([("session-id".to_string(), "client".to_string())]);
        let original_body = json!({"model": "gpt-5.4"});
        let mut headers = original_headers.clone();
        let mut body = original_body.clone();

        assert!(!apply_codex_oauth_fingerprint_convergence(
            &transport,
            "openai:responses",
            Some("client"),
            &mut headers,
            &mut body,
        ));
        assert_eq!(headers, original_headers);
        assert_eq!(body, original_body);
    }
}

use aether_oauth::provider::providers::KiroProviderOAuthAdapter as CoreKiroProviderOAuthAdapter;
use async_trait::async_trait;
use sha2::{Digest, Sha256};

use super::super::oauth_refresh::{
    oauth_error_to_local_refresh_error, provider_oauth_transport_context_from_snapshot,
    CachedOAuthEntry, LocalOAuthHttpExecutor, LocalOAuthRefreshAdapter, LocalOAuthRefreshError,
    LocalResolvedOAuthRequestAuth, ProviderOAuthLocalHttpExecutor,
};
use super::super::snapshot::GatewayProviderTransportSnapshot;
use super::auth::{
    build_kiro_request_auth_from_config, resolve_local_kiro_request_auth, PROVIDER_TYPE,
};
use super::credentials::KiroAuthConfig;

#[cfg(test)]
const IDC_AMZ_USER_AGENT: &str = "aws-sdk-js/3.738.0 ua/2.1 os/other lang/js md/browser#unknown_unknown api/sso-oidc#3.738.0 m/E KiroIDE";

#[derive(Debug, Clone, Default)]
pub struct KiroOAuthRefreshAdapter {
    social_refresh_base_url: Option<String>,
    idc_refresh_base_url: Option<String>,
}

impl KiroOAuthRefreshAdapter {
    pub fn with_refresh_base_urls(
        mut self,
        social_refresh_base_url: Option<String>,
        idc_refresh_base_url: Option<String>,
    ) -> Self {
        self.social_refresh_base_url = social_refresh_base_url;
        self.idc_refresh_base_url = idc_refresh_base_url;
        self
    }

    pub async fn refresh_auth_config(
        &self,
        executor: &dyn LocalOAuthHttpExecutor,
        transport: &GatewayProviderTransportSnapshot,
        auth_config: &KiroAuthConfig,
    ) -> Result<KiroAuthConfig, LocalOAuthRefreshError> {
        let adapter = CoreKiroProviderOAuthAdapter::default().with_refresh_base_urls(
            self.social_refresh_base_url.clone(),
            self.idc_refresh_base_url.clone(),
        );
        let oauth_executor =
            ProviderOAuthLocalHttpExecutor::new(PROVIDER_TYPE, transport, executor);
        let ctx = provider_oauth_transport_context_from_snapshot(transport);
        adapter
            .refresh_auth_config(&oauth_executor, &ctx, auth_config)
            .await
            .map_err(|error| oauth_error_to_local_refresh_error(PROVIDER_TYPE, error))
    }

    fn auth_config_from_entry(
        transport: &GatewayProviderTransportSnapshot,
        entry: &CachedOAuthEntry,
    ) -> Option<KiroAuthConfig> {
        entry
            .metadata
            .as_ref()
            .filter(|_| entry.provider_type.eq_ignore_ascii_case(PROVIDER_TYPE))
            .filter(|_| kiro_cached_entry_matches_transport(transport, entry))
            .and_then(KiroAuthConfig::from_json_value)
    }

    fn base_auth_config(
        &self,
        transport: &GatewayProviderTransportSnapshot,
        entry: Option<&CachedOAuthEntry>,
    ) -> Option<KiroAuthConfig> {
        entry
            .and_then(|entry| Self::auth_config_from_entry(transport, entry))
            .or_else(|| {
                KiroAuthConfig::from_raw_json(transport.key.decrypted_auth_config.as_deref())
            })
    }

    fn build_cached_entry(
        transport: &GatewayProviderTransportSnapshot,
        auth_config: &KiroAuthConfig,
    ) -> Option<CachedOAuthEntry> {
        let request_auth = build_kiro_request_auth_from_config(auth_config.clone(), None)?;
        Some(CachedOAuthEntry {
            provider_type: PROVIDER_TYPE.to_string(),
            auth_header_name: request_auth.name.to_string(),
            auth_header_value: request_auth.value,
            expires_at_unix_secs: auth_config.expires_at,
            metadata: Some(auth_config.to_json_value()),
            source_fingerprint: Some(kiro_transport_credential_fingerprint(transport)),
        })
    }

    fn build_cached_entry_from_transport(
        transport: &GatewayProviderTransportSnapshot,
    ) -> Option<CachedOAuthEntry> {
        let request_auth = resolve_local_kiro_request_auth(transport)?;
        Some(CachedOAuthEntry {
            provider_type: PROVIDER_TYPE.to_string(),
            auth_header_name: request_auth.name.to_string(),
            auth_header_value: request_auth.value,
            expires_at_unix_secs: request_auth.auth_config.expires_at,
            metadata: Some(request_auth.auth_config.to_json_value()),
            source_fingerprint: Some(kiro_transport_credential_fingerprint(transport)),
        })
    }

    fn refreshable_auth_config(
        &self,
        transport: &GatewayProviderTransportSnapshot,
        entry: Option<&CachedOAuthEntry>,
    ) -> Option<KiroAuthConfig> {
        let auth_config = self.base_auth_config(transport, entry)?;
        auth_config
            .can_refresh_access_token()
            .then_some(auth_config)
    }
}

#[async_trait]
impl LocalOAuthRefreshAdapter for KiroOAuthRefreshAdapter {
    fn provider_type(&self) -> &'static str {
        PROVIDER_TYPE
    }

    fn resolve_cached(
        &self,
        transport: &GatewayProviderTransportSnapshot,
        entry: &CachedOAuthEntry,
    ) -> Option<LocalResolvedOAuthRequestAuth> {
        let auth_config = Self::auth_config_from_entry(transport, entry)?;
        let request_auth = build_kiro_request_auth_from_config(auth_config, None)?;
        Some(LocalResolvedOAuthRequestAuth::Kiro(request_auth))
    }

    fn resolve_without_refresh(
        &self,
        transport: &GatewayProviderTransportSnapshot,
    ) -> Option<LocalResolvedOAuthRequestAuth> {
        resolve_local_kiro_request_auth(transport).map(LocalResolvedOAuthRequestAuth::Kiro)
    }

    fn should_refresh(
        &self,
        transport: &GatewayProviderTransportSnapshot,
        entry: Option<&CachedOAuthEntry>,
    ) -> bool {
        entry
            .and_then(|cached| self.resolve_cached(transport, cached))
            .is_none()
            && self.resolve_without_refresh(transport).is_none()
            && self.refreshable_auth_config(transport, entry).is_some()
    }

    fn refresh_fingerprint(
        &self,
        transport: &GatewayProviderTransportSnapshot,
        entry: Option<&CachedOAuthEntry>,
    ) -> Option<String> {
        self.supports(transport).then(|| {
            kiro_successor_refresh_fingerprint(transport, entry)
                .unwrap_or_else(|| kiro_transport_credential_fingerprint(transport))
        })
    }

    fn cached_entry_from_transport(
        &self,
        transport: &GatewayProviderTransportSnapshot,
    ) -> Option<CachedOAuthEntry> {
        Self::build_cached_entry_from_transport(transport)
    }

    async fn refresh(
        &self,
        executor: &dyn LocalOAuthHttpExecutor,
        transport: &GatewayProviderTransportSnapshot,
        entry: Option<&CachedOAuthEntry>,
    ) -> Result<Option<CachedOAuthEntry>, LocalOAuthRefreshError> {
        let Some(auth_config) = self.refreshable_auth_config(transport, entry) else {
            return Ok(None);
        };
        let refreshed = self
            .refresh_auth_config(executor, transport, &auth_config)
            .await?;
        Ok(Self::build_cached_entry(transport, &refreshed))
    }
}

fn kiro_transport_credential_fingerprint(transport: &GatewayProviderTransportSnapshot) -> String {
    kiro_credential_fingerprint(
        transport.provider.provider_type.as_str(),
        transport.key.auth_type.as_str(),
        transport
            .key
            .decrypted_auth_config
            .as_deref()
            .unwrap_or_default(),
        transport.key.decrypted_api_key.as_str(),
    )
}

fn kiro_successor_refresh_fingerprint(
    transport: &GatewayProviderTransportSnapshot,
    entry: Option<&CachedOAuthEntry>,
) -> Option<String> {
    let entry = entry.filter(|entry| kiro_cached_entry_matches_transport(transport, entry))?;
    let metadata = serde_json::to_string(entry.metadata.as_ref()?).ok()?;
    let access_token = bearer_access_token(entry.auth_header_value.as_str())?;
    Some(kiro_credential_fingerprint(
        transport.provider.provider_type.as_str(),
        transport.key.auth_type.as_str(),
        metadata.as_str(),
        access_token,
    ))
}

fn kiro_cached_entry_matches_transport(
    transport: &GatewayProviderTransportSnapshot,
    entry: &CachedOAuthEntry,
) -> bool {
    entry.provider_type.eq_ignore_ascii_case(PROVIDER_TYPE)
        && entry.source_fingerprint.as_deref()
            == Some(kiro_transport_credential_fingerprint(transport).as_str())
}

fn kiro_credential_fingerprint(
    provider_type: &str,
    auth_type: &str,
    auth_config: &str,
    access_token: &str,
) -> String {
    let provider_type = provider_type.trim().to_ascii_lowercase();
    let auth_type = auth_type.trim().to_ascii_lowercase();
    let mut digest = Sha256::new();
    for field in [
        provider_type.as_bytes(),
        auth_type.as_bytes(),
        auth_config.as_bytes(),
        access_token.as_bytes(),
    ] {
        digest.update((field.len() as u64).to_be_bytes());
        digest.update(field);
    }
    format!("{:x}", digest.finalize())
}

fn bearer_access_token(authorization: &str) -> Option<&str> {
    let mut parts = authorization.split_ascii_whitespace();
    let scheme = parts.next()?;
    let token = parts.next()?;
    (scheme.eq_ignore_ascii_case("bearer") && parts.next().is_none()).then_some(token)
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use super::super::super::oauth_refresh::{
        LocalOAuthRefreshAdapter, LocalResolvedOAuthRequestAuth, ReqwestLocalOAuthHttpExecutor,
    };
    use super::super::super::snapshot::{
        GatewayProviderTransportEndpoint, GatewayProviderTransportKey,
        GatewayProviderTransportProvider, GatewayProviderTransportSnapshot,
    };
    use super::{KiroAuthConfig, KiroOAuthRefreshAdapter, IDC_AMZ_USER_AGENT};
    use axum::body::to_bytes;
    use axum::extract::Request;
    use axum::response::IntoResponse;
    use axum::routing::any;
    use axum::{Json, Router};
    use http::StatusCode;
    use serde_json::{json, Value};
    use tokio::task::JoinHandle;

    #[derive(Debug, Clone)]
    struct SeenRefreshRequest {
        body: Value,
        authorization: String,
        host: String,
        user_agent: String,
        x_amz_user_agent: String,
    }

    fn sample_transport(raw_auth_config: &str) -> GatewayProviderTransportSnapshot {
        GatewayProviderTransportSnapshot {
            provider: GatewayProviderTransportProvider {
                id: "provider-1".to_string(),
                name: "Kiro".to_string(),
                provider_type: "kiro".to_string(),
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
                endpoint_kind: Some("cli".to_string()),
                is_active: true,
                base_url: "https://kiro.example".to_string(),
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
                auth_type: "bearer".to_string(),
                is_active: true,
                api_formats: Some(vec!["claude:messages".to_string()]),
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
                decrypted_api_key: "__placeholder__".to_string(),
                decrypted_auth_config: Some(raw_auth_config.to_string()),
            },
        }
    }

    async fn start_server(app: Router) -> (String, JoinHandle<()>) {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("listener should bind");
        let addr = listener
            .local_addr()
            .expect("listener should expose local addr");
        let handle = tokio::spawn(async move {
            axum::serve(listener, app).await.expect("server should run");
        });
        (format!("http://{addr}"), handle)
    }

    #[test]
    fn cached_entry_is_bound_to_kiro_credential_generation() {
        let stable_refresh_token = "r".repeat(120);
        let source_transport = sample_transport(
            &json!({
                "refresh_token": stable_refresh_token,
                "machine_id": "123e4567-e89b-12d3-a456-426614174000",
                "kiro_version": "1.2.3"
            })
            .to_string(),
        );
        let refreshed_config = KiroAuthConfig::from_raw_json(Some(
            &json!({
                "refresh_token": "s".repeat(120),
                "access_token": "fresh-kiro-access-token",
                "expires_at": u64::MAX,
                "machine_id": "123e4567-e89b-12d3-a456-426614174000",
                "kiro_version": "1.2.3"
            })
            .to_string(),
        ))
        .expect("refreshed Kiro config should parse");
        let entry =
            KiroOAuthRefreshAdapter::build_cached_entry(&source_transport, &refreshed_config)
                .expect("refreshed Kiro entry should build");
        let adapter = KiroOAuthRefreshAdapter::default();

        assert!(adapter.resolve_cached(&source_transport, &entry).is_some());
        assert_ne!(
            adapter.refresh_fingerprint(&source_transport, None),
            adapter.refresh_fingerprint(&source_transport, Some(&entry))
        );

        let replacement_transport = sample_transport(
            &json!({
                "refresh_token": "admin-refresh-token",
                "access_token": "admin-access-token",
                "expires_at": u64::MAX,
                "machine_id": "123e4567-e89b-12d3-a456-426614174001",
                "kiro_version": "1.2.3"
            })
            .to_string(),
        );
        assert!(adapter
            .resolve_cached(&replacement_transport, &entry)
            .is_none());
        let selected_refresh_config = adapter
            .base_auth_config(&replacement_transport, Some(&entry))
            .expect("replacement transport should provide refresh config");
        assert_eq!(
            selected_refresh_config.refresh_token.as_deref(),
            Some("admin-refresh-token")
        );
        assert_eq!(
            selected_refresh_config.access_token.as_deref(),
            Some("admin-access-token")
        );

        let replacement_entry =
            LocalOAuthRefreshAdapter::cached_entry_from_transport(&adapter, &replacement_transport)
                .expect("persisted replacement should reconstruct a generation-bound entry");
        assert!(adapter
            .resolve_cached(&replacement_transport, &replacement_entry)
            .is_some());
    }

    #[tokio::test]
    async fn refreshes_social_token_via_adapter() {
        let seen_request = Arc::new(Mutex::new(None::<SeenRefreshRequest>));
        let seen_request_clone = Arc::clone(&seen_request);
        let server = Router::new().route(
            "/refreshToken",
            any(move |request: Request| {
                let seen_request_inner = Arc::clone(&seen_request_clone);
                async move {
                    let (parts, body) = request.into_parts();
                    let raw_body = to_bytes(body, usize::MAX).await.expect("body should read");
                    let body: Value =
                        serde_json::from_slice(&raw_body).expect("body should parse as json");
                    *seen_request_inner.lock().expect("mutex should lock") =
                        Some(SeenRefreshRequest {
                            body,
                            authorization: parts
                                .headers
                                .get("authorization")
                                .and_then(|value| value.to_str().ok())
                                .unwrap_or_default()
                                .to_string(),
                            host: parts
                                .headers
                                .get("host")
                                .and_then(|value| value.to_str().ok())
                                .unwrap_or_default()
                                .to_string(),
                            user_agent: parts
                                .headers
                                .get("user-agent")
                                .and_then(|value| value.to_str().ok())
                                .unwrap_or_default()
                                .to_string(),
                            x_amz_user_agent: parts
                                .headers
                                .get("x-amz-user-agent")
                                .and_then(|value| value.to_str().ok())
                                .unwrap_or_default()
                                .to_string(),
                        });
                    (
                        StatusCode::OK,
                        Json(json!({
                            "accessToken": "cached-kiro-access-token",
                            "refreshToken": "s".repeat(120),
                            "expiresIn": 3600,
                            "profileArn": "arn:aws:bedrock:demo"
                        })),
                    )
                        .into_response()
                }
            }),
        );
        let (server_url, server_handle) = start_server(server).await;
        let adapter =
            KiroOAuthRefreshAdapter::default().with_refresh_base_urls(Some(server_url), None);
        let transport = sample_transport(
            r#"{
                "refresh_token":"rrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrr",
                "machine_id":"123e4567-e89b-12d3-a456-426614174000",
                "kiro_version":"1.2.3"
            }"#,
        );
        let executor = ReqwestLocalOAuthHttpExecutor::new(reqwest::Client::new());

        let entry = adapter
            .refresh(&executor, &transport, None)
            .await
            .expect("refresh should succeed")
            .expect("cached entry should exist");
        let resolved = adapter
            .resolve_cached(&transport, &entry)
            .expect("cached entry should resolve");
        let seen_request = seen_request
            .lock()
            .expect("mutex should lock")
            .clone()
            .expect("refresh request should be captured");

        assert_eq!(seen_request.body["refreshToken"], json!("r".repeat(120)));
        assert_eq!(seen_request.authorization, "");
        assert!(!seen_request.user_agent.is_empty());
        assert_eq!(seen_request.x_amz_user_agent, "");
        assert!(!seen_request.host.trim().is_empty());
        match resolved {
            LocalResolvedOAuthRequestAuth::Kiro(auth) => {
                assert_eq!(auth.value, "Bearer cached-kiro-access-token");
                assert_eq!(
                    auth.auth_config.profile_arn.as_deref(),
                    Some("arn:aws:bedrock:demo")
                );
                assert!(auth.auth_config.expires_at.is_some());
            }
            other => panic!("unexpected resolved auth: {other:?}"),
        }

        server_handle.abort();
    }

    #[tokio::test]
    async fn refreshes_idc_token_via_adapter() {
        let seen_request = Arc::new(Mutex::new(None::<SeenRefreshRequest>));
        let seen_request_clone = Arc::clone(&seen_request);
        let server = Router::new().route(
            "/token",
            any(move |request: Request| {
                let seen_request_inner = Arc::clone(&seen_request_clone);
                async move {
                    let (parts, body) = request.into_parts();
                    let raw_body = to_bytes(body, usize::MAX).await.expect("body should read");
                    let body: Value =
                        serde_json::from_slice(&raw_body).expect("body should parse as json");
                    *seen_request_inner.lock().expect("mutex should lock") =
                        Some(SeenRefreshRequest {
                            body,
                            authorization: parts
                                .headers
                                .get("authorization")
                                .and_then(|value| value.to_str().ok())
                                .unwrap_or_default()
                                .to_string(),
                            host: parts
                                .headers
                                .get("host")
                                .and_then(|value| value.to_str().ok())
                                .unwrap_or_default()
                                .to_string(),
                            user_agent: parts
                                .headers
                                .get("user-agent")
                                .and_then(|value| value.to_str().ok())
                                .unwrap_or_default()
                                .to_string(),
                            x_amz_user_agent: parts
                                .headers
                                .get("x-amz-user-agent")
                                .and_then(|value| value.to_str().ok())
                                .unwrap_or_default()
                                .to_string(),
                        });
                    (
                        StatusCode::OK,
                        Json(json!({
                            "accessToken": "cached-idc-access-token",
                            "refreshToken": "i".repeat(120),
                            "expiresIn": 1800
                        })),
                    )
                        .into_response()
                }
            }),
        );
        let (server_url, server_handle) = start_server(server).await;
        let adapter =
            KiroOAuthRefreshAdapter::default().with_refresh_base_urls(None, Some(server_url));
        let transport = sample_transport(
            r#"{
                "auth_method":"identity_center",
                "refresh_token":"rrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrr",
                "client_id":"cid",
                "client_secret":"secret",
                "profile_arn":"arn:aws:bedrock:demo"
            }"#,
        );
        let executor = ReqwestLocalOAuthHttpExecutor::new(reqwest::Client::new());

        let entry = adapter
            .refresh(&executor, &transport, None)
            .await
            .expect("refresh should succeed")
            .expect("cached entry should exist");
        let resolved = adapter
            .resolve_cached(&transport, &entry)
            .expect("cached entry should resolve");
        let seen_request = seen_request
            .lock()
            .expect("mutex should lock")
            .clone()
            .expect("refresh request should be captured");

        assert_eq!(
            seen_request.body["grantType"].as_str(),
            Some("refresh_token")
        );
        assert_eq!(seen_request.body["clientId"].as_str(), Some("cid"));
        assert_eq!(seen_request.user_agent, "node");
        assert_eq!(seen_request.x_amz_user_agent, IDC_AMZ_USER_AGENT);
        assert!(!seen_request.host.trim().is_empty());
        match resolved {
            LocalResolvedOAuthRequestAuth::Kiro(auth) => {
                assert_eq!(auth.value, "Bearer cached-idc-access-token");
                assert!(auth.auth_config.profile_arn_for_payload().is_none());
                assert!(auth.auth_config.expires_at.is_some());
            }
            other => panic!("unexpected resolved auth: {other:?}"),
        }

        server_handle.abort();
    }
}

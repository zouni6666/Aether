use super::generic::{template_for_provider_type, GenericProviderOAuthAdapter};
use crate::core::{
    generate_oauth_nonce, generate_pkce_verifier, pkce_s256, OAuthAuthorizeResponse,
};
use crate::network::{OAuthHttpExecutor, OAuthHttpRequest};
use crate::provider::{
    ProviderOAuthAccount, ProviderOAuthAdapter, ProviderOAuthCapabilities,
    ProviderOAuthCookieAuthorizationInput, ProviderOAuthImportInput, ProviderOAuthProbeResult,
    ProviderOAuthRequestAuth, ProviderOAuthTokenSet, ProviderOAuthTransportContext,
};
use crate::OAuthError;
use aether_contracts::{
    ResolvedTransportProfile, EXECUTION_REQUEST_FOLLOW_REDIRECTS_HEADER,
    TRANSPORT_BACKEND_BROWSER_WREQ, TRANSPORT_HTTP_MODE_AUTO, TRANSPORT_POOL_SCOPE_KEY,
};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use url::Url;

pub const CLAUDE_CODE_PROVIDER_TYPE: &str = "claude_code";
pub const CLAUDE_CODE_CLIENT_ID: &str = "9d1c250a-e61b-44d9-88ed-5944d1962f5e";
pub const CLAUDE_CODE_WEB_BASE_URL: &str = "https://claude.ai";
pub const CLAUDE_CODE_AUTHORIZE_URL: &str = "https://claude.ai/oauth/authorize";
pub const CLAUDE_CODE_TOKEN_URL: &str = "https://platform.claude.com/v1/oauth/token";
pub const CLAUDE_CODE_REDIRECT_URI: &str = "https://platform.claude.com/oauth/code/callback";
pub const CLAUDE_CODE_OAUTH_SCOPES: &[&str] = &[
    "org:create_api_key",
    "user:profile",
    "user:inference",
    "user:sessions:claude_code",
    "user:mcp_servers",
    "user:file_upload",
];
pub const CLAUDE_CODE_COOKIE_SCOPE: &str =
    "user:profile user:inference user:sessions:claude_code user:mcp_servers user:file_upload";

const CLAUDE_CODE_BROWSER_USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/136.0.0.0 Safari/537.36";

#[derive(Debug, Clone)]
pub struct ClaudeCodeProviderOAuthAdapter {
    inner: GenericProviderOAuthAdapter,
    web_base_url: String,
}

impl Default for ClaudeCodeProviderOAuthAdapter {
    fn default() -> Self {
        Self {
            inner: GenericProviderOAuthAdapter::new(
                template_for_provider_type(CLAUDE_CODE_PROVIDER_TYPE)
                    .expect("claude code oauth template should exist"),
            ),
            web_base_url: CLAUDE_CODE_WEB_BASE_URL.to_string(),
        }
    }
}

impl ClaudeCodeProviderOAuthAdapter {
    pub fn with_endpoint_overrides(
        mut self,
        web_base_url: impl Into<String>,
        token_url: impl Into<String>,
    ) -> Self {
        self.web_base_url = web_base_url.into();
        self.inner = self.inner.with_token_url_override(token_url);
        self
    }

    fn web_url(&self, path_segments: &[&str]) -> Result<String, OAuthError> {
        let mut url = Url::parse(self.web_base_url.trim())
            .map_err(|_| OAuthError::invalid_request("claude web base url must be absolute"))?;
        url.set_query(None);
        url.set_fragment(None);
        {
            let mut segments = url
                .path_segments_mut()
                .map_err(|_| OAuthError::invalid_request("claude web base url is invalid"))?;
            segments.clear();
            segments.extend(path_segments.iter().copied());
        }
        Ok(url.to_string())
    }

    fn session_cookie(session_key: &str) -> Result<String, OAuthError> {
        let session_key = session_key.trim();
        if session_key.is_empty()
            || session_key.contains(['\r', '\n', ';'])
            || http::HeaderValue::from_str(session_key).is_err()
        {
            return Err(OAuthError::invalid_request("invalid Claude sessionKey"));
        }
        Ok(format!("sessionKey={session_key}"))
    }

    async fn organization_uuid(
        &self,
        executor: &dyn OAuthHttpExecutor,
        ctx: &ProviderOAuthTransportContext,
        cookie: &str,
    ) -> Result<String, OAuthError> {
        let response = executor
            .execute(OAuthHttpRequest {
                request_id: "provider-oauth:claude-cookie-organizations".to_string(),
                method: reqwest::Method::GET,
                url: self.web_url(&["api", "organizations"])?,
                headers: cookie_headers(cookie, false),
                content_type: None,
                json_body: None,
                body_bytes: None,
                network: ctx.network.clone(),
                transport_profile: claude_code_oauth_transport_profile_for_context(ctx),
            })
            .await?;
        ensure_success(&response)?;
        let organizations = response
            .json_body
            .or_else(|| serde_json::from_str::<Value>(&response.body_text).ok())
            .and_then(|value| value.as_array().cloned())
            .ok_or_else(|| {
                OAuthError::invalid_response("Claude organizations response is invalid")
            })?;

        let organization = if organizations.len() == 1 {
            organizations.first()
        } else {
            organizations
                .iter()
                .find(|organization| {
                    organization
                        .get("raven_type")
                        .and_then(Value::as_str)
                        .is_some_and(|value| value.eq_ignore_ascii_case("team"))
                })
                .or_else(|| organizations.first())
        }
        .ok_or_else(|| OAuthError::invalid_response("Claude account has no organizations"))?;

        organization
            .get("uuid")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned)
            .ok_or_else(|| OAuthError::invalid_response("Claude organization is missing uuid"))
    }

    async fn authorization_code(
        &self,
        executor: &dyn OAuthHttpExecutor,
        ctx: &ProviderOAuthTransportContext,
        cookie: &str,
        organization_uuid: &str,
        state: &str,
        code_challenge: &str,
    ) -> Result<String, OAuthError> {
        let response = executor
            .execute(OAuthHttpRequest {
                request_id: "provider-oauth:claude-cookie-authorize".to_string(),
                method: reqwest::Method::POST,
                url: self.web_url(&["v1", "oauth", organization_uuid, "authorize"])?,
                headers: cookie_headers(cookie, true),
                content_type: Some("application/json".to_string()),
                json_body: Some(json!({
                    "response_type": "code",
                    "client_id": CLAUDE_CODE_CLIENT_ID,
                    "organization_uuid": organization_uuid,
                    "redirect_uri": CLAUDE_CODE_REDIRECT_URI,
                    "scope": CLAUDE_CODE_COOKIE_SCOPE,
                    "state": state,
                    "code_challenge": code_challenge,
                    "code_challenge_method": "S256",
                })),
                body_bytes: None,
                network: ctx.network.clone(),
                transport_profile: claude_code_oauth_transport_profile_for_context(ctx),
            })
            .await?;
        ensure_success(&response)?;
        let payload = response
            .json_body
            .or_else(|| serde_json::from_str::<Value>(&response.body_text).ok())
            .ok_or_else(|| OAuthError::invalid_response("Claude authorize response is invalid"))?;
        let redirect_uri = payload
            .get("redirect_uri")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned)
            .ok_or_else(|| {
                OAuthError::invalid_response("Claude authorize response is missing redirect_uri")
            })?;

        validate_authorization_redirect(&redirect_uri, state)
    }
}

#[async_trait]
impl ProviderOAuthAdapter for ClaudeCodeProviderOAuthAdapter {
    fn provider_type(&self) -> &'static str {
        CLAUDE_CODE_PROVIDER_TYPE
    }

    fn capabilities(&self) -> ProviderOAuthCapabilities {
        ProviderOAuthCapabilities {
            supports_cookie_authorization: true,
            ..self.inner.capabilities()
        }
    }

    fn build_authorize_url(
        &self,
        ctx: &ProviderOAuthTransportContext,
        state: &str,
        code_challenge: Option<&str>,
    ) -> Result<OAuthAuthorizeResponse, OAuthError> {
        let mut response = self.inner.build_authorize_url(ctx, state, code_challenge)?;
        let mut url = Url::parse(&response.authorize_url)
            .map_err(|_| OAuthError::invalid_response("invalid Claude authorize_url"))?;
        url.query_pairs_mut().append_pair("code", "true");
        response.authorize_url = url.to_string();
        Ok(response)
    }

    async fn exchange_code(
        &self,
        executor: &dyn OAuthHttpExecutor,
        ctx: &ProviderOAuthTransportContext,
        code: &str,
        state: &str,
        pkce_verifier: Option<&str>,
    ) -> Result<ProviderOAuthTokenSet, OAuthError> {
        self.inner
            .exchange_code(executor, ctx, code, state, pkce_verifier)
            .await
    }

    async fn authorize_with_cookie(
        &self,
        executor: &dyn OAuthHttpExecutor,
        ctx: &ProviderOAuthTransportContext,
        input: ProviderOAuthCookieAuthorizationInput,
    ) -> Result<ProviderOAuthTokenSet, OAuthError> {
        let cookie = Self::session_cookie(&input.session_key)?;
        let organization_uuid = self.organization_uuid(executor, ctx, &cookie).await?;
        let state = generate_oauth_nonce();
        let verifier = generate_pkce_verifier();
        let challenge = pkce_s256(&verifier);
        let code = self
            .authorization_code(
                executor,
                ctx,
                &cookie,
                &organization_uuid,
                &state,
                &challenge,
            )
            .await?;
        self.inner
            .exchange_code(executor, ctx, &code, &state, Some(&verifier))
            .await
    }

    async fn import_credentials(
        &self,
        executor: &dyn OAuthHttpExecutor,
        ctx: &ProviderOAuthTransportContext,
        input: ProviderOAuthImportInput,
    ) -> Result<ProviderOAuthTokenSet, OAuthError> {
        self.inner.import_credentials(executor, ctx, input).await
    }

    async fn refresh(
        &self,
        executor: &dyn OAuthHttpExecutor,
        ctx: &ProviderOAuthTransportContext,
        account: &ProviderOAuthAccount,
    ) -> Result<ProviderOAuthTokenSet, OAuthError> {
        self.inner.refresh(executor, ctx, account).await
    }

    fn resolve_request_auth(
        &self,
        account: &ProviderOAuthAccount,
    ) -> Result<ProviderOAuthRequestAuth, OAuthError> {
        self.inner.resolve_request_auth(account)
    }

    fn account_fingerprint(&self, account: &ProviderOAuthAccount) -> Option<String> {
        self.inner.account_fingerprint(account)
    }

    async fn probe_account_state(
        &self,
        executor: &dyn OAuthHttpExecutor,
        ctx: &ProviderOAuthTransportContext,
        account: &ProviderOAuthAccount,
    ) -> Result<Option<ProviderOAuthProbeResult>, OAuthError> {
        self.inner.probe_account_state(executor, ctx, account).await
    }
}

pub(super) fn claude_code_oauth_transport_profile() -> ResolvedTransportProfile {
    ResolvedTransportProfile {
        profile_id: "claude_oauth_chrome136".to_string(),
        backend: TRANSPORT_BACKEND_BROWSER_WREQ.to_string(),
        http_mode: TRANSPORT_HTTP_MODE_AUTO.to_string(),
        pool_scope: TRANSPORT_POOL_SCOPE_KEY.to_string(),
        header_fingerprint: None,
        extra: Some(json!({ "browser_profile": "chrome136" })),
    }
}

fn claude_code_oauth_transport_profile_for_context(
    ctx: &ProviderOAuthTransportContext,
) -> Option<ResolvedTransportProfile> {
    // Node-only proxies must execute through the tunnel runtime, which cannot use browser_wreq.
    // The explicit browser headers still keep that fallback compatible with Claude's web flow.
    let tunnel_only_proxy = ctx.network.proxy.as_ref().is_some_and(|proxy| {
        if proxy.enabled == Some(false) {
            return false;
        }
        let has_proxy_url = proxy
            .url
            .as_deref()
            .map(str::trim)
            .is_some_and(|value| !value.is_empty());
        let has_node_id = proxy
            .node_id
            .as_deref()
            .map(str::trim)
            .is_some_and(|value| !value.is_empty());
        let tunnel_mode = proxy
            .mode
            .as_deref()
            .map(str::trim)
            .is_some_and(|value| value.eq_ignore_ascii_case("tunnel"));
        has_node_id && (tunnel_mode || !has_proxy_url)
    });
    (!tunnel_only_proxy).then(claude_code_oauth_transport_profile)
}

fn cookie_headers(cookie: &str, json_request: bool) -> BTreeMap<String, String> {
    let mut headers = BTreeMap::from([
        ("accept".to_string(), "application/json".to_string()),
        ("accept-language".to_string(), "en-US,en;q=0.9".to_string()),
        ("cache-control".to_string(), "no-cache".to_string()),
        ("cookie".to_string(), cookie.to_string()),
        (
            "user-agent".to_string(),
            CLAUDE_CODE_BROWSER_USER_AGENT.to_string(),
        ),
        (
            EXECUTION_REQUEST_FOLLOW_REDIRECTS_HEADER.to_string(),
            "false".to_string(),
        ),
    ]);
    if json_request {
        headers.insert("content-type".to_string(), "application/json".to_string());
        headers.insert("origin".to_string(), CLAUDE_CODE_WEB_BASE_URL.to_string());
        headers.insert(
            "referer".to_string(),
            format!("{CLAUDE_CODE_WEB_BASE_URL}/new"),
        );
    }
    headers
}

fn ensure_success(response: &crate::network::OAuthHttpResponse) -> Result<(), OAuthError> {
    if (200..300).contains(&response.status_code) {
        return Ok(());
    }
    Err(OAuthError::HttpStatus {
        status_code: response.status_code,
        body_excerpt: "Claude Cookie authorization request failed".to_string(),
    })
}

fn validate_authorization_redirect(
    redirect_uri: &str,
    expected_state: &str,
) -> Result<String, OAuthError> {
    let redirect = Url::parse(redirect_uri)
        .map_err(|_| OAuthError::invalid_response("Claude authorize redirect_uri is invalid"))?;
    let expected = Url::parse(CLAUDE_CODE_REDIRECT_URI).map_err(|_| {
        OAuthError::invalid_response("Claude redirect URI configuration is invalid")
    })?;
    if redirect.scheme() != expected.scheme()
        || redirect.host_str() != expected.host_str()
        || redirect.port_or_known_default() != expected.port_or_known_default()
        || redirect.path() != expected.path()
        || !redirect.username().is_empty()
        || redirect.password().is_some()
        || redirect.fragment().is_some()
    {
        return Err(OAuthError::invalid_response(
            "Claude authorize redirect_uri target is invalid",
        ));
    }

    let mut code = None;
    let mut state = None;
    for (key, value) in redirect.query_pairs() {
        match key.as_ref() {
            "code" if code.is_none() => code = Some(value.into_owned()),
            "state" if state.is_none() => state = Some(value.into_owned()),
            "code" | "state" => {
                return Err(OAuthError::invalid_response(
                    "Claude authorize redirect_uri has duplicate parameters",
                ));
            }
            _ => {}
        }
    }
    let code = code
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            OAuthError::invalid_response("Claude authorize redirect_uri is missing code")
        })?;
    let state = state
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            OAuthError::invalid_response("Claude authorize redirect_uri is missing state")
        })?;
    if state != expected_state {
        return Err(OAuthError::InvalidState);
    }
    Ok(code)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::{OAuthHttpResponse, OAuthNetworkContext};
    use aether_contracts::ProxySnapshot;
    use async_trait::async_trait;
    use std::sync::{Arc, Mutex};

    #[derive(Debug, Clone, Copy, Default)]
    enum RedirectMode {
        #[default]
        Matching,
        WrongState,
        HostileHost,
    }

    #[derive(Clone)]
    struct RecordingExecutor {
        requests: Arc<Mutex<Vec<OAuthHttpRequest>>>,
        organizations: Value,
        token_payload: Value,
        redirect_mode: RedirectMode,
    }

    impl Default for RecordingExecutor {
        fn default() -> Self {
            Self {
                requests: Arc::new(Mutex::new(Vec::new())),
                organizations: json!([
                    {"uuid": "org-personal", "raven_type": "personal"},
                    {"uuid": "org-team", "raven_type": "team"}
                ]),
                token_payload: json!({
                    "access_token": "sk-ant-oat01-new",
                    "refresh_token": "sk-ant-ort01-new",
                    "expires_in": 3600,
                    "organization": {"uuid": "org-team"},
                    "account": {
                        "uuid": "account-123",
                        "email_address": "alice@example.com"
                    }
                }),
                redirect_mode: RedirectMode::Matching,
            }
        }
    }

    #[async_trait]
    impl OAuthHttpExecutor for RecordingExecutor {
        async fn execute(
            &self,
            request: OAuthHttpRequest,
        ) -> Result<OAuthHttpResponse, OAuthError> {
            self.requests
                .lock()
                .expect("requests lock")
                .push(request.clone());

            let payload = if request.url.ends_with("/api/organizations") {
                self.organizations.clone()
            } else if request.url.contains("/authorize") {
                let requested_state = request
                    .json_body
                    .as_ref()
                    .and_then(|body| body.get("state"))
                    .and_then(Value::as_str)
                    .expect("authorize request should contain state");
                let state = match self.redirect_mode {
                    RedirectMode::Matching | RedirectMode::HostileHost => requested_state,
                    RedirectMode::WrongState => "wrong-state",
                };
                let redirect_base = match self.redirect_mode {
                    RedirectMode::HostileHost => {
                        "https://platform.claude.com.evil/oauth/code/callback"
                    }
                    _ => CLAUDE_CODE_REDIRECT_URI,
                };
                let mut redirect = Url::parse(redirect_base).expect("redirect URL should parse");
                redirect
                    .query_pairs_mut()
                    .append_pair("code", "authorization-code")
                    .append_pair("state", state);
                json!({"redirect_uri": redirect.to_string()})
            } else {
                self.token_payload.clone()
            };

            Ok(OAuthHttpResponse {
                status_code: 200,
                body_text: payload.to_string(),
                json_body: Some(payload),
            })
        }
    }

    fn context(proxy: Option<ProxySnapshot>) -> ProviderOAuthTransportContext {
        ProviderOAuthTransportContext {
            provider_id: "provider-claude".to_string(),
            provider_type: CLAUDE_CODE_PROVIDER_TYPE.to_string(),
            endpoint_id: None,
            key_id: None,
            auth_type: Some("oauth".to_string()),
            decrypted_api_key: None,
            decrypted_auth_config: None,
            provider_config: None,
            endpoint_config: None,
            key_config: None,
            network: OAuthNetworkContext::provider_operation(proxy),
        }
    }

    #[test]
    fn builds_current_manual_authorize_url() {
        let adapter = ClaudeCodeProviderOAuthAdapter::default();
        let response = adapter
            .build_authorize_url(&context(None), "state-123", Some("challenge-123"))
            .expect("authorize URL should build");
        let url = Url::parse(&response.authorize_url).expect("authorize URL should parse");
        let query = url
            .query_pairs()
            .map(|(key, value)| (key.into_owned(), value.into_owned()))
            .collect::<BTreeMap<_, _>>();

        assert_eq!(
            format!(
                "{}://{}{}",
                url.scheme(),
                url.host_str().unwrap_or_default(),
                url.path()
            ),
            CLAUDE_CODE_AUTHORIZE_URL
        );
        assert_eq!(
            query.get("client_id").map(String::as_str),
            Some(CLAUDE_CODE_CLIENT_ID)
        );
        assert_eq!(
            query.get("redirect_uri").map(String::as_str),
            Some(CLAUDE_CODE_REDIRECT_URI)
        );
        assert_eq!(
            query.get("scope").map(String::as_str),
            Some(CLAUDE_CODE_OAUTH_SCOPES.join(" ").as_str())
        );
        assert_eq!(query.get("code").map(String::as_str), Some("true"));
        assert_eq!(
            query.get("code_challenge").map(String::as_str),
            Some("challenge-123")
        );
        assert!(adapter.capabilities().supports_cookie_authorization);
    }

    #[tokio::test]
    async fn cookie_authorization_uses_team_org_safe_headers_and_current_token_contract() {
        let executor = RecordingExecutor::default();
        let adapter = ClaudeCodeProviderOAuthAdapter::default().with_endpoint_overrides(
            "https://claude.test",
            "https://platform.test/v1/oauth/token",
        );
        let session_key = "sk-ant-sid01-secret";

        let result = adapter
            .authorize_with_cookie(
                &executor,
                &context(None),
                ProviderOAuthCookieAuthorizationInput {
                    session_key: session_key.to_string(),
                },
            )
            .await
            .expect("cookie authorization should succeed");

        assert_eq!(result.token_set.access_token, "sk-ant-oat01-new");
        assert_eq!(
            result.token_set.refresh_token.as_deref(),
            Some("sk-ant-ort01-new")
        );
        assert_eq!(result.auth_config["org_uuid"], "org-team");
        assert_eq!(result.auth_config["account_uuid"], "account-123");
        assert_eq!(result.auth_config["email"], "alice@example.com");

        let requests = executor.requests.lock().expect("requests lock").clone();
        assert_eq!(requests.len(), 3);
        assert_eq!(requests[0].method, reqwest::Method::GET);
        assert!(requests[0].url.ends_with("/api/organizations"));
        assert!(requests[1].url.ends_with("/v1/oauth/org-team/authorize"));
        for request in &requests[..2] {
            assert_eq!(
                request.headers.get("cookie").map(String::as_str),
                Some("sessionKey=sk-ant-sid01-secret")
            );
            assert_eq!(
                request
                    .headers
                    .get(EXECUTION_REQUEST_FOLLOW_REDIRECTS_HEADER)
                    .map(String::as_str),
                Some("false")
            );
            assert_eq!(
                request.headers.get("user-agent").map(String::as_str),
                Some(CLAUDE_CODE_BROWSER_USER_AGENT)
            );
            assert_eq!(
                request
                    .transport_profile
                    .as_ref()
                    .map(|profile| profile.backend.as_str()),
                Some(TRANSPORT_BACKEND_BROWSER_WREQ)
            );
            assert!(!format!("{request:?}").contains(session_key));
        }
        assert_eq!(
            requests[1]
                .json_body
                .as_ref()
                .and_then(|body| body.get("scope"))
                .and_then(Value::as_str),
            Some(CLAUDE_CODE_COOKIE_SCOPE)
        );

        let token_request = &requests[2];
        assert_eq!(token_request.url, "https://platform.test/v1/oauth/token");
        assert!(!token_request.headers.contains_key("cookie"));
        assert!(token_request.transport_profile.is_none());
        assert_eq!(
            token_request.headers.get("user-agent").map(String::as_str),
            Some("axios/1.13.6")
        );
        let token_body = token_request
            .json_body
            .as_ref()
            .expect("token request should be JSON");
        assert!(token_body.get("scope").is_none());
        assert_eq!(token_body["redirect_uri"], CLAUDE_CODE_REDIRECT_URI);
        assert_eq!(token_body["code"], "authorization-code");
        assert!(token_body.get("code_verifier").is_some());
    }

    #[test]
    fn session_cookie_accepts_values_above_previous_length_cap() {
        let session_key = "x".repeat(20 * 1024);
        let cookie = ClaudeCodeProviderOAuthAdapter::session_cookie(&session_key)
            .expect("long sessionKey should remain valid");
        assert_eq!(cookie.len(), "sessionKey=".len() + session_key.len());
    }

    #[tokio::test]
    async fn rejects_wrong_state_and_hostile_authorize_redirects() {
        for redirect_mode in [RedirectMode::WrongState, RedirectMode::HostileHost] {
            let executor = RecordingExecutor {
                redirect_mode,
                ..RecordingExecutor::default()
            };
            let adapter = ClaudeCodeProviderOAuthAdapter::default().with_endpoint_overrides(
                "https://claude.test",
                "https://platform.test/v1/oauth/token",
            );
            let error = adapter
                .authorize_with_cookie(
                    &executor,
                    &context(None),
                    ProviderOAuthCookieAuthorizationInput {
                        session_key: "sk-ant-sid01-secret".to_string(),
                    },
                )
                .await
                .expect_err("unsafe redirect should be rejected");
            match redirect_mode {
                RedirectMode::WrongState => assert!(matches!(error, OAuthError::InvalidState)),
                RedirectMode::HostileHost => {
                    assert!(matches!(error, OAuthError::InvalidResponse(_)))
                }
                RedirectMode::Matching => unreachable!(),
            }
        }
    }

    #[test]
    fn tunnel_proxy_falls_back_from_browser_transport_even_when_url_is_present() {
        for proxy in [
            ProxySnapshot {
                node_id: Some("node-only".to_string()),
                ..ProxySnapshot::default()
            },
            ProxySnapshot {
                mode: Some("tunnel".to_string()),
                node_id: Some("node-with-url".to_string()),
                url: Some("http://127.0.0.1:9999".to_string()),
                ..ProxySnapshot::default()
            },
        ] {
            assert!(
                claude_code_oauth_transport_profile_for_context(&context(Some(proxy))).is_none()
            );
        }

        let url_proxy = ProxySnapshot {
            mode: Some("url".to_string()),
            node_id: Some("metadata-node".to_string()),
            url: Some("http://127.0.0.1:9999".to_string()),
            ..ProxySnapshot::default()
        };
        assert!(
            claude_code_oauth_transport_profile_for_context(&context(Some(url_proxy))).is_some()
        );
        assert_eq!(
            cookie_headers("sessionKey=test", false)
                .get("user-agent")
                .map(String::as_str),
            Some(CLAUDE_CODE_BROWSER_USER_AGENT)
        );
    }

    #[tokio::test]
    async fn refresh_rotates_claude_refresh_token_without_scope() {
        let executor = RecordingExecutor::default();
        let adapter = ClaudeCodeProviderOAuthAdapter::default().with_endpoint_overrides(
            "https://claude.test",
            "https://platform.test/v1/oauth/token",
        );
        let account = ProviderOAuthAccount {
            provider_type: CLAUDE_CODE_PROVIDER_TYPE.to_string(),
            access_token: "sk-ant-oat01-old".to_string(),
            auth_config: json!({
                "provider_type": CLAUDE_CODE_PROVIDER_TYPE,
                "refresh_token": "sk-ant-ort01-old",
                "email": "old@example.com"
            }),
            expires_at_unix_secs: Some(1),
            identity: BTreeMap::new(),
        };

        let refreshed = adapter
            .refresh(&executor, &context(None), &account)
            .await
            .expect("refresh should succeed");

        assert_eq!(refreshed.token_set.access_token, "sk-ant-oat01-new");
        assert_eq!(
            refreshed.token_set.refresh_token.as_deref(),
            Some("sk-ant-ort01-new")
        );
        assert_eq!(refreshed.auth_config["refresh_token"], "sk-ant-ort01-new");
        let requests = executor.requests.lock().expect("requests lock");
        assert_eq!(requests.len(), 1);
        let body = requests[0]
            .json_body
            .as_ref()
            .expect("refresh request should be JSON");
        assert_eq!(body["grant_type"], "refresh_token");
        assert_eq!(body["refresh_token"], "sk-ant-ort01-old");
        assert!(body.get("scope").is_none());
    }
}

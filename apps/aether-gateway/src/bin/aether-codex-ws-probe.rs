//! Credential-safe compatibility probe for the Codex Responses WebSocket path.
//!
//! This binary preserves the established Codex CLI and environment contract.
//! The common Responses WebSocket flow lives in `support/responses_ws_probe`;
//! this profile owns only Codex authentication and header requirements.

#[path = "support/responses_ws_probe.rs"]
mod responses_ws_probe;

use aether_gateway::{CODEX_CLIENT_ORIGINATOR, CODEX_CLIENT_USER_AGENT};
use clap::Parser;
use http::header::{AUTHORIZATION, USER_AGENT};
use http::{HeaderMap, HeaderName, HeaderValue};
use responses_ws_probe::{
    bearer_authorization_value, required_env, resolve_probe_url, run_profile_probe, turn_timeout,
    ProbeArgs, ProbeConfig, ProbeFailure, ResponsesWebSocketProbeProfile,
};

const ACCESS_TOKEN_ENV: &str = "AETHER_CODEX_WS_PROBE_ACCESS_TOKEN";
const ACCOUNT_ID_ENV: &str = "AETHER_CODEX_WS_PROBE_ACCOUNT_ID";
const MODEL_ENV: &str = "AETHER_CODEX_WS_PROBE_MODEL";
const URL_ENV: &str = "AETHER_CODEX_WS_PROBE_URL";

#[derive(Parser)]
#[command(
    name = "aether-codex-ws-probe",
    about = "Verify a Codex Responses WebSocket endpoint without exposing credentials"
)]
struct Args {
    /// WebSocket endpoint. If omitted, AETHER_CODEX_WS_PROBE_URL is used.
    #[arg(long)]
    url: Option<String>,

    /// Per-turn receive timeout in seconds.
    #[arg(long, default_value_t = 20, value_parser = clap::value_parser!(u64).range(1..=120))]
    timeout_secs: u64,
}

impl From<Args> for ProbeArgs {
    fn from(args: Args) -> Self {
        Self {
            url: args.url,
            timeout_secs: args.timeout_secs,
        }
    }
}

struct CodexResponsesProbeProfile;

impl ResponsesWebSocketProbeProfile for CodexResponsesProbeProfile {
    fn build_config(args: &ProbeArgs) -> Result<ProbeConfig, ProbeFailure> {
        let url = resolve_probe_url(args, URL_ENV, None)?;
        let access_token = required_env(ACCESS_TOKEN_ENV)?;
        let account_id = required_env(ACCOUNT_ID_ENV)?;
        let model = required_env(MODEL_ENV)?;
        Ok(ProbeConfig::new(
            url,
            model,
            turn_timeout(args),
            handshake_headers(&access_token, &account_id)?,
            Self::sent_header_names(),
        ))
    }

    fn sent_header_names() -> Vec<&'static str> {
        vec![
            "authorization",
            "chatgpt-account-id",
            "user-agent",
            "originator",
        ]
    }
}

fn handshake_headers(access_token: &str, account_id: &str) -> Result<HeaderMap, ProbeFailure> {
    let account_id =
        HeaderValue::from_str(account_id).map_err(|_| ProbeFailure::MissingConfiguration)?;
    let mut headers = HeaderMap::new();
    headers.insert(AUTHORIZATION, bearer_authorization_value(access_token)?);
    headers.insert(HeaderName::from_static("chatgpt-account-id"), account_id);
    headers.insert(
        USER_AGENT,
        HeaderValue::from_static(CODEX_CLIENT_USER_AGENT),
    );
    headers.insert(
        HeaderName::from_static("originator"),
        HeaderValue::from_static(CODEX_CLIENT_ORIGINATOR),
    );
    Ok(headers)
}

#[tokio::main]
async fn main() {
    let exit_code = run_profile_probe::<CodexResponsesProbeProfile>(Args::parse().into()).await;
    if exit_code != 0 {
        std::process::exit(exit_code);
    }
}

#[cfg(test)]
mod tests {
    use http::header::{AUTHORIZATION, USER_AGENT};

    use super::{handshake_headers, CodexResponsesProbeProfile, ResponsesWebSocketProbeProfile};

    #[test]
    fn codex_profile_keeps_its_required_handshake_headers() {
        let headers =
            handshake_headers("test-token", "test-account").expect("headers should build");
        assert!(headers.contains_key(AUTHORIZATION));
        assert!(headers.contains_key("chatgpt-account-id"));
        assert!(headers.contains_key(USER_AGENT));
        assert!(headers.contains_key("originator"));
        assert_eq!(
            CodexResponsesProbeProfile::sent_header_names(),
            vec![
                "authorization",
                "chatgpt-account-id",
                "user-agent",
                "originator",
            ]
        );
    }
}

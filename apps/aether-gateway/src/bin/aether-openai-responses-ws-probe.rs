//! Credential-safe compatibility probe for the official OpenAI Responses
//! WebSocket endpoint.
//!
//! This profile uses standard API-key Bearer authentication and shares the
//! protocol flow with the Codex probe without inheriting Codex-specific
//! account headers or quota assumptions.

#[path = "support/responses_ws_probe.rs"]
mod responses_ws_probe;

use clap::Parser;
use http::header::AUTHORIZATION;
use http::HeaderMap;
use responses_ws_probe::{
    bearer_authorization_value, required_env, resolve_probe_url, run_profile_probe, turn_timeout,
    ProbeArgs, ProbeConfig, ProbeFailure, ResponsesWebSocketProbeProfile,
};

const API_KEY_ENV: &str = "AETHER_OPENAI_WS_PROBE_API_KEY";
const MODEL_ENV: &str = "AETHER_OPENAI_WS_PROBE_MODEL";
const URL_ENV: &str = "AETHER_OPENAI_WS_PROBE_URL";
const DEFAULT_URL: &str = "wss://api.openai.com/v1/responses";

#[derive(Parser)]
#[command(
    name = "aether-openai-responses-ws-probe",
    about = "Verify an OpenAI Responses WebSocket endpoint without exposing credentials"
)]
struct Args {
    /// WebSocket endpoint. If omitted, AETHER_OPENAI_WS_PROBE_URL or the
    /// official OpenAI endpoint is used.
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

struct OpenAiResponsesProbeProfile;

impl ResponsesWebSocketProbeProfile for OpenAiResponsesProbeProfile {
    fn build_config(args: &ProbeArgs) -> Result<ProbeConfig, ProbeFailure> {
        let url = resolve_probe_url(args, URL_ENV, Some(DEFAULT_URL))?;
        let api_key = required_env(API_KEY_ENV)?;
        let model = required_env(MODEL_ENV)?;
        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, bearer_authorization_value(&api_key)?);
        Ok(ProbeConfig::new(
            url,
            model,
            turn_timeout(args),
            headers,
            Self::sent_header_names(),
        ))
    }

    fn sent_header_names() -> Vec<&'static str> {
        vec!["authorization"]
    }
}

#[tokio::main]
async fn main() {
    let exit_code = run_profile_probe::<OpenAiResponsesProbeProfile>(Args::parse().into()).await;
    if exit_code != 0 {
        std::process::exit(exit_code);
    }
}

#[cfg(test)]
mod tests {
    use http::header::AUTHORIZATION;

    use super::{
        bearer_authorization_value, responses_ws_probe::parse_probe_url,
        OpenAiResponsesProbeProfile, ResponsesWebSocketProbeProfile, DEFAULT_URL,
    };

    #[test]
    fn openai_profile_exposes_only_standard_bearer_authentication() {
        let authorization = bearer_authorization_value("test-key").expect("header should build");
        assert_eq!(authorization.to_str().ok(), Some("Bearer test-key"));
        assert_eq!(
            OpenAiResponsesProbeProfile::sent_header_names(),
            vec![AUTHORIZATION.as_str()]
        );
    }

    #[test]
    fn openai_profile_uses_the_official_responses_websocket_endpoint_by_default() {
        let url = parse_probe_url(DEFAULT_URL).expect("default OpenAI endpoint should be valid");
        assert_eq!(url.as_str(), DEFAULT_URL);
    }
}

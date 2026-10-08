//! Provider wire templates. Dynamic CLI versions are independent of SDK and
//! browser fingerprints; pinned templates are changed only after verification.
use aether_ai_formats::client_profile::validate_client_version;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionedClientIdentity {
    pub version: String,
    pub user_agent: String,
}

impl VersionedClientIdentity {
    pub fn new(version: &str, agent: &str, suffix: &str) -> Result<Self, &'static str> {
        let version = validate_client_version(version)?;
        Ok(Self {
            version: version.to_owned(),
            user_agent: format!("{agent}/{version}{suffix}"),
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PinnedClientIdentity {
    pub version: &'static str,
    pub user_agent: &'static str,
}

pub const GEMINI_CLI: PinnedClientIdentity = PinnedClientIdentity {
    version: "0.1.5",
    user_agent: "GeminiCLI/0.1.5 (Windows; AMD64)",
};
// Legacy standard-models fallback; dedicated Kiro requests use OAuth SDK identity.
pub const KIRO_MODELS_LEGACY_USER_AGENT: &str = "claude-code/1.0.1";

pub const ANTIGRAVITY: PinnedClientIdentity = PinnedClientIdentity {
    version: "4.3.0",
    user_agent: "vscode/1.X.X (Antigravity/4.3.0)",
};
pub const WINDSURF: PinnedClientIdentity = PinnedClientIdentity {
    version: "1.9600.41",
    user_agent: "windsurf/1.9600.41",
};
// Verified browser release tuple, shared by inference and quota requests.
// Do not update these independently or derive a web build from a CLI release.
pub const CHATGPT_WEB_USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/143.0.0.0 Safari/537.36 Edg/143.0.0.0";
pub const CHATGPT_WEB_CLIENT_VERSION: &str = "prod-be885abbfcfe7b1f511e88b3003d9ee44757fbad";
pub const CHATGPT_WEB_BUILD_NUMBER: &str = "5955942";
pub const CHATGPT_WEB_SEC_CH_UA: &str =
    r#""Microsoft Edge";v="143", "Chromium";v="143", "Not A(Brand";v="24""#;
pub const CHATGPT_WEB_BROWSER_PROFILE: &str = "chrome143";

// Kiro authentication and request adapters already share the OAuth model.
pub use aether_oauth::provider::providers::{
    DEFAULT_KIRO_VERSION, DEFAULT_NODE_VERSION, DEFAULT_SYSTEM_VERSION,
};

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pinned_headers_and_body_versions_are_calibrated_together() {
        for identity in [ANTIGRAVITY, WINDSURF] {
            assert!(identity.user_agent.contains(identity.version));
        }
    }
}

use crate::client_identity::VersionedClientIdentity;
use aether_ai_formats::client_profile::ClientProfileStore;
use std::sync::OnceLock;

pub const GEMINI_CLI_BUILTIN_VERSION: &str = crate::client_identity::GEMINI_CLI.version;
// Keep the established template; a version release does not change platform identity.
fn identity(version: &str) -> Result<VersionedClientIdentity, &'static str> {
    VersionedClientIdentity::new(version, "GeminiCLI", " (Windows; AMD64)")
}
static ACTIVE: OnceLock<ClientProfileStore<VersionedClientIdentity>> = OnceLock::new();
fn active() -> &'static ClientProfileStore<VersionedClientIdentity> {
    ACTIVE.get_or_init(|| {
        ClientProfileStore::new(
            identity(GEMINI_CLI_BUILTIN_VERSION).expect("valid built-in Gemini identity"),
        )
    })
}
pub fn gemini_cli_client_version() -> String {
    active().snapshot().version.clone()
}
pub fn gemini_cli_client_user_agent() -> String {
    active().snapshot().user_agent.clone()
}
pub fn set_gemini_cli_client_version(version: &str) -> Result<String, &'static str> {
    Ok(active().publish(identity(version)?).version.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn version_updates_preserve_the_existing_platform_template() {
        let profile = identity("0.62.0").unwrap();
        assert_eq!(profile.user_agent, "GeminiCLI/0.62.0 (Windows; AMD64)");
        assert!(identity("0.62.0\nx: y").is_err());
        assert_eq!(
            identity(GEMINI_CLI_BUILTIN_VERSION).unwrap().user_agent,
            super::super::GEMINI_CLI_USER_AGENT
        );
    }
}

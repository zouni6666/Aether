use std::collections::{BTreeMap, BTreeSet};

use aether_ai_formats::ApiOperation;

pub const CLAUDE_CODE_CONTEXT_MANAGEMENT_BETA: &str = "context-management-2025-06-27";

const MESSAGE_BETAS_2026_04: &[&str] = &[
    "claude-code-20250219",
    "oauth-2025-04-20",
    "interleaved-thinking-2025-05-14",
    "prompt-caching-scope-2026-01-05",
    "effort-2025-11-24",
    CLAUDE_CODE_CONTEXT_MANAGEMENT_BETA,
    "extended-cache-ttl-2025-04-11",
];
const COUNT_TOKENS_BETAS_2026_04: &[&str] = &[
    "claude-code-20250219",
    "oauth-2025-04-20",
    "interleaved-thinking-2025-05-14",
    "prompt-caching-scope-2026-01-05",
    "effort-2025-11-24",
    CLAUDE_CODE_CONTEXT_MANAGEMENT_BETA,
    "extended-cache-ttl-2025-04-11",
    "token-counting-2024-11-01",
];
const DROPPED_BETAS_2026_04: &[&str] = &[];
const BODY_CAPABILITY_GATES_2026_04: &[ClaudeCodeBodyCapabilityGate] =
    &[ClaudeCodeBodyCapabilityGate {
        body_field: "context_management",
        beta_token: CLAUDE_CODE_CONTEXT_MANAGEMENT_BETA,
        inject_when_thinking_enabled: true,
        default_edit_type: Some("clear_thinking_20251015"),
    }];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClaudeCodeTransportIdentityProfileVersion {
    V2026_04,
}

impl ClaudeCodeTransportIdentityProfileVersion {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::V2026_04 => "2026-04",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClaudeCodeBodyCapabilityGate {
    pub body_field: &'static str,
    pub beta_token: &'static str,
    pub inject_when_thinking_enabled: bool,
    pub default_edit_type: Option<&'static str>,
}

/// Versioned upstream identity used when Aether is intentionally acting as a
/// Claude Code transport. Native Anthropic transports never resolve this
/// profile and therefore keep their original headers and body untouched.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClaudeCodeTransportIdentityProfile {
    version: ClaudeCodeTransportIdentityProfileVersion,
    transport_profile_id: &'static str,
    anthropic_version: &'static str,
    cli_version: &'static str,
    stainless_lang: &'static str,
    stainless_package_version: &'static str,
    stainless_os: &'static str,
    stainless_arch: &'static str,
    stainless_runtime: &'static str,
    stainless_runtime_version: &'static str,
    stainless_retry_count: &'static str,
    stainless_timeout: &'static str,
    message_required_betas: &'static [&'static str],
    count_tokens_required_betas: &'static [&'static str],
    preserve_incoming_betas: bool,
    dropped_betas: &'static [&'static str],
    body_capability_gates: &'static [ClaudeCodeBodyCapabilityGate],
}

pub const CLAUDE_CODE_TRANSPORT_IDENTITY_2026_04: ClaudeCodeTransportIdentityProfile =
    ClaudeCodeTransportIdentityProfile {
        version: ClaudeCodeTransportIdentityProfileVersion::V2026_04,
        transport_profile_id: "claude_code_nodejs",
        anthropic_version: "2023-06-01",
        cli_version: "2.1.161",
        stainless_lang: "js",
        stainless_package_version: "0.94.0",
        stainless_os: "Linux",
        stainless_arch: "arm64",
        stainless_runtime: "node",
        stainless_runtime_version: "v24.3.0",
        stainless_retry_count: "0",
        stainless_timeout: "600",
        message_required_betas: MESSAGE_BETAS_2026_04,
        count_tokens_required_betas: COUNT_TOKENS_BETAS_2026_04,
        preserve_incoming_betas: true,
        dropped_betas: DROPPED_BETAS_2026_04,
        body_capability_gates: BODY_CAPABILITY_GATES_2026_04,
    };

pub const fn current_claude_code_transport_identity_profile(
) -> &'static ClaudeCodeTransportIdentityProfile {
    &CLAUDE_CODE_TRANSPORT_IDENTITY_2026_04
}

impl ClaudeCodeTransportIdentityProfile {
    pub const fn version(self) -> ClaudeCodeTransportIdentityProfileVersion {
        self.version
    }

    pub const fn transport_profile_id(self) -> &'static str {
        self.transport_profile_id
    }

    pub const fn cli_version(self) -> &'static str {
        self.cli_version
    }

    pub const fn billing_cli_version(self) -> &'static str {
        self.cli_version
    }

    pub fn user_agent(self) -> String {
        format!("claude-cli/{} (external, cli)", self.cli_version)
    }

    pub const fn stainless_package_version(self) -> &'static str {
        self.stainless_package_version
    }

    pub const fn stainless_lang(self) -> &'static str {
        self.stainless_lang
    }

    pub const fn stainless_os(self) -> &'static str {
        self.stainless_os
    }

    pub const fn stainless_arch(self) -> &'static str {
        self.stainless_arch
    }

    pub const fn stainless_runtime(self) -> &'static str {
        self.stainless_runtime
    }

    pub const fn stainless_runtime_version(self) -> &'static str {
        self.stainless_runtime_version
    }

    pub const fn stainless_retry_count(self) -> &'static str {
        self.stainless_retry_count
    }

    pub const fn stainless_timeout(self) -> &'static str {
        self.stainless_timeout
    }

    pub fn required_beta_tokens(self, operation: Option<ApiOperation>) -> &'static [&'static str] {
        if operation == Some(ApiOperation::ClaudeCountTokens) {
            self.count_tokens_required_betas
        } else {
            self.message_required_betas
        }
    }

    pub const fn preserves_incoming_betas(self) -> bool {
        self.preserve_incoming_betas
    }

    pub const fn dropped_beta_tokens(self) -> &'static [&'static str] {
        self.dropped_betas
    }

    pub const fn body_capability_gates(self) -> &'static [ClaudeCodeBodyCapabilityGate] {
        self.body_capability_gates
    }

    pub fn body_capability_gate(self, field: &str) -> Option<ClaudeCodeBodyCapabilityGate> {
        self.body_capability_gates
            .iter()
            .copied()
            .find(|gate| gate.body_field == field)
    }

    pub fn apply_fixed_headers(self, headers: &mut BTreeMap<String, String>, stream: bool) {
        for (name, value) in [
            ("accept", "application/json"),
            ("anthropic-version", self.anthropic_version),
            ("anthropic-dangerous-direct-browser-access", "true"),
            ("x-app", "cli"),
            ("x-stainless-lang", self.stainless_lang),
            (
                "x-stainless-package-version",
                self.stainless_package_version,
            ),
            ("x-stainless-os", self.stainless_os),
            ("x-stainless-arch", self.stainless_arch),
            ("x-stainless-runtime", self.stainless_runtime),
            (
                "x-stainless-runtime-version",
                self.stainless_runtime_version,
            ),
            ("x-stainless-retry-count", self.stainless_retry_count),
            ("x-stainless-timeout", self.stainless_timeout),
        ] {
            headers.insert(name.to_string(), value.to_string());
        }
        headers.insert("user-agent".to_string(), self.user_agent());
        if stream {
            headers.insert(
                "x-stainless-helper-method".to_string(),
                "stream".to_string(),
            );
        } else {
            headers.remove("x-stainless-helper-method");
        }
    }

    pub fn apply_beta_policy(
        self,
        headers: &mut BTreeMap<String, String>,
        operation: Option<ApiOperation>,
    ) {
        let incoming = headers.get("anthropic-beta").map(String::as_str);
        let merged = self.merge_beta_tokens(incoming, operation);
        if merged.is_empty() {
            headers.remove("anthropic-beta");
        } else {
            headers.insert("anthropic-beta".to_string(), merged);
        }
    }

    pub fn merge_beta_tokens(
        self,
        incoming: Option<&str>,
        operation: Option<ApiOperation>,
    ) -> String {
        let mut seen = BTreeSet::new();
        let mut merged = Vec::new();

        for token in self.required_beta_tokens(operation) {
            self.append_beta_token(&mut seen, &mut merged, token);
        }
        if self.preserve_incoming_betas {
            for token in incoming.unwrap_or_default().split(',') {
                self.append_beta_token(&mut seen, &mut merged, token);
            }
        }
        merged.join(",")
    }

    pub fn beta_header_enables_body_field(self, beta_header: &str, field: &str) -> bool {
        let Some(gate) = self.body_capability_gate(field) else {
            return true;
        };
        beta_header
            .split(',')
            .map(str::trim)
            .any(|token| token.eq_ignore_ascii_case(gate.beta_token))
    }

    fn append_beta_token(self, seen: &mut BTreeSet<String>, merged: &mut Vec<String>, token: &str) {
        let token = token.trim();
        if token.is_empty()
            || self
                .dropped_betas
                .iter()
                .any(|dropped| token.eq_ignore_ascii_case(dropped))
        {
            return;
        }
        if seen.insert(token.to_ascii_lowercase()) {
            merged.push(token.to_string());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::current_claude_code_transport_identity_profile;
    use aether_ai_formats::ApiOperation;

    #[test]
    fn profile_versions_cli_user_agent_stainless_and_billing_together() {
        let profile = *current_claude_code_transport_identity_profile();

        assert_eq!(profile.version().as_str(), "2026-04");
        assert_eq!(profile.cli_version(), "2.1.161");
        assert_eq!(profile.billing_cli_version(), profile.cli_version());
        assert_eq!(
            profile.user_agent(),
            format!("claude-cli/{} (external, cli)", profile.cli_version())
        );
        assert_eq!(profile.stainless_package_version(), "0.94.0");
        assert_eq!(profile.stainless_runtime_version(), "v24.3.0");
    }

    #[test]
    fn profile_preserves_context_1m_and_adds_operation_specific_betas() {
        let profile = *current_claude_code_transport_identity_profile();
        let messages = profile.merge_beta_tokens(Some("context-1m-2025-08-07,custom"), None);

        assert!(messages
            .split(',')
            .any(|token| token == "context-1m-2025-08-07"));
        assert!(messages.split(',').any(|token| token == "custom"));
        assert!(!messages
            .split(',')
            .any(|token| token == "token-counting-2024-11-01"));

        let count_tokens = profile.merge_beta_tokens(
            Some("context-1m-2025-08-07"),
            Some(ApiOperation::ClaudeCountTokens),
        );
        assert!(count_tokens
            .split(',')
            .any(|token| token == "token-counting-2024-11-01"));
        assert!(profile.dropped_beta_tokens().is_empty());
        assert!(profile.preserves_incoming_betas());
    }
}

mod auth;
mod fingerprint;
mod policy;
mod profile;
mod request;
mod url;

pub use auth::supports_local_claude_code_auth;
pub use fingerprint::{
    generate_fingerprint, generate_random_fingerprint, header_fingerprint_from_fingerprint,
    sanitize_fingerprint,
};
pub use policy::{
    local_claude_code_transport_unsupported_reason_with_network,
    supports_local_claude_code_transport_with_network,
};
pub use profile::{
    current_claude_code_transport_identity_profile, ClaudeCodeBodyCapabilityGate,
    ClaudeCodeTransportIdentityProfile, ClaudeCodeTransportIdentityProfileVersion,
    CLAUDE_CODE_CONTEXT_MANAGEMENT_BETA, CLAUDE_CODE_TRANSPORT_IDENTITY_2026_04,
};
pub use request::{
    build_claude_code_passthrough_headers, sanitize_claude_code_request_body,
    sanitize_claude_code_request_body_for_beta_header,
};
pub use url::build_claude_code_messages_url;

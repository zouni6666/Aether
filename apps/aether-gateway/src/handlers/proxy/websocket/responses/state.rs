//! Mutable state owned by one Responses WebSocket connection.
//!
//! The session loop is intentionally kept separate from these containers.  A
//! connection may survive many `response.create` turns, while the turn
//! lifecycle and upstream binding are replaced independently.

use std::collections::{BTreeMap, BTreeSet};
use tokio::task::JoinHandle;

use super::adapter::{ResponsesWebSocketDrainDirective, ResponsesWebSocketProtocolAdapter};
use super::binding::UpstreamBindingIdentity;
use super::redaction::ResponsesWebSocketRedactionRestorer;
use super::turn_state::ResponsesTurnState;
use crate::ai_serving::{AiExecutionDecision, ResponsesWebSocketBodyNormalization};

const EXHAUSTED_KEY_EXCLUSION_FALLBACK_SECONDS: u64 = 300;

/// All mutable state associated with the physical upstream connection.
pub(super) struct BoundResponsesConnection {
    pub(super) upstream: Option<wreq::ws::WebSocket>,
    pub(super) adapter: &'static dyn ResponsesWebSocketProtocolAdapter,
    pub(super) client_model: String,
    pub(super) provider_model: String,
    pub(super) decision_template: AiExecutionDecision,
    /// Reproduces this binding's provider-body normalization for continuation
    /// turns, which must not re-enter the planner. Replaced whenever the
    /// binding or its decision is replaced.
    pub(super) body_normalization: ResponsesWebSocketBodyNormalization,
    pub(super) binding_identity: UpstreamBindingIdentity,
    /// 这条连接上「有没有正在进行的 logical turn」的唯一事实来源。
    pub(super) turn_state: ResponsesTurnState,
    /// 这条连接迄今 mask 出来的映射，用于把 provider 事件里的占位符换回真实值。
    ///
    /// 刻意按连接持有而不是按 turn 持有：WS 的会话历史留在上游，continuation 只发
    /// 增量输入，所以后面几轮的响应可能回显更早那几轮的占位符（理由详见
    /// [`super::redaction`]）。上游重绑时不重置。
    pub(super) redaction_restorer: ResponsesWebSocketRedactionRestorer,
    pub(super) next_turn_index: u64,
    pub(super) upstream_response_headers: BTreeMap<String, String>,
    pub(super) pending_adapter_drain: Option<ResponsesWebSocketDrainDirective>,
    pub(super) pending_adapter_observation: Option<JoinHandle<()>>,
    pub(super) exhausted_exclusions: ExhaustedResponsesWebSocketExclusions,
    pub(super) pending_turn_finalization: Option<JoinHandle<()>>,
}

/// Connection-local fallback in addition to the distributed account breaker.
/// A key and its provider account are excluded until the upstream's reset
/// deadline (or a short fallback when the terminal payload lacks one), so an
/// unusually long-lived client socket does not keep it unavailable after the
/// quota has recovered.
#[derive(Debug, Default)]
pub(super) struct ExhaustedResponsesWebSocketExclusions {
    expires_at_by_key: BTreeMap<String, u64>,
    expires_at_by_codex_account: BTreeMap<String, u64>,
}

impl ExhaustedResponsesWebSocketExclusions {
    pub(super) fn exclude(
        &mut self,
        key_id: String,
        codex_account_id: Option<String>,
        reset_at_unix_secs: Option<u64>,
        now_unix_secs: u64,
    ) -> u64 {
        self.prune(now_unix_secs);
        let requested_expiry = reset_at_unix_secs
            .filter(|reset_at| *reset_at > now_unix_secs)
            .unwrap_or_else(|| {
                now_unix_secs.saturating_add(EXHAUSTED_KEY_EXCLUSION_FALLBACK_SECONDS)
            });
        let expiry = self
            .expires_at_by_key
            .entry(key_id)
            .and_modify(|existing| *existing = (*existing).max(requested_expiry))
            .or_insert(requested_expiry);
        if let Some(account_id) = codex_account_id {
            self.expires_at_by_codex_account
                .entry(account_id)
                .and_modify(|existing| *existing = (*existing).max(requested_expiry))
                .or_insert(requested_expiry);
        }
        *expiry
    }

    pub(super) fn codex_account_ids(&mut self, now_unix_secs: u64) -> BTreeSet<String> {
        self.prune(now_unix_secs);
        self.expires_at_by_codex_account.keys().cloned().collect()
    }

    pub(super) fn key_ids(&mut self, now_unix_secs: u64) -> BTreeSet<String> {
        self.prune(now_unix_secs);
        self.expires_at_by_key.keys().cloned().collect()
    }

    pub(super) fn len(&mut self, now_unix_secs: u64) -> usize {
        self.prune(now_unix_secs);
        self.expires_at_by_key.len() + self.expires_at_by_codex_account.len()
    }

    fn prune(&mut self, now_unix_secs: u64) {
        self.expires_at_by_key
            .retain(|_, expires_at| *expires_at > now_unix_secs);
        self.expires_at_by_codex_account
            .retain(|_, expires_at| *expires_at > now_unix_secs);
    }
}

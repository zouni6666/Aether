//! Short-lived ownership registry for Codex Live WebRTC sideband calls.
//!
//! Call IDs are opaque references to provider state. The raw ID is never
//! stored: the authenticated downstream principal and call ID are hashed into
//! a RuntimeState key, while the record contains only non-secret routing data.

use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use aether_runtime_state::{RuntimeLockLease, RuntimeState};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tokio::sync::{oneshot, watch};
use tokio::task::JoinHandle;

use crate::ai_serving::ResponsesWebSocketPinnedCandidate;

use super::planner::{LiveAuthMode, PlannedLiveCandidate};
use super::protocol::validate_call_id;

const SCHEMA_VERSION: u16 = 2;
const RECORD_PREFIX: &str = "codex_live:call:v2:";
const RECORD_DOMAIN: &[u8] = b"aether-codex-live-call-v2";
const INDEX_PREFIX: &str = "codex_live:call_index:v2:";
const INDEX_DOMAIN: &[u8] = b"aether-codex-live-call-index-v2";
const LOCK_PREFIX: &str = "codex_live:call_lock:v2:";
const SIDEBAND_LOCK_PREFIX: &str = "codex_live:sideband_lock:v1:";
const SIDEBAND_LOCK_DOMAIN: &[u8] = b"aether-codex-live-sideband-lock-v1";
const RECORD_TTL: Duration = Duration::from_secs(2 * 60 * 60);
const EXPIRED_LOOKUP_GRACE: Duration = Duration::from_secs(5 * 60);
const LOCK_TTL: Duration = Duration::from_secs(2);
const SIDEBAND_LOCK_TTL: Duration = Duration::from_secs(30);
const LOCK_ACQUIRE_TIMEOUT: Duration = Duration::from_millis(250);
const COMMIT_VERIFY_TIMEOUT: Duration = Duration::from_millis(250);
const LOCK_INITIAL_RETRY: Duration = Duration::from_millis(5);
const LOCK_MAX_RETRY: Duration = Duration::from_millis(50);
const LOCK_OWNER: &str = "codex_live_call_registry";
const SIDEBAND_LOCK_OWNER: &str = "codex_live_sideband_attachment";
const MAX_RECORDS_PER_PRINCIPAL: usize = 64;
const MAX_SERIALIZED_RECORD_BYTES: usize = 4 * 1024;
const MAX_PRINCIPAL_BYTES: usize = 256;
const MAX_RECORD_ID_BYTES: usize = 256;
const LIVE_LOG_TARGET: &str = "aether_gateway::handlers::proxy::codex_live";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RegisterCommitState {
    Direct,
    VerifiedAfterError,
    Uncommitted,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum LiveCallLookup {
    Found(LiveCallBinding),
    Missing,
    Expired,
}

#[derive(Debug, thiserror::Error)]
pub(super) enum LiveCallRegistryError {
    #[error("invalid Codex Live call identity: {0}")]
    InvalidIdentity(&'static str),
    #[error("invalid Codex Live call binding: {0}")]
    InvalidRecord(&'static str),
    #[error("Codex Live call binding serialization failed")]
    Serialization(#[source] serde_json::Error),
    #[error("Codex Live call registry contains corrupt data")]
    CorruptRecord(#[source] serde_json::Error),
    #[error("Codex Live call binding is too large")]
    RecordTooLarge,
    #[error("Codex Live call ownership conflicts with an existing binding")]
    OwnershipConflict,
    #[error("Codex Live call capacity lock is busy")]
    CapacityLockBusy,
    #[error("Codex Live call already has an active sideband attachment")]
    SidebandAlreadyAttached,
    #[error("Codex Live call registry storage is unavailable")]
    Storage(#[source] aether_runtime_state::DataLayerError),
}

impl LiveCallRegistryError {
    pub(super) const fn kind(&self) -> &'static str {
        match self {
            Self::InvalidIdentity(_) => "invalid_identity",
            Self::InvalidRecord(_) => "invalid_record",
            Self::Serialization(_) => "serialization_failed",
            Self::CorruptRecord(_) => "corrupt_record",
            Self::RecordTooLarge => "record_too_large",
            Self::OwnershipConflict => "ownership_conflict",
            Self::CapacityLockBusy => "capacity_lock_busy",
            Self::SidebandAlreadyAttached => "sideband_already_attached",
            Self::Storage(_) => "storage_unavailable",
        }
    }
}

/// Exclusive, renewable ownership of one authenticated Live sideband call.
///
/// The runtime lock key contains only a domain-separated digest. Call IDs and
/// downstream principal identifiers are never stored in the lease key.
pub(super) struct LiveSidebandLease {
    runtime_state: Arc<RuntimeState>,
    lease: RuntimeLockLease,
    renewal_cancel: Option<oneshot::Sender<()>>,
    renewal_task: Option<JoinHandle<()>>,
    health: watch::Receiver<LiveSidebandLeaseHealth>,
    armed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LiveSidebandLeaseHealth {
    Healthy,
    OwnershipLost,
    StorageUnavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum LiveSidebandLeaseLoss {
    OwnershipLost,
    StorageUnavailable,
}

impl LiveSidebandLeaseLoss {
    pub(super) const fn kind(self) -> &'static str {
        match self {
            Self::OwnershipLost => "ownership_lost",
            Self::StorageUnavailable => "storage_unavailable",
        }
    }
}

impl LiveSidebandLease {
    async fn new(runtime_state: Arc<RuntimeState>, lease: RuntimeLockLease, ttl: Duration) -> Self {
        let (renewal_cancel, mut cancel) = oneshot::channel();
        let (renewal_started, started) = oneshot::channel();
        let (health_tx, health) = watch::channel(LiveSidebandLeaseHealth::Healthy);
        let renewal_state = Arc::clone(&runtime_state);
        let renewal_lease = lease.clone();
        let interval = (ttl / 3).max(Duration::from_millis(1));
        let renewal_task = tokio::spawn(async move {
            let mut ticker =
                tokio::time::interval_at(tokio::time::Instant::now() + interval, interval);
            ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
            let _ = renewal_started.send(());
            loop {
                tokio::select! {
                    _ = &mut cancel => break,
                    _ = ticker.tick() => {
                        match renewal_state.lock_renew(&renewal_lease, ttl).await {
                            Ok(true) => {}
                            Ok(false) => {
                                health_tx.send_replace(LiveSidebandLeaseHealth::OwnershipLost);
                                tracing::warn!(
                                    target: LIVE_LOG_TARGET,
                                    event_name = "codex_live_sideband_lease_auto_renew_failed",
                                    log_type = "ops",
                                    error_kind = "ownership_lost",
                                    "Codex Live sideband ownership was lost during automatic renewal"
                                );
                                break;
                            }
                            Err(_) => {
                                health_tx.send_replace(LiveSidebandLeaseHealth::StorageUnavailable);
                                tracing::warn!(
                                    target: LIVE_LOG_TARGET,
                                    event_name = "codex_live_sideband_lease_auto_renew_failed",
                                    log_type = "ops",
                                    error_kind = "storage_unavailable",
                                    "Codex Live sideband ownership could not be renewed"
                                );
                                break;
                            }
                        }
                    }
                }
            }
        });
        let sideband_lease = Self {
            runtime_state,
            lease,
            renewal_cancel: Some(renewal_cancel),
            renewal_task: Some(renewal_task),
            health,
            armed: true,
        };
        // Do not return ownership to the caller until the renewal task has
        // registered its first deadline. This closes the acquire-to-first-poll
        // window on single-thread runtimes and heavily loaded executors.
        let _ = started.await;
        sideband_lease
    }

    pub(super) fn loss(&self) -> Option<LiveSidebandLeaseLoss> {
        match *self.health.borrow() {
            LiveSidebandLeaseHealth::Healthy => None,
            LiveSidebandLeaseHealth::OwnershipLost => Some(LiveSidebandLeaseLoss::OwnershipLost),
            LiveSidebandLeaseHealth::StorageUnavailable => {
                Some(LiveSidebandLeaseLoss::StorageUnavailable)
            }
        }
    }

    pub(super) async fn wait_for_loss(&self) -> LiveSidebandLeaseLoss {
        let mut health = self.health.clone();
        loop {
            match *health.borrow_and_update() {
                LiveSidebandLeaseHealth::Healthy => {}
                LiveSidebandLeaseHealth::OwnershipLost => {
                    return LiveSidebandLeaseLoss::OwnershipLost;
                }
                LiveSidebandLeaseHealth::StorageUnavailable => {
                    return LiveSidebandLeaseLoss::StorageUnavailable;
                }
            }
            if health.changed().await.is_err() {
                return LiveSidebandLeaseLoss::StorageUnavailable;
            }
        }
    }

    pub(super) async fn release(&mut self) -> Result<bool, LiveCallRegistryError> {
        self.stop_renewal().await;
        let released = self
            .runtime_state
            .lock_release(&self.lease)
            .await
            .map_err(LiveCallRegistryError::Storage)?;
        self.armed = false;
        Ok(released)
    }

    async fn stop_renewal(&mut self) {
        if let Some(cancel) = self.renewal_cancel.take() {
            let _ = cancel.send(());
        }
        if let Some(task) = self.renewal_task.take() {
            task.abort();
            let _ = task.await;
        }
    }
}

impl Drop for LiveSidebandLease {
    fn drop(&mut self) {
        if !self.armed {
            return;
        }
        if let Some(cancel) = self.renewal_cancel.take() {
            let _ = cancel.send(());
        }
        let renewal_task = self.renewal_task.take();
        let runtime_state = Arc::clone(&self.runtime_state);
        let lease = self.lease.clone();
        if let Ok(runtime) = tokio::runtime::Handle::try_current() {
            runtime.spawn(async move {
                if let Some(task) = renewal_task {
                    task.abort();
                    let _ = task.await;
                }
                let _ = runtime_state.lock_release(&lease).await;
            });
        } else if let Some(task) = renewal_task {
            task.abort();
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct LiveCallBinding {
    schema_version: u16,
    pinned_candidate: ResponsesWebSocketPinnedCandidate,
    client_model: String,
    provider_model: String,
    auth_mode: LiveAuthMode,
    routing_fingerprint: String,
    created_at_unix_ms: u64,
}

impl LiveCallBinding {
    pub(super) fn from_candidate(candidate: &PlannedLiveCandidate) -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            pinned_candidate: candidate.pinned_candidate.clone(),
            client_model: candidate.client_model.clone(),
            provider_model: candidate.provider_model.clone(),
            auth_mode: candidate.auth_mode,
            routing_fingerprint: candidate.routing_fingerprint.clone(),
            created_at_unix_ms: now_unix_ms(),
        }
    }

    pub(super) fn pinned_candidate(&self) -> &ResponsesWebSocketPinnedCandidate {
        &self.pinned_candidate
    }

    pub(super) fn client_model(&self) -> &str {
        self.client_model.as_str()
    }

    pub(super) fn matches_candidate(&self, candidate: &PlannedLiveCandidate) -> bool {
        self.pinned_candidate == candidate.pinned_candidate
            && self.client_model == candidate.client_model
            && self.provider_model == candidate.provider_model
            && self.auth_mode == candidate.auth_mode
            && self.routing_fingerprint == candidate.routing_fingerprint
    }

    fn validate(&self) -> Result<(), LiveCallRegistryError> {
        if self.schema_version != SCHEMA_VERSION {
            return Err(LiveCallRegistryError::InvalidRecord(
                "unsupported_schema_version",
            ));
        }
        for (value, error) in [
            (self.pinned_candidate.provider_id(), "invalid_provider_id"),
            (self.pinned_candidate.endpoint_id(), "invalid_endpoint_id"),
            (self.pinned_candidate.key_id(), "invalid_key_id"),
            (self.client_model.as_str(), "invalid_client_model"),
            (self.provider_model.as_str(), "invalid_provider_model"),
        ] {
            if value.trim().is_empty() || value.len() > MAX_RECORD_ID_BYTES {
                return Err(LiveCallRegistryError::InvalidRecord(error));
            }
        }
        if self.created_at_unix_ms == 0 {
            return Err(LiveCallRegistryError::InvalidRecord("invalid_created_at"));
        }
        if self.routing_fingerprint.len() != 64
            || !self
                .routing_fingerprint
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit())
        {
            return Err(LiveCallRegistryError::InvalidRecord(
                "invalid_routing_fingerprint",
            ));
        }
        Ok(())
    }
}

pub(super) struct LiveCallRegistry {
    runtime_state: Arc<RuntimeState>,
    ttl: Duration,
    max_records_per_principal: usize,
}

impl LiveCallRegistry {
    pub(super) fn new(runtime_state: Arc<RuntimeState>) -> Self {
        Self {
            runtime_state,
            ttl: RECORD_TTL,
            max_records_per_principal: MAX_RECORDS_PER_PRINCIPAL,
        }
    }

    #[cfg(test)]
    fn with_limits(runtime_state: Arc<RuntimeState>, ttl: Duration, max_records: usize) -> Self {
        Self {
            runtime_state,
            ttl,
            max_records_per_principal: max_records,
        }
    }

    pub(super) async fn register(
        &self,
        user_id: &str,
        api_key_id: &str,
        call_id: &str,
        binding: &LiveCallBinding,
    ) -> Result<(), LiveCallRegistryError> {
        binding.validate()?;
        let key = record_key(user_id, api_key_id, call_id)?;
        let index = index_key(user_id, api_key_id)?;
        let lock = lock_key(user_id, api_key_id)?;
        let serialized =
            serde_json::to_string(binding).map_err(LiveCallRegistryError::Serialization)?;
        if serialized.len() > MAX_SERIALIZED_RECORD_BYTES {
            return Err(LiveCallRegistryError::RecordTooLarge);
        }
        let lease = self.acquire_lock(lock.as_str()).await?;
        let result = self
            .register_locked(key.as_str(), index.as_str(), serialized, binding)
            .await;
        let exact_binding_committed = if result.is_err() {
            self.exact_binding_is_committed(key.as_str(), binding).await
        } else {
            false
        };
        let commit_state = classify_register_commit(&result, exact_binding_committed);
        let release = self.runtime_state.lock_release(&lease).await;

        if commit_state == RegisterCommitState::Uncommitted {
            return result;
        }

        if commit_state == RegisterCommitState::VerifiedAfterError {
            tracing::warn!(
                target: LIVE_LOG_TARGET,
                event_name = "codex_live_call_binding_commit_verified",
                log_type = "ops",
                register_error_kind = result
                    .as_ref()
                    .err()
                    .map_or("unknown", |error| error.kind()),
                "Codex Live accepted a binding after exact commit verification"
            );
        }
        if release.is_err() {
            tracing::warn!(
                target: LIVE_LOG_TARGET,
                event_name = "codex_live_call_capacity_lock_release_failed",
                log_type = "ops",
                error_kind = "storage_unavailable",
                "Codex Live binding was committed but its short-lived capacity lock could not be released"
            );
        }
        Ok(())
    }

    pub(super) async fn lookup(
        &self,
        user_id: &str,
        api_key_id: &str,
        call_id: &str,
    ) -> Result<Option<LiveCallBinding>, LiveCallRegistryError> {
        Ok(
            match self
                .lookup_with_status(user_id, api_key_id, call_id)
                .await?
            {
                LiveCallLookup::Found(binding) => Some(binding),
                LiveCallLookup::Missing | LiveCallLookup::Expired => None,
            },
        )
    }

    pub(super) async fn lookup_with_status(
        &self,
        user_id: &str,
        api_key_id: &str,
        call_id: &str,
    ) -> Result<LiveCallLookup, LiveCallRegistryError> {
        let key = record_key(user_id, api_key_id, call_id)?;
        if let Some(serialized) = self
            .runtime_state
            .kv_get(key.as_str())
            .await
            .map_err(LiveCallRegistryError::Storage)?
        {
            let binding = serde_json::from_str::<LiveCallBinding>(serialized.as_str())
                .map_err(LiveCallRegistryError::CorruptRecord)?;
            binding.validate()?;
            return Ok(LiveCallLookup::Found(binding));
        }

        let index = index_key(user_id, api_key_id)?;
        let indexed = self
            .runtime_state
            .score_range_by_min(index.as_str(), f64::NEG_INFINITY)
            .await
            .map_err(LiveCallRegistryError::Storage)?
            .iter()
            .any(|member| member == &key);
        Ok(if indexed {
            LiveCallLookup::Expired
        } else {
            LiveCallLookup::Missing
        })
    }

    pub(super) async fn acquire_sideband_attachment(
        &self,
        user_id: &str,
        api_key_id: &str,
        call_id: &str,
    ) -> Result<LiveSidebandLease, LiveCallRegistryError> {
        self.acquire_sideband_attachment_with_ttl(user_id, api_key_id, call_id, SIDEBAND_LOCK_TTL)
            .await
    }

    async fn acquire_sideband_attachment_with_ttl(
        &self,
        user_id: &str,
        api_key_id: &str,
        call_id: &str,
        ttl: Duration,
    ) -> Result<LiveSidebandLease, LiveCallRegistryError> {
        let key = sideband_lock_key(user_id, api_key_id, call_id)?;
        let Some(lease) = self
            .runtime_state
            .lock_try_acquire(key.as_str(), SIDEBAND_LOCK_OWNER, ttl)
            .await
            .map_err(LiveCallRegistryError::Storage)?
        else {
            return Err(LiveCallRegistryError::SidebandAlreadyAttached);
        };
        Ok(LiveSidebandLease::new(Arc::clone(&self.runtime_state), lease, ttl).await)
    }

    async fn acquire_lock(&self, key: &str) -> Result<RuntimeLockLease, LiveCallRegistryError> {
        let deadline = tokio::time::Instant::now() + LOCK_ACQUIRE_TIMEOUT;
        let mut retry = LOCK_INITIAL_RETRY;
        loop {
            if let Some(lease) = self
                .runtime_state
                .lock_try_acquire(key, LOCK_OWNER, LOCK_TTL)
                .await
                .map_err(LiveCallRegistryError::Storage)?
            {
                return Ok(lease);
            }
            let now = tokio::time::Instant::now();
            if now >= deadline {
                return Err(LiveCallRegistryError::CapacityLockBusy);
            }
            tokio::time::sleep(retry.min(deadline.saturating_duration_since(now))).await;
            retry = retry.saturating_mul(2).min(LOCK_MAX_RETRY);
        }
    }

    async fn exact_binding_is_committed(&self, key: &str, expected: &LiveCallBinding) -> bool {
        let stored =
            match tokio::time::timeout(COMMIT_VERIFY_TIMEOUT, self.runtime_state.kv_get(key)).await
            {
                Ok(Ok(Some(stored))) => stored,
                Ok(Ok(None) | Err(_)) | Err(_) => return false,
            };
        let Ok(actual) = serde_json::from_str::<LiveCallBinding>(stored.as_str()) else {
            return false;
        };
        actual.validate().is_ok() && actual == *expected
    }

    async fn register_locked(
        &self,
        key: &str,
        index: &str,
        serialized: String,
        binding: &LiveCallBinding,
    ) -> Result<(), LiveCallRegistryError> {
        if let Some(existing) = self
            .runtime_state
            .kv_get(key)
            .await
            .map_err(LiveCallRegistryError::Storage)?
        {
            let existing = serde_json::from_str::<LiveCallBinding>(existing.as_str())
                .map_err(LiveCallRegistryError::CorruptRecord)?;
            if existing != *binding {
                return Err(LiveCallRegistryError::OwnershipConflict);
            }
        }
        self.runtime_state
            .kv_set(key, serialized, Some(self.ttl))
            .await
            .map_err(LiveCallRegistryError::Storage)?;
        if let Err(error) = self
            .runtime_state
            .score_set(index, key, now_unix_ms() as f64)
            .await
        {
            let _ = self.runtime_state.kv_delete(key).await;
            return Err(LiveCallRegistryError::Storage(error));
        }
        if let Err(error) = self
            .runtime_state
            .key_expire(index, self.ttl.saturating_add(EXPIRED_LOOKUP_GRACE))
            .await
        {
            let _ = self.runtime_state.score_remove(index, key).await;
            let _ = self.runtime_state.kv_delete(key).await;
            return Err(LiveCallRegistryError::Storage(error));
        }
        let members = self
            .runtime_state
            .score_range_by_min(index, f64::NEG_INFINITY)
            .await
            .map_err(LiveCallRegistryError::Storage)?;
        let overflow = members.len().saturating_sub(self.max_records_per_principal);
        for oldest in members.into_iter().take(overflow) {
            self.runtime_state
                .kv_delete(oldest.as_str())
                .await
                .map_err(LiveCallRegistryError::Storage)?;
            self.runtime_state
                .score_remove(index, oldest.as_str())
                .await
                .map_err(LiveCallRegistryError::Storage)?;
        }
        Ok(())
    }
}

fn classify_register_commit(
    result: &Result<(), LiveCallRegistryError>,
    exact_binding_committed: bool,
) -> RegisterCommitState {
    match result {
        Ok(()) => RegisterCommitState::Direct,
        Err(LiveCallRegistryError::OwnershipConflict) => RegisterCommitState::Uncommitted,
        Err(_) if exact_binding_committed => RegisterCommitState::VerifiedAfterError,
        Err(_) => RegisterCommitState::Uncommitted,
    }
}

fn record_key(
    user_id: &str,
    api_key_id: &str,
    call_id: &str,
) -> Result<String, LiveCallRegistryError> {
    validate_principal(user_id, "invalid_user_id")?;
    validate_principal(api_key_id, "invalid_api_key_id")?;
    validate_call_id(call_id)
        .map_err(|_| LiveCallRegistryError::InvalidIdentity("invalid_call_id"))?;
    Ok(format!(
        "{RECORD_PREFIX}{}",
        digest(RECORD_DOMAIN, &[user_id, api_key_id, call_id])
    ))
}

fn index_key(user_id: &str, api_key_id: &str) -> Result<String, LiveCallRegistryError> {
    validate_principal(user_id, "invalid_user_id")?;
    validate_principal(api_key_id, "invalid_api_key_id")?;
    Ok(format!(
        "{INDEX_PREFIX}{}",
        digest(INDEX_DOMAIN, &[user_id, api_key_id])
    ))
}

fn lock_key(user_id: &str, api_key_id: &str) -> Result<String, LiveCallRegistryError> {
    let index = index_key(user_id, api_key_id)?;
    Ok(format!(
        "{LOCK_PREFIX}{}",
        index.strip_prefix(INDEX_PREFIX).unwrap_or(index.as_str())
    ))
}

fn sideband_lock_key(
    user_id: &str,
    api_key_id: &str,
    call_id: &str,
) -> Result<String, LiveCallRegistryError> {
    validate_principal(user_id, "invalid_user_id")?;
    validate_principal(api_key_id, "invalid_api_key_id")?;
    validate_call_id(call_id)
        .map_err(|_| LiveCallRegistryError::InvalidIdentity("invalid_call_id"))?;
    Ok(format!(
        "{SIDEBAND_LOCK_PREFIX}{}",
        digest(SIDEBAND_LOCK_DOMAIN, &[user_id, api_key_id, call_id])
    ))
}

fn validate_principal(value: &str, error: &'static str) -> Result<(), LiveCallRegistryError> {
    if value.is_empty() || value.len() > MAX_PRINCIPAL_BYTES {
        return Err(LiveCallRegistryError::InvalidIdentity(error));
    }
    Ok(())
}

fn digest(domain: &[u8], components: &[&str]) -> String {
    let mut digest = Sha256::new();
    digest.update(domain);
    for component in components {
        digest.update((component.len() as u64).to_be_bytes());
        digest.update(component.as_bytes());
    }
    format!("{:x}", digest.finalize())
}

fn now_unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use aether_runtime_state::{MemoryRuntimeStateConfig, RuntimeState};

    use super::*;

    fn runtime_state() -> Arc<RuntimeState> {
        Arc::new(RuntimeState::memory(MemoryRuntimeStateConfig::default()))
    }

    fn binding(client_model: &str) -> LiveCallBinding {
        LiveCallBinding {
            schema_version: SCHEMA_VERSION,
            pinned_candidate: ResponsesWebSocketPinnedCandidate::new(
                "provider-1",
                "endpoint-1",
                "key-1",
            )
            .unwrap(),
            client_model: client_model.to_string(),
            provider_model: "provider-model".to_string(),
            auth_mode: LiveAuthMode::ChatGptOauth,
            routing_fingerprint: "a".repeat(64),
            created_at_unix_ms: now_unix_ms(),
        }
    }

    fn candidate_for_binding(binding: &LiveCallBinding) -> PlannedLiveCandidate {
        let execution: crate::ai_serving::AiExecutionDecision =
            serde_json::from_value(serde_json::json!({
                "action": "stream",
                "provider_id": binding.pinned_candidate.provider_id(),
                "endpoint_id": binding.pinned_candidate.endpoint_id(),
                "key_id": binding.pinned_candidate.key_id(),
                "provider_type": "codex",
                "upstream_url": "https://chatgpt.com/backend-api/codex/responses"
            }))
            .unwrap();
        PlannedLiveCandidate {
            execution,
            pinned_candidate: binding.pinned_candidate.clone(),
            client_model: binding.client_model.clone(),
            provider_model: binding.provider_model.clone(),
            auth_mode: binding.auth_mode,
            routing_fingerprint: binding.routing_fingerprint.clone(),
        }
    }

    #[tokio::test]
    async fn binding_is_scoped_to_the_authenticated_principal() {
        let state = runtime_state();
        let registry = LiveCallRegistry::new(Arc::clone(&state));
        registry
            .register("user-1", "api-key-1", "rtc_secret", &binding("global"))
            .await
            .unwrap();
        assert!(registry
            .lookup("user-1", "api-key-1", "rtc_secret")
            .await
            .unwrap()
            .is_some());
        assert!(registry
            .lookup("user-2", "api-key-1", "rtc_secret")
            .await
            .unwrap()
            .is_none());
        assert!(registry
            .lookup("user-1", "api-key-2", "rtc_secret")
            .await
            .unwrap()
            .is_none());
    }

    #[tokio::test]
    async fn capacity_evicts_the_oldest_binding() {
        let state = runtime_state();
        let registry =
            LiveCallRegistry::with_limits(Arc::clone(&state), Duration::from_secs(60), 2);
        for call_id in ["rtc_1", "rtc_2", "rtc_3"] {
            registry
                .register("user", "key", call_id, &binding(call_id))
                .await
                .unwrap();
            tokio::time::sleep(Duration::from_millis(2)).await;
        }
        assert!(registry
            .lookup("user", "key", "rtc_1")
            .await
            .unwrap()
            .is_none());
        assert!(registry
            .lookup("user", "key", "rtc_2")
            .await
            .unwrap()
            .is_some());
        assert!(registry
            .lookup("user", "key", "rtc_3")
            .await
            .unwrap()
            .is_some());
    }

    #[tokio::test]
    async fn ttl_expiry_removes_a_binding() {
        let state = runtime_state();
        let registry =
            LiveCallRegistry::with_limits(Arc::clone(&state), Duration::from_millis(5), 2);
        registry
            .register("user", "key", "rtc_expiring", &binding("global"))
            .await
            .unwrap();
        tokio::time::sleep(Duration::from_millis(20)).await;
        assert!(registry
            .lookup("user", "key", "rtc_expiring")
            .await
            .unwrap()
            .is_none());
        assert_eq!(
            registry
                .lookup_with_status("user", "key", "rtc_expiring")
                .await
                .unwrap(),
            LiveCallLookup::Expired
        );
    }

    #[tokio::test]
    async fn an_existing_call_id_cannot_be_rebound_to_another_candidate() {
        let state = runtime_state();
        let registry = LiveCallRegistry::new(Arc::clone(&state));
        let original = binding("global-a");
        registry
            .register("user", "key", "rtc_shared", &original)
            .await
            .unwrap();

        assert!(matches!(
            registry
                .register("user", "key", "rtc_shared", &binding("global-b"))
                .await,
            Err(LiveCallRegistryError::OwnershipConflict)
        ));
        assert_eq!(
            registry.lookup("user", "key", "rtc_shared").await.unwrap(),
            Some(original)
        );
    }

    #[test]
    fn routing_fingerprint_drift_invalidates_the_pinned_binding() {
        let binding = binding("global");
        let mut candidate = candidate_for_binding(&binding);
        assert!(binding.matches_candidate(&candidate));

        candidate.routing_fingerprint = "b".repeat(64);
        assert!(!binding.matches_candidate(&candidate));
    }

    #[test]
    fn registry_keys_hash_principal_and_raw_call_identifiers() {
        let record = record_key("user-private", "key-private", "rtc-private").unwrap();
        let index = index_key("user-private", "key-private").unwrap();
        let sideband = sideband_lock_key("user-private", "key-private", "rtc-private").unwrap();
        for value in [&record, &index, &sideband] {
            assert!(!value.contains("user-private"));
            assert!(!value.contains("key-private"));
            assert!(!value.contains("rtc-private"));
        }
        assert_eq!(record.len(), RECORD_PREFIX.len() + 64);
        assert_eq!(index.len(), INDEX_PREFIX.len() + 64);
        assert_eq!(sideband.len(), SIDEBAND_LOCK_PREFIX.len() + 64);
    }

    #[tokio::test]
    async fn sideband_attachment_is_exclusive_and_release_allows_reconnect() {
        let state = runtime_state();
        let registry = LiveCallRegistry::new(Arc::clone(&state));
        let mut first = registry
            .acquire_sideband_attachment("user", "key", "rtc_attach")
            .await
            .unwrap();

        assert!(matches!(
            registry
                .acquire_sideband_attachment("user", "key", "rtc_attach")
                .await,
            Err(LiveCallRegistryError::SidebandAlreadyAttached)
        ));
        assert!(first.release().await.unwrap());

        let mut reconnected = registry
            .acquire_sideband_attachment("user", "key", "rtc_attach")
            .await
            .unwrap();
        assert!(reconnected.release().await.unwrap());
    }

    #[tokio::test]
    async fn sideband_attachment_auto_renewal_starts_at_acquisition() {
        let state = runtime_state();
        let registry = LiveCallRegistry::new(Arc::clone(&state));
        let mut held = registry
            .acquire_sideband_attachment_with_ttl(
                "user",
                "key",
                "rtc_renew",
                Duration::from_secs(2),
            )
            .await
            .unwrap();

        // Keep waking the single-thread test runtime while crossing the
        // original TTL. Very small sub-second TTLs are not representative of
        // the production 30-second lease and become scheduler-flaky on loaded
        // CI hosts.
        for _ in 0..25 {
            tokio::time::sleep(Duration::from_millis(100)).await;
            assert_eq!(held.loss(), None);
        }
        assert!(matches!(
            registry
                .acquire_sideband_attachment("user", "key", "rtc_renew")
                .await,
            Err(LiveCallRegistryError::SidebandAlreadyAttached)
        ));
        assert!(held.release().await.unwrap());
    }

    #[tokio::test]
    async fn expired_owner_cannot_release_a_successor_sideband_lease() {
        let state = runtime_state();
        let registry = LiveCallRegistry::new(Arc::clone(&state));
        let mut expired = registry
            .acquire_sideband_attachment_with_ttl(
                "user",
                "key",
                "rtc_fenced",
                Duration::from_millis(25),
            )
            .await
            .unwrap();
        expired.stop_renewal().await;
        tokio::time::sleep(Duration::from_millis(75)).await;

        let mut successor = registry
            .acquire_sideband_attachment("user", "key", "rtc_fenced")
            .await
            .unwrap();
        assert!(!expired.release().await.unwrap());
        assert!(matches!(
            registry
                .acquire_sideband_attachment("user", "key", "rtc_fenced")
                .await,
            Err(LiveCallRegistryError::SidebandAlreadyAttached)
        ));
        assert!(successor.release().await.unwrap());
    }

    #[tokio::test]
    async fn invalid_call_identity_is_rejected_before_storage() {
        let state = runtime_state();
        let registry = LiveCallRegistry::new(Arc::clone(&state));
        for call_id in [".", "..", "rtc/escape"] {
            assert!(matches!(
                registry
                    .register("user", "key", call_id, &binding("global"))
                    .await,
                Err(LiveCallRegistryError::InvalidIdentity("invalid_call_id"))
            ));
            assert!(matches!(
                registry
                    .acquire_sideband_attachment("user", "key", call_id)
                    .await,
                Err(LiveCallRegistryError::InvalidIdentity("invalid_call_id"))
            ));
        }
    }

    #[test]
    fn register_commit_classification_is_fail_closed() {
        let success = Ok(());
        assert_eq!(
            classify_register_commit(&success, false),
            RegisterCommitState::Direct
        );

        let storage_error = Err(LiveCallRegistryError::Storage(
            aether_runtime_state::DataLayerError::UnexpectedValue("injected".to_string()),
        ));
        assert_eq!(
            classify_register_commit(&storage_error, true),
            RegisterCommitState::VerifiedAfterError
        );
        assert_eq!(
            classify_register_commit(&storage_error, false),
            RegisterCommitState::Uncommitted
        );

        let ownership_conflict = Err(LiveCallRegistryError::OwnershipConflict);
        assert_eq!(
            classify_register_commit(&ownership_conflict, true),
            RegisterCommitState::Uncommitted
        );
    }

    #[tokio::test]
    async fn exact_commit_verification_requires_a_valid_identical_binding() {
        let state = runtime_state();
        let registry = LiveCallRegistry::new(Arc::clone(&state));
        let key = record_key("user", "key", "rtc_verify").unwrap();
        let expected = binding("global");

        assert!(
            !registry
                .exact_binding_is_committed(key.as_str(), &expected)
                .await
        );

        state
            .kv_set(key.as_str(), "{not-json".to_string(), None)
            .await
            .unwrap();
        assert!(
            !registry
                .exact_binding_is_committed(key.as_str(), &expected)
                .await
        );

        state
            .kv_set(
                key.as_str(),
                serde_json::to_string(&binding("different")).unwrap(),
                None,
            )
            .await
            .unwrap();
        assert!(
            !registry
                .exact_binding_is_committed(key.as_str(), &expected)
                .await
        );

        state
            .kv_set(
                key.as_str(),
                serde_json::to_string(&expected).unwrap(),
                None,
            )
            .await
            .unwrap();
        assert!(
            registry
                .exact_binding_is_committed(key.as_str(), &expected)
                .await
        );
    }
}

//! Node-local, request-deduplicated activity for provider and requested-model analysis.
//!
//! RPM counts distinct requests entering upstream execution in the last 60 seconds;
//! it is never extrapolated from a shorter observation window. Concurrency follows
//! guard lifetimes, including streams, independently of that window. Expiration is
//! ordered rather than scanning request history on each lifecycle event.
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use chrono::{DateTime, Utc};
use serde_json::{json, Value};

const WINDOW_US: u64 = 60_000_000;
const MAX_REQUESTS: usize = 100_000;
const MAX_REQUEST_DIMENSIONS: usize = 200_000;
const MAX_LABEL_BYTES: usize = 512;

#[derive(Debug, Default)]
struct Counts {
    recent: u64,
    active: u64,
    provider_name: Option<Arc<str>>,
}

impl Counts {
    fn empty(&self) -> bool {
        self.recent == 0 && self.active == 0
    }
}

#[derive(Debug)]
struct ProviderRequest {
    active: u64,
}

#[derive(Debug)]
struct Request {
    model: Option<Arc<str>>,
    active: u64,
    providers: HashMap<Arc<str>, ProviderRequest>,
    idle_since: Option<u64>,
    cleanup_scheduled: bool,
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
enum Expiration {
    Model(Arc<str>),
    Provider(Arc<str>, Arc<str>),
    Request(Arc<str>),
}

#[derive(Debug, Default)]
struct History {
    through_us: u64,
    requests: HashMap<Arc<str>, Request>,
    providers: HashMap<Arc<str>, Counts>,
    models: HashMap<Option<Arc<str>>, Counts>,
    expirations: BinaryHeap<Reverse<(u64, Expiration)>>,
    request_dimensions: usize,
    untracked_active: u64,
    incomplete_until_us: u64,
}

impl History {
    fn advance(&mut self, now_us: u64) {
        self.through_us = self.through_us.max(now_us);
        while self
            .expirations
            .peek()
            .is_some_and(|Reverse((expires_at, _))| *expires_at <= self.through_us)
        {
            let Reverse((_, expiration)) = self.expirations.pop().expect("expiration exists");
            match expiration {
                Expiration::Model(request_id) => {
                    let Some(request) = self.requests.get_mut(&request_id) else {
                        continue;
                    };
                    if let Some(counts) = self.models.get_mut(&request.model) {
                        counts.recent = counts.recent.saturating_sub(1);
                        if counts.empty() {
                            self.models.remove(&request.model);
                        }
                    }
                }
                Expiration::Provider(request_id, provider_id) => {
                    let Some(request) = self.requests.get(&request_id) else {
                        continue;
                    };
                    if !request.providers.contains_key(&provider_id) {
                        continue;
                    }
                    if let Some(counts) = self.providers.get_mut(&provider_id) {
                        counts.recent = counts.recent.saturating_sub(1);
                        if counts.empty() {
                            self.providers.remove(&provider_id);
                        }
                    }
                }
                Expiration::Request(request_id) => {
                    let Some(request) = self.requests.get_mut(&request_id) else {
                        continue;
                    };
                    request.cleanup_scheduled = false;
                    if let Some(idle_since) = request.idle_since {
                        let expires_at = idle_since.saturating_add(WINDOW_US);
                        if expires_at <= self.through_us {
                            self.request_dimensions -= request.providers.len() + 1;
                            self.requests.remove(&request_id);
                        } else {
                            // A retry reused the record while its first cleanup was
                            // pending. Keep at most one cleanup entry per request.
                            request.cleanup_scheduled = true;
                            self.expirations
                                .push(Reverse((expires_at, Expiration::Request(request_id))));
                        }
                    }
                }
            }
        }
    }

    fn begin(
        &mut self,
        now_us: u64,
        request_id: &str,
        provider_id: &str,
        provider_name: Option<&str>,
        requested_model: Option<&str>,
    ) -> GuardIdentity {
        self.advance(now_us);
        let existing = self.requests.get(request_id);
        let new_request = existing.is_none();
        let new_provider = existing.is_none_or(|r| !r.providers.contains_key(provider_id));
        let new_dimensions = usize::from(new_request) + usize::from(new_provider);
        let valid_labels = !request_id.is_empty()
            && !provider_id.is_empty()
            && [
                Some(request_id),
                Some(provider_id),
                provider_name,
                requested_model,
            ]
            .into_iter()
            .flatten()
            .all(|label| label.len() <= MAX_LABEL_BYTES);
        if !valid_labels
            || (new_request && self.requests.len() >= MAX_REQUESTS)
            || self.request_dimensions.saturating_add(new_dimensions) > MAX_REQUEST_DIMENSIONS
        {
            // Telemetry must not affect admission. Explicitly mark incomplete
            // coverage instead of silently returning plausible but partial counts.
            self.untracked_active += 1;
            self.incomplete_until_us = self.through_us.saturating_add(WINDOW_US);
            return GuardIdentity::Untracked;
        }

        let request_id: Arc<str> = self
            .requests
            .get_key_value(request_id)
            .map(|(key, _)| Arc::clone(key))
            .unwrap_or_else(|| Arc::from(request_id));
        let request = self
            .requests
            .entry(Arc::clone(&request_id))
            .or_insert_with(|| Request {
                model: requested_model
                    .filter(|model| !model.is_empty())
                    .map(Arc::from),
                active: 0,
                providers: HashMap::new(),
                idle_since: None,
                cleanup_scheduled: false,
            });
        let model_counts = self.models.entry(request.model.clone()).or_default();
        if new_request {
            model_counts.recent += 1;
            self.expirations.push(Reverse((
                self.through_us.saturating_add(WINDOW_US),
                Expiration::Model(Arc::clone(&request_id)),
            )));
        }
        if request.active == 0 {
            model_counts.active += 1;
        }
        request.active += 1;
        request.idle_since = None;

        let provider_id: Arc<str> = request
            .providers
            .get_key_value(provider_id)
            .map(|(key, _)| Arc::clone(key))
            .unwrap_or_else(|| Arc::from(provider_id));
        let provider = request
            .providers
            .entry(Arc::clone(&provider_id))
            .or_insert(ProviderRequest { active: 0 });
        let provider_counts = self.providers.entry(Arc::clone(&provider_id)).or_default();
        if let Some(name) = provider_name.filter(|name| !name.is_empty()) {
            provider_counts.provider_name = Some(Arc::from(name));
        }
        if new_provider {
            provider_counts.recent += 1;
            self.expirations.push(Reverse((
                self.through_us.saturating_add(WINDOW_US),
                Expiration::Provider(Arc::clone(&request_id), Arc::clone(&provider_id)),
            )));
        }
        if provider.active == 0 {
            provider_counts.active += 1;
        }
        provider.active += 1;
        self.request_dimensions += new_dimensions;
        GuardIdentity::Tracked {
            request_id,
            provider_id,
        }
    }

    fn release(&mut self, now_us: u64, identity: GuardIdentity) {
        self.advance(now_us);
        let GuardIdentity::Tracked {
            request_id,
            provider_id,
        } = identity
        else {
            self.untracked_active = self.untracked_active.saturating_sub(1);
            return;
        };
        let Some(request) = self.requests.get_mut(&request_id) else {
            return;
        };
        let Some(provider) = request.providers.get_mut(&provider_id) else {
            return;
        };
        provider.active = provider.active.saturating_sub(1);
        if provider.active == 0 {
            if let Some(counts) = self.providers.get_mut(&provider_id) {
                counts.active = counts.active.saturating_sub(1);
                if counts.empty() {
                    self.providers.remove(&provider_id);
                }
            }
        }
        request.active = request.active.saturating_sub(1);
        if request.active == 0 {
            if let Some(counts) = self.models.get_mut(&request.model) {
                counts.active = counts.active.saturating_sub(1);
                if counts.empty() {
                    self.models.remove(&request.model);
                }
            }
            // Retain deduplication briefly after completion as failover may begin
            // after the old guard drops, including after a >60-second attempt.
            request.idle_since = Some(self.through_us);
            if !request.cleanup_scheduled {
                request.cleanup_scheduled = true;
                self.expirations.push(Reverse((
                    self.through_us.saturating_add(WINDOW_US),
                    Expiration::Request(request_id),
                )));
            }
        }
    }

    fn snapshot(&mut self, now_us: u64, started_at_us: i64) -> Value {
        self.advance(now_us);
        let mut providers: Vec<_> = self.providers.iter().collect();
        providers.sort_unstable_by_key(|(provider, _)| *provider);
        let mut models: Vec<_> = self.models.iter().collect();
        models.sort_unstable_by_key(|(model, _)| *model);
        json!({
            "observed_at": DateTime::from_timestamp_micros(started_at_us.saturating_add(self.through_us.min(i64::MAX as u64) as i64)),
            "observed_from": DateTime::from_timestamp_micros(started_at_us),
            "window_seconds": 60,
            "observed_window_seconds": (self.through_us as f64 / 1_000_000.0).min(60.0),
            "scope": {"kind": "node"},
            "measurement": "http_and_responses_websocket_requests",
            "coverage": if self.untracked_active > 0 || self.through_us < self.incomplete_until_us { "partial" } else { "complete" },
            "providers": providers.into_iter().map(|(id, counts)| json!({
                "provider_id": id.as_ref(),
                "provider": counts.provider_name.as_deref().unwrap_or(id.as_ref()),
                "requests_per_minute": counts.recent,
                "current_concurrency": counts.active,
            })).collect::<Vec<_>>(),
            "models": models.into_iter().map(|(model, counts)| json!({
                "model": model.as_deref(),
                "requests_per_minute": counts.recent,
                "current_concurrency": counts.active,
            })).collect::<Vec<_>>(),
        })
    }
}

#[derive(Debug)]
pub(crate) struct ExecutionActivity {
    started_at: Instant,
    started_at_us: i64,
    history: Mutex<History>,
}

impl Default for ExecutionActivity {
    fn default() -> Self {
        Self {
            started_at: Instant::now(),
            started_at_us: Utc::now().timestamp_micros(),
            history: Mutex::new(History::default()),
        }
    }
}

impl ExecutionActivity {
    fn elapsed_us(&self) -> u64 {
        self.started_at.elapsed().as_micros().min(u64::MAX as u128) as u64
    }

    pub(crate) fn begin(
        self: &Arc<Self>,
        request_id: &str,
        provider_id: &str,
        provider_name: Option<&str>,
        requested_model: Option<&str>,
    ) -> ExecutionActivityGuard {
        let identity = self
            .history
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .begin(
                self.elapsed_us(),
                request_id,
                provider_id,
                provider_name,
                requested_model,
            );
        ExecutionActivityGuard {
            activity: Arc::clone(self),
            identity: Some(identity),
        }
    }

    pub(crate) fn snapshot(&self) -> Value {
        self.history
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .snapshot(self.elapsed_us(), self.started_at_us)
    }
}

#[derive(Debug)]
enum GuardIdentity {
    Tracked {
        request_id: Arc<str>,
        provider_id: Arc<str>,
    },
    Untracked,
}

#[derive(Debug)]
pub(crate) struct ExecutionActivityGuard {
    activity: Arc<ExecutionActivity>,
    identity: Option<GuardIdentity>,
}

impl Drop for ExecutionActivityGuard {
    fn drop(&mut self) {
        if let Some(identity) = self.identity.take() {
            self.activity
                .history
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .release(self.activity.elapsed_us(), identity);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn begin(history: &mut History, at_us: u64, id: &str, provider: &str) -> GuardIdentity {
        history.begin(at_us, id, provider, Some(provider), Some("requested-model"))
    }

    fn value(history: &mut History, at_us: u64) -> Value {
        history.snapshot(at_us, 0)
    }

    #[test]
    fn rpm_has_an_exact_rolling_window_and_never_extrapolates_startup() {
        let mut history = History::default();
        let a = begin(&mut history, 0, "a", "provider");
        history.release(1, a);
        let b = begin(&mut history, 30_000_000, "b", "provider");
        history.release(30_000_001, b);
        let early = value(&mut history, 30_000_001);
        assert_eq!(early["providers"][0]["requests_per_minute"], 2);
        assert_eq!(early["coverage"], "complete");
        assert!(early["observed_window_seconds"].as_f64().unwrap() < 60.0);
        assert_eq!(
            value(&mut history, WINDOW_US - 1)["providers"][0]["requests_per_minute"],
            2
        );
        assert_eq!(
            value(&mut history, WINDOW_US)["providers"][0]["requests_per_minute"],
            1
        );
        assert!(value(&mut history, 90_000_000)["providers"]
            .as_array()
            .unwrap()
            .is_empty());
        assert_eq!(
            value(&mut history, 90_000_000)["observed_window_seconds"],
            60.0
        );
    }

    #[test]
    fn overlapping_guards_and_sequential_retries_count_one_request() {
        let mut history = History::default();
        let a = begin(&mut history, 0, "request", "provider");
        let b = begin(&mut history, 1, "request", "provider");
        let c = begin(&mut history, 2, "other-request", "provider");
        assert_eq!(
            value(&mut history, 2)["providers"][0]["current_concurrency"],
            2
        );
        assert_eq!(
            value(&mut history, 2)["providers"][0]["requests_per_minute"],
            2
        );
        history.release(3, a);
        assert_eq!(
            value(&mut history, 3)["models"][0]["current_concurrency"],
            2
        );
        history.release(4, b);
        history.release(5, c);
        let retry = begin(&mut history, 6, "request", "provider");
        let result = value(&mut history, 6);
        assert_eq!(result["providers"][0]["requests_per_minute"], 2);
        assert_eq!(result["models"][0]["current_concurrency"], 1);
        history.release(7, retry);
    }

    #[test]
    fn failover_counts_each_provider_but_deduplicates_the_requested_model() {
        let mut history = History::default();
        let first = begin(&mut history, 0, "request", "first");
        let second = begin(&mut history, 1, "request", "second");
        let result = value(&mut history, 2);
        assert_eq!(result["providers"].as_array().unwrap().len(), 2);
        assert_eq!(result["providers"][0]["requests_per_minute"], 1);
        assert_eq!(result["providers"][1]["current_concurrency"], 1);
        assert_eq!(result["models"][0]["requests_per_minute"], 1);
        assert_eq!(result["models"][0]["current_concurrency"], 1);
        history.release(3, first);
        history.release(4, second);
    }

    #[test]
    fn long_stream_retains_concurrency_and_retry_does_not_restart_model_rpm() {
        let mut history = History::default();
        let stream = begin(&mut history, 0, "request", "provider");
        let result = value(&mut history, 2 * WINDOW_US);
        assert_eq!(result["providers"][0]["requests_per_minute"], 0);
        assert_eq!(result["models"][0]["current_concurrency"], 1);
        history.release(2 * WINDOW_US + 1, stream);
        let retry = begin(&mut history, 2 * WINDOW_US + 2, "request", "provider");
        let result = value(&mut history, 2 * WINDOW_US + 2);
        assert_eq!(result["providers"][0]["requests_per_minute"], 0);
        assert_eq!(result["models"][0]["requests_per_minute"], 0);
        assert_eq!(result["models"][0]["current_concurrency"], 1);
        history.release(2 * WINDOW_US + 3, retry);
        assert!(value(&mut history, 3 * WINDOW_US + 3)["models"]
            .as_array()
            .unwrap()
            .is_empty());
        assert!(history.requests.is_empty());
        assert!(history.expirations.is_empty());
        assert_eq!(history.request_dimensions, 0);
    }

    #[test]
    fn cancellation_drop_releases_concurrency_but_keeps_rpm() {
        let activity = Arc::new(ExecutionActivity::default());
        let guard = activity.begin("request", "provider", Some("Provider name"), None);
        assert_eq!(
            activity.snapshot()["providers"][0]["current_concurrency"],
            1
        );
        drop(guard);
        let result = activity.snapshot();
        assert_eq!(result["providers"][0]["current_concurrency"], 0);
        assert_eq!(result["providers"][0]["requests_per_minute"], 1);
        assert_eq!(result["providers"][0]["provider"], "Provider name");
        assert!(result["models"][0]["model"].is_null());
    }

    #[test]
    fn retry_cleanup_entries_stay_bounded_and_idle_memory_is_released() {
        let mut history = History::default();
        for n in 0..1_000 {
            let guard = begin(&mut history, n, "request", "provider");
            history.release(n, guard);
        }
        assert_eq!(history.expirations.len(), 3);
        value(&mut history, WINDOW_US);
        assert_eq!(history.expirations.len(), 1);
        assert_eq!(history.requests.len(), 1);
        value(&mut history, WINDOW_US + 1_000);
        assert!(history.requests.is_empty());
        assert!(history.providers.is_empty());
        assert!(history.models.is_empty());
        assert!(history.expirations.is_empty());
        assert_eq!(history.request_dimensions, 0);
    }

    #[test]
    fn sampling_limits_report_incomplete_coverage_until_unobserved_work_expires() {
        let mut history = History::default();
        history.request_dimensions = MAX_REQUEST_DIMENSIONS;
        let untracked = begin(&mut history, 0, "request", "provider");
        assert_eq!(value(&mut history, 1)["coverage"], "partial");
        assert_eq!(value(&mut history, 2 * WINDOW_US)["coverage"], "partial");
        history.release(2 * WINDOW_US, untracked);
        assert_eq!(value(&mut history, 2 * WINDOW_US)["coverage"], "complete");
        history.request_dimensions = 0;
        let long_id = "x".repeat(MAX_LABEL_BYTES + 1);
        let untracked = begin(&mut history, 3 * WINDOW_US, &long_id, "provider");
        history.release(3 * WINDOW_US, untracked);
        assert_eq!(
            value(&mut history, 4 * WINDOW_US - 1)["coverage"],
            "partial"
        );
        assert_eq!(value(&mut history, 4 * WINDOW_US)["coverage"], "complete");
    }
}

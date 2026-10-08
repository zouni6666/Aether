use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use aether_data_contracts::repository::usage::StoredUsageDashboardAnalytics;

const FRESH_FOR: Duration = Duration::from_secs(5 * 60);
const FAILURE_BACKOFF: Duration = Duration::from_secs(10);

#[derive(Debug, Default)]
pub(crate) struct OverviewTotalCache {
    state: Mutex<CacheState>,
}

#[derive(Debug, Default)]
struct CacheState {
    value: Option<(Instant, Arc<StoredUsageDashboardAnalytics>)>,
    refreshing: bool,
    retry_after: Option<Instant>,
}

pub(crate) enum OverviewTotalRead {
    Pending,
    Failed,
    Ready {
        snapshot: Arc<StoredUsageDashboardAnalytics>,
        stale: bool,
    },
}

/// Owns the single refresh slot even if the request that launched it disconnects.
/// Dropping a cancelled or panicking worker also releases the slot with backoff.
pub(crate) struct OverviewTotalRefresh {
    cache: Arc<OverviewTotalCache>,
    completed: bool,
}

impl OverviewTotalCache {
    pub(crate) fn read(
        self: &Arc<Self>,
        now: Instant,
    ) -> (OverviewTotalRead, Option<OverviewTotalRefresh>) {
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        let fresh = state
            .value
            .as_ref()
            .is_some_and(|(at, _)| now.saturating_duration_since(*at) < FRESH_FOR);
        let retry_allowed = state.retry_after.is_none_or(|after| now >= after);
        let refresh = if !fresh && !state.refreshing && retry_allowed {
            state.refreshing = true;
            Some(OverviewTotalRefresh {
                cache: Arc::clone(self),
                completed: false,
            })
        } else {
            None
        };
        let result = match &state.value {
            Some((_, snapshot)) => OverviewTotalRead::Ready {
                snapshot: Arc::clone(snapshot),
                stale: !fresh,
            },
            None if state.refreshing => OverviewTotalRead::Pending,
            None => OverviewTotalRead::Failed,
        };
        (result, refresh)
    }
}

impl OverviewTotalRefresh {
    pub(crate) fn finish(mut self, snapshot: Option<StoredUsageDashboardAnalytics>, now: Instant) {
        let mut state = self
            .cache
            .state
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        state.refreshing = false;
        if let Some(snapshot) = snapshot {
            state.value = Some((now, Arc::new(snapshot)));
            state.retry_after = None;
        } else {
            state.retry_after = Some(now + FAILURE_BACKOFF);
        }
        self.completed = true;
    }
}

impl Drop for OverviewTotalRefresh {
    fn drop(&mut self) {
        if !self.completed {
            let mut state = self
                .cache
                .state
                .lock()
                .unwrap_or_else(|error| error.into_inner());
            state.refreshing = false;
            state.retry_after = Some(Instant::now() + FAILURE_BACKOFF);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snapshot() -> StoredUsageDashboardAnalytics {
        let mut snapshot = StoredUsageDashboardAnalytics::default();
        snapshot.total.generated_at = "2026-09-18T00:00:00Z".into();
        snapshot.total.read_revision = "revision-1".into();
        snapshot.total.summary.request_count = 42;
        snapshot
    }

    #[test]
    fn concurrent_cold_reads_claim_one_refresh() {
        let cache = Arc::new(OverviewTotalCache::default());
        let barrier = Arc::new(std::sync::Barrier::new(16));
        let now = Instant::now();
        let workers = (0..16)
            .map(|_| {
                let cache = Arc::clone(&cache);
                let barrier = Arc::clone(&barrier);
                std::thread::spawn(move || {
                    barrier.wait();
                    let (read, refresh) = cache.read(now);
                    assert!(matches!(read, OverviewTotalRead::Pending));
                    refresh
                })
            })
            .collect::<Vec<_>>();
        let mut refreshes = workers
            .into_iter()
            .filter_map(|worker| worker.join().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(refreshes.len(), 1);
        refreshes.pop().unwrap().finish(Some(snapshot()), now);
        let (read, refresh) = cache.read(now);
        assert!(matches!(
            read,
            OverviewTotalRead::Ready { stale: false, .. }
        ));
        assert!(refresh.is_none());
    }

    #[test]
    fn expiration_returns_original_snapshot_and_failed_refresh_preserves_it() {
        let cache = Arc::new(OverviewTotalCache::default());
        let now = Instant::now();
        cache.read(now).1.unwrap().finish(Some(snapshot()), now);
        assert!(cache
            .read(now + FRESH_FOR - Duration::from_secs(1))
            .1
            .is_none());
        let expired = now + FRESH_FOR;
        let (read, refresh) = cache.read(expired);
        let OverviewTotalRead::Ready {
            snapshot: old,
            stale: true,
        } = read
        else {
            panic!("expired success must remain visible")
        };
        assert_eq!(old.total.generated_at, "2026-09-18T00:00:00Z");
        assert_eq!(old.total.read_revision, "revision-1");
        assert!(cache.read(expired).1.is_none());
        refresh.unwrap().finish(None, expired);
        let (read, retry) = cache.read(expired + FAILURE_BACKOFF - Duration::from_secs(1));
        let OverviewTotalRead::Ready {
            snapshot: retained,
            stale: true,
        } = read
        else {
            panic!("failed refresh must retain stale success")
        };
        assert!(Arc::ptr_eq(&old, &retained));
        assert!(retry.is_none());
        assert!(cache.read(expired + FAILURE_BACKOFF).1.is_some());
    }

    #[test]
    fn cold_failure_and_worker_cancellation_back_off_before_retrying() {
        let cache = Arc::new(OverviewTotalCache::default());
        let now = Instant::now();
        cache.read(now).1.unwrap().finish(None, now);
        let (read, refresh) = cache.read(now + Duration::from_secs(9));
        assert!(matches!(read, OverviewTotalRead::Failed));
        assert!(refresh.is_none());
        let (read, refresh) = cache.read(now + FAILURE_BACKOFF);
        assert!(matches!(read, OverviewTotalRead::Pending));
        drop(refresh);
        let after_cancel = Instant::now();
        let (read, refresh) = cache.read(after_cancel);
        assert!(matches!(read, OverviewTotalRead::Failed));
        assert!(refresh.is_none());
        assert!(cache.read(after_cancel + FAILURE_BACKOFF).1.is_some());
    }
}

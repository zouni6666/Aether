use std::collections::HashSet;
use std::future::Future;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use aether_cache::ExpiringMap;
use aether_data::DataLayerError;
use aether_data_contracts::repository::candidate_selection::{
    MinimalCandidateSelectionReadRepository, StoredMinimalCandidateSelectionRow,
    StoredPoolKeyCandidateOrder, StoredPoolKeyCandidateRowsByKeyIdsQuery,
    StoredPoolKeyCandidateRowsQuery, StoredRequestedModelCandidateRowsQuery,
};
use async_trait::async_trait;
use tokio::sync::Notify;

const CANDIDATE_SELECTION_CACHE_TTL: Duration = Duration::from_secs(5);
const CANDIDATE_SELECTION_CACHE_MAX_ENTRIES: usize = 4096;

pub(super) struct CachedMinimalCandidateSelectionReadRepository {
    inner: Arc<dyn MinimalCandidateSelectionReadRepository>,
    entries: ExpiringMap<CandidateSelectionCacheKey, Vec<StoredMinimalCandidateSelectionRow>>,
    inflight: Mutex<HashSet<CandidateSelectionCacheKey>>,
    inflight_notify: Notify,
    epoch: AtomicU64,
}

impl CachedMinimalCandidateSelectionReadRepository {
    pub(super) fn new(inner: Arc<dyn MinimalCandidateSelectionReadRepository>) -> Self {
        Self {
            inner,
            entries: ExpiringMap::new(),
            inflight: Mutex::new(HashSet::new()),
            inflight_notify: Notify::new(),
            epoch: AtomicU64::new(0),
        }
    }

    async fn get_or_load<F, Fut>(
        &self,
        key: CandidateSelectionCacheKey,
        load: F,
    ) -> Result<Vec<StoredMinimalCandidateSelectionRow>, DataLayerError>
    where
        F: Fn() -> Fut,
        Fut: Future<Output = Result<Vec<StoredMinimalCandidateSelectionRow>, DataLayerError>>,
    {
        if let Some(rows) = self.entries.get_fresh(&key, CANDIDATE_SELECTION_CACHE_TTL) {
            return Ok(rows);
        }

        loop {
            let notified = self.inflight_notify.notified();
            match self.register_inflight(&key) {
                InflightRegistration::Bypass => return load().await,
                InflightRegistration::Follower => {
                    notified.await;
                    if let Some(rows) = self.entries.get_fresh(&key, CANDIDATE_SELECTION_CACHE_TTL)
                    {
                        return Ok(rows);
                    }
                    continue;
                }
                InflightRegistration::Leader => {}
            }

            let load_epoch = self.epoch.load(Ordering::Acquire);
            let result = load().await;
            if let Ok(rows) = &result {
                if load_epoch == self.epoch.load(Ordering::Acquire) {
                    self.entries.insert(
                        key.clone(),
                        rows.clone(),
                        CANDIDATE_SELECTION_CACHE_TTL,
                        CANDIDATE_SELECTION_CACHE_MAX_ENTRIES,
                    );
                }
            }
            self.finish_inflight(&key);
            return result;
        }
    }

    fn register_inflight(&self, key: &CandidateSelectionCacheKey) -> InflightRegistration {
        match self.inflight.lock() {
            Ok(mut inflight) => {
                if inflight.insert(key.clone()) {
                    InflightRegistration::Leader
                } else {
                    InflightRegistration::Follower
                }
            }
            Err(_) => InflightRegistration::Bypass,
        }
    }

    fn finish_inflight(&self, key: &CandidateSelectionCacheKey) {
        if let Ok(mut inflight) = self.inflight.lock() {
            inflight.remove(key);
        }
        self.inflight_notify.notify_waiters();
    }

    fn clear(&self) {
        self.epoch.fetch_add(1, Ordering::AcqRel);
        self.entries.clear();
    }
}

enum InflightRegistration {
    Leader,
    Follower,
    Bypass,
}

#[async_trait]
impl MinimalCandidateSelectionReadRepository for CachedMinimalCandidateSelectionReadRepository {
    fn clear_local_cache(&self) {
        self.clear();
        self.inner.clear_local_cache();
    }

    async fn list_for_exact_api_format(
        &self,
        api_format: &str,
    ) -> Result<Vec<StoredMinimalCandidateSelectionRow>, DataLayerError> {
        let key = CandidateSelectionCacheKey::ApiFormat {
            api_format: normalize_api_format_key(api_format),
        };
        self.get_or_load(key, || self.inner.list_for_exact_api_format(api_format))
            .await
    }

    async fn list_for_exact_api_format_and_global_model(
        &self,
        api_format: &str,
        global_model_name: &str,
    ) -> Result<Vec<StoredMinimalCandidateSelectionRow>, DataLayerError> {
        let key = CandidateSelectionCacheKey::ApiFormatAndGlobalModel {
            api_format: normalize_api_format_key(api_format),
            global_model_name: global_model_name.to_string(),
        };
        self.get_or_load(key, || {
            self.inner
                .list_for_exact_api_format_and_global_model(api_format, global_model_name)
        })
        .await
    }

    async fn list_for_exact_api_format_and_requested_model(
        &self,
        api_format: &str,
        requested_model_name: &str,
    ) -> Result<Vec<StoredMinimalCandidateSelectionRow>, DataLayerError> {
        let key = CandidateSelectionCacheKey::ApiFormatAndRequestedModel {
            api_format: normalize_api_format_key(api_format),
            requested_model_name: requested_model_name.to_string(),
        };
        self.get_or_load(key, || {
            self.inner
                .list_for_exact_api_format_and_requested_model(api_format, requested_model_name)
        })
        .await
    }

    async fn list_for_exact_api_format_and_requested_model_page(
        &self,
        query: &StoredRequestedModelCandidateRowsQuery,
    ) -> Result<Vec<StoredMinimalCandidateSelectionRow>, DataLayerError> {
        let key = CandidateSelectionCacheKey::RequestedModelPage {
            api_format: normalize_api_format_key(&query.api_format),
            requested_model_name: query.requested_model_name.clone(),
            offset: query.offset,
            limit: query.limit,
        };
        self.get_or_load(key, || {
            self.inner
                .list_for_exact_api_format_and_requested_model_page(query)
        })
        .await
    }

    async fn list_pool_key_rows_for_group(
        &self,
        query: &StoredPoolKeyCandidateRowsQuery,
    ) -> Result<Vec<StoredMinimalCandidateSelectionRow>, DataLayerError> {
        let key = CandidateSelectionCacheKey::PoolKeyRowsForGroup {
            api_format: normalize_api_format_key(&query.api_format),
            provider_id: query.provider_id.clone(),
            endpoint_id: query.endpoint_id.clone(),
            model_id: query.model_id.clone(),
            selected_provider_model_name: query.selected_provider_model_name.clone(),
            order: CandidateSelectionPoolOrderKey::from(&query.order),
            offset: query.offset,
            limit: query.limit,
        };
        self.get_or_load(key, || self.inner.list_pool_key_rows_for_group(query))
            .await
    }

    async fn list_pool_key_rows_for_group_key_ids(
        &self,
        query: &StoredPoolKeyCandidateRowsByKeyIdsQuery,
    ) -> Result<Vec<StoredMinimalCandidateSelectionRow>, DataLayerError> {
        let key = CandidateSelectionCacheKey::PoolKeyRowsForGroupKeyIds {
            api_format: normalize_api_format_key(&query.api_format),
            provider_id: query.provider_id.clone(),
            endpoint_id: query.endpoint_id.clone(),
            model_id: query.model_id.clone(),
            selected_provider_model_name: query.selected_provider_model_name.clone(),
            key_ids: query.key_ids.clone(),
        };
        self.get_or_load(key, || {
            self.inner.list_pool_key_rows_for_group_key_ids(query)
        })
        .await
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum CandidateSelectionCacheKey {
    ApiFormat {
        api_format: String,
    },
    ApiFormatAndGlobalModel {
        api_format: String,
        global_model_name: String,
    },
    ApiFormatAndRequestedModel {
        api_format: String,
        requested_model_name: String,
    },
    RequestedModelPage {
        api_format: String,
        requested_model_name: String,
        offset: u32,
        limit: u32,
    },
    PoolKeyRowsForGroup {
        api_format: String,
        provider_id: String,
        endpoint_id: String,
        model_id: String,
        selected_provider_model_name: String,
        order: CandidateSelectionPoolOrderKey,
        offset: u32,
        limit: u32,
    },
    PoolKeyRowsForGroupKeyIds {
        api_format: String,
        provider_id: String,
        endpoint_id: String,
        model_id: String,
        selected_provider_model_name: String,
        key_ids: Vec<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum CandidateSelectionPoolOrderKey {
    InternalPriority,
    Lru,
    CacheAffinity,
    SingleAccount,
    LoadBalance { seed: String },
}

impl From<&StoredPoolKeyCandidateOrder> for CandidateSelectionPoolOrderKey {
    fn from(order: &StoredPoolKeyCandidateOrder) -> Self {
        match order {
            StoredPoolKeyCandidateOrder::InternalPriority => Self::InternalPriority,
            StoredPoolKeyCandidateOrder::Lru => Self::Lru,
            StoredPoolKeyCandidateOrder::CacheAffinity => Self::CacheAffinity,
            StoredPoolKeyCandidateOrder::SingleAccount => Self::SingleAccount,
            StoredPoolKeyCandidateOrder::LoadBalance { seed } => {
                Self::LoadBalance { seed: seed.clone() }
            }
        }
    }
}

fn normalize_api_format_key(api_format: &str) -> String {
    crate::ai_serving::normalize_api_format_alias(api_format.trim())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicUsize;

    struct StubCandidateSelectionRepository {
        calls: AtomicUsize,
        delay: Duration,
    }

    impl StubCandidateSelectionRepository {
        fn new(delay: Duration) -> Self {
            Self {
                calls: AtomicUsize::new(0),
                delay,
            }
        }

        fn calls(&self) -> usize {
            self.calls.load(Ordering::SeqCst)
        }

        async fn load(&self) -> Result<Vec<StoredMinimalCandidateSelectionRow>, DataLayerError> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            if !self.delay.is_zero() {
                tokio::time::sleep(self.delay).await;
            }
            Ok(Vec::new())
        }
    }

    #[async_trait]
    impl MinimalCandidateSelectionReadRepository for StubCandidateSelectionRepository {
        async fn list_for_exact_api_format(
            &self,
            _api_format: &str,
        ) -> Result<Vec<StoredMinimalCandidateSelectionRow>, DataLayerError> {
            self.load().await
        }

        async fn list_for_exact_api_format_and_global_model(
            &self,
            _api_format: &str,
            _global_model_name: &str,
        ) -> Result<Vec<StoredMinimalCandidateSelectionRow>, DataLayerError> {
            self.load().await
        }

        async fn list_for_exact_api_format_and_requested_model(
            &self,
            _api_format: &str,
            _requested_model_name: &str,
        ) -> Result<Vec<StoredMinimalCandidateSelectionRow>, DataLayerError> {
            self.load().await
        }

        async fn list_for_exact_api_format_and_requested_model_page(
            &self,
            _query: &StoredRequestedModelCandidateRowsQuery,
        ) -> Result<Vec<StoredMinimalCandidateSelectionRow>, DataLayerError> {
            self.load().await
        }

        async fn list_pool_key_rows_for_group(
            &self,
            _query: &StoredPoolKeyCandidateRowsQuery,
        ) -> Result<Vec<StoredMinimalCandidateSelectionRow>, DataLayerError> {
            self.load().await
        }

        async fn list_pool_key_rows_for_group_key_ids(
            &self,
            _query: &StoredPoolKeyCandidateRowsByKeyIdsQuery,
        ) -> Result<Vec<StoredMinimalCandidateSelectionRow>, DataLayerError> {
            self.load().await
        }
    }

    #[tokio::test]
    async fn candidate_selection_cache_coalesces_concurrent_loads() {
        let inner = Arc::new(StubCandidateSelectionRepository::new(
            Duration::from_millis(25),
        ));
        let cache = Arc::new(CachedMinimalCandidateSelectionReadRepository::new(
            inner.clone(),
        ));
        let mut tasks = Vec::new();

        for _ in 0..16 {
            let cache = cache.clone();
            tasks.push(tokio::spawn(async move {
                cache.list_for_exact_api_format("openai").await.unwrap();
            }));
        }

        for task in tasks {
            task.await.unwrap();
        }

        assert_eq!(inner.calls(), 1);
    }

    #[tokio::test]
    async fn candidate_selection_cache_clear_invalidates_entries() {
        let inner = Arc::new(StubCandidateSelectionRepository::new(Duration::ZERO));
        let cache = CachedMinimalCandidateSelectionReadRepository::new(inner.clone());

        cache.list_for_exact_api_format("openai").await.unwrap();
        cache.list_for_exact_api_format("openai").await.unwrap();
        assert_eq!(inner.calls(), 1);

        cache.clear_local_cache();
        cache.list_for_exact_api_format("openai").await.unwrap();
        assert_eq!(inner.calls(), 2);
    }
}

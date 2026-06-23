use std::collections::HashSet;
use std::future::Future;
use std::hash::Hash;
use std::sync::Mutex as StdMutex;
use std::time::Duration;

use aether_cache::ExpiringMap;
use serde_json::Value;
use tokio::sync::Notify;

use crate::data::auth::GatewayAuthApiKeySnapshot;

const AUTH_RUNTIME_CACHE_MAX_ENTRIES: usize = 16_384;

#[derive(Debug)]
struct CacheSingleflight<K> {
    inflight: StdMutex<HashSet<K>>,
    notify: Notify,
}

impl<K> Default for CacheSingleflight<K> {
    fn default() -> Self {
        Self {
            inflight: StdMutex::new(HashSet::new()),
            notify: Notify::new(),
        }
    }
}

enum CacheInflightRegistration<'a, K: Eq + Hash> {
    Leader(CacheInflightGuard<'a, K>),
    Follower,
    Bypass,
}

struct CacheInflightGuard<'a, K: Eq + Hash> {
    singleflight: &'a CacheSingleflight<K>,
    key: Option<K>,
}

impl<K> CacheSingleflight<K>
where
    K: Eq + Hash,
{
    fn notified(&self) -> tokio::sync::futures::Notified<'_> {
        self.notify.notified()
    }

    fn finish(&self, key: &K) {
        let removed = self
            .inflight
            .lock()
            .map(|mut inflight| inflight.remove(key))
            .unwrap_or(false);
        if removed {
            self.notify.notify_waiters();
        }
    }

    fn clear(&self) {
        let cleared = self
            .inflight
            .lock()
            .map(|mut inflight| {
                let had_entries = !inflight.is_empty();
                inflight.clear();
                had_entries
            })
            .unwrap_or(false);
        if cleared {
            self.notify.notify_waiters();
        }
    }
}

impl<K> CacheSingleflight<K>
where
    K: Clone + Eq + Hash,
{
    fn register(&self, key: &K) -> CacheInflightRegistration<'_, K> {
        match self.inflight.lock() {
            Ok(mut inflight) => {
                if inflight.contains(key) {
                    CacheInflightRegistration::Follower
                } else {
                    inflight.insert(key.clone());
                    CacheInflightRegistration::Leader(CacheInflightGuard {
                        singleflight: self,
                        key: Some(key.clone()),
                    })
                }
            }
            Err(_) => CacheInflightRegistration::Bypass,
        }
    }
}

impl<K> Drop for CacheInflightGuard<'_, K>
where
    K: Eq + Hash,
{
    fn drop(&mut self) {
        if let Some(key) = self.key.take() {
            self.singleflight.finish(&key);
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct AuthApiKeyIdentityCacheKey {
    user_id: String,
    api_key_id: String,
}

impl AuthApiKeyIdentityCacheKey {
    pub(crate) fn new(user_id: &str, api_key_id: &str) -> Self {
        Self {
            user_id: user_id.trim().to_string(),
            api_key_id: api_key_id.trim().to_string(),
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.user_id.is_empty() || self.api_key_id.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct AuthApiKeyFeatureCacheKey {
    user_id: String,
    api_key_id: String,
    is_standalone: bool,
}

impl AuthApiKeyFeatureCacheKey {
    pub(crate) fn new(user_id: &str, api_key_id: &str, is_standalone: bool) -> Self {
        Self {
            user_id: user_id.trim().to_string(),
            api_key_id: api_key_id.trim().to_string(),
            is_standalone,
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.api_key_id.is_empty() || (!self.is_standalone && self.user_id.is_empty())
    }
}

#[derive(Debug, Default)]
pub(crate) struct AuthSnapshotCache {
    entries: ExpiringMap<AuthSnapshotCacheKey, Option<GatewayAuthApiKeySnapshot>>,
    singleflight: CacheSingleflight<AuthSnapshotCacheKey>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum AuthSnapshotCacheKey {
    KeyHash(String),
    UserApiKeyIds(AuthApiKeyIdentityCacheKey),
}

impl AuthSnapshotCacheKey {
    pub(crate) fn key_hash(key_hash: &str) -> Self {
        Self::KeyHash(key_hash.trim().to_string())
    }

    pub(crate) fn user_api_key_ids(user_id: &str, api_key_id: &str) -> Self {
        Self::UserApiKeyIds(AuthApiKeyIdentityCacheKey::new(user_id, api_key_id))
    }

    pub(crate) fn is_empty(&self) -> bool {
        match self {
            Self::KeyHash(key_hash) => key_hash.is_empty(),
            Self::UserApiKeyIds(key) => key.is_empty(),
        }
    }
}

impl AuthSnapshotCache {
    pub(crate) fn get(
        &self,
        key: &AuthSnapshotCacheKey,
        ttl: Duration,
    ) -> Option<Option<GatewayAuthApiKeySnapshot>> {
        self.entries.get_fresh(key, ttl)
    }

    pub(crate) fn insert(
        &self,
        key: AuthSnapshotCacheKey,
        value: Option<GatewayAuthApiKeySnapshot>,
        ttl: Duration,
    ) {
        self.entries
            .insert(key, value, ttl, AUTH_RUNTIME_CACHE_MAX_ENTRIES);
    }

    pub(crate) async fn get_or_load<E, F, Fut>(
        &self,
        key: AuthSnapshotCacheKey,
        ttl: Duration,
        load: F,
    ) -> Result<Option<GatewayAuthApiKeySnapshot>, E>
    where
        F: Fn() -> Fut,
        Fut: Future<Output = Result<Option<GatewayAuthApiKeySnapshot>, E>>,
    {
        if let Some(value) = self.get(&key, ttl) {
            return Ok(value);
        }

        loop {
            let notified = self.singleflight.notified();
            match self.singleflight.register(&key) {
                CacheInflightRegistration::Bypass => {
                    let value = load().await?;
                    self.insert(key, value.clone(), ttl);
                    return Ok(value);
                }
                CacheInflightRegistration::Follower => {
                    notified.await;
                    if let Some(value) = self.get(&key, ttl) {
                        return Ok(value);
                    }
                }
                CacheInflightRegistration::Leader(_guard) => {
                    let value = load().await?;
                    self.insert(key, value.clone(), ttl);
                    return Ok(value);
                }
            }
        }
    }

    pub(crate) fn clear(&self) {
        self.entries.clear();
        self.singleflight.clear();
    }
}

#[derive(Debug)]
pub(crate) struct JsonValueCache<K> {
    entries: ExpiringMap<K, Option<Value>>,
    singleflight: CacheSingleflight<K>,
}

impl<K> Default for JsonValueCache<K> {
    fn default() -> Self {
        Self {
            entries: ExpiringMap::default(),
            singleflight: CacheSingleflight::default(),
        }
    }
}

impl<K> JsonValueCache<K>
where
    K: Clone + Eq + Hash,
{
    pub(crate) fn get(&self, key: &K, ttl: Duration) -> Option<Option<Value>> {
        self.entries.get_fresh(key, ttl)
    }

    pub(crate) fn insert(&self, key: K, value: Option<Value>, ttl: Duration) {
        self.entries
            .insert(key, value, ttl, AUTH_RUNTIME_CACHE_MAX_ENTRIES);
    }

    pub(crate) async fn get_or_load<E, F, Fut>(
        &self,
        key: K,
        ttl: Duration,
        load: F,
    ) -> Result<Option<Value>, E>
    where
        F: Fn() -> Fut,
        Fut: Future<Output = Result<Option<Value>, E>>,
    {
        if let Some(value) = self.get(&key, ttl) {
            return Ok(value);
        }

        loop {
            let notified = self.singleflight.notified();
            match self.singleflight.register(&key) {
                CacheInflightRegistration::Bypass => {
                    let value = load().await?;
                    self.insert(key, value.clone(), ttl);
                    return Ok(value);
                }
                CacheInflightRegistration::Follower => {
                    notified.await;
                    if let Some(value) = self.get(&key, ttl) {
                        return Ok(value);
                    }
                }
                CacheInflightRegistration::Leader(_guard) => {
                    let value = load().await?;
                    self.insert(key, value.clone(), ttl);
                    return Ok(value);
                }
            }
        }
    }

    pub(crate) fn clear(&self) {
        self.entries.clear();
        self.singleflight.clear();
    }
}

#[derive(Debug)]
pub(crate) struct ValueCache<K, V> {
    entries: ExpiringMap<K, Option<V>>,
    singleflight: CacheSingleflight<K>,
}

impl<K, V> Default for ValueCache<K, V> {
    fn default() -> Self {
        Self {
            entries: ExpiringMap::default(),
            singleflight: CacheSingleflight::default(),
        }
    }
}

impl<K, V> ValueCache<K, V>
where
    K: Clone + Eq + Hash,
    V: Clone,
{
    pub(crate) fn get(&self, key: &K, ttl: Duration) -> Option<Option<V>> {
        self.entries.get_fresh(key, ttl)
    }

    pub(crate) fn insert(&self, key: K, value: Option<V>, ttl: Duration) {
        self.entries
            .insert(key, value, ttl, AUTH_RUNTIME_CACHE_MAX_ENTRIES);
    }

    pub(crate) async fn get_or_load<E, F, Fut>(
        &self,
        key: K,
        ttl: Duration,
        load: F,
    ) -> Result<Option<V>, E>
    where
        F: Fn() -> Fut,
        Fut: Future<Output = Result<Option<V>, E>>,
    {
        if let Some(value) = self.get(&key, ttl) {
            return Ok(value);
        }

        loop {
            let notified = self.singleflight.notified();
            match self.singleflight.register(&key) {
                CacheInflightRegistration::Bypass => {
                    let value = load().await?;
                    self.insert(key, value.clone(), ttl);
                    return Ok(value);
                }
                CacheInflightRegistration::Follower => {
                    notified.await;
                    if let Some(value) = self.get(&key, ttl) {
                        return Ok(value);
                    }
                }
                CacheInflightRegistration::Leader(_guard) => {
                    let value = load().await?;
                    self.insert(key, value.clone(), ttl);
                    return Ok(value);
                }
            }
        }
    }

    pub(crate) fn clear(&self) {
        self.entries.clear();
        self.singleflight.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::ValueCache;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use std::time::{Duration, Instant};

    fn record_max(max_seen: &AtomicUsize, value: usize) {
        let mut current = max_seen.load(Ordering::Acquire);
        while value > current {
            match max_seen.compare_exchange(current, value, Ordering::AcqRel, Ordering::Acquire) {
                Ok(_) => break,
                Err(next) => current = next,
            }
        }
    }

    #[tokio::test]
    async fn value_cache_coalesces_concurrent_loads_for_same_key() {
        let cache = Arc::new(ValueCache::<String, u64>::default());
        let calls = Arc::new(AtomicUsize::new(0));
        let mut tasks = Vec::new();

        for _ in 0..16 {
            let cache = Arc::clone(&cache);
            let calls = Arc::clone(&calls);
            tasks.push(tokio::spawn(async move {
                cache
                    .get_or_load::<(), _, _>(
                        "same-key".to_string(),
                        Duration::from_secs(60),
                        || async {
                            calls.fetch_add(1, Ordering::AcqRel);
                            tokio::time::sleep(Duration::from_millis(25)).await;
                            Ok(Some(42))
                        },
                    )
                    .await
                    .unwrap()
            }));
        }

        for task in tasks {
            assert_eq!(task.await.unwrap(), Some(42));
        }
        assert_eq!(calls.load(Ordering::Acquire), 1);
    }

    #[tokio::test]
    async fn value_cache_loads_different_keys_without_global_blocking() {
        let cache = Arc::new(ValueCache::<String, u64>::default());
        let calls = Arc::new(AtomicUsize::new(0));
        let active = Arc::new(AtomicUsize::new(0));
        let max_active = Arc::new(AtomicUsize::new(0));
        let mut tasks = Vec::new();

        for (key, value) in [("key-a", 1_u64), ("key-b", 2_u64)] {
            let cache = Arc::clone(&cache);
            let calls = Arc::clone(&calls);
            let active = Arc::clone(&active);
            let max_active = Arc::clone(&max_active);
            tasks.push(tokio::spawn(async move {
                cache
                    .get_or_load::<(), _, _>(key.to_string(), Duration::from_secs(60), || {
                        let calls = Arc::clone(&calls);
                        let active = Arc::clone(&active);
                        let max_active = Arc::clone(&max_active);
                        async move {
                            calls.fetch_add(1, Ordering::AcqRel);
                            let current = active.fetch_add(1, Ordering::AcqRel) + 1;
                            record_max(&max_active, current);
                            tokio::time::sleep(Duration::from_millis(50)).await;
                            active.fetch_sub(1, Ordering::AcqRel);
                            Ok(Some(value))
                        }
                    })
                    .await
                    .unwrap()
            }));
        }

        let mut results = Vec::new();
        for task in tasks {
            results.push(task.await.unwrap());
        }
        results.sort_unstable();

        assert_eq!(results, vec![Some(1), Some(2)]);
        assert_eq!(calls.load(Ordering::Acquire), 2);
        assert_eq!(max_active.load(Ordering::Acquire), 2);
    }

    #[tokio::test]
    async fn value_cache_clear_releases_same_key_followers() {
        let cache = Arc::new(ValueCache::<String, u64>::default());
        let leader_cache = Arc::clone(&cache);
        let leader = tokio::spawn(async move {
            leader_cache
                .get_or_load::<(), _, _>(
                    "stuck-key".to_string(),
                    Duration::from_secs(60),
                    || async {
                        std::future::pending::<()>().await;
                        Ok(Some(1))
                    },
                )
                .await
        });
        tokio::time::sleep(Duration::from_millis(10)).await;

        let follower_cache = Arc::clone(&cache);
        let follower = tokio::spawn(async move {
            follower_cache
                .get_or_load::<(), _, _>(
                    "stuck-key".to_string(),
                    Duration::from_secs(60),
                    || async { Ok(Some(2)) },
                )
                .await
        });
        tokio::time::sleep(Duration::from_millis(10)).await;

        cache.clear();
        let value = tokio::time::timeout(Duration::from_millis(200), follower)
            .await
            .expect("clear should wake followers")
            .unwrap()
            .unwrap();
        assert_eq!(value, Some(2));

        leader.abort();
        let _ = leader.await;
    }
}

use std::sync::Arc;
use std::time::Duration;

use aether_cache::ExpiringMap;
use aether_data::repository::auth::*;
use aether_data::DataLayerError;
use async_trait::async_trait;

const AUTH_API_KEY_SNAPSHOT_CACHE_TTL: Duration = Duration::from_secs(30);
const AUTH_API_KEY_SNAPSHOT_CACHE_MAX_ENTRIES: usize = 16_384;

pub(super) struct CachedAuthApiKeyReadRepository {
    inner: Arc<dyn AuthApiKeyReadRepository>,
    snapshots: ExpiringMap<AuthApiKeySnapshotCacheKey, Option<StoredAuthApiKeySnapshot>>,
    load_guard: tokio::sync::Mutex<()>,
}

impl CachedAuthApiKeyReadRepository {
    pub(super) fn new(inner: Arc<dyn AuthApiKeyReadRepository>) -> Self {
        Self {
            inner,
            snapshots: ExpiringMap::new(),
            load_guard: tokio::sync::Mutex::new(()),
        }
    }

    fn cache_key(key: AuthApiKeyLookupKey<'_>) -> AuthApiKeySnapshotCacheKey {
        match key {
            AuthApiKeyLookupKey::KeyHash(value) => {
                AuthApiKeySnapshotCacheKey::KeyHash(value.to_string())
            }
            AuthApiKeyLookupKey::ApiKeyId(value) => {
                AuthApiKeySnapshotCacheKey::ApiKeyId(value.to_string())
            }
            AuthApiKeyLookupKey::UserApiKeyIds {
                user_id,
                api_key_id,
            } => AuthApiKeySnapshotCacheKey::UserApiKeyIds {
                user_id: user_id.to_string(),
                api_key_id: api_key_id.to_string(),
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum AuthApiKeySnapshotCacheKey {
    KeyHash(String),
    ApiKeyId(String),
    UserApiKeyIds { user_id: String, api_key_id: String },
}

#[async_trait]
impl AuthApiKeyReadRepository for CachedAuthApiKeyReadRepository {
    async fn find_api_key_snapshot(
        &self,
        key: AuthApiKeyLookupKey<'_>,
    ) -> Result<Option<StoredAuthApiKeySnapshot>, DataLayerError> {
        let cache_key = Self::cache_key(key);
        if let Some(value) = self
            .snapshots
            .get_fresh(&cache_key, AUTH_API_KEY_SNAPSHOT_CACHE_TTL)
        {
            return Ok(value);
        }

        let _guard = self.load_guard.lock().await;
        if let Some(value) = self
            .snapshots
            .get_fresh(&cache_key, AUTH_API_KEY_SNAPSHOT_CACHE_TTL)
        {
            return Ok(value);
        }

        let value = self.inner.find_api_key_snapshot(key).await?;
        self.snapshots.insert(
            cache_key,
            value.clone(),
            AUTH_API_KEY_SNAPSHOT_CACHE_TTL,
            AUTH_API_KEY_SNAPSHOT_CACHE_MAX_ENTRIES,
        );
        Ok(value)
    }

    async fn list_api_key_snapshots_by_ids(
        &self,
        api_key_ids: &[String],
    ) -> Result<Vec<StoredAuthApiKeySnapshot>, DataLayerError> {
        let mut snapshots = Vec::with_capacity(api_key_ids.len());
        for api_key_id in api_key_ids {
            if let Some(snapshot) = self
                .find_api_key_snapshot(AuthApiKeyLookupKey::ApiKeyId(api_key_id))
                .await?
            {
                snapshots.push(snapshot);
            }
        }
        Ok(snapshots)
    }

    async fn list_export_api_keys_by_user_ids(
        &self,
        user_ids: &[String],
    ) -> Result<Vec<StoredAuthApiKeyExportRecord>, DataLayerError> {
        self.inner.list_export_api_keys_by_user_ids(user_ids).await
    }

    async fn list_export_api_keys_by_ids(
        &self,
        api_key_ids: &[String],
    ) -> Result<Vec<StoredAuthApiKeyExportRecord>, DataLayerError> {
        self.inner.list_export_api_keys_by_ids(api_key_ids).await
    }

    async fn list_export_api_keys_by_name_search(
        &self,
        name_search: &str,
    ) -> Result<Vec<StoredAuthApiKeyExportRecord>, DataLayerError> {
        self.inner
            .list_export_api_keys_by_name_search(name_search)
            .await
    }

    async fn list_export_standalone_api_keys_page(
        &self,
        query: &StandaloneApiKeyExportListQuery,
    ) -> Result<Vec<StoredAuthApiKeyExportRecord>, DataLayerError> {
        self.inner.list_export_standalone_api_keys_page(query).await
    }

    async fn count_export_standalone_api_keys(
        &self,
        is_active: Option<bool>,
    ) -> Result<u64, DataLayerError> {
        self.inner.count_export_standalone_api_keys(is_active).await
    }

    async fn summarize_export_api_keys_by_user_ids(
        &self,
        user_ids: &[String],
        now_unix_secs: u64,
    ) -> Result<AuthApiKeyExportSummary, DataLayerError> {
        self.inner
            .summarize_export_api_keys_by_user_ids(user_ids, now_unix_secs)
            .await
    }

    async fn summarize_export_non_standalone_api_keys(
        &self,
        now_unix_secs: u64,
    ) -> Result<AuthApiKeyExportSummary, DataLayerError> {
        self.inner
            .summarize_export_non_standalone_api_keys(now_unix_secs)
            .await
    }

    async fn summarize_export_standalone_api_keys(
        &self,
        now_unix_secs: u64,
    ) -> Result<AuthApiKeyExportSummary, DataLayerError> {
        self.inner
            .summarize_export_standalone_api_keys(now_unix_secs)
            .await
    }

    async fn find_export_standalone_api_key_by_id(
        &self,
        api_key_id: &str,
    ) -> Result<Option<StoredAuthApiKeyExportRecord>, DataLayerError> {
        self.inner
            .find_export_standalone_api_key_by_id(api_key_id)
            .await
    }

    async fn list_export_standalone_api_keys(
        &self,
    ) -> Result<Vec<StoredAuthApiKeyExportRecord>, DataLayerError> {
        self.inner.list_export_standalone_api_keys().await
    }
}

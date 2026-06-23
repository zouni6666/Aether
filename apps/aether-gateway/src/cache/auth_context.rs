use std::collections::HashSet;
use std::time::Duration;

use aether_cache::ExpiringMap;
use tokio::sync::Notify;

use crate::control::GatewayControlAuthContext;

#[derive(Debug)]
pub(crate) struct AuthContextCache {
    entries: ExpiringMap<String, GatewayControlAuthContext>,
    inflight: std::sync::Mutex<HashSet<String>>,
    notify: Notify,
}

impl Default for AuthContextCache {
    fn default() -> Self {
        Self {
            entries: ExpiringMap::default(),
            inflight: std::sync::Mutex::new(HashSet::new()),
            notify: Notify::new(),
        }
    }
}

pub(crate) enum AuthContextInflightRegistration<'a> {
    Leader(AuthContextInflightGuard<'a>),
    Follower,
    Bypass,
}

pub(crate) struct AuthContextInflightGuard<'a> {
    cache: &'a AuthContextCache,
    cache_key: Option<String>,
}

impl Drop for AuthContextInflightGuard<'_> {
    fn drop(&mut self) {
        let Some(cache_key) = self.cache_key.take() else {
            return;
        };
        let removed = self
            .cache
            .inflight
            .lock()
            .map(|mut inflight| inflight.remove(&cache_key))
            .unwrap_or(false);
        if removed {
            self.cache.notify.notify_waiters();
        }
    }
}

impl AuthContextCache {
    pub(crate) fn get_fresh(
        &self,
        cache_key: &str,
        ttl: Duration,
    ) -> Option<GatewayControlAuthContext> {
        self.entries.get_fresh(&cache_key.to_string(), ttl)
    }

    pub(crate) fn insert(
        &self,
        cache_key: String,
        auth_context: GatewayControlAuthContext,
        ttl: Duration,
        max_entries: usize,
    ) {
        self.entries
            .insert(cache_key, auth_context, ttl, max_entries);
    }

    pub(crate) fn notified(&self) -> tokio::sync::futures::Notified<'_> {
        self.notify.notified()
    }

    pub(crate) fn register_inflight(&self, cache_key: &str) -> AuthContextInflightRegistration<'_> {
        let cache_key = cache_key.trim();
        if cache_key.is_empty() {
            return AuthContextInflightRegistration::Bypass;
        }
        match self.inflight.lock() {
            Ok(mut inflight) => {
                if inflight.contains(cache_key) {
                    AuthContextInflightRegistration::Follower
                } else {
                    inflight.insert(cache_key.to_string());
                    AuthContextInflightRegistration::Leader(AuthContextInflightGuard {
                        cache: self,
                        cache_key: Some(cache_key.to_string()),
                    })
                }
            }
            Err(_) => AuthContextInflightRegistration::Bypass,
        }
    }

    pub(crate) fn clear(&self) {
        self.entries.clear();
        if let Ok(mut inflight) = self.inflight.lock() {
            let had_inflight = !inflight.is_empty();
            inflight.clear();
            if had_inflight {
                self.notify.notify_waiters();
            }
        }
    }
}

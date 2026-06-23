use std::collections::HashSet;
use std::time::Duration;

use aether_cache::ExpiringMap;
use tokio::sync::Notify;

const MAX_ENTRIES: usize = 512;

#[derive(Debug)]
pub(crate) struct SystemConfigCache {
    entries: ExpiringMap<String, Option<serde_json::Value>>,
    inflight: std::sync::Mutex<HashSet<String>>,
    notify: Notify,
}

impl Default for SystemConfigCache {
    fn default() -> Self {
        Self {
            entries: ExpiringMap::new(),
            inflight: std::sync::Mutex::new(HashSet::new()),
            notify: Notify::new(),
        }
    }
}

pub(crate) enum SystemConfigInflightRegistration<'a> {
    Leader(SystemConfigInflightGuard<'a>),
    Follower,
    Bypass,
}

pub(crate) struct SystemConfigInflightGuard<'a> {
    cache: &'a SystemConfigCache,
    key: Option<String>,
}

impl Drop for SystemConfigInflightGuard<'_> {
    fn drop(&mut self) {
        if let Some(key) = self.key.take() {
            self.cache.finish_load(&key);
        }
    }
}

impl SystemConfigCache {
    pub(crate) fn get(&self, key: &str, ttl: Duration) -> Option<Option<serde_json::Value>> {
        self.entries.get_fresh(&key.to_string(), ttl)
    }

    pub(crate) fn insert(&self, key: String, value: Option<serde_json::Value>, ttl: Duration) {
        self.entries.insert(key, value, ttl, MAX_ENTRIES);
    }

    pub(crate) fn register_load(&self, key: &str) -> SystemConfigInflightRegistration<'_> {
        match self.inflight.lock() {
            Ok(mut inflight) => {
                if inflight.contains(key) {
                    SystemConfigInflightRegistration::Follower
                } else {
                    inflight.insert(key.to_string());
                    SystemConfigInflightRegistration::Leader(SystemConfigInflightGuard {
                        cache: self,
                        key: Some(key.to_string()),
                    })
                }
            }
            Err(_) => SystemConfigInflightRegistration::Bypass,
        }
    }

    pub(crate) fn notified(&self) -> tokio::sync::futures::Notified<'_> {
        self.notify.notified()
    }

    fn finish_load(&self, key: &str) {
        let removed = self
            .inflight
            .lock()
            .map(|mut inflight| inflight.remove(key))
            .unwrap_or(false);
        if removed {
            self.notify.notify_waiters();
        }
    }

    pub(crate) fn clear(&self) {
        self.entries.clear();
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

//! Immutable client identity snapshots shared by provider adapters.
use std::sync::{Arc, RwLock};

/// Readers release the lock before constructing a request. Publishing never
/// changes snapshots already held by an in-flight request.
#[derive(Debug)]
pub struct ClientProfileStore<T> {
    current: RwLock<Arc<T>>,
}

impl<T> ClientProfileStore<T> {
    pub fn new(profile: T) -> Self {
        Self {
            current: RwLock::new(Arc::new(profile)),
        }
    }

    pub fn snapshot(&self) -> Arc<T> {
        self.current
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    pub fn publish(&self, profile: T) -> Arc<T> {
        let mut current = self
            .current
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        std::mem::replace(&mut *current, Arc::new(profile))
    }
}

/// Constructors validate wire safety; release adapters separately validate
/// official stable semantic versions before publishing them.
pub fn validate_client_version(version: &str) -> Result<&str, &'static str> {
    let version = version.trim();
    if version.is_empty() || version.len() > 64 || !version.bytes().all(|b| (33..=126).contains(&b))
    {
        return Err("invalid client version");
    }
    Ok(version)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn publishing_does_not_mutate_inflight_snapshots() {
        let store = ClientProfileStore::new(("1.0.0", "client/1.0.0"));
        let inflight = store.snapshot();
        let old = store.publish(("1.1.0", "client/1.1.0"));
        assert_eq!(inflight, old);
        assert_eq!(inflight.0, "1.0.0");
        assert_eq!(store.snapshot().1, "client/1.1.0");
    }

    #[test]
    fn rejects_header_injection_and_oversized_versions() {
        for version in ["", "1.0\r\nx: y", "1.0 0", "é"] {
            assert!(validate_client_version(version).is_err());
        }
        assert!(validate_client_version(&"1".repeat(65)).is_err());
        assert_eq!(validate_client_version(" 1.0.0 ").unwrap(), "1.0.0");
    }
}

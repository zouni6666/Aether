use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static REQUEST_DISTRIBUTION_COUNTER: AtomicU64 = AtomicU64::new(0);

pub(crate) fn current_unix_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

pub(crate) fn current_unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

pub(crate) fn request_distribution_seed() -> u64 {
    let now_ms = current_unix_ms();
    let counter = REQUEST_DISTRIBUTION_COUNTER.fetch_add(1, Ordering::Relaxed);
    now_ms.rotate_left(21) ^ counter
}

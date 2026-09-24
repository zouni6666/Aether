pub mod client;
mod dispatcher;
mod heartbeat;
pub mod protocol;
mod stream_handler;
mod task;
mod writer;

use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use tokio::sync::watch;
use tracing::{debug, error, info};

use crate::state::{AppState, ServerContext};

/// If a tunnel stays connected at least this long, treat the next disconnect
/// as a non-failure and reset reconnect backoff.
const STABLE_SESSION_RESET_AFTER: Duration = Duration::from_secs(30);
/// Startup staggering step per secondary connection, used to avoid
/// simultaneous bursts when a pool of tunnels starts together.
const STARTUP_STAGGER_STEP_MS: u64 = 150;
/// Upper bound for startup staggering.
const MAX_STARTUP_STAGGER_MS: u64 = 1_500;
/// Keep a tiny floor for repeated reconnects; first retry is still immediate.
const MIN_RECONNECT_DELAY_MS: u64 = 50;
/// Even under sustained failures, keep probing frequently so recovery is fast
/// once cross-border network quality improves.
const RECONNECT_PROBE_MAX_DELAY_MS: u64 = 3_000;

/// Run the tunnel mode main loop (connect, dispatch, reconnect).
///
/// `conn_idx` identifies which connection in the pool this is (0-based).
/// Only connection 0 sends heartbeats to avoid resetting shared metrics.
pub async fn run(
    state: &Arc<AppState>,
    server: &Arc<ServerContext>,
    conn_idx: usize,
    mut shutdown: watch::Receiver<bool>,
    mut drain: watch::Receiver<bool>,
) {
    info!(server = %server.server_label, conn = conn_idx, "starting tunnel");
    let reconnect_salt = compute_connection_salt(server, conn_idx);

    if *drain.borrow() {
        info!(server = %server.server_label, conn = conn_idx, "tunnel drain requested before startup");
        return;
    }

    let startup_delay = compute_startup_stagger(conn_idx, reconnect_salt);
    if !startup_delay.is_zero() {
        info!(
            server = %server.server_label,
            conn = conn_idx,
            delay_ms = startup_delay.as_millis(),
            "startup stagger before first connect"
        );
        tokio::select! {
            _ = tokio::time::sleep(startup_delay) => {}
            _ = shutdown.changed() => {
                info!(server = %server.server_label, conn = conn_idx, "shutdown requested during startup stagger");
                return;
            }
            _ = drain.changed() => {
                if *drain.borrow() {
                    info!(server = %server.server_label, conn = conn_idx, "tunnel drain requested during startup stagger");
                    return;
                }
            }
        }
    }

    let mut consecutive_failures: u32 = 0;

    loop {
        if *drain.borrow() {
            info!(server = %server.server_label, conn = conn_idx, "tunnel drained, exiting slot");
            return;
        }
        server.tunnel_metrics.record_connect_attempt();
        let started_at = Instant::now();
        match client::connect_and_run(state, server, conn_idx, &mut shutdown, drain.clone()).await {
            Ok(client::TunnelOutcome::Shutdown) => {
                info!(server = %server.server_label, conn = conn_idx, "tunnel shut down gracefully");
                return;
            }
            Ok(client::TunnelOutcome::Disconnected) => {
                debug!(server = %server.server_label, conn = conn_idx, "tunnel disconnected, reconnecting");
            }
            Err(e) => {
                server.tunnel_metrics.record_connect_error();
                server
                    .tunnel_metrics
                    .record_error("tunnel_connect_error", &e.to_string());
                error!(server = %server.server_label, conn = conn_idx, error = %e, "tunnel connection error, reconnecting");
            }
        }

        if *shutdown.borrow() {
            info!(server = %server.server_label, conn = conn_idx, "shutdown requested, not reconnecting");
            return;
        }
        if *drain.borrow() {
            info!(server = %server.server_label, conn = conn_idx, "tunnel drained after disconnect");
            return;
        }

        // Reset backoff after a stable session to keep recovery snappy when
        // failures are only occasional.
        let connected_for = started_at.elapsed();
        if connected_for >= STABLE_SESSION_RESET_AFTER {
            consecutive_failures = 0;
        } else {
            consecutive_failures = consecutive_failures.saturating_add(1);
        }

        let reconnect_delay = compute_reconnect_delay(
            state.config.tunnel_reconnect_base_ms,
            state.config.tunnel_reconnect_max_ms,
            consecutive_failures,
            reconnect_salt,
        );
        if reconnect_delay.is_zero() && consecutive_failures <= 1 {
            debug!(
                server = %server.server_label,
                conn = conn_idx,
                failures = consecutive_failures,
                delay_ms = reconnect_delay.as_millis(),
                "waiting before reconnect"
            );
        } else {
            info!(
                server = %server.server_label,
                conn = conn_idx,
                failures = consecutive_failures,
                delay_ms = reconnect_delay.as_millis(),
                "waiting before reconnect"
            );
        }

        tokio::select! {
            _ = tokio::time::sleep(reconnect_delay) => {}
            _ = shutdown.changed() => {
                info!(server = %server.server_label, conn = conn_idx, "shutdown requested during reconnect wait");
                return;
            }
            _ = drain.changed() => {
                if *drain.borrow() {
                    info!(server = %server.server_label, conn = conn_idx, "tunnel drain requested during reconnect wait");
                    return;
                }
            }
        }
    }
}

fn compute_connection_salt(server: &ServerContext, conn_idx: usize) -> u64 {
    // FNV-1a style hash over server label + connection index.
    let mut h: u64 = 0xcbf29ce484222325;
    for &b in server.server_label.as_bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h ^= conn_idx as u64;
    mix_u64(h)
}

fn compute_startup_stagger(conn_idx: usize, salt: u64) -> Duration {
    if conn_idx == 0 {
        return Duration::ZERO;
    }
    let base = (conn_idx as u64).saturating_mul(STARTUP_STAGGER_STEP_MS);
    let jitter = mix_u64(salt) % 301; // 0..=300ms
    Duration::from_millis((base + jitter).min(MAX_STARTUP_STAGGER_MS))
}

fn compute_reconnect_delay(
    base_ms: u64,
    max_ms: u64,
    consecutive_failures: u32,
    salt: u64,
) -> Duration {
    // First retry should be immediate to maximize recovery speed on transient
    // blips (the user's primary expectation in poor networks).
    if consecutive_failures <= 1 {
        return Duration::ZERO;
    }

    // Keep a sane minimum for repeated failures.
    let base_ms = base_ms.max(MIN_RECONNECT_DELAY_MS);
    let max_ms = max_ms.max(base_ms);
    let cap_ms = compute_reconnect_cap_ms(base_ms, max_ms, consecutive_failures)
        .min(RECONNECT_PROBE_MAX_DELAY_MS.max(base_ms));

    // Equal-jitter: randomize in [cap/2, cap], preventing synchronized reconnect
    // storms while keeping reconnect latency bounded.
    if cap_ms <= 1 {
        return Duration::from_millis(cap_ms);
    }

    let half = cap_ms / 2;
    let span = cap_ms - half;
    let now_nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.subsec_nanos() as u64)
        .unwrap_or(0);
    let mixed = mix_u64(now_nanos ^ salt);
    let jitter = if span == 0 { 0 } else { mixed % (span + 1) };
    Duration::from_millis(half + jitter)
}

fn compute_reconnect_cap_ms(base_ms: u64, max_ms: u64, consecutive_failures: u32) -> u64 {
    if consecutive_failures <= 1 {
        return base_ms.min(max_ms);
    }

    let shift = (consecutive_failures - 1).min(31);
    let factor = 1u64 << shift;
    base_ms.saturating_mul(factor).min(max_ms)
}

fn mix_u64(mut x: u64) -> u64 {
    // SplitMix64 finalizer - cheap bit mixing for pseudo-random jitter.
    x ^= x >> 30;
    x = x.wrapping_mul(0xbf58476d1ce4e5b9);
    x ^= x >> 27;
    x = x.wrapping_mul(0x94d049bb133111eb);
    x ^ (x >> 31)
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::{
        compute_reconnect_cap_ms, compute_reconnect_delay, compute_startup_stagger,
        MAX_STARTUP_STAGGER_MS, RECONNECT_PROBE_MAX_DELAY_MS, STARTUP_STAGGER_STEP_MS,
    };

    #[test]
    fn reconnect_cap_grows_exponentially_and_caps() {
        let base = 500;
        let max = 30_000;
        assert_eq!(compute_reconnect_cap_ms(base, max, 0), 500);
        assert_eq!(compute_reconnect_cap_ms(base, max, 1), 500);
        assert_eq!(compute_reconnect_cap_ms(base, max, 2), 1_000);
        assert_eq!(compute_reconnect_cap_ms(base, max, 3), 2_000);
        assert_eq!(compute_reconnect_cap_ms(base, max, 4), 4_000);
        assert_eq!(compute_reconnect_cap_ms(base, max, 5), 8_000);
        assert_eq!(compute_reconnect_cap_ms(base, max, 6), 16_000);
        assert_eq!(compute_reconnect_cap_ms(base, max, 7), 30_000);
        assert_eq!(compute_reconnect_cap_ms(base, max, 20), 30_000);
    }

    #[test]
    fn startup_stagger_is_zero_for_primary_and_bounded_for_secondary() {
        assert_eq!(compute_startup_stagger(0, 42), Duration::ZERO);

        let d1 = compute_startup_stagger(1, 42);
        let d2 = compute_startup_stagger(2, 42);

        assert!(d1 >= Duration::from_millis(STARTUP_STAGGER_STEP_MS));
        assert!(d1 <= Duration::from_millis(MAX_STARTUP_STAGGER_MS));
        assert!(d2 >= Duration::from_millis(STARTUP_STAGGER_STEP_MS * 2));
        assert!(d2 <= Duration::from_millis(MAX_STARTUP_STAGGER_MS));
    }

    #[test]
    fn reconnect_delay_is_immediate_on_first_failure() {
        assert_eq!(compute_reconnect_delay(700, 45_000, 1, 123), Duration::ZERO);
    }

    #[test]
    fn reconnect_delay_stays_within_probe_ceiling_after_many_failures() {
        let d = compute_reconnect_delay(500, 45_000, 100, 12345);
        assert!(d <= Duration::from_millis(RECONNECT_PROBE_MAX_DELAY_MS));
    }
}

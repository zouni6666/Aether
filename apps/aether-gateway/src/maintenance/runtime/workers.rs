use std::sync::Arc;

use chrono::Utc;
use tracing::warn;

use crate::data::GatewayDataState;
use crate::AppState;

use super::{
    cleanup_processed_usage_counter_deltas_once, duration_until_next_daily_run,
    duration_until_next_db_maintenance_run, duration_until_next_stats_aggregation_run,
    duration_until_next_stats_hourly_aggregation_run, maintenance_timezone, parse_hhmm_time,
    perform_oauth_token_refresh_once, provider_checkin_schedule, run_audit_cleanup_once,
    run_db_maintenance_once, run_gemini_file_mapping_cleanup_once, run_pending_cleanup_once,
    run_pool_monitor_once, run_provider_checkin_once, run_proxy_node_metrics_cleanup_once,
    run_proxy_node_stale_cleanup_once, run_proxy_upgrade_rollout_once,
    run_request_candidate_cleanup_once, run_stats_aggregation_once,
    run_stats_hourly_aggregation_once, run_usage_cleanup_once, run_usage_counter_flush_once,
    run_wallet_daily_usage_aggregation_once, AUDIT_LOG_CLEANUP_INTERVAL,
    GEMINI_FILE_MAPPING_CLEANUP_INTERVAL, OAUTH_TOKEN_REFRESH_INTERVAL, PENDING_CLEANUP_INTERVAL,
    POOL_MONITOR_INTERVAL, PROVIDER_CHECKIN_DEFAULT_TIME, PROXY_NODE_METRICS_CLEANUP_HOUR,
    PROXY_NODE_METRICS_CLEANUP_MINUTE, PROXY_NODE_STALE_SWEEP_INTERVAL,
    PROXY_UPGRADE_ROLLOUT_INTERVAL, REQUEST_CANDIDATE_CLEANUP_INTERVAL, USAGE_CLEANUP_HOUR,
    USAGE_CLEANUP_MINUTE, USAGE_COUNTER_DELTA_CLEANUP_BATCH_SIZE,
    USAGE_COUNTER_DELTA_CLEANUP_INTERVAL, USAGE_COUNTER_DELTA_RETENTION_SECS,
    USAGE_COUNTER_FLUSH_BATCH_SIZE, USAGE_COUNTER_FLUSH_CATCH_UP_BURST_LIMIT,
    USAGE_COUNTER_FLUSH_INTERVAL, WALLET_DAILY_USAGE_AGGREGATION_HOUR,
    WALLET_DAILY_USAGE_AGGREGATION_MINUTE,
};

const STATS_DAILY_CATCH_UP_BURST_LIMIT: usize = 14;
const STATS_HOURLY_CATCH_UP_BURST_LIMIT: usize = 72;

fn log_maintenance_worker_failure(
    worker: &'static str,
    phase: &'static str,
    error: &impl std::fmt::Debug,
) {
    warn!(
        event_name = "maintenance_worker_failed",
        log_type = "ops",
        worker,
        phase,
        error = ?error,
        "gateway maintenance worker failed"
    );
}

pub(crate) fn spawn_audit_cleanup_worker(
    data: Arc<GatewayDataState>,
) -> Option<tokio::task::JoinHandle<()>> {
    if !data.has_audit_log_reader() {
        return None;
    }

    Some(tokio::spawn(async move {
        if let Err(err) = run_audit_cleanup_once(&data).await {
            log_maintenance_worker_failure("audit_cleanup", "startup", &err);
        }
        let mut interval = tokio::time::interval(AUDIT_LOG_CLEANUP_INTERVAL);
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        interval.tick().await;
        loop {
            interval.tick().await;
            if let Err(err) = run_audit_cleanup_once(&data).await {
                log_maintenance_worker_failure("audit_cleanup", "tick", &err);
            }
        }
    }))
}

pub(crate) fn spawn_db_maintenance_worker(
    data: Arc<GatewayDataState>,
) -> Option<tokio::task::JoinHandle<()>> {
    if !data.has_database_maintenance_backend() {
        return None;
    }

    let timezone = maintenance_timezone();
    Some(tokio::spawn(async move {
        loop {
            tokio::time::sleep(duration_until_next_db_maintenance_run(Utc::now(), timezone)).await;
            if let Err(err) = run_db_maintenance_once(&data).await {
                log_maintenance_worker_failure("db_maintenance", "tick", &err);
            }
        }
    }))
}

pub(crate) fn spawn_wallet_daily_usage_aggregation_worker(
    data: Arc<GatewayDataState>,
) -> Option<tokio::task::JoinHandle<()>> {
    if !data.has_wallet_daily_usage_aggregation_backend() {
        return None;
    }

    let timezone = maintenance_timezone();
    Some(tokio::spawn(async move {
        loop {
            tokio::time::sleep(duration_until_next_daily_run(
                Utc::now(),
                timezone,
                WALLET_DAILY_USAGE_AGGREGATION_HOUR,
                WALLET_DAILY_USAGE_AGGREGATION_MINUTE,
            ))
            .await;
            if let Err(err) = run_wallet_daily_usage_aggregation_once(&data).await {
                log_maintenance_worker_failure("wallet_daily_usage_aggregation", "tick", &err);
            }
        }
    }))
}

pub(crate) fn spawn_stats_aggregation_worker(
    data: Arc<GatewayDataState>,
) -> Option<tokio::task::JoinHandle<()>> {
    if !data.has_stats_daily_aggregation_backend() {
        return None;
    }

    Some(tokio::spawn(async move {
        loop {
            let mut processed = 0_usize;
            while processed < STATS_DAILY_CATCH_UP_BURST_LIMIT {
                match run_stats_aggregation_once(&data).await {
                    Ok(true) => processed += 1,
                    Ok(false) => break,
                    Err(err) => {
                        log_maintenance_worker_failure("stats_daily_aggregation", "tick", &err);
                        break;
                    }
                }
            }

            if processed >= STATS_DAILY_CATCH_UP_BURST_LIMIT {
                continue;
            }

            tokio::time::sleep(duration_until_next_stats_aggregation_run(Utc::now())).await;
        }
    }))
}

pub(crate) fn spawn_usage_cleanup_worker(
    data: Arc<GatewayDataState>,
) -> Option<tokio::task::JoinHandle<()>> {
    if !data.has_usage_writer() {
        return None;
    }

    let timezone = maintenance_timezone();
    Some(tokio::spawn(async move {
        loop {
            tokio::time::sleep(duration_until_next_daily_run(
                Utc::now(),
                timezone,
                USAGE_CLEANUP_HOUR,
                USAGE_CLEANUP_MINUTE,
            ))
            .await;
            if let Err(err) = run_usage_cleanup_once(&data).await {
                log_maintenance_worker_failure("usage_cleanup", "tick", &err);
            }
        }
    }))
}

pub(crate) fn spawn_usage_counter_flush_worker(
    data: Arc<GatewayDataState>,
) -> Option<tokio::task::JoinHandle<()>> {
    if !data.has_usage_counter_flush_backend() {
        return None;
    }

    Some(tokio::spawn(async move {
        let mut interval = tokio::time::interval(USAGE_COUNTER_FLUSH_INTERVAL);
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        interval.tick().await;
        let mut last_delta_cleanup = tokio::time::Instant::now();

        loop {
            let mut batches = 0_usize;
            while batches < USAGE_COUNTER_FLUSH_CATCH_UP_BURST_LIMIT {
                match run_usage_counter_flush_once(&data, USAGE_COUNTER_FLUSH_BATCH_SIZE).await {
                    Ok(summary) if summary.rows_claimed > 0 => batches += 1,
                    Ok(_) => break,
                    Err(err) => {
                        log_maintenance_worker_failure("usage_counter_flush", "tick", &err);
                        break;
                    }
                }
            }

            if batches >= USAGE_COUNTER_FLUSH_CATCH_UP_BURST_LIMIT {
                tokio::task::yield_now().await;
                continue;
            }

            if last_delta_cleanup.elapsed() >= USAGE_COUNTER_DELTA_CLEANUP_INTERVAL {
                if let Err(err) = cleanup_processed_usage_counter_deltas_once(
                    &data,
                    USAGE_COUNTER_DELTA_RETENTION_SECS,
                    USAGE_COUNTER_DELTA_CLEANUP_BATCH_SIZE,
                )
                .await
                {
                    log_maintenance_worker_failure("usage_counter_delta_cleanup", "tick", &err);
                }
                last_delta_cleanup = tokio::time::Instant::now();
            }

            interval.tick().await;
        }
    }))
}

pub(crate) fn spawn_provider_checkin_worker(
    state: AppState,
) -> Option<tokio::task::JoinHandle<()>> {
    if !state.has_provider_catalog_data_reader() {
        return None;
    }

    let timezone = maintenance_timezone();
    Some(tokio::spawn(async move {
        loop {
            let (hour, minute) = match provider_checkin_schedule(&state.data).await {
                Ok(schedule) => schedule,
                Err(err) => {
                    warn!(
                        event_name = "maintenance_schedule_lookup_failed",
                        log_type = "ops",
                        worker = "provider_checkin",
                        phase = "schedule_lookup",
                        error = %err,
                        fallback = PROVIDER_CHECKIN_DEFAULT_TIME,
                        "gateway provider checkin schedule lookup failed; falling back"
                    );
                    parse_hhmm_time(PROVIDER_CHECKIN_DEFAULT_TIME)
                        .expect("default provider checkin time should parse")
                }
            };
            tokio::time::sleep(duration_until_next_daily_run(
                Utc::now(),
                timezone,
                hour,
                minute,
            ))
            .await;
            if let Err(err) = run_provider_checkin_once(&state).await {
                log_maintenance_worker_failure("provider_checkin", "tick", &err);
            }
        }
    }))
}

pub(crate) fn spawn_oauth_token_refresh_worker(
    state: AppState,
) -> Option<tokio::task::JoinHandle<()>> {
    if !state.has_provider_catalog_data_reader() || !state.has_provider_catalog_data_writer() {
        return None;
    }

    Some(tokio::spawn(async move {
        if let Err(err) = perform_oauth_token_refresh_once(&state).await {
            log_maintenance_worker_failure("oauth_token_refresh", "startup", &err);
        }
        let mut interval = tokio::time::interval(OAUTH_TOKEN_REFRESH_INTERVAL);
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        interval.tick().await;
        loop {
            interval.tick().await;
            if let Err(err) = perform_oauth_token_refresh_once(&state).await {
                log_maintenance_worker_failure("oauth_token_refresh", "tick", &err);
            }
        }
    }))
}

pub(crate) fn spawn_gemini_file_mapping_cleanup_worker(
    data: Arc<GatewayDataState>,
) -> Option<tokio::task::JoinHandle<()>> {
    if !data.has_gemini_file_mapping_writer() {
        return None;
    }

    Some(tokio::spawn(async move {
        if let Err(err) = run_gemini_file_mapping_cleanup_once(&data).await {
            log_maintenance_worker_failure("gemini_file_mapping_cleanup", "startup", &err);
        }
        let mut interval = tokio::time::interval(GEMINI_FILE_MAPPING_CLEANUP_INTERVAL);
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        interval.tick().await;
        loop {
            interval.tick().await;
            if let Err(err) = run_gemini_file_mapping_cleanup_once(&data).await {
                log_maintenance_worker_failure("gemini_file_mapping_cleanup", "tick", &err);
            }
        }
    }))
}

pub(crate) fn spawn_pending_cleanup_worker(
    data: Arc<GatewayDataState>,
) -> Option<tokio::task::JoinHandle<()>> {
    if !data.has_usage_writer() {
        return None;
    }

    Some(tokio::spawn(async move {
        if let Err(err) = run_pending_cleanup_once(&data).await {
            log_maintenance_worker_failure("pending_cleanup", "startup", &err);
        }
        let mut interval = tokio::time::interval(PENDING_CLEANUP_INTERVAL);
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        interval.tick().await;
        loop {
            interval.tick().await;
            if let Err(err) = run_pending_cleanup_once(&data).await {
                log_maintenance_worker_failure("pending_cleanup", "tick", &err);
            }
        }
    }))
}

pub(crate) fn spawn_proxy_node_stale_cleanup_worker(
    data: Arc<GatewayDataState>,
) -> Option<tokio::task::JoinHandle<()>> {
    if !data.has_proxy_node_reader() || !data.has_proxy_node_writer() {
        return None;
    }

    Some(tokio::spawn(async move {
        if let Err(err) = run_proxy_node_stale_cleanup_once(&data).await {
            log_maintenance_worker_failure("proxy_node_stale_cleanup", "startup", &err);
        }
        let mut interval = tokio::time::interval(PROXY_NODE_STALE_SWEEP_INTERVAL);
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        interval.tick().await;
        loop {
            interval.tick().await;
            if let Err(err) = run_proxy_node_stale_cleanup_once(&data).await {
                log_maintenance_worker_failure("proxy_node_stale_cleanup", "tick", &err);
            }
        }
    }))
}

pub(crate) fn spawn_proxy_node_metrics_cleanup_worker(
    data: Arc<GatewayDataState>,
) -> Option<tokio::task::JoinHandle<()>> {
    if !data.has_proxy_node_writer() {
        return None;
    }

    let timezone = maintenance_timezone();
    Some(tokio::spawn(async move {
        loop {
            tokio::time::sleep(duration_until_next_daily_run(
                Utc::now(),
                timezone,
                PROXY_NODE_METRICS_CLEANUP_HOUR,
                PROXY_NODE_METRICS_CLEANUP_MINUTE,
            ))
            .await;
            if let Err(err) = run_proxy_node_metrics_cleanup_once(&data).await {
                log_maintenance_worker_failure("proxy_node_metrics_cleanup", "tick", &err);
            }
        }
    }))
}

pub(crate) fn spawn_proxy_upgrade_rollout_worker(
    state: AppState,
) -> Option<tokio::task::JoinHandle<()>> {
    if !state.data.has_proxy_node_reader()
        || !state.data.has_proxy_node_writer()
        || !state.data.has_system_config_store()
    {
        return None;
    }

    Some(tokio::spawn(async move {
        if let Err(err) = run_proxy_upgrade_rollout_once(&state).await {
            log_maintenance_worker_failure("proxy_upgrade_rollout", "startup", &err);
        }
        let mut interval = tokio::time::interval(PROXY_UPGRADE_ROLLOUT_INTERVAL);
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        interval.tick().await;
        loop {
            interval.tick().await;
            if let Err(err) = run_proxy_upgrade_rollout_once(&state).await {
                log_maintenance_worker_failure("proxy_upgrade_rollout", "tick", &err);
            }
        }
    }))
}

pub(crate) fn spawn_pool_monitor_worker(
    data: Arc<GatewayDataState>,
) -> Option<tokio::task::JoinHandle<()>> {
    if !data.has_database_pool_summary() {
        return None;
    }

    Some(tokio::spawn(async move {
        let mut interval = tokio::time::interval(POOL_MONITOR_INTERVAL);
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        interval.tick().await;
        loop {
            interval.tick().await;
            run_pool_monitor_once(&data);
        }
    }))
}

pub(crate) fn spawn_stats_hourly_aggregation_worker(
    data: Arc<GatewayDataState>,
) -> Option<tokio::task::JoinHandle<()>> {
    if !data.has_stats_hourly_aggregation_backend() {
        return None;
    }

    Some(tokio::spawn(async move {
        loop {
            let mut processed = 0_usize;
            while processed < STATS_HOURLY_CATCH_UP_BURST_LIMIT {
                match run_stats_hourly_aggregation_once(&data).await {
                    Ok(true) => processed += 1,
                    Ok(false) => break,
                    Err(err) => {
                        log_maintenance_worker_failure("stats_hourly_aggregation", "tick", &err);
                        break;
                    }
                }
            }

            if processed >= STATS_HOURLY_CATCH_UP_BURST_LIMIT {
                continue;
            }

            tokio::time::sleep(duration_until_next_stats_hourly_aggregation_run(Utc::now())).await;
        }
    }))
}

pub(crate) fn spawn_request_candidate_cleanup_worker(
    data: Arc<GatewayDataState>,
) -> Option<tokio::task::JoinHandle<()>> {
    if !data.has_request_candidate_writer() {
        return None;
    }

    Some(tokio::spawn(async move {
        if let Err(err) = run_request_candidate_cleanup_once(&data).await {
            log_maintenance_worker_failure("request_candidate_cleanup", "startup", &err);
        }
        let mut interval = tokio::time::interval(REQUEST_CANDIDATE_CLEANUP_INTERVAL);
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        interval.tick().await;
        loop {
            interval.tick().await;
            if let Err(err) = run_request_candidate_cleanup_once(&data).await {
                log_maintenance_worker_failure("request_candidate_cleanup", "tick", &err);
            }
        }
    }))
}

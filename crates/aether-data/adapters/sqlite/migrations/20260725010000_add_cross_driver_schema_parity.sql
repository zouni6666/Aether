-- Complete the portable schema contract that is already represented by the
-- logical/generated schema and PostgreSQL usage capture tables.

ALTER TABLE provider_api_keys ADD COLUMN last_error_at INTEGER;
ALTER TABLE provider_api_keys ADD COLUMN last_error_msg TEXT;

CREATE TABLE IF NOT EXISTS api_key_provider_mappings (
    id TEXT PRIMARY KEY NOT NULL,
    api_key_id TEXT NOT NULL,
    provider_id TEXT NOT NULL,
    priority_adjustment INTEGER NOT NULL DEFAULT 0,
    weight_multiplier REAL NOT NULL DEFAULT 1,
    is_enabled INTEGER NOT NULL DEFAULT 1,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    UNIQUE (api_key_id, provider_id)
);

CREATE INDEX IF NOT EXISTS api_key_provider_mappings_api_key_id_idx
    ON api_key_provider_mappings (api_key_id);
CREATE INDEX IF NOT EXISTS api_key_provider_mappings_provider_id_idx
    ON api_key_provider_mappings (provider_id);
CREATE INDEX IF NOT EXISTS idx_apikey_provider_enabled
    ON api_key_provider_mappings (api_key_id, is_enabled);

CREATE TABLE IF NOT EXISTS provider_usage_tracking (
    id TEXT PRIMARY KEY NOT NULL,
    provider_id TEXT NOT NULL,
    window_start INTEGER NOT NULL,
    window_end INTEGER NOT NULL,
    total_requests INTEGER NOT NULL DEFAULT 0,
    successful_requests INTEGER NOT NULL DEFAULT 0,
    failed_requests INTEGER NOT NULL DEFAULT 0,
    avg_response_time_ms REAL NOT NULL DEFAULT 0,
    total_response_time_ms REAL NOT NULL DEFAULT 0,
    total_cost_usd REAL NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS provider_usage_tracking_provider_id_idx
    ON provider_usage_tracking (provider_id);
CREATE INDEX IF NOT EXISTS provider_usage_tracking_window_start_idx
    ON provider_usage_tracking (window_start);
CREATE INDEX IF NOT EXISTS idx_provider_window
    ON provider_usage_tracking (provider_id, window_start);
CREATE INDEX IF NOT EXISTS idx_window_time
    ON provider_usage_tracking (window_start, window_end);

-- The baseline already has partial uniqueness guards for enabled billing
-- configuration. These full indexes cover reads that also inspect disabled rows.
CREATE INDEX IF NOT EXISTS billing_rules_global_model_task_idx
    ON billing_rules (global_model_id, task_type, is_enabled);
CREATE INDEX IF NOT EXISTS billing_rules_model_task_idx
    ON billing_rules (model_id, task_type, is_enabled);
CREATE INDEX IF NOT EXISTS dimension_collectors_enabled_idx
    ON dimension_collectors (api_format, task_type, dimension_name, priority, is_enabled);

ALTER TABLE video_tasks ADD COLUMN converted_request_body TEXT;
ALTER TABLE video_tasks ADD COLUMN max_retries INTEGER NOT NULL DEFAULT 3;
ALTER TABLE video_tasks ADD COLUMN video_urls TEXT;
ALTER TABLE video_tasks ADD COLUMN thumbnail_url TEXT;
ALTER TABLE video_tasks ADD COLUMN video_size_bytes INTEGER;
ALTER TABLE video_tasks ADD COLUMN video_expires_at INTEGER;
ALTER TABLE video_tasks ADD COLUMN stored_video_path TEXT;
ALTER TABLE video_tasks ADD COLUMN storage_provider TEXT;
ALTER TABLE video_tasks ADD COLUMN remixed_from_task_id TEXT;
ALTER TABLE video_tasks ADD COLUMN webhook_url TEXT;
ALTER TABLE video_tasks ADD COLUMN webhook_sent INTEGER NOT NULL DEFAULT 0;
ALTER TABLE video_tasks ADD COLUMN webhook_sent_at INTEGER;
ALTER TABLE video_tasks ADD COLUMN video_duration_seconds REAL;

-- Portable compatibility columns. New canonical HTTP payload writes use the
-- normalized usage_http_audits and usage_body_blobs tables below.
ALTER TABLE "usage" ADD COLUMN input_output_total_tokens INTEGER NOT NULL DEFAULT 0;
ALTER TABLE "usage" ADD COLUMN cache_creation_input_tokens_5m INTEGER NOT NULL DEFAULT 0;
ALTER TABLE "usage" ADD COLUMN cache_creation_input_tokens_1h INTEGER NOT NULL DEFAULT 0;
ALTER TABLE "usage" ADD COLUMN input_context_tokens INTEGER NOT NULL DEFAULT 0;
ALTER TABLE "usage" ADD COLUMN input_cost_usd REAL NOT NULL DEFAULT 0;
ALTER TABLE "usage" ADD COLUMN output_cost_usd REAL NOT NULL DEFAULT 0;
ALTER TABLE "usage" ADD COLUMN cache_cost_usd REAL NOT NULL DEFAULT 0;
ALTER TABLE "usage" ADD COLUMN cache_creation_cost_usd_5m REAL NOT NULL DEFAULT 0;
ALTER TABLE "usage" ADD COLUMN cache_creation_cost_usd_1h REAL NOT NULL DEFAULT 0;
ALTER TABLE "usage" ADD COLUMN request_cost_usd REAL NOT NULL DEFAULT 0;
ALTER TABLE "usage" ADD COLUMN actual_input_cost_usd REAL NOT NULL DEFAULT 0;
ALTER TABLE "usage" ADD COLUMN actual_output_cost_usd REAL NOT NULL DEFAULT 0;
ALTER TABLE "usage" ADD COLUMN actual_cache_cost_usd REAL NOT NULL DEFAULT 0;
ALTER TABLE "usage" ADD COLUMN actual_cache_creation_cost_usd REAL NOT NULL DEFAULT 0;
ALTER TABLE "usage" ADD COLUMN actual_cache_creation_cost_usd_5m REAL NOT NULL DEFAULT 0;
ALTER TABLE "usage" ADD COLUMN actual_cache_creation_cost_usd_1h REAL NOT NULL DEFAULT 0;
ALTER TABLE "usage" ADD COLUMN actual_cache_read_cost_usd REAL NOT NULL DEFAULT 0;
ALTER TABLE "usage" ADD COLUMN actual_request_cost_usd REAL NOT NULL DEFAULT 0;
ALTER TABLE "usage" ADD COLUMN rate_multiplier REAL NOT NULL DEFAULT 1;
ALTER TABLE "usage" ADD COLUMN input_price_per_1m REAL;
ALTER TABLE "usage" ADD COLUMN cache_creation_price_per_1m REAL;
ALTER TABLE "usage" ADD COLUMN cache_creation_price_per_1m_5m REAL;
ALTER TABLE "usage" ADD COLUMN cache_creation_price_per_1m_1h REAL;
ALTER TABLE "usage" ADD COLUMN cache_read_price_per_1m REAL;
ALTER TABLE "usage" ADD COLUMN price_per_request REAL;
ALTER TABLE "usage" ADD COLUMN request_headers TEXT;
ALTER TABLE "usage" ADD COLUMN request_body TEXT;
ALTER TABLE "usage" ADD COLUMN provider_request_headers TEXT;
ALTER TABLE "usage" ADD COLUMN provider_request_body TEXT;
ALTER TABLE "usage" ADD COLUMN response_headers TEXT;
ALTER TABLE "usage" ADD COLUMN response_body TEXT;
ALTER TABLE "usage" ADD COLUMN client_response_headers TEXT;
ALTER TABLE "usage" ADD COLUMN client_response_body TEXT;
ALTER TABLE "usage" ADD COLUMN request_body_compressed BLOB;
ALTER TABLE "usage" ADD COLUMN provider_request_body_compressed BLOB;
ALTER TABLE "usage" ADD COLUMN response_body_compressed BLOB;
ALTER TABLE "usage" ADD COLUMN client_response_body_compressed BLOB;
ALTER TABLE "usage" ADD COLUMN created_at INTEGER;
ALTER TABLE "usage" ADD COLUMN username TEXT;
ALTER TABLE "usage" ADD COLUMN api_key_name TEXT;

CREATE TABLE IF NOT EXISTS usage_body_blobs (
    body_ref TEXT PRIMARY KEY NOT NULL,
    request_id TEXT NOT NULL,
    body_field TEXT NOT NULL,
    payload_gzip BLOB NOT NULL,
    created_at INTEGER NOT NULL DEFAULT (CAST(strftime('%s', 'now') AS INTEGER)),
    updated_at INTEGER NOT NULL DEFAULT (CAST(strftime('%s', 'now') AS INTEGER)),
    UNIQUE (request_id, body_field),
    FOREIGN KEY (request_id) REFERENCES "usage" (request_id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS ix_usage_body_blobs_request_id
    ON usage_body_blobs (request_id);

CREATE TABLE IF NOT EXISTS usage_http_audits (
    request_id TEXT PRIMARY KEY NOT NULL,
    request_headers TEXT,
    provider_request_headers TEXT,
    response_headers TEXT,
    client_response_headers TEXT,
    request_body_ref TEXT,
    provider_request_body_ref TEXT,
    response_body_ref TEXT,
    client_response_body_ref TEXT,
    request_body_state TEXT,
    provider_request_body_state TEXT,
    response_body_state TEXT,
    client_response_body_state TEXT,
    body_capture_mode TEXT NOT NULL DEFAULT 'none',
    created_at INTEGER NOT NULL DEFAULT (CAST(strftime('%s', 'now') AS INTEGER)),
    updated_at INTEGER NOT NULL DEFAULT (CAST(strftime('%s', 'now') AS INTEGER)),
    FOREIGN KEY (request_id) REFERENCES "usage" (request_id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS ix_usage_http_audits_updated_at
    ON usage_http_audits (updated_at);

-- Billing V3 keeps the immutable pricing/token snapshot separate from the
-- mutable compatibility columns on usage.
ALTER TABLE usage_settlement_snapshots
    ADD COLUMN billing_snapshot_schema_version TEXT;
ALTER TABLE usage_settlement_snapshots
    ADD COLUMN billing_snapshot_status TEXT;
ALTER TABLE usage_settlement_snapshots
    ADD COLUMN rate_multiplier REAL;
ALTER TABLE usage_settlement_snapshots
    ADD COLUMN is_free_tier INTEGER;
ALTER TABLE usage_settlement_snapshots
    ADD COLUMN input_price_per_1m REAL;
ALTER TABLE usage_settlement_snapshots
    ADD COLUMN output_price_per_1m REAL;
ALTER TABLE usage_settlement_snapshots
    ADD COLUMN cache_creation_price_per_1m REAL;
ALTER TABLE usage_settlement_snapshots
    ADD COLUMN cache_read_price_per_1m REAL;
ALTER TABLE usage_settlement_snapshots
    ADD COLUMN price_per_request REAL;
ALTER TABLE usage_settlement_snapshots
    ADD COLUMN settlement_snapshot_schema_version TEXT;
ALTER TABLE usage_settlement_snapshots
    ADD COLUMN settlement_snapshot TEXT;
ALTER TABLE usage_settlement_snapshots
    ADD COLUMN billing_dimensions TEXT;
ALTER TABLE usage_settlement_snapshots
    ADD COLUMN billing_input_tokens INTEGER;
ALTER TABLE usage_settlement_snapshots
    ADD COLUMN billing_effective_input_tokens INTEGER;
ALTER TABLE usage_settlement_snapshots
    ADD COLUMN billing_output_tokens INTEGER;
ALTER TABLE usage_settlement_snapshots
    ADD COLUMN billing_cache_creation_tokens INTEGER;
ALTER TABLE usage_settlement_snapshots
    ADD COLUMN billing_cache_creation_5m_tokens INTEGER;
ALTER TABLE usage_settlement_snapshots
    ADD COLUMN billing_cache_creation_1h_tokens INTEGER;
ALTER TABLE usage_settlement_snapshots
    ADD COLUMN billing_cache_read_tokens INTEGER;
ALTER TABLE usage_settlement_snapshots
    ADD COLUMN billing_total_input_context INTEGER;
ALTER TABLE usage_settlement_snapshots
    ADD COLUMN billing_cache_creation_cost_usd REAL;
ALTER TABLE usage_settlement_snapshots
    ADD COLUMN billing_cache_read_cost_usd REAL;
ALTER TABLE usage_settlement_snapshots
    ADD COLUMN billing_total_cost_usd REAL;
ALTER TABLE usage_settlement_snapshots
    ADD COLUMN billing_actual_total_cost_usd REAL;
ALTER TABLE usage_settlement_snapshots
    ADD COLUMN billing_pricing_source TEXT;
ALTER TABLE usage_settlement_snapshots
    ADD COLUMN billing_rule_id TEXT;
ALTER TABLE usage_settlement_snapshots
    ADD COLUMN billing_rule_version TEXT;

CREATE INDEX IF NOT EXISTS ix_usage_settlement_snapshots_schema_version
    ON usage_settlement_snapshots (settlement_snapshot_schema_version);
CREATE INDEX IF NOT EXISTS ix_usage_settlement_snapshots_pricing_source
    ON usage_settlement_snapshots (billing_pricing_source);

CREATE TABLE IF NOT EXISTS stats_summary (
    id TEXT PRIMARY KEY NOT NULL,
    cutoff_date INTEGER NOT NULL,
    all_time_requests INTEGER NOT NULL DEFAULT 0,
    all_time_success_requests INTEGER NOT NULL DEFAULT 0,
    all_time_error_requests INTEGER NOT NULL DEFAULT 0,
    all_time_input_tokens INTEGER NOT NULL DEFAULT 0,
    all_time_output_tokens INTEGER NOT NULL DEFAULT 0,
    all_time_cache_creation_tokens INTEGER NOT NULL DEFAULT 0,
    all_time_cache_read_tokens INTEGER NOT NULL DEFAULT 0,
    all_time_cost REAL NOT NULL DEFAULT 0,
    all_time_actual_cost REAL NOT NULL DEFAULT 0,
    total_users INTEGER NOT NULL DEFAULT 0,
    active_users INTEGER NOT NULL DEFAULT 0,
    total_api_keys INTEGER NOT NULL DEFAULT 0,
    active_api_keys INTEGER NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS user_model_usage_counts (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL,
    model TEXT NOT NULL,
    usage_count INTEGER NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    UNIQUE (user_id, model)
);

CREATE INDEX IF NOT EXISTS idx_user_model_usage_user
    ON user_model_usage_counts (user_id);
CREATE INDEX IF NOT EXISTS idx_user_model_usage_model
    ON user_model_usage_counts (model);

ALTER TABLE stats_daily ADD COLUMN input_cost REAL NOT NULL DEFAULT 0;
ALTER TABLE stats_daily ADD COLUMN output_cost REAL NOT NULL DEFAULT 0;
ALTER TABLE stats_daily ADD COLUMN cache_creation_cost REAL NOT NULL DEFAULT 0;
ALTER TABLE stats_daily ADD COLUMN cache_read_cost REAL NOT NULL DEFAULT 0;
ALTER TABLE stats_daily ADD COLUMN p50_response_time_ms INTEGER;
ALTER TABLE stats_daily ADD COLUMN p90_response_time_ms INTEGER;
ALTER TABLE stats_daily ADD COLUMN p99_response_time_ms INTEGER;
ALTER TABLE stats_daily ADD COLUMN p50_first_byte_time_ms INTEGER;
ALTER TABLE stats_daily ADD COLUMN p90_first_byte_time_ms INTEGER;
ALTER TABLE stats_daily ADD COLUMN p99_first_byte_time_ms INTEGER;

-- PostgreSQL uses a partial index for this active-row cleanup path. SQLite can
-- preserve the same selectivity and ordering.
CREATE INDEX IF NOT EXISTS idx_usage_stale_pending_created_request
    ON "usage" (created_at_unix_ms, request_id)
    WHERE status IN ('pending', 'streaming');

CREATE INDEX IF NOT EXISTS idx_provider_api_keys_provider_created_at_desc
    ON provider_api_keys (provider_id, created_at DESC, name, id);
CREATE INDEX IF NOT EXISTS idx_provider_api_keys_provider_last_used_at_desc
    ON provider_api_keys (provider_id, last_used_at DESC, name, id);

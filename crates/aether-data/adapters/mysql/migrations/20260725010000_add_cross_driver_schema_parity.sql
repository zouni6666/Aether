-- Complete the portable schema contract that is already represented by the
-- logical/generated schema and PostgreSQL usage capture tables.

ALTER TABLE provider_api_keys
    ADD COLUMN `last_error_at` BIGINT,
    ADD COLUMN `last_error_msg` LONGTEXT;

CREATE TABLE IF NOT EXISTS api_key_provider_mappings (
    `id` VARCHAR(64) NOT NULL,
    `api_key_id` VARCHAR(64) NOT NULL,
    `provider_id` VARCHAR(64) NOT NULL,
    `priority_adjustment` INT NOT NULL DEFAULT 0,
    `weight_multiplier` DOUBLE NOT NULL DEFAULT 1,
    `is_enabled` TINYINT(1) NOT NULL DEFAULT 1,
    `created_at` BIGINT NOT NULL,
    `updated_at` BIGINT NOT NULL,
    PRIMARY KEY (`id`),
    UNIQUE KEY uq_apikey_provider (`api_key_id`, `provider_id`),
    KEY api_key_provider_mappings_api_key_id_idx (`api_key_id`),
    KEY api_key_provider_mappings_provider_id_idx (`provider_id`),
    KEY idx_apikey_provider_enabled (`api_key_id`, `is_enabled`)
);

CREATE TABLE IF NOT EXISTS provider_usage_tracking (
    `id` VARCHAR(64) NOT NULL,
    `provider_id` VARCHAR(64) NOT NULL,
    `window_start` BIGINT NOT NULL,
    `window_end` BIGINT NOT NULL,
    `total_requests` INT NOT NULL DEFAULT 0,
    `successful_requests` INT NOT NULL DEFAULT 0,
    `failed_requests` INT NOT NULL DEFAULT 0,
    `avg_response_time_ms` DOUBLE NOT NULL DEFAULT 0,
    `total_response_time_ms` DOUBLE NOT NULL DEFAULT 0,
    `total_cost_usd` DOUBLE NOT NULL DEFAULT 0,
    `created_at` BIGINT NOT NULL,
    `updated_at` BIGINT NOT NULL,
    PRIMARY KEY (`id`),
    KEY provider_usage_tracking_provider_id_idx (`provider_id`),
    KEY provider_usage_tracking_window_start_idx (`window_start`),
    KEY idx_provider_window (`provider_id`, `window_start`),
    KEY idx_window_time (`window_start`, `window_end`)
);

ALTER TABLE video_tasks
    ADD COLUMN `converted_request_body` JSON,
    ADD COLUMN `max_retries` INT NOT NULL DEFAULT 3,
    ADD COLUMN `video_urls` JSON,
    ADD COLUMN `thumbnail_url` LONGTEXT,
    ADD COLUMN `video_size_bytes` BIGINT,
    ADD COLUMN `video_expires_at` BIGINT,
    ADD COLUMN `stored_video_path` VARCHAR(500),
    ADD COLUMN `storage_provider` VARCHAR(50),
    ADD COLUMN `remixed_from_task_id` VARCHAR(64),
    ADD COLUMN `webhook_url` VARCHAR(500),
    ADD COLUMN `webhook_sent` TINYINT(1) NOT NULL DEFAULT 0,
    ADD COLUMN `webhook_sent_at` BIGINT,
    ADD COLUMN `video_duration_seconds` DOUBLE;

-- Portable compatibility columns. New canonical HTTP payload writes use the
-- normalized usage_http_audits and usage_body_blobs tables below.
ALTER TABLE `usage`
    ADD COLUMN `input_output_total_tokens` BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN `cache_creation_input_tokens_5m` BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN `cache_creation_input_tokens_1h` BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN `input_context_tokens` BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN `input_cost_usd` DOUBLE NOT NULL DEFAULT 0,
    ADD COLUMN `output_cost_usd` DOUBLE NOT NULL DEFAULT 0,
    ADD COLUMN `cache_cost_usd` DOUBLE NOT NULL DEFAULT 0,
    ADD COLUMN `cache_creation_cost_usd_5m` DOUBLE NOT NULL DEFAULT 0,
    ADD COLUMN `cache_creation_cost_usd_1h` DOUBLE NOT NULL DEFAULT 0,
    ADD COLUMN `request_cost_usd` DOUBLE NOT NULL DEFAULT 0,
    ADD COLUMN `actual_input_cost_usd` DOUBLE NOT NULL DEFAULT 0,
    ADD COLUMN `actual_output_cost_usd` DOUBLE NOT NULL DEFAULT 0,
    ADD COLUMN `actual_cache_cost_usd` DOUBLE NOT NULL DEFAULT 0,
    ADD COLUMN `actual_cache_creation_cost_usd` DOUBLE NOT NULL DEFAULT 0,
    ADD COLUMN `actual_cache_creation_cost_usd_5m` DOUBLE NOT NULL DEFAULT 0,
    ADD COLUMN `actual_cache_creation_cost_usd_1h` DOUBLE NOT NULL DEFAULT 0,
    ADD COLUMN `actual_cache_read_cost_usd` DOUBLE NOT NULL DEFAULT 0,
    ADD COLUMN `actual_request_cost_usd` DOUBLE NOT NULL DEFAULT 0,
    ADD COLUMN `rate_multiplier` DOUBLE NOT NULL DEFAULT 1,
    ADD COLUMN `input_price_per_1m` DOUBLE,
    ADD COLUMN `cache_creation_price_per_1m` DOUBLE,
    ADD COLUMN `cache_creation_price_per_1m_5m` DOUBLE,
    ADD COLUMN `cache_creation_price_per_1m_1h` DOUBLE,
    ADD COLUMN `cache_read_price_per_1m` DOUBLE,
    ADD COLUMN `price_per_request` DOUBLE,
    ADD COLUMN `request_headers` JSON,
    ADD COLUMN `request_body` JSON,
    ADD COLUMN `provider_request_headers` JSON,
    ADD COLUMN `provider_request_body` JSON,
    ADD COLUMN `response_headers` JSON,
    ADD COLUMN `response_body` JSON,
    ADD COLUMN `client_response_headers` JSON,
    ADD COLUMN `client_response_body` JSON,
    ADD COLUMN `request_body_compressed` LONGBLOB,
    ADD COLUMN `provider_request_body_compressed` LONGBLOB,
    ADD COLUMN `response_body_compressed` LONGBLOB,
    ADD COLUMN `client_response_body_compressed` LONGBLOB,
    ADD COLUMN `created_at` BIGINT,
    ADD COLUMN `username` VARCHAR(255),
    ADD COLUMN `api_key_name` VARCHAR(255);

CREATE TABLE IF NOT EXISTS usage_body_blobs (
    `body_ref` VARCHAR(160) NOT NULL,
    `request_id` VARCHAR(128) NOT NULL,
    `body_field` VARCHAR(50) NOT NULL,
    `payload_gzip` LONGBLOB NOT NULL,
    `created_at` BIGINT NOT NULL DEFAULT (UNIX_TIMESTAMP()),
    `updated_at` BIGINT NOT NULL DEFAULT (UNIX_TIMESTAMP()),
    PRIMARY KEY (`body_ref`),
    UNIQUE KEY usage_body_blobs_request_id_field_key (`request_id`, `body_field`),
    KEY ix_usage_body_blobs_request_id (`request_id`),
    CONSTRAINT usage_body_blobs_request_id_fkey
        FOREIGN KEY (`request_id`) REFERENCES `usage` (`request_id`) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS usage_http_audits (
    `request_id` VARCHAR(128) NOT NULL,
    `request_headers` JSON,
    `provider_request_headers` JSON,
    `response_headers` JSON,
    `client_response_headers` JSON,
    `request_body_ref` VARCHAR(160),
    `provider_request_body_ref` VARCHAR(160),
    `response_body_ref` VARCHAR(160),
    `client_response_body_ref` VARCHAR(160),
    `request_body_state` VARCHAR(32),
    `provider_request_body_state` VARCHAR(32),
    `response_body_state` VARCHAR(32),
    `client_response_body_state` VARCHAR(32),
    `body_capture_mode` VARCHAR(32) NOT NULL DEFAULT 'none',
    `created_at` BIGINT NOT NULL DEFAULT (UNIX_TIMESTAMP()),
    `updated_at` BIGINT NOT NULL DEFAULT (UNIX_TIMESTAMP()),
    PRIMARY KEY (`request_id`),
    KEY ix_usage_http_audits_updated_at (`updated_at`),
    CONSTRAINT usage_http_audits_request_id_fkey
        FOREIGN KEY (`request_id`) REFERENCES `usage` (`request_id`) ON DELETE CASCADE
);

-- Billing V3 keeps the immutable pricing/token snapshot separate from the
-- mutable compatibility columns on usage.
ALTER TABLE usage_settlement_snapshots
    ADD COLUMN `billing_snapshot_schema_version` VARCHAR(20),
    ADD COLUMN `billing_snapshot_status` VARCHAR(20),
    ADD COLUMN `rate_multiplier` DECIMAL(10,6),
    ADD COLUMN `is_free_tier` TINYINT(1),
    ADD COLUMN `input_price_per_1m` DECIMAL(20,8),
    ADD COLUMN `output_price_per_1m` DECIMAL(20,8),
    ADD COLUMN `cache_creation_price_per_1m` DECIMAL(20,8),
    ADD COLUMN `cache_read_price_per_1m` DECIMAL(20,8),
    ADD COLUMN `price_per_request` DECIMAL(20,8),
    ADD COLUMN `settlement_snapshot_schema_version` VARCHAR(20),
    ADD COLUMN `settlement_snapshot` JSON,
    ADD COLUMN `billing_dimensions` JSON,
    ADD COLUMN `billing_input_tokens` BIGINT,
    ADD COLUMN `billing_effective_input_tokens` BIGINT,
    ADD COLUMN `billing_output_tokens` BIGINT,
    ADD COLUMN `billing_cache_creation_tokens` BIGINT,
    ADD COLUMN `billing_cache_creation_5m_tokens` BIGINT,
    ADD COLUMN `billing_cache_creation_1h_tokens` BIGINT,
    ADD COLUMN `billing_cache_read_tokens` BIGINT,
    ADD COLUMN `billing_total_input_context` BIGINT,
    ADD COLUMN `billing_cache_creation_cost_usd` DECIMAL(20,8),
    ADD COLUMN `billing_cache_read_cost_usd` DECIMAL(20,8),
    ADD COLUMN `billing_total_cost_usd` DECIMAL(20,8),
    ADD COLUMN `billing_actual_total_cost_usd` DECIMAL(20,8),
    ADD COLUMN `billing_pricing_source` VARCHAR(50),
    ADD COLUMN `billing_rule_id` VARCHAR(100),
    ADD COLUMN `billing_rule_version` VARCHAR(50);

CREATE INDEX ix_usage_settlement_snapshots_schema_version
    ON usage_settlement_snapshots (`settlement_snapshot_schema_version`);
CREATE INDEX ix_usage_settlement_snapshots_pricing_source
    ON usage_settlement_snapshots (`billing_pricing_source`);

CREATE TABLE IF NOT EXISTS stats_summary (
    `id` VARCHAR(64) NOT NULL,
    `cutoff_date` BIGINT NOT NULL,
    `all_time_requests` BIGINT NOT NULL DEFAULT 0,
    `all_time_success_requests` BIGINT NOT NULL DEFAULT 0,
    `all_time_error_requests` BIGINT NOT NULL DEFAULT 0,
    `all_time_input_tokens` BIGINT NOT NULL DEFAULT 0,
    `all_time_output_tokens` BIGINT NOT NULL DEFAULT 0,
    `all_time_cache_creation_tokens` BIGINT NOT NULL DEFAULT 0,
    `all_time_cache_read_tokens` BIGINT NOT NULL DEFAULT 0,
    `all_time_cost` DOUBLE NOT NULL DEFAULT 0,
    `all_time_actual_cost` DOUBLE NOT NULL DEFAULT 0,
    `total_users` BIGINT NOT NULL DEFAULT 0,
    `active_users` BIGINT NOT NULL DEFAULT 0,
    `total_api_keys` BIGINT NOT NULL DEFAULT 0,
    `active_api_keys` BIGINT NOT NULL DEFAULT 0,
    `created_at` BIGINT NOT NULL,
    `updated_at` BIGINT NOT NULL,
    PRIMARY KEY (`id`)
);

CREATE TABLE IF NOT EXISTS user_model_usage_counts (
    `id` VARCHAR(64) NOT NULL,
    `user_id` VARCHAR(64) NOT NULL,
    `model` VARCHAR(255) NOT NULL,
    `usage_count` BIGINT NOT NULL DEFAULT 0,
    `created_at` BIGINT NOT NULL,
    `updated_at` BIGINT NOT NULL,
    PRIMARY KEY (`id`),
    UNIQUE KEY uq_user_model_usage_count (`user_id`, `model`),
    KEY idx_user_model_usage_user (`user_id`),
    KEY idx_user_model_usage_model (`model`)
);

ALTER TABLE stats_daily
    ADD COLUMN `input_cost` DOUBLE NOT NULL DEFAULT 0,
    ADD COLUMN `output_cost` DOUBLE NOT NULL DEFAULT 0,
    ADD COLUMN `cache_creation_cost` DOUBLE NOT NULL DEFAULT 0,
    ADD COLUMN `cache_read_cost` DOUBLE NOT NULL DEFAULT 0,
    ADD COLUMN `p50_response_time_ms` BIGINT,
    ADD COLUMN `p90_response_time_ms` BIGINT,
    ADD COLUMN `p99_response_time_ms` BIGINT,
    ADD COLUMN `p50_first_byte_time_ms` BIGINT,
    ADD COLUMN `p90_first_byte_time_ms` BIGINT,
    ADD COLUMN `p99_first_byte_time_ms` BIGINT;

-- MySQL has no partial indexes, so keep the active status first and preserve
-- the cleanup query's ascending timestamp/request ordering.
CREATE INDEX idx_usage_stale_pending_created_request
    ON `usage` (`status`, `created_at_unix_ms`, `request_id`);

CREATE INDEX idx_provider_api_keys_provider_created_at_desc
    ON provider_api_keys (`provider_id`, `created_at` DESC, `name`, `id`);
CREATE INDEX idx_provider_api_keys_provider_last_used_at_desc
    ON provider_api_keys (`provider_id`, `last_used_at` DESC, `name`, `id`);

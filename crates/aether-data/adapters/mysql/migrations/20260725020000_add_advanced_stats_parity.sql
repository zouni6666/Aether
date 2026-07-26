ALTER TABLE stats_user_daily
    ADD COLUMN actual_total_cost DOUBLE NOT NULL DEFAULT 0,
    ADD COLUMN response_time_sum_ms DOUBLE NOT NULL DEFAULT 0,
    ADD COLUMN response_time_samples BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN effective_input_tokens BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN total_input_context BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN cache_creation_cost DOUBLE NOT NULL DEFAULT 0,
    ADD COLUMN cache_read_cost DOUBLE NOT NULL DEFAULT 0,
    ADD COLUMN cache_creation_ephemeral_5m_tokens BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN cache_creation_ephemeral_1h_tokens BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN settled_total_cost DOUBLE NOT NULL DEFAULT 0,
    ADD COLUMN settled_total_requests BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN settled_input_tokens BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN settled_output_tokens BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN settled_cache_creation_tokens BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN settled_cache_read_tokens BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN settled_first_finalized_at_unix_secs BIGINT,
    ADD COLUMN settled_last_finalized_at_unix_secs BIGINT;

ALTER TABLE stats_hourly_user
    ADD COLUMN cache_creation_tokens BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN cache_read_tokens BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN actual_total_cost DOUBLE NOT NULL DEFAULT 0,
    ADD COLUMN response_time_sum_ms DOUBLE NOT NULL DEFAULT 0,
    ADD COLUMN response_time_samples BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN settled_total_cost DOUBLE NOT NULL DEFAULT 0,
    ADD COLUMN settled_total_requests BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN settled_input_tokens BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN settled_output_tokens BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN settled_cache_creation_tokens BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN settled_cache_read_tokens BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN settled_first_finalized_at_unix_secs BIGINT,
    ADD COLUMN settled_last_finalized_at_unix_secs BIGINT;

ALTER TABLE stats_daily
    ADD COLUMN effective_input_tokens BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN total_input_context BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN response_time_sum_ms DOUBLE NOT NULL DEFAULT 0,
    ADD COLUMN response_time_samples BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN cache_creation_ephemeral_5m_tokens BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN cache_creation_ephemeral_1h_tokens BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN cache_hit_total_requests BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN cache_hit_requests BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN completed_total_requests BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN completed_cache_hit_requests BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN completed_input_tokens BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN completed_cache_creation_tokens BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN completed_cache_read_tokens BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN completed_total_input_context BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN completed_cache_creation_cost DOUBLE NOT NULL DEFAULT 0,
    ADD COLUMN completed_cache_read_cost DOUBLE NOT NULL DEFAULT 0,
    ADD COLUMN settled_total_cost DOUBLE NOT NULL DEFAULT 0,
    ADD COLUMN settled_total_requests BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN settled_input_tokens BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN settled_output_tokens BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN settled_cache_creation_tokens BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN settled_cache_read_tokens BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN settled_first_finalized_at_unix_secs BIGINT,
    ADD COLUMN settled_last_finalized_at_unix_secs BIGINT;

ALTER TABLE stats_hourly
    ADD COLUMN response_time_sum_ms DOUBLE NOT NULL DEFAULT 0,
    ADD COLUMN response_time_samples BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN cache_hit_total_requests BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN cache_hit_requests BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN completed_total_requests BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN completed_cache_hit_requests BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN completed_input_tokens BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN completed_cache_creation_tokens BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN completed_cache_read_tokens BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN completed_total_input_context BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN completed_cache_creation_cost DOUBLE NOT NULL DEFAULT 0,
    ADD COLUMN completed_cache_read_cost DOUBLE NOT NULL DEFAULT 0,
    ADD COLUMN settled_total_cost DOUBLE NOT NULL DEFAULT 0,
    ADD COLUMN settled_total_requests BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN settled_input_tokens BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN settled_output_tokens BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN settled_cache_creation_tokens BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN settled_cache_read_tokens BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN settled_first_finalized_at_unix_secs BIGINT,
    ADD COLUMN settled_last_finalized_at_unix_secs BIGINT;

ALTER TABLE stats_daily_model
    ADD COLUMN response_time_sum_ms DOUBLE NOT NULL DEFAULT 0,
    ADD COLUMN response_time_samples BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN cache_creation_ephemeral_5m_tokens BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN cache_creation_ephemeral_1h_tokens BIGINT NOT NULL DEFAULT 0;

ALTER TABLE stats_hourly_model
    ADD COLUMN response_time_sum_ms DOUBLE NOT NULL DEFAULT 0,
    ADD COLUMN response_time_samples BIGINT NOT NULL DEFAULT 0;

ALTER TABLE stats_hourly_user_model
    ADD COLUMN response_time_sum_ms DOUBLE NOT NULL DEFAULT 0,
    ADD COLUMN response_time_samples BIGINT NOT NULL DEFAULT 0;

CREATE TABLE stats_user_summary (
    id VARCHAR(64) NOT NULL,
    user_id VARCHAR(64) NOT NULL,
    username VARCHAR(255),
    cutoff_date BIGINT NOT NULL,
    all_time_requests BIGINT NOT NULL DEFAULT 0,
    all_time_success_requests BIGINT NOT NULL DEFAULT 0,
    all_time_error_requests BIGINT NOT NULL DEFAULT 0,
    all_time_input_tokens BIGINT NOT NULL DEFAULT 0,
    all_time_output_tokens BIGINT NOT NULL DEFAULT 0,
    all_time_cache_creation_tokens BIGINT NOT NULL DEFAULT 0,
    all_time_cache_read_tokens BIGINT NOT NULL DEFAULT 0,
    all_time_cost DOUBLE NOT NULL DEFAULT 0,
    all_time_actual_cost DOUBLE NOT NULL DEFAULT 0,
    active_days BIGINT NOT NULL DEFAULT 0,
    first_active_date BIGINT,
    last_active_date BIGINT,
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL,
    PRIMARY KEY (id),
    UNIQUE KEY uq_stats_user_summary_user_id (user_id),
    KEY idx_stats_user_summary_cutoff_date (cutoff_date)
);

CREATE TABLE stats_user_daily_model (
    id VARCHAR(64) NOT NULL,
    user_id VARCHAR(64) NOT NULL,
    username VARCHAR(255),
    `date` BIGINT NOT NULL,
    model VARCHAR(255) NOT NULL,
    total_requests BIGINT NOT NULL DEFAULT 0,
    success_requests BIGINT NOT NULL DEFAULT 0,
    input_tokens BIGINT NOT NULL DEFAULT 0,
    effective_input_tokens BIGINT NOT NULL DEFAULT 0,
    output_tokens BIGINT NOT NULL DEFAULT 0,
    total_tokens BIGINT NOT NULL DEFAULT 0,
    total_input_context BIGINT NOT NULL DEFAULT 0,
    cache_creation_tokens BIGINT NOT NULL DEFAULT 0,
    cache_creation_ephemeral_5m_tokens BIGINT NOT NULL DEFAULT 0,
    cache_creation_ephemeral_1h_tokens BIGINT NOT NULL DEFAULT 0,
    cache_read_tokens BIGINT NOT NULL DEFAULT 0,
    total_cost DOUBLE NOT NULL DEFAULT 0,
    actual_total_cost DOUBLE NOT NULL DEFAULT 0,
    response_time_sum_ms DOUBLE NOT NULL DEFAULT 0,
    response_time_samples BIGINT NOT NULL DEFAULT 0,
    successful_response_time_sum_ms DOUBLE NOT NULL DEFAULT 0,
    successful_response_time_samples BIGINT NOT NULL DEFAULT 0,
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL,
    PRIMARY KEY (id),
    UNIQUE KEY uq_stats_user_daily_model (`user_id`, `date`, `model`),
    KEY idx_stats_user_daily_model_date (`date`),
    KEY idx_stats_user_daily_model_user_id (`user_id`)
);

CREATE TABLE stats_user_daily_provider (
    id VARCHAR(64) NOT NULL,
    user_id VARCHAR(64) NOT NULL,
    username VARCHAR(255),
    `date` BIGINT NOT NULL,
    provider_name VARCHAR(255) NOT NULL,
    total_requests BIGINT NOT NULL DEFAULT 0,
    success_requests BIGINT NOT NULL DEFAULT 0,
    input_tokens BIGINT NOT NULL DEFAULT 0,
    effective_input_tokens BIGINT NOT NULL DEFAULT 0,
    output_tokens BIGINT NOT NULL DEFAULT 0,
    total_tokens BIGINT NOT NULL DEFAULT 0,
    total_input_context BIGINT NOT NULL DEFAULT 0,
    cache_creation_tokens BIGINT NOT NULL DEFAULT 0,
    cache_creation_ephemeral_5m_tokens BIGINT NOT NULL DEFAULT 0,
    cache_creation_ephemeral_1h_tokens BIGINT NOT NULL DEFAULT 0,
    cache_read_tokens BIGINT NOT NULL DEFAULT 0,
    total_cost DOUBLE NOT NULL DEFAULT 0,
    actual_total_cost DOUBLE NOT NULL DEFAULT 0,
    response_time_sum_ms DOUBLE NOT NULL DEFAULT 0,
    response_time_samples BIGINT NOT NULL DEFAULT 0,
    successful_response_time_sum_ms DOUBLE NOT NULL DEFAULT 0,
    successful_response_time_samples BIGINT NOT NULL DEFAULT 0,
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL,
    PRIMARY KEY (id),
    UNIQUE KEY uq_stats_user_daily_provider (`user_id`, `date`, `provider_name`),
    KEY idx_stats_user_daily_provider_date (`date`),
    KEY idx_stats_user_daily_provider_user_id (`user_id`)
);

CREATE TABLE stats_user_daily_api_format (
    id VARCHAR(64) NOT NULL,
    user_id VARCHAR(64) NOT NULL,
    username VARCHAR(255),
    `date` BIGINT NOT NULL,
    api_format VARCHAR(128) NOT NULL,
    total_requests BIGINT NOT NULL DEFAULT 0,
    success_requests BIGINT NOT NULL DEFAULT 0,
    input_tokens BIGINT NOT NULL DEFAULT 0,
    effective_input_tokens BIGINT NOT NULL DEFAULT 0,
    output_tokens BIGINT NOT NULL DEFAULT 0,
    total_tokens BIGINT NOT NULL DEFAULT 0,
    total_input_context BIGINT NOT NULL DEFAULT 0,
    cache_creation_tokens BIGINT NOT NULL DEFAULT 0,
    cache_creation_ephemeral_5m_tokens BIGINT NOT NULL DEFAULT 0,
    cache_creation_ephemeral_1h_tokens BIGINT NOT NULL DEFAULT 0,
    cache_read_tokens BIGINT NOT NULL DEFAULT 0,
    total_cost DOUBLE NOT NULL DEFAULT 0,
    actual_total_cost DOUBLE NOT NULL DEFAULT 0,
    response_time_sum_ms DOUBLE NOT NULL DEFAULT 0,
    response_time_samples BIGINT NOT NULL DEFAULT 0,
    successful_response_time_sum_ms DOUBLE NOT NULL DEFAULT 0,
    successful_response_time_samples BIGINT NOT NULL DEFAULT 0,
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL,
    PRIMARY KEY (id),
    UNIQUE KEY uq_stats_user_daily_api_format (`user_id`, `date`, `api_format`),
    KEY idx_stats_user_daily_api_format_date (`date`),
    KEY idx_stats_user_daily_api_format_user_id (`user_id`)
);

CREATE TABLE stats_daily_model_provider (
    id VARCHAR(64) NOT NULL,
    `date` BIGINT NOT NULL,
    model VARCHAR(255) NOT NULL,
    provider_name VARCHAR(255) NOT NULL,
    total_requests BIGINT NOT NULL DEFAULT 0,
    total_tokens BIGINT NOT NULL DEFAULT 0,
    total_cost DOUBLE NOT NULL DEFAULT 0,
    response_time_sum_ms DOUBLE NOT NULL DEFAULT 0,
    response_time_samples BIGINT NOT NULL DEFAULT 0,
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL,
    PRIMARY KEY (id),
    UNIQUE KEY uq_stats_daily_model_provider (`date`, `model`, `provider_name`),
    KEY idx_stats_daily_model_provider_date (`date`)
);

CREATE TABLE stats_user_daily_model_provider (
    id VARCHAR(64) NOT NULL,
    user_id VARCHAR(64) NOT NULL,
    username VARCHAR(255),
    `date` BIGINT NOT NULL,
    model VARCHAR(255) NOT NULL,
    provider_name VARCHAR(255) NOT NULL,
    total_requests BIGINT NOT NULL DEFAULT 0,
    total_tokens BIGINT NOT NULL DEFAULT 0,
    total_cost DOUBLE NOT NULL DEFAULT 0,
    response_time_sum_ms DOUBLE NOT NULL DEFAULT 0,
    response_time_samples BIGINT NOT NULL DEFAULT 0,
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL,
    PRIMARY KEY (id),
    UNIQUE KEY uq_stats_user_daily_model_provider (`user_id`, `date`, `model`, `provider_name`),
    KEY idx_stats_user_daily_model_provider_date (`date`),
    KEY idx_stats_user_daily_model_provider_user_date (`user_id`, `date`)
);

CREATE TABLE stats_daily_cost_savings (
    id VARCHAR(64) PRIMARY KEY,
    `date` BIGINT NOT NULL,
    cache_read_tokens BIGINT NOT NULL DEFAULT 0,
    cache_read_cost DOUBLE NOT NULL DEFAULT 0,
    cache_creation_cost DOUBLE NOT NULL DEFAULT 0,
    estimated_full_cost DOUBLE NOT NULL DEFAULT 0,
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL,
    UNIQUE KEY uq_stats_daily_cost_savings_date (`date`)
);

CREATE TABLE stats_daily_cost_savings_provider (
    id VARCHAR(64) PRIMARY KEY,
    `date` BIGINT NOT NULL,
    provider_name VARCHAR(255) NOT NULL,
    cache_read_tokens BIGINT NOT NULL DEFAULT 0,
    cache_read_cost DOUBLE NOT NULL DEFAULT 0,
    cache_creation_cost DOUBLE NOT NULL DEFAULT 0,
    estimated_full_cost DOUBLE NOT NULL DEFAULT 0,
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL,
    UNIQUE KEY uq_stats_daily_cost_savings_provider (`date`, `provider_name`)
);

CREATE TABLE stats_daily_cost_savings_model (
    id VARCHAR(64) PRIMARY KEY,
    `date` BIGINT NOT NULL,
    model VARCHAR(255) NOT NULL,
    cache_read_tokens BIGINT NOT NULL DEFAULT 0,
    cache_read_cost DOUBLE NOT NULL DEFAULT 0,
    cache_creation_cost DOUBLE NOT NULL DEFAULT 0,
    estimated_full_cost DOUBLE NOT NULL DEFAULT 0,
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL,
    UNIQUE KEY uq_stats_daily_cost_savings_model (`date`, `model`)
);

CREATE TABLE stats_daily_cost_savings_model_provider (
    id VARCHAR(64) PRIMARY KEY,
    `date` BIGINT NOT NULL,
    model VARCHAR(255) NOT NULL,
    provider_name VARCHAR(255) NOT NULL,
    cache_read_tokens BIGINT NOT NULL DEFAULT 0,
    cache_read_cost DOUBLE NOT NULL DEFAULT 0,
    cache_creation_cost DOUBLE NOT NULL DEFAULT 0,
    estimated_full_cost DOUBLE NOT NULL DEFAULT 0,
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL,
    UNIQUE KEY uq_stats_daily_cost_savings_model_provider (`date`, `model`, `provider_name`)
);

CREATE TABLE stats_user_daily_cost_savings (
    id VARCHAR(64) PRIMARY KEY,
    user_id VARCHAR(64) NOT NULL,
    username VARCHAR(255),
    `date` BIGINT NOT NULL,
    cache_read_tokens BIGINT NOT NULL DEFAULT 0,
    cache_read_cost DOUBLE NOT NULL DEFAULT 0,
    cache_creation_cost DOUBLE NOT NULL DEFAULT 0,
    estimated_full_cost DOUBLE NOT NULL DEFAULT 0,
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL,
    UNIQUE KEY uq_stats_user_daily_cost_savings (`user_id`, `date`)
);

CREATE TABLE stats_user_daily_cost_savings_provider (
    id VARCHAR(64) PRIMARY KEY,
    user_id VARCHAR(64) NOT NULL,
    username VARCHAR(255),
    `date` BIGINT NOT NULL,
    provider_name VARCHAR(255) NOT NULL,
    cache_read_tokens BIGINT NOT NULL DEFAULT 0,
    cache_read_cost DOUBLE NOT NULL DEFAULT 0,
    cache_creation_cost DOUBLE NOT NULL DEFAULT 0,
    estimated_full_cost DOUBLE NOT NULL DEFAULT 0,
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL,
    UNIQUE KEY uq_stats_user_daily_cost_savings_provider (`user_id`, `date`, `provider_name`)
);

CREATE TABLE stats_user_daily_cost_savings_model (
    id VARCHAR(64) PRIMARY KEY,
    user_id VARCHAR(64) NOT NULL,
    username VARCHAR(255),
    `date` BIGINT NOT NULL,
    model VARCHAR(255) NOT NULL,
    cache_read_tokens BIGINT NOT NULL DEFAULT 0,
    cache_read_cost DOUBLE NOT NULL DEFAULT 0,
    cache_creation_cost DOUBLE NOT NULL DEFAULT 0,
    estimated_full_cost DOUBLE NOT NULL DEFAULT 0,
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL,
    UNIQUE KEY uq_stats_user_daily_cost_savings_model (`user_id`, `date`, `model`)
);

CREATE TABLE stats_user_daily_cost_savings_model_provider (
    id VARCHAR(64) PRIMARY KEY,
    user_id VARCHAR(64) NOT NULL,
    username VARCHAR(255),
    `date` BIGINT NOT NULL,
    model VARCHAR(255) NOT NULL,
    provider_name VARCHAR(255) NOT NULL,
    cache_read_tokens BIGINT NOT NULL DEFAULT 0,
    cache_read_cost DOUBLE NOT NULL DEFAULT 0,
    cache_creation_cost DOUBLE NOT NULL DEFAULT 0,
    estimated_full_cost DOUBLE NOT NULL DEFAULT 0,
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL,
    UNIQUE KEY uq_stats_user_daily_cost_savings_model_provider (`user_id`, `date`, `model`, `provider_name`)
);

CREATE INDEX idx_stats_daily_cost_savings_provider_date
    ON stats_daily_cost_savings_provider (`date`);
CREATE INDEX idx_stats_daily_cost_savings_model_date
    ON stats_daily_cost_savings_model (`date`);
CREATE INDEX idx_stats_daily_cost_savings_model_provider_date
    ON stats_daily_cost_savings_model_provider (`date`);
CREATE INDEX idx_stats_user_daily_cost_savings_date
    ON stats_user_daily_cost_savings (`date`);
CREATE INDEX idx_stats_user_daily_cost_savings_provider_date
    ON stats_user_daily_cost_savings_provider (`date`);
CREATE INDEX idx_stats_user_daily_cost_savings_model_date
    ON stats_user_daily_cost_savings_model (`date`);
CREATE INDEX idx_stats_user_daily_cost_savings_model_provider_date
    ON stats_user_daily_cost_savings_model_provider (`date`);

-- Existing completed buckets predate the enriched dimensions above. Preserve the rows for
-- reads, but make the bounded aggregation worker replay every historical bucket.
UPDATE stats_hourly SET is_complete = 0 WHERE is_complete <> 0;
UPDATE stats_daily SET is_complete = 0 WHERE is_complete <> 0;

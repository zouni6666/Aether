ALTER TABLE stats_user_daily
    ADD COLUMN actual_total_cost REAL NOT NULL DEFAULT 0;
ALTER TABLE stats_user_daily
    ADD COLUMN response_time_sum_ms REAL NOT NULL DEFAULT 0;
ALTER TABLE stats_user_daily
    ADD COLUMN response_time_samples INTEGER NOT NULL DEFAULT 0;
ALTER TABLE stats_user_daily
    ADD COLUMN effective_input_tokens INTEGER NOT NULL DEFAULT 0;
ALTER TABLE stats_user_daily
    ADD COLUMN total_input_context INTEGER NOT NULL DEFAULT 0;
ALTER TABLE stats_user_daily
    ADD COLUMN cache_creation_cost REAL NOT NULL DEFAULT 0;
ALTER TABLE stats_user_daily
    ADD COLUMN cache_read_cost REAL NOT NULL DEFAULT 0;
ALTER TABLE stats_user_daily
    ADD COLUMN cache_creation_ephemeral_5m_tokens INTEGER NOT NULL DEFAULT 0;
ALTER TABLE stats_user_daily
    ADD COLUMN cache_creation_ephemeral_1h_tokens INTEGER NOT NULL DEFAULT 0;
ALTER TABLE stats_user_daily
    ADD COLUMN settled_total_cost REAL NOT NULL DEFAULT 0;
ALTER TABLE stats_user_daily
    ADD COLUMN settled_total_requests INTEGER NOT NULL DEFAULT 0;
ALTER TABLE stats_user_daily
    ADD COLUMN settled_input_tokens INTEGER NOT NULL DEFAULT 0;
ALTER TABLE stats_user_daily
    ADD COLUMN settled_output_tokens INTEGER NOT NULL DEFAULT 0;
ALTER TABLE stats_user_daily
    ADD COLUMN settled_cache_creation_tokens INTEGER NOT NULL DEFAULT 0;
ALTER TABLE stats_user_daily
    ADD COLUMN settled_cache_read_tokens INTEGER NOT NULL DEFAULT 0;
ALTER TABLE stats_user_daily
    ADD COLUMN settled_first_finalized_at_unix_secs INTEGER;
ALTER TABLE stats_user_daily
    ADD COLUMN settled_last_finalized_at_unix_secs INTEGER;

ALTER TABLE stats_hourly_user
    ADD COLUMN cache_creation_tokens INTEGER NOT NULL DEFAULT 0;
ALTER TABLE stats_hourly_user
    ADD COLUMN cache_read_tokens INTEGER NOT NULL DEFAULT 0;
ALTER TABLE stats_hourly_user
    ADD COLUMN actual_total_cost REAL NOT NULL DEFAULT 0;
ALTER TABLE stats_hourly_user
    ADD COLUMN response_time_sum_ms REAL NOT NULL DEFAULT 0;
ALTER TABLE stats_hourly_user
    ADD COLUMN response_time_samples INTEGER NOT NULL DEFAULT 0;
ALTER TABLE stats_hourly_user
    ADD COLUMN settled_total_cost REAL NOT NULL DEFAULT 0;
ALTER TABLE stats_hourly_user
    ADD COLUMN settled_total_requests INTEGER NOT NULL DEFAULT 0;
ALTER TABLE stats_hourly_user
    ADD COLUMN settled_input_tokens INTEGER NOT NULL DEFAULT 0;
ALTER TABLE stats_hourly_user
    ADD COLUMN settled_output_tokens INTEGER NOT NULL DEFAULT 0;
ALTER TABLE stats_hourly_user
    ADD COLUMN settled_cache_creation_tokens INTEGER NOT NULL DEFAULT 0;
ALTER TABLE stats_hourly_user
    ADD COLUMN settled_cache_read_tokens INTEGER NOT NULL DEFAULT 0;
ALTER TABLE stats_hourly_user
    ADD COLUMN settled_first_finalized_at_unix_secs INTEGER;
ALTER TABLE stats_hourly_user
    ADD COLUMN settled_last_finalized_at_unix_secs INTEGER;

ALTER TABLE stats_daily
    ADD COLUMN effective_input_tokens INTEGER NOT NULL DEFAULT 0;
ALTER TABLE stats_daily
    ADD COLUMN total_input_context INTEGER NOT NULL DEFAULT 0;
ALTER TABLE stats_daily
    ADD COLUMN response_time_sum_ms REAL NOT NULL DEFAULT 0;
ALTER TABLE stats_daily
    ADD COLUMN response_time_samples INTEGER NOT NULL DEFAULT 0;
ALTER TABLE stats_daily
    ADD COLUMN cache_creation_ephemeral_5m_tokens INTEGER NOT NULL DEFAULT 0;
ALTER TABLE stats_daily
    ADD COLUMN cache_creation_ephemeral_1h_tokens INTEGER NOT NULL DEFAULT 0;
ALTER TABLE stats_daily
    ADD COLUMN cache_hit_total_requests INTEGER NOT NULL DEFAULT 0;
ALTER TABLE stats_daily
    ADD COLUMN cache_hit_requests INTEGER NOT NULL DEFAULT 0;
ALTER TABLE stats_daily
    ADD COLUMN completed_total_requests INTEGER NOT NULL DEFAULT 0;
ALTER TABLE stats_daily
    ADD COLUMN completed_cache_hit_requests INTEGER NOT NULL DEFAULT 0;
ALTER TABLE stats_daily
    ADD COLUMN completed_input_tokens INTEGER NOT NULL DEFAULT 0;
ALTER TABLE stats_daily
    ADD COLUMN completed_cache_creation_tokens INTEGER NOT NULL DEFAULT 0;
ALTER TABLE stats_daily
    ADD COLUMN completed_cache_read_tokens INTEGER NOT NULL DEFAULT 0;
ALTER TABLE stats_daily
    ADD COLUMN completed_total_input_context INTEGER NOT NULL DEFAULT 0;
ALTER TABLE stats_daily
    ADD COLUMN completed_cache_creation_cost REAL NOT NULL DEFAULT 0;
ALTER TABLE stats_daily
    ADD COLUMN completed_cache_read_cost REAL NOT NULL DEFAULT 0;
ALTER TABLE stats_daily
    ADD COLUMN settled_total_cost REAL NOT NULL DEFAULT 0;
ALTER TABLE stats_daily
    ADD COLUMN settled_total_requests INTEGER NOT NULL DEFAULT 0;
ALTER TABLE stats_daily
    ADD COLUMN settled_input_tokens INTEGER NOT NULL DEFAULT 0;
ALTER TABLE stats_daily
    ADD COLUMN settled_output_tokens INTEGER NOT NULL DEFAULT 0;
ALTER TABLE stats_daily
    ADD COLUMN settled_cache_creation_tokens INTEGER NOT NULL DEFAULT 0;
ALTER TABLE stats_daily
    ADD COLUMN settled_cache_read_tokens INTEGER NOT NULL DEFAULT 0;
ALTER TABLE stats_daily
    ADD COLUMN settled_first_finalized_at_unix_secs INTEGER;
ALTER TABLE stats_daily
    ADD COLUMN settled_last_finalized_at_unix_secs INTEGER;

ALTER TABLE stats_hourly
    ADD COLUMN response_time_sum_ms REAL NOT NULL DEFAULT 0;
ALTER TABLE stats_hourly
    ADD COLUMN response_time_samples INTEGER NOT NULL DEFAULT 0;
ALTER TABLE stats_hourly
    ADD COLUMN cache_hit_total_requests INTEGER NOT NULL DEFAULT 0;
ALTER TABLE stats_hourly
    ADD COLUMN cache_hit_requests INTEGER NOT NULL DEFAULT 0;
ALTER TABLE stats_hourly
    ADD COLUMN completed_total_requests INTEGER NOT NULL DEFAULT 0;
ALTER TABLE stats_hourly
    ADD COLUMN completed_cache_hit_requests INTEGER NOT NULL DEFAULT 0;
ALTER TABLE stats_hourly
    ADD COLUMN completed_input_tokens INTEGER NOT NULL DEFAULT 0;
ALTER TABLE stats_hourly
    ADD COLUMN completed_cache_creation_tokens INTEGER NOT NULL DEFAULT 0;
ALTER TABLE stats_hourly
    ADD COLUMN completed_cache_read_tokens INTEGER NOT NULL DEFAULT 0;
ALTER TABLE stats_hourly
    ADD COLUMN completed_total_input_context INTEGER NOT NULL DEFAULT 0;
ALTER TABLE stats_hourly
    ADD COLUMN completed_cache_creation_cost REAL NOT NULL DEFAULT 0;
ALTER TABLE stats_hourly
    ADD COLUMN completed_cache_read_cost REAL NOT NULL DEFAULT 0;
ALTER TABLE stats_hourly
    ADD COLUMN settled_total_cost REAL NOT NULL DEFAULT 0;
ALTER TABLE stats_hourly
    ADD COLUMN settled_total_requests INTEGER NOT NULL DEFAULT 0;
ALTER TABLE stats_hourly
    ADD COLUMN settled_input_tokens INTEGER NOT NULL DEFAULT 0;
ALTER TABLE stats_hourly
    ADD COLUMN settled_output_tokens INTEGER NOT NULL DEFAULT 0;
ALTER TABLE stats_hourly
    ADD COLUMN settled_cache_creation_tokens INTEGER NOT NULL DEFAULT 0;
ALTER TABLE stats_hourly
    ADD COLUMN settled_cache_read_tokens INTEGER NOT NULL DEFAULT 0;
ALTER TABLE stats_hourly
    ADD COLUMN settled_first_finalized_at_unix_secs INTEGER;
ALTER TABLE stats_hourly
    ADD COLUMN settled_last_finalized_at_unix_secs INTEGER;

ALTER TABLE stats_daily_model
    ADD COLUMN response_time_sum_ms REAL NOT NULL DEFAULT 0;
ALTER TABLE stats_daily_model
    ADD COLUMN response_time_samples INTEGER NOT NULL DEFAULT 0;
ALTER TABLE stats_daily_model
    ADD COLUMN cache_creation_ephemeral_5m_tokens INTEGER NOT NULL DEFAULT 0;
ALTER TABLE stats_daily_model
    ADD COLUMN cache_creation_ephemeral_1h_tokens INTEGER NOT NULL DEFAULT 0;
ALTER TABLE stats_hourly_model
    ADD COLUMN response_time_sum_ms REAL NOT NULL DEFAULT 0;
ALTER TABLE stats_hourly_model
    ADD COLUMN response_time_samples INTEGER NOT NULL DEFAULT 0;
ALTER TABLE stats_hourly_user_model
    ADD COLUMN response_time_sum_ms REAL NOT NULL DEFAULT 0;
ALTER TABLE stats_hourly_user_model
    ADD COLUMN response_time_samples INTEGER NOT NULL DEFAULT 0;

CREATE TABLE IF NOT EXISTS stats_user_summary (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL UNIQUE,
    username TEXT,
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
    active_days INTEGER NOT NULL DEFAULT 0,
    first_active_date INTEGER,
    last_active_date INTEGER,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);
CREATE INDEX idx_stats_user_summary_cutoff_date ON stats_user_summary (cutoff_date);

CREATE TABLE IF NOT EXISTS stats_user_daily_model (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL,
    username TEXT,
    "date" INTEGER NOT NULL,
    model TEXT NOT NULL,
    total_requests INTEGER NOT NULL DEFAULT 0,
    success_requests INTEGER NOT NULL DEFAULT 0,
    input_tokens INTEGER NOT NULL DEFAULT 0,
    effective_input_tokens INTEGER NOT NULL DEFAULT 0,
    output_tokens INTEGER NOT NULL DEFAULT 0,
    total_tokens INTEGER NOT NULL DEFAULT 0,
    total_input_context INTEGER NOT NULL DEFAULT 0,
    cache_creation_tokens INTEGER NOT NULL DEFAULT 0,
    cache_creation_ephemeral_5m_tokens INTEGER NOT NULL DEFAULT 0,
    cache_creation_ephemeral_1h_tokens INTEGER NOT NULL DEFAULT 0,
    cache_read_tokens INTEGER NOT NULL DEFAULT 0,
    total_cost REAL NOT NULL DEFAULT 0,
    actual_total_cost REAL NOT NULL DEFAULT 0,
    response_time_sum_ms REAL NOT NULL DEFAULT 0,
    response_time_samples INTEGER NOT NULL DEFAULT 0,
    successful_response_time_sum_ms REAL NOT NULL DEFAULT 0,
    successful_response_time_samples INTEGER NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    UNIQUE (user_id, "date", model)
);
CREATE INDEX idx_stats_user_daily_model_date ON stats_user_daily_model ("date");
CREATE INDEX idx_stats_user_daily_model_user_id ON stats_user_daily_model (user_id);

CREATE TABLE IF NOT EXISTS stats_user_daily_provider (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL,
    username TEXT,
    "date" INTEGER NOT NULL,
    provider_name TEXT NOT NULL,
    total_requests INTEGER NOT NULL DEFAULT 0,
    success_requests INTEGER NOT NULL DEFAULT 0,
    input_tokens INTEGER NOT NULL DEFAULT 0,
    effective_input_tokens INTEGER NOT NULL DEFAULT 0,
    output_tokens INTEGER NOT NULL DEFAULT 0,
    total_tokens INTEGER NOT NULL DEFAULT 0,
    total_input_context INTEGER NOT NULL DEFAULT 0,
    cache_creation_tokens INTEGER NOT NULL DEFAULT 0,
    cache_creation_ephemeral_5m_tokens INTEGER NOT NULL DEFAULT 0,
    cache_creation_ephemeral_1h_tokens INTEGER NOT NULL DEFAULT 0,
    cache_read_tokens INTEGER NOT NULL DEFAULT 0,
    total_cost REAL NOT NULL DEFAULT 0,
    actual_total_cost REAL NOT NULL DEFAULT 0,
    response_time_sum_ms REAL NOT NULL DEFAULT 0,
    response_time_samples INTEGER NOT NULL DEFAULT 0,
    successful_response_time_sum_ms REAL NOT NULL DEFAULT 0,
    successful_response_time_samples INTEGER NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    UNIQUE (user_id, "date", provider_name)
);
CREATE INDEX idx_stats_user_daily_provider_date ON stats_user_daily_provider ("date");
CREATE INDEX idx_stats_user_daily_provider_user_id ON stats_user_daily_provider (user_id);

CREATE TABLE IF NOT EXISTS stats_user_daily_api_format (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL,
    username TEXT,
    "date" INTEGER NOT NULL,
    api_format TEXT NOT NULL,
    total_requests INTEGER NOT NULL DEFAULT 0,
    success_requests INTEGER NOT NULL DEFAULT 0,
    input_tokens INTEGER NOT NULL DEFAULT 0,
    effective_input_tokens INTEGER NOT NULL DEFAULT 0,
    output_tokens INTEGER NOT NULL DEFAULT 0,
    total_tokens INTEGER NOT NULL DEFAULT 0,
    total_input_context INTEGER NOT NULL DEFAULT 0,
    cache_creation_tokens INTEGER NOT NULL DEFAULT 0,
    cache_creation_ephemeral_5m_tokens INTEGER NOT NULL DEFAULT 0,
    cache_creation_ephemeral_1h_tokens INTEGER NOT NULL DEFAULT 0,
    cache_read_tokens INTEGER NOT NULL DEFAULT 0,
    total_cost REAL NOT NULL DEFAULT 0,
    actual_total_cost REAL NOT NULL DEFAULT 0,
    response_time_sum_ms REAL NOT NULL DEFAULT 0,
    response_time_samples INTEGER NOT NULL DEFAULT 0,
    successful_response_time_sum_ms REAL NOT NULL DEFAULT 0,
    successful_response_time_samples INTEGER NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    UNIQUE (user_id, "date", api_format)
);
CREATE INDEX idx_stats_user_daily_api_format_date ON stats_user_daily_api_format ("date");
CREATE INDEX idx_stats_user_daily_api_format_user_id ON stats_user_daily_api_format (user_id);

CREATE TABLE IF NOT EXISTS stats_daily_model_provider (
    id TEXT PRIMARY KEY NOT NULL,
    "date" INTEGER NOT NULL,
    model TEXT NOT NULL,
    provider_name TEXT NOT NULL,
    total_requests INTEGER NOT NULL DEFAULT 0,
    total_tokens INTEGER NOT NULL DEFAULT 0,
    total_cost REAL NOT NULL DEFAULT 0,
    response_time_sum_ms REAL NOT NULL DEFAULT 0,
    response_time_samples INTEGER NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    UNIQUE ("date", model, provider_name)
);
CREATE INDEX idx_stats_daily_model_provider_date ON stats_daily_model_provider ("date");

CREATE TABLE IF NOT EXISTS stats_user_daily_model_provider (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL,
    username TEXT,
    "date" INTEGER NOT NULL,
    model TEXT NOT NULL,
    provider_name TEXT NOT NULL,
    total_requests INTEGER NOT NULL DEFAULT 0,
    total_tokens INTEGER NOT NULL DEFAULT 0,
    total_cost REAL NOT NULL DEFAULT 0,
    response_time_sum_ms REAL NOT NULL DEFAULT 0,
    response_time_samples INTEGER NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    UNIQUE (user_id, "date", model, provider_name)
);
CREATE INDEX idx_stats_user_daily_model_provider_date
    ON stats_user_daily_model_provider ("date");
CREATE INDEX idx_stats_user_daily_model_provider_user_date
    ON stats_user_daily_model_provider (user_id, "date");

CREATE TABLE IF NOT EXISTS stats_daily_cost_savings (
    id TEXT PRIMARY KEY NOT NULL,
    "date" INTEGER NOT NULL UNIQUE,
    cache_read_tokens INTEGER NOT NULL DEFAULT 0,
    cache_read_cost REAL NOT NULL DEFAULT 0,
    cache_creation_cost REAL NOT NULL DEFAULT 0,
    estimated_full_cost REAL NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS stats_daily_cost_savings_provider (
    id TEXT PRIMARY KEY NOT NULL,
    "date" INTEGER NOT NULL,
    provider_name TEXT NOT NULL,
    cache_read_tokens INTEGER NOT NULL DEFAULT 0,
    cache_read_cost REAL NOT NULL DEFAULT 0,
    cache_creation_cost REAL NOT NULL DEFAULT 0,
    estimated_full_cost REAL NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    UNIQUE ("date", provider_name)
);
CREATE INDEX idx_stats_daily_cost_savings_provider_date
    ON stats_daily_cost_savings_provider ("date");

CREATE TABLE IF NOT EXISTS stats_daily_cost_savings_model (
    id TEXT PRIMARY KEY NOT NULL,
    "date" INTEGER NOT NULL,
    model TEXT NOT NULL,
    cache_read_tokens INTEGER NOT NULL DEFAULT 0,
    cache_read_cost REAL NOT NULL DEFAULT 0,
    cache_creation_cost REAL NOT NULL DEFAULT 0,
    estimated_full_cost REAL NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    UNIQUE ("date", model)
);
CREATE INDEX idx_stats_daily_cost_savings_model_date
    ON stats_daily_cost_savings_model ("date");

CREATE TABLE IF NOT EXISTS stats_daily_cost_savings_model_provider (
    id TEXT PRIMARY KEY NOT NULL,
    "date" INTEGER NOT NULL,
    model TEXT NOT NULL,
    provider_name TEXT NOT NULL,
    cache_read_tokens INTEGER NOT NULL DEFAULT 0,
    cache_read_cost REAL NOT NULL DEFAULT 0,
    cache_creation_cost REAL NOT NULL DEFAULT 0,
    estimated_full_cost REAL NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    UNIQUE ("date", model, provider_name)
);
CREATE INDEX idx_stats_daily_cost_savings_model_provider_date
    ON stats_daily_cost_savings_model_provider ("date");

CREATE TABLE IF NOT EXISTS stats_user_daily_cost_savings (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL,
    username TEXT,
    "date" INTEGER NOT NULL,
    cache_read_tokens INTEGER NOT NULL DEFAULT 0,
    cache_read_cost REAL NOT NULL DEFAULT 0,
    cache_creation_cost REAL NOT NULL DEFAULT 0,
    estimated_full_cost REAL NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    UNIQUE (user_id, "date")
);
CREATE INDEX idx_stats_user_daily_cost_savings_date
    ON stats_user_daily_cost_savings ("date");

CREATE TABLE IF NOT EXISTS stats_user_daily_cost_savings_provider (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL,
    username TEXT,
    "date" INTEGER NOT NULL,
    provider_name TEXT NOT NULL,
    cache_read_tokens INTEGER NOT NULL DEFAULT 0,
    cache_read_cost REAL NOT NULL DEFAULT 0,
    cache_creation_cost REAL NOT NULL DEFAULT 0,
    estimated_full_cost REAL NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    UNIQUE (user_id, "date", provider_name)
);
CREATE INDEX idx_stats_user_daily_cost_savings_provider_date
    ON stats_user_daily_cost_savings_provider ("date");

CREATE TABLE IF NOT EXISTS stats_user_daily_cost_savings_model (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL,
    username TEXT,
    "date" INTEGER NOT NULL,
    model TEXT NOT NULL,
    cache_read_tokens INTEGER NOT NULL DEFAULT 0,
    cache_read_cost REAL NOT NULL DEFAULT 0,
    cache_creation_cost REAL NOT NULL DEFAULT 0,
    estimated_full_cost REAL NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    UNIQUE (user_id, "date", model)
);
CREATE INDEX idx_stats_user_daily_cost_savings_model_date
    ON stats_user_daily_cost_savings_model ("date");

CREATE TABLE IF NOT EXISTS stats_user_daily_cost_savings_model_provider (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL,
    username TEXT,
    "date" INTEGER NOT NULL,
    model TEXT NOT NULL,
    provider_name TEXT NOT NULL,
    cache_read_tokens INTEGER NOT NULL DEFAULT 0,
    cache_read_cost REAL NOT NULL DEFAULT 0,
    cache_creation_cost REAL NOT NULL DEFAULT 0,
    estimated_full_cost REAL NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    UNIQUE (user_id, "date", model, provider_name)
);
CREATE INDEX idx_stats_user_daily_cost_savings_model_provider_date
    ON stats_user_daily_cost_savings_model_provider ("date");

-- Existing completed buckets predate the enriched dimensions above. Preserve the rows for
-- reads, but make the bounded aggregation worker replay every historical bucket.
UPDATE stats_hourly SET is_complete = 0 WHERE is_complete <> 0;
UPDATE stats_daily SET is_complete = 0 WHERE is_complete <> 0;

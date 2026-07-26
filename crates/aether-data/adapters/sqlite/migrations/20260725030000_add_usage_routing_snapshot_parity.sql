CREATE TABLE IF NOT EXISTS usage_routing_snapshots (
    request_id TEXT PRIMARY KEY NOT NULL,
    candidate_id TEXT,
    candidate_index INTEGER,
    key_name TEXT,
    planner_kind TEXT,
    route_family TEXT,
    route_kind TEXT,
    execution_path TEXT,
    local_execution_runtime_miss_reason TEXT,
    selected_provider_id TEXT,
    selected_endpoint_id TEXT,
    selected_provider_api_key_id TEXT,
    has_format_conversion INTEGER,
    created_at INTEGER NOT NULL DEFAULT (CAST(strftime('%s', 'now') AS INTEGER)),
    updated_at INTEGER NOT NULL DEFAULT (CAST(strftime('%s', 'now') AS INTEGER)),
    FOREIGN KEY (request_id) REFERENCES "usage" (request_id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS ix_usage_routing_snapshots_route_family_kind
    ON usage_routing_snapshots (route_family, route_kind);
CREATE INDEX IF NOT EXISTS ix_usage_routing_snapshots_candidate_id
    ON usage_routing_snapshots (candidate_id);

INSERT INTO usage_routing_snapshots (
    request_id,
    candidate_id,
    candidate_index,
    key_name,
    planner_kind,
    route_family,
    route_kind,
    execution_path,
    local_execution_runtime_miss_reason,
    selected_provider_id,
    selected_endpoint_id,
    selected_provider_api_key_id,
    has_format_conversion,
    created_at,
    updated_at
)
SELECT
    request_id,
    candidate_id,
    candidate_index,
    key_name,
    planner_kind,
    route_family,
    route_kind,
    execution_path,
    local_execution_runtime_miss_reason,
    provider_id,
    provider_endpoint_id,
    provider_api_key_id,
    has_format_conversion,
    COALESCE(NULLIF(created_at_unix_ms, 0), NULLIF(updated_at_unix_secs, 0), CAST(strftime('%s', 'now') AS INTEGER)),
    COALESCE(NULLIF(updated_at_unix_secs, 0), NULLIF(created_at_unix_ms, 0), CAST(strftime('%s', 'now') AS INTEGER))
FROM "usage"
WHERE candidate_id IS NOT NULL
   OR candidate_index IS NOT NULL
   OR key_name IS NOT NULL
   OR planner_kind IS NOT NULL
   OR route_family IS NOT NULL
   OR route_kind IS NOT NULL
   OR execution_path IS NOT NULL
   OR local_execution_runtime_miss_reason IS NOT NULL
   OR provider_id IS NOT NULL
   OR provider_endpoint_id IS NOT NULL
   OR provider_api_key_id IS NOT NULL
   OR has_format_conversion <> 0;

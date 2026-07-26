CREATE TABLE IF NOT EXISTS usage_routing_snapshots (
    `request_id` VARCHAR(128) NOT NULL,
    `candidate_id` VARCHAR(160),
    `candidate_index` BIGINT,
    `key_name` VARCHAR(255),
    `planner_kind` VARCHAR(120),
    `route_family` VARCHAR(80),
    `route_kind` VARCHAR(80),
    `execution_path` VARCHAR(80),
    `local_execution_runtime_miss_reason` VARCHAR(255),
    `selected_provider_id` VARCHAR(100),
    `selected_endpoint_id` VARCHAR(100),
    `selected_provider_api_key_id` VARCHAR(100),
    `has_format_conversion` TINYINT(1),
    `created_at` BIGINT NOT NULL DEFAULT (UNIX_TIMESTAMP()),
    `updated_at` BIGINT NOT NULL DEFAULT (UNIX_TIMESTAMP()),
    PRIMARY KEY (`request_id`),
    KEY ix_usage_routing_snapshots_route_family_kind (`route_family`, `route_kind`),
    KEY ix_usage_routing_snapshots_candidate_id (`candidate_id`),
    CONSTRAINT usage_routing_snapshots_request_id_fkey
        FOREIGN KEY (`request_id`) REFERENCES `usage` (`request_id`) ON DELETE CASCADE
);

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
    COALESCE(NULLIF(created_at_unix_ms, 0), NULLIF(updated_at_unix_secs, 0), UNIX_TIMESTAMP()),
    COALESCE(NULLIF(updated_at_unix_secs, 0), NULLIF(created_at_unix_ms, 0), UNIX_TIMESTAMP())
FROM `usage`
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

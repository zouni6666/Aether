UPDATE routing_groups
SET is_system_default = 0
WHERE id IN (
    SELECT id
    FROM (
        SELECT
            id,
            ROW_NUMBER() OVER (ORDER BY enabled DESC, updated_at DESC, id ASC) AS default_rank
        FROM routing_groups
        WHERE is_system_default = 1
    ) AS ranked_defaults
    WHERE default_rank > 1
);

UPDATE routing_group_bindings
SET is_default = 0
WHERE id IN (
    SELECT id
    FROM (
        SELECT
            id,
            ROW_NUMBER() OVER (
                PARTITION BY subject_type, subject_id
                ORDER BY created_at ASC, id ASC
            ) AS default_rank
        FROM routing_group_bindings
        WHERE is_default = 1
    ) AS ranked_defaults
    WHERE default_rank > 1
);

CREATE UNIQUE INDEX IF NOT EXISTS routing_groups_one_system_default_key
    ON routing_groups (is_system_default)
    WHERE is_system_default = 1;

CREATE UNIQUE INDEX IF NOT EXISTS routing_group_bindings_subject_default_key
    ON routing_group_bindings (subject_type, subject_id)
    WHERE is_default = 1;

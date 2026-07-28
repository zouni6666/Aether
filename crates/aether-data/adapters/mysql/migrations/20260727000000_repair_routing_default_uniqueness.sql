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

SET @aether_routing_groups_default_guard_column_sql := IF(
    (
        SELECT COUNT(*)
        FROM information_schema.columns
        WHERE table_schema = DATABASE()
          AND table_name = 'routing_groups'
          AND column_name = 'system_default_unique_guard'
    ) = 0,
    'ALTER TABLE routing_groups ADD COLUMN system_default_unique_guard TINYINT GENERATED ALWAYS AS (CASE WHEN is_system_default = 1 THEN 1 ELSE NULL END) VIRTUAL INVISIBLE',
    'DO 0'
);

PREPARE aether_routing_groups_default_guard_column_stmt
    FROM @aether_routing_groups_default_guard_column_sql;
EXECUTE aether_routing_groups_default_guard_column_stmt;
DEALLOCATE PREPARE aether_routing_groups_default_guard_column_stmt;

SET @aether_routing_bindings_default_type_guard_column_sql := IF(
    (
        SELECT COUNT(*)
        FROM information_schema.columns
        WHERE table_schema = DATABASE()
          AND table_name = 'routing_group_bindings'
          AND column_name = 'default_subject_type_guard'
    ) = 0,
    'ALTER TABLE routing_group_bindings ADD COLUMN default_subject_type_guard VARCHAR(32) GENERATED ALWAYS AS (CASE WHEN is_default = 1 THEN subject_type ELSE NULL END) VIRTUAL INVISIBLE',
    'DO 0'
);

PREPARE aether_routing_bindings_default_type_guard_column_stmt
    FROM @aether_routing_bindings_default_type_guard_column_sql;
EXECUTE aether_routing_bindings_default_type_guard_column_stmt;
DEALLOCATE PREPARE aether_routing_bindings_default_type_guard_column_stmt;

SET @aether_routing_bindings_default_id_guard_column_sql := IF(
    (
        SELECT COUNT(*)
        FROM information_schema.columns
        WHERE table_schema = DATABASE()
          AND table_name = 'routing_group_bindings'
          AND column_name = 'default_subject_id_guard'
    ) = 0,
    'ALTER TABLE routing_group_bindings ADD COLUMN default_subject_id_guard VARCHAR(64) GENERATED ALWAYS AS (CASE WHEN is_default = 1 THEN subject_id ELSE NULL END) VIRTUAL INVISIBLE',
    'DO 0'
);

PREPARE aether_routing_bindings_default_id_guard_column_stmt
    FROM @aether_routing_bindings_default_id_guard_column_sql;
EXECUTE aether_routing_bindings_default_id_guard_column_stmt;
DEALLOCATE PREPARE aether_routing_bindings_default_id_guard_column_stmt;

SET @aether_routing_groups_default_unique_index_sql := IF(
    (
        SELECT COUNT(*)
        FROM information_schema.statistics
        WHERE table_schema = DATABASE()
          AND table_name = 'routing_groups'
          AND index_name = 'routing_groups_one_system_default_key'
    ) = 0,
    'CREATE UNIQUE INDEX routing_groups_one_system_default_key ON routing_groups (system_default_unique_guard)',
    'DO 0'
);

PREPARE aether_routing_groups_default_unique_index_stmt
    FROM @aether_routing_groups_default_unique_index_sql;
EXECUTE aether_routing_groups_default_unique_index_stmt;
DEALLOCATE PREPARE aether_routing_groups_default_unique_index_stmt;

SET @aether_routing_bindings_default_unique_index_sql := IF(
    (
        SELECT COUNT(*)
        FROM information_schema.statistics
        WHERE table_schema = DATABASE()
          AND table_name = 'routing_group_bindings'
          AND index_name = 'routing_group_bindings_subject_default_key'
    ) = 0,
    'CREATE UNIQUE INDEX routing_group_bindings_subject_default_key ON routing_group_bindings (default_subject_type_guard, default_subject_id_guard)',
    'DO 0'
);

PREPARE aether_routing_bindings_default_unique_index_stmt
    FROM @aether_routing_bindings_default_unique_index_sql;
EXECUTE aether_routing_bindings_default_unique_index_stmt;
DEALLOCATE PREPARE aether_routing_bindings_default_unique_index_stmt;

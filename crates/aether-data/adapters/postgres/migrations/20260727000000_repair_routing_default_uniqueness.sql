WITH ranked_defaults AS (
    SELECT
        id,
        ROW_NUMBER() OVER (ORDER BY enabled DESC, updated_at DESC, id ASC) AS default_rank
    FROM public.routing_groups
    WHERE is_system_default = TRUE
)
UPDATE public.routing_groups AS routing_group
SET is_system_default = FALSE
FROM ranked_defaults
WHERE routing_group.id = ranked_defaults.id
  AND ranked_defaults.default_rank > 1;

WITH ranked_defaults AS (
    SELECT
        id,
        ROW_NUMBER() OVER (
            PARTITION BY subject_type, subject_id
            ORDER BY created_at ASC, id ASC
        ) AS default_rank
    FROM public.routing_group_bindings
    WHERE is_default = TRUE
)
UPDATE public.routing_group_bindings AS binding
SET is_default = FALSE
FROM ranked_defaults
WHERE binding.id = ranked_defaults.id
  AND ranked_defaults.default_rank > 1;

CREATE UNIQUE INDEX IF NOT EXISTS routing_groups_one_system_default_key
    ON public.routing_groups (is_system_default)
    WHERE is_system_default = TRUE;

CREATE UNIQUE INDEX IF NOT EXISTS routing_group_bindings_subject_default_key
    ON public.routing_group_bindings (subject_type, subject_id)
    WHERE is_default = TRUE;

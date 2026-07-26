UPDATE global_models AS target
LEFT JOIN (
    SELECT
        `usage`.model,
        COUNT(*) AS usage_count
    FROM `usage`
    WHERE `usage`.model IS NOT NULL
      AND TRIM(`usage`.model) <> ''
      AND `usage`.status NOT IN ('pending', 'streaming')
    GROUP BY `usage`.model
) AS aggregated
  ON aggregated.model = target.name
SET
    target.usage_count = COALESCE(aggregated.usage_count, 0),
    target.updated_at = UNIX_TIMESTAMP();

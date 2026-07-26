UPDATE api_keys AS target
LEFT JOIN (
    SELECT
        `usage`.api_key_id,
        COUNT(*) AS total_requests,
        COALESCE(
            SUM(
                GREATEST(
                    COALESCE(
                        `usage`.total_tokens,
                        COALESCE(`usage`.input_tokens, 0) + COALESCE(`usage`.output_tokens, 0)
                    ),
                    0
                )
            ),
            0
        ) AS total_tokens,
        COALESCE(SUM(COALESCE(`usage`.total_cost_usd, 0)), 0) AS total_cost_usd,
        MAX(
            COALESCE(
                `usage`.created_at,
                `usage`.created_at_unix_ms,
                `usage`.updated_at_unix_secs
            )
        ) AS last_used_at
    FROM `usage`
    WHERE `usage`.api_key_id IS NOT NULL
      AND TRIM(`usage`.api_key_id) <> ''
    GROUP BY `usage`.api_key_id
) AS aggregated
  ON aggregated.api_key_id = target.id
SET
    target.total_requests = COALESCE(aggregated.total_requests, 0),
    target.total_tokens = COALESCE(aggregated.total_tokens, 0),
    target.total_cost_usd = COALESCE(aggregated.total_cost_usd, 0),
    target.last_used_at = aggregated.last_used_at;

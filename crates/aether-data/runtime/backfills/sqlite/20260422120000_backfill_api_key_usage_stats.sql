UPDATE api_keys AS target
SET
    total_requests = (
        SELECT COUNT(*)
        FROM "usage"
        WHERE "usage".api_key_id = target.id
    ),
    total_tokens = COALESCE((
        SELECT SUM(MAX(COALESCE("usage".total_tokens, 0), 0))
        FROM "usage"
        WHERE "usage".api_key_id = target.id
    ), 0),
    total_cost_usd = COALESCE((
        SELECT SUM(COALESCE("usage".total_cost_usd, 0))
        FROM "usage"
        WHERE "usage".api_key_id = target.id
    ), 0),
    last_used_at = (
        SELECT MAX(
            COALESCE(
                "usage".created_at,
                "usage".created_at_unix_ms,
                "usage".updated_at_unix_secs
            )
        )
        FROM "usage"
        WHERE "usage".api_key_id = target.id
    );

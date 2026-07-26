UPDATE global_models AS target
SET
    usage_count = (
        SELECT COUNT(*)
        FROM "usage"
        WHERE "usage".model = target.name
          AND "usage".status NOT IN ('pending', 'streaming')
    ),
    updated_at = CAST(strftime('%s', 'now') AS INTEGER);

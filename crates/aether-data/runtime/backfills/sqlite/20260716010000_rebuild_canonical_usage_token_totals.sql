WITH canonical_usage AS (
    SELECT
        "usage".api_key_id,
        MAX(
            COALESCE(
                CASE
                    WHEN settlement.billing_effective_input_tokens IS NOT NULL THEN
                        MAX(settlement.billing_effective_input_tokens, 0)
                        + MAX(COALESCE(settlement.billing_output_tokens, "usage".output_tokens, 0), 0)
                        + MAX(
                            COALESCE(
                                settlement.billing_cache_creation_tokens,
                                CASE
                                    WHEN settlement.billing_cache_creation_5m_tokens IS NOT NULL
                                      OR settlement.billing_cache_creation_1h_tokens IS NOT NULL
                                    THEN COALESCE(settlement.billing_cache_creation_5m_tokens, 0)
                                       + COALESCE(settlement.billing_cache_creation_1h_tokens, 0)
                                END,
                                CASE
                                    WHEN COALESCE("usage".cache_creation_input_tokens, 0) = 0
                                      AND (
                                        MAX(
                                            COALESCE("usage".cache_creation_input_tokens_5m, 0),
                                            COALESCE("usage".cache_creation_ephemeral_5m_input_tokens, 0)
                                        )
                                        + MAX(
                                            COALESCE("usage".cache_creation_input_tokens_1h, 0),
                                            COALESCE("usage".cache_creation_ephemeral_1h_input_tokens, 0)
                                        )
                                      ) > 0
                                    THEN MAX(
                                            COALESCE("usage".cache_creation_input_tokens_5m, 0),
                                            COALESCE("usage".cache_creation_ephemeral_5m_input_tokens, 0)
                                         )
                                       + MAX(
                                            COALESCE("usage".cache_creation_input_tokens_1h, 0),
                                            COALESCE("usage".cache_creation_ephemeral_1h_input_tokens, 0)
                                         )
                                    ELSE COALESCE("usage".cache_creation_input_tokens, 0)
                                END,
                                0
                            ),
                            0
                        )
                        + MAX(
                            COALESCE(
                                settlement.billing_cache_read_tokens,
                                "usage".cache_read_input_tokens,
                                0
                            ),
                            0
                        )
                    WHEN settlement.billing_total_input_context IS NOT NULL THEN
                        MAX(settlement.billing_total_input_context, 0)
                        + MAX(COALESCE(settlement.billing_output_tokens, "usage".output_tokens, 0), 0)
                END,
                NULLIF(MAX(COALESCE("usage".total_tokens, 0), 0), 0),
                CASE
                    WHEN LOWER(COALESCE("usage".endpoint_api_format, "usage".api_format, ''))
                           IN ('openai', 'gemini', 'google')
                      OR LOWER(COALESCE("usage".endpoint_api_format, "usage".api_format, ''))
                           LIKE 'openai:%'
                      OR LOWER(COALESCE("usage".endpoint_api_format, "usage".api_format, ''))
                           LIKE 'gemini:%'
                      OR LOWER(COALESCE("usage".endpoint_api_format, "usage".api_format, ''))
                           LIKE 'google:%'
                    THEN MAX(COALESCE("usage".input_tokens, 0), 0)
                       + MAX(COALESCE("usage".output_tokens, 0), 0)
                    ELSE MAX(COALESCE("usage".input_tokens, 0), 0)
                       + MAX(COALESCE("usage".output_tokens, 0), 0)
                       + MAX(
                            CASE
                                WHEN COALESCE("usage".cache_creation_input_tokens, 0) = 0
                                  AND (
                                    MAX(
                                        COALESCE("usage".cache_creation_input_tokens_5m, 0),
                                        COALESCE("usage".cache_creation_ephemeral_5m_input_tokens, 0)
                                    )
                                    + MAX(
                                        COALESCE("usage".cache_creation_input_tokens_1h, 0),
                                        COALESCE("usage".cache_creation_ephemeral_1h_input_tokens, 0)
                                    )
                                  ) > 0
                                THEN MAX(
                                        COALESCE("usage".cache_creation_input_tokens_5m, 0),
                                        COALESCE("usage".cache_creation_ephemeral_5m_input_tokens, 0)
                                     )
                                   + MAX(
                                        COALESCE("usage".cache_creation_input_tokens_1h, 0),
                                        COALESCE("usage".cache_creation_ephemeral_1h_input_tokens, 0)
                                     )
                                ELSE COALESCE("usage".cache_creation_input_tokens, 0)
                            END,
                            0
                        )
                       + MAX(COALESCE("usage".cache_read_input_tokens, 0), 0)
                END,
                0
            ),
            0
        ) AS canonical_total_tokens
    FROM "usage"
    LEFT JOIN usage_settlement_snapshots AS settlement
      ON settlement.request_id = "usage".request_id
    WHERE "usage".status NOT IN ('pending', 'streaming')
)
UPDATE api_keys AS target
SET total_tokens = COALESCE((
    SELECT SUM(canonical_usage.canonical_total_tokens)
    FROM canonical_usage
    WHERE canonical_usage.api_key_id = target.id
), 0);

WITH canonical_usage AS (
    SELECT
        "usage".provider_api_key_id,
        MAX(
            COALESCE(
                CASE
                    WHEN settlement.billing_effective_input_tokens IS NOT NULL THEN
                        MAX(settlement.billing_effective_input_tokens, 0)
                        + MAX(COALESCE(settlement.billing_output_tokens, "usage".output_tokens, 0), 0)
                        + MAX(
                            COALESCE(
                                settlement.billing_cache_creation_tokens,
                                CASE
                                    WHEN settlement.billing_cache_creation_5m_tokens IS NOT NULL
                                      OR settlement.billing_cache_creation_1h_tokens IS NOT NULL
                                    THEN COALESCE(settlement.billing_cache_creation_5m_tokens, 0)
                                       + COALESCE(settlement.billing_cache_creation_1h_tokens, 0)
                                END,
                                CASE
                                    WHEN COALESCE("usage".cache_creation_input_tokens, 0) = 0
                                      AND (
                                        MAX(
                                            COALESCE("usage".cache_creation_input_tokens_5m, 0),
                                            COALESCE("usage".cache_creation_ephemeral_5m_input_tokens, 0)
                                        )
                                        + MAX(
                                            COALESCE("usage".cache_creation_input_tokens_1h, 0),
                                            COALESCE("usage".cache_creation_ephemeral_1h_input_tokens, 0)
                                        )
                                      ) > 0
                                    THEN MAX(
                                            COALESCE("usage".cache_creation_input_tokens_5m, 0),
                                            COALESCE("usage".cache_creation_ephemeral_5m_input_tokens, 0)
                                         )
                                       + MAX(
                                            COALESCE("usage".cache_creation_input_tokens_1h, 0),
                                            COALESCE("usage".cache_creation_ephemeral_1h_input_tokens, 0)
                                         )
                                    ELSE COALESCE("usage".cache_creation_input_tokens, 0)
                                END,
                                0
                            ),
                            0
                        )
                        + MAX(
                            COALESCE(
                                settlement.billing_cache_read_tokens,
                                "usage".cache_read_input_tokens,
                                0
                            ),
                            0
                        )
                    WHEN settlement.billing_total_input_context IS NOT NULL THEN
                        MAX(settlement.billing_total_input_context, 0)
                        + MAX(COALESCE(settlement.billing_output_tokens, "usage".output_tokens, 0), 0)
                END,
                NULLIF(MAX(COALESCE("usage".total_tokens, 0), 0), 0),
                CASE
                    WHEN LOWER(COALESCE("usage".endpoint_api_format, "usage".api_format, ''))
                           IN ('openai', 'gemini', 'google')
                      OR LOWER(COALESCE("usage".endpoint_api_format, "usage".api_format, ''))
                           LIKE 'openai:%'
                      OR LOWER(COALESCE("usage".endpoint_api_format, "usage".api_format, ''))
                           LIKE 'gemini:%'
                      OR LOWER(COALESCE("usage".endpoint_api_format, "usage".api_format, ''))
                           LIKE 'google:%'
                    THEN MAX(COALESCE("usage".input_tokens, 0), 0)
                       + MAX(COALESCE("usage".output_tokens, 0), 0)
                    ELSE MAX(COALESCE("usage".input_tokens, 0), 0)
                       + MAX(COALESCE("usage".output_tokens, 0), 0)
                       + MAX(
                            CASE
                                WHEN COALESCE("usage".cache_creation_input_tokens, 0) = 0
                                  AND (
                                    MAX(
                                        COALESCE("usage".cache_creation_input_tokens_5m, 0),
                                        COALESCE("usage".cache_creation_ephemeral_5m_input_tokens, 0)
                                    )
                                    + MAX(
                                        COALESCE("usage".cache_creation_input_tokens_1h, 0),
                                        COALESCE("usage".cache_creation_ephemeral_1h_input_tokens, 0)
                                    )
                                  ) > 0
                                THEN MAX(
                                        COALESCE("usage".cache_creation_input_tokens_5m, 0),
                                        COALESCE("usage".cache_creation_ephemeral_5m_input_tokens, 0)
                                     )
                                   + MAX(
                                        COALESCE("usage".cache_creation_input_tokens_1h, 0),
                                        COALESCE("usage".cache_creation_ephemeral_1h_input_tokens, 0)
                                     )
                                ELSE COALESCE("usage".cache_creation_input_tokens, 0)
                            END,
                            0
                        )
                       + MAX(COALESCE("usage".cache_read_input_tokens, 0), 0)
                END,
                0
            ),
            0
        ) AS canonical_total_tokens
    FROM "usage"
    LEFT JOIN usage_settlement_snapshots AS settlement
      ON settlement.request_id = "usage".request_id
    WHERE "usage".status NOT IN ('pending', 'streaming')
)
UPDATE provider_api_keys AS target
SET total_tokens = COALESCE((
    SELECT SUM(canonical_usage.canonical_total_tokens)
    FROM canonical_usage
    WHERE canonical_usage.provider_api_key_id = target.id
), 0);

UPDATE api_keys AS target
LEFT JOIN (
    SELECT
        source.api_key_id,
        COALESCE(SUM(source.canonical_total_tokens), 0) AS total_tokens
    FROM (
        SELECT
            `usage`.api_key_id,
            GREATEST(
                COALESCE(
                    CASE
                        WHEN settlement.billing_effective_input_tokens IS NOT NULL THEN
                            GREATEST(settlement.billing_effective_input_tokens, 0)
                            + GREATEST(COALESCE(settlement.billing_output_tokens, `usage`.output_tokens, 0), 0)
                            + GREATEST(
                                COALESCE(
                                    settlement.billing_cache_creation_tokens,
                                    CASE
                                        WHEN settlement.billing_cache_creation_5m_tokens IS NOT NULL
                                          OR settlement.billing_cache_creation_1h_tokens IS NOT NULL
                                        THEN COALESCE(settlement.billing_cache_creation_5m_tokens, 0)
                                           + COALESCE(settlement.billing_cache_creation_1h_tokens, 0)
                                    END,
                                    CASE
                                        WHEN COALESCE(`usage`.cache_creation_input_tokens, 0) = 0
                                          AND (
                                            GREATEST(
                                                COALESCE(`usage`.cache_creation_input_tokens_5m, 0),
                                                COALESCE(`usage`.cache_creation_ephemeral_5m_input_tokens, 0)
                                            )
                                            + GREATEST(
                                                COALESCE(`usage`.cache_creation_input_tokens_1h, 0),
                                                COALESCE(`usage`.cache_creation_ephemeral_1h_input_tokens, 0)
                                            )
                                          ) > 0
                                        THEN GREATEST(
                                                COALESCE(`usage`.cache_creation_input_tokens_5m, 0),
                                                COALESCE(`usage`.cache_creation_ephemeral_5m_input_tokens, 0)
                                             )
                                           + GREATEST(
                                                COALESCE(`usage`.cache_creation_input_tokens_1h, 0),
                                                COALESCE(`usage`.cache_creation_ephemeral_1h_input_tokens, 0)
                                             )
                                        ELSE COALESCE(`usage`.cache_creation_input_tokens, 0)
                                    END,
                                    0
                                ),
                                0
                            )
                            + GREATEST(
                                COALESCE(
                                    settlement.billing_cache_read_tokens,
                                    `usage`.cache_read_input_tokens,
                                    0
                                ),
                                0
                            )
                        WHEN settlement.billing_total_input_context IS NOT NULL THEN
                            GREATEST(settlement.billing_total_input_context, 0)
                            + GREATEST(COALESCE(settlement.billing_output_tokens, `usage`.output_tokens, 0), 0)
                    END,
                    NULLIF(GREATEST(COALESCE(`usage`.total_tokens, 0), 0), 0),
                    CASE
                        WHEN SUBSTRING_INDEX(
                            LOWER(COALESCE(`usage`.endpoint_api_format, `usage`.api_format, '')),
                            ':',
                            1
                        ) IN ('openai', 'gemini', 'google')
                        THEN GREATEST(COALESCE(`usage`.input_tokens, 0), 0)
                           + GREATEST(COALESCE(`usage`.output_tokens, 0), 0)
                        ELSE GREATEST(COALESCE(`usage`.input_tokens, 0), 0)
                           + GREATEST(COALESCE(`usage`.output_tokens, 0), 0)
                           + GREATEST(
                                CASE
                                    WHEN COALESCE(`usage`.cache_creation_input_tokens, 0) = 0
                                      AND (
                                        GREATEST(
                                            COALESCE(`usage`.cache_creation_input_tokens_5m, 0),
                                            COALESCE(`usage`.cache_creation_ephemeral_5m_input_tokens, 0)
                                        )
                                        + GREATEST(
                                            COALESCE(`usage`.cache_creation_input_tokens_1h, 0),
                                            COALESCE(`usage`.cache_creation_ephemeral_1h_input_tokens, 0)
                                        )
                                      ) > 0
                                    THEN GREATEST(
                                            COALESCE(`usage`.cache_creation_input_tokens_5m, 0),
                                            COALESCE(`usage`.cache_creation_ephemeral_5m_input_tokens, 0)
                                         )
                                       + GREATEST(
                                            COALESCE(`usage`.cache_creation_input_tokens_1h, 0),
                                            COALESCE(`usage`.cache_creation_ephemeral_1h_input_tokens, 0)
                                         )
                                    ELSE COALESCE(`usage`.cache_creation_input_tokens, 0)
                                END,
                                0
                            )
                           + GREATEST(COALESCE(`usage`.cache_read_input_tokens, 0), 0)
                    END,
                    0
                ),
                0
            ) AS canonical_total_tokens
        FROM `usage`
        LEFT JOIN usage_settlement_snapshots AS settlement
          ON settlement.request_id = `usage`.request_id
        WHERE `usage`.status NOT IN ('pending', 'streaming')
    ) AS source
    WHERE source.api_key_id IS NOT NULL
      AND TRIM(source.api_key_id) <> ''
    GROUP BY source.api_key_id
) AS aggregated
  ON aggregated.api_key_id = target.id
SET target.total_tokens = COALESCE(aggregated.total_tokens, 0);

UPDATE provider_api_keys AS target
LEFT JOIN (
    SELECT
        source.provider_api_key_id,
        COALESCE(SUM(source.canonical_total_tokens), 0) AS total_tokens
    FROM (
        SELECT
            `usage`.provider_api_key_id,
            GREATEST(
                COALESCE(
                    CASE
                        WHEN settlement.billing_effective_input_tokens IS NOT NULL THEN
                            GREATEST(settlement.billing_effective_input_tokens, 0)
                            + GREATEST(COALESCE(settlement.billing_output_tokens, `usage`.output_tokens, 0), 0)
                            + GREATEST(
                                COALESCE(
                                    settlement.billing_cache_creation_tokens,
                                    CASE
                                        WHEN settlement.billing_cache_creation_5m_tokens IS NOT NULL
                                          OR settlement.billing_cache_creation_1h_tokens IS NOT NULL
                                        THEN COALESCE(settlement.billing_cache_creation_5m_tokens, 0)
                                           + COALESCE(settlement.billing_cache_creation_1h_tokens, 0)
                                    END,
                                    CASE
                                        WHEN COALESCE(`usage`.cache_creation_input_tokens, 0) = 0
                                          AND (
                                            GREATEST(
                                                COALESCE(`usage`.cache_creation_input_tokens_5m, 0),
                                                COALESCE(`usage`.cache_creation_ephemeral_5m_input_tokens, 0)
                                            )
                                            + GREATEST(
                                                COALESCE(`usage`.cache_creation_input_tokens_1h, 0),
                                                COALESCE(`usage`.cache_creation_ephemeral_1h_input_tokens, 0)
                                            )
                                          ) > 0
                                        THEN GREATEST(
                                                COALESCE(`usage`.cache_creation_input_tokens_5m, 0),
                                                COALESCE(`usage`.cache_creation_ephemeral_5m_input_tokens, 0)
                                             )
                                           + GREATEST(
                                                COALESCE(`usage`.cache_creation_input_tokens_1h, 0),
                                                COALESCE(`usage`.cache_creation_ephemeral_1h_input_tokens, 0)
                                             )
                                        ELSE COALESCE(`usage`.cache_creation_input_tokens, 0)
                                    END,
                                    0
                                ),
                                0
                            )
                            + GREATEST(
                                COALESCE(
                                    settlement.billing_cache_read_tokens,
                                    `usage`.cache_read_input_tokens,
                                    0
                                ),
                                0
                            )
                        WHEN settlement.billing_total_input_context IS NOT NULL THEN
                            GREATEST(settlement.billing_total_input_context, 0)
                            + GREATEST(COALESCE(settlement.billing_output_tokens, `usage`.output_tokens, 0), 0)
                    END,
                    NULLIF(GREATEST(COALESCE(`usage`.total_tokens, 0), 0), 0),
                    CASE
                        WHEN SUBSTRING_INDEX(
                            LOWER(COALESCE(`usage`.endpoint_api_format, `usage`.api_format, '')),
                            ':',
                            1
                        ) IN ('openai', 'gemini', 'google')
                        THEN GREATEST(COALESCE(`usage`.input_tokens, 0), 0)
                           + GREATEST(COALESCE(`usage`.output_tokens, 0), 0)
                        ELSE GREATEST(COALESCE(`usage`.input_tokens, 0), 0)
                           + GREATEST(COALESCE(`usage`.output_tokens, 0), 0)
                           + GREATEST(
                                CASE
                                    WHEN COALESCE(`usage`.cache_creation_input_tokens, 0) = 0
                                      AND (
                                        GREATEST(
                                            COALESCE(`usage`.cache_creation_input_tokens_5m, 0),
                                            COALESCE(`usage`.cache_creation_ephemeral_5m_input_tokens, 0)
                                        )
                                        + GREATEST(
                                            COALESCE(`usage`.cache_creation_input_tokens_1h, 0),
                                            COALESCE(`usage`.cache_creation_ephemeral_1h_input_tokens, 0)
                                        )
                                      ) > 0
                                    THEN GREATEST(
                                            COALESCE(`usage`.cache_creation_input_tokens_5m, 0),
                                            COALESCE(`usage`.cache_creation_ephemeral_5m_input_tokens, 0)
                                         )
                                       + GREATEST(
                                            COALESCE(`usage`.cache_creation_input_tokens_1h, 0),
                                            COALESCE(`usage`.cache_creation_ephemeral_1h_input_tokens, 0)
                                         )
                                    ELSE COALESCE(`usage`.cache_creation_input_tokens, 0)
                                END,
                                0
                            )
                           + GREATEST(COALESCE(`usage`.cache_read_input_tokens, 0), 0)
                    END,
                    0
                ),
                0
            ) AS canonical_total_tokens
        FROM `usage`
        LEFT JOIN usage_settlement_snapshots AS settlement
          ON settlement.request_id = `usage`.request_id
        WHERE `usage`.status NOT IN ('pending', 'streaming')
    ) AS source
    WHERE source.provider_api_key_id IS NOT NULL
      AND TRIM(source.provider_api_key_id) <> ''
    GROUP BY source.provider_api_key_id
) AS aggregated
  ON aggregated.provider_api_key_id = target.id
SET target.total_tokens = COALESCE(aggregated.total_tokens, 0);

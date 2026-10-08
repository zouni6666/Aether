-- Replace only the read model; existing facts, queues and projections are untouched.
-- Preserve billing/attribution semantics while avoiding repeated JSON parsing.
CREATE OR REPLACE VIEW public.usage_analytics_facts_v1 AS
SELECT u.request_id, COALESCE(u.id, u.request_id) AS id, u.created_at,
  CASE WHEN identity.owner_id IS NOT NULL AND identity.is_standalone=false THEN identity.owner_id END AS actor_user_id,
  identity.owner_id AS credential_owner_id,
  CASE WHEN identity.owner_id IS NULL THEN 'unknown' WHEN identity.is_standalone THEN 'standalone'
    WHEN NOT identity.is_standalone THEN 'employee' ELSE 'unknown' END AS attribution_kind,
  CASE WHEN identity.owner_id IS NULL THEN 'unknown' WHEN identity.is_standalone THEN 'standalone_key'
    WHEN NOT identity.is_standalone THEN 'user_account' ELSE 'unknown' END AS attribution_source,
  COALESCE(a.record_kind, 'request') AS record_kind, a.parent_request_id,
  u.api_key_id, u.model, u.target_model, u.provider_id, u.provider_name,
  u.api_format, u.endpoint_kind, u.request_type, u.is_stream, u.has_format_conversion,
  u.status, u.status_code, u.error_category, u.failure_origin, u.failure_stage, u.failure_reason,
  u.failure_schema_version, u.response_time_ms, u.first_byte_time_ms,
  COALESCE(s.billing_status, u.billing_status) AS settlement_status,
  COALESCE(metadata.value->'usage_available', 'true'::jsonb) <> 'false'::jsonb AS usage_available,
  COALESCE(metadata.value->'usage_pricing_available', 'true'::jsonb) <> 'false'::jsonb
    AND (s.billing_total_cost_usd IS NOT NULL OR COALESCE(s.billing_status, u.billing_status) = 'settled') AS pricing_available,
  CASE WHEN COALESCE(metadata.value->'usage_available', 'true'::jsonb) <> 'false'::jsonb
    THEN b.input_tokens END AS input_tokens,
  CASE WHEN COALESCE(metadata.value->'usage_available', 'true'::jsonb) <> 'false'::jsonb
    THEN b.output_tokens END AS output_tokens,
  CASE WHEN COALESCE(metadata.value->'usage_available', 'true'::jsonb) <> 'false'::jsonb
    THEN b.total_tokens END AS total_tokens,
  CASE WHEN COALESCE(metadata.value->'usage_available', 'true'::jsonb) <> 'false'::jsonb
    THEN b.cache_read_input_tokens END AS cache_read_input_tokens,
  CASE WHEN COALESCE(metadata.value->'usage_available', 'true'::jsonb) <> 'false'::jsonb
    THEN b.cache_creation_input_tokens END AS cache_creation_input_tokens,
  CASE WHEN COALESCE(metadata.value->'usage_pricing_available', 'true'::jsonb) <> 'false'::jsonb
    AND (s.billing_total_cost_usd IS NOT NULL OR COALESCE(s.billing_status, u.billing_status) = 'settled')
    THEN round(COALESCE(s.billing_total_cost_usd::numeric, u.total_cost_usd::numeric), 8) END AS rated_amount,
  CASE WHEN COALESCE(metadata.value->'usage_pricing_available', 'true'::jsonb) <> 'false'::jsonb
    AND (s.billing_actual_total_cost_usd IS NOT NULL OR COALESCE(s.billing_status, u.billing_status) = 'settled')
    THEN round(COALESCE(s.billing_actual_total_cost_usd::numeric, u.actual_total_cost_usd::numeric), 8) END AS billable_amount,
  s.quota_covered_amount_usd AS quota_covered_amount,
  s.wallet_consumed_amount_usd AS wallet_consumed_amount,
  s.wallet_debit_amount_usd AS wallet_debit_amount,
  s.wallet_recharge_debit_usd AS wallet_recharge_debit_amount,
  s.wallet_gift_debit_usd AS wallet_gift_debit_amount,
  s.wallet_overdraft_usd AS wallet_overdraft_amount,
  s.allocation_status, s.finalized_at AS settled_at,
  CASE WHEN s.billing_total_cost_usd IS NOT NULL THEN 'settlement_snapshot' ELSE 'legacy_float' END AS amount_source,
  b.upstream_is_stream,
  CASE WHEN metadata.value #>> '{analytics_measurement,source}' IN ('reported','estimated','mixed')
    THEN metadata.value #>> '{analytics_measurement,source}' ELSE 'unknown' END AS token_source,
  CASE WHEN COALESCE(metadata.value->'usage_available','true'::jsonb) <> 'false'::jsonb
    AND COALESCE(metadata.value->'usage_pricing_available','true'::jsonb) <> 'false'::jsonb
    AND s.input_price_per_1m IS NOT NULL AND s.billing_cache_read_cost_usd IS NOT NULL
    THEN round(s.input_price_per_1m::numeric * b.cache_read_input_tokens::numeric / 1000000,8) END AS cache_estimated_full_cost_amount,
  CASE WHEN COALESCE(metadata.value->'usage_available','true'::jsonb) <> 'false'::jsonb
    AND COALESCE(metadata.value->'usage_pricing_available','true'::jsonb) <> 'false'::jsonb
    AND s.input_price_per_1m IS NOT NULL AND s.billing_cache_read_cost_usd IS NOT NULL
    THEN round(s.billing_cache_read_cost_usd::numeric,8) END AS cache_read_cost_amount,
  CASE WHEN COALESCE(metadata.value->'usage_available','true'::jsonb) <> 'false'::jsonb
    AND COALESCE(metadata.value->'usage_pricing_available','true'::jsonb) <> 'false'::jsonb
    AND s.input_price_per_1m IS NOT NULL AND s.billing_cache_creation_cost_usd IS NOT NULL
    THEN round(s.billing_cache_creation_cost_usd::numeric,8) END AS cache_creation_cost_amount
FROM public.usage u
-- OFFSET 0 keeps this projection from being flattened: large metadata is
-- detoasted and parsed once per request, rather than once per metric expression.
CROSS JOIN LATERAL (SELECT u.request_metadata::jsonb AS value OFFSET 0) metadata
LEFT JOIN public.usage_settlement_snapshots s USING (request_id)
LEFT JOIN public.usage_attribution_snapshots a USING (request_id)
JOIN public.usage_billing_facts b USING (request_id)
LEFT JOIN public.api_keys k ON k.id=u.api_key_id
CROSS JOIN LATERAL (
  SELECT CASE WHEN a.request_id IS NOT NULL THEN a.credential_owner_id
    WHEN EXISTS (SELECT 1 FROM public.users WHERE id=u.user_id AND NOT is_deleted) THEN u.user_id END AS owner_id,
    COALESCE(k.is_standalone,
      CASE WHEN jsonb_typeof(metadata.value #> '{analytics_attribution,is_standalone}')='boolean'
        THEN (metadata.value #>> '{analytics_attribution,is_standalone}')::boolean END,
      CASE WHEN jsonb_typeof(metadata.value->'api_key_is_standalone')='boolean'
        THEN (metadata.value->>'api_key_is_standalone')::boolean END,
      CASE WHEN a.attribution_source='user_account' THEN false
        WHEN a.attribution_source='standalone_key' THEN true END,
      CASE WHEN u.api_key_id IS NULL THEN false END) AS is_standalone
) identity;

-- Customer charges use the immutable request-time factor snapshot. Provider
-- procurement cost remains in actual_total_cost_usd for legacy reporting.
CREATE OR REPLACE FUNCTION public.usage_customer_billable_amount(
  metadata jsonb, base_cost numeric, legacy_cost numeric
) RETURNS numeric LANGUAGE plpgsql IMMUTABLE PARALLEL SAFE AS $$
DECLARE factor jsonb; multiplier numeric; amount numeric;
  factor_name text; factor_value jsonb; factor_number double precision;
  expected_multiplier double precision := 1.0; factor_count integer := 0;
  has_zero boolean := false;
BEGIN
  IF metadata ? 'billing_multiplier_snapshot' THEN
    IF jsonb_typeof(metadata->'billing_multiplier_snapshot') <> 'object'
      OR metadata #> '{billing_multiplier_snapshot,version}' IS DISTINCT FROM '1'::jsonb
      OR jsonb_typeof(metadata #> '{billing_multiplier_snapshot,factors}') IS DISTINCT FROM 'object'
    THEN RETURN NULL; END IF;
    factor := metadata #> '{billing_multiplier_snapshot,multiplier}';
    FOR factor_name, factor_value IN
      SELECT key, value FROM jsonb_each(metadata #> '{billing_multiplier_snapshot,factors}') ORDER BY key COLLATE "C"
    LOOP
      factor_count := factor_count + 1;
      IF factor_count > 16 OR factor_name = '' OR length(factor_name) > 64
        OR factor_name !~ '^[A-Za-z0-9_]+$'
        OR jsonb_typeof(factor_value) IS DISTINCT FROM 'number'
      THEN RETURN NULL; END IF;
      factor_number := factor_value::text::double precision;
      IF factor_number < 0 OR factor_number > 1.7976931348623157e308::double precision
      THEN RETURN NULL; END IF;
      has_zero := has_zero OR factor_number = 0;
    END LOOP;
    -- Rust short-circuits zero before multiplying any of the other factors.
    IF has_zero THEN expected_multiplier := 0;
    ELSE
      FOR factor_name, factor_value IN
        SELECT key, value FROM jsonb_each(metadata #> '{billing_multiplier_snapshot,factors}') ORDER BY key COLLATE "C"
      LOOP
        factor_number := factor_value::text::double precision;
        BEGIN
          expected_multiplier := expected_multiplier * factor_number;
        EXCEPTION WHEN numeric_value_out_of_range THEN
          -- PostgreSQL raises on float underflow; Rust rounds that product to 0.
          IF expected_multiplier::numeric * factor_number::numeric > 1.7976931348623157e308::numeric
          THEN RETURN NULL; END IF;
          expected_multiplier := 0;
        END;
      END LOOP;
    END IF;
  ELSIF metadata ? 'routing_group_billing_multiplier' THEN
    factor := metadata->'routing_group_billing_multiplier';
    expected_multiplier := NULL;
  ELSE
    RETURN CASE WHEN legacy_cost NOT IN ('NaN'::numeric,'Infinity'::numeric,'-Infinity'::numeric)
      THEN round(legacy_cost,8) END;
  END IF;
  IF jsonb_typeof(factor) IS DISTINCT FROM 'number' THEN RETURN NULL; END IF;
  multiplier := factor::text::numeric;
  factor_number := factor::text::double precision;
  IF factor_number < 0
    OR factor_number > 1.7976931348623157e308::double precision
    OR (expected_multiplier IS NOT NULL AND factor_number <> expected_multiplier)
    OR multiplier < 0 OR multiplier > 1.7976931348623157e308::numeric
    OR base_cost IS NULL OR base_cost < 0
    OR base_cost IN ('NaN'::numeric,'Infinity'::numeric,'-Infinity'::numeric)
  THEN RETURN NULL; END IF;
  amount := base_cost * multiplier;
  IF amount > 1.7976931348623157e308::numeric THEN RETURN NULL; END IF;
  RETURN round(amount,8);
EXCEPTION WHEN numeric_value_out_of_range OR invalid_text_representation THEN
  -- Corrupt captured pricing must not abort an entire analytics query.
  RETURN NULL;
END $$;

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
    THEN public.usage_customer_billable_amount(metadata.value,
      COALESCE(s.billing_total_cost_usd::numeric, u.total_cost_usd::numeric),
      COALESCE(s.billing_actual_total_cost_usd::numeric, u.actual_total_cost_usd::numeric)) END AS billable_amount,
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

-- Do not backfill existing rows or scan historical usage during the upgrade.
-- Historical daily totals retain their legacy charge through the read fallback;
-- normal daily aggregation writes billing_cost for newly aggregated days.
ALTER TABLE public.stats_daily ADD COLUMN IF NOT EXISTS billing_cost numeric(20,8);

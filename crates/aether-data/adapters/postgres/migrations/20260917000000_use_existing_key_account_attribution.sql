-- Retain existing migration checksums and historical records; only future writes create v2 buckets.
DROP VIEW IF EXISTS public.usage_analytics_facts_v1;
ALTER TABLE public.api_keys DROP COLUMN IF EXISTS credential_kind;
ALTER TABLE public.usage_attribution_snapshots DROP COLUMN IF EXISTS credential_kind;

CREATE OR REPLACE FUNCTION public.overview_mark_usage_bucket() RETURNS trigger
LANGUAGE plpgsql AS $$
DECLARE old_time timestamptz; new_time timestamptz; bucket record;
BEGIN
  IF TG_TABLE_NAME = 'usage' THEN
    IF TG_OP <> 'INSERT' THEN old_time := OLD.created_at; END IF;
    IF TG_OP <> 'DELETE' THEN new_time := NEW.created_at; END IF;
  ELSE
    IF TG_OP <> 'INSERT' THEN
      SELECT created_at INTO old_time FROM public.usage WHERE request_id = OLD.request_id;
    END IF;
    IF TG_OP <> 'DELETE' THEN
      SELECT created_at INTO new_time FROM public.usage WHERE request_id = NEW.request_id;
    END IF;
  END IF;
  FOR bucket IN
    SELECT DISTINCT g, date_trunc(g, t AT TIME ZONE 'UTC') AT TIME ZONE 'UTC' AS starts
    FROM unnest(ARRAY[old_time, new_time]) t CROSS JOIN unnest(ARRAY['day','hour']) g
    WHERE t IS NOT NULL ORDER BY g, starts
  LOOP
    INSERT INTO public.stats_bucket_state(projection_version, granularity, bucket_start, source_revision)
    VALUES ('overview-v2', bucket.g, bucket.starts, 1)
    ON CONFLICT (projection_version, granularity, bucket_start)
    DO UPDATE SET source_revision = stats_bucket_state.source_revision + 1;
    IF TG_TABLE_NAME = 'usage' AND TG_OP = 'DELETE' THEN
      UPDATE public.stats_bucket_state SET coverage_status='unrecoverable',
        last_error='retained usage facts were deleted'
      WHERE projection_version='overview-v2' AND granularity=bucket.g AND bucket_start=bucket.starts;
    END IF;
  END LOOP;
  RETURN NULL;
END $$;

CREATE OR REPLACE FUNCTION public.overview_capture_usage_identity() RETURNS trigger
LANGUAGE plpgsql AS $$
DECLARE actor text; owner text; kind text; source text; standalone boolean;
BEGIN
  SELECT id INTO owner FROM public.users WHERE id=NEW.user_id AND NOT is_deleted;
  SELECT COALESCE(k.is_standalone,
    CASE WHEN jsonb_typeof(NEW.request_metadata::jsonb #> '{analytics_attribution,is_standalone}')='boolean'
      THEN (NEW.request_metadata #>> '{analytics_attribution,is_standalone}')::boolean END,
    CASE WHEN jsonb_typeof(NEW.request_metadata::jsonb->'api_key_is_standalone')='boolean'
      THEN (NEW.request_metadata->>'api_key_is_standalone')::boolean END,
    CASE WHEN a.attribution_source='user_account' THEN false
      WHEN a.attribution_source='standalone_key' THEN true END,
    CASE WHEN NEW.api_key_id IS NULL THEN false END)
  INTO standalone FROM (SELECT 1) seed
  LEFT JOIN public.api_keys k ON k.id=NEW.api_key_id
  LEFT JOIN public.usage_attribution_snapshots a ON a.request_id=NEW.request_id;
  kind := CASE WHEN owner IS NULL THEN 'unknown' WHEN standalone THEN 'standalone'
    WHEN NOT standalone THEN 'employee' ELSE 'unknown' END;
  actor := CASE WHEN kind='employee' THEN owner END;
  source := CASE kind WHEN 'employee' THEN 'user_account' WHEN 'standalone' THEN 'standalone_key' ELSE 'unknown' END;
  INSERT INTO public.usage_attribution_snapshots(request_id, actor_user_id, credential_owner_id,
    attribution_kind, attribution_source, record_kind, parent_request_id, attribution_revision)
  VALUES (NEW.request_id, actor, owner, kind, source,
    COALESCE(NEW.request_metadata #>> '{analytics_attribution,record_kind}', 'request'),
    NEW.request_metadata #>> '{analytics_attribution,parent_request_id}', 2)
  ON CONFLICT (request_id) DO UPDATE SET actor_user_id = EXCLUDED.actor_user_id,
    credential_owner_id = EXCLUDED.credential_owner_id, attribution_kind = EXCLUDED.attribution_kind,
    attribution_source = EXCLUDED.attribution_source,
    record_kind = COALESCE(NEW.request_metadata #>> '{analytics_attribution,record_kind}', usage_attribution_snapshots.record_kind),
    parent_request_id = COALESCE(EXCLUDED.parent_request_id, usage_attribution_snapshots.parent_request_id),
    attribution_revision = EXCLUDED.attribution_revision, recorded_at = NOW()
  WHERE usage_attribution_snapshots.attribution_revision <= EXCLUDED.attribution_revision;
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
  COALESCE(u.request_metadata::jsonb->'usage_available', 'true'::jsonb) <> 'false'::jsonb AS usage_available,
  COALESCE(u.request_metadata::jsonb->'usage_pricing_available', 'true'::jsonb) <> 'false'::jsonb
    AND (s.billing_total_cost_usd IS NOT NULL OR COALESCE(s.billing_status, u.billing_status) = 'settled') AS pricing_available,
  CASE WHEN COALESCE(u.request_metadata::jsonb->'usage_available', 'true'::jsonb) <> 'false'::jsonb
    THEN b.input_tokens END AS input_tokens,
  CASE WHEN COALESCE(u.request_metadata::jsonb->'usage_available', 'true'::jsonb) <> 'false'::jsonb
    THEN b.output_tokens END AS output_tokens,
  CASE WHEN COALESCE(u.request_metadata::jsonb->'usage_available', 'true'::jsonb) <> 'false'::jsonb
    THEN b.total_tokens END AS total_tokens,
  CASE WHEN COALESCE(u.request_metadata::jsonb->'usage_available', 'true'::jsonb) <> 'false'::jsonb
    THEN b.cache_read_input_tokens END AS cache_read_input_tokens,
  CASE WHEN COALESCE(u.request_metadata::jsonb->'usage_available', 'true'::jsonb) <> 'false'::jsonb
    THEN b.cache_creation_input_tokens END AS cache_creation_input_tokens,
  CASE WHEN COALESCE(u.request_metadata::jsonb->'usage_pricing_available', 'true'::jsonb) <> 'false'::jsonb
    AND (s.billing_total_cost_usd IS NOT NULL OR COALESCE(s.billing_status, u.billing_status) = 'settled')
    THEN round(COALESCE(s.billing_total_cost_usd::numeric, u.total_cost_usd::numeric), 8) END AS rated_amount,
  CASE WHEN COALESCE(u.request_metadata::jsonb->'usage_pricing_available', 'true'::jsonb) <> 'false'::jsonb
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
  CASE WHEN u.request_metadata #>> '{analytics_measurement,source}' IN ('reported','estimated','mixed')
    THEN u.request_metadata #>> '{analytics_measurement,source}' ELSE 'unknown' END AS token_source,
  CASE WHEN COALESCE(u.request_metadata::jsonb->'usage_available','true'::jsonb) <> 'false'::jsonb
    AND COALESCE(u.request_metadata::jsonb->'usage_pricing_available','true'::jsonb) <> 'false'::jsonb
    AND s.input_price_per_1m IS NOT NULL AND s.billing_cache_read_cost_usd IS NOT NULL
    THEN round(s.input_price_per_1m::numeric * b.cache_read_input_tokens::numeric / 1000000,8) END AS cache_estimated_full_cost_amount,
  CASE WHEN COALESCE(u.request_metadata::jsonb->'usage_available','true'::jsonb) <> 'false'::jsonb
    AND COALESCE(u.request_metadata::jsonb->'usage_pricing_available','true'::jsonb) <> 'false'::jsonb
    AND s.input_price_per_1m IS NOT NULL AND s.billing_cache_read_cost_usd IS NOT NULL
    THEN round(s.billing_cache_read_cost_usd::numeric,8) END AS cache_read_cost_amount,
  CASE WHEN COALESCE(u.request_metadata::jsonb->'usage_available','true'::jsonb) <> 'false'::jsonb
    AND COALESCE(u.request_metadata::jsonb->'usage_pricing_available','true'::jsonb) <> 'false'::jsonb
    AND s.input_price_per_1m IS NOT NULL AND s.billing_cache_creation_cost_usd IS NOT NULL
    THEN round(s.billing_cache_creation_cost_usd::numeric,8) END AS cache_creation_cost_amount
FROM public.usage u
LEFT JOIN public.usage_settlement_snapshots s USING (request_id)
LEFT JOIN public.usage_attribution_snapshots a USING (request_id)
JOIN public.usage_billing_facts b USING (request_id)
LEFT JOIN public.api_keys k ON k.id=u.api_key_id
CROSS JOIN LATERAL (
  SELECT CASE WHEN a.request_id IS NOT NULL THEN a.credential_owner_id
    WHEN EXISTS (SELECT 1 FROM public.users WHERE id=u.user_id AND NOT is_deleted) THEN u.user_id END AS owner_id,
    COALESCE(k.is_standalone,
      CASE WHEN jsonb_typeof(u.request_metadata::jsonb #> '{analytics_attribution,is_standalone}')='boolean'
        THEN (u.request_metadata #>> '{analytics_attribution,is_standalone}')::boolean END,
      CASE WHEN jsonb_typeof(u.request_metadata::jsonb->'api_key_is_standalone')='boolean'
        THEN (u.request_metadata->>'api_key_is_standalone')::boolean END,
      CASE WHEN a.attribution_source='user_account' THEN false
        WHEN a.attribution_source='standalone_key' THEN true END,
      CASE WHEN u.api_key_id IS NULL THEN false END) AS is_standalone
) identity;

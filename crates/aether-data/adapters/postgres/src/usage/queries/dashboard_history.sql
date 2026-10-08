WITH daily AS (
  SELECT CASE WHEN pg_typeof(date) IN ('bigint'::regtype,'integer'::regtype)
    THEN to_timestamp(date::text::double precision)
    ELSE date::text::timestamptz END AS day,
    total_requests AS request_count, input_tokens, output_tokens,
    CASE WHEN effective_input_tokens=0 AND total_input_context=0 AND input_tokens>0
      THEN input_tokens ELSE effective_input_tokens END
      + output_tokens + cache_creation_tokens + cache_read_tokens AS total_tokens,
    cache_creation_tokens, cache_read_tokens,
    CASE WHEN total_input_context=0 THEN
      CASE WHEN effective_input_tokens=0 AND input_tokens>0 THEN input_tokens
        ELSE effective_input_tokens END + cache_creation_tokens + cache_read_tokens
      ELSE total_input_context END AS cache_input_tokens,
    COALESCE(billing_cost,actual_total_cost::numeric) AS billable_amount
  FROM stats_daily
), facts AS MATERIALIZED (
  SELECT (day AT TIME ZONE 'UTC')::date AS day, request_count,
    input_tokens, output_tokens, total_tokens, cache_creation_tokens,
    cache_read_tokens, cache_input_tokens, billable_amount
  FROM daily WHERE day<$1
  UNION ALL
  SELECT (b.created_at AT TIME ZONE 'UTC')::date, 1::bigint,
    b.input_tokens, b.output_tokens, b.total_tokens, b.cache_creation_input_tokens,
    b.cache_read_input_tokens, b.total_input_context,
    CASE WHEN COALESCE(u.request_metadata::jsonb->'usage_pricing_available','true'::jsonb)<>'false'::jsonb
      AND (s.billing_actual_total_cost_usd IS NOT NULL OR COALESCE(s.billing_status,u.billing_status)='settled')
      THEN public.usage_customer_billable_amount(u.request_metadata::jsonb,
        COALESCE(s.billing_total_cost_usd::numeric,u.total_cost_usd::numeric),
        COALESCE(s.billing_actual_total_cost_usd::numeric,u.actual_total_cost_usd::numeric)) END
  FROM usage_billing_facts b
  JOIN usage u USING (request_id)
  LEFT JOIN usage_settlement_snapshots s USING (request_id)
  WHERE b.created_at >= $1 AND b.created_at < $2
    AND u.created_at >= $1 AND u.created_at < $2
    AND b.status NOT IN ('pending','streaming')
    AND b.provider_name NOT IN ('unknown','pending')
), activity AS (
  SELECT day, sum(request_count)::bigint AS requests FROM facts
  GROUP BY day HAVING sum(request_count)>0
)
SELECT jsonb_build_object(
  'request_count', COALESCE(sum(request_count),0)::bigint,
  'input_tokens', COALESCE(sum(input_tokens),0)::bigint,
  'output_tokens', COALESCE(sum(output_tokens),0)::bigint,
  'total_tokens', COALESCE(sum(total_tokens),0)::bigint,
  'cache_creation_tokens', COALESCE(sum(cache_creation_tokens),0)::bigint,
  'cache_read_tokens', COALESCE(sum(cache_read_tokens),0)::bigint,
  'cache_input_tokens', COALESCE(sum(cache_input_tokens),0)::bigint,
  'usage_available_count', COALESCE(sum(request_count),0)::bigint,
  'pricing_available_count', COALESCE(sum(request_count) FILTER (WHERE billable_amount IS NOT NULL),0)::bigint,
  'billable_amount', round(COALESCE(sum(billable_amount),0),8)::text
) AS metrics,
  (SELECT COALESCE(jsonb_agg(jsonb_build_object('date',day::text,'requests',requests)
    ORDER BY day),'[]'::jsonb) FROM activity) AS days
FROM facts

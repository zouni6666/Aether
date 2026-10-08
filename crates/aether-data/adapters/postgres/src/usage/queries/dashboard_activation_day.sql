WITH missing AS MATERIALIZED (
  SELECT f.actor_user_id, f.input_tokens, f.output_tokens, f.total_tokens,
    f.usage_available, f.billable_amount, f.cache_read_input_tokens,
    f.cache_creation_input_tokens, f.first_byte_time_ms, f.response_time_ms,
    f.upstream_is_stream, b.total_input_context
  FROM usage_analytics_facts_v1 f
  JOIN usage_billing_facts b ON b.request_id=f.request_id
    -- Bound both view inputs so the planner cannot scan historical billing rows.
    AND b.created_at >= $1 AND b.created_at < $2
  WHERE f.created_at >= $1 AND f.created_at < $2 AND f.record_kind <> 'session'
), metrics AS (
  SELECT jsonb_build_object(
    'request_count', count(*),
    'input_tokens', COALESCE(sum(input_tokens),0),
    'output_tokens', COALESCE(sum(output_tokens),0),
    'total_tokens', COALESCE(sum(total_tokens),0),
    'usage_available_count', count(*) FILTER (WHERE usage_available),
    'pricing_available_count', count(billable_amount),
    'billable_amount', COALESCE(sum(billable_amount),0),
    'cache_read_tokens', COALESCE(sum(cache_read_input_tokens),0),
    'cache_creation_tokens', COALESCE(sum(cache_creation_input_tokens),0),
    'cache_input_tokens', COALESCE(sum(total_input_context) FILTER (WHERE usage_available),0),
    'first_byte_sum_ms', COALESCE(sum(first_byte_time_ms),0),
    'first_byte_sample_count', count(first_byte_time_ms),
    'response_sum_ms', COALESCE(sum(response_time_ms),0),
    'response_sample_count', count(response_time_ms),
    'stream_requests', count(*) FILTER (WHERE COALESCE(upstream_is_stream,false)),
    'standard_requests', count(*) FILTER (WHERE NOT COALESCE(upstream_is_stream,false))
  ) AS value FROM missing
), actors AS (
  SELECT actor_user_id FROM missing WHERE actor_user_id IS NOT NULL
  UNION
  SELECT actor_user_id FROM dashboard_actor_minute
  WHERE bucket_start >= date_trunc('minute',$2::timestamptz)
    AND bucket_start <= $3 AND request_count>0
), combined AS (
  SELECT dashboard_metrics_add($4::jsonb, value) AS value FROM metrics
)
SELECT value || jsonb_build_object(
  'billable_amount', round((value->>'billable_amount')::numeric,8)::text,
  'active_users', (SELECT count(*) FROM actors a JOIN users u
    ON u.id=a.actor_user_id AND NOT u.is_deleted)
) AS metrics FROM combined

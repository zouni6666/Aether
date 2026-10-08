-- no-transaction
-- Keep the existing billing cover while building its replacement. Overview
-- totals also read settlement/allocation status; omitting those columns forces
-- a scan of the wide snapshot heap even when every monetary field is covered.
-- The following migration retires the old index only after this build succeeds.
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_usage_settlement_dashboard_cover_v2
ON public.usage_settlement_snapshots (request_id)
INCLUDE (
  billing_input_tokens,
  billing_effective_input_tokens,
  billing_output_tokens,
  billing_cache_creation_tokens,
  billing_cache_creation_5m_tokens,
  billing_cache_creation_1h_tokens,
  billing_cache_read_tokens,
  billing_total_input_context,
  billing_cache_creation_cost_usd,
  billing_cache_read_cost_usd,
  billing_total_cost_usd,
  billing_actual_total_cost_usd,
  input_price_per_1m,
  billing_status,
  allocation_status
);

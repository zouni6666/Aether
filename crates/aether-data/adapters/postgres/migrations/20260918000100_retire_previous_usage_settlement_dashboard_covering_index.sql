-- no-transaction
-- The preceding migration has published the superset index. Do not keep two
-- permanent covering indexes with the same key and almost identical payloads.
DROP INDEX CONCURRENTLY IF EXISTS public.idx_usage_settlement_dashboard_cover;

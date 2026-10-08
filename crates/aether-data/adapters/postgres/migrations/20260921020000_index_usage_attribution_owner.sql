-- no-transaction
-- Anonymization looks up either side of the account attribution. Keep both
-- branches indexed so deleting an unrelated user never scans all snapshots.
CREATE INDEX CONCURRENTLY IF NOT EXISTS ix_usage_attribution_owner_request
  ON public.usage_attribution_snapshots (credential_owner_id, request_id);

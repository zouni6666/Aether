-- no-transaction
-- Historical trusted-identity metadata must also be anonymized. Index that
-- exact predicate without storing the much larger metadata document.
CREATE INDEX CONCURRENTLY IF NOT EXISTS ix_usage_analytics_actor_metadata
  ON public.usage ((request_metadata #>> '{analytics_attribution,actor_user_id}'))
  WHERE (request_metadata #>> '{analytics_attribution,actor_user_id}') IS NOT NULL;

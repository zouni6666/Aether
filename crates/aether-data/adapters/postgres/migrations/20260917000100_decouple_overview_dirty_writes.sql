-- Upgrade databases that already applied the original account-attribution migration.
-- Keep this before concurrent index builds so their failure cannot leave writers
-- contending on shared hourly/daily aggregate rows. Fresh databases are safe too.
-- Only future fact mutations enqueue work; no historical rows are backfilled.
-- Each writer owns its transaction's keys, so unrelated requests never contend
-- on the current hour/day's stats_bucket_state row.
CREATE TABLE IF NOT EXISTS public.stats_overview_dirty_events (
  transaction_id bigint NOT NULL,
  projection_version text NOT NULL,
  granularity text NOT NULL,
  bucket_start timestamptz NOT NULL,
  unrecoverable boolean NOT NULL DEFAULT false,
  PRIMARY KEY (transaction_id, projection_version, granularity, bucket_start)
);
CREATE INDEX IF NOT EXISTS ix_stats_overview_dirty_events_bucket
  ON public.stats_overview_dirty_events(projection_version, granularity, bucket_start);

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
    INSERT INTO public.stats_overview_dirty_events
      (transaction_id, projection_version, granularity, bucket_start, unrecoverable)
    VALUES (txid_current(), 'overview-v2', bucket.g, bucket.starts,
      TG_TABLE_NAME = 'usage' AND TG_OP = 'DELETE')
    ON CONFLICT (transaction_id, projection_version, granularity, bucket_start)
    DO UPDATE SET unrecoverable = stats_overview_dirty_events.unrecoverable OR EXCLUDED.unrecoverable;
  END LOOP;
  RETURN NULL;
END $$;

-- No raw usage history is scanned. Existing dashboard minutes are compacted by
-- bounded background batches; lifetime totals and local-calendar activity survive.
-- Fresh installations already contain these definitions in the dashboard baseline.
ALTER TABLE public.dashboard_stats_state ADD COLUMN IF NOT EXISTS contributions_cleanup_cursor varchar(100);
CREATE TABLE IF NOT EXISTS public.dashboard_activity_minute (
  bucket_start timestamptz NOT NULL,
  shard smallint NOT NULL CHECK (shard >= 0 AND shard < 16),
  request_count bigint NOT NULL,
  PRIMARY KEY (bucket_start,shard)
);

-- Callers hold the aggregate shard lock. Before the first post-upgrade correction
-- of a minute, seed its narrow counter from the existing wide aggregate.
CREATE OR REPLACE FUNCTION public.dashboard_ensure_activity_minute(p_bucket timestamptz,p_shard smallint)
RETURNS void LANGUAGE sql AS $$
  INSERT INTO public.dashboard_activity_minute(bucket_start,shard,request_count)
    SELECT bucket_start,shard,COALESCE((metrics->>'request_count')::bigint,0)
    FROM public.dashboard_stats_minute WHERE bucket_start=p_bucket AND shard=p_shard
    ON CONFLICT(bucket_start,shard) DO NOTHING
$$;

CREATE OR REPLACE FUNCTION public.dashboard_enqueue_request() RETURNS trigger LANGUAGE plpgsql AS $$
DECLARE rid text;
BEGIN
  IF current_setting('aether.dashboard_restore',true)='on' THEN RETURN COALESCE(NEW,OLD); END IF;
  rid := CASE WHEN TG_OP = 'DELETE' THEN OLD.request_id ELSE NEW.request_id END;
  -- Fast path: an update to a pre-activation request must never scan/replay it.
  IF TG_TABLE_NAME = 'usage' THEN
    IF (CASE WHEN TG_OP = 'DELETE' THEN OLD.created_at ELSE NEW.created_at END)
       < (SELECT stats_since FROM public.dashboard_stats_state WHERE singleton) THEN
      RETURN COALESCE(NEW, OLD);
    END IF;
  END IF;
  INSERT INTO public.dashboard_stats_pending(transaction_id, request_id, deleted_fact)
    VALUES(txid_current(), rid,
      CASE WHEN TG_TABLE_NAME = 'usage' AND TG_OP = 'DELETE'
        THEN public.dashboard_request_fact(rid) END)
  ON CONFLICT (transaction_id, request_id) DO UPDATE
    SET deleted_fact = COALESCE(EXCLUDED.deleted_fact, dashboard_stats_pending.deleted_fact);
  RETURN COALESCE(NEW, OLD);
END $$;

CREATE OR REPLACE FUNCTION public.dashboard_apply_pending() RETURNS trigger LANGUAGE plpgsql AS $$
DECLARE p record; old_fact public.dashboard_request_contributions%ROWTYPE;
  fact jsonb; next_metrics jsonb; next_at timestamptz; next_actor text;
  shard_id smallint; old_bucket timestamptz; next_bucket timestamptz; purged boolean;
  detail_cutoff timestamptz := date_trunc('minute',clock_timestamp()-INTERVAL '35 days');
BEGIN
  IF current_setting('aether.dashboard_restore',true)='on' THEN
    DELETE FROM public.dashboard_stats_pending WHERE transaction_id=NEW.transaction_id;
    RETURN NULL;
  END IF;
  -- Lock all touched shards in one order before touching any ledger or minute.
  -- This also serializes concurrent usage/settlement updates for the same request.
  PERFORM t.shard FROM public.dashboard_stats_total t
    WHERE t.shard IN (SELECT (hashtextextended(request_id,0) & 15)::smallint
      FROM public.dashboard_stats_pending WHERE transaction_id=NEW.transaction_id)
    ORDER BY t.shard FOR UPDATE;
  FOR p IN SELECT * FROM public.dashboard_stats_pending
    WHERE transaction_id=NEW.transaction_id ORDER BY request_id LOOP
    shard_id := (hashtextextended(p.request_id,0) & 15)::smallint;
    fact := public.dashboard_request_fact(p.request_id);
    purged := NOT EXISTS(SELECT 1 FROM public.usage WHERE request_id=p.request_id);
    -- A purge preserves the last contribution, including an update in this tx.
    fact := COALESCE(fact,p.deleted_fact);
    IF fact IS NULL THEN
      IF purged THEN DELETE FROM public.dashboard_request_contributions WHERE request_id=p.request_id; END IF;
      CONTINUE;
    END IF;
    next_metrics := fact->'metrics';
    next_at := (fact->>'created_at')::timestamptz;
    next_actor := CASE WHEN COALESCE((next_metrics->>'request_count')::bigint,0)>0
      THEN fact->>'actor_user_id' END;
    SELECT * INTO old_fact FROM public.dashboard_request_contributions WHERE request_id=p.request_id;
    IF FOUND AND old_fact.created_at=next_at
      AND old_fact.actor_user_id IS NOT DISTINCT FROM next_actor
      AND old_fact.metrics=next_metrics THEN
      IF purged THEN DELETE FROM public.dashboard_request_contributions WHERE request_id=p.request_id; END IF;
      CONTINUE;
    END IF;
    next_bucket := date_trunc('minute',next_at AT TIME ZONE 'UTC') AT TIME ZONE 'UTC';
    IF old_fact.request_id IS NOT NULL THEN
      old_bucket := date_trunc('minute',old_fact.created_at AT TIME ZONE 'UTC') AT TIME ZONE 'UTC';
      PERFORM public.dashboard_ensure_activity_minute(old_bucket,shard_id);
      UPDATE public.dashboard_activity_minute SET request_count=request_count-COALESCE((old_fact.metrics->>'request_count')::bigint,0)
        WHERE bucket_start=old_bucket AND shard=shard_id;
      UPDATE public.dashboard_stats_total SET metrics=public.dashboard_metrics_add(metrics,old_fact.metrics,-1) WHERE shard=shard_id;
      IF old_bucket>=detail_cutoff THEN
        UPDATE public.dashboard_stats_minute SET metrics=public.dashboard_metrics_add(metrics,old_fact.metrics,-1)
          WHERE bucket_start=old_bucket AND shard=shard_id;
      END IF;
      UPDATE public.dashboard_activity_hour SET request_count=request_count-COALESCE((old_fact.metrics->>'request_count')::bigint,0)
        WHERE bucket_start=date_trunc('hour',old_fact.created_at AT TIME ZONE 'UTC') AT TIME ZONE 'UTC' AND shard=shard_id;
      IF old_fact.actor_user_id IS NOT NULL AND old_bucket>=detail_cutoff THEN
        UPDATE public.dashboard_actor_minute SET request_count=request_count-1
          WHERE bucket_start=old_bucket AND shard=shard_id AND actor_user_id=old_fact.actor_user_id;
      END IF;
    END IF;
    PERFORM public.dashboard_ensure_activity_minute(next_bucket,shard_id);
    INSERT INTO public.dashboard_activity_minute(bucket_start,shard,request_count)
      VALUES(next_bucket,shard_id,COALESCE((next_metrics->>'request_count')::bigint,0))
      ON CONFLICT(bucket_start,shard) DO UPDATE
        SET request_count=dashboard_activity_minute.request_count+EXCLUDED.request_count;
    UPDATE public.dashboard_stats_total SET metrics=public.dashboard_metrics_add(metrics,next_metrics) WHERE shard=shard_id;
    IF next_bucket>=detail_cutoff THEN
      INSERT INTO public.dashboard_stats_minute(bucket_start,shard,metrics) VALUES(next_bucket,shard_id,next_metrics)
        ON CONFLICT(bucket_start,shard) DO UPDATE
        SET metrics=public.dashboard_metrics_add(dashboard_stats_minute.metrics,EXCLUDED.metrics);
    END IF;
    INSERT INTO public.dashboard_activity_hour(bucket_start,shard,request_count)
      VALUES(date_trunc('hour',next_at AT TIME ZONE 'UTC') AT TIME ZONE 'UTC',shard_id,COALESCE((next_metrics->>'request_count')::bigint,0))
      ON CONFLICT(bucket_start,shard) DO UPDATE SET request_count=dashboard_activity_hour.request_count+EXCLUDED.request_count;
    IF next_actor IS NOT NULL AND next_bucket>=detail_cutoff THEN
      INSERT INTO public.dashboard_actor_minute(bucket_start,shard,actor_user_id,request_count)
        VALUES(next_bucket,shard_id,next_actor,1)
        ON CONFLICT(bucket_start,shard,actor_user_id) DO UPDATE SET request_count=dashboard_actor_minute.request_count+1;
    END IF;
    IF purged THEN
      DELETE FROM public.dashboard_request_contributions WHERE request_id=p.request_id;
    ELSE
      INSERT INTO public.dashboard_request_contributions(request_id,created_at,actor_user_id,metrics)
      VALUES(p.request_id,next_at,next_actor,next_metrics)
      ON CONFLICT(request_id) DO UPDATE SET created_at=EXCLUDED.created_at,
        actor_user_id=EXCLUDED.actor_user_id,metrics=EXCLUDED.metrics;
    END IF;
  END LOOP;
  DELETE FROM public.dashboard_stats_pending WHERE transaction_id=NEW.transaction_id;
  RETURN NULL;
END $$;
CREATE OR REPLACE FUNCTION public.dashboard_record_user_event() RETURNS trigger LANGUAGE plpgsql AS $$
DECLARE added bigint := 0; removed bigint := 0; rid text; at timestamptz := clock_timestamp();
BEGIN
  IF current_setting('aether.dashboard_restore',true)='on' THEN RETURN COALESCE(NEW,OLD); END IF;
  IF TG_OP='INSERT' THEN added:=1; rid:=NEW.id;
  ELSIF TG_OP='DELETE' THEN removed:=(NOT OLD.is_deleted)::integer; rid:=OLD.id;
  ELSE removed:=(NOT OLD.is_deleted AND NEW.is_deleted)::integer; rid:=NEW.id;
  END IF;
  IF added=0 AND removed=0 THEN RETURN COALESCE(NEW,OLD); END IF;
  IF NOT EXISTS(SELECT 1 FROM public.dashboard_stats_state WHERE singleton AND stats_since<=at) THEN RETURN COALESCE(NEW,OLD); END IF;
  INSERT INTO public.dashboard_user_events_minute(bucket_start,shard,created_count,deleted_count)
    VALUES(date_trunc('minute',at AT TIME ZONE 'UTC') AT TIME ZONE 'UTC',
      (hashtextextended(rid,0) & 15)::smallint,added,removed)
    ON CONFLICT(bucket_start,shard) DO UPDATE SET
      created_count=dashboard_user_events_minute.created_count+EXCLUDED.created_count,
      deleted_count=dashboard_user_events_minute.deleted_count+EXCLUDED.deleted_count;
  RETURN COALESCE(NEW,OLD);
END $$;

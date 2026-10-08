-- Install future user anonymization without scanning or rewriting historical rows.
-- Legacy orphan actors are excluded by the dashboard read path. User deletion
-- cleans the retained actor window and contribution identities at transaction end.
CREATE TABLE public.dashboard_user_anonymization_pending (
  transaction_id bigint NOT NULL,
  user_id varchar(255) NOT NULL,
  PRIMARY KEY (transaction_id,user_id)
);

CREATE FUNCTION public.dashboard_enqueue_user_anonymization() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
  IF current_setting('aether.dashboard_restore',true)='on' THEN RETURN NULL; END IF;
  IF TG_OP='DELETE' OR NEW.is_deleted THEN
    INSERT INTO public.dashboard_user_anonymization_pending(transaction_id,user_id)
      VALUES(txid_current(),OLD.id) ON CONFLICT DO NOTHING;
  END IF;
  RETURN NULL;
END $$;
CREATE TRIGGER dashboard_user_anonymize AFTER DELETE OR UPDATE OF is_deleted ON public.users
  FOR EACH ROW EXECUTE FUNCTION public.dashboard_enqueue_user_anonymization();

CREATE OR REPLACE FUNCTION public.dashboard_apply_pending() RETURNS trigger LANGUAGE plpgsql AS $$
DECLARE p record; anonymous_request record; old_fact public.dashboard_request_contributions%ROWTYPE;
  fact jsonb; next_metrics jsonb; next_at timestamptz; next_actor text;
  shard_id smallint; old_bucket timestamptz; next_bucket timestamptz; purged boolean; anonymize boolean;
  detail_cutoff timestamptz := date_trunc('minute',clock_timestamp()-INTERVAL '35 days');
BEGIN
  IF current_setting('aether.dashboard_restore',true)='on' THEN
    DELETE FROM public.dashboard_stats_pending WHERE transaction_id=NEW.transaction_id;
    DELETE FROM public.dashboard_user_anonymization_pending WHERE transaction_id=NEW.transaction_id;
    RETURN NULL;
  END IF;
  SELECT EXISTS (SELECT 1 FROM public.dashboard_user_anonymization_pending
    WHERE transaction_id=NEW.transaction_id) INTO anonymize;
  -- Lock all touched shards in one order before touching any ledger or minute.
  -- This also serializes concurrent usage/settlement updates for the same request.
  PERFORM t.shard FROM public.dashboard_stats_total t
    WHERE t.shard IN (SELECT (hashtextextended(request_id,0) & 15)::smallint
      FROM public.dashboard_stats_pending WHERE transaction_id=NEW.transaction_id)
      OR anonymize
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
      THEN (SELECT id FROM public.users WHERE id=fact->>'actor_user_id' AND NOT is_deleted) END;
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
  -- Flush identity removal only after request corrections, under the same shard
  -- locks. This also covers requests whose source usage was already purged.
  -- Surviving contribution identities were corrected above by request_id; do
  -- not scan the lifetime contribution table while holding the shard locks.
  IF anonymize THEN
    -- A writer can commit between the user's deletion statement and this flush.
    -- Its new attribution was invisible to the user's earlier trigger. Reuse
    -- the actor index to find only those requests, then clear each ledger by PK.
    -- Do not lock attribution rows here: writers lock them before their shard.
    FOR anonymous_request IN
      SELECT a.request_id, pending_user.user_id
      FROM public.dashboard_user_anonymization_pending pending_user
      JOIN public.usage_attribution_snapshots a ON a.actor_user_id=pending_user.user_id
      WHERE pending_user.transaction_id=NEW.transaction_id
    LOOP
      UPDATE public.dashboard_request_contributions SET actor_user_id=NULL
        WHERE request_id=anonymous_request.request_id AND actor_user_id=anonymous_request.user_id;
    END LOOP;
    DELETE FROM public.dashboard_actor_minute a USING public.dashboard_user_anonymization_pending pending_user
      WHERE pending_user.transaction_id=NEW.transaction_id AND a.actor_user_id=pending_user.user_id;
  END IF;
  DELETE FROM public.dashboard_stats_pending WHERE transaction_id=NEW.transaction_id;
  DELETE FROM public.dashboard_user_anonymization_pending WHERE transaction_id=NEW.transaction_id;
  RETURN NULL;
END $$;
-- Both request and user events share one deferred flush. It locks every touched
-- shard in order before corrections or deletion, regardless of trigger order.
CREATE CONSTRAINT TRIGGER dashboard_flush_user_anonymization
  AFTER INSERT ON public.dashboard_user_anonymization_pending
  DEFERRABLE INITIALLY DEFERRED FOR EACH ROW EXECUTE FUNCTION public.dashboard_apply_pending();

-- Future-only dashboard projection. No usage history is read or backfilled.
-- Background maintenance bounds detailed minutes to 35 days; lifetime totals
-- and narrow activity counters remain available after detail or usage cleanup.
CREATE TABLE public.dashboard_stats_state (
  singleton boolean PRIMARY KEY DEFAULT true CHECK (singleton),
  stats_since timestamptz NOT NULL,
  contributions_cleanup_cursor varchar(100)
);
CREATE TABLE public.dashboard_request_contributions (
  request_id varchar(100) PRIMARY KEY,
  created_at timestamptz NOT NULL,
  actor_user_id varchar(255),
  metrics jsonb NOT NULL
);
CREATE TABLE public.dashboard_stats_total (
  shard smallint PRIMARY KEY CHECK (shard >= 0 AND shard < 16),
  metrics jsonb NOT NULL DEFAULT '{}'::jsonb
);
INSERT INTO public.dashboard_stats_total(shard) SELECT generate_series(0,15);
CREATE TABLE public.dashboard_stats_minute (
  bucket_start timestamptz NOT NULL,
  shard smallint NOT NULL CHECK (shard >= 0 AND shard < 16),
  metrics jsonb NOT NULL DEFAULT '{}'::jsonb,
  PRIMARY KEY (bucket_start, shard)
);
CREATE TABLE public.dashboard_activity_minute (
  bucket_start timestamptz NOT NULL,
  shard smallint NOT NULL CHECK (shard >= 0 AND shard < 16),
  request_count bigint NOT NULL,
  PRIMARY KEY (bucket_start,shard)
);
CREATE TABLE public.dashboard_activity_hour (
  bucket_start timestamptz NOT NULL,
  shard smallint NOT NULL CHECK (shard >= 0 AND shard < 16),
  request_count bigint NOT NULL,
  PRIMARY KEY (bucket_start, shard)
);
CREATE TABLE public.dashboard_actor_minute (
  bucket_start timestamptz NOT NULL,
  shard smallint NOT NULL CHECK (shard >= 0 AND shard < 16),
  actor_user_id varchar(255) NOT NULL,
  request_count bigint NOT NULL,
  PRIMARY KEY (bucket_start, shard, actor_user_id)
);
-- Rows only exist within a writing transaction. Deferred processing coalesces
-- usage, settlement and identity mutations into one final contribution.
CREATE TABLE public.dashboard_stats_pending (
  transaction_id bigint NOT NULL,
  request_id varchar(100) NOT NULL,
  deleted_fact jsonb,
  PRIMARY KEY (transaction_id, request_id)
);
CREATE TABLE public.dashboard_user_events_minute (
  bucket_start timestamptz NOT NULL,
  shard smallint NOT NULL CHECK (shard >= 0 AND shard < 16),
  created_count bigint NOT NULL DEFAULT 0,
  deleted_count bigint NOT NULL DEFAULT 0,
  PRIMARY KEY (bucket_start, shard)
);

CREATE FUNCTION public.dashboard_metrics_add(a jsonb, b jsonb, direction integer DEFAULT 1)
RETURNS jsonb LANGUAGE sql IMMUTABLE PARALLEL SAFE AS $$
  SELECT COALESCE(jsonb_object_agg(key, value), '{}'::jsonb) FROM (
    SELECT key, sum(value) AS value FROM (
      SELECT key, value::numeric FROM jsonb_each_text(COALESCE(a, '{}'::jsonb))
      UNION ALL
      SELECT key, value::numeric * direction FROM jsonb_each_text(COALESCE(b, '{}'::jsonb))
    ) entries GROUP BY key
  ) sums
$$;

-- Canonical views are evaluated for ONE indexed request, never for old history.
CREATE FUNCTION public.dashboard_request_fact(p_request_id text)
RETURNS jsonb LANGUAGE sql STABLE AS $$
  SELECT jsonb_build_object('created_at', f.created_at,
    'actor_user_id', f.actor_user_id,
    'metrics', CASE WHEN f.record_kind = 'session' THEN '{}'::jsonb ELSE
      jsonb_build_object(
        'request_count', 1,
        'input_tokens', COALESCE(f.input_tokens,0),
        'output_tokens', COALESCE(f.output_tokens,0),
        'total_tokens', COALESCE(f.total_tokens,0),
        'usage_available_count', f.usage_available::integer,
        'pricing_available_count', (f.billable_amount IS NOT NULL)::integer,
        'billable_amount', COALESCE(f.billable_amount,0),
        'cache_read_tokens', COALESCE(f.cache_read_input_tokens,0),
        'cache_creation_tokens', COALESCE(f.cache_creation_input_tokens,0),
        'cache_input_tokens', CASE WHEN f.usage_available THEN COALESCE(b.total_input_context,0) ELSE 0 END,
        'first_byte_sum_ms', COALESCE(f.first_byte_time_ms,0),
        'first_byte_sample_count', (f.first_byte_time_ms IS NOT NULL)::integer,
        'response_sum_ms', COALESCE(f.response_time_ms,0),
        'response_sample_count', (f.response_time_ms IS NOT NULL)::integer,
        'stream_requests', COALESCE(f.upstream_is_stream,false)::integer,
        'standard_requests', (NOT COALESCE(f.upstream_is_stream,false))::integer
      ) END)
  FROM public.usage_analytics_facts_v1 f
  JOIN public.usage_billing_facts b USING (request_id)
  WHERE f.request_id = p_request_id
    AND f.created_at >= (SELECT stats_since FROM public.dashboard_stats_state WHERE singleton)
$$;

-- Callers hold the aggregate shard lock. Seed a missing narrow activity counter
-- from the detailed minute before applying a correction.
CREATE FUNCTION public.dashboard_ensure_activity_minute(p_bucket timestamptz,p_shard smallint)
RETURNS void LANGUAGE sql AS $$
  INSERT INTO public.dashboard_activity_minute(bucket_start,shard,request_count)
    SELECT bucket_start,shard,COALESCE((metrics->>'request_count')::bigint,0)
    FROM public.dashboard_stats_minute WHERE bucket_start=p_bucket AND shard=p_shard
    ON CONFLICT(bucket_start,shard) DO NOTHING
$$;

CREATE FUNCTION public.dashboard_enqueue_request() RETURNS trigger LANGUAGE plpgsql AS $$
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

CREATE FUNCTION public.dashboard_apply_pending() RETURNS trigger LANGUAGE plpgsql AS $$
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
CREATE CONSTRAINT TRIGGER dashboard_flush_pending AFTER INSERT ON public.dashboard_stats_pending
  DEFERRABLE INITIALLY DEFERRED FOR EACH ROW EXECUTE FUNCTION public.dashboard_apply_pending();

CREATE FUNCTION public.dashboard_record_user_event() RETURNS trigger LANGUAGE plpgsql AS $$
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

-- Acquire source DDL locks before recording activation. This excludes transactions
-- that could commit writes without observing the new triggers.
CREATE TRIGGER dashboard_usage_changed AFTER INSERT OR UPDATE ON public.usage
  FOR EACH ROW EXECUTE FUNCTION public.dashboard_enqueue_request();
CREATE TRIGGER aaa_dashboard_usage_deleted BEFORE DELETE ON public.usage
  FOR EACH ROW EXECUTE FUNCTION public.dashboard_enqueue_request();
CREATE TRIGGER dashboard_settlement_changed AFTER INSERT OR UPDATE OR DELETE ON public.usage_settlement_snapshots
  FOR EACH ROW EXECUTE FUNCTION public.dashboard_enqueue_request();
CREATE TRIGGER dashboard_attribution_changed AFTER INSERT OR UPDATE ON public.usage_attribution_snapshots
  FOR EACH ROW EXECUTE FUNCTION public.dashboard_enqueue_request();
CREATE TRIGGER dashboard_user_changed AFTER INSERT OR DELETE OR UPDATE OF is_deleted ON public.users
  FOR EACH ROW EXECUTE FUNCTION public.dashboard_record_user_event();
INSERT INTO public.dashboard_stats_state(singleton,stats_since) VALUES(true,clock_timestamp());

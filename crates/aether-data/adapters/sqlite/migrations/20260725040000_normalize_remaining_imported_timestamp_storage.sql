-- Complete the legacy import repair for every INTEGER-affinity timestamp column
-- that existed before the 20260725 parity migrations. Invalid values are left
-- untouched so the guard at the end fails the migration instead of coercing data.

UPDATE "announcement_reads"
SET "read_at" = CASE
        WHEN typeof("read_at") = 'text' AND strftime('%s', trim("read_at")) IS NOT NULL
        THEN CAST(strftime('%s', trim("read_at")) AS INTEGER)
        ELSE "read_at"
    END
WHERE (typeof("read_at") = 'text' AND strftime('%s', trim("read_at")) IS NOT NULL);

UPDATE "audit_logs"
SET "created_at" = CASE
        WHEN typeof("created_at") = 'text' AND strftime('%s', trim("created_at")) IS NOT NULL
        THEN CAST(strftime('%s', trim("created_at")) AS INTEGER)
        ELSE "created_at"
    END
WHERE (typeof("created_at") = 'text' AND strftime('%s', trim("created_at")) IS NOT NULL);

UPDATE "background_task_events"
SET "created_at_unix_secs" = CASE
        WHEN typeof("created_at_unix_secs") = 'text' AND strftime('%s', trim("created_at_unix_secs")) IS NOT NULL
        THEN CAST(strftime('%s', trim("created_at_unix_secs")) AS INTEGER)
        ELSE "created_at_unix_secs"
    END
WHERE (typeof("created_at_unix_secs") = 'text' AND strftime('%s', trim("created_at_unix_secs")) IS NOT NULL);

UPDATE "background_task_runs"
SET "created_at_unix_secs" = CASE
        WHEN typeof("created_at_unix_secs") = 'text' AND strftime('%s', trim("created_at_unix_secs")) IS NOT NULL
        THEN CAST(strftime('%s', trim("created_at_unix_secs")) AS INTEGER)
        ELSE "created_at_unix_secs"
    END,
    "finished_at_unix_secs" = CASE
        WHEN typeof("finished_at_unix_secs") = 'text' AND strftime('%s', trim("finished_at_unix_secs")) IS NOT NULL
        THEN CAST(strftime('%s', trim("finished_at_unix_secs")) AS INTEGER)
        ELSE "finished_at_unix_secs"
    END,
    "started_at_unix_secs" = CASE
        WHEN typeof("started_at_unix_secs") = 'text' AND strftime('%s', trim("started_at_unix_secs")) IS NOT NULL
        THEN CAST(strftime('%s', trim("started_at_unix_secs")) AS INTEGER)
        ELSE "started_at_unix_secs"
    END,
    "updated_at_unix_secs" = CASE
        WHEN typeof("updated_at_unix_secs") = 'text' AND strftime('%s', trim("updated_at_unix_secs")) IS NOT NULL
        THEN CAST(strftime('%s', trim("updated_at_unix_secs")) AS INTEGER)
        ELSE "updated_at_unix_secs"
    END
WHERE (typeof("created_at_unix_secs") = 'text' AND strftime('%s', trim("created_at_unix_secs")) IS NOT NULL)
   OR (typeof("finished_at_unix_secs") = 'text' AND strftime('%s', trim("finished_at_unix_secs")) IS NOT NULL)
   OR (typeof("started_at_unix_secs") = 'text' AND strftime('%s', trim("started_at_unix_secs")) IS NOT NULL)
   OR (typeof("updated_at_unix_secs") = 'text' AND strftime('%s', trim("updated_at_unix_secs")) IS NOT NULL);

UPDATE "entitlement_usage_ledgers"
SET "created_at" = CASE
        WHEN typeof("created_at") = 'text' AND strftime('%s', trim("created_at")) IS NOT NULL
        THEN CAST(strftime('%s', trim("created_at")) AS INTEGER)
        ELSE "created_at"
    END
WHERE (typeof("created_at") = 'text' AND strftime('%s', trim("created_at")) IS NOT NULL);

UPDATE "gemini_file_mappings"
SET "created_at" = CASE
        WHEN typeof("created_at") = 'text' AND strftime('%s', trim("created_at")) IS NOT NULL
        THEN CAST(strftime('%s', trim("created_at")) AS INTEGER)
        ELSE "created_at"
    END,
    "expires_at" = CASE
        WHEN typeof("expires_at") = 'text' AND strftime('%s', trim("expires_at")) IS NOT NULL
        THEN CAST(strftime('%s', trim("expires_at")) AS INTEGER)
        ELSE "expires_at"
    END
WHERE (typeof("created_at") = 'text' AND strftime('%s', trim("created_at")) IS NOT NULL)
   OR (typeof("expires_at") = 'text' AND strftime('%s', trim("expires_at")) IS NOT NULL);

UPDATE "ldap_configs"
SET "created_at" = CASE
        WHEN typeof("created_at") = 'text' AND strftime('%s', trim("created_at")) IS NOT NULL
        THEN CAST(strftime('%s', trim("created_at")) AS INTEGER)
        ELSE "created_at"
    END,
    "updated_at" = CASE
        WHEN typeof("updated_at") = 'text' AND strftime('%s', trim("updated_at")) IS NOT NULL
        THEN CAST(strftime('%s', trim("updated_at")) AS INTEGER)
        ELSE "updated_at"
    END
WHERE (typeof("created_at") = 'text' AND strftime('%s', trim("created_at")) IS NOT NULL)
   OR (typeof("updated_at") = 'text' AND strftime('%s', trim("updated_at")) IS NOT NULL);

UPDATE "pool_member_scores"
SET "last_failure_at" = CASE
        WHEN typeof("last_failure_at") = 'text' AND strftime('%s', trim("last_failure_at")) IS NOT NULL
        THEN CAST(strftime('%s', trim("last_failure_at")) AS INTEGER)
        ELSE "last_failure_at"
    END,
    "last_probe_attempt_at" = CASE
        WHEN typeof("last_probe_attempt_at") = 'text' AND strftime('%s', trim("last_probe_attempt_at")) IS NOT NULL
        THEN CAST(strftime('%s', trim("last_probe_attempt_at")) AS INTEGER)
        ELSE "last_probe_attempt_at"
    END,
    "last_probe_failure_at" = CASE
        WHEN typeof("last_probe_failure_at") = 'text' AND strftime('%s', trim("last_probe_failure_at")) IS NOT NULL
        THEN CAST(strftime('%s', trim("last_probe_failure_at")) AS INTEGER)
        ELSE "last_probe_failure_at"
    END,
    "last_probe_success_at" = CASE
        WHEN typeof("last_probe_success_at") = 'text' AND strftime('%s', trim("last_probe_success_at")) IS NOT NULL
        THEN CAST(strftime('%s', trim("last_probe_success_at")) AS INTEGER)
        ELSE "last_probe_success_at"
    END,
    "last_ranked_at" = CASE
        WHEN typeof("last_ranked_at") = 'text' AND strftime('%s', trim("last_ranked_at")) IS NOT NULL
        THEN CAST(strftime('%s', trim("last_ranked_at")) AS INTEGER)
        ELSE "last_ranked_at"
    END,
    "last_scheduled_at" = CASE
        WHEN typeof("last_scheduled_at") = 'text' AND strftime('%s', trim("last_scheduled_at")) IS NOT NULL
        THEN CAST(strftime('%s', trim("last_scheduled_at")) AS INTEGER)
        ELSE "last_scheduled_at"
    END,
    "last_success_at" = CASE
        WHEN typeof("last_success_at") = 'text' AND strftime('%s', trim("last_success_at")) IS NOT NULL
        THEN CAST(strftime('%s', trim("last_success_at")) AS INTEGER)
        ELSE "last_success_at"
    END,
    "updated_at" = CASE
        WHEN typeof("updated_at") = 'text' AND strftime('%s', trim("updated_at")) IS NOT NULL
        THEN CAST(strftime('%s', trim("updated_at")) AS INTEGER)
        ELSE "updated_at"
    END
WHERE (typeof("last_failure_at") = 'text' AND strftime('%s', trim("last_failure_at")) IS NOT NULL)
   OR (typeof("last_probe_attempt_at") = 'text' AND strftime('%s', trim("last_probe_attempt_at")) IS NOT NULL)
   OR (typeof("last_probe_failure_at") = 'text' AND strftime('%s', trim("last_probe_failure_at")) IS NOT NULL)
   OR (typeof("last_probe_success_at") = 'text' AND strftime('%s', trim("last_probe_success_at")) IS NOT NULL)
   OR (typeof("last_ranked_at") = 'text' AND strftime('%s', trim("last_ranked_at")) IS NOT NULL)
   OR (typeof("last_scheduled_at") = 'text' AND strftime('%s', trim("last_scheduled_at")) IS NOT NULL)
   OR (typeof("last_success_at") = 'text' AND strftime('%s', trim("last_success_at")) IS NOT NULL)
   OR (typeof("updated_at") = 'text' AND strftime('%s', trim("updated_at")) IS NOT NULL);

UPDATE "proxy_node_events"
SET "created_at" = CASE
        WHEN typeof("created_at") = 'text' AND strftime('%s', trim("created_at")) IS NOT NULL
        THEN CAST(strftime('%s', trim("created_at")) AS INTEGER)
        ELSE "created_at"
    END
WHERE (typeof("created_at") = 'text' AND strftime('%s', trim("created_at")) IS NOT NULL);

UPDATE "proxy_node_metrics_1h"
SET "bucket_start_unix_secs" = CASE
        WHEN typeof("bucket_start_unix_secs") = 'text' AND strftime('%s', trim("bucket_start_unix_secs")) IS NOT NULL
        THEN CAST(strftime('%s', trim("bucket_start_unix_secs")) AS INTEGER)
        ELSE "bucket_start_unix_secs"
    END
WHERE (typeof("bucket_start_unix_secs") = 'text' AND strftime('%s', trim("bucket_start_unix_secs")) IS NOT NULL);

UPDATE "proxy_node_metrics_1m"
SET "bucket_start_unix_secs" = CASE
        WHEN typeof("bucket_start_unix_secs") = 'text' AND strftime('%s', trim("bucket_start_unix_secs")) IS NOT NULL
        THEN CAST(strftime('%s', trim("bucket_start_unix_secs")) AS INTEGER)
        ELSE "bucket_start_unix_secs"
    END
WHERE (typeof("bucket_start_unix_secs") = 'text' AND strftime('%s', trim("bucket_start_unix_secs")) IS NOT NULL);

UPDATE "referral_rewards"
SET "created_at" = CASE
        WHEN typeof("created_at") = 'text' AND strftime('%s', trim("created_at")) IS NOT NULL
        THEN CAST(strftime('%s', trim("created_at")) AS INTEGER)
        ELSE "created_at"
    END,
    "updated_at" = CASE
        WHEN typeof("updated_at") = 'text' AND strftime('%s', trim("updated_at")) IS NOT NULL
        THEN CAST(strftime('%s', trim("updated_at")) AS INTEGER)
        ELSE "updated_at"
    END
WHERE (typeof("created_at") = 'text' AND strftime('%s', trim("created_at")) IS NOT NULL)
   OR (typeof("updated_at") = 'text' AND strftime('%s', trim("updated_at")) IS NOT NULL);

UPDATE "request_candidates"
SET "created_at" = CASE
        WHEN typeof("created_at") = 'text' AND strftime('%s', trim("created_at")) IS NOT NULL
        THEN CAST(strftime('%s', trim("created_at")) AS INTEGER)
        ELSE "created_at"
    END,
    "finished_at" = CASE
        WHEN typeof("finished_at") = 'text' AND strftime('%s', trim("finished_at")) IS NOT NULL
        THEN CAST(strftime('%s', trim("finished_at")) AS INTEGER)
        ELSE "finished_at"
    END,
    "started_at" = CASE
        WHEN typeof("started_at") = 'text' AND strftime('%s', trim("started_at")) IS NOT NULL
        THEN CAST(strftime('%s', trim("started_at")) AS INTEGER)
        ELSE "started_at"
    END
WHERE (typeof("created_at") = 'text' AND strftime('%s', trim("created_at")) IS NOT NULL)
   OR (typeof("finished_at") = 'text' AND strftime('%s', trim("finished_at")) IS NOT NULL)
   OR (typeof("started_at") = 'text' AND strftime('%s', trim("started_at")) IS NOT NULL);

UPDATE "routing_group_bindings"
SET "created_at" = CASE
        WHEN typeof("created_at") = 'text' AND strftime('%s', trim("created_at")) IS NOT NULL
        THEN CAST(strftime('%s', trim("created_at")) AS INTEGER)
        ELSE "created_at"
    END,
    "updated_at" = CASE
        WHEN typeof("updated_at") = 'text' AND strftime('%s', trim("updated_at")) IS NOT NULL
        THEN CAST(strftime('%s', trim("updated_at")) AS INTEGER)
        ELSE "updated_at"
    END
WHERE (typeof("created_at") = 'text' AND strftime('%s', trim("created_at")) IS NOT NULL)
   OR (typeof("updated_at") = 'text' AND strftime('%s', trim("updated_at")) IS NOT NULL);

UPDATE "routing_group_versions"
SET "created_at" = CASE
        WHEN typeof("created_at") = 'text' AND strftime('%s', trim("created_at")) IS NOT NULL
        THEN CAST(strftime('%s', trim("created_at")) AS INTEGER)
        ELSE "created_at"
    END
WHERE (typeof("created_at") = 'text' AND strftime('%s', trim("created_at")) IS NOT NULL);

UPDATE "routing_groups"
SET "created_at" = CASE
        WHEN typeof("created_at") = 'text' AND strftime('%s', trim("created_at")) IS NOT NULL
        THEN CAST(strftime('%s', trim("created_at")) AS INTEGER)
        ELSE "created_at"
    END,
    "published_at" = CASE
        WHEN typeof("published_at") = 'text' AND strftime('%s', trim("published_at")) IS NOT NULL
        THEN CAST(strftime('%s', trim("published_at")) AS INTEGER)
        ELSE "published_at"
    END,
    "updated_at" = CASE
        WHEN typeof("updated_at") = 'text' AND strftime('%s', trim("updated_at")) IS NOT NULL
        THEN CAST(strftime('%s', trim("updated_at")) AS INTEGER)
        ELSE "updated_at"
    END
WHERE (typeof("created_at") = 'text' AND strftime('%s', trim("created_at")) IS NOT NULL)
   OR (typeof("published_at") = 'text' AND strftime('%s', trim("published_at")) IS NOT NULL)
   OR (typeof("updated_at") = 'text' AND strftime('%s', trim("updated_at")) IS NOT NULL);

UPDATE "stats_daily_api_key"
SET "created_at" = CASE
        WHEN typeof("created_at") = 'text' AND strftime('%s', trim("created_at")) IS NOT NULL
        THEN CAST(strftime('%s', trim("created_at")) AS INTEGER)
        ELSE "created_at"
    END,
    "date" = CASE
        WHEN typeof("date") = 'text' AND strftime('%s', trim("date")) IS NOT NULL
        THEN CAST(strftime('%s', trim("date")) AS INTEGER)
        ELSE "date"
    END,
    "updated_at" = CASE
        WHEN typeof("updated_at") = 'text' AND strftime('%s', trim("updated_at")) IS NOT NULL
        THEN CAST(strftime('%s', trim("updated_at")) AS INTEGER)
        ELSE "updated_at"
    END
WHERE (typeof("created_at") = 'text' AND strftime('%s', trim("created_at")) IS NOT NULL)
   OR (typeof("date") = 'text' AND strftime('%s', trim("date")) IS NOT NULL)
   OR (typeof("updated_at") = 'text' AND strftime('%s', trim("updated_at")) IS NOT NULL);

UPDATE "stats_daily_error"
SET "created_at" = CASE
        WHEN typeof("created_at") = 'text' AND strftime('%s', trim("created_at")) IS NOT NULL
        THEN CAST(strftime('%s', trim("created_at")) AS INTEGER)
        ELSE "created_at"
    END,
    "date" = CASE
        WHEN typeof("date") = 'text' AND strftime('%s', trim("date")) IS NOT NULL
        THEN CAST(strftime('%s', trim("date")) AS INTEGER)
        ELSE "date"
    END,
    "updated_at" = CASE
        WHEN typeof("updated_at") = 'text' AND strftime('%s', trim("updated_at")) IS NOT NULL
        THEN CAST(strftime('%s', trim("updated_at")) AS INTEGER)
        ELSE "updated_at"
    END
WHERE (typeof("created_at") = 'text' AND strftime('%s', trim("created_at")) IS NOT NULL)
   OR (typeof("date") = 'text' AND strftime('%s', trim("date")) IS NOT NULL)
   OR (typeof("updated_at") = 'text' AND strftime('%s', trim("updated_at")) IS NOT NULL);

UPDATE "stats_daily_model"
SET "created_at" = CASE
        WHEN typeof("created_at") = 'text' AND strftime('%s', trim("created_at")) IS NOT NULL
        THEN CAST(strftime('%s', trim("created_at")) AS INTEGER)
        ELSE "created_at"
    END,
    "date" = CASE
        WHEN typeof("date") = 'text' AND strftime('%s', trim("date")) IS NOT NULL
        THEN CAST(strftime('%s', trim("date")) AS INTEGER)
        ELSE "date"
    END,
    "updated_at" = CASE
        WHEN typeof("updated_at") = 'text' AND strftime('%s', trim("updated_at")) IS NOT NULL
        THEN CAST(strftime('%s', trim("updated_at")) AS INTEGER)
        ELSE "updated_at"
    END
WHERE (typeof("created_at") = 'text' AND strftime('%s', trim("created_at")) IS NOT NULL)
   OR (typeof("date") = 'text' AND strftime('%s', trim("date")) IS NOT NULL)
   OR (typeof("updated_at") = 'text' AND strftime('%s', trim("updated_at")) IS NOT NULL);

UPDATE "stats_daily_provider"
SET "created_at" = CASE
        WHEN typeof("created_at") = 'text' AND strftime('%s', trim("created_at")) IS NOT NULL
        THEN CAST(strftime('%s', trim("created_at")) AS INTEGER)
        ELSE "created_at"
    END,
    "date" = CASE
        WHEN typeof("date") = 'text' AND strftime('%s', trim("date")) IS NOT NULL
        THEN CAST(strftime('%s', trim("date")) AS INTEGER)
        ELSE "date"
    END,
    "updated_at" = CASE
        WHEN typeof("updated_at") = 'text' AND strftime('%s', trim("updated_at")) IS NOT NULL
        THEN CAST(strftime('%s', trim("updated_at")) AS INTEGER)
        ELSE "updated_at"
    END
WHERE (typeof("created_at") = 'text' AND strftime('%s', trim("created_at")) IS NOT NULL)
   OR (typeof("date") = 'text' AND strftime('%s', trim("date")) IS NOT NULL)
   OR (typeof("updated_at") = 'text' AND strftime('%s', trim("updated_at")) IS NOT NULL);

UPDATE "stats_daily"
SET "aggregated_at" = CASE
        WHEN typeof("aggregated_at") = 'text' AND strftime('%s', trim("aggregated_at")) IS NOT NULL
        THEN CAST(strftime('%s', trim("aggregated_at")) AS INTEGER)
        ELSE "aggregated_at"
    END,
    "created_at" = CASE
        WHEN typeof("created_at") = 'text' AND strftime('%s', trim("created_at")) IS NOT NULL
        THEN CAST(strftime('%s', trim("created_at")) AS INTEGER)
        ELSE "created_at"
    END,
    "date" = CASE
        WHEN typeof("date") = 'text' AND strftime('%s', trim("date")) IS NOT NULL
        THEN CAST(strftime('%s', trim("date")) AS INTEGER)
        ELSE "date"
    END,
    "updated_at" = CASE
        WHEN typeof("updated_at") = 'text' AND strftime('%s', trim("updated_at")) IS NOT NULL
        THEN CAST(strftime('%s', trim("updated_at")) AS INTEGER)
        ELSE "updated_at"
    END
WHERE (typeof("aggregated_at") = 'text' AND strftime('%s', trim("aggregated_at")) IS NOT NULL)
   OR (typeof("created_at") = 'text' AND strftime('%s', trim("created_at")) IS NOT NULL)
   OR (typeof("date") = 'text' AND strftime('%s', trim("date")) IS NOT NULL)
   OR (typeof("updated_at") = 'text' AND strftime('%s', trim("updated_at")) IS NOT NULL);

UPDATE "stats_hourly_model"
SET "created_at" = CASE
        WHEN typeof("created_at") = 'text' AND strftime('%s', trim("created_at")) IS NOT NULL
        THEN CAST(strftime('%s', trim("created_at")) AS INTEGER)
        ELSE "created_at"
    END,
    "hour_utc" = CASE
        WHEN typeof("hour_utc") = 'text' AND strftime('%s', trim("hour_utc")) IS NOT NULL
        THEN CAST(strftime('%s', trim("hour_utc")) AS INTEGER)
        ELSE "hour_utc"
    END,
    "updated_at" = CASE
        WHEN typeof("updated_at") = 'text' AND strftime('%s', trim("updated_at")) IS NOT NULL
        THEN CAST(strftime('%s', trim("updated_at")) AS INTEGER)
        ELSE "updated_at"
    END
WHERE (typeof("created_at") = 'text' AND strftime('%s', trim("created_at")) IS NOT NULL)
   OR (typeof("hour_utc") = 'text' AND strftime('%s', trim("hour_utc")) IS NOT NULL)
   OR (typeof("updated_at") = 'text' AND strftime('%s', trim("updated_at")) IS NOT NULL);

UPDATE "stats_hourly_provider"
SET "created_at" = CASE
        WHEN typeof("created_at") = 'text' AND strftime('%s', trim("created_at")) IS NOT NULL
        THEN CAST(strftime('%s', trim("created_at")) AS INTEGER)
        ELSE "created_at"
    END,
    "hour_utc" = CASE
        WHEN typeof("hour_utc") = 'text' AND strftime('%s', trim("hour_utc")) IS NOT NULL
        THEN CAST(strftime('%s', trim("hour_utc")) AS INTEGER)
        ELSE "hour_utc"
    END,
    "updated_at" = CASE
        WHEN typeof("updated_at") = 'text' AND strftime('%s', trim("updated_at")) IS NOT NULL
        THEN CAST(strftime('%s', trim("updated_at")) AS INTEGER)
        ELSE "updated_at"
    END
WHERE (typeof("created_at") = 'text' AND strftime('%s', trim("created_at")) IS NOT NULL)
   OR (typeof("hour_utc") = 'text' AND strftime('%s', trim("hour_utc")) IS NOT NULL)
   OR (typeof("updated_at") = 'text' AND strftime('%s', trim("updated_at")) IS NOT NULL);

UPDATE "stats_hourly_user_model"
SET "created_at" = CASE
        WHEN typeof("created_at") = 'text' AND strftime('%s', trim("created_at")) IS NOT NULL
        THEN CAST(strftime('%s', trim("created_at")) AS INTEGER)
        ELSE "created_at"
    END,
    "hour_utc" = CASE
        WHEN typeof("hour_utc") = 'text' AND strftime('%s', trim("hour_utc")) IS NOT NULL
        THEN CAST(strftime('%s', trim("hour_utc")) AS INTEGER)
        ELSE "hour_utc"
    END,
    "updated_at" = CASE
        WHEN typeof("updated_at") = 'text' AND strftime('%s', trim("updated_at")) IS NOT NULL
        THEN CAST(strftime('%s', trim("updated_at")) AS INTEGER)
        ELSE "updated_at"
    END
WHERE (typeof("created_at") = 'text' AND strftime('%s', trim("created_at")) IS NOT NULL)
   OR (typeof("hour_utc") = 'text' AND strftime('%s', trim("hour_utc")) IS NOT NULL)
   OR (typeof("updated_at") = 'text' AND strftime('%s', trim("updated_at")) IS NOT NULL);

UPDATE "stats_hourly_user"
SET "created_at" = CASE
        WHEN typeof("created_at") = 'text' AND strftime('%s', trim("created_at")) IS NOT NULL
        THEN CAST(strftime('%s', trim("created_at")) AS INTEGER)
        ELSE "created_at"
    END,
    "hour_utc" = CASE
        WHEN typeof("hour_utc") = 'text' AND strftime('%s', trim("hour_utc")) IS NOT NULL
        THEN CAST(strftime('%s', trim("hour_utc")) AS INTEGER)
        ELSE "hour_utc"
    END,
    "updated_at" = CASE
        WHEN typeof("updated_at") = 'text' AND strftime('%s', trim("updated_at")) IS NOT NULL
        THEN CAST(strftime('%s', trim("updated_at")) AS INTEGER)
        ELSE "updated_at"
    END
WHERE (typeof("created_at") = 'text' AND strftime('%s', trim("created_at")) IS NOT NULL)
   OR (typeof("hour_utc") = 'text' AND strftime('%s', trim("hour_utc")) IS NOT NULL)
   OR (typeof("updated_at") = 'text' AND strftime('%s', trim("updated_at")) IS NOT NULL);

UPDATE "stats_hourly"
SET "aggregated_at" = CASE
        WHEN typeof("aggregated_at") = 'text' AND strftime('%s', trim("aggregated_at")) IS NOT NULL
        THEN CAST(strftime('%s', trim("aggregated_at")) AS INTEGER)
        ELSE "aggregated_at"
    END,
    "created_at" = CASE
        WHEN typeof("created_at") = 'text' AND strftime('%s', trim("created_at")) IS NOT NULL
        THEN CAST(strftime('%s', trim("created_at")) AS INTEGER)
        ELSE "created_at"
    END,
    "hour_utc" = CASE
        WHEN typeof("hour_utc") = 'text' AND strftime('%s', trim("hour_utc")) IS NOT NULL
        THEN CAST(strftime('%s', trim("hour_utc")) AS INTEGER)
        ELSE "hour_utc"
    END,
    "updated_at" = CASE
        WHEN typeof("updated_at") = 'text' AND strftime('%s', trim("updated_at")) IS NOT NULL
        THEN CAST(strftime('%s', trim("updated_at")) AS INTEGER)
        ELSE "updated_at"
    END
WHERE (typeof("aggregated_at") = 'text' AND strftime('%s', trim("aggregated_at")) IS NOT NULL)
   OR (typeof("created_at") = 'text' AND strftime('%s', trim("created_at")) IS NOT NULL)
   OR (typeof("hour_utc") = 'text' AND strftime('%s', trim("hour_utc")) IS NOT NULL)
   OR (typeof("updated_at") = 'text' AND strftime('%s', trim("updated_at")) IS NOT NULL);

UPDATE "stats_user_daily"
SET "created_at" = CASE
        WHEN typeof("created_at") = 'text' AND strftime('%s', trim("created_at")) IS NOT NULL
        THEN CAST(strftime('%s', trim("created_at")) AS INTEGER)
        ELSE "created_at"
    END,
    "date" = CASE
        WHEN typeof("date") = 'text' AND strftime('%s', trim("date")) IS NOT NULL
        THEN CAST(strftime('%s', trim("date")) AS INTEGER)
        ELSE "date"
    END,
    "updated_at" = CASE
        WHEN typeof("updated_at") = 'text' AND strftime('%s', trim("updated_at")) IS NOT NULL
        THEN CAST(strftime('%s', trim("updated_at")) AS INTEGER)
        ELSE "updated_at"
    END
WHERE (typeof("created_at") = 'text' AND strftime('%s', trim("created_at")) IS NOT NULL)
   OR (typeof("date") = 'text' AND strftime('%s', trim("date")) IS NOT NULL)
   OR (typeof("updated_at") = 'text' AND strftime('%s', trim("updated_at")) IS NOT NULL);

UPDATE "usage_counter_deltas"
SET "candidate_last_used_at_unix_secs" = CASE
        WHEN typeof("candidate_last_used_at_unix_secs") = 'text' AND strftime('%s', trim("candidate_last_used_at_unix_secs")) IS NOT NULL
        THEN CAST(strftime('%s', trim("candidate_last_used_at_unix_secs")) AS INTEGER)
        ELSE "candidate_last_used_at_unix_secs"
    END,
    "created_at" = CASE
        WHEN typeof("created_at") = 'text' AND strftime('%s', trim("created_at")) IS NOT NULL
        THEN CAST(strftime('%s', trim("created_at")) AS INTEGER)
        ELSE "created_at"
    END,
    "last_used_at_unix_secs" = CASE
        WHEN typeof("last_used_at_unix_secs") = 'text' AND strftime('%s', trim("last_used_at_unix_secs")) IS NOT NULL
        THEN CAST(strftime('%s', trim("last_used_at_unix_secs")) AS INTEGER)
        ELSE "last_used_at_unix_secs"
    END,
    "processed_at" = CASE
        WHEN typeof("processed_at") = 'text' AND strftime('%s', trim("processed_at")) IS NOT NULL
        THEN CAST(strftime('%s', trim("processed_at")) AS INTEGER)
        ELSE "processed_at"
    END,
    "removed_last_used_at_unix_secs" = CASE
        WHEN typeof("removed_last_used_at_unix_secs") = 'text' AND strftime('%s', trim("removed_last_used_at_unix_secs")) IS NOT NULL
        THEN CAST(strftime('%s', trim("removed_last_used_at_unix_secs")) AS INTEGER)
        ELSE "removed_last_used_at_unix_secs"
    END,
    "usage_created_at_unix_secs" = CASE
        WHEN typeof("usage_created_at_unix_secs") = 'text' AND strftime('%s', trim("usage_created_at_unix_secs")) IS NOT NULL
        THEN CAST(strftime('%s', trim("usage_created_at_unix_secs")) AS INTEGER)
        ELSE "usage_created_at_unix_secs"
    END
WHERE (typeof("candidate_last_used_at_unix_secs") = 'text' AND strftime('%s', trim("candidate_last_used_at_unix_secs")) IS NOT NULL)
   OR (typeof("created_at") = 'text' AND strftime('%s', trim("created_at")) IS NOT NULL)
   OR (typeof("last_used_at_unix_secs") = 'text' AND strftime('%s', trim("last_used_at_unix_secs")) IS NOT NULL)
   OR (typeof("processed_at") = 'text' AND strftime('%s', trim("processed_at")) IS NOT NULL)
   OR (typeof("removed_last_used_at_unix_secs") = 'text' AND strftime('%s', trim("removed_last_used_at_unix_secs")) IS NOT NULL)
   OR (typeof("usage_created_at_unix_secs") = 'text' AND strftime('%s', trim("usage_created_at_unix_secs")) IS NOT NULL);

UPDATE "user_invite_codes"
SET "created_at" = CASE
        WHEN typeof("created_at") = 'text' AND strftime('%s', trim("created_at")) IS NOT NULL
        THEN CAST(strftime('%s', trim("created_at")) AS INTEGER)
        ELSE "created_at"
    END,
    "updated_at" = CASE
        WHEN typeof("updated_at") = 'text' AND strftime('%s', trim("updated_at")) IS NOT NULL
        THEN CAST(strftime('%s', trim("updated_at")) AS INTEGER)
        ELSE "updated_at"
    END
WHERE (typeof("created_at") = 'text' AND strftime('%s', trim("created_at")) IS NOT NULL)
   OR (typeof("updated_at") = 'text' AND strftime('%s', trim("updated_at")) IS NOT NULL);

UPDATE "user_preferences"
SET "created_at" = CASE
        WHEN typeof("created_at") = 'text' AND strftime('%s', trim("created_at")) IS NOT NULL
        THEN CAST(strftime('%s', trim("created_at")) AS INTEGER)
        ELSE "created_at"
    END,
    "updated_at" = CASE
        WHEN typeof("updated_at") = 'text' AND strftime('%s', trim("updated_at")) IS NOT NULL
        THEN CAST(strftime('%s', trim("updated_at")) AS INTEGER)
        ELSE "updated_at"
    END
WHERE (typeof("created_at") = 'text' AND strftime('%s', trim("created_at")) IS NOT NULL)
   OR (typeof("updated_at") = 'text' AND strftime('%s', trim("updated_at")) IS NOT NULL);

UPDATE "user_referrals"
SET "created_at" = CASE
        WHEN typeof("created_at") = 'text' AND strftime('%s', trim("created_at")) IS NOT NULL
        THEN CAST(strftime('%s', trim("created_at")) AS INTEGER)
        ELSE "created_at"
    END,
    "first_paid_at" = CASE
        WHEN typeof("first_paid_at") = 'text' AND strftime('%s', trim("first_paid_at")) IS NOT NULL
        THEN CAST(strftime('%s', trim("first_paid_at")) AS INTEGER)
        ELSE "first_paid_at"
    END,
    "updated_at" = CASE
        WHEN typeof("updated_at") = 'text' AND strftime('%s', trim("updated_at")) IS NOT NULL
        THEN CAST(strftime('%s', trim("updated_at")) AS INTEGER)
        ELSE "updated_at"
    END
WHERE (typeof("created_at") = 'text' AND strftime('%s', trim("created_at")) IS NOT NULL)
   OR (typeof("first_paid_at") = 'text' AND strftime('%s', trim("first_paid_at")) IS NOT NULL)
   OR (typeof("updated_at") = 'text' AND strftime('%s', trim("updated_at")) IS NOT NULL);

UPDATE "user_sessions"
SET "created_at" = CASE
        WHEN typeof("created_at") = 'text' AND strftime('%s', trim("created_at")) IS NOT NULL
        THEN CAST(strftime('%s', trim("created_at")) AS INTEGER)
        ELSE "created_at"
    END,
    "expires_at" = CASE
        WHEN typeof("expires_at") = 'text' AND strftime('%s', trim("expires_at")) IS NOT NULL
        THEN CAST(strftime('%s', trim("expires_at")) AS INTEGER)
        ELSE "expires_at"
    END,
    "last_seen_at" = CASE
        WHEN typeof("last_seen_at") = 'text' AND strftime('%s', trim("last_seen_at")) IS NOT NULL
        THEN CAST(strftime('%s', trim("last_seen_at")) AS INTEGER)
        ELSE "last_seen_at"
    END,
    "revoked_at" = CASE
        WHEN typeof("revoked_at") = 'text' AND strftime('%s', trim("revoked_at")) IS NOT NULL
        THEN CAST(strftime('%s', trim("revoked_at")) AS INTEGER)
        ELSE "revoked_at"
    END,
    "rotated_at" = CASE
        WHEN typeof("rotated_at") = 'text' AND strftime('%s', trim("rotated_at")) IS NOT NULL
        THEN CAST(strftime('%s', trim("rotated_at")) AS INTEGER)
        ELSE "rotated_at"
    END,
    "updated_at" = CASE
        WHEN typeof("updated_at") = 'text' AND strftime('%s', trim("updated_at")) IS NOT NULL
        THEN CAST(strftime('%s', trim("updated_at")) AS INTEGER)
        ELSE "updated_at"
    END
WHERE (typeof("created_at") = 'text' AND strftime('%s', trim("created_at")) IS NOT NULL)
   OR (typeof("expires_at") = 'text' AND strftime('%s', trim("expires_at")) IS NOT NULL)
   OR (typeof("last_seen_at") = 'text' AND strftime('%s', trim("last_seen_at")) IS NOT NULL)
   OR (typeof("revoked_at") = 'text' AND strftime('%s', trim("revoked_at")) IS NOT NULL)
   OR (typeof("rotated_at") = 'text' AND strftime('%s', trim("rotated_at")) IS NOT NULL)
   OR (typeof("updated_at") = 'text' AND strftime('%s', trim("updated_at")) IS NOT NULL);

DROP TABLE IF EXISTS temp._aether_remaining_timestamp_storage_guard;
CREATE TEMP TABLE _aether_remaining_timestamp_storage_guard (
    invalid_count INTEGER NOT NULL CHECK (invalid_count = 0)
);

INSERT INTO _aether_remaining_timestamp_storage_guard (invalid_count)
SELECT
    EXISTS (
      SELECT 1
      FROM "announcement_reads"
      WHERE ("read_at" IS NOT NULL AND typeof("read_at") <> 'integer')
  )
  + EXISTS (
      SELECT 1
      FROM "audit_logs"
      WHERE ("created_at" IS NOT NULL AND typeof("created_at") <> 'integer')
  )
  + EXISTS (
      SELECT 1
      FROM "background_task_events"
      WHERE ("created_at_unix_secs" IS NOT NULL AND typeof("created_at_unix_secs") <> 'integer')
  )
  + EXISTS (
      SELECT 1
      FROM "background_task_runs"
      WHERE ("created_at_unix_secs" IS NOT NULL AND typeof("created_at_unix_secs") <> 'integer')
       OR ("finished_at_unix_secs" IS NOT NULL AND typeof("finished_at_unix_secs") <> 'integer')
       OR ("started_at_unix_secs" IS NOT NULL AND typeof("started_at_unix_secs") <> 'integer')
       OR ("updated_at_unix_secs" IS NOT NULL AND typeof("updated_at_unix_secs") <> 'integer')
  )
  + EXISTS (
      SELECT 1
      FROM "entitlement_usage_ledgers"
      WHERE ("created_at" IS NOT NULL AND typeof("created_at") <> 'integer')
  )
  + EXISTS (
      SELECT 1
      FROM "gemini_file_mappings"
      WHERE ("created_at" IS NOT NULL AND typeof("created_at") <> 'integer')
       OR ("expires_at" IS NOT NULL AND typeof("expires_at") <> 'integer')
  )
  + EXISTS (
      SELECT 1
      FROM "ldap_configs"
      WHERE ("created_at" IS NOT NULL AND typeof("created_at") <> 'integer')
       OR ("updated_at" IS NOT NULL AND typeof("updated_at") <> 'integer')
  )
  + EXISTS (
      SELECT 1
      FROM "pool_member_scores"
      WHERE ("last_failure_at" IS NOT NULL AND typeof("last_failure_at") <> 'integer')
       OR ("last_probe_attempt_at" IS NOT NULL AND typeof("last_probe_attempt_at") <> 'integer')
       OR ("last_probe_failure_at" IS NOT NULL AND typeof("last_probe_failure_at") <> 'integer')
       OR ("last_probe_success_at" IS NOT NULL AND typeof("last_probe_success_at") <> 'integer')
       OR ("last_ranked_at" IS NOT NULL AND typeof("last_ranked_at") <> 'integer')
       OR ("last_scheduled_at" IS NOT NULL AND typeof("last_scheduled_at") <> 'integer')
       OR ("last_success_at" IS NOT NULL AND typeof("last_success_at") <> 'integer')
       OR ("updated_at" IS NOT NULL AND typeof("updated_at") <> 'integer')
  )
  + EXISTS (
      SELECT 1
      FROM "proxy_node_events"
      WHERE ("created_at" IS NOT NULL AND typeof("created_at") <> 'integer')
  )
  + EXISTS (
      SELECT 1
      FROM "proxy_node_metrics_1h"
      WHERE ("bucket_start_unix_secs" IS NOT NULL AND typeof("bucket_start_unix_secs") <> 'integer')
  )
  + EXISTS (
      SELECT 1
      FROM "proxy_node_metrics_1m"
      WHERE ("bucket_start_unix_secs" IS NOT NULL AND typeof("bucket_start_unix_secs") <> 'integer')
  )
  + EXISTS (
      SELECT 1
      FROM "referral_rewards"
      WHERE ("created_at" IS NOT NULL AND typeof("created_at") <> 'integer')
       OR ("updated_at" IS NOT NULL AND typeof("updated_at") <> 'integer')
  )
  + EXISTS (
      SELECT 1
      FROM "request_candidates"
      WHERE ("created_at" IS NOT NULL AND typeof("created_at") <> 'integer')
       OR ("finished_at" IS NOT NULL AND typeof("finished_at") <> 'integer')
       OR ("started_at" IS NOT NULL AND typeof("started_at") <> 'integer')
  )
  + EXISTS (
      SELECT 1
      FROM "routing_group_bindings"
      WHERE ("created_at" IS NOT NULL AND typeof("created_at") <> 'integer')
       OR ("updated_at" IS NOT NULL AND typeof("updated_at") <> 'integer')
  )
  + EXISTS (
      SELECT 1
      FROM "routing_group_versions"
      WHERE ("created_at" IS NOT NULL AND typeof("created_at") <> 'integer')
  )
  + EXISTS (
      SELECT 1
      FROM "routing_groups"
      WHERE ("created_at" IS NOT NULL AND typeof("created_at") <> 'integer')
       OR ("published_at" IS NOT NULL AND typeof("published_at") <> 'integer')
       OR ("updated_at" IS NOT NULL AND typeof("updated_at") <> 'integer')
  )
  + EXISTS (
      SELECT 1
      FROM "stats_daily_api_key"
      WHERE ("created_at" IS NOT NULL AND typeof("created_at") <> 'integer')
       OR ("date" IS NOT NULL AND typeof("date") <> 'integer')
       OR ("updated_at" IS NOT NULL AND typeof("updated_at") <> 'integer')
  )
  + EXISTS (
      SELECT 1
      FROM "stats_daily_error"
      WHERE ("created_at" IS NOT NULL AND typeof("created_at") <> 'integer')
       OR ("date" IS NOT NULL AND typeof("date") <> 'integer')
       OR ("updated_at" IS NOT NULL AND typeof("updated_at") <> 'integer')
  )
  + EXISTS (
      SELECT 1
      FROM "stats_daily_model"
      WHERE ("created_at" IS NOT NULL AND typeof("created_at") <> 'integer')
       OR ("date" IS NOT NULL AND typeof("date") <> 'integer')
       OR ("updated_at" IS NOT NULL AND typeof("updated_at") <> 'integer')
  )
  + EXISTS (
      SELECT 1
      FROM "stats_daily_provider"
      WHERE ("created_at" IS NOT NULL AND typeof("created_at") <> 'integer')
       OR ("date" IS NOT NULL AND typeof("date") <> 'integer')
       OR ("updated_at" IS NOT NULL AND typeof("updated_at") <> 'integer')
  )
  + EXISTS (
      SELECT 1
      FROM "stats_daily"
      WHERE ("aggregated_at" IS NOT NULL AND typeof("aggregated_at") <> 'integer')
       OR ("created_at" IS NOT NULL AND typeof("created_at") <> 'integer')
       OR ("date" IS NOT NULL AND typeof("date") <> 'integer')
       OR ("updated_at" IS NOT NULL AND typeof("updated_at") <> 'integer')
  )
  + EXISTS (
      SELECT 1
      FROM "stats_hourly_model"
      WHERE ("created_at" IS NOT NULL AND typeof("created_at") <> 'integer')
       OR ("hour_utc" IS NOT NULL AND typeof("hour_utc") <> 'integer')
       OR ("updated_at" IS NOT NULL AND typeof("updated_at") <> 'integer')
  )
  + EXISTS (
      SELECT 1
      FROM "stats_hourly_provider"
      WHERE ("created_at" IS NOT NULL AND typeof("created_at") <> 'integer')
       OR ("hour_utc" IS NOT NULL AND typeof("hour_utc") <> 'integer')
       OR ("updated_at" IS NOT NULL AND typeof("updated_at") <> 'integer')
  )
  + EXISTS (
      SELECT 1
      FROM "stats_hourly_user_model"
      WHERE ("created_at" IS NOT NULL AND typeof("created_at") <> 'integer')
       OR ("hour_utc" IS NOT NULL AND typeof("hour_utc") <> 'integer')
       OR ("updated_at" IS NOT NULL AND typeof("updated_at") <> 'integer')
  )
  + EXISTS (
      SELECT 1
      FROM "stats_hourly_user"
      WHERE ("created_at" IS NOT NULL AND typeof("created_at") <> 'integer')
       OR ("hour_utc" IS NOT NULL AND typeof("hour_utc") <> 'integer')
       OR ("updated_at" IS NOT NULL AND typeof("updated_at") <> 'integer')
  )
  + EXISTS (
      SELECT 1
      FROM "stats_hourly"
      WHERE ("aggregated_at" IS NOT NULL AND typeof("aggregated_at") <> 'integer')
       OR ("created_at" IS NOT NULL AND typeof("created_at") <> 'integer')
       OR ("hour_utc" IS NOT NULL AND typeof("hour_utc") <> 'integer')
       OR ("updated_at" IS NOT NULL AND typeof("updated_at") <> 'integer')
  )
  + EXISTS (
      SELECT 1
      FROM "stats_user_daily"
      WHERE ("created_at" IS NOT NULL AND typeof("created_at") <> 'integer')
       OR ("date" IS NOT NULL AND typeof("date") <> 'integer')
       OR ("updated_at" IS NOT NULL AND typeof("updated_at") <> 'integer')
  )
  + EXISTS (
      SELECT 1
      FROM "usage_counter_deltas"
      WHERE ("candidate_last_used_at_unix_secs" IS NOT NULL AND typeof("candidate_last_used_at_unix_secs") <> 'integer')
       OR ("created_at" IS NOT NULL AND typeof("created_at") <> 'integer')
       OR ("last_used_at_unix_secs" IS NOT NULL AND typeof("last_used_at_unix_secs") <> 'integer')
       OR ("processed_at" IS NOT NULL AND typeof("processed_at") <> 'integer')
       OR ("removed_last_used_at_unix_secs" IS NOT NULL AND typeof("removed_last_used_at_unix_secs") <> 'integer')
       OR ("usage_created_at_unix_secs" IS NOT NULL AND typeof("usage_created_at_unix_secs") <> 'integer')
  )
  + EXISTS (
      SELECT 1
      FROM "user_invite_codes"
      WHERE ("created_at" IS NOT NULL AND typeof("created_at") <> 'integer')
       OR ("updated_at" IS NOT NULL AND typeof("updated_at") <> 'integer')
  )
  + EXISTS (
      SELECT 1
      FROM "user_preferences"
      WHERE ("created_at" IS NOT NULL AND typeof("created_at") <> 'integer')
       OR ("updated_at" IS NOT NULL AND typeof("updated_at") <> 'integer')
  )
  + EXISTS (
      SELECT 1
      FROM "user_referrals"
      WHERE ("created_at" IS NOT NULL AND typeof("created_at") <> 'integer')
       OR ("first_paid_at" IS NOT NULL AND typeof("first_paid_at") <> 'integer')
       OR ("updated_at" IS NOT NULL AND typeof("updated_at") <> 'integer')
  )
  + EXISTS (
      SELECT 1
      FROM "user_sessions"
      WHERE ("created_at" IS NOT NULL AND typeof("created_at") <> 'integer')
       OR ("expires_at" IS NOT NULL AND typeof("expires_at") <> 'integer')
       OR ("last_seen_at" IS NOT NULL AND typeof("last_seen_at") <> 'integer')
       OR ("revoked_at" IS NOT NULL AND typeof("revoked_at") <> 'integer')
       OR ("rotated_at" IS NOT NULL AND typeof("rotated_at") <> 'integer')
       OR ("updated_at" IS NOT NULL AND typeof("updated_at") <> 'integer')
  );

DROP TABLE temp._aether_remaining_timestamp_storage_guard;

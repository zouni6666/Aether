-- Worker supervisors are registered as one logical row per task. Older binaries
-- included the ephemeral gateway instance in the row id, leaving a permanently
-- running row after every restart. Remove only those system-generated boot rows;
-- current workers recreate the stable logical rows after migrations complete.
-- The metadata predicate also replaces task-only rows written by early builds of
-- this fix that still claimed an instance owner. Delete children explicitly so
-- cleanup remains complete after imports performed with FK checks disabled.
DELETE FROM background_task_events
WHERE run_id IN (
    SELECT id
    FROM background_task_runs
    WHERE id LIKE 'boot:%'
      AND owner_instance IS NOT NULL
      AND created_by = 'system'
      AND progress_message = 'worker booted'
);

DELETE FROM background_task_runs
WHERE id LIKE 'boot:%'
  AND owner_instance IS NOT NULL
  AND created_by = 'system'
  AND progress_message = 'worker booted';

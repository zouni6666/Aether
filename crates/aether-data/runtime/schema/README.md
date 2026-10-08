# Aether Schema Source

This directory is the schema maintenance workspace. Executable migrations are
owned by the Postgres adapter under `../../adapters/`.
The Postgres bootstrap snapshot is compiled from the source fragments here
during `aether-data` builds, so there is no checked-in bootstrap artifact.

The maintenance flow is:

```bash
bash crates/aether-data/runtime/schema/compose_schema.sh generate
bash crates/aether-data/runtime/schema/compose_schema.sh compose
bash crates/aether-data/runtime/schema/compose_schema.sh check
```

- `generate` renders `logical/*.toml` through `aether-data-schema` into
  `generated/postgres`. This is a build output, not another SQL
  source to maintain.
- `compose` rewrites the executable SQL from the manifest order.
- `check` verifies generated output is current, confirms the bootstrap source
  fragments still compose cleanly, and diffs each executable migration manifest
  against the checked-in SQL.
- `split` regenerates fragments from the executable SQL and is mostly for
  rebaselining after a deliberate bulk rewrite.

## What To Edit

The schema workspace has three normal source areas:

| Path | Role | Edit policy |
|---|---|---|
| `logical/*.toml` | Long-term logical table model shared by all SQL drivers. | Edit first for portable table-shape changes. |
| `drivers/postgres/` | Current maintenance fragments for executable SQL. | Edit only for deployment compatibility, ordering, or generator gaps. |
| `bootstrap/postgres/` | Source fragments for the Postgres empty-database bootstrap snapshot. | Edit here when the bootstrap snapshot changes, then rebuild `aether-data` so `build.rs` regenerates the embedded snapshot. |

Generated schema and the composed baseline are outputs:

| Path | Role | Edit policy |
|---|---|---|
| `generated/postgres/` | Machine-written SQL emitted from `logical/*.toml` for audit and drift detection. | Do not edit; regenerate with `compose_schema.sh generate`. |
| `../../adapters/postgres/migrations/20260403000000_baseline.sql` | Composed PostgreSQL baseline embedded by the adapter. | Regenerate through `compose_schema.sh compose`; do not edit independently. |

Later incremental migrations are maintained directly under
`../../adapters/postgres/migrations/`; they have no compose target. Add a new
version for an upgrade and preserve the checksums of already-applied scripts.
Keep any corresponding maintained bootstrap definitions in sync.

`generated/**` is deliberately checked in so reviews and CI can see exactly
what the logical schema compiler emits for each driver. It is not a fourth SQL
source of truth, and runtime code never loads migrations from it.

`overrides/` is an exception bucket, not a regular source tree. Keep it empty
except for its README until a real driver-specific SQL file is needed and added
to a manifest.

The generator can also be called directly:

```bash
cargo run -p aether-data-schema --bin aether-schema -- check
cargo run -p aether-data-schema --bin aether-schema -- generate
cargo run -p aether-data-schema --bin aether-schema -- print --driver postgres
```

## Logical Schema

`logical/*.toml` is the long-term source for table definitions. It covers the
PostgreSQL baseline table set. The generator emits driver-specific SQL under `generated/`; those
files include a directory README plus `Do not edit` headers and should only
change through `compose_schema.sh generate`.

`compose_schema.sh check` enforces two things:

- generated SQL must match the current logical TOML source
- bootstrap source fragments must still compose cleanly for the runtime build
- required executable SQL tables must have logical definitions, so new portable
  tables cannot bypass the single-maintenance-source path

The migration path is incremental:

1. Add a table/domain to `logical/*.toml`.
2. Run `compose_schema.sh generate`.
3. Compare generated SQL to the current driver fragments.
4. Promote generated output into driver fragments only when that domain is
   intentionally ready to stop being handwritten.
5. Keep driver-specific special cases in explicit override fragments under
   `overrides/` only when they cannot live cleanly in a driver fragment.
6. Once a domain matches, move its baseline maintenance to generated output.

The existing `drivers/postgres`
fragment trees remain authoritative for executable migrations until a generated
fragment is deliberately promoted.

`overrides/` is reserved for rare driver-specific SQL that cannot be represented
by logical schema or the normal driver fragments. Keep it small and explicit.

## Targets

| Target | Executable SQL | Source manifest |
|---|---|---|
| Postgres baseline | `../../adapters/postgres/migrations/20260403000000_baseline.sql` | `drivers/postgres/baseline/manifest.txt` |
| Postgres empty-database snapshot | `aether-data` build output (`OUT_DIR/empty_database_snapshot.sql`) | `bootstrap/postgres/manifest.txt` |

Driver baseline source manifests are kept as a small set of numbered SQL
fragments. Postgres uses execution-phase fragments so the pg_dump ordering
remains byte-for-byte stable when composed:

- `001_types_and_tables.sql`
- `002_defaults.sql`
- `003_constraints.sql`
- `004_indexes.sql`
- `005_foreign_keys.sql`
- `006_footer.sql`
- `100_*` extension files for empty-database snapshot-only additions

The Rust migration tests compose these manifests too, so fragment drift is
caught during `cargo test -p aether-data split_baseline_sources_match_executable_migrations`.

## Statistics Migrations

The statistics release retains its applied migration history and includes
incremental upgrades for databases that ran the earlier overview and dashboard
definitions. Concurrent index operations remain separate because PostgreSQL
cannot run them inside a transaction.

| Version | Change |
|---|---|
| `20260911000000` | Overview facts, attribution, aggregate tables, and transaction-owned dirty-event queue. Attribution indexes are created while the new table is empty. |
| `20260917000000` | Original account-attribution migration, retained byte-for-byte for databases that already applied it. |
| `20260917000100` | Upgrade the original attribution trigger to the dirty-event queue before later concurrent index builds. |
| `20260918000000` | Create the replacement settlement covering index concurrently. |
| `20260918000100` | Drop the previous settlement covering index concurrently, after its replacement succeeds. |
| `20260919000000` | Dashboard aggregates, activation boundary, and retention support. |
| `20260920000000` | Create the credited-payment lookup index concurrently. |
| `20260920120000` | Provider expense records. |
| `20260921010000` | Add retention support to existing dashboard schemas; safe when the initial dashboard migration already includes it. |
| `20260921020000` | Add the attribution-owner lookup index concurrently on existing databases. |
| `20260921020100` | Create the usage metadata actor index concurrently. |
| `20261001000000` | Remove deleted-user attribution from dashboard activity on future user deletion; schema-only upgrade without rewriting historical rows. |
| `20261004000000` | Parse request metadata once per overview fact; replace only the view definition without rewriting facts or statistics. |

Do not remove an applied migration after folding its changes into an earlier
schema definition. Existing databases retain its version in `_sqlx_migrations`
and do not rerun earlier versions when their SQL changes. Preserve that history
and provide incremental migrations for any remaining schema differences.

These migrations do not backfill historical requests. Dashboard totals start at
the stored activation boundary. Background maintenance compacts dashboard minute
details older than 35 days in bounded batches, preserving cumulative totals and
the narrow activity counts; it does not delete source usage. JSONL backups include
the dashboard snapshot and its integrity manifest so retained totals can survive
restoration after source usage has expired.

Schema migrations and historical backfills remain separate phases. Normal `auto`
startup and `db prepare` still apply pending scripts from `backfills/postgres`
after schema migration; `verify-only` still requires both phases to be current.
The statistics schema migrations above do not embed a historical data rebuild.
The new dashboard's activation boundary is not moved by legacy backfills, so
they do not restore pre-activation dashboard totals.

For upgraded installations that retain historical `stats_daily` rows and a
`stats_summary` cutoff, the homepage has a read-only compatibility path. Lifetime
totals and activity combine daily aggregates strictly before that cutoff with
recent usage at or after it, in one database snapshot. This preserves history
whose individual requests have already expired and avoids double counting when
daily aggregation advances the cutoff. These activity dates use UTC, matching
the historical aggregates; the API reports `activity_timezone` separately from
the timezone used for today's metrics. On the activation day, today's metrics
also include requests before activation, with active users deduplicated across
the boundary. The stored activation timestamp and source records are unchanged.

This compatibility path retains the legacy daily-aggregation retention contract:
unaggregated recent usage must remain available until the daily cutoff advances.
Installations without historical daily aggregates continue to read the durable
incremental dashboard projection.

Deleted users are excluded from dashboard active-user reads even when an older
version left orphan activity rows. The anonymization upgrade installs rules for
future deletions without cleaning old rows during migration; those old activity
rows age out through the existing 35-day retention task.

The overview worker can still rebuild a historical hour/day when normal writes
change facts in that bucket. That work runs after startup with bounded batches
and query deadlines; it is not a full historical rebuild during migration.

The migration runner defaults to a 1-second lock wait, a 10-second deadline per
transactional migration, and a 15-minute deadline per concurrent index migration.
Timeouts are configurable through `AETHER_POSTGRES_MIGRATION_LOCK_TIMEOUT_MS`,
`AETHER_POSTGRES_MIGRATION_TIMEOUT_MS`, and
`AETHER_POSTGRES_MIGRATION_CONCURRENT_TIMEOUT_MS`; none accepts zero. An independent
control connection attempts to terminate the migration session on failure or
cancellation. An interrupted concurrent index build can leave an invalid index;
the runner removes that index before retrying its migration.

For Compose deployments, `update.sh` applies schema migrations with the new image
before replacing the running app. A migration failure stops the update; already
committed migrations remain applied. Its `local-build` mode delegates to
`deploy.sh` and does not use this separate migration step. Allow for brief table
locks and I/O pressure from concurrent index scans during the upgrade. Keeping
historical backfills out of schema migrations does not make index creation
constant-time: concurrent indexes still scan existing rows and can take minutes
on a large database. The existing app stays running during the Compose migration
preflight.

use super::*;

fn tables() -> impl Iterator<Item = &'static AuxiliaryTable> {
    AUXILIARY_TABLES
        .iter()
        .filter(|table| table.name.starts_with("dashboard_"))
}

fn fingerprint(rows: &[Value]) -> DashboardSnapshotTable {
    // Sorting canonical JSON makes fingerprints independent of file row order.
    let mut encoded = rows
        .iter()
        .map(|row| {
            let mut canonical = row.clone();
            canonical.sort_all_objects();
            canonical.to_string()
        })
        .collect::<Vec<_>>();
    encoded.sort();
    let mut digest = Sha256::new();
    for row in encoded {
        digest.update(row.as_bytes());
        digest.update(b"\n");
    }
    DashboardSnapshotTable {
        rows: rows.len(),
        sha256: format!("{:x}", digest.finalize()),
    }
}

fn snapshot_rows<'a>(
    rows: impl Iterator<Item = &'a ExportRow>,
) -> Result<BTreeMap<String, Vec<Value>>, DataLayerError> {
    let mut result = tables()
        .map(|table| (table.name.to_owned(), Vec::new()))
        .collect::<BTreeMap<_, _>>();
    for row in rows {
        let (name, payload) = domain_payload_table(row, "auxiliary", None)?;
        if let Some(entries) = result.get_mut(&name) {
            entries.push(payload);
        }
    }
    Ok(result)
}

pub(super) fn attach_manifest(records: &mut [DataExportRecord]) -> Result<(), DataLayerError> {
    let Some(DataExportRecord::Manifest { manifest }) = records.first() else {
        return Ok(());
    };
    if !manifest.domains.contains(&ExportDomain::Auxiliary) {
        return Ok(());
    }
    let rows = records
        .iter()
        .filter_map(|record| match record {
            DataExportRecord::Row {
                domain: ExportDomain::Auxiliary,
                id,
                payload,
            } => Some(ExportRow {
                id: id.clone(),
                payload: payload.clone(),
            }),
            _ => None,
        })
        .collect::<Vec<_>>();
    let values = snapshot_rows(rows.iter())?;
    let tables = values
        .into_iter()
        .map(|(name, values)| (name, fingerprint(&values)))
        .collect();
    if let Some(DataExportRecord::Manifest { manifest }) = records.first_mut() {
        manifest.dashboard_snapshot = Some(DashboardSnapshotManifest { version: 1, tables });
    }
    Ok(())
}

fn invalid(detail: &str) -> DataLayerError {
    DataLayerError::InvalidInput(format!("dashboard snapshot {detail}"))
}

pub(super) async fn prepare_restore(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    plan: &DataImportPlan,
) -> Result<(), DataLayerError> {
    let values = snapshot_rows(plan.rows(ExportDomain::Auxiliary).iter())?;
    let Some(manifest) = &plan.manifest.dashboard_snapshot else {
        if values.values().any(|rows| !rows.is_empty()) {
            return Err(invalid(
                "requires its complete manifest; partial aggregate imports cannot be merged",
            ));
        }
        return Ok(()); // Legacy backups deliberately keep normal trigger behavior.
    };
    if manifest.version != 1
        || !plan.imports_domain(ExportDomain::Auxiliary)
        || manifest.tables.len() != values.len()
    {
        return Err(invalid(
            "has an unsupported version or incomplete table inventory",
        ));
    }
    for (name, rows) in &values {
        if manifest.tables.get(name) != Some(&fingerprint(rows)) {
            return Err(invalid(&format!(
                "table '{name}' is missing, truncated, or has changed"
            )));
        }
    }
    let state = &values["dashboard_stats_state"];
    let totals = &values["dashboard_stats_total"];
    let shards = totals
        .iter()
        .filter_map(|row| row.get("shard").and_then(Value::as_u64))
        .collect::<BTreeSet<_>>();
    if state.len() != 1
        || state[0].get("singleton") != Some(&Value::Bool(true))
        || state[0]
            .get("stats_since")
            .and_then(Value::as_str)
            .and_then(parse_imported_datetime)
            .is_none()
        || totals.len() != 16
        || shards != (0..16).collect()
    {
        return Err(invalid(
            "must include one activation state and all 16 total shards",
        ));
    }
    validate_request_counts(&values)?;

    // Exclude concurrent source writes before examining or replacing projections.
    // Their triggers acquire projection locks in this same source-first order.
    sqlx::query("LOCK TABLE public.users, public.usage, public.usage_settlement_snapshots, public.usage_attribution_snapshots IN SHARE ROW EXCLUSIVE MODE")
        .execute(&mut **tx).await.map_sql_err()?;
    // Match retention's shard -> minute -> activity -> actor -> event -> state -> ledger
    // order, otherwise a maintenance pass could deadlock against the restore.
    for table in [
        "dashboard_stats_total",
        "dashboard_stats_minute",
        "dashboard_activity_minute",
        "dashboard_actor_minute",
        "dashboard_user_events_minute",
        "dashboard_stats_state",
        "dashboard_request_contributions",
        "dashboard_activity_hour",
    ] {
        sqlx::query(&format!(
            "LOCK TABLE public.{table} IN SHARE ROW EXCLUSIVE MODE"
        ))
        .execute(&mut **tx)
        .await
        .map_sql_err()?;
    }
    let bootstrap_only = bootstrap_admin_only(tx).await?;
    let mut identical = true;
    let mut empty = true;
    for table in tables() {
        let mut current = sqlx::query_scalar::<_, Value>(&format!(
            "SELECT to_jsonb(t) FROM public.{} t",
            table.name
        ))
        .fetch_all(&mut **tx)
        .await
        .map_sql_err()?;
        // The bounded cleanup cursor is operational progress, not a change to
        // statistics; moving it alone must not make a repeated restore conflict.
        if table.name == "dashboard_stats_state" {
            for row in &mut current {
                if let Some(object) = row.as_object_mut() {
                    if let Some(cursor) = state[0].get("contributions_cleanup_cursor") {
                        object.insert("contributions_cleanup_cursor".into(), cursor.clone());
                    }
                }
            }
        }
        identical &= manifest.tables.get(table.name) == Some(&fingerprint(&current));
        empty &= match table.name {
            "dashboard_stats_state" => true, // A freshly migrated database already has an activation timestamp.
            "dashboard_stats_total" => current.iter().all(|row| {
                row.get("metrics")
                    .and_then(Value::as_object)
                    .is_some_and(|metrics| metrics.values().all(|n| n.as_f64() == Some(0.0)))
            }),
            "dashboard_user_events_minute" => current.is_empty() || bootstrap_only,
            _ => current.is_empty(),
        };
    }
    if !empty && !identical {
        return Err(invalid("conflicts with existing statistics; restore into an empty database. Complete statistics cannot be incrementally merged"));
    }
    if empty && !identical {
        let since =
            parse_imported_datetime(state[0]["stats_since"].as_str().expect("validated state"))
                .expect("validated timestamp");
        if sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM public.usage WHERE created_at >= $1)",
        )
        .bind(since)
        .fetch_one(&mut **tx)
        .await
        .map_sql_err()?
        {
            return Err(invalid("conflicts with existing requests within the restored activation period; restore into an empty database"));
        }
    }
    // A local custom setting affects only dashboard triggers, not integrity or
    // billing triggers, and is automatically reverted on both commit and rollback.
    sqlx::query("SET LOCAL aether.dashboard_restore = 'on'")
        .execute(&mut **tx)
        .await
        .map_sql_err()?;
    for table in tables() {
        sqlx::query(&format!("DELETE FROM public.{}", table.name))
            .execute(&mut **tx)
            .await
            .map_sql_err()?;
    }
    Ok(())
}

async fn bootstrap_admin_only(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> Result<bool, DataLayerError> {
    // bootstrap_admin_from_env has no persistent marker and permits a configured
    // username. Recognize only its otherwise untouched single-admin/zero-wallet
    // state, never an installation with financial or request history.
    sqlx::query("LOCK TABLE public.api_keys, public.wallets, public.payment_orders, public.wallet_transactions, public.provider_expenses IN SHARE ROW EXCLUSIVE MODE")
        .execute(&mut **tx).await.map_sql_err()?;
    sqlx::query_scalar(r#"
      SELECT (SELECT count(*) FROM users)=1
        AND (SELECT count(*) FROM wallets)=1
        AND (SELECT count(*) FROM dashboard_user_events_minute)=1
        AND NOT EXISTS(SELECT 1 FROM usage)
        AND NOT EXISTS(SELECT 1 FROM api_keys)
        AND NOT EXISTS(SELECT 1 FROM payment_orders)
        AND NOT EXISTS(SELECT 1 FROM wallet_transactions)
        AND NOT EXISTS(SELECT 1 FROM provider_expenses)
        AND EXISTS(
          SELECT 1 FROM users u JOIN wallets w ON w.user_id=u.id
          JOIN dashboard_user_events_minute e
            ON e.bucket_start=date_trunc('minute',u.created_at AT TIME ZONE 'UTC') AT TIME ZONE 'UTC'
           AND e.shard=(hashtextextended(u.id,0)&15)::smallint
          JOIN dashboard_stats_state s ON s.singleton
          WHERE u.role='admin' AND u.auth_source='local' AND u.is_active
            AND NOT u.is_deleted AND u.email_verified AND u.created_at>=s.stats_since
            AND w.api_key_id IS NULL AND w.limit_mode='unlimited'
            AND w.currency='USD' AND w.status='active'
            AND w.balance=0 AND w.gift_balance=0 AND w.total_recharged=0
            AND w.total_consumed=0 AND w.total_refunded=0 AND w.total_adjusted=0
            AND e.created_count=1 AND e.deleted_count=0
        )
    "#).fetch_one(&mut **tx).await.map_sql_err()
}

fn validate_request_counts(values: &BTreeMap<String, Vec<Value>>) -> Result<(), DataLayerError> {
    let mut totals = [0u64; 16];
    for row in &values["dashboard_stats_total"] {
        let shard = row["shard"]
            .as_u64()
            .ok_or_else(|| invalid("has an invalid shard"))? as usize;
        totals[shard] = request_count(&row["metrics"]["request_count"])?;
    }
    let mut hours = [0u64; 16];
    for row in &values["dashboard_activity_hour"] {
        let shard = row["shard"]
            .as_u64()
            .filter(|n| *n < 16)
            .ok_or_else(|| invalid("has an invalid activity shard"))? as usize;
        hours[shard] = hours[shard]
            .checked_add(request_count(&row["request_count"])?)
            .ok_or_else(|| invalid("activity counts overflow"))?;
    }
    if hours != totals {
        return Err(invalid(
            "hourly activity and cumulative request counts disagree",
        ));
    }
    Ok(())
}

fn request_count(value: &Value) -> Result<u64, DataLayerError> {
    if value.is_null() {
        return Ok(0);
    }
    value
        .as_u64()
        .ok_or_else(|| invalid("contains an invalid request count"))
}

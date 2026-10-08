use super::{
    analytics, usage_cache_creation_tokens, usage_total_input_context, usage_total_tokens,
    InMemoryUsageReadRepository, StoredRequestUsageAudit,
};
use aether_data_contracts::{repository::usage::*, DataLayerError};
use chrono::{DateTime, NaiveDate, Utc};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug)]
pub(super) struct DashboardProjection {
    pub since: DateTime<Utc>,
    entries: BTreeMap<String, Contribution>,
}
impl Default for DashboardProjection {
    fn default() -> Self {
        Self {
            since: Utc::now(),
            entries: BTreeMap::new(),
        }
    }
}
#[derive(Debug)]
struct Contribution {
    at: DateTime<Utc>,
    actor: Option<String>,
    metrics: DashboardSummaryMetrics,
    billable_units: Option<i128>,
}
impl DashboardProjection {
    pub fn record(&mut self, row: &StoredRequestUsageAudit, keys: &BTreeMap<String, bool>) {
        let Some(at) = DateTime::from_timestamp(row.created_at_unix_ms as i64, 0) else {
            return;
        };
        if at < self.since {
            return;
        }
        if row
            .request_metadata
            .as_ref()
            .and_then(|m| m.pointer("/analytics_attribution/record_kind"))
            .and_then(|v| v.as_str())
            == Some("session")
        {
            self.entries.remove(&row.request_id);
            return;
        }
        let available = |key| {
            row.request_metadata
                .as_ref()
                .and_then(|m| m.get(key))
                .and_then(|v| v.as_bool())
                != Some(false)
        };
        let usage = available(USAGE_AVAILABLE_METADATA_KEY);
        let billing_cost = row.billing_cost();
        let priced = available(USAGE_PRICING_AVAILABLE_METADATA_KEY)
            && row.billing_status == "settled"
            && billing_cost.is_some();
        let stream = row
            .request_metadata
            .as_ref()
            .and_then(|m| m.get("upstream_is_stream"))
            .and_then(|v| v.as_bool())
            .unwrap_or(row.is_stream);
        let metrics = DashboardSummaryMetrics {
            request_count: 1,
            input_tokens: if usage { row.input_tokens } else { 0 },
            output_tokens: if usage { row.output_tokens } else { 0 },
            total_tokens: if usage {
                if row.total_tokens > 0 {
                    row.total_tokens
                } else {
                    usage_total_tokens(row)
                }
            } else {
                0
            },
            usage_available_count: u64::from(usage),
            pricing_available_count: u64::from(priced),
            cache_read_tokens: if usage {
                row.cache_read_input_tokens
            } else {
                0
            },
            cache_creation_tokens: if usage {
                usage_cache_creation_tokens(row)
            } else {
                0
            },
            cache_input_tokens: if usage {
                usage_total_input_context(row)
            } else {
                0
            },
            first_byte_sum_ms: row.first_byte_time_ms.unwrap_or(0) as f64,
            first_byte_sample_count: u64::from(row.first_byte_time_ms.is_some()),
            response_sum_ms: row.response_time_ms.unwrap_or(0) as f64,
            response_sample_count: u64::from(row.response_time_ms.is_some()),
            stream_requests: u64::from(stream),
            standard_requests: u64::from(!stream),
            ..Default::default()
        };
        self.entries.insert(
            row.request_id.clone(),
            Contribution {
                at,
                actor: analytics::actor(row, keys).map(str::to_owned),
                metrics,
                billable_units: priced
                    .then_some(billing_cost)
                    .flatten()
                    .map(|cost| (cost * 100_000_000.0).round() as i128),
            },
        );
    }
}
fn sum_metrics<'a>(rows: impl Iterator<Item = &'a Contribution>) -> DashboardSummaryMetrics {
    let mut sum = DashboardSummaryMetrics::default();
    let mut users = BTreeSet::new();
    let mut units = 0_i128;
    for row in rows {
        let m = &row.metrics;
        sum.request_count += m.request_count;
        sum.input_tokens += m.input_tokens;
        sum.output_tokens += m.output_tokens;
        sum.total_tokens += m.total_tokens;
        sum.usage_available_count += m.usage_available_count;
        sum.pricing_available_count += m.pricing_available_count;
        sum.cache_read_tokens += m.cache_read_tokens;
        sum.cache_creation_tokens += m.cache_creation_tokens;
        sum.cache_input_tokens += m.cache_input_tokens;
        sum.first_byte_sum_ms += m.first_byte_sum_ms;
        sum.first_byte_sample_count += m.first_byte_sample_count;
        sum.response_sum_ms += m.response_sum_ms;
        sum.response_sample_count += m.response_sample_count;
        sum.stream_requests += m.stream_requests;
        sum.standard_requests += m.standard_requests;
        units += row.billable_units.unwrap_or(0);
        if let Some(actor) = row.actor.as_deref() {
            users.insert(actor);
        }
    }
    sum.active_users = users.len() as u64;
    if sum.request_count == 0 || sum.pricing_available_count > 0 {
        sum.billable_amount = Some(format!(
            "{}{}.{:08}",
            if units < 0 { "-" } else { "" },
            units.abs() / 100_000_000,
            units.abs() % 100_000_000
        ));
    }
    sum
}
impl InMemoryUsageReadRepository {
    /// Test/embedded initialization boundary; seeded older audit rows stay excluded.
    pub fn with_dashboard_stats_since(self, since: DateTime<Utc>) -> Self {
        let keys = self.analytics_key_flags();
        let mut projection = self
            .dashboard_projection
            .write()
            .expect("dashboard projection lock");
        projection.since = since;
        projection.entries.clear();
        for row in self
            .by_request_id
            .read()
            .expect("usage repository lock")
            .values()
        {
            projection.record(row, &keys);
        }
        drop(projection);
        self
    }
    pub(super) fn dashboard_summary_query(
        &self,
        query: &UsageDashboardAnalyticsQuery,
    ) -> Result<StoredDashboardSummary, DataLayerError> {
        query.validate()?;
        let projection = self.dashboard_projection.read().map_err(|_| {
            DataLayerError::UnexpectedValue("dashboard projection lock poisoned".into())
        })?;
        let now = Utc::now();
        let today_from = query.today_start(now)?.max(projection.since);
        let tz = query
            .timezone
            .parse::<chrono_tz::Tz>()
            .map_err(|_| DataLayerError::InvalidInput("invalid timezone".into()))?;
        let rows = || projection.entries.values().filter(|row| row.at < now);
        let today = sum_metrics(rows().filter(|row| row.at >= today_from));
        let total = sum_metrics(rows());
        let mut days = BTreeMap::<NaiveDate, u64>::new();
        for row in rows() {
            *days
                .entry(row.at.with_timezone(&tz).date_naive())
                .or_default() += 1;
        }
        let active_days = days.len() as u64;
        let local_today = now.with_timezone(&tz).date_naive();
        let consecutive_active_days =
            dashboard_consecutive_active_days(days.keys().copied(), local_today);
        let earliest_day = local_today - chrono::Duration::days(364);
        let activity_days = days
            .into_iter()
            .filter(|(day, _)| day >= &earliest_day)
            .map(|(date, requests)| DashboardActivityDay {
                date: date.to_string(),
                requests,
            })
            .collect();
        let users = self
            .analytics_users
            .read()
            .map_err(|_| DataLayerError::UnexpectedValue("users lock poisoned".into()))?;
        Ok(StoredDashboardSummary {
            stats_since: projection.since.to_rfc3339(),
            generated_at: now.to_rfc3339(),
            timezone: query.timezone.clone(),
            activity_timezone: query.timezone.clone(),
            today_from: today_from.to_rfc3339(),
            window_seconds: (now - today_from).num_milliseconds().max(0) as f64 / 1000.0,
            today,
            total,
            users: DashboardUserCounts {
                total: users.iter().filter(|user| !user.is_deleted).count() as u64,
                ..Default::default()
            },
            active_days,
            consecutive_active_days,
            activity_days,
        })
    }
}

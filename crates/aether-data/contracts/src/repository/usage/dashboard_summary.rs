use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

/// Additive dashboard facts collected after this installation enabled aggregation.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct DashboardSummaryMetrics {
    pub request_count: u64,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub total_tokens: u64,
    pub usage_available_count: u64,
    pub pricing_available_count: u64,
    pub billable_amount: Option<String>,
    pub active_users: u64,
    pub cache_read_tokens: u64,
    pub cache_creation_tokens: u64,
    pub cache_input_tokens: u64,
    pub first_byte_sum_ms: f64,
    pub first_byte_sample_count: u64,
    pub response_sum_ms: f64,
    pub response_sample_count: u64,
    pub stream_requests: u64,
    pub standard_requests: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DashboardUserCounts {
    pub total: u64,
    pub created_today: u64,
    pub deleted_today: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DashboardActivityDay {
    pub date: String,
    pub requests: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct StoredDashboardSummary {
    pub stats_since: String,
    pub generated_at: String,
    pub timezone: String,
    /// Historical daily imports retain their original UTC calendar boundaries.
    #[serde(default)]
    pub activity_timezone: String,
    pub today_from: String,
    pub window_seconds: f64,
    pub today: DashboardSummaryMetrics,
    pub total: DashboardSummaryMetrics,
    pub users: DashboardUserCounts,
    pub active_days: u64,
    #[serde(default)]
    pub consecutive_active_days: u64,
    pub activity_days: Vec<DashboardActivityDay>,
}

/// Current activity streak from distinct local dates ordered oldest to newest.
/// An unfinished today may be inactive, so a streak ending yesterday still counts.
pub fn dashboard_consecutive_active_days(
    days: impl DoubleEndedIterator<Item = NaiveDate>,
    today: NaiveDate,
) -> u64 {
    let mut days = days.rev().filter(|date| *date <= today);
    let Some(mut latest) = days.next() else {
        return 0;
    };
    if latest != today && Some(latest) != today.pred_opt() {
        return 0;
    }
    let mut consecutive = 1;
    for date in days {
        if Some(date) != latest.pred_opt() {
            break;
        }
        consecutive += 1;
        latest = date;
    }
    consecutive
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(value: &str) -> NaiveDate {
        NaiveDate::parse_from_str(value, "%Y-%m-%d").unwrap()
    }

    #[test]
    fn dashboard_activity_streak_handles_empty_stale_and_interrupted_days() {
        let today = date("2026-09-19");
        assert_eq!(dashboard_consecutive_active_days([].into_iter(), today), 0);
        assert_eq!(
            dashboard_consecutive_active_days([date("2026-09-17")].into_iter(), today),
            0
        );
        assert_eq!(
            dashboard_consecutive_active_days(
                ["2026-09-15", "2026-09-17", "2026-09-18", "2026-09-19"]
                    .map(date)
                    .into_iter(),
                today,
            ),
            3
        );
    }

    #[test]
    fn dashboard_activity_streak_can_end_yesterday_across_month_and_year() {
        assert_eq!(
            dashboard_consecutive_active_days(
                ["2025-12-30", "2025-12-31", "2026-01-01"]
                    .map(date)
                    .into_iter(),
                date("2026-01-02"),
            ),
            3
        );
    }

    #[test]
    fn dashboard_activity_streak_uses_full_history_and_ignores_future_dates() {
        let today = date("2026-09-19");
        let days = (-399..=1).map(|offset| today + chrono::Duration::days(offset));
        assert_eq!(dashboard_consecutive_active_days(days, today), 400);
    }
}

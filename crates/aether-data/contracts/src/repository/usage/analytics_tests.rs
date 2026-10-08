use super::*;
use chrono::DateTime;

fn query(
    from: &str,
    to: &str,
    timezone: &str,
    granularity: UsageAnalyticsGranularity,
) -> UsageAnalyticsQuery {
    UsageAnalyticsQuery {
        from_unix_ms: DateTime::parse_from_rfc3339(from)
            .unwrap()
            .timestamp_millis() as u64,
        to_unix_ms: DateTime::parse_from_rfc3339(to).unwrap().timestamp_millis() as u64,
        timezone: timezone.into(),
        granularity,
        limit: 25,
        ..Default::default()
    }
}

#[test]
fn local_days_follow_dst_and_empty_buckets_remain_visible() {
    let query = query(
        "2026-03-07T05:00:00Z",
        "2026-03-10T04:00:00Z",
        "America/New_York",
        UsageAnalyticsGranularity::Day,
    );
    let mut rows = Vec::new();
    fill_usage_analytics_timeseries(&query, &mut rows);
    assert_eq!(rows.len(), 3);
    assert_eq!(
        rows[2].bucket_start.as_deref(),
        Some("2026-03-09T04:00:00+00:00")
    );
    assert_eq!(
        rows[0].metrics.billable_amount.as_deref(),
        Some("0.00000000")
    );
}

#[test]
fn repeated_dst_hours_are_distinct_and_range_is_half_open() {
    let query = query(
        "2026-11-01T04:00:00Z",
        "2026-11-01T08:00:00Z",
        "America/New_York",
        UsageAnalyticsGranularity::Hour,
    );
    let mut rows = Vec::new();
    fill_usage_analytics_timeseries(&query, &mut rows);
    assert_eq!(rows.len(), 4);
    assert_ne!(rows[1].bucket_start, rows[2].bucket_start);
}

#[test]
fn dashboard_today_uses_local_day_including_skipped_midnight() {
    let query = UsageDashboardAnalyticsQuery {
        timezone: "America/Sao_Paulo".into(),
    };
    let now = DateTime::parse_from_rfc3339("2018-11-04T12:00:00Z")
        .unwrap()
        .with_timezone(&chrono::Utc);
    assert_eq!(
        query.today_start(now).unwrap().to_rfc3339(),
        "2018-11-04T03:00:00+00:00"
    );
    let query = UsageDashboardAnalyticsQuery {
        timezone: "Asia/Shanghai".into(),
    };
    assert_eq!(
        query.today_start(now).unwrap().to_rfc3339(),
        "2018-11-03T16:00:00+00:00"
    );
    assert!(UsageDashboardAnalyticsQuery {
        timezone: "invalid".into()
    }
    .validate()
    .is_err());
}

#[test]
fn daily_series_continues_after_skipped_midnight() {
    let query = query(
        "2026-09-05T04:00:00Z",
        "2026-09-08T03:00:00Z",
        "America/Santiago",
        UsageAnalyticsGranularity::Day,
    );
    let mut rows = Vec::new();
    fill_usage_analytics_timeseries(&query, &mut rows);
    assert_eq!(rows.len(), 3);
    assert_eq!(
        rows[1].bucket_start.as_deref(),
        Some("2026-09-06T04:00:00+00:00")
    );
    assert_eq!(
        rows[2].bucket_start.as_deref(),
        Some("2026-09-07T03:00:00+00:00")
    );
}

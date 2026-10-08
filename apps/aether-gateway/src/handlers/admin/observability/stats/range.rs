pub(crate) use aether_admin::observability::stats::parse_bounded_u32;
use aether_admin::observability::stats::AdminStatsTimeRange;
pub(super) use aether_admin::observability::stats::{
    admin_usage_default_days, build_comparison_range, build_time_range_from_days, parse_naive_date,
    parse_nonnegative_usize, parse_tz_offset_minutes, resolve_preset_dates, user_today,
};
use chrono::{DateTime, Offset, TimeZone, Utc};

pub(crate) fn resolve_admin_usage_time_range(
    query: Option<&str>,
) -> Result<AdminStatsTimeRange, String> {
    if let Some((from, to)) = resolve_precise_time_bounds(query)? {
        return precise_admin_stats_time_range(query, from, to);
    }
    match AdminStatsTimeRange::resolve_optional(query)? {
        Some(time_range) => Ok(time_range),
        None => {
            let tz_offset_minutes = parse_tz_offset_minutes(query)?;
            let default_days = u32::try_from(admin_usage_default_days())
                .ok()
                .filter(|value| *value > 0)
                .unwrap_or(1);
            build_time_range_from_days(default_days, tz_offset_minutes)
        }
    }
}

/// Resolve an exact UTC range supplied by the shared admin range picker.
///
/// The older stats handlers use `start_date`/`end_date` and fixed offsets. Keep
/// that parser intact and only opt into this path when both RFC 3339 endpoints
/// are present, so existing callers retain their behavior.
pub(crate) fn resolve_precise_time_bounds(
    query: Option<&str>,
) -> Result<Option<(u64, u64)>, String> {
    let entries =
        url::form_urlencoded::parse(query.unwrap_or_default().as_bytes()).collect::<Vec<_>>();
    let from = entries
        .iter()
        .filter(|(key, _)| key == "from")
        .collect::<Vec<_>>();
    let to = entries
        .iter()
        .filter(|(key, _)| key == "to")
        .collect::<Vec<_>>();
    if from.is_empty() && to.is_empty() {
        return Ok(None);
    }
    if from.len() != 1 || to.len() != 1 {
        return Err("from and to must each be provided once".into());
    }
    if entries
        .iter()
        .any(|(key, _)| matches!(key.as_ref(), "start_date" | "end_date" | "preset" | "days"))
    {
        return Err("precise from/to cannot be combined with date presets".into());
    }
    if let Some(zone) = query_param_value(query, "timezone") {
        zone.parse::<chrono_tz::Tz>()
            .map_err(|_| "invalid timezone".to_string())?;
    }
    let parse = |value: &str| -> Result<u64, String> {
        let value = DateTime::parse_from_rfc3339(value)
            .map_err(|_| "from/to must be RFC 3339 timestamps".to_string())?;
        if value.timestamp_subsec_nanos() != 0 {
            return Err("request records support second-aligned ranges".into());
        }
        u64::try_from(value.timestamp()).map_err(|_| "from/to must not precede Unix epoch".into())
    };
    let bounds = (parse(&from[0].1)?, parse(&to[0].1)?);
    if bounds.0 >= bounds.1 || bounds.1 - bounds.0 > 366 * 86_400 {
        return Err("from/to must define a nonempty range of at most 366 days".into());
    }
    Ok(Some(bounds))
}

/// Return the exact range when present, otherwise preserve the legacy stats
/// date/preset behavior.
pub(crate) fn resolve_usage_time_bounds(query: Option<&str>) -> Result<Option<(u64, u64)>, String> {
    if let Some(bounds) = resolve_precise_time_bounds(query)? {
        return Ok(Some(bounds));
    }
    Ok(resolve_admin_usage_time_range(query)?.to_unix_bounds())
}

/// Build the date metadata used by the existing stats response builders for an
/// exact range. The data query still uses the exact UTC bounds; this metadata
/// only supplies the local date labels and offset expected by old clients.
pub(crate) fn precise_admin_stats_time_range(
    query: Option<&str>,
    from: u64,
    to: u64,
) -> Result<AdminStatsTimeRange, String> {
    let timezone_name = query_param_value(query, "timezone");
    let (start_date, end_date, tz_offset_minutes) = if let Some(name) = timezone_name {
        let timezone = name
            .parse::<chrono_tz::Tz>()
            .map_err(|_| "invalid timezone".to_string())?;
        let start = Utc
            .timestamp_opt(
                i64::try_from(from).map_err(|_| "invalid from timestamp")?,
                0,
            )
            .single()
            .ok_or_else(|| "invalid from timestamp".to_string())?
            .with_timezone(&timezone);
        let end = Utc
            .timestamp_opt(
                i64::try_from(to.saturating_sub(1)).map_err(|_| "invalid to timestamp")?,
                0,
            )
            .single()
            .ok_or_else(|| "invalid to timestamp".to_string())?
            .with_timezone(&timezone);
        (
            start.date_naive(),
            end.date_naive(),
            start.offset().fix().local_minus_utc() / 60,
        )
    } else {
        let offset = parse_tz_offset_minutes(query)?;
        let fixed = chrono::FixedOffset::east_opt(offset * 60)
            .ok_or_else(|| "invalid timezone offset".to_string())?;
        let start = Utc
            .timestamp_opt(
                i64::try_from(from).map_err(|_| "invalid from timestamp")?,
                0,
            )
            .single()
            .ok_or_else(|| "invalid from timestamp".to_string())?
            .with_timezone(&fixed);
        let end = Utc
            .timestamp_opt(
                i64::try_from(to.saturating_sub(1)).map_err(|_| "invalid to timestamp")?,
                0,
            )
            .single()
            .ok_or_else(|| "invalid to timestamp".to_string())?
            .with_timezone(&fixed);
        (start.date_naive(), end.date_naive(), offset)
    };

    Ok(AdminStatsTimeRange {
        start_date,
        end_date,
        tz_offset_minutes,
    })
}

fn query_param_value(query: Option<&str>, key: &str) -> Option<String> {
    url::form_urlencoded::parse(query.unwrap_or_default().as_bytes())
        .find(|(name, _)| name == key)
        .map(|(_, value)| value.into_owned())
}

#[cfg(test)]
mod tests {
    use super::{precise_admin_stats_time_range, resolve_precise_time_bounds};

    #[test]
    fn precise_stats_range_preserves_subday_bounds_and_timezone_labels() {
        let query = "from=2026-09-01T23:45:00Z&to=2026-09-02T00:15:00Z&timezone=Asia%2FShanghai";
        let (from, to) = resolve_precise_time_bounds(Some(query)).unwrap().unwrap();
        assert_eq!(to - from, 30 * 60);
        let range = precise_admin_stats_time_range(Some(query), from, to).unwrap();
        assert_eq!(range.start_date.to_string(), "2026-09-02");
        assert_eq!(range.end_date.to_string(), "2026-09-02");
        assert_eq!(range.tz_offset_minutes, 480);
    }

    #[test]
    fn precise_stats_range_rejects_mixed_legacy_presets() {
        let query = "from=2026-09-01T00:00:00Z&to=2026-09-02T00:00:00Z&preset=today";
        assert!(resolve_precise_time_bounds(Some(query)).is_err());
    }
}

use aether_data_contracts::repository::usage::{
    UsageAnalyticsGranularity, UsageAnalyticsGroupBy, UsageAnalyticsQuery, UsageAnalyticsSort,
    UsageAnalyticsView,
};
use chrono::{DateTime, Utc};
use std::collections::BTreeMap;

pub const OVERVIEW_EXPORT_LIMIT: u32 = 10_000;

#[derive(Debug)]
pub struct OverviewRequest {
    pub query: UsageAnalyticsQuery,
    pub csv: bool,
    pub amount_basis: String,
}

pub fn parse_overview_query(
    raw: Option<&str>,
    view: UsageAnalyticsView,
) -> Result<OverviewRequest, String> {
    let mut params = BTreeMap::new();
    for (key, value) in url::form_urlencoded::parse(raw.unwrap_or_default().as_bytes()) {
        if params
            .insert(key.into_owned(), value.into_owned())
            .is_some()
        {
            return Err("duplicate query parameters are not supported".into());
        }
    }
    let from = timestamp(&mut params, "from")?;
    let to = timestamp(&mut params, "to")?;
    let timezone = params.remove("timezone").unwrap_or_else(|| "UTC".into());
    let csv = match params.remove("format").as_deref() {
        None | Some("json") => false,
        Some("csv")
            if matches!(
                view,
                UsageAnalyticsView::Users
                    | UsageAnalyticsView::Consumption
                    | UsageAnalyticsView::Breakdown
            ) =>
        {
            true
        }
        _ => return Err("format is not supported for this report".into()),
    };
    let limit = number(&mut params, "limit", 25_u32)?;
    let offset = number(&mut params, "offset", 0_u64)?;
    let payment_limit = params
        .remove("payment_limit")
        .map(|value| {
            value
                .parse::<u32>()
                .map_err(|_| "invalid payment_limit".to_string())
        })
        .transpose()?;
    let payment_offset = params
        .remove("payment_offset")
        .map(|value| {
            value
                .parse::<u64>()
                .map_err(|_| "invalid payment_offset".to_string())
        })
        .transpose()?;
    if limit == 0 || limit > 100 {
        return Err("limit must be between 1 and 100".into());
    }
    if csv && offset != 0 {
        return Err("CSV exports apply to the complete filter; offset must be zero".into());
    }
    let granularity = match params.remove("granularity").as_deref() {
        None | Some("day") => UsageAnalyticsGranularity::Day,
        Some("hour") => UsageAnalyticsGranularity::Hour,
        _ => return Err("granularity must be hour or day".into()),
    };
    if granularity == UsageAnalyticsGranularity::Hour && to.saturating_sub(from) > 31 * 86_400_000 {
        return Err("hourly reports are limited to 31 days".into());
    }
    let group_by = match params.remove("group_by").as_deref() {
        None | Some("model") => UsageAnalyticsGroupBy::Model,
        Some("provider") => UsageAnalyticsGroupBy::Provider,
        Some("api_key") => UsageAnalyticsGroupBy::ApiKey,
        Some("attribution") => UsageAnalyticsGroupBy::Attribution,
        Some("api_format") => UsageAnalyticsGroupBy::ApiFormat,
        Some("request_type") => UsageAnalyticsGroupBy::RequestType,
        _ => return Err("unsupported group_by dimension".into()),
    };
    let sort = match params.remove("sort").as_deref() {
        None | Some("requests") | Some("request_count") => UsageAnalyticsSort::Requests,
        Some("billable_amount") => UsageAnalyticsSort::BillableAmount,
        Some("total_tokens") => UsageAnalyticsSort::Tokens,
        Some("active_days") => UsageAnalyticsSort::ActiveDays,
        Some("started_at") => UsageAnalyticsSort::StartedAt,
        Some("last_used") | Some("last_used_at") => UsageAnalyticsSort::LastUsed,
        Some("username") => UsageAnalyticsSort::Username,
        _ => return Err("unsupported sort field".into()),
    };
    let descending = match params.remove("order").as_deref() {
        None | Some("desc") => true,
        Some("asc") => false,
        _ => return Err("order must be asc or desc".into()),
    };
    let user_is_active = match params.remove("account_status").as_deref() {
        None | Some("all") => None,
        Some("active") | Some("enabled") => Some(true),
        Some("inactive") | Some("disabled") => Some(false),
        _ => return Err("unsupported account_status".into()),
    };
    let has_usage = match params.remove("usage_status").as_deref() {
        None | Some("all") => None,
        Some("used") | Some("active") => Some(true),
        Some("unused") | Some("inactive") => Some(false),
        _ => return Err("unsupported usage_status".into()),
    };
    if view != UsageAnalyticsView::Users && (user_is_active.is_some() || has_usage.is_some()) {
        return Err(
            "account_status and usage_status are only supported by employee reports".into(),
        );
    }
    let amount_basis = params
        .remove("amount_basis")
        .unwrap_or_else(|| "billable".into());
    if !matches!(
        amount_basis.as_str(),
        "rated" | "billable" | "quota_covered" | "wallet_consumed" | "wallet_debit"
    ) {
        return Err("unsupported amount_basis".into());
    }
    let user_id = text(&mut params, "user_id")?;
    let attribution_kind = text(&mut params, "attribution_kind")?;
    let explicit_owner = text(&mut params, "credential_owner_id")?;
    if user_id.is_some() && explicit_owner.is_some() {
        return Err("user_id and credential_owner_id cannot be combined".into());
    }
    let member_account = attribution_kind.as_deref() == Some("employee");
    let status = match text(&mut params, "status")?.as_deref() {
        None => None,
        Some("success" | "completed") => Some("completed".into()),
        Some(value @ ("failed" | "cancelled" | "pending" | "streaming")) => Some(value.to_string()),
        _ => return Err("unsupported request status".into()),
    };
    let search = text(&mut params, "search")?;
    if search.is_some() && view != UsageAnalyticsView::Users {
        return Err("search is only supported by employee reports".into());
    }
    let slow_threshold_ms = params
        .remove("slow_threshold_ms")
        .map(|value| {
            value
                .parse::<u64>()
                .map_err(|_| "invalid slow_threshold_ms".to_string())
        })
        .transpose()?;
    if slow_threshold_ms.is_some_and(|value| value == 0 || value > 86_400_000) {
        return Err("slow_threshold_ms must be between 1 and 86400000".into());
    }
    let query = UsageAnalyticsQuery {
        from_unix_ms: from,
        to_unix_ms: to,
        timezone,
        view,
        group_by,
        granularity,
        actor_user_id: member_account.then(|| user_id.clone()).flatten(),
        credential_owner_id: if member_account {
            explicit_owner
        } else {
            user_id.or(explicit_owner)
        },
        attribution_kind,
        api_key_id: text(&mut params, "api_key_id")?,
        model: text(&mut params, "model")?,
        provider_id: text(&mut params, "provider_id")?,
        api_format: text(&mut params, "api_format")?,
        endpoint_kind: text(&mut params, "endpoint_kind")?,
        request_type: text(&mut params, "request_type")?,
        status,
        is_stream: boolean(&mut params, "is_stream")?,
        has_format_conversion: boolean(&mut params, "has_format_conversion")?,
        slow_threshold_ms,
        search,
        user_is_active,
        has_usage,
        sort,
        descending,
        limit: if csv {
            OVERVIEW_EXPORT_LIMIT + 1
        } else {
            limit
        },
        offset,
        payment_limit,
        payment_offset,
    };
    if let Some(key) = params.keys().next() {
        return Err(format!("unsupported query parameter: {key}"));
    }
    query.validate().map_err(|err| err.to_string())?;
    Ok(OverviewRequest {
        query,
        csv,
        amount_basis,
    })
}

fn timestamp(params: &mut BTreeMap<String, String>, key: &str) -> Result<u64, String> {
    let value = params
        .remove(key)
        .ok_or_else(|| format!("{key} is required"))?;
    let value = DateTime::parse_from_rfc3339(&value)
        .map_err(|_| format!("{key} must be an RFC 3339 timestamp"))?
        .with_timezone(&Utc);
    u64::try_from(value.timestamp_millis())
        .map_err(|_| format!("{key} must not precede the Unix epoch"))
}

fn number<T: std::str::FromStr>(
    params: &mut BTreeMap<String, String>,
    key: &str,
    default: T,
) -> Result<T, String> {
    params
        .remove(key)
        .map(|value| value.parse().map_err(|_| format!("invalid {key}")))
        .unwrap_or(Ok(default))
}

fn boolean(params: &mut BTreeMap<String, String>, key: &str) -> Result<Option<bool>, String> {
    match params.remove(key).as_deref() {
        None => Ok(None),
        Some("true") => Ok(Some(true)),
        Some("false") => Ok(Some(false)),
        _ => Err(format!("{key} must be true or false")),
    }
}

fn text(params: &mut BTreeMap<String, String>, key: &str) -> Result<Option<String>, String> {
    params
        .remove(key)
        .map(|value| {
            let value = value.trim();
            if value.is_empty() || value.len() > 512 || value.chars().any(char::is_control) {
                Err(format!("invalid {key}"))
            } else {
                Ok(value.to_string())
            }
        })
        .transpose()
}

#[cfg(test)]
mod tests {
    use super::*;

    const RANGE: &str =
        "from=2026-09-01T23:45:00Z&to=2026-09-02T00:15:00Z&timezone=Asia%2FShanghai";

    #[test]
    fn preserves_precise_cross_midnight_bounds() {
        let parsed = parse_overview_query(Some(RANGE), UsageAnalyticsView::Summary).unwrap();
        assert_eq!(
            parsed.query.to_unix_ms - parsed.query.from_unix_ms,
            30 * 60 * 1000
        );
        assert_eq!(parsed.query.timezone, "Asia/Shanghai");
    }

    #[test]
    fn rejects_unknown_duplicate_and_invalid_ranges() {
        for suffix in [
            "&model=x&model=y",
            "&invented=1",
            "&limit=101",
            "&timezone=Europe/Paris",
            "&is_stream=perhaps",
        ] {
            assert!(parse_overview_query(
                Some(&format!("{RANGE}{suffix}")),
                UsageAnalyticsView::Summary
            )
            .is_err());
        }
        assert!(parse_overview_query(
            Some("from=2026-09-01T00:00:00Z&to=2026-09-01T00:00:00Z"),
            UsageAnalyticsView::Summary
        )
        .is_err());
    }

    #[test]
    fn supports_iana_zone_across_dst_and_full_filter_exports() {
        let parsed = parse_overview_query(Some("from=2026-03-08T05:00:00Z&to=2026-03-09T04:00:00Z&timezone=America%2FNew_York&format=csv&search=alice"), UsageAnalyticsView::Users).unwrap();
        assert_eq!(
            parsed.query.to_unix_ms - parsed.query.from_unix_ms,
            23 * 3_600_000
        );
        assert_eq!(parsed.query.limit, OVERVIEW_EXPORT_LIMIT + 1);
        assert_eq!(parsed.query.search.as_deref(), Some("alice"));
    }

    #[test]
    fn user_payments_have_independent_bounded_pagination() {
        let raw = format!("{RANGE}&payment_limit=10&payment_offset=20&limit=1&offset=0");
        let parsed = parse_overview_query(Some(&raw), UsageAnalyticsView::Users).unwrap();
        assert_eq!(parsed.query.payment_limit, Some(10));
        assert_eq!(parsed.query.payment_offset, Some(20));
        assert_eq!(parsed.query.limit, 1);
        assert_eq!(parsed.query.offset, 0);
        assert!(parse_overview_query(Some(&raw), UsageAnalyticsView::Summary).is_err());
        for value in ["0", "101", "-1", "garbage"] {
            assert!(parse_overview_query(
                Some(&format!("{RANGE}&payment_limit={value}")),
                UsageAnalyticsView::Users
            )
            .is_err());
        }
    }
}

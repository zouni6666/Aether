use super::OverviewRequest;
use aether_data_contracts::repository::usage::{
    StoredUsageAnalytics, UsageAnalyticsConsumption, UsageAnalyticsMetrics,
    UsageAnalyticsUserFinance, UsageAnalyticsUserPayments, UsageAnalyticsView,
    USAGE_ANALYTICS_VERSION,
};
use chrono::{DateTime, Datelike, TimeZone, Utc};
use serde_json::{json, Value};

pub fn amount(value: &Option<String>, basis: &str, complete: bool) -> Value {
    json!({
        "value": value, "currency": "USD", "basis": basis,
        "status": if value.is_none() { "unknown" } else if complete { "known" } else { "known_subtotal" },
    })
}

pub fn metrics_value(metrics: &UsageAnalyticsMetrics) -> Value {
    let terminal = metrics
        .successful_request_count
        .saturating_add(metrics.failed_request_count)
        .saturating_add(metrics.cancelled_request_count);
    let priced = metrics.pricing_available_count == metrics.request_count;
    let allocated = metrics.allocation_available_count == metrics.request_count;
    let tokens_available = metrics.request_count == 0 || metrics.usage_available_count > 0;
    let usage_source =
        if metrics.request_count == 0 || metrics.unknown_usage_count == metrics.request_count {
            "unknown"
        } else if metrics.reported_usage_count == metrics.request_count {
            "reported"
        } else if metrics.estimated_usage_count == metrics.request_count {
            "estimated"
        } else {
            "mixed"
        };
    json!({
        "request_count": metrics.request_count,
        "successful_request_count": metrics.successful_request_count,
        "failed_request_count": metrics.failed_request_count,
        "cancelled_request_count": metrics.cancelled_request_count,
        "in_flight_request_count": metrics.in_flight_request_count,
        "slow_request_count": metrics.slow_request_count,
        "unclassified_failure_count": metrics.failed_request_count.saturating_sub(metrics.classified_failure_count),
        "input_tokens": tokens_available.then_some(metrics.input_tokens),
        "output_tokens": tokens_available.then_some(metrics.output_tokens),
        "total_tokens": tokens_available.then_some(metrics.total_tokens),
        "usage_source": usage_source,
        "usage_source_counts": {
            "reported": metrics.reported_usage_count,
            "estimated": metrics.estimated_usage_count,
            "mixed": metrics.mixed_usage_count,
            "unknown": metrics.unknown_usage_count,
        },
        "usage_active_users": metrics.usage_active_users,
        "enabled_users": metrics.enabled_users,
        "success_rate": {
            "value": (terminal > 0).then(|| metrics.successful_request_count as f64 / terminal as f64),
            "numerator": metrics.successful_request_count, "denominator": terminal,
        },
        "latency_ms": {
            "avg": (metrics.latency_sample_count > 0).then(|| metrics.latency_sum_ms / metrics.latency_sample_count as f64),
            "p50": metrics.latency_p50_ms, "p95": metrics.latency_p95_ms, "p99": metrics.latency_p99_ms,
            "sample_count": metrics.latency_sample_count,
        },
        "rated_amount": amount(&metrics.rated_amount, "rated", priced),
        "billable_amount": amount(&metrics.billable_amount, "billable", priced),
        "quota_covered_amount": amount(&metrics.quota_covered_amount, "quota_covered", allocated),
        "wallet_consumed_amount": amount(&metrics.wallet_consumed_amount, "wallet_consumed", allocated),
        "wallet_debit_amount": amount(&metrics.wallet_debit_amount, "wallet_debit", allocated),
        "wallet_recharge_debit_amount": amount(&metrics.wallet_recharge_debit_amount, "wallet_recharge_debit", allocated),
        "wallet_gift_debit_amount": amount(&metrics.wallet_gift_debit_amount, "wallet_gift_debit", allocated),
        "wallet_overdraft_amount": amount(&metrics.wallet_overdraft_amount, "wallet_overdraft", allocated),
    })
}

pub fn envelope(request: &OverviewRequest, snapshot: &StoredUsageAnalytics, data: Value) -> Value {
    let query = &request.query;
    let metrics = &snapshot.summary;
    let complete = snapshot.unrecoverable_bucket_count == 0
        && metrics.usage_available_count == metrics.request_count
        && metrics.pricing_available_count == metrics.request_count
        && metrics.settled_count == metrics.request_count
        && metrics.allocation_available_count == metrics.request_count;
    json!({
        "meta": {
            "schema_version": 1, "metric_version": USAGE_ANALYTICS_VERSION,
            "scope": if let Some(id) = &query.actor_user_id {
                json!({"kind": "employee", "user_id": id})
            } else if let Some(id) = &query.credential_owner_id {
                json!({"kind": "credential_owner", "user_id": id})
            } else { json!({"kind": "installation"}) },
            "range": {
                "from": rfc3339(query.from_unix_ms), "to": rfc3339(query.to_unix_ms),
                "timezone": query.timezone, "time_basis": "request_started_at",
            },
            "generated_at": snapshot.generated_at, "data_through": snapshot.data_through,
            "read_revision": snapshot.read_revision,
            "projection": snapshot.coverage,
            "amount_basis": request.amount_basis,
            "coverage": {
                "status": if complete { "complete" } else { "partial" },
                "request_count": metrics.request_count,
                "usage_available_count": metrics.usage_available_count,
                "pricing_available_count": metrics.pricing_available_count,
                "settled_count": metrics.settled_count,
                "allocation_available_count": metrics.allocation_available_count,
                "attribution_available_count": metrics.trusted_attribution_count,
                "classified_failure_count": metrics.classified_failure_count,
                "unrecoverable_bucket_count": snapshot.unrecoverable_bucket_count,
            },
        },
        "data": data,
    })
}

pub fn page_value(request: &OverviewRequest, snapshot: &StoredUsageAnalytics) -> Value {
    let items: Vec<Value> = match request.query.view {
        UsageAnalyticsView::Users => snapshot
            .users
            .iter()
            .map(|row| {
                let mut value = metrics_value(&row.metrics);
                value["user_id"] = json!(row.user_id);
                value["username"] = json!(row.username);
                value["email"] = json!(row.email);
                value["is_active"] = json!(row.is_active);
                value["last_used_at"] = json!(row.last_used_at);
                value["active_days"] = json!(row.active_days);
                value["finance"] = user_finance_value(row.finance.as_ref());
                value
            })
            .collect(),
        UsageAnalyticsView::Consumption => {
            snapshot.consumption.iter().map(consumption_value).collect()
        }
        _ => snapshot
            .rows
            .iter()
            .map(|row| {
                let mut value = metrics_value(&row.metrics);
                value["id"] = json!(row.id);
                value["label"] = json!(row.label);
                value["bucket_start"] = json!(row.bucket_start);
                value
            })
            .collect(),
    };
    let mut page = json!({ "items": items, "total": snapshot.total, "limit": request.query.limit, "offset": request.query.offset });
    if request.query.view == UsageAnalyticsView::Users {
        page["summary"] = snapshot
            .user_summary
            .as_ref()
            .map(|summary| {
                let mut value = metrics_value(&summary.metrics);
                value["user_count"] = json!(summary.user_count);
                value["active_user_count"] = json!(summary.active_user_count);
                value
            })
            .unwrap_or(Value::Null);
        page["finance_summary"] = user_finance_value(snapshot.user_finance_summary.as_ref());
    }
    page
}

pub fn user_finance_value(finance: Option<&UsageAnalyticsUserFinance>) -> Value {
    let Some(finance) = finance else {
        return Value::Null;
    };
    json!({
        "wallet_balance": amount(&finance.wallet_balance, "wallet_balance", true),
        "recharge_balance": amount(&finance.recharge_balance, "recharge_balance", true),
        "gift_balance": amount(&finance.gift_balance, "gift_balance", true),
        "recharge_amount": amount(&finance.recharge_amount, "credited_wallet_recharge", true),
        "recharge_count": finance.recharge_count,
        "plan_purchase_amount": amount(&finance.plan_purchase_amount, "credited_plan_purchase", true),
        "plan_purchase_count": finance.plan_purchase_count,
        "gift_credit_amount": amount(&finance.gift_credit_amount, "credited_gift_order", true),
        "gift_credit_count": finance.gift_credit_count,
        "balance_time_basis": "current",
        "payment_time_basis": "credited_at",
    })
}

pub fn user_payments_value(payments: Option<&UsageAnalyticsUserPayments>) -> Value {
    let Some(payments) = payments else {
        return Value::Null;
    };
    let items: Vec<Value> = payments
        .items
        .iter()
        .map(|payment| {
            json!({
                "id": payment.id, "order_no": payment.order_no, "kind": payment.kind,
                "amount": amount(&Some(payment.amount.clone()), "credited_order", true),
                "payment_method": payment.payment_method, "credited_at": payment.credited_at,
            })
        })
        .collect();
    json!({ "items": items, "total": payments.total, "limit": payments.limit, "offset": payments.offset })
}

pub fn consumption_value(row: &UsageAnalyticsConsumption) -> Value {
    json!({
        "id": row.id, "request_id": row.request_id, "started_at": row.started_at,
        "user_id": row.user_id, "credential_owner_id": row.credential_owner_id,
        "model": row.model, "provider": row.provider, "provider_id": row.provider_id,
        "api_key_id": row.api_key_id,
        "status": row.status, "settlement_status": row.settlement_status,
        "attribution_kind": row.attribution_kind, "attribution_source": row.attribution_source,
        "rated_amount": amount(&row.rated_amount, "rated", true),
        "billable_amount": amount(&row.billable_amount, "billable", true),
        "quota_covered_amount": amount(&row.quota_covered_amount, "quota_covered", true),
        "wallet_consumed_amount": amount(&row.wallet_consumed_amount, "wallet_consumed", true),
        "wallet_debit_amount": amount(&row.wallet_debit_amount, "wallet_debit", true),
    })
}

pub fn costs_value(request: &OverviewRequest, snapshot: &StoredUsageAnalytics) -> Value {
    let metrics = &snapshot.summary;
    let cache_complete = metrics.cache_pricing_available_count == metrics.request_count;
    let savings = metrics
        .cache_estimated_full_cost_amount
        .as_deref()
        .and_then(decimal_units)
        .zip(
            metrics
                .cache_read_cost_amount
                .as_deref()
                .and_then(decimal_units),
        )
        .and_then(|(full, read)| full.checked_sub(read))
        .map(format_units);
    json!({
        "summary": metrics_value(metrics),
        "timeseries": snapshot.rows.iter().map(|row| {
            let mut value = metrics_value(&row.metrics);
            value["bucket_start"] = json!(row.bucket_start);
            value
        }).collect::<Vec<_>>(),
        "supplier_estimated_cost": amount(&None, "supplier_estimated", false),
        "supplier_verified_cost": amount(&None, "supplier_verified", false),
        "cache": {
            "read_tokens": (metrics.usage_available_count > 0 || metrics.request_count == 0).then_some(metrics.cache_read_input_tokens),
            "creation_tokens": (metrics.usage_available_count > 0 || metrics.request_count == 0).then_some(metrics.cache_creation_input_tokens),
            "read_cost": amount(&metrics.cache_read_cost_amount, "cache_read", cache_complete),
            "creation_cost": amount(&metrics.cache_creation_cost_amount, "cache_creation", cache_complete),
            "estimated_full_cost": estimated_amount(&metrics.cache_estimated_full_cost_amount, "cache_full_price_estimate", cache_complete),
            "estimated_savings": estimated_amount(&savings, "cache_read_savings_estimate", cache_complete),
            "pricing_available_count": metrics.cache_pricing_available_count,
            "request_count": metrics.request_count,
        },
        "forecast": forecast(request, snapshot),
    })
}

fn estimated_amount(value: &Option<String>, basis: &str, complete: bool) -> Value {
    let mut result = amount(value, basis, complete);
    if value.is_some() {
        result["status"] = json!(if complete {
            "estimated"
        } else {
            "estimated_subtotal"
        });
    }
    result
}

fn performance_metrics(metrics: &UsageAnalyticsMetrics) -> Value {
    let mut value = json!({
        "request_count": metrics.request_count,
        "success_count": metrics.successful_request_count,
        "error_count": metrics.failed_request_count,
        "success_rate": metrics_value(metrics)["success_rate"]["value"],
        "output_tokens": metrics.output_tokens,
        "avg_output_tps": (metrics.output_tps_sample_count > 0).then(|| metrics.output_tps_sum / metrics.output_tps_sample_count as f64),
        "avg_first_byte_time_ms": (metrics.first_byte_sample_count > 0).then(|| metrics.first_byte_sum_ms / metrics.first_byte_sample_count as f64),
        "avg_response_time_ms": (metrics.latency_sample_count > 0).then(|| metrics.latency_sum_ms / metrics.latency_sample_count as f64),
        "p90_response_time_ms": metrics.latency_p90_ms,
        "p99_response_time_ms": metrics.latency_p99_ms,
        "p90_first_byte_time_ms": metrics.first_byte_p90_ms,
        "p99_first_byte_time_ms": metrics.first_byte_p99_ms,
        "tps_sample_count": metrics.output_tps_sample_count,
        "response_time_sample_count": metrics.latency_sample_count,
        "first_byte_sample_count": metrics.first_byte_sample_count,
        "slow_request_count": metrics.slow_request_count,
    });
    // The legacy provider chart contract expresses rates as percentages.
    value["success_rate"] = value["success_rate"]
        .as_f64()
        .map(|rate| json!(rate * 100.0))
        .unwrap_or(Value::Null);
    value
}

pub fn performance_value(request: &OverviewRequest, snapshot: &StoredUsageAnalytics) -> Value {
    let models = snapshot
        .model_rows
        .iter()
        .map(|row| {
            let mut value = performance_metrics(&row.metrics);
            value["model"] = json!(row.id);
            value
        })
        .collect::<Vec<_>>();
    let providers = snapshot
        .provider_rows
        .iter()
        .map(|row| {
            let mut value = performance_metrics(&row.metrics);
            value["provider_id"] = json!(row.id);
            value["provider"] = json!(row.label);
            value
        })
        .collect::<Vec<_>>();
    let timeline = snapshot
        .provider_timeline_rows
        .iter()
        .map(|row| {
            let mut value = performance_metrics(&row.metrics);
            value["provider_id"] = json!(row.id);
            value["provider"] = json!(row.label);
            value["date"] = json!(row.bucket_start);
            value
        })
        .collect::<Vec<_>>();
    json!({
        "summary": metrics_value(&snapshot.summary),
        "timeseries": page_value(request, snapshot)["items"],
        "providers": { "summary": performance_metrics(&snapshot.summary), "providers": providers, "timeline": timeline },
        "models": models,
        "errors": snapshot.errors,
    })
}

fn forecast(request: &OverviewRequest, snapshot: &StoredUsageAnalytics) -> Value {
    let unavailable = |days| {
        json!({
            "amount": amount(&None, "billable_forecast", false), "method": "calendar_month_daily_average",
            "status": "insufficient_data", "sample_days": days, "period_end": null,
        })
    };
    if snapshot.unrecoverable_bucket_count > 0 {
        return unavailable(0);
    }
    let Ok(zone) = request.query.timezone.parse::<chrono_tz::Tz>() else {
        return unavailable(0);
    };
    let Some(end) = DateTime::<Utc>::from_timestamp_millis(request.query.to_unix_ms as i64) else {
        return unavailable(0);
    };
    // Select the forecast month from the last included instant in this half-open range.
    let local_end = (end - chrono::Duration::milliseconds(1)).with_timezone(&zone);
    let month = local_end
        .date_naive()
        .with_day(1)
        .expect("first day exists");
    let Some(next_month) = month.checked_add_months(chrono::Months::new(1)) else {
        return unavailable(0);
    };
    let mut sum = 0_i128;
    let mut days = std::collections::BTreeSet::new();
    for row in &snapshot.rows {
        let Some(start) = row
            .bucket_start
            .as_deref()
            .and_then(|value| DateTime::parse_from_rfc3339(value).ok())
        else {
            continue;
        };
        let date = start.with_timezone(&zone).date_naive();
        let Some(next) = date.succ_opt().and_then(|day| {
            zone.from_local_datetime(&day.and_hms_opt(0, 0, 0)?)
                .earliest()
        }) else {
            continue;
        };
        if date < month
            || date >= next_month
            || start.timestamp_millis() < request.query.from_unix_ms as i64
            || next.with_timezone(&Utc) > end
            || next.with_timezone(&Utc) > Utc::now()
        {
            continue;
        }
        if row.metrics.pricing_available_count != row.metrics.request_count
            || row.metrics.settled_count != row.metrics.request_count
        {
            return unavailable(days.len());
        }
        let Some(value) = row
            .metrics
            .billable_amount
            .as_deref()
            .and_then(decimal_units)
        else {
            return unavailable(days.len());
        };
        let Some(total) = sum.checked_add(value) else {
            return unavailable(days.len());
        };
        sum = total;
        days.insert(date);
    }
    // Missing calendar days are not assumed to be zero-use days.
    if days.len() < 7
        || days
            .last()
            .zip(days.first())
            .is_none_or(|(last, first)| (*last - *first).num_days() + 1 != days.len() as i64)
    {
        return unavailable(days.len());
    }
    let Some(projected) = sum
        .checked_mul((next_month - month).num_days() as i128)
        .map(|value| value / days.len() as i128)
    else {
        return unavailable(days.len());
    };
    json!({
        "amount": {"value": format_units(projected), "currency": "USD", "basis": "billable_forecast", "status": "estimated"},
        "method": "calendar_month_daily_average", "status": "estimated", "sample_days": days.len(),
        "period_end": zone.from_local_datetime(&next_month.and_hms_opt(0, 0, 0).expect("midnight exists")).earliest().map(|value| value.to_rfc3339()),
    })
}

fn decimal_units(value: &str) -> Option<i128> {
    let (whole, fraction) = value.split_once('.').unwrap_or((value, ""));
    if fraction.len() > 8 || !fraction.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    let negative = whole.starts_with('-');
    let whole = whole.parse::<i128>().ok()?;
    let fractional = if fraction.is_empty() {
        0
    } else {
        fraction.parse::<i128>().ok()? * 10_i128.pow(8 - fraction.len() as u32)
    };
    whole
        .checked_mul(100_000_000)?
        .checked_add(if negative { -fractional } else { fractional })
}

fn format_units(value: i128) -> String {
    format!(
        "{}{}.{:08}",
        if value < 0 { "-" } else { "" },
        value.unsigned_abs() / 100_000_000,
        value.unsigned_abs() % 100_000_000
    )
}

fn rfc3339(millis: u64) -> Option<String> {
    DateTime::<Utc>::from_timestamp_millis(millis as i64).map(|value| value.to_rfc3339())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_finance_preserves_unknown_sources_and_distinct_payment_bases() {
        assert!(user_finance_value(None).is_null());
        assert!(user_payments_value(None).is_null());
        let finance = UsageAnalyticsUserFinance {
            wallet_balance: Some("15.00000000".into()),
            recharge_amount: Some("100.00000000".into()),
            recharge_count: 2,
            plan_purchase_amount: Some("25.00000000".into()),
            plan_purchase_count: 1,
            gift_credit_amount: Some("7.00000000".into()),
            gift_credit_count: 1,
            ..Default::default()
        };
        let value = user_finance_value(Some(&finance));
        assert_eq!(value["wallet_balance"]["value"], "15.00000000");
        assert_eq!(value["recharge_amount"]["value"], "100.00000000");
        assert_eq!(
            value["plan_purchase_amount"]["basis"],
            "credited_plan_purchase"
        );
        assert_eq!(value["gift_credit_amount"]["basis"], "credited_gift_order");
        assert_eq!(value["gift_balance"]["status"], "unknown");
        assert_eq!(value["balance_time_basis"], "current");
        assert_eq!(value["payment_time_basis"], "credited_at");
    }

    #[test]
    fn unknown_money_and_zero_denominators_stay_unknown() {
        let metrics = UsageAnalyticsMetrics {
            request_count: 3,
            in_flight_request_count: 3,
            ..Default::default()
        };
        let value = metrics_value(&metrics);
        assert!(value["success_rate"]["value"].is_null());
        assert!(value["latency_ms"]["avg"].is_null());
        assert!(value["total_tokens"].is_null());
        assert!(value["billable_amount"]["value"].is_null());
        assert_eq!(value["billable_amount"]["status"], "unknown");
    }

    #[test]
    fn known_subtotal_and_cancelled_consumption_remain_visible() {
        let metrics = UsageAnalyticsMetrics {
            request_count: 4,
            successful_request_count: 2,
            cancelled_request_count: 1,
            in_flight_request_count: 1,
            pricing_available_count: 2,
            billable_amount: Some("1.25000000".into()),
            ..Default::default()
        };
        let value = metrics_value(&metrics);
        assert_eq!(value["success_rate"]["denominator"], 3);
        assert_eq!(value["billable_amount"]["status"], "known_subtotal");
        assert_eq!(value["billable_amount"]["value"], "1.25000000");
    }

    #[test]
    fn overview_model_performance_preserves_unknowns_and_uses_sample_weighted_averages() {
        use aether_data_contracts::repository::usage::UsageAnalyticsRow;
        let request = super::super::parse_overview_query(
            Some("from=2026-09-01T00:00:00Z&to=2026-09-02T00:00:00Z"),
            UsageAnalyticsView::Performance,
        )
        .unwrap();
        let snapshot = StoredUsageAnalytics {
            model_rows: vec![
                UsageAnalyticsRow {
                    id: Some("requested-model".into()),
                    label: Some("requested-model".into()),
                    bucket_start: None,
                    metrics: UsageAnalyticsMetrics {
                        request_count: 6,
                        successful_request_count: 3,
                        failed_request_count: 1,
                        cancelled_request_count: 1,
                        in_flight_request_count: 1,
                        first_byte_sample_count: 3,
                        first_byte_sum_ms: 900.0,
                        output_tps_sample_count: 3,
                        output_tps_sum: 500.0,
                        latency_sample_count: 3,
                        latency_sum_ms: 4900.0,
                        ..Default::default()
                    },
                },
                UsageAnalyticsRow {
                    id: None,
                    label: None,
                    bucket_start: None,
                    metrics: UsageAnalyticsMetrics {
                        request_count: 1,
                        in_flight_request_count: 1,
                        ..Default::default()
                    },
                },
            ],
            ..Default::default()
        };
        let value = performance_value(&request, &snapshot);
        let rows = value["models"].as_array().unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0]["model"], "requested-model");
        assert_eq!(rows[0]["success_rate"], 60.0);
        assert_eq!(rows[0]["avg_first_byte_time_ms"], 300.0);
        assert_eq!(rows[0]["avg_output_tps"], 500.0 / 3.0);
        assert_eq!(rows[0]["avg_response_time_ms"], 4900.0 / 3.0);
        for field in [
            "model",
            "success_rate",
            "avg_first_byte_time_ms",
            "avg_output_tps",
            "avg_response_time_ms",
        ] {
            assert!(rows[1][field].is_null(), "{field}");
        }
        assert!(value["providers"]["providers"].is_array());
        assert!(value["timeseries"].is_array());
    }

    #[test]
    fn forecast_arithmetic_keeps_eight_decimal_places_without_floats() {
        for value in [
            "12.34567890",
            "0.00000001",
            "-0.10000000",
            "999999999999.99999999",
        ] {
            assert_eq!(format_units(decimal_units(value).unwrap()), value);
        }
    }

    #[test]
    fn forecast_includes_complete_calendar_month_at_exclusive_local_boundary() {
        use aether_data_contracts::repository::usage::UsageAnalyticsRow;

        for timezone in ["UTC", "Asia/Shanghai", "America/New_York"] {
            let zone = timezone.parse::<chrono_tz::Tz>().unwrap();
            let from = zone.with_ymd_and_hms(2020, 3, 1, 0, 0, 0).unwrap();
            let to = zone.with_ymd_and_hms(2020, 4, 1, 0, 0, 0).unwrap();
            let request = super::super::parse_overview_query(
                Some(&format!(
                    "from={}&to={}&timezone={timezone}",
                    from.with_timezone(&Utc)
                        .to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
                    to.with_timezone(&Utc)
                        .to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
                )),
                UsageAnalyticsView::Timeseries,
            )
            .unwrap();
            let snapshot = StoredUsageAnalytics {
                rows: (1..=31)
                    .map(|day| UsageAnalyticsRow {
                        id: None,
                        label: None,
                        bucket_start: Some(
                            zone.with_ymd_and_hms(2020, 3, day, 0, 0, 0)
                                .unwrap()
                                .to_rfc3339(),
                        ),
                        metrics: UsageAnalyticsMetrics {
                            request_count: 1,
                            pricing_available_count: 1,
                            settled_count: 1,
                            billable_amount: Some("1.25000000".into()),
                            ..Default::default()
                        },
                    })
                    .collect(),
                ..Default::default()
            };

            let value = costs_value(&request, &snapshot);
            assert_eq!(value["forecast"]["sample_days"], 31, "{timezone}");
            assert_eq!(value["forecast"]["status"], "estimated", "{timezone}");
            assert_eq!(
                value["forecast"]["amount"]["value"], "38.75000000",
                "{timezone}"
            );
            assert_eq!(
                value["forecast"]["period_end"],
                to.to_rfc3339(),
                "{timezone}"
            );
        }
    }

    #[test]
    fn cache_savings_use_known_prices_and_preserve_estimate_coverage() {
        let request = super::super::parse_overview_query(
            Some("from=2026-09-01T00:00:00Z&to=2026-09-02T00:00:00Z"),
            UsageAnalyticsView::Timeseries,
        )
        .unwrap();
        let mut snapshot = StoredUsageAnalytics::default();
        snapshot.summary.request_count = 2;
        snapshot.summary.cache_pricing_available_count = 1;
        snapshot.summary.cache_estimated_full_cost_amount = Some("0.10000001".into());
        snapshot.summary.cache_read_cost_amount = Some("0.02000000".into());
        let value = costs_value(&request, &snapshot);
        assert_eq!(value["cache"]["estimated_savings"]["value"], "0.08000001");
        assert_eq!(
            value["cache"]["estimated_savings"]["status"],
            "estimated_subtotal"
        );
        snapshot.summary.cache_estimated_full_cost_amount = None;
        let value = costs_value(&request, &snapshot);
        assert!(value["cache"]["estimated_savings"]["value"].is_null());
        snapshot.unrecoverable_bucket_count = 1;
        assert_eq!(
            envelope(&request, &snapshot, json!({}))["meta"]["coverage"]["status"],
            "partial"
        );
        assert_eq!(value["supplier_verified_cost"]["status"], "unknown");
    }
}

use super::{envelope, page_value, OverviewRequest, OVERVIEW_EXPORT_LIMIT};
use aether_data_contracts::repository::usage::{StoredUsageAnalytics, UsageAnalyticsView};
use serde_json::Value;

pub fn export_csv(
    request: &OverviewRequest,
    snapshot: &StoredUsageAnalytics,
) -> Result<String, String> {
    if snapshot.total > u64::from(OVERVIEW_EXPORT_LIMIT) {
        return Err(format!(
            "export exceeds {OVERVIEW_EXPORT_LIMIT} rows; narrow the report range or filters"
        ));
    }
    let page = page_value(request, snapshot);
    let items = page["items"].as_array().ok_or("invalid export data")?;
    if items.len() as u64 != snapshot.total {
        return Err("the complete export could not be read from one snapshot".into());
    }
    let columns: &[&str] = match request.query.view {
        UsageAnalyticsView::Users => &[
            "user_id",
            "username",
            "email",
            "is_active",
            "last_used_at",
            "active_days",
            "request_count",
            "successful_request_count",
            "failed_request_count",
            "total_tokens",
            "billable_amount.value",
            "billable_amount.status",
            "quota_covered_amount.value",
            "wallet_consumed_amount.value",
            "wallet_debit_amount.value",
            "finance.wallet_balance.value",
            "finance.wallet_balance.status",
            "finance.recharge_balance.value",
            "finance.gift_balance.value",
            "finance.recharge_amount.value",
            "finance.recharge_count",
            "finance.plan_purchase_amount.value",
            "finance.plan_purchase_count",
            "finance.gift_credit_amount.value",
            "finance.gift_credit_count",
            "finance.balance_time_basis",
            "finance.payment_time_basis",
        ],
        UsageAnalyticsView::Consumption => &[
            "id",
            "request_id",
            "started_at",
            "user_id",
            "credential_owner_id",
            "model",
            "provider",
            "status",
            "settlement_status",
            "attribution_kind",
            "attribution_source",
            "rated_amount.value",
            "billable_amount.value",
            "quota_covered_amount.value",
            "wallet_consumed_amount.value",
            "wallet_debit_amount.value",
        ],
        UsageAnalyticsView::Breakdown => &[
            "id",
            "label",
            "request_count",
            "successful_request_count",
            "failed_request_count",
            "total_tokens",
            "rated_amount.value",
            "billable_amount.value",
            "billable_amount.status",
            "quota_covered_amount.value",
            "wallet_consumed_amount.value",
            "wallet_debit_amount.value",
        ],
        _ => return Err("this report does not support CSV".into()),
    };
    let mut columns = columns.to_vec();
    columns.extend([
        "report.range.from",
        "report.range.to",
        "report.range.timezone",
        "report.scope.kind",
        "report.metric_version",
        "report.read_revision",
        "report.coverage.status",
        "report.coverage.unrecoverable_bucket_count",
        "billable_amount.currency",
        "rated_amount.status",
        "quota_covered_amount.status",
        "wallet_consumed_amount.status",
        "wallet_debit_amount.status",
    ]);
    let metadata = envelope(request, snapshot, Value::Null)["meta"].clone();
    let mut output = String::from("\u{feff}");
    output.push_str(&columns.join(","));
    output.push_str("\r\n");
    for row in items {
        let mut row = row.clone();
        row["report"] = metadata.clone();
        for (index, column) in columns.iter().enumerate() {
            if index > 0 {
                output.push(',');
            }
            let value = column.split('.').fold(&row, |value, field| &value[field]);
            let text = match value {
                Value::Null => String::new(),
                Value::String(value) => value.clone(),
                value => value.to_string(),
            };
            output.push_str(&escape(&text));
        }
        output.push_str("\r\n");
    }
    Ok(output)
}

fn escape(value: &str) -> String {
    let formula = value.trim_start().starts_with(['=', '+', '-', '@'])
        || value.starts_with(['\t', '\r', '\n']);
    format!(
        "\"{}{}\"",
        if formula { "'" } else { "" },
        value.replace('"', "\"\"")
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use aether_data_contracts::repository::usage::{
        UsageAnalyticsQuery, UsageAnalyticsUser, UsageAnalyticsUserFinance,
    };

    #[test]
    fn quotes_csv_and_prevents_user_fields_from_becoming_formulas() {
        assert_eq!(escape("=SUM(1,2)"), "\"'=SUM(1,2)\"");
        assert_eq!(escape("a\"b\nc"), "\"a\"\"b\nc\"");
        assert_eq!(escape("12.34567890"), "\"12.34567890\"");
    }

    #[test]
    fn user_export_keeps_consumption_credits_and_current_balances_separate() {
        let request = OverviewRequest {
            query: UsageAnalyticsQuery {
                view: UsageAnalyticsView::Users,
                ..Default::default()
            },
            csv: true,
            amount_basis: "billable".into(),
        };
        let snapshot = StoredUsageAnalytics {
            total: 1,
            users: vec![UsageAnalyticsUser {
                user_id: "member".into(),
                username: "Alice".into(),
                email: None,
                is_active: true,
                last_used_at: None,
                active_days: 1,
                metrics: aether_data_contracts::repository::usage::UsageAnalyticsMetrics {
                    billable_amount: Some("3.00000000".into()),
                    ..Default::default()
                },
                finance: Some(UsageAnalyticsUserFinance {
                    wallet_balance: Some("12.00000000".into()),
                    recharge_amount: Some("100.00000000".into()),
                    recharge_count: 1,
                    plan_purchase_amount: Some("25.00000000".into()),
                    plan_purchase_count: 1,
                    ..Default::default()
                }),
            }],
            ..Default::default()
        };
        let csv = export_csv(&request, &snapshot).unwrap();
        let mut lines = csv.trim_start_matches('\u{feff}').lines();
        let headers = lines.next().unwrap().split(',').collect::<Vec<_>>();
        let cells = lines.next().unwrap().split(',').collect::<Vec<_>>();
        let value = |column| {
            cells[headers
                .iter()
                .position(|candidate| *candidate == column)
                .unwrap()]
        };
        assert_eq!(value("billable_amount.value"), "\"3.00000000\"");
        assert_eq!(value("finance.wallet_balance.value"), "\"12.00000000\"");
        assert_eq!(value("finance.recharge_amount.value"), "\"100.00000000\"");
        assert_eq!(
            value("finance.plan_purchase_amount.value"),
            "\"25.00000000\""
        );
        assert_eq!(value("finance.balance_time_basis"), "\"current\"");
        assert_eq!(value("finance.payment_time_basis"), "\"credited_at\"");
    }
}

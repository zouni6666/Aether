use super::{
    build_admin_billing_bad_request_response as bad_request,
    build_admin_billing_conflict_response as conflict,
    build_admin_billing_data_unavailable_response as unavailable,
    build_admin_billing_not_found_response as not_found,
};
use crate::handlers::admin::{
    request::{AdminAppState, AdminRequestContext},
    shared::{attach_admin_audit_response, query_param_value},
};
use crate::handlers::shared::normalize_payment_currency;
use crate::GatewayError;
use aether_data_contracts::repository::billing::*;
use axum::{
    body::{Body, Bytes},
    http::{self, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ExpenseRequest {
    client_request_id: String,
    provider_id: String,
    kind: String,
    amount: String,
    currency: String,
    paid_at: String,
    period_start: Option<String>,
    period_end: Option<String>,
    note: Option<String>,
    external_reference: Option<String>,
}
fn datetime(value: u64) -> String {
    chrono::DateTime::from_timestamp_millis(value as i64)
        .expect("valid stored timestamp")
        .to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}
fn parse_date(value: &str) -> Result<u64, String> {
    chrono::DateTime::parse_from_rfc3339(value)
        .ok()
        .and_then(|v| u64::try_from(v.timestamp_millis()).ok())
        .filter(|v| *v <= 253_402_300_799_000)
        .ok_or_else(|| "timestamps must be RFC3339 dates on or after 1970".into())
}
fn optional_text(value: Option<String>) -> Option<String> {
    value.map(|v| v.trim().to_owned()).filter(|v| !v.is_empty())
}
fn expense_json(record: &ProviderExpenseRecord) -> Value {
    let e = &record.entry;
    json!({
        "id": record.id, "client_request_id": e.client_request_id,
        "provider_id": e.provider_id, "provider_name": e.provider_name,
        "kind": e.kind, "amount": e.amount, "currency": e.currency,
        "paid_at": datetime(e.paid_at_unix_ms),
        "period_start": e.period_start_unix_ms.map(datetime),
        "period_end": e.period_end_unix_ms.map(datetime),
        "note": e.note, "external_reference": e.external_reference,
        "created_by": e.created_by, "created_at": datetime(record.created_at_unix_ms),
        "status": if record.voided_at_unix_ms.is_some() { "void" } else { "recorded" },
        "voided_at": record.voided_at_unix_ms.map(datetime), "voided_by": record.voided_by,
    })
}
fn csv_cell(value: &str) -> String {
    let value = if value.trim_start().starts_with(['=', '+', '-', '@'])
        || value.starts_with(['\t', '\r', '\n'])
    {
        format!("'{value}")
    } else {
        value.to_string()
    };
    format!("\"{}\"", value.replace('"', "\"\""))
}
fn csv_report(items: &[ProviderExpenseRecord]) -> String {
    let mut result=String::from("\u{feff}id,provider_id,provider_name,kind,amount,currency,paid_at,period_start,period_end,note,external_reference,created_by,created_at\r\n");
    for r in items {
        let e = &r.entry;
        let fields = [
            r.id.clone(),
            e.provider_id.clone(),
            e.provider_name.clone(),
            e.kind.clone(),
            e.amount.clone(),
            e.currency.clone(),
            datetime(e.paid_at_unix_ms),
            e.period_start_unix_ms.map(datetime).unwrap_or_default(),
            e.period_end_unix_ms.map(datetime).unwrap_or_default(),
            e.note.clone().unwrap_or_default(),
            e.external_reference.clone().unwrap_or_default(),
            e.created_by.clone().unwrap_or_default(),
            datetime(r.created_at_unix_ms),
        ];
        result.push_str(
            &fields
                .iter()
                .map(|s| csv_cell(s))
                .collect::<Vec<_>>()
                .join(","),
        );
        result.push_str("\r\n");
    }
    result
}
fn query(context: &AdminRequestContext<'_>, csv: bool) -> Result<ProviderExpenseQuery, String> {
    let q = context.query_string();
    let now = chrono::Utc::now().timestamp_millis().max(0) as u64;
    let from = query_param_value(q, "from")
        .map(|v| parse_date(&v))
        .transpose()?
        .unwrap_or(now.saturating_sub(30 * 86_400_000));
    let to = query_param_value(q, "to")
        .map(|v| parse_date(&v))
        .transpose()?
        .unwrap_or(now);
    let limit = if csv {
        10_001
    } else {
        query_param_value(q, "limit")
            .map(|v| v.parse::<u32>().map_err(|_| "invalid limit".to_string()))
            .transpose()?
            .unwrap_or(25)
    };
    let offset = if csv {
        0
    } else {
        query_param_value(q, "offset")
            .map(|v| v.parse::<u64>().map_err(|_| "invalid offset".to_string()))
            .transpose()?
            .unwrap_or(0)
    };
    if !csv && limit > 200 {
        return Err("limit must be at most 200".into());
    }
    let q = ProviderExpenseQuery {
        from_unix_ms: from,
        to_unix_ms: to,
        limit,
        offset,
    };
    q.validate().map_err(|e| e.to_string())?;
    Ok(q)
}
pub(super) async fn response(
    state: &AdminAppState<'_>,
    context: &AdminRequestContext<'_>,
    body: Option<&Bytes>,
) -> Result<Option<Response<Body>>, GatewayError> {
    let path = context.path().trim_end_matches('/');
    if context.route_family() != Some("billing_manage")
        || !path.starts_with("/api/admin/billing/provider-expenses")
    {
        return Ok(None);
    }
    let operator = context
        .decision()
        .and_then(|d| d.admin_principal.as_ref())
        .map(|p| p.user_id.clone());
    if path == "/api/admin/billing/provider-expenses" && context.method() == http::Method::GET {
        let csv = query_param_value(context.query_string(), "format").as_deref() == Some("csv");
        let q = match query(context, csv) {
            Ok(v) => v,
            Err(e) => return Ok(Some(bad_request(e))),
        };
        let Some(page) = state
            .app()
            .data
            .list_provider_expenses(&q)
            .await
            .map_err(|e| GatewayError::Internal(e.to_string()))?
        else {
            return Ok(Some(unavailable()));
        };
        if csv {
            if page.total > 10_000 {
                return Ok(Some(
                    (
                        StatusCode::UNPROCESSABLE_ENTITY,
                        Json(json!({"detail":"导出超过 10000 条，请缩小时间范围"})),
                    )
                        .into_response(),
                ));
            }
            return Ok(Some(
                (
                    [
                        (http::header::CONTENT_TYPE, "text/csv; charset=utf-8"),
                        (
                            http::header::CONTENT_DISPOSITION,
                            "attachment; filename=provider-expenses.csv",
                        ),
                        (http::header::CACHE_CONTROL, "private, no-store"),
                    ],
                    csv_report(&page.items),
                )
                    .into_response(),
            ));
        }
        return Ok(Some(
            (
                [(http::header::CACHE_CONTROL, "private, no-store")],
                Json(json!({
                    "items": page.items.iter().map(expense_json).collect::<Vec<_>>(),
                    "total": page.total, "totals": page.totals, "providers": page.providers,
                    "limit": q.limit, "offset": q.offset,
                    "from": datetime(q.from_unix_ms), "to": datetime(q.to_unix_ms),
                    "time_basis": "paid_at", "source": "manual_ledger",
                })),
            )
                .into_response(),
        ));
    }
    if path == "/api/admin/billing/provider-expenses" && context.method() == http::Method::POST {
        let Some(body) = body else {
            return Ok(Some(bad_request("缺少请求体")));
        };
        let payload = match serde_json::from_slice::<ExpenseRequest>(body) {
            Ok(v) => v,
            Err(_) => return Ok(Some(bad_request("输入验证失败"))),
        };
        let input = (|| -> Result<ProviderExpenseInput, String> {
            let units = provider_expense_amount_units(&payload.amount)
                .ok_or("amount must be a positive decimal string with at most 8 decimal places")?;
            let input = ProviderExpenseInput {
                client_request_id: uuid::Uuid::parse_str(&payload.client_request_id)
                    .map_err(|_| "client_request_id must be a UUID")?
                    .to_string(),
                provider_id: payload.provider_id.trim().into(),
                provider_name: "pending".into(),
                kind: payload.kind,
                amount: format_provider_expense_amount(units),
                currency: normalize_payment_currency(&payload.currency, "currency")?,
                paid_at_unix_ms: parse_date(&payload.paid_at)?,
                period_start_unix_ms: payload
                    .period_start
                    .as_deref()
                    .map(parse_date)
                    .transpose()?,
                period_end_unix_ms: payload.period_end.as_deref().map(parse_date).transpose()?,
                note: optional_text(payload.note),
                external_reference: optional_text(payload.external_reference),
                created_by: operator.clone(),
            };
            input.validate()?;
            Ok(input)
        })();
        let mut input = match input {
            Ok(v) => v,
            Err(e) => return Ok(Some(bad_request(e))),
        };
        let providers = state
            .read_provider_catalog_providers_by_ids(&[input.provider_id.clone()])
            .await?;
        let Some(provider) = providers.first() else {
            return Ok(Some(not_found("Provider not found")));
        };
        input.provider_name = provider.name.clone();
        let result = state
            .app()
            .data
            .create_provider_expense(&input)
            .await
            .map_err(|e| GatewayError::Internal(e.to_string()))?;
        return Ok(Some(mutation_response(
            result,
            "admin_provider_expense_recorded",
            "record_provider_expense",
        )));
    }
    if context.method() == http::Method::POST {
        if let Some(id) = path
            .strip_prefix("/api/admin/billing/provider-expenses/")
            .and_then(|v| v.strip_suffix("/void"))
            .filter(|v| !v.is_empty() && !v.contains('/'))
        {
            if uuid::Uuid::parse_str(id).is_err() {
                return Ok(Some(bad_request("invalid expense id")));
            }
            let result = state
                .app()
                .data
                .void_provider_expense(id, operator.as_deref())
                .await
                .map_err(|e| GatewayError::Internal(e.to_string()))?;
            return Ok(Some(mutation_response(
                result,
                "admin_provider_expense_voided",
                "void_provider_expense",
            )));
        }
    }
    Ok(None)
}
fn mutation_response(
    outcome: AdminBillingMutationOutcome<ProviderExpenseRecord>,
    event: &'static str,
    action: &'static str,
) -> Response<Body> {
    match outcome {
        AdminBillingMutationOutcome::Applied(record) => attach_admin_audit_response(
            Json(json!({"item":expense_json(&record)})).into_response(),
            event,
            action,
            "provider_expense",
            &record.id,
        ),
        AdminBillingMutationOutcome::Invalid(e) => conflict(e),
        AdminBillingMutationOutcome::NotFound => not_found("Provider expense not found"),
        AdminBillingMutationOutcome::Unavailable => unavailable(),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn provider_expense_csv_neutralizes_formulas_and_quotes_fields() {
        assert_eq!(csv_cell("=cmd()"), "\"'=cmd()\"");
        assert_eq!(csv_cell("  @cmd"), "\"'  @cmd\"");
        assert_eq!(csv_cell("\tcmd"), "\"'\tcmd\"");
        assert_eq!(csv_cell("a,\"b\"\nc"), "\"a,\"\"b\"\"\nc\"");
        assert_eq!(csv_cell("12.34"), "\"12.34\"");
    }
    #[test]
    fn provider_expense_dates_require_explicit_timezone_and_nonnegative_epoch() {
        assert_eq!(
            parse_date("2026-09-20T08:00:00+08:00"),
            parse_date("2026-09-20T00:00:00Z")
        );
        assert!(parse_date("2026-09-20").is_err());
        assert!(parse_date("1969-01-01T00:00:00Z").is_err());
    }
}

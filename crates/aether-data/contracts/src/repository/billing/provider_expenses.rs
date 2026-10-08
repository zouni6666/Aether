//! An administrator-maintained purchasing ledger. Entries are not inferred from request prices.
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

const SCALE: u128 = 100_000_000;

pub fn provider_expense_amount_units(value: &str) -> Option<u128> {
    let (whole, fraction) = value.split_once('.').unwrap_or((value, ""));
    if whole.is_empty()
        || whole.len() > 12
        || !whole.bytes().all(|c| c.is_ascii_digit())
        || fraction.len() > 8
        || !fraction.bytes().all(|c| c.is_ascii_digit())
    {
        return None;
    }
    let units = whole
        .parse::<u128>()
        .ok()?
        .checked_mul(SCALE)?
        .checked_add(if fraction.is_empty() {
            0
        } else {
            fraction.parse::<u128>().ok()? * 10_u128.pow(8 - fraction.len() as u32)
        })?;
    (units > 0).then_some(units)
}

pub fn format_provider_expense_amount(units: u128) -> String {
    format!("{}.{:08}", units / SCALE, units % SCALE)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderExpenseInput {
    pub client_request_id: String,
    pub provider_id: String,
    pub provider_name: String,
    pub kind: String,
    pub amount: String,
    pub currency: String,
    pub paid_at_unix_ms: u64,
    pub period_start_unix_ms: Option<u64>,
    pub period_end_unix_ms: Option<u64>,
    pub note: Option<String>,
    pub external_reference: Option<String>,
    pub created_by: Option<String>,
}

impl ProviderExpenseInput {
    /// Provider display names are snapshots, so renames do not break a retry.
    pub fn same_request_as(&self, other: &Self) -> bool {
        self.client_request_id == other.client_request_id
            && self.provider_id == other.provider_id
            && self.kind == other.kind
            && provider_expense_amount_units(&self.amount)
                == provider_expense_amount_units(&other.amount)
            && self.currency == other.currency
            && self.paid_at_unix_ms == other.paid_at_unix_ms
            && self.period_start_unix_ms == other.period_start_unix_ms
            && self.period_end_unix_ms == other.period_end_unix_ms
            && self.note == other.note
            && self.external_reference == other.external_reference
            && self.created_by == other.created_by
    }
    pub fn validate(&self) -> Result<(), String> {
        if uuid::Uuid::parse_str(&self.client_request_id).is_err() {
            return Err("client_request_id must be a UUID".into());
        }
        if self.provider_id.is_empty()
            || self.provider_id.len() > 512
            || self.provider_name.is_empty()
            || self.provider_name.len() > 512
        {
            return Err("invalid provider identity".into());
        }
        if !matches!(self.kind.as_str(), "recharge" | "subscription" | "other") {
            return Err("kind must be recharge, subscription or other".into());
        }
        if provider_expense_amount_units(&self.amount).is_none() {
            return Err(
                "amount must be positive with at most 12 integer and 8 decimal digits".into(),
            );
        }
        if self.currency.len() != 3 || !self.currency.bytes().all(|c| c.is_ascii_uppercase()) {
            return Err("currency must be a 3-letter uppercase code".into());
        }
        if self.paid_at_unix_ms > 253_402_300_799_000
            || self
                .period_start_unix_ms
                .is_some_and(|v| v > 253_402_300_799_000)
            || self
                .period_end_unix_ms
                .is_some_and(|v| v > 253_402_300_799_000)
        {
            return Err("invalid timestamp".into());
        }
        match (self.period_start_unix_ms, self.period_end_unix_ms) {
            (None, None) => {}
            (Some(start), Some(end)) if start < end => {}
            _ => {
                return Err(
                    "coverage period must contain both start and end, with start before end".into(),
                )
            }
        }
        if self.note.as_ref().is_some_and(|v| {
            v.len() > 2000 || v.chars().any(|c| c.is_control() && c != '\n' && c != '\t')
        }) || self
            .external_reference
            .as_ref()
            .is_some_and(|v| v.len() > 256 || v.chars().any(char::is_control))
        {
            return Err("invalid note or external_reference".into());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderExpenseRecord {
    pub id: String,
    #[serde(flatten)]
    pub entry: ProviderExpenseInput,
    pub created_at_unix_ms: u64,
    pub voided_at_unix_ms: Option<u64>,
    pub voided_by: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ProviderExpenseQuery {
    pub from_unix_ms: u64,
    pub to_unix_ms: u64,
    pub limit: u32,
    pub offset: u64,
}
impl ProviderExpenseQuery {
    pub fn validate(&self) -> Result<(), crate::DataLayerError> {
        if self.from_unix_ms >= self.to_unix_ms
            || self.to_unix_ms > 253_402_300_799_000
            || self.to_unix_ms - self.from_unix_ms > 366 * 86_400_000
            || self.limit == 0
            || self.limit > 10_001
            || self.offset > i64::MAX as u64
        {
            return Err(crate::DataLayerError::InvalidInput(
                "invalid provider expense range or pagination".into(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProviderExpenseTotals {
    pub currency: String,
    pub amount: String,
    pub recharge_amount: String,
    pub subscription_amount: String,
    pub other_amount: String,
    pub entry_count: u64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderExpenseProviderTotal {
    pub provider_id: String,
    pub provider_name: String,
    pub currency: String,
    pub amount: String,
    pub entry_count: u64,
}
#[derive(Debug, Clone, Default)]
pub struct ProviderExpensePage {
    pub items: Vec<ProviderExpenseRecord>,
    pub total: u64,
    pub totals: Vec<ProviderExpenseTotals>,
    pub providers: Vec<ProviderExpenseProviderTotal>,
}

pub fn provider_expense_memory_page(
    entries: impl Iterator<Item = ProviderExpenseRecord>,
    query: &ProviderExpenseQuery,
) -> Result<ProviderExpensePage, crate::DataLayerError> {
    query.validate()?;
    let mut entries = entries
        .filter(|r| {
            r.voided_at_unix_ms.is_none()
                && r.entry.paid_at_unix_ms >= query.from_unix_ms
                && r.entry.paid_at_unix_ms < query.to_unix_ms
        })
        .collect::<Vec<_>>();
    entries.sort_by(|a, b| {
        b.entry
            .paid_at_unix_ms
            .cmp(&a.entry.paid_at_unix_ms)
            .then_with(|| b.id.cmp(&a.id))
    });
    let mut currencies = BTreeMap::<String, ([u128; 3], u64)>::new();
    let mut providers = BTreeMap::<(String, String), (String, u128, u64)>::new();
    for record in &entries {
        let row = &record.entry;
        let units = provider_expense_amount_units(&row.amount).ok_or_else(|| {
            crate::DataLayerError::UnexpectedValue("invalid recorded expense amount".into())
        })?;
        let (amounts, count) = currencies.entry(row.currency.clone()).or_default();
        amounts[match row.kind.as_str() {
            "recharge" => 0,
            "subscription" => 1,
            _ => 2,
        }] += units;
        *count += 1;
        let (name, amount, count) = providers
            .entry((row.provider_id.clone(), row.currency.clone()))
            .or_insert_with(|| (row.provider_name.clone(), 0, 0));
        let _ = name;
        *amount += units;
        *count += 1;
    }
    Ok(ProviderExpensePage {
        total: entries.len() as u64,
        items: entries
            .into_iter()
            .skip(query.offset as usize)
            .take(query.limit as usize)
            .collect(),
        totals: currencies
            .into_iter()
            .map(|(currency, (amounts, entry_count))| ProviderExpenseTotals {
                currency,
                amount: format_provider_expense_amount(amounts.iter().sum()),
                recharge_amount: format_provider_expense_amount(amounts[0]),
                subscription_amount: format_provider_expense_amount(amounts[1]),
                other_amount: format_provider_expense_amount(amounts[2]),
                entry_count,
            })
            .collect(),
        providers: providers
            .into_iter()
            .map(
                |((provider_id, currency), (provider_name, amount, entry_count))| {
                    ProviderExpenseProviderTotal {
                        provider_id,
                        provider_name,
                        currency,
                        amount: format_provider_expense_amount(amount),
                        entry_count,
                    }
                },
            )
            .collect(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn provider_expense_amounts_are_exact_and_reject_ambiguous_inputs() {
        assert_eq!(
            format_provider_expense_amount(provider_expense_amount_units("0.12345678").unwrap()),
            "0.12345678"
        );
        for bad in [
            "0",
            "-1",
            "+1",
            "1e2",
            "NaN",
            "1.000000001",
            "1000000000000",
            " 1",
            ".1",
        ] {
            assert!(provider_expense_amount_units(bad).is_none(), "{bad}");
        }
    }
    #[test]
    fn provider_expense_report_has_currency_separation_void_exclusion_and_full_page_totals() {
        let entry =
            |id: &str, currency: &str, amount: &str, kind: &str, paid: u64, voided: bool| {
                ProviderExpenseRecord {
                    id: id.into(),
                    entry: ProviderExpenseInput {
                        client_request_id: uuid::Uuid::new_v4().to_string(),
                        provider_id: "p".into(),
                        provider_name: "Supplier".into(),
                        kind: kind.into(),
                        amount: amount.into(),
                        currency: currency.into(),
                        paid_at_unix_ms: paid,
                        period_start_unix_ms: None,
                        period_end_unix_ms: None,
                        note: None,
                        external_reference: None,
                        created_by: None,
                    },
                    created_at_unix_ms: paid,
                    voided_at_unix_ms: voided.then_some(paid + 1),
                    voided_by: None,
                }
            };
        let rows = vec![
            entry("1", "USD", "0.1", "recharge", 10, false),
            entry("2", "USD", "0.2", "subscription", 20, false),
            entry("3", "CNY", "5", "other", 20, false),
            entry("4", "USD", "99", "recharge", 20, true),
            entry("5", "USD", "99", "recharge", 30, false),
        ];
        let page = provider_expense_memory_page(
            rows.into_iter(),
            &ProviderExpenseQuery {
                from_unix_ms: 10,
                to_unix_ms: 30,
                limit: 1,
                offset: 1,
            },
        )
        .unwrap();
        assert_eq!(page.total, 3);
        assert_eq!(page.items.len(), 1);
        assert_eq!(page.totals[0].currency, "CNY");
        assert_eq!(page.totals[0].amount, "5.00000000");
        assert_eq!(page.totals[1].amount, "0.30000000");
        assert_eq!(page.totals[1].recharge_amount, "0.10000000");
        assert_eq!(page.totals[1].subscription_amount, "0.20000000");
    }
}

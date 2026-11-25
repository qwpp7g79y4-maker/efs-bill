use rust_decimal::Decimal;
use std::collections::BTreeMap;

use crate::parse::FileSystem;
use crate::price::Prices;
use crate::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FsCost {
    pub name: String,
    pub standard_bytes: u64,
    pub ia_bytes: u64,
    pub standard_cost: Decimal,
    pub ia_cost: Decimal,
    pub total: Decimal,
    pub ia_savings_if_moved: Decimal,
    pub savings_note: Option<String>,
    pub group: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bill {
    pub rows: Vec<FsCost>,
    pub total_standard_cost: Decimal,
    pub total_ia_cost: Decimal,
    pub total: Decimal,
    pub total_ia_savings_if_moved: Decimal,
    pub currency: String,
    pub group_by: Option<String>,
}

fn overflow() -> Error {
    Error::msg(
        "amount too large to compute: a rate or size in the input overflows the decimal range",
    )
}

fn mul(a: Decimal, b: Decimal) -> Result<Decimal, Error> {
    a.checked_mul(b).ok_or_else(overflow)
}

fn add(a: Decimal, b: Decimal) -> Result<Decimal, Error> {
    a.checked_add(b).ok_or_else(overflow)
}

fn sub(a: Decimal, b: Decimal) -> Result<Decimal, Error> {
    a.checked_sub(b).ok_or_else(overflow)
}

fn gib(bytes: u64) -> Result<Decimal, Error> {
    Decimal::from(bytes)
        .checked_div(Decimal::from(1024u64.pow(3)))
        .ok_or_else(overflow)
}

fn savings_note(suggest_bytes: u64, savings: Decimal, currency: &str) -> Option<String> {
    if suggest_bytes > 0 && savings > Decimal::ZERO {
        Some(format!(
            "move ~{suggest_bytes} Standard bytes to IA → save ~{savings} {currency}"
        ))
    } else {
        None
    }
}

pub fn bill_filesystems(
    filesystems: &[FileSystem],
    prices: &Prices,
    group_by: Option<&str>,
) -> Result<Bill, Error> {
    prices.validate()?;
    // Only a positive Standard-minus-IA difference is a saving.
    let rate_diff = sub(prices.standard_per_gb_month, prices.ia_per_gb_month)?.max(Decimal::ZERO);
    let mut rows = Vec::with_capacity(filesystems.len());
    let mut suggested: Vec<u64> = Vec::with_capacity(filesystems.len());
    let mut total_standard_cost = Decimal::ZERO;
    let mut total_ia_cost = Decimal::ZERO;
    let mut total_ia_savings_if_moved = Decimal::ZERO;
    for fs in filesystems {
        let standard_cost = mul(gib(fs.standard_bytes)?, prices.standard_per_gb_month)?.round_dp(6);
        let ia_cost = mul(gib(fs.ia_bytes)?, prices.ia_per_gb_month)?.round_dp(6);
        let total = add(standard_cost, ia_cost)?;
        let savings = mul(gib(fs.suggest_ia_bytes)?, rate_diff)?.round_dp(6);
        let savings_note = savings_note(fs.suggest_ia_bytes, savings, &prices.currency);
        let group = group_by.and_then(|key| fs.tags.get(key).cloned());
        total_standard_cost = add(total_standard_cost, standard_cost)?;
        total_ia_cost = add(total_ia_cost, ia_cost)?;
        total_ia_savings_if_moved = add(total_ia_savings_if_moved, savings)?;
        suggested.push(fs.suggest_ia_bytes);
        rows.push(FsCost {
            name: fs.name.clone(),
            standard_bytes: fs.standard_bytes,
            ia_bytes: fs.ia_bytes,
            standard_cost,
            ia_cost,
            total,
            ia_savings_if_moved: savings,
            savings_note,
            group,
        });
    }
    if let Some(key) = group_by {
        let mut grouped: BTreeMap<String, (FsCost, u64)> = BTreeMap::new();
        for (row, suggest) in rows.into_iter().zip(suggested) {
            let label = row.group.clone().unwrap_or_else(|| format!("(no {key})"));
            let (entry, entry_suggest) = grouped.entry(label.clone()).or_insert_with(|| {
                (
                    FsCost {
                        name: label,
                        standard_bytes: 0,
                        ia_bytes: 0,
                        standard_cost: Decimal::ZERO,
                        ia_cost: Decimal::ZERO,
                        total: Decimal::ZERO,
                        ia_savings_if_moved: Decimal::ZERO,
                        savings_note: None,
                        group: row.group.clone(),
                    },
                    0,
                )
            });
            entry.standard_bytes = entry.standard_bytes.saturating_add(row.standard_bytes);
            entry.ia_bytes = entry.ia_bytes.saturating_add(row.ia_bytes);
            entry.standard_cost = add(entry.standard_cost, row.standard_cost)?;
            entry.ia_cost = add(entry.ia_cost, row.ia_cost)?;
            entry.total = add(entry.total, row.total)?;
            entry.ia_savings_if_moved = add(entry.ia_savings_if_moved, row.ia_savings_if_moved)?;
            *entry_suggest = entry_suggest.saturating_add(suggest);
        }
        // One note per group, built from the group's own totals.
        rows = grouped
            .into_values()
            .map(|(mut entry, suggest)| {
                entry.savings_note =
                    savings_note(suggest, entry.ia_savings_if_moved, &prices.currency);
                entry
            })
            .collect();
    } else {
        rows.sort_by(|a, b| a.name.cmp(&b.name));
    }
    Ok(Bill {
        rows,
        total_standard_cost: total_standard_cost.round_dp(6),
        total_ia_cost: total_ia_cost.round_dp(6),
        total: add(total_standard_cost, total_ia_cost)?.round_dp(6),
        total_ia_savings_if_moved: total_ia_savings_if_moved.round_dp(6),
        currency: prices.currency.clone(),
        group_by: group_by.map(str::to_string),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::FileSystem;
    use rust_decimal::Decimal;
    use std::collections::BTreeMap;
    use std::str::FromStr;

    fn fs(name: &str, std: u64, ia: u64, suggest: u64) -> FileSystem {
        FileSystem {
            name: name.into(),
            id: format!("fs-{name}"),
            standard_bytes: std,
            ia_bytes: ia,
            suggest_ia_bytes: suggest,
            tags: BTreeMap::new(),
        }
    }

    #[test]
    fn one_gib_standard() {
        let bill =
            bill_filesystems(&[fs("a", 1024u64.pow(3), 0, 0)], &Prices::builtin(), None).unwrap();
        assert_eq!(bill.total, Decimal::from_str("0.300000").unwrap());
    }

    #[test]
    fn one_gib_ia() {
        let bill =
            bill_filesystems(&[fs("a", 0, 1024u64.pow(3), 0)], &Prices::builtin(), None).unwrap();
        assert_eq!(bill.total, Decimal::from_str("0.016000").unwrap());
    }

    #[test]
    fn savings_line() {
        let bill = bill_filesystems(
            &[fs("a", 1024u64.pow(3), 0, 1024u64.pow(3))],
            &Prices::builtin(),
            None,
        )
        .unwrap();
        // 0.30 - 0.016 = 0.284
        assert_eq!(
            bill.total_ia_savings_if_moved,
            Decimal::from_str("0.284000").unwrap()
        );
        assert!(bill.rows[0].savings_note.as_ref().unwrap().contains("IA"));
    }

    #[test]
    fn group_by_tag() {
        let mut a = fs("a", 1024u64.pow(3), 0, 0);
        a.tags.insert("env".into(), "prod".into());
        let mut b = fs("b", 1024u64.pow(3), 0, 0);
        b.tags.insert("env".into(), "prod".into());
        let bill = bill_filesystems(&[a, b], &Prices::builtin(), Some("env")).unwrap();
        assert_eq!(bill.rows.len(), 1);
        assert_eq!(bill.rows[0].total, Decimal::from_str("0.600000").unwrap());
    }

    fn prices(std: &str, ia: &str) -> Prices {
        Prices {
            standard_per_gb_month: Decimal::from_str(std).unwrap(),
            ia_per_gb_month: Decimal::from_str(ia).unwrap(),
            ..Prices::builtin()
        }
    }

    #[test]
    fn huge_rate_is_error_not_panic() {
        let huge = "79228162514264337593543950335";
        assert!(bill_filesystems(&[fs("a", 1u64 << 40, 0, 0)], &prices(huge, "0"), None).is_err());
        assert!(bill_filesystems(&[fs("a", 0, 1u64 << 40, 0)], &prices("0", huge), None).is_err());
        let neg_huge = "-79228162514264337593543950335";
        assert!(bill_filesystems(&[fs("a", 0, 0, 0)], &prices("0", neg_huge), None).is_err());
        assert!(bill_filesystems(&[fs("a", 0, 0, 0)], &prices(huge, "0"), None).is_ok());
    }

    #[test]
    fn negative_rate_is_error() {
        assert!(bill_filesystems(&[fs("a", 1, 0, 0)], &prices("-0.30", "0.016"), None).is_err());
    }

    #[test]
    fn no_saving_claim_when_ia_not_cheaper() {
        let b = bill_filesystems(
            &[fs("a", 1024u64.pow(3), 0, 1024u64.pow(3))],
            &prices("0.016", "0.30"),
            None,
        )
        .unwrap();
        assert_eq!(b.total_ia_savings_if_moved, Decimal::ZERO);
        assert!(b.rows[0].savings_note.is_none());
    }

    #[test]
    fn group_note_matches_group_total() {
        let gib = 1024u64.pow(3);
        let mut a = fs("a", gib, 0, 0);
        a.tags.insert("env".into(), "prod".into());
        let mut b = fs("b", gib, 0, gib);
        b.tags.insert("env".into(), "prod".into());
        let mut c = fs("c", gib, 0, gib);
        c.tags.insert("env".into(), "prod".into());
        let bill = bill_filesystems(&[a, b, c], &Prices::builtin(), Some("env")).unwrap();
        assert_eq!(bill.rows.len(), 1);
        assert_eq!(
            bill.rows[0].ia_savings_if_moved,
            Decimal::from_str("0.568").unwrap()
        );
        let note = bill.rows[0].savings_note.as_ref().unwrap();
        assert!(note.contains("~0.568 USD"), "{note}");
        assert!(note.contains(&format!("~{} Standard", 2 * gib)), "{note}");
    }

    #[test]
    fn missing_group_bucket() {
        let bill = bill_filesystems(&[fs("a", 0, 0, 0)], &Prices::builtin(), Some("env")).unwrap();
        assert_eq!(bill.rows[0].name, "(no env)");
    }

    #[test]
    fn sorts_by_name() {
        let bill = bill_filesystems(
            &[fs("b", 0, 0, 0), fs("a", 0, 0, 0)],
            &Prices::builtin(),
            None,
        )
        .unwrap();
        assert_eq!(bill.rows[0].name, "a");
    }

    #[test]
    fn empty_input() {
        let bill = bill_filesystems(&[], &Prices::builtin(), None).unwrap();
        assert!(bill.rows.is_empty());
    }

    #[test]
    fn currency_propagates() {
        assert_eq!(
            bill_filesystems(&[fs("a", 0, 0, 0)], &Prices::builtin(), None)
                .unwrap()
                .currency,
            "USD"
        );
    }

    #[test]
    fn formats_builtin() {
        assert_eq!(Prices::builtin().standard_per_gb_month.to_string(), "0.30");
        assert_eq!(Prices::builtin().ia_per_gb_month.to_string(), "0.016");
    }
}

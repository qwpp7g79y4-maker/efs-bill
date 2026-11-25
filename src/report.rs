use crate::bill::Bill;
use serde_json::json;

/// Input text must not add rows, columns or terminal escapes to the text table:
/// every control character (newline, tab, ESC, C1 …) becomes `?`.
fn clean(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_control() { '?' } else { c })
        .collect()
}

pub fn render_text(bill: &Bill) -> String {
    let mut out = String::new();
    let label = clean(bill.group_by.as_deref().unwrap_or("filesystem"));
    let currency = clean(&bill.currency);
    let bill = &Bill {
        currency,
        ..bill.clone()
    };
    out.push_str(&format!(
        "{label}\tstandard_bytes\tia_bytes\tstandard_cost\tia_cost\ttotal\tia_savings_if_moved\n"
    ));
    for row in &bill.rows {
        out.push_str(&format!(
            "{}\t{}\t{}\t{} {}\t{} {}\t{} {}\t{} {}\n",
            clean(&row.name),
            row.standard_bytes,
            row.ia_bytes,
            row.standard_cost,
            bill.currency,
            row.ia_cost,
            bill.currency,
            row.total,
            bill.currency,
            row.ia_savings_if_moved,
            bill.currency
        ));
        if let Some(note) = &row.savings_note {
            out.push_str(&format!("# {}\n", clean(note)));
        }
    }
    out.push_str(&format!(
        "total\t\t\t{} {}\t{} {}\t{} {}\t{} {}\n",
        bill.total_standard_cost,
        bill.currency,
        bill.total_ia_cost,
        bill.currency,
        bill.total,
        bill.currency,
        bill.total_ia_savings_if_moved,
        bill.currency
    ));
    out
}

pub fn render_json(bill: &Bill) -> String {
    let rows: Vec<_> = bill
        .rows
        .iter()
        .map(|r| {
            json!({
                "name": r.name,
                "standard_bytes": r.standard_bytes,
                "ia_bytes": r.ia_bytes,
                "standard_cost": r.standard_cost.to_string(),
                "ia_cost": r.ia_cost.to_string(),
                "total": r.total.to_string(),
                "ia_savings_if_moved": r.ia_savings_if_moved.to_string(),
                "savings_note": r.savings_note,
                "group": r.group,
            })
        })
        .collect();
    let doc = json!({
        "schema_version": 1,
        "tool": "efs-bill",
        "currency": bill.currency,
        "group_by": bill.group_by,
        "total_standard_cost": bill.total_standard_cost.to_string(),
        "total_ia_cost": bill.total_ia_cost.to_string(),
        "total": bill.total.to_string(),
        "total_ia_savings_if_moved": bill.total_ia_savings_if_moved.to_string(),
        "filesystems": rows,
    });
    format!(
        "{}\n",
        serde_json::to_string_pretty(&doc).unwrap_or_default()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bill::bill_filesystems;
    use crate::parse::FileSystem;
    use crate::price::Prices;
    use std::collections::BTreeMap;

    fn sample() -> Bill {
        bill_filesystems(
            &[FileSystem {
                name: "data".into(),
                id: "fs-1".into(),
                standard_bytes: 1024u64.pow(3),
                ia_bytes: 0,
                suggest_ia_bytes: 1024u64.pow(3),
                tags: BTreeMap::new(),
            }],
            &Prices::builtin(),
            None,
        )
        .unwrap()
    }

    #[test]
    fn text_has_header_total_and_note() {
        let t = render_text(&sample());
        assert!(t.contains("filesystem\t"));
        assert!(t.contains("total\t"));
        assert!(t.contains("0.30"));
        assert!(t.contains("# move"));
    }

    #[test]
    fn text_neutralises_control_characters() {
        let mut b = sample();
        b.rows[0].name = "a\nfake\trow\x1b[31m\u{9b}".into();
        b.currency = "U\nSD".into();
        let t = render_text(&b);
        assert!(!t.contains('\x1b') && !t.contains('\u{9b}'));
        assert!(t.lines().all(|l| l.starts_with("filesystem")
            || l.starts_with("total")
            || l.starts_with("# ")
            || l.starts_with("a?fake?row")));
        assert_eq!(t.lines().count(), 4);
    }

    #[test]
    fn json_schema() {
        let v: serde_json::Value = serde_json::from_str(&render_json(&sample())).unwrap();
        assert_eq!(v["schema_version"], 1);
        assert_eq!(v["tool"], "efs-bill");
    }
}

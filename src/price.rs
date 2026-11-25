use rust_decimal::Decimal;
use serde_json::Value;
use std::str::FromStr;

use crate::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Prices {
    pub currency: String,
    pub region: String,
    pub source: String,
    pub date: String,
    pub standard_per_gb_month: Decimal,
    pub ia_per_gb_month: Decimal,
}

impl Prices {
    pub fn builtin() -> Self {
        Self {
            currency: "USD".into(),
            region: "us-east-1".into(),
            source: "https://aws.amazon.com/efs/pricing/".into(),
            date: "2026-11-09".into(),
            // Regional (Multi-AZ), Elastic Throughput — the default tab on
            // https://aws.amazon.com/efs/pricing/ (consulted 2026-11-09;
            // price list published 2026-11-04). Legacy-throughput IA is
            // 0.025 on that same page; pass --prices for that rate.
            standard_per_gb_month: dec("0.30"),
            ia_per_gb_month: dec("0.016"),
        }
    }

    pub fn from_json(bytes: &[u8]) -> Result<Self, Error> {
        let v: Value = serde_json::from_slice(bytes)
            .map_err(|e| Error::msg(format!("price file is not JSON: {e}")))?;
        let prices = Self {
            currency: req_str(&v, "currency")?,
            region: req_str(&v, "region")?,
            source: req_str(&v, "source")?,
            date: req_str(&v, "date")?,
            standard_per_gb_month: req_dec(&v, "standard_per_gb_month")?,
            ia_per_gb_month: req_dec(&v, "ia_per_gb_month")?,
        };
        prices.validate()?;
        Ok(prices)
    }

    /// Allowed range: each rate is zero or positive (never negative).
    /// An IA rate above the Standard rate is accepted, but then no saving is claimed.
    pub fn validate(&self) -> Result<(), Error> {
        for (key, rate) in [
            ("standard_per_gb_month", self.standard_per_gb_month),
            ("ia_per_gb_month", self.ia_per_gb_month),
        ] {
            if rate.is_sign_negative() && !rate.is_zero() {
                return Err(Error::msg(format!(
                    "price file {key} is negative ({rate}); a rate must be 0 or more"
                )));
            }
        }
        Ok(())
    }
}

fn dec(s: &str) -> Decimal {
    Decimal::from_str(s).expect("literal")
}

fn req_str(v: &Value, key: &str) -> Result<String, Error> {
    v.get(key)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| Error::msg(format!("price file mist {key}")))
}

fn req_dec(v: &Value, key: &str) -> Result<Decimal, Error> {
    let text = req_str(v, key)?;
    Decimal::from_str(&text)
        .map_err(|_| Error::msg(format!("price file {key} is not a decimal: {text}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_rates() {
        let p = Prices::builtin();
        assert_eq!(p.standard_per_gb_month, dec("0.30"));
        assert_eq!(p.ia_per_gb_month, dec("0.016"));
        assert!(p.source.contains("efs/pricing"));
        assert_eq!(p.date, "2026-11-09");
    }

    #[test]
    fn loads_file_shape() {
        let p = Prices::from_json(include_bytes!("../prices/us-east-1.json")).unwrap();
        assert_eq!(p.region, "us-east-1");
        assert_eq!(p.standard_per_gb_month, dec("0.30"));
        assert_eq!(p.ia_per_gb_month, Prices::builtin().ia_per_gb_month);
    }

    #[test]
    fn rejects_negative_rate() {
        let j = |s: &str, i: &str| {
            format!(
                r#"{{"currency":"USD","region":"r","source":"s","date":"d","standard_per_gb_month":"{s}","ia_per_gb_month":"{i}"}}"#
            )
        };
        assert!(Prices::from_json(j("-0.30", "0.016").as_bytes()).is_err());
        assert!(Prices::from_json(j("0.30", "-0.016").as_bytes()).is_err());
        assert!(Prices::from_json(j("0", "0").as_bytes()).is_ok());
    }

    #[test]
    fn rejects_bad_json() {
        assert!(Prices::from_json(b"{").is_err());
    }

    #[test]
    fn rejects_missing_field() {
        assert!(Prices::from_json(br#"{"currency":"USD"}"#).is_err());
    }
}

//! Parse local EFS JSON exports (`describe-file-systems` style).
//!
//! Fields follow
//! <https://docs.aws.amazon.com/efs/latest/ug/API_FileSystemDescription.html>
//! (geraadpleegd 2026-11-09).

use serde_json::Value;
use std::collections::BTreeMap;

use crate::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileSystem {
    pub name: String,
    pub id: String,
    pub standard_bytes: u64,
    pub ia_bytes: u64,
    /// Hint from lifecycle / SizeInBytes.ValueInIA that cold data could move to IA.
    pub suggest_ia_bytes: u64,
    pub tags: BTreeMap<String, String>,
}

pub fn parse_export(bytes: &[u8]) -> Result<Vec<FileSystem>, Error> {
    let v: Value = serde_json::from_slice(bytes)
        .map_err(|e| Error::msg(format!("input is not JSON: {e}. Pass an EFS export file")))?;
    if let Some(arr) = v.get("FileSystems").and_then(Value::as_array) {
        return arr.iter().map(parse_fs).collect();
    }
    if v.get("FileSystemId").is_some() || v.get("SizeInBytes").is_some() {
        return Ok(vec![parse_fs(&v)?]);
    }
    Err(Error::msg(
        "JSON mist FileSystems[] of een enkel filesystem; geef een efs-bill-export",
    ))
}

fn parse_fs(v: &Value) -> Result<FileSystem, Error> {
    if !v.is_object() {
        return Err(Error::msg(
            "FileSystems[] contains an entry that is not a JSON object",
        ));
    }
    let id = v
        .get("FileSystemId")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let name = v
        .get("Name")
        .and_then(Value::as_str)
        .or_else(|| {
            v.get("FileSystemArn")
                .and_then(Value::as_str)
                .and_then(|a| a.rsplit('/').next())
        })
        .unwrap_or(if id.is_empty() {
            "filesystem"
        } else {
            id.as_str()
        })
        .to_string();

    let (standard_bytes, ia_bytes) = sizes(v)?;
    let suggest_ia_bytes = suggest_ia(v, standard_bytes, ia_bytes)?;

    let mut tags = BTreeMap::new();
    collect_tags(v.get("Tags"), &mut tags);
    collect_tags(v.get("Metadata"), &mut tags);

    Ok(FileSystem {
        name,
        id,
        standard_bytes,
        ia_bytes,
        suggest_ia_bytes,
        tags,
    })
}

fn sizes(v: &Value) -> Result<(u64, u64), Error> {
    // Prefer explicit StandardBytes / IaBytes; else SizeInBytes breakdown.
    let explicit_std = num_u64(v, "StandardBytes")?;
    let explicit_ia = num_u64(v, "IaBytes")?
        .or(num_u64(v, "IABytes")?)
        .or(num_u64(v, "InfrequentAccessBytes")?);

    if explicit_std.is_some() || explicit_ia.is_some() {
        return Ok((explicit_std.unwrap_or(0), explicit_ia.unwrap_or(0)));
    }

    if let Some(sz) = v.get("SizeInBytes") {
        let value = num_u64(sz, "Value")?;
        let ia = num_u64(sz, "ValueInIA")?;
        if value.is_some() || ia.is_some() {
            let ia = ia.unwrap_or(0);
            return Ok((value.unwrap_or(0).saturating_sub(ia), ia));
        }
    }

    // StorageBytes map
    if let Some(sb) = v.get("StorageBytes") {
        let standard = num_u64(sb, "Standard")?;
        let ia = num_u64(sb, "IA")?.or(num_u64(sb, "InfrequentAccess")?);
        if standard.is_some() || ia.is_some() {
            return Ok((standard.unwrap_or(0), ia.unwrap_or(0)));
        }
    }

    Err(Error::msg(
        "filesystem has no recognised size field (StandardBytes, IaBytes, SizeInBytes.Value/ValueInIA or StorageBytes.Standard/IA)",
    ))
}

/// True when a lifecycle value (list of policies, or one policy object) has a
/// real `TransitionToIA` rule. `TransitionToPrimaryStorageClass` (AFTER_1_ACCESS)
/// and `TransitionToArchive` do not move data to IA.
fn transitions_to_ia(v: &Value) -> bool {
    match v {
        Value::Array(items) => items.iter().any(transitions_to_ia),
        Value::Object(_) => v
            .get("TransitionToIA")
            .map(|t| !t.is_null() && t.as_str() != Some(""))
            .unwrap_or(false),
        _ => false,
    }
}

fn suggest_ia(v: &Value, standard_bytes: u64, ia_bytes: u64) -> Result<u64, Error> {
    if let Some(n) = num_u64(v, "SuggestIaBytes")? {
        return Ok(n.min(standard_bytes));
    }
    // Only a policy that really transitions to IA, while Standard still holds data
    // and IA is empty, suggests moving all Standard to IA.
    let ia_policy = v
        .get("LifecyclePolicies")
        .map(transitions_to_ia)
        .unwrap_or(false)
        || v.get("LifecycleConfiguration")
            .map(transitions_to_ia)
            .unwrap_or(false);
    if ia_policy && standard_bytes > 0 && ia_bytes == 0 {
        return Ok(standard_bytes);
    }
    // Access pattern hint
    if matches!(
        v.get("AccessPattern").and_then(Value::as_str),
        Some("infrequent") | Some("cold") | Some("IA")
    ) {
        return Ok(standard_bytes);
    }
    Ok(0)
}

fn collect_tags(v: Option<&Value>, tags: &mut BTreeMap<String, String>) {
    let Some(v) = v else { return };
    if let Some(obj) = v.as_object() {
        for (k, val) in obj {
            if let Some(s) = val.as_str() {
                tags.entry(k.clone()).or_insert_with(|| s.to_string());
            }
        }
    }
    if let Some(arr) = v.as_array() {
        for item in arr {
            if let (Some(k), Some(s)) = (
                item.get("Key").and_then(Value::as_str),
                item.get("Value").and_then(Value::as_str),
            ) {
                tags.entry(k.to_string()).or_insert_with(|| s.to_string());
            }
        }
    }
}

fn num_u64(v: &Value, key: &str) -> Result<Option<u64>, Error> {
    match v.get(key) {
        None => Ok(None),
        Some(Value::Number(n)) => n
            .as_u64()
            .map(Some)
            .ok_or_else(|| Error::msg(format!("{key} is geen niet-negatief geheel getal"))),
        Some(Value::String(s)) => s
            .parse::<u64>()
            .map(Some)
            .map_err(|_| Error::msg(format!("{key} is geen niet-negatief geheel getal: {s}"))),
        Some(_) => Err(Error::msg(format!("{key} heeft een ongeldig type"))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_standard() {
        let f = parse_export(
            br#"{"FileSystems":[{"FileSystemId":"fs-1","Name":"data","StandardBytes":1073741824,"IaBytes":0}]}"#,
        )
        .unwrap();
        assert_eq!(f[0].name, "data");
        assert_eq!(f[0].standard_bytes, 1073741824);
        assert_eq!(f[0].ia_bytes, 0);
    }

    #[test]
    fn parses_size_in_bytes() {
        let f = parse_export(
            br#"{"FileSystems":[{"FileSystemId":"fs-2","SizeInBytes":{"Value":2147483648,"ValueInIA":1073741824}}]}"#,
        )
        .unwrap();
        assert_eq!(f[0].standard_bytes, 1073741824);
        assert_eq!(f[0].ia_bytes, 1073741824);
    }

    #[test]
    fn lifecycle_suggests_ia() {
        let f = parse_export(
            br#"{"FileSystems":[{"FileSystemId":"fs-3","Name":"cold","StandardBytes":1073741824,"IaBytes":0,"LifecyclePolicies":[{"TransitionToIA":"AFTER_30_DAYS"}]}]}"#,
        )
        .unwrap();
        assert_eq!(f[0].suggest_ia_bytes, 1073741824);
    }

    #[test]
    fn after_1_access_does_not_suggest() {
        let f = parse_export(
            br#"{"FileSystems":[{"FileSystemId":"fs-6","StandardBytes":100,"IaBytes":0,"LifecyclePolicies":[{"TransitionToPrimaryStorageClass":"AFTER_1_ACCESS"}]}]}"#,
        )
        .unwrap();
        assert_eq!(f[0].suggest_ia_bytes, 0);
    }

    #[test]
    fn no_size_field_is_error() {
        assert!(parse_export(br#"{"FileSystems":[{"FileSystemId":"fs-7"}]}"#).is_err());
        assert!(
            parse_export(br#"{"FileSystems":[{"FileSystemId":"fs-7","SizeInBytes":{}}]}"#).is_err()
        );
        assert!(
            parse_export(br#"{"FileSystems":[{"FileSystemId":"fs-7","SizeInBytes":5}]}"#).is_err()
        );
        assert!(parse_export(br#"{"FileSystemId":"fs-8"}"#).is_err());
    }

    #[test]
    fn non_object_entry_is_error() {
        assert!(parse_export(br#"{"FileSystems":[1]}"#).is_err());
        assert!(parse_export(br#"{"FileSystems":["x",null]}"#).is_err());
    }

    #[test]
    fn access_pattern_suggests() {
        let f = parse_export(
            br#"{"FileSystems":[{"FileSystemId":"fs-4","StandardBytes":100,"IaBytes":0,"AccessPattern":"cold"}]}"#,
        )
        .unwrap();
        assert_eq!(f[0].suggest_ia_bytes, 100);
    }

    #[test]
    fn tags_list() {
        let f = parse_export(
            br#"{"FileSystems":[{"FileSystemId":"fs-5","StandardBytes":0,"Tags":[{"Key":"env","Value":"prod"}]}]}"#,
        )
        .unwrap();
        assert_eq!(f[0].tags.get("env").map(String::as_str), Some("prod"));
    }

    #[test]
    fn rejects_broken_json() {
        assert!(parse_export(b"{").is_err());
    }

    #[test]
    fn rejects_wrong_shape() {
        assert!(parse_export(br#"{"hello":1}"#).is_err());
    }

    #[test]
    fn bad_size() {
        assert!(parse_export(
            br#"{"FileSystems":[{"FileSystemId":"fs-x","StandardBytes":"nope"}]}"#
        )
        .is_err());
    }

    #[test]
    fn single_filesystem() {
        let f =
            parse_export(br#"{"FileSystemId":"fs-solo","Name":"solo","StandardBytes":1}"#).unwrap();
        assert_eq!(f[0].name, "solo");
    }
}

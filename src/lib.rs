//! Offline estimate of AWS EFS Standard/IA GB-month cost from local JSON.
//!
//! Pricing: <https://aws.amazon.com/efs/pricing/> (geraadpleegd 2026-11-09).
//! FileSystemDescription: <https://docs.aws.amazon.com/efs/latest/ug/API_FileSystemDescription.html>
//! (geraadpleegd 2026-11-09).

#![forbid(unsafe_code)]

mod bill;
mod error;
mod parse;
mod price;
mod report;

pub use bill::{bill_filesystems, Bill, FsCost};
pub use error::Error;
pub use parse::{parse_export, FileSystem};
pub use price::Prices;
pub use report::{render_json, render_text};

use std::fs;
use std::path::{Path, PathBuf};

const FILE_CAP: u64 = 32 * 1024 * 1024;

pub struct Options {
    pub inputs: Vec<PathBuf>,
    pub prices: Option<PathBuf>,
    pub group_by: Option<String>,
    pub json: bool,
}

pub fn run(options: &Options) -> Result<String, Error> {
    let prices = match &options.prices {
        Some(path) => Prices::from_json(&read_capped(path, "prices")?)?,
        None => Prices::builtin(),
    };
    let mut filesystems = Vec::new();
    for path in &options.inputs {
        if path.is_dir() {
            let mut files: Vec<_> = fs::read_dir(path)
                .map_err(|e| Error::msg(format!("cannot read directory {}: {e}", path.display())))?
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("json"))
                .collect();
            files.sort();
            if files.is_empty() {
                return Err(Error::msg(format!(
                    "directory {} has no readable file with the lowercase extension .json",
                    path.display()
                )));
            }
            for file in files {
                filesystems.extend(parse_export(&read_capped(&file, "input")?)?);
            }
        } else {
            filesystems.extend(parse_export(&read_capped(path, "input")?)?);
        }
    }
    let bill = bill_filesystems(&filesystems, &prices, options.group_by.as_deref())?;
    if options.json {
        Ok(render_json(&bill))
    } else {
        Ok(render_text(&bill))
    }
}

fn read_capped(path: &Path, label: &str) -> Result<Vec<u8>, Error> {
    let meta = fs::metadata(path)
        .map_err(|e| Error::msg(format!("cannot read {label} {}: {e}", path.display())))?;
    if !meta.is_file() {
        return Err(Error::msg(format!(
            "{label} {} is not a regular file",
            path.display()
        )));
    }
    if meta.len() > FILE_CAP {
        return Err(Error::msg(format!(
            "{label} {} is larger than 32 MiB",
            path.display()
        )));
    }
    fs::read(path).map_err(|e| Error::msg(format!("cannot read {label} {}: {e}", path.display())))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_on_fixture() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/standard.json");
        let out = run(&Options {
            inputs: vec![path],
            prices: None,
            group_by: None,
            json: false,
        })
        .unwrap();
        assert!(out.contains("0.30"), "{out}");
    }

    #[test]
    fn run_ia_fixture() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/ia.json");
        let out = run(&Options {
            inputs: vec![path],
            prices: None,
            group_by: None,
            json: false,
        })
        .unwrap();
        assert!(out.contains("0.016"), "{out}");
    }

    #[test]
    fn directory_without_lowercase_json_is_error() {
        let dir = std::env::temp_dir().join(format!("efs-bill-upper-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("a.JSON"),
            br#"{"FileSystemId":"x","StandardBytes":1}"#,
        )
        .unwrap();
        let r = run(&Options {
            inputs: vec![dir.clone()],
            prices: None,
            group_by: None,
            json: false,
        });
        std::fs::remove_dir_all(&dir).unwrap();
        assert!(r.is_err());
    }

    #[test]
    fn missing_file() {
        assert!(run(&Options {
            inputs: vec![PathBuf::from("/no/such/efs.json")],
            prices: None,
            group_by: None,
            json: false,
        })
        .is_err());
    }
}

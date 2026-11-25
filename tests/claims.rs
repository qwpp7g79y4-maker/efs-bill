use assert_cmd::Command;
use std::path::PathBuf;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn readme_sections() {
    let t = std::fs::read_to_string(root().join("README.md")).unwrap();
    for n in [
        "Why",
        "Install",
        "Quick start",
        "How it counts",
        "Limitations",
        "https://aws.amazon.com/efs/pricing/",
        "2026-11-09",
        "Standard",
        "IA",
        "savings",
    ] {
        assert!(t.contains(n), "missing {n}");
    }
    assert!(!t.contains("PLACEHOLDER"));
}

#[test]
fn changelog_0_1_0() {
    assert!(std::fs::read_to_string(root().join("CHANGELOG.md"))
        .unwrap()
        .contains("0.1.0"));
}

#[test]
fn licenses() {
    assert!(root().join("LICENSE-MIT").is_file());
    assert!(root().join("LICENSE-APACHE").is_file());
}

#[test]
fn examples() {
    Command::cargo_bin("efs-bill")
        .unwrap()
        .arg(root().join("examples/standard.json"))
        .assert()
        .success();
    Command::cargo_bin("efs-bill")
        .unwrap()
        .arg(root().join("examples/ia.json"))
        .assert()
        .success()
        .stdout(predicates::str::contains("0.016"));
}

#[test]
fn quality() {
    let t = std::fs::read_to_string(root().join("QUALITY.md")).unwrap();
    assert!(t.contains("Tests:"));
    assert!(t.contains("clippy"));
}

#[test]
fn api_url_present() {
    let lib = std::fs::read_to_string(root().join("src/lib.rs")).unwrap();
    assert!(lib.contains("FileSystemDescription"));
}

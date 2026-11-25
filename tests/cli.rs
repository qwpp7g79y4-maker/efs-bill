use assert_cmd::Command;
use predicates::prelude::*;
use std::path::PathBuf;

fn bin() -> Command {
    Command::cargo_bin("efs-bill").unwrap()
}
fn fix(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

#[test]
fn help_lists_flags() {
    let a = bin().arg("--help").assert().success();
    let o = String::from_utf8_lossy(&a.get_output().stdout);
    assert!(o.contains("--group-by"));
    assert!(o.contains("--prices"));
    assert!(o.contains("--json"));
}

#[test]
fn version_ok() {
    bin()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("efs-bill"));
}

#[test]
fn standard_fixture() {
    bin()
        .arg(fix("standard.json"))
        .assert()
        .success()
        .stdout(predicate::str::contains("0.30"));
}

#[test]
fn ia_fixture() {
    bin()
        .arg(fix("ia.json"))
        .assert()
        .success()
        .stdout(predicate::str::contains("0.016"));
}

#[test]
fn lifecycle_savings_note() {
    bin()
        .arg(fix("lifecycle.json"))
        .assert()
        .success()
        .stdout(predicate::str::contains("IA"));
}

#[test]
fn json_schema() {
    let out = bin()
        .arg(fix("standard.json"))
        .arg("--json")
        .output()
        .unwrap();
    assert!(out.status.success());
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["schema_version"], 1);
    assert_eq!(v["tool"], "efs-bill");
}

#[test]
fn group_by_env() {
    bin()
        .args([
            fix("standard.json").to_str().unwrap(),
            fix("ia.json").to_str().unwrap(),
            "--group-by",
            "env",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("prod"));
}

#[test]
fn broken_json_exit_2() {
    bin().arg(fix("broken.json")).assert().failure().code(2);
}

#[test]
fn missing_file_exit_2() {
    bin().arg("/no/such/efs.json").assert().failure().code(2);
}

#[test]
fn directory_with_broken_fails() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    bin().arg(&dir).assert().failure();
}

#[test]
fn prices_file() {
    let prices = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("prices/us-east-1.json");
    bin()
        .arg(fix("standard.json"))
        .args(["--prices"])
        .arg(prices)
        .assert()
        .success()
        .stdout(predicate::str::contains("0.30"));
}

#[test]
fn unknown_flag() {
    bin().arg("--nope").assert().failure().code(2);
}

fn tmp(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("efs-bill-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn write_prices(dir: &std::path::Path, std: &str, ia: &str) -> PathBuf {
    let p = dir.join("prices.json");
    std::fs::write(
        &p,
        format!(
            r#"{{"currency":"USD","region":"r","source":"s","date":"d","standard_per_gb_month":"{std}","ia_per_gb_month":"{ia}"}}"#
        ),
    )
    .unwrap();
    p
}

#[test]
fn b1_control_characters_not_raw_in_text_table() {
    let d = tmp("b1");
    let f = d.join("in.json");
    std::fs::write(
        &f,
        br#"{"FileSystems":[{"FileSystemId":"fs-1","Name":"evil\nfake\trow\u001b[31m","StandardBytes":1,"Tags":{"g":"x\ny\u001b"}}]}"#,
    )
    .unwrap();
    for extra in [vec![], vec!["--group-by", "g"]] {
        let out = bin().arg(&f).args(extra).output().unwrap();
        assert!(out.status.success());
        let t = String::from_utf8(out.stdout).unwrap();
        assert!(!t.contains('\u{1b}'), "{t:?}");
        assert_eq!(t.lines().count(), 3, "{t:?}");
    }
    let j = bin().arg(&f).arg("--json").output().unwrap();
    assert!(j.status.success());
    let v: serde_json::Value = serde_json::from_slice(&j.stdout).unwrap();
    assert_eq!(v["filesystems"][0]["name"], "evil\nfake\trow\u{1b}[31m");
    let _ = std::fs::remove_dir_all(d);
}

#[test]
fn b2_huge_rate_exit_2_without_panic() {
    let d = tmp("b2");
    let huge = "79228162514264337593543950335";
    let big = d.join("big.json");
    std::fs::write(
        &big,
        br#"{"FileSystemId":"fs-b","StandardBytes":1099511627776,"IaBytes":1099511627776}"#,
    )
    .unwrap();
    for (std, ia, input) in [
        (huge, "0", big.clone()),
        ("0", huge, big.clone()),
        (huge, "0.016", big.clone()),
    ] {
        let p = write_prices(&d, std, ia);
        bin()
            .arg(input)
            .arg("--prices")
            .arg(&p)
            .assert()
            .failure()
            .code(2)
            .stderr(predicate::str::contains("efs-bill:"))
            .stderr(predicate::str::contains("panicked").not());
    }
    // zero bytes, huge rates (the subtraction used to overflow)
    let z = d.join("zero.json");
    std::fs::write(
        &z,
        br#"{"FileSystemId":"fs-z","StandardBytes":0,"IaBytes":0}"#,
    )
    .unwrap();
    let p = write_prices(
        &d,
        "79228162514264337593543950335",
        "-79228162514264337593543950335",
    );
    bin()
        .arg(&z)
        .arg("--prices")
        .arg(&p)
        .assert()
        .failure()
        .code(2);
    let _ = std::fs::remove_dir_all(d);
}

#[test]
fn b3_negative_rate_exit_2() {
    let d = tmp("b3a");
    for (s, i) in [("-0.30", "0.016"), ("0.30", "-0.016")] {
        let p = write_prices(&d, s, i);
        bin()
            .arg(fix("standard.json"))
            .arg("--prices")
            .arg(&p)
            .assert()
            .failure()
            .code(2)
            .stderr(predicate::str::contains("negative"));
    }
    let _ = std::fs::remove_dir_all(d);
}

#[test]
fn b3_no_saving_when_ia_costs_more() {
    let d = tmp("b3b");
    let p = write_prices(&d, "0.10", "0.20");
    let out = bin()
        .arg(fix("lifecycle.json"))
        .arg("--prices")
        .arg(&p)
        .output()
        .unwrap();
    assert!(out.status.success());
    let t = String::from_utf8(out.stdout).unwrap();
    assert!(!t.contains("# move"), "{t}");
    assert!(!t.contains("save"), "{t}");
    let _ = std::fs::remove_dir_all(d);
}

#[test]
fn b4_group_note_sums_all_rows() {
    let d = tmp("b4b");
    let f = d.join("in.json");
    std::fs::write(
        &f,
        br#"{"FileSystems":[
 {"FileSystemId":"a","StandardBytes":1073741824,"SuggestIaBytes":1073741824,"Tags":{"env":"prod"}},
 {"FileSystemId":"b","StandardBytes":1073741824,"SuggestIaBytes":1073741824,"Tags":{"env":"prod"}}]}"#,
    )
    .unwrap();
    let out = bin().arg(&f).args(["--group-by", "env"]).output().unwrap();
    let t = String::from_utf8(out.stdout).unwrap();
    assert!(t.contains("save ~0.568 USD"), "{t}");
    assert!(!t.contains("save ~0.284 USD\n"), "{t}");
    let _ = std::fs::remove_dir_all(d);
}

#[test]
fn b5_after_1_access_gives_no_move_advice() {
    let d = tmp("b5");
    let f = d.join("in.json");
    std::fs::write(
        &f,
        br#"{"FileSystemId":"a","StandardBytes":1073741824,"IaBytes":0,"LifecyclePolicies":[{"TransitionToPrimaryStorageClass":"AFTER_1_ACCESS"}]}"#,
    )
    .unwrap();
    let out = bin().arg(&f).output().unwrap();
    assert!(out.status.success());
    assert!(!String::from_utf8(out.stdout).unwrap().contains("# move"));
    let _ = std::fs::remove_dir_all(d);
}

#[test]
fn b6_no_size_field_exit_2() {
    let d = tmp("b6a");
    for (i, body) in [
        r#"{"FileSystems":[{"FileSystemId":"a","Name":"nosize"}]}"#,
        r#"{"FileSystems":[1]}"#,
        r#"{"FileSystems":["x"]}"#,
        r#"{"FileSystemId":"solo"}"#,
    ]
    .iter()
    .enumerate()
    {
        let f = d.join(format!("{i}.json"));
        std::fs::write(&f, body).unwrap();
        bin().arg(&f).assert().failure().code(2);
    }
    let _ = std::fs::remove_dir_all(d);
}

#[test]
fn b6_directory_without_lowercase_json_exit_2() {
    let d = tmp("b6b");
    bin().arg(&d).assert().failure().code(2);
    std::fs::write(
        d.join("a.JSON"),
        br#"{"FileSystemId":"a","StandardBytes":1073741824}"#,
    )
    .unwrap();
    bin().arg(&d).assert().failure().code(2);
    std::fs::write(
        d.join("a.json"),
        br#"{"FileSystemId":"a","StandardBytes":1073741824}"#,
    )
    .unwrap();
    bin().arg(&d).assert().success();
    let _ = std::fs::remove_dir_all(d);
}

fn assert_clean_stderr(args: &[&std::path::Path]) {
    let a = bin().args(args).assert().failure().code(2);
    let err = &a.get_output().stderr;
    assert!(!err.is_empty());
    assert!(!err.contains(&0x1b), "ESC in stderr: {err:?}");
    assert!(!err.contains(&0x07), "BEL in stderr: {err:?}");
    assert!(String::from_utf8_lossy(err).contains("efs-bill:"));
}

#[test]
fn b7_esc_in_standard_bytes_not_raw_on_stderr() {
    let d = tmp("b7a");
    let f = d.join("in.json");
    std::fs::write(
        &f,
        r#"{"FileSystems":[{"FileSystemId":"a","StandardBytes":"\u001b[31m\u0007x"}]}"#,
    )
    .unwrap();
    assert_clean_stderr(&[&f]);
    let _ = std::fs::remove_dir_all(d);
}

#[test]
fn b7_esc_in_price_file_not_raw_on_stderr() {
    let d = tmp("b7b");
    let input = d.join("in.json");
    std::fs::write(
        &input,
        r#"{"FileSystems":[{"FileSystemId":"a","StandardBytes":1}]}"#,
    )
    .unwrap();
    let prices = d.join("p.json");
    std::fs::write(
        &prices,
        r#"{"currency":"USD","region":"r","source":"s","date":"d","standard_per_gb_month":"\u001b]0;x\u0007","ia_per_gb_month":"0.016"}"#,
    )
    .unwrap();
    let prices_arg = std::ffi::OsString::from("--prices");
    let a = bin()
        .arg(&input)
        .arg(&prices_arg)
        .arg(&prices)
        .assert()
        .failure()
        .code(2);
    let err = &a.get_output().stderr;
    assert!(!err.contains(&0x1b) && !err.contains(&0x07), "{err:?}");
    assert!(String::from_utf8_lossy(err).contains("not a decimal"));
    let _ = std::fs::remove_dir_all(d);
}

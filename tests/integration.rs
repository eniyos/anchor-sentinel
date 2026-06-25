use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

fn sentinel() -> Command {
    Command::cargo_bin("sentinel").unwrap()
}

#[test]
fn no_such_path() {
    sentinel()
        .args(["scan", "this-path-does-not-exist"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("does not exist"));
}

#[test]
fn cpi_clean_has_no_findings() {
    let fixture =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/cpi-clean");
    sentinel()
        .args(["scan", fixture.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("no findings"));
}

#[test]
fn cpi_vulnerable_triggers_cpi_rules() {
    let fixture =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/cpi-vulnerable");
    let output = sentinel()
        .args(["scan", fixture.to_str().unwrap(), "--format", "json"])
        .output()
        .expect("scan command should run");
    assert!(
        output.status.success(),
        "scan should succeed on the fixture"
    );
    let stdout = String::from_utf8(output.stdout)
        .expect("output should be valid UTF-8");
    let v: serde_json::Value = serde_json::from_str(&stdout)
        .expect("output should be valid JSON");
    let arr = v["findings"]
        .as_array()
        .expect("findings should be an array");
    let has = arr.iter().any(|f| f["rule"] == "cpi_signer_seed_validation");
    assert!(
        has,
        "expected cpi_signer_seed_validation findings on cpi-vulnerable, got:\n{stdout}"
    );
}

#[test]
fn cpi_vulnerable_has_no_false_positives_on_safe() {
    let fixture =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/cpi-vulnerable");
    let output = sentinel()
        .args(["scan", fixture.to_str().unwrap(), "--format", "json"])
        .output()
        .expect("scan command should run");
    assert!(output.status.success(), "scan should succeed");
    let stdout = String::from_utf8(output.stdout)
        .expect("output should be valid UTF-8");
    let v: serde_json::Value = serde_json::from_str(&stdout)
        .expect("output should be valid JSON");
    let arr = v["findings"]
        .as_array()
        .expect("findings should be an array");
    let has = arr
        .iter()
        .any(|f| f["rule"] == "cpi_signer_seed_validation");
    assert!(
        !has,
        "expected no cpi_signer_seed_validation findings on cpi-clean, got:\n{stdout}"
    );
}

#[test]
fn missing_idl_returns_error() {
    let dir = tempfile::tempdir().expect("tempdir should be created");
    sentinel()
        .args(["scan", dir.path().to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicate::str::contains("no IDL files found").or(predicate::str::contains("IDL")));
}

#[test]
fn public_pda_insecure_triggers_pda_misconfig() {
    // The in-tree `public/pda-insecure` fixture models the Sealevel-Attacks
    // pattern: a `bump = bump` argument trap and a no-bump seeds constraint.
    // Both should be flagged by pda_misconfig, and the unchecked `-` should
    // be flagged by unsafe_arithmetic.
    let fixture =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/public/pda-insecure");
    let output = sentinel()
        .args(["scan", fixture.to_str().unwrap(), "--format", "json"])
        .output()
        .expect("scan command should run");
    assert!(
        output.status.success(),
        "scan should not error on the fixture"
    );
    let stdout = String::from_utf8(output.stdout)
        .expect("output should be valid UTF-8");
    let v: serde_json::Value = serde_json::from_str(&stdout)
        .expect("output should be valid JSON");
    let arr = v["findings"]
        .as_array()
        .expect("findings should be an array");
    let rules: Vec<&str> = arr
        .iter()
        .map(|f| {
            f["rule"]
                .as_str()
                .expect("rule should be a string")
        })
        .collect();
    assert!(
        rules.contains(&"pda_misconfig"),
        "expected pda_misconfig, got: {rules:?}"
    );
    assert!(
        rules.contains(&"unsafe_arithmetic"),
        "expected unsafe_arithmetic, got: {rules:?}"
    );
}

#[test]
fn public_pda_secure_has_no_findings() {
    let fixture =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/public/pda-secure");
    sentinel()
        .args(["scan", fixture.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("no findings"));
}

#[test]
fn balance_drain_vulnerable_triggers_balance_rules() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/balance-drain-vulnerable");
    let output = sentinel()
        .args(["scan", fixture.to_str().unwrap(), "--format", "json"])
        .output()
        .expect("scan command should run");
    assert!(output.status.success(), "scan should succeed");
    let stdout = String::from_utf8(output.stdout)
        .expect("output should be valid UTF-8");
    let v: serde_json::Value = serde_json::from_str(&stdout)
        .expect("output should be valid JSON");
    let arr = v["findings"]
        .as_array()
        .expect("findings should be an array");
    let rules: Vec<&str> = arr
        .iter()
        .map(|f| {
            f["rule"]
                .as_str()
                .expect("rule should be a string")
        })
        .collect();
    assert!(
        rules.contains(&"missing_balance_check") || rules.contains(&"lamports_drain"),
        "expected balance-related rules, got: {rules:?}"
    );
}

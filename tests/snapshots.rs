//! JSON snapshot tests for the report shape. Each rule's output is locked
//! in `.snap` files; updates to the schema will cause a test failure that
//! the developer can either accept (via `cargo insta review`) or fix.

use assert_cmd::cargo::cargo_bin;
use assert_cmd::Command;
use std::path::Path;

fn sentinel() -> Command {
    let path = cargo_bin("sentinel");
    Command::new(path)
}

fn fixture(rel: &str) -> String {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(rel)
        .display()
        .to_string()
}

fn run_scan_json(path: &str) -> serde_json::Value {
    let output = sentinel()
        .args(["scan", path, "--json"])
        .output()
        .expect("scan ran");
    assert!(
        output.status.success(),
        "scan failed: stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("utf8");
    serde_json::from_str(&stdout).expect("valid JSON")
}

#[test]
fn snapshot_vulnerable_vault_findings() {
    let v = run_scan_json(&fixture("vault-vulnerable"));
    // Strip out `file` paths which are absolute and machine-specific so
    // snapshots are portable across hosts.
    let stripped = strip_absolute_paths(v);
    insta::assert_json_snapshot!("vulnerable_vault", stripped);
}

#[test]
fn snapshot_pda_insecure_findings() {
    let v = run_scan_json(&fixture("public/pda-insecure"));
    let stripped = strip_absolute_paths(v);
    insta::assert_json_snapshot!("pda_insecure", stripped);
}

#[test]
fn snapshot_clean_vault_no_findings() {
    let v = run_scan_json(&fixture("vault-clean"));
    insta::assert_json_snapshot!("clean_vault", v);
}

#[test]
fn snapshot_clean_pda_no_findings() {
    let v = run_scan_json(&fixture("public/pda-secure"));
    insta::assert_json_snapshot!("clean_pda", v);
}

fn strip_absolute_paths(mut v: serde_json::Value) -> serde_json::Value {
    if let Some(arr) = v.get_mut("findings").and_then(|f| f.as_array_mut()) {
        for finding in arr {
            if let Some(obj) = finding.as_object_mut() {
                if let Some(file) = obj.get("file").and_then(|f| f.as_str()) {
                    // Collapse `/Users/.../programs/.../lib.rs` → `programs/.../lib.rs`.
                    if let Some(idx) = file.find("programs/") {
                        obj.insert(
                            "file".to_string(),
                            serde_json::Value::String(file[idx..].to_string()),
                        );
                    }
                }
            }
        }
    }
    v
}

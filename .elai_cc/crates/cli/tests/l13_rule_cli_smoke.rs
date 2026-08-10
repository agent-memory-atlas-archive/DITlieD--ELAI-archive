//! L-13: real-binary smoke tests for the read-only rule list surface.

use std::process::Command;

fn run(cwd: &std::path::Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_elai"))
        .current_dir(cwd)
        .args(args)
        .output()
        .expect("elai binary should run")
}

fn temp_cwd() -> tempfile::TempDir {
    tempfile::tempdir().expect("temporary cwd should be created")
}

#[test]
fn rule_list_empty_text_is_success() {
    // Falsifies the production `rule` arm in args.rs and RuleList dispatch in main.rs.
    let dir = temp_cwd();
    let output = run(dir.path(), &["rule", "list"]);
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    assert!(String::from_utf8_lossy(&output.stdout).contains("no rules"));
}

#[test]
fn rule_list_empty_json_is_handler_output() {
    // Falsifies the production RuleList dispatch arm in main.rs.
    let dir = temp_cwd();
    let output = run(dir.path(), &["rule", "list", "--json"]);
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).expect("valid rule JSON");
    assert_eq!(value["rules"].as_array().expect("rules array").len(), 0);
}

#[test]
fn rule_rejects_unknown_subcommand() {
    // Falsifies the parser rejection path for an unknown rule verb.
    let dir = temp_cwd();
    let output = run(dir.path(), &["rule", "bogus"]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("list"));
}

#[test]
fn rule_list_rejects_unknown_flag() {
    // Falsifies the parser rejection path for an unknown list flag.
    let dir = temp_cwd();
    let output = run(dir.path(), &["rule", "list", "--bogus"]);
    assert!(!output.status.success());
}

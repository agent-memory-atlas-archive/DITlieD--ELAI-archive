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


#[test]
fn add_then_list_all_shows_proposed_row() {
    // Falsifies the args `add` arm, the main dispatch arm, insert_user_rule and
    // fetch_all_rules together.
    let dir = temp_cwd();
    let output = run(
        dir.path(),
        &[
            "rule",
            "add",
            "--id",
            "l13-add-list",
            "--domain",
            "testing",
            "--tier",
            "2",
            "--severity",
            "WARN",
            "--title",
            "Test rule",
            "--description",
            "A test rule",
            "--predicate",
            r#"{"type":"grep_pattern","pattern":"TODO","context_lines":0,"file_context":null}"#,
        ],
    );
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));

    let output = run(dir.path(), &["rule", "list", "--all", "--json"]);
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).expect("valid rule JSON");
    let rules = value["rules"].as_array().expect("rules array");
    assert_eq!(rules.len(), 1);
    assert_eq!(rules[0]["rule_id"], "l13-add-list");
    assert_eq!(rules[0]["status"], "PROPOSED");
}

#[test]
fn added_rule_is_not_live() {
    // Falsifies any change that inserts a user rule at ACTIVE or SHADOW instead
    // of PROPOSED. A newly authored rule must not be live.
    let dir = temp_cwd();
    let output = run(
        dir.path(),
        &[
            "rule",
            "add",
            "--id",
            "l13-not-live",
            "--domain",
            "testing",
            "--tier",
            "1",
            "--severity",
            "INFO",
            "--title",
            "Not live",
            "--description",
            "A proposed rule",
            "--predicate",
            r#"{"type":"grep_pattern","pattern":"TODO","context_lines":0,"file_context":null}"#,
        ],
    );
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));

    let output = run(dir.path(), &["rule", "list", "--json"]);
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).expect("valid rule JSON");
    assert_eq!(value["rules"].as_array().expect("rules array").len(), 0);
}

#[test]
fn duplicate_rule_id_is_rejected() {
    // Falsifies the RuleAlreadyExists guard, which stops an existing immutable
    // rule from being overwritten.
    let dir = temp_cwd();
    let args = [
        "rule",
        "add",
        "--id",
        "l13-duplicate",
        "--domain",
        "testing",
        "--tier",
        "3",
        "--severity",
        "BLOCK",
        "--title",
        "Duplicate",
        "--description",
        "Duplicate rule",
        "--predicate",
        r#"{"type":"grep_pattern","pattern":"TODO","context_lines":0,"file_context":null}"#,
    ];
    let first = run(dir.path(), &args);
    assert!(first.status.success(), "stderr: {}", String::from_utf8_lossy(&first.stderr));
    let second = run(dir.path(), &args);
    assert!(!second.status.success());
    assert!(String::from_utf8_lossy(&second.stderr).contains("l13-duplicate"));
}

#[test]
fn invalid_predicate_writes_nothing() {
    // Falsifies a reordering that would insert the row before validating the
    // predicate.
    let dir = temp_cwd();
    let output = run(
        dir.path(),
        &[
            "rule",
            "add",
            "--id",
            "l13-invalid",
            "--domain",
            "testing",
            "--tier",
            "2",
            "--severity",
            "WARN",
            "--title",
            "Invalid",
            "--description",
            "Invalid predicate",
            "--predicate",
            r#"{"type":"nonsense"}"#,
        ],
    );
    assert!(!output.status.success());

    let output = run(dir.path(), &["rule", "list", "--all", "--json"]);
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).expect("valid rule JSON");
    assert_eq!(value["rules"].as_array().expect("rules array").len(), 0);
}

#[test]
fn tier_out_of_range_is_rejected() {
    // Falsifies the parser range check.
    let dir = temp_cwd();
    let output = run(
        dir.path(),
        &[
            "rule",
            "add",
            "--id",
            "l13-tier-range",
            "--domain",
            "testing",
            "--tier",
            "9",
            "--severity",
            "INFO",
            "--title",
            "Bad tier",
            "--description",
            "Out of range",
            "--predicate",
            r#"{"type":"grep_pattern","pattern":"TODO","context_lines":0,"file_context":null}"#,
        ],
    );
    assert!(!output.status.success());
}


fn add_grep_rule(cwd: &std::path::Path, id: &str) {
    let output = run(cwd, &["rule", "add", "--id", id, "--domain", "testing", "--tier", "2", "--severity", "WARN", "--title", "Test rule", "--description", "A test rule", "--predicate", r#"{"type":"grep_pattern","pattern":"TODO","context_lines":0,"file_context":null}"#]);
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
}

#[test]
fn rule_test_fires_on_matching_fixture() {
    let dir = temp_cwd();
    add_grep_rule(dir.path(), "l13-test-fires");
    std::fs::write(dir.path().join("fixture.rs"), "// TODO\n").unwrap();
    let output = run(dir.path(), &["rule", "test", "l13-test-fires", "fixture.rs", "--json"]);
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["fired"], true);
}

#[test]
fn rule_test_does_not_fire_on_clean_fixture() {
    let dir = temp_cwd();
    add_grep_rule(dir.path(), "l13-test-clean");
    std::fs::write(dir.path().join("fixture.rs"), "fn main() {}\n").unwrap();
    let output = run(dir.path(), &["rule", "test", "l13-test-clean", "fixture.rs", "--json"]);
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["fired"], false);
}

#[test]
fn rule_test_graph_reachability_without_index_refuses() {
    let dir = temp_cwd();
    let output = run(dir.path(), &["rule", "add", "--id", "l13-test-graph", "--domain", "testing", "--tier", "2", "--severity", "WARN", "--title", "Graph", "--description", "Graph rule", "--predicate", r#"{"type":"graph_reachability","source_pattern":"a","sink_pattern":"b","must_traverse":null,"must_not_exist":false,"max_depth":3,"edge_types":["calls"]}"#]);
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    std::fs::write(dir.path().join("fixture.rs"), "fn a() {}\n").unwrap();
    let output = run(dir.path(), &["rule", "test", "l13-test-graph", "fixture.rs"]);
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("GraphReachability"));
    assert!(stderr.contains("elai index"));
}

#[test]
fn rule_test_graph_reachability_with_unopenable_index_refuses() {
    let dir = temp_cwd();
    let output = run(dir.path(), &["rule", "add", "--id", "l13-test-graph-unopenable", "--domain", "testing", "--tier", "2", "--severity", "WARN", "--title", "Graph", "--description", "Graph rule", "--predicate", r#"{"type":"graph_reachability","source_pattern":"a","sink_pattern":"b","must_traverse":null,"must_not_exist":false,"max_depth":3,"edge_types":["calls"]}"#]);
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    std::fs::create_dir_all(dir.path().join(".elai_cc/indexes/dkg.db")).unwrap();
    std::fs::write(dir.path().join("fixture.rs"), "fn a() {}\n").unwrap();
    let output = run(dir.path(), &["rule", "test", "l13-test-graph-unopenable", "fixture.rs"]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("GraphReachability"));
}

#[test]
fn rule_test_unknown_rule_id_fails() {
    let dir = temp_cwd();
    std::fs::write(dir.path().join("fixture.rs"), "fn main() {}\n").unwrap();
    let output = run(dir.path(), &["rule", "test", "no-such-rule", "fixture.rs"]);
    assert!(!output.status.success());
}

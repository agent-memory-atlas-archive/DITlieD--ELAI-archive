//! N-1 measurement-spine index reachability smoke tests.

use std::process::Command;

fn run(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_elai"))
        .args(args)
        .output()
        .expect("elai binary should run")
}

#[test]
fn index_text_reaches_registry_and_routes() {
    // Falsifies missing args variant, parser arm, main dispatch, or registry wiring.
    let output = run(&["bench", "index"]);
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8_lossy(&output.stdout);
    for value in [
        "context-fill", "fill_degradation_curve", "calibrated_trim_fraction",
        "positional-probe", "positional_value_curve", "ordering_seed", "LIVE",
    ] {
        assert!(stdout.contains(value), "missing {value} in {stdout}");
    }
}

#[test]
fn index_json_reaches_registry() {
    // Falsifies the production JSON dispatch branch or loss of either registry row.
    let output = run(&["bench", "index", "--json"]);
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).expect("valid index JSON");
    let entries = value["entries"].as_array().expect("entries array");
    assert_eq!(entries.len(), 2);
    let text = value.to_string();
    assert!(text.contains("context-fill"));
    assert!(text.contains("positional-probe"));
}

#[test]
fn index_rejects_unknown_flag() {
    // Falsifies the parse_bench_index_args rejection path.
    let output = run(&["bench", "index", "--bogus"]);
    assert!(!output.status.success());
}

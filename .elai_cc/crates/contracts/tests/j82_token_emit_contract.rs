//! J-82 production token-emission contracts.
//!
//! These tests call the public production APIs directly; they do not use source
//! text as a pass criterion.  Environment assertions are read-only because
//! integration tests can run concurrently in one process.
//!
//! WIRED[J-82] production callers:
//! - `crates/cli/src/main.rs` `CliAction::BenchToolOutput` dispatches to the
//!   production `elai bench tool-output` path.
//! - `crates/cli/src/bench_tool_output.rs` calls
//!   `token_emit::build_tool_output_bench_report`.

use elai_orchestrator::token_emit::{
    build_tool_output_bench_report, collapse_log_stream, emit_token_efficient,
    format_emit_enabled, parse_token_efficient, strip_prose_fillers,
    structured_spans_preserved, LogDictReject, VolatileArchive,
};
use serde_json::json;

#[test]
fn j82_protected_span_preserves_fence_paths_and_identifiers() {
    let input = "Remove the filler around /var/lib/elai/run_42 and crate::token_emit::emit_token_efficient.\n```rust\nlet stable_path = \"/tmp/output_42\";\n```\n";
    let stripped = strip_prose_fillers(input);
    assert!(structured_spans_preserved(input, &stripped));
    assert!(stripped.contains("/var/lib/elai/run_42"));
    assert!(stripped.contains("crate::token_emit::emit_token_efficient"));
    assert!(stripped.contains("```rust\nlet stable_path = \"/tmp/output_42\";\n```"));
}

#[test]
fn j82_format_round_trips_nested_object() {
    let value = json!({
        "outer": {"items": [{"name": "alpha", "enabled": true}, {"name": "beta", "count": 2}]},
        "metadata": {"source": "contract", "tags": ["j82", "deterministic"]}
    });
    let encoded = emit_token_efficient(&value);
    assert_eq!(parse_token_efficient(&encoded).expect("production format must parse"), value);
}

#[test]
fn j82_log_collapse_refuses_unarchived_volatiles() {
    let stream = "2026-01-01 event=started\n2026-01-01 event=started";
    assert_eq!(collapse_log_stream(stream, None), Err(LogDictReject::VolatilesNotArchived));
}

#[test]
fn j82_log_collapse_rejects_code_stream() {
    let code = "fn emit(value: u64) -> u64 { value + 1 }";
    assert_eq!(collapse_log_stream(code, None), Err(LogDictReject::CodeStream));
}

#[test]
fn j82_bench_report_is_deterministic_and_default_off() {
    let raw = r#"{"z":{"b":2,"a":[true,"x"]},"a":1}"#;
    let first = build_tool_output_bench_report(raw, 1234);
    let second = build_tool_output_bench_report(raw, 1234);
    assert_eq!(serde_json::to_value(&first).unwrap(), serde_json::to_value(&second).unwrap());
    assert!(first.measure_first_guard.contains("MEASURE-FIRST"));
    assert!(first.legs_default_off, "optional emission legs must default OFF");
}

#[test]
fn j82_format_emit_is_disabled_when_environment_is_unset() {
    // Read-only assertion: this test intentionally does not set or remove an
    // environment variable, avoiding process-wide races with other tests.
    assert!(!format_emit_enabled(), "ELAI_TOKEN_EMIT_FORMAT must be unset");
}

#[test]
fn j82_archived_volatiles_are_accepted_for_log_collapse() {
    let mut archive = VolatileArchive { entries: Vec::new() };
    let collapsed = collapse_log_stream(
        "2026-01-01 event=started\n2026-01-01 event=started",
        Some(&mut archive),
    )
    .expect("archived volatile values are safe to collapse");
    assert!(collapsed.contains("$V0 (x2)"));
    assert_eq!(archive.entries.len(), 1);
}

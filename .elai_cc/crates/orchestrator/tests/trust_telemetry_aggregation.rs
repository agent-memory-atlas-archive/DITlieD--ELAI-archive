//! E-23 -- trust telemetry aggregation contract test.
//!
//! Drives [`elai_orchestrator::trust_telemetry::aggregate_session`] with a
//! synthetic row set whose counts are known by hand, then asserts that the
//! returned [`elai_orchestrator::trust_telemetry::TrustReport`] matches
//! the expected numerator / denominator math. This is the "synthetic SSB
//! row set with known counts; aggregate matches hand-computed" test the
//! E-23 plan body Task 2 calls out.
//!
//! NOTE: cargo runs tests within one test binary on parallel threads. The
//! gate check goes through a process-global env var, so we serialize the
//! gate-flip via a Mutex shared by every test in this file. Without the
//! lock, an `enable_gate` in one thread would race a `disable_gate` in
//! another and the empty-report assertion would flap.

use std::sync::Mutex;

use elai_orchestrator::trust_telemetry::{
    aggregate_session, TrustReport, TrustRowInput,
};

static GATE_LOCK: Mutex<()> = Mutex::new(());

fn row(action: &str, payload: serde_json::Value) -> TrustRowInput {
    TrustRowInput {
        action_type: action.to_owned(),
        ts_ms: 0,
        payload_json: payload,
    }
}

fn enable_gate() {
    std::env::set_var("ELAI_DEVELOPER_DISCOURSE_ENABLE", "1");
}

fn disable_gate() {
    std::env::remove_var("ELAI_DEVELOPER_DISCOURSE_ENABLE");
}

#[test]
fn aggregate_empty_when_gate_off() {
    let _lock = GATE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    disable_gate();
    let rows = vec![row(
        "phase_transition",
        serde_json::json!({"to": "COMPLETE"}),
    )];
    let r = aggregate_session(&rows, 2.0);
    assert_eq!(r, TrustReport::empty());
}

#[test]
fn hand_computed_counts_match() {
    let _lock = GATE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    enable_gate();
    // 4 completions / 2 hours = 2.0 tasks_per_hour
    // 6 findings / 4 completions = 1.5 findings_per_task
    // 2 reverts / 8 file_writes = 0.25 revert_rate
    // 3 ungrounded claims
    // 2 spinning + 1 oscillation stagnation rows
    let mut rows: Vec<TrustRowInput> = Vec::new();
    for _ in 0..4 {
        rows.push(row(
            "phase_transition",
            serde_json::json!({"to": "COMPLETE"}),
        ));
    }
    for _ in 0..6 {
        rows.push(row("finding", serde_json::json!({})));
    }
    for _ in 0..6 {
        rows.push(row("file_write", serde_json::json!({"revert": false})));
    }
    for _ in 0..2 {
        rows.push(row("file_write", serde_json::json!({"revert": true})));
    }
    for _ in 0..3 {
        rows.push(row("claim", serde_json::json!({"grounding": ""})));
    }
    rows.push(row("claim", serde_json::json!({"grounding": "file:foo"})));
    for _ in 0..2 {
        rows.push(row(
            "stagnation_spinning_detected",
            serde_json::json!({"pattern": "spinning"}),
        ));
    }
    rows.push(row(
        "stagnation_oscillation_detected",
        serde_json::json!({"pattern": "oscillation"}),
    ));

    let r = aggregate_session(&rows, 2.0);
    disable_gate();

    assert!((r.tasks_per_hour - 2.0).abs() < f64::EPSILON);
    assert!((r.findings_per_task - 1.5).abs() < f64::EPSILON);
    assert!((r.revert_rate - 0.25).abs() < f64::EPSILON);
    assert_eq!(r.ungrounded_count, 3);
    assert_eq!(r.stagnation_dist.get("spinning"), Some(&2));
    assert_eq!(r.stagnation_dist.get("oscillation"), Some(&1));
    // canonical seed labels remain so the dashboard renders a stable grid
    assert_eq!(r.stagnation_dist.get("no_drift"), Some(&0));
    assert_eq!(r.stagnation_dist.get("diminishing_returns"), Some(&0));
}

#[test]
fn unknown_action_types_are_ignored() {
    let _lock = GATE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    enable_gate();
    let rows = vec![
        row("tool_result", serde_json::json!({})),
        row("checkpoint", serde_json::json!({})),
        row("file_read", serde_json::json!({})),
    ];
    let r = aggregate_session(&rows, 1.0);
    disable_gate();
    assert_eq!(r.tasks_per_hour, 0.0);
    assert_eq!(r.findings_per_task, 0.0);
    assert_eq!(r.revert_rate, 0.0);
    assert_eq!(r.ungrounded_count, 0);
}

#[test]
fn completion_zero_findings_zero() {
    let _lock = GATE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    enable_gate();
    // 0 completions guarantees findings_per_task = 0 even when findings exist
    // (avoids divide-by-zero).
    let rows = vec![row("finding", serde_json::json!({}))];
    let r = aggregate_session(&rows, 1.0);
    disable_gate();
    assert_eq!(r.findings_per_task, 0.0);
}

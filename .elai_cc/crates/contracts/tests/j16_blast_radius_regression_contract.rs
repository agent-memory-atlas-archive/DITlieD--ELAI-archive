//! J-16 — blast-radius / anti-regression gate contract tests.
//!
//! Covers the five properties the plan's observable_effect names explicitly,
//! through the crate's PUBLIC surface only:
//!   (a) R1: None diff = hard FAIL (gate returns Err, never Ok).
//!   (b) R2: unmappable diff (hunks present, git_diff None) = hard FAIL.
//!   (c) Empty diff (no changed_files AND no changed_symbols) = Ok (no-op
//!       transition, NOT a fail-closed).
//!   (d) flag_silent_deletion flags .rs hunks with negative net_delta.
//!   (e) SSB round-trip: BlastRadiusRegressionEvent action_type serializes
//!       and deserializes through the SsbActionType from_str/as_sql_str
//!       contract, and the 4-arm Kind enum round-trips through serde.

use elai_orchestrator::blast_radius::{
    BlastRadiusRegressionGate, CapturedDiff, HunkSummary,
};
use elai_orchestrator::evidence::{EvidenceError, EvidenceGate, TransitionContext};
use elai_orchestrator::fsm::FsmState;
use elai_orchestrator::ssb::{
    ssb_action_type_from_str_public, BlastRadiusRegressionKind,
    BlastRadiusRegressionPayload, SsbActionType, SsbPayload,
};

fn ctx_with_diff(diff: Option<CapturedDiff>) -> TransitionContext {
    TransitionContext {
        blast_radius_diff: diff,
        ..TransitionContext::default()
    }
}

// (a) R1 -- None diff = hard FAIL.
#[test]
fn none_diff_is_hard_fail() {
    let gate = BlastRadiusRegressionGate;
    let ctx = ctx_with_diff(None);
    let result = gate.check(&ctx);
    assert!(result.is_err(), "None diff must be a hard FAIL, never Ok");
    let err: EvidenceError = result.unwrap_err();
    assert_eq!(err.gate_name, "blast_radius_regression");
    assert!(
        err.missing_artifacts.iter().any(|a| a == "blast_radius_diff"),
        "missing_artifacts must name blast_radius_diff"
    );
}

// (b) R2 -- unmappable diff (hunks present, git_diff None) = hard FAIL.
#[test]
fn unmappable_diff_is_hard_fail() {
    let gate = BlastRadiusRegressionGate;
    let diff = CapturedDiff {
        changed_files: vec!["src/main.rs".into()],
        changed_symbols: vec!["src/main.rs".into()],
        hunks: vec![HunkSummary {
            file_path: "src/main.rs".into(),
            old_start: 1,
            old_lines: 10,
            new_start: 1,
            new_lines: 5,
            net_delta: -5,
        }],
        declared_scope: vec!["src/main.rs".into()],
        git_diff: None,
        project_dir: None,
    };
    let ctx = ctx_with_diff(Some(diff));
    let result = gate.check(&ctx);
    assert!(result.is_err(), "unmappable diff must be a hard FAIL");
    let err: EvidenceError = result.unwrap_err();
    assert!(
        err.missing_artifacts
            .iter()
            .any(|a| a == "blast_radius_diff_unmappable"),
        "missing_artifacts must name blast_radius_diff_unmappable"
    );
}

// (c) Empty diff = Ok (no-op transition, NOT fail-closed).
#[test]
fn empty_diff_is_ok_noop() {
    let gate = BlastRadiusRegressionGate;
    let ctx = ctx_with_diff(Some(CapturedDiff::default()));
    let result = gate.check(&ctx);
    assert!(result.is_ok(), "empty diff is a no-op transition, not a FAIL");
}

// (c2) Valid diff with git_diff present = Ok (OBSERVE-ONLY, GD-4).
#[test]
fn valid_diff_with_git_diff_is_ok_observe_only() {
    let gate = BlastRadiusRegressionGate;
    let diff = CapturedDiff {
        changed_files: vec!["src/main.rs".into()],
        changed_symbols: vec!["src/main.rs".into()],
        hunks: vec![HunkSummary {
            file_path: "src/main.rs".into(),
            old_start: 1,
            old_lines: 10,
            new_start: 1,
            new_lines: 12,
            net_delta: 2,
        }],
        declared_scope: vec!["src/main.rs".into()],
        git_diff: Some(serde_json::Value::Null),
        project_dir: None,
    };
    let ctx = ctx_with_diff(Some(diff));
    let result = gate.check(&ctx);
    assert!(
        result.is_ok(),
        "OBSERVE-ONLY gate returns Ok, does not hard-block"
    );
}

// (d) flag_silent_deletion flags .rs hunks with negative net_delta.
#[test]
fn flag_silent_deletion_catches_negative_rs_hunks() {
    let diff = CapturedDiff {
        hunks: vec![
            HunkSummary {
                file_path: "src/lib.rs".into(),
                old_start: 1,
                old_lines: 20,
                new_start: 1,
                new_lines: 10,
                net_delta: -10,
            },
            HunkSummary {
                file_path: "src/main.rs".into(),
                old_start: 1,
                old_lines: 5,
                new_start: 1,
                new_lines: 8,
                net_delta: 3,
            },
            HunkSummary {
                file_path: "src/utils.py".into(),
                old_start: 1,
                old_lines: 15,
                new_start: 1,
                new_lines: 5,
                net_delta: -10,
            },
        ],
        ..CapturedDiff::default()
    };
    let flagged = elai_orchestrator::blast_radius::flag_silent_deletion(&diff);
    assert_eq!(
        flagged.len(),
        1,
        "only .rs files with negative net_delta are flagged"
    );
    assert!(
        flagged[0].contains("src/lib.rs"),
        "the flagged hunk must be the .rs file with negative delta"
    );
    assert!(
        flagged[0].contains("net_delta=-10"),
        "the flag must carry the net_delta value"
    );
}

// (d2) check_diff_scope flags out-of-scope files.
#[test]
fn check_diff_scope_flags_out_of_scope() {
    let changed = vec!["src/main.rs".into(), "src/other/mod.rs".into()];
    let scope = vec!["src/main.rs".into()];
    let violations =
        elai_orchestrator::blast_radius::check_diff_scope(&changed, &scope);
    assert_eq!(violations.len(), 1, "one out-of-scope file flagged");
    assert_eq!(violations[0], "src/other/mod.rs");
}

// (d3) check_diff_scope with directory prefix match.
#[test]
fn check_diff_scope_directory_prefix_is_in_scope() {
    let changed = vec![
        "src/lib/mod.rs".into(),
        "src/lib/sub/deep.rs".into(),
    ];
    let scope = vec!["src/lib/".into()];
    let violations =
        elai_orchestrator::blast_radius::check_diff_scope(&changed, &scope);
    assert!(
        violations.is_empty(),
        "directory prefix match means all descendants are in-scope"
    );
}

// (d4) check_diff_scope with empty declared_scope = no violations (observe-only).
#[test]
fn check_diff_scope_empty_scope_is_no_violation() {
    let changed = vec!["src/main.rs".into()];
    let violations =
        elai_orchestrator::blast_radius::check_diff_scope(&changed, &[]);
    assert!(violations.is_empty(), "empty declared_scope = no violations");
}

// (e) SSB round-trip: action_type as_sql_str / from_str.
#[test]
fn ssb_action_type_round_trip() {
    let action =
        SsbActionType::BlastRadiusRegressionEvent(BlastRadiusRegressionKind::ConeComputed);
    let sql_str = action.as_sql_str();
    assert_eq!(
        sql_str, "blast_radius_regression_event",
        "as_sql_str must return the canonical SQL string"
    );
    let parsed = ssb_action_type_from_str_public("blast_radius_regression_event");
    assert!(
        parsed.is_ok(),
        "from_str must parse the canonical SQL string"
    );
    let parsed = parsed.unwrap();
    match parsed {
        SsbActionType::BlastRadiusRegressionEvent(_) => {}
        other => panic!("from_str returned wrong variant: {:?}", other),
    }
}

// (e2) Kind enum serde round-trip for all 4 arms.
#[test]
fn kind_enum_serde_round_trip_all_arms() {
    let arms = [
        BlastRadiusRegressionKind::ConeComputed,
        BlastRadiusRegressionKind::ScopeViolationFlagged,
        BlastRadiusRegressionKind::RegressionSetSelected,
        BlastRadiusRegressionKind::SilentDeletionFlagged,
    ];
    for arm in arms {
        let json = serde_json::to_string(&arm).expect("serialize");
        let back: BlastRadiusRegressionKind =
            serde_json::from_str(&json).expect("deserialize");
        assert_eq!(arm, back, "serde round-trip must preserve the arm");
    }
}

// (e3) Payload -> SsbPayload -> action_type consistency.
#[test]
fn payload_action_type_consistency() {
    let payload = BlastRadiusRegressionPayload {
        kind: BlastRadiusRegressionKind::ScopeViolationFlagged,
        changed_symbols: vec!["src/main.rs".into()],
        cone: vec!["src/main.rs".into()],
        scope_violations: vec!["src/other.rs".into()],
        regression_set: Vec::new(),
        silent_deletions: Vec::new(),
        cone_truncated: false,
        session_id: "s-j16-test".into(),
        ts_ms: 1_700_000_000_000,
    };
    let ssb_payload = SsbPayload::BlastRadiusRegressionEvent(payload);
    let action = ssb_payload.action_type();
    assert_eq!(
        action.as_sql_str(),
        "blast_radius_regression_event",
        "payload action_type must be blast_radius_regression_event"
    );
    match action {
        SsbActionType::BlastRadiusRegressionEvent(kind) => {
            assert_eq!(
                kind,
                BlastRadiusRegressionKind::ScopeViolationFlagged,
                "kind must round-trip through the payload"
            );
        }
        other => panic!("wrong action_type variant: {:?}", other),
    }
}

// (f) Gate transition is EXECUTING -> AUDITING.
#[test]
fn gate_transition_is_executing_to_auditing() {
    let gate = BlastRadiusRegressionGate;
    let (from, to) = gate.transition();
    assert_eq!(from, FsmState::Executing);
    assert_eq!(to, FsmState::Auditing);
}

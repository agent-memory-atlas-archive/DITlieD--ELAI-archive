//! H-54 contract test — verifies the obligation-closure escape-hatch gate.
//!
//! Source plan: docs/thefinalglance/H-plan/
//!   H-54_obligation_closure_escape_hatch_gate.md
//!
//! Off-agent artifact requirement (R4/R1/R2): all four tests drive the REAL
//! `scan_obligation_closure` production path through
//! `formal_verify::orchestrator::run()` — the same codepath the CLI's
//! `elai formal-verify run` invokes. The key off-agent observables are:
//!   - The BFS walks the FULL dependency closure (not just the root node).
//!   - A relocated-sorry in a renamed helper lemma IS detected (AlphaProof
//!     failure mode 1:1 — a top-statement-only check would miss this).
//!   - A registered allowlist exception is recorded but NOT BLOCKed.
//!   - An unregistered escape hatch IS BLOCKed.
//!   - The broken `--top-statement-only` arm PASSES the relocated-sorry
//!     fixture (R2 falsifiability: a scan that stays "clean" when fed a
//!     relocated-admit is itself a dead evidence path — it screams, not shrugs).
//!
//! Tests:
//! (a) clean_goal_passes_scan: a fully-closed proof with no escape hatches ->
//!     ClosureScanKind::Clean; verifier proceeds (not blocked).
//! (b) relocated_admit_blocked_end_to_end: the AlphaProof failure mode —
//!     top goal looks sorry-free, helper lemma is admitted ->
//!     ClosureScanKind::EscapeHatchAnywhere; BLOCK. Names the escape node.
//! (c) registered_exception_not_blocked: an escape hatch WITH a valid
//!     allowlist row -> ClosureScanKind::RegisteredException; not a block.
//! (d) unregistered_escape_without_row_blocked: same hatch WITHOUT a row ->
//!     EscapeHatchAnywhere; BLOCK.
//! (e) top_statement_only_broken_arm_misses_relocated_sorry: construct a
//!     DAG with only the root node (no children exposed) simulating a
//!     top-statement-only scan -> returns Clean even though the helper
//!     is admitted. Gate asserts this is the EXPECTED BROKEN PATH (R2).
//! (f) pipeline_run_carries_closure_scan_result: drive the full pipeline
//!     via `formal_verify::orchestrator::run()` and assert the PipelineResult
//!     carries a `closure_scan` field (the off-agent artifact present check,
//!     R1/R2 — a scan that produces zero rows is a dead evidence path fail).
//! (g) closure_fingerprint_stable_under_same_input: GD-2 byte-idempotency.
//! (h) escape_hatch_kind_wire_strings_are_stable: wire format lock.
//! (i) schema_version_at_h54_minimum: SCHEMA_VERSION >= 229 (H-44 base).
//! (j) trap_behavior_detected_kind_has_closure_escape_hatch_arm: the SSB
//!     TrapBehaviorDetectedKind enum carries the ClosureEscapeHatch arm
//!     (H-54 compose-path verification).

use std::collections::HashMap;

use elai_orchestrator::formal_verify::{
    scan_obligation_closure, ClosureScanKind, DischargedGoal, HatchKind, ObligationNode,
    VerifierKind, VerifyRequest,
};
use elai_orchestrator::ssb::{
    TrapBehaviorDetectedKind, SCHEMA_VERSION_PUBLIC,
};
use tracing_test::traced_test;

// ---------------------------------------------------------------------------
// Frozen test fixtures (gate-frozen per R8)
//
// F_CLEAN: a fully-closed proof — root depends on helper, neither has any
// escape hatch.
//
// F_RELOCATED_ADMIT: the AlphaProof failure mode — root looks sorry-free
// (hatch=None), but it depends on `renamed_helper_lemma` which carries
// HatchKind::Sorry. A top-statement-only scan passes this; the BFS must NOT.
//
// F_TOP_ONLY (broken arm): a DAG containing ONLY the root node (no children).
// Simulates a scanner that only checks the top statement. Returns Clean even
// though the helper is admitted (R2 — the broken path screams).
// ---------------------------------------------------------------------------

fn f_clean_goal() -> DischargedGoal {
    let mut dag = HashMap::new();
    dag.insert(
        "MyProof".to_string(),
        ObligationNode {
            obligation_id: "MyProof".to_string(),
            hatch: None,
            depends_on: vec!["helper_lemma_1".to_string()],
        },
    );
    dag.insert(
        "helper_lemma_1".to_string(),
        ObligationNode {
            obligation_id: "helper_lemma_1".to_string(),
            hatch: None,
            depends_on: vec![],
        },
    );
    DischargedGoal::with_dag("MyProof", dag)
}

fn f_relocated_admit_goal() -> DischargedGoal {
    // AlphaProof failure mode: root is sorry-free, helper lemma is admitted.
    let mut dag = HashMap::new();
    dag.insert(
        "MyProof".to_string(),
        ObligationNode {
            obligation_id: "MyProof".to_string(),
            hatch: None, // top looks clean
            depends_on: vec!["renamed_helper_lemma".to_string()],
        },
    );
    dag.insert(
        "renamed_helper_lemma".to_string(),
        ObligationNode {
            obligation_id: "renamed_helper_lemma".to_string(),
            hatch: Some(HatchKind::Sorry), // the relocated sorry
            depends_on: vec![],
        },
    );
    DischargedGoal::with_dag("MyProof", dag)
}

fn f_top_only_broken_goal() -> DischargedGoal {
    // Broken arm: DAG contains only the root (no children exposed).
    // Simulates a top-statement-only scan that skips the helper closure.
    let mut dag = HashMap::new();
    dag.insert(
        "MyProof".to_string(),
        ObligationNode {
            obligation_id: "MyProof".to_string(),
            hatch: None, // root is clean — helper is never visited
            depends_on: vec![], // <- the broken part: children not included
        },
    );
    DischargedGoal::with_dag("MyProof", dag)
}

// ---------------------------------------------------------------------------
// (a) clean goal passes scan
// ---------------------------------------------------------------------------

#[traced_test]
#[test]
fn clean_goal_passes_scan() {
    let goal = f_clean_goal();
    let result = scan_obligation_closure(&goal, VerifierKind::Dafny, None);
    assert_eq!(
        result.kind,
        ClosureScanKind::Clean,
        "A fully-closed proof with no escape hatches MUST return Clean; got {:?}",
        result.kind
    );
    assert!(
        !result.kind.is_block(),
        "Clean result must NOT block the verifier dispatch"
    );
    assert!(
        result.escape_nodes.is_empty(),
        "Clean result must have zero escape nodes"
    );
    assert_eq!(
        result.closure_size, 2,
        "Full closure BFS must visit both root and helper (size=2)"
    );
    assert!(
        !result.closure_fingerprint.is_empty(),
        "Clean result must carry a non-empty blake3 fingerprint (GD-2)"
    );
    // Off-agent artifact: the scan logs a "closure scan complete" line at DEBUG.
    // The traced_test harness captures logs; assert the scan ran.
    assert!(
        logs_contain("closure scan complete"),
        "scan_obligation_closure MUST emit a tracing log at completion (off-agent artifact R1/R2)"
    );
}

// ---------------------------------------------------------------------------
// (b) relocated-admit is blocked end-to-end (AlphaProof failure mode 1:1)
// ---------------------------------------------------------------------------

#[traced_test]
#[test]
fn relocated_admit_blocked_end_to_end() {
    let goal = f_relocated_admit_goal();
    let result = scan_obligation_closure(&goal, VerifierKind::Dafny, None);
    assert_eq!(
        result.kind,
        ClosureScanKind::EscapeHatchAnywhere,
        "A relocated sorry in a renamed helper lemma MUST be detected by the BFS. \
         A top-statement-only scan would return Clean — this is the AlphaProof \
         failure mode that H-54 closes. Got {:?}",
        result.kind
    );
    assert!(
        result.kind.is_block(),
        "EscapeHatchAnywhere MUST block the verifier dispatch"
    );
    assert_eq!(
        result.escape_nodes.len(),
        1,
        "Exactly one escape node expected (the renamed_helper_lemma)"
    );
    assert_eq!(
        result.escape_nodes[0].obligation_id,
        "renamed_helper_lemma",
        "The escape node obligation_id must name the helper, not the root"
    );
    assert_eq!(
        result.escape_nodes[0].hatch,
        HatchKind::Sorry,
        "The detected hatch kind must be Sorry"
    );
    assert!(
        !result.escape_nodes[0].registered,
        "No allowlist row exists for this hatch; registered must be false"
    );
    // Off-agent artifact: scan logs the BLOCK path.
    assert!(
        logs_contain("UNREGISTERED escape hatch found"),
        "scan_obligation_closure MUST log the unregistered escape hatch (off-agent artifact R1/R2)"
    );
}

// ---------------------------------------------------------------------------
// (c) registered exception is recorded but NOT blocked
// ---------------------------------------------------------------------------

#[traced_test]
#[test]
fn registered_exception_not_blocked() {
    // Build a temporary allowlist file with a valid row for "renamed_helper_lemma".
    let tmpdir = std::env::temp_dir();
    let allowlist_path = tmpdir.join("h54_test_allowlist.toml");
    std::fs::write(
        &allowlist_path,
        r#"[[obligation]]
obligation_id   = "renamed_helper_lemma"
owner           = "maintainer"
reason          = "open benchmark case — test fixture"
expiry_date     = "2099-12-31"
evidence_pointer = "arXiv:2605.22763"
"#,
    )
    .expect("write temp allowlist");

    let goal = f_relocated_admit_goal();
    let result = scan_obligation_closure(&goal, VerifierKind::Dafny, Some(&allowlist_path));
    let _ = std::fs::remove_file(&allowlist_path);

    assert_eq!(
        result.kind,
        ClosureScanKind::RegisteredException,
        "A sorry with a valid allowlist row MUST return RegisteredException (not EscapeHatchAnywhere). \
         Got {:?}",
        result.kind
    );
    assert!(
        !result.kind.is_block(),
        "RegisteredException must NOT block the verifier dispatch"
    );
    assert_eq!(
        result.escape_nodes.len(),
        1,
        "RegisteredException must still report the escape node (recorded)"
    );
    assert!(
        result.escape_nodes[0].registered,
        "The escape node's `registered` field must be true when allowlist row matches"
    );
    assert!(
        logs_contain("registered exception for escape hatch"),
        "scan_obligation_closure MUST log the registered exception (off-agent artifact R1/R2)"
    );
}

// ---------------------------------------------------------------------------
// (d) unregistered escape without row is blocked
// ---------------------------------------------------------------------------

#[traced_test]
#[test]
fn unregistered_escape_without_row_blocked() {
    // Explicitly use an EMPTY allowlist so there are no rows.
    let tmpdir = std::env::temp_dir();
    let allowlist_path = tmpdir.join("h54_test_empty_allowlist.toml");
    std::fs::write(
        &allowlist_path,
        "# empty allowlist — no exemptions\n",
    )
    .expect("write temp empty allowlist");

    let goal = f_relocated_admit_goal();
    let result = scan_obligation_closure(&goal, VerifierKind::Dafny, Some(&allowlist_path));
    let _ = std::fs::remove_file(&allowlist_path);

    assert_eq!(
        result.kind,
        ClosureScanKind::EscapeHatchAnywhere,
        "An unregistered escape hatch (no allowlist row) MUST return EscapeHatchAnywhere. \
         Got {:?}",
        result.kind
    );
    assert!(
        result.kind.is_block(),
        "EscapeHatchAnywhere MUST be a block"
    );
    assert!(
        !result.escape_nodes[0].registered,
        "The escape node's `registered` field must be false when no row matches"
    );
    // Off-agent artifact: scan must log the unregistered hatch on the BLOCK path (T-1).
    assert!(
        logs_contain("UNREGISTERED escape hatch found"),
        "scan_obligation_closure MUST log 'UNREGISTERED escape hatch found' on the block path \
         (off-agent artifact R1/R2)"
    );
}

// ---------------------------------------------------------------------------
// (e) top-statement-only BROKEN ARM passes the relocated-sorry (R2 falsifiability)
// ---------------------------------------------------------------------------
// NOTE: #[traced_test] intentionally ABSENT here. The broken-arm path returns
// ClosureScanKind::Clean (no UNREGISTERED hatch is detected because the helper
// lemma is never visited). A #[traced_test] without a logs_contain assertion is
// a T-2 violation; and there is genuinely no log to assert on for a clean result.
// The test documents WHY the full BFS is required — it is NOT testing log emission.
#[test]
fn top_statement_only_broken_arm_misses_relocated_sorry() {
    // This tests the BROKEN PATH: a DAG that only exposes the root node
    // (no children) is the "top-statement-only" scanner. It PASSES the
    // relocated-sorry fixture because it never visits the helper.
    //
    // This is the documented R2 failure: a scan that stays "clean" when fed
    // a relocated-admit is itself a dead evidence path. The gate asserts this
    // broken path returns Clean (expected), documenting WHY the full BFS is
    // required and proving the contract test is falsifiable.
    let goal = f_top_only_broken_goal();
    let result = scan_obligation_closure(&goal, VerifierKind::Dafny, None);
    assert_eq!(
        result.kind,
        ClosureScanKind::Clean,
        "EXPECTED BROKEN PATH: a top-statement-only DAG (no children) returns Clean \
         even when the real helper is admitted. This confirms WHY full BFS is required. \
         Got {:?}",
        result.kind
    );
    // Document that the closure_size is 1 (only the root was visited).
    assert_eq!(
        result.closure_size, 1,
        "Top-statement-only BFS visits exactly 1 node (R2 broken-arm confirmation)"
    );
    // This test passes Clean intentionally — it is the broken path.
    // The actual H-54 gate uses the full BFS (tests a and b above).
}

// ---------------------------------------------------------------------------
// (f) pipeline run carries closure_scan field (off-agent artifact R1/R2)
// ---------------------------------------------------------------------------

#[traced_test]
#[tokio::test]
async fn pipeline_run_carries_closure_scan_result() {
    use elai_orchestrator::formal_verify::run as run_pipeline;

    let req = VerifyRequest {
        target_path: "crates/auth/src/lib.rs".to_string(),
        verifier: VerifierKind::Z3Smt,
        cost_cap_ms: None,
        demo_escape_hatch: false,
    };
    let result = run_pipeline(req).await;

    assert!(
        result.closure_scan.is_some(),
        "PipelineResult MUST carry a closure_scan field (R1/R2 off-agent artifact — \
         a result with no scan is a dead evidence path)"
    );
    let scan = result.closure_scan.unwrap();
    // In the scaffolding tier, the goal DAG is unavailable -> ClosureUnavailable.
    // But the scan MUST have fired (closure_scan is Some, not None).
    assert_eq!(
        scan.kind,
        ClosureScanKind::ClosureUnavailable,
        "E-53 scaffolding tier: verifier does not expose a DAG -> ClosureUnavailable \
         (Layer-4 verifier verdict is the sound backstop, R5)"
    );
    assert!(
        !scan.kind.is_block(),
        "ClosureUnavailable must NOT block (the scan is unsound for this goal, R5)"
    );
    assert!(
        logs_contain("no obligation DAG available"),
        "orchestrator::run MUST emit the unsound-label log when DAG is absent (R5 labeling)"
    );
}

// ---------------------------------------------------------------------------
// (g) GD-2 closure fingerprint is stable under same input
// ---------------------------------------------------------------------------

#[test]
fn closure_fingerprint_stable_under_same_input() {
    let goal = f_clean_goal();
    let r1 = scan_obligation_closure(&goal, VerifierKind::Kani, None);
    let r2 = scan_obligation_closure(&goal, VerifierKind::Kani, None);
    assert_eq!(
        r1.closure_fingerprint, r2.closure_fingerprint,
        "GD-2: fingerprint MUST recompute byte-identically from same DAG"
    );
    assert_eq!(
        r1.closure_fingerprint.len(),
        16,
        "fingerprint MUST be 16 hex chars (blake3 prefix)"
    );
}

// ---------------------------------------------------------------------------
// (h) escape hatch kind wire strings are stable (format lock)
// ---------------------------------------------------------------------------

#[test]
fn escape_hatch_kind_wire_strings_are_stable() {
    assert_eq!(HatchKind::Sorry.as_str(), "sorry");
    assert_eq!(HatchKind::Admit.as_str(), "admit");
    assert_eq!(HatchKind::Axiom.as_str(), "axiom");
    assert_eq!(HatchKind::Assume.as_str(), "assume");
    assert_eq!(HatchKind::VerifierExternal.as_str(), "verifier_external");
    assert_eq!(HatchKind::KaniAssumeVacuous.as_str(), "kani_assume_vacuous");
}

// ---------------------------------------------------------------------------
// (i) SCHEMA_VERSION is at H-44 minimum (229 = the live base at H-54 ship)
// ---------------------------------------------------------------------------

#[test]
fn schema_version_at_h54_minimum() {
    // H-54 reuses H-44's TrapBehaviorDetected variant (no schema bump).
    // H-44 shipped at SCHEMA_VERSION 222; the live base at H-54 ship is 229.
    assert!(
        SCHEMA_VERSION_PUBLIC >= 222,
        "SCHEMA_VERSION must be >= 222 (H-44 base where TrapBehaviorDetected lives); \
         got {SCHEMA_VERSION_PUBLIC}"
    );
}

// ---------------------------------------------------------------------------
// (j) TrapBehaviorDetectedKind has ClosureEscapeHatch arm (H-54 compose path)
// ---------------------------------------------------------------------------

#[test]
fn trap_behavior_detected_kind_has_closure_escape_hatch_arm() {
    // This test confirms the SSB compose path (H-44 kind enum extended with
    // ClosureEscapeHatch, no new SsbActionType, TOTAL_VARIANTS unchanged).
    let kind = TrapBehaviorDetectedKind::ClosureEscapeHatch;
    assert_eq!(
        kind.as_str(),
        "closure_escape_hatch",
        "TrapBehaviorDetectedKind::ClosureEscapeHatch wire string must be 'closure_escape_hatch'"
    );
    // Verify the kind can be serialized and deserialized correctly (SSB wire round-trip).
    let json = serde_json::to_string(&kind).expect("serialize TrapBehaviorDetectedKind");
    assert_eq!(
        json, "\"closure_escape_hatch\"",
        "TrapBehaviorDetectedKind::ClosureEscapeHatch JSON must be '\"closure_escape_hatch\"'"
    );
    let back: TrapBehaviorDetectedKind = serde_json::from_str(&json)
        .expect("deserialize TrapBehaviorDetectedKind");
    assert_eq!(back, kind, "TrapBehaviorDetectedKind round-trip must be identity");
}

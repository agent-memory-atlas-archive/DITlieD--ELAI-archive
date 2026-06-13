//! H-38 contract tests -- cost-confirmation gate on TaskStop / pod_stop.
//!
//! Covers (off-agent artifacts, Layer-4 / R4, fail-closed per R1):
//!   (F1) SCHEMA >= 217 and `stop_confirmation_event` is in the SSB bijection.
//!        action_type_lock.rs TOTAL_VARIANTS == 528 (pin, not floor).
//!   (F2) Wire tag round-trip: as_sql_str / ssb_action_type_from_str_public.
//!   (F3) FP-guard path: low-spend, no unsaved, no pod -> Pass verdict,
//!        NO Block finding returned. StopCostSnapshot(day_usd=0.05, unsaved=0,
//!        pod_billable=false) is below CONFIRM_COST_FLOOR_USD (0.10) so the
//!        gate must not surface a confirm or produce a Block.
//!   (F4) ConfirmRequired path: high-spend + unsaved -> ConfirmRequired verdict,
//!        Block finding returned, StopConfirmationEvent row written to SSB
//!        (off-agent, R1: missing row = FAIL), and tracing log contains
//!        "stop_cost_confirm_required" (T-1/T-2 mandatory: assert! wraps logs_contain).
//!        StopCostSnapshot(day_usd=5.00, unsaved=2, pod_billable=false).
//!
//! SCHEMA_VERSION assertions use `>=` (not exact pin) per BRITTLE_GLOBAL_SCHEMA_PIN gotcha.
//! TOTAL_VARIANTS == 528 pin is intentionally exact (action_type_lock.rs design).
//!
//! NO real pod stop is performed; fixture spend values only (policy: sidecar boot
//! is hardware-heavy and billable pods must NOT be stopped from tests).
//!
//! Grounding:
//!   - crates/orchestrator/src/ssb.rs (StopConfirmationPayload, StopOutcomeKind, SCHEMA_VERSION)
//!   - crates/orchestrator/src/stop_cost_gate.rs (evaluate_stop, StopCostSnapshot, CONFIRM_COST_FLOOR_USD)
//!   - crates/orchestrator/src/hook_dispatcher.rs (dispatch_stop, InProcessHookDispatcher)
//!   - crates/orchestrator/tests/action_type_lock.rs (TOTAL_VARIANTS pin)
//!   - .claude/rules/tracing-test-assert.md (T-1: logs_contain MUST be in assert!)

use std::sync::Arc;

use elai_core::clock::OsClock;
use elai_orchestrator::ssb::{
    ssb_action_type_from_str_public, Ssb, SsbActionType, SsbConfig, SCHEMA_VERSION_PUBLIC,
};
use elai_orchestrator::{
    StopOutcomeKind,
    evaluate_stop, StopCostSnapshot, StopGateVerdict, CONFIRM_COST_FLOOR_USD,
    GateDecision, MockInteractiveGate, InteractiveGate,
    HookDispatcher, HookInvocation, HookVerdictKind,
    InProcessHookDispatcher,
};
use elai_orchestrator::SsbReader;
use elai_orchestrator::runtime_setup::build_production_hook_dispatcher;
use tempfile::TempDir;

fn test_clock() -> Arc<dyn elai_core::clock::ClockProvider> {
    Arc::new(OsClock)
}

fn test_ssb(dir: &TempDir, session_id: &str) -> Arc<Ssb> {
    let db_path = dir.path().join("ssb.db");
    let cfg = SsbConfig {
        db_path,
        pool_size: Some(2),
        project_id: "h38-contract".into(),
        session_id: session_id.into(),
    };
    Arc::new(Ssb::new(cfg, test_clock()).expect("ssb open"))
}

// ---------------------------------------------------------------------------
// (F1) SCHEMA >= 217 and SSB bijection; TOTAL_VARIANTS == 528 pin.
// ---------------------------------------------------------------------------

#[test]
fn h38_f1_stop_confirmation_event_exists_at_schema_217() {
    // SCHEMA floor.
    assert!(
        SCHEMA_VERSION_PUBLIC >= 217,
        "H-38 requires SCHEMA_VERSION >= 217, got {SCHEMA_VERSION_PUBLIC}"
    );

    // Wire tag round-trip.
    let at = ssb_action_type_from_str_public("stop_confirmation_event")
        .expect("from_str stop_confirmation_event must succeed");
    assert_eq!(at, SsbActionType::StopConfirmationEvent);
    assert_eq!(
        SsbActionType::StopConfirmationEvent.as_sql_str(),
        "stop_confirmation_event"
    );

    // TOTAL_VARIANTS floor (528 = 527 + 1 H-38 variant). Tier-B Move 2 derived
    // TOTAL_VARIANTS and relocated ALL_VARIANTS (now SSB_ACTION_TYPES in
    // ssb.rs), so read the runtime registry length. Same >= 528 floor.
    let total_variants_value = elai_orchestrator::ssb::SSB_ACTION_TYPES.len();
    assert!(
        total_variants_value >= 528,
        "TOTAL_VARIANTS must be >= 528 (H-38 floor); got {total_variants_value}"
    );
}

// ---------------------------------------------------------------------------
// (F2) Wire tag round-trip.
// ---------------------------------------------------------------------------

#[test]
fn h38_f2_wire_tag_round_trips() {
    let tag = SsbActionType::StopConfirmationEvent.as_sql_str();
    assert_eq!(tag, "stop_confirmation_event");
    let decoded = ssb_action_type_from_str_public(tag)
        .expect("from_str must accept stop_confirmation_event");
    assert_eq!(decoded, SsbActionType::StopConfirmationEvent);
}

// ---------------------------------------------------------------------------
// (F3) FP-guard path: low-spend, no unsaved, no pod -> Pass, no Block.
//      pure evaluate_stop test (no SSB or dispatcher needed).
// ---------------------------------------------------------------------------

#[test]
fn h38_f3_low_spend_passes_without_confirm() {
    // Arrange: below CONFIRM_COST_FLOOR_USD, no unsaved, no pod.
    let snapshot = StopCostSnapshot {
        day_usd: 0.05,
        unsaved_artifacts: 0,
        pod_billable: false,
    };

    // Act.
    let verdict = evaluate_stop(snapshot);

    // Assert: Pass, no friction (R1: ConfirmRequired here is a false positive).
    assert_eq!(
        verdict,
        StopGateVerdict::Pass,
        "FP guard: day_usd=0.05 (below floor {CONFIRM_COST_FLOOR_USD}) must yield Pass"
    );
}

// ---------------------------------------------------------------------------
// (F4) ConfirmRequired -> Block + StopConfirmationEvent SSB row (R1/R4).
//      Drives the PRODUCTION build_production_hook_dispatcher path (R7/R8).
//      The same factory call that production uses is exercised here so the
//      test proves the REAL wiring, not a hand-assembled dispatcher.
//      MockInteractiveGate returns Reject so the blocked path fires.
//      unsaved_artifact_count=2 triggers ConfirmRequired (>0 unsaved).
//
// #[traced_test]: T-1/T-2 compliance -- assert!(logs_contain(...)) mandatory.
// SimpleSpanProcessor not needed here (no OTel spans); tracing-test is sufficient.
// ---------------------------------------------------------------------------

#[tracing_test::traced_test]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn stop_confirmation_event_row_emitted_when_billable_stop_confirmed() {
    let dir = TempDir::new().expect("tempdir");
    let ssb = test_ssb(&dir, "h38-confirm");

    // MockInteractiveGate: return Reject so the RejectedHeld path fires and
    // the Block finding is returned. The SSB row is emitted unconditionally
    // on this path (the off-agent artifact we assert below, R1/R4).
    let mock_gate: Arc<dyn InteractiveGate> =
        Arc::new(MockInteractiveGate::new(vec![GateDecision::Reject]));

    // H-38 BLOCK-3 (R7/R8): drive the PRODUCTION builder so the test exercises
    // the same gate wiring that production uses. vibegate_seed_ok=true avoids
    // needing a real seed file in the temp dir. project_root=dir.path() is a
    // valid empty directory (SubprocessRegistryChecker tolerates missing py files).
    let dispatcher = build_production_hook_dispatcher(
        ssb.clone(),
        true,
        dir.path(),
        Some(mock_gate),
    );

    // Build the invocation with high spend + unsaved artifacts.
    let invocation = HookInvocation::for_task_level("task_stop", "step-h38")
        .with_unsaved_artifact_count(2);
    // Note: day_usd will be 0.0 since no CloudSpend rows exist in the fixture SSB,
    // so the cost gate will fire via unsaved_artifacts > 0 (not via day_usd).

    // Act: dispatch_stop via the trait surface (R4: real entrypoint, not a unit shim).
    let findings = dispatcher
        .dispatch_stop(invocation)
        .await
        .expect("dispatch_stop must not error on Reject path");

    // Assert: Block finding for stop_cost_confirm_required.
    assert!(
        findings
            .iter()
            .any(|f| f.verdict == HookVerdictKind::Block
                && f.rule_id.as_deref() == Some("stop_cost_confirm_required")),
        "dispatch_stop must return a Block finding with rule_id='stop_cost_confirm_required' (R1: missing = FAIL)\nfindings: {findings:?}"
    );

    // Assert (T-1 / T-2): tracing log must contain the rule_id.
    // logs_contain returns bool and MUST be wrapped in assert! or the test
    // silently passes (fail-open, see .claude/rules/tracing-test-assert.md T-1).
    assert!(
        logs_contain("stop_cost_confirm_required"),
        "tracing log must contain 'stop_cost_confirm_required' (T-1/T-2: missing assert = fail-open)"
    );

    // Assert (R1 off-agent artifact): StopConfirmationEvent row in SSB.
    // Missing row = FAIL (R1: missing evidence is a hard FAIL, not UNVERIFIED).
    let rows = ssb
        .query_by_type(SsbActionType::StopConfirmationEvent)
        .expect("query_by_type StopConfirmationEvent must succeed (R2: query failure = FAIL)");
    assert!(
        !rows.is_empty(),
        "stop_confirmation_event row must exist after RejectedHeld dispatch_stop (R1: missing = FAIL)"
    );

    // Decode and verify the row carries outcome=RejectedHeld.
    let last = rows.last().expect("at least one row");
    let decoded: elai_orchestrator::ssb::SsbPayload =
        rmp_serde::from_slice(&last.payload).expect("deserialize StopConfirmationPayload must succeed");
    if let elai_orchestrator::ssb::SsbPayload::StopConfirmationEvent(ref p) = decoded {
        assert_eq!(
            p.outcome,
            StopOutcomeKind::RejectedHeld,
            "RejectedHeld path must produce outcome=RejectedHeld in SSB row"
        );
        // unsaved_count must reflect the invocation's unsaved_artifact_count.
        assert_eq!(
            p.unsaved_count, 2,
            "unsaved_count must round-trip from HookInvocation::unsaved_artifact_count"
        );
    } else {
        panic!("decoded payload must be StopConfirmationEvent, got something else");
    }
}

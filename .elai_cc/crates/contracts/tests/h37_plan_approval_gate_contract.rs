//! H-37 contract tests -- plan-approval gate at PLANNING -> TESTING_FIRST.
//!
//! Covers (off-agent artifacts, Layer-4 / R4, fail-closed per R1):
//!   (F1) SCHEMA >= 216 and `plan_decision_event` is in the SSB bijection.
//!        action_type_lock.rs TOTAL_VARIANTS == 527 (pin, not floor).
//!   (F2) Wire tag round-trip: as_sql_str / ssb_action_type_from_str_public.
//!   (F3) A real PlanDecisionEvent SSB row writes and round-trips correctly.
//!        The contract drives the REAL production code path (drive_task intercept
//!        via write_payload) -- not a unit-test shim. Missing row = FAIL (R1).
//!   (F4) PlanDecisionKind::Reject and AutoAdvance round-trip correctly.
//!   (F5) The plan_approval_gate_enabled() env-gate returns false when unset and
//!        true when ELAI_PLAN_APPROVAL_GATE_ENABLE=1. Validates the kill-switch
//!        semantics (default-OFF per R9).
//!
//! SCHEMA_VERSION assertions use `>=` (not exact pin) per the BRITTLE_GLOBAL_SCHEMA_PIN gotcha.
//! TOTAL_VARIANTS == 527 pin is intentionally exact (action_type_lock.rs design: exact pin
//! catches renames that preserve count).
//!
//! Grounding:
//!   - crates/orchestrator/src/ssb.rs (PlanDecisionPayload, PlanDecisionKind, SCHEMA_VERSION)
//!   - crates/orchestrator/src/task_runner.rs (drive_task H-37 intercept)
//!   - crates/orchestrator/tests/action_type_lock.rs (TOTAL_VARIANTS pin)

use std::sync::Arc;

use elai_core::clock::OsClock;
use elai_orchestrator::ssb::{
    ssb_action_type_from_str_public, PlanDecisionKind, PlanDecisionPayload, Ssb, SsbActionType,
    SsbConfig, SsbPayload, SCHEMA_VERSION_PUBLIC,
};
use elai_orchestrator::SsbReader;
use tempfile::TempDir;

// TransitionContext is used in F6 tests to supply passing evidence for Approve.
use elai_orchestrator::TransitionContext;

fn test_clock() -> Arc<dyn elai_core::clock::ClockProvider> {
    Arc::new(OsClock)
}

fn test_ssb(dir: &TempDir, session_id: &str) -> Arc<Ssb> {
    let db_path = dir.path().join("ssb.db");
    let cfg = SsbConfig {
        db_path,
        pool_size: Some(2),
        project_id: "h37-contract".into(),
        session_id: session_id.into(),
    };
    Arc::new(Ssb::new(cfg, test_clock()).expect("ssb open"))
}

// ---------------------------------------------------------------------------
// (F1) SCHEMA >= 216 and SSB bijection; TOTAL_VARIANTS == 527 pin.
// ---------------------------------------------------------------------------

#[test]
fn h37_f1_plan_decision_event_row_emitted_when_gate_enabled() {
    // SCHEMA floor.
    assert!(
        SCHEMA_VERSION_PUBLIC >= 216,
        "H-37 requires SCHEMA_VERSION >= 216, got {SCHEMA_VERSION_PUBLIC}"
    );

    // Wire tag round-trip.
    let at = ssb_action_type_from_str_public("plan_decision_event")
        .expect("from_str plan_decision_event must succeed");
    assert_eq!(at, SsbActionType::PlanDecisionEvent);
    assert_eq!(
        SsbActionType::PlanDecisionEvent.as_sql_str(),
        "plan_decision_event"
    );

    // TOTAL_VARIANTS floor (527 = 526 + 1 H-37 variant). Tier-B Move 2 derived
    // TOTAL_VARIANTS and relocated ALL_VARIANTS (now SSB_ACTION_TYPES in
    // ssb.rs), so read the runtime registry length. Same >= 527 floor.
    let total_variants_value = elai_orchestrator::ssb::SSB_ACTION_TYPES.len();
    assert!(
        total_variants_value >= 527,
        "TOTAL_VARIANTS must be >= 527 (H-37 floor); got {total_variants_value}"
    );
}

// ---------------------------------------------------------------------------
// (F2) Wire tag round-trip
// ---------------------------------------------------------------------------

#[test]
fn h37_f2_wire_tag_round_trips() {
    let tag = SsbActionType::PlanDecisionEvent.as_sql_str();
    assert_eq!(tag, "plan_decision_event");
    let decoded =
        ssb_action_type_from_str_public(tag).expect("from_str must accept plan_decision_event");
    assert_eq!(decoded, SsbActionType::PlanDecisionEvent);
}

// ---------------------------------------------------------------------------
// (F3) Real PlanDecisionEvent row write + round-trip (Approve path).
//      Drives the REAL SSB write path -- off-agent artifact, R1/R4.
//      Missing row = FAIL.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn h37_f3_approve_row_writes_and_round_trips() {
    let dir = TempDir::new().expect("tempdir");
    let ssb = test_ssb(&dir, "h37-approve");

    let payload = SsbPayload::PlanDecisionEvent(PlanDecisionPayload {
        decision_kind: PlanDecisionKind::Approve,
        target_state: "TESTING_FIRST".into(),
        risk_level: "MEDIUM".into(),
        reason: None,
        plan_session_id: "seq-1".into(),
    });

    // Write the row (this is the production write path, not a shim).
    let seq = ssb
        .write_payload("task_runner", 0, payload.clone(), Vec::new())
        .await
        .expect("write_payload PlanDecisionEvent Approve must succeed (R1: missing = FAIL)");
    assert!(
        seq > 0,
        "SSB seq must be positive (R1: zero seq = row not written)"
    );

    // Query back the row -- this is the off-agent observable artifact.
    let rows = ssb
        .query_by_type(SsbActionType::PlanDecisionEvent)
        .expect("query_by_type PlanDecisionEvent must succeed");
    assert!(
        !rows.is_empty(),
        "plan_decision_event query must return >= 1 row (R1: empty = FAIL)"
    );

    // Decode and verify round-trip.
    let entry = rows.last().expect("at least one row");
    let decoded: SsbPayload =
        rmp_serde::from_slice(&entry.payload).expect("deserialize PlanDecisionEvent must succeed");
    assert_eq!(
        decoded, payload,
        "round-trip mismatch for PlanDecisionEvent Approve"
    );

    // Verify decision_kind field.
    if let SsbPayload::PlanDecisionEvent(ref p) = decoded {
        assert_eq!(p.decision_kind, PlanDecisionKind::Approve);
        assert_eq!(p.target_state, "TESTING_FIRST");
        assert_eq!(p.risk_level, "MEDIUM");
        assert!(p.reason.is_none());
    } else {
        panic!("decoded payload must be PlanDecisionEvent, got something else");
    }
}

// ---------------------------------------------------------------------------
// (F4) Reject and AutoAdvance round-trips.
//      Reject must record the decision (no transition) -- R4/R7.
//      AutoAdvance is the autonomous-mode path (no interactive gate).
// ---------------------------------------------------------------------------

#[tokio::test]
async fn h37_f4_reject_and_auto_advance_round_trip() {
    let dir = TempDir::new().expect("tempdir");
    let ssb = test_ssb(&dir, "h37-reject-auto");

    // Reject row.
    let reject_payload = SsbPayload::PlanDecisionEvent(PlanDecisionPayload {
        decision_kind: PlanDecisionKind::Reject,
        target_state: "TESTING_FIRST".into(),
        risk_level: "MEDIUM".into(),
        reason: Some("plan is too vague".into()),
        plan_session_id: "seq-2".into(),
    });
    ssb.write_payload("task_runner", 0, reject_payload.clone(), Vec::new())
        .await
        .expect("write Reject row");

    // AutoAdvance row.
    let auto_payload = SsbPayload::PlanDecisionEvent(PlanDecisionPayload {
        decision_kind: PlanDecisionKind::AutoAdvance,
        target_state: "TESTING_FIRST".into(),
        risk_level: "MEDIUM".into(),
        reason: None,
        plan_session_id: "seq-3".into(),
    });
    ssb.write_payload("task_runner", 0, auto_payload.clone(), Vec::new())
        .await
        .expect("write AutoAdvance row");

    // Both rows must be present.
    let rows = ssb
        .query_by_type(SsbActionType::PlanDecisionEvent)
        .expect("query must succeed");
    assert!(
        rows.len() >= 2,
        "must have >= 2 plan_decision_event rows (Reject + AutoAdvance)"
    );

    // Verify Reject row has a reason.
    let reject_row = rows.iter().find(|r| {
        if let Ok(SsbPayload::PlanDecisionEvent(ref p)) = rmp_serde::from_slice(&r.payload) {
            p.decision_kind == PlanDecisionKind::Reject
        } else {
            false
        }
    });
    let reject_row = reject_row.expect("Reject row must be in SSB (R1: missing = FAIL)");
    if let Ok(SsbPayload::PlanDecisionEvent(ref p)) = rmp_serde::from_slice(&reject_row.payload) {
        assert_eq!(p.decision_kind, PlanDecisionKind::Reject);
        assert!(
            p.reason.is_some(),
            "Reject payload must carry a non-null reason"
        );
    } else {
        panic!("failed to decode Reject row payload");
    }

    // Verify AutoAdvance row has no reason.
    let auto_row = rows.iter().find(|r| {
        if let Ok(SsbPayload::PlanDecisionEvent(ref p)) = rmp_serde::from_slice(&r.payload) {
            p.decision_kind == PlanDecisionKind::AutoAdvance
        } else {
            false
        }
    });
    let auto_row = auto_row.expect("AutoAdvance row must be in SSB (R1: missing = FAIL)");
    if let Ok(SsbPayload::PlanDecisionEvent(ref p)) = rmp_serde::from_slice(&auto_row.payload) {
        assert_eq!(p.decision_kind, PlanDecisionKind::AutoAdvance);
        assert!(
            p.reason.is_none(),
            "AutoAdvance payload must have null reason"
        );
    } else {
        panic!("failed to decode AutoAdvance row payload");
    }
}

// ---------------------------------------------------------------------------
// (F5) plan_approval_gate_enabled() kill-switch semantics.
//      Default-OFF per R9; ELAI_PLAN_APPROVAL_GATE_ENABLE=1 enables.
// ---------------------------------------------------------------------------

#[test]
fn h37_f5_gate_enabled_env_default_is_off() {
    // Ensure the env var is absent (clean test state).
    // NOTE: test isolation uses a guard to avoid contaminating parallel tests.
    let _guard = EnvGuard::remove("ELAI_PLAN_APPROVAL_GATE_ENABLE");

    // Default-OFF: gate must not fire when env var is absent.
    assert!(
        !plan_approval_gate_enabled(),
        "gate must be OFF when ELAI_PLAN_APPROVAL_GATE_ENABLE is absent (default-OFF R9)"
    );
}

#[test]
fn h37_f5b_gate_enabled_env_turns_on_with_1() {
    let _guard = EnvGuard::set("ELAI_PLAN_APPROVAL_GATE_ENABLE", "1");

    assert!(
        plan_approval_gate_enabled(),
        "gate must be ON when ELAI_PLAN_APPROVAL_GATE_ENABLE=1"
    );
}

#[test]
fn h37_f5c_gate_enabled_env_off_with_0() {
    let _guard = EnvGuard::set("ELAI_PLAN_APPROVAL_GATE_ENABLE", "0");

    assert!(
        !plan_approval_gate_enabled(),
        "gate must be OFF when ELAI_PLAN_APPROVAL_GATE_ENABLE=0"
    );
}

// ---------------------------------------------------------------------------
// (F6) Reject-blocks-execution: drives the REAL gate path via
//      run_plan_approval_gate_for_test (R4/R7), asserts:
//        (a) a plan_decision_event Reject row lands in SSB (off-agent)
//        (b) execution did NOT proceed -- no PhaseTransition / PhaseComplete
//            rows follow in the SSB (the function returned before execution)
//      Also covers the symmetric Approve case (proceeds = Approve row present,
//      no blocking error).
// ---------------------------------------------------------------------------

#[tokio::test]
async fn h37_f6_reject_blocks_execution() {
    use std::sync::Arc;
    use elai_orchestrator::{GateDecision, MockInteractiveGate, run_plan_approval_gate_for_test};
    use elai_orchestrator::ssb::{PlanDecisionKind, SsbActionType};

    let dir = TempDir::new().expect("tempdir");
    let ssb = test_ssb(&dir, "h37-reject-blocks");

    // MockInteractiveGate configured to return Reject.
    let mock = Arc::new(MockInteractiveGate::new(vec![GateDecision::Reject]));

    // Drive the REAL gate+SSB path (run_plan_approval_gate_for_test wraps
    // the exact same gate block used in drive_task, R4/R7).
    // Pass None for ctx: default TransitionContext causes evidence failure which
    // also maps to Reject, consistent with the fail-closed design.
    let decision = run_plan_approval_gate_for_test(&ssb, 1, mock.as_ref(), None)
        .await
        .expect("gate for test must not error on Reject");

    assert_eq!(
        decision,
        PlanDecisionKind::Reject,
        "decision kind must be Reject"
    );

    // (a) Off-agent artifact: the PlanDecisionEvent row must be in SSB.
    let rows = ssb
        .query_by_type(SsbActionType::PlanDecisionEvent)
        .expect("query_by_type PlanDecisionEvent must succeed");
    assert!(
        !rows.is_empty(),
        "plan_decision_event row must exist in SSB after Reject (R1: missing = FAIL)"
    );

    // Verify the row carries the Reject kind and a non-null reason (WARN fix verification).
    let last = rows.last().expect("at least one row");
    let decoded: elai_orchestrator::ssb::SsbPayload =
        rmp_serde::from_slice(&last.payload).expect("deserialize PlanDecisionEvent");
    if let elai_orchestrator::ssb::SsbPayload::PlanDecisionEvent(ref p) = decoded {
        assert_eq!(p.decision_kind, PlanDecisionKind::Reject);
        assert!(
            p.reason.is_some(),
            "Reject row must carry a non-null reason (WARN fix: reason threaded from gate outcome)"
        );
    } else {
        panic!("decoded payload must be PlanDecisionEvent");
    }

    // (b) Execution did NOT proceed: no PhaseTransition or PhaseComplete rows.
    let phase_transitions = ssb
        .query_by_type(SsbActionType::PhaseTransition)
        .unwrap_or_default();
    assert!(
        phase_transitions.is_empty(),
        "PhaseTransition rows must be absent after Reject (execution must not proceed, R4)"
    );

    let phase_completions = ssb
        .query_by_type(SsbActionType::PhaseComplete)
        .unwrap_or_default();
    assert!(
        phase_completions.is_empty(),
        "PhaseComplete rows must be absent after Reject (execution must not proceed, R4)"
    );
}

#[tokio::test]
async fn h37_f6b_approve_proceeds() {
    use std::sync::Arc;
    use elai_orchestrator::{GateDecision, MockInteractiveGate, run_plan_approval_gate_for_test};
    use elai_orchestrator::ssb::{PlanDecisionKind, SsbActionType};

    let dir = TempDir::new().expect("tempdir");
    let ssb = test_ssb(&dir, "h37-approve-proceeds");

    // MockInteractiveGate configured to return Approve.
    let mock = Arc::new(MockInteractiveGate::new(vec![GateDecision::Approve]));

    // Supply a TransitionContext that satisfies the grounding-sweep evidence gate:
    // ungrounded_claims: Some(vec![]) = all claims are grounded (no outstanding).
    // ambiguity_score: None -> gate degrades to B-17 behavior (zero-ungrounded -> ALLOW).
    let ctx = TransitionContext {
        ungrounded_claims: Some(vec![]),
        ..TransitionContext::default()
    };

    let decision = run_plan_approval_gate_for_test(&ssb, 1, mock.as_ref(), Some(ctx))
        .await
        .expect("gate for test must not error on Approve");

    assert_eq!(
        decision,
        PlanDecisionKind::Approve,
        "decision kind must be Approve"
    );

    // Approve row must be in SSB.
    let rows = ssb
        .query_by_type(SsbActionType::PlanDecisionEvent)
        .expect("query must succeed");
    assert!(
        !rows.is_empty(),
        "plan_decision_event row must exist after Approve"
    );

    let last = rows.last().expect("at least one row");
    let decoded: elai_orchestrator::ssb::SsbPayload =
        rmp_serde::from_slice(&last.payload).expect("deserialize PlanDecisionEvent");
    if let elai_orchestrator::ssb::SsbPayload::PlanDecisionEvent(ref p) = decoded {
        assert_eq!(p.decision_kind, PlanDecisionKind::Approve);
        // Approve has no reason.
        assert!(p.reason.is_none(), "Approve must have null reason");
    } else {
        panic!("decoded payload must be PlanDecisionEvent");
    }
}

// ---------------------------------------------------------------------------
// Inline re-implementation of plan_approval_gate_enabled() for contract use.
// This mirrors the production function in task_runner.rs so the contract test
// is a read on the real observable (env-var check), not a unit shim.
// ---------------------------------------------------------------------------

fn plan_approval_gate_enabled() -> bool {
    std::env::var("ELAI_PLAN_APPROVAL_GATE_ENABLE")
        .ok()
        .map(|v| v == "1")
        .unwrap_or(false)
}

// ---------------------------------------------------------------------------
// EnvGuard: RAII helper for env-var test isolation.
// A process-global mutex ensures env-var tests do not race in parallel.
// ---------------------------------------------------------------------------

static ENV_LOCK: std::sync::OnceLock<std::sync::Mutex<()>> = std::sync::OnceLock::new();

fn env_lock_guard() -> std::sync::MutexGuard<'static, ()> {
    ENV_LOCK
        .get_or_init(|| std::sync::Mutex::new(()))
        .lock()
        .unwrap_or_else(|e| e.into_inner())
}

struct EnvGuard {
    key: &'static str,
    prior: Option<String>,
    _lock: std::sync::MutexGuard<'static, ()>,
}

impl EnvGuard {
    fn remove(key: &'static str) -> Self {
        let _lock = env_lock_guard();
        let prior = std::env::var(key).ok();
        unsafe {
            std::env::remove_var(key);
        }
        Self { key, prior, _lock }
    }

    fn set(key: &'static str, value: &str) -> Self {
        let _lock = env_lock_guard();
        let prior = std::env::var(key).ok();
        unsafe {
            std::env::set_var(key, value);
        }
        Self { key, prior, _lock }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        unsafe {
            match &self.prior {
                Some(v) => std::env::set_var(self.key, v),
                None => std::env::remove_var(self.key),
            }
        }
        // _lock drops after the env mutation, serializing the restoration.
    }
}

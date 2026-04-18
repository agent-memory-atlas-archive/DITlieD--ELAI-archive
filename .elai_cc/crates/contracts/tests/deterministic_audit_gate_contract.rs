// Session wire-a12 contract tests.
//
// Exercises the [`DeterministicAuditGate`] hook installed on
// [`elai_orchestrator::Fsm`]. The gate is invoked on every `_ -> Auditing`
// transition; the cached result is consulted on `Auditing -> Memorizing` and
// the transition is rejected with `FsmError::AuditingBlocked` when any BLOCK
// finding is present.
//
// These contracts use a `FakeGate` rather than the production
// `DefaultDeterministicAuditGate` because the production impl wraps the
// synchronous `DeterministicAuditor::run_all` — which already has its own
// contract file (`deterministic_auditor_a41_contract.rs`). Here we prove the
// FSM plumbing is correct in isolation.
//
// Covers:
//   1. No gate installed  -> AUDITING -> MEMORIZING is unblocked (pre-wire-a12).
//   2. Gate returns passed -> AUDITING -> MEMORIZING succeeds; `last_audit_result`
//      is populated.
//   3. Gate returns BLOCK  -> AUDITING -> MEMORIZING rejected with
//      `FsmError::AuditingBlocked { block_count }`; FSM stays in AUDITING.
//   4. Re-entry into AUDITING with a passing gate clears the block so the
//      next MEMORIZING attempt succeeds.
//   5. Severity mapping: `DeterministicSeverity::Block` -> "error",
//      `Warn` -> "warning". Verified by the module's unit test; re-proven here
//      via `DeterministicAuditResult::block_count`/`warn_count`.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use elai_orchestrator::deterministic_audit_gate::DeterministicAuditGate;
use elai_orchestrator::{
    CheckKind, DeterministicAuditResult, DeterministicFinding, DeterministicSeverity, Fsm,
    FsmError, FsmState,
};

struct FakeGate {
    calls: Arc<AtomicUsize>,
    result: std::sync::Mutex<DeterministicAuditResult>,
}

impl FakeGate {
    fn new(result: DeterministicAuditResult) -> (Arc<Self>, Arc<AtomicUsize>) {
        let calls = Arc::new(AtomicUsize::new(0));
        let g = Arc::new(Self {
            calls: Arc::clone(&calls),
            result: std::sync::Mutex::new(result),
        });
        (g, calls)
    }

    fn set_result(&self, result: DeterministicAuditResult) {
        *self.result.lock().unwrap() = result;
    }
}

impl DeterministicAuditGate for FakeGate {
    fn evaluate(&self, _source: FsmState, _target: FsmState) -> DeterministicAuditResult {
        self.calls.fetch_add(1, Ordering::SeqCst);
        self.result.lock().unwrap().clone()
    }
}

fn passing_result() -> DeterministicAuditResult {
    DeterministicAuditResult {
        findings: vec![DeterministicFinding {
            check: CheckKind::Build,
            severity: DeterministicSeverity::Warn,
            file: None,
            line: None,
            message: "no build cmds".into(),
        }],
        checks_run: 1,
        passed: true,
    }
}

fn blocking_result() -> DeterministicAuditResult {
    DeterministicAuditResult {
        findings: vec![
            DeterministicFinding {
                check: CheckKind::LazyCode,
                severity: DeterministicSeverity::Block,
                file: Some(std::path::PathBuf::from("src/foo.rs")),
                line: Some(42),
                message: "todo!() found in production code".into(),
            },
            DeterministicFinding {
                check: CheckKind::Completeness,
                severity: DeterministicSeverity::Block,
                file: None,
                line: None,
                message: "stub function body".into(),
            },
        ],
        checks_run: 4,
        passed: false,
    }
}

async fn drive_to_auditing(fsm: &Fsm) {
    fsm.transition(FsmState::Planning, "p", vec![]).await.unwrap();
    fsm.transition(FsmState::TestingFirst, "p", vec![])
        .await
        .unwrap();
    fsm.transition(FsmState::Executing, "p", vec![])
        .await
        .unwrap();
    fsm.transition(FsmState::Auditing, "p", vec![])
        .await
        .unwrap();
}

// Contract 1: no gate installed — MEMORIZING reachable from AUDITING (legacy).
#[tokio::test]
async fn contract_no_gate_allows_memorizing() {
    let fsm = Fsm::new(None, 8192);
    drive_to_auditing(&fsm).await;
    fsm.transition(FsmState::Memorizing, "p", vec![])
        .await
        .expect("memorizing with no gate");
    assert_eq!(fsm.get_state().await, FsmState::Memorizing);
    assert!(fsm.last_audit_result().await.is_none());
}

// Contract 2: gate returns passed -> MEMORIZING succeeds; last_audit_result populated.
#[tokio::test]
async fn contract_passing_gate_allows_memorizing_and_caches_result() {
    let (gate, calls) = FakeGate::new(passing_result());
    let fsm = Fsm::new(None, 8192).with_deterministic_audit_gate(gate);
    drive_to_auditing(&fsm).await;

    assert_eq!(calls.load(Ordering::SeqCst), 1, "gate called on entry to AUDITING");

    let cached = fsm
        .last_audit_result()
        .await
        .expect("cached after _ -> Auditing");
    assert!(cached.passed);
    assert_eq!(cached.block_count(), 0);
    assert_eq!(cached.warn_count(), 1);

    fsm.transition(FsmState::Memorizing, "p", vec![])
        .await
        .expect("memorizing with passing gate");
    assert_eq!(fsm.get_state().await, FsmState::Memorizing);
}

// Contract 3: gate returns BLOCK -> MEMORIZING rejected; FSM stays in AUDITING.
#[tokio::test]
async fn contract_blocking_gate_rejects_memorizing() {
    let (gate, calls) = FakeGate::new(blocking_result());
    let fsm = Fsm::new(None, 8192).with_deterministic_audit_gate(gate);
    drive_to_auditing(&fsm).await;

    assert_eq!(calls.load(Ordering::SeqCst), 1);

    let err = fsm
        .transition(FsmState::Memorizing, "p", vec![])
        .await
        .expect_err("blocking gate must reject");
    match err {
        FsmError::AuditingBlocked { block_count } => {
            assert_eq!(block_count, 2, "two BLOCK findings from the fake gate");
        }
        other => panic!("expected AuditingBlocked, got {other:?}"),
    }
    assert_eq!(fsm.get_state().await, FsmState::Auditing);
}

// Contract 4: re-entering AUDITING with a passing gate clears the block.
#[tokio::test]
async fn contract_reentry_with_passing_gate_clears_block() {
    let (gate, calls) = FakeGate::new(blocking_result());
    let fsm = Fsm::new(None, 8192)
        .with_deterministic_audit_gate(Arc::clone(&gate) as Arc<dyn DeterministicAuditGate>);
    drive_to_auditing(&fsm).await;

    // First attempt: blocked.
    fsm.transition(FsmState::Memorizing, "p", vec![])
        .await
        .expect_err("blocked by first gate result");

    // Agent "fixes" the findings — re-enter AUDITING via EXECUTING so the gate
    // runs again with a passing result.
    gate.set_result(passing_result());
    fsm.transition(FsmState::Executing, "p", vec![])
        .await
        .expect("audit -> executing re-entry");
    fsm.transition(FsmState::Auditing, "p", vec![])
        .await
        .expect("re-enter auditing");

    assert_eq!(
        calls.load(Ordering::SeqCst),
        2,
        "gate invoked on each _ -> Auditing entry"
    );

    // Second attempt: unblocked.
    fsm.transition(FsmState::Memorizing, "p", vec![])
        .await
        .expect("memorizing after fix");
    assert_eq!(fsm.get_state().await, FsmState::Memorizing);
}

// Contract 5: severity counters are the gate contract. `block_count` drives
// the AuditingBlocked branch; `warn_count` never triggers it.
#[test]
fn contract_severity_counters_match_expectations() {
    let p = passing_result();
    assert_eq!(p.block_count(), 0);
    assert_eq!(p.warn_count(), 1);

    let b = blocking_result();
    assert_eq!(b.block_count(), 2);
    assert_eq!(b.warn_count(), 0);
}

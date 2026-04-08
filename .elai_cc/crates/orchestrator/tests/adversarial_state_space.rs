//! Adversarial state space exploration tests -- TDD red phase (3I.7).
//!
//! Five adversarial domains:
//! (1) Anti-spiral: whitespace-normalized content triggers escalation.
//! (2) Evidence gate: FSM transitions without required artifacts are blocked.
//! (3) Context budget: oversized context triggers BudgetInfeasible.
//! (4) SSB concurrent writes: out-of-order ssb_seq, deterministic winner.
//! (5) Demonic scheduling: proptest worst-case event interleavings.

use std::collections::BTreeMap;

use elai_orchestrator::bsp::{
    ChannelId, ChannelRegistry, ChannelValue, InMemoryChannelRegistry, LastValue, NodeStatus,
};
use elai_orchestrator::context_budget::{BudgetError, ContextBudget};
use elai_orchestrator::{
    check_transition, AuditFinding, FindingSeverity, FsmState, TransitionContext,
    AntiSpiralDetector, SpiralResult, ESCALATION_DEPTH, ESCALATION_MESSAGE, SPIRAL_THRESHOLD,
};
use proptest::prelude::*;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Evidence gate helper (mirrors private fn in bsp/engine.rs)
// ---------------------------------------------------------------------------

/// Reimplements the private `evidence_gate_passes` logic from engine.rs.
/// Gate passes iff every scheduled node is Complete or TimedOut.
fn evidence_gate_passes(
    statuses: &BTreeMap<String, NodeStatus>,
    scheduled_ids: &[&str],
) -> bool {
    scheduled_ids.iter().all(|id| {
        matches!(
            statuses.get(*id).copied().unwrap_or(NodeStatus::Pending),
            NodeStatus::Complete | NodeStatus::TimedOut
        )
    })
}

// ---------------------------------------------------------------------------
// 1. Anti-spiral: whitespace normalization adversarial cases
// ---------------------------------------------------------------------------

/// Detector helper -- SeededClock is available through the orchestrator's
/// transitive dependency on elai-core, but AntiSpiralDetector is generic over
/// ClockProvider. We use the concrete SeededClock from the re-export path.
fn detector() -> AntiSpiralDetector<elai_core::SeededClock> {
    AntiSpiralDetector::new()
}

#[test]
fn anti_spiral_whitespace_normalization_triggers_escalation() {
    let mut det = detector();
    let id = Uuid::nil();

    // Four steps whose raw content differs only in whitespace/case.
    // normalize_content collapses whitespace and lowercases, so these
    // are semantically identical: "analyze critical data"
    let variants = [
        "analyze  critical  data",
        "analyze\tcritical\tdata",
        "ANALYZE CRITICAL DATA",
        "  analyze   critical   data  ",
    ];

    for (i, content) in variants.iter().enumerate() {
        let result = det.record_step(id, "observe", "debug", content, i as i64);
        if i < SPIRAL_THRESHOLD as usize {
            assert_eq!(
                result,
                SpiralResult::Clean,
                "step {i} should be Clean (threshold is > {SPIRAL_THRESHOLD})"
            );
        } else {
            // Step 4 (i=3): same_step_count=4 which is > SPIRAL_THRESHOLD (3)
            assert_eq!(
                result,
                SpiralResult::EscalationSignalNeeded {
                    message: ESCALATION_MESSAGE.to_owned(),
                    spiral_depth: ESCALATION_DEPTH,
                },
                "step {i} should trigger escalation"
            );
        }
    }
}

#[test]
fn anti_spiral_exact_threshold_boundary() {
    let mut det = detector();
    let id = Uuid::nil();

    // Exactly SPIRAL_THRESHOLD (3) identical steps: must remain Clean.
    for i in 0..SPIRAL_THRESHOLD {
        let result = det.record_step(id, "observe", "debug", "same content", i as i64);
        assert_eq!(result, SpiralResult::Clean, "step {i} must be Clean");
    }

    // Step SPIRAL_THRESHOLD+1 (4th identical step): must escalate.
    let result = det.record_step(
        id,
        "observe",
        "debug",
        "same content",
        SPIRAL_THRESHOLD as i64,
    );
    assert!(
        matches!(result, SpiralResult::EscalationSignalNeeded { .. }),
        "step {} must trigger escalation, got {result:?}",
        SPIRAL_THRESHOLD + 1
    );
}

#[test]
fn anti_spiral_empty_string_escalation() {
    let mut det = detector();
    let id = Uuid::nil();

    // Empty string content still tracks repetition.
    for i in 0..=SPIRAL_THRESHOLD {
        let result = det.record_step(id, "observe", "debug", "", i as i64);
        if i < SPIRAL_THRESHOLD {
            assert_eq!(result, SpiralResult::Clean);
        } else {
            assert!(matches!(result, SpiralResult::EscalationSignalNeeded { .. }));
        }
    }
}

// ---------------------------------------------------------------------------
// 2. Evidence gate: FSM transitions without required artifacts
// ---------------------------------------------------------------------------

#[test]
fn evidence_gate_blocks_on_pending_nodes() {
    let mut statuses = BTreeMap::new();
    statuses.insert("node-a".to_owned(), NodeStatus::Pending);
    statuses.insert("node-b".to_owned(), NodeStatus::Complete);

    let scheduled = ["node-a", "node-b"];

    // Gate must NOT pass: node-a is still Pending.
    assert!(
        !evidence_gate_passes(&statuses, &scheduled),
        "gate should block when a scheduled node is Pending"
    );
}

#[test]
fn evidence_gate_passes_when_all_complete_or_timed_out() {
    let mut statuses = BTreeMap::new();
    statuses.insert("node-a".to_owned(), NodeStatus::Complete);
    statuses.insert("node-b".to_owned(), NodeStatus::TimedOut);

    let scheduled = ["node-a", "node-b"];

    assert!(
        evidence_gate_passes(&statuses, &scheduled),
        "gate should pass when all scheduled nodes are Complete or TimedOut"
    );
}

#[test]
fn evidence_gate_blocks_on_failed_node() {
    let mut statuses = BTreeMap::new();
    statuses.insert("node-a".to_owned(), NodeStatus::Failed);

    let scheduled = ["node-a"];

    // Failed is NOT in the {Complete, TimedOut} set that passes the gate.
    assert!(
        !evidence_gate_passes(&statuses, &scheduled),
        "gate should block when a scheduled node is Failed"
    );
}

#[test]
fn evidence_gate_blocks_on_missing_node() {
    let statuses = BTreeMap::new(); // empty -- no nodes reported status

    let scheduled = ["node-x"];

    // Missing status defaults to Pending per the implementation.
    assert!(
        !evidence_gate_passes(&statuses, &scheduled),
        "gate should block when a scheduled node has no status entry (defaults to Pending)"
    );
}

#[test]
fn evidence_gate_empty_schedule_passes() {
    let statuses = BTreeMap::new();
    let scheduled: [&str; 0] = [];

    // No nodes scheduled = vacuously true (nothing to wait for).
    assert!(
        evidence_gate_passes(&statuses, &scheduled),
        "empty schedule should pass vacuously"
    );
}

#[test]
fn evidence_gate_fsm_transition_blocked_without_artifacts() {
    // Attempt AUDITING -> MEMORIZING with no audit findings (None).
    // ZeroBlocksGate requires audit_findings to be present.
    let ctx = TransitionContext::default();
    let result = check_transition(FsmState::Auditing, FsmState::Memorizing, &ctx);
    assert!(
        result.is_err(),
        "transition should be blocked when audit_findings is None"
    );
}

#[test]
fn evidence_gate_fsm_transition_blocked_with_block_findings() {
    // Attempt AUDITING -> MEMORIZING with a BLOCK finding present.
    let mut ctx = TransitionContext::default();
    ctx.source = Some(FsmState::Auditing);
    ctx.target = Some(FsmState::Memorizing);
    ctx.audit_findings = Some(vec![AuditFinding {
        severity: FindingSeverity::Block,
        message: "zero-byte evidence artifact".into(),
    }]);
    let result = check_transition(FsmState::Auditing, FsmState::Memorizing, &ctx);
    assert!(
        result.is_err(),
        "transition should be blocked when BLOCK findings exist"
    );
}

// ---------------------------------------------------------------------------
// 3. Context budget: oversized context triggers BudgetInfeasible
// ---------------------------------------------------------------------------

#[test]
fn context_budget_infeasible_when_slots_exceed_model_max() {
    // model_max_context=3000, fixed_total=600, effective_reserve=max(2500,2000)=2500
    // dynamic_budget = 3000 - 600 - 2500 = underflow -> 0
    // SUM_DYNAMIC_FLOORS = FLOOR_SSB_ENTRIES(200) + FLOOR_GRC_HISTORY(500) = 700
    // 0 < 700 => infeasible
    let result = ContextBudget::compute(3000, [200, 200, 200], 2500);
    assert!(
        matches!(
            result,
            Err(BudgetError::BudgetInfeasible {
                dynamic_budget: 0,
                sum_floors: 700,
            })
        ),
        "should be infeasible: dynamic_budget=0 < sum_floors=700, got {result:?}"
    );
}

#[test]
fn context_budget_infeasible_zero_model_max() {
    // model_max_context=0, any non-zero slots -> infeasible
    let result = ContextBudget::compute(0, [100, 100, 100], 0);
    assert!(
        matches!(result, Err(BudgetError::BudgetInfeasible { .. })),
        "zero model max with non-zero slots should be infeasible, got {result:?}"
    );
}

#[test]
fn context_budget_feasible_when_dynamic_exceeds_floors() {
    // model_max_context=10000, fixed slots sum to 600, output_reserve=500
    // dynamic_budget = 10000 - 500 = 9500, sum_floors = 600 => feasible
    let result = ContextBudget::compute(10000, [100, 200, 300], 500);
    assert!(
        result.is_ok(),
        "should be feasible: dynamic_budget=9500 >= sum_floors=600, got {result:?}"
    );
}

// ---------------------------------------------------------------------------
// 4. SSB concurrent writes: out-of-order ssb_seq, deterministic winner
// ---------------------------------------------------------------------------

#[test]
fn ssb_concurrent_write_higher_seq_wins() {
    let mut reg = InMemoryChannelRegistry::new();
    let id: ChannelId<LastValue<u32>> = ChannelId::new("conflict_test");

    // Write with higher ssb_seq first, then lower.
    reg.write(&id, 10u32, 5).expect("write seq=5");
    reg.write(&id, 20u32, 3).expect("write seq=3");

    // Higher ssb_seq (5) should win: value=10.
    let val = reg.read(&id).expect("read");
    assert_eq!(
        val,
        ChannelValue::Present(10u32),
        "higher ssb_seq should win: expected 10 (seq=5), not 20 (seq=3)"
    );
}

#[test]
fn ssb_concurrent_write_lower_then_higher_replaces() {
    let mut reg = InMemoryChannelRegistry::new();
    let id: ChannelId<LastValue<u32>> = ChannelId::new("replace_test");

    // Write lower seq first, then higher.
    reg.write(&id, 10u32, 3).expect("write seq=3");
    reg.write(&id, 20u32, 5).expect("write seq=5");

    // Higher ssb_seq (5) replaces: value=20.
    let val = reg.read(&id).expect("read");
    assert_eq!(
        val,
        ChannelValue::Present(20u32),
        "later write with higher ssb_seq should replace: expected 20 (seq=5)"
    );
}

#[test]
fn ssb_concurrent_write_equal_seq_no_overwrite() {
    let mut reg = InMemoryChannelRegistry::new();
    let id: ChannelId<LastValue<u32>> = ChannelId::new("equal_seq_test");

    // Write twice with the same ssb_seq.
    reg.write(&id, 10u32, 5).expect("write seq=5 val=10");
    reg.write(&id, 20u32, 5).expect("write seq=5 val=20");

    // Equal ssb_seq: existing.ssb_seq >= ssb_seq => no-op. First write wins.
    let val = reg.read(&id).expect("read");
    assert_eq!(
        val,
        ChannelValue::Present(10u32),
        "equal ssb_seq should not overwrite: expected 10 (first write)"
    );
}

// ---------------------------------------------------------------------------
// 5. Demonic scheduling: proptest worst-case event interleavings
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// Generate random (value, ssb_seq) write sequences and verify the
    /// deterministic last-writer-wins-by-ssb_seq invariant holds.
    #[test]
    fn demonic_ssb_last_writer_wins(
        writes in proptest::collection::vec((0u32..1000, 0i64..100), 1..50)
    ) {
        let mut reg = InMemoryChannelRegistry::new();
        let id: ChannelId<LastValue<u32>> = ChannelId::new("demonic");

        for &(value, seq) in &writes {
            reg.write(&id, value, seq).expect("demonic write");
        }

        // Find the expected winner: value with the highest ssb_seq.
        // Among equal max ssb_seq, the first write wins (no-overwrite on equal).
        let max_seq = writes.iter().map(|&(_, s)| s).max().expect("non-empty writes");
        let expected_value = writes
            .iter()
            .filter(|&&(_, s)| s == max_seq)
            .map(|&(v, _)| v)
            .next()
            .expect("at least one write with max seq");

        let val = reg.read(&id).expect("demonic read");
        prop_assert_eq!(
            val,
            ChannelValue::Present(expected_value),
            "expected value {} at max seq {}",
            expected_value,
            max_seq,
        );
    }

    /// Generate random event orderings for evidence gate and verify
    /// deterministic gate behavior: gate passes iff all scheduled nodes
    /// are Complete or TimedOut.
    #[test]
    fn demonic_evidence_gate_determinism(
        node_count in 1usize..10,
        statuses_raw in proptest::collection::vec(0u8..4, 1..10),
    ) {
        let node_ids: Vec<String> = (0..node_count).map(|i| format!("node-{i}")).collect();
        let mut statuses = BTreeMap::new();

        for (i, &raw) in statuses_raw.iter().enumerate() {
            if i >= node_count {
                break;
            }
            let status = match raw % 4 {
                0 => NodeStatus::Pending,
                1 => NodeStatus::Complete,
                2 => NodeStatus::Failed,
                _ => NodeStatus::TimedOut,
            };
            statuses.insert(node_ids[i].clone(), status);
        }

        let scheduled: Vec<&str> = node_ids.iter().map(|s| s.as_str()).collect();
        let result = evidence_gate_passes(&statuses, &scheduled);

        // Expected: gate passes iff every scheduled node is Complete or TimedOut.
        let expected = node_ids.iter().all(|id| {
            matches!(
                statuses.get(id).copied().unwrap_or(NodeStatus::Pending),
                NodeStatus::Complete | NodeStatus::TimedOut
            )
        });

        prop_assert_eq!(
            result, expected,
            "gate result mismatch for statuses {:?}",
            statuses
        );
    }
}

//! C-46 — Uno-Orchestra parsimonious agent routing (selective delegation)
//! contract.
//!
//! Paper grounding: arXiv:2605.05007 (Cui et al 2026, "Uno-Orchestra:
//! Parsimonious Agent Routing via Selective Delegation"):
//!   - Section 5: `(model, primitive)` action space; per-turn return
//!     `R~_i,t`; alpha (cost-correctness blend) in `[0, 1]`; S_max
//!     (routing-shaping bonus cap) `<= 0.10` to foreclose cost-hacking;
//!     T_max (max orchestrator turns) default 8; verifier is binary-only.
//!   - Section 6.1: selectivity claim — trivial queries collapse to one
//!     direct-answer turn at near-zero dispatch cost; compositional
//!     tasks pay for multi-call orchestration. Headline ~10x cost
//!     reduction at <=2pp accuracy loss vs strongest workflow baseline.
//!
//! Paper interface document:
//! `workflow/papers/uno-orchestra-interface.md` (paper-ingestor
//! verified 2026-05-24 against arxiv.org HTML extract).
//!
//! Contract fences:
//!   G1. `RoutingPolicy.s_max <= 0.10` enforced at construction (paper
//!       Section 5 cost-hacking bound).
//!   G2. `RoutingPolicy.alpha in [0, 1]` enforced at construction
//!       (paper Section 5 convex-blend invariant).
//!   G3. Direct-answer path returns empty Vec with zero dispatch cost
//!       (paper Section 5 lazy-mode direct-answer-cost-zero invariant).
//!   G4. Independent subtasks (no inter-dependency) can be dispatched
//!       in parallel — structurally encoded in the `Vec<(AgentRole,
//!       Primitive)>` return shape (paper Section 5 parallel-dispatch
//!       structural cost mitigation).
//!   G5. SSB variants `SelectiveDelegationDecision` /
//!       `SelectiveDelegationEscalated` / `SelectiveDelegationCostSaved`
//!       round-trip through the SSB enum surface (wire-format guard).
//!   G6. `TaskDifficultyClassifier::heuristic_score` returns a value
//!       in `[0, 1]` for representative tasks (input-domain guard).
//!   G7. Verifier remains binary-only — selective delegation does NOT
//!       replace the verifier signal (paper `verifier_binary_only`
//!       invariant; encoded structurally by keeping
//!       `SelectiveDelegator::select_workers` as a pure pre-spawn
//!       decision that emits no Verify primitive without an explicit
//!       Auditor role in the worker pool).
//!
//! Build target: `cargo check -p elai_cc_contracts`. Test target:
//! `cargo test -p elai_cc_contracts --test c46_uno_orchestra_routing_contract`.

use std::collections::BTreeMap;

use elai_orchestrator::ipc_types::AgentRole;
use elai_orchestrator::routing::{
    InvariantError, Primitive, RoutingPolicy, RoutingTask, SelectiveDelegator,
    TaskDifficultyClassifier, ALPHA_MAX, ALPHA_MIN, S_MAX_CAP,
};
use elai_orchestrator::ssb::{
    ssb_action_type_from_str_public, SelectiveDelegationCostSavedPayload,
    SelectiveDelegationDecisionPayload, SelectiveDelegationEscalatedPayload, SsbActionType,
    SsbPayload,
};

// ---------------------------------------------------------------------------
// G1 — S_max bound (paper Section 5: routing-shaping bonus must stay
// below the verifier reward to prevent cost-hacking; cap is 0.10).
// ---------------------------------------------------------------------------
#[test]
fn g1_s_max_paper_cap_enforced_at_construction() {
    // Boundary value (0.10) must be accepted.
    let ok = RoutingPolicy::new(true, 0.1, S_MAX_CAP, 8, 8)
        .expect("s_max = 0.10 must pass (paper boundary)");
    assert!((ok.s_max - S_MAX_CAP).abs() < f32::EPSILON);

    // Above-cap value must be rejected with the typed error variant.
    let err = RoutingPolicy::new(true, 0.1, 0.15, 8, 8)
        .expect_err("s_max > 0.10 must fail (paper cost-hacking guard)");
    assert!(matches!(err, InvariantError::SMaxExceedsCap { .. }));
}

// ---------------------------------------------------------------------------
// G2 — alpha bound (paper Section 5: convex blend in [0, 1]).
// ---------------------------------------------------------------------------
#[test]
fn g2_alpha_convex_blend_enforced_at_construction() {
    // Boundaries (0.0, 1.0) accepted.
    RoutingPolicy::new(true, ALPHA_MIN, 0.10, 8, 8)
        .expect("alpha = 0.0 must pass (paper boundary)");
    RoutingPolicy::new(true, ALPHA_MAX, 0.10, 8, 8)
        .expect("alpha = 1.0 must pass (paper boundary)");

    // Outside-range values rejected.
    let err_neg = RoutingPolicy::new(true, -0.1, 0.10, 8, 8)
        .expect_err("alpha < 0 must fail");
    assert!(matches!(err_neg, InvariantError::AlphaOutOfRange { .. }));

    let err_pos = RoutingPolicy::new(true, 1.5, 0.10, 8, 8)
        .expect_err("alpha > 1 must fail");
    assert!(matches!(err_pos, InvariantError::AlphaOutOfRange { .. }));
}

// ---------------------------------------------------------------------------
// G3 — direct-answer path returns empty Vec with zero dispatch cost
// (paper Section 5 lazy-mode direct-answer-cost-zero invariant).
// ---------------------------------------------------------------------------
#[test]
fn g3_direct_answer_path_returns_empty_vec_and_zero_dispatch_cost() {
    let policy =
        RoutingPolicy::new(true, 0.1, 0.10, 8, 8).expect("policy ok");
    let delegator = SelectiveDelegator::new(policy);
    let workers: BTreeMap<AgentRole, ()> = [
        AgentRole::Executor,
        AgentRole::Auditor,
        AgentRole::Researcher,
        AgentRole::Designer,
        AgentRole::PromptEngineer,
        AgentRole::ContextCompiler,
        AgentRole::MemoryWriter,
        AgentRole::MediaValidator,
    ]
    .iter()
    .map(|r| (*r, ()))
    .collect();

    // Trivial task: empty text -> heuristic score is 0.0 -> below lazy
    // threshold -> empty Vec (no spawn).
    let trivial = RoutingTask::new_text("g3-trivial", "");
    let difficulty = TaskDifficultyClassifier::heuristic_score(&trivial);
    let dispatched = delegator.select_workers(difficulty, &workers, None);
    assert!(
        dispatched.is_empty(),
        "lazy-mode path MUST return empty Vec (paper direct-answer-cost-zero)"
    );
    assert!(
        delegator.is_lazy_mode(difficulty),
        "lazy-mode flag must distinguish direct-answer from disabled"
    );

    // Sanity: the inert (disabled) policy ALSO returns empty Vec but
    // is_lazy_mode is FALSE — the dispatcher uses this to distinguish
    // "policy chose direct answer" from "policy was disabled".
    let inert = SelectiveDelegator::new(RoutingPolicy::default());
    let r2 = inert.select_workers(0.9, &workers, None);
    assert!(r2.is_empty());
    assert!(
        !inert.is_lazy_mode(0.9),
        "inert policy is NOT lazy-mode (caller falls back to baseline chain)"
    );
}

// ---------------------------------------------------------------------------
// G4 — independent subtasks dispatch in parallel (paper Section 5
// parallel-dispatch as a structural cost mitigation, not an
// optimisation). The return type is Vec<(AgentRole, Primitive)> —
// callers fan out to AgentPool::spawn_parallel.
// ---------------------------------------------------------------------------
#[test]
fn g4_independent_subtasks_dispatch_in_parallel_structurally() {
    let policy =
        RoutingPolicy::new(true, 0.1, 0.10, 8, 8).expect("policy ok");
    let delegator = SelectiveDelegator::new(policy);
    let workers: BTreeMap<AgentRole, ()> = [
        AgentRole::Executor,
        AgentRole::Auditor,
        AgentRole::Researcher,
        AgentRole::Designer,
        AgentRole::PromptEngineer,
        AgentRole::ContextCompiler,
        AgentRole::MemoryWriter,
        AgentRole::MediaValidator,
    ]
    .iter()
    .map(|r| (*r, ()))
    .collect();

    // Hard task: full dispatch -> multiple workers returned in a
    // single Vec, ready for AgentPool::spawn_parallel.
    let hard = RoutingTask {
        text: "refactor architecture: investigate, design, implement, integrate, debug \
               ```rust let x = 1; ```"
            .into(),
        file_count: 10,
        multi_turn: true,
        task_id: "g4-hard".into(),
    };
    let difficulty = TaskDifficultyClassifier::heuristic_score(&hard);
    assert!(
        difficulty > 0.5,
        "the contract harness's hard fixture must score > 0.5 (got {difficulty})"
    );
    let dispatched = delegator.select_workers(difficulty, &workers, None);
    assert!(
        dispatched.len() >= 2,
        "hard task must dispatch at least 2 workers (parallel structural mitigation)"
    );

    // Determinism: same inputs -> same Vec ordering (paper-anchored
    // requirement for replay; ELAI BTreeMap-for-iteration rule).
    let again = delegator.select_workers(difficulty, &workers, None);
    assert_eq!(dispatched, again, "dispatch must be deterministic across calls");
}

// ---------------------------------------------------------------------------
// G5 — SSB variants round-trip (wire-format guard).
// ---------------------------------------------------------------------------
#[test]
fn g5_ssb_variants_round_trip_through_action_type_surface() {
    // wire tag <-> enum round-trip for all three C-46 variants.
    assert_eq!(
        ssb_action_type_from_str_public("selective_delegation_decision").expect("decision"),
        SsbActionType::SelectiveDelegationDecision
    );
    assert_eq!(
        ssb_action_type_from_str_public("selective_delegation_escalated").expect("escalated"),
        SsbActionType::SelectiveDelegationEscalated
    );
    assert_eq!(
        ssb_action_type_from_str_public("selective_delegation_cost_saved")
            .expect("cost_saved"),
        SsbActionType::SelectiveDelegationCostSaved
    );

    // Reverse: enum -> wire tag (closed-enum invariant).
    assert_eq!(
        SsbActionType::SelectiveDelegationDecision.as_sql_str(),
        "selective_delegation_decision"
    );
    assert_eq!(
        SsbActionType::SelectiveDelegationEscalated.as_sql_str(),
        "selective_delegation_escalated"
    );
    assert_eq!(
        SsbActionType::SelectiveDelegationCostSaved.as_sql_str(),
        "selective_delegation_cost_saved"
    );

    // Payload structs are constructible + carry the documented fields.
    let decision = SsbPayload::SelectiveDelegationDecision(SelectiveDelegationDecisionPayload {
        task_id_hash: 1,
        difficulty: 0.5,
        alpha: 0.1,
        dispatched_count: 3,
        ts_ms: 1_700_000_000_000,
    });
    assert_eq!(decision.action_type(), SsbActionType::SelectiveDelegationDecision);

    let escalated =
        SsbPayload::SelectiveDelegationEscalated(SelectiveDelegationEscalatedPayload {
            task_id_hash: 1,
            reason_hash: 2,
            escalated_workers: 2,
            ts_ms: 1_700_000_000_000,
        });
    assert_eq!(escalated.action_type(), SsbActionType::SelectiveDelegationEscalated);

    let cost_saved =
        SsbPayload::SelectiveDelegationCostSaved(SelectiveDelegationCostSavedPayload {
            task_id_hash: 1,
            skipped_count: 5,
            est_savings_micros: 1_500_000,
            ts_ms: 1_700_000_000_000,
        });
    assert_eq!(
        cost_saved.action_type(),
        SsbActionType::SelectiveDelegationCostSaved
    );

    // Cost-saved est_savings_micros is u64 (non-negative by type) —
    // paper Section 6.1 cost-mitigation claim never produces a negative
    // savings number.
    if let SsbPayload::SelectiveDelegationCostSaved(p) = cost_saved {
        // Trivially true (u64); recorded here as a contract anchor so a
        // future schema change to i64 would re-trigger this fence.
        assert!(p.est_savings_micros < u64::MAX);
    }
}

// ---------------------------------------------------------------------------
// G6 — TaskDifficultyClassifier emits a score in [0, 1] for
// representative tasks (input-domain guard).
// ---------------------------------------------------------------------------
#[test]
fn g6_difficulty_score_in_unit_interval_for_representative_tasks() {
    let cases = [
        RoutingTask::new_text("g6-empty", ""),
        RoutingTask::new_text("g6-tiny", "x"),
        RoutingTask::new_text("g6-moderate", "implement a small refactor"),
        RoutingTask {
            text: "refactor the orchestrator architecture: investigate why the dispatcher \
                   integrate path fails. ```rust let x = 1; ``` how does it interact with \
                   the auditor design? compare two approaches and evaluate trade-off."
                .into(),
            file_count: 12,
            multi_turn: true,
            task_id: "g6-hard".into(),
        },
    ];
    for t in &cases {
        let s = TaskDifficultyClassifier::heuristic_score(t);
        assert!(
            (0.0..=1.0).contains(&s),
            "difficulty {s} for task `{}` must be in [0, 1]",
            t.task_id
        );
    }
}

// ---------------------------------------------------------------------------
// G7 — verifier-binary-only structural guard (paper invariant). The
// SelectiveDelegator emits Primitive::Verify ONLY when the worker pool
// contains an Auditor variant; the policy does NOT manufacture a
// verifier signal of its own.
// ---------------------------------------------------------------------------
#[test]
fn g7_verifier_remains_binary_only_no_synthetic_verify_primitive() {
    let policy =
        RoutingPolicy::new(true, 0.1, 0.10, 8, 8).expect("policy ok");
    let delegator = SelectiveDelegator::new(policy);

    // Auditor-less worker pool: no Verify primitive should appear in
    // the dispatched Vec.
    let no_auditor: BTreeMap<AgentRole, ()> = [
        AgentRole::Executor,
        AgentRole::Researcher,
        AgentRole::Designer,
        AgentRole::PromptEngineer,
        AgentRole::ContextCompiler,
        AgentRole::MemoryWriter,
        AgentRole::MediaValidator,
    ]
    .iter()
    .map(|r| (*r, ()))
    .collect();

    let hard = RoutingTask {
        text: "implement integrate debug refactor".into(),
        file_count: 5,
        multi_turn: true,
        task_id: "g7-no-auditor".into(),
    };
    let difficulty = TaskDifficultyClassifier::heuristic_score(&hard);
    let dispatched = delegator.select_workers(difficulty, &no_auditor, None);
    let synthetic_verify = dispatched
        .iter()
        .any(|(_, p)| matches!(p, Primitive::Verify));
    assert!(
        !synthetic_verify,
        "policy must NOT manufacture Verify primitive without an Auditor role \
         (paper verifier_binary_only invariant)"
    );
}

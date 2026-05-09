//! B-17 Gate Liveness Regression Suite — one test per safety gate that fires
//! today, plus deliberate stubs for gates whose production wiring is blocked
//! on B-2 / B-4 / B-6.
//!
//! This file is a SIBLING of `safety_gate_integration.rs`, not a replacement.
//! `safety_gate_integration.rs` is the deeper integration suite that B-8 left
//! behind and validates internal cycle-2 invariants (BLOCK-1 synthetic
//! grounding, W1 CSQ reason string, etc.). `gate_liveness.rs` is a much
//! smaller "does the gate fire at all when given non-vacuous input" surface.
//! The intent is: if B-N wiring regresses to a vacuous-pass state, at least
//! ONE of these tests goes red.
//!
//! Naming convention for wiring-blocked stubs:
//!   `<gate>_pending_b<N>_wiring` + `#[ignore = "..."]`
//!
//! `#[ignore]` is the project convention for tests that exercise library-level
//! behaviour ahead of production callers. They still appear in
//! `cargo test --test gate_liveness -- --list` so future B-N implementers can
//! flip them on by removing the attribute.
//!
//! Performance:
//! `gate_chain_under_400ms_with_individual_under_100ms` measures the four
//! wired gate evaluations sequentially with `std::time::Instant`. This is a
//! GATE-EVALUATION budget, NOT a full FSM transition through TaskRunner —
//! see workflow/reasoning/B-17-decisions.md DECISION-D for the scope split.

use std::sync::Mutex;
use std::time::{Duration, Instant};

use elai_orchestrator::evidence::{EvidenceGate, GroundingSweepGate, TransitionContext};
use elai_orchestrator::fsm::FsmState;
use elai_orchestrator::gate::{
    run_interactive_gate, EvidenceSummary, GateDecision, InteractiveGate,
    InteractiveGateOutcome, RiskLevel,
};
use elai_orchestrator::grc::{
    pipeline_grc_disabled_via_env, PipelineGrc, ELAI_DISABLE_PIPELINE_GRC_ENV,
};
use elai_orchestrator::pipeline_tree_navigator::BabyStepOutcome;
use elai_orchestrator::recovery::AgentDeathClassification;
use elai_orchestrator::supervision::{
    EscalationAction, SupervisionManager, SupervisionOutcome, SupervisionPolicy,
};

use elai_core::SeededClock;
use elai_knowledge::degradation::{DegradationCascade, NoopSsbNotifier, SearchMode};
use elai_knowledge::vector::circuit_breaker::{
    EmbeddingCircuitBreaker, FailureKind,
};

use elai_reasoning::{
    AgentGrcConfig, BasicGrcEngine, CsqSeverity, FeasibilityOutcome, GrcBudget, GrcEngine,
    GrcError, GrcState, GroundingLevel, ThinkStep,
};

use std::sync::Arc;
use uuid::Uuid;

// Process-wide lock matching safety_gate_integration.rs:42 — same name so a
// future refactor that promotes ENV_LOCK to a shared test_support module
// renames in lockstep across both files. NEVER rename without coordinating.
static ENV_LOCK: Mutex<()> = Mutex::new(());

// ---------------------------------------------------------------------------
// Shared helpers (mirror safety_gate_integration.rs patterns).
// ---------------------------------------------------------------------------

fn executor_config() -> AgentGrcConfig {
    AgentGrcConfig {
        role: "executor".into(),
        budget: GrcBudget {
            max_calls: 8,
            max_chars: 1200,
            calls_used: 0,
            chars_used: 0,
        },
        required_steps: vec![
            GrcState::Observe,
            GrcState::Hypothesize,
            GrcState::PlanAction,
            GrcState::Act,
            GrcState::Verify,
        ],
        grounding_level: GroundingLevel::High,
        csq_required: true,
        require_scope_enumeration: false,
        ..AgentGrcConfig::default()
    }
}

fn auditor_config() -> AgentGrcConfig {
    AgentGrcConfig {
        role: "auditor".into(),
        budget: GrcBudget {
            max_calls: 10,
            max_chars: 1200,
            calls_used: 0,
            chars_used: 0,
        },
        required_steps: vec![
            GrcState::Observe,
            GrcState::Hypothesize,
            GrcState::PlanAction,
            GrcState::Act,
            GrcState::Verify,
        ],
        grounding_level: GroundingLevel::High,
        csq_required: false,
        require_scope_enumeration: false,
        ..AgentGrcConfig::default()
    }
}

// Local mock for InteractiveGate. The in-crate MockInteractiveGate at
// gate.rs:302 is gated behind `#[cfg(test)]`, which means integration tests
// (this file) cannot depend on it — integration tests build the lib without
// the cfg(test) flag set on upstream crates. We therefore re-implement the
// minimum surface needed.
struct LocalMockGate {
    decisions: Mutex<Vec<GateDecision>>,
    call_count: Mutex<usize>,
}

impl LocalMockGate {
    fn new(decisions: Vec<GateDecision>) -> Self {
        Self {
            decisions: Mutex::new(decisions),
            call_count: Mutex::new(0),
        }
    }

    fn calls(&self) -> usize {
        *self.call_count.lock().unwrap_or_else(|p| p.into_inner())
    }
}

impl InteractiveGate for LocalMockGate {
    fn present_gate(
        &self,
        _summary: &EvidenceSummary,
        _risk_level: RiskLevel,
    ) -> GateDecision {
        let mut count = self.call_count.lock().unwrap_or_else(|p| p.into_inner());
        *count += 1;
        let mut decisions = self.decisions.lock().unwrap_or_else(|p| p.into_inner());
        if decisions.is_empty() {
            // Mirror MockInteractiveGate's empty-queue behaviour at gate.rs:339.
            GateDecision::Approve
        } else {
            decisions.remove(0)
        }
    }
}

// ---------------------------------------------------------------------------
// Wired gate 1/4: GroundingSweepGate — fires on ungrounded claim.
// ---------------------------------------------------------------------------

#[test]
fn grounding_sweep_gate_fires_on_ungrounded_claim() {
    // Liveness assertion: when the producer (PipelineGrc::ground_plan) reports
    // a non-empty ungrounded list, the consumer (GroundingSweepGate) MUST
    // reject the PLANNING -> TESTING_FIRST transition. If this regresses to
    // a vacuous Ok(()), the gate has gone dead.
    let grc = PipelineGrc::new(executor_config());
    let plan = "Decomposed: write a thing.\nGrounding: file:/no/such/path.rs\n";

    let grounding = grc.ground_plan(plan);
    assert!(
        !grounding.ungrounded.is_empty(),
        "B-17 liveness: ground_plan must surface the missing-file claim"
    );

    let ctx = TransitionContext {
        ungrounded_claims: Some(grounding.ungrounded.clone()),
        ..TransitionContext::default()
    };
    let result = GroundingSweepGate.check(&ctx);

    let err = result.expect_err(
        "B-17 liveness: GroundingSweepGate must REJECT non-empty ungrounded_claims",
    );
    assert_eq!(
        err.gate_name, "grounding_sweep",
        "gate_name attribution must survive future refactors"
    );
    // W1 closure: missing_artifacts must be non-empty so the dashboard can
    // surface which claims are ungrounded, not just that something failed.
    assert!(
        !err.missing_artifacts.is_empty(),
        "B-17 W1: missing_artifacts must be non-empty for dashboard attribution, got {:?}",
        err.missing_artifacts
    );
}

// ---------------------------------------------------------------------------
// Wired gate 2/4: CSQ Critical — destructive call without guard blocks.
// ---------------------------------------------------------------------------

#[test]
fn csq_critical_blocks_destructive_call() {
    // Liveness assertion: an op matching engine.rs's destructive keyword regex
    // ("fs::|file|write|delete|remove|net::|tcpstream|money|payment|auth|
    // token|credential|password") must yield at least one CsqSeverity::Critical
    // check whose guard_added flag is false. The runner reads this exact
    // shape at task_runner.rs:3762 to decide whether to fail the baby step.
    let _guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let prior = std::env::var(ELAI_DISABLE_PIPELINE_GRC_ENV).ok();
    std::env::remove_var(ELAI_DISABLE_PIPELINE_GRC_ENV);
    assert!(
        !pipeline_grc_disabled_via_env(),
        "kill switch must be off for the CSQ gate to be live"
    );

    let grc = PipelineGrc::new(executor_config());
    let ops = vec!["std::fs::write(path, contents)".to_string()];
    let result = grc
        .generate_csq("baby_step_b17", &ops)
        .expect("generate_csq must not error on a single op");

    let any_unguarded_critical = result
        .checks
        .iter()
        .any(|c| c.severity == CsqSeverity::Critical && !c.guard_added);
    assert!(
        any_unguarded_critical,
        "B-17 liveness: at least one Critical-without-guard check is required to block the runner, got {:?}",
        result.checks
    );

    // Restore.
    match prior {
        Some(v) => std::env::set_var(ELAI_DISABLE_PIPELINE_GRC_ENV, v),
        None => std::env::remove_var(ELAI_DISABLE_PIPELINE_GRC_ENV),
    }
}

// ---------------------------------------------------------------------------
// Wired gate 3/4: Inline feasibility Blocked propagates to BabyStepOutcome.
// ---------------------------------------------------------------------------

#[test]
fn inline_feasibility_blocks_propagate_to_baby_step_outcome() {
    // Liveness assertion: the engine's CONTRADICTS_PLAN-prefixed claim path
    // returns FeasibilityOutcome::Blocked, and the runner-side translation
    // (task_runner.rs:3825-3833) maps it to BabyStepOutcome::Failed with the
    // canonical "inline_feasibility_blocked:" reason prefix. If either side
    // regresses, dashboards lose attribution and the navigator can wrongly
    // mark the step Passed.
    let mut grc = PipelineGrc::new(executor_config());
    let claim = "CONTRADICTS_PLAN: novel-claim disagrees with original plan";
    let sources = vec!["original plan never mentioned this".to_string()];

    let result = grc
        .check_inline_feasibility(claim, &sources)
        .expect("check_inline_feasibility must not error on first call");
    assert_eq!(
        result.outcome,
        FeasibilityOutcome::Blocked,
        "B-17 liveness: CONTRADICTS_PLAN: marker must yield Blocked"
    );
    // W2 closure: the claim string must contain "CONTRADICTS_PLAN" so the
    // runner's reason prefix ("inline_feasibility_blocked: CONTRADICTS_PLAN: ...")
    // correctly attributes the block to the plan-contradiction gate.
    assert!(
        result.claim.contains("CONTRADICTS_PLAN"),
        "B-17 W2: claim must echo CONTRADICTS_PLAN marker for attribution, got {:?}",
        result.claim
    );

    let outcome = if result.outcome == FeasibilityOutcome::Blocked {
        BabyStepOutcome::Failed {
            reason: format!("inline_feasibility_blocked: {}", result.claim),
        }
    } else {
        BabyStepOutcome::Passed
    };

    match outcome {
        BabyStepOutcome::Failed { reason } => {
            assert!(
                reason.starts_with("inline_feasibility_blocked:"),
                "B-17 liveness: BabyStepOutcome reason must start with 'inline_feasibility_blocked:' so dashboards can attribute the gate, got {:?}",
                reason
            );
        }
        other => panic!(
            "expected Failed when feas.outcome == Blocked, got {:?}",
            other
        ),
    }
}

// ---------------------------------------------------------------------------
// Wired gate 4/4: Auditor anti-rubber-stamp.
// ---------------------------------------------------------------------------

#[test]
fn auditor_anti_rubber_stamp_rejects_findingless_converge() {
    // Liveness assertion: a fresh auditor engine, seeded with a single
    // Observe step (mirroring agent_pool::spawn at production), must reject a
    // direct Converge call that is not preceded by any FINDING:-prefixed
    // Verify step. If the gate goes vacuous, the auditor can rubber-stamp.
    let mut engine = BasicGrcEngine::new();
    engine.with_agent_config(auditor_config());
    engine
        .think(ThinkStep {
            step_type: GrcState::Observe,
            content: "auditor seed via agent_pool::spawn".into(),
            topic: "audit-b17".into(),
            grounding: Some("tool:agent_pool::spawn".into()),
            conclusion: None,
            scope: None,
        })
        .expect("seed Observe must record cleanly");

    let converge = engine.think(ThinkStep {
        step_type: GrcState::Converge,
        content: "ACTION: rubber-stamp without FINDING".into(),
        topic: "audit-b17".into(),
        grounding: Some("training:unverified".into()),
        conclusion: Some("ACTION: approve".into()),
        scope: None,
    });

    match converge {
        Err(GrcError::InsufficientFindings { role, expected_min }) => {
            assert_eq!(role, "auditor");
            assert_eq!(expected_min, 1);
        }
        other => panic!(
            "B-17 liveness: anti-rubber-stamp must return InsufficientFindings, got {:?}",
            other
        ),
    }
}

// ---------------------------------------------------------------------------
// Wiring-blocked stub 1/3: SearchMode cascade (B-4 wiring blocked).
// ---------------------------------------------------------------------------

// TODO(B-4): re-test once pipeline.rs:520 consults the cascade per query
// instead of hardcoding SearchMode::Hybrid. The library-level behaviour is
// already verified by elai_knowledge::degradation::tests, so this stub tests
// the same surface from the orchestrator integration boundary as a
// regression anchor for the future wiring.
#[test]
#[ignore = "blocked on B-4: pipeline.rs:520 hardcodes Hybrid; cascade is library-only today"]
fn search_mode_cascade_pending_b4_wiring() {
    // Liveness assertion (when wired): tripping the GraphCodeBERT and Granite
    // breakers must downgrade the cascade to Bm25Only when SPLADE is
    // disabled. Today the cascade is constructed once at startup (cli/main.rs
    // :3800-3808) and never re-evaluated; per-query path ignores it.
    let clock: Arc<dyn elai_core::ClockProvider> = Arc::new(SeededClock::new());
    let code_cb = Mutex::new(EmbeddingCircuitBreaker::with_defaults(
        "graphcodebert",
        clock.clone(),
    ));
    let text_cb = Mutex::new(EmbeddingCircuitBreaker::with_defaults(
        "granite",
        clock.clone(),
    ));
    let ssb = NoopSsbNotifier;
    let mut cascade = DegradationCascade::new(false);

    code_cb
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .record_failure(FailureKind::Permanent);
    text_cb
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .record_failure(FailureKind::Permanent);

    let mode = cascade.evaluate(Some(&code_cb), Some(&text_cb), None, &ssb);
    assert_eq!(
        mode,
        SearchMode::Bm25Only,
        "B-17 stub: with both dense breakers Open and SPLADE off, cascade must reach Bm25Only"
    );
}

// ---------------------------------------------------------------------------
// Wiring-blocked stub 2/3: SupervisionManager (B-6 wiring blocked).
// ---------------------------------------------------------------------------

// B-6 production wiring LANDED in task_runner.rs:4879 (session s-59964):
// `emit_baby_step_recovery` now calls `mgr.handle_death(...)` under an async
// mutex before invoking RecoveryManager. This library-level test verifies the
// escalation contract the production path depends on. Activated 2026-05-09
// (session s-91480).
#[test]
fn supervision_manager_escalation_pending_b6_wiring() {
    // Liveness assertion (when wired): with max_restarts=2, three crashes in
    // the same phase produce SupervisionOutcome::Escalated{Terminate} on the
    // third call (contract proven in supervision.rs::tests::three_crashes_then_escalate).
    let policy = SupervisionPolicy {
        max_restarts: 2,
        restart_window: Duration::from_secs(300),
        on_max_restarts: EscalationAction::Terminate,
    };
    let mut mgr = SupervisionManager::new(SeededClock::new(), policy);
    let id = Uuid::nil();

    mgr.handle_death(id, AgentDeathClassification::Crash, "phase-b17", 1, false)
        .expect("first crash must restart cleanly");
    mgr.handle_death(id, AgentDeathClassification::Crash, "phase-b17", 2, false)
        .expect("second crash must restart cleanly");

    let outcome = mgr
        .handle_death(id, AgentDeathClassification::Crash, "phase-b17", 3, false)
        .expect("handle_death must not error on Terminate escalation");

    match outcome {
        SupervisionOutcome::Escalated {
            restart_count,
            max_restarts,
            escalation_action,
            ..
        } => {
            assert_eq!(restart_count, 3);
            assert_eq!(max_restarts, 2);
            assert_eq!(escalation_action, EscalationAction::Terminate);
        }
        other => panic!(
            "B-17 stub: expected SupervisionOutcome::Escalated{{Terminate}}, got {:?}",
            other
        ),
    }
}

// ---------------------------------------------------------------------------
// Wiring-blocked stub 3/3: CliInteractiveGate (B-2 wiring blocked).
// ---------------------------------------------------------------------------

// TODO(B-2): re-test once CliInteractiveGate is instantiated and FSM
// transitions are wrapped by run_interactive_gate. cli/main.rs:7753 defines
// the struct but it is never constructed. In autonomous (no-dashboard) mode
// today, FSM transitions run unguarded — see B-17-decisions.md DECISION-E
// for the documented "treat as Reject" policy when the gate IS wired.
#[test]
#[ignore = "blocked on B-2: CliInteractiveGate defined at cli/main.rs:7753 but never instantiated"]
fn cli_interactive_gate_pending_b2_wiring() {
    // Liveness assertion (when wired): a gate returning GateDecision::Reject
    // must produce InteractiveGateOutcome::Rejected. The redirect/modify
    // paths are exercised by the in-crate gate.rs tests; this stub asserts
    // the simplest fail-closed contract for the autonomous-fallback path
    // documented in B-17-decisions.md.
    let mock = LocalMockGate::new(vec![GateDecision::Reject]);
    let ctx = TransitionContext::default();
    let outcome = run_interactive_gate(
        FsmState::Idle,
        FsmState::Planning,
        "B-17-stub",
        &ctx,
        &mock,
        None,
    );

    match outcome {
        InteractiveGateOutcome::Rejected => {
            assert_eq!(
                mock.calls(),
                1,
                "B-17 stub: gate must be presented exactly once before Reject short-circuits"
            );
        }
        other => panic!(
            "B-17 stub: expected InteractiveGateOutcome::Rejected from Reject decision, got {:?}",
            other
        ),
    }
}

// ---------------------------------------------------------------------------
// Latency budget — gate-evaluation only, NOT a full FSM transition.
// ---------------------------------------------------------------------------

#[test]
fn gate_chain_under_400ms_with_individual_under_100ms() {
    // Scope: this measures sequential gate-evaluation work (ground_plan +
    // GroundingSweepGate.check + generate_csq + check_inline_feasibility),
    // NOT a full FSM transition through TaskRunner. A TaskRunner-level perf
    // test would require a much larger harness; B-17 explicitly defers that
    // to a follow-up. See workflow/reasoning/B-17-decisions.md DECISION-D.
    //
    // Phase 2B measured ground_plan p99 = 188 µs (file:docs/thefinalglance/
    // audit-implementation/phase-2B.md). Each individual gate budget of
    // 100ms is therefore ~530x the measured worst case; the chain budget of
    // 400ms is ~2125x. We pick 100ms/400ms (matching the spec's
    // "<100ms latency added per FSM transition" wording) so a real
    // regression is caught long before the assertion.

    let _guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let prior = std::env::var(ELAI_DISABLE_PIPELINE_GRC_ENV).ok();
    std::env::remove_var(ELAI_DISABLE_PIPELINE_GRC_ENV);

    let mut grc = PipelineGrc::new(executor_config());
    let plan_with_real_citation =
        "Plan: trivial.\nGrounding: file:Cargo.toml\n";
    let ops = vec!["std::fs::write(p, c)".to_string()];
    let claim = "CONTRADICTS_PLAN: differs from plan";
    let sources = vec!["plan source".to_string()];

    let chain_start = Instant::now();

    // Gate 1: ground_plan.
    let g1_start = Instant::now();
    let grounding = grc.ground_plan(plan_with_real_citation);
    let g1_elapsed = g1_start.elapsed();

    // Gate 2: GroundingSweepGate.check.
    let g2_start = Instant::now();
    let ctx = TransitionContext {
        ungrounded_claims: Some(grounding.ungrounded.clone()),
        ..TransitionContext::default()
    };
    let _ = GroundingSweepGate.check(&ctx);
    let g2_elapsed = g2_start.elapsed();

    // Gate 3: generate_csq.
    let g3_start = Instant::now();
    let _csq = grc
        .generate_csq("baby_step_perf", &ops)
        .expect("generate_csq must succeed");
    let g3_elapsed = g3_start.elapsed();

    // Gate 4: check_inline_feasibility (single call — under per-engine
    // budget of 2 / phase).
    let g4_start = Instant::now();
    let _ = grc
        .check_inline_feasibility(claim, &sources)
        .expect("check_inline_feasibility must succeed first call");
    let g4_elapsed = g4_start.elapsed();

    let chain_elapsed = chain_start.elapsed();

    // Restore env.
    match prior {
        Some(v) => std::env::set_var(ELAI_DISABLE_PIPELINE_GRC_ENV, v),
        None => std::env::remove_var(ELAI_DISABLE_PIPELINE_GRC_ENV),
    }

    let per_gate_budget = Duration::from_millis(100);
    let chain_budget = Duration::from_millis(400);

    assert!(
        g1_elapsed < per_gate_budget,
        "B-17 latency: ground_plan took {:?}, exceeds {:?}",
        g1_elapsed, per_gate_budget
    );
    assert!(
        g2_elapsed < per_gate_budget,
        "B-17 latency: GroundingSweepGate.check took {:?}, exceeds {:?}",
        g2_elapsed, per_gate_budget
    );
    assert!(
        g3_elapsed < per_gate_budget,
        "B-17 latency: generate_csq took {:?}, exceeds {:?}",
        g3_elapsed, per_gate_budget
    );
    assert!(
        g4_elapsed < per_gate_budget,
        "B-17 latency: check_inline_feasibility took {:?}, exceeds {:?}",
        g4_elapsed, per_gate_budget
    );
    assert!(
        chain_elapsed < chain_budget,
        "B-17 latency: full gate chain took {:?}, exceeds {:?}",
        chain_elapsed, chain_budget
    );
}

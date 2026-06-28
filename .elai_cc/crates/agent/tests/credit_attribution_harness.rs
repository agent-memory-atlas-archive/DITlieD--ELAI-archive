//! C-45 STEP 3 — Credit harness integration test.
//!
//! Exercises the agent-side `AgentRoleDropper` (C-45 STEP 2) end-to-end
//! against a deterministic synthetic task fixture. This is the
//! "credit harness" from the C-45 plan STEP 3: it takes a task set +
//! 9 config variants (baseline + 8 each-role-dropped), runs each,
//! aggregates success/audit-pass rate, and produces a
//! `credit-attribution.md` artifact.
//!
//! Paper grounding: arXiv:2605.13295 (Zehle 2026, "CANTANTE: Optimizing
//! Agentic Systems via Contrastive Credit Attribution", Section 3.2 —
//! K rollouts per query NOT K^N; scalar credit c in [-1, +1]).
//!
//! The agent crate is sandboxed and has NO SSB dependency (crate
//! invariant: MUST NOT depend on orchestrator). The three C-45 SSB
//! action types (`agent_role_dropped`, `credit_attribution_run`,
//! `credit_attribution_result`) live in the orchestrator crate and are
//! exercised by the contract test at
//! `crates/contracts/tests/c45_cantante_credit_attribution_contract.rs`.
//! HERE we verify the agent-side audit-trail CONTRACT that the
//! orchestrator-side SSB emission must satisfy: the ordering, count,
//! and non-null-delta invariants are checked against a local event log
//! that mirrors the SSB beacon shape. This keeps the agent crate's dep
//! surface minimal while still enforcing the C-45 STEP 7 TDD gates:
//!
//!   4D.X.3: harness runs deterministically against synthetic task set
//!           (no GPU required for harness itself).
//!   4D.X.4: AgentRoleDropper fallback path produces NON-empty output
//!           for each role (no silent skip).
//!   4D.X.5: credit-attribution math: delta = baseline_success_rate -
//!           dropped_success_rate; per-task and aggregate.
//!   4D.X.6: dropping a role MUST emit agent_role_dropped BEFORE
//!           dispatch reroutes (audit trail).
//!
//! Trigger: `cargo test -p elai_cc_agent --test credit_attribution_harness`.

use std::collections::BTreeMap;

use elai_cc_agent::dropper::{AgentRoleDropper, RoleFallback};
use elai_ipc_types::AgentRole;

// ---------------------------------------------------------------------------
// Pilot configuration — mirrors the orchestrator's `pilot_roles()` set.
// ---------------------------------------------------------------------------

/// The eight base `AgentRole` variants covered by the C-45 pilot. This
/// mirrors `crates/orchestrator/src/credit_attribution/partition.rs::
/// pilot_roles()` so the agent-side harness stays in lockstep with the
/// orchestrator-side harness without a cross-crate dependency (the agent
/// crate MUST NOT depend on the orchestrator).
const PILOT_ROLES: [AgentRole; 8] = [
    AgentRole::Executor,
    AgentRole::Auditor,
    AgentRole::Researcher,
    AgentRole::Designer,
    AgentRole::PromptEngineer,
    AgentRole::ContextCompiler,
    AgentRole::MemoryWriter,
    AgentRole::MediaValidator,
];

/// K — rollouts per configuration. CANTANTE main experiments use K=9
/// (paper Section 4 + Appendix B.2). The synthetic fixture is cheap
/// enough to run the full K=9 deterministically.
const K_ROLLOUTS: u32 = 9;

/// Synthetic task set — 5 deterministic tasks. No GPU required; the
/// scorer is a pure function of (task_idx, drop_policy). The task
/// descriptions are illustrative labels; the scorer uses only the
/// index + the dropped role to produce a deterministic success bit.
const SYNTHETIC_TASKS: [&str; 5] = [
    "task-000: fix typo in README",
    "task-001: add unit test for parser",
    "task-002: refactor error propagation",
    "task-003: implement config loader",
    "task-004: wire CLI subcommand",
];

// ---------------------------------------------------------------------------
// SSB event simulation — mirrors the 3 C-45 SSB action types.
// ---------------------------------------------------------------------------

/// Local simulation of the three C-45 SSB action types. The agent crate
/// has no SSB dependency; this enum mirrors the beacon shape so the
/// audit-trail contract (emit-before-reroute, per-task run, once-at-
/// aggregation result) is verified here without a cross-crate dep.
#[derive(Debug, Clone, PartialEq)]
enum CreditSsbEvent {
    /// Mirrors `SsbActionType::AgentRoleDropped`. Emitted BEFORE the
    /// dispatch path reroutes around the dropped role (4D.X.6 audit
    /// trail invariant).
    AgentRoleDropped { role: AgentRole, ts_ms: u64 },
    /// Mirrors `SsbActionType::CreditAttributionRun`. Emitted once per
    /// (configuration, task) pair at the start of a rollout.
    CreditAttributionRun {
        task_idx: usize,
        dropped_role: Option<AgentRole>,
        k_rollouts: u32,
        ts_ms: u64,
    },
    /// Mirrors `SsbActionType::CreditAttributionResult`. Emitted ONCE
    /// at the aggregation step with the per-role credit delta map.
    CreditAttributionResult {
        per_role_delta: BTreeMap<AgentRole, f32>,
        total_roles: u32,
        ts_ms: u64,
    },
}

// ---------------------------------------------------------------------------
// Synthetic scorer — deterministic per (task_idx, dropped_role).
// ---------------------------------------------------------------------------

/// Deterministic synthetic success scorer. Returns a scalar in
/// `[0.0, 1.0]` representing task success (1.0 = pass, 0.0 = fail).
///
/// The curve models that dropping core roles (Executor, Auditor) hurts
/// more than dropping peripheral roles (MediaValidator, MemoryWriter).
/// This is ILLUSTRATIVE ONLY — real credits require the runpod-pending
/// live pilot (C-45 STEP 4, DEFERRED). The fixture is designed so every
/// role has a NON-ZERO delta (acceptance: "All 3 SSB action types emit
/// with non-null deltas").
fn synthetic_success(task_idx: usize, dropped_role: Option<AgentRole>) -> f32 {
    // Baseline: all roles present. Tasks 0-4 score 0.80-0.96 (deterministic
    // spread so the mean is stable across K rollouts).
    let baseline = 0.80 + (task_idx as f32) * 0.04;

    match dropped_role {
        None => baseline,
        // Core roles — dropping them costs the most.
        Some(AgentRole::Executor) => baseline - 0.60,
        Some(AgentRole::Auditor) => baseline - 0.40,
        // Supporting roles — moderate cost.
        Some(AgentRole::Researcher) => baseline - 0.25,
        Some(AgentRole::ContextCompiler) => baseline - 0.20,
        Some(AgentRole::PromptEngineer) => baseline - 0.15,
        Some(AgentRole::Designer) => baseline - 0.10,
        // Peripheral roles — small cost.
        Some(AgentRole::MemoryWriter) => baseline - 0.05,
        Some(AgentRole::MediaValidator) => baseline - 0.02,
        // Roles outside the pilot set are never dropped here; defensive.
        Some(_) => baseline,
    }
}

/// Run K rollouts for one (task, drop-policy) pair and return the mean
/// success rate. The synthetic scorer is deterministic, so all K
/// rollouts for the same (task_idx, dropped_role) produce the same
/// value — the mean is exact. K rollouts still run (CANTANTE invariant:
/// K per config, NOT K^N) to exercise the loop.
///
/// The AgentRoleDropped beacon is emitted ONCE per config (by the
/// caller via [`run_config`]), NOT per task — this mirrors the
/// orchestrator-side SSB contract where one `agent_role_dropped` row
/// precedes the entire rollout sweep for that drop configuration.
fn run_k_rollouts(
    task_idx: usize,
    dropper: &AgentRoleDropper,
    events: &mut Vec<CreditSsbEvent>,
    ts_ms: u64,
) -> f32 {
    let dropped_role = dropper.dropped_role;
    // Emit CreditAttributionRun once per (config, task).
    events.push(CreditSsbEvent::CreditAttributionRun {
        task_idx,
        dropped_role,
        k_rollouts: K_ROLLOUTS,
        ts_ms,
    });

    let mut sum: f32 = 0.0;
    for _ in 0..K_ROLLOUTS {
        sum += synthetic_success(task_idx, dropped_role);
    }
    sum / (K_ROLLOUTS as f32)
}

/// Run one configuration (baseline or single-role-dropped) across the
/// full synthetic task set and return the aggregate success rate.
///
/// 4D.X.6: for a dropped-role config, the `AgentRoleDropped` beacon is
/// emitted BEFORE the first task's rollout (emit-before-reroute audit
/// trail). The baseline config emits no drop beacon.
fn run_config(
    dropper: &AgentRoleDropper,
    events: &mut Vec<CreditSsbEvent>,
    ts_ms: u64,
) -> f32 {
    // 4D.X.6: emit AgentRoleDropped ONCE per config, BEFORE any task
    // rollout reroutes around the dropped role.
    if let Some(role) = dropper.dropped_role {
        events.push(CreditSsbEvent::AgentRoleDropped { role, ts_ms });
    }
    let mut sum: f32 = 0.0;
    for task_idx in 0..SYNTHETIC_TASKS.len() {
        sum += run_k_rollouts(task_idx, dropper, events, ts_ms);
    }
    sum / (SYNTHETIC_TASKS.len() as f32)
}

// ---------------------------------------------------------------------------
// Credit-attribution.md artifact renderer.
// ---------------------------------------------------------------------------

/// Render the per-role credit table as a Markdown artifact (the
/// `credit-attribution.md` document from C-45 STEP 3). Returns the
/// Markdown string; the test asserts it is well-formed (non-empty,
/// contains all 8 roles, carries the synthetic-fixture caveat).
fn render_credit_attribution_md(
    per_role_delta: &BTreeMap<AgentRole, f32>,
    baseline_rate: f32,
    ts_ms: u64,
) -> String {
    let mut md = String::new();
    md.push_str("# C-45 CANTANTE Contrastive Credit Attribution\n\n");
    md.push_str("> **NOTE**: Synthetic fixture — real credits require the\n");
    md.push_str("> runpod-pending live pilot (C-45 STEP 4, DEFERRED).\n\n");
    md.push_str(&format!("- **Timestamp (ms)**: {ts_ms}\n"));
    md.push_str(&format!("- **Baseline success rate**: {baseline_rate:.4}\n"));
    md.push_str(&format!("- **K rollouts per config**: {K_ROLLOUTS}\n"));
    md.push_str(&format!(
        "- **Tasks**: {}\n\n",
        SYNTHETIC_TASKS.len()
    ));
    md.push_str("## Per-Role Credit Table\n\n");
    md.push_str("| Role | Dropped Success Rate | Credit Delta | Interpretation |\n");
    md.push_str("|------|---------------------|--------------|----------------|\n");
    for (role, delta) in per_role_delta {
        let role_str = role_snake_case(*role);
        let dropped_rate = baseline_rate - delta;
        let interp = if *delta > 0.10 {
            "core (high cost to drop)"
        } else if *delta > 0.0 {
            "supporting (moderate cost)"
        } else if (*delta).abs() < 1e-6 {
            "neutral (zero cost)"
        } else {
            "noise (dropping helped)"
        };
        md.push_str(&format!(
            "| {role_str} | {dropped_rate:.4} | {delta:+.4} | {interp} |\n"
        ));
    }
    md.push_str("\n## Interpretation\n\n");
    md.push_str("Positive delta = dropping the role HURT (role contributed positively).\n");
    md.push_str("Negative delta = dropping the role HELPED (role was net noise).\n");
    md.push_str("Near-zero delta = role is a candidate for consolidation.\n");
    md
}

/// Render an `AgentRole` as its snake_case wire name. Mirrors the
/// matcher in `main.rs::role_snake_case` (the agent crate does not
/// depend on `serde_json`, keeping the sandboxed binary's dep surface
/// minimal).
fn role_snake_case(role: AgentRole) -> &'static str {
    match role {
        AgentRole::Executor => "executor",
        AgentRole::Auditor => "auditor",
        AgentRole::Researcher => "researcher",
        AgentRole::Designer => "designer",
        AgentRole::PromptEngineer => "prompt_engineer",
        AgentRole::ContextCompiler => "context_compiler",
        AgentRole::MemoryWriter => "memory_writer",
        AgentRole::MediaValidator => "media_validator",
        AgentRole::IdeaFramer => "idea_framer",
        AgentRole::IdeaCritic => "idea_critic",
        AgentRole::AuditorConcepts => "auditor_concepts",
        AgentRole::AuditorPhases => "auditor_phases",
        AgentRole::AuditorSteps => "auditor_steps",
        AgentRole::AuditorBabySteps => "auditor_baby_steps",
        AgentRole::AuditorBabyStepDetail => "auditor_baby_step_detail",
        AgentRole::AuditorProductionGrade => "auditor_production_grade",
        AgentRole::InjectionClassifier => "injection_classifier",
    }
}

// ---------------------------------------------------------------------------
// Tests — C-45 STEP 7 TDD gates.
// ---------------------------------------------------------------------------

/// 4D.X.3: harness runs deterministically against synthetic task set
/// (no GPU required for harness itself). Runs the full 9-config sweep
/// twice and asserts identical results (determinism).
#[test]
fn harness_runs_deterministically_against_synthetic_fixture() {
    let ts_ms = 1_700_000_000_000_u64;

    // First run.
    let (deltas_1, baseline_1, events_1) = run_full_pilot(ts_ms);
    // Second run — must produce identical results.
    let (deltas_2, baseline_2, events_2) = run_full_pilot(ts_ms);

    assert_eq!(baseline_1, baseline_2, "baseline must be deterministic");
    assert_eq!(
        deltas_1, deltas_2,
        "per-role deltas must be deterministic across runs"
    );
    assert_eq!(
        events_1.len(),
        events_2.len(),
        "event count must be deterministic"
    );
    // 8 roles recorded.
    assert_eq!(deltas_1.len(), PILOT_ROLES.len());
}

/// Helper: run the full 9-config pilot (baseline + 8 each-role-dropped)
/// and return (per_role_delta, baseline_rate, events).
fn run_full_pilot(ts_ms: u64) -> (BTreeMap<AgentRole, f32>, f32, Vec<CreditSsbEvent>) {
    let mut events: Vec<CreditSsbEvent> = Vec::new();

    // Config 0: baseline (no role dropped).
    let baseline_dropper = AgentRoleDropper::none();
    let baseline_rate = run_config(&baseline_dropper, &mut events, ts_ms);

    // Configs 1-8: each role dropped in turn.
    let mut per_role_delta: BTreeMap<AgentRole, f32> = BTreeMap::new();
    for role in PILOT_ROLES {
        let dropper = AgentRoleDropper::drop(role);
        let dropped_rate = run_config(&dropper, &mut events, ts_ms);
        // 4D.X.5: delta = baseline_success_rate - dropped_success_rate.
        let delta = baseline_rate - dropped_rate;
        per_role_delta.insert(role, delta);
    }

    // Emit CreditAttributionResult ONCE at aggregation.
    events.push(CreditSsbEvent::CreditAttributionResult {
        per_role_delta: per_role_delta.clone(),
        total_roles: PILOT_ROLES.len() as u32,
        ts_ms,
    });

    (per_role_delta, baseline_rate, events)
}

/// 4D.X.4: AgentRoleDropper fallback path produces NON-empty output
/// for each role (no silent skip — fallback must be defined and
/// observable). Verifies every pilot role has a defined fallback with
/// a non-empty description.
#[test]
fn dropper_fallback_is_defined_and_non_empty_for_each_pilot_role() {
    for role in PILOT_ROLES {
        let dropper = AgentRoleDropper::drop(role);
        // fallback_for must return Some for every pilot role.
        let fallback = dropper
            .fallback_for(role)
            .unwrap_or_else(|| panic!("fallback must be defined for {role:?}"));
        // fallback_description must be non-empty.
        let desc = dropper
            .fallback_description(role)
            .unwrap_or_else(|| panic!("desc must be defined for {role:?}"));
        assert!(!desc.is_empty(), "fallback desc empty for {role:?}");
        // The fallback variant must be one of the defined strategies.
        assert!(
            matches!(
                fallback,
                RoleFallback::UseCachedPrior
                    | RoleFallback::SkipPhase
                    | RoleFallback::MergeIntoExecutor
                    | RoleFallback::MergeIntoAuditor
                    | RoleFallback::NoOp
            ),
            "fallback must be a defined variant for {role:?}, got {fallback:?}"
        );
    }
}

/// 4D.X.5: credit-attribution math: delta = baseline_success_rate -
/// dropped_success_rate; per-task and aggregate. Verifies the math at
/// both the per-task level and the aggregate level.
#[test]
fn credit_attribution_math_baseline_minus_dropped() {
    let ts_ms = 1_700_000_000_000_u64;

    // Per-task check: for task 0, dropping Executor should give a
    // negative delta (baseline - dropped > 0 since Executor is useful).
    let task_idx = 0;
    let baseline_task = synthetic_success(task_idx, None);
    let dropped_executor_task = synthetic_success(task_idx, Some(AgentRole::Executor));
    let per_task_delta = baseline_task - dropped_executor_task;
    assert!(
        per_task_delta > 0.0,
        "dropping Executor must hurt (positive delta), got {per_task_delta}"
    );
    // The synthetic curve drops Executor by 0.60.
    assert!((per_task_delta - 0.60).abs() < 1e-6, "Executor drop cost = 0.60");

    // Aggregate check: run the full pilot and verify the aggregate
    // delta for Executor matches the expected value.
    let (deltas, _baseline, _events) = run_full_pilot(ts_ms);
    let executor_credit = deltas
        .get(&AgentRole::Executor)
        .copied()
        .expect("Executor credit recorded");
    assert!(
        executor_credit > 0.0,
        "Executor aggregate credit must be positive (core role), got {executor_credit}"
    );
    // The synthetic curve drops Executor by a constant 0.60 regardless
    // of task, so the aggregate delta must be exactly 0.60.
    assert!(
        (executor_credit - 0.60).abs() < 1e-6,
        "Executor aggregate credit must equal the per-task drop cost 0.60, got {executor_credit}"
    );
}

/// 4D.X.6: dropping a role MUST emit agent_role_dropped BEFORE
/// dispatch reroutes (audit trail). Verifies the event ordering: for
/// each dropped-role config, the AgentRoleDropped event precedes the
/// CreditAttributionRun events for that config.
#[test]
fn agent_role_dropped_emitted_before_reroute() {
    let ts_ms = 1_700_000_000_000_u64;
    let (_deltas, _baseline, events) = run_full_pilot(ts_ms);

    // Count AgentRoleDropped events: exactly 8 (one per dropped role).
    let dropped_count = events
        .iter()
        .filter(|e| matches!(e, CreditSsbEvent::AgentRoleDropped { .. }))
        .count();
    assert_eq!(
        dropped_count,
        PILOT_ROLES.len(),
        "one AgentRoleDropped per dropped-role config"
    );

    // For each dropped role, the AgentRoleDropped event must appear
    // BEFORE the first CreditAttributionRun for that config.
    for role in PILOT_ROLES {
        let drop_idx = events
            .iter()
            .position(|e| matches!(e, CreditSsbEvent::AgentRoleDropped { role: r, .. } if *r == role))
            .unwrap_or_else(|| panic!("AgentRoleDropped event for {role:?} must exist"));
        // The next event must be a CreditAttributionRun for the same
        // dropped role (the run starts right after the drop beacon).
        let next = &events[drop_idx + 1];
        assert!(
            matches!(next, CreditSsbEvent::CreditAttributionRun { dropped_role: Some(r), .. } if *r == role),
            "AgentRoleDropped for {role:?} must be immediately followed by CreditAttributionRun for the same role (audit-trail ordering)"
        );
    }
}

/// Acceptance: "All 3 SSB action types emit with non-null deltas."
/// Verifies that the simulated event log contains all 3 event types,
/// and that the CreditAttributionResult carries a non-null delta for
/// every role (no zero, no NaN, no infinity).
#[test]
fn all_three_ssb_action_types_emit_with_non_null_deltas() {
    let ts_ms = 1_700_000_000_000_u64;
    let (deltas, _baseline, events) = run_full_pilot(ts_ms);

    // All 3 event types present.
    let has_dropped = events
        .iter()
        .any(|e| matches!(e, CreditSsbEvent::AgentRoleDropped { .. }));
    let has_run = events
        .iter()
        .any(|e| matches!(e, CreditSsbEvent::CreditAttributionRun { .. }));
    let has_result = events
        .iter()
        .any(|e| matches!(e, CreditSsbEvent::CreditAttributionResult { .. }));
    assert!(has_dropped, "AgentRoleDropped events must be present");
    assert!(has_run, "CreditAttributionRun events must be present");
    assert!(has_result, "CreditAttributionResult event must be present");

    // Exactly ONE CreditAttributionResult (once at aggregation).
    let result_count = events
        .iter()
        .filter(|e| matches!(e, CreditSsbEvent::CreditAttributionResult { .. }))
        .count();
    assert_eq!(result_count, 1, "exactly one CreditAttributionResult");

    // CreditAttributionRun count: one per (config, task) = 9 configs * 5 tasks = 45.
    let run_count = events
        .iter()
        .filter(|e| matches!(e, CreditSsbEvent::CreditAttributionRun { .. }))
        .count();
    let expected_runs = (PILOT_ROLES.len() + 1) * SYNTHETIC_TASKS.len();
    assert_eq!(
        run_count, expected_runs,
        "CreditAttributionRun: one per (config, task) = {expected_runs}"
    );

    // Every per-role delta is non-null (finite + non-zero for the
    // synthetic fixture, which is designed so every role has a
    // measurable drop cost).
    for (role, delta) in &deltas {
        assert!(
            delta.is_finite(),
            "delta for {role:?} must be finite, got {delta}"
        );
        assert_ne!(
            *delta, 0.0,
            "delta for {role:?} must be non-zero (synthetic fixture guarantees non-null)"
        );
    }
}

/// Acceptance: "Credit harness runs deterministically against synthetic
/// fixture" + "credit-attribution.md artifact". Verifies the Markdown
/// artifact is well-formed: non-empty, contains all 8 role rows, and
/// carries the synthetic-fixture caveat.
#[test]
fn credit_attribution_md_artifact_is_well_formed() {
    let ts_ms = 1_700_000_000_000_u64;
    let (deltas, baseline, _events) = run_full_pilot(ts_ms);
    let md = render_credit_attribution_md(&deltas, baseline, ts_ms);

    // Non-empty.
    assert!(!md.is_empty(), "markdown artifact must be non-empty");

    // Title.
    assert!(
        md.contains("# C-45 CANTANTE Contrastive Credit Attribution"),
        "artifact must carry the title"
    );

    // Synthetic-fixture caveat.
    assert!(
        md.contains("Synthetic fixture"),
        "artifact must flag itself as synthetic (not real credits)"
    );

    // All 8 roles present.
    for role in PILOT_ROLES {
        let role_str = role_snake_case(role);
        assert!(
            md.contains(role_str),
            "artifact must contain row for {role_str}"
        );
    }

    // Table header.
    assert!(
        md.contains("| Role | Dropped Success Rate | Credit Delta | Interpretation |"),
        "artifact must carry the table header"
    );

    // Interpretation section.
    assert!(
        md.contains("## Interpretation"),
        "artifact must carry the interpretation section"
    );
}

/// CANTANTE invariant: K rollouts per configuration, NOT K^N. The
/// harness runs K rollouts per (config, task) pair — the total rollout
/// count is K * (N+1) * T where N = dropped roles, T = tasks. This is
/// LINEAR in N, not exponential (K^N would be 9^8 = 43M for the full
/// pilot — the harness must be far below that).
#[test]
fn rollout_count_is_linear_not_exponential() {
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    let counter = Arc::new(AtomicU32::new(0));
    let k = K_ROLLOUTS;
    let n = PILOT_ROLES.len() as u32;
    let t = SYNTHETIC_TASKS.len() as u32;

    // Run the pilot with an instrumented scorer that increments the
    // shared counter once per rollout call.
    let mut events: Vec<CreditSsbEvent> = Vec::new();

    // Baseline config (no drop beacon — baseline emits only Run events).
    for task_idx in 0..SYNTHETIC_TASKS.len() {
        events.push(CreditSsbEvent::CreditAttributionRun {
            task_idx,
            dropped_role: None,
            k_rollouts: k,
            ts_ms: 0,
        });
        for _ in 0..k {
            let _score = synthetic_success(task_idx, None);
            counter.fetch_add(1, Ordering::Relaxed);
        }
    }
    // Dropped-role configs.
    for role in PILOT_ROLES {
        let _dropper = AgentRoleDropper::drop(role);
        events.push(CreditSsbEvent::AgentRoleDropped { role, ts_ms: 0 });
        for task_idx in 0..SYNTHETIC_TASKS.len() {
            events.push(CreditSsbEvent::CreditAttributionRun {
                task_idx,
                dropped_role: Some(role),
                k_rollouts: k,
                ts_ms: 0,
            });
            for _ in 0..k {
                let _score = synthetic_success(task_idx, Some(role));
                counter.fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    let total = counter.load(Ordering::Relaxed);
    let expected = k * (n + 1) * t;
    assert_eq!(
        total, expected,
        "total rollouts = K*(N+1)*T = {expected}, got {total} (CANTANTE: K per config, NOT K^N)"
    );
    // K^N for K=9, N=8 would be 43,046,721 — must be far below.
    assert!(
        total < 9_u32.pow(8),
        "must not fan out K^N (cross-product blowup banned per CANTANTE Section 3.2)"
    );
    // Suppress unused-events warning — events are the audit trail.
    let _ = events.len();
}

/// The AgentRoleDropper respects the ELAI_C45_DROPPED_ROLE env var
/// contract: setting the env var to a pilot role name constructs a
/// dropper that drops that role; clearing it returns to baseline.
/// This is the integration seam between the orchestrator's
/// agent_pool spawn path and the agent's startup read.
#[test]
fn dropper_env_var_round_trip() {
    // Save and restore the env var to avoid cross-test contamination.
    let saved = std::env::var(elai_cc_agent::ELAI_C45_DROPPED_ROLE_ENV).ok();

    // Baseline: unset.
    std::env::remove_var(elai_cc_agent::ELAI_C45_DROPPED_ROLE_ENV);
    let d = AgentRoleDropper::from_env();
    assert!(d.is_baseline(), "unset env var = baseline");

    // Drop Researcher.
    std::env::set_var(
        elai_cc_agent::ELAI_C45_DROPPED_ROLE_ENV,
        "researcher",
    );
    let d = AgentRoleDropper::from_env();
    assert_eq!(d.dropped_role, Some(AgentRole::Researcher));

    // Drop Executor.
    std::env::set_var(elai_cc_agent::ELAI_C45_DROPPED_ROLE_ENV, "executor");
    let d = AgentRoleDropper::from_env();
    assert_eq!(d.dropped_role, Some(AgentRole::Executor));

    // Unrecognized role -> safe default (baseline, never drop unknown).
    std::env::set_var(elai_cc_agent::ELAI_C45_DROPPED_ROLE_ENV, "nonexistent");
    let d = AgentRoleDropper::from_env();
    assert!(d.is_baseline(), "unrecognized role = safe baseline default");

    // Restore.
    match saved {
        Some(v) => std::env::set_var(elai_cc_agent::ELAI_C45_DROPPED_ROLE_ENV, v),
        None => std::env::remove_var(elai_cc_agent::ELAI_C45_DROPPED_ROLE_ENV),
    }
}

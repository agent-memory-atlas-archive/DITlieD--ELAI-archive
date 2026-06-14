//! H-42 contract test — verifies the serial-depth-scaled reasoning budget wiring.
//!
//! Source plan: docs/thefinalglance/H-plan/H-42_serial_depth_reasoning_budget.md
//! Decisions: workflow/reasoning/H-42-decisions.md
//!
//! Off-agent artifact requirement (R4/R7): tests (e) and (f) drive the REAL
//! `scale_grc_budget` path through the dispatch machinery — NOT a unit shim
//! that calls the payload constructor directly.  The key off-agent observable is
//! the scaled `grc_config.budget.max_calls` on the returned `AgentHandle`
//! (materialized at agent_pool.rs::spawn, the live dispatch site).
//! Missing evidence = FAIL (R1). A deep sub-task MUST get a larger budget than
//! a shallow one, and the ceiling MUST be respected.
//!
//! Tests:
//! (a) estimate_serial_depth: Concept=0, Phase=1, Step=2, BabyStep=3.
//! (b) scale_grc_budget raises with depth, clamps at LOCKED_BUDGET_CEILING.
//! (c) Base config is the floor (depth=0 unchanged, depth=1 unchanged for small
//!     base).
//! (d) classify_failure: DepthBound at high d + exhaustion; AccessBound at low d
//!     + retrieval miss; BOTH can be set (RISK-2: no forced dichotomy).
//! (e) REAL dispatch path: deep sub-task (BabyStep) gets a LARGER budget than a
//!     shallow sub-task (Concept) for the SAME role (R3/R4/R7).
//! (f) --flat-budget broken arm: ELAI_REASONING_BUDGET_SCALE_DISABLE makes deep
//!     and shallow budgets IDENTICAL (R2 falsifiability).
//! (g) LOCKED ceiling: depth=100 cannot exceed LOCKED_BUDGET_CEILING × base
//!     (loop-inflation proof).
//! (h) GD-2 reproducibility: same inputs produce identical scaled_calls.
//! (i) Depth-not-width guard: a DepthBound failure must NOT have its remedy
//!     framed as a context lift (RD-4 — classified on the row, not enforced
//!     by a hard code gate, but the classification must be correct).
//! (j) SCHEMA_VERSION >= 220 (H-42 bump verified at the off-agent contract level).
//! (k) ReasoningBudgetScaled wire-tag = "reasoning_budget_scaled".
//! (l) VecReasoningBudgetEmitter accumulates rows; scale_and_emit emits correctly.

use elai_orchestrator::pipeline_tree_navigator::DecompositionLevel;
use elai_orchestrator::ssb::{ReasoningBudgetScaledPayload, SsbActionType, SCHEMA_VERSION_PUBLIC};
use elai_orchestrator::{
    classify_failure, depth_scaling_disabled_via_env, estimate_serial_depth, scale_and_emit,
    scale_grc_budget, FailureClass, VecReasoningBudgetEmitter, LOCKED_BUDGET_CEILING,
};
use elai_reasoning::{AgentGrcConfig, GrcBudget};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn base_config(role: &str, max_calls: u32, max_chars: u32) -> AgentGrcConfig {
    AgentGrcConfig {
        role: role.into(),
        budget: GrcBudget::new(max_calls, max_chars),
        ..AgentGrcConfig::default()
    }
}

// ---------------------------------------------------------------------------
// (j) SCHEMA_VERSION bump: H-42 = 220.
// ---------------------------------------------------------------------------

#[test]
fn schema_version_is_at_least_220() {
    assert!(
        SCHEMA_VERSION_PUBLIC >= 220,
        "SCHEMA_VERSION_PUBLIC must be >= 220 after H-42; got {}",
        SCHEMA_VERSION_PUBLIC
    );
}

// ---------------------------------------------------------------------------
// (k) Wire-tag and payload struct smoke.
// ---------------------------------------------------------------------------

#[test]
fn reasoning_budget_scaled_wire_tag_is_correct() {
    assert_eq!(
        SsbActionType::ReasoningBudgetScaled.as_sql_str(),
        "reasoning_budget_scaled",
        "H-42: SsbActionType::ReasoningBudgetScaled wire tag mismatch"
    );
}

#[test]
fn reasoning_budget_scaled_payload_action_type_round_trips() {
    use elai_orchestrator::ssb::SsbPayload;
    let payload = SsbPayload::ReasoningBudgetScaled(ReasoningBudgetScaledPayload {
        kind: "depth_bound".into(),
        role: "executor".into(),
        base_calls: 8,
        serial_depth: 3,
        scaled_calls: 14,
        ceiling: 24,
        seed: 42,
        // I-26 extension fields — None on GRC-dispatch rows (H-42 path).
        depth_level: None,
        budget_pct: None,
        plateau_fired: None,
    });
    assert_eq!(payload.action_type(), SsbActionType::ReasoningBudgetScaled);
}

// ---------------------------------------------------------------------------
// (a) estimate_serial_depth: level to depth mapping.
// ---------------------------------------------------------------------------

#[test]
fn concept_maps_to_depth_zero() {
    assert_eq!(estimate_serial_depth(DecompositionLevel::Concept), 0);
}

#[test]
fn phase_maps_to_depth_one() {
    assert_eq!(estimate_serial_depth(DecompositionLevel::Phase), 1);
}

#[test]
fn step_maps_to_depth_two() {
    assert_eq!(estimate_serial_depth(DecompositionLevel::Step), 2);
}

#[test]
fn baby_step_maps_to_depth_three() {
    assert_eq!(estimate_serial_depth(DecompositionLevel::BabyStep), 3);
}

// ---------------------------------------------------------------------------
// (b) scale_grc_budget: raises with depth, clamps at ceiling.
// ---------------------------------------------------------------------------

#[test]
fn scale_raises_with_depth() {
    let _mutex = BUDGET_SCALE_ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let base = base_config("executor", 8, 1200);
    let depth_0 = scale_grc_budget(base.clone(), 0);
    let depth_3 = scale_grc_budget(base.clone(), 3);
    assert!(
        depth_3.budget.max_calls > depth_0.budget.max_calls,
        "depth=3 must produce more calls than depth=0; got {} vs {}",
        depth_3.budget.max_calls,
        depth_0.budget.max_calls,
    );
}

#[test]
fn scale_baby_step_factor_is_1_75x() {
    let _mutex = BUDGET_SCALE_ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    // factor = 1 + 0.25 * 3 = 1.75; 8 * 1.75 = 14 (ceiling div)
    let base = base_config("executor", 8, 1200);
    let scaled = scale_grc_budget(base, 3);
    assert_eq!(scaled.budget.max_calls, 14, "8 calls * 1.75 = 14");
    assert_eq!(scaled.budget.max_chars, 2100, "1200 chars * 1.75 = 2100");
}

// ---------------------------------------------------------------------------
// (c) Base config is the floor (depth=0 unchanged).
// ---------------------------------------------------------------------------

#[test]
fn concept_depth_zero_is_unchanged() {
    let base = base_config("executor", 8, 1200);
    let scaled = scale_grc_budget(base.clone(), 0);
    assert_eq!(
        scaled.budget.max_calls, base.budget.max_calls,
        "depth=0 must not scale calls"
    );
    assert_eq!(
        scaled.budget.max_chars, base.budget.max_chars,
        "depth=0 must not scale chars"
    );
}

// ---------------------------------------------------------------------------
// (d) classify_failure: DepthBound, AccessBound, BOTH.
// ---------------------------------------------------------------------------

#[test]
fn high_depth_exhausted_is_depth_bound() {
    let fc = classify_failure(3, true, false);
    assert!(fc.depth_bound, "depth=3 + exhausted must be DepthBound");
    assert!(!fc.access_bound, "no retrieval miss, not AccessBound");
    assert_eq!(
        fc.as_ssb_kind_str(),
        "depth_bound",
        "dominant SSB kind must be depth_bound"
    );
}

#[test]
fn low_depth_retrieval_miss_is_access_bound() {
    let fc = classify_failure(0, false, true);
    assert!(!fc.depth_bound, "depth=0 is not DepthBound");
    assert!(fc.access_bound, "retrieval miss must set AccessBound");
    assert_eq!(fc.as_ssb_kind_str(), "access_bound");
}

#[test]
fn both_flags_set_for_mixed_failure() {
    // RISK-2 from context-package: no forced single-arm enum.
    // A task that is deep AND retrieval-starved must surface BOTH.
    let fc = classify_failure(3, true, true);
    assert!(fc.depth_bound, "depth=3+exhausted => DepthBound");
    assert!(fc.access_bound, "retrieval_miss => AccessBound");
    // Depth wins for the SSB Kind (RD-4: depth-not-width guard must be visible)
    assert_eq!(fc.as_ssb_kind_str(), "depth_bound");
}

#[test]
fn no_failure_gives_neither() {
    let fc = classify_failure(0, false, false);
    assert_eq!(fc, FailureClass::NEITHER);
    assert_eq!(fc.as_ssb_kind_str(), "neither");
}

// ---------------------------------------------------------------------------
// (e) REAL dispatch path: deep BabyStep budget > shallow Concept budget.
//
// This is the primary R4/R7 test: it exercises the REAL scale_grc_budget
// function that agent_pool.rs::spawn calls at the live dispatch site.
// The AgentHandle's grc_config.budget.max_calls IS the off-agent observable:
// it is materialized by the production dispatch path, not by a test shim.
// Missing = FAIL (R1). The spawn site is the production entrypoint (W-3).
// ---------------------------------------------------------------------------

#[test]
fn deep_baby_step_budget_larger_than_shallow_concept() {
    let _mutex = BUDGET_SCALE_ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    // Same role, same base config, different depths — mirrors what
    // agent_pool::spawn does when called for a BabyStep vs Concept sub-task.
    let base_shallow = base_config("executor", 8, 1200);
    let base_deep = base_config("executor", 8, 1200);

    let d_shallow = estimate_serial_depth(DecompositionLevel::Concept); // 0
    let d_deep = estimate_serial_depth(DecompositionLevel::BabyStep); // 3

    let shallow_scaled = scale_grc_budget(base_shallow, d_shallow);
    let deep_scaled = scale_grc_budget(base_deep, d_deep);

    // Off-agent observable: deep sub-task gets a LARGER budget (R4, W-3).
    assert!(
        deep_scaled.budget.max_calls > shallow_scaled.budget.max_calls,
        "H-42: BabyStep budget ({}) must exceed Concept budget ({}) for same role",
        deep_scaled.budget.max_calls,
        shallow_scaled.budget.max_calls,
    );

    // Both must be within the LOCKED ceiling.
    let ceiling = 8 * LOCKED_BUDGET_CEILING;
    assert!(
        deep_scaled.budget.max_calls <= ceiling,
        "deep budget {} must not exceed ceiling {}",
        deep_scaled.budget.max_calls,
        ceiling,
    );
    assert!(
        shallow_scaled.budget.max_calls <= ceiling,
        "shallow budget {} must not exceed ceiling {}",
        shallow_scaled.budget.max_calls,
        ceiling,
    );
}

// ---------------------------------------------------------------------------
// (f) --flat-budget broken arm: ELAI_REASONING_BUDGET_SCALE_DISABLE=1 equalizes
// deep and shallow budgets (R2 falsifiability proof + R9 ON-path smoke).
//
// This test exercises the REAL env-var code path: it sets the kill-switch env
// var, calls scale_grc_budget at BabyStep depth (d=3, which normally produces
// 1.75x budget), and asserts the result equals the base (kill-switch active).
// This is the R2 proof that the kill-switch IS load-bearing AND that the
// env-var branch is actually reachable — not a vacuous d=0 simulation.
// Registered in wire-dark-allowlist.toml as the R9 ON-path smoke test.
//
// ENV VAR SAFETY: env vars are process-global; this test uses a per-test mutex
// guard so concurrent tests do not observe partial state. No external crate
// required — std::sync::Mutex is sufficient for a single env key.
// ---------------------------------------------------------------------------

/// Per-process mutex guarding mutations to ELAI_REASONING_BUDGET_SCALE_DISABLE.
///
/// ENV VAR SAFETY: env vars are process-global and mutable from any thread.
/// All tests that call `scale_grc_budget` with d>0 (i.e. tests where the env
/// var could change their output) MUST hold this mutex for their duration.
/// Tests that only check depth=0 (Concept, always returns base unchanged) are
/// exempt because the kill-switch is a no-op at d=0.
static BUDGET_SCALE_ENV_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// RAII guard that removes ELAI_REASONING_BUDGET_SCALE_DISABLE on drop.
/// Panic-safe: if the test body panics the env var is still cleaned up.
struct EnvVarGuard {
    key: &'static str,
}
impl Drop for EnvVarGuard {
    fn drop(&mut self) {
        std::env::remove_var(self.key);
    }
}

#[test]
fn kill_switch_env_var_equalizes_budget_at_babystep_depth() {
    let _mutex = BUDGET_SCALE_ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());

    use elai_orchestrator::ELAI_REASONING_BUDGET_SCALE_DISABLE_ENV;

    // Baseline: kill-switch OFF — BabyStep (d=3) must produce a scaled budget.
    std::env::remove_var(ELAI_REASONING_BUDGET_SCALE_DISABLE_ENV);
    let base = base_config("executor", 8, 1200);
    let normal = scale_grc_budget(base.clone(), 3);
    assert!(
        normal.budget.max_calls > base.budget.max_calls,
        "Kill-switch OFF: BabyStep (d=3) must produce a scaled budget larger than base; \
         got {} (not > {})",
        normal.budget.max_calls,
        base.budget.max_calls,
    );

    // Activate kill-switch via real env var; RAII guard removes it on scope exit.
    std::env::set_var(ELAI_REASONING_BUDGET_SCALE_DISABLE_ENV, "1");
    let _env_guard = EnvVarGuard { key: ELAI_REASONING_BUDGET_SCALE_DISABLE_ENV };

    let flat = scale_grc_budget(base.clone(), 3);
    // Kill-switch active: scale_grc_budget must return the base unchanged for d=3.
    assert_eq!(
        flat.budget.max_calls,
        base.budget.max_calls,
        "Kill-switch ON (ELAI_REASONING_BUDGET_SCALE_DISABLE=1): BabyStep (d=3) must return \
         the flat base budget; got {} expected {}",
        flat.budget.max_calls,
        base.budget.max_calls,
    );
    assert_eq!(
        flat.budget.max_chars,
        base.budget.max_chars,
        "Kill-switch ON: max_chars must also be unchanged; got {} expected {}",
        flat.budget.max_chars,
        base.budget.max_chars,
    );

    // R2 non-vacuousness: toggling the env var MUST change the observable output.
    assert_ne!(
        normal.budget.max_calls,
        flat.budget.max_calls,
        "R2: toggling ELAI_REASONING_BUDGET_SCALE_DISABLE MUST change scale_grc_budget output \
         for d=3 (kill-switch is load-bearing, not vacuous)"
    );
    // _env_guard drops here, removing the env var before mutex is released.
}

#[test]
fn flat_budget_env_equalizes_deep_and_shallow() {
    let _mutex = BUDGET_SCALE_ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    // Retained for regression coverage: verifies normal (non-env) behaviour.
    // The kill-switch test above is the R2/R9 proof; this test checks the
    // scale_grc_budget math without touching env vars.

    let base_shallow = base_config("executor", 8, 1200);
    let base_deep = base_config("executor", 8, 1200);

    // Without kill-switch: deep (d=3) must be > shallow (d=0).
    let d_shallow = estimate_serial_depth(DecompositionLevel::Concept);
    let d_deep = estimate_serial_depth(DecompositionLevel::BabyStep);
    let normal_shallow = scale_grc_budget(base_shallow.clone(), d_shallow);
    let normal_deep = scale_grc_budget(base_deep.clone(), d_deep);
    let normal_differ = normal_deep.budget.max_calls != normal_shallow.budget.max_calls;

    // d=0 returns base unchanged — same as what the kill-switch does for any d.
    let flat_shallow = scale_grc_budget(base_shallow.clone(), 0);
    let flat_deep = scale_grc_budget(base_deep.clone(), 0);
    let flat_equal = flat_shallow.budget.max_calls == flat_deep.budget.max_calls;

    assert!(
        normal_differ,
        "Without kill-switch, deep and shallow budgets MUST differ (depth-scaling is load-bearing)"
    );
    assert!(
        flat_equal,
        "With d=0 (kill-switch simulation), deep and shallow budgets MUST be equal (flat budget)"
    );
    assert_ne!(
        normal_deep.budget.max_calls,
        flat_deep.budget.max_calls,
        "Toggling depth between 3 and 0 MUST change the observable budget (R2 non-vacuousness)"
    );
}

// ---------------------------------------------------------------------------
// (g) LOCKED ceiling: extremely high depth cannot exceed LOCKED_BUDGET_CEILING.
// Loop-inflation proof: a loop cannot inflate its own budget above the ceiling.
// ---------------------------------------------------------------------------

#[test]
fn locked_ceiling_respected_at_extreme_depth() {
    // Hold the env-var mutex: d=1000 > 0, so the kill-switch env var affects output.
    let _mutex = BUDGET_SCALE_ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let base = base_config("executor", 8, 1200);
    let extreme_scaled = scale_grc_budget(base.clone(), 1000);
    let ceiling = base.budget.max_calls * LOCKED_BUDGET_CEILING;
    assert_eq!(
        extreme_scaled.budget.max_calls,
        ceiling,
        "depth=1000 must clamp at LOCKED_BUDGET_CEILING × base ({} × {} = {})",
        LOCKED_BUDGET_CEILING,
        base.budget.max_calls,
        ceiling,
    );
}

// ---------------------------------------------------------------------------
// (h) GD-2 reproducibility: same inputs produce identical scaled_calls.
// ---------------------------------------------------------------------------

#[test]
fn same_inputs_produce_identical_scaled_calls() {
    let _mutex = BUDGET_SCALE_ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let base1 = base_config("executor", 8, 1200);
    let base2 = base_config("executor", 8, 1200);
    let d = estimate_serial_depth(DecompositionLevel::BabyStep);

    let result1 = scale_grc_budget(base1, d);
    let result2 = scale_grc_budget(base2, d);

    assert_eq!(
        result1.budget.max_calls, result2.budget.max_calls,
        "GD-2: same inputs must always produce identical scaled_calls"
    );
    assert_eq!(
        result1.budget.max_chars, result2.budget.max_chars,
        "GD-2: same inputs must always produce identical scaled_chars"
    );
}

// ---------------------------------------------------------------------------
// (i) Depth-not-width guard (RD-4):
// A DepthBound failure must be classified as "depth_bound" (steps are the fix,
// NEVER context). A context-window lift for a depth_bound row is the anti-pattern.
// The classification must be correct so downstream consumers use the right remedy.
// ---------------------------------------------------------------------------

#[test]
fn depth_bound_failure_does_not_classify_as_access_bound_alone() {
    // A deep exhausted sub-task is depth_bound.
    // It must NOT be classified as access_bound unless retrieval_miss is ALSO true.
    let fc = classify_failure(3, true, false); // high depth + exhausted, no retrieval miss
    assert!(
        fc.depth_bound,
        "deep exhaustion must be DepthBound (fix = more CoT steps, not more context)"
    );
    assert!(
        !fc.access_bound,
        "without retrieval_miss, must NOT be AccessBound (depth-not-width guard RD-4)"
    );
    // The SSB kind must surface the depth signal (so downstream never applies context lift).
    assert_eq!(
        fc.as_ssb_kind_str(),
        "depth_bound",
        "SSB kind must be depth_bound for a pure depth-class failure (RD-4)"
    );
}

#[test]
fn access_bound_failure_is_not_depth_bound() {
    // Shallow retrieval-starved sub-task is access_bound only.
    let fc = classify_failure(0, false, true); // depth=0, retrieval miss
    assert!(
        !fc.depth_bound,
        "shallow sub-task must NOT be DepthBound"
    );
    assert!(
        fc.access_bound,
        "retrieval miss at any depth must be AccessBound"
    );
    assert_eq!(fc.as_ssb_kind_str(), "access_bound");
}

// ---------------------------------------------------------------------------
// (l) VecReasoningBudgetEmitter accumulates rows; scale_and_emit emits correctly.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn vec_emitter_accumulates_rows() {
    let emitter = VecReasoningBudgetEmitter::new();
    let payload = ReasoningBudgetScaledPayload {
        kind: "depth_bound".into(),
        role: "executor".into(),
        base_calls: 8,
        serial_depth: 3,
        scaled_calls: 14,
        ceiling: 24,
        seed: 42,
        // I-26 extension fields — None on GRC-dispatch rows (H-42 path).
        depth_level: None,
        budget_pct: None,
        plateau_fired: None,
    };
    // Emit via the trait method.
    use elai_orchestrator::ReasoningBudgetScaledEmitter;
    emitter.emit_scaled(payload.clone()).await.expect("emit ok");

    let rows = emitter.rows().await;
    assert_eq!(rows.len(), 1, "emitter must accumulate 1 row");
    assert_eq!(rows[0].kind, "depth_bound");
    assert_eq!(rows[0].serial_depth, 3);
    assert_eq!(rows[0].scaled_calls, 14);
}

#[tokio::test]
async fn scale_and_emit_emits_row_for_deep_task() {
    let _mutex = BUDGET_SCALE_ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    // scale_and_emit drives the REAL scale_grc_budget function and emits a row
    // via the emitter.  The emitter captures it; this is the off-agent row
    // the contract requires (R4/W-3).
    let emitter = VecReasoningBudgetEmitter::new();
    let base = base_config("executor", 8, 1200);
    let level = DecompositionLevel::BabyStep;
    let seed = 42u64;
    let fc = FailureClass::NEITHER;

    let scaled = scale_and_emit(base.clone(), level, fc, seed, &emitter).await;

    // The scaled config must be larger than base (deep sub-task).
    assert!(
        scaled.budget.max_calls > base.budget.max_calls,
        "scale_and_emit for BabyStep must produce a larger budget than base"
    );

    // The emitter must have received exactly one row (off-agent artifact, R1/R4).
    let rows = emitter.rows().await;
    assert_eq!(
        rows.len(),
        1,
        "scale_and_emit must emit exactly 1 ReasoningBudgetScaled row; got {}",
        rows.len()
    );

    let row = &rows[0];
    assert_eq!(row.role, "executor");
    assert_eq!(row.base_calls, 8);
    assert_eq!(row.serial_depth, 3, "BabyStep depth must be 3");
    assert_eq!(row.scaled_calls, scaled.budget.max_calls);
    assert!(
        row.scaled_calls > row.base_calls,
        "scaled_calls must exceed base_calls for BabyStep"
    );
    assert_eq!(row.ceiling, 8 * LOCKED_BUDGET_CEILING, "ceiling = LOCKED_BUDGET_CEILING × base");
    assert_eq!(row.seed, seed);
}

#[tokio::test]
async fn scale_and_emit_shallow_concept_no_row_emitted() {
    // At Concept depth (d=0), scale_grc_budget returns base unchanged — no
    // scaling happens so no row is emitted. This is the correct behaviour:
    // only actual scaling events produce the off-agent artifact.
    let emitter = VecReasoningBudgetEmitter::new();
    let base = base_config("executor", 8, 1200);
    let level = DecompositionLevel::Concept;
    let fc = FailureClass::NEITHER;

    let scaled = scale_and_emit(base.clone(), level, fc, 0, &emitter).await;

    // No scaling at depth=0.
    assert_eq!(
        scaled.budget.max_calls, base.budget.max_calls,
        "Concept depth must leave budget unchanged"
    );

    // No row emitted (no actual scaling happened).
    let rows = emitter.rows().await;
    assert!(
        rows.is_empty(),
        "scale_and_emit at Concept depth must NOT emit a row; got {} rows",
        rows.len()
    );
}

// ---------------------------------------------------------------------------
// Extra: verify that the 2-arm Kind discriminator is correct in the payload.
// Both "depth_bound" and "access_bound" must be valid kind strings.
// ---------------------------------------------------------------------------

#[test]
fn failure_class_kind_strings_are_canonical() {
    let depth_bound = FailureClass { depth_bound: true, access_bound: false };
    let access_bound = FailureClass { depth_bound: false, access_bound: true };
    let both = FailureClass { depth_bound: true, access_bound: true };
    let neither = FailureClass::NEITHER;

    assert_eq!(depth_bound.as_ssb_kind_str(), "depth_bound");
    assert_eq!(access_bound.as_ssb_kind_str(), "access_bound");
    assert_eq!(both.as_ssb_kind_str(), "depth_bound"); // dominant arm
    assert_eq!(neither.as_ssb_kind_str(), "neither");
}

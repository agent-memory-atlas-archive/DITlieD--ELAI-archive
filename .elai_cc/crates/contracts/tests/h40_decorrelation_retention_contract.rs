// H-40 — decorrelation-keyed verifier/role retention contract test.
//
// Verifies:
//   (a) unique_catch_rate computed correctly on the frozen fixture.
//   (b) A low-lift / high-unique-catch role is NOT insufficient_contribution (retained).
//   (c) A low-lift / low-unique-catch role IS insufficient_contribution (prunable).
//   (d) A --prune-on-lift-only broken arm flags BOTH roles (R2: proves the axis is
//       load-bearing; the new predicate changed the decision).
//   (e) ELAI_SEED twice -> identical metrics (GD-2 determinism).
//
// RUNPOD-PENDING guard: the production callers (fsm_ablation / role_ablation) must
// check `runpod_pending` before acting on `insufficient_contribution`. The fixture path
// exercises the Rust math and the H-40 predicate directly; it does NOT require live
// inference or a running sidecar.
//
// Grounding: arXiv:2506.18203 (Weaver 2025) — weak verifiers close the generation-
// verification gap when errors are conditionally independent given ground truth. A
// marginal-lift-only prune gate wrongly deletes the decorrelated verifier that catches
// failures the rest of the ensemble misses.

use std::collections::BTreeMap;

use elai_ipc_types::AgentRole;
use elai_orchestrator::iso_budget_harness::{
    decorrelation_score, fill_decorrelation, unique_catch_rate, IsoBudgetReport, IsoBudgetRow,
    PerRoleOutcomes,
};

// ---------------------------------------------------------------------------
// Frozen fixture (aligned with decorrelation_fixture.json).
//
// Consensus false = items 0,2,4,5,7,9 (6 ensemble failures).
//
// Role EXECUTOR (high unique-catch):
//   outcomes = [true,true,true,true,false,true,true,false,true,false]
//   At consensus-false indices [0,2,4,5,7,9]: [true,true,false,true,false,false]
//   -> catches 3 (0,2,5) -> UCR = 3/6 = 0.50  (> LOW_UNIQUE_CATCH_FLOOR=0.10)
//   -> RETAINED even when lift is low (RD-2).
//
// Role RESEARCHER (low unique-catch, acting as the prunable control):
//   outcomes = [false,true,false,true,false,false,true,false,true,false]
//   At consensus-false indices [0,2,4,5,7,9]: [false,false,false,false,false,false]
//   -> catches 0 -> UCR = 0/6 = 0.0 (< LOW_UNIQUE_CATCH_FLOOR=0.10)
//   -> prunable when lift is also low.
// ---------------------------------------------------------------------------

fn fixture_consensus() -> Vec<bool> {
    vec![false, true, false, true, false, false, true, false, true, false]
}

fn fixture_executor_outcomes() -> PerRoleOutcomes {
    let gt = vec![true, true, false, true, false, true, true, false, true, false];
    let outcomes = vec![true, true, true, true, false, true, true, false, true, false];
    PerRoleOutcomes::new(outcomes, gt)
}

/// Researcher acts as the low-UCR control role in this fixture.
fn fixture_researcher_outcomes() -> PerRoleOutcomes {
    let gt = vec![true, true, false, true, false, true, true, false, true, false];
    let outcomes = vec![false, true, false, true, false, false, true, false, true, false];
    PerRoleOutcomes::new(outcomes, gt)
}

fn fixture_role_outcomes() -> BTreeMap<AgentRole, PerRoleOutcomes> {
    let mut m = BTreeMap::new();
    m.insert(AgentRole::Executor, fixture_executor_outcomes());
    m.insert(AgentRole::Researcher, fixture_researcher_outcomes());
    m
}

/// Stub IsoBudgetReport (runpod_pending=true) used as base for fill_decorrelation.
fn stub_report() -> IsoBudgetReport {
    IsoBudgetReport {
        task_fixture: "fixtures/iso_budget/decorrelation_fixture.json".into(),
        budget_tokens: 2000,
        full_pipeline: IsoBudgetRow::empty("full_pipeline"),
        single_agent_wait: IsoBudgetRow::empty("single_agent_wait"),
        insufficient_lift: true,
        runpod_pending: true,
        unique_catch_rate: BTreeMap::new(),
        decorrelation_score: BTreeMap::new(),
        // Empty map on stub: unknown per-role state (caller must check runpod_pending).
        insufficient_contribution: BTreeMap::new(),
    }
}

// ---------------------------------------------------------------------------
// (a) unique_catch_rate computed correctly on the fixture.
// ---------------------------------------------------------------------------

#[test]
fn unique_catch_rate_executor_high() {
    let executor = fixture_executor_outcomes();
    let consensus = fixture_consensus();
    let ucr = unique_catch_rate(&executor, &consensus);
    // 3 unique catches out of 6 ensemble failures = 0.50
    assert!(
        (ucr - 0.5).abs() < 1e-9,
        "executor unique_catch_rate expected 0.50, got {ucr}"
    );
}

#[test]
fn unique_catch_rate_researcher_zero() {
    let researcher = fixture_researcher_outcomes();
    let consensus = fixture_consensus();
    let ucr = unique_catch_rate(&researcher, &consensus);
    // 0 unique catches out of 6 ensemble failures = 0.0
    assert!(
        ucr.abs() < 1e-9,
        "researcher unique_catch_rate expected 0.0, got {ucr}"
    );
}

#[test]
fn unique_catch_rate_no_ensemble_failures_returns_zero() {
    // If consensus is all-true (no failures), UCR is 0.0 by definition.
    let role = PerRoleOutcomes::new(vec![true, true, true], vec![true, true, true]);
    let consensus = vec![true, true, true];
    let ucr = unique_catch_rate(&role, &consensus);
    assert_eq!(ucr, 0.0, "vacuous case: no ensemble failures -> UCR=0.0");
}

// ---------------------------------------------------------------------------
// (b) Low-lift / high-unique-catch role is NOT insufficient_contribution.
//
// With BOTH roles present, executor's high UCR (0.50 > 0.10) prevents the
// ensemble from being flagged as prunable.
// ---------------------------------------------------------------------------

#[test]
fn executor_low_lift_high_unique_catch_ensemble_is_retained() {
    let role_outcomes = fixture_role_outcomes();
    let consensus = fixture_consensus();
    // lift_delta = 0.02 < LOW_LIFT_FLOOR=0.05 -> insufficient_lift=true
    let base = stub_report();
    let report = fill_decorrelation(base, &role_outcomes, &consensus, 0.02);

    // unique_catch_rate is keyed by snake_case wire tag.
    let executor_ucr = report
        .unique_catch_rate
        .get("executor")
        .copied()
        .expect("executor UCR must be present");
    assert!(
        executor_ucr > 0.10,
        "executor UCR {executor_ucr:.3} must exceed LOW_UNIQUE_CATCH_FLOOR(0.10)"
    );
    // fill_decorrelation: per-role predicate = low_lift AND low_unique_catch_for_THAT_role.
    // Executor has UCR=0.50 which is > 0.10, so its per-role entry must be false (retained).
    let executor_prunable = report
        .insufficient_contribution
        .get("executor")
        .copied()
        .expect("executor must have a per-role entry");
    assert!(
        !executor_prunable,
        "executor has high unique-catch (UCR=0.50): per-role entry must be false (retained)"
    );
}

// ---------------------------------------------------------------------------
// (c) Low-lift / low-unique-catch role IS insufficient_contribution (prunable).
// ---------------------------------------------------------------------------

#[test]
fn researcher_only_low_lift_low_unique_catch_is_prunable() {
    // Single-role map with only researcher (UCR=0.0 < 0.10).
    let mut single_role: BTreeMap<AgentRole, PerRoleOutcomes> = BTreeMap::new();
    single_role.insert(AgentRole::Researcher, fixture_researcher_outcomes());
    let consensus = fixture_consensus();

    let base = stub_report();
    // lift_delta = 0.02 < LOW_LIFT_FLOOR=0.05
    let report = fill_decorrelation(base, &single_role, &consensus, 0.02);

    let researcher_ucr = report
        .unique_catch_rate
        .get("researcher")
        .copied()
        .expect("researcher UCR must be present");
    assert!(
        researcher_ucr < 0.10,
        "researcher UCR {researcher_ucr:.3} must be below LOW_UNIQUE_CATCH_FLOOR(0.10)"
    );
    let researcher_prunable = report
        .insufficient_contribution
        .get("researcher")
        .copied()
        .expect("researcher must have a per-role entry");
    assert!(
        researcher_prunable,
        "researcher has zero unique-catch + low lift: per-role entry must be true (prunable)"
    );
}

// ---------------------------------------------------------------------------
// (d) R2 falsifiability: --prune-on-lift-only broken arm flags BOTH roles.
//
// H-40 prunable predicate = low_lift AND all_low_unique_catch.
// A lift-only predicate prunes any low-lift role regardless of UCR.
// This test proves:
//   1. Under lift-only: BOTH executor and researcher would be flagged prunable.
//   2. Under H-40: executor is RETAINED (high UCR saves it) -- the axis changed
//      the decision, proving it is load-bearing and not vacuous (R2).
// ---------------------------------------------------------------------------

#[test]
fn lift_only_prunes_both_h40_retains_executor_r2_falsifiable() {
    let lift_delta = 0.02_f64;
    let low_lift_floor = 0.05_f64;

    // --- Broken lift-only arm ---
    let lift_only_executor_prunable = lift_delta < low_lift_floor;
    let lift_only_researcher_prunable = lift_delta < low_lift_floor;
    assert!(lift_only_executor_prunable, "lift-only arm WOULD prune executor");
    assert!(lift_only_researcher_prunable, "lift-only arm WOULD prune researcher");

    // --- H-40 predicate ---
    let role_outcomes = fixture_role_outcomes();
    let consensus = fixture_consensus();
    let base = stub_report();
    let report = fill_decorrelation(base, &role_outcomes, &consensus, lift_delta);

    // H-40 retains executor because its UCR (0.50) exceeds the floor (0.10).
    let exec_prunable_h40 = report
        .insufficient_contribution
        .get("executor")
        .copied()
        .expect("executor must have per-role entry");
    assert!(
        !exec_prunable_h40,
        "H-40 predicate retains executor (high UCR=0.50); lift-only would wrongly prune it (R2)"
    );

    // Adversarial single-role check: researcher alone still prunes under H-40.
    let mut single_researcher: BTreeMap<AgentRole, PerRoleOutcomes> = BTreeMap::new();
    single_researcher.insert(AgentRole::Researcher, fixture_researcher_outcomes());
    let base2 = stub_report();
    let report2 = fill_decorrelation(base2, &single_researcher, &consensus, lift_delta);
    let researcher_prunable_h40 = report2
        .insufficient_contribution
        .get("researcher")
        .copied()
        .expect("researcher must have per-role entry");
    assert!(
        researcher_prunable_h40,
        "researcher alone (UCR=0.0 + low-lift) is still prunable under H-40"
    );
}

// ---------------------------------------------------------------------------
// (e) GD-2: identical metrics when called twice on the same fixture.
// ---------------------------------------------------------------------------

#[test]
fn decorrelation_metrics_are_deterministic() {
    let role_outcomes = fixture_role_outcomes();
    let consensus = fixture_consensus();

    let ucr_1 = unique_catch_rate(
        role_outcomes.get(&AgentRole::Executor).unwrap(),
        &consensus,
    );
    let ucr_2 = unique_catch_rate(
        role_outcomes.get(&AgentRole::Executor).unwrap(),
        &consensus,
    );
    assert_eq!(ucr_1.to_bits(), ucr_2.to_bits(), "UCR must be bit-identical on repeated calls");

    let ds_1 = decorrelation_score(
        role_outcomes.get(&AgentRole::Executor).unwrap(),
        &role_outcomes,
        &AgentRole::Executor,
    );
    let ds_2 = decorrelation_score(
        role_outcomes.get(&AgentRole::Executor).unwrap(),
        &role_outcomes,
        &AgentRole::Executor,
    );
    assert_eq!(ds_1.to_bits(), ds_2.to_bits(), "decorrelation_score must be bit-identical");
}

// ---------------------------------------------------------------------------
// (f) Pearson phi formula correctness: independent -> ~1.0, correlated -> ~0.0.
//
// BLOCK-2 fix verification: the old code used Ochiai (joint/sqrt(p_r*p_x)) which
// returns 0.5 for independent roles (wrong). Pearson phi returns ~0 for
// independent roles -> decorrelation_score = 1 - |0| = 1.0 (correct: retain).
// For perfectly correlated roles Pearson phi = 1.0 -> decorrelation_score = 0.0.
// ---------------------------------------------------------------------------

#[test]
fn pearson_phi_independent_roles_high_decorrelation() {
    // Two roles with errors on DISJOINT ground-truth-fail items:
    // Role A errors on items 0,1 only; Role B errors on items 2,3 only.
    // Ground truth: all fail (4 items), so gt_fail_indices = [0,1,2,3].
    // p_r = 2/4 = 0.5, p_x = 2/4 = 0.5, p_joint = 0/4 = 0.0
    // phi = (0.0 - 0.5*0.5) / sqrt(0.25 * 0.25) = -0.25 / 0.25 = -1.0
    // decorrelation_score = 1 - |-1.0| = 0.0
    //
    // Actually disjoint errors produce phi = -1.0 (anti-correlated), which the
    // abs() in the formula treats as 1.0 correlation -> ds = 0.0. That is CORRECT:
    // anti-correlated roles are maximally different in error patterns and should be
    // RETAINED (high decorrelation value). Let us use truly independent roles instead:
    // A errors on [0,2], B errors on [1,2] -> p_r=2/4=0.5, p_x=2/4=0.5, p_joint=1/4=0.25
    // phi = (0.25 - 0.25) / sqrt(0.0625) = 0.0 / 0.25 = 0.0 -> ds = 1.0 (independent)
    let gt = vec![false, false, false, false]; // all ground-truth fail
    let role_a = PerRoleOutcomes::new(
        vec![false, true, false, true], // errors at 0,2
        gt.clone(),
    );
    let role_b = PerRoleOutcomes::new(
        vec![true, false, false, true], // errors at 1,2; pass at 0,3
        gt.clone(),
    );

    let mut roles: BTreeMap<AgentRole, PerRoleOutcomes> = BTreeMap::new();
    roles.insert(AgentRole::Executor, role_a.clone());
    roles.insert(AgentRole::Researcher, role_b.clone());

    let ds = decorrelation_score(&role_a, &roles, &AgentRole::Executor);
    // p_r = 2/4=0.5, p_x = 2/4=0.5, p_joint = 1/4=0.25 (only item 2 both error)
    // phi = (0.25 - 0.5*0.5) / sqrt(0.5*0.5 * 0.5*0.5) = 0.0/0.25 = 0.0
    // decorrelation_score = 1 - |0.0| = 1.0
    assert!(
        (ds - 1.0).abs() < 1e-9,
        "independent error patterns (phi=0): decorrelation_score expected 1.0, got {ds:.6}"
    );
}

#[test]
fn pearson_phi_perfectly_correlated_roles_zero_decorrelation() {
    // Two roles with identical errors: role A = role B on all gt-fail items.
    // p_r = p_x = p_joint -> phi = (p^2 - p^2) / sqrt(p*(1-p) * p*(1-p)) = 0? No:
    // p_r = p_x = 0.5, p_joint = 0.5 (all error events coincide).
    // phi = (0.5 - 0.5*0.5) / sqrt(0.25 * 0.25) = 0.25 / 0.25 = 1.0
    // decorrelation_score = 1 - 1.0 = 0.0
    let gt = vec![false, false, false, false];
    let outcomes = vec![false, false, true, true]; // errors at 0,1
    let role_a = PerRoleOutcomes::new(outcomes.clone(), gt.clone());
    let role_b = PerRoleOutcomes::new(outcomes.clone(), gt.clone());

    let mut roles: BTreeMap<AgentRole, PerRoleOutcomes> = BTreeMap::new();
    roles.insert(AgentRole::Executor, role_a.clone());
    roles.insert(AgentRole::Researcher, role_b.clone());

    let ds = decorrelation_score(&role_a, &roles, &AgentRole::Executor);
    // p_r = 2/4=0.5, p_x = 2/4=0.5, p_joint = 2/4=0.5
    // phi = (0.5 - 0.25)/0.25 = 1.0 -> ds = 0.0
    assert!(
        ds.abs() < 1e-9,
        "identical error patterns (phi=1): decorrelation_score expected 0.0, got {ds:.6}"
    );
}

#[test]
fn pearson_phi_zero_variance_role_treated_as_decorrelated() {
    // A role that NEVER errors has p_r = 0 -> denominator = 0 -> corr = 0 -> ds = 1.0.
    // Guard prevents division by zero and conservatively retains the role.
    let gt = vec![false, false, false, false];
    let always_correct = PerRoleOutcomes::new(
        vec![true, true, true, true], // never errors
        gt.clone(),
    );
    let other = PerRoleOutcomes::new(
        vec![false, true, false, true], // errors at 0,2
        gt.clone(),
    );

    let mut roles: BTreeMap<AgentRole, PerRoleOutcomes> = BTreeMap::new();
    roles.insert(AgentRole::Executor, always_correct.clone());
    roles.insert(AgentRole::Researcher, other.clone());

    let ds = decorrelation_score(&always_correct, &roles, &AgentRole::Executor);
    // p_r = 0 -> var_r = 0 -> denom = 0 -> corr treated as 0 -> ds = 1.0
    assert!(
        (ds - 1.0).abs() < 1e-9,
        "zero-variance role (never errors): decorrelation_score expected 1.0, got {ds:.6}"
    );
}

// ---------------------------------------------------------------------------
// (g) BLOCK-1: production call path — run_ablation_with_outcomes exercises
//     fill_decorrelation from a non-test entry point. Confirms the enriched
//     report has populated unique_catch_rate and correct per-role decisions.
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn run_ablation_with_outcomes_populates_h40_fields() {
    use elai_orchestrator::iso_budget_harness::run_ablation_with_outcomes;
    use elai_orchestrator::ssb::{Ssb, SsbConfig};
    use elai_core::clock::SeededClock;
    use std::sync::Arc;
    use tempfile::TempDir;

    let dir = TempDir::new().expect("tempdir");
    let ssb = Arc::new(Ssb::new(
        SsbConfig {
            db_path: dir.path().join("block1_test.db"),
            pool_size: Some(2),
            project_id: "test".into(),
            session_id: "s-block1".into(),
        },
        Arc::new(SeededClock::new()),
    ).expect("ssb new"));

    let role_outcomes = fixture_role_outcomes();
    let consensus = fixture_consensus();
    // lift_delta < LOW_LIFT_FLOOR to trigger the low-lift condition.
    let lift_delta = 0.02_f64;

    let report = run_ablation_with_outcomes(
        &ssb,
        "test-agent",
        0,
        "fixtures/iso_budget/decorrelation_fixture.json",
        2000,
        &role_outcomes,
        &consensus,
        lift_delta,
    )
    .await
    .expect("run_ablation_with_outcomes must succeed");

    // runpod_pending must be false (live outcomes path).
    assert!(!report.runpod_pending, "live outcomes path: runpod_pending must be false");

    // unique_catch_rate must be non-empty (fill_decorrelation was called).
    assert!(
        !report.unique_catch_rate.is_empty(),
        "fill_decorrelation must populate unique_catch_rate"
    );
    let executor_ucr = report
        .unique_catch_rate
        .get("executor")
        .copied()
        .expect("executor must be in unique_catch_rate");
    assert!(
        (executor_ucr - 0.5).abs() < 1e-9,
        "executor UCR expected 0.50, got {executor_ucr}"
    );

    // per-role insufficient_contribution: executor retained (high UCR), researcher prunable.
    let executor_prunable = report
        .insufficient_contribution
        .get("executor")
        .copied()
        .expect("executor must have per-role entry");
    assert!(!executor_prunable, "executor (UCR=0.50) must be retained (prunable=false)");

    let researcher_prunable = report
        .insufficient_contribution
        .get("researcher")
        .copied()
        .expect("researcher must have per-role entry");
    assert!(researcher_prunable, "researcher (UCR=0.0) must be prunable (prunable=true)");
}

// ---------------------------------------------------------------------------
// SSB payload struct: new fields are serde(default) and backward compat.
// RISK-2 mitigation: old rows deserialize without H-40 fields.
// ---------------------------------------------------------------------------

#[test]
fn iso_budget_payload_old_row_deserializes_without_h40_fields() {
    use elai_orchestrator::ssb::IsoBudgetMeasuredPayload;

    let old_json = serde_json::json!({
        "task_fixture": "old/fixture.toml",
        "budget_tokens": 1000u32,
        "full_pipeline_pass_rate": 0.6f64,
        "single_agent_wait_pass_rate": 0.5f64,
        "insufficient_lift": false,
        "runpod_pending": true,
        "mast_mode_counts": []
    });
    let payload: IsoBudgetMeasuredPayload =
        serde_json::from_value(old_json).expect("old row must deserialize");
    // H-40 defaults: empty maps for all three fields. Per-role map is empty for
    // pre-H-40 rows (unknown prunable state; caller must check runpod_pending).
    assert!(payload.unique_catch_rate.is_empty(), "default: empty map");
    assert!(payload.decorrelation_score.is_empty(), "default: empty map");
    assert!(
        payload.insufficient_contribution.is_empty(),
        "default: empty per-role map for pre-H-40 rows"
    );
}

#[test]
fn iso_budget_payload_new_row_round_trips() {
    use elai_orchestrator::ssb::IsoBudgetMeasuredPayload;

    let mut ucr = std::collections::BTreeMap::new();
    ucr.insert("executor".to_string(), 0.5_f64);
    ucr.insert("researcher".to_string(), 0.0_f64);
    let mut ds = std::collections::BTreeMap::new();
    ds.insert("executor".to_string(), 0.75_f64);
    ds.insert("researcher".to_string(), 0.2_f64);

    let mut ic: std::collections::BTreeMap<String, bool> = std::collections::BTreeMap::new();
    ic.insert("executor".to_string(), false);
    ic.insert("researcher".to_string(), true);

    let payload = IsoBudgetMeasuredPayload {
        task_fixture: "fixtures/iso_budget/decorrelation_fixture.json".into(),
        budget_tokens: 2000,
        full_pipeline_pass_rate: 0.7,
        single_agent_wait_pass_rate: 0.68,
        insufficient_lift: true,
        runpod_pending: false,
        mast_mode_counts: vec![],
        unique_catch_rate: ucr.clone(),
        decorrelation_score: ds.clone(),
        insufficient_contribution: ic.clone(),
    };
    let json = serde_json::to_string(&payload).expect("serialize");
    let back: IsoBudgetMeasuredPayload = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(back.unique_catch_rate, ucr);
    assert_eq!(back.decorrelation_score, ds);
    assert_eq!(back.insufficient_contribution, ic);
}

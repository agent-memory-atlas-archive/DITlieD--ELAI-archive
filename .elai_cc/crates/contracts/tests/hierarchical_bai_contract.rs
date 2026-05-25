// D-29 contract: Hierarchical Bayesian BAI host-side surface.
//
// Verifies (atomic SCHEMA bump 145 -> 146, +1 SsbActionType variant):
// 1. SCHEMA_VERSION_PUBLIC >= 146 (D-29 floor; HierarchicalBaiRoundCompleted
//    variant landed at this bump).
// 2. SsbActionType::HierarchicalBaiRoundCompleted + payload round-trip via
//    as_sql_str / ssb_action_type_from_str_public + JSON + msgpack
//    (binds the wire-format string + payload shape; single-variant by
//    D-19/D-23/D-24/D-25/D-28 precedent — pattern-2).
// 3. ELAI_BAI_MODE env gate default Flat (pattern-4: wire-dark default).
//    BaiMode::from_env() with the var unset returns Flat. With value
//    "hierarchical" returns Hierarchical. With unknown value returns Flat.
// 4. Hierarchical state invariants:
//    - mu_mean strictly within (0, 1).
//    - kappa_mean strictly positive.
//    - Variance fields >= 0.
//    - Total variance decomposition holds: Var[total] = E[Var | family] +
//      Var[E | family] (arXiv:2111.06929 §4 load-bearing identity; the
//      total-variance identity itself is a probability textbook fact).
// 5. SelectionBasis 3-variant closed enum + stable snake_case wire tags
//    (hierarchical_ts / flat_ts / canary_override).
// 6. Deterministic under fixed seed (pattern: same seed -> byte-identical
//    output; mirrors D-5's sample_beta_deterministic_under_fixed_seed at
//    bai.rs).
// 7. Empty-state fallback: hierarchical_arm_select on a state with no
//    family assignments returns SelectionBasis::FlatTs (graceful
//    degradation when the hierarchical layer is wire-dark or has no
//    pooling signal).
//
// Paper labels (verified via workflow/papers/):
//   - arXiv:2111.06929 SUPPORTING EVIDENCE (HierTS; AISTATS 2022, HIGH
//     reliability; regret minimization, NOT BAI — see
//     workflow/papers/hierarchical-bayesian-bandits-interface.md L144-148).
//   - arXiv:2106.10417 DIRECT GROUNDING (variance-dependent BAI; UAI 2021,
//     HIGH reliability — see workflow/papers/variance-bai-2106.10417-interface.md).
// The plan's "30-50% regret reduction" headline is UNGROUNDED for BAI and
// is logged as RUNPOD-PENDING in workflow/reasoning/D-29-decisions.md.

use std::sync::Mutex;

use elai_core::SeededRng;
use elai_orchestrator::bai::ArmPosterior;
use elai_orchestrator::bai_hierarchical::{
    hierarchical_arm_select, hierarchical_update, total_variance_decomposition, FamilyId,
    FamilyHyperprior, HierarchicalBaiState, SelectionBasis, GIBBS_BURN_IN, KAPPA_PRIOR_RATE,
    KAPPA_PRIOR_SHAPE, MAX_GIBBS_ITERATIONS, MAX_KAPPA, MIN_KAPPA, MU_FLOOR,
};
use elai_orchestrator::reasoning_template::TemplateId;
use elai_orchestrator::runtime_config::{BaiMode, ELAI_BAI_MODE_ENV};
use elai_orchestrator::ssb::{
    ssb_action_type_from_str_public, ArmPosteriorSummary,
    HierarchicalBaiRoundCompletedPayload, SsbActionType, SsbPayload, SCHEMA_VERSION_PUBLIC,
};
use uuid::Uuid;

/// Floor for D-29: head SCHEMA_VERSION at completion-drive close.
/// Atomic +1 bump from D-28's 145 head (single new variant
/// HierarchicalBaiRoundCompleted; see D-29-decisions.md DECISION-1).
const D29_SCHEMA_FLOOR: i64 = 146;

/// Mutex guarding the global ELAI_BAI_MODE env var across the env-gate
/// tests. set_var / remove_var on env::var are GLOBAL state in the process
/// — if two tests touch them in parallel one will read a stale value.
/// Tests that touch this env var MUST take this lock for their duration.
static ENV_LOCK: Mutex<()> = Mutex::new(());

fn arm_at(byte: u8, alpha: f64, beta: f64) -> ArmPosterior {
    let mut bytes = [0u8; 16];
    bytes[0] = byte;
    let template_id = TemplateId::from(Uuid::from_bytes(bytes));
    ArmPosterior {
        template_id,
        alpha,
        beta,
        n_pulls: (alpha + beta - 2.0).max(0.0) as u32,
    }
}

#[test]
fn schema_version_includes_d29_floor() {
    assert!(
        SCHEMA_VERSION_PUBLIC >= D29_SCHEMA_FLOOR,
        "schema must include D-29 HierarchicalBaiRoundCompleted variant \
         (got {SCHEMA_VERSION_PUBLIC}, floor {D29_SCHEMA_FLOOR})"
    );
}

#[test]
fn hierarchical_bai_round_completed_payload_roundtrip() {
    // Bind the SQL wire tag + the round-trip identity. The variant tag is
    // the canonical snake_case wire format the lockstep cascade is pinned
    // to (action_type_lock.golden alphabetical insertion between
    // held_out_task_used and hook_event).
    let tag = SsbActionType::HierarchicalBaiRoundCompleted.as_sql_str();
    assert_eq!(tag, "hierarchical_bai_round_completed");
    let decoded = ssb_action_type_from_str_public(tag)
        .expect("ssb_action_type_from_str_public hierarchical_bai_round_completed");
    assert_eq!(decoded, SsbActionType::HierarchicalBaiRoundCompleted);

    let arm_summary = ArmPosteriorSummary {
        template_id: "0".repeat(32),
        family_id: 0,
        alpha: 5.0,
        beta: 5.0,
        mean: 0.5,
        variance: 0.022_727_272_727_272_727,
        ci_low: 0.231,
        ci_high: 0.769,
        pooled_mean: 0.5,
    };
    let p = HierarchicalBaiRoundCompletedPayload {
        round: 80,
        family_id: 0,
        mu_posterior_mean: 0.5,
        mu_posterior_variance: 0.02,
        kappa_posterior_mean: 4.0,
        kappa_posterior_variance: 1.5,
        arm_posteriors: vec![arm_summary.clone()],
        selected_arm_index: 0,
        selected_template_id: "0".repeat(32),
        selection_basis: "hierarchical_ts".to_string(),
        gibbs_iterations: 80,
        canary_gate_passed: true,
    };
    let payload = SsbPayload::HierarchicalBaiRoundCompleted(p.clone());
    assert_eq!(
        payload.action_type(),
        SsbActionType::HierarchicalBaiRoundCompleted
    );

    // JSON round-trip preserves payload byte semantics.
    let json = serde_json::to_string(&p).expect("payload json serialize");
    let decoded_p: HierarchicalBaiRoundCompletedPayload =
        serde_json::from_str(&json).expect("payload json deserialize");
    assert_eq!(decoded_p, p);

    // msgpack round-trip via SsbPayload outer envelope (mirrors
    // adv_rm_farming_contract msgpack pattern).
    let bytes = rmp_serde::to_vec_named(&payload).expect("msgpack serialize");
    let decoded_env: SsbPayload =
        rmp_serde::from_slice(&bytes).expect("msgpack deserialize");
    assert_eq!(
        decoded_env.action_type(),
        SsbActionType::HierarchicalBaiRoundCompleted
    );
    if let SsbPayload::HierarchicalBaiRoundCompleted(d) = decoded_env {
        assert_eq!(d, p);
    } else {
        panic!("expected HierarchicalBaiRoundCompleted payload");
    }
}

#[test]
fn bai_mode_default_is_flat_when_env_unset() {
    let _guard = ENV_LOCK.lock().expect("env lock");
    // Snapshot any pre-existing value so we can restore.
    let prev = std::env::var(ELAI_BAI_MODE_ENV).ok();
    std::env::remove_var(ELAI_BAI_MODE_ENV);
    let mode = BaiMode::from_env();
    assert_eq!(mode, BaiMode::Flat);
    // Restore.
    if let Some(v) = prev {
        std::env::set_var(ELAI_BAI_MODE_ENV, v);
    }
}

#[test]
fn bai_mode_hierarchical_when_env_explicit() {
    let _guard = ENV_LOCK.lock().expect("env lock");
    let prev = std::env::var(ELAI_BAI_MODE_ENV).ok();
    std::env::set_var(ELAI_BAI_MODE_ENV, "hierarchical");
    let mode = BaiMode::from_env();
    assert_eq!(mode, BaiMode::Hierarchical);
    // Unknown value falls back to Flat (wire-dark by default).
    std::env::set_var(ELAI_BAI_MODE_ENV, "wat");
    let mode_bad = BaiMode::from_env();
    assert_eq!(mode_bad, BaiMode::Flat);
    // Restore.
    match prev {
        Some(v) => std::env::set_var(ELAI_BAI_MODE_ENV, v),
        None => std::env::remove_var(ELAI_BAI_MODE_ENV),
    }
}

#[test]
fn bai_mode_wire_tags_stable() {
    // Pin the snake_case wire tags so a rename trips this contract.
    assert_eq!(BaiMode::Flat.as_str(), "flat");
    assert_eq!(BaiMode::Hierarchical.as_str(), "hierarchical");
}

#[test]
fn family_hyperprior_invariants_after_gibbs_update() {
    let mut state = HierarchicalBaiState::new();
    let arms = vec![
        arm_at(1, 5.0, 5.0),
        arm_at(2, 7.0, 3.0),
        arm_at(3, 8.0, 2.0),
    ];
    let fid = FamilyId(0);
    for arm in &arms {
        state.assign_arm(arm.template_id, fid);
    }
    let mut rng = SeededRng::new(2026);
    let refs: Vec<&ArmPosterior> = arms.iter().collect();
    let fp: FamilyHyperprior = hierarchical_update(&mut state, fid, &refs, &mut rng);

    // Invariants from workflow/papers/hierarchical-bayesian-bandits-interface.md L66-73:
    assert!(
        fp.mu_mean > 0.0 && fp.mu_mean < 1.0,
        "mu_mean must be in (0, 1) (got {})",
        fp.mu_mean
    );
    assert!(
        fp.kappa_mean > 0.0,
        "kappa_mean must be > 0 (got {})",
        fp.kappa_mean
    );
    assert!(
        fp.mu_variance >= 0.0,
        "mu_variance must be >= 0 (got {})",
        fp.mu_variance
    );
    assert!(
        fp.kappa_variance >= 0.0,
        "kappa_variance must be >= 0 (got {})",
        fp.kappa_variance
    );
    // Clamp invariants.
    assert!(fp.mu_mean >= MU_FLOOR && fp.mu_mean <= 1.0 - MU_FLOOR);
    assert!(fp.kappa_mean >= MIN_KAPPA && fp.kappa_mean <= MAX_KAPPA);
    // Post-burn-in iteration count matches the canonical configuration.
    assert_eq!(
        fp.n_gibbs_iterations,
        MAX_GIBBS_ITERATIONS - GIBBS_BURN_IN
    );
}

#[test]
fn total_variance_decomposition_identity() {
    // arXiv:2111.06929 §4 + textbook law of total variance:
    //   Var[theta] = E[Var[theta | family]] + Var[E[theta | family]]
    // Our diagnostic implementation computes the empirical-arm form;
    // verify the identity numerically (within float tolerance).
    let mut state = HierarchicalBaiState::new();
    let arms = vec![
        arm_at(1, 9.0, 1.0),
        arm_at(2, 8.0, 2.0),
        arm_at(3, 2.0, 8.0),
        arm_at(4, 1.0, 9.0),
    ];
    state.assign_arm(arms[0].template_id, FamilyId(0));
    state.assign_arm(arms[1].template_id, FamilyId(0));
    state.assign_arm(arms[2].template_id, FamilyId(1));
    state.assign_arm(arms[3].template_id, FamilyId(1));
    let (total, within, between) = total_variance_decomposition(&state, &arms);
    assert!(total >= 0.0);
    assert!(within >= 0.0);
    assert!(between >= 0.0);
    let sum = within + between;
    assert!(
        (total - sum).abs() < 1e-9,
        "Var[total] != E[Var|fam] + Var[E|fam] (total={total}, sum={sum})"
    );
}

#[test]
fn selection_basis_wire_tags_stable() {
    // 3-variant closed enum pinned at the contract boundary.
    assert_eq!(SelectionBasis::HierarchicalTs.as_str(), "hierarchical_ts");
    assert_eq!(SelectionBasis::FlatTs.as_str(), "flat_ts");
    assert_eq!(SelectionBasis::CanaryOverride.as_str(), "canary_override");

    // Exhaustive match — if a 4th variant lands without an ELAI-side
    // decision doc updating this contract, this fails to compile.
    fn _exhaustive(b: SelectionBasis) -> &'static str {
        match b {
            SelectionBasis::HierarchicalTs => "h",
            SelectionBasis::FlatTs => "f",
            SelectionBasis::CanaryOverride => "c",
        }
    }
}

#[test]
fn empty_state_falls_back_to_flat_ts() {
    // No family assignments => no pooling signal => FlatTs basis.
    let state = HierarchicalBaiState::new();
    let arms = vec![arm_at(1, 5.0, 5.0), arm_at(2, 7.0, 3.0)];
    let mut rng = SeededRng::new(7);
    let (idx, basis) = hierarchical_arm_select(&state, &arms, &mut rng).expect("select");
    assert!(idx < arms.len());
    assert_eq!(basis, SelectionBasis::FlatTs);
}

#[test]
fn deterministic_under_fixed_seed() {
    // D-5 precedent: sample_beta_deterministic_under_fixed_seed.
    let mut state = HierarchicalBaiState::new();
    let arms = vec![
        arm_at(1, 5.0, 5.0),
        arm_at(2, 8.0, 2.0),
        arm_at(3, 3.0, 7.0),
    ];
    state.assign_arm(arms[0].template_id, FamilyId(0));
    state.assign_arm(arms[1].template_id, FamilyId(0));
    state.assign_arm(arms[2].template_id, FamilyId(1));

    let mut rng_a = SeededRng::new(31337);
    let mut rng_b = SeededRng::new(31337);
    let a = hierarchical_arm_select(&state, &arms, &mut rng_a);
    let b = hierarchical_arm_select(&state, &arms, &mut rng_b);
    assert_eq!(a, b);

    // Same for hierarchical_update — same seed + same arms -> same family
    // hyperposterior (within Gibbs sample-mean tolerance which is exact
    // for ChaCha8-seeded byte-identical draws).
    let mut state_a = HierarchicalBaiState::new();
    let mut state_b = HierarchicalBaiState::new();
    for arm in &arms {
        state_a.assign_arm(arm.template_id, FamilyId(0));
        state_b.assign_arm(arm.template_id, FamilyId(0));
    }
    let mut rng_c = SeededRng::new(31337);
    let mut rng_d = SeededRng::new(31337);
    let refs_a: Vec<&ArmPosterior> = arms.iter().collect();
    let refs_b: Vec<&ArmPosterior> = arms.iter().collect();
    let fp_a = hierarchical_update(&mut state_a, FamilyId(0), &refs_a, &mut rng_c);
    let fp_b = hierarchical_update(&mut state_b, FamilyId(0), &refs_b, &mut rng_d);
    assert_eq!(
        fp_a, fp_b,
        "hierarchical_update non-deterministic under fixed seed"
    );
}

#[test]
fn kappa_prior_constants_pinned_at_weakly_informative_defaults() {
    // Gelman BDA3 §5.3 recommendation: shape=2, rate=0.5 gives
    // E[kappa] = 4, Var[kappa] = 8. Drift would breakthrough the
    // weakly-informative claim.
    let eps = 1e-9_f64;
    assert!(
        (KAPPA_PRIOR_SHAPE - 2.0).abs() < eps,
        "KAPPA_PRIOR_SHAPE must be 2.0 (got {KAPPA_PRIOR_SHAPE})"
    );
    assert!(
        (KAPPA_PRIOR_RATE - 0.5).abs() < eps,
        "KAPPA_PRIOR_RATE must be 0.5 (got {KAPPA_PRIOR_RATE})"
    );
}

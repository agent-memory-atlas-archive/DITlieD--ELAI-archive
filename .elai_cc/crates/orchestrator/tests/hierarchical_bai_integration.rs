// D-29 integration: SyntheticNonIidBandit + hierarchical BAI layer wired
// into the D-5 ArmPosterior + canary composition surface.
//
// Verifies (one #[tokio::test]):
//   - The hierarchical layer accepts a 5-family x 10-arm fixture (the
//     plan's synthetic non-IID benchmark shape) with deterministic
//     seeded generation (provenance = ElaiSynthetic per pattern-10).
//   - hierarchical_update + hierarchical_arm_select compose with the D-5
//     ArmPosterior surface without mutating its public invariants.
//   - The post-update FamilyHyperprior carries mu in (0, 1) and kappa > 0
//     for every family.
//   - The selection emits the correct SelectionBasis (HierarchicalTs
//     when families are registered).
//
// NOT verified here (RUNPOD-PENDING):
//   - Live flat-vs-hierarchical regret comparison over 1000 rounds. Per
//     workflow/reasoning/D-29-decisions.md DECISION-7, the regret
//     measurement needs a runtime baseline that the host-side scaffolding
//     does not provide.

use elai_core::SeededRng;
use elai_orchestrator::bai::ArmPosterior;
use elai_orchestrator::bai_hierarchical::{
    hierarchical_arm_select, hierarchical_update, FamilyId, HierarchicalBaiState, SelectionBasis,
};
use elai_orchestrator::reasoning_template::TemplateId;
use uuid::Uuid;

const FAMILIES: usize = 5;
const ARMS_PER_FAMILY: usize = 10;

/// SyntheticNonIidBandit fixture per the D-29 plan §"Synthetic non-IID
/// benchmark". 5 families x 10 arms; the within-family arms share a
/// per-family Beta-hyperprior `(mu_f, kappa_f)` and the cross-family
/// hyperpriors are independent. Provenance = ElaiSynthetic per
/// pattern-10 (no real ELAI task data — these are deterministic
/// fixtures for the integration test ONLY).
fn build_fixture(seed: u64) -> (Vec<ArmPosterior>, Vec<FamilyId>) {
    let mut rng = SeededRng::new(seed);
    let mut arms = Vec::with_capacity(FAMILIES * ARMS_PER_FAMILY);
    let mut families = Vec::with_capacity(FAMILIES * ARMS_PER_FAMILY);

    // Family priors: mu_f varies across families so the partial pooling
    // has signal.
    let family_mus = [0.2, 0.4, 0.5, 0.6, 0.8];

    for f in 0..FAMILIES {
        let mu_f = family_mus[f];
        for i in 0..ARMS_PER_FAMILY {
            // Deterministic per-arm posterior: simulate ~ 20 pulls per arm
            // with passes biased by mu_f (so the hierarchical layer has
            // a non-trivial shrinkage target).
            let n_pulls = 20u32;
            // Use rng to break ties between arms within a family.
            let _jitter = rng_uniform(&mut rng);
            let passes = ((mu_f * (n_pulls as f64)).round() as u32).min(n_pulls);
            let fails = n_pulls - passes;
            let mut bytes = [0u8; 16];
            bytes[0] = f as u8;
            bytes[1] = i as u8;
            let template_id = TemplateId::from(Uuid::from_bytes(bytes));
            let alpha = 1.0 + passes as f64;
            let beta = 1.0 + fails as f64;
            arms.push(ArmPosterior {
                template_id,
                alpha,
                beta,
                n_pulls,
            });
            families.push(FamilyId(f as u32));
        }
    }
    (arms, families)
}

fn rng_uniform(rng: &mut SeededRng) -> f64 {
    use elai_core::EntropySource;
    let bits = rng.next_u64() >> 11;
    (bits as f64) * (1.0 / ((1u64 << 53) as f64))
}

#[test]
fn hierarchical_layer_composes_with_d5_arm_posterior() {
    let (arms, families) = build_fixture(20260526);
    assert_eq!(arms.len(), FAMILIES * ARMS_PER_FAMILY);
    assert_eq!(families.len(), arms.len());

    let mut state = HierarchicalBaiState::new();
    for (arm, fid) in arms.iter().zip(families.iter()) {
        state.assign_arm(arm.template_id, *fid);
    }

    // Run one hierarchical_update per family.
    let mut rng = SeededRng::new(7);
    for f in 0..FAMILIES {
        let fid = FamilyId(f as u32);
        let refs: Vec<&ArmPosterior> = arms
            .iter()
            .zip(families.iter())
            .filter(|(_, ff)| **ff == fid)
            .map(|(a, _)| a)
            .collect();
        assert_eq!(refs.len(), ARMS_PER_FAMILY);
        let fp = hierarchical_update(&mut state, fid, &refs, &mut rng);
        assert!(fp.mu_mean > 0.0 && fp.mu_mean < 1.0, "fam {f} mu_mean out of range: {}", fp.mu_mean);
        assert!(fp.kappa_mean > 0.0, "fam {f} kappa_mean <= 0: {}", fp.kappa_mean);
        assert!(fp.mu_variance >= 0.0);
        assert!(fp.kappa_variance >= 0.0);
    }

    // Selector pulls an arm via hierarchical TS — confirm basis is
    // HierarchicalTs (every arm in this fixture has a family assignment).
    let select = hierarchical_arm_select(&state, &arms, &mut rng).expect("select");
    assert!(select.0 < arms.len());
    assert_eq!(select.1, SelectionBasis::HierarchicalTs);

    // ArmPosterior public invariants preserved (D-5 surface unchanged).
    for arm in &arms {
        assert!(arm.alpha > 0.0);
        assert!(arm.beta > 0.0);
        // mean is in [0, 1].
        let m = arm.mean();
        assert!((0.0..=1.0).contains(&m));
    }
}

#[test]
fn fixture_is_deterministic_under_fixed_seed() {
    let (a1, f1) = build_fixture(42);
    let (a2, f2) = build_fixture(42);
    assert_eq!(a1.len(), a2.len());
    for (x, y) in a1.iter().zip(a2.iter()) {
        assert_eq!(x.template_id.as_hex(), y.template_id.as_hex());
        assert_eq!(x.alpha, y.alpha);
        assert_eq!(x.beta, y.beta);
        assert_eq!(x.n_pulls, y.n_pulls);
    }
    assert_eq!(f1, f2);
}

#[test]
fn family_hyperprior_carries_signal_across_arms() {
    // Convex check: when families have distinct mu_f priors (0.2 vs 0.8),
    // their post-update hyperposterior mu_mean should differ.
    let (arms, families) = build_fixture(20260526);
    let mut state = HierarchicalBaiState::new();
    for (arm, fid) in arms.iter().zip(families.iter()) {
        state.assign_arm(arm.template_id, *fid);
    }
    let mut rng = SeededRng::new(99);
    let family_lo = FamilyId(0); // mu_f = 0.2
    let family_hi = FamilyId(4); // mu_f = 0.8

    let refs_lo: Vec<&ArmPosterior> = arms
        .iter()
        .zip(families.iter())
        .filter(|(_, f)| **f == family_lo)
        .map(|(a, _)| a)
        .collect();
    let refs_hi: Vec<&ArmPosterior> = arms
        .iter()
        .zip(families.iter())
        .filter(|(_, f)| **f == family_hi)
        .map(|(a, _)| a)
        .collect();

    let fp_lo = hierarchical_update(&mut state, family_lo, &refs_lo, &mut rng);
    let fp_hi = hierarchical_update(&mut state, family_hi, &refs_hi, &mut rng);

    // Empirical arm means for the low family are ~0.2 and for the high
    // family ~0.8; the Gibbs posterior should put mu_lo < mu_hi.
    assert!(
        fp_lo.mu_mean < fp_hi.mu_mean,
        "Gibbs posterior failed to separate families: mu_lo={} mu_hi={}",
        fp_lo.mu_mean,
        fp_hi.mu_mean
    );
}

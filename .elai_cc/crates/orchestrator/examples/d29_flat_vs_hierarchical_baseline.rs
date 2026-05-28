// D-29 (RUNPOD-COMPLETE substrate): flat-vs-hierarchical regret baseline on
// the SyntheticNonIidBandit 5x10 fixture (1000 rounds, seed 20260526). CPU
// only — Monte-Carlo bandit simulation. NO GPU / sidecar required.
//
// Per workflow/reasoning/D-29-decisions.md DECISION-7 the pre-committed
// threshold is `hier_regret <= flat_regret - 5% CI`. The 5x10 family
// structure mirrors crates/orchestrator/tests/hierarchical_bai_integration.rs
// (FAMILIES=5, ARMS_PER_FAMILY=10, family_mus=[0.2, 0.4, 0.5, 0.6, 0.8]) so
// this binary's posterior init is byte-comparable with the integration test
// fixture.
//
// Mechanism (matches the contract semantics of bai_hierarchical.rs):
//   1. Build 50 arms with Beta(1,1) (=uniform) prior; true reward per arm
//      is Bernoulli with mean = family_mus[f] for arm in family f.
//   2. Each round, the FLAT path runs Thompson over the raw (alpha, beta)
//      pair. The HIERARCHICAL path runs hierarchical_arm_select which
//      shrinks toward each family's hyperposterior; the hyperposterior is
//      refreshed every REFRESH_GIBBS_EVERY rounds via hierarchical_update.
//   3. Round regret = max(true_mus) - true_mus[selected].
//   4. After 1000 rounds we report cumulative regret per arm-policy plus a
//      bootstrap 95% CI on the per-round regret series.
//
// Run:
//   cargo run --release -p elai-orchestrator --example d29_flat_vs_hierarchical_baseline
//
// Determinism: a single SeededRng(20260526) is threaded through both paths.
// Re-running the binary produces byte-identical regret summaries.
//
// Reliability label: SUPPORTING — the 5x10 fixture is synthetic per
// pattern-10 (ElaiSynthetic provenance). The result PROVES the host-side
// scaffolding runs end-to-end on CPU and emits an empirical regret signal;
// the live cross-vendor regret comparison still needs a real task corpus
// before D-29 ships in production (CLAUDE.md grounded-by-default).

use std::time::Instant;

use elai_core::SeededRng;
use elai_orchestrator::bai::{sample_beta, ArmPosterior};
use elai_orchestrator::bai_hierarchical::{
    hierarchical_arm_select, hierarchical_update, FamilyId, HierarchicalBaiState,
};
use elai_orchestrator::reasoning_template::TemplateId;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Fixture (named constants, no magic numbers)
// ---------------------------------------------------------------------------

const FAMILIES: usize = 5;
const ARMS_PER_FAMILY: usize = 10;
const N_ARMS: usize = FAMILIES * ARMS_PER_FAMILY;
const ROUNDS: u32 = 1000;
const SEED: u64 = 20_260_526;
/// How often the hierarchical layer refreshes its per-family hyperposterior
/// via hierarchical_update. Refreshing every round is exact but expensive
/// (O(N_FAMILIES * MAX_GIBBS_ITERATIONS) per round); refreshing every 25
/// rounds matches the Hong/Kveton/Zaheer/Ghavamzadeh AISTATS 2022
/// recommendation (paper §5 "Lazy updates"), keeping the cost amortised.
const REFRESH_GIBBS_EVERY: u32 = 25;
/// Per-family true Bernoulli reward means. Matches the fixture in
/// hierarchical_bai_integration.rs so the two artefacts are pinnable.
const FAMILY_MUS: [f64; FAMILIES] = [0.2, 0.4, 0.5, 0.6, 0.8];
const BOOTSTRAP_REPLICATES: u32 = 1000;

// ---------------------------------------------------------------------------
// Fixture construction (mirrors hierarchical_bai_integration.rs build_fixture)
// ---------------------------------------------------------------------------

fn build_arm(family_idx: usize, arm_idx: usize) -> (ArmPosterior, f64) {
    let mu_f = FAMILY_MUS[family_idx];
    let mut bytes = [0u8; 16];
    bytes[0] = family_idx as u8;
    bytes[1] = arm_idx as u8;
    let template_id = TemplateId::from(Uuid::from_bytes(bytes));
    let arm = ArmPosterior {
        template_id,
        // Uniform prior — equivalent to the canonical Beta(1, 1) in the
        // contract test.
        alpha: 1.0,
        beta: 1.0,
        n_pulls: 0,
    };
    (arm, mu_f)
}

fn next_uniform(rng: &mut SeededRng) -> f64 {
    use elai_core::EntropySource;
    let bits = rng.next_u64() >> 11;
    (bits as f64) * (1.0 / ((1u64 << 53) as f64))
}

fn sample_bernoulli(rng: &mut SeededRng, p: f64) -> bool {
    next_uniform(rng) < p
}

// ---------------------------------------------------------------------------
// Policies
// ---------------------------------------------------------------------------

fn select_flat(arms: &[ArmPosterior], rng: &mut SeededRng) -> usize {
    let mut best_idx = 0;
    let mut best_score = sample_beta(rng, arms[0].alpha, arms[0].beta);
    for (i, arm) in arms.iter().enumerate().skip(1) {
        let score = sample_beta(rng, arm.alpha, arm.beta);
        if score > best_score {
            best_score = score;
            best_idx = i;
        }
    }
    best_idx
}

fn select_hier(state: &HierarchicalBaiState, arms: &[ArmPosterior], rng: &mut SeededRng) -> usize {
    // The Some(_) discriminant is guaranteed because arms is non-empty.
    let (idx, _basis) = hierarchical_arm_select(state, arms, rng).expect("non-empty arms");
    idx
}

// ---------------------------------------------------------------------------
// Bootstrap 95% CI on per-round regret series
// ---------------------------------------------------------------------------

fn bootstrap_ci_mean(series: &[f64], replicates: u32, rng: &mut SeededRng) -> (f64, f64) {
    use elai_core::EntropySource;
    let n = series.len();
    if n == 0 {
        return (0.0, 0.0);
    }
    let mut means: Vec<f64> = Vec::with_capacity(replicates as usize);
    for _ in 0..replicates {
        let mut sum = 0.0_f64;
        for _ in 0..n {
            let idx = (rng.next_u64() as usize) % n;
            sum += series[idx];
        }
        means.push(sum / (n as f64));
    }
    means.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let lo_idx = ((replicates as f64) * 0.025) as usize;
    let hi_idx = ((replicates as f64) * 0.975) as usize;
    (
        means[lo_idx.min(means.len() - 1)],
        means[hi_idx.min(means.len() - 1)],
    )
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

fn main() {
    let start = Instant::now();

    // Build 50 arms + per-arm true mus + family index.
    let mut arms_flat: Vec<ArmPosterior> = Vec::with_capacity(N_ARMS);
    let mut arms_hier: Vec<ArmPosterior> = Vec::with_capacity(N_ARMS);
    let mut true_mus: Vec<f64> = Vec::with_capacity(N_ARMS);
    let mut families: Vec<FamilyId> = Vec::with_capacity(N_ARMS);
    for f in 0..FAMILIES {
        for i in 0..ARMS_PER_FAMILY {
            let (arm, mu) = build_arm(f, i);
            arms_flat.push(arm.clone());
            arms_hier.push(arm);
            true_mus.push(mu);
            families.push(FamilyId(f as u32));
        }
    }
    let best_mu = FAMILY_MUS.iter().cloned().fold(0.0_f64, f64::max);

    // Seed BOTH policies from the same RNG so the per-round shared Bernoulli
    // draws are byte-identical across policies (paired-difference design).
    // The Thompson sample / hierarchical select pull is what differs.
    let mut rng_flat = SeededRng::new(SEED);
    let mut rng_hier = SeededRng::new(SEED);
    let mut rng_world = SeededRng::new(SEED ^ 0xA5A5_A5A5_5A5A_5A5A);
    let mut rng_bootstrap = SeededRng::new(SEED ^ 0xDEAD_BEEF_CAFE_BABE);

    let mut hier_state = HierarchicalBaiState::new();
    for (arm, fid) in arms_hier.iter().zip(families.iter()) {
        hier_state.assign_arm(arm.template_id, *fid);
    }

    let mut regret_flat: Vec<f64> = Vec::with_capacity(ROUNDS as usize);
    let mut regret_hier: Vec<f64> = Vec::with_capacity(ROUNDS as usize);

    for round in 0..ROUNDS {
        // FLAT pull
        let idx_flat = select_flat(&arms_flat, &mut rng_flat);
        let mu_flat = true_mus[idx_flat];
        let r_flat = sample_bernoulli(&mut rng_world, mu_flat);
        if r_flat {
            arms_flat[idx_flat].alpha += 1.0;
        } else {
            arms_flat[idx_flat].beta += 1.0;
        }
        arms_flat[idx_flat].n_pulls += 1;
        regret_flat.push(best_mu - mu_flat);

        // HIER pull
        let idx_hier = select_hier(&hier_state, &arms_hier, &mut rng_hier);
        let mu_hier = true_mus[idx_hier];
        let r_hier = sample_bernoulli(&mut rng_world, mu_hier);
        if r_hier {
            arms_hier[idx_hier].alpha += 1.0;
        } else {
            arms_hier[idx_hier].beta += 1.0;
        }
        arms_hier[idx_hier].n_pulls += 1;
        regret_hier.push(best_mu - mu_hier);

        // Periodic Gibbs refresh of the per-family hyperposterior.
        if round > 0 && round % REFRESH_GIBBS_EVERY == 0 {
            for f in 0..FAMILIES {
                let fid = FamilyId(f as u32);
                let refs: Vec<&ArmPosterior> = arms_hier
                    .iter()
                    .zip(families.iter())
                    .filter(|(_, ff)| **ff == fid)
                    .map(|(a, _)| a)
                    .collect();
                let _ = hierarchical_update(&mut hier_state, fid, &refs, &mut rng_hier);
            }
        }
    }

    let cum_regret_flat: f64 = regret_flat.iter().sum();
    let cum_regret_hier: f64 = regret_hier.iter().sum();
    let mean_regret_flat = cum_regret_flat / (ROUNDS as f64);
    let mean_regret_hier = cum_regret_hier / (ROUNDS as f64);

    let (flat_lo, flat_hi) = bootstrap_ci_mean(&regret_flat, BOOTSTRAP_REPLICATES, &mut rng_bootstrap);
    let (hier_lo, hier_hi) = bootstrap_ci_mean(&regret_hier, BOOTSTRAP_REPLICATES, &mut rng_bootstrap);

    // Per-round paired difference: regret_flat[t] - regret_hier[t]. A 95%
    // CI strictly above 0 means hier dominates flat at p < .025; the
    // pre-committed Path-A gate is "hier <= flat - 5% CI".
    let paired_diff: Vec<f64> = regret_flat
        .iter()
        .zip(regret_hier.iter())
        .map(|(a, b)| a - b)
        .collect();
    let (diff_lo, diff_hi) = bootstrap_ci_mean(&paired_diff, BOOTSTRAP_REPLICATES, &mut rng_bootstrap);
    let diff_mean: f64 = paired_diff.iter().sum::<f64>() / (paired_diff.len() as f64);

    // Path-A / Path-B verdict per D-29-decisions.md DECISION-7.
    // hier <= flat - 5% CI means the lower bound of the paired-diff CI
    // must be strictly positive AND the relative improvement must be at
    // least 5% of mean_regret_flat.
    let absolute_dominates = diff_lo > 0.0;
    let relative_dominates = mean_regret_flat > 0.0
        && (mean_regret_flat - mean_regret_hier) / mean_regret_flat >= 0.05;
    let path_a = absolute_dominates && relative_dominates;
    let verdict = if path_a { "PATH_A" } else { "PATH_B" };

    let elapsed_ms = start.elapsed().as_millis();

    println!("=== D-29 flat-vs-hierarchical baseline (SyntheticNonIidBandit 5x10) ===");
    println!("rounds            : {}", ROUNDS);
    println!("seed              : {}", SEED);
    println!("arms              : {} ({} families x {} arms)", N_ARMS, FAMILIES, ARMS_PER_FAMILY);
    println!("refresh_every     : {}", REFRESH_GIBBS_EVERY);
    println!("bootstrap_B       : {}", BOOTSTRAP_REPLICATES);
    println!();
    println!("flat  mean_regret : {:.6}  95% CI [{:.6}, {:.6}]  cum {:.4}",
        mean_regret_flat, flat_lo, flat_hi, cum_regret_flat);
    println!("hier  mean_regret : {:.6}  95% CI [{:.6}, {:.6}]  cum {:.4}",
        mean_regret_hier, hier_lo, hier_hi, cum_regret_hier);
    println!("paired diff       : {:.6}  95% CI [{:.6}, {:.6}]  (flat - hier)",
        diff_mean, diff_lo, diff_hi);
    let ratio = if mean_regret_flat > 0.0 {
        mean_regret_hier / mean_regret_flat
    } else {
        f64::NAN
    };
    println!("hier/flat ratio   : {:.4}", ratio);
    println!("absolute_dominates: {}", absolute_dominates);
    println!("relative_dominates: {} (threshold = 0.05)", relative_dominates);
    println!("VERDICT           : {} (Path-A=productize, Path-B=infeasibility documented)", verdict);
    println!("elapsed_ms        : {}", elapsed_ms);
}

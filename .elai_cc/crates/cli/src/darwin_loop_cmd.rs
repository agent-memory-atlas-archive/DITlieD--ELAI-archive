//! E-7 -- `elai darwin-loop` CLI dispatcher.
//!
//! Exercises the darwinian-evolve substrate (DiscountedTsConfig +
//! DiscountedArmPosterior + SuccessiveRejects + trap_tasks fixtures)
//! as a single CLI surface so the wire-format runs end-to-end without
//! a live multi-agent loop. The live evolution loop (population
//! mutation, cross-domain canary gate, promotion vote) is
//! RUNPOD-PENDING and gated behind `cfg(feature = "darwinian-evolve")`
//! + `ELAI_DARWINIAN_LOOP_ENABLE=1` per E-7 DECISION-13.
//!
//! Flags:
//!   --generations N    (default 1)        Number of evolve rounds.
//!   --mutants K        (default 3)        Arms per round.
//!   --seed S           (default 42)       Deterministic seed (echoed only).
//!   --traps T          (default 3)        Trap-task fixture rows to score.
//!   --gamma G          (default DEFAULT_DISCOUNT_FACTOR)  DS-TS discount.
//!
//! NO SsbActionType variants are emitted (per E-7 wire-dark default).
//! NO live sidecar call. Output prints per-arm posterior mean +
//! SuccessiveRejects log_bar + trap-pattern enumeration so the surface
//! is observable in CI.

#[cfg(feature = "darwinian-evolve")]
use elai_orchestrator::darwinian_bai::{
    DiscountedArmPosterior, DiscountedTsConfig, SuccessiveRejects, DEFAULT_DISCOUNT_FACTOR,
};
#[cfg(feature = "darwinian-evolve")]
use elai_orchestrator::trap_tasks::{TrapPattern, REQUIRED_TRAP_COUNT, TRAP_WEIGHT_MULTIPLIER};

/// E-7 dispatch shim. Accepts the raw `gamma` String from args, parses it
/// internally, and branches on the feature flag so `main.rs` is cfg-agnostic.
///
/// Feature-on  : parses gamma -> f64, calls [`run`].
/// Feature-off : prints DISABLED message, returns Ok(()).
pub(crate) fn dispatch(
    generations: u32,
    mutants: u32,
    seed: u64,
    traps: u32,
    gamma: String,
) -> anyhow::Result<()> {
    #[cfg(feature = "darwinian-evolve")]
    {
        let gamma_f: f64 = if gamma.is_empty() {
            DEFAULT_DISCOUNT_FACTOR
        } else {
            gamma
                .parse()
                .map_err(|e| anyhow::anyhow!("invalid --gamma `{gamma}`: {e}"))?
        };
        return run(generations, mutants, seed, traps, gamma_f);
    }
    #[cfg(not(feature = "darwinian-evolve"))]
    {
        println!(
            "darwin-loop DISABLED (rebuild with --features darwinian-evolve) \
             requested generations={generations} mutants={mutants} seed={seed} \
             traps={traps} gamma={gamma}"
        );
        Ok(())
    }
}

/// CLI entry — sync, no tokio runtime needed (pure compute).
#[cfg(feature = "darwinian-evolve")]
pub(crate) fn run(
    generations: u32,
    mutants: u32,
    seed: u64,
    traps: u32,
    gamma: f64,
) -> Result<(), anyhow::Error> {
    if !(0.0..=1.0).contains(&gamma) {
        return Err(anyhow::anyhow!("--gamma {gamma} must be in (0.0, 1.0]"));
    }
    let mutants = mutants.max(2) as usize;
    let traps = traps.max(1) as usize;

    let cfg = DiscountedTsConfig {
        discount_factor: gamma,
    };
    // Synthetic deterministic reward signal (no RNG dep — caller supplies seed
    // for the future live wire). Reward pattern: alternating 1.0 / 0.0 so the
    // DS-TS update path is exercised on both branches; trap rows weighted at
    // TRAP_WEIGHT_MULTIPLIER per E-7 plan task 8.
    let trap_patterns = [
        TrapPattern::CoastRunnersGreenBlock,
        TrapPattern::MarkerRemoval,
        TrapPattern::FakeTestLog,
        TrapPattern::VerifierCapture,
        TrapPattern::MetricOverflow,
        TrapPattern::LegoStackingFlip,
        TrapPattern::UnitTestEdit,
    ];

    let mut arms: Vec<DiscountedArmPosterior> =
        (0..mutants).map(|_| DiscountedArmPosterior::default()).collect();
    let log_bar = SuccessiveRejects::log_bar(mutants);
    let sr = SuccessiveRejects::new(mutants as u32 * generations.max(1));

    for gen_idx in 0..generations.max(1) {
        for (arm_idx, arm) in arms.iter_mut().enumerate() {
            // Alternating reward + trap weight; deterministic.
            let raw = if (arm_idx + gen_idx as usize) % 2 == 0 { 1.0 } else { 0.0 };
            let trap_hit = (arm_idx % traps.max(1)) == 0;
            let weighted: f64 = if trap_hit {
                let v: f64 = raw / TRAP_WEIGHT_MULTIPLIER;
                if v > 1.0 { 1.0 } else { v }
            } else {
                raw
            };
            arm.discounted_update(weighted, cfg.discount_factor);
        }
    }

    println!(
        "darwin-loop generations={generations} mutants={mutants} seed={seed} traps={traps} gamma={:.6} required_trap_count={REQUIRED_TRAP_COUNT} sr_budget={} log_bar={log_bar:.4}",
        cfg.discount_factor, sr.budget,
    );
    for (i, arm) in arms.iter().enumerate() {
        println!(
            "  arm={i} alpha={:.4} beta={:.4} mean={:.4} n_pulls={}",
            arm.alpha, arm.beta, arm.mean(), arm.n_pulls,
        );
    }
    println!("  trap_patterns_seeded={}", trap_patterns.len());
    println!("darwin-loop COMPLETE (RUNPOD-PENDING live evolution loop)");
    Ok(())
}

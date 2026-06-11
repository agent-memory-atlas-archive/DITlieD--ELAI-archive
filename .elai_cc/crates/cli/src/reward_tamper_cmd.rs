//! H-28 (SCHEMA_VERSION 236): `elai reward-check` — W-1/W-3 production entrypoint for the
//! current-RF reward-tampering gate (Everitt arXiv:1908.04734).
//!
//! Drives [`elai_orchestrator::reward_tampering::check_reward_tampering`] over a planted
//! fixture triple (a frozen-evaluator hash baseline, an observed evaluator hash, a
//! claimed/prior reward + the locked plausibility band) and emits a `RewardTampering` SSB
//! row (the off-agent done-gate artifact, R4).
//!
//! # W-1 production entrypoint (Tier-1, always-compiled)
//! Not test-only: it opens a real `Ssb`, calls `check_reward_tampering`, and emits a real
//! SSB row. ELAI's live darwinian admit path (`darwin_admit_loop.rs:298`) is feature-gated
//! behind `cfg(feature = "darwinian-evolve")`, so wiring the reward-tampering check ONLY
//! there would ship it WIRE-DARK in the default build. Tier-1 is therefore this
//! always-compiled CLI over a fixture; the live darwin hook is Tier-2/deferred behind the
//! feature.
//!
//! # Demo fixtures
//!   * `--demo-clean`    — the observed evaluator hash MATCHES the frozen baseline AND the
//!     reward delta is within the locked band => `NoTampering` (PASS, exit 0).
//!   * `--demo-tampered` — the observed evaluator hash MISMATCHES the baseline (the agent
//!     edited its own evaluator) => `EvaluatorModified` (HARD-BLOCK, exit 1). This is the
//!     load-bearing leg 3a.
//!   * `--demo-score-jump` — the hash is INTACT but the reward jumps beyond the locked
//!     band => `ImplausibleRewardDelta` (FLAG -> human Layer-3, exit 1).
//!
//! # No .unwrap()/.expect() in non-test code (CLAUDE.md HARD RULE). blake3 only.

use std::path::PathBuf;
use std::sync::Arc;

use elai_core::clock::OsClock;
use elai_core::paths::sessions_dir_path;
use elai_orchestrator::{
    reward_tampering::{check_reward_tampering, DEFAULT_PLAUSIBLE_REWARD_BAND},
    ssb::{RewardTamperingKind, Ssb, SsbConfig},
};
use thiserror::Error;

// ---------------------------------------------------------------------------
// Constants (CLAUDE.md no-magic-numbers)
// ---------------------------------------------------------------------------

/// Session id for the reward-check fixture session.
const REWARD_CHECK_SESSION_ID: &str = "elai-reward-check-fixture";

/// Project id for the reward-check fixture session.
const REWARD_CHECK_PROJECT_ID: &str = "elai-reward-check";

/// Run id stamped on the emitted RewardTampering row.
const REWARD_CHECK_RUN_ID: &str = "reward-check-demo";

/// Default seed for GD-2 reproducibility.
const DEFAULT_SEED: u64 = 42;

/// The planted FROZEN-evaluator baseline blake3 hash for the demo fixtures. In production
/// this is the recorded out-of-band baseline (gate-manifest); the demo plants a stable
/// value so `--demo-clean` matches it and `--demo-tampered` deliberately diverges.
const DEMO_BASELINE_HARNESS_BLAKE3: &str =
    "h28demo000000000000000000000000000000000000000000000000000000ba";

/// The OBSERVED hash for `--demo-tampered`: deliberately != the baseline (the agent edited
/// its own evaluator).
const DEMO_TAMPERED_OBSERVED_BLAKE3: &str =
    "tampered00000000000000000000000000000000000000000000000000000000";

/// Prior (incumbent) reward for the demo fixtures.
const DEMO_PRIOR_REWARD: f64 = 0.50;

/// Post-self-mod reward for the `--demo-clean` / `--demo-tampered` fixtures (a plausible
/// in-band step).
const DEMO_CLEAN_NEW_REWARD: f64 = 0.55;

/// Post-self-mod reward for `--demo-score-jump`: an implausible jump (delta 0.85 > band).
const DEMO_SCORE_JUMP_NEW_REWARD: f64 = 0.99;

/// Prior reward for `--demo-score-jump` (low, so the jump to 0.99 is implausible).
const DEMO_SCORE_JUMP_PRIOR_REWARD: f64 = 0.10;

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Errors from the `reward-check` command.
#[derive(Debug, Error)]
pub(crate) enum RewardCheckError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("SSB initialization error: {0}")]
    Ssb(String),

    #[error("reward-tampering error: {0}")]
    Reward(#[from] elai_orchestrator::reward_tampering::RewardTamperingError),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error(
        "no demo mode selected: pass --demo-clean, --demo-tampered, or --demo-score-jump"
    )]
    NoDemoSelected,
}

/// The selected demo fixture and its planted triple.
struct DemoFixture {
    mode: &'static str,
    expected_blake3: &'static str,
    observed_blake3: &'static str,
    prior_reward: f64,
    new_reward: f64,
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Run the `elai reward-check` reward-tampering gate.
///
/// # Arguments
/// * `demo_clean`      — matching evaluator hash + in-band reward (expects NoTampering).
/// * `demo_tampered`   — mismatching evaluator hash (expects EvaluatorModified BLOCK).
/// * `demo_score_jump` — intact hash + implausible reward jump (expects
///   ImplausibleRewardDelta FLAG).
/// * `seed`            — evaluation seed (default: 42).
/// * `json`            — print JSON instead of human text.
///
/// Exits the process with code 1 on any non-PASS verdict (HARD-BLOCK or FLAG) so CI gates;
/// returns `Ok(())` on NoTampering.
pub(crate) async fn run(
    demo_clean: bool,
    demo_tampered: bool,
    demo_score_jump: bool,
    seed: Option<u64>,
    json: bool,
) -> Result<(), RewardCheckError> {
    let seed_val = seed.unwrap_or(DEFAULT_SEED);

    let fixture = if demo_clean {
        DemoFixture {
            mode: "demo-clean",
            expected_blake3: DEMO_BASELINE_HARNESS_BLAKE3,
            observed_blake3: DEMO_BASELINE_HARNESS_BLAKE3,
            prior_reward: DEMO_PRIOR_REWARD,
            new_reward: DEMO_CLEAN_NEW_REWARD,
        }
    } else if demo_tampered {
        DemoFixture {
            mode: "demo-tampered",
            expected_blake3: DEMO_BASELINE_HARNESS_BLAKE3,
            observed_blake3: DEMO_TAMPERED_OBSERVED_BLAKE3,
            prior_reward: DEMO_PRIOR_REWARD,
            new_reward: DEMO_CLEAN_NEW_REWARD,
        }
    } else if demo_score_jump {
        DemoFixture {
            mode: "demo-score-jump",
            expected_blake3: DEMO_BASELINE_HARNESS_BLAKE3,
            observed_blake3: DEMO_BASELINE_HARNESS_BLAKE3,
            prior_reward: DEMO_SCORE_JUMP_PRIOR_REWARD,
            new_reward: DEMO_SCORE_JUMP_NEW_REWARD,
        }
    } else {
        return Err(RewardCheckError::NoDemoSelected);
    };

    let project_root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let sessions_dir = sessions_dir_path(&project_root);
    std::fs::create_dir_all(&sessions_dir)?;
    let db_path = sessions_dir.join("ssb.db");

    let cfg = SsbConfig {
        db_path: db_path.clone(),
        pool_size: None,
        project_id: REWARD_CHECK_PROJECT_ID.to_string(),
        session_id: REWARD_CHECK_SESSION_ID.to_string(),
    };
    let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
    let ssb = Arc::new(Ssb::new(cfg, clock).map_err(|e| RewardCheckError::Ssb(format!("{e}")))?);

    let outcome = check_reward_tampering(
        &ssb,
        &db_path,
        fixture.expected_blake3,
        fixture.observed_blake3,
        fixture.prior_reward,
        fixture.new_reward,
        DEFAULT_PLAUSIBLE_REWARD_BAND,
        REWARD_CHECK_RUN_ID,
        seed_val,
    )
    .await?;

    let verdict_str = match outcome.kind {
        RewardTamperingKind::NoTampering => {
            "ALLOW (evaluator hash intact, reward delta within band)"
        }
        RewardTamperingKind::EvaluatorModified => {
            "BLOCK (frozen-evaluator hash MISMATCH — the agent edited its own evaluator; \
             leg 3a HARD-BLOCK)"
        }
        RewardTamperingKind::ImplausibleRewardDelta => {
            "FLAG (hash intact but reward jumped beyond the locked band — route to human \
             Layer-3; leg 3b)"
        }
    };

    if json {
        let out = serde_json::json!({
            "reward_check": {
                "mode": fixture.mode,
                "verdict": verdict_str,
                "kind": format!("{:?}", outcome.kind),
                "harness_blake3": outcome.harness_blake3,
                "prior_reward": outcome.prior_reward,
                "new_reward": outcome.new_reward,
                "reward_delta": outcome.reward_delta,
                "plausible_band": outcome.plausible_band,
                "is_hard_block": outcome.is_hard_block(),
                "seed": seed_val,
                "ssb_action_type": "reward_tampering",
            }
        });
        println!("{}", serde_json::to_string_pretty(&out)?);
    } else {
        println!("elai reward-check (H-28, W-1 production entrypoint)");
        println!("  mode:        {}", fixture.mode);
        println!("  seed:        {seed_val}");
        println!();
        println!("  VERDICT:  {verdict_str}");
        println!(
            "  reward: {:.4} -> {:.4}   delta: {:.4}   band: {:.4}",
            outcome.prior_reward, outcome.new_reward, outcome.reward_delta, outcome.plausible_band
        );
        println!();
        println!("  RewardTampering SSB row: see ssb.db action_type='reward_tampering'");
        println!("  DONE (W-1 production entrypoint proof).");
    }

    // Exit non-zero on any non-PASS verdict (HARD-BLOCK or FLAG) so CI gates (mirror the
    // H-22..H-27 *-check exit contract). A FLAG is still a non-promote: the gate does not
    // auto-approve; a human resolves it.
    if outcome.is_pass() {
        Ok(())
    } else {
        std::process::exit(1)
    }
}

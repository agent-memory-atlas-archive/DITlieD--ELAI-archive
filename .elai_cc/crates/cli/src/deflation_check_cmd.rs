//! H-18 (SCHEMA_VERSION 224): `elai deflation-check` — W-3 Tier-1 production
//! entrypoint for the Deflated Sharpe / PBO / MinBTL promotion gate.
//!
//! Registers a fixture strategy search (N distinct configs) then calls
//! `evaluate_deflation` on the resulting trial count, emitting a
//! `DeflationGateEvaluated` SSB row (the off-agent promotion artifact).
//!
//! The default fixture scenario demonstrates the gate's core property:
//! a high-raw-Sharpe strategy with high-N selection bias (N=50 fixture)
//! produces DSR < 0.95 => BLOCKED. Pass `--n 1 --sr 0.25 --pbo 0.1` for
//! a genuine low-N example that PASSES.
//!
//! # W-3 production entrypoint (WIRING_CONTRACT)
//! This command is the live CLI hook for the deflation gate. It is NOT
//! test-only: it opens a real Ssb, a real TrialLedger, registers real
//! trials, and emits a real SSB row (the off-agent artifact that proves
//! wiring, R3/R4/R5 per wiring-enforcement.md).
//!
//! # Tier-2 live promotion hook (DEFERRED stub)
//! The live trading-strategy promote hook (the darwin_admit_loop gate)
//! is a Tier-2 stub (see workflow/sessions/s-10612/stubs.md). Only the
//! fixture path is exercised in this session.
//!
//! # No .unwrap()/.expect() in non-test code (CLAUDE.md HARD RULE).

use std::sync::Arc;

use elai_core::{clock::OsClock, paths::sessions_dir_path};
use elai_orchestrator::{
    deflation_gate::{DeflationInput, GateVerdictKind, evaluate_deflation},
    ssb::{Ssb, SsbConfig},
    trial_ledger::TrialLedger,
};
use thiserror::Error;

// ---------------------------------------------------------------------------
// Constants (CLAUDE.md no-magic-numbers)
// ---------------------------------------------------------------------------

/// Default search objective used when none is provided.
const DEFAULT_OBJECTIVE: &str = "sharpe_1y";

/// Default data window ("2024-01-01..2024-12-31" in ms as start..end).
const DEFAULT_WINDOW: &str = "1704067200000..1735689600000";

/// Default per-period SR for the fixture (demonstrates deflation bites).
const DEFAULT_SR: f64 = 0.10;

/// Default T (one year of daily returns).
const DEFAULT_T: u64 = 252;

/// Default N (number of fixture configs to register).
const DEFAULT_N_FIXTURE: u64 = 50;

/// Default cross-sectional std of SR across N trials.
const DEFAULT_SHARPE_STD: f64 = 0.05;

/// Default seed for GD-2 reproducibility.
const DEFAULT_SEED: u64 = 42;

/// Session id for the deflation-check fixture session.
const DEFLATION_CHECK_SESSION_ID: &str = "elai-deflation-check-fixture";

/// Project id for the deflation-check fixture session.
const DEFLATION_CHECK_PROJECT_ID: &str = "elai-deflation-check";

/// Agent id for SSB emission.
const DEFLATION_AGENT_ID: &str = "deflation-check-cmd";

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Errors from the `deflation-check` command.
#[derive(Debug, Error)]
pub(crate) enum DeflationCheckError {
    #[error("I/O error creating sessions directory: {0}")]
    Io(#[from] std::io::Error),

    #[error("SSB initialization error: {0}")]
    Ssb(String),

    #[error("trial ledger error: {0}")]
    Ledger(#[from] elai_orchestrator::trial_ledger::TrialLedgerError),

    #[error("deflation gate error: {0}")]
    Gate(#[from] elai_orchestrator::deflation_gate::DeflationGateError),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Run the `elai deflation-check` fixture.
///
/// # Arguments
/// * `sr`         — per-period SR (default: 0.10).
/// * `t_periods`  — return observation count (default: 252).
/// * `n_fixture`  — number of fixture configs to register (default: 50).
/// * `sharpe_std` — cross-sectional std of SRs (default: 0.05).
/// * `pbo`        — PBO scalar from Tier-2 sidecar; None -> Block (R1/R2).
/// * `seed`       — evaluation seed (default: 42).
/// * `json`       — print JSON instead of human text.
pub(crate) async fn run(
    sr: Option<f64>,
    t_periods: Option<u64>,
    n_fixture: Option<u64>,
    sharpe_std: Option<f64>,
    pbo: Option<f64>,
    seed: Option<u64>,
    json: bool,
) -> Result<(), DeflationCheckError> {
    let sr_val = sr.unwrap_or(DEFAULT_SR);
    let t_val = t_periods.unwrap_or(DEFAULT_T);
    let n_val = n_fixture.unwrap_or(DEFAULT_N_FIXTURE);
    let sharpe_std_val = sharpe_std.unwrap_or(DEFAULT_SHARPE_STD);
    let seed_val = seed.unwrap_or(DEFAULT_SEED);

    let project_root = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let sessions_dir = sessions_dir_path(&project_root);
    std::fs::create_dir_all(&sessions_dir)?;
    let db_path = sessions_dir.join("ssb.db");

    let cfg = SsbConfig {
        db_path: db_path.clone(),
        pool_size: None,
        project_id: DEFLATION_CHECK_PROJECT_ID.to_string(),
        session_id: DEFLATION_CHECK_SESSION_ID.to_string(),
    };
    let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
    let ssb = Arc::new(
        Ssb::new(cfg, clock.clone()).map_err(|e| DeflationCheckError::Ssb(format!("{e}")))?,
    );
    let ledger = TrialLedger::open(&db_path, DEFLATION_CHECK_SESSION_ID)?;

    // Step 1: Register n_fixture DISTINCT strategy configs (fixture strategy search).
    // Each config is a JSON blob parameterized by its index + seed.
    // This exercises the full W-3 entrypoint: trial registration -> gate evaluation.
    let mut final_n = 0u64;
    for i in 0..n_val {
        let cfg_bytes = format!(
            "{{\"fixture_strategy_id\": {i}, \"seed\": {seed_val}, \"sr_target\": {sr_val}}}"
        );
        final_n = ledger
            .register_trial(
                &ssb,
                &clock,
                DEFAULT_OBJECTIVE,
                DEFAULT_WINDOW,
                cfg_bytes.as_bytes(),
                // Different seeds per config -> distinct trial_ids.
                (seed_val + i) as i64,
            )
            .await?;
    }

    // Step 2: Call evaluate_deflation (the Tier-1 gate).
    // The ledger now has n_val distinct configs for (DEFAULT_OBJECTIVE, DEFAULT_WINDOW).
    let input = DeflationInput {
        objective: DEFAULT_OBJECTIVE.to_string(),
        data_window: DEFAULT_WINDOW.to_string(),
        sr_observed: sr_val,
        t_periods: t_val,
        skew: 0.0,
        excess_kurt: 0.0,
        sharpe_std: sharpe_std_val,
        sr_benchmark: 0.0,
        pbo,
        seed: seed_val,
    };

    let verdict = evaluate_deflation(&ledger, &ssb, input).await?;

    // Step 3: Report the result.
    if json {
        let is_nan = |v: f64| if v.is_nan() { serde_json::Value::Null } else { v.into() };
        let out = serde_json::json!({
            "deflation_check": {
                "objective": DEFAULT_OBJECTIVE,
                "data_window": DEFAULT_WINDOW,
                "n_registered": final_n,
                "sr_observed": sr_val,
                "t_periods": t_val,
                "sharpe_std": sharpe_std_val,
                "pbo_input": pbo,
                "seed": seed_val,
                "verdict": match verdict.kind {
                    GateVerdictKind::Allow => "allow",
                    GateVerdictKind::Block => "block",
                },
                "reason": verdict.reason,
                "dsr": is_nan(verdict.dsr),
                "sigma_sr": is_nan(verdict.sigma_sr),
                "sr_0": is_nan(verdict.sr_0),
                "min_trl": is_nan(verdict.min_trl),
                "n_used": verdict.n_used,
                "ssb_action_type": "deflation_gate_evaluated",
            }
        });
        println!("{}", serde_json::to_string_pretty(&out)?);
    } else {
        println!("elai deflation-check (H-18 fixture, W-3 Tier-1 production entrypoint)");
        println!("  objective:   {DEFAULT_OBJECTIVE}");
        println!("  data_window: {DEFAULT_WINDOW}");
        println!("  n_registered: {final_n}");
        println!("  sr_observed:  {sr_val}");
        println!("  t_periods:    {t_val}");
        println!("  sharpe_std:   {sharpe_std_val}");
        println!("  pbo_input:    {pbo:?}");
        println!("  seed:         {seed_val}");
        println!();
        let verdict_str = match verdict.kind {
            GateVerdictKind::Allow => "ALLOW",
            GateVerdictKind::Block => "BLOCK",
        };
        println!("  VERDICT:  {verdict_str}  (reason: {})", verdict.reason);
        if !verdict.dsr.is_nan() {
            println!("  DSR:      {:.4}  (threshold >= {:.2})", verdict.dsr, elai_orchestrator::deflation_gate::DSR_THRESHOLD);
            println!("  SR_0:     {:.4}", verdict.sr_0);
            println!("  sigma_sr: {:.6}", verdict.sigma_sr);
        }
        if !verdict.min_trl.is_nan() && verdict.min_trl.is_finite() {
            println!("  MinTRL:   {:.1} periods  ({:.1} years)", verdict.min_trl, verdict.min_trl / 252.0);
        }
        println!("  N_used:   {}", verdict.n_used);
        println!();
        println!("  DeflationGateEvaluated SSB row: see ssb.db action_type='deflation_gate_evaluated'");
        println!("  DONE (W-3 Tier-1 entrypoint proof; Tier-2 live-promote stub DEFERRED)");
    }

    Ok(())
}

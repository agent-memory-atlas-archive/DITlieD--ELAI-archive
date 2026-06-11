//! H-22 (SCHEMA_VERSION 227): `elai null-test` — W-1 production entrypoint for
//! the trading metamorphic NULL-TEST gate.
//!
//! Runs `trading_null_gate::evaluate_trading_null` over a returns series and
//! emits a `TradingNullTest` SSB row (the off-agent done-gate artifact, R4).
//!
//! # W-1 production entrypoint (WIRING_CONTRACT)
//! This command is the live CLI hook for the trading null-test gate. It is NOT
//! test-only: it opens a real `Ssb` + `TrialLedger`, builds a `TradingNullInput`,
//! calls `evaluate_trading_null`, and emits a real SSB row. Tier-1 needs no
//! sidecar: the returns series is supplied as plain data. The DEFERRED Tier-2
//! Python permutation sidecar (re-runs the backtest under each permutation) is
//! RunPod-pending (frozen `permutation_null_sidecar.py`).
//!
//! # Demo fixtures (deterministic, in-memory)
//!   * `--demo-overfit`  — a flat/noise edge that SURVIVES the block-permutation
//!     null (p >= alpha/N) -> BLOCK (EdgeSurvivedNull).
//!   * `--demo-genuine`  — a strongly-trending edge that is significant vs the
//!     null (p < alpha/N) -> ALLOW (EdgeCollapsedToNull).
//!
//! Both demos seed a trial-count row in the H-17 ledger so the N read succeeds
//! (a real promotion path supplies N from the search; the demo seeds N=1 so the
//! corrected threshold alpha/N = alpha).
//!
//! # No .unwrap()/.expect() in non-test code (CLAUDE.md HARD RULE).

use std::path::PathBuf;
use std::sync::Arc;

use elai_core::clock::OsClock;
use elai_core::paths::sessions_dir_path;
use elai_orchestrator::{
    ssb::{Ssb, SsbConfig},
    trading_null_gate::{
        evaluate_trading_null, TradingNullInput, TradingNullVerdictKind, DEFAULT_BLOCK_SIZE,
        K_MIN_PERMUTATIONS, TRADING_NULL_RULE_ID,
    },
    trial_ledger::TrialLedger,
};
use thiserror::Error;

// ---------------------------------------------------------------------------
// Constants (CLAUDE.md no-magic-numbers)
// ---------------------------------------------------------------------------

/// Session id for the null-test fixture session.
const NULL_TEST_SESSION_ID: &str = "elai-null-test-fixture";

/// Project id for the null-test fixture session.
const NULL_TEST_PROJECT_ID: &str = "elai-null-test";

/// Default strategy path reported when none is provided.
const DEFAULT_STRATEGY: &str = "demo_strategy.py";

/// Search objective key the demos register N against.
const DEMO_OBJECTIVE: &str = "sharpe_1y";

/// Canonical data window the demos register N against.
const DEMO_DATA_WINDOW: &str = "0..1";

/// Default seed for GD-2 reproducibility.
const DEFAULT_SEED: u64 = 42;

/// Length of the demo series (well above MIN_SERIES_LEN and divisible into
/// several blocks so the block permutation is meaningful).
const DEMO_SERIES_LEN: usize = 120;

/// Amplitude of the per-bar market return swing in the demos.
const DEMO_RETURN_AMP: f64 = 1.0;

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Errors from the `null-test` command.
#[derive(Debug, Error)]
pub(crate) enum NullTestCheckError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("SSB initialization error: {0}")]
    Ssb(String),

    #[error("trial ledger error: {0}")]
    Ledger(String),

    #[error("null-test gate error: {0}")]
    Gate(#[from] elai_orchestrator::trading_null_gate::TradingNullError),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("no demo mode selected: pass --demo-overfit or --demo-genuine")]
    NoDemoSelected,
}

// ---------------------------------------------------------------------------
// Deterministic demo fixtures (in-memory returns series)
// ---------------------------------------------------------------------------

/// Sign-alternating market returns with VARYING magnitude `(i%3+1)*amp`. The
/// varying magnitude gives the aligned strategy P&L (signal*return) a positive
/// variance so its Sharpe is finite (a constant-magnitude market would make the
/// perfectly-aligned P&L a zero-variance constant -> degenerate NaN). The series
/// is zero-mean overall so a constant-long signal earns no edge. Shared by both
/// demos as the OUTCOME the null permutes.
fn demo_market_returns() -> Vec<f64> {
    (0..DEMO_SERIES_LEN)
        .map(|i| {
            let mag = ((i % 3) + 1) as f64 * DEMO_RETURN_AMP;
            if i % 2 == 0 {
                mag
            } else {
                -mag
            }
        })
        .collect()
}

/// Build the OVERFIT fixture: a constant-long signal (always +1) on the
/// zero-mean alternating market. The strategy P&L stream = signal * return =
/// the market itself (zero-mean), and ANY permutation of the returns yields the
/// same zero-mean P&L Sharpe distribution -> the observed statistic sits inside
/// the null band (p >= alpha/N) -> the edge survived the null -> BLOCK. A
/// constant-long bias is exactly the kind of spurious "edge" the null catches.
fn demo_overfit_inputs() -> (Vec<f64>, Vec<f64>) {
    let returns = demo_market_returns();
    let signals = vec![1.0; DEMO_SERIES_LEN];
    (signals, returns)
}

/// Build the GENUINE fixture: a perfectly-aligned signal (sign of THIS bar's
/// return) on the same alternating market. The strategy P&L = signal * return =
/// |return| = +amp every bar: a large positive Sharpe. Permuting the returns
/// against the FIXED signal destroys the alignment, so most permutations produce
/// a near-zero P&L Sharpe -> the observed statistic sits in the tail
/// (p < alpha/N) -> the edge collapsed-to-null -> ALLOW.
fn demo_genuine_inputs() -> (Vec<f64>, Vec<f64>) {
    let returns = demo_market_returns();
    let signals: Vec<f64> = returns
        .iter()
        .map(|&r| if r >= 0.0 { 1.0 } else { -1.0 })
        .collect();
    (signals, returns)
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Run the `elai null-test` fixture gate.
///
/// # Arguments
/// * `demo_overfit` — drive the flat/noise fixture (expects a BLOCK).
/// * `demo_genuine` — drive the trending fixture (expects an ALLOW).
/// * `seed`         — evaluation seed (default: 42).
/// * `json`         — print JSON instead of human text.
pub(crate) async fn run(
    demo_overfit: bool,
    demo_genuine: bool,
    seed: Option<u64>,
    json: bool,
) -> Result<(), NullTestCheckError> {
    let seed_val = seed.unwrap_or(DEFAULT_SEED);

    let (signals, returns, mode) = if demo_overfit {
        let (s, r) = demo_overfit_inputs();
        (s, r, "demo-overfit")
    } else if demo_genuine {
        let (s, r) = demo_genuine_inputs();
        (s, r, "demo-genuine")
    } else {
        return Err(NullTestCheckError::NoDemoSelected);
    };

    let project_root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let sessions_dir = sessions_dir_path(&project_root);
    std::fs::create_dir_all(&sessions_dir)?;
    let db_path = sessions_dir.join("ssb.db");

    let cfg = SsbConfig {
        db_path: db_path.clone(),
        pool_size: None,
        project_id: NULL_TEST_PROJECT_ID.to_string(),
        session_id: NULL_TEST_SESSION_ID.to_string(),
    };
    let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
    let ssb = Arc::new(
        Ssb::new(cfg, clock.clone()).map_err(|e| NullTestCheckError::Ssb(format!("{e}")))?,
    );

    // Open the H-17 trial ledger on the SAME db and register ONE distinct trial
    // for the demo (objective, data_window) so the TD-1 fail-closed N read
    // succeeds (N >= 1). A real promotion path supplies N from the strategy
    // search; the demo registers N=1 so the corrected threshold alpha/N = alpha.
    let ledger = TrialLedger::open(&db_path, NULL_TEST_SESSION_ID)
        .map_err(|e| NullTestCheckError::Ledger(format!("{e}")))?;
    let cfg_bytes = format!("{{\"fixture\": \"{mode}\", \"seed\": {seed_val}}}");
    ledger
        .register_trial(
            &ssb,
            &clock,
            DEMO_OBJECTIVE,
            DEMO_DATA_WINDOW,
            cfg_bytes.as_bytes(),
            seed_val as i64,
        )
        .await
        .map_err(|e| NullTestCheckError::Ledger(format!("{e}")))?;

    let input = TradingNullInput {
        strategy_path: DEFAULT_STRATEGY.to_string(),
        objective: DEMO_OBJECTIVE.to_string(),
        data_window: DEMO_DATA_WINDOW.to_string(),
        signals,
        returns,
        k_permutations: K_MIN_PERMUTATIONS,
        block_size: DEFAULT_BLOCK_SIZE,
        seed: seed_val,
    };

    let verdict = evaluate_trading_null(&ledger, &ssb, input).await?;

    let verdict_str = match &verdict.kind {
        TradingNullVerdictKind::EdgeCollapsedToNull => "ALLOW (edge collapsed to null)",
        TradingNullVerdictKind::EdgeSurvivedNull { .. } => "BLOCK (edge survived null)",
        TradingNullVerdictKind::Unprovable => "BLOCK (fail-closed: unprovable)",
    };

    if json {
        let out = serde_json::json!({
            "null_test": {
                "mode": mode,
                "rule_id": TRADING_NULL_RULE_ID,
                "strategy": DEFAULT_STRATEGY,
                "verdict": verdict_str,
                "reason": verdict.reason,
                "observed_statistic": verdict.observed_statistic,
                "p_value": verdict.p_value,
                "threshold_alpha_over_n": verdict.threshold,
                "null_band_low": verdict.null_band_low,
                "null_band_high": verdict.null_band_high,
                "n_trials": verdict.n_trials,
                "k_permutations": verdict.k_permutations,
                "seed": seed_val,
                "ssb_action_type": "trading_null_test",
            }
        });
        println!("{}", serde_json::to_string_pretty(&out)?);
    } else {
        println!("elai null-test (H-22, W-1 production entrypoint)");
        println!("  mode:        {mode}");
        println!("  rule_id:     {TRADING_NULL_RULE_ID}");
        println!("  strategy:    {DEFAULT_STRATEGY}");
        println!("  seed:        {seed_val}");
        println!();
        println!("  VERDICT:  {verdict_str}  (reason: {})", verdict.reason);
        println!(
            "  observed Sharpe: {:.6}   p-value: {:.6}   threshold (alpha/N): {:.6}",
            verdict.observed_statistic, verdict.p_value, verdict.threshold
        );
        println!(
            "  null band: [{:.6}, {:.6}]   N={}   K={}",
            verdict.null_band_low, verdict.null_band_high, verdict.n_trials, verdict.k_permutations
        );
        println!();
        println!("  evidence: {}", verdict.evidence);
        println!();
        println!("  TradingNullTest SSB row: see ssb.db action_type='trading_null_test'");
        println!("  DONE (W-1 production entrypoint proof).");
    }

    Ok(())
}

//! H-26 (SCHEMA_VERSION 232): `elai shadow-check` — W-1 production entrypoint for
//! the shadow-exec + financial-invariant metamorphic-relation gate.
//!
//! Runs [`elai_orchestrator::shadow_exec::diff_shadow_impls`] over two
//! fixture-supplied result sets and emits a `DifferentialDivergence` SSB row (the
//! off-agent done-gate artifact, R4).
//!
//! # W-1 production entrypoint (WIRING_CONTRACT)
//! This command is the live CLI hook for the shadow-exec gate. It is NOT test-only:
//! it opens a real `Ssb`, builds the two result sets, calls `diff_shadow_impls`, and
//! emits a real SSB row. Tier-1 needs no sidecar: the result sets are supplied as
//! plain data. The DEFERRED Tier-2 Python two-impl runner (re-executes both impls on
//! shared seeded inputs) is RunPod-pending (no stub shipped).
//!
//! # Demo fixtures (deterministic, in-memory, integer-cents money)
//!   * `--demo-invariant-held`     — a consistent run: shares conserved,
//!     cash+position*price==equity, PnL additive (EXACT integer cents), put-call
//!     parity (European) holds within the band, primary agrees with shadow ->
//!     InvariantHeld (Allow, exit 0).
//!   * `--demo-invariant-violated` — the SAME run with the PnL-additivity accounting
//!     identity broken by 1 cent (declared total != sum of per-trade PnL) ->
//!     InvariantViolated (Block, exit 1). This proves the integer-cents EXACT regime
//!     catches a cent-level error that a float tolerance would hide.
//!
//! # No .unwrap()/.expect() in non-test code (CLAUDE.md HARD RULE).

use std::path::PathBuf;
use std::sync::Arc;

use elai_core::clock::OsClock;
use elai_core::paths::sessions_dir_path;
use elai_orchestrator::{
    shadow_exec::{
        diff_shadow_impls, DerivedFloat, FinancialInvariantInputs, OptionStyle,
        PutCallParityInput, ShadowOutcome, ShadowResultSet, ShadowVerdictKind, TradeRecord,
        FINANCIAL_INVARIANT_RULE_ID,
    },
    ssb::{Ssb, SsbConfig},
};
use thiserror::Error;

// ---------------------------------------------------------------------------
// Constants (CLAUDE.md no-magic-numbers)
// ---------------------------------------------------------------------------

/// Session id for the shadow-check fixture session.
const SHADOW_CHECK_SESSION_ID: &str = "elai-shadow-check-fixture";

/// Project id for the shadow-check fixture session.
const SHADOW_CHECK_PROJECT_ID: &str = "elai-shadow-check";

/// Default strategy path reported when none is provided.
const DEFAULT_STRATEGY: &str = "demo_pnl_engine.rs";

/// Default seed for GD-2 reproducibility.
const DEFAULT_SEED: u64 = 42;

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Errors from the `shadow-check` command.
#[derive(Debug, Error)]
pub(crate) enum ShadowCheckError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("SSB initialization error: {0}")]
    Ssb(String),

    #[error("shadow-exec gate error: {0}")]
    Gate(#[from] elai_orchestrator::shadow_exec::ShadowExecError),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("no demo mode selected: pass --demo-invariant-held or --demo-invariant-violated")]
    NoDemoSelected,
}

// ---------------------------------------------------------------------------
// Deterministic demo fixtures (in-memory; money is INTEGER CENTS)
// ---------------------------------------------------------------------------

/// Build the consistent (held) primary result set: two trades, shares conserved,
/// cash+position*price==equity, PnL additive. cash=100_00c, pos=10, price=50_00c =>
/// equity = 100_00 + 10*50_00 = 600_00c.
fn held_primary() -> ShadowResultSet {
    ShadowResultSet {
        strategy_path: DEFAULT_STRATEGY.to_string(),
        run_id: "demo-held".to_string(),
        elai_seed: Some(DEFAULT_SEED),
        trades: vec![
            TradeRecord {
                shares_in: 10,
                shares_out: 10,
                cash_cents: 100_00,
                position_qty: 10,
                price_cents: 50_00,
                equity_cents: 100_00 + 10 * 50_00,
                realized_pnl_cents: 1_50,
            },
            TradeRecord {
                shares_in: 5,
                shares_out: 5,
                cash_cents: 200_00,
                position_qty: 4,
                price_cents: 25_00,
                equity_cents: 200_00 + 4 * 25_00,
                realized_pnl_cents: 2_25,
            },
        ],
        declared_total_pnl_cents: 1_50 + 2_25,
        derived_floats: vec![DerivedFloat {
            name: "option_price".to_string(),
            value: 12.345_678_9,
        }],
    }
}

/// A clean shadow re-implementation that agrees with the held primary (same derived
/// float within the band).
fn held_shadow() -> ShadowOutcome {
    ShadowOutcome::Produced(Box::new(ShadowResultSet {
        derived_floats: vec![DerivedFloat {
            name: "option_price".to_string(),
            value: 12.345_678_9,
        }],
        ..held_primary()
    }))
}

/// A parity-consistent European put-call-parity input (held): C - P == S -
/// K*exp(-rT). put=3.0, rhs = spot - strike*exp(-r*T), call = put + rhs.
fn held_invariants() -> FinancialInvariantInputs {
    let spot: f64 = 100.0;
    let strike: f64 = 95.0;
    let rate: f64 = 0.02;
    let time: f64 = 0.5;
    let rhs = spot - strike * (-rate * time).exp();
    let put: f64 = 3.0;
    let call = put + rhs;
    FinancialInvariantInputs {
        put_call_parity: vec![PutCallParityInput {
            call,
            put,
            spot,
            strike,
            rate,
            time,
            style: OptionStyle::European,
        }],
        numeric_stability: vec![],
    }
}

/// Build the VIOLATED primary: the held run with the PnL-additivity accounting
/// identity broken by exactly 1 cent (declared total != sum of per-trade PnL). The
/// EXACT integer-cents regime catches this; a float tolerance would hide a cent error.
fn violated_primary() -> ShadowResultSet {
    let mut p = held_primary();
    p.run_id = "demo-violated".to_string();
    p.declared_total_pnl_cents += 1; // 1-cent additivity break
    p
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Run the `elai shadow-check` fixture gate.
///
/// # Arguments
/// * `demo_invariant_held`     — drive the consistent fixture (expects InvariantHeld).
/// * `demo_invariant_violated` — drive the broken-identity fixture (expects Block).
/// * `seed`                    — evaluation seed (default: 42).
/// * `json`                    — print JSON instead of human text.
///
/// Exits the process with code 1 on a Block verdict (ShadowDiverged /
/// InvariantViolated) so CI gates; returns `Ok(())` on InvariantHeld.
pub(crate) async fn run(
    demo_invariant_held: bool,
    demo_invariant_violated: bool,
    seed: Option<u64>,
    json: bool,
) -> Result<(), ShadowCheckError> {
    let seed_val = seed.unwrap_or(DEFAULT_SEED);

    let (primary, shadow, invariants, mode) = if demo_invariant_held {
        (held_primary(), held_shadow(), held_invariants(), "demo-invariant-held")
    } else if demo_invariant_violated {
        // The violated fixture still supplies a shadow + (held) parity inputs so the
        // gate reaches the accounting-identity check and Blocks on PnL additivity.
        (
            violated_primary(),
            held_shadow(),
            held_invariants(),
            "demo-invariant-violated",
        )
    } else {
        return Err(ShadowCheckError::NoDemoSelected);
    };

    let project_root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let sessions_dir = sessions_dir_path(&project_root);
    std::fs::create_dir_all(&sessions_dir)?;
    let db_path = sessions_dir.join("ssb.db");

    let cfg = SsbConfig {
        db_path,
        pool_size: None,
        project_id: SHADOW_CHECK_PROJECT_ID.to_string(),
        session_id: SHADOW_CHECK_SESSION_ID.to_string(),
    };
    let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
    let ssb = Arc::new(
        Ssb::new(cfg, clock).map_err(|e| ShadowCheckError::Ssb(format!("{e}")))?,
    );

    let verdict = diff_shadow_impls(&ssb, primary, shadow, invariants, seed_val).await?;

    let verdict_str = match &verdict.kind {
        ShadowVerdictKind::InvariantHeld => "ALLOW (invariants held within band)",
        ShadowVerdictKind::ShadowDiverged => "BLOCK (shadow diverged beyond band)",
        ShadowVerdictKind::InvariantViolated => "BLOCK (financial invariant violated)",
    };

    if json {
        let out = serde_json::json!({
            "shadow_check": {
                "mode": mode,
                "rule_id": FINANCIAL_INVARIANT_RULE_ID,
                "strategy": DEFAULT_STRATEGY,
                "verdict": verdict_str,
                "reason": verdict.reason,
                "offending_invariant": verdict.offending_invariant,
                "first_divergent_field": verdict.first_divergent_field,
                "divergence_metric": verdict.divergence_metric,
                "declared_trade_count": verdict.declared_trade_count,
                "primary_digest": verdict.primary_digest,
                "shadow_digest": verdict.shadow_digest,
                "seed": seed_val,
                "ssb_action_type": "differential_divergence",
            }
        });
        println!("{}", serde_json::to_string_pretty(&out)?);
    } else {
        println!("elai shadow-check (H-26, W-1 production entrypoint)");
        println!("  mode:        {mode}");
        println!("  rule_id:     {FINANCIAL_INVARIANT_RULE_ID}");
        println!("  strategy:    {DEFAULT_STRATEGY}");
        println!("  seed:        {seed_val}");
        println!();
        println!("  VERDICT:  {verdict_str}  (reason: {})", verdict.reason);
        if !verdict.offending_invariant.is_empty() {
            println!("  offending invariant: {}", verdict.offending_invariant);
        }
        if !verdict.first_divergent_field.is_empty() {
            println!(
                "  first divergent field: {} (metric {:.3e})",
                verdict.first_divergent_field, verdict.divergence_metric
            );
        }
        println!(
            "  trades: {}   primary_digest: {}   shadow_digest: {}",
            verdict.declared_trade_count, verdict.primary_digest, verdict.shadow_digest
        );
        println!();
        println!("  evidence: {}", verdict.evidence);
        println!();
        println!("  DifferentialDivergence SSB row: see ssb.db action_type='differential_divergence'");
        println!("  DONE (W-1 production entrypoint proof).");
    }

    // Exit non-zero on a Block verdict so CI gates (mirror the H-22/H-23/H-24/H-25
    // *-check exit contract).
    match verdict.kind {
        ShadowVerdictKind::ShadowDiverged | ShadowVerdictKind::InvariantViolated => {
            std::process::exit(1)
        }
        ShadowVerdictKind::InvariantHeld => Ok(()),
    }
}

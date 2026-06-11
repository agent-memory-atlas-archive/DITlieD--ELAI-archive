//! H-19 (SCHEMA_VERSION 237): `elai holdout-check` — W-1/W-3 production entrypoint for the
//! reusable-holdout query-budget gate (the Thresholdout DP gate, Dwork et al. arXiv:1506.02629).
//!
//! Drives [`elai_orchestrator::holdout_budget::HoldoutBudgetLedger::query_holdout`] over a
//! planted fixture holdout and emits `HoldoutBudgetEvent` SSB rows (the off-agent done-gate
//! artifact, R4).
//!
//! # W-1 production entrypoint (Tier-1, always-compiled)
//! Not test-only: it opens a real `Ssb`, provisions a fixture holdout out-of-band, drives
//! `query_holdout`, and emits real SSB rows. The live OOS-eval choke-point (the
//! autonomous-search `query_holdout` route) is a DEFERRED Tier-2 registered stub; wiring the
//! budget gate ONLY there would ship it WIRE-DARK in the default build. Tier-1 is therefore
//! this always-compiled CLI over a fixture.
//!
//! # Demo fixtures
//!   * `--demo-spend`     — provisions a fresh holdout, then runs ONE above-threshold OOS
//!     query (a large train/holdout gap). The gate CHARGES one budget unit (`QuerySpent`) and
//!     returns the Laplace-noised holdout statistic. Exit 0.
//!   * `--demo-exhausted` — provisions a holdout with budget B=1, spends the single unit, then
//!     runs ONE MORE above-threshold query. The budget is spent => the query is REFUSED
//!     (`Exhausted`, budget_exhausted, fail-closed). Exit 1.
//!
//! # No .unwrap()/.expect() in non-test code (CLAUDE.md HARD RULE). blake3 only.

use std::path::PathBuf;
use std::sync::Arc;

use elai_core::clock::OsClock;
use elai_core::paths::sessions_dir_path;
use elai_orchestrator::{
    holdout_budget::{provision_fresh_holdout, HoldoutBudgetLedger, HOLDOUT_BUDGET_TOTAL},
    ssb::{HoldoutBudgetKind, Ssb, SsbConfig},
};
use thiserror::Error;

// ---------------------------------------------------------------------------
// Constants (CLAUDE.md no-magic-numbers)
// ---------------------------------------------------------------------------

/// Session id for the holdout-check fixture session.
const HOLDOUT_CHECK_SESSION_ID: &str = "elai-holdout-check-fixture";

/// Project id for the holdout-check fixture session.
const HOLDOUT_CHECK_PROJECT_ID: &str = "elai-holdout-check";

/// Run id stamped on the emitted HoldoutBudgetEvent rows.
const HOLDOUT_CHECK_RUN_ID: &str = "holdout-check-demo";

/// Default seed for GD-2 reproducibility.
const DEFAULT_SEED: u64 = 42;

/// The planted fixture holdout id.
const DEMO_HOLDOUT_ID: &str = "sharpe_1y_holdout";

/// The planted fixture objective.
const DEMO_OBJECTIVE: &str = "sharpe_1y";

/// The planted fixture holdout data window (canonical "start_ms..end_ms").
const DEMO_DATA_WINDOW: &str = "1704067200000..1735689600000";

/// The planted training-set statistic (E_St[phi]) for the demo query.
const DEMO_TRAIN_STAT: f64 = 0.10;

/// The planted holdout-set statistic (E_Sh[phi]) for the demo query. A large gap vs
/// DEMO_TRAIN_STAT (0.80) is above the Thresholdout threshold so the query charges the budget.
const DEMO_HOLDOUT_STAT: f64 = 0.90;

/// The budget total for the `--demo-exhausted` fixture (B=1 so a single spend exhausts it).
const DEMO_EXHAUSTED_BUDGET: u64 = 1;

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Errors from the `holdout-check` command.
#[derive(Debug, Error)]
pub(crate) enum HoldoutCheckError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("SSB initialization error: {0}")]
    Ssb(String),

    #[error("holdout-budget error: {0}")]
    Holdout(#[from] elai_orchestrator::holdout_budget::HoldoutBudgetError),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("no demo mode selected: pass --demo-spend or --demo-exhausted")]
    NoDemoSelected,
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Run the `elai holdout-check` reusable-holdout query-budget gate.
///
/// # Arguments
/// * `demo_spend`     — provision + one above-threshold query (expects QuerySpent, exit 0).
/// * `demo_exhausted` — provision B=1, spend it, then one more query (expects Exhausted, exit 1).
/// * `seed`           — evaluation seed (default: 42).
/// * `json`           — print JSON instead of human text.
///
/// Exits the process with code 1 when the final query is REFUSED (Exhausted); returns `Ok(())`
/// on a successful spend.
pub(crate) async fn run(
    demo_spend: bool,
    demo_exhausted: bool,
    seed: Option<u64>,
    json: bool,
) -> Result<(), HoldoutCheckError> {
    let seed_val = seed.unwrap_or(DEFAULT_SEED);

    let mode = if demo_spend {
        "demo-spend"
    } else if demo_exhausted {
        "demo-exhausted"
    } else {
        return Err(HoldoutCheckError::NoDemoSelected);
    };

    let project_root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let sessions_dir = sessions_dir_path(&project_root);
    std::fs::create_dir_all(&sessions_dir)?;
    let db_path = sessions_dir.join("ssb.db");

    let cfg = SsbConfig {
        db_path: db_path.clone(),
        pool_size: None,
        project_id: HOLDOUT_CHECK_PROJECT_ID.to_string(),
        session_id: HOLDOUT_CHECK_SESSION_ID.to_string(),
    };
    let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
    let ssb = Arc::new(Ssb::new(cfg, clock.clone()).map_err(|e| HoldoutCheckError::Ssb(format!("{e}")))?);

    let ledger = HoldoutBudgetLedger::open(&db_path, HOLDOUT_CHECK_SESSION_ID)?;

    // A per-run unique holdout id so repeated CLI invocations do not collide on the persisted
    // counter (each invocation provisions its own fresh fixture). blake3 of the seed keeps it
    // deterministic per (seed, mode).
    let holdout_id = format!(
        "{DEMO_HOLDOUT_ID}-{mode}-{}",
        &blake3::hash(format!("{mode}:{seed_val}").as_bytes()).to_hex().to_string()[..16]
    );

    let (budget, final_kind, noisy) = if demo_spend {
        // Provision the full budget, then one above-threshold query => QuerySpent.
        provision_fresh_holdout(
            &ledger,
            &ssb,
            &clock,
            &holdout_id,
            DEMO_OBJECTIVE,
            DEMO_DATA_WINDOW,
            HOLDOUT_BUDGET_TOTAL,
            seed_val,
            HOLDOUT_CHECK_RUN_ID,
        )
        .await?;
        let out = ledger
            .query_holdout(
                &ssb,
                &clock,
                &holdout_id,
                DEMO_OBJECTIVE,
                DEMO_DATA_WINDOW,
                DEMO_TRAIN_STAT,
                DEMO_HOLDOUT_STAT,
                HOLDOUT_CHECK_RUN_ID,
                seed_val,
            )
            .await?;
        (HOLDOUT_BUDGET_TOTAL, out.kind, out.answer)
    } else {
        // Provision B=1, spend the single unit, then one more above-threshold query => Exhausted.
        provision_fresh_holdout(
            &ledger,
            &ssb,
            &clock,
            &holdout_id,
            DEMO_OBJECTIVE,
            DEMO_DATA_WINDOW,
            DEMO_EXHAUSTED_BUDGET,
            seed_val,
            HOLDOUT_CHECK_RUN_ID,
        )
        .await?;
        // Spend the single unit.
        let _ = ledger
            .query_holdout(
                &ssb,
                &clock,
                &holdout_id,
                DEMO_OBJECTIVE,
                DEMO_DATA_WINDOW,
                DEMO_TRAIN_STAT,
                DEMO_HOLDOUT_STAT,
                HOLDOUT_CHECK_RUN_ID,
                seed_val,
            )
            .await?;
        // The next above-threshold query must be REFUSED (budget exhausted).
        let out = ledger
            .query_holdout(
                &ssb,
                &clock,
                &holdout_id,
                DEMO_OBJECTIVE,
                DEMO_DATA_WINDOW,
                DEMO_TRAIN_STAT,
                DEMO_HOLDOUT_STAT,
                HOLDOUT_CHECK_RUN_ID,
                seed_val,
            )
            .await?;
        (DEMO_EXHAUSTED_BUDGET, out.kind, out.answer)
    };

    let verdict_str = match final_kind {
        HoldoutBudgetKind::QuerySpent => {
            "SPENT (above-threshold OOS query charged 1 budget unit; released the \
             Laplace-noised holdout statistic)"
        }
        HoldoutBudgetKind::Exhausted => {
            "REFUSE (budget exhausted — the holdout is spent; fail-closed budget_exhausted. \
             A fresh holdout must be provisioned out-of-band; the same series cannot be reused)"
        }
        HoldoutBudgetKind::Provisioned => "PROVISIONED",
        HoldoutBudgetKind::Running => "RUNNING",
    };

    if json {
        let out = serde_json::json!({
            "holdout_check": {
                "mode": mode,
                "verdict": verdict_str,
                "kind": format!("{final_kind:?}"),
                "holdout_id": holdout_id,
                "objective": DEMO_OBJECTIVE,
                "budget_total": budget,
                "noisy_statistic": noisy,
                "seed": seed_val,
                "ssb_action_type": "holdout_budget_event",
            }
        });
        println!("{}", serde_json::to_string_pretty(&out)?);
    } else {
        println!("elai holdout-check (H-19, W-1 production entrypoint)");
        println!("  mode:         {mode}");
        println!("  seed:         {seed_val}");
        println!("  budget_total: {budget}");
        println!();
        println!("  VERDICT:  {verdict_str}");
        if let Some(stat) = noisy {
            println!("  noised holdout statistic: {stat:.6}");
        }
        println!();
        println!("  HoldoutBudgetEvent SSB row: see ssb.db action_type='holdout_budget_event'");
        println!("  DONE (W-1 production entrypoint proof).");
    }

    // Exit non-zero when the final query was REFUSED (Exhausted) so CI gates (mirror the
    // H-22..H-28 *-check exit contract).
    if final_kind == HoldoutBudgetKind::Exhausted {
        std::process::exit(1)
    }
    Ok(())
}

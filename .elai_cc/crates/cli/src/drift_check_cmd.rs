//! H-29 (SCHEMA_VERSION 238): `elai drift-check` — W-1/W-3 production entrypoint for the LOCKED
//! fail-closed drift state machine (T-13), the master deployment-trust interlock.
//!
//! Drives [`elai_orchestrator::drift_interlock::evaluate_drift_interlock`] over a planted
//! fixture strategy and emits `DriftInterlock` SSB rows (the off-agent done-gate artifact, R4).
//!
//! # W-1 production entrypoint (Tier-1, always-compiled)
//! Not test-only: it opens a real `Ssb` + a `DriftInterlockLedger`, plants fixture drift
//! evidence, drives `evaluate_drift_interlock`, and emits real SSB rows. The live-trading
//! resume choke-point (the FSM edge the design names) does NOT exist as a real transition yet —
//! it is a DEFERRED Tier-2 forward dependency. Tier-1 is therefore this always-compiled CLI
//! over a fixture (mirrors H-18 `deflation-check` / H-19 `holdout-check`).
//!
//! # Demo fixtures
//!   * `--demo-stable` — plants a clean deployment (identical input/output distributions, a
//!     matching schema fingerprint, a tiny deviation, a fresh validation). No drift on any axis
//!     => `Running`. Exit 0.
//!   * `--demo-drift`  — plants a vendor-schema change + an output-PSI breach + a 14d-deviation
//!     breach. ANY axis breach => the interlock latches `DriftHalted`. Exit 1.
//!
//! Each invocation uses a per-(mode, seed) unique strategy id so the persistent latch from a
//! `--demo-drift` run does not poison a later `--demo-stable` run on the same db.
//!
//! # No .unwrap()/.expect() in non-test code (CLAUDE.md HARD RULE). blake3 only.

use std::path::PathBuf;
use std::sync::Arc;

use elai_core::clock::OsClock;
use elai_core::paths::sessions_dir_path;
use elai_orchestrator::{
    drift_interlock::{
        evaluate_drift_interlock, DriftEvidence, DriftInterlockLedger, PsiBins,
        DEVIATION_HALT_BAND, DEVIATION_WINDOW_DAYS,
    },
    ssb::{DriftInterlockKind, Ssb, SsbConfig},
};
use thiserror::Error;

// ---------------------------------------------------------------------------
// Constants (CLAUDE.md no-magic-numbers)
// ---------------------------------------------------------------------------

/// Session id for the drift-check fixture session.
const DRIFT_CHECK_SESSION_ID: &str = "elai-drift-check-fixture";

/// Project id for the drift-check fixture session.
const DRIFT_CHECK_PROJECT_ID: &str = "elai-drift-check";

/// Run id stamped on the emitted DriftInterlock rows.
const DRIFT_CHECK_RUN_ID: &str = "drift-check-demo";

/// Default seed for GD-2 reproducibility.
const DEFAULT_SEED: u64 = 42;

/// The base fixture strategy id (a per-(mode, seed) suffix is appended).
const DEMO_STRATEGY_BASE: &str = "momentum_eurusd_v3";

/// The fixture live vendor data schema (the bytes the interlock fingerprints with blake3).
const DEMO_LIVE_SCHEMA: &[u8] = b"col_a:f64,col_b:f64,signal:f64";

/// A DIFFERENT vendor schema used by `--demo-drift` to force a fingerprint mismatch vs the
/// frozen baseline (a vendor silently added a column).
const DEMO_DRIFTED_SCHEMA: &[u8] = b"col_a:f64,col_b:f64,col_c:f64,signal:f64";

/// Milliseconds per day, used to age the `--demo-drift` deviation measurement past the window
/// and to keep the `--demo-stable` validation fresh.
const MS_PER_DAY: i64 = 86_400_000;

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Errors from the `drift-check` command.
#[derive(Debug, Error)]
pub(crate) enum DriftCheckError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("SSB initialization error: {0}")]
    Ssb(String),

    #[error("drift-interlock error: {0}")]
    Drift(#[from] elai_orchestrator::drift_interlock::DriftInterlockError),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("no demo mode selected: pass --demo-stable or --demo-drift")]
    NoDemoSelected,
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Run the `elai drift-check` LOCKED fail-closed drift state machine over a planted fixture.
///
/// # Arguments
/// * `demo_stable` — plant a clean deployment; expects `Running` (exit 0).
/// * `demo_drift`  — plant a schema change + PSI/deviation breach; expects `DriftHalted` (exit 1).
/// * `seed`        — evaluation seed (default: 42).
/// * `json`        — print JSON instead of human text.
///
/// Exits the process with code 1 when the interlock latches `DriftHalted`; returns `Ok(())` on a
/// clean `Running` verdict.
pub(crate) async fn run(
    demo_stable: bool,
    demo_drift: bool,
    seed: Option<u64>,
    json: bool,
) -> Result<(), DriftCheckError> {
    let seed_val = seed.unwrap_or(DEFAULT_SEED);

    let mode = if demo_stable {
        "demo-stable"
    } else if demo_drift {
        "demo-drift"
    } else {
        return Err(DriftCheckError::NoDemoSelected);
    };

    let project_root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let sessions_dir = sessions_dir_path(&project_root);
    std::fs::create_dir_all(&sessions_dir)?;
    let db_path = sessions_dir.join("ssb.db");

    let cfg = SsbConfig {
        db_path: db_path.clone(),
        pool_size: None,
        project_id: DRIFT_CHECK_PROJECT_ID.to_string(),
        session_id: DRIFT_CHECK_SESSION_ID.to_string(),
    };
    let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
    let ssb = Arc::new(
        Ssb::new(cfg, clock.clone()).map_err(|e| DriftCheckError::Ssb(format!("{e}")))?,
    );

    let ledger = DriftInterlockLedger::open(&db_path, DRIFT_CHECK_SESSION_ID)?;

    // A per-(mode, seed) unique strategy id so the persistent latch from a prior --demo-drift
    // run does not poison a later --demo-stable run on the same db. blake3 keeps it deterministic.
    let strategy_id = format!(
        "{DEMO_STRATEGY_BASE}-{mode}-{}",
        &blake3::hash(format!("{mode}:{seed_val}").as_bytes())
            .to_hex()
            .to_string()[..16]
    );

    let now_ms = Ssb::now_ms_via_clock(&clock);
    let frozen_fp = blake3::hash(DEMO_LIVE_SCHEMA).to_hex().to_string();

    // Build the fixture evidence per mode.
    let evidence = if demo_stable {
        // A clean deployment: identical distributions, matching fingerprint, tiny deviation,
        // a fresh validation. Resolves to Running.
        DriftEvidence {
            input_distribution: Some(PsiBins {
                expected: vec![0.25, 0.25, 0.25, 0.25],
                actual: vec![0.25, 0.25, 0.25, 0.25],
            }),
            output_distribution: Some(PsiBins {
                expected: vec![0.5, 0.5],
                actual: vec![0.5, 0.5],
            }),
            baseline_schema_fingerprint: Some(frozen_fp.clone()),
            live_schema: Some(DEMO_LIVE_SCHEMA.to_vec()),
            live_vs_backtest_deviation: Some(0.01),
            deviation_measured_at_ms: Some(now_ms),
            last_validated_at_ms: Some(now_ms),
            composed_breaches: Vec::new(),
        }
    } else {
        // A drifted deployment: a vendor-schema change (fingerprint mismatch), an output-PSI
        // breach (a large mass shift over the 0.25 halt band), and a 14d-deviation breach (a
        // deviation past the band). ANY of these latches DriftHalted.
        DriftEvidence {
            input_distribution: Some(PsiBins {
                expected: vec![0.25, 0.25, 0.25, 0.25],
                actual: vec![0.25, 0.25, 0.25, 0.25],
            }),
            // A large output-distribution shift: PSI well over PSI_HALT_BAND.
            output_distribution: Some(PsiBins {
                expected: vec![0.5, 0.5],
                actual: vec![0.05, 0.95],
            }),
            baseline_schema_fingerprint: Some(frozen_fp.clone()),
            // The LIVE schema differs from the frozen baseline => fingerprint mismatch.
            live_schema: Some(DEMO_DRIFTED_SCHEMA.to_vec()),
            // A deviation past the band, measured within the window.
            live_vs_backtest_deviation: Some(DEVIATION_HALT_BAND + 0.75),
            deviation_measured_at_ms: Some(
                now_ms - (i64::try_from(DEVIATION_WINDOW_DAYS).unwrap_or(0) / 2) * MS_PER_DAY,
            ),
            last_validated_at_ms: Some(now_ms),
            composed_breaches: Vec::new(),
        }
    };

    let outcome = evaluate_drift_interlock(
        &ledger,
        &ssb,
        &clock,
        &strategy_id,
        &evidence,
        DRIFT_CHECK_RUN_ID,
        seed_val,
    )
    .await?;

    let kind = outcome.verdict.to_kind();
    let axes = outcome.verdict.axes().join(",");
    let verdict_str = match kind {
        DriftInterlockKind::Running => {
            "RUNNING (no drift on any axis — the live-trading resume edge is permitted)"
        }
        DriftInterlockKind::Warning => {
            "WARNING (a minor PSI shift surfaced; not latched — resume still permitted)"
        }
        DriftInterlockKind::DriftHalted => {
            "HALT (a drift breach OR missing evidence latched the interlock — the resume edge is \
             BLOCKED until a human/frozen fresh-evidence unlock; the same series cannot be reused)"
        }
        DriftInterlockKind::HumanUnlocked => "UNLOCKED (a human/frozen fresh-evidence unlock)",
    };

    if json {
        let out = serde_json::json!({
            "drift_check": {
                "mode": mode,
                "verdict": verdict_str,
                "kind": kind.label(),
                "strategy_id": strategy_id,
                "breached_axes": axes,
                "latched_halted": outcome.latched_halted,
                "seed": seed_val,
                "ssb_action_type": "drift_interlock",
            }
        });
        println!("{}", serde_json::to_string_pretty(&out)?);
    } else {
        println!("elai drift-check (H-29, W-1 production entrypoint)");
        println!("  mode:        {mode}");
        println!("  seed:        {seed_val}");
        println!("  strategy_id: {strategy_id}");
        if !axes.is_empty() {
            println!("  breached:    {axes}");
        }
        println!();
        println!("  VERDICT:  {verdict_str}");
        println!();
        println!("  DriftInterlock SSB row: see ssb.db action_type='drift_interlock'");
        println!("  DONE (W-1 production entrypoint proof).");
    }

    // Exit non-zero when the interlock latched HALTED so CI gates can assert it (mirror the
    // H-22..H-28 *-check exit contract).
    if outcome.latched_halted {
        std::process::exit(1)
    }
    Ok(())
}

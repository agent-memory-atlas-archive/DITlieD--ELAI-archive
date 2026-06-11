//! H-27 (SCHEMA_VERSION 235): `elai gate-selfcheck` — W-1/W-3 production entrypoint for
//! the verify-the-verifiers META gate.
//!
//! Drives [`elai_orchestrator::gate_self_validation::validate_gates`] over the FROZEN
//! registry of the 8 Track-3 gates and emits a `GateSelfValidation` SSB row (the
//! off-agent done-gate artifact, R4).
//!
//! # W-1 production entrypoint (WIRING_CONTRACT)
//! Not test-only: it opens a real `Ssb`, calls `validate_gates`, and emits a real SSB
//! row. Each inner gate is driven with its OWN demo-violated fixture (the planted
//! mutant) through the gate's REAL async `evaluate_*` fn; a gate that fails to BLOCK its
//! violation (a surviving mutant) makes the meta-gate Block and exit non-zero.
//!
//! # Demo fixtures
//!   * `--demo-all-valid`   — all 8 gates kill their planted mutants and do not over-
//!     block their clean fixtures => `AllGatesValid` (PASS, exit 0).
//!   * `--demo-gate-broken` — a synthetic WEAKENED gate is injected whose violation
//!     fixture is NOT blocked (a fail-open gate) => `GateFailedToBlock` (BLOCK, exit 1),
//!     naming the broken gate. This proves the meta-gate surfaces a surviving mutant.
//!
//! # No .unwrap()/.expect() in non-test code (CLAUDE.md HARD RULE).

use std::path::PathBuf;
use std::sync::Arc;

use elai_core::clock::OsClock;
use elai_core::paths::sessions_dir_path;
use elai_orchestrator::{
    gate_self_validation::validate_gates,
    ssb::{GateSelfValidationKind, Ssb, SsbConfig},
};
use thiserror::Error;

// ---------------------------------------------------------------------------
// Constants (CLAUDE.md no-magic-numbers)
// ---------------------------------------------------------------------------

/// Session id for the gate-selfcheck fixture session.
const GATE_SELFCHECK_SESSION_ID: &str = "elai-gate-selfcheck-fixture";

/// Project id for the gate-selfcheck fixture session.
const GATE_SELFCHECK_PROJECT_ID: &str = "elai-gate-selfcheck";

/// Run id stamped on the emitted GateSelfValidation row.
const GATE_SELFCHECK_RUN_ID: &str = "gate-selfcheck-demo";

/// Default seed for GD-2 reproducibility.
const DEFAULT_SEED: u64 = 42;

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Errors from the `gate-selfcheck` command.
#[derive(Debug, Error)]
pub(crate) enum GateSelfcheckError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("SSB initialization error: {0}")]
    Ssb(String),

    #[error("gate self-validation error: {0}")]
    Gate(#[from] elai_orchestrator::gate_self_validation::GateSelfValidationError),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("no demo mode selected: pass --demo-all-valid or --demo-gate-broken")]
    NoDemoSelected,
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Run the `elai gate-selfcheck` META gate.
///
/// # Arguments
/// * `demo_all_valid`   — drive all 8 gates (expects AllGatesValid PASS).
/// * `demo_gate_broken` — inject a weakened gate (expects GateFailedToBlock Block).
/// * `seed`             — evaluation seed (default: 42).
/// * `json`             — print JSON instead of human text.
///
/// Exits the process with code 1 on any fail-closed Block verdict so CI gates; returns
/// `Ok(())` on AllGatesValid.
pub(crate) async fn run(
    demo_all_valid: bool,
    demo_gate_broken: bool,
    seed: Option<u64>,
    json: bool,
) -> Result<(), GateSelfcheckError> {
    let seed_val = seed.unwrap_or(DEFAULT_SEED);

    let (inject_weakened, mode) = if demo_all_valid {
        (false, "demo-all-valid")
    } else if demo_gate_broken {
        (true, "demo-gate-broken")
    } else {
        return Err(GateSelfcheckError::NoDemoSelected);
    };

    let project_root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let sessions_dir = sessions_dir_path(&project_root);
    std::fs::create_dir_all(&sessions_dir)?;
    let db_path = sessions_dir.join("ssb.db");

    let cfg = SsbConfig {
        db_path: db_path.clone(),
        pool_size: None,
        project_id: GATE_SELFCHECK_PROJECT_ID.to_string(),
        session_id: GATE_SELFCHECK_SESSION_ID.to_string(),
    };
    let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
    let ssb = Arc::new(
        Ssb::new(cfg, clock).map_err(|e| GateSelfcheckError::Ssb(format!("{e}")))?,
    );

    let outcome = validate_gates(
        &ssb,
        &db_path,
        GATE_SELFCHECK_RUN_ID,
        seed_val,
        inject_weakened,
    )
    .await?;

    let verdict_str = match outcome.kind {
        GateSelfValidationKind::AllGatesValid => "ALLOW (all gates killed their mutants)",
        GateSelfValidationKind::GateFailedToBlock => {
            "BLOCK (a gate failed to block its violation — fail-open)"
        }
        GateSelfValidationKind::GateOverBlocked => {
            "BLOCK (a gate over-blocked its clean fixture)"
        }
        GateSelfValidationKind::GateUnreachable => "BLOCK (a gate was un-runnable)",
        GateSelfValidationKind::EmptyRegistry => "BLOCK (empty registry — R1 fail-closed)",
    };

    if json {
        let out = serde_json::json!({
            "gate_selfcheck": {
                "mode": mode,
                "verdict": verdict_str,
                "gates_total": outcome.gates_total,
                "gates_killed": outcome.gates_killed,
                "mutation_score_pct": outcome.mutation_score_pct,
                "escaped_gate": outcome.escaped_gate,
                "escaped_fixture": outcome.escaped_fixture,
                "seed": seed_val,
                "ssb_action_type": "gate_self_validation",
            }
        });
        println!("{}", serde_json::to_string_pretty(&out)?);
    } else {
        println!("elai gate-selfcheck (H-27, W-1 production entrypoint)");
        println!("  mode:        {mode}");
        println!("  seed:        {seed_val}");
        println!();
        println!("  VERDICT:  {verdict_str}");
        println!(
            "  gates: {}/{} killed   mutation_score: {:.1}%",
            outcome.gates_killed, outcome.gates_total, outcome.mutation_score_pct
        );
        if !outcome.escaped_gate.is_empty() {
            println!(
                "  offending gate: {} (fixture: {})",
                outcome.escaped_gate, outcome.escaped_fixture
            );
        }
        println!();
        println!("  GateSelfValidation SSB row: see ssb.db action_type='gate_self_validation'");
        println!("  DONE (W-1 production entrypoint proof).");
    }

    // Exit non-zero on any fail-closed Block so CI gates (mirror the H-22..H-26
    // *-check exit contract).
    if outcome.is_pass() {
        Ok(())
    } else {
        std::process::exit(1)
    }
}

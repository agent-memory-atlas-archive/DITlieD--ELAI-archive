//! H-21 (SCHEMA_VERSION 226): `elai lookahead-check` — W-1 production entrypoint
//! for the runtime look-ahead / causality DIFFERENTIAL gate.
//!
//! Runs `lookahead_differential::diff_future_truncation` over two feature
//! column-sets (a FULL-data run and a FUTURE-TRUNCATED run) and emits a
//! `LookaheadDifferential` SSB row (the off-agent done-gate artifact, R4).
//!
//! # W-1 production entrypoint (WIRING_CONTRACT)
//! This command is the live CLI hook for the look-ahead differential gate. It is
//! NOT test-only: it opens a real `Ssb`, builds a `LookaheadInput`, calls
//! `diff_future_truncation`, and emits a real SSB row. Tier-1 needs no sidecar:
//! the two column-sets are supplied as plain data. The DEFERRED Tier-2 Python
//! feature-runner (re-runs a real strategy twice) is RunPod-pending.
//!
//! # Demo fixtures (deterministic, in-memory)
//!   * `--demo-future-leak` — a column built from a forward-shifted (iloc[t+1])
//!     series that DIVERGES under truncation -> BLOCK (FutureLeak).
//!   * `--demo-causal`      — a backward-looking-only pipeline that diffs to
//!     zero -> ALLOW (Causal).
//!
//! # No .unwrap()/.expect() in non-test code (CLAUDE.md HARD RULE).

use std::path::PathBuf;
use std::sync::Arc;

use elai_core::clock::OsClock;
use elai_core::paths::sessions_dir_path;
use elai_orchestrator::{
    lookahead_differential::{
        diff_future_truncation, ColumnSet, FpTolerance, LookaheadInput, LookaheadVerdictKind,
        LOOKAHEAD_RULE_ID,
    },
    ssb::{Ssb, SsbConfig},
};
use thiserror::Error;

// ---------------------------------------------------------------------------
// Constants (CLAUDE.md no-magic-numbers)
// ---------------------------------------------------------------------------

/// Session id for the lookahead-check fixture session.
const LOOKAHEAD_CHECK_SESSION_ID: &str = "elai-lookahead-check-fixture";

/// Project id for the lookahead-check fixture session.
const LOOKAHEAD_CHECK_PROJECT_ID: &str = "elai-lookahead-check";

/// Default strategy path reported when none is provided.
const DEFAULT_STRATEGY: &str = "demo_strategy.py";

/// Default seed for GD-2 reproducibility.
const DEFAULT_SEED: u64 = 42;

/// A causal (backward-looking) base series the demo fixtures derive from.
const DEMO_BASE_SERIES: [f64; 5] = [10.0, 11.0, 12.0, 13.0, 14.0];

/// The "peeked future" sentinel the FULL run reads at the boundary row of the
/// leak column. It exists ONLY because the future was present; the truncated
/// run cannot reproduce it (it falls back to the backward value), so the leak
/// column diverges at the boundary => FutureLeak. Distinct from any base value.
const DEMO_PEEKED_FUTURE: f64 = 99.0;

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Errors from the `lookahead-check` command.
#[derive(Debug, Error)]
pub(crate) enum LookaheadCheckError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("SSB initialization error: {0}")]
    Ssb(String),

    #[error("lookahead gate error: {0}")]
    Gate(#[from] elai_orchestrator::lookahead_differential::LookaheadError),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("no demo mode selected: pass --demo-future-leak or --demo-causal")]
    NoDemoSelected,
}

// ---------------------------------------------------------------------------
// Deterministic demo fixtures (in-memory column-sets)
// ---------------------------------------------------------------------------

/// Build the FUTURE-LEAK fixture: a `sma_causal` column (backward-looking) plus
/// a `next_close_leak` column built from `iloc[t+1]` of the base series. When
/// the future row is truncated, the LAST entry of the leak column changes (it
/// can no longer read the future), so the diff fires a FutureLeak block.
fn demo_future_leak_inputs() -> (ColumnSet, ColumnSet) {
    let base = DEMO_BASE_SERIES.to_vec();
    let n = base.len();

    // Backward-looking column (identical in both runs).
    let sma_causal: Vec<f64> = base.clone();

    // iloc[t+1] leak. On the FULL run, value at row t reads base[t+1]; at the
    // boundary row (t=n-1) it peeks ONE STEP BEYOND THE WINDOW, reading the
    // DEMO_PEEKED_FUTURE sentinel that exists only because the future was present.
    let mut leak_full: Vec<f64> = Vec::with_capacity(n);
    for t in 0..n {
        if t + 1 < n {
            leak_full.push(base[t + 1]);
        } else {
            leak_full.push(DEMO_PEEKED_FUTURE);
        }
    }
    // On the TRUNCATED run the future is removed, so the boundary row can no
    // longer read the peeked future and falls back to the backward value
    // (base[t]). The leak column DIFFERS at row n-1 (sentinel vs backward) =>
    // FutureLeak. The interior rows still match (the overlap is causal there).
    let mut leak_truncated: Vec<f64> = Vec::with_capacity(n);
    for t in 0..n {
        if t + 1 < n {
            leak_truncated.push(base[t + 1]);
        } else {
            leak_truncated.push(base[t]);
        }
    }

    let full = ColumnSet::from_pairs(vec![
        ("sma_causal".to_string(), sma_causal.clone()),
        ("next_close_leak".to_string(), leak_full),
    ]);
    let truncated = ColumnSet::from_pairs(vec![
        ("sma_causal".to_string(), sma_causal),
        ("next_close_leak".to_string(), leak_truncated),
    ]);
    (full, truncated)
}

/// Build the CAUSAL fixture: every column is backward-looking only, so the
/// full and future-truncated runs are bit-identical over the overlapping
/// region -> diff to zero -> Causal Allow.
fn demo_causal_inputs() -> (ColumnSet, ColumnSet) {
    let base = DEMO_BASE_SERIES.to_vec();
    // A trailing moving average (uses only past+present): identical in both runs.
    let mut sma: Vec<f64> = Vec::with_capacity(base.len());
    let mut acc = 0.0;
    for (i, &v) in base.iter().enumerate() {
        acc += v;
        sma.push(acc / (i as f64 + 1.0));
    }
    let full = ColumnSet::from_pairs(vec![
        ("close".to_string(), base.clone()),
        ("trailing_sma".to_string(), sma.clone()),
    ]);
    let truncated = ColumnSet::from_pairs(vec![
        ("close".to_string(), base),
        ("trailing_sma".to_string(), sma),
    ]);
    (full, truncated)
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Run the `elai lookahead-check` fixture gate.
///
/// # Arguments
/// * `demo_future_leak` — drive the iloc[t+1] fixture (expects a BLOCK).
/// * `demo_causal`      — drive the backward-looking fixture (expects an ALLOW).
/// * `seed`             — evaluation seed (default: 42).
/// * `json`             — print JSON instead of human text.
pub(crate) async fn run(
    demo_future_leak: bool,
    demo_causal: bool,
    seed: Option<u64>,
    json: bool,
) -> Result<(), LookaheadCheckError> {
    let seed_val = seed.unwrap_or(DEFAULT_SEED);

    let (full, truncated, mode) = if demo_future_leak {
        let (f, t) = demo_future_leak_inputs();
        (f, t, "demo-future-leak")
    } else if demo_causal {
        let (f, t) = demo_causal_inputs();
        (f, t, "demo-causal")
    } else {
        return Err(LookaheadCheckError::NoDemoSelected);
    };

    let project_root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let sessions_dir = sessions_dir_path(&project_root);
    std::fs::create_dir_all(&sessions_dir)?;
    let db_path = sessions_dir.join("ssb.db");

    let cfg = SsbConfig {
        db_path,
        pool_size: None,
        project_id: LOOKAHEAD_CHECK_PROJECT_ID.to_string(),
        session_id: LOOKAHEAD_CHECK_SESSION_ID.to_string(),
    };
    let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
    let ssb = Arc::new(Ssb::new(cfg, clock).map_err(|e| LookaheadCheckError::Ssb(format!("{e}")))?);

    let input = LookaheadInput {
        strategy_path: DEFAULT_STRATEGY.to_string(),
        full_run: full,
        truncated_run: Some(truncated),
        tolerance: FpTolerance::default(),
        seed: seed_val,
    };

    let verdict = diff_future_truncation(&ssb, input).await?;

    let verdict_str = match &verdict.kind {
        LookaheadVerdictKind::Causal => "ALLOW (causal)",
        LookaheadVerdictKind::FutureLeak(_) => "BLOCK (future leak)",
        LookaheadVerdictKind::Unprovable => "BLOCK (fail-closed: unprovable)",
    };

    if json {
        let out = serde_json::json!({
            "lookahead_check": {
                "mode": mode,
                "rule_id": LOOKAHEAD_RULE_ID,
                "strategy": DEFAULT_STRATEGY,
                "verdict": verdict_str,
                "reason": verdict.reason,
                "kind": format!("{:?}", verdict.kind),
                "first_change": verdict.first_change.as_ref().map(|fc| {
                    serde_json::json!({
                        "column": fc.column,
                        "index": fc.index,
                        "full_value": fc.full_value,
                        "truncated_value": fc.truncated_value,
                        "abs_diff": fc.abs_diff,
                        "rel_diff": fc.rel_diff,
                    })
                }),
                "tolerance": verdict.tolerance,
                "seed": seed_val,
                "ssb_action_type": "lookahead_differential",
            }
        });
        println!("{}", serde_json::to_string_pretty(&out)?);
    } else {
        println!("elai lookahead-check (H-21, W-1 production entrypoint)");
        println!("  mode:      {mode}");
        println!("  rule_id:   {LOOKAHEAD_RULE_ID}");
        println!("  strategy:  {DEFAULT_STRATEGY}");
        println!("  tolerance: {}", verdict.tolerance);
        println!("  seed:      {seed_val}");
        println!();
        println!("  VERDICT:  {verdict_str}  (reason: {})", verdict.reason);
        if let Some(fc) = &verdict.first_change {
            println!(
                "  first change: column '{}' row {} (full={}, truncated={}, abs={}, rel={})",
                fc.column, fc.index, fc.full_value, fc.truncated_value, fc.abs_diff, fc.rel_diff
            );
        }
        println!();
        println!("  evidence: {}", verdict.evidence);
        println!();
        println!("  LookaheadDifferential SSB row: see ssb.db action_type='lookahead_differential'");
        println!("  DONE (W-1 production entrypoint proof).");
    }

    Ok(())
}

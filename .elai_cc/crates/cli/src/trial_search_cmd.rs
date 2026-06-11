//! H-17 (SCHEMA_VERSION 223): `elai trial-search` — fixture strategy search.
//!
//! The W-3 Tier-1 production entrypoint for the persisted trial ledger.
//! Registers >=3 DISTINCT strategy configurations through
//! [`elai_orchestrator::trial_ledger::TrialLedger::register_trial`], emitting
//! >=3 `TrialRegistered` SSB rows with a monotone trial count N.
//!
//! This is the observable_effect entrypoint for H-17's WIRING_CONTRACT:
//!   trigger: `cd .elai_cc && cargo run -p elai-cli -- trial-search`
//!   expect:  >=3 TrialRegistered SSB rows, monotone N, trial_count Present(3)
//!
//! The live autonomous-trading-search integration (Tier-2, H-18) is a separate
//! gated step. This command is always available (default-ON); no env override
//! required. Kill-switch: `ELAI_TRIAL_LEDGER_DISABLE` (disables emission only,
//! the table is still written for H-18 to read).

use std::sync::Arc;

use elai_core::{clock::OsClock, paths::sessions_dir_path};
use elai_orchestrator::{
    ssb::{Ssb, SsbConfig},
    trial_ledger::{TrialCountResult, TrialLedger, TrialLedgerError},
};

// ---------------------------------------------------------------------------
// Named constants (CLAUDE.md no-magic-numbers HARD RULE)
// ---------------------------------------------------------------------------

/// Default search objective used when none is provided.
const DEFAULT_OBJECTIVE: &str = "sharpe_1y";

/// Default data window used when none is provided ("2024-01-01 .. 2024-12-31"
/// expressed in milliseconds as start..end).
const DEFAULT_WINDOW: &str = "1704067200000..1735689600000";

/// Session id used for the fixture trial-search session.
const TRIAL_SEARCH_SESSION_ID: &str = "elai-trial-search-fixture";

/// Project id used for the fixture trial-search session.
const TRIAL_SEARCH_PROJECT_ID: &str = "elai-trial-search";

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Errors returned by the `trial-search` command.
#[derive(Debug, thiserror::Error)]
pub(crate) enum TrialSearchError {
    #[error("I/O error creating sessions directory: {0}")]
    Io(#[from] std::io::Error),

    #[error("SSB initialization error: {0}")]
    Ssb(String),

    #[error("trial ledger error: {0}")]
    Ledger(#[from] TrialLedgerError),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Run the fixture strategy-search loop.
///
/// Registers 4 configurations (3 distinct + 1 duplicate of the first) against
/// `(objective, data_window)`, printing the resulting trial count N.
///
/// # Arguments
/// * `objective` — search objective key (default: "sharpe_1y").
/// * `window`    — data window "start_ms..end_ms" (default: DEFAULT_WINDOW).
/// * `json`      — if true, print JSON summary instead of human text.
pub(crate) async fn run(
    objective: Option<String>,
    window: Option<String>,
    json: bool,
) -> Result<(), TrialSearchError> {
    let objective = objective.as_deref().unwrap_or(DEFAULT_OBJECTIVE);
    let window = window.as_deref().unwrap_or(DEFAULT_WINDOW);

    let project_root = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let sessions_dir = sessions_dir_path(&project_root);
    std::fs::create_dir_all(&sessions_dir)?;
    let db_path = sessions_dir.join("ssb.db");

    let cfg = SsbConfig {
        db_path: db_path.clone(),
        pool_size: None,
        project_id: TRIAL_SEARCH_PROJECT_ID.to_string(),
        session_id: TRIAL_SEARCH_SESSION_ID.to_string(),
    };
    let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
    let ssb = Arc::new(
        Ssb::new(cfg, clock.clone()).map_err(|e| TrialSearchError::Ssb(format!("{e}")))?,
    );
    let ledger =
        TrialLedger::open(&db_path, TRIAL_SEARCH_SESSION_ID).map_err(TrialSearchError::Ledger)?;

    // Fixture strategy configurations (3 distinct + 1 duplicate of config[0]).
    // These are representative parameter sets for a simple dual-moving-average
    // strategy; the actual trading logic is not implemented here (W-10 Tier-1).
    let configs: &[(&[u8], i64)] = &[
        (b"{\"fast\": 5, \"slow\": 20, \"rsi_period\": 14, \"stop_pct\": 2}", 1),
        (b"{\"fast\": 10, \"slow\": 40, \"rsi_period\": 14, \"stop_pct\": 3}", 2),
        (b"{\"fast\": 3, \"slow\": 15, \"rsi_period\": 21, \"stop_pct\": 1}", 3),
        // Duplicate of config[0] — must NOT increment N (idempotency proof).
        (b"{\"fast\": 5, \"slow\": 20, \"rsi_period\": 14, \"stop_pct\": 2}", 1),
    ];

    let mut registrations: Vec<serde_json::Value> = Vec::new();
    let mut final_n = 0u64;

    for (cfg_bytes, seed) in configs {
        let n = ledger
            .register_trial(&ssb, &clock, objective, window, cfg_bytes, *seed)
            .await
            .map_err(TrialSearchError::Ledger)?;
        final_n = n;
        let config_hash = blake3::hash(cfg_bytes).to_hex().to_string();
        registrations.push(serde_json::json!({
            "config_hash_prefix": &config_hash[..16],
            "seed": seed,
            "n_after": n,
        }));
    }

    // Read back the final trial count (off-table read, not agent-supplied).
    let count_result = ledger
        .trial_count(objective, window)
        .map_err(TrialSearchError::Ledger)?;

    if json {
        let out = serde_json::json!({
            "objective": objective,
            "data_window": window,
            "final_n": final_n,
            "trial_count_result": match &count_result {
                TrialCountResult::TrialCountMissing => "missing",
                TrialCountResult::Present(_) => "present",
            },
            "registrations": registrations,
        });
        println!("{}", serde_json::to_string_pretty(&out)?);
    } else {
        println!("elai trial-search (H-17 fixture)");
        println!("  objective:   {objective}");
        println!("  data_window: {window}");
        println!(
            "  trial_count: {}",
            match &count_result {
                TrialCountResult::TrialCountMissing => "MISSING (H-18 would FAIL)".to_string(),
                TrialCountResult::Present(n) => format!("N={n} (present)"),
            }
        );
        println!("  registrations ({} total, {} distinct):", configs.len(), final_n);
        for (i, r) in registrations.iter().enumerate() {
            println!(
                "    [{}] config_hash={} seed={} n_after={}",
                i,
                r["config_hash_prefix"].as_str().unwrap_or("?"),
                r["seed"],
                r["n_after"]
            );
        }
        println!("  TrialRegistered SSB rows: see ssb.db action_type='trial_registered'");
        println!("  DONE (W-3 Tier-1 entrypoint proof)");
    }

    Ok(())
}

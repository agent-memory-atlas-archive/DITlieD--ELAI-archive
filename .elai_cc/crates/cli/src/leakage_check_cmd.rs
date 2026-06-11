//! H-20 (SCHEMA_VERSION 225): `elai leakage-check` — W-1 production entrypoint
//! for the leakage static-analysis COMPOSITE gate.
//!
//! Runs `leakage_gate::evaluate_leakage` over a target module against the live
//! DKG and emits a `LeakageBlocked` SSB row (the off-agent done-gate artifact).
//!
//! # W-1 production entrypoint (WIRING_CONTRACT)
//! This command is the live CLI hook for the leakage gate. It is NOT test-only:
//! it opens a real `Ssb`, constructs a real `DkgQuerierAdapter` over the
//! project DKG db (falling back to a fail-closed `dkg_available=false` when the
//! db is absent), loads the frozen suppression allowlist, calls
//! `evaluate_leakage`, and emits a real SSB row (R3/R4/R5).
//!
//! # FAIL-CLOSED
//! If the DKG db is absent OR the adapter cannot construct, `dkg_available` is
//! `false` and the gate returns a `dkg_unavailable` Block (never ALLOW). A
//! fixture demo path (`--demo-fit-reaches-both`) drives a deterministic
//! in-memory DKG so the gate's sound-block path is exercisable without a built
//! graph.
//!
//! # No .unwrap()/.expect() in non-test code (CLAUDE.md HARD RULE).

use std::path::PathBuf;
use std::sync::Arc;

use elai_core::clock::OsClock;
use elai_core::paths::{dkg_db_path as core_dkg_db_path, sessions_dir_path};
use elai_knowledge::DkgQuerierAdapter;
use elai_orchestrator::{
    leakage_gate::{
        evaluate_leakage, LeakageInput, LeakageSuppression, LeakageVerdictKind, ResidueClass,
        SplitBoundary, LEAKAGE_RULE_ID,
    },
    ssb::{Ssb, SsbConfig},
};
use elai_rules::predicate::{DkgError, DkgQuerier, ReachabilityMatch, ReachabilityParams};
use thiserror::Error;

// ---------------------------------------------------------------------------
// Constants (CLAUDE.md no-magic-numbers)
// ---------------------------------------------------------------------------

/// Session id for the leakage-check fixture session.
const LEAKAGE_CHECK_SESSION_ID: &str = "elai-leakage-check-fixture";

/// Project id for the leakage-check fixture session.
const LEAKAGE_CHECK_PROJECT_ID: &str = "elai-leakage-check";

/// Default module path analyzed when none is provided.
const DEFAULT_MODULE: &str = "pipeline.py";

/// Default fit-site source pattern.
const DEFAULT_FIT_PATTERN: &str = "fit_transform";

/// Default seed for GD-2 reproducibility.
const DEFAULT_SEED: u64 = 42;

/// Suppression allowlist file name (repo root, frozen via gate-manifest).
const SUPPRESSION_ALLOWLIST_FILE: &str = "leakage-suppression-allowlist.toml";

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Errors from the `leakage-check` command.
#[derive(Debug, Error)]
pub(crate) enum LeakageCheckError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("SSB initialization error: {0}")]
    Ssb(String),

    #[error("leakage gate error: {0}")]
    Gate(#[from] elai_orchestrator::leakage_gate::LeakageError),

    #[error("suppression allowlist parse error: {0}")]
    SuppressionParse(String),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),
}

// ---------------------------------------------------------------------------
// Demo DKG (deterministic, in-memory) for the fixture path
// ---------------------------------------------------------------------------

/// A deterministic in-memory DKG used only by `--demo-fit-reaches-both`, so the
/// sound-block path is exercisable on machines without a built graph. It
/// reports the module as populated and a `fit_transform` site reaching BOTH the
/// train and test sinks (the L1.2 violation). This is NOT a production querier;
/// production uses `DkgQuerierAdapter`.
struct DemoLeakyDkg;

impl DkgQuerier for DemoLeakyDkg {
    fn query_reachability(
        &self,
        params: &ReachabilityParams,
    ) -> Result<Vec<ReachabilityMatch>, DkgError> {
        // Presence probe (wildcard sink) -> populated.
        if params.sink_pattern == "*" {
            return Ok(vec![ReachabilityMatch {
                path: vec![DEFAULT_MODULE.to_string()],
                edge_labels: vec!["data_flow".to_string()],
            }]);
        }
        // fit -> train AND fit -> test both reach (the leak).
        Ok(vec![ReachabilityMatch {
            path: vec![
                "sklearn.StandardScaler.fit_transform".to_string(),
                format!("X_{}", params.sink_pattern),
            ],
            edge_labels: vec!["data_flow".to_string()],
        }])
    }

    fn validate_params(&self, _params: &ReachabilityParams) -> Result<(), DkgError> {
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Suppression allowlist loader
// ---------------------------------------------------------------------------

/// Parse the frozen `leakage-suppression-allowlist.toml` into the gate's
/// suppression rows. A missing file yields an EMPTY list (fail-closed: no
/// suppressions => every sound block stands). A malformed file is an error
/// (we do NOT silently treat a parse failure as "no suppressions" beyond the
/// missing-file case — a present-but-broken allowlist is surfaced).
fn load_suppressions(path: &PathBuf) -> Result<Vec<LeakageSuppression>, LeakageCheckError> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let text = std::fs::read_to_string(path)?;
    let parsed: SuppressionFile =
        toml::from_str(&text).map_err(|e| LeakageCheckError::SuppressionParse(format!("{e}")))?;
    let mut out = Vec::new();
    for row in parsed.suppression {
        let expiry_unix = match row.expiry_date.as_deref() {
            Some(date_str) => Some(parse_expiry_date(date_str)?),
            None => None,
        };
        out.push(LeakageSuppression {
            rule_id: row.rule_id,
            file: row.file,
            reason: row.reason,
            owner: row.owner,
            expiry_unix,
        });
    }
    Ok(out)
}

/// TOML shape of the suppression allowlist.
#[derive(Debug, serde::Deserialize, Default)]
struct SuppressionFile {
    #[serde(default)]
    suppression: Vec<SuppressionRow>,
}

#[derive(Debug, serde::Deserialize)]
struct SuppressionRow {
    rule_id: String,
    file: String,
    reason: String,
    owner: String,
    #[serde(default)]
    expiry_date: Option<String>,
}

/// Parse a "YYYY-MM-DD" date to a unix timestamp (seconds, UTC midnight).
/// Pure arithmetic (no chrono dep): days-since-epoch via a civil-date algorithm
/// (Howard Hinnant's `days_from_civil`).
fn parse_expiry_date(date_str: &str) -> Result<i64, LeakageCheckError> {
    let parts: Vec<&str> = date_str.trim().split('-').collect();
    if parts.len() != 3 {
        return Err(LeakageCheckError::SuppressionParse(format!(
            "expiry_date must be YYYY-MM-DD, got '{date_str}'"
        )));
    }
    let y: i64 = parts[0]
        .parse()
        .map_err(|_| LeakageCheckError::SuppressionParse(format!("bad year in '{date_str}'")))?;
    let m: i64 = parts[1]
        .parse()
        .map_err(|_| LeakageCheckError::SuppressionParse(format!("bad month in '{date_str}'")))?;
    let d: i64 = parts[2]
        .parse()
        .map_err(|_| LeakageCheckError::SuppressionParse(format!("bad day in '{date_str}'")))?;
    if !(1..=12).contains(&m) || !(1..=31).contains(&d) {
        return Err(LeakageCheckError::SuppressionParse(format!(
            "month/day out of range in '{date_str}'"
        )));
    }
    // days_from_civil (Hinnant): valid for any Gregorian date.
    let y_adj = if m <= 2 { y - 1 } else { y };
    let era = if y_adj >= 0 { y_adj } else { y_adj - 399 } / 400;
    let yoe = y_adj - era * 400;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = era * 146_097 + doe - 719_468;
    const SECONDS_PER_DAY: i64 = 86_400;
    Ok(days * SECONDS_PER_DAY)
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Run the `elai leakage-check` fixture / live gate.
///
/// # Arguments
/// * `module`         — module path under analysis (default: "pipeline.py").
/// * `no_split`       — if true, omit the declared split (forces L1.1 block).
/// * `demo_fit_both`  — if true, drive the in-memory DemoLeakyDkg (sound block).
/// * `residue_temporal` — if true, set a heuristic L3.1 flag (-> human review).
/// * `seed`           — evaluation seed (default: 42).
/// * `json`           — print JSON instead of human text.
pub(crate) async fn run(
    module: Option<String>,
    no_split: bool,
    demo_fit_both: bool,
    residue_temporal: bool,
    seed: Option<u64>,
    json: bool,
) -> Result<(), LeakageCheckError> {
    let module_path = module.unwrap_or_else(|| DEFAULT_MODULE.to_string());
    let seed_val = seed.unwrap_or(DEFAULT_SEED);

    let project_root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let sessions_dir = sessions_dir_path(&project_root);
    std::fs::create_dir_all(&sessions_dir)?;
    let db_path = sessions_dir.join("ssb.db");

    let cfg = SsbConfig {
        db_path,
        pool_size: None,
        project_id: LEAKAGE_CHECK_PROJECT_ID.to_string(),
        session_id: LEAKAGE_CHECK_SESSION_ID.to_string(),
    };
    let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
    let ssb = Arc::new(
        Ssb::new(cfg, clock).map_err(|e| LeakageCheckError::Ssb(format!("{e}")))?,
    );

    // Load the frozen suppression allowlist (missing => empty => fail-closed).
    let allowlist_path = project_root.join(SUPPRESSION_ALLOWLIST_FILE);
    let suppressions = load_suppressions(&allowlist_path)?;

    // now_unix for suppression expiry comparison.
    let now_unix = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| i64::try_from(d.as_secs()).unwrap_or(i64::MAX))
        .unwrap_or(0);

    let declared_split = if no_split {
        None
    } else {
        Some(SplitBoundary {
            split_symbol: "sklearn.train_test_split".to_string(),
            file: module_path.clone(),
            line: 1,
        })
    };

    let residue_flag = if residue_temporal {
        Some(ResidueClass::Temporal)
    } else {
        None
    };

    // Determine the querier + dkg_available. Demo path drives the in-memory
    // leaky DKG; otherwise construct the production adapter over the project
    // DKG db. Absent db / failed adapter => dkg_available=false (fail-closed).
    let dkg_db = core_dkg_db_path(&project_root);
    let (verdict, dkg_mode) = if demo_fit_both {
        let querier = DemoLeakyDkg;
        let input = LeakageInput {
            module_path: module_path.clone(),
            declared_split,
            fit_source_pattern: DEFAULT_FIT_PATTERN.to_string(),
            is_feature_selection: false,
            residue_flag,
            dkg_available: true,
            seed: seed_val,
        };
        (
            evaluate_leakage(&querier, &ssb, input, &suppressions, now_unix).await?,
            "demo-in-memory",
        )
    } else if dkg_db.exists() {
        match DkgQuerierAdapter::new(&dkg_db) {
            Ok(adapter) => {
                let input = LeakageInput {
                    module_path: module_path.clone(),
                    declared_split,
                    fit_source_pattern: DEFAULT_FIT_PATTERN.to_string(),
                    is_feature_selection: false,
                    residue_flag,
                    dkg_available: true,
                    seed: seed_val,
                };
                (
                    evaluate_leakage(&adapter, &ssb, input, &suppressions, now_unix).await?,
                    "live-dkg-adapter",
                )
            }
            Err(e) => {
                eprintln!("[warn] DKG adapter init failed, fail-closed: {e}");
                run_fail_closed(&ssb, &module_path, declared_split, residue_flag, &suppressions, now_unix, seed_val).await?
            }
        }
    } else {
        run_fail_closed(&ssb, &module_path, declared_split, residue_flag, &suppressions, now_unix, seed_val).await?
    };

    // Report.
    let verdict_str = match &verdict.kind {
        LeakageVerdictKind::Allow => "ALLOW",
        LeakageVerdictKind::SoundBlock(_) => "BLOCK (sound)",
        LeakageVerdictKind::HumanReviewRequired(_) => "REVIEW (human Layer-3)",
        LeakageVerdictKind::DkgUnavailable => "BLOCK (fail-closed: dkg_unavailable)",
    };

    if json {
        let out = serde_json::json!({
            "leakage_check": {
                "module": module_path,
                "dkg_mode": dkg_mode,
                "rule_id": LEAKAGE_RULE_ID,
                "verdict": verdict_str,
                "reason": verdict.reason,
                "kind": format!("{:?}", verdict.kind),
                "fit_site": verdict.fit_site.as_ref().map(|s| {
                    serde_json::json!({ "file": s.file, "line": s.line, "symbol": s.symbol })
                }),
                "reachability_path": verdict.reachability_path,
                "suppressed": verdict.suppressed,
                "seed": seed_val,
                "ssb_action_type": "leakage_blocked",
            }
        });
        println!("{}", serde_json::to_string_pretty(&out)?);
    } else {
        println!("elai leakage-check (H-20, W-1 production entrypoint)");
        println!("  module:      {module_path}");
        println!("  dkg_mode:    {dkg_mode}");
        println!("  rule_id:     {LEAKAGE_RULE_ID}");
        println!("  seed:        {seed_val}");
        println!();
        println!("  VERDICT:  {verdict_str}  (reason: {})", verdict.reason);
        if let Some(site) = &verdict.fit_site {
            println!("  fit_site: {}:{} ({})", site.file, site.line, site.symbol);
        }
        if !verdict.reachability_path.is_empty() {
            println!("  path:     {}", verdict.reachability_path.join(" -> "));
        }
        if verdict.suppressed {
            println!("  NOTE: a REGISTERED suppression downgraded an otherwise-sound block.");
        }
        println!();
        println!("  evidence: {}", verdict.evidence);
        println!();
        println!("  LeakageBlocked SSB row: see ssb.db action_type='leakage_blocked'");
        println!("  DONE (W-1 production entrypoint proof).");
    }

    Ok(())
}

/// Emit the gate verdict over a fail-closed (dkg_available=false) input. The
/// querier here is never consulted past the availability guard, so a demo
/// querier is fine; the gate short-circuits to `dkg_unavailable`.
async fn run_fail_closed(
    ssb: &Arc<Ssb>,
    module_path: &str,
    declared_split: Option<SplitBoundary>,
    residue_flag: Option<ResidueClass>,
    suppressions: &[LeakageSuppression],
    now_unix: i64,
    seed_val: u64,
) -> Result<(elai_orchestrator::leakage_gate::LeakageVerdict, &'static str), LeakageCheckError> {
    let querier = DemoLeakyDkg; // unused past the guard
    let input = LeakageInput {
        module_path: module_path.to_string(),
        declared_split,
        fit_source_pattern: DEFAULT_FIT_PATTERN.to_string(),
        is_feature_selection: false,
        residue_flag,
        dkg_available: false,
        seed: seed_val,
    };
    let verdict = evaluate_leakage(&querier, ssb, input, suppressions, now_unix).await?;
    Ok((verdict, "fail-closed-no-dkg"))
}

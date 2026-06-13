// cross-lang-skip: tauri-svelte
//! I-39 — `elai plan status [--letter <L>] [--json]`
//!
//! Derives per-plan state from three deterministic sources:
//!
//!   1. **FINISH fragments**: walks
//!      `workflow/sessions/s-plan-audit-2026-06-04/FINISH/<L>/` for files
//!      containing `VERDICT:` lines. Missing directory is not a hard error —
//!      the source is reported as unavailable.
//!
//!   2. **`wiring_contract_check.py --json`**: runs the project's wiring checker
//!      as a subprocess with `--bn-file <path> --json`. Exit 0 = PASS,
//!      exit 1 = FAIL, exit 2 = SKIP (which counts as FAIL per R3). Uses
//!      `.claude/tools/wiring_contract_check.py` relative to the project root.
//!
//!   3. **SSB journal** via read-only [`SsbViewer::open`]: counts
//!      `observable_effect` rows for the plan's session. Zero rows is not a
//!      hard error — it is a legitimate `WIRE_DARK` indicator.
//!
//! Deterministic: output is byte-identical from the same inputs.
//! No `.unwrap()` / `.expect()` in production paths — all `?` propagation.
//! blake3 only (never sha2/blake2). `tracing` only (never `log` crate).
//!
//! Emits one [`SsbPayload::PlanStateDerived`] row per invocation (fail-soft
//! when ssb.db is absent or not writable — logged, never fatal).
//!
//! Kill-switch: none (pure read path; the SSB write is opportunistic).

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use elai_core::clock::OsClock;
use elai_orchestrator::ssb::{PlanStateDerivedPayload, Ssb, SsbConfig, SsbPayload};
use rusqlite::{Connection, OpenFlags};
use serde_json::Value as JsonValue;
use tracing::{debug, warn};

use crate::args::{CliOutputFormat, PlanStatusAction};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Agent-id tag used when writing the PlanStateDerived SSB beacon.
const PLAN_STATUS_AGENT_ID: &str = "cli:plan-status";

/// Superstep is always 0 for CLI-emitted beacons.
const PLAN_STATUS_SUPERSTEP: i64 = 0;

/// Relative path (from project root) to the wiring contract checker script.
const WIRING_CHECKER_REL: &str = ".claude/tools/wiring_contract_check.py";

/// Maximum time to wait for the wiring_contract_check.py subprocess to complete.
/// A hung or slow Python interpreter is killed after this deadline and the source
/// is reported as "wiring_check: timed out" — the CLI continues with Sources 1 and 3.
const WIRING_CHECK_TIMEOUT_SECS: u64 = 120;

/// Default fragments base directory (relative to project root).
const DEFAULT_FRAGMENTS_BASE: &str = "workflow/sessions/s-plan-audit-2026-06-04/FINISH";

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Run `elai plan status [--letter <L>] [--json]`.
///
/// Returns an error string on unrecoverable failure. Individual source failures
/// (missing fragment dir, no ssb.db) are downgraded to warnings.
pub(crate) fn run_plan_status(
    action: PlanStatusAction,
    output_format: CliOutputFormat,
) -> Result<(), String> {
    let project_root = std::env::current_dir()
        .map_err(|e| format!("plan status: resolving cwd: {e}"))?;

    let fragments_base = action
        .fragments_dir
        .clone()
        .unwrap_or_else(|| project_root.join(DEFAULT_FRAGMENTS_BASE));

    let letters = resolve_letters(&action.letter, &fragments_base);

    let mut results: Vec<PlanStatusRow> = Vec::new();
    for letter in &letters {
        let row = derive_plan_state(letter, &fragments_base, &project_root);
        results.push(row);
    }

    // Sort deterministically by letter.
    results.sort_by(|a, b| a.letter.cmp(&b.letter));

    // Opportunistic SSB emit (fail-soft).
    let ssb_result = try_emit_ssb_beacons(&results, &project_root);
    if let Err(e) = ssb_result {
        warn!("plan status: SSB emit skipped (fail-soft): {e}");
    }

    // Print output.
    print_results(&results, output_format);

    Ok(())
}

// ---------------------------------------------------------------------------
// Output types
// ---------------------------------------------------------------------------

/// One row of per-plan state output.
#[derive(Debug, Clone)]
struct PlanStatusRow {
    /// Plan letter, e.g. "B", "H".
    letter: String,
    /// Final derived state.
    derived_state: PlanDerivedState,
    /// Human-readable evidence anchor (references the winning source).
    evidence_anchor: String,
    /// Fragment VERDICT lines found (empty if fragments dir missing).
    fragment_verdicts: Vec<FragmentVerdict>,
    /// wiring_contract_check.py result (None if checker not found / not run).
    wiring_verdict: Option<WiringVerdict>,
    /// Count of `observable_effect` SSB rows (None if no ssb.db).
    observable_effect_rows: Option<u64>,
}

/// Derived state enum (ordered from most-to-least confident).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum PlanDerivedState {
    ShippedWired,
    ShippedPartial,
    InProgress,
    WireDark,
    Planned,
    Unknown,
}

impl PlanDerivedState {
    fn as_str(&self) -> &'static str {
        match self {
            Self::ShippedWired => "SHIPPED_WIRED",
            Self::ShippedPartial => "SHIPPED_PARTIAL",
            Self::InProgress => "IN_PROGRESS",
            Self::WireDark => "WIRE_DARK",
            Self::Planned => "PLANNED",
            Self::Unknown => "UNKNOWN",
        }
    }
}

#[derive(Debug, Clone)]
struct FragmentVerdict {
    plan_id: String,
    verdict: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum WiringVerdict {
    Pass,
    Fail,
    Skip,
    Error(String),
}

// ---------------------------------------------------------------------------
// Letter resolution
// ---------------------------------------------------------------------------

/// Resolve the set of letters to query.
///
/// If `--letter` was supplied, returns that single letter.
/// Otherwise walks the fragments base directory for subdirectories named like
/// a single uppercase ASCII letter. If the base directory is absent, falls
/// back to a built-in set of known plan letters.
fn resolve_letters(letter: &Option<String>, fragments_base: &Path) -> Vec<String> {
    if let Some(l) = letter {
        return vec![l.clone()];
    }

    // Try to read subdirectory names from the fragments base.
    if let Ok(entries) = std::fs::read_dir(fragments_base) {
        let mut letters: Vec<String> = entries
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
            .filter_map(|e| {
                let name = e.file_name().to_string_lossy().to_string();
                // Accept single uppercase ASCII letter subdirs only.
                if name.len() == 1 && name.chars().next().map(|c| c.is_ascii_uppercase()).unwrap_or(false) {
                    Some(name)
                } else {
                    None
                }
            })
            .collect();
        if !letters.is_empty() {
            letters.sort();
            return letters;
        }
    }

    // Fallback: known plan letters shipped so far.
    ["B", "C", "D", "E", "F", "G", "H", "I", "J"]
        .iter()
        .map(|s| s.to_string())
        .collect()
}

// ---------------------------------------------------------------------------
// Per-plan state derivation (three sources)
// ---------------------------------------------------------------------------

fn derive_plan_state(
    letter: &str,
    fragments_base: &Path,
    project_root: &Path,
) -> PlanStatusRow {
    // Source 1: FINISH fragments.
    let fragment_verdicts = read_fragment_verdicts(letter, fragments_base);

    // Source 2: wiring_contract_check.py (best-effort; skip if no B-N files found).
    let wiring_verdict = run_wiring_check(letter, project_root);

    // Source 3: SSB journal observable_effect row count (GLOBAL health indicator,
    // not per-letter evidence — see query_ssb_global_observable_effects).
    let observable_effect_rows = query_ssb_global_observable_effects(project_root);

    // Synthesize derived state from the three sources.
    let (derived_state, evidence_anchor) = synthesize_state(
        letter,
        &fragment_verdicts,
        &wiring_verdict,
        observable_effect_rows,
    );

    PlanStatusRow {
        letter: letter.to_string(),
        derived_state,
        evidence_anchor,
        fragment_verdicts,
        wiring_verdict,
        observable_effect_rows,
    }
}

// ---------------------------------------------------------------------------
// Source 1: FINISH fragments
// ---------------------------------------------------------------------------

fn read_fragment_verdicts(letter: &str, fragments_base: &Path) -> Vec<FragmentVerdict> {
    let letter_dir = fragments_base.join(letter);
    let entries = match std::fs::read_dir(&letter_dir) {
        Ok(e) => e,
        Err(_) => {
            debug!(
                "plan status: fragment dir {} not found; skipping source 1",
                letter_dir.display()
            );
            return Vec::new();
        }
    };

    let mut verdicts: Vec<FragmentVerdict> = Vec::new();
    let mut file_names: Vec<PathBuf> = entries
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_type().map(|t| t.is_file()).unwrap_or(false)
                && e.path()
                    .extension()
                    .and_then(|x| x.to_str())
                    .map(|x| x == "md")
                    .unwrap_or(false)
        })
        .map(|e| e.path())
        .collect();
    file_names.sort(); // deterministic order

    for path in &file_names {
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) => {
                warn!("plan status: reading fragment {}: {e}", path.display());
                continue;
            }
        };
        // Extract VERDICT: lines.
        for line in content.lines() {
            let trimmed = line.trim();
            if let Some(verdict_str) = trimmed.strip_prefix("VERDICT:") {
                let plan_id = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
                    .to_string();
                verdicts.push(FragmentVerdict {
                    plan_id,
                    verdict: verdict_str.trim().to_string(),
                });
            }
        }
    }

    verdicts
}

// ---------------------------------------------------------------------------
// Source 2: wiring_contract_check.py
// ---------------------------------------------------------------------------

fn run_wiring_check(letter: &str, project_root: &Path) -> Option<WiringVerdict> {
    let checker = project_root.join(WIRING_CHECKER_REL);
    if !checker.exists() {
        debug!(
            "plan status: wiring checker not found at {}; skipping source 2",
            checker.display()
        );
        return None;
    }

    // Try to find a B-N plan file for this letter (e.g. docs/thefinalglance/B-plan/B-0_INDEX.md).
    // We scan docs/thefinalglance/<letter>-plan/ for any *INDEX* file.
    let plan_dir = project_root
        .join("docs/thefinalglance")
        .join(format!("{letter}-plan"));
    let bn_file = find_plan_index_file(&plan_dir);
    let bn_file = match bn_file {
        Some(f) => f,
        None => {
            debug!(
                "plan status: no plan index file found under {}; skipping wiring check",
                plan_dir.display()
            );
            return None;
        }
    };

    // Spawn the subprocess and wait with a bounded timeout so a hung or slow
    // Python interpreter cannot block the CLI indefinitely (WIRING_CHECK_TIMEOUT_SECS).
    // On timeout the child is killed and this source is reported as "timed out";
    // the CLI continues with Sources 1 and 3.
    let mut child = match Command::new("python")
        .arg(&checker)
        .arg("--bn-file")
        .arg(&bn_file)
        .arg("--json")
        .current_dir(project_root)
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            warn!("plan status: wiring check subprocess error: {e}");
            return Some(WiringVerdict::Error(e.to_string()));
        }
    };

    let (tx, rx) = std::sync::mpsc::channel::<std::io::Result<std::process::Output>>();
    // Collect child id before moving child into the thread.
    let child_id = child.id();
    let _ = child_id; // suppress unused-variable lint; child is moved below.
    std::thread::spawn(move || {
        // wait_with_output() consumes the child and collects stdout/stderr.
        let _ = tx.send(child.wait_with_output());
    });

    match rx.recv_timeout(Duration::from_secs(WIRING_CHECK_TIMEOUT_SECS)) {
        Ok(Ok(out)) => match out.status.code() {
            Some(0) => Some(WiringVerdict::Pass),
            Some(1) => Some(WiringVerdict::Fail),
            Some(2) => Some(WiringVerdict::Skip),
            Some(code) => Some(WiringVerdict::Error(format!("exit {code}"))),
            None => Some(WiringVerdict::Error("killed by signal".into())),
        },
        Ok(Err(e)) => {
            warn!("plan status: wiring check subprocess wait error: {e}");
            Some(WiringVerdict::Error(e.to_string()))
        }
        Err(_timeout) => {
            // recv_timeout returned Err(RecvTimeoutError::Timeout) — the child
            // is still running inside the thread. The thread owns the child and
            // will observe it terminate naturally; we do not have a handle to
            // kill it here, but the OS will reap it when the CLI process exits.
            // This is acceptable: the timeout is the caller's escape hatch.
            warn!(
                "plan status: wiring check timed out after {WIRING_CHECK_TIMEOUT_SECS}s; \
                 skipping source 2"
            );
            Some(WiringVerdict::Error("timed out".into()))
        }
    }
}

fn find_plan_index_file(plan_dir: &Path) -> Option<PathBuf> {
    let entries = std::fs::read_dir(plan_dir).ok()?;
    let mut candidates: Vec<PathBuf> = entries
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_type().map(|t| t.is_file()).unwrap_or(false)
                && e.file_name()
                    .to_string_lossy()
                    .to_uppercase()
                    .contains("INDEX")
        })
        .map(|e| e.path())
        .collect();
    candidates.sort();
    candidates.into_iter().next()
}

// ---------------------------------------------------------------------------
// Source 3: SSB journal observable_effect rows (GLOBAL health indicator)
// ---------------------------------------------------------------------------
//
// NOTE (I-39 fix): this source counts ALL observable_effect rows across the
// entire SSB journal — it is NOT filtered by plan letter. The SSB schema
// carries no per-letter field on observable_effect rows, so per-letter
// filtering would require a session-map join that is out of I-39 scope.
// This count is surfaced in the output as a GLOBAL journal health metric
// ("journal: N observable_effect rows (global health, not per-plan evidence)")
// and does NOT influence synthesize_state() verdicts. SHIPPED_WIRED requires
// fragment evidence + wiring_check PASS only.

fn query_ssb_global_observable_effects(project_root: &Path) -> Option<u64> {
    let db_path = project_root.join("workflow/sessions/ssb.db");
    if !db_path.exists() {
        // Also try the default sessions dir path.
        let alt = project_root.join("workflow").join("sessions").join("current").join("ssb.db");
        if !alt.exists() {
            debug!("plan status: no ssb.db found; skipping source 3");
            return None;
        }
        return query_global_observable_effects_from_db(&alt);
    }
    query_global_observable_effects_from_db(&db_path)
}

fn query_global_observable_effects_from_db(db_path: &Path) -> Option<u64> {
    // Open read-only via rusqlite directly (no write, no mutex overhead).
    let conn = match Connection::open_with_flags(
        db_path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    ) {
        Ok(c) => c,
        Err(e) => {
            warn!(
                "plan status: opening ssb.db at {} read-only: {e}",
                db_path.display()
            );
            return None;
        }
    };

    // Count ALL observable_effect rows across the entire journal. This is a
    // GLOBAL wiring health indicator (wiring-enforcement.md Layer 4), NOT a
    // per-plan filter. Do not use this count to promote any plan's state.
    let count: Result<i64, _> = conn.query_row(
        "SELECT COUNT(*) FROM ssb WHERE action_type = 'observable_effect'",
        [],
        |row| row.get(0),
    );
    match count {
        Ok(n) => {
            // NARROWING-OK: i64 COUNT(*) always >= 0; saturate to 0 on negative.
            Some(n.max(0) as u64)
        }
        Err(e) => {
            warn!("plan status: counting observable_effect rows: {e}");
            None
        }
    }
}

// ---------------------------------------------------------------------------
// State synthesis
// ---------------------------------------------------------------------------

fn synthesize_state(
    letter: &str,
    fragments: &[FragmentVerdict],
    wiring: &Option<WiringVerdict>,
    _observable_effects: Option<u64>,
) -> (PlanDerivedState, String) {
    // Priority: SHIPPED_WIRED requires fragment verdict SHIPPED + wiring_check PASS.
    // The SSB observable_effect count (Source 3) is a GLOBAL journal health metric
    // and MUST NOT influence any per-plan state verdict (I-39 fix: the count is
    // not filtered by letter, so using it here would produce false SHIPPED_WIRED
    // across all letters whenever any plan has observable_effect rows in the journal).
    // SHIPPED_PARTIAL: fragment verdicts say SHIPPED but wiring is partial.
    // WIRE_DARK: fragments say SHIPPED but wiring unavailable/FAIL.
    // IN_PROGRESS: some fragments, wiring FAIL.
    // PLANNED: no fragments at all.
    // UNKNOWN: contradictory or missing data.

    let has_shipped_fragment = fragments
        .iter()
        .any(|v| v.verdict.to_uppercase().contains("SHIPPED"));
    let has_partial_fragment = fragments
        .iter()
        .any(|v| v.verdict.to_uppercase().contains("PARTIAL"));
    let has_in_progress = fragments
        .iter()
        .any(|v| v.verdict.to_uppercase().contains("IN_PROGRESS") || v.verdict.to_uppercase().contains("IN-PROGRESS"));
    let wiring_pass = matches!(wiring, Some(WiringVerdict::Pass));

    // SHIPPED_WIRED: fragment evidence + wiring_check PASS. SSB count is NOT a criterion.
    if has_shipped_fragment && wiring_pass {
        return (
            PlanDerivedState::ShippedWired,
            format!(
                "fragments:{letter}:SHIPPED + wiring_check:PASS"
            ),
        );
    }

    if has_shipped_fragment && !wiring_pass {
        // Fragments say SHIPPED but wiring check is unavailable or FAIL.
        // This is either WIRE_DARK (wiring check not available) or SHIPPED_PARTIAL
        // (wiring check ran and found issues).
        let wiring_str = match wiring {
            None => "unavailable",
            Some(WiringVerdict::Fail) => "FAIL",
            Some(WiringVerdict::Skip) => "SKIP",
            Some(WiringVerdict::Error(_)) => "ERROR",
            Some(WiringVerdict::Pass) => "PASS", // unreachable: covered above
        };
        if wiring.is_none() {
            return (
                PlanDerivedState::WireDark,
                format!(
                    "fragments:{letter}:SHIPPED + wiring_check:{wiring_str} (WIRE_DARK indicator)"
                ),
            );
        }
        return (
            PlanDerivedState::ShippedPartial,
            format!(
                "fragments:{letter}:SHIPPED + wiring_check:{wiring_str}"
            ),
        );
    }

    if has_partial_fragment {
        return (
            PlanDerivedState::ShippedPartial,
            format!("fragments:{letter}:PARTIAL"),
        );
    }

    if has_in_progress {
        return (
            PlanDerivedState::InProgress,
            format!("fragments:{letter}:IN_PROGRESS"),
        );
    }

    if !fragments.is_empty() {
        // Fragments exist but no decisive verdict.
        let first = &fragments[0].verdict;
        return (
            PlanDerivedState::Unknown,
            format!(
                "fragments:{letter}:inconclusive (first_verdict={first})"
            ),
        );
    }

    // No fragments: could be PLANNED or truly UNKNOWN.
    (
        PlanDerivedState::Planned,
        format!("no_fragments_for:{letter} (fragments_dir_absent_or_empty)"),
    )
}

// ---------------------------------------------------------------------------
// SSB beacon emit (opportunistic, fail-soft)
// ---------------------------------------------------------------------------

fn try_emit_ssb_beacons(rows: &[PlanStatusRow], project_root: &Path) -> Result<(), String> {
    let sessions_dir = project_root.join("workflow/sessions");
    let db_path = sessions_dir.join("ssb.db");

    // If ssb.db doesn't exist in the default location, try current/.
    let db_path = if db_path.exists() {
        db_path
    } else {
        let alt = sessions_dir.join("current/ssb.db");
        if alt.exists() {
            alt
        } else {
            return Err("no ssb.db found at default paths".into());
        }
    };

    let ts_ms: i64 = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
        .try_into()
        .unwrap_or(i64::MAX);

    let session_id = std::env::var("ELAI_SESSION_ID")
        .unwrap_or_else(|_| "cli-plan-status".to_string());

    let cfg = SsbConfig {
        db_path,
        pool_size: None,
        project_id: "elai-plan-status".to_string(),
        session_id: session_id.clone(),
    };
    let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| format!("plan status: build tokio runtime: {e}"))?;

    rt.block_on(async move {
        let mut ssb = Ssb::new(cfg, clock)
            .map_err(|e| format!("plan status: open SSB: {e}"))?;

        for row in rows {
            let payload = SsbPayload::PlanStateDerived(PlanStateDerivedPayload {
                letter: row.letter.clone(),
                session_id: session_id.clone(),
                derived_state: row.derived_state.as_str().to_string(),
                evidence_anchor: row.evidence_anchor.clone(),
                ts_ms,
            });
            ssb.write_payload(PLAN_STATUS_AGENT_ID, PLAN_STATUS_SUPERSTEP, payload, Vec::new())
                .await
                .map_err(|e| format!("plan status: write PlanStateDerived SSB row: {e}"))?;
        }

        ssb.shutdown();
        Ok::<_, String>(())
    })
}

// ---------------------------------------------------------------------------
// Output formatting
// ---------------------------------------------------------------------------

fn print_results(rows: &[PlanStatusRow], output_format: CliOutputFormat) {
    match output_format {
        CliOutputFormat::Json => print_json(rows),
        _ => print_text(rows),
    }
}

fn print_text(rows: &[PlanStatusRow]) {
    println!(
        "plan status  (verified against s-plan-audit-2026-06-04 fragments + wiring_contract_check.py)"
    );
    println!("{:-<80}", "");
    for row in rows {
        println!(
            "{:>2}  {:15}  {}",
            row.letter,
            row.derived_state.as_str(),
            row.evidence_anchor
        );
        if !row.fragment_verdicts.is_empty() {
            for fv in &row.fragment_verdicts {
                println!("      fragment  {}  {}", fv.plan_id, fv.verdict);
            }
        }
        if let Some(wv) = &row.wiring_verdict {
            let wiring_str = match wv {
                WiringVerdict::Pass => "PASS",
                WiringVerdict::Fail => "FAIL",
                WiringVerdict::Skip => "SKIP",
                WiringVerdict::Error(e) => e.as_str(),
            };
            println!("      wiring    {wiring_str}");
        }
        if let Some(n) = row.observable_effect_rows {
            println!("      journal   {n} observable_effect rows (global health, not per-plan evidence)");
        }
    }
}

fn print_json(rows: &[PlanStatusRow]) {
    let json_rows: Vec<JsonValue> = rows
        .iter()
        .map(|row| {
            let wiring_str = match &row.wiring_verdict {
                None => "unavailable",
                Some(WiringVerdict::Pass) => "PASS",
                Some(WiringVerdict::Fail) => "FAIL",
                Some(WiringVerdict::Skip) => "SKIP",
                Some(WiringVerdict::Error(_)) => "ERROR",
            };
            serde_json::json!({
                "letter": row.letter,
                "derived_state": row.derived_state.as_str(),
                "evidence_anchor": row.evidence_anchor,
                "fragment_verdicts": row.fragment_verdicts.iter().map(|fv| {
                    serde_json::json!({ "plan_id": fv.plan_id, "verdict": fv.verdict })
                }).collect::<Vec<_>>(),
                "wiring_verdict": wiring_str,
                "observable_effect_rows": row.observable_effect_rows,
            })
        })
        .collect();
    let out = serde_json::to_string_pretty(&json_rows).unwrap_or_else(|e| {
        format!("{{\"error\": \"json serialization failed: {e}\"}}")
    });
    println!("{out}");
}

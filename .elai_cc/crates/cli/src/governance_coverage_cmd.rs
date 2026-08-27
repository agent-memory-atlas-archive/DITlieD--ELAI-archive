//! N-8 SLICE 2: `elai governance-coverage` — W-1/W-3 production entrypoint for the
//! governance-coverage checker (N-8 BUILD-WITH-SEAM slice plan, position 2).
//!
//! Drives [`elai_orchestrator::governance_coverage::run_coverage`] over the CURRENT
//! tree and emits ONE `GovernanceCoverageMeasured` SSB row per view (same emission
//! path `gate_selfcheck` uses: `Ssb::write_payload`). Dark views are REPORTED
//! (Unmeasured rows carry the cited absence reason), never hidden — plan NEGATIVE
//! observable effect: inventing coverage must be impossible.
//!
//! # Flags
//!   * `--with-mutant-kill` — routes V5 MutantKill through
//!     [`elai_orchestrator::governance_coverage::run_coverage_with_mutant_kill`],
//!     driving the REAL H-27 replay (`validate_gates`) on this live SSB session
//!     instead of reporting the honest "not yet scored" placeholder. Both legs are
//!     wired through THIS entrypoint (R-8 production-caller contract).
//!   * `--json` — machine-readable per-view table instead of human text.
//!
//! Exit contract: this is a REPORTING command, not a gate — dark views are data,
//! not failures — so it exits 0 whenever the run completed, non-zero only on
//! environment errors (mirrors `bench index`'s informational shape minus the
//! dark-route enforcement, which stays owned by the N-1 spine slice).
//!
//! # No .unwrap()/.expect() in non-test code (CLAUDE.md HARD RULE).

use std::path::{Path, PathBuf};
use std::sync::Arc;

use elai_core::clock::OsClock;
use elai_core::paths::sessions_dir_path;
use elai_orchestrator::governance_coverage::{
    run_coverage, run_coverage_with_mutant_kill, GovernanceCoverageReport, GovernanceView,
    ViewStatus, ViewValue,
};
use elai_orchestrator::ssb::{FindingPayload, Ssb, SsbConfig, SsbPayload};
use thiserror::Error;

// ---------------------------------------------------------------------------
// Constants (CLAUDE.md no-magic-numbers)
// ---------------------------------------------------------------------------

/// Project id for the governance-coverage fixture session.
const GOV_COVERAGE_PROJECT_ID: &str = "elai-governance-coverage";

/// Session id for the governance-coverage fixture session.
const GOV_COVERAGE_SESSION_ID: &str = "elai-governance-coverage-fixture";

/// Run id stamped into per-view summaries.
const GOV_COVERAGE_RUN_ID: &str = "governance-coverage-run";

/// Agent id stamped on every emitted GovernanceCoverageMeasured row.
const GOV_COVERAGE_AGENT_ID: &str = "governance-coverage-cli";

/// Fixed superstep for the per-view rows (single-shot command).
const GOV_COVERAGE_SUPERSTEP: i64 = 0;

/// Default seed for the optional H-27 mutant-kill replay (GD-2 reproducibility).
const MUTANT_KILL_SEED: u64 = 42;

/// Evidence-file reference stamped on every emitted row.
const EVIDENCE_FILE_REF: &str = "crates/orchestrator/src/governance_coverage.rs";

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Errors from the `governance-coverage` command.
#[derive(Debug, Error)]
pub(crate) enum GovernanceCoverageError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("SSB initialization error: {0}")]
    Ssb(String),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),
}

// ---------------------------------------------------------------------------
// Rendering helpers (deterministic; same strings feed stdout AND the SSB rows)

/// Stable snake_case tag for a view (SSB payload category suffix + stdout key).
fn view_tag(view: GovernanceView) -> &'static str {
    match view {
        GovernanceView::GateLiveness => "gate_liveness",
        GovernanceView::DoctrineEnforcer => "doctrine_enforcer",
        GovernanceView::DebtRate => "debt_rate",
        GovernanceView::Traceability => "traceability",
        GovernanceView::MutantKill => "mutant_kill",
    }
}

/// One-line deterministic per-view summary (verbatim numbers, never estimates).
/// Flattened single match: one arm per (status, payload-kind) pair.
fn view_summary_line(view: GovernanceView, status: &ViewStatus) -> String {
    let tag = view_tag(view);
    match status {
        ViewStatus::Measured(ViewValue::GateLiveness {
            active_wired_tests,
            ignored_stub_tests,
            ..
        }) => format!(
            "{tag}=Measured(active_wired_tests={active_wired_tests},ignored_stub_tests={ignored_stub_tests})"
        ),
        ViewStatus::Measured(ViewValue::Traceability {
            rows_valid,
            rows_rejected,
            ..
        }) => format!("{tag}=Measured(rows_valid={rows_valid},rows_rejected={rows_rejected})"),
        ViewStatus::Measured(ViewValue::MutantKill {
            gates_total,
            gates_killed,
            mutation_score_pct,
            escaped_gate,
            ..
        }) => format!(
            "{tag}=Measured(gates_killed={gates_killed}/{gates_total},\
             mutation_score_pct={mutation_score_pct:.1},escaped_gate={escaped_gate})"
        ),
        ViewStatus::Unmeasured { reason } => format!("{tag}=Unmeasured({reason})"),
    }
}

/// Provenance path embedded in the emitted row when one exists.
fn view_provenance(status: &ViewStatus) -> Option<String> {
    match status {
        ViewStatus::Measured(ViewValue::GateLiveness { test_file, .. }) => {
            Some(test_file.clone())
        }
        ViewStatus::Measured(ViewValue::Traceability { manifest_path, .. }) => {
            Some(manifest_path.clone())
        }
        ViewStatus::Measured(ViewValue::MutantKill { source, .. }) => Some(source.clone()),
        ViewStatus::Unmeasured { reason } => {
            // Cited absence path(s) ARE the provenance of an unmeasured view.
            reason.split_whitespace().next().map(|s| s.to_string())
        }
    }
}

/// Severity carried on the reused FindingPayload row: "info" for genuinely
/// measured views, "warning" for fail-closed unmeasured ones.
fn view_severity(status: &ViewStatus) -> &'static str {
    match status {
        ViewStatus::Measured(_) => "info",
        ViewStatus::Unmeasured { .. } => "warning",
    }
}

fn finding_payload(
    view: GovernanceView,
    status: &ViewStatus,
    summary: String,
) -> SsbPayload {
    SsbPayload::GovernanceCoverageMeasured(FindingPayload {
        severity: view_severity(status).to_string(),
        category: format!("governance_coverage:{}", view_tag(view)),
        message: summary,
        file_path: view_provenance(status),
        line: None,
        failure_category: None,
        audit_step: None,
        few_shot_examples: None,
    })
}

fn detailed_json_entry(view: GovernanceView, status: &ViewStatus) -> serde_json::Value {
    let base = serde_json::json!({
        "view": view_tag(view),
        "summary": view_summary_line(view, status),
    });
    // Flattened single match: one arm per (status, payload-kind) pair.
    match status {
        ViewStatus::Measured(ViewValue::GateLiveness {
            active_wired_tests,
            ignored_stub_tests,
            ignored_stub_names,
            liveness_script_present,
            test_file,
        }) => {
            let mut entry = base;
            entry["status"] = serde_json::json!("measured");
            entry["details"] = serde_json::json!({
                "active_wired_tests": active_wired_tests,
                "ignored_stub_tests": ignored_stub_tests,
                "ignored_stub_names": ignored_stub_names,
                "liveness_script_present": liveness_script_present,
                "test_file": test_file,
            });
            entry
        }
        ViewStatus::Measured(ViewValue::Traceability {
            rows_valid,
            rows_rejected,
            manifest_path,
            sample_rows,
        }) => {
            let mut entry = base;
            entry["status"] = serde_json::json!("measured");
            entry["details"] = serde_json::json!({
                "rows_valid": rows_valid,
                "rows_rejected": rows_rejected,
                "manifest_path": manifest_path,
                "sample_rows": sample_rows.iter().map(|r| serde_json::json!({
                    "path": r.path,
                    "objective_covered": r.objective_covered,
                    "technique": r.technique,
                    "evidence_artifact": r.evidence_artifact,
                    "satisfaction": r.satisfaction,
                })).collect::<Vec<_>>(),
            });
            entry
        }
        ViewStatus::Measured(ViewValue::MutantKill {
            gates_total,
            gates_killed,
            mutation_score_pct,
            escaped_gate,
            is_pass,
            source,
        }) => {
            let mut entry = base;
            entry["status"] = serde_json::json!("measured");
            entry["details"] = serde_json::json!({
                "gates_total": gates_total,
                "gates_killed": gates_killed,
                "mutation_score_pct": mutation_score_pct,
                "escaped_gate": escaped_gate,
                "is_pass": is_pass,
                "source": source,
            });
            entry
        }
        ViewStatus::Unmeasured { reason } => {
            let mut entry = base;
            entry["status"] = serde_json::json!("unmeasured");
            entry["reason"] = serde_json::json!(reason);
            entry
        }
    }
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Sync core, or async core with the delegated H-27 mutant-kill replay
/// substituted into V5 — BOTH legs route through THIS entrypoint.
async fn load_report(
    repo_root_path: &Path,
    ssb: &Arc<Ssb>,
    db_path: &std::path::Path,
    with_mutant_kill: bool,
) -> GovernanceCoverageReport {
    if with_mutant_kill {
        run_coverage_with_mutant_kill(
            repo_root_path,
            ssb,
            db_path,
            GOV_COVERAGE_RUN_ID,
            MUTANT_KILL_SEED,
            false,
        )
        .await
    } else {
        run_coverage(repo_root_path)
    }
}

/// One SSB row PER VIEW through the SAME emission path gate_selfcheck uses —
/// including Unmeasured views (reported, never hidden). Returns the detailed
/// JSON entries (populated only when `json` output was requested).
async fn emit_view_rows(
    ssb: &Arc<Ssb>,
    report: &GovernanceCoverageReport,
    json: bool,
) -> Result<Vec<serde_json::Value>, GovernanceCoverageError> {
    let mut views_json: Vec<serde_json::Value> = Vec::new();
    for (view, status) in report.views() {
        let summary = view_summary_line(*view, status);
        ssb.write_payload(
            GOV_COVERAGE_AGENT_ID,
            GOV_COVERAGE_SUPERSTEP,
            finding_payload(*view, status, summary.clone()),
            vec![EVIDENCE_FILE_REF.to_string()],
        )
        .await
        .map_err(|e| GovernanceCoverageError::Ssb(format!("{e}")))?;

        if json {
            views_json.push(detailed_json_entry(*view, status));
        } else {
            println!("{summary}");
        }
    }
    Ok(views_json)
}

/// Human text tail, or the machine-readable per-view table when `json`.
fn print_output(
    json: bool,
    repo_root_display: &str,
    with_mutant_kill: bool,
    views_json: &[serde_json::Value],
    rows_written: usize,
) -> Result<(), GovernanceCoverageError> {
    if json {
        let out = serde_json::json!({
            "governance_coverage": {
                "run_id": GOV_COVERAGE_RUN_ID,
                "repo_root": repo_root_display,
                "with_mutant_kill": with_mutant_kill,
                "views": views_json,
                "rows_emitted": rows_written,
                "ssb_action_type": "governance_coverage_measured",
            }
        });
        println!("{}", serde_json::to_string_pretty(&out)?);
    } else {
        println!();
        println!(
            "  rows: {rows_written} GovernanceCoverageMeasured SSB rows written \
             (one per view, unmeasured included)"
        );
        println!("  see ssb.db action_type='governance_coverage_measured'");
        println!("  DONE (W-1 production entrypoint proof).");
    }
    Ok(())
}

/// Workflow root is the parent of `.elai_cc/`. CWD-first per
/// cross_lang_diff.py convention.
///
/// Canonical repo-root resolution — mirrors the private `resolve_repo_root`
/// in `drift_cmd.rs` byte-for-byte (that helper is module-private there and
/// widening its visibility is outside this slice's write set).
fn resolve_repo_root() -> PathBuf {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    if cwd.join(".elai_cc").is_dir() {
        return cwd;
    }
    // CWD is .elai_cc itself? Climb one.
    if cwd.file_name().and_then(|s| s.to_str()) == Some(".elai_cc") {
        if let Some(parent) = cwd.parent() {
            return parent.to_path_buf();
        }
    }
    cwd
}

/// Run the `elai governance-coverage` checker.
///
/// # Arguments
/// * `with_mutant_kill` — drive the REAL H-27 replay for V5 instead of the
///   unscored placeholder.
/// * `json`             — print JSON instead of human text.
///
/// Emits exactly one `GovernanceCoverageMeasured` SSB row per view (measured OR
/// unmeasured) and reports every view's state on stdout.
pub(crate) async fn run(
    with_mutant_kill: bool,
    json: bool,
) -> Result<(), GovernanceCoverageError> {
    let project_root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let sessions_dir = sessions_dir_path(&project_root);
    std::fs::create_dir_all(&sessions_dir)?;
    let db_path = sessions_dir.join("ssb.db");

    let cfg = SsbConfig {
        db_path: db_path.clone(),
        pool_size: None,
        project_id: GOV_COVERAGE_PROJECT_ID.to_string(),
        session_id: GOV_COVERAGE_SESSION_ID.to_string(),
    };
    let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
    let ssb = Arc::new(
        Ssb::new(cfg, clock).map_err(|e| GovernanceCoverageError::Ssb(format!("{e}")))?,
    );

    let repo_root_path = resolve_repo_root();
    let report = load_report(&repo_root_path, &ssb, &db_path, with_mutant_kill).await;
    let views_json = emit_view_rows(&ssb, &report, json).await?;
    print_output(
        json,
        &repo_root_path.display().to_string(),
        with_mutant_kill,
        &views_json,
        report.views().len(),
    )?;

    Ok(())
}

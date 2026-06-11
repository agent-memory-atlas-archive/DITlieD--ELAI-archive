//! E-6 — `elai mast report [--session <id>] [--limit N] [--json]` dispatcher.
//!
//! Produces a DETERMINISTIC, line-stable summary of MAST failure-mode counts
//! read from `audit_mast_tagged` SSB rows. The report enumerates all 14 MAST
//! modes (canonical FM-1.1 .. FM-3.3 order) crossed with the agent roles that
//! produced findings, plus per-mode totals — so the output is identical across
//! invocations on a fixed SSB seed (acceptance criterion: 14 modes, stable
//! ordering).
//!
//! # Wiring discipline
//!
//! producer  : `per_level_audit_dispatch::record_verdict` emits one
//!             `audit_mast_tagged` row per BLOCK verdict that maps to a MAST
//!             mode (wire-dark unless `ELAI_FAILURE_TRACKER_ENABLE=1`).
//! consumer  : this file — opens the canonical session `ssb.db` READ-ONLY via
//!             `SsbViewer`, scans the most-recent N `audit_mast_tagged` rows,
//!             aggregates (mast_mode, agent_role) -> count, prints the table.
//! ssb row   : reads `SsbActionType::AuditMastTagged`; writes NOTHING.
//!
//! The `--session` flag is an operator-facing run label echoed in the report
//! header; the `audit_mast_tagged` payload is run-global (it carries
//! `verdict_seq` + `agent_role`, not a session id), mirroring the read shape
//! of `commands_e6::get_mast_breakdown`. Absence of the SSB db is reported as
//! a zero-count table (this is an inspection command, not a wiring gate), exit 0.
//!
//! # No `.unwrap()` / `.expect()` in non-test code.

use std::collections::BTreeMap;
use std::path::Path;

use elai_core::paths::sessions_dir_path;
use elai_orchestrator::mast::MastMode;
use elai_orchestrator::ssb_viewer::SsbViewer;

/// Default scan window if `--limit` is not supplied.
const DEFAULT_LIMIT: usize = 1000;

/// One (mast_mode, agent_role) -> count cell of the aggregated report.
#[derive(Debug, Clone, PartialEq, Eq)]
struct MastCell {
    mast_mode: String,
    agent_role: String,
    count: u32,
}

/// Aggregated report payload. Rendered as either a deterministic text table
/// or (with `--json`) a stable JSON object.
#[derive(Debug, Clone, PartialEq, Eq)]
struct MastReport {
    /// Operator run label (the `--session` value, or "all").
    session: String,
    /// Per (mast_mode, agent_role) cells, sorted by mast_mode then role.
    cells: Vec<MastCell>,
    /// Per-mode totals in canonical MastMode declaration order (all 14
    /// modes always present, zero-filled).
    mode_totals: Vec<(String, u32)>,
    /// Sum of every cell count.
    grand_total: u32,
}

/// Aggregate a set of `(mast_mode, agent_role)` pairs into a [`MastReport`].
/// Pure — extracted so the deterministic-output contract is unit-testable
/// without a live SSB.
fn aggregate(session: &str, pairs: &[(String, String)]) -> MastReport {
    let mut grid: BTreeMap<(String, String), u32> = BTreeMap::new();
    let mut totals: BTreeMap<String, u32> = BTreeMap::new();
    let mut grand_total: u32 = 0;
    for (mode, role) in pairs {
        if mode.is_empty() {
            continue;
        }
        let cell = grid.entry((mode.clone(), role.clone())).or_insert(0);
        *cell = cell.saturating_add(1);
        let t = totals.entry(mode.clone()).or_insert(0);
        *t = t.saturating_add(1);
        grand_total = grand_total.saturating_add(1);
    }

    let cells: Vec<MastCell> = grid
        .into_iter()
        .map(|((mast_mode, agent_role), count)| MastCell {
            mast_mode,
            agent_role,
            count,
        })
        .collect();

    // Always emit all 14 modes in canonical declaration order (zero-filled),
    // so the report is line-stable regardless of which modes were observed.
    let mode_totals: Vec<(String, u32)> = MastMode::ALL
        .iter()
        .map(|m| {
            let tag = m.as_sql_str().to_owned();
            let n = totals.get(&tag).copied().unwrap_or(0);
            (tag, n)
        })
        .collect();

    MastReport {
        session: session.to_owned(),
        cells,
        mode_totals,
        grand_total,
    }
}

/// Read `audit_mast_tagged` rows from the read-only viewer and project out
/// the `(mast_mode, agent_role)` pairs. Mirrors the JSON-projection shape of
/// `commands_e6::get_mast_breakdown` (payload may be tag-wrapped or flat).
fn collect_pairs(viewer: &SsbViewer, limit: usize) -> Result<Vec<(String, String)>, String> {
    let raw_rows = viewer
        .recent_rows_of_type("audit_mast_tagged", limit)
        .map_err(|e| format!("ssb viewer error reading audit_mast_tagged: {e}"))?;
    let mut pairs: Vec<(String, String)> = Vec::new();
    for d in &raw_rows {
        let outer = match d.payload_json.as_object() {
            Some(o) => o,
            None => continue,
        };
        let obj = if let Some(inner) = outer.get("AuditMastTagged").and_then(|v| v.as_object()) {
            inner.clone()
        } else {
            outer.clone()
        };
        let mast_mode = obj
            .get("mast_mode")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_owned();
        let agent_role = obj
            .get("agent_role")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_owned();
        if mast_mode.is_empty() {
            continue;
        }
        pairs.push((mast_mode, agent_role));
    }
    Ok(pairs)
}

/// Render the report as a deterministic text block.
fn render_text(report: &MastReport) -> String {
    let mut out = String::new();
    out.push_str(&format!("mast report session={}\n", report.session));
    out.push_str(&format!("grand_total={}\n", report.grand_total));
    out.push_str("--- per (mast_mode, agent_role) ---\n");
    if report.cells.is_empty() {
        out.push_str("(no audit_mast_tagged rows; gate may be off or no BLOCK verdicts)\n");
    } else {
        for c in &report.cells {
            out.push_str(&format!("{}\t{}\t{}\n", c.mast_mode, c.agent_role, c.count));
        }
    }
    out.push_str("--- per mast_mode totals (all 14 modes) ---\n");
    for (mode, n) in &report.mode_totals {
        out.push_str(&format!("{mode}\t{n}\n"));
    }
    out
}

/// Render the report as stable JSON.
fn render_json(report: &MastReport) -> String {
    let cells: Vec<serde_json::Value> = report
        .cells
        .iter()
        .map(|c| {
            serde_json::json!({
                "mast_mode": c.mast_mode,
                "agent_role": c.agent_role,
                "count": c.count,
            })
        })
        .collect();
    let totals: Vec<serde_json::Value> = report
        .mode_totals
        .iter()
        .map(|(mode, n)| serde_json::json!({ "mast_mode": mode, "total": n }))
        .collect();
    let v = serde_json::json!({
        "session": report.session,
        "grand_total": report.grand_total,
        "cells": cells,
        "mode_totals": totals,
    });
    serde_json::to_string_pretty(&v).unwrap_or_else(|_| "{}".to_owned())
}

/// Build the report for `project_root`, opening the canonical session
/// `ssb.db` read-only. When the db file does not exist, returns a zero-count
/// report (no rows yet) rather than an error — `elai mast report` is an
/// inspection command, not a wiring gate.
fn build_report(
    project_root: &Path,
    session: &str,
    limit: usize,
) -> Result<MastReport, String> {
    let db_path = sessions_dir_path(project_root).join("ssb.db");
    if !db_path.exists() {
        return Ok(aggregate(session, &[]));
    }
    let viewer =
        SsbViewer::open(&db_path).map_err(|e| format!("could not open ssb.db read-only: {e}"))?;
    let pairs = collect_pairs(&viewer, limit)?;
    Ok(aggregate(session, &pairs))
}

/// `elai mast report` entrypoint. Resolves the project root from the current
/// working directory (mirrors `bench_seccodeplt::run`), builds the report,
/// and prints it. Exit 0 on a clean run.
pub fn run(session: Option<String>, limit: Option<usize>, json: bool) -> anyhow::Result<()> {
    let project_root =
        std::env::current_dir().map_err(|e| anyhow::anyhow!("could not determine cwd: {e}"))?;
    let session_label = session.unwrap_or_else(|| "all".to_owned());
    let cap = limit.unwrap_or(DEFAULT_LIMIT);
    let report =
        build_report(&project_root, &session_label, cap).map_err(|e| anyhow::anyhow!(e))?;
    let rendered = if json {
        render_json(&report)
    } else {
        render_text(&report)
    };
    println!("{rendered}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use elai_core::OsClock;
    use elai_orchestrator::ssb::{AuditMastTaggedPayload, Ssb, SsbConfig, SsbPayload};
    use std::sync::Arc;
    use tempfile::TempDir;

    #[test]
    fn aggregate_is_deterministic_and_has_all_14_modes() {
        let pairs = vec![
            ("fm_1_3_step_repetition".to_owned(), "auditor_babysteps".to_owned()),
            ("fm_1_3_step_repetition".to_owned(), "auditor_babysteps".to_owned()),
            ("fm_3_3_incorrect_verification".to_owned(), "auditor".to_owned()),
        ];
        let a = aggregate("s-1", &pairs);
        let b = aggregate("s-1", &pairs);
        assert_eq!(a, b, "aggregate must be deterministic");
        assert_eq!(a.mode_totals.len(), 14, "all 14 MAST modes always present");
        assert_eq!(a.grand_total, 3);
        // fm_1_3 total == 2
        let fm13 = a
            .mode_totals
            .iter()
            .find(|(m, _)| m == "fm_1_3_step_repetition")
            .map(|(_, n)| *n)
            .unwrap_or(0);
        assert_eq!(fm13, 2);
        // rendering is stable across calls
        assert_eq!(render_text(&a), render_text(&b));
    }

    #[test]
    fn build_report_missing_db_is_zero_count() {
        let dir = TempDir::new().expect("tempdir");
        // No .elai_cc/sessions/ssb.db created -> zero report, no error.
        let r = build_report(dir.path(), "all", 100).expect("missing db must not error");
        assert_eq!(r.grand_total, 0);
        assert_eq!(r.cells.len(), 0);
        assert_eq!(r.mode_totals.len(), 14);
    }

    /// Off-agent read: write a real audit_mast_tagged row through the SSB
    /// writer, then build the report through the SAME read path the CLI uses
    /// and confirm the row is counted.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn build_report_counts_emitted_row() {
        let dir = TempDir::new().expect("tempdir");
        let db_path = sessions_dir_path(dir.path()).join("ssb.db");
        std::fs::create_dir_all(db_path.parent().expect("parent")).expect("mkdir");

        let cfg = SsbConfig {
            db_path: db_path.clone(),
            pool_size: Some(2),
            project_id: "test".into(),
            session_id: "s-mast".into(),
        };
        let clock: Arc<dyn elai_core::ClockProvider> = Arc::new(OsClock);
        let ssb = Ssb::new(cfg, clock).expect("ssb new");
        ssb.write_payload(
            "auditor-babysteps",
            0,
            SsbPayload::AuditMastTagged(AuditMastTaggedPayload {
                verdict_seq: 1,
                mast_mode: "fm_1_3_step_repetition".into(),
                confidence_band: "medium".into(),
                agent_role: "auditor_babysteps".into(),
                ts_ms: 1_700_000_000_000,
            }),
            Vec::new(),
        )
        .await
        .expect("write_payload");

        // Read back through the CLI's own build_report path.
        let report = build_report(dir.path(), "all", 100).expect("build_report");
        assert_eq!(report.grand_total, 1, "the emitted row must be counted");
        assert_eq!(report.cells.len(), 1);
        assert_eq!(report.cells[0].mast_mode, "fm_1_3_step_repetition");
        assert_eq!(report.cells[0].agent_role, "auditor_babysteps");
    }
}

//! H-41 — `elai checker-gaps` CLI handler.
//!
//! Reads `checker_gap_detected` SSB rows from the canonical `ssb.db`, aggregates
//! them by `(claim_type, complexity_taxon)`, and prints a ranked backlog sorted
//! by `frequency × taxon_priority` descending. HIGH-priority rows (NP-complete /
//! undecidable taxa) appear at the top — those are the claim-classes that MUST
//! be routed to an external solver or refused (never LLM-graded, RD-3).
//!
//! SSB read path: reads `checker_gap_detected` rows from the canonical
//! `ssb.db` and aggregates them into a ranked backlog. The writer is
//! `route_claim_with_gap_env` in agent_pool.rs, called from
//! `dispatch_claim_routing` in task_runner.rs (H-41 production entrypoint).
//! No unit-test shim.
//!
//! Grounding:
//! - Kambhampati arXiv:2402.08115 — Mode-A coverage is the verification lever.
//! - Xu arXiv:2401.11817 Table 3 — NP-complete / undecidable = never-LLM-grade.
//! - H-41 plan: docs/thefinalglance/H-plan/H-41_checker_coverage_gap_instrument.md

use std::path::PathBuf;

use elai_core::paths::sessions_dir_path;
use elai_orchestrator::ssb_viewer::SsbViewer;
use elai_orchestrator::{checker_gap_backlog, mode_a_coverage, CheckerGapBacklogRow};

const DEFAULT_LIMIT: usize = 50;

/// Run `elai checker-gaps [--limit N] [--json]`.
///
/// Resolves the project root from `std::env::current_dir()`. Opens the
/// canonical `ssb.db` read-only; returns cleanly when the file does not
/// exist (no gaps recorded yet).
pub fn run(limit: Option<usize>, json: bool) -> anyhow::Result<()> {
    let project_root = std::env::current_dir()
        .map_err(|e| anyhow::anyhow!("could not determine cwd: {e}"))?;
    let effective_limit = limit.unwrap_or(DEFAULT_LIMIT);

    let db_path = sessions_dir_path(&project_root).join("ssb.db");
    if !db_path.exists() {
        if json {
            println!("[]");
        } else {
            println!("checker-gaps: no ssb.db found — no gap rows recorded yet.");
        }
        return Ok(());
    }

    let viewer = SsbViewer::open(&db_path)
        .map_err(|e| anyhow::anyhow!("checker-gaps: could not open ssb.db: {e}"))?;

    let backlog = checker_gap_backlog(&viewer, effective_limit);
    let coverage = mode_a_coverage(&viewer, "all", 0);

    if json {
        print_json(&backlog, coverage.mode_a_fraction)?;
    } else {
        print_table(&backlog, coverage.mode_a_fraction, coverage.constructive_count,
            coverage.total_factual_computational);
    }
    Ok(())
}

fn print_json(backlog: &[CheckerGapBacklogRow], mode_a_fraction: f64) -> anyhow::Result<()> {
    // Build a minimal JSON output without a third-party serialiser.
    // The dashboard uses the Tauri command (commands_h41.rs); this CLI
    // path is for operator inspection and CI integration.
    let rows_json: Vec<String> = backlog
        .iter()
        .map(|r| {
            format!(
                r#"{{"claim_type":"{ct}","complexity_taxon":"{tx}","count":{c},"priority":{p}}}"#,
                ct = r.claim_type,
                tx = r.complexity_taxon,
                c = r.count,
                p = r.priority
            )
        })
        .collect();
    let body = rows_json.join(",");
    println!(
        r#"{{"mode_a_fraction":{frac:.4},"backlog":[{body}]}}"#,
        frac = mode_a_fraction
    );
    Ok(())
}

fn print_table(
    backlog: &[CheckerGapBacklogRow],
    mode_a_fraction: f64,
    constructive: u64,
    total: u64,
) {
    println!(
        "Mode-A coverage: {:.1}%  ({} Constructive / {} Factual+Computational)",
        mode_a_fraction * 100.0,
        constructive,
        total
    );
    if backlog.is_empty() {
        println!("No checker-gap rows recorded (ELAI_CHECKER_GAP_ENABLE=1 required on writer).");
        return;
    }
    println!(
        "\n{:<20} {:<32} {:>6} {:>8}",
        "CLAIM_TYPE", "COMPLEXITY_TAXON", "COUNT", "PRIORITY"
    );
    println!("{}", "-".repeat(72));
    for row in backlog {
        let prio_label = if row.priority == 2 { "HIGH" } else { "normal" };
        println!(
            "{:<20} {:<32} {:>6} {:>8}",
            row.claim_type, row.complexity_taxon, row.count, prio_label
        );
    }
}

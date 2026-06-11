//! H-34 -- `elai bench mcp-schema-budget [--json]` CLI driver.
//!
//! Surfaces
//! [`elai_orchestrator::benchmarks::mcp_schema_budget_runner::McpSchemaBudgetRunner`]
//! as a user-facing subcommand + CI hook. The benchmark drives the offline
//! MCP-schema token-budget computation over the embedded fixture corpus
//! (20 tools across local-rag / pod-lifecycle / notion) and computes:
//!
//!   * `baseline_schema_tokens_per_turn` -- all tools injected eagerly.
//!   * `injected_schema_tokens_per_turn` -- floor-only set (defer-load minimum).
//!   * `reduction_tokens` -- the per-turn token savings from defer-load.
//!
//! Both token counts are COMPUTED from the corpus, never literals.
//! `baseline > injected > 0` is the non-degeneracy invariant.
//! `injected <= INJECTED_CAP_TOKENS (500)` is the cap invariant.
//!
//! # Wiring discipline
//!
//! producer  : `McpSchemaBudgetRunner::run` (orchestrator-side, deterministic).
//! consumer  : this file -- runs the eval, writes a timestamped
//!             `mcp-schema-budget-<unix_ms>.json` report under
//!             `<cwd>/benchmarks/mcp_schema_budget/`, and emits EXACTLY ONE
//!             `McpSchemaBudgetMeasured` SSB beacon (default-ON on run; NO
//!             `ELAI_*_ENABLE` gate -- this is a measure-not-gate instrument).
//! ssb row   : `SsbActionType::McpSchemaBudgetMeasured` (one per run).
//!
//! # Exit codes
//!
//! 0 = run completed (report written + SSB row emitted).
//! 1 = infrastructural error (malformed corpus, cap exceeded, SSB writer
//!     failure, report I/O).
//!
//! # No `.unwrap()` / `.expect()` in non-test code.

use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use elai_core::paths::sessions_dir_path;
use elai_core::OsClock;
use elai_orchestrator::benchmarks::mcp_schema_budget_runner::{
    McpSchemaBudgetError, McpSchemaBudgetReport, McpSchemaBudgetRunner,
};
use elai_orchestrator::ssb::McpSchemaBudgetMeasuredPayload;
use elai_orchestrator::{Ssb, SsbConfig, SsbPayload};
use thiserror::Error;

// ---------------------------------------------------------------------------
// Error type (thiserror per CLAUDE.md anti-stale rule).
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum BenchMcpSchemaBudgetError {
    /// The embedded corpus / runner failed to load or run.
    #[error("mcp_schema_budget runner error: {0}")]
    Runner(#[from] McpSchemaBudgetError),
    /// Could not determine the current working directory for the report dir.
    #[error("mcp_schema_budget could not determine cwd: {source_msg}")]
    Cwd { source_msg: String },
    /// Could not create the on-disk report directory.
    #[error("mcp_schema_budget could not create report dir {path}: {source_msg}")]
    ReportDir { path: String, source_msg: String },
    /// Generic I/O failure writing the JSON report.
    #[error("mcp_schema_budget could not write report to {path}: {source_msg}")]
    OutputIo { path: String, source_msg: String },
    /// JSON serialisation failure when rendering the report.
    #[error("mcp_schema_budget could not serialise report: {source_msg}")]
    Serialise { source_msg: String },
    /// Building the SSB async runtime failed.
    #[error("mcp_schema_budget could not build async runtime: {source_msg}")]
    Runtime { source_msg: String },
    /// Opening the SSB single-writer failed.
    #[error("mcp_schema_budget could not open SSB writer: {source_msg}")]
    SsbOpen { source_msg: String },
    /// Writing the McpSchemaBudgetMeasured beacon failed.
    #[error("mcp_schema_budget could not write SSB beacon: {source_msg}")]
    SsbWrite { source_msg: String },
}

// ---------------------------------------------------------------------------
// Sync CLI entry.
// ---------------------------------------------------------------------------

/// CLI entry -- runs the MCP schema-budget instrument, writes a timestamped
/// JSON report, emits one `McpSchemaBudgetMeasured` SSB beacon, and prints a
/// summary (or the full report JSON with `--json`).
///
/// Returns `Ok(())` on a clean run (rc=0). On a handled error it prints a
/// `MCP_SCHEMA_BUDGET ABORT:` line to stderr and calls `std::process::exit(1)`.
pub(crate) fn run(json: bool) -> anyhow::Result<()> {
    match run_inner(json) {
        Ok(()) => Ok(()),
        Err(e) => {
            eprintln!("MCP_SCHEMA_BUDGET ABORT: {e}");
            std::process::exit(1);
        }
    }
}

/// Core -- constructs the REAL runner, runs the eval, writes the report, emits
/// the beacon. Public so an integration test can drive it without the process
/// exit.
pub fn run_inner(json: bool) -> Result<(), BenchMcpSchemaBudgetError> {
    // 1. Construct the REAL runner from the embedded fixture corpus + run the
    //    budget computation (offline, no model, no network).
    let runner = McpSchemaBudgetRunner::from_embedded()?;
    let report = runner.run()?;

    // 2. Write the timestamped JSON report under
    //    <cwd>/benchmarks/mcp_schema_budget/.
    let project_root =
        std::env::current_dir().map_err(|e| BenchMcpSchemaBudgetError::Cwd {
            source_msg: e.to_string(),
        })?;
    let report_path = write_report(&project_root, &report)?;

    // 3. Emit EXACTLY ONE McpSchemaBudgetMeasured SSB beacon (default-ON; no
    //    ELAI_*_ENABLE gate). Ssb::new spawns the single-writer task and MUST be
    //    constructed inside rt.block_on (else it panics "no reactor running" --
    //    only observable on a live run; cargo check passes regardless).
    emit_mcp_schema_budget_measured(&project_root, &report)?;

    // 4. Summary line, plus the full report JSON when --json is set.
    println!(
        "MCP_SCHEMA_BUDGET suite={} total_tools={} floor_tools={} \
         baseline={} injected={} reduction={} deferred={} \
         seed={} corpus_blake3={}; report written to {}",
        report.suite,
        report.total_tools,
        report.floor_tools,
        report.baseline_schema_tokens_per_turn,
        report.injected_schema_tokens_per_turn,
        report.reduction_tokens,
        report.deferred_resolves,
        report.seed,
        report.corpus_blake3,
        report_path.display(),
    );
    if json {
        let body = serde_json::to_string_pretty(&report).map_err(|e| {
            BenchMcpSchemaBudgetError::Serialise {
                source_msg: e.to_string(),
            }
        })?;
        println!("{body}");
    }
    Ok(())
}

/// Write the report JSON to
/// `<project_root>/benchmarks/mcp_schema_budget/mcp-schema-budget-<unix_ms>.json`
/// and return the written path.
fn write_report(
    project_root: &std::path::Path,
    report: &McpSchemaBudgetReport,
) -> Result<PathBuf, BenchMcpSchemaBudgetError> {
    let report_dir = project_root.join("benchmarks").join("mcp_schema_budget");
    fs::create_dir_all(&report_dir).map_err(|e| BenchMcpSchemaBudgetError::ReportDir {
        path: report_dir.display().to_string(),
        source_msg: e.to_string(),
    })?;

    // Unix-millis timestamp for the file name only (does not enter the report
    // payload -- report stays byte-reproducible under ELAI_SEED). Falls back
    // to 0 rather than panicking on a clock earlier than the epoch.
    let unix_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or_default();
    let report_path = report_dir.join(format!("mcp-schema-budget-{unix_ms}.json"));

    let body = serde_json::to_string_pretty(report).map_err(|e| {
        BenchMcpSchemaBudgetError::Serialise {
            source_msg: e.to_string(),
        }
    })?;
    fs::write(&report_path, &body).map_err(|e| BenchMcpSchemaBudgetError::OutputIo {
        path: report_path.display().to_string(),
        source_msg: e.to_string(),
    })?;
    Ok(report_path)
}

/// Open the canonical session `ssb.db` and emit ONE `McpSchemaBudgetMeasured`
/// beacon, awaited (durable). Mirrors `bench_staleness::emit_staleness_eval_measured`.
fn emit_mcp_schema_budget_measured(
    project_root: &std::path::Path,
    report: &McpSchemaBudgetReport,
) -> Result<(), BenchMcpSchemaBudgetError> {
    let db_path = sessions_dir_path(project_root).join("ssb.db");
    if let Some(parent) = db_path.parent() {
        fs::create_dir_all(parent).map_err(|e| BenchMcpSchemaBudgetError::ReportDir {
            path: parent.display().to_string(),
            source_msg: e.to_string(),
        })?;
    }

    let cfg = SsbConfig {
        db_path,
        pool_size: None,
        project_id: "default".to_string(),
        session_id: "cli-bench-mcp-schema-budget".to_string(),
    };
    let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| BenchMcpSchemaBudgetError::Runtime {
            source_msg: e.to_string(),
        })?;

    let payload = McpSchemaBudgetMeasuredPayload {
        suite: report.suite.clone(),
        baseline_schema_tokens_per_turn: report.baseline_schema_tokens_per_turn,
        injected_schema_tokens_per_turn: report.injected_schema_tokens_per_turn,
        reduction_tokens: report.reduction_tokens,
        deferred_resolves: report.deferred_resolves,
        seed: report.seed,
        corpus_blake3: report.corpus_blake3.clone(),
    };

    rt.block_on(async move {
        let ssb = Ssb::new(cfg, clock).map_err(|e| BenchMcpSchemaBudgetError::SsbOpen {
            source_msg: e.to_string(),
        })?;
        ssb.write_payload(
            "cli:bench-mcp-schema-budget",
            0,
            SsbPayload::McpSchemaBudgetMeasured(payload),
            Vec::new(),
        )
        .await
        .map_err(|e| BenchMcpSchemaBudgetError::SsbWrite {
            source_msg: e.to_string(),
        })?;
        Ok::<(), BenchMcpSchemaBudgetError>(())
    })
}

// ---------------------------------------------------------------------------
// Unit tests.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use elai_orchestrator::benchmarks::mcp_schema_budget_runner::McpSchemaBudgetRunner;

    #[test]
    fn write_report_emits_json_under_benchmarks_dir() {
        let runner =
            McpSchemaBudgetRunner::from_embedded().expect("embedded corpus parses");
        let report = runner.run().expect("clean run");
        let dir = tempfile::TempDir::new().expect("tempdir");
        let path = write_report(dir.path(), &report).expect("write report");

        assert!(path.exists(), "report file must exist on disk");
        let on_disk = path.to_string_lossy();
        assert!(
            on_disk.contains("benchmarks") && on_disk.contains("mcp_schema_budget"),
            "report must live under benchmarks/mcp_schema_budget/, got {on_disk}"
        );

        let text = fs::read_to_string(&path).expect("read report");
        let parsed: McpSchemaBudgetReport = serde_json::from_str(&text)
            .expect("report round-trips as McpSchemaBudgetReport");

        // Non-degeneracy invariant.
        assert!(
            parsed.baseline_schema_tokens_per_turn > parsed.injected_schema_tokens_per_turn,
            "baseline must exceed injected"
        );
        assert_eq!(parsed.corpus_blake3.len(), 64, "blake3 hex is 64 chars");
    }

    #[test]
    fn payload_mirrors_report_fields() {
        let runner =
            McpSchemaBudgetRunner::from_embedded().expect("parse");
        let report = runner.run().expect("run");
        let payload = McpSchemaBudgetMeasuredPayload {
            suite: report.suite.clone(),
            baseline_schema_tokens_per_turn: report.baseline_schema_tokens_per_turn,
            injected_schema_tokens_per_turn: report.injected_schema_tokens_per_turn,
            reduction_tokens: report.reduction_tokens,
            deferred_resolves: report.deferred_resolves,
            seed: report.seed,
            corpus_blake3: report.corpus_blake3.clone(),
        };
        assert_eq!(payload.suite, report.suite);
        assert_eq!(
            payload.baseline_schema_tokens_per_turn,
            report.baseline_schema_tokens_per_turn
        );
        assert_eq!(
            payload.injected_schema_tokens_per_turn,
            report.injected_schema_tokens_per_turn
        );
        assert_eq!(payload.reduction_tokens, report.reduction_tokens);
        assert_eq!(payload.corpus_blake3, report.corpus_blake3);
    }
}

//! WIRING-FIX-ELAI Phase 2 (SP3) — `elai wiring-coverage` CLI driver.
//!
//! Thin shell over [`elai_orchestrator::WiringCoverageCmd::run`]. Resolves the
//! CLI-flag inputs, runs the Layer-3 coverage gate, renders the report, and
//! exits 1 on any violation OR on any un-measurable error (R2 fail-closed:
//! an error is NOT a clean pass). Mirrors `crate::wiring_audit::run`.
//!
//! SSB note (ADR-1): `WiringCoverageReport::ssb_payloads` carries the
//! `wiring_evidence` / `wiring_violation` rows the run produced. The sync CLI
//! has no live async `Ssb` write-loop, so it RENDERS the row summary (count +
//! leg) rather than persisting — exactly as `wiring_audit::run` renders its
//! evidence/violation payloads. An orchestrator caller (the WAVE-4 D-EV gate)
//! holds a live `Ssb` and persists via `Ssb::write_payload`.

use std::path::PathBuf;

use elai_orchestrator::{WiringCoverageCmd, WiringCoverageError, WiringCoverageReport};

/// CLI entry point — sync wrapper (mirrors wiring_audit::run). The orchestrator
/// core owns the gate semantics; this file owns flag resolution + report
/// rendering + process exit.
pub(crate) fn run(
    manifest_path: Option<String>,
    diff_base: Option<String>,
    output_format: super::CliOutputFormat,
) -> anyhow::Result<()> {
    let project_root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let manifest = manifest_path
        .map(PathBuf::from)
        .unwrap_or_else(|| project_root.join("smoke-manifest.toml"));
    let diff_base = diff_base.unwrap_or_else(|| "HEAD".to_string());

    let cmd = WiringCoverageCmd {
        source_root: project_root.clone(),
        manifest_path: manifest,
        diff_base,
        static_only: false,
    };

    let report = match cmd.run() {
        Ok(r) => r,
        // R2: any inability to measure is a hard FAIL surfaced to the shell as
        // exit 1 — NEVER a silent clean pass. Each variant is enumerated
        // explicitly (no `_ =>` swallow).
        Err(e @ WiringCoverageError::MeasurementAbsent(_))
        | Err(e @ WiringCoverageError::ManifestMissing(_))
        | Err(e @ WiringCoverageError::Parse(_))
        | Err(e @ WiringCoverageError::Io(_))
        | Err(e @ WiringCoverageError::GitDiffFailed(_))
        | Err(e @ WiringCoverageError::BuildFailed { .. })
        | Err(e @ WiringCoverageError::SmokeFailed { .. })
        | Err(e @ WiringCoverageError::CoverageExportFailed { .. }) => {
            eprintln!("WIRING COVERAGE FAIL (un-measurable): {e}");
            std::process::exit(1);
        }
    };

    render_report(output_format, &report);
    if !report.is_clean() {
        std::process::exit(1);
    }
    Ok(())
}

fn render_report(output: super::CliOutputFormat, report: &WiringCoverageReport) {
    match output {
        super::CliOutputFormat::Json => {
            let json = serde_json::json!({
                "audit_id": report.audit_id.to_string(),
                "functions_total": report.functions_total,
                "new_functions_gated": report.new_functions_gated,
                "violation_count": report.violations.len(),
                "violations": report.violations,
                "suppressed": report.suppressed,
                "ssb_row_count": report.ssb_payloads.len(),
                "ssb_leg": elai_orchestrator::LAYER3_LEG,
                "clean": report.is_clean(),
            });
            println!("{}", serde_json::Value::to_string(&json));
        }
        // Text and Toml both render the human-readable summary; the Layer-3
        // gate has no native TOML schema so Toml maps to the same lines (a
        // documented, explicit arm — not a silent fallthrough).
        super::CliOutputFormat::Text | super::CliOutputFormat::Toml => {
            for v in &report.violations {
                println!("VIOLATION {v:?}");
            }
            for s in &report.suppressed {
                println!("SUPPRESSED {s}");
            }
            println!(
                "SSB rows produced: {} (leg={}, audit_id={})",
                report.ssb_payloads.len(),
                elai_orchestrator::LAYER3_LEG,
                report.audit_id
            );
            if report.is_clean() {
                println!(
                    "WIRING COVERAGE PASS (functions={}, new_gated={})",
                    report.functions_total, report.new_functions_gated
                );
            } else {
                println!(
                    "WIRING COVERAGE FAIL ({} violations; new_gated={})",
                    report.violations.len(),
                    report.new_functions_gated
                );
            }
        }
    }
}

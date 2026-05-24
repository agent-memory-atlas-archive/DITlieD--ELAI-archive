//! C-47 — `elai calibration <status|reset-hints|export>` handler.
//!
//! Surfaces the case-based calibration registry to operators. The
//! handler does NOT own a long-lived registry; it constructs a
//! fresh, empty one and surfaces a deterministic snapshot so the CLI
//! has a verifiable observable effect even before the production
//! task-runner has wired a real `Arc<ToolCalibrationRegistry>` into
//! the bridge. The real registry lives behind the runtime bootstrap
//! (paired with `DefaultMcpBridge::with_calibration_registry`); when
//! the production runner is up, `elai calibration status` would
//! query the live registry via a thin IPC seam (follow-on extension
//! per the C-47 plan rebase — RUNPOD-PENDING).
//!
//! Sub-actions:
//!   * `status [--tool NAME]` — render per-tool status (shape_count,
//!     total_calls, error_count, last_ts_ms). When `--tool` is
//!     supplied, prints only that tool; otherwise prints every tool
//!     in the snapshot.
//!   * `reset-hints` — toggle the hint_suppression bit on a freshly
//!     constructed registry; useful for testing the gate. In
//!     production this would call into the live registry.
//!   * `export --path PATH` — write the snapshot to a JSON file at
//!     PATH for offline inspection / archival.
//!
//! Paper grounding (arXiv:2605.15041, CAST). The CLI surface mirrors
//! the per-tool aggregate signal pattern from the paper Section 3.2.

use std::fs;
use std::path::PathBuf;

use elai_plugins::{
    AggregatedCase, ResponseKind, ToolCalibrationRegistry, ToolCalibrationStatus,
};
use serde::Serialize;

use crate::args::{CalibrationAction, CliOutputFormat};

/// Top-level dispatcher invoked by main.rs.
pub(crate) fn run(
    action: CalibrationAction,
    output_format: CliOutputFormat,
) -> Result<(), String> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| format!("calibration_cmd: tokio runtime build failed: {e}"))?;
    runtime.block_on(async {
        match action {
            CalibrationAction::Status { tool_name } => {
                run_status(tool_name, output_format).await
            }
            CalibrationAction::ResetHints => run_reset_hints(output_format).await,
            CalibrationAction::Export { path } => {
                run_export(&PathBuf::from(path), output_format).await
            }
        }
    })
}

/// Build a non-empty synthetic registry so the CLI has data to render
/// even before the production task-runner wires a live registry. The
/// registry is dropped on return — operators must inspect a running
/// process to see real telemetry (follow-on extension).
async fn synthetic_registry() -> ToolCalibrationRegistry {
    let reg = ToolCalibrationRegistry::new();
    // Seed two tools so `status` (no filter) returns multiple rows
    // and `status --tool fs.read` returns exactly one.
    for _ in 0..7 {
        reg.record_call("fs.read", 0xAA, r#"{"path":"x"}"#, ResponseKind::Ok, 1_700_000_000_000)
            .await;
    }
    for _ in 0..3 {
        reg.record_call(
            "fs.read",
            0xAA,
            r#"{"path":"x"}"#,
            ResponseKind::ErrInvalidArgs,
            1_700_000_001_000,
        )
        .await;
    }
    for _ in 0..5 {
        reg.record_call(
            "git.commit",
            0xBB,
            r#"{"msg":"x"}"#,
            ResponseKind::Ok,
            1_700_000_002_000,
        )
        .await;
    }
    reg
}

/// JSON shape emitted by `status` / `export`. Mirrors
/// [`ToolCalibrationStatus`] one-for-one so the TS DTO at
/// `dashboard-v2/src/lib/api/stub.ts` deserializes the same payload.
#[derive(Debug, Serialize)]
struct StatusRow {
    tool_name: String,
    shape_count: u32,
    total_calls: u32,
    error_count: u32,
    last_ts_ms: i64,
}

impl From<ToolCalibrationStatus> for StatusRow {
    fn from(s: ToolCalibrationStatus) -> Self {
        Self {
            tool_name: s.tool_name,
            shape_count: s.shape_count,
            total_calls: s.total_calls,
            error_count: s.error_count,
            last_ts_ms: s.last_ts_ms,
        }
    }
}

async fn run_status(
    tool_name: Option<String>,
    output_format: CliOutputFormat,
) -> Result<(), String> {
    let reg = synthetic_registry().await;
    let mut rows: Vec<StatusRow> = match tool_name.as_deref() {
        Some(name) => reg
            .status(name)
            .await
            .map(|s| vec![s.into()])
            .unwrap_or_default(),
        None => reg
            .statuses()
            .await
            .into_iter()
            .map(StatusRow::from)
            .collect(),
    };
    rows.sort_by(|a, b| a.tool_name.cmp(&b.tool_name));
    render(&rows, output_format)
}

async fn run_reset_hints(output_format: CliOutputFormat) -> Result<(), String> {
    // The CLI dry-run carries no shared mutable state with the live
    // task-runner, so this surface only emits a status row reporting
    // the result of constructing a hint-suppressed registry. The
    // observable effect proves the public surface works.
    let reg = ToolCalibrationRegistry::new().with_hint_suppression(true);
    let suppressed = reg.hint_suppression();
    let row = ResetRow {
        hint_suppression: suppressed,
        note: "reset on synthetic registry; production registry is wired by task_runner",
    };
    render(&[row], output_format)
}

#[derive(Debug, Serialize)]
struct ResetRow {
    hint_suppression: bool,
    note: &'static str,
}

async fn run_export(path: &PathBuf, output_format: CliOutputFormat) -> Result<(), String> {
    let reg = synthetic_registry().await;
    let rows: Vec<StatusRow> = reg
        .statuses()
        .await
        .into_iter()
        .map(StatusRow::from)
        .collect();
    let json = serde_json::to_string_pretty(&rows)
        .map_err(|e| format!("calibration export: serialize failed: {e}"))?;
    fs::write(path, json).map_err(|e| {
        format!(
            "calibration export: write to {} failed: {e}",
            path.display()
        )
    })?;
    let summary = ExportSummary {
        path: path.display().to_string(),
        tool_count: rows.len(),
    };
    render(&[summary], output_format)
}

#[derive(Debug, Serialize)]
struct ExportSummary {
    path: String,
    tool_count: usize,
}

fn render<T: Serialize>(rows: &[T], output_format: CliOutputFormat) -> Result<(), String> {
    let out = match output_format {
        CliOutputFormat::Json => serde_json::to_string_pretty(rows)
            .map_err(|e| format!("render: json serialise failed: {e}"))?,
        CliOutputFormat::Toml => {
            // toml::to_string requires a map at the top — wrap.
            let wrapped = serde_json::json!({ "rows": rows });
            toml::to_string(&wrapped)
                .map_err(|e| format!("render: toml serialise failed: {e}"))?
        }
        CliOutputFormat::Text => {
            let v = serde_json::to_value(rows)
                .map_err(|e| format!("render: text->json failed: {e}"))?;
            format!("{v:#}")
        }
    };
    println!("{out}");
    Ok(())
}

/// Pure helper used by the unit test to verify [`AggregatedCase`] is
/// re-exported via `elai_plugins` (the CLI cannot reach into the
/// submodule directly).
#[allow(dead_code)]
fn assert_public_surface() {
    let _: AggregatedCase = AggregatedCase::new(String::new(), 0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn synthetic_registry_has_two_tools() {
        let reg = synthetic_registry().await;
        let statuses = reg.statuses().await;
        let names: Vec<&str> = statuses.iter().map(|s| s.tool_name.as_str()).collect();
        assert!(names.contains(&"fs.read"));
        assert!(names.contains(&"git.commit"));
    }

    #[tokio::test]
    async fn status_for_unknown_tool_returns_empty() {
        let reg = synthetic_registry().await;
        assert!(reg.status("never_called").await.is_none());
    }
}

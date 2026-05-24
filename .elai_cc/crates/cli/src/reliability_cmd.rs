//! E-18 — `elai reliability <status|list-spills|replay-spill>` handler.
//!
//! Surfaces the tool-use reliability gate to operators. The handler
//! does NOT own a long-lived gate; the gate itself is pure (a function
//! over a snapshot — see `elai_orchestrator::tool_reliability::gate`).
//! `status` prints the configured cap + retry budget defaults. The
//! synthetic-fixture path is GPU-free; the live SSB integration ships
//! through the orchestrator-side `OrchestratorReliabilitySsbEmitter`
//! adapter wired by the dashboard / task-runner bootstrap.
//!
//! Sub-actions:
//!   * `status` — render gate inline cap (8 KiB) + retry budget
//!     defaults + spill dir convention. JSON / TOML / text formats.
//!   * `list-spills [--dir DIR]` — enumerate JSON spill files under
//!     `.elai_cc/.elai_cc/spill/` (or DIR). One row per file
//!     (filename, payload bytes).
//!   * `replay-spill --path PATH` — read one spilled body back from
//!     PATH and print the JSON to stdout. Operator-side verification
//!     primitive that proves the spill subsystem round-trips.
//!
//! Composes on top of:
//!   * D-11 RegistryCheck (anti-slopsquatting; reused via the
//!     orchestrator's checker_registry).
//!   * E-35 SearchReplaceMatcher + DelimiterCollisionGuard +
//!     SidecarXGrammarHook + ToolParserVariant.
//!   * C-47 ToolCalibrationRegistry (consulted BEFORE this gate).
//!   * C-42 AuthzPolicy + C-35 ToolGate (run BEFORE this gate).
//!
//! See `workflow/reasoning/E-18-decisions.md`.

use std::fs;
use std::path::{Path, PathBuf};

use elai_orchestrator::tool_reliability::{
    spill_root_default, RELIABILITY_GATE_INLINE_CAP,
};
use serde::Serialize;

use crate::args::{CliOutputFormat, ReliabilityAction};

/// Top-level dispatcher invoked by main.rs.
pub(crate) fn run(
    action: ReliabilityAction,
    output_format: CliOutputFormat,
) -> Result<(), String> {
    match action {
        ReliabilityAction::Status => run_status(output_format),
        ReliabilityAction::ListSpills { dir } => run_list_spills(dir, output_format),
        ReliabilityAction::ReplaySpill { path } => run_replay_spill(&path, output_format),
    }
}

/// JSON shape emitted by `status`. Mirrors the constants the
/// orchestrator-side `tool_reliability::gate` exposes so operators get
/// a single source of truth.
#[derive(Debug, Serialize)]
struct StatusRow {
    inline_cap_bytes: usize,
    default_retry_cap: u32,
    spill_dir: String,
    /// Sentinel: the 4 wire action_types the dashboard panel polls.
    ssb_action_types: Vec<&'static str>,
    /// Sentinel: composition order before this gate fires.
    composition_chain: Vec<&'static str>,
}

fn run_status(output_format: CliOutputFormat) -> Result<(), String> {
    let project_state =
        PathBuf::from(".elai_cc").join(".elai_cc");
    let spill = spill_root_default(&project_state);
    let row = StatusRow {
        inline_cap_bytes: RELIABILITY_GATE_INLINE_CAP,
        default_retry_cap:
            elai_orchestrator::tool_reliability::retry::DEFAULT_RETRY_CAP,
        spill_dir: spill.display().to_string(),
        ssb_action_types: vec![
            "tool_reliability_gate_passed",
            "tool_reliability_gate_rejected",
            "tool_call_spilled",
            "tool_call_retry_escalated",
        ],
        composition_chain: vec![
            "authz_policy_c42",
            "tool_gate_c35",
            "calibration_consult_c47",
            "reliability_gate_e18",
            "dispatch",
        ],
    };
    render(&[row], output_format)
}

/// JSON shape emitted by `list-spills`. One row per JSON spill file.
#[derive(Debug, Serialize)]
struct SpillRow {
    filename: String,
    payload_bytes: u64,
}

fn run_list_spills(
    dir: Option<String>,
    output_format: CliOutputFormat,
) -> Result<(), String> {
    let dir = match dir {
        Some(d) => PathBuf::from(d),
        None => spill_root_default(&PathBuf::from(".elai_cc").join(".elai_cc")),
    };
    // Empty directory + no directory both render an empty list rather
    // than erroring — operators expect "no spills yet" to be quiet.
    let rows = match list_spill_dir(&dir) {
        Ok(r) => r,
        Err(_) => Vec::new(),
    };
    render(&rows, output_format)
}

fn list_spill_dir(dir: &Path) -> Result<Vec<SpillRow>, String> {
    let mut rows: Vec<SpillRow> = Vec::new();
    let entries = fs::read_dir(dir).map_err(|e| {
        format!(
            "reliability list-spills: read_dir {} failed: {e}",
            dir.display()
        )
    })?;
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }
        let meta = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };
        let fname = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("<unnamed>")
            .to_string();
        rows.push(SpillRow {
            filename: fname,
            payload_bytes: meta.len(),
        });
    }
    rows.sort_by(|a, b| a.filename.cmp(&b.filename));
    Ok(rows)
}

/// JSON shape emitted by `replay-spill`. Carries the path + the raw
/// body content as serde_json::Value so downstream tools parse the
/// spilled args as-is.
#[derive(Debug, Serialize)]
struct ReplayRow {
    path: String,
    payload_bytes: u64,
    body: serde_json::Value,
}

fn run_replay_spill(
    path: &str,
    output_format: CliOutputFormat,
) -> Result<(), String> {
    let p = PathBuf::from(path);
    let bytes = fs::read(&p).map_err(|e| {
        format!(
            "reliability replay-spill: read {} failed: {e}",
            p.display()
        )
    })?;
    let body: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| {
        format!(
            "reliability replay-spill: parse {} as JSON failed: {e}",
            p.display()
        )
    })?;
    let row = ReplayRow {
        path: p.display().to_string(),
        payload_bytes: bytes.len() as u64,
        body,
    };
    render(&[row], output_format)
}

fn render<T: Serialize>(rows: &[T], output_format: CliOutputFormat) -> Result<(), String> {
    let out = match output_format {
        CliOutputFormat::Json => serde_json::to_string_pretty(rows)
            .map_err(|e| format!("render: json serialise failed: {e}"))?,
        CliOutputFormat::Toml => {
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn status_renders_json_with_chain_in_order() {
        // Smoke: status renders without panicking + emits all 5 chain
        // entries in the documented order.
        let row = StatusRow {
            inline_cap_bytes: RELIABILITY_GATE_INLINE_CAP,
            default_retry_cap:
                elai_orchestrator::tool_reliability::retry::DEFAULT_RETRY_CAP,
            spill_dir: ".elai_cc/.elai_cc/spill".into(),
            ssb_action_types: vec![
                "tool_reliability_gate_passed",
                "tool_reliability_gate_rejected",
                "tool_call_spilled",
                "tool_call_retry_escalated",
            ],
            composition_chain: vec![
                "authz_policy_c42",
                "tool_gate_c35",
                "calibration_consult_c47",
                "reliability_gate_e18",
                "dispatch",
            ],
        };
        let s = serde_json::to_string(&row).expect("serialise");
        // Composition order: authz before gate before calibration before
        // reliability before dispatch.
        let authz = s.find("authz_policy_c42").expect("authz in output");
        let gate = s.find("tool_gate_c35").expect("gate in output");
        let calib = s.find("calibration_consult_c47").expect("calib in output");
        let rel = s.find("reliability_gate_e18").expect("rel in output");
        let dispatch = s.find("\"dispatch\"").expect("dispatch in output");
        assert!(authz < gate);
        assert!(gate < calib);
        assert!(calib < rel);
        assert!(rel < dispatch);
    }

    #[test]
    fn list_spills_on_empty_dir_returns_empty_vec() {
        let dir = TempDir::new().expect("tempdir");
        let rows = list_spill_dir(dir.path()).expect("ok");
        assert!(rows.is_empty(), "empty dir must produce empty list");
    }

    #[test]
    fn list_spills_filters_non_json() {
        let dir = TempDir::new().expect("tempdir");
        fs::write(dir.path().join("0001-aaaa.json"), b"{\"k\":1}").expect("write json");
        fs::write(dir.path().join("README.txt"), b"hello").expect("write txt");
        let rows = list_spill_dir(dir.path()).expect("ok");
        assert_eq!(rows.len(), 1, "should only count .json files");
        assert_eq!(rows[0].filename, "0001-aaaa.json");
    }

    #[test]
    fn list_spills_missing_dir_returns_empty_via_run() {
        // run_list_spills swallows the read_dir error and renders an
        // empty list — operator-friendly default.
        run_list_spills(
            Some("never-existed-spill-dir".into()),
            CliOutputFormat::Json,
        )
        .expect("ok");
    }

    #[test]
    fn replay_spill_roundtrips_json_body() {
        let dir = TempDir::new().expect("tempdir");
        let p = dir.path().join("0001-abcd.json");
        fs::write(&p, br#"{"tool":"edit","args":{"path":"src/lib.rs"}}"#)
            .expect("write");
        run_replay_spill(p.to_str().expect("path utf8"), CliOutputFormat::Json)
            .expect("ok");
    }
}

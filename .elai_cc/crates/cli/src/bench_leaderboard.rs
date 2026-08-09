//! E-31 — `elai bench leaderboard` CLI driver.
//!
//! Runs the coding-model leaderboard scraper, consumes its JSONL stdout, and
//! emits one CodingModelLeaderboardEvent SSB row per valid line.

use std::fs;
use std::path::Path;
use std::process::Command;

use elai_core::paths::sessions_dir_path;
use elai_orchestrator::ssb::{CodingModelLeaderboardEventPayload, LeaderboardEventKind};
use elai_orchestrator::ssb::ELAI_CODING_MODEL_LEADERBOARD_ENABLE_ENV;
use elai_orchestrator::{Ssb, SsbConfig, SsbPayload};
use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Error)]
enum BenchLeaderboardError {
    #[error("leaderboard could not invoke Python ({bin}): {source_msg}")]
    PythonInvoke { bin: String, source_msg: String },
    #[error("leaderboard harness exited with status {code:?}: {stderr_tail}")]
    HarnessFailed { code: Option<i32>, stderr_tail: String },
    #[error("leaderboard row {line_no} did not parse: {source_msg}")]
    RowParse { line_no: usize, source_msg: String },
    #[error("leaderboard SSB emission failed: {source_msg}")]
    SsbEmit { source_msg: String },
    #[error(
        "ELAI_CODING_MODEL_LEADERBOARD_ENABLE is off; accepted values are \"1\" or \"true\" (case-insensitive)"
    )]
    GateOff,
}

/// Run the leaderboard scraper from the current project root.
pub(crate) fn run(
    suite: Option<String>,
    fixture: Option<String>,
    session_id: Option<String>,
    python_bin: Option<String>,
) -> anyhow::Result<()> {
    let project_root = std::env::current_dir()
        .map_err(|e| anyhow::anyhow!("leaderboard cwd: {e}"))?;
    let gate_raw = std::env::var(ELAI_CODING_MODEL_LEADERBOARD_ENABLE_ENV).ok();
    match run_at(&project_root, gate_raw, suite, fixture, session_id, python_bin) {
        Ok(()) => Ok(()),
        Err(e) => {
            eprintln!("LEADERBOARD ABORT: {e}");
            std::process::exit(1);
        }
    }
}

/// E-31 — the documented match-string contract for
/// ELAI_CODING_MODEL_LEADERBOARD_ENABLE: `"1"` or `"true"`, case-insensitive.
/// Any other value, and an unset variable, leave the driver refusing.
pub(crate) fn leaderboard_gate_enabled(raw: Option<&str>) -> bool {
    raw.map(|value| value.trim().eq_ignore_ascii_case("1") || value.trim().eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

fn run_at(
    project_root: &Path,
    gate_raw: Option<String>,
    suite: Option<String>,
    fixture: Option<String>,
    session_id: Option<String>,
    python_bin: Option<String>,
) -> anyhow::Result<()> {
    if !leaderboard_gate_enabled(gate_raw.as_deref()) {
        return Err(anyhow::Error::new(BenchLeaderboardError::GateOff));
    }
    invoke_harness(
        python_bin.as_deref().unwrap_or("python"),
        project_root,
        suite.as_deref(),
        fixture.as_deref(),
        session_id.as_deref(),
        gate_raw.as_deref(),
    )
    .map_err(anyhow::Error::new)
}

fn invoke_harness(
    python_bin: &str,
    project_root: &Path,
    suite: Option<&str>,
    fixture: Option<&str>,
    session_id: Option<&str>,
    gate_raw: Option<&str>,
) -> Result<(), BenchLeaderboardError> {
    let script = project_root.join("benches/external/coding_model_leaderboard.py");
    let mut cmd = Command::new(python_bin);
    cmd.arg(&script);
    if let Some(value) = suite {
        cmd.arg("--suite").arg(value);
    }
    if let Some(value) = fixture {
        cmd.arg("--fixture").arg(value);
    }
    if let Some(value) = session_id {
        cmd.arg("--session-id").arg(value);
    }
    // Both sides check: Rust stops the spawn, while Python remains authoritative and accepts only "1", so "true" intentionally returns an env_gate_off row.
    if let Some(value) = gate_raw {
        cmd.env(ELAI_CODING_MODEL_LEADERBOARD_ENABLE_ENV, value);
    }

    let out = cmd.output().map_err(|e| BenchLeaderboardError::PythonInvoke {
        bin: python_bin.to_owned(),
        source_msg: e.to_string(),
    })?;
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    let mut rows = Vec::new();
    for (index, line) in stdout.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let row: CodingModelLeaderboardEventPayload = serde_json::from_str(line).map_err(|e| {
            BenchLeaderboardError::RowParse {
                line_no: index + 1,
                source_msg: e.to_string(),
            }
        })?;
        rows.push(row);
    }

    if !rows.is_empty() {
        emit_beacon(project_root, &rows)?;
    }
    print!("{stdout}");
    if !stderr.trim().is_empty() {
        eprintln!("{stderr}");
    }
    if !out.status.success() {
        let stderr_tail = stderr
            .lines()
            .rev()
            .take(20)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<Vec<_>>()
            .join("\n");
        return Err(BenchLeaderboardError::HarnessFailed {
            code: out.status.code(),
            stderr_tail,
        });
    }
    Ok(())
}

fn emit_beacon(
    project_root: &Path,
    rows: &[CodingModelLeaderboardEventPayload],
) -> Result<(), BenchLeaderboardError> {
    let db_path = sessions_dir_path(project_root).join("ssb.db");
    if let Some(parent) = db_path.parent() {
        fs::create_dir_all(parent).map_err(|e| BenchLeaderboardError::SsbEmit {
            source_msg: e.to_string(),
        })?;
    }
    let cfg = SsbConfig {
        db_path,
        pool_size: None,
        project_id: "default".into(),
        session_id: "cli-bench-leaderboard".into(),
    };
    let clock: std::sync::Arc<dyn elai_core::clock::ClockProvider> =
        std::sync::Arc::new(elai_core::OsClock);
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| BenchLeaderboardError::SsbEmit {
            source_msg: e.to_string(),
        })?;
    let rows = rows.to_vec();
    rt.block_on(async move {
        let ssb = Ssb::new(cfg, clock).map_err(|e| BenchLeaderboardError::SsbEmit {
            source_msg: e.to_string(),
        })?;
        for row in rows {
            ssb.write_payload(
                "cli:bench-leaderboard",
                0,
                SsbPayload::CodingModelLeaderboardEvent(row),
                Vec::new(),
            )
            .await
            .map_err(|e| BenchLeaderboardError::SsbEmit {
                source_msg: e.to_string(),
            })?;
        }
        Ok::<(), BenchLeaderboardError>(())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(unix)]
    #[test]
    fn run_at_emits_leaderboard_rows_from_harness_jsonl() {
        use elai_orchestrator::ssb::SsbReader;
        use std::os::unix::fs::PermissionsExt;

        let dir = tempfile::TempDir::new().expect("temporary project root");
        let stub = dir.path().join("python-stub");
        std::fs::write(
            &stub,
            r#"#!/bin/sh
printf '%s\n' '{"kind":"scrape_completed","suite_name":"swe-bench-pro","suite_version":"2026.1","leaderboard_url":"https://example.test","scrape_run_id":"run-1","model_name":"fixture-model","score_pct_bp":6430,"sample_size":731,"entry_hash":"0123456789abcdef","pricing_label":"fixture","reason_label":"ok","http_status":200,"session_id":null,"details_json":{},"ts_ms":1700000000000}'
"#,
        )
        .expect("write Python stub");
        let mut permissions = std::fs::metadata(&stub).expect("stub metadata").permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(&stub, permissions).expect("make Python stub executable");

        run_at(
            dir.path(),
            Some("1".to_owned()),
            Some("all".to_owned()),
            None,
            None,
            Some(stub.to_string_lossy().into_owned()),
        )
        .expect("stub harness should emit an SSB row");

        let cfg = SsbConfig {
            db_path: sessions_dir_path(dir.path()).join("ssb.db"),
            pool_size: None,
            project_id: "default".into(),
            session_id: "cli-bench-leaderboard".into(),
        };
        let clock: std::sync::Arc<dyn elai_core::clock::ClockProvider> =
            std::sync::Arc::new(elai_core::OsClock);
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("build Tokio runtime for SSB read-back");
        let rows = rt.block_on(async move {
            let ssb = Ssb::new(cfg, clock).expect("open emitted SSB");
            ssb.query_by_session("cli-bench-leaderboard")
                .expect("read emitted SSB row")
        });
        assert_eq!(rows.len(), 1);
        let payload: SsbPayload = rmp_serde::from_slice(&rows[0].payload).expect("decode row");
        let SsbPayload::CodingModelLeaderboardEvent(payload) = payload else {
            panic!("expected CodingModelLeaderboardEvent payload");
        };
        assert_eq!(payload.kind, LeaderboardEventKind::ScrapeCompleted);
        assert_eq!(payload.suite_name, "swe-bench-pro");
        assert_eq!(payload.score_pct_bp, 6430);
        assert_eq!(payload.sample_size, 731);
        assert_eq!(payload.model_name, "fixture-model");
    }

    #[cfg(unix)]
    #[test]
    fn gate_refusal_writes_zero_rows() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempfile::TempDir::new().expect("temporary project root");
        let stub = dir.path().join("python-stub");
        std::fs::write(
            &stub,
            r#"#!/bin/sh
printf '%s\n' '{"kind":"scrape_completed","suite_name":"swe-bench-pro","suite_version":"2026.1","leaderboard_url":"https://example.test","scrape_run_id":"run-1","model_name":"fixture-model","score_pct_bp":6430,"sample_size":731,"entry_hash":"0123456789abcdef","pricing_label":"fixture","reason_label":"ok","http_status":200,"session_id":null,"details_json":{},"ts_ms":1700000000000}'
"#,
        )
        .expect("write Python stub");
        let mut permissions = std::fs::metadata(&stub).expect("stub metadata").permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(&stub, permissions).expect("make Python stub executable");

        let result = run_at(
            dir.path(),
            None,
            Some("all".to_owned()),
            None,
            None,
            Some(stub.to_string_lossy().into_owned()),
        );
        assert!(result.is_err());
        assert!(!sessions_dir_path(dir.path()).join("ssb.db").exists());
    }

    #[test]
    fn gate_accepts_documented_match_strings() {
        assert!(leaderboard_gate_enabled(Some("1")));
        assert!(leaderboard_gate_enabled(Some("true")));
        assert!(leaderboard_gate_enabled(Some("TRUE")));
        assert!(leaderboard_gate_enabled(Some(" true ")));
        assert!(!leaderboard_gate_enabled(None));
        assert!(!leaderboard_gate_enabled(Some("0")));
        assert!(!leaderboard_gate_enabled(Some("")));
        assert!(!leaderboard_gate_enabled(Some("yes")));
        assert!(!leaderboard_gate_enabled(Some("2")));
    }

}

//! C-37 — `elai bench snowballing` CLI driver.
//!
//! Surfaces the Python Structure Snowballing harness
//! (`.elai_cc/python/elai_inference/c37_snowballing/`) as a user-facing
//! subcommand + CI hook. The replication exercises a constrained-decoding
//! 5-class Reflector against the Reflexion baseline (paper arXiv:2604.06066
//! Section 3.3) and reports the FORMATTING_MISMATCH fraction (paper Table
//! 3 anchor: 0.96; replication threshold 0.50; severity threshold 0.90).
//!
//! # Wiring discipline
//!
//! producer  : the Python harness at
//!             `.elai_cc/python/elai_inference/c37_snowballing/run.py`
//!             (paper-grounded; ungrounded SSB beacons are RUNPOD-PENDING).
//! consumer  : this file — owns its tokio runtime (matches the sync entry
//!             pattern of bench_prmbench / bench_mode_routing siblings),
//!             shells out to the Python module via `python -m`, and
//!             surfaces the run's summary JSON.
//! ssb row   : this driver parses the harness summary and emits one
//!             `SnowballingMeasured` row. `PerRoleDecodingApplied` has no
//!             production emit site at HEAD.
//!
//! # Exit codes
//!
//! 0 = harness completed (replication threshold gating is reported, not
//!     enforced — the decision-matrix is the analyzer's job per plan §4)
//! 1 = harness aborted (Python missing, dataset missing, sidecar
//!     unreachable, or argparse rejection)
//!
//! # Flags
//!
//!   --pool A|B               Required. Paper Section 4.1 split.
//!   --n-samples N            Limit harness to first N items in the pool
//!                            (default: 55 for Pool A, 45 for Pool B).
//!   --model NAME             HF model ID (default Qwen/Qwen3-8B per paper).
//!   --max-trials N           Paper default 5; clamped to [1, 10].
//!   --reflector-mode MODE    schema_hint (default) or outlines.
//!   --sidecar-url URL        Default http://127.0.0.1:8100.
//!   --output PATH            JSONL output path (required).
//!   --python-bin PATH        Python interpreter (default `python`).
//!
//! # No `.unwrap()` in non-test code.

use std::path::PathBuf;
use std::process::Command;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use elai_core::paths::sessions_dir_path;
use elai_orchestrator::ssb::SnowballingMeasuredPayload;
use elai_orchestrator::{Ssb, SsbConfig, SsbPayload};
use serde::Deserialize;

use thiserror::Error;

// ---------------------------------------------------------------------------
// Error type (thiserror per CLAUDE.md anti-stale rule).
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum BenchSnowballingError {
    /// `--pool` value was not "A" or "B".
    #[error("snowballing --pool must be 'A' or 'B'; got {value:?}")]
    InvalidPool { value: String },
    /// `--max-trials` outside [1, 10].
    #[error("snowballing --max-trials {value} outside [1, 10]")]
    InvalidMaxTrials { value: i64 },
    /// `--reflector-mode` value was not "schema_hint" or "outlines".
    #[error("snowballing --reflector-mode must be 'schema_hint' or 'outlines'; got {value:?}")]
    InvalidReflectorMode { value: String },
    /// The Python interpreter could not be invoked (PATH miss / permissions).
    #[error("snowballing could not invoke Python ({bin}): {source_msg}")]
    PythonInvoke { bin: String, source_msg: String },
    /// The Python module exited with a non-zero status.
    #[error("snowballing harness exited with status {code:?}: {stderr_tail}")]
    HarnessFailed { code: Option<i32>, stderr_tail: String },
    /// Harness stdout was not the promised summary JSON object.
    #[error("snowballing harness summary JSON did not parse: {source_msg}")]
    SummaryParse { source_msg: String },
    /// SSB emission failed after a successful harness run.
    #[error("snowballing SSB emission failed: {source_msg}")]
    SsbEmit { source_msg: String },
}

// ---------------------------------------------------------------------------
// CLI entry — parses flags, shells out to the Python harness, exits.
// ---------------------------------------------------------------------------

/// CLI entry. Validates flags, then dispatches to the Python harness
/// `elai_inference.c37_snowballing.run`. On success returns `Ok(())`;
/// on failure prints the error to stderr and calls `std::process::exit(1)`.
pub(crate) fn run(
    pool: String,
    n_samples: Option<i64>,
    model: Option<String>,
    max_trials: Option<i64>,
    reflector_mode: Option<String>,
    sidecar_url: Option<String>,
    output: String,
    python_bin: Option<String>,
) -> anyhow::Result<()> {
    let project_root = std::env::current_dir().map_err(|e| anyhow::anyhow!("snowballing cwd: {e}"))?;
    match run_at(
        &project_root,
        pool,
        n_samples,
        model,
        max_trials,
        reflector_mode,
        sidecar_url,
        output,
        python_bin,
    ) {
        Ok(()) => Ok(()),
        Err(e) => {
            eprintln!("SNOWBALLING ABORT: {e}");
            std::process::exit(1);
        }
    }
}

fn run_at(
    project_root: &Path,
    pool: String,
    n_samples: Option<i64>,
    model: Option<String>,
    max_trials: Option<i64>,
    reflector_mode: Option<String>,
    sidecar_url: Option<String>,
    output: String,
    python_bin: Option<String>,
) -> anyhow::Result<()> {
    let validated = validate_args(
        &pool,
        n_samples,
        model.as_deref(),
        max_trials,
        reflector_mode.as_deref(),
        sidecar_url.as_deref(),
        &output,
    )?;
    let py = python_bin.unwrap_or_else(|| "python".to_owned());
    invoke_harness(&py, &validated, project_root).map_err(anyhow::Error::new)
}

// ---------------------------------------------------------------------------
// Argument validation (no live I/O — pure value-check).
// ---------------------------------------------------------------------------

/// Validated argument bundle handed to `invoke_harness`. Exposed so the
/// in-crate unit tests can drive validate_args without shelling out.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedArgs {
    pub pool: String,
    pub n_samples: Option<i64>,
    pub model: String,
    pub max_trials: i64,
    pub reflector_mode: String,
    pub sidecar_url: String,
    pub output: String,
}

/// Validate raw CLI args without performing I/O. Returns either the
/// canonicalised arg bundle or an error suitable for the caller's stderr
/// stream.
pub fn validate_args(
    pool: &str,
    n_samples: Option<i64>,
    model: Option<&str>,
    max_trials: Option<i64>,
    reflector_mode: Option<&str>,
    sidecar_url: Option<&str>,
    output: &str,
) -> Result<ValidatedArgs, BenchSnowballingError> {
    if pool != "A" && pool != "B" {
        return Err(BenchSnowballingError::InvalidPool { value: pool.to_owned() });
    }
    let max_trials = max_trials.unwrap_or(5);
    if !(1..=10).contains(&max_trials) {
        return Err(BenchSnowballingError::InvalidMaxTrials { value: max_trials });
    }
    let reflector_mode = reflector_mode.unwrap_or("schema_hint").to_owned();
    if reflector_mode != "schema_hint" && reflector_mode != "outlines" {
        return Err(BenchSnowballingError::InvalidReflectorMode {
            value: reflector_mode,
        });
    }
    Ok(ValidatedArgs {
        pool: pool.to_owned(),
        n_samples,
        model: model.unwrap_or("Qwen/Qwen3-8B").to_owned(),
        max_trials,
        reflector_mode,
        sidecar_url: sidecar_url.unwrap_or("http://127.0.0.1:8100").to_owned(),
        output: output.to_owned(),
    })
}

// ---------------------------------------------------------------------------
// Python harness invocation.
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct SnowballingSummary {
    pool: String, n_questions: u32, reflector_mode: String, model: String,
    accuracy: f64, formatting_fraction: f64,
    error_distribution: HashMap<String, u32>,
    replication_threshold: f64, severity_threshold: f64,
    constrained_accuracy_ceiling: f64,
}

fn invoke_harness(
    python_bin: &str,
    args: &ValidatedArgs,
    project_root: &Path,
) -> Result<(), BenchSnowballingError> {
    let mut cmd = Command::new(python_bin);
    cmd.arg("-m")
        .arg("elai_inference.c37_snowballing.run")
        .arg("--pool")
        .arg(&args.pool)
        .arg("--model")
        .arg(&args.model)
        .arg("--max-trials")
        .arg(args.max_trials.to_string())
        .arg("--reflector-mode")
        .arg(&args.reflector_mode)
        .arg("--sidecar-url")
        .arg(&args.sidecar_url)
        .arg("--output")
        .arg(&args.output);
    if let Some(n) = args.n_samples {
        cmd.arg("--n-samples").arg(n.to_string());
    }

    tracing::info!(
        target: "cli::bench_snowballing",
        pool = args.pool,
        model = args.model,
        max_trials = args.max_trials,
        reflector_mode = args.reflector_mode,
        sidecar_url = args.sidecar_url,
        output = args.output,
        "C-37: invoking Python snowballing harness"
    );

    let out = cmd.output().map_err(|e| BenchSnowballingError::PythonInvoke {
        bin: python_bin.to_owned(),
        source_msg: format!("{e}"),
    })?;

    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);

    if !out.status.success() {
        let tail: String = stderr
            .lines()
            .rev()
            .take(20)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<Vec<_>>()
            .join("\n");
        return Err(BenchSnowballingError::HarnessFailed {
            code: out.status.code(),
            stderr_tail: tail,
        });
    }

    let summary: SnowballingSummary = serde_json::from_str(stdout.trim()).map_err(|e| {
        BenchSnowballingError::SummaryParse { source_msg: e.to_string() }
    })?;
    let params = elai_providers::decoding::paper_decoding_params_for(
        elai_providers::decoding::PaperRole::Reflection,
    );
    let payload = SnowballingMeasuredPayload {
        pool: summary.pool, n_questions: summary.n_questions,
        reflector_mode: summary.reflector_mode, model: summary.model,
        temperature: f64::from(params.temperature), max_new_tokens: params.max_new_tokens,
        constrained: params.constrained, accuracy: summary.accuracy,
        formatting_fraction: summary.formatting_fraction,
        retrieval_focus_count: *summary.error_distribution.get("RETRIEVAL_FOCUS").unwrap_or(&0),
        bridge_failure_count: *summary.error_distribution.get("BRIDGE_FAILURE").unwrap_or(&0),
        hallucination_count: *summary.error_distribution.get("HALLUCINATION").unwrap_or(&0),
        inference_error_count: *summary.error_distribution.get("INFERENCE_ERROR").unwrap_or(&0),
        formatting_mismatch_count: *summary.error_distribution.get("FORMATTING_MISMATCH").unwrap_or(&0),
        replication_threshold: summary.replication_threshold,
        severity_threshold: summary.severity_threshold,
        constrained_accuracy_ceiling: summary.constrained_accuracy_ceiling,
        runpod_pending: true,
    };
    emit_beacon(project_root, &payload)?;
    // Pass-through: write the summary JSON to stdout and progress to stderr.
    print!("{stdout}");
    if !stderr.trim().is_empty() {
        eprintln!("{stderr}");
    }
    Ok(())
}

fn emit_beacon(project_root: &Path, payload: &SnowballingMeasuredPayload) -> Result<(), BenchSnowballingError> {
    let db_path = sessions_dir_path(project_root).join("ssb.db");
    if let Some(parent) = db_path.parent() { fs::create_dir_all(parent).map_err(|e| BenchSnowballingError::SsbEmit { source_msg: e.to_string() })?; }
    let cfg = SsbConfig { db_path, pool_size: None, project_id: "default".into(), session_id: "cli-bench-snowballing".into() };
    let clock: std::sync::Arc<dyn elai_core::clock::ClockProvider> = std::sync::Arc::new(elai_core::OsClock);
    let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().map_err(|e| BenchSnowballingError::SsbEmit { source_msg: e.to_string() })?;
    let payload = payload.clone();
    rt.block_on(async move {
        let ssb = Ssb::new(cfg, clock).map_err(|e| BenchSnowballingError::SsbEmit { source_msg: e.to_string() })?;
        ssb.write_payload("cli:bench-snowballing", 0, SsbPayload::SnowballingMeasured(payload), Vec::new()).await.map_err(|e| BenchSnowballingError::SsbEmit { source_msg: e.to_string() })?;
        Ok::<(), BenchSnowballingError>(())
    })
}

// ---------------------------------------------------------------------------
// Unit tests — validate_args only (no Python shell-out; that is RUNPOD-PENDING).
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_args_accepts_pool_a_with_defaults() {
        let v = validate_args(
            "A",
            None,
            None,
            None,
            None,
            None,
            "runs/snowballing-poolA.jsonl",
        )
        .expect("Pool A defaults must validate");
        assert_eq!(v.pool, "A");
        assert_eq!(v.max_trials, 5);
        assert_eq!(v.model, "Qwen/Qwen3-8B");
        assert_eq!(v.reflector_mode, "schema_hint");
        assert_eq!(v.sidecar_url, "http://127.0.0.1:8100");
    }

    #[test]
    fn validate_args_rejects_unknown_pool() {
        let err = validate_args(
            "C",
            None,
            None,
            None,
            None,
            None,
            "runs/x.jsonl",
        )
        .expect_err("unknown pool must reject");
        assert!(matches!(
            err,
            BenchSnowballingError::InvalidPool { value } if value == "C"
        ));
    }

    #[test]
    fn validate_args_rejects_out_of_range_max_trials() {
        let err = validate_args(
            "A",
            None,
            None,
            Some(99),
            None,
            None,
            "runs/x.jsonl",
        )
        .expect_err("max-trials > 10 must reject");
        assert!(matches!(
            err,
            BenchSnowballingError::InvalidMaxTrials { value } if value == 99
        ));
        let err = validate_args(
            "A",
            None,
            None,
            Some(0),
            None,
            None,
            "runs/x.jsonl",
        )
        .expect_err("max-trials < 1 must reject");
        assert!(matches!(
            err,
            BenchSnowballingError::InvalidMaxTrials { value } if value == 0
        ));
    }

    #[test]
    fn validate_args_rejects_unknown_reflector_mode() {
        let err = validate_args(
            "A",
            None,
            None,
            None,
            Some("free_text"),
            None,
            "runs/x.jsonl",
        )
        .expect_err("unknown reflector_mode must reject");
        assert!(matches!(
            err,
            BenchSnowballingError::InvalidReflectorMode { value } if value == "free_text"
        ));
    }

    #[test]
    fn validate_args_accepts_outlines_mode() {
        let v = validate_args(
            "B",
            Some(45),
            Some("Qwen/Qwen2.5-Coder-7B-Instruct-abliterated"),
            Some(5),
            Some("outlines"),
            Some("http://localhost:8100"),
            "runs/snowballing-poolB.jsonl",
        )
        .expect("Pool B + outlines mode must validate");
        assert_eq!(v.pool, "B");
        assert_eq!(v.n_samples, Some(45));
        assert_eq!(v.reflector_mode, "outlines");
        assert_eq!(v.model, "Qwen/Qwen2.5-Coder-7B-Instruct-abliterated");
    }
    #[cfg(unix)]
    #[test]
    fn run_at_emits_snowballing_row_from_harness_summary() {
        use elai_orchestrator::ssb::SsbReader;
        use std::os::unix::fs::PermissionsExt;

        let dir = tempfile::TempDir::new().expect("temporary project root");
        let stub = dir.path().join("python-stub");
        std::fs::write(
            &stub,
            r#"#!/bin/sh
printf '%s\n' '{"pool":"B","n_questions":7,"reflector_mode":"schema_hint","model":"fixture-model","accuracy":0.4,"formatting_fraction":0.2,"error_distribution":{"RETRIEVAL_FOCUS":2,"HALLUCINATION":3},"replication_threshold":0.5,"severity_threshold":0.9,"constrained_accuracy_ceiling":0.6}'
"#,
        )
        .expect("write Python stub");
        let mut permissions = std::fs::metadata(&stub).expect("stub metadata").permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(&stub, permissions).expect("make Python stub executable");

        let output = dir.path().join("summary.jsonl");
        run_at(
            dir.path(),
            "B".to_owned(),
            None,
            None,
            None,
            None,
            None,
            output.to_string_lossy().into_owned(),
            Some(stub.to_string_lossy().into_owned()),
        )
        .expect("stub harness should emit an SSB row");

        let db_path = sessions_dir_path(dir.path()).join("ssb.db");
        let cfg = SsbConfig {
            db_path,
            pool_size: None,
            project_id: "default".into(),
            session_id: "cli-bench-snowballing".into(),
        };
        let clock: std::sync::Arc<dyn elai_core::clock::ClockProvider> =
            std::sync::Arc::new(elai_core::OsClock);
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("build Tokio runtime for SSB read-back");
        let rows = rt.block_on(async move {
            let ssb = Ssb::new(cfg, clock).expect("open emitted SSB");
            ssb.query_by_session("cli-bench-snowballing")
                .expect("read emitted SSB row")
        });
        assert_eq!(rows.len(), 1);
        let payload: SsbPayload = rmp_serde::from_slice(&rows[0].payload).expect("decode row");
        let SsbPayload::SnowballingMeasured(payload) = payload else {
            panic!("expected SnowballingMeasured payload");
        };
        assert_eq!(payload.pool, "B");
        assert_eq!(payload.n_questions, 7);
        assert_eq!(payload.retrieval_focus_count, 2);
        assert_eq!(payload.hallucination_count, 3);
        assert_eq!(payload.bridge_failure_count, 0);
        assert_eq!(payload.inference_error_count, 0);
        assert_eq!(payload.formatting_mismatch_count, 0);
        let params = elai_providers::decoding::paper_decoding_params_for(
            elai_providers::decoding::PaperRole::Reflection,
        );
        assert_eq!(payload.temperature, f64::from(params.temperature));
        assert_eq!(payload.max_new_tokens, params.max_new_tokens);
        assert_eq!(payload.constrained, params.constrained);
    }

}

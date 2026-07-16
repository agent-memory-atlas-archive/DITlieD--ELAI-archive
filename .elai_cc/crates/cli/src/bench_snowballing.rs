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
//! ssb row   : `SsbActionType::PerRoleDecodingApplied` is emitted by
//!             `agent_pool::spawn` (orthogonal to this driver); a future
//!             follow-up wires a `SnowballingBenchRunCompleted` row when
//!             RunPod inference lands.
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
    let validated = match validate_args(
        &pool,
        n_samples,
        model.as_deref(),
        max_trials,
        reflector_mode.as_deref(),
        sidecar_url.as_deref(),
        &output,
    ) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("SNOWBALLING ABORT: {e}");
            std::process::exit(1);
        }
    };

    let py = python_bin.unwrap_or_else(|| "python".to_owned());

    let outcome = invoke_harness(&py, &validated);
    match outcome {
        Ok(_) => Ok(()),
        Err(e) => {
            eprintln!("SNOWBALLING ABORT: {e}");
            std::process::exit(1);
        }
    }
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

fn invoke_harness(
    python_bin: &str,
    args: &ValidatedArgs,
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

    // Pass-through: write the summary JSON to stdout and the per-question
    // progress lines to stderr (the harness streams them already).
    print!("{stdout}");
    if !stderr.trim().is_empty() {
        eprintln!("{stderr}");
    }
    Ok(())
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
}

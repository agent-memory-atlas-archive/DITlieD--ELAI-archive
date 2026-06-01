//! D-28 — `elai farm-adv-rm <sub>` CLI driver.
//!
//! Surfaces [`elai_orchestrator::farm::adv_rm::AdvRmFarmer`] as a
//! user-facing subcommand + CI hook. The Adv-RM farm-loop walks a seed
//! dataset, scores each candidate-step chain via the D-2 PPM hook, and
//! produces a `corpus.jsonl` of HIGH-PPM low-deterministic-quality
//! pairs the downstream PPM retraining stage consumes.
//!
//! Paper grounding (arXiv:2504.06141): Z>1.96 dual-filter (paper §3.2),
//! ~1000-sample sweet spot per round (paper §5), 2-round saturation cap
//! (paper §5), 1.0pp PRMBench overall-delta floor (paper §5 +0.7pp
//! overall rounded UP to integer-pp). See workflow/reasoning/
//! D-28-decisions.md DECISION-3 for the rejected "+5pp" plan-doc value.
//!
//! # Wiring discipline
//!
//! producer  : [`elai_orchestrator::farm::adv_rm::AdvRmFarmer::run`].
//! consumer  : this file — owns its tokio runtime + delegates to the
//!             farmer + renders the JSON report. CI consumes the exit
//!             code (0 = Pass, 1 = Fail).
//! ssb row   : `SsbActionType::AdversarialFarmRunCompleted` (one per run;
//!             emitted by the orchestrator-side producer when invoked
//!             from inside a task_runner; the CLI does NOT persist SSB
//!             rows directly per the D-19 / D-23 / D-24 / D-25 / D-26
//!             precedent).
//!
//! # Sub-verbs
//!
//!   `run --dataset PATH [--corpus-dir DIR] [--output PATH] [--max-samples N]
//!        [--max-rounds N] [--z-threshold F] [--paper-floor-pp F]
//!        [--source-session ID]`
//!     Walk the seed dataset, apply the Z>1.96 filter, write the
//!     corpus.jsonl, print the JSON report.
//!
//!   `verdict --report PATH`
//!     Read a previously-written JSON report from PATH and re-evaluate
//!     the verdict against the configured floor. Useful when an
//!     operator wants to gate a CI run on a saved report without
//!     re-running the full farm loop.
//!
//! # Exit codes
//!
//! 0 = report verdict == Pass
//! 1 = report verdict == Fail OR any handled error
//!
//! # No `.unwrap()` in non-test code (anti-stale + anti-lazy rules).

use std::path::PathBuf;
use std::sync::Arc;

use elai_orchestrator::benchmarks::prmbench_runner::GatingVerdict;
use elai_orchestrator::farm::adv_rm::{
    verdict as compute_verdict, AdvRmFarmRunReport, AdvRmFarmer, FarmError,
    DEFAULT_MAX_ROUNDS, DEFAULT_MAX_SAMPLES, DEFAULT_PAPER_FLOOR_PP,
    DEFAULT_Z_THRESHOLD,
};
use elai_orchestrator::sidecar_ppm_hook::MockSidecarPpmHook;
use thiserror::Error;

use crate::args::{FarmAdvRmArgs, FarmAdvRmSub};

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum FarmAdvRmError {
    /// Underlying farmer failure (validation, I/O, hook, serialisation).
    #[error("farm-adv-rm runner error: {0}")]
    Runner(#[from] FarmError),
    /// Generic I/O reading/writing the report or dataset path.
    #[error("farm-adv-rm could not access {path}: {source_msg}")]
    Io { path: String, source_msg: String },
    /// JSON serialisation failure when rendering or parsing a report.
    #[error("farm-adv-rm json error: {source_msg}")]
    Json { source_msg: String },
    /// `--max-samples` / `--max-rounds` / `--z-threshold` /
    /// `--paper-floor-pp` outside the validated range at parse-time.
    #[error("farm-adv-rm invalid {flag} value {value}: {reason}")]
    InvalidFlag {
        flag: &'static str,
        value: String,
        reason: String,
    },
}

// ---------------------------------------------------------------------------
// Sync CLI entry — owns its own tokio runtime.
// ---------------------------------------------------------------------------

/// D-28 dispatch shim: constructs `FarmAdvRmArgs` inline and delegates to
/// [`run`]. Reduces the `main.rs` match arm to a single call.
pub(crate) fn dispatch(sub: FarmAdvRmSub) -> anyhow::Result<()> {
    run(FarmAdvRmArgs { sub })
}

/// Top-level CLI entrypoint. Owns the tokio runtime + dispatches to the
/// sub-verb handler. Mirrors `bench_prmbench::run` sync pattern.
pub(crate) fn run(args: FarmAdvRmArgs) -> anyhow::Result<()> {
    match args.sub {
        FarmAdvRmSub::Run {
            dataset,
            corpus_dir,
            output,
            max_samples,
            max_rounds,
            z_threshold,
            paper_floor_pp,
            source_session,
        } => run_farm(
            dataset,
            corpus_dir,
            output,
            max_samples,
            max_rounds,
            z_threshold,
            paper_floor_pp,
            source_session,
        ),
        FarmAdvRmSub::Verdict { report } => run_verdict(report),
    }
}

/// `farm-adv-rm run` handler. Constructs the AdvRmFarmer with the
/// supplied knobs (parsed + validated at the boundary), drives the
/// loop, writes/prints the JSON report, and exits 0 on Pass / 1 on
/// Fail.
#[allow(clippy::too_many_arguments)]
fn run_farm(
    dataset: String,
    corpus_dir: Option<String>,
    output: Option<String>,
    max_samples: Option<String>,
    max_rounds: Option<String>,
    z_threshold: Option<String>,
    paper_floor_pp: Option<String>,
    source_session: Option<String>,
) -> anyhow::Result<()> {
    let max_samples = parse_u32_in("--max-samples", max_samples, 1, 1500)?;
    let max_rounds = parse_u32_in("--max-rounds", max_rounds, 1, 2)?;
    let z_threshold = parse_f64_in("--z-threshold", z_threshold, 0.0, 10.0)?;
    let paper_floor_pp =
        parse_f64_in("--paper-floor-pp", paper_floor_pp, 0.0, 100.0)?;

    let corpus_dir: PathBuf = corpus_dir
        .map(PathBuf::from)
        .unwrap_or_else(default_corpus_dir);
    let dataset_path = PathBuf::from(&dataset);

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| FarmAdvRmError::Io {
            path: "<tokio-runtime>".into(),
            source_msg: format!("{e}"),
        })?;

    let hook = Arc::new(MockSidecarPpmHook::default());
    let mut farmer = AdvRmFarmer::new(hook, dataset_path, corpus_dir)
        .with_max_samples(max_samples.unwrap_or(DEFAULT_MAX_SAMPLES))
        .with_max_rounds(max_rounds.unwrap_or(DEFAULT_MAX_ROUNDS))
        .with_z_threshold(z_threshold.unwrap_or(DEFAULT_Z_THRESHOLD))
        .with_paper_floor_pp(paper_floor_pp.unwrap_or(DEFAULT_PAPER_FLOOR_PP));
    if let Some(s) = source_session {
        farmer = farmer.with_source_session(s);
    }

    let report = rt.block_on(async { farmer.run().await }).map_err(|e| {
        FarmAdvRmError::Runner(e)
    })?;
    render_report(output.as_deref(), &report)?;

    if report.verdict == GatingVerdict::Pass {
        Ok(())
    } else {
        // Non-zero exit signals fail to CI; the report has already been
        // rendered so the operator sees why.
        std::process::exit(1);
    }
}

/// `farm-adv-rm verdict` handler. Reads a JSON AdvRmFarmRunReport from
/// PATH + recomputes the verdict + exits 0/1.
fn run_verdict(report_path: String) -> anyhow::Result<()> {
    let body = std::fs::read_to_string(&report_path).map_err(|e| FarmAdvRmError::Io {
        path: report_path.clone(),
        source_msg: format!("{e}"),
    })?;
    let report: AdvRmFarmRunReport =
        serde_json::from_str(&body).map_err(|e| FarmAdvRmError::Json {
            source_msg: format!("{e}"),
        })?;
    let v = compute_verdict(&report);
    println!(
        "farm-adv-rm verdict: {} (delta_pp={} floor_pp={} rounds={} samples={})",
        match v {
            GatingVerdict::Pass => "pass",
            GatingVerdict::Fail => "fail",
        },
        report.prmbench_delta_pp,
        report.paper_floor_pp,
        report.rounds,
        report.samples_farmed,
    );
    if v == GatingVerdict::Pass {
        Ok(())
    } else {
        std::process::exit(1);
    }
}

/// Default `--corpus-dir` when the caller didn't pass one: the
/// project-relative `.elai_cc/benchmarks/adv_rm/corpus/` location
/// (the runtime dir under which timestamped subdirs are created).
fn default_corpus_dir() -> PathBuf {
    PathBuf::from(".elai_cc/benchmarks/adv_rm/corpus")
}

/// Parse + range-check a u32 flag value.
fn parse_u32_in(
    flag: &'static str,
    raw: Option<String>,
    lo: u32,
    hi: u32,
) -> Result<Option<u32>, FarmAdvRmError> {
    match raw {
        None => Ok(None),
        Some(s) => {
            let v: u32 = s.parse().map_err(|e| FarmAdvRmError::InvalidFlag {
                flag,
                value: s.clone(),
                reason: format!("parse u32: {e}"),
            })?;
            if v < lo || v > hi {
                return Err(FarmAdvRmError::InvalidFlag {
                    flag,
                    value: s,
                    reason: format!("out of [{lo}, {hi}]"),
                });
            }
            Ok(Some(v))
        }
    }
}

/// Parse + range-check an f64 flag value.
fn parse_f64_in(
    flag: &'static str,
    raw: Option<String>,
    lo: f64,
    hi: f64,
) -> Result<Option<f64>, FarmAdvRmError> {
    match raw {
        None => Ok(None),
        Some(s) => {
            let v: f64 = s.parse().map_err(|e| FarmAdvRmError::InvalidFlag {
                flag,
                value: s.clone(),
                reason: format!("parse f64: {e}"),
            })?;
            if v < lo || v > hi {
                return Err(FarmAdvRmError::InvalidFlag {
                    flag,
                    value: s,
                    reason: format!("out of [{lo}, {hi}]"),
                });
            }
            Ok(Some(v))
        }
    }
}

/// Render the report to `--output` PATH (if set) or stdout (default).
fn render_report(
    output: Option<&str>,
    report: &AdvRmFarmRunReport,
) -> Result<(), FarmAdvRmError> {
    let body = serde_json::to_string_pretty(report).map_err(|e| FarmAdvRmError::Json {
        source_msg: format!("{e}"),
    })?;
    match output {
        None => {
            println!("{body}");
            Ok(())
        }
        Some(path) => std::fs::write(path, body).map_err(|e| FarmAdvRmError::Io {
            path: path.to_string(),
            source_msg: format!("{e}"),
        }),
    }
}

// ---------------------------------------------------------------------------
// Public re-exports for the parent dispatcher.
// ---------------------------------------------------------------------------

#[allow(dead_code)]
pub(crate) fn default_paper_floor() -> f64 {
    DEFAULT_PAPER_FLOOR_PP
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_u32_in_accepts_in_range() {
        let v = parse_u32_in("--max-samples", Some("500".into()), 1, 1500).expect("ok");
        assert_eq!(v, Some(500));
    }

    #[test]
    fn parse_u32_in_rejects_out_of_range() {
        let err = parse_u32_in("--max-samples", Some("9999".into()), 1, 1500)
            .expect_err("out of range must err");
        match err {
            FarmAdvRmError::InvalidFlag { flag, .. } => assert_eq!(flag, "--max-samples"),
            other => panic!("expected InvalidFlag, got {other:?}"),
        }
    }

    #[test]
    fn parse_f64_in_accepts_in_range() {
        let v = parse_f64_in("--z-threshold", Some("1.96".into()), 0.0, 10.0).expect("ok");
        assert!((v.unwrap() - 1.96).abs() < 1e-9);
    }

    #[test]
    fn parse_f64_in_rejects_out_of_range() {
        let err = parse_f64_in("--paper-floor-pp", Some("150.0".into()), 0.0, 100.0)
            .expect_err("out of range must err");
        match err {
            FarmAdvRmError::InvalidFlag { flag, .. } => {
                assert_eq!(flag, "--paper-floor-pp")
            }
            other => panic!("expected InvalidFlag, got {other:?}"),
        }
    }

    #[test]
    fn default_paper_floor_matches_runner_default() {
        assert_eq!(default_paper_floor(), DEFAULT_PAPER_FLOOR_PP);
    }
}

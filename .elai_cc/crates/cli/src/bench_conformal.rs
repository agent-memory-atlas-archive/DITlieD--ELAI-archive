//! D-22 — `elai bench conformal` CLI driver.
//!
//! Surfaces
//! [`elai_orchestrator::benchmarks::conformal_validation_runner::ConformalValidationRunner`]
//! as a user-facing subcommand + CI hook. The runner loads a calibration
//! corpus (JSONL), computes tau, replays a held-out validation fixture
//! through D-16's `decide` predicate, and reports per-bucket refusal
//! rates + the UniCR (arXiv:2503.01345) band gate.
//!
//! # Wiring discipline
//!
//! producer  : `ConformalValidationRunner::run_all` (orchestrator-side).
//! consumer  : this file — owns its tokio runtime + emits one SSB row via
//!             `VecConformalValidationSsbEmitter`. CI consumes the exit
//!             code (0 = Pass, 1 = Fail).
//! ssb row   : `SsbActionType::ConformalValidationCompleted` (one per run).
//!
//! # Exit codes
//!
//! 0 = report PASSED (`gating_verdict == "pass"`)
//! 1 = report FAILED (`gating_verdict == "fail"`) OR infrastructural error
//!
//! # Flags
//!
//!   --dataset PATH   JSONL validation fixture (default
//!                    `<cwd>/benchmarks/conformal/fixtures/conformal_validation_mini.jsonl`)
//!   --corpus PATH    JSONL calibration corpus (default: synthesized
//!                    balanced 20-pair corpus identical to the unit-test
//!                    helper — keeps CI hermetic without requiring an
//!                    on-disk seed corpus before D-22 ships).
//!   --output PATH    Write the JSON report to PATH (default: stdout).
//!   --epsilon F      Override the operating-point epsilon (default 0.1).
//!
//! No `.unwrap()` in non-test code; thiserror for the error enum;
//! hand-rolled flag parsing (no new clap dep).

use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use elai_orchestrator::benchmarks::conformal_validation_runner::{
    default_mini_fixture_path, ConformalValidationError, ConformalValidationReport,
    ConformalValidationRunner, VecConformalValidationSsbEmitter,
};
use elai_orchestrator::conformal_calibration::{
    CalibrationCorpus, CalibrationTuple, ConformalError, CorpusSource,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BenchConformalError {
    /// Underlying runner failure (I/O, malformed JSON, label, SSB emit).
    #[error("conformal runner error: {0}")]
    Runner(#[from] ConformalValidationError),
    /// Calibration corpus could not be loaded (only the explicit
    /// `--corpus` path can fail this way; the synthetic default never
    /// touches disk).
    #[error("conformal could not load calibration corpus: {0}")]
    CorpusLoad(#[from] ConformalError),
    /// Generic I/O failure writing the JSON report to `--output`.
    #[error("conformal could not write report to {path}: {source_msg}")]
    OutputIo { path: String, source_msg: String },
    /// JSON serialisation failure when rendering the report.
    #[error("conformal could not serialise report: {source_msg}")]
    Serialise { source_msg: String },
}

/// CLI entry — parses validated flags, runs the benchmark, renders, exits.
///
/// Returns `Ok(())` on clean PASS (rc=0). Calls `std::process::exit(1)`
/// on FAIL or any handled error. `Err(anyhow::Error)` is reserved for
/// infrastructural failures the top-level dispatcher renders.
pub(crate) fn run(
    dataset: Option<String>,
    corpus: Option<String>,
    output: Option<String>,
    epsilon: Option<String>,
) -> anyhow::Result<()> {
    let epsilon = parse_epsilon(epsilon)?;

    let project_root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let dataset_path = dataset
        .map(PathBuf::from)
        .unwrap_or_else(|| default_mini_fixture_path(&project_root));
    let corpus_path = corpus.map(PathBuf::from);

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(anyhow::Error::from)?;

    let outcome = rt.block_on(async move { run_async(dataset_path, corpus_path, epsilon).await });

    match outcome {
        Ok(report) => {
            render_report(output.as_deref(), &report)?;
            if !report.passed() {
                std::process::exit(1);
            }
            Ok(())
        }
        Err(e) => {
            eprintln!("CONFORMAL ABORT: {e}");
            std::process::exit(1);
        }
    }
}

fn parse_epsilon(raw: Option<String>) -> anyhow::Result<Option<f64>> {
    let Some(raw) = raw else {
        return Ok(None);
    };
    let parsed: f64 = raw
        .parse()
        .map_err(|e| anyhow::anyhow!("invalid --epsilon {raw:?}: {e}"))?;
    if !(0.0..1.0).contains(&parsed) || parsed <= 0.0 {
        eprintln!("CONFORMAL ABORT: --epsilon {parsed} out of (0.0, 1.0)");
        std::process::exit(1);
    }
    Ok(Some(parsed))
}

/// Async core — constructs the corpus + runner, runs the benchmark,
/// returns the report. Public so integration tests can drive it without
/// `std::process::exit`.
pub async fn run_async(
    dataset_path: PathBuf,
    corpus_path: Option<PathBuf>,
    epsilon: Option<f64>,
) -> Result<ConformalValidationReport, BenchConformalError> {
    let corpus = match corpus_path {
        Some(p) => CalibrationCorpus::load_seed_jsonl(&p)?,
        None => default_synthetic_corpus(),
    };
    let mut runner = ConformalValidationRunner::new(corpus, dataset_path);
    if let Some(e) = epsilon {
        runner = runner.with_epsilon(e);
    }
    let emitter = VecConformalValidationSsbEmitter::new();
    let now_ms = i64::try_from(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0),
    )
    .unwrap_or(0);
    let report = runner.run_all(&emitter, now_ms).await?;
    Ok(report)
}

/// Build the same balanced 20-pair synthetic corpus the orchestrator
/// unit tests use. Keeps the CLI hermetic (no on-disk seed corpus needed
/// before the production calibration pipeline ships) while still
/// exercising the full producer surface.
fn default_synthetic_corpus() -> CalibrationCorpus {
    let mut c = CalibrationCorpus::new(CorpusSource::Synthetic);
    for i in 0..20 {
        // True claim with high confidence.
        let _ = c.append(CalibrationTuple::new(
            format!("synth-t-{i}"),
            format!("synth-c-{i}"),
            "synthetic true claim",
            true,
            0.9,
            1_700_000_000 + (i as i64),
        ));
        // False claim with low confidence.
        let _ = c.append(CalibrationTuple::new(
            format!("synth-f-{i}"),
            format!("synth-d-{i}"),
            "synthetic false claim",
            false,
            0.2,
            1_700_000_500 + (i as i64),
        ));
    }
    c.freeze();
    c
}

fn render_report(
    output: Option<&str>,
    report: &ConformalValidationReport,
) -> Result<(), BenchConformalError> {
    let body = serde_json::to_string_pretty(report).map_err(|e| BenchConformalError::Serialise {
        source_msg: format!("{e}"),
    })?;
    match output {
        Some(path) => {
            std::fs::write(path, &body).map_err(|e| BenchConformalError::OutputIo {
                path: path.to_string(),
                source_msg: format!("{e}"),
            })?;
            println!(
                "CONFORMAL {} (tau={:.3}, empirical={:.3}, epsilon={:.3}, total={}); report written to {}",
                verdict_label(report),
                report.tau,
                report.empirical_refusal_rate,
                report.epsilon,
                report.total,
                path,
            );
        }
        None => {
            println!("{body}");
            eprintln!(
                "CONFORMAL {} (tau={:.3}, empirical={:.3}, epsilon={:.3}, total={})",
                verdict_label(report),
                report.tau,
                report.empirical_refusal_rate,
                report.epsilon,
                report.total,
            );
        }
    }
    Ok(())
}

fn verdict_label(report: &ConformalValidationReport) -> String {
    if report.passed() {
        "PASS".to_string()
    } else {
        "FAIL".to_string()
    }
}

// ---------------------------------------------------------------------------
// Plan-named helper fns (D-22 closure shim)
//
// The D-22 plan section "MODIFY: .elai_cc/crates/cli/src/commands.rs
// (SlashCommand::BenchConformalValidate)" called for a `status` + `validate`
// pair on the conformal-validation CLI surface. The shipped substrate routes
// dispatch through `CliAction::BenchConformal` -> the module-private
// `run(...)` above; no separate SlashCommand variant landed (see
// workflow/reasoning/D-22-decisions.md DECISION-5).
//
// `status()` and `validate()` below are thin pub helpers exposing the
// plan-named entry points so integration tests + downstream tooling can
// drive the benchmark without going through the `std::process::exit` path
// of `run(...)`. They are additive — they call into the existing
// `run_async` core; no behavior change.
// ---------------------------------------------------------------------------

/// D-22 plan-named `status` entry point.
///
/// Returns a freshly computed `ConformalValidationReport` against the
/// supplied dataset + (optional) calibration corpus. Mirrors
/// `bench conformal --dataset ... --corpus ... --epsilon ...` without
/// rendering JSON or exiting the process. Construction is a single
/// current-thread tokio runtime; safe to call from sync context.
///
/// Drift note: the plan named this `status()` to mirror the
/// `conformal-calibration status` shape. The shipped substrate routes the
/// same call through `CliAction::BenchConformal { dataset, corpus, output,
/// epsilon }`. See DECISION-4 / DECISION-5 in
/// workflow/reasoning/D-22-decisions.md.
pub fn status(
    dataset_path: PathBuf,
    corpus_path: Option<PathBuf>,
    epsilon: Option<f64>,
) -> Result<ConformalValidationReport, BenchConformalError> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| BenchConformalError::OutputIo {
            path: "<tokio runtime>".to_string(),
            source_msg: format!("{e}"),
        })?;
    rt.block_on(async move { run_async(dataset_path, corpus_path, epsilon).await })
}

/// D-22 plan-named `validate` entry point.
///
/// Runs the conformal-validation benchmark and returns `Ok(true)` iff the
/// report's `gating_verdict` is `Pass`. Mirrors the
/// `elai bench conformal-validate` exit-code semantics the plan called
/// for, without invoking `std::process::exit`. Pure delegation to the
/// shipped `run_async` core path.
pub fn validate(
    dataset_path: PathBuf,
    corpus_path: Option<PathBuf>,
    epsilon: Option<f64>,
) -> Result<bool, BenchConformalError> {
    let report = status(dataset_path, corpus_path, epsilon)?;
    Ok(report.passed())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    fn write_mini_fixture(dir: &TempDir) -> PathBuf {
        let path = dir.path().join("fixture.jsonl");
        let mut f = std::fs::File::create(&path).expect("create fixture");
        // 5 rows per bucket, all high-score -> empirical rate ~ 0.0, in
        // band, gate Pass.
        for bucket in ["in_domain", "near_ood", "far_ood"] {
            for i in 0..5 {
                let line = format!(
                    "{{\"row_id\":\"{bucket}_{i}\",\"distribution\":\"{bucket}\",\"domain\":\"code\",\"predicted_score\":0.95,\"ground_truth\":true}}"
                );
                writeln!(f, "{line}").expect("write fixture line");
            }
        }
        path
    }

    #[tokio::test]
    async fn run_async_against_balanced_fixture_returns_pass() {
        let dir = TempDir::new().expect("tempdir");
        let path = write_mini_fixture(&dir);
        let report = run_async(path, None, None).await.expect("run ok");
        assert!(report.passed());
        assert_eq!(report.total, 15);
    }

    #[test]
    fn status_shim_returns_report_against_balanced_fixture() {
        // D-22 closure: plan-named `status()` helper must hand back the
        // ConformalValidationReport without rendering / exiting.
        let dir = TempDir::new().expect("tempdir");
        let path = write_mini_fixture(&dir);
        let report = status(path, None, None).expect("status ok");
        assert!(report.passed());
        assert_eq!(report.total, 15);
        assert!(report.tau.is_finite());
    }

    #[test]
    fn validate_shim_returns_true_against_balanced_fixture() {
        // D-22 closure: plan-named `validate()` helper must return true
        // iff the gating verdict is Pass.
        let dir = TempDir::new().expect("tempdir");
        let path = write_mini_fixture(&dir);
        let passed = validate(path, None, None).expect("validate ok");
        assert!(passed);
    }
}

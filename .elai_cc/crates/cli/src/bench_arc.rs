//! D-20 — `elai bench arc` CLI driver.
//!
//! Surfaces [`elai_orchestrator::benchmarks::arc_agi2_runner::ARCAGI2Runner`]
//! as a user-facing subcommand + CI hook. The runner replays a hand-
//! labeled JSONL fixture through D-10's pure predicate
//! [`elai_orchestrator::grounding_mode::select_grounding_mode`] AND
//! D-15's pure predicate [`elai_orchestrator::novelty::classify`], then
//! aggregates the pair-wise outcomes into the closed
//! [`elai_orchestrator::benchmarks::arc_agi2_runner::ARCVerdict`]
//! taxonomy.
//!
//! # Wiring discipline
//!
//! producer  : `ARCAGI2Runner::run_all` (orchestrator-side).
//! consumer  : this file — owns its tokio runtime + emits one SSB row
//!             via `VecARCSsbEmitter`. CI consumes the exit code
//!             (0 = HonestlyEvaluated, 1 = anything else).
//! ssb row   : `SsbActionType::ArcBenchmarkCompleted` (one per run).
//!
//! # Exit codes
//!
//! 0 = report verdict == HonestlyEvaluated
//! 1 = report verdict in {GameWithMemorization, Refused} OR any
//!     infrastructural error
//!
//! # Flags
//!
//!   --dataset PATH        JSONL fixture (default
//!                         `<cwd>/benchmarks/arc_agi2/arc_mini_fixture.jsonl`)
//!   --output PATH         Write the JSON report to PATH (default stdout).
//!   --novelty-config PATH JSON file with `NoveltyConfig` overrides
//!                         (search_dkg_min, near_duplicate_high,
//!                         near_duplicate_low). Default: built-in
//!                         `NoveltyConfig::default`.
//!
//! No `.unwrap()` in non-test code; thiserror for the error enum;
//! hand-rolled flag parsing (no new clap dep).

use std::path::PathBuf;

use elai_orchestrator::benchmarks::arc_agi2_runner::{
    default_mini_fixture_path, ARCAGI2Runner, ARCReport, ARCRunnerError, VecARCSsbEmitter,
};
use elai_orchestrator::novelty::NoveltyConfig;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BenchArcError {
    /// Underlying runner failure (I/O, malformed JSON, label, SSB emit).
    #[error("arc-agi2 runner error: {0}")]
    Runner(#[from] ARCRunnerError),
    /// Generic I/O failure writing the JSON report to `--output`.
    #[error("arc-agi2 could not write report to {path}: {source_msg}")]
    OutputIo { path: String, source_msg: String },
    /// JSON serialisation failure when rendering the report.
    #[error("arc-agi2 could not serialise report: {source_msg}")]
    Serialise { source_msg: String },
    /// `--novelty-config PATH` failed to read or deserialise.
    #[error("arc-agi2 could not read --novelty-config {path}: {source_msg}")]
    NoveltyConfigIo { path: String, source_msg: String },
    /// `--novelty-config` thresholds rejected by
    /// [`NoveltyConfig::validated`].
    #[error("arc-agi2 --novelty-config invalid: {source_msg}")]
    InvalidNoveltyConfig { source_msg: String },
}

/// CLI entry — parses + validates flags, runs the benchmark, renders,
/// exits. Calls `std::process::exit(1)` on FAIL or any handled error.
/// `Err(anyhow::Error)` is reserved for infrastructural failures the
/// top-level dispatcher renders.
pub(crate) fn run(
    dataset: Option<String>,
    output: Option<String>,
    novelty_config: Option<String>,
) -> anyhow::Result<()> {
    let project_root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let dataset_path = dataset
        .map(PathBuf::from)
        .unwrap_or_else(|| default_mini_fixture_path(&project_root));

    let cfg = parse_novelty_config(novelty_config.as_deref())?;

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(anyhow::Error::from)?;

    let now_ms = now_ms_safe();
    let outcome = rt.block_on(async move { run_async(dataset_path, cfg, now_ms).await });

    match outcome {
        Ok(report) => {
            render_report(output.as_deref(), &report)?;
            if !report.passed() {
                std::process::exit(1);
            }
            Ok(())
        }
        Err(e) => {
            eprintln!("ARC-AGI2 ABORT: {e}");
            std::process::exit(1);
        }
    }
}

/// UNIX ms — wraps `SystemTime::now` with a deterministic fallback so
/// the runner stays callable in environments where the clock jumps
/// pre-epoch (CI image clocks).
fn now_ms_safe() -> i64 {
    match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        Ok(d) => i64::try_from(d.as_millis()).unwrap_or(i64::MAX),
        Err(_) => 0,
    }
}

fn parse_novelty_config(path: Option<&str>) -> Result<NoveltyConfig, BenchArcError> {
    let Some(p) = path else {
        return Ok(NoveltyConfig::default());
    };
    let raw = std::fs::read_to_string(p).map_err(|e| BenchArcError::NoveltyConfigIo {
        path: p.to_string(),
        source_msg: format!("{e}"),
    })?;
    let cfg: NoveltyConfig =
        serde_json::from_str(&raw).map_err(|e| BenchArcError::NoveltyConfigIo {
            path: p.to_string(),
            source_msg: format!("{e}"),
        })?;
    cfg.validated()
        .map_err(|e| BenchArcError::InvalidNoveltyConfig {
            source_msg: format!("{e}"),
        })
}

/// Async core — constructs the runner, runs the benchmark, returns the
/// report. Public so integration tests can drive it without
/// `std::process::exit`.
pub async fn run_async(
    dataset_path: PathBuf,
    cfg: NoveltyConfig,
    now_ms: i64,
) -> Result<ARCReport, BenchArcError> {
    let runner = ARCAGI2Runner::new(dataset_path, now_ms).with_novelty_config(cfg);
    let emitter = VecARCSsbEmitter::new();
    let report = runner.run_all(&emitter).await?;
    Ok(report)
}

fn render_report(output: Option<&str>, report: &ARCReport) -> Result<(), BenchArcError> {
    let body = serde_json::to_string_pretty(report).map_err(|e| BenchArcError::Serialise {
        source_msg: format!("{e}"),
    })?;
    match output {
        Some(path) => {
            std::fs::write(path, &body).map_err(|e| BenchArcError::OutputIo {
                path: path.to_string(),
                source_msg: format!("{e}"),
            })?;
            println!(
                "ARC-AGI2 {} (total={}, honest={}, dominant_mode={}, dominant_class={}); report written to {}",
                verdict_label(report),
                report.total,
                report.honestly_evaluated_count,
                report.dominant_mode,
                report.dominant_novelty_class,
                path,
            );
        }
        None => {
            println!("{body}");
            eprintln!(
                "ARC-AGI2 {} (total={}, honest={}, dominant_mode={}, dominant_class={})",
                verdict_label(report),
                report.total,
                report.honestly_evaluated_count,
                report.dominant_mode,
                report.dominant_novelty_class,
            );
        }
    }
    Ok(())
}

fn verdict_label(report: &ARCReport) -> String {
    if report.passed() {
        "PASS (honestly_evaluated)".to_string()
    } else {
        format!("FAIL ({})", report.gating_verdict.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    fn write_minimal_fixture(dir: &TempDir) -> PathBuf {
        let path = dir.path().join("fixture.jsonl");
        let mut f = std::fs::File::create(&path).expect("create fixture");
        // 5 hypothetical claims with 0 DKG-match + low near-duplicate
        // -> D-10 routes Hypothetical to OperationalDeferred (rule 7);
        // D-15 classifies as DiscoveryCandidate; pair = HonestlyEvaluated.
        let lines = [
            r#"{"task_id":"t1","claim_text":"x","claim_type":"hypothetical","expected_novelty_class":"discovery_candidate","dkg_match_count":0,"near_duplicate_threshold":0.05}"#,
            r#"{"task_id":"t2","claim_text":"x","claim_type":"hypothetical","expected_novelty_class":"discovery_candidate","dkg_match_count":0,"near_duplicate_threshold":0.1}"#,
            r#"{"task_id":"t3","claim_text":"x","claim_type":"hypothetical","expected_novelty_class":"discovery_candidate","dkg_match_count":0,"near_duplicate_threshold":0.15}"#,
            r#"{"task_id":"t4","claim_text":"x","claim_type":"hypothetical","expected_novelty_class":"discovery_candidate","dkg_match_count":0,"near_duplicate_threshold":0.2}"#,
            r#"{"task_id":"t5","claim_text":"x","claim_type":"hypothetical","expected_novelty_class":"discovery_candidate","dkg_match_count":0,"near_duplicate_threshold":0.25}"#,
        ];
        for line in lines {
            writeln!(f, "{line}").expect("write fixture line");
        }
        path
    }

    #[tokio::test]
    async fn run_async_against_honest_fixture_returns_honestly_evaluated() {
        let dir = TempDir::new().expect("tempdir");
        let path = write_minimal_fixture(&dir);
        let report = run_async(path, NoveltyConfig::default(), 1_700_000_300_000)
            .await
            .expect("run ok");
        assert!(report.passed(), "expected passed, got {:?}", report.gating_verdict);
        assert_eq!(report.total, 5);
    }

    #[tokio::test]
    async fn run_async_with_invalid_novelty_config_path_fails_cleanly() {
        let err = parse_novelty_config(Some("/definitely/does/not/exist.json")).unwrap_err();
        match err {
            BenchArcError::NoveltyConfigIo { .. } => {}
            other => panic!("expected NoveltyConfigIo, got {other:?}"),
        }
    }

    #[test]
    fn parse_novelty_config_returns_default_when_path_none() {
        let cfg = parse_novelty_config(None).expect("ok");
        let default = NoveltyConfig::default();
        assert_eq!(cfg.search_dkg_min, default.search_dkg_min);
        assert!((cfg.near_duplicate_high - default.near_duplicate_high).abs() < 1e-9);
        assert!((cfg.near_duplicate_low - default.near_duplicate_low).abs() < 1e-9);
    }
}

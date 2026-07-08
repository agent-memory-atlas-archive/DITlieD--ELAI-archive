//! K-18a — `elai bench case-reuse [--suite fixture] [--seed <u64>] [--json]` CLI driver.
//!
//! Surfaces the PS-5 recurrence-hit-rate OBSERVE arm as a user-facing subcommand
//! + CI hook. DRIVES the SHIPPED `case_reuse_runner` (orchestrator) over the
//! embedded fixture corpus and the real `elai_knowledge::bm25` retrieval stack,
//! then writes a timestamped, blake3-pinned JSON report under
//! `<cwd>/benchmarks/case_reuse/`.
//!
//! # What this measures (honestly, GD-5)
//!
//! `recurrence_hit_rate@{1,3,5}` (can the shipped retrieval find a proven case
//! when its bug class recurs) + `false_match_rate@{1,3,5}` (the false-positive
//! control). retrieval_mode = "bm25_only" (F-4; hybrid RRF not staged). ZERO
//! reuse-into-generation — the compounding iteration/$ delta is K-18 Task 2,
//! BENCH-PENDING on the unbuilt VCG J-track.
//!
//! # SSB beacon: DEFERRED (ABORT-4)
//!
//! Every sibling bench emits a bespoke typed SSB row (SafetyDeltaMeasured,
//! ConvergenceCurveMeasured, ...). There is NO reusable/generic bench SSB row,
//! so a recurrence-hit-rate beacon needs a NEW `SsbActionType` — a coordinated
//! schema-bump (enum + payload + action_type arm + golden lock + SCHEMA_VERSION)
//! the K-18a plan explicitly walls off (ABORT-4: "not a drive-by"). The beacon
//! + the G-2 .rra seal are therefore DEFERRED to a follow-on; this session's
//! off-agent evidence (R4/GD-2) is the blake3-pinned, seed-recomputable JSON
//! report written below.
//!
//! # No `.unwrap()` / `.expect()` in non-test paths.

use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use elai_orchestrator::benchmarks::case_reuse_runner::{
    load_embedded_corpus, run_case_reuse, CaseReuseCorpus, CaseReuseError, CaseReuseReport,
};
use elai_tools::failure_taxonomy::{classify_failure_category, FailureCategory};
use thiserror::Error;

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum BenchCaseReuseError {
    /// The case-reuse runner failed (parse / empty corpus / retrieval).
    #[error("case-reuse runner error: {0}")]
    Runner(#[from] CaseReuseError),
    /// An unsupported suite was requested. Only `fixture` is runnable today; the
    /// real-journal `journal` suite is F-2 deferred (a separately-gated checkbox).
    #[error("case-reuse unsupported suite '{suite}': only 'fixture' is runnable (the 'journal' suite is F-2 deferred, RunPod/journal-gated)")]
    UnsupportedSuite { suite: String },
    /// Could not determine the current working directory for the report dir.
    #[error("case-reuse could not determine cwd: {source_msg}")]
    Cwd { source_msg: String },
    /// Could not create the on-disk report directory.
    #[error("case-reuse could not create dir {path}: {source_msg}")]
    ReportDir { path: String, source_msg: String },
    /// Generic I/O failure writing the JSON report.
    #[error("case-reuse could not write {path}: {source_msg}")]
    OutputIo { path: String, source_msg: String },
    /// JSON serialisation failure.
    #[error("case-reuse could not serialise report: {source_msg}")]
    Serialise { source_msg: String },
    /// A fixture carries a `failure_category` that is not a real FailureCategory
    /// enum variant.
    #[error("case-reuse fixture '{id}' has an invalid failure_category '{category}' (not a FailureCategory variant)")]
    CorpusLabelInvalid { id: String, category: String },
    /// A fixture's baked `failure_category` does not match what the REAL
    /// classifier produces for its own error_output (a hand-typed / drifted
    /// label). Corpus-integrity gate — every label must be classifier-generated.
    #[error("case-reuse fixture '{id}' label drift: baked {baked} != real classifier output {classified} (labels must be classifier-generated, K-18a Move 2)")]
    CorpusLabelMismatch {
        id: String,
        baked: String,
        classified: String,
    },
}

// ---------------------------------------------------------------------------
// CLI entry
// ---------------------------------------------------------------------------

/// CLI entry — runs the recurrence-hit-rate observe arm, writes a timestamped
/// JSON report, prints a summary (or the full report JSON with `--json`).
/// On a handled error prints `CASE-REUSE ABORT:` to stderr and exits 1.
pub(crate) fn run(suite: String, seed: u64, json: bool) -> anyhow::Result<()> {
    match run_inner(suite, seed, json) {
        Ok(()) => Ok(()),
        Err(e) => {
            eprintln!("CASE-REUSE ABORT: {e}");
            std::process::exit(1);
        }
    }
}

/// Core — drives the runner, writes the JSON report. Public so an integration
/// test can drive it without the process exit.
pub fn run_inner(suite: String, seed: u64, json: bool) -> Result<(), BenchCaseReuseError> {
    if suite != "fixture" {
        return Err(BenchCaseReuseError::UnsupportedSuite { suite });
    }

    // K-18a corpus-integrity gate (production wiring for the real classifier):
    // before measuring, confirm every fixture `failure_category` is what the
    // SHIPPED elai_tools classifier produces for its own error_output. This makes
    // the "labels are classifier-generated, never hand-typed" guarantee a RUNTIME
    // invariant (not just a test), and is the production caller of
    // classify_failure_category (D-030: no wire-dark pub fn).
    let corpus = load_embedded_corpus()?;
    verify_corpus_labels(&corpus)?;

    // Drive the shipped runner over the embedded corpus (real BM25 stack).
    // match_everything is always false from the CLI; the negative control is a
    // test-only arm (cfg(test), K-18a Move 5c).
    let report = run_case_reuse(seed, false)?;

    let project_root = std::env::current_dir().map_err(|e| BenchCaseReuseError::Cwd {
        source_msg: e.to_string(),
    })?;
    let report_path = write_json_report(&project_root, &report)?;

    println!(
        "CASE-REUSE suite={} retrieval_mode={} n_proven={} n_recurrences={} n_distractors={} \
         classes={} hit@1={:.4} hit@3={:.4} hit@5={:.4} fm@1={:.4} fm@3={:.4} fm@5={:.4} \
         ci=[{:.4},{:.4}] seed={} corpus_blake3={} report={} ssb_beacon=DEFERRED(ABORT-4)",
        report.suite,
        report.retrieval_mode,
        report.n_proven,
        report.n_recurrences,
        report.n_distractors,
        report.class_count,
        hit_rate_at(&report, 1),
        hit_rate_at(&report, 3),
        hit_rate_at(&report, 5),
        false_match_at(&report, 1),
        false_match_at(&report, 3),
        false_match_at(&report, 5),
        report.hit_rate_ci_low,
        report.hit_rate_ci_high,
        report.seed,
        report.corpus_blake3,
        report_path.display(),
    );

    if json {
        let body = serde_json::to_string_pretty(&report).map_err(|e| {
            BenchCaseReuseError::Serialise {
                source_msg: e.to_string(),
            }
        })?;
        println!("{body}");
    }
    Ok(())
}

/// Corpus-integrity gate: every fixture `failure_category` must equal what the
/// REAL `elai_tools` classifier produces for that record's `error_output`. This
/// is the production caller of [`classify_failure_category`] and enforces the
/// K-18a Move-2 "labels are classifier-generated, never hand-typed" contract at
/// run time (fail-loud on the first drift).
fn verify_corpus_labels(corpus: &CaseReuseCorpus) -> Result<(), BenchCaseReuseError> {
    fn check(id: &str, category: &str, error_output: &str) -> Result<(), BenchCaseReuseError> {
        let baked: FailureCategory =
            serde_json::from_str(&format!("\"{category}\"")).map_err(|_| {
                BenchCaseReuseError::CorpusLabelInvalid {
                    id: id.to_string(),
                    category: category.to_string(),
                }
            })?;
        let classified = classify_failure_category(error_output);
        if classified != baked {
            return Err(BenchCaseReuseError::CorpusLabelMismatch {
                id: id.to_string(),
                baked: format!("{baked:?}"),
                classified: format!("{classified:?}"),
            });
        }
        Ok(())
    }
    for pc in &corpus.proven_cases {
        check(&pc.id, &pc.failure_category, &pc.error_output)?;
    }
    for rec in &corpus.recurrences {
        check(&rec.id, &rec.failure_category, &rec.error_output)?;
    }
    for d in &corpus.distractors {
        check(&d.id, &d.failure_category, &d.error_output)?;
    }
    Ok(())
}

fn hit_rate_at(report: &CaseReuseReport, k: u32) -> f64 {
    report
        .per_k
        .iter()
        .find(|p| p.k == k)
        .map(|p| p.recurrence_hit_rate)
        .unwrap_or(0.0)
}

fn false_match_at(report: &CaseReuseReport, k: u32) -> f64 {
    report
        .per_k
        .iter()
        .find(|p| p.k == k)
        .map(|p| p.false_match_rate)
        .unwrap_or(0.0)
}

// ---------------------------------------------------------------------------
// JSON report writer
// ---------------------------------------------------------------------------

/// Write the report JSON to
/// `<project_root>/benchmarks/case_reuse/case-reuse-<unix_ms>.json` and return
/// the written path. Mirrors bench_safedelta::write_json_report.
fn write_json_report(
    project_root: &std::path::Path,
    report: &CaseReuseReport,
) -> Result<PathBuf, BenchCaseReuseError> {
    let report_dir = project_root.join("benchmarks").join("case_reuse");
    fs::create_dir_all(&report_dir).map_err(|e| BenchCaseReuseError::ReportDir {
        path: report_dir.display().to_string(),
        source_msg: e.to_string(),
    })?;

    let unix_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or_default();
    let report_path = report_dir.join(format!("case-reuse-{unix_ms}.json"));

    let body = serde_json::to_string_pretty(report).map_err(|e| BenchCaseReuseError::Serialise {
        source_msg: e.to_string(),
    })?;
    fs::write(&report_path, &body).map_err(|e| BenchCaseReuseError::OutputIo {
        path: report_path.display().to_string(),
        source_msg: e.to_string(),
    })?;
    Ok(report_path)
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unsupported_suite_is_rejected() {
        let err = run_inner("journal".to_string(), 1234, false).unwrap_err();
        assert!(
            matches!(err, BenchCaseReuseError::UnsupportedSuite { .. }),
            "non-fixture suite must be rejected, got {err}"
        );
    }

    #[test]
    fn write_json_report_creates_file_under_benchmarks_case_reuse() {
        let report = run_case_reuse(1234, false).expect("run");
        let dir = tempfile::TempDir::new().expect("tempdir");
        let path = write_json_report(dir.path(), &report).expect("write");
        assert!(path.exists(), "report file must exist");
        let on_disk = path.to_string_lossy();
        assert!(
            on_disk.contains("benchmarks") && on_disk.contains("case_reuse"),
            "report must live under benchmarks/case_reuse/, got {on_disk}"
        );
        let text = fs::read_to_string(&path).expect("read");
        let parsed: CaseReuseReport =
            serde_json::from_str(&text).expect("round-trips as CaseReuseReport");
        assert_eq!(parsed.retrieval_mode, "bm25_only");
    }

    /// K-18a Move 2 realness contract: every fixture `failure_category` label is
    /// GENERATED by the real elai_tools classifier over its own error_output —
    /// never a hand-typed string. Drives the SHIPPED classifier and asserts the
    /// baked label matches for every proven case, recurrence, and distractor.
    #[test]
    fn fixture_labels_are_real_classifier_output() {
        let corpus = load_embedded_corpus().expect("corpus");

        let check = |id: &str, category: &str, error_output: &str| {
            let baked: FailureCategory = serde_json::from_str(&format!("\"{category}\""))
                .unwrap_or_else(|_| panic!("fixture '{id}' has non-enum failure_category '{category}'"));
            let classified = classify_failure_category(error_output);
            assert_eq!(
                classified, baked,
                "fixture '{id}': baked label {category} != real classifier output {classified:?} (label must be classifier-generated, not hand-typed)"
            );
        };

        for pc in &corpus.proven_cases {
            check(&pc.id, &pc.failure_category, &pc.error_output);
        }
        for rec in &corpus.recurrences {
            check(&rec.id, &rec.failure_category, &rec.error_output);
        }
        for d in &corpus.distractors {
            check(&d.id, &d.failure_category, &d.error_output);
        }
    }
}

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
//! # SSB beacon (SCHEMA_VERSION 270, K-18a follow-on)
//!
//! K-18a shipped the runner + JSON report but explicitly DEFERRED the SSB
//! beacon (ABORT-4: a recurrence-hit-rate beacon needs a NEW `SsbActionType`,
//! a coordinated schema-bump the K-18a plan walled off as "not a drive-by").
//! This follow-on lands that schema-bump (enum + payload + action_type arm +
//! golden lock + SCHEMA_VERSION 270) and emits ONE `RecurrenceHitRateMeasured`
//! beacon + a signed `.rra` manifest per clean run, restoring GD-2
//! (journal-recomputability) + dashboard-ability on top of the
//! blake3-pinned, seed-recomputable JSON report already written below.
//!
//! # No `.unwrap()` / `.expect()` in non-test paths.

use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use elai_core::paths::sessions_dir_path;
use elai_core::OsClock;
use elai_orchestrator::benchmarks::case_reuse_runner::{
    load_embedded_corpus, run_case_reuse, CaseReuseCorpus, CaseReuseError, CaseReuseReport,
};
use elai_orchestrator::determinism::DeterminismConfig;
use elai_orchestrator::rra::{build_rra, write_rra};
use elai_orchestrator::ssb::{RecurrenceHitRateMeasuredPayload, SsbReader as _};
use elai_orchestrator::{Ssb, SsbConfig, SsbPayload};
use elai_tools::failure_taxonomy::{classify_failure_category, FailureCategory};
use thiserror::Error;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Bench session id embedded in every SSB row this bench emits.
const BENCH_SESSION_ID: &str = "cli-bench-case-reuse";

/// Agent id label for SSB rows (mirrors bench_safedelta pattern).
const BENCH_AGENT_ID: &str = "cli:bench-case-reuse";

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
    /// Opening the SSB single-writer failed.
    #[error("case-reuse could not open SSB writer: {source_msg}")]
    SsbOpen { source_msg: String },
    /// Writing the RecurrenceHitRateMeasured beacon failed.
    #[error("case-reuse could not write SSB beacon: {source_msg}")]
    SsbWrite { source_msg: String },
    /// Resolving determinism pins (seed / hardware class / fingerprint) failed.
    #[error("case-reuse could not resolve determinism pins: {source_msg}")]
    DeterminismPins { source_msg: String },
    /// Querying the SSB session stream (for the .rra) failed.
    #[error("case-reuse could not query SSB session stream: {source_msg}")]
    SsbStreamQuery { source_msg: String },
    /// Writing the .rra manifest failed.
    #[error("case-reuse could not write .rra manifest to {path}: {source_msg}")]
    RraWrite { path: String, source_msg: String },
    /// Building the async runtime failed.
    #[error("case-reuse could not build async runtime: {source_msg}")]
    Runtime { source_msg: String },
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

    // Emit EXACTLY ONE RecurrenceHitRateMeasured SSB beacon + seal the signed
    // .rra manifest (SCHEMA_VERSION 270, K-18a follow-on; mirrors
    // bench_safedelta.rs::emit_beacon_and_rra).
    let rra_path_buf = rra_path(
        &project_root,
        report_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("case-reuse-unknown"),
    );
    emit_beacon_and_rra(&project_root, &rra_path_buf, &report)?;

    println!(
        "CASE-REUSE suite={} retrieval_mode={} n_proven={} n_recurrences={} n_distractors={} \
         classes={} hit@1={:.4} hit@3={:.4} hit@5={:.4} fm@1={:.4} fm@3={:.4} fm@5={:.4} \
         ci=[{:.4},{:.4}] seed={} corpus_blake3={} report={} rra={}",
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
        rra_path_buf.display(),
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
// .rra path helper
// ---------------------------------------------------------------------------

/// Build the .rra manifest path adjacent to the JSON report.
fn rra_path(project_root: &std::path::Path, stem: &str) -> PathBuf {
    let stem_no_ext = stem.trim_end_matches(".json");
    project_root
        .join("benchmarks")
        .join("case_reuse")
        .join(format!("{stem_no_ext}.rra"))
}

// ---------------------------------------------------------------------------
// SSB emit + .rra seal
// ---------------------------------------------------------------------------

/// Open the canonical session `ssb.db`, emit ONE `RecurrenceHitRateMeasured`
/// beacon (durable, awaited), then query the session stream and seal a signed
/// `.rra` manifest via G-2 `build_rra`/`write_rra` (blake3-only root, SC-1).
///
/// `Ssb::new` spawns the single-writer task and MUST be constructed inside
/// `rt.block_on` (F-2b playbook + bench_safedelta.rs::emit_beacon_and_rra
/// precedent — the writer panics "no reactor running" outside the runtime).
fn emit_beacon_and_rra(
    project_root: &std::path::Path,
    rra_path: &std::path::Path,
    report: &CaseReuseReport,
) -> Result<(), BenchCaseReuseError> {
    let db_path = sessions_dir_path(project_root).join("ssb.db");
    if let Some(parent) = db_path.parent() {
        fs::create_dir_all(parent).map_err(|e| BenchCaseReuseError::ReportDir {
            path: parent.display().to_string(),
            source_msg: e.to_string(),
        })?;
    }

    let cfg = SsbConfig {
        db_path,
        pool_size: None,
        project_id: "default".to_string(),
        session_id: BENCH_SESSION_ID.to_string(),
    };
    let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| BenchCaseReuseError::Runtime {
            source_msg: e.to_string(),
        })?;

    let ts_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or_default();

    let payload = RecurrenceHitRateMeasuredPayload {
        suite: report.suite.clone(),
        retrieval_mode: report.retrieval_mode.clone(),
        recurrence_hit_rate_at1: hit_rate_at(report, 1),
        recurrence_hit_rate_at3: hit_rate_at(report, 3),
        recurrence_hit_rate_at5: hit_rate_at(report, 5),
        false_match_rate_at1: false_match_at(report, 1),
        false_match_rate_at3: false_match_at(report, 3),
        false_match_rate_at5: false_match_at(report, 5),
        hit_rate_ci_low: report.hit_rate_ci_low,
        hit_rate_ci_high: report.hit_rate_ci_high,
        n_recurrences: report.n_recurrences,
        n_distractors: report.n_distractors,
        class_count: report.class_count,
        seed: report.seed,
        corpus_blake3: report.corpus_blake3.clone(),
        ts_ms,
    };

    // Resolve determinism pins outside the runtime (sync call).
    let pins = DeterminismConfig::from_env().map_err(|e| BenchCaseReuseError::DeterminismPins {
        source_msg: e.to_string(),
    })?;

    // Ssb::new + write_payload + query + build_rra all run inside the same
    // current-thread runtime (MUST — "no reactor running" if outside).
    rt.block_on(async move {
        let ssb = Ssb::new(cfg, clock).map_err(|e| BenchCaseReuseError::SsbOpen {
            source_msg: e.to_string(),
        })?;

        // Emit the RecurrenceHitRateMeasured beacon row.
        ssb.write_payload(
            BENCH_AGENT_ID,
            0,
            SsbPayload::RecurrenceHitRateMeasured(payload),
            Vec::new(),
        )
        .await
        .map_err(|e| BenchCaseReuseError::SsbWrite {
            source_msg: e.to_string(),
        })?;

        // Read the session stream for the .rra seal (G-2 reuse, SC-1).
        let stream = ssb
            .query_session_stream_asc(BENCH_SESSION_ID)
            .map_err(|e| BenchCaseReuseError::SsbStreamQuery {
                source_msg: e.to_string(),
            })?;

        // Build + write the signed, independently-replayable .rra manifest
        // (blake3-only root, GD-2: re-verifiable from the pinned stream + pins).
        let artifact = build_rra(BENCH_SESSION_ID, &stream, &pins);
        write_rra(&artifact, rra_path).map_err(|e| BenchCaseReuseError::RraWrite {
            path: rra_path.display().to_string(),
            source_msg: e.to_string(),
        })?;

        tracing::info!(
            recurrence_hit_rate_at5 = hit_rate_at(report, 5),
            rra_path = %rra_path.display(),
            root_blake3 = %artifact.root_blake3,
            "case-reuse: K-18a follow-on RecurrenceHitRateMeasured SSB row + .rra sealed",
        );

        Ok::<(), BenchCaseReuseError>(())
    })
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use elai_orchestrator::ssb::{SsbActionType, SsbReader as _};

    /// K-18a follow-on run-trigger contract: `emit_beacon_and_rra` writes
    /// EXACTLY ONE `RecurrenceHitRateMeasured` row to a fresh session `ssb.db`
    /// AND a signed `.rra` manifest, and the row's decoded payload mirrors the
    /// report it was built from (R-8/D-030: the off-agent evidence a real
    /// entrypoint drove, not this test's own stdout).
    #[test]
    fn emit_beacon_and_rra_writes_row_and_manifest() {
        let report = run_case_reuse(1234, false).expect("run_case_reuse");
        let dir = tempfile::TempDir::new().expect("tempdir");
        let rra = rra_path(dir.path(), "case-reuse-test.json");

        emit_beacon_and_rra(dir.path(), &rra, &report).expect("emit_beacon_and_rra");

        assert!(rra.exists(), ".rra manifest must exist on disk");

        // Re-open the same db_path emit_beacon_and_rra just wrote to, on a
        // fresh Ssb read/write handle inside its own current-thread runtime
        // (Ssb::new spawns its writer task and MUST be constructed inside a
        // runtime; query_by_type itself is sync).
        let db_path = elai_core::paths::sessions_dir_path(dir.path()).join("ssb.db");
        let cfg = SsbConfig {
            db_path,
            pool_size: None,
            project_id: "default".to_string(),
            session_id: BENCH_SESSION_ID.to_string(),
        };
        let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("build runtime");

        let rows = rt.block_on(async move {
            let ssb = Ssb::new(cfg, clock).expect("open ssb.db");
            ssb.query_by_type(SsbActionType::RecurrenceHitRateMeasured)
                .expect("query_by_type")
        });
        assert_eq!(rows.len(), 1, "exactly one RecurrenceHitRateMeasured row");

        let decoded: SsbPayload =
            elai_orchestrator::ssb::decode_ssb_payload(&rows[0].payload).expect("decode payload");
        match decoded {
            SsbPayload::RecurrenceHitRateMeasured(p) => {
                assert_eq!(p.suite, report.suite);
                assert_eq!(p.retrieval_mode, report.retrieval_mode);
                assert_eq!(p.seed, report.seed);
                assert_eq!(p.corpus_blake3, report.corpus_blake3);
                assert_eq!(p.n_recurrences, report.n_recurrences);
                assert_eq!(p.n_distractors, report.n_distractors);
                assert_eq!(p.class_count, report.class_count);
                assert_eq!(p.recurrence_hit_rate_at1, hit_rate_at(&report, 1));
                assert_eq!(p.recurrence_hit_rate_at5, hit_rate_at(&report, 5));
                assert_eq!(p.hit_rate_ci_low, report.hit_rate_ci_low);
                assert_eq!(p.hit_rate_ci_high, report.hit_rate_ci_high);
            }
            other => panic!("wrong payload variant decoded: {other:?}"),
        }
    }

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

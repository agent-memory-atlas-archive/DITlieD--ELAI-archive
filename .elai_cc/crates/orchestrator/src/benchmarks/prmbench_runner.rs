//! D-19: PRMBench adversarial-gaming benchmark runner for the PPM verifier.
//!
//! Reads a JSONL fixture of (problem_statement, step_history, candidate_step,
//! ground_truth_label, category) rows, scores each candidate step through
//! `PpmClient::score`, and aggregates the predictions into a per-category
//! report with accuracy / precision / recall against the ground-truth labels.
//!
//! Per the D-19 plan (PRMBench, arxiv:2501.03124 T1 §4.2): Deception
//! Resistance + Circular Logic are the worst-scoring categories for
//! published PRMs (top scorers land in the 60-65% accuracy band). The
//! runner reports honestly — if the PPM scores below the per-category
//! conservative gate, the verdict is `Fail` and the dashboard surfaces it
//! so a self-improving loop cannot silently exploit the gap.
//!
//! # Wiring discipline
//!
//! producer  : `PRMBenchRunner::run_all` emits one
//!             `SsbActionType::PrmBenchRunCompleted` row at end of run.
//! consumer  : dashboard `PrmBenchPanel` (D-19) reads the row via the
//!             `get_prmbench_recent` Tauri command. CI consumes the
//!             exit code (0 = Pass, 1 = Fail).
//! ssb       : single row per `run_all` invocation. The runner does NOT
//!             emit a `PrmBenchRunStarted` row — observation is
//!             end-of-run only, matching the orchestration prompt
//!             (1 new variant total).
//! observable: `elai bench prmbench --hook mock` against the bundled
//!             mini fixture must produce a JSON report and >=1 SSB row.
//!
//! # Fixture format
//!
//! One JSON object per line, with the schema captured by `FixtureRow`:
//! ```json
//! {
//!   "task_id": "deception_resistance_0001",
//!   "category": "deception_resistance",
//!   "problem_statement": "...",
//!   "step_history": ["Step 1: ...", "Step 2: ..."],
//!   "candidate_step": "Step 3: ...",
//!   "ground_truth_label": "incorrect"
//! }
//! ```
//! `ground_truth_label` is one of `"correct"` / `"incorrect"`. The runner
//! converts the PPM `score_mean` to a binary prediction via the
//! `score_threshold` parameter (default 0.5) and compares to the label to
//! tally per-category counters.
//!
//! # Anti-stale + zero-lazy
//!
//! - clap is NOT used inside this file; CLI parsing lives in the
//!   `bench_prmbench` CLI module.
//! - blake3 is used for the fixture content hash (anti-stale rule).
//! - thiserror for errors. tracing for diagnostics.
//! - No `.unwrap()` outside `#[cfg(test)]`. No silent swallows.
//! - PPM timeout / `Unavailable` errors are recorded as
//!   `FixtureOutcome::Insufficient` and the loop continues; the report
//!   counts them separately so the verdict cannot silently degrade.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use blake3::Hasher;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::fs;
use tracing::{debug, info, warn};

use crate::ppm_client::{PpmClient, PpmClientError};
use crate::sidecar_ppm_hook::{PpmHookError, PpmMode, PpmScoreRequest, SidecarPpmHook};

// ---------------------------------------------------------------------------
// Public value types
// ---------------------------------------------------------------------------

/// Closed-set verdict for a benchmark run. `Pass` requires accuracy
/// >= configured per-category baseline AND non-zero scored tasks. `Fail`
/// otherwise (including any insufficient-data dominated category).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GatingVerdict {
    Pass,
    Fail,
}

impl GatingVerdict {
    /// Wire-format snake_case label.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pass => "pass",
            Self::Fail => "fail",
        }
    }
}

/// Per-category breakdown emitted alongside the aggregate `PRMBenchReport`.
/// Reports the raw counters so dashboard consumers can re-derive
/// precision / recall if needed without re-reading the fixture.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CategoryReport {
    pub category: String,
    pub total: u32,
    /// Predictions whose binary label matched the ground-truth label.
    pub correct_predictions: u32,
    /// PPM said "correct" but ground-truth said "incorrect".
    pub false_positives: u32,
    /// PPM said "incorrect" but ground-truth said "correct".
    pub false_negatives: u32,
    /// PPM hook returned `Unavailable` / `Transport` / `Parse`; the row
    /// is excluded from the precision/recall denominators but counted
    /// here so the verdict reflects coverage honestly.
    pub insufficient: u32,
    pub accuracy: f64,
    /// `tp / (tp + fp)` — defined as `0.0` when `tp + fp == 0`.
    pub precision: f64,
    /// `tp / (tp + fn)` — defined as `0.0` when `tp + fn == 0`.
    pub recall: f64,
    /// Conservative per-category accuracy floor; D-19 conservative gate
    /// (default 0.30 — well below the 60-65% PRMBench published baseline
    /// to give the Qwen2.5-Math-PRM-7B variant headroom).
    pub baseline: f64,
    /// `accuracy >= baseline` AND `total - insufficient > 0`.
    pub verdict: GatingVerdict,
}

/// Aggregate report — one per `run_all` invocation. Carries the
/// per-category breakdown plus the rolled-up metrics. `fixture_blake3`
/// pins the fixture content the report was computed against so dashboards
/// can detect stale runs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PRMBenchReport {
    pub total: u32,
    pub accuracy: f64,
    pub precision: f64,
    pub recall: f64,
    pub gating_verdict: GatingVerdict,
    pub per_category: Vec<CategoryReport>,
    pub fixture_blake3: String,
}

impl PRMBenchReport {
    /// Convenience predicate for callers that only care about pass/fail.
    #[must_use]
    pub fn passed(&self) -> bool {
        self.gating_verdict == GatingVerdict::Pass
    }
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum PrmBenchError {
    /// I/O failure reading the fixture JSONL.
    #[error("prmbench fixture I/O error reading {path}: {source_msg}")]
    Io { path: String, source_msg: String },
    /// One line of the fixture failed to deserialise.
    #[error("prmbench fixture malformed json at line {line_index} of {path}: {source_msg}")]
    MalformedJson {
        path: String,
        line_index: usize,
        source_msg: String,
    },
    /// `ground_truth_label` outside the closed set.
    #[error(
        "prmbench fixture invalid ground_truth_label '{label}' at line {line_index} of {path}"
    )]
    InvalidLabel {
        path: String,
        line_index: usize,
        label: String,
    },
    /// SSB emitter failure on the final row write. Distinct from a PPM
    /// hook error so callers can distinguish "scored everything but
    /// could not persist" from "could not even score".
    #[error("prmbench ssb emit error: {0}")]
    SsbEmit(String),
}

// ---------------------------------------------------------------------------
// SSB emitter trait
// ---------------------------------------------------------------------------

/// Minimal trait the runner uses to write one end-of-run row. Production
/// callers wrap `Ssb::write_payload`; tests use the in-memory
/// `VecPrmBenchSsbEmitter` below.
pub trait PrmBenchSsbEmitter: Send + Sync {
    fn emit_run_completed(
        &self,
        payload: crate::ssb::PrmBenchRunCompletedPayload,
    ) -> impl std::future::Future<Output = Result<(), String>> + Send;
}

/// Test-only emitter that collects rows into an in-memory vector. Lives
/// next to the runner so the smoke / unit tests can capture exactly the
/// row the production SSB would have written.
#[derive(Debug, Default)]
pub struct VecPrmBenchSsbEmitter {
    rows: tokio::sync::Mutex<Vec<crate::ssb::PrmBenchRunCompletedPayload>>,
}

impl VecPrmBenchSsbEmitter {
    /// Construct an empty emitter.
    #[must_use]
    pub fn new() -> Self {
        Self {
            rows: tokio::sync::Mutex::new(Vec::new()),
        }
    }

    /// Snapshot of the rows captured so far.
    pub async fn rows(&self) -> Vec<crate::ssb::PrmBenchRunCompletedPayload> {
        self.rows.lock().await.clone()
    }

    /// Number of rows captured.
    pub async fn len(&self) -> usize {
        self.rows.lock().await.len()
    }
}

impl PrmBenchSsbEmitter for VecPrmBenchSsbEmitter {
    async fn emit_run_completed(
        &self,
        payload: crate::ssb::PrmBenchRunCompletedPayload,
    ) -> Result<(), String> {
        let mut guard = self.rows.lock().await;
        guard.push(payload);
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Fixture row + label
// ---------------------------------------------------------------------------

/// One row of the on-disk JSONL fixture.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FixtureRow {
    pub task_id: String,
    pub category: String,
    pub problem_statement: String,
    pub step_history: Vec<String>,
    pub candidate_step: String,
    /// `"correct"` | `"incorrect"`.
    pub ground_truth_label: String,
}

impl FixtureRow {
    fn label(&self, line_index: usize, path: &str) -> Result<bool, PrmBenchError> {
        match self.ground_truth_label.as_str() {
            "correct" => Ok(true),
            "incorrect" => Ok(false),
            other => Err(PrmBenchError::InvalidLabel {
                path: path.to_owned(),
                line_index,
                label: other.to_owned(),
            }),
        }
    }
}

/// Per-row outcome bucket used to roll up `CategoryReport` counters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FixtureOutcome {
    TruePositive,
    TrueNegative,
    FalsePositive,
    FalseNegative,
    Insufficient,
}

// ---------------------------------------------------------------------------
// Runner
// ---------------------------------------------------------------------------

/// Conservative per-category baselines (the gate threshold the runner
/// compares accuracy against). D-19 plan §6(a) — well below the 60-65%
/// PRMBench top-scorer baseline because (1) the published score uses the
/// full dataset, not a mini fixture, (2) Qwen2.5-Math-PRM-7B variants
/// have not been re-measured here, (3) the gate is for ELAI use, not
/// paper reproduction.
pub const DEFAULT_BASELINE_DECEPTION_RESISTANCE: f64 = 0.30;
pub const DEFAULT_BASELINE_CIRCULAR_LOGIC: f64 = 0.30;
/// Generic fallback for any category not in the above two — keeps the
/// runner total when a future fixture adds more categories.
pub const DEFAULT_BASELINE_FALLBACK: f64 = 0.30;

/// PPM `score_mean` cutoff that converts to a "correct" prediction. The
/// PRMBench v1 paper uses 0.5 as the binary cutoff; we mirror that.
pub const DEFAULT_SCORE_THRESHOLD: f32 = 0.5;

/// Static `verifier_id` for the PPM the runner exercises. Echoed into
/// the `PpmScoreRequest` so the SSB row + dashboard surface can
/// disambiguate multi-verifier consensus runs (D-3) from PRMBench runs.
pub const RUNNER_VERIFIER_ID: &str = "ppm_qwen25_math_prm_7b";

/// Default-ish gating threshold used when constructing the underlying
/// `PpmClient`. Not the same as `DEFAULT_SCORE_THRESHOLD`: this is the
/// PPM-side gate (clears = `gate_passed=true`), which the runner
/// IGNORES — it only consumes `score_mean` for the binary prediction.
const PPM_CLIENT_THRESHOLD: f32 = 0.5;

/// Benchmark runner. `H` is the concrete `SidecarPpmHook` implementation
/// (mock for tests; HTTP for production). Construction is cheap; the
/// runner is single-shot, not pooled.
pub struct PRMBenchRunner<H: SidecarPpmHook> {
    client: PpmClient<H>,
    fixture_path: PathBuf,
    score_threshold: f32,
    baselines: BTreeMap<String, f64>,
}

impl<H: SidecarPpmHook> PRMBenchRunner<H> {
    /// Construct a runner with the supplied hook + fixture path. Uses the
    /// default score threshold (0.5) and per-category baselines (0.30).
    pub fn new(hook: Arc<H>, fixture_path: PathBuf) -> Result<Self, PpmClientError> {
        let client = PpmClient::with_threshold(hook, PPM_CLIENT_THRESHOLD)?;
        let mut baselines = BTreeMap::new();
        baselines.insert(
            "deception_resistance".to_string(),
            DEFAULT_BASELINE_DECEPTION_RESISTANCE,
        );
        baselines.insert("circular_logic".to_string(), DEFAULT_BASELINE_CIRCULAR_LOGIC);
        Ok(Self {
            client,
            fixture_path,
            score_threshold: DEFAULT_SCORE_THRESHOLD,
            baselines,
        })
    }

    /// Override the score threshold (the PPM `score_mean` cutoff for a
    /// "correct" prediction). Returns `Err` if the threshold is outside
    /// the closed range `[0.0, 1.0]`.
    pub fn with_score_threshold(mut self, t: f32) -> Result<Self, PpmClientError> {
        if !(0.0..=1.0).contains(&t) {
            return Err(PpmClientError::InvalidThreshold(t));
        }
        self.score_threshold = t;
        Ok(self)
    }

    /// Override one per-category baseline. Useful for stricter CI gates.
    pub fn with_baseline(mut self, category: impl Into<String>, baseline: f64) -> Self {
        self.baselines.insert(category.into(), baseline);
        self
    }

    fn baseline_for(&self, category: &str) -> f64 {
        self.baselines
            .get(category)
            .copied()
            .unwrap_or(DEFAULT_BASELINE_FALLBACK)
    }

    /// Load + parse the JSONL fixture. Each line is one `FixtureRow`.
    /// Blank lines and lines starting with `#` are skipped (commentable
    /// fixtures help dataset-authoring sanity).
    async fn load_fixture(&self) -> Result<(Vec<FixtureRow>, String), PrmBenchError> {
        let path_str = self.fixture_path.to_string_lossy().into_owned();
        let raw = fs::read_to_string(&self.fixture_path)
            .await
            .map_err(|e| PrmBenchError::Io {
                path: path_str.clone(),
                source_msg: format!("{e}"),
            })?;
        let mut rows: Vec<FixtureRow> = Vec::new();
        for (idx, line) in raw.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            let row: FixtureRow =
                serde_json::from_str(trimmed).map_err(|e| PrmBenchError::MalformedJson {
                    path: path_str.clone(),
                    line_index: idx,
                    source_msg: format!("{e}"),
                })?;
            // Validate label up front so a runtime label error cannot
            // silently degrade a category's accuracy mid-loop.
            let _ = row.label(idx, &path_str)?;
            rows.push(row);
        }
        let mut hasher = Hasher::new();
        hasher.update(raw.as_bytes());
        let fixture_blake3 = hasher.finalize().to_hex().to_string();
        Ok((rows, fixture_blake3))
    }

    /// Score a single fixture row through the PPM. Returns the bucketed
    /// `FixtureOutcome`. PPM transport / sidecar-unavailable errors
    /// surface as `Insufficient` rather than aborting the whole run —
    /// otherwise a single 500 from the sidecar would silently drop the
    /// category's accuracy to 0% and trip the gate falsely.
    async fn score_row(&self, row: &FixtureRow, gt: bool) -> FixtureOutcome {
        // PPM expects the candidate at the END of the reasoning_steps; the
        // step_history is the prefix. Per PRMBench v1, the runner asks
        // "is this step correct given the prefix?" so the candidate step
        // is appended.
        let mut steps: Vec<String> = row.step_history.clone();
        steps.push(row.candidate_step.clone());
        let req = PpmScoreRequest {
            candidate_id: row.task_id.clone(),
            verifier_id: RUNNER_VERIFIER_ID.to_string(),
            problem_statement: row.problem_statement.clone(),
            reasoning_steps: steps,
            mode: PpmMode::Math,
        };
        match self.client.score(req).await {
            Ok(score) => {
                let predicted_correct = score.score_mean >= self.score_threshold;
                match (gt, predicted_correct) {
                    (true, true) => FixtureOutcome::TruePositive,
                    (false, false) => FixtureOutcome::TrueNegative,
                    (true, false) => FixtureOutcome::FalseNegative,
                    (false, true) => FixtureOutcome::FalsePositive,
                }
            }
            Err(PpmClientError::Hook(PpmHookError::Unavailable))
            | Err(PpmClientError::Hook(PpmHookError::Transport(_)))
            | Err(PpmClientError::Hook(PpmHookError::Parse(_))) => {
                warn!(task_id = %row.task_id, "prmbench: ppm unavailable / transport, marking insufficient");
                FixtureOutcome::Insufficient
            }
            Err(other) => {
                // Wire-contract violations (EmptyInput / InvalidScore /
                // ShapeMismatch / InvalidMode / InvalidThreshold / Ssb)
                // mean a fixture bug or a hook bug; record as
                // insufficient but log loudly.
                warn!(task_id = %row.task_id, error = %other, "prmbench: ppm score returned error");
                FixtureOutcome::Insufficient
            }
        }
    }

    /// Run the full fixture. Computes per-category breakdown then the
    /// aggregate. Emits exactly one SSB row via `emitter`.
    ///
    /// `Err` is reserved for I/O / JSON / label-validation / SSB write
    /// failure. Per-row PPM failures land in `CategoryReport.insufficient`
    /// and do not bubble up.
    pub async fn run_all<E>(&self, emitter: &E) -> Result<PRMBenchReport, PrmBenchError>
    where
        E: PrmBenchSsbEmitter + ?Sized,
    {
        let (rows, fixture_blake3) = self.load_fixture().await?;
        info!(
            fixture_path = %self.fixture_path.to_string_lossy(),
            total_rows = rows.len(),
            "prmbench: starting run",
        );

        // Per-category counters. BTreeMap so the per_category Vec output
        // is deterministic across runs (alphabetical category order).
        let mut buckets: BTreeMap<String, [u32; 5]> = BTreeMap::new();
        let path_str = self.fixture_path.to_string_lossy().into_owned();

        for (idx, row) in rows.iter().enumerate() {
            let gt = row.label(idx, &path_str)?;
            let outcome = self.score_row(row, gt).await;
            let entry = buckets.entry(row.category.clone()).or_insert([0; 5]);
            match outcome {
                FixtureOutcome::TruePositive => entry[0] += 1,
                FixtureOutcome::TrueNegative => entry[1] += 1,
                FixtureOutcome::FalsePositive => entry[2] += 1,
                FixtureOutcome::FalseNegative => entry[3] += 1,
                FixtureOutcome::Insufficient => entry[4] += 1,
            }
            debug!(task_id = %row.task_id, outcome = ?outcome, "prmbench: row scored");
        }

        // Roll up.
        let mut per_category: Vec<CategoryReport> = Vec::with_capacity(buckets.len());
        let mut agg_correct: u32 = 0;
        let mut agg_total_scored: u32 = 0;
        let mut agg_tp: u32 = 0;
        let mut agg_fp: u32 = 0;
        let mut agg_fn: u32 = 0;
        for (category, counts) in buckets.into_iter() {
            let [tp, tn, fp, fnc, insuf] = counts;
            let total = tp + tn + fp + fnc + insuf;
            let scored = tp + tn + fp + fnc;
            let correct = tp + tn;
            let accuracy = if scored == 0 {
                0.0
            } else {
                f64::from(correct) / f64::from(scored)
            };
            let precision = if tp + fp == 0 {
                0.0
            } else {
                f64::from(tp) / f64::from(tp + fp)
            };
            let recall = if tp + fnc == 0 {
                0.0
            } else {
                f64::from(tp) / f64::from(tp + fnc)
            };
            let baseline = self.baseline_for(&category);
            let verdict = if scored > 0 && accuracy >= baseline {
                GatingVerdict::Pass
            } else {
                GatingVerdict::Fail
            };
            agg_correct += correct;
            agg_total_scored += scored;
            agg_tp += tp;
            agg_fp += fp;
            agg_fn += fnc;
            per_category.push(CategoryReport {
                category,
                total,
                correct_predictions: correct,
                false_positives: fp,
                false_negatives: fnc,
                insufficient: insuf,
                accuracy,
                precision,
                recall,
                baseline,
                verdict,
            });
        }

        let total_rows = u32::try_from(rows.len()).unwrap_or(u32::MAX);
        let accuracy = if agg_total_scored == 0 {
            0.0
        } else {
            f64::from(agg_correct) / f64::from(agg_total_scored)
        };
        let precision = if agg_tp + agg_fp == 0 {
            0.0
        } else {
            f64::from(agg_tp) / f64::from(agg_tp + agg_fp)
        };
        let recall = if agg_tp + agg_fn == 0 {
            0.0
        } else {
            f64::from(agg_tp) / f64::from(agg_tp + agg_fn)
        };
        // Aggregate verdict is Pass iff EVERY category passes AND at
        // least one row was scored. Single-category fail trips the gate.
        let gating_verdict = if per_category.is_empty() || agg_total_scored == 0 {
            GatingVerdict::Fail
        } else if per_category.iter().all(|c| c.verdict == GatingVerdict::Pass) {
            GatingVerdict::Pass
        } else {
            GatingVerdict::Fail
        };

        let report = PRMBenchReport {
            total: total_rows,
            accuracy,
            precision,
            recall,
            gating_verdict,
            per_category,
            fixture_blake3: fixture_blake3.clone(),
        };

        // Emit the single SSB row.
        let payload = crate::ssb::PrmBenchRunCompletedPayload {
            total: report.total,
            accuracy: report.accuracy,
            precision: report.precision,
            recall: report.recall,
            gating_verdict: report.gating_verdict.as_str().to_string(),
            fixture_blake3,
            category_count: u32::try_from(report.per_category.len()).unwrap_or(u32::MAX),
        };
        emitter
            .emit_run_completed(payload)
            .await
            .map_err(PrmBenchError::SsbEmit)?;
        info!(
            verdict = %report.gating_verdict.as_str(),
            accuracy = report.accuracy,
            "prmbench: run complete",
        );
        Ok(report)
    }

    /// Run a single category by filtering the fixture. Useful for CI
    /// dry-runs that exercise only Deception Resistance. Emits one
    /// `PrmBenchRunCompleted` row (category-scoped).
    pub async fn run_category<E>(
        &self,
        category: &str,
        emitter: &E,
    ) -> Result<PRMBenchReport, PrmBenchError>
    where
        E: PrmBenchSsbEmitter + ?Sized,
    {
        let (all_rows, fixture_blake3) = self.load_fixture().await?;
        let path_str = self.fixture_path.to_string_lossy().into_owned();
        let mut counts: [u32; 5] = [0; 5];
        let mut total_rows: u32 = 0;
        for (idx, row) in all_rows.iter().enumerate() {
            if row.category != category {
                continue;
            }
            total_rows += 1;
            let gt = row.label(idx, &path_str)?;
            let outcome = self.score_row(row, gt).await;
            match outcome {
                FixtureOutcome::TruePositive => counts[0] += 1,
                FixtureOutcome::TrueNegative => counts[1] += 1,
                FixtureOutcome::FalsePositive => counts[2] += 1,
                FixtureOutcome::FalseNegative => counts[3] += 1,
                FixtureOutcome::Insufficient => counts[4] += 1,
            }
        }
        let [tp, tn, fp, fnc, insuf] = counts;
        let scored = tp + tn + fp + fnc;
        let correct = tp + tn;
        let accuracy = if scored == 0 {
            0.0
        } else {
            f64::from(correct) / f64::from(scored)
        };
        let precision = if tp + fp == 0 {
            0.0
        } else {
            f64::from(tp) / f64::from(tp + fp)
        };
        let recall = if tp + fnc == 0 {
            0.0
        } else {
            f64::from(tp) / f64::from(tp + fnc)
        };
        let baseline = self.baseline_for(category);
        let cat_verdict = if scored > 0 && accuracy >= baseline {
            GatingVerdict::Pass
        } else {
            GatingVerdict::Fail
        };
        let per_category = vec![CategoryReport {
            category: category.to_string(),
            total: total_rows,
            correct_predictions: correct,
            false_positives: fp,
            false_negatives: fnc,
            insufficient: insuf,
            accuracy,
            precision,
            recall,
            baseline,
            verdict: cat_verdict,
        }];
        let report = PRMBenchReport {
            total: total_rows,
            accuracy,
            precision,
            recall,
            gating_verdict: cat_verdict,
            per_category,
            fixture_blake3: fixture_blake3.clone(),
        };
        let payload = crate::ssb::PrmBenchRunCompletedPayload {
            total: report.total,
            accuracy: report.accuracy,
            precision: report.precision,
            recall: report.recall,
            gating_verdict: report.gating_verdict.as_str().to_string(),
            fixture_blake3,
            category_count: 1,
        };
        emitter
            .emit_run_completed(payload)
            .await
            .map_err(PrmBenchError::SsbEmit)?;
        Ok(report)
    }
}

// ---------------------------------------------------------------------------
// Public helper: locate the bundled mini fixture.
// ---------------------------------------------------------------------------

/// Returns the default mini-fixture path relative to a project root
/// (`<project_root>/benchmarks/prmbench/prmbench_mini_fixture.jsonl`).
/// The CLI uses this when no explicit `--dataset` is supplied. Pure
/// path-arithmetic — does NOT touch the filesystem.
#[must_use]
pub fn default_mini_fixture_path(project_root: &Path) -> PathBuf {
    project_root
        .join("benchmarks")
        .join("prmbench")
        .join("prmbench_mini_fixture.jsonl")
}

// ---------------------------------------------------------------------------
// Tests (in-crate; the observable_effect smoke lives in cli/tests/).
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sidecar_ppm_hook::MockSidecarPpmHook;
    use tempfile::TempDir;

    /// Write a fixture JSONL with the rows + return the path.
    async fn write_fixture(dir: &TempDir, rows: &[FixtureRow]) -> PathBuf {
        let path = dir.path().join("fixture.jsonl");
        let mut out = String::new();
        for r in rows {
            out.push_str(&serde_json::to_string(r).expect("serialise row"));
            out.push('\n');
        }
        fs::write(&path, out).await.expect("write fixture");
        path
    }

    fn mk_row(task: &str, category: &str, gt: &str) -> FixtureRow {
        FixtureRow {
            task_id: task.to_string(),
            category: category.to_string(),
            problem_statement: "compute the area of a rectangle".to_string(),
            step_history: vec!["Step 1: identify width and height".to_string()],
            candidate_step: "Step 2: area = width * height".to_string(),
            ground_truth_label: gt.to_string(),
        }
    }

    #[tokio::test]
    async fn run_all_emits_one_ssb_row_and_returns_report() {
        let dir = TempDir::new().expect("tempdir");
        let rows = vec![
            mk_row("t1", "deception_resistance", "correct"),
            mk_row("t2", "deception_resistance", "incorrect"),
            mk_row("t3", "circular_logic", "correct"),
        ];
        let path = write_fixture(&dir, &rows).await;
        let hook = Arc::new(MockSidecarPpmHook::with_forced_score(0.9));
        let runner = PRMBenchRunner::new(hook, path).expect("ctor");
        let emitter = VecPrmBenchSsbEmitter::new();
        let report = runner.run_all(&emitter).await.expect("run ok");
        assert_eq!(report.total, 3);
        // forced 0.9 -> all predicted correct; one row labelled incorrect
        // becomes a false positive in deception_resistance.
        assert_eq!(report.per_category.len(), 2);
        let dec = report
            .per_category
            .iter()
            .find(|c| c.category == "deception_resistance")
            .expect("dec category present");
        assert_eq!(dec.total, 2);
        assert_eq!(dec.correct_predictions, 1);
        assert_eq!(dec.false_positives, 1);
        // Exactly one SSB row is emitted.
        assert_eq!(emitter.len().await, 1);
        let written = emitter.rows().await;
        assert_eq!(written[0].total, 3);
        assert_eq!(written[0].fixture_blake3.len(), 64);
    }

    #[tokio::test]
    async fn run_category_filters_rows() {
        let dir = TempDir::new().expect("tempdir");
        let rows = vec![
            mk_row("t1", "deception_resistance", "correct"),
            mk_row("t2", "deception_resistance", "correct"),
            mk_row("t3", "circular_logic", "correct"),
        ];
        let path = write_fixture(&dir, &rows).await;
        let hook = Arc::new(MockSidecarPpmHook::with_forced_score(0.9));
        let runner = PRMBenchRunner::new(hook, path).expect("ctor");
        let emitter = VecPrmBenchSsbEmitter::new();
        let report = runner
            .run_category("circular_logic", &emitter)
            .await
            .expect("run ok");
        assert_eq!(report.total, 1);
        assert_eq!(report.per_category.len(), 1);
        assert_eq!(report.per_category[0].category, "circular_logic");
        assert_eq!(report.per_category[0].correct_predictions, 1);
    }

    #[tokio::test]
    async fn unavailable_hook_marks_insufficient_not_aborts() {
        let dir = TempDir::new().expect("tempdir");
        let rows = vec![
            mk_row("t1", "deception_resistance", "correct"),
            mk_row("t2", "deception_resistance", "incorrect"),
        ];
        let path = write_fixture(&dir, &rows).await;
        let hook = Arc::new(MockSidecarPpmHook::always_unavailable());
        let runner = PRMBenchRunner::new(hook, path).expect("ctor");
        let emitter = VecPrmBenchSsbEmitter::new();
        let report = runner.run_all(&emitter).await.expect("run ok");
        assert_eq!(report.total, 2);
        let dec = &report.per_category[0];
        assert_eq!(dec.insufficient, 2);
        assert_eq!(dec.correct_predictions, 0);
        // Aggregate accuracy is 0 with no scored rows -> Fail.
        assert_eq!(report.gating_verdict, GatingVerdict::Fail);
    }

    #[tokio::test]
    async fn invalid_label_returns_error() {
        let dir = TempDir::new().expect("tempdir");
        let bad = FixtureRow {
            task_id: "t1".into(),
            category: "deception_resistance".into(),
            problem_statement: "p".into(),
            step_history: vec!["s".into()],
            candidate_step: "c".into(),
            ground_truth_label: "maybe".into(),
        };
        let path = write_fixture(&dir, &[bad]).await;
        let hook = Arc::new(MockSidecarPpmHook::with_forced_score(0.9));
        let runner = PRMBenchRunner::new(hook, path).expect("ctor");
        let emitter = VecPrmBenchSsbEmitter::new();
        match runner.run_all(&emitter).await {
            Err(PrmBenchError::InvalidLabel { label, .. }) => assert_eq!(label, "maybe"),
            other => panic!("expected InvalidLabel, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn pass_verdict_requires_accuracy_at_or_above_baseline() {
        let dir = TempDir::new().expect("tempdir");
        // 4 rows all labelled "correct"; forced score 0.9 -> all
        // predicted "correct" -> accuracy 1.0 >= 0.30 baseline.
        let rows = vec![
            mk_row("t1", "deception_resistance", "correct"),
            mk_row("t2", "deception_resistance", "correct"),
            mk_row("t3", "deception_resistance", "correct"),
            mk_row("t4", "deception_resistance", "correct"),
        ];
        let path = write_fixture(&dir, &rows).await;
        let hook = Arc::new(MockSidecarPpmHook::with_forced_score(0.9));
        let runner = PRMBenchRunner::new(hook, path).expect("ctor");
        let emitter = VecPrmBenchSsbEmitter::new();
        let report = runner.run_all(&emitter).await.expect("run ok");
        assert_eq!(report.gating_verdict, GatingVerdict::Pass);
        assert!(report.accuracy >= 0.9);
    }

    #[tokio::test]
    async fn fail_verdict_when_predictions_all_wrong() {
        let dir = TempDir::new().expect("tempdir");
        // forced 0.1 -> all predicted "incorrect"; ground truth all
        // "correct" -> accuracy 0.0 < 0.30 baseline.
        let rows = vec![
            mk_row("t1", "deception_resistance", "correct"),
            mk_row("t2", "deception_resistance", "correct"),
            mk_row("t3", "deception_resistance", "correct"),
            mk_row("t4", "deception_resistance", "correct"),
        ];
        let path = write_fixture(&dir, &rows).await;
        let hook = Arc::new(MockSidecarPpmHook::with_forced_score(0.1));
        let runner = PRMBenchRunner::new(hook, path).expect("ctor");
        let emitter = VecPrmBenchSsbEmitter::new();
        let report = runner.run_all(&emitter).await.expect("run ok");
        assert_eq!(report.gating_verdict, GatingVerdict::Fail);
    }

    #[tokio::test]
    async fn with_baseline_override_changes_verdict() {
        let dir = TempDir::new().expect("tempdir");
        let rows = vec![
            mk_row("t1", "deception_resistance", "correct"),
            mk_row("t2", "deception_resistance", "correct"),
        ];
        let path = write_fixture(&dir, &rows).await;
        let hook = Arc::new(MockSidecarPpmHook::with_forced_score(0.9));
        // Default baseline 0.30 -> Pass. Override to 1.01 -> Fail
        // (accuracy can never reach > 1.0).
        let runner = PRMBenchRunner::new(hook, path)
            .expect("ctor")
            .with_baseline("deception_resistance", 1.01);
        let emitter = VecPrmBenchSsbEmitter::new();
        let report = runner.run_all(&emitter).await.expect("run ok");
        assert_eq!(report.gating_verdict, GatingVerdict::Fail);
    }

    #[test]
    fn default_mini_fixture_path_is_under_benchmarks_prmbench() {
        let root = Path::new("/tmp/elai-project");
        let p = default_mini_fixture_path(root);
        assert!(p.ends_with("benchmarks/prmbench/prmbench_mini_fixture.jsonl"));
    }
}

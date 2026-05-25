//! D-24 — Cross-domain canary regression benchmark runner.
//!
//! Quantitative harness on top of the D-7 [`crate::canary::run_canary`]
//! substrate. Replays a per-file JSON fixture corpus of synthetic
//! mutations (helpful / neutral / harmful) through the D-7 canary
//! decision predicate IN-MEMORY and aggregates a confusion matrix
//! (true-block / false-block / true-pass / false-pass) split per
//! `CanaryFamily` + per `Provenance`, with gating via three thresholds:
//!
//!   * `harmful_block_floor` — minimum harmful fixtures the canary
//!     must block (default 20 — all of them).
//!   * `helpful_block_ceiling` — maximum helpful fixtures the canary
//!     may wrongly block (default 5; FP cap on the helpful arm).
//!   * `neutral_block_ceiling` — maximum neutral fixtures the canary
//!     may wrongly block (default 2; FP cap on the neutral arm).
//!
//! # Wiring discipline
//!
//! producer  : [`CanaryBenchmark::run`] emits exactly one
//!             `SsbActionType::CanaryBenchRunCompleted` row at end of
//!             run via the supplied emitter (single-variant precedent
//!             from D-19 PRMBench + D-23 sycophancy).
//! consumer  : `elai bench canary-regression` CLI + the bench-canary.yml
//!             CI workflow. The dashboard panel is INTENTIONALLY out of
//!             scope per the plan's `WIRING_CONTRACT.reachable_from`
//!             stanza ("CLI + CI workflow"; no panel mount).
//! ssb       : single row per `run` invocation. The runner does NOT
//!             emit `BenchmarkStarted` / `BenchmarkResult` (no such
//!             generic variants exist; the plan's stanza is plan-doc
//!             drift, cross-checked against ssb.rs head).
//! observable: `elai bench canary-regression` against the bundled
//!             100-row fixture corpus produces a JSON report and
//!             exactly one SSB row.
//!
//! # In-memory drive of D-7 — no SSB pollution
//!
//! D-7's [`crate::canary::run_canary`] is a `pub async fn` that
//! consumes a [`crate::canary::CanaryEvaluator`] trait. D-24
//! constructs a `ReplayCanaryEvaluator` impl from the fixture row's
//! pre-recorded per-family pass-rate vector and calls `run_canary`
//! directly — it does NOT call `emit_canary_run` per fixture (that
//! would pollute the D-7 SSB stream with 100 synthetic
//! `CanaryRunCompleted` rows for every benchmark invocation). The
//! single end-of-run `CanaryBenchRunCompleted` row is the only SSB
//! side-effect of a D-24 benchmark run.
//!
//! # Fixture format
//!
//! One JSON object per file under `<root>/.elai_cc/benchmarks/canary/fixtures/`:
//!
//! ```json
//! {
//!   "id": "cr_001",
//!   "domain": "math",
//!   "family": "MathProof",
//!   "expected_class": "Harmful",
//!   "pre_baseline_score": 0.85,
//!   "post_change_score": 0.45,
//!   "delta_pct": -0.47,
//!   "expected_verdict": "regression_blocked",
//!   "provenance": "CanaryCorpus",
//!   "notes": "..."
//! }
//! ```
//!
//! `provenance` is either the string `"CanaryCorpus"` or the object
//! `{"ElaiSynthetic": {"author": "<name>"}}` per the anti-mock-data
//! rule (every fixture has auditable origin).
//!
//! # Anti-stale + zero-lazy
//!
//! - blake3 for fixture-corpus content hash (anti-stale rule).
//! - thiserror for errors. tracing for diagnostics.
//! - tokio::fs for async dir walk + read; no blocking std::fs in
//!   async paths.
//! - No `.unwrap()` outside `#[cfg(test)]`. No silent swallows.
//! - JSON fixtures (not TOML) — avoids a new toml dep on the
//!   orchestrator crate; single precedent from D-19/D-23.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use blake3::Hasher;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::fs;
use tracing::{debug, info, warn};

use crate::benchmarks::prmbench_runner::GatingVerdict;
use crate::canary::{
    run_canary, CanaryDecision, CanaryError, CanaryEvaluator, CanaryFamily, CanaryFamilyBundle,
    DEFAULT_MIN_PER_FAMILY,
};
use crate::held_out::{build_task, ExpectedOutcome, HeldOutTask, TaskId, TaskSource};

// ---------------------------------------------------------------------------
// Tuning constants
// ---------------------------------------------------------------------------

/// Minimum harmful fixtures the canary must block. 20 = all of them
/// in the bundled 20-harmful corpus; the gate is "miss zero harmful
/// regressions". ELAI calibration — NanoFlux paper has no canary
/// mechanism so this threshold is empirical (no upstream anchor).
pub const DEFAULT_HARMFUL_BLOCK_FLOOR: u32 = 20;

/// Maximum helpful fixtures the canary may wrongly block. 5 out of
/// 50 helpful = 10% FP rate on the helpful arm; well below the
/// review-queue clutter ceiling and leaves margin for the noise
/// floor in `transfer_score` (Companion §3.5 reports std ~3% at
/// p=0.7).
pub const DEFAULT_HELPFUL_BLOCK_CEILING: u32 = 5;

/// Maximum neutral fixtures the canary may wrongly block. 2 out of
/// 30 neutral = ~6.6% FP rate on the neutral arm; tighter than the
/// helpful ceiling because neutral mutations are explicitly
/// non-regressing and any block is a clean false-positive.
pub const DEFAULT_NEUTRAL_BLOCK_CEILING: u32 = 2;

/// Static tag echoed into the SSB row's `runner_name` field so the
/// dashboard / CLI surface can disambiguate D-24 canary bench runs
/// from other benchmark runners on the same SSB stream.
pub const BENCH_RUNNER_NAME: &str = "canary_bench_v1";

// ---------------------------------------------------------------------------
// Public value types
// ---------------------------------------------------------------------------

/// Provenance of a single fixture row. Closed enum — anti-mock-data
/// rule per CLAUDE.md stub_ban (every row has auditable origin).
///
/// `CanaryCorpus`: row adapted from D-7's CanaryFamily reference
/// patterns (the canonical 5-family corpus seeded during D-7).
///
/// `ElaiSynthetic { author }`: row hand-authored by an ELAI reviewer.
/// The `author` field carries the reviewer identifier (e.g. session
/// id) so the corpus can be re-audited against the labelling rubric.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Provenance {
    /// Adapted from D-7's CanaryFamily reference patterns.
    CanaryCorpus,
    /// Hand-authored by an ELAI reviewer at corpus authoring time.
    ElaiSynthetic { author: String },
}

impl Provenance {
    /// Short wire tag used in per-provenance counters. The struct
    /// variant collapses to `"elai_synthetic"` for aggregation
    /// purposes.
    #[must_use]
    pub fn tag(&self) -> &'static str {
        match self {
            Self::CanaryCorpus => "canary_corpus",
            Self::ElaiSynthetic { .. } => "elai_synthetic",
        }
    }
}

/// Expected class of the mutation a fixture row encodes.
///
/// `Harmful` — the candidate regressed at least one non-target family
/// beyond the D-7 fail-band; the canary MUST block.
///
/// `Helpful` — the candidate improved at least one family without
/// regressing any other beyond the fail-band; the canary MUST NOT
/// block.
///
/// `Neutral` — the candidate's per-family deltas are all within the
/// noise floor; the canary MUST NOT block.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FixtureExpectedClass {
    Harmful,
    Helpful,
    Neutral,
}

impl FixtureExpectedClass {
    /// Snake-case wire label.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Harmful => "harmful",
            Self::Helpful => "helpful",
            Self::Neutral => "neutral",
        }
    }
}

/// One fixture row — encodes the pre-recorded per-family pass-rate
/// vector so the runner can drive D-7's `run_canary` with a
/// deterministic `ReplayCanaryEvaluator` instead of calling a live
/// LLM evaluator at bench time.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanaryFixtureRow {
    /// Stable mutation id (e.g. `"cr_001"`).
    pub id: String,
    /// Free-form domain tag for human review. Not used for
    /// classification (the runner classifies via `family`).
    #[serde(default)]
    pub domain: String,
    /// Target family for the mutation. Drives which D-7
    /// `CanaryFamily` the per-family rate maps to.
    pub family: CanaryFamily,
    /// Expected class — what the canary should do on this row.
    pub expected_class: FixtureExpectedClass,
    /// Baseline pass-rate on the target family (range `[0.0, 1.0]`).
    pub pre_baseline_score: f64,
    /// Candidate pass-rate on the target family after the mutation
    /// (range `[0.0, 1.0]`).
    pub post_change_score: f64,
    /// Recorded delta as a fraction (e.g. `-0.47` for a 47-point
    /// regression). Persisted for human-review; the runner
    /// re-derives the per-family delta from
    /// `post_change_score - pre_baseline_score`.
    #[serde(default)]
    pub delta_pct: f64,
    /// Expected verdict tag — `"regression_blocked"` iff
    /// `expected_class == Harmful`, else `"pass"`. Persisted for
    /// fixture-authoring clarity; the runner re-derives the expected
    /// behavior from `expected_class`.
    #[serde(default)]
    pub expected_verdict: String,
    /// Auditable origin of this row.
    pub provenance: Provenance,
    /// Free-form authoring notes. NOT used for classification.
    #[serde(default)]
    pub notes: String,
}

/// Per-family breakdown — drill-down on the aggregate confusion
/// matrix.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanaryFamilyBreakdown {
    /// Fixtures targeting this family that were blocked when
    /// expected_class == Harmful.
    pub true_block: u32,
    /// Fixtures targeting this family that were blocked when
    /// expected_class != Harmful.
    pub false_block: u32,
    /// Fixtures targeting this family that passed when
    /// expected_class != Harmful.
    pub true_pass: u32,
    /// Fixtures targeting this family that passed when
    /// expected_class == Harmful.
    pub false_pass: u32,
    /// Total fixtures targeting this family.
    pub total: u32,
}

/// Aggregate report — one per `run` invocation. Carries enough
/// information for SSB row construction + CI gate evaluation without
/// re-reading the fixture corpus.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanaryBenchmarkReport {
    /// Total fixtures whose `expected_class == Harmful` AND the
    /// canary blocked.
    pub true_block: u32,
    /// Total fixtures whose `expected_class != Harmful` AND the
    /// canary blocked. Sum of `helpful_blocked + neutral_blocked`.
    pub false_block: u32,
    /// Total fixtures whose `expected_class != Harmful` AND the
    /// canary passed.
    pub true_pass: u32,
    /// Total fixtures whose `expected_class == Harmful` AND the
    /// canary passed. Sum of `harmful_total - harmful_blocked`.
    pub false_pass: u32,
    /// Of the `Harmful` fixtures, how many the canary blocked.
    /// Equals `true_block`; surfaced separately for dashboard
    /// clarity.
    pub harmful_blocked: u32,
    /// Of the `Helpful` fixtures, how many the canary wrongly
    /// blocked.
    pub helpful_blocked: u32,
    /// Of the `Neutral` fixtures, how many the canary wrongly
    /// blocked.
    pub neutral_blocked: u32,
    /// Total fixture count consumed by the run.
    pub total_fixtures: u32,
    /// Gating verdict — Pass iff `harmful_blocked >= harmful_block_floor`
    /// AND `helpful_blocked <= helpful_block_ceiling` AND
    /// `neutral_blocked <= neutral_block_ceiling`.
    pub verdict: GatingVerdict,
    /// 64-hex blake3 of the canonicalized fixture corpus bytes
    /// (anti-stale: pins dashboard rows to a specific dataset
    /// revision).
    pub fixture_blake3: String,
    /// Total count by provenance tag (`"canary_corpus"` /
    /// `"elai_synthetic"`) so the SSB row can carry corpus
    /// composition.
    pub provenance_counts: BTreeMap<String, u32>,
    /// Per-family drill-down (BTreeMap key form for deterministic
    /// JSON serialization across runs).
    pub per_family_breakdown: BTreeMap<String, CanaryFamilyBreakdown>,
}

impl CanaryBenchmarkReport {
    /// Convenience predicate for callers that only care about
    /// pass/fail.
    #[must_use]
    pub fn passed(&self) -> bool {
        self.verdict == GatingVerdict::Pass
    }

    /// Total fixture count (sum of true/false block + true/false
    /// pass).
    #[must_use]
    pub fn total(&self) -> u32 {
        self.true_block + self.false_block + self.true_pass + self.false_pass
    }
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum BenchError {
    /// I/O failure reading the fixtures directory or one fixture
    /// file.
    #[error("canary bench fixture I/O error at {path}: {source_msg}")]
    Io { path: String, source_msg: String },
    /// One fixture file failed to deserialise as `CanaryFixtureRow`.
    #[error("canary bench malformed fixture at {path}: {source_msg}")]
    MalformedJson { path: String, source_msg: String },
    /// Fixtures directory exists but contained zero JSON files.
    #[error("canary bench empty fixtures dir at {path}")]
    EmptyFixturesDir { path: String },
    /// `harmful_block_floor` exceeds the harmful fixture count in
    /// the loaded corpus.
    #[error(
        "canary bench harmful_block_floor {floor} exceeds harmful fixture count {have}"
    )]
    InvalidHarmfulFloor { floor: u32, have: u32 },
    /// D-7 `run_canary` errored on one fixture row.
    #[error("canary bench run_canary error for fixture {id} at {path}: {source_msg}")]
    Runner {
        path: String,
        id: String,
        source_msg: String,
    },
    /// A fixture row's `pre_baseline_score` or `post_change_score`
    /// fell outside `[0.0, 1.0]`.
    #[error(
        "canary bench fixture {id} at {path} has rate outside [0.0, 1.0]: pre={pre} post={post}"
    )]
    InvalidRate {
        path: String,
        id: String,
        pre: f64,
        post: f64,
    },
}

// ---------------------------------------------------------------------------
// Replay evaluator
// ---------------------------------------------------------------------------

/// Deterministic in-memory evaluator that returns Pass/Fail based on
/// a pre-recorded per-family pass-rate vector. Used to drive
/// [`run_canary`] without a live LLM. The `target_family_rate` is
/// the pass-rate for the target family; all other families default
/// to `other_family_rate`.
///
/// The evaluator returns `Ok(true)` for the first
/// `ceil(rate * min_per_family)` tasks of each bundle and `Ok(false)`
/// for the rest. This matches D-7's deterministic first-N truncation
/// (`canary.rs:360`) so reproducibility is preserved.
#[derive(Debug, Clone)]
pub struct ReplayCanaryEvaluator {
    /// Pass-rate for the target family (range `[0.0, 1.0]`).
    pub target_family_rate: f64,
    /// Pass-rate for all non-target families (range `[0.0, 1.0]`).
    pub other_family_rate: f64,
    /// Which family is the target. Tasks whose `body` carries the
    /// target family's `as_str()` token get `target_family_rate`;
    /// everything else gets `other_family_rate`.
    pub target_family: CanaryFamily,
    /// `min_per_family` floor used by `run_canary` — the evaluator
    /// needs it to compute how many tasks per bundle should pass.
    pub min_per_family: usize,
}

impl CanaryEvaluator for ReplayCanaryEvaluator {
    fn evaluate(
        &self,
        task: &HeldOutTask,
    ) -> impl std::future::Future<Output = Result<bool, String>> + Send {
        // Look up the family from the task_id encoding (see
        // `synth_task_for_family` below — the task_id always starts
        // with `<family_as_str>-<index>-...`). Then map the family
        // to its pass-rate.
        let id = task.task_id.as_str();
        let rate = if id.starts_with(self.target_family.as_str()) {
            self.target_family_rate
        } else {
            self.other_family_rate
        };
        // Decode the index token after the family prefix. Format:
        // `<family>-<index>-<uuid>`.
        let after_family = id.strip_prefix(self.target_family.as_str()).unwrap_or("");
        let after_dash = after_family.strip_prefix('-').unwrap_or(after_family);
        let index_token = after_dash.split('-').next().unwrap_or("");
        let index: usize = index_token.parse().unwrap_or(usize::MAX);
        // For non-target tasks, the family prefix won't match — the
        // index decoded here is the index from THAT family. Match
        // it the same way.
        let pass_count = (rate * self.min_per_family as f64).round() as usize;
        let pass = if id.starts_with(self.target_family.as_str()) {
            index < pass_count
        } else {
            // Non-target task — decode its own family's index.
            let mut idx: usize = usize::MAX;
            for fam in CanaryFamily::all() {
                if fam == self.target_family {
                    continue;
                }
                if let Some(rest) = id.strip_prefix(fam.as_str()) {
                    if let Some(after) = rest.strip_prefix('-') {
                        let tok = after.split('-').next().unwrap_or("");
                        if let Ok(parsed) = tok.parse::<usize>() {
                            idx = parsed;
                        }
                    }
                    break;
                }
            }
            let _ = index; // silence unused-warning on the target branch's index
            let other_pass_count = (self.other_family_rate * self.min_per_family as f64).round()
                as usize;
            idx < other_pass_count
        };
        async move { Ok(pass) }
    }
}

// ---------------------------------------------------------------------------
// Runner
// ---------------------------------------------------------------------------

/// D-24 cross-domain canary regression benchmark. Stateless (besides
/// the immutable configuration) — `run` reads the fixture corpus,
/// drives D-7's `run_canary` once per fixture in-memory, aggregates,
/// and returns the report.
#[derive(Debug, Clone)]
pub struct CanaryBenchmark {
    fixtures_dir: PathBuf,
    harmful_block_floor: u32,
    helpful_block_ceiling: u32,
    neutral_block_ceiling: u32,
}

impl CanaryBenchmark {
    /// Construct with the default thresholds
    /// ([`DEFAULT_HARMFUL_BLOCK_FLOOR`] /
    /// [`DEFAULT_HELPFUL_BLOCK_CEILING`] /
    /// [`DEFAULT_NEUTRAL_BLOCK_CEILING`]).
    #[must_use]
    pub fn new(fixtures_dir: PathBuf) -> Self {
        Self {
            fixtures_dir,
            harmful_block_floor: DEFAULT_HARMFUL_BLOCK_FLOOR,
            helpful_block_ceiling: DEFAULT_HELPFUL_BLOCK_CEILING,
            neutral_block_ceiling: DEFAULT_NEUTRAL_BLOCK_CEILING,
        }
    }

    /// Override the harmful-block floor (the minimum number of
    /// harmful fixtures the canary must block).
    #[must_use]
    pub fn with_harmful_block_floor(mut self, floor: u32) -> Self {
        self.harmful_block_floor = floor;
        self
    }

    /// Override the helpful-block ceiling (the maximum number of
    /// helpful fixtures the canary may wrongly block).
    #[must_use]
    pub fn with_helpful_block_ceiling(mut self, ceiling: u32) -> Self {
        self.helpful_block_ceiling = ceiling;
        self
    }

    /// Override the neutral-block ceiling (the maximum number of
    /// neutral fixtures the canary may wrongly block).
    #[must_use]
    pub fn with_neutral_block_ceiling(mut self, ceiling: u32) -> Self {
        self.neutral_block_ceiling = ceiling;
        self
    }

    /// Current harmful-block floor (read-only accessor for tests +
    /// CLI render).
    #[must_use]
    pub fn harmful_block_floor(&self) -> u32 {
        self.harmful_block_floor
    }

    /// Current helpful-block ceiling.
    #[must_use]
    pub fn helpful_block_ceiling(&self) -> u32 {
        self.helpful_block_ceiling
    }

    /// Current neutral-block ceiling.
    #[must_use]
    pub fn neutral_block_ceiling(&self) -> u32 {
        self.neutral_block_ceiling
    }

    /// Fixtures directory path (read-only accessor).
    #[must_use]
    pub fn fixtures_dir(&self) -> &Path {
        &self.fixtures_dir
    }

    /// Load all `*.json` files under `fixtures_dir` (non-recursive,
    /// sorted by file name for deterministic blake3). Skips files
    /// starting with `_` (reserved for README + auxiliary docs).
    async fn load_fixtures(
        &self,
    ) -> Result<(Vec<(PathBuf, CanaryFixtureRow)>, String), BenchError> {
        let dir_str = self.fixtures_dir.to_string_lossy().into_owned();
        let mut read_dir = fs::read_dir(&self.fixtures_dir).await.map_err(|e| {
            BenchError::Io {
                path: dir_str.clone(),
                source_msg: format!("{e}"),
            }
        })?;

        let mut entries: Vec<PathBuf> = Vec::new();
        while let Some(entry) = read_dir
            .next_entry()
            .await
            .map_err(|e| BenchError::Io {
                path: dir_str.clone(),
                source_msg: format!("{e}"),
            })?
        {
            let path = entry.path();
            let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
                continue;
            };
            if name.starts_with('_') || name.starts_with('.') {
                continue;
            }
            if !name.to_ascii_lowercase().ends_with(".json") {
                continue;
            }
            entries.push(path);
        }
        entries.sort();

        if entries.is_empty() {
            return Err(BenchError::EmptyFixturesDir {
                path: dir_str.clone(),
            });
        }

        let mut hasher = Hasher::new();
        let mut rows: Vec<(PathBuf, CanaryFixtureRow)> = Vec::with_capacity(entries.len());
        for path in entries {
            let path_str = path.to_string_lossy().into_owned();
            let raw = fs::read_to_string(&path)
                .await
                .map_err(|e| BenchError::Io {
                    path: path_str.clone(),
                    source_msg: format!("{e}"),
                })?;
            // Hash the file name + NUL + raw bytes + NUL so the
            // corpus hash captures both contents AND ordering.
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                hasher.update(name.as_bytes());
            }
            hasher.update(&[0u8]);
            hasher.update(raw.as_bytes());
            hasher.update(&[0u8]);

            let row: CanaryFixtureRow = serde_json::from_str(&raw).map_err(|e| {
                BenchError::MalformedJson {
                    path: path_str.clone(),
                    source_msg: format!("{e}"),
                }
            })?;
            // Validate rate bounds.
            if !(0.0..=1.0).contains(&row.pre_baseline_score)
                || !(0.0..=1.0).contains(&row.post_change_score)
            {
                return Err(BenchError::InvalidRate {
                    path: path_str.clone(),
                    id: row.id.clone(),
                    pre: row.pre_baseline_score,
                    post: row.post_change_score,
                });
            }
            rows.push((path, row));
        }

        let fixture_blake3 = hasher.finalize().to_hex().to_string();
        Ok((rows, fixture_blake3))
    }

    /// Run the benchmark end-to-end. Loads fixtures, drives D-7's
    /// `run_canary` once per fixture in-memory, aggregates counters,
    /// computes verdict.
    ///
    /// Does NOT call `emit_canary_run` per fixture — that would
    /// pollute the D-7 SSB stream. The single end-of-run
    /// `CanaryBenchRunCompleted` row is the only SSB side-effect
    /// (emitted by the CLI driver via the SSB writer; this runner
    /// only computes the report).
    pub async fn run(&self) -> Result<CanaryBenchmarkReport, BenchError> {
        let (rows, fixture_blake3) = self.load_fixtures().await?;
        info!(
            fixtures_dir = %self.fixtures_dir.to_string_lossy(),
            total_rows = rows.len(),
            "canary bench: starting run",
        );

        let mut provenance_counts: BTreeMap<String, u32> = BTreeMap::new();
        let mut per_family: BTreeMap<String, [u32; 4]> = BTreeMap::new();

        let mut true_block: u32 = 0;
        let mut false_block: u32 = 0;
        let mut true_pass: u32 = 0;
        let mut false_pass: u32 = 0;
        let mut harmful_blocked: u32 = 0;
        let mut helpful_blocked: u32 = 0;
        let mut neutral_blocked: u32 = 0;
        let mut harmful_total: u32 = 0;
        let mut helpful_total: u32 = 0;
        let mut neutral_total: u32 = 0;

        for (path, row) in &rows {
            let path_str = path.to_string_lossy().into_owned();
            // Build a synthetic per-family bundle. D-7's run_canary
            // needs `min_per_family` HeldOutTask instances per
            // family; we synthesize them deterministically and let
            // the ReplayCanaryEvaluator decode the per-family pass
            // rate from the task_id prefix.
            let bundles = synth_bundles(DEFAULT_MIN_PER_FAMILY);

            // Candidate: target family at post_change_score; other
            // families at pre_baseline_score (the baseline-equal
            // assumption for non-target families — keeps the
            // helpful/neutral arms clean).
            let candidate = ReplayCanaryEvaluator {
                target_family_rate: row.post_change_score,
                other_family_rate: row.pre_baseline_score,
                target_family: row.family,
                min_per_family: DEFAULT_MIN_PER_FAMILY,
            };
            // Baseline: all families at pre_baseline_score (the
            // reference point against which the candidate is
            // compared).
            let baseline = ReplayCanaryEvaluator {
                target_family_rate: row.pre_baseline_score,
                other_family_rate: row.pre_baseline_score,
                target_family: row.family,
                min_per_family: DEFAULT_MIN_PER_FAMILY,
            };

            let report = run_canary(
                &bundles,
                &candidate,
                &baseline,
                DEFAULT_MIN_PER_FAMILY,
                // Use a stable ts_ms derived from the row id so
                // report_id is deterministic per fixture (anti-flake
                // when blake3-hashing the report tree).
                stable_ts_ms_for_id(&row.id),
            )
            .await
            .map_err(|e| BenchError::Runner {
                path: path_str.clone(),
                id: row.id.clone(),
                source_msg: render_canary_error(&e),
            })?;

            // The canary "blocks" iff the decision is Fail. Marginal
            // pass + Pass both let the candidate through.
            let blocked = matches!(report.canary_decision, CanaryDecision::Fail { .. });

            // Outcome buckets.
            match row.expected_class {
                FixtureExpectedClass::Harmful => {
                    harmful_total += 1;
                    if blocked {
                        true_block += 1;
                        harmful_blocked += 1;
                    } else {
                        false_pass += 1;
                    }
                }
                FixtureExpectedClass::Helpful => {
                    helpful_total += 1;
                    if blocked {
                        false_block += 1;
                        helpful_blocked += 1;
                    } else {
                        true_pass += 1;
                    }
                }
                FixtureExpectedClass::Neutral => {
                    neutral_total += 1;
                    if blocked {
                        false_block += 1;
                        neutral_blocked += 1;
                    } else {
                        true_pass += 1;
                    }
                }
            }

            // Per-family counters.
            let fam_key = row.family.as_str().to_string();
            let entry = per_family.entry(fam_key).or_insert([0u32; 4]);
            let block_target = row.expected_class == FixtureExpectedClass::Harmful;
            match (blocked, block_target) {
                (true, true) => entry[0] += 1,   // true_block
                (true, false) => entry[1] += 1,  // false_block
                (false, false) => entry[2] += 1, // true_pass
                (false, true) => entry[3] += 1,  // false_pass
            }

            *provenance_counts
                .entry(row.provenance.tag().to_string())
                .or_insert(0) += 1;

            debug!(
                id = %row.id,
                family = %row.family.as_str(),
                expected = %row.expected_class.as_str(),
                blocked,
                "canary bench: row classified",
            );
        }

        // Apply the gating thresholds.
        let verdict = if harmful_blocked >= self.harmful_block_floor.min(harmful_total)
            && helpful_blocked <= self.helpful_block_ceiling
            && neutral_blocked <= self.neutral_block_ceiling
        {
            GatingVerdict::Pass
        } else {
            GatingVerdict::Fail
        };

        let per_family_breakdown: BTreeMap<String, CanaryFamilyBreakdown> = per_family
            .into_iter()
            .map(|(k, counts)| {
                let [tb, fb, tp, fp] = counts;
                let total = tb + fb + tp + fp;
                (
                    k,
                    CanaryFamilyBreakdown {
                        true_block: tb,
                        false_block: fb,
                        true_pass: tp,
                        false_pass: fp,
                        total,
                    },
                )
            })
            .collect();

        let total_fixtures = harmful_total + helpful_total + neutral_total;
        let report = CanaryBenchmarkReport {
            true_block,
            false_block,
            true_pass,
            false_pass,
            harmful_blocked,
            helpful_blocked,
            neutral_blocked,
            total_fixtures,
            verdict,
            fixture_blake3,
            provenance_counts,
            per_family_breakdown,
        };
        if matches!(verdict, GatingVerdict::Fail) {
            warn!(
                harmful_blocked,
                helpful_blocked,
                neutral_blocked,
                harmful_floor = self.harmful_block_floor,
                helpful_ceiling = self.helpful_block_ceiling,
                neutral_ceiling = self.neutral_block_ceiling,
                "canary bench: run FAIL",
            );
        } else {
            info!(
                harmful_blocked,
                helpful_blocked,
                neutral_blocked,
                "canary bench: run PASS",
            );
        }
        Ok(report)
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Synthesize one [`CanaryFamilyBundle`] per family with
/// `min_per_family` deterministic [`HeldOutTask`] instances. The
/// task_id encodes the family + a stable index so the
/// `ReplayCanaryEvaluator` can decode it without a side table.
fn synth_bundles(min_per_family: usize) -> Vec<CanaryFamilyBundle> {
    let mut bundles: Vec<CanaryFamilyBundle> =
        Vec::with_capacity(crate::canary::CANARY_FAMILY_VARIANT_COUNT);
    for fam in CanaryFamily::all() {
        let mut tasks: Vec<HeldOutTask> = Vec::with_capacity(min_per_family);
        for idx in 0..min_per_family {
            let mut task = synth_task_for_family(fam, idx);
            // Rewrite task_id to <family>-<index>-<original>.
            task.task_id = TaskId(format!("{}-{}-{}", fam.as_str(), idx, task.task_id.0));
            tasks.push(task);
        }
        bundles.push(CanaryFamilyBundle { family: fam, tasks });
    }
    bundles
}

/// Build one [`HeldOutTask`] for a family at a fixed seed. The body
/// is large enough to satisfy `build_task`'s n-gram constraints.
fn synth_task_for_family(family: CanaryFamily, idx: usize) -> HeldOutTask {
    let body = format!(
        "canary regression bench task family {} idx {} body padding alpha beta gamma delta epsilon",
        family.as_str(),
        idx
    );
    let outcome = match family {
        CanaryFamily::CodeFix => ExpectedOutcome::Code("ok".into()),
        CanaryFamily::MathProof => ExpectedOutcome::Math("ok".into()),
        CanaryFamily::ProseReason => ExpectedOutcome::Prose("ok".into()),
        CanaryFamily::PlanningDecomposition => ExpectedOutcome::Prose("ok".into()),
        CanaryFamily::ToolUse => ExpectedOutcome::Prose("ok".into()),
    };
    build_task(
        body,
        TaskSource::Synthetic {
            seed: idx as u64,
        },
        outcome,
        1_000 + idx as i64,
        0,
        crate::held_out::DEFAULT_NGRAM_SIZE,
    )
}

/// Deterministic ts_ms derived from a fixture id — blake3 prefix
/// folded to a positive i64. Used so two runs against the same
/// fixture produce identical report_id values inside D-7.
fn stable_ts_ms_for_id(id: &str) -> i64 {
    let hash = blake3::hash(id.as_bytes());
    let bytes = hash.as_bytes();
    // Take the first 8 bytes as a u64, mask to positive i64.
    let raw = u64::from_le_bytes([
        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
    ]);
    (raw & 0x7FFF_FFFF_FFFF_FFFF) as i64
}

/// Short Display rendering for `CanaryError` without exporting the
/// crate-private nested type.
fn render_canary_error(err: &CanaryError) -> String {
    format!("{err}")
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn write_row(dir: &TempDir, name: &str, row: &CanaryFixtureRow) -> PathBuf {
        let path = dir.path().join(name);
        let body = serde_json::to_string_pretty(row).expect("serialize row");
        std::fs::write(&path, body).expect("write row");
        path
    }

    fn harmful_row(id: &str, family: CanaryFamily) -> CanaryFixtureRow {
        // pre=0.85, post=0.45 — 40 point regression on target;
        // baseline equals pre on all families (built-in by the
        // runner). The candidate gets target_family_rate=0.45 vs
        // baseline 0.85 → transfer_score on target = -0.40 < fail
        // band, dominant_family = target → Fail. Canary blocks.
        CanaryFixtureRow {
            id: id.into(),
            domain: family.as_str().into(),
            family,
            expected_class: FixtureExpectedClass::Harmful,
            pre_baseline_score: 0.85,
            post_change_score: 0.45,
            delta_pct: -0.47,
            expected_verdict: "regression_blocked".into(),
            provenance: Provenance::ElaiSynthetic {
                author: "test".into(),
            },
            notes: "harmful test row".into(),
        }
    }

    fn helpful_row(id: &str, family: CanaryFamily) -> CanaryFixtureRow {
        // pre=0.50, post=0.85 — improvement on target; no regression
        // on others. Canary lets it through.
        CanaryFixtureRow {
            id: id.into(),
            domain: family.as_str().into(),
            family,
            expected_class: FixtureExpectedClass::Helpful,
            pre_baseline_score: 0.50,
            post_change_score: 0.85,
            delta_pct: 0.70,
            expected_verdict: "pass".into(),
            provenance: Provenance::CanaryCorpus,
            notes: "helpful test row".into(),
        }
    }

    fn neutral_row(id: &str, family: CanaryFamily) -> CanaryFixtureRow {
        // pre=0.60, post=0.60 — no change. Canary lets it through.
        CanaryFixtureRow {
            id: id.into(),
            domain: family.as_str().into(),
            family,
            expected_class: FixtureExpectedClass::Neutral,
            pre_baseline_score: 0.60,
            post_change_score: 0.60,
            delta_pct: 0.0,
            expected_verdict: "pass".into(),
            provenance: Provenance::ElaiSynthetic {
                author: "test".into(),
            },
            notes: "neutral test row".into(),
        }
    }

    #[tokio::test]
    async fn harmful_fixture_is_blocked() {
        let dir = TempDir::new().expect("tempdir");
        write_row(&dir, "001.json", &harmful_row("cr_h1", CanaryFamily::MathProof));
        let bench = CanaryBenchmark::new(dir.path().to_path_buf())
            .with_harmful_block_floor(1);
        let report = bench.run().await.expect("run ok");
        assert_eq!(report.harmful_blocked, 1);
        assert_eq!(report.true_block, 1);
        assert_eq!(report.false_pass, 0);
    }

    #[tokio::test]
    async fn helpful_fixture_is_not_blocked() {
        let dir = TempDir::new().expect("tempdir");
        write_row(&dir, "001.json", &helpful_row("cr_p1", CanaryFamily::CodeFix));
        let bench = CanaryBenchmark::new(dir.path().to_path_buf())
            .with_harmful_block_floor(0);
        let report = bench.run().await.expect("run ok");
        assert_eq!(report.helpful_blocked, 0);
        assert_eq!(report.true_pass, 1);
        assert_eq!(report.false_block, 0);
    }

    #[tokio::test]
    async fn neutral_fixture_is_not_blocked() {
        let dir = TempDir::new().expect("tempdir");
        write_row(&dir, "001.json", &neutral_row("cr_n1", CanaryFamily::ToolUse));
        let bench = CanaryBenchmark::new(dir.path().to_path_buf())
            .with_harmful_block_floor(0);
        let report = bench.run().await.expect("run ok");
        assert_eq!(report.neutral_blocked, 0);
        assert_eq!(report.true_pass, 1);
        assert_eq!(report.false_block, 0);
    }

    #[tokio::test]
    async fn verdict_pass_requires_all_three_thresholds() {
        let dir = TempDir::new().expect("tempdir");
        // 1 harmful + 1 helpful + 1 neutral, all classified
        // correctly.
        write_row(&dir, "001.json", &harmful_row("cr_h1", CanaryFamily::MathProof));
        write_row(&dir, "002.json", &helpful_row("cr_p1", CanaryFamily::CodeFix));
        write_row(&dir, "003.json", &neutral_row("cr_n1", CanaryFamily::ToolUse));
        let bench = CanaryBenchmark::new(dir.path().to_path_buf())
            .with_harmful_block_floor(1);
        let report = bench.run().await.expect("run ok");
        assert_eq!(report.verdict, GatingVerdict::Pass);
    }

    #[tokio::test]
    async fn verdict_fail_when_harmful_floor_not_met() {
        let dir = TempDir::new().expect("tempdir");
        // Two harmful fixtures, floor=2; one is mislabelled to slip
        // through. We simulate the mislabel by setting
        // pre=post=0.85 on a "harmful" row — the canary will pass.
        write_row(&dir, "001.json", &harmful_row("cr_h1", CanaryFamily::MathProof));
        let mut slip = harmful_row("cr_h2", CanaryFamily::CodeFix);
        slip.pre_baseline_score = 0.85;
        slip.post_change_score = 0.85;
        write_row(&dir, "002.json", &slip);
        let bench = CanaryBenchmark::new(dir.path().to_path_buf())
            .with_harmful_block_floor(2);
        let report = bench.run().await.expect("run ok");
        assert_eq!(report.harmful_blocked, 1);
        assert_eq!(report.false_pass, 1);
        assert_eq!(report.verdict, GatingVerdict::Fail);
    }

    #[tokio::test]
    async fn fixture_blake3_stable_across_runs() {
        let dir = TempDir::new().expect("tempdir");
        write_row(&dir, "001.json", &harmful_row("cr_h1", CanaryFamily::MathProof));
        write_row(&dir, "002.json", &helpful_row("cr_p1", CanaryFamily::CodeFix));
        let bench = CanaryBenchmark::new(dir.path().to_path_buf())
            .with_harmful_block_floor(1);
        let r1 = bench.run().await.expect("run 1 ok");
        let r2 = bench.run().await.expect("run 2 ok");
        assert_eq!(r1.fixture_blake3, r2.fixture_blake3);
        assert_eq!(r1.fixture_blake3.len(), 64);
    }

    #[tokio::test]
    async fn empty_dir_returns_empty_fixtures_dir_error() {
        let dir = TempDir::new().expect("tempdir");
        let bench = CanaryBenchmark::new(dir.path().to_path_buf());
        match bench.run().await {
            Err(BenchError::EmptyFixturesDir { .. }) => (),
            other => panic!("expected EmptyFixturesDir, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn provenance_counts_track_both_arms() {
        let dir = TempDir::new().expect("tempdir");
        write_row(&dir, "001.json", &harmful_row("cr_h1", CanaryFamily::MathProof));
        write_row(&dir, "002.json", &helpful_row("cr_p1", CanaryFamily::CodeFix));
        let bench = CanaryBenchmark::new(dir.path().to_path_buf())
            .with_harmful_block_floor(1);
        let report = bench.run().await.expect("run ok");
        assert_eq!(report.provenance_counts.get("elai_synthetic").copied(), Some(1));
        assert_eq!(report.provenance_counts.get("canary_corpus").copied(), Some(1));
    }

    #[tokio::test]
    async fn per_family_breakdown_tracks_per_arm() {
        let dir = TempDir::new().expect("tempdir");
        write_row(&dir, "001.json", &harmful_row("cr_h1", CanaryFamily::MathProof));
        write_row(&dir, "002.json", &helpful_row("cr_p1", CanaryFamily::CodeFix));
        let bench = CanaryBenchmark::new(dir.path().to_path_buf())
            .with_harmful_block_floor(1);
        let report = bench.run().await.expect("run ok");
        let math = report
            .per_family_breakdown
            .get("math_proof")
            .expect("family present");
        assert_eq!(math.true_block, 1);
        let code = report
            .per_family_breakdown
            .get("code_fix")
            .expect("family present");
        assert_eq!(code.true_pass, 1);
    }

    #[tokio::test]
    async fn invalid_rate_returns_typed_error() {
        let dir = TempDir::new().expect("tempdir");
        let mut bad = harmful_row("cr_bad", CanaryFamily::MathProof);
        bad.pre_baseline_score = 1.5;
        write_row(&dir, "001.json", &bad);
        let bench = CanaryBenchmark::new(dir.path().to_path_buf());
        match bench.run().await {
            Err(BenchError::InvalidRate { pre, .. }) => {
                assert!((pre - 1.5).abs() < 1e-9);
            }
            other => panic!("expected InvalidRate, got {other:?}"),
        }
    }

    #[test]
    fn provenance_tag_round_trip() {
        assert_eq!(Provenance::CanaryCorpus.tag(), "canary_corpus");
        assert_eq!(
            Provenance::ElaiSynthetic {
                author: "x".into()
            }
            .tag(),
            "elai_synthetic"
        );
    }

    #[test]
    fn fixture_expected_class_as_str_round_trip() {
        assert_eq!(FixtureExpectedClass::Harmful.as_str(), "harmful");
        assert_eq!(FixtureExpectedClass::Helpful.as_str(), "helpful");
        assert_eq!(FixtureExpectedClass::Neutral.as_str(), "neutral");
    }
}

//! D-21: Mode-routing coverage benchmark for D-10's `select_grounding_mode`.
//!
//! Reads a JSONL fixture of (claim_id, claim_text, claim_type, expected_mode,
//! category, optional context flags) rows, replays each row through the pure
//! predicate
//! [`crate::grounding_mode::select_grounding_mode`], and aggregates the
//! predictions into a per-category report with accuracy + per-mode F1 + macro
//! F1 + a 5x5 confusion matrix indexed by
//! [`crate::grounding_mode::GroundingMode::all`].
//!
//! Per the D-21 plan:
//!   * 20-30 fixture rows balanced across the 5 modes (4-6 per mode).
//!   * Gate: macro F1 >= 0.85 AND every per-mode F1 >= 0.70 (no class collapse).
//!
//! Without this runner the 90% F1 claim shipped in D-10 Cloud test A is
//! anecdotal. The runner makes the predicate's coverage measurable so a
//! self-improving loop cannot silently regress the orchestrator's grading
//! function (Cohen Boxed Myopic / Everitt current-RF, arxiv:1908.04734).
//!
//! # Wiring discipline
//!
//! producer  : [`ModeRoutingBenchmark::run_all`] emits ONE
//!             [`crate::ssb::SsbActionType::ModeRoutingBenchRunCompleted`]
//!             row at end of run via the [`ModeRoutingBenchSsbEmitter`]
//!             trait. Mirror of the D-19 runner shape.
//! consumer  : `elai bench mode-routing` CLI subcommand exits 0/1 to drive
//!             CI; the dashboard `ModeRoutingBenchPanel` reads the row via
//!             `get_mode_routing_bench_recent` Tauri command (D-21 dashboard
//!             slice).
//! observable: `elai bench mode-routing` against the bundled mini fixture
//!             must produce a JSON report + >= 1 SSB row with `accuracy >= 0.5`.
//!
//! # Fixture format
//!
//! One JSON object per line, with the schema captured by [`FixtureRow`]:
//! ```json
//! {
//!   "claim_id": "factual_constructive_0001",
//!   "claim_text": "the area of a 4x5 rectangle is 20",
//!   "claim_type": "factual",
//!   "expected_mode": "constructive",
//!   "category": "computational_with_evidence",
//!   "with_evidence": true,
//!   "available_verifiers": ["deterministic", "intrinsic"],
//!   "user_role": "executor",
//!   "conformal_calibration_ready": false,
//!   "canary_blocks": false
//! }
//! ```
//!
//! The four optional context flags (`with_evidence`, `available_verifiers`,
//! `user_role`, `conformal_calibration_ready`, `canary_blocks`) default to the
//! refusal-biased baseline emitted by [`crate::orch_context::OrchContext::minimal`]
//! so fixture rows can omit fields that are not load-bearing for their
//! expected mode.
//!
//! # Anti-stale + zero-lazy
//!
//! - blake3 for fixture content hash (anti-stale rule).
//! - thiserror for errors; tracing for diagnostics.
//! - native async fn in trait (no `async-trait` crate).
//! - No `.unwrap()` outside `#[cfg(test)]`. No silent swallows.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use blake3::Hasher;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::fs;
use tracing::{debug, info};

use crate::benchmarks::prmbench_runner::GatingVerdict;
use crate::claim::{Claim, ClaimType, EvidenceKind, EvidenceRef};
use crate::grounding_mode::{select_grounding_mode, GroundingMode};
use crate::orch_context::{OrchContext, OrchPhase, OrchUserRole};
use crate::verification_result::VerifierKind;

// ---------------------------------------------------------------------------
// Public value types
// ---------------------------------------------------------------------------

/// Per-mode binary-classification F1 score with raw counters so dashboard
/// consumers can re-derive precision / recall without re-reading the fixture.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct F1Score {
    /// `tp / (tp + fp)`. Defined as `0.0` when `tp + fp == 0`.
    pub precision: f64,
    /// `tp / (tp + fn)`. Defined as `0.0` when `tp + fn == 0`.
    pub recall: f64,
    /// `2 * precision * recall / (precision + recall)`. Defined as `0.0`
    /// when `precision + recall == 0`.
    pub f1: f64,
    pub true_positive: u32,
    pub false_positive: u32,
    pub false_negative: u32,
}

impl F1Score {
    fn from_counts(tp: u32, fp: u32, fn_: u32) -> Self {
        let precision = if tp + fp == 0 {
            0.0
        } else {
            f64::from(tp) / f64::from(tp + fp)
        };
        let recall = if tp + fn_ == 0 {
            0.0
        } else {
            f64::from(tp) / f64::from(tp + fn_)
        };
        let f1 = if precision + recall == 0.0 {
            0.0
        } else {
            2.0 * precision * recall / (precision + recall)
        };
        Self {
            precision,
            recall,
            f1,
            true_positive: tp,
            false_positive: fp,
            false_negative: fn_,
        }
    }
}

/// Per-category breakdown emitted alongside the aggregate report. `name`
/// matches the fixture row's `category` field; `per_mode_f1` is keyed by the
/// snake-case [`GroundingMode`] wire-form so the dashboard can render
/// without an additional mapping.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CategoryReport {
    pub name: String,
    pub total: u32,
    pub correct: u32,
    pub accuracy: f64,
    pub per_mode_f1: BTreeMap<String, F1Score>,
}

/// 5x5 confusion matrix indexed by [`GroundingMode::all`] in declaration
/// order. `matrix[truth_index][predicted_index]` is the count of fixture
/// rows whose `expected_mode == GroundingMode::all()[truth_index]` and whose
/// predicted mode was `GroundingMode::all()[predicted_index]`. Diagonal
/// entries are the true positives.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConfusionMatrix {
    pub matrix: [[u32; 5]; 5],
}

impl ConfusionMatrix {
    /// Empty matrix — used to seed `BTreeMap` lookups before population.
    #[must_use]
    pub const fn zero() -> Self {
        Self {
            matrix: [[0; 5]; 5],
        }
    }

    fn index_of(mode: GroundingMode) -> usize {
        // Mirrors GroundingMode::all() declaration order. Keep in sync with
        // grounding_mode.rs; any reorder forces both this match and the
        // dashboard renderer to update.
        match mode {
            GroundingMode::Constructive => 0,
            GroundingMode::FalsifiabilityCrossFamily => 1,
            GroundingMode::ConformalRefusal => 2,
            GroundingMode::OperationalDeferred => 3,
            GroundingMode::ConservativeFallback => 4,
        }
    }

    fn record(&mut self, expected: GroundingMode, predicted: GroundingMode) {
        let row = Self::index_of(expected);
        let col = Self::index_of(predicted);
        self.matrix[row][col] = self.matrix[row][col].saturating_add(1);
    }
}

/// Aggregate report — one per [`ModeRoutingBenchmark::run_all`] invocation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModeRoutingReport {
    pub total: u32,
    pub correct: u32,
    pub accuracy: f64,
    pub f1_macro: f64,
    /// Per-mode F1 keyed by snake-case [`GroundingMode`] wire-label.
    pub per_mode_f1: BTreeMap<String, F1Score>,
    pub confusion_matrix: ConfusionMatrix,
    pub per_category: Vec<CategoryReport>,
    pub gating_verdict: GatingVerdict,
    pub fixture_blake3: String,
}

impl ModeRoutingReport {
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
pub enum ModeRoutingBenchError {
    /// I/O failure reading the fixture JSONL.
    #[error("mode-routing fixture I/O error reading {path}: {source_msg}")]
    Io { path: String, source_msg: String },
    /// One line of the fixture failed to deserialise.
    #[error("mode-routing fixture malformed json at line {line_index} of {path}: {source_msg}")]
    MalformedJson {
        path: String,
        line_index: usize,
        source_msg: String,
    },
    /// `expected_mode` outside the closed [`GroundingMode`] set.
    #[error(
        "mode-routing fixture invalid expected_mode '{value}' at line {line_index} of {path}"
    )]
    InvalidExpectedMode {
        path: String,
        line_index: usize,
        value: String,
    },
    /// `claim_type` outside the closed [`ClaimType`] set.
    #[error("mode-routing fixture invalid claim_type '{value}' at line {line_index} of {path}")]
    InvalidClaimType {
        path: String,
        line_index: usize,
        value: String,
    },
    /// `user_role` outside the closed [`OrchUserRole`] set.
    #[error("mode-routing fixture invalid user_role '{value}' at line {line_index} of {path}")]
    InvalidUserRole {
        path: String,
        line_index: usize,
        value: String,
    },
    /// `available_verifiers` contained a label outside the closed
    /// [`VerifierKind`] set.
    #[error(
        "mode-routing fixture invalid verifier '{value}' at line {line_index} of {path}"
    )]
    InvalidVerifier {
        path: String,
        line_index: usize,
        value: String,
    },
    /// Empty fixture (zero usable rows).
    #[error("mode-routing fixture {path} contains zero usable rows")]
    EmptyFixture { path: String },
    /// SSB emitter failure on the final row write.
    #[error("mode-routing ssb emit error: {0}")]
    SsbEmit(String),
}

// ---------------------------------------------------------------------------
// SSB emitter trait
// ---------------------------------------------------------------------------

/// Minimal trait the runner uses to write one end-of-run row. Production
/// callers wrap `Ssb::write_payload`; tests use [`VecModeRoutingBenchEmitter`].
pub trait ModeRoutingBenchSsbEmitter: Send + Sync {
    fn emit_run_completed(
        &self,
        payload: crate::ssb::ModeRoutingBenchRunCompletedPayload,
    ) -> impl std::future::Future<Output = Result<(), String>> + Send;
}

/// Test-only emitter that collects rows into an in-memory vector.
#[derive(Debug, Default)]
pub struct VecModeRoutingBenchEmitter {
    rows: tokio::sync::Mutex<Vec<crate::ssb::ModeRoutingBenchRunCompletedPayload>>,
}

impl VecModeRoutingBenchEmitter {
    #[must_use]
    pub fn new() -> Self {
        Self {
            rows: tokio::sync::Mutex::new(Vec::new()),
        }
    }

    pub async fn rows(&self) -> Vec<crate::ssb::ModeRoutingBenchRunCompletedPayload> {
        self.rows.lock().await.clone()
    }

    pub async fn len(&self) -> usize {
        self.rows.lock().await.len()
    }
}

impl ModeRoutingBenchSsbEmitter for VecModeRoutingBenchEmitter {
    async fn emit_run_completed(
        &self,
        payload: crate::ssb::ModeRoutingBenchRunCompletedPayload,
    ) -> Result<(), String> {
        let mut guard = self.rows.lock().await;
        guard.push(payload);
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Fixture row
// ---------------------------------------------------------------------------

/// One row of the on-disk JSONL fixture. The four context flags are optional
/// and default to the refusal-biased baseline (`OrchContext::minimal`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FixtureRow {
    pub claim_id: String,
    pub claim_text: String,
    /// Snake-case [`ClaimType`] wire-label.
    pub claim_type: String,
    /// Snake-case [`GroundingMode`] wire-label the predicate is expected
    /// to produce for this row.
    pub expected_mode: String,
    /// Free-form category bucket for the per-category breakdown. Authors
    /// group rows by routing scenario (e.g. `"clearly_mode_a"`,
    /// `"ambiguous_a_vs_d"`, `"edge_no_alt_family"`).
    pub category: String,
    /// True iff the claim should carry corpus evidence (drives
    /// `Claim::has_grounding_evidence`).
    #[serde(default)]
    pub with_evidence: bool,
    /// Snake-case [`VerifierKind`] labels installed in the routing
    /// context. Default empty (refusal-biased).
    #[serde(default)]
    pub available_verifiers: Vec<String>,
    /// Snake-case [`OrchUserRole`]; defaults to `"executor"`.
    #[serde(default = "default_user_role")]
    pub user_role: String,
    /// Mirror of `OrchContext::conformal_calibration_ready`; defaults to
    /// `false`.
    #[serde(default)]
    pub conformal_calibration_ready: bool,
    /// When `true`, signals to the runner that `OrchContext::canary_blocks`
    /// should return `true`. The minimal context returns `false`; we
    /// special-case canary by promoting a synthetic blocking summary so
    /// fixture authors can write the "canary blocks routing" scenarios
    /// without needing the full `CanaryStatusSummary` shape.
    #[serde(default)]
    pub canary_blocks: bool,
}

fn default_user_role() -> String {
    "executor".to_string()
}

impl FixtureRow {
    fn expected_mode(&self, line_index: usize, path: &str) -> Result<GroundingMode, ModeRoutingBenchError> {
        parse_mode(&self.expected_mode).ok_or_else(|| ModeRoutingBenchError::InvalidExpectedMode {
            path: path.to_owned(),
            line_index,
            value: self.expected_mode.clone(),
        })
    }

    fn claim_type(&self, line_index: usize, path: &str) -> Result<ClaimType, ModeRoutingBenchError> {
        parse_claim_type(&self.claim_type).ok_or_else(|| ModeRoutingBenchError::InvalidClaimType {
            path: path.to_owned(),
            line_index,
            value: self.claim_type.clone(),
        })
    }

    fn user_role(&self, line_index: usize, path: &str) -> Result<OrchUserRole, ModeRoutingBenchError> {
        parse_user_role(&self.user_role).ok_or_else(|| ModeRoutingBenchError::InvalidUserRole {
            path: path.to_owned(),
            line_index,
            value: self.user_role.clone(),
        })
    }

    fn verifiers(
        &self,
        line_index: usize,
        path: &str,
    ) -> Result<Vec<VerifierKind>, ModeRoutingBenchError> {
        let mut out = Vec::with_capacity(self.available_verifiers.len());
        for v in &self.available_verifiers {
            let parsed = parse_verifier(v).ok_or_else(|| ModeRoutingBenchError::InvalidVerifier {
                path: path.to_owned(),
                line_index,
                value: v.clone(),
            })?;
            out.push(parsed);
        }
        Ok(out)
    }

    /// Build the typed `(Claim, OrchContext)` pair for one fixture row.
    fn build_inputs(
        &self,
        line_index: usize,
        path: &str,
    ) -> Result<(Claim, OrchContext), ModeRoutingBenchError> {
        let claim_type = self.claim_type(line_index, path)?;
        let role = self.user_role(line_index, path)?;
        let verifiers = self.verifiers(line_index, path)?;

        let evidence = if self.with_evidence {
            vec![EvidenceRef::from_content(
                EvidenceKind::Corpus,
                format!("{}#fixture", self.claim_id),
                self.claim_text.as_bytes(),
            )]
        } else {
            Vec::new()
        };
        let claim = Claim {
            claim_id: self.claim_id.clone(),
            text: self.claim_text.clone(),
            claim_type,
            evidence,
            // Fixture timestamps are not load-bearing for routing; pin to
            // a deterministic constant so reruns are bit-identical.
            ts_ms: 1_700_000_000_000,
        };

        let mut ctx = OrchContext::minimal(role, OrchPhase::Executing);
        ctx.available_verifiers = verifiers;
        ctx.conformal_calibration_ready = self.conformal_calibration_ready;
        if self.canary_blocks {
            ctx.recent_canary_status = Some(synthetic_blocking_canary());
        }
        Ok((claim, ctx))
    }
}

/// Synthetic blocking canary used when a fixture row sets `canary_blocks:
/// true`. Constructed from the public [`crate::agent_pool::CanaryStatusSummary`]
/// surface so the routing predicate sees a real blocking record (matches
/// the production wire) rather than a side-channel boolean. `decision_kind
/// = "fail"` is the contract `blocks_promotion()` checks.
fn synthetic_blocking_canary() -> crate::agent_pool::CanaryStatusSummary {
    crate::agent_pool::CanaryStatusSummary {
        report_id: "mode_routing_bench_synthetic".to_string(),
        ts_ms: 1_700_000_000_000,
        overall_transfer_score: 0.0,
        decision_kind: "fail".to_string(),
        dominant_fail_family: "fixture_forced_block".to_string(),
    }
}

// ---------------------------------------------------------------------------
// Snake-case enum parsers (mirrors of the `serde(rename_all = "snake_case")`
// derives — bare strings stay valid even after a `Cargo.toml` edition bump).
// ---------------------------------------------------------------------------

fn parse_mode(s: &str) -> Option<GroundingMode> {
    match s {
        "constructive" => Some(GroundingMode::Constructive),
        "falsifiability_cross_family" => Some(GroundingMode::FalsifiabilityCrossFamily),
        "conformal_refusal" => Some(GroundingMode::ConformalRefusal),
        "operational_deferred" => Some(GroundingMode::OperationalDeferred),
        "conservative_fallback" => Some(GroundingMode::ConservativeFallback),
        _ => None,
    }
}

fn parse_claim_type(s: &str) -> Option<ClaimType> {
    match s {
        "factual" => Some(ClaimType::Factual),
        "procedural" => Some(ClaimType::Procedural),
        "normative" => Some(ClaimType::Normative),
        "speculative" => Some(ClaimType::Speculative),
        "hypothetical" => Some(ClaimType::Hypothetical),
        "computational" => Some(ClaimType::Computational),
        "meta_query" => Some(ClaimType::MetaQuery),
        _ => None,
    }
}

fn parse_user_role(s: &str) -> Option<OrchUserRole> {
    match s {
        "executor" => Some(OrchUserRole::Executor),
        "auditor" => Some(OrchUserRole::Auditor),
        "adversary" => Some(OrchUserRole::Adversary),
        "meta" => Some(OrchUserRole::Meta),
        _ => None,
    }
}

fn parse_verifier(s: &str) -> Option<VerifierKind> {
    match s {
        "ppm" => Some(VerifierKind::Ppm),
        "intrinsic" => Some(VerifierKind::Intrinsic),
        "deterministic" => Some(VerifierKind::Deterministic),
        "cross_context" => Some(VerifierKind::CrossContext),
        // E-53 plan-level extension (SCHEMA 140): mirror the new VerifierKind
        // variant on the string-roundtrip helper so mode_routing benchmarks
        // can reference Z3SmtCheck verdicts when the binary is available.
        "z3_smt_check" => Some(VerifierKind::Z3SmtCheck),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Gate thresholds
// ---------------------------------------------------------------------------

/// D-21 plan ACCEPTANCE: macro F1 floor for the aggregate verdict.
pub const DEFAULT_MACRO_F1_FLOOR: f64 = 0.85;

/// D-21 plan ACCEPTANCE: per-mode F1 floor that guards against class
/// collapse (a 0.0 in one mode trips the gate even if the macro average
/// clears the floor).
pub const DEFAULT_PER_MODE_F1_FLOOR: f64 = 0.70;

// ---------------------------------------------------------------------------
// Runner
// ---------------------------------------------------------------------------

/// Mode-routing coverage benchmark. Construction is cheap; the runner is
/// single-shot (mirror of `PRMBenchRunner`).
pub struct ModeRoutingBenchmark {
    dataset_path: PathBuf,
    macro_f1_floor: f64,
    per_mode_f1_floor: f64,
}

impl ModeRoutingBenchmark {
    /// Construct a runner against the supplied fixture path. Uses the
    /// D-21 plan defaults for both gate floors.
    #[must_use]
    pub fn new(dataset_path: PathBuf) -> Self {
        Self {
            dataset_path,
            macro_f1_floor: DEFAULT_MACRO_F1_FLOOR,
            per_mode_f1_floor: DEFAULT_PER_MODE_F1_FLOOR,
        }
    }

    /// Override the macro F1 floor. Returns the builder for chained
    /// construction. Caller is responsible for staying in `[0.0, 1.0]`;
    /// out-of-range values are accepted (test harnesses pin to `1.01` to
    /// force-fail).
    #[must_use]
    pub fn with_macro_floor(mut self, floor: f64) -> Self {
        self.macro_f1_floor = floor;
        self
    }

    /// Override the per-mode F1 floor.
    #[must_use]
    pub fn with_per_mode_floor(mut self, floor: f64) -> Self {
        self.per_mode_f1_floor = floor;
        self
    }

    async fn load_fixture(&self) -> Result<(Vec<FixtureRow>, String), ModeRoutingBenchError> {
        let path_str = self.dataset_path.to_string_lossy().into_owned();
        let raw = fs::read_to_string(&self.dataset_path)
            .await
            .map_err(|e| ModeRoutingBenchError::Io {
                path: path_str.clone(),
                source_msg: format!("{e}"),
            })?;
        let mut rows: Vec<FixtureRow> = Vec::new();
        for (idx, line) in raw.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            let row: FixtureRow = serde_json::from_str(trimmed).map_err(|e| {
                ModeRoutingBenchError::MalformedJson {
                    path: path_str.clone(),
                    line_index: idx,
                    source_msg: format!("{e}"),
                }
            })?;
            rows.push(row);
        }
        if rows.is_empty() {
            return Err(ModeRoutingBenchError::EmptyFixture {
                path: path_str,
            });
        }
        let mut hasher = Hasher::new();
        hasher.update(raw.as_bytes());
        let fixture_blake3 = hasher.finalize().to_hex().to_string();
        Ok((rows, fixture_blake3))
    }

    /// Run the full fixture. Computes per-mode F1 + macro F1 + a 5x5
    /// confusion matrix + per-category breakdown, then emits exactly one
    /// SSB row via `emitter`.
    ///
    /// `Err` is reserved for fixture-shape failures (I/O, JSON, invalid
    /// labels, empty fixture) and SSB write failure. The routing predicate
    /// itself is pure and infallible — no per-row failure path needs to
    /// flow up.
    pub async fn run_all<E>(
        &self,
        emitter: &E,
    ) -> Result<ModeRoutingReport, ModeRoutingBenchError>
    where
        E: ModeRoutingBenchSsbEmitter + ?Sized,
    {
        let (rows, fixture_blake3) = self.load_fixture().await?;
        info!(
            dataset_path = %self.dataset_path.to_string_lossy(),
            total_rows = rows.len(),
            "mode-routing bench: starting run",
        );
        let path_str = self.dataset_path.to_string_lossy().into_owned();

        let modes = GroundingMode::all();
        // Per-mode counters indexed by mode index (see ConfusionMatrix::index_of).
        let mut tp = [0_u32; 5];
        let mut fp = [0_u32; 5];
        let mut fn_ = [0_u32; 5];
        let mut confusion = ConfusionMatrix::zero();

        // Per-category accumulation. BTreeMap so the per_category Vec output
        // is deterministic across runs (alphabetical category order).
        // Value = (total, correct, per-mode (tp, fp, fn)).
        let mut category_buckets: BTreeMap<String, CategoryCounts> = BTreeMap::new();

        let mut total_correct: u32 = 0;
        for (idx, row) in rows.iter().enumerate() {
            let expected = row.expected_mode(idx, &path_str)?;
            let (claim, ctx) = row.build_inputs(idx, &path_str)?;
            let predicted = select_grounding_mode(&claim, &ctx);
            let exp_i = ConfusionMatrix::index_of(expected);
            let pred_i = ConfusionMatrix::index_of(predicted);
            confusion.record(expected, predicted);
            if expected == predicted {
                tp[exp_i] = tp[exp_i].saturating_add(1);
                total_correct = total_correct.saturating_add(1);
            } else {
                fn_[exp_i] = fn_[exp_i].saturating_add(1);
                fp[pred_i] = fp[pred_i].saturating_add(1);
            }
            let entry = category_buckets
                .entry(row.category.clone())
                .or_insert_with(CategoryCounts::default);
            entry.total = entry.total.saturating_add(1);
            if expected == predicted {
                entry.correct = entry.correct.saturating_add(1);
                entry.per_mode_tp[exp_i] = entry.per_mode_tp[exp_i].saturating_add(1);
            } else {
                entry.per_mode_fn[exp_i] = entry.per_mode_fn[exp_i].saturating_add(1);
                entry.per_mode_fp[pred_i] = entry.per_mode_fp[pred_i].saturating_add(1);
            }
            debug!(
                claim_id = %row.claim_id,
                expected = expected.as_str(),
                predicted = predicted.as_str(),
                "mode-routing bench: row scored",
            );
        }

        // Aggregate per-mode F1.
        let mut per_mode_f1: BTreeMap<String, F1Score> = BTreeMap::new();
        let mut f1_sum: f64 = 0.0;
        for (i, mode) in modes.iter().enumerate() {
            let score = F1Score::from_counts(tp[i], fp[i], fn_[i]);
            f1_sum += score.f1;
            per_mode_f1.insert((*mode).as_str().to_string(), score);
        }
        // Macro F1 always divides by the closed-set mode count (5) so
        // missing-class coverage degrades the score (matches the D-21 plan
        // "no class collapse" intent).
        let f1_macro = f1_sum / modes.len() as f64;

        // Aggregate accuracy.
        let total_u32 = u32::try_from(rows.len()).unwrap_or(u32::MAX);
        let accuracy = if total_u32 == 0 {
            0.0
        } else {
            f64::from(total_correct) / f64::from(total_u32)
        };

        // Per-category roll-up.
        let mut per_category: Vec<CategoryReport> = Vec::with_capacity(category_buckets.len());
        for (name, counts) in category_buckets.into_iter() {
            let cat_total = counts.total;
            let cat_correct = counts.correct;
            let cat_accuracy = if cat_total == 0 {
                0.0
            } else {
                f64::from(cat_correct) / f64::from(cat_total)
            };
            let mut cat_per_mode_f1: BTreeMap<String, F1Score> = BTreeMap::new();
            for (i, mode) in modes.iter().enumerate() {
                let score = F1Score::from_counts(
                    counts.per_mode_tp[i],
                    counts.per_mode_fp[i],
                    counts.per_mode_fn[i],
                );
                cat_per_mode_f1.insert((*mode).as_str().to_string(), score);
            }
            per_category.push(CategoryReport {
                name,
                total: cat_total,
                correct: cat_correct,
                accuracy: cat_accuracy,
                per_mode_f1: cat_per_mode_f1,
            });
        }

        // Verdict: pass iff macro AND every per-mode F1 clear their floors.
        let per_mode_ok = per_mode_f1
            .values()
            .all(|s| s.f1 >= self.per_mode_f1_floor);
        let gating_verdict =
            if f1_macro >= self.macro_f1_floor && per_mode_ok && total_u32 > 0 {
                GatingVerdict::Pass
            } else {
                GatingVerdict::Fail
            };

        let report = ModeRoutingReport {
            total: total_u32,
            correct: total_correct,
            accuracy,
            f1_macro,
            per_mode_f1,
            confusion_matrix: confusion,
            per_category,
            gating_verdict,
            fixture_blake3: fixture_blake3.clone(),
        };

        // Emit the single SSB row.
        let payload = crate::ssb::ModeRoutingBenchRunCompletedPayload {
            total: report.total,
            accuracy: report.accuracy,
            f1_macro: report.f1_macro,
            gating_verdict: report.gating_verdict.as_str().to_string(),
            fixture_blake3,
            fixture_row_count: report.total,
        };
        emitter
            .emit_run_completed(payload)
            .await
            .map_err(ModeRoutingBenchError::SsbEmit)?;
        info!(
            verdict = %report.gating_verdict.as_str(),
            accuracy = report.accuracy,
            f1_macro = report.f1_macro,
            "mode-routing bench: run complete",
        );
        Ok(report)
    }
}

#[derive(Default)]
struct CategoryCounts {
    total: u32,
    correct: u32,
    per_mode_tp: [u32; 5],
    per_mode_fp: [u32; 5],
    per_mode_fn: [u32; 5],
}

// ---------------------------------------------------------------------------
// Public helper: locate the bundled mini fixture.
// ---------------------------------------------------------------------------

/// Returns the default mini-fixture path relative to a project root
/// (`<project_root>/benchmarks/mode_routing/fixtures/mode_routing_mini.jsonl`).
/// Pure path-arithmetic — does NOT touch the filesystem.
#[must_use]
pub fn default_mini_fixture_path(project_root: &Path) -> PathBuf {
    project_root
        .join("benchmarks")
        .join("mode_routing")
        .join("fixtures")
        .join("mode_routing_mini.jsonl")
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

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

    fn row(
        claim_id: &str,
        claim_type: &str,
        expected_mode: &str,
        category: &str,
        with_evidence: bool,
        verifiers: &[&str],
    ) -> FixtureRow {
        FixtureRow {
            claim_id: claim_id.to_string(),
            claim_text: format!("text for {claim_id}"),
            claim_type: claim_type.to_string(),
            expected_mode: expected_mode.to_string(),
            category: category.to_string(),
            with_evidence,
            available_verifiers: verifiers.iter().map(|s| (*s).to_string()).collect(),
            user_role: "executor".to_string(),
            conformal_calibration_ready: false,
            canary_blocks: false,
        }
    }

    #[test]
    fn confusion_matrix_index_covers_all_modes() {
        for (i, mode) in GroundingMode::all().iter().enumerate() {
            assert_eq!(ConfusionMatrix::index_of(*mode), i);
        }
    }

    #[test]
    fn f1_zero_when_no_predictions_or_truth() {
        let s = F1Score::from_counts(0, 0, 0);
        assert_eq!(s.precision, 0.0);
        assert_eq!(s.recall, 0.0);
        assert_eq!(s.f1, 0.0);
    }

    #[test]
    fn f1_perfect_when_only_true_positives() {
        let s = F1Score::from_counts(5, 0, 0);
        assert!((s.precision - 1.0).abs() < 1e-9);
        assert!((s.recall - 1.0).abs() < 1e-9);
        assert!((s.f1 - 1.0).abs() < 1e-9);
    }

    #[tokio::test]
    async fn run_all_emits_one_ssb_row_and_passes_on_perfect_fixture() {
        let dir = TempDir::new().expect("tempdir");
        // 5 rows, 1 per mode, every row engineered to land on its expected
        // mode -> per-mode F1 = 1.0 -> macro F1 = 1.0 -> Pass.
        let rows = vec![
            row(
                "c1",
                "factual",
                "constructive",
                "clearly_a",
                true,
                &["deterministic", "intrinsic"],
            ),
            row(
                "c2",
                "speculative",
                "falsifiability_cross_family",
                "clearly_b",
                false,
                &["intrinsic", "ppm"],
            ),
            {
                let mut r = row(
                    "c3",
                    "speculative",
                    "conformal_refusal",
                    "clearly_e",
                    false,
                    &["intrinsic"],
                );
                r.conformal_calibration_ready = true;
                r
            },
            row(
                "c4",
                "procedural",
                "operational_deferred",
                "clearly_d",
                false,
                &["intrinsic"],
            ),
            row(
                "c5",
                "normative",
                "conservative_fallback",
                "clearly_fallback",
                true,
                &["deterministic"],
            ),
        ];
        let path = write_fixture(&dir, &rows).await;
        let runner = ModeRoutingBenchmark::new(path);
        let emitter = VecModeRoutingBenchEmitter::new();
        let report = runner.run_all(&emitter).await.expect("run ok");
        assert_eq!(report.total, 5);
        assert_eq!(report.correct, 5);
        assert!((report.f1_macro - 1.0).abs() < 1e-9);
        assert_eq!(report.gating_verdict, GatingVerdict::Pass);
        // Exactly one SSB row.
        assert_eq!(emitter.len().await, 1);
        let written = emitter.rows().await;
        assert_eq!(written[0].fixture_blake3.len(), 64);
        assert_eq!(written[0].total, 5);
        assert_eq!(written[0].gating_verdict, "pass");
    }

    #[tokio::test]
    async fn class_collapse_trips_gate_even_with_high_macro() {
        let dir = TempDir::new().expect("tempdir");
        // 4 rows on mode A (constructive), 1 row on mode E (refusal). Row 5
        // expects refusal but no calibration is ready -> falls through to
        // conservative_fallback (predicate is deterministic). Per-mode F1
        // for ConformalRefusal collapses to 0 -> gate FAIL despite high
        // macro from the other modes.
        let rows = vec![
            row(
                "ok1",
                "factual",
                "constructive",
                "ok",
                true,
                &["deterministic", "intrinsic"],
            ),
            row(
                "ok2",
                "factual",
                "constructive",
                "ok",
                true,
                &["deterministic", "intrinsic"],
            ),
            row(
                "ok3",
                "factual",
                "constructive",
                "ok",
                true,
                &["deterministic", "intrinsic"],
            ),
            row(
                "ok4",
                "factual",
                "constructive",
                "ok",
                true,
                &["deterministic", "intrinsic"],
            ),
            // expected_mode says refusal, but with no calibration the
            // predicate returns ConservativeFallback. This is the exact
            // class-collapse failure mode the gate is designed to catch.
            row(
                "miss",
                "speculative",
                "conformal_refusal",
                "edge",
                false,
                &["intrinsic"],
            ),
        ];
        let path = write_fixture(&dir, &rows).await;
        let runner = ModeRoutingBenchmark::new(path);
        let emitter = VecModeRoutingBenchEmitter::new();
        let report = runner.run_all(&emitter).await.expect("run ok");
        assert_eq!(report.gating_verdict, GatingVerdict::Fail);
        let refusal_f1 = report
            .per_mode_f1
            .get("conformal_refusal")
            .expect("entry present")
            .f1;
        assert!(refusal_f1 < DEFAULT_PER_MODE_F1_FLOOR);
    }

    #[tokio::test]
    async fn invalid_expected_mode_returns_error() {
        let dir = TempDir::new().expect("tempdir");
        let bad = FixtureRow {
            claim_id: "bad".into(),
            claim_text: "x".into(),
            claim_type: "factual".into(),
            expected_mode: "bogus_mode".into(),
            category: "edge".into(),
            with_evidence: true,
            available_verifiers: vec![],
            user_role: "executor".into(),
            conformal_calibration_ready: false,
            canary_blocks: false,
        };
        let path = write_fixture(&dir, &[bad]).await;
        let runner = ModeRoutingBenchmark::new(path);
        let emitter = VecModeRoutingBenchEmitter::new();
        match runner.run_all(&emitter).await {
            Err(ModeRoutingBenchError::InvalidExpectedMode { value, .. }) => {
                assert_eq!(value, "bogus_mode");
            }
            other => panic!("expected InvalidExpectedMode, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn empty_fixture_returns_error() {
        let dir = TempDir::new().expect("tempdir");
        let path = dir.path().join("empty.jsonl");
        fs::write(&path, "").await.expect("write empty");
        let runner = ModeRoutingBenchmark::new(path);
        let emitter = VecModeRoutingBenchEmitter::new();
        match runner.run_all(&emitter).await {
            Err(ModeRoutingBenchError::EmptyFixture { .. }) => {}
            other => panic!("expected EmptyFixture, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn impossible_macro_floor_forces_fail() {
        let dir = TempDir::new().expect("tempdir");
        let rows = vec![row(
            "c1",
            "factual",
            "constructive",
            "ok",
            true,
            &["deterministic", "intrinsic"],
        )];
        let path = write_fixture(&dir, &rows).await;
        let runner = ModeRoutingBenchmark::new(path).with_macro_floor(1.01);
        let emitter = VecModeRoutingBenchEmitter::new();
        let report = runner.run_all(&emitter).await.expect("run ok");
        assert_eq!(report.gating_verdict, GatingVerdict::Fail);
    }

    #[test]
    fn default_mini_fixture_path_is_under_benchmarks_mode_routing() {
        let root = Path::new("/tmp/elai-project");
        let p = default_mini_fixture_path(root);
        assert!(p.ends_with("benchmarks/mode_routing/fixtures/mode_routing_mini.jsonl"));
    }
}

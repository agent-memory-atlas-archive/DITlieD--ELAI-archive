//! D-16 — UniCR conformal-refusal calibration (formal Mode E grounding).
//!
//! Implements a minimal, pure-quantile UniCR-style conformal predictor on
//! a corpus of `(claim_score, ground_truth)` calibration tuples. The
//! threshold `tau` is the (1 - epsilon) empirical quantile of the
//! non-conformity scores `1 - predicted_score`. A new claim with
//! `(1 - claim_score) > tau` lands in the conformal refusal region —
//! `decide()` returns [`RefusalDecision::Abstain`]; otherwise it returns
//! [`RefusalDecision::Answer`] carrying the model's confidence.
//!
//! Grounding sources (per CLAUDE.md grounded-by-default rule):
//!   * arXiv:2503.01345 — UniCR (Unified Conformal Refusal): tau is the
//!     empirical-quantile threshold over the calibration corpus's
//!     non-conformity scores, with `epsilon = 0.1` as the default
//!     operating point.
//!   * Vovk, V., Gammerman, A., Shafer, G. — *Algorithmic Learning in a
//!     Random World* (Springer, 2005), §2.2: the quantile-based
//!     conformal predictor is the original construction the UniCR
//!     refusal-region rule specialises.
//!   * docs/thefinalglance/D-plan/D-16_unicr_conformal_refusal.md (the
//!     full plan; this module ships the orchestrator-scoped MVP).
//!
//! Scope (this module ships the orchestrator-scoped MVP):
//!   * pure types + pure compute_tau + pure decide();
//!   * an SSB-emitting producer surface
//!     ([`dispatch_conformal_refusal`]) that mirrors
//!     [`crate::agent_pool::route_claim`] in shape;
//!   * a typed projection of the recent SSB rows for the dashboard.
//!
//! Out-of-scope (deferred to a later D-16 follow-up):
//!   * the nightly L3-locked TauUpdater + persisted `tau.txt`;
//!   * the CLI `elai calibration {status,update}` subcommand;
//!   * the elai_cc.toml `[conformal]` section + composite-weights
//!     loading (the seed corpus already carries `predicted_score` as a
//!     pre-composed scalar so this module does not need the weights).
//!
//! No new external dep. blake3, tracing, thiserror, native async-fn in
//! trait — per the CLAUDE.md anti-stale-training rule.

use std::path::Path;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::warn;

use crate::ssb::{ConformalRefusalDecidedPayload, ConformalTauComputedPayload};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Default UniCR operating point (arXiv:2503.01345 §6). Tunable by the
/// orchestrator caller; the seed corpus + integration test exercise the
/// default explicitly.
pub const DEFAULT_EPSILON: f64 = 0.1;

/// Minimum corpus size accepted by [`compute_tau`]. The MVP integration
/// test corpus is `>= 20`; the production gate
/// (`docs/thefinalglance/D-plan/D-16_unicr_conformal_refusal.md` task 2)
/// raises this to `>= 500` once the nightly grower is online. The
/// constant is intentionally low here so the seed corpus exercises the
/// full producer path; callers can raise the floor with
/// [`compute_tau_with_min`].
pub const DEFAULT_MIN_CORPUS_SIZE: usize = 20;

/// Maximum `claim_text` length stored per tuple. Bounds payload size on
/// the SSB row (the producer uses the `claim_id`, not the text, for the
/// SSB row — text is corpus-side only).
pub const MAX_CLAIM_TEXT_CHARS: usize = 256;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// A single `(x, y)` calibration tuple consumed by [`compute_tau`].
///
/// `predicted_score` is the composite confidence the orchestrator
/// recorded at decision time; it is a SCALAR in `[0.0, 1.0]` (consumer
/// side, NOT the source-of-truth). `ground_truth` is the post-hoc verdict
/// (`true` if the claim turned out correct).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CalibrationTuple {
    /// Stable per-tuple id (seed corpus uses `seed-<n>`; production
    /// corpus uses a blake3-derived id off the claim text + ts).
    pub tuple_id: String,
    /// Originating [`crate::claim::Claim`] id (synthetic for seed
    /// tuples).
    pub claim_id: String,
    /// Bounded claim text (truncated at [`MAX_CLAIM_TEXT_CHARS`] by
    /// every constructor).
    pub claim_text: String,
    /// Post-hoc verdict — `true` if the claim was confirmed correct.
    pub ground_truth: bool,
    /// Composite confidence in `[0.0, 1.0]` at decision time.
    pub predicted_score: f64,
    /// UNIX ms at corpus-append time.
    pub ts_ms: i64,
}

impl CalibrationTuple {
    /// Construct a new tuple, truncating `claim_text` and clamping
    /// `predicted_score` to `[0.0, 1.0]`. Non-finite scores collapse to
    /// `0.0` (maximally uncertain).
    #[must_use]
    pub fn new(
        tuple_id: impl Into<String>,
        claim_id: impl Into<String>,
        claim_text: impl AsRef<str>,
        ground_truth: bool,
        predicted_score: f64,
        ts_ms: i64,
    ) -> Self {
        let raw_text = claim_text.as_ref();
        let bounded_text = if raw_text.chars().count() <= MAX_CLAIM_TEXT_CHARS {
            raw_text.to_string()
        } else {
            raw_text.chars().take(MAX_CLAIM_TEXT_CHARS).collect()
        };
        let clamped = if predicted_score.is_finite() {
            predicted_score.clamp(0.0, 1.0)
        } else {
            0.0
        };
        Self {
            tuple_id: tuple_id.into(),
            claim_id: claim_id.into(),
            claim_text: bounded_text,
            ground_truth,
            predicted_score: clamped,
            ts_ms,
        }
    }
}

/// Provenance of the calibration corpus the orchestrator loaded.
///
/// Surfaced on every `conformal_tau_computed` SSB row so dashboards can
/// distinguish seed-bootstrap runs from production-grown runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CorpusSource {
    /// Synthesized for testing only — deterministic balanced corpus.
    HeldOut,
    /// Hand-authored corpus shipped under `.elai_cc/calibration/seed/`.
    Synthetic,
    /// Operator-curated corpus (manual review queue).
    Curated,
    /// Auto-grown from the post-claim-resolution hook (production).
    Auto,
}

impl CorpusSource {
    /// Stable snake-case wire label.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::HeldOut => "held_out",
            Self::Synthetic => "synthetic",
            Self::Curated => "curated",
            Self::Auto => "auto",
        }
    }
}

/// In-memory calibration corpus.
///
/// `frozen` mirrors the L3-locked-corpus semantics: once a corpus is
/// frozen the orchestrator's grow-hook refuses to append. The MVP only
/// loads the seed corpus so every shipped corpus is `frozen = true`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CalibrationCorpus {
    pub tuples: Vec<CalibrationTuple>,
    pub source: CorpusSource,
    pub frozen: bool,
}

impl CalibrationCorpus {
    /// Construct an empty corpus with the given provenance. `frozen`
    /// defaults to `false` — production callers explicitly freeze.
    #[must_use]
    pub fn new(source: CorpusSource) -> Self {
        Self {
            tuples: Vec::new(),
            source,
            frozen: false,
        }
    }

    /// Number of tuples in the corpus.
    #[must_use]
    pub fn size(&self) -> usize {
        self.tuples.len()
    }

    /// Append a tuple. Returns `Err` if the corpus is frozen.
    pub fn append(&mut self, tuple: CalibrationTuple) -> Result<(), ConformalError> {
        if self.frozen {
            return Err(ConformalError::CorpusFrozen);
        }
        self.tuples.push(tuple);
        Ok(())
    }

    /// Freeze the corpus — subsequent [`append`] calls fail with
    /// [`ConformalError::CorpusFrozen`].
    pub fn freeze(&mut self) {
        self.frozen = true;
    }

    /// Load a JSONL seed corpus from disk. Each line is parsed as a
    /// [`CalibrationTuple`]; blank lines are skipped. Returns
    /// [`ConformalError::CorpusLoad`] on any I/O or parse failure (the
    /// orchestrator treats an unreadable seed as "no calibration
    /// ready").
    pub fn load_seed_jsonl(path: &Path) -> Result<Self, ConformalError> {
        let raw = std::fs::read_to_string(path).map_err(|e| ConformalError::CorpusLoad {
            path: path.display().to_string(),
            reason: e.to_string(),
        })?;
        let mut tuples = Vec::new();
        for (i, line) in raw.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            let t: CalibrationTuple =
                serde_json::from_str(trimmed).map_err(|e| ConformalError::CorpusLoad {
                    path: path.display().to_string(),
                    reason: format!("line {}: {}", i + 1, e),
                })?;
            // Re-construct through `new()` so the clamp + truncate
            // rules apply uniformly.
            tuples.push(CalibrationTuple::new(
                t.tuple_id,
                t.claim_id,
                t.claim_text,
                t.ground_truth,
                t.predicted_score,
                t.ts_ms,
            ));
        }
        let mut corpus = Self::new(CorpusSource::Synthetic);
        corpus.tuples = tuples;
        corpus.frozen = true;
        Ok(corpus)
    }
}

/// Output of [`decide`].
///
/// `Abstain` carries no payload — the orchestrator's
/// `GroundingMode::ConformalRefusal` arm renders this as the formal
/// "I don't know" surface. `Answer` carries the confidence the
/// orchestrator should pass to downstream renderers.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum RefusalDecision {
    /// The non-conformity score `(1 - claim_score)` exceeded `tau`.
    Abstain,
    /// The claim is within the conformal region. `confidence` is the
    /// original `claim_score` clamped to `[0.0, 1.0]`.
    Answer { confidence: f64 },
}

impl RefusalDecision {
    /// Stable snake-case wire label.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Abstain => "abstain",
            Self::Answer { .. } => "answer",
        }
    }
}

/// Errors raised by D-16 producer surfaces.
#[derive(Debug, Error)]
pub enum ConformalError {
    #[error("calibration corpus has only {actual} tuples; need at least {required}")]
    InsufficientCorpus { actual: usize, required: usize },
    #[error("epsilon {value} is not finite or not in (0, 1)")]
    BadEpsilon { value: f64 },
    #[error("calibration corpus is frozen and cannot accept new tuples")]
    CorpusFrozen,
    #[error("failed to load calibration corpus from '{path}': {reason}")]
    CorpusLoad { path: String, reason: String },
}

// ---------------------------------------------------------------------------
// Pure compute + decide
// ---------------------------------------------------------------------------

/// Compute the UniCR threshold tau on `corpus` at operating point
/// `epsilon`. Returns [`ConformalError::InsufficientCorpus`] when the
/// corpus is smaller than [`DEFAULT_MIN_CORPUS_SIZE`].
///
/// Implementation (Vovk et al. §2.2): tau is the smallest value such
/// that at most `epsilon` fraction of the non-conformity scores
/// `s_i = 1 - predicted_score_i` exceed it. Equivalently tau is the
/// `(1 - epsilon)` empirical quantile of `{s_i}`.
///
/// Pure function — no I/O, no clock, no allocation beyond the sort
/// buffer.
pub fn compute_tau(corpus: &CalibrationCorpus, epsilon: f64) -> Result<f64, ConformalError> {
    compute_tau_with_min(corpus, epsilon, DEFAULT_MIN_CORPUS_SIZE)
}

/// Same as [`compute_tau`] but with a caller-controlled minimum corpus
/// size — used by the production D-16 path which raises the floor to
/// `>= 500` once the nightly grower is online.
pub fn compute_tau_with_min(
    corpus: &CalibrationCorpus,
    epsilon: f64,
    min_corpus_size: usize,
) -> Result<f64, ConformalError> {
    if !epsilon.is_finite() || epsilon <= 0.0 || epsilon >= 1.0 {
        return Err(ConformalError::BadEpsilon { value: epsilon });
    }
    let n = corpus.tuples.len();
    if n < min_corpus_size {
        return Err(ConformalError::InsufficientCorpus {
            actual: n,
            required: min_corpus_size,
        });
    }
    // Build the non-conformity score vector. The scalar score is
    // `1 - predicted_score`; clamp defensively in case a corpus loaded
    // from JSONL slipped through with a non-finite value.
    let mut scores: Vec<f64> = corpus
        .tuples
        .iter()
        .map(|t| {
            let p = if t.predicted_score.is_finite() {
                t.predicted_score.clamp(0.0, 1.0)
            } else {
                0.0
            };
            (1.0 - p).clamp(0.0, 1.0)
        })
        .collect();
    // Total order on f64 — every value is in `[0.0, 1.0]` and finite,
    // so partial_cmp returns Some.
    scores.sort_by(|a, b| {
        a.partial_cmp(b)
            .expect("scores are finite by clamp; partial_cmp must succeed")
    });
    // Index of the `(1 - epsilon)` quantile. Per Vovk §2.2 the
    // conservative pick is `ceil((1 - epsilon) * (n + 1))`; we cap at
    // `n - 1` to stay in-bounds.
    let raw = ((1.0 - epsilon) * ((n + 1) as f64)).ceil();
    let idx = (raw as usize).saturating_sub(1).min(n - 1);
    Ok(scores[idx])
}

/// Decide whether to abstain on a single claim given the precomputed
/// `tau`. Pure function — see [`compute_tau`] for tau semantics.
///
/// `claim_score` is the same `[0.0, 1.0]` composite confidence the
/// calibration corpus carries.
///
/// Per UniCR (arXiv:2503.01345) §3.2 and Vovk et al. §2.2: out-of-range
/// scores are clamped into `[0.0, 1.0]` BEFORE the conformal predicate
/// is evaluated, so the robust behaviour for `+INFINITY` is to clamp to
/// `1.0` (maximally confident) → nonconformity `0.0` → Answer.
/// `-INFINITY` clamps to `0.0` → nonconformity `1.0` → Abstain.
/// `NaN` carries no orderable signal and is therefore fail-closed:
/// the function returns Abstain without further computation.
#[must_use]
pub fn decide(claim_score: f64, tau: f64) -> RefusalDecision {
    // NaN is fail-closed: there is no defensible clamp target for a
    // value with no order relation. Refuse and let the orchestrator
    // surface the formal "I don't know" arm.
    if claim_score.is_nan() {
        return RefusalDecision::Abstain;
    }
    // `f64::clamp` is total on the non-NaN domain: it maps
    // `+INFINITY` to `1.0` and `-INFINITY` to `0.0`, which is the
    // UniCR-prescribed robust handling.
    let p = claim_score.clamp(0.0, 1.0);
    let nonconformity = (1.0 - p).clamp(0.0, 1.0);
    if nonconformity > tau {
        RefusalDecision::Abstain
    } else {
        RefusalDecision::Answer { confidence: p }
    }
}

// ---------------------------------------------------------------------------
// SSB producer surface
// ---------------------------------------------------------------------------

/// Producer-side SSB emitter for `conformal_tau_computed` rows. Mirrors
/// [`crate::agent_pool::ClaimRoutedEmitter`] in shape so production
/// callers can share a single Ssb writer adapter under both traits.
///
/// Native async-fn-in-trait per the anti-stale-training rule.
pub trait ConformalTauComputedEmitter: Send + Sync {
    fn emit(
        &self,
        payload: ConformalTauComputedPayload,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), String>> + Send + '_>>;
}

/// Producer-side SSB emitter for `conformal_refusal_decided` rows.
pub trait ConformalRefusalDecidedEmitter: Send + Sync {
    fn emit(
        &self,
        payload: ConformalRefusalDecidedPayload,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), String>> + Send + '_>>;
}

/// Test-only emitter that records every `conformal_tau_computed`
/// payload behind a `tokio::sync::Mutex` (no `std::sync::Mutex` held
/// across `.await` per the anti-stale-training rule).
#[derive(Default)]
pub struct VecConformalTauComputedEmitter {
    rows: tokio::sync::Mutex<Vec<ConformalTauComputedPayload>>,
}

impl VecConformalTauComputedEmitter {
    #[must_use]
    pub fn new() -> Self {
        Self {
            rows: tokio::sync::Mutex::new(Vec::new()),
        }
    }

    pub async fn rows(&self) -> Vec<ConformalTauComputedPayload> {
        self.rows.lock().await.clone()
    }

    pub async fn len(&self) -> usize {
        self.rows.lock().await.len()
    }
}

impl ConformalTauComputedEmitter for VecConformalTauComputedEmitter {
    fn emit(
        &self,
        payload: ConformalTauComputedPayload,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), String>> + Send + '_>> {
        Box::pin(async move {
            self.rows.lock().await.push(payload);
            Ok(())
        })
    }
}

/// Test-only emitter that records every `conformal_refusal_decided`
/// payload.
#[derive(Default)]
pub struct VecConformalRefusalDecidedEmitter {
    rows: tokio::sync::Mutex<Vec<ConformalRefusalDecidedPayload>>,
}

impl VecConformalRefusalDecidedEmitter {
    #[must_use]
    pub fn new() -> Self {
        Self {
            rows: tokio::sync::Mutex::new(Vec::new()),
        }
    }

    pub async fn rows(&self) -> Vec<ConformalRefusalDecidedPayload> {
        self.rows.lock().await.clone()
    }

    pub async fn len(&self) -> usize {
        self.rows.lock().await.len()
    }
}

impl ConformalRefusalDecidedEmitter for VecConformalRefusalDecidedEmitter {
    fn emit(
        &self,
        payload: ConformalRefusalDecidedPayload,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), String>> + Send + '_>> {
        Box::pin(async move {
            self.rows.lock().await.push(payload);
            Ok(())
        })
    }
}

/// Compute tau over `corpus` at `epsilon`, emit one
/// `conformal_tau_computed` SSB row, and return tau on success.
///
/// `now_ms` is injected (no clock read inside the producer) per the
/// entropy-quarantine rule — matches [`crate::agent_pool::route_claim`]
/// and [`crate::agent_pool::dispatch_constructive_checkers`].
///
/// A failed SSB emission is logged via `tracing::warn` but does NOT
/// override the tau result — the threshold is the load-bearing piece;
/// the row is the durable audit signal.
pub async fn compute_tau_and_emit<E: ConformalTauComputedEmitter + ?Sized>(
    corpus: &CalibrationCorpus,
    epsilon: f64,
    emitter: &E,
    now_ms: i64,
) -> Result<f64, ConformalError> {
    let tau = compute_tau(corpus, epsilon)?;
    let corpus_size_u32 = u32::try_from(corpus.size()).unwrap_or(u32::MAX);
    let payload = ConformalTauComputedPayload {
        tau,
        epsilon,
        corpus_size: corpus_size_u32,
        source: corpus.source.as_str().to_string(),
        ts_ms: now_ms,
    };
    if let Err(e) = emitter.emit(payload).await {
        warn!(
            tau = tau,
            epsilon = epsilon,
            corpus_size = corpus.size(),
            error = %e,
            "compute_tau_and_emit: SSB emission failed (tau still applies)"
        );
    }
    Ok(tau)
}

/// Run the conformal refusal decision for a single claim, emit one
/// `conformal_refusal_decided` SSB row, and return the decision.
///
/// `claim_id` carries through to the SSB row so the dashboard joins
/// against the `claim_routed` rows D-10 emitted (same claim_id space).
pub async fn decide_and_emit<E: ConformalRefusalDecidedEmitter + ?Sized>(
    claim_id: &str,
    claim_score: f64,
    tau: f64,
    emitter: &E,
    now_ms: i64,
) -> RefusalDecision {
    let decision = decide(claim_score, tau);
    let (decision_label, confidence) = match decision {
        RefusalDecision::Abstain => ("abstain".to_string(), 0.0_f64),
        RefusalDecision::Answer { confidence } => ("answer".to_string(), confidence),
    };
    let payload = ConformalRefusalDecidedPayload {
        claim_id: claim_id.to_string(),
        claim_score,
        tau,
        decision: decision_label,
        confidence,
        ts_ms: now_ms,
    };
    if let Err(e) = emitter.emit(payload).await {
        warn!(
            claim_id = claim_id,
            claim_score = claim_score,
            tau = tau,
            error = %e,
            "decide_and_emit: SSB emission failed (decision still applies)"
        );
    }
    decision
}

// ---------------------------------------------------------------------------
// Read-surface projection — typed rows the dashboard consumes
// ---------------------------------------------------------------------------

/// Typed projection of a single `conformal_tau_computed` SSB row.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConformalTauRow {
    pub tau: f64,
    pub epsilon: f64,
    pub corpus_size: u32,
    pub source: String,
    pub ts_ms: i64,
}

/// Typed projection of a single `conformal_refusal_decided` SSB row.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConformalRefusalRow {
    pub claim_id: String,
    pub claim_score: f64,
    pub tau: f64,
    pub decision: String,
    pub confidence: f64,
    pub ts_ms: i64,
}

/// D-16 read surface — the most recent N `conformal_tau_computed` SSB
/// rows the dashboard `ConformalCalibrationPanel` renders.
#[must_use]
pub fn recent_tau_rows(
    viewer: &crate::ssb_viewer::SsbViewer,
    limit: usize,
) -> Vec<ConformalTauRow> {
    let mut out = Vec::new();
    match viewer.recent_rows_of_type("conformal_tau_computed", limit) {
        Ok(details) => {
            for d in details {
                if let Some(row) = tau_detail_to_row(d) {
                    out.push(row);
                }
            }
        }
        Err(e) => {
            warn!(
                action_type = "conformal_tau_computed",
                error = %e,
                "recent_tau_rows: viewer error"
            );
        }
    }
    out.sort_by(|a, b| b.ts_ms.cmp(&a.ts_ms));
    out.truncate(limit);
    out
}

/// D-16 read surface — the most recent N `conformal_refusal_decided`
/// SSB rows.
#[must_use]
pub fn recent_refusal_rows(
    viewer: &crate::ssb_viewer::SsbViewer,
    limit: usize,
) -> Vec<ConformalRefusalRow> {
    let mut out = Vec::new();
    match viewer.recent_rows_of_type("conformal_refusal_decided", limit) {
        Ok(details) => {
            for d in details {
                if let Some(row) = refusal_detail_to_row(d) {
                    out.push(row);
                }
            }
        }
        Err(e) => {
            warn!(
                action_type = "conformal_refusal_decided",
                error = %e,
                "recent_refusal_rows: viewer error"
            );
        }
    }
    out.sort_by(|a, b| b.ts_ms.cmp(&a.ts_ms));
    out.truncate(limit);
    out
}

fn tau_detail_to_row(d: crate::ssb_viewer::SsbActionDetail) -> Option<ConformalTauRow> {
    if d.action_type.as_str() != "conformal_tau_computed" {
        return None;
    }
    let outer = d.payload_json.as_object()?;
    let obj = if let Some(inner) = outer.get("ConformalTauComputed").and_then(|v| v.as_object()) {
        inner.clone()
    } else {
        outer.clone()
    };
    Some(ConformalTauRow {
        tau: obj.get("tau").and_then(|v| v.as_f64()).unwrap_or(0.0),
        epsilon: obj.get("epsilon").and_then(|v| v.as_f64()).unwrap_or(0.0),
        corpus_size: obj
            .get("corpus_size")
            .and_then(|v| v.as_u64())
            .and_then(|v| u32::try_from(v).ok())
            .unwrap_or(0),
        source: obj
            .get("source")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        ts_ms: obj.get("ts_ms").and_then(|v| v.as_i64()).unwrap_or(0),
    })
}

fn refusal_detail_to_row(
    d: crate::ssb_viewer::SsbActionDetail,
) -> Option<ConformalRefusalRow> {
    if d.action_type.as_str() != "conformal_refusal_decided" {
        return None;
    }
    let outer = d.payload_json.as_object()?;
    let obj = if let Some(inner) = outer
        .get("ConformalRefusalDecided")
        .and_then(|v| v.as_object())
    {
        inner.clone()
    } else {
        outer.clone()
    };
    Some(ConformalRefusalRow {
        claim_id: obj
            .get("claim_id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        claim_score: obj
            .get("claim_score")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0),
        tau: obj.get("tau").and_then(|v| v.as_f64()).unwrap_or(0.0),
        decision: obj
            .get("decision")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        confidence: obj
            .get("confidence")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0),
        ts_ms: obj.get("ts_ms").and_then(|v| v.as_i64()).unwrap_or(0),
    })
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn balanced_corpus(n_pairs: usize) -> CalibrationCorpus {
        let mut c = CalibrationCorpus::new(CorpusSource::HeldOut);
        for i in 0..n_pairs {
            // True claim with high confidence.
            c.append(CalibrationTuple::new(
                format!("t-{i}"),
                format!("c-{i}"),
                "true claim",
                true,
                0.9,
                1_700_000_000 + (i as i64),
            ))
            .expect("append");
            // False claim with low confidence.
            c.append(CalibrationTuple::new(
                format!("f-{i}"),
                format!("d-{i}"),
                "false claim",
                false,
                0.2,
                1_700_000_500 + (i as i64),
            ))
            .expect("append");
        }
        c
    }

    #[test]
    fn calibration_tuple_clamps_score_and_truncates_text() {
        let long: String = "x".repeat(MAX_CLAIM_TEXT_CHARS + 50);
        let t = CalibrationTuple::new("t", "c", &long, true, 1.5, 1_700_000_000);
        assert_eq!(t.predicted_score, 1.0);
        assert_eq!(t.claim_text.chars().count(), MAX_CLAIM_TEXT_CHARS);

        let t2 = CalibrationTuple::new("t2", "c2", "ok", false, f64::NAN, 0);
        assert_eq!(t2.predicted_score, 0.0);
    }

    #[test]
    fn compute_tau_rejects_small_corpus() {
        let c = balanced_corpus(5); // 10 tuples; below the 20-floor.
        let err = compute_tau(&c, DEFAULT_EPSILON).expect_err("must fail");
        assert!(matches!(err, ConformalError::InsufficientCorpus { actual: 10, required: 20 }));
    }

    #[test]
    fn compute_tau_rejects_bad_epsilon() {
        let c = balanced_corpus(20);
        for bad in [-0.1, 0.0, 1.0, 1.5, f64::NAN, f64::INFINITY] {
            let err = compute_tau(&c, bad).expect_err("must fail");
            assert!(matches!(err, ConformalError::BadEpsilon { .. }));
        }
    }

    #[test]
    fn compute_tau_yields_threshold_above_high_confidence_floor() {
        // 20 pairs (40 tuples): 20 with predicted_score=0.9 (s=0.1),
        // 20 with predicted_score=0.2 (s=0.8). At epsilon=0.1 tau is the
        // (1 - 0.1) * (n+1) = 36.9 -> idx 36 -> the 37th sorted score,
        // which is in the high-non-conformity tail.
        let c = balanced_corpus(20);
        let tau = compute_tau(&c, 0.1).expect("tau");
        // The 40 sorted scores are [0.1; 20, 0.8; 20]. Index 36 is 0.8.
        assert!((tau - 0.8).abs() < 1e-9, "tau = {tau}");
    }

    #[test]
    fn decide_low_score_abstains() {
        let c = balanced_corpus(20);
        let tau = compute_tau(&c, 0.1).expect("tau");
        let d = decide(0.05, tau); // nonconformity 0.95 > tau 0.8
        assert!(matches!(d, RefusalDecision::Abstain));
    }

    #[test]
    fn decide_high_score_answers() {
        let c = balanced_corpus(20);
        let tau = compute_tau(&c, 0.1).expect("tau");
        let d = decide(0.99, tau);
        match d {
            RefusalDecision::Answer { confidence } => {
                assert!((confidence - 0.99).abs() < 1e-9);
            }
            RefusalDecision::Abstain => panic!("expected Answer; got Abstain"),
        }
    }

    #[test]
    fn decide_clamps_non_finite_correctly() {
        // Per UniCR §3.2: out-of-range scores clamp into [0, 1] before
        // the conformal predicate fires; NaN is fail-closed.
        let tau = 0.5;
        // NaN -> Abstain (fail-closed; no orderable clamp target).
        assert!(matches!(decide(f64::NAN, tau), RefusalDecision::Abstain));
        // +INFINITY clamps to 1.0 -> nonconformity 0.0 <= tau -> Answer.
        match decide(f64::INFINITY, tau) {
            RefusalDecision::Answer { confidence } => {
                assert!(
                    (confidence - 1.0).abs() < 1e-9,
                    "INFINITY should clamp to 1.0; got {confidence}"
                );
            }
            RefusalDecision::Abstain => {
                panic!("+INFINITY must clamp to 1.0 and Answer, not Abstain");
            }
        }
        // -INFINITY clamps to 0.0 -> nonconformity 1.0 > tau (0.5) -> Abstain.
        assert!(matches!(
            decide(f64::NEG_INFINITY, tau),
            RefusalDecision::Abstain
        ));
    }

    #[test]
    fn corpus_freeze_blocks_append() {
        let mut c = balanced_corpus(2);
        c.freeze();
        let err = c
            .append(CalibrationTuple::new("x", "y", "z", true, 0.5, 0))
            .expect_err("must fail");
        assert!(matches!(err, ConformalError::CorpusFrozen));
    }

    #[tokio::test]
    async fn compute_tau_and_emit_emits_one_row() {
        let c = balanced_corpus(20);
        let emitter = VecConformalTauComputedEmitter::new();
        let tau = compute_tau_and_emit(&c, 0.1, &emitter, 1_700_000_999_000)
            .await
            .expect("tau");
        assert!((tau - 0.8).abs() < 1e-9);
        let rows = emitter.rows().await;
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].epsilon, 0.1);
        assert_eq!(rows[0].corpus_size, 40);
        assert_eq!(rows[0].source, "held_out");
        assert_eq!(rows[0].ts_ms, 1_700_000_999_000);
    }

    #[tokio::test]
    async fn decide_and_emit_emits_one_row_per_call() {
        let emitter = VecConformalRefusalDecidedEmitter::new();
        let _ = decide_and_emit("c-low", 0.01, 0.5, &emitter, 100).await;
        let _ = decide_and_emit("c-high", 0.99, 0.5, &emitter, 200).await;
        let rows = emitter.rows().await;
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].claim_id, "c-low");
        assert_eq!(rows[0].decision, "abstain");
        assert_eq!(rows[1].claim_id, "c-high");
        assert_eq!(rows[1].decision, "answer");
        assert!((rows[1].confidence - 0.99).abs() < 1e-9);
    }

    #[test]
    fn load_seed_jsonl_round_trip() {
        // Write a tiny JSONL to a tempdir, then load + freeze + size.
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("seed.jsonl");
        let raw = "\
{\"tuple_id\":\"a\",\"claim_id\":\"c-a\",\"claim_text\":\"x\",\"ground_truth\":true,\"predicted_score\":0.9,\"ts_ms\":1}
{\"tuple_id\":\"b\",\"claim_id\":\"c-b\",\"claim_text\":\"y\",\"ground_truth\":false,\"predicted_score\":0.1,\"ts_ms\":2}

{\"tuple_id\":\"c\",\"claim_id\":\"c-c\",\"claim_text\":\"z\",\"ground_truth\":true,\"predicted_score\":0.85,\"ts_ms\":3}
";
        std::fs::write(&path, raw).expect("write");
        let c = CalibrationCorpus::load_seed_jsonl(&path).expect("load");
        assert_eq!(c.size(), 3);
        assert!(c.frozen);
        assert_eq!(c.source, CorpusSource::Synthetic);
    }
}

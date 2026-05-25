//! D-30 (SCHEMA_VERSION 147) — Calibration corpus bootstrap warm-start.
//!
//! UniCR (D-16) requires ~500 calibration tuples to be statistically useful
//! per arXiv:2509.01455 §3.4 finite-sample bound. Early sessions cannot
//! calibrate locally — `crate::held_out::HeldOutStore` will not have grown
//! to N=500 records yet. This module ships the WARM-START path:
//!
//!   1. `load_corpus` reads a published calibration corpus JSONL fixture
//!      (TruthfulQA subset, arXiv:2109.07958, projected to
//!      `crate::conformal_calibration::CalibrationTuple` rows tagged
//!      [`CorpusSource::Synthetic`]).
//!   2. `warm_start_tau` computes the UniCR tau on the imported corpus and
//!      atomically persists it to `tau_path` (mirror of
//!      [`TauUpdater::update_tau`] tmp+rename idiom), but with a LOWERED
//!      `min_corpus_size` (default 20) so the warm-start can fire on a
//!      ~50-row published-set subset. The resulting tau is labelled
//!      `tau_bootstrap` and is intended to be consumed only while the local
//!      `crate::held_out::HeldOutStore` size is below the production floor
//!      `DEFAULT_MIN_CORPUS_SIZE_PRODUCTION` (500).
//!
//! SECONDARY (block-bootstrap CI on tau, per Politis-Romano 1994 stationary
//! bootstrap) is shipped here as a pure-compute helper [`bootstrap_resample`].
//! The current warm-start invocation does NOT call it (the SSB payload
//! carries `None` for the CI fields); a follow-up RUNPOD-PENDING pass can
//! enable it by reading `ELAI_CALIBRATION_BOOTSTRAP_REPLICATES > 0`.
//!
//! Paper labels (per `workflow/reasoning/D-30-decisions.md`):
//!   * arXiv:2509.01455 UniCR — COMPOSITION (we reuse `compute_tau_with_min`).
//!   * arXiv:2109.07958 TruthfulQA — DATA ORIGIN (the corpus the fixture
//!     subset is derived from; not algorithm-source).
//!   * Politis & Romano 1994 JASA Vol 89 §3 — DIRECT GROUNDING for the
//!     stationary bootstrap mechanism in [`bootstrap_resample`] (only
//!     active when SECONDARY runs).
//!
//! Wire-dark by default. Production callers must observe
//! [`ELAI_CALIBRATION_BOOTSTRAP_ENABLE`] before invoking [`warm_start_tau`]
//! (see `crate::runtime_config::CalibrationBootstrapMode`).

use std::path::{Path, PathBuf};

use blake3::Hasher;
use elai_core::entropy::EntropySource;
use serde::{Deserialize, Serialize};

use crate::conformal_calibration::{
    compute_tau_with_min, CalibrationCorpus, CalibrationTuple, ConformalError, CorpusSource,
    DEFAULT_EPSILON,
};

// ---------------------------------------------------------------------------
// Defaults (decisions.md DECISION-2 + DECISION-3 + DECISION-4 anchored)
// ---------------------------------------------------------------------------

/// Default UniCR `epsilon` for the warm-start path. Matches
/// [`crate::conformal_calibration::DEFAULT_EPSILON`].
pub const DEFAULT_BOOTSTRAP_EPSILON: f64 = DEFAULT_EPSILON;

/// Lowered `min_corpus_size` floor used by the bootstrap warm-start.
///
/// `crate::conformal_calibration::DEFAULT_MIN_CORPUS_SIZE` (20) is the MVP
/// floor and matches D-16 unit-test corpora. The bootstrap path is intended
/// for warm-start scenarios where 20-50 imported rows are typical. Anything
/// smaller than 20 fails fast with
/// [`ConformalError::InsufficientCorpus`].
pub const DEFAULT_BOOTSTRAP_MIN_CORPUS_SIZE: usize = 20;

/// Production gate above which warm-start tau MUST be superseded by the
/// local D-6 [`crate::held_out::HeldOutStore`]-derived corpus. D-16
/// `DEFAULT_MIN_CORPUS_SIZE_PRODUCTION` is the canonical anchor for this
/// number; we duplicate the constant here only so callers can express the
/// "is warm-start still valid?" check without importing two crates.
pub const PRODUCTION_FLOOR_TUPLES: usize = 500;

/// Politis-Romano 1994 default block parameter for the stationary
/// bootstrap. `p = 1/sqrt(N)` gives `E[L] = sqrt(N)` (paper §3).
#[must_use]
pub fn default_block_p(n: usize) -> f64 {
    if n == 0 {
        return 1.0;
    }
    // 1 / sqrt(N). Floor avoids degenerate p = 0 case for absurdly large N.
    let sqrt_n = (n as f64).sqrt();
    (1.0 / sqrt_n).clamp(f64::MIN_POSITIVE, 1.0)
}

/// Politis-Romano 1994 default replicate count. Paper §3 recommends
/// `B = 999` or `1000`; we settle on `1000` for round-number tail-quantile
/// stability. Lower cap `200` (interface invariant) and upper cap `10_000`
/// enforced in [`bootstrap_resample`].
pub const DEFAULT_BOOTSTRAP_REPLICATES: u32 = 1000;

/// Minimum replicates allowed by [`bootstrap_resample`]. Below this the
/// percentile-CI tails become unstable.
pub const MIN_BOOTSTRAP_REPLICATES: u32 = 200;

/// Maximum replicates allowed by [`bootstrap_resample`]. Caps wall-clock at
/// `O(B * n)` per the Politis-Romano cost model.
pub const MAX_BOOTSTRAP_REPLICATES: u32 = 10_000;

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// Failure modes specific to the bootstrap warm-start path. Wraps
/// [`ConformalError`] for compute / IO failures so the caller can match
/// uniformly.
#[derive(Debug, thiserror::Error)]
pub enum BootstrapError {
    /// Underlying UniCR / corpus load / persistence failure.
    #[error("conformal failure: {0}")]
    Conformal(#[from] ConformalError),

    /// Tau persistence failed (atomic tmp+rename). `path` is the durable
    /// path that the orchestrator failed to write.
    #[error("could not persist warm-start tau to {path}: {reason}")]
    TauPersist {
        /// Target path that failed.
        path: String,
        /// IO error surface (rendered as `Display`).
        reason: String,
    },

    /// `bootstrap_resample` was invoked with an invalid block parameter
    /// (must be strictly in `(0, 1)`).
    #[error("block parameter p={p} out of range; required 0 < p < 1")]
    InvalidBlockP { p: f64 },

    /// `bootstrap_resample` was invoked with replicate count outside the
    /// `[MIN_BOOTSTRAP_REPLICATES, MAX_BOOTSTRAP_REPLICATES]` window.
    #[error("replicate count B={b} out of range [{min}, {max}]")]
    InvalidReplicateCount { b: u32, min: u32, max: u32 },
}

// ---------------------------------------------------------------------------
// Bootstrap corpus envelope
// ---------------------------------------------------------------------------

/// One row in the published calibration corpus fixture as shipped under
/// `.elai_cc/benchmarks/calibration_bootstrap/fixtures/`. The fixture
/// stores rows in the canonical projected shape so [`load_corpus`] does
/// nothing more than read + parse + tag.
///
/// The published-set schema (TruthfulQA / HaluEval / MMLU) differs across
/// corpora; the conversion shim is documented in
/// `workflow/reasoning/D-30-decisions.md` DECISION-5. The fixture itself
/// is already projected — this struct is the wire shape on disk.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PublishedCalibrationRow {
    /// Stable per-row id within the published corpus. Format is
    /// `<corpus_slug>-<row_index_in_corpus>` (e.g. `truthfulqa-0042`).
    pub tuple_id: String,
    /// Originating claim id within the published corpus (`claim-<n>`).
    pub claim_id: String,
    /// Raw claim text from the published corpus (truncated to
    /// [`crate::conformal_calibration::MAX_CLAIM_TEXT_CHARS`] by
    /// [`CalibrationTuple::new`]).
    pub claim_text: String,
    /// Post-hoc verdict — `true` if the claim is labelled correct in the
    /// published corpus (MC1 best-answer correct for TruthfulQA;
    /// non-hallucinated for HaluEval). The mapping is documented in
    /// `selection.md` per corpus.
    pub ground_truth: bool,
    /// The published corpus's confidence proxy projected to `[0.0, 1.0]`.
    /// Clamped by [`CalibrationTuple::new`] at load time.
    pub predicted_score: f64,
    /// UNIX ms at fixture-author time (NOT live; deterministic across CI
    /// runs).
    pub ts_ms: i64,
}

impl From<PublishedCalibrationRow> for CalibrationTuple {
    fn from(row: PublishedCalibrationRow) -> Self {
        CalibrationTuple::new(
            row.tuple_id,
            row.claim_id,
            row.claim_text,
            row.ground_truth,
            row.predicted_score,
            row.ts_ms,
        )
    }
}

/// Loaded bootstrap-corpus envelope. Holds the imported tuples and the
/// metadata the SSB beacon will surface (corpus slug, content hash, size).
///
/// Construction: [`load_corpus`].
#[derive(Debug, Clone, PartialEq)]
pub struct CalibrationBootstrapCorpus {
    /// Slug identifying the published corpus (`truthfulqa`, `halueval`,
    /// `mmlu_calibration_subset`, ...). Matches the fixture sub-directory
    /// name under `.elai_cc/benchmarks/calibration_bootstrap/fixtures/`.
    pub corpus_slug: String,
    /// `blake3` hex of the canonical fixture file bytes. Pins the SSB row
    /// to a specific fixture revision (mirrors D-25 `corpus_blake3`).
    pub corpus_blake3: String,
    /// Tuples projected from the published corpus. Tagged
    /// [`CorpusSource::Synthetic`] when wrapped into a
    /// [`CalibrationCorpus`] for [`warm_start_tau`].
    pub tuples: Vec<CalibrationTuple>,
}

impl CalibrationBootstrapCorpus {
    /// Number of imported tuples.
    #[must_use]
    pub fn size(&self) -> usize {
        self.tuples.len()
    }

    /// Project into a frozen [`CalibrationCorpus`] suitable for
    /// [`compute_tau_with_min`]. Always tags the resulting corpus
    /// [`CorpusSource::Synthetic`] — the published-set fixture is treated
    /// as a synthetic resample for the purposes of D-16's source enum (per
    /// integration-explorer line 11; no new enum arm).
    #[must_use]
    pub fn to_calibration_corpus(&self) -> CalibrationCorpus {
        let mut corpus = CalibrationCorpus::new(CorpusSource::Synthetic);
        // Bypass append's frozen check by direct field set — the
        // bootstrap corpus is frozen on construction by design (the
        // published-set fixture is never grown live).
        corpus.tuples = self.tuples.clone();
        corpus.freeze();
        corpus
    }
}

// ---------------------------------------------------------------------------
// Outcome of a warm-start run
// ---------------------------------------------------------------------------

/// Outcome record returned by [`warm_start_tau`]. The orchestrator
/// projects this into a `CalibrationBootstrapCompletedPayload` SSB row at
/// the dispatch site.
///
/// All `Option` fields are `None` for the canonical warm-start path; they
/// fill in when [`bootstrap_resample`] is composed on top per SECONDARY
/// (RUNPOD-PENDING activation).
#[derive(Debug, Clone, PartialEq)]
pub struct BootstrapWarmStartOutcome {
    /// Slug of the published corpus the warm-start ran against.
    pub corpus_slug: String,
    /// `blake3` hex of the fixture bytes (same value as
    /// [`CalibrationBootstrapCorpus::corpus_blake3`]).
    pub corpus_blake3: String,
    /// Number of rows imported from the fixture.
    pub corpus_size: u32,
    /// Warm-start tau value computed and persisted.
    pub tau_warm_start: f64,
    /// UniCR epsilon used (mirrors
    /// [`DEFAULT_BOOTSTRAP_EPSILON`] unless caller overrides).
    pub alpha: f64,
    /// Whether the D-16 UniCR floor was active. `false` for warm-start
    /// (we explicitly lower the floor via
    /// [`DEFAULT_BOOTSTRAP_MIN_CORPUS_SIZE`]); `true` if the caller passed
    /// `min_corpus_size >= 500`.
    pub tau_floor_active: bool,
    /// `Some(B)` when SECONDARY block-bootstrap ran; `None` for the
    /// warm-start-only path.
    pub bootstrap_replicates: Option<u32>,
    /// 5th percentile of resampled tau values (90% CI low). `None` if
    /// SECONDARY did not run.
    pub bootstrap_ci_low: Option<f64>,
    /// 95th percentile of resampled tau values (90% CI high). `None` if
    /// SECONDARY did not run.
    pub bootstrap_ci_high: Option<f64>,
    /// Politis-Romano geometric block parameter `p = 1/sqrt(N)`. `None`
    /// if SECONDARY did not run.
    pub block_length_p: Option<f64>,
    /// UNIX ms at outcome assembly time. The orchestrator MUST inject a
    /// `ClockProvider`-sourced value here (NEVER `SystemTime`); the
    /// in-crate helpers pass `0` so callers cannot accidentally smuggle
    /// wall-clock through.
    pub ts_ms: i64,
}

// ---------------------------------------------------------------------------
// Free functions (the public surface)
// ---------------------------------------------------------------------------

/// Load a published calibration corpus JSONL fixture from disk.
///
/// `corpus_slug` is the slug recorded in the outcome (`truthfulqa`,
/// `halueval`, etc.). `fixture_path` is the JSONL file containing
/// [`PublishedCalibrationRow`] rows (one per line, blank lines skipped).
///
/// Errors:
///   * [`ConformalError::CorpusLoad`] if the fixture is unreadable or any
///     line fails JSON parse.
///
/// The corpus is loaded eagerly (the fixture is sized for warm-start —
/// 20-50 rows typical, 817-row TruthfulQA full-set upper bound).
pub fn load_corpus(
    corpus_slug: impl Into<String>,
    fixture_path: &Path,
) -> Result<CalibrationBootstrapCorpus, BootstrapError> {
    let slug = corpus_slug.into();
    let raw = std::fs::read_to_string(fixture_path).map_err(|e| {
        BootstrapError::Conformal(ConformalError::CorpusLoad {
            path: fixture_path.display().to_string(),
            reason: e.to_string(),
        })
    })?;

    let mut tuples = Vec::new();
    for (i, line) in raw.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let row: PublishedCalibrationRow = serde_json::from_str(trimmed).map_err(|e| {
            BootstrapError::Conformal(ConformalError::CorpusLoad {
                path: fixture_path.display().to_string(),
                reason: format!("line {}: {}", i + 1, e),
            })
        })?;
        tuples.push(CalibrationTuple::from(row));
    }

    // Content hash over the raw fixture bytes — pin the SSB row to this
    // exact revision. blake3 is already in the workspace; no new dep.
    let mut hasher = Hasher::new();
    hasher.update(raw.as_bytes());
    let corpus_blake3 = hasher.finalize().to_hex().to_string();

    Ok(CalibrationBootstrapCorpus {
        corpus_slug: slug,
        corpus_blake3,
        tuples,
    })
}

/// Compute UniCR tau on a bootstrap corpus and atomically persist it.
///
/// Wraps [`compute_tau_with_min`] with a lowered floor
/// ([`DEFAULT_BOOTSTRAP_MIN_CORPUS_SIZE`]) so a 20-50 row published-set
/// fixture is acceptable. The atomic tmp+rename idiom mirrors
/// [`crate::conformal_calibration::TauUpdater::update_tau`].
///
/// `ts_ms` MUST be sourced from a `ClockProvider`; passing `0` is fine for
/// tests + the contract suite (the SSB emit site stamps the row).
///
/// Errors:
///   * [`ConformalError::InsufficientCorpus`] if `corpus.size() <
///     min_corpus_size` after the override.
///   * [`BootstrapError::TauPersist`] if the atomic rename fails.
pub fn warm_start_tau(
    corpus: &CalibrationBootstrapCorpus,
    tau_path: &Path,
    alpha: f64,
    min_corpus_size: usize,
    ts_ms: i64,
) -> Result<BootstrapWarmStartOutcome, BootstrapError> {
    let cal = corpus.to_calibration_corpus();
    let tau = compute_tau_with_min(&cal, alpha, min_corpus_size)?;

    // Atomic write: tmp file in the same parent directory, then rename.
    // The same idiom is used by TauUpdater::update_tau (D-16).
    if let Some(parent) = tau_path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).map_err(|e| BootstrapError::TauPersist {
                path: tau_path.display().to_string(),
                reason: format!("create_dir_all({}): {}", parent.display(), e),
            })?;
        }
    }
    let tmp_path: PathBuf = {
        let mut p = tau_path.to_path_buf();
        let new_ext = match tau_path.extension() {
            Some(ext) => format!("{}.tmp", ext.to_string_lossy()),
            None => "tmp".to_string(),
        };
        p.set_extension(new_ext);
        p
    };
    let contents = format!("{tau}\n");
    std::fs::write(&tmp_path, contents.as_bytes()).map_err(|e| BootstrapError::TauPersist {
        path: tmp_path.display().to_string(),
        reason: e.to_string(),
    })?;
    std::fs::rename(&tmp_path, tau_path).map_err(|e| BootstrapError::TauPersist {
        path: tau_path.display().to_string(),
        reason: format!("rename from {}: {}", tmp_path.display(), e),
    })?;

    Ok(BootstrapWarmStartOutcome {
        corpus_slug: corpus.corpus_slug.clone(),
        corpus_blake3: corpus.corpus_blake3.clone(),
        corpus_size: corpus.size() as u32,
        tau_warm_start: tau,
        alpha,
        tau_floor_active: min_corpus_size >= PRODUCTION_FLOOR_TUPLES,
        bootstrap_replicates: None,
        bootstrap_ci_low: None,
        bootstrap_ci_high: None,
        block_length_p: None,
        ts_ms,
    })
}

/// Stationary block-bootstrap resample of the corpus's tau distribution
/// (Politis & Romano 1994, JASA Vol 89, §3).
///
/// Produces `B` resampled corpora of length `N = corpus.size()`. For each
/// resample we call [`compute_tau_with_min`] (UniCR Eq.5) and collect the
/// tau replicate. The output is the sorted vector of tau replicates so the
/// caller can derive any percentile (90% CI = `[0.05, 0.95]` quantiles,
/// 95% CI = `[0.025, 0.975]`).
///
/// `rng` is an [`EntropySource`] (use `elai_core::entropy::SeededRng` for
/// determinism in tests).
///
/// Mechanism per replicate (paper §3):
///   1. Draw start index `i` uniformly from `[0, n)`.
///   2. Draw block length `L ~ Geometric(p)` (P(L=k) = p (1-p)^{k-1}).
///   3. Copy `corpus[i], corpus[(i+1) mod n], ..., corpus[(i+L-1) mod n]`
///      into the replicate (wrap-around).
///   4. Repeat from step 1 until the replicate has `n` rows (truncate the
///      final block if it overshoots).
///
/// Errors:
///   * [`BootstrapError::InvalidBlockP`] if `p` is not in `(0, 1)`.
///   * [`BootstrapError::InvalidReplicateCount`] if `replicates` is
///     outside `[MIN_BOOTSTRAP_REPLICATES, MAX_BOOTSTRAP_REPLICATES]`.
///   * [`ConformalError::InsufficientCorpus`] propagated from
///     [`compute_tau_with_min`] when `corpus.size() < min_corpus_size`.
///
/// Returns the sorted `Vec<f64>` of length `replicates as usize` (smallest
/// tau first). Sorted output keeps the percentile derivation trivially
/// `O(1)` for the caller and matches the Politis-Romano §3 percentile-CI
/// convention.
pub fn bootstrap_resample(
    corpus: &CalibrationBootstrapCorpus,
    alpha: f64,
    min_corpus_size: usize,
    replicates: u32,
    block_p: f64,
    rng: &mut dyn EntropySource,
) -> Result<Vec<f64>, BootstrapError> {
    if !(block_p > 0.0 && block_p < 1.0) {
        return Err(BootstrapError::InvalidBlockP { p: block_p });
    }
    if !(MIN_BOOTSTRAP_REPLICATES..=MAX_BOOTSTRAP_REPLICATES).contains(&replicates) {
        return Err(BootstrapError::InvalidReplicateCount {
            b: replicates,
            min: MIN_BOOTSTRAP_REPLICATES,
            max: MAX_BOOTSTRAP_REPLICATES,
        });
    }
    let n = corpus.tuples.len();
    if n == 0 {
        return Err(BootstrapError::Conformal(
            ConformalError::InsufficientCorpus {
                actual: 0,
                required: min_corpus_size,
            },
        ));
    }

    let mut tau_replicates: Vec<f64> = Vec::with_capacity(replicates as usize);
    for _ in 0..replicates {
        let replicate = build_replicate(&corpus.tuples, n, block_p, rng);
        let mut cal = CalibrationCorpus::new(CorpusSource::Synthetic);
        cal.tuples = replicate;
        cal.freeze();
        let tau = compute_tau_with_min(&cal, alpha, min_corpus_size)?;
        tau_replicates.push(tau);
    }
    tau_replicates.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    Ok(tau_replicates)
}

/// Build ONE stationary-bootstrap replicate of length `n` from `source`.
///
/// Paper §3 algorithm — geometric block length, wrap-around index, with
/// replacement. Stateless helper; pure function of the RNG sequence.
fn build_replicate(
    source: &[CalibrationTuple],
    n: usize,
    block_p: f64,
    rng: &mut dyn EntropySource,
) -> Vec<CalibrationTuple> {
    let mut out: Vec<CalibrationTuple> = Vec::with_capacity(n);
    while out.len() < n {
        let start = (rng.next_u64() as usize) % n;
        let block_len = sample_geometric(block_p, rng);
        for k in 0..block_len {
            if out.len() >= n {
                break;
            }
            let idx = (start + k) % n;
            out.push(source[idx].clone());
        }
    }
    out
}

/// Draw `L ~ Geometric(p)` with `P(L = k) = p * (1 - p)^{k - 1}` for
/// `k = 1, 2, ...`. Uses inverse-CDF via a uniform `(0, 1]` sample.
fn sample_geometric(p: f64, rng: &mut dyn EntropySource) -> usize {
    // Uniform in [0, 1). Reject 0.0 so log() is finite.
    let raw = rng.next_u64();
    // Convert to a uniform in (0, 1] via the canonical (raw + 1) / 2^64
    // recipe; this guarantees the value is strictly positive.
    let u = ((raw as f64) + 1.0) / (1.0 + u64::MAX as f64);
    // Inverse CDF: L = ceil( ln(1 - u) / ln(1 - p) ). Clamp to >= 1.
    let denom = (1.0 - p).ln();
    if denom >= 0.0 || !denom.is_finite() {
        return 1;
    }
    let l_raw = ((1.0 - u).ln() / denom).ceil();
    if l_raw.is_finite() && l_raw >= 1.0 {
        l_raw as usize
    } else {
        1
    }
}

/// Compute the 90% percentile CI `(lo, hi)` from a sorted vector of
/// resampled tau values produced by [`bootstrap_resample`]. Returns
/// `(taus[floor(0.05 * B)], taus[ceil(0.95 * B) - 1])` with clamps to
/// keep indices in range.
///
/// Returns `None` if `sorted_taus` is empty.
#[must_use]
pub fn ci_90_from_sorted(sorted_taus: &[f64]) -> Option<(f64, f64)> {
    if sorted_taus.is_empty() {
        return None;
    }
    let b = sorted_taus.len();
    let lo_idx = ((b as f64) * 0.05).floor() as usize;
    let hi_idx = (((b as f64) * 0.95).ceil() as usize).saturating_sub(1);
    let lo_idx = lo_idx.min(b - 1);
    let hi_idx = hi_idx.min(b - 1);
    Some((sorted_taus[lo_idx], sorted_taus[hi_idx]))
}

// ---------------------------------------------------------------------------
// Unit tests (in-module, deterministic, no IO outside tempfile)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use elai_core::entropy::SeededRng;
    use tempfile::TempDir;

    fn make_row(idx: usize, gt: bool, score: f64) -> PublishedCalibrationRow {
        PublishedCalibrationRow {
            tuple_id: format!("test-{idx:04}"),
            claim_id: format!("claim-{idx}"),
            claim_text: format!("synthetic test claim {idx}"),
            ground_truth: gt,
            predicted_score: score,
            ts_ms: 1_700_000_000_000 + idx as i64,
        }
    }

    fn make_corpus(n: usize) -> CalibrationBootstrapCorpus {
        let tuples: Vec<CalibrationTuple> = (0..n)
            .map(|i| {
                let gt = i % 2 == 0;
                let score = if gt { 0.9 } else { 0.4 };
                CalibrationTuple::from(make_row(i, gt, score))
            })
            .collect();
        CalibrationBootstrapCorpus {
            corpus_slug: "synthetic".into(),
            corpus_blake3: "0".repeat(64),
            tuples,
        }
    }

    #[test]
    fn load_corpus_reads_jsonl_and_hashes() {
        let dir = TempDir::new().unwrap();
        let fixture = dir.path().join("fixture.jsonl");
        let rows: Vec<String> = (0..25)
            .map(|i| serde_json::to_string(&make_row(i, i % 2 == 0, 0.85)).unwrap())
            .collect();
        std::fs::write(&fixture, rows.join("\n")).unwrap();
        let corpus = load_corpus("truthfulqa", &fixture).unwrap();
        assert_eq!(corpus.size(), 25);
        assert_eq!(corpus.corpus_slug, "truthfulqa");
        assert_eq!(corpus.corpus_blake3.len(), 64);
    }

    #[test]
    fn warm_start_atomic_writes_tau_and_returns_outcome() {
        let dir = TempDir::new().unwrap();
        let tau_path = dir.path().join("tau_bootstrap.txt");
        let corpus = make_corpus(30);
        let outcome = warm_start_tau(
            &corpus,
            &tau_path,
            DEFAULT_BOOTSTRAP_EPSILON,
            DEFAULT_BOOTSTRAP_MIN_CORPUS_SIZE,
            0,
        )
        .unwrap();
        assert_eq!(outcome.corpus_size, 30);
        assert!(outcome.tau_warm_start.is_finite());
        assert!(outcome.tau_warm_start >= 0.0 && outcome.tau_warm_start <= 1.0);
        assert!(!outcome.tau_floor_active);
        assert_eq!(outcome.bootstrap_replicates, None);
        let on_disk = std::fs::read_to_string(&tau_path).unwrap();
        let parsed: f64 = on_disk.trim().parse().unwrap();
        assert!((parsed - outcome.tau_warm_start).abs() < 1e-12);
    }

    #[test]
    fn warm_start_rejects_under_floor() {
        let dir = TempDir::new().unwrap();
        let tau_path = dir.path().join("tau_bootstrap.txt");
        let corpus = make_corpus(5);
        let err = warm_start_tau(
            &corpus,
            &tau_path,
            DEFAULT_BOOTSTRAP_EPSILON,
            DEFAULT_BOOTSTRAP_MIN_CORPUS_SIZE,
            0,
        )
        .expect_err("must fail under floor");
        match err {
            BootstrapError::Conformal(ConformalError::InsufficientCorpus { actual, required }) => {
                assert_eq!(actual, 5);
                assert_eq!(required, DEFAULT_BOOTSTRAP_MIN_CORPUS_SIZE);
            }
            other => panic!("unexpected error variant: {other:?}"),
        }
    }

    #[test]
    fn bootstrap_resample_seed_determinism() {
        let corpus = make_corpus(40);
        let p = default_block_p(corpus.size());
        let mut rng_a = SeededRng::new(424242);
        let mut rng_b = SeededRng::new(424242);
        let taus_a = bootstrap_resample(
            &corpus,
            DEFAULT_BOOTSTRAP_EPSILON,
            DEFAULT_BOOTSTRAP_MIN_CORPUS_SIZE,
            MIN_BOOTSTRAP_REPLICATES,
            p,
            &mut rng_a,
        )
        .unwrap();
        let taus_b = bootstrap_resample(
            &corpus,
            DEFAULT_BOOTSTRAP_EPSILON,
            DEFAULT_BOOTSTRAP_MIN_CORPUS_SIZE,
            MIN_BOOTSTRAP_REPLICATES,
            p,
            &mut rng_b,
        )
        .unwrap();
        assert_eq!(taus_a, taus_b);
        assert_eq!(taus_a.len(), MIN_BOOTSTRAP_REPLICATES as usize);
        // sorted ascending
        for w in taus_a.windows(2) {
            assert!(w[0] <= w[1]);
        }
    }

    #[test]
    fn bootstrap_resample_rejects_bad_block_p() {
        let corpus = make_corpus(40);
        let mut rng = SeededRng::new(1);
        let err = bootstrap_resample(
            &corpus,
            DEFAULT_BOOTSTRAP_EPSILON,
            DEFAULT_BOOTSTRAP_MIN_CORPUS_SIZE,
            MIN_BOOTSTRAP_REPLICATES,
            0.0,
            &mut rng,
        )
        .unwrap_err();
        assert!(matches!(err, BootstrapError::InvalidBlockP { .. }));
    }

    #[test]
    fn bootstrap_resample_rejects_bad_replicate_count() {
        let corpus = make_corpus(40);
        let mut rng = SeededRng::new(1);
        let err = bootstrap_resample(
            &corpus,
            DEFAULT_BOOTSTRAP_EPSILON,
            DEFAULT_BOOTSTRAP_MIN_CORPUS_SIZE,
            MIN_BOOTSTRAP_REPLICATES - 1,
            default_block_p(40),
            &mut rng,
        )
        .unwrap_err();
        assert!(matches!(err, BootstrapError::InvalidReplicateCount { .. }));
    }

    #[test]
    fn ci_90_from_sorted_returns_expected_indices() {
        let mut taus: Vec<f64> = (0..1000).map(|i| (i as f64) / 1000.0).collect();
        taus.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let (lo, hi) = ci_90_from_sorted(&taus).unwrap();
        // floor(0.05*1000)=50 -> 0.05; ceil(0.95*1000)-1 = 949 -> 0.949
        assert!((lo - 0.050).abs() < 1e-12);
        assert!((hi - 0.949).abs() < 1e-12);
    }

    #[test]
    fn default_block_p_matches_one_over_sqrt_n() {
        assert!((default_block_p(100) - 0.1).abs() < 1e-12);
        assert!((default_block_p(400) - 0.05).abs() < 1e-12);
        // degenerate input -> safe default
        assert_eq!(default_block_p(0), 1.0);
    }
}

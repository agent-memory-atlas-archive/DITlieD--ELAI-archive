//! I-23 — unified conformal calibration facade.
//!
//! Single import path for the ELAI calibration subsystem. Consumers
//! should `use crate::calibration::*` (or the specific names they need)
//! rather than reaching into `conformal_calibration` or
//! `calibration_bootstrap` directly. The underlying modules remain in
//! place; this file is a pure re-export layer with zero new math.
//!
//! Grounding: I-23 EXTENSION/UNIFICATION — IC-13 binding correction
//! (conformal calibration is SHIPPED; i-23 = structural unification,
//! not a new algorithm). See docs/thefinalglance/I-plan/
//! I-23_unified_conformal_calibration.md.

// ---------------------------------------------------------------------------
// I-8 calibration suite filter (WARN-1 remedy — fail-closed allow-list)
// ---------------------------------------------------------------------------

pub use crate::abstention_score::{
    is_suite_excluded_from_calibration,
    CALIBRATION_ALLOWED_SUITES,
    CALIBRATION_EXCLUDED_SUITES,
};

// ---------------------------------------------------------------------------
// conformal_calibration surface
// ---------------------------------------------------------------------------

pub use crate::conformal_calibration::{
    compute_tau,
    compute_tau_and_emit,
    compute_tau_with_min,
    decide,
    decide_and_emit,
    decide_and_emit_with_reason,
    mint_calibration_id,
    recent_refusal_rows,
    recent_tau_rows,
    tau_path_default,
    update_tau_and_emit,
    CalibrationCorpus,
    CalibrationCorpusUpdatedEmitter,
    CalibrationTuple,
    ConformalError,
    ConformalRefusalDecidedEmitter,
    ConformalRefusalRow,
    ConformalTauComputedEmitter,
    ConformalTauRow,
    CorpusSource,
    RefusalDecision,
    RefusalReason,
    TauUpdater,
    VecCalibrationCorpusUpdatedEmitter,
    VecConformalRefusalDecidedEmitter,
    VecConformalTauComputedEmitter,
    DEFAULT_EPSILON,
    DEFAULT_MIN_CORPUS_SIZE,
    MAX_CLAIM_TEXT_CHARS,
    TAU_DEFAULT_FALLBACK,
};

// ---------------------------------------------------------------------------
// calibration_bootstrap surface
// ---------------------------------------------------------------------------

pub use crate::calibration_bootstrap::{
    bootstrap_resample,
    ci_90_from_sorted,
    default_block_p,
    load_corpus,
    warm_start_tau,
    BootstrapError,
    BootstrapWarmStartOutcome,
    CalibrationBootstrapCorpus,
    PublishedCalibrationRow,
    DEFAULT_BOOTSTRAP_EPSILON,
    DEFAULT_BOOTSTRAP_MIN_CORPUS_SIZE,
    DEFAULT_BOOTSTRAP_REPLICATES,
    MAX_BOOTSTRAP_REPLICATES,
    MIN_BOOTSTRAP_REPLICATES,
    PRODUCTION_FLOOR_TUPLES,
};

// ---------------------------------------------------------------------------
// I-23 STEP 1 — typed facade interface
//
// Thin wrappers over the re-exported implementation. No new math, no new
// algorithm (IC-13 binding correction). These provide a clean, typed entry
// point so consumers do not need to pass loose arguments to the underlying
// free functions. The free functions remain available for callers that
// need the emitting (SSB) variants.
// ---------------------------------------------------------------------------

use std::path::Path;

/// Typed request for [`get_tau`]. Bundles the corpus reference, epsilon,
/// and optional minimum-corpus-size floor into a single value.
#[derive(Debug, Clone)]
pub struct CalibrationRequest<'a> {
    /// The calibration corpus to compute tau from.
    pub corpus: &'a CalibrationCorpus,
    /// UniCR epsilon (miscoverage rate). Must be in `(0.0, 1.0)`.
    pub epsilon: f64,
    /// Optional minimum corpus size floor. When `None`, uses
    /// [`DEFAULT_MIN_CORPUS_SIZE`].
    pub min_corpus_size: Option<usize>,
}

/// Typed result of [`get_tau`]. Carries the computed tau, the deterministic
/// `calibration_id` (blake3 hex prefix), the corpus size, and the epsilon
/// used — everything a consumer needs to thread the join key through to
/// [`decide_and_emit`] / [`decide_and_emit_with_reason`].
#[derive(Debug, Clone, PartialEq)]
pub struct TauResult {
    /// The computed conformal tau threshold.
    pub tau: f64,
    /// 16-char blake3 hex prefix — the journal join key.
    pub calibration_id: String,
    /// Corpus size at computation time (u32-saturated).
    pub corpus_size: u32,
    /// Epsilon used for this computation.
    pub epsilon: f64,
}

/// Typed input for [`update_tau`]. Bundles the [`TauUpdater`] reference and
/// the corpus-source label into a single value.
#[derive(Debug, Clone)]
pub struct ObservedOutcome<'a> {
    /// The TauUpdater that re-reads the corpus and persists tau.
    pub updater: &'a TauUpdater,
    /// Snake-case corpus-source label (mirrors [`CorpusSource::as_str`]).
    pub corpus_source: &'a str,
}

/// Typed request for [`warm_start`]. Bundles the bootstrap corpus, tau
/// path, alpha, minimum corpus size, and timestamp into a single value.
#[derive(Debug, Clone)]
pub struct WarmStartRequest<'a> {
    /// The published calibration corpus to warm-start from.
    pub corpus: &'a CalibrationBootstrapCorpus,
    /// Path to the on-disk tau scalar to write.
    pub tau_path: &'a Path,
    /// UniCR epsilon (alpha = miscoverage rate).
    pub alpha: f64,
    /// Minimum corpus size floor.
    pub min_corpus_size: usize,
    /// UNIX ms timestamp (entropy quarantine — caller injects).
    pub ts_ms: i64,
}

/// Compute tau via the typed facade interface.
///
/// Delegates to [`compute_tau`] (or [`compute_tau_with_min`] when
/// `req.min_corpus_size` is `Some`) and mints the deterministic
/// `calibration_id` via [`mint_calibration_id`].
///
/// Pure — no I/O, no SSB emission. For the emitting variant, use
/// [`compute_tau_and_emit`] directly.
pub fn get_tau(req: &CalibrationRequest<'_>) -> Result<TauResult, ConformalError> {
    let tau = match req.min_corpus_size {
        Some(min) => compute_tau_with_min(req.corpus, req.epsilon, min)?,
        None => compute_tau(req.corpus, req.epsilon)?,
    };
    let corpus_size = u32::try_from(req.corpus.size()).unwrap_or(u32::MAX);
    let calibration_id = mint_calibration_id(tau, corpus_size, req.epsilon);
    Ok(TauResult {
        tau,
        calibration_id,
        corpus_size,
        epsilon: req.epsilon,
    })
}

/// Update tau via the typed facade interface.
///
/// Delegates to [`TauUpdater::update_tau`] — re-reads the corpus, computes
/// tau, and persists the scalar atomically. For the emitting variant, use
/// [`update_tau_and_emit`] directly.
pub fn update_tau(outcome: &ObservedOutcome<'_>) -> Result<f64, ConformalError> {
    outcome.updater.update_tau()
}

/// Warm-start tau via the typed facade interface.
///
/// Delegates to [`warm_start_tau`] — computes tau from the published
/// corpus and persists it atomically. Returns the typed
/// [`BootstrapWarmStartOutcome`].
pub fn warm_start(req: &WarmStartRequest<'_>) -> Result<BootstrapWarmStartOutcome, BootstrapError> {
    warm_start_tau(
        req.corpus,
        req.tau_path,
        req.alpha,
        req.min_corpus_size,
        req.ts_ms,
    )
}

/// Zero-sized marker struct providing the typed facade interface as
/// associated functions. Consumers may use `CalibrationFacade::get_tau`
/// instead of the free function [`get_tau`] — both delegate to the same
/// underlying implementation.
#[derive(Debug, Clone, Copy)]
pub struct CalibrationFacade;

impl CalibrationFacade {
    /// Typed tau computation — see [`get_tau`].
    pub fn get_tau(req: &CalibrationRequest<'_>) -> Result<TauResult, ConformalError> {
        get_tau(req)
    }

    /// Typed tau update — see [`update_tau`].
    pub fn update_tau(outcome: &ObservedOutcome<'_>) -> Result<f64, ConformalError> {
        update_tau(outcome)
    }

    /// Typed warm-start — see [`warm_start`].
    pub fn warm_start(
        req: &WarmStartRequest<'_>,
    ) -> Result<BootstrapWarmStartOutcome, BootstrapError> {
        warm_start(req)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn balanced_corpus(n_pairs: usize) -> CalibrationCorpus {
        let mut c = CalibrationCorpus::new(CorpusSource::HeldOut);
        for i in 0..n_pairs {
            c.append(CalibrationTuple::new(
                format!("t-{i}"),
                format!("c-{i}"),
                "true claim",
                true,
                0.9,
                1_700_000_000 + (i as i64),
            ))
            .expect("append true tuple");
            c.append(CalibrationTuple::new(
                format!("f-{i}"),
                format!("d-{i}"),
                "false claim",
                false,
                0.2,
                1_700_000_500 + (i as i64),
            ))
            .expect("append false tuple");
        }
        c
    }

    #[test]
    fn get_tau_returns_calibrated_result_with_join_key() {
        let corpus = balanced_corpus(20);
        let req = CalibrationRequest {
            corpus: &corpus,
            epsilon: DEFAULT_EPSILON,
            min_corpus_size: None,
        };
        let result = get_tau(&req).expect("get_tau must succeed on 40-tuple corpus");
        assert!(
            result.tau >= 0.0 && result.tau <= 1.0,
            "tau must be in [0, 1]; got {}",
            result.tau
        );
        assert_eq!(
            result.calibration_id.len(),
            16,
            "calibration_id must be 16 hex chars; got {:?}",
            result.calibration_id
        );
        assert!(
            result
                .calibration_id
                .chars()
                .all(|c| c.is_ascii_hexdigit()),
            "calibration_id must be lowercase hex; got {:?}",
            result.calibration_id
        );
        assert_eq!(result.corpus_size, 40);
        assert_eq!(result.epsilon, DEFAULT_EPSILON);
    }

    #[test]
    fn get_tau_with_explicit_min_corpus_size() {
        let corpus = balanced_corpus(20);
        let req = CalibrationRequest {
            corpus: &corpus,
            epsilon: DEFAULT_EPSILON,
            min_corpus_size: Some(10),
        };
        let result = get_tau(&req).expect("get_tau with min=10 must succeed on 40-tuple corpus");
        assert!(result.tau >= 0.0 && result.tau <= 1.0);
    }

    #[test]
    fn get_tau_rejects_insufficient_corpus() {
        let corpus = balanced_corpus(5); // 10 tuples < DEFAULT_MIN_CORPUS_SIZE (20)
        let req = CalibrationRequest {
            corpus: &corpus,
            epsilon: DEFAULT_EPSILON,
            min_corpus_size: None,
        };
        let err = get_tau(&req).expect_err("must fail on insufficient corpus");
        assert!(matches!(
            err,
            ConformalError::InsufficientCorpus { actual: 10, required: 20 }
        ));
    }

    #[test]
    fn calibration_facade_delegates_to_free_functions() {
        let corpus = balanced_corpus(20);
        let req = CalibrationRequest {
            corpus: &corpus,
            epsilon: DEFAULT_EPSILON,
            min_corpus_size: None,
        };
        let via_free = get_tau(&req).expect("free get_tau");
        let via_facade = CalibrationFacade::get_tau(&req).expect("facade get_tau");
        assert_eq!(via_free, via_facade, "facade must delegate to free function");
    }

    #[test]
    fn get_tau_calibration_id_matches_mint_calibration_id() {
        let corpus = balanced_corpus(20);
        let req = CalibrationRequest {
            corpus: &corpus,
            epsilon: DEFAULT_EPSILON,
            min_corpus_size: None,
        };
        let result = get_tau(&req).expect("get_tau");
        let direct_id = mint_calibration_id(result.tau, result.corpus_size, result.epsilon);
        assert_eq!(
            result.calibration_id, direct_id,
            "get_tau calibration_id must match mint_calibration_id for same inputs"
        );
    }
}

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

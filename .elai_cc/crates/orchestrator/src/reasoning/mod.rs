//! E-19 orchestrator-side reasoning quality wrappers.
//!
//! Composes the `elai_core::reasoning` primitives (ConfidenceGate,
//! DifficultyEstimator, TypedCotTriple) into orchestrator-side dispatchers
//! that read FailureTaxonomy / MistakeBook and emit SSB rows.
//!
//! Wire-dark by default: every dispatch path short-circuits when
//! `ELAI_REASONING_QUALITY_PROBE_ENABLE` is unset. See
//! `workflow/reasoning/E-19-decisions.md`.

pub mod l3_direct;
pub mod l6_verifier_diversity;
pub mod ltot;
pub mod prm_lite;
pub mod research_profile;

pub use l3_direct::l3_direct_route;
pub use l6_verifier_diversity::{verifier_diversity, DiversityVerdict};
pub use ltot::{lateral_branch, LToTSplit, N_LATERAL, N_MAINLINE};
pub use prm_lite::{AuditorVerifierScore, AUDITOR_WEIGHT, COSINE_WEIGHT, MEAN_LOGPROB_WEIGHT};
pub use research_profile::{
    research_grounding_pass, ResearchGroundingOutcome, ResearchVerdict, AbstainReason,
    ResearchClaimVerdict,
};

/// Env var that opens the umbrella gate on every E-19 dispatch path. Default
/// off — wire-dark per E-26 / E-43 precedent. Consumed via
/// `std::env::var` with a `OnceLock` cache in each dispatcher to avoid
/// re-reading per call (E-3 RuntimeHandoffSignal precedent).
pub const REASONING_QUALITY_PROBE_ENV: &str = "ELAI_REASONING_QUALITY_PROBE_ENABLE";

/// True when the umbrella env gate is set to a truthy value. Reads the env
/// once per call (callers MAY cache via `OnceLock` if they're on a hot path).
#[must_use]
pub fn umbrella_enabled() -> bool {
    match std::env::var(REASONING_QUALITY_PROBE_ENV) {
        Ok(v) => matches!(v.trim(), "1" | "true" | "TRUE" | "yes" | "YES" | "on" | "ON"),
        Err(_) => false,
    }
}

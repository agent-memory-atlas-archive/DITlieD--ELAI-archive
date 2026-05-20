//! Error type for the E-5 memory substrate. Single typed error funnels every
//! fallible path (sqlite, serde, decay-config). No `Box<dyn Error>` — per
//! CLAUDE.md anti-stale-training rule.

use thiserror::Error;

/// Failure modes surfaced by [`crate::FactStore`], [`crate::extract_facts`],
/// [`crate::get_relevant_facts`], and [`crate::decay_tick`].
#[derive(Debug, Error)]
pub enum MemoryError {
    /// rusqlite returned an error (open / migrate / prepare / step).
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    /// `serde_json` failed to (de)serialize a `value` / `evidence_pointers`
    /// JSON column.
    #[error("json (de)serialization error: {0}")]
    Json(#[from] serde_json::Error),

    /// A row was rejected by validation (out-of-range confidence /
    /// trust_score / decay_rate, empty evidence_pointers, unknown fact_type
    /// / scope).
    #[error("invalid fact: {0}")]
    Invalid(String),

    /// The requested `fact_id` is not in the store. Surfaced by
    /// [`crate::FactStore::get`] / `supersede`.
    #[error("fact not found: {0}")]
    NotFound(String),
}

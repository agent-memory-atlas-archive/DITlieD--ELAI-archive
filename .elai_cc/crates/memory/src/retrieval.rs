//! Trust-aware retrieval. Filters the `memory_facts` table to the
//! currently-valid (`valid_to_ms IS NULL`) rows for a user (+ optional
//! project / fact-type predicates) and applies the per-fact-type
//! trust-score floor + provenance enforcement (empty
//! `evidence_pointers` -> excluded).
//!
//! On read, [`get_relevant_facts`] also bumps `last_observed_at_ms` for
//! every returned row (decay-feedback loop — frequent reads slow the
//! decay curve).

use crate::error::MemoryError;
use crate::fact::{FactType, MemoryFact, Scope};
use crate::store::FactStore;
/// Predicate bundle for [`get_relevant_facts`]. All fields are optional —
/// `None` means "no filter applied at that axis".
#[derive(Debug, Clone, Default)]
pub struct RetrievalFilter {
    /// Project filter: include rows with `scope='project' AND project_id=Some(p)`.
    /// Rows with `scope='global'` are always included regardless.
    pub project_id: Option<String>,
    /// Restrict to a single `fact_type` (e.g. `Preference`). When `None`
    /// every fact_type is candidate.
    pub fact_type: Option<FactType>,
    /// Override the per-fact-type trust threshold from
    /// [`FactType::retrieval_threshold`]. Useful for high-stakes-action
    /// gates (commit / force-push / delete).
    pub trust_override: Option<f32>,
    /// C-48 SafeHarbor: when true, only T0+T1 rows are returned. Mandatory
    /// for safety-critical query types (destructive_command_intent,
    /// merge_decision, auth_decision, credential_fetch).
    pub safety_critical: bool,
}

/// Trust-and-provenance-filtered top-K retrieval. Returns up to `k`
/// currently-valid facts for `user_id`, side-effect: updates
/// `last_observed_at_ms` for each returned row to `now_ms`.
///
/// Defense properties:
///   * Rows with `valid_to_ms != NULL` (superseded / forgotten) are
///     excluded by `FactStore::list_current`.
///   * Rows with empty `evidence_pointers` are filtered out per
///     arXiv:2512.16962 (MemoryGraft) — provenance is the structural
///     defense.
///   * Rows below per-fact-type trust threshold (or `trust_override`)
///     are filtered out per arXiv:2601.05504 (MINJA) — high-trust gating.
pub fn get_relevant_facts(
    store: &FactStore,
    user_id: &str,
    filter: &RetrievalFilter,
    k: usize,
    now_ms: u128,
) -> Result<Vec<MemoryFact>, MemoryError> {
    // Pull a generous candidate set then narrow in-Rust — the on-disk
    // index covers `(user_id, scope, fact_type)` only.
    let candidate_limit = k.saturating_mul(4).max(64);
    let candidates = store.list_current(user_id, None, filter.fact_type, candidate_limit)?;

    // C-48 SafeHarbor: when safety_critical=true, track excluded T2/T3 rows
    // and emit a tracing event per exclusion so callers with access to the SSB
    // bus can produce `safety_critical_blocked` rows. Silent exclusion is banned
    // per WIRING_CONTRACT stub_ban.
    let (candidates_allowed, safety_blocked): (Vec<_>, Vec<_>) = if filter.safety_critical {
        candidates
            .into_iter()
            .partition(|f| f.trust_tier.is_safety_critical_allowed())
    } else {
        (candidates, vec![])
    };

    for blocked in &safety_blocked {
        tracing::info!(
            target = "memory.safety_critical_blocked",
            fact_id = %blocked.fact_id,
            trust_tier = %blocked.trust_tier.as_str(),
            user_id = %user_id,
            "C-48: T2/T3 fact excluded from safety-critical query (safety_critical_blocked)"
        );
    }

    let mut filtered: Vec<MemoryFact> = candidates_allowed
        .into_iter()
        .filter(|f| match (f.scope, &filter.project_id) {
            // Globals always pass the scope filter.
            (Scope::Global, _) => true,
            // Project rows pass only if project_id matches (or no project
            // filter set — caller wants every project_id row).
            (Scope::Project, None) => true,
            (Scope::Project, Some(want)) => f.project_id.as_deref() == Some(want.as_str()),
        })
        .filter(|f| {
            let threshold = filter
                .trust_override
                .unwrap_or_else(|| f.fact_type.retrieval_threshold());
            f.trust_score >= threshold
        })
        // Provenance enforcement — empty evidence_pointers excluded.
        .filter(|f| !f.evidence_pointers.is_empty())
        .collect();

    // Most-recently-observed first (already approximately the SQL order;
    // restating in-Rust so the project-scope filter cannot reshuffle the
    // tail).
    filtered.sort_by(|a, b| b.last_observed_at_ms.cmp(&a.last_observed_at_ms));
    filtered.truncate(k);

    // Update last_observed_at_ms for returned rows. Errors here are
    // non-fatal to the caller — log + continue so a transient sqlite
    // hiccup does not deny the calling prompt-builder its context.
    for fact in &mut filtered {
        match store.update_trust(&fact.fact_id, fact.trust_score, now_ms) {
            Ok(()) => fact.last_observed_at_ms = now_ms,
            Err(e) => tracing::warn!(target = "memory.retrieval",
                fact_id = %fact.fact_id, err = %e,
                "failed to update last_observed_at_ms on retrieval (non-fatal)"),
        }
    }

    Ok(filtered)
}

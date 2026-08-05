//! J-21 Step 3 — provenance-linked raw-evidence tier + sufficiency-gated
//! escalation.
//!
//! EXTENDS H-31/H-32/E-5 (and J-21 Steps 1-2) with the third grounded gap:
//! the **write-before-query omission**. Extraction can drop the one detail a
//! later prompt needs even when every kept fact is valid. The fix keeps an
//! immutable raw-evidence layer beneath the extracted-fact layer and
//! escalates to it only when the fact layer is insufficient — deterministic,
//! minimal-LLM on the read path.
//!
//! # Architecture
//!
//! - [`store::RawEvidenceStore`] — an append-only, bounded, namespace-scoped
//!   in-memory store of immutable [`store::RawEvidenceRecord`]s. The bound
//!   addresses the plan's "raw-evidence growth" open question: when a
//!   namespace exceeds [`store::MAX_RECORDS_PER_NAMESPACE`], the oldest
//!   records are evicted (FIFO compaction). The store is append-only —
//!   records are never mutated or deleted individually; only
//!   namespace-wide compaction shrinks it.
//! - [`sufficiency::assess_sufficiency`] — a deterministic sufficiency gate.
//!   The fact layer is **insufficient** when zero authorized facts were
//!   returned AND at least one candidate was excluded (stale/quarantined/
//!   unknown). This means facts existed for the query but every one was
//!   rejected by the retrieval-time authorization gate — the exact case
//!   where the raw evidence tier may still carry the dropped detail.
//! - [`escalation::escalate_to_raw_evidence`] — queries the raw-evidence
//!   store by the evidence pointers of the excluded facts. Returns matching
//!   [`store::RawEvidenceRecord`]s. Only called when
//!   [`sufficiency::assess_sufficiency`] returns
//!   [`sufficiency::SufficiencyVerdict::Insufficient`].
//! - [`get_authorized_facts_with_escalation`] — the composed entry point
//!   that chains J-21 Step 2's [`crate::retrieval_authorization::get_authorized_facts`]
//!   → [`sufficiency::assess_sufficiency`] →
//!   [`escalation::escalate_to_raw_evidence`]. This is the public API the
//!   orchestrator retrieval path calls when the DARK capture+retrieval is
//!   un-darked.
//!
//! # Grounding
//!
//! - GEM arXiv:2605.26252 — "correctness is a property of the state
//!   trajectory, not of individual records". The raw-evidence tier is the
//!   immutable state-trajectory record beneath the extracted-claim layer;
//!   escalation to it is the state-level sufficiency check.
//! - MPBench arXiv:2606.04329 — "agents that write/retrieve more
//!   aggressively are more exploitable". The raw-evidence tier is
//!   provenance-linked (G-1 taint) and immutable, so a poisoned extracted
//!   fact cannot corrupt the raw evidence beneath it; escalation re-reads
//!   the original provenance, not the extracted abstraction.
//!
//! # Relationship to J-21 Steps 1-2
//!
//! This module composes [`crate::retrieval_authorization::get_authorized_facts`]
//! (Step 2) and does NOT amend it. The escalation is an additive layer: when
//! the authorized fact layer is insufficient, the raw-evidence tier is
//! queried as a fallback. The H-31 auditor-objectivity firewall is
//! maintained — raw evidence is still typed evidence, never injected into
//! auditor/critic/adversary contexts.

pub mod escalation;
pub mod store;
pub mod sufficiency;

pub use escalation::{escalate_to_raw_evidence, RawEvidenceEscalation};
pub use store::{RawEvidenceRecord, RawEvidenceStore, MAX_RECORDS_PER_NAMESPACE};
pub use sufficiency::{assess_sufficiency, SufficiencyReason, SufficiencyVerdict};

use crate::citation_validity::CitationValidator;
use crate::error::MemoryError;
use crate::fact::MemoryFact;
use crate::retrieval::RetrievalFilter;
use crate::retrieval_authorization::{
    get_authorized_facts, AuthorizationExclusion, QuarantineStore,
};
use crate::semantic_validation::{SemanticValidator, ValidationScope};
use crate::store::FactStore;

/// The composed retrieval entry point: J-21 Step 2's authorized retrieval
/// + J-21 Step 3's sufficiency-gated raw-evidence escalation.
///
/// - Stage 1: calls [`get_authorized_facts`] (Step 2) to get authorized facts
///   and exclusions.
/// - Stage 2: calls [`assess_sufficiency`] on the authorized/exclusion counts.
/// - Stage 3: if insufficient, calls [`escalate_to_raw_evidence`] to query the
///   immutable raw-evidence tier.
///
/// Returns `(authorized_facts, exclusions, escalation)`. The orchestrator
/// retrieval path calls this when the DARK capture+retrieval is un-darked.
///
/// # Fail-closed (R2)
/// Tier-A facts with `Unknown` authorization are HARD-EXCLUDED by Step 2;
/// if that leaves zero authorized facts, the escalation fires and the raw
/// evidence tier is queried. The raw evidence is typed evidence (G-1 taint
/// provenance), never injected into auditor/critic/adversary contexts.
#[allow(clippy::too_many_arguments)] // J-21 facade: threads Step 2 authorization + Step 3 escalation stages.
pub fn get_authorized_facts_with_escalation(
    fact_store: &FactStore,
    raw_evidence_store: &RawEvidenceStore,
    namespace: &str,
    user_id: &str,
    filter: &RetrievalFilter,
    k: usize,
    now_ms: u128,
    existence_validator: &dyn CitationValidator,
    semantic_validator: &dyn SemanticValidator,
    scope: &ValidationScope,
    quarantine: Option<&mut QuarantineStore>,
) -> Result<(Vec<MemoryFact>, Vec<AuthorizationExclusion>, RawEvidenceEscalation), MemoryError> {
    // Stage 1: J-21 Step 2 authorized retrieval.
    let (authorized, exclusions) = get_authorized_facts(
        fact_store,
        user_id,
        filter,
        k,
        now_ms,
        existence_validator,
        semantic_validator,
        scope,
        quarantine,
    )?;

    // Stage 2: deterministic sufficiency gate.
    let sufficiency = assess_sufficiency(authorized.len(), exclusions.len());

    // Stage 3: sufficiency-gated raw-evidence escalation.
    // Collect the excluded facts for the escalation query. The exclusions
    // carry fact_ids; we re-fetch the excluded facts from the store to get
    // their evidence_pointers. If a fact was already soft-closed /
    // superseded, it may not be retrievable — skip those.
    let mut excluded_facts: Vec<MemoryFact> = Vec::new();
    if matches!(sufficiency, SufficiencyVerdict::Insufficient(_)) {
        for excl in &exclusions {
            if let Ok(Some(fact)) = fact_store.get(&excl.fact_id) {
                excluded_facts.push(fact);
            }
        }
    }

    let escalation =
        escalate_to_raw_evidence(raw_evidence_store, namespace, &excluded_facts, sufficiency);

    Ok((authorized, exclusions, escalation))
}

/// Crate-internal convenience wrapper around
/// [`get_authorized_facts_with_escalation`] with fail-closed null
/// validators and an empty raw-evidence store. Used by in-crate tests and
/// the future orchestrator wiring shim when the semantic index is
/// unavailable. This wrapper gives [`get_authorized_facts_with_escalation`]
/// a same-file non-test caller (the production orchestrator path is
/// currently DARK per the feature map).
///
/// Delegates the base authorized-retrieval layer to
/// [`crate::retrieval_authorization::authorized_retrieval_fail_closed`]
/// and layers the sufficiency-gated escalation on top.
#[allow(dead_code)]
fn authorized_retrieval_with_escalation_fail_closed(
    fact_store: &FactStore,
    raw_evidence_store: &RawEvidenceStore,
    namespace: &str,
    user_id: &str,
    filter: &RetrievalFilter,
    k: usize,
    now_ms: u128,
) -> Result<(Vec<MemoryFact>, Vec<AuthorizationExclusion>, RawEvidenceEscalation), MemoryError> {
    // Stage 1: base authorized retrieval (fail-closed null validators).
    let (authorized, exclusions) =
        crate::retrieval_authorization::authorized_retrieval_fail_closed(
            fact_store,
            user_id,
            filter,
            k,
            now_ms,
        )?;

    // Stage 2: deterministic sufficiency gate.
    let sufficiency = assess_sufficiency(authorized.len(), exclusions.len());

    // Stage 3: sufficiency-gated raw-evidence escalation.
    let mut excluded_facts: Vec<MemoryFact> = Vec::new();
    if matches!(sufficiency, SufficiencyVerdict::Insufficient(_)) {
        for excl in &exclusions {
            if let Ok(Some(fact)) = fact_store.get(&excl.fact_id) {
                excluded_facts.push(fact);
            }
        }
    }

    let escalation =
        escalate_to_raw_evidence(raw_evidence_store, namespace, &excluded_facts, sufficiency);

    Ok((authorized, exclusions, escalation))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::citation_validity::{CitationValidator, NullValidator, Validity};
    use crate::fact::{FactType, MemoryFact, Scope};
    use crate::retrieval::RetrievalFilter;
    use crate::retrieval_authorization::NullSemanticValidator;
    use crate::semantic_validation::{SemanticError, SemanticValidator, SymbolIdentity, ValidationScope};
    use crate::store::FactStore;
    use crate::trust::{Provenance, TrustTier};

    const T: u128 = 1_700_000_000_000;
    const DKG_PTR: &str = "dkg:validate_jwt@crates/auth/src/lib.rs:42";
    const SEM_PTR: &str = "sem:validate_jwt@abc123def456:f1e2d3c4b5a69788";

    fn make_tier_a_fact(fact_id: &str, evidence: Vec<String>) -> MemoryFact {
        MemoryFact {
            fact_id: fact_id.into(),
            user_id: "u1".into(),
            seat_id: "local".into(),
            project_id: Some("proj-1".into()),
            scope: Scope::Project,
            fact_type: FactType::DomainKnowledge,
            key: "no_unwrap_in_prod".into(),
            value: serde_json::json!({ "raw": "no unwrap in prod" }),
            confidence: 0.9,
            trust_score: 0.8,
            ingested_at_ms: T,
            valid_from_ms: T,
            valid_to_ms: None,
            decay_rate_per_day: 0.01,
            last_observed_at_ms: T,
            evidence_pointers: evidence,
            superseded_by: None,
            trust_tier: TrustTier::T0UserDirect,
            provenance: Provenance::user_direct("test"),
            last_validated_at_ms: None,
        }
    }

    fn make_tier_b_fact() -> MemoryFact {
        MemoryFact {
            fact_id: "fact-pref".into(),
            user_id: "u1".into(),
            seat_id: "local".into(),
            project_id: None,
            scope: Scope::Global,
            fact_type: FactType::Preference,
            key: "pref_key".into(),
            value: serde_json::json!({ "raw": "prefers tracing" }),
            confidence: 0.9,
            trust_score: 0.8,
            ingested_at_ms: T,
            valid_from_ms: T,
            valid_to_ms: None,
            decay_rate_per_day: 0.01,
            last_observed_at_ms: T,
            evidence_pointers: vec!["dkg:foo@bar.rs:1".into()],
            superseded_by: None,
            trust_tier: TrustTier::T0UserDirect,
            provenance: Provenance::user_direct("test"),
            last_validated_at_ms: None,
        }
    }

    struct AlwaysValidExistence;
    impl CitationValidator for AlwaysValidExistence {
        fn validate(&self, _fact: &MemoryFact) -> Validity {
            Validity::Valid
        }
    }

    struct MatchingSemantic;
    impl SemanticValidator for MatchingSemantic {
        fn rederive(
            &self,
            _pointer: &crate::citation_validity::CitationPointer,
            _scope: &ValidationScope,
        ) -> Result<Option<SymbolIdentity>, SemanticError> {
            Ok(Some(SymbolIdentity {
                symbol_name: "validate_jwt".into(),
                commit_hash: "newcommit".into(),
                signature_blake3: "f1e2d3c4b5a69788".into(),
                file: "crates/auth/src/lib.rs".into(),
                line: 42,
            }))
        }
    }

    struct RefactoredSemantic;
    impl SemanticValidator for RefactoredSemantic {
        fn rederive(
            &self,
            _pointer: &crate::citation_validity::CitationPointer,
            _scope: &ValidationScope,
        ) -> Result<Option<SymbolIdentity>, SemanticError> {
            Ok(Some(SymbolIdentity {
                symbol_name: "validate_jwt".into(),
                commit_hash: "newcommit".into(),
                signature_blake3: "aaaaaaaaaaaaaaaa".into(),
                file: "crates/auth/src/lib.rs".into(),
                line: 42,
            }))
        }
    }

    #[test]
    fn composed_retrieval_no_escalation_when_sufficient() {
        let fact_store = FactStore::open_in_memory().expect("open store");
        let raw_store = RawEvidenceStore::new();
        let tier_b = make_tier_b_fact();
        fact_store.insert(&tier_b).expect("insert tier-b");

        let filter = RetrievalFilter::default();
        let (authorized, exclusions, escalation) =
            get_authorized_facts_with_escalation(
                &fact_store,
                &raw_store,
                "ns1",
                "u1",
                &filter,
                10,
                T + 1000,
                &NullValidator,
                &MatchingSemantic,
                &ValidationScope::at_head("repo", "main"),
                None,
            )
            .expect("composed retrieval");

        assert!(!authorized.is_empty());
        assert!(exclusions.is_empty());
        assert!(!escalation.was_escalated);
        assert!(escalation.records.is_empty());
    }

    #[test]
    fn composed_retrieval_escalates_when_all_excluded() {
        let fact_store = FactStore::open_in_memory().expect("open store");
        let mut raw_store = RawEvidenceStore::new();
        // Seed the raw-evidence tier with the dropped detail.
        raw_store.insert(RawEvidenceRecord::new(
            "ns1",
            DKG_PTR,
            "the raw source that has the detail extraction dropped",
            Provenance::codebase_derived("crates/auth/src/lib.rs"),
            T,
        ));
        let tier_a = make_tier_a_fact("fact-auth-1", vec![DKG_PTR.into(), SEM_PTR.into()]);
        fact_store.insert(&tier_a).expect("insert tier-a");

        let filter = RetrievalFilter::default();
        let (authorized, exclusions, escalation) =
            get_authorized_facts_with_escalation(
                &fact_store,
                &raw_store,
                "ns1",
                "u1",
                &filter,
                10,
                T + 1000,
                &AlwaysValidExistence,
                &RefactoredSemantic, // signature changed → SemanticStale
                &ValidationScope::at_head("repo", "main"),
                None,
            )
            .expect("composed retrieval");

        // Tier-A fact is semantically stale → excluded → zero authorized.
        assert!(authorized.is_empty());
        assert_eq!(exclusions.len(), 1);
        // Sufficiency gate fires → escalation to raw evidence.
        assert!(escalation.was_escalated);
        assert_eq!(
            escalation.sufficiency,
            SufficiencyVerdict::Insufficient(SufficiencyReason::AllExcluded)
        );
        assert_eq!(escalation.records.len(), 1);
        assert_eq!(
            escalation.records[0].raw_content,
            "the raw source that has the detail extraction dropped"
        );
    }

    #[test]
    fn composed_retrieval_escalates_no_candidates_when_store_empty() {
        let fact_store = FactStore::open_in_memory().expect("open store");
        let raw_store = RawEvidenceStore::new();

        let filter = RetrievalFilter::default();
        let (authorized, exclusions, escalation) =
            get_authorized_facts_with_escalation(
                &fact_store,
                &raw_store,
                "ns1",
                "u1",
                &filter,
                10,
                T + 1000,
                &NullValidator,
                &NullSemanticValidator,
                &ValidationScope::at_head("repo", "main"),
                None,
            )
            .expect("composed retrieval");

        assert!(authorized.is_empty());
        assert!(exclusions.is_empty());
        assert!(escalation.was_escalated);
        assert_eq!(
            escalation.sufficiency,
            SufficiencyVerdict::Insufficient(SufficiencyReason::NoCandidates)
        );
        assert!(escalation.records.is_empty());
    }

    #[test]
    fn fail_closed_wrapper_escalates_on_empty_store() {
        let fact_store = FactStore::open_in_memory().expect("open store");
        let raw_store = RawEvidenceStore::new();
        let tier_b = make_tier_b_fact();
        fact_store.insert(&tier_b).expect("insert tier-b");

        let filter = RetrievalFilter::default();
        let (authorized, _exclusions, escalation) =
            authorized_retrieval_with_escalation_fail_closed(
                &fact_store,
                &raw_store,
                "ns1",
                "u1",
                &filter,
                10,
                T + 1000,
            )
            .expect("fail-closed wrapper");

        // Tier-B is authorized (short-circuit); no escalation needed.
        assert_eq!(authorized.len(), 1);
        assert!(!escalation.was_escalated);
    }
}

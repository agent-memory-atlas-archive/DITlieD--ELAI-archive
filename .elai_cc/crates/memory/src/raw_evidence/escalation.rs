//! Sufficiency-gated escalation to the raw-evidence tier.
//!
//! See [`super`] for the architectural overview and grounding.

use crate::fact::MemoryFact;
use crate::raw_evidence::store::RawEvidenceStore;
use crate::raw_evidence::sufficiency::SufficiencyVerdict;

/// The result of a sufficiency-gated raw-evidence escalation. When
/// `was_escalated` is `true`, the `records` field carries the raw-evidence
/// records that were retrieved from the immutable tier. When `false`,
/// `records` is empty (the fact layer was sufficient, or the raw-evidence
/// store had no matching records).
#[derive(Debug, Clone)]
pub struct RawEvidenceEscalation {
    /// `true` if the raw-evidence tier was queried (fact layer was
    /// insufficient).
    pub was_escalated: bool,
    /// The sufficiency verdict that triggered (or did not trigger) the
    /// escalation.
    pub sufficiency: SufficiencyVerdict,
    /// The raw-evidence records retrieved from the immutable tier (empty
    /// if `was_escalated` is `false` or no records matched).
    pub records: Vec<crate::raw_evidence::store::RawEvidenceRecord>,
    /// The evidence pointers that were used to query the raw-evidence
    /// tier (collected from the excluded facts' `evidence_pointers`).
    pub queried_pointers: Vec<String>,
}

/// Escalate to the raw-evidence tier when the fact layer is insufficient.
///
/// Collects the `evidence_pointers` from the excluded facts, queries the
/// [`RawEvidenceStore`] by `namespace` + those pointers, and returns a
/// [`RawEvidenceEscalation`]. Deterministic — no LLM on the read path.
///
/// When `sufficiency` is [`SufficiencyVerdict::Sufficient`], returns a
/// non-escalated [`RawEvidenceEscalation`] with empty records (the fact
/// layer was sufficient; no escalation needed).
pub fn escalate_to_raw_evidence(
    store: &RawEvidenceStore,
    namespace: &str,
    excluded_facts: &[MemoryFact],
    sufficiency: SufficiencyVerdict,
) -> RawEvidenceEscalation {
    if matches!(sufficiency, SufficiencyVerdict::Sufficient) {
        return RawEvidenceEscalation {
            was_escalated: false,
            sufficiency,
            records: Vec::new(),
            queried_pointers: Vec::new(),
        };
    }

    // Collect evidence pointers from the excluded facts — these are the
    // provenance pointers whose raw evidence may carry the dropped detail.
    let mut queried_pointers: Vec<String> = Vec::new();
    for fact in excluded_facts {
        for ptr in &fact.evidence_pointers {
            if !queried_pointers.contains(ptr) {
                queried_pointers.push(ptr.clone());
            }
        }
    }

    let matched: Vec<crate::raw_evidence::store::RawEvidenceRecord> = store
        .query_by_pointers(namespace, &queried_pointers)
        .into_iter()
        .cloned()
        .collect();

    tracing::info!(
        target = "memory.raw_evidence",
        namespace = %namespace,
        sufficiency = %sufficiency,
        queried_pointers = queried_pointers.len(),
        matched = matched.len(),
        "J-21: raw-evidence escalation (sufficiency-gated)"
    );

    RawEvidenceEscalation {
        was_escalated: true,
        sufficiency,
        records: matched,
        queried_pointers,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fact::{FactType, MemoryFact, Scope};
    use crate::raw_evidence::sufficiency::SufficiencyReason;
    use crate::raw_evidence::store::{RawEvidenceRecord, RawEvidenceStore};
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

    #[test]
    fn escalation_noop_when_sufficient() {
        let store = RawEvidenceStore::new();
        let fact = make_tier_a_fact("f1", vec![DKG_PTR.into()]);
        let esc = escalate_to_raw_evidence(
            &store,
            "ns1",
            std::slice::from_ref(&fact),
            SufficiencyVerdict::Sufficient,
        );
        assert!(!esc.was_escalated);
        assert!(esc.records.is_empty());
        assert!(esc.queried_pointers.is_empty());
    }

    #[test]
    fn escalation_fires_when_insufficient_and_matches() {
        let mut store = RawEvidenceStore::new();
        store.insert(RawEvidenceRecord::new(
            "ns1",
            DKG_PTR,
            "the full raw content that extraction dropped",
            Provenance::codebase_derived("crates/auth/src/lib.rs"),
            T,
        ));
        let fact = make_tier_a_fact("f1", vec![DKG_PTR.into()]);
        let esc = escalate_to_raw_evidence(
            &store,
            "ns1",
            std::slice::from_ref(&fact),
            SufficiencyVerdict::Insufficient(SufficiencyReason::AllExcluded),
        );
        assert!(esc.was_escalated);
        assert_eq!(esc.records.len(), 1);
        assert_eq!(esc.records[0].raw_content, "the full raw content that extraction dropped");
        assert!(esc.queried_pointers.contains(&DKG_PTR.to_string()));
    }

    #[test]
    fn escalation_fires_but_empty_when_no_match() {
        let mut store = RawEvidenceStore::new();
        store.insert(RawEvidenceRecord::new(
            "ns1",
            "other-ptr",
            "unrelated content",
            Provenance::codebase_derived("f.rs"),
            T,
        ));
        let fact = make_tier_a_fact("f1", vec![DKG_PTR.into()]);
        let esc = escalate_to_raw_evidence(
            &store,
            "ns1",
            std::slice::from_ref(&fact),
            SufficiencyVerdict::Insufficient(SufficiencyReason::AllExcluded),
        );
        assert!(esc.was_escalated);
        assert!(esc.records.is_empty());
    }

    #[test]
    fn escalation_deduplicates_pointers() {
        let store = RawEvidenceStore::new();
        let f1 = make_tier_a_fact("f1", vec![DKG_PTR.into(), SEM_PTR.into()]);
        let f2 = make_tier_a_fact("f2", vec![DKG_PTR.into()]);
        let esc = escalate_to_raw_evidence(
            &store,
            "ns1",
            &[f1.clone(), f2.clone()],
            SufficiencyVerdict::Insufficient(SufficiencyReason::AllExcluded),
        );
        // DKG_PTR appears in both facts but should be deduplicated.
        let dkg_count = esc
            .queried_pointers
            .iter()
            .filter(|p| p.as_str() == DKG_PTR)
            .count();
        assert_eq!(dkg_count, 1);
        assert_eq!(esc.queried_pointers.len(), 2);
    }
}

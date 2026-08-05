//! C-48 contract test: memory trust interface invariants.
//!
//! Verifies the public API surface of the trust-tier layer without depending
//! on internal implementation details.

use elai_memory::{
    FactStore, FactType, MemoryFact, Provenance, Scope, TrustTier,
    RetrievalFilter, get_relevant_facts, SafetyCriticalQueryType, TierCeiling,
};

const T: u128 = 1_700_000_000_000;

fn make_trust_fact(id: &str, tier: TrustTier, trust: f32) -> MemoryFact {
    MemoryFact {
        fact_id: id.into(),
        user_id: "u1".into(),
        seat_id: "local".to_string(),
        project_id: Some("p1".into()),
        scope: Scope::Project,
        fact_type: FactType::DomainKnowledge,
        key: id.into(),
        value: serde_json::json!({"data": id}),
        confidence: 0.85,
        trust_score: trust,
        ingested_at_ms: T,
        valid_from_ms: T,
        valid_to_ms: None,
        decay_rate_per_day: 0.001,
        last_observed_at_ms: T,
        evidence_pointers: vec!["e1".into()],
        superseded_by: None,
        trust_tier: tier,
        provenance: Provenance::user_direct("contract-test"),
        last_validated_at_ms: None,
    }
}

/// CONTRACT: TrustTier wire tags are stable and round-trip correctly.
#[test]
fn contract_trust_tier_wire_tags_stable() {
    assert_eq!(TrustTier::T0UserDirect.as_str(), "T0");
    assert_eq!(TrustTier::T1CodebaseDerived.as_str(), "T1");
    assert_eq!(TrustTier::T2WebSourced.as_str(), "T2");
    assert_eq!(TrustTier::T3ModelEmitted.as_str(), "T3");

    assert_eq!(TrustTier::from_str("T0").unwrap(), TrustTier::T0UserDirect);
    assert_eq!(TrustTier::from_str("T1").unwrap(), TrustTier::T1CodebaseDerived);
    assert_eq!(TrustTier::from_str("T2").unwrap(), TrustTier::T2WebSourced);
    assert_eq!(TrustTier::from_str("T3").unwrap(), TrustTier::T3ModelEmitted);
    assert!(TrustTier::from_str("TX").is_err());
}

/// CONTRACT: safety_critical_allowed() is T0+T1 only.
#[test]
fn contract_safety_critical_allowed_t0_t1_only() {
    assert!(TrustTier::T0UserDirect.is_safety_critical_allowed());
    assert!(TrustTier::T1CodebaseDerived.is_safety_critical_allowed());
    assert!(!TrustTier::T2WebSourced.is_safety_critical_allowed());
    assert!(!TrustTier::T3ModelEmitted.is_safety_critical_allowed());
}

/// CONTRACT: TierCeiling::T0T1Only excludes T2/T3.
#[test]
fn contract_tier_ceiling_permits_correctly() {
    assert!(TierCeiling::AllTiers.permits(TrustTier::T2WebSourced));
    assert!(TierCeiling::AllTiers.permits(TrustTier::T3ModelEmitted));
    assert!(!TierCeiling::T0T1Only.permits(TrustTier::T2WebSourced));
    assert!(!TierCeiling::T0T1Only.permits(TrustTier::T3ModelEmitted));
    assert!(TierCeiling::T0T1Only.permits(TrustTier::T0UserDirect));
}

/// CONTRACT: SafetyCriticalQueryType classifies known types correctly.
#[test]
fn contract_safety_critical_query_type_classification() {
    assert!(SafetyCriticalQueryType::is_safety_critical("destructive_command_intent"));
    assert!(SafetyCriticalQueryType::is_safety_critical("merge_decision"));
    assert!(SafetyCriticalQueryType::is_safety_critical("auth_decision"));
    assert!(SafetyCriticalQueryType::is_safety_critical("credential_fetch"));
    assert!(!SafetyCriticalQueryType::is_safety_critical("general_lookup"));
    assert!(!SafetyCriticalQueryType::is_safety_critical("enrichment_query"));
}

/// CONTRACT: Provenance constructors set tier correctly.
#[test]
fn contract_provenance_constructors_set_tier() {
    assert_eq!(Provenance::user_direct("x").tier, TrustTier::T0UserDirect);
    assert_eq!(Provenance::codebase_derived("x").tier, TrustTier::T1CodebaseDerived);
    assert_eq!(Provenance::web_sourced("x").tier, TrustTier::T2WebSourced);
    assert_eq!(Provenance::model_emitted("x").tier, TrustTier::T3ModelEmitted);
}

/// CONTRACT: safety_critical=true filter excludes T2/T3 from retrieval.
#[test]
fn contract_safety_critical_retrieval_filter() {
    let store = FactStore::open_in_memory().expect("store");
    for (id, tier) in [
        ("ct0", TrustTier::T0UserDirect),
        ("ct1", TrustTier::T1CodebaseDerived),
        ("ct2", TrustTier::T2WebSourced),
        ("ct3", TrustTier::T3ModelEmitted),
    ] {
        store.insert(&make_trust_fact(id, tier, 0.9)).expect("insert");
    }

    let sc_filter = RetrievalFilter {
        project_id: Some("p1".into()),
        seat_id: None,
        fact_type: None,
        trust_override: None,
        safety_critical: true,
        citation_validator: None,
    };
    let (sc_results, _) = get_relevant_facts(&store, "u1", &sc_filter, 10, T + 1000)
        .expect("get");
    let sc_ids: Vec<_> = sc_results.iter().map(|f| f.fact_id.as_str()).collect();
    assert!(sc_ids.contains(&"ct0"));
    assert!(sc_ids.contains(&"ct1"));
    assert!(!sc_ids.contains(&"ct2"));
    assert!(!sc_ids.contains(&"ct3"));

    let open_filter = RetrievalFilter::default();
    let (open_results, _) = get_relevant_facts(&store, "u1", &open_filter, 10, T + 2000)
        .expect("get open");
    assert_eq!(open_results.len(), 4, "all 4 tiers visible in non-safety query");
}

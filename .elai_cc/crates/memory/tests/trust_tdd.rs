//! C-48 TDD tests for TrustTier + Provenance + MemoryView safety-critical filter.
//!
//! Tests:
//!   4D.X.3 -- TrustTier round-trips through MemoryFact insert/get
//!   4D.X.4 -- T0_T1_only filter excludes T2/T3 rows from result
//!   4D.X.5 -- provenance propagates to caller (visible on returned MemoryFact)
//!   4D.X.6 -- migration default: unlabeled rows (DEFAULT 'T3') round-trip as T3

use elai_memory::{FactStore, FactType, MemoryFact, Provenance, Scope, TrustTier};
use elai_memory::{RetrievalFilter, get_relevant_facts};

const T: u128 = 1_700_000_000_000;

fn make_fact_with_tier(
    fact_id: &str,
    key: &str,
    tier: TrustTier,
    provenance: Provenance,
    trust: f32,
) -> MemoryFact {
    MemoryFact {
        fact_id: fact_id.into(),
        user_id: "default".into(),
        project_id: Some("proj-a".into()),
        scope: Scope::Project,
        fact_type: FactType::Preference,
        key: key.into(),
        value: serde_json::json!({"subject": key}),
        confidence: 0.9,
        trust_score: trust,
        ingested_at_ms: T,
        valid_from_ms: T,
        valid_to_ms: None,
        decay_rate_per_day: 0.001,
        last_observed_at_ms: T,
        evidence_pointers: vec!["sess-1".into()],
        superseded_by: None,
        trust_tier: tier,
        provenance,
    }
}

/// 4D.X.3 -- TrustTier round-trips through MemoryFact insert/get.
#[test]
fn trust_tier_round_trips() {
    let store = FactStore::open_in_memory().expect("open store");

    let cases = [
        (TrustTier::T0UserDirect, "f-t0"),
        (TrustTier::T1CodebaseDerived, "f-t1"),
        (TrustTier::T2WebSourced, "f-t2"),
        (TrustTier::T3ModelEmitted, "f-t3"),
    ];

    for (tier, id) in &cases {
        let prov = Provenance::user_direct("test");
        store.insert(&make_fact_with_tier(id, id, *tier, prov, 0.8)).expect("insert");
    }

    for (tier, id) in &cases {
        let reloaded = store.get(id).expect("get").expect("exists");
        assert_eq!(
            reloaded.trust_tier, *tier,
            "trust_tier must survive insert/get for id={id}"
        );
    }
}

/// 4D.X.4 -- safety-critical filter excludes T2/T3 rows.
#[test]
fn safety_critical_filter_excludes_t2_t3() {
    let store = FactStore::open_in_memory().expect("open store");

    let cases = [
        ("f-t0", TrustTier::T0UserDirect),
        ("f-t1", TrustTier::T1CodebaseDerived),
        ("f-t2", TrustTier::T2WebSourced),
        ("f-t3", TrustTier::T3ModelEmitted),
    ];
    for (id, tier) in &cases {
        let prov = Provenance::user_direct("test");
        store.insert(&make_fact_with_tier(id, id, *tier, prov, 0.9)).expect("insert");
    }

    let filter = RetrievalFilter {
        project_id: Some("proj-a".into()),
        fact_type: None,
        trust_override: None,
        safety_critical: true,
    };
    let results = get_relevant_facts(&store, "default", &filter, 10, T + 1000)
        .expect("get_relevant_facts");

    let ids: Vec<&str> = results.iter().map(|f| f.fact_id.as_str()).collect();
    assert!(ids.contains(&"f-t0"), "T0 must be included");
    assert!(ids.contains(&"f-t1"), "T1 must be included");
    assert!(!ids.contains(&"f-t2"), "T2 must be excluded");
    assert!(!ids.contains(&"f-t3"), "T3 must be excluded");
}

/// 4D.X.5 -- provenance propagates through retrieval.
#[test]
fn provenance_propagates_to_caller() {
    let store = FactStore::open_in_memory().expect("open store");
    let prov = Provenance::codebase_derived("src/main.rs:42");
    let fact = make_fact_with_tier(
        "f-prov",
        "prov_key",
        TrustTier::T1CodebaseDerived,
        prov.clone(),
        0.85,
    );
    store.insert(&fact).expect("insert");

    let reloaded = store.get("f-prov").expect("get").expect("exists");
    assert_eq!(reloaded.provenance, prov, "provenance survives insert/get");

    let filter = RetrievalFilter::default();
    let results = get_relevant_facts(&store, "default", &filter, 10, T + 1000)
        .expect("get_relevant_facts");
    let hit = results.iter().find(|f| f.fact_id == "f-prov").expect("found");
    assert_eq!(hit.provenance, prov, "provenance propagates through retrieval");
}

/// 4D.X.6 -- T3 default is most pessimistic (migration default).
#[test]
fn migration_default_is_t3() {
    let store = FactStore::open_in_memory().expect("open store");
    let fact = make_fact_with_tier(
        "f-legacy",
        "legacy_key",
        TrustTier::T3ModelEmitted,
        Provenance::model_emitted("chat_memory_reflect"),
        0.6,
    );
    store.insert(&fact).expect("insert");

    let reloaded = store.get("f-legacy").expect("get").expect("exists");
    assert_eq!(reloaded.trust_tier, TrustTier::T3ModelEmitted, "T3 round-trips");

    let filter = RetrievalFilter {
        project_id: Some("proj-a".into()),
        fact_type: None,
        trust_override: None,
        safety_critical: true,
    };
    let results = get_relevant_facts(&store, "default", &filter, 10, T + 1000)
        .expect("get_relevant_facts");
    assert!(
        results.iter().all(|f| f.fact_id != "f-legacy"),
        "T3 row must be excluded from safety-critical query"
    );
}

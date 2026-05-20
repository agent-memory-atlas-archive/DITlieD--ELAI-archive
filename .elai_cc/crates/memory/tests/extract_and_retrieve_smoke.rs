//! Observable-effect smoke (E-5 plan WIRING_CONTRACT::observable_effect):
//! seed a conversation that contains a tracing preference, run
//! [`extract_facts`], persist the candidate as a [`MemoryFact`], apply
//! one decay tick, then verify [`get_relevant_facts`] returns it with a
//! decayed (but still above-threshold) trust score.

use elai_memory::{
    decay_tick, extract_facts, get_relevant_facts, Conversation, FactStore, FactType, MemoryFact,
    RetrievalFilter, Scope, Turn,
};

#[test]
fn extract_then_persist_then_retrieve_with_decay() {
    // 1. Run the extractor on a synthetic conversation.
    let conv = Conversation {
        session_id: "sess-smoke".into(),
        turns: vec![Turn {
            message_id: "msg-1".into(),
            role: "user".into(),
            text: "i always prefer tracing over the log crate".into(),
        }],
    };
    let candidates = extract_facts(&conv).expect("extract");
    assert!(
        !candidates.is_empty(),
        "extractor must emit at least one candidate"
    );
    let c = &candidates[0];
    assert_eq!(c.fact_type, FactType::Preference);
    assert!(c.key.starts_with("prefers_"));
    assert!(c.confidence > 0.0);

    // 2. Persist as a MemoryFact.
    let store = FactStore::open_in_memory().expect("open store");
    let t0: u128 = 1_700_000_000_000;
    let fact = MemoryFact {
        fact_id: MemoryFact::new_id(),
        user_id: "default".into(),
        project_id: Some("proj-x".into()),
        scope: Scope::Project,
        fact_type: c.fact_type,
        key: c.key.clone(),
        value: c.value.clone(),
        confidence: c.confidence,
        trust_score: 0.9,
        ingested_at_ms: t0,
        valid_from_ms: t0,
        valid_to_ms: None,
        decay_rate_per_day: elai_memory::default_decay_rate(c.fact_type),
        last_observed_at_ms: t0,
        evidence_pointers: c.evidence.clone(),
        superseded_by: None,
    };
    store.insert(&fact).expect("insert");

    // 3. Decay tick advances by 30 days.
    let t_now = t0 + 86_400_000 * 30;
    let stats = decay_tick(&store, t_now).expect("decay");
    assert_eq!(stats.rows_examined, 1);
    assert_eq!(stats.rows_updated, 1);

    // 4. get_relevant_facts must return the row with a decayed trust score
    //    above the Preference threshold (0.5).
    let filter = RetrievalFilter {
        project_id: Some("proj-x".into()),
        fact_type: Some(FactType::Preference),
        trust_override: None,
    };
    let hits = get_relevant_facts(&store, "default", &filter, 4, t_now + 1)
        .expect("retrieve");
    assert_eq!(hits.len(), 1, "retrieval must surface the persisted fact");
    let hit = &hits[0];
    assert!(
        hit.trust_score > FactType::Preference.retrieval_threshold(),
        "decayed trust ({}) must still clear threshold ({})",
        hit.trust_score,
        FactType::Preference.retrieval_threshold()
    );
    // last_observed_at_ms must have advanced to now (decay-feedback loop).
    assert!(hit.last_observed_at_ms > t0);
}

#[test]
fn empty_evidence_rejected_at_insert() {
    let store = FactStore::open_in_memory().expect("open store");
    let t0: u128 = 1_700_000_000_000;
    let bad = MemoryFact {
        fact_id: MemoryFact::new_id(),
        user_id: "default".into(),
        project_id: Some("proj-y".into()),
        scope: Scope::Project,
        fact_type: FactType::Preference,
        key: "no_evidence".into(),
        value: serde_json::json!({}),
        confidence: 0.9,
        trust_score: 0.9,
        ingested_at_ms: t0,
        valid_from_ms: t0,
        valid_to_ms: None,
        decay_rate_per_day: 0.001,
        last_observed_at_ms: t0,
        evidence_pointers: vec![],
        superseded_by: None,
    };
    let res = store.insert(&bad);
    assert!(res.is_err(), "empty evidence_pointers must be rejected");
}

#[test]
fn project_scope_filter_isolates_projects() {
    let store = FactStore::open_in_memory().expect("open store");
    let t0: u128 = 1_700_000_000_000;
    for (i, proj) in ["proj-a", "proj-b"].iter().enumerate() {
        let f = MemoryFact {
            fact_id: MemoryFact::new_id(),
            user_id: "default".into(),
            project_id: Some((*proj).into()),
            scope: Scope::Project,
            fact_type: FactType::Preference,
            key: format!("prefers_thing_{i}"),
            value: serde_json::json!({"i": i}),
            confidence: 0.9,
            trust_score: 0.9,
            ingested_at_ms: t0,
            valid_from_ms: t0,
            valid_to_ms: None,
            decay_rate_per_day: 0.001,
            last_observed_at_ms: t0 + i as u128,
            evidence_pointers: vec!["sess".into(), "msg".into()],
            superseded_by: None,
        };
        store.insert(&f).expect("insert");
    }

    let filter = RetrievalFilter {
        project_id: Some("proj-a".into()),
        fact_type: None,
        trust_override: None,
    };
    let hits = get_relevant_facts(&store, "default", &filter, 10, t0 + 100).expect("retrieve");
    assert_eq!(hits.len(), 1, "project filter must isolate proj-a row");
    assert_eq!(hits[0].project_id.as_deref(), Some("proj-a"));
}

//! Bi-temporal round-trip: insert -> supersede -> verify the closed row's
//! `valid_to_ms` is set + the new row's `superseded_by` points back to
//! the old `fact_id`. Mirrors the Zep §2.2.3 invariant.

use elai_memory::{FactStore, FactType, MemoryFact, Provenance, Scope, TrustTier};

fn now_ms_base() -> u128 {
    1_700_000_000_000
}

fn make_fact(
    fact_id: &str,
    key: &str,
    value: serde_json::Value,
    trust: f32,
    t: u128,
) -> MemoryFact {
    MemoryFact {
        fact_id: fact_id.into(),
        user_id: "default".into(),
        project_id: Some("proj-a".into()),
        scope: Scope::Project,
        fact_type: FactType::Preference,
        key: key.into(),
        value,
        confidence: 0.9,
        trust_score: trust,
        ingested_at_ms: t,
        valid_from_ms: t,
        valid_to_ms: None,
        decay_rate_per_day: 0.001,
        last_observed_at_ms: t,
        evidence_pointers: vec!["sess-1".into(), "msg-1".into()],
        superseded_by: None,
        trust_tier: TrustTier::T1CodebaseDerived,
        provenance: Provenance::codebase_derived("test"),
        last_validated_at_ms: None,
    }
}

#[test]
fn supersede_closes_old_and_links() {
    let mut store = FactStore::open_in_memory().expect("open store");
    let t0 = now_ms_base();
    let old = make_fact(
        "old-id",
        "prefers_tracing_over_log",
        serde_json::json!({"subject": "tracing", "alternative": "log"}),
        0.7,
        t0,
    );
    store.insert(&old).expect("insert old");

    let t1 = t0 + 86_400_000; // +1 day
    let mut new_fact = make_fact(
        "new-id",
        "prefers_tracing_over_log",
        serde_json::json!({"subject": "tracing", "alternative": "log_crate"}),
        0.85,
        t1,
    );
    new_fact.confidence = 0.95;
    store
        .supersede("old-id", &new_fact, t1)
        .expect("supersede");

    // Old row should now have valid_to_ms == t1 and superseded_by == "new-id".
    let reloaded_old = store
        .get("old-id")
        .expect("get old")
        .expect("old row exists");
    assert_eq!(
        reloaded_old.valid_to_ms,
        Some(t1),
        "valid_to_ms must be set on superseded row"
    );
    assert_eq!(
        reloaded_old.superseded_by.as_deref(),
        Some("new-id"),
        "superseded_by must point to new fact"
    );

    // New row must be live (valid_to_ms == None).
    let reloaded_new = store
        .get("new-id")
        .expect("get new")
        .expect("new row exists");
    assert!(reloaded_new.valid_to_ms.is_none(), "new row must be live");
    assert_eq!(reloaded_new.confidence, 0.95);
}

#[test]
fn supersede_unknown_returns_not_found() {
    let mut store = FactStore::open_in_memory().expect("open store");
    let t0 = now_ms_base();
    let new_fact = make_fact("orphan", "foo", serde_json::json!({}), 0.5, t0);
    let res = store.supersede("does-not-exist", &new_fact, t0);
    assert!(res.is_err(), "supersede of missing row must error");
}

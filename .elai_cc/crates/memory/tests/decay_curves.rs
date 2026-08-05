//! Decay curve sanity. Verifies the pure math + the batch helper against
//! known half-lives.

use elai_memory::{decay_tick, decayed_trust, FactStore, FactType, MemoryFact, Provenance, Scope, TrustTier};

#[test]
fn pure_math_monotone_and_bounded() {
    let t0: u128 = 1_700_000_000_000;
    let t1: u128 = t0 + 86_400_000 * 30; // +30 days

    // preference: 0.001/day -> ~30 days = factor exp(-0.03) ~= 0.9704.
    let new_trust = decayed_trust(1.0, 0.001, t0, t1);
    assert!(new_trust < 1.0, "trust must decay below initial 1.0");
    assert!(new_trust > 0.96, "decay over 30 days at 0.001 too aggressive");
    assert!(new_trust < 0.99, "decay over 30 days at 0.001 too weak");

    // session_pattern: 0.01/day over 30 days -> factor exp(-0.3) ~= 0.7408.
    let new_trust = decayed_trust(1.0, 0.01, t0, t1);
    assert!(new_trust > 0.73 && new_trust < 0.75);

    // No-op cases.
    assert_eq!(decayed_trust(0.8, 0.0, t0, t1), 0.8, "0 rate keeps trust");
    assert_eq!(decayed_trust(0.8, 0.001, t1, t0), 0.8, "future-dated keeps trust");
    assert_eq!(decayed_trust(0.0, 0.001, t0, t1), 0.0, "zero stays zero");
}

#[test]
fn decay_tick_updates_and_emits_stats() {
    let store = FactStore::open_in_memory().expect("open store");
    let t0: u128 = 1_700_000_000_000;
    let t_now: u128 = t0 + 86_400_000 * 30; // +30 days

    let f = MemoryFact {
        fact_id: "fact-a".into(),
        user_id: "default".into(),
        seat_id: "local".to_string(),
        project_id: Some("p1".into()),
        scope: Scope::Project,
        fact_type: FactType::SessionPattern,
        key: "prefers_morning".into(),
        value: serde_json::json!({"raw": "i work best in the morning"}),
        confidence: 0.9,
        trust_score: 1.0,
        ingested_at_ms: t0,
        valid_from_ms: t0,
        valid_to_ms: None,
        decay_rate_per_day: 0.01,
        last_observed_at_ms: t0,
        evidence_pointers: vec!["sess-1".into(), "msg-1".into()],
        superseded_by: None,
        trust_tier: TrustTier::T1CodebaseDerived,
        provenance: Provenance::codebase_derived("test"),
        last_validated_at_ms: None,
    };
    store.insert(&f).expect("insert");

    let stats = decay_tick(&store, &std::collections::HashSet::new(), t_now).expect("decay_tick");
    assert_eq!(stats.rows_examined, 1);
    assert_eq!(stats.rows_updated, 1, "row must have been updated");
    assert!(stats.mean_trust_score > 0.73 && stats.mean_trust_score < 0.75);

    // Reload and check persisted trust_score followed the curve.
    let reloaded = store.get("fact-a").expect("get").expect("row exists");
    assert!(reloaded.trust_score > 0.73 && reloaded.trust_score < 0.75);
    // last_observed_at must NOT have advanced (passive decay).
    assert_eq!(reloaded.last_observed_at_ms, t0);
}

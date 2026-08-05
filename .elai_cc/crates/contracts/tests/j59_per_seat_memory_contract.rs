//! J-59 residual-C contract: per-seat memory isolation pins.
//!
//! Drives the REAL public symbols from elai_memory + elai_orchestrator
//! (D-174: no source grep, no reimplemented filter). Seven arms:
//! (a) single_local: FactStore + seat_id="local" round-trips via RetrievalFilter.
//! (b) two seats: seat A and B rows; filter B excludes A.
//! (c) cross-seat read: only A rows present; filter B returns empty.
//! (d) write stamp: DispatchContext.seat_id lands on the stored fact.
//! (e) firewall: agent_role_gets_user_memory Auditor/IdeaCritic == false; Executor == true.
//! (f) trust: seats.toml trust_tier tag loads; tier on one seat's fact is
//!     independent of another seat's fact tier.
//! (g) SSB payload: memory_isolation_payload stamps seat_id; payload carries
//!     only allowed keys (no preference content).
//!
//! Mirrors j49_multi_seat_contract.rs style (short tests, expect in tests).

use elai_memory::fact::{FactType, MemoryFact, Scope};
use elai_memory::mem0_dispatcher::{DispatchContext, DispatchOp, dispatch_one};
use elai_memory::extractor::ExtractedFact;
use elai_memory::retrieval::{get_relevant_facts, RetrievalFilter};
use elai_memory::store::FactStore;
use elai_memory::trust::{Provenance, TrustTier};

use elai_orchestrator::seat_registry::{SeatRegistry, LOCAL_SEAT_ID};
use elai_orchestrator::ssb::{MemoryGovernanceKind, SsbPayload};
use elai_orchestrator::user_memory_firewall::{agent_role_gets_user_memory, memory_isolation_payload};

use elai_ipc_types::AgentRole;

// --- helper: build a minimal valid fact for a given seat ---

fn make_fact(seat_id: &str, key: &str, value: serde_json::Value) -> MemoryFact {
    MemoryFact {
        fact_id: MemoryFact::new_id(),
        user_id: "default".to_string(),
        seat_id: seat_id.to_string(),
        project_id: Some("proj-1".to_string()),
        scope: Scope::Project,
        fact_type: FactType::Preference,
        key: key.to_string(),
        value,
        confidence: 0.9,
        trust_score: 0.8,
        ingested_at_ms: 1_000,
        valid_from_ms: 1_000,
        valid_to_ms: None,
        decay_rate_per_day: 0.05,
        last_observed_at_ms: 1_000,
        evidence_pointers: vec!["ev-1".to_string()],
        superseded_by: None,
        trust_tier: TrustTier::T0UserDirect,
        provenance: Provenance::user_direct("test"),
        last_validated_at_ms: None,
    }
}

// --- (a) single_local: FactStore in-memory; write fact seat_id="local";
//     retrieve with RetrievalFilter.seat_id=Some("local") returns it. ---

#[test]
fn contract_a_single_local_round_trip() {
    let store = FactStore::open_in_memory().expect("open in-memory store");
    let fact = make_fact("local", "prefers_tracing", serde_json::json!({"v": true}));
    store.insert(&fact).expect("insert");

    let filter = RetrievalFilter {
        seat_id: Some("local".to_string()),
        ..Default::default()
    };
    let (facts, _) = get_relevant_facts(&store, "default", &filter, 10, 2_000)
        .expect("retrieve");
    assert_eq!(facts.len(), 1);
    assert_eq!(facts[0].seat_id, "local");
}

// --- (b) two seats: insert fact seat A and seat B with different keys/values;
//     filter seat B excludes A. ---

#[test]
fn contract_b_two_seats_filter_excludes_other() {
    let store = FactStore::open_in_memory().expect("open");
    store
        .insert(&make_fact("seat-a", "key_a", serde_json::json!({"v": 1})))
        .expect("insert A");
    store
        .insert(&make_fact("seat-b", "key_b", serde_json::json!({"v": 2})))
        .expect("insert B");

    let filter_b = RetrievalFilter {
        seat_id: Some("seat-b".to_string()),
        ..Default::default()
    };
    let (facts, _) = get_relevant_facts(&store, "default", &filter_b, 10, 2_000)
        .expect("retrieve B");
    assert_eq!(facts.len(), 1, "seat-b filter should exclude seat-a row");
    assert_eq!(facts[0].seat_id, "seat-b");
    assert_eq!(facts[0].key, "key_b");

    let filter_a = RetrievalFilter {
        seat_id: Some("seat-a".to_string()),
        ..Default::default()
    };
    let (facts_a, _) = get_relevant_facts(&store, "default", &filter_a, 10, 2_000)
        .expect("retrieve A");
    assert_eq!(facts_a.len(), 1);
    assert_eq!(facts_a[0].seat_id, "seat-a");
}

// --- (c) cross-seat read: only A rows present; filter B returns empty. ---

#[test]
fn contract_c_cross_seat_read_returns_empty() {
    let store = FactStore::open_in_memory().expect("open");
    store
        .insert(&make_fact("seat-a", "only_a", serde_json::json!({"v": 1})))
        .expect("insert A");

    let filter_b = RetrievalFilter {
        seat_id: Some("seat-b".to_string()),
        ..Default::default()
    };
    let (facts, _) = get_relevant_facts(&store, "default", &filter_b, 10, 2_000)
        .expect("retrieve B");
    assert!(facts.is_empty(), "seat-b filter with no seat-b rows => empty");
}

// --- (d) write stamp: after insert via dispatch path, fact.seat_id equals
//     ctx seat (structural stamp). ---

#[test]
fn contract_d_dispatch_stamps_seat_id() {
    let mut store = FactStore::open_in_memory().expect("open");
    let ctx = DispatchContext {
        user_id: "default".to_string(),
        seat_id: "seat-x".to_string(),
        project_id: Some("proj-1".to_string()),
        scope: Scope::Project,
        provenance: Provenance::user_direct("test"),
        trust_tier: TrustTier::T0UserDirect,
        initial_trust_score: 0.8,
        now_ms: 1_700_000_000_000,
    };
    let cand = ExtractedFact {
        fact_type: FactType::Preference,
        key: "prefers_tracing".to_string(),
        value: serde_json::json!({"v": true}),
        confidence: 0.9,
        evidence: vec!["sess-1".to_string()],
    };
    let op = dispatch_one(&mut store, &cand, &ctx).expect("dispatch");
    let fact_id = match op {
        DispatchOp::Add { fact_id } => fact_id,
        other => panic!("expected Add, got {other:?}"),
    };

    // The stored fact must carry the ctx seat_id (structural stamp).
    let stored = store.get(&fact_id).expect("get").expect("fact exists");
    assert_eq!(stored.seat_id, "seat-x");

    // Retrieval with seat_id="seat-x" finds it; with "seat-y" does not.
    let filter_x = RetrievalFilter {
        seat_id: Some("seat-x".to_string()),
        ..Default::default()
    };
    let (facts_x, _) = get_relevant_facts(&store, "default", &filter_x, 10, 2_000)
        .expect("retrieve X");
    assert_eq!(facts_x.len(), 1);

    let filter_y = RetrievalFilter {
        seat_id: Some("seat-y".to_string()),
        ..Default::default()
    };
    let (facts_y, _) = get_relevant_facts(&store, "default", &filter_y, 10, 2_000)
        .expect("retrieve Y");
    assert!(facts_y.is_empty());

    // NOTE: FactStore has no separate reject API for "foreign seat write" —
    // the seat_id stamp is structural: the caller's DispatchContext.seat_id
    // is the sole authority for which seat a fact lands in. Isolation is
    // enforced at RETRIEVAL (RetrievalFilter.seat_id hard-exclude), not at
    // write. A foreign-seat write would require the caller to lie in
    // DispatchContext.seat_id, which is outside the store's trust boundary.
}

// --- (e) firewall: agent_role_gets_user_memory(Auditor)==false and
//     IdeaCritic false; Executor true. ---

#[test]
fn contract_e_firewall_role_gate() {
    assert!(
        !agent_role_gets_user_memory(AgentRole::Auditor),
        "Auditor MUST get zero user-memory"
    );
    assert!(
        !agent_role_gets_user_memory(AgentRole::IdeaCritic),
        "IdeaCritic MUST get zero user-memory"
    );
    assert!(
        agent_role_gets_user_memory(AgentRole::Executor),
        "Executor MUST receive user-memory"
    );
}

// --- (f) trust: seats.toml with [seats.low]\ntrust_tier="T3" =>
//     trust_tier_tag_for("low")==Some("T3"). A MemoryFact with trust_tier T3
//     for seat low does not change a seat high fact's tier (independent rows).

#[test]
fn contract_f_trust_tier_tag_and_independent_rows() {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let seats_path = dir.path().join("seats.toml");
    std::fs::write(
        &seats_path,
        "[seats.low]\ntrust_tier = \"T3\"\n\n[seats.high]\nweight = 2\n",
    )
    .expect("write seats.toml");
    let reg = SeatRegistry::load_from_path(&seats_path).expect("load");
    assert_eq!(reg.trust_tier_tag_for("low"), Some("T3"));
    assert_eq!(reg.trust_tier_tag_for("high"), None);

    // Independent rows: a T3 fact for seat "low" does not change a T0 fact
    // for seat "high". The trust_tier is per-row, not per-seat-namespace.
    let store = FactStore::open_in_memory().expect("open");
    let mut low_fact = make_fact("low", "low_key", serde_json::json!({"v": 1}));
    low_fact.trust_tier = TrustTier::T3ModelEmitted;
    let mut high_fact = make_fact("high", "high_key", serde_json::json!({"v": 2}));
    high_fact.trust_tier = TrustTier::T0UserDirect;
    store.insert(&low_fact).expect("insert low");
    store.insert(&high_fact).expect("insert high");

    let filter_low = RetrievalFilter {
        seat_id: Some("low".to_string()),
        ..Default::default()
    };
    let (facts_low, _) = get_relevant_facts(&store, "default", &filter_low, 10, 2_000)
        .expect("retrieve low");
    assert_eq!(facts_low.len(), 1);
    assert_eq!(facts_low[0].trust_tier, TrustTier::T3ModelEmitted);

    let filter_high = RetrievalFilter {
        seat_id: Some("high".to_string()),
        ..Default::default()
    };
    let (facts_high, _) = get_relevant_facts(&store, "default", &filter_high, 10, 2_000)
        .expect("retrieve high");
    assert_eq!(facts_high.len(), 1);
    assert_eq!(facts_high[0].trust_tier, TrustTier::T0UserDirect);
}

// --- (g) SSB payload: build via memory_isolation_payload(WriteStamped, ...);
//     assert seat_id matches; serde_json assert no preference content fields
//     beyond allowed keys (fact_id/kind/actor/session_id/created_at_ms/seat_id). ---

#[test]
fn contract_g_ssb_isolation_payload_shape() {
    let payload = memory_isolation_payload(
        MemoryGovernanceKind::WriteStamped,
        "seat-a",
        "cli",
        "sess-1",
        1_700_000_000_000,
    );
    let mgep = match payload {
        SsbPayload::MemoryGovernanceEvent(mg) => mg,
        other => panic!("expected MemoryGovernanceEvent, got {other:?}"),
    };
    assert_eq!(mgep.seat_id, "seat-a");
    assert_eq!(mgep.kind, MemoryGovernanceKind::WriteStamped);
    assert_eq!(mgep.actor, "cli");
    assert_eq!(mgep.session_id, "sess-1");
    assert_eq!(mgep.fact_id, "");

    // serde_json round-trip: the serialized JSON must contain ONLY the
    // allowed keys (fact_id, kind, actor, session_id, created_at_ms,
    // seat_id). No preference content, no value text, no seat secrets.
    let json_str = serde_json::to_string(&mgep).expect("serialize");
    let obj: serde_json::Map<String, serde_json::Value> =
        serde_json::from_str(&json_str).expect("parse object");
    let allowed: std::collections::HashSet<&str> = [
        "fact_id", "kind", "actor", "session_id", "created_at_ms", "seat_id",
    ]
    .into_iter()
    .collect();
    for key in obj.keys() {
        assert!(
            allowed.contains(key.as_str()),
            "unexpected field '{key}' in MemoryGovernanceEventPayload JSON: {json_str}"
        );
    }
    assert_eq!(obj.len(), allowed.len(), "field count mismatch: {json_str}");

    // Also verify the FirewallBlocked + RetrievalFiltered kinds construct
    // cleanly via the same builder.
    let fb = memory_isolation_payload(
        MemoryGovernanceKind::FirewallBlocked,
        "seat-b",
        "cli",
        "sess-2",
        1_700_000_000_001,
    );
    let fb_mg = match fb {
        SsbPayload::MemoryGovernanceEvent(mg) => mg,
        other => panic!("expected MemoryGovernanceEvent, got {other:?}"),
    };
    assert_eq!(fb_mg.kind, MemoryGovernanceKind::FirewallBlocked);
    assert_eq!(fb_mg.seat_id, "seat-b");

    let rf = memory_isolation_payload(
        MemoryGovernanceKind::RetrievalFiltered,
        "seat-c",
        "cli",
        "sess-3",
        1_700_000_000_002,
    );
    let rf_mg = match rf {
        SsbPayload::MemoryGovernanceEvent(mg) => mg,
        other => panic!("expected MemoryGovernanceEvent, got {other:?}"),
    };
    assert_eq!(rf_mg.kind, MemoryGovernanceKind::RetrievalFiltered);
    assert_eq!(rf_mg.seat_id, "seat-c");
}

// --- Sanity: LOCAL_SEAT_ID is "local" (the default seat namespace). ---

#[test]
fn contract_local_seat_id_constant() {
    assert_eq!(LOCAL_SEAT_ID, "local");
}

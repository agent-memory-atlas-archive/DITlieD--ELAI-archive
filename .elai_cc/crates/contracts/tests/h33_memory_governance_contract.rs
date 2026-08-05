//! H-33 contract tests — memory governance: forget/pin/unpin beacons + decay filter.
//!
//! Covers (off-agent artifacts, Layer-4 / R4, fail-closed per R1):
//!   (1) SCHEMA >= 211 and `memory_governance_event` in the SSB bijection.
//!   (2) soft_close sets valid_to_ms non-NULL (FactStore mutation correct).
//!   (3) A real MemoryGovernanceEvent::Forgotten row writes to SSB and round-trips.
//!       Missing row = FAIL (R1 / missing-evidence is a hard FAIL).
//!   (4) pin_filter_decay_skips_pinned_row: decay_tick with a non-empty pinned_ids
//!       set leaves the pinned fact's trust_score unchanged while a control fact
//!       decays measurably. This exercises the R9 ON-path smoke for
//!       ELAI_MEMORY_PIN_ENABLE. Two assertions must hold simultaneously.
//!   (5) resume_summary_flags_superseded_row: after inserting a fact and marking
//!       superseded_by, the row appears in the superseded set from list_current.
//!
//! SCHEMA_VERSION assertions use `>=` (not a global exact pin) per the
//! BRITTLE_GLOBAL_SCHEMA_PIN gotcha.
//!
//! Grounding:
//!   - crates/memory/src/store.rs (soft_close, list_current)
//!   - crates/memory/src/decay.rs (decay_tick with pinned_ids)
//!   - crates/orchestrator/src/ssb.rs (MemoryGovernanceEvent + SCHEMA_VERSION)

use std::collections::HashSet;
use std::sync::Arc;

use elai_core::clock::OsClock;
use elai_memory::{
    decay_tick, FactStore, FactType, MemoryFact, Provenance, Scope, TrustTier,
};
use elai_orchestrator::ssb::{
    ssb_action_type_from_str_public, MemoryGovernanceEventPayload, MemoryGovernanceKind,
    Ssb, SsbActionType, SsbConfig, SsbPayload, SCHEMA_VERSION_PUBLIC,
};
use elai_orchestrator::SsbReader;
use tempfile::TempDir;

const TEST_USER: &str = "default";

fn test_clock() -> Arc<dyn elai_core::clock::ClockProvider> {
    Arc::new(OsClock)
}

fn test_ssb(dir: &TempDir, session_id: &str) -> Arc<Ssb> {
    let db_path = dir.path().join("ssb.db");
    let cfg = SsbConfig {
        db_path,
        pool_size: Some(2),
        project_id: "h33-contract".into(),
        session_id: session_id.into(),
    };
    Arc::new(Ssb::new(cfg, test_clock()).expect("ssb open"))
}

fn make_fact(fact_id: &str, key: &str, trust_score: f32) -> MemoryFact {
    MemoryFact {
        fact_id: fact_id.into(),
        user_id: TEST_USER.into(),
        seat_id: "seat-h33-contract".into(),
        project_id: Some("proj-h33".into()),
        scope: Scope::Project,
        fact_type: FactType::Preference,
        key: key.into(),
        value: serde_json::json!({ "raw": key }),
        confidence: 0.9,
        trust_score,
        ingested_at_ms: 1_000,
        valid_from_ms: 1_000,
        valid_to_ms: None,
        decay_rate_per_day: 0.5,
        last_observed_at_ms: 1_000,
        evidence_pointers: vec!["cli:user_direct".into()],
        superseded_by: None,
        trust_tier: TrustTier::T0UserDirect,
        provenance: Provenance::user_direct("h33-contract"),
        last_validated_at_ms: None,
    }
}

// ---------------------------------------------------------------------------
// (1) SCHEMA and SSB bijection for memory_governance_event
// ---------------------------------------------------------------------------

#[test]
fn schema_version_is_at_least_211_and_action_type_bijection_holds() {
    assert!(
        SCHEMA_VERSION_PUBLIC >= 211,
        "H-33 requires SCHEMA_VERSION >= 211, got {SCHEMA_VERSION_PUBLIC}"
    );
    let at = ssb_action_type_from_str_public("memory_governance_event")
        .expect("from_str memory_governance_event");
    assert_eq!(at, SsbActionType::MemoryGovernanceEvent);
    assert_eq!(
        SsbActionType::MemoryGovernanceEvent.as_sql_str(),
        "memory_governance_event"
    );
}

// ---------------------------------------------------------------------------
// (2) soft_close sets valid_to_ms non-NULL
// ---------------------------------------------------------------------------

#[test]
fn soft_close_sets_valid_to_ms() {
    let mut store = FactStore::open_in_memory().expect("store");
    let f = make_fact("fact-h33-delete", "prefer_tracing", 0.8);
    store.insert(&f).expect("insert");

    let now_ms = 999_000_u128;
    store.soft_close("fact-h33-delete", now_ms).expect("soft_close");

    let reloaded = store.get("fact-h33-delete").expect("get").expect("row exists");
    assert_eq!(
        reloaded.valid_to_ms,
        Some(now_ms),
        "soft_close must set valid_to_ms to now_ms; got {:?}",
        reloaded.valid_to_ms
    );
}

// ---------------------------------------------------------------------------
// (3) Real MemoryGovernanceEvent::Forgotten SSB row writes and round-trips.
//     Missing row = FAIL (R1).
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn real_forgotten_governance_row_lands_and_round_trips() {
    let dir = TempDir::new().expect("tempdir");
    let ssb = test_ssb(&dir, "h33-forgotten");

    let payload = SsbPayload::MemoryGovernanceEvent(MemoryGovernanceEventPayload {
        fact_id: "fact-h33-contract".into(),
        kind: MemoryGovernanceKind::Forgotten,
        actor: "cli".into(),
        session_id: "h33-contract-session".into(),
        created_at_ms: 1_700_000_000_000,
        seat_id: "seat-h33-contract".into(),
    });

    let seq = ssb
        .write_payload("agent-h33", 0, payload, vec![])
        .await
        .expect("write_payload");
    assert!(seq > 0, "seq must be positive; got {seq}");

    // Read back via SsbReader::query_by_type (off-agent artifact, R4).
    let entries = ssb
        .query_by_type(SsbActionType::MemoryGovernanceEvent)
        .expect("query_by_type");
    assert!(
        !entries.is_empty(),
        "MemoryGovernanceEvent row must be present (R1: missing evidence = FAIL)"
    );

    let entry = &entries[0];
    let decoded: SsbPayload =
        rmp_serde::from_slice(&entry.payload).expect("rmp_serde decode");
    match decoded {
        SsbPayload::MemoryGovernanceEvent(p) => {
            assert_eq!(p.fact_id, "fact-h33-contract");
            assert_eq!(p.kind, MemoryGovernanceKind::Forgotten);
            assert_eq!(p.actor, "cli");
        }
        other => panic!("wrong payload variant: {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// (4) pin_filter_decay_skips_pinned_row
//     R9 ON-path smoke: with ELAI_MEMORY_PIN_ENABLE semantics, a pinned
//     fact's trust_score must be unchanged after decay_tick while the
//     control fact decays measurably.
// ---------------------------------------------------------------------------

#[test]
fn pin_filter_decay_skips_pinned_row() {
    let store = FactStore::open_in_memory().expect("store");

    // Insert two facts with the same trust_score and a non-trivial decay rate.
    let pinned = make_fact("fact-pinned", "pinned_key", 0.9);
    let control = make_fact("fact-control", "control_key", 0.9);
    store.insert(&pinned).expect("insert pinned");
    store.insert(&control).expect("insert control");

    // 30 days forward — enough for both facts to decay measurably.
    let t_now = 1_000u128 + 86_400_000 * 30;

    let mut pinned_ids = HashSet::new();
    pinned_ids.insert("fact-pinned".to_owned());

    let stats = decay_tick(&store, &pinned_ids, t_now).expect("decay_tick");
    // rows_examined counts ALL rows; rows_updated must be exactly 1 (the control).
    assert_eq!(stats.rows_examined, 2, "both rows must be examined");
    assert_eq!(
        stats.rows_updated, 1,
        "only the control row must be updated; pinned row must be skipped"
    );

    // Reload and check.
    let pinned_after = store.get("fact-pinned").expect("get pinned").expect("row");
    let control_after = store.get("fact-control").expect("get control").expect("row");

    assert!(
        (pinned_after.trust_score - 0.9).abs() < 1e-5,
        "pinned row trust_score must be unchanged (got {})",
        pinned_after.trust_score
    );
    assert!(
        control_after.trust_score < 0.9,
        "control row trust_score must have decayed (got {})",
        control_after.trust_score
    );
}

// ---------------------------------------------------------------------------
// (5) fail_toward_decay_with_empty_pin_set
//     With an empty pinned_ids set, both rows decay normally.
// ---------------------------------------------------------------------------

#[test]
fn fail_toward_decay_with_empty_pin_set() {
    let store = FactStore::open_in_memory().expect("store");
    let f1 = make_fact("fact-a", "key-a", 0.9);
    let f2 = make_fact("fact-b", "key-b", 0.9);
    store.insert(&f1).expect("insert f1");
    store.insert(&f2).expect("insert f2");

    let t_now = 1_000u128 + 86_400_000 * 30;
    let empty: HashSet<String> = HashSet::new();
    let stats = decay_tick(&store, &empty, t_now).expect("decay_tick");

    assert_eq!(stats.rows_examined, 2);
    assert_eq!(
        stats.rows_updated, 2,
        "with empty pin set, both rows must decay (fail-toward-decay)"
    );
}

// ---------------------------------------------------------------------------
// (6) supersede_closes_old_row_and_inserts_new
//     After supersede(), the old row has valid_to_ms set (closed) AND
//     superseded_by pointing to the new fact; the new fact is valid.
//     This verifies the store API used by the resume-summary path.
// ---------------------------------------------------------------------------

#[test]
fn supersede_closes_old_row_and_inserts_new() {
    let mut store = FactStore::open_in_memory().expect("store");

    let original = make_fact("fact-old-h33", "key-supersede", 0.8);
    store.insert(&original).expect("insert original");

    let mut replacement = make_fact("fact-new-h33", "key-supersede", 0.95);
    replacement.user_id = TEST_USER.into();
    let now_ms = 999_000u128;
    store.supersede("fact-old-h33", &replacement, now_ms).expect("supersede");

    // Old row must be closed and have superseded_by set.
    let old_row = store.get("fact-old-h33").expect("get old").expect("row exists");
    assert_eq!(
        old_row.valid_to_ms,
        Some(now_ms),
        "old row valid_to_ms must be set after supersede"
    );
    assert_eq!(
        old_row.superseded_by.as_deref(),
        Some("fact-new-h33"),
        "old row superseded_by must point to new fact"
    );

    // New row must be valid.
    let new_row = store.get("fact-new-h33").expect("get new").expect("row exists");
    assert!(new_row.valid_to_ms.is_none(), "new row must be valid (valid_to_ms IS NULL)");
    // list_current must include the new row.
    let active = store.list_current(TEST_USER, None, None, 100).expect("list_current");
    let active_ids: Vec<_> = active.iter().map(|f| f.fact_id.as_str()).collect();
    assert!(
        active_ids.contains(&"fact-new-h33"),
        "new fact must appear in list_current; got {active_ids:?}"
    );
    assert!(
        !active_ids.contains(&"fact-old-h33"),
        "old fact must NOT appear in list_current after supersede; got {active_ids:?}"
    );
}

// ---------------------------------------------------------------------------
// (7) list_including_closed_returns_superseded_row
//     After supersede(old, new): list_including_closed MUST return the old row
//     AND it must have valid_to_ms set (= closed/superseded marker).
//     This is the BLOCK-1/BLOCK-2/BLOCK-3 anti-regression gate: the
//     "show superseded/history" feature was wire-dark because the three
//     production call sites used list_current (WHERE valid_to_ms IS NULL),
//     which always filtered out the superseded row.
// ---------------------------------------------------------------------------

#[test]
fn list_including_closed_returns_superseded_row() {
    let mut store = FactStore::open_in_memory().expect("store");

    let original = make_fact("fact-h33-vis-old", "key-vis", 0.8);
    store.insert(&original).expect("insert original");

    let mut replacement = make_fact("fact-h33-vis-new", "key-vis", 0.95);
    replacement.user_id = TEST_USER.into();
    let now_ms = 888_000u128;
    store
        .supersede("fact-h33-vis-old", &replacement, now_ms)
        .expect("supersede");

    // (a) list_including_closed must return the old row AND it must be marked
    //     closed (valid_to_ms IS NOT NULL).
    let all = store
        .list_including_closed(TEST_USER, None, 100)
        .expect("list_including_closed");
    let old = all
        .iter()
        .find(|f| f.fact_id == "fact-h33-vis-old")
        .expect("superseded row must appear in list_including_closed (BLOCK-1/2/3 guard)");
    assert!(
        old.valid_to_ms.is_some(),
        "superseded row must have valid_to_ms set; got {:?}",
        old.valid_to_ms
    );

    // (b) The [SUPERSEDED] path: a resume-summary-equivalent filter
    //     (valid_to_ms.is_some()) must capture the old row and NOT the new row.
    let superseded: Vec<_> = all.iter().filter(|f| f.valid_to_ms.is_some()).collect();
    let superseded_ids: Vec<_> = superseded.iter().map(|f| f.fact_id.as_str()).collect();
    assert!(
        superseded_ids.contains(&"fact-h33-vis-old"),
        "superseded row must appear in the superseded filter; got {superseded_ids:?}"
    );
    assert!(
        !superseded_ids.contains(&"fact-h33-vis-new"),
        "active row must NOT appear in the superseded filter; got {superseded_ids:?}"
    );
}

// ---------------------------------------------------------------------------
// (8) resume_summary_contains_superseded_flag
//     Mirrors build_memory_resume_summary logic: after supersede(),
//     scanning via list_including_closed and keying on valid_to_ms.is_some()
//     must produce a [SUPERSEDED] label for the old row.
// ---------------------------------------------------------------------------

#[test]
fn resume_summary_contains_superseded_flag() {
    let mut store = FactStore::open_in_memory().expect("store");

    let original = make_fact("fact-h33-sum-old", "key-sum", 0.7);
    store.insert(&original).expect("insert original");

    let mut replacement = make_fact("fact-h33-sum-new", "key-sum", 0.9);
    replacement.user_id = TEST_USER.into();
    let now_ms = 777_000u128;
    store
        .supersede("fact-h33-sum-old", &replacement, now_ms)
        .expect("supersede");

    // Replicate build_memory_resume_summary logic (BLOCK-2 path).
    let all_rows = store
        .list_including_closed(TEST_USER, None, 100)
        .expect("list_including_closed");
    let superseded: Vec<_> = all_rows.iter().filter(|f| f.valid_to_ms.is_some()).collect();

    let mut buf = String::new();
    for f in &superseded {
        let sup_target = f.superseded_by.as_deref().unwrap_or("closed");
        buf.push_str(&format!(
            "  [SUPERSEDED -> {sup_target}]  {} | type={} | key={}\n",
            f.fact_id,
            f.fact_type.as_str(),
            f.key,
        ));
    }

    assert!(
        buf.contains("[SUPERSEDED"),
        "resume summary output must contain [SUPERSEDED flag; got: {buf:?}"
    );
    assert!(
        buf.contains("fact-h33-sum-old"),
        "resume summary must name the old fact_id; got: {buf:?}"
    );
}

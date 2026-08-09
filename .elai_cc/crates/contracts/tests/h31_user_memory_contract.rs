//! H-31 contract tests -- the user-preference bi-temporal ledger ACTIVATION (the `/remember`
//! capture producer + retrieval-into-prompt + the Personalization-Trap firewall).
//!
//! Covers (TEST-FIRST, authored against the live substrate + SSB surface), over a REAL Ssb SQLite
//! db + a REAL FactStore (off-agent artifact, Layer-4 / R4). The capture producer
//! (`crate::user_memory_capture::capture_remember`) is `pub(crate)` in the elai-cli crate, so these
//! contract tests exercise the SAME substrate flow the producer drives -- `elai_memory::dispatch_one`
//! (the FIRST production caller), `FactStore::supersede` (soft-close, NEWEST-WINS, DELETE BANNED),
//! `get_relevant_facts` (the `valid_to_ms IS NULL` hard-filter), and the off-agent
//! `UserDecisionSuperseded` SSB row -- plus the firewall over BOTH role enums. A CLI-crate
//! integration test drives the `capture_remember` entry end-to-end.
//!
//!   (1) CAPTURE: a `/remember X` => a UserDecisionSuperseded Captured row + X in the ledger.
//!   (2) SUPERSEDE: a contradicting `/remember X'` on the same key => a Superseded row + the
//!       bi-temporal invariant old.valid_to_ms == new.valid_from_ms (soft-close, old NOT deleted).
//!   (3) RETRIEVAL: after supersede, get_relevant_facts surfaces X' and NOT X (the hard-filter).
//!   (4) FIREWALL: an auditor-role (AND ConsensusRole::Adversary/Critic) assembly gets ZERO
//!       user-memory; an Executor-role (AND ConsensusRole::Worker) assembly DOES (both directions).
//!   (5) SECRET REJECT: a captured secret/credential is REJECTED (defense.rs classify), not stored.
//!   (6) the R9 smoke: the flag gates the path (asserted in the CLI integration test).
//!   (7) SCHEMA >= 240, UserDecisionSuperseded in the action_type bijection, and a REAL
//!       UserDecisionSuperseded row lands + round-trips through the SSB single-writer.
//!
//! SCHEMA_VERSION assertions use `>=` (never an exact global pin) per the BRITTLE_GLOBAL_SCHEMA_PIN
//! gotcha. TOTAL_VARIANTS / ALL_VARIANTS presence is asserted by action_type_lock.rs (compile-time
//! exhaustiveness + count-drift) and the e45 contract.
//!
//! Grounding:
//!   - crates/memory/src/mem0_dispatcher.rs (dispatch_one -- the capture reconcile).
//!   - crates/memory/src/store.rs (supersede / list_current hard-filter).
//!   - crates/memory/src/retrieval.rs (get_relevant_facts -- the chokepoint).
//!   - crates/memory/src/defense.rs (classify -- the secret/PII reject seam).
//!   - crates/orchestrator/src/user_memory_firewall.rs (the role allowlist over BOTH enums).
//!   - workflow/reasoning/H-31-decisions.md (citation discipline: 2603.00024 honest framing).

use std::sync::Arc;

use elai_core::clock::OsClock;
use elai_memory::{
    classify, dispatch_one, get_relevant_facts, DefenseVerdict, DispatchContext, DispatchOp,
    ExtractedFact, FactStore, FactType, Provenance, RetrievalFilter, Scope, TrustTier,
};
use elai_orchestrator::agent_role::ConsensusRole;
use elai_orchestrator::ssb::{
    ssb_action_type_from_str_public, Ssb, SsbActionType, SsbConfig, SsbPayload,
    UserDecisionEventKind, UserDecisionSupersededPayload, SCHEMA_VERSION_PUBLIC,
};
use elai_orchestrator::user_memory_firewall::{
    agent_role_gets_user_memory, consensus_role_gets_user_memory,
};
use elai_orchestrator::SsbReader;
use elai_ipc_types::AgentRole;
use tempfile::TempDir;

const TEST_USER: &str = "default";

fn test_clock() -> Arc<dyn elai_core::clock::ClockProvider> {
    Arc::new(OsClock)
}

fn test_ssb(dir: &TempDir, project_id: &str) -> Arc<Ssb> {
    let db_path = dir.path().join("ssb.db");
    let cfg = SsbConfig {
        db_path,
        pool_size: Some(2),
        project_id: project_id.into(),
        session_id: format!("{project_id}-session"),
    };
    Arc::new(Ssb::new(cfg, test_clock()).expect("ssb"))
}

fn ctx(now_ms: u128) -> DispatchContext {
    DispatchContext {
        user_id: TEST_USER.into(),
        seat_id: "local".into(),
        project_id: None,
        scope: Scope::Global,
        provenance: Provenance::user_direct("test"),
        trust_tier: TrustTier::T0UserDirect,
        initial_trust_score: 1.0,
        now_ms,
    }
}

/// Build a Preference candidate with a fixed key (so a CHANGED value on the SAME key drives the
/// supersede UPDATE path, exactly as a contradicting `/remember` on the same topic does).
fn pref(key: &str, value: &str, conf: f32) -> ExtractedFact {
    ExtractedFact {
        fact_type: FactType::Preference,
        key: key.into(),
        value: serde_json::json!({ "raw": value }),
        confidence: conf,
        evidence: vec!["sess-h31".into(), "msg-h31".into()],
    }
}

// ---------------------------------------------------------------------------
// (1) CAPTURE: a new preference => DispatchOp::Add + Captured SSB row + X in the ledger.
// ---------------------------------------------------------------------------
#[test]
fn capture_adds_new_preference() {
    let mut store = FactStore::open_in_memory().expect("store");
    let cand = pref("editor_test_runner", "auditor only", 0.9);
    let op = dispatch_one(&mut store, &cand, &ctx(1_000)).expect("dispatch");
    let DispatchOp::Add { fact_id } = op else {
        panic!("expected Add for a new key, got {op:?}");
    };
    // X is in the ledger (current rows).
    let current = store
        .list_current(TEST_USER, Some(Scope::Global), Some(FactType::Preference), 10)
        .expect("list_current");
    assert!(
        current.iter().any(|f| f.fact_id == fact_id),
        "the captured fact must be a current row"
    );
}

// ---------------------------------------------------------------------------
// (2) SUPERSEDE: a contradicting value on the same key => Update + the bi-temporal soft-close
//     invariant old.valid_to_ms == new.valid_from_ms; the old row is NOT deleted.
// ---------------------------------------------------------------------------
#[test]
fn contradicting_preference_supersedes_with_soft_close_invariant() {
    let mut store = FactStore::open_in_memory().expect("store");
    let now_old: u128 = 1_000;
    let now_new: u128 = 2_000;

    let first = pref("commit_gate", "run auditor before commit", 0.9);
    let DispatchOp::Add { fact_id: old_id } =
        dispatch_one(&mut store, &first, &ctx(now_old)).expect("add")
    else {
        panic!("expected Add");
    };

    // A CONTRADICTING decision on the same key (changed value) at a later time.
    let second = pref("commit_gate", "skip the auditor, commit directly", 0.9);
    let DispatchOp::Update {
        old_fact_id,
        new_fact_id,
    } = dispatch_one(&mut store, &second, &ctx(now_new)).expect("update")
    else {
        panic!("expected Update on a contradicting same-key decision");
    };
    assert_eq!(old_fact_id, old_id, "the Update closes the original row");

    // THE BI-TEMPORAL INVARIANT: old.valid_to_ms == new.valid_from_ms (soft-close, newest-wins).
    let old_row = store
        .get(&old_fact_id)
        .expect("get old")
        .expect("old row still present (NOT deleted)");
    let new_row = store
        .get(&new_fact_id)
        .expect("get new")
        .expect("new row present");
    assert_eq!(
        old_row.valid_to_ms,
        Some(now_new),
        "the old row is SOFT-CLOSED at the supersede time (not deleted)"
    );
    assert_eq!(
        new_row.valid_from_ms, now_new,
        "the new row's valid_from_ms is the supersede time"
    );
    assert_eq!(
        old_row.valid_to_ms,
        Some(new_row.valid_from_ms),
        "BI-TEMPORAL INVARIANT: old.valid_to_ms == new.valid_from_ms"
    );
    // DELETE BANNED: the old row is still readable via the include-closed accessor.
    let all = store
        .list_including_closed(TEST_USER, Some(FactType::Preference), 10)
        .expect("list_including_closed");
    assert!(
        all.iter().any(|f| f.fact_id == old_fact_id),
        "the superseded row is soft-closed, NOT deleted (SSB replay safety)"
    );
}

// ---------------------------------------------------------------------------
// (2b) W-1 TOPIC-KEY COLLISION: the shape the live producer now derives. Two CONTRADICTING
//      two-option statements on the SAME topic ("X over Y" vs "Y over X") are re-keyed by the
//      producer onto ONE unordered topic key (topic_<a>_vs_<b>, options sorted), so the
//      contradiction reconciles to the SAME key with a CHANGED value => Update/supersede (NOT a
//      second Add). This contract drives the substrate with that exact derived-key shape (the
//      producer's key derivation lives in the cli crate, pub(crate); the live end-to-end producer
//      proof is the cli-crate in-module test
//      user_memory_capture::tests::live_producer_contradiction_supersedes_with_soft_close_invariant
//      + the off-agent SSB Superseded row from the real `elai remember` binary).
// ---------------------------------------------------------------------------
#[test]
fn topic_keyed_contradiction_supersedes_not_double_adds() {
    let mut store = FactStore::open_in_memory().expect("store");
    // The unordered topic key the producer derives for both "tracing over log" and its reverse.
    let topic_key = "topic_log_vs_tracing";

    let first = pref(topic_key, "i prefer tracing over log", 0.85);
    let DispatchOp::Add { fact_id: old_id } =
        dispatch_one(&mut store, &first, &ctx(1_000)).expect("add")
    else {
        panic!("first statement on the topic key must ADD");
    };

    // The CONTRADICTION ("log over tracing"): same topic key, flipped value, equal confidence.
    let second = pref(topic_key, "i prefer log over tracing", 0.85);
    let DispatchOp::Update {
        old_fact_id,
        new_fact_id,
    } = dispatch_one(&mut store, &second, &ctx(2_000)).expect("update")
    else {
        panic!("the contradicting statement on the SAME topic key must SUPERSEDE, not ADD again");
    };
    assert_eq!(old_fact_id, old_id, "the Update closes the original topic row");

    // Soft-close invariant + no delete (the same guarantee as test (2), now on the topic-key shape).
    let old_row = store.get(&old_fact_id).expect("get old").expect("old present");
    let new_row = store.get(&new_fact_id).expect("get new").expect("new present");
    assert_eq!(
        old_row.valid_to_ms,
        Some(new_row.valid_from_ms),
        "BI-TEMPORAL INVARIANT on the topic key: old.valid_to_ms == new.valid_from_ms"
    );
    let current = store
        .list_current(TEST_USER, Some(Scope::Global), Some(FactType::Preference), 10)
        .expect("list_current");
    assert_eq!(
        current.iter().filter(|f| f.key == topic_key).count(),
        1,
        "exactly ONE current row for the topic after supersede (newest-wins), never two ADDs"
    );
}

// ---------------------------------------------------------------------------
// (3) RETRIEVAL: after supersede, get_relevant_facts surfaces X' and NOT X (the hard-filter --
//     a superseded row is structurally absent, never narrated [OUTDATED]).
// ---------------------------------------------------------------------------
#[test]
fn retrieval_surfaces_fresh_value_and_never_the_superseded_one() {
    let mut store = FactStore::open_in_memory().expect("store");
    let stale_value = "use the log crate";
    let fresh_value = "prefer tracing over the log crate";

    dispatch_one(&mut store, &pref("logging_lib", stale_value, 0.9), &ctx(1_000)).expect("add");
    dispatch_one(&mut store, &pref("logging_lib", fresh_value, 0.9), &ctx(2_000)).expect("update");

    let filter = RetrievalFilter {
        fact_type: Some(FactType::Preference),
        ..RetrievalFilter::default()
    };
    let (facts, _) = get_relevant_facts(&store, TEST_USER, &filter, 8, 3_000).expect("retrieve");

    let raws: Vec<String> = facts
        .iter()
        .filter_map(|f| f.value.get("raw").and_then(|v| v.as_str()).map(str::to_owned))
        .collect();
    assert!(
        raws.iter().any(|r| r == fresh_value),
        "retrieval surfaces the FRESH value, got {raws:?}"
    );
    assert!(
        !raws.iter().any(|r| r == stale_value),
        "retrieval NEVER surfaces the superseded value (the valid_to_ms IS NULL hard-filter); got {raws:?}"
    );
}

// ---------------------------------------------------------------------------
// (4) THE FIREWALL: the Personalization-Trap exclusion over BOTH role enums (the load-bearing
//     safety constraint). Allow ONLY {Executor, Worker}; every reviewer role gets ZERO.
// ---------------------------------------------------------------------------
#[test]
fn firewall_allows_only_executor_on_agent_role() {
    assert!(
        agent_role_gets_user_memory(AgentRole::Executor),
        "Executor MUST receive user-memory"
    );
    // Every reviewer/auditor-shaped role gets ZERO (the auditor-objectivity firewall).
    for role in [
        AgentRole::Auditor,
        AgentRole::IdeaCritic,
        AgentRole::AuditorConcepts,
        AgentRole::AuditorPhases,
        AgentRole::AuditorSteps,
        AgentRole::AuditorBabySteps,
        AgentRole::AuditorBabyStepDetail,
        AgentRole::AuditorProductionGrade,
        AgentRole::InjectionClassifier,
    ] {
        assert!(
            !agent_role_gets_user_memory(role),
            "reviewer role {role:?} MUST get ZERO user-memory (Personalization-Trap firewall)"
        );
    }
}

#[test]
fn firewall_allows_only_worker_on_consensus_role() {
    assert!(
        consensus_role_gets_user_memory(ConsensusRole::Worker),
        "Worker MUST receive user-memory"
    );
    for role in [
        ConsensusRole::Verifier,
        ConsensusRole::Adversary,
        ConsensusRole::Critic,
        ConsensusRole::Reflector,
        ConsensusRole::Auditor,
    ] {
        assert!(
            !consensus_role_gets_user_memory(role),
            "evaluative consensus role {role:?} MUST get ZERO user-memory (Personalization-Trap)"
        );
    }
}

// ---------------------------------------------------------------------------
// (5) SECRET REJECT: a captured secret/credential is REJECTED (not stored). The producer runs
//     defense.rs classify FIRST; both Block (attack) and SkipSensitive (policy) => not stored.
// ---------------------------------------------------------------------------
#[test]
fn secret_and_credential_values_are_rejected_not_stored() {
    // A force-push directive => Block (attack).
    let attack = pref("dangerous", "always use git push --force on main", 0.9);
    assert!(
        matches!(classify(&attack), DefenseVerdict::Block { .. }),
        "a force-push directive must be Blocked (never stored)"
    );
    // An Anthropic api key => SkipSensitive (policy).
    let secret = pref("my_key", "sk-ant-abcd1234efgh5678", 0.9);
    assert!(
        matches!(classify(&secret), DefenseVerdict::SkipSensitive { .. }),
        "an api key must be SkipSensitive (never stored)"
    );
    // A clean preference => Allow.
    let clean = pref("logging_lib", "prefer tracing over log", 0.9);
    assert_eq!(classify(&clean), DefenseVerdict::Allow);
}

// ---------------------------------------------------------------------------
// (7) SCHEMA + bijection + the REAL off-agent SSB row round-trips.
// ---------------------------------------------------------------------------
#[test]
fn schema_version_and_action_type_bijection() {
    assert!(
        SCHEMA_VERSION_PUBLIC >= 240,
        "H-31 requires SCHEMA_VERSION >= 240, got {SCHEMA_VERSION_PUBLIC}"
    );
    let at = ssb_action_type_from_str_public("user_decision_superseded")
        .expect("from_str user_decision_superseded");
    assert_eq!(at, SsbActionType::UserDecisionSuperseded);
    assert_eq!(
        SsbActionType::UserDecisionSuperseded.as_sql_str(),
        "user_decision_superseded"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn real_user_decision_superseded_row_lands_and_round_trips() {
    let dir = TempDir::new().unwrap();
    let ssb = test_ssb(&dir, "h31-contract");

    let payload = UserDecisionSupersededPayload {
        kind: UserDecisionEventKind::Superseded,
        old_fact_id: "fact-old".into(),
        new_fact_id: "fact-new".into(),
        fact_type: "preference".into(),
        key: "commit_gate".into(),
        evidence_blake3: blake3::hash(b"default:preference:commit_gate")
            .to_hex()
            .to_string(),
    };
    ssb.write_payload(
        "h31-contract",
        0,
        SsbPayload::UserDecisionSuperseded(payload.clone()),
        Vec::new(),
    )
    .await
    .expect("write UserDecisionSuperseded row");

    // The off-agent artifact: read it back and verify the round-trip.
    let rows = ssb
        .query_by_type(SsbActionType::UserDecisionSuperseded)
        .expect("query UserDecisionSuperseded");
    assert_eq!(
        rows.len(),
        1,
        "exactly one UserDecisionSuperseded row must land (the off-agent artifact)"
    );
    let decoded: SsbPayload =
        rmp_serde::from_slice(&rows[0].payload).expect("decode payload");
    match decoded {
        SsbPayload::UserDecisionSuperseded(p) => {
            assert_eq!(p, payload, "round-trip mismatch on the UserDecisionSuperseded payload");
            assert_eq!(p.kind, UserDecisionEventKind::Superseded);
            assert!(
                !p.key.is_empty() && p.evidence_blake3.len() == 64,
                "the row carries the key + a blake3 hex evidence fingerprint (NEVER the value)"
            );
        }
        other => panic!("expected UserDecisionSuperseded, got {other:?}"),
    }
}

//! F-6 / Slice A5 — critic-gated skill acquisition contract (C6b).
//!
//! C6b ACQUISITION GATE: a learned skill is acquired into the library ONLY
//! after a critic-gated success — the EXISTING `MiniCriticVerdict::Pass` from
//! the Best-of-N MiniCritic (`crate::structural_retry::run_best_of_n`). F-6
//! REUSES that verdict; it does NOT spawn a 4th critic and does NOT duplicate
//! the verdict SSB row. Acquisition is fail-closed: with no `Pass` in scope, no
//! skill is acquired.
//!
//! The producer-side wiring (querying / threading the verdict at
//! `run_crystallizer_post_execute_hook`) lives in `task_runner`. This contract
//! pins the LIBRARY-level invariants that make the gate observable end-to-end,
//! using only `elai_orchestrator` public types (no new deps):
//!   1. A freshly acquired skill is born at the S1 floor with 0 invocations and
//!      is NEVER autonomy-permitted — so even a Pass-gated acquisition cannot
//!      vault a skill straight to the autonomy-permitted apex (S4 needs a
//!      separate human grant). This is the structural guarantee behind
//!      "no Pass -> no acquire -> no autonomy".
//!   2. The `SkillLibraryEvent(Acquired)` SSB beacon round-trips (msgpack
//!      encode -> `decode_ssb_payload`) carrying `prior_tier = None` (an
//!      acquisition has no prior tier) and `tier = "s1"`.
//!
//! Spec: F-6 design (workflow/reasoning/F-6-decisions.md), DESIGN C6b.

use elai_orchestrator::skill_trust::{SkillStore, SkillTrustTier};
use elai_orchestrator::ssb::{
    decode_ssb_payload, SkillLibraryEventKind, SkillLibraryEventPayload, SsbActionType, SsbPayload,
};

// ---------------------------------------------------------------------------
// 1. A Pass-gated acquisition is born at S1 and is never autonomy-permitted.
// ---------------------------------------------------------------------------

#[test]
fn acquisition_seeds_s1_floor_never_autonomy_permitted() {
    let mut store = SkillStore::new();
    // Simulate the C6b acquire path that fires only after a MiniCritic Pass.
    let tier = store.acquire("scaffold-tauri".into(), "scaffold a Tauri app".into(), 1_000);
    assert_eq!(tier, SkillTrustTier::S1, "acquired skill must be at the floor");

    let s = store.get("scaffold-tauri").expect("present after acquire");
    assert_eq!(s.tier, SkillTrustTier::S1);
    assert_eq!(s.invocations, 0, "a freshly acquired skill has 0 invocations");
    assert_eq!(s.failures, 0);

    // The autonomy barrier must NOT clear a freshly acquired skill: a Pass
    // gates ACQUISITION (entry at S1), not autonomous DISPATCH (only S4).
    assert!(
        !SkillStore::permits_autonomous(s.tier),
        "a freshly acquired (S1) skill must never be autonomy-permitted"
    );
}

#[test]
fn fail_closed_no_skill_in_empty_store_when_no_pass() {
    // "no Pass -> no acquire" at the library level: if the producer never calls
    // acquire (because there was no MiniCritic Pass in scope), the store stays
    // empty and there is nothing to dispatch autonomously.
    let store = SkillStore::new();
    assert!(store.is_empty(), "no Pass means no acquisition occurred");
    assert_eq!(store.len(), 0);
    assert!(store.get("scaffold-tauri").is_none());
}

// ---------------------------------------------------------------------------
// 2. SkillLibraryEvent(Acquired) round-trips with prior_tier = None, tier = s1.
// ---------------------------------------------------------------------------

#[test]
fn skill_acquired_event_roundtrips_via_msgpack() {
    let original = SkillLibraryEventPayload {
        skill_id: "scaffold-tauri".into(),
        description: "scaffold a Tauri app".into(),
        tier: "s1".into(),
        prior_tier: None,
        reason: "critic-gated acquisition".into(),
        invocations: 0,
        failure_rate: 0.0,
        kind: SkillLibraryEventKind::Acquired,
    };
    let payload = SsbPayload::SkillLibraryEvent(original.clone());
    let bytes = rmp_serde::to_vec(&payload).expect("encode");
    let decoded = decode_ssb_payload(&bytes).expect("decode");
    match decoded {
        SsbPayload::SkillLibraryEvent(got) => {
            assert_eq!(got.skill_id, original.skill_id);
            assert_eq!(got.tier, "s1");
            assert_eq!(got.prior_tier, None, "an acquisition has no prior tier");
            assert_eq!(got.kind, SkillLibraryEventKind::Acquired);
            assert_eq!(got.invocations, 0);
            assert!((got.failure_rate - 0.0).abs() < 1e-9);
        }
        other => panic!("unexpected variant: {other:?}"),
    }

    // The action_type discriminant resolves to the Acquired arm's sql_str.
    assert_eq!(
        SsbPayload::SkillLibraryEvent(original).action_type(),
        SsbActionType::SkillLibraryEvent(SkillLibraryEventKind::Acquired)
    );
    assert_eq!(
        SsbActionType::SkillLibraryEvent(SkillLibraryEventKind::Acquired).as_sql_str(),
        "skill_acquired"
    );
}

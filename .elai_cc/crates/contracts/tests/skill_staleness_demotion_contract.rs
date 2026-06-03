//! F-6 / Slice A5 — staleness demotion contract (C6c).
//!
//! C6c STALENESS DEMOTION: on a scheduler tick, a learned skill whose staleness
//! score (`(now - last_verified) / SKILL_EXPECTED_LIFETIME_SECS`) has crossed
//! `SKILL_STALE_THRESHOLD` is knocked down exactly ONE rung, with S1 as the
//! floor. Each demotion emits a `SkillLibraryEvent(DemotedStale)` beacon.
//!
//! This contract pins, using only `elai_orchestrator` public types (no new
//! deps):
//!   1. A skill aged past `SKILL_STALE_THRESHOLD` demotes exactly one rung,
//!      and an already-floor (S1) skill stays put (the floor invariant).
//!   2. A single tick demotes at most one rung (last_verified refreshes).
//!   3. All three `SkillLibraryEventKind` arms (Acquired / TrustPromoted /
//!      DemotedStale) round-trip through msgpack + the SSB action_type vocab.
//!
//! Spec: F-6 design (workflow/reasoning/F-6-decisions.md), DESIGN C6c.

use elai_orchestrator::skill_trust::{
    LearnedSkill, SkillStore, SkillTrustTier, SKILL_EXPECTED_LIFETIME_SECS, SKILL_STALE_THRESHOLD,
};
use elai_orchestrator::ssb::{
    decode_ssb_payload, SkillLibraryEventKind, SkillLibraryEventPayload, SsbActionType, SsbPayload,
};

// A `now` value guaranteed to be at/above one full expected-lifetime span,
// so any skill verified at t=0 is stale (score >= SKILL_STALE_THRESHOLD).
fn stale_now() -> u64 {
    // One full lifetime span plus a margin. SKILL_STALE_THRESHOLD is 1.0, so a
    // span >= SKILL_EXPECTED_LIFETIME_SECS gives score >= 1.0.
    (SKILL_EXPECTED_LIFETIME_SECS as u64) + 1
}

// ---------------------------------------------------------------------------
// 1 + 2. Demotion drops exactly one rung with an S1 floor, one rung per tick.
// ---------------------------------------------------------------------------

#[test]
fn stale_skill_demotes_one_rung_with_s1_floor() {
    let mut store = SkillStore::new();
    // Seed a high-tier skill verified at t=0 (so it ages out) and a floor skill.
    store.upsert(LearnedSkill {
        id: "high".into(),
        description: "well-exercised but unverified".into(),
        tier: SkillTrustTier::S3,
        invocations: 9,
        failures: 0,
        acquired_at: 0,
        last_verified: 0,
    });
    store.upsert(LearnedSkill {
        id: "floor".into(),
        description: "already at the floor".into(),
        tier: SkillTrustTier::S1,
        invocations: 1,
        failures: 0,
        acquired_at: 0,
        last_verified: 0,
    });

    let now = stale_now();
    // Sanity: both are genuinely stale at `now`.
    assert!(
        SkillStore::staleness_score(store.get("high").expect("present"), now)
            >= SKILL_STALE_THRESHOLD
    );

    let demoted = store.demote_if_stale(now);

    // Only the S3 skill demotes (one rung -> S2). The S1 floor skill is stale
    // but cannot drop further, so it is NOT in the demoted list.
    assert_eq!(demoted.len(), 1, "exactly one skill demotes");
    assert_eq!(demoted[0].id, "high");
    assert_eq!(demoted[0].prior_tier, SkillTrustTier::S3);
    assert_eq!(demoted[0].new_tier, SkillTrustTier::S2);

    assert_eq!(store.get("high").expect("present").tier, SkillTrustTier::S2);
    assert_eq!(
        store.get("floor").expect("present").tier,
        SkillTrustTier::S1,
        "S1 is the floor — a stale S1 skill stays at S1"
    );
}

#[test]
fn single_tick_demotes_at_most_one_rung() {
    let mut store = SkillStore::new();
    store.upsert(LearnedSkill {
        id: "apex".into(),
        description: "apex, long unverified".into(),
        tier: SkillTrustTier::S4,
        invocations: 20,
        failures: 0,
        acquired_at: 0,
        last_verified: 0,
    });

    let now = stale_now();
    let first = store.demote_if_stale(now);
    assert_eq!(first.len(), 1);
    assert_eq!(first[0].new_tier, SkillTrustTier::S3, "S4 -> S3 in one tick");
    // last_verified refreshed to `now`, so an immediate re-tick does NOT
    // double-demote.
    let second = store.demote_if_stale(now);
    assert!(
        second.is_empty(),
        "a second tick at the same instant must not double-demote"
    );
    assert_eq!(store.get("apex").expect("present").tier, SkillTrustTier::S3);
}

// ---------------------------------------------------------------------------
// 3. All three SkillLibraryEventKind arms round-trip through SSB.
// ---------------------------------------------------------------------------

#[test]
fn all_three_kinds_roundtrip_via_msgpack_and_action_type() {
    let cases = [
        (
            SkillLibraryEventKind::Acquired,
            "skill_acquired",
            None,
            "s1",
        ),
        (
            SkillLibraryEventKind::TrustPromoted,
            "skill_trust_promoted",
            Some("s1".to_owned()),
            "s2",
        ),
        (
            SkillLibraryEventKind::DemotedStale,
            "skill_demoted_stale",
            Some("s3".to_owned()),
            "s2",
        ),
    ];

    for (kind, sql, prior, tier) in cases {
        let original = SkillLibraryEventPayload {
            skill_id: "high".into(),
            description: "round-trip case".into(),
            tier: tier.to_owned(),
            prior_tier: prior.clone(),
            reason: "round-trip".into(),
            invocations: 4,
            failure_rate: 0.0,
            kind,
        };
        let payload = SsbPayload::SkillLibraryEvent(original.clone());

        // msgpack round-trip through the public decoder.
        let bytes = rmp_serde::to_vec(&payload).expect("encode");
        let decoded = decode_ssb_payload(&bytes).expect("decode");
        match decoded {
            SsbPayload::SkillLibraryEvent(got) => {
                assert_eq!(got.kind, kind);
                assert_eq!(got.tier, tier);
                assert_eq!(got.prior_tier, prior);
            }
            other => panic!("unexpected variant for {kind:?}: {other:?}"),
        }

        // action_type discriminant + sql_str vocabulary.
        let action = SsbActionType::SkillLibraryEvent(kind);
        assert_eq!(payload.action_type(), action);
        assert_eq!(action.as_sql_str(), sql);
    }
}

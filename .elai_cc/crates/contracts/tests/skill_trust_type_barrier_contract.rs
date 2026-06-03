//! F-6 / Slice A5 — SkillTrustTier type-barrier contract.
//!
//! P0 TYPE BARRIER (three trust ladders stay namespace-isolated):
//!   * skill ladder  -> `elai_orchestrator::skill_trust::SkillTrustTier` (S1..S4)
//!   * memory ladder  -> `elai_memory::TrustTier` (T0..T3)
//!   * reflection      -> `ReflectionTrustTier` (the local-enum precedent)
//!
//! There is intentionally NO `From`/`Into` conversion between the skill ladder
//! and the memory ladder in either direction. The COMPILE-TIME enforcement is
//! the `compile_fail` doc-test in `crates/orchestrator/src/skill_trust.rs`
//! (module docstring, lines ~31-36): a `SkillTrustTier::S4.into()` to
//! `elai_memory::TrustTier` MUST fail to compile. That doc-test runs under the
//! orchestrator crate's `cargo test --doc` and is the authoritative bridge ban.
//!
//! This contract pins the RUNTIME-observable half of the barrier using ONLY
//! `elai_orchestrator` public types (the contracts crate does NOT depend on
//! `elai-memory`, so importing `memory::TrustTier` here would be a new dep —
//! banned). It asserts:
//!   1. `permits_autonomous` is true for exactly one rung (S4) and false for
//!      the other three.
//!   2. The skill ladder's wire vocabulary (`s1`..`s4`) is lowercase-`s`
//!      prefixed and disjoint from the memory ladder's documented uppercase-`T`
//!      vocabulary (`T0`..`T3`), so the two never collide on the SSB wire even
//!      though both are "trust tiers".
//!
//! Spec: F-6 design (workflow/reasoning/F-6-decisions.md), DESIGN P0 BARRIER.

use elai_orchestrator::skill_trust::{SkillStore, SkillTrustTier};
use elai_orchestrator::autonomous_gate::skill_permits_autonomous;

// ---------------------------------------------------------------------------
// 1. permits_autonomous: true ONLY for the apex rung S4.
// ---------------------------------------------------------------------------

#[test]
fn permits_autonomous_true_only_for_s4_via_store() {
    assert!(!SkillStore::permits_autonomous(SkillTrustTier::S1));
    assert!(!SkillStore::permits_autonomous(SkillTrustTier::S2));
    assert!(!SkillStore::permits_autonomous(SkillTrustTier::S3));
    assert!(SkillStore::permits_autonomous(SkillTrustTier::S4));
}

#[test]
fn permits_autonomous_true_only_for_s4_via_gate() {
    // The autonomy gate's barrier helper MUST mirror the store helper exactly
    // (it is intentionally store-free so the gate stays decoupled from the
    // SkillStore — F-6 AUTONOMY GATE decoupling rule).
    assert!(!skill_permits_autonomous(SkillTrustTier::S1));
    assert!(!skill_permits_autonomous(SkillTrustTier::S2));
    assert!(!skill_permits_autonomous(SkillTrustTier::S3));
    assert!(skill_permits_autonomous(SkillTrustTier::S4));
}

#[test]
fn store_and_gate_permits_autonomous_agree_on_every_rung() {
    for tier in [
        SkillTrustTier::S1,
        SkillTrustTier::S2,
        SkillTrustTier::S3,
        SkillTrustTier::S4,
    ] {
        assert_eq!(
            SkillStore::permits_autonomous(tier),
            skill_permits_autonomous(tier),
            "store and gate disagree on autonomy for {tier:?}"
        );
    }
}

// ---------------------------------------------------------------------------
// 2. Wire-vocabulary disjointness: s1..s4 vs the documented memory T0..T3.
// ---------------------------------------------------------------------------

#[test]
fn skill_wire_labels_are_lowercase_s_prefixed() {
    for (tier, expect) in [
        (SkillTrustTier::S1, "s1"),
        (SkillTrustTier::S2, "s2"),
        (SkillTrustTier::S3, "s3"),
        (SkillTrustTier::S4, "s4"),
    ] {
        assert_eq!(tier.as_str(), expect);
        assert!(
            tier.as_str().starts_with('s'),
            "skill tier label {} must be lowercase-s prefixed",
            tier.as_str()
        );
    }
}

#[test]
fn skill_vocabulary_disjoint_from_memory_vocabulary() {
    // The memory ladder's wire vocabulary is the documented uppercase set
    // {T0, T1, T2, T3} (see crates/memory/src/trust.rs TrustTier::as_str). We
    // assert disjointness WITHOUT importing elai-memory (not a contracts dep):
    // every skill label is lowercase-s prefixed and never equals a T-prefixed
    // memory tag. This is the runtime witness that the two ladders cannot
    // collide on the SSB wire; the compile_fail doc-test in skill_trust.rs is
    // the type-level witness that there is no From/Into bridge.
    let memory_tags = ["T0", "T1", "T2", "T3"];
    for tier in [
        SkillTrustTier::S1,
        SkillTrustTier::S2,
        SkillTrustTier::S3,
        SkillTrustTier::S4,
    ] {
        let label = tier.as_str();
        assert!(
            !memory_tags.contains(&label),
            "skill tier label {label} collides with a memory tier tag"
        );
        assert!(
            !label.starts_with('T'),
            "skill tier label {label} must not use the memory ladder's T prefix"
        );
    }
}

#[test]
fn skill_tier_parse_round_trips_and_rejects_memory_tags() {
    for tier in [
        SkillTrustTier::S1,
        SkillTrustTier::S2,
        SkillTrustTier::S3,
        SkillTrustTier::S4,
    ] {
        let parsed = SkillTrustTier::parse(tier.as_str()).expect("round trip");
        assert_eq!(parsed, tier);
    }
    // A memory-ladder tag must NOT parse as a skill tier (vocab is disjoint).
    for memory_tag in ["T0", "T1", "T2", "T3"] {
        assert!(
            SkillTrustTier::parse(memory_tag).is_err(),
            "memory tag {memory_tag} must not parse as a skill tier"
        );
    }
}

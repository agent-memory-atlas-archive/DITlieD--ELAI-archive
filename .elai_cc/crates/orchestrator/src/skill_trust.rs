//! F-6 / Slice A1 — learned-skill trust ladder + skill store.
//!
//! SYNOPSIS
//! --------
//! [`SkillTrustTier`] is a FOUR-rung trust ladder (S1..S4) for *learned
//! skills* — distilled, reusable execution recipes the runtime acquires
//! after a critic-gated success (C6b, a later slice). A skill is born at
//! S1 and climbs the ladder as it accrues invocations with a low failure
//! rate and sufficient age; the apex rung **S4 requires an explicit human
//! grant** (`promote(.., human_grant=true)`). Only an S4 skill
//! [`SkillStore::permits_autonomous`] dispatch — sub-S4 skills route to a
//! human via the autonomy barrier in `crate::autonomous_gate`.
//!
//! [`SkillStore`] is the in-memory authority (a [`BTreeMap`] keyed by skill
//! id, borrowing the [`crate::compiled_pipeline::CompiledPipelineStore`]
//! shape — `HashMap` is banned per B-15). [`SkillStore::demote_if_stale`]
//! (C6c maintenance) decays a skill one rung per scheduler tick once its
//! [`SkillStore::staleness_score`] crosses [`SKILL_STALE_THRESHOLD`], with
//! S1 as the floor.
//!
//! P0 TYPE BARRIER (namespace isolation)
//! -------------------------------------
//! [`SkillTrustTier`] is a NEW LOCAL enum. It is intentionally **not**
//! bridged to [`elai_memory::TrustTier`] (the C-48 SafeHarbor T0..T3 memory
//! ladder) nor to the reflection ladder
//! ([`crate::planning_decomposition::reflection_buffer::ReflectionTrustTier`]).
//! Three trust ladders stay namespace-isolated: there is **no `From`/`Into`
//! conversion in either direction**. The doc-test below proves the bridge
//! does not exist (it MUST fail to compile):
//!
//! ```compile_fail
//! use elai_orchestrator::skill_trust::SkillTrustTier;
//! // There is intentionally NO From/Into between the skill ladder (S1..S4)
//! // and the memory ladder (T0..T3). This line MUST NOT compile.
//! let _: elai_memory::TrustTier = SkillTrustTier::S4.into();
//! ```
//!
//! GROUNDING
//! ---------
//! Mechanism shapes (a tiered acquisition ladder + a staleness-driven
//! decay) are grounded in the SHARD line of work and the Voyager
//! critic-loop, per `workflow/reasoning/F-6-decisions.md`. The threshold
//! CONSTANTS below ([`TIER_MIN_INVOCATIONS`], [`TIER_MAX_FAILURE_RATE`],
//! [`TIER_MIN_AGE_SECS`], [`SKILL_EXPECTED_LIFETIME_SECS`],
//! [`SKILL_STALE_THRESHOLD`]) are ELAI's own design values — they are NOT
//! taken from any external benchmark and carry no external target number.
//!
//! NO STUBS. No `.unwrap()`/`.expect()` outside `#[cfg(test)]`. blake3 only
//! (no hashing needed here, but the ban stands).

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use thiserror::Error;

// ---------------------------------------------------------------------------
// Constants — ELAI-own design values, no magic numbers, NOT externally sourced.
// ---------------------------------------------------------------------------

/// F-6 — minimum cumulative successful invocations a skill must accrue
/// before [`SkillStore::promote`] will advance it one rung. ELAI-own value.
pub const TIER_MIN_INVOCATIONS: u32 = 3;

/// F-6 — maximum tolerated failure rate (failures / max(invocations, 1)) for
/// a promotion to be granted. At or above this ratio the skill is too flaky
/// to climb. ELAI-own value (0.20 = 20%).
pub const TIER_MAX_FAILURE_RATE: f64 = 0.20;

/// F-6 — minimum wall-clock age (seconds since `acquired_at`) before a skill
/// may climb a rung. Prevents a burst of same-tick successes from vaulting a
/// brand-new skill to the top. ELAI-own value (1 hour).
pub const TIER_MIN_AGE_SECS: u64 = 3_600;

/// F-6 — denominator for [`SkillStore::staleness_score`]: the wall-clock span
/// (seconds) over which an unverified skill is considered to have fully aged
/// out (score reaches 1.0). ELAI-own value (14 days).
pub const SKILL_EXPECTED_LIFETIME_SECS: f64 = 1_209_600.0;

/// F-6 — staleness-score threshold at or above which
/// [`SkillStore::demote_if_stale`] knocks a skill down one rung on a
/// scheduler tick. ELAI-own value: 1.0 = one full expected-lifetime span
/// elapsed since last verification.
pub const SKILL_STALE_THRESHOLD: f64 = 1.0;

// ---------------------------------------------------------------------------
// SkillTrustTier — the F-6 trust ladder. LOCAL enum, NO bridge to other ladders.
// ---------------------------------------------------------------------------

/// F-6 trust rung for a learned skill. `S1` (lowest, freshly acquired) ..
/// `S4` (apex, human-granted, autonomy-permitted).
///
/// LOCAL to `elai-orchestrator`. NOT convertible to/from
/// [`elai_memory::TrustTier`] (T0..T3) or
/// [`crate::planning_decomposition::reflection_buffer::ReflectionTrustTier`]
/// — the three ladders are namespace-isolated by design (see module
/// docstring + the `compile_fail` doc-test). `ReflectionTrustTier` is the
/// local-enum precedent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SkillTrustTier {
    /// Freshly acquired. Never autonomy-permitted. Floor of the ladder.
    S1,
    /// Proven once: cleared the invocation/failure/age gate a single rung up.
    S2,
    /// Well-exercised: cleared the gate twice. Still requires a human for S4.
    S3,
    /// Apex. Reached only via an explicit human grant. The ONLY rung that
    /// [`SkillStore::permits_autonomous`] dispatch.
    S4,
}

impl SkillTrustTier {
    /// Snake-case wire label (used by the SSB `tier` / `prior_tier` fields
    /// and the CLI/dashboard surfaces).
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::S1 => "s1",
            Self::S2 => "s2",
            Self::S3 => "s3",
            Self::S4 => "s4",
        }
    }

    /// Parse a snake-case wire label back into a tier. Returns
    /// [`SkillStoreError::UnknownTier`] for any unrecognized label.
    pub fn parse(s: &str) -> Result<Self, SkillStoreError> {
        match s {
            "s1" => Ok(Self::S1),
            "s2" => Ok(Self::S2),
            "s3" => Ok(Self::S3),
            "s4" => Ok(Self::S4),
            other => Err(SkillStoreError::UnknownTier(other.to_owned())),
        }
    }

    /// The next rung up, or `None` at the apex (`S4`).
    #[must_use]
    fn next_up(self) -> Option<Self> {
        match self {
            Self::S1 => Some(Self::S2),
            Self::S2 => Some(Self::S3),
            Self::S3 => Some(Self::S4),
            Self::S4 => None,
        }
    }

    /// The next rung down, or `None` at the floor (`S1`).
    #[must_use]
    fn next_down(self) -> Option<Self> {
        match self {
            Self::S1 => None,
            Self::S2 => Some(Self::S1),
            Self::S3 => Some(Self::S2),
            Self::S4 => Some(Self::S3),
        }
    }
}

// ---------------------------------------------------------------------------
// LearnedSkill — one acquired skill record.
// ---------------------------------------------------------------------------

/// F-6 — one learned skill in the [`SkillStore`].
///
/// `invocations` / `failures` drive the promotion gate; `acquired_at` gates
/// minimum age; `last_verified` drives the staleness score. All timestamps
/// are wall-clock UNIX seconds supplied by the caller (the store is clock-free
/// so it stays deterministic + testable).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LearnedSkill {
    /// Stable skill identifier (caller-supplied; e.g. a concept-class slug).
    pub id: String,
    /// Human-readable description (also the retrieval-embedding source in C6b).
    pub description: String,
    /// Current trust rung.
    pub tier: SkillTrustTier,
    /// Cumulative successful invocations observed for this skill.
    pub invocations: u32,
    /// Cumulative failed invocations observed for this skill.
    pub failures: u32,
    /// Wall-clock UNIX seconds when the skill was first acquired.
    pub acquired_at: u64,
    /// Wall-clock UNIX seconds of the most recent verification (acquisition,
    /// promotion, or a fresh critic-gated success). Drives staleness.
    pub last_verified: u64,
}

impl LearnedSkill {
    /// Failure rate = failures / max(invocations + failures, 1). Saturates at
    /// 0.0 for a never-run skill (no failures observed).
    #[must_use]
    pub fn failure_rate(&self) -> f64 {
        let total = self.invocations.saturating_add(self.failures);
        if total == 0 {
            return 0.0;
        }
        f64::from(self.failures) / f64::from(total)
    }
}

// ---------------------------------------------------------------------------
// SkillStore — in-memory authority. BTreeMap (HashMap banned, B-15).
// ---------------------------------------------------------------------------

/// F-6 — in-memory learned-skill store. Keyed by skill id. Borrows the
/// [`crate::compiled_pipeline::CompiledPipelineStore`] shape (a
/// [`BTreeMap`] + clock-free methods that take `now` explicitly).
#[derive(Debug, Default)]
pub struct SkillStore {
    skills: BTreeMap<String, LearnedSkill>,
}

impl SkillStore {
    /// Construct an empty store.
    #[must_use]
    pub fn new() -> Self {
        Self {
            skills: BTreeMap::new(),
        }
    }

    /// Read-only accessor (tests / `elai skills-mgmt library list`).
    #[must_use]
    pub fn skills(&self) -> &BTreeMap<String, LearnedSkill> {
        &self.skills
    }

    /// Insert (or replace) a fully-specified [`LearnedSkill`] record, keyed on
    /// its `id`. Unlike [`Self::acquire`] (which always seeds a fresh S1 floor
    /// record), `upsert` stores the record verbatim — used by the C6c
    /// scheduler-tick tests and the `elai skills-mgmt library` CLI surface to
    /// seed a known tier/`last_verified` for an observable promote/demote
    /// sweep. Returns the stored record's tier.
    pub fn upsert(&mut self, skill: LearnedSkill) -> SkillTrustTier {
        let tier = skill.tier;
        self.skills.insert(skill.id.clone(), skill);
        tier
    }

    /// Number of skills currently tracked.
    #[must_use]
    pub fn len(&self) -> usize {
        self.skills.len()
    }

    /// Whether the store holds no skills.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.skills.is_empty()
    }

    /// Fetch a skill by id.
    #[must_use]
    pub fn get(&self, id: &str) -> Option<&LearnedSkill> {
        self.skills.get(id)
    }

    /// Insert (or replace) a freshly acquired skill at the floor rung `S1`.
    /// `now` is the acquisition timestamp (also seeds `last_verified`).
    /// Returns the inserted record's tier for the caller's SSB beacon.
    pub fn acquire(&mut self, id: String, description: String, now: u64) -> SkillTrustTier {
        let skill = LearnedSkill {
            id: id.clone(),
            description,
            tier: SkillTrustTier::S1,
            invocations: 0,
            failures: 0,
            acquired_at: now,
            last_verified: now,
        };
        let tier = skill.tier;
        self.skills.insert(id, skill);
        tier
    }

    /// True only for the apex rung `S4`. Sub-S4 skills are NOT cleared for
    /// autonomous dispatch — the autonomy barrier routes them to a human.
    #[must_use]
    pub fn permits_autonomous(tier: SkillTrustTier) -> bool {
        tier == SkillTrustTier::S4
    }

    /// Promote a skill one rung if it clears the gate.
    ///
    /// Gate (all must hold):
    ///   * invocations >= [`TIER_MIN_INVOCATIONS`]
    ///   * failure_rate < [`TIER_MAX_FAILURE_RATE`]
    ///   * age (`now - acquired_at`) >= [`TIER_MIN_AGE_SECS`]
    ///   * for the apex jump `S3 -> S4`, `human_grant` MUST be true.
    ///
    /// On success the skill's `tier` advances one rung, `last_verified` is
    /// refreshed to `now`, and the NEW tier is returned. Already at `S4`
    /// returns [`SkillStoreError::AtApex`]. A failed gate returns the
    /// specific [`SkillStoreError`] reason so the caller can surface it.
    pub fn promote(
        &mut self,
        id: &str,
        now: u64,
        human_grant: bool,
    ) -> Result<SkillTrustTier, SkillStoreError> {
        let skill = self
            .skills
            .get_mut(id)
            .ok_or_else(|| SkillStoreError::NotFound(id.to_owned()))?;

        let next = skill
            .tier
            .next_up()
            .ok_or(SkillStoreError::AtApex)?;

        if skill.invocations < TIER_MIN_INVOCATIONS {
            return Err(SkillStoreError::InsufficientInvocations {
                got: skill.invocations,
                min: TIER_MIN_INVOCATIONS,
            });
        }

        let rate = skill.failure_rate();
        if rate >= TIER_MAX_FAILURE_RATE {
            return Err(SkillStoreError::FailureRateTooHigh {
                got: rate,
                max: TIER_MAX_FAILURE_RATE,
            });
        }

        let age = now.saturating_sub(skill.acquired_at);
        if age < TIER_MIN_AGE_SECS {
            return Err(SkillStoreError::TooYoung {
                got: age,
                min: TIER_MIN_AGE_SECS,
            });
        }

        if next == SkillTrustTier::S4 && !human_grant {
            return Err(SkillStoreError::HumanGrantRequired);
        }

        skill.tier = next;
        skill.last_verified = now;
        Ok(next)
    }

    /// Staleness score = `(now - last_verified) / SKILL_EXPECTED_LIFETIME_SECS`.
    /// 0.0 = just verified; 1.0 = one full expected-lifetime span elapsed.
    /// Clamped at 0.0 if `last_verified` is in the (caller-clock) future.
    #[must_use]
    pub fn staleness_score(skill: &LearnedSkill, now: u64) -> f64 {
        let elapsed = now.saturating_sub(skill.last_verified);
        elapsed as f64 / SKILL_EXPECTED_LIFETIME_SECS
    }

    /// One staleness sweep (C6c maintenance, called on a scheduler tick).
    /// For every skill whose [`Self::staleness_score`] is at or above
    /// [`SKILL_STALE_THRESHOLD`] AND that is above the `S1` floor, knock the
    /// skill down exactly ONE rung and refresh `last_verified` to `now` (so a
    /// single tick demotes at most one rung per skill). Returns one
    /// [`DemotedSkill`] record per affected skill (empty when nothing aged
    /// out), for the caller's `SkillLibraryEvent(DemotedStale)` beacons.
    pub fn demote_if_stale(&mut self, now: u64) -> Vec<DemotedSkill> {
        let mut demoted = Vec::new();
        for skill in self.skills.values_mut() {
            let score = Self::staleness_score(skill, now);
            if score < SKILL_STALE_THRESHOLD {
                continue;
            }
            let Some(lower) = skill.tier.next_down() else {
                // Already at the S1 floor — nothing to demote.
                continue;
            };
            let prior = skill.tier;
            skill.tier = lower;
            skill.last_verified = now;
            demoted.push(DemotedSkill {
                id: skill.id.clone(),
                prior_tier: prior,
                new_tier: lower,
                staleness_score: score,
            });
        }
        demoted
    }
}

/// F-6 — record of one staleness demotion, returned by
/// [`SkillStore::demote_if_stale`] so the caller can emit a
/// `SkillLibraryEvent(DemotedStale)` SSB beacon per affected skill.
#[derive(Debug, Clone, PartialEq)]
pub struct DemotedSkill {
    /// The demoted skill's id.
    pub id: String,
    /// Tier before the demotion.
    pub prior_tier: SkillTrustTier,
    /// Tier after the demotion (exactly one rung down).
    pub new_tier: SkillTrustTier,
    /// The staleness score that triggered the demotion.
    pub staleness_score: f64,
}

/// F-6 — error type for [`SkillStore`] operations. Mirrors the
/// `thiserror` style of [`crate::compiled_pipeline::CompiledPipelineStoreError`].
#[derive(Debug, Error, PartialEq)]
pub enum SkillStoreError {
    /// No skill with the given id is tracked.
    #[error("learned skill not found: {0}")]
    NotFound(String),
    /// Cannot promote past the apex rung `S4`.
    #[error("learned skill already at apex tier S4")]
    AtApex,
    /// Not enough successful invocations to climb a rung.
    #[error("insufficient invocations for promotion: got {got}, need {min}")]
    InsufficientInvocations {
        /// Observed invocation count.
        got: u32,
        /// Required minimum.
        min: u32,
    },
    /// Failure rate at or above the promotion ceiling.
    #[error("failure rate too high for promotion: got {got:.3}, max {max:.3}")]
    FailureRateTooHigh {
        /// Observed failure rate.
        got: f64,
        /// Maximum tolerated rate.
        max: f64,
    },
    /// Skill is younger than the minimum promotion age.
    #[error("learned skill too young for promotion: age {got}s, need {min}s")]
    TooYoung {
        /// Observed age in seconds.
        got: u64,
        /// Required minimum age in seconds.
        min: u64,
    },
    /// The apex jump `S3 -> S4` requires an explicit human grant.
    #[error("apex tier S4 requires an explicit human grant")]
    HumanGrantRequired,
    /// Unrecognized snake-case tier label in [`SkillTrustTier::parse`].
    #[error("unknown skill trust tier: {0}")]
    UnknownTier(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seed(store: &mut SkillStore, id: &str, now: u64) {
        store.acquire(id.to_owned(), format!("desc for {id}"), now);
    }

    #[test]
    fn acquire_starts_at_s1_floor() {
        let mut store = SkillStore::new();
        let tier = store.acquire("k".into(), "d".into(), 1_000);
        assert_eq!(tier, SkillTrustTier::S1);
        let s = store.get("k").expect("present");
        assert_eq!(s.tier, SkillTrustTier::S1);
        assert_eq!(s.invocations, 0);
        assert_eq!(s.last_verified, 1_000);
    }

    #[test]
    fn permits_autonomous_only_s4() {
        assert!(!SkillStore::permits_autonomous(SkillTrustTier::S1));
        assert!(!SkillStore::permits_autonomous(SkillTrustTier::S2));
        assert!(!SkillStore::permits_autonomous(SkillTrustTier::S3));
        assert!(SkillStore::permits_autonomous(SkillTrustTier::S4));
    }

    #[test]
    fn promote_climbs_one_rung_when_gate_clears() {
        let mut store = SkillStore::new();
        seed(&mut store, "k", 0);
        {
            let s = store.skills.get_mut("k").expect("present");
            s.invocations = TIER_MIN_INVOCATIONS;
            s.failures = 0;
        }
        let now = TIER_MIN_AGE_SECS + 1;
        let new = store.promote("k", now, false).expect("promote ok");
        assert_eq!(new, SkillTrustTier::S2);
        assert_eq!(store.get("k").expect("present").last_verified, now);
    }

    #[test]
    fn promote_rejects_insufficient_invocations() {
        let mut store = SkillStore::new();
        seed(&mut store, "k", 0);
        let err = store
            .promote("k", TIER_MIN_AGE_SECS + 1, false)
            .expect_err("must reject");
        assert!(matches!(
            err,
            SkillStoreError::InsufficientInvocations { .. }
        ));
    }

    #[test]
    fn promote_rejects_high_failure_rate() {
        let mut store = SkillStore::new();
        seed(&mut store, "k", 0);
        {
            let s = store.skills.get_mut("k").expect("present");
            s.invocations = 5;
            s.failures = 5; // 50% > 20% ceiling
        }
        let err = store
            .promote("k", TIER_MIN_AGE_SECS + 1, false)
            .expect_err("must reject");
        assert!(matches!(err, SkillStoreError::FailureRateTooHigh { .. }));
    }

    #[test]
    fn promote_rejects_too_young() {
        let mut store = SkillStore::new();
        seed(&mut store, "k", 0);
        {
            let s = store.skills.get_mut("k").expect("present");
            s.invocations = TIER_MIN_INVOCATIONS;
        }
        let err = store.promote("k", 10, false).expect_err("must reject");
        assert!(matches!(err, SkillStoreError::TooYoung { .. }));
    }

    #[test]
    fn promote_to_s4_requires_human_grant() {
        let mut store = SkillStore::new();
        seed(&mut store, "k", 0);
        // Walk it up to S3 with grants disabled (S1->S2->S3 do not need one).
        {
            let s = store.skills.get_mut("k").expect("present");
            s.invocations = TIER_MIN_INVOCATIONS;
        }
        let now = TIER_MIN_AGE_SECS + 1;
        assert_eq!(store.promote("k", now, false).expect("s2"), SkillTrustTier::S2);
        assert_eq!(store.promote("k", now, false).expect("s3"), SkillTrustTier::S3);
        // S3 -> S4 without a grant must be refused.
        let err = store.promote("k", now, false).expect_err("must require grant");
        assert_eq!(err, SkillStoreError::HumanGrantRequired);
        // With the grant it reaches the apex.
        assert_eq!(store.promote("k", now, true).expect("s4"), SkillTrustTier::S4);
        // Apex cannot climb further.
        assert_eq!(store.promote("k", now, true).expect_err("apex"), SkillStoreError::AtApex);
    }

    #[test]
    fn staleness_score_scales_with_elapsed() {
        let s = LearnedSkill {
            id: "k".into(),
            description: "d".into(),
            tier: SkillTrustTier::S3,
            invocations: 0,
            failures: 0,
            acquired_at: 0,
            last_verified: 0,
        };
        // Exactly one expected-lifetime span -> score 1.0.
        let now = SKILL_EXPECTED_LIFETIME_SECS as u64;
        let score = SkillStore::staleness_score(&s, now);
        assert!((score - 1.0).abs() < 1e-9, "score={score}");
        // Just verified -> 0.0.
        assert_eq!(SkillStore::staleness_score(&s, 0), 0.0);
    }

    #[test]
    fn demote_if_stale_drops_one_rung_with_s1_floor() {
        let mut store = SkillStore::new();
        store.acquire("hi".into(), "d".into(), 0);
        store.acquire("floor".into(), "d".into(), 0);
        {
            let s = store.skills.get_mut("hi").expect("present");
            s.tier = SkillTrustTier::S3;
            s.last_verified = 0;
        }
        {
            let s = store.skills.get_mut("floor").expect("present");
            s.tier = SkillTrustTier::S1; // already at floor
            s.last_verified = 0;
        }
        let now = SKILL_EXPECTED_LIFETIME_SECS as u64 + 1;
        let demoted = store.demote_if_stale(now);
        // Only the S3 skill demotes (one rung -> S2). The S1 floor skill is
        // stale but cannot drop further, so it is NOT in the demoted list.
        assert_eq!(demoted.len(), 1);
        assert_eq!(demoted[0].id, "hi");
        assert_eq!(demoted[0].prior_tier, SkillTrustTier::S3);
        assert_eq!(demoted[0].new_tier, SkillTrustTier::S2);
        assert_eq!(store.get("hi").expect("present").tier, SkillTrustTier::S2);
        assert_eq!(store.get("floor").expect("present").tier, SkillTrustTier::S1);
        // last_verified refreshed so the next tick does not double-demote.
        assert_eq!(store.get("hi").expect("present").last_verified, now);
    }

    #[test]
    fn demote_if_stale_skips_fresh_skills() {
        let mut store = SkillStore::new();
        store.acquire("fresh".into(), "d".into(), 0);
        {
            let s = store.skills.get_mut("fresh").expect("present");
            s.tier = SkillTrustTier::S4;
            s.last_verified = 100;
        }
        // now == last_verified -> score 0 -> no demotion.
        let demoted = store.demote_if_stale(100);
        assert!(demoted.is_empty());
        assert_eq!(store.get("fresh").expect("present").tier, SkillTrustTier::S4);
    }

    #[test]
    fn tier_wire_labels_round_trip() {
        for t in [
            SkillTrustTier::S1,
            SkillTrustTier::S2,
            SkillTrustTier::S3,
            SkillTrustTier::S4,
        ] {
            let parsed = SkillTrustTier::parse(t.as_str()).expect("round trip");
            assert_eq!(parsed, t);
        }
        assert!(SkillTrustTier::parse("s5").is_err());
    }
}

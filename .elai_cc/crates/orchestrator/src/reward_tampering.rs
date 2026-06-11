//! H-28 (SCHEMA_VERSION 236): reward-tampering gate — the self-evaluator-capture
//! defense (Everitt current-RF, arXiv:1908.04734).
//!
//! A self-MODIFYING optimizer that can read or mutate its own evaluator/threshold/reward
//! function will route itself to PASS: the DGM self-improvement loop EMPIRICALLY did this
//! (node 114 stripped its own hallucination markers to achieve a perfect score —
//! `workflow/papers/dgm-interface.md:49`). Everitt et al. name two reward-tampering
//! subtypes (arXiv:1908.04734): RF TAMPERING (the agent edits the implemented reward
//! function itself, Section 3) and RF-INPUT TAMPERING (the agent leaves the reward fn
//! intact but games the inputs it reads, Section 4). The fix is the current-RF principle:
//! the reward function is PRIVATE and uninfluenceable, and the agent optimizes the CURRENT
//! frozen reward (Claim 3, Section 3.2).
//!
//! ELAI's shipped blake3 tripwire ([`crate::ssb::SsbActionType::ArchiveVerifyResult`])
//! catches the FIRST subtype — an eval-harness EDIT changes the harness hash. H-28 makes
//! that the load-bearing leg AND adds a flag for the SECOND subtype — a score jump with
//! the hash intact.
//!
//! # The composite gate ([`check_reward_tampering`]) — TWO SOUND Tier-1 legs
//!   * LEG 3a (deterministic HARD-BLOCK, sound): the OBSERVED frozen-evaluator blake3 hash
//!     != the recorded baseline hash => the agent edited / monkeypatched the evaluator =>
//!     [`RewardTamperingKind::EvaluatorModified`] Block. This REUSES the shipped
//!     ArchiveVerifyResult blake3-mismatch semantics. It is also the FAIL-CLOSED arm: a
//!     missing / empty / un-readable baseline hash maps here (cannot prove no-tampering =>
//!     Block, R1/R2 — "not found means it never ran and produced it").
//!   * LEG 3b (FLAG-to-Layer-3, sound-as-a-flag): the eval hash is INTACT but the claimed
//!     reward jumped beyond a LOCKED plausibility band vs the prior reward =>
//!     [`RewardTamperingKind::ImplausibleRewardDelta`] FLAG. This routes to a human
//!     (Layer-3), NOT a hard auto-block, because a legitimate large improvement is possible
//!     (the band too-tight blocks a real gain; too-loose misses gaming — fail-to-human, not
//!     fail-to-silent-reject).
//!
//! # TD-3 ordering (load-bearing, structural)
//! Leg 3a is checked FIRST and SHORT-CIRCUITS (early return) before 3b can be reached. A
//! deterministic hash mismatch is dispositive: a probabilistic 3b green can NEVER override
//! a 3a Block. The precedence is encoded as an early return on hash mismatch, NOT a flag
//! the optimizer could flip.
//!
//! # Tier-1 = always-compiled, pure Rust, no seed (the wire-dark trap avoided)
//! ELAI's live darwinian admit path (`darwin_admit_loop.rs:298` two_gate_promote) is
//! feature-gated behind `cfg(feature = "darwinian-evolve")` + env
//! `ELAI_DARWINIAN_LOOP_ENABLE`. A production-default `cargo run` binary does NOT compile
//! that admit. Wiring the reward-tampering check ONLY to the gated admit would ship it
//! WIRE-DARK in the default build (the exact failure wiring-enforcement exists to catch).
//! So Tier-1 is the always-compiled `elai reward-check` CLI over a planted fixture triple
//! ([`crate::reward_tampering`] is NOT under any feature). The live darwin hook is
//! Tier-2/deferred behind the feature. See `workflow/reasoning/H-28-decisions.md`.
//!
//! # Who-freezes-the-freezer (R8, the irreducible top)
//! The frozen baseline hash is authored OUT-OF-BAND — the agent cannot freeze its own
//! reward (R8). The frozen reward fn (`promotion_gate.rs`) and this gate
//! (`reward_tampering.rs`) go in the gate-manifest LOCKED set, re-locked by a human via
//! `gate_freeze_check.py --lock` OUT-OF-BAND (the executing run cannot re-lock its own
//! gate). The human is the root of trust; this residue is documented, not enforced inside
//! the gate. See `workflow/reasoning/H-28-decisions.md`.
//!
//! # Fail-closed (R1/R2)
//!   * a missing / empty / un-readable baseline hash => `EvaluatorModified` Block (a dead
//!     evidence path screams — cannot prove no-tampering => fail closed).
//!   * a hash MISMATCH (leg 3a) => `EvaluatorModified` Block, returns immediately.
//!   * a non-finite reward / band => `EvaluatorModified` Block (un-checkable reward =>
//!     fail closed).
//!
//! # No .unwrap()/.expect() in non-test code (CLAUDE.md HARD RULE). blake3 only.

use std::path::Path;
use std::sync::Arc;

use thiserror::Error;
use tracing::{debug, warn};

use crate::ssb::{RewardTamperingKind, RewardTamperingPayload, Ssb, SsbPayload};

// ---------------------------------------------------------------------------
// Constants (CLAUDE.md no-magic-numbers)
// ---------------------------------------------------------------------------

/// The LOCKED maximum plausible single-step reward delta (the leg-3b threshold). The
/// reward / fitness lives in `[0, 1]` (`MutantCandidate::derived_alpha`,
/// `darwin_admit_loop.rs`), so a single-step jump of more than this fraction of the full
/// range, with the eval hash intact, is implausible and is FLAGGED to human Layer-3. Set
/// CONSERVATIVELY (a large value): a too-tight band blocks a legitimate gain, a too-loose
/// band misses gaming — and 3b is a FLAG (fail-to-human), not a hard auto-block, so the
/// conservative default biases toward not crying wolf while still catching a self-mod that
/// claims it doubled its own fitness in one step with the evaluator untouched.
pub const DEFAULT_PLAUSIBLE_REWARD_BAND: f64 = 0.50;

/// Agent id stamped on the emitted [`SsbPayload::RewardTampering`] row.
const REWARD_TAMPERING_AGENT_ID: &str = "reward-tampering";

/// Superstep stamped on the emitted row (single-shot gate; not part of a graph superstep
/// loop).
const REWARD_TAMPERING_SUPERSTEP: i64 = 0;

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// Errors from the reward-tampering gate. An SSB write failure for the gate's own row is
/// the only propagated error; every TAMPERING signal (including the fail-closed
/// un-checkable cases) is an `EvaluatorModified` BLOCK VERDICT, not an error — the gate
/// always emits its row.
#[derive(Debug, Error)]
pub enum RewardTamperingError {
    /// SSB write failure for the gate's own row.
    #[error("SSB error: {0}")]
    Ssb(#[from] crate::ssb::SsbError),
}

// ---------------------------------------------------------------------------
// Outcome
// ---------------------------------------------------------------------------

/// The reward-tampering outcome. `kind == NoTampering` is the only PASS.
#[derive(Debug, Clone, PartialEq)]
pub struct RewardTamperingOutcome {
    /// The verdict class (one PASS arm + two non-PASS arms).
    pub kind: RewardTamperingKind,
    /// The OBSERVED frozen-evaluator blake3 hash at decision time (empty when the baseline
    /// was un-readable). BLAKE3 ONLY.
    pub harness_blake3: String,
    /// The incumbent / parent reward (the pre-self-mod fitness).
    pub prior_reward: f64,
    /// The claimed post-self-mod reward.
    pub new_reward: f64,
    /// `new_reward - prior_reward`.
    pub reward_delta: f64,
    /// The LOCKED plausibility band used for the leg-3b check.
    pub plausible_band: f64,
}

impl RewardTamperingOutcome {
    /// `true` iff the gate ALLOWS promotion (the evaluator hash matched the baseline AND
    /// the reward delta was within the locked band). Any non-PASS arm => `false`.
    ///
    /// NOTE: `ImplausibleRewardDelta` is NOT a pass — the gate does not auto-promote on a
    /// score jump; it routes to human Layer-3. `is_pass() == false` for that arm, but the
    /// caller distinguishes "hard block" (`EvaluatorModified`) from "flag to human"
    /// (`ImplausibleRewardDelta`) on `kind`.
    pub fn is_pass(&self) -> bool {
        self.kind == RewardTamperingKind::NoTampering
    }

    /// `true` iff this verdict is a HARD BLOCK (leg 3a / fail-closed). Distinct from the
    /// `ImplausibleRewardDelta` FLAG (routed to a human). Used by the CLI / admit driver to
    /// decide a non-zero exit vs a human-review route.
    pub fn is_hard_block(&self) -> bool {
        self.kind == RewardTamperingKind::EvaluatorModified
    }
}

// ---------------------------------------------------------------------------
// Pure core (no IO) — the deterministic comparison (GD-2 recomputable)
// ---------------------------------------------------------------------------

/// Pure, deterministic reward-tampering classification (no IO / clock / RNG). The
/// load-bearing logic, factored out of [`check_reward_tampering`] so the contract test can
/// drive it without opening an SSB (mirrors `validate_gates` vs `validate_gates_with` in
/// `gate_self_validation.rs`). Recomputable under `ELAI_SEED` (GD-2): same inputs => same
/// verdict.
///
/// # Arguments
/// * `expected_harness_blake3` — the FROZEN baseline blake3 hash of the evaluator (the
///   recorded, out-of-band reference). EMPTY string => the baseline is missing /
///   un-readable => fail-closed `EvaluatorModified` (R2).
/// * `observed_harness_blake3` — the RECOMPUTED blake3 hash of the evaluator at decision
///   time. EMPTY string => the observed harness could not be hashed => fail-closed
///   `EvaluatorModified`.
/// * `prior_reward`            — the incumbent / parent reward.
/// * `new_reward`              — the claimed post-self-mod reward.
/// * `band`                    — the LOCKED plausibility band (the 3b threshold).
///
/// # Legs (TD-3 ordering)
///   1. LEG 3a FIRST (dispositive): a missing baseline, a missing observed hash, a
///      mismatch, or a non-finite reward/band => `EvaluatorModified` (HARD-BLOCK / fail
///      closed). RETURNS before 3b.
///   2. LEG 3b (only when 3a passed): `new_reward - prior_reward > band` =>
///      `ImplausibleRewardDelta` (FLAG).
///   3. otherwise `NoTampering` (PASS).
pub fn classify_reward_tampering(
    expected_harness_blake3: &str,
    observed_harness_blake3: &str,
    prior_reward: f64,
    new_reward: f64,
    band: f64,
) -> RewardTamperingOutcome {
    let reward_delta = new_reward - prior_reward;

    // -- LEG 3a (deterministic HARD-BLOCK, dispositive, checked FIRST — TD-3). --------
    // Fail-closed: a missing/empty baseline OR a missing/empty observed hash means we
    // cannot PROVE the evaluator is untouched. "Not found = never ran = FAIL" (R2).
    if expected_harness_blake3.is_empty() || observed_harness_blake3.is_empty() {
        warn!(
            expected_empty = expected_harness_blake3.is_empty(),
            observed_empty = observed_harness_blake3.is_empty(),
            "reward_tampering: missing frozen-evaluator hash -> EvaluatorModified Block \
             (fail-closed R2: cannot prove no-tampering)"
        );
        return block(observed_harness_blake3, prior_reward, new_reward, reward_delta, band);
    }
    // Fail-closed: a non-finite reward or band is un-checkable (NaN comparisons silently
    // fail-open — `NaN > band` is always false), so treat it as tampering, not a pass.
    if !prior_reward.is_finite() || !new_reward.is_finite() || !band.is_finite() {
        warn!(
            prior_finite = prior_reward.is_finite(),
            new_finite = new_reward.is_finite(),
            band_finite = band.is_finite(),
            "reward_tampering: non-finite reward/band -> EvaluatorModified Block \
             (fail-closed: un-checkable reward)"
        );
        return block(observed_harness_blake3, prior_reward, new_reward, reward_delta, band);
    }
    // The load-bearing leg: the agent edited its own evaluator (hash mismatch) — a
    // deterministic, dispositive HARD-BLOCK. Reuses the ArchiveVerifyResult blake3-mismatch
    // semantics. Returns IMMEDIATELY so a 3b green can never override (TD-3).
    if expected_harness_blake3 != observed_harness_blake3 {
        warn!(
            "reward_tampering: frozen-evaluator blake3 MISMATCH (expected != observed) -> \
             EvaluatorModified HARD-BLOCK (leg 3a, the agent edited its own evaluator)"
        );
        return block(observed_harness_blake3, prior_reward, new_reward, reward_delta, band);
    }

    // -- LEG 3b (FLAG-to-Layer-3, only reached when the hash MATCHED). ----------------
    // The eval hash is intact but the reward jumped beyond the locked band => the agent
    // gamed the inputs the evaluator reads (RF-INPUT tampering, Everitt Section 4). FLAG,
    // not a hard block — a legitimate big gain is possible, so route to a human.
    if reward_delta > band {
        warn!(
            reward_delta,
            band,
            "reward_tampering: hash INTACT but reward delta beyond the locked band -> \
             ImplausibleRewardDelta FLAG (leg 3b, route to human Layer-3)"
        );
        return RewardTamperingOutcome {
            kind: RewardTamperingKind::ImplausibleRewardDelta,
            harness_blake3: observed_harness_blake3.to_string(),
            prior_reward,
            new_reward,
            reward_delta,
            plausible_band: band,
        };
    }

    debug!(
        reward_delta,
        band, "reward_tampering: hash matched and delta within band -> NoTampering (PASS)"
    );
    RewardTamperingOutcome {
        kind: RewardTamperingKind::NoTampering,
        harness_blake3: observed_harness_blake3.to_string(),
        prior_reward,
        new_reward,
        reward_delta,
        plausible_band: band,
    }
}

/// Build an `EvaluatorModified` HARD-BLOCK outcome (leg 3a / fail-closed). Centralised so
/// every fail-closed path returns the identical verdict shape.
fn block(
    observed_harness_blake3: &str,
    prior_reward: f64,
    new_reward: f64,
    reward_delta: f64,
    band: f64,
) -> RewardTamperingOutcome {
    RewardTamperingOutcome {
        kind: RewardTamperingKind::EvaluatorModified,
        harness_blake3: observed_harness_blake3.to_string(),
        prior_reward,
        new_reward,
        reward_delta,
        plausible_band: band,
    }
}

// ---------------------------------------------------------------------------
// Public driver (emits the off-agent SSB row)
// ---------------------------------------------------------------------------

/// Drive the reward-tampering gate over a fixture triple and emit ONE
/// [`SsbPayload::RewardTampering`] row (the off-agent done-gate artifact, R4).
///
/// This is the SEEDED + emitting boundary; the pure classification lives in
/// [`classify_reward_tampering`] (which the contract test drives without IO). The verdict
/// is recomputable from the journal given the same triple + seed (GD-2).
///
/// # Arguments
/// * `ssb`                     — the live SSB the gate emits its row into.
/// * `_db_path`                — the SSB db path (kept for signature symmetry with the
///   sibling gates that open a `TrialLedger`; this gate is pure and does not, so it is
///   unused — but the parameter keeps the call shape identical for the admit driver).
/// * `expected_harness_blake3` — the FROZEN baseline blake3 hash (empty => fail-closed).
/// * `observed_harness_blake3` — the recomputed blake3 hash at decision time.
/// * `prior_reward`            — the incumbent / parent reward.
/// * `new_reward`              — the claimed post-self-mod reward.
/// * `band`                    — the LOCKED plausibility band (use
///   [`DEFAULT_PLAUSIBLE_REWARD_BAND`] unless a domain override is locked).
/// * `run_id`                  — declared run id stamped on the row.
/// * `seed`                    — declared `ELAI_SEED` (GD-2); 0 when undeclared.
#[allow(clippy::too_many_arguments)]
pub async fn check_reward_tampering(
    ssb: &Arc<Ssb>,
    _db_path: &Path,
    expected_harness_blake3: &str,
    observed_harness_blake3: &str,
    prior_reward: f64,
    new_reward: f64,
    band: f64,
    run_id: &str,
    seed: u64,
) -> Result<RewardTamperingOutcome, RewardTamperingError> {
    let outcome = classify_reward_tampering(
        expected_harness_blake3,
        observed_harness_blake3,
        prior_reward,
        new_reward,
        band,
    );
    emit_outcome(ssb, &outcome, run_id, seed).await?;
    Ok(outcome)
}

/// Emit the single off-agent [`SsbPayload::RewardTampering`] row. The `corpus_blake3`
/// binds the verdict to the fixture triple so it is recomputable (GD-2). BLAKE3 ONLY.
async fn emit_outcome(
    ssb: &Arc<Ssb>,
    outcome: &RewardTamperingOutcome,
    run_id: &str,
    seed: u64,
) -> Result<(), RewardTamperingError> {
    let key = format!(
        "{}:{}:{}:{}:{}:{}",
        run_id,
        outcome.harness_blake3,
        outcome.prior_reward,
        outcome.new_reward,
        outcome.plausible_band,
        seed
    );
    let corpus_blake3 = blake3::hash(key.as_bytes()).to_hex().to_string();

    let payload = SsbPayload::RewardTampering(RewardTamperingPayload {
        kind: outcome.kind,
        harness_blake3: outcome.harness_blake3.clone(),
        prior_reward: outcome.prior_reward,
        new_reward: outcome.new_reward,
        reward_delta: outcome.reward_delta,
        plausible_band: outcome.plausible_band,
        run_id: run_id.to_string(),
        seed,
        corpus_blake3,
    });

    ssb.write_payload(
        REWARD_TAMPERING_AGENT_ID,
        REWARD_TAMPERING_SUPERSTEP,
        payload,
        vec!["reward_tampering.rs".to_string()],
    )
    .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ssb::{SsbActionType, SsbConfig};
    use crate::SsbReader;
    use tempfile::TempDir;

    const BASELINE_HASH: &str = "baseline0000000000000000000000000000000000000000000000000000000a";

    fn test_ssb(dir: &TempDir) -> (Arc<Ssb>, std::path::PathBuf) {
        let db_path = dir.path().join("ssb.db");
        let cfg = SsbConfig {
            db_path: db_path.clone(),
            pool_size: None,
            project_id: "h28-test".to_string(),
            session_id: "h28-test-session".to_string(),
        };
        let clock: Arc<dyn elai_core::clock::ClockProvider> =
            Arc::new(elai_core::clock::OsClock);
        let ssb = Arc::new(Ssb::new(cfg, clock).expect("ssb init"));
        (ssb, db_path)
    }

    #[test]
    fn matching_hash_in_band_is_no_tampering() {
        let out = classify_reward_tampering(
            BASELINE_HASH,
            BASELINE_HASH,
            0.50,
            0.55,
            DEFAULT_PLAUSIBLE_REWARD_BAND,
        );
        assert_eq!(out.kind, RewardTamperingKind::NoTampering);
        assert!(out.is_pass());
        assert!(!out.is_hard_block());
    }

    #[test]
    fn hash_mismatch_is_evaluator_modified_block() {
        let out = classify_reward_tampering(
            BASELINE_HASH,
            "observed_DIFFERENT_hash",
            0.50,
            0.55,
            DEFAULT_PLAUSIBLE_REWARD_BAND,
        );
        assert_eq!(out.kind, RewardTamperingKind::EvaluatorModified);
        assert!(!out.is_pass());
        assert!(out.is_hard_block());
    }

    #[test]
    fn implausible_delta_with_intact_hash_is_flag() {
        // Hash matches; reward jumps 0.10 -> 0.95 (delta 0.85 > 0.50 band).
        let out = classify_reward_tampering(
            BASELINE_HASH,
            BASELINE_HASH,
            0.10,
            0.95,
            DEFAULT_PLAUSIBLE_REWARD_BAND,
        );
        assert_eq!(out.kind, RewardTamperingKind::ImplausibleRewardDelta);
        assert!(!out.is_pass());
        // It is a FLAG, not a hard block.
        assert!(!out.is_hard_block());
    }

    #[test]
    fn td3_hash_mismatch_short_circuits_implausible_delta() {
        // BOTH a hash mismatch AND an implausible delta. The verdict must be the
        // hash-mismatch HARD-BLOCK (3a dispositive), never the 3b FLAG.
        let out = classify_reward_tampering(
            BASELINE_HASH,
            "observed_DIFFERENT_hash",
            0.10,
            0.99,
            DEFAULT_PLAUSIBLE_REWARD_BAND,
        );
        assert_eq!(
            out.kind,
            RewardTamperingKind::EvaluatorModified,
            "leg 3a (hash mismatch) must short-circuit leg 3b (implausible delta) — TD-3"
        );
        assert!(out.is_hard_block());
    }

    #[test]
    fn missing_baseline_is_fail_closed_block() {
        let out = classify_reward_tampering(
            "",
            BASELINE_HASH,
            0.50,
            0.55,
            DEFAULT_PLAUSIBLE_REWARD_BAND,
        );
        assert_eq!(out.kind, RewardTamperingKind::EvaluatorModified);
        assert!(out.is_hard_block());
    }

    #[test]
    fn missing_observed_hash_is_fail_closed_block() {
        let out = classify_reward_tampering(
            BASELINE_HASH,
            "",
            0.50,
            0.55,
            DEFAULT_PLAUSIBLE_REWARD_BAND,
        );
        assert_eq!(out.kind, RewardTamperingKind::EvaluatorModified);
        assert!(out.is_hard_block());
    }

    #[test]
    fn non_finite_reward_is_fail_closed_block() {
        let out = classify_reward_tampering(
            BASELINE_HASH,
            BASELINE_HASH,
            0.50,
            f64::NAN,
            DEFAULT_PLAUSIBLE_REWARD_BAND,
        );
        assert_eq!(
            out.kind,
            RewardTamperingKind::EvaluatorModified,
            "a NaN reward must fail closed (NaN > band is silently false — fail-open hazard)"
        );
    }

    #[test]
    fn delta_exactly_at_band_is_not_flagged() {
        // delta == band is within tolerance (only a STRICT exceedance flags).
        let out = classify_reward_tampering(
            BASELINE_HASH,
            BASELINE_HASH,
            0.10,
            0.60,
            DEFAULT_PLAUSIBLE_REWARD_BAND,
        );
        assert_eq!(out.kind, RewardTamperingKind::NoTampering);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn check_emits_a_row() {
        let dir = TempDir::new().expect("tempdir");
        let (ssb, db_path) = test_ssb(&dir);
        let out = check_reward_tampering(
            &ssb,
            &db_path,
            BASELINE_HASH,
            "observed_DIFFERENT_hash",
            0.50,
            0.55,
            DEFAULT_PLAUSIBLE_REWARD_BAND,
            "h28-unit",
            42,
        )
        .await
        .expect("check");
        assert_eq!(out.kind, RewardTamperingKind::EvaluatorModified);
        let rows = ssb
            .query_by_type(SsbActionType::RewardTampering)
            .expect("query");
        assert_eq!(rows.len(), 1, "exactly one RewardTampering row must land");
    }
}

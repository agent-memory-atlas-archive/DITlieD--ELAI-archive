//! H-28 contract tests — the current-RF reward-tampering gate (Everitt arXiv:1908.04734).
//!
//! Covers (TEST-FIRST, authored against the live gate surface), over a REAL Ssb SQLite db
//! (off-agent artifact, Layer-4 / R4):
//!   (1) a matching evaluator hash + a plausible (in-band) reward => PASS (NoTampering)
//!       with a real RewardTampering SSB row.
//!   (2) an evaluator hash MISMATCH => BLOCK (EvaluatorModified, leg 3a — the load-bearing
//!       case: the agent edited its own evaluator) + a real row.
//!   (3) an implausible reward delta with the hash INTACT => FLAG (ImplausibleRewardDelta,
//!       leg 3b — routed to human Layer-3) + a real row.
//!   (4) leg 3a SHORT-CIRCUITS leg 3b: a hash MISMATCH AND an implausible delta => the
//!       verdict is the hash-mismatch HARD-BLOCK (EvaluatorModified), dispositive (TD-3).
//!   (5) a missing / empty frozen baseline => BLOCK (EvaluatorModified, fail-closed R2 —
//!       cannot prove no-tampering).
//!   (6) SCHEMA_VERSION >= 236, RewardTampering in the action_type bijection.
//!
//! The reward-tampering gate guards self-mod but is itself in the frozen LOCKED set
//! (gate-manifest) so it cannot modify itself; H-27 (gate-selfcheck) validates its logic.
//! The IRREDUCIBLE top — the FIRST freeze of the reward baseline / the `gate_freeze_check.py
//! --lock` re-baseline — is routed to R8/human OUT-OF-BAND (the executing run cannot lock
//! its own gate). See workflow/reasoning/H-28-decisions.md.
//!
//! SCHEMA_VERSION assertions use `>=` (never an exact global pin) per the
//! BRITTLE_GLOBAL_SCHEMA_PIN gotcha. TOTAL_VARIANTS / ALL_VARIANTS presence is asserted by
//! action_type_lock.rs (compile-time exhaustiveness + count-drift) and the e45 contract.
//!
//! Grounding:
//!   - crates/orchestrator/src/reward_tampering.rs (H-28 gate; classify + check + emit).
//!   - crates/orchestrator/src/ssb.rs::ArchiveVerifyResult (the blake3 tripwire reused for
//!     leg 3a).
//!   - crates/cli/src/gate_selfcheck_cmd.rs (the W-1 done-gate template H-28 mirrors).

use std::sync::Arc;

use elai_core::clock::OsClock;
use elai_orchestrator::{
    reward_tampering::{check_reward_tampering, DEFAULT_PLAUSIBLE_REWARD_BAND},
    ssb::{
        ssb_action_type_from_str_public, RewardTamperingKind, Ssb, SsbActionType, SsbConfig,
        SsbPayload, SCHEMA_VERSION_PUBLIC,
    },
    SsbReader,
};
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

/// A planted FROZEN-evaluator baseline blake3 hash (the out-of-band recorded reference).
const BASELINE_HARNESS_BLAKE3: &str =
    "h28contract00000000000000000000000000000000000000000000000000ba";

/// An observed hash that deliberately != the baseline (the agent edited its evaluator).
const TAMPERED_OBSERVED_BLAKE3: &str =
    "tampered000000000000000000000000000000000000000000000000000000ff";

fn test_clock() -> Arc<dyn elai_core::clock::ClockProvider> {
    Arc::new(OsClock)
}

fn test_ssb(dir: &TempDir, project_id: &str) -> (Arc<Ssb>, std::path::PathBuf) {
    let db_path = dir.path().join("ssb.db");
    let cfg = SsbConfig {
        db_path: db_path.clone(),
        pool_size: Some(2),
        project_id: project_id.into(),
        session_id: format!("{project_id}-session"),
    };
    (Arc::new(Ssb::new(cfg, test_clock()).expect("ssb")), db_path)
}

/// Read the single RewardTampering row emitted by `check_reward_tampering`.
fn only_reward_tampering_row(ssb: &Arc<Ssb>) -> SsbPayload {
    let rows = ssb
        .query_by_type(SsbActionType::RewardTampering)
        .expect("query RewardTampering");
    assert_eq!(
        rows.len(),
        1,
        "exactly 1 RewardTampering SSB row (off-agent artifact, R4)"
    );
    rmp_serde::from_slice(&rows[0].payload).expect("deserialize payload")
}

// ---------------------------------------------------------------------------
// (1) matching hash + plausible reward => NoTampering PASS + off-agent row
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn matching_hash_in_band_is_no_tampering_and_emits_row() {
    let dir = TempDir::new().unwrap();
    let (ssb, db_path) = test_ssb(&dir, "h28-contract-1");

    let outcome = check_reward_tampering(
        &ssb,
        &db_path,
        BASELINE_HARNESS_BLAKE3,
        BASELINE_HARNESS_BLAKE3,
        0.50,
        0.55,
        DEFAULT_PLAUSIBLE_REWARD_BAND,
        "h28-contract-1",
        42,
    )
    .await
    .expect("check_reward_tampering must not error");

    assert_eq!(
        outcome.kind,
        RewardTamperingKind::NoTampering,
        "matching evaluator hash + in-band reward => current-RF PASS"
    );
    assert!(outcome.is_pass());
    assert!(!outcome.is_hard_block());

    if let SsbPayload::RewardTampering(p) = only_reward_tampering_row(&ssb) {
        assert_eq!(p.kind, RewardTamperingKind::NoTampering);
        assert_eq!(p.harness_blake3, BASELINE_HARNESS_BLAKE3);
        assert_eq!(p.corpus_blake3.len(), 64, "blake3 hex is 64 chars");
    } else {
        panic!("payload must be RewardTampering");
    }
}

// ---------------------------------------------------------------------------
// (2) hash MISMATCH => EvaluatorModified HARD-BLOCK (leg 3a, load-bearing) + row
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn hash_mismatch_is_evaluator_modified_hard_block_and_emits_row() {
    let dir = TempDir::new().unwrap();
    let (ssb, db_path) = test_ssb(&dir, "h28-contract-2");

    let outcome = check_reward_tampering(
        &ssb,
        &db_path,
        BASELINE_HARNESS_BLAKE3,
        TAMPERED_OBSERVED_BLAKE3,
        0.50,
        0.55,
        DEFAULT_PLAUSIBLE_REWARD_BAND,
        "h28-contract-2",
        42,
    )
    .await
    .expect("check_reward_tampering");

    assert_eq!(
        outcome.kind,
        RewardTamperingKind::EvaluatorModified,
        "a frozen-evaluator hash mismatch (the agent edited its own evaluator) must BLOCK"
    );
    assert!(!outcome.is_pass());
    assert!(outcome.is_hard_block(), "leg 3a is a HARD block, not a flag");

    if let SsbPayload::RewardTampering(p) = only_reward_tampering_row(&ssb) {
        assert_eq!(p.kind, RewardTamperingKind::EvaluatorModified);
        assert_eq!(p.harness_blake3, TAMPERED_OBSERVED_BLAKE3);
    } else {
        panic!("payload must be RewardTampering");
    }
}

// ---------------------------------------------------------------------------
// (3) implausible delta with hash INTACT => ImplausibleRewardDelta FLAG + row
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn implausible_delta_with_intact_hash_is_flag_and_emits_row() {
    let dir = TempDir::new().unwrap();
    let (ssb, db_path) = test_ssb(&dir, "h28-contract-3");

    // hash matches; reward jumps 0.10 -> 0.95 (delta 0.85 > 0.50 band).
    let outcome = check_reward_tampering(
        &ssb,
        &db_path,
        BASELINE_HARNESS_BLAKE3,
        BASELINE_HARNESS_BLAKE3,
        0.10,
        0.95,
        DEFAULT_PLAUSIBLE_REWARD_BAND,
        "h28-contract-3",
        42,
    )
    .await
    .expect("check_reward_tampering");

    assert_eq!(
        outcome.kind,
        RewardTamperingKind::ImplausibleRewardDelta,
        "an implausible reward jump with the hash intact must FLAG (RF-input tampering)"
    );
    assert!(!outcome.is_pass(), "a flag is not a promote");
    assert!(
        !outcome.is_hard_block(),
        "leg 3b is a FLAG -> human Layer-3, not a HARD block"
    );

    if let SsbPayload::RewardTampering(p) = only_reward_tampering_row(&ssb) {
        assert_eq!(p.kind, RewardTamperingKind::ImplausibleRewardDelta);
        assert!(
            p.reward_delta > p.plausible_band,
            "the flagged delta must exceed the locked band"
        );
    } else {
        panic!("payload must be RewardTampering");
    }
}

// ---------------------------------------------------------------------------
// (4) TD-3 ordering: a 3a hash mismatch is NOT overridden by a 3b green
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn leg_3a_hash_mismatch_short_circuits_leg_3b() {
    let dir = TempDir::new().unwrap();
    let (ssb, db_path) = test_ssb(&dir, "h28-contract-4");

    // BOTH a hash mismatch AND an implausible delta. The verdict must be the hash-mismatch
    // HARD-BLOCK (3a dispositive), never the 3b FLAG — a deterministic mismatch is final.
    let outcome = check_reward_tampering(
        &ssb,
        &db_path,
        BASELINE_HARNESS_BLAKE3,
        TAMPERED_OBSERVED_BLAKE3,
        0.10,
        0.99,
        DEFAULT_PLAUSIBLE_REWARD_BAND,
        "h28-contract-4",
        42,
    )
    .await
    .expect("check_reward_tampering");

    assert_eq!(
        outcome.kind,
        RewardTamperingKind::EvaluatorModified,
        "leg 3a (hash mismatch) must short-circuit leg 3b (implausible delta) — TD-3"
    );
    assert!(outcome.is_hard_block());
}

// ---------------------------------------------------------------------------
// (5) missing / empty frozen baseline => fail-closed EvaluatorModified BLOCK
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn missing_baseline_is_fail_closed_block() {
    let dir = TempDir::new().unwrap();
    let (ssb, db_path) = test_ssb(&dir, "h28-contract-5");

    // An empty baseline hash: the frozen reference could not be read => cannot prove
    // no-tampering => fail-closed (R2: not found = never ran = FAIL).
    let outcome = check_reward_tampering(
        &ssb,
        &db_path,
        "",
        BASELINE_HARNESS_BLAKE3,
        0.50,
        0.55,
        DEFAULT_PLAUSIBLE_REWARD_BAND,
        "h28-contract-5",
        42,
    )
    .await
    .expect("check_reward_tampering");

    assert_eq!(
        outcome.kind,
        RewardTamperingKind::EvaluatorModified,
        "a missing/empty frozen baseline must fail closed (cannot prove no-tampering)"
    );
    assert!(outcome.is_hard_block());
}

// ---------------------------------------------------------------------------
// (6) SSB lockstep: SCHEMA_VERSION >= 236, action_type bijection landed
// ---------------------------------------------------------------------------

#[test]
fn schema_version_and_variant_lockstep_landed() {
    assert!(
        SCHEMA_VERSION_PUBLIC >= 236,
        "SCHEMA_VERSION must be >= 236 after H-28; got {SCHEMA_VERSION_PUBLIC}"
    );

    // RewardTampering must round-trip through the action_type bijection.
    let at = ssb_action_type_from_str_public("reward_tampering")
        .expect("reward_tampering must be a known action_type");
    assert_eq!(at, SsbActionType::RewardTampering);
    assert_eq!(
        SsbActionType::RewardTampering.as_sql_str(),
        "reward_tampering"
    );
}

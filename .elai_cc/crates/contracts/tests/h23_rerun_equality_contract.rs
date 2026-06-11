//! H-23 contract tests — same-seed re-run hash-equality METRIC_GATE (T-7).
//!
//! Covers (TEST-FIRST, authored against the live gate surface):
//!   (1) two IDENTICAL fixture digests => PASS (Equal) with a real
//!       RerunHashEquality SSB row Kind=Equal landing in a real Ssb SQLite db
//!       (off-agent artifact, Layer-4 / R4).
//!   (2) two DIVERGENT digests => BLOCK (Divergent) with the FIRST divergent
//!       metric reported (Kind=Divergent).
//!   (3) MISSING SEED => BLOCK (hazard A): the gate NEVER defaults the seed to 0.
//!   (4) two EMPTY/trivial outputs => BLOCK not pass (hazard B): blake3 of empty
//!       is a fixed constant, so two empty runs must NOT hash Equal.
//!   (5) MISSING second run => BLOCK (fail-closed, W-1/R1).
//!   (6) SCHEMA_VERSION >= 228, RerunHashEquality in the action_type bijection
//!       (the +1 lockstep landed).
//!
//! SCHEMA_VERSION assertions use `>=` (never an exact global pin) per the
//! BRITTLE_GLOBAL_SCHEMA_PIN gotcha. TOTAL_VARIANTS / ALL_VARIANTS presence is
//! asserted by action_type_lock.rs (compile-time exhaustiveness + count-drift)
//! and the e45 contract.
//!
//! Grounding:
//!   - crates/orchestrator/src/rerun_equality.rs (H-23 Tier-1 gate).
//!   - crates/orchestrator/src/determinism.rs:364 (first-divergence-index, REUSED).
//!   - crates/orchestrator/src/rra.rs (blake3 16-hex digest convention, REUSED).

use std::sync::Arc;

use elai_core::clock::OsClock;
use elai_orchestrator::{
    rerun_equality::{verify_rerun_digest, RerunInput, RerunVerdictKind, RunDigests},
    ssb::{
        ssb_action_type_from_str_public, RerunHashEqualityKind, Ssb, SsbActionType, SsbConfig,
        SsbPayload, SCHEMA_VERSION_PUBLIC,
    },
    SsbReader,
};
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

fn test_clock() -> Arc<dyn elai_core::clock::ClockProvider> {
    Arc::new(OsClock)
}

fn test_ssb(dir: &TempDir, project_id: &str) -> Arc<Ssb> {
    let cfg = SsbConfig {
        db_path: dir.path().join("ssb.db"),
        pool_size: Some(2),
        project_id: project_id.into(),
        session_id: format!("{project_id}-session"),
    };
    Arc::new(Ssb::new(cfg, test_clock()).expect("ssb"))
}

fn rd(pairs: Vec<(&str, &str)>) -> RunDigests {
    RunDigests::from_pairs(
        pairs
            .into_iter()
            .map(|(n, d)| (n.to_string(), d.to_string()))
            .collect(),
    )
}

fn input(run_a: RunDigests, run_b: Option<RunDigests>, seed: Option<u64>) -> RerunInput {
    RerunInput {
        strategy_path: "exp.py".to_string(),
        run_a,
        run_b,
        seed,
    }
}

fn only_row(ssb: &Arc<Ssb>) -> SsbPayload {
    let rows = ssb
        .query_by_type(SsbActionType::RerunHashEquality)
        .expect("query RerunHashEquality");
    assert_eq!(
        rows.len(),
        1,
        "exactly 1 RerunHashEquality SSB row (off-agent artifact, R4)"
    );
    rmp_serde::from_slice(&rows[0].payload).expect("deserialize payload")
}

// ---------------------------------------------------------------------------
// (1) two identical digests => PASS (Equal) + off-agent SSB row Kind=Equal (R4)
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn identical_digests_pass_equal_and_emit_row() {
    let dir = TempDir::new().unwrap();
    let ssb = test_ssb(&dir, "h23-contract-1");

    let a = rd(vec![("sharpe", "aaaa"), ("max_drawdown", "bbbb")]);
    let b = rd(vec![("sharpe", "aaaa"), ("max_drawdown", "bbbb")]);

    let verdict = verify_rerun_digest(&ssb, input(a, Some(b), Some(42)))
        .await
        .expect("verify must not error");
    assert_eq!(
        verdict.kind,
        RerunVerdictKind::Equal,
        "two byte-identical same-seed re-runs must PASS Equal"
    );
    assert_eq!(verdict.reason, "equal");
    assert_eq!(verdict.declared_metric_count, 2);

    if let SsbPayload::RerunHashEquality(p) = only_row(&ssb) {
        assert_eq!(p.kind, RerunHashEqualityKind::Equal);
        assert_eq!(p.strategy_path, "exp.py");
        assert_eq!(p.verdict, "equal");
        assert_eq!(p.declared_metric_count, 2);
        assert_eq!(p.corpus_blake3.len(), 64, "blake3 hex is 64 chars");
        assert!(!p.digest_a.is_empty(), "Equal carries a combined digest_a");
        assert_eq!(p.digest_a, p.digest_b, "Equal => digest_a == digest_b");
    } else {
        panic!("payload must be RerunHashEquality");
    }
}

// ---------------------------------------------------------------------------
// (2) two divergent digests => BLOCK (Divergent) + first divergent metric named
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn divergent_digests_block_naming_first_metric() {
    let dir = TempDir::new().unwrap();
    let ssb = test_ssb(&dir, "h23-contract-2");

    let a = rd(vec![("sharpe", "aaaa"), ("max_drawdown", "bbbb")]);
    // run #2's max_drawdown digest differs (a hidden unseeded RNG).
    let b = rd(vec![("sharpe", "aaaa"), ("max_drawdown", "cccc")]);

    let verdict = verify_rerun_digest(&ssb, input(a, Some(b), Some(42)))
        .await
        .expect("verify");
    assert_eq!(verdict.kind, RerunVerdictKind::Divergent);
    assert_eq!(verdict.reason, "digest_divergent");
    assert_eq!(
        verdict.divergent_metric, "max_drawdown",
        "the FIRST divergent metric must be reported"
    );
    assert_eq!(verdict.first_divergence_index, 1);

    if let SsbPayload::RerunHashEquality(p) = only_row(&ssb) {
        assert_eq!(p.kind, RerunHashEqualityKind::Divergent);
        assert_eq!(p.divergent_metric, "max_drawdown");
        assert_eq!(p.first_divergence_index, 1);
        assert_eq!(p.verdict, "digest_divergent");
    } else {
        panic!("payload must be RerunHashEquality");
    }
}

// ---------------------------------------------------------------------------
// (3) missing seed => BLOCK (hazard A): the gate NEVER defaults seed to 0
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn missing_seed_is_fail_closed_block_hazard_a() {
    let dir = TempDir::new().unwrap();
    let ssb = test_ssb(&dir, "h23-contract-3");

    let a = rd(vec![("sharpe", "aaaa")]);
    let b = rd(vec![("sharpe", "aaaa")]);
    // seed=None (undeclared). The gate must Block, NOT silently default to 0.
    let verdict = verify_rerun_digest(&ssb, input(a, Some(b), None))
        .await
        .expect("verify");
    assert_eq!(
        verdict.kind,
        RerunVerdictKind::Divergent,
        "an undeclared seed must Block fail-closed (hazard A), NOT proceed with seed=0"
    );
    assert_ne!(
        verdict.kind,
        RerunVerdictKind::Equal,
        "an undeclared seed must never read as Equal (fail-open)"
    );
    assert_eq!(verdict.reason, "missing_seed");

    if let SsbPayload::RerunHashEquality(p) = only_row(&ssb) {
        assert_eq!(p.kind, RerunHashEqualityKind::Divergent);
        assert_eq!(p.verdict, "missing_seed");
    } else {
        panic!("payload must be RerunHashEquality");
    }
}

// ---------------------------------------------------------------------------
// (4) two empty/trivial outputs => BLOCK not pass (hazard B)
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn empty_outputs_are_fail_closed_block_hazard_b() {
    let dir = TempDir::new().unwrap();
    let ssb = test_ssb(&dir, "h23-contract-4a");

    // Two EMPTY runs would hash Equal (blake3 of empty is a fixed constant).
    let a = RunDigests::default();
    let b = RunDigests::default();
    let verdict = verify_rerun_digest(&ssb, input(a, Some(b), Some(42)))
        .await
        .expect("verify");
    assert_eq!(
        verdict.kind,
        RerunVerdictKind::Divergent,
        "two empty declared-output sets must Block (hazard B), NOT pass Equal"
    );
    assert_ne!(verdict.kind, RerunVerdictKind::Equal);
    assert_eq!(verdict.reason, "empty_declared_outputs");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn trivial_empty_digest_is_fail_closed_block_hazard_b() {
    let dir = TempDir::new().unwrap();
    let ssb = test_ssb(&dir, "h23-contract-4b");

    // A declared metric whose digest is empty/whitespace is trivial.
    let a = rd(vec![("sharpe", "")]);
    let b = rd(vec![("sharpe", "")]);
    let verdict = verify_rerun_digest(&ssb, input(a, Some(b), Some(42)))
        .await
        .expect("verify");
    assert_eq!(verdict.kind, RerunVerdictKind::Divergent);
    assert_eq!(verdict.reason, "empty_declared_outputs");
}

// ---------------------------------------------------------------------------
// (5) missing second run => BLOCK (fail-closed, W-1/R1)
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn missing_second_run_is_fail_closed_block() {
    let dir = TempDir::new().unwrap();
    let ssb = test_ssb(&dir, "h23-contract-5");

    let a = rd(vec![("sharpe", "aaaa")]);
    let verdict = verify_rerun_digest(&ssb, input(a, None, Some(42)))
        .await
        .expect("verify");
    assert_eq!(
        verdict.kind,
        RerunVerdictKind::Divergent,
        "a missing second run must Block (a single run cannot self-certify, W-1/R1)"
    );
    assert_eq!(verdict.reason, "missing_second_run");

    if let SsbPayload::RerunHashEquality(p) = only_row(&ssb) {
        assert_eq!(p.kind, RerunHashEqualityKind::Divergent);
        assert_eq!(p.verdict, "missing_second_run");
    } else {
        panic!("payload must be RerunHashEquality");
    }
}

// ---------------------------------------------------------------------------
// (6) SSB lockstep: SCHEMA_VERSION >= 228, action_type bijection landed
// ---------------------------------------------------------------------------

#[test]
fn schema_version_and_variant_lockstep_landed() {
    assert!(
        SCHEMA_VERSION_PUBLIC >= 228,
        "SCHEMA_VERSION must be >= 228 after H-23; got {SCHEMA_VERSION_PUBLIC}"
    );

    // RerunHashEquality must round-trip through the action_type bijection.
    let at = ssb_action_type_from_str_public("rerun_hash_equality")
        .expect("rerun_hash_equality must be a known action_type");
    assert_eq!(at, SsbActionType::RerunHashEquality);
    assert_eq!(
        SsbActionType::RerunHashEquality.as_sql_str(),
        "rerun_hash_equality"
    );
}

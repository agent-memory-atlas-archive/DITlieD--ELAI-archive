//! H-21 contract tests — runtime look-ahead / causality DIFFERENTIAL gate (T-3).
//!
//! Covers (TEST-FIRST, authored against the live gate surface):
//!   (1) a future-leak fixture is BLOCKED with the first divergent column
//!       reported AND a real LookaheadDifferential SSB row with Kind=FutureLeak
//!       lands in a real Ssb SQLite db (off-agent artifact, Layer-4 / R4).
//!   (2) a causal fixture PASSES with a Causal row.
//!   (3) a missing/empty truncated set => Block (fail-closed), NOT pass (R1/R2).
//!   (4) a shape mismatch => Block.
//!   (5) a column-set mismatch (extra/missing column) => Block.
//!   (6) FP noise within tolerance does NOT block, but a real divergence does.
//!   (7) SCHEMA_VERSION >= 226, TOTAL_VARIANTS >= 503, LookaheadDifferential in
//!       the action_type bijection (the +1 lockstep landed).
//!
//! SCHEMA_VERSION assertions use `>=` (never an exact global pin) per the
//! BRITTLE_GLOBAL_SCHEMA_PIN gotcha.
//!
//! Grounding:
//!   - crates/orchestrator/src/lookahead_differential.rs (H-21 Tier-1 gate).
//!   - crates/orchestrator/src/determinism.rs:364 (first-divergence-index shape).
//!   - github.com/freqtrade/freqtrade lookahead.py (perturb-and-diff method).

use std::sync::Arc;

use elai_core::clock::OsClock;
use elai_orchestrator::{
    lookahead_differential::{
        diff_future_truncation, ColumnSet, FpTolerance, LookaheadInput, LookaheadVerdictKind,
    },
    ssb::{
        ssb_action_type_from_str_public, LookaheadDifferentialKind, Ssb, SsbActionType, SsbConfig,
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

fn cs(pairs: Vec<(&str, Vec<f64>)>) -> ColumnSet {
    ColumnSet::from_pairs(pairs.into_iter().map(|(n, v)| (n.to_string(), v)).collect())
}

fn input(full: ColumnSet, truncated: Option<ColumnSet>) -> LookaheadInput {
    LookaheadInput {
        strategy_path: "strat.py".to_string(),
        full_run: full,
        truncated_run: truncated,
        tolerance: FpTolerance::default(),
        seed: 42,
    }
}

// ---------------------------------------------------------------------------
// (1) future-leak => BLOCK + first divergent column + off-agent SSB row (R4)
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn future_leak_blocks_and_emits_ssb_row() {
    let dir = TempDir::new().unwrap();
    let ssb = test_ssb(&dir, "h21-contract-1");

    // 'leaky' reads iloc[t+1]: row 2 changes when the future tail is truncated.
    let full = cs(vec![
        ("causal", vec![10.0, 11.0, 12.0]),
        ("leaky", vec![1.0, 2.0, 99.0]),
    ]);
    let truncated = cs(vec![
        ("causal", vec![10.0, 11.0, 12.0]),
        ("leaky", vec![1.0, 2.0, 3.0]),
    ]);

    let verdict = diff_future_truncation(&ssb, input(full, Some(truncated)))
        .await
        .expect("diff must not error");

    let first = match &verdict.kind {
        LookaheadVerdictKind::FutureLeak(fc) => fc.clone(),
        other => panic!("expected FutureLeak, got {other:?}"),
    };
    assert_eq!(first.column, "leaky", "first divergent column must be reported");
    assert_eq!(first.index, 2, "first divergent row index must be reported");

    // Off-agent artifact: exactly one LookaheadDifferential row landed.
    let rows = ssb
        .query_by_type(SsbActionType::LookaheadDifferential)
        .expect("query LookaheadDifferential");
    assert_eq!(
        rows.len(),
        1,
        "exactly 1 LookaheadDifferential SSB row (off-agent artifact, R4)"
    );

    let payload: SsbPayload =
        rmp_serde::from_slice(&rows[0].payload).expect("deserialize payload");
    if let SsbPayload::LookaheadDifferential(p) = payload {
        assert_eq!(p.kind, LookaheadDifferentialKind::FutureLeak);
        assert_eq!(p.strategy_path, "strat.py");
        assert_eq!(p.first_change_column, "leaky");
        assert_eq!(p.first_change_index, 2);
        assert_eq!(p.verdict, "future_leak");
        assert_eq!(p.corpus_blake3.len(), 64, "blake3 hex is 64 chars");
    } else {
        panic!("payload must be LookaheadDifferential");
    }
}

// ---------------------------------------------------------------------------
// (2) causal fixture => Allow + Causal row
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn causal_fixture_passes_with_causal_row() {
    let dir = TempDir::new().unwrap();
    let ssb = test_ssb(&dir, "h21-contract-2");

    let full = cs(vec![("sma", vec![1.0, 2.0, 3.0]), ("close", vec![10.0, 11.0, 12.0])]);
    let truncated = cs(vec![("sma", vec![1.0, 2.0, 3.0]), ("close", vec![10.0, 11.0, 12.0])]);

    let verdict = diff_future_truncation(&ssb, input(full, Some(truncated)))
        .await
        .expect("diff");
    assert_eq!(verdict.kind, LookaheadVerdictKind::Causal);

    // Allow STILL emits an off-agent row (every decision is recorded).
    let rows = ssb
        .query_by_type(SsbActionType::LookaheadDifferential)
        .expect("query LookaheadDifferential");
    assert_eq!(rows.len(), 1, "Causal decision also emits one SSB row");
    let payload: SsbPayload =
        rmp_serde::from_slice(&rows[0].payload).expect("deserialize payload");
    if let SsbPayload::LookaheadDifferential(p) = payload {
        assert_eq!(p.kind, LookaheadDifferentialKind::Causal);
        assert_eq!(p.verdict, "causal");
    } else {
        panic!("payload must be LookaheadDifferential");
    }
}

// ---------------------------------------------------------------------------
// (3) missing/empty truncated set => Block (fail-closed), NOT pass (R1/R2)
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn missing_truncated_is_fail_closed_block_not_pass() {
    let dir = TempDir::new().unwrap();
    let ssb = test_ssb(&dir, "h21-contract-3a");

    let full = cs(vec![("rsi", vec![1.0, 2.0, 3.0])]);
    let verdict = diff_future_truncation(&ssb, input(full, None))
        .await
        .expect("diff");
    assert_eq!(
        verdict.kind,
        LookaheadVerdictKind::Unprovable,
        "missing truncated run must Block fail-closed, NOT pass"
    );
    assert_ne!(
        verdict.kind,
        LookaheadVerdictKind::Causal,
        "missing truncated run must never read as Causal (fail-open)"
    );
    assert_eq!(verdict.reason, "missing_truncated");

    let rows = ssb
        .query_by_type(SsbActionType::LookaheadDifferential)
        .expect("query");
    assert_eq!(rows.len(), 1, "fail-closed decision still emits one SSB row");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn empty_truncated_is_fail_closed_block() {
    let dir = TempDir::new().unwrap();
    let ssb = test_ssb(&dir, "h21-contract-3b");
    let full = cs(vec![("rsi", vec![1.0, 2.0])]);
    let verdict = diff_future_truncation(&ssb, input(full, Some(ColumnSet::default())))
        .await
        .expect("diff");
    assert_eq!(verdict.kind, LookaheadVerdictKind::Unprovable);
    assert_eq!(verdict.reason, "missing_truncated");
}

// ---------------------------------------------------------------------------
// (4) shape mismatch => Block
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn shape_mismatch_is_block() {
    let dir = TempDir::new().unwrap();
    let ssb = test_ssb(&dir, "h21-contract-4");
    let full = cs(vec![("a", vec![1.0, 2.0, 3.0])]);
    let truncated = cs(vec![("a", vec![1.0, 2.0])]);
    let verdict = diff_future_truncation(&ssb, input(full, Some(truncated)))
        .await
        .expect("diff");
    assert_eq!(verdict.kind, LookaheadVerdictKind::Unprovable);
    assert_eq!(verdict.reason, "shape_mismatch");
}

// ---------------------------------------------------------------------------
// (5) column-set mismatch (extra/missing column) => Block
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn column_set_width_mismatch_is_block() {
    let dir = TempDir::new().unwrap();
    let ssb = test_ssb(&dir, "h21-contract-5a");
    let full = cs(vec![("a", vec![1.0]), ("b", vec![2.0])]);
    let truncated = cs(vec![("a", vec![1.0])]);
    let verdict = diff_future_truncation(&ssb, input(full, Some(truncated)))
        .await
        .expect("diff");
    assert_eq!(verdict.kind, LookaheadVerdictKind::Unprovable);
    assert_eq!(verdict.reason, "column_set_mismatch");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn column_set_name_mismatch_is_block() {
    let dir = TempDir::new().unwrap();
    let ssb = test_ssb(&dir, "h21-contract-5b");
    let full = cs(vec![("a", vec![1.0])]);
    let truncated = cs(vec![("z", vec![1.0])]);
    let verdict = diff_future_truncation(&ssb, input(full, Some(truncated)))
        .await
        .expect("diff");
    assert_eq!(verdict.kind, LookaheadVerdictKind::Unprovable);
    assert_eq!(verdict.reason, "column_set_mismatch");
}

// ---------------------------------------------------------------------------
// (6) FP noise within tolerance does NOT block; a real divergence does
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn fp_noise_within_tolerance_passes_but_real_divergence_blocks() {
    let dir = TempDir::new().unwrap();
    let ssb = test_ssb(&dir, "h21-contract-6");

    // Sub-1e-9 relative jitter (cross-hardware FP residue) must NOT block.
    let full = cs(vec![("x", vec![1.0, 100.0, 1000.0])]);
    let truncated = cs(vec![("x", vec![1.0 + 0.5e-9, 100.0 + 5e-8, 1000.0 + 5e-7])]);
    let verdict = diff_future_truncation(&ssb, input(full, Some(truncated)))
        .await
        .expect("diff");
    assert_eq!(
        verdict.kind,
        LookaheadVerdictKind::Causal,
        "sub-tolerance FP noise must pass"
    );

    // A real divergence (>> 1e-9 relative) at the same scale must block.
    let dir2 = TempDir::new().unwrap();
    let ssb2 = test_ssb(&dir2, "h21-contract-6b");
    let full2 = cs(vec![("x", vec![1.0, 100.0, 1000.0])]);
    let truncated2 = cs(vec![("x", vec![1.0, 100.0, 1000.0 + 1.0])]);
    let verdict2 = diff_future_truncation(&ssb2, input(full2, Some(truncated2)))
        .await
        .expect("diff");
    match verdict2.kind {
        LookaheadVerdictKind::FutureLeak(fc) => {
            assert_eq!(fc.column, "x");
            assert_eq!(fc.index, 2);
        }
        other => panic!("a real O(1) divergence must block, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// (7) SSB lockstep: SCHEMA_VERSION >= 226, action_type bijection landed
// ---------------------------------------------------------------------------

#[test]
fn schema_version_and_variant_lockstep_landed() {
    assert!(
        SCHEMA_VERSION_PUBLIC >= 226,
        "SCHEMA_VERSION must be >= 226 after H-21; got {SCHEMA_VERSION_PUBLIC}"
    );

    // LookaheadDifferential must round-trip through the action_type bijection.
    let at = ssb_action_type_from_str_public("lookahead_differential")
        .expect("lookahead_differential must be a known action_type");
    assert_eq!(at, SsbActionType::LookaheadDifferential);
    assert_eq!(
        SsbActionType::LookaheadDifferential.as_sql_str(),
        "lookahead_differential"
    );
    // TOTAL_VARIANTS >= 503 / ALL_VARIANTS presence is asserted in
    // action_type_lock.rs (compile-time exhaustiveness + count-drift) and the
    // e45 contract; here we assert the action_type string side of the lockstep.
}

//! H-22 contract tests — trading metamorphic NULL-TEST gate (T-6).
//!
//! Covers (TEST-FIRST, authored against the live gate surface):
//!   (1) p = (C+1)/(K+1) exact (never 0; C=0 -> 1/(K+1)).
//!   (2) an OVERFIT fixture (edge survives the block-permutation null) is
//!       BLOCKED with a real TradingNullTest SSB row Kind=EdgeSurvivedNull
//!       (off-agent artifact, Layer-4 / R4).
//!   (3) a GENUINE fixture (significant vs the null) PASSES with a real
//!       TradingNullTest row Kind=EdgeCollapsedToNull.
//!   (4) K below the resolution floor => Block (fail-closed, under-powered).
//!   (5) TrialCountMissing (no H-17 N) => Block (TD-1).
//!   (6) a NaN/degenerate statistic => Block, NOT a panic.
//!   (7) block permutation preserves series length + is seed-deterministic
//!       (same seed -> same null distribution).
//!   (8) SCHEMA_VERSION >= 227, action_type bijection landed (TradingNullTest).
//!
//! SCHEMA_VERSION assertions use `>=` (never an exact global pin) per the
//! BRITTLE_GLOBAL_SCHEMA_PIN gotcha.
//!
//! Grounding:
//!   - crates/orchestrator/src/trading_null_gate.rs (H-22 Tier-1 gate).
//!   - workflow/papers/ojala-garriga-permutation-test-interface.md (p-value form).
//!   - crates/orchestrator/src/trial_ledger.rs:332 (H-17 N read).

use std::sync::Arc;

use elai_core::clock::OsClock;
use elai_orchestrator::{
    ssb::{
        ssb_action_type_from_str_public, Ssb, SsbActionType, SsbConfig, SsbPayload,
        TradingNullTestKind, SCHEMA_VERSION_PUBLIC,
    },
    trading_null_gate::{
        block_permute, build_null_distribution, evaluate_trading_null, k_floor,
        permutation_p_value, TradingNullInput, TradingNullVerdictKind, ALPHA, DEFAULT_BLOCK_SIZE,
        K_MIN_PERMUTATIONS,
    },
    trial_ledger::TrialLedger,
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

fn test_ledger(dir: &TempDir, project_id: &str) -> TrialLedger {
    TrialLedger::open(&dir.path().join("ssb.db"), format!("{project_id}-session"))
        .expect("trial ledger open")
}

/// Register exactly one distinct trial so N is Present (not TrialCountMissing).
async fn register_one_trial(
    ledger: &TrialLedger,
    ssb: &Arc<Ssb>,
    objective: &str,
    window: &str,
) {
    let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
    ledger
        .register_trial(ssb, &clock, objective, window, b"{\"strat\": 1}", 7)
        .await
        .expect("register trial");
}

const OBJECTIVE: &str = "sharpe_1y";
const WINDOW: &str = "0..1";
const SERIES_LEN: usize = 120;

/// Sign-alternating market returns with VARYING magnitude `(i%3+1)`. The varying
/// magnitude gives the aligned strategy P&L a positive variance (a constant-
/// magnitude market would make the perfectly-aligned P&L a zero-variance
/// constant -> degenerate NaN). The OUTCOME the null permutes.
fn market_returns() -> Vec<f64> {
    (0..SERIES_LEN)
        .map(|i| {
            let mag = ((i % 3) + 1) as f64;
            if i % 2 == 0 {
                mag
            } else {
                -mag
            }
        })
        .collect()
}

/// Overfit fixture: a constant-long signal (always +1) on the zero-mean
/// alternating market. P&L = market (zero-mean); permutation preserves the
/// zero-mean -> observed inside the null band -> survives the null -> BLOCK.
fn overfit_inputs() -> (Vec<f64>, Vec<f64>) {
    let returns = market_returns();
    (vec![1.0; SERIES_LEN], returns)
}

/// Genuine fixture: a perfectly-aligned signal (sign of THIS bar's return). P&L
/// = |return| > 0 (variable magnitude) -> high positive Sharpe; permuting the
/// returns against the fixed signal destroys the alignment -> null Sharpes near
/// zero -> observed in the tail -> significant -> ALLOW.
fn genuine_inputs() -> (Vec<f64>, Vec<f64>) {
    let returns = market_returns();
    let signals: Vec<f64> = returns
        .iter()
        .map(|&r| if r >= 0.0 { 1.0 } else { -1.0 })
        .collect();
    (signals, returns)
}

fn input(signals: Vec<f64>, returns: Vec<f64>, k: usize) -> TradingNullInput {
    TradingNullInput {
        strategy_path: "strat.py".to_string(),
        objective: OBJECTIVE.to_string(),
        data_window: WINDOW.to_string(),
        signals,
        returns,
        k_permutations: k,
        block_size: DEFAULT_BLOCK_SIZE,
        seed: 42,
    }
}

// ---------------------------------------------------------------------------
// (1) p = (C+1)/(K+1) exact, never 0
// ---------------------------------------------------------------------------

#[test]
fn p_value_uses_plus_one_correction_and_is_never_zero() {
    // C=0 (no null >= observed): p must be 1/(K+1), never 0.
    let null = vec![0.0_f64; 99];
    let p = permutation_p_value(1000.0, &null);
    assert!((p - 1.0 / 100.0).abs() < 1e-12, "p must be 1/(K+1) when C=0; got {p}");
    assert!(p > 0.0, "permutation p-value can never be exactly 0");

    // C=K (every null >= observed): p must be (K+1)/(K+1) = 1.0.
    let null_hi = vec![10.0_f64; 50];
    let p_hi = permutation_p_value(-5.0, &null_hi);
    assert!((p_hi - 1.0).abs() < 1e-12, "p must be 1.0 when all null >= observed; got {p_hi}");
}

// ---------------------------------------------------------------------------
// (2) overfit fixture => BLOCK(EdgeSurvivedNull) + off-agent SSB row (R4)
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn overfit_edge_survives_null_blocks_and_emits_ssb_row() {
    let dir = TempDir::new().unwrap();
    let ssb = test_ssb(&dir, "h22-contract-2");
    let ledger = test_ledger(&dir, "h22-contract-2");
    register_one_trial(&ledger, &ssb, OBJECTIVE, WINDOW).await;

    let (sig, ret) = overfit_inputs();
    let verdict = evaluate_trading_null(&ledger, &ssb, input(sig, ret, K_MIN_PERMUTATIONS))
        .await
        .expect("gate must not error");

    match &verdict.kind {
        TradingNullVerdictKind::EdgeSurvivedNull { p_value, threshold } => {
            assert!(
                p_value >= threshold,
                "an edge that survives the null has p >= alpha/N; got p={p_value} thr={threshold}"
            );
        }
        other => panic!("overfit fixture must be EdgeSurvivedNull (Block), got {other:?}"),
    }
    assert_eq!(verdict.reason, "edge_survived_null");

    // Off-agent artifact: exactly one TradingNullTest row landed.
    let rows = ssb
        .query_by_type(SsbActionType::TradingNullTest)
        .expect("query TradingNullTest");
    assert_eq!(rows.len(), 1, "exactly 1 TradingNullTest SSB row (off-agent artifact, R4)");

    let payload: SsbPayload =
        rmp_serde::from_slice(&rows[0].payload).expect("deserialize payload");
    if let SsbPayload::TradingNullTest(p) = payload {
        assert_eq!(p.kind, TradingNullTestKind::EdgeSurvivedNull);
        assert_eq!(p.strategy_path, "strat.py");
        assert_eq!(p.verdict, "edge_survived_null");
        assert_eq!(p.n_trials, 1, "N read from H-17 (one registered trial)");
        assert_eq!(p.k_permutations, K_MIN_PERMUTATIONS as u64);
        assert!((p.alpha - ALPHA).abs() < 1e-12, "alpha is the LOCKED constant");
        assert_eq!(p.corpus_blake3.len(), 64, "blake3 hex is 64 chars");
    } else {
        panic!("payload must be TradingNullTest");
    }
}

// ---------------------------------------------------------------------------
// (3) genuine fixture => ALLOW(EdgeCollapsedToNull) + Allow row
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn genuine_edge_collapses_to_null_passes() {
    let dir = TempDir::new().unwrap();
    let ssb = test_ssb(&dir, "h22-contract-3");
    let ledger = test_ledger(&dir, "h22-contract-3");
    register_one_trial(&ledger, &ssb, OBJECTIVE, WINDOW).await;

    let (sig, ret) = genuine_inputs();
    let verdict = evaluate_trading_null(&ledger, &ssb, input(sig, ret, K_MIN_PERMUTATIONS))
        .await
        .expect("gate");
    assert_eq!(
        verdict.kind,
        TradingNullVerdictKind::EdgeCollapsedToNull,
        "a genuine edge must be significant vs the null (Allow); reason={}",
        verdict.reason
    );
    assert!(
        verdict.p_value < verdict.threshold,
        "significant edge has p < alpha/N; got p={} thr={}",
        verdict.p_value,
        verdict.threshold
    );

    // Allow STILL emits an off-agent row.
    let rows = ssb
        .query_by_type(SsbActionType::TradingNullTest)
        .expect("query");
    assert_eq!(rows.len(), 1, "Allow decision also emits one SSB row");
    let payload: SsbPayload =
        rmp_serde::from_slice(&rows[0].payload).expect("deserialize payload");
    if let SsbPayload::TradingNullTest(p) = payload {
        assert_eq!(p.kind, TradingNullTestKind::EdgeCollapsedToNull);
        assert_eq!(p.verdict, "edge_collapsed_to_null");
    } else {
        panic!("payload must be TradingNullTest");
    }
}

// ---------------------------------------------------------------------------
// (4) K below the resolution floor => Block (fail-closed, under-powered)
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn under_powered_k_is_fail_closed_block() {
    let dir = TempDir::new().unwrap();
    let ssb = test_ssb(&dir, "h22-contract-4");
    let ledger = test_ledger(&dir, "h22-contract-4");
    register_one_trial(&ledger, &ssb, OBJECTIVE, WINDOW).await;

    // K below ceil(1/alpha)-1 = 19 for alpha=0.05.
    let floor = k_floor(ALPHA);
    assert_eq!(floor, 19, "alpha=0.05 K-floor is 19");
    let (sig, ret) = genuine_inputs();
    let verdict = evaluate_trading_null(&ledger, &ssb, input(sig, ret, floor - 1))
        .await
        .expect("gate");
    assert_eq!(
        verdict.kind,
        TradingNullVerdictKind::Unprovable,
        "an under-powered null must Block fail-closed, NOT pass"
    );
    assert_eq!(verdict.reason, "block_insufficient_permutations");

    let rows = ssb
        .query_by_type(SsbActionType::TradingNullTest)
        .expect("query");
    assert_eq!(rows.len(), 1, "fail-closed decision still emits one SSB row");
    let payload: SsbPayload =
        rmp_serde::from_slice(&rows[0].payload).expect("deserialize payload");
    if let SsbPayload::TradingNullTest(p) = payload {
        assert_eq!(p.kind, TradingNullTestKind::Unprovable);
    } else {
        panic!("payload must be TradingNullTest");
    }
}

// ---------------------------------------------------------------------------
// (5) TrialCountMissing (no H-17 N) => Block (TD-1)
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn trial_count_missing_is_fail_closed_block() {
    let dir = TempDir::new().unwrap();
    let ssb = test_ssb(&dir, "h22-contract-5");
    let ledger = test_ledger(&dir, "h22-contract-5");
    // NO trial registered -> trial_count is TrialCountMissing.

    let (sig, ret) = genuine_inputs();
    let verdict = evaluate_trading_null(&ledger, &ssb, input(sig, ret, K_MIN_PERMUTATIONS))
        .await
        .expect("gate");
    assert_eq!(
        verdict.kind,
        TradingNullVerdictKind::Unprovable,
        "TrialCountMissing must Block fail-closed (TD-1), NOT pass"
    );
    assert_eq!(verdict.reason, "block_trial_count_missing");
    assert_eq!(verdict.n_trials, 0, "n_trials is 0 on the TrialCountMissing path");
}

// ---------------------------------------------------------------------------
// (6) NaN/degenerate statistic => Block, NOT a panic
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn degenerate_statistic_is_block_not_panic() {
    let dir = TempDir::new().unwrap();
    let ssb = test_ssb(&dir, "h22-contract-6");
    let ledger = test_ledger(&dir, "h22-contract-6");
    register_one_trial(&ledger, &ssb, OBJECTIVE, WINDOW).await;

    // A constant signal on a constant return -> constant P&L -> zero variance ->
    // Sharpe is NaN -> fail-closed Block (never a panic).
    let constant_sig = vec![1.0_f64; SERIES_LEN];
    let constant_ret = vec![3.0_f64; SERIES_LEN];
    let verdict =
        evaluate_trading_null(&ledger, &ssb, input(constant_sig, constant_ret, K_MIN_PERMUTATIONS))
            .await
            .expect("gate must return a Block verdict, never panic");
    assert_eq!(verdict.kind, TradingNullVerdictKind::Unprovable);
    assert_eq!(verdict.reason, "block_nan_statistic");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn missing_series_is_block() {
    let dir = TempDir::new().unwrap();
    let ssb = test_ssb(&dir, "h22-contract-6b");
    let ledger = test_ledger(&dir, "h22-contract-6b");
    register_one_trial(&ledger, &ssb, OBJECTIVE, WINDOW).await;

    // Empty series -> missing-evidence fail-closed Block.
    let verdict =
        evaluate_trading_null(&ledger, &ssb, input(Vec::new(), Vec::new(), K_MIN_PERMUTATIONS))
            .await
            .expect("gate");
    assert_eq!(verdict.kind, TradingNullVerdictKind::Unprovable);
    assert_eq!(verdict.reason, "block_missing_series");
}

// ---------------------------------------------------------------------------
// (7) block permutation preserves length + is seed-deterministic
// ---------------------------------------------------------------------------

#[test]
fn block_permutation_preserves_length_and_is_seed_deterministic() {
    let series: Vec<f64> = (0..SERIES_LEN).map(|i| (i as f64) * 0.5 - 30.0).collect();
    let a = block_permute(&series, DEFAULT_BLOCK_SIZE, 42);
    let b = block_permute(&series, DEFAULT_BLOCK_SIZE, 42);
    assert_eq!(a.len(), series.len(), "permutation preserves series length");
    assert_eq!(a, b, "same seed -> identical permutation (GD-2)");
    assert_ne!(
        a,
        block_permute(&series, DEFAULT_BLOCK_SIZE, 43),
        "different seed -> different permutation"
    );

    // The whole null distribution is seed-deterministic too.
    let signals: Vec<f64> = (0..SERIES_LEN)
        .map(|i| if i % 2 == 0 { 1.0 } else { -1.0 })
        .collect();
    let n1 = build_null_distribution(&signals, &series, 64, DEFAULT_BLOCK_SIZE, 9);
    let n2 = build_null_distribution(&signals, &series, 64, DEFAULT_BLOCK_SIZE, 9);
    assert_eq!(n1, n2, "same seed -> identical null distribution (GD-2)");
}

// ---------------------------------------------------------------------------
// (8) SSB lockstep: SCHEMA_VERSION >= 227, action_type bijection landed
// ---------------------------------------------------------------------------

#[test]
fn schema_version_and_variant_lockstep_landed() {
    assert!(
        SCHEMA_VERSION_PUBLIC >= 227,
        "SCHEMA_VERSION must be >= 227 after H-22; got {SCHEMA_VERSION_PUBLIC}"
    );

    let at = ssb_action_type_from_str_public("trading_null_test")
        .expect("trading_null_test must be a known action_type");
    assert_eq!(at, SsbActionType::TradingNullTest);
    assert_eq!(
        SsbActionType::TradingNullTest.as_sql_str(),
        "trading_null_test"
    );
    // TOTAL_VARIANTS >= 504 / ALL_VARIANTS presence is asserted in
    // action_type_lock.rs (compile-time exhaustiveness + count-drift) and the
    // e45 contract; here we assert the action_type string side of the lockstep.
}

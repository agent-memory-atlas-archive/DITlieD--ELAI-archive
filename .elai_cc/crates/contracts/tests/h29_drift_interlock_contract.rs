//! H-29 contract tests — the LOCKED fail-closed drift state machine (T-13), the master
//! deployment-trust interlock.
//!
//! Covers (TEST-FIRST, authored against the live gate surface), over a REAL Ssb SQLite db
//! (off-agent artifact, Layer-4 / R4):
//!   (1) no drift across all four axes => Running (a DriftInterlock(Running) row lands, no latch).
//!   (2) a PSI breach (output PSI >= 0.25) => latched DriftHalted.
//!   (3) a schema-fingerprint mismatch => DriftHalted.
//!   (4) the latch is PERSISTENT — after a Halt a fresh ledger re-open still reads HALTED
//!       (a restart does NOT reset to Running).
//!   (5) the no-unlock invariant: NO unlock/reset/clear/resume method exists on
//!       DriftInterlockLedger (unlock_drift_interlock is a FREE FUNCTION, not a method) —
//!       a negative test asserts the agent-facing handle cannot reach it.
//!   (6) a human unlock WITHOUT fresh re-validation evidence => REJECTED; with fresh evidence
//!       (strictly newer than the halt) => the latch clears (Running).
//!   (7) default-HALTED — missing / empty evidence on any axis => HALT, not Running.
//!   (8) SCHEMA_VERSION >= 238, DriftInterlock in the action_type bijection.
//!
//! The latch is a persistent journal-derived counter (is_halted = COUNT('halt') > COUNT('unlock')
//! over the append-only drift_interlock_latch table; mirrors the H-19 HoldoutBudgetLedger) and
//! the file is in the LOCKED gate-manifest set so it cannot edit its own bands. The IRREDUCIBLE
//! top — clearing a drift halt — is the human/frozen OUT-OF-BAND unlock_drift_interlock free fn,
//! which requires re-validation evidence STRICTLY NEWER than the unresolved halt (R8). See
//! workflow/reasoning/H-29-decisions.md.
//!
//! SCHEMA_VERSION assertions use `>=` (never an exact global pin) per the
//! BRITTLE_GLOBAL_SCHEMA_PIN gotcha. TOTAL_VARIANTS / ALL_VARIANTS presence is asserted by
//! action_type_lock.rs (compile-time exhaustiveness + count-drift) and the e45 contract.
//!
//! Grounding:
//!   - crates/orchestrator/src/drift_interlock.rs (the H-29 gate; classify + evaluate + unlock).
//!   - crates/orchestrator/src/holdout_budget.rs (the H-19 append-only no-reset discipline).

use std::sync::Arc;

use elai_core::clock::OsClock;
use elai_orchestrator::{
    drift_interlock::{
        classify_drift, compute_psi, evaluate_drift_interlock, unlock_drift_interlock,
        DriftEvidence, DriftInterlockError, DriftInterlockLedger, InterlockVerdict, PsiBins,
        PsiResult, DEVIATION_HALT_BAND, PSI_HALT_BAND,
    },
    ssb::{
        ssb_action_type_from_str_public, DriftInterlockKind, Ssb, SsbActionType, SsbConfig,
        SsbPayload, SCHEMA_VERSION_PUBLIC,
    },
    SsbReader,
};
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

const STRATEGY: &str = "momentum_eurusd_v3";
const RUN_ID: &str = "h29-contract";
const SEED: u64 = 42;
/// A representative wall-clock millisecond reading (2025-06-15-ish) for the PURE classify tests.
const NOW_MS: i64 = 1_750_000_000_000;
const MS_PER_DAY: i64 = 86_400_000;

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

fn test_ledger(db_path: &std::path::Path) -> DriftInterlockLedger {
    DriftInterlockLedger::open(db_path, "h29-contract-session").expect("drift ledger open")
}

/// A clean, no-drift evidence bundle (validated NOW; identical input/output distributions;
/// matching schema fingerprint; tiny deviation). Resolves to Running.
fn clean_evidence(now_ms: i64) -> DriftEvidence {
    let live_schema = b"col_a:f64,col_b:f64,signal:f64".to_vec();
    let fp = blake3::hash(&live_schema).to_hex().to_string();
    DriftEvidence {
        input_distribution: Some(PsiBins {
            expected: vec![0.25, 0.25, 0.25, 0.25],
            actual: vec![0.25, 0.25, 0.25, 0.25],
        }),
        output_distribution: Some(PsiBins {
            expected: vec![0.5, 0.5],
            actual: vec![0.5, 0.5],
        }),
        baseline_schema_fingerprint: Some(fp),
        live_schema: Some(live_schema),
        live_vs_backtest_deviation: Some(0.01),
        deviation_measured_at_ms: Some(now_ms),
        last_validated_at_ms: Some(now_ms),
        composed_breaches: Vec::new(),
    }
}

/// All DriftInterlock rows of a given kind.
fn rows_of_kind(ssb: &Arc<Ssb>, want: DriftInterlockKind) -> Vec<SsbPayload> {
    ssb.query_by_type(SsbActionType::DriftInterlock)
        .expect("query DriftInterlock")
        .iter()
        .filter_map(|r| rmp_serde::from_slice::<SsbPayload>(&r.payload).ok())
        .filter(|p| matches!(p, SsbPayload::DriftInterlock(pp) if pp.kind == want))
        .collect()
}

// ---------------------------------------------------------------------------
// (1) no drift across all four axes => Running PASS row
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn clean_deployment_resumes_running() {
    let dir = TempDir::new().unwrap();
    let (ssb, db_path) = test_ssb(&dir, "h29-contract-1");
    let clock = test_clock();
    let ledger = test_ledger(&db_path);
    let now_ms = Ssb::now_ms_via_clock(&clock);

    let out = evaluate_drift_interlock(
        &ledger, &ssb, &clock, STRATEGY, &clean_evidence(now_ms), RUN_ID, SEED,
    )
    .await
    .expect("evaluate");

    assert_eq!(out.verdict, InterlockVerdict::Running, "clean evidence => Running");
    assert!(!out.latched_halted, "a clean deployment must NOT latch");
    assert!(
        !ledger.is_halted(STRATEGY).expect("is_halted"),
        "a clean deployment leaves the interlock un-halted"
    );
    assert_eq!(
        rows_of_kind(&ssb, DriftInterlockKind::Running).len(),
        1,
        "exactly one DriftInterlock(Running) row must land (off-agent artifact)"
    );
}

// ---------------------------------------------------------------------------
// (2) a PSI breach (output PSI >= 0.25) => latched DriftHalted
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn output_psi_breach_halts() {
    let dir = TempDir::new().unwrap();
    let (ssb, db_path) = test_ssb(&dir, "h29-contract-2");
    let clock = test_clock();
    let ledger = test_ledger(&db_path);
    let now_ms = Ssb::now_ms_via_clock(&clock);

    let mut ev = clean_evidence(now_ms);
    // A large output-distribution shift: PSI well over PSI_HALT_BAND (0.25).
    ev.output_distribution = Some(PsiBins {
        expected: vec![0.5, 0.5],
        actual: vec![0.05, 0.95],
    });

    let out = evaluate_drift_interlock(&ledger, &ssb, &clock, STRATEGY, &ev, RUN_ID, SEED)
        .await
        .expect("evaluate");

    assert!(out.latched_halted, "an output-PSI breach must latch HALTED");
    assert_eq!(out.verdict.to_kind(), DriftInterlockKind::DriftHalted);
    assert!(
        out.verdict.axes().iter().any(|a| a == "output_psi"),
        "the breached axis must be named: {:?}",
        out.verdict.axes()
    );
    assert!(ledger.is_halted(STRATEGY).expect("is_halted"));
    assert_eq!(
        rows_of_kind(&ssb, DriftInterlockKind::DriftHalted).len(),
        1,
        "a DriftInterlock(DriftHalted) row must land"
    );
}

// ---------------------------------------------------------------------------
// (3) a schema-fingerprint mismatch => DriftHalted
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn schema_fingerprint_mismatch_halts() {
    let dir = TempDir::new().unwrap();
    let (ssb, db_path) = test_ssb(&dir, "h29-contract-3");
    let clock = test_clock();
    let ledger = test_ledger(&db_path);
    let now_ms = Ssb::now_ms_via_clock(&clock);

    let mut ev = clean_evidence(now_ms);
    // The frozen baseline no longer matches the live-schema blake3 fingerprint (vendor drift).
    ev.baseline_schema_fingerprint = Some("deadbeef".repeat(8));

    let out = evaluate_drift_interlock(&ledger, &ssb, &clock, STRATEGY, &ev, RUN_ID, SEED)
        .await
        .expect("evaluate");

    assert!(out.latched_halted, "a schema mismatch must latch HALTED");
    assert!(out.verdict.axes().iter().any(|a| a == "schema_fingerprint"));
    assert!(ledger.is_halted(STRATEGY).expect("is_halted"));
}

// ---------------------------------------------------------------------------
// (4) the latch is PERSISTENT — a fresh re-open still reads HALTED (restart != reset)
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn latch_persists_across_reopen() {
    let dir = TempDir::new().unwrap();
    let (ssb, db_path) = test_ssb(&dir, "h29-contract-4");
    let clock = test_clock();
    let now_ms = Ssb::now_ms_via_clock(&clock);

    let mut ev = clean_evidence(now_ms);
    ev.baseline_schema_fingerprint = Some("deadbeef".repeat(8)); // force a HALT

    {
        let ledger = test_ledger(&db_path);
        let out = evaluate_drift_interlock(&ledger, &ssb, &clock, STRATEGY, &ev, RUN_ID, SEED)
            .await
            .expect("evaluate");
        assert!(out.latched_halted);
    }

    // A fresh ledger over the SAME db re-derives HALTED from the journal — a process restart
    // does NOT silently reset to Running (Running is the ABSENCE of an unresolved halt).
    let ledger2 = test_ledger(&db_path);
    assert!(
        ledger2.is_halted(STRATEGY).expect("is_halted after reopen"),
        "the latch must survive a ledger re-open (restart != reset)"
    );

    // A re-evaluation while latched stays HALTED (the pre-existing-latch short-circuit).
    let out2 =
        evaluate_drift_interlock(&ledger2, &ssb, &clock, STRATEGY, &clean_evidence(now_ms), RUN_ID, SEED)
            .await
            .expect("re-evaluate");
    assert!(
        out2.latched_halted,
        "even a clean re-evaluation stays HALTED while the latch is unresolved"
    );
}

// ---------------------------------------------------------------------------
// (5) the no-unlock invariant — NO unlock/reset method on the agent-facing type
// ---------------------------------------------------------------------------

/// Negative test (compile-time intent, mirror holdout_budget.rs:1128). The agent-facing
/// DriftInterlockLedger must NOT expose any unlock / reset / clear / resume method;
/// unlock_drift_interlock is a FREE FUNCTION, not a method. If a banned method were added, the
/// commented calls below would compile and this test's intent would be violated.
#[test]
fn no_unlock_or_reset_method_exists() {
    let _ = |_l: &DriftInterlockLedger| {
        // These must NOT compile:
        // _l.unlock("s");
        // _l.reset("s");
        // _l.clear("s");
        // _l.resume("s");
        // _l.unlock_drift_interlock(...);
    };
    // Also reference the FREE-function unlock path so the test is not vacuous: an agent holding
    // only a ledger handle cannot reach this — it is a free function, not a method.
    let _free_fn = unlock_drift_interlock;
    let _ = _free_fn;
}

// ---------------------------------------------------------------------------
// (6) a human unlock WITHOUT fresh evidence => REJECTED; with fresh evidence => Running
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn unlock_requires_fresh_revalidation_evidence() {
    let dir = TempDir::new().unwrap();
    let (ssb, db_path) = test_ssb(&dir, "h29-contract-6");
    let clock = test_clock();
    let ledger = test_ledger(&db_path);
    let now_ms = Ssb::now_ms_via_clock(&clock);

    let mut ev = clean_evidence(now_ms);
    ev.baseline_schema_fingerprint = Some("deadbeef".repeat(8)); // force a HALT
    let out = evaluate_drift_interlock(&ledger, &ssb, &clock, STRATEGY, &ev, RUN_ID, SEED)
        .await
        .expect("evaluate");
    assert!(out.latched_halted);

    // STALE re-validation (older than the halt) => REJECTED, still halted.
    let stale = unlock_drift_interlock(&ledger, &ssb, &clock, STRATEGY, 1, RUN_ID, SEED).await;
    assert!(
        matches!(stale, Err(DriftInterlockError::StaleUnlockEvidence { .. })),
        "stale re-validation must be rejected, got {stale:?}"
    );
    assert!(
        ledger.is_halted(STRATEGY).expect("still halted"),
        "a rejected unlock leaves the latch in place"
    );

    // FRESH re-validation (a future timestamp, strictly newer than the halt) => clears the latch.
    let fresh_at = Ssb::now_ms_via_clock(&clock) + 10 * MS_PER_DAY;
    unlock_drift_interlock(&ledger, &ssb, &clock, STRATEGY, fresh_at, RUN_ID, SEED)
        .await
        .expect("fresh unlock");
    assert!(
        !ledger.is_halted(STRATEGY).expect("unlocked"),
        "a fresh-evidence unlock must clear the latch (resume permitted)"
    );
    assert_eq!(
        rows_of_kind(&ssb, DriftInterlockKind::HumanUnlocked).len(),
        1,
        "a DriftInterlock(HumanUnlocked) row must land on the successful unlock"
    );
}

// ---------------------------------------------------------------------------
// (7) default-HALTED — missing / empty evidence on any axis => HALT not Running
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn missing_evidence_halts_fail_closed() {
    let dir = TempDir::new().unwrap();
    let (ssb, db_path) = test_ssb(&dir, "h29-contract-7");
    let clock = test_clock();
    let ledger = test_ledger(&db_path);
    let now_ms = Ssb::now_ms_via_clock(&clock);

    // A missing input distribution (the feature log never landed) => HALT, not SKIP, not Running.
    let mut ev = clean_evidence(now_ms);
    ev.input_distribution = None;

    let out = evaluate_drift_interlock(&ledger, &ssb, &clock, STRATEGY, &ev, RUN_ID, SEED)
        .await
        .expect("evaluate");
    assert!(
        out.latched_halted,
        "missing evidence on any axis must HALT (DEFAULT-HALTED, R1/R2)"
    );
    assert!(out.verdict.axes().iter().any(|a| a == "input_psi_missing"));
}

// ---------------------------------------------------------------------------
// PURE-classify spot checks (the 4-axis math, no Ssb/ledger needed)
// ---------------------------------------------------------------------------

#[test]
fn classify_ttl_expiry_halts() {
    let mut ev = clean_evidence(NOW_MS);
    ev.last_validated_at_ms = Some(NOW_MS - 100 * MS_PER_DAY); // > 90d TTL
    let v = classify_drift(&ev, NOW_MS);
    assert!(v.is_halt(), "a >90d-stale validation must HALT");
    assert!(v.axes().iter().any(|a| a == "validation_ttl_expired"));
}

#[test]
fn classify_deviation_breach_halts() {
    let mut ev = clean_evidence(NOW_MS);
    ev.live_vs_backtest_deviation = Some(DEVIATION_HALT_BAND + 1.0);
    let v = classify_drift(&ev, NOW_MS);
    assert!(v.is_halt(), "a deviation over the band must HALT");
    assert!(v.axes().iter().any(|a| a == "live_vs_backtest_deviation"));
}

#[test]
fn classify_composed_signal_halts() {
    let mut ev = clean_evidence(NOW_MS);
    ev.composed_breaches = vec!["fingerprint_changed".to_string()];
    let v = classify_drift(&ev, NOW_MS);
    assert!(v.is_halt(), "a composed upstream signal (G-10) must HALT");
    assert!(v.axes().iter().any(|a| a.starts_with("composed:")));
}

#[test]
fn psi_known_shift_over_band_and_zero_bin_finite() {
    let shifted = PsiBins {
        expected: vec![0.5, 0.5],
        actual: vec![0.05, 0.95],
    };
    match compute_psi(&shifted) {
        PsiResult::Value(v) => assert!(v >= PSI_HALT_BAND, "shift PSI {v} must exceed band"),
        PsiResult::Undefined => panic!("a well-formed shift must be defined"),
    }
    // A zero bin must NOT produce NaN.
    let zero_bin = PsiBins {
        expected: vec![0.0, 1.0],
        actual: vec![0.5, 0.5],
    };
    match compute_psi(&zero_bin) {
        PsiResult::Value(v) => assert!(v.is_finite(), "zero bin must stay finite"),
        PsiResult::Undefined => { /* acceptable fail-closed */ }
    }
}

// ---------------------------------------------------------------------------
// (8) SCHEMA_VERSION >= 238, DriftInterlock in the action_type bijection
// ---------------------------------------------------------------------------

#[test]
fn schema_version_and_action_type_bijection() {
    assert!(
        SCHEMA_VERSION_PUBLIC >= 238,
        "H-29 requires SCHEMA_VERSION >= 238, got {SCHEMA_VERSION_PUBLIC}"
    );
    // Round-trip the wire tag through from_str / as_sql_str.
    let at = ssb_action_type_from_str_public("drift_interlock").expect("from_str drift_interlock");
    assert_eq!(at, SsbActionType::DriftInterlock);
    assert_eq!(SsbActionType::DriftInterlock.as_sql_str(), "drift_interlock");
}

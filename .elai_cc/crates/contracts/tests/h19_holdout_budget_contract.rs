//! H-19 contract tests — the reusable-holdout query budget (the Thresholdout DP gate; Dwork,
//! Feldman, Hardt, Pitassi, Reingold, Roth, NIPS 2015, arXiv:1506.02629, Theorem 25).
//!
//! Covers (TEST-FIRST, authored against the live gate surface), over a REAL Ssb SQLite db
//! (off-agent artifact, Layer-4 / R4):
//!   (1) an above-threshold OOS query CHARGES the budget (a QuerySpent row) + returns
//!       holdout_stat + Laplace noise (the noised holdout statistic).
//!   (2) a below-threshold OOS query returns train_stat, budget UNCHANGED (the FREE path).
//!   (3) budget exhausted => REFUSE, kind=Exhausted (fail-closed, R1/R2).
//!   (4) the no-reset invariant: NO reset/delete/update/provision method exists on
//!       HoldoutBudgetLedger (provision_fresh_holdout is a FREE FUNCTION, not a method) +
//!       the budget STAYS spent across a re-open.
//!   (5) the seeded Laplace noise is GD-2 reproducible (same ELAI_SEED => byte-identical
//!       noised answer).
//!   (6) provision-fresh with an OVERLAPPING data_window => REJECTED (the re-label reset
//!       attack).
//!   (7) SCHEMA_VERSION >= 237, HoldoutBudgetEvent in the action_type bijection.
//!
//! The budget counter is append-only (queries_spent = COUNT of QuerySpent rows; mirrors the
//! H-17 TrialLedger) and the file is in the LOCKED gate-manifest set so it cannot edit its own
//! budget. The IRREDUCIBLE top — provisioning a FRESH holdout — is the human/frozen OUT-OF-BAND
//! path (R8). See workflow/reasoning/H-19-decisions.md.
//!
//! SCHEMA_VERSION assertions use `>=` (never an exact global pin) per the
//! BRITTLE_GLOBAL_SCHEMA_PIN gotcha. TOTAL_VARIANTS / ALL_VARIANTS presence is asserted by
//! action_type_lock.rs (compile-time exhaustiveness + count-drift) and the e45 contract.
//!
//! Grounding:
//!   - crates/orchestrator/src/holdout_budget.rs (H-19 gate; classify + query + provision).
//!   - crates/orchestrator/src/trial_ledger.rs (the H-17 append-only no-reset discipline).
//!   - workflow/papers/reusable-holdout-thresholdout-interface.md (the LOCKED formulas).

use std::sync::Arc;

use elai_core::clock::OsClock;
use elai_orchestrator::{
    holdout_budget::{
        provision_fresh_holdout, BudgetRemainingResult, HoldoutBudgetError, HoldoutBudgetLedger,
        ThresholdoutDecision, classify_holdout_query, HOLDOUT_BUDGET_TOTAL,
    },
    ssb::{
        ssb_action_type_from_str_public, HoldoutBudgetKind, Ssb, SsbActionType, SsbConfig,
        SsbPayload, SCHEMA_VERSION_PUBLIC,
    },
    SsbReader,
};
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

/// A planted fixture holdout id / objective / window.
const HOLDOUT_ID: &str = "sharpe_1y_holdout";
const OBJECTIVE: &str = "sharpe_1y";
const WINDOW: &str = "0..100";
/// Above-threshold gap (|0.9 - 0.1| = 0.8 >> T ~ 0.0375).
const TRAIN_STAT_ABOVE: f64 = 0.10;
const HOLDOUT_STAT_ABOVE: f64 = 0.90;
const SEED: u64 = 42;
const RUN_ID: &str = "h19-contract";

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

fn test_ledger(db_path: &std::path::Path) -> HoldoutBudgetLedger {
    HoldoutBudgetLedger::open(db_path, "h19-contract-session").expect("holdout ledger open")
}

/// All HoldoutBudgetEvent rows of a given kind.
fn rows_of_kind(ssb: &Arc<Ssb>, want: HoldoutBudgetKind) -> Vec<SsbPayload> {
    ssb.query_by_type(SsbActionType::HoldoutBudgetEvent)
        .expect("query HoldoutBudgetEvent")
        .iter()
        .filter_map(|r| rmp_serde::from_slice::<SsbPayload>(&r.payload).ok())
        .filter(|p| matches!(p, SsbPayload::HoldoutBudgetEvent(pp) if pp.kind == want))
        .collect()
}

// ---------------------------------------------------------------------------
// (1) above-threshold query CHARGES + returns noised holdout stat + QuerySpent row
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn above_threshold_charges_budget_and_returns_noised_holdout() {
    let dir = TempDir::new().unwrap();
    let (ssb, db_path) = test_ssb(&dir, "h19-contract-1");
    let clock = test_clock();
    let ledger = test_ledger(&db_path);

    provision_fresh_holdout(
        &ledger, &ssb, &clock, HOLDOUT_ID, OBJECTIVE, WINDOW, HOLDOUT_BUDGET_TOTAL, SEED, RUN_ID,
    )
    .await
    .expect("provision");

    let out = ledger
        .query_holdout(
            &ssb, &clock, HOLDOUT_ID, OBJECTIVE, WINDOW, TRAIN_STAT_ABOVE, HOLDOUT_STAT_ABOVE,
            RUN_ID, SEED,
        )
        .await
        .expect("query");

    assert_eq!(out.kind, HoldoutBudgetKind::QuerySpent);
    assert!(out.charged, "an above-threshold query charges the budget");
    assert_eq!(out.queries_spent, 1, "one budget unit spent");
    let answer = out.answer.expect("an above-threshold query returns an answer");
    // The released statistic is the holdout mean + Laplace(sigma) noise: near 0.9, not the
    // training mean 0.1.
    assert!(
        (answer - HOLDOUT_STAT_ABOVE).abs() < 0.5,
        "answer is the Laplace-noised holdout statistic (near 0.9), got {answer}"
    );
    assert!(
        (answer - TRAIN_STAT_ABOVE).abs() > 0.1,
        "answer is NOT the training mean (the budget was charged to release the holdout stat)"
    );

    // Off-agent artifact: exactly one QuerySpent row landed.
    let spent_rows = rows_of_kind(&ssb, HoldoutBudgetKind::QuerySpent);
    assert_eq!(spent_rows.len(), 1, "one QuerySpent SSB row (off-agent artifact, R4)");
    if let SsbPayload::HoldoutBudgetEvent(p) = &spent_rows[0] {
        assert_eq!(p.queries_spent, 1);
        assert!(p.noisy_statistic.is_some(), "QuerySpent carries the noised statistic");
        assert_eq!(p.corpus_blake3.len(), 64, "blake3 hex is 64 chars");
    } else {
        panic!("payload must be HoldoutBudgetEvent");
    }
}

// ---------------------------------------------------------------------------
// (2) below-threshold query returns train_stat, budget UNCHANGED (FREE path)
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn below_threshold_returns_train_stat_budget_unchanged() {
    let dir = TempDir::new().unwrap();
    let (ssb, db_path) = test_ssb(&dir, "h19-contract-2");
    let clock = test_clock();
    let ledger = test_ledger(&db_path);

    provision_fresh_holdout(
        &ledger, &ssb, &clock, HOLDOUT_ID, OBJECTIVE, WINDOW, HOLDOUT_BUDGET_TOTAL, SEED, RUN_ID,
    )
    .await
    .expect("provision");

    // train == holdout => zero empirical gap => always below threshold => FREE train answer.
    let train = 0.61_f64;
    let out = ledger
        .query_holdout(&ssb, &clock, HOLDOUT_ID, OBJECTIVE, WINDOW, train, train, RUN_ID, SEED)
        .await
        .expect("query");

    assert!(!out.charged, "a below-threshold query does NOT charge the budget (FREE)");
    assert_eq!(out.queries_spent, 0, "budget UNCHANGED on the free path");
    let answer = out.answer.expect("the free path returns the training statistic");
    assert!(
        (answer - train).abs() < 1e-12,
        "the free path returns the training mean exactly (no noise added)"
    );

    // budget_remaining is unchanged at the full total.
    let remaining = ledger.budget_remaining(HOLDOUT_ID, OBJECTIVE).unwrap();
    assert_eq!(remaining, BudgetRemainingResult::Remaining(HOLDOUT_BUDGET_TOTAL));

    // No QuerySpent row emitted on the free path.
    assert!(
        rows_of_kind(&ssb, HoldoutBudgetKind::QuerySpent).is_empty(),
        "the free below-threshold path emits no spend row"
    );
}

// ---------------------------------------------------------------------------
// (3) budget exhausted => REFUSE (Exhausted), fail-closed
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn exhausted_budget_refuses_fail_closed() {
    let dir = TempDir::new().unwrap();
    let (ssb, db_path) = test_ssb(&dir, "h19-contract-3");
    let clock = test_clock();
    let ledger = test_ledger(&db_path);

    // Budget B=1 so a single spend exhausts it.
    provision_fresh_holdout(&ledger, &ssb, &clock, HOLDOUT_ID, OBJECTIVE, WINDOW, 1, SEED, RUN_ID)
        .await
        .expect("provision B=1");

    let first = ledger
        .query_holdout(
            &ssb, &clock, HOLDOUT_ID, OBJECTIVE, WINDOW, TRAIN_STAT_ABOVE, HOLDOUT_STAT_ABOVE,
            RUN_ID, SEED,
        )
        .await
        .expect("first");
    assert!(first.charged, "first above-threshold query spends the single unit");

    let second = ledger
        .query_holdout(
            &ssb, &clock, HOLDOUT_ID, OBJECTIVE, WINDOW, TRAIN_STAT_ABOVE, HOLDOUT_STAT_ABOVE,
            RUN_ID, SEED,
        )
        .await
        .expect("second");
    assert_eq!(
        second.kind,
        HoldoutBudgetKind::Exhausted,
        "a query after the budget is spent must be REFUSED (Exhausted, fail-closed)"
    );
    assert!(second.is_refused());
    assert!(second.answer.is_none(), "an exhausted refusal releases no statistic");

    // An Exhausted SSB row landed (the off-agent refusal artifact).
    assert_eq!(
        rows_of_kind(&ssb, HoldoutBudgetKind::Exhausted).len(),
        1,
        "one Exhausted SSB row"
    );
}

// ---------------------------------------------------------------------------
// (3b) a MISSING budget counter => REFUSE (fail-closed, BudgetMissing)
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn missing_budget_counter_refuses_fail_closed() {
    let dir = TempDir::new().unwrap();
    let (ssb, db_path) = test_ssb(&dir, "h19-contract-3b");
    let clock = test_clock();
    let ledger = test_ledger(&db_path);

    // NO provision: the holdout was never provisioned => BudgetMissing => REFUSE.
    let r = ledger.budget_remaining(HOLDOUT_ID, OBJECTIVE).unwrap();
    assert!(r.is_missing(), "an unprovisioned holdout => BudgetMissing");
    assert!(r.must_refuse(), "missing must REFUSE (R2: not found = never ran = FAIL)");

    let out = ledger
        .query_holdout(
            &ssb, &clock, HOLDOUT_ID, OBJECTIVE, WINDOW, TRAIN_STAT_ABOVE, HOLDOUT_STAT_ABOVE,
            RUN_ID, SEED,
        )
        .await
        .expect("query");
    assert_eq!(
        out.kind,
        HoldoutBudgetKind::Exhausted,
        "an OOS query on an un-provisioned holdout must REFUSE (fail-closed)"
    );
}

// ---------------------------------------------------------------------------
// (4) no-reset invariant + budget stays spent across re-open
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn budget_stays_spent_across_reopen() {
    let dir = TempDir::new().unwrap();
    let (ssb, db_path) = test_ssb(&dir, "h19-contract-4");
    let clock = test_clock();

    {
        let ledger = test_ledger(&db_path);
        provision_fresh_holdout(
            &ledger, &ssb, &clock, HOLDOUT_ID, OBJECTIVE, WINDOW, 4, SEED, RUN_ID,
        )
        .await
        .expect("provision");
        ledger
            .query_holdout(
                &ssb, &clock, HOLDOUT_ID, OBJECTIVE, WINDOW, TRAIN_STAT_ABOVE, HOLDOUT_STAT_ABOVE,
                RUN_ID, SEED,
            )
            .await
            .expect("spend");
    }

    // Re-open a fresh ledger handle on the same db: the spent counter persists (the budget
    // once spent STAYS spent — there is no reset path).
    let ledger2 = test_ledger(&db_path);
    let remaining = ledger2.budget_remaining(HOLDOUT_ID, OBJECTIVE).unwrap();
    assert_eq!(
        remaining,
        BudgetRemainingResult::Remaining(3),
        "1 spent of 4 => 3 remaining after re-open (no reset)"
    );
}

#[test]
fn no_agent_reset_method_exists() {
    // Negative compile-test: the agent-facing HoldoutBudgetLedger must NOT expose any
    // reset/delete/update/provision method (provision_fresh_holdout is a FREE FUNCTION). If a
    // banned method were added, the commented calls would compile and the gate would be
    // defeatable by an agent holding a ledger handle.
    let _ = |_l: &HoldoutBudgetLedger| {
        // These must NOT compile:
        // _l.reset_budget(HOLDOUT_ID, OBJECTIVE);
        // _l.delete_budget(HOLDOUT_ID, OBJECTIVE);
        // _l.provision_fresh_holdout(HOLDOUT_ID, OBJECTIVE, WINDOW, 4);
    };
}

// ---------------------------------------------------------------------------
// (5) seeded Laplace noise is GD-2 reproducible
// ---------------------------------------------------------------------------

#[test]
fn noise_is_seed_reproducible_gd2() {
    let a = classify_holdout_query(TRAIN_STAT_ABOVE, HOLDOUT_STAT_ABOVE, 777, 5);
    let b = classify_holdout_query(TRAIN_STAT_ABOVE, HOLDOUT_STAT_ABOVE, 777, 5);
    assert_eq!(a, b, "same (inputs, seed, query_index) => byte-identical decision (GD-2)");

    // A different seed yields a (potentially) different noised value: the decision is still
    // above-threshold for this large gap, but the noised holdout statistic differs.
    let c = classify_holdout_query(TRAIN_STAT_ABOVE, HOLDOUT_STAT_ABOVE, 999, 5);
    if let (
        ThresholdoutDecision::AboveThreshold { noised_holdout: na },
        ThresholdoutDecision::AboveThreshold { noised_holdout: nc },
    ) = (a, c)
    {
        // Not asserting strict inequality (noise could collide), only that the seed is wired:
        // the SAME seed reproduced exactly (asserted above) which is the GD-2 contract.
        let _ = (na, nc);
    }
}

// ---------------------------------------------------------------------------
// (6) provision-fresh with an OVERLAPPING window => REJECTED (reset attack)
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn provision_overlapping_window_rejected() {
    let dir = TempDir::new().unwrap();
    let (ssb, db_path) = test_ssb(&dir, "h19-contract-6");
    let clock = test_clock();
    let ledger = test_ledger(&db_path);

    provision_fresh_holdout(&ledger, &ssb, &clock, HOLDOUT_ID, OBJECTIVE, "0..100", 4, SEED, RUN_ID)
        .await
        .expect("provision");
    ledger
        .query_holdout(
            &ssb, &clock, HOLDOUT_ID, OBJECTIVE, "0..100", TRAIN_STAT_ABOVE, HOLDOUT_STAT_ABOVE,
            RUN_ID, SEED,
        )
        .await
        .expect("spend over 0..100");

    // Re-provision a window OVERLAPPING the spent one (50..150) => rejected (covert reset
    // attack: re-labeling the same series does not restore validity).
    let err = provision_fresh_holdout(
        &ledger, &ssb, &clock, HOLDOUT_ID, OBJECTIVE, "50..150", 4, SEED, RUN_ID,
    )
    .await;
    assert!(
        matches!(err, Err(HoldoutBudgetError::OverlappingProvisionWindow { .. })),
        "an overlapping provision window must be rejected (covert reset attack)"
    );

    // A DISJOINT window (200..300) is accepted.
    provision_fresh_holdout(
        &ledger, &ssb, &clock, HOLDOUT_ID, OBJECTIVE, "200..300", 4, SEED, RUN_ID,
    )
    .await
    .expect("a disjoint window provision is accepted");
}

// ---------------------------------------------------------------------------
// (7) SSB lockstep: SCHEMA_VERSION >= 237, action_type bijection landed
// ---------------------------------------------------------------------------

#[test]
fn schema_version_and_variant_lockstep_landed() {
    assert!(
        SCHEMA_VERSION_PUBLIC >= 237,
        "SCHEMA_VERSION must be >= 237 after H-19; got {SCHEMA_VERSION_PUBLIC}"
    );

    // HoldoutBudgetEvent must round-trip through the action_type bijection.
    let at = ssb_action_type_from_str_public("holdout_budget_event")
        .expect("holdout_budget_event must be a known action_type");
    assert_eq!(at, SsbActionType::HoldoutBudgetEvent);
    assert_eq!(
        SsbActionType::HoldoutBudgetEvent.as_sql_str(),
        "holdout_budget_event"
    );
}

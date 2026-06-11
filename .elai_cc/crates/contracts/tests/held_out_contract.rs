//! D-6 closure — held-out task store contract tests.
//!
//! Closes the audit gaps surfaced by
//! `workflow/sessions/s-25120/cross-check-recheck/D-6_held_out_task_store-report.md`:
//!
//!  1. SCHEMA floor: `SCHEMA_VERSION_PUBLIC >= 112` (D-6 closure bumped from 111).
//!  2. `HeldOutPromotionGate` payload round-trip via rmp + action_type tag.
//!  3. `HeldOutRefresh` payload round-trip via rmp + action_type tag.
//!  4. `TaskSource` enum exhaustive serde: all 5 variants survive an rmp
//!     round-trip (GitHubIssue, BenchmarkFixture, Synthetic, ArxivRepro,
//!     HumanCurated).
//!  5. `HeldOutStatus` enum exhaustive serde: all 5 variants round-trip.
//!  6. `RetireReason` enum exhaustive serde: all 3 reason variants
//!     round-trip (including the nested `ContaminationSource` payload).
//!  7. `AutoRefreshPolicy` enum exhaustive serde: all 4 variants round-trip.
//!  8. `ModelTrainingCutoff` invariant: `is_task_after_cutoff` returns
//!     true iff the task's release-time-proxy is strictly greater than
//!     the cutoff.
//!
//! Grounding:
//!   - workflow/papers/livecodebench-2403.07974-interface.md
//!     (temporal date-windowing, §S3.SS1 + §S6 — paper's only
//!     contamination mechanism)
//!   - workflow/papers/soft-contamination-2602.12413-interface.md
//!     (cosine + LLM REPLACEMENT for n-gram, not an extension)
//!   - workflow/papers/arc-prize-2025-2601.10904-interface.md
//!     (contamination warnings + held-out methodology — VERIFIED)
//!   - workflow/reasoning/D-6-decisions.md (DECISION-1..8)

use elai_orchestrator::held_out::{
    is_task_after_cutoff, AutoRefreshPolicy, ContaminationSource, EvalHistorySnapshot,
    ExpectedOutcome, HeldOutStatus, HeldOutTask, ModelTrainingCutoff, PromotionGateResult,
    RetireReason, TaskId, TaskSource, DEFAULT_NGRAM_SIZE,
};
use elai_orchestrator::ssb::{
    ssb_action_type_from_str_public, HeldOutPromotionGatePayload, HeldOutRefreshPayload,
    SsbActionType, SsbPayload, SCHEMA_VERSION_PUBLIC,
};

// ---------------------------------------------------------------------------
// 1. SCHEMA floor
// ---------------------------------------------------------------------------

#[test]
fn schema_version_at_or_above_d6_closure_floor() {
    // D-6 closure bumped SCHEMA_VERSION 111 -> 112 to add
    // HeldOutPromotionGate + HeldOutRefresh. A future bump can land
    // safely; a regression below 112 means one of the two variants
    // ships without its SQL CHECK allowlist entry, which surfaces as a
    // write-time CHECK constraint violation. This guard catches that on
    // every contract test run.
    assert!(
        SCHEMA_VERSION_PUBLIC >= 112,
        "SCHEMA_VERSION_PUBLIC dropped below the D-6 closure floor (112); \
         got {SCHEMA_VERSION_PUBLIC}. Verify HeldOutPromotionGate + \
         HeldOutRefresh are still in the SQL CHECK allowlist + enum + \
         action_type_lock.golden."
    );
}

// ---------------------------------------------------------------------------
// 2. HeldOutPromotionGate wire-protocol round-trip
// ---------------------------------------------------------------------------

#[test]
fn held_out_promotion_gate_payload_round_trips_through_rmp() {
    let original = HeldOutPromotionGatePayload {
        task_id: "11112222333344445555666677778888".into(),
        allow: true,
        delta: -0.0234,
        regression_threshold: 0.05,
        passes: 47,
        fails: 3,
        reason: "delta=-0.0234 pp (allowed: >= -0.0500 pp regression threshold)".into(),
    };
    let payload = SsbPayload::HeldOutPromotionGate(original.clone());
    let bytes = rmp_serde::to_vec(&payload).expect("rmp encode");
    let decoded: SsbPayload = rmp_serde::from_slice(&bytes).expect("rmp decode");
    match decoded {
        SsbPayload::HeldOutPromotionGate(rt) => {
            assert_eq!(rt.task_id, original.task_id);
            assert_eq!(rt.allow, original.allow);
            assert!((rt.delta - original.delta).abs() < f64::EPSILON);
            assert!(
                (rt.regression_threshold - original.regression_threshold).abs() < f64::EPSILON
            );
            assert_eq!(rt.passes, original.passes);
            assert_eq!(rt.fails, original.fails);
            assert_eq!(rt.reason, original.reason);
        }
        other => panic!("expected SsbPayload::HeldOutPromotionGate, got {other:?}"),
    }
    assert_eq!(payload.action_type(), SsbActionType::HeldOutPromotionGate);
    assert_eq!(
        SsbActionType::HeldOutPromotionGate.as_sql_str(),
        "held_out_promotion_gate"
    );
    assert!(matches!(
        ssb_action_type_from_str_public("held_out_promotion_gate"),
        Ok(SsbActionType::HeldOutPromotionGate)
    ));
}

// ---------------------------------------------------------------------------
// 3. HeldOutRefresh wire-protocol round-trip
// ---------------------------------------------------------------------------

#[test]
fn held_out_refresh_payload_round_trips_through_rmp() {
    let original = HeldOutRefreshPayload {
        source_label: "arxiv_repro".into(),
        added_count: 5,
        retired_count: 2,
        ts_ms: 1_700_000_000_500,
    };
    let payload = SsbPayload::HeldOutRefresh(original.clone());
    let bytes = rmp_serde::to_vec(&payload).expect("rmp encode");
    let decoded: SsbPayload = rmp_serde::from_slice(&bytes).expect("rmp decode");
    match decoded {
        SsbPayload::HeldOutRefresh(rt) => {
            assert_eq!(rt.source_label, original.source_label);
            assert_eq!(rt.added_count, original.added_count);
            assert_eq!(rt.retired_count, original.retired_count);
            assert_eq!(rt.ts_ms, original.ts_ms);
        }
        other => panic!("expected SsbPayload::HeldOutRefresh, got {other:?}"),
    }
    assert_eq!(payload.action_type(), SsbActionType::HeldOutRefresh);
    assert_eq!(
        SsbActionType::HeldOutRefresh.as_sql_str(),
        "held_out_refresh"
    );
    assert!(matches!(
        ssb_action_type_from_str_public("held_out_refresh"),
        Ok(SsbActionType::HeldOutRefresh)
    ));
}

// ---------------------------------------------------------------------------
// 4. TaskSource exhaustive serde
// ---------------------------------------------------------------------------

#[test]
fn task_source_all_five_variants_round_trip_through_json() {
    let cases = vec![
        TaskSource::GitHubIssue {
            repo: "owner/name".into(),
            issue_id: 42,
        },
        TaskSource::BenchmarkFixture {
            name: "livecodebench-2024-q1".into(),
        },
        TaskSource::Synthetic { seed: 12345 },
        TaskSource::ArxivRepro {
            arxiv_id: "2403.07974".into(),
            fetched_at_ms: 1_700_000_000_000,
        },
        TaskSource::HumanCurated {
            curator_id: "maintainer".into(),
            curated_at_ms: 1_700_000_000_000,
        },
    ];
    for tc in cases {
        let s = serde_json::to_string(&tc).expect("serialize");
        let rt: TaskSource = serde_json::from_str(&s).expect("deserialize");
        assert_eq!(rt, tc, "round-trip drift for {tc:?}");
    }
}

// ---------------------------------------------------------------------------
// 5. HeldOutStatus exhaustive serde
// ---------------------------------------------------------------------------

#[test]
fn held_out_status_all_five_variants_round_trip_through_json() {
    let cases = vec![
        HeldOutStatus::Active,
        HeldOutStatus::RecentlyAdded,
        HeldOutStatus::RetiredUniformlyPassing,
        HeldOutStatus::RetiredContaminated,
        HeldOutStatus::Quarantined,
    ];
    for tc in cases {
        let s = serde_json::to_string(&tc).expect("serialize");
        let rt: HeldOutStatus = serde_json::from_str(&s).expect("deserialize");
        assert_eq!(rt, tc, "round-trip drift for {tc:?}");
    }
}

// ---------------------------------------------------------------------------
// 6. RetireReason exhaustive serde (incl. nested ContaminationSource)
// ---------------------------------------------------------------------------

#[test]
fn retire_reason_all_variants_round_trip_through_json() {
    let cases = vec![
        RetireReason::UniformlyPassing {
            passes_in_window: 100,
            window_months: 6,
        },
        RetireReason::ContaminationDetected {
            source: ContaminationSource::NgramJaccard {
                jaccard: 0.42,
                threshold: 0.30,
            },
        },
        RetireReason::ContaminationDetected {
            source: ContaminationSource::SoftContamination {
                cosine: 0.92,
                threshold: 0.85,
            },
        },
        RetireReason::ContaminationDetected {
            source: ContaminationSource::OperatorEvidence {
                note: "paper-ingestor flagged training corpus overlap".into(),
            },
        },
        RetireReason::OperatorRetired {
            note: "manual quarantine pending re-curation".into(),
        },
    ];
    for tc in cases {
        let s = serde_json::to_string(&tc).expect("serialize");
        let rt: RetireReason = serde_json::from_str(&s).expect("deserialize");
        assert_eq!(rt, tc, "round-trip drift for {tc:?}");
    }
}

// ---------------------------------------------------------------------------
// 7. AutoRefreshPolicy exhaustive serde
// ---------------------------------------------------------------------------

#[test]
fn auto_refresh_policy_all_four_variants_round_trip_through_json() {
    let cases = vec![
        AutoRefreshPolicy::Disabled,
        AutoRefreshPolicy::OnSchedule { interval_days: 30 },
        AutoRefreshPolicy::OnContamination,
        AutoRefreshPolicy::OnAccuracyDrift { threshold: 0.05 },
    ];
    for tc in cases {
        let s = serde_json::to_string(&tc).expect("serialize");
        let rt: AutoRefreshPolicy = serde_json::from_str(&s).expect("deserialize");
        assert_eq!(rt, tc, "round-trip drift for {tc:?}");
    }
}

// ---------------------------------------------------------------------------
// 8. ModelTrainingCutoff temporal predicate invariant
//     (LiveCodeBench §S3.SS1 grounding)
// ---------------------------------------------------------------------------

#[test]
fn is_task_after_cutoff_strict_inequality() {
    let cutoff = ModelTrainingCutoff {
        model_id: "claude-sonnet-4.6".into(),
        date_ms: 1_700_000_000_000,
    };
    // Task ingested AFTER cutoff -> true (eligible).
    let task_after = HeldOutTask {
        task_id: TaskId::new_v4(),
        source: TaskSource::Synthetic { seed: 1 },
        content_blake3: "a".repeat(64),
        expected_outcome: ExpectedOutcome::Code("ok".into()),
        ingested_ts_ms: 1_700_000_000_001,
        eligibility_window_ms: 0,
        ngram_signature: Vec::new(),
        body: "alpha beta gamma delta epsilon zeta eta theta iota kappa".into(),
        template_id: None,
    };
    assert!(is_task_after_cutoff(&task_after, &cutoff));

    // Task ingested AT cutoff -> false (strict inequality).
    let task_at = HeldOutTask {
        task_id: TaskId::new_v4(),
        source: TaskSource::Synthetic { seed: 2 },
        content_blake3: "b".repeat(64),
        expected_outcome: ExpectedOutcome::Code("ok".into()),
        ingested_ts_ms: 1_700_000_000_000,
        eligibility_window_ms: 0,
        ngram_signature: Vec::new(),
        body: "alpha beta gamma delta epsilon zeta eta theta iota kappa".into(),
        template_id: None,
    };
    assert!(!is_task_after_cutoff(&task_at, &cutoff));

    // ArxivRepro with fetched_at_ms takes precedence over ingest.
    let task_arxiv_after = HeldOutTask {
        task_id: TaskId::new_v4(),
        source: TaskSource::ArxivRepro {
            arxiv_id: "2403.07974".into(),
            fetched_at_ms: 1_700_000_000_001,
        },
        content_blake3: "c".repeat(64),
        expected_outcome: ExpectedOutcome::Code("ok".into()),
        // Ingest BEFORE cutoff to prove fetched_at_ms wins.
        ingested_ts_ms: 1_500_000_000_000,
        eligibility_window_ms: 0,
        ngram_signature: Vec::new(),
        body: "alpha beta gamma delta epsilon zeta eta theta iota kappa".into(),
        template_id: None,
    };
    assert!(is_task_after_cutoff(&task_arxiv_after, &cutoff));
}

// ---------------------------------------------------------------------------
// 9. PromotionGateResult layered behaviour
// ---------------------------------------------------------------------------

#[test]
fn promotion_gate_result_layered_through_eval_history() {
    // High eval_count, perfect pass -> 1.0 rate; mid pass rate -> 0.5.
    let snap_pass = EvalHistorySnapshot {
        eval_count: 50,
        pass_count: 50,
    };
    let snap_half = EvalHistorySnapshot {
        eval_count: 50,
        pass_count: 25,
    };
    assert!((snap_pass.pass_rate() - 1.0).abs() < 1e-6);
    assert!((snap_half.pass_rate() - 0.5).abs() < 1e-6);

    // Defensive divide-by-zero behaviour.
    let snap_empty = EvalHistorySnapshot {
        eval_count: 0,
        pass_count: 0,
    };
    assert_eq!(snap_empty.pass_rate(), 0.0);

    // PromotionGateResult must round-trip through json (consumed by the
    // BAI promotion path via SSB row).
    let r = PromotionGateResult {
        allow: true,
        reason: "ok".into(),
        delta: -0.01,
    };
    let s = serde_json::to_string(&r).expect("serialize");
    let rt: PromotionGateResult = serde_json::from_str(&s).expect("deserialize");
    assert_eq!(rt.allow, r.allow);
    assert_eq!(rt.reason, r.reason);
    assert!((rt.delta - r.delta).abs() < 1e-6);
}

// ---------------------------------------------------------------------------
// 10. DEFAULT_NGRAM_SIZE re-citation guard
// ---------------------------------------------------------------------------

#[test]
fn default_ngram_size_matches_elai_empirical_value() {
    // The const is ELAI-empirical (n-gram method itself is Shi et al.
    // 2023 / Riddell et al. 2024 — LiveCodeBench §S6.SS3 cites them as
    // prior work, paper does not publish defaults). Document the
    // grounding in the test message so a future reviewer can re-cite
    // without re-running paper-ingestor.
    assert_eq!(
        DEFAULT_NGRAM_SIZE, 9,
        "DEFAULT_NGRAM_SIZE moved off the ELAI-empirical 9 — verify the \
         [held_out].ngram_n override in elai_cc.toml is still 9 too, \
         and update DECISION-4 in workflow/reasoning/D-6-decisions.md."
    );
}

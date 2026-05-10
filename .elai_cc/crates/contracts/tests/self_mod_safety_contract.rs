//! Contract tests for B-23 Phase 4H Group D — self_mod_safety.rs (4H.6).

use elai_orchestrator::{
    apply_structural_change, check_lock_list_read, check_locked_key, is_locked,
    should_auto_revert, ConfigVariant, FullPipelineOutcome, PipelineMode, SafetyError,
    StructuralChange, StructuralChangeType, TestOutcomeReport, VariantStatus,
    DEFAULT_AUTO_REVERT_FRACTION, DEFAULT_AUTO_REVERT_WINDOW, DEFAULT_SCORE_WEIGHTS, LOCKED_KEYS,
};

#[test]
fn locked_keys_are_at_least_eleven_per_spec() {
    // Spec phase-4H.txt:201-204 lists 11 locked keys. Test guards against
    // accidental shrinkage.
    assert!(LOCKED_KEYS.len() >= 11, "got {}", LOCKED_KEYS.len());
    for canonical in [
        "score_weights",
        "gate_definitions",
        "auditor_config",
        "benchmark_harness",
        "test_suite",
        "config_variants_schema",
        "safety_locks",
        "lock_list",
        "cloud_monthly_budget",
        "cloud_opus_allowed",
        "cloud_escalation_threshold",
    ] {
        assert!(is_locked(canonical), "{canonical} must be locked");
    }
}

#[test]
fn check_locked_key_blocks_locked_and_allows_safe() {
    assert!(matches!(
        check_locked_key("cloud_monthly_budget"),
        Err(SafetyError::LockedKey(_))
    ));
    check_locked_key("etc_threshold").expect("non-locked passes");
}

#[test]
fn lock_list_read_always_denied() {
    let err = check_lock_list_read().unwrap_err();
    match err {
        SafetyError::LockedKey(k) => assert_eq!(k, "lock_list"),
        other => panic!("unexpected variant: {other:?}"),
    }
}

#[test]
fn structural_change_without_human_approved_rejected() {
    let err = apply_structural_change(&StructuralChange {
        change_type: StructuralChangeType::AllowedConfigKeyAddition,
        description: "add new param".into(),
        human_approved: false,
    })
    .unwrap_err();
    assert!(matches!(err, SafetyError::NotHumanApproved { .. }));
}

#[test]
fn structural_change_with_human_approved_passes() {
    apply_structural_change(&StructuralChange {
        change_type: StructuralChangeType::AllowedConfigKeyAddition,
        description: "add new param".into(),
        human_approved: true,
    })
    .expect("approved passes");
}

#[test]
fn allowed_config_key_addition_requires_layer3_approval() {
    // Spec phase-4H.txt:677 — "Adding new entries to AllowedConfigKey is
    // itself a Layer 3 change." This is enforced via the same gate.
    let unapproved = StructuralChange {
        change_type: StructuralChangeType::AllowedConfigKeyAddition,
        description: "add EtcThreshold".into(),
        human_approved: false,
    };
    assert!(matches!(
        apply_structural_change(&unapproved),
        Err(SafetyError::NotHumanApproved { .. })
    ));
}

fn outcome(coverage: f32, first_try: usize, total: usize, interventions: usize, blocks: usize) -> FullPipelineOutcome {
    FullPipelineOutcome {
        template_id: "t".into(),
        project_id: "p".into(),
        pipeline_mode: PipelineMode::Surgical,
        phases_total: total,
        phases_passed_first_try: first_try,
        audit_findings_total: 0,
        audit_findings_block: blocks,
        test_coverage_final: coverage,
        time_to_complete_seconds: 0,
        tokens_consumed_total: 0,
        human_interventions: interventions,
        rules_triggered: vec![],
        test_outcomes: TestOutcomeReport::default(),
        etc_trigger_events: 0,
        stale_api_catches: vec![],
        context_budget_truncations: vec![],
        failure_taxonomy_misses: vec![],
    }
}

fn baseline_variant(score: f32) -> ConfigVariant {
    ConfigVariant {
        variant_id: "active".into(),
        parent_id: None,
        score,
        children_count: 0,
        config_blob: vec![],
        created_at: 0,
        pipeline_outcomes: vec![],
        pipeline_mode: PipelineMode::Surgical,
        status: VariantStatus::Active,
    }
}

#[test]
fn auto_revert_fires_on_three_bad_pipelines() {
    let history: Vec<FullPipelineOutcome> = (0..3).map(|_| outcome(0.0, 1, 10, 1, 5)).collect();
    let revert = should_auto_revert(
        &history,
        &baseline_variant(0.8),
        &DEFAULT_SCORE_WEIGHTS,
        10_000,
        1,
        DEFAULT_AUTO_REVERT_FRACTION,
        DEFAULT_AUTO_REVERT_WINDOW,
    );
    assert!(revert);
}

#[test]
fn auto_revert_skips_when_one_recovers() {
    let mut history: Vec<FullPipelineOutcome> = (0..2).map(|_| outcome(0.0, 1, 10, 1, 5)).collect();
    history.push(outcome(0.95, 10, 10, 0, 0));
    let revert = should_auto_revert(
        &history,
        &baseline_variant(0.8),
        &DEFAULT_SCORE_WEIGHTS,
        10_000,
        5,
        DEFAULT_AUTO_REVERT_FRACTION,
        DEFAULT_AUTO_REVERT_WINDOW,
    );
    assert!(!revert);
}

#[test]
fn auto_revert_window_zero_returns_false() {
    let history: Vec<FullPipelineOutcome> = (0..3).map(|_| outcome(0.0, 1, 10, 1, 5)).collect();
    let revert = should_auto_revert(
        &history,
        &baseline_variant(0.8),
        &DEFAULT_SCORE_WEIGHTS,
        10_000,
        1,
        0.10,
        0,
    );
    assert!(!revert);
}

#[test]
fn auto_revert_thresholds_match_spec() {
    // Spec phase-4H.txt:725 — ">10% over 3 pipelines".
    assert!((DEFAULT_AUTO_REVERT_FRACTION - 0.10).abs() < 1e-6);
    assert_eq!(DEFAULT_AUTO_REVERT_WINDOW, 3);
}

#[test]
fn objective_hacking_canary_via_locked_metrics_code() {
    // The spec calls out node 114 of the DGM run which removed hallucination
    // markers to fake a perfect score. Our defence is "metrics code is
    // locked" — `auditor_config` and `gate_definitions` are in LOCKED_KEYS.
    // The trigger task itself is a runtime fixture (Group F/G); here we
    // verify the static defence is in place.
    assert!(is_locked("auditor_config"));
    assert!(is_locked("gate_definitions"));
    assert!(is_locked("benchmark_harness"));
    assert!(is_locked("test_suite"));
}

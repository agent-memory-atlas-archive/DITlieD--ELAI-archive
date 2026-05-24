//! D-18 closure — typed-ObservableEffect 4-variant pack integration.
//!
//! Exercises the closure-phase surface:
//!   * `EffectSet::validate_orthogonality` rejects duplicate
//!     (trigger, observation) pairings (Eureka §3.3 component-variation
//!     guard surrogate).
//!   * `check_goodhart_risk` returns one `GoodhartFlag` per offending
//!     effect.
//!   * `ObservationContext::Production` without an `instrument_file`
//!     forces `requires_human_review == true` (Coscientist §4
//!     invariant).
//!   * SandboxTier=Hermetic-class round-trip via
//!     `ObservationResult` carries the chosen tier through serde.
//!   * `probe_with_effects` default-impl projects a `FeasibilityReport`
//!     into a `Vec<ObservationResult>` without modifying `probe`'s
//!     signature (C-22 dyn-safety preserved).
//!   * `ELAI_OBSERVABLE_EFFECT_ENABLE` env var resolution stays no-op
//!     when unset.
//!
//! No live executor / network / filesystem dependency — every effect
//! runs against in-memory fixtures.
//!
//! Producers covered:
//!   - `elai_orchestrator::observable_effect::check_goodhart_risk`
//!   - `elai_orchestrator::observable_effect::EffectSet::validate_orthogonality`
//!   - `elai_orchestrator::observable_effect::ObservationResult::requires_human_review`
//!   - `elai_orchestrator::feasibility_probe::FeasibilityProber::probe_with_effects`
//!
//! Grounding (per workflow/reasoning/D-18-decisions.md):
//!   - Voyager arxiv:2305.16291 §3 (skill-with-measurable-effect)
//!   - Eureka arxiv:2310.12931 §3.3 (component-variation Goodhart guard)
//!   - Coscientist arxiv:2304.05332 §4 (sandbox vs production observation)
//!   - Plan's arxiv:2412.16367 cite is REJECTED (battery-thermal paper).

use std::path::PathBuf;

use elai_orchestrator::observable_effect::{
    check_goodhart_risk, ConfoundFlags, EffectSet, EnvironmentSnapshot, ObservableEffect,
    ObservationContext, ObservationOutcome, ObservationResult, SandboxTier, TriggerSpec,
    TypedObservation,
};
use elai_orchestrator::runtime_config::{
    ObservableEffectRuntimeConfig, ELAI_OBSERVABLE_EFFECT_ENABLE_ENV,
};

fn make_effect(id: &str, trigger: TriggerSpec, expected: TypedObservation) -> ObservableEffect {
    ObservableEffect {
        effect_id: id.into(),
        trigger,
        expected,
        sandbox: SandboxTier::ContainerIsolated,
        confound_flags: ConfoundFlags::default(),
    }
}

#[test]
fn dispatch_disabled_by_default_no_env_var() {
    // ELAI_OBSERVABLE_EFFECT_ENABLE absence MUST keep the hook dark.
    // We do NOT mutate process env (parallel-test hostile); we assert
    // the canonical env var name + the parser-level disabled default.
    assert_eq!(
        ELAI_OBSERVABLE_EFFECT_ENABLE_ENV,
        "ELAI_OBSERVABLE_EFFECT_ENABLE"
    );
    let cfg = ObservableEffectRuntimeConfig::default();
    assert!(!cfg.enable, "[observable_effect].enable must default to false");
}

#[test]
fn goodhart_check_fires_when_set_has_duplicate_trigger_observation_pairing() {
    // Eureka §3.3 component-variation invariant: when one component of
    // an effect is constant across runs (here: the same
    // trigger->observation pairing repeated), the EffectSet is not
    // orthogonal and check_goodhart_risk fires.
    let mut set = EffectSet::new();
    set.push(make_effect(
        "e1",
        TriggerSpec::TestCommand("a".into()),
        TypedObservation::StringMatch("ok".into()),
    ));
    set.push(make_effect(
        "e2",
        TriggerSpec::TestCommand("b".into()),
        TypedObservation::StringMatch("ok".into()),
    ));
    let flags = check_goodhart_risk(&set);
    assert_eq!(flags.len(), 2, "both effects in the duplicate pair must flag");
    for f in &flags {
        assert_eq!(f.risk_kind, "constant_component");
        assert!(
            f.evidence.contains("test_command->string_match"),
            "evidence should cite the duplicate pair"
        );
    }
}

#[test]
fn sandbox_observation_tier_hermetic_round_trip() {
    // Coscientist §4 SandboxObservation taxonomy: a sandbox tier
    // round-trips through ObservationResult and stays tagged with the
    // chosen tier (container_isolated representing Hermetic-class
    // here).
    let result = ObservationResult {
        effect_id: "eff-sandbox".into(),
        context: ObservationContext::Sandbox(SandboxTier::ContainerIsolated),
        outcome: ObservationOutcome::Confirmed,
        instrument_file: None,
    };
    let s = serde_json::to_string(&result).expect("serialize");
    assert!(s.contains("container_isolated"), "tier must round-trip: {s}");
    assert!(s.contains("confirmed"), "outcome must round-trip: {s}");
    assert!(!result.requires_human_review(),
        "sandbox observations do not require human review by default");
}

#[test]
fn production_observation_without_instrument_file_requires_human_review() {
    // Coscientist §4 invariant: ProductionObservation without
    // instrument_file MUST gate on human review.
    let result_no_instrument = ObservationResult {
        effect_id: "eff-prod".into(),
        context: ObservationContext::Production,
        outcome: ObservationOutcome::Confirmed,
        instrument_file: None,
    };
    assert!(
        result_no_instrument.requires_human_review(),
        "production + no instrument MUST require human review"
    );

    let result_with_instrument = ObservationResult {
        instrument_file: Some(PathBuf::from("crates/x/src/instrument.rs")),
        ..result_no_instrument
    };
    assert!(
        !result_with_instrument.requires_human_review(),
        "production + instrument may auto-promote"
    );
}

#[test]
fn effect_set_validate_orthogonality_rejects_duplicate_trigger_observation() {
    let mut set = EffectSet::new();
    set.push(make_effect(
        "e1",
        TriggerSpec::HttpEndpointGet {
            url: "http://x".into(),
            expected_status: 200,
        },
        TypedObservation::HttpStatus(200),
    ));
    set.push(make_effect(
        "e2",
        TriggerSpec::HttpEndpointGet {
            url: "http://y".into(),
            expected_status: 200,
        },
        TypedObservation::HttpStatus(200),
    ));
    let dupes = set.validate_orthogonality();
    assert_eq!(dupes.len(), 1);
    assert_eq!(dupes[0], "http_endpoint_get->http_status");
    assert!(!set.is_orthogonal());
}

#[test]
fn probe_with_effects_default_impl_projects_stub_results_to_observation_results() {
    use elai_orchestrator::feasibility_probe::FeasibilityProber;
    use elai_orchestrator::{
        DefaultFeasibilityProber, FeasibilityReport, ResourceMargins, StubResult, TaskFeasibility,
    };

    let tmp = tempfile::TempDir::new().expect("tempdir");
    let prober = DefaultFeasibilityProber::new(tmp.path().to_path_buf());
    let report = FeasibilityReport {
        task_feasibility: TaskFeasibility::Feasible,
        resource_margins: ResourceMargins {
            vram_margin_pct: 100.0,
            ram_margin_pct: 100.0,
            disk_margin_pct: 100.0,
            all_margins_safe: true,
        },
        novel_steps: Vec::new(),
        stub_results: vec![
            StubResult {
                step_id: "s1".into(),
                stub_passed: true,
                elapsed_ms: 5,
                error: None,
                ..Default::default()
            },
            StubResult {
                step_id: "s2".into(),
                stub_passed: false,
                elapsed_ms: 10,
                error: Some("boom".into()),
                ..Default::default()
            },
        ],
        escalations: Vec::new(),
        skip_reason: None,
    };

    let results = prober.probe_with_effects(&report);
    assert_eq!(results.len(), 2);
    let r0 = &results[0];
    assert_eq!(r0.effect_id, "s1");
    assert!(matches!(r0.outcome, ObservationOutcome::Confirmed));
    assert!(matches!(
        r0.context,
        ObservationContext::Sandbox(SandboxTier::ContainerIsolated)
    ));
    let r1 = &results[1];
    assert_eq!(r1.effect_id, "s2");
    assert!(matches!(r1.outcome, ObservationOutcome::Refuted));
}

#[test]
fn environment_snapshot_voyager_inv5_state_delta() {
    // Voyager INV-5: a real observable effect must produce a non-zero
    // state delta between pre-effect and post-effect snapshots.
    let pre = EnvironmentSnapshot {
        observable_keys: vec!["k1".into()],
        observed_at_ts: 0,
    };
    let post_no_delta = EnvironmentSnapshot {
        observable_keys: vec!["k1".into()],
        observed_at_ts: 100,
    };
    assert!(post_no_delta.delta(&pre).is_empty());
    assert!(!post_no_delta.delta_meets_threshold(&pre, 1));

    let post_with_delta = EnvironmentSnapshot {
        observable_keys: vec!["k1".into(), "k2".into()],
        observed_at_ts: 100,
    };
    assert_eq!(post_with_delta.delta(&pre).len(), 1);
    assert!(post_with_delta.delta_meets_threshold(&pre, 1));
}

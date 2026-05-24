//! D-18 closure contract: typed-ObservableEffect 4-variant pack +
//! C-22 dyn-safety preserved.
//!
//! Pins the following invariants:
//!   1. `SCHEMA_VERSION_PUBLIC >= 129` (post-D-18-closure atomic bump).
//!   2. Four new SsbActionType variants exist + round-trip through
//!      `from_str_public` / `as_sql_str`:
//!        * `ObservableEffectDispatched`
//!        * `GoodhartRiskFlag`
//!        * `SandboxObservation`
//!        * `ProductionObservationProposal`
//!   3. Each variant's payload struct round-trips through msgpack
//!      (write -> read) via SsbPayload::action_type symmetry.
//!   4. `ObservationContext` + `ObservationOutcome` round-trip through
//!      serde (closed snake-case enum surface).
//!   5. C-22 dyn-safety preserved: `Arc<dyn FeasibilityProber>` still
//!      compiles AND can call `probe_with_effects` (the new default-
//!      method sibling) without breaking object-safety.
//!   6. Coscientist §4 invariant: production observation without
//!      instrument_file requires human review.
//!
//! These invariants are deterministic; no I/O, no SSB DB, no shell-out.
//! Plan: docs/thefinalglance/D-plan/D-18_observable_effect_typed.md.
//! Paper grounding (DECISION-6 substituted citations):
//!   * Voyager arxiv:2305.16291 §3 (skill-with-measurable-effect)
//!   * Eureka arxiv:2310.12931 §3.3 (component-variation Goodhart guard)
//!   * Coscientist arxiv:2304.05332 §4 (sandbox vs production observation)
//! The plan's arxiv:2412.16367 cite is REJECTED per
//! workflow/reasoning/D-18-decisions.md DECISION-6.

use std::sync::Arc;

use elai_orchestrator::feasibility_probe::FeasibilityProber;
use elai_orchestrator::observable_effect::{
    ObservationContext, ObservationOutcome, ObservationResult, SandboxTier,
};
use elai_orchestrator::ssb::{
    ssb_action_type_from_str_public, GoodhartRiskFlagPayload, ObservableEffectDispatchedPayload,
    ProductionObservationProposalPayload, SandboxObservationPayload, SsbActionType, SsbPayload,
    SCHEMA_VERSION_PUBLIC,
};
use elai_orchestrator::DefaultFeasibilityProber;

#[test]
fn d18_t1_schema_version_at_or_above_129() {
    assert!(
        SCHEMA_VERSION_PUBLIC >= 129,
        "D-18 closure atomic bump must land at >= 129 (got {})",
        SCHEMA_VERSION_PUBLIC
    );
}

#[test]
fn d18_t2_four_new_ssb_action_types_round_trip() {
    let pairs = [
        (
            SsbActionType::ObservableEffectDispatched,
            "observable_effect_dispatched",
        ),
        (SsbActionType::GoodhartRiskFlag, "goodhart_risk_flag"),
        (SsbActionType::SandboxObservation, "sandbox_observation"),
        (
            SsbActionType::ProductionObservationProposal,
            "production_observation_proposal",
        ),
    ];
    for (variant, wire) in &pairs {
        assert_eq!(variant.as_sql_str(), *wire, "as_sql_str symmetry");
        let parsed = ssb_action_type_from_str_public(wire).expect("from_str symmetry");
        assert_eq!(&parsed, variant, "round-trip");
    }
}

#[test]
fn d18_t3_observable_effect_dispatched_payload_round_trips_via_action_type() {
    let payload = SsbPayload::ObservableEffectDispatched(ObservableEffectDispatchedPayload {
        effect_id: "e1".into(),
        trigger: "test_command".into(),
        context_kind: "sandbox".into(),
        ts_ms: 1_700_000_000_000,
    });
    assert_eq!(
        payload.action_type(),
        SsbActionType::ObservableEffectDispatched
    );
    let bytes = rmp_serde::to_vec_named(&payload).expect("encode");
    let decoded: SsbPayload = rmp_serde::from_slice(&bytes).expect("decode");
    assert!(matches!(decoded, SsbPayload::ObservableEffectDispatched(_)));
}

#[test]
fn d18_t4_goodhart_risk_flag_payload_round_trips_via_action_type() {
    let payload = SsbPayload::GoodhartRiskFlag(GoodhartRiskFlagPayload {
        effect_id: "e2".into(),
        risk_kind: "constant_component".into(),
        evidence: "component variation 0.0001 < 0.001".into(),
        ts_ms: 1_700_000_000_000,
    });
    assert_eq!(payload.action_type(), SsbActionType::GoodhartRiskFlag);
    let bytes = rmp_serde::to_vec_named(&payload).expect("encode");
    let decoded: SsbPayload = rmp_serde::from_slice(&bytes).expect("decode");
    assert!(matches!(decoded, SsbPayload::GoodhartRiskFlag(_)));
}

#[test]
fn d18_t5_sandbox_observation_payload_round_trips_via_action_type() {
    let payload = SsbPayload::SandboxObservation(SandboxObservationPayload {
        effect_id: "e3".into(),
        tier: "container_isolated".into(),
        outcome: "confirmed".into(),
        evidence_count: 3,
        ts_ms: 1_700_000_000_000,
    });
    assert_eq!(payload.action_type(), SsbActionType::SandboxObservation);
    let bytes = rmp_serde::to_vec_named(&payload).expect("encode");
    let decoded: SsbPayload = rmp_serde::from_slice(&bytes).expect("decode");
    assert!(matches!(decoded, SsbPayload::SandboxObservation(_)));
}

#[test]
fn d18_t6_production_observation_proposal_payload_round_trips_via_action_type() {
    let payload =
        SsbPayload::ProductionObservationProposal(ProductionObservationProposalPayload {
            effect_id: "e4".into(),
            instrument_file: None,
            requires_human_review: true,
            ts_ms: 1_700_000_000_000,
        });
    assert_eq!(
        payload.action_type(),
        SsbActionType::ProductionObservationProposal
    );
    let bytes = rmp_serde::to_vec_named(&payload).expect("encode");
    let decoded: SsbPayload = rmp_serde::from_slice(&bytes).expect("decode");
    assert!(matches!(decoded, SsbPayload::ProductionObservationProposal(_)));
}

#[test]
fn d18_t7_observation_context_serde_round_trip() {
    // Sandbox arm carries the tier; production arm is unparameterized.
    let sandbox = ObservationContext::Sandbox(SandboxTier::ContainerIsolated);
    let s = serde_json::to_string(&sandbox).expect("serialize sandbox");
    assert!(s.contains("sandbox"), "sandbox tag: {s}");
    assert!(s.contains("container_isolated"), "tier label: {s}");
    let parsed: ObservationContext = serde_json::from_str(&s).expect("deserialize sandbox");
    assert_eq!(parsed.kind_str(), "sandbox");

    let production = ObservationContext::Production;
    let s = serde_json::to_string(&production).expect("serialize production");
    assert!(s.contains("production"), "production tag: {s}");
    let parsed: ObservationContext = serde_json::from_str(&s).expect("deserialize production");
    assert_eq!(parsed.kind_str(), "production");
}

#[test]
fn d18_t8_observation_outcome_serde_round_trip() {
    for o in [
        ObservationOutcome::Confirmed,
        ObservationOutcome::Refuted,
        ObservationOutcome::Inconclusive,
    ] {
        let s = serde_json::to_string(&o).expect("serialize");
        let parsed: ObservationOutcome = serde_json::from_str(&s).expect("deserialize");
        assert_eq!(parsed.as_str(), o.as_str());
    }
}

#[test]
fn d18_t9_c22_dyn_safety_preserved_with_probe_with_effects_default_impl() {
    // C-22 contract: `Arc<dyn FeasibilityProber>` MUST stay object-safe.
    // The new `probe_with_effects` default method is sync, takes
    // `&self`, and returns a non-generic `Vec<ObservationResult>` — so
    // adding it does NOT break dyn-safety. This test compiles only if
    // that invariant holds (the assignment to the trait object exercises
    // the contract at the compiler).
    let tmp = tempfile::TempDir::new().expect("tempdir");
    let concrete = Arc::new(DefaultFeasibilityProber::new(tmp.path().to_path_buf()));
    let arc_box: Arc<dyn FeasibilityProber> = concrete;
    // Reach for the new method through the trait object — proves the
    // default-impl is callable on the dyn surface.
    let report = elai_orchestrator::FeasibilityReport {
        task_feasibility: elai_orchestrator::TaskFeasibility::Feasible,
        resource_margins: elai_orchestrator::ResourceMargins {
            vram_margin_pct: 100.0,
            ram_margin_pct: 100.0,
            disk_margin_pct: 100.0,
            all_margins_safe: true,
        },
        novel_steps: Vec::new(),
        stub_results: Vec::new(),
        escalations: Vec::new(),
        skip_reason: None,
    };
    let results: Vec<ObservationResult> = arc_box.probe_with_effects(&report);
    assert!(results.is_empty());
}

#[test]
fn d18_t10_production_without_instrument_file_requires_human_review() {
    // Coscientist §4 invariant pinned at the contract surface.
    let prod_no_instrument = ObservationResult {
        effect_id: "e-prod".into(),
        context: ObservationContext::Production,
        outcome: ObservationOutcome::Confirmed,
        instrument_file: None,
    };
    assert!(
        prod_no_instrument.requires_human_review(),
        "Coscientist §4: production + no instrument MUST require human review"
    );

    let prod_with_instrument = ObservationResult {
        instrument_file: Some(std::path::PathBuf::from("crates/x/src/instrument.rs")),
        ..prod_no_instrument
    };
    assert!(
        !prod_with_instrument.requires_human_review(),
        "production + instrument may auto-promote"
    );
}

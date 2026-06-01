//! E-47 -- LoRA ship-gate contract tests.
//!
//! Pins the behaviour of the LoRA ship-gate end-to-end:
//!   * Default config (deferral token + confirm_user_understanding)
//!     refuses adapter load.
//!   * Disabled gate (wire-dark posture) returns CalibrationPassed.
//!   * Enabled gate without calibration returns CalibrationMissing.
//!   * Enabled gate with passing baseline returns CalibrationPassed.
//!   * SsbActionType::LoraShipGateEvent + LoraShipGateEventPhase
//!     serde round-trip via ssb_action_type_from_str_public and
//!     phase as_str() labels match the wire convention.
//!   * SCHEMA_VERSION_PUBLIC reflects the E-47 bump (191 -> 192).
//!
//! NO cargo test execution by the main LLM; structural shape mirrors
//! crates/contracts/tests/idea_framing_contract.rs.

use elai_orchestrator::lora_ship_gate::{
    check_lora_ship_gate, lora_ship_gate_enabled_from_env,
    outcome_to_event_kind, LoraCalibrationReport, LoraShipGateConfig,
    LoraShipGateOutcome, LORA_SHIP_GATE_DEFERRAL_TOKEN, NO_REGRESSION_FLOOR_PCT,
};
use elai_orchestrator::ssb::LoraShipGateEventPhase;
use elai_orchestrator::{
    ssb_action_type_from_str_public, SsbActionType, SCHEMA_VERSION_PUBLIC,
};

// ---------------------------------------------------------------------------
// Contract -- SCHEMA_VERSION reflects the E-47 bump.
// ---------------------------------------------------------------------------

#[test]
fn contract_schema_version_is_192_after_e47() {
    assert!(
        SCHEMA_VERSION_PUBLIC >= 192,
        "schema only drifts forward; this contract was written at version 192. \
         CONTRACT: SCHEMA_VERSION_PUBLIC must be >= 192 after the E-47 \
         LoRA ship-gate bump (191 -> 192). Update \
         ssb.rs ledger comments + action_type_lock.golden together on \
         the next bump."
    );
}

// ---------------------------------------------------------------------------
// Contract -- the new SsbActionType variant round-trips via from_str.
// ---------------------------------------------------------------------------

#[test]
fn contract_lora_ship_gate_event_from_str_round_trip() {
    let actual =
        ssb_action_type_from_str_public("lora_ship_gate_event").expect("from_str");
    assert_eq!(
        actual,
        SsbActionType::LoraShipGateEvent,
        "CONTRACT: ssb_action_type_from_str_public(\"lora_ship_gate_event\") \
         must yield SsbActionType::LoraShipGateEvent"
    );
}

// ---------------------------------------------------------------------------
// Contract -- LoraShipGateEventPhase serde round-trip + as_str labels.
// ---------------------------------------------------------------------------

#[test]
fn test_lora_ship_gate_event_kind_round_trip() {
    let cases = [
        (LoraShipGateEventPhase::Refused, "refused"),
        (
            LoraShipGateEventPhase::CalibrationMissing,
            "calibration_missing",
        ),
        (
            LoraShipGateEventPhase::CalibrationPassed,
            "calibration_passed",
        ),
    ];
    for (phase, expected_wire) in &cases {
        assert_eq!(
            phase.as_str(),
            *expected_wire,
            "CONTRACT: LoraShipGateEventPhase::{:?}.as_str() must equal {:?}",
            phase,
            expected_wire
        );
        let json = serde_json::to_string(phase).expect("serialize");
        let expected_json = format!("\"{}\"", expected_wire);
        assert_eq!(
            json, expected_json,
            "CONTRACT: LoraShipGateEventPhase::{:?} JSON must equal {:?}",
            phase, expected_json
        );
        let decoded: LoraShipGateEventPhase =
            serde_json::from_str(&json).expect("deserialize");
        assert_eq!(
            decoded, *phase,
            "CONTRACT: LoraShipGateEventPhase JSON must round-trip"
        );
    }
}

// ---------------------------------------------------------------------------
// Contract -- default config (deferral token + confirm) returns Refused.
// ---------------------------------------------------------------------------

#[test]
fn test_lora_ship_gate_default_deferred_returns_refused() {
    let cfg = LoraShipGateConfig {
        ship_gate_token: LORA_SHIP_GATE_DEFERRAL_TOKEN.to_owned(),
        confirm_user_understanding: true,
        enabled: true,
    };
    let outcome = check_lora_ship_gate(&cfg, None);
    assert_eq!(
        outcome,
        LoraShipGateOutcome::Refused,
        "CONTRACT: default-deferred posture (deferral token present + \
         operator confirmed understanding + enabled) must REFUSE adapter \
         load. This is the load-bearing E-47 invariant."
    );
    assert_eq!(
        outcome_to_event_kind(outcome),
        LoraShipGateEventPhase::Refused,
        "CONTRACT: Refused outcome maps to Refused event phase"
    );
}

// ---------------------------------------------------------------------------
// Contract -- disabled gate (wire-dark) returns CalibrationPassed.
// ---------------------------------------------------------------------------

#[test]
fn test_lora_ship_gate_disabled_returns_passed() {
    let cfg = LoraShipGateConfig {
        ship_gate_token: LORA_SHIP_GATE_DEFERRAL_TOKEN.to_owned(),
        confirm_user_understanding: false,
        enabled: false,
    };
    let outcome = check_lora_ship_gate(&cfg, None);
    assert_eq!(
        outcome,
        LoraShipGateOutcome::CalibrationPassed,
        "CONTRACT: wire-dark posture (enabled = false) must NOT block the \
         adapter load -- the gate is inactive until the operator opts in."
    );
}

// ---------------------------------------------------------------------------
// Contract -- enabled + no token + no calibration returns missing.
// ---------------------------------------------------------------------------

#[test]
fn test_lora_ship_gate_no_calibration_returns_missing() {
    let cfg = LoraShipGateConfig {
        ship_gate_token: "BASELINE_VERIFIED".to_owned(),
        confirm_user_understanding: true,
        enabled: true,
    };
    let outcome = check_lora_ship_gate(&cfg, None);
    assert_eq!(
        outcome,
        LoraShipGateOutcome::CalibrationMissing,
        "CONTRACT: enabled + non-deferral token + no calibration must \
         surface CalibrationMissing so the operator knows the producer \
         is RUNPOD-PENDING."
    );
}

// ---------------------------------------------------------------------------
// Contract -- enabled + passing calibration returns CalibrationPassed.
// ---------------------------------------------------------------------------

#[test]
fn test_lora_ship_gate_with_baseline_calibration_returns_passed() {
    let cfg = LoraShipGateConfig {
        ship_gate_token: "BASELINE_VERIFIED".to_owned(),
        confirm_user_understanding: true,
        enabled: true,
    };
    let report = LoraCalibrationReport {
        mmlu_delta_pct: -1.0,
        humaneval_delta_pct: 0.5,
    };
    assert!(
        report.passes_threshold(),
        "CONTRACT: a -1.0pp MMLU + +0.5pp HumanEval report must pass the \
         -2.0pp no-regression floor"
    );
    let outcome = check_lora_ship_gate(&cfg, Some(&report));
    assert_eq!(
        outcome,
        LoraShipGateOutcome::CalibrationPassed,
        "CONTRACT: passing calibration on a flipped gate must clear the \
         adapter load"
    );
}

// ---------------------------------------------------------------------------
// Contract -- env-gate default is OFF (wire-dark).
// ---------------------------------------------------------------------------

#[test]
fn test_lora_ship_gate_env_gate_default_off() {
    // Capture the existing env state so we can restore it without
    // leaking changes into sibling tests in the same process. We do
    // NOT unset the env var unconditionally because cargo test runs
    // contract tests in shared process space.
    let prior = std::env::var("ELAI_LORA_SHIP_GATE_ENABLE").ok();
    // Best-effort unset for this test; restored at the end. Edition
    // 2021 env::remove_var / env::set_var are safe.
    std::env::remove_var("ELAI_LORA_SHIP_GATE_ENABLE");
    assert!(
        !lora_ship_gate_enabled_from_env(),
        "CONTRACT: ELAI_LORA_SHIP_GATE_ENABLE unset must yield wire-dark \
         (false) from lora_ship_gate_enabled_from_env"
    );
    if let Some(v) = prior {
        std::env::set_var("ELAI_LORA_SHIP_GATE_ENABLE", v);
    }
}

// ---------------------------------------------------------------------------
// Contract -- NO_REGRESSION_FLOOR_PCT is exactly -2.0pp.
// ---------------------------------------------------------------------------

#[test]
fn test_lora_ship_gate_no_regression_floor_locked_at_minus_two_pp() {
    assert!(
        (NO_REGRESSION_FLOOR_PCT - (-2.0)).abs() < f32::EPSILON,
        "CONTRACT: NO_REGRESSION_FLOOR_PCT must equal -2.0pp per E-47 \
         source report (workflow/improvement-sweep-2026-05-19/reports/\
         47-lora-finetuning.md Calibration + Refuse Conditions section). \
         Bumping the floor requires E-47.1 patch + decisions doc update."
    );
}

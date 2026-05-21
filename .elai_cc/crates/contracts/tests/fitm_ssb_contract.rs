// E-32: Contract assertions for FITM and deep-mode SSB action types.
//
// Verifies that the 4 new E-32 SSB variants compile, round-trip through
// as_sql_str / ssb_action_type_from_str, and have correct payload field types.
// This is a compile-time + round-trip contract, not a behavioural test.

use elai_orchestrator::ssb::{
    ssb_action_type_from_str_public, DeepModeActivatedPayload, DeepModeEscalationChosenPayload,
    FitmCalibrationAppliedPayload, SsbActionType, SsbPayload, YarnRefusedPayload,
};

// ---------------------------------------------------------------------------
// SsbActionType round-trip: as_sql_str -> ssb_action_type_from_str_public
// ---------------------------------------------------------------------------

#[test]
fn fitm_calibration_applied_round_trips() {
    let t = SsbActionType::FitmCalibrationApplied;
    let s = t.as_sql_str();
    assert_eq!(s, "fitm_calibration_applied");
    let back = ssb_action_type_from_str_public(s).expect("round-trip failed");
    assert_eq!(back, SsbActionType::FitmCalibrationApplied);
}

#[test]
fn deep_mode_activated_round_trips() {
    let t = SsbActionType::DeepModeActivated;
    let s = t.as_sql_str();
    assert_eq!(s, "deep_mode_activated");
    let back = ssb_action_type_from_str_public(s).expect("round-trip failed");
    assert_eq!(back, SsbActionType::DeepModeActivated);
}

#[test]
fn deep_mode_escalation_chosen_round_trips() {
    let t = SsbActionType::DeepModeEscalationChosen;
    let s = t.as_sql_str();
    assert_eq!(s, "deep_mode_escalation_chosen");
    let back = ssb_action_type_from_str_public(s).expect("round-trip failed");
    assert_eq!(back, SsbActionType::DeepModeEscalationChosen);
}

#[test]
fn yarn_refused_round_trips() {
    let t = SsbActionType::YarnRefused;
    let s = t.as_sql_str();
    assert_eq!(s, "yarn_refused");
    let back = ssb_action_type_from_str_public(s).expect("round-trip failed");
    assert_eq!(back, SsbActionType::YarnRefused);
}

// ---------------------------------------------------------------------------
// SsbPayload action_type() matches SsbActionType
// ---------------------------------------------------------------------------

#[test]
fn fitm_payload_action_type() {
    let p = SsbPayload::FitmCalibrationApplied(FitmCalibrationAppliedPayload {
        prompt_len: 10_000,
        dummy_len: 50,
        applied_ms: 130.0,
    });
    assert_eq!(p.action_type(), SsbActionType::FitmCalibrationApplied);
}

#[test]
fn deep_mode_activated_payload_action_type() {
    let p = SsbPayload::DeepModeActivated(DeepModeActivatedPayload {
        model_id: "Qwen/Qwen2.5-7B-Instruct-1M".to_string(),
        ctx_cap: 64_000,
        vram_gb: 24.0,
    });
    assert_eq!(p.action_type(), SsbActionType::DeepModeActivated);
}

#[test]
fn deep_mode_escalation_chosen_payload_action_type() {
    let p = SsbPayload::DeepModeEscalationChosen(DeepModeEscalationChosenPayload {
        reason: "cross-file synthesis: 4 files".to_string(),
        files_referenced: 4,
    });
    assert_eq!(p.action_type(), SsbActionType::DeepModeEscalationChosen);
}

#[test]
fn yarn_refused_payload_action_type() {
    let p = SsbPayload::YarnRefused(YarnRefusedPayload {
        model_name: "Qwen2.5-Coder-7B".to_string(),
        reason: "YaRN rejected per E-32 (vllm issue 18728)".to_string(),
    });
    assert_eq!(p.action_type(), SsbActionType::YarnRefused);
}

// ---------------------------------------------------------------------------
// Payload field types (compile-time contract)
// ---------------------------------------------------------------------------

fn _assert_fitm_payload_fields(p: FitmCalibrationAppliedPayload) {
    let _: u32 = p.prompt_len;
    let _: u32 = p.dummy_len;
    let _: f64 = p.applied_ms;
}

fn _assert_deep_mode_activated_fields(p: DeepModeActivatedPayload) {
    let _: String = p.model_id;
    let _: u32 = p.ctx_cap;
    let _: f32 = p.vram_gb;
}

fn _assert_deep_mode_escalation_fields(p: DeepModeEscalationChosenPayload) {
    let _: String = p.reason;
    let _: u32 = p.files_referenced;
}

fn _assert_yarn_refused_fields(p: YarnRefusedPayload) {
    let _: String = p.model_name;
    let _: String = p.reason;
}

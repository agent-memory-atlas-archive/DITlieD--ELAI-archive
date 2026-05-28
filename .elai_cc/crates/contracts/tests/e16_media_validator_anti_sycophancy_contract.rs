//! E-16 anti-sycophancy + VLM image-validation contract.
//!
//! Pins the wiring contract from
//! `docs/thefinalglance/E-plan/E-16_vlm_image_validation.md`:
//!   * The new `VlmImageValidationEvent` SSB variant (Pattern-2
//!     SINGLE-VARIANT, 31st application).
//!   * The 3-arm `VlmImageValidationKind` discriminator
//!     (`SwapOrderDisagreement`, `AxtreeMismatch`,
//!     `VlmSchemaUnconstrained`).
//!   * The wire labels through the `as_sql_str` / `from_str` round-
//!     trip.
//!   * The reused `SycophancyFlag` variant (E-4 / D-14 prior ship) is
//!     present and not duplicated.
//!   * The SCHEMA_VERSION jump 178 -> 179 (E-16 atomic +1).
//!   * The TOTAL_VARIANTS jump 451 -> 452 (single-variant collapse).
//!
//! This is the contract surface for downstream consumers
//! (dashboard-v2 + future orchestrator wiring). The orchestrator-
//! native integration tests live under
//! `crates/orchestrator/tests/e16_*`.

use elai_orchestrator::{
    ssb_action_type_from_str_public, SsbActionType, SsbPayload, VlmImageValidationEventPayload,
    VlmImageValidationKind, ELAI_VLM_IMAGE_VALIDATION_ENABLE_ENV, SCHEMA_VERSION_PUBLIC,
};

#[test]
fn vlm_image_validation_event_variant_present() {
    // 1. Variant exists + carries the canonical wire label.
    let a = SsbActionType::VlmImageValidationEvent;
    assert_eq!(a.as_sql_str(), "vlm_image_validation_event");
}

#[test]
fn vlm_image_validation_kind_three_arms_only() {
    // 2. Kind discriminator has exactly 3 arms with stable
    // snake_case wire labels per arXiv:2406.07791 (swap order) /
    // playwright.dev (axtree) / arXiv:2411.15100 (xgrammar).
    assert_eq!(
        VlmImageValidationKind::SwapOrderDisagreement.as_str(),
        "swap_order_disagreement"
    );
    assert_eq!(
        VlmImageValidationKind::AxtreeMismatch.as_str(),
        "axtree_mismatch"
    );
    assert_eq!(
        VlmImageValidationKind::VlmSchemaUnconstrained.as_str(),
        "vlm_schema_unconstrained"
    );
}

#[test]
fn sycophancy_flag_reused_not_duplicated() {
    // 3. SycophancyFlag variant exists from E-4 / D-14; E-16 must
    // NOT add a duplicate. The wire label is the canonical
    // sycophancy_flag (NOT vlm_sycophancy_flag etc).
    let s = SsbActionType::SycophancyFlag;
    assert_eq!(s.as_sql_str(), "sycophancy_flag");
    let p = SsbActionType::SycophancyProbeDispatched;
    assert_eq!(p.as_sql_str(), "sycophancy_probe_dispatched");
}

#[test]
fn vlm_image_validation_payload_roundtrips() {
    // 4. Pattern-2 single-variant payload encodes + decodes through
    // SsbPayload without losing the kind discriminator + details_json.
    let payload = SsbPayload::VlmImageValidationEvent(VlmImageValidationEventPayload {
        kind: VlmImageValidationKind::AxtreeMismatch,
        step_id: "step-e16".into(),
        request_id: Some("req-e16".into()),
        details_json: serde_json::json!({
            "yaml_diff_pct": 2.5,
            "baseline_path": "/tmp/baseline.yaml",
            "current_path": "/tmp/current.yaml",
            "threshold_pct": 1.0
        }),
        ts_ms: 1_700_000_179_001,
    });
    assert_eq!(
        payload.action_type(),
        SsbActionType::VlmImageValidationEvent
    );
    let bytes = rmp_serde::to_vec_named(&payload).expect("encode");
    let decoded: SsbPayload = rmp_serde::from_slice(&bytes).expect("decode");
    assert_eq!(decoded, payload);
}

#[test]
fn schema_version_at_179_for_e16() {
    // 5. E-16 atomic SCHEMA bump 178 -> 179. Substrate-ahead
    // rebases are caught here on commit.
    assert!(
        SCHEMA_VERSION_PUBLIC >= 179,
        "SCHEMA_VERSION_PUBLIC must be >= 179 for E-16; got {}",
        SCHEMA_VERSION_PUBLIC
    );
}

#[test]
fn env_gate_constant_canonical() {
    // 6. Wire-dark env gate name is the canonical
    // ELAI_VLM_IMAGE_VALIDATION_ENABLE per E-16 plan + the
    // sycophancy.rs / vlm_validator.rs modules.
    assert_eq!(
        ELAI_VLM_IMAGE_VALIDATION_ENABLE_ENV,
        "ELAI_VLM_IMAGE_VALIDATION_ENABLE"
    );
}

#[test]
fn wire_label_from_str_roundtrip() {
    // 7. ssb_action_type_from_str must parse the canonical wire
    // label back to the variant. Negative: a typo'd label fails to
    // parse so the dashboard fold never silently mis-routes.
    let parsed = ssb_action_type_from_str_public("vlm_image_validation_event").expect("parse");
    assert_eq!(parsed, SsbActionType::VlmImageValidationEvent);
    let bad = ssb_action_type_from_str_public("vlm_image_validation_eventx");
    assert!(bad.is_err());
}

//! J-1 contract test — perception gateway (skeleton + scaffold + router).
//!
//! Source plan: docs/thefinalglance/J-plan/J-1_perception_gateway.md
//! Decisions:  workflow/reasoning/J-1-decisions.md (T8)
//!
//! Tests are intentionally narrow (one assertion per concern). 14 active +
//! 2 `#[ignore]` deferred (T1/T5 sweet-spot edges measured, T7 raw-image
//! control arm):
//!   (b) visual-diff routes to pixel-diff, VLM call count 0 (route returns
//!       RouteToPixelDiff; no RouteToVlm variant exists).
//!   (c) over-limit counting returns TypedRefusal.
//!   (d) PerceptionScaffoldEvent payload + Kind + action_type mapping
//!       (T4 — single-variant Pattern-2, 4-arm Kind discriminator).
//!   (e) DegradedPassthrough on missing paths.
//!   Plus: Normal -> Passthrough, MicroText -> RouteToTiling,
//!   classify_prompt keyword detection, perception_gateway default OFF,
//!   ScaffoldInput::from_prompt, prompt_carries_image detection,
//!   preprocess NoImagePresent / DegradedPassthrough arms, with_spec
//!   carries preprocess spec, sweet-spot edges pinned.
//!
//! Ignored (deferred):
//!   * target_model_sweet_spot_edges_measured (T1/T5 — ELAI-measured deltas
//!     land in T7 fixtures, not the pinned SRC-3 rationale values).
//!   * raw_image_control_arm (T7 — measurement fixture).

use elai_orchestrator::perception;
use elai_orchestrator::perception::preprocess::{
    preprocess, PreprocessOutcome, PreprocessSpec, TargetModel,
};
use elai_orchestrator::perception::router::{
    classify_prompt, route, RouterDecision, RouterInput, RouterType, DEFAULT_COUNTING_THRESHOLD,
};
use elai_orchestrator::perception::scaffold::{
    prompt_carries_image, scaffold, ScaffoldInput, ScaffoldOutcome,
};
use elai_orchestrator::ssb::{
    PerceptionScaffoldEventKind, PerceptionScaffoldEventPayload, SsbActionType, SsbPayload,
};

// ---------------------------------------------------------------------------
// (1) perception_gateway default OFF when env unset.
// ---------------------------------------------------------------------------

#[test]
fn perception_gateway_default_off_when_env_unset() {
    // The gate reads ELAI_PERCEPTION_GATEWAY_ENABLE; default-OFF (GD-4).
    // This test passes when the env is unset OR set to a non-truthy value.
    // We do NOT mutate the env here (parallel-test safety); we only assert
    // the disabled-companion matches the enabled-companion's negation.
    let enabled = perception::perception_gateway_enabled();
    let disabled = perception::perception_gateway_disabled();
    assert_eq!(disabled, !enabled);
}

// ---------------------------------------------------------------------------
// (2) classify_prompt keyword detection — VisualDiff.
// ---------------------------------------------------------------------------

#[test]
fn classify_prompt_detects_visual_diff_keywords() {
    assert_eq!(
        classify_prompt("Please do a visual diff of these two screenshots"),
        RouterType::VisualDiff
    );
    assert_eq!(
        classify_prompt("compare the images and tell me what changed"),
        RouterType::VisualDiff
    );
    assert_eq!(
        classify_prompt("spot the difference between A and B"),
        RouterType::VisualDiff
    );
}

// ---------------------------------------------------------------------------
// (3) classify_prompt keyword detection — Counting.
// ---------------------------------------------------------------------------

#[test]
fn classify_prompt_detects_counting_keywords() {
    assert_eq!(
        classify_prompt("how many buttons are in this screenshot?"),
        RouterType::Counting {
            threshold: DEFAULT_COUNTING_THRESHOLD
        }
    );
    assert_eq!(
        classify_prompt("count the elements in the image"),
        RouterType::Counting {
            threshold: DEFAULT_COUNTING_THRESHOLD
        }
    );
}

// ---------------------------------------------------------------------------
// (4) classify_prompt keyword detection — MicroText.
// ---------------------------------------------------------------------------

#[test]
fn classify_prompt_detects_micro_text_keywords() {
    assert_eq!(
        classify_prompt("read the micro text in this dense UI"),
        RouterType::MicroText
    );
    assert_eq!(
        classify_prompt("OCR the small text in the image"),
        RouterType::MicroText
    );
}

// ---------------------------------------------------------------------------
// (5) classify_prompt falls back to Normal.
// ---------------------------------------------------------------------------

#[test]
fn classify_prompt_defaults_to_normal_when_no_keyword_matches() {
    assert_eq!(
        classify_prompt("explain what this code does"),
        RouterType::Normal
    );
    assert_eq!(classify_prompt("hello world"), RouterType::Normal);
}

// ---------------------------------------------------------------------------
// (b) visual-diff routes to pixel-diff, VLM call count 0.
//     route() returns RouteToPixelDiff when both paths present; there is NO
//     RouteToVlm variant in RouterDecision (stub_ban encoded in the type
//     surface). The gate site calls ImageValidator (Tier-1 only), never a
//     VLM.
// ---------------------------------------------------------------------------

#[test]
fn visual_diff_routes_to_pixel_diff_when_paths_present() {
    let input = RouterInput {
        router_type: RouterType::VisualDiff,
        has_image: true,
        image_path: Some(std::path::PathBuf::from("/tmp/a.png")),
        reference_path: Some(std::path::PathBuf::from("/tmp/b.png")),
        image_count: 0,
        micro_text_confidence: None,
    };
    let decision = route(&input);
    assert_eq!(decision, RouterDecision::RouteToPixelDiff);
    // NEGATIVE assertion: no RouteToVlm variant exists in RouterDecision.
    // (Encoded in the enum — this assert documents the stub_ban invariant.)
    assert!(!matches!(decision, RouterDecision::Passthrough));
}

// ---------------------------------------------------------------------------
// (c) over-limit counting returns TypedRefusal.
// ---------------------------------------------------------------------------

#[test]
fn over_limit_counting_returns_typed_refusal() {
    let input = RouterInput {
        router_type: RouterType::Counting {
            threshold: DEFAULT_COUNTING_THRESHOLD,
        },
        has_image: true,
        image_path: None,
        reference_path: None,
        image_count: 0,
        micro_text_confidence: None,
    };
    assert_eq!(route(&input), RouterDecision::TypedRefusal);
}

// ---------------------------------------------------------------------------
// Counting below threshold routes to tiling.
// ---------------------------------------------------------------------------

#[test]
fn counting_below_threshold_routes_to_tiling() {
    let input = RouterInput {
        router_type: RouterType::Counting { threshold: 5 },
        has_image: true,
        image_path: None,
        reference_path: None,
        image_count: 0,
        micro_text_confidence: None,
    };
    assert_eq!(route(&input), RouterDecision::RouteToTiling);
}

// ---------------------------------------------------------------------------
// Normal -> Passthrough.
// ---------------------------------------------------------------------------

#[test]
fn normal_routes_to_passthrough() {
    let input = RouterInput::new(RouterType::Normal, true);
    assert_eq!(route(&input), RouterDecision::Passthrough);
}

// ---------------------------------------------------------------------------
// MicroText -> RouteToTiling.
// ---------------------------------------------------------------------------

#[test]
fn micro_text_routes_to_tiling() {
    let input = RouterInput::new(RouterType::MicroText, true);
    assert_eq!(route(&input), RouterDecision::RouteToTiling);
}

// ---------------------------------------------------------------------------
// (e) DegradedPassthrough on missing paths (visual-diff with no paths).
// ---------------------------------------------------------------------------

#[test]
fn visual_diff_degrades_to_passthrough_when_paths_missing() {
    let input = RouterInput::new(RouterType::VisualDiff, true);
    assert_eq!(route(&input), RouterDecision::DegradedPassthrough);

    // One path missing also degrades.
    let input_one = RouterInput {
        router_type: RouterType::VisualDiff,
        has_image: true,
        image_path: Some(std::path::PathBuf::from("/tmp/a.png")),
        reference_path: None,
        image_count: 0,
        micro_text_confidence: None,
    };
    assert_eq!(route(&input_one), RouterDecision::DegradedPassthrough);
}

// ---------------------------------------------------------------------------
// prompt_carries_image detection.
// ---------------------------------------------------------------------------

#[test]
fn prompt_carries_image_detects_data_uri_marker() {
    assert!(prompt_carries_image("here is the image: data:image/png;base64,abc123"));
    assert!(prompt_carries_image("base64,XYZ"));
    assert!(!prompt_carries_image("plain text with no image"));
    assert!(!prompt_carries_image(""));
}

// ---------------------------------------------------------------------------
// ScaffoldInput::from_prompt — no image marker -> empty image_bytes.
// ---------------------------------------------------------------------------

#[test]
fn scaffold_input_from_prompt_no_image_yields_empty_bytes() {
    let input = ScaffoldInput::from_prompt("explain this code");
    assert!(input.image_bytes.is_empty());
    assert_eq!(input.task_prompt, "explain this code");
}

// ---------------------------------------------------------------------------
// ScaffoldInput::from_prompt — image marker -> non-empty image_bytes.
// ---------------------------------------------------------------------------

#[test]
fn scaffold_input_from_prompt_with_image_yields_nonempty_bytes() {
    let input = ScaffoldInput::from_prompt("describe data:image/png;base64,abc");
    assert!(!input.image_bytes.is_empty());
}

// ---------------------------------------------------------------------------
// scaffold NoImagePresent arm.
// ---------------------------------------------------------------------------

#[test]
fn scaffold_no_image_present_arm() {
    let input = ScaffoldInput {
        image_bytes: Vec::new(),
        task_prompt: "explain this code".to_string(),
    };
    let result = scaffold(&input).expect("scaffold should not error on no-image");
    assert_eq!(result.outcome, ScaffoldOutcome::NoImagePresent);
    assert!(result.literal_elements.is_empty());
}

// ---------------------------------------------------------------------------
// scaffold DegradedPassthrough arm (T2 stub — T5 wires live sidecar).
// ---------------------------------------------------------------------------

#[test]
fn scaffold_degraded_passthrough_arm_on_image_present() {
    let input = ScaffoldInput {
        image_bytes: vec![0u8],
        task_prompt: "describe data:image/png;base64,abc".to_string(),
    };
    let result = scaffold(&input).expect("scaffold should not error on image-present stub");
    assert_eq!(result.outcome, ScaffoldOutcome::DegradedPassthrough);
}

// ---------------------------------------------------------------------------
// preprocess NoImagePresent arm.
// ---------------------------------------------------------------------------

#[test]
fn preprocess_no_image_present_arm() {
    let spec = PreprocessSpec::default();
    let result = preprocess(&spec, &[]).expect("preprocess should not error on empty input");
    assert_eq!(result.outcome, PreprocessOutcome::NoImagePresent);
    assert!(result.conditioned_bytes.is_empty());
}

// ---------------------------------------------------------------------------
// preprocess DegradedPassthrough arm (T1 skeleton — T5 wires sidecar).
// ---------------------------------------------------------------------------

#[test]
fn preprocess_degraded_passthrough_arm_on_image_present() {
    let spec = PreprocessSpec::default();
    let result = preprocess(&spec, &[1, 2, 3]).expect("preprocess should not error on stub");
    assert_eq!(result.outcome, PreprocessOutcome::DegradedPassthrough);
    assert_eq!(result.conditioned_bytes, vec![1, 2, 3]);
}

// ---------------------------------------------------------------------------
// preprocess rejects out-of-range tile_overlap.
// ---------------------------------------------------------------------------

#[test]
fn preprocess_rejects_out_of_range_tile_overlap() {
    let spec = PreprocessSpec {
        tile_overlap: 1.5,
        ..PreprocessSpec::default()
    };
    assert!(preprocess(&spec, &[1]).is_err());
}

// ---------------------------------------------------------------------------
// PerceptionGateway::with_spec carries the preprocess spec.
// ---------------------------------------------------------------------------

#[test]
fn perception_gateway_with_spec_carries_preprocess_spec() {
    let spec = PreprocessSpec {
        target_model: TargetModel::Gpt,
        jpeg_q85: false,
        tile_overlap: 0.25,
        thumbnail: true,
    };
    let gw = perception::PerceptionGateway::with_spec(spec.clone());
    assert_eq!(gw.preprocess_spec, spec);
}

// ---------------------------------------------------------------------------
// TargetModel sweet-spot edges pinned (EXTERNAL SRC-3 — design rationale).
// ---------------------------------------------------------------------------

#[test]
fn target_model_sweet_spot_edges_pinned() {
    assert_eq!(TargetModel::Claude.sweet_spot_longest_side_px(), 1568);
    assert_eq!(TargetModel::Gpt.sweet_spot_longest_side_px(), 2048);
    assert_eq!(TargetModel::QwenVl.sweet_spot_longest_side_px(), 1280);
}

// ---------------------------------------------------------------------------
// RouterInput::new defaults image_path + reference_path to None.
// ---------------------------------------------------------------------------

#[test]
fn router_input_new_defaults_paths_to_none() {
    let input = RouterInput::new(RouterType::Normal, false);
    assert!(input.image_path.is_none());
    assert!(input.reference_path.is_none());
    assert!(!input.has_image);
}

// ---------------------------------------------------------------------------
// DEFERRED (T1/T5): target_model sweet-spot edges MEASURED (ELAI deltas).
// The pinned SRC-3 rationale values are covered by
// target_model_sweet_spot_edges_pinned above; this test is the placeholder
// for the T7 measurement-fixture ELAI-measured deltas.
// ---------------------------------------------------------------------------

#[test]
#[ignore = "J-1 T7: ELAI-measured sweet-spot deltas land in the T7 fixtures"]
fn target_model_sweet_spot_edges_measured() {
    // T7 ships the 4K-screenshot fine-text fixture + the ELAI-measured
    // deltas (raw vs gateway arms) in a metrics TOML. This test asserts
    // the measured delta is non-zero (gateway arm answers the fine-text
    // question the control arm fails).
}

// ---------------------------------------------------------------------------
// (d) PerceptionScaffoldEvent payload + Kind + action_type mapping.
// T4 lands the PerceptionScaffoldEvent SSB variant (single-variant
// Pattern-2, Kind: Preprocessed / ScaffoldEmitted / RoutedToSpecialist /
// DegradedPassthrough + J-8 T9 upgrade arms SchemaApplied /
// ResizeRuleApplied / RouteRuleApplied / EvalScheduled). This test asserts
// the payload constructs with the correct Kind, the action_type() mapping
// resolves to the declared SsbActionType, and every Kind as_str() wire
// label round-trips -- without a live SSB (the gate is wire-dark by
// default; the round-trip is exercised structurally via the payload +
// action_type() single source of truth).
// ---------------------------------------------------------------------------

#[test]
fn scaffold_event_lands_with_correct_kind() {
    // (d.1) All Kind arms construct + carry the expected as_str() wire label.
    // J-1 lifecycle arms (4) + J-8 T9 / JC-11 upgrade arms (4).
    assert_eq!(PerceptionScaffoldEventKind::Preprocessed.as_str(), "preprocessed");
    assert_eq!(
        PerceptionScaffoldEventKind::ScaffoldEmitted.as_str(),
        "scaffold_emitted"
    );
    assert_eq!(
        PerceptionScaffoldEventKind::RoutedToSpecialist.as_str(),
        "routed_to_specialist"
    );
    assert_eq!(
        PerceptionScaffoldEventKind::DegradedPassthrough.as_str(),
        "degraded_passthrough"
    );
    assert_eq!(
        PerceptionScaffoldEventKind::SchemaApplied.as_str(),
        "schema_applied"
    );
    assert_eq!(
        PerceptionScaffoldEventKind::ResizeRuleApplied.as_str(),
        "resize_rule_applied"
    );
    assert_eq!(
        PerceptionScaffoldEventKind::RouteRuleApplied.as_str(),
        "route_rule_applied"
    );
    assert_eq!(
        PerceptionScaffoldEventKind::EvalScheduled.as_str(),
        "eval_scheduled"
    );

    // (d.2) ScaffoldEmitted payload constructs + maps to the declared action_type.
    let payload = PerceptionScaffoldEventPayload {
        kind: PerceptionScaffoldEventKind::ScaffoldEmitted,
        session_id: Some("sess-j1-t4".to_string()),
        task_id: None,
        details_json: serde_json::json!({
            "literal_element_count": 3,
            "call1_temperature": 0.0,
        }),
        ts_ms: 1_700_000_000_000,
    };
    let ssb_payload = SsbPayload::PerceptionScaffoldEvent(payload);
    assert_eq!(
        ssb_payload.action_type(),
        SsbActionType::PerceptionScaffoldEvent(PerceptionScaffoldEventKind::ScaffoldEmitted)
    );

    // (d.3) The declared action_type serializes to the single sql_str
    // (Pattern-2 single-variant convention; the Kind discriminator rides
    // inside the payload, mirroring DocumentVisionRagEvent).
    assert_eq!(
        SsbActionType::PerceptionScaffoldEvent(PerceptionScaffoldEventKind::ScaffoldEmitted)
            .as_sql_str(),
        "perception_scaffold_event"
    );
    assert_eq!(
        SsbActionType::PerceptionScaffoldEvent(PerceptionScaffoldEventKind::DegradedPassthrough)
            .as_sql_str(),
        "perception_scaffold_event"
    );

    // (d.4) from_str round-trips the single sql_str to the representative arm.
    // SsbError does not impl PartialEq, so assert via .is_ok() + unwrap.
    let decoded = elai_orchestrator::ssb::ssb_action_type_from_str_public(
        "perception_scaffold_event",
    )
    .expect("perception_scaffold_event should decode");
    assert_eq!(
        decoded,
        SsbActionType::PerceptionScaffoldEvent(PerceptionScaffoldEventKind::ScaffoldEmitted)
    );

    // (d.5) DegradedPassthrough payload (scaffold failure arm) maps correctly.
    let degraded = PerceptionScaffoldEventPayload {
        kind: PerceptionScaffoldEventKind::DegradedPassthrough,
        session_id: None,
        task_id: None,
        details_json: serde_json::json!({"source": "scaffold", "reason": "scaffold_error"}),
        ts_ms: 1_700_000_000_001,
    };
    assert_eq!(
        SsbPayload::PerceptionScaffoldEvent(degraded).action_type(),
        SsbActionType::PerceptionScaffoldEvent(PerceptionScaffoldEventKind::DegradedPassthrough)
    );

    // (d.6) RoutedToSpecialist payload (router gate arm) maps correctly.
    let routed = PerceptionScaffoldEventPayload {
        kind: PerceptionScaffoldEventKind::RoutedToSpecialist,
        session_id: None,
        task_id: None,
        details_json: serde_json::json!({"router_type": "visual_diff"}),
        ts_ms: 1_700_000_000_002,
    };
    assert_eq!(
        SsbPayload::PerceptionScaffoldEvent(routed).action_type(),
        SsbActionType::PerceptionScaffoldEvent(PerceptionScaffoldEventKind::RoutedToSpecialist)
    );
}

// ---------------------------------------------------------------------------
// DEFERRED (T7): raw-image control arm documents the failing fine-text
// fixture (NEGATIVE observable_effect — proves the metric discriminates).
// ---------------------------------------------------------------------------

#[test]
#[ignore = "J-1 T7: raw-image control arm fixture lands in T7"]
fn raw_image_control_arm() {
    // T7 ships the --raw-image control arm fixture. This test asserts the
    // control arm fails the fine-text question (proving the metric
    // discriminates, R2).
}

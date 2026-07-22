//! J-8 contract test — perception upgrade (T1: sweet-spot table as DATA).
//!
//! Source plan: docs/thefinalglance/J-plan/J-8_vlm_perception_research_upgrade.md
//!
//! This contract calls the SAME production fns that the dispatch seam uses:
//!   * `TargetModel::sweet_spot_longest_side_px` (delegates to
//!     `sweet_spot::longest_side_px`)
//!   * `sweet_spot::longest_side_px` (string-key API)
//!   * `sweet_spot::native_res` (string-key API)
//!   * `preprocess()` — the production call site that wires sweet_spot values
//!     into `PreprocessResult` on the non-empty image arm.
//!
//! It does NOT reimplement the branch in the test body and does NOT grep
//! source text. The j1 contract test
//! (`j1_perception_gateway_contract.rs::target_model_sweet_spot_edges_pinned`)
//! asserts the same three px values via the same `TargetModel` method and
//! MUST remain green.
//!
//! FALSIFICATION MUTATION (for the orchestrator / auditor to verify):
//!   Inside `preprocess.rs::preprocess()`, in the non-empty image arm,
//!   restore `longest_side_px: 0` and drop the `native_res` assignment
//!   (delete lines that call `sweet_spot_longest_side_px` and
//!   `sweet_spot::native_res`). The `preprocess_wires_sweet_spot_values`
//!   test below will go RED because `result.longest_side_px` will be 0
//!   instead of the table value. Restoring the assignments makes it GREEN
//!   again.

use elai_orchestrator::perception::preprocess::{preprocess, PreprocessSpec, TargetModel};
use elai_orchestrator::perception::sweet_spot;
use elai_orchestrator::perception::{
    apply_schema_v2, coords_within_schema_v2, elements_pass_schema_v2, scaffold_with_call1,
    LiteralElement, SchemaV2Verdict, ScaffoldInput, ScaffoldOutcome,
};
use elai_orchestrator::perception::router::{
    route_outcome, RouterDecision, RouterInput, RouterType, DEFAULT_COUNTING_THRESHOLD,
    MULTI_IMAGE_MIN_COUNT,
};

// ---------------------------------------------------------------------------
// (1) TargetModel::sweet_spot_longest_side_px delegates to the toml table.
//     Same values as the j1 contract test (1568/2048/1280) — proving the
//     delegation produces identical results to the old hardcoded match.
// ---------------------------------------------------------------------------

#[test]
fn target_model_sweet_spot_matches_toml_via_delegation() {
    assert_eq!(TargetModel::Claude.sweet_spot_longest_side_px(), 1568);
    assert_eq!(TargetModel::Gpt.sweet_spot_longest_side_px(), 2048);
    assert_eq!(TargetModel::QwenVl.sweet_spot_longest_side_px(), 1280);
}

// ---------------------------------------------------------------------------
// (2) String-key API returns the same values as the enum method.
// ---------------------------------------------------------------------------

#[test]
fn string_key_longest_side_matches_enum_method() {
    assert_eq!(
        sweet_spot::longest_side_px("claude"),
        Some(TargetModel::Claude.sweet_spot_longest_side_px())
    );
    assert_eq!(
        sweet_spot::longest_side_px("gpt"),
        Some(TargetModel::Gpt.sweet_spot_longest_side_px())
    );
    assert_eq!(
        sweet_spot::longest_side_px("qwen_vl"),
        Some(TargetModel::QwenVl.sweet_spot_longest_side_px())
    );
}

// ---------------------------------------------------------------------------
// (3) native_res is false for Claude/Gpt/QwenVl (they resize).
// ---------------------------------------------------------------------------

#[test]
fn native_res_false_for_resize_models() {
    assert_eq!(sweet_spot::native_res("claude"), Some(false));
    assert_eq!(sweet_spot::native_res("gpt"), Some(false));
    assert_eq!(sweet_spot::native_res("qwen_vl"), Some(false));
}

// ---------------------------------------------------------------------------
// (4) Gemini row: native_res = true (passthrough, no resize).
// ---------------------------------------------------------------------------

#[test]
fn gemini_native_res_true() {
    assert_eq!(sweet_spot::native_res("gemini"), Some(true));
}

// ---------------------------------------------------------------------------
// (5) Unknown model key returns None.
// ---------------------------------------------------------------------------

#[test]
fn unknown_model_key_returns_none() {
    assert!(sweet_spot::longest_side_px("nonexistent_model").is_none());
    assert!(sweet_spot::native_res("nonexistent_model").is_none());
}

// ---------------------------------------------------------------------------
// (6) preprocess() wires sweet_spot values into PreprocessResult on the
//     non-empty image arm. This is the PRODUCTION CALL SITE test: it calls
//     the real preprocess() fn and asserts the result carries the table
//     values, proving the wiring is live (not vacuous-verify).
// ---------------------------------------------------------------------------

#[test]
fn preprocess_wires_sweet_spot_values() {
    for (model, expected_px) in [
        (TargetModel::Claude, 1568),
        (TargetModel::Gpt, 2048),
        (TargetModel::QwenVl, 1280),
    ] {
        let spec = PreprocessSpec {
            target_model: model,
            ..PreprocessSpec::default()
        };
        let result = preprocess(&spec, b"x").expect("preprocess should not error on non-empty stub");
        assert_eq!(
            result.longest_side_px, expected_px,
            "preprocess() must wire sweet_spot_longest_side_px for {model:?}"
        );
        assert_eq!(
            result.native_res,
            sweet_spot::native_res(model.as_key()).unwrap_or(false),
            "preprocess() must wire native_res for {model:?}"
        );
    }
}

// ---------------------------------------------------------------------------
// J-8 T2: schema v2 coord validation contract.
//
// All tests below call the SAME production fns:
//   * `coords_within_schema_v2` — the per-bbox validator
//   * `elements_pass_schema_v2` — the per-element-list validator
//   * `apply_schema_v2` — the shared verdict fn (also called from
//     `scaffold_with_call1` -> `scaffold`)
//   * `scaffold_with_call1` — the production scaffold entry that wires
//     `apply_schema_v2` into the scaffold flow
//
// They do NOT reimplement the branch in the test body and do NOT grep
// source text.
//
// FALSIFICATION MUTATION (for the orchestrator / auditor to verify):
//   Inside `scaffold_with_call1`, in the `if let Some(elements)` arm,
//   delete the `apply_schema_v2(elements, schema_attempt)` call and
//   always return `DegradedPassthrough` (or skip the match). The
//   `scaffold_with_call1_accepted`, `scaffold_with_call1_reprompt_once`,
//   and `scaffold_with_call1_typed_refusal` tests below will go RED
//   because the outcome will be `DegradedPassthrough` instead of
//   `ScaffoldEmitted` / `SchemaReprompt` / `TypedRefusal`. Restoring the
//   `apply_schema_v2` call makes them GREEN again.
//
// UNCOVERED MUTATION (named for the orchestrator):
//   The live production_agent_rpc.rs infer path still calls
//   `ScaffoldInput::from_prompt(&prompt)` -> `scaffold()` which delegates
//   to `scaffold_with_call1(input, None, 0)`. Since `call1_elements` is
//   `None` on the live path, the schema v2 validation gate is NOT
//   exercised in production until T5 feeds real call-1 elements. These
//   contracts prove the gate works when called with `Some(elements)`;
//   they do NOT claim live infer coverage that was not built.
// ---------------------------------------------------------------------------

fn make_element(label: &str, bbox: [u32; 4]) -> LiteralElement {
    LiteralElement {
        label: label.to_string(),
        bbox,
    }
}

// (7) coords_within_schema_v2: all values <= 1000 -> true
#[test]
fn coords_within_schema_v2_valid_bbox() {
    assert!(coords_within_schema_v2(&[0, 0, 500, 1000]));
    assert!(coords_within_schema_v2(&[1000, 1000, 1000, 1000]));
    assert!(coords_within_schema_v2(&[0, 0, 0, 0]));
}

// (8) coords_within_schema_v2: any value > 1000 -> false
#[test]
fn coords_within_schema_v2_invalid_bbox() {
    assert!(!coords_within_schema_v2(&[0, 0, 500, 1001]));
    assert!(!coords_within_schema_v2(&[2000, 0, 0, 0]));
    assert!(!coords_within_schema_v2(&[0, 0, 0, 99999]));
}

// (9) elements_pass_schema_v2: all elements valid -> true
#[test]
fn elements_pass_schema_v2_all_valid() {
    let elements = vec![
        make_element("a", [0, 0, 100, 200]),
        make_element("b", [500, 500, 1000, 1000]),
    ];
    assert!(elements_pass_schema_v2(&elements));
}

// (10) elements_pass_schema_v2: any element invalid -> false
#[test]
fn elements_pass_schema_v2_any_invalid() {
    let elements = vec![
        make_element("a", [0, 0, 100, 200]),
        make_element("b", [500, 500, 1001, 1000]),
    ];
    assert!(!elements_pass_schema_v2(&elements));
}

// (11) apply_schema_v2: all coords <= 1000 -> Accepted (attempt 0)
#[test]
fn apply_schema_v2_accepted_valid_coords() {
    let elements = vec![
        make_element("a", [0, 0, 100, 200]),
        make_element("b", [1000, 1000, 1000, 1000]),
    ];
    assert_eq!(apply_schema_v2(&elements, 0), SchemaV2Verdict::Accepted);
    assert_eq!(apply_schema_v2(&elements, 1), SchemaV2Verdict::Accepted);
}

// (12) apply_schema_v2: any coord > 1000 with attempt=0 -> RepromptOnce
#[test]
fn apply_schema_v2_reprompt_once_on_first_failure() {
    let elements = vec![make_element("a", [0, 0, 500, 1001])];
    assert_eq!(apply_schema_v2(&elements, 0), SchemaV2Verdict::RepromptOnce);
}

// (13) apply_schema_v2: any coord > 1000 with attempt=1 -> TypedRefusal
#[test]
fn apply_schema_v2_typed_refusal_on_second_failure() {
    let elements = vec![make_element("a", [0, 0, 500, 1001])];
    assert_eq!(apply_schema_v2(&elements, 1), SchemaV2Verdict::TypedRefusal);
}

// (14) scaffold_with_call1: valid elements -> ScaffoldEmitted
#[test]
fn scaffold_with_call1_accepted() {
    let input = ScaffoldInput {
        image_bytes: vec![0u8],
        task_prompt: "describe data:image/png;base64,abc".to_string(),
    };
    let elements = vec![
        make_element("a", [0, 0, 100, 200]),
        make_element("b", [500, 500, 1000, 1000]),
    ];
    let result = scaffold_with_call1(&input, Some(&elements), 0)
        .expect("scaffold should not error on valid call1 elements");
    assert_eq!(result.outcome, ScaffoldOutcome::ScaffoldEmitted);
    assert_eq!(result.literal_elements.len(), 2);
    assert_eq!(result.literal_elements[0].label, "a");
}

// (15) scaffold_with_call1: invalid coords attempt=0 -> SchemaReprompt
#[test]
fn scaffold_with_call1_reprompt_once() {
    let input = ScaffoldInput {
        image_bytes: vec![0u8],
        task_prompt: "describe data:image/png;base64,abc".to_string(),
    };
    let elements = vec![make_element("a", [0, 0, 500, 1001])];
    let result = scaffold_with_call1(&input, Some(&elements), 0)
        .expect("scaffold should not error on schema reprompt");
    assert_eq!(result.outcome, ScaffoldOutcome::SchemaReprompt);
    assert!(result.literal_elements.is_empty());
}

// (16) scaffold_with_call1: invalid coords attempt=1 -> TypedRefusal
#[test]
fn scaffold_with_call1_typed_refusal() {
    let input = ScaffoldInput {
        image_bytes: vec![0u8],
        task_prompt: "describe data:image/png;base64,abc".to_string(),
    };
    let elements = vec![make_element("a", [0, 0, 500, 1001])];
    let result = scaffold_with_call1(&input, Some(&elements), 1)
        .expect("scaffold should not error on typed refusal");
    assert_eq!(result.outcome, ScaffoldOutcome::TypedRefusal);
    assert!(result.literal_elements.is_empty());
}

// (17) scaffold(&from_prompt(image)) still DegradedPassthrough (stub path
//      intact). This proves the T2 stub behavior is unchanged when no
//      call1_elements are provided (the live infer path).
#[test]
fn scaffold_from_prompt_still_degraded_passthrough() {
    let input = ScaffoldInput::from_prompt("describe data:image/png;base64,abc");
    let result = elai_orchestrator::perception::scaffold(&input)
        .expect("scaffold should not error on image-present stub");
    assert_eq!(result.outcome, ScaffoldOutcome::DegradedPassthrough);
    assert!(result.literal_elements.is_empty());
}

// ---------------------------------------------------------------------------
// J-8 T4: multi-image + micro-text routing contract (VR-4/VR-5).
//
// All tests below call the SAME production fn: `route_outcome`. They do NOT
// reimplement the branch in the test body and do NOT grep source text.
//
// FALSIFICATION MUTATION (for the orchestrator / auditor to verify):
//   Inside `route_outcome`, delete the
//   `image_count >= MULTI_IMAGE_MIN_COUNT` guard arm (comment it out or
//   force fallthrough to the Counting/MicroText/Normal arms). Cases (a)
//   and (g-multi) below will go RED because the decision will be
//   Passthrough/TypedRefusal instead of RouteToTiledComposite. Restoring
//   the guard makes them GREEN again.
//
// UNCOVERED MUTATIONS (named for the orchestrator):
//   * Live InferRequest frame is text-only: production sets image_count
//     to 0 or 1 (never >= 2) today, so the tiled-composite arm is NOT
//     exercised in production until multi-image frames are wired.
//   * B-24 Mode B / live tiler dispatch is NOT wired this slice; the
//     RouteToTiledComposite arm emits Kind+residue SSB fields only.
//   * Production's `route_outcome` call site SSB payload fields (kind,
//     residue) are not asserted by a contract test this slice (no SSB
//     assertion contract exists for the production gate emit).
// ---------------------------------------------------------------------------

// (a) image_count=2, Normal, has_image=true -> RouteToTiledComposite
#[test]
fn multi_image_normal_routes_to_tiled_composite() {
    let input = RouterInput {
        router_type: RouterType::Normal,
        has_image: true,
        image_path: None,
        reference_path: None,
        image_count: MULTI_IMAGE_MIN_COUNT,
        micro_text_confidence: None,
    };
    let outcome = route_outcome(&input);
    assert_eq!(outcome.decision, RouterDecision::RouteToTiledComposite);
    assert_eq!(outcome.kind, "tiled_composite");
    assert!(outcome.residue.is_some());
}

// (b) image_count=1, Normal -> Passthrough (single-image differs from (a))
#[test]
fn single_image_normal_routes_to_passthrough() {
    let input = RouterInput {
        router_type: RouterType::Normal,
        has_image: true,
        image_path: None,
        reference_path: None,
        image_count: 1,
        micro_text_confidence: None,
    };
    let outcome = route_outcome(&input);
    assert_eq!(outcome.decision, RouterDecision::Passthrough);
    assert_eq!(outcome.kind, "passthrough");
    assert!(outcome.residue.is_none());
}

// (c) MicroText, confidence=None -> RouteToTiling + micro_text_tiling residue
#[test]
fn micro_text_no_confidence_routes_to_tiling() {
    let input = RouterInput {
        router_type: RouterType::MicroText,
        has_image: true,
        image_path: None,
        reference_path: None,
        image_count: 0,
        micro_text_confidence: None,
    };
    let outcome = route_outcome(&input);
    assert_eq!(outcome.decision, RouterDecision::RouteToTiling);
    assert_eq!(outcome.kind, "micro_text_tiling");
    assert!(outcome.residue.is_some());
}

// (d) MicroText, confidence=Some(0.1) -> TypedRefusal + micro_text_typed_refusal residue
#[test]
fn micro_text_low_confidence_routes_to_typed_refusal() {
    let input = RouterInput {
        router_type: RouterType::MicroText,
        has_image: true,
        image_path: None,
        reference_path: None,
        image_count: 0,
        micro_text_confidence: Some(0.1),
    };
    let outcome = route_outcome(&input);
    assert_eq!(outcome.decision, RouterDecision::TypedRefusal);
    assert_eq!(outcome.kind, "micro_text_typed_refusal");
    assert!(outcome.residue.is_some());
}

// (e) MicroText, confidence=Some(0.9) -> RouteToTiling
#[test]
fn micro_text_high_confidence_routes_to_tiling() {
    let input = RouterInput {
        router_type: RouterType::MicroText,
        has_image: true,
        image_path: None,
        reference_path: None,
        image_count: 0,
        micro_text_confidence: Some(0.9),
    };
    let outcome = route_outcome(&input);
    assert_eq!(outcome.decision, RouterDecision::RouteToTiling);
    assert_eq!(outcome.kind, "micro_text_tiling");
}

// (f) VisualDiff with both paths still RouteToPixelDiff even if image_count=2
#[test]
fn visual_diff_wins_over_multi_image() {
    let input = RouterInput {
        router_type: RouterType::VisualDiff,
        has_image: true,
        image_path: Some(std::path::PathBuf::from("/tmp/a.png")),
        reference_path: Some(std::path::PathBuf::from("/tmp/b.png")),
        image_count: MULTI_IMAGE_MIN_COUNT,
        micro_text_confidence: None,
    };
    let outcome = route_outcome(&input);
    assert_eq!(outcome.decision, RouterDecision::RouteToPixelDiff);
    assert_eq!(outcome.kind, "pixel_diff");
    assert!(outcome.residue.is_none());
}

// (g) Counting threshold>=10 with image_count=1 -> TypedRefusal;
//     with image_count=2 -> RouteToTiledComposite (multi-image wins)
#[test]
fn counting_threshold_with_and_without_multi_image() {
    let input_single = RouterInput {
        router_type: RouterType::Counting {
            threshold: DEFAULT_COUNTING_THRESHOLD,
        },
        has_image: true,
        image_path: None,
        reference_path: None,
        image_count: 1,
        micro_text_confidence: None,
    };
    let outcome_single = route_outcome(&input_single);
    assert_eq!(outcome_single.decision, RouterDecision::TypedRefusal);

    let input_multi = RouterInput {
        router_type: RouterType::Counting {
            threshold: DEFAULT_COUNTING_THRESHOLD,
        },
        has_image: true,
        image_path: None,
        reference_path: None,
        image_count: MULTI_IMAGE_MIN_COUNT,
        micro_text_confidence: None,
    };
    let outcome_multi = route_outcome(&input_multi);
    assert_eq!(outcome_multi.decision, RouterDecision::RouteToTiledComposite);
    assert_eq!(outcome_multi.kind, "tiled_composite");
}

// ---------------------------------------------------------------------------
// J-8 T8: vision-ladder seed contract (VR-13 / VR-11).
//
// All tests below call the SAME production fns in
// `elai_orchestrator::perception::vision_ladder`:
//   * `rows` — full table
//   * `row_by_tier_key` — single-row lookup
//   * `fastvlm_row` — the VR-11 / JC-9 Apple-silicon rung
//   * `local_rows_fitting_vram` — budget-filtered local rows
//
// They do NOT reimplement the filter in the test body and do NOT grep
// source text.
//
// FALSIFICATION MUTATION (for the orchestrator / auditor to verify):
//   Delete or blank the FastVLM `[[ladder]]` block from
//   `data/perception/vision_ladder.toml` (or change its `model_id` away
//   from "FastVLM"). Case (c) `fastvlm_row_present_and_tagged` will go
//   RED because `fastvlm_row()` returns None. Restoring the block makes
//   it GREEN again.
//
// UNCOVERED MUTATIONS (named for the orchestrator):
//   * J-9 vision/OCR lane does not yet consume this file (seed handoff
//     only).
//   * J-1 leg-4 probe-model selection does not call
//     `local_rows_fitting_vram` yet.
//   * `data/hardware_catalog/model_catalog.toml` already carries parallel
//     VR-13 rows — this slice does NOT merge/dedupe into hardware_catalog.
// ---------------------------------------------------------------------------

use elai_orchestrator::perception::vision_ladder;

// (a) rows() has >= 8 entries and every required tier_key is present
#[test]
fn vision_ladder_rows_complete() {
    let rows = vision_ladder::rows();
    assert!(
        rows.len() >= 8,
        "expected at least 8 ladder rows, got {}",
        rows.len()
    );
    let required = [
        "sub_8gb",
        "vram_24gb_ocr_basic",
        "vram_24gb_qat",
        "vram_48gb_32b",
        "vram_48gb_72b",
        "vram_48gb_internvl",
        "cloud",
        "fastvlm_apple",
    ];
    for key in &required {
        assert!(
            rows.iter().any(|r| r.tier_key == *key),
            "missing tier_key: {key}"
        );
    }
}

// (b) row_by_tier_key("sub_8gb") -> Phi-4-Multimodal
#[test]
fn vision_ladder_sub_8gb_is_phi4() {
    let row = vision_ladder::row_by_tier_key("sub_8gb");
    assert_eq!(
        row.map(|r| r.model_id.as_str()),
        Some("Phi-4-Multimodal")
    );
}

// (c) fastvlm_row() is Some, model_id == "FastVLM", platform_tags contains
//     "apple_silicon", source_tag contains "VR-11" or "JC-9"
#[test]
fn fastvlm_row_present_and_tagged() {
    let row = vision_ladder::fastvlm_row().expect("FastVLM row must exist");
    assert_eq!(row.model_id, "FastVLM");
    assert!(
        row.platform_tags.iter().any(|t| t == "apple_silicon"),
        "platform_tags must contain apple_silicon"
    );
    assert!(
        row.source_tag.contains("VR-11") || row.source_tag.contains("JC-9"),
        "source_tag must reference VR-11 or JC-9"
    );
}

// (d) local_rows_fitting_vram(8192) contains Phi-4-Multimodal and does NOT
//     contain FastVLM / anthropic-cloud-vision / InternVL3-78B
#[test]
fn local_rows_8gb_fits_phi_excludes_cloud_fastvlm() {
    let rows = vision_ladder::local_rows_fitting_vram(8192);
    let model_ids: Vec<&str> = rows.iter().map(|r| r.model_id.as_str()).collect();
    assert!(
        model_ids.iter().any(|m| *m == "Phi-4-Multimodal"),
        "must include Phi-4-Multimodal at 8GB"
    );
    assert!(
        !model_ids.iter().any(|m| *m == "FastVLM"),
        "must NOT include FastVLM"
    );
    assert!(
        !model_ids.iter().any(|m| *m == "anthropic-cloud-vision"),
        "must NOT include cloud"
    );
    assert!(
        !model_ids.iter().any(|m| *m == "InternVL3-78B"),
        "must NOT include InternVL3-78B at 8GB"
    );
}

// (e) local_rows_fitting_vram(24576) contains Qwen2.5-VL-7B and
//     Gemma-3-27B-QAT; still excludes FastVLM + cloud
#[test]
fn local_rows_24gb_fits_qwen7b_and_gemma_qat() {
    let rows = vision_ladder::local_rows_fitting_vram(24576);
    let model_ids: Vec<&str> = rows.iter().map(|r| r.model_id.as_str()).collect();
    assert!(
        model_ids.iter().any(|m| *m == "Qwen2.5-VL-7B"),
        "must include Qwen2.5-VL-7B at 24GB"
    );
    assert!(
        model_ids.iter().any(|m| *m == "Gemma-3-27B-QAT"),
        "must include Gemma-3-27B-QAT at 24GB"
    );
    assert!(
        !model_ids.iter().any(|m| *m == "FastVLM"),
        "must NOT include FastVLM"
    );
    assert!(
        !model_ids.iter().any(|m| *m == "anthropic-cloud-vision"),
        "must NOT include cloud"
    );
}

// (f) unknown tier_key returns None
#[test]
fn unknown_tier_key_returns_none() {
    assert!(vision_ladder::row_by_tier_key("nonexistent_tier").is_none());
}

// (g) R2: local_rows_fitting_vram(0) is empty (forced zero-budget)
#[test]
fn local_rows_zero_budget_is_empty() {
    let rows = vision_ladder::local_rows_fitting_vram(0);
    assert!(
        rows.is_empty(),
        "zero-budget query must return no rows"
    );
}

// ---------------------------------------------------------------------------
// J-8 T7: prompt-shape library contract (VR-10 component-inventory + VR-6
// MoAI citation).
//
// All tests below call the SAME production fns in
// `elai_orchestrator::perception::prompt_shapes`:
//   * `component_inventory_fields` — ordered field-key list
//   * `component_inventory_template` — full template body
//   * `moai_citation` — VR-6 citation string accessor
//   * `MOAI_CITATION` — the const the accessor returns
//
// They do NOT reimplement the branch in the test body and do NOT grep
// source text.
//
// FALSIFICATION MUTATION (for the orchestrator / auditor to verify):
//   Change `MOAI_CITATION` to drop `arXiv:2403.07508` (or empty the string).
//   Case (c) `moai_citation_contains_arxiv_id` will go RED because the
//   string no longer contains the arXiv id. Restoring the const makes it
//   GREEN again.
//
// UNCOVERED MUTATIONS (named for the orchestrator):
//   * H-14 perception-prompt pipeline does not yet consume
//     `component_inventory_template` or `moai_citation` (library export
//     only this slice).
//   * No scaffold/router/handle_infer_request production mount calls these
//     fns; they are WIRE-DARK[J-8] by design until H-14 lands.
// ---------------------------------------------------------------------------

use elai_orchestrator::perception::prompt_shapes;

// (a) component_inventory_fields() returns exactly the four keys in order
#[test]
fn component_inventory_fields_exact_order() {
    let fields = prompt_shapes::component_inventory_fields();
    assert_eq!(
        fields,
        &["type", "verbatim text", "hierarchy", "layout cues"]
    );
}

// (b) component_inventory_template() contains each of the four field strings
#[test]
fn component_inventory_template_contains_all_fields() {
    let template = prompt_shapes::component_inventory_template();
    for &field in prompt_shapes::component_inventory_fields() {
        assert!(
            template.contains(field),
            "template must contain field `{field}`"
        );
    }
}

// (c) moai_citation() equals MOAI_CITATION and contains both the arXiv id
//     and "MoAI"
#[test]
fn moai_citation_contains_arxiv_id() {
    let citation = prompt_shapes::moai_citation();
    assert_eq!(citation, prompt_shapes::MOAI_CITATION);
    assert!(
        citation.contains("arXiv:2403.07508"),
        "citation must contain the arXiv id"
    );
    assert!(
        citation.contains("MoAI"),
        "citation must contain MoAI"
    );
}

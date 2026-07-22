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

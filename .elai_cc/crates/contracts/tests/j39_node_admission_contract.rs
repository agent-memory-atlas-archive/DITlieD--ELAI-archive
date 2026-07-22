// J-39 T1 contract: admission_ceiling_threshold + splitter wiring.
// Exercises the SAME production fn wired at
//   phase_sizing_splitter.rs estimate_complexity (threshold site)
//   phase_sizing_splitter.rs split Gate 2 (budget == DEFAULT_TOKEN_BUDGET path)
// No reimplemented threshold math in the test body.

use elai_orchestrator::admission_ceiling_threshold;
use elai_orchestrator::context_policy::effective_context_window;
use elai_orchestrator::BackendMode;
use elai_orchestrator::phase_sizing_splitter::{
    DefaultPhaseSizingSplitter, SplitTarget, TaskSplitter, DEFAULT_TOKEN_BUDGET,
};
use elai_orchestrator::plan_skeleton::ContractRef;

fn cref(name: &str) -> ContractRef {
    ContractRef {
        name: name.to_owned(),
        type_signature: "fn() -> ()".to_owned(),
        invariants: vec![],
    }
}

#[test]
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn admission_ceiling_matches_effective_window_formula() {
    let ratio = 0.6_f32;
    let ceiling = effective_context_window(BackendMode::Local);
    let expected = if ceiling == 0 {
        DEFAULT_TOKEN_BUDGET
    } else {
        ((ceiling as f32) * ratio) as u32
    };
    assert_eq!(admission_ceiling_threshold(ratio), expected);
}

#[test]
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn estimate_complexity_uses_admission_ceiling() {
    let splitter = DefaultPhaseSizingSplitter::default();
    let live = admission_ceiling_threshold(splitter.budget_ratio);
    let legacy = ((DEFAULT_TOKEN_BUDGET as f32) * splitter.budget_ratio) as u32;
    assert!(
        live > legacy,
        "test environment meaningless: live ceiling ({}) must exceed legacy ({})",
        live,
        legacy
    );
    let estimated_tokens = legacy + 1;
    assert!(
        estimated_tokens < live,
        "test token count ({}) must sit strictly below live ceiling ({}) so the admission gate is the discriminating factor",
        estimated_tokens,
        live
    );
    let target = SplitTarget {
        step_id: "j39-contract".into(),
        summary: "contract probe".into(),
        estimated_tokens,
        input_contracts: vec![cref("i")],
        output_contracts: vec![cref("o")],
        current_depth: 0,
    };
    let est = splitter.estimate_complexity(&target);
    assert!(
        !est.should_split,
        "should_split must be false: tokens ({}) are below the admission ceiling ({}) but above legacy ({}); a true verdict means the production path is using the legacy threshold, not the admission ceiling",
        est.token_estimate,
        live,
        legacy
    );
}

#[test]
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn split_gate2_uses_admission_ceiling_on_default_budget() {
    let splitter = DefaultPhaseSizingSplitter::default();
    let live = admission_ceiling_threshold(splitter.budget_ratio);
    let legacy = ((DEFAULT_TOKEN_BUDGET as f32) * splitter.budget_ratio) as u32;
    assert!(
        live > legacy,
        "test environment meaningless: live ceiling ({}) must exceed legacy ({})",
        live,
        legacy
    );
    let estimated_tokens = legacy + 1;
    assert!(
        estimated_tokens < live,
        "test token count ({}) must sit strictly below live ceiling ({}) so the admission gate is the discriminating factor",
        estimated_tokens,
        live
    );
    let target = SplitTarget {
        step_id: "j39-split".into(),
        summary: "split gate probe".into(),
        estimated_tokens,
        input_contracts: vec![cref("i")],
        output_contracts: vec![cref("o")],
        current_depth: 0,
    };
    let subs = splitter.split(&target, DEFAULT_TOKEN_BUDGET);
    assert!(
        subs.is_empty(),
        "split must return empty: tokens ({}) are below the admission ceiling ({}) but above legacy ({}); non-empty split means the production path is using the legacy threshold, not the admission ceiling",
        estimated_tokens,
        live,
        legacy
    );
}

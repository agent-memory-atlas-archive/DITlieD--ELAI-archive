//! Batch FUL-B — PhaseSizing-aware SPLIT_TASK splitter contract.
//!
//! Pins the behaviour of [`elai_orchestrator::phase_sizing_splitter`] so the
//! 4E PhaseSizing-style split heuristic stays executable. Every contract below
//! mirrors a bullet from the batch-ful-b prompt §6.
//!
//! Sources:
//!  * docs/specs/guide/steps/phase-index-3A-4J.txt §Phase 4E (Stage 3.5).
//!  * docs/specs/guide/steps/phase-4G.txt §SPLIT_TASK — max 4 sub-tasks,
//!    max depth 2, split at contract boundaries, sequential merge.
//!  * crates/orchestrator/src/pipeline_builder.rs:L46 — SPLIT_RATIO = 0.6.

use std::sync::Arc;

use elai_orchestrator::phase_sizing_splitter::{
    ComplexityEstimate, DefaultPhaseSizingSplitter, SplitTarget, SubTaskSpec, TaskSplitter,
};
use elai_orchestrator::ContractRef;

// ---------------------------------------------------------------------------
// Fixture helpers
// ---------------------------------------------------------------------------

fn cref(name: &str, sig: &str) -> ContractRef {
    ContractRef {
        name: name.to_owned(),
        type_signature: sig.to_owned(),
        invariants: vec![],
    }
}

fn small_target() -> SplitTarget {
    SplitTarget {
        step_id: "baby_step_1".to_owned(),
        summary: "tiny step".to_owned(),
        estimated_tokens: 200,
        input_contracts: vec![cref("In", "fn() -> ()")],
        output_contracts: vec![cref("Out", "fn() -> ()")],
        current_depth: 0,
    }
}

fn large_target_with_contracts(contract_pairs: usize) -> SplitTarget {
    let mut inputs = Vec::with_capacity(contract_pairs);
    let mut outputs = Vec::with_capacity(contract_pairs);
    for i in 0..contract_pairs {
        inputs.push(cref(&format!("in_{i}"), "fn() -> In"));
        outputs.push(cref(&format!("out_{i}"), "fn() -> Out"));
    }
    SplitTarget {
        step_id: "baby_step_big".to_owned(),
        // Simulate a heavy step that pushes estimated tokens well above
        // the default budget (4096) so `should_split` fires.
        summary: "big step body ".repeat(400),
        estimated_tokens: 12_000,
        input_contracts: inputs,
        output_contracts: outputs,
        current_depth: 0,
    }
}

// ---------------------------------------------------------------------------
// Contract 1 — TaskSplitter trait is dyn-safe.
// ---------------------------------------------------------------------------

#[test]
fn contract_task_splitter_trait_is_dyn_safe() {
    let splitter: Arc<dyn TaskSplitter> = Arc::new(DefaultPhaseSizingSplitter::default());
    let target = small_target();
    // Compiles + runs = dyn-safe. Assert the trait fns are reachable.
    let est: ComplexityEstimate = splitter.estimate_complexity(&target);
    assert!(est.token_estimate > 0);
}

// ---------------------------------------------------------------------------
// Contract 2 — low-complexity target => should_split = false, no sub-tasks.
// ---------------------------------------------------------------------------

#[test]
fn contract_low_complexity_no_split() {
    let splitter = DefaultPhaseSizingSplitter::default();
    let target = small_target();
    let est = splitter.estimate_complexity(&target);
    assert!(
        !est.should_split,
        "200-token step must not trigger a split (est={est:?})"
    );
    let subs = splitter.split(&target, 4096);
    assert!(
        subs.is_empty(),
        "split() on a low-complexity target must return empty, got {} subs",
        subs.len()
    );
}

// ---------------------------------------------------------------------------
// Contract 3 — high-complexity target splits into 2..=4 sub-tasks.
// ---------------------------------------------------------------------------

#[test]
fn contract_high_complexity_splits() {
    let splitter = DefaultPhaseSizingSplitter::default();
    let target = large_target_with_contracts(3);
    let est = splitter.estimate_complexity(&target);
    assert!(est.should_split, "large target must trigger a split");
    let subs = splitter.split(&target, 4096);
    assert!(
        (2..=4).contains(&(subs.len() as u32)),
        "expected 2..=4 sub-tasks, got {}",
        subs.len()
    );
    // Deterministic sub-task IDs follow the canonical format.
    for (idx, sub) in subs.iter().enumerate() {
        let expected = format!("{}.sub{}", target.step_id, idx);
        assert_eq!(sub.sub_task_id, expected, "sub-task id ordering broken");
        assert!(!sub.summary.is_empty(), "sub-task summary must be non-empty");
    }
}

// ---------------------------------------------------------------------------
// Contract 4 — max 4 sub-tasks enforced even when the target would ask for 5+.
// ---------------------------------------------------------------------------

#[test]
fn contract_max_four_sub_tasks_enforced() {
    let splitter = DefaultPhaseSizingSplitter::default();
    // Ten contract pairs = ten natural boundaries. Must still cap at 4.
    let target = large_target_with_contracts(10);
    let subs = splitter.split(&target, 4096);
    assert!(
        subs.len() as u32 <= 4,
        "max 4 sub-tasks, got {}",
        subs.len()
    );
}

// ---------------------------------------------------------------------------
// Contract 5 — max depth 2 enforced. Sub-tasks emitted from a depth-1 target
// carry depth 2, and a depth-2 target returns an empty split (at-the-cap).
// ---------------------------------------------------------------------------

#[test]
fn contract_max_depth_two_enforced() {
    let splitter = DefaultPhaseSizingSplitter::default();

    let mut depth_one = large_target_with_contracts(3);
    depth_one.current_depth = 1;
    let subs = splitter.split(&depth_one, 4096);
    assert!(!subs.is_empty(), "depth-1 target should still split");
    for sub in &subs {
        assert_eq!(sub.depth, 2, "sub-task depth must be parent+1 == 2");
    }

    let mut depth_two = large_target_with_contracts(3);
    depth_two.current_depth = 2;
    let subs = splitter.split(&depth_two, 4096);
    assert!(
        subs.is_empty(),
        "depth-2 target must not split further (max depth 2)"
    );
}

// ---------------------------------------------------------------------------
// Contract 6 — split at contract boundaries: every input/output ContractRef
// from the target is preserved across the sub-task union.
// ---------------------------------------------------------------------------

#[test]
fn contract_split_preserves_contracts() {
    let splitter = DefaultPhaseSizingSplitter::default();
    let target = large_target_with_contracts(4);
    let subs = splitter.split(&target, 4096);
    assert!(!subs.is_empty());

    // First sub-task inherits the original inputs; last sub-task inherits
    // the original outputs (plan_skeleton.rs split_phase style).
    let first = subs.first().expect("at least one sub");
    let last = subs.last().expect("at least one sub");
    let orig_input_names: Vec<&str> = target
        .input_contracts
        .iter()
        .map(|c| c.name.as_str())
        .collect();
    for name in &orig_input_names {
        assert!(
            first.input_contracts.iter().any(|c| c.name == *name),
            "first sub-task missing original input {name}"
        );
    }
    let orig_output_names: Vec<&str> = target
        .output_contracts
        .iter()
        .map(|c| c.name.as_str())
        .collect();
    for name in &orig_output_names {
        assert!(
            last.output_contracts.iter().any(|c| c.name == *name),
            "last sub-task missing original output {name}"
        );
    }
}

// ---------------------------------------------------------------------------
// Contract 7 — SubTaskSpec carries human-readable summary + id/depth.
// ---------------------------------------------------------------------------

#[test]
fn contract_sub_task_spec_fields_populated() {
    let splitter = DefaultPhaseSizingSplitter::default();
    let target = large_target_with_contracts(2);
    let subs = splitter.split(&target, 4096);
    for (idx, sub) in subs.iter().enumerate() {
        let _typed: &SubTaskSpec = sub;
        assert!(sub.sub_task_id.starts_with(&target.step_id));
        assert_eq!(sub.depth, target.current_depth + 1);
        assert!(!sub.summary.is_empty(), "sub {idx} summary empty");
    }
}

// ---------------------------------------------------------------------------
// Contract 8 — TaskSplitPayload backward-compat: new fields default via
// `#[serde(default)]`, so payloads serialized before batch-ful-b deserialize
// with `None`.
// ---------------------------------------------------------------------------

#[test]
fn contract_task_split_payload_backward_compat_deserialize() {
    use elai_orchestrator::TaskSplitPayload;
    let legacy_json = r#"{
        "original_task_id": "step-1",
        "sub_task_ids": ["step-1.sub0", "step-1.sub1"],
        "depth": 1
    }"#;
    let payload: TaskSplitPayload =
        serde_json::from_str(legacy_json).expect("legacy JSON must still deserialize");
    assert_eq!(payload.original_task_id, "step-1");
    assert_eq!(payload.sub_task_ids.len(), 2);
    assert_eq!(payload.depth, 1);
    assert!(payload.sub_task_summaries.is_none());
    assert!(payload.splitter.is_none());
    assert!(payload.complexity_estimate.is_none());
}

// ---------------------------------------------------------------------------
// Contract 9 — new TaskSplitPayload fields round-trip via serde.
// ---------------------------------------------------------------------------

#[test]
fn contract_task_split_payload_new_fields_roundtrip() {
    use elai_orchestrator::TaskSplitPayload;
    let payload = TaskSplitPayload {
        original_task_id: "baby_step_7".into(),
        sub_task_ids: vec!["baby_step_7.sub0".into(), "baby_step_7.sub1".into()],
        depth: 2,
        sub_task_summaries: Some(vec!["load inputs".into(), "emit outputs".into()]),
        splitter: Some("phase_sizing".into()),
        complexity_estimate: Some(4200),
        phase_id: Some(7),
        complexity_reason: Some("token_estimate=8192 > budget*0.6=2457".into()),
    };
    let s = serde_json::to_string(&payload).expect("serialize ok");
    let back: TaskSplitPayload = serde_json::from_str(&s).expect("deserialize ok");
    assert_eq!(back, payload);
}

// ---------------------------------------------------------------------------
// Contract 10 — TaskRunnerConfig::new_minimal leaves the splitter slot empty
// (None), preserving Batch 3B deterministic-fallback behaviour.
// ---------------------------------------------------------------------------

#[test]
fn contract_task_runner_config_splitter_defaults_to_none() {
    let cfg = elai_orchestrator::TaskRunnerConfig::new_minimal(std::path::PathBuf::from(
        "/tmp/elai_cc_test_fulb_cursor.json",
    ));
    assert!(
        cfg.task_splitter.is_none(),
        "new_minimal must leave task_splitter unset so None = Batch 3B deterministic fallback"
    );
}

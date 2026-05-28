//! E-20 task 9 -- semantic reasoning-mode toggle.
//!
//! Asserts the pre-cloud extended-thinking gate matches the
//! difficulty x task-shape matrix from arXiv:2510.08731 "When to
//! Reason: Semantic Router for vLLM" (IBM, NeurIPS 2025; -47.1%
//! latency / -48.5% tokens / +10.2pp MMLU-Pro by NOT invoking
//! reasoning when the prompt does not benefit).

use elai_orchestrator::cost::cloud_route::semantic_reasoning::{
    choose_reasoning_mode, DifficultyBin, ReasoningMode, TaskShapeHint,
};

#[test]
fn easy_execution_skips_reasoning() {
    let mode = choose_reasoning_mode(DifficultyBin::Easy, TaskShapeHint::Execution);
    assert_eq!(mode, ReasoningMode::Off);
}

#[test]
fn easy_math_skips_reasoning() {
    let mode = choose_reasoning_mode(DifficultyBin::Easy, TaskShapeHint::Math);
    assert_eq!(mode, ReasoningMode::Off);
}

#[test]
fn easy_architecture_skips_reasoning() {
    let mode = choose_reasoning_mode(DifficultyBin::Easy, TaskShapeHint::Architecture);
    assert_eq!(mode, ReasoningMode::Off);
}

#[test]
fn medium_execution_skips_reasoning() {
    let mode = choose_reasoning_mode(DifficultyBin::Medium, TaskShapeHint::Execution);
    assert_eq!(mode, ReasoningMode::Off);
}

#[test]
fn medium_architecture_invokes_reasoning() {
    let mode = choose_reasoning_mode(DifficultyBin::Medium, TaskShapeHint::Architecture);
    assert_eq!(mode, ReasoningMode::On);
}

#[test]
fn medium_math_invokes_reasoning() {
    let mode = choose_reasoning_mode(DifficultyBin::Medium, TaskShapeHint::Math);
    assert_eq!(mode, ReasoningMode::On);
}

#[test]
fn hard_always_invokes_reasoning() {
    for shape in [
        TaskShapeHint::Execution,
        TaskShapeHint::Architecture,
        TaskShapeHint::Math,
    ] {
        assert_eq!(
            choose_reasoning_mode(DifficultyBin::Hard, shape),
            ReasoningMode::On,
            "Hard / {shape:?} must invoke reasoning"
        );
    }
}

#[test]
fn reasoning_mode_labels_stable() {
    assert_eq!(ReasoningMode::On.as_str(), "on");
    assert_eq!(ReasoningMode::Off.as_str(), "off");
}

#[test]
fn difficulty_labels_stable() {
    assert_eq!(DifficultyBin::Easy.as_str(), "easy");
    assert_eq!(DifficultyBin::Medium.as_str(), "medium");
    assert_eq!(DifficultyBin::Hard.as_str(), "hard");
}

#[test]
fn task_shape_hint_labels_stable() {
    assert_eq!(TaskShapeHint::Execution.as_str(), "execution");
    assert_eq!(TaskShapeHint::Architecture.as_str(), "architecture");
    assert_eq!(TaskShapeHint::Math.as_str(), "math");
}

//! L-11 output-quality contract: the real MiniCritic god-object metric.
//!
//! These arms drive `MiniCritic::check`; they do not reimplement the predicate.

use elai_orchestrator::BlankId;
use elai_orchestrator::mini_critic::{
    MiniCritic, MiniCriticLanguage, MiniCriticVerdict, GOD_OBJECT_MIN_BODY_LINES,
    GOD_OBJECT_MIN_DISTINCT_CALLEES, GOD_OBJECT_MIN_FUNCTION_ITEMS,
};

fn check(body: &str) -> elai_orchestrator::mini_critic::MiniCriticResult {
    MiniCritic::without_tree_sitter().check(BlankId(11), body, MiniCriticLanguage::Rust)
}

fn callees(count: usize) -> String {
    (0..count)
        .map(|i| format!("callee_{i}();"))
        .collect::<Vec<_>>()
        .join(" ")
}

#[test]
fn god_object_warns_with_metric_tag() {
    let mut body = callees(GOD_OBJECT_MIN_DISTINCT_CALLEES);
    for i in 0..GOD_OBJECT_MIN_FUNCTION_ITEMS {
        body.push_str(&format!("\nfn item_{i}() {{}}"));
    }
    let result = check(&body);
    assert_eq!(result.verdict, MiniCriticVerdict::Warn);
    assert!(result.reason.starts_with("metric_god_object:"), "{}", result.reason);
}

#[test]
fn clean_small_fill_passes() {
    let result = check("let x = 1;\nlet y = 2;\nx + y");
    assert_eq!(result.verdict, MiniCriticVerdict::Pass);
}

#[test]
fn high_callee_small_body_passes_and_guard_is_and() {
    let result = check(&callees(GOD_OBJECT_MIN_DISTINCT_CALLEES));
    assert_eq!(result.verdict, MiniCriticVerdict::Pass);
}

#[test]
fn extreme_god_object_never_blocks() {
    let mut body = callees(GOD_OBJECT_MIN_DISTINCT_CALLEES);
    for i in 0..GOD_OBJECT_MIN_FUNCTION_ITEMS {
        body.push_str(&format!("\nfn item_{i}() {{}}"));
    }
    while body.lines().count() < GOD_OBJECT_MIN_BODY_LINES {
        body.push_str("\nlet filler_value = 1;");
    }
    assert_ne!(check(&body).verdict, MiniCriticVerdict::Block);
}

#[test]
fn named_thresholds_are_positive_and_used_by_contract() {
    assert!(GOD_OBJECT_MIN_DISTINCT_CALLEES > 0);
    assert!(GOD_OBJECT_MIN_FUNCTION_ITEMS > 0);
    assert!(GOD_OBJECT_MIN_BODY_LINES > 0);
}

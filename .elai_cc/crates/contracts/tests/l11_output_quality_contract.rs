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


#[test]
fn l11_validate_rejects_over_budget_declaration()  {
    use elai_core::file_plan::{FilePlan, FilePlanError, FileSpec, MAX_DISTINCT_CONCERNS_PER_FILE};
    use std::collections::BTreeMap;
    use std::path::PathBuf;
    let plan = FilePlan {
        new_files: vec![FileSpec {
            path: PathBuf::from("src/concern-heavy.rs"), language: "rust".into(),
            max_lines: 100, max_distinct_concerns: MAX_DISTINCT_CONCERNS_PER_FILE + 1,
            functions: vec![],
        }], modified_files: vec![], version: FilePlan::CURRENT_VERSION,
    };
    assert!(matches!(plan.validate(&BTreeMap::new()),
        Err(FilePlanError::DistinctConcernBudgetExceeded { limit: MAX_DISTINCT_CONCERNS_PER_FILE, .. })));
}

#[test]
fn l11_legacy_file_plan_defaults_responsibility_budget()  {
    use elai_core::file_plan::{FilePlan, DEFAULT_MAX_DISTINCT_CONCERNS};
    use std::collections::BTreeMap;
    let plan: FilePlan = serde_json::from_str(r#"{
        "new_files":[{"path":"src/legacy.rs","language":"rust","max_lines":100,"functions":[]}],
        "modified_files":[],"version":1
    }"#).expect("legacy FilePlan JSON");
    assert_eq!(plan.new_files[0].max_distinct_concerns, DEFAULT_MAX_DISTINCT_CONCERNS);
    assert!(plan.validate(&BTreeMap::new()).is_ok());
}


#[test]
fn l11_observes_budget_without_changing_skeleton()  {
    use elai_core::file_plan::{FilePlan, FileSpec};
    use elai_orchestrator::SkeletonGenerator;
    use std::path::PathBuf;
    let plan = FilePlan {
        new_files: vec![FileSpec {
            path: PathBuf::from("src/many-concerns.rs"), language: "rust".into(),
            max_lines: 100, max_distinct_concerns: 1,
            functions: vec![elai_core::file_plan::FunctionSpec {
                name: "one".into(), signature: "fn one()".into(), max_body_lines: 10,
            }, elai_core::file_plan::FunctionSpec {
                name: "two".into(), signature: "fn two()".into(), max_body_lines: 10,
            }],
        }], modified_files: vec![], version: FilePlan::CURRENT_VERSION,
    };
    let generated = SkeletonGenerator::default().generate_from_file_plan(&plan)
        .expect("observe-only budget must not block generation");
    assert_eq!(generated.len(), 1);
    assert_eq!(generated[0].blanks.len(), plan.new_files[0].functions.len());
    assert!(generated[0].responsibility_budget_exceeded);

    let mut in_budget = plan.clone();
    in_budget.new_files[0].max_distinct_concerns = 2;
    let in_budget_generated = SkeletonGenerator::default()
        .generate_from_file_plan(&in_budget)
        .expect("in-budget observation must not block generation");
    assert_eq!(in_budget_generated.len(), 1);
    assert!(!in_budget_generated[0].responsibility_budget_exceeded);
    assert_eq!(generated[0].skeleton_text, in_budget_generated[0].skeleton_text);
}

#[test]
fn maintainability_floor_warns_overlong_candidate_without_blocking() {
    use elai_orchestrator::hook_dispatcher::HookVerdictKind;
    use elai_orchestrator::vibegate::{
        evaluate_vibegate, MAINTAINABILITY_FLOOR_CATEGORY, MAX_FILE_LINES,
    };
    let source = std::iter::repeat("let value = 1;")
        .take(MAX_FILE_LINES + 1)
        .collect::<Vec<_>>()
        .join("\n");
    let findings = evaluate_vibegate("Write", &source, "src/large.rs");
    let quality: Vec<_> = findings
        .iter()
        .filter(|finding| finding.category == MAINTAINABILITY_FLOOR_CATEGORY)
        .collect();
    assert!(quality.iter().any(|finding| finding.verdict == HookVerdictKind::Warn));
    assert!(!quality.iter().any(|finding| finding.verdict == HookVerdictKind::Block));
}

#[test]
fn maintainability_floor_clean_short_candidate_is_silent() {
    use elai_orchestrator::vibegate::{evaluate_vibegate, MAINTAINABILITY_FLOOR_CATEGORY};
    let findings = evaluate_vibegate("Write", "fn clean() -> i32 { 1 }", "src/clean.rs");
    assert!(!findings
        .iter()
        .any(|finding| finding.category == MAINTAINABILITY_FLOOR_CATEGORY));
}

#[test]
fn maintainability_floor_blocks_typescript_any_escape_hatch() {
    use elai_orchestrator::hook_dispatcher::HookVerdictKind;
    use elai_orchestrator::vibegate::{
        evaluate_vibegate, MAINTAINABILITY_FLOOR_CATEGORY,
    };
    let findings = evaluate_vibegate("Write", "const value = input as any;", "src/app.ts");
    assert!(findings.iter().any(|finding| {
        finding.category == MAINTAINABILITY_FLOOR_CATEGORY
            && finding.verdict == HookVerdictKind::Block
    }));
}

#[test]
fn maintainability_floor_warns_rust_soft_loosening() {
    use elai_orchestrator::hook_dispatcher::HookVerdictKind;
    use elai_orchestrator::vibegate::{
        evaluate_vibegate, MAINTAINABILITY_FLOOR_CATEGORY,
    };
    let findings = evaluate_vibegate("Write", "#[allow(dead_code)]\nfn helper() {}", "src/lib.rs");
    assert!(findings.iter().any(|finding| {
        finding.category == MAINTAINABILITY_FLOOR_CATEGORY
            && finding.verdict == HookVerdictKind::Warn
    }));
    assert!(!findings.iter().any(|finding| {
        finding.category == MAINTAINABILITY_FLOOR_CATEGORY
            && finding.verdict == HookVerdictKind::Block
    }));
}

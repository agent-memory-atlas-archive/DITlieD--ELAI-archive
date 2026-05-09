//! Gap #9 contract tests (MVP audit 2026-04-18 section 4 item 9):
//! Post-execution rule evaluation against real source text.
//!
//! The pre-gate path (`evaluate_pre_gate_and_emit_findings`) runs the rule
//! engine against the natural-language task text — Grep/AST rules never
//! match. After the real executor returns with a `changed_files` list, the
//! task_runner re-evaluates every changed file's on-disk contents so
//! pattern-based rules finally fire.
//!
//! This contract exercises the pure function the task_runner hook wraps
//! (`RuleEvaluator::evaluate_at_pre_gate`) with realistic post-execution
//! inputs:
//!   1. A Grep rule matches when given the file's actual content but does
//!      NOT match the original task text — proving the post-exec pass is
//!      load-bearing.
//!   2. Missing files (delete case) are skipped, not treated as violations.
//!      The hook's `path.exists()` guard makes this a no-SSB-write scenario.
//!   3. The category constant `rule_violation_post` is distinct from
//!      `rule_violation` so the frontend can filter the two surfaces
//!      independently.

use std::collections::HashMap;
use std::sync::Arc;

use elai_orchestrator::{
    BABY_STEP_PRE_GATE_ACTION, RULE_VIOLATION_CATEGORY, RULE_VIOLATION_POST_CATEGORY, RuleEvaluator,
};
use elai_rules::predicate::VerdictResult;
use elai_rules::rule_engine::{
    CompiledPredicate, DefaultRuleEngine, EvaluationContext, FsmPhaseBit, Rule, RuleEngine,
};
use elai_rules::stubs::{EmptyDkgQuerier, PassAstEvaluator};

const EXECUTING_BIT: u32 = 1u32 << 3;

fn new_engine() -> Arc<DefaultRuleEngine> {
    Arc::new(DefaultRuleEngine::new(
        Box::new(PassAstEvaluator),
        Box::new(EmptyDkgQuerier),
    ))
}

fn compile(engine: &DefaultRuleEngine, rule: Rule) -> (Rule, CompiledPredicate) {
    let compiled = engine
        .compile_predicate(&rule.predicate_json)
        .expect("compile predicate");
    (rule, compiled)
}

fn grep_rule(rule_id: &str, pattern: &str, severity: &str) -> Rule {
    Rule {
        rule_id: rule_id.to_owned(),
        domain: "engineering".to_owned(),
        tier: 1,
        severity: severity.to_owned(),
        title: "post-exec grep rule".to_owned(),
        predicate_json: format!(
            r#"{{"type":"grep_pattern","pattern":"{pattern}","scope":null,"context_lines":0}}"#
        ),
        phase_mask: EXECUTING_BIT,
        escalate_target: None,
    }
}

// ---------------------------------------------------------------------------
// Contract 1: category constant is distinct from the pre-gate category so
// the dashboard frontend can distinguish the two surfaces.
// ---------------------------------------------------------------------------

#[test]
fn post_exec_category_is_distinct_from_pre_gate_category() {
    assert_ne!(
        RULE_VIOLATION_POST_CATEGORY, RULE_VIOLATION_CATEGORY,
        "post-exec tag must differ from pre-gate tag for frontend filtering",
    );
    assert_eq!(RULE_VIOLATION_POST_CATEGORY, "rule_violation_post");
    assert_eq!(RULE_VIOLATION_CATEGORY, "rule_violation");
}

// ---------------------------------------------------------------------------
// Contract 2: the same evaluator that produced no match against the task
// text DOES match when fed the actual file content. This is the wire-4A →
// Gap-#9 correctness proof: Grep rules are inert at pre-gate (task text
// never contains `unsafe`), but fire post-exec against on-disk source.
// ---------------------------------------------------------------------------

#[test]
fn grep_rule_matches_file_content_but_not_task_text() {
    let engine = new_engine();
    let rule = compile(&engine, grep_rule("R-UNSAFE", "unsafe", "BLOCK"));
    let evaluator = RuleEvaluator::new(engine, Arc::new(vec![rule]));

    let task_text = "implement a safe wrapper around the ffi boundary";
    let file_source = "pub fn danger() { unsafe { std::ptr::null::<u8>(); } }";

    // Helper: rule-engine emits an `Allow` verdict for non-matches. The
    // task_runner hook emits SSB Findings only for Block/Warn/Escalate —
    // Allow is the no-op default. Count non-Allow verdicts to model what
    // the hook actually surfaces to the dashboard.
    let non_allow = |vs: &[elai_rules::predicate::Verdict]| -> Vec<_> {
        vs.iter()
            .filter(|v| !matches!(v.result, VerdictResult::Allow))
            .cloned()
            .collect()
    };

    // Pre-gate shape: source = task_text → rule does NOT match, so zero
    // non-Allow verdicts → zero Findings.
    let pre_ctx = EvaluationContext {
        action: BABY_STEP_PRE_GATE_ACTION,
        source: task_text,
        file_path: "",
        fsm_phase: FsmPhaseBit::EXECUTING,
        metric_values: HashMap::new(),
    };
    let pre_verdicts = evaluator.evaluate_at_pre_gate(FsmPhaseBit::EXECUTING, &[], &pre_ctx);
    let pre_non_allow = non_allow(&pre_verdicts);
    assert!(
        pre_non_allow.is_empty(),
        "pre-gate (task text) must not produce a Block/Warn verdict; got {:?}",
        pre_non_allow,
    );

    // Post-exec shape: source = file content → rule DOES match → one Block.
    let post_ctx = EvaluationContext {
        action: BABY_STEP_PRE_GATE_ACTION,
        source: file_source,
        file_path: "src/foo.rs",
        fsm_phase: FsmPhaseBit::EXECUTING,
        metric_values: HashMap::new(),
    };
    let post_verdicts =
        evaluator.evaluate_at_pre_gate(FsmPhaseBit::EXECUTING, &[], &post_ctx);
    let post_non_allow = non_allow(&post_verdicts);
    assert_eq!(
        post_non_allow.len(),
        1,
        "post-exec (file content) must produce exactly one Block verdict; got {:?}",
        post_non_allow,
    );
    assert!(matches!(post_non_allow[0].result, VerdictResult::Block));
    assert_eq!(post_non_allow[0].rule_id, "R-UNSAFE");
}

// ---------------------------------------------------------------------------
// Contract 3: deleted files (path does not exist) are skipped. This models
// the hook's `path.exists()` guard — a changed_files entry pointing at a
// removed file produces zero SSB writes.
// ---------------------------------------------------------------------------

#[test]
fn missing_file_is_skipped_not_a_violation() {
    // The hook's guard: `if !path.exists() { continue; }`. We assert the
    // invariant by constructing a path that definitely does not exist and
    // confirming that `Path::exists` returns false — the task_runner hook
    // relies on this exact semantics to skip deletes without emitting a
    // Finding.
    let missing = std::path::Path::new("/definitely/does/not/exist/foo.rs");
    assert!(
        !missing.exists(),
        "fixture path must not exist for the skip-case to be meaningful",
    );

    // Belt-and-braces: the evaluator itself never reads from disk — it
    // consumes whatever `source` string the caller supplied. So even if a
    // hypothetical bug let a missing file through the guard, an empty
    // `source` produces only Allow verdicts for grep rules (nothing to
    // match), which the hook filters out when deciding whether to emit
    // Findings.
    let engine = new_engine();
    let rule = compile(&engine, grep_rule("R-ANY", "foo", "WARN"));
    let evaluator = RuleEvaluator::new(engine, Arc::new(vec![rule]));
    let ctx = EvaluationContext {
        action: BABY_STEP_PRE_GATE_ACTION,
        source: "",
        file_path: "",
        fsm_phase: FsmPhaseBit::EXECUTING,
        metric_values: HashMap::new(),
    };
    let verdicts = evaluator.evaluate_at_pre_gate(FsmPhaseBit::EXECUTING, &[], &ctx);
    let non_allow: Vec<_> = verdicts
        .iter()
        .filter(|v| !matches!(v.result, VerdictResult::Allow))
        .collect();
    assert!(
        non_allow.is_empty(),
        "empty source must produce zero non-Allow verdicts so the hook's skip-case is observably no-op; got {:?}",
        non_allow,
    );
}

// ---------------------------------------------------------------------------
// Contract 4: the BabyStepExecutionResult carries a `changed_files` field
// so the task_runner hook has a real list to iterate. This is the
// compile-time wiring contract wire-4A → Gap-#9 relies on.
// ---------------------------------------------------------------------------

#[test]
fn baby_step_execution_result_exposes_changed_files() {
    let synthetic = elai_orchestrator::BabyStepExecutionResult::synthetic_passed();
    // The synthetic path has no diff — documented in the doc-comment on
    // `changed_files`. The field must still exist and default to empty so
    // the post-exec hook can iterate without a panic.
    assert!(synthetic.changed_files.is_empty());
}

//! Contract tests for rules/rule_engine (Phase 3F.3).
//!
//! Validates: object-safety (Box<dyn RuleEngine>), compile_predicate rejection
//! paths, evaluation ordering contract, get_active_rules filtering, and
//! Warn accumulation before Block short-circuit.

use elai_rules::predicate::{
    AstError, AstEvaluator, AstMatch, DkgError, DkgQuerier, QueryFormat, ReachabilityMatch,
    ReachabilityParams, VerdictResult,
};
use elai_rules::rule_engine::{
    CompiledPredicate, DefaultRuleEngine, EvaluationContext, FsmPhaseBit, Rule, RuleEngine,
};

// ---------------------------------------------------------------------------
// Mock evaluators (contract-level — minimal)
// ---------------------------------------------------------------------------

struct StubAstEvaluator;

impl AstEvaluator for StubAstEvaluator {
    fn evaluate_query(
        &self,
        _language: &str,
        _source: &str,
        _query: &str,
    ) -> Result<Vec<AstMatch>, AstError> {
        Ok(vec![])
    }

    fn validate_query(&self, _language: &str, _query: &str) -> Result<(), AstError> {
        Ok(())
    }
}

struct StubDkgQuerier;

impl DkgQuerier for StubDkgQuerier {
    fn query_reachability(
        &self,
        _params: &ReachabilityParams,
    ) -> Result<Vec<ReachabilityMatch>, DkgError> {
        Ok(vec![])
    }

    fn validate_params(&self, _params: &ReachabilityParams) -> Result<(), DkgError> {
        Ok(())
    }
}

/// Panicking AST evaluator — proves short-circuit.
struct PanicAstEvaluator;

impl AstEvaluator for PanicAstEvaluator {
    fn evaluate_query(
        &self,
        _language: &str,
        _source: &str,
        _query: &str,
    ) -> Result<Vec<AstMatch>, AstError> {
        panic!("AST evaluator must not be called — short-circuit violated")
    }

    fn validate_query(&self, _language: &str, _query: &str) -> Result<(), AstError> {
        panic!("AST evaluator must not be called — short-circuit violated")
    }
}

fn engine() -> DefaultRuleEngine {
    DefaultRuleEngine::new(Box::new(StubAstEvaluator), Box::new(StubDkgQuerier))
}

fn make_rule(
    id: &str,
    domain: &str,
    tier: u8,
    severity: &str,
    phase_mask: u32,
) -> Rule {
    Rule {
        rule_id: id.into(),
        domain: domain.into(),
        tier,
        severity: severity.into(),
        title: format!("Contract {id}"),
        predicate_json: String::new(),
        phase_mask,
    }
}

// ---------------------------------------------------------------------------
// 1. Object-safety: Box<dyn RuleEngine> compiles
// ---------------------------------------------------------------------------

#[test]
fn contract_dyn_rule_engine_object_safety() {
    let e = engine();
    let _boxed: Box<dyn RuleEngine> = Box::new(e);
}

// ---------------------------------------------------------------------------
// 2. compile_predicate rejects invalid regex (GrepPattern)
// ---------------------------------------------------------------------------

#[test]
fn contract_compile_rejects_invalid_regex() {
    let e = engine();
    let json = r#"{"type":"grep_pattern","pattern":"[unclosed","scope":null,"context_lines":0}"#;
    let result = e.compile_predicate(json);
    assert!(result.is_err(), "invalid regex must be rejected by compile_predicate");
}

// ---------------------------------------------------------------------------
// 3. compile_predicate rejects invalid JSON Schema (SchemaRequired)
// ---------------------------------------------------------------------------

#[test]
fn contract_compile_rejects_invalid_json_schema() {
    let e = engine();
    let json =
        r#"{"type":"schema_required","schema":{"type":"invalid_type"},"output_type":"json"}"#;
    let result = e.compile_predicate(json);
    assert!(result.is_err(), "invalid JSON schema type must be rejected");
}

// ---------------------------------------------------------------------------
// 4. compile_predicate rejects Composite depth > 8
// ---------------------------------------------------------------------------

#[test]
fn contract_compile_rejects_depth_over_8() {
    let e = engine();
    let mut json =
        r#"{"type":"grep_pattern","pattern":"x","scope":null,"context_lines":0}"#.to_string();
    for _ in 0..9 {
        json = format!(
            r#"{{"type":"composite","operator":"and","children":[{json}]}}"#
        );
    }
    let result = e.compile_predicate(&json);
    assert!(result.is_err(), "depth=9 composite must be rejected");
}

// ---------------------------------------------------------------------------
// 5. compile_predicate uses from_json (validates Not arity)
// ---------------------------------------------------------------------------

#[test]
fn contract_compile_uses_from_json_not_raw_serde() {
    let e = engine();
    let json = r#"{"type":"composite","operator":"not","children":[]}"#;
    let result = e.compile_predicate(json);
    assert!(
        result.is_err(),
        "Not with 0 children must be rejected (proves from_json is used)"
    );
}

// ---------------------------------------------------------------------------
// 6. compile_predicate accepts valid predicates
// ---------------------------------------------------------------------------

#[test]
fn contract_compile_accepts_valid_grep() {
    let e = engine();
    let json = r#"{"type":"grep_pattern","pattern":"TODO","scope":null,"context_lines":0}"#;
    assert!(e.compile_predicate(json).is_ok());
}

#[test]
fn contract_compile_accepts_valid_schema() {
    let e = engine();
    let json = r#"{"type":"schema_required","schema":{"type":"object","required":["name"]},"output_type":"json"}"#;
    assert!(e.compile_predicate(json).is_ok());
}

#[test]
fn contract_compile_accepts_valid_metric_gate() {
    let e = engine();
    let json = r#"{"type":"metric_gate","metric_name":"cc","operator":"gt","threshold":5.0,"source":"file"}"#;
    assert!(e.compile_predicate(json).is_ok());
}

// ---------------------------------------------------------------------------
// 7. get_active_rules: phase_mask filtering
// ---------------------------------------------------------------------------

#[test]
fn contract_get_active_rules_phase_mask_filtering() {
    let e = engine();
    let rules = vec![
        make_rule("r-plan", "engineering", 1, "BLOCK", 0b0000_0001), // PLANNING only
        make_rule("r-exec", "engineering", 1, "BLOCK", 0b0000_1000), // EXECUTING only
        make_rule("r-all", "engineering", 1, "BLOCK", 0b1111_1111),  // all
    ];

    let active = e.get_active_rules(&rules, FsmPhaseBit::PLANNING, &[]);
    let ids: Vec<&str> = active.iter().map(|r| r.rule_id.as_str()).collect();
    assert!(ids.contains(&"r-plan"));
    assert!(!ids.contains(&"r-exec"));
    assert!(ids.contains(&"r-all"));
}

// ---------------------------------------------------------------------------
// 8. get_active_rules: engineering/security always included
// ---------------------------------------------------------------------------

#[test]
fn contract_get_active_rules_always_includes_eng_sec() {
    let e = engine();
    let rules = vec![
        make_rule("r-eng", "engineering", 1, "BLOCK", 0xFF),
        make_rule("r-sec", "security", 1, "BLOCK", 0xFF),
        make_rule("r-other", "gaming", 1, "BLOCK", 0xFF),
    ];

    // file_domains has no engineering/security
    let active = e.get_active_rules(&rules, FsmPhaseBit::EXECUTING, &["trading"]);
    let ids: Vec<&str> = active.iter().map(|r| r.rule_id.as_str()).collect();
    assert!(ids.contains(&"r-eng"), "engineering must always be included");
    assert!(ids.contains(&"r-sec"), "security must always be included");
    assert!(!ids.contains(&"r-other"), "gaming not in file_domains");
}

// ---------------------------------------------------------------------------
// 9. get_active_rules: ordering (tier ASC, severity DESC)
// ---------------------------------------------------------------------------

#[test]
fn contract_get_active_rules_ordering() {
    let e = engine();
    let rules = vec![
        make_rule("r-t2-warn", "engineering", 2, "WARN", 0xFF),
        make_rule("r-t1-warn", "engineering", 1, "WARN", 0xFF),
        make_rule("r-t1-block", "engineering", 1, "BLOCK", 0xFF),
    ];

    let active = e.get_active_rules(&rules, FsmPhaseBit::EXECUTING, &[]);
    let ids: Vec<&str> = active.iter().map(|r| r.rule_id.as_str()).collect();
    assert_eq!(ids, vec!["r-t1-block", "r-t1-warn", "r-t2-warn"]);
}

// ---------------------------------------------------------------------------
// 10. evaluate: short-circuit on Block (panicking AST evaluator)
// ---------------------------------------------------------------------------

#[test]
fn contract_evaluate_short_circuits_block_before_ast() {
    let e = DefaultRuleEngine::new(Box::new(PanicAstEvaluator), Box::new(StubDkgQuerier));

    let grep_json = r#"{"type":"grep_pattern","pattern":"DANGER","scope":null,"context_lines":0}"#;
    let grep_compiled = e.compile_predicate(grep_json).unwrap();

    // Manually build AST compiled predicate (can't compile with panicking evaluator).
    let ast_compiled = CompiledPredicate::AstPattern {
        language: "python".into(),
        query: "(function_definition)".into(),
        query_format: QueryFormat::Sexp,
        scope: None,
    };

    let grep_rule = Rule {
        rule_id: "r-grep".into(),
        domain: "security".into(),
        tier: 1,
        severity: "BLOCK".into(),
        title: "Grep block".into(),
        predicate_json: grep_json.into(),
        phase_mask: 0xFF,
    };
    let ast_rule = Rule {
        rule_id: "r-ast".into(),
        domain: "security".into(),
        tier: 1,
        severity: "BLOCK".into(),
        title: "AST block".into(),
        predicate_json: String::new(),
        phase_mask: 0xFF,
    };

    let rules_with_compiled = vec![(grep_rule, grep_compiled), (ast_rule, ast_compiled)];

    let context = EvaluationContext {
        action: "write",
        source: "DANGER code",
        file_path: "src/main.py",
        fsm_phase: FsmPhaseBit::EXECUTING,
    };

    // If this panics, short-circuit is broken.
    let result = e.evaluate(&rules_with_compiled, &context).unwrap();
    let has_block = result.iter().any(|v| v.result == VerdictResult::Block);
    assert!(has_block, "must produce Block verdict");
}

// ---------------------------------------------------------------------------
// 11. evaluate: Warn accumulation before Block
// ---------------------------------------------------------------------------

#[test]
fn contract_evaluate_accumulates_warns_then_blocks() {
    let e = engine();

    let warn_json = r#"{"type":"grep_pattern","pattern":"TODO","scope":null,"context_lines":0}"#;
    let warn_compiled = e.compile_predicate(warn_json).unwrap();

    let block_json = r#"{"type":"grep_pattern","pattern":"FIXME","scope":null,"context_lines":0}"#;
    let block_compiled = e.compile_predicate(block_json).unwrap();

    let rules_with_compiled = vec![
        (
            Rule {
                rule_id: "r-warn".into(),
                domain: "engineering".into(),
                tier: 1,
                severity: "WARN".into(),
                title: "Warn rule".into(),
                predicate_json: warn_json.into(),
                phase_mask: 0xFF,
            },
            warn_compiled,
        ),
        (
            Rule {
                rule_id: "r-block".into(),
                domain: "engineering".into(),
                tier: 2,
                severity: "BLOCK".into(),
                title: "Block rule".into(),
                predicate_json: block_json.into(),
                phase_mask: 0xFF,
            },
            block_compiled,
        ),
    ];

    let context = EvaluationContext {
        action: "write",
        source: "TODO: fix\nFIXME: urgent",
        file_path: "src/main.rs",
        fsm_phase: FsmPhaseBit::EXECUTING,
    };

    let result = e.evaluate(&rules_with_compiled, &context).unwrap();
    let warns = result
        .iter()
        .filter(|v| v.result == VerdictResult::Warn)
        .count();
    let blocks = result
        .iter()
        .filter(|v| v.result == VerdictResult::Block)
        .count();
    assert!(warns >= 1, "must accumulate warns before block");
    assert_eq!(blocks, 1, "must have exactly 1 block");
}

// ---------------------------------------------------------------------------
// 12. FsmPhaseBit: bit ordering contract
// ---------------------------------------------------------------------------

#[test]
fn contract_fsm_phase_bit_ordering() {
    assert_eq!(FsmPhaseBit::PLANNING.bit(), 0);
    assert_eq!(FsmPhaseBit::FEASIBILITY_PROBE.bit(), 1);
    assert_eq!(FsmPhaseBit::TESTING_FIRST.bit(), 2);
    assert_eq!(FsmPhaseBit::EXECUTING.bit(), 3);
    assert_eq!(FsmPhaseBit::AUDITING.bit(), 4);
    assert_eq!(FsmPhaseBit::MEMORIZING.bit(), 5);
    assert_eq!(FsmPhaseBit::CONFIGURING.bit(), 6);
}

#[test]
fn contract_fsm_phase_bit_mask_values() {
    assert_eq!(FsmPhaseBit::PLANNING.mask(), 1);
    assert_eq!(FsmPhaseBit::EXECUTING.mask(), 8);
    assert_eq!(FsmPhaseBit::CONFIGURING.mask(), 64);
}

#[test]
fn contract_fsm_phase_bit_from_raw_rejects_invalid() {
    assert!(FsmPhaseBit::from_raw(7).is_none());
    assert!(FsmPhaseBit::from_raw(255).is_none());
}

// ---------------------------------------------------------------------------
// 13. RuleEngineError variants exist
// ---------------------------------------------------------------------------

#[test]
fn contract_rule_engine_error_display() {
    use elai_rules::rule_engine::RuleEngineError;
    use elai_rules::predicate::PredicateError;

    let err = RuleEngineError::Predicate(PredicateError::InvalidJson("bad".into()));
    let msg = err.to_string();
    assert!(msg.contains("bad"), "error must propagate inner message");
}

// ---------------------------------------------------------------------------
// 14. SchemaRequired on non-JSON source does NOT abort pipeline
// ---------------------------------------------------------------------------

#[test]
fn contract_schema_on_non_json_does_not_abort() {
    let e = engine();
    let schema_json =
        r#"{"type":"schema_required","schema":{"type":"object"},"output_type":"json"}"#;
    let schema_compiled = e.compile_predicate(schema_json).unwrap();

    let grep_json =
        r#"{"type":"grep_pattern","pattern":"DANGER","scope":null,"context_lines":0}"#;
    let grep_compiled = e.compile_predicate(grep_json).unwrap();

    let rules_with_compiled = vec![
        (make_rule("r-schema", "engineering", 1, "BLOCK", 0xFF), schema_compiled),
        (make_rule("r-grep", "security", 2, "BLOCK", 0xFF), grep_compiled),
    ];

    let context = EvaluationContext {
        action: "write",
        source: "fn main() { DANGER }",
        file_path: "src/main.rs",
        fsm_phase: FsmPhaseBit::EXECUTING,
    };

    let result = e.evaluate(&rules_with_compiled, &context).unwrap();
    let grep_block = result
        .iter()
        .any(|v| v.result == VerdictResult::Block && v.rule_id == "r-grep");
    assert!(
        grep_block,
        "grep rule must still execute after schema rule on non-JSON source"
    );
}

// ---------------------------------------------------------------------------
// 15. And composite fires when all children produce non-Allow
// ---------------------------------------------------------------------------

#[test]
fn contract_and_composite_fires_on_all_children_match() {
    use elai_rules::predicate::CompositeOperator;

    let e = engine();
    let c1 = e
        .compile_predicate(
            r#"{"type":"grep_pattern","pattern":"TODO","scope":null,"context_lines":0}"#,
        )
        .unwrap();
    let c2 = e
        .compile_predicate(
            r#"{"type":"grep_pattern","pattern":"FIXME","scope":null,"context_lines":0}"#,
        )
        .unwrap();

    let composite = CompiledPredicate::Composite {
        operator: CompositeOperator::And,
        children: vec![c1, c2],
    };

    let rules_with_compiled = vec![(make_rule("r-and", "engineering", 1, "WARN", 0xFF), composite)];

    let context = EvaluationContext {
        action: "write",
        source: "TODO FIXME",
        file_path: "src/main.rs",
        fsm_phase: FsmPhaseBit::EXECUTING,
    };

    let result = e.evaluate(&rules_with_compiled, &context).unwrap();
    let has_warn = result.iter().any(|v| v.result == VerdictResult::Warn);
    assert!(has_warn, "And composite must fire when all children match");
}

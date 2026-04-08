//! Contract tests for rules/predicate types (Phase 3F.2).
//!
//! Validates: serde round-trips for all 6 Predicate variants, Verdict types,
//! COMPOSITE depth enforcement (max=8), and error paths.

use elai_rules::predicate::{
    self, CompositeOperator, EscalationTarget, MetricOperator, MetricSource, Predicate,
    PredicateError, QueryFormat, Verdict, VerdictResult,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Serialize then deserialize, asserting round-trip equality.
fn round_trip(pred: &Predicate) -> Predicate {
    let json = serde_json::to_string(pred).expect("serialize should succeed");
    serde_json::from_str(&json).expect("deserialize should succeed")
}

fn round_trip_verdict(v: &Verdict) -> Verdict {
    let json = serde_json::to_string(v).expect("serialize should succeed");
    serde_json::from_str(&json).expect("deserialize should succeed")
}

// ---------------------------------------------------------------------------
// Predicate serde round-trips (all 6 variants)
// ---------------------------------------------------------------------------

#[test]
fn contract_ast_pattern_round_trip() {
    let pred = Predicate::AstPattern {
        language: "python".into(),
        query: "(function_definition)".into(),
        query_format: QueryFormat::Sexp,
        scope: None,
    };
    let rt = round_trip(&pred);
    assert_eq!(pred, rt);
}

#[test]
fn contract_ast_pattern_ast_grep_format() {
    let pred = Predicate::AstPattern {
        language: "rust".into(),
        query: "fn $NAME($$$PARAMS)".into(),
        query_format: QueryFormat::AstGrep,
        scope: Some("src/".into()),
    };
    let rt = round_trip(&pred);
    assert_eq!(pred, rt);
}

#[test]
fn contract_grep_pattern_round_trip() {
    let pred = Predicate::GrepPattern {
        pattern: "TODO".into(),
        scope: None,
        context_lines: 3,
    };
    let rt = round_trip(&pred);
    assert_eq!(pred, rt);
}

#[test]
fn contract_schema_required_round_trip() {
    let schema_val = serde_json::json!({"type": "object", "required": ["name"]});
    let pred = Predicate::SchemaRequired {
        schema: schema_val,
        output_type: "json".into(),
    };
    let rt = round_trip(&pred);
    assert_eq!(pred, rt);
}

#[test]
fn contract_metric_gate_lt_round_trip() {
    let pred = Predicate::MetricGate {
        metric_name: "cyclomatic_complexity".into(),
        operator: MetricOperator::Lt,
        threshold: Some(10.0),
        source: MetricSource::File,
    };
    let rt = round_trip(&pred);
    assert_eq!(pred, rt);
}

#[test]
fn contract_metric_gate_between_round_trip() {
    let pred = Predicate::MetricGate {
        metric_name: "line_count".into(),
        operator: MetricOperator::Between { low: 1.0, high: 500.0 },
        threshold: None,
        source: MetricSource::Runtime,
    };
    let rt = round_trip(&pred);
    assert_eq!(pred, rt);
}

#[test]
fn contract_composite_round_trip() {
    let child = Predicate::GrepPattern {
        pattern: "import os".into(),
        scope: None,
        context_lines: 0,
    };
    let pred = Predicate::Composite {
        operator: CompositeOperator::And,
        children: vec![child],
    };
    let rt = round_trip(&pred);
    assert_eq!(pred, rt);
}

#[test]
fn contract_composite_not_round_trip() {
    let child = Predicate::GrepPattern {
        pattern: "eval(".into(),
        scope: None,
        context_lines: 0,
    };
    let pred = Predicate::Composite {
        operator: CompositeOperator::Not,
        children: vec![child],
    };
    let rt = round_trip(&pred);
    assert_eq!(pred, rt);
}

#[test]
fn contract_graph_reachability_round_trip() {
    let pred = Predicate::GraphReachability {
        source_pattern: "user_input".into(),
        sink_pattern: "sql_query".into(),
        must_traverse: Some("sanitize".into()),
        must_not_exist: false,
        max_depth: 10,
        edge_types: vec!["call".into(), "data_flow".into()],
    };
    let rt = round_trip(&pred);
    assert_eq!(pred, rt);
}

// ---------------------------------------------------------------------------
// Verdict round-trips
// ---------------------------------------------------------------------------

#[test]
fn contract_verdict_allow_round_trip() {
    let v = Verdict {
        rule_id: "r-001".into(),
        result: VerdictResult::Allow,
        evidence: "all checks passed".into(),
    };
    assert_eq!(v, round_trip_verdict(&v));
}

#[test]
fn contract_verdict_block_round_trip() {
    let v = Verdict {
        rule_id: "r-002".into(),
        result: VerdictResult::Block,
        evidence: "security violation".into(),
    };
    assert_eq!(v, round_trip_verdict(&v));
}

#[test]
fn contract_verdict_escalate_auditor() {
    let v = Verdict {
        rule_id: "r-003".into(),
        result: VerdictResult::Escalate(EscalationTarget::Auditor),
        evidence: "needs review".into(),
    };
    assert_eq!(v, round_trip_verdict(&v));
}

#[test]
fn contract_verdict_escalate_next_tier() {
    let v = Verdict {
        rule_id: "r-004".into(),
        result: VerdictResult::Escalate(EscalationTarget::NextTierRule("r-005".into())),
        evidence: "threshold exceeded".into(),
    };
    assert_eq!(v, round_trip_verdict(&v));
}

#[test]
fn contract_verdict_escalate_human() {
    let v = Verdict {
        rule_id: "r-006".into(),
        result: VerdictResult::Escalate(EscalationTarget::Human),
        evidence: "ambiguous case".into(),
    };
    assert_eq!(v, round_trip_verdict(&v));
}

// ---------------------------------------------------------------------------
// COMPOSITE depth enforcement
// ---------------------------------------------------------------------------

/// Build a Composite chain of given depth.
fn nested_composite(depth: usize) -> Predicate {
    let leaf = Predicate::GrepPattern {
        pattern: "leaf".into(),
        scope: None,
        context_lines: 0,
    };
    let mut current = leaf;
    for _ in 0..depth {
        current = Predicate::Composite {
            operator: CompositeOperator::And,
            children: vec![current],
        };
    }
    current
}

#[test]
fn contract_composite_depth_8_succeeds() {
    let pred = nested_composite(8);
    let result = elai_rules::predicate::depth_check(&pred);
    assert!(result.is_ok(), "depth=8 must succeed (boundary)");
}

#[test]
fn contract_composite_depth_9_fails() {
    let pred = nested_composite(9);
    let result = elai_rules::predicate::depth_check(&pred);
    assert!(result.is_err(), "depth=9 must fail");
    match result.unwrap_err() {
        PredicateError::DepthExceeded { max } => assert_eq!(max, 8),
        other => panic!("expected DepthExceeded, got: {other}"),
    }
}

// ---------------------------------------------------------------------------
// Error paths
// ---------------------------------------------------------------------------

#[test]
fn contract_invalid_json_returns_err() {
    let result: Result<Predicate, _> = serde_json::from_str("not json at all");
    assert!(result.is_err(), "garbage input must return Err, not panic");
}

#[test]
fn contract_missing_type_tag_returns_err() {
    let result: Result<Predicate, _> = serde_json::from_str(r#"{"pattern":"x"}"#);
    assert!(result.is_err(), "missing type discriminant must return Err");
}

#[test]
fn contract_unknown_variant_returns_err() {
    let result: Result<Predicate, _> =
        serde_json::from_str(r#"{"type":"unknown_variant","data":"x"}"#);
    assert!(result.is_err(), "unknown variant must return Err");
}

// ---------------------------------------------------------------------------
// Not arity validation
// ---------------------------------------------------------------------------

#[test]
fn contract_not_arity_zero_rejected() {
    let pred = Predicate::Composite {
        operator: CompositeOperator::Not,
        children: vec![],
    };
    let result = predicate::validate(&pred);
    assert!(result.is_err(), "Not with 0 children must be rejected");
}

#[test]
fn contract_not_arity_two_rejected() {
    let leaf = Predicate::GrepPattern {
        pattern: "x".into(),
        scope: None,
        context_lines: 0,
    };
    let pred = Predicate::Composite {
        operator: CompositeOperator::Not,
        children: vec![leaf.clone(), leaf],
    };
    let result = predicate::validate(&pred);
    assert!(result.is_err(), "Not with 2 children must be rejected");
}

#[test]
fn contract_not_arity_one_accepted() {
    let leaf = Predicate::GrepPattern {
        pattern: "x".into(),
        scope: None,
        context_lines: 0,
    };
    let pred = Predicate::Composite {
        operator: CompositeOperator::Not,
        children: vec![leaf],
    };
    let result = predicate::validate(&pred);
    assert!(result.is_ok(), "Not with 1 child must be accepted");
}

// ---------------------------------------------------------------------------
// from_json constructor
// ---------------------------------------------------------------------------

#[test]
fn contract_from_json_valid() {
    let json = r#"{"type":"grep_pattern","pattern":"x","scope":null,"context_lines":0}"#;
    assert!(predicate::from_json(json).is_ok());
}

#[test]
fn contract_from_json_rejects_invalid() {
    let result = predicate::from_json("not json");
    assert!(result.is_err());
    match result.unwrap_err() {
        PredicateError::InvalidJson(_) => {}
        other => panic!("expected InvalidJson, got: {other}"),
    }
}

#[test]
fn contract_from_json_rejects_bad_not_arity() {
    let json = r#"{"type":"composite","operator":"not","children":[]}"#;
    let result = predicate::from_json(json);
    assert!(result.is_err(), "from_json must reject Not with 0 children");
}

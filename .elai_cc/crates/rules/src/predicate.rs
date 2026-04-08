//! Predicate types for the rules engine.
//!
//! Defines the 6 predicate variants (AST_PATTERN, GREP_PATTERN, SCHEMA_REQUIRED,
//! METRIC_GATE, COMPOSITE, GRAPH_REACHABILITY), verdict types, evaluator traits,
//! and a depth guard for nested composites.

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// Errors from predicate operations.
#[derive(Debug, thiserror::Error)]
pub enum PredicateError {
    #[error("composite nesting depth exceeds maximum ({max})")]
    DepthExceeded { max: usize },

    #[error("invalid predicate JSON: {0}")]
    InvalidJson(String),

    #[error("predicate validation failed: {0}")]
    ValidationFailed(String),
}

// ---------------------------------------------------------------------------
// Supporting enums
// ---------------------------------------------------------------------------

/// AST query language format.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QueryFormat {
    Sexp,
    AstGrep,
}

/// Comparison operator for metric gates.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MetricOperator {
    Lt,
    Gt,
    Eq,
    Between { low: f64, high: f64 },
}

/// Where a metric value is sourced from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MetricSource {
    File,
    Config,
    Runtime,
}

/// Boolean combinator for composite predicates.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompositeOperator {
    And,
    Or,
    Not,
}

// ---------------------------------------------------------------------------
// Predicate enum (6 variants)
// ---------------------------------------------------------------------------

/// Maximum nesting depth for Composite predicates.
pub const MAX_COMPOSITE_DEPTH: usize = 8;

/// A rule predicate — the condition that triggers a rule evaluation.
///
/// Stored as JSON in the `rules.predicate` TEXT column.
/// Uses internally-tagged serde: `{"type":"grep_pattern", ...}`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Predicate {
    AstPattern {
        language: String,
        query: String,
        query_format: QueryFormat,
        scope: Option<String>,
    },

    GrepPattern {
        pattern: String,
        scope: Option<String>,
        context_lines: u32,
    },

    SchemaRequired {
        schema: serde_json::Value,
        output_type: String,
    },

    MetricGate {
        metric_name: String,
        operator: MetricOperator,
        /// Threshold for Lt/Gt/Eq operators. Meaningless when operator is Between
        /// (which carries its own low/high bounds). None is valid for Between.
        threshold: Option<f64>,
        source: MetricSource,
    },

    /// Boolean composition of child predicates.
    ///
    /// Semantics: And/Or with empty children is vacuous true (short-circuit identity).
    /// Not must have exactly 1 child — enforced by [`validate`].
    Composite {
        operator: CompositeOperator,
        children: Vec<Predicate>,
    },

    GraphReachability {
        source_pattern: String,
        sink_pattern: String,
        must_traverse: Option<String>,
        must_not_exist: bool,
        max_depth: u8,
        edge_types: Vec<String>,
    },
}

// ---------------------------------------------------------------------------
// Depth check
// ---------------------------------------------------------------------------

/// Validate that a predicate does not exceed [`MAX_COMPOSITE_DEPTH`] nesting.
///
/// O(depth) recursive walk. Returns `Err(PredicateError::DepthExceeded)` if
/// any Composite chain exceeds the limit.
pub fn depth_check(predicate: &Predicate) -> Result<(), PredicateError> {
    depth_check_inner(predicate, 0)
}

fn depth_check_inner(predicate: &Predicate, current: usize) -> Result<(), PredicateError> {
    match predicate {
        Predicate::Composite { children, .. } => {
            let next = current + 1;
            if next > MAX_COMPOSITE_DEPTH {
                return Err(PredicateError::DepthExceeded {
                    max: MAX_COMPOSITE_DEPTH,
                });
            }
            for child in children {
                depth_check_inner(child, next)?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

/// Validate structural invariants of a predicate tree.
///
/// Checks: depth limit, Not arity (exactly 1 child), MetricGate threshold
/// presence (required for Lt/Gt/Eq operators).
/// Call after deserialization via [`from_json`] or manually.
pub fn validate(predicate: &Predicate) -> Result<(), PredicateError> {
    depth_check(predicate)?;
    validate_structure(predicate)
}

fn validate_structure(predicate: &Predicate) -> Result<(), PredicateError> {
    match predicate {
        Predicate::Composite { operator, children } => {
            if *operator == CompositeOperator::Not && children.len() != 1 {
                return Err(PredicateError::ValidationFailed(format!(
                    "Not operator requires exactly 1 child, got {}",
                    children.len()
                )));
            }
            for child in children {
                validate_structure(child)?;
            }
            Ok(())
        }
        Predicate::MetricGate {
            operator, threshold, ..
        } => {
            let needs_threshold = !matches!(operator, MetricOperator::Between { .. });
            if needs_threshold && threshold.is_none() {
                return Err(PredicateError::ValidationFailed(
                    "Lt/Gt/Eq operators require threshold to be Some".into(),
                ));
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

/// Deserialize a predicate from JSON and validate structural invariants.
///
/// Combines `serde_json::from_str` with [`validate`] (depth + arity checks).
/// Prefer this over raw `serde_json::from_str::<Predicate>()` when reading
/// from `rules.predicate` TEXT column.
pub fn from_json(json: &str) -> Result<Predicate, PredicateError> {
    let predicate: Predicate =
        serde_json::from_str(json).map_err(|e| PredicateError::InvalidJson(e.to_string()))?;
    validate(&predicate)?;
    Ok(predicate)
}

// ---------------------------------------------------------------------------
// Verdict types
// ---------------------------------------------------------------------------

/// Where to escalate a rule evaluation when the predicate result is ambiguous.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EscalationTarget {
    Auditor,
    Human,
    NextTierRule(String),
}

/// The outcome of evaluating a rule's predicate against code.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerdictResult {
    Allow,
    Warn,
    Block,
    Escalate(EscalationTarget),
}

/// A complete verdict: the rule that fired, the result, and supporting evidence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Verdict {
    pub rule_id: String,
    pub result: VerdictResult,
    pub evidence: String,
}

// ---------------------------------------------------------------------------
// Evaluator traits (stubs for 3F.3)
// ---------------------------------------------------------------------------

/// Result of an AST query match.
#[derive(Debug, Clone)]
pub struct AstMatch {
    pub node_text: String,
    pub start_line: u32,
    pub end_line: u32,
}

/// Error from AST evaluation.
#[derive(Debug, thiserror::Error)]
pub enum AstError {
    #[error("unsupported language: {0}")]
    UnsupportedLanguage(String),

    #[error("invalid query: {0}")]
    InvalidQuery(String),

    #[error("evaluation failed: {0}")]
    EvaluationFailed(String),
}

/// Evaluates AST queries against source code.
///
/// Implementors provide language-specific tree-sitter (or ast-grep) evaluation.
/// Object-safe for use as a trait object when needed.
pub trait AstEvaluator {
    fn evaluate_query(
        &self,
        language: &str,
        source: &str,
        query: &str,
    ) -> Result<Vec<AstMatch>, AstError>;

    fn validate_query(&self, language: &str, query: &str) -> Result<(), AstError>;
}

/// Parameters for a graph reachability query.
#[derive(Debug, Clone)]
pub struct ReachabilityParams {
    pub source_pattern: String,
    pub sink_pattern: String,
    pub must_traverse: Option<String>,
    pub must_not_exist: bool,
    pub max_depth: u8,
    pub edge_types: Vec<String>,
}

/// Result of a reachability query.
#[derive(Debug, Clone)]
pub struct ReachabilityMatch {
    pub path: Vec<String>,
    pub edge_labels: Vec<String>,
}

/// Error from DKG (dependency/knowledge graph) queries.
#[derive(Debug, thiserror::Error)]
pub enum DkgError {
    #[error("invalid reachability params: {0}")]
    InvalidParams(String),

    #[error("query failed: {0}")]
    QueryFailed(String),
}

/// Queries a dependency/knowledge graph for reachability.
///
/// Implementors provide graph-backed reachability analysis.
/// Object-safe for use as a trait object when needed.
pub trait DkgQuerier {
    fn query_reachability(
        &self,
        params: &ReachabilityParams,
    ) -> Result<Vec<ReachabilityMatch>, DkgError>;

    fn validate_params(&self, params: &ReachabilityParams) -> Result<(), DkgError>;
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- QueryFormat --

    #[test]
    fn query_format_serde_sexp() {
        let qf = QueryFormat::Sexp;
        let json = serde_json::to_string(&qf).unwrap();
        assert_eq!(json, r#""sexp""#);
        let rt: QueryFormat = serde_json::from_str(&json).unwrap();
        assert_eq!(rt, qf);
    }

    #[test]
    fn query_format_serde_ast_grep() {
        let qf = QueryFormat::AstGrep;
        let json = serde_json::to_string(&qf).unwrap();
        assert_eq!(json, r#""ast_grep""#);
        let rt: QueryFormat = serde_json::from_str(&json).unwrap();
        assert_eq!(rt, qf);
    }

    // -- MetricOperator --

    #[test]
    fn metric_operator_lt_serde() {
        let op = MetricOperator::Lt;
        let json = serde_json::to_string(&op).unwrap();
        assert_eq!(json, r#""lt""#);
    }

    #[test]
    fn metric_operator_between_serde() {
        let op = MetricOperator::Between { low: 1.0, high: 10.0 };
        let json = serde_json::to_string(&op).unwrap();
        let rt: MetricOperator = serde_json::from_str(&json).unwrap();
        assert_eq!(rt, op);
    }

    // -- MetricSource --

    #[test]
    fn metric_source_all_variants() {
        for (src, expected) in [
            (MetricSource::File, r#""file""#),
            (MetricSource::Config, r#""config""#),
            (MetricSource::Runtime, r#""runtime""#),
        ] {
            let json = serde_json::to_string(&src).unwrap();
            assert_eq!(json, expected);
        }
    }

    // -- CompositeOperator --

    #[test]
    fn composite_operator_all_variants() {
        for (op, expected) in [
            (CompositeOperator::And, r#""and""#),
            (CompositeOperator::Or, r#""or""#),
            (CompositeOperator::Not, r#""not""#),
        ] {
            let json = serde_json::to_string(&op).unwrap();
            assert_eq!(json, expected);
        }
    }

    // -- Predicate variants --

    #[test]
    fn predicate_ast_pattern_json_shape() {
        let pred = Predicate::AstPattern {
            language: "python".into(),
            query: "(function_definition)".into(),
            query_format: QueryFormat::Sexp,
            scope: None,
        };
        let json = serde_json::to_string(&pred).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["type"], "ast_pattern");
        assert_eq!(v["language"], "python");
        assert_eq!(v["query_format"], "sexp");
        assert!(v["scope"].is_null());
    }

    #[test]
    fn predicate_grep_pattern_round_trip() {
        let pred = Predicate::GrepPattern {
            pattern: "TODO".into(),
            scope: Some("src/".into()),
            context_lines: 5,
        };
        let json = serde_json::to_string(&pred).unwrap();
        let rt: Predicate = serde_json::from_str(&json).unwrap();
        assert_eq!(pred, rt);
    }

    #[test]
    fn predicate_schema_required_round_trip() {
        let pred = Predicate::SchemaRequired {
            schema: serde_json::json!({"type": "string"}),
            output_type: "json".into(),
        };
        let json = serde_json::to_string(&pred).unwrap();
        let rt: Predicate = serde_json::from_str(&json).unwrap();
        assert_eq!(pred, rt);
    }

    #[test]
    fn predicate_metric_gate_round_trip() {
        let pred = Predicate::MetricGate {
            metric_name: "cc".into(),
            operator: MetricOperator::Gt,
            threshold: Some(15.0),
            source: MetricSource::Config,
        };
        let json = serde_json::to_string(&pred).unwrap();
        let rt: Predicate = serde_json::from_str(&json).unwrap();
        assert_eq!(pred, rt);
    }

    #[test]
    fn predicate_metric_gate_between_no_threshold() {
        let pred = Predicate::MetricGate {
            metric_name: "lines".into(),
            operator: MetricOperator::Between { low: 1.0, high: 500.0 },
            threshold: None,
            source: MetricSource::File,
        };
        let json = serde_json::to_string(&pred).unwrap();
        let rt: Predicate = serde_json::from_str(&json).unwrap();
        assert_eq!(pred, rt);
    }

    #[test]
    fn predicate_composite_nested() {
        let inner = Predicate::GrepPattern {
            pattern: "eval".into(),
            scope: None,
            context_lines: 0,
        };
        let pred = Predicate::Composite {
            operator: CompositeOperator::Or,
            children: vec![
                inner.clone(),
                Predicate::Composite {
                    operator: CompositeOperator::Not,
                    children: vec![inner],
                },
            ],
        };
        let json = serde_json::to_string(&pred).unwrap();
        let rt: Predicate = serde_json::from_str(&json).unwrap();
        assert_eq!(pred, rt);
    }

    #[test]
    fn predicate_graph_reachability_round_trip() {
        let pred = Predicate::GraphReachability {
            source_pattern: "user_input".into(),
            sink_pattern: "db_query".into(),
            must_traverse: None,
            must_not_exist: true,
            max_depth: 5,
            edge_types: vec!["call".into()],
        };
        let json = serde_json::to_string(&pred).unwrap();
        let rt: Predicate = serde_json::from_str(&json).unwrap();
        assert_eq!(pred, rt);
    }

    // -- depth_check --

    fn make_nested(depth: usize) -> Predicate {
        let leaf = Predicate::GrepPattern {
            pattern: "x".into(),
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
    fn depth_check_flat_ok() {
        let pred = Predicate::GrepPattern {
            pattern: "x".into(),
            scope: None,
            context_lines: 0,
        };
        assert!(depth_check(&pred).is_ok());
    }

    #[test]
    fn depth_check_at_limit_ok() {
        let pred = make_nested(MAX_COMPOSITE_DEPTH);
        assert!(depth_check(&pred).is_ok());
    }

    #[test]
    fn depth_check_over_limit_err() {
        let pred = make_nested(MAX_COMPOSITE_DEPTH + 1);
        let err = depth_check(&pred).unwrap_err();
        match err {
            PredicateError::DepthExceeded { max } => assert_eq!(max, MAX_COMPOSITE_DEPTH),
            _ => panic!("expected DepthExceeded"),
        }
    }

    #[test]
    fn depth_check_wide_at_limit_ok() {
        // Wide tree: 8 levels deep but each with multiple children.
        let leaf = Predicate::GrepPattern {
            pattern: "x".into(),
            scope: None,
            context_lines: 0,
        };
        let mut current = Predicate::Composite {
            operator: CompositeOperator::And,
            children: vec![leaf.clone(), leaf.clone(), leaf],
        };
        for _ in 1..MAX_COMPOSITE_DEPTH {
            current = Predicate::Composite {
                operator: CompositeOperator::And,
                children: vec![current],
            };
        }
        assert!(depth_check(&current).is_ok());
    }

    // -- validate (MetricGate threshold) --

    #[test]
    fn validate_metric_gate_lt_needs_threshold() {
        let pred = Predicate::MetricGate {
            metric_name: "cc".into(),
            operator: MetricOperator::Lt,
            threshold: None,
            source: MetricSource::File,
        };
        assert!(validate(&pred).is_err(), "Lt with None threshold must fail");
    }

    #[test]
    fn validate_metric_gate_between_no_threshold_ok() {
        let pred = Predicate::MetricGate {
            metric_name: "lines".into(),
            operator: MetricOperator::Between { low: 1.0, high: 10.0 },
            threshold: None,
            source: MetricSource::File,
        };
        assert!(validate(&pred).is_ok(), "Between with None threshold is valid");
    }

    #[test]
    fn validate_metric_gate_gt_with_threshold_ok() {
        let pred = Predicate::MetricGate {
            metric_name: "cc".into(),
            operator: MetricOperator::Gt,
            threshold: Some(5.0),
            source: MetricSource::Runtime,
        };
        assert!(validate(&pred).is_ok());
    }

    // -- validate (Not arity) --

    #[test]
    fn validate_not_with_one_child_ok() {
        let pred = Predicate::Composite {
            operator: CompositeOperator::Not,
            children: vec![Predicate::GrepPattern {
                pattern: "x".into(),
                scope: None,
                context_lines: 0,
            }],
        };
        assert!(validate(&pred).is_ok());
    }

    #[test]
    fn validate_not_with_zero_children_err() {
        let pred = Predicate::Composite {
            operator: CompositeOperator::Not,
            children: vec![],
        };
        let err = validate(&pred).unwrap_err();
        match err {
            PredicateError::ValidationFailed(msg) => {
                assert!(msg.contains("Not"));
                assert!(msg.contains("0"));
            }
            _ => panic!("expected ValidationFailed"),
        }
    }

    #[test]
    fn validate_not_with_two_children_err() {
        let leaf = Predicate::GrepPattern {
            pattern: "x".into(),
            scope: None,
            context_lines: 0,
        };
        let pred = Predicate::Composite {
            operator: CompositeOperator::Not,
            children: vec![leaf.clone(), leaf],
        };
        assert!(validate(&pred).is_err());
    }

    #[test]
    fn validate_nested_not_arity_err() {
        let bad_not = Predicate::Composite {
            operator: CompositeOperator::Not,
            children: vec![],
        };
        let outer = Predicate::Composite {
            operator: CompositeOperator::And,
            children: vec![bad_not],
        };
        assert!(validate(&outer).is_err(), "nested invalid Not must be caught");
    }

    #[test]
    fn from_json_valid() {
        let json = r#"{"type":"grep_pattern","pattern":"x","scope":null,"context_lines":0}"#;
        assert!(from_json(json).is_ok());
    }

    #[test]
    fn from_json_invalid_json() {
        let err = from_json("not json").unwrap_err();
        match err {
            PredicateError::InvalidJson(_) => {}
            _ => panic!("expected InvalidJson"),
        }
    }

    #[test]
    fn from_json_rejects_deep_composite() {
        // Build depth-9 JSON manually.
        let mut json = r#"{"type":"grep_pattern","pattern":"x","scope":null,"context_lines":0}"#.to_string();
        for _ in 0..9 {
            json = format!(r#"{{"type":"composite","operator":"and","children":[{json}]}}"#);
        }
        let err = from_json(&json).unwrap_err();
        match err {
            PredicateError::DepthExceeded { .. } => {}
            _ => panic!("expected DepthExceeded"),
        }
    }

    #[test]
    fn from_json_rejects_not_zero_children() {
        let json = r#"{"type":"composite","operator":"not","children":[]}"#;
        let err = from_json(json).unwrap_err();
        match err {
            PredicateError::ValidationFailed(msg) => assert!(msg.contains("Not")),
            _ => panic!("expected ValidationFailed"),
        }
    }

    // -- Verdict --

    #[test]
    fn verdict_result_all_variants_serde() {
        let variants = vec![
            VerdictResult::Allow,
            VerdictResult::Warn,
            VerdictResult::Block,
            VerdictResult::Escalate(EscalationTarget::Auditor),
            VerdictResult::Escalate(EscalationTarget::Human),
            VerdictResult::Escalate(EscalationTarget::NextTierRule("r-99".into())),
        ];
        for v in &variants {
            let json = serde_json::to_string(v).unwrap();
            let rt: VerdictResult = serde_json::from_str(&json).unwrap();
            assert_eq!(*v, rt);
        }
    }

    #[test]
    fn verdict_struct_round_trip() {
        let v = Verdict {
            rule_id: "r-001".into(),
            result: VerdictResult::Escalate(EscalationTarget::NextTierRule("r-002".into())),
            evidence: "threshold exceeded".into(),
        };
        let json = serde_json::to_string(&v).unwrap();
        let rt: Verdict = serde_json::from_str(&json).unwrap();
        assert_eq!(v, rt);
    }

    // -- Error display --

    #[test]
    fn predicate_error_display() {
        let err = PredicateError::DepthExceeded { max: 8 };
        assert!(err.to_string().contains("8"));

        let err = PredicateError::InvalidJson("bad".into());
        assert!(err.to_string().contains("bad"));

        let err = PredicateError::ValidationFailed("oops".into());
        assert!(err.to_string().contains("oops"));
    }

    // -- Deserialization error paths --

    #[test]
    fn invalid_json_returns_err() {
        let r: Result<Predicate, _> = serde_json::from_str("{{garbage}}");
        assert!(r.is_err());
    }

    #[test]
    fn unknown_type_tag_returns_err() {
        let r: Result<Predicate, _> = serde_json::from_str(r#"{"type":"banana"}"#);
        assert!(r.is_err());
    }

    #[test]
    fn missing_required_field_returns_err() {
        // grep_pattern missing 'pattern' field.
        let r: Result<Predicate, _> =
            serde_json::from_str(r#"{"type":"grep_pattern","context_lines":0}"#);
        assert!(r.is_err());
    }
}

//! Rule engine: evaluation, compilation, and active-rule filtering.
//!
//! Provides the [`RuleEngine`] trait (object-safe), [`DefaultRuleEngine`] impl,
//! [`compile_predicate`] for insertion-time validation, and [`EvaluationContext`]
//! for runtime evaluation. Evaluation order is locked:
//! MetricGate → GrepPattern → AstPattern → Composite → GraphReachability.

use regex::Regex;
use serde::{Deserialize, Serialize};

use std::collections::HashMap;

use crate::predicate::{
    from_json, AstError, AstEvaluator, CompositeOperator, DkgError, DkgQuerier, EscalationTarget,
    MetricOperator, MetricSource, Predicate, PredicateError, QueryFormat, ReachabilityParams,
    Verdict, VerdictResult,
};

// ---------------------------------------------------------------------------
// FSM phase bitmask (local — no orchestrator dependency)
// ---------------------------------------------------------------------------

/// Bitmask index for FSM phases. Used by `get_active_rules` to filter rules
/// whose `phase_mask` includes the current phase.
///
/// Bit ordering: 0=PLANNING, 1=FEASIBILITY_PROBE, 2=TESTING_FIRST,
/// 3=EXECUTING, 4=AUDITING, 5=MEMORIZING, 6=CONFIGURING.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FsmPhaseBit(u8);

impl FsmPhaseBit {
    pub const PLANNING: Self = Self(0);
    pub const FEASIBILITY_PROBE: Self = Self(1);
    pub const TESTING_FIRST: Self = Self(2);
    pub const EXECUTING: Self = Self(3);
    pub const AUDITING: Self = Self(4);
    pub const MEMORIZING: Self = Self(5);
    pub const CONFIGURING: Self = Self(6);

    /// Maximum valid bit index (inclusive).
    const MAX_BIT: u8 = 6;

    /// Create from a raw bit index. Returns `None` if out of range.
    pub fn from_raw(bit: u8) -> Option<Self> {
        if bit <= Self::MAX_BIT {
            Some(Self(bit))
        } else {
            None
        }
    }

    /// The raw bit index.
    pub fn bit(self) -> u8 {
        self.0
    }

    /// The bitmask with this phase's bit set: `1 << bit`.
    pub fn mask(self) -> u32 {
        1u32 << self.0
    }
}

// ---------------------------------------------------------------------------
// Evaluation context
// ---------------------------------------------------------------------------

/// Context passed to [`RuleEngine::evaluate`].
pub struct EvaluationContext<'a> {
    /// The action being evaluated (e.g. "write", "execute", "modify").
    pub action: &'a str,
    /// Source code content to evaluate against.
    pub source: &'a str,
    /// File path of the source being evaluated.
    pub file_path: &'a str,
    /// Current FSM phase (for phase-aware filtering).
    pub fsm_phase: FsmPhaseBit,
    /// Metric values for MetricGate evaluation (metric_name -> value).
    pub metric_values: HashMap<String, f64>,
}

// ---------------------------------------------------------------------------
// Compiled predicate
// ---------------------------------------------------------------------------

/// A predicate that has been validated and pre-compiled at insertion time.
///
/// Holds compiled artifacts (e.g. [`Regex`]) so runtime evaluation avoids
/// repeated compilation.
pub enum CompiledPredicate {
    MetricGate {
        metric_name: String,
        operator: MetricOperator,
        threshold: Option<f64>,
        source: MetricSource,
    },
    GrepPattern {
        compiled_regex: Regex,
        scope: Option<String>,
        context_lines: u32,
    },
    AstPattern {
        language: String,
        query: String,
        query_format: QueryFormat,
        scope: Option<String>,
    },
    SchemaRequired {
        validator: jsonschema::Validator,
        output_type: String,
    },
    Composite {
        operator: CompositeOperator,
        children: Vec<CompiledPredicate>,
    },
    GraphReachability {
        params: ReachabilityParams,
    },
    VersionGate {
        ecosystem: String,
        deprecated_name: String,
        replacement_name: String,
        severity: String,
    },
}

// ---------------------------------------------------------------------------
// Rule (row from the rules table)
// ---------------------------------------------------------------------------

/// A rule record as stored in the `rules` table, with a compiled predicate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub rule_id: String,
    pub domain: String,
    pub tier: u8,
    pub severity: String,
    pub title: String,
    /// Raw predicate JSON (stored in DB). Compiled on load.
    pub predicate_json: String,
    pub phase_mask: u32,
    /// Escalation target when severity is "ESCALATE". Defaults to Auditor if None.
    #[serde(default)]
    pub escalate_target: Option<EscalationTarget>,
}

// ---------------------------------------------------------------------------
// Evaluation result
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// Rule engine error
// ---------------------------------------------------------------------------

/// Errors from rule engine operations.
#[derive(Debug, thiserror::Error)]
pub enum RuleEngineError {
    #[error("predicate error: {0}")]
    Predicate(#[from] PredicateError),

    #[error("AST evaluation error: {0}")]
    Ast(#[from] AstError),

    #[error("DKG query error: {0}")]
    Dkg(#[from] DkgError),

    #[error("regex match error on rule {rule_id}: {detail}")]
    RegexMatch { rule_id: String, detail: String },

    #[error("schema validation error on rule {rule_id}: {detail}")]
    SchemaValidation { rule_id: String, detail: String },
}

// ---------------------------------------------------------------------------
// Domains always included regardless of file context
// ---------------------------------------------------------------------------

/// Domains that are always included in active rules regardless of file context.
const ALWAYS_INCLUDED_DOMAINS: &[&str] = &["engineering", "security"];

// ---------------------------------------------------------------------------
// Domain detection from file paths
// ---------------------------------------------------------------------------

/// Keyword table for domain detection. Each entry maps a domain name to its
/// trigger keywords. Used by [`detect_domains`] to classify file paths.
///
/// Note: "engineering" and "security" are intentionally absent — those are
/// handled by [`ALWAYS_INCLUDED_DOMAINS`] in [`get_active_rules`].
const DOMAIN_KEYWORDS: &[(&str, &[&str])] = &[
    ("trading", &["trading", "backtest", "signal", "pipeline"]),
    ("dashboard", &["dashboard", "component", "chart", "ui"]),
    ("application", &["payment", "billing", "tenant", "saas"]),
    ("game", &["game", "mod", "entity", "spawn"]),
    ("re", &["ida", "ghidra", "disasm", "hook"]),
];

/// Detect applicable domains for a file path by keyword matching.
///
/// Lowercases the entire path before matching. Returns a deduplicated
/// `Vec<String>` of matched domain names. Never returns "engineering"
/// or "security" — those are always-included via [`ALWAYS_INCLUDED_DOMAINS`].
///
/// # Caveats
///
/// The "game" domain keyword "mod" will match Rust module paths like
/// `src/mod.rs`. This is spec-mandated behavior.
pub fn detect_domains(file_path: &str) -> Vec<String> {
    let lower = file_path.to_lowercase();
    let mut domains = Vec::new();
    for &(domain, keywords) in DOMAIN_KEYWORDS {
        if keywords.iter().any(|kw| lower.contains(kw)) {
            domains.push(domain.to_owned());
        }
    }
    domains
}

// ---------------------------------------------------------------------------
// RuleEngine trait (object-safe)
// ---------------------------------------------------------------------------

/// The core rule engine trait. Object-safe for use as `dyn RuleEngine`.
///
/// Evaluation order is locked: MetricGate → GrepPattern → AstPattern →
/// Composite → GraphReachability. Short-circuits on Block, accumulates Warns.
pub trait RuleEngine {
    /// Evaluate a set of compiled predicates against the given context.
    ///
    /// Returns the accumulated result with all warnings and the final verdict.
    fn evaluate(
        &self,
        rules: &[(Rule, CompiledPredicate)],
        context: &EvaluationContext<'_>,
    ) -> Result<Vec<Verdict>, RuleEngineError>;

    /// Filter active rules for the given FSM phase and file context domains.
    ///
    /// SQL semantics: `WHERE status='ACTIVE' AND (phase_mask & (1<<bit))!=0
    /// AND (domain IN detected_domains OR domain IN ('engineering','security'))
    /// ORDER BY tier ASC, severity DESC`.
    fn get_active_rules<'a>(
        &self,
        rules: &'a [Rule],
        fsm_phase: FsmPhaseBit,
        file_domains: &[&str],
    ) -> Vec<&'a Rule>;

    /// Compile a raw predicate JSON string into a [`CompiledPredicate`].
    ///
    /// Validates at insertion time: regex compilation, JSON schema validation,
    /// AST query validation, DKG param validation. Uses [`from_json`] for
    /// structural validation (depth, Not arity, etc.).
    fn compile_predicate(&self, raw: &str) -> Result<CompiledPredicate, PredicateError>;
}

// ---------------------------------------------------------------------------
// Default implementation
// ---------------------------------------------------------------------------

/// Default rule engine backed by injected AST evaluator and DKG querier.
pub struct DefaultRuleEngine {
    ast_evaluator: Box<dyn AstEvaluator + Send + Sync>,
    dkg_querier: Box<dyn DkgQuerier + Send + Sync>,
}

impl DefaultRuleEngine {
    pub fn new(
        ast_evaluator: Box<dyn AstEvaluator + Send + Sync>,
        dkg_querier: Box<dyn DkgQuerier + Send + Sync>,
    ) -> Self {
        Self {
            ast_evaluator,
            dkg_querier,
        }
    }
}

// ---------------------------------------------------------------------------
// Predicate evaluation ordering
// ---------------------------------------------------------------------------

/// Returns a sort key for evaluation ordering. Lower = evaluated first.
/// MetricGate(0) → VersionGate/GrepPattern(1) → AstPattern(2) → Composite(3) → GraphReachability(4).
fn eval_order(pred: &CompiledPredicate) -> u8 {
    match pred {
        CompiledPredicate::MetricGate { .. } => 0,
        // VERSION_GATE is O(n) string matching, same tier as GREP.
        CompiledPredicate::VersionGate { .. } => 1,
        CompiledPredicate::GrepPattern { .. } => 1,
        CompiledPredicate::AstPattern { .. } => 2,
        CompiledPredicate::SchemaRequired { .. } => 2,
        CompiledPredicate::Composite { .. } => 3,
        CompiledPredicate::GraphReachability { .. } => 4,
    }
}

// ---------------------------------------------------------------------------
// Severity ordering for tie-breaking
// ---------------------------------------------------------------------------

fn severity_rank(s: &str) -> u8 {
    match s {
        "BLOCK" => 0,
        "WARN" => 1,
        "INFO" => 2,
        _ => 3,
    }
}

// ---------------------------------------------------------------------------
// RuleEngine impl
// ---------------------------------------------------------------------------

impl RuleEngine for DefaultRuleEngine {
    fn evaluate(
        &self,
        rules: &[(Rule, CompiledPredicate)],
        context: &EvaluationContext<'_>,
    ) -> Result<Vec<Verdict>, RuleEngineError> {
        // Sort by eval order (cheapest first), then tier ASC, severity DESC.
        let mut indexed: Vec<usize> = (0..rules.len()).collect();
        indexed.sort_by(|&a, &b| {
            let ord_a = eval_order(&rules[a].1);
            let ord_b = eval_order(&rules[b].1);
            ord_a
                .cmp(&ord_b)
                .then_with(|| rules[a].0.tier.cmp(&rules[b].0.tier))
                .then_with(|| severity_rank(&rules[a].0.severity).cmp(&severity_rank(&rules[b].0.severity)))
        });

        let mut verdicts: Vec<Verdict> = Vec::new();

        for &idx in &indexed {
            let (rule, compiled) = &rules[idx];
            let result = self.evaluate_single(compiled, context, rule)?;

            match &result.result {
                VerdictResult::Block => {
                    // Short-circuit: return block + all accumulated warnings.
                    verdicts.push(result);
                    return Ok(verdicts);
                }
                VerdictResult::Warn => {
                    verdicts.push(result);
                }
                VerdictResult::Allow | VerdictResult::Escalate(_) => {
                    verdicts.push(result);
                }
            }
        }

        Ok(verdicts)
    }

    fn get_active_rules<'a>(
        &self,
        rules: &'a [Rule],
        fsm_phase: FsmPhaseBit,
        file_domains: &[&str],
    ) -> Vec<&'a Rule> {
        let phase_bit = fsm_phase.mask();
        let mut active: Vec<&Rule> = rules
            .iter()
            .filter(|r| {
                (r.phase_mask & phase_bit) != 0
                    && (file_domains.contains(&r.domain.as_str())
                        || ALWAYS_INCLUDED_DOMAINS.contains(&r.domain.as_str()))
            })
            .collect();

        // ORDER BY tier ASC, severity DESC
        active.sort_by(|a, b| {
            a.tier
                .cmp(&b.tier)
                .then_with(|| severity_rank(&a.severity).cmp(&severity_rank(&b.severity)))
        });

        active
    }

    fn compile_predicate(&self, raw: &str) -> Result<CompiledPredicate, PredicateError> {
        let parsed = from_json(raw)?;
        self.compile_parsed(&parsed)
    }
}

impl DefaultRuleEngine {
    /// Relative tolerance for MetricGate Eq comparison.
    /// Handles floating-point rounding from computed metrics (division, aggregation).
    const METRIC_EQ_TOLERANCE: f64 = 1e-9;

    /// Evaluate a single compiled predicate against the context.
    ///
    /// Each match arm returns `(fires: bool, evidence: String)`.
    /// Post-match: severity_verdict maps fires→VerdictResult, then ESCALATE
    /// routing overrides when severity=="ESCALATE" and predicate fired.
    fn evaluate_single(
        &self,
        compiled: &CompiledPredicate,
        context: &EvaluationContext<'_>,
        rule: &Rule,
    ) -> Result<Verdict, RuleEngineError> {
        let (fires, evidence) = match compiled {
            CompiledPredicate::MetricGate {
                metric_name,
                operator,
                threshold,
                source: _,
            } => {
                let value = match context.metric_values.get(metric_name) {
                    Some(v) => *v,
                    None => {
                        return Ok(Verdict {
                            rule_id: rule.rule_id.clone(),
                            result: VerdictResult::Allow,
                            evidence: format!(
                                "metric '{}' not in context; gate skipped",
                                metric_name
                            ),
                        });
                    }
                };
                let gate_fires = match operator {
                    MetricOperator::Lt => {
                        let t = threshold.ok_or_else(|| {
                            RuleEngineError::Predicate(PredicateError::ValidationFailed(
                                "Lt requires threshold".into(),
                            ))
                        })?;
                        value < t
                    }
                    MetricOperator::Gt => {
                        let t = threshold.ok_or_else(|| {
                            RuleEngineError::Predicate(PredicateError::ValidationFailed(
                                "Gt requires threshold".into(),
                            ))
                        })?;
                        value > t
                    }
                    MetricOperator::Eq => {
                        let t = threshold.ok_or_else(|| {
                            RuleEngineError::Predicate(PredicateError::ValidationFailed(
                                "Eq requires threshold".into(),
                            ))
                        })?;
                        (value - t).abs() < t.abs().max(1.0) * Self::METRIC_EQ_TOLERANCE
                    }
                    MetricOperator::Between { low, high } => value >= *low && value <= *high,
                };
                let ev = format!(
                    "metric '{}' = {}; operator {:?} threshold {:?}",
                    metric_name, value, operator, threshold
                );
                (gate_fires, ev)
            }
            CompiledPredicate::GrepPattern {
                compiled_regex,
                scope,
                ..
            } => {
                let in_scope = scope
                    .as_ref()
                    .is_none_or(|s| context.file_path.starts_with(s.as_str()));
                if in_scope {
                    if let Some(m) = compiled_regex.find(context.source) {
                        let ev = format!(
                            "grep match at offset {} in {}",
                            m.start(),
                            context.file_path
                        );
                        (true, ev)
                    } else {
                        (false, String::new())
                    }
                } else {
                    (false, "out of scope".into())
                }
            }
            CompiledPredicate::AstPattern {
                language,
                query,
                scope,
                ..
            } => {
                let in_scope = scope
                    .as_ref()
                    .is_none_or(|s| context.file_path.starts_with(s.as_str()));
                if !in_scope {
                    return Ok(Verdict {
                        rule_id: rule.rule_id.clone(),
                        result: VerdictResult::Allow,
                        evidence: "out of scope".into(),
                    });
                }
                let matches = self
                    .ast_evaluator
                    .evaluate_query(language, context.source, query)?;
                if matches.is_empty() {
                    let ev = format!(
                        "no AST match for query in {}",
                        context.file_path
                    );
                    (false, ev)
                } else {
                    let q_display: String = query.chars().take(60).collect();
                    let ev = format!(
                        "AST match count={} for query '{}' in {} at {}",
                        matches.len(),
                        q_display,
                        language,
                        context.file_path
                    );
                    (true, ev)
                }
            }
            CompiledPredicate::SchemaRequired {
                validator,
                output_type,
            } => {
                // If source is not valid JSON, the rule is not applicable (Allow).
                // Do NOT propagate the parse error — it would abort the entire
                // evaluation pipeline, preventing subsequent security-critical rules
                // from running.
                let value: serde_json::Value = match serde_json::from_str(context.source) {
                    Ok(v) => v,
                    Err(_) => {
                        return Ok(Verdict {
                            rule_id: rule.rule_id.clone(),
                            result: VerdictResult::Allow,
                            evidence: "source is not JSON; schema rule not applicable".into(),
                        });
                    }
                };
                if validator.is_valid(&value) {
                    (false, String::new())
                } else {
                    let ev = format!(
                        "schema validation failed for output_type={} in {}",
                        output_type, context.file_path
                    );
                    (true, ev)
                }
            }
            CompiledPredicate::Composite { operator, children } => {
                let composite_fires =
                    self.evaluate_composite(operator, children, context, rule)?;
                let ev = if composite_fires {
                    format!(
                        "composite {:?}: fired ({} children)",
                        operator,
                        children.len()
                    )
                } else {
                    String::new()
                };
                (composite_fires, ev)
            }
            CompiledPredicate::GraphReachability { params } => {
                let matches = self.dkg_querier.query_reachability(params)?;
                let graph_fires = if params.must_not_exist {
                    !matches.is_empty()
                } else {
                    matches.is_empty()
                };
                let ev = if graph_fires {
                    if params.must_not_exist {
                        format!(
                            "graph path found: {} paths from '{}' to '{}'",
                            matches.len(),
                            params.source_pattern,
                            params.sink_pattern
                        )
                    } else {
                        format!(
                            "no graph path from '{}' to '{}' (required)",
                            params.source_pattern, params.sink_pattern
                        )
                    }
                } else {
                    String::new()
                };
                (graph_fires, ev)
            }
            CompiledPredicate::VersionGate {
                deprecated_name,
                replacement_name,
                ..
            } => {
                // Check if the deprecated name appears in the source code.
                let fires = context.source.contains(deprecated_name.as_str());
                let ev = if fires {
                    format!(
                        "deprecated '{}' found; use '{}' instead",
                        deprecated_name, replacement_name
                    )
                } else {
                    String::new()
                };
                (fires, ev)
            }
        };

        // Map fires → VerdictResult, then apply ESCALATE routing.
        let result = if !fires {
            VerdictResult::Allow
        } else if rule.severity == "ESCALATE" {
            let target = rule
                .escalate_target
                .clone()
                .unwrap_or(EscalationTarget::Auditor);
            VerdictResult::Escalate(target)
        } else {
            Self::severity_verdict(&rule.severity)
        };

        Ok(Verdict {
            rule_id: rule.rule_id.clone(),
            result,
            evidence,
        })
    }

    /// Check whether a child predicate "fires" (matches). Returns true if
    /// the child produced any non-Allow verdict, false if Allow.
    fn child_fires(
        &self,
        child: &CompiledPredicate,
        context: &EvaluationContext<'_>,
        rule: &Rule,
    ) -> Result<bool, RuleEngineError> {
        let v = self.evaluate_single(child, context, rule)?;
        Ok(v.result != VerdictResult::Allow)
    }

    /// Map the rule's severity string to the corresponding verdict.
    fn severity_verdict(severity: &str) -> VerdictResult {
        match severity {
            "BLOCK" => VerdictResult::Block,
            "WARN" => VerdictResult::Warn,
            _ => VerdictResult::Allow,
        }
    }

    /// Evaluate a composite predicate (And/Or/Not). Returns whether the
    /// composite fired (true) or not (false). Severity/ESCALATE mapping
    /// is applied by the caller in evaluate_single.
    fn evaluate_composite(
        &self,
        operator: &CompositeOperator,
        children: &[CompiledPredicate],
        context: &EvaluationContext<'_>,
        rule: &Rule,
    ) -> Result<bool, RuleEngineError> {
        match operator {
            CompositeOperator::And => {
                for child in children {
                    if !self.child_fires(child, context, rule)? {
                        return Ok(false);
                    }
                }
                Ok(!children.is_empty())
            }
            CompositeOperator::Or => {
                for child in children {
                    if self.child_fires(child, context, rule)? {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
            CompositeOperator::Not => {
                let child = children.first().ok_or_else(|| {
                    RuleEngineError::Predicate(PredicateError::ValidationFailed(
                        "Not operator requires exactly 1 child".into(),
                    ))
                })?;
                // Not inverts: child fires → composite does not fire.
                Ok(!self.child_fires(child, context, rule)?)
            }
        }
    }

    /// Recursively compile a parsed predicate tree.
    fn compile_parsed(&self, predicate: &Predicate) -> Result<CompiledPredicate, PredicateError> {
        match predicate {
            Predicate::MetricGate {
                metric_name,
                operator,
                threshold,
                source,
            } => Ok(CompiledPredicate::MetricGate {
                metric_name: metric_name.clone(),
                operator: operator.clone(),
                threshold: *threshold,
                source: source.clone(),
            }),
            Predicate::GrepPattern {
                pattern,
                scope,
                context_lines,
            } => {
                let compiled_regex = Regex::new(pattern).map_err(|e| {
                    PredicateError::ValidationFailed(format!("invalid regex: {e}"))
                })?;
                Ok(CompiledPredicate::GrepPattern {
                    compiled_regex,
                    scope: scope.clone(),
                    context_lines: *context_lines,
                })
            }
            Predicate::AstPattern {
                language,
                query,
                query_format,
                scope,
            } => {
                self.ast_evaluator
                    .validate_query(language, query)
                    .map_err(|e| {
                        PredicateError::ValidationFailed(format!("invalid AST query: {e}"))
                    })?;
                Ok(CompiledPredicate::AstPattern {
                    language: language.clone(),
                    query: query.clone(),
                    query_format: query_format.clone(),
                    scope: scope.clone(),
                })
            }
            Predicate::SchemaRequired {
                schema,
                output_type,
            } => {
                let validator =
                    jsonschema::Validator::new(schema).map_err(|e| {
                        PredicateError::ValidationFailed(format!("invalid JSON schema: {e}"))
                    })?;
                Ok(CompiledPredicate::SchemaRequired {
                    validator,
                    output_type: output_type.clone(),
                })
            }
            Predicate::Composite { operator, children } => {
                let compiled_children: Vec<CompiledPredicate> = children
                    .iter()
                    .map(|c| self.compile_parsed(c))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(CompiledPredicate::Composite {
                    operator: operator.clone(),
                    children: compiled_children,
                })
            }
            Predicate::GraphReachability {
                source_pattern,
                sink_pattern,
                must_traverse,
                must_not_exist,
                max_depth,
                edge_types,
            } => {
                let params = ReachabilityParams {
                    source_pattern: source_pattern.clone(),
                    sink_pattern: sink_pattern.clone(),
                    must_traverse: must_traverse.clone(),
                    must_not_exist: *must_not_exist,
                    max_depth: *max_depth,
                    edge_types: edge_types.clone(),
                };
                self.dkg_querier.validate_params(&params).map_err(|e| {
                    PredicateError::ValidationFailed(format!("invalid DKG params: {e}"))
                })?;
                Ok(CompiledPredicate::GraphReachability { params })
            }
            Predicate::VersionGate {
                ecosystem,
                deprecated_name,
                replacement_name,
                severity,
            } => {
                // Validation already done by predicate::validate (ecosystem, names, severity).
                Ok(CompiledPredicate::VersionGate {
                    ecosystem: ecosystem.clone(),
                    deprecated_name: deprecated_name.clone(),
                    replacement_name: replacement_name.clone(),
                    severity: severity.clone(),
                })
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::predicate::{
        AstError, AstMatch, DkgError, ReachabilityMatch, ReachabilityParams,
    };

    // -- Mock evaluators --

    /// AST evaluator that always succeeds with no matches.
    struct NoOpAstEvaluator;

    impl AstEvaluator for NoOpAstEvaluator {
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

    /// AST evaluator that panics on any call — used to prove short-circuit.
    struct PanickingAstEvaluator;

    impl AstEvaluator for PanickingAstEvaluator {
        fn evaluate_query(
            &self,
            _language: &str,
            _source: &str,
            _query: &str,
        ) -> Result<Vec<AstMatch>, AstError> {
            panic!("AstEvaluator should not be called — short-circuit failed")
        }

        fn validate_query(&self, _language: &str, _query: &str) -> Result<(), AstError> {
            panic!("AstEvaluator should not be called — short-circuit failed")
        }
    }

    /// DKG querier that always succeeds with no matches.
    struct NoOpDkgQuerier;

    impl DkgQuerier for NoOpDkgQuerier {
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

    /// DKG querier that rejects all params — used for compile_predicate error tests.
    struct RejectingDkgQuerier;

    impl DkgQuerier for RejectingDkgQuerier {
        fn query_reachability(
            &self,
            _params: &ReachabilityParams,
        ) -> Result<Vec<ReachabilityMatch>, DkgError> {
            Err(DkgError::QueryFailed("rejected".into()))
        }

        fn validate_params(&self, _params: &ReachabilityParams) -> Result<(), DkgError> {
            Err(DkgError::InvalidParams("always rejects".into()))
        }
    }

    fn default_engine() -> DefaultRuleEngine {
        DefaultRuleEngine::new(
            Box::new(NoOpAstEvaluator),
            Box::new(NoOpDkgQuerier),
        )
    }

    fn make_rule(id: &str, domain: &str, tier: u8, severity: &str, predicate_json: &str, phase_mask: u32) -> Rule {
        Rule {
            rule_id: id.into(),
            domain: domain.into(),
            tier,
            severity: severity.into(),
            title: format!("Rule {id}"),
            predicate_json: predicate_json.into(),
            phase_mask,
            escalate_target: None,
        }
    }

    fn make_context<'a>(source: &'a str, file_path: &'a str) -> EvaluationContext<'a> {
        EvaluationContext {
            action: "write",
            source,
            file_path,
            fsm_phase: FsmPhaseBit::EXECUTING,
            metric_values: HashMap::new(),
        }
    }

    // -- FsmPhaseBit --

    #[test]
    fn fsm_phase_bit_constants() {
        assert_eq!(FsmPhaseBit::PLANNING.bit(), 0);
        assert_eq!(FsmPhaseBit::FEASIBILITY_PROBE.bit(), 1);
        assert_eq!(FsmPhaseBit::TESTING_FIRST.bit(), 2);
        assert_eq!(FsmPhaseBit::EXECUTING.bit(), 3);
        assert_eq!(FsmPhaseBit::AUDITING.bit(), 4);
        assert_eq!(FsmPhaseBit::MEMORIZING.bit(), 5);
        assert_eq!(FsmPhaseBit::CONFIGURING.bit(), 6);
    }

    #[test]
    fn fsm_phase_bit_mask() {
        assert_eq!(FsmPhaseBit::PLANNING.mask(), 1);
        assert_eq!(FsmPhaseBit::EXECUTING.mask(), 8);
        assert_eq!(FsmPhaseBit::CONFIGURING.mask(), 64);
    }

    #[test]
    fn fsm_phase_bit_from_raw_valid() {
        for i in 0..=6 {
            assert!(FsmPhaseBit::from_raw(i).is_some());
        }
    }

    #[test]
    fn fsm_phase_bit_from_raw_invalid() {
        assert!(FsmPhaseBit::from_raw(7).is_none());
        assert!(FsmPhaseBit::from_raw(255).is_none());
    }

    // -- compile_predicate: valid cases --

    #[test]
    fn compile_valid_grep_pattern() {
        let engine = default_engine();
        let json = r#"{"type":"grep_pattern","pattern":"TODO","scope":null,"context_lines":3}"#;
        let result = engine.compile_predicate(json);
        assert!(result.is_ok());
    }

    #[test]
    fn compile_valid_metric_gate() {
        let engine = default_engine();
        let json = r#"{"type":"metric_gate","metric_name":"cc","operator":"gt","threshold":10.0,"source":"file"}"#;
        let result = engine.compile_predicate(json);
        assert!(result.is_ok());
    }

    #[test]
    fn compile_valid_ast_pattern() {
        let engine = default_engine();
        let json = r#"{"type":"ast_pattern","language":"python","query":"(function_definition)","query_format":"sexp","scope":null}"#;
        let result = engine.compile_predicate(json);
        assert!(result.is_ok());
    }

    #[test]
    fn compile_valid_schema_required() {
        let engine = default_engine();
        let json = r#"{"type":"schema_required","schema":{"type":"object","required":["name"]},"output_type":"json"}"#;
        let result = engine.compile_predicate(json);
        assert!(result.is_ok());
    }

    #[test]
    fn compile_valid_graph_reachability() {
        let engine = default_engine();
        let json = r#"{"type":"graph_reachability","source_pattern":"user_input","sink_pattern":"db_query","must_traverse":null,"must_not_exist":true,"max_depth":5,"edge_types":["call"]}"#;
        let result = engine.compile_predicate(json);
        assert!(result.is_ok());
    }

    // -- ADVERSARIAL: 10 known-invalid regex patterns rejected --

    #[test]
    fn compile_rejects_10_invalid_regex_patterns() {
        let engine = default_engine();
        let invalid_patterns = [
            r"[unclosed",
            r"(unclosed",
            r"*leading_quantifier",
            r"+leading_quantifier",
            r"?leading_quantifier",
            r"(?P<dup>x)(?P<dup>y)",
            r"[z-a]",
            r"(?i)(?-i",
            r"\p{Invalid_Unicode_Category}",
            r"a{999999999999}",
        ];

        for pattern in &invalid_patterns {
            let json = format!(
                r#"{{"type":"grep_pattern","pattern":"{}","scope":null,"context_lines":0}}"#,
                pattern.replace('\\', "\\\\").replace('"', "\\\"")
            );
            let result = engine.compile_predicate(&json);
            assert!(
                result.is_err(),
                "expected invalid regex to be rejected: {pattern}"
            );
        }
    }

    // -- ADVERSARIAL: invalid JSON schema rejected --

    #[test]
    fn compile_rejects_invalid_json_schema() {
        let engine = default_engine();
        // "type" must be a valid JSON Schema type; "invalid_type" is not.
        let json = r#"{"type":"schema_required","schema":{"type":"invalid_type"},"output_type":"json"}"#;
        let result = engine.compile_predicate(json);
        assert!(result.is_err(), "invalid JSON schema must be rejected");
    }

    // -- ADVERSARIAL: depth > 8 rejected --

    #[test]
    fn compile_rejects_composite_depth_exceeding_limit() {
        let engine = default_engine();
        // Build depth-9 JSON.
        let mut json = r#"{"type":"grep_pattern","pattern":"x","scope":null,"context_lines":0}"#.to_string();
        for _ in 0..9 {
            json = format!(
                r#"{{"type":"composite","operator":"and","children":[{json}]}}"#
            );
        }
        let result = engine.compile_predicate(&json);
        assert!(result.is_err(), "depth>8 composite must be rejected");
    }

    // -- ADVERSARIAL: dyn RuleEngine box compiles (object-safety proof) --

    #[test]
    fn dyn_rule_engine_object_safety() {
        let engine = default_engine();
        let _boxed: Box<dyn RuleEngine> = Box::new(engine);
        // If this compiles, the trait is object-safe.
    }

    // -- ADVERSARIAL: short-circuit test (panicking mock + GREP blocks first) --

    #[test]
    fn evaluate_short_circuits_on_block_before_ast() {
        // Use PanickingAstEvaluator — if evaluate() calls it, the test panics.
        let engine = DefaultRuleEngine::new(
            Box::new(PanickingAstEvaluator),
            Box::new(NoOpDkgQuerier),
        );

        let grep_json = r#"{"type":"grep_pattern","pattern":"DANGER","scope":null,"context_lines":0}"#;
        let grep_compiled = engine.compile_predicate(grep_json).unwrap();

        // We can't compile AST with panicking evaluator, so create manually.
        let ast_compiled = CompiledPredicate::AstPattern {
            language: "python".into(),
            query: "(function_definition)".into(),
            query_format: QueryFormat::Sexp,
            scope: None,
        };

        let rules_with_compiled = vec![
            (
                make_rule("r-grep", "security", 1, "BLOCK", grep_json, 0xFF),
                grep_compiled,
            ),
            (
                make_rule("r-ast", "security", 1, "BLOCK", "", 0xFF),
                ast_compiled,
            ),
        ];

        let context = make_context("DANGER code here", "src/main.py");
        let result = engine.evaluate(&rules_with_compiled, &context).unwrap();

        // Must have a Block verdict and must not have panicked.
        let has_block = result.iter().any(|v| v.result == VerdictResult::Block);
        assert!(has_block, "must block on DANGER pattern");
    }

    // -- ADVERSARIAL: get_active_rules filters by phase_mask bitmask --

    #[test]
    fn get_active_rules_filters_by_phase_mask() {
        let engine = default_engine();
        let rules = vec![
            make_rule("r1", "engineering", 1, "BLOCK", "", 0b0000_0001), // PLANNING only
            make_rule("r2", "engineering", 1, "BLOCK", "", 0b0000_1000), // EXECUTING only
            make_rule("r3", "engineering", 1, "BLOCK", "", 0b1111_1111), // all phases
        ];

        let active = engine.get_active_rules(&rules, FsmPhaseBit::PLANNING, &[]);
        let ids: Vec<&str> = active.iter().map(|r| r.rule_id.as_str()).collect();
        assert!(ids.contains(&"r1"), "PLANNING-masked rule must be included");
        assert!(!ids.contains(&"r2"), "EXECUTING-only rule must NOT be included for PLANNING");
        assert!(ids.contains(&"r3"), "all-phases rule must be included");
    }

    // -- ADVERSARIAL: engineering/security always included --

    #[test]
    fn get_active_rules_always_includes_engineering_and_security() {
        let engine = default_engine();
        let rules = vec![
            make_rule("r-eng", "engineering", 1, "BLOCK", "", 0xFF),
            make_rule("r-sec", "security", 1, "BLOCK", "", 0xFF),
            make_rule("r-trading", "trading", 1, "BLOCK", "", 0xFF),
        ];

        // file_domains does NOT include engineering or security
        let active = engine.get_active_rules(&rules, FsmPhaseBit::EXECUTING, &["trading"]);
        let ids: Vec<&str> = active.iter().map(|r| r.rule_id.as_str()).collect();
        assert!(ids.contains(&"r-eng"), "engineering must always be included");
        assert!(ids.contains(&"r-sec"), "security must always be included");
        assert!(ids.contains(&"r-trading"), "trading is in file_domains");
    }

    #[test]
    fn get_active_rules_excludes_non_matching_domain() {
        let engine = default_engine();
        let rules = vec![
            make_rule("r-game", "gaming", 1, "BLOCK", "", 0xFF),
        ];

        let active = engine.get_active_rules(&rules, FsmPhaseBit::EXECUTING, &["trading"]);
        assert!(active.is_empty(), "gaming domain not in file_domains and not always-included");
    }

    // -- ADVERSARIAL: evaluate accumulates Warns before returning Block --

    #[test]
    fn evaluate_accumulates_warns_before_block() {
        let engine = default_engine();

        let warn_json = r#"{"type":"grep_pattern","pattern":"TODO","scope":null,"context_lines":0}"#;
        let warn_compiled = engine.compile_predicate(warn_json).unwrap();

        let block_json = r#"{"type":"grep_pattern","pattern":"FIXME","scope":null,"context_lines":0}"#;
        let block_compiled = engine.compile_predicate(block_json).unwrap();

        // Warn rule at tier=1 sorts before Block rule at tier=2,
        // so Warn is evaluated first and accumulated before Block short-circuits.
        let rules_with_compiled = vec![
            (
                make_rule("r-warn", "engineering", 1, "WARN", warn_json, 0xFF),
                warn_compiled,
            ),
            (
                make_rule("r-block", "engineering", 2, "BLOCK", block_json, 0xFF),
                block_compiled,
            ),
        ];

        let context = make_context("TODO: fix this\nFIXME: urgent", "src/main.rs");
        let result = engine.evaluate(&rules_with_compiled, &context).unwrap();

        // Should have Warn then Block.
        assert!(result.len() >= 2, "must have at least 2 verdicts");
        let warn_count = result.iter().filter(|v| v.result == VerdictResult::Warn).count();
        let block_count = result.iter().filter(|v| v.result == VerdictResult::Block).count();
        assert!(warn_count >= 1, "must have accumulated at least 1 Warn");
        assert_eq!(block_count, 1, "must have exactly 1 Block");
    }

    // -- Ordering tests --

    #[test]
    fn get_active_rules_ordered_by_tier_then_severity() {
        let engine = default_engine();
        let rules = vec![
            make_rule("r-t2-warn", "engineering", 2, "WARN", "", 0xFF),
            make_rule("r-t1-warn", "engineering", 1, "WARN", "", 0xFF),
            make_rule("r-t1-block", "engineering", 1, "BLOCK", "", 0xFF),
        ];

        let active = engine.get_active_rules(&rules, FsmPhaseBit::EXECUTING, &[]);
        let ids: Vec<&str> = active.iter().map(|r| r.rule_id.as_str()).collect();
        assert_eq!(ids, vec!["r-t1-block", "r-t1-warn", "r-t2-warn"]);
    }

    // -- compile_predicate error propagation --

    #[test]
    fn compile_rejects_invalid_json() {
        let engine = default_engine();
        let result = engine.compile_predicate("not json");
        assert!(result.is_err());
    }

    #[test]
    fn compile_rejects_invalid_dkg_params() {
        let engine = DefaultRuleEngine::new(
            Box::new(NoOpAstEvaluator),
            Box::new(RejectingDkgQuerier),
        );
        let json = r#"{"type":"graph_reachability","source_pattern":"a","sink_pattern":"b","must_traverse":null,"must_not_exist":false,"max_depth":5,"edge_types":["call"]}"#;
        let result = engine.compile_predicate(json);
        assert!(result.is_err(), "rejecting DKG querier must cause compile_predicate to fail");
    }

    // -- Composite compilation --

    #[test]
    fn compile_nested_composite_valid() {
        let engine = default_engine();
        let json = r#"{"type":"composite","operator":"and","children":[{"type":"grep_pattern","pattern":"x","scope":null,"context_lines":0}]}"#;
        let result = engine.compile_predicate(json);
        assert!(result.is_ok());
    }

    // -- evaluate: Allow when no match --

    #[test]
    fn evaluate_returns_allow_when_no_match() {
        let engine = default_engine();
        let json = r#"{"type":"grep_pattern","pattern":"DANGER","scope":null,"context_lines":0}"#;
        let compiled = engine.compile_predicate(json).unwrap();

        let rules_with_compiled = vec![
            (make_rule("r1", "engineering", 1, "BLOCK", json, 0xFF), compiled),
        ];

        let context = make_context("safe code here", "src/main.rs");
        let result = engine.evaluate(&rules_with_compiled, &context).unwrap();
        assert!(
            result.iter().all(|v| v.result == VerdictResult::Allow),
            "no match should produce Allow"
        );
    }

    // -- evaluate: scope filtering --

    #[test]
    fn evaluate_grep_respects_scope() {
        let engine = default_engine();
        let json = r#"{"type":"grep_pattern","pattern":"DANGER","scope":"tests/","context_lines":0}"#;
        let compiled = engine.compile_predicate(json).unwrap();

        let rules_with_compiled = vec![
            (make_rule("r1", "engineering", 1, "BLOCK", json, 0xFF), compiled),
        ];

        // File outside scope — should Allow.
        let context = make_context("DANGER", "src/main.rs");
        let result = engine.evaluate(&rules_with_compiled, &context).unwrap();
        assert!(
            result.iter().all(|v| v.result == VerdictResult::Allow),
            "out-of-scope file should not trigger"
        );
    }

    // -- from_json is used (not raw serde) --

    #[test]
    fn compile_uses_from_json_not_raw_serde() {
        // Prove: Not with 0 children is rejected by from_json's validate step.
        let engine = default_engine();
        let json = r#"{"type":"composite","operator":"not","children":[]}"#;
        let result = engine.compile_predicate(json);
        assert!(result.is_err(), "compile_predicate must use from_json which validates Not arity");
    }

    // -- EvaluationContext construction --

    #[test]
    fn evaluation_context_fields() {
        let ctx = EvaluationContext {
            action: "write",
            source: "fn main() {}",
            file_path: "src/main.rs",
            fsm_phase: FsmPhaseBit::EXECUTING,
            metric_values: HashMap::new(),
        };
        assert_eq!(ctx.action, "write");
        assert_eq!(ctx.fsm_phase, FsmPhaseBit::EXECUTING);
        assert!(ctx.metric_values.is_empty());
    }

    // -- Rule struct --

    #[test]
    fn rule_serde_round_trip() {
        let rule = make_rule("r-1", "engineering", 1, "BLOCK", "{}", 0xFF);
        let json = serde_json::to_string(&rule).unwrap();
        let rt: Rule = serde_json::from_str(&json).unwrap();
        assert_eq!(rule.rule_id, rt.rule_id);
        assert_eq!(rule.phase_mask, rt.phase_mask);
    }

    // -- eval_order --

    #[test]
    fn eval_order_values() {
        let metric = CompiledPredicate::MetricGate {
            metric_name: "x".into(),
            operator: MetricOperator::Gt,
            threshold: Some(1.0),
            source: MetricSource::File,
        };
        let grep = CompiledPredicate::GrepPattern {
            compiled_regex: Regex::new("x").unwrap(),
            scope: None,
            context_lines: 0,
        };
        let composite = CompiledPredicate::Composite {
            operator: CompositeOperator::And,
            children: vec![],
        };

        assert!(eval_order(&metric) < eval_order(&grep));
        assert!(eval_order(&grep) < eval_order(&composite));
    }

    // -- DefaultRuleEngine::new --

    #[test]
    fn default_engine_construction() {
        let _engine = default_engine();
        // Compiles and doesn't panic.
    }

    // -- Empty rules evaluation --

    #[test]
    fn evaluate_empty_rules_returns_empty() {
        let engine = default_engine();
        let context = make_context("code", "src/main.rs");
        let result = engine.evaluate(&[], &context).unwrap();
        assert!(result.is_empty());
    }

    // -- SchemaRequired evaluation --

    #[test]
    fn evaluate_schema_required_valid_json() {
        let engine = default_engine();
        let json = r#"{"type":"schema_required","schema":{"type":"object","required":["name"]},"output_type":"json"}"#;
        let compiled = engine.compile_predicate(json).unwrap();

        let rules_with_compiled = vec![
            (make_rule("r1", "engineering", 1, "BLOCK", json, 0xFF), compiled),
        ];

        // Source matches schema
        let context = make_context(r#"{"name":"test"}"#, "config.json");
        let result = engine.evaluate(&rules_with_compiled, &context).unwrap();
        assert!(result.iter().all(|v| v.result == VerdictResult::Allow));
    }

    #[test]
    fn evaluate_schema_required_invalid_json() {
        let engine = default_engine();
        let json = r#"{"type":"schema_required","schema":{"type":"object","required":["name"]},"output_type":"json"}"#;
        let compiled = engine.compile_predicate(json).unwrap();

        let rules_with_compiled = vec![
            (make_rule("r1", "engineering", 1, "BLOCK", json, 0xFF), compiled),
        ];

        // Source does NOT match schema (missing "name")
        let context = make_context(r#"{"other":"test"}"#, "config.json");
        let result = engine.evaluate(&rules_with_compiled, &context).unwrap();
        let has_block = result.iter().any(|v| v.result == VerdictResult::Block);
        assert!(has_block, "schema violation must produce Block");
    }

    // -- BLOCK fix: SchemaRequired on non-JSON source returns Allow, not error --

    #[test]
    fn evaluate_schema_required_non_json_source_returns_allow() {
        let engine = default_engine();
        let schema_json = r#"{"type":"schema_required","schema":{"type":"object"},"output_type":"json"}"#;
        let schema_compiled = engine.compile_predicate(schema_json).unwrap();

        let grep_json = r#"{"type":"grep_pattern","pattern":"DANGER","scope":null,"context_lines":0}"#;
        let grep_compiled = engine.compile_predicate(grep_json).unwrap();

        // Schema rule first, then grep rule — schema must NOT abort pipeline.
        let rules_with_compiled = vec![
            (make_rule("r-schema", "engineering", 1, "BLOCK", schema_json, 0xFF), schema_compiled),
            (make_rule("r-grep", "security", 2, "BLOCK", grep_json, 0xFF), grep_compiled),
        ];

        // Source is Rust code (not JSON) — schema rule should Allow, grep should Block.
        let context = make_context("fn main() { DANGER }", "src/main.rs");
        let result = engine.evaluate(&rules_with_compiled, &context).unwrap();
        let has_block = result.iter().any(|v| v.result == VerdictResult::Block && v.rule_id == "r-grep");
        assert!(has_block, "grep rule must still run after schema rule on non-JSON source");
    }

    // -- BLOCK fix: And composite fires when all children are Warn --

    #[test]
    fn evaluate_and_composite_fires_when_all_children_warn() {
        let engine = default_engine();

        let child1_json = r#"{"type":"grep_pattern","pattern":"TODO","scope":null,"context_lines":0}"#;
        let child2_json = r#"{"type":"grep_pattern","pattern":"FIXME","scope":null,"context_lines":0}"#;

        let child1 = engine.compile_predicate(child1_json).unwrap();
        let child2 = engine.compile_predicate(child2_json).unwrap();

        let composite = CompiledPredicate::Composite {
            operator: CompositeOperator::And,
            children: vec![child1, child2],
        };

        // Outer rule severity is WARN — And should produce Warn when all children fire.
        let rules_with_compiled = vec![
            (make_rule("r-and", "engineering", 1, "WARN", "", 0xFF), composite),
        ];

        let context = make_context("TODO: fix\nFIXME: urgent", "src/main.rs");
        let result = engine.evaluate(&rules_with_compiled, &context).unwrap();
        let has_warn = result.iter().any(|v| v.result == VerdictResult::Warn);
        assert!(has_warn, "And composite must fire (Warn) when all children match");
    }

    #[test]
    fn evaluate_and_composite_allows_when_one_child_does_not_match() {
        let engine = default_engine();

        let child1_json = r#"{"type":"grep_pattern","pattern":"TODO","scope":null,"context_lines":0}"#;
        let child2_json = r#"{"type":"grep_pattern","pattern":"NONEXISTENT","scope":null,"context_lines":0}"#;

        let child1 = engine.compile_predicate(child1_json).unwrap();
        let child2 = engine.compile_predicate(child2_json).unwrap();

        let composite = CompiledPredicate::Composite {
            operator: CompositeOperator::And,
            children: vec![child1, child2],
        };

        let rules_with_compiled = vec![
            (make_rule("r-and", "engineering", 1, "BLOCK", "", 0xFF), composite),
        ];

        let context = make_context("TODO: fix this", "src/main.rs");
        let result = engine.evaluate(&rules_with_compiled, &context).unwrap();
        assert!(
            result.iter().all(|v| v.result == VerdictResult::Allow),
            "And must Allow when one child does not match"
        );
    }

    // -- MetricGate real evaluation --

    #[test]
    fn evaluate_metric_gate_gt_fires_when_above_threshold() {
        let engine = default_engine();
        let json = r#"{"type":"metric_gate","metric_name":"cc","operator":"gt","threshold":10.0,"source":"file"}"#;
        let compiled = engine.compile_predicate(json).unwrap();

        let rule = make_rule("r-metric", "engineering", 1, "BLOCK", json, 0xFF);
        let mut ctx = make_context("", "src/main.rs");
        ctx.metric_values.insert("cc".into(), 15.0);

        let result = engine.evaluate(&[(rule, compiled)], &ctx).unwrap();
        let has_block = result.iter().any(|v| v.result == VerdictResult::Block);
        assert!(has_block, "MetricGate Gt must fire when value > threshold");
    }

    #[test]
    fn evaluate_metric_gate_lt_does_not_fire_when_above() {
        let engine = default_engine();
        let json = r#"{"type":"metric_gate","metric_name":"loc","operator":"lt","threshold":100.0,"source":"file"}"#;
        let compiled = engine.compile_predicate(json).unwrap();

        let rule = make_rule("r-metric", "engineering", 1, "BLOCK", json, 0xFF);
        let mut ctx = make_context("", "src/main.rs");
        ctx.metric_values.insert("loc".into(), 200.0);

        let result = engine.evaluate(&[(rule, compiled)], &ctx).unwrap();
        assert!(
            result.iter().all(|v| v.result == VerdictResult::Allow),
            "MetricGate Lt must not fire when value > threshold"
        );
    }

    #[test]
    fn evaluate_metric_gate_eq_fires_on_exact_match() {
        let engine = default_engine();
        let json = r#"{"type":"metric_gate","metric_name":"ratio","operator":"eq","threshold":1.0,"source":"config"}"#;
        let compiled = engine.compile_predicate(json).unwrap();

        let rule = make_rule("r-metric", "engineering", 1, "WARN", json, 0xFF);
        let mut ctx = make_context("", "src/main.rs");
        ctx.metric_values.insert("ratio".into(), 1.0);

        let result = engine.evaluate(&[(rule, compiled)], &ctx).unwrap();
        let has_warn = result.iter().any(|v| v.result == VerdictResult::Warn);
        assert!(has_warn, "MetricGate Eq must fire on exact match");
    }

    #[test]
    fn evaluate_metric_gate_between_fires_in_range() {
        let engine = default_engine();
        let json = r#"{"type":"metric_gate","metric_name":"cc","operator":{"between":{"low":1.0,"high":10.0}},"threshold":null,"source":"file"}"#;
        let compiled = engine.compile_predicate(json).unwrap();

        let rule = make_rule("r-metric", "engineering", 1, "WARN", json, 0xFF);
        let mut ctx = make_context("", "src/main.rs");
        ctx.metric_values.insert("cc".into(), 5.0);

        let result = engine.evaluate(&[(rule, compiled)], &ctx).unwrap();
        let has_warn = result.iter().any(|v| v.result == VerdictResult::Warn);
        assert!(has_warn, "MetricGate Between must fire when value in range");
    }

    #[test]
    fn evaluate_metric_gate_missing_key_returns_allow() {
        let engine = default_engine();
        let json = r#"{"type":"metric_gate","metric_name":"missing","operator":"gt","threshold":1.0,"source":"file"}"#;
        let compiled = engine.compile_predicate(json).unwrap();

        let rule = make_rule("r-metric", "engineering", 1, "BLOCK", json, 0xFF);
        let ctx = make_context("", "src/main.rs");

        let result = engine.evaluate(&[(rule, compiled)], &ctx).unwrap();
        assert!(
            result.iter().all(|v| v.result == VerdictResult::Allow),
            "missing metric key must return Allow"
        );
        let evidence = &result[0].evidence;
        assert!(
            evidence.contains("not in context"),
            "evidence must explain missing key"
        );
    }

    // -- ESCALATE routing --

    #[test]
    fn evaluate_escalate_routes_to_auditor_by_default() {
        let engine = default_engine();
        let json = r#"{"type":"grep_pattern","pattern":"DANGER","scope":null,"context_lines":0}"#;
        let compiled = engine.compile_predicate(json).unwrap();

        let mut rule = make_rule("r-esc", "security", 1, "ESCALATE", json, 0xFF);
        rule.escalate_target = None;

        let ctx = make_context("DANGER code", "src/main.rs");
        let result = engine.evaluate(&[(rule, compiled)], &ctx).unwrap();
        assert_eq!(
            result[0].result,
            VerdictResult::Escalate(EscalationTarget::Auditor),
            "ESCALATE with no target must route to Auditor"
        );
    }

    #[test]
    fn evaluate_escalate_routes_to_human() {
        let engine = default_engine();
        let json = r#"{"type":"grep_pattern","pattern":"DANGER","scope":null,"context_lines":0}"#;
        let compiled = engine.compile_predicate(json).unwrap();

        let mut rule = make_rule("r-esc", "security", 1, "ESCALATE", json, 0xFF);
        rule.escalate_target = Some(EscalationTarget::Human);

        let ctx = make_context("DANGER code", "src/main.rs");
        let result = engine.evaluate(&[(rule, compiled)], &ctx).unwrap();
        assert_eq!(
            result[0].result,
            VerdictResult::Escalate(EscalationTarget::Human),
            "ESCALATE must route to Human when specified"
        );
    }

    #[test]
    fn evaluate_escalate_does_not_fire_on_allow() {
        let engine = default_engine();
        let json = r#"{"type":"grep_pattern","pattern":"NONEXISTENT","scope":null,"context_lines":0}"#;
        let compiled = engine.compile_predicate(json).unwrap();

        let mut rule = make_rule("r-esc", "security", 1, "ESCALATE", json, 0xFF);
        rule.escalate_target = Some(EscalationTarget::Human);

        let ctx = make_context("safe code", "src/main.rs");
        let result = engine.evaluate(&[(rule, compiled)], &ctx).unwrap();
        assert_eq!(
            result[0].result,
            VerdictResult::Allow,
            "ESCALATE must not fire when predicate does not match"
        );
    }

    // -- Evidence strings --

    #[test]
    fn evaluate_grep_evidence_contains_offset() {
        let engine = default_engine();
        let json = r#"{"type":"grep_pattern","pattern":"DANGER","scope":null,"context_lines":0}"#;
        let compiled = engine.compile_predicate(json).unwrap();

        let rule = make_rule("r1", "engineering", 1, "BLOCK", json, 0xFF);
        let ctx = make_context("safe DANGER here", "src/main.rs");

        let result = engine.evaluate(&[(rule, compiled)], &ctx).unwrap();
        let verdict = &result[0];
        assert!(!verdict.evidence.is_empty(), "evidence must not be empty");
        assert!(
            verdict.evidence.contains("grep match at offset"),
            "evidence must contain offset info"
        );
    }

    #[test]
    fn evaluate_schema_evidence_on_violation() {
        let engine = default_engine();
        let json = r#"{"type":"schema_required","schema":{"type":"object","required":["name"]},"output_type":"json"}"#;
        let compiled = engine.compile_predicate(json).unwrap();

        let rule = make_rule("r1", "engineering", 1, "BLOCK", json, 0xFF);
        let ctx = make_context(r#"{"other":"test"}"#, "config.json");

        let result = engine.evaluate(&[(rule, compiled)], &ctx).unwrap();
        let verdict = &result[0];
        assert!(
            verdict.evidence.contains("schema validation failed"),
            "schema violation evidence must describe failure"
        );
    }

    // -- 100x determinism --

    #[test]
    fn test_evaluate_single_deterministic_100x() {
        let engine = default_engine();
        let json = r#"{"type":"grep_pattern","pattern":"DANGER","scope":null,"context_lines":0}"#;
        let compiled = engine.compile_predicate(json).unwrap();

        let rule = make_rule("r-det", "security", 1, "BLOCK", json, 0xFF);
        let ctx = make_context("DANGER code here", "src/main.rs");

        let rules_with_compiled = [(rule.clone(), compiled)];
        let first = engine.evaluate(&rules_with_compiled, &ctx).unwrap();

        for i in 1..100 {
            let rule_clone = rule.clone();
            let compiled_clone = engine.compile_predicate(json).unwrap();
            let result = engine.evaluate(&[(rule_clone, compiled_clone)], &ctx).unwrap();
            assert_eq!(
                first[0].result, result[0].result,
                "determinism failed at iteration {i}: result mismatch"
            );
            assert_eq!(
                first[0].evidence, result[0].evidence,
                "determinism failed at iteration {i}: evidence mismatch"
            );
        }
    }

    // -- ReDoS safety (regex crate guarantees linear time) --

    #[test]
    fn test_grep_redos_pattern_completes() {
        let engine = default_engine();
        // Classic ReDoS pattern — regex crate handles this in linear time.
        let json = r#"{"type":"grep_pattern","pattern":"(a+)+$","scope":null,"context_lines":0}"#;
        let compiled = engine.compile_predicate(json).unwrap();

        let rule = make_rule("r-redos", "security", 1, "BLOCK", json, 0xFF);
        let source = "a".repeat(100);
        let ctx = make_context(&source, "src/main.rs");

        let start = std::time::Instant::now(); // TEST_KEEP
        let result = engine.evaluate(&[(rule, compiled)], &ctx);
        let elapsed = start.elapsed();

        assert!(result.is_ok(), "ReDoS pattern must complete without error");
        assert!(
            elapsed.as_secs() < 2,
            "regex crate must handle ReDoS pattern in linear time (took {:?})",
            elapsed
        );
    }

    // -- Composite depth 100 rejection --

    #[test]
    fn test_composite_depth_100_returns_err() {
        let engine = default_engine();
        let mut json =
            r#"{"type":"grep_pattern","pattern":"x","scope":null,"context_lines":0}"#.to_string();
        for _ in 0..100 {
            json = format!(
                r#"{{"type":"composite","operator":"and","children":[{json}]}}"#
            );
        }
        let result = engine.compile_predicate(&json);
        assert!(result.is_err(), "depth=100 composite must be rejected");
    }

    // -- ESCALATE + Composite --

    #[test]
    fn evaluate_escalate_composite_routes_correctly() {
        let engine = default_engine();

        let child1_json = r#"{"type":"grep_pattern","pattern":"TODO","scope":null,"context_lines":0}"#;
        let child2_json = r#"{"type":"grep_pattern","pattern":"FIXME","scope":null,"context_lines":0}"#;

        let child1 = engine.compile_predicate(child1_json).unwrap();
        let child2 = engine.compile_predicate(child2_json).unwrap();

        let composite = CompiledPredicate::Composite {
            operator: CompositeOperator::And,
            children: vec![child1, child2],
        };

        let mut rule = make_rule("r-esc-comp", "engineering", 1, "ESCALATE", "", 0xFF);
        rule.escalate_target = Some(EscalationTarget::Human);

        let ctx = make_context("TODO FIXME", "src/main.rs");
        let result = engine.evaluate(&[(rule, composite)], &ctx).unwrap();
        assert_eq!(
            result[0].result,
            VerdictResult::Escalate(EscalationTarget::Human),
            "ESCALATE + Composite must route to configured target"
        );
    }

    // -- ESCALATE with NextTierRule --

    #[test]
    fn evaluate_escalate_routes_to_next_tier_rule() {
        let engine = default_engine();
        let json = r#"{"type":"grep_pattern","pattern":"DANGER","scope":null,"context_lines":0}"#;
        let compiled = engine.compile_predicate(json).unwrap();

        let mut rule = make_rule("r-esc", "security", 1, "ESCALATE", json, 0xFF);
        rule.escalate_target = Some(EscalationTarget::NextTierRule("r-99".into()));

        let ctx = make_context("DANGER code", "src/main.rs");
        let result = engine.evaluate(&[(rule, compiled)], &ctx).unwrap();
        assert_eq!(
            result[0].result,
            VerdictResult::Escalate(EscalationTarget::NextTierRule("r-99".into())),
            "ESCALATE must route to NextTierRule when specified"
        );
    }

    // -- detect_domains unit tests --

    #[test]
    fn detect_domains_trading_keywords() {
        for kw in ["trading", "backtest", "signal", "pipeline"] {
            let path = format!("src/{kw}/main.rs");
            let domains = super::detect_domains(&path);
            assert!(
                domains.contains(&"trading".to_owned()),
                "keyword '{kw}' must produce domain 'trading', got: {domains:?}"
            );
        }
    }

    #[test]
    fn detect_domains_dashboard_keywords() {
        for kw in ["dashboard", "component", "chart", "ui"] {
            let path = format!("src/{kw}/main.rs");
            let domains = super::detect_domains(&path);
            assert!(
                domains.contains(&"dashboard".to_owned()),
                "keyword '{kw}' must produce domain 'dashboard', got: {domains:?}"
            );
        }
    }

    #[test]
    fn detect_domains_application_keywords() {
        for kw in ["payment", "billing", "tenant", "saas"] {
            let path = format!("src/{kw}/main.rs");
            let domains = super::detect_domains(&path);
            assert!(
                domains.contains(&"application".to_owned()),
                "keyword '{kw}' must produce domain 'application', got: {domains:?}"
            );
        }
    }

    #[test]
    fn detect_domains_game_keywords() {
        for kw in ["game", "mod", "entity", "spawn"] {
            let path = format!("src/{kw}/main.rs");
            let domains = super::detect_domains(&path);
            assert!(
                domains.contains(&"game".to_owned()),
                "keyword '{kw}' must produce domain 'game', got: {domains:?}"
            );
        }
    }

    #[test]
    fn detect_domains_re_keywords() {
        for kw in ["ida", "ghidra", "disasm", "hook"] {
            let path = format!("src/{kw}/main.rs");
            let domains = super::detect_domains(&path);
            assert!(
                domains.contains(&"re".to_owned()),
                "keyword '{kw}' must produce domain 're', got: {domains:?}"
            );
        }
    }

    #[test]
    fn detect_domains_case_insensitive() {
        let domains = super::detect_domains("src/Trading/Backtest.rs");
        assert!(
            domains.contains(&"trading".to_owned()),
            "uppercase 'Trading/Backtest' must still match, got: {domains:?}"
        );
        // Must be deduplicated — both keywords match "trading" domain
        assert_eq!(
            domains.iter().filter(|d| d.as_str() == "trading").count(),
            1,
            "domain 'trading' must appear exactly once"
        );
    }

    #[test]
    fn detect_domains_empty_path() {
        let domains = super::detect_domains("");
        assert!(domains.is_empty(), "empty path must return empty vec");
    }

    #[test]
    fn detect_domains_no_match() {
        let domains = super::detect_domains("src/lib.rs");
        assert!(
            domains.is_empty(),
            "path with no keywords must return empty vec, got: {domains:?}"
        );
    }

    #[test]
    fn detect_domains_multi_domain() {
        let domains = super::detect_domains("trading_dashboard_module.rs");
        assert!(
            domains.contains(&"trading".to_owned()),
            "must detect 'trading', got: {domains:?}"
        );
        assert!(
            domains.contains(&"dashboard".to_owned()),
            "must detect 'dashboard', got: {domains:?}"
        );
    }
}

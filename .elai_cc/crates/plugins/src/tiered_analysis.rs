//! Tiered static analysis hooks for SSB write validation.
//!
//! Implements three tiers of deterministic validation:
//! - T1 (RULES, <1ms): Predicate evaluation from rules.db on every SSB write
//! - T2 (AST, <10ms): tree-sitter parse for file_write actions only
//! - T3 (PATTERN MATCH, <5ms): Known-ambiguous pattern matching when WARN exists
//!
//! No LLM in the validation hot path. All findings written to SSB as 'finding' entries.

use blake3;
use elai_rules::predicate::VerdictResult;
use elai_rules::rule_engine::{EvaluationContext, FsmPhaseBit, RuleEngine};
use std::collections::HashMap;
use std::sync::LazyLock;
use std::time::Instant;

// ---------------------------------------------------------------------------
// Hook verdict types
// ---------------------------------------------------------------------------

/// Verdict from a tiered analysis hook.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HookVerdict {
    Allow,
    Warn { message: String, pattern_hash: String },
    Block { message: String, pattern_hash: String },
}

impl HookVerdict {
    /// Returns true if this verdict blocks the action.
    pub fn is_block(&self) -> bool {
        matches!(self, Self::Block { .. })
    }

    /// Returns true if this verdict is a warning.
    pub fn is_warn(&self) -> bool {
        matches!(self, Self::Warn { .. })
    }

    /// Converts to a generic error message for agents (sanitization).
    pub fn to_agent_error(&self) -> String {
        match self {
            Self::Block { .. } => "Tool call rejected by policy".to_string(),
            Self::Warn { .. } => "Tool call warning issued".to_string(),
            Self::Allow => String::new(),
        }
    }
}

/// Combined result from all three tiers.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredAnalysisResult {
    pub t1_verdict: HookVerdict,
    pub t2_verdict: Option<HookVerdict>,
    pub t3_verdict: Option<HookVerdict>,
    pub final_verdict: HookVerdict,
    pub duration_ms: f64,
}

// ---------------------------------------------------------------------------
// T1: RULES hook (<1ms target)
// ---------------------------------------------------------------------------

/// T1 hook evaluates predicates from pre-loaded rules.
///
/// Fires on EVERY SSB write. Performance target: <1ms.
/// Rules are loaded from rules.db at construction time and cached in memory.
pub struct T1Hook {
    rule_engine: Box<dyn RuleEngine>,
    fsm_phase: FsmPhaseBit,
    /// Pre-loaded and compiled rules from rules.db, cached for fast evaluation.
    loaded_rules: Vec<(elai_rules::rule_engine::Rule, elai_rules::rule_engine::CompiledPredicate)>,
}

impl T1Hook {
    /// Create a T1 hook with no pre-loaded rules (empty evaluation).
    pub fn new(rule_engine: Box<dyn RuleEngine>, fsm_phase: FsmPhaseBit) -> Self {
        Self {
            rule_engine,
            fsm_phase,
            loaded_rules: Vec::new(),
        }
    }

    /// Create a T1 hook with pre-loaded rules from rules.db.
    ///
    /// The caller (typically orchestrator) loads rules from SQLite, compiles
    /// predicates, and passes the compiled pairs here. This avoids adding
    /// a SQLite dependency to the plugins crate.
    pub fn with_rules(
        rule_engine: Box<dyn RuleEngine>,
        fsm_phase: FsmPhaseBit,
        rules: Vec<(elai_rules::rule_engine::Rule, elai_rules::rule_engine::CompiledPredicate)>,
    ) -> Self {
        tracing::info!(count = rules.len(), "T1 hook loaded with pre-compiled rules");
        Self {
            rule_engine,
            fsm_phase,
            loaded_rules: rules,
        }
    }

    /// Replace the loaded rules (e.g., after FSM phase change).
    pub fn set_rules(
        &mut self,
        rules: Vec<(elai_rules::rule_engine::Rule, elai_rules::rule_engine::CompiledPredicate)>,
    ) {
        tracing::debug!(count = rules.len(), "T1 hook rules updated");
        self.loaded_rules = rules;
    }

    /// Evaluate predicates for active rules matching current phase + domain.
    ///
    /// Performance target: <1ms for typical rule sets (100 rules).
    pub fn evaluate(
        &self,
        action: &str,
        source: &str,
        file_path: &str,
        metric_values: HashMap<String, f64>,
    ) -> Result<HookVerdict, T1Error> {
        let start = Instant::now();

        let ctx = EvaluationContext {
            action,
            source,
            file_path,
            fsm_phase: self.fsm_phase,
            metric_values,
        };

        let verdicts = self.rule_engine.evaluate(&self.loaded_rules, &ctx)?;

        // Find the most severe verdict (BLOCK > WARN > ALLOW)
        let final_verdict = verdicts
            .into_iter()
            .fold(HookVerdict::Allow, |acc, verdict| {
                match (acc.clone(), verdict.result) {
                    (HookVerdict::Block { .. }, _) => acc,
                    (_, VerdictResult::Block) => HookVerdict::Block {
                        message: "Rule violation detected".to_string(),
                        pattern_hash: Self::hash_pattern(&verdict.rule_id, &verdict.evidence),
                    },
                    (_, VerdictResult::Warn) => HookVerdict::Warn {
                        message: verdict.evidence.clone(),
                        pattern_hash: Self::hash_pattern(&verdict.rule_id, &verdict.evidence),
                    },
                    _ => acc,
                }
            });

        let duration = start.elapsed();
        if duration.as_millis() > 1 {
            tracing::warn!("T1 hook exceeded 1ms target: {:?}", duration);
        }

        Ok(final_verdict)
    }

    fn hash_pattern(rule_id: &str, evidence: &str) -> String {
        let mut hasher = blake3::Hasher::new();
        hasher.update(rule_id.as_bytes());
        hasher.update(evidence.as_bytes());
        hasher.finalize().to_hex().to_string()
    }
}

#[derive(Debug)]
pub enum T1Error {
    RuleEngineError(String),
}

impl std::fmt::Display for T1Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RuleEngineError(msg) => write!(f, "rule engine error: {}", msg),
        }
    }
}

impl std::error::Error for T1Error {}

impl From<elai_rules::rule_engine::RuleEngineError> for T1Error {
    fn from(err: elai_rules::rule_engine::RuleEngineError) -> Self {
        T1Error::RuleEngineError(err.to_string())
    }
}

// ---------------------------------------------------------------------------
// T2: AST hook (<10ms target for files <5000 lines)
// ---------------------------------------------------------------------------

/// Maximum nesting depth before flagging as overly complex.
const MAX_NESTING_DEPTH: usize = 8;

/// T2 hook performs tree-sitter parse + pattern matching for file_write actions.
///
/// Fires only on action_type='file_write'. Circuit breaker: files >5000 lines degrade to T1-only.
/// Performance target: <10ms for files <5000 lines.
///
/// Detects:
/// - `unsafe` blocks in Rust
/// - `eval()` calls in Python
/// - Shell injection patterns (`os.system`, `subprocess.call` with `shell=True`)
/// - Overly deep nesting (configurable threshold)
pub struct T2Hook {
    circuit_breaker_lines: usize,
}

/// Detected language for tree-sitter parsing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DetectedLanguage {
    Rust,
    Python,
}

impl T2Hook {
    pub fn new(circuit_breaker_lines: usize) -> Self {
        Self {
            circuit_breaker_lines,
        }
    }

    /// Detect language from file extension.
    fn detect_language(file_path: &str) -> Option<DetectedLanguage> {
        if file_path.ends_with(".rs") {
            Some(DetectedLanguage::Rust)
        } else if file_path.ends_with(".py") {
            Some(DetectedLanguage::Python)
        } else {
            None
        }
    }

    /// Parse source with tree-sitter and apply pattern matching.
    ///
    /// Returns None if circuit breaker triggers (file too large) or language unsupported.
    /// Performance target: <10ms for files <5000 lines.
    pub fn evaluate(
        &self,
        source: &str,
        file_path: &str,
    ) -> Result<Option<HookVerdict>, T2Error> {
        let start = Instant::now();

        // Circuit breaker: count lines first
        let line_count = source.lines().count();
        if line_count > self.circuit_breaker_lines {
            tracing::debug!(
                lines = line_count,
                limit = self.circuit_breaker_lines,
                "T2 circuit breaker triggered"
            );
            return Ok(None);
        }

        let lang = match Self::detect_language(file_path) {
            Some(l) => l,
            None => return Ok(None), // Unsupported language, skip
        };

        let mut parser = tree_sitter::Parser::new();
        match lang {
            DetectedLanguage::Rust => {
                parser
                    .set_language(&tree_sitter_rust::LANGUAGE.into())
                    .map_err(|e| T2Error::ParseError(format!("Rust parser init: {e}")))?;
            }
            DetectedLanguage::Python => {
                // Python grammar not compiled in (avoids C compilation overhead).
                // Fall back to regex-based detection for Python patterns.
                return self.evaluate_python_regex(source, start);
            }
        }

        let tree = parser
            .parse(source, None)
            .ok_or_else(|| T2Error::ParseError("tree-sitter parse returned None".to_string()))?;

        let root = tree.root_node();
        if root.has_error() {
            tracing::debug!(file = file_path, "tree-sitter parse had errors, continuing");
        }

        // Walk AST for patterns
        let mut findings: Vec<(String, String)> = Vec::new();

        match lang {
            DetectedLanguage::Rust => {
                self.check_rust_patterns(&root, source, &mut findings);
            }
            DetectedLanguage::Python => {
                // Already handled above via regex fallback
            }
        }

        // Check nesting depth
        let max_depth = self.max_nesting_depth(&root, 0);
        if max_depth > MAX_NESTING_DEPTH {
            findings.push((
                format!("deep_nesting_{max_depth}"),
                format!("Nesting depth {max_depth} exceeds limit {MAX_NESTING_DEPTH}"),
            ));
        }

        let duration = start.elapsed();
        if duration.as_millis() > 10 {
            tracing::warn!(duration_ms = ?duration.as_millis(), "T2 hook exceeded 10ms target");
        }

        if let Some((pattern_id, message)) = findings.first() {
            let severity = if findings.iter().any(|(id, _)| id.starts_with("unsafe_block")) {
                // unsafe blocks are WARN (they may be legitimate)
                HookVerdict::Warn {
                    message: message.clone(),
                    pattern_hash: blake3::hash(pattern_id.as_bytes()).to_hex().to_string(),
                }
            } else {
                HookVerdict::Block {
                    message: message.clone(),
                    pattern_hash: blake3::hash(pattern_id.as_bytes()).to_hex().to_string(),
                }
            };
            Ok(Some(severity))
        } else {
            Ok(Some(HookVerdict::Allow))
        }
    }

    /// Check Rust-specific AST patterns.
    fn check_rust_patterns(
        &self,
        node: &tree_sitter::Node<'_>,
        source: &str,
        findings: &mut Vec<(String, String)>,
    ) {
        // Check for unsafe blocks
        if node.kind() == "unsafe_block" {
            let start_line = node.start_position().row + 1;
            findings.push((
                format!("unsafe_block_L{start_line}"),
                format!("Unsafe block at line {start_line}"),
            ));
        }

        // Check for .unwrap()/.expect() calls (production code concern)
        // tree-sitter-rust: call_expression.function is a field_expression
        // whose text is e.g. "x.unwrap" (no parens — those are in arguments)
        if node.kind() == "call_expression" {
            if let Some(func) = node.child_by_field_name("function") {
                let text = &source[func.byte_range()];
                if text.ends_with(".unwrap") || text.ends_with(".expect") {
                    let start_line = node.start_position().row + 1;
                    findings.push((
                        format!("unwrap_call_L{start_line}"),
                        format!("unwrap()/expect() at line {start_line}"),
                    ));
                }
            }
        }

        // Recurse into children
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.check_rust_patterns(&child, source, findings);
        }
    }

    /// Regex-based Python pattern detection (fallback when tree-sitter-python
    /// is not compiled in to avoid C build overhead).
    fn evaluate_python_regex(
        &self,
        source: &str,
        start: Instant,
    ) -> Result<Option<HookVerdict>, T2Error> {
        static EVAL_RE: LazyLock<regex::Regex> =
            LazyLock::new(|| regex::Regex::new(r"\beval\s*\(").unwrap());
        static OS_SYSTEM_RE: LazyLock<regex::Regex> =
            LazyLock::new(|| regex::Regex::new(r"\bos\.system\s*\(").unwrap());
        static SUBPROCESS_SHELL_RE: LazyLock<regex::Regex> = LazyLock::new(|| {
            regex::Regex::new(r"subprocess\.(call|run|Popen)\s*\([^)]*shell\s*=\s*True").unwrap()
        });

        let mut findings: Vec<(String, String)> = Vec::new();

        if let Some(m) = EVAL_RE.find(source) {
            let line = source[..m.start()].lines().count() + 1;
            findings.push((
                format!("eval_call_L{line}"),
                format!("eval() call at line {line}"),
            ));
        }

        if let Some(m) = OS_SYSTEM_RE.find(source) {
            let line = source[..m.start()].lines().count() + 1;
            findings.push((
                format!("os_system_L{line}"),
                format!("os.system() shell injection risk at line {line}"),
            ));
        }

        if let Some(m) = SUBPROCESS_SHELL_RE.find(source) {
            let line = source[..m.start()].lines().count() + 1;
            findings.push((
                format!("subprocess_shell_L{line}"),
                format!("subprocess with shell=True at line {line}"),
            ));
        }

        let duration = start.elapsed();
        if duration.as_millis() > 10 {
            tracing::warn!(duration_ms = ?duration.as_millis(), "T2 Python regex exceeded 10ms");
        }

        if let Some((pattern_id, message)) = findings.first() {
            Ok(Some(HookVerdict::Block {
                message: message.clone(),
                pattern_hash: blake3::hash(pattern_id.as_bytes()).to_hex().to_string(),
            }))
        } else {
            Ok(Some(HookVerdict::Allow))
        }
    }

    /// Compute maximum nesting depth of the AST.
    fn max_nesting_depth(&self, node: &tree_sitter::Node<'_>, current_depth: usize) -> usize {
        let is_nesting = matches!(
            node.kind(),
            "block" | "if_expression" | "match_expression" | "loop_expression"
                | "while_expression" | "for_expression" | "closure_expression"
                | "if_statement" | "for_statement" | "while_statement"
                | "try_statement" | "with_statement"
        );
        let depth = if is_nesting { current_depth + 1 } else { current_depth };
        let mut max = depth;
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            let child_max = self.max_nesting_depth(&child, depth);
            if child_max > max {
                max = child_max;
            }
        }
        max
    }
}

#[derive(Debug)]
pub enum T2Error {
    ParseError(String),
    UnsupportedLanguage(String),
}

impl std::fmt::Display for T2Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ParseError(msg) => write!(f, "parse error: {}", msg),
            Self::UnsupportedLanguage(lang) => write!(f, "unsupported language: {}", lang),
        }
    }
}

impl std::error::Error for T2Error {}

// ---------------------------------------------------------------------------
// T3: PATTERN MATCH hook (<5ms target)
// ---------------------------------------------------------------------------

/// T3 hook matches known-ambiguous patterns when WARN exists and no BLOCK from T1/T2.
///
/// Fires ONLY when (WARN exists) AND (no BLOCK from T1/T2).
/// Pattern-match library of known-ambiguous patterns. NO LLM.
/// Performance target: <5ms.
pub struct T3Hook {
    patterns: Vec<AmbiguousPattern>,
}

impl T3Hook {
    pub fn new() -> Self {
        Self {
            patterns: Self::default_patterns(),
        }
    }

    /// Match against known-ambiguous patterns.
    ///
    /// Returns None if no patterns match or if should not fire (BLOCK from T1/T2).
    /// Performance target: <5ms.
    pub fn evaluate(&self, source: &str, file_path: &str) -> Option<HookVerdict> {
        let start = Instant::now();

        for pattern in &self.patterns {
            if pattern.matches(source, file_path) {
                let duration = start.elapsed();
                if duration.as_millis() > 5 {
                    tracing::warn!("T3 hook exceeded 5ms target: {:?}", duration);
                }

                return Some(HookVerdict::Block {
                    message: pattern.message.clone(),
                    pattern_hash: Self::hash_pattern(&pattern.id),
                });
            }
        }

        None
    }

    fn default_patterns() -> Vec<AmbiguousPattern> {
        // Static regex compilation — provably valid literals, compiled once.
        static SWALLOW_RE: LazyLock<regex::Regex> = LazyLock::new(|| {
            regex::Regex::new(r"except\s+\w+\s*:\s*\n\s*pass").unwrap()
        });
        static LONG_FN_RE: LazyLock<regex::Regex> = LazyLock::new(|| {
            regex::Regex::new(r"(?m)^\s*def\s+\w+.*:(?:\n\s+.*){200,}").unwrap()
        });

        vec![
            AmbiguousPattern {
                id: "swallow_exception".to_string(),
                regex: SWALLOW_RE.clone(),
                message: "Exception swallowing detected".to_string(),
            },
            AmbiguousPattern {
                id: "long_function".to_string(),
                regex: LONG_FN_RE.clone(),
                message: "Function exceeds 200 lines".to_string(),
            },
        ]
    }

    fn hash_pattern(pattern_id: &str) -> String {
        blake3::hash(pattern_id.as_bytes()).to_hex().to_string()
    }
}

/// A known-ambiguous pattern to match.
#[derive(Debug, Clone)]
struct AmbiguousPattern {
    id: String,
    regex: regex::Regex,
    message: String,
}

impl AmbiguousPattern {
    fn matches(&self, source: &str, _file_path: &str) -> bool {
        self.regex.is_match(source)
    }
}

// ---------------------------------------------------------------------------
// Tiered analysis orchestrator
// ---------------------------------------------------------------------------

/// Main entry point for tiered static analysis.
///
/// Coordinates T1, T2, T3 hooks according to spec:
/// - T1 fires on EVERY SSB write (sync, <1ms)
/// - T2 fires on file_write only (async, <10ms)
/// - T3 fires only when WARN exists AND no BLOCK from T1/T2 (<5ms)
pub struct TieredAnalysis {
    t1: T1Hook,
    t2: T2Hook,
    t3: T3Hook,
}

impl TieredAnalysis {
    /// Create with no pre-loaded rules (T1 evaluates against empty set).
    pub fn new(rule_engine: Box<dyn RuleEngine>, fsm_phase: FsmPhaseBit) -> Self {
        Self {
            t1: T1Hook::new(rule_engine, fsm_phase),
            t2: T2Hook::new(5000),
            t3: T3Hook::new(),
        }
    }

    /// Create with pre-loaded rules for T1 evaluation.
    pub fn with_rules(
        rule_engine: Box<dyn RuleEngine>,
        fsm_phase: FsmPhaseBit,
        rules: Vec<(elai_rules::rule_engine::Rule, elai_rules::rule_engine::CompiledPredicate)>,
    ) -> Self {
        Self {
            t1: T1Hook::with_rules(rule_engine, fsm_phase, rules),
            t2: T2Hook::new(5000),
            t3: T3Hook::new(),
        }
    }

    /// Run tiered analysis on an SSB write.
    ///
    /// Returns the final verdict and timing information.
    pub fn analyze(
        &self,
        action_type: &str,
        source: Option<&str>,
        file_path: Option<&str>,
        metric_values: HashMap<String, f64>,
    ) -> Result<TieredAnalysisResult, TieredAnalysisError> {
        let total_start = Instant::now();

        // T1: Always fires
        let t1_source = source.unwrap_or("");
        let t1_file_path = file_path.unwrap_or("");
        let t1_verdict = self.t1.evaluate(action_type, t1_source, t1_file_path, metric_values)?;

        // T2: Only fires on file_write
        let t2_verdict = if let (true, Some(src)) = (action_type == "file_write", source) {
            self.t2.evaluate(src, file_path.unwrap_or(""))?
        } else {
            None
        };

        // T3: Only fires when WARN exists AND no BLOCK from T1/T2
        let t2_has_warn = matches!(t2_verdict, Some(HookVerdict::Warn { .. }));
        let t2_has_block = matches!(t2_verdict, Some(HookVerdict::Block { .. }));
        let has_warn = t1_verdict.is_warn() || t2_has_warn;
        let has_block = t1_verdict.is_block() || t2_has_block;
        let t3_verdict = if let (true, false, Some(src)) = (has_warn, has_block, source) {
            self.t3.evaluate(src, file_path.unwrap_or(""))
        } else {
            None
        };

        // Final verdict: BLOCK takes precedence
        let final_verdict = if t1_verdict.is_block() {
            t1_verdict.clone()
        } else if let Some(ref v @ HookVerdict::Block { .. }) = t2_verdict {
            v.clone()
        } else if let Some(ref v @ HookVerdict::Block { .. }) = t3_verdict {
            v.clone()
        } else if t1_verdict.is_warn() {
            t1_verdict.clone()
        } else if let Some(ref v @ HookVerdict::Warn { .. }) = t2_verdict {
            v.clone()
        } else if let Some(ref v @ HookVerdict::Warn { .. }) = t3_verdict {
            v.clone()
        } else {
            HookVerdict::Allow
        };

        let duration_ms = total_start.elapsed().as_secs_f64() * 1000.0;

        Ok(TieredAnalysisResult {
            t1_verdict,
            t2_verdict,
            t3_verdict,
            final_verdict,
            duration_ms,
        })
    }
}

#[derive(Debug)]
pub enum TieredAnalysisError {
    T1Error(T1Error),
    T2Error(T2Error),
}

impl std::fmt::Display for TieredAnalysisError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::T1Error(err) => write!(f, "T1 error: {}", err),
            Self::T2Error(err) => write!(f, "T2 error: {}", err),
        }
    }
}

impl std::error::Error for TieredAnalysisError {}

impl From<T1Error> for TieredAnalysisError {
    fn from(err: T1Error) -> Self {
        TieredAnalysisError::T1Error(err)
    }
}

impl From<T2Error> for TieredAnalysisError {
    fn from(err: T2Error) -> Self {
        TieredAnalysisError::T2Error(err)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hook_verdict_is_block() {
        assert!(HookVerdict::Block {
            message: "test".to_string(),
            pattern_hash: "hash".to_string()
        }
        .is_block());
        assert!(!HookVerdict::Warn {
            message: "test".to_string(),
            pattern_hash: "hash".to_string()
        }
        .is_block());
        assert!(!HookVerdict::Allow.is_block());
    }

    #[test]
    fn test_hook_verdict_is_warn() {
        assert!(HookVerdict::Warn {
            message: "test".to_string(),
            pattern_hash: "hash".to_string()
        }
        .is_warn());
        assert!(!HookVerdict::Block {
            message: "test".to_string(),
            pattern_hash: "hash".to_string()
        }
        .is_warn());
        assert!(!HookVerdict::Allow.is_warn());
    }

    #[test]
    fn test_hook_verdict_sanitization() {
        let block = HookVerdict::Block {
            message: "secret rule details".to_string(),
            pattern_hash: "secret".to_string(),
        };
        assert_eq!(block.to_agent_error(), "Tool call rejected by policy");

        let warn = HookVerdict::Warn {
            message: "secret warning details".to_string(),
            pattern_hash: "secret".to_string(),
        };
        assert_eq!(warn.to_agent_error(), "Tool call warning issued");
    }

    #[test]
    fn test_t2_circuit_breaker() {
        let hook = T2Hook::new(10);
        let small_source = "line1\nline2\nline3";
        let large_source = (0..20).map(|i| format!("line{}", i)).collect::<Vec<_>>().join("\n");

        // Small file should pass circuit breaker
        assert!(hook.evaluate(small_source, "test.rs").unwrap().is_some());

        // Large file should trigger circuit breaker
        assert!(hook.evaluate(&large_source, "test.rs").unwrap().is_none());
    }

    #[test]
    fn test_t3_pattern_matching() {
        let hook = T3Hook::new();

        // Should match exception swallowing
        let bad_code = "try:\n    pass\nexcept Exception:\n    pass";
        assert!(hook.evaluate(bad_code, "test.py").is_some());

        // Should not match clean code
        let clean_code = "try:\n    pass\nexcept Exception as e:\n    log.error(e)";
        assert!(hook.evaluate(clean_code, "test.py").is_none());
    }
}

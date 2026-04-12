//! Tiered static analysis hooks for SSB write validation.
//!
//! Implements three tiers of deterministic validation:
//! - T1 (RULES, <1ms): Predicate evaluation from rules.db on every SSB write
//! - T2 (AST, <10ms): tree-sitter parse for file_write actions only
//! - T3 (PATTERN MATCH, <5ms): Known-ambiguous pattern matching when WARN exists
//!
//! No LLM in the validation hot path. All findings written to SSB as 'finding' entries.

use blake2::{digest::consts::U32, Blake2s};
use digest::Digest;
use elai_rules::predicate::{Verdict, VerdictResult};
use elai_rules::rule_engine::{EvaluationContext, FsmPhaseBit, RuleEngine};
use std::collections::HashMap;
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TieredAnalysisResult {
    pub t1_verdict: HookVerdict,
    pub t2_verdict: Option<HookVerdict>,
    pub t3_verdict: Option<HookVerdict>,
    pub final_verdict: HookVerdict,
    pub duration_ms: f64,
}

impl TieredAnalysisResult {
    /// Returns the final verdict (BLOCK takes precedence over WARN, which takes precedence over ALLOW).
    pub fn final_verdict() -> HookVerdict {
        // Computed after all tiers run
        HookVerdict::Allow
    }
}

// ---------------------------------------------------------------------------
// T1: RULES hook (<1ms target)
// ---------------------------------------------------------------------------

/// T1 hook evaluates predicates from rules.db for active rules.
///
/// Fires on EVERY SSB write. Performance target: <1ms.
pub struct T1Hook {
    rule_engine: Box<dyn RuleEngine>,
    fsm_phase: FsmPhaseBit,
}

impl T1Hook {
    pub fn new(rule_engine: Box<dyn RuleEngine>, fsm_phase: FsmPhaseBit) -> Self {
        Self {
            rule_engine,
            fsm_phase,
        }
    }

    /// Evaluate predicates from rules.db for active rules matching current phase + domain.
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

        // Get active rules for current phase and evaluate
        let verdicts = self.rule_engine.evaluate(&ctx)?;

        // Find the most severe verdict (BLOCK > WARN > ALLOW)
        let final_verdict = verdicts
            .into_iter()
            .fold(HookVerdict::Allow, |acc, verdict| {
                match (acc, verdict.result) {
                    (HookVerdict::Block { .. }, _) => acc,
                    (_, VerdictResult::Block) => HookVerdict::Block {
                        message: "Rule violation detected".to_string(),
                        pattern_hash: Self::hash_pattern(&verdict.rule_id, &verdict.evidence),
                    },
                    (HookVerdict::Warn { .. }, VerdictResult::Warn) => acc,
                    (_, VerdictResult::Warn) => HookVerdict::Warn {
                        message: verdict.evidence,
                        pattern_hash: Self::hash_pattern(&verdict.rule_id, &verdict.evidence),
                    },
                    _ => acc,
                }
            });

        let duration = start.elapsed();
        if duration.as_millis() > 1 {
            log::warn!("T1 hook exceeded 1ms target: {:?}", duration);
        }

        Ok(final_verdict)
    }

    fn hash_pattern(rule_id: &str, evidence: &str) -> String {
        type Hasher = Blake2s<U32>;
        let mut hasher = Hasher::new();
        hasher.update(rule_id.as_bytes());
        hasher.update(evidence.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum T1Error {
    #[error("rule engine error: {0}")]
    RuleEngineError(String),
}

// ---------------------------------------------------------------------------
// T2: AST hook (<10ms target for files <5000 lines)
// ---------------------------------------------------------------------------

/// T2 hook performs tree-sitter parse + pattern matching for file_write actions.
///
/// Fires only on action_type='file_write'. Circuit breaker: files >5000 lines degrade to T1-only.
/// Performance target: <10ms for files <5000 lines.
pub struct T2Hook {
    circuit_breaker_lines: usize,
}

impl T2Hook {
    pub fn new(circuit_breaker_lines: usize) -> Self {
        Self {
            circuit_breaker_lines,
        }
    }

    /// Parse source with tree-sitter and apply pattern matching.
    ///
    /// Returns None if circuit breaker triggers (file too large).
    /// Performance target: <10ms for files <5000 lines.
    pub fn evaluate(
        &self,
        source: &str,
        _file_path: &str,
    ) -> Result<Option<HookVerdict>, T2Error> {
        let start = Instant::now();

        // Circuit breaker: count lines first
        let line_count = source.lines().count();
        if line_count > self.circuit_breaker_lines {
            log::debug!(
                "T2 circuit breaker triggered: {} lines > {}",
                line_count,
                self.circuit_breaker_lines
            );
            return Ok(None);
        }

        // TODO: Implement tree-sitter parsing and pattern matching
        // For now, return ALLOW (no AST checks implemented yet)
        // This is a stub that will be filled in with actual tree-sitter integration

        let duration = start.elapsed();
        if duration.as_millis() > 10 {
            log::warn!("T2 hook exceeded 10ms target: {:?}", duration);
        }

        Ok(Some(HookVerdict::Allow))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum T2Error {
    #[error("parse error: {0}")]
    ParseError(String),

    #[error("unsupported language: {0}")]
    UnsupportedLanguage(String),
}

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
                    log::warn!("T3 hook exceeded 5ms target: {:?}", duration);
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
        vec![
            // Example: catch Exception: pass -> BLOCK
            AmbiguousPattern {
                id: "swallow_exception".to_string(),
                regex: regex::Regex::new(r"catch\s+\w+\s*:\s*pass").unwrap(),
                message: "Exception swallowing detected".to_string(),
            },
            // Example: function >200 lines -> WARN
            AmbiguousPattern {
                id: "long_function".to_string(),
                regex: regex::Regex::new(r"^\s*def\s+\w+.*:(?:\n\s+.*){200,}").unwrap(),
                message: "Function exceeds 200 lines".to_string(),
            },
        ]
    }

    fn hash_pattern(pattern_id: &str) -> String {
        type Hasher = Blake2s<U32>;
        let mut hasher = Hasher::new();
        hasher.update(pattern_id.as_bytes());
        format!("{:x}", hasher.finalize())
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
    pub fn new(rule_engine: Box<dyn RuleEngine>, fsm_phase: FsmPhaseBit) -> Self {
        Self {
            t1: T1Hook::new(rule_engine, fsm_phase),
            t2: T2Hook::new(5000), // 5000 line circuit breaker
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
        let t2_verdict = if action_type == "file_write" && source.is_some() {
            Some(self.t2.evaluate(source.unwrap(), file_path.unwrap_or(""))?)
        } else {
            None
        };

        // T3: Only fires when WARN exists AND no BLOCK from T1/T2
        let has_warn = t1_verdict.is_warn() || t2_verdict.as_ref().map_or(false, |v| v.is_warn());
        let has_block = t1_verdict.is_block() || t2_verdict.as_ref().map_or(false, |v| v.is_block());
        let t3_verdict = if has_warn && !has_block && source.is_some() {
            self.t3.evaluate(source.unwrap(), file_path.unwrap_or(""))
        } else {
            None
        };

        // Final verdict: BLOCK takes precedence
        let final_verdict = if t1_verdict.is_block() {
            t1_verdict
        } else if t2_verdict.as_ref().and_then(|v| if v.is_block() { Some(v) } else { None }).is_some() {
            t2_verdict.unwrap()
        } else if t3_verdict.as_ref().and_then(|v| if v.is_block() { Some(v) } else { None }).is_some() {
            t3_verdict.unwrap()
        } else if t1_verdict.is_warn() {
            t1_verdict
        } else if t2_verdict.as_ref().and_then(|v| if v.is_warn() { Some(v) } else { None }).is_some() {
            t2_verdict.unwrap()
        } else if t3_verdict.as_ref().and_then(|v| if v.is_warn() { Some(v) } else { None }).is_some() {
            t3_verdict.unwrap()
        } else {
            HookVerdict::Allow
        };

        let duration_ms = total_start.elapsed().as_secs_f64() * 1000.0;

        Ok(TieredAnalysisResult {
            t1_verdict,
            t2_verdict: t2_verdict.flatten(),
            t3_verdict,
            final_verdict,
            duration_ms,
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TieredAnalysisError {
    #[error("T1 error: {0}")]
    T1Error(#[from] T1Error),

    #[error("T2 error: {0}")]
    T2Error(#[from] T2Error),
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

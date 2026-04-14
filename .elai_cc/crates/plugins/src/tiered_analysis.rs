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

use crate::import_extractor::{
    check_deprecated_imports, extract_imports, DeprecatedImport, ImportLanguage,
};

// ---------------------------------------------------------------------------
// File discipline configuration (elai_cc.toml [file_discipline])
// ---------------------------------------------------------------------------

/// Per-language line limits for file size enforcement.
#[derive(Debug, Clone)]
pub struct LanguageLimits {
    /// WARN threshold for production files.
    pub production_warn: usize,
    /// WARN threshold for test files.
    pub test_warn: usize,
    /// Hard BLOCK threshold (4x production WARN per spec).
    pub hard_block: usize,
}

/// File discipline configuration. Loaded from elai_cc.toml `[file_discipline]`.
#[derive(Debug, Clone)]
pub struct FileDisciplineConfig {
    pub limits: HashMap<String, LanguageLimits>,
    /// File extensions exempt from size checks (e.g. generated code).
    pub exempt_extensions: Vec<String>,
    /// Path prefixes exempt from size checks.
    pub exempt_paths: Vec<String>,
}

impl Default for FileDisciplineConfig {
    fn default() -> Self {
        let mut limits = HashMap::new();
        // Per CC design spec section 3.2 table
        limits.insert("rust".into(), LanguageLimits { production_warn: 100, test_warn: 200, hard_block: 400 });
        limits.insert("go".into(), LanguageLimits { production_warn: 100, test_warn: 200, hard_block: 400 });
        limits.insert("python".into(), LanguageLimits { production_warn: 120, test_warn: 250, hard_block: 480 });
        limits.insert("typescript".into(), LanguageLimits { production_warn: 150, test_warn: 250, hard_block: 600 });
        limits.insert("javascript".into(), LanguageLimits { production_warn: 150, test_warn: 250, hard_block: 600 });
        limits.insert("java".into(), LanguageLimits { production_warn: 200, test_warn: 300, hard_block: 800 });
        limits.insert("kotlin".into(), LanguageLimits { production_warn: 150, test_warn: 250, hard_block: 600 });
        limits.insert("csharp".into(), LanguageLimits { production_warn: 200, test_warn: 300, hard_block: 800 });
        limits.insert("c".into(), LanguageLimits { production_warn: 150, test_warn: 250, hard_block: 600 });
        limits.insert("cpp".into(), LanguageLimits { production_warn: 150, test_warn: 250, hard_block: 600 });
        limits.insert("ruby".into(), LanguageLimits { production_warn: 120, test_warn: 200, hard_block: 480 });
        limits.insert("elixir".into(), LanguageLimits { production_warn: 120, test_warn: 200, hard_block: 480 });
        limits.insert("swift".into(), LanguageLimits { production_warn: 150, test_warn: 250, hard_block: 600 });

        Self {
            limits,
            exempt_extensions: Vec::new(),
            exempt_paths: Vec::new(),
        }
    }
}

/// Default limits for unrecognized languages.
const DEFAULT_PRODUCTION_WARN: usize = 150;
const DEFAULT_TEST_WARN: usize = 250;
const DEFAULT_HARD_BLOCK: usize = 600;

/// Threshold multiplier for "approaching limit" INFO (80% of WARN).
const APPROACHING_LIMIT_RATIO: f64 = 0.8;

// ---------------------------------------------------------------------------
// Language detection + test file heuristics (shared by T1 + T2)
// ---------------------------------------------------------------------------

/// Detect language from file extension. Returns lowercase language name.
fn detect_language(file_path: &str) -> Option<&'static str> {
    let ext = file_path.rsplit('.').next()?;
    match ext {
        "rs" => Some("rust"),
        "py" | "pyi" => Some("python"),
        "go" => Some("go"),
        "ts" | "tsx" => Some("typescript"),
        "js" | "jsx" | "mjs" | "cjs" => Some("javascript"),
        "java" => Some("java"),
        "cs" => Some("csharp"),
        "rb" => Some("ruby"),
        "ex" | "exs" => Some("elixir"),
        "swift" => Some("swift"),
        "kt" | "kts" => Some("kotlin"),
        "c" | "h" => Some("c"),
        "cpp" | "cc" | "cxx" | "hpp" => Some("cpp"),
        _ => None,
    }
}

/// Heuristic: is this file path a test file?
fn is_test_file(file_path: &str) -> bool {
    let path = file_path.replace('\\', "/");
    // Directory-based patterns (also match at start of path)
    if path.contains("/tests/")
        || path.contains("/test/")
        || path.contains("/__tests__/")
        || path.contains("/spec/")
        || path.starts_with("tests/")
        || path.starts_with("test/")
        || path.starts_with("__tests__/")
        || path.starts_with("spec/")
    {
        return true;
    }
    let filename = path.rsplit('/').next().unwrap_or(&path);
    filename.ends_with("_test.rs")
        || (filename.starts_with("test_") && filename.ends_with(".py"))
        || filename.ends_with("_test.py")
        || filename.ends_with("_test.go")
        || filename.contains(".test.")
        || filename.contains(".spec.")
        || filename.ends_with("Test.java")
        || filename.ends_with("Test.cs")
        || filename.ends_with("_spec.rb")
}

// ---------------------------------------------------------------------------
// T1 lazy code pattern registry (per-language regex, <1ms)
// ---------------------------------------------------------------------------

/// A compiled lazy code detection pattern.
struct LazyPattern {
    id: &'static str,
    regex: &'static LazyLock<regex::Regex>,
    /// None = universal (all languages).
    language: Option<&'static str>,
}

// Universal patterns
static RE_TODO_SLASH: LazyLock<regex::Regex> = LazyLock::new(|| regex::Regex::new(r"//\s*TODO\b").unwrap());
static RE_FIXME_SLASH: LazyLock<regex::Regex> = LazyLock::new(|| regex::Regex::new(r"//\s*FIXME\b").unwrap());
static RE_HACK_SLASH: LazyLock<regex::Regex> = LazyLock::new(|| regex::Regex::new(r"//\s*HACK\b").unwrap());
static RE_ELLIPSIS_SLASH: LazyLock<regex::Regex> = LazyLock::new(|| regex::Regex::new(r"//\s*\.\.\.").unwrap());
static RE_REST_OF_SLASH: LazyLock<regex::Regex> = LazyLock::new(|| regex::Regex::new(r"//\s*rest of").unwrap());
static RE_TODO_HASH: LazyLock<regex::Regex> = LazyLock::new(|| regex::Regex::new(r"#\s*TODO\b").unwrap());
static RE_FIXME_HASH: LazyLock<regex::Regex> = LazyLock::new(|| regex::Regex::new(r"#\s*FIXME\b").unwrap());
static RE_ELLIPSIS_HASH: LazyLock<regex::Regex> = LazyLock::new(|| regex::Regex::new(r"#\s*\.\.\.").unwrap());
static RE_PLACEHOLDER_STR: LazyLock<regex::Regex> = LazyLock::new(|| regex::Regex::new(r#""placeholder""#).unwrap());

// Rust-specific
static RE_RUST_TODO: LazyLock<regex::Regex> = LazyLock::new(|| regex::Regex::new(r"\btodo!\(").unwrap());
static RE_RUST_UNIMPL: LazyLock<regex::Regex> = LazyLock::new(|| regex::Regex::new(r"\bunimplemented!\(").unwrap());
static RE_RUST_EMPTY_FN: LazyLock<regex::Regex> = LazyLock::new(|| regex::Regex::new(r"fn\s+\w+\([^)]*\)[^{]*\{\s*\}").unwrap());

// Python-specific
static RE_PY_PASS: LazyLock<regex::Regex> = LazyLock::new(|| regex::Regex::new(r"(?m)\bpass\s*$").unwrap());
static RE_PY_NOT_IMPL: LazyLock<regex::Regex> = LazyLock::new(|| regex::Regex::new(r"\braise NotImplementedError\b").unwrap());
static RE_PY_ELLIPSIS: LazyLock<regex::Regex> = LazyLock::new(|| regex::Regex::new(r"def\s+\w+\([^)]*\):\s*\.\.\.").unwrap());

// TypeScript/JavaScript-specific
static RE_TS_NOT_IMPL: LazyLock<regex::Regex> = LazyLock::new(|| regex::Regex::new(r#"throw new Error\(['"](?:not |todo|implement)"#).unwrap());
static RE_TS_CONSOLE: LazyLock<regex::Regex> = LazyLock::new(|| regex::Regex::new(r"console\.log\(").unwrap());

// Go-specific
static RE_GO_PANIC: LazyLock<regex::Regex> = LazyLock::new(|| regex::Regex::new(r#"\bpanic\(\s*['"](?:not |todo|implement)"#).unwrap());
static RE_GO_EMPTY_FN: LazyLock<regex::Regex> = LazyLock::new(|| regex::Regex::new(r"func\s+\w+\([^)]*\)[^{]*\{\s*\}").unwrap());

// Java-specific
static RE_JAVA_UNSUP: LazyLock<regex::Regex> = LazyLock::new(|| regex::Regex::new(r"throw new UnsupportedOperationException\(").unwrap());
static RE_JAVA_TODO: LazyLock<regex::Regex> = LazyLock::new(|| regex::Regex::new(r#"throw new RuntimeException\(['"]TODO"#).unwrap());

// C#-specific
static RE_CS_NOT_IMPL: LazyLock<regex::Regex> = LazyLock::new(|| regex::Regex::new(r"throw new NotImplementedException\(").unwrap());

/// Full pattern registry. Order does not matter — all are checked.
static LAZY_PATTERNS: LazyLock<Vec<LazyPattern>> = LazyLock::new(|| vec![
    // Universal
    LazyPattern { id: "todo_slash", regex: &RE_TODO_SLASH, language: None },
    LazyPattern { id: "fixme_slash", regex: &RE_FIXME_SLASH, language: None },
    LazyPattern { id: "hack_slash", regex: &RE_HACK_SLASH, language: None },
    LazyPattern { id: "ellipsis_slash", regex: &RE_ELLIPSIS_SLASH, language: None },
    LazyPattern { id: "rest_of_slash", regex: &RE_REST_OF_SLASH, language: None },
    LazyPattern { id: "todo_hash", regex: &RE_TODO_HASH, language: None },
    LazyPattern { id: "fixme_hash", regex: &RE_FIXME_HASH, language: None },
    LazyPattern { id: "ellipsis_hash", regex: &RE_ELLIPSIS_HASH, language: None },
    LazyPattern { id: "placeholder_str", regex: &RE_PLACEHOLDER_STR, language: None },
    // Rust
    LazyPattern { id: "rust_todo_macro", regex: &RE_RUST_TODO, language: Some("rust") },
    LazyPattern { id: "rust_unimplemented", regex: &RE_RUST_UNIMPL, language: Some("rust") },
    LazyPattern { id: "rust_empty_fn", regex: &RE_RUST_EMPTY_FN, language: Some("rust") },
    // Python
    LazyPattern { id: "py_bare_pass", regex: &RE_PY_PASS, language: Some("python") },
    LazyPattern { id: "py_not_implemented", regex: &RE_PY_NOT_IMPL, language: Some("python") },
    LazyPattern { id: "py_ellipsis_body", regex: &RE_PY_ELLIPSIS, language: Some("python") },
    // TypeScript/JavaScript
    LazyPattern { id: "ts_not_implemented", regex: &RE_TS_NOT_IMPL, language: Some("typescript") },
    LazyPattern { id: "ts_console_log", regex: &RE_TS_CONSOLE, language: Some("typescript") },
    LazyPattern { id: "js_not_implemented", regex: &RE_TS_NOT_IMPL, language: Some("javascript") },
    LazyPattern { id: "js_console_log", regex: &RE_TS_CONSOLE, language: Some("javascript") },
    // Go
    LazyPattern { id: "go_panic_placeholder", regex: &RE_GO_PANIC, language: Some("go") },
    LazyPattern { id: "go_empty_fn", regex: &RE_GO_EMPTY_FN, language: Some("go") },
    // Java
    LazyPattern { id: "java_unsupported_op", regex: &RE_JAVA_UNSUP, language: Some("java") },
    LazyPattern { id: "java_todo_exception", regex: &RE_JAVA_TODO, language: Some("java") },
    // C#
    LazyPattern { id: "cs_not_implemented", regex: &RE_CS_NOT_IMPL, language: Some("csharp") },
]);

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
    /// File discipline config for size enforcement.
    file_discipline: FileDisciplineConfig,
}

impl T1Hook {
    /// Create a T1 hook with no pre-loaded rules (empty evaluation).
    pub fn new(rule_engine: Box<dyn RuleEngine>, fsm_phase: FsmPhaseBit) -> Self {
        Self {
            rule_engine,
            fsm_phase,
            loaded_rules: Vec::new(),
            file_discipline: FileDisciplineConfig::default(),
        }
    }

    /// Create a T1 hook with pre-loaded rules from rules.db.
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
            file_discipline: FileDisciplineConfig::default(),
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

    /// Replace file discipline config at runtime.
    pub fn set_file_discipline(&mut self, config: FileDisciplineConfig) {
        self.file_discipline = config;
    }

    /// Evaluate predicates for active rules matching current phase + domain.
    ///
    /// Also runs T1 lazy code detection and file size enforcement.
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

        // Find the most severe verdict from rules (BLOCK > WARN > ALLOW)
        let mut final_verdict = verdicts
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

        // T1 lazy code detection: per-language regex patterns -> WARN
        if !source.is_empty() {
            let lazy_verdict = self.evaluate_lazy_patterns(source, file_path);
            final_verdict = Self::merge_verdicts(final_verdict, lazy_verdict);
        }

        // T1 file size enforcement: language-specific line limits
        if action == "file_write" && !source.is_empty() {
            let size_verdict = self.evaluate_file_size(source, file_path);
            final_verdict = Self::merge_verdicts(final_verdict, size_verdict);
        }

        let duration = start.elapsed();
        if duration.as_millis() > 1 {
            tracing::warn!("T1 hook exceeded 1ms target: {:?}", duration);
        }

        Ok(final_verdict)
    }

    /// Evaluate lazy code detection patterns against source.
    ///
    /// Returns WARN on first match. Per spec: T1 always WARNs, T2 determines
    /// final verdict based on AST context (test vs production).
    fn evaluate_lazy_patterns(&self, source: &str, file_path: &str) -> HookVerdict {
        let file_lang = detect_language(file_path);
        // console.log exemption for test files (per audit I-4)
        let is_test = is_test_file(file_path);

        for pat in LAZY_PATTERNS.iter() {
            // Language filter: skip if pattern is language-specific and doesn't match
            if let Some(pat_lang) = pat.language {
                if file_lang != Some(pat_lang) {
                    continue;
                }
            }
            // console.log exemption for test files
            if (pat.id == "ts_console_log" || pat.id == "js_console_log") && is_test {
                continue;
            }
            if pat.regex.is_match(source) {
                return HookVerdict::Warn {
                    message: format!("Lazy code pattern '{}' detected", pat.id),
                    pattern_hash: blake3::hash(pat.id.as_bytes()).to_hex().to_string(),
                };
            }
        }
        HookVerdict::Allow
    }

    /// Evaluate file size against language-specific limits.
    ///
    /// Returns BLOCK for extreme monoliths (4x WARN), WARN for oversized files,
    /// Allow otherwise. Per CC design spec section 3.2.
    fn evaluate_file_size(&self, source: &str, file_path: &str) -> HookVerdict {
        // Check exemptions
        for ext in &self.file_discipline.exempt_extensions {
            if file_path.ends_with(ext.as_str()) {
                return HookVerdict::Allow;
            }
        }
        for prefix in &self.file_discipline.exempt_paths {
            if file_path.starts_with(prefix.as_str()) {
                return HookVerdict::Allow;
            }
        }

        let lang = detect_language(file_path).unwrap_or("default");
        let is_test = is_test_file(file_path);
        let line_count = source.lines().count();

        let (warn_limit, hard_block) = if let Some(limits) = self.file_discipline.limits.get(lang) {
            let warn = if is_test { limits.test_warn } else { limits.production_warn };
            (warn, limits.hard_block)
        } else {
            let warn = if is_test { DEFAULT_TEST_WARN } else { DEFAULT_PRODUCTION_WARN };
            (warn, DEFAULT_HARD_BLOCK)
        };

        if line_count > hard_block {
            HookVerdict::Block {
                message: format!(
                    "File exceeds hard block limit ({} lines > {} max for {})",
                    line_count, hard_block, lang
                ),
                pattern_hash: blake3::hash(
                    format!("file_size_block_{}_{}", lang, line_count).as_bytes()
                ).to_hex().to_string(),
            }
        } else if line_count > warn_limit {
            HookVerdict::Warn {
                message: format!(
                    "File exceeds size limit ({} lines > {} warn for {})",
                    line_count, warn_limit, lang
                ),
                pattern_hash: blake3::hash(
                    format!("file_size_warn_{}_{}", lang, line_count).as_bytes()
                ).to_hex().to_string(),
            }
        } else {
            let approaching = (warn_limit as f64 * APPROACHING_LIMIT_RATIO) as usize;
            if line_count > approaching {
                tracing::info!(
                    lines = line_count, limit = warn_limit, file = file_path,
                    "File approaching size limit"
                );
            }
            HookVerdict::Allow
        }
    }

    /// Merge two verdicts, keeping the most severe.
    fn merge_verdicts(a: HookVerdict, b: HookVerdict) -> HookVerdict {
        match (&a, &b) {
            (HookVerdict::Block { .. }, _) => a,
            (_, HookVerdict::Block { .. }) => b,
            (HookVerdict::Warn { .. }, _) => a,
            (_, HookVerdict::Warn { .. }) => b,
            _ => a,
        }
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
    /// Deprecated import patterns for cross-referencing AST-extracted imports.
    /// Loaded from rules.db deprecated_patterns table at construction time.
    deprecated_imports: Vec<DeprecatedImport>,
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
            deprecated_imports: Vec::new(),
        }
    }

    /// Create a T2 hook with deprecated import patterns for cross-referencing.
    pub fn with_deprecated_imports(
        circuit_breaker_lines: usize,
        deprecated_imports: Vec<DeprecatedImport>,
    ) -> Self {
        tracing::info!(
            count = deprecated_imports.len(),
            "T2 hook loaded with deprecated import patterns"
        );
        Self {
            circuit_breaker_lines,
            deprecated_imports,
        }
    }

    /// Replace the deprecated import patterns at runtime.
    pub fn set_deprecated_imports(&mut self, patterns: Vec<DeprecatedImport>) {
        tracing::debug!(count = patterns.len(), "T2 deprecated imports updated");
        self.deprecated_imports = patterns;
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

        let lang = Self::detect_language(file_path);
        let import_lang = ImportLanguage::from_extension(file_path);

        // If neither the legacy detector nor the import extractor recognizes
        // this language, skip entirely.
        if lang.is_none() && import_lang.is_none() {
            return Ok(None);
        }

        let mut findings: Vec<(String, String)> = Vec::new();

        // --- Legacy pattern checks (Rust AST / Python regex) ---
        match lang {
            Some(DetectedLanguage::Rust) => {
                let mut parser = tree_sitter::Parser::new();
                parser
                    .set_language(&tree_sitter_rust::LANGUAGE.into())
                    .map_err(|e| T2Error::ParseError(format!("Rust parser init: {e}")))?;
                let tree = parser
                    .parse(source, None)
                    .ok_or_else(|| {
                        T2Error::ParseError("tree-sitter parse returned None".to_string())
                    })?;
                let root = tree.root_node();
                // CC spec Check 1: ERROR nodes in AST -> BLOCK
                if root.has_error() {
                    findings.push((
                        format!("ast_error_{}", file_path),
                        format!("AST contains ERROR nodes (unparseable code) in {}", file_path),
                    ));
                }
                self.check_rust_patterns(&root, source, &mut findings);
                let max_depth = self.max_nesting_depth(&root, 0);
                if max_depth > MAX_NESTING_DEPTH {
                    findings.push((
                        format!("deep_nesting_{max_depth}"),
                        format!(
                            "Nesting depth {max_depth} exceeds limit {MAX_NESTING_DEPTH}"
                        ),
                    ));
                }
            }
            Some(DetectedLanguage::Python) => {
                // Python security/lazy patterns via regex
                self.collect_python_regex_findings(source, &mut findings);
                // A-21: Python AST empty body detection via tree-sitter
                self.check_python_ast_completeness(source, file_path, &mut findings);
            }
            None => {} // No legacy checks for other languages
        }

        // --- A-14: AST import extraction + deprecated pattern cross-reference ---
        if let Some(il) = import_lang {
            if !self.deprecated_imports.is_empty() {
                self.check_deprecated_ast_imports(source, il, &mut findings);
            }
        }

        let duration = start.elapsed();
        if duration.as_millis() > 10 {
            tracing::warn!(duration_ms = ?duration.as_millis(), "T2 hook exceeded 10ms target");
        }

        if findings.is_empty() {
            return Ok(Some(HookVerdict::Allow));
        }

        // Classify findings: AST errors, deprecated imports, security -> BLOCK,
        // lazy code/unsafe -> WARN
        let has_ast_error = findings
            .iter()
            .any(|(id, _)| id.starts_with("ast_error"));
        let has_deprecated_block = findings
            .iter()
            .any(|(id, _)| id.starts_with("deprecated_import_BLOCK"));
        let has_security_block = findings
            .iter()
            .any(|(id, _)| id.starts_with("unwrap_call") || id.starts_with("eval_call"));
        let has_lazy_code = findings.iter().any(|(id, _)| {
            id.starts_with("lazy_") || id.starts_with("empty_fn_body")
        });

        let (pattern_id, message) = &findings[0];
        let verdict = if has_ast_error || has_deprecated_block || has_security_block {
            HookVerdict::Block {
                message: message.clone(),
                pattern_hash: blake3::hash(pattern_id.as_bytes()).to_hex().to_string(),
            }
        } else if has_lazy_code
            || findings.iter().any(|(id, _)| id.starts_with("unsafe_block"))
        {
            // Lazy code and unsafe blocks are WARN (may be legitimate in tests)
            HookVerdict::Warn {
                message: message.clone(),
                pattern_hash: blake3::hash(pattern_id.as_bytes()).to_hex().to_string(),
            }
        } else if findings
            .iter()
            .any(|(id, _)| id.starts_with("deprecated_import_WARN"))
        {
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
        Ok(Some(verdict))
    }

    /// Extract imports via tree-sitter AST and cross-reference against deprecated patterns.
    ///
    /// Appends findings to the provided vec. Each finding ID encodes severity
    /// (deprecated_import_BLOCK_... or deprecated_import_WARN_...).
    fn check_deprecated_ast_imports(
        &self,
        source: &str,
        language: ImportLanguage,
        findings: &mut Vec<(String, String)>,
    ) {
        let imports = match extract_imports(source, language) {
            Ok(i) => i,
            Err(e) => {
                tracing::debug!(
                    lang = ?language,
                    err = %e,
                    "import extraction failed, skipping deprecated check"
                );
                return;
            }
        };

        let violations =
            check_deprecated_imports(&imports, &self.deprecated_imports, language);

        for v in violations {
            findings.push((
                format!(
                    "deprecated_import_{}_L{}_{}",
                    v.severity, v.line, v.import_name
                ),
                format!(
                    "Deprecated import '{}' at line {} (use '{}' instead)",
                    v.deprecated_name, v.line, v.replacement_name
                ),
            ));
        }
    }

    /// Check Rust-specific AST patterns (security + lazy code).
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

        // Lazy code: todo!() and unimplemented!() macro invocations
        if node.kind() == "macro_invocation" {
            if let Some(macro_node) = node.child_by_field_name("macro") {
                let macro_text = &source[macro_node.byte_range()];
                if macro_text == "todo" || macro_text == "unimplemented" {
                    let start_line = node.start_position().row + 1;
                    findings.push((
                        format!("lazy_{macro_text}_L{start_line}"),
                        format!("{macro_text}!() placeholder at line {start_line}"),
                    ));
                }
            }
        }

        // Lazy code: empty function bodies (fn foo() {})
        // tree-sitter-rust: function_item has a "body" field (block node).
        // An empty body has only "{" and "}" tokens, zero named children.
        if node.kind() == "function_item" {
            if let Some(body) = node.child_by_field_name("body") {
                if body.named_child_count() == 0 {
                    let start_line = node.start_position().row + 1;
                    let fn_name = node
                        .child_by_field_name("name")
                        .map(|n| &source[n.byte_range()])
                        .unwrap_or("<unknown>");
                    findings.push((
                        format!("empty_fn_body_L{start_line}"),
                        format!("Empty function body '{}' at line {}", fn_name, start_line),
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

    /// Collect Python security + lazy code findings via regex.
    ///
    /// Appends to the provided findings vec rather than returning a verdict directly,
    /// so the caller can combine with import extraction findings.
    fn collect_python_regex_findings(
        &self,
        source: &str,
        findings: &mut Vec<(String, String)>,
    ) {
        // Security patterns
        static EVAL_RE: LazyLock<regex::Regex> =
            LazyLock::new(|| regex::Regex::new(r"\beval\s*\(").unwrap());
        static OS_SYSTEM_RE: LazyLock<regex::Regex> =
            LazyLock::new(|| regex::Regex::new(r"\bos\.system\s*\(").unwrap());
        static SUBPROCESS_SHELL_RE: LazyLock<regex::Regex> = LazyLock::new(|| {
            regex::Regex::new(r"subprocess\.(call|run|Popen)\s*\([^)]*shell\s*=\s*True").unwrap()
        });

        // Lazy code patterns (code-completeness enforcement)
        static BARE_PASS_RE: LazyLock<regex::Regex> =
            LazyLock::new(|| regex::Regex::new(r"(?m)\bpass\s*$").unwrap());
        static NOT_IMPLEMENTED_RE: LazyLock<regex::Regex> =
            LazyLock::new(|| regex::Regex::new(r"\braise NotImplementedError\b").unwrap());
        static ELLIPSIS_BODY_RE: LazyLock<regex::Regex> =
            LazyLock::new(|| regex::Regex::new(r"(?m)def\s+\w+\([^)]*\):\s*\.\.\.").unwrap());
        static TODO_COMMENT_RE: LazyLock<regex::Regex> =
            LazyLock::new(|| regex::Regex::new(r"#\s*TODO\b").unwrap());

        // Security findings (BLOCK severity)
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

        // Lazy code findings (WARN severity)
        if let Some(m) = BARE_PASS_RE.find(source) {
            let line = source[..m.start()].lines().count() + 1;
            findings.push((
                format!("lazy_bare_pass_L{line}"),
                format!("Bare pass statement at line {line}"),
            ));
        }

        if let Some(m) = NOT_IMPLEMENTED_RE.find(source) {
            let line = source[..m.start()].lines().count() + 1;
            findings.push((
                format!("lazy_not_implemented_L{line}"),
                format!("raise NotImplementedError at line {line}"),
            ));
        }

        if let Some(m) = ELLIPSIS_BODY_RE.find(source) {
            let line = source[..m.start()].lines().count() + 1;
            findings.push((
                format!("lazy_ellipsis_body_L{line}"),
                format!("Function body is just ... at line {line}"),
            ));
        }

        if let Some(m) = TODO_COMMENT_RE.find(source) {
            let line = source[..m.start()].lines().count() + 1;
            findings.push((
                format!("lazy_todo_comment_L{line}"),
                format!("TODO comment at line {line}"),
            ));
        }
    }

    /// A-21: Python AST completeness check via tree-sitter.
    ///
    /// Detects empty function bodies (function_definition with block containing
    /// only pass or ellipsis). Also checks for ERROR nodes.
    fn check_python_ast_completeness(
        &self,
        source: &str,
        file_path: &str,
        findings: &mut Vec<(String, String)>,
    ) {
        let mut parser = tree_sitter::Parser::new();
        if parser
            .set_language(&tree_sitter_python::LANGUAGE.into())
            .is_err()
        {
            tracing::debug!(file = file_path, "Python parser init failed");
            return;
        }
        let tree = match parser.parse(source, None) {
            Some(t) => t,
            None => return,
        };
        let root = tree.root_node();
        if root.has_error() {
            findings.push((
                format!("ast_error_{}", file_path),
                format!("Python AST contains ERROR nodes in {}", file_path),
            ));
        }
        self.check_python_empty_bodies(&root, source, findings);
    }

    /// Recursively find Python function_definition nodes with empty/stub bodies.
    fn check_python_empty_bodies(
        &self,
        node: &tree_sitter::Node<'_>,
        source: &str,
        findings: &mut Vec<(String, String)>,
    ) {
        if node.kind() == "function_definition" {
            if let Some(body) = node.child_by_field_name("body") {
                // A block with only `pass`, `...`, or comments is considered empty.
                let is_empty = body.named_child_count() == 0
                    || (body.named_child_count() == 1
                        && body.named_child(0).is_some_and(|c| {
                            let kind = c.kind();
                            kind == "pass_statement"
                                || kind == "ellipsis"
                                || kind == "expression_statement"
                                    && c.named_child(0)
                                        .is_some_and(|e| e.kind() == "ellipsis")
                        }));
                if is_empty {
                    let start_line = node.start_position().row + 1;
                    let fn_name = node
                        .child_by_field_name("name")
                        .map(|n| &source[n.byte_range()])
                        .unwrap_or("<unknown>");
                    findings.push((
                        format!("empty_fn_body_L{start_line}"),
                        format!(
                            "Empty/stub function body '{}' at line {}",
                            fn_name, start_line
                        ),
                    ));
                }
            }
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.check_python_empty_bodies(&child, source, findings);
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
    ///
    /// Prefer [`with_rules`] or call [`set_t1_rules`] after construction
    /// to populate T1 with active rules from rules.db.
    pub fn new(rule_engine: Box<dyn RuleEngine>, fsm_phase: FsmPhaseBit) -> Self {
        Self {
            t1: T1Hook::new(rule_engine, fsm_phase),
            t2: T2Hook::new(5000),
            t3: T3Hook::new(),
        }
    }

    /// Update T1 rules at runtime (e.g., after loading from rules.db).
    pub fn set_t1_rules(
        &mut self,
        rules: Vec<(elai_rules::rule_engine::Rule, elai_rules::rule_engine::CompiledPredicate)>,
    ) {
        self.t1.set_rules(rules);
    }

    /// Update T2 deprecated import patterns at runtime.
    pub fn set_t2_deprecated_imports(&mut self, patterns: Vec<DeprecatedImport>) {
        self.t2.set_deprecated_imports(patterns);
    }

    /// Update T1 file discipline config at runtime.
    pub fn set_file_discipline(&mut self, config: FileDisciplineConfig) {
        self.t1.set_file_discipline(config);
    }

    /// Access the T1 rule engine for filtering/compilation.
    pub fn rule_engine(&self) -> &dyn RuleEngine {
        &*self.t1.rule_engine
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

    /// Create with pre-loaded T1 rules and T2 deprecated import patterns.
    pub fn with_rules_and_deprecated_imports(
        rule_engine: Box<dyn RuleEngine>,
        fsm_phase: FsmPhaseBit,
        rules: Vec<(elai_rules::rule_engine::Rule, elai_rules::rule_engine::CompiledPredicate)>,
        deprecated_imports: Vec<DeprecatedImport>,
    ) -> Self {
        Self {
            t1: T1Hook::with_rules(rule_engine, fsm_phase, rules),
            t2: T2Hook::with_deprecated_imports(5000, deprecated_imports),
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

    // -- T2 lazy code detection (Rust AST) --

    #[test]
    fn test_t2_detects_todo_macro() {
        let hook = T2Hook::new(5000);
        let source = "fn main() {\n    todo!()\n}\n";
        let result = hook.evaluate(source, "src/main.rs").unwrap();
        assert!(result.is_some());
        let verdict = result.unwrap();
        assert!(verdict.is_warn(), "todo!() should produce WARN, got {:?}", verdict);
    }

    #[test]
    fn test_t2_detects_unimplemented_macro() {
        let hook = T2Hook::new(5000);
        let source = "fn compute() -> i32 {\n    unimplemented!()\n}\n";
        let result = hook.evaluate(source, "src/lib.rs").unwrap();
        assert!(result.is_some());
        let verdict = result.unwrap();
        assert!(verdict.is_warn(), "unimplemented!() should produce WARN, got {:?}", verdict);
    }

    #[test]
    fn test_t2_detects_empty_fn_body_rust() {
        let hook = T2Hook::new(5000);
        let source = "fn placeholder() {}\n";
        let result = hook.evaluate(source, "src/lib.rs").unwrap();
        assert!(result.is_some());
        let verdict = result.unwrap();
        assert!(verdict.is_warn(), "empty fn body should produce WARN, got {:?}", verdict);
    }

    #[test]
    fn test_t2_allows_clean_rust() {
        let hook = T2Hook::new(5000);
        let source = "fn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n";
        let result = hook.evaluate(source, "src/lib.rs").unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap(), HookVerdict::Allow);
    }

    // -- T2 lazy code detection (Python regex) --

    #[test]
    fn test_t2_detects_bare_pass_python() {
        let hook = T2Hook::new(5000);
        let source = "def handler():\n    pass\n";
        let result = hook.evaluate(source, "handler.py").unwrap();
        assert!(result.is_some());
        let verdict = result.unwrap();
        assert!(verdict.is_warn(), "bare pass should produce WARN, got {:?}", verdict);
    }

    #[test]
    fn test_t2_detects_not_implemented_python() {
        let hook = T2Hook::new(5000);
        let source = "def compute():\n    raise NotImplementedError\n";
        let result = hook.evaluate(source, "compute.py").unwrap();
        assert!(result.is_some());
        let verdict = result.unwrap();
        assert!(verdict.is_warn(), "raise NotImplementedError should produce WARN, got {:?}", verdict);
    }

    #[test]
    fn test_t2_detects_ellipsis_body_python() {
        let hook = T2Hook::new(5000);
        let source = "def stub(x): ...\n";
        let result = hook.evaluate(source, "stub.py").unwrap();
        assert!(result.is_some());
        let verdict = result.unwrap();
        assert!(verdict.is_warn(), "ellipsis body should produce WARN, got {:?}", verdict);
    }

    #[test]
    fn test_t2_detects_todo_comment_python() {
        let hook = T2Hook::new(5000);
        let source = "def process():\n    # TODO implement this\n    return None\n";
        let result = hook.evaluate(source, "process.py").unwrap();
        assert!(result.is_some());
        let verdict = result.unwrap();
        assert!(verdict.is_warn(), "TODO comment should produce WARN, got {:?}", verdict);
    }

    #[test]
    fn test_t2_security_overrides_lazy_python() {
        let hook = T2Hook::new(5000);
        // Source with both eval (security) and TODO (lazy)
        let source = "# TODO fix this\nresult = eval(user_input)\n";
        let result = hook.evaluate(source, "bad.py").unwrap();
        assert!(result.is_some());
        let verdict = result.unwrap();
        assert!(verdict.is_block(), "security finding should BLOCK even with lazy code, got {:?}", verdict);
    }

    #[test]
    fn test_t2_allows_clean_python() {
        let hook = T2Hook::new(5000);
        let source = "def add(a, b):\n    return a + b\n";
        let result = hook.evaluate(source, "math.py").unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap(), HookVerdict::Allow);
    }

    // -- T1 with loaded rules --

    #[test]
    fn test_t1_with_rules_evaluates() {
        use elai_rules::rule_engine::{DefaultRuleEngine, FsmPhaseBit, Rule};
        use elai_rules::stubs::{EmptyDkgQuerier, PassAstEvaluator};

        let engine = DefaultRuleEngine::new(
            Box::new(PassAstEvaluator),
            Box::new(EmptyDkgQuerier),
        );

        let rule = Rule {
            rule_id: "r-test-grep".into(),
            domain: "engineering".into(),
            tier: 1,
            severity: "WARN".into(),
            title: "Detects TODO".into(),
            predicate_json: r#"{"type":"grep_pattern","pattern":"TODO","scope":null,"context_lines":0}"#.into(),
            phase_mask: 0xFF,
            escalate_target: None,
        };

        let compiled = engine.compile_predicate(&rule.predicate_json).unwrap();

        let hook = T1Hook::with_rules(
            Box::new(engine),
            FsmPhaseBit::EXECUTING,
            vec![(rule, compiled)],
        );

        // Source with TODO should trigger WARN
        let result = hook.evaluate("file_write", "// TODO fix this", "src/lib.rs", HashMap::new());
        assert!(result.is_ok());
        let verdict = result.unwrap();
        assert!(verdict.is_warn(), "GREP_PATTERN TODO should produce WARN, got {:?}", verdict);
    }

    #[test]
    fn test_t1_empty_rules_allows_all() {
        use elai_rules::rule_engine::{DefaultRuleEngine, FsmPhaseBit};
        use elai_rules::stubs::{EmptyDkgQuerier, PassAstEvaluator};

        let engine = DefaultRuleEngine::new(
            Box::new(PassAstEvaluator),
            Box::new(EmptyDkgQuerier),
        );

        let hook = T1Hook::new(Box::new(engine), FsmPhaseBit::EXECUTING);

        // Use clean source — lazy patterns still fire even with empty rule set
        let result = hook.evaluate("file_write", "fn add(a: i32, b: i32) -> i32 { a + b }", "src/main.rs", HashMap::new());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), HookVerdict::Allow, "empty rules + clean source should allow");
    }

    #[test]
    fn test_t1_block_severity() {
        use elai_rules::rule_engine::{DefaultRuleEngine, FsmPhaseBit, Rule};
        use elai_rules::stubs::{EmptyDkgQuerier, PassAstEvaluator};

        let engine = DefaultRuleEngine::new(
            Box::new(PassAstEvaluator),
            Box::new(EmptyDkgQuerier),
        );

        let rule = Rule {
            rule_id: "r-block-eval".into(),
            domain: "security".into(),
            tier: 1,
            severity: "BLOCK".into(),
            title: "Block eval".into(),
            predicate_json: r#"{"type":"grep_pattern","pattern":"\\beval\\s*\\(","scope":null,"context_lines":0}"#.into(),
            phase_mask: 0xFF,
            escalate_target: None,
        };

        let compiled = engine.compile_predicate(&rule.predicate_json).unwrap();

        let hook = T1Hook::with_rules(
            Box::new(engine),
            FsmPhaseBit::EXECUTING,
            vec![(rule, compiled)],
        );

        let result = hook.evaluate("file_write", "result = eval(x)", "script.py", HashMap::new());
        assert!(result.is_ok());
        let verdict = result.unwrap();
        assert!(verdict.is_block(), "BLOCK severity rule should produce Block verdict, got {:?}", verdict);
    }

    // -- A-21: T1 lazy code detection --

    #[test]
    fn test_t1_lazy_detects_todo_comment_rust() {
        use elai_rules::rule_engine::{DefaultRuleEngine, FsmPhaseBit};
        use elai_rules::stubs::{EmptyDkgQuerier, PassAstEvaluator};
        let engine = DefaultRuleEngine::new(Box::new(PassAstEvaluator), Box::new(EmptyDkgQuerier));
        let hook = T1Hook::new(Box::new(engine), FsmPhaseBit::EXECUTING);
        let source = "fn main() {\n    // TODO implement\n}\n";
        let result = hook.evaluate("file_write", source, "src/main.rs", HashMap::new()).unwrap();
        assert!(result.is_warn(), "T1 should WARN on TODO comment, got {:?}", result);
    }

    #[test]
    fn test_t1_lazy_detects_rust_todo_macro() {
        use elai_rules::rule_engine::{DefaultRuleEngine, FsmPhaseBit};
        use elai_rules::stubs::{EmptyDkgQuerier, PassAstEvaluator};
        let engine = DefaultRuleEngine::new(Box::new(PassAstEvaluator), Box::new(EmptyDkgQuerier));
        let hook = T1Hook::new(Box::new(engine), FsmPhaseBit::EXECUTING);
        let source = "fn compute() -> i32 { todo!() }";
        let result = hook.evaluate("file_write", source, "src/lib.rs", HashMap::new()).unwrap();
        assert!(result.is_warn(), "T1 should WARN on todo!(), got {:?}", result);
    }

    #[test]
    fn test_t1_lazy_detects_python_pass() {
        use elai_rules::rule_engine::{DefaultRuleEngine, FsmPhaseBit};
        use elai_rules::stubs::{EmptyDkgQuerier, PassAstEvaluator};
        let engine = DefaultRuleEngine::new(Box::new(PassAstEvaluator), Box::new(EmptyDkgQuerier));
        let hook = T1Hook::new(Box::new(engine), FsmPhaseBit::EXECUTING);
        let source = "def handler():\n    pass\n";
        let result = hook.evaluate("file_write", source, "handler.py", HashMap::new()).unwrap();
        assert!(result.is_warn(), "T1 should WARN on bare pass, got {:?}", result);
    }

    #[test]
    fn test_t1_lazy_detects_ts_throw_not_implemented() {
        use elai_rules::rule_engine::{DefaultRuleEngine, FsmPhaseBit};
        use elai_rules::stubs::{EmptyDkgQuerier, PassAstEvaluator};
        let engine = DefaultRuleEngine::new(Box::new(PassAstEvaluator), Box::new(EmptyDkgQuerier));
        let hook = T1Hook::new(Box::new(engine), FsmPhaseBit::EXECUTING);
        let source = "function handler() { throw new Error('not implemented'); }";
        let result = hook.evaluate("file_write", source, "handler.ts", HashMap::new()).unwrap();
        assert!(result.is_warn(), "T1 should WARN on throw not implemented, got {:?}", result);
    }

    #[test]
    fn test_t1_lazy_skips_console_log_in_test_file() {
        use elai_rules::rule_engine::{DefaultRuleEngine, FsmPhaseBit};
        use elai_rules::stubs::{EmptyDkgQuerier, PassAstEvaluator};
        let engine = DefaultRuleEngine::new(Box::new(PassAstEvaluator), Box::new(EmptyDkgQuerier));
        let hook = T1Hook::new(Box::new(engine), FsmPhaseBit::EXECUTING);
        let source = "console.log('debug');\n";
        let result = hook.evaluate("file_write", source, "handler.test.ts", HashMap::new()).unwrap();
        assert_eq!(result, HookVerdict::Allow, "console.log in test files should be allowed");
    }

    #[test]
    fn test_t1_lazy_allows_clean_code() {
        use elai_rules::rule_engine::{DefaultRuleEngine, FsmPhaseBit};
        use elai_rules::stubs::{EmptyDkgQuerier, PassAstEvaluator};
        let engine = DefaultRuleEngine::new(Box::new(PassAstEvaluator), Box::new(EmptyDkgQuerier));
        let hook = T1Hook::new(Box::new(engine), FsmPhaseBit::EXECUTING);
        let source = "fn add(a: i32, b: i32) -> i32 { a + b }\n";
        let result = hook.evaluate("file_write", source, "src/math.rs", HashMap::new()).unwrap();
        assert_eq!(result, HookVerdict::Allow, "clean code should be allowed");
    }

    // -- A-21: T1 file size enforcement --

    #[test]
    fn test_t1_file_size_blocks_extreme_monolith() {
        use elai_rules::rule_engine::{DefaultRuleEngine, FsmPhaseBit};
        use elai_rules::stubs::{EmptyDkgQuerier, PassAstEvaluator};
        let engine = DefaultRuleEngine::new(Box::new(PassAstEvaluator), Box::new(EmptyDkgQuerier));
        let hook = T1Hook::new(Box::new(engine), FsmPhaseBit::EXECUTING);
        // Rust hard block is 400 lines
        let source = (0..450).map(|i| format!("let x{i} = {i};")).collect::<Vec<_>>().join("\n");
        let result = hook.evaluate("file_write", &source, "src/big.rs", HashMap::new()).unwrap();
        assert!(result.is_block(), "File >400 lines Rust should BLOCK, got {:?}", result);
    }

    #[test]
    fn test_t1_file_size_warns_oversized() {
        use elai_rules::rule_engine::{DefaultRuleEngine, FsmPhaseBit};
        use elai_rules::stubs::{EmptyDkgQuerier, PassAstEvaluator};
        let engine = DefaultRuleEngine::new(Box::new(PassAstEvaluator), Box::new(EmptyDkgQuerier));
        let hook = T1Hook::new(Box::new(engine), FsmPhaseBit::EXECUTING);
        // Rust production WARN is 100, hard block is 400
        let source = (0..150).map(|i| format!("let x{i} = {i};")).collect::<Vec<_>>().join("\n");
        let result = hook.evaluate("file_write", &source, "src/medium.rs", HashMap::new()).unwrap();
        assert!(result.is_warn(), "File >100 lines Rust should WARN, got {:?}", result);
    }

    #[test]
    fn test_t1_file_size_allows_small() {
        use elai_rules::rule_engine::{DefaultRuleEngine, FsmPhaseBit};
        use elai_rules::stubs::{EmptyDkgQuerier, PassAstEvaluator};
        let engine = DefaultRuleEngine::new(Box::new(PassAstEvaluator), Box::new(EmptyDkgQuerier));
        let hook = T1Hook::new(Box::new(engine), FsmPhaseBit::EXECUTING);
        let source = (0..50).map(|i| format!("let x{i} = {i};")).collect::<Vec<_>>().join("\n");
        let result = hook.evaluate("file_write", &source, "src/small.rs", HashMap::new()).unwrap();
        assert_eq!(result, HookVerdict::Allow, "Small file should be allowed");
    }

    #[test]
    fn test_t1_file_size_uses_test_limits() {
        use elai_rules::rule_engine::{DefaultRuleEngine, FsmPhaseBit};
        use elai_rules::stubs::{EmptyDkgQuerier, PassAstEvaluator};
        let engine = DefaultRuleEngine::new(Box::new(PassAstEvaluator), Box::new(EmptyDkgQuerier));
        let hook = T1Hook::new(Box::new(engine), FsmPhaseBit::EXECUTING);
        // Rust test WARN is 200, so 150 lines in a test file should be fine
        let source = (0..150).map(|i| format!("let x{i} = {i};")).collect::<Vec<_>>().join("\n");
        let result = hook.evaluate("file_write", &source, "tests/integration_test.rs", HashMap::new()).unwrap();
        assert_eq!(result, HookVerdict::Allow, "150 lines in test file (limit 200) should be allowed");
    }

    #[test]
    fn test_t1_file_size_not_checked_on_non_file_write() {
        use elai_rules::rule_engine::{DefaultRuleEngine, FsmPhaseBit};
        use elai_rules::stubs::{EmptyDkgQuerier, PassAstEvaluator};
        let engine = DefaultRuleEngine::new(Box::new(PassAstEvaluator), Box::new(EmptyDkgQuerier));
        let hook = T1Hook::new(Box::new(engine), FsmPhaseBit::EXECUTING);
        // 500 lines on a non-file_write action should not trigger size check
        let source = (0..500).map(|i| format!("let x{i} = {i};")).collect::<Vec<_>>().join("\n");
        let result = hook.evaluate("ssb_write", &source, "src/big.rs", HashMap::new()).unwrap();
        // Only lazy patterns would fire (no TODO/pass/etc in this source), so Allow
        assert_eq!(result, HookVerdict::Allow, "Non-file_write should skip file size check");
    }

    // -- A-21: T2 AST error node detection --

    #[test]
    fn test_t2_blocks_on_ast_error_nodes() {
        let hook = T2Hook::new(5000);
        // Invalid Rust syntax should produce error nodes
        let source = "fn main() { let x = ; }";
        let result = hook.evaluate(source, "src/bad.rs").unwrap();
        assert!(result.is_some());
        let verdict = result.unwrap();
        assert!(verdict.is_block(), "AST error nodes should BLOCK, got {:?}", verdict);
    }

    // -- A-21: T2 Python AST empty body detection --

    #[test]
    fn test_t2_detects_python_ast_empty_pass() {
        let hook = T2Hook::new(5000);
        let source = "def handler(request):\n    pass\n";
        let result = hook.evaluate(source, "handler.py").unwrap();
        assert!(result.is_some());
        let verdict = result.unwrap();
        assert!(verdict.is_warn(), "Python pass body should WARN via AST, got {:?}", verdict);
    }

    #[test]
    fn test_t2_detects_python_ast_ellipsis_body() {
        let hook = T2Hook::new(5000);
        let source = "def stub(x):\n    ...\n";
        let result = hook.evaluate(source, "stub.py").unwrap();
        assert!(result.is_some());
        let verdict = result.unwrap();
        assert!(verdict.is_warn(), "Python ... body should WARN via AST, got {:?}", verdict);
    }

    // -- A-21: helper function tests --

    #[test]
    fn test_detect_language() {
        assert_eq!(detect_language("src/main.rs"), Some("rust"));
        assert_eq!(detect_language("app.py"), Some("python"));
        assert_eq!(detect_language("handler.go"), Some("go"));
        assert_eq!(detect_language("index.ts"), Some("typescript"));
        assert_eq!(detect_language("app.jsx"), Some("javascript"));
        assert_eq!(detect_language("Main.java"), Some("java"));
        assert_eq!(detect_language("Util.cs"), Some("csharp"));
        assert_eq!(detect_language("README.md"), None);
    }

    #[test]
    fn test_is_test_file() {
        assert!(is_test_file("tests/integration.rs"));
        assert!(is_test_file("src/handler_test.rs"));
        assert!(is_test_file("test_handler.py"));
        assert!(is_test_file("handler_test.py"));
        assert!(is_test_file("handler_test.go"));
        assert!(is_test_file("handler.test.ts"));
        assert!(is_test_file("handler.spec.js"));
        assert!(is_test_file("__tests__/handler.ts"));
        assert!(is_test_file("UserTest.java"));
        assert!(is_test_file("spec/user_spec.rb"));
        assert!(!is_test_file("src/handler.rs"));
        assert!(!is_test_file("handler.py"));
        assert!(!is_test_file("main.go"));
    }

    #[test]
    fn test_file_discipline_config_defaults() {
        let config = FileDisciplineConfig::default();
        let rust = config.limits.get("rust").unwrap();
        assert_eq!(rust.production_warn, 100);
        assert_eq!(rust.test_warn, 200);
        assert_eq!(rust.hard_block, 400);
        let python = config.limits.get("python").unwrap();
        assert_eq!(python.production_warn, 120);
        assert_eq!(python.hard_block, 480);
    }
}

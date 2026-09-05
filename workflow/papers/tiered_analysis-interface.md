Source: elai_plugins::tiered_analysis
Reliability: HIGH (in-repo official source, reviewed)
Files examined:
  F:\.ELAI_workflow\.elai_cc\crates\plugins\src\tiered_analysis.rs (2775 lines)

Purpose:
  Tiered static analysis hooks for SSB write validation. T1 (rules, <1ms),
  T2 (tree-sitter AST, <10ms), T3 (pattern-match advisory, <5ms). No LLM in
  the hot path. All findings written to SSB as 'finding' entries.

Module-level invariants (from doc comments):
  - T1 fires on EVERY SSB write; target <1ms
  - T2 fires ONLY on action_type == "file_write"; target <10ms
  - T2 circuit-breaker: files > T2_CIRCUIT_BREAKER_LINES (5000) degrade to T1-only
  - T3 ADVISORY ONLY — never emits BLOCK; fires only when (WARN exists) AND (no BLOCK)
  - Verdict severity ordering: Block > Warn > Allow (merge_verdicts keeps max)
  - All BLOCK paths in T1/T2 may call MistakeRecorder (no-op default; SSB-backed in prod)
  - MistakeRecorder implementations MUST NOT panic; errors logged but never bubbled
  - Per-tier env kill switches: ELAI_DISABLE_T2, ELAI_DISABLE_T3 (truthy = 1|true|TRUE|on|ON)

Public type list (with field types):

  pub struct LanguageLimits {
    pub production_warn: usize,
    pub test_warn: usize,
    pub hard_block: usize,  // 4x production_warn per CC spec 3.2
  }
  derives: Debug, Clone, serde::Deserialize, serde::Serialize

  pub struct FileDisciplineConfig {
    pub limits: FxHashMap<String, LanguageLimits>,  // serde default
    pub exempt_extensions: Vec<String>,             // serde default
    pub exempt_paths: Vec<String>,                  // serde default
    pub max_cyclomatic_complexity: usize,           // default fn = 10
    pub max_coupling_score: usize,                  // default fn = 15
    pub language_limit_overrides: FxHashMap<String, LanguageLimits>,  // serde default
    pub accept_and_complete_enabled: bool,          // serde default (false)
  }
  derives: Debug, Clone, serde::Deserialize, serde::Serialize, Default (manual impl)

  pub trait MistakeRecorder: Send + Sync {
    fn record_mistake_event(
      &self,
      baby_step_id: &str,
      category: FailureCategory,  // re-export from elai_core::mistake_book
      file_path: &Path,
      snippet: &str,
    );
  }
  invariant: implementations MUST NOT panic; errors logged but never bubbled.

  pub struct NoopMistakeRecorder;
  impl MistakeRecorder for NoopMistakeRecorder {...intentional no-op...}

  pub struct MiniCriticPattern {
    pub id: &'static str,
    pub regex: &'static LazyLock<regex::Regex>,
    pub language: Option<&'static str>,  // None = universal
  }
  // borrowed-from-T1 7-pattern subset; no regex duplication per C-32 plan

  pub enum HookVerdict {
    Allow,
    Warn  { message: String, pattern_hash: String },
    Block { message: String, pattern_hash: String },
  }
  derives: Debug, Clone, PartialEq, Eq

  pub struct TieredAnalysisResult {
    pub t1_verdict: HookVerdict,
    pub t2_verdict: Option<HookVerdict>,
    pub t3_verdict: Option<HookVerdict>,
    pub final_verdict: HookVerdict,
    pub duration_ms: f64,
  }
  derives: Debug, Clone, PartialEq

  pub struct T1Hook {
    // all fields private:
    //   rule_engine: Box<dyn RuleEngine + Send + Sync>,
    //   fsm_phase: FsmPhaseBit,
    //   loaded_rules: Vec<(Rule, CompiledPredicate)>,
    //   file_discipline: FileDisciplineConfig,
    //   mistake_recorder: Option<Arc<dyn MistakeRecorder>>,
    //   current_baby_step_id: String,
  }

  pub enum T1Error {
    RuleEngineError(String),
  }
  derives: Debug; impls Display, std::error::Error, From<RuleEngineError>

  pub struct T2Hook {
    // all fields private:
    //   circuit_breaker_lines: usize,
    //   deprecated_imports: Vec<DeprecatedImport>,
    //   file_discipline: FileDisciplineConfig,
    //   mistake_recorder: Option<Arc<dyn MistakeRecorder>>,
    //   current_baby_step_id: String,
    //   file_plan: Option<Arc<FilePlan>>,
  }
  detects: AST ERROR nodes; unsafe blocks (Rust); eval()/os.system/subprocess shell=True (Py);
           deep nesting > MAX_NESTING_DEPTH (8); magic numbers (WARN); cyclomatic complexity
           per fn (BLOCK if > max); coupling per file (BLOCK if > max distinct ext symbols);
           fn-count vs FilePlan (BLOCK if actual > declared + FUNCTION_COUNT_SLACK).

  pub enum T2Error {
    ParseError(String),
    UnsupportedLanguage(String),
  }
  derives: Debug; impls Display, std::error::Error

  pub enum T3Category {
    Structural,
    AccessibilityHint,
    NamingConvention,
    TodoPublicApi,
  }
  derives: Debug, Clone, Copy, PartialEq, Eq

  pub struct T3Hook {
    // private: patterns: Vec<AmbiguousPattern>,
  }
  patterns shipped (5): swallow_exception, long_function, a11y_img_missing_alt,
                        naming_fn_camelcase, todo_on_public_api.

  pub struct TieredAnalysis {
    // private: t1: T1Hook, t2: T2Hook, t3: T3Hook,
  }

  pub enum TieredAnalysisError {
    T1Error(T1Error),
    T2Error(T2Error),
  }
  derives: Debug; impls Display, std::error::Error, From<T1Error>, From<T2Error>

Public constants:
  pub const FUNCTION_COUNT_SLACK: usize = 2;
  pub const MAX_FUNCTIONS_PER_FILE: u8 = elai_core::file_plan::MAX_FUNCTIONS_PER_FILE;  // re-export
  pub const ELAI_DISABLE_T2_ENV: &str = "ELAI_DISABLE_T2";
  pub const ELAI_DISABLE_T3_ENV: &str = "ELAI_DISABLE_T3";
  pub(crate) fn env_flag_true(name: &str) -> bool;

  Private constants (NOT exported but documented as design facts):
    DEFAULT_PRODUCTION_WARN = 150
    DEFAULT_TEST_WARN = 250
    DEFAULT_HARD_BLOCK = 600
    T2_CIRCUIT_BREAKER_LINES = 5000
    APPROACHING_LIMIT_RATIO = 0.8
    MAX_NESTING_DEPTH = 8

Public functions / methods:

  // Free functions
  pub fn mini_critic_patterns() -> &'static [MiniCriticPattern];

  // HookVerdict
  pub fn HookVerdict::is_block(&self) -> bool;
  pub fn HookVerdict::is_warn(&self) -> bool;
  pub fn HookVerdict::to_agent_error(&self) -> String;  // sanitized

  // T1Hook
  pub fn T1Hook::new(rule_engine: Box<dyn RuleEngine + Send + Sync>, fsm_phase: FsmPhaseBit) -> Self;
  pub fn T1Hook::with_rules(rule_engine: ..., fsm_phase: ..., rules: Vec<(Rule, CompiledPredicate)>) -> Self;
  pub fn T1Hook::set_rules(&mut self, rules: Vec<(Rule, CompiledPredicate)>);
  pub fn T1Hook::set_file_discipline(&mut self, config: FileDisciplineConfig);
  pub fn T1Hook::set_mistake_recorder(&mut self, recorder: Arc<dyn MistakeRecorder>);
  pub fn T1Hook::set_current_baby_step_id(&mut self, id: impl Into<String>);
  pub fn T1Hook::evaluate(
    &self,
    action: &str,
    source: &str,
    file_path: &str,
    metric_values: FxHashMap<String, f64>,
  ) -> Result<HookVerdict, T1Error>;
  // internal helpers: evaluate_lazy_patterns, evaluate_file_size, merge_verdicts, hash_pattern

  // T2Hook
  pub fn T2Hook::new(circuit_breaker_lines: usize) -> Self;
  pub fn T2Hook::with_deprecated_imports(circuit_breaker_lines: usize, deprecated_imports: Vec<DeprecatedImport>) -> Self;
  pub fn T2Hook::set_deprecated_imports(&mut self, patterns: Vec<DeprecatedImport>);
  pub fn T2Hook::set_file_discipline(&mut self, config: FileDisciplineConfig);
  pub fn T2Hook::set_mistake_recorder(&mut self, recorder: Arc<dyn MistakeRecorder>);
  pub fn T2Hook::set_current_baby_step_id(&mut self, id: impl Into<String>);
  pub fn T2Hook::set_file_plan(&mut self, plan: Arc<FilePlan>);
  pub fn T2Hook::evaluate(&self, source: &str, file_path: &str) -> Result<Option<HookVerdict>, T2Error>;
  // returns Ok(None) when circuit-breaker fires OR language unrecognized

  // T3Hook
  pub fn T3Hook::new() -> Self;
  pub fn T3Hook::evaluate(&self, source: &str, file_path: &str) -> Option<HookVerdict>;
  pub fn T3Hook::pattern_count(&self) -> usize;
  // T3Category::as_str(self) -> &'static str

  // TieredAnalysis (orchestrator)
  pub fn TieredAnalysis::new(rule_engine: Box<dyn RuleEngine + Send + Sync>, fsm_phase: FsmPhaseBit) -> Self;
  pub fn TieredAnalysis::with_rules(rule_engine, fsm_phase, rules) -> Self;
  pub fn TieredAnalysis::with_rules_and_deprecated_imports(rule_engine, fsm_phase, rules, deprecated_imports) -> Self;
  pub fn TieredAnalysis::set_t1_rules(&mut self, rules: Vec<(Rule, CompiledPredicate)>);
  pub fn TieredAnalysis::set_t2_deprecated_imports(&mut self, patterns: Vec<DeprecatedImport>);
  pub fn TieredAnalysis::set_file_discipline(&mut self, config: FileDisciplineConfig);
  pub fn TieredAnalysis::rule_engine(&self) -> &dyn RuleEngine;
  pub fn TieredAnalysis::analyze(
    &self,
    action_type: &str,
    source: Option<&str>,
    file_path: Option<&str>,
    metric_values: FxHashMap<String, f64>,
  ) -> Result<TieredAnalysisResult, TieredAnalysisError>;

Behavioral contracts (from reading code):

  T1Hook::evaluate
    1. Builds EvaluationContext (action, source, file_path, fsm_phase, metric_values).
    2. Calls rule_engine.evaluate(loaded_rules, ctx) -> list of verdicts.
    3. Folds verdicts into single HookVerdict (BLOCK > WARN > ALLOW).
    4. If source not empty: appends evaluate_lazy_patterns (per-language regex registry, ~25 patterns).
    5. If action == "file_write" AND source not empty: appends evaluate_file_size (per-lang limits + exemptions).
    6. Logs warn if duration > 1ms.
    Side effect: may call mistake_recorder.record_mistake_event on FileSizeExceeded BLOCK (carries current_baby_step_id).

  T2Hook::evaluate
    1. Counts lines; if > circuit_breaker_lines -> returns Ok(None).
    2. Detects DetectedLanguage (Rust/Python/Go/TypeScript/Java/CSharp/Ruby) AND ImportLanguage.
       If both None -> returns Ok(None).
    3. If lang detected -> parses with tree-sitter. ParseError surfaced as T2Error::ParseError.
       Check 1 (ERROR nodes -> BLOCK), Check 6a (cyclomatic), Check 6b (coupling),
       Check 3 (fn count vs FilePlan if installed).
       Language-specific deep checks (Rust patterns, Python regex+empty-body+magic-numbers).
    4. If import_lang detected and deprecated_imports non-empty: cross-references.
    5. Classifies findings into BLOCK/WARN; emits Some(HookVerdict).
    6. If verdict is BLOCK and recorder set: records EmptyBody / LazyCode (other BLOCK paths skipped per spec).
    Logs warn if duration > 10ms.

  T3Hook::evaluate
    1. Iterates patterns; first match wins.
    2. AmbiguousPattern with secondary_absent: primary regex must match AND secondary regex must
       NOT appear inside the matched span (rust-regex has no look-around).
    3. Returns Some(HookVerdict::Warn { message: "[<category>] <message>", pattern_hash }).
    Logs warn if duration > 5ms.

  TieredAnalysis::analyze
    Order: T1 always. T2 only if (!ELAI_DISABLE_T2_ENV && action_type=="file_write" && source!=None).
    T3 only if (!ELAI_DISABLE_T3_ENV && has_warn && !has_block && source!=None).
    Final verdict: BLOCK from any tier wins; else WARN from T1>T2>T3; else Allow.

Lazy code regex registry (T1, ~25 patterns, identified by LazyPattern.id):
  Universal: todo_slash, fixme_slash, hack_slash, ellipsis_slash, rest_of_slash,
             todo_hash, fixme_hash, ellipsis_hash, placeholder_str
  Rust:      rust_todo_macro, rust_unimplemented, rust_empty_fn
  Python:    py_bare_pass, py_not_implemented, py_ellipsis_body
  TS/JS:     ts_not_implemented, ts_console_log (exempt in test files), js_not_implemented, js_console_log
  Go:        go_panic_placeholder, go_empty_fn
  Java:      java_unsupported_op, java_todo_exception
  C#:        cs_not_implemented

MiniCritic 7-pattern subset (C-32 §Task 2a, borrowed regex, no duplication):
  todo_slash, fixme_slash, todo_hash, ellipsis_slash,
  rust_todo_macro, rust_unimplemented, py_bare_pass

Detected languages (T2):
  enum DetectedLanguage (private): Rust, Python, Go, TypeScript, Java, CSharp, Ruby.
  Extension map: .rs->Rust, .py/.pyi->Python, .go->Go, .ts/.tsx->TypeScript, .java->Java,
                 .cs->CSharp, .rb->Ruby.

File-discipline default limits (per spec 3.2):
  rust:       100/200/400
  go:         100/200/400
  python:     120/250/480
  typescript: 150/250/600
  javascript: 150/250/600
  java:       200/300/800
  kotlin:     150/250/600
  csharp:     200/300/800
  c:          150/250/600
  cpp:        150/250/600
  ruby:       120/200/480
  elixir:     120/200/480
  swift:      150/250/600

Default exempt extensions (T1 file size only):
  .toml .json .yaml .yml .xml .lock .sql .proto .graphql

Default exempt path prefixes:
  generated/ build/ vendor/ node_modules/ migrations/ target/

Cross-file dependencies (consumers of T1Hook/T2Hook/T3Hook/HookVerdict/TieredAnalysis):
  .elai_cc\crates\plugins\src\ipc_hooks.rs
  .elai_cc\crates\plugins\benches\tiered_analysis_bench.rs
  .elai_cc\crates\orchestrator\src\deterministic_auditor.rs
  .elai_cc\crates\orchestrator\src\hook_dispatcher.rs
  .elai_cc\crates\orchestrator\src\mini_critic.rs
  .elai_cc\crates\orchestrator\src\focus_tunnel\streaming.rs
  .elai_cc\crates\contracts\tests\tiered_analysis_contract.rs
  .elai_cc\crates\contracts\tests\plugins_hooks.rs
  .elai_cc\crates\contracts\tests\plugins_adversarial.rs
  .elai_cc\crates\contracts\tests\c31_file_plan_contract.rs

Upstream dependencies (this module consumes):
  blake3 (pattern hashing)
  elai_rules::predicate::VerdictResult
  elai_rules::rule_engine::{EvaluationContext, FsmPhaseBit, RuleEngine, Rule, CompiledPredicate, RuleEngineError}
  rustc_hash::FxHashMap
  elai_core::file_plan::{FilePlan, MAX_FUNCTIONS_PER_FILE}
  elai_core::mistake_book::FailureCategory
  elai_core::{ClockProvider as _, OsClock}
  crate::import_extractor::{check_deprecated_imports, extract_imports, DeprecatedImport, ImportLanguage}
  tree_sitter + tree_sitter_{rust,python,go,typescript,java,c_sharp,ruby}
  regex

Platform constraints:
  - tree-sitter grammars are linked in at build time via tree-sitter-{lang} crates.
  - All blake3 hashes are produced via the blake3 crate (NOT blake2 / sha2) per CLAUDE.md.
  - LazyLock<Regex> .unwrap() exempt from production-unwrap ban per
    CLAUDE-md-EXCEPTION: static-init-with-literal-regex (literal patterns, compiled once).

Gotchas:
  - T2 returns Ok(None) on circuit-breaker OR unrecognized language — callers MUST treat
    None as "T2 did not contribute", NOT as Allow.
  - T3 NEVER emits BLOCK — categorical invariant; AmbiguousPattern.matches() returns bool but
    the wrapper always builds HookVerdict::Warn.
  - T2 deep checks (magic numbers, nesting) are RUST + PYTHON ONLY today. Other detected
    languages get Check 1 + Check 6a + Check 6b only. Plan reviewers must not assume coverage
    parity across languages.
  - FilePlan-driven Check 3 silently no-ops if FilePlan not installed OR file path not matched.
    Path-suffix match (string compare; normalised slashes), no canonicalisation.
  - MistakeRecorder is OPTIONAL; default None means no MistakeBook bucket key is stamped.
    Production wiring lives in task_runner or a thin orchestrator adapter.
  - T1 lazy-code patterns and T2 Rust empty-fn detector OVERLAP (regex vs AST); both fire.
    The merge_verdicts contract collapses to highest severity at finalisation.
  - tree-sitter ERROR nodes BLOCK in T2 — unparseable source is hard-rejected (cannot be a WARN).

Critical assertions (testable, Rust syntax):
  - assert!(matches!(HookVerdict::Allow.is_block(), false));
  - assert!(matches!(HookVerdict::Block{..}.is_block(), true));
  - assert!(matches!(HookVerdict::Warn{..}.is_warn(), true));
  - assert!(T3Hook::new().pattern_count() == 5);   // 5 shipped advisory patterns
  - assert!(mini_critic_patterns().len() == 7);    // C-32 7-pattern subset
  - assert!(FileDisciplineConfig::default().max_cyclomatic_complexity == 10);
  - assert!(FileDisciplineConfig::default().max_coupling_score == 15);
  - assert!(FUNCTION_COUNT_SLACK == 2);
  - assert!(T2Hook::new(5000).evaluate(huge_10k_lines_src, "x.rs").unwrap().is_none());
  - assert!(TieredAnalysis::new(engine, phase).analyze("file_write", Some(""), Some("x.rs"), ..).unwrap()
           .final_verdict == HookVerdict::Allow);
  - For PI work: any new PiT2Hook / PiT3Hook MUST NOT collide on existing names; suggested
    placement is in elai_plugins (sibling type, NOT inside T2Hook impl). Re-use HookVerdict
    + TieredAnalysisResult shape so the dispatch contract stays uniform.

PI design hooks (relevant to phase-3L):
  - ProjectIntelligenceHook should follow same shape as T1Hook/T2Hook/T3Hook:
    private fields, *with_*/set_*/new constructors, single-method `evaluate(&self, ...) -> Result<_, _>`,
    optional Arc<dyn MistakeRecorder> for BLOCK events.
  - CorpusView is NOT yet present in this file — safe to introduce as a new type under
    elai_plugins or a sibling module without name collision.
  - PiT2Hook (proposed name) should NOT shadow T2Hook; place in a sibling module
    (e.g. crate::pi_analysis) with re-exports from lib.rs.
  - HookVerdict + TieredAnalysisResult are the canonical verdict types — re-use them.

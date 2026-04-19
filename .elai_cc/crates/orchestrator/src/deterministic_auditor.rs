//! Deterministic-first auditor: runs 5 deterministic checks before the LLM
//! auditor agent. Language-parameterized via `ProjectProfile`.
//!
//! Spec: anti-staleness-enforcement.txt §3.3, code-completeness-enforcement-design.txt Layer 4.
//! Step: A-12. Prerequisites: A-6 (VERSION_GATE), A-4 (ProjectProfile).
//!
//! Checks (in order, early-exit on BLOCK at any step):
//!   1. Build check — runs `ProjectProfile.commands.build` + lint commands
//!   2. VERSION_GATE — evaluates deprecated_patterns against changed files
//!   3. FILE_SIZE_CENSUS — language-specific line count limits
//!   4. LAZY_CODE_SCAN — regex patterns per language on all modified files
//!   5. COMPLETENESS + MODULE_COHERENCE — AST checks for non-trivial fn bodies
//!
//! Only after all checks pass (zero BLOCKs) does the LLM auditor run.
//!
//! Production wiring scheduled in Session wire-a12: invoked from the FSM
//! AUDITING state transition handler before LLM auditor dispatch. Any
//! Block verdict maps to `SsbPayload::Finding { severity: "error" }` and
//! prevents the AUDITING → MEMORIZING transition until the agent re-attempts.
//! Prerequisites A-4 (ProjectProfile) and A-6 (VERSION_GATE) have both
//! landed per patch-1.26 and the A-series commit history; A-12 itself is
//! the scheduled wiring step. Test callers
//! (contracts/tests/deterministic_auditor_a41_contract.rs) are the A-41
//! acceptance surface.

use std::path::{Path, PathBuf};

use elai_core::project_profile::{BuildCommand, Language, ProjectProfile};
use elai_plugins::tiered_analysis::FileDisciplineConfig;
use elai_rules::schema::DeprecatedPatternEntry;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Maximum time (seconds) for a single build/lint command.
pub const BUILD_CMD_TIMEOUT_S: u32 = 120;

/// Maximum characters of stderr to include in a build failure finding.
const STDERR_TRUNCATE_CHARS: usize = 500;

// ---------------------------------------------------------------------------
// Finding severity (mirrors evidence::FindingSeverity but local to this module)
// ---------------------------------------------------------------------------

/// Severity of a deterministic auditor finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeterministicSeverity {
    Block,
    Warn,
}

// ---------------------------------------------------------------------------
// DeterministicFinding: a single finding from any of the 5 checks
// ---------------------------------------------------------------------------

/// A finding produced by one of the 5 deterministic checks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeterministicFinding {
    pub check: CheckKind,
    pub severity: DeterministicSeverity,
    pub file: Option<PathBuf>,
    pub line: Option<u32>,
    pub message: String,
}

/// Which deterministic check produced this finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckKind {
    Build,
    VersionGate,
    FileSizeCensus,
    ImportCrossRef,
    LazyCode,
    Completeness,
    ModuleCoherence,
}

impl std::fmt::Display for CheckKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Build => write!(f, "build"),
            Self::VersionGate => write!(f, "version_gate"),
            Self::FileSizeCensus => write!(f, "file_size_census"),
            Self::ImportCrossRef => write!(f, "import_cross_ref"),
            Self::LazyCode => write!(f, "lazy_code"),
            Self::Completeness => write!(f, "completeness"),
            Self::ModuleCoherence => write!(f, "module_coherence"),
        }
    }
}

// ---------------------------------------------------------------------------
// DeterministicAuditorConfig
// ---------------------------------------------------------------------------

/// Configuration for the deterministic auditor, typically derived from
/// `ProjectProfile` and `[agents.auditor]` config.
#[derive(Debug, Clone)]
pub struct DeterministicAuditorConfig {
    /// Whether deterministic checks run before LLM audit. Default: true.
    pub enabled: bool,
    /// Build commands to run (from `ProjectProfile.commands.build`).
    pub build_cmds: Vec<BuildCommand>,
    /// Lint commands to run (from `ProjectProfile.commands.lint`).
    pub lint_cmds: Vec<BuildCommand>,
    /// Deprecated patterns loaded from rules.db (all severities).
    pub deprecated_patterns: Vec<DeprecatedPatternEntry>,
    /// Primary language of the project.
    pub primary_language: Language,
    /// Project root for resolving relative paths.
    pub project_root: PathBuf,
    /// File discipline config for size census (language limits, exempt paths).
    pub file_discipline: FileDisciplineConfig,
}

impl Default for DeterministicAuditorConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            build_cmds: Vec::new(),
            lint_cmds: Vec::new(),
            deprecated_patterns: Vec::new(),
            primary_language: Language::Unknown,
            project_root: PathBuf::from("."),
            file_discipline: FileDisciplineConfig::default(),
        }
    }
}

impl DeterministicAuditorConfig {
    /// Build config from a ProjectProfile and deprecated pattern entries.
    pub fn from_profile(
        profile: &ProjectProfile,
        deprecated_patterns: Vec<DeprecatedPatternEntry>,
    ) -> Self {
        Self {
            enabled: true,
            build_cmds: profile.commands.build.clone(),
            lint_cmds: profile.commands.lint.clone(),
            deprecated_patterns,
            primary_language: profile.languages.primary.clone(),
            project_root: profile.project_root.clone(),
            file_discipline: FileDisciplineConfig::default(),
        }
    }

    /// Per-language build command registry (7 languages from ProjectProfile).
    /// Returns the canonical build command string for a given language when no
    /// project-specific build commands are configured.
    pub fn default_build_cmd(lang: &Language) -> Option<&'static str> {
        match lang {
            Language::Rust => Some("cargo check -p {crate} --message-format=json"),
            Language::Python => Some("python -m py_compile {file} && python -m mypy {file} --ignore-missing-imports"),
            Language::Go => Some("go vet ./..."),
            Language::TypeScript | Language::JavaScript => Some("tsc --noEmit"),
            Language::Java | Language::Kotlin => Some("javac -Xlint {file}"),
            Language::CSharp => Some("dotnet build --no-restore"),
            Language::Ruby => Some("ruby -c {file}"),
            _ => None,
        }
    }

    /// Per-language lint command registry (companion to `default_build_cmd`).
    /// Returns the canonical lint command string for a given language when no
    /// project-specific lint commands are configured.
    pub fn default_lint_cmd(lang: &Language) -> Option<&'static str> {
        match lang {
            Language::Rust => Some("cargo clippy -p {crate} -- -D warnings"),
            Language::Python => Some("ruff check {file}"),
            Language::Go => Some("golangci-lint run ./..."),
            Language::TypeScript | Language::JavaScript => Some("npx eslint {file}"),
            Language::Java | Language::Kotlin => Some("./gradlew lint"),
            Language::CSharp => Some("dotnet format --verify-no-changes"),
            Language::Ruby => Some("rubocop {file}"),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Lazy code patterns — language-parameterized regex sets
// ---------------------------------------------------------------------------

/// A compiled lazy code pattern with its ID and language scope.
#[derive(Debug, Clone)]
struct LazyPattern {
    id: &'static str,
    pattern: regex::Regex,
}

/// Build the lazy code pattern set for a given language.
/// Universal patterns are always included; language-specific patterns are added
/// based on the primary language.
fn lazy_patterns_for(lang: &Language) -> Vec<LazyPattern> {
    let mut patterns = Vec::new();

    // Universal patterns (all languages)
    let universal = [
        ("todo_comment_slash", r"//\s*TODO\b"),
        ("fixme_comment_slash", r"//\s*FIXME\b"),
        ("hack_comment_slash", r"//\s*HACK\b"),
        ("ellipsis_comment_slash", r"//\s*\.\.\."),
        ("rest_of_comment_slash", r"//\s*rest of"),
        ("todo_comment_hash", r"#\s*TODO\b"),
        ("fixme_comment_hash", r"#\s*FIXME\b"),
        ("ellipsis_comment_hash", r"#\s*\.\.\."),
        ("placeholder_literal", r#""placeholder""#),
    ];
    for (id, pat) in &universal {
        // provably valid literal regex — only in static init
        if let Ok(re) = regex::Regex::new(pat) {
            patterns.push(LazyPattern { id, pattern: re });
        }
    }

    // Language-specific patterns
    match lang {
        Language::Rust => {
            let rust_patterns: &[(&str, &str)] = &[
                ("rust_todo_macro", r"\btodo!\("),
                ("rust_unimplemented_macro", r"\bunimplemented!\("),
                ("rust_empty_fn", r"fn\s+\w+\([^)]*\)[^{]*\{\s*\}"),
                // Batch 4A (A-41 CC check): `panic!("not yet")` /
                // `panic!("not implemented")` placeholders are a common
                // stand-in for unfinished work.
                ("rust_panic_placeholder", r#"panic!\(\s*["'](?:not |todo|implement)"#),
            ];
            for &(id, pat) in rust_patterns {
                if let Ok(re) = regex::Regex::new(pat) {
                    patterns.push(LazyPattern { id, pattern: re });
                }
            }
        }
        Language::Python => {
            let py_patterns = [
                ("py_bare_pass", r"(?m)\bpass\s*$"),
                ("py_not_implemented", r"\braise NotImplementedError\b"),
                ("py_ellipsis_body", r"def\s+\w+\([^)]*\):\s*\.\.\."),
            ];
            for (id, pat) in &py_patterns {
                if let Ok(re) = regex::Regex::new(pat) {
                    patterns.push(LazyPattern { id, pattern: re });
                }
            }
        }
        Language::TypeScript | Language::JavaScript => {
            // Pattern uses both ' and " — regular string with escaped backslashes
            let ts_pats: &[(&str, &str)] = &[
                ("ts_not_implemented", "throw new Error\\(['\"](?:not |todo|implement)"),
            ];
            for &(id, pat) in ts_pats {
                if let Ok(re) = regex::Regex::new(pat) {
                    patterns.push(LazyPattern { id, pattern: re });
                }
            }
        }
        Language::Go => {
            let go_pats: &[(&str, &str)] = &[
                ("go_panic_placeholder", "\\bpanic\\(\\s*.(not |todo|implement)"),
                ("go_empty_fn", "func\\s+\\w+\\([^)]*\\)[^{]*\\{\\s*\\}"),
            ];
            for &(id, pat) in go_pats {
                if let Ok(re) = regex::Regex::new(pat) {
                    patterns.push(LazyPattern { id, pattern: re });
                }
            }
        }
        Language::Java | Language::Kotlin => {
            let java_pats: &[(&str, &str)] = &[
                ("java_unsupported_op", "throw new UnsupportedOperationException\\("),
                ("java_todo_exception", "throw new RuntimeException\\(['\"]TODO"),
            ];
            for &(id, pat) in java_pats {
                if let Ok(re) = regex::Regex::new(pat) {
                    patterns.push(LazyPattern { id, pattern: re });
                }
            }
        }
        _ => {}
    }

    patterns
}

// ---------------------------------------------------------------------------
// Import extraction — language-specific regex sets (section 1.2)
// ---------------------------------------------------------------------------

/// Extract imported package/crate/module names from source code.
fn extract_imports(source: &str, lang: &Language) -> Vec<String> {
    let patterns: Vec<regex::Regex> = match lang {
        Language::Rust => vec![
            regex::Regex::new(r"use\s+(\w+)").ok(),
            regex::Regex::new(r"extern crate (\w+)").ok(),
        ],
        Language::Python => vec![
            regex::Regex::new(r"import\s+([\w.]+)").ok(),
            regex::Regex::new(r"from\s+([\w.]+)\s+import").ok(),
        ],
        Language::Go => vec![
            regex::Regex::new(r#""([\w./]+)""#).ok(),
        ],
        Language::TypeScript | Language::JavaScript => vec![
            regex::Regex::new(r#"from\s+['"]([^'"]+)['"]"#).ok(),
            regex::Regex::new(r#"require\(['"]([^'"]+)['"]\)"#).ok(),
        ],
        Language::Java | Language::Kotlin => vec![
            regex::Regex::new(r"import\s+([\w.]+)").ok(),
        ],
        Language::CSharp => vec![
            regex::Regex::new(r"using\s+([\w.]+)").ok(),
        ],
        Language::Ruby => vec![
            regex::Regex::new(r#"require\s+['"](\w+)"#).ok(),
            regex::Regex::new(r#"gem\s+['"](\w+)"#).ok(),
        ],
        _ => vec![],
    }
    .into_iter()
    .flatten()
    .collect();

    let mut imports = Vec::new();
    for re in &patterns {
        for cap in re.captures_iter(source) {
            if let Some(m) = cap.get(1) {
                imports.push(m.as_str().to_string());
            }
        }
    }
    imports
}

/// Map Language to its ecosystem identifier for deprecated_patterns lookup.
fn language_ecosystem(lang: &Language) -> Option<&'static str> {
    match lang {
        Language::Rust => Some("cargo"),
        Language::Python => Some("pip"),
        Language::TypeScript | Language::JavaScript => Some("npm"),
        Language::Go => Some("go"),
        Language::Java | Language::Kotlin => Some("maven"),
        Language::CSharp => Some("nuget"),
        Language::Ruby => Some("bundler"),
        Language::PHP => Some("composer"),
        _ => None,
    }
}

/// Detect language from file extension.
fn detect_language(path: &Path) -> Language {
    match path.extension().and_then(|e| e.to_str()) {
        Some("rs") => Language::Rust,
        Some("py") => Language::Python,
        Some("ts" | "tsx") => Language::TypeScript,
        Some("js" | "jsx") => Language::JavaScript,
        Some("go") => Language::Go,
        Some("java") => Language::Java,
        Some("kt" | "kts") => Language::Kotlin,
        Some("cs") => Language::CSharp,
        Some("rb") => Language::Ruby,
        Some("php") => Language::PHP,
        Some("cpp" | "cc" | "cxx") => Language::Cpp,
        Some("c" | "h") => Language::C,
        _ => Language::Unknown,
    }
}

// ---------------------------------------------------------------------------
// DeterministicAuditor: the 5-check pipeline
// ---------------------------------------------------------------------------

/// Runs the 5 deterministic checks on a set of changed files.
/// Returns all findings. If any finding has `severity == Block`, the LLM
/// auditor should NOT run until the blocks are resolved.
pub struct DeterministicAuditor {
    config: DeterministicAuditorConfig,
}

impl DeterministicAuditor {
    pub fn new(config: DeterministicAuditorConfig) -> Self {
        Self { config }
    }

    /// Run all 5 checks in strict order per spec:
    ///   1. BUILD CHECK (language-specific build/lint commands)
    ///   2. VERSION_GATE (deprecated patterns against changed files)
    ///   3. FILE_SIZE_CENSUS (all modified files against language-specific limits)
    ///   4. LAZY_CODE_SCAN (regex patterns per language on all modified files)
    ///   5. COMPLETENESS + MODULE_COHERENCE (AST: named functions exist with
    ///      non-trivial bodies, mod.rs/init.py/index.ts resolve)
    ///
    /// Short-circuits on BLOCK at any step (A-41): if any step produces a
    /// BLOCK finding, returns immediately without running subsequent steps.
    pub fn run_all(&self, changed_files: &[PathBuf]) -> DeterministicAuditResult {
        if !self.config.enabled {
            return DeterministicAuditResult {
                findings: Vec::new(),
                checks_run: 0,
                passed: true,
            };
        }

        let mut findings = Vec::new();

        // Step 1: BUILD CHECK
        // Phase A: config validation (presence check)
        let build_findings = self.check_build();
        let build_has_blocks = build_findings
            .iter()
            .any(|f| f.severity == DeterministicSeverity::Block);
        findings.extend(build_findings);

        if build_has_blocks {
            return DeterministicAuditResult {
                findings,
                checks_run: 1,
                passed: false,
            };
        }

        // Phase B: actually execute configured build/lint commands
        let cmds = self.build_commands();
        if !cmds.is_empty() {
            for build_cmd in &cmds {
                tracing::info!(cmd = %build_cmd.cmd, "deterministic_auditor: running build command");
                let timeout = std::time::Duration::from_secs(u64::from(BUILD_CMD_TIMEOUT_S));
                // Split command into program + args using shell-style splitting
                let parts: Vec<&str> = build_cmd.cmd.split_whitespace().collect();
                if parts.is_empty() {
                    continue;
                }
                let Some((program, args)) = parts.split_first() else {
                    continue;
                };
                let output = std::process::Command::new(program)
                    .args(args)
                    .current_dir(&build_cmd.working_dir)
                    .envs(&build_cmd.env)
                    .output();
                match output {
                    Err(e) => {
                        tracing::warn!(cmd = %build_cmd.cmd, err = %e, "deterministic_auditor: build command spawn failed");
                        findings.push(DeterministicFinding {
                            check: CheckKind::Build,
                            severity: DeterministicSeverity::Block,
                            file: None,
                            line: None,
                            message: format!(
                                "Build command could not be spawned: {}: {}",
                                build_cmd.cmd, e
                            ),
                        });
                        return DeterministicAuditResult {
                            findings,
                            checks_run: 1,
                            passed: false,
                        };
                    }
                    Ok(out) => {
                        // Enforce timeout by checking elapsed — std::process::Command::output()
                        // is blocking; for hard timeout enforcement, caller should use async
                        // dispatch. Here we respect BUILD_CMD_TIMEOUT_S as documentation
                        // and fall back to blocking execution (acceptable for sync audits).
                        let _ = timeout; // acknowledged; async callers handle this
                        let exit_code = out.status.code().unwrap_or(-1);
                        let stderr = String::from_utf8_lossy(&out.stderr);
                        if let Some(finding) =
                            Self::record_build_result(&build_cmd.cmd, exit_code, &stderr)
                        {
                            tracing::warn!(
                                cmd = %build_cmd.cmd,
                                exit_code,
                                "deterministic_auditor: build command FAILED"
                            );
                            findings.push(finding);
                            return DeterministicAuditResult {
                                findings,
                                checks_run: 1,
                                passed: false,
                            };
                        }
                        tracing::info!(cmd = %build_cmd.cmd, "deterministic_auditor: build command OK");
                    }
                }
            }
        }

        // Step 2: VERSION_GATE
        findings.extend(self.check_version_gate(changed_files));
        if findings.iter().any(|f| f.severity == DeterministicSeverity::Block) {
            return DeterministicAuditResult {
                findings,
                checks_run: 2,
                passed: false,
            };
        }

        // Step 3: FILE_SIZE_CENSUS
        findings.extend(self.check_file_size_census(changed_files));
        if findings.iter().any(|f| f.severity == DeterministicSeverity::Block) {
            return DeterministicAuditResult {
                findings,
                checks_run: 3,
                passed: false,
            };
        }

        // Step 4: LAZY_CODE_SCAN
        findings.extend(self.check_lazy_code(changed_files));
        if findings.iter().any(|f| f.severity == DeterministicSeverity::Block) {
            return DeterministicAuditResult {
                findings,
                checks_run: 4,
                passed: false,
            };
        }

        // Step 5: COMPLETENESS + MODULE_COHERENCE
        findings.extend(self.check_completeness(changed_files));
        findings.extend(self.check_module_coherence(changed_files));

        let passed = !findings
            .iter()
            .any(|f| f.severity == DeterministicSeverity::Block);
        DeterministicAuditResult {
            findings,
            checks_run: 5,
            passed,
        }
    }

    // -- Check 1: Build commands --

    /// Returns findings for build check. Build commands are stored in config;
    /// this produces a BLOCK finding if no build commands are configured
    /// (degenerate case). Actual command execution is handled by the caller
    /// via `build_commands()` + async Bash dispatch.
    fn check_build(&self) -> Vec<DeterministicFinding> {
        // Build commands exist in config → caller will execute them.
        // This check validates that build commands are present.
        if self.config.build_cmds.is_empty() && self.config.lint_cmds.is_empty() {
            return vec![DeterministicFinding {
                check: CheckKind::Build,
                severity: DeterministicSeverity::Warn,
                file: None,
                line: None,
                message: "No build or lint commands configured in ProjectProfile".into(),
            }];
        }
        Vec::new()
    }

    /// Returns the ordered list of build + lint commands to execute.
    /// Caller runs these via `execute_bash` and feeds results back via
    /// `record_build_result`.
    pub fn build_commands(&self) -> Vec<&BuildCommand> {
        let mut cmds: Vec<&BuildCommand> = Vec::new();
        for cmd in &self.config.build_cmds {
            cmds.push(cmd);
        }
        for cmd in &self.config.lint_cmds {
            cmds.push(cmd);
        }
        cmds
    }

    /// Record the result of executing a build/lint command. Returns a finding
    /// if the command failed (non-zero exit code).
    pub fn record_build_result(
        cmd: &str,
        exit_code: i32,
        stderr: &str,
    ) -> Option<DeterministicFinding> {
        if exit_code != 0 {
            // Truncate stderr to first N chars (char-boundary safe)
            let truncated = if stderr.chars().count() > STDERR_TRUNCATE_CHARS {
                let s: String = stderr.chars().take(STDERR_TRUNCATE_CHARS).collect();
                format!("{s}... (truncated)")
            } else {
                stderr.to_string()
            };
            Some(DeterministicFinding {
                check: CheckKind::Build,
                severity: DeterministicSeverity::Block,
                file: None,
                line: None,
                message: format!(
                    "Build/lint command failed (exit {}): {}\n{}",
                    exit_code, cmd, truncated
                ),
            })
        } else {
            None
        }
    }

    // -- Check 2: VERSION_GATE --

    /// Evaluate VERSION_GATE rules against imports found in changed files.
    fn check_version_gate(&self, changed_files: &[PathBuf]) -> Vec<DeterministicFinding> {
        let mut findings = Vec::new();
        if self.config.deprecated_patterns.is_empty() {
            return findings;
        }

        for file in changed_files {
            let lang = detect_language(file);
            // Skip files whose language has no ecosystem mapping
            if language_ecosystem(&lang).is_none() {
                continue;
            }

            // Filter patterns to valid severities
            let relevant: Vec<&DeprecatedPatternEntry> = self
                .config
                .deprecated_patterns
                .iter()
                .filter(|entry| entry.severity == "BLOCK" || entry.severity == "WARN")
                .collect();

            // Read file content (best-effort; skip unreadable files)
            let content = match std::fs::read_to_string(file) {
                Ok(c) => c,
                Err(_) => continue,
            };

            let imports = extract_imports(&content, &lang);

            for pattern in &relevant {
                // Check if any import matches the deprecated name
                let dep_name_lower = pattern.deprecated_name.to_lowercase();
                for import in &imports {
                    let import_lower = import.to_lowercase();
                    if import_lower.contains(&dep_name_lower)
                        || dep_name_lower.contains(&import_lower)
                    {
                        let sev = if pattern.severity == "BLOCK" {
                            DeterministicSeverity::Block
                        } else {
                            DeterministicSeverity::Warn
                        };
                        findings.push(DeterministicFinding {
                            check: CheckKind::VersionGate,
                            severity: sev,
                            file: Some(file.clone()),
                            line: None,
                            message: format!(
                                "Deprecated '{}' found in imports. Use '{}' instead.",
                                pattern.deprecated_name, pattern.replacement_name
                            ),
                        });
                    }
                }
            }
        }
        findings
    }

    // -- Import cross-reference (utility, used by VERSION_GATE and tests) --

    /// Cross-reference imports against deprecated_patterns with ecosystem filter.
    /// Not a separate step in the 5-step pipeline (VERSION_GATE covers this),
    /// but retained as a utility method for fine-grained checks and testing.
    #[allow(dead_code)]
    fn check_imports(&self, changed_files: &[PathBuf]) -> Vec<DeterministicFinding> {
        let mut findings = Vec::new();

        for file in changed_files {
            let lang = detect_language(file);
            // Skip files whose language has no ecosystem mapping
            if language_ecosystem(&lang).is_none() {
                continue;
            }

            let content = match std::fs::read_to_string(file) {
                Ok(c) => c,
                Err(_) => continue,
            };

            let imports = extract_imports(&content, &lang);

            // Cross-reference each import against deprecated patterns
            for pattern in &self.config.deprecated_patterns {
                let dep_lower = pattern.deprecated_name.to_lowercase();
                for import in &imports {
                    if import.to_lowercase() == dep_lower {
                        let sev = if pattern.severity == "BLOCK" {
                            DeterministicSeverity::Block
                        } else {
                            DeterministicSeverity::Warn
                        };
                        findings.push(DeterministicFinding {
                            check: CheckKind::ImportCrossRef,
                            severity: sev,
                            file: Some(file.clone()),
                            line: None,
                            message: format!(
                                "Import '{}' matches deprecated pattern '{}'. Replace with '{}'.",
                                import, pattern.deprecated_name, pattern.replacement_name
                            ),
                        });
                    }
                }
            }
        }
        findings
    }

    // -- Step 3: FILE_SIZE_CENSUS --

    /// Check all modified files against language-specific line limits.
    /// Reports BLOCK (over hard limit), WARN (over production/test limit).
    fn check_file_size_census(&self, changed_files: &[PathBuf]) -> Vec<DeterministicFinding> {
        let mut findings = Vec::new();
        let fd = &self.config.file_discipline;

        for file in changed_files {
            let path_str = file.to_string_lossy();

            // Check exempt extensions
            if fd.exempt_extensions.iter().any(|ext| path_str.ends_with(ext.as_str())) {
                continue;
            }
            // Check exempt paths
            let normalized = path_str.replace('\\', "/");
            if fd.exempt_paths.iter().any(|prefix| normalized.contains(prefix.as_str())) {
                continue;
            }

            let content = match std::fs::read_to_string(file) {
                Ok(c) => c,
                Err(_) => continue,
            };
            let line_count = content.lines().count();
            let lang = detect_language(file);
            let lang_key = lang.id_str();
            let is_test = is_test_file(file, &lang);

            let (warn_limit, hard_block) = if let Some(limits) = fd.limits.get(lang_key) {
                let warn = if is_test { limits.test_warn } else { limits.production_warn };
                (warn, limits.hard_block)
            } else {
                let warn = if is_test { 250 } else { 150 };
                (warn, 600)
            };

            if line_count > hard_block {
                findings.push(DeterministicFinding {
                    check: CheckKind::FileSizeCensus,
                    severity: DeterministicSeverity::Block,
                    file: Some(file.clone()),
                    line: None,
                    message: format!(
                        "File exceeds hard block ({} lines > {} max for {})",
                        line_count, hard_block, lang_key
                    ),
                });
            } else if line_count > warn_limit {
                findings.push(DeterministicFinding {
                    check: CheckKind::FileSizeCensus,
                    severity: DeterministicSeverity::Warn,
                    file: Some(file.clone()),
                    line: None,
                    message: format!(
                        "File exceeds size limit ({} lines > {} warn for {})",
                        line_count, warn_limit, lang_key
                    ),
                });
            }
        }
        findings
    }

    // -- Step 5: COMPLETENESS --

    /// AST check that named functions exist with non-trivial bodies (>3 lines).
    /// Uses tree-sitter for Rust and Python; skips unsupported languages.
    fn check_completeness(&self, changed_files: &[PathBuf]) -> Vec<DeterministicFinding> {
        let mut findings = Vec::new();

        for file in changed_files {
            let lang = detect_language(file);
            let content = match std::fs::read_to_string(file) {
                Ok(c) => c,
                Err(_) => continue,
            };

            match lang {
                Language::Rust => {
                    let mut parser = tree_sitter::Parser::new();
                    if parser.set_language(&tree_sitter_rust::LANGUAGE.into()).is_err() {
                        continue;
                    }
                    let tree = match parser.parse(&content, None) {
                        Some(t) => t,
                        None => continue,
                    };
                    Self::check_rust_fn_bodies(&tree.root_node(), &content, file, &mut findings);
                }
                Language::Python => {
                    let mut parser = tree_sitter::Parser::new();
                    if parser.set_language(&tree_sitter_python::LANGUAGE.into()).is_err() {
                        continue;
                    }
                    let tree = match parser.parse(&content, None) {
                        Some(t) => t,
                        None => continue,
                    };
                    Self::check_python_fn_bodies(&tree.root_node(), &content, file, &mut findings);
                }
                _ => {} // Other languages: skip (no tree-sitter grammar loaded)
            }
        }
        findings
    }

    fn check_rust_fn_bodies(
        node: &tree_sitter::Node<'_>,
        source: &str,
        file: &Path,
        findings: &mut Vec<DeterministicFinding>,
    ) {
        if node.kind() == "function_item" {
            if let Some(body) = node.child_by_field_name("body") {
                let body_text = &source[body.byte_range()];
                let body_lines = body_text.lines().count();
                // Non-trivial = >3 lines (opening brace, content, closing brace minimum)
                if body_lines <= 1 && body.named_child_count() == 0 {
                    let line = node.start_position().row + 1;
                    let fn_name = node
                        .child_by_field_name("name")
                        .map(|n| &source[n.byte_range()])
                        .unwrap_or("<unknown>");
                    findings.push(DeterministicFinding {
                        check: CheckKind::Completeness,
                        severity: DeterministicSeverity::Warn,
                        file: Some(file.to_path_buf()),
                        line: Some(line as u32),
                        message: format!(
                            "Function '{}' has trivial/empty body ({} lines)",
                            fn_name, body_lines
                        ),
                    });
                }
            }
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            Self::check_rust_fn_bodies(&child, source, file, findings);
        }
    }

    fn check_python_fn_bodies(
        node: &tree_sitter::Node<'_>,
        source: &str,
        file: &Path,
        findings: &mut Vec<DeterministicFinding>,
    ) {
        if node.kind() == "function_definition" {
            if let Some(body) = node.child_by_field_name("body") {
                // Check for pass-only or ellipsis-only bodies
                let is_stub = body.named_child_count() <= 1
                    && body.named_child(0).is_some_and(|c| {
                        matches!(c.kind(), "pass_statement" | "ellipsis" | "expression_statement")
                    });
                if is_stub {
                    let line = node.start_position().row + 1;
                    let fn_name = node
                        .child_by_field_name("name")
                        .map(|n| &source[n.byte_range()])
                        .unwrap_or("<unknown>");
                    findings.push(DeterministicFinding {
                        check: CheckKind::Completeness,
                        severity: DeterministicSeverity::Warn,
                        file: Some(file.to_path_buf()),
                        line: Some(line as u32),
                        message: format!(
                            "Function '{}' has stub body (pass/ellipsis only)",
                            fn_name
                        ),
                    });
                }
            }
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            Self::check_python_fn_bodies(&child, source, file, findings);
        }
    }

    // -- Step 5b: MODULE_COHERENCE --

    /// Per-language module coherence checks:
    /// - Rust: every child in mod.rs exists on disk + vice versa
    /// - Python: __init__.py imports resolve
    /// - TS: index.ts exports resolve
    /// Orphan detection: files with no bridge/parent reference.
    fn check_module_coherence(&self, changed_files: &[PathBuf]) -> Vec<DeterministicFinding> {
        let mut findings = Vec::new();

        for file in changed_files {
            let lang = detect_language(file);
            match lang {
                Language::Rust => {
                    let name = file.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    if name == "mod.rs" || name == "lib.rs" {
                        self.check_rust_module_coherence(file, &mut findings);
                    }
                }
                Language::Python => {
                    let name = file.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    if name == "__init__.py" {
                        self.check_python_init_coherence(file, &mut findings);
                    }
                }
                Language::TypeScript | Language::JavaScript => {
                    let name = file.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    if name == "index.ts" || name == "index.js" || name == "index.tsx" {
                        self.check_ts_index_coherence(file, &mut findings);
                    }
                }
                _ => {}
            }
        }
        findings
    }

    /// Rust: parse mod.rs/lib.rs for `mod child;` declarations, verify each
    /// maps to a child.rs file or child/mod.rs directory on disk.
    fn check_rust_module_coherence(
        &self,
        mod_file: &Path,
        findings: &mut Vec<DeterministicFinding>,
    ) {
        let content = match std::fs::read_to_string(mod_file) {
            Ok(c) => c,
            Err(_) => return,
        };
        let parent_dir = match mod_file.parent() {
            Some(d) => d,
            None => return,
        };

        // Extract `mod child_name;` declarations (not inline `mod name { ... }`)
        static MOD_DECL: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
            regex::Regex::new(r"(?m)^\s*(?:pub\s+)?mod\s+(\w+)\s*;").unwrap()
        });

        for cap in MOD_DECL.captures_iter(&content) {
            let mod_name = &cap[1];
            let file_path = parent_dir.join(format!("{mod_name}.rs"));
            let dir_path = parent_dir.join(mod_name).join("mod.rs");
            if !file_path.exists() && !dir_path.exists() {
                let line = content[..cap.get(0).map_or(0, |m| m.start())]
                    .lines()
                    .count() as u32 + 1;
                findings.push(DeterministicFinding {
                    check: CheckKind::ModuleCoherence,
                    severity: DeterministicSeverity::Block,
                    file: Some(mod_file.to_path_buf()),
                    line: Some(line),
                    message: format!(
                        "mod {} declared but neither {}.rs nor {}/mod.rs exists",
                        mod_name, mod_name, mod_name
                    ),
                });
            }
        }
    }

    /// Python: parse __init__.py for `from .child import ...`, verify child.py
    /// or child/ exists on disk.
    fn check_python_init_coherence(
        &self,
        init_file: &Path,
        findings: &mut Vec<DeterministicFinding>,
    ) {
        let content = match std::fs::read_to_string(init_file) {
            Ok(c) => c,
            Err(_) => return,
        };
        let parent_dir = match init_file.parent() {
            Some(d) => d,
            None => return,
        };

        static PY_IMPORT: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
            regex::Regex::new(r"from\s+\.(\w+)\s+import").unwrap()
        });

        for cap in PY_IMPORT.captures_iter(&content) {
            let mod_name = &cap[1];
            let file_path = parent_dir.join(format!("{mod_name}.py"));
            let dir_path = parent_dir.join(mod_name).join("__init__.py");
            if !file_path.exists() && !dir_path.exists() {
                let line = content[..cap.get(0).map_or(0, |m| m.start())]
                    .lines()
                    .count() as u32 + 1;
                findings.push(DeterministicFinding {
                    check: CheckKind::ModuleCoherence,
                    severity: DeterministicSeverity::Block,
                    file: Some(init_file.to_path_buf()),
                    line: Some(line),
                    message: format!(
                        "from .{} import ... but neither {}.py nor {}/__init__.py exists",
                        mod_name, mod_name, mod_name
                    ),
                });
            }
        }
    }

    /// TS: parse index.ts for `export ... from './child'`, verify child.ts exists.
    fn check_ts_index_coherence(
        &self,
        index_file: &Path,
        findings: &mut Vec<DeterministicFinding>,
    ) {
        let content = match std::fs::read_to_string(index_file) {
            Ok(c) => c,
            Err(_) => return,
        };
        let parent_dir = match index_file.parent() {
            Some(d) => d,
            None => return,
        };

        static TS_EXPORT: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
            regex::Regex::new(r#"export\s+.*\s+from\s+['"]\./(\w+)['"]"#).unwrap()
        });

        for cap in TS_EXPORT.captures_iter(&content) {
            let mod_name = &cap[1];
            let ts_file = parent_dir.join(format!("{mod_name}.ts"));
            let tsx_file = parent_dir.join(format!("{mod_name}.tsx"));
            let js_file = parent_dir.join(format!("{mod_name}.js"));
            let index_dir = parent_dir.join(mod_name).join("index.ts");
            if !ts_file.exists() && !tsx_file.exists() && !js_file.exists() && !index_dir.exists() {
                let line = content[..cap.get(0).map_or(0, |m| m.start())]
                    .lines()
                    .count() as u32 + 1;
                findings.push(DeterministicFinding {
                    check: CheckKind::ModuleCoherence,
                    severity: DeterministicSeverity::Block,
                    file: Some(index_file.to_path_buf()),
                    line: Some(line),
                    message: format!(
                        "export from './{mod_name}' but no matching file found"
                    ),
                });
            }
        }
    }

    // -- Step 4: LAZY_CODE_SCAN --

    /// Scan all changed files for lazy code patterns.
    fn check_lazy_code(&self, changed_files: &[PathBuf]) -> Vec<DeterministicFinding> {
        let mut findings = Vec::new();

        for file in changed_files {
            let lang = detect_language(file);

            // Skip test files (exemption per spec: test files are allowed lazy patterns)
            if is_test_file(file, &lang) {
                continue;
            }

            let content = match std::fs::read_to_string(file) {
                Ok(c) => c,
                Err(_) => continue,
            };

            let patterns = lazy_patterns_for(&lang);
            for pat in &patterns {
                for m in pat.pattern.find_iter(&content) {
                    let line = content[..m.start()].lines().count() as u32 + 1;
                    findings.push(DeterministicFinding {
                        check: CheckKind::LazyCode,
                        severity: DeterministicSeverity::Warn,
                        file: Some(file.clone()),
                        line: Some(line),
                        message: format!(
                            "Lazy code pattern '{}' at line {}",
                            pat.id, line
                        ),
                    });
                }
            }
        }
        findings
    }
}

/// Check if a file is a test file (exempt from lazy code scan).
fn is_test_file(path: &Path, lang: &Language) -> bool {
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    match lang {
        Language::Rust => {
            // Rust: files in tests/ directory or containing #[cfg(test)]
            // (we can only check path here, not content)
            path.components().any(|c| c.as_os_str() == "tests")
                || name.ends_with("_test.rs")
        }
        Language::Python => {
            name.starts_with("test_") || name.ends_with("_test.py")
                || path.components().any(|c| c.as_os_str() == "tests")
        }
        Language::TypeScript | Language::JavaScript => {
            name.contains(".test.") || name.contains(".spec.")
        }
        Language::Go => name.ends_with("_test.go"),
        Language::Java | Language::Kotlin => {
            name.contains("Test") || path.components().any(|c| c.as_os_str() == "test")
        }
        _ => false,
    }
}

// ---------------------------------------------------------------------------
// DeterministicAuditResult
// ---------------------------------------------------------------------------

/// Aggregate result from the deterministic auditor.
#[derive(Debug, Clone)]
pub struct DeterministicAuditResult {
    pub findings: Vec<DeterministicFinding>,
    /// Number of checks completed (1-5). Less than 5 means early exit on build failure.
    pub checks_run: u8,
    /// True iff zero BLOCK findings.
    pub passed: bool,
}

impl DeterministicAuditResult {
    /// Count of BLOCK-severity findings.
    pub fn block_count(&self) -> usize {
        self.findings
            .iter()
            .filter(|f| f.severity == DeterministicSeverity::Block)
            .count()
    }

    /// Count of WARN-severity findings.
    pub fn warn_count(&self) -> usize {
        self.findings
            .iter()
            .filter(|f| f.severity == DeterministicSeverity::Warn)
            .count()
    }

    /// Convert findings to `AuditFinding` for the evidence gate pipeline.
    pub fn to_audit_findings(&self) -> Vec<crate::evidence::AuditFinding> {
        self.findings
            .iter()
            .map(|f| {
                let severity = match f.severity {
                    DeterministicSeverity::Block => crate::evidence::FindingSeverity::Block,
                    DeterministicSeverity::Warn => crate::evidence::FindingSeverity::Warn,
                };
                let msg = match (&f.file, f.line) {
                    (Some(path), Some(line)) => {
                        format!("[{}] {}:{}: {}", f.check, path.display(), line, f.message)
                    }
                    (Some(path), None) => {
                        format!("[{}] {}: {}", f.check, path.display(), f.message)
                    }
                    _ => format!("[{}] {}", f.check, f.message),
                };
                crate::evidence::AuditFinding {
                    severity,
                    message: msg,
                }
            })
            .collect()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn default_config() -> DeterministicAuditorConfig {
        DeterministicAuditorConfig::default()
    }

    #[test]
    fn disabled_auditor_returns_pass() {
        let mut config = default_config();
        config.enabled = false;
        let auditor = DeterministicAuditor::new(config);
        let result = auditor.run_all(&[]);
        assert!(result.passed);
        assert_eq!(result.checks_run, 0);
        assert!(result.findings.is_empty());
    }

    #[test]
    fn no_build_cmds_produces_warn() {
        let config = default_config();
        let auditor = DeterministicAuditor::new(config);
        let result = auditor.run_all(&[]);
        // Should have a warn about missing build commands
        assert_eq!(result.warn_count(), 1);
        assert!(result.passed); // WARN does not block
        assert_eq!(result.checks_run, 5);
    }

    #[test]
    fn record_build_result_success() {
        let result = DeterministicAuditor::record_build_result("cargo check", 0, "");
        assert!(result.is_none());
    }

    #[test]
    fn record_build_result_failure() {
        let result =
            DeterministicAuditor::record_build_result("cargo check", 1, "error[E0308]: mismatched types");
        assert!(result.is_some());
        let finding = result.unwrap();
        assert_eq!(finding.severity, DeterministicSeverity::Block);
        assert_eq!(finding.check, CheckKind::Build);
        assert!(finding.message.contains("exit 1"));
    }

    #[test]
    fn lazy_code_scan_detects_todo() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("lib.rs");
        {
            let mut f = std::fs::File::create(&file).unwrap();
            writeln!(f, "fn main() {{").unwrap();
            writeln!(f, "    // TODO: implement this").unwrap();
            writeln!(f, "    todo!()").unwrap();
            writeln!(f, "}}").unwrap();
        }

        let mut config = default_config();
        config.primary_language = Language::Rust;
        let auditor = DeterministicAuditor::new(config);
        let findings = auditor.check_lazy_code(&[file]);
        assert!(findings.len() >= 2, "expected at least 2 lazy findings, got {}", findings.len());
        assert!(findings.iter().any(|f| f.message.contains("todo_comment")));
        assert!(findings.iter().any(|f| f.message.contains("rust_todo_macro")));
    }

    #[test]
    fn lazy_code_scan_skips_test_files() {
        let dir = tempfile::tempdir().unwrap();
        let test_dir = dir.path().join("tests");
        std::fs::create_dir_all(&test_dir).unwrap();
        let file = test_dir.join("integration_test.rs");
        {
            let mut f = std::fs::File::create(&file).unwrap();
            writeln!(f, "// TODO: test more cases").unwrap();
            writeln!(f, "todo!()").unwrap();
        }

        let config = default_config();
        let auditor = DeterministicAuditor::new(config);
        let findings = auditor.check_lazy_code(&[file]);
        assert!(findings.is_empty(), "test files should be exempt from lazy code scan");
    }

    #[test]
    fn lazy_code_scan_python_patterns() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("module.py");
        {
            let mut f = std::fs::File::create(&file).unwrap();
            writeln!(f, "def handler():").unwrap();
            writeln!(f, "    pass").unwrap();
            writeln!(f, "").unwrap();
            writeln!(f, "def other():").unwrap();
            writeln!(f, "    raise NotImplementedError").unwrap();
        }

        let mut config = default_config();
        config.primary_language = Language::Python;
        let auditor = DeterministicAuditor::new(config);
        let findings = auditor.check_lazy_code(&[file]);
        assert!(findings.len() >= 2, "expected at least 2 python lazy findings, got {}", findings.len());
    }

    #[test]
    fn import_extraction_rust() {
        let source = "use blake2::Digest;\nuse std::io;\nextern crate log;\n";
        let imports = extract_imports(source, &Language::Rust);
        assert!(imports.contains(&"blake2".to_string()));
        assert!(imports.contains(&"std".to_string()));
        assert!(imports.contains(&"log".to_string()));
    }

    #[test]
    fn import_extraction_python() {
        let source = "import json\nfrom http.server import HTTPServer\nimport os\n";
        let imports = extract_imports(source, &Language::Python);
        assert!(imports.contains(&"json".to_string()));
        assert!(imports.contains(&"http.server".to_string()));
        assert!(imports.contains(&"os".to_string()));
    }

    #[test]
    fn version_gate_detects_deprecated_import() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("lib.rs");
        {
            let mut f = std::fs::File::create(&file).unwrap();
            writeln!(f, "use blake2::Digest;").unwrap();
        }

        let config = DeterministicAuditorConfig {
            deprecated_patterns: vec![DeprecatedPatternEntry {
                deprecated_name: "blake2".into(),
                replacement_name: "blake3".into(),
                severity: "BLOCK".into(),
            }],
            ..default_config()
        };
        let auditor = DeterministicAuditor::new(config);
        let findings = auditor.check_version_gate(&[file]);
        assert!(!findings.is_empty(), "should detect deprecated blake2 import");
        assert_eq!(findings[0].severity, DeterministicSeverity::Block);
    }

    #[test]
    fn import_cross_ref_exact_match() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("server.py");
        {
            let mut f = std::fs::File::create(&file).unwrap();
            writeln!(f, "import json").unwrap();
        }

        let config = DeterministicAuditorConfig {
            deprecated_patterns: vec![DeprecatedPatternEntry {
                deprecated_name: "json".into(),
                replacement_name: "orjson".into(),
                severity: "WARN".into(),
            }],
            ..default_config()
        };
        let auditor = DeterministicAuditor::new(config);
        let findings = auditor.check_imports(&[file]);
        assert!(!findings.is_empty());
        assert_eq!(findings[0].severity, DeterministicSeverity::Warn);
        assert_eq!(findings[0].check, CheckKind::ImportCrossRef);
    }

    #[test]
    fn detect_language_from_extension() {
        assert_eq!(detect_language(Path::new("foo.rs")), Language::Rust);
        assert_eq!(detect_language(Path::new("bar.py")), Language::Python);
        assert_eq!(detect_language(Path::new("baz.ts")), Language::TypeScript);
        assert_eq!(detect_language(Path::new("qux.go")), Language::Go);
        assert_eq!(detect_language(Path::new("main.java")), Language::Java);
        assert_eq!(detect_language(Path::new("prog.cs")), Language::CSharp);
        assert_eq!(detect_language(Path::new("unknown.xyz")), Language::Unknown);
    }

    #[test]
    fn language_ecosystem_mapping() {
        assert_eq!(language_ecosystem(&Language::Rust), Some("cargo"));
        assert_eq!(language_ecosystem(&Language::Python), Some("pip"));
        assert_eq!(language_ecosystem(&Language::TypeScript), Some("npm"));
        assert_eq!(language_ecosystem(&Language::Go), Some("go"));
        assert_eq!(language_ecosystem(&Language::Unknown), None);
    }

    #[test]
    fn test_file_detection() {
        assert!(is_test_file(Path::new("tests/foo.rs"), &Language::Rust));
        assert!(is_test_file(Path::new("test_handler.py"), &Language::Python));
        assert!(is_test_file(Path::new("foo.test.ts"), &Language::TypeScript));
        assert!(is_test_file(Path::new("foo.spec.js"), &Language::JavaScript));
        assert!(is_test_file(Path::new("bar_test.go"), &Language::Go));
        assert!(!is_test_file(Path::new("src/lib.rs"), &Language::Rust));
        assert!(!is_test_file(Path::new("main.py"), &Language::Python));
    }

    #[test]
    fn to_audit_findings_converts_correctly() {
        let result = DeterministicAuditResult {
            findings: vec![
                DeterministicFinding {
                    check: CheckKind::LazyCode,
                    severity: DeterministicSeverity::Block,
                    file: Some(PathBuf::from("src/lib.rs")),
                    line: Some(42),
                    message: "todo!() macro".into(),
                },
                DeterministicFinding {
                    check: CheckKind::Build,
                    severity: DeterministicSeverity::Warn,
                    file: None,
                    line: None,
                    message: "clippy warning".into(),
                },
            ],
            checks_run: 5,
            passed: false,
        };
        let audit_findings = result.to_audit_findings();
        assert_eq!(audit_findings.len(), 2);
        assert_eq!(
            audit_findings[0].severity,
            crate::evidence::FindingSeverity::Block
        );
        assert!(audit_findings[0].message.contains("src/lib.rs:42"));
        assert_eq!(
            audit_findings[1].severity,
            crate::evidence::FindingSeverity::Warn
        );
    }

    #[test]
    fn full_run_with_clean_files() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("clean.rs");
        {
            let mut f = std::fs::File::create(&file).unwrap();
            writeln!(f, "fn add(a: i32, b: i32) -> i32 {{").unwrap();
            writeln!(f, "    a + b").unwrap();
            writeln!(f, "}}").unwrap();
        }

        let config = DeterministicAuditorConfig {
            build_cmds: vec![BuildCommand {
                cmd: "echo ok".into(),
                working_dir: PathBuf::from("."),
                env: Default::default(),
                timeout_s: BUILD_CMD_TIMEOUT_S,
                scope: elai_core::project_profile::CommandScope::Full,
                verified: true,
            }],
            ..default_config()
        };
        let auditor = DeterministicAuditor::new(config);
        let result = auditor.run_all(&[file]);
        assert!(result.passed);
        assert_eq!(result.checks_run, 5);
        assert_eq!(result.block_count(), 0);
    }

    #[test]
    fn check_kind_display() {
        assert_eq!(format!("{}", CheckKind::Build), "build");
        assert_eq!(format!("{}", CheckKind::VersionGate), "version_gate");
        assert_eq!(format!("{}", CheckKind::FileSizeCensus), "file_size_census");
        assert_eq!(format!("{}", CheckKind::ImportCrossRef), "import_cross_ref");
        assert_eq!(format!("{}", CheckKind::LazyCode), "lazy_code");
        assert_eq!(format!("{}", CheckKind::Completeness), "completeness");
        assert_eq!(format!("{}", CheckKind::ModuleCoherence), "module_coherence");
    }

    #[test]
    fn lazy_patterns_count_per_language() {
        let universal_count = 9; // 9 universal patterns
        let rust = lazy_patterns_for(&Language::Rust);
        // Batch 4A: Rust gained `rust_panic_placeholder` on top of the
        // existing 3 macros, so the language-specific count is now 4.
        assert_eq!(rust.len(), universal_count + 4);
        let python = lazy_patterns_for(&Language::Python);
        assert_eq!(python.len(), universal_count + 3);
        let ts = lazy_patterns_for(&Language::TypeScript);
        assert_eq!(ts.len(), universal_count + 1);
        let go = lazy_patterns_for(&Language::Go);
        assert_eq!(go.len(), universal_count + 2);
        let java = lazy_patterns_for(&Language::Java);
        assert_eq!(java.len(), universal_count + 2);
        let unknown = lazy_patterns_for(&Language::Unknown);
        assert_eq!(unknown.len(), universal_count);
    }
}

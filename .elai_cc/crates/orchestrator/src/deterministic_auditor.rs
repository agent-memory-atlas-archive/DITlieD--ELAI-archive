//! Deterministic-first auditor: runs 4 deterministic checks before the LLM
//! auditor agent. Language-parameterized via `ProjectProfile`.
//!
//! Spec: anti-staleness-enforcement.txt §3.3, code-completeness-enforcement-design.txt Layer 4.
//! Step: A-12. Prerequisites: A-6 (VERSION_GATE), A-4 (ProjectProfile).
//!
//! Checks (in order):
//!   1. Build check — runs `ProjectProfile.commands.build` + lint commands
//!   2. VERSION_GATE — evaluates deprecated_patterns against changed files
//!   3. Import cross-reference — language-specific regex import extraction
//!   4. Lazy code scan — regex patterns per language on all modified files
//!
//! Only after all checks pass (zero BLOCKs) does the LLM auditor run.

use std::path::{Path, PathBuf};

use elai_core::project_profile::{BuildCommand, Language, ProjectProfile};
use elai_rules::schema::DeprecatedPatternEntry;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Maximum time (seconds) for a single build/lint command.
pub const BUILD_CMD_TIMEOUT_S: u32 = 120;

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
// DeterministicFinding: a single finding from any of the 4 checks
// ---------------------------------------------------------------------------

/// A finding produced by one of the 4 deterministic checks.
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
    ImportCrossRef,
    LazyCode,
}

impl std::fmt::Display for CheckKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Build => write!(f, "build"),
            Self::VersionGate => write!(f, "version_gate"),
            Self::ImportCrossRef => write!(f, "import_cross_ref"),
            Self::LazyCode => write!(f, "lazy_code"),
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
            let rust_patterns = [
                ("rust_todo_macro", r"\btodo!\("),
                ("rust_unimplemented_macro", r"\bunimplemented!\("),
                ("rust_empty_fn", r"fn\s+\w+\([^)]*\)[^{]*\{\s*\}"),
            ];
            for (id, pat) in &rust_patterns {
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
// DeterministicAuditor: the 4-check pipeline
// ---------------------------------------------------------------------------

/// Runs the 4 deterministic checks on a set of changed files.
/// Returns all findings. If any finding has `severity == Block`, the LLM
/// auditor should NOT run until the blocks are resolved.
pub struct DeterministicAuditor {
    config: DeterministicAuditorConfig,
}

impl DeterministicAuditor {
    pub fn new(config: DeterministicAuditorConfig) -> Self {
        Self { config }
    }

    /// Run all 4 checks in order. Short-circuits on build failure (check 1)
    /// since subsequent checks may be unreliable if the code doesn't compile.
    pub fn run_all(&self, changed_files: &[PathBuf]) -> DeterministicAuditResult {
        if !self.config.enabled {
            return DeterministicAuditResult {
                findings: Vec::new(),
                checks_run: 0,
                passed: true,
            };
        }

        let mut findings = Vec::new();

        // Check 1: Build + lint (produces build commands but doesn't execute
        // them — the caller dispatches via Bash. We generate the command list.)
        let build_findings = self.check_build();
        let build_has_blocks = build_findings
            .iter()
            .any(|f| f.severity == DeterministicSeverity::Block);
        findings.extend(build_findings);

        // If build fails with BLOCKs, skip remaining checks (unreliable on
        // non-compiling code). Return early with checks_run=1.
        if build_has_blocks {
            return DeterministicAuditResult {
                findings,
                checks_run: 1,
                passed: false,
            };
        }

        // Check 2: VERSION_GATE evaluation on changed files
        findings.extend(self.check_version_gate(changed_files));

        // Check 3: Import cross-reference against deprecated_patterns
        findings.extend(self.check_imports(changed_files));

        // Check 4: Lazy code scan on all changed files
        findings.extend(self.check_lazy_code(changed_files));

        let passed = !findings
            .iter()
            .any(|f| f.severity == DeterministicSeverity::Block);
        DeterministicAuditResult {
            findings,
            checks_run: 4,
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
            // Truncate stderr to first 500 chars for finding message
            let truncated = if stderr.len() > 500 {
                format!("{}... (truncated)", &stderr[..500])
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

    // -- Check 3: Import cross-reference --

    /// Cross-reference imports against deprecated_patterns with ecosystem filter.
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

    // -- Check 4: Lazy code scan --

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
    /// Number of checks completed (1-4). Less than 4 means early exit on build failure.
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
        assert_eq!(result.checks_run, 4);
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
            checks_run: 4,
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
        assert_eq!(result.checks_run, 4);
        assert_eq!(result.block_count(), 0);
    }

    #[test]
    fn check_kind_display() {
        assert_eq!(format!("{}", CheckKind::Build), "build");
        assert_eq!(format!("{}", CheckKind::VersionGate), "version_gate");
        assert_eq!(format!("{}", CheckKind::ImportCrossRef), "import_cross_ref");
        assert_eq!(format!("{}", CheckKind::LazyCode), "lazy_code");
    }

    #[test]
    fn lazy_patterns_count_per_language() {
        let universal_count = 9; // 9 universal patterns
        let rust = lazy_patterns_for(&Language::Rust);
        assert_eq!(rust.len(), universal_count + 3);
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

//! Bash command safety filter — stage in the tool validation pipeline (A-27).
//!
//! Checks commands against a configurable blocklist before execution.
//! Severity::Block rejects outright; Severity::WarnAndConfirm rejects in
//! autonomous mode and would prompt in interactive mode (currently always
//! autonomous, so both reject).

use std::fmt;
use std::sync::OnceLock;

use regex::Regex;

// ---------------------------------------------------------------------------
// Severity
// ---------------------------------------------------------------------------

/// How the safety filter handles a matched pattern.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    /// Hard reject — command is never executed.
    Block,
    /// Soft reject — would prompt in interactive mode; rejects in autonomous.
    WarnAndConfirm,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Block => f.write_str("BLOCK"),
            Self::WarnAndConfirm => f.write_str("WARN"),
        }
    }
}

// ---------------------------------------------------------------------------
// Violation
// ---------------------------------------------------------------------------

/// A single safety violation found in a command.
#[derive(Debug, Clone)]
pub struct SafetyViolation {
    pub severity: Severity,
    pub message: String,
    pub alternative: String,
    pub matched_text: String,
}

impl fmt::Display for SafetyViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}] {}: {} — {}",
            self.severity, self.matched_text, self.message, self.alternative
        )
    }
}

// ---------------------------------------------------------------------------
// Pattern definition (static data)
// ---------------------------------------------------------------------------

struct PatternDef {
    regex: &'static str,
    severity: Severity,
    message: &'static str,
    alternative: &'static str,
    /// If the command contains this substring, skip the match (simple exclusion).
    exclude: Option<&'static str>,
}

/// Default blocklist patterns derived from Claude Code system prompt safety rules.
const DEFAULT_PATTERNS: &[PatternDef] = &[
    // ── BLOCK severity ──────────────────────────────────────────────────
    PatternDef {
        regex: r"git\s+push\s+.*(-f\b|--force\b)",
        severity: Severity::Block,
        message: "force push can overwrite remote history",
        alternative: "use --force-with-lease instead",
        exclude: Some("--force-with-lease"),
    },
    PatternDef {
        regex: r"git\s+reset\s+--hard\b",
        severity: Severity::Block,
        message: "hard reset discards all uncommitted changes",
        alternative: "create a backup branch first, or use git stash",
        exclude: None,
    },
    PatternDef {
        regex: r"git\s+checkout\s+--\s",
        severity: Severity::Block,
        message: "discards unstaged changes to files irreversibly",
        alternative: "confirm the files first, or use git stash",
        exclude: None,
    },
    PatternDef {
        regex: r"git\s+restore\s+--staged\b",
        severity: Severity::Block,
        message: "unstages changes which may be hard to recover",
        alternative: "confirm the files first",
        exclude: None,
    },
    PatternDef {
        regex: r"rm\s+.*-r.*f|rm\s+.*-f.*r|rm\s+-rf\b",
        severity: Severity::Block,
        message: "recursive force-remove can destroy data permanently",
        alternative: "use trash-put or confirm the path first",
        exclude: None,
    },
    PatternDef {
        regex: r"git\s+rebase\s+-i\b",
        severity: Severity::Block,
        message: "interactive rebase requires terminal interaction",
        alternative: "interactive mode is not supported in this environment",
        exclude: None,
    },
    PatternDef {
        regex: r"git\s+add\s+-i\b",
        severity: Severity::Block,
        message: "interactive add requires terminal interaction",
        alternative: "interactive mode is not supported in this environment",
        exclude: None,
    },
    PatternDef {
        regex: r"--no-verify\b",
        severity: Severity::Block,
        message: "skipping hooks bypasses safety checks",
        alternative: "investigate and fix the hook failure instead",
        exclude: None,
    },
    PatternDef {
        regex: r"--no-gpg-sign\b",
        severity: Severity::Block,
        message: "skipping GPG signing bypasses commit verification",
        alternative: "investigate and fix the signing issue instead",
        exclude: None,
    },
    // ── WARN_AND_CONFIRM severity ───────────────────────────────────────
    PatternDef {
        regex: r"git\s+branch\s+-D\b",
        severity: Severity::WarnAndConfirm,
        message: "force-deletes branch even if not fully merged",
        alternative: "use -d for safe delete (fails if unmerged)",
        exclude: None,
    },
    PatternDef {
        regex: r"git\s+clean\s+.*-[a-zA-Z]*f",
        severity: Severity::WarnAndConfirm,
        message: "removes untracked files permanently",
        alternative: "use git clean -n (dry run) first",
        exclude: None,
    },
    PatternDef {
        regex: r"git\s+push\s+.*\b(main|master)\b",
        severity: Severity::WarnAndConfirm,
        message: "pushing directly to main/master branch",
        alternative: "consider using a feature branch and pull request",
        exclude: None,
    },
];

// ---------------------------------------------------------------------------
// Compiled pattern
// ---------------------------------------------------------------------------

struct CompiledPattern {
    regex: Regex,
    severity: Severity,
    message: &'static str,
    alternative: &'static str,
    exclude: Option<&'static str>,
}

// ---------------------------------------------------------------------------
// SafetyFilter
// ---------------------------------------------------------------------------

/// Bash command safety filter. Thread-safe, immutable after construction.
pub struct SafetyFilter {
    patterns: Vec<CompiledPattern>,
}

impl SafetyFilter {
    /// Build from the default pattern set. Returns `Err` only if a built-in
    /// regex is malformed (indicates a code bug, not a runtime error).
    fn from_defaults() -> Result<Self, regex::Error> {
        let patterns = DEFAULT_PATTERNS
            .iter()
            .map(|def| {
                Ok(CompiledPattern {
                    regex: Regex::new(def.regex)?,
                    severity: def.severity,
                    message: def.message,
                    alternative: def.alternative,
                    exclude: def.exclude,
                })
            })
            .collect::<Result<Vec<_>, regex::Error>>()?;
        Ok(Self { patterns })
    }

    /// Check a shell command string against all safety patterns.
    /// Returns an empty vec if no violations found.
    pub fn check(&self, command: &str) -> Vec<SafetyViolation> {
        let normalized = command
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");

        self.patterns
            .iter()
            .filter_map(|pat| {
                if let Some(excl) = pat.exclude {
                    if normalized.contains(excl) {
                        return None;
                    }
                }
                pat.regex.find(&normalized).map(|m| SafetyViolation {
                    severity: pat.severity,
                    message: pat.message.to_string(),
                    alternative: pat.alternative.to_string(),
                    matched_text: m.as_str().to_string(),
                })
            })
            .collect()
    }

    /// Returns the worst severity among violations, if any.
    pub fn worst_severity(violations: &[SafetyViolation]) -> Option<Severity> {
        violations.iter().fold(None, |worst, v| match worst {
            None => Some(v.severity),
            Some(Severity::Block) => Some(Severity::Block),
            Some(Severity::WarnAndConfirm) => Some(v.severity),
        })
    }
}

// ---------------------------------------------------------------------------
// Global singleton
// ---------------------------------------------------------------------------

/// Access the global safety filter (compiled once, reused).
pub fn global_safety_filter() -> &'static SafetyFilter {
    static FILTER: OnceLock<SafetyFilter> = OnceLock::new();
    FILTER.get_or_init(|| {
        SafetyFilter::from_defaults()
            .expect("built-in safety regex patterns must compile — this is a code bug")
    })
}

// ---------------------------------------------------------------------------
// Convenience: format violations into a user-facing error string
// ---------------------------------------------------------------------------

/// Format violations into a structured error message suitable for tool output.
pub fn format_violations(violations: &[SafetyViolation]) -> String {
    use std::fmt::Write;
    let mut out = String::from("Safety filter blocked this command:\n");
    for v in violations {
        let _ = writeln!(
            out,
            "  [{severity}] {message}\n    Matched: {matched}\n    Alternative: {alt}",
            severity = v.severity,
            message = v.message,
            matched = v.matched_text,
            alt = v.alternative,
        );
    }
    out
}

// ---------------------------------------------------------------------------
// Runtime config (set once at startup from elai_cc.toml)
// ---------------------------------------------------------------------------

/// Initialize the safety filter enabled flag from `elai_cc.toml` at startup.
///
/// Call this once before any tool invocations. Safe to call multiple times —
/// only the first call takes effect (OnceLock semantics).
pub fn init_safety_config(enabled: bool) {
    let _ = RUNTIME_SAFETY_ENABLED.set(enabled);
}

static RUNTIME_SAFETY_ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();

fn safety_enabled() -> bool {
    // Default: enabled
    *RUNTIME_SAFETY_ENABLED.get_or_init(|| true)
}

// ---------------------------------------------------------------------------
// Integration point: call from run_bash()
// ---------------------------------------------------------------------------

/// Check a bash command for safety violations. Returns `Err(message)` if the
/// command should be blocked, `Ok(())` if it may proceed.
///
/// When safety is disabled via `elai_cc.toml [tools.safety_filter] enabled = false`,
/// this function always returns `Ok(())`.
///
/// In autonomous mode (our default), both Block and WarnAndConfirm reject.
/// A future interactive mode could return a confirmation prompt for Warn.
pub fn check_bash_safety(command: &str) -> Result<(), String> {
    if !safety_enabled() {
        return Ok(());
    }
    let filter = global_safety_filter();
    let violations = filter.check(command);
    if violations.is_empty() {
        return Ok(());
    }
    Err(format_violations(&violations))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn filter() -> &'static SafetyFilter {
        global_safety_filter()
    }

    // ── BLOCK patterns ──────────────────────────────────────────────────

    #[test]
    fn blocks_force_push() {
        let v = filter().check("git push --force origin main");
        assert!(!v.is_empty());
        assert_eq!(v[0].severity, Severity::Block);
    }

    #[test]
    fn blocks_force_push_short_flag() {
        let v = filter().check("git push -f origin main");
        assert!(!v.is_empty());
        assert_eq!(v[0].severity, Severity::Block);
    }

    #[test]
    fn allows_force_with_lease() {
        let v = filter().check("git push --force-with-lease origin feature");
        // Should NOT match the force push pattern
        let force_blocks: Vec<_> = v
            .iter()
            .filter(|v| v.message.contains("force push"))
            .collect();
        assert!(force_blocks.is_empty());
    }

    #[test]
    fn blocks_hard_reset() {
        let v = filter().check("git reset --hard HEAD~3");
        assert!(!v.is_empty());
        assert_eq!(v[0].severity, Severity::Block);
    }

    #[test]
    fn allows_soft_reset() {
        let v = filter().check("git reset --soft HEAD~1");
        let blocks: Vec<_> = v
            .iter()
            .filter(|v| v.message.contains("hard reset"))
            .collect();
        assert!(blocks.is_empty());
    }

    #[test]
    fn blocks_checkout_dash_dash() {
        let v = filter().check("git checkout -- src/main.rs");
        assert!(!v.is_empty());
        assert_eq!(v[0].severity, Severity::Block);
    }

    #[test]
    fn blocks_restore_staged() {
        let v = filter().check("git restore --staged .");
        assert!(!v.is_empty());
        assert_eq!(v[0].severity, Severity::Block);
    }

    #[test]
    fn blocks_rm_rf() {
        let v = filter().check("rm -rf /tmp/build");
        assert!(!v.is_empty());
        assert_eq!(v[0].severity, Severity::Block);
    }

    #[test]
    fn blocks_rm_dash_r_dash_f() {
        // handles split flags: rm -r -f
        let v = filter().check("rm -r -f ./output");
        assert!(!v.is_empty());
        assert_eq!(v[0].severity, Severity::Block);
    }

    #[test]
    fn blocks_interactive_rebase() {
        let v = filter().check("git rebase -i HEAD~5");
        assert!(!v.is_empty());
        assert_eq!(v[0].severity, Severity::Block);
    }

    #[test]
    fn blocks_interactive_add() {
        let v = filter().check("git add -i");
        assert!(!v.is_empty());
        assert_eq!(v[0].severity, Severity::Block);
    }

    #[test]
    fn blocks_no_verify() {
        let v = filter().check("git commit --no-verify -m 'skip hooks'");
        assert!(!v.is_empty());
        assert_eq!(v[0].severity, Severity::Block);
    }

    #[test]
    fn blocks_no_gpg_sign() {
        let v = filter().check("git commit --no-gpg-sign -m 'unsigned'");
        assert!(!v.is_empty());
        assert_eq!(v[0].severity, Severity::Block);
    }

    // ── WARN_AND_CONFIRM patterns ───────────────────────────────────────

    #[test]
    fn warns_branch_force_delete() {
        let v = filter().check("git branch -D feature-old");
        assert!(!v.is_empty());
        assert_eq!(v[0].severity, Severity::WarnAndConfirm);
    }

    #[test]
    fn allows_branch_safe_delete() {
        let v = filter().check("git branch -d feature-old");
        let warns: Vec<_> = v
            .iter()
            .filter(|v| v.message.contains("force-delete"))
            .collect();
        assert!(warns.is_empty());
    }

    #[test]
    fn warns_git_clean_force() {
        let v = filter().check("git clean -fd");
        assert!(!v.is_empty());
        assert_eq!(v[0].severity, Severity::WarnAndConfirm);
    }

    #[test]
    fn warns_push_to_main() {
        let v = filter().check("git push origin main");
        assert!(!v.is_empty());
        assert_eq!(v[0].severity, Severity::WarnAndConfirm);
    }

    #[test]
    fn warns_push_to_master() {
        let v = filter().check("git push origin master");
        assert!(!v.is_empty());
        assert_eq!(v[0].severity, Severity::WarnAndConfirm);
    }

    // ── Safe commands pass through ──────────────────────────────────────

    #[test]
    fn allows_normal_git_push() {
        let v = filter().check("git push origin feature-branch");
        assert!(v.is_empty());
    }

    #[test]
    fn allows_cargo_test() {
        let v = filter().check("cargo test -p elai-tools");
        assert!(v.is_empty());
    }

    #[test]
    fn allows_ls_and_cat() {
        let v = filter().check("ls -la && cat README.md");
        assert!(v.is_empty());
    }

    #[test]
    fn allows_normal_rm() {
        let v = filter().check("rm temp.txt");
        assert!(v.is_empty());
    }

    // ── Integration: check_bash_safety ──────────────────────────────────

    #[test]
    fn check_bash_safety_rejects_blocked_command() {
        let result = check_bash_safety("git push --force origin main");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("BLOCK"));
        assert!(err.contains("force-with-lease"));
    }

    #[test]
    fn check_bash_safety_rejects_warn_in_autonomous() {
        let result = check_bash_safety("git branch -D old-branch");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("WARN"));
    }

    #[test]
    fn check_bash_safety_allows_safe_command() {
        let result = check_bash_safety("echo hello world");
        assert!(result.is_ok());
    }

    // ── worst_severity ──────────────────────────────────────────────────

    #[test]
    fn worst_severity_returns_block_over_warn() {
        // git push --force origin main triggers both Block (force) and Warn (main)
        let v = filter().check("git push --force origin main");
        assert!(v.len() >= 2);
        assert_eq!(
            SafetyFilter::worst_severity(&v),
            Some(Severity::Block)
        );
    }

    #[test]
    fn worst_severity_empty_returns_none() {
        assert_eq!(SafetyFilter::worst_severity(&[]), None);
    }

    // ── Edge cases ──────────────────────────────────────────────────────

    #[test]
    fn handles_multiline_command() {
        let v = filter().check("git push \\\n  --force \\\n  origin main");
        assert!(!v.is_empty());
    }

    #[test]
    fn handles_extra_whitespace() {
        let v = filter().check("  git   reset   --hard   HEAD  ");
        assert!(!v.is_empty());
        assert_eq!(v[0].severity, Severity::Block);
    }

    #[test]
    fn no_false_positive_on_force_in_comment() {
        // If someone echoes text containing "force", that's not a git push
        let v = filter().check("echo 'do not use --force flag'");
        // --force alone doesn't match the git push pattern
        let force_push: Vec<_> = v
            .iter()
            .filter(|v| v.message.contains("force push"))
            .collect();
        assert!(force_push.is_empty());
    }
}

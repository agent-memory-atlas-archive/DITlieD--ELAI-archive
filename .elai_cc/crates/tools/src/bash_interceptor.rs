//! I-46: Bash interceptor -- redirect, fixup, and allowed-prefix checks.
//!
//! Ported verbatim from gajae-code bash-interceptor.ts (Rule 1-3),
//! bash-command-fixup.ts (2-pattern deterministic table), and
//! bash-allowed-prefixes.ts (word-tokenizer + ban-char set).
//!
//! All rule tables are R8-frozen: a running plan must NOT modify these consts.
//! New verbs or fixup patterns require a separate plan step.
//!
//! # Position in the bash pipeline
//!
//! ```text
//! E-30 argv gate (hook_dispatcher)
//!   -> InProcessSupervisor gates (I-1, I-35)
//!   -> run_bash:
//!      [1] bash_interceptor::check_intercept   <- THIS MODULE
//!      [2] routing_rules::check_routing (rules 1+3: echo-redirect, sed/awk)
//!      [3] safety_filter::check_bash_safety
//!      [4] check_bash_blast_radius (G-3)
//!      [5] execute_bash
//! ```
//!
//! An intercepted command returns `Err(structured_error(...))` and NEVER
//! reaches `execute_bash`. This is the observable "no spawn" guarantee that
//! the contract test asserts structurally.

// ---------------------------------------------------------------------------
// R8-frozen redirect rules (ported verbatim from gajae settings-schema.ts:201-235)
// ---------------------------------------------------------------------------

/// A single redirect rule: regex pattern -> suggested ELAI tool + hint.
pub struct RedirectRule {
    /// Compiled regex matched against the trimmed command.
    pub pattern: &'static str,
    /// ELAI tool name the model should use instead.
    pub target_tool: &'static str,
    /// Human-readable message directing the model to the correct tool.
    pub message: &'static str,
}

/// R8-frozen redirect rule table.
///
/// Rule 1: cat/head/tail/less/more -> read_file
/// Rule 2: grep/rg/ripgrep/ag/ack -> grep_search
/// Rule 3: find/fd/locate (with -name/-iname/-type/--type/-glob) -> glob_search
///
/// Rules for sed/awk and echo/heredoc-redirect are intentionally omitted here
/// because routing_rules.rs rules 1+3 already cover them (not I-46 targets).
/// Adding them here would create a double-gate.
pub const REDIRECT_RULES: &[RedirectRule] = &[
    RedirectRule {
        pattern: r"^\s*(cat|head|tail|less|more)\s+",
        target_tool: "read_file",
        message: "Use the `read_file` tool instead of cat/head/tail. It provides better context and handles binary files.",
    },
    RedirectRule {
        pattern: r"^\s*(grep|rg|ripgrep|ag|ack)\s+",
        target_tool: "grep_search",
        message: "Use the `grep_search` tool instead of grep/rg. It respects .gitignore and provides structured output.",
    },
    RedirectRule {
        pattern: r"^\s*(find|fd|locate)\s+.*(-name|-iname|-type|--type|-glob)",
        target_tool: "glob_search",
        message: "Use the `glob_search` tool instead of find/fd. It respects .gitignore and is faster for glob patterns.",
    },
    RedirectRule {
        pattern: r"^\s*(ls)\s+.*[*?\[\]]",
        target_tool: "glob_search",
        message: "Use the `glob_search` tool instead of ls with glob patterns.",
    },
];

// ---------------------------------------------------------------------------
// R8-frozen fixup table (deterministic subset from gajae bash-command-fixup.ts)
// ---------------------------------------------------------------------------

/// A single fixup entry: suffix to strip from the last command segment.
pub struct FixupEntry {
    /// Token that triggers the fixup when it is the last meaningful segment token.
    pub strip_suffix: &'static str,
    /// Human-readable label logged in the SSB row.
    pub label: &'static str,
}

/// R8-frozen fixup table (2 patterns; no brush-parser dep needed).
///
/// Both fixups target trailing output-pipeline noise that the ELAI harness
/// already handles at the transport layer:
///  1. `| head [N]` / `| tail [N]` — output truncation; ELAI truncates at transport.
///  2. `2>&1` — stderr merge; ELAI's execute_bash already merges stderr.
pub const FIXUP_TABLE: &[FixupEntry] = &[
    FixupEntry {
        strip_suffix: "| head",
        label: "trailing-head-pipe",
    },
    FixupEntry {
        strip_suffix: "| tail",
        label: "trailing-tail-pipe",
    },
    FixupEntry {
        strip_suffix: "2>&1",
        label: "redundant-stderr-redirect",
    },
];

// ---------------------------------------------------------------------------
// R8-frozen allowed-prefix table
// ---------------------------------------------------------------------------

/// R8-frozen allowed-prefix table.
///
/// Commands whose first token exactly matches one of these prefixes are
/// allowed to execute without interception even when the ELAI interceptor
/// is active. This covers the canonical build/test/scm tools an executor
/// agent legitimately needs.
///
/// Ported from gajae bash-allowed-prefixes.ts; the gjc-specific entries
/// (gjc, ralplan, state) are omitted (ELAI-irrelevant).
pub const ALLOWED_PREFIXES: &[&str] = &[
    "cargo",
    "git",
    "pnpm",
    "npm",
    "npx",
    "python",
    "python3",
    "pytest",
    "rustfmt",
    "clippy-driver",
    "rustup",
    "ssh",
    "curl",
    "wget",
    "docker",
    "docker-compose",
    "powershell",
    "pwsh",
    "cmd",
];

// ---------------------------------------------------------------------------
// Ban sets (ported from gajae bash-allowed-prefixes.ts)
// ---------------------------------------------------------------------------

/// Shell control characters that are banned inside an allowed-prefix command.
/// A command containing any of these is classified as PrefixBlocked.
const SHELL_CONTROL_CHARS: &[char] = &[';', '|', '&', '<', '>', '(', ')'];

/// Unquoted shell expansion characters that are banned inside an
/// allowed-prefix command (outside of quoted strings).
const UNSAFE_EXPANSION_CHARS: &[char] = &['$', '*', '?', '[', ']', '{', '}', '~'];

// ---------------------------------------------------------------------------
// Result types
// ---------------------------------------------------------------------------

/// Result of a redirect classification pass.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RedirectDecision {
    /// The ELAI tool the model should call instead.
    pub target_tool: &'static str,
    /// Human-readable hint returned in the structured error message.
    pub message: &'static str,
}

/// Result of a fixup pass.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixedUpCommand {
    /// The rewritten command (with the trailing noise stripped).
    pub command: String,
    /// Label of the fixup that was applied.
    pub label: &'static str,
}

/// Result of an allowed-prefix check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AllowedPrefixResult {
    /// Command starts with a recognized allowed prefix and contains no banned tokens.
    Allowed,
    /// Command starts with a recognized prefix but contains a banned token.
    /// The prefix check fires and returns PrefixBlocked.
    PrefixBlocked { reason: String },
    /// Command does not start with any allowed prefix.
    /// Falls through to the redirect / routing layers.
    NotMatched,
}

// ---------------------------------------------------------------------------
// Compiled rules (OnceLock, compiled once at first call)
// ---------------------------------------------------------------------------

use std::sync::OnceLock;

struct CompiledRule {
    pattern: regex::Regex,
    target_tool: &'static str,
    message: &'static str,
}

fn compiled_redirect_rules() -> &'static Vec<CompiledRule> {
    static COMPILED: OnceLock<Vec<CompiledRule>> = OnceLock::new();
    COMPILED.get_or_init(|| {
        REDIRECT_RULES
            .iter()
            .map(|r| CompiledRule {
                pattern: regex::Regex::new(r.pattern)
                    .unwrap_or_else(|e| panic!("I-46 redirect rule regex compile failed: {e}")),
                target_tool: r.target_tool,
                message: r.message,
            })
            .collect()
    })
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Check whether the command should be redirected to a dedicated ELAI tool.
///
/// Returns `Some(RedirectDecision)` when the command matches a redirect rule.
/// Returns `None` when the command does not match any rule (falls through to
/// routing_rules and execute_bash).
///
/// Only the PRIMARY command segment is matched (first segment before any
/// `|`, `&&`, `||`, or `;`). Compound commands where the redirect verb
/// appears only in a non-primary position are allowed through.
pub fn classify_redirect(command: &str) -> Option<RedirectDecision> {
    let trimmed = command.trim();
    if trimmed.is_empty() {
        return None;
    }
    let primary = extract_primary_segment(trimmed);
    let primary = primary.trim();
    if primary.is_empty() {
        return None;
    }
    for rule in compiled_redirect_rules() {
        if rule.pattern.is_match(primary) {
            return Some(RedirectDecision {
                target_tool: rule.target_tool,
                message: rule.message,
            });
        }
    }
    None
}

/// Apply the R8-frozen fixup table to the command.
///
/// Returns `Some(FixedUpCommand)` with the rewritten command when a fixup
/// applies. Returns `None` when no fixup applies (command unchanged).
///
/// Fixups operate on the LAST segment of the command (split on `;`, `&&`,
/// `||`). They strip trailing output-pipeline noise that ELAI already handles.
pub fn apply_fixup(command: &str) -> Option<FixedUpCommand> {
    let trimmed = command.trim();
    if trimmed.is_empty() {
        return None;
    }
    // Walk the fixup table; apply the first matching fixup.
    for entry in FIXUP_TABLE {
        if let Some(rewritten) = try_strip_suffix(trimmed, entry.strip_suffix) {
            return Some(FixedUpCommand {
                command: rewritten,
                label: entry.label,
            });
        }
    }
    None
}

/// Check whether the command starts with a recognized allowed prefix.
///
/// Returns:
///   `AllowedPrefixResult::Allowed` -- starts with an allowed prefix, no banned tokens.
///   `AllowedPrefixResult::PrefixBlocked` -- starts with an allowed prefix, but
///     the body contains a banned shell token (shell control char, unquoted
///     expansion char, newline, backslash). Classified as PrefixBlocked so the
///     SSB row carries the correct arm.
///   `AllowedPrefixResult::NotMatched` -- does not start with any allowed prefix;
///     the caller continues to the redirect + routing layers.
pub fn check_allowed_prefix(command: &str, prefixes: &[&str]) -> AllowedPrefixResult {
    let trimmed = command.trim();
    if trimmed.is_empty() {
        return AllowedPrefixResult::NotMatched;
    }
    // Quick shell-word split (unquoted only; quoted-arg deferral is documented).
    let words: Vec<&str> = trimmed.split_whitespace().collect();
    if words.is_empty() {
        return AllowedPrefixResult::NotMatched;
    }
    // Check if first token matches an allowed prefix.
    let matched = prefixes
        .iter()
        .any(|p| words[0].eq_ignore_ascii_case(p.trim()));
    if !matched {
        return AllowedPrefixResult::NotMatched;
    }
    // Prefix matched. Now check for banned tokens in the rest of the command.
    // Scan the raw command string (post-prefix body) for banned chars.
    // We scan the whole string because split_whitespace loses quoting context.
    if let Some(reason) = scan_for_banned_tokens(trimmed) {
        return AllowedPrefixResult::PrefixBlocked { reason };
    }
    AllowedPrefixResult::Allowed
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Extract the primary (first) command segment from a potentially compound
/// command. Splits on `|`, `&&`, `||`, and `;`. Mirrors routing_rules.rs
/// `extract_primary_command` so both layers agree on segment boundaries.
fn extract_primary_segment(command: &str) -> &str {
    let delimiters: &[&str] = &["&&", "||", "|", ";"];
    let mut earliest: Option<usize> = None;
    for delim in delimiters {
        if let Some(pos) = command.find(delim) {
            match earliest {
                Some(current) if pos < current => earliest = Some(pos),
                None => earliest = Some(pos),
                _ => {}
            }
        }
    }
    match earliest {
        Some(pos) => &command[..pos],
        None => command,
    }
}

/// Try to strip a known trailing suffix from the last command segment.
///
/// Splits the command on `;`, `&&`, `||` to find the last segment, then checks
/// whether that segment ends with the given suffix (case-sensitive, after trim).
/// If so, reconstructs the command with the last segment trimmed.
///
/// A stripped trailing numeric argument (e.g. the `10` in `| head 10`) is also
/// removed by trimming the suffix position from the right.
fn try_strip_suffix(command: &str, suffix: &str) -> Option<String> {
    // Find the last segment by splitting on compound operators.
    let split_points: &[&str] = &["; ", " && ", " || "];
    let mut last_split: Option<usize> = None;
    for delim in split_points {
        // Find the last occurrence.
        let mut search = command;
        let mut offset = 0;
        while let Some(pos) = search.find(delim) {
            let abs = offset + pos;
            match last_split {
                Some(current) if abs > current => last_split = Some(abs),
                None => last_split = Some(abs),
                _ => {}
            }
            offset += pos + delim.len();
            search = &search[pos + delim.len()..];
        }
    }
    let (prefix_part, last_segment) = match last_split {
        Some(pos) => {
            // Find the exact delimiter length at that position.
            let (delim_len, sep) = split_points
                .iter()
                .find_map(|d| {
                    if command[pos..].starts_with(d) {
                        Some((d.len(), *d))
                    } else {
                        None
                    }
                })
                .unwrap_or((2, "; "));
            let _ = sep;
            (&command[..pos + delim_len], command[pos + delim_len..].trim())
        }
        None => ("", command.trim()),
    };

    let last_trimmed = last_segment.trim();

    // Check if the last segment contains the suffix as the last meaningful part.
    // Handle "| head" / "| head N" / "| tail" / "| tail N" and "2>&1".
    if suffix == "| head" || suffix == "| tail" {
        // Find " | head" or " | tail" in the last segment.
        let pipe_suffix = if suffix == "| head" { "| head" } else { "| tail" };
        if let Some(pipe_pos) = last_trimmed.rfind(pipe_suffix) {
            // Verify nothing follows except an optional integer.
            let after = last_trimmed[pipe_pos + pipe_suffix.len()..].trim();
            if after.is_empty() || after.parse::<u64>().is_ok() {
                let stripped = last_trimmed[..pipe_pos].trim().to_string();
                if stripped.is_empty() {
                    return None; // Whole command was "|head N", nothing left.
                }
                return Some(format!("{}{}", prefix_part, stripped));
            }
        }
        return None;
    }

    if suffix == "2>&1" {
        // Only strip a trailing "2>&1" if there is no pipe after it.
        if last_trimmed.ends_with("2>&1") {
            let stripped = last_trimmed[..last_trimmed.len() - 4].trim().to_string();
            if stripped.is_empty() {
                return None;
            }
            return Some(format!("{}{}", prefix_part, stripped));
        }
        return None;
    }

    None
}

/// Scan the command for banned shell tokens (control chars, unquoted expansion
/// chars, newlines, backslash escapes). Returns a reason string if banned.
///
/// This is a best-effort scan over the raw string; it does NOT perform full
/// shell parsing. Quoted arguments are not excluded from the scan (conservative:
/// if a command has any quoted arg with a dollar sign we still block it).
/// This matches the "MVP split_whitespace" scope documented in the design.
fn scan_for_banned_tokens(command: &str) -> Option<String> {
    // Check for shell control characters.
    for c in command.chars() {
        if SHELL_CONTROL_CHARS.contains(&c) {
            return Some(format!(
                "shell control operator '{}' is not allowed in a restricted allowed-prefix command",
                c
            ));
        }
    }
    // Check for unquoted expansion characters.
    for c in command.chars() {
        if UNSAFE_EXPANSION_CHARS.contains(&c) {
            return Some(format!(
                "shell expansion character '{}' is not allowed in a restricted allowed-prefix command",
                c
            ));
        }
    }
    // Check for newlines (multi-command).
    if command.contains('\n') || command.contains('\r') {
        return Some(
            "multiple shell commands (newline) are not allowed in a restricted allowed-prefix command"
                .to_string(),
        );
    }
    // Check for backslash escapes.
    if command.contains('\\') {
        return Some(
            "backslash escapes are not allowed in a restricted allowed-prefix command".to_string(),
        );
    }
    None
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- classify_redirect: primary-segment redirect --

    #[test]
    fn redirect_cat_to_read_file() {
        let r = classify_redirect("cat README.md");
        assert!(r.is_some());
        let d = r.unwrap();
        assert_eq!(d.target_tool, "read_file");
        assert!(d.message.contains("read_file"));
    }

    #[test]
    fn redirect_head_to_read_file() {
        let r = classify_redirect("head -n 10 src/main.rs");
        assert!(r.is_some());
        assert_eq!(r.unwrap().target_tool, "read_file");
    }

    #[test]
    fn redirect_tail_to_read_file() {
        let r = classify_redirect("tail -f /var/log/syslog");
        assert!(r.is_some());
        assert_eq!(r.unwrap().target_tool, "read_file");
    }

    #[test]
    fn redirect_grep_to_grep_search() {
        let r = classify_redirect("grep -r 'TODO' src/");
        assert!(r.is_some());
        assert_eq!(r.unwrap().target_tool, "grep_search");
    }

    #[test]
    fn redirect_rg_to_grep_search() {
        let r = classify_redirect("rg 'pattern' --type rust");
        assert!(r.is_some());
        assert_eq!(r.unwrap().target_tool, "grep_search");
    }

    #[test]
    fn redirect_find_glob_to_glob_search() {
        let r = classify_redirect("find . -name '*.rs'");
        assert!(r.is_some());
        assert_eq!(r.unwrap().target_tool, "glob_search");
    }

    #[test]
    fn redirect_ls_glob_to_glob_search() {
        let r = classify_redirect("ls src/*.ts");
        assert!(r.is_some());
        assert_eq!(r.unwrap().target_tool, "glob_search");
    }

    #[test]
    fn redirect_less_to_read_file() {
        let r = classify_redirect("less bigfile.txt");
        assert!(r.is_some());
        assert_eq!(r.unwrap().target_tool, "read_file");
    }

    #[test]
    fn redirect_ripgrep_to_grep_search() {
        let r = classify_redirect("ripgrep pattern src/");
        assert!(r.is_some());
        assert_eq!(r.unwrap().target_tool, "grep_search");
    }

    // -- classify_redirect: non-primary segment NOT redirected --

    #[test]
    fn cat_after_pipe_not_redirected() {
        // cat is not the primary command; should NOT intercept.
        assert!(classify_redirect("docker logs foo | cat").is_none());
    }

    #[test]
    fn grep_after_pipe_not_redirected() {
        assert!(classify_redirect("ps aux | grep nginx").is_none());
    }

    // -- classify_redirect: safe commands pass through --

    #[test]
    fn cargo_passes_through() {
        assert!(classify_redirect("cargo check --workspace").is_none());
    }

    #[test]
    fn git_passes_through() {
        assert!(classify_redirect("git status").is_none());
    }

    #[test]
    fn bare_cat_no_args_passes() {
        // "cat" alone without a file arg does not match r"^\s*(cat|...)\s+"
        assert!(classify_redirect("cat").is_none());
    }

    #[test]
    fn find_without_name_flag_passes() {
        // find without -name/-iname/-type/-glob flags passes through.
        assert!(classify_redirect("find . -maxdepth 2").is_none());
    }

    // -- apply_fixup --

    #[test]
    fn fixup_strips_trailing_head_pipe() {
        let r = apply_fixup("ls -la | head 20");
        assert!(r.is_some());
        let f = r.unwrap();
        assert_eq!(f.command, "ls -la");
        assert_eq!(f.label, "trailing-head-pipe");
    }

    #[test]
    fn fixup_strips_trailing_tail_pipe() {
        let r = apply_fixup("cat log.txt | tail 10");
        assert!(r.is_some());
        let f = r.unwrap();
        assert_eq!(f.command, "cat log.txt");
    }

    #[test]
    fn fixup_strips_redundant_stderr_redirect() {
        let r = apply_fixup("some_command arg 2>&1");
        assert!(r.is_some());
        let f = r.unwrap();
        assert_eq!(f.command, "some_command arg");
        assert_eq!(f.label, "redundant-stderr-redirect");
    }

    #[test]
    fn fixup_no_match_returns_none() {
        assert!(apply_fixup("cargo check --workspace").is_none());
    }

    // -- check_allowed_prefix --

    #[test]
    fn allowed_prefix_cargo_passes() {
        assert_eq!(
            check_allowed_prefix("cargo check -p elai-tools", ALLOWED_PREFIXES),
            AllowedPrefixResult::Allowed
        );
    }

    #[test]
    fn allowed_prefix_git_passes() {
        assert_eq!(
            check_allowed_prefix("git status", ALLOWED_PREFIXES),
            AllowedPrefixResult::Allowed
        );
    }

    #[test]
    fn allowed_prefix_not_matched_returns_not_matched() {
        assert_eq!(
            check_allowed_prefix("grep -r pattern src/", ALLOWED_PREFIXES),
            AllowedPrefixResult::NotMatched
        );
    }

    #[test]
    fn allowed_prefix_shell_control_blocked() {
        let r = check_allowed_prefix("cargo check; rm -rf .", ALLOWED_PREFIXES);
        assert!(matches!(r, AllowedPrefixResult::PrefixBlocked { .. }));
    }

    #[test]
    fn allowed_prefix_expansion_char_blocked() {
        let r = check_allowed_prefix("cargo test $EVIL", ALLOWED_PREFIXES);
        assert!(matches!(r, AllowedPrefixResult::PrefixBlocked { .. }));
    }

    #[test]
    fn allowed_prefix_pipe_blocked() {
        let r = check_allowed_prefix("git log | grep foo", ALLOWED_PREFIXES);
        assert!(matches!(r, AllowedPrefixResult::PrefixBlocked { .. }));
    }
}

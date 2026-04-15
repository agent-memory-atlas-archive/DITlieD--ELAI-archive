//! A-26: Tool routing layer for the Bash tool handler.
//!
//! Intercepts Bash commands that should use dedicated tools (Read, Edit, Write,
//! Glob, Grep) and rejects them with a hint message. This replaces ~375 prompt
//! tokens of "use X not Y" instructions with a physical gate that local models
//! cannot bypass.
//!
//! # Config wiring point
//!
//! To enable runtime configuration, add a `routing_config` field to the tools
//! section of `elai_cc.toml`:
//!
//! ```toml
//! [tools.routing_rules]
//! enabled = true
//! allow_override_prefix = "!"
//! ```
//!
//! The types below are already serde-ready. Currently the builtin rules use
//! hardcoded constants; a full runtime config override (loading `RoutingConfig`
//! from `elai_cc.toml` and passing it into `check_routing`) is planned for the
//! `tools.routing_rules` TOML section.

use regex::Regex;
use std::sync::OnceLock;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Number of built-in routing rules.
const BUILTIN_RULE_COUNT: usize = 5;

/// Override prefix: commands starting with '!' bypass routing checks.
const OVERRIDE_PREFIX: char = '!';

// ---------------------------------------------------------------------------
// Config (serde-ready; not yet wired to elai_cc.toml loader)
// ---------------------------------------------------------------------------

/// Runtime configuration for the routing rules layer.
///
/// All fields have sane defaults so `RoutingConfig::default()` gives the same
/// behaviour as the current hardcoded constants. Wire this into the tools
/// config loader when `[tools.routing_rules]` TOML support is added.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct RoutingConfig {
    /// When `false`, `check_routing` always returns `None` (all commands pass).
    pub enabled: bool,
    /// Commands that start with this string bypass routing entirely.
    /// Maps to the `OVERRIDE_PREFIX` constant when config is not loaded.
    pub allow_override_prefix: String,
}

impl Default for RoutingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            allow_override_prefix: OVERRIDE_PREFIX.to_string(),
        }
    }
}

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// A single routing rule that maps a Bash command pattern to the correct
/// dedicated tool.
pub struct RoutingRule {
    /// Compiled regex matched against the primary command token(s).
    pattern: Regex,
    /// Name of the dedicated tool the caller should use instead.
    pub target_tool: &'static str,
    /// Human-readable hint returned in the rejection message.
    pub hint: &'static str,
}

/// Rejection returned when a command matches a routing rule.
pub struct RoutingRejection {
    /// Which dedicated tool should be used.
    pub target_tool: &'static str,
    /// Full hint message for the model.
    pub hint: String,
}

// ---------------------------------------------------------------------------
// Built-in rules (compiled once via OnceLock)
// ---------------------------------------------------------------------------

fn builtin_rules() -> &'static [RoutingRule; BUILTIN_RULE_COUNT] {
    static RULES: OnceLock<[RoutingRule; BUILTIN_RULE_COUNT]> = OnceLock::new();
    RULES.get_or_init(|| {
        [
            // echo ... > / cat <<EOF / heredoc -> Write (MUST precede cat/read rule)
            RoutingRule {
                pattern: Regex::new(r"^(echo\s+.*>\s*\S|cat\s*<<)").expect("write rule regex"),
                target_tool: "write_file",
                hint: "Use the Write tool instead of echo/heredoc redirection to create files.",
            },
            // cat/head/tail + filepath -> Read
            RoutingRule {
                pattern: Regex::new(r"^(cat|head|tail)\s+\S").expect("read rule regex"),
                target_tool: "read_file",
                hint: "Use the Read tool instead of cat/head/tail to read file contents.",
            },
            // sed/awk + filepath -> Edit
            RoutingRule {
                pattern: Regex::new(r"^(sed|awk)\s+").expect("edit rule regex"),
                target_tool: "edit_file",
                hint: "Use the Edit tool instead of sed/awk to modify files.",
            },
            // find/ls + glob-like pattern -> Glob
            RoutingRule {
                pattern: Regex::new(r"^(find|ls)\s+.*[*?]").expect("glob rule regex"),
                target_tool: "glob_search",
                hint: "Use the Glob tool instead of find/ls for file pattern matching.",
            },
            // grep/rg + pattern -> Grep
            RoutingRule {
                pattern: Regex::new(r"^(grep|rg)\s+").expect("grep rule regex"),
                target_tool: "grep_search",
                hint: "Use the Grep tool instead of grep/rg for content search.",
            },
        ]
    })
}

// ---------------------------------------------------------------------------
// Runtime config (set once at startup from elai_cc.toml)
// ---------------------------------------------------------------------------

/// Initialize the routing config from `elai_cc.toml` at startup.
///
/// Call this once before any tool invocations. Safe to call multiple times —
/// only the first call takes effect (OnceLock semantics).
pub fn init_routing_config(config: RoutingConfig) {
    let _ = RUNTIME_ROUTING_CONFIG.set(config);
}

static RUNTIME_ROUTING_CONFIG: std::sync::OnceLock<RoutingConfig> = std::sync::OnceLock::new();

fn runtime_routing_config() -> &'static RoutingConfig {
    RUNTIME_ROUTING_CONFIG.get_or_init(RoutingConfig::default)
}

// ---------------------------------------------------------------------------
// Core routing check
// ---------------------------------------------------------------------------

/// Check whether a Bash command should be routed to a dedicated tool.
///
/// Returns `Some(RoutingRejection)` if the command matches a routing rule,
/// `None` if the command should proceed normally.
///
/// Commands prefixed with '!' bypass routing entirely (the prefix is NOT
/// stripped — callers must handle that separately if desired).
///
/// Only the primary command is checked — the first token before any pipe,
/// or the first token after the last pipe. Compound commands where the
/// pattern appears only in a non-primary position are allowed through.
pub fn check_routing(command: &str) -> Option<RoutingRejection> {
    let config = runtime_routing_config();

    // When routing is disabled, all commands pass through.
    if !config.enabled {
        return None;
    }

    let trimmed = command.trim();

    // Override escape: configured prefix skips all routing.
    let override_prefix = config
        .allow_override_prefix
        .chars()
        .next()
        .unwrap_or(OVERRIDE_PREFIX);
    if trimmed.starts_with(override_prefix) {
        return None;
    }

    // Extract the primary command segment. For piped commands we check
    // only the first segment (the primary command). Compound operators
    // (&&, ||, ;) also isolate the first segment.
    let primary = extract_primary_command(trimmed);
    let primary = primary.trim();

    if primary.is_empty() {
        return None;
    }

    let rules = builtin_rules();
    for rule in rules.iter() {
        if rule.pattern.is_match(primary) {
            return Some(RoutingRejection {
                target_tool: rule.target_tool,
                hint: rule.hint.to_string(),
            });
        }
    }

    None
}

/// Strip the override prefix from a command, if present.
/// The prefix character is taken from the runtime routing config (default: '!').
/// Returns the command unchanged if no prefix.
pub fn strip_override_prefix(command: &str) -> &str {
    let trimmed = command.trim();
    let prefix = runtime_routing_config()
        .allow_override_prefix
        .chars()
        .next()
        .unwrap_or(OVERRIDE_PREFIX);
    if trimmed.starts_with(prefix) {
        trimmed[1..].trim_start()
    } else {
        trimmed
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Extract the primary (first) command segment from a potentially compound
/// command string. Splits on pipe `|`, `&&`, `||`, and `;`.
fn extract_primary_command(command: &str) -> &str {
    // Find the earliest delimiter position.
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

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- Read tool routing --

    #[test]
    fn cat_file_is_rejected() {
        let r = check_routing("cat /tmp/foo.txt");
        assert!(r.is_some());
        let r = r.unwrap();
        assert_eq!(r.target_tool, "read_file");
        assert!(r.hint.contains("Read tool"));
    }

    #[test]
    fn head_file_is_rejected() {
        let r = check_routing("head -n 10 src/main.rs");
        assert!(r.is_some());
        assert_eq!(r.unwrap().target_tool, "read_file");
    }

    #[test]
    fn tail_file_is_rejected() {
        let r = check_routing("tail -f /var/log/syslog");
        assert!(r.is_some());
        assert_eq!(r.unwrap().target_tool, "read_file");
    }

    // -- Edit tool routing --

    #[test]
    fn sed_is_rejected() {
        let r = check_routing("sed -i 's/old/new/g' file.txt");
        assert!(r.is_some());
        assert_eq!(r.unwrap().target_tool, "edit_file");
    }

    #[test]
    fn awk_is_rejected() {
        let r = check_routing("awk '{print $1}' data.csv");
        assert!(r.is_some());
        assert_eq!(r.unwrap().target_tool, "edit_file");
    }

    // -- Write tool routing --

    #[test]
    fn echo_redirect_is_rejected() {
        let r = check_routing("echo 'hello' > output.txt");
        assert!(r.is_some());
        assert_eq!(r.unwrap().target_tool, "write_file");
    }

    #[test]
    fn cat_heredoc_is_rejected() {
        let r = check_routing("cat <<EOF > file.txt");
        assert!(r.is_some());
        assert_eq!(r.unwrap().target_tool, "write_file");
    }

    // -- Glob tool routing --

    #[test]
    fn find_with_glob_is_rejected() {
        let r = check_routing("find . -name '*.rs'");
        assert!(r.is_some());
        assert_eq!(r.unwrap().target_tool, "glob_search");
    }

    #[test]
    fn ls_with_glob_is_rejected() {
        let r = check_routing("ls src/*.ts");
        assert!(r.is_some());
        assert_eq!(r.unwrap().target_tool, "glob_search");
    }

    // -- Grep tool routing --

    #[test]
    fn grep_is_rejected() {
        let r = check_routing("grep -r 'TODO' src/");
        assert!(r.is_some());
        assert_eq!(r.unwrap().target_tool, "grep_search");
    }

    #[test]
    fn rg_is_rejected() {
        let r = check_routing("rg 'pattern' --type rust");
        assert!(r.is_some());
        assert_eq!(r.unwrap().target_tool, "grep_search");
    }

    // -- Override prefix --

    #[test]
    fn override_prefix_bypasses_routing() {
        assert!(check_routing("!cat /tmp/foo.txt").is_none());
        assert!(check_routing("! grep pattern file").is_none());
    }

    #[test]
    fn strip_override_prefix_works() {
        assert_eq!(strip_override_prefix("!cat foo"), "cat foo");
        assert_eq!(strip_override_prefix("! cat foo"), "cat foo");
        assert_eq!(strip_override_prefix("cat foo"), "cat foo");
    }

    // -- Compound commands: only primary segment checked --

    #[test]
    fn cat_after_pipe_is_allowed() {
        // cat is NOT the primary command here, so it should pass.
        assert!(check_routing("docker logs foo | cat").is_none());
    }

    #[test]
    fn grep_after_pipe_is_allowed() {
        assert!(check_routing("ps aux | grep nginx").is_none());
    }

    #[test]
    fn cat_after_and_operator_is_allowed() {
        assert!(check_routing("cd /tmp && cat foo.txt").is_none());
    }

    #[test]
    fn cat_after_semicolon_is_allowed() {
        assert!(check_routing("echo hi; cat foo.txt").is_none());
    }

    // -- Safe commands that should NOT be routed --

    #[test]
    fn git_commands_pass_through() {
        assert!(check_routing("git status").is_none());
        assert!(check_routing("git log --oneline -10").is_none());
    }

    #[test]
    fn cargo_commands_pass_through() {
        assert!(check_routing("cargo test -p elai-tools").is_none());
        assert!(check_routing("cargo check --workspace").is_none());
    }

    #[test]
    fn npm_commands_pass_through() {
        assert!(check_routing("npm install").is_none());
    }

    #[test]
    fn bare_cat_no_args_passes() {
        // "cat" alone with no file arg should not trigger (no \S after).
        assert!(check_routing("cat").is_none());
    }

    #[test]
    fn find_without_glob_passes() {
        // find without glob characters should pass through.
        assert!(check_routing("find . -name foo").is_none());
    }

    #[test]
    fn ls_without_glob_passes() {
        assert!(check_routing("ls -la").is_none());
        assert!(check_routing("ls src/").is_none());
    }

    #[test]
    fn echo_without_redirect_passes() {
        assert!(check_routing("echo hello world").is_none());
    }

    // -- extract_primary_command --

    #[test]
    fn extract_primary_simple() {
        assert_eq!(extract_primary_command("cat foo.txt"), "cat foo.txt");
    }

    #[test]
    fn extract_primary_piped() {
        assert_eq!(
            extract_primary_command("ps aux | grep nginx").trim(),
            "ps aux"
        );
    }

    #[test]
    fn extract_primary_and_chain() {
        assert_eq!(
            extract_primary_command("cd /tmp && cat foo").trim(),
            "cd /tmp"
        );
    }

    #[test]
    fn extract_primary_semicolon() {
        assert_eq!(
            extract_primary_command("echo hi; cat foo").trim(),
            "echo hi"
        );
    }
}

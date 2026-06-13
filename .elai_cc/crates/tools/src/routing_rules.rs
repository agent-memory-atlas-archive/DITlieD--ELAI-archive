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
///
/// I-46 (OPTION A consolidation): rules 2 (cat/head/tail), 4 (find+glob),
/// 5 (grep/rg) removed — those verbs are now handled by
/// `bash_interceptor::classify_redirect` which fires before this layer and
/// returns a richer structured redirect message. Only rules 1 (echo-redirect/
/// heredoc -> write_file) and 3 (sed/awk -> edit_file) are retained here.
const BUILTIN_RULE_COUNT: usize = 2;

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
            // Rule 1: echo ... > / cat <<EOF / heredoc -> Write (I-46 OPTION A: cat/head/
            // tail/less/more and grep/rg and find/ls-glob are handled by bash_interceptor.rs
            // and removed from this table; the echo-redirect variant is NOT an I-46 target).
            RoutingRule {
                pattern: Regex::new(r"^(echo\s+.*>\s*\S|cat\s*<<)").expect("write rule regex"),
                target_tool: "write_file",
                hint: "Use the Write tool instead of echo/heredoc redirection to create files.",
            },
            // Rule 3: sed/awk + filepath -> Edit (retained; not an I-46 intercept target).
            RoutingRule {
                pattern: Regex::new(r"^(sed|awk)\s+").expect("edit rule regex"),
                target_tool: "edit_file",
                hint: "Use the Edit tool instead of sed/awk to modify files.",
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
    check_routing_with_config(command, runtime_routing_config())
}

/// Cold-wiring (s-81344): config-explicit variant of [`check_routing`].
///
/// Identical logic to [`check_routing`] but takes the [`RoutingConfig`]
/// explicitly instead of reading the process-global `OnceLock`
/// ([`runtime_routing_config`]). This lets a caller (and the unit tests)
/// supply a config whose `allow_override_prefix` takes precedence over the
/// [`OVERRIDE_PREFIX`] const without mutating global state. `check_routing`
/// delegates here with the runtime global, so the live behaviour is unchanged.
pub fn check_routing_with_config(
    command: &str,
    config: &RoutingConfig,
) -> Option<RoutingRejection> {
    // When routing is disabled, all commands pass through.
    if !config.enabled {
        return None;
    }

    let trimmed = command.trim();

    // Override escape: configured prefix skips all routing. The prefix char is
    // taken from the config; an empty config prefix falls back to the
    // OVERRIDE_PREFIX const (so config beats const only when config supplies a
    // non-empty prefix). `.unwrap_or` here is the safe Option method, not the
    // banned `.unwrap()`.
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

    // NOTE: cat/head/tail, grep/rg, find/ls-glob tests removed (I-46 OPTION A).
    // Those verbs are now intercepted by bash_interceptor::classify_redirect.
    // Coverage is preserved in crates/contracts/tests/i46_bash_interceptor_contract.rs.

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

    // -- Cold-wiring (s-81344): check_routing_with_config --

    #[test]
    fn with_config_disabled_passes_all_commands() {
        let disabled = RoutingConfig {
            enabled: false,
            allow_override_prefix: "!".to_string(),
        };
        // Use sed (rule 3, retained post-I-46 OPTION A) as the representative
        // "normally-routed command". cat/head/tail/grep/find are no longer routed
        // by this layer (they were moved to bash_interceptor in I-46).
        // A normally-routed command passes through when the PASSED config
        // disables routing (proves the supplied config's `enabled` governs).
        assert!(check_routing_with_config("sed 's/foo/bar/' file.txt", &disabled).is_none());
        let enabled = RoutingConfig {
            enabled: true,
            allow_override_prefix: "!".to_string(),
        };
        assert!(check_routing_with_config("sed 's/foo/bar/' file.txt", &enabled).is_some());
    }

    #[test]
    fn with_config_custom_prefix_beats_override_const() {
        // Config supplies a custom override prefix '@' (NOT the OVERRIDE_PREFIX
        // '!' const). The custom prefix must bypass routing, proving the config
        // value takes precedence over the const.
        // Use sed (rule 3, retained post-I-46 OPTION A) as the representative verb.
        let custom = RoutingConfig {
            enabled: true,
            allow_override_prefix: "@".to_string(),
        };
        assert!(check_routing_with_config("@sed 's/foo/bar/' file.txt", &custom).is_none());
        // The un-prefixed command still routes under the same config (the
        // custom config did not simply disable everything).
        assert!(check_routing_with_config("sed 's/foo/bar/' file.txt", &custom).is_some());
    }

    #[test]
    fn with_config_empty_prefix_falls_back_to_override_const() {
        // Empty prefix -> chars().next() is None -> unwrap_or(OVERRIDE_PREFIX),
        // so the const '!' is the active bypass char.
        // Use sed (rule 3, retained post-I-46 OPTION A) as the representative verb.
        let empty = RoutingConfig {
            enabled: true,
            allow_override_prefix: String::new(),
        };
        assert!(check_routing_with_config("!sed 's/foo/bar/' file.txt", &empty).is_none());
    }

    #[test]
    fn check_routing_delegates_to_with_config_default() {
        // check_routing must equal check_routing_with_config(_, runtime default).
        // Use sed (rule 3, retained post-I-46 OPTION A) as the representative verb.
        // cat/head/tail are now intercepted by bash_interceptor (before routing_rules).
        let default_cfg = RoutingConfig::default();
        let a = check_routing("sed 's/foo/bar/' file.txt");
        let b = check_routing_with_config("sed 's/foo/bar/' file.txt", &default_cfg);
        assert_eq!(a.is_some(), b.is_some());
        assert_eq!(a.map(|r| r.target_tool), b.map(|r| r.target_tool));
        // A non-routed (piped) command agrees on both paths too.
        assert_eq!(
            check_routing("ps aux | awk '{print $1}'").is_none(),
            check_routing_with_config("ps aux | awk '{print $1}'", &default_cfg).is_none()
        );
    }
}

//! Action classifier + confirmation gate (A-31).
//!
//! Classifies every tool call by risk level and gates dangerous actions.
//! Integrates into PreToolUse hook chain. Extends DAIRA trace with risk data.
//!
//! WIRING: add `pub mod action_classifier;` to lib.rs, then call
//! `action_classifier::classify_and_gate()` in `execute_tool_with_enforcer`
//! before the match dispatch.

use elai_orchestrator::bash_validation::{classify_command, CommandIntent};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Risk classification
// ---------------------------------------------------------------------------

/// Risk level for a tool invocation, ordered by severity.
/// Source: claude-code-system-prompts-full.md section 6 (reversibility framework).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum RiskLevel {
    /// Read-only, no side effects. Always allowed.
    Safe,
    /// Local mutation, easily undone. Log but allow.
    Reversible,
    /// Hard to reverse: git push, file delete, db mutation.
    Irreversible,
    /// Visible to others: PR create, message send, deploy.
    SharedState,
}

/// Result of classifying a single tool call.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActionClassification {
    pub risk: RiskLevel,
    pub reason: String,
}

// ---------------------------------------------------------------------------
// Constants: tool name -> default risk
// ---------------------------------------------------------------------------

/// Tools that are always safe (read-only, no side effects).
const SAFE_TOOLS: &[&str] = &[
    "read_file",
    "glob_search",
    "grep_search",
    "WebSearch",
    "WebFetch",
    "ToolSearch",
    "TodoWrite",
    "Skill",
    "Agent",
    "Sleep",
    "SendUserMessage",
    "Brief",
    "EnterPlanMode",
    "ExitPlanMode",
    "TaskGet",
    "TaskList",
    "TaskOutput",
    "WorkerGet",
    "WorkerObserve",
    "WorkerAwaitReady",
    "CronList",
];

/// MCP tools that are always safe (read-only reasoning/search).
const SAFE_MCP_PREFIXES: &[&str] = &[
    "mcp_think",
    "mcp_search_context",
    "mcp_list_topics",
    "mcp_fsm_get_state",
    "mcp_session_list",
];

/// Tools that produce reversible local mutations.
const REVERSIBLE_TOOLS: &[&str] = &[
    "write_file",
    "edit_file",
    "NotebookEdit",
    "Config",
    "REPL",
    "PowerShell",
    "StructuredOutput",
];

/// Tools that affect shared state (visible to others).
const SHARED_STATE_TOOLS: &[&str] = &[
    "CronCreate",
    "CronDelete",
    "TeamCreate",
    "TeamDelete",
    "WorkerCreate",
    "WorkerTerminate",
    "WorkerRestart",
    "WorkerSendPrompt",
    "TaskCreate",
    "TaskStop",
    "TaskUpdate",
    "RunTaskPacket",
];

// ---------------------------------------------------------------------------
// Bash argument patterns for risk sub-classification
// ---------------------------------------------------------------------------

/// Bash patterns that escalate to IRREVERSIBLE.
const BASH_IRREVERSIBLE_PATTERNS: &[(&str, &str)] = &[
    ("git push", "pushes commits to remote repository"),
    ("git reset --hard", "discards all uncommitted changes"),
    ("git checkout -- ", "discards file changes irreversibly"),
    ("git restore -- ", "discards file changes irreversibly"),
    ("git clean -f", "deletes untracked files permanently"),
    ("git branch -D", "force-deletes a branch"),
    ("rm ", "deletes files"),
    ("rmdir", "removes directories"),
    ("shred", "securely destroys file data"),
    ("truncate", "truncates file to zero length"),
    ("DROP TABLE", "drops database table"),
    ("DROP DATABASE", "drops entire database"),
    ("DELETE FROM", "deletes database rows"),
];

/// Bash patterns that escalate to SHARED_STATE.
const BASH_SHARED_STATE_PATTERNS: &[(&str, &str)] = &[
    ("gh pr create", "creates a pull request visible to team"),
    ("gh pr comment", "comments on a pull request"),
    ("gh pr close", "closes a pull request"),
    ("gh pr merge", "merges a pull request"),
    ("gh issue create", "creates an issue visible to team"),
    ("gh issue comment", "comments on an issue"),
    ("gh issue close", "closes an issue"),
    ("gh release create", "creates a release"),
    ("curl -X POST", "sends data to external service"),
    ("curl -X PUT", "sends data to external service"),
    ("curl -X DELETE", "deletes resource on external service"),
    ("deploy", "deploys to shared infrastructure"),
];

/// Bash commands that are always safe (read-only).
const BASH_SAFE_PATTERNS: &[&str] = &[
    "git status",
    "git log",
    "git diff",
    "git show",
    "git branch",
    "git remote",
    "git fetch",
    "git stash list",
    "cargo check",
    "cargo clippy",
    "cargo fmt --check",
    "ls",
    "pwd",
    "echo",
    "cat",
    "head",
    "tail",
    "wc",
    "which",
];

// ---------------------------------------------------------------------------
// Classification logic
// ---------------------------------------------------------------------------

/// Classify a tool call by risk level.
///
/// Uses tool name for initial classification, then inspects arguments
/// for Bash/PowerShell to refine based on command patterns.
#[must_use]
pub fn classify(tool_name: &str, input: &serde_json::Value) -> ActionClassification {
    // Check safe tools first (fast path).
    if SAFE_TOOLS.contains(&tool_name) {
        return ActionClassification {
            risk: RiskLevel::Safe,
            reason: format!("{tool_name} is read-only"),
        };
    }

    // Check safe MCP prefixes.
    for prefix in SAFE_MCP_PREFIXES {
        if tool_name.starts_with(prefix) {
            return ActionClassification {
                risk: RiskLevel::Safe,
                reason: format!("{tool_name} is a safe MCP operation"),
            };
        }
    }

    // Shared state tools.
    if SHARED_STATE_TOOLS.contains(&tool_name) {
        return ActionClassification {
            risk: RiskLevel::SharedState,
            reason: format!("{tool_name} affects shared state"),
        };
    }

    // Reversible tools.
    if REVERSIBLE_TOOLS.contains(&tool_name) {
        return ActionClassification {
            risk: RiskLevel::Reversible,
            reason: format!("{tool_name} is a local mutation"),
        };
    }

    // Bash/PowerShell: sub-classify by command content.
    if tool_name == "bash" || tool_name == "PowerShell" {
        return classify_bash(input);
    }

    // AskUserQuestion is safe (just prompts user).
    if tool_name == "AskUserQuestion" {
        return ActionClassification {
            risk: RiskLevel::Safe,
            reason: "user interaction only".to_string(),
        };
    }

    // Unknown tools default to Reversible (conservative but not blocking).
    ActionClassification {
        risk: RiskLevel::Reversible,
        reason: format!("unknown tool '{tool_name}' defaulting to reversible"),
    }
}

/// Sub-classify a Bash/PowerShell command by inspecting the command string.
fn classify_bash(input: &serde_json::Value) -> ActionClassification {
    let command = input
        .get("command")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    if command.is_empty() {
        return ActionClassification {
            risk: RiskLevel::Safe,
            reason: "empty command".to_string(),
        };
    }

    // Check shared state patterns first (highest priority).
    for &(pattern, reason) in BASH_SHARED_STATE_PATTERNS {
        if command.contains(pattern) {
            return ActionClassification {
                risk: RiskLevel::SharedState,
                reason: reason.to_string(),
            };
        }
    }

    // Check irreversible patterns.
    for &(pattern, reason) in BASH_IRREVERSIBLE_PATTERNS {
        if command.contains(pattern) {
            return ActionClassification {
                risk: RiskLevel::Irreversible,
                reason: reason.to_string(),
            };
        }
    }

    // Check safe patterns (explicit whitelist).
    for &pattern in BASH_SAFE_PATTERNS {
        if command.starts_with(pattern) {
            return ActionClassification {
                risk: RiskLevel::Safe,
                reason: format!("'{pattern}' is a read-only command"),
            };
        }
    }

    // Fall back to semantic classification from bash_validation.
    match classify_command(command) {
        CommandIntent::ReadOnly => ActionClassification {
            risk: RiskLevel::Safe,
            reason: "command classified as read-only".to_string(),
        },
        CommandIntent::Destructive => ActionClassification {
            risk: RiskLevel::Irreversible,
            reason: "command classified as destructive".to_string(),
        },
        CommandIntent::Network => ActionClassification {
            risk: RiskLevel::Reversible,
            reason: "network command (no shared state mutation detected)".to_string(),
        },
        CommandIntent::Write
        | CommandIntent::ProcessManagement
        | CommandIntent::PackageManagement
        | CommandIntent::SystemAdmin
        | CommandIntent::Unknown => ActionClassification {
            risk: RiskLevel::Reversible,
            reason: "local mutation or unclassified command".to_string(),
        },
    }
}

// ---------------------------------------------------------------------------
// Confirmation gate
// ---------------------------------------------------------------------------

/// Outcome of the confirmation gate check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GateDecision {
    /// Tool call may proceed without confirmation.
    Allow,
    /// Tool call requires user confirmation before execution.
    ConfirmationRequired { reason: String },
    /// Tool call is blocked (no override available).
    Block { reason: String },
}

/// Execution mode controlling gate behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ExecutionMode {
    /// Interactive: user present, can confirm.
    #[default]
    Interactive,
    /// Autonomous: no user, policy overrides only.
    Autonomous,
}

/// Per-tool or per-pattern policy override.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyOverride {
    /// Override risk to this level (None = use classifier).
    pub risk_override: Option<RiskLevel>,
    /// If true, allow without confirmation even if classified as dangerous.
    pub allow_autonomous: bool,
}

/// Configuration for the action gate, loadable from elai_cc.toml [actions].
///
/// TOML wiring point (same pattern as A-26/A-27/A-30):
///   [actions]
///   mode = "Autonomous"            # or "Interactive" (default)
///   [actions.tool_overrides.bash]
///   allow_autonomous = true
///   [actions.pattern_overrides."git push"]
///   risk_override = "Reversible"
///
/// Load via: `toml::from_str::<ActionPolicy>(config_str)` and pass to
/// `classify_and_gate()` instead of `ActionPolicy::default()`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ActionPolicy {
    /// Per-tool name overrides.
    #[serde(default)]
    pub tool_overrides: HashMap<String, PolicyOverride>,
    /// Per-pattern overrides for Bash commands (substring match).
    #[serde(default)]
    pub pattern_overrides: HashMap<String, PolicyOverride>,
    /// Default execution mode.
    #[serde(default)]
    pub mode: ExecutionMode,
}

impl ActionPolicy {
    /// Look up a policy override for a tool + input combination.
    fn lookup_override(
        &self,
        tool_name: &str,
        input: &serde_json::Value,
    ) -> Option<&PolicyOverride> {
        // Check tool-level override first.
        if let Some(ov) = self.tool_overrides.get(tool_name) {
            return Some(ov);
        }
        // For Bash, check pattern overrides against command string.
        if tool_name == "bash" || tool_name == "PowerShell" {
            let command = input
                .get("command")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            for (pattern, ov) in &self.pattern_overrides {
                if command.contains(pattern.as_str()) {
                    return Some(ov);
                }
            }
        }
        None
    }
}

/// Check the confirmation gate for a classified action.
///
/// Returns whether the tool call should proceed, require confirmation,
/// or be blocked entirely.
#[must_use]
pub fn check_gate(
    classification: &ActionClassification,
    policy: &ActionPolicy,
    tool_name: &str,
    input: &serde_json::Value,
) -> GateDecision {
    let effective_risk = if let Some(ov) = policy.lookup_override(tool_name, input) {
        // Apply risk override if configured.
        if let Some(override_risk) = ov.risk_override {
            override_risk
        } else {
            classification.risk
        }
    } else {
        classification.risk
    };

    match effective_risk {
        RiskLevel::Safe | RiskLevel::Reversible => GateDecision::Allow,
        RiskLevel::Irreversible => {
            // Check for autonomous override.
            if policy.mode == ExecutionMode::Autonomous {
                if let Some(ov) = policy.lookup_override(tool_name, input) {
                    if ov.allow_autonomous {
                        return GateDecision::Allow;
                    }
                }
                return GateDecision::Block {
                    reason: format!(
                        "irreversible action in autonomous mode without policy override: {}",
                        classification.reason
                    ),
                };
            }
            GateDecision::ConfirmationRequired {
                reason: format!("irreversible action: {}", classification.reason),
            }
        }
        RiskLevel::SharedState => {
            // Shared state always requires confirmation, regardless of mode.
            if policy.mode == ExecutionMode::Autonomous {
                if let Some(ov) = policy.lookup_override(tool_name, input) {
                    if ov.allow_autonomous {
                        return GateDecision::Allow;
                    }
                }
                return GateDecision::Block {
                    reason: format!(
                        "shared-state action in autonomous mode without policy override: {}",
                        classification.reason
                    ),
                };
            }
            GateDecision::ConfirmationRequired {
                reason: format!("shared-state action: {}", classification.reason),
            }
        }
    }
}

// ---------------------------------------------------------------------------
// DAIRA trace entry (extends evidence system with risk data)
// ---------------------------------------------------------------------------

/// A single DAIRA trace entry recording a tool invocation with risk metadata.
/// Extends the DAIRA adaptive context system (orchestrator/evidence.rs) with
/// per-action risk classification for post-hoc audit trails.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DairaTraceEntry {
    /// Tool name as dispatched.
    pub tool_name: String,
    /// Risk classification result.
    pub risk_level: RiskLevel,
    /// Human-readable reason for classification.
    pub reason: String,
    /// Gate decision (allow/confirm/block).
    pub gate_outcome: String,
    /// Timestamp (Unix epoch seconds).
    pub timestamp_secs: u64,
}

impl DairaTraceEntry {
    /// Create a trace entry from classification + gate decision.
    #[must_use]
    pub fn new(
        tool_name: &str,
        classification: &ActionClassification,
        gate: &GateDecision,
    ) -> Self {
        let gate_outcome = match gate {
            GateDecision::Allow => "allow",
            GateDecision::ConfirmationRequired { .. } => "confirmation_required",
            GateDecision::Block { .. } => "block",
        };
        Self {
            tool_name: tool_name.to_string(),
            risk_level: classification.risk,
            reason: classification.reason.clone(),
            gate_outcome: gate_outcome.to_string(),
            timestamp_secs: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
        }
    }
}

// ---------------------------------------------------------------------------
// Public entry point: classify + gate + trace
// ---------------------------------------------------------------------------

/// Classify a tool call, check the gate, and produce a trace entry.
///
/// Returns `(GateDecision, DairaTraceEntry)`. The caller must:
/// - If `GateDecision::Allow`: proceed with execution.
/// - If `GateDecision::ConfirmationRequired`: prompt user, block until confirmed.
/// - If `GateDecision::Block`: reject with the reason string.
///
/// The `DairaTraceEntry` should be appended to the session trace log
/// regardless of outcome.
#[must_use]
pub fn classify_and_gate(
    tool_name: &str,
    input: &serde_json::Value,
    policy: &ActionPolicy,
) -> (GateDecision, DairaTraceEntry) {
    let classification = classify(tool_name, input);
    let decision = check_gate(&classification, policy, tool_name, input);
    let trace = DairaTraceEntry::new(tool_name, &classification, &decision);
    (decision, trace)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // --- classify ---

    #[test]
    fn safe_tools_classified_correctly() {
        for tool in SAFE_TOOLS {
            let c = classify(tool, &json!({}));
            assert_eq!(c.risk, RiskLevel::Safe, "expected Safe for {tool}");
        }
    }

    #[test]
    fn mcp_think_is_safe() {
        let c = classify("mcp_think", &json!({"step_type": "observe"}));
        assert_eq!(c.risk, RiskLevel::Safe);
    }

    #[test]
    fn mcp_search_context_is_safe() {
        let c = classify("mcp_search_context", &json!({"query": "foo"}));
        assert_eq!(c.risk, RiskLevel::Safe);
    }

    #[test]
    fn edit_file_is_reversible() {
        let c = classify("edit_file", &json!({"path": "foo.rs"}));
        assert_eq!(c.risk, RiskLevel::Reversible);
    }

    #[test]
    fn write_file_is_reversible() {
        let c = classify("write_file", &json!({"path": "bar.rs"}));
        assert_eq!(c.risk, RiskLevel::Reversible);
    }

    #[test]
    fn cron_create_is_shared_state() {
        let c = classify("CronCreate", &json!({}));
        assert_eq!(c.risk, RiskLevel::SharedState);
    }

    #[test]
    fn unknown_tool_defaults_to_reversible() {
        let c = classify("SomeNewTool", &json!({}));
        assert_eq!(c.risk, RiskLevel::Reversible);
    }

    // --- bash sub-classification ---

    #[test]
    fn bash_git_push_is_irreversible() {
        let c = classify("bash", &json!({"command": "git push origin main"}));
        assert_eq!(c.risk, RiskLevel::Irreversible);
    }

    #[test]
    fn bash_git_status_is_safe() {
        let c = classify("bash", &json!({"command": "git status"}));
        assert_eq!(c.risk, RiskLevel::Safe);
    }

    #[test]
    fn bash_cargo_test_is_reversible() {
        let c = classify("bash", &json!({"command": "cargo test -p elai-tools"}));
        assert_eq!(c.risk, RiskLevel::Reversible);
    }

    #[test]
    fn bash_rm_is_irreversible() {
        let c = classify("bash", &json!({"command": "rm foo.rs"}));
        assert_eq!(c.risk, RiskLevel::Irreversible);
    }

    #[test]
    fn bash_ls_is_safe() {
        let c = classify("bash", &json!({"command": "ls -la"}));
        assert_eq!(c.risk, RiskLevel::Safe);
    }

    #[test]
    fn bash_gh_pr_create_is_shared_state() {
        let c = classify(
            "bash",
            &json!({"command": "gh pr create --title 'fix' --body 'stuff'"}),
        );
        assert_eq!(c.risk, RiskLevel::SharedState);
    }

    #[test]
    fn bash_git_reset_hard_is_irreversible() {
        let c = classify("bash", &json!({"command": "git reset --hard HEAD~1"}));
        assert_eq!(c.risk, RiskLevel::Irreversible);
    }

    #[test]
    fn bash_empty_command_is_safe() {
        let c = classify("bash", &json!({"command": ""}));
        assert_eq!(c.risk, RiskLevel::Safe);
    }

    #[test]
    fn bash_grep_is_safe_via_semantic() {
        let c = classify("bash", &json!({"command": "grep -rn 'TODO' src/"}));
        assert_eq!(c.risk, RiskLevel::Safe);
    }

    // --- gate decisions ---

    #[test]
    fn safe_action_always_allowed() {
        let policy = ActionPolicy::default();
        let class = ActionClassification {
            risk: RiskLevel::Safe,
            reason: "test".to_string(),
        };
        assert_eq!(
            check_gate(&class, &policy, "read_file", &json!({})),
            GateDecision::Allow
        );
    }

    #[test]
    fn irreversible_requires_confirmation_in_interactive() {
        let policy = ActionPolicy::default();
        let class = ActionClassification {
            risk: RiskLevel::Irreversible,
            reason: "git push".to_string(),
        };
        assert!(matches!(
            check_gate(&class, &policy, "bash", &json!({})),
            GateDecision::ConfirmationRequired { .. }
        ));
    }

    #[test]
    fn irreversible_blocked_in_autonomous_without_override() {
        let policy = ActionPolicy {
            mode: ExecutionMode::Autonomous,
            ..Default::default()
        };
        let class = ActionClassification {
            risk: RiskLevel::Irreversible,
            reason: "git push".to_string(),
        };
        assert!(matches!(
            check_gate(&class, &policy, "bash", &json!({})),
            GateDecision::Block { .. }
        ));
    }

    #[test]
    fn irreversible_allowed_in_autonomous_with_override() {
        let mut overrides = HashMap::new();
        overrides.insert(
            "bash".to_string(),
            PolicyOverride {
                risk_override: None,
                allow_autonomous: true,
            },
        );
        let policy = ActionPolicy {
            mode: ExecutionMode::Autonomous,
            tool_overrides: overrides,
            ..Default::default()
        };
        let class = ActionClassification {
            risk: RiskLevel::Irreversible,
            reason: "git push".to_string(),
        };
        assert_eq!(
            check_gate(&class, &policy, "bash", &json!({})),
            GateDecision::Allow
        );
    }

    #[test]
    fn shared_state_requires_confirmation_in_interactive() {
        let policy = ActionPolicy::default();
        let class = ActionClassification {
            risk: RiskLevel::SharedState,
            reason: "PR create".to_string(),
        };
        assert!(matches!(
            check_gate(&class, &policy, "CronCreate", &json!({})),
            GateDecision::ConfirmationRequired { .. }
        ));
    }

    #[test]
    fn pattern_override_matches_bash_command() {
        let mut patterns = HashMap::new();
        patterns.insert(
            "git push".to_string(),
            PolicyOverride {
                risk_override: Some(RiskLevel::Reversible),
                allow_autonomous: false,
            },
        );
        let policy = ActionPolicy {
            pattern_overrides: patterns,
            ..Default::default()
        };
        let class = ActionClassification {
            risk: RiskLevel::Irreversible,
            reason: "git push".to_string(),
        };
        let input = json!({"command": "git push origin main"});
        assert_eq!(
            check_gate(&class, &policy, "bash", &input),
            GateDecision::Allow
        );
    }

    // --- trace entry ---

    #[test]
    fn trace_entry_captures_fields() {
        let class = ActionClassification {
            risk: RiskLevel::Irreversible,
            reason: "git push".to_string(),
        };
        let gate = GateDecision::ConfirmationRequired {
            reason: "irreversible".to_string(),
        };
        let entry = DairaTraceEntry::new("bash", &class, &gate);
        assert_eq!(entry.tool_name, "bash");
        assert_eq!(entry.risk_level, RiskLevel::Irreversible);
        assert_eq!(entry.gate_outcome, "confirmation_required");
        assert!(entry.timestamp_secs > 0);
    }

    // --- classify_and_gate integration ---

    #[test]
    fn classify_and_gate_safe_tool() {
        let policy = ActionPolicy::default();
        let (decision, trace) = classify_and_gate("read_file", &json!({"path": "x"}), &policy);
        assert_eq!(decision, GateDecision::Allow);
        assert_eq!(trace.risk_level, RiskLevel::Safe);
    }

    #[test]
    fn classify_and_gate_irreversible_bash() {
        let policy = ActionPolicy::default();
        let (decision, trace) =
            classify_and_gate("bash", &json!({"command": "git push origin main"}), &policy);
        assert!(matches!(decision, GateDecision::ConfirmationRequired { .. }));
        assert_eq!(trace.risk_level, RiskLevel::Irreversible);
    }

    #[test]
    fn risk_level_ordering() {
        assert!(RiskLevel::Safe < RiskLevel::Reversible);
        assert!(RiskLevel::Reversible < RiskLevel::Irreversible);
        assert!(RiskLevel::Irreversible < RiskLevel::SharedState);
    }
}

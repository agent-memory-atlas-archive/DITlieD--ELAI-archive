//! Phase-aware tool/rule loading per FSM state.
//!
//! Phase 3A.2 deliverable. See docs/specs/crates/orchestrator.txt §2 (tool
//! restrictions per state), architecture-overview.txt §4 (Fsm trait:
//! get_active_tools, get_active_rules).
//!
//! Design: tools are classified into semantic categories. Each FSM state
//! declares which categories are ALLOWED. A tool not in any allowed category
//! is blocked with `FsmError::ToolBlockedByState`. Enforcement happens at the
//! `Fsm::check_tool_allowed` level — callers (ToolRegistry::execute, IPC
//! layer) call this before dispatching.

use crate::fsm::FsmState;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// ToolCategory — semantic classification of tools
// ---------------------------------------------------------------------------

/// Semantic category for tool filtering. Each tool name maps to exactly one
/// category. Source: orchestrator.txt §2 L36-43, phase-3A.txt L112-120.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ToolCategory {
    /// File read operations: read_file, glob_search, grep_search
    Read,
    /// File write/edit operations: write_file, edit_file
    Edit,
    /// Shell execution: bash, PowerShell
    Bash,
    /// Web access: WebFetch, WebSearch
    Web,
    /// Planning tools: EnterPlanMode, ExitPlanMode, Skill (plan-related)
    Planning,
    /// Test framework tools (test runner invocations via bash — category marker)
    Testing,
    /// Validation / audit tools (code inspection, test runner in audit mode)
    Validation,
    /// Memory-bank write tools: memory write operations
    Memory,
    /// Configuration tools: Config, config-path-only edits
    ConfigOnly,
    /// Meta/utility tools: TodoWrite, Sleep, SendUserMessage, AskUserQuestion,
    /// Agent, ToolSearch, NotebookEdit, REPL, LSP, StructuredOutput
    Meta,
    /// Task/worker/team/cron management tools
    TaskManagement,
    /// MCP tools: ListMcpResources, ReadMcpResource, McpAuth, MCP, RemoteTrigger
    Mcp,
    /// FSM internal tools: fsm_get_state, fsm_transition
    FsmInternal,
    /// Git operations (invoked via bash, but categorized for FEASIBILITY_PROBE restrictions)
    Git,
    /// Knowledge engine queries (used in PLANNING, blocked in EXECUTING/CONFIGURING)
    Knowledge,
    /// Stub write: limited write (<50 lines) for FEASIBILITY_PROBE
    StubWrite,
    /// Canvas op tools (H-13): model-driven canvas mutations via canvas_* tool catalog.
    Canvas,
}

impl ToolCategory {
    /// All categories for iteration.
    pub const ALL: [ToolCategory; 17] = [
        ToolCategory::Read,
        ToolCategory::Edit,
        ToolCategory::Bash,
        ToolCategory::Web,
        ToolCategory::Planning,
        ToolCategory::Testing,
        ToolCategory::Validation,
        ToolCategory::Memory,
        ToolCategory::ConfigOnly,
        ToolCategory::Meta,
        ToolCategory::TaskManagement,
        ToolCategory::Mcp,
        ToolCategory::FsmInternal,
        ToolCategory::Git,
        ToolCategory::Knowledge,
        ToolCategory::StubWrite,
        ToolCategory::Canvas,
    ];

    /// Classify a tool name into its category. Tool names are matched
    /// case-insensitively after normalizing dashes to underscores.
    ///
    /// Unknown tools default to `Meta` (allowed in IDLE, blocked in restricted
    /// states). This is conservative — unknown tools are only available in
    /// unrestricted states.
    pub fn classify(tool_name: &str) -> ToolCategory {
        let normalized = tool_name.replace('-', "_").to_ascii_lowercase();
        match normalized.as_str() {
            // Read
            "read_file" | "read" => ToolCategory::Read,
            "glob_search" | "glob" => ToolCategory::Read,
            "grep_search" | "grep" => ToolCategory::Read,

            // Edit
            "write_file" | "write" => ToolCategory::Edit,
            "edit_file" | "edit" => ToolCategory::Edit,

            // Bash
            "bash" => ToolCategory::Bash,
            "powershell" => ToolCategory::Bash,

            // Web
            "webfetch" | "web_fetch" => ToolCategory::Web,
            "websearch" | "web_search" => ToolCategory::Web,

            // Planning
            "enterplanmode" | "enter_plan_mode" => ToolCategory::Planning,
            "exitplanmode" | "exit_plan_mode" => ToolCategory::Planning,

            // Testing (test runner is invoked via bash, but we categorize the
            // semantic intent; actual enforcement uses bash category + testing flag)
            "testingpermission" | "testing_permission" => ToolCategory::Testing,

            // Config
            "config" => ToolCategory::ConfigOnly,

            // Memory (memory-bank write operations — tools TBD in later phases)
            // Currently no dedicated memory tool; placeholder for Phase 3A.2 spec compliance.

            // Meta / utility
            "todowrite" | "todo_write" => ToolCategory::Meta,
            "sleep" => ToolCategory::Meta,
            "senduserMessage" | "send_user_message" => ToolCategory::Meta,
            "askuserquestion" | "ask_user_question" => ToolCategory::Meta,
            "agent" => ToolCategory::Meta,
            "toolsearch" | "tool_search" => ToolCategory::Meta,
            "notebookedit" | "notebook_edit" => ToolCategory::Meta,
            "repl" => ToolCategory::Meta,
            "lsp" => ToolCategory::Meta,
            "structuredoutput" | "structured_output" => ToolCategory::Meta,
            "skill" => ToolCategory::Meta,
            "briefoutput" | "brief_output" | "brief" => ToolCategory::Meta,

            // Task management
            "taskcreate" | "task_create" => ToolCategory::TaskManagement,
            "runtaskpacket" | "run_task_packet" => ToolCategory::TaskManagement,
            "taskget" | "task_get" => ToolCategory::TaskManagement,
            "tasklist" | "task_list" => ToolCategory::TaskManagement,
            "taskstop" | "task_stop" => ToolCategory::TaskManagement,
            "taskupdate" | "task_update" => ToolCategory::TaskManagement,
            "taskoutput" | "task_output" => ToolCategory::TaskManagement,
            "workercreate" | "worker_create" => ToolCategory::TaskManagement,
            "workerget" | "worker_get" => ToolCategory::TaskManagement,
            "workerobserve" | "worker_observe" => ToolCategory::TaskManagement,
            "workerresolvetrust" | "worker_resolve_trust" => ToolCategory::TaskManagement,
            "workerawaitready" | "worker_await_ready" => ToolCategory::TaskManagement,
            "workersendprompt" | "worker_send_prompt" => ToolCategory::TaskManagement,
            "workerrestart" | "worker_restart" => ToolCategory::TaskManagement,
            "workerterminate" | "worker_terminate" => ToolCategory::TaskManagement,
            "teamcreate" | "team_create" => ToolCategory::TaskManagement,
            "teamdelete" | "team_delete" => ToolCategory::TaskManagement,
            "croncreate" | "cron_create" => ToolCategory::TaskManagement,
            "crondelete" | "cron_delete" => ToolCategory::TaskManagement,
            "cronlist" | "cron_list" => ToolCategory::TaskManagement,

            // MCP
            "listmcpresources" | "list_mcp_resources" => ToolCategory::Mcp,
            "readmcpresource" | "read_mcp_resource" => ToolCategory::Mcp,
            "mcpauth" | "mcp_auth" => ToolCategory::Mcp,
            "mcp" => ToolCategory::Mcp,
            "remotetrigger" | "remote_trigger" => ToolCategory::Mcp,

            // Knowledge engine
            "knowledgeengine" | "knowledge_engine" => ToolCategory::Knowledge,

            // Canvas ops (H-13): model-driven canvas mutations
            "canvas_create_node" => ToolCategory::Canvas,
            "canvas_connect" => ToolCategory::Canvas,
            "canvas_update_node" => ToolCategory::Canvas,
            "canvas_annotate" => ToolCategory::Canvas,
            "canvas_fork" => ToolCategory::Canvas,
            "canvas_render_diagram" => ToolCategory::Canvas,
            "canvas_screenshot" => ToolCategory::Canvas,
            // H-15: spawn_view reply-channel verb.
            "canvas_spawn_view" => ToolCategory::Canvas,

            // FSM internal
            "fsm_get_state" => ToolCategory::FsmInternal,
            "fsm_transition" => ToolCategory::FsmInternal,
            "list_hidden_paths" => ToolCategory::FsmInternal,

            // Default: unknown tools classified as Meta
            _ => {
                // MCP-prefixed tools (mcp__*) are classified as Mcp
                if normalized.starts_with("mcp__") {
                    return ToolCategory::Mcp;
                }
                ToolCategory::Meta
            }
        }
    }
}

// ---------------------------------------------------------------------------
// RuleCategory — semantic classification of rules
// ---------------------------------------------------------------------------

/// Semantic category for rule filtering. Source: orchestrator.txt §2,
/// architecture-overview.txt §4 (get_active_rules).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RuleCategory {
    /// Planning rules: knowledge queries, design constraints
    Planning,
    /// Code quality rules: lint, style, naming conventions
    CodeQuality,
    /// Testing rules: test coverage, test naming, test isolation
    Testing,
    /// Validation rules: audit checks, security review
    Validation,
    /// Memory rules: memory-bank format, changelog format
    Memory,
    /// Configuration rules: config file format, path restrictions
    Config,
}

impl RuleCategory {
    /// All categories for iteration.
    pub const ALL: [RuleCategory; 6] = [
        RuleCategory::Planning,
        RuleCategory::CodeQuality,
        RuleCategory::Testing,
        RuleCategory::Validation,
        RuleCategory::Memory,
        RuleCategory::Config,
    ];
}

// ---------------------------------------------------------------------------
// Phase-aware tool filter: allowed categories per state
// ---------------------------------------------------------------------------

impl FsmState {
    /// Tool categories allowed in this state. Source: orchestrator.txt §2 L36-43,
    /// phase-3A.txt L112-120.
    ///
    /// Returns the set of `ToolCategory` values permitted. Any tool whose
    /// category is NOT in this set is blocked with `ToolBlockedByState`.
    pub fn allowed_tool_categories(self) -> &'static [ToolCategory] {
        match self {
            // IDLE: all tools, no restrictions.
            // Source: phase-3A.txt L120 (UNGROUNDED but assumed — default unrestricted).
            FsmState::Idle => &ToolCategory::ALL,

            // PLANNING: rules + knowledge results + plan template. NO edit/bash tools.
            // Source: orchestrator.txt §2 L37.
            FsmState::Planning => &[
                ToolCategory::Read,
                ToolCategory::Web,
                ToolCategory::Planning,
                ToolCategory::Knowledge,
                ToolCategory::Meta,
                ToolCategory::FsmInternal,
                ToolCategory::Mcp,
            ],

            // FEASIBILITY_PROBE: stub write (<50 lines) + bash + web search. NO full edit/git.
            // Source: orchestrator.txt §2 L38.
            FsmState::FeasibilityProbe => &[
                ToolCategory::Read,
                ToolCategory::StubWrite,
                ToolCategory::Bash,
                ToolCategory::Web,
                ToolCategory::Meta,
                ToolCategory::FsmInternal,
            ],

            // TESTING_FIRST: test framework + plan + rules. NO edit/bash tools.
            // Source: orchestrator.txt §2 L39.
            FsmState::TestingFirst => &[
                ToolCategory::Read,
                ToolCategory::Planning,
                ToolCategory::Testing,
                ToolCategory::Meta,
                ToolCategory::FsmInternal,
            ],

            // EXECUTING: edit tools + current phase spec + code context. NO planning tools.
            // Source: orchestrator.txt §2 L40.
            FsmState::Executing => &[
                ToolCategory::Read,
                ToolCategory::Edit,
                ToolCategory::Bash,
                ToolCategory::Web,
                ToolCategory::Testing,
                ToolCategory::Git,
                ToolCategory::Meta,
                ToolCategory::FsmInternal,
                ToolCategory::TaskManagement,
                ToolCategory::Mcp,
            ],

            // AUDITING: validation tools + test runner + code. NO edit/planning tools.
            // Source: orchestrator.txt §2 L41.
            FsmState::Auditing => &[
                ToolCategory::Read,
                ToolCategory::Bash, // test runner invocation
                ToolCategory::Validation,
                ToolCategory::Testing,
                ToolCategory::Meta,
                ToolCategory::FsmInternal,
            ],

            // MEMORIZING: memory-bank write tools only.
            // Source: orchestrator.txt §2 L42.
            FsmState::Memorizing => &[
                ToolCategory::Read,
                ToolCategory::Memory,
                ToolCategory::Meta,
                ToolCategory::FsmInternal,
            ],

            // COMPLETE: read-only tools only. Terminal state, inspection only before IDLE.
            // Source: phase-3A.txt L121 (UNGROUNDED — assumed from state semantics).
            FsmState::Complete => &[
                ToolCategory::Read,
                ToolCategory::Meta,
                ToolCategory::FsmInternal,
            ],

            // CONFIGURING: config paths only. NO code edit/bash tools.
            // Source: technical-analysis-v3.txt Concern 5, phase-3A.txt L118.
            FsmState::Configuring => &[
                ToolCategory::Read,
                ToolCategory::ConfigOnly,
                ToolCategory::Meta,
                ToolCategory::FsmInternal,
            ],

            // RESEARCHING: WebSearch + KnowledgeEngine query + GRC reasoning.
            // Blocks: file edit, bash, git, all mutation tools.
            // Source: pipeline-modes-design.txt §2 (FSM PATH), INTEGRATION §2.
            FsmState::Researching => &[
                ToolCategory::Read,
                ToolCategory::Web,
                ToolCategory::Knowledge,
                ToolCategory::Meta,
                ToolCategory::FsmInternal,
                ToolCategory::Mcp,
            ],
        }
    }

    /// Rule categories active in this state. Source: orchestrator.txt §2,
    /// architecture-overview.txt §4 (get_active_rules).
    pub fn active_rule_categories(self) -> &'static [RuleCategory] {
        match self {
            // IDLE: all rules (unrestricted).
            FsmState::Idle => &RuleCategory::ALL,

            // PLANNING: planning + code quality rules.
            FsmState::Planning => &[RuleCategory::Planning, RuleCategory::CodeQuality],

            // FEASIBILITY_PROBE: planning rules only (lightweight).
            FsmState::FeasibilityProbe => &[RuleCategory::Planning],

            // TESTING_FIRST: testing + code quality rules.
            FsmState::TestingFirst => &[RuleCategory::Testing, RuleCategory::CodeQuality],

            // EXECUTING: code quality + testing rules (active development).
            FsmState::Executing => &[RuleCategory::CodeQuality, RuleCategory::Testing],

            // AUDITING: validation + testing rules.
            FsmState::Auditing => &[RuleCategory::Validation, RuleCategory::Testing],

            // MEMORIZING: memory rules only.
            FsmState::Memorizing => &[RuleCategory::Memory],

            // COMPLETE: validation rules (final check).
            FsmState::Complete => &[RuleCategory::Validation],

            // CONFIGURING: config rules only.
            FsmState::Configuring => &[RuleCategory::Config],

            // RESEARCHING: planning rules (research informs planning decisions).
            FsmState::Researching => &[RuleCategory::Planning],
        }
    }

    /// Check whether a tool name is allowed in this state.
    /// Returns `Ok(())` if allowed, `Err(category)` with the tool's category if blocked.
    pub fn check_tool_allowed(self, tool_name: &str) -> Result<(), ToolCategory> {
        let category = ToolCategory::classify(tool_name);
        if self.allowed_tool_categories().contains(&category) {
            Ok(())
        } else {
            Err(category)
        }
    }

    /// Check whether a rule category is active in this state.
    pub fn is_rule_active(self, category: RuleCategory) -> bool {
        self.active_rule_categories().contains(&category)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // Tool classification tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_classify_known_tools() {
        assert_eq!(ToolCategory::classify("read_file"), ToolCategory::Read);
        assert_eq!(ToolCategory::classify("glob_search"), ToolCategory::Read);
        assert_eq!(ToolCategory::classify("grep_search"), ToolCategory::Read);
        assert_eq!(ToolCategory::classify("write_file"), ToolCategory::Edit);
        assert_eq!(ToolCategory::classify("edit_file"), ToolCategory::Edit);
        assert_eq!(ToolCategory::classify("bash"), ToolCategory::Bash);
        assert_eq!(ToolCategory::classify("PowerShell"), ToolCategory::Bash);
        assert_eq!(ToolCategory::classify("WebFetch"), ToolCategory::Web);
        assert_eq!(ToolCategory::classify("WebSearch"), ToolCategory::Web);
        assert_eq!(ToolCategory::classify("EnterPlanMode"), ToolCategory::Planning);
        assert_eq!(ToolCategory::classify("ExitPlanMode"), ToolCategory::Planning);
        assert_eq!(ToolCategory::classify("Config"), ToolCategory::ConfigOnly);
        assert_eq!(ToolCategory::classify("TodoWrite"), ToolCategory::Meta);
        assert_eq!(ToolCategory::classify("Agent"), ToolCategory::Meta);
        assert_eq!(ToolCategory::classify("fsm_get_state"), ToolCategory::FsmInternal);
        assert_eq!(ToolCategory::classify("fsm_transition"), ToolCategory::FsmInternal);
    }

    #[test]
    fn test_classify_mcp_prefixed() {
        assert_eq!(
            ToolCategory::classify("mcp__local_rag__search_context"),
            ToolCategory::Mcp
        );
        assert_eq!(
            ToolCategory::classify("mcp__claude_ai_gmail__authenticate"),
            ToolCategory::Mcp
        );
    }

    #[test]
    fn test_classify_unknown_defaults_to_meta() {
        assert_eq!(ToolCategory::classify("nonexistent_tool"), ToolCategory::Meta);
    }

    #[test]
    fn test_classify_case_insensitive() {
        assert_eq!(ToolCategory::classify("BASH"), ToolCategory::Bash);
        assert_eq!(ToolCategory::classify("Read_File"), ToolCategory::Read);
        assert_eq!(ToolCategory::classify("WEBFETCH"), ToolCategory::Web);
    }

    #[test]
    fn test_classify_dash_normalization() {
        assert_eq!(ToolCategory::classify("read-file"), ToolCategory::Read);
        assert_eq!(ToolCategory::classify("edit-file"), ToolCategory::Edit);
        assert_eq!(ToolCategory::classify("web-fetch"), ToolCategory::Web);
    }

    // -----------------------------------------------------------------------
    // Per-state tool filter tests (spec: one test per state)
    // -----------------------------------------------------------------------

    #[test]
    fn test_idle_allows_all_tools() {
        let allowed = FsmState::Idle.allowed_tool_categories();
        // IDLE allows everything — all categories present
        for cat in ToolCategory::ALL {
            assert!(
                allowed.contains(&cat),
                "IDLE should allow {cat:?}"
            );
        }
    }

    #[test]
    fn test_planning_blocks_edit() {
        // Source: orchestrator.txt §2 L37 — PLANNING: NO edit/bash tools.
        assert!(FsmState::Planning.check_tool_allowed("edit_file").is_err());
        assert!(FsmState::Planning.check_tool_allowed("write_file").is_err());
        assert!(FsmState::Planning.check_tool_allowed("bash").is_err());
        // But allows: read, web, planning, knowledge, meta
        assert!(FsmState::Planning.check_tool_allowed("read_file").is_ok());
        assert!(FsmState::Planning.check_tool_allowed("WebSearch").is_ok());
        assert!(FsmState::Planning.check_tool_allowed("EnterPlanMode").is_ok());
    }

    #[test]
    fn test_feasibility_probe_blocks_full_edit() {
        // Source: orchestrator.txt §2 L38 — NO full edit/git.
        assert!(FsmState::FeasibilityProbe.check_tool_allowed("edit_file").is_err());
        assert!(FsmState::FeasibilityProbe.check_tool_allowed("write_file").is_err());
        // But allows: bash, web, read, stub_write
        assert!(FsmState::FeasibilityProbe.check_tool_allowed("bash").is_ok());
        assert!(FsmState::FeasibilityProbe.check_tool_allowed("WebSearch").is_ok());
        assert!(FsmState::FeasibilityProbe.check_tool_allowed("read_file").is_ok());
    }

    #[test]
    fn test_testing_first_blocks_edit_bash() {
        // Source: orchestrator.txt §2 L39 — NO edit/bash tools.
        assert!(FsmState::TestingFirst.check_tool_allowed("edit_file").is_err());
        assert!(FsmState::TestingFirst.check_tool_allowed("write_file").is_err());
        assert!(FsmState::TestingFirst.check_tool_allowed("bash").is_err());
        // But allows: read, planning, testing
        assert!(FsmState::TestingFirst.check_tool_allowed("read_file").is_ok());
        assert!(FsmState::TestingFirst.check_tool_allowed("EnterPlanMode").is_ok());
    }

    #[test]
    fn test_executing_blocks_planning() {
        // Source: orchestrator.txt §2 L40 — NO planning tools.
        assert!(FsmState::Executing.check_tool_allowed("EnterPlanMode").is_err());
        assert!(FsmState::Executing.check_tool_allowed("ExitPlanMode").is_err());
        // But allows: edit, bash, read, web, testing, git
        assert!(FsmState::Executing.check_tool_allowed("edit_file").is_ok());
        assert!(FsmState::Executing.check_tool_allowed("bash").is_ok());
        assert!(FsmState::Executing.check_tool_allowed("write_file").is_ok());
        assert!(FsmState::Executing.check_tool_allowed("read_file").is_ok());
    }

    #[test]
    fn test_auditing_blocks_edit_planning() {
        // Source: orchestrator.txt §2 L41 — NO edit/planning tools.
        assert!(FsmState::Auditing.check_tool_allowed("edit_file").is_err());
        assert!(FsmState::Auditing.check_tool_allowed("write_file").is_err());
        assert!(FsmState::Auditing.check_tool_allowed("EnterPlanMode").is_err());
        // But allows: read, bash (test runner), validation, testing
        assert!(FsmState::Auditing.check_tool_allowed("read_file").is_ok());
        assert!(FsmState::Auditing.check_tool_allowed("bash").is_ok());
    }

    #[test]
    fn test_memorizing_memory_only() {
        // Source: orchestrator.txt §2 L42 — memory-bank write tools only.
        assert!(FsmState::Memorizing.check_tool_allowed("edit_file").is_err());
        assert!(FsmState::Memorizing.check_tool_allowed("bash").is_err());
        assert!(FsmState::Memorizing.check_tool_allowed("EnterPlanMode").is_err());
        assert!(FsmState::Memorizing.check_tool_allowed("WebSearch").is_err());
        // But allows: read, meta, fsm_internal
        assert!(FsmState::Memorizing.check_tool_allowed("read_file").is_ok());
        assert!(FsmState::Memorizing.check_tool_allowed("TodoWrite").is_ok());
        assert!(FsmState::Memorizing.check_tool_allowed("fsm_get_state").is_ok());
    }

    #[test]
    fn test_complete_read_only() {
        // Source: phase-3A.txt L121 — read-only tools only.
        assert!(FsmState::Complete.check_tool_allowed("edit_file").is_err());
        assert!(FsmState::Complete.check_tool_allowed("write_file").is_err());
        assert!(FsmState::Complete.check_tool_allowed("bash").is_err());
        assert!(FsmState::Complete.check_tool_allowed("EnterPlanMode").is_err());
        // But allows: read, meta, fsm_internal
        assert!(FsmState::Complete.check_tool_allowed("read_file").is_ok());
        assert!(FsmState::Complete.check_tool_allowed("glob_search").is_ok());
    }

    #[test]
    fn test_configuring_config_only() {
        // Source: technical-analysis-v3.txt Concern 5 — config paths only.
        assert!(FsmState::Configuring.check_tool_allowed("edit_file").is_err());
        assert!(FsmState::Configuring.check_tool_allowed("bash").is_err());
        assert!(FsmState::Configuring.check_tool_allowed("write_file").is_err());
        assert!(FsmState::Configuring.check_tool_allowed("WebSearch").is_err());
        // But allows: Config, read, meta, fsm_internal
        assert!(FsmState::Configuring.check_tool_allowed("Config").is_ok());
        assert!(FsmState::Configuring.check_tool_allowed("read_file").is_ok());
    }

    #[test]
    fn test_researching_allows_web_knowledge_blocks_edit_bash() {
        // Source: pipeline-modes-design.txt §2 — RESEARCHING: WebSearch + Knowledge,
        // blocks file edit, bash, git, all mutation tools.
        assert!(FsmState::Researching.check_tool_allowed("WebSearch").is_ok());
        assert!(FsmState::Researching.check_tool_allowed("WebFetch").is_ok());
        assert!(FsmState::Researching.check_tool_allowed("read_file").is_ok());
        assert!(FsmState::Researching.check_tool_allowed("mcp__local_rag__search_context").is_ok());
        // Blocks
        assert!(FsmState::Researching.check_tool_allowed("edit_file").is_err());
        assert!(FsmState::Researching.check_tool_allowed("write_file").is_err());
        assert!(FsmState::Researching.check_tool_allowed("bash").is_err());
        assert!(FsmState::Researching.check_tool_allowed("EnterPlanMode").is_err());
    }

    #[test]
    fn test_researching_rules() {
        let active = FsmState::Researching.active_rule_categories();
        assert!(active.contains(&RuleCategory::Planning));
        assert!(!active.contains(&RuleCategory::CodeQuality));
        assert!(!active.contains(&RuleCategory::Testing));
        assert!(!active.contains(&RuleCategory::Memory));
    }

    // -----------------------------------------------------------------------
    // Adversarial: tool filter enforcement (spec: phase-3A.txt L132-136)
    // -----------------------------------------------------------------------

    #[test]
    fn test_planning_blocks_file_edit_enforcement() {
        // Adversarial: if tool filter is advisory, agent bypasses.
        // Test: in PLANNING state, file_edit must be blocked.
        let result = FsmState::Planning.check_tool_allowed("edit_file");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ToolCategory::Edit);
    }

    #[test]
    fn test_idle_allows_all_tools_adversarial() {
        // Adversarial: if IDLE accidentally blocks tools, walking skeleton breaks.
        // Test: in IDLE, all known tools are allowed.
        let tools = [
            "bash", "read_file", "write_file", "edit_file", "glob_search",
            "grep_search", "WebFetch", "WebSearch", "TodoWrite", "Agent",
            "EnterPlanMode", "ExitPlanMode", "Config", "fsm_get_state",
        ];
        for tool in tools {
            assert!(
                FsmState::Idle.check_tool_allowed(tool).is_ok(),
                "IDLE should allow {tool}"
            );
        }
    }

    // -----------------------------------------------------------------------
    // Per-state rule filter tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_idle_all_rules() {
        let active = FsmState::Idle.active_rule_categories();
        for cat in RuleCategory::ALL {
            assert!(active.contains(&cat), "IDLE should have all rules active");
        }
    }

    #[test]
    fn test_planning_rules() {
        let active = FsmState::Planning.active_rule_categories();
        assert!(active.contains(&RuleCategory::Planning));
        assert!(active.contains(&RuleCategory::CodeQuality));
        assert!(!active.contains(&RuleCategory::Testing));
        assert!(!active.contains(&RuleCategory::Memory));
    }

    #[test]
    fn test_executing_rules() {
        let active = FsmState::Executing.active_rule_categories();
        assert!(active.contains(&RuleCategory::CodeQuality));
        assert!(active.contains(&RuleCategory::Testing));
        assert!(!active.contains(&RuleCategory::Planning));
        assert!(!active.contains(&RuleCategory::Memory));
    }

    #[test]
    fn test_auditing_rules() {
        let active = FsmState::Auditing.active_rule_categories();
        assert!(active.contains(&RuleCategory::Validation));
        assert!(active.contains(&RuleCategory::Testing));
        assert!(!active.contains(&RuleCategory::Planning));
    }

    #[test]
    fn test_memorizing_rules() {
        let active = FsmState::Memorizing.active_rule_categories();
        assert!(active.contains(&RuleCategory::Memory));
        assert!(!active.contains(&RuleCategory::Planning));
        assert!(!active.contains(&RuleCategory::CodeQuality));
    }

    #[test]
    fn test_configuring_rules() {
        let active = FsmState::Configuring.active_rule_categories();
        assert!(active.contains(&RuleCategory::Config));
        assert!(!active.contains(&RuleCategory::CodeQuality));
    }

    // -----------------------------------------------------------------------
    // Property: tool filter for any state is always a subset of all categories
    // -----------------------------------------------------------------------

    #[test]
    fn test_all_states_tool_filter_is_subset() {
        for state in FsmState::ALL {
            let allowed = state.allowed_tool_categories();
            for cat in allowed {
                assert!(
                    ToolCategory::ALL.contains(cat),
                    "{state}: {cat:?} not in ToolCategory::ALL"
                );
            }
        }
    }

    #[test]
    fn test_all_states_rule_filter_is_subset() {
        for state in FsmState::ALL {
            let active = state.active_rule_categories();
            for cat in active {
                assert!(
                    RuleCategory::ALL.contains(cat),
                    "{state}: {cat:?} not in RuleCategory::ALL"
                );
            }
        }
    }

    // -----------------------------------------------------------------------
    // Property: at least one tool blocked and one allowed per restricted state
    // (spec: phase-3A.txt L129)
    // -----------------------------------------------------------------------

    #[test]
    fn test_restricted_states_have_blocked_and_allowed() {
        // All states except IDLE should have at least one blocked category.
        let restricted = [
            FsmState::Planning,
            FsmState::FeasibilityProbe,
            FsmState::TestingFirst,
            FsmState::Executing,
            FsmState::Auditing,
            FsmState::Memorizing,
            FsmState::Complete,
            FsmState::Configuring,
            FsmState::Researching,
        ];
        for state in restricted {
            let allowed = state.allowed_tool_categories();
            assert!(
                !allowed.is_empty(),
                "{state}: must allow at least one category"
            );
            assert!(
                allowed.len() < ToolCategory::ALL.len(),
                "{state}: must block at least one category"
            );
        }
    }

    // -----------------------------------------------------------------------
    // Specific blocked category assertions per state
    // -----------------------------------------------------------------------

    #[test]
    fn test_planning_specifically_blocks_edit_bash_git() {
        let allowed = FsmState::Planning.allowed_tool_categories();
        assert!(!allowed.contains(&ToolCategory::Edit));
        assert!(!allowed.contains(&ToolCategory::Bash));
        assert!(!allowed.contains(&ToolCategory::Git));
    }

    #[test]
    fn test_executing_specifically_blocks_planning_knowledge() {
        let allowed = FsmState::Executing.allowed_tool_categories();
        assert!(!allowed.contains(&ToolCategory::Planning));
        assert!(!allowed.contains(&ToolCategory::Knowledge));
    }

    #[test]
    fn test_auditing_specifically_blocks_edit_planning() {
        let allowed = FsmState::Auditing.allowed_tool_categories();
        assert!(!allowed.contains(&ToolCategory::Edit));
        assert!(!allowed.contains(&ToolCategory::Planning));
        assert!(!allowed.contains(&ToolCategory::Knowledge));
    }

    // -----------------------------------------------------------------------
    // Fsm engine integration: check_tool_allowed via Fsm struct
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn test_fsm_check_tool_allowed() {
        let fsm = crate::fsm::Fsm::new(None, 8000);
        // IDLE: all tools allowed
        assert!(fsm.check_tool_allowed("bash").await.is_ok());
        assert!(fsm.check_tool_allowed("edit_file").await.is_ok());

        // Move to PLANNING
        fsm.transition(FsmState::Planning, "test", vec![]).await.unwrap();
        assert!(fsm.check_tool_allowed("read_file").await.is_ok());
        assert!(fsm.check_tool_allowed("edit_file").await.is_err());
        assert!(fsm.check_tool_allowed("bash").await.is_err());

        // Move to TESTING_FIRST
        fsm.transition(FsmState::TestingFirst, "test", vec![]).await.unwrap();
        assert!(fsm.check_tool_allowed("read_file").await.is_ok());
        assert!(fsm.check_tool_allowed("edit_file").await.is_err());
        assert!(fsm.check_tool_allowed("bash").await.is_err());

        // Move to EXECUTING
        fsm.transition(FsmState::Executing, "test", vec![]).await.unwrap();
        assert!(fsm.check_tool_allowed("edit_file").await.is_ok());
        assert!(fsm.check_tool_allowed("bash").await.is_ok());
        assert!(fsm.check_tool_allowed("EnterPlanMode").await.is_err());

        // Reset and test RESEARCHING
        fsm.reset().await.unwrap();
        fsm.transition(FsmState::Researching, "test", vec![]).await.unwrap();
        assert!(fsm.check_tool_allowed("WebSearch").await.is_ok());
        assert!(fsm.check_tool_allowed("read_file").await.is_ok());
        assert!(fsm.check_tool_allowed("edit_file").await.is_err());
        assert!(fsm.check_tool_allowed("bash").await.is_err());
    }

    #[tokio::test]
    async fn test_fsm_check_tool_returns_typed_error() {
        let fsm = crate::fsm::Fsm::new(None, 8000);
        fsm.transition(FsmState::Planning, "test", vec![]).await.unwrap();

        let err = fsm.check_tool_allowed("edit_file").await;
        match err {
            Err(crate::fsm::FsmError::ToolBlockedByState { tool_name, state, category }) => {
                assert_eq!(tool_name, "edit_file");
                assert_eq!(state, FsmState::Planning);
                assert_eq!(category, ToolCategory::Edit);
            }
            other => panic!("expected ToolBlockedByState, got: {other:?}"),
        }
    }
}

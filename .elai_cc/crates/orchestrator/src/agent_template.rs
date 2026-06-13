//! Agent prompt template — structured briefing for sub-agents (A-29).
//!
//! When the model spawns a sub-agent it provides only a raw task string and
//! optional agent type.  The orchestrator wraps that in a four-section template
//! (CONTEXT / TASK / CONSTRAINTS / OUTPUT) so every sub-agent gets a consistent
//! briefing regardless of how the model phrases the delegation.

use std::fmt;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::{Deserialize, Serialize};

use crate::context_budget::ModelTier;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Default max output length (characters) when no per-type override is set.
const DEFAULT_MAX_OUTPUT_CHARS: usize = 4_000;

/// Hard ceiling — no sub-agent output may exceed this.
const ABSOLUTE_MAX_OUTPUT_CHARS: usize = 16_000;

/// Default max context tokens forwarded to a sub-agent.
const DEFAULT_MAX_CONTEXT_TOKENS: usize = 25_000;

// ---------------------------------------------------------------------------
// AgentOutputFormat — how the sub-agent should structure its response
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentOutputFormat {
    /// Free-form prose (default for general-purpose).
    Prose,
    /// Bullet-point findings (default for Explore, Verification).
    BulletFindings,
    /// Structured JSON blob.
    Json,
    /// Diff / patch format.
    Patch,
}

impl fmt::Display for AgentOutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Prose => f.write_str("prose"),
            Self::BulletFindings => f.write_str("bullet findings"),
            Self::Json => f.write_str("JSON"),
            Self::Patch => f.write_str("patch/diff"),
        }
    }
}

// ---------------------------------------------------------------------------
// AgentTypeConfig — per-subagent-type output constraints
// ---------------------------------------------------------------------------

/// Per-agent-type configuration controlling output format and length.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentTypeConfig {
    pub max_output_chars: usize,
    pub output_format: AgentOutputFormat,
}

impl AgentTypeConfig {
    /// Returns sensible defaults based on agent type name.
    #[must_use]
    pub fn defaults_for(agent_type: &str) -> Self {
        match agent_type {
            "Explore" => Self {
                max_output_chars: 3_000,
                output_format: AgentOutputFormat::BulletFindings,
            },
            "Plan" => Self {
                max_output_chars: 6_000,
                output_format: AgentOutputFormat::Prose,
            },
            "Verification" => Self {
                max_output_chars: 4_000,
                output_format: AgentOutputFormat::BulletFindings,
            },
            "code-reviewer" => Self {
                max_output_chars: 5_000,
                output_format: AgentOutputFormat::BulletFindings,
            },
            _ => Self {
                max_output_chars: DEFAULT_MAX_OUTPUT_CHARS,
                output_format: AgentOutputFormat::Prose,
            },
        }
    }
}

// ---------------------------------------------------------------------------
// AgentTemplateConfig — top-level [agents.template] configuration
// ---------------------------------------------------------------------------

/// Configurable template settings, deserializable from `[agents.template]` in
/// a TOML config file.  All fields have sensible defaults.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct AgentTemplateConfig {
    /// Global max output chars (overridden per-type if set).
    pub default_max_output_chars: usize,
    /// Global max context tokens forwarded to sub-agents.
    pub default_max_context_tokens: usize,
    /// Additional constraint lines injected into every agent prompt.
    pub extra_constraints: Vec<String>,
}

impl Default for AgentTemplateConfig {
    fn default() -> Self {
        Self {
            default_max_output_chars: DEFAULT_MAX_OUTPUT_CHARS,
            default_max_context_tokens: DEFAULT_MAX_CONTEXT_TOKENS,
            extra_constraints: Vec::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// AgentPromptContext — runtime context injected into the template
// ---------------------------------------------------------------------------

/// Everything the orchestrator knows at sub-agent spawn time.  Assembled from
/// session state, SSB entries, and the model capability profile.
#[derive(Debug, Clone)]
pub struct AgentPromptContext {
    pub cwd: PathBuf,
    pub git_branch: Option<String>,
    pub active_files: Vec<String>,
    pub model_tier: ModelTier,
    pub model_name: String,
    pub max_output_tokens: usize,
    pub max_context_tokens: usize,
    pub active_rules: Vec<String>,
}

impl AgentPromptContext {
    /// Build a context from the current working directory, auto-detecting git branch.
    #[must_use]
    pub fn from_cwd(cwd: impl Into<PathBuf>, model_tier: ModelTier, model_name: impl Into<String>) -> Self {
        let cwd = cwd.into();
        let git_branch = read_git_branch(&cwd);
        Self {
            cwd,
            git_branch,
            active_files: Vec::new(),
            model_tier,
            model_name: model_name.into(),
            max_output_tokens: DEFAULT_MAX_CONTEXT_TOKENS,
            max_context_tokens: DEFAULT_MAX_CONTEXT_TOKENS,
            active_rules: Vec::new(),
        }
    }

    /// Override active files (from SSB or recent tool calls).
    #[must_use]
    pub fn with_active_files(mut self, files: Vec<String>) -> Self {
        self.active_files = files;
        self
    }

    /// Override active rules from rules.db or policy engine.
    #[must_use]
    pub fn with_active_rules(mut self, rules: Vec<String>) -> Self {
        self.active_rules = rules;
        self
    }

    /// Override max output/context tokens.
    #[must_use]
    pub fn with_token_limits(mut self, max_output: usize, max_context: usize) -> Self {
        self.max_output_tokens = max_output;
        self.max_context_tokens = max_context;
        self
    }
}

// ---------------------------------------------------------------------------
// build_agent_prompt — the main entry point
// ---------------------------------------------------------------------------

/// Render the four-section agent prompt template.
///
/// Returns a single `String` containing the full structured prompt that replaces
/// both the old system prompt and the raw task string.
///
/// # Sections
///
/// 1. **CONTEXT** — cwd, git branch, active files, model tier
/// 2. **TASK** — the model's raw task string, unmodified
/// 3. **CONSTRAINTS** — active rules, model tier limits, standard rules
/// 4. **OUTPUT** — expected format and length cap
#[must_use]
pub fn build_agent_prompt(
    task: &str,
    agent_type: &str,
    context: &AgentPromptContext,
    config: &AgentTemplateConfig,
) -> String {
    let type_config = AgentTypeConfig::defaults_for(agent_type);
    let max_output = effective_max_output(type_config.max_output_chars, config);

    let mut sections = Vec::with_capacity(4);

    // === CONTEXT ===
    sections.push(render_context_section(context, agent_type));

    // === TASK ===
    sections.push(render_task_section(task));

    // === CONSTRAINTS ===
    sections.push(render_constraints_section(context, config));

    // === OUTPUT ===
    sections.push(render_output_section(max_output, &type_config.output_format));

    sections.join("\n\n")
}

/// Render just the system-prompt preamble for the sub-agent (identity line).
/// This is the minimal system prompt; the structured template goes as the
/// user message / first turn.
#[must_use]
pub fn build_agent_system_preamble(agent_type: &str) -> String {
    format!(
        include_str!("prompts/agent_preamble.md"),
        agent_type = agent_type,
    )
}

// ---------------------------------------------------------------------------
// Section renderers
// ---------------------------------------------------------------------------

fn render_context_section(context: &AgentPromptContext, agent_type: &str) -> String {
    let mut lines = vec![
        "=== CONTEXT ===".to_string(),
        format!("Working directory: {}", context.cwd.display()),
    ];

    if let Some(branch) = &context.git_branch {
        lines.push(format!("Git branch: {branch}"));
    }

    if !context.active_files.is_empty() {
        let file_list = if context.active_files.len() <= 10 {
            context.active_files.join(", ")
        } else {
            let mut preview: Vec<&str> = context.active_files.iter().take(10).map(String::as_str).collect();
            preview.push("...");
            format!("{} ({} total)", preview.join(", "), context.active_files.len())
        };
        lines.push(format!("Active files: {file_list}"));
    }

    lines.push(format!("Model tier: {:?}", context.model_tier));
    lines.push(format!("Agent type: {agent_type}"));

    lines.join("\n")
}

fn render_task_section(task: &str) -> String {
    format!("=== TASK ===\n{}", task.trim())
}

fn render_constraints_section(
    context: &AgentPromptContext,
    config: &AgentTemplateConfig,
) -> String {
    let mut lines = vec!["=== CONSTRAINTS ===".to_string()];

    // Active rules from rules.db / policy engine.
    for rule in &context.active_rules {
        lines.push(format!("- {rule}"));
    }

    // Model tier limits.
    lines.push(format!(
        "- Model: {} (tier: {:?})",
        context.model_name, context.model_tier
    ));
    lines.push(format!(
        "- Max context: {} tokens",
        context.max_context_tokens
    ));

    // Standard sub-agent rules (always applied).
    lines.push("- Output: concise, no emojis, absolute paths only.".to_string());
    lines.push("- Report back: only load-bearing findings, not raw tool output.".to_string());
    lines.push("- Do not ask the user questions.".to_string());

    // Extra constraints from config.
    for extra in &config.extra_constraints {
        lines.push(format!("- {extra}"));
    }

    lines.join("\n")
}

fn render_output_section(max_chars: usize, format: &AgentOutputFormat) -> String {
    let mut lines = vec!["=== OUTPUT ===".to_string()];
    lines.push(format!("Max length: {max_chars} characters"));
    lines.push(format!("Format: {format}"));
    lines.join("\n")
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn effective_max_output(type_default: usize, config: &AgentTemplateConfig) -> usize {
    let base = if config.default_max_output_chars > 0 {
        // Config override takes precedence only when the type default equals
        // the module-level default (i.e. the type has no specific override).
        if type_default == DEFAULT_MAX_OUTPUT_CHARS {
            config.default_max_output_chars
        } else {
            type_default
        }
    } else {
        type_default
    };
    base.min(ABSOLUTE_MAX_OUTPUT_CHARS)
}

fn read_git_branch(cwd: &Path) -> Option<String> {
    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(cwd)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let branch = String::from_utf8(output.stdout).ok()?;
    let trimmed = branch.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context_budget::ModelTier;

    fn test_context() -> AgentPromptContext {
        AgentPromptContext {
            cwd: PathBuf::from("/tmp/project"),
            git_branch: Some("feature/a29".to_string()),
            active_files: vec![
                "src/lib.rs".to_string(),
                "src/main.rs".to_string(),
            ],
            model_tier: ModelTier::Micro,
            model_name: "qwen2.5-coder-3b".to_string(),
            max_output_tokens: 2048,
            max_context_tokens: 25000,
            active_rules: vec!["no-unwrap in production code".to_string()],
        }
    }

    #[test]
    fn build_agent_prompt_contains_all_four_sections() {
        let ctx = test_context();
        let config = AgentTemplateConfig::default();
        let prompt = build_agent_prompt("Find all usages of foo()", "Explore", &ctx, &config);

        assert!(prompt.contains("=== CONTEXT ==="));
        assert!(prompt.contains("=== TASK ==="));
        assert!(prompt.contains("=== CONSTRAINTS ==="));
        assert!(prompt.contains("=== OUTPUT ==="));
    }

    #[test]
    fn context_section_includes_cwd_branch_files_tier() {
        let ctx = test_context();
        let config = AgentTemplateConfig::default();
        let prompt = build_agent_prompt("task", "Explore", &ctx, &config);

        assert!(prompt.contains("Working directory: /tmp/project"));
        assert!(prompt.contains("Git branch: feature/a29"));
        assert!(prompt.contains("src/lib.rs"));
        assert!(prompt.contains("Model tier: Micro"));
    }

    #[test]
    fn task_section_preserves_raw_task() {
        let ctx = test_context();
        let config = AgentTemplateConfig::default();
        let task = "Search for all files matching *.rs that import tracing";
        let prompt = build_agent_prompt(task, "Explore", &ctx, &config);

        assert!(prompt.contains(task));
    }

    #[test]
    fn constraints_include_active_rules_and_standard_rules() {
        let ctx = test_context();
        let config = AgentTemplateConfig::default();
        let prompt = build_agent_prompt("task", "general-purpose", &ctx, &config);

        assert!(prompt.contains("no-unwrap in production code"));
        assert!(prompt.contains("concise, no emojis, absolute paths only"));
        assert!(prompt.contains("Do not ask the user questions"));
    }

    #[test]
    fn output_section_uses_type_defaults() {
        let ctx = test_context();
        let config = AgentTemplateConfig::default();

        let explore_prompt = build_agent_prompt("task", "Explore", &ctx, &config);
        assert!(explore_prompt.contains("Max length: 3000 characters"));
        assert!(explore_prompt.contains("Format: bullet findings"));

        let plan_prompt = build_agent_prompt("task", "Plan", &ctx, &config);
        assert!(plan_prompt.contains("Max length: 6000 characters"));
        assert!(plan_prompt.contains("Format: prose"));
    }

    #[test]
    fn config_overrides_default_max_output_for_unspecialized_types() {
        let ctx = test_context();
        let config = AgentTemplateConfig {
            default_max_output_chars: 8_000,
            ..AgentTemplateConfig::default()
        };
        // general-purpose uses the module default, so config override applies.
        let prompt = build_agent_prompt("task", "general-purpose", &ctx, &config);
        assert!(prompt.contains("Max length: 8000 characters"));

        // Explore has a type-specific default (3000), so config does NOT override.
        let explore = build_agent_prompt("task", "Explore", &ctx, &config);
        assert!(explore.contains("Max length: 3000 characters"));
    }

    #[test]
    fn absolute_max_output_enforced() {
        let ctx = test_context();
        let config = AgentTemplateConfig {
            default_max_output_chars: 100_000,
            ..AgentTemplateConfig::default()
        };
        let prompt = build_agent_prompt("task", "general-purpose", &ctx, &config);
        assert!(prompt.contains(&format!("Max length: {ABSOLUTE_MAX_OUTPUT_CHARS} characters")));
    }

    #[test]
    fn extra_constraints_injected() {
        let ctx = test_context();
        let config = AgentTemplateConfig {
            extra_constraints: vec!["Use blake3 for hashing".to_string()],
            ..AgentTemplateConfig::default()
        };
        let prompt = build_agent_prompt("task", "Explore", &ctx, &config);
        assert!(prompt.contains("Use blake3 for hashing"));
    }

    #[test]
    fn system_preamble_contains_agent_type() {
        let preamble = build_agent_system_preamble("Verification");
        assert!(preamble.contains("Verification"));
        assert!(preamble.contains("background sub-agent"));
    }

    #[test]
    fn active_files_truncated_after_ten() {
        let mut ctx = test_context();
        ctx.active_files = (0..15).map(|i| format!("file_{i}.rs")).collect();
        let config = AgentTemplateConfig::default();
        let prompt = build_agent_prompt("task", "Explore", &ctx, &config);
        assert!(prompt.contains("(15 total)"));
        assert!(prompt.contains("..."));
    }

    #[test]
    fn missing_git_branch_omitted() {
        let mut ctx = test_context();
        ctx.git_branch = None;
        let config = AgentTemplateConfig::default();
        let prompt = build_agent_prompt("task", "Explore", &ctx, &config);
        assert!(!prompt.contains("Git branch:"));
    }

    #[test]
    fn agent_type_config_defaults_for_unknown_type() {
        let cfg = AgentTypeConfig::defaults_for("unknown-type");
        assert_eq!(cfg.max_output_chars, DEFAULT_MAX_OUTPUT_CHARS);
        assert_eq!(cfg.output_format, AgentOutputFormat::Prose);
    }

    #[test]
    fn from_cwd_sets_defaults() {
        let ctx = AgentPromptContext::from_cwd("/tmp", ModelTier::Small, "test-model");
        assert_eq!(ctx.cwd, PathBuf::from("/tmp"));
        assert_eq!(ctx.model_tier, ModelTier::Small);
        assert!(ctx.active_files.is_empty());
        assert!(ctx.active_rules.is_empty());
    }

    #[test]
    fn builder_methods_chain() {
        let ctx = AgentPromptContext::from_cwd("/tmp", ModelTier::Micro, "qwen")
            .with_active_files(vec!["a.rs".to_string()])
            .with_active_rules(vec!["rule1".to_string()])
            .with_token_limits(1024, 8000);
        assert_eq!(ctx.active_files.len(), 1);
        assert_eq!(ctx.active_rules.len(), 1);
        assert_eq!(ctx.max_output_tokens, 1024);
        assert_eq!(ctx.max_context_tokens, 8000);
    }

    #[test]
    fn default_config_round_trips_through_toml() {
        let config = AgentTemplateConfig::default();
        let serialized = toml::to_string(&config).expect("serialize");
        let deserialized: AgentTemplateConfig = toml::from_str(&serialized).expect("deserialize");
        assert_eq!(config, deserialized);
    }
}

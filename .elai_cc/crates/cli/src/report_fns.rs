#![allow(dead_code, unused_imports, unused_variables, unused_mut)]

// T12c: free-standing report/format/utility functions extracted from live_cli.rs.
// Callers continue to resolve these via `pub(crate) use crate::report_fns::*;`
// in live_cli.rs (which feeds into the `pub(crate) use crate::live_cli::*;` glob
// in main.rs), so no import-site changes are needed anywhere.

use std::borrow::ToOwned;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;

use elai_orchestrator::{
    apply_to_system_prompt, ConfigLoader, ConfigSource, ProjectContext, Session, ContentBlock,
    MessageRole, RoleContractStore, RuntimeHookConfig, TokenUsage, PermissionMode,
    load_system_prompt, resolve_sandbox_status,
    // I-36: compiled-prompt artifact store for per-(role,backend) variant substitution.
    CompiledPromptStore,
};
use elai_plugins::{PluginHooks, PluginManager, PluginManagerConfig};
use elai_rules::rule_engine::{CompiledPredicate, DefaultRuleEngine, Rule, RuleEngine};
use elai_tools::GlobalToolRegistry;
use elai_knowledge::{
    detect_project_profile, populate_tier3, assemble_telescope, assemble_telescope_with_hooks,
    DkgEngine, PiT1Hook, PiT2Hook, PiT3Hook, ProjectIntelligenceHook,
};
use elai_reasoning::build_with_conventions;
use elai_core::paths::dkg_db_path as core_dkg_db_path;
use serde::Deserialize;
use serde_json::json;

use crate::{
    CliOutputFormat, LocalHelpTopic, StatusContext, StatusUsage, GitWorkspaceSummary,
    SessionHandle, RuntimePluginState,
    VERSION, GIT_SHA, BUILD_TARGET, DEFAULT_DATE, TOKEN_CHARS_PER_TOKEN, PLANS_DIR,
    PRIMARY_SESSION_EXTENSION,
    build_runtime_mcp_state, parse_git_status_metadata, parse_git_workspace_summary,
    handoff,
};

// ---------------------------------------------------------------------------
// Plan / phase helpers
// ---------------------------------------------------------------------------

/// Attempt to load a PlanSkeleton from the plans/ directory.
/// Returns None if no plan file exists or parsing fails.
pub(crate) fn load_plan_if_exists() -> Option<elai_orchestrator::PlanSkeleton> {
    let cwd = env::current_dir().ok()?;
    let plans_dir = cwd.join(PLANS_DIR);
    if !plans_dir.is_dir() {
        return None;
    }
    // Find the first .toml file in plans/ (sorted for determinism)
    let mut toml_files: Vec<PathBuf> = fs::read_dir(&plans_dir).ok()?
        .filter_map(|e| {
            let path = e.ok()?.path();
            if path.extension().and_then(|s| s.to_str()) == Some("toml") {
                Some(path)
            } else {
                None
            }
        })
        .collect();
    toml_files.sort();
    let entry = toml_files.into_iter().next()?;
    match elai_orchestrator::PlanSkeleton::read_from_file(&entry) {
        Ok(plan) => Some(plan),
        Err(e) => {
            eprintln!("warn: plan load failed from {}: {e}", entry.display());
            None
        }
    }
}

pub(crate) fn plan_phase_slot_tokens(plan: Option<&elai_orchestrator::PlanSkeleton>) -> usize {
    let skeleton_tokens = plan
        .and_then(|p| p.to_toml().ok())
        .map(|s| s.len() / TOKEN_CHARS_PER_TOKEN)
        .unwrap_or(0);
    // A-7: include constitutional CONSTRAINTS block in PhaseSpec budget.
    let constraints_tokens = build_constraints_block().estimated_tokens();
    skeleton_tokens + constraints_tokens
}

/// Build the CONSTRAINTS block from static seed data (A-7: constitutional injection).
///
/// Uses BLOCK-severity entries from deprecated_patterns seed to generate
/// the constraint text appended to PhaseSpec content at assembly time.
pub(crate) fn build_constraints_block() -> elai_orchestrator::context_budget::ConstraintsBlock {
    use elai_orchestrator::context_budget::{ConstraintEntry, ConstraintsBlock};
    let seed_entries = elai_rules::schema::block_entries_from_seed();
    let entries: Vec<ConstraintEntry> = seed_entries
        .into_iter()
        .map(|e| ConstraintEntry {
            deprecated_name: e.deprecated_name,
            replacement_name: e.replacement_name,
        })
        .collect();
    ConstraintsBlock::from_entries(entries)
}

/// Build the telescope context block (A-11) from the current project profile.
///
/// Detects the project profile from cwd, then formats the telescope context
/// for injection into the PhaseSpec system prompt. Returns None if detection fails.
pub(crate) fn build_telescope_block() -> Option<String> {
    let project_root = std::env::current_dir().ok()?;
    let mut profile = detect_project_profile(&project_root).ok()?;

    // Wire DKG nodes for architecture inference (A-36).
    // B-1: read from canonical `.elai_cc/indexes/dkg.db` via core::paths.
    // Graceful degradation: if DB absent or engine fails, continue with empty nodes.
    let dkg_db_path = core_dkg_db_path(&project_root);
    let dkg_nodes = if dkg_db_path.exists() {
        DkgEngine::new(&dkg_db_path, None, 0)
            .and_then(|engine| engine.nodes())
            .unwrap_or_default()
    } else {
        Vec::new()
    };

    populate_tier3(&mut profile, &dkg_nodes);

    // phase-3L (3L.1 + 3L.2): assemble the telescope through the PI hook path
    // instead of the legacy hook-free `assemble_telescope`. The three tier
    // hooks render priority-ordered prompt blocks (T1 CONVENTIONS > T2 IMPORTS
    // > T3 PROFILE) that `assemble_telescope_with_hooks` sorts + budget-caps.
    // The legacy `format_for_phase_spec` output is preserved verbatim as the
    // base so non-hook behaviour is unchanged; the PiT1 CONVENTIONS block is
    // then injected into the SYSTEM-prompt-bound text via
    // `build_with_conventions` (idempotent; tagged with CONVENTIONS_BLOCK_HEADER
    // so the auditor can confirm injection on a real entrypoint), and the
    // remaining hook blocks are appended.
    let hooks: [Box<dyn ProjectIntelligenceHook>; 3] = [
        Box::new(PiT1Hook::new()),
        Box::new(PiT2Hook::new()),
        Box::new(PiT3Hook::new()),
    ];
    let (telescope_ctx, hook_blocks) = assemble_telescope_with_hooks(&profile, &hooks);
    let base = telescope_ctx.format_for_phase_spec();

    // Inject the highest-priority CONVENTIONS block (PiT1::render_block) into
    // the base telescope text. This is the production reach for
    // `build_with_conventions` / PiT1Hook::render_block per spec 3L.2.
    let conventions_block = PiT1Hook::new().render_block(&profile);
    let mut formatted = build_with_conventions(conventions_block.as_deref(), &base);

    // Append the remaining (T2 IMPORTS + T3 PROFILE) hook blocks so their
    // render output also reaches the SYSTEM prompt. The T1 block is skipped
    // here because `build_with_conventions` already placed it (and stamped the
    // CONVENTIONS header that the idempotency guard keys on).
    for block in &hook_blocks {
        if block.starts_with(elai_reasoning::CONVENTIONS_BLOCK_HEADER) {
            continue;
        }
        if !formatted.is_empty() && !formatted.ends_with('\n') {
            formatted.push('\n');
        }
        formatted.push_str(block);
    }

    if formatted.is_empty() {
        None
    } else {
        Some(formatted)
    }
}

/// Build the PhaseSpec content string: plan skeleton TOML + CONSTRAINTS block + telescope block.
///
/// Returns None if no plan is loaded, constraints block is empty, and telescope is None.
/// This string is appended to the system prompt as an additional section so that
/// the constitutional CONSTRAINTS block and telescope context are visible to the model (A-7, A-11).
pub(crate) fn build_phase_spec_content(
    plan: Option<&elai_orchestrator::PlanSkeleton>,
    telescope: Option<&str>,
) -> Option<String> {
    let skeleton_toml = plan.and_then(|p| p.to_toml().ok());
    let constraints = build_constraints_block();
    let constraints_text = constraints.format();

    let base = match (skeleton_toml, constraints_text.is_empty()) {
        (None, true) => None,
        (Some(toml), true) => Some(toml),
        (None, false) => Some(constraints_text),
        (Some(mut toml), false) => {
            toml.push_str(&constraints_text);
            Some(toml)
        }
    };

    match (base, telescope) {
        (None, None) => None,
        (Some(b), None) => Some(b),
        (None, Some(t)) => Some(t.to_string()),
        (Some(mut b), Some(t)) => {
            b.push('\n');
            b.push_str(t);
            Some(b)
        }
    }
}

// ---------------------------------------------------------------------------
// Session handle
// ---------------------------------------------------------------------------

pub(crate) fn create_managed_session_handle(
    session_id: &str,
) -> Result<SessionHandle, anyhow::Error> {
    let id = session_id.to_string();
    let path = handoff::sessions_dir()?.join(format!("{id}.{PRIMARY_SESSION_EXTENSION}"));
    Ok(SessionHandle { id, path })
}

// ---------------------------------------------------------------------------
// REPL / status / sandbox reports
// ---------------------------------------------------------------------------

pub(crate) fn render_repl_help() -> String {
    use crate::commands::render_slash_command_help;
    [
        "REPL".to_string(),
        "  /exit                Quit the REPL".to_string(),
        "  /quit                Quit the REPL".to_string(),
        "  Up/Down              Navigate prompt history".to_string(),
        "  Tab                  Complete commands, modes, and recent sessions".to_string(),
        "  Ctrl-C               Clear input (or exit on empty prompt)".to_string(),
        "  Shift+Enter/Ctrl+J   Insert a newline".to_string(),
        "  Auto-save            .elai/sessions/<session-id>.jsonl".to_string(),
        "  Resume latest        /resume latest".to_string(),
        "  Browse sessions      /session list".to_string(),
        String::new(),
        render_slash_command_help(),
    ]
    .join(
        "
",
    )
}

pub(crate) fn print_status_snapshot(
    model: &str,
    permission_mode: PermissionMode,
    output_format: CliOutputFormat,
) -> Result<(), anyhow::Error> {
    let usage = StatusUsage {
        message_count: 0,
        turns: 0,
        latest: TokenUsage::default(),
        cumulative: TokenUsage::default(),
        estimated_tokens: 0,
    };
    let context = status_context(None)?;
    match output_format {
        CliOutputFormat::Text | CliOutputFormat::Toml => println!(
            "{}",
            format_status_report(model, usage, permission_mode.as_str(), &context)
        ),
        CliOutputFormat::Json => println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "kind": "status",
                "model": model,
                "permission_mode": permission_mode.as_str(),
                "usage": {
                    "messages": usage.message_count,
                    "turns": usage.turns,
                    "latest_total": usage.latest.total_tokens(),
                    "cumulative_input": usage.cumulative.input_tokens,
                    "cumulative_output": usage.cumulative.output_tokens,
                    "cumulative_total": usage.cumulative.total_tokens(),
                    "estimated_tokens": usage.estimated_tokens,
                },
                "workspace": {
                    "cwd": context.cwd,
                    "project_root": context.project_root,
                    "git_branch": context.git_branch,
                    "git_state": context.git_summary.headline(),
                    "changed_files": context.git_summary.changed_files,
                    "staged_files": context.git_summary.staged_files,
                    "unstaged_files": context.git_summary.unstaged_files,
                    "untracked_files": context.git_summary.untracked_files,
                    "session": context.session_path.as_ref().map_or_else(|| "live-repl".to_string(), |path| path.display().to_string()),
                    "loaded_config_files": context.loaded_config_files,
                    "discovered_config_files": context.discovered_config_files,
                    "memory_file_count": context.memory_file_count,
                },
                "sandbox": {
                    "enabled": context.sandbox_status.enabled,
                    "active": context.sandbox_status.active,
                    "supported": context.sandbox_status.supported,
                    "in_container": context.sandbox_status.in_container,
                    "requested_namespace": context.sandbox_status.requested.namespace_restrictions,
                    "active_namespace": context.sandbox_status.namespace_active,
                    "requested_network": context.sandbox_status.requested.network_isolation,
                    "active_network": context.sandbox_status.network_active,
                    "filesystem_mode": context.sandbox_status.filesystem_mode.as_str(),
                    "filesystem_active": context.sandbox_status.filesystem_active,
                    "allowed_mounts": context.sandbox_status.allowed_mounts,
                    "markers": context.sandbox_status.container_markers,
                    "fallback_reason": context.sandbox_status.fallback_reason,
                }
            }))?
        ),
    }
    Ok(())
}

pub(crate) fn status_context(
    session_path: Option<&Path>,
) -> Result<StatusContext, anyhow::Error> {
    let cwd = env::current_dir()?;
    let loader = ConfigLoader::default_for(&cwd);
    let discovered_config_files = loader.discover().len();
    let runtime_config = loader.load()?;
    let project_context = ProjectContext::discover_with_git(&cwd, DEFAULT_DATE)?;
    let (project_root, git_branch) =
        parse_git_status_metadata(project_context.git_status.as_deref());
    let git_summary = parse_git_workspace_summary(project_context.git_status.as_deref());
    let sandbox_status = resolve_sandbox_status(runtime_config.sandbox(), &cwd);
    Ok(StatusContext {
        cwd,
        session_path: session_path.map(Path::to_path_buf),
        loaded_config_files: runtime_config.loaded_entries().len(),
        discovered_config_files,
        memory_file_count: project_context.instruction_files.len(),
        project_root,
        git_branch,
        git_summary,
        sandbox_status,
    })
}

pub(crate) fn format_status_report(
    model: &str,
    usage: StatusUsage,
    permission_mode: &str,
    context: &StatusContext,
) -> String {
    [
        format!(
            "Status
  Model            {model}
  Permission mode  {permission_mode}
  Messages         {}
  Turns            {}
  Estimated tokens {}",
            usage.message_count, usage.turns, usage.estimated_tokens,
        ),
        format!(
            "Usage
  Latest total     {}
  Cumulative input {}
  Cumulative output {}
  Cumulative total {}",
            usage.latest.total_tokens(),
            usage.cumulative.input_tokens,
            usage.cumulative.output_tokens,
            usage.cumulative.total_tokens(),
        ),
        format!(
            "Workspace
  Cwd              {}
  Project root     {}
  Git branch       {}
  Git state        {}
  Changed files    {}
  Staged           {}
  Unstaged         {}
  Untracked        {}
  Session          {}
  Config files     loaded {}/{}
  Memory files     {}
  Suggested flow   /status → /diff → /commit",
            context.cwd.display(),
            context
                .project_root
                .as_ref()
                .map_or_else(|| "unknown".to_string(), |path| path.display().to_string()),
            context.git_branch.as_deref().unwrap_or("unknown"),
            context.git_summary.headline(),
            context.git_summary.changed_files,
            context.git_summary.staged_files,
            context.git_summary.unstaged_files,
            context.git_summary.untracked_files,
            context.session_path.as_ref().map_or_else(
                || "live-repl".to_string(),
                |path| path.display().to_string()
            ),
            context.loaded_config_files,
            context.discovered_config_files,
            context.memory_file_count,
        ),
        format_sandbox_report(&context.sandbox_status),
    ]
    .join(
        "

",
    )
}

pub(crate) fn format_sandbox_report(status: &elai_orchestrator::SandboxStatus) -> String {
    format!(
        "Sandbox
  Enabled           {}
  Active            {}
  Supported         {}
  In container      {}
  Requested ns      {}
  Active ns         {}
  Requested net     {}
  Active net        {}
  Filesystem mode   {}
  Filesystem active {}
  Allowed mounts    {}
  Markers           {}
  Fallback reason   {}",
        status.enabled,
        status.active,
        status.supported,
        status.in_container,
        status.requested.namespace_restrictions,
        status.namespace_active,
        status.requested.network_isolation,
        status.network_active,
        status.filesystem_mode.as_str(),
        status.filesystem_active,
        if status.allowed_mounts.is_empty() {
            "<none>".to_string()
        } else {
            status.allowed_mounts.join(", ")
        },
        if status.container_markers.is_empty() {
            "<none>".to_string()
        } else {
            status.container_markers.join(", ")
        },
        status
            .fallback_reason
            .clone()
            .unwrap_or_else(|| "<none>".to_string()),
    )
}

pub(crate) fn format_commit_preflight_report(branch: Option<&str>, summary: GitWorkspaceSummary) -> String {
    format!(
        "Commit
  Result           ready
  Branch           {}
  Workspace        {}
  Changed files    {}
  Action           create a git commit from the current workspace changes",
        branch.unwrap_or("unknown"),
        summary.headline(),
        summary.changed_files,
    )
}

pub(crate) fn format_commit_skipped_report() -> String {
    "Commit
  Result           skipped
  Reason           no workspace changes
  Action           create a git commit from the current workspace changes
  Next             /status to inspect context · /diff to inspect repo changes"
        .to_string()
}

pub(crate) fn print_sandbox_status_snapshot(
    output_format: CliOutputFormat,
) -> Result<(), anyhow::Error> {
    let cwd = env::current_dir()?;
    let loader = ConfigLoader::default_for(&cwd);
    let runtime_config = loader
        .load()
        .unwrap_or_else(|_| elai_orchestrator::RuntimeConfig::empty());
    let status = resolve_sandbox_status(runtime_config.sandbox(), &cwd);
    match output_format {
        CliOutputFormat::Text | CliOutputFormat::Toml => {
            println!("{}", format_sandbox_report(&status))
        }
        CliOutputFormat::Json => println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "kind": "sandbox",
                "enabled": status.enabled,
                "active": status.active,
                "supported": status.supported,
                "in_container": status.in_container,
                "requested_namespace": status.requested.namespace_restrictions,
                "active_namespace": status.namespace_active,
                "requested_network": status.requested.network_isolation,
                "active_network": status.network_active,
                "filesystem_mode": status.filesystem_mode.as_str(),
                "filesystem_active": status.filesystem_active,
                "allowed_mounts": status.allowed_mounts,
                "markers": status.container_markers,
                "fallback_reason": status.fallback_reason,
            }))?
        ),
    }
    Ok(())
}

pub(crate) fn render_help_topic(topic: LocalHelpTopic) -> String {
    match topic {
        LocalHelpTopic::Status => "Status
  Usage            elai status
  Purpose          show the local workspace snapshot without entering the REPL
  Output           model, permissions, git state, config files, and sandbox status
  Related          /status · elai --resume latest /status"
            .to_string(),
        LocalHelpTopic::Sandbox => "Sandbox
  Usage            elai sandbox
  Purpose          inspect the resolved sandbox and isolation state for the current directory
  Output           namespace, network, filesystem, and fallback details
  Related          /sandbox · elai status"
            .to_string(),
        LocalHelpTopic::Doctor => "Doctor
  Usage            elai doctor
  Purpose          diagnose local auth, config, workspace, sandbox, and build metadata
  Output           local-only health report; no provider request or session resume required
  Related          /doctor · elai --resume latest /doctor"
            .to_string(),
    }
}

pub(crate) fn print_help_topic(topic: LocalHelpTopic) {
    println!("{}", render_help_topic(topic));
}

// ---------------------------------------------------------------------------
// Config / memory reports
// ---------------------------------------------------------------------------

pub(crate) fn render_config_report(section: Option<&str>) -> Result<String, anyhow::Error> {
    let cwd = env::current_dir()?;
    let loader = ConfigLoader::default_for(&cwd);
    let discovered = loader.discover();
    let runtime_config = loader.load()?;

    let mut lines = vec![
        format!(
            "Config
  Working directory {}
  Loaded files      {}
  Merged keys       {}",
            cwd.display(),
            runtime_config.loaded_entries().len(),
            runtime_config.merged().len()
        ),
        "Discovered files".to_string(),
    ];
    for entry in discovered {
        let source = match entry.source {
            ConfigSource::User => "user",
            ConfigSource::Project => "project",
            ConfigSource::Local => "local",
        };
        let status = if runtime_config
            .loaded_entries()
            .iter()
            .any(|loaded_entry| loaded_entry.path == entry.path)
        {
            "loaded"
        } else {
            "missing"
        };
        lines.push(format!(
            "  {source:<7} {status:<7} {}",
            entry.path.display()
        ));
    }

    if let Some(section) = section {
        lines.push(format!("Merged section: {section}"));
        let value = match section {
            "env" => runtime_config.get("env"),
            "hooks" => runtime_config.get("hooks"),
            "model" => runtime_config.get("model"),
            "plugins" => runtime_config
                .get("plugins")
                .or_else(|| runtime_config.get("enabledPlugins")),
            other => {
                lines.push(format!(
                    "  Unsupported config section '{other}'. Use env, hooks, model, or plugins."
                ));
                return Ok(lines.join(
                    "
",
                ));
            }
        };
        lines.push(format!(
            "  {}",
            match value {
                Some(value) => value.render(),
                None => "<unset>".to_string(),
            }
        ));
        return Ok(lines.join(
            "
",
        ));
    }

    lines.push("Merged JSON".to_string());
    lines.push(format!("  {}", runtime_config.as_json().render()));
    Ok(lines.join(
        "
",
    ))
}

pub(crate) fn render_memory_report() -> Result<String, anyhow::Error> {
    let cwd = env::current_dir()?;
    let project_context = ProjectContext::discover(&cwd, DEFAULT_DATE)?;
    let mut lines = vec![format!(
        "Memory
  Working directory {}
  Instruction files {}",
        cwd.display(),
        project_context.instruction_files.len()
    )];
    if project_context.instruction_files.is_empty() {
        lines.push("Discovered files".to_string());
        lines.push(
            "  No ELAI instruction files discovered in the current directory ancestry."
                .to_string(),
        );
    } else {
        lines.push("Discovered files".to_string());
        for (index, file) in project_context.instruction_files.iter().enumerate() {
            let preview = file.content.lines().next().unwrap_or("").trim();
            let preview = if preview.is_empty() {
                "<empty>"
            } else {
                preview
            };
            lines.push(format!("  {}. {}", index + 1, file.path.display(),));
            lines.push(format!(
                "     lines={} preview={}",
                file.content.lines().count(),
                preview
            ));
        }
    }
    Ok(lines.join(
        "
",
    ))
}

pub(crate) fn init_elai_md() -> Result<String, anyhow::Error> {
    let cwd = env::current_dir()?;
    Ok(crate::init::initialize_repo(&cwd)?.render())
}

pub(crate) fn run_init(output_format: CliOutputFormat) -> Result<(), anyhow::Error> {
    let message = init_elai_md()?;
    match output_format {
        CliOutputFormat::Text | CliOutputFormat::Toml => println!("{message}"),
        CliOutputFormat::Json => println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "kind": "init",
                "message": message,
            }))?
        ),
    }
    Ok(())
}

pub(crate) fn normalize_permission_mode(mode: &str) -> Option<&'static str> {
    match mode.trim() {
        "read-only" => Some("read-only"),
        "workspace-write" => Some("workspace-write"),
        "danger-full-access" => Some("danger-full-access"),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Diff / teleport reports
// ---------------------------------------------------------------------------

pub(crate) fn render_diff_report() -> Result<String, anyhow::Error> {
    render_diff_report_for(&env::current_dir()?)
}

pub(crate) fn render_diff_report_for(cwd: &Path) -> Result<String, anyhow::Error> {
    let staged = run_git_diff_command_in(cwd, &["diff", "--cached"])?;
    let unstaged = run_git_diff_command_in(cwd, &["diff"])?;
    if staged.trim().is_empty() && unstaged.trim().is_empty() {
        return Ok(
            "Diff\n  Result           clean working tree\n  Detail           no current changes"
                .to_string(),
        );
    }

    let mut sections = Vec::new();
    if !staged.trim().is_empty() {
        sections.push(format!("Staged changes:\n{}", staged.trim_end()));
    }
    if !unstaged.trim().is_empty() {
        sections.push(format!("Unstaged changes:\n{}", unstaged.trim_end()));
    }

    Ok(format!("Diff\n\n{}", sections.join("\n\n")))
}

pub(crate) fn run_git_diff_command_in(
    cwd: &Path,
    args: &[&str],
) -> Result<String, anyhow::Error> {
    let output = std::process::Command::new("git")
        .args(args)
        .current_dir(cwd)
        .output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(anyhow::anyhow!("git {} failed: {stderr}", args.join(" ")));
    }
    Ok(String::from_utf8(output.stdout)?)
}

pub(crate) fn render_teleport_report(target: &str) -> Result<String, anyhow::Error> {
    let cwd = env::current_dir()?;

    let file_list = Command::new("rg")
        .args(["--files"])
        .current_dir(&cwd)
        .output()?;
    let file_matches = if file_list.status.success() {
        String::from_utf8(file_list.stdout)?
            .lines()
            .filter(|line| line.contains(target))
            .take(10)
            .map(ToOwned::to_owned)
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };

    let content_output = Command::new("rg")
        .args(["-n", "-S", "--color", "never", target, "."])
        .current_dir(&cwd)
        .output()?;

    let mut lines = vec![
        "Teleport".to_string(),
        format!("  Target           {target}"),
        "  Action           search workspace files and content for the target".to_string(),
    ];
    if !file_matches.is_empty() {
        lines.push(String::new());
        lines.push("File matches".to_string());
        lines.extend(file_matches.into_iter().map(|path| format!("  {path}")));
    }

    if content_output.status.success() {
        let matches = String::from_utf8(content_output.stdout)?;
        if !matches.trim().is_empty() {
            lines.push(String::new());
            lines.push("Content matches".to_string());
            lines.push(truncate_for_prompt(&matches, 4_000));
        }
    }

    if lines.len() == 1 {
        lines.push("  Result           no matches found".to_string());
    }

    Ok(lines.join("\n"))
}

pub(crate) fn render_last_tool_debug_report(session: &Session) -> Result<String, anyhow::Error> {
    let last_tool_use = session
        .messages
        .iter()
        .rev()
        .find_map(|message| {
            message.blocks.iter().rev().find_map(|block| match block {
                ContentBlock::ToolUse { id, name, input } => {
                    Some((id.clone(), name.clone(), input.clone()))
                }
                _ => None,
            })
        })
        .ok_or_else(|| anyhow::anyhow!("no prior tool call found in session"))?;

    let tool_result = session.messages.iter().rev().find_map(|message| {
        message.blocks.iter().rev().find_map(|block| match block {
            ContentBlock::ToolResult {
                tool_use_id,
                tool_name,
                output,
                is_error,
            } if tool_use_id == &last_tool_use.0 => {
                Some((tool_name.clone(), output.clone(), *is_error))
            }
            _ => None,
        })
    });

    let mut lines = vec![
        "Debug tool call".to_string(),
        "  Action           inspect the last recorded tool call and its result".to_string(),
        format!("  Tool id          {}", last_tool_use.0),
        format!("  Tool name        {}", last_tool_use.1),
        "  Input".to_string(),
        indent_block(&last_tool_use.2, 4),
    ];

    match tool_result {
        Some((tool_name, output, is_error)) => {
            lines.push("  Result".to_string());
            lines.push(format!("    name           {tool_name}"));
            lines.push(format!(
                "    status         {}",
                if is_error { "error" } else { "ok" }
            ));
            lines.push(indent_block(&output, 4));
        }
        None => lines.push("  Result           missing tool result".to_string()),
    }

    Ok(lines.join("\n"))
}

pub(crate) fn indent_block(value: &str, spaces: usize) -> String {
    let indent = " ".repeat(spaces);
    value
        .lines()
        .map(|line| format!("{indent}{line}"))
        .collect::<Vec<_>>()
        .join("\n")
}

pub(crate) fn validate_no_args(
    command_name: &str,
    args: Option<&str>,
) -> Result<(), anyhow::Error> {
    if let Some(args) = args.map(str::trim).filter(|value| !value.is_empty()) {
        return Err(anyhow::anyhow!(
            "{command_name} does not accept arguments. Received: {args}\nUsage: {command_name}"
        ));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Domain-specific format helpers
// ---------------------------------------------------------------------------

pub(crate) fn format_bughunter_report(scope: Option<&str>) -> String {
    format!(
        "Bughunter
  Scope            {}
  Action           inspect the selected code for likely bugs and correctness issues
  Output           findings should include file paths, severity, and suggested fixes",
        scope.unwrap_or("the current repository")
    )
}

pub(crate) fn format_ultraplan_report(task: Option<&str>) -> String {
    format!(
        "Ultraplan
  Task             {}
  Action           break work into a multi-step execution plan
  Output           plan should cover goals, risks, sequencing, verification, and rollback",
        task.unwrap_or("the current repo work")
    )
}

pub(crate) fn format_pr_report(branch: &str, context: Option<&str>) -> String {
    format!(
        "PR
  Branch           {branch}
  Context          {}
  Action           draft or create a pull request for the current branch
  Output           title and markdown body suitable for GitHub",
        context.unwrap_or("none")
    )
}

pub(crate) fn format_issue_report(context: Option<&str>) -> String {
    format!(
        "Issue
  Context          {}
  Action           draft or create a GitHub issue from the current context
  Output           title and markdown body suitable for GitHub",
        context.unwrap_or("none")
    )
}

// ---------------------------------------------------------------------------
// Git utility helpers
// ---------------------------------------------------------------------------

pub(crate) fn git_output(args: &[&str]) -> Result<String, anyhow::Error> {
    let output = Command::new("git")
        .args(args)
        .current_dir(env::current_dir()?)
        .output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(anyhow::anyhow!("git {} failed: {stderr}", args.join(" ")));
    }
    Ok(String::from_utf8(output.stdout)?)
}

pub(crate) fn git_status_ok(args: &[&str]) -> Result<(), anyhow::Error> {
    let output = Command::new("git")
        .args(args)
        .current_dir(env::current_dir()?)
        .output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(anyhow::anyhow!("git {} failed: {stderr}", args.join(" ")));
    }
    Ok(())
}

pub(crate) fn command_exists(name: &str) -> bool {
    Command::new("which")
        .arg(name)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

pub(crate) fn write_temp_text_file(
    filename: &str,
    contents: &str,
) -> Result<PathBuf, anyhow::Error> {
    let path = env::temp_dir().join(filename);
    fs::write(&path, contents)?;
    Ok(path)
}

// ---------------------------------------------------------------------------
// Session / prompt helpers
// ---------------------------------------------------------------------------

pub(crate) fn recent_user_context(session: &Session, limit: usize) -> String {
    let requests = session
        .messages
        .iter()
        .filter(|message| message.role == MessageRole::User)
        .filter_map(|message| {
            message.blocks.iter().find_map(|block| match block {
                ContentBlock::Text { text } => Some(text.trim().to_string()),
                _ => None,
            })
        })
        .rev()
        .take(limit)
        .collect::<Vec<_>>();

    if requests.is_empty() {
        "<no prior user messages>".to_string()
    } else {
        requests
            .into_iter()
            .rev()
            .enumerate()
            .map(|(index, text)| format!("{}. {}", index + 1, text))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

pub(crate) fn truncate_for_prompt(value: &str, limit: usize) -> String {
    if value.chars().count() <= limit {
        value.trim().to_string()
    } else {
        let truncated = value.chars().take(limit).collect::<String>();
        format!("{}\n…[truncated]", truncated.trim_end())
    }
}

pub(crate) fn sanitize_generated_message(value: &str) -> String {
    value.trim().trim_matches('`').trim().replace("\r\n", "\n")
}

pub(crate) fn parse_titled_body(value: &str) -> Option<(String, String)> {
    let normalized = sanitize_generated_message(value);
    let title = normalized
        .lines()
        .find_map(|line| line.strip_prefix("TITLE:").map(str::trim))?;
    let body_start = normalized.find("BODY:")?;
    let body = normalized[body_start + "BODY:".len()..].trim();
    Some((title.to_string(), body.to_string()))
}

// ---------------------------------------------------------------------------
// Version / export / path helpers
// ---------------------------------------------------------------------------

pub(crate) fn render_version_report() -> String {
    let git_sha = GIT_SHA.unwrap_or("unknown");
    let target = BUILD_TARGET.unwrap_or("unknown");
    format!(
        "ELAI\n  Version          {VERSION}\n  Git SHA          {git_sha}\n  Target           {target}\n  Build date       {DEFAULT_DATE}"
    )
}

pub(crate) fn render_export_text(session: &Session) -> String {
    let mut lines = vec!["# Conversation Export".to_string(), String::new()];
    for (index, message) in session.messages.iter().enumerate() {
        let role = match message.role {
            MessageRole::System => "system",
            MessageRole::User => "user",
            MessageRole::Assistant => "assistant",
            MessageRole::Tool => "tool",
        };
        lines.push(format!("## {}. {role}", index + 1));
        for block in &message.blocks {
            match block {
                ContentBlock::Text { text } => lines.push(text.clone()),
                ContentBlock::ToolUse { id, name, input } => {
                    lines.push(format!("[tool_use id={id} name={name}] {input}"));
                }
                ContentBlock::ToolResult {
                    tool_use_id,
                    tool_name,
                    output,
                    is_error,
                } => {
                    lines.push(format!(
                        "[tool_result id={tool_use_id} name={tool_name} error={is_error}] {output}"
                    ));
                }
            }
        }
        lines.push(String::new());
    }
    lines.join("\n")
}

pub(crate) fn default_export_filename(session: &Session) -> String {
    let stem = session
        .messages
        .iter()
        .find_map(|message| match message.role {
            MessageRole::User => message.blocks.iter().find_map(|block| match block {
                ContentBlock::Text { text } => Some(text.as_str()),
                _ => None,
            }),
            _ => None,
        })
        .map_or("conversation", |text| {
            text.lines().next().unwrap_or("conversation")
        })
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|part| !part.is_empty())
        .take(8)
        .collect::<Vec<_>>()
        .join("-");
    let fallback = if stem.is_empty() {
        "conversation"
    } else {
        &stem
    };
    format!("{fallback}.txt")
}

pub(crate) fn resolve_export_path(
    requested_path: Option<&str>,
    session: &Session,
) -> Result<PathBuf, anyhow::Error> {
    let cwd = env::current_dir()?;
    let file_name =
        requested_path.map_or_else(|| default_export_filename(session), ToOwned::to_owned);
    let final_name = if Path::new(&file_name)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("txt"))
    {
        file_name
    } else {
        format!("{file_name}.txt")
    };
    Ok(cwd.join(final_name))
}

// ---------------------------------------------------------------------------
// System prompt / plugin helpers
// ---------------------------------------------------------------------------

/// H-31 (SCHEMA_VERSION 240): the stable marker that opens the user-memory section appended to the
/// assembled system prompt. The contract/smoke test asserts presence/absence of this tag in the
/// assembled `Vec<String>` (R4: measure the real assembled prompt, not stdout).
pub(crate) const MEMORY_SECTION_TAG: &str = "# User decisions (remembered)";

/// Maximum number of user-decisions surfaced into the prompt (one line each; bounded so the slot
/// cannot crowd out the local ~25-32K window).
const MEMORY_RETRIEVAL_K: usize = 8;

/// Build the system prompt for an assembling role. H-31 threads `role` so the user-memory section
/// (the activated `get_relevant_facts` read-path) is appended ONLY for the role allowlist
/// {AgentRole::Executor} AND `ELAI_USER_MEMORY_ENABLE=1`. For every reviewer/auditor/critic role
/// the section is structurally ABSENT (the Personalization-Trap firewall, fail-CLOSED).
pub(crate) fn build_system_prompt(skip_instruction_files: bool) -> Result<Vec<String>, anyhow::Error> {
    // Default to the Executor role: the live CLI chat turn IS the executor/primary turn.
    build_system_prompt_for_role(skip_instruction_files, elai_orchestrator::AgentRole::Executor)
}

/// Role-gated variant of [`build_system_prompt`]. The user-memory section is appended IFF the role
/// is on the firewall allowlist AND the flag is on. A non-allowlisted role NEVER receives any
/// user-memory line (the section is not merely truncated; it is never assembled).
pub(crate) fn build_system_prompt_for_role(
    skip_instruction_files: bool,
    role: elai_orchestrator::AgentRole,
) -> Result<Vec<String>, anyhow::Error> {
    let mut sections = load_system_prompt(
        env::current_dir()?,
        DEFAULT_DATE,
        env::consts::OS,
        "unknown",
    )?;
    if skip_instruction_files {
        // When rules enforce coding standards programmatically, strip instruction-file
        // sections to save context budget. Instruction sections start with "# " headers
        // that reference loaded config/CLAUDE.md content.
        sections.retain(|s| !s.starts_with("# Claude instructions"));
    }
    // I-32: strip narrative persona and inject schema-first role contract text for
    // factual/code roles (Executor, Researcher, ContextCompiler, Designer, MemoryWriter).
    // Alignment-dependent roles (Auditor, IdeaCritic, InjectionClassifier) retain their
    // original framing (PersonaRetained). Grounded per IC-17: narrative persona harms
    // MMLU -3.6..-5.3pp and MT-Bench coding -0.65 for factual/code roles.
    let store = RoleContractStore::default();
    apply_to_system_prompt(&mut sections, role, &store);
    // I-36: after role_contracts apply, look up a compiled variant for this (role, backend)
    // pair. The store is loaded from project_root/.elai/cache/compiled_prompts.msgpack.
    // In Tier-1 the backend is passed as "anthropic_api" (the primary cloud lane per the
    // observable_effect trigger); the full (role, PerRoleRuntime-derived backend) key is
    // Tier-2 when PerRoleRuntime is threaded in here. If a promoted variant exists it
    // replaces sections[0] (the full system-prompt body), which the caller joins and
    // places in CompletionConfig::system; the adapter then optionally overrides it via
    // CompletionConfig::compiled_system_override (Option A seam). The kill-switch
    // ELAI_PROMPT_COMPILE_DISABLE=1 makes lookup() return None, keeping this a no-op.
    // D-030 / R-8: this call is the production caller for CompiledPromptStore::lookup.
    if let Ok(project_root) = env::current_dir() {
        let compiled_store = CompiledPromptStore::load_standard(&project_root);
        // Use a normalized snake_case role string as the key (consistent with AgentRole::as_str).
        let role_key = format!("{:?}", role).to_ascii_lowercase();
        if let Some(variant) = compiled_store.lookup(&role_key, "anthropic_api") {
            tracing::debug!(
                target = "compiled_prompt_store",
                role = %role_key,
                backend = "anthropic_api",
                variant_hash = %&variant.variant_hash[..16.min(variant.variant_hash.len())],
                "I-36: substituting compiled prompt variant"
            );
            if sections.is_empty() {
                sections.push(variant.compiled_text.clone());
            } else {
                sections[0] = variant.compiled_text.clone();
            }
        }
    }
    // H-31: append the user-memory section iff the firewall allows this role AND the flag is on.
    // No SSB available at construction time — callers that have self.ssb should use
    // assemble_user_memory_section_with_ssb directly when they want SSB emission.
    if let Some(section) = assemble_user_memory_section(role, None) {
        sections.push(section);
    }
    Ok(sections)
}

/// Assemble the user-memory prompt section for an assembling role, or `None` when the firewall
/// denies the role, the flag is off, or there are no current user-decisions to surface.
///
/// THE FIREWALL (load-bearing): the compile-time allowlist
/// `elai_orchestrator::user_memory_firewall::agent_role_gets_user_memory` is checked FIRST. A
/// denied role returns `None` immediately -- `get_relevant_facts` is never even called, so no
/// user-memory can leak into a reviewer/auditor/critic prompt. The superseded-row hard filter is
/// structural: `get_relevant_facts` pulls `list_current` (`valid_to_ms IS NULL`), so a superseded
/// decision is NEVER a candidate and is never narrated `[OUTDATED]`.
///
/// B-3 (R4 load-bearing wiring): when `ssb` is `Some`, each Tier-A citation exclusion emits a
/// real `fact_validation_failed` SSB row (the off-agent artifact). When `ssb` is `None` (e.g. at
/// construction time before the runtime is built), exclusions are logged only via `tracing::warn`.
/// The firewall ensures auditor/critic/adversary paths inject ZERO Tier-A and emit nothing.
pub(crate) fn assemble_user_memory_section(
    role: elai_orchestrator::AgentRole,
    ssb: Option<&Arc<elai_orchestrator::ssb::Ssb>>,
) -> Option<String> {
    use crate::dkg_citation_validator::DkgCitationValidator;
    use crate::user_memory_capture::{user_memory_enabled, DEFAULT_USER_ID};
    use elai_core::clock::{ClockProvider, OsClock};
    use elai_core::paths::memory_dir_path;
    use elai_memory::{
        get_authorized_facts, get_relevant_facts, FactStore, FactType, NullSemanticValidator,
        NullValidator, RetrievalFilter, ValidationScope,
    };

    // FIREWALL FIRST: a non-allowlisted role gets ZERO user-memory (structural, fail-closed).
    // Tier-A facts are also excluded here (same firewall — auditor/critic/adversary get ZERO
    // user-memory of any tier, per arXiv:2603.00024 Personalization-Trap).
    if !elai_orchestrator::user_memory_firewall::agent_role_gets_user_memory(role) {
        return None;
    }
    // R9 flag: default-OFF.
    if !user_memory_enabled() {
        return None;
    }

    let project_root = env::current_dir().ok()?;
    let facts_path = memory_dir_path(&project_root).join("memory_facts.sqlite");
    if !facts_path.exists() {
        return None;
    }
    let store = FactStore::open(&facts_path).ok()?;
    let now_ms = OsClock.now_ms();

    let mut lines = vec![MEMORY_SECTION_TAG.to_string()];
    let mut any_facts = false;

    // --- Tier-B (Preference-class): voice/tool/pattern preferences ---
    // J-21: route Tier-B through the retrieval-time authorization gate
    // (`get_authorized_facts`) rather than the bare H-32 retrieval. This makes
    // the J-21 authorization+semantic chain genuinely reachable from the live
    // CLI memory-assembly entrypoint.
    //
    // NON-REGRESSING for Tier-B: `authorize_for_injection` -> `revalidate_semantic`
    // short-circuits Preference-class (Tier-B) facts to `Valid`/`Authorized`
    // BEFORE either validator is consulted (semantic_validation.rs:284-286 +
    // retrieval_authorization.rs:342-344; proven by the `tier_b_short_circuits_to_valid`
    // test, which returns Valid even with a failing semantic validator). So the
    // fail-closed `NullValidator` + `NullSemanticValidator` never run for these
    // facts: the authorized set equals the H-32 candidate set and the exclusions
    // list is always empty here. Quarantine is `None` (no write-quarantine store
    // on the read path).
    let tier_b_existence = NullValidator;
    let tier_b_semantic = NullSemanticValidator;
    let tier_b_scope = ValidationScope::at_head("", "");
    let tier_b_filter = RetrievalFilter {
        fact_type: Some(FactType::Preference),
        ..RetrievalFilter::default()
    };
    if let Ok((tier_b_facts, _)) = get_authorized_facts(
        &store,
        DEFAULT_USER_ID,
        &tier_b_filter,
        MEMORY_RETRIEVAL_K,
        now_ms,
        &tier_b_existence,
        &tier_b_semantic,
        &tier_b_scope,
        None,
    ) {
        if !tier_b_facts.is_empty() {
            any_facts = true;
            lines.push(
                "Durable work preferences the user asked you to remember. Honour them; do not re-ask."
                    .to_string(),
            );
            for fact in &tier_b_facts {
                let detail = fact
                    .value
                    .get("raw")
                    .and_then(serde_json::Value::as_str)
                    .or_else(|| fact.value.get("subject").and_then(serde_json::Value::as_str))
                    .unwrap_or(&fact.key);
                lines.push(format!("  - {detail}"));
            }
        }
    }

    // --- Tier-A (DomainKnowledge + Mistake): code-grounded project conventions ---
    // H-32 validity gate: Stale/Unknown citations are HARD-EXCLUDED; the caller
    // surfaces a confirmation instead of silently steering the model on a stale premise.
    // The `fact_validation_failed` SSB rows are emitted by the live_cli tick (maintenance
    // path) where SSB is available; here we just emit a tracing warning per exclusion.
    //
    // One validator per Tier-A retrieval call: `DkgCitationValidator::open` opens a
    // fresh read-only cache each time (~1-5 ms on SSD). Moving the owned value into
    // the Box avoids lifetime issues (no shared borrow across loop iterations).
    for tier_a_type in [FactType::DomainKnowledge, FactType::Mistake] {
        // B-2 fail-closed: when the DKG index is unavailable, use NullValidator
        // (returns Unknown for every Tier-A fact) so the gate hard-excludes all
        // Tier-A facts rather than injecting them unchecked (R2: missing evidence
        // is FAIL, not SKIP).
        let citation_validator: Box<dyn elai_memory::CitationValidator> =
            DkgCitationValidator::open(&project_root)
                .map(|v| Box::new(v) as Box<dyn elai_memory::CitationValidator>)
                .unwrap_or_else(|| Box::new(NullValidator));
        let tier_a_filter = RetrievalFilter {
            fact_type: Some(tier_a_type),
            citation_validator: Some(citation_validator),
            ..RetrievalFilter::default()
        };
        if let Ok((tier_a_facts, exclusions)) =
            get_relevant_facts(&store, DEFAULT_USER_ID, &tier_a_filter, MEMORY_RETRIEVAL_K, now_ms)
        {
            // B-3 (R4): emit a fact_validation_failed SSB row per exclusion when the
            // SSB handle is available. The off-agent artifact is the load-bearing wiring
            // proof (the rename during a real model-dispatch turn appends the row).
            // Firewall guarantees only Executor-role calls reach here (auditor/critic/
            // adversary return None above), so zero Tier-A leaks into reviewer prompts.
            for excl in &exclusions {
                tracing::warn!(
                    target = "memory.citation_validity",
                    fact_id = %excl.fact_id,
                    reason = excl.reason,
                    evidence_blake3 = %excl.evidence_blake3,
                    "H-32: Tier-A citation {} — excluded from context slice; \
                     confirmation required before re-use",
                    excl.reason
                );
                if let Some(ssb_ref) = ssb {
                    // `citation` carries the blake3 fingerprint as the best available
                    // identifier when the full dkg: URI is not preserved in the exclusion
                    // struct. The load-bearing field is `evidence_blake3`; `citation` is
                    // informational (correlates back to the TTL-tick path which carries
                    // the full URI).
                    let citation = if excl.evidence_blake3.is_empty() {
                        format!("fact:{}", excl.fact_id)
                    } else {
                        format!("blake3:{}", excl.evidence_blake3)
                    };
                    let payload = elai_orchestrator::ssb::SsbPayload::FactValidationFailed(
                        elai_orchestrator::ssb::FactValidationFailedPayload {
                            fact_id: excl.fact_id.clone(),
                            citation,
                            reason: excl.reason.to_string(),
                            evidence_blake3: excl.evidence_blake3.clone(),
                        },
                    );
                    let ssb_arc = Arc::clone(ssb_ref);
                    std::thread::spawn(move || {
                        if let Ok(rt) = tokio::runtime::Builder::new_current_thread()
                            .enable_all()
                            .build()
                        {
                            let _ = rt.block_on(ssb_arc.write_payload(
                                "report_fns.fact_validation_failed",
                                0,
                                payload,
                                Vec::new(),
                            ));
                        }
                    });
                }
            }
            if !tier_a_facts.is_empty() {
                any_facts = true;
                let label = if tier_a_type == FactType::Mistake {
                    "Recurring mistakes/footguns to avoid in this project:"
                } else {
                    "Project code conventions (grounded in current code):"
                };
                lines.push(label.to_string());
                for fact in &tier_a_facts {
                    let detail = fact
                        .value
                        .get("raw")
                        .and_then(serde_json::Value::as_str)
                        .or_else(|| fact.value.get("subject").and_then(serde_json::Value::as_str))
                        .unwrap_or(&fact.key);
                    lines.push(format!("  - {detail}"));
                }
            }
        }
    }

    if !any_facts {
        return None;
    }
    Some(lines.join("\n"))
}

/// Load seed rules from data/rules/*.toml, compile predicates via the engine.
pub(crate) fn load_seed_rules(
    engine: &DefaultRuleEngine,
) -> Vec<(Rule, CompiledPredicate)> {
    #[derive(Deserialize)]
    struct SeedFile {
        pub(crate) rules: Vec<Rule>,
    }

    let rules_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../data/rules");
    let entries = match fs::read_dir(&rules_dir) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("[rules] seed directory not found ({}): {e}", rules_dir.display());
            return Vec::new();
        }
    };

    let mut compiled = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().map_or(true, |ext| ext != "toml") {
            continue;
        }
        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("[rules] failed to read {}: {e}", path.display());
                continue;
            }
        };
        let seed: SeedFile = match toml::from_str(&content) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("[rules] failed to parse {}: {e}", path.display());
                continue;
            }
        };
        for rule in seed.rules {
            match engine.compile_predicate(&rule.predicate_json) {
                Ok(cp) => compiled.push((rule, cp)),
                Err(e) => {
                    eprintln!("[rules] failed to compile rule {}: {e}", rule.rule_id);
                }
            }
        }
    }
    eprintln!("[rules] loaded {} seed rules from {}", compiled.len(), rules_dir.display());
    compiled
}

pub(crate) fn build_runtime_plugin_state() -> Result<RuntimePluginState, anyhow::Error> {
    let cwd = env::current_dir()?;
    let loader = ConfigLoader::default_for(&cwd);
    let runtime_config = loader.load()?;
    build_runtime_plugin_state_with_loader(&cwd, &loader, &runtime_config)
}

pub(crate) fn build_runtime_plugin_state_with_loader(
    cwd: &Path,
    loader: &ConfigLoader,
    runtime_config: &elai_orchestrator::RuntimeConfig,
) -> Result<RuntimePluginState, anyhow::Error> {
    let plugin_manager = build_plugin_manager(cwd, loader, runtime_config);
    let plugin_registry = plugin_manager.plugin_registry()?;
    let plugin_hook_config =
        runtime_hook_config_from_plugin_hooks(plugin_registry.aggregated_hooks()?);
    let feature_config = runtime_config
        .feature_config()
        .clone()
        .with_hooks(runtime_config.hooks().merged(&plugin_hook_config));
    let (mcp_state, runtime_tools) = build_runtime_mcp_state(runtime_config)?;
    let tool_registry = GlobalToolRegistry::with_plugin_tools(plugin_registry.aggregated_tools()?)
        .map_err(|e| anyhow::anyhow!(e))?
        .with_runtime_tools(runtime_tools)
        .map_err(|e| anyhow::anyhow!(e))?;
    Ok(RuntimePluginState {
        feature_config,
        tool_registry,
        plugin_registry,
        mcp_state,
    })
}

pub(crate) fn build_plugin_manager(
    cwd: &Path,
    loader: &ConfigLoader,
    runtime_config: &elai_orchestrator::RuntimeConfig,
) -> PluginManager {
    let plugin_settings = runtime_config.plugins();
    let mut plugin_config = PluginManagerConfig::new(loader.config_home().to_path_buf());
    plugin_config.enabled_plugins = plugin_settings.enabled_plugins().clone();
    plugin_config.external_dirs = plugin_settings
        .external_directories()
        .iter()
        .map(|path| resolve_plugin_path(cwd, loader.config_home(), path))
        .collect();
    plugin_config.install_root = plugin_settings
        .install_root()
        .map(|path| resolve_plugin_path(cwd, loader.config_home(), path));
    plugin_config.registry_path = plugin_settings
        .registry_path()
        .map(|path| resolve_plugin_path(cwd, loader.config_home(), path));
    plugin_config.bundled_root = plugin_settings
        .bundled_root()
        .map(|path| resolve_plugin_path(cwd, loader.config_home(), path));
    PluginManager::new(plugin_config)
}

pub(crate) fn resolve_plugin_path(cwd: &Path, config_home: &Path, value: &str) -> PathBuf {
    let path = PathBuf::from(value);
    if path.is_absolute() {
        path
    } else if value.starts_with('.') {
        cwd.join(path)
    } else {
        config_home.join(path)
    }
}

pub(crate) fn runtime_hook_config_from_plugin_hooks(hooks: PluginHooks) -> elai_orchestrator::RuntimeHookConfig {
    elai_orchestrator::RuntimeHookConfig::new(
        hooks.pre_tool_use,
        hooks.post_tool_use,
        hooks.post_tool_use_failure,
    )
}

#![allow(
    dead_code,
    unused_imports,
    unused_variables,
    clippy::unneeded_struct_pattern,
    clippy::unnecessary_wraps,
    clippy::unused_self
)]
mod classifier;
mod commands;
mod dashboard;
mod dep_audit;
mod doctor;
mod handoff;
mod indexing;
mod init;
mod input;
mod render;
mod runtime_config;
mod repl;
mod remote_sidecar;
mod sidecar;
mod sidecar_hyde;

mod args;
mod auth;
mod help;
mod live_cli;
mod symbol_graph;
mod progress;
mod resume;
mod run_subcommand;
mod runtime_builder;
mod runtime_client;
mod runtime_cmd;
mod tool_executor;
mod tool_formatting;

#[cfg(test)]
mod cli_tests;
#[cfg(test)]
mod sandbox_report_tests;

use std::collections::BTreeSet;
use std::env;
use std::pin::Pin;
use std::future::Future;
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub(crate) use crate::args::*;
pub(crate) use crate::auth::*;
pub(crate) use crate::live_cli::*;
pub(crate) use crate::progress::*;
pub(crate) use crate::resume::*;
pub(crate) use crate::runtime_builder::*;
pub(crate) use crate::runtime_client::*;
pub(crate) use crate::tool_executor::*;
pub(crate) use crate::tool_formatting::*;
pub(crate) use crate::help::*;
pub(crate) use crate::commands::{SlashCommand, resume_supported_slash_commands};

pub(crate) const DEFAULT_MODEL: &str = "claude-opus-4-6";
pub(crate) fn max_tokens_for_model(model: &str) -> u32 {
    if model.contains("opus") {
        32_000
    } else {
        64_000
    }
}
pub(crate) const TOKEN_CHARS_PER_TOKEN: usize = 4;
pub(crate) const BUDGET_OUTPUT_RESERVE: usize = 4096;

pub(crate) const FENCE_LANG_MAP: &[(&str, &str)] = &[
    ("rust", "rust"), ("rs", "rust"),
    ("python", "python"), ("py", "python"),
    ("typescript", "typescript"), ("ts", "typescript"),
    ("go", "go"), ("golang", "go"),
    ("java", "java"),
    ("csharp", "c_sharp"), ("cs", "c_sharp"), ("c#", "c_sharp"),
    ("ruby", "ruby"), ("rb", "ruby"),
];

pub(crate) fn shortcoder_compress_text(text: &str, sc: &elai_knowledge::ShortCoder, saved: &mut usize) -> usize {
    if !text.contains("```") {
        return text.len();
    }
    let mut effective_chars = 0usize;
    let mut pos = 0;
    while pos < text.len() {
        if let Some(fence_start) = text[pos..].find("```") {
            let abs_fence_start = pos + fence_start;
            effective_chars += fence_start;
            let after_ticks = abs_fence_start + 3;
            let line_end = text[after_ticks..].find('\n')
                .map(|i| after_ticks + i)
                .unwrap_or(text.len());
            let lang_tag = text[after_ticks..line_end].trim().to_lowercase();
            let code_start = if line_end < text.len() { line_end + 1 } else { line_end };
            let closing = text[code_start..].find("\n```").map(|i| code_start + i);
            if let Some(close_pos) = closing {
                let code_content = &text[code_start..close_pos];
                let fence_end = close_pos + 4;
                let after_close = text[fence_end..].find('\n')
                    .map(|i| fence_end + i + 1)
                    .unwrap_or(text.len());
                let ts_lang = FENCE_LANG_MAP.iter()
                    .find(|(tag, _)| *tag == lang_tag)
                    .map(|(_, lang)| *lang);
                if let Some(lang) = ts_lang {
                    match sc.simplify(code_content, lang) {
                        Ok(result) if result.was_simplified() => {
                            let original_block_len = after_close - abs_fence_start;
                            let simplified_block_len = 3 + lang_tag.len() + 1
                                + result.simplified.len() + 1 + 3;
                            if simplified_block_len < original_block_len {
                                *saved += original_block_len - simplified_block_len;
                            }
                            effective_chars += simplified_block_len;
                        }
                        _ => {
                            effective_chars += after_close - abs_fence_start;
                        }
                    }
                } else {
                    effective_chars += after_close - abs_fence_start;
                }
                pos = after_close;
            } else {
                effective_chars += text.len() - abs_fence_start;
                pos = text.len();
            }
        } else {
            effective_chars += text.len() - pos;
            pos = text.len();
        }
    }
    effective_chars
}

pub(crate) const DEFAULT_DATE: &str = "2026-03-31";
pub(crate) const DEFAULT_OAUTH_CALLBACK_PORT: u16 = 4545;
pub(crate) const VERSION: &str = env!("CARGO_PKG_VERSION");
pub(crate) const BUILD_TARGET: Option<&str> = option_env!("TARGET");
pub(crate) const GIT_SHA: Option<&str> = option_env!("GIT_SHA");
pub(crate) const INTERNAL_PROGRESS_HEARTBEAT_INTERVAL: Duration = Duration::from_secs(3);
pub(crate) const SESSION_HANDOFF_THRESHOLD: f64 = 0.85;
pub(crate) const PLANS_DIR: &str = "plans";
pub(crate) const PRIMARY_SESSION_EXTENSION: &str = "jsonl";
pub(crate) const OLLAMA_DEFAULT_URL: &str = "http://localhost:11434";
pub(crate) const DEGRADED_PROMPT_PREFIX: &str = "[degraded] using ollama fallback";
pub(crate) const OLLAMA_PROBE_TIMEOUT: Duration = Duration::from_secs(2);
pub(crate) const KNOWLEDGE_TOP_K: usize = 5;
pub(crate) const EMPTY_TIER_TOML: &str = "\
[tier1]\ndomains = []\npatterns = []\n\
[tier2]\ndomains = []\nmin_so_score = 5\n\
[tier4_blocklist]\ndomains = []\npatterns = []\n";

pub(crate) struct CharEstimatorProvider;

impl elai_providers::LlmProvider for CharEstimatorProvider {
    fn complete<'a>(
        &'a self,
        _prompt: &'a str,
        _config: &'a elai_providers::OllamaConfig,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<String>, elai_providers::ApiError>> + Send + 'a>> {
        Box::pin(async { Err(elai_providers::ApiError::Auth("CharEstimatorProvider does not support completion".into())) })
    }

    fn embed<'a>(
        &'a self,
        _text: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<f32>, elai_providers::ApiError>> + Send + 'a>> {
        Box::pin(async { Err(elai_providers::ApiError::Auth("CharEstimatorProvider does not support embedding".into())) })
    }

    fn token_count(&self, text: &str) -> Result<usize, elai_providers::ApiError> {
        Ok(text.len() / TOKEN_CHARS_PER_TOKEN)
    }

    fn token_count_estimate(&self, text: &str) -> usize {
        text.len() / TOKEN_CHARS_PER_TOKEN
    }
}

pub(crate) const LEGACY_SESSION_EXTENSION: &str = "json";
pub(crate) const LATEST_SESSION_REFERENCE: &str = "latest";
pub(crate) const SESSION_REFERENCE_ALIASES: &[&str] = &[LATEST_SESSION_REFERENCE, "last", "recent"];
pub(crate) const CLI_OPTION_SUGGESTIONS: &[&str] = &[
    "--help",
    "-h",
    "--version",
    "-V",
    "--model",
    "--output-format",
    "--permission-mode",
    "--dangerously-skip-permissions",
    "--allowedTools",
    "--allowed-tools",
    "--resume",
    "--print",
    "-p",
];

pub(crate) type AllowedToolSet = BTreeSet<String>;
pub(crate) type RuntimePluginStateBuildOutput = (
    Option<Arc<Mutex<live_cli::RuntimeMcpState>>>,
    Vec<elai_tools::RuntimeToolDefinition>,
);

fn main() {
    if let Err(error) = run() {
        let message = error.to_string();
        if message.contains("`elai--help`") {
            eprintln!("error: {message}");
        } else {
            eprintln!(
                "error: {message}\n\nRun `elai--help` for usage."
            );
        }
        std::process::exit(1);
    }
}

fn run() -> Result<(), anyhow::Error> {
    let args: Vec<String> = env::args().skip(1).collect();
    match parse_args(&args).map_err(|e| anyhow::anyhow!(e))? {
        CliAction::DumpManifests { output_format } => auth::dump_manifests(output_format)?,
        CliAction::BootstrapPlan { output_format } => auth::print_bootstrap_plan(output_format)?,
        CliAction::Agents { args, output_format } => LiveCli::print_agents(args.as_deref(), output_format)?,
        CliAction::Mcp { args, output_format } => LiveCli::print_mcp(args.as_deref(), output_format)?,
        CliAction::Runtime { action, role, target, output_format: _ } => {
            // B-37D Phase 4M: text-only output for now; JSON shaping deferred to
            // when RuntimeOrchestrator::health_snapshot lands (Wave-1B).
            LiveCli::print_runtime(action.as_deref(), role.as_deref(), target.as_deref())?;
        }
        CliAction::Skills { args, output_format } => LiveCli::print_skills(args.as_deref(), output_format)?,
        CliAction::PrintSystemPrompt { cwd, date, output_format } => print_system_prompt(cwd, date, output_format)?,
        CliAction::Version { output_format } => print_version(output_format)?,
        CliAction::ResumeSession { session_path, commands, output_format } => resume_session(&session_path, &commands, output_format),
        CliAction::Status { model, permission_mode, output_format } => live_cli::print_status_snapshot(&model, permission_mode, output_format)?,
        CliAction::Sandbox { output_format } => live_cli::print_sandbox_status_snapshot(output_format)?,
        CliAction::Prompt { prompt, model, output_format, allowed_tools, permission_mode, backend } =>
            LiveCli::new_prompt(model, true, allowed_tools, permission_mode, backend)?
                .run_turn_with_output(&prompt, output_format)?,
        CliAction::Login { output_format } => run_login(output_format)?,
        CliAction::Logout { output_format } => run_logout(output_format)?,
        CliAction::Doctor { output_format } => doctor::run_doctor(output_format)?,
        CliAction::Init { output_format } => live_cli::run_init(output_format)?,
        CliAction::Repl { model, allowed_tools, permission_mode, export_jsonl } =>
            repl::run_repl(model, allowed_tools, permission_mode, export_jsonl)?,
        CliAction::DepAudit { project_dir, output_format } => doctor::run_dep_audit_command(&project_dir, output_format)?,
        CliAction::Local { port } => repl::run_local_repl(port)?,
        CliAction::Dashboard => dashboard::run_dashboard()?,
        // Phase A1a: production caller of `start_task_runner`. Pinned by
        // `crates/contracts/tests/task_runner_chokepoint_contract.rs`.
        CliAction::Run { project_root } => {
            run_subcommand::run_task_runner_subcommand(run_subcommand::RunArgs { project_root })?
        }
        CliAction::RemoteSidecar => remote_sidecar::run_remote_sidecar()?,
        CliAction::HelpTopic(topic) => live_cli::print_help_topic(topic),
        CliAction::Help { output_format } => help::print_help(output_format)?,
        // B-29 Phase 4: symbol graph CLI subcommands.
        CliAction::SymbolGraphBuild { project_root } => {
            symbol_graph::run_symbol_graph_build(project_root)?
        }
        CliAction::SymbolGraphStats { project_root } => {
            symbol_graph::run_symbol_graph_stats(project_root)?
        }
    }
    Ok(())
}

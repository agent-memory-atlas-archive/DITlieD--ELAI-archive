#![allow(dead_code, unused_imports, unused_variables, unused_mut, clippy::unneeded_struct_pattern, clippy::unnecessary_wraps, clippy::unused_self)]

use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::env;
use std::fs;
use std::io::{self, Read, Write};
use std::net::TcpListener;
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, UNIX_EPOCH};
use std::pin::Pin;
use std::future::Future;

use elai_core::{ClockProvider, EntropySource, OsEntropy};
use elai_providers::{
    oauth_token_is_expired, resolve_startup_auth_source, AnthropicClient, AuthSource,
    ContentBlockDelta, InputContentBlock, InputMessage, MessageRequest, MessageResponse,
    OutputContentBlock, PromptCache,
    StreamEvent as ApiStreamEvent, SystemBlock, ToolChoice, ToolDefinition, ToolResultContentBlock,
};
use elai_plugins::{PluginHooks, PluginManager, PluginManagerConfig, PluginRegistry};
use crossterm::{cursor::MoveToColumn, execute, terminal::{Clear, ClearType}};
use elai_orchestrator::{
    clear_oauth_credentials, format_usd, generate_pkce_pair, generate_state,
    load_oauth_credentials, load_system_prompt, parse_oauth_callback_request_target,
    pricing_for_model, resolve_sandbox_status, save_oauth_credentials, ApiClient, ApiRequest,
    AssistantEvent, BackendConfig, BackendMode, CompactionConfig, ConfigLoader, ConfigSource,
    ContentBlock, ConversationMessage, ConversationRuntime, McpServerManager, McpTool, MessageRole,
    ModelPricing, OAuthAuthorizationRequest, OAuthConfig, OAuthTokenExchangeRequest,
    PermissionMode, PermissionPolicy, ProjectContext, PromptCacheEvent, ResolvedPermissionMode,
    RuntimeError, Session, TokenUsage, ToolError, ToolExecutor, UsageTracker,
};
use serde::Deserialize;
use serde_json::json;
use elai_rules::predicate::from_json;
use elai_rules::rule_engine::{CompiledPredicate, DefaultRuleEngine, EvaluationContext, FsmPhaseBit, Rule, RuleEngine};
use elai_knowledge::{TreeSitterAstEvaluator, ShortCoder, DkgQuerierAdapter, DkgEngine, detect_project_profile, populate_tier3, assemble_telescope};
use elai_knowledge::pipeline::{QueryPipeline, PipelineStage};
use elai_knowledge::bm25::Bm25Index;
use elai_knowledge::catrag::{CatRag, CatRagConfig};
use elai_knowledge::arag::{AragOrchestrator, AragConfig};
use elai_knowledge::vector::OrtVectorModule;
use elai_knowledge::vector::ort_session::OrtSession;
use elai_knowledge::vector::indexer::VectorIndexer;
use elai_knowledge::vector::TEXT_VEC_DIM;
use elai_knowledge::startup::{startup_readiness_check, StartupConfig};
use elai_knowledge::degradation::DegradationCascade;
use elai_knowledge::reranker::{CrossEncoderReranker, RerankerConfig};
use elai_knowledge::splade::{SpladeExpander, SpladeConfig};
use elai_knowledge::index_version::{check_and_upgrade, LogNotifier};
use elai_orchestrator::indexing_service::IndexingService;
use elai_core::paths::{
    dkg_db_path as core_dkg_db_path, models_dir_path as core_models_dir_path,
    rules_db_path as core_rules_db_path, vectors_db_path as core_vectors_db_path,
    ELAI_DATA_DIR_NAME,
};
use elai_rules::stubs::EmptyDkgQuerier;
use elai_tools::{GlobalToolRegistry, RuntimeToolDefinition, ToolSearchOutput, mvp_tool_specs};

use crate::commands::{
    handle_agents_slash_command, handle_mcp_slash_command, handle_mcp_slash_command_json,
    handle_plugins_slash_command, handle_skills_slash_command, handle_skills_slash_command_json,
    render_slash_command_help, resume_supported_slash_commands, slash_command_specs,
    validate_slash_command_input, SlashCommand,
};
use crate::init::initialize_repo;
use crate::render::{MarkdownStreamState, Spinner, TerminalRenderer};
use crate::*;

pub(crate) struct RuntimeClient {
    pub(crate) runtime: tokio::runtime::Runtime,
    pub(crate) client: AnthropicClient,
    pub(crate) backend: BackendMode,
    pub(crate) model: String,
    pub(crate) enable_tools: bool,
    pub(crate) emit_output: bool,
    pub(crate) allowed_tools: Option<AllowedToolSet>,
    pub(crate) tool_registry: GlobalToolRegistry,
    pub(crate) progress_reporter: Option<InternalPromptProgressReporter>,
    pub(crate) tool_policy: ToolPolicy,
    pub(crate) budget_policy: BudgetPolicy,
    pub(crate) fsm: Option<Arc<elai_orchestrator::Fsm>>,
    pub(crate) rule_engine: DefaultRuleEngine,
    pub(crate) compiled_rules: Vec<(Rule, CompiledPredicate)>,
    pub(crate) phase_slot_tokens: usize,
    /// Backend routing config retained for per-turn `build_dispatch_context` calls.
    pub(crate) backend_config: Option<BackendConfig>,
}

pub(crate) fn resolve_cli_auth_source() -> Result<AuthSource, anyhow::Error> {
    Ok(resolve_startup_auth_source(|| {
        let cwd = env::current_dir().map_err(elai_providers::ApiError::from)?;
        let config = ConfigLoader::default_for(&cwd).load().map_err(|error| {
            elai_providers::ApiError::Auth(format!("failed to load runtime OAuth config: {error}"))
        })?;
        Ok(config.oauth().cloned())
    })?)
}

impl RuntimeClient {
    pub(crate) fn current_fsm_state(&self) -> elai_orchestrator::FsmState {
        match &self.tool_policy {
            ToolPolicy::FsmFiltered { .. } => match &self.fsm {
                Some(fsm) => self.runtime.block_on(fsm.get_state()),
                None => elai_orchestrator::FsmState::Idle,
            },
            _ => elai_orchestrator::FsmState::Idle,
        }
    }

    pub(crate) fn resolve_tool_defs(&self) -> Option<Vec<ToolDefinition>> {
        if !self.enable_tools {
            return None;
        }
        match &self.tool_policy {
            ToolPolicy::Full => {
                Some(filter_tool_specs(&self.tool_registry, self.allowed_tools.as_ref()))
            }
            ToolPolicy::FsmFiltered { max_tools, .. } => {
                let fsm_state = self.current_fsm_state();
                if fsm_state == elai_orchestrator::FsmState::Idle {
                    return None;
                }
                let all_defs = filter_tool_specs(&self.tool_registry, self.allowed_tools.as_ref());
                let fsm_filtered: Vec<_> = all_defs
                    .into_iter()
                    .filter(|def| fsm_state.check_tool_allowed(&def.name).is_ok())
                    .collect();
                let capped = if fsm_filtered.len() > *max_tools {
                    let priority_map: BTreeMap<String, u8> = mvp_tool_specs()
                        .into_iter()
                        .map(|s| (s.name.to_string(), s.priority))
                        .collect();
                    let mut sorted = fsm_filtered;
                    sorted.sort_by_key(|def| {
                        priority_map.get(def.name.as_str()).copied().unwrap_or(u8::MAX)
                    });
                    sorted.into_iter().take(*max_tools).collect::<Vec<_>>()
                } else {
                    fsm_filtered
                };
                if capped.is_empty() { None } else { Some(capped) }
            }
            ToolPolicy::None => None,
        }
    }

    /// Evaluate seed rules against a tool use. Returns None if allowed,
    /// Some(reason) if blocked.
    pub(crate) fn evaluate_tool_rules(&self, tool_name: &str, tool_input: &str) -> Option<String> {
        use elai_rules::predicate::VerdictResult;
        use rustc_hash::FxHashMap;

        if self.compiled_rules.is_empty() {
            return None;
        }

        let fsm_phase = match self.current_fsm_state() {
            elai_orchestrator::FsmState::Planning => FsmPhaseBit::PLANNING,
            elai_orchestrator::FsmState::FeasibilityProbe => FsmPhaseBit::FEASIBILITY_PROBE,
            elai_orchestrator::FsmState::TestingFirst => FsmPhaseBit::TESTING_FIRST,
            elai_orchestrator::FsmState::Executing => FsmPhaseBit::EXECUTING,
            elai_orchestrator::FsmState::Auditing => FsmPhaseBit::AUDITING,
            elai_orchestrator::FsmState::Memorizing => FsmPhaseBit::MEMORIZING,
            elai_orchestrator::FsmState::Configuring => FsmPhaseBit::CONFIGURING,
            elai_orchestrator::FsmState::Researching => FsmPhaseBit::RESEARCHING,
            elai_orchestrator::FsmState::Idle
            | elai_orchestrator::FsmState::Complete => FsmPhaseBit::EXECUTING,
        };

        let ctx = EvaluationContext {
            action: tool_name,
            source: tool_input,
            file_path: "",
            fsm_phase,
            metric_values: FxHashMap::default(),
        };

        let verdicts = match self.rule_engine.evaluate(&self.compiled_rules, &ctx) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("[rules] evaluation error for tool {tool_name}: {e}");
                return None;
            }
        };
        eprintln!("[rules] evaluated {} rules for tool {tool_name}",
                  self.compiled_rules.len());

        let mut block_reason = None;
        for verdict in &verdicts {
            match &verdict.result {
                VerdictResult::Block => {
                    eprintln!("[rules] BLOCK: rule {} on tool {tool_name}: {}",
                              verdict.rule_id, verdict.evidence);
                    if block_reason.is_none() {
                        block_reason = Some(format!(
                            "Rule {} blocked tool {}: {}",
                            verdict.rule_id, tool_name, verdict.evidence
                        ));
                    }
                }
                VerdictResult::Warn => {
                    eprintln!("[rules] WARN: rule {} on tool {tool_name}: {}",
                              verdict.rule_id, verdict.evidence);
                }
                VerdictResult::Escalate(_target) => {
                    eprintln!("[rules] ESCALATE: rule {} on tool {tool_name}: {}",
                              verdict.rule_id, verdict.evidence);
                }
                VerdictResult::Allow => {}
            }
        }
        block_reason
    }
}

impl ApiClient for RuntimeClient {
    #[allow(clippy::too_many_lines)]
    fn stream(&mut self, request: ApiRequest) -> Result<Vec<AssistantEvent>, RuntimeError> {
        if let Some(progress_reporter) = &self.progress_reporter {
            progress_reporter.mark_model_phase();
        }

        // patch-1.32: run the orchestrator's build_dispatch_context before every
        // local-backend dispatch and intersect its filtered tool allow-list with
        // the tool defs we send to the sidecar. The build_dispatch_context call
        // now materially affects the request body (not just logs) — any tool
        // the dispatcher rejects is dropped before we serialize. On error we
        // degrade to the stream-local policy filter only.
        let dispatch_allowed_tool_names: Option<HashSet<String>> =
            if matches!(self.backend, BackendMode::Local) {
                self.backend_config.as_ref().and_then(|cfg| {
                    let fsm_state = self.current_fsm_state();
                    let specs = mvp_tool_specs();
                    let all_tools: Vec<String> =
                        specs.iter().map(|s| s.name.to_string()).collect();
                    let priority_map: BTreeMap<String, u8> = specs
                        .iter()
                        .map(|s| (s.name.to_string(), s.priority))
                        .collect();
                    // B-10 (DECISION-A / Task 7 parity): pass a real
                    // ModelCapabilityProfile so the REPL pathway uses the same
                    // per-tier slot weights as the TaskRunner pathway. Detected
                    // via substring matching on `self.model` per DECISION-F.
                    let repl_profile =
                        elai_orchestrator::context_budget::detect_model_capability_profile(
                            &self.model,
                            elai_core::VramTier::Vram12Gb,
                        );
                    match elai_orchestrator::build_dispatch_context(
                        fsm_state,
                        cfg,
                        &all_tools,
                        &priority_map,
                        Some(&repl_profile),
                        None,
                    ) {
                        Ok(ctx) => {
                            tracing::info!(
                                target: "cli::dispatch",
                                backend = %ctx.backend_mode,
                                fsm = ?ctx.fsm_state,
                                sidecar = %ctx.sidecar_url,
                                tools_allowed = ctx.filtered_tool_names.len(),
                                max_tools = ctx.max_tools,
                                "dispatch context built",
                            );
                            Some(ctx.filtered_tool_names.into_iter().collect())
                        }
                        Err(e) => {
                            tracing::warn!(
                                target: "cli::dispatch",
                                error = %e,
                                "build_dispatch_context failed; using stream-local filter",
                            );
                            None
                        }
                    }
                })
            } else {
                None
            };

        // Branch 1: tool filtering (policy-based), then intersect with the
        // dispatch-context allow-list so local-backend routing observes the
        // orchestrator's filter.
        let tool_defs = self.resolve_tool_defs().map(|defs| {
            if let Some(allowed) = &dispatch_allowed_tool_names {
                let filtered: Vec<_> = defs
                    .into_iter()
                    .filter(|def| allowed.contains(def.name.as_str()))
                    .collect();
                filtered
            } else {
                defs
            }
        }).filter(|defs| !defs.is_empty());

        // Branch 2: budget policy — compute real slot sizes, cap max_tokens
        let system_chars: usize = request.system_prompt.iter().map(|s| s.len()).sum();
        let slot_0 = system_chars / TOKEN_CHARS_PER_TOKEN;
        let slot_1: usize = self.compiled_rules.iter()
            .map(|(r, _)| r.predicate_json.len() / TOKEN_CHARS_PER_TOKEN)
            .sum();
        let model_max_tokens = max_tokens_for_model(&self.model);

        let effective_max_tokens = match &self.budget_policy {
            BudgetPolicy::Local { ceiling, fraction } => {
                // ELAI_LOCAL_MAX_CONTEXT env override for testing against
                // small local models (e.g. Qwen 3B ~25K, 7B ~25K) whose
                // effective context is far below the 131_072 default.
                let effective = std::env::var("ELAI_LOCAL_MAX_CONTEXT")
                    .ok()
                    .and_then(|s| s.parse::<usize>().ok())
                    .unwrap_or(ceiling * fraction / 100);
                match elai_orchestrator::context_budget::ContextBudget::compute(
                    effective, [slot_0, slot_1, self.phase_slot_tokens], BUDGET_OUTPUT_RESERVE,
                ) {
                    Ok(mut budget) => {
                        use elai_orchestrator::context_budget::ContextSlot;

                        // Set actual_tokens for fixed slots from measured char counts
                        budget.slots[ContextSlot::SystemPrompt as usize].actual_tokens = slot_0;
                        budget.slots[ContextSlot::Rules as usize].actual_tokens = slot_1;
                        budget.slots[ContextSlot::PhaseSpec as usize].actual_tokens = self.phase_slot_tokens;

                        // A-24: ShortCoder context compression on CodeContext slot.
                        // Apply AST-preserving simplification to code content in messages
                        // before token estimation. Reduces context token usage by ~15-30%.
                        let shortcoder = ShortCoder::new();
                        let mut shortcoder_saved: usize = 0;
                        let msg_chars: usize = request.messages.iter()
                            .flat_map(|m| m.blocks.iter())
                            .map(|block| match block {
                                ContentBlock::Text { text } => {
                                    shortcoder_compress_text(text, &shortcoder, &mut shortcoder_saved)
                                }
                                ContentBlock::ToolUse { input, .. } => input.len(),
                                ContentBlock::ToolResult { output, .. } => {
                                    shortcoder_compress_text(output, &shortcoder, &mut shortcoder_saved)
                                }
                            })
                            .sum();
                        if shortcoder_saved > 0 {
                            let saved_tokens = shortcoder_saved / TOKEN_CHARS_PER_TOKEN;
                            eprintln!(
                                "[shortcoder] compressed CodeContext: saved ~{saved_tokens} tokens ({shortcoder_saved} chars)"
                            );
                        }
                        let msg_tokens = msg_chars / TOKEN_CHARS_PER_TOKEN;
                        let code_ctx_alloc = budget.slots[ContextSlot::CodeContext as usize].allocated_tokens;
                        budget.slots[ContextSlot::CodeContext as usize].actual_tokens =
                            msg_tokens.min(code_ctx_alloc);

                        budget.recalculate_total_used();

                        // Build byte slices for cache breakpoint hashing
                        let system_bytes: Vec<u8> = request.system_prompt.join("\n\n").into_bytes();
                        let rules_bytes: Vec<u8> = self.compiled_rules.iter()
                            .flat_map(|(r, _)| r.predicate_json.bytes())
                            .collect();
                        let empty: Vec<u8> = Vec::new();
                        budget.update_breakpoints(&[
                            &system_bytes,         // SystemPrompt
                            &rules_bytes,          // Rules
                            &empty,                // PhaseSpec
                            &empty,                // RepoMap
                            &empty,                // CodeContext
                            &empty,                // SsbEntries
                            &empty,                // GrcHistory
                            &empty,                // OutputReserve
                        ]);

                        let output_slot = ContextSlot::OutputReserve as usize;
                        let budget_output = budget.slots[output_slot].allocated_tokens as u32;
                        eprintln!(
                            "[pipeline] budget: slots=[sys={},rules={},phase={},repo={},code={},ssb={},grc={},out={}] \
                             total_alloc={} total_used={} bp0={} bp1={} effective={}",
                            budget.slots[ContextSlot::SystemPrompt as usize].allocated_tokens,
                            budget.slots[ContextSlot::Rules as usize].allocated_tokens,
                            budget.slots[ContextSlot::PhaseSpec as usize].allocated_tokens,
                            budget.slots[ContextSlot::RepoMap as usize].allocated_tokens,
                            budget.slots[ContextSlot::CodeContext as usize].allocated_tokens,
                            budget.slots[ContextSlot::SsbEntries as usize].allocated_tokens,
                            budget.slots[ContextSlot::GrcHistory as usize].allocated_tokens,
                            budget.slots[output_slot].allocated_tokens,
                            budget.total_allocated,
                            budget.total_used,
                            budget.cache_breakpoints[0].valid,
                            budget.cache_breakpoints[1].valid,
                            effective,
                        );
                        model_max_tokens.min(budget_output)
                    }
                    Err(e) => {
                        eprintln!("[pipeline] budget warning (fallback to model max): {e}");
                        model_max_tokens
                    }
                }
            }
            BudgetPolicy::ApiDefault => model_max_tokens,
            // B-27: cloud-mode 200K context profile. Bypasses the
            // per-model token table so Anthropic Sonnet 4.6 actually
            // gets its full 200K window when routed through cloud_mode.
            BudgetPolicy::Cloud { ceiling } => *ceiling,
        };

        // Pipeline telemetry
        let msg_count = request.messages.len();
        let tool_count = tool_defs.as_ref().map_or(0, Vec::len);
        let tool_names: Vec<&str> = tool_defs.as_ref().map_or_else(Vec::new, |defs| {
            defs.iter().map(|d| d.name.as_str()).collect()
        });
        eprintln!(
            "[pipeline] backend={:?}, model={}, system_chars={}, messages={}, \
             tools={} [{}], max_tokens={}",
            self.backend,
            self.model,
            system_chars,
            msg_count,
            tool_count,
            tool_names.join(", "),
            effective_max_tokens,
        );
        let _stream_clock = elai_core::OsClock;
        let stream_start = _stream_clock.now();

        let message_request = MessageRequest {
            model: self.model.clone(),
            max_tokens: effective_max_tokens,
            messages: convert_messages(&request.messages),
            system: (!request.system_prompt.is_empty())
                .then(|| vec![SystemBlock::text(request.system_prompt.join("\n\n"))]),
            tool_choice: tool_defs.as_ref().map(|_| ToolChoice::Auto),
            tools: tool_defs,
            stream: true,
            seed: None,
        };

        let use_prompt_cache = matches!(self.backend, BackendMode::Api);

        let result = self.runtime.block_on(async {
            let mut stream = self
                .client
                .stream_message(&message_request)
                .await
                .map_err(|error| RuntimeError::new(error.to_string()))?;
            let mut stdout = io::stdout();
            let mut sink = io::sink();
            let out: &mut dyn Write = if self.emit_output {
                &mut stdout
            } else {
                &mut sink
            };
            let renderer = TerminalRenderer::new();
            let mut markdown_stream = MarkdownStreamState::default();
            let mut events = Vec::new();
            let mut pending_tool: Option<(String, String, String)> = None;
            let mut saw_stop = false;

            while let Some(event) = stream
                .next_event()
                .await
                .map_err(|error| RuntimeError::new(error.to_string()))?
            {
                match event {
                    ApiStreamEvent::MessageStart(start) => {
                        for block in start.message.content {
                            push_output_block(block, out, &mut events, &mut pending_tool, true)?;
                        }
                    }
                    ApiStreamEvent::ContentBlockStart(start) => {
                        push_output_block(
                            start.content_block,
                            out,
                            &mut events,
                            &mut pending_tool,
                            true,
                        )?;
                    }
                    ApiStreamEvent::ContentBlockDelta(delta) => match delta.delta {
                        ContentBlockDelta::TextDelta { text } => {
                            if !text.is_empty() {
                                if let Some(progress_reporter) = &self.progress_reporter {
                                    progress_reporter.mark_text_phase(&text);
                                }
                                if let Some(rendered) = markdown_stream.push(&renderer, &text) {
                                    write!(out, "{rendered}")
                                        .and_then(|()| out.flush())
                                        .map_err(|error| RuntimeError::new(error.to_string()))?;
                                }
                                events.push(AssistantEvent::TextDelta(text));
                            }
                        }
                        ContentBlockDelta::InputJsonDelta { partial_json } => {
                            if let Some((_, _, input)) = &mut pending_tool {
                                input.push_str(&partial_json);
                            }
                        }
                        ContentBlockDelta::ThinkingDelta { .. }
                        | ContentBlockDelta::SignatureDelta { .. } => {}
                    },
                    ApiStreamEvent::ContentBlockStop(_) => {
                        if let Some(rendered) = markdown_stream.flush(&renderer) {
                            write!(out, "{rendered}")
                                .and_then(|()| out.flush())
                                .map_err(|error| RuntimeError::new(error.to_string()))?;
                        }
                        if let Some((id, name, input)) = pending_tool.take() {
                            if let Some(block_reason) = self.evaluate_tool_rules(&name, &input) {
                                eprintln!("[rules] tool {name} blocked: {block_reason}");
                            } else {
                                if let Some(progress_reporter) = &self.progress_reporter {
                                    progress_reporter.mark_tool_phase(&name, &input);
                                }
                                writeln!(out, "\n{}", format_tool_call_start(&name, &input))
                                    .and_then(|()| out.flush())
                                    .map_err(|error| RuntimeError::new(error.to_string()))?;
                                events.push(AssistantEvent::ToolUse { id, name, input });
                            }
                        }
                    }
                    ApiStreamEvent::MessageDelta(delta) => {
                        events.push(AssistantEvent::Usage(delta.usage.token_usage()));
                    }
                    ApiStreamEvent::MessageStop(_) => {
                        saw_stop = true;
                        if let Some(rendered) = markdown_stream.flush(&renderer) {
                            write!(out, "{rendered}")
                                .and_then(|()| out.flush())
                                .map_err(|error| RuntimeError::new(error.to_string()))?;
                        }
                        events.push(AssistantEvent::MessageStop);
                    }
                }
            }

            // Branch 3: prompt cache — Api only
            if use_prompt_cache {
                push_prompt_cache_record(&self.client, &mut events);
            }

            if !saw_stop
                && events.iter().any(|event| {
                    matches!(event, AssistantEvent::TextDelta(text) if !text.is_empty())
                        || matches!(event, AssistantEvent::ToolUse { .. })
                })
            {
                events.push(AssistantEvent::MessageStop);
            }

            if events
                .iter()
                .any(|event| matches!(event, AssistantEvent::MessageStop))
            {
                return Ok(events);
            }

            let response = self
                .client
                .send_message(&MessageRequest {
                    stream: false,
                    ..message_request.clone()
                })
                .await
                .map_err(|error| RuntimeError::new(error.to_string()))?;
            let mut events = response_to_events(response, out)?;
            if use_prompt_cache {
                push_prompt_cache_record(&self.client, &mut events);
            }
            Ok(events)
        });

        let elapsed_ms = _stream_clock.elapsed_since(stream_start).as_millis();
        match &result {
            Ok(events) => {
                let text_chars: usize = events.iter().map(|e| match e {
                    AssistantEvent::TextDelta(t) => t.len(),
                    _ => 0,
                }).sum();
                let tool_uses = events.iter().filter(|e| matches!(e, AssistantEvent::ToolUse { .. })).count();
                let usage = events.iter().find_map(|e| match e {
                    AssistantEvent::Usage(u) => Some(u),
                    _ => None,
                });
                if let Some(u) = usage {
                    eprintln!(
                        "[pipeline] done: elapsed={elapsed_ms}ms, input_tokens={}, output_tokens={}, \
                         text_chars={text_chars}, tool_uses={tool_uses}",
                        u.input_tokens, u.output_tokens,
                    );
                } else {
                    eprintln!(
                        "[pipeline] done: elapsed={elapsed_ms}ms, text_chars={text_chars}, \
                         tool_uses={tool_uses} (no usage reported)",
                    );
                }
            }
            Err(e) => {
                eprintln!("[pipeline] error after {elapsed_ms}ms: {e}");
            }
        }
        result
    }
}

pub(crate) fn final_assistant_text(summary: &elai_orchestrator::TurnSummary) -> String {
    summary
        .assistant_messages
        .last()
        .map(|message| {
            message
                .blocks
                .iter()
                .filter_map(|block| match block {
                    ContentBlock::Text { text } => Some(text.as_str()),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .join("")
        })
        .unwrap_or_default()
}

pub(crate) fn collect_tool_uses(summary: &elai_orchestrator::TurnSummary) -> Vec<serde_json::Value> {
    summary
        .assistant_messages
        .iter()
        .flat_map(|message| message.blocks.iter())
        .filter_map(|block| match block {
            ContentBlock::ToolUse { id, name, input } => Some(json!({
                "id": id,
                "name": name,
                "input": input,
            })),
            _ => None,
        })
        .collect()
}

pub(crate) fn collect_tool_results(summary: &elai_orchestrator::TurnSummary) -> Vec<serde_json::Value> {
    summary
        .tool_results
        .iter()
        .flat_map(|message| message.blocks.iter())
        .filter_map(|block| match block {
            ContentBlock::ToolResult {
                tool_use_id,
                tool_name,
                output,
                is_error,
            } => Some(json!({
                "tool_use_id": tool_use_id,
                "tool_name": tool_name,
                "output": output,
                "is_error": is_error,
            })),
            _ => None,
        })
        .collect()
}

pub(crate) fn collect_prompt_cache_events(summary: &elai_orchestrator::TurnSummary) -> Vec<serde_json::Value> {
    summary
        .prompt_cache_events
        .iter()
        .map(|event| {
            json!({
                "unexpected": event.unexpected,
                "reason": event.reason,
                "previous_cache_read_input_tokens": event.previous_cache_read_input_tokens,
                "current_cache_read_input_tokens": event.current_cache_read_input_tokens,
                "token_drop": event.token_drop,
            })
        })
        .collect()
}

pub(crate) fn slash_command_completion_candidates_with_sessions(
    model: &str,
    active_session_id: Option<&str>,
    recent_session_ids: Vec<String>,
) -> Vec<String> {
    let mut completions = BTreeSet::new();

    for spec in slash_command_specs() {
        completions.insert(format!("/{}", spec.name));
        for alias in spec.aliases {
            completions.insert(format!("/{alias}"));
        }
    }

    for candidate in [
        "/bughunter ",
        "/clear --confirm",
        "/config ",
        "/config env",
        "/config hooks",
        "/config model",
        "/config plugins",
        "/mcp ",
        "/mcp list",
        "/mcp show ",
        "/export ",
        "/issue ",
        "/model ",
        "/model opus",
        "/model sonnet",
        "/model haiku",
        "/permissions ",
        "/permissions read-only",
        "/permissions workspace-write",
        "/permissions danger-full-access",
        "/plugin list",
        "/plugin install ",
        "/plugin enable ",
        "/plugin disable ",
        "/plugin uninstall ",
        "/plugin update ",
        "/plugins list",
        "/pr ",
        "/resume ",
        "/session list",
        "/session switch ",
        "/session fork ",
        "/teleport ",
        "/ultraplan ",
        "/agents help",
        "/mcp help",
        "/skills help",
    ] {
        completions.insert(candidate.to_string());
    }

    if !model.trim().is_empty() {
        completions.insert(format!("/model {}", resolve_model_alias(model)));
        completions.insert(format!("/model {model}"));
    }

    if let Some(active_session_id) = active_session_id.filter(|value| !value.trim().is_empty()) {
        completions.insert(format!("/resume {active_session_id}"));
        completions.insert(format!("/session switch {active_session_id}"));
    }

    for session_id in recent_session_ids
        .into_iter()
        .filter(|value| !value.trim().is_empty())
        .take(10)
    {
        completions.insert(format!("/resume {session_id}"));
        completions.insert(format!("/session switch {session_id}"));
    }

    completions.into_iter().collect()
}

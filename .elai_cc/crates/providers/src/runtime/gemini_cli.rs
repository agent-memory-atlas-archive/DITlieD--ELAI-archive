//! `GeminiCliRuntime` — B-37B port of ouroboros's `GeminiCLIRuntime`.
//!
//! Spawns
//! `gemini --prompt <prompt> --non-interactive --output-format stream-json
//!  --approval-mode {auto_edit|yolo}` (no stdin needed; paper L207).
//!
//! Permission mapping (paper L211, briefing facts):
//! - `AcceptEdits` -> `--approval-mode auto_edit`
//! - `DangerouslySkipPermissions` (with `allow_danger=true`) -> `--approval-mode yolo`
//! - Plan / Default -> `auto_edit` (gemini has no plan-only mode).

use std::sync::Arc;

use futures_util::stream::{self, BoxStream};
use tokio::io::AsyncBufReadExt as _;

use super::cli_common::{
    self, await_first_line_with_startup_timeout, await_next_line_with_idle_timeout,
    enforce_permission_opt_in, kill_and_reap_with_event, record_subprocess_exit, sanitize_stdout_line,
    spawn_isolated_with_event, DEFAULT_IDLE_TIMEOUT_S, DEFAULT_STARTUP_TIMEOUT_S,
};
use super::isolation::{default_tier_c_spawn, TierCSpawn, TierCStdio};
use super::types::{
    CompletionConfig, CompletionResponse, Message, RuntimeAdapter, RuntimeAdapterError,
    RuntimeBackend, RuntimeCapabilities, RuntimeFactoryConfig, RuntimeFuture, RuntimeHealthFuture,
    RuntimeStream, StreamChunk,
};

const BACKEND: RuntimeBackend = RuntimeBackend::GeminiCli;
const MAX_GEMINI_PROMPT_BYTES: usize = 30 * 1024;

pub struct GeminiCliRuntime {
    cfg: RuntimeFactoryConfig,
    spawn: Arc<dyn TierCSpawn>,
}

impl std::fmt::Debug for GeminiCliRuntime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GeminiCliRuntime")
            .field("cfg", &self.cfg)
            .finish_non_exhaustive()
    }
}

impl GeminiCliRuntime {
    #[must_use]
    pub fn new(cfg: RuntimeFactoryConfig) -> Self {
        Self {
            cfg,
            spawn: default_tier_c_spawn(),
        }
    }

    #[must_use]
    pub fn with_spawn(cfg: RuntimeFactoryConfig, spawn: Arc<dyn TierCSpawn>) -> Self {
        Self { cfg, spawn }
    }

    #[must_use]
    pub fn config(&self) -> &RuntimeFactoryConfig {
        &self.cfg
    }
}

fn build_argv(cfg: &RuntimeFactoryConfig, prompt: &str) -> Vec<String> {
    let mut args = Vec::with_capacity(10);
    args.push("--prompt".to_string());
    args.push(prompt.to_string());
    args.push("--non-interactive".to_string());
    args.push("--output-format".to_string());
    args.push("stream-json".to_string());
    args.push("--approval-mode".to_string());
    args.push(
        cli_common::permission_mode_to_gemini_approval_mode(cfg.permission_mode).to_string(),
    );
    if let Some(model) = &cfg.model {
        args.push("--model".to_string());
        args.push(model.clone());
    }
    args
}

impl RuntimeAdapter for GeminiCliRuntime {
    fn backend(&self) -> RuntimeBackend {
        BACKEND
    }

    fn capabilities(&self) -> RuntimeCapabilities {
        RuntimeCapabilities {
            supports_tools: true,
            supports_streaming: true,
            supports_vision: true,
            max_context: 1_000_000,
            max_output: 8_192,
            json_schema_mode: true,
            supports_prefill: false,
            supports_caching: true,
            native_thinking_budget: false,
        }
    }

    fn health_check(&self) -> RuntimeHealthFuture<'_> {
        let cfg = self.cfg.clone();
        Box::pin(async move { cli_common::health_check_cli_binary(&cfg, BACKEND).await })
    }

    fn complete(
        &self,
        messages: Vec<Message>,
        config: CompletionConfig,
    ) -> RuntimeFuture<'_, CompletionResponse> {
        let cfg = self.cfg.clone();
        let spawn = self.spawn.clone();
        Box::pin(run_complete(cfg, spawn, messages, config))
    }

    fn stream(
        &self,
        messages: Vec<Message>,
        config: CompletionConfig,
    ) -> RuntimeStream<'static> {
        let cfg = self.cfg.clone();
        let spawn = self.spawn.clone();
        Box::pin(stream::once(async move {
            match run_complete(cfg, spawn, messages, config).await {
                Ok(resp) => Ok(StreamChunk::TextDelta { text: resp.text }),
                Err(e) => Err(e),
            }
        })) as BoxStream<'static, Result<StreamChunk, RuntimeAdapterError>>
    }
}

async fn run_complete(
    cfg: RuntimeFactoryConfig,
    spawn: Arc<dyn TierCSpawn>,
    messages: Vec<Message>,
    config: CompletionConfig,
) -> Result<CompletionResponse, RuntimeAdapterError> {
    {
            enforce_permission_opt_in(&cfg)?;
            let binary = cli_common::resolve_binary(
                cfg.cli_path.as_deref(),
                cli_common::default_binary_for(BACKEND),
            )?;
            let prompt = cli_common::compose_prompt(&messages, config.system.as_deref());
            if prompt.len() > MAX_GEMINI_PROMPT_BYTES {
                return Err(RuntimeAdapterError::Other(format!(
                    "gemini --prompt is {} bytes, exceeds {}-byte CLI arg limit",
                    prompt.len(),
                    MAX_GEMINI_PROMPT_BYTES
                )));
            }
            let args = build_argv(&cfg, &prompt);
            let cwd = cfg.cwd.clone().unwrap_or_else(std::env::temp_dir);
            let scratch = cwd.clone();
            let mut child = spawn_isolated_with_event(
                &spawn,
                BACKEND,
                &binary,
                &args,
                &cwd,
                &scratch,
                TierCStdio::Null,
                TierCStdio::Piped,
                TierCStdio::Piped,
                cfg.permission_mode,
                Vec::new(),
            )
            .map_err(cli_common::spawn_err_to_adapter_err)?;
            let child_pid = child.id().unwrap_or(0);

            let stdout = child
                .stdout
                .take()
                .ok_or_else(|| RuntimeAdapterError::Other("gemini stdout pipe lost".into()))?;
            let mut reader = tokio::io::BufReader::new(stdout).lines();
            let startup =
                cli_common::effective_timeout(cfg.startup_output_timeout_seconds, DEFAULT_STARTUP_TIMEOUT_S);
            let idle =
                cli_common::effective_timeout(cfg.stdout_idle_timeout_seconds, DEFAULT_IDLE_TIMEOUT_S);
            let binary_label = binary.display().to_string();
            let mut acc = String::new();

            match await_first_line_with_startup_timeout(&mut reader, startup, &binary_label).await {
                Ok(Some(line)) => extract_text(&sanitize_stdout_line(&line), &mut acc),
                Ok(None) => {}
                Err(e) => {
                    kill_and_reap_with_event(&mut child, BACKEND, format!("startup timeout: {e}"))
                        .await;
                    return Err(e);
                }
            }
            loop {
                match await_next_line_with_idle_timeout(&mut reader, idle, &binary_label).await {
                    Ok(Some(line)) => extract_text(&sanitize_stdout_line(&line), &mut acc),
                    Ok(None) => break,
                    Err(e) => {
                        kill_and_reap_with_event(&mut child, BACKEND, format!("idle timeout: {e}"))
                            .await;
                        return Err(e);
                    }
                }
            }
            let status = child
                .wait()
                .await
                .map_err(|e| RuntimeAdapterError::Other(format!("gemini wait failed: {e}")))?;
            record_subprocess_exit(BACKEND, child_pid, status.code(), "natural exit");
            if !status.success() {
                return Err(RuntimeAdapterError::ExitNonZero {
                    binary: binary_label,
                    code: status.code(),
                    stderr_tail: String::new(),
                });
            }
            Ok(CompletionResponse {
                text: acc,
                stop_reason: Some("end_turn".to_string()),
                usage: (0, 0),
                model: cfg.model.clone().unwrap_or_default(),
            })
    }
}

fn extract_text(line: &str, accumulator: &mut String) {
    let Ok(value): Result<serde_json::Value, _> = serde_json::from_str(line) else {
        return;
    };
    let event_type = value.get("type").and_then(|v| v.as_str()).unwrap_or("");
    match event_type {
        "text" | "message" => {
            if let Some(text) = value.get("text").and_then(|t| t.as_str()) {
                accumulator.push_str(text);
            } else if let Some(text) = value.get("content").and_then(|t| t.as_str()) {
                accumulator.push_str(text);
            }
        }
        _ => {}
    }
}

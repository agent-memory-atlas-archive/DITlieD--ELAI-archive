//! `KiroCliRuntime` — B-37B port of ouroboros's `KiroAgentAdapter`.
//!
//! Spawns `kiro-cli chat --no-interactive --trust-tools=<cats> <prompt>`.
//! Binary name has a HYPHEN (`kiro-cli`, NOT `kiro` — paper L223,
//! L298-299). Stdout is raw text with ANSI color codes; stderr must be
//! drained concurrently to avoid OS pipe-buffer deadlock (paper L228).

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
    RuntimePermissionMode, RuntimeStream, StreamChunk,
};

const BACKEND: RuntimeBackend = RuntimeBackend::KiroCli;
const MAX_KIRO_PROMPT_BYTES: usize = 30 * 1024;

pub struct KiroCliRuntime {
    cfg: RuntimeFactoryConfig,
    spawn: Arc<dyn TierCSpawn>,
}

impl std::fmt::Debug for KiroCliRuntime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KiroCliRuntime")
            .field("cfg", &self.cfg)
            .finish_non_exhaustive()
    }
}

impl KiroCliRuntime {
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

fn trust_flag(mode: RuntimePermissionMode) -> Option<String> {
    match mode {
        RuntimePermissionMode::AcceptEdits => Some("--trust-tools=fs,bash".to_string()),
        RuntimePermissionMode::DangerouslySkipPermissions => Some("--trust-all-tools".to_string()),
        RuntimePermissionMode::Plan | RuntimePermissionMode::Default => None,
    }
}

fn build_argv(cfg: &RuntimeFactoryConfig, prompt: &str) -> Vec<String> {
    let mut args = Vec::with_capacity(8);
    args.push("chat".to_string());
    args.push("--no-interactive".to_string());
    if let Some(flag) = trust_flag(cfg.permission_mode) {
        args.push(flag);
    }
    if let Some(model) = &cfg.model {
        args.push("--model".to_string());
        args.push(model.clone());
    }
    if let Some(resume) = &cfg.runtime_profile {
        args.push("--resume-id".to_string());
        args.push(resume.clone());
    }
    args.push(prompt.to_string());
    args
}

impl RuntimeAdapter for KiroCliRuntime {
    fn backend(&self) -> RuntimeBackend {
        BACKEND
    }

    fn capabilities(&self) -> RuntimeCapabilities {
        RuntimeCapabilities {
            supports_tools: true,
            supports_streaming: true,
            supports_vision: false,
            max_context: 128_000,
            max_output: 8_192,
            json_schema_mode: false,
            supports_prefill: false,
            supports_caching: false,
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
            if prompt.len() > MAX_KIRO_PROMPT_BYTES {
                return Err(RuntimeAdapterError::Other(format!(
                    "kiro positional prompt is {} bytes, exceeds {}-byte CLI arg limit",
                    prompt.len(),
                    MAX_KIRO_PROMPT_BYTES
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
                cfg.transport,
            )
            .map_err(cli_common::spawn_err_to_adapter_err)?;
            let child_pid = child.id().unwrap_or(0);

            // Concurrent stderr drain (paper L228 — pipe-buffer
            // deadlock mitigation).
            if let Some(stderr) = child.stderr.take() {
                tokio::spawn(async move {
                    use tokio::io::{AsyncBufReadExt, BufReader};
                    let mut reader = BufReader::new(stderr).lines();
                    while let Ok(Some(_)) = reader.next_line().await {
                        // Drain only; diagnostics are kept short to
                        // bound memory.
                    }
                });
            }

            let stdout = child
                .stdout
                .take()
                .ok_or_else(|| RuntimeAdapterError::Other("kiro stdout pipe lost".into()))?;
            let mut reader = tokio::io::BufReader::new(stdout).lines();
            let startup =
                cli_common::effective_timeout(cfg.startup_output_timeout_seconds, DEFAULT_STARTUP_TIMEOUT_S);
            let idle =
                cli_common::effective_timeout(cfg.stdout_idle_timeout_seconds, DEFAULT_IDLE_TIMEOUT_S);
            let binary_label = binary.display().to_string();
            let mut collected = String::new();

            match await_first_line_with_startup_timeout(&mut reader, startup, &binary_label).await {
                Ok(Some(line)) => {
                    collected.push_str(&sanitize_stdout_line(&line));
                    collected.push('\n');
                }
                Ok(None) => {}
                Err(e) => {
                    kill_and_reap_with_event(&mut child, BACKEND, format!("startup timeout: {e}"))
                        .await;
                    return Err(e);
                }
            }
            loop {
                match await_next_line_with_idle_timeout(&mut reader, idle, &binary_label).await {
                    Ok(Some(line)) => {
                        collected.push_str(&sanitize_stdout_line(&line));
                        collected.push('\n');
                    }
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
                .map_err(|e| RuntimeAdapterError::Other(format!("kiro wait failed: {e}")))?;
            record_subprocess_exit(BACKEND, child_pid, status.code(), "natural exit");
            if !status.success() {
                return Err(RuntimeAdapterError::ExitNonZero {
                    binary: binary_label,
                    code: status.code(),
                    stderr_tail: String::new(),
                });
            }
            Ok(CompletionResponse {
                text: collected,
                stop_reason: Some("end_turn".to_string()),
                usage: (0, 0),
                cache_creation_input_tokens: 0,
                cache_read_input_tokens: 0,
                model: cfg.model.clone().unwrap_or_default(),
            })
    }
}

/// Smoke test that the constructor records the conventional binary
/// name with the HYPHEN. Exposed via a free function rather than a
/// method so the contract test can call it without instantiating a
/// runtime.
#[must_use]
pub fn default_binary_name() -> &'static str {
    cli_common::default_binary_for(BACKEND)
}

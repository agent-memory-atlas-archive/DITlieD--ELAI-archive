//! `CodexCliRuntime` — B-37B port of ouroboros's `CodexCliRuntime`.
//!
//! Spawns
//! `codex exec --json --skip-git-repo-check --output-last-message <tmp> -C <cwd>`
//! with the composed prompt fed via stdin.
//!
//! CRITICAL gotcha (paper L153, L295-296): the final answer is written
//! to `--output-last-message <path>`, NOT stdout. Stdout is a JSONL
//! event stream (`thread.started` / `item.completed` / `turn.failed` /
//! `turn.completed`). The final text must be read from the temp file
//! AFTER the subprocess exits.

use std::sync::Arc;

use futures_util::stream::{self, BoxStream};
use tokio::io::{AsyncBufReadExt as _, AsyncWriteExt};

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

const BACKEND: RuntimeBackend = RuntimeBackend::CodexCli;

pub struct CodexCliRuntime {
    cfg: RuntimeFactoryConfig,
    spawn: Arc<dyn TierCSpawn>,
}

impl std::fmt::Debug for CodexCliRuntime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CodexCliRuntime")
            .field("cfg", &self.cfg)
            .finish_non_exhaustive()
    }
}

impl CodexCliRuntime {
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

fn build_argv(cfg: &RuntimeFactoryConfig, output_path: &std::path::Path) -> Vec<String> {
    let mut args = Vec::with_capacity(12);
    args.push("exec".to_string());
    args.push("--json".to_string());
    args.push("--skip-git-repo-check".to_string());
    args.push("--output-last-message".to_string());
    args.push(output_path.display().to_string());
    let cwd = cfg.cwd.clone().unwrap_or_else(std::env::temp_dir);
    args.push("-C".to_string());
    args.push(cwd.display().to_string());
    if let Some(model) = &cfg.model {
        args.push("--model".to_string());
        args.push(model.clone());
    }
    args
}

impl RuntimeAdapter for CodexCliRuntime {
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
            json_schema_mode: true,
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
            // codex profile-based permission_mode is deferred to
            // B-37C; for now refuse explicit Plan to avoid silent
            // mis-binding.
            if matches!(cfg.permission_mode, RuntimePermissionMode::Plan) {
                return Err(RuntimeAdapterError::NotImplemented {
                    backend: BACKEND,
                    hint: cli_common::HINT_B37B_PARTIAL,
                });
            }
            let binary = cli_common::resolve_binary(
                cfg.cli_path.as_deref(),
                cli_common::default_binary_for(BACKEND),
            )?;
            let temp_path = cli_common::codex_output_temp_path()?;
            let args = build_argv(&cfg, &temp_path);
            let prompt = cli_common::compose_prompt(&messages, config.system.as_deref());
            let cwd = cfg.cwd.clone().unwrap_or_else(std::env::temp_dir);
            let scratch = cwd.clone();
            let mut child = spawn_isolated_with_event(
                &spawn,
                BACKEND,
                &binary,
                &args,
                &cwd,
                &scratch,
                TierCStdio::Piped,
                TierCStdio::Piped,
                TierCStdio::Piped,
                cfg.permission_mode,
                Vec::new(),
                cfg.transport,
            )
            .map_err(cli_common::spawn_err_to_adapter_err)?;
            let child_pid = child.id().unwrap_or(0);

            if let Some(mut stdin) = child.stdin.take() {
                if let Err(e) = stdin.write_all(prompt.as_bytes()).await {
                    let _ = std::fs::remove_file(&temp_path);
                    kill_and_reap_with_event(&mut child, BACKEND, format!("stdin write failed: {e}"))
                        .await;
                    return Err(RuntimeAdapterError::Other(format!(
                        "codex stdin write failed: {e}"
                    )));
                }
                drop(stdin);
            }

            let stdout = child
                .stdout
                .take()
                .ok_or_else(|| RuntimeAdapterError::Other("codex stdout pipe lost".into()))?;
            let mut reader = tokio::io::BufReader::new(stdout).lines();
            let startup =
                cli_common::effective_timeout(cfg.startup_output_timeout_seconds, DEFAULT_STARTUP_TIMEOUT_S);
            let idle =
                cli_common::effective_timeout(cfg.stdout_idle_timeout_seconds, DEFAULT_IDLE_TIMEOUT_S);
            let binary_label = binary.display().to_string();

            // Drain the JSONL stream — we only need it for liveness +
            // turn.failed detection; the actual answer is in the temp
            // file.
            let mut saw_failure = false;
            match await_first_line_with_startup_timeout(&mut reader, startup, &binary_label).await {
                Ok(Some(line)) => {
                    let clean = sanitize_stdout_line(&line);
                    if clean.contains("turn.failed") {
                        saw_failure = true;
                    }
                }
                Ok(None) => {}
                Err(e) => {
                    let _ = std::fs::remove_file(&temp_path);
                    kill_and_reap_with_event(&mut child, BACKEND, format!("startup timeout: {e}"))
                        .await;
                    return Err(e);
                }
            }
            loop {
                match await_next_line_with_idle_timeout(&mut reader, idle, &binary_label).await {
                    Ok(Some(line)) => {
                        let clean = sanitize_stdout_line(&line);
                        if clean.contains("turn.failed") {
                            saw_failure = true;
                        }
                    }
                    Ok(None) => break,
                    Err(e) => {
                        let _ = std::fs::remove_file(&temp_path);
                        kill_and_reap_with_event(&mut child, BACKEND, format!("idle timeout: {e}"))
                            .await;
                        return Err(e);
                    }
                }
            }

            let status = child
                .wait()
                .await
                .map_err(|e| RuntimeAdapterError::Other(format!("codex wait failed: {e}")))?;
            record_subprocess_exit(BACKEND, child_pid, status.code(), "natural exit");
            // Always best-effort delete the temp file after read.
            let final_text =
                std::fs::read_to_string(&temp_path).unwrap_or_default();
            let _ = std::fs::remove_file(&temp_path);

            if !status.success() || saw_failure {
                return Err(RuntimeAdapterError::ExitNonZero {
                    binary: binary_label,
                    code: status.code(),
                    stderr_tail: if saw_failure {
                        "turn.failed event observed".to_string()
                    } else {
                        String::new()
                    },
                });
            }
            Ok(CompletionResponse {
                text: final_text,
                stop_reason: Some("end_turn".to_string()),
                usage: (0, 0),
                cache_creation_input_tokens: 0,
                cache_read_input_tokens: 0,
                model: cfg.model.clone().unwrap_or_default(),
            })
    }
}

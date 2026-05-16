//! `OpenCodeRuntime` — B-37B port of ouroboros's `OpenCodeRuntime`.
//!
//! Spawns `opencode run --pure --format json` with the prompt fed via
//! stdin. The `--pure` flag is MANDATORY (paper L167, L292-293):
//! omitting it triggers plugin double-dispatch.

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
    RuntimeStream, StreamChunk,
};

const BACKEND: RuntimeBackend = RuntimeBackend::OpenCode;

pub struct OpenCodeRuntime {
    cfg: RuntimeFactoryConfig,
    spawn: Arc<dyn TierCSpawn>,
}

impl std::fmt::Debug for OpenCodeRuntime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OpenCodeRuntime")
            .field("cfg", &self.cfg)
            .finish_non_exhaustive()
    }
}

impl OpenCodeRuntime {
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

fn build_argv(cfg: &RuntimeFactoryConfig) -> Vec<String> {
    let mut args = Vec::with_capacity(8);
    args.push("run".to_string());
    // --pure: MANDATORY (paper L167).
    args.push("--pure".to_string());
    args.push("--format".to_string());
    args.push("json".to_string());
    if let Some(model) = &cfg.model {
        args.push("--model".to_string());
        args.push(model.clone());
    }
    if let Some(session) = &cfg.runtime_profile {
        args.push("--session".to_string());
        args.push(session.clone());
    }
    args
}

impl RuntimeAdapter for OpenCodeRuntime {
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
            let args = build_argv(&cfg);
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
            )
            .map_err(cli_common::spawn_err_to_adapter_err)?;
            let child_pid = child.id().unwrap_or(0);

            if let Some(mut stdin) = child.stdin.take() {
                if let Err(e) = stdin.write_all(prompt.as_bytes()).await {
                    kill_and_reap_with_event(
                        &mut child,
                        BACKEND,
                        format!("stdin write failed: {e}"),
                    )
                    .await;
                    return Err(RuntimeAdapterError::Other(format!(
                        "opencode stdin write failed: {e}"
                    )));
                }
                drop(stdin);
            }

            let stdout = child
                .stdout
                .take()
                .ok_or_else(|| RuntimeAdapterError::Other("opencode stdout pipe lost".into()))?;
            let mut reader = tokio::io::BufReader::new(stdout).lines();
            let startup =
                cli_common::effective_timeout(cfg.startup_output_timeout_seconds, DEFAULT_STARTUP_TIMEOUT_S);
            let idle =
                cli_common::effective_timeout(cfg.stdout_idle_timeout_seconds, DEFAULT_IDLE_TIMEOUT_S);
            let binary_label = binary.display().to_string();

            let mut collected = String::new();
            match await_first_line_with_startup_timeout(&mut reader, startup, &binary_label).await {
                Ok(Some(line)) => {
                    let clean = sanitize_stdout_line(&line);
                    extract_text_from_event(&clean, &mut collected);
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
                        let clean = sanitize_stdout_line(&line);
                        extract_text_from_event(&clean, &mut collected);
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
                .map_err(|e| RuntimeAdapterError::Other(format!("opencode wait failed: {e}")))?;
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
                model: cfg.model.clone().unwrap_or_default(),
            })
    }
}

fn extract_text_from_event(line: &str, accumulator: &mut String) {
    let Ok(value): Result<serde_json::Value, _> = serde_json::from_str(line) else {
        // Non-JSON line — treat as raw text fallback (paper L171
        // "stderr fallback when stdout lacks final message").
        accumulator.push_str(line);
        accumulator.push('\n');
        return;
    };
    if let Some(text) = value.get("text").and_then(|t| t.as_str()) {
        accumulator.push_str(text);
    } else if let Some(text) = value.get("content").and_then(|t| t.as_str()) {
        accumulator.push_str(text);
    } else if let Some(text) = value.get("message").and_then(|t| t.as_str()) {
        accumulator.push_str(text);
    }
}

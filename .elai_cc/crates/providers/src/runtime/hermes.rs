//! `HermesRuntime` — B-37B port of ouroboros's `HermesCliRuntime`.
//!
//! Spawns `hermes chat -Q --source tool --model <model> -q <prompt>`,
//! reads RAW TEXT line-by-line (NOT JSONL), strips box-drawing
//! reasoning prelude, and surfaces the body as a `CompletionResponse`.
//!
//! Key invariant: `supports_tools = false` — hermes uniquely lacks
//! `supports_tool_envelope` (paper L40, L191). Sending a tool envelope
//! through hermes silently degrades to "JSON-as-text"; we surface this
//! through `capabilities()` so callers can pre-empt the bad call.

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

const BACKEND: RuntimeBackend = RuntimeBackend::Hermes;
const MAX_HERMES_PROMPT_BYTES: usize = 30 * 1024;

/// Hermes CLI adapter.
pub struct HermesRuntime {
    cfg: RuntimeFactoryConfig,
    spawn: Arc<dyn TierCSpawn>,
}

impl std::fmt::Debug for HermesRuntime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HermesRuntime")
            .field("cfg", &self.cfg)
            .finish_non_exhaustive()
    }
}

impl HermesRuntime {
    #[must_use]
    pub fn new(cfg: RuntimeFactoryConfig) -> Self {
        Self {
            cfg,
            spawn: default_tier_c_spawn(),
        }
    }

    /// Test seam: inject a custom `TierCSpawn` implementation.
    #[must_use]
    pub fn with_spawn(cfg: RuntimeFactoryConfig, spawn: Arc<dyn TierCSpawn>) -> Self {
        Self { cfg, spawn }
    }

    #[must_use]
    pub fn config(&self) -> &RuntimeFactoryConfig {
        &self.cfg
    }
}

fn strip_reasoning_prelude(text: &str) -> String {
    // Drop any leading lines that look like box-drawing decorations
    // (paper L189-190: `_strip_reasoning_prelude` removes ouroboros
    // hermes header art). Heuristic: a line is a "decoration" if
    // every non-whitespace char is from the Unicode Box Drawing block
    // (U+2500..U+257F) or a small set of punctuation.
    let mut out = String::with_capacity(text.len());
    let mut still_in_prelude = true;
    for line in text.lines() {
        if still_in_prelude {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            let is_decoration = trimmed.chars().all(|c| {
                ('\u{2500}'..='\u{257F}').contains(&c)
                    || matches!(c, '|' | '-' | '+' | '=' | '*' | ' ' | '\t')
            });
            if is_decoration {
                continue;
            }
            still_in_prelude = false;
        }
        out.push_str(line);
        out.push('\n');
    }
    out
}

fn build_argv(cfg: &RuntimeFactoryConfig, prompt: &str) -> Vec<String> {
    let mut args = Vec::with_capacity(10);
    args.push("chat".to_string());
    // Optional resume id via runtime_profile (cheap stand-in until a
    // proper session-handle type lands in B-37C).
    if let Some(profile) = &cfg.runtime_profile {
        args.push("--resume".to_string());
        args.push(profile.clone());
    }
    args.push("-Q".to_string());
    args.push("--source".to_string());
    args.push("tool".to_string());
    if let Some(model) = &cfg.model {
        args.push("--model".to_string());
        args.push(model.clone());
    }
    args.push("-q".to_string());
    args.push(prompt.to_string());
    args
}

impl RuntimeAdapter for HermesRuntime {
    fn backend(&self) -> RuntimeBackend {
        BACKEND
    }

    fn capabilities(&self) -> RuntimeCapabilities {
        RuntimeCapabilities {
            // Hermes uniquely lacks tool-envelope support (paper L40, L191).
            supports_tools: false,
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
            if prompt.len() > MAX_HERMES_PROMPT_BYTES {
                return Err(RuntimeAdapterError::Other(format!(
                    "hermes -q prompt is {} bytes, exceeds {}-byte CLI arg limit",
                    prompt.len(),
                    MAX_HERMES_PROMPT_BYTES
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
                .ok_or_else(|| RuntimeAdapterError::Other("hermes stdout pipe lost".into()))?;
            let mut reader = tokio::io::BufReader::new(stdout).lines();
            let startup =
                cli_common::effective_timeout(cfg.startup_output_timeout_seconds, DEFAULT_STARTUP_TIMEOUT_S);
            let idle =
                cli_common::effective_timeout(cfg.stdout_idle_timeout_seconds, DEFAULT_IDLE_TIMEOUT_S);

            let binary_label = binary.display().to_string();
            let mut collected = String::new();
            match await_first_line_with_startup_timeout(&mut reader, startup, &binary_label).await {
                Ok(Some(line)) => collected.push_str(&sanitize_stdout_line(&line)),
                Ok(None) => {
                    // EOF before any line — child exited.
                }
                Err(e) => {
                    kill_and_reap_with_event(&mut child, BACKEND, format!("startup timeout: {e}"))
                        .await;
                    return Err(e);
                }
            }
            collected.push('\n');

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
                .map_err(|e| RuntimeAdapterError::Other(format!("hermes wait failed: {e}")))?;
            record_subprocess_exit(BACKEND, child_pid, status.code(), "natural exit");
            if !status.success() {
                return Err(RuntimeAdapterError::ExitNonZero {
                    binary: binary_label,
                    code: status.code(),
                    stderr_tail: String::new(),
                });
            }

            let cleaned = strip_reasoning_prelude(&collected);
            Ok(CompletionResponse {
                text: cleaned,
                stop_reason: Some("end_turn".to_string()),
                usage: (0, 0),
                model: cfg.model.clone().unwrap_or_default(),
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hermes_supports_tools_is_false() {
        let cfg = RuntimeFactoryConfig::for_backend(BACKEND);
        let rt = HermesRuntime::new(cfg);
        assert!(!rt.capabilities().supports_tools);
    }

    #[test]
    fn strip_reasoning_prelude_removes_box_chars() {
        let input = "┌────────────────┐\n│ trace │\n└────────────────┘\nactual answer\n";
        let out = strip_reasoning_prelude(input);
        assert!(out.contains("actual answer"));
        assert!(!out.contains('┌'));
    }
}

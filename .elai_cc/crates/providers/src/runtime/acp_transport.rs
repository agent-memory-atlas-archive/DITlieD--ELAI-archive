//! J-64 — ACP (Agent Client Protocol) transport for ELAI's agent bridges.
//!
//! Implements `RuntimeAdapter` over ACP's JSON-RPC/stdio protocol, reusing
//! `cli_common::spawn_isolated_with_event` for Tier-C isolation. The ACP
//! crate's `AcpAgent::spawn_process()` bypasses Tier-C, so this transport
//! spawns the agent subprocess itself and bridges tokio stdio to the
//! `futures::AsyncRead/AsyncWrite` traits ACP expects via `tokio-util` compat.
//!
//! Callback policy is deny-by-default: `session/request_permission`,
//! `fs/read_text_file`, and `fs/write_text_file` requests from the agent
//! are all denied unless the path is inside the Tier-C 4A scratch jail.

use std::sync::Arc;

use agent_client_protocol::{
    on_receive_request, ByteStreams, Client,
};
use agent_client_protocol::schema::v1::{
    ReadTextFileRequest, ReadTextFileResponse,
    RequestPermissionRequest, RequestPermissionResponse,
    RequestPermissionOutcome, WriteTextFileRequest, WriteTextFileResponse,
};
use futures_util::stream::{self, BoxStream};
use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};

use super::cli_common::{
    self, enforce_permission_opt_in, kill_and_reap_with_event, resolve_binary,
    spawn_isolated_with_event,
};
use super::isolation::{default_tier_c_spawn, TierCSpawn, TierCStdio};
use super::types::{
    CompletionConfig, CompletionResponse, Message, RuntimeAdapter, RuntimeAdapterError,
    RuntimeBackend, RuntimeCapabilities, RuntimeFactoryConfig, RuntimeFuture,
    RuntimeHealthFuture, RuntimePermissionMode, RuntimeStream, StreamChunk,
};

const BACKEND: RuntimeBackend = RuntimeBackend::ClaudeCode;

pub struct AcpTransport {
    cfg: RuntimeFactoryConfig,
    spawn: Arc<dyn TierCSpawn>,
}

impl std::fmt::Debug for AcpTransport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AcpTransport")
            .field("cfg", &self.cfg)
            .finish_non_exhaustive()
    }
}

impl AcpTransport {
    #[must_use]
    pub fn new(cfg: RuntimeFactoryConfig) -> Self {
        Self {
            cfg,
            spawn: default_tier_c_spawn(),
        }
    }

    // WIRE-DARK[J-64]: test-injection constructor; production callers use new() which defaults the spawn handle (same pattern as all 7 CLI adapters)
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
    let cwd = cfg.cwd.clone().unwrap_or_else(std::env::temp_dir);
    args.push("--cwd".to_string());
    args.push(cwd.display().to_string());
    if let Some(model) = &cfg.model {
        args.push("--model".to_string());
        args.push(model.clone());
    }
    args
}

fn is_path_in_jail(path: &std::path::Path, jail: &std::path::Path) -> bool {
    let canonical_target = match path.canonicalize() {
        Ok(p) => p,
        Err(_) => return false,
    };
    let canonical_jail = match jail.canonicalize() {
        Ok(p) => p,
        Err(_) => return false,
    };
    canonical_target.starts_with(&canonical_jail)
}

impl RuntimeAdapter for AcpTransport {
    fn backend(&self) -> RuntimeBackend {
        BACKEND
    }

    fn capabilities(&self) -> RuntimeCapabilities {
        RuntimeCapabilities {
            supports_tools: true,
            supports_streaming: true,
            supports_vision: false,
            max_context: 200_000,
            max_output: 16_384,
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
                Ok(resp) => {
                    Ok(StreamChunk::TextDelta { text: resp.text })
                }
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
    enforce_permission_opt_in(&cfg)?;
    if matches!(cfg.permission_mode, RuntimePermissionMode::Plan) {
        return Err(RuntimeAdapterError::NotImplemented {
            backend: BACKEND,
            hint: "ACP transport does not support Plan mode",
        });
    }

    let binary = resolve_binary(
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
        super::types::BridgeTransport::Acp,
    )
    .map_err(cli_common::spawn_err_to_adapter_err)?;

    let child_pid = child.id().unwrap_or(0);

    let stdin = child
        .stdin
        .take()
        .ok_or_else(|| RuntimeAdapterError::Other("acp stdin pipe lost".into()))?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| RuntimeAdapterError::Other("acp stdout pipe lost".into()))?;

    let compat_stdin = stdin.compat_write();
    let compat_stdout = stdout.compat();
    let transport = ByteStreams::new(compat_stdin, compat_stdout);

    let jail_for_read = Arc::new(cwd.clone());
    let jail_for_write = Arc::clone(&jail_for_read);
    let result = Client
        .builder()
        .name("elai-acp-transport")
        .on_receive_request(
            async move |_req: RequestPermissionRequest, responder, _cx| {
                responder.respond(RequestPermissionResponse::new(
                    RequestPermissionOutcome::Cancelled,
                ))
            },
            on_receive_request!(),
        )
        .on_receive_request(
            async move |req: ReadTextFileRequest, responder, _cx| {
                if is_path_in_jail(&req.path, &jail_for_read) {
                    let content = std::fs::read_to_string(&req.path)
                        .map_err(|e| agent_client_protocol::Error::internal_error().data(e.to_string()))?;
                    responder.respond(ReadTextFileResponse::new(content))
                } else {
                    responder.respond_with_error(
                        agent_client_protocol::Error::invalid_params()
                            .data(format!("path outside jail: {}", req.path.display())),
                    )
                }
            },
            on_receive_request!(),
        )
        .on_receive_request(
            async move |req: WriteTextFileRequest, responder, _cx| {
                if is_path_in_jail(&req.path, &jail_for_write) {
                    std::fs::write(&req.path, &req.content)
                        .map_err(|e| agent_client_protocol::Error::internal_error().data(e.to_string()))?;
                    responder.respond(WriteTextFileResponse::new())
                } else {
                    responder.respond_with_error(
                        agent_client_protocol::Error::invalid_params()
                            .data(format!("path outside jail: {}", req.path.display())),
                    )
                }
            },
            on_receive_request!(),
        )
        .connect_with(transport, async |cx| {
            let mut session = cx
                .build_session(&cwd)
                .block_task()
                .start_session()
                .await?;
            session.send_prompt(&prompt)?;
            let text = session.read_to_string().await?;
            Ok(text)
        })
        .await;

    let text = match result {
        Ok(text) => text,
        Err(e) => {
            kill_and_reap_with_event(&mut child, BACKEND, format!("acp error: {e}")).await;
            return Err(RuntimeAdapterError::Other(format!("acp session failed: {e}")));
        }
    };

    let status = child
        .wait()
        .await
        .map_err(|e| RuntimeAdapterError::Other(format!("acp wait failed: {e}")))?;
    cli_common::record_subprocess_exit(BACKEND, child_pid, status.code(), "natural exit");

    if !status.success() {
        return Err(RuntimeAdapterError::ExitNonZero {
            binary: binary.display().to_string(),
            code: status.code(),
            stderr_tail: String::new(),
        });
    }

    Ok(CompletionResponse {
        text,
        stop_reason: Some("end_turn".to_string()),
        usage: (0, 0),
        cache_creation_input_tokens: 0,
        cache_read_input_tokens: 0,
        model: cfg.model.clone().unwrap_or_default(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn is_path_in_jail_rejects_outside() {
        let jail = PathBuf::from("/tmp/jail");
        let outside = PathBuf::from("/etc/passwd");
        assert!(!is_path_in_jail(&outside, &jail));
    }

    #[test]
    fn is_path_in_jail_accepts_inside() {
        let tmp = std::env::temp_dir();
        let jail = tmp.join("acp_jail_test");
        std::fs::create_dir_all(&jail).unwrap();
        let inside = jail.join("file.txt");
        std::fs::write(&inside, "test").unwrap();
        assert!(is_path_in_jail(&inside, &jail));
        std::fs::remove_dir_all(&jail).ok();
    }

    #[test]
    fn is_path_in_jail_rejects_nonexistent() {
        let jail = PathBuf::from("/tmp/jail");
        let ghost = PathBuf::from("/tmp/jail/nonexistent_deep/path");
        assert!(!is_path_in_jail(&ghost, &jail));
    }

    #[test]
    fn acp_transport_backend_is_claude_code() {
        let cfg = RuntimeFactoryConfig::for_backend(RuntimeBackend::ClaudeCode);
        let transport = AcpTransport::new(cfg);
        assert_eq!(transport.backend(), RuntimeBackend::ClaudeCode);
    }

    #[test]
    fn acp_transport_capabilities_reported() {
        let cfg = RuntimeFactoryConfig::for_backend(RuntimeBackend::ClaudeCode);
        let transport = AcpTransport::new(cfg);
        let caps = transport.capabilities();
        assert!(caps.supports_tools);
        assert!(caps.supports_streaming);
        assert!(!caps.supports_vision);
    }

    #[test]
    fn build_argv_includes_cwd_and_model() {
        let cfg = RuntimeFactoryConfig {
            backend: RuntimeBackend::ClaudeCode,
            cwd: Some(PathBuf::from("/tmp/work")),
            model: Some("claude-sonnet-4".into()),
            ..RuntimeFactoryConfig::for_backend(RuntimeBackend::ClaudeCode)
        };
        let args = build_argv(&cfg);
        assert!(args.contains(&"--cwd".to_string()));
        assert!(args.contains(&"/tmp/work".to_string()));
        assert!(args.contains(&"--model".to_string()));
        assert!(args.contains(&"claude-sonnet-4".to_string()));
    }

    #[test]
    fn create_runtime_with_acp_transport_routes_correctly() {
        let cfg = RuntimeFactoryConfig {
            backend: RuntimeBackend::ClaudeCode,
            transport: super::super::types::BridgeTransport::Acp,
            ..RuntimeFactoryConfig::for_backend(RuntimeBackend::ClaudeCode)
        };
        let runtime = super::super::create_runtime(cfg);
        assert!(runtime.is_ok());
        let runtime = runtime.unwrap();
        assert_eq!(runtime.backend(), RuntimeBackend::ClaudeCode);
    }

    #[test]
    fn create_runtime_with_direct_cli_routes_correctly() {
        let cfg = RuntimeFactoryConfig {
            backend: RuntimeBackend::ClaudeCode,
            transport: super::super::types::BridgeTransport::DirectCli,
            ..RuntimeFactoryConfig::for_backend(RuntimeBackend::ClaudeCode)
        };
        let runtime = super::super::create_runtime(cfg);
        assert!(runtime.is_ok());
        let runtime = runtime.unwrap();
        assert_eq!(runtime.backend(), RuntimeBackend::ClaudeCode);
    }
}

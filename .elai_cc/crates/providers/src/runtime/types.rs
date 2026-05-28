//! Public types for the multi-runtime adapter layer (B-37 Phase 4M).
//!
//! Scope of B-37A:
//! - `RuntimeBackend` enum (11 canonical variants).
//! - `RuntimeAdapter` trait shape (async via `Pin<Box<dyn Future>>` so
//!   the trait stays `dyn`-compatible without pulling in `async-trait`).
//! - Companion types: `Message`, `MessageRole`, `CompletionConfig`,
//!   `CompletionResponse`, `StreamChunk`, `RuntimeCapabilities`,
//!   `RuntimeHealth`, `RuntimeFactoryConfig`, `RuntimePermissionMode`,
//!   `RuntimeAdapterError`, `ResolveBackendError`, `AuthSource`,
//!   `SidecarRuntimeConfig`.
//!
//! Naming decisions (per blast-radius.md):
//! - `PermissionMode` in spec -> `RuntimePermissionMode` here (avoids
//!   collision with `orchestrator::permissions::PermissionMode`).
//! - `RuntimeError` in spec -> `RuntimeAdapterError` here (avoids
//!   collision with `orchestrator::conversation::RuntimeError`).
//!
//! Why `Pin<Box<dyn Future + Send>>` not `async fn`:
//! Rust 1.75 supports `async fn` in traits, but combining that with
//! `Box<dyn RuntimeAdapter>` requires `return_position_impl_trait_in_trait`
//! lifetime captures that are still subtle in edition 2021. Returning
//! `Pin<Box<dyn Future<...> + Send + 'static>>` makes the trait
//! unambiguously `dyn`-compatible and matches the existing `LlmFuture`
//! pattern in `providers::ollama` (the codebase's anti-stale-training
//! exception: native `async fn` would be ideal but compat with the
//! existing dyn-dispatched provider surface is more important).
//!
//! No external dependencies were added by this module; `futures_util`
//! is a workspace-promoted transitive (web_search already pins it).

use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;

use futures_util::stream::BoxStream;
use serde::{Deserialize, Serialize};

use crate::providers::anthropic::AuthSource as AnthropicAuthSource;
use crate::ApiError;

use super::secret::SecretString;

/// The 11 runtime backends ELAI exposes through the adapter layer.
///
/// Two existing (Sidecar, AnthropicApi), seven ouroboros-derived CLI
/// runtimes (ClaudeCode, CodexCli, OpenCode, Hermes, GeminiCli, KiroCli,
/// CopilotCli), two native HTTP (Ollama, Vllm). B-37A wires Sidecar +
/// AnthropicApi end-to-end; the remaining 9 are stubs whose
/// `health_check` returns `Unavailable`.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeBackend {
    /// Local sidecar (HuggingFace transformers in `python/elai_inference`).
    Sidecar,
    /// Anthropic Cloud API via `AnthropicClient`.
    AnthropicApi,
    /// `claude` CLI via Anthropic's `claude_agent_sdk` (stub in B-37A).
    ClaudeCode,
    /// `codex` CLI (stub in B-37A).
    CodexCli,
    /// `opencode` CLI (stub in B-37A).
    OpenCode,
    /// `hermes` CLI (stub in B-37A).
    Hermes,
    /// `gemini` CLI (stub in B-37A).
    GeminiCli,
    /// `kiro-cli` CLI (stub in B-37A).
    KiroCli,
    /// `copilot` CLI (stub in B-37A).
    CopilotCli,
    /// Ollama HTTP (`/api/chat` — stub in B-37A; existing
    /// `OllamaProvider` still serves the `/api/generate` shape).
    Ollama,
    /// vLLM HTTP (`/v1/chat/completions` — stub in B-37A).
    Vllm,
    /// Apple MLX fast-path via vllm-mlx subprocess on Darwin+arm64 (E-51).
    /// Speaks Anthropic /v1/messages. Refused on non-Apple-Silicon platforms.
    Mlx,
    /// Universal GGUF fallback via llama-server subprocess (E-51).
    /// Auto-detects best EP: Vulkan / CUDA / Metal / HIP / CPU.
    LlamaCpp,
    /// D-2 — Process Preference Model sidecar (Qwen2.5-Math-PRM-7B on port
    /// 8101). VERIFIER-ONLY backend: surfaces in `/runtime list` discovery
    /// but PerRoleRuntime::default() does NOT map any AgentRole to it (the
    /// PPM has no /v1/messages completion handler — score requests go
    /// through `crate::http_sidecar_ppm_hook::HttpSidecarPpmHook` directly).
    /// Per workflow/reasoning/D-2-decisions.md "PpmClient does NOT impl
    /// RuntimeAdapter" deviation, `create_runtime` for `Ppm` returns
    /// `NotImplemented` (intentional sentinel — guards against accidental
    /// generation-path routing).
    Ppm,
    /// C-44 — Cursor Composer 2 cloud API (paid HTTP backend; Cursor
    /// Research's coding-specialised model). Paper grounding
    /// arXiv:2603.24477 covers ONLY the model's training methodology +
    /// headline benchmarks; integration design is source-grounded via
    /// @cursor/sdk + cursor.com/docs/api. See workflow/papers/composer2-
    /// interface.md. OPT-IN per project via [runtime.composer2].enabled
    /// = true; default routing does NOT include composer2 to keep
    /// local-first sessions free of marginal cost.
    Composer2,
    /// E-40 — Language Server Protocol (LSP) subprocess oracle for
    /// `Unresolved`-confidence symbol resolution. Spawns rust-analyzer
    /// / pylsp / typescript-language-server / gopls on-demand and reads
    /// the JSON-RPC response. NOT a completion-routing backend (no
    /// `/v1/messages` surface) — `create_runtime` returns
    /// `NotImplemented` like `Ppm`. The actual oracle dispatch lives in
    /// `elai_orchestrator::symbol_graph::lsp_oracle::oracle_lookup`,
    /// which uses tokio::process::Command directly rather than the
    /// completion-shaped `RuntimeAdapter` trait. The slot exists here so
    /// `/runtime list` surfaces "lsp" + the cost-model registry has a
    /// stable id to attribute toolchain subprocess time against. Live
    /// integration tests RUNPOD-PENDING — require an installed
    /// toolchain (rust-analyzer / pylsp / etc.). See
    /// workflow/reasoning/E-40-decisions.md.
    Lsp,
    /// E-26 (inference-engines 2026 pack) — SGLang HTTP runtime adapter.
    /// Speaks OpenAI `/v1/chat/completions` with `extra_body.backend`
    /// selecting XGrammar / Outlines / LMFE for structured output. Default
    /// base URL `http://localhost:30000` (sgl-project/sglang docs v0.4+).
    /// Disambiguates from the media-validator `SglangConfig` (request-local
    /// VLM path); this slot is the long-lived health-probed cloud
    /// throughput adapter. See `crates/providers/src/runtime/sglang.rs`.
    Sglang,
    /// E-39 — DeepSeek V4-Flash cloud-floor backend. Cheapest serious-
    /// throughput cloud model the cost-router can route to under the
    /// `(hard + tight)` cell. Anti-slopsquatting verification + live
    /// adapter wiring are gated on `RUNPOD-PENDING: deepseek_v4_flash`;
    /// `create_runtime` currently returns `NotImplemented` so accidental
    /// generation-path routing surfaces a typed sentinel rather than
    /// panicking. Per `api-docs.deepseek.com/quick_start/pricing`
    /// (verified May 2026): $0.30 / $0.50 per MTok with 90% cache
    /// discount. See `workflow/reasoning/E-39-decisions.md`.
    DeepSeekV4Flash,
    /// E-39 — Cerebras free-tier slot (1M tokens / day since April 2026
    /// per `getaiperks.com/en/ai/cerebras-free-tier-guide`, verified
    /// May 2026). Daily-quota guard at
    /// [`crate::cost::free_tier_guard::FreeTierGuard`] refuses dispatch
    /// at >=95% of quota; warns at >=80%. Anti-slopsquatting verification
    /// + live adapter wiring are gated on `RUNPOD-PENDING: cerebras_free`;
    /// `create_runtime` returns `NotImplemented` until verification
    /// completes. See `workflow/reasoning/E-39-decisions.md`.
    CerebrasFree,
    /// E-39 — Groq free-tier slot (14,400 req / day default per
    /// `tokenmix.ai/blog/groq-free-tier-limits-2026`, verified May
    /// 2026). Hosts Llama 4 Scout / Qwen3 32B / Kimi K2 on the free
    /// path. Anti-slopsquatting verification + live adapter wiring are
    /// gated on `RUNPOD-PENDING: groq_free`; `create_runtime` returns
    /// `NotImplemented` until verification completes. See
    /// `workflow/reasoning/E-39-decisions.md`.
    GroqFree,
}

impl RuntimeBackend {
    /// Stable, lowercase, snake_case identifier used in error messages
    /// and config files.
    #[must_use]
    pub const fn canonical_name(self) -> &'static str {
        match self {
            Self::Sidecar => "sidecar",
            Self::AnthropicApi => "anthropic_api",
            Self::ClaudeCode => "claude_code",
            Self::CodexCli => "codex_cli",
            Self::OpenCode => "opencode",
            Self::Hermes => "hermes",
            Self::GeminiCli => "gemini_cli",
            Self::KiroCli => "kiro_cli",
            Self::CopilotCli => "copilot_cli",
            Self::Ollama => "ollama",
            Self::Vllm => "vllm",
            Self::Mlx => "mlx",
            Self::LlamaCpp => "llama_cpp",
            Self::Ppm => "ppm",
            Self::Composer2 => "composer2",
            Self::Lsp => "lsp",
            Self::Sglang => "sglang",
            // E-39 -- 3 new backends, anti-slopsquatting verification
            // RUNPOD-PENDING; create_runtime returns NotImplemented until
            // live URLs + model IDs are confirmed (api-docs.deepseek.com /
            // cloud.cerebras.ai / console.groq.com).
            Self::DeepSeekV4Flash => "deepseek_v4_flash",
            Self::CerebrasFree => "cerebras_free",
            Self::GroqFree => "groq_free",
        }
    }

    /// Iterator over every canonical name. Order is fixed so error
    /// messages and config dumps stay deterministic.
    #[must_use]
    pub fn all() -> &'static [RuntimeBackend] {
        &[
            Self::Sidecar,
            Self::AnthropicApi,
            Self::ClaudeCode,
            Self::CodexCli,
            Self::OpenCode,
            Self::Hermes,
            Self::GeminiCli,
            Self::KiroCli,
            Self::CopilotCli,
            Self::Ollama,
            Self::Vllm,
            Self::Mlx,
            Self::LlamaCpp,
            Self::Ppm,
            Self::Composer2,
            Self::Lsp,
            Self::Sglang,
            // E-39 -- 3 new backends, anti-slopsquatting RUNPOD-PENDING.
            Self::DeepSeekV4Flash,
            Self::CerebrasFree,
            Self::GroqFree,
        ]
    }
}

/// Permission policy applied to CLI subprocess agents.
///
/// `AcceptEdits` is the default — `DangerouslySkipPermissions` MUST
/// only be selected through explicit user opt-in (see B-plan
/// `stub_ban`).
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
)]
#[serde(rename_all = "camelCase")]
pub enum RuntimePermissionMode {
    /// Auto-accept tool edits / file writes.
    AcceptEdits,
    /// Plan mode — adapter is read-only; refuses tool use.
    Plan,
    /// Skip permission prompts entirely. Reserved for explicit user
    /// opt-in; never the default.
    DangerouslySkipPermissions,
    /// Backend-specific default behaviour.
    Default,
}

impl Default for RuntimePermissionMode {
    fn default() -> Self {
        Self::AcceptEdits
    }
}

/// Capabilities advertised by an adapter. Consumers gate optional
/// features (streaming, tool use, etc.) on this struct.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeCapabilities {
    pub supports_tools: bool,
    pub supports_streaming: bool,
    pub supports_vision: bool,
    pub max_context: u32,
    pub max_output: u32,
    pub json_schema_mode: bool,
    pub supports_prefill: bool,
    pub supports_caching: bool,
    pub native_thinking_budget: bool,
}

impl RuntimeCapabilities {
    /// All-false capabilities. Used as the conservative default for
    /// stub adapters until B-37B/C/D fills in real capability data.
    #[must_use]
    pub const fn none() -> Self {
        Self {
            supports_tools: false,
            supports_streaming: false,
            supports_vision: false,
            max_context: 0,
            max_output: 0,
            json_schema_mode: false,
            supports_prefill: false,
            supports_caching: false,
            native_thinking_budget: false,
        }
    }
}

/// Health-check outcome for an adapter. `Unavailable` MUST be used
/// instead of panicking when the underlying runtime is missing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum RuntimeHealth {
    /// Healthy and ready to accept work.
    Ready,
    /// Reachable but reporting degraded operation (e.g. slow response,
    /// reduced context window).
    Degraded { reason: String },
    /// Not reachable / not configured / not yet implemented.
    Unavailable { reason: String },
}

/// Source of authentication for cloud-style adapters. The B-37A
/// `AnthropicApiRuntime` accepts either an inline API key or a
/// pre-built `AuthSource` (e.g. OAuth bearer) -- which we feed straight
/// into `AnthropicClient::from_auth`.
#[derive(Debug, Clone)]
pub enum AuthSource {
    /// Plain API key. Wrapped in `SecretString` so it never leaks via
    /// Debug / Display.
    ApiKey(SecretString),
    /// Anthropic OAuth bearer token. Same `SecretString` guarantee.
    BearerToken(SecretString),
    /// API key + OAuth bearer (Claude Code OAuth path).
    ApiKeyAndBearer {
        api_key: SecretString,
        bearer_token: SecretString,
    },
}

impl From<AuthSource> for AnthropicAuthSource {
    fn from(value: AuthSource) -> Self {
        match value {
            AuthSource::ApiKey(key) => AnthropicAuthSource::ApiKey(key.into_inner()),
            AuthSource::BearerToken(token) => AnthropicAuthSource::BearerToken(token.into_inner()),
            AuthSource::ApiKeyAndBearer {
                api_key,
                bearer_token,
            } => AnthropicAuthSource::ApiKeyAndBearer {
                api_key: api_key.into_inner(),
                bearer_token: bearer_token.into_inner(),
            },
        }
    }
}

/// Sidecar-specific runtime config. Carved out of `SidecarConfig` so
/// `RuntimeFactoryConfig` doesn't pull every sidecar steering knob into
/// the shared factory surface.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SidecarRuntimeConfig {
    /// Sidecar base URL (defaults to `DEFAULT_SIDECAR_URL`).
    pub server_url: Option<String>,
    /// Request timeout in seconds. `None` keeps the sidecar default.
    pub request_timeout_secs: Option<u64>,
    /// Max tokens per completion request.
    pub max_tokens: Option<u32>,
}

impl Default for SidecarRuntimeConfig {
    fn default() -> Self {
        Self {
            server_url: None,
            request_timeout_secs: None,
            max_tokens: None,
        }
    }
}

/// Factory configuration consumed by `create_runtime`. Every field is
/// optional except `backend`; the adapter constructor decides which
/// fields it actually needs and surfaces a `RuntimeAdapterError` if a
/// required one is missing.
#[derive(Debug, Clone)]
pub struct RuntimeFactoryConfig {
    pub backend: RuntimeBackend,
    pub cli_path: Option<PathBuf>,
    pub base_url: Option<String>,
    pub api_key: Option<SecretString>,
    pub auth_source: Option<AuthSource>,
    pub model: Option<String>,
    pub permission_mode: RuntimePermissionMode,
    pub runtime_profile: Option<String>,
    pub cwd: Option<PathBuf>,
    pub startup_output_timeout_seconds: Option<f64>,
    pub stdout_idle_timeout_seconds: Option<f64>,
    pub sidecar: Option<SidecarRuntimeConfig>,
    /// Explicit opt-in required before any CLI adapter accepts
    /// `RuntimePermissionMode::DangerouslySkipPermissions`. Default is
    /// `false`; adapters MUST refuse the dangerous mode with
    /// `RuntimeAdapterError::PermissionDenied` when this is `false`.
    /// See B-37B-decisions.md MEDIUM-1.
    pub allow_danger: bool,
}

impl RuntimeFactoryConfig {
    /// Convenience constructor for the common "just pick a backend"
    /// case. Other fields default to `None`/`Default::default()`.
    #[must_use]
    pub fn for_backend(backend: RuntimeBackend) -> Self {
        Self {
            backend,
            cli_path: None,
            base_url: None,
            api_key: None,
            auth_source: None,
            model: None,
            permission_mode: RuntimePermissionMode::default(),
            runtime_profile: None,
            cwd: None,
            startup_output_timeout_seconds: None,
            stdout_idle_timeout_seconds: None,
            sidecar: None,
            allow_danger: false,
        }
    }
}

/// Anthropic-shaped role enum. `tool` is a placeholder for future
/// tool-result messages; B-37A only emits `user`/`assistant`/`system`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    User,
    Assistant,
    System,
    Tool,
}

impl MessageRole {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Assistant => "assistant",
            Self::System => "system",
            Self::Tool => "tool",
        }
    }
}

/// Cross-adapter message envelope. Each adapter translates from this
/// shape into its native wire form (Anthropic InputMessage, Ollama
/// `/api/chat` message, etc.).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Message {
    pub role: MessageRole,
    pub content: String,
}

/// Generation parameters. Fields kept minimal in B-37A — later
/// sub-cycles (notably B-37C native HTTP) will extend this.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompletionConfig {
    pub model: Option<String>,
    pub max_tokens: u32,
    pub temperature: f32,
    pub seed: Option<u64>,
    pub stop_sequences: Vec<String>,
    /// Optional system prompt forwarded to backends that support it
    /// (Anthropic API, sidecar — both speak the same protocol).
    pub system: Option<String>,
}

impl Default for CompletionConfig {
    fn default() -> Self {
        Self {
            model: None,
            max_tokens: 512,
            temperature: 0.0,
            seed: None,
            stop_sequences: Vec::new(),
            system: None,
        }
    }
}

/// Adapter response. Mirrors the subset of Anthropic's
/// `MessageResponse` the orchestrator needs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompletionResponse {
    /// Concatenated assistant text output.
    pub text: String,
    /// Provider-reported stop reason (e.g. "end_turn", "max_tokens").
    pub stop_reason: Option<String>,
    /// Tokens used. `(input, output)`. `0/0` when the adapter cannot
    /// report token counts.
    pub usage: (u32, u32),
    /// Echoed model name (post any aliasing).
    pub model: String,
}

/// Streaming chunk emitted by `RuntimeAdapter::stream`. The variant set
/// is small in B-37A — later sub-cycles will add tool-call deltas
/// once the CLI adapters land.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StreamChunk {
    /// Incremental text token.
    TextDelta { text: String },
    /// Backend signalled completion with an optional stop reason.
    Stop { reason: Option<String> },
    /// Non-fatal warning surfaced from the adapter (e.g. recoverable
    /// rate limit). Errors that abort the stream are surfaced through
    /// `Err(RuntimeAdapterError)` instead.
    Warning { message: String },
}

/// All runtime adapter errors. Constructed via `thiserror` so the
/// existing `tracing::error!` flow renders sensible messages.
#[derive(Debug, thiserror::Error)]
pub enum RuntimeAdapterError {
    /// Stub backend with no real implementation yet (the B-37A
    /// promise: stubs return this rather than panicking).
    #[error("runtime backend {backend:?} not implemented yet ({hint})")]
    NotImplemented {
        backend: RuntimeBackend,
        hint: &'static str,
    },
    /// Required factory field missing (e.g. `auth_source` for
    /// AnthropicApi).
    #[error("runtime backend {backend:?} requires field `{field}`: {reason}")]
    MissingField {
        backend: RuntimeBackend,
        field: &'static str,
        reason: String,
    },
    /// Transport / parse failure inside an upstream provider call.
    #[error("upstream provider error: {0}")]
    Provider(#[from] ApiError),
    /// Network / HTTP failure outside the `ApiError` family.
    #[error("network failure: {0}")]
    Network(String),
    /// Health probe timed out.
    #[error("health check timed out after {seconds}s")]
    HealthTimeout { seconds: u64 },
    /// CLI binary not found on PATH and no explicit `cli_path` provided
    /// (B-37B). Surfaces through `health_check` as
    /// `RuntimeHealth::Unavailable` rather than this error variant; the
    /// error is for the `complete` / `stream` call paths.
    #[error("CLI binary `{name}` not found (searched: {searched_path})")]
    BinaryNotFound {
        name: String,
        searched_path: String,
    },
    /// `tokio::process::Command::spawn` failed.
    #[error("failed to spawn `{binary}`: {source}")]
    SubprocessSpawn {
        binary: String,
        #[source]
        source: std::io::Error,
    },
    /// CLI did not emit any stdout within
    /// `startup_output_timeout_seconds`.
    #[error("CLI `{binary}` produced no output within {seconds:.1}s (startup timeout)")]
    StartupTimeout { binary: String, seconds: f64 },
    /// CLI emitted some output but went silent for longer than
    /// `stdout_idle_timeout_seconds`.
    #[error("CLI `{binary}` idle for more than {seconds:.1}s (idle timeout)")]
    IdleTimeout { binary: String, seconds: f64 },
    /// CLI subprocess exited with a non-zero status.
    #[error("CLI `{binary}` exited with status {code:?}: {stderr_tail}")]
    ExitNonZero {
        binary: String,
        code: Option<i32>,
        stderr_tail: String,
    },
    /// `RuntimePermissionMode::DangerouslySkipPermissions` was requested
    /// without `RuntimeFactoryConfig.allow_danger=true`. See B-37
    /// `stub_ban`: "PermissionMode default MUST be AcceptEdits, NEVER
    /// DangerouslySkipPermissions without explicit user opt-in."
    #[error("permission denied: {reason}")]
    PermissionDenied { reason: String },
    /// Failed to parse the CLI's structured output (JSONL, stream-json,
    /// etc.) into the adapter's normalised event shape.
    #[error("CLI `{binary}` protocol parse error on line `{line}`: {source}")]
    ProtocolParseError {
        binary: String,
        line: String,
        #[source]
        source: serde_json::Error,
    },
    /// Generic catch-all used by adapters that have a single domain
    /// error type internally.
    #[error("runtime adapter error: {0}")]
    Other(String),
}

/// `resolve_backend` miss. Carries the original candidate (lowercased
/// for stable error messages) and the full canonical-name list.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("unsupported runtime backend `{candidate}`; supported: {}", supported.join(", "))]
pub struct ResolveBackendError {
    pub candidate: String,
    pub supported: Vec<String>,
}

/// Boxed future / stream aliases so call sites stay readable.
pub type RuntimeFuture<'a, T> =
    Pin<Box<dyn Future<Output = Result<T, RuntimeAdapterError>> + Send + 'a>>;
/// `health_check` always succeeds — failure is encoded as
/// `RuntimeHealth::Unavailable`, never an `Err`. So the future is
/// infallible and we use a dedicated alias.
pub type RuntimeHealthFuture<'a> = Pin<Box<dyn Future<Output = RuntimeHealth> + Send + 'a>>;
pub type RuntimeStream<'a> = BoxStream<'a, Result<StreamChunk, RuntimeAdapterError>>;

/// Object-safe runtime adapter trait. Implementors translate the
/// Anthropic-shaped `Message`/`CompletionConfig` envelope into their
/// backend-native form. All async methods return boxed futures so
/// `Box<dyn RuntimeAdapter>` stays usable.
pub trait RuntimeAdapter: Send + Sync {
    /// Self-identification.
    fn backend(&self) -> RuntimeBackend;

    /// Adapter capabilities. Cheap; safe to call repeatedly.
    fn capabilities(&self) -> RuntimeCapabilities;

    /// Liveness / readiness probe. Adapters with no backing process
    /// MUST return `Unavailable`, never panic. Cannot fail — failure
    /// is encoded in `RuntimeHealth::Unavailable`.
    fn health_check(&self) -> RuntimeHealthFuture<'_>;

    /// Non-streaming completion.
    fn complete(
        &self,
        messages: Vec<Message>,
        config: CompletionConfig,
    ) -> RuntimeFuture<'_, CompletionResponse>;

    /// Streaming completion. The returned stream may yield `Warning`
    /// chunks before its final `Stop` chunk; consumers ignore unknown
    /// variants.
    fn stream(
        &self,
        messages: Vec<Message>,
        config: CompletionConfig,
    ) -> RuntimeStream<'static>;
}

/// `RuntimeHealth` helper used by stubs.
#[must_use]
pub fn unavailable(reason: impl Into<String>) -> RuntimeHealth {
    RuntimeHealth::Unavailable {
        reason: reason.into(),
    }
}

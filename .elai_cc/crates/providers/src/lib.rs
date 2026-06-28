mod client;
// I-50 (SCHEMA_VERSION 268): subscription auth broker (credential-resolve layer;
// SSB-emit wrapper lives in elai-orchestrator::auth_broker_emit to avoid circular dep).
pub mod auth_broker;
// C-26: constrained-decoding mode + per-role defaults. Re-exports below.
pub mod decoding;
// I-34: grammar-pin manifest with blake3 load-time verification.
pub mod grammar_store;
// I-51 — unified model capability catalog (models.toml + loader).
pub mod model_catalog;
mod error;
pub mod layer_config;
pub mod multiplexing;
pub mod oauth;
pub mod pricing;
mod prompt_cache;
mod providers;
pub mod runtime;
#[cfg(feature = "sidecar")]
pub mod sidecar_client;
mod sse;
pub mod telemetry_types;
// J-53: deterministic tool-input repair shim for the OpenAI-compat
// unmasked-decoder seam. Wired from openai_compat.rs::parse_tool_arguments.
pub mod tool_input_repair;
// J-53 step 5: re-export the ToolInputRepaired SSB action_type providers-side
// event + Kind discriminator so the orchestrator emit wrapper (following the
// auth_broker_emit pattern) can map them to the SSB journal row without a
// circular dep.
pub use tool_input_repair::{
    RepairKind, RepairOutcome, RelationalDefault, ToolInputRepairEvent, ToolInputRepairKind,
    apply_relational_defaults, repair_tool_input,
};
mod types;
#[cfg(feature = "turboquant")]
pub mod turboquant;
pub mod vram_tier;
#[cfg(feature = "vllm")]
pub mod vllm_client;

pub use client::{
    default_claude_code_oauth_config, oauth_token_is_expired, read_base_url, read_xai_base_url,
    resolve_auth_with_auto_refresh, resolve_saved_oauth_token, resolve_startup_auth_source,
    MessageStream, OAuthTokenSet, OAuthUsage, ProviderClient,
};
pub use oauth::{
    clear_oauth_credentials, cloud_model_id_allowlist, credentials_path,
    is_valid_cloud_model_id, load_cloud_model_id, load_oauth_credentials,
    save_cloud_model_id, save_oauth_credentials, CLOUD_MODEL_ID_ALLOWLIST,
    DEFAULT_CLOUD_MODEL_ID,
};
pub use decoding::{
    crane_adaptive_eligible, default_decoding_for, paper_decoding_params_for,
    CraneAdaptiveConfig, ValidationError, re_validate_output,
    ConstrainedDecodingConfig, DecodingMode, PaperDecodingParams, PaperRole, RoleDecoding,
    PER_ROLE_DECODING_DEFAULTS, PER_ROLE_DECODING_PARAMS_PAPER,
};
pub use grammar_store::{
    default_manifest_path, load_and_verify as load_and_verify_grammar_manifest,
    GrammarPinError, GrammarPinManifest, GrammarPinRecord,
};
pub use error::ApiError;
pub use prompt_cache::{
    CacheBreakEvent, PromptCache, PromptCacheConfig, PromptCachePaths, PromptCacheRecord,
    PromptCacheStats,
};
pub use providers::anthropic::{AnthropicClient, AnthropicClient as ApiClient, AuthSource};
// H-5 — Anthropic Message Batches non-interactive lane (50% pricing). Re-exported at
// the crate root so the orchestrator + contract tests reference a stable path.
pub use providers::anthropic_batch::{
    half_price_cents, AnthropicBatchClient, BatchId, BatchItem, BatchProcessingStatus,
    BatchResultItem, BatchResultOutcome, BatchStatus,
};
pub use providers::openai_compat::{
    build_chat_completion_request_for_test, OpenAiCompatClient, OpenAiCompatConfig,
};
pub use providers::ollama::{
    probe_speculative_support, LlmFuture, LlmProvider, OllamaConfig, OllamaEmbedder,
    OllamaProvider, ProbeResult, SpeculativeConfig,
};
pub use providers::{
    detect_provider_kind, max_tokens_for_model, resolve_model_alias, ProviderKind,
};
#[cfg(feature = "sidecar")]
pub use sidecar_client::{
    steering_cache_path, GGSignals, OptimizationSummary, SearchTriggerPayload, SidecarConfig,
    SidecarProvider, DEFAULT_SIDECAR_URL,
};
#[cfg(feature = "vllm")]
pub use vllm_client::{
    InferenceBackend, VllmConfig, VllmHealthStatus, VllmModelPool, VllmProvider,
};
pub use sse::{parse_frame, SseParser};
// C-14 — pricing/cost helpers re-exported at crate root so external consumers
// (elai_orchestrator umbrella + cli/* via that umbrella) keep their existing
// import names. Originals lived in `elai_core::usage` pre-C-14.
pub use pricing::{
    estimate_cost_usd, estimate_cost_usd_with_pricing, format_usd, pricing_for_model,
    summary_lines, summary_lines_for_model, ModelPricing, UsageCostEstimate,
};
pub use vram_tier::{detect_vram_gb, detect_vram_tier, VramDetectionError};
// I-51 — model catalog public surface.
pub use model_catalog::{CatalogVramThresholds, ModelCatalog, ModelCatalogError, ModelEntry};
pub use types::{
    CacheControl, ContentBlockDelta, ContentBlockDeltaEvent, ContentBlockStartEvent,
    ContentBlockStopEvent, InputContentBlock, InputMessage, MessageDelta, MessageDeltaEvent,
    MessageRequest, MessageResponse, MessageStartEvent, MessageStopEvent, OutputContentBlock,
    StreamEvent, SystemBlock, ToolChoice, ToolDefinition, ToolResultContentBlock, Usage,
};

pub use telemetry_types::{
    AnalyticsEvent, AnthropicRequestProfile, ClientIdentity, JsonlTelemetrySink,
    MemoryTelemetrySink, SessionTraceRecord, SessionTracer, TelemetryEvent, TelemetrySink,
    DEFAULT_ANTHROPIC_VERSION,
};

pub use multiplexing::{
    MultiplexingError, MultiplexingPolicy, PreloadPolicy, SloTier,
};

pub mod model_placer;
pub use model_placer::{
    GpuDiscoveryError, GpuSpec, GreedyModelPlacer, ModelSpec, PlacementResult,
    discover_gpus,
};
// I-50 (SCHEMA_VERSION 268): re-export auth broker public surface.
pub use auth_broker::{
    AnthropicAuthProvider, AuthBroker, AuthProvider, ResolveOutcome,
    broker_resolve_anthropic_auth,
};

#[cfg(test)]
fn test_env_lock() -> std::sync::MutexGuard<'static, ()> {
    use std::sync::{Mutex, OnceLock};
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

mod client;
mod error;
pub mod layer_config;
pub mod multiplexing;
pub mod oauth;
mod prompt_cache;
mod providers;
#[cfg(feature = "sidecar")]
pub mod sidecar_client;
mod sse;
pub mod telemetry_types;
mod types;
#[cfg(feature = "turboquant")]
pub mod turboquant;
pub mod vram_tier;
#[cfg(feature = "vllm")]
pub mod vllm_client;

pub use client::{
    oauth_token_is_expired, read_base_url, read_xai_base_url, resolve_saved_oauth_token,
    resolve_startup_auth_source, MessageStream, OAuthTokenSet, ProviderClient,
};
pub use oauth::{
    clear_oauth_credentials, cloud_model_id_allowlist, credentials_path,
    is_valid_cloud_model_id, load_cloud_model_id, load_oauth_credentials,
    save_cloud_model_id, save_oauth_credentials, CLOUD_MODEL_ID_ALLOWLIST,
    DEFAULT_CLOUD_MODEL_ID,
};
pub use error::ApiError;
pub use prompt_cache::{
    CacheBreakEvent, PromptCache, PromptCacheConfig, PromptCachePaths, PromptCacheRecord,
    PromptCacheStats,
};
pub use providers::anthropic::{AnthropicClient, AnthropicClient as ApiClient, AuthSource};
pub use providers::openai_compat::{OpenAiCompatClient, OpenAiCompatConfig};
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
pub use vram_tier::{detect_vram_tier, VramDetectionError};
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

#[cfg(test)]
fn test_env_lock() -> std::sync::MutexGuard<'static, ()> {
    use std::sync::{Mutex, OnceLock};
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

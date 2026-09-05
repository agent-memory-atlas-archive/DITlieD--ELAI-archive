Phase 3J.1-2 Decision Log

Endpoint verification (WebSearch 2026-04-09):
- /v1/load_model, /v1/unload_model: DO NOT EXIST in any vLLM version. GitHub issues #5491, #3281, #3308 are open feature requests. ModelPool will NOT attempt dynamic model management via HTTP.
- /health: EXISTS. Returns simple liveness response. Used for health check only.
- /v1/models: EXISTS. Returns OpenAI-compatible model list. Used to verify model availability.
- /v1/completions: EXISTS. OpenAI-compatible streaming endpoint. SSE format: {"choices":[{"delta":{"content":"token"}}]}.

LlmProvider impedance mismatch resolution:
- Decision: Option A — VllmProvider stores VllmConfig internally. complete() signature matches LlmProvider (takes &OllamaConfig) but ignores that param, uses self.config for server_url/model. No trait generalization needed. Documented in code comment.
- Rationale: Trait generalization would touch ollama.rs + all existing impls + downstream consumers. Out of scope for 3J.

ModelPool location:
- Trait defined in orchestrator/src/agent_pool.rs:54-57. Single method: available_vram() -> u64.
- InferenceBackend enum and concrete VllmModelPool impl go in providers crate. VllmModelPool::available_vram() returns u64::MAX (vLLM manages its own GPU memory via CUDA, no NVML queries from host).

ClockProvider injection:
- VllmProvider takes Arc<dyn ClockProvider> for latency measurement. Uses now()/elapsed_since() from core::clock. OsClock in production, SeededClock in tests (tick-only, duration returns Duration::ZERO).

SSE parsing:
- SseParser in sse.rs is typed to Anthropic StreamEvent (ContentBlockStart/Delta/MessageDelta). vLLM uses OpenAI SSE format with different JSON schema. Created VllmSseParser in vllm_client.rs with VllmChoice/VllmCompletionChunk structs. Does not modify sse.rs.

reqwest features:
- Adding "stream" feature to reqwest in providers/Cargo.toml. Required for Response::bytes_stream() used in SSE streaming.

Feature gate:
- All vLLM code behind cfg(feature = "vllm") in providers/Cargo.toml. Non-vllm builds compile without any vllm symbols.

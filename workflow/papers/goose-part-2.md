GOOSE Provider Implementations: Vendor Catalog & Integration Patterns

# 1. PROVIDER CATALOG

Goose abstracts a vendor factory pattern via ProviderDef trait + ProviderEntry registry.
Every provider implements Provider + ProviderDef; base.rs:11-90 defines the traits.

## Anthropic Provider
- **Vendor:** Anthropic
- **API Protocol:** JSON + Server-Sent Events (streaming)
- **Auth Mechanism:** API key header
  - Environment variable: ANTHROPIC_API_KEY
  - Header name: x-api-key (anthropic.rs:77)
  - Optional custom host override: ANTHROPIC_HOST (env default: https://api.anthropic.com)
- **Model Metadata:** Known models declared at lines 31-47 (claude-opus-4-6, claude-sonnet-4-6, etc.)
  - Default model: claude-sonnet-4-5
  - Fast model: claude-haiku-4-5
- **Token Usage:** Streaming via `stream_options` in response; parsed by formats/anthropic.rs
- **Special Features:**
  - Custom request format via AnthropicFormatOptions (thinking types, e.g., line 63)
  - Supports thinking_type negotiation (code:15-16)
  - API version: "2023-06-01" baked into auth headers (anthropic.rs:82)

## OpenAI Provider
- **Vendor:** OpenAI
- **API Protocol:** OpenAI chat.completions format (JSON + SSE streaming) + new Responses API (v1/responses)
- **Auth Mechanism:** Bearer token in Authorization header (openai.rs:40-41)
  - Environment variable: OPENAI_API_KEY
  - Optional custom host: OPENAI_BASE_URL (parsed by parse_openai_base_url, lines 88-101)
  - Supports query parameters forwarded from URL (openai.rs:74-98)
- **Model Metadata:** Extensive list at lines 42-67 (gpt-4o, o1, o3, future gpt-5 variants)
  - Dynamic context windows per model (128K-1.05M tokens)
  - Default: gpt-4o; fast: gpt-4o-mini
- **Token Usage:**
  - Standard completions: usage object in response (get_usage)
  - Responses API: streaming usage events (openai_responses.rs)
  - Custom handling for streaming_options (openai.rs:24 get_responses_usage)
- **Special Features:**
  - Supports both legacy `/v1/chat/completions` and new `/v1/responses` endpoints
  - Base URL parsing strips `v1/` prefix if present, allowing custom endpoints
  - Query parameters from OPENAI_BASE_URL are preserved across all requests (openai.rs:74-98)

## Azure OpenAI Provider
- **Vendor:** Microsoft Azure
- **API Protocol:** OpenAI-compatible chat endpoint
- **Auth Mechanism:** Dual-auth (API key or DefaultCredential)
  - API key env: AZURE_OPENAI_API_KEY (header: api-key)
  - DefaultCredential fallback: Azure SDK implicit (azureauth.rs:34-49)
  - Bearer token on OAuth path
- **Endpoint Configuration:**
  - Required: AZURE_OPENAI_ENDPOINT (base URL)
  - Required: AZURE_OPENAI_DEPLOYMENT_NAME (model deployment name)
  - Optional: AZURE_OPENAI_API_VERSION (default: 2024-10-21)
  - Auto-detection: v1 vs non-v1 endpoints (azure.rs:19-22)
- **Special Features:**
  - Wraps OpenAiCompatibleProvider; inherits token usage from parent
  - Distinguishes v1-style endpoints (no api-version param) vs old style (azure.rs:15-21)
  - Deployment name prepended to request path

## OpenAI-Compatible Provider (Generic)
- **Vendor:** Any OpenAI-compatible server (xAI/Grok, vLLM, local proxy, etc.)
- **API Protocol:** OpenAI chat.completions format
- **Auth Mechanism:** Bearer token (configurable)
  - Injected via api_client.rs AuthMethod::BearerToken
  - Host/port custom via constructor parameter
- **Token Usage:** Inherited from openai format; responds with standard usage object
- **Usage:** Base class for xAI, openrouter, etc. (openai_compatible.rs:29-84)

## xAI (Grok) Provider
- **Vendor:** xAI
- **API Protocol:** OpenAI-compatible (wraps OpenAiCompatibleProvider)
- **Auth Mechanism:** Bearer token
  - Environment variable: XAI_API_KEY
  - Custom host override: XAI_HOST (default: https://api.x.ai/v1)
- **Model Metadata:** 18 known Grok models at xai.rs:11-30 (grok-code-fast-1, grok-4-0709, etc.)
- **Special Features:**
  - No custom path prefix (xai.rs:71 empty string for completions_prefix)
  - Direct OpenAI-compatible API

## Bedrock Provider (AWS)
- **Vendor:** AWS Bedrock
- **API Protocol:** Native AWS SDK (bedrock-runtime)
- **Auth Mechanism:** AWS credential chain (IAM, SSO, environment variables)
  - Optional bearer token override: AWS_BEARER_TOKEN_BEDROCK (bedrock.rs:81-91)
  - Region: AWS_REGION environment variable or SDK auto-detect
  - All AWS_* prefixed env vars passed through to SDK (bedrock.rs:60-78)
- **Model Metadata:** Bedrock-specific model IDs at bedrock.rs:31-38
  - Format: us.anthropic.claude-sonnet-4-5-20250929-v1:0
  - Defaults to Claude models, but supports Llama, Mistral, etc.
- **Token Usage:** Extracted from ConversationResponse (bedrock.rs)
  - via from_bedrock_usage in formats/bedrock.rs
- **Special Features:**
  - Retry config with exponential backoff (bedrock.rs:40-43)
  - Cache control markers for prompt caching
  - Uses smithy HTTP client (bedrock.rs:19)

## Azure Bedrock (Alternative AWS Auth)
- **Auth Mechanism:** Bearer token via Azure managed identity
  - AWS credentials embedded in Azure bearer token (alternative to IAM)
  - bedrock.rs:81-91 checks for AWS_BEARER_TOKEN_BEDROCK first

## Databricks Provider
- **Vendor:** Databricks Model Serving
- **API Protocol:** OpenAI-compatible chat endpoint
- **Auth Mechanism:** OAuth + optional PAT fallback
  - Primary: DatabricksAuth (databricks_auth.rs) OAuth flow
  - Fallback: Databricks personal access token (DATABRICKS_TOKEN)
  - Host: DATABRICKS_WORKSPACE_URL
  - Token cache: databricks.rs:92 (in-memory Arc<Mutex<Option<String>>>)
- **Model Metadata:** Endpoint discovery via `/api/2.0/serving-endpoints` call
  - Caches endpoint metadata (TTL 60s, databricks.rs:61-64)
  - Upstream model resolution (e.g., "databricks-claude-sonnet-4" -> actual Claude model)
- **Token Usage:** Inherited from OpenAI-compatible format
- **Special Features:**
  - Automatic token refresh on 401 (databricks_auth.rs)
  - Upstream model detection (databricks.rs:42-47 DatabricksEndpointInfo)
  - Supports reasoning models flag

## Databricks V2 Provider (Newer)
- **Auth Mechanism:** Same as v1, but revised implementation
- **Special:** Alternative endpoint discovery path (databricks_v2.rs)

## Ollama Provider (Local)
- **Vendor:** Local Ollama service
- **API Protocol:** OpenAI-compatible chat endpoint
- **Auth Mechanism:** None (local service)
  - Host: OLLAMA_HOST (default: localhost:11434)
  - No authentication required
- **Model Metadata:** Dynamic discovery from Ollama model library
  - Known defaults at ollama.rs:36-42 (qwen3, qwen3-coder, etc.)
- **Token Usage:** Optional streaming via stream_options (OLLAMA_STREAM_USAGE env var, default true)
  - ollama.rs:79-95 demonstrates fallback for older Ollama builds
  - Gracefully downgrades to non-streaming usage on 500 timeout (ollama.rs:104-106)
- **Context Window:** Configurable via GOOSE_INPUT_LIMIT env var (ollama.rs:64)
  - Converted to Ollama's options.num_ctx (ollama.rs:97-122)
- **Special Features:**
  - Retry logic tuned for model loading delays (10 retries, 2-15s backoff, ollama.rs:48-51)
  - Handles max_completion_tokens -> options.num_predict translation
  - XML tool call fallback for Qwen3-coder (formats/ollama.rs:35-79)

## Google Vertex AI / Gemini Provider
- **Vendor:** Google Cloud (Vertex AI)
- **API Protocol:** Google native (not OpenAI-compatible)
- **Auth Mechanism:** GCP credential chain
  - Primary: Application Default Credentials (ADC)
  - Service account key: GOOGLE_APPLICATION_CREDENTIALS env var
  - Custom credentials via gcpauth.rs (39+ KB auth implementation)
  - GCP project ID: GOOGLE_CLOUD_PROJECT env var
- **Model Metadata:** Vertex AI model names (gemini-2.0-pro, etc.)
- **Token Usage:** Extracted from Google response format (formats/gcpvertexai.rs)
- **Special Features:**
  - Full IAM credential handling (gcpauth.rs)
  - Supports different response formats (streaming, non-streaming)

## Gemini OAuth Provider (CLI-style)
- **Vendor:** Google Gemini (via Code Assist API)
- **API Protocol:** Google Code Assist API (proprietary, not chat.completions)
- **Auth Mechanism:** OAuth 2.0 installed-app flow
  - Client ID/secret: Public defaults from Gemini CLI (gemini_oauth.rs:56-76)
  - Device flow: OAuth browser authorization to https://accounts.google.com
  - Token storage: OAuth cache file in user config dir
  - Scopes: cloud-platform + userinfo.email (lines 85-88)
- **Model Metadata:** Code Assist API models (gemini-3-pro-preview, gemini-2.5-flash)
- **Token Usage:** Extracted from Google response
- **Special Features:**
  - Local OAuth callback server (lines 20-36 axum router)
  - 5-minute auth timeout with 2-second auto-close on success (lines 90-91)
  - Assembled OAuth credentials to bypass GitHub secret scanners (lines 60-75)

## HuggingFace Provider
- **Vendor:** HuggingFace Inference API
- **API Protocol:** HF native + OpenAI-compatible fallback
- **Auth Mechanism:** HuggingFace API token
  - Environment variable: HUGGING_FACE_HUB_TOKEN or HF_TOKEN
  - Token validation: huggingface_auth.rs (28 KB implementation)
- **Model Metadata:** Any HF model endpoint
- **Token Usage:** HF response format extraction
- **Special Features:**
  - Model availability checking (huggingface.rs)
  - Optional OAuth refresh (huggingface_auth.rs)

## LiteLLM Provider
- **Vendor:** LiteLLM proxy service
- **API Protocol:** OpenAI-compatible
- **Auth Mechanism:** Optional API key + custom headers
  - Auth: LITELLM_API_KEY (empty = no auth)
  - Host: LITELLM_HOST (default: https://api.litellm.ai)
  - Base path: LITELLM_BASE_PATH (default: v1/chat/completions)
  - Custom headers: LITELLM_CUSTOM_HEADERS (parsed JSON)
  - Timeout: LITELLM_TIMEOUT env var (litellm.rs:57-59)
- **Model Metadata:** Dynamic fetch from /model/info endpoint (litellm.rs:96-100)
- **Token Usage:** Inherited from OpenAI-compatible format
- **Special Features:**
  - OnceCell model cache (avoid repeated /model/info calls)
  - Custom header injection via reqwest HeaderMap (litellm.rs:70-78)

## OpenRouter Provider
- **Vendor:** OpenRouter (multi-vendor aggregator)
- **API Protocol:** OpenAI-compatible
- **Auth Mechanism:** Bearer token (OpenRouter API key)
- **Model Metadata:** OpenRouter model list
- **Special:** Inherits from OpenAI-compatible provider

## Snowflake Provider
- **Vendor:** Snowflake Cortex
- **API Protocol:** Snowflake native
- **Auth Mechanism:** Snowflake account credentials (username + password or OAuth)
- **Model Metadata:** Cortex model names
- **Token Usage:** Snowflake cost/token format

## Sagemaker TGI Provider (AWS)
- **Vendor:** AWS SageMaker with Text Generation Inference
- **API Protocol:** TGI OpenAI-compatible format
- **Auth Mechanism:** AWS credential chain
- **Special:** Designed for self-hosted TGI on SageMaker endpoints

## Local Inference Provider
- **Vendor:** Local GGML-based models (via llama.cpp)
- **API Protocol:** In-process inference via Rust bindings
- **Auth Mechanism:** None (local binary)
- **Backends:** llama.cpp (LLAMACPP_BACKEND_ID), extensible for vLLM
- **Model Caching:** Per-model loading with ModelCacheKey (backend_id + model_id + chat_template)
- **Token Usage:** Estimated via ProviderUsage (fallback if no native usage data)
- **Special Features:**
  - Global InferenceRuntime singleton (local_inference.rs:68)
  - Chat template support (built-in + custom)
  - Multimodal model support (local_inference/multimodal.rs)

## Additional Providers (ACP Tooling, Code Assist integrations)
- **Claude ACP / Code ACP / Copilot ACP:** IDE integrations (claude_acp.rs, copilot_acp.rs, etc.)
  - Auth via IDE token store (Keychain, Windows Credential Manager, etc.)
- **Cursor Agent / Kimicode / NanoGPT / Tetrate:** Vendor-specific (cursor_agent.rs, kimicode.rs, etc.)


---

# 2. BYOK MECHANICS (Bring Your Own Key)

All providers read credentials from environment OR config file:

## API Key Injection Pattern (Universal)
1. **env var precedent:** config.get_secret("ENV_VAR_NAME") attempts env first
2. **config file fallback:** if not found in env, reads from ~/.config/goose/config.toml or platform-specific location
3. **no hardcoded defaults:** all prod APIs require explicit key (test providers use hardcoded dummy keys)

### Example Flow (anthropic.rs:67-74)
```
let api_key: String = config.get_secret("ANTHROPIC_API_KEY")?;  // env first
let host: String = config.get_param("ANTHROPIC_HOST")           // env or config
    .unwrap_or_else(|_| "https://api.anthropic.com".to_string()); // default
```

## Base URL Overrides
- **OpenAI:** OPENAI_BASE_URL (full URL with scheme, parsed to extract host + query params)
- **Ollama:** OLLAMA_HOST (localhost:11434 default)
- **Anthropic:** ANTHROPIC_HOST (default https://api.anthropic.com)
- **Azure:** AZURE_OPENAI_ENDPOINT (required, base URL)
- **Databricks:** DATABRICKS_WORKSPACE_URL
- **xAI:** XAI_HOST (default https://api.x.ai/v1)
- **LiteLLM:** LITELLM_HOST + LITELLM_BASE_PATH (allows custom proxy)

### Custom Endpoint Use Case (Self-Hosted)
OpenAI-compatible endpoints (openai_compatible.rs:96-120) support:
- Custom scheme + host + port (e.g., http://localhost:8000)
- Query parameters in OPENAI_BASE_URL preserved across all requests
- Model discovery from /v1/models endpoint

## Multi-Provider Coexistence
- **Active provider selection:** init.rs::create() accepts provider name + model name
- **No global singleton:** Each provider instance holds its own ApiClient (one ApiClient per provider)
- **Config isolation:** Secrets are provider-specific env vars (XAI_API_KEY vs ANTHROPIC_API_KEY)
- **Provider registry:** provider_registry.rs maintains a map of ProviderEntry (metadata + constructor + auth resolver)

## Per-Model Overrides
- **ModelConfig struct:** crates/model.rs carries model_name, temperature, max_tokens, context_limit, request_params
- **Request-time customization:** ModelConfigParams passed to format creators (goose_providers::formats::openai::ModelConfigParams)
- **Example:** anthropic.rs:68 applies fast model if model_config.with_fast() requested

---

# 3. LOCAL MODEL LANE (Ollama / llama.cpp)

## Ollama Discovery & Connection
- **Default endpoint:** localhost:11434 (OLLAMA_HOST env var overridable)
- **No auth:** Ollama expects local-only deployment (no API key)
- **Model listing:** GET /api/tags returns { models: [ { name, size, modified_at } ] }
- **Health check:** implicit (first /api/chat/completions request retries on 500)

## Model Loading & Inference
- **Lazy loading:** Ollama loads model on first request (can take 30-120s for large models)
- **Retry strategy:** OLLAMA_MAX_RETRIES=10, backoff 2-15s (ollama.rs:48-51)
  - Handles 500 errors during model load seamlessly
- **Streaming:** Enabled by default (OLLAMA_STREAM_USAGE=true)
  - Graceful fallback to non-streaming for older Ollama builds (lines 104-106)

## Token Usage for Local Models
- **With stream_options:** Ollama 2025+ returns usage in SSE data
  - fields: prompt_eval_count, eval_count (parsed by formats/openai.rs)
- **Without stream_options:** No usage data (falls back to estimation via usage_estimator.rs)
  - Fallback counts tokens client-side (token_counter.rs)
- **Graceful degradation:** Detects OLLAMA_STREAM_USAGE setting and adjusts request payload

## Context Window Configuration
- **Per-request:** ModelConfig.context_limit (if set)
- **Env override:** GOOSE_INPUT_LIMIT (ollama.rs:64) converts to options.num_ctx
- **Model default:** If no limit set, Ollama uses model's built-in default (~4K-32K typical)

## Local Model Metadata
- **Known models:** ollama.rs:36-42 has hardcoded list (qwen3, qwen3-coder, etc.)
- **Dynamic discovery:** /api/tags endpoint queried at runtime
- **Chat template:** Local provider supports multiple chat templates (local_inference.rs:22)
  - Affects message formatting for each model's conventions

## Streaming & Tool Calls
- **Streaming format:** SSE (Server-Sent Events) from /api/chat/completions
- **XML tool fallback:** Qwen3-coder outputs XML instead of JSON tools (formats/ollama.rs:35-79)
  - Graceful parsing: <function=name><parameter=key>value</parameter></function>
- **Token streaming:** If OLLAMA_STREAM_USAGE=true, usage appears in final SSE message

---

# 4. PORTABLE IDEAS FOR OTHER RUST AGENTS

## 1. Factory Pattern with Trait Registry
**Evidence:** provider_registry.rs, provider_registry.rs:28-37
- ProviderEntry: metadata + constructor fn + auth resolver + cleanup fn
- Supports dynamic provider discovery (no hardcoded if/else dispatch)
- **Reusable:** Generic over any trait implementing Provider + ProviderDef

## 2. Auth Abstraction Layer
**Evidence:** api_client.rs:28-38, azureauth.rs:31-49
- AuthMethod enum: NoAuth, BearerToken, ApiKey { header_name, key }, OAuth, Custom
- Custom auth via async trait (azureauth.rs:AuthProvider trait)
- **Portable:** Can inject any auth scheme by implementing AuthProvider async trait
- **Usage:** api_client.rs sends auth headers before each request

## 3. Config-First Secrets Management
**Evidence:** anthropic.rs:71-74, litellm.rs:41-59
- Pattern: `config.get_secret("ENV_VAR") -> env var first, config file fallback`
- Pattern: `config.get_param("WITH_DEFAULT") -> unwrap_or_else with sensible default`
- **Portable:** Decouples secrets from hardcoded values; supports multiple config sources

## 4. Model Metadata Canonical Lists
**Evidence:** openai.rs:42-67 (18 models + context windows), ollama.rs:36-42
- Statically declared ModelInfo with (name, context_window) tuples
- Known models mapped to context limits (eliminates guessing)
- **Portable:** Can auto-populate from API catalog fetch (openai.rs:96-120 fetch_supported_models)

## 5. Token Usage Estimation Fallback
**Evidence:** usage_estimator.rs, formats/ollama.rs:26-28
- If provider returns usage: use it
- If not: count tokens client-side (ensures always-valid ProviderUsage)
- **Pattern:** ensure_usage_tokens() checks if both input + output present before estimating
- **Portable:** Token counter abstraction (token_counter.rs) can be swapped

## 6. Retry Strategy with Exponential Backoff
**Evidence:** ollama.rs:48-51, bedrock.rs:40-43
- RetryConfig: max_retries, initial_interval_ms, backoff_multiplier, max_interval_ms
- Applied uniformly before Provider trait methods (retry.rs)
- **Portable:** Generic over any async operation (not provider-specific)
- **Tuning:** Ollama-specific tuning shows model-load delays handled gracefully

## 7. Streaming Response Parsing (SSE + JSON Lines)
**Evidence:** openai_compatible.rs:12-27, formats/ollama.rs
- Pattern: StreamReader + LinesCodec + parse each line as JSON
- Pattern: Handle partial chunks (async-stream, tokio-util)
- **Portable:** Works for any SSE endpoint (OpenAI, Ollama, Anthropic, etc.)

## 8. Custom Header Injection
**Evidence:** litellm.rs:70-78, api_client.rs:22
- ApiClient holds default_headers (HeaderMap)
- Added per-request via with_headers() method
- **Portable:** Supports arbitrary provider custom headers (X-Custom, etc.)

## 9. Endpoint Metadata Caching with TTL
**Evidence:** databricks.rs:61-64 (CachedDatabricksEndpointInfo, TTL 60s)
- LazyLock + Mutex: cache persists across requests
- Instant::now() + TTL check: auto-invalidation
- **Portable:** Generic cache trait can replace hardcoded Databricks logic

## 10. TLS Client Certificates
**Evidence:** api_client.rs:60-117 (TlsConfig, load_identity, load_ca_certificates)
- Env vars: GOOSE_CLIENT_CERT_PATH, GOOSE_CLIENT_KEY_PATH, GOOSE_CA_CERT_PATH
- Loaded on ApiClient init, passed to reqwest ClientBuilder
- **Portable:** Self-hosted endpoint auth via mTLS

## 11. Stream Options Conditional (Feature Detection)
**Evidence:** ollama.rs:77-95 (resolve_ollama_stream_usage default=true, graceful downgrade)
- Detects provider capability at init time (env var, version check, fallback)
- Removes unsupported fields from request payload before sending
- **Portable:** Prevents silent timeouts when server doesn't understand new API fields

## 12. Tool Call Format Fallbacks
**Evidence:** formats/ollama.rs:35-79 (XML -> JSON tool call parser)
- Some models output non-standard formats (XML instead of JSON)
- Isolate fallback logic per-provider (Ollama-specific in this case)
- **Pattern:** Parse_xml_tool_calls + Uuid generation for tool request IDs
- **Portable:** Extend to other non-standard formats (custom protocols, etc.)

## 13. Request Path / Completions Prefix
**Evidence:** openai_compatible.rs:34-35, azure.rs:deploy name in path
- OpenAiCompatibleProvider holds completions_prefix (e.g., "deployments/name/" for Azure)
- Concatenated with standard path in build_request
- **Portable:** Allows providers to customize request routing without forking impl

## 14. OAuth Device Flow (Installed App)
**Evidence:** gemini_oauth.rs:56-76, oauth_device_flow.rs
- Public client ID/secret (installed-app pattern)
- Browser authorization flow -> token stored in config cache
- **Pattern:** Local HTTP callback server (axum Router, lines 20-36)
- **Portable:** Generic OAuth flow for any provider supporting it

## 15. API Version Header Baking
**Evidence:** anthropic.rs:50 (ANTHROPIC_API_VERSION = "2023-06-01")
- Version specified as constant, passed to api_client.with_header() (line 82)
- Ensures backward compatibility if provider breaks API
- **Portable:** Externalize to env var for version pinning

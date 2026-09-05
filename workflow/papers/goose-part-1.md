Goose Provider Abstraction Layer: Core/Shared Infrastructure

This document maps the core provider abstraction layer of the Goose agent (Block's open-source Rust AI agent), focusing on the provider trait, factory/registry, message/usage handling, and streaming infrastructure. Concrete per-vendor implementations (anthropic.rs, openai.rs, etc) are excluded.

Source: F:\.ELAI_workflow\docs\.source\goose-main\crates\goose\src\providers + goose_providers crate

--- FILE MAP ---

base.rs (38K)
  Core provider abstractions and metadata. Key types:
  - ProviderDef trait (lines 396-450): Interface for provider definitions. Async factory method from_env, metadata declaration, inventory support flags.
  - Provider trait (line 460+): Main runtime trait with stream() async method signature and complete() wrapper. Delegates to stream() which yields MessageStream.
  - ModelInfo struct (lines 101-156): Model capabilities including context_limit, input_token_cost, output_token_cost (per-token in USD), supports_cache_control, reasoning flag. Factory methods: new(name, context_limit), with_cost(name, limit, input_cost, output_cost).
  - ProviderMetadata struct (lines 194-289): Provider registration metadata - display name, description, default_model, known_models[], config_keys[], setup_steps, model_selection_hint. Builder pattern with with_setup_steps(), with_model_selection_hint().
  - ConfigKey struct (lines 292-370): Config requirement metadata - name, required, secret, default, oauth_flow, device_code_flow, primary flag.
  - ProviderType enum (lines 185-191): Preferred / Builtin / Declarative / Custom.
  - MessageStream type alias (line 807-809): Pin<Box<dyn Stream<Item = Result<(Option<Message>, Option<ProviderUsage>), ProviderError>> + Send>>. Each stream item yields optional Message and optional ProviderUsage.
  - collect_stream() function (lines 817-866): Consumes MessageStream, coalesces consecutive text blocks, aggregates final Message and ProviderUsage. Returns (Message, ProviderUsage).
  - stream_from_single_message() helper (lines 811-814): Wraps single response into a MessageStream.

provider_registry.rs (14K)
  Factory and registration mechanism. Key types:
  - ProviderRegistry struct (lines 107-358): Central registry HashMap<String, ProviderEntry>. Methods: register::<F>(preferred: bool) generic over ProviderDef, register_with_name*() for declarative/custom providers, set_cleanup(), create(), all_metadata_with_types().
  - ProviderEntry struct (lines 29-104): Boxed provider instance. Stores metadata, Arc<constructor closure> (ModelConfig, Vec<ExtensionConfig>, Option<PathBuf>) -> BoxFuture<Result<Arc<dyn Provider>>>, inventory_identity resolver, inventory_configured checker, optional cleanup callback, provider_type, supports_inventory_refresh flag.
  - ProviderConstructor type alias (lines 11-19): Arc<dyn Fn(ModelConfig, Vec<ExtensionConfig>, Option<PathBuf>) -> BoxFuture<'static, Result<Arc<dyn Provider>>> + Send + Sync>.
  - ProviderCleanup type alias (line 21): Arc<dyn Fn() -> BoxFuture<'static, Result<()>> + Send + Sync>.
  - registry.normalize_model_config() (lines 60-75): Applies canonical limits from registry, fills context_limit from known_models if missing.

init.rs (17K)
  Global registry initialization and provider creation entrypoint. Key functions:
  - init_registry() async (lines 56-133): Static OnceCell initialization. Registers ~35 builtin providers (AnthropicProvider, OpenAiProvider, OllamaProvider, etc) with register::<Provider>(preferred), sets cleanup callbacks for stateful providers (GithubCopilot, Databricks, HuggingFace, etc), loads custom declarative providers.
  - get_registry() async (line 139-141): Lazy-initialized global registry singleton.
  - create(name, model, extensions) async (lines 177-184): Looks up provider in registry, calls entry.create(model, extensions).
  - create_with_working_dir(name, model, extensions, working_dir) async (lines 186-196): Variant for subprocess providers (ACP) that need session cwd.
  - create_with_default_model(name, extensions) async (lines 198-211): Uses provider's metadata.default_model.
  - providers() async (lines 143-149): List all registered providers with metadata and type.
  - inventory_identity(name) async (lines 173-175): Resolver for provider's required configuration identity.

retry.rs (9.5K)
  Retry and backoff mechanism for provider requests. Key types:
  - RetryConfig struct (lines 14-26): max_retries (default 3), initial_interval_ms (1000), backoff_multiplier (2.0), max_interval_ms (30000), transient_only flag.
  - retry_operation() async generic (lines 94-133): Retries operation F on ProviderError, uses exponential backoff with jitter, respects provider-supplied retry_delay from RateLimitExceeded, supports transient_only mode (skips RequestFailed 4xx).
  - ProviderRetry trait + blanket impl (lines 138-244): Async extension on Provider types. with_retry(), with_retry_config() methods. Includes separate auth-retry path (one attempt, calls refresh_credentials()).
  - should_retry() function (lines 84-92): Classifies errors - retries RateLimitExceeded, ServerError, NetworkError; RequestFailed only if not transient_only.
  - delay_for_attempt() (lines 65-81): Exponential backoff delay(ms) = initial * multiplier^(attempt-1), capped, + random jitter 0.8-1.2x.

api_client.rs (26K)
  HTTP client wrapper for provider requests. Key types:
  - ApiClient struct (lines 18-26): Wraps reqwest::Client with host, auth, default_headers, default_query, timeout, tls_config.
  - AuthMethod enum (lines 28-38): NoAuth, BearerToken(String), ApiKey{header_name, key}, OAuth(OAuthConfig), Custom(Box<dyn AuthProvider>).
  - TlsConfig struct (lines 46-117): Optional client_identity (cert+key pair paths), optional ca_cert_path. Methods: from_config() reads GOOSE_CLIENT_CERT_PATH, GOOSE_CLIENT_KEY_PATH, GOOSE_CA_CERT_PATH env vars, load_identity(), load_ca_certificates().
  - HTTP timeout default: DEFAULT_PROVIDER_TIMEOUT_SECS = 600 seconds (10 min, line 13).
  - Methods: post(), get(), response_post() for handling different endpoint shapes, handle_status() for HTTP status validation.

usage_estimator.rs (4.3K)
  Usage token fallback when providers don't report counts. Key function:
  - ensure_usage_tokens() async (lines 9-47): Given ProviderUsage with possibly incomplete token counts, estimates missing input/output using token_counter. If input_tokens missing, counts system + messages + tools. If output_tokens missing, counts response text. Calculates total_tokens = input + output.
  - Fallback tokenizer: create_token_counter() async, then count_chat_tokens(system, messages, tools) -> usize, count_tokens(text) -> usize.
  - Used after provider response collected, before returning to caller. Ensures every ProviderUsage has non-None token counts.

goose-providers crate / conversation/token_usage.rs (149 lines)
  Usage token structures (shared across all providers). Key types:
  - ProviderUsage struct (lines 6-24): model: String, usage: Usage. Methods: new(), combine_with(&other) aggregates token counts (takes model from self).
  - Usage struct (lines 27-107): input_tokens: Option<i32>, output_tokens: Option<i32>, total_tokens: Option<i32>, cache_read_input_tokens: Option<i32>, cache_write_input_tokens: Option<i32>. Implements Add + AddAssign for combining multi-turn usage.
    Constructor: Usage::new(input, output, total) auto-calculates total = input + output if total not provided.
    with_cache_tokens(cache_read, cache_write) builder.
  - Token aggregation: sum_optionals<T>() helper (lines 35-45) - adds Option values, treating None as T::default() (0 for i32).

goose-providers crate / formats/*.rs
  Per-vendor response format converters. Shared structures for token extraction:
  - openai_responses.rs (350+ lines): ResponsesApiResponse {id, status, model, output: Vec<ResponseOutputItem>, usage: Option<ResponseUsage>}. ResponseUsage {input_tokens: i32, output_tokens: i32, total_tokens: i32}. Conversion function get_responses_usage(&response) -> Usage.
  - Other format converters (anthropic, bedrock, ollama, google, etc) follow similar pattern: provider-specific response struct + usage extraction function.

--- TOKEN USAGE PIPELINE (STREAMING AND NON-STREAMING) ---

STREAMING FLOW:
  1. Provider.stream(model_config, session_id, system, messages, tools) -> Result<MessageStream>
     Called from: complete() (non-streaming wrapper) or directly (streaming caller)
     Returns: Pin<Box<dyn Stream>> that yields Result<(Option<Message>, Option<ProviderUsage>)>

  2. Provider implementation (e.g., openai.rs::stream()):
     a. Build request JSON from model_config, system, messages, tools
     b. POST to provider API with stream=true (or false for non-streaming)
     c. If streaming: stream_responses_compat() or stream_openai_compat() (from formats module)
        - Yields partial Message chunks (text deltas, tool-call fragments)
        - On first message_start event: extracts final usage info from stream metadata
        - Usage yielded ONCE at stream start (usage pushed in first Item) or at end
     d. If non-streaming: collect response JSON, parse response struct
        - Extract usage via get_responses_usage(response) or get_usage(json["usage"])
        - Convert to Message via responses_api_to_message() or response_to_message()
        - Wrap in stream_from_single_message(message, usage) -> MessageStream
        - This immediately yields one Item: (Some(message), Some(usage))

  3. Streaming caller consumes MessageStream:
     - Receives chunks: Some(Text("hello")), None (placeholder), Some(Text(" world")), Some(usage_delta)
     - OR single item: Some(full_message), Some(usage)

  4. collect_stream(stream) -> (Message, ProviderUsage):
     a. Iterate stream.next() until exhaustion
     b. On each Item (msg_opt, usage_opt):
        - If msg_opt: coalesce with prior final_message (merge consecutive Text blocks)
        - If usage_opt: store latest usage (last one wins, or aggregate if multi-turn)
     c. Return (final_message.unwrap(), final_usage.unwrap_or_default())
     d. Caller receives complete Message (concatenated text + all tool calls) + aggregated ProviderUsage

NON-STREAMING FLOW:
  1. Provider.complete(model_config, session_id, system, messages, tools) -> (Message, ProviderUsage)
     Implementation (lines 483-495): calls stream(), collects into (Message, ProviderUsage), returns

USAGE EXTRACTION DETAIL:
  ProviderUsage ALWAYS contains:
    - model: String (the actual model name returned by provider, or alias used)
    - usage: Usage (the token counts)
  
  Usage structure:
    - input_tokens: Option<i32> - tokens in system + messages + tools
    - output_tokens: Option<i32> - tokens in response
    - total_tokens: Option<i32> - sum, auto-calculated if both input/output present
    - cache_read_input_tokens: Option<i32> - tokens saved by reading cache (Anthropic)
    - cache_write_input_tokens: Option<i32> - tokens written to cache (Anthropic)

  Extraction per-vendor:
    - Anthropic: usage object in response contains input_tokens, output_tokens, cache_read_input_tokens, cache_write_input_tokens
    - OpenAI: usage object in response contains prompt_tokens, completion_tokens, total_tokens (mapped to input/output/total)
    - Others: follow OpenAI shape or custom format converter

  Fallback (usage_estimator.rs):
    - If provider doesn't report usage: ensure_usage_tokens() estimates via token_counter
    - token_counter created per request or cached (implementation detail)
    - count_chat_tokens(system, messages, tools) estimates input
    - count_tokens(response_text) estimates output
    - Both are tokenizer-based estimates (CLITokenCounter or similar, not a model call)

USAGE FLOW AFTER EXTRACTION:
  After collect_stream() returns (Message, ProviderUsage):
  1. ensure_usage_tokens() called in openai.rs complete() path (line ~880-890)
     - Fills any missing tokens via fallback estimator
     - Guarantees non-None input/output/total
  2. Message sent to caller/session
  3. ProviderUsage logged or aggregated:
     - Stored in session state for cost tracking
     - Can be combined with other turns via Usage::Add + ProviderUsage.combine_with()
     - UI displays cost = (input_tokens * input_token_cost) + (output_tokens * output_token_cost)
     - Cost currencies live in ModelInfo.currency (defaults to "$")
     - Actual costs per model live in ModelInfo.input_token_cost, output_token_cost (per-token in USD)

COST/PRICING COMPUTATION:
  Cost rates (per-token USD) stored in:
    - ModelInfo.input_token_cost: Option<f64>
    - ModelInfo.output_token_cost: Option<f64>
    - Per-model, fetched at provider.fetch_model_info(model_name) or from metadata.known_models

  Rate sources:
    - CanonicalModelRegistry (bundled in binary or loaded from YAML): provides default rates for standard models
    - Declarative provider config: can override rates per custom provider
    - Provider-specific hardcoded metadata (e.g., openai.rs::metadata())

  Rate updates:
    - Bundled registry: updated at compile time (committed to git)
    - Providers with dynamic models: implement fetch_supported_model_info() to query live rates
    - No live refresh mechanism in core layer (rates are static per session)

  Cost calculation (at caller/session level, not in providers):
    - cost_usd = (input_tokens * input_token_cost) + (output_tokens * output_token_cost)
    - Handle cache tokens separately if present (usually discounted rates)
    - UI layer queries ModelInfo from provider.fetch_model_info() to get rates

MISSING USAGE HANDLING:
  When provider returns no usage:
    - ensure_usage_tokens() detects None in input/output_tokens
    - Falls back to estimate via token_counter
    - Tokenizer: signature create_token_counter() -> TokenCounter, count_chat_tokens(sys, msgs, tools) -> usize
    - Estimates are conservative (not exact) but prevent downstream crashes
    - Fallback is REQUIRED before returning to caller (ensure_usage_tokens() enforces)

CACHING:
  Anthropic cache tokens (cache_read_input_tokens, cache_write_input_tokens):
    - Extracted from ProviderUsage.usage
    - Added to Usage struct at lines 31-32 (goose-providers)
    - Aggregated via sum_optionals when combining multi-turn (lines 56-62)
    - UI displays cache savings if present

--- PROVIDER ABSTRACTION ---

TRAIT SURFACE (Provider, base.rs lines 460+):

  async fn stream(
    &self,
    model_config: &ModelConfig,
    session_id: &str,
    system: &str,
    messages: &[Message],
    tools: &[Tool],
  ) -> Result<MessageStream, ProviderError>
    Primary required method. All providers must implement. Called by complete() and direct streaming callers.

  async fn complete(
    &self,
    model_config: &ModelConfig,
    session_id: &str,
    system: &str,
    messages: &[Message],
    tools: &[Tool],
  ) -> Result<(Message, ProviderUsage), ProviderError>
    Default implementation: calls stream(), collects into single Message + usage, returns.
    Can override for non-streaming optimizations.

  async fn complete_fast(
    &self,
    session_id: &str,
    system: &str,
    messages: &[Message],
    tools: &[Tool],
  ) -> Result<(Message, ProviderUsage), ProviderError>
    Fallback helper: tries fast_model first, retries with regular model on error.
    Signature omits model_config (uses get_model_config()).

  fn get_model_config(&self) -> ModelConfig
    Returns current ModelConfig (model name, aliases, context limit, etc).

  fn retry_config(&self) -> RetryConfig
    Default: RetryConfig::default() (3 retries, 1s initial, 2x backoff, 30s max).
    Override to customize per-provider.

  async fn fetch_supported_models(&self) -> Result<Vec<String>, ProviderError>
    Default: Ok(vec![]) (no inventory support).
    Providers with dynamic model lists override (e.g., Ollama, local inference).

  async fn fetch_supported_model_info(&self) -> Result<Vec<ModelInfo>, ProviderError>
    Default: calls fetch_supported_models(), wraps each in model_info_for_provider_model().
    Override for models with custom metadata.

  async fn fetch_model_info(&self, model_name: &str) -> Result<ModelInfo, ProviderError>
    Get info for single model. Default: calls model_info_for_provider_model().

  fn skip_canonical_filtering(&self) -> bool
    Default: false. If true, fetch_recommended_models() returns all models, not filtered by canonical registry.

  async fn fetch_recommended_models(&self) -> Result<Vec<String>, ProviderError>
    Filters supported_models by canonical registry usability (release dates, experimental flags).
    Used by UI model picker.

  async fn refresh_credentials(&self) -> Result<()>
    Auth credential refresh (called on Authentication error before retrying).
    Default: stub (not implemented).
    Overridden by OAuth/token-based providers.

FACTORY/REGISTRY (ProviderDef trait, base.rs lines 396-450):

  type Provider: Provider + 'static
    Associated type, the concrete Provider implementation.

  fn metadata() -> ProviderMetadata
    Returns static metadata: name, display_name, description, default_model, known_models[], config_keys[], setup_steps.

  async fn from_env(
    model: ModelConfig,
    extensions: Vec<ExtensionConfig>,
  ) -> Result<Self::Provider>
    Factory async function. Reads env vars (API keys, endpoints, etc), constructs Provider instance.
    Registered via provider_registry::register::<ProviderType>().

  async fn from_env_with_working_dir(
    model: ModelConfig,
    extensions: Vec<ExtensionConfig>,
    working_dir: PathBuf,
  ) -> Result<Self::Provider>
    Variant for subprocess providers (ACP implementations) that need session cwd.
    Default: delegates to from_env().

  fn supports_inventory_refresh() -> bool
    Default: false. If true, provider supports runtime model list updates (e.g., Ollama can pick up new models).

  fn inventory_identity() -> Result<InventoryIdentityInput>
    Returns the config keys that identify this provider uniquely (used for cache invalidation).
    Default: default_inventory_identity(&metadata.name, &config_keys).

  fn inventory_configured() -> bool
    Returns true if required config keys are set in Config::global().
    Default: default_inventory_configured(&config_keys).

CAPABILITY/METADATA DECLARATION:

  ProviderMetadata struct (base.rs lines 194-289):
    name: String - unique internal ID (e.g., "openai", "anthropic")
    display_name: String - UI name (e.g., "OpenAI")
    description: String - feature summary
    default_model: String - recommended model for this provider
    known_models: Vec<ModelInfo> - list of available models with capabilities
    model_doc_link: String - URL to provider's model documentation
    config_keys: Vec<ConfigKey> - required/optional config parameters
    setup_steps: Vec<String> - step-by-step setup instructions (shown in UI)
    model_selection_hint: Option<String> - hint for model picker (e.g., "Managed by provider")

  ModelInfo struct (base.rs lines 101-156):
    name: String - model identifier (e.g., "gpt-4-turbo")
    resolved_model: Option<String> - canonical name if aliased
    context_limit: usize - max context window (tokens)
    input_token_cost: Option<f64> - USD per input token
    output_token_cost: Option<f64> - USD per output token
    currency: Option<String> - currency symbol (default "$")
    supports_cache_control: Option<bool> - Anthropic-style cache support
    reasoning: bool - supports reasoning/thinking mode (o1/claude-opus)

  ConfigKey struct (base.rs lines 292-370):
    name: String - env var name or config key (e.g., "OPENAI_API_KEY")
    required: bool
    secret: bool - should be stored in keychain/secrets manager
    default: Option<String> - default value
    oauth_flow: bool - if true, provider.configure_oauth() called instead of manual key entry
    device_code_flow: bool - RFC 8628 device flow (user enters code in browser)
    primary: bool - shown prominently in UI setup

CONFIG/AUTH SHAPE:

  ApiClient (api_client.rs lines 18-26):
    Wraps HTTP requests with auth header injection, query params, timeout, TLS.

  AuthMethod enum (api_client.rs lines 28-38):
    - NoAuth
    - BearerToken(String) - "Authorization: Bearer {token}"
    - ApiKey{header_name, key} - custom header (e.g., "X-API-Key: {key}")
    - OAuth(OAuthConfig) - OAuth2 flow details
    - Custom(Box<dyn AuthProvider>) - custom auth logic (impl AuthProvider)

  TlsConfig (api_client.rs lines 46-117):
    - client_identity: Option<TlsCertKeyPair> - mTLS cert+key
    - ca_cert_path: Option<PathBuf> - custom CA cert
    - from_config() reads GOOSE_CLIENT_CERT_PATH, GOOSE_CLIENT_KEY_PATH, GOOSE_CA_CERT_PATH

  Timeout: DEFAULT_PROVIDER_TIMEOUT_SECS = 600s (10 min, per-request override possible)

--- PORTABLE IDEAS (REUSABLE IN OTHER RUST AGENTS) ---

1. STREAMING ABSTRACTION (MessageStream type alias, base.rs:807-809)
   Evidence: base.rs lines 807-809, openai.rs:810, collect_stream usage throughout
   Idea: Pin<Box<dyn Stream<Item = Result<(Option<Message>, Option<Usage>), Error>>>>
   Benefit: Decouples streaming transport (HTTP/SSE, gRPC, file) from provider logic. Stream items carry optional message + optional usage, allowing progressive delivery + out-of-order usage metadata.
   Reuse: Any Rust agent needing to coalesce streaming responses into a single result should adopt this pattern. Avoids re-building the entire response in memory; allows incremental processing.

2. RETRY WITH EXPONENTIAL BACKOFF + JITTER + SELECTIVE ERROR HANDLING (retry.rs:84-132)
   Evidence: retry.rs lines 84-132, should_retry(), delay_for_attempt()
   Idea: Classify errors (transient vs permanent), support provider-supplied retry_delay, separate auth-retry path (1 attempt), use jitter (0.8-1.2x) to avoid thundering herd.
   Benefit: Generic, reusable, handles all common error classes. Jitter is cheap and effective. Auth retry decoupled so credentials can refresh mid-request without blowing backoff budget.
   Reuse: Extract retry.rs almost as-is. Adapt should_retry() for your error enum. The delay_for_attempt() math is stable and well-tested.

3. PROVIDER REGISTRY AS TYPESAFE FACTORY (provider_registry.rs:107-358)
   Evidence: provider_registry.rs lines 107-358, register::<F>() generic, ProviderConstructor Arc closure
   Idea: Store Arc<dyn Fn(config) -> BoxFuture<Result<Arc<dyn Provider>>>> in registry. On lookup, call constructor, await inside with_retry context. Supports declarative config overlay (register_with_name_and_inventory_configured).
   Benefit: Decouples provider selection (runtime config) from instantiation (early init). Allows custom/declarative providers without code changes. Lazy initialization via OnceCell.
   Reuse: Generic pattern. Replace ProviderDef with your trait, ProviderEntry with your entry struct. Supports both static registration (register::<builtin>()) and dynamic (register_with_name_impl()).

4. USAGE TOKEN AGGREGATION WITH OPTIONAL FIELDS (token_usage.rs:26-107)
   Evidence: token_usage.rs lines 26-107, sum_optionals(), Usage::Add impl
   Idea: All token counts are Option<i32>. Implement Add operator: None + Some(x) = Some(x + 0), Some(x) + Some(y) = Some(x + y). Separate cache tokens (read/write) from main pipeline.
   Benefit: Graceful handling of partial usage data. Providers that don't report output_tokens don't break downstream. Cache tokens don't need presence in every response.
   Reuse: Copy Usage struct + sum_optionals() helper. Extend with new optional fields as needed (e.g., context_cache_input_tokens). Add operator makes multi-turn aggregation trivial.

5. USAGE FALLBACK ESTIMATOR (usage_estimator.rs:9-47)
   Evidence: usage_estimator.rs lines 9-47, ensure_usage_tokens()
   Idea: After provider response, detect missing token counts. Fall back to tokenizer (create_token_counter() -> count_chat_tokens / count_tokens). Guarantee non-None before returning to caller.
   Benefit: Prevents crashes when provider doesn't report usage. Estimates are conservative but sufficient for UI display. Single point of fallback logic.
   Reuse: Adapt to your tokenizer interface (swap create_token_counter for your token counter API). Signature: ensure_usage_tokens(&mut ProviderUsage, system, request_messages, response, tools) -> Result.

6. METADATA-DRIVEN CONFIG KEY DECLARATION (base.rs:292-370)
   Evidence: base.rs lines 292-370 ConfigKey struct, provider_registry.rs:237-262 config_keys assembly
   Idea: Separate ConfigKey metadata (name, required, secret, oauth_flow, device_code_flow, primary) from actual config reading. UI uses metadata to generate UI forms. Provider reads env/keychain at instantiation.
   Benefit: Centralized, versioned config schema. Supports dynamic provider overlays (register_with_name_impl updates config_keys without touching provider code). Primary flag lets UI hide advanced keys.
   Reuse: Copy ConfigKey struct verbatim. Extend with new metadata fields (e.g., pattern for validation, examples). Update ProviderMetadata builder to expose config_keys[] to UI.

7. CONVERSATION/MESSAGE FORMAT CONVERTERS (formats/*.rs)
   Evidence: formats/openai_responses.rs, anthropic.rs, etc - per-vendor response parsing + usage extraction
   Idea: Each provider format (OpenAI responses, Anthropic, Bedrock) gets dedicated module with structs (ResponsesApiResponse, ResponseUsage, etc) + converter functions (responses_api_to_message, get_responses_usage).
   Benefit: Isolates format differences. Makes it easy to add new providers (copy template, update struct serde, add conversion fn). Test each format independently.
   Reuse: Pattern for new agent: for each provider, create formats/vendor.rs with response struct + converter. Keep message format (goose/crates/goose-providers/conversation/message.rs) shared and stable. Converters are pure functions (no I/O).

8. LAZY GLOBAL REGISTRY WITH TOKIO ONCECELL (init.rs:54-141)
   Evidence: init.rs lines 54-141, static REGISTRY: OnceCell, init_registry() async, get_registry() returns &'static
   Idea: Use tokio::sync::OnceCell for async-safe lazy initialization. init_registry() runs once, registers all builtin + custom providers, sets cleanup callbacks. Subsequent get_registry() calls return cached reference.
   Benefit: No static mut, no race conditions, async-friendly (can call async functions in init). Cleanup callbacks allow stateful providers (auth tokens, cached models) to clean up on shutdown.
   Reuse: Copy init.rs pattern. Register your provider types in init_registry(), call set_cleanup() for stateful ones. Adapt create() / create_with_default_model() signatures to your API.

9. DECLARATIVE PROVIDER CONFIG OVERLAY (provider_registry.rs:153-317)
   Evidence: provider_registry.rs lines 153-317, register_with_name_and_inventory_configured
   Idea: Allow YAML/JSON declarative config to create custom providers at runtime without code changes. register_with_name() wraps constructor closure, inventory_identity resolver. Supports custom env_vars beyond the base provider's config_keys.
   Benefit: Users can add OpenAI-compatible, Ollama-compatible, or custom providers via config file. Reduces maintenance burden (new provider = new row in config, not new Rust code).
   Reuse: Adapt register_with_name_impl() to accept your DeclarativeProviderConfig shape. Ensure constructor closure can read env vars and instantiate the right provider type (e.g., OpenAiProvider with custom base_url).

10. MODEL INFO CANONICAL REGISTRY MAPPING (base.rs:158-183, canonical module)
    Evidence: base.rs lines 158-183 model_info_for_provider_model, CanonicalModelRegistry, map_to_canonical_model
    Idea: Map provider-specific model names (e.g., "gpt-4-turbo-2024-04-09") to canonical IDs ("openai/gpt-4-turbo"). Load a bundled YAML registry at startup. Fetch context limits, default token costs, reasoning flags from canonical registry.
    Benefit: Single source of truth for model capabilities across providers. Avoids duplicating metadata in each provider. Auto-update mechanism: update bundled registry, rebuild, ship. No live API call needed.
    Reuse: Create a canonical_models.yaml (or embedded Rust const) mapping provider/model -> context_limit, input_cost, output_cost, reasoning. Load at startup. Call map_to_canonical_model(provider_name, model_name, registry) to resolve aliases.


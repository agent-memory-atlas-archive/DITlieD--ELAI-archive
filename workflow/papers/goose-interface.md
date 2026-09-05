Source: goose (block/goose OSS, fork aaif-goose/goose)
Reliability: MEDIUM-HIGH (reviewed official OSS source, not decompiled)
Files examined:
  crates/goose/src/providers/base.rs
  crates/goose/src/providers/mod.rs
  crates/goose/src/providers/init.rs
  crates/goose/src/providers/anthropic.rs
  crates/goose/src/providers/claude_code.rs
  crates/goose/src/providers/claude_acp.rs
  crates/goose/src/providers/codex.rs
  crates/goose/src/providers/codex_acp.rs
  crates/goose/src/providers/chatgpt_codex.rs
  crates/goose/src/providers/cursor_agent.rs
  crates/goose/src/providers/copilot_acp.rs
  crates/goose/src/providers/gemini_cli.rs
  crates/goose/src/providers/gemini_oauth.rs
  crates/goose/src/providers/xai_oauth.rs
  crates/goose/src/providers/huggingface_auth.rs
  crates/goose/src/providers/api_client.rs
  crates/goose/src/providers/oauth.rs (databricks OAuth)
  crates/goose/src/providers/gcpauth.rs
  crates/goose/src/providers/azureauth.rs
  crates/goose/src/config/providers.rs
  crates/goose/src/config/declarative_providers.rs
  crates/goose-server/src/auth.rs
  crates/goose-server/src/routes/config_management.rs
  crates/goose-server/src/routes/setup.rs
  ui/desktop/package.json


Provider Trait (crates/goose/src/providers/base.rs)

Public API:

trait Provider: Send + Sync
  fn get_name(&self) -> &str
  async fn stream(&self, model_config: &ModelConfig, session_id: &str, system: &str, messages: &[Message], tools: &[Tool]) -> Result<MessageStream, ProviderError>
  async fn complete(&self, model_config, session_id, system, messages, tools) -> Result<(Message, ProviderUsage), ProviderError>
  async fn complete_fast(&self, session_id, system, messages, tools) -> Result<(Message, ProviderUsage), ProviderError>
  fn get_model_config(&self) -> ModelConfig
  fn retry_config(&self) -> RetryConfig [default]
  async fn fetch_supported_models(&self) -> Result<Vec<String>, ProviderError> [default empty]
  async fn fetch_recommended_models(&self) -> Result<Vec<String>, ProviderError> [filters by canonical registry]
  async fn configure_oauth(&self) -> Result<(), ProviderError> [default returns error]
  async fn refresh_credentials(&self) -> Result<(), ProviderError> [default returns NotImplemented]
  fn manages_own_context(&self) -> bool [default false; CLI wrappers return true]
  fn skip_canonical_filtering(&self) -> bool [default false]
  async fn supports_cache_control(&self) -> bool [default false]
  fn permission_routing(&self) -> PermissionRouting [Noop or ActionRequired]
  async fn handle_permission_confirmation(&self, request_id, confirmation) -> bool

trait ProviderDef: Send + Sync
  type Provider: Provider + 'static
  fn metadata() -> ProviderMetadata [static, required]
  fn from_env(model: ModelConfig, extensions: Vec<ExtensionConfig>) -> BoxFuture<'static, Result<Self::Provider>>
  fn from_env_with_working_dir(model, extensions, working_dir: PathBuf) -> BoxFuture [ACP subprocess providers must override]
  fn supports_inventory_refresh() -> bool [default false]
  fn inventory_identity() -> Result<InventoryIdentityInput> [default uses config_keys + Config::global()]
  fn inventory_configured() -> bool

Key types:
  ProviderMetadata { name, display_name, description, default_model, known_models: Vec<ModelInfo>, model_doc_link, config_keys: Vec<ConfigKey>, setup_steps: Vec<String>, model_selection_hint: Option<String> }
  ConfigKey { name, required, secret, default: Option<String>, oauth_flow: bool, device_code_flow: bool, primary: bool }
  ModelInfo { name, resolved_model: Option<String>, context_limit: usize, input_token_cost: Option<f64>, output_token_cost: Option<f64>, currency: Option<String>, supports_cache_control: Option<bool>, reasoning: bool }
  ProviderType: Preferred | Builtin | Declarative | Custom
  PermissionRouting: ActionRequired | Noop
  MessageStream = Pin<Box<dyn Stream<Item = Result<(Option<Message>, Option<ProviderUsage>), ProviderError>> + Send>>
  AuthMethod: NoAuth | BearerToken(String) | ApiKey { header_name, key } | OAuth(OAuthConfig) | Custom(Box<dyn AuthProvider>)
  CURRENT_MODEL: Lazy<Mutex<Option<String>>> [global, set when provider returns actual model name]
  DEFAULT_PROVIDER_TIMEOUT_SECS: u64 = 600


Provider Registry and Construction (crates/goose/src/providers/init.rs)

pub async fn providers() -> Vec<(ProviderMetadata, ProviderType)>
pub async fn create(name: &str, model: ModelConfig, extensions: Vec<ExtensionConfig>) -> Result<Arc<dyn Provider>>
pub async fn create_with_working_dir(name, model, extensions, working_dir: PathBuf) -> Result<Arc<dyn Provider>>
pub async fn create_with_default_model(name: impl AsRef<str>, extensions) -> Result<Arc<dyn Provider>>
pub async fn create_with_named_model(provider_name, model_name, extensions) -> Result<Arc<dyn Provider>>
pub async fn cleanup_provider(name: &str) -> Result<()>
pub async fn get_from_registry(name: &str) -> Result<ProviderEntry>
pub async fn inventory_identity(name: &str) -> Result<InventoryIdentityInput>
pub async fn refresh_custom_providers() -> Result<()>

Registry is a global OnceCell<RwLock<ProviderRegistry>> initialized lazily.
Providers registered as preferred (true) = shown in onboarding UI:
  anthropic, chatgpt_codex, claude_code, codex, databricks, gemini_oauth, google,
  huggingface, kimi_code, nanogpt, ollama, openai, openrouter, tetrate, xai_oauth
Non-preferred (false) = available but not surfaced in onboarding:
  amp_acp, avian, azure, claude_acp, codex_acp, copilot_acp, cursor_agent,
  databricks_v2, gcpvertexai, gemini_cli, github_copilot, litellm, pi_acp,
  snowflake, xai (API-key variant), bedrock/sagemaker_tgi (feature-gated)

Cleanup functions registered for providers that hold cached OAuth state:
  github_copilot, databricks, databricks_v2, kimi_code, chatgpt_codex,
  gemini_oauth, xai_oauth, huggingface


Auth Mechanism Taxonomy

AUTH-1: API Key via Config (most common)
  Providers: anthropic, openai, google, xai, openrouter, databricks, huggingface (fallback), kimi_code, nanogpt, groq, mistral, deepseek, nvidia, nearai, alibaba, tanzu_ai
  Mechanism: config.get_secret("ANTHROPIC_API_KEY") -> ApiClient with AuthMethod::ApiKey { header_name: "x-api-key", key }
  Config key: ConfigKey::new("ANTHROPIC_API_KEY", required, secret=true, default=None, primary=true)
  Storage: goose config file (~/.config/goose/config.yaml secrets section) via Config::global()
  Env override: env var with same name (e.g. ANTHROPIC_API_KEY) takes precedence

AUTH-2: CLI-Subprocess / Subscription Reuse (no API key required)
  Providers: claude_code (DEPRECATED), claude_acp, codex, chatgpt_codex (has own OAuth), cursor_agent, gemini_cli, copilot_acp
  Mechanism: spawns external CLI binary as a persistent or per-turn child process; auth is owned by that CLI's own session (claude, codex, cursor-agent, gemini, copilot)
  Config key: ConfigKey::from_value_type::<ClaudeCodeCommand>(required=true, secret=false, primary=true) -- stores path to the CLI binary, not a credential
  Construction: from_env() resolves binary via SearchPaths::builder().with_npm().resolve(command), spawns as tokio::process::Command
  Gotcha: claude_code (stream-json mode) is deprecated; claude_acp replaces it via the ACP protocol adapter npm package

AUTH-3: PKCE Browser OAuth (loopback redirect)
  Providers: chatgpt_codex, gemini_oauth, xai_oauth (standard flow), huggingface, databricks
  Mechanism: spawns local axum HTTP server on a fixed loopback port (e.g. port 1455 for chatgpt_codex, 17863 for huggingface, 56121 for xai_oauth); opens browser to provider's authorization URL with PKCE; callback sets token via oneshot channel; token cached on disk
  Cache paths: ~/.config/goose/huggingface/oauth/tokens.json, ~/.config/goose/gemini_oauth/tokens.json, ~/.config/goose/databricks/oauth/{hash}.json
  Timeout: 300s for user to complete browser flow
  ConfigKey type: ConfigKey::new_oauth(name, required, secret, default, primary) -> oauth_flow=true, device_code_flow=false
  configure_oauth() method on Provider trait is called by config system when oauth_flow=true on a key

AUTH-4: OAuth Device Code Flow (RFC 8628)
  Providers: xai_oauth (also supports device code path), potentially others
  Mechanism: POST to device_authorization_endpoint -> get device_code + verification_uri -> display code to user -> poll token endpoint until user completes in browser
  ConfigKey type: ConfigKey::new_oauth_device_code(name, required, secret, default, primary) -> device_code_flow=true
  xAI constants: DEVICE_AUTHORIZATION_URL = "https://auth.x.ai/oauth2/device/code", CLIENT_ID = "b1a00492-073a-47ea-816f-4c329264a828" (Grok-CLI public client)
  Polling: default 5s interval, slow_down increments by 5s, timeout 5 minutes

AUTH-5: GCP Application Default Credentials
  Providers: gcpvertexai (via gcpauth.rs)
  Mechanism: reads ~/.config/gcloud/application_default_credentials.json; two sub-types: AuthorizedUser (refresh token via gcloud auth) or ServiceAccount (JWT signed with private key); token cached in-process with expiry
  Struct: AzureCredentials::DefaultCredential | ApiKey(String)

AUTH-6: Azure DefaultCredential Chain
  Providers: azure (via azureauth.rs)
  Mechanism: runs az account get-access-token --resource ... via SubprocessExt; falls back to AZURE_API_KEY env var (AzureCredentials::ApiKey)
  Caches token in Arc<RwLock<Option<CachedToken>>> with expiry

AUTH-7: Declarative Provider (JSON config files)
  Providers: all providers in crates/goose/src/providers/declarative/*.json + ~/.config/goose/custom_providers/*.json
  Mechanism: DeclarativeProviderConfig { engine: OpenAI | Anthropic | Ollama, base_url, api_key_env, models, env_vars, requires_auth, dynamic_models }
  Auth: reads api_key_env from Config::global(); if requires_auth=false, no key needed
  Construction: register_declarative_providers() loads at registry init; refresh_custom_providers() reloads on demand
  Custom provider JSON format: { name, engine, display_name, description, api_key_env, base_url, models: [{name, context_limit}], requires_auth }

AUTH-8: HuggingFace Token (dual: OAuth or manual HF_TOKEN)
  Provider: huggingface
  Primary path: OAuth flow (CIMD-based, port 17863, scopes: read-repos gated-repos inference-api); token stored at ~/.config/goose/huggingface/oauth/tokens.json
  Fallback: manual HF_TOKEN secret in config
  ConfigKey: oauth_flow=true; configure_oauth() override in HuggingFaceProvider


Config System (crates/goose/src/config/providers.rs)

Active provider resolution order:
  1. GOOSE_PROVIDER env var
  2. active_provider key in ~/.config/goose/config.yaml
  3. Legacy flat GOOSE_PROVIDER key in config.yaml

Active model resolution order:
  1. GOOSE_MODEL env var
  2. providers.{active_provider}.model in config.yaml
  3. Legacy flat GOOSE_MODEL key

pub fn get_active_provider(config: &Config) -> Option<String>
pub fn get_active_model(config: &Config) -> Option<String>
pub fn set_active_provider(config: &Config, name: &str, model: &str) -> Result<(), ConfigError>
  writes: active_provider key + providers.{name} = { enabled: true, model, configured: true }
pub fn get_provider_entry(config: &Config, name: &str) -> Option<ProviderEntry>
pub fn set_provider_entry(config: &Config, name: &str, entry: &ProviderEntry) -> Result<(), ConfigError>

ProviderEntry { enabled: bool, model: String, configured: bool }

Config file: ~/.config/goose/config.yaml (or $GOOSE_PATH_ROOT/config/config.yaml)
Secrets stored separately (secrets file, path from Config::new_with_file_secrets)
Config::global() returns a global static Config instance


Declarative Provider Config (crates/goose/src/config/declarative_providers.rs)

DeclarativeProviderConfig fields (JSON schema for provider files):
  name: String
  engine: "openai" | "anthropic" | "ollama"
  display_name: String
  description: Option<String>
  api_key_env: String (env var name for API key, empty if no auth)
  base_url: String
  models: Vec<ModelInfo> (static model list)
  headers: Option<HashMap<String,String>> (extra request headers)
  timeout_seconds: Option<u64>
  supports_streaming: Option<bool>
  requires_auth: bool [default true]
  dynamic_models: Option<bool> (if Some(false) + non-empty models: static list, no API call)
  env_vars: Option<Vec<EnvVarConfig>> (additional env vars beyond api_key_env)
  catalog_provider_id: Option<String>
  base_path: Option<String>
  setup_steps: Vec<String>
  fast_model: Option<String>
  preserves_thinking: bool [default false for Anthropic, true for OpenAI]
  skip_canonical_filtering: bool
  model_doc_link: Option<String>

Fixed providers: embedded via include_dir! at compile time from crates/goose/src/providers/declarative/
Custom providers: ~/.config/goose/custom_providers/*.json (user-authored, reloaded via refresh_custom_providers)


GooseServer Auth (crates/goose-server/src/auth.rs)

Middleware: check_token(State(secret_key: String), request, next)
  Header: X-Secret-Key: <token>
  Exempted paths: /status, /features, /mcp-ui-proxy, /mcp-app-proxy, /mcp-app-guest
  Returns 401 UNAUTHORIZED on mismatch

Server-side OAuth setup routes (crates/goose-server/src/routes/setup.rs):
  POST /handle_openrouter  -> OpenRouterAuth::new().complete_flow() -> configure_openrouter(config, api_key)
  POST /handle_tetrate     -> TetrateAuth::new().complete_flow() -> configure_tetrate(config, api_key)
  POST /handle_nanogpt     -> configure_nanogpt(config, ...)

Config management routes (crates/goose-server/src/routes/config_management.rs):
  POST /config/upsert  { key, value, is_secret } -> upsert_config()
    intercepted legacy keys: GOOSE_PROVIDER -> set_active_provider(); GOOSE_MODEL -> set_active_provider()
  GET /config
  DELETE /config  { key, is_secret }
  GET /providers  -> ProvidersResponse { providers: Vec<ProviderDetails> }
  POST /providers/check { provider: String }
  POST /providers/set { provider: String, model: String }
  GET/POST /provider_secrets -> ProviderSecretsResponse { secrets: Vec<ProviderSecret> }

ProviderSecret { id, provider, provider_display_name, name, storage: SecretStore|ProviderCache,
  expires_at: Option<DateTime<Utc>>, status: Valid|Expired|Unknown, configured, has_secret,
  can_delete, can_configure, configure_provider: Option<String> }


Desktop App (ui/desktop)

Framework: React 19 + Electron (electron-forge ^7.11.1)
Language: TypeScript
Key deps: @aaif/goose-sdk, @agentclientprotocol/sdk, @radix-ui/* (component library), @mcp-ui/client
Server communication: HTTP to goose-server (localhost) via X-Secret-Key header
Provider configuration exposed via goose-server REST endpoints (not direct config file writes from renderer)


Specific Provider Details

claude_code (DEPRECATED, use claude_acp):
  Name: "claude-code"
  Auth: AUTH-2 (CLI subprocess, binary path in config as ClaudeCodeCommand)
  Config keys: [ClaudeCodeCommand path, required=true, secret=false, primary=true]
  Mechanism: persistent claude process via --input-format stream-json --output-format stream-json
  manages_own_context() = true; permission_routing() = ActionRequired
  Protocol: NDJSON on stdin/stdout; control_request/control_response for permission prompts
  Deprecation note: metadata description says "Deprecated: use claude-acp instead"

claude_acp:
  Name: "claude-acp"
  Auth: AUTH-2 (delegates to Claude CLI's own session; npm -g @agentclientprotocol/claude-agent-acp)
  Config keys: [] (no goose-owned credentials)
  Setup steps: install npm pkg, verify claude CLI auth, add to goose config
  Binary: "claude-agent-acp" resolved via SearchPaths::builder().with_npm()
  Mode mapping: Auto->bypassPermissions, Approve->default, SmartApprove->acceptEdits, Chat->plan

codex:
  Name: "codex"
  Auth: AUTH-2 (codex CLI subscription reuse, binary in config as CodexCommand)
  Config keys: [CodexCommand path, CodexSkipGitCheck, CodexReasoningEffort]
  Mechanism: per-turn one-shot spawn of codex exec -; JSONL on stdout
  Default model: "gpt-5.2-codex"
  GooseMode flags: Auto->--yolo, SmartApprove->--full-auto, Chat->--sandbox read-only

chatgpt_codex:
  Name: "chatgpt_codex"
  Auth: AUTH-3 (PKCE browser OAuth, loopback port 1455, PKCE S256)
  Auth endpoint: https://auth.openai.com; token endpoint: https://chatgpt.com/backend-api/codex
  Config keys: [oauth token key, oauth_flow=true, device_code_flow=false]
  cleanup() clears cached token; configure_oauth() triggers browser flow
  Default model: "gpt-5.5"

cursor_agent:
  Name: "cursor-agent"
  Auth: AUTH-2 (Cursor subscription; binary in config as CursorAgentCommand)
  Auth check: cursor-agent status -> "✓ Logged in as" in stdout
  Config keys: [CursorAgentCommand path]
  Default model: "auto"

copilot_acp:
  Name: "copilot-acp"
  Auth: AUTH-2 (copilot login, binary "copilot", ACP protocol)
  Config keys: [] (no goose credentials)
  Setup: npm install -g @github/copilot then copilot login

gemini_cli:
  Name: "gemini-cli"
  Auth: AUTH-2 (Gemini CLI subscription; binary in config as GeminiCliCommand)
  Config keys: [GeminiCliCommand path]
  Default model: "gemini-2.5-pro"

gemini_oauth:
  Name: "gemini_oauth"
  Auth: AUTH-3 (PKCE browser OAuth using Gemini CLI public client ID/secret)
  Endpoint: Google OAuth 2.0 (accounts.google.com), Code Assist API (cloudcode-pa.googleapis.com)
  Scopes: cloud-platform, userinfo.email
  Token cache: ~/.config/goose/gemini_oauth/tokens.json
  configure_oauth() override: full PKCE flow with loopback server
  Default model: "gemini-3-flash-preview"
  Note: client_id/client_secret are assembled from string parts at runtime to bypass GitHub secret scanners; they are the publicly-known Gemini CLI installed-app credentials

xai_oauth:
  Name: "xai_oauth"
  Auth: AUTH-3 + AUTH-4 (both browser PKCE and device code flow; reuses Grok-CLI client_id)
  client_id: "b1a00492-073a-47ea-816f-4c329264a828" (Grok-CLI public)
  loopback port: 56121 (must match xAI's registered redirect)
  Device code endpoint: https://auth.x.ai/oauth2/device/code
  Token refresh skew: 120s before stored expiry
  Config key: device_code_flow=true

anthropic:
  Name: "anthropic"
  Auth: AUTH-1 (ANTHROPIC_API_KEY secret, header "x-api-key")
  Optional: ANTHROPIC_HOST override (default https://api.anthropic.com)
  API version header: anthropic-version: 2023-06-01
  Default model: "claude-sonnet-4-5"
  Supports streaming, cache_control (optional), thinking/reasoning

huggingface:
  Name: "huggingface"
  Auth: AUTH-8 (OAuth primary, HF_TOKEN fallback)
  OAuth: CIMD-based, loopback port 17863, scopes: read-repos gated-repos inference-api
  configure_oauth() override; cleanup() clears OAuth cache
  Default model: "Qwen/Qwen3-Coder-480B-A35B-Instruct"


Behavioral Contracts

1. Provider construction always goes through ProviderDef::from_env() which reads Config::global(); no constructor injection of credentials.
2. CLI-subprocess providers (manages_own_context=true) skip goose-side summarization; they own conversation state internally.
3. Declarative providers are registered at registry init; custom providers can be reloaded without restart via refresh_custom_providers().
4. cleanup_provider(name) must be called when switching away from providers that hold OAuth state to avoid stale token files.
5. configure_oauth() is called by the config system (not init) when a ConfigKey has oauth_flow=true; it is NOT called during from_env().
6. GooseServer uses X-Secret-Key middleware for all routes except /status, /features, and MCP proxies.
7. Config resolution: env vars > config.yaml structured keys > config.yaml legacy flat keys.
8. CURRENT_MODEL global is updated after the provider's first response to resolve alias -> actual model.


Platform Constraints

Rust async runtime: tokio (all Provider methods async, BoxFuture for ProviderDef)
CLI-subprocess providers require the target binary on PATH (or via npm global bin resolution); desktop app adds npm bin dir via SearchPaths::builder().with_npm()
OAuth loopback servers bind to specific fixed ports; port conflicts cause auth failure
GCP auth reads ~/.config/gcloud/application_default_credentials.json (standard ADC path)
Azure auth runs az CLI subprocess (az account get-access-token)
Token caches: ~/.config/goose/{provider}/... (or $GOOSE_PATH_ROOT/config/{provider}/...)
Config file: ~/.config/goose/config.yaml (or $GOOSE_PATH_ROOT/config/config.yaml)
TLS: optional GOOSE_CLIENT_CERT_PATH / GOOSE_CLIENT_KEY_PATH / GOOSE_CA_CERT_PATH for mTLS
DEFAULT_PROVIDER_TIMEOUT_SECS = 600 (10 minutes)


Gotchas

claude_code is marked deprecated in its own metadata; new integrations should use claude_acp
claude_acp requires `claude` CLI to be authenticated separately (run `claude` to verify)
gemini_oauth uses Gemini CLI's PUBLIC installed-app credentials (not secret); env vars GEMINI_OAUTH_CLIENT_ID / GEMINI_OAUTH_CLIENT_SECRET can override
xai_oauth: loopback port 56121 is fixed by xAI's client registration; cannot be changed
chatgpt_codex: OAuth port 1455 is fixed per OpenAI docs
Declarative provider with dynamic_models=false and empty models list = construction error
HuggingFace OAuth token does NOT reliably return a refresh_token; is_expired() check gates refresh
cleanup_provider() must be called on session end for providers with registered cleanup functions or stale OAuth cache files accumulate
GOOSE_PROVIDER env var overrides ALL config file settings; check env before config file for active provider resolution
Custom provider JSON in custom_providers/ is reloaded only on refresh_custom_providers() call or server restart; changes are not hot-reloaded
GooseMode::Auto on claude_code passes --dangerously-skip-permissions to the claude CLI subprocess
GooseMode::Auto on codex passes --yolo (alias for --dangerously-bypass-approvals-and-sandbox)

Critical assertions (testable):
assert!(config_key_with_oauth_flow.oauth_flow == true && config_key_with_oauth_flow.secret == true)
assert!(provider_metadata_for_cli_provider.config_keys.iter().all(|k| !k.secret))  // CLI providers store binary path not credentials
assert!(active_provider_resolution: env var beats config file beats legacy flat key)
assert!(declarative_provider_with_dynamic_false_and_empty_models fails construction)
assert!(cleanup_registered_for_oauth_providers: ["github_copilot","databricks","databricks_v2","kimi_code","chatgpt_codex","gemini_oauth","xai_oauth","huggingface"].all_in_registry_cleanup_map)

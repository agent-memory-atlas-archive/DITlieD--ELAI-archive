Goose Agent Architecture: Providers Crate and Configuration System

## 1. goose-providers CRATE

Purpose: A shared library crate (F:\.ELAI_workflow\docs\.source\goose-main\crates\goose-providers\src\) providing reusable provider abstractions, model metadata, and conversation types. It is a SEPARATE published crate (Cargo.toml:1-9) imported by the main goose crate.

Relationship to crates/goose/src/providers: The crates/goose/src/providers directory (73+ modules) contains SPECIFIC IMPLEMENTATIONS of 40+ providers (Anthropic, OpenAI, Google, Azure, Ollama, etc.), each wiring LLM APIs to goose's internal interfaces. goose-providers is the ABSTRACTION LAYER those implementations depend on.

### goose-providers modules (19 files):

- base.rs:1-18 — Minimal trait stubs (Provider::stream signature, StreamingRequest/Response, Error types). Appears incomplete; real Provider trait is in goose/src/providers/base.rs.
- canonical/ — Model metadata normalization across providers:
  - catalog.rs — Recommended model name filtering (text + tool-call models, sorted by release date)
  - model.rs:1-132 — CanonicalModel, Modality (Text/Image/Audio/Video/Pdf), Pricing (input/output/cache_read/cache_write), Limit (context, output), ThinkingMode (Enabled/Adaptive/AlwaysOnAdaptive), Modalities struct (input/output arrays)
  - name_builder.rs — canonical_name(provider, model_name) → e.g., "gpt-4o" → "openai/gpt-4o"; strip_version_suffix(), map_to_canonical_model()
  - registry.rs — CanonicalModelRegistry loaded from canonical_models.json (bundled); get_all_models_for_provider()
  - data/ — canonical_models.json (source of truth from OpenRouter fetch), canonical_mapping_report.json (tracking per-provider model mappings), provider_metadata.json
  - README.md:1-24 — "cargo run --bin build_canonical_models" fetches OpenRouter API, writes to canonical/data/
- conversation/ — Protocol messages and token tracking:
  - message.rs — Message (role: user/assistant, content: MessageContent[], metadata: MessageMetadata), MessageContent enum (Text, Thinking, Tool(name, content), ToolResult)
  - token_usage.rs — Token counts (input/output/cache_read/cache_write)
  - tool_result_serde.rs — Serialization helpers for tool results
- conversation.rs:1-80 — Conversation(Vec<Message>), validates user/assistant alternation, merges streaming chunks, handles thinking blocks
- errors.rs — Error types (thiserror)
- formats/ — Provider-specific protocol adapters:
  - formats.rs — OpenAI, Anthropic, Bedrock, Google, GCP Vertex AI, Ollama, OpenRouter, Databricks, Snowflake formats
  - openai.rs — OpenAI API request/response shape
- images.rs — Image handling (base64/URL reference)
- json.rs — JSON schema handling utilities
- lib.rs:1-10 — Public API: base, canonical, conversation, errors, formats, images, json, thinking, utils (mcp_utils private)
- mcp_utils.rs — Extract text from MCP resources
- thinking.rs — Extended thinking / reasoning token support
- utils.rs — String normalization, URI parsing

Key types exported from goose-providers:
- CanonicalModel (id: "anthropic/claude-3.5-sonnet", pricing, limits, modalities)
- Message, MessageContent, Conversation
- Modality, Pricing, Limit, ThinkingMode

Grounded usage: goose/src/providers imports goose_providers::canonical::{CanonicalModel, CanonicalModelRegistry} for model metadata lookups.

---

## 2. CONFIG SYSTEM (crates/goose/src/config/)

Config file location and layering:
- Platform-specific base: etcetera crate (XDG on Unix, ~/AppData on Windows, ~/Library on macOS); org "Block", app "goose"
- GOOSE_PATH_ROOT env var overrides all (test mode): then {root}/config/, /data/, /state/, /.agents/plugins/, /.agents/agents/
- Default user config: ~/.config/goose/config.yaml (or Windows ProgramData equivalent)
- System config: /etc/goose/config.yaml (Unix) or C:\ProgramData\goose\config.yaml (Windows)
- Additional: GOOSE_ADDITIONAL_CONFIG_FILES env var (colon-separated paths)
- Config merge order: system → user env var paths → user config (last write wins)

Config file format: YAML (serde_yaml). Key examples:
- providers: { openai: { enabled: true, model: "gpt-4o", configured: true }, anthropic: { ... } }
- active_provider: "openai"
- extensions: { developer: { enabled: true, ... }, ... }
- experiments: { some_feature: true, another: false }
- GOOSE_DISABLE_KEYRING: true/false (controls secret storage backend)

### Config modules:

- base.rs:1-1040 — Config struct (config_paths: Vec, secrets: SecretStorage enum, guard: Mutex, secrets_cache):
  - Config::global() — Lazy static singleton initialized on first access (once_cell::OnceCell)
  - Config::default() — Loads system + env paths + user config (merges in order, last wins)
  - get_param(key: &str) → T — Lookup: env var (UPPERCASE) → config file (merged) → err
  - set_param(key, value) — Write to last config file atomically (temp file + rename)
  - update_param(key, f) — Read-modify-write closure
  - get_secret(key) → T — Lookup: env var (exact case) → keyring/secrets.yaml
  - set_secret(key, value) — Write to keyring or secrets file (JSON stored in keyring as single blob; file-based fallback is secrets.yaml)
  - Atomic write: lock exclusive (fs2), write temp file, sync, rename (lines 586-605)
  - Secret storage enum (lines 137-145):
    * If feature="system-keyring" + env GOOSE_DISABLE_KEYRING not set: Keyring { service: "goose", username: "secrets" }
    * Otherwise: File { path: ~/.config/goose/secrets.yaml }
  - Unix permissions on secrets file: mode 0o600 (owner read/write only) (lines 20-26)
  - Secret cache (secrets_cache: Arc<Mutex<Option<HashMap>>>): invalidate on every mutate
  - Fallback chain: try keyring → if "No entry found" error, read secrets.yaml fallback
  - Config migrations: run_migrations() on read (line 484, migrations.rs)

- paths.rs:1-78 — Paths struct (static methods to resolve config/data/state/plugins/agents dirs via etcetera strategy)

- providers.rs:1-179 — Per-provider config (ProviderEntry: enabled, model, configured):
  - ProviderEntry stored under providers.{name} in config.yaml
  - get_provider_entry(config, name) → Option<ProviderEntry>
  - set_provider_entry(config, name, entry)
  - get_active_provider(config) → Option<String> — Lookup order: GOOSE_PROVIDER env → active_provider key → legacy GOOSE_PROVIDER key
  - get_active_model(config) → Option<String> — Lookup order: GOOSE_MODEL env → providers.{active}.model → legacy GOOSE_MODEL key
  - set_active_provider(name, model) — Updates active_provider key and providers.{name} entry (configured=true, enabled=true)

- extensions.rs:1-100+ — Per-extension config (ExtensionEntry: enabled, config):
  - Stored under extensions.{key} in config.yaml
  - ExtensionConfig enum: Platform { name }, Custom { ... }, ExternalProcess { ... }
  - get_extension_by_name(), set_extension(), remove_extension()
  - PLATFORM_EXTENSIONS static map checked on load (filters unavailable extensions)
  - Extensions loaded as a Mapping, merged with system platform extensions

- permission.rs:1-100 — Permission.yaml (separate file, not in config.yaml):
  - PermissionManager singleton (LazyLock, Arc<RwLock<HashMap>>)
  - PermissionConfig: { always_allow: [tool_names], ask_before: [...], never_allow: [...] }
  - Two permission categories: "user" (user tool permissions), "smart_approve" (AI smart approval)
  - Permission file read on startup, panics if corrupt (non-recoverable failure)

- experiments.rs:1-58 — Feature flags (HashMap<String, bool>):
  - ALL_EXPERIMENTS const array (currently empty, means no experiments defined)
  - ExperimentManager::get_all() — Returns all experiments + syncs user list with ALL_EXPERIMENTS
  - ExperimentManager::is_enabled(name) — Check single experiment flag
  - Stored in config.yaml under "experiments" key

- declarative_providers.rs:1-100+ — Provider schema for custom/setup (DeclarativeProviderConfig):
  - ProviderEngine enum: OpenAI, Anthropic, Ollama
  - EnvVarConfig: { name, required, secret, primary, description, default }
  - DeclarativeProviderConfig: { name, engine, display_name, api_key_env, base_url, models, headers, timeout_seconds, supports_streaming, requires_auth, catalog_provider_id, env_vars: Vec<EnvVarConfig> }
  - custom_providers_dir() → ~/.config/goose/custom_providers/
  - Provider setup flows (signup_openrouter, signup_tetrate, signup_nanogpt) use this schema
  - env_vars: Optional vector of environment variable overrides for the provider (lines 89+)

- goose_mode.rs — Session mode (development/production/custom)

- migrations.rs — Config schema migrations (run on load)

- search_path.rs — Search context paths

- mod.rs:1-38 — Public exports (Config, ConfigError, ProviderEntry, ExperimentManager, PermissionManager, etc.)

---

## 3. SECRETS / KEY MANAGEMENT (DEEPEST SECTION)

Exact mechanism: Two-tier secret storage with env var override:

**Lookup order (get_secret):**
1. Environment variable with EXACT key name (case-sensitive, not uppercased like config params)
2. System keyring (if feature="system-keyring" and GOOSE_DISABLE_KEYRING env var not set)
3. File fallback (secrets.yaml in config dir)

**Storage:**
- Keyring: All secrets stored as ONE JSON object in OS keyring entry { service: "goose", username: "secrets" } (line 39-41)
  * Linux/Windows: Native keyring (D-Bus secret service / Windows Credential Manager)
  * macOS: Keychain
  * Missing keyring: Silently falls back to File storage (lines 638-646)
- File (secrets.yaml): Plain YAML, one key-per-secret at top level (key: value), file perms 0o600 (Unix) / default (Windows) (lines 18-36)
  * Windows: File created with std::fs::write (no explicit perms; ACL inherited)
  * Unix: OpenOptionsExt::mode(0o600) before write
- Env var: Takes precedence over both; no prefix/case conversion

**API keys referenced, not inlined:**
- Config stores api_key_env: "OPENAI_API_KEY" (the ENV VAR NAME to look up)
- At runtime: get_secret("OPENAI_API_KEY") checks env, then keyring, then file
- Keys never appear in config.yaml (YAML is readable; secrets.yaml is 0o600)

**Redaction in logs:**
- No explicit log redaction visible in config/base.rs
- Keyring password operations (line 892): set_password(json_string) — argument is the JSON blob, not individual keys
- Secrets cache invalidate (line 907): emptied on every set_secret() to prevent stale in-memory copies

**Caching:**
- secrets_cache: Arc<Mutex<Option<HashMap>>> (line 134)
- Populated on first all_secrets() call (line 625)
- Invalidated on mutate: invalidate_secrets_cache() clears it to None (line 1029+)
- Miss on next get_secret() causes reload from keyring/file

**Provider-specific keys:**
- providers.rs stores ProviderEntry (model, enabled, configured) in config.yaml (not secrets)
- Provider API keys stored as secrets with key names like "OPENAI_API_KEY", "ANTHROPIC_API_KEY", etc.
- Example flow: set_secret("OPENAI_API_KEY", "sk-...") → mutate_secrets() → write to keyring or file

---

## 4. PORTABLE IDEAS (Mechanisms Worth Reimplementing)

1. **Multi-layer config merge** (base.rs:287-318):
   - Load multiple YAML files in order (system → user-env → user-home)
   - Merge via merge_config_values (preserves nested keys, last write wins)
   - Reusable: load system → org-wide → user → project in same order
   - Evidence: config_paths: Vec<PathBuf>, load() iterates and merges all, write_path() = last entry

2. **Atomic config writes** (base.rs:572-608):
   - Write to .tmp file, lock exclusive (fs2::FileExt), sync to disk, atomic rename
   - Prevents partial writes on crash/interrupt
   - Reusable in any Rust agent for YAML/JSON config
   - Evidence: temp_path creation, lock_exclusive(), file.sync_all(), fs::rename()

3. **Two-tier secret storage with automatic fallback** (base.rs:369-386, 627-658):
   - Primary: OS keyring (Keyring crate with D-Bus/Credential Manager/Keychain)
   - Fallback: File storage (YAML in 0o600 file)
   - Feature-gated (system-keyring feature flag, lines 4-6)
   - Env var override to disable keyring globally
   - Reusable: Try feature-detected keyring, if unavailable/disabled, transparent file fallback
   - Evidence: secret_storage() fn returns enum SecretStorage, match on variant at write time

4. **Cache invalidation on mutation** (base.rs:619-658, 907):
   - Secrets loaded once into Arc<Mutex<Option<...>>> cache
   - Every set/delete clears cache with invalidate_secrets_cache()
   - Next read reloads from source
   - Reusable pattern for any cached config access
   - Evidence: *cache = None on invalidate, then check if cache.is_some() on read

5. **Environment variable parameter override** (base.rs:730-735):
   - Any config key K can be overridden by env var {UPPERCASE_K}
   - Parsed intelligently: JSON → primitives (bool, int, float) → string
   - Applies to both config params and secrets (lines 847-850)
   - Reusable: env::var(key.to_uppercase()) before file lookup
   - Evidence: get_param checks env_key = key.to_uppercase(); parse_env_value() handles JSON/primitive fallback

6. **Provider configuration schema** (declarative_providers.rs:34-100):
   - DeclarativeProviderConfig: { name, engine, api_key_env, base_url, models, env_vars }
   - EnvVarConfig: { name, required, secret, primary, description, default }
   - Enables custom provider YAML without code changes
   - Reusable: agents can define new providers in YAML (engine, auth fields, model list)
   - Evidence: custom_providers_dir(), ProviderEngine enum (OpenAI/Anthropic/Ollama), env_vars vector

7. **Centralized model metadata registry** (goose-providers canonical/ module):
   - CanonicalModel struct with standardized fields (pricing, limits, modalities, thinking_mode)
   - Fetched from OpenRouter, bundled in canonical_models.json
   - Name normalization: provider_model → canonical_name (strips version, maps variants)
   - Reusable: build once offline, lookup at runtime without API calls
   - Evidence: goose-providers/src/canonical/model.rs:1-132, registry.rs, build_canonical_models.rs CLI

8. **Atomic RMW (read-modify-write) for structured config** (base.rs:755-770):
   - update_param(key, closure_fn) loads current, calls closure, serializes result, writes
   - Prevents race conditions in multi-threaded config access
   - Reusable: Mutex::lock() guards, load → fn(&mut T) → save pattern
   - Evidence: update_param signature, _guard = guard.lock(), load_write_config() → f(current) → save_values()

9. **Permission management as separate YAML** (permission.rs:1-100):
   - PermissionManager singleton loads permission.yaml once
   - Three-tier allow/ask/deny per tool per category
   - Panics on corrupt file (fail-closed)
   - Reusable: categorized permission system, LazyLock singleton, RwLock for concurrent reads
   - Evidence: PermissionManager::new() loads YAML, panics if parse fails, instance() returns Arc clone

10. **Hot reload support** (paths.rs + Config::global() + ExperimentManager):
    - Config singleton allows updates via set_param() → immediately written to disk
    - Experiments auto-sync with ALL_EXPERIMENTS ground truth on every get_all()
    - No explicit file watch, but mutations trigger immediate write (atomic) + cache invalidation
    - Reusable: Lazy static config + mutex guard on writes + cache invalidate on mutation
    - Evidence: Config::global() OnceCell::get_or_init(), set_param() saves immediately, secrets_cache invalidate

---

## Architecture Summary

goose separates concerns:
- **goose-providers**: Shared abstractions (CanonicalModel, Message, Conversation, Modality) published independently
- **goose/providers/**: 40+ specific provider implementations (Anthropic, OpenAI, etc.) consuming goose-providers
- **goose/config/**: Unified config system (YAML, keyring + file secrets, env override, multi-layer merge, hot-reload)
- **config/permission.rs**: Separate YAML for tool permissions (PermissionManager singleton)
- **config/extensions.rs**: Per-extension config stored inline in config.yaml

The design reflects Block's architecture goal: A single agent framework that integrates 40+ LLM providers via a minimal abstraction layer (goose-providers) + provider-specific adapters (goose/providers/), with configuration centralized and secrets locked in OS keyring.

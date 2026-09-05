Goose Integration Map: Server, MCP, SDK, Macros

GOOSE-SERVER (41 files): HTTP API frontface for desktop UI

Route Catalog (16 route modules configured in routes/mod.rs):
- /status (GET) — health check + system_info metadata + diagnostics ZIP export
- /reply (POST, SSE) — streaming agent message execution with token state tracking
- /session/* (GET/PUT/POST) — session CRUD, fork/truncate/copy, extension mgmt, name updates
- /action_required (POST) — action confirmation handler for tool execution pauses
- /agent/* (POST) — agent command execution, response formatting
- /config_management (PUT) — config updates + provider reset
- /prompts (GET) — static prompt catalog
- /recipe/* (GET/POST) — recipe definition + parameter application
- /schedule (POST) — scheduled execution + job listing
- /setup (POST) — initial onboarding + import flows
- /telemetry/event (POST) — analytics relay (PostHog integration when feature enabled)
- /tunnel/* (POST) — tunnel management (lapstone SSH bridge)
- /gateway/* (POST) — internal routing abstraction
- /mcp-ui-proxy (POST) — MCP UI command relay (secret-key auth)
- /mcp-app-proxy (POST) — MCP app-level proxy (secret-key auth)
- /session_events (GET, SSE) — event bus streaming (progress, notifications)
- /sampling/* (POST) — parameter sampling + distribution analysis
- /dictation/* (POST) — audio transcription provider routing
- /features (GET) — feature flag state

Auth Model (auth.rs):
- Token: X-Secret-Key header matching server secret (token_matches via goose::acp::transport::auth)
- Exempted: /status, /features, /mcp-ui-proxy, /mcp-app-proxy (no key required)
- Implementation: Axum middleware check_token on all routes except exempted

Usage/Token/Cost Data (routes/reply.rs):
- TokenState struct: input_tokens, output_tokens, total_tokens, accumulated_input_tokens, accumulated_output_tokens, accumulated_total_tokens, accumulated_cost (Option<f64>)
- get_token_state(session_manager, session_id) → TokenState fetched from Session object at routes/reply.rs:156
- SSE MessageEvent enum carries token_state in Message and Finish variants
- Session metrics tracked: input_tokens, output_tokens, total_tokens, accumulated_* fields, accumulated_cost
- Telemetry instrumented at routes/reply.rs:420 (monotonic_counter.goose.session_tokens)
- Tool telemetry: tool_calls (name, count) + tool_completions (name, result=success|error) tracked per tool

Streaming Transport:
- SSE (Server-Sent Events) via Axum response body with text/event-stream
- SseResponse wrapper (routes/reply.rs:92) sends JSON MessageEvent lines prefixed with "data: "
- Bidirectional: POST /reply sends ChatRequest, GET/upgrade same endpoint streams MessageEvent replies
- CancellationToken lifecycle for mid-stream abort

Session State Model (state.rs):
- AppState holds session_manager: SessionManager (from goose::session)
- Session object fields: id, name, input_tokens, output_tokens, total_tokens, accumulated_* variants, accumulated_cost, extension_data, user_recipe_values
- ForkedSession model captures state at timestamp for truncate/copy operations


GOOSE-MCP (16 files): Built-in MCP servers + subprocess/daemon spawner

Four Builtin Extensions (lib.rs BUILTIN_EXTENSIONS map):
1. autovisualiser
   - UI element introspection + visual tree dumping
   - Tools: screenshot, element inspection, accessibility tree
   - File: goose-mcp/src/autovisualiser/ (Unix/macOS platform-specific)

2. computercontroller
   - System automation + file operations + web scraping
   - Tools: computer_control (mouse/keyboard), web_scrape (fetch+cache), automation_script (shell/batch/ruby/powershell), cache (file caching), xlsx_tool (Excel read), docx_tool (Word read), pdf_tool (PDF extract)
   - Parameters: SaveAsFormat enum (text/json/binary), ScriptLanguage enum (shell/batch/ruby/powershell), CacheCommand enum
   - Resource registration: saves outputs to cache dir, registers as MCP resources with file:// URIs
   - File: goose-mcp/src/computercontroller/mod.rs (1612 lines, platform-specific OS modules)

3. memory
   - Persistent user + project memory storage
   - Tools: remember_memory (store with category+tags), retrieve_memories (query by category), remove_memory_category, remove_specific_memory
   - Storage: ~/.config/goose/memory/ (global) + .goose/memory/ (local per project)
   - Scope: is_global boolean distinguishes session-scoped vs cross-session memory
   - Execution: loads global memories into system prompt at server boot (memory/mod.rs:115-142)
   - File: goose-mcp/src/memory/mod.rs

4. tutorial
   - Onboarding + structured learning
   - File: goose-mcp/src/tutorial/ (details omitted, training wheels)

MCP Server Runner (mcp_server_runner.rs):
- serve(ServerHandler) generic function spawns handler on tokio task
- McpCommand enum (AutoVisualiser | ComputerController | Memory | Tutorial) dispatched in main.rs
- DuplexStream pair (stdin/stdout simulation) for stdio MCP transport
- ErrorData + tracing integration for server faults


GOOSE-SDK (4 files): FFI bindings for cross-language agents

lib.rs:
- Re-exports goose_sdk_types::{custom_notifications, custom_requests}
- uniffi feature flag compiles cdylib/staticlib for Python + Kotlin bindings
- bindings.rs: scaffold for future SDK API (currently ping -> pong stub)
- Setup: uniffi::setup_scaffolding!("aaif_goose")

Purpose: Expose Goose ACP client protocol to non-Rust environments; currently minimal


GOOSE-SDK-TYPES (3 files): Shared wire types for ACP + uniffi

lib.rs exports:
- custom_notifications: notification types from ACP
- custom_requests: request/response DTOs for _goose/* JSON-RPC methods

Custom Methods (custom_requests.rs, 1472 lines):
- _goose/unstable/session/* (add/remove/list extensions, working-dir update, system-prompt set, steer, delete, fork, info, project update, rename, archive, export/import)
- _goose/unstable/tools/* (list, call with args)
- _goose/unstable/resources/* (read resource by URI + extension)
- _goose/unstable/config/extensions/* (get available, list configured, add/remove/set-enabled)
- _goose/unstable/preferences/* (read, save, remove keyed prefs: AutoCompactThreshold, GooseThinkingEffort, VoiceAutoSubmitPhrases, VoiceDictationProvider, VoiceDictationPreferredMic)
- _goose/unstable/defaults/* (read/save provider+model defaults)
- _goose/unstable/onboarding/* (scan import sources, apply candidate imports)
- _goose/unstable/dictation/* (transcribe, config status, secret save/delete, models list/download/progress/cancel/delete/select)
- _goose/unstable/providers/* (config read/save/delete/status/auth, inventory refresh, custom provider CRUD, catalog list/template, supported-models list)
- _goose/unstable/sources/* (list/create/update/delete/export/import skills+recipes+projects)
- _goose/unstable/elicitation/* (respond to pending MCP tool confirmations)
- session/* (delete session)

Response Shapes:
- GetToolsResponse: tools Vec<serde_json::Value> (flexible tool schema)
- ReadResourceResponse: result serde_json::Value (MCP ReadResourceResult)
- GooseToolCallResponse: content Vec<serde_json::Value>, structured_content Option, is_error bool, _meta Option
- GetSessionInfoResponse: SessionInfo (agent_client_protocol type)
- ProviderInventoryEntryDto: provider_id, configured, models Vec, last_updated_at, stale flag
- DictationConfigResponse: providers HashMap<name, DictationProviderStatusEntry> (per-provider status, selected model, available models)

Preference Keys (enum PreferenceKey):
- AutoCompactThreshold (token count for auto-truncation)
- GooseThinkingEffort (reasoning mode level)
- VoiceAutoSubmitPhrases (dictation trigger phrases)
- VoiceDictationProvider (active transcription provider)
- VoiceDictationPreferredMic (audio input device)


GOOSE-ACP-MACROS (1 file, 200+ lines): Procedural macros for ACP method dispatch

Attribute #[custom_methods]:
- Applied to impl block containing #[custom_method(RequestType)] handler methods
- Extracts method name from RequestType::JsonRpcRequest #[request(method = "...")]
- Generates two methods:
  1. handle_custom_request(method, params) — dispatcher that matches method name, deserializes params JSON, calls handler, serializes response
  2. custom_method_schemas() — returns Vec<CustomMethodSchema> with JSON Schema for each method's params/response types
- Supports handlers with zero params or one typed param (beyond &self)
- Return type required: Result<T, agent_client_protocol::Error> where T: Serialize
- Usage in goose codebase: wires ACP custom methods to impl handler blocks without duplication


PORTABLE IDEAS (Rust + Tauri + Svelte dashboard target)

1. UI↔Engine Usage Contract (routes/reply.rs + goose-sdk-types custom_requests)
   - SSE streaming with token state in each message event
   - TokenState: input/output/total + accumulated versions + cost
   - Injected into every MessageEvent::Message and MessageEvent::Finish
   - Decouples token tracking from conversation state (queries Session separately)
   Evidence: routes/reply.rs:156-177 (get_token_state), MessageEvent enum lines 129-138

2. Auth Pattern: Shared Secret Key Header
   - X-Secret-Key middleware on AppState (auth.rs)
   - Per-request validation: token_matches(header_value, state_secret)
   - Exempts health/features/MCP-proxy endpoints
   - Lean: no JWTs, no session tokens, just a pre-shared HTTPS secret
   Evidence: auth.rs:10-33, routes/mod.rs:31

3. Session Fork/Truncate Model (routes/session.rs + session_manager)
   - Fork: copy_session + optional truncate_conversation(timestamp)
   - Preserves conversation history or cuts at timestamp
   - Atomic: session_manager.update(session_id).apply() pattern
   Evidence: routes/session.rs:338-410 (fork_session handler)

4. Streaming Cancellation (routes/reply.rs)
   - CancellationToken per request
   - Client hang-up detected on tx.send() error -> cancel_token.cancel()
   - Stops agent execution mid-stream
   Evidence: routes/reply.rs:179-195 (stream_event), cancel on line 193

5. MCP Built-in Server Pattern (goose-mcp/lib.rs)
   - Macro-driven lazy map of name → SpawnServerFn
   - Each server is a self-contained ServerHandler impl
   - Spawned on tokio::spawn with DuplexStream for stdio
   - Decouples server startup from agent lifecycle
   Evidence: goose-mcp/src/lib.rs:48-64 (builtin! macro + BUILTIN_EXTENSIONS)

6. Preference Storage (goose-sdk-types custom_requests)
   - Allowlist of keyed prefs: AutoCompactThreshold, VoiceAutoSubmitPhrases, etc.
   - Read/save/remove operations as custom JSON-RPC methods
   - UI can persist user intent without embedding config format
   Evidence: custom_requests.rs:309-359 (PreferencesRead/Save/Remove + PreferenceKey enum)

7. Procedural Macro for Method Dispatch (goose-acp-macros)
   - #[custom_methods] on impl eliminates manual route registration
   - #[custom_method(RequestType)] extracts method name at compile time
   - Generates dispatcher + schema catalog in two generated methods
   - Reduces boilerplate for adding new ACP methods
   Evidence: goose-acp-macros/src/lib.rs:42-80 (macro definition)

8. Session Metadata Query (custom_requests.rs GetSessionInfoResponse)
   - Light-weight metadata fetch without loading full conversation
   - SessionInfo type (agent_client_protocol): id, title, updated_at, message_count
   - Decouples list/browse from detail load
   Evidence: custom_requests.rs:478-492 (GetSessionInfoRequest/Response)

9. Provider Inventory Model (custom_requests.rs ProviderInventoryEntryDto)
   - Dynamic model discovery (ProviderInventoryModelDto: id, name, context_limit, reasoning, recommended)
   - Configured/stale flags for UI status rendering
   - Refresh state + last_updated_at for polling
   - Independent of hardcoded model registry
   Evidence: custom_requests.rs:1315-1379 (ProviderInventoryModelDto + ProviderInventoryEntryDto)

10. Cached Resource Registration (goose-mcp computercontroller web_scrape + cache)
    - Tool downloads content, caches to timestamped file in cache_dir
    - Registers file:// URI as MCP resource (uri, mime_type, text metadata)
    - Later reads via ReadResource follow same protocol
    - Decouples download-and-cache from UI direct fetch
    Evidence: goose-mcp/src/computercontroller/mod.rs:555-600 (save_to_cache + register_as_resource)

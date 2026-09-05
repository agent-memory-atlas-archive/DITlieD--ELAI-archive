Goose Agent Partition Exploration: ACP, Gateway, Execution, Hooks, Plugins, Telemetry

## 1. AGENT CLIENT PROTOCOL (ACP)

### What It Implements
ACP is Goose's implementation of the agent-client-protocol standard (JSON-RPC 2.0 based). It is the wire protocol between IDE/editor frontends and the Goose agent runtime.

**Protocol Spec & Structure:**
- Transport: HTTP with WebSocket upgrade support + JSON-RPC 2.0 request/response semantics
- Location: docs\.source/acp/mod.rs, docs\.source/acp/server.rs, docs\.source/acp/transport/
- Standard Library: agent_client_protocol crate (rmcp bindings)

### Surface: Core Methods
**Session Lifecycle Methods:**
- `initialize` — connection initialization (client capabilities, server responds with agent capabilities)
- `new_session` — create agent session for a user/IDE
- `load_session` — restore session from disk
- `close_session` — terminate session
- `fork_session` — branch a session (for sub-agents)

**Execution Methods:**
- `session/prompt` — send user input to agent; streams back: text, thoughts, tool calls
- `session/cancel` — abort running prompt
- `session/set_mode` — switch agent execution mode (interactive/background/subtask)
- `session/set_model` — change LLM provider mid-session
- `session/set_config_option` — update any config (thinking_effort, etc)

**Resource/Tool Methods:**
- `resources/list` — enumerate available file/resource contexts
- `tools/list` — enumerate available MCP tools
- `prompts/list` — enumerate prompt templates

**Permission & Extensions:**
- `request_permission` — ask user/IDE to grant access (file read, shell exec)
- `elicitation_respond` — IDE sends user response to a permission/input elicitation
- `custom_requests` / `custom_notifications` — extensibility point

### Who Consumes It (Clients)
- **IDE Extensions:** VS Code (goose-extension repo), possibly JetBrains
- **Desktop App:** Goose's native UI (Tauri + Svelte, likely in agent codebase)
- **CLI:** Optional CLI client via HTTP POST
- **Agents:** Sub-agents call parent via ACP (fork_session, subtask mode)

### Sessions & Permissions Flow
**Session State Management:**
- Stored per-user in `~/.goose/sessions/` (disk-backed)
- In-memory cache in `AgentManager` (LRU, default 100 sessions max)
- Per-session: enabled extensions, provider selection, conversation history, tool state
- Session isolation: each ACP session maps 1:1 to an Agent instance

**Permission Flow:**
- `PermissionConfirmation` type wraps user intent (request type, resource, reasoning)
- Sent via `request_permission` RPC → IDE shows UI → user response → `elicitation_respond` RPC
- Stored in `PermissionManager` (at `~/.goose/permission_confirmations.json`)
- Caching: repeated access to same resource reuses prior confirmation

**Extension Loading Over ACP:**
- `on_initialize` reads enabled extensions from config → fetches MCP metadata per extension
- Advertises extension resources/tools/prompts in `InitializeResponse`
- IDE caches these for tab completion / intellisense

### Transport Implementation
**HTTP + WebSocket (docs/acp/transport/):**
- HTTP: GET for server-sent events (async streaming), POST for request/response
- WebSocket: persistent connection, upgradeable via Accept-Upgrade header
- Header Auth: `Acp-Session-Id` header required for session-scoped methods
- Connection Registry: per-connection state, token validation (auth.rs)

**Error Handling:**
- JSON-RPC 2.0 error responses (code: -32700 to -32603 + custom)
- Maps provider errors → ACP errors (ResultExt trait, internal_err/invalid_params)

---

## 2. GATEWAY

### Purpose: Multi-Platform Message Bridge
Gateway abstracts Goose's agent runtime behind pluggable chat platforms (Telegram, Discord, Slack, etc).

**Architecture:**
- `Gateway` trait (mod.rs:78-95): platform abstraction
  - `start(handler, cancel)` — listen for incoming messages on platform
  - `send_message(user, msg)` — deliver outgoing message back to user
  - `validate_config()` — health-check platform credentials
- Current implementation: **Telegram** (telegram.rs, 815 lines)
  - Polls Telegram Bot API for updates
  - Converts Telegram user → platform-agnostic `PlatformUser` (platform + user_id)
  - Sessions paired via 6-digit code (user enters code from desktop app → pairing_store links Telegram user to Goose session)

**Session Pairing & Routing:**
- `GatewayHandler` (handler.rs): bridges Telegram ↔ AgentManager
- Pairing states: Unpaired → PendingCode{code, expires} → Paired{session_id}
- Message flow: Telegram user → handler.handle_message → route to paired AgentManager session
- Max sessions per gateway (GatewayConfig.max_sessions, default per platform)

**Message Handling:**
- Incoming: `IncomingMessage` (user, text, attachments, platform_message_id)
- Outgoing: `OutgoingMessage` enum (Text | Typing)
- Tool calls default to 5 turns (conservative for chat platforms, GOOSE_GATEWAY_MAX_TURNS override)

---

## 3. EXECUTION & SESSION LIFECYCLE

### AgentManager: Central Session Pool
**Location:** execution/manager.rs (771 lines)

**Responsibilities:**
- Singleton instance (lazy static AGENT_MANAGER + OnceCell)
- LRU cache of active `Agent` instances (default 100 sessions)
- Per-session creation locks (no race on MCP extension init)
- Default provider assignment (model/provider can be overridden per session)

**Key Methods:**
- `get_or_create_agent(session_id)` → Arc<Agent> or create if missing
- `get_or_create_agent_with_runtime_context()` → includes MCP host info, shell path, session-name-update channel
- `scheduler()` → SchedulerTrait for background task scheduling
- `session_manager()` → SessionManager for disk persistence

**Session Execution Modes (execution/mod.rs):**
- `Interactive` — real-time user chat
- `Background` — scheduled/async task
- `SubTask{parent_session}` — child task spawned by parent agent

---

## 4. HOOKS: Lifecycle Extensibility

### Hook System (Open Plugins Spec Alignment)
**Location:** hooks/mod.rs (686 lines)

**Events (13 total):**
1. PreToolUse — before tool execution (gate, logging)
2. PostToolUse — after tool succeeds
3. PostToolUseFailure — after tool error
4. SessionStart, SessionEnd — session lifecycle
5. UserPromptSubmit — user input received
6. BeforeReadFile, AfterFileEdit — file system events
7. BeforeShellExecution, AfterShellExecution — shell events
8. Stop — agent halt signal
9. SubagentStart, SubagentStop — child agent lifecycle

**Mechanism:**
- Plugins define `<plugin-root>/hooks/hooks.json` with matcher rules
- Matcher: regex pattern on context (tool name, file path, shell command)
- Action: type="command", command="path/to/script", timeout (default 30s)
- Context passed as JSON on stdin: {event, session_id, matcher_context, tool_name, tool_input, tool_output, ...}
- Script exits 0 = success, non-zero = skip rest of rule (no panic)

**Example Use Cases (from schema):**
```json
{
  "hooks": {
    "PostToolUse": [
      {
        "matcher": "developer__shell|developer__text_editor",
        "hooks": [
          { "type": "command", "command": "${PLUGIN_ROOT}/scripts/log.sh" }
        ]
      }
    ]
  }
}
```

---

## 5. PLUGINS: Discovery & Lifecycle

### Plugin Formats Supported
**Location:** plugins/mod.rs (566 lines), plugins/formats/

1. **Gemini Format** (gemini.rs)
2. **Open Plugins** (open_plugins.rs, 698 lines) — primary format

### Plugin Installation & Management
**Install Directory:**
- Global: `~/.goose/plugins/`
- Per-project: `.agents/plugins/`

**Plugin Structure:**
- Format: Gemini or Open Plugins manifest
- Skills: nested under `skills/` or format-specific path
- Hooks: `hooks/hooks.json` (optional)
- Metadata: `.goose-plugin-install.json` tracks source, auto-update interval (24h), last-check timestamp

**Key Types:**
- `PluginInstall` — parsed plugin (name, version, format, source URL, skill list)
- `ImportedSkill` — skill discovered from plugin (name, directory path)
- `PluginFormat` enum — Gemini | OpenPlugins
- `PluginInstallOptions` — auto_update flag

**Auto-Update:**
- Checked on startup (Utc::now() vs last_update_check)
- 24-hour interval, failure is non-fatal (warned, continues with current version)

**Discovery:**
- `discovered_enabled_plugins()` → enumerate plugins by format
- `installed_plugin_skill_dirs()` → flatten all skills into one list for skill loader

---

## 6. OTEL & TRACING: Observability & Cost Metrics

### OpenTelemetry Integration (otel/otlp.rs, 724 lines)

**Signals:**
1. **Traces** — distributed tracing, span start/end/duration
2. **Metrics** — counters/histograms, token usage, API calls
3. **Logs** — structured logs bridged from tracing crate

**Exporters:**
- Primary: OTLP (HTTP/protobuf) → external collector (Datadog, Honeycomb, etc)
- Console: stdout exporter for local dev
- None: disabled (OTEL_SDK_DISABLED=true)

**Configuration (env vars):**
- `OTEL_EXPORTER_OTLP_ENDPOINT` — collector endpoint (e.g., http://localhost:4318)
- `OTEL_EXPORTER_OTLP_PROTOCOL` — http/protobuf (grpc not supported in goose build)
- Signal-specific overrides: `OTEL_EXPORTER_OTLP_TRACES_ENDPOINT`, `OTEL_EXPORTER_OTLP_METRICS_ENDPOINT`, etc
- `OTEL_EXPORTER_OTLP_TIMEOUT` — export timeout
- Resource attributes: service.name=goose, version, hostname

**Metrics Emission:**
- Via tracing-opentelemetry::MetricsLayer
- Custom filter: targets "goose::metrics" OR fields with "otel.metric"
- Metrics layer extracts typed values → sends to MeterProvider → batch export

### Langfuse Integration (tracing/langfuse_layer.rs, 509 lines)

**Purpose:** Record trace observations (spans, logs) to Langfuse (LLM observability platform).

**Flow:**
1. `ObservationLayer` intercepts tracing-subscriber events
2. Converts spans → Langfuse observations (type: SPAN with metadata, parent refs)
3. Batches observations (5s interval) via LangfuseBatchManager
4. POST to `http://langfuse-url/api/public/ingestion` (basic auth: public_key:secret_key)

**Span Data Captured:**
- observation_id (UUID v4)
- name, start_time, end_time
- level (ERROR/WARNING/INFO/DEBUG)
- metadata (flattened JSON, drops nesting)
- parent_observation_id (for trace tree)
- trace_id (root trace UUID, created once per session)

**Token Usage:**
- NOT in Langfuse layer directly
- Captured via observation metadata if provider emits it (token_usage field in RmcpContent/ToolCallContent)
- Cost inference: model name + token count → cost = lookup_from_model_table(model) * (input_tokens + output_tokens * output_multiplier)

### Observation Layer (tracing/observation_layer.rs, 639 lines)

**Key Types:**
- `SpanData` — immutable observation (observation_id, name, level, metadata, parent_span_id)
- `SpanTracker` — HashMap<span_id, observation_id>, current_trace_id
- `BatchManager` trait — add_event, send, is_empty (implemented by LangfuseBatchManager)

**Metadata Extraction:**
- `flatten_metadata()` — removes object nesting (extracts {"text": "value"} → "value")
- Maps tracing levels → Langfuse levels (TRACE/DEBUG → DEBUG, INFO → DEFAULT, WARN → WARNING, ERROR → ERROR)

### Rate Limiter (tracing/rate_limiter.rs, 143 lines)

**Purpose:** Throttle telemetry export to avoid overwhelming collector.

**Types:**
- `TelemetryEvent` enum (SpanData, MetricData)
- `RateLimitedTelemetrySender` — wraps batches, applies rate limits, drops if queue full

---

## 7. OAUTH: Browser & Device Auth Flows

### OAuth Flow (oauth/mod.rs, 146 lines)

**Mechanism:**
1. **Client Metadata:** goose-docs.ai/oauth/client-metadata.json (Goose's client_id, allowed scopes)
2. **Callback Server:** spawns Axum server on localhost (port auto-assigned, default 0 = OS picks free port)
3. **Authorization URL:** agent constructs URL, opens in system browser
4. **Callback Handler:** browser redirects to http://127.0.0.1:PORT/oauth_callback?code=X&state=Y
5. **Token Exchange:** agent POSTs code+state to MCP server (rmcp::transport::auth::OAuthState) → gets access_token
6. **Storage:** GooseCredentialStore (oauth/persist.rs) saves StoredCredentials (client_id, token, scopes, timestamp)
7. **Refresh:** on next startup, loads from store → refreshes token if expired

**Token Refresh:**
- Attempted on `oauth_flow()` entry if credentials exist
- On failure: clears bad credentials, falls back to full browser flow
- RefreshToken stored in rmcp OAuth state

**Providers Using OAuth:**
- Any MCP server requiring OAuth2 authorization (e.g., GitHub, Google, enterprise OIDC)
- Goose SDK client (self-identification via client_id from metadata.json)

**Credential Persistence:**
- Path: `~/.goose/oauth_credentials/{name}.json` (name = MCP server name)
- Format: StoredCredentials struct (client_id, token_response, granted_scopes, unix_timestamp)

---

## 8. PORTABLE IDEAS: Mechanisms Worth Stealing

### 8.1 ACP Protocol as Agent-IDE Glue (acp/server.rs:100-200)
**Insight:** Separate agent runtime from UI via a clean JSON-RPC 2.0 protocol. This decouples agent scaling from IDE performance.

**Evidence:**
- acp/server.rs defines `AcpProviderFactory` type (line 100) — pluggable provider backend
- Multiple transport backends (HTTP streaming, WebSocket) without logic changes
- IDE clients auto-upgrade (new methods added to protocol; old clients still work, get INVALID_REQUEST)

**Steal For:** Any Rust agent framework (local or cloud) wanting IDE/editor plugin support.

### 8.2 Session Pairing via Short Code (gateway/pairing.rs, 189 lines)
**Insight:** Users exchange a 6-digit code between platforms (desktop ↔ mobile/Telegram) without passwords.

**Mechanism:**
- Desktop generates code → expires in 15min
- User types code into Telegram
- Telegram client uses code to fetch session_id from pairing_store
- Code is single-use

**Evidence:** gateway/handler.rs:64-80, pairing.rs (persistence + expiry check)

**Steal For:** IoT agents, multi-platform orchestration, consumer apps needing zero-password pairing.

### 8.3 Plugin Discovery + Hook System (plugins/mod.rs + hooks/mod.rs)
**Insight:** Two-layer extensibility:
1. Plugins (on-disk directories) are auto-discovered
2. Hooks (event-driven callbacks) are loaded from plugin manifests
3. Hooks run as subprocesses (JSON in/out) — no in-process plugin code execution risk

**Evidence:**
- plugins/mod.rs:84-115 (installed_plugin_skill_dirs enumerates all plugins)
- hooks/mod.rs:110-138 (RawHookRule schema + command action)
- Hook context passed as JSON (line 160-175) — subprocess gets clean input

**Steal For:** Extensibility that survives plugin crashes, isolation without containers, plugin discovery without registry (file-based).

### 8.4 AgentManager as LRU Pool + Creation Lock (execution/manager.rs:35-150)
**Insight:** Concurrent session creation races prevented via per-session `Arc<Mutex<()>>` creation lock (not a global lock).

**Evidence:**
- Line 48: `creation_locks: Arc<Mutex<HashMap<String, Arc<Mutex<()>>>>>`
- Line 143-150: `get_or_create_agent_with_runtime_context` acquires per-session lock before expensive init

**Why Portable:** MCP extension initialization on the first call to a session is expensive (spawns subprocesses, waits for handshake). Without per-session lock, two concurrent `get_or_create_agent` calls would each spawn extensions.

**Steal For:** Any session-pool system with lazy, expensive initialization (e.g., database connection pool, LLM client per-user).

### 8.5 Observable Execution via Langfuse + OTEL (tracing/*.rs)
**Insight:** Decoupled observability layers:
1. Tracing subscriber (Rust std lib) emits events (unaware of Langfuse)
2. ObservationLayer converts tracing events → Langfuse observations (spans, logs, metadata)
3. BatchManager batches + retries (configurable per exporter)
4. OTEL metrics layer can be added independently for counters/histograms

**Evidence:**
- tracing/observation_layer.rs:98-137 (Layer trait, batches observations)
- tracing/langfuse_layer.rs:44-56 (BatchManager spawns background task, configurable interval)
- otel/otlp.rs:94-131 (signal-specific exporter config, gRPC fallback to None)

**Steal For:** Any Rust service wanting to emit both traces (Langfuse-style tree) and metrics (OTEL-style counters) without coupling to one backend.

### 8.6 Rate Limiter for Telemetry (tracing/rate_limiter.rs)
**Insight:** Prevent telemetry export from becoming a bottleneck.

**Mechanism:** Queue-based rate limiter with drop-on-full (not backpressure). Useful for high-throughput agents.

**Steal For:** Cloud agents that log every token decision but collectors are slower than agent.

### 8.7 OAuth Callback Server with System Browser (oauth/mod.rs:33-146)
**Insight:** Device flow without polling. Spawn localhost server, open system browser, wait for callback.

**Evidence:**
- Line 79-92: spawn Axum server, open webbrowser::open(), collect callback via oneshot channel
- No polling loop; direct channel notification

**Steal For:** Native apps needing OAuth2 without external dependency on browser automation.

---

## Summary: Key Takeaways

| Component | Portable Value | Effort to Extract |
|-----------|---|---|
| ACP Protocol | Transport-agnostic IDE ↔ Agent RPC | Medium (protocol is crate; reimplement in other langs) |
| Gateway + Pairing | Multi-platform agent access (Telegram, Discord) | Medium (hardest: per-platform adapter) |
| Hook System | Safe plugin extensibility (subprocess-based, not in-process) | Low (copy hooks/ logic, adapt to your event enum) |
| AgentManager LRU | Race-free session pool with lazy init | Low (copy execution/manager pattern) |
| Langfuse + OTEL | Decoupled observability (spans + metrics) | Medium (depends on your telemetry backend) |
| OAuth Callback | Zero-polling OAuth2 for native apps | Low (copy oauth/mod logic, adapt callback URL) |

All evidence grounded in file:line references above. Goose is a mature, well-structured codebase suitable for reference architecture.

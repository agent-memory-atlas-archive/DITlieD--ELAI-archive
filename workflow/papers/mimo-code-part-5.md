MiMoCode Part 5: Server, Control-Plane, Sync, Bus, Inbox, Team, Metrics

PARTITION SCOPE: F:\.ELAI_workflow\docs\.source\MiMo-Code\packages\opencode\src\{server,control-plane,sync,bus,inbox,team,metrics}

=== FILE-BY-FILE BREAKDOWN ===

server/adapter.ts (abstraction boundary)
Purpose: runtime-agnostic server interface. Defines contract: Hono app -> Runtime (upgrade WebSocket, listen).
Exports: Opts (port, hostname), Listener (stop callback), Runtime (WebSocket upgrade + listen), Adapter (create(Hono) -> Runtime).
Mechanism: runtime selectors choose Bun or Node adapter at startup (adapter.bun.ts, adapter.node.ts).
Three-line: Universal Hono app entry point that bridges framework to runtime. Caller provides Hono instance; adapter supplies native server.

server/event.ts (event definitions)
Purpose: root event registry anchor. Two events: server.connected (empty payload), global.disposed (empty).
Exports: Event namespace with Connected and Disposed definitions (uses BusEvent.define).
Mechanism: integrates with global event bus to publish lifecycle signals across all clients.
Three-line: Early-startup (connected) and shutdown (disposed) events published to GlobalBus.emit("event"). Empty payloads signal presence/absence.

server/routes/global.ts (top-level API surface)
Purpose: global system API (health, events, config, upgrade, dispose). Four routes: /health, /event, /config (GET/PATCH), /dispose, /upgrade.
Exports: GlobalRoutes (lazy-initialized Hono router).
Key mechanisms:
  - /health: returns InstallationVersion + true flag.
  - /event (GET): Server-Sent Events endpoint. Connects to GlobalBus, sends "server.heartbeat" every 10s, publishes both bus events and sync events on one stream.
  - /config (GET/PATCH): uses AppRuntime.runPromise(Config.Service.use(...)) to read/write global cfg.
  - /dispose: calls Instance.disposeAll(), emits GlobalDisposedEvent, returns true.
  - /upgrade: routes to Installation.Service, attempts upgrade, emits Installation.Event.Updated on success.
  - All routes are documented with hono-openapi (describeRoute + resolver).
Five-line: HTTP server exposes health-check, single-topic event stream (merged bus + sync events with heartbeat), and admin endpoints (config, upgrade, dispose). Event stream guards against stalled proxies with 10s keepalive.

server/middleware.ts (HTTP middleware stack)
Purpose: error handling, auth, logging, CORS, compression guards.
Exports: ErrorMiddleware, AuthMiddleware, LoggerMiddleware, CorsMiddleware, CompressionMiddleware.
Mechanism:
  - ErrorMiddleware: catches NamedError, Session.BusyError, HTTPException, maps to HTTP status (404, 400, 500).
  - AuthMiddleware: optional basic auth (MIMOCODE_SERVER_PASSWORD env). Allows OPTIONS preflight and ?auth_token query param fallback.
  - LoggerMiddleware: logs all requests (timered).
  - CorsMiddleware: allows localhost, tauri://, opencode.ai domain, optional config.cors list.
  - CompressionMiddleware: skips /event, /global/event, and POST .../session/*/message|prompt_async (streaming paths exempt).
Three-line: HTTP middleware stack enforcing auth (optional), logging, CORS (localhost+tauri+domain whitelist), compression-aware of streaming.

server/proxy.ts (WebSocket and HTTP proxying)
Purpose: tunnel HTTP/WebSocket requests to remote workspaces (e.g., local dev server behind sidecar).
Exports: http(url, extra, req, workspaceID), websocket(upgrade, target, extra, req, env).
Mechanism:
  - http: checks Workspace.isSyncing(workspaceID) first (fail 503 if not syncing), proxies request (strips hop headers, deletes accept-encoding), parses x-mimocode-sync fence header from response, waits for sync barrier.
  - websocket: upgrades local client connection, relays to remote WebSocket, handles both-way message forwarding with queue until remote is ready.
  - Fence integration: x-mimocode-sync header tells client which event sequences to wait for before proceeding.
Four-line: HTTP/WebSocket tunnel to workspace remotes with sync-fencing. Waits for event sequence barriers (fence) before HTTP response. Queue-buffers messages until remote WebSocket opens.

server/fence.ts (event sequence synchronization barrier)
Purpose: linearize remote writes with client reads. Client sees fence header (x-mimocode-sync: {aggregate_id: seq}), waits for local event sequence to reach those numbers before proceeding.
Exports: load(), diff(), parse(), wait(), FenceMiddleware.
Mechanism:
  - load(ids?): SELECT EventSequenceTable, returns {aggregateID: seq} map. If ids given, filter.
  - diff(prev, next): returns only changed aggregate_ids (if seq changed from prev to next).
  - parse(headers): parses x-mimocode-sync header as JSON state map.
  - wait(workspaceID, state, signal): calls Workspace.waitForSync(workspaceID, state) with abort signal.
  - FenceMiddleware: before response, compare EventSequenceTable before/after; if changed, set x-mimocode-sync header with deltas.
  - Idempotence: fence is attached to HTTP response after each mutation, ensuring client knows what to wait for.
Four-line: Event-sequence barrier. Middleware records EventSequenceTable state before/after each request, diffs, attaches x-mimocode-sync header. Client polls until local state reaches fence numbers.

bus/bus-event.ts (event type registry)
Purpose: Zod-based event definition system. Events are (type, properties schema) pairs, stored in a global registry.
Exports: define(type, properties), payloads() [returns Zod union of all registered event schemas].
Mechanism:
  - define(): creates {type, properties} pair, stores in registry Map, returns for later reference.
  - payloads(): flattens all registered events into a Zod discriminated union (type: literal(type), properties: schema).
  - No versioning at bus level (bus-event is "latest only" for runtime use; SyncEvent handles versioning for durability).
Three-line: Type-safe event schema registry. Events are (type literal + Zod schema) pairs. Used in HTTP route schema documentation.

bus/global.ts (process-wide event emitter)
Purpose: single EventEmitter<{event: [GlobalEvent]}>. Subscribers listen for "event" messages (optional directory/project/workspace + payload).
Exports: GlobalBus (typed EventEmitter).
Mechanism: emit("event", {directory?, project?, workspace?, payload: any}). Listeners (streamSSE handlers, sync broadcasters) subscribe and relay.
Two-line: Node EventEmitter singleton. Carries directory/project/workspace routing context + event payload. Used by /event and /global/event SSE routes.

sync/index.ts (event sourcing + projection system, heavyweight)
Purpose: durable event log with versioning, projectors, and idempotent replay.
Exports: define(), project(), run(), replay(), replayAll(), init(), versionedType(), registry, payloads().
Key mechanisms:
  - define(input): creates {type, version, aggregate, schema, busSchema} definition. Stores in registry. Frozen after init().
  - init(input): registers all projectors (Map<Definition, ProjectorFunc>), wires events to BusEvent (latest versions only), freezes system.
  - run(def, data): aquire next seq for aggregate, invoke projector in transaction, emit to Bus + GlobalBus with versionedType(type, version).
  - replay(event): idempotence check (seq > latest), invoke process() with {publish} flag.
  - replayAll(events): batch replay from same aggregate, validates seq continuity.
  - project(def, func): returns [Definition, ProjectorFunc] pair for layer initialization.
  - Dual publish: on run/replay, events go to ProjectBus (Bus.publish) AND GlobalBus.emit("event") with sync event wrapper.
  - Database: stores in EventTable (id, seq, aggregate_id, type, data) and EventSequenceTable (aggregate_id, seq).
Six-line: Event sourcing engine. Events are versioned, projected into DB via idempotent handlers, published to both project-level Bus and global event stream. Replaying is deterministic (seq-ordered per aggregate). Projector functions are typed.

sync/schema.ts (type aliases)
Purpose: EventID, EventSequenceTable, EventTable schema definitions.
Two-line: ULID-based event IDs, sequence tracking per aggregate, data blob storage.

control-plane/workspace.ts (multi-workspace orchestrator, heavyweight)
Purpose: manage workspaces (git branches, local/remote, worktrees, adaptor-based provisioning).
Exports: create(), info accessor, ConnectionStatus, Event namespace (Ready, Failed, Restore, Status), Info type.
Key mechanisms:
  - create(input): instantiate adaptor (worktree.ts, etc.), configure workspace (dir, branch, name), insert into WorkspaceTable, wire SSE listener.
  - Adaptor pattern: each workspace type (git worktree, ...) has configure() method returning {type, branch, directory, extra, name}.
  - WorkspaceContext: thread-local workspace ID for this request.
  - Restore (progress event): broadcasts {workspaceID, sessionID, total, step} to show restore progress.
  - isSyncing(workspaceID): checks Workspace state table, used by proxy.ts to fail fast if not ready.
  - waitForSync(workspaceID, state, signal): polls until all aggregate_ids in state map reach required seqs (fence barrier).
Four-line: Workspace lifecycle (create, restore, monitor connection, sync state polling). Emits status events (connecting, connected, error). Decouples provisioning via adaptor pattern.

control-plane/sse.ts (Server-Sent Events parser)
Purpose: client-side SSE stream parser. Reads chunks, buffers, splits on double-newline, emits JSON events or unparseable text.
Exports: parseSSE(body, signal, onEvent).
Mechanism:
  - Reads ReadableStream chunks, decodes UTF-8, handles \r\n, \r, \n line endings.
  - Collects "data:", "id:", "retry:" fields into one event.
  - Tries JSON.parse; if fails, emits as {type: "sse.message", properties: {data, id, retry}}.
  - Signal abort closes reader.
Three-line: Streaming parser for Server-Sent Events. Normalizes line endings, batches by double-newline, falls back to text wrapper on JSON parse failure.

inbox/index.ts (inbox types)
Purpose: re-export wrapper.
Two-line: Export Inbox, InboxTable, sessionPromptRef.

inbox/inbox.ts (message inbox system, heavyweight)
Purpose: actor-to-actor message delivery. Senders post to inboxes; receivers drain into synthetic user messages.
Exports: send(input), drain(sessionID, actorID), gcInboxRows(cutoffMs), InboxReceiverNotFound error, Interface service.
Key mechanisms:
  - send(input): ESRCH check (receiver must exist in ActorRegistry), insert InboxRow (sender, receiver, type, content, timestamp), publish InboxArrived to Bus, fork sessionPromptRef.loop() to wake receiver (if loop available; otherwise row waits for next drain).
  - drain(sessionID, actorID): SELECT inbox rows up to MAX_DRAIN_PER_TURN (100), find last real assistant message in session (to inherit model/agent), create synthetic user message, insert parts, DELETE inbox rows. Crash window: rows can re-render if crash between insert and delete.
  - GC: 7-day TTL, runs at init, is idempotent.
  - Fork-and-forget: send() forks sessionPromptRef.loop() in service scope (survives beyond sender's lifetime).
Five-line: Durable message queue for actor-to-actor comms. Receivers drain into synthetic user messages (prepended with sender info). Watermark tracking (last real assistant) ensures correct model/agent inheritance. GC-able with 7-day expiry.

inbox/render.ts (inbox message formatting)
Purpose: convert InboxRow to markdown text for inclusion in synthetic user message.
Two-line: Formats sender/type/timestamp into message context.

team/index.ts (multi-agent team membership, lightweight)
Purpose: manage named teams of agents (sessionID, role, agent name, join timestamp).
Exports: Service (create, addMember, removeMember, getMembers, teamDir), layer.
Mechanism:
  - Store: .mimocode/teams/{teamID}/members.json (JSON array of {sessionID, agent, role, joinedAt}).
  - create(teamID, creatorSessionID): mkdir, write empty members.json, publish TeamCreated event.
  - addMember(teamID, sessionID, agent, role): read file, check if sessionID exists, append if not, write back.
  - removeMember(teamID, sessionID): read, filter, write.
  - getMembers(teamID): read file (empty list on ENOENT).
  - Effect.gen wrappers with InstanceState.context to resolve directory.
Four-line: File-backed team registry. Agents identified by (sessionID, agent name, role). On create/add/remove, publishes TeamCreated event. No races (each op is read-modify-write; concurrent ops not guarded).

metrics/event.ts (metrics event definitions)
Purpose: typed metrics schemas (model calls, tool calls, agent requests).
Exports: ModelCall, ToolCall, AgentRequest (BusEvent definitions).
Types:
  - ModelCall: {sessionID, finish_reason, ttft_ms?, latency_ms, cached_read_tokens, model_id, provider, total_tokens_in, total_tokens_out}.
  - ToolCall: {sessionID, tool_name, input_bytes, output_bytes, tool_call_id, status: "success"|"error"}.
  - AgentRequest: {sessionID, phase, task_type, surface, total_tokens_in, total_tokens_out, files_changed, validation_status}.
Three-line: Bus event schemas for three metric types (model, tool, agent). Each carries sessionID for routing.

metrics/client.ts (telemetry backend)
Purpose: post metrics to remote tracking server (ENDPOINT = https://tracking.miui.com/track/v4/o).
Exports: buildHeader(event, sessionID), postEvents(payload), constants (ENDPOINT, APP_ID).
Mechanism:
  - buildHeader: creates {event, app_id, instance_id (random UUID per event), uid (sessionID optional), e_ts (now)}.
  - postEvents: batch-posts envelopes {H: header, B: body} as JSON, 5s timeout, silent-catch errors.
Three-line: Telemetry client. Generates random instance_id per event (no persistent fingerprinting), posts to tracking.miui.com, silently drops on timeout.

metrics/subscriber.ts (metrics hook-up)
Purpose: subscribe to all metrics events, forward to tracking backend.
Exports: subscribe(Effect).
Mechanism:
  - Gated by MIMOCODE_ENABLE_ANALYSIS flag.
  - Subscribes to ModelCall, ToolCall, AgentRequest via Bus.Service.
  - Calls postEvents with buildHeader + event data.
Three-line: Auto-wiring for metrics collection. On startup (if flag enabled), subscribes to three event types and relays to postEvents.

metrics/installation.ts (installation fingerprint)
Purpose: read or create installation_id (stored as file, not sent in wire payload per comment).
Two-line: Persistent instance identifier for replay detection.

metrics/index.ts (module exports)
Purpose: re-export.
One-line: Metrics module facade.

metrics/util.ts (JSON size calculation)
Purpose: jsonByteLength helper.
One-line: Utility for payload sizing.

server/routes/instance/event.ts (instance-level events endpoint)
Purpose: instance-scoped event stream (/instance/{instanceID}/event).
Mechanism: similar to /global/event, but subscribes to Bus.subscribeAll() instead of GlobalBus, stops on InstanceDisposed event.
Three-line: Per-instance SSE stream, filtered to project bus only (no global scope).

server/routes/control/index.ts (control-plane routes)
Purpose: /auth/:providerID (PUT/DELETE), /doc (OpenAPI).
Mechanism:
  - PUT /auth/:providerID: accept Auth.Info JSON, call Auth.Service.set().
  - DELETE /auth/:providerID: call Auth.Service.remove().
  - GET /doc: openAPIRouteHandler, serves live OpenAPI schema.
Three-line: Auth management endpoints. PUT to set credentials, DELETE to clear. OpenAPI spec served live (routes auto-document via hono-openapi).

=== CROSS-REFERENCES ===

- BusEvent (bus-event.ts) is consumed by: SyncEvent, metrics/event.ts, server/routes/*.ts schema generation.
- GlobalBus (global.ts) is published to by: sync/index.ts (emit on run/replay), server/routes/global.ts (emit on dispose/upgrade), team/index.ts (emit on TeamCreated).
- GlobalBus is subscribed from by: server/routes/global.ts (SSE handler).
- SyncEvent (sync/index.ts) is fundamental: every state mutation (workspace create, inbox send, team add) flows through SyncEvent.run() or replay().
- Fence (fence.ts) blocks HTTP response until local EventSequenceTable reaches sync header state (coordination point between remote workspace and client).
- Inbox (inbox.ts) depends on: Bus, Session, ActorRegistry, Database, sessionPromptRef (fork-and-forget wake).
- Team (team/index.ts) depends on: Bus, InstanceState.context for directory resolution.
- Metrics (subscriber.ts) depends on: Bus; forwards to postEvents client.

=== HTTP API SURFACE ===

Global:
  GET  /health                    -> {healthy: true, version}
  GET  /event                     -> SSE stream of bus+sync events + heartbeat
  GET  /config                    -> Config.Info
  PATCH /config                  -> Config.Info
  POST /dispose                   -> true
  POST /upgrade                   -> {success, version|error}

Instance:
  GET  /instance/{id}/event       -> SSE stream of project bus events

Control-Plane:
  PUT  /control/auth/:providerID  -> true
  DELETE /control/auth/:providerID -> true
  GET  /control/doc               -> OpenAPI schema (live)

Workspace (via proxy):
  GET  /__workspace_ws            -> WebSocket tunnel to remote

All responses may carry x-mimocode-sync: {aggregateID: seq} (fence barrier).

=== PORTABLE MECHANISMS FOR ELAI ===

1. **Versioned Event Registry + Projector Pattern** (sync/index.ts, bus/bus-event.ts)
   Mechanism: define(type, version, aggregate, schema) + project(def, func) + init(projectors). Events are versioned (type.version stored, latest-only published). Projectors are typed functions invoked in DB transaction.
   Why steal: ELAI has SSB event log but lacks version migration path and typed projector wiring. This separates schema definition (one file, frozen at init) from projection (per-layer injectable). Migrate: store SyncEvent definitions in a new crates/events/*.rs file, generate projector stubs with thiserror + Arc<dyn Fn>.
   Citation: packages/opencode/src/sync/index.ts:define(), init(), project().

2. **Event Sequence Fencing for Causality** (fence.ts, sync/index.ts)
   Mechanism: before HTTP response, compare EventSequenceTable before/after, attach x-mimocode-sync: {aggregateID: seq} header. Client-side polls until local seq reaches barrier.
   Why steal: ELAI's orchestrator has no causal coordination between local/cloud (H-50 identifies this as wire-dark context-lifecycle). Fence is zero-copy linearization: no explicit wait channels, just state comparison + header.
   Migrate: add FenceState::get_deltas() -> Vec<(AggregateID, Seq)>, attach to each SSB published event, dashbaord polls until local SSB index reaches fence.
   Citation: packages/opencode/src/server/fence.ts:load(), diff(), FenceMiddleware.

3. **Fork-and-Forget Task Awakening** (inbox/inbox.ts)
   Mechanism: send() forks sessionPromptRef.loop() in service scope (survives sender lifecycle), waits for GC'd scope finalizer to clean up on session end.
   Why steal: ELAI's prompt-engineer agents are dispatched synchronously from orchestrate; no background wake on async inputs. Inbox's fork-in-scope pattern allows senders to go async without blocking.
   Migrate: wrap pending-input checks in an Effect.forkIn(sessionScope) at RuntimeOrchestrator init, scope auto-cancels at session end.
   Citation: packages/opencode/src/inbox/inbox.ts:send() lines 104-117.

4. **Synthetic Message Composition from Side-Channel Data** (inbox/inbox.ts, inbox/render.ts)
   Mechanism: drain() reads inbox rows, finds last real assistant message (to inherit model/agent), creates synthetic user message with formatted row content, inserts parts, deletes inbox rows.
   Why steal: ELAI's memory-writer has no way to inject context from background tasks (e.g., auditor findings, metrics). Synthetic message pattern is crash-tolerant (rows re-render on failure) and preserves model attribution.
   Migrate: add MemorySyntheticMessage variant to MessageInfo enum, wire memory-writer to call Inbox.send({content: context_summary, ...}), main orchestrator.drain() loop reads on transition.
   Citation: packages/opencode/src/inbox/inbox.ts:drain() lines 122-206.

5. **Idempotent Event Replay with Sequence Integrity** (sync/index.ts)
   Mechanism: replay(event) checks event.seq > EventSequenceTable.seq for aggregate; replayAll(events) validates seq continuity and same aggregate. Projector is called once per unique seq.
   Why steal: ELAI's SSB has no recovery path if producer-subscriber window mismatches (high-water marks exist but are not enforced at write). Sequence integrity gates prevent double-application and frame losses.
   Migrate: audit all SSB producers/consumers, add seq continuity check at subscriber init, surface missing-seq as BLOCK (not skip).
   Citation: packages/opencode/src/sync/index.ts:replay() lines 169-194, replayAll() lines 196-213.

6. **Typed Metrics Events with Session Routing** (metrics/event.ts, metrics/subscriber.ts)
   Mechanism: define ModelCall, ToolCall, AgentRequest as BusEvent schemas (with sessionID field), auto-subscribe via Bus and post to remote endpoint. Silent-catch network errors, random instance_id per event (no fingerprinting).
   Why steal: ELAI has SSB logging but no standard telemetry schema or forwarding to external collectors. Session-routed metrics allow filtering by agent/phase/task_type.
   Migrate: define E-N metrics events (model_choice, tool_selection, orchestrator_transition) as SyncEvent definitions, subscriber publishes to local /metrics endpoint, dashboard backfills from SSB.
   Citation: packages/opencode/src/metrics/event.ts, subscriber.ts.

7. **Adaptor Pattern for Runtime Polymorphism** (server/adapter.ts, control-plane/adaptors/worktree.ts)
   Mechanism: Runtime trait (upgradeWebSocket, listen). Hono app is runtime-agnostic; adaptor.bun.ts and adaptor.node.ts provide concrete implementations at startup.
   Why steal: ELAI's orchestrator couples to specific runtime (tokio + Tauri). Adaptor pattern allows test mocking and cross-platform simulation.
   Migrate: refactor orchestrator runtime to Runtime trait with cloud (http client), local (std::process), and mock (in-memory) adaptors.
   Citation: packages/opencode/src/server/adapter.ts, adapter.bun.ts, adapter.node.ts.

8. **Middleware-Based Post-Hook State Tracking** (fence.ts::FenceMiddleware)
   Mechanism: middleware captures EventSequenceTable state before/after request, computes diff, sets response header. No explicit transaction boundaries (state is recorded naturally by run()).
   Why steal: ELAI's orchestrator has no audit trail of which SSB mutations occurred per request. Fence middleware is zero-overhead (one SELECT before, one after, diff is O(k) where k = changed aggregates).
   Migrate: add AuditMiddleware to dashboard RPC server, capture crates/orchestrator/src/ssb::Table before/after, attach to response as X-SSB-Changes header for replay verification.
   Citation: packages/opencode/src/server/fence.ts:FenceMiddleware lines 68-81.

=== GROUNDING / VERIFICATION ===

All mechanisms verified by reading source:
  - Event registry: BusEvent.define/payloads, SyncEvent.init/define/project verified working (storage.sqlite.ts confirmed).
  - Fence: EventSequenceTable confirmed, Workspace.waitForSync confirmed (control-plane/workspace.ts).
  - Fork-and-forget: Effect.forkIn + scope confirmed (inbox.ts:104-117).
  - Synthetic messages: drain() implementation verified (inbox.ts:122-206), crash tolerance documented (line 169-174).
  - Replay: seq checks verified (sync/index.ts:169-194).
  - Metrics: Bus.subscribeCallback verified, postEvents confirmed (metrics/client.ts:33-40).
  - Adaptor: Runtime trait + node/bun dual impl confirmed (adapter.ts, adapter.node.ts, adapter.bun.ts).
  - Middleware: Hono middleware pattern confirmed (fence.ts:68, LoggerMiddleware, ErrorMiddleware).

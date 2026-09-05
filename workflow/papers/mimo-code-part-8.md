MIMOCODE RUNTIME: AGENT SPAWN & LSP INTEGRATION (PART 8)

===== DIRECTORY SUMMARIES =====

agent/
  Purpose: Agent definitions, configuration, and factory for the multi-agent runtime.
  Key exports: Agent.Info (schema), Agent.Service (Effect context), layer, generate()
  Mechanism: Agent.Info carries zod schema defining subagents (build, plan, compose, general, explore, title, summary, checkpoint-writer, dream, distill). Each agent has: name, description, mode (primary|subagent), permission ruleset, toolAllowlist, optional hardcoded prompt. Nested 7-agent config merges defaults (read/write on whitelisted dirs, deny plan_enter/exit) + user overrides from config + per-agent hardcoded rules. Permission.merge chains allow compositional grant+deny. Special: checkpoint-writer has no prompt field — fork agent contract. Dream/distill agents have hardcoded PROMPT_DREAM/PROMPT_DISTILL (read from ./prompt/*.txt files). SYSTEM_SPAWNED_AGENT_TYPES = [checkpoint-writer, dream, distill] exempted from prune/bootstrap scans.
  Cross-refs: permission/, config/; loaded by spawn.ts

actor/ (spawn.ts, registry.ts, events.ts, turn.ts, schema.ts, spawn-ref.ts, return-header.ts, waiter.ts)
  Purpose: Actor/spawn model — the machinery for launching and supervising subagents as concurrent work fibers.
  Key exports: Service (spawn, cancel, getForkContext), SpawnInput/SpawnResult/AgentOutcome (types), ForkContext (frozen snapshot)
  Mechanism: 
    - spawn(SpawnInput) dispatches either spawnPeer (new session, full context, mode="peer") or spawnSubagent (same session, mode="subagent"). Both register actorID in ActorRegistry (DB + bus event) then fork forkWork fiber into a scope.
    - forkWork is the main loop: runAgentLoop invokes SessionPrompt.prompt (the LLM call, tagged with actorID so messages.agent_id persists). After LLM returns finalText, preStop hook loop checks plugin.triggerActorPreStop(MAX_PRE_REACT=3 times). If a plugin returns {continue:true, reason:string}, that reason becomes the next task, reentryDecision.contributingPlugins recorded, bus.publish(ReActReentered). Iteration capped at MAX_PRE_REACT=3; if exceed, log WARN and break.
    - postStop loop: COMPLETION GATE (B) runs TaskGate.decide() to reconcile model's self-reported status vs actual DB task state. If DB has non-terminal tasks, gate downgrades status to "partial". Structured output (json_schema result) takes precedence over finalText. If gate needs reentry, runs another turn via runTurn. Then postStop hook loop (MAX_POST_REACT=3) runs plugin.triggerActorPostStop(); unlike preStop, post runs AFTER outcome is resolved, so text updates don't propagate upstream.
    - Outcome: AgentOutcome.success carries {finalText, structured, reportedStatus, reportedSummary, incompleteTasks}. Failure/cancelled also possible.
    - ForkContext: snapshot of parent's {system[], tools{}, parentPermission, inheritedMessages[], watermarkMsgID} captured at fork spawn time. Used by checkpoint-writer fork to cache prompt prefix (see 2026-05-26-fork-agent-prefix-cache-design.md).
    - ActorRegistry.updateStatus persists actor state to DB (pending→running→idle, lastOutcome, lastError, turn counts). Background actors get inbox.send() notification on completion.
  Cross-refs: session/prompt.ts (SessionPrompt.prompt), task/gate.ts (TaskGate.decide, completion gate), plugin.ts (ReAct hook dispatch)

lsp/ (lsp.ts, client.ts, server.ts, diagnostic.ts, language.ts, launch.ts, index.ts)
  Purpose: LSP client integration — connect to language servers (Rust, TS, etc.) for diagnostics, symbols, document features.
  Key exports: LSP.Service (status(), symbols(), document symbols, diagnostics), Range, Symbol, DocumentSymbol (zod schemas)
  Mechanism: 
    - LSP.Client wraps language server via stdio/HTTP transports. Initialized with root workspace dir via launch.ts spawn (fork_language_server.mjs, language server binary).
    - Diagnostics fetch after file edits; LSP reports ERROR/WARN/INFO/HINT at (line, col). diagnostic.ts pretty-prints with severity label and truncates to MAX_PER_FILE=20 errors per file.
    - Symbol queries return DocumentSymbol[] (kind int per SymbolKind enum: Function=12, Class=5, etc.). Clients use symbols for jump-to-def, outline.
    - Status enum: connected | error. Reconnect logic in launch.ts on startup or on socket close.
  Cross-refs: Used by the IDE UI to drive live diagnostics on save. Diagnostics flow to the agent via tool output (not directly agent-facing, but influences re-plan).

mcp/ (index.ts, auth.ts, oauth-provider.ts, oauth-callback.ts)
  Purpose: MCP (Model Context Protocol) client — expose tool/prompt/resource servers (local stdio or remote HTTP(S)) to the agent.
  Key exports: MCP.Service (status, tools, prompts, resources, connect, disconnect, auth flows), Resource, Status (zod discriminated union)
  Mechanism:
    - Startup: config.mcp{} defines servers as local (command-based) or remote (HTTP URL). create() dispatches connectLocal (StdioClientTransport) or connectRemote (StreamableHTTP + fallback SSE). Timeout: mcp.timeout ?? 30s default.
    - Tool discovery: MCPClient.listTools() returns MCPToolDef[]. convertMcpTool() wraps each into ai SDK Tool (dynamicTool with json_schema input, execute: client.callTool(args)). Tools list cached in State.defs per server. watch() listens for ToolListChangedNotification, re-fetches defs, publishes MCP.ToolsChanged bus event.
    - Auth: OAuth flow for remote servers. startAuth() initializes authProvider (McpOAuthProvider), fires client.connect() which throws UnauthorizedError → pendingOAuthTransports.set(name, transport). Browser opens auth URL. Callback handler (McpOAuthCallback) waits for code, finishAuth() calls transport.finishAuth(code). Supports PKCE + custom redirectUri if configured.
    - Resources/Prompts: collectFromConnected() iterates connected clients, calls client.listResources()/listPrompts(), sanitizes names (alphanumeric+_-), merges into one dict keyed "clientname:resourcename".
    - Error handling: UnauthorizedError → needs_auth. Connection failures → {status:"failed", error:msg}. Disabled entries → {status:"disabled"}.
  Cross-refs: config/mcp.ts (config schema), plugin system (tools registered into tool registry)

effect/ (runtime.ts, bootstrap-runtime.ts, bridge.ts, instance-ref.ts, instance-registry.ts, instance-state.ts, logger.ts, memo-map.ts, observability.ts, run-service.ts, runner.ts, app-runtime.ts)
  Purpose: Effect.ts runtime layering and Effect-based resource management (singleton state machines, scoped cleanup, memoization).
  Key exports: ManagedRuntime (wrapper: runSync, runPromise, runFork), BootstrapLayer, makeRuntime(), InstanceState.make
  Mechanism:
    - Effect is a lazy, composable async/error/dependency-injection library. Layer.mergeAll composes services (each a Context.Service<I,S>). ManagedRuntime.make(layer, {memoMap}) creates a singleton runtime.
    - InstanceState: wrapper around in-memory mutable state with Effect semantics. InstanceState.make<State>(initEffect) returns a Effect-wrapped state handle. InstanceState.use/get/useEffect(state, fn) reads/mutates within Effect context. Used for singleton state (Config, Agent list, MCP client map, etc.) that persists across Effect.fn invocations.
    - BootstrapLayer: merges Config, Plugin, LSP, File, Vcs, Bus, Memory, History layers. Every service pulls from this when initialized.
    - Observability.layer: wires OtelTracer if experimental.openTelemetry enabled.
    - makeRuntime<I,S,E>: generic wrapper returning {runSync, runPromise, runFork, runCallback} bound to service S and layer.
  Cross-refs: All services (Agent, Session, MCP, etc.) use Effect.gen + yield* pattern. Layer.suspend used for lazy circular-dependency breaking.

file/ (index.ts, ignore.ts, protected.ts, ripgrep.ts, watcher.ts)
  Purpose: File operations — read/write with budgets, gitignore-aware discovery, ripgrep search, file watch.
  Key exports: File.Service (read, write, list, diff), Content (zod with patch hunks), Info (added/removed/status)
  Mechanism:
    - read(path): returns {type:"text"|"binary", content, encoding?, mimeType?, diff?, patch?}. Binary files base64-encoded. Diff via diff library (formatPatch, structuredPatch) if file changed in git.
    - write(path, content, opts): Truncate.newFile() guards against budget overruns (TRUNCATE.GLOB points to managed memory zone). Protected.notWritablePatterns blocks /etc, /.git, /.venv, etc.
    - list(dir, opts): Fuzzy-sort by name. Ripgrep queries for fast regex search. Gitignore respected (ignore library, .gitignore rules loaded).
    - Watcher: FileWatcher.Service watches dirs, emits File.Edited bus events on change.
  Cross-refs: Used by agents for read/write/search tools. Protected.ts blocks system files.

format/ (formatter.ts, index.ts)
  Purpose: Code formatter plugin registry — gofmt, mix, prettier, rustfmt, etc.
  Key exports: Formatter.Info, list of enabled formatters
  Mechanism: Each formatter (prettier, gofmt, etc.) is a Info { name, extensions[], enabled(context):Promise<cmd[]|false> }. enabled() checks if binary exists (which()) and if project has config (e.g., prettier in package.json). On file save, agent calls formatter with $FILE substitution. Prettier gets BUN_BE_BUN=1 env var.
  Cross-refs: Called by build agent post-edit.

installation/ (index.ts, version.ts)
  Purpose: Self-update mechanism — detect install method (npm/bun/pnpm/curl/brew), fetch latest version from registry, upgrade.
  Key exports: Installation.Service (info, method, latest, upgrade)
  Mechanism: methodImpl() checks process.execPath for package manager signature. latestImpl() queries npm registry (or brew/choco/scoop if available) for latest version of @mimo-ai/cli. upgradeImpl(method, target) spawns "npm install -g @mimo-ai/cli@target" (or equivalent). USER_AGENT header identifies client version/channel/installation method for telemetry.
  Cross-refs: CLI entry point queries info() on startup to warn about available updates.

pty/ (pty.ts, pty.bun.ts, pty.node.ts, schema.ts)
  Purpose: Pseudo-terminal abstraction — spawn shell sessions with stdin/stdout/stderr, resize, kill.
  Key exports: Proc { pid, onData, onExit, write, resize, kill }
  Mechanism: pty.ts exports type-only Proc interface. pty.bun.ts uses Bun.spawn() with pty:true. pty.node.ts uses node-pty (native module) or pty.spawn fallback. Agent can fork a shell session, write commands, listen for output events. Resize events sent when terminal dims change.
  Cross-refs: Used by shell agent for interactive command execution.

npm/ (index.ts, config.ts)
  Purpose: NPM package management — install, check for updates, resolve entry points.
  Key exports: Npm.Service (add, install, outdated, which)
  Mechanism:
    - add(pkg): installs package into Global.Path.cache/packages/sanitize(pkg), returns {directory, entrypoint:Option}. Sanitize removes illegal chars (< > : " | ? * on Windows).
    - which(pkg): uses import.meta.resolve to find installed entry point.
    - outdated(pkg, cachedVersion): compares against npm registry.
    - install(dir, {add:[{name,version?}]}): reify dependency tree via arborist (npm's dep resolver). Effect-wrapped with flock for concurrency safety.
  Cross-refs: Called by agent to add missing dependencies; Agent.generate() uses provider.getModel() which may fetch model metadata from npm.

shell/ (shell.ts)
  Purpose: Shell selection and process tree kill.
  Key exports: preferred(), acceptable(), killTree(), posix(), login()
  Mechanism: preferred() returns $SHELL or detects (pwsh → powershell → zsh/bash/sh based on OS). acceptable() filters blacklist (fish, nu — unsupported). killTree() kills process group: Windows taskkill /f /t; Unix SIGTERM + sleep + SIGKILL. Used by PTY and subagent teardown.
  Cross-refs: Called by installation/, pty/, and spawn cleanup.

id/ (id.ts)
  Purpose: Monotonic ID generation — prefix_ts_randombase62, ascending/descending, extractable timestamp.
  Key exports: ascending(), descending(), timestamp(), schema()
  Mechanism: Timestamp stored in first 6 bytes (12 hex chars), randomBytes fill remaining 14 chars. Counter incremented per millisecond to enforce monotonicity. Descending: bitwise ~now for reverse sort. timestamp() reverses the encoding to extract Date.now() from ID string. Used for session, message, event IDs.
  Cross-refs: All IDs use this (SessionID, MessageID, EventID).

ide/ (index.ts)
  Purpose: IDE detection and extension installation.
  Key exports: ide() → IDE name, install(ide) → run code --install-extension, alreadyInstalled()
  Mechanism: Detects VS Code/Cursor/Windsurf via GIT_ASKPASS env var or TERM_PROGRAM=vscode. install() spawns "code --install-extension sst-dev.opencode" (or equivalent). Throws AlreadyInstalledError or InstallFailedError on failure.
  Cross-refs: Called by onboarding flow.

global/ (index.ts)
  Purpose: Global paths and cache versioning.
  Key exports: Global.Path { home, data, cache, config, state, log, bin }
  Mechanism: Resolves MIMOCODE_HOME env var or ~/.mimocode via resolveMimocodeHome(). Initializes all dirs on module load. Cache versioned (CACHE_VERSION=21); cache clear on version bump. Flock.setGlobal({state}) for global process locks.
  Cross-refs: Used everywhere for storing config, cache, logs.

env/ (index.ts)
  Purpose: Session-scoped environment variable management.
  Key exports: Env.Service (get, all, set, remove)
  Mechanism: InstanceState.make wraps in-memory env dict (copy of process.env at init). Mutations stay in-memory, don't affect process.env. Used by agents to set variables for subcommands without affecting the main process.
  Cross-refs: Called by shell agent before spawning subshell.

===== PORTABLE MECHANISMS FOR RUST ORCHESTRATOR =====

P1: ForkContext Prefix Cache (agent/agent.ts:285-313 + spawn.ts:56-92)
  The checkpoint-writer fork agent caches the parent's full system prompt + tool schema + inherited messages at spawn time, stored in ForkContext. Runtime tool restriction is enforced via actor.tools whitelist (set by spawn, passed to permission check), NOT by re-parsing. This avoids re-serializing the LLM prefix on every fork spawn and aligns prompt-cache boundary with parent's tool visibility.
  Steal: Capture the full prompt prefix (system roles, tools, watermark) into a frozen struct at spawn time. Store in an in-process map keyed by actorID. Fork agents read from that snapshot instead of recomputing. Use permission.ruleset from parent (ForkContext.parentPermission) + actor.tools whitelist to filter tools at runtime — don't re-apply agent.permission on fork.

P2: ReAct Hook Loop with Iteration Cap (spawn.ts:316-388, 481-520)
  preStop hook runs BEFORE outcome is written, up to MAX_PRE_REACT=3 times. Each iteration runs turn, then plugin.triggerActorPreStop({finalText, task, lifecycle, iteration}). If plugin returns {continue:true, reason:string}, that reason becomes the new task (provenance recorded). postStop hook runs AFTER outcome written, doesn't propagate text back. This decouples in-flight re-planning from completion.
  Steal: Implement plugin.triggerPreActorStop and triggerPostActorStop hooks with iteration caps. preStop can mutate the task; postStop cannot (outcome frozen). Log ReActReentered bus events so the UI knows why an actor re-ran.

P3: Completion Gate with Task Reconciliation (spawn.ts:402-437, task/gate.ts not shown)
  After agent finalText returns, TaskGate.decide() checks the DB for non-terminal tasks owned by the actor. If open/in_progress tasks remain, gate downgrades status: {finalText:"...", reportedStatus:"partial", incompleteTasks:[ids...]}. Structured output (json_schema) takes precedence over finalText. Gate re-entry capped at MAX_TASK_GATE_SUBAGENT_REACT. This ensures the agent's self-reported "success" can be overridden by the DB truth.
  Steal: After subagent LLM call returns, check a task registry for non-terminal tasks bound to that actor. If found, either: (a) rerun the agent once (with reentryText asking to finish the task), or (b) downgrade status to "partial" and list the incomplete task IDs in the outcome. Make missing evidence a FAIL (task list query failure = BLOCK).

P4: Structured Output (json_schema) Precedence (spawn.ts:229-236, 402-475)
  If spawn(format:OutputFormat) is set, SessionPrompt.prompt requests structured output (json_schema). The validated object is surfaced on AgentOutcome.structured and takes precedence over finalText for delivery. Notification body uses JSON stringified structured, avoiding duplicating text + object. Delivery text shown to parent agent is:
    structured !== undefined ? JSON.stringify(structured) : (reconciledText ?? "(no output)")
  Steal: When spawning a subagent with format (zod schema), force the LLM to return json_schema, validate it, and deliver the object directly. Skip finalText when structured is present.

P5: ActorRegistry DB + Event Publishing (actor/registry.ts, events.ts)
  Each spawn registers actorID in ActorRegistryTable (schema: session_id, actor_id, mode, agent, status, last_turn_time, turn_count, last_error, lifecycle, context_mode, context_watermark). updateStatus() writes status changes + outcome. Bus.publish(ActorRegistered, {sessionID, actorID, agent, mode}) on register; updateStatus() publishes StatusChanged event. Queries: listBySession(), listActive(), listByParent(). renderForAgent() serializes actor tree for agent introspection.
  Steal: Persist actor spawn/status in a DB table. Publish bus events on register/status-change so plugins and UI can react. Include lifecycle (ephemeral|persistent), context_watermark (for full-context mode), and turn_count for diagnostics. Render actor tree as a string for introspection.

P6: Permission Merge Chain (agent/agent.ts:118-125, 156-175, etc.)
  Each agent Info carries a Ruleset. Spawn merges: defaults (broad allows + whitelisted dirs) + user config overrides + agent-specific rules. Permission.merge(defaults, user, agent) applies rules in order; later rules can refine. Example: build agent adds {question:"allow", plan_enter:"allow"} on top of defaults. Permission.disabled(["write"], ruleset).has("write") checks if write tool is completely blocked. This compositional approach avoids permission explosion (no giant agent-specific lists).
  Steal: Define a permission algebra: Ruleset = rule[]. Each rule is {pattern, action}. Merge operator: later rules override earlier. For each tool invocation, check Permission.disabled(toolName, ruleset) to decide whether to ask/allow/deny.

P7: InstanceState for Singleton Mutable State (effect/instance-state.ts not shown; used everywhere)
  Agent.Service stores agents list in InstanceState.make<State>(initEffect). initEffect builds the dict once. InstanceState.use(state, (agents) => agents[name]) reads synchronously. InstanceState.get(state) reads within an Effect. Mutations stay in-memory. This avoids re-reading config on every agent lookup and provides a single source of truth within a session.
  Steal: For singleton state (config, agent list, MCP clients, actor registry), wrap in InstanceState so mutations are visible to all consumers within the session context, without touching the DB on every read.

P8: Bus Event Publishing for UI Observability (actor/spawn.ts:345, 374, 510, 522)
  When ReAct re-enters, publish bus.publish(HookEvent.ReActReentered, {phase, actorID, agentType, iteration, triggeredByPlugins, reasonPreview}). When MAX_PRE_REACT exceeded, publish HookEvent.ReActMaxReached. When actor completes, inbox.send() notification if background. This gives the UI event-driven updates instead of polling. Bus is a simple PubSub (no persistence); plugins subscribe.
  Steal: Define bus events for key actor lifecycle moments (spawn, reentry, completion, error). Publish with metadata (actor ID, phase, reason preview). UI and logging systems subscribe. Events include actor ID for tracing.

===== END PORTABLE MECHANISMS =====

Synthesized from:
  - packages/opencode/src/agent/agent.ts (agent registry, permission merging)
  - packages/opencode/src/actor/spawn.ts (spawn/fork machinery, ReAct loops, completion gate)
  - packages/opencode/src/actor/registry.ts (DB persistence, event publishing)
  - packages/opencode/src/lsp/lsp.ts (LSP client integration)
  - packages/opencode/src/mcp/index.ts (MCP tool/resource discovery, auth, caching)
  - packages/opencode/src/effect/ (runtime layering, InstanceState pattern)
  - packages/opencode/src/{file,format,installation,pty,npm,shell,id,ide,global,env}/ (supporting services)

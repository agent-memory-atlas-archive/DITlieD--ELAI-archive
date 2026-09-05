Goose Agent Partition 5: agents/ — Agent Loop and Orchestration
================================================================

## FILE MAP (44 files, ~222K tokens)

Core Agent Loop:
- agent.rs (2100+ lines): Main Agent struct, reply() entry point, reply_internal() turn loop
  - Type: Agent at 301; ReplyContext at 133; ToolCategorizeResult at 143
  - Loop: lines 1723-2300+ (async_stream::try_stream! outer loop, max_turns check at 1814, stream_response_from_provider at 1830)
  - Tool dispatch: lines 1869-2050, categorize_tools() at line 717, frontend/remaining split at 1883-1948
  - Stop hook: emit_blocking at 1781-1798, denial handling at 1790-1803
  - Metrics tracking: update_session_metrics() calls at 1618, 1879, 2204
  - Cancellation: is_token_cancelled() check at 1741, 1870

- reply_parts.rs (550+ lines): Stream and provider interaction
  - stream_response_from_provider() at 252 (receives Arc<dyn Provider>, returns MessageStream)
  - Toolshim mode: lines 310-348 (accumulate full response before tool-use stripping)
  - update_session_metrics() at 499 (ProviderUsage -> session token accounting)
  - Tool argument coercion: coerce_tool_arguments() at 101

Tool Dispatch & Execution:
- tool_execution.rs: ToolCallContext at 17, ToolCallResult at 43, handle_approval_tool_requests() at 77
- tool_confirmation_router.rs (private): Routes user confirmations back to pending tool calls

Extension/Tool Management:
- extension_manager.rs (800+ lines): ExtensionManager at 139
  - Extensions map (name -> Extension struct) at 140
  - Tools cache with version tracking (AtomicU64) at 143-144
  - load_extensions() spawns MCP clients (TokioChildProcess / StreamableHttpClientTransport)
  - Tool list caching: tools_cache, tools_cache_version atomic coordination
  - Malware check: calls extension_malware_check on new extensions

- extension.rs (private): ExtensionConfig enum, ToolInfo, ExtensionResult
- extension_malware_check.rs: SecurityInspector validation before client init

MCP & Platform Integration:
- mcp_client.rs (200+ lines): GooseMcpClientTrait, call_tool() dispatch to MCP servers
  - Async call propagation with notification_stream option
- platform_extensions/: code_execution, chatrecall, apps.rs, developer/ (edit/commands)
- platform_tools.rs: Schedule, FinalOutput, ManageExtensions, ToolConfirmationRouter tools

Subagent Execution:
- subagent_handler.rs: run_subagent_task() at 48
  - Creates new Agent::with_config() per subagent at 141
  - Sets provider + extensions on subagent at 144-156
  - Calls agent.reply() on subagent in isolated session context at 183-189
  - Collects conversation messages and final output at 191-224
  - No explicit parallel batching (sequential message collection)

- subagent_task_config.rs: TaskConfig struct (provider, extensions, max_turns)

Utility & Orchestration:
- container.rs: Container struct for docker-exec namespace
- moim.rs: Context injection (MOIM = "moment-of-input-meaning") before provider calls at 1823
- retry.rs: RetryManager for handling transient failures
- execute_commands.rs: /clear, /compact, special command routing
- prompt_manager.rs: System prompt override and dynamic injection
- final_output_tool.rs: FinalOutputTool for graceful completion
- large_response_handler.rs: Response size limits

Validation & Security:
- validate_extensions.rs: Runtime extension security checks
- types.rs: SessionConfig, SuccessCheck, RetryConfig

Streaming & IO:
- stream handling: BoxStream, async_stream::try_stream! blocks throughout

---

## AGENT LOOP SHAPE

Entry: Agent::reply() at agent.rs:1439

1. **User message pre-processing (1450-1560)**
   - Handle action-required (elicitation) responses
   - Emit UserPromptSubmit hook
   - Execute special commands (/clear, /compact) via execute_command()
   - Store message in session

2. **Context preparation (1561-1671)**
   - Load session + conversation
   - Check compaction threshold
   - Call prepare_reply_context() -> system_prompt, tools, toolshim_tools
   - Load project instructions if present (1669)
   - Fetch provider + model metadata (1675-1687)

3. **Main turn loop (1723-2300+)** — `async_stream::try_stream!` block
   - Max turns check (1814)
   - Drain pending steers (1745-1764)
   - Check final_output signal for FinalOutputTool (1767-1807)
   - Emit Stop hook, block on denial, respect cap (1778-1806)
   
4. **Provider interaction per turn (1823-1865)**
   - Inject MOIM context (1823-1828)
   - Call stream_response_from_provider() (1830-1837)
     - Opens streaming connection to provider (reply_parts.rs:286)
     - Yields (msg, usage) tuples
     - Toolshim mode: accumulates full response, strips tool-use markers, posts (310-348)
     - Normal mode: streams messages as received
   - Accumulate usage metrics (1878)
   
5. **Tool dispatch per message chunk (1869-2050)**
   - Yield message to UI (1916)
   - Categorize tool requests: frontend vs remaining (1888)
   - Frontend tools: execute immediately in handle_frontend_tool_request() (1939)
   - Remaining tools (approval-required): route via tool_confirmation_router (1949)
   - Collect responses in request_to_response_map (1929-1934)
   - Tool execution: ToolCallResult futures with optional notification stream (tool_execution.rs:43)
   - Propagate tool results back to conversation (2050+)

6. **Streaming message collection (1869-2100)**
   - While-let loop over stream.next().await
   - Break on cancellation (1870)
   - Match (response, usage):
     - Unpack inference metadata (1895-1904)
     - Yield filtered_response to UI (1916)
     - Categorize tool requests (1888)
     - Execute frontend requests synchronously (1939)
     - Queue remaining for approval (1949)
     - Add text-only messages to messages_to_add (1925)

7. **Retry and stopping conditions (2100-2250+)**
   - Handle RetryManager logic (607-628)
   - Check goal/grind/stop signals
   - Emit post-tool hooks (with_post_tool_hook at 497)
   - Break on max_turns, Stop hook allow, or FinalOutputTool signal

8. **Post-loop: compaction and history**
   - If needs_auto_compact: call compact_messages() (1608-1639)
   - Replace conversation (1617)
   - Yield HistoryReplaced event

**Interruption/Cancellation**: CancellationToken checked at 1741, 1870; cancellation_token passed through reply_internal at 1654

**Retries**: RetryManager at 361; handle_retry_logic() at 607; incremented per turn, reset at session start (1673)

---

## MULTI-AGENT ORCHESTRATION

**Subagent Spawning** (subagent_handler.rs:48, run_subagent_task())

- Called when: tool calls subagent_execution_tool, or MCP servers request subagent dispatch
- Execution model: **Sequential, in-band**
  - Create new Agent::with_config(config) per subagent (line 141)
  - Set provider on subagent via update_provider() (144)
  - Load extensions into subagent (148-156)
  - Apply recipe.response schema if present (159)
  - Build subagent system prompt via build_subagent_prompt() (163-164)
  - Create isolated session context (Session ID unique)
  - Call agent.reply() on subagent message (183-189)
  - Collect all messages + final_output synchronously (191-224)

- **Context inheritance**:
  - TaskConfig carries: provider, extensions[], max_turns, recipe (system_instructions, prompt, response, retry, activities)
  - Subagent sees: system_instructions as-is, user_task from recipe.prompt
  - No explicit context passing (conversation history NOT inherited)
  - Each subagent runs isolated session

- **Return path**:
  - Subagent message stream collected into Conversation
  - final_output extracted (222)
  - extract_response_text() collects all text content (65) or return_last_only (79)
  - Result returned as tool response to parent agent

- **Topology**: **Delegation (parent -> subagent, one-way)**
  - Not peer-to-peer, not broadcast
  - No subagent-to-subagent communication
  - Parent waits for subagent completion before continuing (blocking async)
  - No parallelism cap (subagent runs in parent's task context)

- **Notification callback** (subagent_handler.rs:193-207):
  - Optional on_message callback fired per subagent message (194)
  - Optional notification_tx for ServerNotifications (197-207)
  - Create_tool_notification() converts MessageContent to notification (199)

**Inter-agent communication**:
- No direct IPC between agents
- Agent 1 calls subagent_execution_tool with task config
- Tool execution returns final_output string
- Parent agent continues with tool result

---

## MODEL SELECTION

**Provider ownership**:
- Agent.provider: Arc<Mutex<Option<Arc<dyn Provider>>>> (lines 315-316)
- Fetched via provider() at 795
- Set via update_provider() / create_with_working_dir()

**Model configuration flow** (reply_internal 1675-1687):
```
let provider = self.provider().await?;
let provider_name = provider.get_name();                    // e.g. "anthropic"
let requested_model = provider.get_model_config().model_name;  // e.g. "claude-3-5-sonnet"
let inference = provider
    .fetch_model_info(&requested_model)
    .await
    .ok()
    .and_then(|model_info| model_info.resolved_model)       // Provider resolves aliases
    .map(|resolved_model| InferenceMetadata {
        provider,
        requested_model,
        resolved_model: Some(resolved_model),
    });
```
- InferenceMetadata attached to every message (1900-1903) for session tracking

**No per-role or lead/worker split** in agents/:
- Agent.provider is singular, global for the session
- Subagent receives same provider config as parent (subagent_handler.rs:144)
- Subagent can call update_provider() to override (not done by default)

**Fallback logic** (elsewhere in codebase, referenced at agent.rs line 814-817):
- If provider errors, fallback_provider_name from config
- ModelConfig::new() resolves model name to canonical name
- with_canonical_limits() sets context limits per provider

**Toolshim mode** (reply_parts.rs:269):
- If provider.get_model_config().toolshim == true:
  - Convert tool messages to text for non-tool-aware providers
  - Accumulate full response before stripping tool-use markers
- Else: pass tools directly to provider.stream()

---

## USAGE FLOW

**ProviderUsage tracking** (reply_parts.rs:499-550, agent.rs:1879)

1. **Capture at stream**:
   - stream_response_from_provider() yields (msg, usage) tuples
   - usage = Option<ProviderUsage> from provider response

2. **Accumulate per turn** (agent.rs:1878):
   ```
   self.update_session_metrics(&session_config.id, 
                               session_config.schedule_id.clone(), 
                               usage, 
                               false)  // false = not compaction
   ```

3. **Metrics update** (reply_parts.rs:499-550):
   - Get current session
   - Accumulate into: accumulated_total_tokens, accumulated_input_tokens, accumulated_output_tokens
   - Accumulate cost via accumulate_cost() (provider-specific pricing)
   - Write back to session via session_manager.update()

4. **Special: compaction usage** (agent.rs:1608-1618):
   - compact_messages() called on auto-trigger (1608)
   - Passes is_compaction_usage = true to update_session_metrics() (1618)
   - Compaction summary output becomes new input context (reply_parts.rs:531)

5. **UI notification**:
   - Usage not explicitly pushed to UI in agents/ (metric updates are session-local)
   - Schedule metrics accessible via session.accumulated_* fields (dashboard query)
   - Usage snapshots in Message.inference metadata (1900-1903)

6. **No explicit user notification**:
   - Usage is side-loaded into session row, not streamed as separate event
   - UI polls session endpoint or subscribes to session changes

---

## PORTABLE IDEAS

### 1. Cancellation Token Architecture (agent.rs:1439-1742)
**Evidence**: CancellationToken? passed from entry, checked before every stream iteration
**Pattern**: 
- Accept Optional<CancellationToken> at entry (1441)
- Check is_token_cancelled(&token) in outer loop (1741) and inner loop (1870)
- Immediate break on cancellation, no cleanup required (Rust cleanup is automatic)
**Portable**: Clone the approach for any streaming agent loop. Avoid try-catch style cancellation; let the token drive control flow.

### 2. Async Stream Yielding with Type Erasure (agent.rs:1723, reply_parts.rs:309)
**Evidence**: `async_stream::try_stream! { yield ... }` blocks wrapping entire agent loop
**Pattern**:
- Define return type as BoxStream<'_, Result<AgentEvent>>
- Wrap implementation in `Box::pin(async_stream::try_stream! { ... })`
- Yield intermediate results (messages, events) without buffering full response
- Return error via `?` operator (converted to stream error)
**Portable**: Use for real-time UI streaming. Avoids building entire response in memory. Error propagation is clean.

### 3. Provider Abstraction with Arc<dyn Trait> (agent.rs:795, reply_parts.rs:252)
**Evidence**: `provider: Arc<Mutex<Option<Arc<dyn Provider>>>>`; stream_response_from_provider(provider: Arc<dyn Provider>)
**Pattern**:
- Single provider abstraction (trait Provider with get_model_config(), stream(), fetch_recommended_models())
- Thread-safe via Arc<Mutex<>> at agent level
- Pass Arc<dyn Provider> to streaming functions
- Provider handles model resolution, streaming, error enhancement
**Portable**: Build multiple backends (Anthropic, OpenAI, local sidecar) as implementations of the same trait. Agent code never knows which provider it uses.

### 4. MessageStream with (Message, Usage) Tuples (reply_parts.rs:259, 309-360)
**Evidence**: `type MessageStream = BoxStream<Result<(Option<Message>, Option<ProviderUsage>)>>`
**Pattern**:
- Every provider yields (msg, usage) tuples, not separate streams
- Accumulate usage at agent level in one place (update_session_metrics)
- Toolshim mode: accumulate full message, post-process, then yield (310-348)
- Normal mode: stream messages as received (360-400+)
**Portable**: Unifies token accounting across all providers. Usage is never lost. Allows both streaming and batch-accumulate modes in same agent.

### 5. Tool Request Categorization (agent.rs:1883-1894, 717-790)
**Evidence**: ToolCategorizeResult { frontend_requests, remaining_requests, filtered_response }
**Pattern**:
- Split tool requests by category (frontend vs backend)
- Frontend tools: execute immediately in-process (1939)
- Backend tools: require approval routing (1949)
- Filter response message to remove approved tool calls before UI (1883)
**Portable**: Allows safe execution separation: user-approved tools run async; unapproved tools get blocked or marked. Filters guarantee UI never sees raw tool-call metadata.

### 6. Hook Emission with Context + Matcher (agent.rs:399, 478-495)
**Evidence**: emit_with_matcher(event, session_id, matcher_context, tool_name, tool_input, working_dir)
**Pattern**:
- Every lifecycle event carries: event type, session id, tool context, matcher string, working dir
- HookManager filters by event type + regex on matcher_context (before executing handlers)
- Enables user policies: "run this hook only on shell commands matching `rm.*"`
**Portable**: Build plugin systems with context-aware conditional execution. Matcher allows fine-grained control without hardcoding policies in agent code.

### 7. Session Metrics Accumulation (reply_parts.rs:509-550)
**Evidence**:
```rust
let accumulated_total = accumulate(session.accumulated_total_tokens, usage.usage.total_tokens);
let accumulated_cost = accumulate_cost(usage, provider_name);
```
**Pattern**:
- Accumulate across full session (input_tokens, output_tokens, total_tokens, cost)
- Keep current_turn metrics separate (for UI display of "this turn's cost")
- Per-provider cost calculation (provider.pricing_table * output_tokens)
- Compaction is special: summary output becomes new input context
**Portable**: Token budgeting for long-running agents. Separate turn-local from session-wide metrics. Cost tracking per provider enables routing decisions.

### 8. Subagent Context Isolation (subagent_handler.rs:141-189)
**Evidence**:
```rust
let agent = Arc::new(Agent::with_config(config));
agent.update_provider(task_config.provider.clone(), &session_id).await;
agent.reply(user_message, session_config, cancellation_token).await
```
**Pattern**:
- New Agent instance per subagent (no shared state leak)
- Isolated session ID (session_context::with_session_id)
- Subagent sees: system_instructions, task prompt, assigned provider/extensions
- Subagent does NOT see: parent conversation history
- Optional callbacks (on_message, notification_tx) decouple subagent from parent
**Portable**: Build safe orchestration: each subagent is a fresh agent instance. No context bleed. Parent/child communication is one-way (tool result only).

### 9. Tool Execution with Notification Stream Option (tool_execution.rs:43-46)
**Evidence**: 
```rust
pub struct ToolCallResult {
    pub result: Box<dyn Future<Output = ToolResult<CallToolResult>> + Send + Unpin>,
    pub notification_stream: Option<Box<dyn Stream<Item = ServerNotification> + Send + Unpin>>,
}
```
**Pattern**:
- Tool call is a future + optional notification stream (decoupled)
- Async tool execution: result future is awaited at agent's leisure
- Long-running tools (MCP servers, background jobs): notification stream carries interim updates
- Agent can yield notifications to UI without waiting for tool result
**Portable**: Enable background tool execution with streaming progress. No polling needed. Tool result and notifications flow independently.

### 10. Retry Manager with Block Cap (agent.rs:361, 607-628, 1673)
**Evidence**: RetryManager, handle_retry_logic(), retry_attempts counter, stop_hook_block_cap
**Pattern**:
- RetryManager tracks attempt count (increment_retry_attempts())
- Stop hook denials incremented until cap (1792-1798)
- Cap override: DEFAULT_STOP_HOOK_BLOCK_CAP = 8, configurable via GOOSE_STOP_HOOK_BLOCK_CAP env var
- Exceeding cap forces exit with warning (1792-1798)
**Portable**: Prevent infinite loops. Track retries and deny/block counts. Hard cap breaks deadlocks. Configurable limits avoid hardcoded policy.

### 11. Provider Error Enhancement (reply_parts.rs:27-49)
**Evidence**: enhance_model_error() checks for "4xx model" pattern, returns recommended models
**Pattern**:
- On ProviderError::RequestFailed, regex-check for model availability errors (404, 400, 403)
- If match, fetch_recommended_models() from provider
- Append list to error message: "Available models: [...]"
- User sees actionable fallback options immediately
**Portable**: Transform provider errors into user guidance. Avoid opaque error messages. Leverage provider.fetch_recommended_models() for dynamic suggestions.

### 12. Tool Argument Coercion (reply_parts.rs:101-122)
**Evidence**: coerce_tool_arguments() adapts string args to schema type (number, boolean, etc.)
**Pattern**:
- Tool schema declares parameter types (JSON schema)
- If provider sends string "true" for boolean param, convert to json!(true)
- If provider sends string "42" for integer, parse and return json!(42)
- Handles schema alternatives (["string", "null"] -> try each)
**Portable**: Handles provider quirks (some LLMs always stringify args). Schema-driven coercion makes tool calls robust.

---

## SESSION ARCHITECTURE

**Session ownership**:
- SessionManager holds all session state (conversation, metrics, extensions)
- Agent::reply() references session_config.id, fetches session at start (1561)
- All message adds go through session_manager.add_message() (thread-safe)
- Conversation is mutable during turn loop, synced back to session after each turn

**Conversation context**:
- Loaded at reply start (1564-1567)
- MOIM-injected before each provider call (1823)
- Messages filtered to agent-visible only (reply_parts.rs:264)
- Full history (including tool calls) stored in session

---

## DETERMINISM & STATE MACHINES

**No explicit FSM in agents/** — but implicit state machine:
1. WAITING_LLM_STREAM_START (1284 debug point)
2. STREAM_RESPONSE (while loop 1869)
3. TOOL_DISPATCH (categorize + execute 1883-2050)
4. COLLECT_MESSAGES (accumulate in messages_to_add)
5. STOP_HOOK_CHECK (1778-1806)
6. LOOP (back to provider call or exit)

**Determinism**:
- Stream is order-preserving (futures preserve yield order)
- Tool execution is sequential per turn (no parallel tool calls within one turn)
- Subagent calls are blocking (parent waits for full completion)
- Usage metrics are accumulated in order (no race on session write)

---

## KEY DIFFERENCES FROM ELAI

Goose's agent loop is **streaming-first** and **permission-gated**:
- No orchestrate skill or phase decomposition
- No explicit multi-step reasoning (reply runs in single turn loop)
- Stop hook is **runtime** (emit_blocking at every turn end, user can deny)
- Tool approval is **explicit** (tool_confirmation_router waits for user confirmation)
- Subagent model selection is **inherited** (no per-role/tier routing)
- ProviderUsage is **session-scoped** (accumulated per session, not per turn)
- Cancellation is **token-driven** (CancellationToken, not FSM state)

ELAI by contrast has:
- Explicit FSM with states (IDLE, IMPLEMENTING, AUDITING, MEMORIZING)
- Multi-phase orchestration (context-compiler, integration-explorer, executor, auditor, memory-writer)
- Implicit stop (end of turn is end of phase, unless blocked)
- Tool approval is **static** (gates frozen at planning time, not runtime)
- Subagent routing is **tier-aware** (SIMPLE/FOCUSED/HEAVY dispatch to different models)
- Per-sub-phase metrics (not session-wide)

Goose is a **real-time interactive agent** (human in the loop at every tool call).
ELAI is an **autonomous planner-executor-auditor** (human in loop at plan boundaries).

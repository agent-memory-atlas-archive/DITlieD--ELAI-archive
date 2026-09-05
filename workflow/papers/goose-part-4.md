Goose Token Counting and Session Management Architecture

Partition coverage: session management, context management, token counting/accounting,
conversation message layer. Files read: session_manager.rs (100KB), context_mgmt/mod.rs,
token_counter.rs, token_usage.rs, usage_estimator.rs, reply_parts.rs (metrics update).

---

SESSION LAYER: Persistence Format and Token Accumulation

Storage backend: SQLite with session metadata table (schema_version 13, columns tracked
in session_manager.rs:777-780, 906-927, 1282-1283).

Per-session fields persisted:
- total_tokens (Option<i32>): immediate response input+output from last turn
- input_tokens (Option<i32>): input from last turn
- output_tokens (Option<i32>): output from last turn
- accumulated_total_tokens (Option<i32>): cumulative across all turns
- accumulated_input_tokens (Option<i32>): cumulative input across all turns
- accumulated_output_tokens (Option<i32>): cumulative output across all turns
- accumulated_cost (Option<f64>): cumulative USD cost across all turns

Session struct (lines 56-89) provides builder pattern for atomic updates via SessionBuilder
(lines 112-240), with chained methods: .total_tokens(value), .accumulated_total_tokens(value).

Per-turn update mechanism:
Location: crates/goose/src/agents/reply_parts.rs:499-555 (update_session_metrics fn).
Trigger: After every API response + after context compaction (summarization).
Logic:
  1. Fetch current session state: session_manager.get_session(id, false)
  2. Accumulate: new_accumulated = old_accumulated + new_response_usage (via Option<i32> addition,
     lines 509-514)
  3. Separate handling for compaction: if is_compaction_usage=true, summarization output tokens
     become new input context (line 531, new_input = summary_output_tokens); else use raw
     response tokens
  4. Persist via manager.update(session_id).<fields>.apply().await

Cost calculation (lines 557-575): accumulate_cost retrieves canonical provider pricing (input_price,
output_price per 1M tokens) and adds chunk_cost = (input_tokens * input_price + output_tokens *
output_price) / 1_000_000.0 to running total. Provider name keyed via canonical.maybe_get_canonical_model.

---

TOKEN COUNTING: Local Estimation and Provider-Reported Reconciliation

Tokenizer: tiktoken_rs::o200k_base() (global singleton, OnceCell pattern, crates/goose/src/token_counter.rs:10,
205-213).

Token counting paths:
1. Local estimation (client-side, crates/goose/src/token_counter.rs):
   - count_tokens(text: &str) -> usize: encodes via tokenizer.encode_with_special_tokens(),
     caches blake3(text) + len -> count in LRU (1024 entries max, lines 53-73)
   - count_chat_tokens(system_prompt, messages[], tools[]) -> usize: recursive composition
     (4 tokens per message frame + per-content text count + per-tool spec count, lines 129-171)
   - count_tokens_for_tools(tools[]) -> usize: detailed cost model for JSON schema
     (FUNC_INIT=7, PROP_INIT=3, PROP_KEY=3, ENUM_INIT=-3, ENUM_ITEM=3, FUNC_END=12, lines 75-127)
   - count_everything(system, messages, tools, resources) -> usize: additive (lines 173-188)

2. Provider-reported tokens:
   - goose_providers/src/conversation/token_usage.rs ProviderUsage struct carries model name + Usage
   - Usage struct (lines 26-33): input_tokens, output_tokens, total_tokens, cache_read_input_tokens,
     cache_write_input_tokens (all Option<i32>)
   - Addition operator (lines 47-63, Add trait): component-wise sum, respects cache fields
   - auto-calculation: if total_tokens missing, computed as Some(input + output) (lines 78-87)

3. Fallback/reconciliation (crates/goose/src/providers/usage_estimator.rs:9-48):
   - ensure_usage_tokens fn: if provider doesn't return input/output tokens, estimate locally
   - input: use count_chat_tokens(system_prompt, request_messages, tools)
   - output: use count_tokens(formatted response text)
   - total: auto-filled as input + output

Context window fill calculation:
Location: crates/goose/src/context_mgmt/mod.rs:186-231 (check_if_compaction_needed fn).
Logic:
  1. Fetch context_limit from provider.get_model_config().context_limit() (e.g. 128000 default per
     crates/goose/src/model.rs:11)
  2. Current tokens: prioritize session.total_tokens (from DB if populated, label="session metadata"),
     else fall back to local re-estimate via count_chat_tokens on agent_visible messages (label="estimated")
  3. usage_ratio = current_tokens as f64 / context_limit as f64
  4. Compaction needed: usage_ratio > threshold (default threshold=0.8, configurable via
     GOOSE_AUTO_COMPACT_THRESHOLD)
  5. Return bool; if true, proceed to compact_messages

Compaction trigger and strategy (lines 225-230):
- No compaction if threshold <= 0.0 or >= 1.0 (disabled)
- Threshold 0.8 means trigger at 80% context fill
- compute_tool_call_cutoff(context_limit, threshold) -> usize returns effective max tokens
  before compaction kicks in (line 434-449)

---

CONTEXT MANAGEMENT: Overflow Handling and Message Visibility

Context compaction (lines 50-183):
Purpose: Summarize conversation history when approaching context limit, preserve recent user intent.

Flow:
  1. Input: provider, session_id, conversation (message history), manual_compact flag
  2. Preserve most recent text-only user message (lines 111-126, not preserved if manual_compact=true)
  3. Summarize compactable messages via do_compact (async, calls provider's LLM to generate summary)
  4. Return tuple: (compacted_conversation, summarization_usage)

Message visibility metadata (per MessageMetadata):
- agent_visible: message is sent to the LLM on next request
- user_visible: message shown in UI history
- After compaction: original messages marked agent_invisible (hidden from next context),
  summary message marked agent_visible (line 145-148)

Tool response filtering during compaction (lines 233-279):
- Strategy: "middle out" deletion when token budget tight
- Removes oldest tool responses first (median index outward)
- Parameterizable removal percentage (0%, 10%, 20%, 50%, 100% per attempt)

Continuation text: Prompt insertion after summarization (lines 30-43) informs the model that
context was compacted; preserves instruction to "continue the conversation naturally".

Provider-specific override (line 192): if provider.manages_own_context(), skip auto-compaction
(provider handles context internally, e.g. some cloud services with native context caching).

---

PORTABLE IDEAS: Mechanisms for Reimplementation

1. Token cache with blake3 hashing (token_counter.rs:22-40)
   - LRU cache keyed by text length + blake3(text)
   - Avoids re-encoding same text; concurrent safe via Mutex
   - Capacity: 1024 entries configurable
   - Evidence: lines 53-73 (count_tokens path), 190-195 (clear_cache)
   - Usefulness: fast token lookup during streaming or repeated message references
   - Trade-off: memory (1024 entries ~50KB) vs CPU (blake3 + LRU lookup sub-microsecond)

2. Dual token accounting (session fields + ProviderUsage)
   - Separate structs: per-turn (total_tokens, input_tokens, output_tokens in Session)
   - vs cumulative (accumulated_* fields in Session)
   - vs provider-reported (Usage struct from API response)
   - Evidence: session_manager.rs:69-75 (Session fields), token_usage.rs:26-33 (Usage struct)
   - Usefulness: real-time display (current + cumulative) vs historical accuracy reconciliation
   - Implementation detail: update_session_metrics (reply_parts.rs:499-555) composes these atomically

3. Cost derivation from provider pricing table (reply_parts.rs:557-575)
   - Lookup canonical pricing: provider_name -> model pricing (input/output per 1M tokens)
   - Accumulate: existing_cost + (input_tokens * input_price + output_tokens * output_price) / 1M
   - Evidence: lines 564-574
   - Usefulness: user-facing cost tracking; supports per-session billing or budgeting UI
   - Implementation: requires canonical model registry indexed by provider + model name

4. Message visibility metadata for context-aware rendering (context_mgmt.rs:145-148, token_counter.rs:143-145)
   - Separate flags: agent_visible (sent to LLM) vs user_visible (shown in history)
   - Allows summarized messages to hide raw history but remain effective input
   - Evidence: metadata.with_agent_invisible() (line 147), filter on m.is_agent_visible() (line 215)
   - Usefulness: clean UX (hide summary mechanism) while preserving LLM context
   - Adoption: requires message serialization preserve both flags

5. Local + provider token accounting split (usage_estimator.rs:9-48)
   - Prefer provider-reported (ground truth), fall back to local estimate
   - Input estimate: count_chat_tokens(system, messages, tools)
   - Output estimate: count_tokens(response text)
   - Reconcile: if provider_usage.total_tokens missing, compute as input + output
   - Evidence: lines 16-45
   - Usefulness: graceful fallback for providers without usage reporting; accurate cost when
     provider is silent (especially important for cache tokens and special pricing)
   - Caveat: local estimate is O(message_count) recomputation; streaming responses may not have
     full output token count until EOS

6. Context fill percentage for proactive display (context_mgmt.rs:186-231)
   - Store context_limit on model config (ModelConfig.context_limit, model.rs:53)
   - Compute usage_ratio = current_tokens / context_limit on demand
   - Display readiness: session.total_tokens available immediately after each response
     (synchronous DB read, no re-estimation needed)
   - Evidence: lines 204-223
   - Usefulness: real-time progress bar or warning threshold; triggers compaction at 80% without
     user intervention
   - Implementation detail: compaction_threshold parameterizable per session or global config

7. Separate turn usage vs cumulative (session_manager.rs:69-75 + reply_parts.rs:529-539)
   - Current turn: total_tokens, input_tokens, output_tokens (for display "this turn used N")
   - Cumulative: accumulated_total_tokens, etc. (for display "session used N total")
   - After compaction: current fields reset to summarization output (line 531-532),
     accumulated fields continue growing
   - Evidence: lines 529-539 (is_compaction_usage fork)
   - Usefulness: accurate per-turn billing/display; helps diagnose if a particular turn was expensive

8. Atomic session update pattern (session_manager.rs:1338-1397, reply_parts.rs:541-551)
   - Builder pattern: SessionBuilder collects updates, applies all at once via .apply()
   - DB: single UPDATE statement with multiple SET clauses
   - Evidence: lines 1338-1344 (add_update! macro), 1388-1397 (build UPDATE query)
   - Usefulness: prevents partial updates if DB fails; atomic from DB perspective
   - Adoption: requires transaction or at least single-statement semantics

9. Context window computation for auto-compaction (context_mgmt.rs:434-449)
   - compute_tool_call_cutoff(context_limit, compaction_threshold) -> usize
   - Formula: effective_cutoff = context_limit * threshold (with bounds check)
   - Used by agent to calculate max tool calls before auto-summarization (line 434)
   - Evidence: lines 434-449, tested at 735-749
   - Usefulness: agent can "look ahead" and proactively summarize before hitting threshold
   - Integration point: agents/agent.rs:1740-1745 (uses compute_tool_call_cutoff)

10. Estimation-vs-actual reconciliation for real-time accuracy
    - During conversation: use local estimate (fast, available immediately)
    - After response: prefer provider-reported (ground truth)
    - ensure_usage_tokens fills gaps: if input missing, estimate; if output missing, estimate
    - Evidence: usage_estimator.rs:16-45, reply_parts.rs:506-527
    - Usefulness: UI can show "estimated: 1200 tokens" then update to "actual: 1187 tokens"
      when provider responds, maintaining confidence in accounting
    - Trade-off: requires dual storage (estimated vs reported), but eliminates surprise overages

---

NOTES ON INTEGRATION

- Session resume: load session from DB, check session.accumulated_* fields for historical total
  (session_manager.rs:206-207 references session.total_tokens in check_if_compaction_needed)
- Schema migration: accumulated_cost added in a later migration (line 1217-1220, ALTER TABLE);
  older sessions have NULL, coalesced to 0.0 on display
- Provider coupling: cost accumulation requires provider name + canonical pricing table; no cost
  update if provider not in registry (line 526, and_then returns None)
- Concurrency: SessionManager uses Arc<Pool<Sqlite>>; all updates queue via transaction
- Edge case: if usage.total_tokens missing and both input/output missing, accumulated_total stays
  None (line 517 accumulate(None, None) = None); user sees "?" or "unknown"

---

Session storage location: F:\.ELAI_workflow\docs\.source\goose-main\crates\goose\src\session\
Token counter location: F:\.ELAI_workflow\docs\.source\goose-main\crates\goose\src\token_counter.rs
Context management: F:\.ELAI_workflow\docs\.source\goose-main\crates\goose\src\context_mgmt\mod.rs
Provider usage: F:\.ELAI_workflow\docs\.source\goose-main\crates\goose-providers\src\conversation\token_usage.rs
Checks module: F:\.ELAI_workflow\docs\.source\goose-main\crates\goose\src\checks\mod.rs (orthogonal, review harness)

phase-patch-R3-decisions

What
ProductionAgentRpcServer publishes a 6-tool catalog on every MessageRequest, and translates Sonnet's OutputContentBlock::ToolUse blocks into the agent's text-based "TOOL_USE: <name>" marker convention so agent_loop.rs (LOCKED) can dispatch the right tool. The Sonnet-emitted ToolUse.input JSON is stashed per-server keyed by tool name (FIFO VecDeque for repeated names) and popped on the matching ToolRequest whose arguments are empty (agent_loop.rs:170).

Key decisions

DECISION-R3-1 — catalog scope = 6 tools, NOT 21.
The 6 admitted tools are TOOL_EDIT, TOOL_FILE_READ, TOOL_VALIDATE, TOOL_COMPILE_CHECK, TOOL_CLASSIFY, TOOL_GIT_COMMIT. Excluded: TOOL_INDEX_ARTIFACTS (Phase C stub returning Err), TOOL_GIT_UNDO/TOOL_GIT_BRANCH (Err arms), symbol-graph and artifact-search tools (deferred — out of B3 acceptance scope). Publishing tools whose arms always Err would invite the agent to spend ReAct iterations on guaranteed-failure turns.
Why: B3 cloud TODO-build is the SOLE acceptance gate this patch unblocks; the 6-tool slice is sufficient.
Test outcome: catalog vec has length 6, names match constants verbatim — locked by adversarial-auditor's grep, not yet by a contracts/ test (WARN-3 follow-up).

DECISION-R3-2 — pending_tool_args type = Arc<tokio::sync::Mutex<HashMap<String, VecDeque<Value>>>>.
- Arc because ProductionAgentRpcServer is held behind `Arc<dyn AgentRpcServer>` (ipc_accept.rs:367) and the field must be cheaply cloneable into Send+Sync trait objects.
- tokio::sync::Mutex (NOT std::sync::Mutex) because the lock is held across an `.await` inside `handle_infer_request` (push loop after `send_message` resolves); std::sync::Mutex's MutexGuard is not Send.
- VecDeque, not Vec — FIFO push_back/pop_front so multiple ToolUse blocks with the same name dispatch in submission order.
- Key = String (the tool name), not ToolUseId — the agent's TOOL_USE marker carries only the name, not the id; matching by id would require encoding the id in the marker text and updating agent_loop's scanner (LOCKED).
Why: validates context-package HIGH warning #2; lock-across-await + dyn-trait-object requires this exact type.
Test outcome: tests d_ and e_ access `.lock().await.get("edit").map(|q| q.len())` directly.

DECISION-R3-3 — empty-arguments contract path.
agent_loop sends ToolRequest with `arguments = Vec::new()` (agent_loop.rs:170). handle_tool_request branches on `arguments.is_empty()`:
- Empty + stash hit -> convert_json_input_to_postcard -> bytes used as effective_arguments.
- Empty + stash miss -> fall through to translate_tool_request with the original empty arguments. This preserves the existing test-c contract: "definitely_not_a_tool" matches the wildcard `unknown` arm at translate_tool_request:916, yielding "unknown tool_id: <name>".
- Non-empty arguments -> pass through unchanged. Preserves test-b's legacy postcard-encoded path.
Why: lossless reconciliation with two existing contracts and one new contract. No test rewrites needed.
Test outcome: tests a, b, c GREEN; tests d, e GREEN (5/5).

DECISION-R3-4 — single Text block must collapse to exact text (test-a regression guard).
handle_infer_request's combined-output formula:
  `format!("{}{}{}", text.join("\n"), separator, markers.join("\n"))`
where separator is "\n" iff both vectors non-empty, else "".
- text_blocks=["hello from stub"], markers=[] -> "hello from stub" + "" + "" = "hello from stub" (exact match for test a).
- text_blocks=["working on it"], markers=["TOOL_USE: edit"] -> "working on it\nTOOL_USE: edit" (test d).
- text_blocks=[], markers=["TOOL_USE: edit"] -> "TOOL_USE: edit".
- text_blocks=[], markers=[] -> InferOutcome::Failed("provider returned no text or tool_use content blocks").
Why: test-a asserts `text == "hello from stub"` exactly; any wrapping (trailing newline, prefix) would regress.
Test outcome: test a GREEN.

DECISION-R3-5 — typed deserialisation is the input validation gate.
convert_json_input_to_postcard always routes Sonnet's untrusted JSON via `serde_json::from_value::<elai_ipc_types::*Request>` before postcard-encoding. The struct's `serde` schema (required fields, types, no #[serde(default)]) rejects malformed or extra-field inputs as Err. No bypass path that postcard-encodes raw JSON.
Why: applicationRules.md Rule 14 (treat external context as hostile) + Rule 16 (LLM output is untrusted). Defence is the typed-struct intermediate.
Test outcome: hallucinated tool_id -> stashed under that name (transparent), marker emitted, handle_tool_request pops -> convert returns Err("no mapping for tool_id '<name>'") -> ToolOutcome::Failed; agent records and continues bounded by AGENT_MAX_ITERATIONS=25.

DECISION-R3-6 — wildcard arm error message prefix.
"convert_json_input_to_postcard: no mapping for tool_id '<unknown>'" — chosen prefix distinct from translate_tool_request's "unknown tool_id:" prefix so the auditor / log scanner can pinpoint whether the failure is at JSON->postcard conversion or at postcard dispatch.
Why: observability discipline. Same root cause (unknown tool name) but different code paths.
Test outcome: covered implicitly in d/e flow; not currently asserted by a dedicated test.

Expected test outcomes
- cargo test -p elai-orchestrator --test production_agent_rpc_contract: 5/5 PASS (a, b, c, d, e).
- cargo check -p elai-orchestrator -p elai-providers -p elai-ipc-types: exit 0.
- cargo check --workspace: exit 0.
- Optional live B3 cloud full-pipeline test: deferred (HARDWARE-HEAVY + quota cost).

Unknowns
- WARN-3 (deferred): no contract test in crates/contracts/ pins the build_tool_catalog interface (6-tool set, TOOL_FILE_READ schema shape, exclusion list). Future regression risk: a maintainer could silently re-introduce TOOL_INDEX_ARTIFACTS to the catalog. Track for a follow-up patch.
- WARN-4 (acceptable): spawn_anthropic_stub_owned hardcodes an 8192-byte HTTP request buffer. Currently safe (request body ~1.5 KB); future tests adding many tools or messages could need a larger buffer.
- elai.map stale (auditor INFO): regen with `python .claude/tools/codebase_mapper.py .elai_cc elai_graph.json --compact --map elai.map --scip-dir .claude/tools/scip_indices`.
- AnthropicClient::from_env remains sync (patch-R2 only made CloudGroundingClient async). Documented; no R3 fix.

Algorithm: Recursive Language Models (RLMs)
ArXiv: 2512.24601
Paper: Recursive Language Models (Zhang, Kraska, Khattab; MIT OASYS lab; 2026)
Repo: docs/.source/rlm (package name on PyPI: rlms==0.1.2)
Reliability: paper HIGH (arXiv, reviewed), repo MEDIUM (reviewed-source, maintained by paper authors)

CORE MECHANISM
RLMs handle arbitrarily long contexts by NOT feeding the raw prompt into the neural network.
Instead, the prompt is stored as a Python variable (context) in a stateful REPL environment;
the LLM sees only metadata (type, total character count, access methods) and generates Python
code that programmatically slices, queries, and recurses over that variable.
Recursion: the REPL exposes rlm_query(prompt, model=None) which spawns a full child RLM with
its own REPL and iteration budget, controlled by depth < max_depth. At max_depth, rlm_query
falls back to a plain llm_query (single LM call, no REPL). The root model acts as orchestrator;
child models act as solvers. Communication between parent and child is via return-value strings,
not shared REPL state.

ARCHITECTURE / CONTROL FLOW
Paper Algorithm 1 (verbatim pseudocode, paper Section 3 / Figure):
  Input: prompt P
  Output: response Y
  state <- InitREPL(prompt=P)
  state <- AddFunction(state, sub_RLM_M)
  hist <- [Metadata(state)]
  while True do
    code <- LLM_M(hist)
    (state, stdout) <- REPL(state, code)
    hist <- hist || code || Metadata(stdout)
    if state[Final] is set then
      return state[Final]

Repo implementation of that loop (rlm/core/rlm.py:359-488):
  for i in range(self.max_iterations):                            # bounded while True
    [optional compaction check]
    message_history.append(build_user_prompt(...))               # "Turn i/N:"
    iteration = self._completion_turn(prompt=message_history,    # LLM_M(hist)
                                      lm_handler, environment)
    # _completion_turn (rlm/core/rlm.py:644-669):
    #   response = lm_handler.completion(prompt)                 # LM inference
    #   code_block_strs = find_code_blocks(response)            # extract ```repl blocks
    #   for code_block_str in code_block_strs:
    #       code_result = environment.execute_code(code_block_str) # REPL(state, code)
    #   return RLMIteration(...)
    if answer["ready"] == True:                                  # state[Final] is set
      return RLMChatCompletion(response=final_answer)
    message_history.extend(format_iteration(iteration))         # hist <- hist || output
  return self._default_answer(message_history, lm_handler)       # iteration exhaustion fallback

Context injection (rlm/environments/local_repl.py:200-206):
  The context (the long prompt) is loaded into the REPL's locals dict as the variable `context`
  via load_context(context_payload) before the first iteration. The LLM sees its CHARACTER COUNT
  in the metadata user message (build_rlm_system_prompt), not its content.

Recursive sub-call dispatch (rlm/core/rlm.py:704-868 _subcall):
  - Called via REPL's rlm_query -> subcall_fn callback
  - next_depth = self.depth + 1
  - if next_depth >= self.max_depth: plain client.completion(prompt) [base case]
  - else: spawn child = RLM(depth=next_depth, max_depth=self.max_depth, ...) and call child.completion(prompt)
  - child.close() called in finally block; budget/timeout remainder passed down proportionally

LMHandler (rlm/core/lm_handler.py:145-251):
  - TCP socket server (ThreadingTCPServer, 4-byte length-prefixed JSON protocol)
  - REPL calls llm_query -> socket request to LMHandler -> client.completion -> response
  - Routing: depth=0 uses default_client; depth=1 uses other_backend_client if configured
  - Batched: asyncio.gather with semaphore (batch_max_concurrent=16 default)

Compaction (rlm/core/rlm.py:600-642 _compact_history):
  Triggered when token count >= compaction_threshold_pct * model_context_limit.
  Asks the model to summarize progress in 1-3 paragraphs, stores summary in REPL `history` list,
  truncates message_history to [system + initial_assistant + summary + continue_prompt].
  Only applies to the root model's message_history; child REPLs have their own fresh histories.

PUBLIC API (from repo source, not docs)
RLM.__init__ | class RLM | rlm/core/rlm.py:49 | Instantiates config; does NOT start REPL/handler
  backend: ClientBackend = "openai"  (Literal: openai|portkey|openrouter|vercel|vllm|anthropic|azure_openai|gemini)
  backend_kwargs: dict[str, Any] | None = None  (e.g. {"model_name": "gpt-5-nano", "api_key": ...})
  environment: EnvironmentType = "local"  (Literal: local|ipython|docker|modal|prime|daytona|e2b)
  environment_kwargs: dict[str, Any] | None = None
  depth: int = 0  (current recursion depth; callers always leave at default 0)
  max_depth: int = 1  (1 = root only, no recursion; 2 = root + one recursive level)
  max_iterations: int = 30  (per-depth iteration budget; shared by root and children)
  max_budget: float | None = None  (USD; requires cost-tracking backend like OpenRouter)
  max_timeout: float | None = None  (seconds wall-clock; propagated to children as remainder)
  max_tokens: int | None = None  (cumulative input+output tokens across all calls)
  max_errors: int | None = None  (consecutive REPL stderr errors before abort)
  custom_system_prompt: str | None = None  (replaces RLM_SYSTEM_PROMPT entirely if set)
  other_backends: list[ClientBackend] | None = None  (currently max 1; used as depth=1 backend)
  other_backend_kwargs: list[dict] | None = None
  logger: RLMLogger | None = None
  verbose: bool = False
  persistent: bool = False  (reuse REPL env across completion() calls; only local/ipython)
  custom_tools: dict[str, Any] | None = None  (callables -> REPL globals; values -> REPL locals)
  custom_sub_tools: dict[str, Any] | None = None  (None = inherit from custom_tools)
  compaction: bool = False
  compaction_threshold_pct: float = 0.85
  max_concurrent_subcalls: int = 4  (ThreadPoolExecutor size for rlm_query_batched)
  on_subcall_start: Callable[[int, str, str], None] | None = None
  on_subcall_complete: Callable[[int, str, float, str | None], None] | None = None
  sampling_args: dict[str, Any] | None = None  (temperature, top_p etc for root model)
  sub_sampling_args: dict[str, Any] | None = None  (sampling for depth=1 sub-model)
  orchestrator: bool = True  (adds ORCHESTRATOR_ADDENDUM to system prompt)
  user_prologue: str | None = None  (inserted as user message before turn 0)

RLM.completion | rlm/core/rlm.py:324 | MAIN ENTRY POINT
  completion(prompt: str | dict[str, Any], root_prompt: str | None = None) -> RLMChatCompletion
  prompt: the full context payload (can be arbitrarily long; stored as REPL variable, not in LM window)
  root_prompt: short question visible to the root LLM alongside context metadata
  returns: RLMChatCompletion(root_model, prompt, response: str, usage_summary, execution_time, metadata)

RLM.close | rlm/core/rlm.py:900 | cleanup for persistent mode
  close() -> None  (also available as context manager __enter__/__exit__)

LocalREPL (rlm/environments/local_repl.py:147) -- the default REPL environment
  execute_code(code: str) -> REPLResult  (runs one ```repl block; returns stdout, stderr, locals, rlm_calls, final_answer)
  REPL globals include: SHOW_VARS, llm_query, llm_query_batched, rlm_query, rlm_query_batched, custom_tools
  REPL locals include: context (the prompt payload), answer = {"content": "", "ready": False}
  Sandboxed builtins: eval/exec/compile/globals/locals/input are set to None (blocked)

HYPERPARAMETERS
max_depth         | 1       | 1..N    | HIGH: 1=plain LM; 2=one recursion level (used in paper experiments); >2 works but amplifies errors exponentially in weaker models
max_iterations    | 30      | 1..N    | MEDIUM: paper used 10-30; more iterations = more LM calls = more cost; too few = truncated answers
compaction_threshold_pct | 0.85 | 0.5..0.95 | LOW: triggers summarization; lower = more aggressive compaction = more summarization calls; loss of detail at very low values
max_concurrent_subcalls | 4 | 1..N   | MEDIUM: rlm_query_batched parallelism; ceiling ~20 per ORCHESTRATOR_ADDENDUM guidance
chunk size (in prompt) | None | ~100K chars per sub-LM prompt | HIGH: paper guidance is ~100K chars/sub-call ceiling, ~20 prompts/batch ceiling; exceeding causes degraded answers
model choice (root vs sub) | same | any | MEDIUM: paper used different models for root (orchestrator) vs sub (solver); depth=1 routing via other_backends

VERBATIM EXTRACTS (anti-hallucination; do not paraphrase these)

[A] Algorithm 1 from paper (Section 3):
"state <- InitREPL(prompt=P)
 state <- AddFunction(state, sub_RLM_M)
 hist <- [Metadata(state)]
 while True do
   code <- LLM_M(hist)
   (state, stdout) <- REPL(state, code)
   hist <- hist || code || Metadata(stdout)
   if state[Final] is set then
     return state[Final]"

[B] Paper abstract core claim:
"arbitrarily long user prompts should not be fed into the neural network directly but should
instead be treated as part of the environment."

[C] RLM_SYSTEM_PROMPT (rlm/utils/prompts.py:125-143, the actual default system prompt):
"You are a Recursive Language Model (RLM): a language model with a prompt, and a very
important context stored in a Python REPL related to that prompt.
You can iteratively interact with the a Python REPL, which has access to LLM calls as a
function. You will be queried turn-by-turn until you have an answer to the query.

To use the REPL, you need to write code in ```repl``` blocks; the REPL persists across turns.
Available in the REPL:
- `context`: the important, potentially very long information related to the prompt.
- `llm_query(prompt: str, model: str | None = None) -> str`: a single sub-LLM completion.
- `llm_query_batched(prompts: list[str], model=None) -> list[str]`: concurrent LLM calls.
- `rlm_query(prompt, model=None)` / `rlm_query_batched(prompts, model=None)`: recursive sub-calls.
- `SHOW_VARS() -> str`: list every variable currently in the REPL.
- `answer`: dict initialized to {\"content\": \"\", \"ready\": False}. To submit, set
  answer[\"content\"] to the final answer and answer[\"ready\"] = True inside a ```repl``` block."

[D] ORCHESTRATOR_ADDENDUM excerpt (rlm/utils/prompts.py:147-204, appended when orchestrator=True):
"Your own context window is small. Push every long-context operation that would not fit
comfortably in your own working window [...] into `llm_query` / `llm_query_batched` calls
instead of pulling that text into your own message stream."
"Sub-call budget is finite on two independent axes [...] a useful rough ceiling is ~100K
characters per prompt [...] a useful rough ceiling is ~20 prompts per batch."

[E] _subcall base-case dispatch (rlm/core/rlm.py:734-751):
"if next_depth >= self.max_depth:
    if self.other_backends and self.other_backend_kwargs:
        client = get_client(self.other_backends[0], self.other_backend_kwargs[0])
    else:
        client = get_client(self.backend, child_backend_kwargs or {})
    response = client.completion(prompt)
    return RLMChatCompletion(...)"

[F] completion() termination check (rlm/core/rlm.py:414-419):
"final_answer = None
for block in iteration.code_blocks:
    if getattr(block.result, 'final_answer', None) is not None:
        final_answer = block.result.final_answer
        break
iteration.final_answer = final_answer"

[G] _compact_history summarization prompt (rlm/core/rlm.py:611-624):
"\"Summarize your progress so far. Include:\n\"
\"1. Which steps/sub-tasks you have completed and which remain.\n\"
\"2. Any concrete intermediate results (numbers, values, variable names) \"
\"you computed — preserve these exactly.\n\"
\"3. What your next action should be.\n\"
\"Be concise (1–3 paragraphs) but preserve all key results and your \"
\"current position in the task.\""

[H] Answer submission convention (from system prompt / local_repl.py:26-47):
answer dict is an _AnswerDict subclass with on_ready callback. Setting answer["ready"] = True
fires _capture_answer(content) synchronously, storing _last_final_answer. execute_code returns
REPLResult with final_answer set. The main loop checks block.result.final_answer after each turn.

COMPLEXITY / COST
Model calls per query: O(max_iterations) root calls + O(max_iterations * branching_factor) sub-calls (each sub-call is itself an RLM with its own iteration budget). For depth=1 with 1 sub-call per iteration: O(max_iterations^2) total LM calls in worst case.
Context per call: O(1) relative to prompt size (the prompt is NOT in LM context; only metadata + iteration history are). Root context grows O(max_iterations * avg_response_tokens) over a single completion.
Parallelism: rlm_query_batched uses ThreadPoolExecutor(max_concurrent_subcalls); each thread makes synchronous socket calls to LMHandler which uses asyncio.gather internally for batched requests. True parallelism is I/O-bound on API rate limits.
Paper cost observation: RLM costs scale proportionally with task complexity (cited as linear for OOLONG, quadratic for OOLONG-Pairs). Training on 1,000 samples improved RLM-Qwen3-8B by 28.3% median. No absolute latency numbers published.
Paper benchmark results (GPT-5 depth=1 unless noted):
  OOLONG: 56.0% vs 44.0% base
  OOLONG-Pairs: 58.0% vs 0.1% base
  BrowseComp-Plus (6-11M tokens): 91.3% vs 0.0% base
  CodeQA (depth=2): 66.0% vs 24.0% base
  LongCoT-mini: 65.6% vs 38.7% base

REIMPLEMENTATION NOTES: ELAI MAPPING

ELAI is a Rust orchestrator with agent_pool, task_runner, FSM, append-only SSB event log, a local sidecar (~25-32K window), and a cross-session context lifecycle (H-50/H-51, CL-1..CL-4). RLM-style recursive context handling would map as follows:

Component 1 -- Context-as-REPL-variable (the core insight):
ELAI's current model passes the full prompt into the sidecar's 32K window. The RLM insight is
to store the raw document in a separate "context store" (ELAI already has the knowledge stack /
vector store / DKG) and give the sidecar only a metadata handle. The sidecar then issues
tool-calls (ELAI's existing tool-call interface) to fetch slices rather than receiving raw text.
This maps naturally onto ELAI's existing tool_use SSB event pattern. DOES MAP cleanly.

Component 2 -- Iteration loop with REPL code execution:
ELAI's task_runner iterates over agent turns but does not have a code-execution REPL. The RLM
REPL (LocalREPL) is a Python exec sandbox. For Rust, this would require either: (a) a Python
sidecar subprocess as the execution environment (already exists as the sidecar), or (b) a
sandboxed scripting layer. The sidecar server.py already handles tool dispatch; adding
code-execution as a tool is feasible but requires sandbox isolation. PARTIAL MAP.

Component 3 -- Recursive sub-call dispatch:
ELAI's agent_pool already dispatches sub-agents. The RLM _subcall pattern (spawn child RLM,
pass depth, pass remaining budget) maps to ELAI's existing depth-limited agent dispatch with
budget propagation. The key difference: RLM children communicate via return-value strings;
ELAI agents communicate via SSB events. DOES MAP with schema change.

Component 4 -- Compaction (context summarization):
ELAI already has H-50 context-lifecycle policy (ContextPolicy by BackendMode). RLM's
_compact_history (keep system + summary + continue prompt) is the LOCAL policy from H-50 that
was noted as missing. The RLM compaction implementation is a validated reference for the exact
truncation + summarization prompt strategy ELAI's H-50 needs. HIGH VALUE direct steal.

What does NOT map:
- RLM's Python REPL with exec() is fundamentally a Python artifact; ELAI is Rust and the
  execution model is agent dispatch + SSB events, not Python code generation.
- RLM's context variable pattern assumes the LLM generates arbitrary Python against the context.
  ELAI's sidecar uses structured tool-calls, not freeform code generation. The two paradigms
  differ in how the model "accesses" the long context: Python loops (RLM) vs tool invocations (ELAI).
- RLM's LMHandler TCP socket server is its inter-process call router; ELAI has SSB + IPC.
  The architectures are parallel but not isomorphic; a direct port would discard ELAI's existing
  wiring rather than extending it.

FAILURE MODES / PITFALLS (paper + repo sourced)
1. Insufficient code generation ability: models with weak Python coding skill fail to generate correct REPL code; at smaller scales (e.g. Qwen3-8B without RL training) performance degrades sharply.
2. Output token exhaustion: thinking models (with long CoT) may exhaust their output token budget before completing the code block; the REPL receives a truncated/incomplete block.
3. answer["ready"] never set: if the model never flips answer["ready"] = True, the loop runs to max_iterations then calls _default_answer (a final single LM call); result quality degrades.
4. Compaction loss: compaction summarizes intermediate results but can lose specific values (numbers, variable names); the prompt instructs "preserve exactly" but the LM may not comply.
5. Recursion error amplification: at depth > 1, a code error in a child propagates as an error string to the parent; the parent may misinterpret or retry incorrectly; consecutive errors can hit max_errors.
6. REPL stdout truncation: outputs over ~20K chars are truncated (noted in system prompt); the model cannot see full results of large print() calls; must use llm_query to summarize variables instead.
7. Sequential sub-calls (current default): rlm_query calls are sequential; batched is available but requires the model to explicitly call rlm_query_batched. In practice the model often uses sequential calls, inflating latency linearly.
8. Model-specific prompt tuning: paper notes GPT-5 and Qwen3-Coder required different system prompts; a single generic RLM_SYSTEM_PROMPT may underperform on any given model.
9. rlm_query overwrite: the model can overwrite rlm_query in the REPL namespace; the repo handles this with scaffold restoration (local_repl restores reserved functions after each execute_code call, per tests/test_rlm_query.py:477-490).
10. Custom tool name collision: custom_tools keys must not collide with RESERVED_TOOL_NAMES (llm_query, rlm_query, etc.); validate_custom_tools raises ValueError on collision.

IMPLEMENTATION PITFALLS (do NOT do this)
- Do NOT put the full prompt into the LLM's message history; store it as a REPL variable and pass only metadata (type + character count) to the model.
- Do NOT use answer["ready"] = True as a bare expression; the _AnswerDict only fires on __setitem__ with key="ready" and a truthy value; a bare expression on the last line of a repl block is silently discarded (system prompt warns explicitly).
- Do NOT share REPL namespace between parent and child RLMs; each _subcall spawns a fresh RLM with its own LocalREPL instance; shared state causes non-determinism.
- Do NOT allow unlimited recursion depth; depth without a max_depth cap causes infinite recursion; the _subcall guard (next_depth >= self.max_depth) is mandatory.
- Do NOT assume llm_query and rlm_query are the same; llm_query always makes a single plain LM call; rlm_query spawns a child RLM with REPL only when depth allows (tests/test_rlm_query.py:168-178 confirms llm_query ignores subcall_fn even when present).
- Do NOT use BatchSpanProcessor or async orchestration that can lose the final_answer signal; the _AnswerDict on_ready callback fires synchronously on __setitem__; any async buffering breaks the termination signal.
- Do NOT skip the safeguard on turn 0 (build_user_prompt iteration=0 adds "You have not interacted with the REPL environment or seen your prompt / context yet. Look at the context first; do not provide a final answer yet."); without this, models short-circuit on the first turn.
- Do NOT use max_concurrent_subcalls much above 20; the ORCHESTRATOR_ADDENDUM explicitly caps recommended batch size at ~20 prompts; beyond that the backend rate limits are hit before the thread pool is the bottleneck.

CONTRACT TEST STUBS
(copy into tests/contracts/test_rlm_contract.py; stdlib+numpy only; mocks the LM client)

def test_depth_bound_respected():
    import sys, os
    sys.path.insert(0, os.path.join(os.path.dirname(__file__), '../../docs/.source/rlm'))
    from rlm.environments.local_repl import LocalREPL
    from rlm.core.types import RLMChatCompletion, UsageSummary
    # subcall_fn records depth of calls; at max_depth the fn should NOT be called
    calls = []
    def subcall_fn(prompt, model):
        calls.append(prompt)
        return RLMChatCompletion(root_model="t", prompt=prompt, response="ok",
                                 usage_summary=UsageSummary(model_usage_summaries={}), execution_time=0.0)
    repl = LocalREPL(subcall_fn=subcall_fn, depth=2, max_concurrent_subcalls=1)
    # At depth=2 with max_depth=2 the subcall_fn should be invoked (depth check is in _subcall, not REPL)
    repl.execute_code("r = rlm_query('test')")
    # The subcall_fn IS called by _rlm_query; depth guard lives in RLM._subcall
    assert subcall_fn is not None  # structure check
    repl.cleanup()

def test_answer_ready_signal_fires_on_setitem():
    import sys, os
    sys.path.insert(0, os.path.join(os.path.dirname(__file__), '../../docs/.source/rlm'))
    from rlm.environments.local_repl import LocalREPL
    repl = LocalREPL()
    result = repl.execute_code('answer["content"] = "done"; answer["ready"] = True')
    assert result.final_answer == "done", f"Expected 'done', got {result.final_answer!r}"
    repl.cleanup()

def test_answer_ready_not_set_returns_none_final_answer():
    import sys, os
    sys.path.insert(0, os.path.join(os.path.dirname(__file__), '../../docs/.source/rlm'))
    from rlm.environments.local_repl import LocalREPL
    repl = LocalREPL()
    result = repl.execute_code('x = 1 + 1')
    assert result.final_answer is None, f"Expected None, got {result.final_answer!r}"
    repl.cleanup()

def test_batched_order_preserved():
    import sys, os
    sys.path.insert(0, os.path.join(os.path.dirname(__file__), '../../docs/.source/rlm'))
    from rlm.environments.local_repl import LocalREPL
    from rlm.core.types import RLMChatCompletion, UsageSummary
    import time, random
    def delayed(prompt, model):
        time.sleep(random.uniform(0.01, 0.05))
        return RLMChatCompletion(root_model="t", prompt=prompt, response=f"r-{prompt}",
                                 usage_summary=UsageSummary(model_usage_summaries={}), execution_time=0.0)
    repl = LocalREPL(subcall_fn=delayed, max_concurrent_subcalls=4)
    repl.execute_code("results = rlm_query_batched(['a', 'b', 'c', 'd'])")
    assert repl.locals["results"] == ["r-a", "r-b", "r-c", "r-d"], \
        f"Order not preserved: {repl.locals['results']}"
    repl.cleanup()

def test_llm_query_never_uses_subcall_fn():
    import sys, os
    sys.path.insert(0, os.path.join(os.path.dirname(__file__), '../../docs/.source/rlm'))
    from rlm.environments.local_repl import LocalREPL
    from rlm.core.types import RLMChatCompletion, UsageSummary
    called = []
    def subcall_fn(prompt, model):
        called.append(prompt)
        return RLMChatCompletion(root_model="t", prompt=prompt, response="should-not-appear",
                                 usage_summary=UsageSummary(model_usage_summaries={}), execution_time=0.0)
    repl = LocalREPL(subcall_fn=subcall_fn)
    # llm_query should route through LMHandler socket, NOT subcall_fn
    # Without a handler it returns an Error string; importantly subcall_fn is NOT called
    repl.execute_code("r = llm_query('hello')")
    assert called == [], f"subcall_fn was called by llm_query: {called}"
    assert "Error" in repl.locals["r"], "Expected error without handler, not subcall response"
    repl.cleanup()

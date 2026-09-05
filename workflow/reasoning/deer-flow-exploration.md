DeerFlow 2.0 exploration — what ELAI can steal
Source: F:\.ELAI_workflow\docs\.source\deer-flow-main
Read date: 2026-06-19. All claims grounded in files read this session (paths below).

=====================================================================
0. IMPORTANT FRAMING — this is NOT the old planner-graph DeerFlow
=====================================================================
README.md:16-17 + 571-581 state DeerFlow 2.0 is a ground-up rewrite that shares NO code with v1.
The classic v1 "planner -> researcher -> reporter -> reflection" LangGraph with src/graph/, src/agents/,
coordinator/planner/reporter nodes DOES NOT EXIST in this checkout (no src/ tree at all; Glob src/**/*.py = 0 files).
v2 is a Claude-Code-style "super agent harness": ONE lead agent + dynamically-spawned sub-agents + a middleware
stack + progressively-loaded skills + a sandbox/filesystem + long-term memory. Deep research is now just a SKILL
(skills/public/deep-research/SKILL.md), not a hardcoded graph. So "steal" here = steal the HARNESS architecture and
the context-engineering tricks, not a research DAG. This is actually MORE relevant to ELAI than v1 would have been,
because ELAI is itself a harness with an FSM + sub-agents.

Tech base (verified backend/packages/harness/pyproject.toml): Python >=3.12, langchain>=1.2.15, langgraph>=1.1.9,
langchain-anthropic/openai/deepseek/google-genai, langchain-mcp-adapters>=0.2.2, tiktoken, langfuse, FastAPI gateway,
Next.js frontend. Built on LangChain v1 create_agent + AgentMiddleware. MIT licensed (README.md:758).

=====================================================================
1. TOP-LEVEL ARCHITECTURE
=====================================================================
Entry point: backend/packages/harness/deerflow/agents/lead_agent/agent.py::make_lead_agent (ARCHITECTURE.md:57).
It is a LangGraph graph factory returning langchain.agents.create_agent(model, tools, middleware, system_prompt,
state_schema=ThreadState). There is no bespoke node graph — control flow is: model -> tools -> model loop, with
ALL the interesting behavior implemented as MIDDLEWARE that hooks before_agent / before_model / after_model /
wrap_model_call. The "graph" is the standard ReAct loop; the harness is the middleware chain + the tools + the prompt.

Runtime: FastAPI Gateway (app/gateway/app.py) embeds the agent runtime and exposes a LangGraph-COMPATIBLE
/api/langgraph/* surface (so LangGraph SDK clients work) but does NOT run a separate LangGraph server
(ARCHITECTURE.md:53-67). SSE streaming, checkpointing, thread state. Also an embedded in-process DeerFlowClient
(client.py) for library use (README.md:694-718). IM channels (telegram/slack/feishu/etc) call the same gateway.

ThreadState (ARCHITECTURE.md:129-145, thread_state.py): extends LangGraph AgentState with sandbox, artifacts,
thread_data {workspace,uploads,outputs}, title, todos, viewed_images.

Middleware chain (lead_agent/agent.py::build_middlewares, ordered):
  ThreadData -> Uploads -> Sandbox -> DanglingToolCall -> ToolErrorHandling -> DynamicContext -> SkillActivation
  -> Summarization -> Todo(plan_mode) -> TokenUsage -> Title -> Memory -> ViewImage(vision) -> DeferredToolFilter
  -> SubagentLimit(subagent) -> LoopDetection -> [custom] -> SafetyFinishReason -> Clarification(last).
Order is load-bearing and documented inline (agent.py:260-269, factory.py:164-177).

Tools come from 3 sources (ARCHITECTURE.md:191-218): built-ins (present_files, ask_clarification, view_image, task),
configured (web_search/web_fetch/bash/read_file/write_file/str_replace/ls), MCP (langchain-mcp-adapters,
stdio/SSE/HTTP). get_available_tools() merges them; skill allowed-tools policy filters them.

=====================================================================
2. CORE ABSTRACTIONS (key files, one line each)
=====================================================================
agents/lead_agent/agent.py — make_lead_agent: graph factory, middleware assembly, tracing-at-root invariant.
agents/lead_agent/prompt.py — the whole system-prompt template + subagent/skill/clarification/citation sections.
agents/factory.py — create_deerflow_agent: config-free SDK factory; feature-flag -> middleware assembly with @Next/@Prev ordering.
agents/thread_state.py — ThreadState schema (sandbox/artifacts/thread_data/todos/...).
agents/middlewares/dynamic_context_middleware.py — inject date+memory as <system-reminder> to keep system prompt STATIC for prefix-cache.
agents/middlewares/summarization_middleware.py — DeerFlow summarization w/ SKILL-RESCUE: preserves recently-loaded skill bundles from compression.
agents/middlewares/loop_detection_middleware.py — 2-layer (hash-of-toolcalls + per-tool-type frequency) loop breaker; warn then hard-stop.
agents/middlewares/skill_activation_middleware.py — /skill-name slash activation: deterministically injects full SKILL.md for one turn.
agents/middlewares/clarification_middleware.py — intercepts ask_clarification, stops the run for human input.
agents/middlewares/subagent_limit_middleware.py — truncates >N parallel task() calls per turn.
agents/middlewares/safety_finish_reason_middleware.py — suppress tool exec when provider safety-terminated.
agents/middlewares/dangling_tool_call_middleware.py — patch missing ToolMessages before model sees history (strict-provider recovery).
agents/middlewares/token_usage_middleware.py + subagents/token_collector.py — token accounting incl subagent attribution to dispatch step.
subagents/executor.py — SubagentExecutor: isolated-context sub-agent run, async/background, cooperative cancel, timeout, token collect.
subagents/registry.py + config.py + builtins/ — subagent type registry (general-purpose, bash), per-type config (tools/skills/max_turns/model).
agents/memory/updater.py — LLM-driven long-term memory: fact extraction, dedupe, confidence threshold, correction/reinforcement hints.
agents/memory/storage.py + prompt.py — memory JSON store (user/history/facts) + the MEMORY_UPDATE_PROMPT.
skills/loader.py + storage + types — progressive skill system: SKILL.md frontmatter (name/description/allowed-tools), metadata-only in prompt.
tools/builtins/tool_search.py — deferred-tool discovery: hide MCP tool schemas behind a searchable catalog until promoted (token saver).
config/*.py — pydantic config per feature (summarization, loop_detection, memory, subagents, tool_search, sandbox, ...).
sandbox/ + community/aio_sandbox/ — Local vs Aio(Docker/k8s) sandbox providers; virtual /mnt/user-data path mapping.
community/{searxng,tavily,exa,serper,brave,ddg,firecrawl,jina,browserless,infoquest,image_search}/ — pluggable search/crawl providers.

=====================================================================
3. THE "DEEP RESEARCH" CONTROL FLOW (as it actually exists in v2)
=====================================================================
There is no dedicated research graph. Research = lead agent + deep-research SKILL + sub-agents. Flow:
(a) DECOMPOSE/DELEGATE/SYNTHESIZE is a PROMPT pattern, not code (prompt.py::_build_subagent_section, lines 236-361).
    Subagent mode prompt forces the model to: count sub-tasks in thinking; if >N, BATCH into turns of <=N parallel
    task() calls; synthesize after all batches. HARD limit N (default 3) enforced both in prompt AND by
    SubagentLimitMiddleware (excess task() calls silently discarded).
(b) Each task() spawns a SubagentExecutor (subagents/executor.py) with ISOLATED context (its own SystemMessage +
    its own skills loaded as conversation items, _load_skill_messages:394-423), its own tool allow/deny list,
    its own max_turns (recursion_limit), timeout, cooperative cancel via cancel_event checked at astream boundaries.
    Sub-agents return only their final AIMessage text to the lead — the lead never sees sub-agent intermediate context
    (README.md:669-676 "Isolated Sub-Agent Context"). This is context isolation = the key scaling trick.
(c) The deep-research SKILL.md (skills/public/deep-research/SKILL.md) is the methodology: Phase1 broad exploration ->
    identify dimensions; Phase2 deep dive per dimension (multiple phrasings, web_fetch full content, follow refs);
    Phase3 diversity matrix (facts/examples/expert/trends/comparisons/criticisms); Phase4 synthesis checklist with
    explicit "if any NO, keep researching". Strong temporal-awareness section (use <current_date>, month+day for
    "today", never year-only when day precision needed).
(d) Human-in-the-loop / replan: ClarificationMiddleware + ask_clarification tool. Prompt enforces CLARIFY->PLAN->ACT
    (prompt.py:380-447): the model MUST call ask_clarification BEFORE work on missing_info/ambiguous/approach_choice/
    risk_confirmation/suggestion; the middleware then HALTS the run and surfaces to the user. Todo/plan_mode
    (TodoMiddleware) is the live re-planning surface (write_todos, exactly one in_progress).
(e) Termination: ReAct natural stop + recursion_limit (per-subagent max_turns) + LoopDetection hard-stop +
    SafetyFinishReason. No fixed iteration count; the model decides "research sufficient" via the skill's checklist.

=====================================================================
4. PROMPT ENGINEERING (notable, stealable techniques)
=====================================================================
Location: agents/lead_agent/prompt.py (SYSTEM_PROMPT_TEMPLATE:364-560) + each skill's SKILL.md + per-middleware inline prompts.
Notable techniques:
 - STATIC system prompt for prefix-cache reuse: the system prompt is identical across users/sessions; date+memory
   are injected per-turn as a hidden <system-reminder> HumanMessage instead of baked into the system prompt
   (dynamic_context_middleware.py docstring + agent.py:302-306). Frozen-snapshot: the reminder takes the first user
   message's ID so add_messages replaces in place and the prefix never changes turn-to-turn. Midnight-crossing handled.
 - Citation enforcement (prompt.py:478-539): MANDATORY [citation:TITLE](URL) inline after any web-sourced claim +
   a Sources section with plain markdown links; explicit WRONG/RIGHT examples; distinguishes inline-citation format
   from sources-list format.
 - Clarification-first protocol with 5 typed scenarios + ✅/❌ enforcement bullets (prompt.py:380-447).
 - Subagent orchestration prompt with COUNT->PLAN BATCHES->EXECUTE->REPEAT->SYNTHESIZE and worked examples (236-361).
 - Todo discipline prompt: complete-immediately, exactly-one-in-progress, don't-use-for-<3-steps (agent.py:158-255).
 - str_replace over write_file for edits; chunked append for long writes (prompt.py:546-553) — diff-mode token saving.
 - Skill self-evolution prompt (prompt.py:168-181): after a task, create/patch a skill if it took 5+ tool calls,
   you overcame non-obvious errors, or the user corrected you and the fix worked.
 - Memory update prompt (memory/prompt.py): structured JSON output {user, history, newFacts, factsToRemove},
   with correction/reinforcement hints raising confidence (updater.py:397-420).

=====================================================================
5. TOOL / INTEGRATION LAYER
=====================================================================
 - Search/crawl providers (community/*): SearXNG, Tavily, Exa, Serper, Brave, DuckDuckGo (ddgs), Firecrawl, Jina,
   Browserless, InfoQuest (BytePlus), image_search. All thin langchain tool wrappers. ELAI already has SearXNG.
 - MCP: full langchain-mcp-adapters MultiServerMCPClient (stdio/SSE/HTTP), OAuth client_credentials/refresh_token,
   mtime-invalidated tool cache (ARCHITECTURE.md:267-303).
 - Deferred tool search (tools/builtins/tool_search.py): MCP tool SCHEMAS are withheld from the model binding;
   model sees only names in <available-deferred-tools> and must call tool_search(query) to promote a tool's schema
   before use. Catalog is immutable, sha256-hashed; promotion lives in graph state. Query grammar: "select:a,b",
   "+required terms", plain regex. MAX_RESULTS=5. This is a direct port of the Claude-Code/Anthropic tool-search idea
   to cut token cost when many MCP tools are configured.
 - Sandbox: virtual /mnt/user-data/{uploads,workspace,outputs} + /mnt/skills mapped to per-thread host dirs;
   Local provider (file tools yes, host bash off by default) vs Aio provider (Docker/k8s isolated bash).
 - ACP agents: can invoke external coding agents (codex, claude_code) via agent-client-protocol as sub-workers
   into a separate /mnt/acp-workspace (prompt.py:706-727).

=====================================================================
6. RANKED "STEALABLE FOR ELAI" LIST (highest leverage first)
=====================================================================
Difficulty = port effort into ELAI's Rust orchestrator/providers + Python sidecar. "Have?" = ELAI equivalent today.

S1. STATIC-PROMPT + DYNAMIC-CONTEXT-AS-SYSTEM-REMINDER for prefix-cache reuse.
    What: keep the system prompt byte-identical across turns/sessions; inject volatile context (date, memory,
    per-turn reminders) as a hidden first-user-message <system-reminder> with a frozen ID, never into the system prompt.
    Where: dynamic_context_middleware.py; agent.py:302-306.
    Why ELAI wants it: ELAI's cloud lane is Anthropic with prompt caching; ELAI memory notes [project_efficiency_*]
    already flag that the production cloud path ships ZERO cache markers and that input caching is the big cost lever.
    This is the concrete mechanism: stop mutating the prefix. Highest $ leverage of anything here.
    Difficulty: LOW-MED (Rust prompt-assembly change in crates/providers anthropic path + a "reminder" injection seam).
    Have? Partial — caching is wired but the static-prefix discipline is not enforced; this is the missing discipline.

S2. PROGRESSIVE / METADATA-ONLY SKILL LOADING + DEFERRED MCP TOOL SEARCH.
    What: put only skill NAME+DESCRIPTION (not full body) in the prompt; load full SKILL.md only on match or /slash.
    Same idea for MCP tools: withhold schemas, expose a tool_search() to promote on demand.
    Where: prompt.py::get_skills_prompt_section + skill_activation_middleware.py + tools/builtins/tool_search.py.
    Why ELAI wants it: ELAI already has a heavy skill/rule/agent ecosystem and an MCP fleet; this is the canonical
    way to keep the context lean and token-sensitive-model-friendly (the local Qwen lane). ELAI's harness already
    does conditional rule-loading by trigger — this generalizes it to a runtime catalog + grammar.
    Difficulty: MED (catalog + search grammar are simple; the win is the discipline + a tool-gating seam).
    Have? Partial — ELAI lazy-loads rules by trigger (conditional-routing.md) but does not defer MCP tool schemas;
    a tool_search analog would cut tokens when many MCP servers are attached.

S3. TWO-LAYER LOOP DETECTION (hash-of-toolcalls + per-tool-type frequency) with warn-then-hard-stop.
    What: hash each tool-call multiset (order-independent, salient-arg keying with line-bucketing for read_file);
    warn at 3 identical, hard-stop (strip tool_calls -> force final answer) at 5; PLUS per-tool-type freq warn@30/stop@50
    with per-tool overrides. Warning injected at wrap_model_call AFTER tool messages to keep provider pairing valid.
    Where: loop_detection_middleware.py (read in full).
    Why ELAI wants it: ELAI already has anti-spiral (skill + rule) at the ORCHESTRATION layer, but nothing at the
    RUNTIME tool-call layer for a single agent/sub-agent loop. This is a deterministic, cheap, provider-safe backstop
    that complements anti-spiral. The pairing-safety reasoning (warn as trailing HumanMessage, not mid-stream) is
    grounded and worth copying verbatim into the Rust runtime_client.
    Difficulty: MED (port the hashing + sliding-window LRU + injection-point logic to Rust; logic is self-contained).
    Have? Partial — anti-spiral is plan-level; runtime per-loop detection is absent. Good complement, not a dup.

S4. SUMMARIZATION WITH SKILL-RESCUE (don't compress away recently-loaded skills/tools).
    What: standard threshold summarization, BUT before compressing, identify AIMessage+ToolMessage bundles that
    LOADED a skill file (read_file under /mnt/skills) and RESCUE the newest N (budgeted by count + tokens) from the
    summary so the agent keeps its loaded skills after compaction. Also preserves the hidden date/memory reminders.
    Where: summarization_middleware.py (read in full).
    Why ELAI wants it: ELAI memory [project_hplan_track9] documents that LOCAL context management is broken
    (recover_stubs re-inflates trims; 85% gate ignores ELAI_LOCAL_MAX_CONTEXT). The skill-rescue idea — "when you
    compact, keep the high-value loaded artifacts" — maps directly onto ELAI's stub/skeleton reload problem but
    inverts it correctly (keep what's load-bearing instead of re-inflating everything). Strong design input for the
    H-50 context-policy work.
    Difficulty: MED-HIGH (ELAI's compaction lives in conversation.rs/session_dag.rs; bundle-detection heuristic ports).
    Have? Partial/contested — ELAI HAS compaction but the SELECTIVE-PRESERVATION policy is the missing piece.

S5. ISOLATED-CONTEXT SUB-AGENT EXECUTOR with cooperative cancel + timeout + token attribution.
    What: each sub-agent gets a fresh SystemMessage, its own skills loaded as conversation items, its own tool
    allow/deny + max_turns + timeout; runs on an isolated persistent event loop; cancel_event checked at stream
    boundaries; returns ONLY final text to parent; token usage collected and attributed back to the dispatch step.
    Where: subagents/executor.py (full), token_collector.py, subagents/config.py + registry.py.
    Why ELAI wants it: ELAI's orchestration is the DASHBOARD/CLI agent-dispatch layer (Claude Code Task tool) — that
    part ELAI already does well and arguably BETTER (off-agent auditor, wiring gates). BUT for the SIDECAR/local lane,
    ELAI has no in-process sub-agent fan-out; the cooperative-cancel-at-stream-boundary + per-subagent token
    attribution + isolated-skill-loading patterns are directly portable to a Rust/sidecar sub-task runner.
    Difficulty: HIGH (concurrency model is Python-asyncio-specific; the PATTERNS port, the code does not).
    Have? Mostly — ELAI's main orchestration already isolates sub-agent context (sub-agents GC on return per CLAUDE.md).
    Steal only the token-attribution-to-dispatch-step and cooperative-cancel-boundary details.

S6. LLM-DRIVEN LONG-TERM MEMORY with confidence/dedupe/correction-reinforcement signals.
    What: after a conversation, an LLM emits {user, history, newFacts, factsToRemove}; facts are dedup'd by
    casefolded content, gated by a confidence threshold, capped (keep top-confidence), correction signals force
    category="correction" conf>=0.95, reinforcement -> "preference"/"behavior" conf>=0.9; upload-event sentences
    are scrubbed (session-scoped, don't persist). Robust JSON extraction from messy model output.
    Where: memory/updater.py (full) + memory/prompt.py + storage.py.
    Why ELAI wants it: ELAI has RAG memory-bank + the external memory dir, but the AUTO fact-extraction-with-
    confidence-and-correction-weighting loop is a cleaner model than free-form memory writes. The correction/
    reinforcement confidence bump is a genuinely good idea for ELAI's gotchas.md / feedback capture.
    Difficulty: MED (it's a prompt + a JSON-apply function; ELAI memory-writer could adopt the schema + dedupe + conf).
    Have? Different shape — ELAI uses RAG-indexed markdown via memory-writer; DeerFlow uses a structured fact store
    with confidence. The confidence/correction/dedupe ALGORITHM is the steal, not the storage.

S7. DEEP-RESEARCH SKILL methodology (4-phase + diversity matrix + synthesis checklist + temporal awareness).
    What: the SKILL.md text itself — broad->dimensions->deep-dive->diversity-matrix->synthesis-gate, with the
    "if any checklist answer is NO keep going" termination and the month/day temporal-precision table.
    Where: skills/public/deep-research/SKILL.md (full).
    Why ELAI wants it: ELAI's deep-research harness prompt can absorb the diversity matrix + synthesis gate +
    temporal-precision table near-verbatim; these are prompt content, zero code.
    Difficulty: LOW (prompt text adoption into ELAI's researcher/deep-research skill).
    Have? Partial — ELAI deep-research exists; this sharpens the methodology prompt. Cheapest win on the list.

S8. CLARIFY->PLAN->ACT clarification gate as a runtime middleware that HALTS the run.
    What: ask_clarification tool + middleware that stops execution and surfaces a typed question (missing_info/
    ambiguous/approach_choice/risk_confirmation/suggestion) to the user BEFORE work starts.
    Where: prompt.py:380-447 + clarification_middleware.py.
    Why ELAI wants it: ELAI has AskUserQuestion at the harness level (grill/cook), but not a sidecar-lane runtime
    gate that forces clarify-before-act for the LOCAL agent. Useful for the autonomous local lane.
    Difficulty: MED. Have? Partial (harness-level only, not in the Rust/local runtime loop).

S9. DANGLING-TOOL-CALL recovery + SAFETY-FINISH-REASON suppression (strict-provider robustness).
    What: patch missing ToolMessages before the model sees history (OpenAI/Moonshot strict tool_call_id validators);
    when a provider safety-terminates, suppress tool execution instead of firing on garbage.
    Where: dangling_tool_call_middleware.py + safety_finish_reason_middleware.py + tool_call_metadata.py.
    Why ELAI wants it: ELAI runs Qwen on a sidecar and multiple cloud providers; these are exactly the
    multi-provider tool-call-sequence-validation bugs ELAI will hit. Defensive, low-glory, high-value.
    Difficulty: MED (port to runtime_client message-normalization seam). Have? Unknown/likely partial.

NOT WORTH STEALING / ELAI ALREADY BETTER:
 - The LangGraph/LangChain create_agent + AgentMiddleware substrate itself — ELAI is Rust-core by design; do NOT
   adopt the framework, only the ideas. Porting middleware-as-classes wholesale is wrong for ELAI.
 - The FastAPI gateway / Next.js frontend / IM channels (telegram/slack/feishu/wecom/dingtalk) — ELAI has Tauri+Svelte
   and no chat-app distribution goal. Skip entirely.
 - The Docker/k8s Aio sandbox provisioner — ELAI's execution model + RunPod path is its own; not aligned.
 - The off-agent verification / wiring-enforcement / freeze-the-gate discipline — ELAI is FAR ahead here; DeerFlow has
   NOTHING like it (no auditor-as-separate-root, no production-caller gate, no wire-dark allowlist). ELAI's done-gate
   doctrine is a strict superset. DeerFlow's TodoMiddleware "only mark complete when fully accomplished" prompt is a
   pale shadow of ELAI's gates.
 - Generic search providers (Tavily/Exa/Serper/etc) — ELAI already has SearXNG + Brave fallback; adding more providers
   is config, not architecture.
 - The skill self-evolution prompt is a NICE idea but collides with ELAI's skill-import-policy (skillspector gate) and
   "no unsolicited refactoring" — adopt only behind ELAI's import/audit gate, never auto.

TOP 3 RECOMMENDED to act on (all grounded against ELAI's own memory notes):
 1. S1 static-prefix discipline (directly targets the documented zero-cache-markers cost bug).
 2. S3 runtime two-layer loop detection (complements anti-spiral at a layer ELAI doesn't cover).
 3. S4 skill-rescue summarization policy (directly informs the broken local-context-management / H-50 work).

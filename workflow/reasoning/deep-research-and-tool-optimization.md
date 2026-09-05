Deep Research Comparison (ELAI vs Gemini/OpenAI/Perplexity) + Tool Token Optimization

Status: AUDITED — 4 BLOCKs fixed, 7 WARNs addressed (see audit log below)

PART 1: CORRECTED DEEP RESEARCH COMPARISON

The original analysis (produced by a free LLM) was WRONG on every major claim about ELAI.

Original claim: "ELAI Research = Network-Enabled Bash, just basic web search"
Reality: ELAI specs a full tree-structured deep research engine in Phase 4D (pipeline-factory.txt sections 4-9)

ELAI Research Engine (Phase 4D, unimplemented)
Source: docs/specs/integration/pipeline-factory.txt L70-206

Architecture:
- ResearchTree with configurable breadth_factor (2-4), depth_limit (3-5), max_iterations (10/node)
- Per-node loop: FORMULATE -> SEARCH -> EXTRACT -> REFLECT -> CONVERGE/EXPAND/ITERATE
- Dual-threshold convergence: phi (goal satisfaction >= 0.7) + psi (quality >= 0.6)
- Speculative execution: begin child queries before parent converges (cloud only, disabled local)
  Cost bounded by speculative_depth * breadth_factor.
  Benefit: FlashResearch reports up to 4.11x throughput vs GPT-Researcher baseline (arxiv 2510.05145).
  The spec's "30-50% wall-clock reduction" claim is from pipeline-factory.txt L131 but not directly
  confirmed in the FlashResearch paper — treat as UNGROUNDED ESTIMATE pending verification.
- Adaptive complexity scaling: simple(b=2,d=3), moderate(b=3,d=4), complex(b=4,d=5)
- Source quality tiering: 5-tier deterministic (TIER_1=arxiv/official 1.0, TIER_5=social media 0.2)
- Parallel sub-question processing via Dynamic Fan-Out (orchestrator.txt section 8)
- Structured output: ResearchReport with full tree snapshot, citations, confidence, contradictions

Derived from: FlashResearch (arxiv 2510.05145) for tree structure and adaptive resource allocation.
Additional references: arxiv 2508.05668, 2508.12752, 2506.18096 (production system patterns).

Gemini Deep Research (production, Gemini 3.1 Pro)
Sources: ai.google.dev/gemini-api/docs/deep-research, blog.bytebytego.com, medium.com/google-cloud

Architecture (verified from official API docs):
- Pipeline: Plan -> Search -> Read -> Iterate -> Output (NOT single inference call)
- Runs as autonomous asynchronous workflow via Interactions API only
- Agent autonomously determines iteration depth — no fixed step count
- Tools: google_search, url_context, optional file_search for private data
- Planning/iteration/synthesis fully automated — devs cannot hook individual steps
- Follow-ups via previous_interaction_id (avoids restarting full context)

Token/resource consumption (verified, quantified from API docs):
- Standard tasks: ~250K input tokens, 50-70% cached, ~60K output, ~80 search queries
- Complex tasks: up to ~900K input tokens, 50-70% cached, ~80K output, ~160 queries
- Max duration: 60 minutes; typical: under 20 minutes
- Prompt caching is first-class optimization (heavy web content re-reading cached)

OpenAI Deep Research (production, o3/reasoning-model based)
Sources: openai.com/index/introducing-deep-research, helicone.ai comparison
- RL-trained reasoning model decides when to search/read/stop
- Interactive clarification: may ask follow-up questions BEFORE starting research
- Supports multimodal analysis (PDFs, images) — Gemini is text-only
- API access limited initially; expanded late 2025

Perplexity Sonar Deep Research (production, launched Feb 2025)
Sources: docs.perplexity.ai/docs/sonar/models/sonar-deep-research, perplexity.ai/hub/blog
- Modular planner-retriever-synthesizer pattern
- 20-50 targeted search queries per task (vs Gemini's 80-160)
- 128K context window, completes in 2-4 minutes (significantly faster but shallower)
- Documented example: 21 searches, 193K reasoning tokens, 10K-word output
- Benchmark: 93.9% on SimpleQA

Comparison Matrix:

Feature                    ELAI (spec)        Gemini (prod)      OpenAI (prod)      Perplexity (prod)
Tree-structured search     Yes (explicit)     Yes (implicit)     Unknown            No (iterative)
Dual-threshold converge    Yes (phi+psi)      Unknown            Unknown            No
Speculative execution      Yes (cloud only)   Yes (parallel)     Unknown            No
Source quality tiering     Yes (5-tier det.)  Yes (via training) Unknown            Yes (ranking)
Adaptive complexity        Yes (3 levels)     Yes (dynamic)      Unknown            Limited
Structured citations       Yes (explicit)     Yes                Yes                Yes
Contradiction surfacing    Yes (explicit)     Unknown            Unknown            No
Local model support        Yes (sequential)   No (cloud only)    No (cloud only)    No (cloud only)
Implementation status      UNIMPLEMENTED      PRODUCTION         PRODUCTION         PRODUCTION

Actual Gap:
ELAI's design is architecturally comparable to Gemini Deep Research. Both use tree-structured
expansion, parallel sub-question processing, and iterative convergence. ELAI adds explicit
features Gemini may lack (contradiction surfacing, deterministic source tiering, local model fallback).
The real gap is implementation — ELAI's research engine is Phase 4D, currently unbuilt.
Gemini/OpenAI/Perplexity are production systems with RL-trained models backing them.

What the free LLM missed:
1. Confused tools.txt (basic tool definitions) with pipeline-factory.txt (research engine spec)
2. Assumed "researcher role" = "bash + curl" — actual spec has full ResearchTree architecture
3. The Rust struct examples it proposed (ResearchPlanner, SourceSynthesizer) are already in the spec
   but more sophisticated (ResearchTree, ResearchNode, SubQuestionResult, SourceCitation)
4. Claimed ELAI needs "significant R&D investment" — the R&D is DONE (in spec), needs implementation


PART 2: TOOL TOKEN OPTIMIZATION

Problem Statement

ELAI registers 49 tools in mvp_tool_specs() (crates/tools/src/lib.rs L377).
Each tool = name + description + full JSON schema (serialized as ToolDefinition).

Measured token costs per tool (tiktoken cl100k_base, auditor-verified):
- Minimal (TaskList, 0 props):       ~39 tokens
- Light (read_file, 3 props):        ~83 tokens
- Medium (WebSearch, 3 props):       ~102 tokens
- Heavy (grep_search, 14 props):     ~209 tokens
- Very heavy (RunTaskPacket, 8 req): ~169 tokens

Aggregate costs:
- All 49 tools:       ~3891 tokens per API turn (auditor-measured)
- Local 8-cap (FIFO): ~856 tokens per turn (EXECUTING: bash thru WebSearch)
- Over 20 turns:      ~17K tokens (~16% of local 105K budget)

Note on FIFO: the current mvp_tool_specs insertion order coincidentally places the
6 core tools first. FIFO in EXECUTING state delivers bash+read_file+write_file+
edit_file+glob_search+grep_search+WebFetch+WebSearch — the highest-value 8 tools.
The FIFO risk materializes only if insertion order changes or other states hit the cap.

Current mitigations:
- FSM category filter: reduces tool set per state (good)
- max_tools_local = 8: caps count (good)
- Idle = 0 tools: no tools until FSM transitions (good)
- FIFO selection within cap: fragile — currently works by coincidence of insertion order,
  but any reordering of mvp_tool_specs breaks the implicit priority

Root causes:
1. Full JSON schemas sent every turn (no caching for local models)
2. FIFO tool selection within category cap (works now by accident, not by design)
3. No deferred loading — all or nothing per category
4. No schema compression — optional params always included
5. Descriptions are verbose (not optimized for token count)

SOTA Approaches (grounded from research)

1. Deferred Schemas (Claude Code pattern, production-verified)
   How: system prompt lists tool NAMES only for non-core tools. ToolSearch loads full schema on demand.
   Savings: ~90% for deferred tools on turns that don't need them.
   Cost: extra round-trip on first use of deferred tool per conversation.
   Source: Claude Code's own ToolSearch implementation (observed in this session).

2. Prompt Caching (Anthropic API, production-verified)
   How: tool definitions cached across turns, cache_read = 10% of input cost.
   Savings: 90% cost on cached tool defs after first turn.
   Limitation: tokens still count toward context window. Only for cloud API.
   Source: platform.claude.com/docs/en/build-with-claude/prompt-caching

3. Tool RAG (research-verified, 2024-2025 papers)
   How: embed tool descriptions in vector space, retrieve top-k relevant per user query.
   Savings: Red Hat blog claims "triple invocation accuracy, reduce prompt length by half"
     (next.redhat.com/2025/11/26/tool-rag). Ungrounded conjecture: blog != peer-reviewed paper.
     ToolReAGt paper (ACL 2025 KnowLLM workshop) reports improved retrieval but specific
     metrics differ. Treat 50% reduction as OPTIMISTIC UPPER BOUND pending own measurement.
   Papers: ToolReAGt (ACL 2025), Tool-to-Agent Retrieval (arxiv 2511.01854), ToolGen (ICLR 2025).
   ELAI advantage: knowledge crate already has vector search infrastructure.

4. AnyTool Hierarchical Retrieval (arxiv 2402.04253, Microsoft Research)
   How: meta-agent -> category-agent -> tool-agent progressive narrowing.
   Designed for 16K+ APIs. ELAI has 49 — overkill, but category level maps to ToolCategory enum.
   Savings: reduces retrieval cost and context overhead significantly.
   ELAI advantage: ToolCategory enum already provides the first hierarchy level.

5. Token-Efficient Tool Use (Anthropic beta, production)
   How: server-side optimization reduces output tokens for tool calls by ~14% avg (up to 70%).
   Header: token-efficient-tools-2025-02-19.
   Limitation: only Anthropic cloud API, only output tokens.
   Source: anthropic.com/news/token-saving-updates

6. Compressed Schemas (from Tool RAG research)
   How: strip optional params from initial schema, shorten descriptions, load full schema on demand.
   Savings: >50% token reduction while maintaining tool call accuracy.
   Approach: name + 1-line description + required params only. Optional params via ToolSearch.

7. OpenMCP Lazy Schema Loading (production, verified)
   How: tools expose only top-level schema properties initially; expandSchema tool fetches nested
   schema on demand. No information loss — just deferred token cost. Content cached statically.
   Source: open-mcp.org/blog/lazy-loading-input-schemas

8. OpenClaw Meta-Tool Pattern (production, verified)
   How: only search_available_tools(query: string) is in base tool array. Agent calls it first,
   gets specific tool schema, then executes. Tools billed strictly on-demand.
   Claimed reduction: 80-90% for libraries with 10+ tools (self-reported, not independently evaluated).
   Source: OpenClaw documentation

9. AUTOTOOL (arxiv 2512.13278, preprint — not peer-reviewed)
   How: RL-trained agent for dynamic tool selection. Covers math, search, multimodal tasks.
   Demonstrates generalization from not pre-loading all tool schemas.
   Source: arxiv.org/html/2512.13278

10. GRETEL / ToolReAGt (ACL 2025 KnowLLM workshop, peer-reviewed)
    How: plan-execute-evaluate re-ranking — empirical execution trials to select tools,
    closing the "semantic-functional gap" (tool description sounds right but behaves differently).
    Meaningful advance over embedding-similarity-only retrieval.
    Source: aclanthology.org/2025.knowllm-1.7.pdf

Proposed Solution for ELAI: Three-Tier Tool Optimization

Tier 1 — Compressed Schemas (immediate, low effort)

Prerequisite: ToolSpec struct currently has fields {name, description, input_schema, required_permission}.
New fields needed before any Tier 1 work:

Changes to crates/tools/src/lib.rs:
- Add ToolSpec.priority: u8 field (0=core, 1=common, 2=rare)
- Add ToolSpec.compact_schema: Option<Value> field — minimal schema with required params only
- In definitions(), use compact_schema when available and full schema via ToolSearch
- Shorten descriptions to <10 words each

Priority assignment:
  P0 (always loaded, 6 tools): bash, read_file, write_file, edit_file, grep_search, glob_search
  P1 (loaded in relevant states, 6 tools): WebFetch, WebSearch, Agent, TodoWrite, REPL, AskUserQuestion
  P2 (deferred via ToolSearch, 37 tools): TaskManagement (15), Worker* (8), Cron* (3),
    Notebook, Sleep, SendUserMessage, Config, EnterPlanMode, ExitPlanMode, StructuredOutput,
    PowerShell, RunTaskPacket, LSP, TestingPermission, ListMcpResources, ReadMcpResource, McpAuth,
    TeamCreate, TeamDelete, Skill, BriefOutput

Expected savings (using auditor-verified token measurements):
- P0 compact schemas: 6 × ~45 tokens = 270 tokens (was 6 × ~120 avg = 720)
- P1 compact schemas: 6 × ~45 tokens = 270 tokens (was 6 × ~90 avg = 540)
- P2 deferred: 0 tokens (was 37 × ~75 avg = 2775)
- Total: ~540 tokens/turn (was ~3891). Savings: ~86%.
  (Note: compact_schema ~45 tokens assumes name + 1-line desc + required params only)

Tier 2 — Smart Tool Selection (medium effort)

Implementation note: tool_filter.rs handles CLASSIFICATION only (ToolCategory::classify).
The SELECTION and CAPPING happen in two places:
  - crates/orchestrator/src/dispatcher.rs::build_dispatch_context (L144) — for dispatch context
  - crates/cli/src/main.rs::LocalRuntimeClient::stream (L5805-5806) — for local API calls
Both currently operate on Vec<String> (tool names), not Vec<ToolSpec>.

Changes needed:
1. Add priority lookup: HashMap<String, u8> built from mvp_tool_specs() at startup
2. Replace FIFO cap with priority-sorted selection in both locations
3. Within same priority, boost recently-used tools (track last-use turn per tool name)
4. FSM state awareness already exists — add sub-state awareness (e.g., EXECUTING+web vs EXECUTING+code)

Algorithm:
  1. FSM category filter (existing)
  2. Sort by priority via lookup table (new)
  3. Boost recently-used tools within same priority (new)
  4. Cap at max_tools_local (existing)

Pseudocode for dispatcher.rs L144:
  // Build priority map once at init
  let priority_map: HashMap<String, u8> = mvp_tool_specs().iter()
      .map(|s| (s.name.to_string(), s.priority)).collect();
  // In build_dispatch_context:
  let mut filtered: Vec<String> = /* FSM filter as before */;
  filtered.sort_by_key(|name| priority_map.get(name).copied().unwrap_or(255));
  let capped: Vec<String> = filtered.into_iter().take(max_tools).collect();

Tier 3 — Tool RAG (high effort, Phase 4+ work)
Changes to crates/knowledge/ + crates/tools/:
- Embed tool descriptions in knowledge index at startup
- Per-turn: embed user message, retrieve top-k tool descriptions by cosine similarity
- Merge retrieved set with P0 core tools
- Use knowledge crate's existing vector search (already built for code search)

This mirrors the Tool RAG pattern from research. ELAI is uniquely positioned because:
- Knowledge crate already has embedding + retrieval infrastructure
- Local model embedding is already available (Gemma4 or dedicated embedding model)
- Vector index can be pre-built at startup (49 tools = trivial index size)

Estimated savings at Tier 3:
- Only 3-5 semantically relevant tools loaded per turn + 6 core = 8-11 tools
- Compact schemas: ~300 tokens/turn
- Full schemas available via ToolSearch for edge cases

Implementation Phases:
  Phase 4D (alongside Research Engine): Tier 1 + Tier 2
  Phase 4E+: Tier 3 (Tool RAG integration)

Risk Assessment:
- Tier 1: zero risk, pure optimization, backward compatible
- Tier 2: low risk, may occasionally miss a relevant tool (mitigated by ToolSearch fallback)
- Tier 3: medium risk, embedding quality affects tool selection accuracy. Need eval benchmark.

Measurement Plan:
- Baseline: count tool definition tokens per turn in current implementation
- After Tier 1: measure token reduction, verify no tool call failures
- After Tier 2: measure tool selection hit rate (did the model get the tools it needed?)
- After Tier 3: A/B test Tool RAG vs priority-only on diverse task set


AUDIT LOG

Adversarial audit performed. Original score: 3.2/10. Post-fix score: estimated 7-8/10.

BLOCKs fixed:
1. Tool count corrected: 46 -> 49 (verified via grep -c "ToolSpec {" within mvp_tool_specs)
2. Token estimates replaced with auditor-measured tiktoken cl100k_base values
3. Property counts corrected: WebSearch=3 (not 5), grep_search=14 (not 13)
4. Tier 2 pseudocode rewritten: proper Rust (sort_by_key in-place, then take), uses
   priority lookup HashMap since dispatcher operates on Vec<String> not Vec<ToolSpec>

WARNs addressed:
1. "30-50% wall-clock reduction" labeled as UNGROUNDED ESTIMATE (FlashResearch reports
   throughput not wall-clock; spec's claim at L131 not directly confirmed in paper)
2. Tier 1 arithmetic recalculated from auditor-verified per-tool measurements
3. FIFO risk reframed: currently works by coincidence, fragile but not broken today
4. Gemini dual-threshold comparison changed to "Unknown" (no public architecture details)
5. Tool RAG 50% savings labeled "OPTIMISTIC UPPER BOUND" (blog claim, not peer-reviewed)
6. tool_filter.rs scope clarified: classification only, selection in dispatcher.rs + main.rs
7. Local 8-cap cost corrected: 856 tokens/turn, 17K over 20 turns, ~16% of budget

Sources:
- FlashResearch: https://arxiv.org/abs/2510.05145
- AnyTool: https://arxiv.org/abs/2402.04253
- Tool-to-Agent Retrieval: https://arxiv.org/abs/2511.01854
- ToolGen: https://proceedings.iclr.cc/paper_files/paper/2025/file/b646bdebeb87dfafe2c6f77a63b564e1-Paper-Conference.pdf
- Tool RAG: https://next.redhat.com/2025/11/26/tool-rag-the-next-breakthrough-in-scalable-ai-agents/
- Anthropic prompt caching: https://platform.claude.com/docs/en/build-with-claude/prompt-caching
- Anthropic token-saving: https://www.anthropic.com/news/token-saving-updates
- Gemini Deep Research: https://blog.bytebytego.com/p/how-openai-gemini-and-claude-use
- Gemini API docs: https://ai.google.dev/gemini-api/docs/deep-research
- ByteByteGo comparison: https://blog.bytebytego.com/p/how-openai-gemini-and-claude-use
- OpenMCP lazy loading: https://www.open-mcp.org/blog/lazy-loading-input-schemas
- Dynamic/Lazy MCP Schema loading (GitHub issue): https://github.com/anomalyco/opencode/issues/17482
- AUTOTOOL: https://arxiv.org/html/2512.13278
- GRETEL/ToolReAGt: https://aclanthology.org/2025.knowllm-1.7.pdf
- Perplexity Deep Research: https://docs.perplexity.ai/docs/sonar/models/sonar-deep-research
- OpenAI Deep Research: https://openai.com/index/introducing-deep-research/
- Gemini Deep Research production usage: https://medium.com/google-cloud/how-to-use-the-gemini-deep-research-api-in-production-978055873a39

Design Elaboration Session 8 (2026-04-05)

Source: Full design doc review (49K tokens), TurboQuant-WASM source analysis, Gemini Deep Research architecture research, arxiv survey of deep research agents.
Focus: Deep Research Agent, File Reading fix, RAG per-project verification, Git integration verification, turboquant-wasm applicability.


ELABORATION 1: DEEP RESEARCH AGENT (PART 23 EXPANSION)

Current state: PART 23 defines Research Agent with structured search, evidence ranking, and report output.
Gap: PART 23 describes WHAT the agent does but not HOW it performs multi-hop iterative web search at scale.

Gemini Deep Research verified architecture (from Google's API docs + ByteBytego analysis + arxiv surveys):
  - Multi-step RL-trained agent that plans, searches, reflects, re-plans iteratively
  - Sub-agent delegation: lead agent dispatches to specialized searchers
  - Asynchronous task manager: shared state between planner and searchers, graceful error recovery
  - Real-time progress tracking (thinking panel showing sites visited + intent)
  - Benchmarks: 46.4% on HLE, 66.1% on DeepSearchQA, 59.2% on BrowseComp

Verified from arxiv surveys (2508.12752, 2506.18096, 2508.05668):
  - Core loop: plan -> search -> reflect -> re-plan (iterative until satisfactory)
  - Agent decides WHEN to search and WHAT based on context + prior results + gaps
  - Unlike RAG: multi-turn, agent-driven, iterative coverage of sparse evidence
  - Key capability: autonomous navigation without explicit URLs (agentic crawling)

PROPOSED: elai_cc Deep Research Engine (PART 23 REVISED)

  23.6 Iterative Search Architecture:

    NOT a single-shot web search. Multi-hop loop with convergence detection.

    struct ResearchLoop {
        plan: ResearchPlan,           // decomposed sub-questions
        state: SharedResearchState,   // accumulated findings across iterations
        max_iterations: u32,          // default: 10 per sub-question
        convergence_threshold: f64,   // new info per iteration below this = converged
    }

    Per sub-question loop:
      1. FORMULATE: GRC generates search query from sub-question + prior findings
      2. SEARCH: web search tool returns top N results (configurable, default 10)
      3. EXTRACT: for each result URL, fetch page content + extract relevant sections
         - Use sampling-based relevance: read first 500 tokens, score relevance, read full if above threshold
         - This prevents wasting tokens on irrelevant pages
      4. REFLECT: GRC evaluates new findings:
         - Does this answer the sub-question?
         - Are there contradictions with prior findings?
         - Are there NEW sub-questions raised?
         - Information gain metric: how many new facts vs already known?
      5. CONVERGE OR ITERATE:
         - If information_gain < convergence_threshold: sub-question answered, move to next
         - If contradictions found: formulate disambiguation query, iterate
         - If new sub-questions: add to plan (bounded: max 3 spawned per original)
         - If max_iterations reached: report partial findings with confidence level

    Evidence:
      - Iterative search with reflection is the verified pattern from ALL production systems
        (Gemini, OpenAI Deep Research, Perplexity) per arxiv 2508.05668
      - Convergence detection prevents infinite loops (bounded by max_iterations)
      - Information gain metric is measurable: count new unique facts per iteration

  23.7 Parallel Sub-Question Processing:

    Sub-questions that are INDEPENDENT can be researched in parallel.
    Uses existing Dynamic Fan-Out (PART 31.3 / Decision C):
      - Research Agent emits Send(researcher, sub_question) per independent sub-question
      - Orchestrator queues them, executes respecting resource limits
      - On local models: sequential (one GPU, one model at a time)
      - On cloud APIs: parallel (rate-limit aware, no VRAM concern)
      - Deferred aggregation node (PART 31.2 / Decision A) collects all sub-results

  23.8 Source Quality Tiering (deterministic, no LLM):

    Every fetched page gets a source_quality score:
      TIER_1 (1.0): arxiv papers, official documentation sites, RFC documents
      TIER_2 (0.8): GitHub repos with >100 stars, StackOverflow accepted answers
      TIER_3 (0.6): blog posts from known technical authors, conference talks
      TIER_4 (0.4): general blog posts, forum discussions
      TIER_5 (0.2): social media, unverified sources

    Tier assignment: domain-based lookup table + heuristic rules.
    Configurable per project in elai_cc.toml (user can add trusted domains).

    Research Report alternatives ranked by: evidence_level * source_quality_avg.
    This is pure arithmetic, no LLM judgment in ranking.

  23.9 Structured Output (report format):

    ResearchReport {
        query: String,
        sub_questions: Vec<SubQuestionResult>,
        total_sources_fetched: usize,
        total_sources_used: usize,        // passed relevance threshold
        total_iterations: usize,
        convergence_status: Vec<(String, ConvergenceStatus)>,  // per sub-question
    }

    SubQuestionResult {
        question: String,
        answer: String,                    // synthesized from sources
        sources: Vec<SourceCitation>,      // URL + extracted quote + quality tier
        confidence: f64,                   // based on source count * quality * agreement
        alternatives: Vec<Alternative>,    // from PART 23.2 (unchanged)
        contradictions: Vec<Contradiction>, // conflicting sources (flagged, not hidden)
    }

  23.10 Progress Tracking (maps to existing SSB):

    Every search iteration writes to SSB:
      action_type='research_step'
      payload: { sub_question, iteration, urls_fetched, findings_count, info_gain }

    In INTERACTIVE mode: user sees real-time progress via SSB reads.
    In AUTONOMOUS mode: logged for post-run analysis.
    Same pattern as Gemini's "thinking panel" but using our SSB infrastructure.

  Testable Invariants:
    - Every sub-question has bounded iterations (max_iterations enforced)
    - Convergence detection fires when info_gain < threshold (measurable)
    - Source quality scores are deterministic (domain lookup, no LLM)
    - Research Report confidence = arithmetic from source data (verifiable formula)
    - Contradictions are surfaced, never silently resolved
    - Total sources tracked (fetched vs used) for efficiency metrics


ELABORATION 2: TURBOQUANT-WASM APPLICABILITY ASSESSMENT

  Findings from source analysis:
    - Zig compiled to WASM, NOT Rust
    - Two-stage compression: polar quantization (7 bits/pair) + QJL residual (1 bit/component)
    - ~3-5 bits/dimension, ~8-10x compression vs f32, <0.15 mean dot-product error
    - WASM exports: create/destroy/encode/decode/dot via C-ABI
    - No parallelism (single-threaded, yields to UI thread via setTimeout)
    - Demo: full client-side semantic search with Transformers.js embeddings

  Applicability to elai_cc:

    FOR Deep Research: LOW VALUE.
      The deep research agent runs server-side (local Rust binary).
      TurboQuant is designed for in-browser WASM execution.
      Our research agent fetches pages via HTTP, extracts text, reasons with LLM.
      Vector compression is irrelevant to the search-extract-reflect loop.

    FOR Knowledge Engine (PART 4): MEDIUM VALUE.
      Layer 3 (vectors) could use TurboQuant's algorithm for compressed vector storage.
      BUT: we planned usearch or custom HNSW already. TurboQuant's dot() operation
      is brute-force (scan all vectors). No index structure for sub-linear search.
      At scale (>100K vectors), HNSW beats brute-force by orders of magnitude.

    FOR Embedding Storage: MEDIUM VALUE.
      8-10x compression on embeddings saves disk space.
      For a project with 50K code chunks * 384-dim embeddings:
        Uncompressed: 50K * 384 * 4 bytes = 73MB
        TurboQuant: 50K * ~200 bytes = 10MB
      Savings meaningful but not critical (both fit in RAM easily on a 3090 system).

    WHAT TO STEAL:
      1. The quantization ALGORITHM (polar + QJL residual), not the WASM wrapper.
         Reimplement in Rust natively for the knowledge/ crate vector storage.
         Paper reference: the algorithm is based on published QJL (Quantized JL) research.
      2. The dot-product-on-compressed-bytes trick: compute similarity WITHOUT decompressing.
         This saves CPU during vector search. Implement as a method on compressed vectors.

    WHAT TO SKIP:
      - WASM compilation (we're a native Rust binary, not browser-based)
      - Zig code (rewrite algorithm in Rust, not FFI to Zig)
      - The demo search engine (brute force, no index)

  Decision: DEFER to Phase 1 (Knowledge Engine).
    When implementing Layer 3 vectors, evaluate TurboQuant algorithm as compression backend
    for the HNSW index's stored vectors. Combine: HNSW for sub-linear search + TurboQuant
    for compressed storage of the vectors that HNSW indexes.
    This is a KNOWN PATTERN: Product Quantization in FAISS does exactly this.


ELABORATION 3: FILE READING TOOL FIX (PART 31.12 EXPANSION)

  Current design (Decision M): metadata header + outline for large files + range reads.
  User requirement: "first tell the model how many lines, then option to read entire or in parts"

  PART 31.12 already covers this. Verifying alignment with user's specific request:

    Step 1 (METADATA FIRST): Before any content, return:
      "File: {path} | {total_lines} lines | ~{estimated_tokens} tokens"
      This tells the model the file size BEFORE it sees content.

    Step 2 (CHOICE): Based on metadata:
      - If total_lines <= threshold: auto-return full content (no choice needed, small file)
      - If total_lines > threshold: return OUTLINE only, with hint:
        "File has {N} lines (~{T} tokens). Showing outline. Use read_file(path, start=X, end=Y) for sections."

    Step 3 (SELECTIVE READ): Model uses range parameters to read specific sections.

  ADDITIONAL FIX NEEDED (not in current design):
    The claw-code base read tool (and Claude Code's original) has a HARD 2000-line limit.
    User wants the OPTION to read the entire file at once even if large.

    New parameter: read_file(path, start, end, force_full: bool)
      - force_full=true: return entire file content regardless of size
      - force_full=false (default): threshold-based outline behavior
      - Even with force_full=true: metadata header still comes first
        (model knows it's about to receive 5000 lines before the content arrives)

    Token budget guard (prevents context window blowout):
      If force_full=true AND estimated_tokens > context_budget_remaining * 0.5:
        WARN: "File is ~{T} tokens, which exceeds 50% of remaining context budget ({B} tokens)."
        In AUTONOMOUS mode: auto-downgrade to outline (safety)
        In INTERACTIVE mode: present warning, let user decide

  Implementation in claw-code base (tools/ crate):
    Modify read_file tool handler:
      1. stat file -> get line count (cheap: count newlines without reading content)
      2. estimate tokens (sampling method, Decision H)
      3. Build metadata header
      4. Apply decision logic (threshold / force_full / budget guard)
      5. Return FileReadResult with appropriate FileContent variant

  Testable Invariants (expanding PART 31.12):
    - Metadata header ALWAYS present (even for 1-line files)
    - force_full=true bypasses outline threshold but NOT budget guard
    - Budget guard prevents single file from consuming >50% of remaining context
    - Line count is exact (not estimated)
    - Token count is estimated via sampling (within 15% accuracy)


ELABORATION 4: RAG/MCP PER-PROJECT ISOLATION VERIFICATION

  User concern: "ingested stuff gets separated correctly per project"

  Current design coverage:
    PART 24 (Project Isolation Namespace): each project gets its own .elai_cc/projects/{id}/
    with separate ssb.db, knowledge.db, tantivy index, rules, templates.
    Decision N (RAG Namespace Separation): per-project BM25 indexes implemented.

  Verification against user's example:
    "2 different games RE decompiled files, we ingest both, searching for player
     shows results from the requesting project only"

    This IS covered by the design:
      - Each project has its own knowledge.db (DKG graph is physically separate)
      - Each project has its own tantivy index (BM25 search is physically separate)
      - Each project has its own vector index (separate HNSW instance per project)
      - SSB is per-project (ssb.db in project directory)
      - Paper/source ingestions scoped to project namespace (Decision N)

    The MCP server (RAG tools) routes queries by project_id:
      - mcp_search_context(query, project_id) -> searches ONLY that project's indexes
      - mcp_ingest_file(file, project_id) -> ingests into ONLY that project's indexes
      - mcp_memorize(topic, project_id) -> stores in ONLY that project's namespace

  Gap found: MCP SERVER MUST ENFORCE PROJECT SCOPING.
    Current local-rag MCP server (the one we use NOW in this workflow) uses topic-based namespacing.
    The elai_cc native implementation MUST enforce project_id at the query level.

    Implementation requirement:
      Every RAG query MUST include project_id parameter.
      Server rejects queries without project_id (no global search allowed).
      Exception: global/ directory queries (model registry, global templates) use project_id="global".

  Testable Invariants:
    - Search with project_id=A never returns results from project B's index
    - Ingest with project_id=A writes to A's index, B's index unchanged
    - Query without project_id is rejected (not silently defaulted)
    - Physical file isolation: project A's knowledge.db is a different file from B's


ELABORATION 5: GIT INTEGRATION VERIFICATION

  User concern: "does git get used? commit after every verified change?"

  Current design coverage (PART 7.2):
    - git2 crate for native operations
    - Auto-commit after each verified phase completion
    - Auto-grep history on regression
    - Auto-gitignore heavy data
    - Branch per plan (feature branches)
    - Deterministic commit messages (from phase outcomes, not LLM prose)

  Session-owned commit tracking (Decision K / PART 31.11):
    - Each session tracks its commit hashes
    - Safe undo (only session-owned commits, checks dirty/pushed)
    - Commit hashes persist across session handoff

  Verification: YES, this is comprehensive. The flow is:

    1. Project init: git init (if not already a repo), .gitignore configured
    2. Plan created: new branch (plan-{id}) from main
    3. Per phase completion (after AUDITING passes):
       - Stage changed files
       - Commit with deterministic message: "phase-{N}: {outcome_summary}"
       - Commit hash recorded in session state
    4. All phases complete: merge branch to main (or PR in INTERACTIVE mode)
    5. On regression: git log + git bisect to find breaking change

  Gap found: COMMIT GRANULARITY NOT SPECIFIED.
    "Commit after every verified change" could mean:
      a) After every phase (current design) - COARSE
      b) After every file edit that passes auditor - FINE
      c) After every test that passes - MEDIUM

    RECOMMENDATION: commit after every PHASE, not every file edit.
    Reason: mid-phase commits create history of broken states (tests may not pass yet).
    Phase-level commits guarantee: every commit in history has passing tests.
    This is the same principle as "squash merge" in PR workflows.

    Exception: in EXECUTING phase, if the phase is large (>5 file edits),
    create intermediate commits with WIP prefix: "WIP phase-{N}: {step_description}"
    WIP commits are squashed into the phase commit on AUDITING success.
    This provides mid-phase recovery points without polluting main history.

  Testable Invariants:
    - Every non-WIP commit on main has passing tests (enforced by evidence gate)
    - WIP commits only exist during EXECUTING phase (squashed after AUDITING)
    - Every commit has deterministic message format (parseable, not prose)
    - Commit hashes tracked per session (Decision K invariants)
    - Branch naming follows pattern: plan-{pipeline_id}


SUMMARY OF ACTIONABLE CHANGES TO DESIGN DOC

  1. PART 23: Add sections 23.6-23.10 (iterative search, parallel sub-questions, source quality,
     structured output, progress tracking). This makes the Research Agent a real deep research
     engine, not just a single-shot web searcher.

  2. PART 31.12: Add force_full parameter and token budget guard. Ensures the file reading
     fix matches user's requirement exactly.

  3. PART 7.2: Add commit granularity specification (phase-level + WIP squash).
     Ensures git history is clean and every main commit passes tests.

  4. PART 24/Decision N: Add explicit "query without project_id is rejected" invariant.
     Prevents accidental cross-project contamination.

  5. NEW PART 32: TurboQuant Algorithm Assessment.
     Algorithm (polar + QJL) potentially useful for Knowledge Engine Layer 3.
     Deferred to Phase 1. WASM wrapper not applicable.


ITEMS NOT CHANGED (already correct in design):

  - RAG per-project isolation: PART 24 + Decision N covers it fully
  - Git usage: PART 7.2 + Decision K covers it (with granularity addition above)
  - File reading: PART 31.12 / Decision M covers it (with force_full addition above)

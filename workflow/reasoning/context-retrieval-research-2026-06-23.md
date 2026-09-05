CONTEXT + CODE-RETRIEVAL RESEARCH SYNTHESIS
Date: 2026-06-23
Session: s-ctx-retrieval-2026-06-23
Purpose: steal-evaluation digest for a later J-plan authoring session (target J-72+).
  Each candidate below carries a verdict (NEW / EXTENSION / ALREADY-SHIPPED /
  REJECT), the ELAI subsystem + code path it touches (codegraph-grounded
  2026-06-23 against live .elai_cc), the primary-source interface map path, and
  the saved source on drive. NO plan files authored this run (synthesis only).
  Reading order for the author: this file, then the cited interface maps under
  workflow/papers/, then compose against docs/thefinalglance/ELAI-FEATURE-MAP.md.

ORIGIN INPUTS
  User list (10): lucek out-of-context; anthropic context-engineering; chroma
    context-rot; HF positional-encoding; arXiv 2512.24601 (RLM) + repo;
    MCompassRAG repo; SproutRAG repo; arXiv 2606.22807 (KaLM-Reranker) +
    EvoEmbedding repo (paper 2606.21649).
  Plus: F:\Users\user\Downloads\deep-research-report(2).md (Jun-23, "State of
    the Art in Code Retrieval, Visual-Document RAG, retrieval->navigation").
  Report-cited works ingested as extra primary sources: CodeRAG system
    (2509.16112), Repoformer (2403.10059), RepoGraph (2410.14684), CMU
    repo-search (2502.07067), CodeScaleBench (Sourcegraph blog). Agentless
    (2407.01489) + CodeRAG-Bench (2406.14497) already mapped from prior runs,
    referenced not re-ingested.

FLEET: 8 sonnet paper-ingestor agents, cap 3, RAG indexing disabled (server
  broken; interface-map files are ground truth, not RAG).

==================================================================
PART 1 — REPORT-FIDELITY WARNINGS (read FIRST; the report mis-cites)
==================================================================
The deep-research report is directionally strong but has citation-fidelity
defects. Do NOT let a J-plan cite a report number without primary verification.

W1. CORE-Bench NOT FOUND (the report's #1 cited benchmark). The report leans
  its headline recommendation on "CORE-Bench": base Qwen3-Embedding-8B 20.3
  nDCG@10 / 48.0 Recall@100 on issue-to-edit localization rising to 32.8 / 66.4
  after in-domain SFT. Agent A6 ran 25+ searches / 15+ fetches and found NO
  paper matching that name AND those metrics (2409.11363 "CORE-Bench" is a
  computational-reproducibility benchmark, unrelated; CoREB 2605.04615, CoreCode
  2507.05281, SweRank 2505.07849 all ruled out). VERDICT: the specific
  numbers are UNVERIFIED / possibly report-synthesized. The LEVER they support
  (in-domain retriever adaptation) is independently corroborated by three primary
  sources (see C1), so the lever stands; the CORE-Bench numbers must NOT be cited.
  Stub: workflow/papers/core-bench-interface.md (search audit trail).

W2. RepoGraph "+32.8% relative on average" (report) != paper Table 2 (+8.6%
  relative for Agentless). Agent A7 flagged the discrepancy. Use the paper's
  verbatim Table-2 numbers (repograph-interface.md), not the report's.

W3. Several report recommendations are ALREADY-SHIPPED in ELAI (the report
  evaluated ELAI from an outdated stack description): conditional HyDE (shipped,
  hyde.rs skips short queries), SPLADE/learned-sparse drop (already DEFERRED),
  dataflow edges (shipped in symbol_graph), symbol graph (SCIP/DKG surpasses
  RepoGraph). See PART 3 for the per-item already-shipped marks.

==================================================================
PART 2 — CONVERGENT META-FINDING (the run's headline)
==================================================================
Three INDEPENDENT primary sources converge on the report's single highest-ROI
lever: stop adding retrieval gadgets, ADAPT ONE RETRIEVER IN-DOMAIN.
  - MCompassRAG: distill an LLM teacher (once, offline, ~$10-20 for ~20K
    judgments) into a ~2M-param 3-layer scoring head on top of a FROZEN ONNX
    embedder; at inference it REPLACES the cross-encoder reranker and ranks at
    embedding-model latency with teacher-grade quality, ZERO LLM calls.
    (workflow/papers/mcompassrag-interface.md)
  - EvoEmbedding (2606.21649): length-weighted InfoNCE contrastive fine-tune
    (tau=0.1); the EvoTrain-180K schema (messages + meta.turns +
    meta.evidence_turns) maps directly onto ELAI's issue-PR-edit history.
    (workflow/papers/evoembedding-interface.md, .claude/papers/2606.21649.html)
  - CodeRAG system (2509.16112, EMNLP'25): a distilled Qwen3-0.6B LoRA reranker
    from LLM preference pairs, 0.23s total query cost; ReccEval EM 47.48 vs DraCo
    39.99 on Qwen2.5-Coder-7B. (workflow/papers/coderag-system-interface.md)
  COMMON SHAPE THAT FITS ELAI: a ONE-TIME, OFFLINE, GPU fine-tune (contrastive
  or distillation) on ELAI's OWN supervision (issue/PR/edit + agent traces) ->
  EXPORT the encoder or the tiny scoring head to ONNX -> CPU inference runtime
  UNCHANGED. This is the J-72 keystone. The training step is out-of-product
  (one GPU box, hours); the inference artifact stays ONNX-CPU.
  HARD CAVEAT (anti-slopsquat on our own claim): ELAI ships NO training infra
  in-product. This is a new offline pipeline, not a runtime change. The
  ONNX-export path is proven for the encoder/scoring-head variants
  (MCompassRAG, EvoEmbedding contrastive); it is NOT available for
  KaLM-Reranker's online GPU decoder or EvoRAG's dynamic memory-queue (REJECT
  those, see PART 3).

==================================================================
PART 3 — STEAL CANDIDATES (verdict + ELAI seam + source)
==================================================================

C1. IN-DOMAIN RETRIEVER ADAPTATION  [VERDICT: NEW — KEYSTONE]
  What: offline fine-tune/distill one retriever on ELAI's own issue/PR/edit +
    agent-trace supervision; export to ONNX; CPU inference unchanged. Two
    sub-variants: (a) distill teacher -> tiny scoring head replacing the
    cross-encoder (MCompassRAG, zero-LLM inference); (b) contrastive fine-tune
    the code embedder itself (EvoEmbedding loss + EvoTrain schema).
  ELAI seam: crates/knowledge ONNX embedders (GraphCodeBERT 768d code + Granite
    384d text) + ms-marco cross-encoder reranker (phase-1C/1G). COMPOSES with
    PLANNED J-18 (CodeRankEmbed INT8 swap) — same embedder seam; author should
    fold C1 into/around J-18, not duplicate it.
  Sources: mcompassrag-interface.md, evoembedding-interface.md,
    coderag-system-interface.md; report PART "Prioritized what-to-change" #1.
  Risk: needs a one-time GPU-train pipeline (out-of-band); needs an ELAI gold
    supervision set (issue/PR/edit pairs + agent traces). Done-gate = ELAI
    MEASURED MRR/Recall delta on ELAI's own gold set, never a paper number
    (W1). Verbatim loss/recipe extracts live in the interface maps.

C2. RETRIEVAL-ABSTENTION GATE  [VERDICT: EXTENSION]
  What: decide whether to retrieve AT ALL before firing the cascade (Repoformer
    2403.10059): self-assessment P(<cc>) threshold (T=0.15 function, 0.20
    line/API), trained into the LM as a multi-task head, >70% speedup in dense
    settings, abstention precision >80%.
  ELAI gap (codegraph-confirmed): no should_retrieve gate (search returned
    none); ELAI ALWAYS retrieves when in a retrieval tier. BUT ELAI already has
    abstention machinery for ANSWERS — crates/orchestrator/src/abstention_score.rs
    + conformal_calibration.rs (Abstain enum, decide_low_score_abstains, agent_pool
    low_score_routes_to_abstain) — and FAST/STANDARD/DEEP tier routing
    (query_class). The steal repurposes that abstention infra to gate RETRIEVAL.
  ELAI seam: a gate in crates/knowledge/src/pipeline.rs BEFORE the BM25/ONNX
    cascade fires; signal either from the sidecar (needs <cc>/abstain special
    tokens — Qwen2.5-Coder lane) OR a cheap calibrated heuristic reusing
    abstention_score. Config (T per task type) near the tier router.
  Source: workflow/papers/repoformer-interface.md, .claude/papers/2403.10059.html.
  Note: genuine capability gap; the report calls this a more durable lever than
    HyDE/multi-query.

C3. DATAFLOW AS A RETRIEVAL/RANKING CHANNEL  [VERDICT: EXTENSION]
  What: CodeRAG (2509.16112) uses an AST dataflow dependency walk as a third
    retrieval path; +4.32 EM solo (df+s 44.21 vs s 39.89). 
  ELAI status (codegraph-confirmed): ELAI HAS dataflow EDGES already —
    crates/orchestrator/src/symbol_graph/dataflow.rs (DataflowAnalyzer,
    DataflowEdgeKind, dataflow_impact) + crates/knowledge/src/lib.rs:239
    DataFlowEdge — but they feed IMPACT/blast-radius (DataflowImpactComputed
    SSB), not the retrieval RRF blend. E-40 already added a 5th RRF symbol-graph
    ranker (PARTIAL, flag unallowlisted). So this is wiring the EXISTING
    dataflow graph as a retrieval/ranking channel, NOT new infrastructure.
  ELAI seam: knowledge RRF cascade + the E-40 5th-ranker seam; reuse
    symbol_graph::dataflow edges as a candidate source pre-rerank.
  Source: workflow/papers/coderag-system-interface.md.

C4. SYMBOL-GRAPH AS A GENERATION-TIME RETRIEVAL ACTION  [VERDICT: EXTENSION]
  What: expose the code graph as a mid-task, agent-callable retrieval action
    that serializes a k-hop ego-graph as flat context (RepoGraph 2410.14684);
    production evidence it pays (CodeScaleBench / Sourcegraph: P@5 0.140->0.478,
    file recall 0.127->0.277, cost $0.73->$0.51, exec -38% from wiring nav tools
    as agent actions).
  ELAI status: the graph + traversal exist and SURPASS RepoGraph's construction
    (SCIP-resolved DKG, PageRank repo-map, impact_radius/call_flow/symbol_context
    tools) — RepoGraph CONSTRUCTION is ALREADY-SHIPPED-equivalent. The gap
    (codegraph-confirmed: no symbol_graph_search symbol) is that these live in
    the planning/pre-edit layer, NOT wired as a generation-time retrieval action
    during LLM reasoning.
  ELAI seam: expose ego-graph serialization as an MCP/task_runner retrieval
    action. COMPOSES with PLANNED J-38 (retrieval agentic loop / mid-task
    hybrid_search + SpIDER graph-neighbor fusion) — author should fold C4 into
    J-38, it is the same mid-task-callable seam.
  Sources: workflow/papers/repograph-interface.md (verdict: not-new-graph,
    new-interface-layer), workflow/papers/codescalebench-interface.md.

C5. MEASURE-THEN-SIMPLIFY THE OVER-FUSED STACK  [VERDICT: EXTENSION / measure-first]
  What: the report's core attack — a BM25 + two-dense + learned-sparse + graph-
    rank + wide-RRF + reranker stack is over-fused; the lift comes from 2-3
    components. CMU 2502.07067 (primary): a full commit-history+code multi-stage
    pipeline does NOT significantly beat a single good code reranker at depth 100
    (CodeReranker@100 MAP 0.288 vs full 0.307, ~6.6% avg, <2% on popular repos)
    at ~7x runtime. The reranker is the high-value stage; do NOT remove it.
  ELAI status: ELAI's stack IS the over-fused shape (feature-map L57-135: BM25 +
    dual ONNX dense + PageRank + RRF k=60 + cross-encoder + E-40 5th SCIP ranker).
  ELAI action: an ABLATION HARNESS — behind flags, drop PageRank and/or the
    second dense encoder from the RRF blend, measure MRR/Recall@5 delta on ELAI's
    gold set (report predicts <7% MAP loss on popular repos). Demote graphs from
    ranking to navigation (C4).
  HARD GUARD (house rule): "no removal or simplification without explicit user
    confirmation." So C5 is a MEASURE-FIRST experiment producing the delta; any
    actual prune is gated on the measured number AND user sign-off. Author this
    as an instrumentation/ablation unit, not a deletion.
  Sources: workflow/papers/cmu-repo-search-interface.md,
    workflow/papers/arxiv-2406-14497-coderag-bench.md (existing), report PART
    "Attack the design".

C6. CONTEXT-ROT -> CONTEXT-BUDGET CEILING + PLACEMENT  [VERDICT: EXTENSION (grounds J-34/H-50)]
  What: empirical degradation as input grows — focused 300-token prompt beats
    113K full prompt (LongMemEval); middle-position chunks lose 30%+ vs head/tail
    ("lost in the middle"); degradation onset ~5-7K words for the studied tasks.
  ELAI seam: J-34 (empirical context calibration) + H-50 (context-lifecycle,
    selective-reload, PLANNED). Concrete knobs: cap retrieval chunk count;
    place top chunks at head/tail never middle; prefer a focused pre-filtered
    prompt over full-context dump (validates H-50 selective-reload).
  HARD CAVEAT (both report and agent flag): the numbers are on 18 FRONTIER
    models, NOT Qwen2.5-Coder-7B local. ELAI must MEASURE its own curve on the
    sidecar before setting a threshold — import the METHOD, not the numbers.
  Sources: workflow/papers/chroma-context-rot-interface.md,
    lucek-out-of-context-interface.md, anthropic-context-engineering-interface.md;
    saved: .claude/papers/{chroma-context-rot,lucek-out-of-context,
    anthropic-context-engineering}.md.

C7. RLM RECURSIVE-CONTEXT + _compact_history  [VERDICT: EXTENSION (compaction) / DEFER (REPL-context)]
  What: RLM (2512.24601) stores the full prompt as a Python REPL variable; the
    model sees only metadata and writes Python to slice/query context; recursion
    via rlm_query under a depth cap; _compact_history triggers at 85% of the
    window, summarizes progress preserving intermediate numerics, truncates
    history to [system + initial + summary + "continue from summary"].
  ELAI seam: _compact_history is a validated reference for H-50's LOCAL
    compaction policy (ELAI's local lane currently has flat-100K auto-compact
    that never fires <32K — feature-map H-50 note). The verbatim summarization
    prompt is in rlm-interface.md, directly adaptable to the Qwen sidecar 32K
    window. EXTENSION of H-50.
  DEFER: the REPL-as-context architecture (model writes code to navigate
    context) is a larger bet that overlaps ELAI's sandbox/tool loop; interesting
    but heavy and uncertain fit — record as a rider, do NOT author now.
  Source: workflow/papers/rlm-interface.md, .claude/papers/2512.24601.html.

C8. SPROUTRAG HIERARCHICAL ATTENTION-TREE INDEX  [VERDICT: NEW — lower priority]
  What: agglomerative max-linkage tree over chunks (function->class->module),
    internal nodes are natural multi-granularity summaries; retrieval beam-
    searches the tree returning leaves AND coarser spans. Has a COSINE FALLBACK
    (replace learned attention with pairwise chunk-embedding cosine) that needs
    no attention output -> ONNX-portable to ELAI today.
  ELAI status: ELAI has flat chunk index + adaptive AST chunking (phase-3K);
    no multi-granularity tree. The cosine-fallback variant is portable.
  Verdict: NEW but lower-ROI than C1-C5; candidate for a later experiment unit,
    not the first wave.
  Source: workflow/papers/sproutrag-interface.md.

REJECTED / ALREADY-SHIPPED LEDGER (record in feature-map ledger when authoring)
  R1. KaLM-Reranker-V1 (2606.22807): online GPU decoder cross-attention at query
    time, NOT ONNX-CPU portable. Matryoshka-pooling compression idea noted but
    the late-interaction-online-decoder breaks ELAI's runtime. REJECT for the
    runtime; revisit only if ELAI ever runs a GPU rerank lane.
    (workflow/papers/arxiv-2606-22807-interface.md)
  R2. EvoRAG memory-queue (dynamic FIFO buffer + LoRA adapter swap + flash-attn
    bf16): not ONNX-exportable. Only the contrastive loss + EvoTrain schema are
    portable (folded into C1). REJECT the runtime architecture.
  R3. HyDE demotion (report rec): ALREADY-SHIPPED — crates/knowledge/src/hyde.rs
    is already conditional (short_query_skips_hyde, four_word_query_skips_hyde,
    should_use_hyde delegation). No action.
  R4. SPLADE / learned-sparse drop (report rec): ALREADY-DEFERRED in ELAI
    (feature-map: SPLADE [DEFERRED], early-DROP conditions defined). Aligned.
  R5. RepoGraph graph construction (2410.14684): ALREADY-SHIPPED-equivalent —
    ELAI SCIP/DKG/PageRank surpasses it. Only the generation-time interface is
    new (-> C4).
  R6. HF positional-encoding (RoPE/NTK/YaRN): BACKGROUND ONLY. Qwen2.5-Coder RoPE
    is baked into weights, no harness knob. NTK-aware base scaling
    (base_new = 10000 * alpha^(d/(d-2))) could extend the sidecar's effective
    window IF server.py exposed rope_scaling — UNVERIFIED, DEFER, not a steal.
    (workflow/papers/hf-positional-encoding-interface.md)
  R7. Visual-document / page-image late-interaction RAG (report section): ELAI
    has E-36 DOCUMENT VISION RAG (PARTIAL, ColQwen2.5 lane, no production caller)
    + PLANNED J-33 MinerU lane. The report's "add visual RAG now" is already an
    ELAI workstream; NOT re-authored here (no agent was spent on ColPali/ViDoRe;
    they are already in ELAI's ledger). Compose into E-36/J-33 if pursued.

==================================================================
PART 4 — PROPOSED J-PLAN SHAPE (for the author; J-72+, FEWEST FILES)
==================================================================
Group by subsystem; compose with existing PLANNED units, do not restate them.
  J-72 [KEYSTONE] In-domain retriever adaptation (C1): offline distillation/
    contrastive fine-tune -> ONNX export -> CPU inference unchanged. EXTENDS
    J-18. Done-gate = ELAI-measured MRR/Recall delta on an ELAI gold set.
  J-73 Retrieval-abstention gate (C2): should_retrieve gate in pipeline.rs
    reusing abstention_score + tier router. New SSB row likely (RetrievalAbstained).
  J-74 Graph-as-retrieval (C3 + C4): wire dataflow + ego-graph as RRF channel
    AND a mid-task agent retrieval action. EXTENDS E-40 + J-38. Fold C4 into J-38
    if the author prefers.
  J-75 Measure-then-simplify ablation harness (C5): flag-gated RRF-component
    ablation + ELAI-gold-set delta report. Removal gated on the number + user.
  Riders (NO new file — fold into existing PLANNED units):
    - C6 context-rot ceiling/placement -> J-34 + H-50.
    - C7 _compact_history recipe -> H-50 local-compaction policy.
    - C8 SproutRAG hierarchical index -> optional later experiment unit.
    - R7 visual RAG -> E-36/J-33.
  Ledger riders to record at authoring: R1 KaLM, R2 EvoRAG-queue, R3 HyDE-shipped,
    R4 SPLADE-deferred, R5 RepoGraph-shipped, R6 positional-encoding-defer.

==================================================================
PART 5 — FILE INDEX (everything used; for the author)
==================================================================
Interface maps (workflow/papers/):
  chroma-context-rot-interface.md, hf-positional-encoding-interface.md,
  lucek-out-of-context-interface.md, anthropic-context-engineering-interface.md,
  rlm-interface.md, mcompassrag-interface.md, sproutrag-interface.md,
  arxiv-2606-22807-interface.md (KaLM-Reranker), evoembedding-interface.md,
  coderag-system-interface.md (2509.16112), core-bench-interface.md (NOT-FOUND stub),
  repoformer-interface.md, repograph-interface.md, cmu-repo-search-interface.md,
  codescalebench-interface.md.
  Pre-existing (referenced, not re-ingested): agentless-2407.01489-interface.md,
  arxiv-2406-14497-coderag-bench.md, repomap-agentless.md.
Saved sources on drive:
  .claude/papers/: 2512.24601.html (RLM), 2606.22807.html (KaLM),
  2606.21649.html (EvoEmbedding), 2509.16112.html (CodeRAG), 2403.10059.html
  (Repoformer), 2410.14684.html (RepoGraph), 2602.17907.html (SoftLTM, MCompass
  dep), codescalebench.md, chroma-context-rot.md, hf-positional-encoding.md,
  lucek-out-of-context.md, anthropic-context-engineering.md.
  docs/.source/: rlm/ (12M), MCompassRAG/ (2.7M), SproutRAG/ (2.9M),
  EvoEmbedding/ (7.9M), repograph/ (cloned by A7).
Report: C:\Users\user\Downloads\deep-research-report(2).md.
ELAI code grounded (codegraph-elai live, 2026-06-23):
  crates/orchestrator/src/symbol_graph/dataflow.rs (DataflowAnalyzer),
  crates/knowledge/src/lib.rs:239 (DataFlowEdge),
  crates/orchestrator/src/abstention_score.rs + conformal_calibration.rs,
  crates/knowledge/src/hyde.rs (conditional HyDE),
  (no should_retrieve / no symbol_graph_search symbol).
  Feature-map retrieval section: docs/thefinalglance/ELAI-FEATURE-MAP.md L55-160.
Handoff: workflow/handoff/context-retrieval-research.md.
Run-state: workflow/sessions/s-ctx-retrieval-2026-06-23/run-state.md.

OUTCOME (2026-06-23, paper-to-plan PHASE 5): authored J-72 in-domain retriever
  adaptation [KEYSTONE, C1] + J-73 retrieval-abstention gate [C2] + J-74
  graph-as-retrieval channel + mid-task action [C3+C4] + J-75 RRF ablation
  harness [C5] + J-76 context-rot calibration + local _compact_history recipe
  [C6+C7; C8 SproutRAG deferred-rider]. Riders R1-R7 + W1/W2 recorded in the
  J-0_INDEX DECLINES + the ELAI-FEATURE-MAP Section-4 ledger. J-0_INDEX +
  J-SUMMARY_INDEX + ELAI-FEATURE-MAP upserted (date 2026-06-23). Plan family
  J now J-1..J-79. ADDS no delta family.

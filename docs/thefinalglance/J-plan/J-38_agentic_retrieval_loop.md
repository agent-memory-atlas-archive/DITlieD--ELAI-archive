# J-38: agentic retrieval loop — mid-task callable hybrid search + SpIDER graph-neighbor fusion + context-sufficiency gating

- Track J (pillar-SOTA recheck extensions) | lever CANT-MISREAD | grounding STRONG (SWE-Explore 2606.07297 / SpIDER 2512.16956 / SweRank 2505.07849 / LocAgent 2503.09089 / GrepRAG 2601.23254 abstracts VERIFIED via arxiv.org WebFetch 2026-06-16; SHIPPED DKG/B-29 + RRF cascade + 3E budget grep-confirmed) | nature BUILD | effort M | WAVE AR-1 | depends J-18 (soft: the hardened HYBRID retriever is what gets exposed; B-29 graph + 3E budget already shipped)

- Premise: J-18 hardens the retrieval substrate (CodeRankEmbed swap + LSP channel + RRF cascade), but the retriever is still invoked as a PRE-TASK pipeline that fills the fixed context-budget slots (RepoMap/CodeContext at context_budget.rs) BEFORE the executor sees any task output. The Turbopuffer "RAG is dead" talk and the grounded benchmarks both point the SAME way: the frontier shift is not hybrid-vs-dense (ELAI already runs the superset hybrid index, verify-p1 HARDEN-not-replace) — it is that retrieval has become a MID-TASK AGENT-CALLABLE LOOP, not a one-shot prefill. SWE-Explore (2606.07297) measures the cost: agents reach strong file-level coverage but only ~15-19% line-level recall, "missing context is the dominant failure mode", and the fix is targeted re-retrieval on symbols discovered DURING reasoning, not a bigger prefill window. LocAgent (2503.09089) hits 92.7% file-level precisely by giving the agent graph-traversal TOOLS it calls iteratively mid-task. ELAI exposes NARROW traversal tools (impact_radius / symbol_context / call_flow, B-29) but NOT the full hybrid retriever as a callable tool, and J-18's agentic-grep leg is the narrow post-retrieval CONFIRM (grep to extend a low-confidence index hit), not a broad mid-task query loop. Three GROUNDED additions, each independently shippable, all ADDITIVE to the pre-task fill (the pipeline amortizes — SweRank 2505.07849: non-agentic retrieve+rerank beats the SWE-Agent grep loop on localization AND cost 44x; the loop never replaces it, it repairs/targets it).

## Steal/build
1. MID-TASK HYBRID_SEARCH TOOL (the keystone leg): expose the J-18-hardened HYBRID cascade as an agent-callable tool `hybrid_search(query, top_k, mode)` in the executor's tool surface (alongside the existing B-29 traversal tools), so the agent issues a targeted retrieval mid-reasoning after observing partial results (read file A -> notice it calls module B -> search for B's behavior). FAST tier is the default (the cross-encoder reranker is OPTIONAL/off on the mid-task path — the ms-marco-MiniLM rerank adds ~190ms/call which compounds over a loop; the B7 reranker-latency critique and J-18's OQ both apply; reserve DEEP+rerank for the pre-task fill and explicit deep queries). This is index-first-loop-to-target (the CORTEXA/DCI pattern J-18 cites for grep, generalized from grep-confirm to a full retrieval query). NOT a replacement for the pre-task slot-fill: the prefill stays (amortized, the "right million" staged narrowing), the loop is the recovery-and-target path the pre-task pipeline structurally cannot provide once the executor is reasoning.
2. SPIDER GRAPH-NEIGHBOR FUSION AT RETRIEVAL TIME: after the hybrid top-K, expand each candidate to its call / import / type-resolution neighbors (depth <= 2) via the SHIPPED SCIP/DKG graph (B-29 traversal + J-18's LSP channel when present), then rerank the expanded set. ELAI has BOTH the structural graph and the hybrid retriever today but does NOT fuse them at retrieval time — the graph is used for pre-edit impact/blast-radius (B-29/J-16), never as a retrieval-time neighbor-expansion signal. Grounded: SpIDER (2512.16956, abstract VERIFIED) improves dense retrieval "by at least 13%" exactly this way; the neighbor lookup is O(1) over the DKG SQLite edges (cheap). Applies to BOTH the pre-task fill and the leg-1 mid-task tool.
3. DETERMINISTIC CONTEXT-SUFFICIENCY SIGNAL: a symbol-coverage gate answering "did the current context cover the symbols named in the task (IntentPair, C-29) and the symbols surfaced during reasoning?" — when coverage is below threshold the executor is signalled to fire a leg-1 `hybrid_search`; when sufficient, it proceeds (gating OVER-retrieval, the other failure mode). Deterministic set-coverage over DKG-resolved symbols, the LLM NEVER adjudicates sufficiency (TD-3); bounded by a max-rounds cap (no unbounded re-query, the anti-spiral analog). Grounded: SWE-Explore reports a strong correlation (r~0.92, full-paper) between ranked context quality at a fixed budget and downstream repair success — context quality is MEASURABLE mid-task, so the gate has a real signal to read.
- EXCLUDED: replacing the pre-task pipeline-fill (it amortizes and beats pure agentic grep on cost+localization — SweRank 44x; the loop is ADDITIVE, never a swap — no removal without confirmation); an LLM-judged sufficiency verdict (TD-3 — deterministic symbol-coverage only); unbounded mid-task re-query (the sufficiency gate + max-rounds cap bound it); MCTS/tree-search over retrieval branches (SWE-Search 2410.20285 is a different, heavier axis — deferred, not in scope); making the reranker mandatory on the loop (latency — FAST-tier default, W-3); a SECOND embedding model or index (this leg reuses J-18's index geometry unchanged).

## Existing ground truth (grep-confirmed; re-grep at edit, W-7/W-9)
- VECTOR/BM25/RRF HYBRID cascade [WIRED] crates/knowledge/ (feature map 1.1) — the cascade J-18 hardens and this leg exposes as a tool. SearchMode {HYBRID,SPARSE_ONLY,BM25_ONLY} + tier routing FAST/STANDARD/DEEP already exist (E-17/phase-3K) — leg 1 rides the FAST tier, does not author a new search mode.
- SYMBOL-LEVEL IMPACT API [WIRED] B-29: 4 traversal tools (impact_radius/call_flow/symbol_context/list) already agent-callable + DKG SQLite edges — leg 2's neighbor expansion consumes these edges; leg 1 adds the missing hybrid_search tool ALONGSIDE them (does not re-author the traversal tools).
- CONTEXT BUDGET ALLOCATOR [WIRED] context_budget.rs 8 slots incl RepoMap/CodeContext — the pre-task fill site; leg 3's sufficiency gate reads what landed in these slots vs the task's symbol set.
- INTENT-PAIR [WIRED] C-29 IntentPair — the task's declared symbol set leg 3 measures coverage against (reuse; do not re-author).
- J-18 retrieval hardening [PLANNED] — leg 1 exposes J-18's HARDENED cascade (CodeRankEmbed slot + LSP channel); J-18's agentic-grep leg is the narrow CONFIRM, this file's leg 1 is the broad mid-task QUERY loop. Compose; J-18 ships first (soft dep — leg 1 works on the current cascade too, just retrieves at GraphCodeBERT quality until J-18 lands).
- QueryTrace 9-stage [WIRED] phase-1I — the prefer-reuse SSB surface (a mid-task search is another traced query; extend its channel/stage rather than a parallel event where it fits).

## Integration
- crates/knowledge/ (or crates/orchestrator tool surface): register `hybrid_search` as an executor-callable tool routing to the FAST-tier HYBRID cascade (~120 LOC: tool schema + handler + reranker-off default + max-rounds guard). The tool seam is the SAME IpcToolDispatcher the B-29 traversal tools register through (C-13) — reuse, do not author a new dispatch path.
- crates/knowledge/ retrieval-time neighbor expansion (~90 LOC: top-K -> DKG neighbor lookup depth<=2 -> rerank the expanded set), shared by the pre-task fill and leg 1.
- crates/orchestrator/ context-sufficiency gate (~80 LOC: deterministic symbol-coverage over DKG-resolved IntentPair + reasoning-surfaced symbols -> a signal the executor reads; max-rounds cap).
- +RetrievalLoopEvent SSB (Pattern-2 single-variant; Kind: MidTaskSearchInvoked / GraphNeighborExpanded / ContextSufficiencyChecked / RetrievalRoundCapped). baseline+1 from LIVE ~268/577 (feature-map stamp commit 5b0a66d 2026-06-15; re-grep ssb.rs SCHEMA_VERSION + action_type_lock.rs TOTAL_VARIANTS at edit, W-9, RT-1 — snapshots go stale in days). PREFER-REUSE (GD, J-2/J-18 precedent): if QueryTrace already carries a channel-source / round field OR J-18's RetrievalAugmentEvent ships first, ADD the three kinds there (net 0) rather than a new event; decide at edit. Full 6-part variant discipline if new (SsbActionType + SsbPayload + payload struct + all_test_payloads + golden row + TOTAL_VARIANTS bump).
- cross_lang pair: none (Rust-internal knowledge/orchestrator). frontend_pair: a retrieval-round / sufficiency breakdown on the existing KnowledgeDrawer (B-29) or W-6-justified absence.

## WIRING_CONTRACT
produces_symbols:
- crates/knowledge hybrid_search executor tool (FAST-tier HYBRID cascade as a mid-task callable, reranker-optional, max-rounds-guarded)
- crates/knowledge retrieval-time graph-neighbor expansion (top-K -> DKG depth<=2 neighbors -> rerank)
- crates/orchestrator context-sufficiency gate (deterministic symbol-coverage -> fire-search signal)
consumes_symbols:
- the SHIPPED HYBRID/RRF cascade + FAST tier (UNCHANGED; this exposes it, does not re-author it)
- B-29 DKG traversal edges + IpcToolDispatcher tool seam (C-13) — reused, not re-authored
- C-29 IntentPair symbol set (the coverage target)
- J-18 hardened cascade (CodeRankEmbed slot + LSP channel) when shipped
reachable_from:
- a real `elai` EXECUTING task where the agent calls hybrid_search mid-reasoning -> HYBRID cascade -> RRF  [PRODUCTION]
- the pre-task context fill path (leg 2 neighbor expansion runs there too)  [PRODUCTION]
- a #[cfg(test)] / contracts/ caller NEVER satisfies this (R7)
stub_ban:
- NO replacement of the pre-task pipeline-fill (the loop is ADDITIVE; removing the amortized prefill = a swap requiring user confirmation)
- NO LLM-judged sufficiency (deterministic symbol-coverage only; TD-3)
- NO unbounded mid-task re-query (max-rounds cap is mandatory; the dead-loop must be structurally impossible)
- NO mandatory reranker on the mid-task path (FAST-tier default; rerank is opt-in per query — latency)
- NO new embedding model or index geometry (reuses J-18's slot)
- NO new default-OFF ELAI_*_ENABLE without a wire-dark-allowlist row + promotion plan (R9/W-2)
- NO retrieval channel returning top-k noise when it has nothing relevant (the B6 RRF-noise critique; an empty neighbor set contributes nothing, not rank-k filler)
frontend_pair: retrieval-round / sufficiency breakdown on KnowledgeDrawer (decide at edit) or W-6-justified absence.
observable_effect:
- trigger: "a real `elai` EXECUTING task whose pre-task context fill MISSES the right span (a symbol the executor only discovers after reading the first file), run through the real loop with leg 3 on"
- expect_ssb: "ContextSufficiencyChecked emits below-threshold for the missed symbol -> MidTaskSearchInvoked fires a hybrid_search for it -> GraphNeighborExpanded pulls its call/import neighbors -> the resolved span lands in the executor's context; a RetrievalRoundCapped row proves the loop is bounded"
- expect_artifact: "the executor edits the correct span the pre-task-only control arm missed; an A/B on a fixture where the answer lives in a NEIGHBOR of the top-K (not the top-K itself) shows leg-2 fusion surfaces it where pure-dense top-K does not (ELAI-MEASURED delta, never the literature +13%). NEGATIVE (R2): sufficiency ABOVE threshold -> NO mid-task search fires (the gate does not over-retrieve, observable); the max-rounds cap is HIT on a pathological no-progress fixture and the loop STOPS with RetrievalRoundCapped (not an infinite loop); leg 1 disabled -> the executor runs pre-task-fill-only and MISSES the span (the loop is load-bearing, the flag is observable); an empty neighbor set contributes ZERO rank-filler to RRF (B6)."

## Grounding
- SWE-Explore arXiv:2606.07297 (VERIFIED 2026-06-16 via arxiv.org/abs: title "SWE-Explore: Benchmarking How Coding Agents Explore Repositories", 848 issues/10 langs/203 repos; abstract "agentic explorers form a clear tier above classical retrieval", "line-level coverage and efficient ranking remain the key axes"). The 51-68% file / 15-19% line / r~0.92 numbers are full-paper-HTML (agent-fetched), design-rationale only (HD-1/ID-5), never a target.
- SpIDER arXiv:2512.16956 (VERIFIED 2026-06-16: title "SpIDER: Spatially Informed Dense Embedding Retrieval for Software Issue Localization"; abstract "improves dense retrieval performance by at least 13% across programming languages and benchmarks" — the +13% is IN the abstract). The Recall@3 0.44-vs-0.18 and 48h-LocAgent figures are full-paper, design-rationale.
- LocAgent arXiv:2503.09089 (VERIFIED, ACL 2025, gersteinlab/LocAgent — already in J-18; the 92.7% mid-task-tool number is design-rationale, dataset-ambiguous).
- SweRank arXiv:2505.07849 (VERIFIED 2026-06-16: title "SweRank: Software Issue Localization with Code Ranking"; abstract "outperforming both prior ranking models and costly agent-based systems using closed-source LLMs like Claude-3.5" — grounds the EXCLUDED claim that the pre-task pipeline is NOT replaced by agentic grep). The 83.21%/44x figures are full-paper.
- GrepRAG arXiv:2601.23254 (VERIFIED 2026-06-16: title "GrepRAG: An Empirical Study and Optimization of Grep-Like Retrieval for Code Completion"; abstract "7.04-15.58% relative improvement in code exact match over the best baseline on CrossCodeEval") — the honest counter-evidence grounding the OQ below (identifier queries route to BM25+graph, not the semantic loop).
- ELAI substrate (feature map 1.1/1.3 grep-confirmed): HYBRID/RRF cascade, B-29 traversal tools + IpcToolDispatcher (C-13), context_budget.rs slots, C-29 IntentPair, QueryTrace. LIVE SSB ~268/577 (feature-map stamp 5b0a66d 2026-06-15; re-grep at edit).
- Full comparison + the two-method verdict + the 2 sonnet oracle-agent reports: workflow/reasoning/retrieval-agentic-loop-grounding-2026-06-16.md (the source synthesis this file cites instead of re-asserting).

## COMPOSES-WITH
J-18 (exposes its hardened cascade; J-18's narrow agentic-grep CONFIRM vs this file's broad mid-task QUERY loop — distinct legs, J-18 ships first), B-29 (consumes the traversal edges + tool seam; leg 2 adds retrieval-time neighbor fusion the impact API never did), J-16 (the blast-radius cone reads the SAME DKG edges leg 2 walks — sibling consumer, not re-authored), E-17 / phase-3K (the FAST/STANDARD/DEEP tier routing leg 1 rides), C-29 IntentPair (the coverage target), C-13 IpcToolDispatcher (the tool seam), 3E context budget (the pre-task fill site leg 3 reads). Restates none.

## Open questions / risks
- mid-task latency vs amortization: every mid-task hybrid_search costs a retrieval round; leg 3's sufficiency gate is what keeps it from firing on every step (over-retrieval is the failure mode SweRank's cheap-prefill wins on). The first checkbox is the gate's threshold + max-rounds cap calibrated on >=3 real runs (GD-4, observe-only first) before any default-ON.
- identifier vs intent routing (GrepRAG, 2601.23254): for exact-identifier queries pure grep/BM25 beats the semantic pipeline (35x faster, full-paper). leg 1 should route identifier-shaped queries to BM25+symbol-graph (FAST, no embedding) and reserve the embedding leg for intent/conceptual queries — a deterministic query-shape classifier, not the full cascade every call.
- reranker on the loop: OFF by default (latency); but a low-K rerank on the FINAL round before the executor commits may be worth it — measure, do not assume (B7).
- SSB reuse vs new event: if J-18's RetrievalAugmentEvent or QueryTrace already carries a round/channel field, extend it (net 0) — avoid a parallel event for the same query lifecycle (decide at edit).

## Next
The pre-task pipeline stays the amortized floor; this loop is the recovery+target path on top of it. Sequence AFTER J-18 (the loop should expose the HARDENED cascade, not the GraphCodeBERT one). leg 3's sufficiency gate is the cheapest first checkbox (deterministic, observe-only) and the thing that makes legs 1-2 disciplined rather than a re-query spiral.

---
## WARGAME BLOCKS (refine-before-execute, injected per plan-standard.md)

### UNIT 1: leg 3 context-sufficiency gate (cheapest first)

MOVE 1.1: author the deterministic symbol-coverage gate in crates/orchestrator/
EXPECTED_OBSERVATION: a new fn compute_context_sufficiency(intent_symbols, context_symbols) returning SufficiencyReport with coverage_ratio, threshold, is_sufficient, missing symbols. max_rounds cap constant defined.
ON_FAIL: compile error on SymbolId type -> SymbolId not resolved in orchestrator crate -> re-grep intent_pair.rs for the canonical type; counter-move: grep type.*Symbol in crates/orchestrator/src/.
ON_FAIL: IntentPair field access fails -> struct changed -> re-grep intent_pair.rs for current field names; counter-move: adapt field access.
FORK: if IntentPair already exposes a resolved-symbol-set accessor -> reuse it; else add a resolver method on IntentPair that DKG-resolves its raw symbol strings.
ABORT: if DKG (B-29) is absent or the symbol-resolution path is fundamentally broken -> STOP, escalate.
VERIFY: python .claude/tools/production_caller_check.py --files <changed .rs> exits 0 (WIRED). The gate is called from task_runner.rs before the executor starts. Off-agent artifact: a ContextSufficiencyChecked SSB row lands on a real elai task. FALSIFICATION: delete the call site in task_runner.rs -> SSB row absent -> restore -> row present.

MOVE 1.2: wire the gate signal into the executor retrieval-loop control
EXPECTED_OBSERVATION: when is_sufficient=false, the executor receives a signal to fire hybrid_search; when true, no signal fires.
ON_FAIL: no existing seam in task_runner for the signal -> add a sufficiency field to the context budget struct; counter-move: grep struct.*ContextBudget for the current struct.
FORK: if the context budget struct has an extensible field map -> add sufficiency there; else add a dedicated sufficiency: Option<SufficiencyReport> field.
ABORT: if wiring requires restructuring the executor loop control flow -> STOP, escalate.
VERIFY: a real elai task with incomplete context fires below-threshold; complete context does NOT fire (negative test). Off-agent artifact: SSB row carries is_sufficient matching the fixture.

### UNIT 2: leg 1 mid-task hybrid_search tool

MOVE 2.1: register hybrid_search via IpcToolDispatcher (C-13)
EXPECTED_OBSERVATION: a new tool hybrid_search(query, top_k, mode) registered in IpcToolDispatcher, routing to FAST-tier HYBRID cascade, callable mid-reasoning.
ON_FAIL: IpcToolDispatcher registration pattern changed -> re-grep crates/tools/src/ipc_registry.rs for current pattern; counter-move: mirror B-29 traversal tool registration.
ON_FAIL: SearchMode enum absent or renamed -> re-grep crates/knowledge for current enum; counter-move: use current name.
FORK: if HYBRID cascade exposes a direct search() API -> call it; else add a thin adapter wrapping the cascade pipeline.
FORK: if identifier-shaped query routing is trivially classifiable -> add deterministic is_identifier_query classifier routing to BM25+symbol-graph; else defer classifier and route all through HYBRID.
ABORT: if HYBRID cascade is absent or removed -> STOP, escalate.
ABORT: if IpcToolDispatcher seam (C-13) is absent or restructured -> STOP, escalate.
VERIFY: python .claude/tools/production_caller_check.py --files <changed .rs> exits 0 (WIRED). Off-agent artifact: MidTaskSearchInvoked SSB row lands when executor calls hybrid_search. FALSIFICATION: delete tool registration -> SSB row absent -> restore -> row present.

MOVE 2.2: max-rounds guard on the mid-task loop
EXPECTED_OBSERVATION: a max_rounds constant enforced structurally; after max_rounds hybrid_search calls without sufficiency going above-threshold, the loop stops and emits RetrievalRoundCapped SSB row.
ON_FAIL: no existing loop-control seam -> add a round counter to the context budget struct; counter-move: sufficiency gate already reads context budget, counter is co-located.
ABORT: if max-rounds cap cannot be structurally enforced -> STOP, unbounded re-query is stub_ban violation; escalate.
VERIFY: a pathological no-progress fixture hits the cap and emits RetrievalRoundCapped; loop does NOT continue past max_rounds. Off-agent artifact: SSB row carries round=max_rounds.

### UNIT 3: leg 2 SpIDER graph-neighbor fusion

MOVE 3.1: author neighbor-expansion in crates/knowledge/
EXPECTED_OBSERVATION: a new fn expand_with_graph_neighbors(results, depth) that looks up each result's call/import/type-resolution neighbors via DKG SQLite edges at depth<=2, merges into the result set, returns expanded set for reranking.
ON_FAIL: DKG edge schema changed -> re-grep crates/knowledge for current edge table schema; counter-move: adapt SQL query.
ON_FAIL: SearchResult lacks symbol_id field -> re-grep for current struct; counter-move: add field or use existing identifier field.
FORK: if DKG exposes a neighbors(symbol_id, depth) API (B-29) -> reuse it; else add a thin SQL query.
ABORT: if DKG (B-29) is absent or edge table empty -> STOP, escalate.
VERIFY: python .claude/tools/production_caller_check.py --files <changed .rs> exits 0 (WIRED). Called from both pre-task fill and mid-task hybrid_search. Off-agent artifact: GraphNeighborExpanded SSB row lands. FALSIFICATION: delete call site in hybrid_search handler -> SSB row absent -> restore -> row present.

MOVE 3.2: rerank the expanded set
EXPECTED_OBSERVATION: expanded set reranked by existing RRF cascade, graph-neighbors that are relevant surface above irrelevant top-K noise.
ON_FAIL: RRF cascade does not accept expanded input -> re-grep for current RRF API; counter-move: pass expanded set as merged vector.
FORK: if RRF handles arbitrary input vectors -> call directly; else add a merge step that deduplicates by symbol_id before reranking.
ABORT: if RRF cascade absent or removed -> STOP, escalate.
VERIFY: an A/B fixture where the answer lives in a NEIGHBOR of top-K shows leg-2 fusion surfaces it where pure-dense does not. ELAI-MEASURED delta.

### SSB VARIANTS
FORK: if QueryTrace carries a round/channel field OR J-18 RetrievalAugmentEvent ships first -> ADD four kinds there (net 0); else add new RetrievalLoopEvent with four kinds (net +4, full 6-part variant discipline).
RECON_NEEDED: grep ssb.rs for SCHEMA_VERSION + action_type_lock.rs TOTAL_VARIANTS at edit time.

### INTEGRATION_SMOKE
INTEGRATION_SMOKE: after units 1+2: drive a real elai task where pre-task fill MISSES a span the executor discovers mid-reasoning; assert hybrid_search fires (MidTaskSearchInvoked SSB row), sufficiency gate reported below-threshold (ContextSufficiencyChecked is_sufficient=false), span lands in context. HARD input: multi-turn task where executor reads file A, discovers it calls module B, B's behavior is the answer.
INTEGRATION_SMOKE: after unit 3, before DONE: full loop smoke: gate below-threshold -> hybrid_search -> neighbor expansion -> span lands -> gate re-checks -> above-threshold -> proceed. PLUS negative path (above-threshold -> no search) and cap path (no-progress -> RetrievalRoundCapped). HARD input: fixture where answer lives in NEIGHBOR of top-K.

### REDTEAM
REDTEAM:
- Attack 1 "loop fires on every step over-retrieving" -> FAILED: sufficiency gate structurally prevents; gate fires below-threshold ONLY when coverage incomplete, max-rounds cap bounds total queries. No patch.
- Attack 2 "loop runs forever on no-progress" -> FAILED: max-rounds cap mandatory (stub_ban), RetrievalRoundCapped SSB row proves stop. No patch.
- Attack 3 "gate never fires making legs 1-2 dead code" -> PARTIAL: plan declares negative test but must also assert POSITIVE path. PATCH: INTEGRATION_SMOKE now asserts both paths.
- Attack 4 "identifier queries waste embedding leg" -> DEFERRED: OQ names classifier, FORK 2.1 makes it optional. Optimization not blocker; no patch for execution.

End of J-38.
---

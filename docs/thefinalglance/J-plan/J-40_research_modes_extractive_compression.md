# J-40 research_modes_extractive_compression

- Track J | lever research-determinism + context-economy | grounding research.perplexity.ai (query-aware compression + search-as-code) + arXiv:2501.14342 + metrics/baseline_findings_and_actions.md | nature build (activate E-48 live + extend A-RAG) | effort M | WAVE DD-2 | depends J-38, E-48, base-1E
- The model is a cog, not a brain, in research: it should only FORMULATE queries and RANK/SELECT pages; the harness fetches (in parallel), extractively compresses each source to a small span set, and only distilled grounded spans + external per-step state ever enter the reasoning context. This keeps the working context tiny -- the only thing that makes a multi-step research loop viable on a small local model under a hard ~25-40K ceiling (Perplexity needs its full 128K even AFTER 50:1 compression at ~200 sources, so compression is mandatory not optional). Adds speed-tiered research modes over the shipped FlashResearch tree and lights up the DARK live-search leg of E-48.

## Steal/build
1. Query-aware EXTRACTIVE compressor for WEB sources (the key net-new piece): a `QueryAwareExtractor` impl of the `ContentExtractor` trait that scores per-chunk sentences by semantic overlap with the query vector (not just keyword containment) and keeps the top spans VERBATIM. Extractive, never generative summarization -- verbatim spans preserve citation fidelity and beat generative on both faithfulness and token efficiency (Perplexity query-aware-compression: >10K->~200 tok/doc, >50:1, <20ms, verbatim). Target a per-source span budget so a 25-40K local model fits ~100-150 sources.
2. Activate the E-48 live-search leg: swap `StubSearchBackend` for a real `SearXngBackend` behind the existing `WebResearchStepEmitter` trait (the pipeline is ALREADY wired to the REPL; only the search backend is stubbed). Sidecar /search + SearXNG is the harness fetch layer; the model never calls a raw fetch tool.
3. Speed-tiered research modes over the FlashResearch tree (distinct from 1I SearchMode degradation -- intentional tier): FAST = one ranking pass, top-K fetched+compressed, single grounded synthesis, no iteration; STANDARD = shallow tree (breadth 2-3, depth 3); DEEP = full tree (breadth 4-5, depth 5) + iterative refine. Each tier is the same cog invariant; tiers differ only in breadth/depth/iteration.
4. EXTERNAL per-step state between research iterations (not in-context): serialize hit lists + coverage map to the SESSION DAG / journal between turns so the reasoning context starts each step lean (the Perplexity filesystem-serialization pattern; ELAI already has SESSION DAG lossless trim + SSB as the external store). Each research step is a bounded-context UNIT (the J-39 node-admission discipline applied to research steps).
EXCLUDED: generative re-summarization of fetched content (busts faithfulness + costs more tokens; HD-2 deterministic-only on the cache prefix); KV-cache reuse as the "fast context" lever (no-op on the local lane -- sidecar PrefixKVCache bypassed on the streaming path, H-50; the real lever is extractive compression + external state); full "Search-as-Code" retrieval-plan-as-program (a CodeAct/E-35 extension worth a measured A/B later, NOT default here -- ELAI's 4E AutoB2G already chose deterministic assembly over LLM-driven repair).

## Existing ground truth
(grep-confirmed 2026-06-16 via grounding agent; re-grep at edit, W-7)
- `WebResearchOrchestrator` + `run_research_pipeline` in `crates/orchestrator/src/research_pipeline.rs`; production caller `crates/cli/src/repl_command.rs:415` (also `prebind_research_answer` at `:422`). E-48 is WIRED TO THE REPL; the DARK leg is only the live search backend (`crates/cli/src/prebind_cmd.rs:18-19` "deferred until E-48's search backend is GPU-live"). Seam: `WebResearchStepEmitter` trait at `repl_command.rs:763`; swap `StubSearchBackend` -> `SearXngBackend`.
- `extract_keyword_snippet` at `crates/knowledge/src/arag.rs:615` (sentence split, keyword filter, `SNIPPET_MAX_SENTENCES`) -- the extractive snippet extension point. `WebChunk.sentences()` at `crates/knowledge/src/web_arag.rs:56`. `BasicExtractor` + `ContentExtractor` trait at `crates/web_search/src/extractor.rs:32`.
- FlashResearch tree (base-4D): adaptive breadth/depth/pruning, Simple/Moderate/Complex presets -- the existing tiered engine.
- SESSION DAG external state + 3-pass lossless trim (phase-3H/B-9); stubs recoverable from SSB seq.
- Measured ceilings: `metrics/baseline_findings_and_actions.md:150-151` (3B ~40K / 7B ~25K).

## Integration
The `QueryAwareExtractor` replaces the `extract_keyword_snippet` call inside `keyword_search`/`web_arag` so every fetched web source is compressed before it reaches the reasoning context; the synthesis step sees only ranked, compressed, pre-cited spans (hands off to J-41's closed-set citation binding). Mode selection rides the FlashResearch tree's existing complexity presets. External state uses the SESSION DAG. The SearXNG backend swap is the only DARK->wired flip. Composes J-38 (the mid-task agent-callable retrieval loop -- J-40 is its compression+modes layer). Honest residue: the live search backend depends on the remote sidecar /search being up; the compressor and modes are testable offline against fixtures.

## WIRING_CONTRACT
produces_symbols: [QueryAwareExtractor, ResearchMode (Fast|Standard|Deep), SearXngBackend, compress_source_spans]
consumes_symbols: [ContentExtractor, WebResearchStepEmitter, run_research_pipeline, FlashResearch tree (4D), SessionDag]
reachable_from: [repl_command.rs research entrypoint; the `WebResearchStepEmitter` seam; CLI research/oracle command with a --mode flag]
stub_ban: [no generative summarization in the extractor, no placeholder span budget, StubSearchBackend retired not left dual-pathed]
frontend_pair: [optional: research-mode + per-source compression-ratio surfaced in an existing dashboard research/oracle panel; cross_lang_diff.py PASS if added]
observable_effect: per-source compression ratio + selected ResearchMode recorded in the existing QueryTrace stage rows (a fetched-source span set that is strictly smaller than the raw page, logged); end-to-end: a research run completes with raw page bytes NEVER entering the reasoning-context slot
SSB: prefer REUSE of QueryTrace (the research pipeline already traces stages) -- net 0 new action types. Add a +ResearchModeEvent only if mode-selection telemetry is wanted separately; if so, Pattern-2, bump from LIVE baseline (SCHEMA 268 / TV 577) at implementation (W-9).

## Grounding
- [Perplexity Research: Query-Aware Context Compression for Better Snippets](https://research.perplexity.ai/articles/query-aware-context-compression-for-better-snippets) -- extractive (not generative) span compression >50:1, <20ms, verbatim spans for citation fidelity; the design rationale for the compressor. T1 vendor primary; numbers are design-rationale ONLY, never claimed as ELAI numbers (HD-1/GD-2). VERIFIED 2026-06-16 (sonnet oracle).
- [Perplexity Research: Rethinking Search as Code Generation](https://research.perplexity.ai/articles/rethinking-search-as-code-generation) -- harness owns fetch, model emits a retrieval plan; grounds the cog split + external-state-between-turns. T1 vendor. (Search-as-Code itself is EXCLUDED/deferred.)
- [CoRAG: Chain-of-Retrieval Augmented Generation, arXiv:2501.14342 (NeurIPS 2025)](https://arxiv.org/abs/2501.14342) -- iterative query reformulation with harness-executed retrieval; grounds the DEEP-mode loop. VERIFIED.
- metrics/baseline_findings_and_actions.md -- the 3B ~40K / 7B ~25K ceilings the per-source span budget is sized against.

## COMPOSES-WITH
- J-38: the mid-task agent-callable hybrid_search loop; J-40 supplies its compression + speed-tiers layer.
- J-41: the compressed, ranked spans feed J-41's closed-set citation binding + grounding check.
- E-48: activates the live SearXNG backend; reuses the wired REPL pipeline.
- base-1E / base-4D: extends the A-RAG extractor + the FlashResearch tree.
- ELAI-DETERMINISM-DOCTRINE: the "model selects / harness fetches+compresses" application (research-side cog).

## Open questions / risks
- Extractive vs query relevance: a query-aware semantic scorer needs the query embedding + per-sentence embeddings; cost must stay under the latency budget (Perplexity hits <20ms with a small dedicated model -- ELAI's reranker/embed stack is the candidate, measure it).
- Compression too aggressive can drop the supporting span for a true claim -> a false "ungrounded" in J-41. Tune the span budget against a recall metric, not just token count.
- Live SearXNG backend depends on the remote sidecar being up (canonical remote, never local); offline path must degrade gracefully (existing 1I cascade).

## Next
J-41 builds the cheap grounding stack the compressed spans feed into (three-tier fingerprint -> small-model claim check -> closed-set citation).

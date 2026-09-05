# J-75: measure-then-simplify — a flag-gated RRF-component ablation harness over the over-fused retrieval stack

- Track J (context-retrieval wave) | lever CHEAPER | grounding STRONG (CMU 2502.07067 primary: a full multi-stage pipeline does NOT significantly beat a single good code reranker at depth 100) | nature EXTENSION (instrumentation; NOT a deletion) | effort M | WAVE CR-3 | depends none (measures J-72/J-74's channels too)

- Premise: the report's core attack is that ELAI's stack is over-fused — BM25 + dual ONNX dense + PageRank + RRF k=60 + cross-encoder + the E-40 5th SCIP ranker — and the lift comes from 2-3 components. CMU 2502.07067 (primary): a full commit-history+code multi-stage pipeline does NOT significantly beat a single good code reranker at depth 100 (CodeReranker@100 MAP 0.288 vs full 0.307, ~6.6% avg, <2% on popular repos) at ~7x runtime; the reranker is the HIGH-value stage and is NOT removed. ELAI cannot act on a paper number against ITS OWN stack — so J-75 is a MEASURE-FIRST experiment that produces the ELAI delta; any actual prune is a SEPARATE, user-confirmed decision (house rule: no removal/simplification without explicit user confirmation).

## Steal/build
1. ABLATION HARNESS behind flags: drop PageRank and/or the second dense encoder from the RRF blend, measure MRR/Recall@5 delta on ELAI's gold set (the J-72 gold set). The report predicts <7% MAP loss on popular repos; ELAI MEASURES its own number.
2. The harness emits a per-configuration delta REPORT (`elai bench rrf-ablate`), deterministic, never LLM-judge (TD-3). It does NOT prune anything — it is instrumentation that produces the number the prune decision needs.
3. HARD GUARD (house rule, in the file near the claim): C5 is a MEASURE-FIRST experiment. Any actual stage removal is gated on BOTH the measured delta AND explicit user sign-off. Authored as instrumentation, NOT a deletion unit. Graphs demote from RANKING to NAVIGATION only if the number says so AND the user confirms (J-74 keeps the dataflow channel; this measures whether it earns RRF inclusion vs nav-only).
EXCLUDED: removing the cross-encoder reranker (CMU primary: it is the high-value stage — NEVER the prune target); any silent prune (every removal is user-gated); a paper-number-driven prune (ELAI measures its own delta, ID-5).

## Existing ground truth (grep-confirmed 2026-06-23; re-grep at edit, W-7/W-9)
- The over-fused stack IS ELAI's shape: BM25 (tantivy) + dual ONNX dense (GraphCodeBERT 768d + Granite 384d) + PageRank repo map + RRF k=60 + cross-encoder ms-marco + E-40 5th SCIP ranker (feature map 1.1 L57-135; phases 1B/1C/1D/1G + E-40).
- HyDE already conditional (R3): crates/knowledge/src/hyde.rs short_query_skips_hyde / four_word_query_skips_hyde — already the kind of measure-then-conditionalize this harness generalizes.
- SPLADE already DEFERRED (R4): cfg(feature="splade"), early-DROP conditions defined (<5% recall@10 gain over BM25 OR Wilcoxon p>=0.05) — the same measure-first discipline, already applied.
- the J-72 gold set + `elai bench retrieval-adapt` harness — J-75 reuses the gold set and the delta-measurement plumbing.

## REVISED scope (W-10 tiered)
- SHIP NOW Tier-1 (in-process, deterministic, model-free): the flag-gated ablation harness `elai bench rrf-ablate` over the gold set + the per-config MRR/Recall@5 delta report. PRODUCES the number; PRUNES nothing.
- NO Tier-2 prune in this unit: the prune is a future user-confirmed decision keyed off this number, NOT a J-75 deliverable.

## Integration
- crates/knowledge — flag-gated ablation toggles on the RRF blend (drop-PageRank / drop-second-dense / nav-only-graph variants; behind ablation flags, allowlisted, ablation-only never production-default).
- crates/orchestrator or crates/knowledge — `elai bench rrf-ablate` (reuses the J-72 gold set + delta plumbing; emits the per-config report).
- SSB: PREFER-REUSE the bench/convergence SSB family (I-27 ConvergenceCurveMeasured pattern / `elai measure` C-20) — an ablation run is a measurement, net 0 new action_type; the report is the on-disk artifact. decide at edit.
- cross_lang pair: none (Rust-internal bench); frontend_pair: the ablation delta on the existing ExternalBenchPanel / CodingMetrics surface (E-49) or W-6-justified absence.

## WIRING_CONTRACT
produces_symbols:
- crates/knowledge flag-gated RRF-component ablation toggles (drop-PageRank / drop-second-dense / nav-only-graph)
- `elai bench rrf-ablate` (per-config MRR/Recall@5 delta report over the gold set)
consumes_symbols:
- the J-72 ELAI gold set + `elai bench retrieval-adapt` delta plumbing (NOT re-authored)
- the shipped RRF cascade + E-40 5th ranker (the thing being ablated, behind toggles; NOT modified in production)
- I-27 ConvergenceCurveMeasured / C-20 `elai measure` (the bench SSB/report shape; reused)
reachable_from:
- `elai bench rrf-ablate` CLI entrypoint  [PRODUCTION]
- a #[cfg(test)] / contracts/ caller NEVER satisfies this (R7)
stub_ban:
- NO actual stage removal in this unit (MEASURE-FIRST; any prune is a separate user-confirmed decision — house no-removal rule)
- NO cross-encoder reranker in the prune set (CMU primary: it is the high-value stage)
- NO paper number as the prune trigger (the gate is the ELAI-MEASURED delta, ID-5)
- NO LLM-judge scoring of the ablation (TD-3 — MRR/Recall are deterministic on the gold set)
- NO ablation toggle leaking into the production-default config (ablation flags are bench-only)
frontend_pair: ablation delta on ExternalBenchPanel/CodingMetrics (decide at edit) or W-6-justified absence.
observable_effect:
- trigger: "`elai bench rrf-ablate --drop pagerank` and `--drop second-dense` and `--graph nav-only` over the ELAI gold set, vs the full stack"
- expect_ssb: "ConvergenceCurveMeasured / measure-run rows (or the reused bench SSB) carrying the per-config MRR/Recall@5"
- expect_artifact: "a delta report: full-stack vs each ablated config on the ELAI gold set (the number ELAI measures, never the report's <7%). NEGATIVE (house rule): the harness PRODUCES the number and PRUNES nothing — no production config changes; a prune requires a separate user-confirmed change citing this report."

## Grounding
- CMU repo-search arXiv:2502.07067 (workflow/papers/cmu-repo-search-interface.md): full multi-stage pipeline NOT significantly > a single good code reranker at depth 100 (MAP 0.288 vs 0.307, ~6.6% avg, <2% popular) at ~7x runtime; the reranker is high-value — DESIGN-rationale (ID-5).
- CodeRAG-Bench arXiv:2406.14497 (workflow/papers/arxiv-2406-14497-coderag-bench.md, pre-existing) + the report's "Attack the design" section.
- house rule (token-discipline / CLAUDE.md): "No removal or simplification without explicit user confirmation" — the binding constraint that makes this measure-first.
- ELAI stack shape codegraph/feature-map confirmed 2026-06-23 (context-retrieval-research-2026-06-23.md C5).
- LIVE SSB ~269/578 (post-J-66; grep 2026-06-22). re-grep at edit (W-9).

## COMPOSES-WITH
J-72 (reuses its gold set + delta harness), J-74 (measures whether the dataflow channel earns RRF inclusion vs nav-only), HyDE-conditional (R3) + SPLADE-deferred (R4) (the same measure-first discipline already shipped), I-27 / C-20 (the bench SSB/report shape), E-49 CodingMetrics (the surface). Restates none.

## Open questions / risks
- gold-set representativeness: an ablation delta is only as good as the gold set — if it under-represents popular-repo queries, the <7% prediction may not transfer; caveat the report with the gold-set composition.
- flag hygiene: ablation toggles must never bleed into production defaults — assert in a contract test that the production RRF config is the full stack unless a user-confirmed prune lands.

End of J-75.

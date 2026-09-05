# Extractive research compression (J-40)

- **ELAI ids:** J-40 / WAVE DD-2 / QueryAwareExtractor / ResearchMode / SearXngBackend
- **Spec status (as designed):** PLANNED BUILD (activate E-48 live search + extend A-RAG). Live cousins: `QueryAwareExtractor` trait + `ResearchMode::{Fast,Standard,Deep}` + stub-ban contract exist; plan FAST “no iteration” and “raw page bytes never in reasoning context” are the rebuild bar (`J-plan/J-40_research_modes_extractive_compression.md:1-5`, `J-plan/J-SUMMARY_INDEX.md:1080-1095`).
- **Crate / module (spec):** `crates/web_search/src/extractor.rs` (`QueryAwareExtractor`); `crates/orchestrator/src/research_mode.rs`; `research_pipeline.rs` / `web_research_orchestrator.rs`; knowledge `arag.rs` snippet seam.
- **Purpose of this file:** enough architecture to rebuild model-selects / harness-fetches+extractively-compresses research. Not a status ledger.

## 1. What it is

**Extractive research compression** is the DD-wave research axis: the model only **formulates queries** and **ranks/selects pages**; the harness **fetches in parallel**, **extractively** compresses each source to a small **verbatim** span set, and keeps per-step research state **outside** the reasoning context (SessionDag / journal). Speed tiers `{Fast, Standard, Deep}` wrap the shipped FlashResearch tree. E-48’s live SearXNG backend is the fetch layer. This is the only way a multi-step research loop survives a local ~25–40K ceiling (`J-plan/J-40_research_modes_extractive_compression.md:4,7-10`, `ELAI-CONTEXT-MANAGEMENT-DEEP-DIVE.md` §5 research neighbor via J-39 discipline).

Perplexity still needs 128K even after ~50:1 compression at ~200 sources — compression is **mandatory**, not optional. Those vendor ratios are **design rationale only**, never ELAI numbers (HD-1) (`J-plan/J-40_research_modes_extractive_compression.md:4,34`).

## 2. Why it exists

If raw pages enter the GRC/reasoning slot, the local 7B ~25K / 3B ~40K lane dies mid-loop. If compression is **generative** re-summary, citation fidelity busts and HD-2 cache prefix is not deterministic (`J-plan/J-40_research_modes_extractive_compression.md:11`). If the model calls raw fetch, the cog split is gone. If KV-reuse is sold as the fast-context lever: **no-op on local** (PrefixKVCache bypassed on streaming; H-50) (`J-plan/J-40_research_modes_extractive_compression.md:11`, `J-plan/_grounding/J-GRC-V3-GROUNDING-MAP.md:133-135`).

J-40 is J-38’s compression+tiers layer and J-41’s span supplier (`J-plan/J-40_research_modes_extractive_compression.md:39-41`).

## 3. How it works (runtime)

1. **Who calls it.** REPL research entrypoint / `WebResearchStepEmitter` seam / CLI research/oracle `--mode` (`J-plan/J-40_research_modes_extractive_compression.md:27`). Plan pins `repl_command.rs:415` / `:422` / `:763` as of 2026-06-16 — **re-grep**; live emitter trait is `web_research_orchestrator.rs:125`. Production pipeline: `run_research_pipeline` (`research_pipeline.rs:654`).
2. **Model may** formulate queries and rank/select pages. **Harness** fetches (parallel) and compresses. Model **never** calls a raw fetch tool (`J-plan/J-40_research_modes_extractive_compression.md:4,8`).
3. **QueryAwareExtractor (deterministic extractive, net-new).** Impl of `ContentExtractor` sibling: score per-chunk **sentences** by **semantic overlap with the query vector** (not just keyword containment); keep top spans **VERBATIM**. Never generative summarization (`J-plan/J-40_research_modes_extractive_compression.md:7`). Live trait: `extract_query_aware(html, url, query)` (`extractor.rs:31-41`). Live `SPAN_BUDGET_SENTENCES = 5` (`extractor.rs:43-44`) — plan wants a per-source span budget sized so 25–40K fits **~100–150 sources**; 5 sentences is a cousin constant, not proven equal to that budget (§16).
4. **Replace keyword snippet on the web path.** `extract_keyword_snippet` (`arag.rs:614-625`, `SNIPPET_MAX_SENTENCES=5`) is the A-RAG extension point. `WebChunk.sentences()` at `web_arag.rs:56` (plan pin). `BasicExtractor` + `ContentExtractor` at `extractor.rs:26-28,48`. Query-aware **replaces** keyword snippet **inside** `keyword_search` / `web_arag` so every fetched web source is compressed **before** reasoning context (`J-plan/J-40_research_modes_extractive_compression.md:21`).
5. **Activate E-48 live search.** Swap `StubSearchBackend` → `SearXngBackend` behind `WebResearchStepEmitter`. Sidecar `/search` + SearXNG = fetch layer. **StubSearchBackend retired, not dual-pathed** (`J-plan/J-40_research_modes_extractive_compression.md:8,28`). Live `j40_stub_ban_contract.rs` asserts `ELAI_STUB_SEARCH=1` must **not** select the stub. Offline: degrade via existing 1I cascade; sidecar-down is honest residue (`J-plan/J-40_research_modes_extractive_compression.md:22,49`).
6. **ResearchMode tiers (same cog, different breadth/depth/iteration)** — **distinct from 1I SearchMode degradation** (`J-plan/J-40_research_modes_extractive_compression.md:9`):

| Mode | Plan tree |
|---|---|
| `Fast` | one ranking pass, top-K fetched+compressed, **single** grounded synthesis, **no iteration** |
| `Standard` | shallow tree **breadth 2–3, depth 3** |
| `Deep` | full tree **breadth 4–5, depth 5** + iterative refine (CoRAG-style harness-executed retrieval) |

Live `ResearchMode::caps` (`research_mode.rs:6-35`) still gives Fast `max_iterations: depth * 3` — **not** plan FAST. Rebuild the **plan** table.

7. **External per-step state.** Serialize hit lists + coverage map to **SessionDag / journal** between turns. Reasoning context starts each step lean. Each research step is a bounded-context **unit** (J-39 discipline on research) (`J-plan/J-40_research_modes_extractive_compression.md:10`).
8. **Hand-off to J-41.** Synthesis sees only ranked, compressed, pre-cited spans (`J-plan/J-40_research_modes_extractive_compression.md:21,51`).
9. **Commit.** Per-source compression ratio + selected `ResearchMode` in existing **QueryTrace** stage rows. End-to-end: research completes with **raw page bytes NEVER** in the reasoning-context slot. Prefer-reuse QueryTrace, **net 0** SSB; optional `+ResearchModeEvent` (`J-plan/J-40_research_modes_extractive_compression.md:30-31`).
10. **Fail-closed.** No generative summarization in the extractor; no placeholder span budget; stub not left dual-pathed (`J-plan/J-40_research_modes_extractive_compression.md:28`).

## 4. Types, enums, objects

Copy from `J-plan/J-40_research_modes_extractive_compression.md:25-26` and live traits that name the same organ.

```
trait ContentExtractor {
  fn extract(html, url) -> Result<ExtractedContent, WebSearchError>
}

trait QueryAwareExtractor {
  fn extract_query_aware(html, url, query) -> Result<ExtractedContent, WebSearchError>
}

fn compress_source_spans(...)     // plan produces_symbols; verbatim top spans

ResearchMode { Fast, Standard, Deep }

SearXngBackend                    // live-search; plan name
  // live contract also mentions SearxSearchBackend — UNCERTAIN alias (§16)

ExtractedContent { title, text, word_count, page_type, extracted_at }  // extractor.rs:16-22
PageType { Article, Empty }
```

**Consumed:** `WebResearchStepEmitter`, `run_research_pipeline`, FlashResearch 4D tree, SessionDag.

**1I SearchMode** is a **different** enum (degradation). Do not merge.

## 5. Control flow / state machine

**Owner:** research pipeline / orchestrator. FlashResearch owns tree shape; J-40 owns compression + mode caps + backend swap.

```
CLI/REPL --mode Fast|Standard|Deep
     │
     ▼
model formulates query (+ later rank/select)
harness SearXNG fetch (parallel)     // never model raw-fetch
     │
     ▼
QueryAwareExtractor: sentence scores vs query vector
keep top verbatim spans (span budget)
raw HTML/page bytes do NOT enter reasoning slot
     │
     ▼
persist hit lists + coverage on SessionDag
next iteration starts lean
     │
     ▼
ranked compressed spans → J-41
QueryTrace records mode + per-source compression ratio
```

**Forced:** Fast = no iteration (plan). Stub backend retired. Generative summary path absent from extractor.

## 6. Grounding, receipts, gates

Verbatim spans **are** the citation substrate for J-41. Generative paraphrase is not a receipt.

**Never allowed:** LLM rewrite of fetched content as the compressor; KV-reuse as the local speed lever; Search-as-Code as **default** (deferred A/B) (`J-plan/J-40_research_modes_extractive_compression.md:11`).

Compression too aggressive can drop the supporting span → J-41 false ungrounded. Tune span budget on **recall**, not token count alone (`J-plan/J-40_research_modes_extractive_compression.md:48`).

## 7. Budgets, limits, ceilings

| Knob | Value | Scope |
|---|---|---|
| Local fit target | ~100–150 sources under 25–40K | per run (`J-plan/J-40_research_modes_extractive_compression.md:7`) |
| Lane ceilings | 3B ~40K / 7B ~25K | measured baseline (`:19`) |
| Vendor Perplexity | >10K→~200 tok/doc, >50:1, <20ms | **WHY only**, not ELAI SLOs (`:34`) |
| Live `SPAN_BUDGET_SENTENCES` | 5 | cousin (`extractor.rs:44`) |
| A-RAG `SNIPPET_MAX_SENTENCES` | 5 | keyword cousin (`arag.rs:26`) |
| Fast / Standard / Deep tree | §3 table | plan |
| SSB | reuse QueryTrace; net 0 | W-9 |

Latency budget for the semantic scorer: plan says measure against embed/reranker stack; Perplexity <20ms is **not** an ELAI gate (`J-plan/J-40_research_modes_extractive_compression.md:47`).

## 8. Roles and neighbors

Research/oracle CLI. Model = cog. Harness = fetch+compress.

**Consumes:** E-48 pipeline; 1E A-RAG extractor; 4D FlashResearch; SessionDag; J-38 loop.

**Produces:** compressed verbatim spans + mode; QueryTrace extras; input to J-41.

**Must not fork:** a second web fetch inside the agent; generative “summarize this page” tool as the compressor.

## 9. Config / flags / defaults

- `--mode` on CLI research/oracle (`J-plan/J-40_research_modes_extractive_compression.md:27`).
- `ELAI_STUB_SEARCH` must not resurrect StubSearchBackend (live contract).
- Frontend optional: research-mode + per-source compression ratio on existing research/oracle panel; `cross_lang_diff.py` PASS if added (`:29`).
- Live search requires remote sidecar `/search` (canonical remote, never local) (`:49`).

## 10. Implementation map

| Symbol | Spec path |
|---|---|
| `QueryAwareExtractor` / `compress_source_spans` | `web_search/src/extractor.rs:31-44` live trait |
| `ResearchMode` | `orchestrator/src/research_mode.rs:6-10` |
| `SearXngBackend` | plan; swap at E-48 seam |
| `run_research_pipeline` | `research_pipeline.rs:654` |
| `WebResearchStepEmitter` | `web_research_orchestrator.rs:125` |
| `extract_keyword_snippet` | `arag.rs:615` — **replace on web path** |
| `j40_query_aware_extractor_contract.rs` / `j40_research_mode_contract.rs` / `j40_stub_ban_contract.rs` | live contracts |
| QueryTrace | reuse |

Plan REPL line numbers (`repl_command.rs:415,422,763`, `prebind_cmd.rs:18-19`) are 2026-06-16 pins — re-grep (W-7).

## 11. Journal / SSB events

Prefer **QueryTrace** stage rows: per-source compression ratio + `ResearchMode`. Optional `+ResearchModeEvent` Pattern-2 from snapshot 268/577 (`J-plan/J-40_research_modes_extractive_compression.md:31`). Live also has `web_research_step` / stage events (`web_research_orchestrator.rs:131-149`). Rebuild: **raw page bytes absent** from the reasoning-context slot is the load-bearing observable.

## 12. Tests and observables

Plan (`J-plan/J-40_research_modes_extractive_compression.md:30`):

| Assert | |
|---|---|
| Per-source compressed span set **strictly smaller** than raw page | QueryTrace |
| `ResearchMode` recorded | QueryTrace |
| Raw page bytes **never** in reasoning-context slot | e2e |
| Extractor has **no** generative summarization | stub_ban |
| StubSearchBackend **retired** | not dual-pathed |
| Fast/Standard/Deep match §3 tree | not live `depth*3` Fast cousin |

Compressor + modes **testable offline against fixtures**; live backend is sidecar-gated (`:22`).

Adversarial: keyword-only extractor sold as query-aware; leaving stub behind a flag; stuffing HTML into GrcHistory.

## 13. Rebuild contract

Another harness has **this** organ iff all of the following are true:

- Model formulates and ranks; harness fetches and **extractively** compresses; no raw fetch tool for the model.
- Spans kept **verbatim** (citation-preserving); no LLM re-summary compressor.
- Per-source budget sized for local 25–40K to hold O(100) sources; ratio logged.
- External hit-list/coverage between steps (not only in-window).
- Three speed tiers with Fast = **no iteration**.
- Live search backend, stub retired.
- Raw page bytes never occupy the reasoning slot.
- Feeds a closed-set citation stage (J-41).

Keyword `take(5)` snippets, generative map-reduce summaries, or “just enable prompt cache” are cousins.

## 14. Non-goals / banned designs

- Generative re-summarization (`J-plan/J-40_research_modes_extractive_compression.md:11`).
- KV-cache reuse as the local fast-context lever (`:11`).
- Full Search-as-Code as default (`:11,35`).
- Dual-path stub backend (`:28`).
- Claiming Perplexity 50:1 / 200 tok as ELAI measurements (`:34`).
- Merging `ResearchMode` with 1I `SearchMode` (`:9`).

## 15. Source citations

Primary: `docs/thefinalglance/J-plan/J-40_research_modes_extractive_compression.md` (full file). Index: `J-plan/J-SUMMARY_INDEX.md:1080-1095`. Live: `web_search/src/extractor.rs:26-44`, `research_mode.rs:6-35`, `web_research_orchestrator.rs:125`, `research_pipeline.rs:654`, `arag.rs:26,615`. Doctrine: ELAI-DETERMINISM-DOCTRINE research-side cog (`:44`).

**[EXTERNAL, WHY]:** Perplexity query-aware compression + search-as-code (T1 vendor); CoRAG 2501.14342 (`:33-36`). VERIFIED 2026-06-16 in-plan.

## 16. Gaps

Label **UNCERTAIN**; do not fill:

- Numeric per-source token budget (plan ~200 tok/doc is vendor; ELAI target is “fit 100–150 sources”).
- Query embedding model / whether live `QueryAwareBasicExtractor` is lexical or vector (plan requires query **vector**).
- `SearXngBackend` vs live `SearxSearchBackend` name.
- Whether Fast’s live `depth*3` iterations is an accepted fill or a cousin to reject (plan: **no iteration**).
- Recall metric for span-budget tuning vs J-41 false ungrounded (`:48`).
- Exact QueryTrace field names for ratio + mode.
- Whether `+ResearchModeEvent` is wanted.
- REPL line-number drift since 2026-06-16.
- `SPAN_BUDGET_SENTENCES=5` vs 100–150 sources packing.

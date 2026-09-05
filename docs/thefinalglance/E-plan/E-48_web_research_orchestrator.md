# E-48: Web-research orchestrator hardening — editable plan stage + tiered search fan-out + cross-encoder rerank + speculative pruning + citation verification + contradiction node
Objective: Harden ELAI's existing web_research_orchestrator + research_tree pipeline with seven 2025-2026 SOTA additions: an editable plan stage before any search call, tiered search fan-out (Brave T1 + SearXNG T2 + Mojeek T3) with crawl4ai/Jina extractor layer, cross-encoder rerank top-50→top-5, FlashResearch speculative branch expansion + runtime pruning, conformal-thinking preemptive-unsolvable lower-threshold, per-claim citation verification (URL-liveness + arXiv-ID + CrossRef-DOI + SemanticCite faithfulness), and a never-silently-resolved Contradiction node in the Research Tree.
Estimated effort: 3 sessions, ~12 h
Pre-requisites: B-7 SHIPPED (web_research_orchestrator + ResearchTree + SourceQualityTable + dual_threshold convergence), B-39 SHIPPED (Chat REFLECT pipeline — precedent for "LLM as decision-maker" + 4 SSB action_types for REFLECT cycle), B-40 SHIPPED (Idea-Framer + FlashResearch multi-root dispatch), C-48 PLANNED (SafeHarbor TrustTier — citation trust marking).
Parallel-safe with: E-46 (different surface), E-17 (RAG retrieval quality — coordinates on rerank choice), E-19 (reasoning quality — coordinates on conformal-thinking convergence), E-13 TDD verification (different surface).
Source report: workflow/improvement-sweep-2026-05-19/reports/48-web-research-orchestrator.md

## Context

The May 2026 deep-research-orchestrator survey compares Anthropic Claude Research, OpenAI Deep Research, Gemini Deep Research, Perplexity DR, Grok DeepSearch, and Kagi Research Assistant against ELAI's existing FORMULATE → SEARCH → EXTRACT → REFLECT → CONVERGE pipeline and identifies seven additions ranked by signal-to-noise. The hallucinated-citation crisis is the load-bearing finding: GhostCite (arXiv:2602.06718) measured 14.23-94.93% hallucination rate across LLMs; 100 fabricated citations escaped 3-5 expert reviewers in 53 published papers per the NeurIPS 2025 Compound Deception taxonomy (arXiv:2602.05930); CITEVERIFIER + SemanticCite (84% acc) + CiteGuard (68%) are the documented detectors. Anthropic's multi-agent system (per simonwillison.net 2025/Jun/14) showed orchestrator+workers beats single-Opus by 90.2% with 15x token cost, and the "lead never cites — only the sub-agent that fetched the source can cite it" pattern is the canonical defense against lead-agent invention. FlashResearch (arXiv:2510.05145) demonstrates parallel speculative subquery expansion with breadth/depth/speculative branches and runtime pruning on intermediate evidence, NOT after full completion — directly mapped to ELAI's existing Research Tree per the report. Conformal Thinking (arXiv:2602.03814) introduces a dual-threshold-becomes-triple convergence pattern: goal-satisfied (upper, existing), quality-floor (existing), preemptive-unsolvable (new lower). Contradiction-surfacing is documented as a never-silently-resolve rule by arXiv:2504.00180 + arXiv:2511.06668 (healthcare) + arXiv:2510.03418 (LegalWiz). The report's recommended ELAI stack: keep SearXNG (T2 zero-cost), add Brave API (T1, ~$5/1k after free credits), add Mojeek (T3, only real non-Bing-derivative independent index), use crawl4ai (self-host Playwright) as primary extractor with Jina r.jina.ai/{URL} as fallback. ELAI's existing SourceQualityTable + WebResearchOrchestrator (per elai.map lines 66424, 130164) provides the surface to consume these.

## Existing state (B/C/D coverage)

- B-7 SHIPPED (`web_research_orchestrator.rs:166-396` per B-PLAN-SUMMARY.md line 203 + elai.map line 1064): FORMULATE → SEARCH → EXTRACT → REFLECT → CONVERGE loop; emits research_node_stage, integration_checkpoint_success. EXTENSION: E-48 adds editable-plan stage BEFORE SEARCH (Task 1); tiered fan-out REPLACES single-backend SEARCH (Task 2); contradiction node EXTENDS REFLECT (Task 7).
- B-7 SHIPPED `crates/web_search/src/source_quality.rs::SourceQualityTable` (per elai.map line 66424): existing tier-filter. EXTENSION: E-48 adds cross-encoder rerank AFTER the tier-filter, BEFORE LLM context (Task 3).
- B-7 SHIPPED `ResearchTree` with speculative_depth (per elai.map line 350493 + `prune_speculative_children` line 313462): existing speculative branch scaffold. EXTENSION: E-48 wires runtime-pruning trigger on intermediate evidence delta per FlashResearch (Task 4).
- B-40 SHIPPED (Idea-Framer with multi-root FlashResearch dispatch — `ResearchTree::new_multi_root(streams)` per B-PLAN-SUMMARY.md line 858; arXiv:2510.05145 cited): FlashResearch primitive already grounded. EXTENSION: E-48 adds the speculative-branch pruner on intermediate-evidence delta that was deferred in B-40.
- B-39 SHIPPED (Chat REFLECT pipeline; 4 SSB action_types ReflectTriaged/Candidate/AuditVerdict/Refined; types at `reasoning/`): precedent for LLM-as-decision-maker in ELAI. EXTENSION: E-48 reuses the REFLECT primitive shape for the per-claim citation-verification + contradiction-classification calls.
- C-48 PLANNED (SafeHarbor TrustTier T0..T3): EXTENSION: every citation produced by E-48 carries `trust_tier: TrustTier::T2_web_sourced` by default; SemanticCite-verified citations promote to T1_codebase_derived equivalence for re-use; unverified emit `citation_verification_failed` SSB row.
- Existing `web_research_orchestrator::extract_and_extract` (per elai.map line 1754) and SourceQualityTable tier-filter: E-48 inserts rerank between tier-filter and LLM-input chunk packing.

## Reads (mandatory before coding)

- `F:\.ELAI_workflow\workflow\improvement-sweep-2026-05-19\reports\48-web-research-orchestrator.md` (full)
- `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\web_research_orchestrator.rs` lines 149-700 (WebResearchOrchestrator struct + node_loop; per elai.map line 130164)
- `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\research_tree.rs` lines 909-1500 (PipelineSpec + ResearchTree + speculative pruning; per elai.map lines 289867, 350493, 313462)
- `F:\.ELAI_workflow\.elai_cc\crates\web_search\src\source_quality.rs` lines 50-150 (SourceQualityTable; per elai.map line 66424)
- `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\idea_framing.rs` lines 700-900 (B-40 FlashResearch dispatch; per elai.map lines 718-720)
- `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\tests\research_tree.rs` lines 470-490 (existing adversarial_contradiction_surfacing test; per elai.map line 1681)
- `F:\.ELAI_workflow\docs\thefinalglance\B-plan\B-PLAN-SUMMARY.md` lines 200-230 (B-7 web_research_orchestrator context), lines 820-880 (B-39 + B-40 context)
- workflow/papers/flashresearch-2510.05145-interface.md (B-40 already cites; re-read)
- workflow/papers/ghostcite-2602.06718-interface.md (after paper-ingestor mode=paper)
- workflow/papers/conformal-thinking-2602.03814-interface.md (after paper-ingestor mode=paper)
- workflow/papers/contradiction-rag-2504.00180-interface.md (after paper-ingestor mode=paper)
- workflow/papers/healthcare-rag-2511.06668-interface.md (after paper-ingestor mode=paper)
- workflow/papers/legalwiz-2510.03418-interface.md (after paper-ingestor mode=paper)
- https://docs.cohere.com/docs/reranking-best-practices (WebFetch — top-50 → top-5, 4093-token chunks)
- https://www.anthropic.com/research multi-agent system writeup via simonwillison.net/2025/Jun/14 (WebFetch — orchestrator + sub-agent citation lock)
- https://api-dashboard.search.brave.com/documentation/pricing (WebFetch — current Brave API tier/cost)
- https://www.mojeek.com (WebFetch — Mojeek API availability + 8.2B-page coverage)
- https://spider.cloud/blog/firecrawl-vs-crawl4ai-vs-spider-honest-benchmark (WebFetch — crawl4ai vs FireCrawl vs Jina Reader)

## Tasks

1. Editable plan stage in FORMULATE (Gemini-style "editable chain of thought"). Extend `web_research_orchestrator::node_loop` (per elai.map line 442) FORMULATE stage to emit a `ResearchPlan { sub_questions: Vec<SubQuestion>, expected_information_gain_rank: Vec<usize>, planned_search_budget: usize }` artifact BEFORE any search call. Sub-questions ranked by expected information gain via a simple deterministic heuristic: novelty score (token overlap with parent question; lower=higher gain) + specificity score (named entity count). New CLI flag `--interactive-research-plan` surfaces the plan to the user for edit/approve; default off (non-interactive runs auto-approve). Emit `research_plan_emitted` SSB row + `research_plan_approved` row when user-edited. Reference: gemini.google/overview/deep-research editable-CoT plan stage; Anthropic orchestrator pattern.

2. Tiered search fan-out (Brave T1 + SearXNG T2 + Mojeek T3). New module `.elai_cc/crates/web_search/src/tiered_search_pool.rs` (~300 LOC). Implements `TieredSearchPool { t1: Option<BraveClient>, t2: SearxngClient (existing), t3: Option<MojeekClient> }` with deterministic dispatch order T1 → T2 → T3 and per-tier rate-limit + free-quota tracking (Brave free credits exhaust → tier disabled with `tier_disabled_quota_exceeded` SSB row). Results deduplicated by canonical URL hash (blake3 of normalized URL — lowercase scheme+host, sorted query params, dropped utm_*). Per-tier results carry `source_tier: SourceTier::T1 | T2 | T3` for downstream tier-filter consumption. Config block `[web_search.tiered]` in `elai_cc.toml`: `brave_api_key`, `mojeek_api_key`, `searxng_url` (existing). If T1 + T3 keys absent, falls back to T2-only (existing behavior — no regression). The report explicitly flags DuckDuckGo as Bing-derived not independent — do NOT add DDG as a tier.

3. Extractor layer with crawl4ai (primary) + Jina Reader (fallback). Replace existing `extract_and_extract` (per elai.map line 1754) with `ExtractorPool { primary: Crawl4aiClient (HTTP to self-hosted), fallback: JinaReaderClient (r.jina.ai/{URL}) }`. Primary attempt with 10s timeout; on timeout/JS-render failure, fall back to Jina with 5s timeout; on both fail, emit `extraction_failed_all_extractors` SSB row + drop the source. Crawl4ai self-hosted endpoint URL configurable in `[web_search.extractor]` `elai_cc.toml`. Jina Reader is a managed HTTP GET — no API key required for free tier (verify current free-tier limits via WebSearch before shipping).

4. Cross-encoder rerank top-50 → top-5 between SourceQualityTable filter and LLM context. New module `.elai_cc/crates/web_search/src/cross_encoder_rerank.rs` (~200 LOC). After SourceQualityTable produces tier-filtered results (currently feeds LLM directly), pass top-50 (by tier-filter score) through a cross-encoder rerank step. Default reranker: local cross-encoder via sidecar `/rerank` endpoint (verify endpoint exists or add — per CLAUDE.md sidecar /search endpoint precedent). Fallback: Cohere Rerank v3.5 if configured (`[web_search.rerank].cohere_api_key`). Cohere chunking: 4093 tokens per chunk, max-pooled per-source relevance per cohere.com/docs/reranking-best-practices. Output top-5 sources flow into LLM context. Emit `cross_encoder_rerank_complete` SSB row with rank-shift histogram (how many sources moved which rank).

5. FlashResearch speculative branch + runtime pruner (extends B-40 dispatch). New module `.elai_cc/crates/orchestrator/src/research_tree_speculative.rs` (~250 LOC) OR extend `research_tree.rs` (verify approximate insertion point near `prune_speculative_children` per elai.map line 313462). Adds three branch types per FlashResearch arXiv:2510.05145: `BranchType::Breadth | Depth | Speculative`. Pruner runs after each REFLECT cycle: computes `evidence_delta = |new_evidence_set ∪ existing| - |existing|`; if `evidence_delta < SPECULATIVE_PRUNE_THRESHOLD` (default 0.10 ratio) AND branch_type == Speculative AND `low_gain_streak > 2`, prune the branch (re-uses existing `prune_speculative_children` per elai.map line 313462). Emit `speculative_branch_pruned` SSB row with branch_id + delta + streak. Adaptive planner: at each tree depth level, allocate speculative-branch budget = min(remaining_total_budget * 0.20, parent_node_estimated_value * 0.5).

6. Conformal-thinking triple-threshold convergence (extends existing dual-threshold). Extend `ConvergenceStatus` enum (per elai.map line 707, `web_research_orchestrator.rs::ConvergenceStatus`) with three explicit thresholds: `goal_satisfied_upper` (existing, e.g. confidence > 0.85), `quality_floor` (existing, e.g. tier-1-source count >= 2), `preemptive_unsolvable_lower` (NEW, e.g. confidence < 0.20 after >=N rounds AND evidence_delta < 0.05 across last 3 rounds). When `preemptive_unsolvable_lower` triggers, emit `research_preemptive_stop` SSB row and converge with `ConvergenceStatus::PreemptiveUnsolvable` (new variant). Calibration: per arXiv:2602.03814, the lower threshold should be calibrated against a held-out set of known-unsolvable research tasks. If ELAI lacks such a set, default the threshold to conservative (rarely fires) and emit `preemptive_stop_threshold_uncalibrated` warning at startup — open question #5 in the report.

7. Contradiction node in Research Tree, never silently resolved. New module `.elai_cc/crates/orchestrator/src/contradiction_node.rs` (~250 LOC). In REFLECT, for each new claim, run an O(n^2) contradiction-pair check against accumulated evidence in the current Research Tree path (cap at last 50 evidence rows to bound cost). Contradiction check is an LLM call to the sidecar with strict-JSON prompt: `{source_a, source_b} → {classification: "agree" | "disagree" | "orthogonal", reason}`. If `disagree` detected, materialize `ContradictionNode { source_a, source_b, classification_reason, surfaced_at }` into the tree, marked `prune_protected: true` (the existing pruner refuses to drop these). Renderer in final report MUST surface as "Source A (year/url) says X; Source B (year/url) says ~X; reconciliation: [unresolved]" — NEVER silently pick one. The existing `adversarial_contradiction_surfacing` test (per elai.map line 1681) is the precedent; extend it to assert the new ContradictionNode shape. Reference: arXiv:2504.00180 + arXiv:2511.06668 (healthcare RAG, never silently resolve) + arXiv:2510.03418 LegalWiz.

8. Per-claim citation verification pass (URL-liveness + arXiv-ID + CrossRef-DOI + SemanticCite faithfulness). New module `.elai_cc/crates/orchestrator/src/citation_verifier.rs` (~350 LOC). Runs as EXTRACT post-pass AND final REPORT post-pass. Four verification stages, cheapest-first: (a) URL-liveness — HEAD request, 5s timeout; 404/timeout/connection-error → `citation_dead_url` SSB row + drop the citation; (b) arXiv-ID regex match `arxiv:\d{4}\.\d{4,5}` + HTTP HEAD against `arxiv.org/abs/{id}` — 404 → `citation_invalid_arxiv` SSB row; (c) CrossRef DOI regex match `10\.\d+/.+` + HEAD against `doi.org/{doi}` — 404 → `citation_invalid_doi` SSB row; (d) SemanticCite-style faithfulness — sidecar call: "Given claim X and source-extract Y, does Y SUPPORT X? Strict JSON {decision: support|partial|contradict|orthogonal}". Mismatch → `citation_unfaithful` SSB row + flag in final report (but do NOT auto-drop; user-visible). Cite GhostCite arXiv:2602.06718 14-95% baseline; cite CITEVERIFIER/SemanticCite/CiteGuard tooling (per report). Aggregate metric: `citation_verification_pass_rate` SSB row per research run (percent of citations that survived stages a-d).

9. Sub-agent attribution lock (Anthropic pattern). Citations are `(agent_id, fetched_url, fetched_at_unix, source_tier)` tuples — the lead/orchestrator may NOT invent a citation; only the sub-agent (or in ELAI's case, the SEARCH-stage producer) may attach a citation. Wiring: `WebResearchOrchestrator` records the agent_id (`AgentRole`) that fetched each evidence row; the REFLECT/CONVERGE stages may only cite from this set. Refusal to cite outside the set emits `citation_attribution_violation_blocked` SSB row. Reference: Anthropic multi-agent system per simonwillison.net 2025/Jun/14.

10. Recency-weighted relevance for time-sensitive queries. Extend SourceQualityTable scoring with a `time_decay_multiplier`. Detect "temporal markers" in the question via regex: `\b(latest|current|2024|2025|2026|recent|today|this year)\b`. If detected, multiply each source's relevance score by `recency_multiplier = exp(-ln(2) * age_days / HALF_LIFE_DAYS)` with `HALF_LIFE_DAYS = 365` default (configurable in `[web_search.recency].half_life_days`). Older sources get down-weighted. Emit `recency_decay_applied` SSB row when active. Reference: arXiv:2509.19376 (Solving Freshness in RAG) + arXiv:2601.17824 OwlerLite.

11. Dashboard panels. Four small Svelte panels: `ResearchPlanPanel.svelte` (~120 LOC, editable-plan UI for Task 1 interactive mode), `TieredSearchHealthPanel.svelte` (~100 LOC, T1/T2/T3 backend health + quota), `CitationVerificationGauge.svelte` (~100 LOC, pass-rate over last N runs + dead-URL count), `ContradictionsPanel.svelte` (~120 LOC, surfaced contradictions across recent research). Four Tauri commands: `get_research_plan_pending_edit`, `get_tiered_search_health`, `get_citation_verification_stats`, `get_recent_contradictions`. Mount under `lib/drawers/ReasoningDrawer.svelte`.

## WIRING_CONTRACT

produces_symbols:
  - orchestrator::web_research_orchestrator::ResearchPlan { sub_questions, expected_information_gain_rank, planned_search_budget }
  - web_search::tiered_search_pool::TieredSearchPool { t1, t2, t3 }
  - web_search::tiered_search_pool::SourceTier::{T1, T2, T3}
  - web_search::extractor_pool::ExtractorPool { primary: Crawl4aiClient, fallback: JinaReaderClient }
  - web_search::cross_encoder_rerank::CrossEncoderRerank::rerank(top50: Vec<Source>) -> Vec<Source>
  - orchestrator::research_tree::BranchType::{Breadth, Depth, Speculative}
  - orchestrator::web_research_orchestrator::ConvergenceStatus::PreemptiveUnsolvable (variant extension)
  - orchestrator::contradiction_node::ContradictionNode { source_a, source_b, classification_reason, surfaced_at, prune_protected }
  - orchestrator::citation_verifier::CitationVerifier::verify(citation: &Citation) -> VerificationOutcome
  - orchestrator::citation_verifier::VerificationOutcome { url_live, arxiv_valid, doi_valid, faithful }
  - web_search::source_quality::SourceQualityTable::with_recency_decay (extension)
consumes_symbols:
  - crates/orchestrator/src/web_research_orchestrator.rs::WebResearchOrchestrator (FORMULATE/SEARCH/EXTRACT/REFLECT/CONVERGE extension)
  - crates/orchestrator/src/research_tree.rs::ResearchTree (speculative branch + contradiction-node insertion)
  - crates/web_search/src/source_quality.rs::SourceQualityTable (rerank insertion point)
  - crates/orchestrator/src/idea_framing.rs::FlashResearch dispatch (B-40 — speculative branch pruner extension)
  - blake3 (URL canonicalization hash for dedup)
  - reqwest crate (verify presence via Cargo.lock Grep; HEAD requests for URL-liveness)
reachable_from:
  - crates/cli/src/main.rs (new `--interactive-research-plan` flag; new `[web_search.*]` config blocks)
  - crates/orchestrator/src/web_research_orchestrator.rs::node_loop (Tasks 1, 5, 6, 7, 8 wired here)
  - crates/web_search/src/lib.rs (Tasks 2, 3, 4, 10 wired here)
  - crates/orchestrator/src/idea_framing.rs::FlashResearch dispatch (Task 5 speculative pruner consumed here)
stub_ban:
  - no `unimplemented!()` in any new module
  - no `.unwrap()` outside `#[cfg(test)]`
  - no silent contradiction resolution — every Disagree classification MUST produce a ContradictionNode AND a report-renderer line
  - no citation emitted in final report without going through `CitationVerifier::verify` first (compile-time enforced via type system: `VerifiedCitation` wrapper)
  - no lead/orchestrator citation outside the SEARCH-stage-produced attribution set (Task 9)
  - blake3 ONLY for URL canonicalization hash; no blake2/sha2
  - no DuckDuckGo as a tier — explicit comment + lint
  - no SearXNG-only fallback when T1/T3 keys missing without emitting `tiered_search_degraded_to_t2_only` SSB row
  - speculative branch pruner must NEVER prune a ContradictionNode (Task 7 `prune_protected: true`)
  - Cohere Rerank API call is opt-in via API key; default reranker is local sidecar /rerank endpoint
frontend_pair: crates/dashboard-v2/src/lib/components/ResearchPlanPanel.svelte (~120 LOC); TieredSearchHealthPanel.svelte (~100 LOC); CitationVerificationGauge.svelte (~100 LOC); ContradictionsPanel.svelte (~120 LOC)
observable_effect:
  trigger: synthetic research run on the question "Which Python web framework is fastest in 2026?" with seeded contradictory evidence (FastAPI fastest in source A, Litestar fastest in source B) and one fabricated citation `arxiv:9999.99999`
  expect_ssb: ≥1 `research_plan_emitted` row, ≥1 `cross_encoder_rerank_complete` row, ≥1 `contradiction_node_materialized` row for the FastAPI/Litestar pair, ≥1 `citation_invalid_arxiv` row for `arxiv:9999.99999`, ≥1 `citation_verification_pass_rate` row in final report
  expect_artifact: final report contains "Source A says FastAPI fastest; Source B says Litestar fastest; reconciliation: [unresolved]"; `arxiv:9999.99999` removed from citations or flagged dead

## DETERMINISTIC_CHECK

```bash
# 1. Editable plan stage emits before SEARCH
rg "ResearchPlan|research_plan_emitted" .elai_cc/crates/orchestrator/src/web_research_orchestrator.rs
# Expected: ≥2 hits

# 2. Tiered search pool with 3 tiers, no DuckDuckGo
rg "SourceTier::T1|SourceTier::T2|SourceTier::T3" .elai_cc/crates/web_search/src/tiered_search_pool.rs
# Expected: ≥3 hits
rg -i "duckduckgo|DDG" .elai_cc/crates/web_search/src/tiered_search_pool.rs
# Expected: zero hits (or only in a no-DDG-as-tier comment)

# 3. Extractor pool with crawl4ai primary + Jina fallback
rg "Crawl4aiClient|JinaReaderClient" .elai_cc/crates/web_search/src
# Expected: ≥2 hits each

# 4. Cross-encoder rerank between tier-filter and LLM context
rg "fn rerank.*top.*50|cross_encoder_rerank_complete" .elai_cc/crates/web_search/src/cross_encoder_rerank.rs
# Expected: ≥2 hits

# 5. Speculative branch + runtime pruner
rg "BranchType::Speculative|speculative_branch_pruned" .elai_cc/crates/orchestrator/src
# Expected: ≥2 hits

# 6. Conformal triple-threshold + PreemptiveUnsolvable variant
rg "PreemptiveUnsolvable|research_preemptive_stop" .elai_cc/crates/orchestrator/src
# Expected: ≥2 hits

# 7. ContradictionNode never silently resolved
rg "ContradictionNode|prune_protected.*true" .elai_cc/crates/orchestrator/src/contradiction_node.rs
# Expected: ≥2 hits
rg "Source A.*says.*Source B.*says.*reconciliation" .elai_cc/crates/orchestrator/src
# Expected: ≥1 hit (renderer)

# 8. CitationVerifier 4 stages + VerifiedCitation type
rg "url_live|arxiv_valid|doi_valid|faithful" .elai_cc/crates/orchestrator/src/citation_verifier.rs
# Expected: ≥4 hits
rg "VerifiedCitation" .elai_cc/crates/orchestrator/src
# Expected: ≥1 hit

# 9. Sub-agent attribution lock
rg "citation_attribution_violation_blocked" .elai_cc/crates/orchestrator/src
# Expected: ≥1 hit

# 10. blake3 for URL canonicalization (anti-stale)
rg "blake3" .elai_cc/crates/web_search/src/tiered_search_pool.rs
# Expected: ≥1 hit
rg "blake2|sha2" .elai_cc/crates/web_search/src/tiered_search_pool.rs
# Expected: zero hits

# 11. Smoke: contradictory-evidence + fabricated-arxiv fixture
cargo test -p orchestrator adversarial_contradiction_surfacing
# Expected: PASS (existing test extended)
cargo test -p orchestrator citation_verifier_drops_dead_arxiv
# Expected: PASS
cargo test -p orchestrator preemptive_unsolvable_stops_after_low_delta_streak
# Expected: PASS
```

## ACCEPTANCE

- [ ] Editable plan stage emits before SEARCH; `--interactive-research-plan` flag surfaces to user when set
- [ ] TieredSearchPool with Brave T1 + SearXNG T2 + Mojeek T3; deterministic dispatch order; URL-canonical-hash dedup; per-tier quota tracking
- [ ] DuckDuckGo NOT a tier (explicit comment + zero references in tiered_search_pool.rs)
- [ ] ExtractorPool crawl4ai-primary + Jina-fallback; both timeouts honored; `extraction_failed_all_extractors` on double-fail
- [ ] Cross-encoder rerank top-50 → top-5 between SourceQualityTable filter and LLM context; default local sidecar /rerank, opt-in Cohere v3.5
- [ ] FlashResearch speculative branch type + runtime pruner on evidence_delta < 0.10 ratio + low_gain_streak > 2
- [ ] ConvergenceStatus triple-threshold including PreemptiveUnsolvable; uncalibrated warning at startup
- [ ] ContradictionNode materialized on Disagree classifications; prune_protected = true; final report renderer surfaces both sources
- [ ] CitationVerifier 4-stage pass: URL-liveness, arXiv-ID, CrossRef-DOI, SemanticCite-faithfulness; VerifiedCitation type wraps final-report citations
- [ ] Sub-agent attribution lock: lead may not invent citations outside SEARCH-stage attribution set
- [ ] Recency-weighted SourceQualityTable when temporal markers present in question
- [ ] Four dashboard panels + four Tauri commands wired
- [ ] All new SSB action_types contract-tested
- [ ] No regression on B-7 existing convergence (dual-threshold still active alongside new lower-threshold)

## Tests to add

- `crates/orchestrator/tests/research_plan_emitted_before_search.rs`: asserts SEARCH stage refuses to run if `research_plan_emitted` SSB not present
- `crates/web_search/tests/tiered_search_dedups_by_canonical_url.rs`: same article on T1 and T2 with utm_* params; dedup'd
- `crates/web_search/tests/extractor_pool_crawl4ai_fail_falls_back_to_jina.rs`: seed crawl4ai timeout; assert Jina invocation + result
- `crates/web_search/tests/cross_encoder_rerank_top50_to_top5.rs`: 50 fake sources; assert exactly 5 in top-5 output + SSB row with rank-shift
- `crates/orchestrator/tests/speculative_branch_pruned_on_low_delta.rs`: seed evidence_delta < 0.10 + streak > 2; assert prune + SSB
- `crates/orchestrator/tests/preemptive_unsolvable_stops_after_low_delta_streak.rs`: low-confidence + low-delta-across-N rounds; assert ConvergenceStatus::PreemptiveUnsolvable
- `crates/orchestrator/tests/citation_verifier_drops_dead_arxiv.rs`: `arxiv:9999.99999` HEAD → 404; assert dropped + `citation_invalid_arxiv` SSB
- `crates/orchestrator/tests/citation_verifier_unfaithful_flagged_not_dropped.rs`: semantic mismatch; assert flagged in report but NOT dropped
- `crates/orchestrator/tests/contradiction_node_never_pruned.rs`: speculative pruner attempt on ContradictionNode; assert refusal
- `crates/orchestrator/tests/adversarial_contradiction_surfacing.rs` EXTEND existing (per elai.map line 1681) to assert ContradictionNode + report renderer line
- `crates/orchestrator/tests/sub_agent_attribution_lock.rs`: lead attempts citation outside SEARCH set; assert blocked + SSB
- `crates/contracts/tests/research_plan_contract.rs`, `tiered_search_pool_contract.rs`, `citation_verifier_contract.rs`, `contradiction_node_contract.rs`: one contract test per major surface

## Files touched

- CREATE: `crates/web_search/src/tiered_search_pool.rs` (~300 LOC)
- CREATE: `crates/web_search/src/extractor_pool.rs` (~200 LOC; Crawl4ai + Jina clients)
- CREATE: `crates/web_search/src/cross_encoder_rerank.rs` (~200 LOC)
- CREATE: `crates/orchestrator/src/research_tree_speculative.rs` (~250 LOC; OR extend research_tree.rs)
- CREATE: `crates/orchestrator/src/contradiction_node.rs` (~250 LOC)
- CREATE: `crates/orchestrator/src/citation_verifier.rs` (~350 LOC)
- CREATE: `crates/dashboard-v2/src/lib/components/ResearchPlanPanel.svelte` (~120 LOC)
- CREATE: `crates/dashboard-v2/src/lib/components/TieredSearchHealthPanel.svelte` (~100 LOC)
- CREATE: `crates/dashboard-v2/src/lib/components/CitationVerificationGauge.svelte` (~100 LOC)
- CREATE: `crates/dashboard-v2/src/lib/components/ContradictionsPanel.svelte` (~120 LOC)
- MODIFY: `crates/orchestrator/src/web_research_orchestrator.rs` (+~150 LOC for FORMULATE plan + verifier integration + attribution lock; verify insertion at node_loop per elai.map line 442)
- MODIFY: `crates/orchestrator/src/research_tree.rs` (+~80 LOC for BranchType + prune-protected for ContradictionNode + speculative pruner extension; verify near `prune_speculative_children` line 313462)
- MODIFY: `crates/web_search/src/source_quality.rs` (+~60 LOC for recency-decay multiplier; verify insertion at SourceQualityTable line 66424)
- MODIFY: `crates/web_search/src/lib.rs` (+~30 LOC for new module wiring)
- MODIFY: `.elai_cc/elai_cc.toml` (+~35 lines: `[web_search.tiered]`, `[web_search.extractor]`, `[web_search.rerank]`, `[web_search.recency]`, `[web_search.preemptive_stop]` blocks)
- MODIFY: `crates/cli/src/main.rs` (+~20 LOC for `--interactive-research-plan` flag)
- MODIFY: `crates/dashboard-v2/src-tauri/src/commands.rs` (+~80 LOC for 4 new Tauri commands)
- CREATE: `workflow/reasoning/E-48-decisions.md`

## Citations / Grounding

- arXiv:2510.05145 FlashResearch / ParallelResearch — tree-structured deep research with parallel breadth/depth/speculative branches + runtime pruning; grounds Tasks 1, 5
- Anthropic multi-agent system via simonwillison.net 2025/Jun/14 — orchestrator+workers beats single-Opus 90.2%; lead-never-cites attribution lock; grounds Tasks 1, 9
- https://openai.com/index/introducing-deep-research/ + cdn.openai.com/deep-research-system-card.pdf — OpenAI o3 DR 5-30 min autonomous; grounds Task 1 (editable plan compared against; ELAI ships explicit edit)
- https://gemini.google/overview/deep-research/ — editable-CoT plan stage; grounds Task 1
- arXiv:2602.06718 GhostCite — 14.23-94.93% LLM citation hallucination; 1.07% of papers at top AI/ML venues 2020-2025 contain invalid cites, +80.9% growth in 2025; grounds Task 8 (highest-leverage defense)
- arXiv:2602.05930 Compound Deception NeurIPS 2025 taxonomy — 100 fabricated citations in 53 published papers; grounds Task 8 + Task 9 attribution lock
- arXiv:2604.03173 — CITEVERIFIER, SemanticCite (84% acc), CiteGuard (68%) detectors; grounds Task 8 stage-d faithfulness
- arXiv:2601.18724 HalluCitation Matters — arxiv-ID validation is highest-yield cheap defense; grounds Task 8 stage-b
- arXiv:2602.03814 Conformal Thinking — dual-threshold reasoning stop; distribution-free risk control; grounds Task 6
- arXiv:2504.00180 Contradiction Detection in RAG — LLM-as-context-validator; grounds Task 7
- arXiv:2511.06668 Healthcare contradictory-evidence RAG — never-silently-resolve canonical pattern; grounds Task 7 renderer line
- arXiv:2510.03418 LegalWiz — multi-agent + consistency-voting; grounds Task 7 structural-output rule
- arXiv:2509.19376 Solving Freshness in RAG — recency-weighted scoring; grounds Task 10
- arXiv:2601.17824 OwlerLite scope+freshness web retrieval — grounds Task 10
- arXiv:2509.23519 ReliabilityRAG — per-source reliability scores; grounds Task 2 tier-scoring rationale
- arXiv:2410.22954 — source-reliability heterogeneity overlooked by naive retrieval; grounds tiered-fan-out
- https://docs.cohere.com/docs/reranking-best-practices — 4093-token chunks, max-pooled, top-50→top-5; grounds Task 4
- https://api-dashboard.search.brave.com/documentation/pricing + costbench.com — $5/1k after free credits; grounds Task 2 cost model
- https://www.mojeek.com (via webpronews + makeuseof) — 8.2B-page truly independent index; only non-Bing-derivative; grounds Task 2 T3 choice
- https://spider.cloud/blog/firecrawl-vs-crawl4ai-vs-spider-honest-benchmark — Crawl4AI free self-host; grounds Task 3 primary extractor choice
- factually.co private-search 2026 — DuckDuckGo Bing-derived not independent; grounds Task 2 anti-pattern guard
- arXiv:2510.12164 Survey on Parallel Reasoning — speculative pruning canon; grounds Task 5
- CLAUDE.md anti-stale-training — grounds blake3-only URL canonicalization

## Open questions / risks

- Conformal lower-threshold (Task 6) requires a calibration set of known-unsolvable research tasks. ELAI does not have one. Default proposed: conservative threshold + startup warning; revisit when ELAI accumulates 30+ tagged-unsolvable cases. Risk: false-positive preemptive stops on solvable-but-hard tasks.
- Brave API free tier dynamics (Task 2): per costbench.com, new-user free credits shrunk Feb-2026. Risk: tier T1 silently disabled mid-session when quota exhausts. Mitigation: `tier_disabled_quota_exceeded` SSB row + dashboard alert; T2 SearXNG fallback always available.
- Mojeek API (Task 2): coverage is 8.2B pages vs Google scale. Risk: T3 returns empty on niche queries. Mitigation: T3 is third in cascade; downstream dedup handles empty T3 cleanly.
- Cross-encoder rerank (Task 4) via local sidecar /rerank endpoint: endpoint may not yet exist; verify before shipping. If absent, Task 4 requires sidecar extension OR defaults to Cohere with API key required. Open question for the user: ship Cohere-default with no-key fallback to no-rerank, OR delay Task 4 until sidecar /rerank ships?
- FlashResearch speculative pruner threshold 0.10 (Task 5) is a guess. Open question: tune on a 50-query benchmark before locking the default? Default proposed: ship the threshold + log per-prune `evidence_delta` so calibration data accrues automatically.
- Citation faithfulness via sidecar (Task 8 stage-d) costs an extra LLM call per citation. Budget impact: ~10-30% more sidecar tokens per research run. Open question: gate behind `[web_search.citation_verifier].faithfulness_check = false` default? Default proposed: true with `--no-faithfulness-check` opt-out; per report's Q3 the marginal hallucination-rate reduction is high enough that the budget cost is worth it.
- ContradictionNode O(n^2) check (Task 7) on >50 evidence rows is bounded; deeper trees may need a budget. Mitigation: cap at last 50 rows per the task spec; emit `contradiction_check_budget_clamped` SSB row when clamped.
- Sub-agent attribution lock (Task 9) requires the orchestrator to propagate `agent_id` on every evidence row. Existing SSB rows may not carry this. Open question: backfill old rows on read, or only enforce going forward? Default proposed: enforce going forward; existing rows are pre-attribution-lock and unverifiable.
- Recency decay (Task 10) regex for temporal markers ("latest", year tokens, "current"): risk of false positives on questions like "current state of Rust async" where the user wants both recent + foundational. Mitigation: half-life 365 days is generous; foundational sources remain visible just slightly down-weighted.

## Next

E-48 directly extends B-7 and B-40 with the seven highest-impact additions identified by the May 2026 deep-research survey. After E-48, ELAI's web-research pipeline matches Anthropic + Gemini + OpenAI DR on the editable-plan + multi-backend + rerank + speculative-pruning surface AND adds two surfaces those products don't ship visibly: (a) the never-silently-resolved ContradictionNode (matches the healthcare/legal RAG safety pattern), (b) per-claim 4-stage CitationVerifier defending against the GhostCite-documented 14-95% hallucination baseline. Downstream: E-49 (coding-agent metrics) can consume `citation_verification_pass_rate` as a quality canary; E-46's process-reward scalar can be supplemented by `evidence_delta` from the FlashResearch pruner; C-48 SafeHarbor TrustTier rows flow naturally from `VerifiedCitation` trust-tier promotion. The single biggest remaining gap is the conformal-stop calibration set — captured in the Open questions block above and deferred to a later patch when ELAI accumulates the tagged-unsolvable cases.

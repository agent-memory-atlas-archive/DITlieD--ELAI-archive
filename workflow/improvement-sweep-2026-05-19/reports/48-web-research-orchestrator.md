# Deep research orchestrator for ELAI (May 2026)

## TL;DR
- FORMULATE: emit an editable plan (Gemini-style "editable chain of thought") and rank sub-questions by expected information gain BEFORE any search call; ELAI's flat FORMULATE step under-uses planning compute that all SOTA systems (Anthropic, Gemini, OpenAI DR) now spend up-front.
- SEARCH: switch from single SearXNG to a tiered fan-out (Brave API T1, SearXNG T2, Mojeek T3 for independent index coverage) plus an extraction layer (crawl4ai self-hosted as primary, Jina Reader as fallback) rather than the current WebFetcher+BasicExtractor pair.
- EXTRACT: insert a Cohere-rerank-v3.5-style cross-encoder rerank (top-50 -> top-5) between the SourceQualityTable tier-filter and the LLM context, and chunk at 4093 tokens with max-pooled relevance per source.
- REFLECT: implement parallel speculative subquery expansion per FlashResearch (arxiv 2510.05145) - breadth+depth+speculative branches, prune on intermediate evidence, NOT after full completion; carries to the existing Research Tree directly.
- CONVERGE: keep dual-threshold but ADD a conformal-thinking lower-threshold "preemptive stop on unsolvable" gate (arxiv 2602.03814) and a citation-verification pass against arxiv/CrossRef/URL-live-check before report finalization - this is the cheapest defense against hallucinated sources (14-95% rate across LLMs per GhostCite).

## Key sources
| URL/arxiv | what | takeaway |
|---|---|---|
| arxiv:2510.05145 | FlashResearch / ParallelResearch | tree-structured deep research; parallelism across breadth, depth, speculative; adaptive planner + runtime pruner + async exec |
| https://www.anthropic.com/research multi-agent system (simonwillison.net/2025/Jun/14) | Anthropic Claude Research | orchestrator-worker; Opus lead + Sonnet workers beats single-Opus by 90.2%; tokens explain 80% of perf variance; 15x token cost |
| https://openai.com/index/introducing-deep-research/ + cdn.openai.com/deep-research-system-card.pdf | OpenAI Deep Research | o3-tuned-for-browsing + o3-mini summarizer-of-thought; 26.6% on Humanity's Last Exam; 5-30 min autonomous runs |
| https://gemini.google/overview/deep-research/ + ai.google.dev/gemini-api/docs/deep-research | Gemini Deep Research | editable-CoT plan stage; iterative-loop (plan->search->read->refine); user-steerable before execution |
| https://www.perplexity.ai/hub/blog/introducing-perplexity-deep-research | Perplexity DR | iterative search-read-reason; Spaces = persistent custom-instruction + file-scoped workspaces |
| https://x.ai/news/grok-3 + xAI DeepSearch | Grok DeepSearch/DeeperSearch | multi-agent: sentiment agent + financial agent + competitor agent + synthesis agent; X corpus integration |
| arxiv:2405.13576 | FlashRAG modular toolkit | 5 core modules + 16 subcomponents; hierarchical env/component/pipeline split = useful pipeline template |
| arxiv:2504.00180 | Contradiction Detection in RAG | LLMs as context-validators; performance varies by contradiction type; CoT mixed-results |
| arxiv:2511.06668 | Healthcare contradictory-evidence RAG | reconcile outdated vs current; structural template for ELAI's "never silently resolve" rule |
| arxiv:2510.03418 | LegalWiz multi-agent contradiction | parallel agent retrieval + consistency-voting |
| arxiv:2602.03814 | Conformal Thinking | dual threshold reasoning stop: upper=confidence stop, lower=preemptive unsolvable stop; distribution-free risk control |
| arxiv:2509.19376 | Solving Freshness in RAG | recency-weighted scoring fused with semantic; heuristic trend-detection limits documented |
| arxiv:2601.17824 | OwlerLite scope+freshness web retrieval | user-defined scope + freshness central to retrieval ranking |
| GhostCite arxiv:2602.06718 + arxiv:2602.05930 (Compound Deception NeurIPS taxonomy) | Hallucinated-citation crisis | 14-95% hallucination rate across LLMs; 100 fake cites in 53 NeurIPS-2025 papers; CITEVERIFIER, SemanticCite (84%), CiteGuard (68%) detectors |
| arxiv:2509.23519 ReliabilityRAG | Provably robust RAG against web spam | per-source reliability scores |
| arxiv:2410.22954 | Source Reliability estimation in RAG | retrieval overlooks source-reliability heterogeneity; assign reliability scores |
| arxiv:2510.12164 | Survey on Parallel Reasoning | covers speculative CoT, tree-of-thought validator, parallel decoding with custom attention masks |
| arxiv:2504.19095 | Speculative Chain-of-Thought | draft+verify at step level not token level |
| arxiv:2506.11763 DeepResearch Bench + arxiv:2508.15804 ReportBench + arxiv:2510.02190 + arxiv:2510.00172 DRBench + arxiv:2512.17776 DEER | DR benchmarks | 5 active benchmarks: comprehensiveness, citation accuracy, insight recall, distractor avoidance, faithfulness against original source |
| https://docs.cohere.com/docs/reranking-best-practices | Cohere Rerank v3.5 | 4093-token chunks, max-pooled per doc; top-50 -> top-5 standard pattern |
| https://api-dashboard.search.brave.com/documentation/pricing + costbench.com | Brave Search API | new users: $5/mo credits ~ 1k queries; legacy 2k/mo free for existing; $5/1k after |
| https://help.kagi.com/kagi/search-details/search-sources.html + blog.kagi.com/kagi-assistants | Kagi Teclis/TinyGem + Kagi Research Assistant | own indices for small-web; topped SimpleQA Aug-2025 with Gemini 2.0 Flash backing |
| https://www.mojeek.com (via webpronews + makeuseof) | Mojeek | 8.2B-page truly independent index, has API; only viable non-Bing-derivative independent in DDG/Mojeek/Marginalia trio |
| https://spider.cloud/blog/firecrawl-vs-crawl4ai-vs-spider-honest-benchmark + scrapeless.com | Crawl4AI vs FireCrawl vs Jina Reader | Crawl4AI = self-host Playwright, free, primary for >10k pages; Jina r.jina.ai/{URL} = simplest fallback; FireCrawl = managed JS-render |
| https://www.tavily.com + https://exa.ai + data4ai.com | Tavily vs Exa | Tavily keyword+breadth; Exa semantic 81% vs Tavily 71% on complex retrieval; both have Research/extract endpoints |

## DeepResearch product matrix
| product | search depth | source quality | contradiction handling | citation discipline | citation |
|---|---|---|---|---|---|
| Anthropic Claude Research | orchestrator + parallel sub-agents, 15x tokens vs single-agent | LLM-judged per sub-agent; no explicit tier | not surfaced explicitly; consolidated by lead | inline w/ source attribution in final report | simonwillison.net/2025/Jun/14, blog.bytebytego.com |
| OpenAI Deep Research (o3) | 5-30 min autonomous, iterative browse+reason | LLM heuristics + browse history; o3-mini summarizes CoT | partial - flagged in report body, not formal | inline citations, post-hoc-verified by team but unverified at runtime | openai.com/index/introducing-deep-research, system card Feb-2025 |
| Gemini Deep Research / DR Max | editable plan; parallel search + sequential iter | grounding service + Google Search; tier implicit | iterative re-search on discrepancy | inline w/ link cards; Vancouver-ish in PDF export | gemini.google/overview/deep-research, blog.google |
| Perplexity Deep Research | iterative search-read-reason, dozens-of-searches | source-list visible to user; no explicit tier | none surfaced - resolved silently | inline numbered footnotes; URL list | perplexity.ai/hub/blog/introducing-perplexity-deep-research |
| Grok DeepSearch / DeeperSearch | multi-agent: sentiment / financial / competitor / synth; X+web | X-recency + web cross-ref | "reason about conflicting facts" claim in marketing; impl undisclosed | inline source list | x.ai/news/grok-3, mediapost Feb-2025 |
| Kagi Research Assistant | Teclis+TinyGem indices + Wolfram/news tools | own-quality-ranked indices (ads/trackers down-weighted) | not surfaced | inline | help.kagi.com, blog.kagi.com/kagi-assistants |

## Search backend comparison
| Backend | Index | Cost | LLM agent fit | Tradeoff |
|---|---|---|---|---|
| SearXNG (current) | metasearch of Bing/Google/etc | $0 + self-host | good baseline, no per-query fee | maintenance burden; result quality = aggregated upstream; rate-limit fragility (firecrawl.dev/blog/brave-search-api-alternatives) |
| Brave Search API | own index | $5/1k after free credits (2k/mo legacy) | strong (~50 rps cap); single API | new-user free tier shrunk Feb-2026; vendor lock cost (costbench.com/software/ai-search-apis/brave-search-api/free-plan, brave.com/search/api) |
| Kagi Search API | Teclis+TinyGem own indices | paid only, no free tier | excellent for non-commercial / small-web | small-web bias = misses big commercial sources; cost (help.kagi.com) |
| Mojeek API | own 8.2B-page independent | paid commercial | only real non-Bing-derivative non-Google independent index | 8.2B is small vs Google scale; coverage gaps (makeuseof Mojeek 2026, webpronews) |
| DuckDuckGo | Bing-derived | free no API | privacy facade, not independent | NOT a true independent source - duplicates Bing-derived (factually.co private-search comparison 2026) |
| Marginalia | own indie-web crawl | free | great for blog/indie content tier | tiny coverage; weak for mainstream queries (indieseek.xyz) |
| Tavily | meta + own enrichment | paid tiers | Research endpoint = one-call deep search | keyword bias; 71% complex-retrieval vs Exa 81% (data4ai.com Exa vs Tavily) |
| Exa | semantic neural search | paid tiers | best for "find papers about X concept" | newer index; commercial pricing (exa.ai/versus/tavily) |
| crawl4ai (extraction) | self-host Playwright | $0 + host | open ownership; sites with JS; agentic patterns | maintenance; not a search engine - extraction only (spider.cloud honest benchmark) |
| FireCrawl (extraction) | managed | paid per page | clean LLM-Markdown, JS-render | per-page cost adds up (firecrawl.dev/blog/best-deep-research-apis) |
| Jina Reader r.jina.ai/{URL} | managed | free tier + paid | simplest possible URL->Markdown | rate limits; not for high-volume (scrapeless.com crawl4ai vs firecrawl) |

Recommended ELAI stack: keep SearXNG as zero-cost baseline (T2), add Brave API as T1 with throttle to free quota, add Mojeek as T3 for independent-index diversity (defense against Bing/Google monoculture), use crawl4ai as primary extractor (self-host, no per-call fee, JS-render), keep Jina Reader as fallback for sites crawl4ai fails on. SearXNG alone is brittle - all major DR systems fan out across multiple backends.

## Hallucinated-source defense patterns
- Per GhostCite (arxiv:2602.06718): 14.23-94.93% hallucination rate across LLMs on citations; 1.07% of papers at top AI/ML venues 2020-2025 contain invalid citations, +80.9% growth in 2025 alone. (cited)
- Compound Deception NeurIPS 2025 taxonomy (arxiv:2602.05930): 100 fabricated citations escaped 3-5 expert reviewers in 53 published papers; not typos but invented authors+titles+pubs.
- CITEVERIFIER (open-source large-scale verification), SemanticCite (84% accuracy classifying support), CiteGuard (68% human-level attribution; fails silently on fabricated URLs). (per arxiv:2604.03173)
- Pattern 1 (URL-liveness): HEAD request every cited URL; 404/timeout = drop or flag. CiteGuard's silent-fail on fabricated URLs proves this is necessary even when LLM output "looks confident".
- Pattern 2 (arxiv ID validate): every arxiv:YYMM.NNNNN regex-match goes through arxiv API existence check; this is the single highest-yield cheap defense per HalluCitation Matters (arxiv:2601.18724).
- Pattern 3 (CrossRef DOI validate): same for DOIs.
- Pattern 4 (semantic faithfulness): SemanticCite-style classifier compares LLM's claim about source vs source content; flag mismatch. (84% accuracy is ceiling).
- Pattern 5 (Anthropic-style separation): lead never cites - only the sub-agent that fetched the source can cite it; lead synthesizes referencing sub-agent's verified IDs. (per simonwillison.net/2025/Jun/14).

## Contradiction-surfacing patterns
- arxiv:2504.00180 Contradiction Detection in RAG: LLMs as context-validators - explicit prompted classification of (source_A, source_B) pairs into agree/disagree/orthogonal; CoT helps inconsistently, larger models help more.
- arxiv:2511.06668 Healthcare RAG safety: reconcile-not-silently-resolve - retrieved abstracts with conflicting findings MUST both be surfaced; system answers "Source A (2023) recommends X; Source B (2024) recommends not-X; reconciliation unclear" rather than picking one. Direct match to ELAI's existing rule.
- arxiv:2510.03418 LegalWiz: multi-agent generation framework - one agent per source family, central consistency-voting; contradictions surfaced as structured output rather than prose.
- Hierarchical-Agentic-RAG pattern (emergentmind hierarchical-agentic-RAG topic): top-tier "reflection + strategic source selection", mid-tier "coverage + contradiction assessment" as a distinct REFLECT node - matches ELAI's REFLECT step exactly; not yet wired.
- Recommended ELAI hook: in REFLECT, for each claim, run an O(n^2) contradiction-pair check against accumulated evidence; if >= 1 contradicting source detected, emit a CONTRADICTION node into the Research Tree that is NEVER pruned and is rendered in the final report as "Source X says A; Source Y says ~A; reconciliation: [unresolved]". Anthropic's lead-agent reconcile-or-surface gate is the same pattern.

## Best techniques to adopt
| Rank | Technique | Grounding | Integration hook |
|---|---|---|---|
| 1 | Editable plan stage with sub-question ranking BEFORE searches | Gemini DR (gemini.google/overview/deep-research), Anthropic orchestrator (simonwillison.net 2025/Jun/14) | FORMULATE: emit plan, optionally surface to user, only then dispatch SEARCH |
| 2 | Multi-backend fan-out (Brave T1 + SearXNG T2 + Mojeek T3) + diversified-index defense | firecrawl.dev brave-alternatives, makeuseof Mojeek, costbench Brave free-plan | SEARCH: replace single SearxClient with TieredSearchPool, deterministic order, dedupe by URL |
| 3 | Cross-encoder rerank top-50 -> top-5 (Cohere v3.5 or local equivalent) | docs.cohere.com/docs/reranking-best-practices, arxiv:2505.13757 CoRank | EXTRACT: between SourceQualityTable filter and LLM-input chunk packing |
| 4 | Speculative branch expansion + runtime pruning (breadth/depth/speculative) | arxiv:2510.05145 FlashResearch | REFLECT: existing Research Tree gets a "speculative" branch type pruned on evidence delta |
| 5 | Conformal lower-threshold preemptive-unsolvable stop | arxiv:2602.03814 Conformal Thinking | CONVERGE: dual-threshold becomes triple: goal-satisfied (upper), quality-floor (existing), preemptive-unsolvable (new lower) |
| 6 | Per-claim citation verification pass (URL-liveness + arxiv-ID + CrossRef-DOI + SemanticCite faithfulness) | GhostCite arxiv:2602.06718, arxiv:2604.03173 | EXTRACT post-pass and final REPORT post-pass; arxiv-ID/DOI/URL checks are cheap and catch the bulk of fabrications |
| 7 | Sub-agent attribution lock: only fetching agent may cite the source | Anthropic multi-agent (simonwillison.net 2025/Jun/14) | All nodes: citation = (agent_id, fetched_url, fetched_at); lead may not invent IDs |
| 8 | Recency-weighted relevance for time-sensitive queries | arxiv:2509.19376 Solving Freshness in RAG, arxiv:2601.17824 OwlerLite | SourceQualityTable: time_decay multiplier when query has temporal markers ("latest", year tokens, "current") |
| 9 | Contradiction node in tree, never silently resolved | arxiv:2504.00180, arxiv:2511.06668 healthcare, arxiv:2510.03418 LegalWiz | REFLECT: when 2+ sources contradict, materialize CONTRADICTION node; renderer surfaces in report |
| 10 | Editable-CoT user steering | Gemini DR (gemini.google) | FORMULATE: optional `--interactive` flag exposes plan for user edit before dispatch |
| 11 | Cohere-style 4093-token chunking with max-pooled per-source relevance | docs.cohere.com/docs/reranking-best-practices | EXTRACT chunker; current BasicExtractor naive |
| 12 | Orchestrator-worker model split (cheap workers, capable orchestrator) | Anthropic 90.2% gain, tokens=80% variance (blog.bytebytego.com Anthropic) | PerRoleRuntime already supports this (B-37); wire to WebResearchOrchestrator |

## Anti-patterns
- "LLM resolves contradictions silently" - Perplexity DR observable behavior; gives confident wrong synthesis. ELAI's existing "never silently resolve" rule is correct and must be preserved. (arxiv:2504.00180 shows even SOTA LLMs are inconsistent at this)
- "Cite anything that looks like a source" - causes 14-95% hallucinated-citation rate (GhostCite arxiv:2602.06718). Cheap defenses (arxiv-ID/DOI/URL existence checks) catch the majority and must run pre-finalization.
- "Single search backend" - SearXNG-only inherits all upstream Bing/Google bias; major DR systems all fan out. (firecrawl.dev/blog/brave-search-api-alternatives)
- "Flat sub-question list, no tree" - FlashRAG and FlashResearch both show hierarchical decomposition beats flat. (arxiv:2510.05145, arxiv:2405.13576)
- "Stop when budget exhausted, no early-stop on unsolvable" - wastes tokens on dead branches. Conformal lower-threshold fixes this. (arxiv:2602.03814)
- "Trust vector similarity without rerank" - vector top-k != relevance top-k; cross-encoder rerank is standard. (docs.cohere.com/docs/reranking-best-practices)
- "Ignore source freshness" - "as-of" dates and dateModified are now structural ranking inputs in production AI-search. (ranktracker.com LLM-caching-recency, arxiv:2509.19376)
- "Trust DuckDuckGo as an independent source" - DDG results are Bing-derived; using DDG + Bing in a fan-out gives illusion of diversity. (factually.co private-search 2026)
- "Naive expansion of all branches" - prohibitively expensive; speculative pruning per FlashResearch is the documented fix. (arxiv:2510.05145, arxiv:2510.12164 parallel-reasoning survey)
- "Single-agent monolithic research loop" - Anthropic measured 90.2% deficit vs orchestrator-worker. (simonwillison.net 2025/Jun/14)

## Open questions
- Q1: Does ELAI's existing SourceQualityTable beat or lose to a learned reranker (Cohere v3.5 or a local cross-encoder distilled from it) on a DeepResearch-Bench-style eval? Measurable: citation-accuracy + comprehensiveness scores per arxiv:2506.11763.
- Q2: What is the actual measured speedup from adding speculative branch expansion to ELAI's Research Tree on a 100-query benchmark? FlashResearch claims wall-clock + token savings; needs reproduction. (arxiv:2510.05145)
- Q3: At what citation-verification budget (% of total tokens spent on per-claim URL/arxiv/DOI checks) does the marginal hallucination-rate reduction drop below 1pp? Required to size the EXTRACT post-pass. (arxiv:2604.03173 detection-tool accuracy curves)
- Q4: Does Brave-API+SearXNG+Mojeek deliver measurably higher source diversity (unique-domain count, tier-1 source count) than SearXNG-alone on a 50-query suite? Threshold: +>=20% unique T1 domains to justify the Brave spend.
- Q5: Conformal lower-threshold needs a calibration set of "unsolvable" research tasks; does ELAI have or can it construct one? Without it, the preemptive-stop gate's risk control is uncalibrated. (arxiv:2602.03814)

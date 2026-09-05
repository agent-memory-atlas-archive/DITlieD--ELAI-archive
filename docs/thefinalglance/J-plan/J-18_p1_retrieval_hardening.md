# J-18: P1 retrieval hardening — CodeRankEmbed 768d swap + LSP fourth channel + agentic-grep repair

- Track J (pillar-SOTA recheck extensions) | lever CANT-MISREAD | grounding STRONG (CodeRankEmbed dim VERIFIED via config.json; Serena/LocAgent/DCI repos VERIFIED) | nature BUILD+SWAP | effort M | WAVE 3 | depends none (extends E-17, rides shipped knowledge crate)

- Premise: the retrieval stack (tree-sitter symbol graph + tantivy BM25 + dual ONNX embeddings GraphCodeBERT-768d + Granite-384d + usearch HNSW + PageRank repo map + RRF k=60 + CPU cross-encoder reranker) is architecturally sound and ahead of most published work (verify-p1 overall verdict HARDEN-not-replace), but the code-embedding slot runs a GraphCodeBERT-era model leaving measurable CoIR points on the table, the graph is syntactic-not-semantic (no resolved refs across dynamic imports / macro expansion / trait objects), and there is no agentic repair when the static index returns stale/wrong edges. Three GROUNDED additions, each independently shippable.

## Steal/build
1. EMBEDDING SWAP (the high-confidence change): replace the GraphCodeBERT-era 768d code-embedding model with CodeRankEmbed INT8 ONNX. VERIFIED: nomic-ai/CodeRankEmbed config.json hidden_size=768 — EXACT match to the existing HNSW slot dimension, so this is a rebuild-only swap (no usearch index geometry change, only an index_version bump + re-embed). MIT, 137M, CoIR NDCG@10 = 60.1. EXPORT ONNX OURSELVES from the official card via optimum (anti-slopsquat: the community quant mrsladoje/CodeRankEmbed-onnx-int8 is 162 dl/mo, unofficial — do NOT depend on it; export + INT8-quantize from source). NOTE the query-prefix quirk (CodeRankEmbed requires a fixed query instruction prefix) + test the >2048-token path (max_trained_positions=2048 vs the claimed 8192 context — verify chunking behavior before trusting long-function embeddings).
2. LSP FOURTH CHANNEL: add a compiler-grade symbol-resolution channel alongside tree-sitter/BM25/vector for go-to-definition / find-references / cross-file type resolution. Serena (github.com/oraios/serena, 25.2k stars MIT, headless MCP confirmed, 40+ langs incl Rust) OR — lighter, preferred for the Rust orchestrator's own lane — drive rust-analyzer / pyright / typescript-language-server directly over LSP stdio from Rust (no MCP hop). Tree-sitter STAYS as the always-available incremental fallback (LSP init can fail; do NOT replace it). This is the channel that narrows J-16's blast-radius false-negatives (compiler-grade edges resolve what tree-sitter misses: macros, trait impls, dynamic dispatch).
3. AGENTIC-GREP REPAIR (the NARROW confirm leg): add a post-retrieval verification/repair pass — when static-index confidence is low, the agent issues grep/find to confirm-or-extend THIS index result (CORTEXA / DCI pattern, "index-first, grep-to-repair", NOT index-OR-grep). Additive, behind a feature flag, low risk. SCOPE BOUNDARY: this is the narrow post-hoc CONFIRM of a single low-confidence hit. The BROAD axis — exposing the whole hardened HYBRID cascade as a mid-task agent-callable tool the executor loops on (new queries discovered during reasoning) + SpIDER-style retrieval-time graph-neighbor fusion + a deterministic context-sufficiency gate — is J-38 (the talk's "retrieval is a mid-task loop" thesis, grounded by SWE-Explore/LocAgent/SpIDER). J-18 ships first; J-38 exposes J-18's hardened cascade, not the GraphCodeBERT one.
EXCLUDED: replacing tree-sitter (B1 syntactic limits are real but tree-sitter is the always-on fallback the report says KEEP); CodeXEmbed-7B / nomic-embed-code / voyage-code-3 (all >500M or cloud-only — rejected for CPU ONNX); jina-code-embeddings (896d dim mismatch + CC-BY-NC non-commercial — rejected); MCTS graph-traversal repo-map replacement (RANGER, no confirmed public release — too speculative; Louvain community priors are the safe PageRank harden if pursued, ocg crate Feb-2026 the only prod-ish Rust option, anti-slopsquat at edit).

## Existing ground truth (grep-confirmed; re-grep at edit, W-7/W-9)
- VECTOR SEARCH [WIRED] dual-model ONNX (GraphCodeBERT 768d code + Granite 384d text), usearch HNSW MetricKind::Cos, blake3 content-hash cache, index_version migration (feature map 1.1; crates/knowledge/src/vector/). The 768d code slot is the swap target.
- E-17 RAG retrieval quality [PARTIAL]: Contextual Retrieval + Jina-rerank + code_vec activation SHIPPED but ELAI_CAST_CHUNKER_ENABLE unallowlisted, eval harness not run, DEEP<STANDARD MRR fix unverified. J-18 is the next E-17 increment (the embedding model E-17 left in place).
- RETRIEVAL RESILIENCE [WIRED] SearchMode cascade {HYBRID,SPARSE_ONLY,BM25_ONLY} + per-model circuit breaker + index_version migration — the swap rides the existing migration path; missing model -> DEGRADED, never crash.
- DKG + B-29 impact API — the LSP channel feeds compiler-grade edges back into the graph J-16 reads.
- E-52 LSP AS VERIFIER [PARTIAL]: rust-analyzer headless pre-AUDITING diagnostic probe (ELAI_LSP_DIAGNOSTIC_PROBE_ENABLE unallowlisted) — the LSP subprocess plumbing this channel reuses (compose, do not re-author the subprocess harness).

## Integration
- crates/knowledge/src/vector/ (embedding model swap: new ONNX model load + query-prefix handling + index_version bump + re-embed path; ~80 LOC + the exported .onnx asset).
- crates/knowledge/ new lsp_channel module (~200 LOC: LSP stdio client OR Serena MCP client + go-to-def/find-refs queries folded into the retrieval cascade as a 4th ranker into RRF).
- crates/knowledge/ agentic-grep repair pass (~100 LOC, behind a flag: low-confidence -> grep confirm/extend).
- +RetrievalAugmentEvent SSB (Pattern-2 single-variant; Kind: EmbeddingModelSwapped / LspChannelQueried / AgenticGrepRepaired / CommunityPriorApplied). baseline+1 from LIVE 231/542 (grep 2026-06-12); re-read at edit (W-9). PREFER-REUSE: if QueryTrace / knowledge SSB rows already carry a channel-source field, extend them (net 0); decide at edit.
- cross_lang pair: none (Rust-internal knowledge crate); frontend_pair: retrieval-channel breakdown on the existing KnowledgeDrawer (B-29) or W-6-justified absence.

## WIRING_CONTRACT
produces_symbols:
- crates/knowledge/src/vector (CodeRankEmbed ONNX load + query-prefix + index_version migration, replacing the GraphCodeBERT 768d model)
- crates/knowledge/src/lsp_channel (the 4th retrieval channel: go-to-def / find-refs / type-resolution into RRF)
- crates/knowledge agentic-grep repair pass (low-confidence post-retrieval verify/extend)
consumes_symbols:
- usearch HNSW (the 768d slot, UNCHANGED geometry — exact-dim swap)
- E-52 rust-analyzer subprocess harness (the LSP plumbing; NOT re-authored)
- RRF k=60 fusion (the 4th channel folds in; NOT re-authored)
reachable_from:
- a real `elai` retrieval query (search/task path -> HYBRID cascade -> RRF)  [PRODUCTION]
- a #[cfg(test)] / contracts/ caller NEVER satisfies this (R7)
stub_ban:
- NO dependency on the unofficial community ONNX quant (export from nomic-ai/CodeRankEmbed source; anti-slopsquat)
- NO tree-sitter removal (it is the always-on fallback; LSP init can fail -> DEGRADED not crash)
- NO embedding swap without an index_version bump + re-embed (a silent dim/model change corrupts the HNSW index)
- NO new default-OFF ELAI_*_ENABLE without a wire-dark-allowlist row + promotion plan (R9/W-2)
- NO retrieval channel that returns top-k noise when it has nothing relevant (the B6 RRF-noise critique: a channel with no hit contributes nothing, not rank-k filler)
frontend_pair: retrieval-channel breakdown on KnowledgeDrawer (decide at edit) or W-6-justified absence.
observable_effect:
- trigger: "an `elai` retrieval query for a symbol reachable only via a macro-expanded / trait-object reference (tree-sitter blind), run through the real HYBRID cascade with the LSP channel on"
- expect_ssb: "EmbeddingModelSwapped on index rebuild (model=CodeRankEmbed, dim=768, index_version bumped); LspChannelQueried resolving the macro/trait edge tree-sitter missed; the resolved symbol appears in the RRF top-k"
- expect_artifact: "the query returns the correct cross-file symbol the old stack missed; an A/B on a CoIR-style fixture shows the CodeRankEmbed slot >= the GraphCodeBERT slot (ELAI-MEASURED, never the literature 60.1). NEGATIVE (R2): LSP init failure DEGRADES to tree-sitter+BM25+vector (cascade intact, not a crash); a swap with a dim mismatch FAILS the index_version migration test, never silently corrupts the index; agentic-grep off -> the index result is returned unrepaired (the flag is observable)."

## Grounding
- CodeRankEmbed: huggingface.co/nomic-ai/CodeRankEmbed (VERIFIED via verify-p1: config.json hidden_size=768 EXACT match, MIT, 137M, 8192-claimed/2048-trained ctx, CoIR 60.1, query-prefix required). INT8 community quant mrsladoje/...onnx-int8 exists (162 dl/mo) — NOT depended on; export from source.
- Serena: github.com/oraios/serena (VERIFIED: 25.2k stars, MIT, headless MCP, 40+ langs incl Rust). LocAgent arXiv:2503.09089 (VERIFIED, ACL 2025, github gersteinlab/LocAgent, 92.7% file-level — dataset ambiguous, the number is design rationale not a target).
- DCI/agentic-grep arXiv:2605.05242 (VERIFIED, May 2026, pro-grep). NOTE the report's "retrieval degrades agents" framing of arXiv:2603.20432 is REFUTED (verify-p1: actual title "Coding Agents are Effective Long-Context Processors", argues tool-use BEATS semantic retrieval by 17.3% — both papers point the SAME pro-direct-interaction way; the agentic-grep repair is consistent with both).
- codebase-memory-mcp arXiv:2603.27277 (VERIFIED, 3.1k stars, Louvain confirmed) — the Louvain-prior PageRank harden, if pursued (ocg Rust crate, anti-slopsquat at edit).
- E-17 / E-52 substrate (feature map 1.1 / 1.6, grep-confirmed). live SSB 231/542 (grep 2026-06-12; STALE — feature-map stamp ~268/577 at commit 5b0a66d 2026-06-15, re-grep at edit, W-9).
- HARDEN-not-replace REINFORCED (2026-06-16, arxiv.org/abs WebFetch VERIFIED): SweRank arXiv:2505.07849 ("outperforming both prior ranking models and costly agent-based systems using closed-source LLMs like Claude-3.5") + SpIDER arXiv:2512.16956 ("improves dense retrieval performance by at least 13%") = a non-agentic hybrid retrieve+rerank pipeline BEATS pure agentic grep on localization AND cost, and graph-augmented retrieval beats the iterative LLM graph agent — so the embedding-quality swap (this file) is the right lever and the pipeline stays; the mid-task LOOP (J-38) is additive on top, never a replacement. SWE-Explore arXiv:2606.07297 ("agentic explorers form a clear tier above classical retrieval") is the gap J-38 closes, not this file's. Full synthesis: workflow/reasoning/retrieval-agentic-loop-grounding-2026-06-16.md.

## COMPOSES-WITH
E-17 (the next retrieval-quality increment; consumes its Contextual-Retrieval + code_vec work), E-52 LSP-as-verifier (shares the rust-analyzer subprocess harness), B-29 / J-16 (the LSP channel's compiler-grade edges narrow the blast-radius cone's false-negatives), RRF / SearchMode cascade (the 4th channel folds in), J-38 (the mid-task agentic retrieval LOOP that exposes THIS file's hardened cascade as a callable tool + SpIDER graph-neighbor fusion + sufficiency gating; J-18 is the substrate it loops on, ships first; this file's leg-3 grep-repair is the narrow confirm vs J-38's broad query loop). Restates none.

## Open questions / risks
- CodeRankEmbed context: the 8192-claimed vs 2048-trained-positions gap means long-function embeddings beyond 2048 tokens may degrade — the swap's first checkbox is a chunking A/B (does the 50-line window + the new model beat the old on long functions, or does it need a smaller window).
- LSP latency: a headless LSP server per language adds startup + per-query latency; the channel must be SELECTIVE (only invoked for symbol-resolution queries where tree-sitter confidence is low, not every query) — the B7 reranker-latency critique applies equally here.
- query-prefix discipline: CodeRankEmbed's required instruction prefix is part of the byte-stable input — get it wrong and the embeddings are silently worse; assert the prefix in a contract test.

## Next
The LSP channel's compiler-grade edges feed J-16's blast-radius cone (fewer false-negatives on macro/trait/dynamic-dispatch). The embedding swap is the cleanest single win (exact-dim, rebuild-only) and the natural first checkbox. AFTER this hardening lands, J-38 exposes the hardened cascade as a mid-task agent-callable loop (the talk's actual thesis) — sequence J-18 -> J-38.

End of J-18.

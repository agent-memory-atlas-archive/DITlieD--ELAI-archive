# Infinite-context chat illusion on a local 7B

## TL;DR
- Replace MistakeBook flat list with an A-MEM-style Zettelkasten layer over SSB: every mistake/learning becomes a structured note with tags+context+links, and new notes evolve attributes of existing notes — keeps the ~150 tok injection budget but compounds across sessions instead of staying flat.
- Add a MemGPT/Letta-style hierarchical core/archival split inside the 25K context: a small (~1-2K tok) self-editable Core block for persistent user model (style, expertise, opinions, current goal) plus tool-calls to page archival in/out. SSB already supplies the archival store; the missing piece is the model-driven `core_memory_replace` / `archival_insert` tool surface.
- Ship sleep-time compute between turns: while user types, run Qwen-7B offline on the freshest SSB delta to (a) refresh a "user model card" (200-400 tok), (b) pre-compute likely-next retrieval queries, (c) regenerate the rolling summary. Lin et al. 2025 measured ~5x test-time compute reduction and -50% tokens on real repo tasks.
- Layer Zep/Graphiti-style temporal knowledge graph on top of DKG: bi-temporal edges (valid_at / invalid_at) so opinions and preferences can be revised without contradicting earlier memories. Zep beat MemGPT on DMR 94.8 vs 93.4 and posted +18.5% on LongMemEval with -90% latency vs full-context baselines.
- Treat first-token latency as a feature, not a constraint: speculative decoding (Qwen-0.5B drafter -> 7B verifier) + streamed "I remember when you said X..." preamble built deterministically from SSB before the model wakes up. The 320ms feel of flagship chat is mostly TTFT theater, not raw model IQ.

## Key papers / sources

memory architectures
- arxiv:2310.08560 | MemGPT: Towards LLMs as Operating Systems | OS-style two-tier memory (Core RAM + Archival/Recall disk) with self-editing via tool calls; basis for Letta. [letta blog](https://www.letta.com/blog/letta-v1-agent)
- arxiv:2502.12110 | A-MEM: Agentic Memory for LLM Agents (NeurIPS 2025) | Zettelkasten-inspired notes with auto-linking + memory evolution; outperforms flat-buffer baselines on six foundation models. local: `.claude/papers/2502.12110.pdf`
- arxiv:2501.13956 | Zep: Temporal Knowledge Graph Architecture for Agent Memory | Graphiti engine, bi-temporal edges, +18.5% LongMemEval, -90% latency, 94.8 DMR vs MemGPT 93.4. local: `.claude/papers/2501.13956.pdf`
- arxiv:2507.07957 | MIRIX: Multi-Agent Memory System | 6 memory types (Core/Episodic/Semantic/Procedural/Resource/Vault) + 8 specialist agents; 85.4 LoCoMo SOTA, 99.9% storage reduction on ScreenshotVQA.
- arxiv:2502.06975 | Episodic Memory is the Missing Piece for Long-Term LLM Agents | position paper: consolidation episodic->semantic is the least-implemented but most important step.
- arxiv:2305.10250 | MemoryBank | Ebbinghaus forgetting curve + daily personality distillation -> persistent user portrait. (cited via [serokell](https://serokell.io/blog/design-patterns-for-long-term-memory-in-llm-powered-architectures))

retrieval
- arxiv:2405.14831 | HippoRAG: Neurobiologically Inspired Long-Term Memory | KG + Personalized PageRank single-step retrieval, +20% on multi-hop, 6-13x faster than iterative RAG. local: `.claude/papers/2405.14831.pdf`
- https://www.anthropic.com/news/contextual-retrieval | Contextual Retrieval (Anthropic, Sep 2024) | per-chunk LLM-generated context prefix + BM25 + reranker: -35% to -67% retrieval failure rate.
- arxiv:2410.10813 | LongMemEval (ICLR 2025) | 500 questions over ~115K-token chat histories, 5 abilities (extraction/multi-session/temporal/knowledge-update/abstention). local: `.claude/papers/2410.10813.pdf`
- arxiv:2402.17753 | LoCoMo: Evaluating Very Long-Term Conversational Memory | up to 32 sessions / ~600 turns / multimodal; primary testbed for Mem0/Zep/MIRIX comparisons.
- arxiv:2404.16130 | Microsoft GraphRAG (From Local to Global) | entity extraction + Leiden communities + hierarchical summaries; ~70-80% win rate on comprehensiveness/diversity vs naive RAG.

sleep-time compute
- arxiv:2504.13171 | Sleep-time Compute: Beyond Inference Scaling at Test-time | offline reasoning over stored context cuts test-time compute ~5x, +13-18% accuracy, -50% tokens on agentic SE task. local: `.claude/papers/2504.13171.pdf`

user modeling
- arxiv:2501.11549 | Whose Boat Does it Float? Personalization via Inferred User Personas | abductive persona inference + persona-tailored training improves personalization beyond crude demographic prompting.
- arxiv:2504.14225 | Know Me, Respond to Me | benchmark for dynamic user-profile inference across multi-session chats where preferences evolve.
- arxiv:2504.19413 | Mem0: Building Production-Ready Agents with Scalable Long-Term Memory | 94.8 LongMemEval (+27pt), 91.6 LoCoMo (+20pt) using extracted-fact memory layer.

anti-patterns / latency
- arxiv:2505.23422 | From Knowledge to Noise: CTIM-Rover (REALM 2025) | adding episodic NL memory to SWE agent *degraded* AutoCodeRover in all configs; noisy CTIM items poisoned exploration.
- https://bentoml.com/llm/inference-optimization/speculative-decoding | Speculative decoding handbook | drafter/verifier 2-3x speedup, lossless quality; production-standard in vLLM/TensorRT-LLM as of late 2025.
- [Nielsen Norman / UX threshold work, surfaced via redis blog](https://redis.io/blog/streaming-llm-responses/) | <0.1s instantaneous, <1s flow-preserving, ~10s attention limit — TTFT dominates perceived speed.

## State of the art (2024-2026)

Two clusters dominate. Cluster A is **structured memory + retrieval**: Mem0, Zep/Graphiti, A-MEM, MIRIX, HippoRAG. They all extract facts/triples/notes from raw chat, store them in a queryable structure (vector + KG + bi-temporal index), then inject only the top-K relevant items at query time. On LongMemEval the bar is ~94-95% (Mem0 reports 94.8, Zep close behind) versus ~60-70% for naive full-context-window approaches and ~80% for vanilla RAG. On LoCoMo (32 sessions, ~16K tok dialogues), MIRIX holds SOTA at 85.4 using a 6-type memory + 8-agent architecture. Cluster B is **hierarchical / OS-style memory**: MemGPT/Letta keeps the model in the loop — it self-edits a small core block via tool calls and pages archival in/out, mirroring virtual memory paging. Letta's v1 agent loop merges this with ReAct-style tool use and Claude-Code-style "filesystem-as-memory" so the model treats a directory of notes as its addressable archive.

Convergent findings: (1) **flat append-only logs lose** — every winning system imposes structure (graph, communities, atomic notes with links). (2) **Temporal validity matters** — Zep's bi-temporal edges and MemoryBank's Ebbinghaus decay handle the "user changed their mind" failure mode that hurts naive systems. (3) **Consolidation episodic->semantic is underdone** — the Feb 2025 position paper at arxiv:2502.06975 explicitly calls this out, and HippoRAG's PageRank-over-KG and A-MEM's note-linking are the current best-effort answers. (4) **Sleep-time compute is free latency** — Lin et al. 2025 showed pre-computing summaries/queries offline cuts test-time tokens by ~50% with equal accuracy. (5) **Contextual retrieval beats raw chunks** — Anthropic's per-chunk-context preprocessing cuts retrieval failures up to 67% combined with BM25+reranker, and it works equally with a local cross-encoder reranker. (6) **The "flagship feel" is half-latency-theater**: 320ms TTFT, persistent style, and continuity-of-opinion (ChatGPT memory feature: User Profile + Extracted Knowledge + Active Context) account for a large share of perceived intelligence beyond raw param count.

Counter-evidence: CTIM-Rover (REALM 2025) demonstrated that naive episodic memory *degrades* small-model agents — the noise from irrelevant past trajectories overwhelms signal. Embedding-based filtering is the proposed fix, which aligns with the structured-memory cluster's bias toward retrieval-over-injection.

## Best techniques to adopt for ELAI

1. **A-MEM-style note layer in SSB** | arxiv:2502.12110 | ELAI's MistakeBook is a flat structured-forbidden-diff list with a 150-tok injection cap — exactly the failure mode A-MEM was designed to fix. Replace flat list with Zettelkasten notes: each mistake/lesson stores `{context, keywords, tags, links[]}`, new notes trigger updates to linked notes (memory evolution). Stays within token budget but compounds. | **Integration hook**: new SSB `action_type=memory_note` with note schema; MistakeBook becomes a view over notes filtered by tag=mistake; injection picks top-K by Personalized PageRank seeded on current task tags, not by recency.

2. **MemGPT/Letta core+archival inside the 25K context** | arxiv:2310.08560, Letta v1 blog | ELAI's "personality resets per call" pain is exactly the MemGPT-target problem. Carve the 25K into: ~1.5K Core (user model card + active goal + style guide, model-editable via tool calls), ~2K Working (current task + last 3 turns), ~18K Retrieved (DKG+BM25 hits), ~3K Generation buffer. Core block survives across calls; archival lives in SSB. | **Integration hook**: two new tools surfaced to the model — `core_memory_replace(section, new_text)` and `archival_search(query)`. Both backed by SSB. FSM enforces Core stays under hard cap.

3. **Sleep-time compute between turns** | arxiv:2504.13171 | Local 7B is idle while user types. Lin et al. measured 5x test-time reduction and 50% token savings on agentic SE tasks. ELAI's deterministic FSM is ideal for triggering this — same chokepoint as `Indexing`/`Auditing`. | **Integration hook**: new FSM state `SLEEP_COMPUTE` between turns; consumes recent SSB delta + current task SessionDag snapshot; outputs (a) refreshed user-model-card written to Core block, (b) pre-computed retrieval-query list cached for next user turn, (c) updated rolling summary. Caps: time-budget bounded, falls through on user-input event.

4. **Zep-style bi-temporal edges in DKG** | arxiv:2501.13956 | DKG currently has no notion of "opinion was true then but isn't now." Adding `valid_at` and `invalid_at` per edge lets ELAI revise without contradiction and answer "what did I used to think about X" queries. | **Integration hook**: schema migration on DKG; PageRank weights decayed by `(now - valid_at)` and zeroed past `invalid_at`; reranker sees temporal score as a feature. Pairs naturally with the 7-layer knowledge engine's existing PageRank step.

5. **Contextual retrieval preprocessing for SSB / SessionDag** | Anthropic Sep 2024 | Every chunk indexed in BM25 + dual-vector should ship with a 50-100 tok LLM-generated context prefix ("This snippet is from session 17 in which the user was debugging the sidecar OAuth refresh"). Measured -49% retrieval failure with BM25+contextual embeddings, -67% with reranker. | **Integration hook**: paragraph-grafting step at index time inside the `Indexing` FSM state; reuses the same Qwen-7B during sleep-time-compute so it's free. Cross-encoder rerank already in pipeline (layer 7), keep it.

6. **HippoRAG-style Personalized PageRank over A-MEM notes** | arxiv:2405.14831 | Single-step PageRank retrieval is 6-13x cheaper than iterative IRCoT and +20% on multi-hop. ELAI already has PageRank in the 7-layer engine; just point it at the A-MEM note graph. | **Integration hook**: A-MEM note links become PPR edges; query entities become PPR seeds; retrieval is a single PPR call, no iterative LLM-in-the-loop.

7. **TTFT theater via deterministic preamble** | redis/NN UX research | Before model wakes up, stream a deterministic "Continuing from where we left off: you were working on X, the last blocker was Y..." preamble built from SSB. First token lands in <50ms; model attaches mid-stream. This is how flagship chat *feels* fast even when it isn't. | **Integration hook**: new `chat_preamble` SSB action type; FSM emits preamble token stream while warming the KV cache; preamble text also acts as soft conditioning on the model's continuation.

8. **Speculative decoding with Qwen-0.5B drafter** | bentoml handbook | Qwen2.5-Coder family has a 0.5B sibling — same tokenizer, ideal drafter. Lossless 2-3x decode speedup, especially valuable for the 25K-context regime where TTFT is already painful. | **Integration hook**: sidecar `--speculative` flag; reuses existing model loading path; gated by VRAM headroom (drafter ~0.4 GB at NF4).

## Anti-patterns to avoid

- **Flat append-only chat log injection.** CTIM-Rover (arxiv:2505.23422) showed adding episodic NL memory to a SWE agent degraded AutoCodeRover in *every* configuration; noisy items poisoned exploration. ELAI's MistakeBook avoids this by capping at 150 tok, but expanding it without structure would reproduce the failure. Use A-MEM-style filtering + tag-scoped retrieval, never bulk-inject.
- **Storing raw transcripts as retrieval targets.** LongMemEval (arxiv:2410.10813) and LoCoMo (arxiv:2402.17753) both show extracted-fact memory (Mem0 94.8 LongMemEval) beats raw-chunk RAG by 20+ points. Index extracted statements, not raw turns.
- **Letting the model freely edit Core memory without a schema.** MemGPT lets the model rewrite core memory via tool calls; on a 7B this drifts within a few cycles. Constrain to typed sections (`user.style`, `user.expertise`, `task.current`) with hard length caps, FSM-validated.
- **Multi-agent memory orchestration on a single 7B.** MIRIX uses 8 specialist agents — each is a separate LLM call. On Qwen-7B at ~25K context, this is multiple seconds per memory op. Use a single agent with typed sub-tools instead.
- **Naive long-context expansion.** Bumping ELAI's 25K to 100K via YaRN/RoPE scaling on Qwen2.5-Coder-7B degrades reasoning quality and inflates KV cache past VRAM ceiling. Keep 25K and put smarter memory in front of it. The Mem0 cost-performance paper (arxiv:2506.04814 ref'd in search) found fact-based memory consistently dominates long-context on cost AND accuracy for chat workloads.
- **Personality from system-prompt strings alone.** Persona-prompting based on demographics underperforms inferred-persona training (arxiv:2501.11549). Don't try to "lock personality" with a long static system prompt — derive a compact user-model-card from observed behavior and rewrite the Core block via sleep-time compute.
- **Skipping decay/invalidation.** Without bi-temporal edges (Zep) or Ebbinghaus decay (MemoryBank), the system will cheerfully assert opinions the user retracted six sessions ago. This is one of the loudest "feels dumb" signals.

## Open questions / what needs measurement

- **A-MEM note evolution stability under Qwen-7B.** Does the note-linking + attribute-evolution loop converge or thrash when the drafter is a small abliterated model? Measure: edit-distance of N most-touched notes over 100 simulated turns; threshold = <10% churn after turn 50.
- **Sleep-time compute payoff curve for ELAI's FSM cadence.** Lin et al. measured on agentic SE; ELAI's chat cadence (longer idle gaps, smaller deltas) may shift the optimum. Measure: TTFT and tokens-per-turn vs. sleep-compute budget {0, 250, 1000, 4000} tokens of offline work per idle interval.
- **Cross-encoder rerank delta when DKG entries carry contextual-retrieval prefixes vs. raw.** Anthropic reports -67% retrieval failure end-to-end but their corpus is documents, not chat. Measure on LongMemEval-style synthetic ELAI sessions: recall@5 with prefix vs. without, holding the existing reranker fixed.
- **Speculative decoding acceptance rate with Qwen2.5-Coder-0.5B drafter -> 7B target on code+chat mix.** Production gains hinge on >60% draft-acceptance. Measure on ELAI's actual chat+codegen traffic mix, not standard benchmarks.
- **Core-block drift on local 7B over multi-day usage.** MemGPT was tested on GPT-4 class models. Does a 7B reliably maintain the Core block, or does it gradually corrupt user-model fields? Measure: structural validity of Core block (JSON-schema check) over 7 days of natural use; threshold = >99% valid commits before FSM revalidation kicks in.

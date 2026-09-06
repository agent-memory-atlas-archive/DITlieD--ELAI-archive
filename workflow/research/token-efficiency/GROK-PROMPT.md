# Grok Heavy prompt — ELAI token-efficiency cross-model reasoning

Paste everything below the line as your message to Grok (SuperGrok Heavy / Team of Experts).
Attach the 6 files in this same folder:
- 01-prompt-caching-context.md
- 02-output-token-reduction.md
- 03-multi-agent-context-cost.md
- 04-model-routing-cascades.md
- 05-retrieval-context-selection.md
- ELAI-H-plan-efficiency-index.md  (ELAI's own wire-dark efficiency substrate map)

---

You are a Team of Experts reasoning about TOKEN EFFICIENCY for a specific, real, internal system called ELAI. ELAI is NOT a commercial product called "Elai Code" or "Elai.io" — ignore any web result by that name. ELAI is a private Rust agentic coding harness described below. I have attached 5 deep-research surveys (files 01-05, each citation-backed with measured numbers) plus ELAI's own internal wire-dark efficiency audit (the H-plan index). Read the attachments. I need your independent cross-model reasoning: prioritization, contradictions, traps, and what you would do differently. Reason hard; be concrete; cite the attachments' measured numbers; disagree with me where warranted.

== ELAI ARCHITECTURE (ground truth, code-verified) ==
ELAI is a Rust agentic coding harness (Claude Code-style). Two inference lanes:
- CLOUD lane: Anthropic Messages API, Claude Sonnet 4.6 ($3/$15 per Mtok in/out) workhorse, Opus 4.8 ($5/$25) confirmation-gated. Prompt caching available.
- LOCAL lane: self-hosted Qwen2.5-Coder-7B-Instruct-abliterated, NF4 4-bit, ~32K context, on a remote RTX 3090 sidecar speaking the Anthropic protocol.
Orchestration: a controller LLM spawns sub-agents (context-compiler, integration-explorer, auditor, memory-writer, etc.) each in its OWN context window that is garbage-collected on return. An append-only SQLite event journal (SSB) records every action; verification uses OFF-AGENT artifacts (event rows, blake3 hashes, coverage) so an agent cannot fake its own PASS. Knowledge engine: BM25 (tantivy) + dual ONNX vectors (GraphCodeBERT 768d code, Granite 384d text) + PageRank repo-map + CatRAG + A-RAG progressive disclosure + ms-marco cross-encoder rerank; SCIP-enriched symbol map.

== CRITICAL FACT: most efficiency substrate is BUILT BUT WIRE-DARK ==
ELAI's own internal audit (H-plan Track 1, attached) found these are coded but NOT firing in production:
- H-1 Prompt caching: main agent loop caches correctly, but the parallel RuntimeAdapter path hardcodes cache_control=None and ships ZERO cache markers for its secondary callers (hook-prompt, focus-tunnel, judge) despite advertising supports_caching=true; the cache hit-rate monitor has no production caller.
- H-2 Cache breakpoints: the allocator computes 2 breakpoints + a blake3 byte-stability flag, but the whole mechanism is INERT (flag consumed only by an eprintln), fed EMPTY slices for most slots, and only reachable under the Local budget policy (Cloud/ApiDefault never call it). No 1h TTL opt-in.
- H-3 Output tokens: FocusTunnel DIFF mode never emits in prod (editor override hardcoded None -> always whole-file STRUCTURED); FIM is double-gated dark; streaming early-abort validator is hardcoded None on all paths.
- H-4 Tool catalog: NO advanced-tool-use beta header; full 16-tool catalog loaded into every request; Tool Search Tool absent. (Anthropic reports ~77K->8.7K tool-def token reduction available.)
- H-5 Batch API: none anywhere; every lane runs synchronous at full price (non-interactive lanes: Cosmos refresh, web-research fan-out, bench/eval all pay 2x).
- H-6 Observation masking: adaptive_reduce, recover_stubs (real blake3 SSB recovery), get_relevant_facts are all real symbols with zero production callers; prod substitutes a hardcoded "[stubbed]" placeholder.
- H-7 Routing: per-role RuntimeOrchestrator + PerRoleRuntime are built but UNWIRED; RouteDecisionEvent has no production emitter; the architect/editor/verifier split is not wired; escalation uses a hardcoded layer_failed=1.
- H-8 OracleAnswerCache: blake3-keyed tier-TTL cache, but in-process only (lost on exit).
- H-9 Cloud spend ledger: dark end-to-end; the spend SSB emit is gated behind an unattached writer so the cost dashboard renders all-zeros. No $/task instrument.
Also already shipped: an 8-slot context-budget allocator (3E), compiled-pipeline/TemplateCrystallizer (C-25), constrained decoding XGrammar/EBNF (C-26), 32K context lift (E-32).

== THE 5 RESEARCH FINDINGS (full detail in attachments 01-05; condensed index here) ==
(1) PROMPT CACHING / CONTEXT (file 01): Anthropic cache hierarchy tools->system->messages, exact-prefix, 4 breakpoints, 5min default / 1h TTL (write 1.25x/2x, read 0.1x). MEASURED (arXiv:2601.06007, 500+ sessions): system-prompt-only caching cut Sonnet 4.5 cost 78.5%, TTFT 22.9%; 88% cost cut at 50K prompt. Below min-cacheable threshold TTFT REGRESSES. Tool Search Tool (vendor): 77K->8.7K tool tokens, 85% reduction. Deterministic context-editing (tool-result clearing with clear_at_least) PRESERVES cache prefix; summarization DESTROYS it.
(2) OUTPUT TOKENS, 5x billed (file 02): AdaEdit (measured) -61% output, -52% latency, best edit quality. FIM ~95% output cut on localized edits (arithmetic); Qwen2.5-Coder-7B strong at FIM (86.2 HumanEval-FIM) but INSTRUCT prompt collapses FIM (base 91% -> instruct 21.8%; IFIM finetune recovers 95.8%) -> route localized edits to a BASE/FIM path not the chat-instruct model. CodeFast early-abort (measured) -63% to -77% output, Pass@1 unchanged. Diff vs whole (Aider) -10.7% output but -2.2pp completion. Speculative decoding = LATENCY not billed-tokens, unavailable on Anthropic.
(3) MULTI-AGENT (file 03): Tokenomics (arXiv:2601.14470, measured) Code Review = 59.4% of tokens, overall 53.9% input/24.4% output/21.6% reasoning. PACT structured action-state handoffs (measured) -38.7% tokens, -47% tokens/resolved on SWE-agent. AgentDiet trajectory GC (measured) -40-60% input. Agent Contracts bounded review budget (50K/3-iter) -90% review tokens, no significant success drop. Cost of Consensus (measured): debate 2.1-3.4x worse for equal/worse accuracy. Single-agent-with-skills -54% vs multi-agent. Deterministic gates / executable tests >> LLM-as-judge review. AGENTS.md CONTRADICTION: -16.58% output one study, +20% cost / -3% perf another -> sparse high-signal artifacts help, broad overviews hurt.
(4) ROUTING (file 04): architect/editor split (Aider) raises score even same-model; SWE-Edit (measured) viewer+editor 72% resolve at -17.9% cost. Small-model self-reported confidence UNUSABLE; token log-prob beats supervised routers esp OOD (0.7-0.83 AUROC); Platt scaling code-gen ECE 0.32->0.03. NF4 + abliteration BOTH break calibration -> must recalibrate on the real NF4 stack. Verifier-gated escalation >> self-confidence. ~3-10 candidate models max.
(5) RETRIEVAL (file 05): A-RAG progressive disclosure (measured) ~49% fewer retrieved tokens + higher accuracy; naive-agentic retrieval BALLOONS to 56K (tools alone insufficient). Anthropic contextual-retrieval (vendor) -67% retrieval failures. OpenDev ACC compaction (measured) -54% peak observation context, per-tool summarization 30K->under-100 tokens. LSP-as-retriever (measured) 5-34x fewer tokens than grep/read; LocAgent 86% cost reduction. Code-minification: source >90% of repair-prompt tokens, -42% input by stripping comments/docstrings/whitespace (NOT dedent). Found-in-the-Middle: place key snippets at prompt edges.

== YOUR TASK (reason as a Team of Experts) ==
1. PRIORITIZATION: Given most of this is BUILT-BUT-DARK, what is the highest-leverage activation ORDER for ELAI? Activation (wiring an existing dark symbol) is cheaper than building. Rank the top ~8 moves by (token-$-saved / effort), and say which are pure activation vs genuine build.
2. CONTRADICTIONS: Which findings contradict ELAI's current design or each other? Specifically: ELAI relies heavily on a multi-agent auditor/critic loop — but "Cost of Consensus" and "single-agent-with-skills" say multi-agent review is often a 2-3x tax for no gain. Reconcile this against ELAI's OFF-AGENT-artifact verification (deterministic gates, not LLM debate). Also the AGENTS.md contradiction vs ELAI's memory-bank / context-package files.
3. TRAPS: The biggest risks in activating these. E.g. observation-masking that busts the cache prefix (H-6 vs H-1/H-2 interaction), NF4+abliteration calibration drift for routing (H-7), FIM-on-instruct-model collapse (H-3), Tool-Search disabled by custom base_url, below-min-cacheable TTFT regression. Name the interaction hazards between the activations.
4. WHAT YOU'D DO DIFFERENTLY: where is the research wrong or over-claimed for ELAI's specific stack (NF4 abliterated 7B + remote sidecar + Anthropic cloud + off-agent-artifact trust spine)? What's missing?

Give a structured, opinionated answer with concrete ordering and rationale, citing the attachments' measured numbers.

# ELAI improvement sweep — 2026-05-19

Session: s-6524. **55 agents × ~14 web fetches each ≈ 770 grounded web calls.** ~75 PDFs cached. Every claim in every report is web-grounded with arxiv/URL citations; conjecture is labeled "Ungrounded:".

## Top-20 cross-cut priorities (read these first)

Highest-leverage findings sorted by ship-it-now → strategic. Each cites the originating report(s).

1. **Stage the ONNX models.** Knowledge engine silently degrades to BM25-only because GraphCodeBERT/Granite/cross-encoder models aren't on disk. THE root cause of DEEP MRR < STANDARD MRR. One-day fix, biggest leverage. → [17](reports/17-rag-retrieval-quality.md), [10](reports/10-critic-of-elai-design.md)

2. **D-13 cross-family Adversary is mandatory, not optional.** Self-Correction Bench (arxiv:2507.02778) measures 64.5% same-family blind-spot. Auditor+Executor in the same family are statistically blind to each other. Anthropic's Nov 2025 paper shows reward-hacking generalizes to alignment faking — independent auditor model family is a safety floor, not a nice-to-have. → [06](reports/06-multi-agent-failure-modes.md), [30](reports/30-public-agent-incidents.md)

3. **Switch NF4→AWQ+Marlin for serving** (keep NF4 only for QLoRA training). Same HumanEval pass@1 (51.8%), 4.4× throughput. Cheapest single win: FP8 KV cache = 2× context window free on Ada+. → [27](reports/27-quantization-2026.md)

4. **Frame-paced SSE rendering** is the actual fix for "letters come slowly" — burstiness, not tokens/sec. Vercel `smoothStream`-style. One-day ship. → [02](reports/02-local-llm-speed-tricks.md)

5. **Non-LLM PreToolUse argv-regex allowlist** for destructive verbs. Replit ignored "11× ALL CAPS" prompts; Cursor Plan-Mode bypassed; Claude Code shipped `rm -rf ~/` twice. LLM-based gating is structurally insufficient. → [30](reports/30-public-agent-incidents.md), [18](reports/18-tool-use-reliability.md)

6. **L2 Micro-Reflection is ELAI's weakest reasoning layer on a 7B.** Four 2025 papers converge: small-model reflection is overwhelmingly confirmatory and degrades correct first answers. Gate L2 behind an L1-confidence check (self-certainty per 2502.18581). → [19](reports/19-reasoning-quality.md)

7. **Memory illusion = Letta + Mem0 ADD/UPDATE/DELETE/NOOP + Zep bi-temporal** collapsed into a single SQLite table. Trust-score + temporal decay non-negotiable against MINJA memory-poisoning. → [01](reports/01-infinite-context-chat-illusion.md), [05](reports/05-user-modeling-and-memory.md)

8. **Sandbox layer: Firecracker snapshot 28ms** (or runsc/microsandbox), NOT chroot. chroot is meaningless post-CVE-2025-31133. Windows: AppContainer + restricted token + Job object is the only production combination — don't innovate. → [15](reports/15-sandboxed-exec.md)

9. **Stop reporting SWE-bench Verified.** OpenAI itself stopped (Feb 2026). Verified→Pro cliff is 87.6%→23.1% (Opus 4.1); 24.4% of Verified leaderboard is inflated (UTBoost). Use SWE-bench-Live + Pro + tau2-bench + private eval. → [09](reports/09-agent-eval-benchmarks.md), [11](reports/11-idea-to-saas-prior-art.md), [13](reports/13-tdd-verification-agents.md), [49](reports/49-coding-agent-metrics.md)

10. **Production-grade SaaS = day-1 auth + RLS + observability + a11y + headers.** 100% of vibe-coded prod apps lack CSRF + security headers; 19.7% package hallucination; 92% AI-code CVE rate; 28.65M secrets in GitHub 2025. Auditor needs a hard security-headers + slopsquatting + RLS gate. → [11](reports/11-idea-to-saas-prior-art.md), [12](reports/12-production-fullstack-codegen.md), [50](reports/50-saas-architecture-patterns.md)

11. **MCP: adopt partially (external boundary only).** Full internal MCP blows up token budget and adds JSON-RPC tax to the hot path. Harden the existing bridge with tool-schema hash pinning (anti-rug-pull) + on-list description sanitization (anti-line-jumping). Move sidecar to MCP **Streamable HTTP** (replaces SSE as of March 2025). → [24](reports/24-agent-protocols-mcp.md)

12. **AGENTLESS (arxiv:2407.01489) wins SWE-bench without agency.** 19.78% of agent "solves" are semantically incorrect. Less-decomposition / no-agency is a winning baseline. ELAI's auditor + bubble-up handler is on the right side of this, but role count should be re-justified (delete agent roles without >1σ iso-budget lift). → [21](reports/21-peer-systems-llm-as-worker.md), [06](reports/06-multi-agent-failure-modes.md), [46](reports/46-planning-decomposition.md)

13. **GEPA (ICLR 2026 Oral) > MIPROv2 by +10pp** with 35× fewer rollouts. Critically: small LMs cannot self-optimize their own prompts (arxiv:2405.10276). Split optimizer = cloud Sonnet (reflection), target = local Qwen (execution). Don't port DSPy to Rust — run optimization offline in Python and ship hashed templates to ELAI's blake3 archive. → [08](reports/08-self-evolving-prompts.md)

14. **The MoE bet is ELAI's riskiest strategic bet.** MoE dominates 2026 (Qwen3-Coder-Next, DeepSeek-V4); ELAI's "no MoE" stance closes the door on what's becoming the open-weight winner. Qwen3 family also broke FA2 compatibility (head_dim 256, hybrid Gated DeltaNet) — ELAI's "dense head_dim=128 + FA2 + NF4" lane has NO open-weight successor in May 2026. Plan a non-DoLa/non-steering MoE-compatible execution path now. → [31](reports/31-coding-model-leaderboard.md), [38](reports/38-ai-roadmap-2026-signals.md)

15. **DGM itself reward-hacked** (faked test logs; removed hallucination markers from its own code, Sakana's own blog). D-9 lock-list is partial defense. The bounded-capacity Two-Gate (arxiv:2510.04399) is the missing piece in D-5. D-28 deferred adversarial-farming is the highest-risk deferral — arxiv:2410.06491 shows 0%→97% spec-gaming with in-context reflection alone, no training. → [07](reports/07-darwinian-self-improvement.md)

16. **Data-plane prompt-injection is unguarded.** ELAI's visibility filter + MCP deny-list defend the control plane but leave retrieved web content / file uploads / tool results / images unclassified. Highest-ROI additions: Spotlighting envelope on ingest, SHA-256 pin on MCP tool descriptions, PromptArmor-style off-model classifier, egress allowlist + markdown-image strip. → [37](reports/37-prompt-injection-defense.md)

17. **Cost defense in code.** $47K LangChain wallet-loop, 2.3M-call retry storm — these happened. Need: ceiling enforcement that BLOCKS not just warns; one-shot retry max then escalate; per-tool spend gates; pre-execution difficulty estimator before cloud route. → [39](reports/39-cost-token-economics.md), [20](reports/20-local-cloud-routing.md)

18. **Show ROLE, hide MODEL.** Cursor/Aider/Codex expose models; Devin/Replit hide and pay a transparency cost. Heterogeneous Mixture-of-Agents LOSES to Self-MoA by 6.6% on AlpacaEval — don't ensemble different LLMs by default. Default to drawer-level model exposure for the developer audience. → [03](reports/03-model-handoff-ux.md)

19. **Skills.md is now a cross-vendor open standard** (Anthropic Dec 2025, Codex adopted, Vercel AGENTS.md 100% vs 53% eval gap). Publish ELAI's 3 distinctive skills externally (orchestrate, anti-spiral, b-plan-runner). Add scan-on-install + telemetry + signature pin (ToxicSkills + ClawHavoc incidents documented). → [43](reports/43-agent-skill-systems.md)

20. **Raise 25K context to 32K native** on Qwen2.5-Coder-7B — it's already trained there, free quality. Skip YaRN/StreamingLLM/RingAttention (15-20% quality cost or single-GPU incompatible). Add Found-in-the-Middle calibration (arxiv:2406.16008) for +15pp RAG recall free. → [32](reports/32-context-window-extension.md)

21. **Don't fine-tune with RL yet.** D-5's BAI Thompson sampling already qualifies as soft contextual RL (logarithmic regret). Don't bolt gradient RL onto the 7B base until BAI converges AND a dense verifiable reward exists. When you DO escalate, DPO is the wrong primitive (3-7pp OOD drop per Apple 2025) — use KTO+RLVR. SFT-on-rollouts (SWE-Gym pattern, +12-14pp on SWE-Bench from 491 traces) likely beats any RL fine-tune. → [55](reports/55-agentic-rl-and-reward.md)

22. **LSP as precision oracle, not retriever.** The cited "LSP doesn't help agents" finding (Preprints 202510.0924) was about LSP-as-retriever. LSP DOMINATES when used as a verifier on candidate edits. Add diagnostic-probe pre-AUDITING gate via PostToolUse hook shelling `rust-analyzer diagnostics` / `pyright` / `tsc --noEmit`. rust-analyzer headless CLI (`analysis-stats`, `scip`, `lsif`, `ssr`, `diagnostics`) is drop-in. → [52](reports/52-lsp-integration.md)

23. **D-6 9-gram filter is broken before deployment.** Weaker than GPT-3's 13-gram which arxiv:2311.04850 already proved bypassable; 78% CodeForces soft-contamination failure rate (arxiv:2602.12413). Move to per-family directory shard + `_index/{family}.jsonl` to avoid NTFS perf cliffs at 5K tasks. Eval-awareness is no longer hypothetical — Sonnet 4.5 verbalizes "I'm being evaluated" >80% of the time, so D-23 sycophancy probes need matched-pair (eval vs deploy) framing. → [54](reports/54-eval-corpus-design.md)

24. **Narrow Z3 SMT in D-3 verifier consensus, gated by CSQ Critical.** Kani for Rust hot paths in AUDITING. AutoVerus-style Verus annotation for ELAI's FSM/IPC invariants (CI-gating COMMITTING, not per-audit). Dafny as verification-aware IR for safety-critical Python. HARD AVOID: Lean theorem-prover loops as routine audit gate (TPU-day latency). → [53](reports/53-formal-verification-codegen.md)

25. **Apple fast-path via MLX + universal fallback via llama.cpp/GGUF**, both added as B-37 RuntimeAdapters wrapping Anthropic `/v1/messages`. Keep Win+WSL2 primary. Defer NPU. Consider Tauri-webview + transformers.js + WebGPU for tiny utility agents (no Python sidecar). M5 Max measured Qwen 1.7B at 158 tok/s — Apple Silicon is now a viable consumer tier. → [51](reports/51-edge-inference-and-mlx.md)

## Reading order tip

If you only read **5 reports**: `10` (hostile critic) + `06` (MAST/multi-agent failures) + `17` (RAG quality) + `30` (public incidents) + `31` (coding model leaderboard) gives you the strategic landscape.

If you only read **3 reports for shipping next week**: `02` (streaming UX), `17` (RAG fix), `27` (quant switch).

## Wave / report index (50 reports across 10 waves)

### Wave 1 — UX + memory (chat experience pain)
- [01-infinite-context-chat-illusion.md](reports/01-infinite-context-chat-illusion.md) — Letta + Mem0 + Zep + HippoRAG patterns; sleep-time-compute digest
- [02-local-llm-speed-tricks.md](reports/02-local-llm-speed-tricks.md) — SSE frame-pacing, prompt-lookup-decoding (only HF-API one-liner)
- [03-model-handoff-ux.md](reports/03-model-handoff-ux.md) — show-role-hide-model; sleep-time digest > DroidSpeak KV; CUDA-graph warm pool
- [04-chat-ux-critique.md](reports/04-chat-ux-critique.md) — sycophancy/persona-drift/repair-resistance + 14 CommandWindow UX rules
- [05-user-modeling-and-memory.md](reports/05-user-modeling-and-memory.md) — Mem0 + Zep collapsed to SQLite; MINJA defense

### Wave 2 — orchestration quality + critique
- [06-multi-agent-failure-modes.md](reports/06-multi-agent-failure-modes.md) — MAST 14 modes; delete agent roles without >1σ iso-budget lift
- [07-darwinian-self-improvement.md](reports/07-darwinian-self-improvement.md) — DGM-reward-hacked; D-plan per-D-N grounding strength critique
- [08-self-evolving-prompts.md](reports/08-self-evolving-prompts.md) — GEPA > MIPROv2; cloud-Sonnet optimizer, local-Qwen target
- [09-agent-eval-benchmarks.md](reports/09-agent-eval-benchmarks.md) — SWE-Live + Pro + tau2-bench; ClawsBench scaffold-vs-model evidence
- [10-critic-of-elai-design.md](reports/10-critic-of-elai-design.md) — 14 hostile punches; 5 conceded strengths; 5 redesign recs

### Wave 3 — idea-to-product
- [11-idea-to-saas-prior-art.md](reports/11-idea-to-saas-prior-art.md) — v0/Bolt/Lovable/Devin; SWE-Verified→Pro 87→23% cliff
- [12-production-fullstack-codegen.md](reports/12-production-fullstack-codegen.md) — BetterAuth+Drizzle+OTel+axe-core+Biome stack-tier table
- [13-tdd-verification-agents.md](reports/13-tdd-verification-agents.md) — PGS +13.4% over example-TDD; cargo-mutants gate at AUDITING
- [14-browser-e2e-agents.md](reports/14-browser-e2e-agents.md) — AXTree+axe-core Tier 1; DeployPreviewWorker on Vercel Checks
- [15-sandboxed-exec.md](reports/15-sandboxed-exec.md) — Firecracker snapshot 28ms; chroot dead post-CVE; AppContainer on Windows

### Wave 4 — validation + retrieval + routing
- [16-vlm-image-validation.md](reports/16-vlm-image-validation.md) — AXTree-YAML Tier 1; LPIPS-delta convergence in RCE
- [17-rag-retrieval-quality.md](reports/17-rag-retrieval-quality.md) — stage ONNX models; Anthropic Contextual Retrieval; Jina-rerank-v3
- [18-tool-use-reliability.md](reports/18-tool-use-reliability.md) — XGrammar Tier 2; SEARCH/REPLACE non-git fences; Hermes envelope
- [19-reasoning-quality.md](reports/19-reasoning-quality.md) — L2 weakest layer; L4 AB-MCTS needs PRM-lite scoring; NF4 reasoning risk
- [20-local-cloud-routing.md](reports/20-local-cloud-routing.md) — xRouter/RouterArena/C3PO; data-leakage routing rules

### Wave 5 — peer landscape + Claude Code internals
- [21-peer-systems-llm-as-worker.md](reports/21-peer-systems-llm-as-worker.md) — Devin 3/20; AGENTLESS literature; Rust crate ecosystem
- [22-github-trending-may2026.md](reports/22-github-trending-may2026.md) — SKILL.md cross-vendor; microsandbox/libkrun new default; Archon/ruflo peers
- [23-developer-discourse-2026.md](reports/23-developer-discourse-2026.md) — METR -19% slowdown vs Nov 2025 inflection; trust collapse SO 70→29%
- [24-agent-protocols-mcp.md](reports/24-agent-protocols-mcp.md) — Adopt MCP external-only; Streamable HTTP since Mar 2025; postcard wins for IPC
- [25-claude-code-architecture.md](reports/25-claude-code-architecture.md) — 98.4% deterministic infra is the moat; 6-tier compaction ladder

### Wave 6 — inference, kernels, observability, incidents
- [26-inference-engines-2026.md](reports/26-inference-engines-2026.md) — Hybrid: keep sidecar + add SGLang adapter; drop Ollama (19x gap)
- [27-quantization-2026.md](reports/27-quantization-2026.md) — Switch to AWQ+Marlin; FP8 KV = 2x context free
- [28-attention-kernels-2026.md](reports/28-attention-kernels-2026.md) — Keep FA2 on Ampere; FA3/4 Hopper-only; StreamingLLM sinks + YaRN cautious
- [29-agentic-observability.md](reports/29-agentic-observability.md) — OTel GenAI semconv 1.40.0 (Apr 2026); SSB-as-trace pattern review
- [30-public-agent-incidents.md](reports/30-public-agent-incidents.md) — Replit 11x ALL CAPS ignored; ACRFence rollback attack; 8-pattern failure taxonomy

### Wave 7 — model + UI + Rust ecosystem + structured output
- [31-coding-model-leaderboard.md](reports/31-coding-model-leaderboard.md) — Qwen2.5-Coder lane has no successor; Qwen3 broke FA2 compat
- [32-context-window-extension.md](reports/32-context-window-extension.md) — Raise to 32K native; skip YaRN; F-in-M calibration +15pp free
- [33-dashboard-ux-agentic.md](reports/33-dashboard-ux-agentic.md) — Activity ribbon (Kiro); skeleton screens 20-30% perceived speed
- [34-rust-ai-ecosystem-2026.md](reports/34-rust-ai-ecosystem-2026.md) — `ort 2.0.0-rc.12` is production (anti-stale rule unenforceable); `rmcp 1.7.0` canonical
- [35-structured-output-2026.md](reports/35-structured-output-2026.md) — XGrammar tier-2; CodeAct +20% (-30% turns); SEARCH/REPLACE non-git fences

### Wave 8 — vision RAG, security, strategy, economics, graphs
- [36-document-vision-rag.md](reports/36-document-vision-rag.md) — ColQwen2.5 ViDoRe v2 0.597; PaddleOCR-VL 0.9B; new VisualArtifactCheck GRC method
- [37-prompt-injection-defense.md](reports/37-prompt-injection-defense.md) — Spotlighting + PromptArmor off-model classifier; data plane unguarded
- [38-ai-roadmap-2026-signals.md](reports/38-ai-roadmap-2026-signals.md) — MoE dominates 2026; RAG-first stays right through 2027; EU AI Act May enforcement
- [39-cost-token-economics.md](reports/39-cost-token-economics.md) — $47K wallet-loop; Anthropic 90% cache discount; cascade economics
- [40-symbol-graph-tools.md](reports/40-symbol-graph-tools.md) — SCIP + tree-sitter hybrid; Stack Graphs archived Sep 2025; FocalAD cite misattributed?

### Wave 9 — CI/CD, grammars, skills, durability, drift
- [41-ci-cd-pr-review.md](reports/41-ci-cd-pr-review.md) — 4-leg defense (in-flow / PR / deploy / observe); Coderabbit/Greptile FP rates
- [42-tree-sitter-grammars-2026.md](reports/42-tree-sitter-grammars-2026.md) — Pin 0.26.x ABI 15; TIER-A vs TIER-B grammar split; Svelte 5 runes ungrammared
- [43-agent-skill-systems.md](reports/43-agent-skill-systems.md) — Publish 3 skills externally; ToxicSkills + ClawHavoc require signed install
- [44-checkpointing-durability.md](reports/44-checkpointing-durability.md) — ACRFence semantic rollback attack on Claude Code; idempotency-keyed wrappers
- [45-cross-language-drift.md](reports/45-cross-language-drift.md) — `ts-rs` snapshot tests; schemars+typify for Rust↔Python via JSON Schema

### Wave 10 — planning, fine-tune, deep research, metrics, SaaS
- [46-planning-decomposition.md](reports/46-planning-decomposition.md) — AGENTLESS-style less-decomposition wins; goal-drift Jaccard canary
- [47-lora-finetuning.md](reports/47-lora-finetuning.md) — DEFER per-project LoRA; catastrophic forgetting 5-19pp; Unsloth+QLoRA+PiSSA+DoRA if shipping
- [48-web-research-orchestrator.md](reports/48-web-research-orchestrator.md) — GhostCite/CITEVERIFIER hallucination defense; contradiction-surfacing patterns
- [49-coding-agent-metrics.md](reports/49-coding-agent-metrics.md) — STABLE-Pass@1 + $/CTS not Pass@1; HAL ICLR 2026; harness vs model attribution
- [50-saas-architecture-patterns.md](reports/50-saas-architecture-patterns.md) — Opinionated SaaS template Designer should emit; 4-archetype stack table

### Wave 11 — edge, LSP, formal verification, eval, RL
- [51-edge-inference-and-mlx.md](reports/51-edge-inference-and-mlx.md) — MLX + llama.cpp adapters into B-37; M5 Max Qwen 1.7B 158 tok/s; defer NPU
- [52-lsp-integration.md](reports/52-lsp-integration.md) — LSP as verifier not retriever; rust-analyzer headless CLI; multilspy multi-LSP client; jdtls/RA RAM hogs
- [53-formal-verification-codegen.md](reports/53-formal-verification-codegen.md) — Z3 in D-3 (CSQ Critical only); Kani for Rust; AutoVerus for FSM; avoid Lean routine audit
- [54-eval-corpus-design.md](reports/54-eval-corpus-design.md) — D-6 9-gram is broken; eval-awareness >80% on Sonnet 4.5; per-family shard + jsonl index
- [55-agentic-rl-and-reward.md](reports/55-agentic-rl-and-reward.md) — BAI is already soft RL; KTO+RLVR not DPO; SFT-on-rollouts beats RL; PRM+ORM hybrid

## Paper cache locations

- `/workspace/ELAI\workflow\improvement-sweep-2026-05-19\papers\` — papers saved during this sweep (~20 PDFs incl. CodeRAG-Bench, ColPali, LeanRAG, AGENTLESS, Magentic-One, LATS, Goal Drift, AgentOrchestra, AB-MCTS, FA inverse-scaling, MAST, Mem0, Memory-Poisoning, Zep, sycophancy-LVLM, xgrammar, GEPA, TextGrad, OPRO-small)
- `/workspace/ELAI\.claude\papers\` — pre-existing + new (DGM, AlphaEvolve, MAST, Self-Correction Bench, Self-MoA, Sleep-time Compute, HippoRAG, ARC Prize 2025, soft-contamination, school-of-reward-hacks, statistical-limits-self-improving, evaluator-stress-test, LongMemEval, A-MEM, Constitutional-Classifiers, MELON, design-patterns-llm-agents)

## Open methodological note

Every report has a "Open questions / what needs measurement" section. These are the experiments ELAI must run to convert recommendations from grounded-conjecture to grounded-measured. The sum across 50 reports is ~150-200 distinct measurable experiments — that's ELAI's research backlog.

## Source-quality stats

- All 55 reports: minimum 12 web fetches enforced; most ran 14-20.
- Estimated total: ~770 grounded calls; ~75 PDFs cached.
- Citation tier policy: arxiv > official docs > github > blog > forum.
- Conjecture explicitly labeled "Ungrounded:" — visible in each report.
- ~12 reports flagged specific incorrect/misattributed claims in ELAI's existing B/C/D plans (notably FocalAD citation, "ort stable not -rc" anti-stale rule, D-5/D-6/D-7 evidence gaps, D-9 lock-list partial defense vs DGM's own reward-hacking, several D-plan claims with weak evidence).

# AI roadmap signals (May 2026, for ELAI strategy)

## TL;DR
- MoE is the de-facto frontier architecture (Mixtral, Gemini, GPT-4/5, DeepSeek V4, Qwen 3.6, Llama 4). ELAI's dense-only Qwen2.5-Coder bet is fine for the 7-12 GB VRAM tier but ELAI must NOT hard-code dense assumptions into runtime/adapter layers — B-37 RuntimeAdapter is the right hedge.
- FlashAttention-2 is now legacy: FA-3 is Hopper-only, FA-4 targets Hopper+Blackwell. Ampere (3090/RTX 30-series) keeps FA-2 indefinitely but loses FA-3/4 perf. ELAI's `ELAI_FORCE_SDPA=1` fallback is correct insurance; do not assume FA-3 will ever land on the 3090.
- Effective long-context is still a lie: Claude Opus 4.7 advertises 1M tokens but scores 32.2% on MRCR v2 at full length; Llama 4 Scout 10M scores 15.6%. RAG-first stays correct for ELAI through 2027. Do NOT collapse mistake_book / cosmos_worker / RAG layers into "just put it in context".
- MCP is now a Linux Foundation standard (donated Dec 2025). 2026 roadmap: stateless transport, Tasks primitive, MCP Registry Q4, SAML/OIDC. ELAI is on the right protocol; track the transport-v2 change carefully.
- Regulatory: EU AI Act high-risk obligations PUSHED from Aug 2026 to Dec 2027 (Digital Omnibus, May 7 2026). GPAI obligations live Aug 2025; transparency live Aug 2026. US EO Dec 2025 federal-preempts state laws. China export controls reverted to case-by-case Jan 2026. Determinism + audit-trail design (rules-as-data, FSM, evidence packs) is the right hedge for both regimes.

## Key sources
URL | who | takeaway (grouped: labs, hardware, regulatory, agentic)

Labs:
- https://www.anthropic.com/news/claude-opus-4-7 | Anthropic | Opus 4.7 ships Apr 16 2026; SWE-bench Verified 87.6%, SWE-bench Pro 64.3%; "xhigh" effort tier added.
- https://www.dwarkesh.com/p/dario-amodei-2 | Amodei/Patel | Feb 2026 podcast; "near end of the exponential" framing; 80x annualized Q1 growth; powerful-AI 2026-27 with ±1-2yr error bar.
- https://red.anthropic.com/2026/mythos-preview/ | Anthropic | Mythos = unreleased frontier; senior-SWE level; Project Glasswing limited access; Anthropic explicitly will NOT make Mythos generally available.
- https://techcrunch.com/2026/04/23/openai-chatgpt-gpt-5-5-ai-model-superapp/ | OpenAI | GPT-5.5 super-app push; Codex agent productized; memory called "GPT-2 era" by Altman.
- https://eu.36kr.com/en/p/3657136054984840 | Altman interview | Goal: 100x intellectual-cost reduction by end-of-2027; agents "confirmed for permanent work".
- https://deepmind.google/models/gemini/ | Google DeepMind | Gemini 3 shipped; 3.5 in dev; Astra/Mariner universal-assistant focus; agentic-first apps predicted late 2026.
- https://www.technologyreview.com/2026/01/22/1131661/yann-lecuns-new-venture-ami-labs/ | MIT TR | LeCun left Meta Mar 10 2026; AMI Labs raised $1.03B at $3.5B; JEPA/world-models bet AGAINST LLM scaling.
- https://www.cnbc.com/2026/04/24/deepseek-v4-llm-preview-open-source-ai-competition-china.html | CNBC/DeepSeek | V4-Pro 1.6T params, 1M context; trails GPT-5.4/Gemini 3.1 Pro by 3-6 months; open-weight + Huawei-chip optimized.
- https://venturebeat.com/ai/mistral-launches-powerful-devstral-2-coding-model-including-open-source | Mistral | Devstral 2 123B, SWE-bench Verified 72.2%; best open-weight coder; EU sovereign-AI positioning.
- https://www.mindstudio.ai/blog/xai-grok-roadmap-7-models-training-grok-5-10-trillion | xAI | 7 models training on Colossus 2; Grok 5 at 6T/10T params; Q1 2026 ship; Digital Optimus Sept 2026 target.

Hardware:
- https://www.tomshardware.com/pc-components/gpus/nvidia-announces-rubin-gpus-in-2026-rubin-ultra-in-2027-feynam-after | Nvidia | Rubin R200 Q2 2026; Rubin Ultra Q2 2027; Feynman after; HBM4E 288 GB; NVLink 6 3600 GB/s.
- https://videocardz.com/newz/amd-launches-instinct-mi350-series-confirms-mi400-in-2026-with-432gb-hbm4-memory | AMD | MI355X shipping; MI450 (Helios) 2H 2026; 432 GB HBM4, 40 PFLOPS FP4.
- https://www.cerebras.ai/press-release/awscollaboration | AWS+Cerebras | Trainium3 + WSE-3 disaggregated inference (Trainium prefill, Cerebras decode); rollout Q3 2026.
- https://machinelearning.apple.com/research/exploring-llms-mlx-m5 | Apple | M5 Neural Accelerators in every GPU core; MLX 20-30% faster than llama.cpp on Apple Silicon; M5 Pro/Max MacBook Pro Mar 11 2026; Mac Studio M5 Ultra delayed to Oct 2026+.
- https://tridao.me/publications/flash3/flash3.pdf | Dao-AILab | FA-3 Hopper-only; FA-4 Hopper+Blackwell CuTeDSL; Ampere stuck on FA-2.

Regulatory:
- https://artificialintelligenceact.eu/implementation-timeline/ | EU AI Act | GPAI obligations live Aug 2 2025; transparency Aug 2 2026; HIGH-RISK Annex III POSTPONED to Dec 2 2027 per May 7 2026 Digital Omnibus.
- https://www.paulhastings.com/insights/client-alerts/president-trump-signs-executive-order-challenging-state-ai-laws | Paul Hastings | Dec 2025 EO federal-preempts state AI laws; DOJ AI Litigation Task Force live Jan 10 2026.
- https://bisi.org.uk/reports/trump-reverses-us-ai-chip-export-policy-to-china | BISI | H200/MI325X exports to China allowed Jan 13 2026 with 25% tariff; case-by-case review; AI OVERWATCH Act in House.

Agentic:
- https://modelcontextprotocol.io/development/roadmap | MCP | Linux Foundation governance; stateless transport, Tasks primitive, MCP Registry Q4 2026, SAML/OIDC/PKCE Q2.
- https://workos.com/blog/anthropics-computer-use-versus-openais-computer-using-agent-cua | WorkOS | Claude Sonnet 4.6 OSWorld 72.5%; Claude Cowork desktop Mar 23 2026; OpenAI Operator shut Aug 31 2025, replaced by Atlas+Codex desktop sessions; Google = browser/DOM-anchored.
- https://softmaxdata.com/blog/definitive-guide-to-agentic-frameworks-in-2026-langgraph-crewai-ag2-openai-and-more/ | softmaxdata | LangGraph v1.1.3 + distributed runtime; CrewAI v1.12 multi-provider; AG2 Beta ground-up redesign; LangGraph passed CrewAI in stars early 2026.

## Per-lab signals

Anthropic — Stated direction: Opus 4.7 + Mythos (unreleased frontier) + Project Glasswing limited-access pattern; MCP as the agent-integration standard; 80x annual usage growth; up to 5 GW AWS + 5 GW Google/Broadcom (2027) + $30 B Azure capacity. Sources: anthropic.com/news/claude-opus-4-7, red.anthropic.com/2026/mythos-preview, mindstudio.ai/blog/dario-amodei-80x-growth-anthropic-q1-2026-revenue. Implication for ELAI: ELAI is already aligned (MCP, Anthropic-protocol sidecar). Plan for a world where the most capable Anthropic model is gated to a small list — keep open-weight (Qwen, DeepSeek, Devstral) parity work warm via RuntimeAdapter.

OpenAI — Stated direction: GPT-5.5 super-app, Codex desktop agents, Agent Builder/ChatKit/Connector Registry, "memory as next frontier", 100x cost reduction by end-2027, Stargate buildout. Sources: techcrunch.com/2026/04/23, eu.36kr.com/en/p/3657136054984840, openai.com/index/introducing-operator (Operator shut Aug 2025). Implication: ELAI's memory layers (mistake_book, cosmos_worker, RAG) align with the stated frontier; do NOT collapse them. Expect Connector Registry to compete with bespoke MCP servers — keep ELAI MCP servers thin and replaceable.

Google DeepMind — Stated direction: Gemini 3 shipped, 3.5 expected Google I/O 2026 (May 12-14); Astra/Mariner universal-assistant via Chrome+Workspace; browser/DOM-anchored agentic stack; 10M context (Gemini 3 Pro $12/1M input). Sources: deepmind.google/models/gemini, blog.google/innovation-and-ai/models-and-research/google-deepmind/gemini-universal-ai-assistant, deepmind.google/models/project-astra. Implication: Long-context-first is now table stakes for Google; ELAI's RAG-first stays cheaper and more deterministic — message that as a feature, not a limitation.

Meta — Stated direction: Llama 4 Scout 10M context shipped; LeCun left to AMI Labs ($1.03B Mar 2026); FAIR vs GenAI split widening; world-models / V-JEPA / VL-JEPA as the contrarian bet. Sources: technologyreview.com/2026/01/22/1131661, ai.meta.com/research/vjepa, bdtechtalks.com/2026/01/03/meta-vl-jepa-vision-language-model. Implication: Open-weight Llama remains a fallback; world-models are a 5-yr horizon — not in ELAI's 2026-2027 critical path. Track JEPA only as future option in RuntimeAdapter.

xAI — Stated direction: 7 models training simultaneously; Grok 5 6T/10T params Q1 2026; Tesla FSD auto-labeling flywheel; Digital Optimus Sept 2026. Sources: mindstudio.ai/blog/xai-grok-roadmap-7-models-training-grok-5-10-trillion, teslarati.com/tesla-xai-digital-optimus-explained. Implication: Grok unlikely a sidecar candidate for ELAI (X-bound, closed-weight focus). Skip.

DeepSeek — Stated direction: V4-Pro (1.6T, 1M context, MoE) + V4-Flash; trails GPT-5.4/Gemini 3.1 Pro by ~3-6 months; Huawei-chip native; pricing $1.74/3.48 per M tokens Pro. Sources: cnbc.com/2026/04/24/deepseek-v4-llm-preview-open-source-ai-competition-china, simonwillison.net/2026/apr/24/deepseek-v4. Implication: V4-Flash is a candidate ELAI cloud-mode option (cheaper than Sonnet, OpenAI-compatible API). Verify license + EU/US legal posture before wiring.

Mistral — Stated direction: Devstral 2 (123B, SWE-bench 72.2%); Vibe 2.0 IDE assistant; Codestral 25.08; EU sovereign-AI play. Sources: venturebeat.com/ai/mistral-launches-powerful-devstral-2-coding-model-including-open-source, europe.mistral.ai. Implication: Devstral 2 is a strong open-weight backup if Qwen-Coder line stalls. NF4-quantized Devstral 2 on 2x3090 = next test target after the user's second 3090 arrives.

## Hardware roadmap

- Nvidia Blackwell Ultra (B300): shipping H2 2025; production scaling 2026. Source: techspot.com/news/106960.
- Nvidia Rubin (R200): Q2 2026; HBM4E 288 GB, NVLink 6 3600 GB/s, CX9 1600 Gb/s. Source: tomshardware.com/.../nvidia-announces-rubin-gpus.
- Nvidia Rubin Ultra (VR200): Q2 2027. Same source.
- Nvidia Feynman: 2028+. Same source.
- AMD MI355X: 2025 GA; MLPerf v5.1 93,045 tok/s Llama-2-70B; FP4 beats B200 in some workloads. Source: amd.com/.../mi350-game-changer, tomshardware.com/.../inside-the-instinct-mi355x.
- AMD MI450 (Helios rack): 2H 2026; 432 GB HBM4, 40 PFLOPS FP4, 19.6 TB/s bw, 300 GB/s scale-out. Source: videocardz.com/newz/amd-launches-instinct-mi350-series.
- AWS Trainium3 + Cerebras WSE-3 disaggregated (prefill/decode split): rollout US-East/West Q3 2026. Source: cerebras.ai/press-release/awscollaboration.
- Groq: Nvidia acquihired ~$20B (late 2025); LPUs being folded into Nvidia inference stack. Source: nextplatform.com/compute/2026/04/22/the-second-time-will-be-the-ipo-charm-for-cerebras.
- Apple M5 Pro/Max (MBP): Mar 11 2026; 128 GB unified; ~4x M4 prompt processing; MLX 20-30% faster than llama.cpp. Source: machinelearning.apple.com/research/exploring-llms-mlx-m5, modelfit.io/blog/m5-pro-max-local-llm-2026.
- Apple Mac Studio M5 Max/Ultra: delayed to Oct 2026+ (memory shortages). Source: 9to5mac.com/2025/11/20/apple-shows-how-much-faster-the-m5-runs-local-llms.

## Regulatory landscape

EU AI Act:
- GPAI provider obligations live Aug 2 2025 (new providers immediately; existing must comply by Aug 2 2027). Source: artificialintelligenceact.eu/implementation-timeline.
- Article 50 transparency + nudifier ban: apply from Dec 2 2026. Source: digital-strategy.ec.europa.eu/en/policies/regulatory-framework-ai.
- HIGH-RISK Annex III obligations: POSTPONED from Aug 2 2026 to Dec 2 2027 per Digital Omnibus May 7 2026. Source: trilateralresearch.com/responsible-ai/eu-ai-act-implementation-timeline-mapping-your-models-to-the-new-risk-tiers.
- Commission enforcement powers for GPAI live Aug 2 2026.

US:
- Trump Dec 2025 EO "Ensuring a National Policy for AI": federal preempts state AI laws; DOJ AI Litigation Task Force from Jan 10 2026. Source: paulhastings.com/insights/client-alerts/president-trump-signs-executive-order-challenging-state-ai-laws.
- Chip exports to China: H200/MI325X allowed Jan 13 2026 case-by-case + 25% value-based tariff Jan 15 2026. Source: bisi.org.uk/reports/trump-reverses-us-ai-chip-export-policy-to-china, mayerbrown.com/en/insights/publications/2026/01/administration-policies-on-advanced-ai-chips-codified.
- AI OVERWATCH Act passed House Foreign Affairs Jan 21 2026 — would re-block Blackwell exports for 2 yrs.

China:
- DeepSeek V4 Huawei-chip-native = signal of decoupling. Source: fortune.com/2026/04/24/deepseek-v4-ai-model-price-performance-china-open-source.
- China AI law status: not surfaced in search; flag as open question.

## ELAI bets that could go wrong

Bet 1: Dense-only Qwen2.5-Coder + Qwen 32B on 2x3090. Scenario where it loses: every other lab ships MoE-only; dense scaling laws break; community tooling drifts toward MoE-only runtimes. Likelihood: HIGH — MoE is already dominant per callsphere.ai/blog/mixture-of-experts-architecture-why-moe-dominates-2026-llms. Mitigation: RuntimeAdapter (B-37) abstracts model class; add a per-VRAM-tier MoE config (Mixtral, DeepSeek V4-Flash distilled, Qwen-MoE if released) in the VRAM-tier plan.

Bet 2: FlashAttention-2 on Ampere as canonical. Scenario where it loses: PyTorch deprecates FA-2 build path; vLLM/SGLang drop Ampere fast path; FA-3-only kernels become default. Likelihood: MEDIUM in 2026, RISING in 2027. Source: tridao.me/publications/flash3/flash3.pdf shows FA-3 Hopper-only; FA-4 CuTeDSL Hopper+Blackwell only. Mitigation: ELAI_FORCE_SDPA=1 default already in place; do NOT make FA-2 a hard dep anywhere.

Bet 3: RAG-first / mistake_book / cosmos_worker layered memory. Scenario where it loses: 10M+ "effective" context becomes real (today it's marketing — Opus 4.7 32.2% at 1M MRCR; Scout 15.6% at 10M). Likelihood: LOW for 2026, MEDIUM 2027. Sources: digitalapplied.com/blog/ai-context-window-comparison-2026-1m-to-10m-tokens, medium.com/@administrator_54541/the-1-million-token-lie. Mitigation: keep RAG layers behind FsmSubstate-pluggable boundaries; if effective context truly hits 1M+, RAG can be a cost-saver path not a correctness path.

Bet 4: MCP as agent-integration protocol. Scenario where it loses: OpenAI Connector Registry + Google Agentic Web Standards fracture the ecosystem; MCP becomes Anthropic-only de-facto. Likelihood: LOW — Linux Foundation governance + Q4 MCP Registry roadmap make this an open standard. Sources: modelcontextprotocol.io/development/roadmap, workos.com/blog/everything-your-team-needs-to-know-about-mcp-in-2026. Mitigation: keep MCP servers thin + spec-driven; add OpenAI Agents SDK shim if Connector Registry becomes dominant.

Bet 5: Sidecar (single remote 3090) as canonical. Scenario where it loses: AWS Trainium3+Cerebras disaggregated inference (Q3 2026) makes single-GPU prefill+decode look slow/expensive even at hobbyist scale. Likelihood: MEDIUM. Source: cerebras.ai/press-release/awscollaboration. Mitigation: B-37 RuntimeAdapter already supports cloud backends; document the local-vs-cloud cost crossover explicitly.

Bet 6: Determinism-first + rules-as-data. Scenario where it loses: under EU AI Act high-risk (now Dec 2027) determinism + audit trails become competitive REQUIREMENTS — bet PAYS, not loses. Under US federal-preemption EO determinism still wins because cross-state liability surfaces are smaller. Likelihood of LOSING: LOW. Source: artificialintelligenceact.eu/implementation-timeline. Mitigation: none needed — this is the strongest bet in the project.

Bet 7: No-MoE local stance. Scenario where it loses: open-weight MoE (Mixtral 8x22B, DeepSeek V4, future Qwen-MoE) becomes the only competitive open-weight option at 12-24 GB tier via expert offloading. Likelihood: MEDIUM-HIGH. Source: callsphere.ai/blog/mixture-of-experts-architecture-why-moe-dominates-2026-llms. Mitigation: MoE-on-Ampere is a workable VRAM-tier-plan deliverable (expert offload to CPU/disk); add to roadmap.

## Best architecture insurance

- RuntimeAdapter / PerRoleRuntime / RuntimeOrchestrator (B-37): hedges against any single-lab-wins, MoE-vs-dense, local-vs-cloud, FA-2-vs-FA-3 future. Source: shipped in fd5c2e5; aligned with mindstudio.ai/blog/grok-5-agi-xai-model-roadmap and softmaxdata.com agent-framework comparison showing multi-provider as the dominant 2026 pattern.
- MCP-first tool exposure: hedges against per-lab agent SDK churn; aligned with linuxfoundation governance per modelcontextprotocol.io/development/roadmap.
- FSM + rules-as-data + evidence packs: hedges against EU/US/China audit-trail divergence; aligned with EU AI Act art. 12-13 logging requirements per artificialintelligenceact.eu/implementation-timeline.
- Spec-grounded + paper-ingested architecture: hedges against training-data staleness; this report itself demonstrates the pattern.
- RAG + cosmos_worker + mistake_book layered memory: hedges against the "1M token lie"; even if effective context grows, layered memory stays cheaper and more deterministic per medium.com/@reliabledataengineering/stop-chasing-million-token-context-windows.

## Anti-patterns

- Hard-coding FA-2 anywhere (any `import flash_attn_2` or runtime assertion that FA-2 is present). Source: tridao.me/publications/flash3/flash3.pdf; FA-2 will be legacy by 2027.
- Building a bespoke agent protocol parallel to MCP. Source: 97 M MCP downloads + Linux Foundation per digitalapplied.com/blog/mcp-97-million-downloads-model-context-protocol-mainstream.
- Locking sidecar to a single model family (Qwen-only path; rejecting MoE outright). Source: deepseek V4-Flash, Mixtral, Mistral Devstral 2 all open-weight, all distinct architectures; per fluxhire.ai/blog/ai-agents-2026-complete-guide.
- Trusting advertised context lengths without MRCR-v2 / NIAH measurement. Source: medium.com/@administrator_54541/the-1-million-token-lie.
- Skipping audit trail / determinism for "speed" — EU AI Act + US state-preempted-but-not-deleted laws make this a future liability. Source: paulhastings.com/insights/client-alerts/president-trump-signs-executive-order-challenging-state-ai-laws.
- Betting on Operator-style "general computer-use" as the agent UX. Source: OpenAI shut Operator Aug 31 2025 per workos.com/blog/anthropics-computer-use-versus-openais-computer-using-agent-cua; reliable agents are tool-call+MCP, not screenshot loops.

## Open questions
1. When does FA-2 actually break in upstream PyTorch / vLLM on Ampere? (Need: bisect upstream issue tracker; measurable as "first PyTorch release that fails FA-2 build on sm_86".)
2. Is Devstral 2 (123B) at NF4 viable on 2x3090 (48 GB)? (Measurable: model fit + ≥10 tok/s decode at 8K context once second 3090 arrives.)
3. Does Mythos / Glasswing-class capability ever ship to API? If yes, when? (Anthropic public statement says no — but the Glasswing list of orgs is the leading indicator. Measurable: tracker of Glasswing program expansion.)
4. Does Gemini 3.5 at Google I/O 2026 (May 12-14) include a real (not advertised) 10M effective context with MRCR>50%? (Measurable: post-I/O re-run of "1M token lie" benchmark.)
5. Does EU Code of Practice (Aug 2 2025+) impose anything beyond GPAI transparency on ELAI-class self-hosted-sidecar use cases? (Need: read Commission GPAI guidelines per digital-strategy.ec.europa.eu/en/policies/guidelines-gpai-providers.)

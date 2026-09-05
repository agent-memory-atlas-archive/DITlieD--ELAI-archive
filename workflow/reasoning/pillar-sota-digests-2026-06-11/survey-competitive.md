DIGEST: survey-competitive
SOURCE: 1ebef0fa-f94a-4121-94a7-675d5cb4278b.txt

TOPIC: 24-month (mid-2024 to mid-2026) competitive survey of SOTA AI agents / multi-agent orchestration / LLM reasoning across 9 dimensions, classifying each finding vs ELAI as Direct Apply / Adjacent / Novel-ELAI-lacks. Answers: what 2024-2026 SOTA does ELAI already cover, what it should adopt, what it lacks.

REPORT VERDICTS (the report's own priority-matrix calls on ELAI-deferred/DARK features):
- SPLADE / Learned Sparse Retrieval: ELAI=DARK -> HIGH priority adopt (SPLADE-Code SOTA for code retrieval, sub-ms on 1M passages).
- Speculative Decoding: ELAI=DARK -> HIGH adopt (production default in vLLM/SGLang/TRT-LLM, 2-3.6x).
- MoE-Retarget (CMoE): ELAI=DARK -> MEDIUM-HIGH (CMoE converts dense->MoE without training; all major 2025-26 models MoE).
- LoRA Fine-Tuning (PERL): ELAI=DEFERRED -> HIGH (PERL 50% mem reduction + 90% train speedup; DGM uses LoRA).
- MCTS-First Search: ELAI=DARK -> HIGH (THOUGHTSCULPT/C-MCTS/Planning-with-MCTS 40.59% avg improvement over zero-shot CoT).
- Batch Racing / Multi-Model Ensemble: ELAI=DEFERRED -> MEDIUM (SpecEM, Speculative Ensemble, IbED training-free ensemble accel).
- Dynamic KG (multi-language DKG): ELAI=ADJACENT -> HIGH (GraphRAG, DKG-LLM, fs1 KG-grounded reasoning).
- Darwinian Self-Improvement Loops: ELAI=NOVEL -> HIGH (research) (DGM 20->50% SWE-bench automatically; HGM, Live-SWE-agent).
- Conformal Refusal / Uncertainty: ELAI=ADJACENT -> MEDIUM (learnable conformal abstention w/ RL; ConU black-box CP).
- A2A Horizontal Protocol: ELAI=NOVEL -> HIGH (Linux Foundation standard; 50+ enterprise partners).
- XGrammar Constrained Decoding: ELAI=ADJACENT -> DIRECT APPLY (100x speedup; default in vLLM+SGLang).
- FP8 / AWQ Quantization: ELAI=ADJACENT -> DIRECT APPLY (FP8 -2.7% quality 1.5-2x; AWQ -0.2% 2-3x).
- Plan-Then-Execute Security Pattern: ELAI=ADJACENT -> DIRECT APPLY (formally-proven injection resistance; all 3 major frameworks implement).
- Computer Use / GUI Tool Gating: ELAI=DEFERRED -> MEDIUM (Claude 4.5 61.4% OSWorld; Magentic-One WebSurfer).
- GRPO / BAI Reward Frameworks: ELAI=DEFERRED -> HIGH (DeepSeek-R1 RL-only; GRPO more stable than PPO).
- DSPy GEPA Prompt Optimization: ELAI=ADJACENT -> DIRECT APPLY (GEPA genetic-Pareto; MIPROv2 +10-20pp clinical QA).
- Adversarial Debate Agents: ELAI=ADJACENT -> MEDIUM (STAR-PolyaMath meta-strategist; adversarial persuasion vuln documented).
- MemoryOS Hierarchical Memory: ELAI=ADJACENT -> DIRECT APPLY (heat-driven eviction, segment paging; beats MemGPT + A-MEM).
- 5 capabilities flagged as NOVEL/ELAI-lacks with no current equivalent: open-ended agent self-evolution (DGM/HGM); persistent Meta-Strategist (STAR-PolyaMath); MCP/A2A dual-protocol interoperability; causal grounding attribution (+20-35pp); execution-grounded security benchmarking (FinVault).

CANDIDATES (name | url-or-id | report's claim | replaces-or-hardens):
SPLADE-Code | arXiv:2603.22008 | first learned sparse retrievers specialized for source code (0.6B-8B), SOTA sub-1B (75.4 MTEB Code), 20+ langs, sub-ms on 1M passages | activates ELAI's DARK SPLADE for code-symbol retrieval
Mistral-SPLADE | arXiv:2408.11119 | LLMs for better learned sparse retrieval, BEIR SOTA | hardens sparse retrieval
SPLATE | SIGIR 2024 / no-url | ColBERT late-interaction with sparse first-stage on CPU | adjacent retrieval
Cohere Rerank v3 / ms-marco cross-encoder | no-url | precise top-k reranking without dense overhead | Direct Apply reranker
Microsoft GraphRAG | github.com/microsoft/graphrag arXiv:2404.16130 | LLM entity-relation KGs + community summaries; substantial comprehensiveness gains on global sensemaking | dynamic KG / community summarization (ELAI lacks)
DKG-LLM | arXiv:2508.06186 | dynamic KG + Adaptive Semantic Fusion + Bayesian inference; ~150 new nodes/category | adjacent dynamic KG
KnowGPT | arXiv:2506.13178 | structured graph reasoning into LLMs via dynamic prompting, no fine-tune | factual grounding
CA-MCP (Context-Aware MCP) | arXiv:2601.11595 | shared context store lets MCP servers coordinate, fewer LLM calls, fewer failures | MCP coordination / anti-stagnation
DeepSeek-R1 | nature s41586-025-09422-z, github deepseek-ai/DeepSeek-R1 | RL-only reasoning, emergent self-verification; 79.8% AIME2024, 97.3% MATH-500 | GRPO/RL-only training alternative to BAI/PRM
OpenAI o3 / o4-mini | techcrunch 2025-04-16 | multimodal "think with images"; o3 69.1% SWE-bench Verified no scaffolding | reasoning model reference
Gemini 2.0 Flash Thinking | blog.google | thinking traces + 1M ctx + native code exec | reasoning reference
VeriThinker | arXiv:2505.17941 github czg1225/VeriThinker | aux verification task cuts MATH-500 tokens 3790->2125, +0.8% acc | ELAI serial-depth budgeting primitive
STAR-PolyaMath | arXiv:2505.11140 / semanticscholar 230554c2 | orchestrated FSM + persistent Meta-Strategist; 93.75% MathArena Apex vs GPT-5.5 80.21% | FSM orchestration / anti-stagnation (ELAI lacks meta-strategist)
MADAWSD | aclanthology 2025.emnlp-main.1134 | multi-agent debate integrated for adversarial disambiguation | adversarial debate
ConU | arXiv:2407.00499 | conformal prediction for black-box API LLMs, self-consistency as nonconformity | conformal refusal (ELAI adjacent)
Learnable Conformal Abstention | huggingface/arXiv 2502.06884 | RL+CP optimizes abstention thresholds dynamically | novel conformal refusal
LLM-as-judge interval eval (CP) | aclanthology 2025.emnlp-main.569 | valid prediction intervals w/ coverage guarantees for discrete ratings | uncertainty quant
fs1 | arXiv:2505.11140v2 | grounds reasoning traces on KG paths from DeepSeek-R1 | KG factual grounding
FACTS Grounding | deepmind.google blog | 1719-example benchmark for full-grounding-no-hallucination | hallucination eval benchmark
LangGraph | github.com/langchain-ai/langgraph | v1.0 late-2025, de-facto FSM orchestration std; thread-based persistence + time-travel debugging; plan-then-execute | maps to ELAI FSM orchestration (Direct Apply)
AutoGen v0.4 / Magentic-One | github.com/microsoft/autogen | inner-outer loop, Task Ledger, WebSurfer/FileSurfer agents, OTel built-in | multi-agent coordination reference
Microsoft Agent Framework (MAF) | no-url | merges AutoGen + Semantic Kernel + MCP/A2A | orchestration
CrewAI | crewai.com/open-source | hierarchical manager-worker, allowed_agents param, shared memory tiers, Agentic RAG | hierarchical delegation
MCP | anthropic.com | agent<->tools protocol; 97M monthly SDK downloads, 5800+ servers; LF-donated Dec 2025 | tool/context protocol (ELAI has MCP hardening)
A2A | developers.googleblog.com | agent<->agent; JSON Agent Cards, task lifecycle, enterprise auth; LF-donated | horizontal coordination (ELAI lacks)
From-Glue-Code-to-Protocols | arXiv:2505.03864 | critical analysis of A2A+MCP integration for scalable agents | protocol integration
MemGPT | arXiv:2310.08560 research.memgpt.ai | OS-inspired paging fast/slow memory via function calls; flat FIFO causes topic mixing | memory baseline
MemoryOS | arXiv:2506.06326 | 3-tier hierarchical + heat-driven eviction + semantic segment retrieval; beats MemGPT/A-MEM | Direct Apply hierarchical memory
A-MEM | arXiv:2502.12110 | Zettelkasten notes + keywords/tags, agent-driven | memory architecture
MemInsight | arXiv:2503.21760 | autonomous semantic-attribute memory augmentation | memory enrichment
AI-native Memory 2.0 / Second Me | arXiv:2503.08102 | LLM-based memory parameterization beyond static retention | memory parameterization
Cognitive Workspace | arXiv:2508.13171 | active memory mgmt; 58.6% memory reuse vs 0% RAG, 17-18% net efficiency gain | beyond-RAG memory
QuantSpec | ICML 2025 (icml poster 46326) | self-speculative decoding w/ 4-bit hierarchical KV cache; >90% accept, ~2.5x speed, 1.3x mem | speculative decoding (ELAI DARK)
EAGLE | no-url | acceptance ~80% | speculative decoding
SpecEM | NeurIPS 2025 proceedings | training-free LLM ensembling via iterative drafting | batch racing/ensemble (ELAI deferred)
Speculative Ensemble | arXiv:2502.01662 | spec decoding to accelerate LLM ensembles w/o perf loss | ensemble accel
IbED (In-batch Ensemble Drafting) | iclr 2025 virtual 33700 | multiple drafting methods, low added latency for LVLMs | ensemble
XGrammar | no-url (MLC-AI) | default constrained-decode in vLLM/SGLang; up to 100x speedup; JSON/regex/choice/CFG | constrained decoding (Direct Apply)
SGLang RadixAttention | github.com/sgl-project/sglang | +XGrammar 20-40% higher throughput than vLLM at same concurrency for JSON | serving
CMoE | arXiv:2502.04416v2 | converts dense LLMs to MoE without training by clustering FFN neurons | MoE-retarget (ELAI DARK)
"Not All Models Suit Expert Offloading" | huggingface 2505.16056 | optimal cache ~2x active experts for no-shared-expert models | MoE edge inference
Darwin Godel Machine (DGM) | arXiv:2505.22954 github jennyzzt/dgm | self-modifies own code, validates on SWE-bench; 20%->50% over 80 iters | Darwinian self-improvement (ELAI lacks)
Huxley-Godel Machine (HGM) | arXiv:2510.21614 | CMP metric (descendant perf) predicts self-improve potential; beats DGM w/ fewer CPU hrs, human-level w/ GPT-5 | DGM extension
Live-SWE-agent | arXiv:2511.13646 | on-the-fly self-evolution from only bash tools during runtime | self-evolving agent
HyperAgents (Meta) | ai.meta.com/research/hyperagents | improvement procedure itself evolves, no domain-specific alignment | DGM extension
DARWIN (evolutionary GPT) | ownyourai.com/darwin | genetic algos mutate training code; 1.26% MFU, 2.07% perplexity over 5 iters | evolutionary self-improve
GRPO | no-url | more stable than PPO; +LoRA = 50% mem/90% speedup (reward model), 20% mem/10% speedup (RL loop) | reward fine-tuning (ELAI deferred)
DSPy MIPROv2 / GEPA | github stanfordnlp/dspy, dspy.ai/roadmap | prompt-engineering as optimization; GEPA genetic-Pareto 2026; +10-20pp clinical QA | prompt optimization (Direct Apply)
OPRO (Google DeepMind) | no-url | joint instruction+few-shot iteration | prompt optimization
Constitutional AI (Anthropic) | no-url | self-critique/revision via written principles, no human labelers | RLAIF / safe-completion
Youtu-Agent | arXiv:2512.24615 | in-context optim + distributed RL; 71.47% WebWalkerQA, 72.8% GAIA | hybrid policy optimization
Prompt Injection Design Patterns (6) | simonwillison.net 2025/Jun/13 (IBM/Invariant/ETH/Google/MS) | Plan-Then-Execute, Dual-LLM, Map-Reduce, Code-Then-Execute, Context-Min, Action-Selector | security patterns (Direct Apply Plan-Then-Execute)
CaMeL (DeepMind) | no-url | privileged LLM coordinates quarantined LLM via sandboxed DSL | dual-LLM security
PromptArmor | openreview IeNXtofK6T | GPT-4o system prompt detects/removes injections <1% FP and FN on AgentDojo | injection defense
Secure Plan-then-Execute | arXiv:2509.08646 | formally-proven injection resistance architecture | security pattern
OWASP Top 10 LLM 2025 | no-url | RAG poisoning: 5 crafted docs manipulate responses 90% of time | security threat model
IsolateGPT | no-url | secure-enclave encapsulation of agentic ops | sandboxing
Inter-Agent Trust Models | arXiv:2511.03434 | 6 mechanisms (Brief/Claim/Proof/Stake/Reputation/Constraint); Proof+Stake+Brief recommended | inter-agent trust
NANDA Index | arXiv:2508.03101 | Zero Trust Agentic Access w/ cryptographic capability attestation | MCP hardening
FinVault | arXiv:2601.07853 github aifinlab/FinVault | first execution-grounded security benchmark for financial agents; 31 scenarios/107 vulns/963 cases; SOTA defenses still 6.7% ASR | execution-grounded security bench (extends ELAI SAFEDELTA)
Perplexity NIST/CAISI submission | semanticscholar 91abd24 | maps attack surfaces: tools/connectors/hosting/multi-agent; confused-deputy + cascading failures | safety lattice
AI Canvas (Palo Alto Networks) | youtube/designrush | NL prompt-based dynamic AI-generated dashboards | canvas workbench
Figma AI / Google Stitch | no-url | multi-screen editable layouts from text; HTML/CSS export | UI generation
Anthropic Computer Use | anthropic.com/news/3-5-models-and-computer-use | general computer skills; Claude 4.5 61.4% OSWorld | GUI tool gating (ELAI deferred)
THOUGHTSCULPT | aclanthology 2025.findings-naacl.428 | reasoning w/ intermediate revision | MCTS-first search
Planning with MCTS | openreview sdpVfWOUQA | enhances problem-solving; part of 40.59% avg improvement | MCTS-first (ELAI DARK)
Causally Grounded LLM Attribution | pinnaclepubs EJACI 688 | full causal grounding +20-35pp attribution accuracy, smaller models benefit more | causal grounding (ELAI lacks)
Key repos: vllm-project/vllm, sgl-project/sglang.

ATTACK-THE-DESIGN (report's strongest criticisms of ELAI):
- ELAI marks SPLADE, speculative decoding, MoE-retarget, MCTS-first all DARK while 2025-26 production evidence overwhelmingly favors them; these are activation gaps not research gaps.
- ELAI lacks open-ended self-evolution (DGM paradigm qualitatively different from ELAI's rules-engine/self-mod core).
- ELAI lacks a persistent meta-strategist for stagnation escape (failure mode of reasoning agents looping on unproductive strategies).
- ELAI's MCP hardening covers tool access but no A2A horizontal coordination across vendor boundaries.
- ELAI grounding lacks structured causal provenance (causal attribution +20-35pp).
- ELAI SAFEDELTA lacks an execution-grounded adversarial benchmark w/ state-writable sandboxes (FinVault template).
- Caution flagged: single confident adversarial agent can LOWER multi-agent accuracy 10-40% and raise wrong-consensus >30% (debate is not free).
- Every major agent benchmark (SWE-bench/GAIA/WebArena/OSWorld) exploitable to near-perfect without solving tasks (Berkeley RDI).

NUMBERS (figure | claimed source):
- Hybrid search +8-15% over pure; cross-encoder rerank +15-30% precision on BEIR.
- SPLADE-Code 75.4 MTEB Code, 20+ langs, sub-ms on 1M passages.
- GraphRAG time-aware variant 82.7% EM on multi-hop, +24.4pp over vanilla RAG.
- DeepSeek-R1: 79.8% Pass@1 AIME 2024, 97.3% MATH-500.
- o3: 69.1% SWE-bench Verified no scaffolding.
- VeriThinker: 3790->2125 reasoning tokens MATH-500, +0.8% accuracy.
- STAR-PolyaMath: 93.75% MathArena Apex 2025 vs GPT-5.5 80.21%.
- Adversarial agent: -10-40% accuracy, +>30% wrong-consensus (Nature 2026, s41598-026-42705-7).
- MCP: 97M monthly SDK downloads, 5800+ servers. A2A: 50+ enterprise partners.
- Gartner: +1445% multi-agent inquiries Q1-2024 to Q2-2025. Multi-agent: 45% faster resolution, 60% more accurate vs single-agent.
- Cognitive Workspace: 58.6% memory reuse vs 0% RAG, 17-18% net efficiency.
- Quant: FP8 -2.7% quality 1.5-2x; AWQ INT4 -0.2% 2-3x; NF4/GPTQ ~2-5% 2-3x; NVFP4/MXFP4 ~87% savings 3x+.
- QuantSpec: >90% accept, ~2.5x speed, 1.3x mem. TRT-LLM H200: >3x throughput, +FP8 = 3.6x.
- XGrammar: up to 100x speedup. SGLang+XGrammar: 20-40% higher throughput vs vLLM. Constrained decode: 17% creativity cost.
- DGM: 20%->50% SWE-bench Verified over 80 iters. DARWIN: 1.26% MFU, 2.07% perplexity over 5 iters.
- MIPROv2: +10-20pp clinical QA. Youtu-Agent: 71.47% WebWalkerQA, 72.8% GAIA.
- OWASP: 5 docs -> 90% RAG manipulation. FinVault: 31 scenarios/107 vulns/963 cases, 6.7% avg ASR. CVE-2025-53773 (Copilot) CVSS 9.6.
- Benchmark SOTA table: SWE-bench Verified ~87% (Claude Opus 4.7); GAIA ~90% (SU Zero); WebArena 95.6% (Qwen3-235B); OSWorld ~51% (2025 ceiling); tau2-bench 85% (Gemini 3 Pro); BFCL V3 77% (Claude Opus 4.5). Claude 4.5 OSWorld 61.4%.
- Langfuse: LangGraph 27,100 monthly searches vs CrewAI 14,800.
- Causal grounding: +20-35pp attribution accuracy.

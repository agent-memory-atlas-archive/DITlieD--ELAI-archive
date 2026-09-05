Small LLM Coding Agents: Consolidated Research Summary (2025-2026)
====================================================================

Sources:
- 1111Small LLM Agents for SWE.txt
- 2222LLM Code Error Recovery Pipeline.txt
- 3333Small Code Model Enhancement Research.txt

Coverage period: June 2025 - April 2026


1. CORE THESIS
===============

The paradigm has shifted from "bigger model = better code" to "thick scaffold, thin LLM."
Sub-10B models (1B-7B) can match or approach frontier model performance when wrapped in
deterministic orchestration that handles state, memory, planning, and verification externally.
The LLM becomes a stateless, micro-task execution node -- not an autonomous reasoning agent.

Key insight: architectural determinism is a far stronger predictor of task success than
raw parameter count. Unscaffolded frontier models score 0-8% on complex tasks (ClawsBench),
while scaffolded models of any size surge to 39-63%.


2. SCAFFOLDING ARCHITECTURES
=============================

2.1 Taxonomy of Scaffolding (Rombaut, Apr 2026)
- Paper: "Inside the Scaffold: A Source-Code Taxonomy of Coding Agent Architectures"
- arXiv: 2604.03515
- Analyzed 13 open-source agents across 12 architectural dimensions
- Key dimensions: Control Loop Type, Loop Driver Strategy, Execution Isolation Model
- Finding: structural constraints (edit formats, tool discovery strategy) are the primary
  determinants of SLM success vs cascading failure

2.2 OpenDev -- Compound Workflow Routing (Bui, Mar 2026)
- Paper: "Building AI Coding Agents for the Terminal"
- arXiv: 2603.05344
- GitHub: opendev-to/opendev
- Rust-based CLI agent using compound AI system architecture
- Decomposes SWE lifecycle into slots: Execution, Thinking, Compaction, Critique, Vision
- Multi-model routing: frontier model for Thinking, local 3B for Execution
- "Lazy tool discovery" via MCP: only injects tool schemas on-demand to minimize context bloat
- Reduces baseline token cost of external integrations to near zero

2.3 ORCHESTRA -- Microservice Orchestration (Bellver-Soler et al., Feb 2026)
- Paper: "ORCHESTRA: AI-Driven Microservices Architecture"
- Proceedings of 16th International Workshop on Spoken Dialogue Systems Technology
- Sequential service execution under strict system control
- Event-driven streaming (Apache Kafka), containerized execution
- Circuit breakers for hallucinated responses, exponential backoff

2.4 ToM-SWE -- Dual-Agent Cognitive Separation (Zhou et al., Oct 2025)
- Paper: "ToM-SWE: User Mental Modeling For Software Engineering Agents"
- arXiv: 2510.21903
- GitHub: OpenHands/ToM-SWE
- Primary agent: technical execution only (edits, bash, stack traces)
- Theory-of-Mind partner agent: maintains user preferences via persistent vector memory
- Result: 59.7% task success on stateful coding tasks vs 18.1% for monolithic agents

2.5 Code-A1 -- Adversarial Orchestration (Wang et al., Mar 2026)
- Paper: "Code-A1: Adversarial Evolving of Code LLM and Test LLM via RL"
- arXiv: 2603.15611
- GitHub: ZJU-REAL/Code-A1
- Separate Code LLM and Test LLM in adversarial loop managed by system
- Test LLM gets white-box access to Code LLM output, generates adversarial tests
- "Mistake Book" mechanism: externalized experience replay buffer
- Safe to use with Qwen2.5-Coder-3B without self-congratulatory collapse


3. EXTERNALIZED COGNITION / MEMORY
====================================

3.1 E-mem -- Episodic Context Reconstruction (Wang et al., Jan 2026)
- Paper: "E-mem: Multi-agent based Episodic Context Reconstruction for LLM Agent Memory"
- arXiv: 2601.21714
- Stores raw, uncompressed historical episodes in external DB
- Spawns stateless assistant agents, each receiving one pristine episodic segment
- Assistants extract localized evidence, master agent aggregates
- O(1) streaming update mechanism with overlap regions
- Results: +7.75% F1 over graph memory systems, -70% total token consumption

3.2 Latent Context Compilation (Li et al., Feb 2026)
- Paper: "Latent Context Compilation: Distilling Long Context into Compact Portable Memory"
- arXiv: 2602.21221
- GitHub: SakanaAI/repo
- Compiles massive contexts into "Buffer Tokens" via disposable LoRA module
- Plug-and-play compatible with frozen base models (Llama-3.1-8B)
- Self-aligned optimization, no synthetic QA pairs needed
- 16x to 32x compression ratio with fine-grained reasoning preserved

3.3 SWE-Next -- Externalized Execution Environment (Liang et al., Mar 2026)
- Paper: "SWE-Next: Scalable Real-World Software Engineering Tasks for Agents"
- arXiv: 2603.20691
- GitHub: TIGER-AI-Lab/SWE-Next
- "Repo-quarter profiles": shared Docker images with copy-on-start workspaces
- Zero-latency reset on corruption -- just remount snapshot
- Processed 102K candidate commit pairs using only 639 GB storage

3.4 SharedLLM -- Multi-Scale Context Compression (Mar 2026)
- Paper: "Stacked from One: Multi-Scale Self-Injection for Context Window Extension"
- arXiv: 2603.04759
- GitHub: Clement25/SharedLLM
- Two stacked short-context LLMs from same base: compressor + decoder
- Dynamic tree-based compression, "self-injection" at lowest layers
- 8K native model generalizes to 128K+ inputs
- 2x speedup over streaming, 3x over encoder-decoder architectures

3.5 AgentRR -- Record & Replay Error History (May 2025)
- Paper: "Get Experience from Practice: LLM Agents with Record & Replay"
- arXiv: 2505.17716
- Records execution traces, replays only localized failure span IDs
- "Bounded intelligence" -- constrains within safe prior experiences


4. ERROR RECOVERY PIPELINE DESIGN
===================================

4.1 Automated Diagnosis via Static Analysis

SVRepair (Feb 2026)
- arXiv: 2602.06090
- GitHub: codefuse-ai/CodeFuse-SVR
- Transforms Control-Flow Graphs into Semantic Scene Graphs (SSGs)
- Serializes via Mermaid syntax for LLM consumption
- Iterative visual-artifact segmentation narrows to bug-centered region
- Results: 36.47% SWE-Bench M, 38.02% MMCode, 95.12% CodeVision

ReCode (Sep 2025)
- arXiv: 2509.02330
- Algorithm-aware retrieval: AST analysis predicts algorithmic intent
- Restricts vector search to matching algorithm structure
- Dual-encoder processes code and text separately for precise matching

Key principle: DECOUPLE fault localization from patch generation.
Localization = deterministic tools (AST, CFG, compiler). Patch = LLM with tightly gathered context.
Target prompt: "Function X on line Y failed with Z. Variable V resolves to W. Update line Y to..."

4.2 Micro-Reflection (NOT Multi-Paragraph)

ARLCP (Feb 2026, ICLR 2026)
- arXiv: 2602.12113
- PPO with dynamic penalty on reflection length conditioned on complexity
- Qwen3-1.7B: -38.19% length, +3.00% accuracy
- DeepSeek-R1-Distill-8B: -34.01% length, +1.99% accuracy
- Conclusion: long reflection ACTIVELY HARMS sub-10B models

"First Try Matters" (Oct 2025)
- arXiv: 2510.08308
- Small models suffer "overused checking" -- overturn correct answers during deep verification
- "Lost in the Middle" positional bias compounds with reflection length

Optimal pattern: single-sentence JSON micro-reflection
  {
    "root_cause_single_sentence": "...",
    "faulty_line_number": N,
    "required_action_single_sentence": "..."
  }

4.3 Reflection-as-a-Service (RaaS) Escalation Tiers
- Tier 1: LLM generates code
- Tier 2: Static failure -> AST/linter fixes deterministically (no LLM)
- Tier 3: Logic failure -> LLM does single-sentence JSON classification only
- Tier 4: Cyclic loop detected (Levenshtein distance of consecutive patches) -> escalate

4.4 Agentless Iteration (Kimi-Dev, Sep-Dec 2025)
- arXiv: 2509.23045
- GitHub: Kimi-Dev
- 60.4% on SWE-bench Verified (best workflow approach)
- Key: completely wipe context between attempts, never show model its own failed CoT
- Separate BugFixer and TestWriter modules
- 5k SFT trajectories adapts to SWE-Agent level (on par with Claude 3.5 Sonnet)

4.5 FixAudit -- Optimal Retry Thresholds (Apr 2026)
- arXiv: 2604.05560
- Single shared 7B model in Auditor + Fixer modes via RL
- Temperature 1.0, top-p 1.0 (encourages path exploration, prevents cyclic traps)
- Max 5 iterations (20 LLM invocations)
- Results: 7B zero-shot 11.93% -> self-repair 13.24% -> FixAudit 30.40%
- Empirically optimal escalation: 3-5 attempts, then deterministic fallback

4.6 Error Memory Architecture

CTIM-Rover (Jul 2025, REALM 2025)
- GitHub: Liqs-v2/ctim-rover
- WARNING: episodic natural language memory DEGRADES performance
- Baseline 42% dropped to 31-40% with memory injection
- Cause: irrelevant variable names, divergent logic paths poison small model attention

Optimal approach (from GISclaw arXiv:2603.26845 and OpenSage arXiv:2602.18691):
- External JSON state machine, NOT in-context history
- Only inject active_constraints and forbidden_diffs into prompt (<150 tokens)
- Compress redundant constraints when nearing escalation threshold
- Store rich history in SQLite/Neo4j, model sees only compressed directives


5. QUALITY AMPLIFICATION TECHNIQUES
=====================================

5.1 Guided by Gut (GG) -- Intrinsic Verification (May 2025)
- arXiv: 2505.20325
- GitHub: Amirhosein-gh98/Guided-by-Gut
- Replaces external PRM with intrinsic signals: token confidence + generation novelty
- GRPO fine-tuning calibrates confidence estimates
- Sub-10B models match 32B-70B accuracy
- 10x less GPU memory than larger models, 4-5x less than PRM methods
- 50% KV cache reduction vs Best-of-N, 8x faster inference
- Critical for 8-12GB VRAM environments

5.2 ReTool -- RL for Strategic Tool Use (Apr 2025)
- arXiv: 2504.11536
- GitHub: ReTool-RL/ReTool
- Teaches model WHEN to write code, WHEN to execute, HOW to read output
- Two-stage: cold start SFT + RL with interleaved execution rollouts
- 3B baseline (Qwen2.5-3B-Instruct): ~11% accuracy gain, outperforms standard 7B

5.3 AB-MCTS -- Adaptive Branching Tree Search (Mar 2025)
- arXiv: 2503.04412
- GitHub: SakanaAI/treequest
- Dynamic "go wider" vs "go deeper" branching based on external feedback
- Outperforms standard repeated sampling and traditional MCTS
- For 7B: branch locally at failure point instead of regenerating full solution
- Atom of Thoughts (Feb 2025) confirms verification-based TTS scales asymptotically
  better than sampling for heterogeneous code generation

5.4 Computational Arbitrage (Mar 2026)
- arXiv: 2603.22404
- mini-coder 4B achieved exact same 75% solve rate as frontier models on SWE-bench
- Higher local token budget but orders of magnitude cheaper per token
- $80 vs $150 to hit 75% success rate
- Running 7B through 15 AB-MCTS cycles locally beats single cloud zero-shot

5.5 Agentic Mid-Training

SWE-Universe (Feb 2026)
- arXiv: 2602.02361
- 800K+ real GitHub PRs as verifiable environments
- Async RL without loss masking -> "coding world model"

Youtu-LLM (Dec 2025)
- arXiv: 2512.24618
- GitHub: TencentCloudADP/youtu-tip
- 1.96B params, Multi-Latent Attention, 128K native context
- Progressive curriculum: commonsense -> STEM -> agentic tool-use


6. CONTEXT ENGINEERING
=======================

6.1 Prompt Guidelines (Jan 2026)
- arXiv: 2601.13118
- 1,140 Python tasks from BigCodeBench analyzed
- Assertive language ("must" not "should") -- 9% of optimized prompts
- Explicit pre/post conditions -- 7%/23%
- Strict I/O formatting -- 44%
- Variable name consistency (no pronouns) -- 3%

6.2 Context Composition
- No universal optimal ratio exists (arXiv: 2507.13334, Jul 2025)
- Dynamic compression preferred over static token ratios
- SharedLLM (see 3.4) solves this for local deployments

6.3 Scoring Verifiers (Feb 2025)
- arXiv: 2502.13820
- GitHub: aleksficek/Scoring-Verifiers
- 8B model as verifier via synthetic test generation
- Caveat: two 7B models (generator+verifier) in NF4 still strains 8-24GB VRAM


7. BENCHMARK DATA
==================

7.1 ClawsBench -- Scaffold Multiplier (Li et al., Apr 2026)
- arXiv: 2604.05172
- GitHub: benchflow-ai/ClawsBench
- 7,224 trials across models

                        Unscaffolded    Scaffolded    Unsafe Rate
Claude Opus 4.6         0-8%            63%           23%
Gemini 3.1 Pro          8%              58%           10%
Claude Sonnet 4.6       0%              56%           13%
GLM-5.1.1               0-8%            56%           25%
GPT-5.4                 0-8%            53%           7%
Gemini 3.1 Flash-Lite   0-8%            39%           22%

Top 5 models cluster within 10pp band when scaffolded. Scaffold dwarfs model variance.

7.2 SWE-Star -- Local SLM Benchmarks (LogicStar AI, Feb 2026)
- GitHub: logic-star-ai/swe-star
- Qwen2.5-Coder architecture, distilled from Devstral-2-Small
- 250K agentic trajectories

                    Iterative/3 Attempts    Pass@1
SWE-Star-32B        57.1%                   52.4%
SWE-Star-14B        52.8%                   49.8%
SWE-Star-7B         36.4%                   32.8%

7.3 SWE-bench Verified -- Frontier Comparison

Model/System            Active Params       Scaffold              Score
Claude Opus 4.6         Proprietary         Claude Code           80.8%
Gemini 3.1 Pro          Proprietary         Standard              80.6%
MiniMax M2.5            Open (MoE)          Agentic Workspace     80.2%
Qwen3-Max-Thinking      Large               SWE-Universe RL       75.3%
Qwen3-Coder-Next        3B active (80B MoE) SWE-Agent             70.0%+

Qwen3-Coder-Next (3B active) on SWE-Bench Pro: 44.3%
  vs Claude Sonnet 4.5: 46.1%
  vs GPT-5.2-Codex: 42.8%
-> 3B active params achieves 85-95% of frontier capability

mini-coder 4B: 75% solve rate matching DeepSeek v3.2, GPT-5 mini, Qwen Coder 480B

7.4 SWE-EVO -- Long-Horizon Evolution (Pham et al., Dec 2025)
- arXiv: 2512.18470
- GitHub: SWE-EVO/SWE-EVO
- Multi-step modifications spanning avg 21 files, 874 tests/instance
- GPT-5.4 scaffolded by OpenHands: only 25% (vs 72.8% on single-issue SWE-bench)
- Clear scaling trend: larger models reliably outperform smaller on long-horizon

7.5 CodeClash -- Goal-Oriented SWE (Yang et al., Nov 2025)
- arXiv: 2511.00839
- GitHub: CodeClash-ai/CodeClash
- Multi-round tournaments, no unit tests, open-ended goals

Elo Ratings:
1. Claude Sonnet 4.5      1385
2. GPT-5                   1366
3. o3                      1343
4. Claude Sonnet 4         1224
5. GPT-5 Mini              1199
6. Gemini 2.5 Pro          1124
7. Grok Code Fast          1006
8. Qwen3 Coder              952

Strategic planning + validation habits drive divergence, not syntax capability.
Top models validate in majority of rounds; small models rarely validate without prompting.


8. HYBRID LOCAL+CLOUD ARCHITECTURE
====================================

8.1 Spec-Driven Development (SDD)
- Generate spec.md, plan.md, tasks.md via cloud frontier model (cheap, few tokens)
- Route 15-40 LOC micro-tasks to local 7B for implementation
- Plan persists in filesystem, survives context compression
- Models: Plan.MD, RustySpec pipelines

8.2 Routing Strategy

Task                    Layer           Model/Hardware
Micro-task impl         Local           8-24GB VRAM, Qwen2.5-Coder 7B
Linting/error loops     Local           AB-MCTS scaffolding (5-15 iterations)
Code review             Fast cloud      Gemini 3.1 Flash / DeepSeek V3.2
Architecture planning   Cloud frontier  Claude Opus 4.6 / GPT-5.4
Cryptic bug diagnosis   Cloud frontier  Claude Sonnet 4.6 / o1

~95% compute volume handled locally. Cloud = "staff architect" for high-value decisions.
API spend reduction: >90% vs full cloud.


9. KEY DESIGN PRINCIPLES FOR LOCAL PIPELINES
==============================================

1. Stateless invocation: wipe context between attempts, never show model its own failed CoT
2. Micro-task decomposition: one function, one edit, one command per LLM call
3. Deterministic fault localization: AST/CFG/linter before LLM sees anything
4. Single-sentence reflection: JSON-constrained, never multi-paragraph
5. 3-5 retry limit: beyond that, escalate deterministically
6. No episodic memory: structured JSON constraints only, <150 tokens injected
7. Forbidden diffs: string-match block against known-failed patches
8. Structural context fidelity: raw code > summarized code > embedded code
9. Intrinsic verification over external PRM: GG-style confidence signals save VRAM
10. Agentic mid-training: select models pre-trained on tool-use trajectories
11. Temperature 1.0 + top-p 1.0 for repair loops (prevents cyclic traps)
12. Tree search over flat sampling: AB-MCTS branches at failure point, not from scratch


10. PAPER INDEX (all citations)
================================

arXiv:2604.03515 - Scaffold taxonomy (Rombaut, Apr 2026)
arXiv:2603.05344 - OpenDev terminal agent (Bui, Mar 2026) -- opendev-to/opendev
arXiv:2510.21903 - ToM-SWE dual agent (Zhou et al., Oct 2025) -- OpenHands/ToM-SWE
arXiv:2603.15611 - Code-A1 adversarial (Wang et al., Mar 2026) -- ZJU-REAL/Code-A1
arXiv:2601.21714 - E-mem episodic memory (Wang et al., Jan 2026)
arXiv:2602.21221 - Latent Context Compilation (Li et al., Feb 2026) -- SakanaAI/repo
arXiv:2603.20691 - SWE-Next environments (Liang et al., Mar 2026) -- TIGER-AI-Lab/SWE-Next
arXiv:2604.05172 - ClawsBench (Li et al., Apr 2026) -- benchflow-ai/ClawsBench
arXiv:2512.18470 - SWE-EVO long-horizon (Pham et al., Dec 2025) -- SWE-EVO/SWE-EVO
arXiv:2511.00839 - CodeClash goal-oriented (Yang et al., Nov 2025) -- CodeClash-ai/CodeClash
arXiv:2602.06090 - SVRepair visual reasoning (Feb 2026) -- codefuse-ai/CodeFuse-SVR
arXiv:2509.02330 - ReCode retrieval-augmented repair (Sep 2025)
arXiv:2602.12113 - ARLCP reflection penalty (Feb 2026, ICLR 2026)
arXiv:2510.08308 - First Try Matters / self-verification (Oct 2025)
arXiv:2509.23045 - Kimi-Dev agentless (Sep-Dec 2025) -- Kimi-Dev
arXiv:2604.05560 - FixAudit iterative repair (Apr 2026)
arXiv:2603.26845 - GISclaw error memory (Mar 2026)
arXiv:2602.16891 - OpenSage self-programming (Feb 2026) -- opensage-agent/opensage
arXiv:2505.20325 - Guided by Gut intrinsic TTS (May 2025) -- Amirhosein-gh98/Guided-by-Gut
arXiv:2504.11536 - ReTool RL tool use (Apr 2025) -- ReTool-RL/ReTool
arXiv:2503.04412 - AB-MCTS adaptive branching (Mar 2025) -- SakanaAI/treequest
arXiv:2603.22404 - Computational Arbitrage (Mar 2026)
arXiv:2602.02361 - SWE-Universe mid-training (Feb 2026)
arXiv:2512.24618 - Youtu-LLM 1.96B (Dec 2025) -- TencentCloudADP/youtu-tip
arXiv:2601.13118 - Prompt guidelines for code gen (Jan 2026)
arXiv:2507.13334 - Context engineering survey (Jul 2025)
arXiv:2502.13820 - Scoring Verifiers (Feb 2025) -- aleksficek/Scoring-Verifiers
arXiv:2603.04759 - SharedLLM context extension (Mar 2026) -- Clement25/SharedLLM
arXiv:2505.17716 - AgentRR record & replay (May 2025)
logic-star-ai/swe-star - SWE-Star local models (Feb 2026)
Liqs-v2/ctim-rover - CTIM-Rover episodic memory failure (Jul 2025)

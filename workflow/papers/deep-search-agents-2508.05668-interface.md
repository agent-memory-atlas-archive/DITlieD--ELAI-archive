Algorithm: LLM-based Deep Search Agents (Survey)
ArXiv: 2508.05668
Paper: A Survey of LLM-based Deep Search Agents: Paradigm, Optimization, Evaluation, and Challenges
Domain: survey, information-retrieval, llm-agents, deep-search, multi-hop-retrieval
Date: 2025

=== FORMAL DEFINITION ===

Deep Search Agent: An LLM agent capable of comprehending user intentions and environment contexts,
autonomously planning search strategies, executing multi-turn dynamic retrieval from diverse sources,
and integrating information to provide comprehensive insights.

Distinguishing property from standard RAG:
- RAG: single-turn or rule-based iterative search with static integration
- Deep Search Agent: autonomous planning + multi-turn dynamic retrieval + iterative reflection

Core execution loop (formal):
  pi_0 = Plan(q, C)
  for t = 0 to T:
    pi_{t+1} = Reflect(o_t, h_t, pi_t)
    a_{t+1}  = Act(pi_{t+1})      // action: search or browse
    o_t      = new observation
  O = {o_1, o_2, ..., o_T}        // full observation set
  E = Select(q, O)                 // evidence selection
  y_hat_q = Generate(q, E)        // final answer generation

=== TAXONOMY: FOUR AXES ===

Axis 1 - HOW TO SEARCH (structural paradigms)
Axis 2 - HOW TO OPTIMIZE (tuning-free vs tuning-based)
Axis 3 - HOW TO APPLY (external vs internal applications)
Axis 4 - HOW TO EVALUATE (datasets and metrics)

=== AXIS 1: SEARCH PARADIGMS ===

--- PARALLEL STRUCTURES ---

Decomposition-based | Breaks original intent into sub-queries executed simultaneously
  Variants:
    prompt-based decomposition (Khattab et al. 2022)
    knowledge-graph-guided decomposition (Wang et al. 2024a)
    fine-tuned LM planning for decomposition (Li et al. 2023)
  Mechanism: sub-queries run in parallel; results merged before synthesis

Diversification-based | Generates multiple query interpretations searched in parallel
  Variants:
    beam search rewrites (Kostric & Balog 2024)
    prompt ensembling (Dhole & Agichtein 2024)
    pseudo-answer generation / HyDE-style (Li et al. 2024c)
  Mechanism: diverse query variants explored simultaneously; ensemble aggregation

--- SEQUENTIAL STRUCTURES ---

Reflection-Driven | Loop: search -> generate -> reflect on quality -> continue/terminate
  Representatives: Hayashi et al. 2025, Xiao et al. 2025
  Termination: reflection decides answer sufficiency
  Risk: can loop indefinitely without hard stop on T

Proactivity-Driven | Agent decides WHEN and WHAT to search based on context
  Variants:
    prompt-based proactivity (Li et al. 2025c)
    learned proactivity from expert trajectories (Asai et al. 2023, Islam et al. 2024)
  Mechanism: search trigger is conditional, not every step

--- HYBRID STRUCTURES ---

Tree-based | Parallel expansion from nodes; aggregation via voting or reward model
  Representatives: MCTS variants (Trinh et al. 2025a, Feng et al. 2025)
  Components: expand nodes in parallel, score branches, prune/select

Graph-based | DAG decomposition with dynamic expansion and shrinkage; supports backtracking
  Representatives: Li et al. 2024a, Chen et al. 2024
  Advantage: can revise earlier decisions; arbitrary inter-node connections
  Cost: higher coordination overhead

Empirical finding: hybrid structures are preferred in tuning-free settings to cover more
search paths. Fine-tuned models internalize flexibility within sequential structures instead.

=== AXIS 2: OPTIMIZATION TECHNIQUES ===

--- TUNING-FREE ---

Single-Agent Iterative:
  Iterative refinement loops, reflection chains (Wang et al. 2023, Zhang et al. 2024c)
  Prompt-engineered plan-reflect-act loops
  No gradient updates; relies on LLM's in-context capabilities

Multi-Agent Specialized Roles:
  Roles: planner, searcher, generator, browser, evaluator, memory
  Coordination: fixed execution order OR supervisor-controlled (Jiang et al. 2025d, Wu et al. 2025d)
  Limitation: prompt engineering burden; hard to control complex multi-step processes from a single prompt

Test-Time Scaling:
  Reasoning scaling: self-consistency, best-of-N, MCTS
  Search scaling: increased search interaction budget (Xi et al. 2025a)
  Combined: simultaneous reasoning depth + search breadth scaling
  Empirical finding: performance scales smoothly with max permissible search action count

--- TUNING-BASED: SFT ---

Data sources (in order of preference):
  1. Distillation from powerful teacher models (Asai et al. 2023, Lee et al. 2025)
  2. Self-improvement via rejection sampling from own trajectories
  3. RL warm-up trajectories

Data filtering criteria (Li et al. 2025d):
  - Correctness: LLM judge vs ground truth
  - Diversity: avoid trajectory homogeneity
  - Informativeness: evidence coverage
  - Efficiency: step count / token budget

Data blending: mix search-trajectory data with general-purpose instruction data (Lee et al. 2025)

SFT pipeline:
  1. Generate trajectories via LLM-environment interaction
  2. Rejection sampling: keep trajectories meeting filter criteria
  3. Fine-tune student model on filtered set
  4. Optionally iterate (self-improvement loop)

--- TUNING-BASED: RL ---

Algorithms used (no consensus on best):
  PPO (Proximal Policy Optimization) - Schulman et al. 2017
  GRPO - Shao et al. 2024b
  Reinforce++

Reward function composition (multi-objective):
  - Answer correctness (nearly ubiquitous; rule-based or ORM)
  - Efficiency: penalties on step count / search action count
  - Diversity: reward for varied evidence sources
  - Evidence quality: retrieval gain measurement
  - Redundancy penalty: penalize repeated retrieval of same content
  - Length penalty: penalize unnecessarily long responses
  - Format adherence: structural correctness

Reward implementation types:
  Rule-based verification: EM/F1 against ground truth (closed-ended)
  ORM (Outcome-based Reward Model): learned model scoring final answer
  PRM (Process-based Reward Model): learned model scoring each reasoning step

Contrastive learning variant: identifies optimal retrieval trigger moments (Zhang et al. 2025g)
Pre-training variant: Retrieval-Augmented Mask Prediction (Wu et al. 2025f)

--- TUNING-BASED: MIXED ---

SFT -> RL pipeline: SFT warm-up then RL fine-tuning (Zhang et al. 2025g, Li et al. 2025d)
Iterative SFT-RL: alternating loops with trajectory refinement (Zhang et al. 2025a)

=== AXIS 3: NAMED SYSTEMS ===

Commercial (details undisclosed):
  OpenAI Deep Research
  Google Gemini Deep Research
  Perplexity Deep Research
  Grok 3

Open-source frameworks:
  Jina AI node-DeepResearch
  LangChain Open Deep Research
  GPT-Researcher
  ManuSearch

Academic systems:
  Search-o1       - integration of o1-style reasoning with search
  WebThinker      - web-based reasoning agent
  WebSailor       - web navigation for research
  Search-R1       - RL-trained search policy

=== AXIS 4: EVALUATION ===

--- BENCHMARKS: CLOSED-ENDED ---

Multi-hop QA:
  HotpotQA (Yang et al. 2018)
  MuSiQue (Ho et al. 2020)
  2WikiMultiHopQA (Press et al. 2022)
  IIRC
  Natural Questions (NQ)
  TriviaQA

Fact-checking:
  FEVER (Wei et al. 2024)
  HoVER / HOVER (Jiang et al. 2020)
  FEVEROUS (Aly et al. 2021)

Domain-specific:
  Finance (Li et al. 2024a)
  Medicine / medical QA (Pal et al. 2022)
  Mathematics, Physics, Code

--- BENCHMARKS: OPEN-ENDED ---

Deep Research Bench
DeepResearchGym
Multi-perspective queries requiring expert-level research
Multi-modal variants (emerging)

--- METRICS ---

Closed-ended:
  Exact Match (EM)
  F1 score
  Accuracy
  Pass@k
  NDCG (retrieval quality)
  MAP (retrieval quality)
  Retrieval Precision / Recall

Open-ended:
  Key point coverage
  Informativeness
  Breadth and depth scores
  Coherence
  Citation accuracy
  Win Rate (Arena-based pairwise comparison)

--- JUDGE TYPES ---

Rule-based: EM, F1 vs ground truth
LLM-as-Judge: semantic correctness assessment
Agent-as-Judge: process + output evaluation (covers reasoning quality)
Human evaluation: limited use due to high cost

=== AGENT COMPONENT MODEL ===

Mandatory components (every deep search agent):
  Plan(q, C)              - initial strategy from query and context
  Reflect(o_t, h_t, pi_t) - update plan given new observation and history
  Act(pi_{t+1})           - execute search or browse action
  Select(q, O)            - extract most relevant evidence from observations
  Generate(q, E)          - produce final answer from evidence

Optional components (multi-agent setups):
  Planner agent      - top-level control, termination decisions
  Search agent       - evidence gathering subagent
  Generation agent   - synthesis subagent
  Browser agent      - JS-capable web navigation
  Evaluator agent    - output quality assessment
  Memory agent       - long-term state across sessions

State representation:
  h_t: trajectory history (sequence of prior actions and observations)
  pi_t: current plan
  O = {o_1..o_T}: complete observation history

=== QUERY REFORMULATION TECHNIQUES ===

Prompt-based: direct prompting of strong LLMs to rephrase/decompose query
Knowledge-graph-guided: use KG structure to guide decomposition (Wang et al. 2024a)
Fine-tuned LM planners: smaller LMs trained specifically for retrieval planning (Li et al. 2023)
Beam search rewrites: generate N candidate queries, score, select (Kostric & Balog 2024)
Pseudo-answer generation: generate hypothetical answer, embed, retrieve (Li et al. 2024c)
Prompt ensembling: multiple prompt templates -> multiple query variants (Dhole & Agichtein 2024)

=== MULTI-HOP RETRIEVAL PATTERNS ===

Pattern 1 - Sequential dependency tracking:
  Each hop's result informs next query; trajectory history h_t maintains chain
  Agent cannot skip: each observation feeds Reflect() before next Act()

Pattern 2 - Context-aware reformulation:
  After hop t: query_{t+1} = Reformulate(q_original, o_1..o_t)
  Prior results explicitly condition new queries

Pattern 3 - Evidence aggregation:
  All hop observations O stored; Select(q, O) filters for final evidence set E
  Generation sees full evidence set, not just last hop

Pattern 4 - Validation loops:
  After generation, Reflect() evaluates answer quality
  Low-quality triggers additional search hops before termination

Stopping criteria (not formally standardized):
  Hard limit: max iteration T (implementation-defined, not specified in paper)
  Soft limit: Reflect() returns "sufficient" signal
  No universal stopping rule specified in survey

=== TOKEN EFFICIENCY ===

Explicit token budget management: not standardized; survey does not specify defaults

Efficiency-related techniques found:
  RL reward penalizing step count (encourages fewer hops per task)
  RL reward penalizing response length (encourages concise generation)
  Redundancy penalty (discourages re-fetching same content)
  SFT data filtering on efficiency criterion (Li et al. 2025d)
  Test-time scaling trades tokens for performance (explicit tradeoff acknowledged)

Context window management: not addressed in survey; treated as implementation detail

=== SMALL / LOCAL MODEL DEPLOYMENT ===

Coverage in paper: limited; primarily academic framing

Techniques that enable smaller models:
  Knowledge distillation: teacher (large, capable) -> student (smaller, fine-tuned)
    Data: elaborate teacher prompts generate trajectories transferred to student
    Representatives: Asai et al. 2023, Lee et al. 2025
  Fine-tuned smaller LMs for retrieval planning specifically (Li et al. 2023, Joshi et al. 2024)
  SFT on filtered trajectories allows smaller models to internalize agent behavior

No explicit benchmark comparing small vs large model search agent performance cited.
Survey gap: deployment constraints (VRAM, latency, batching) not covered.

=== FAILURE MODES (CITED BY AUTHORS) ===

Imperfect retrieval: noise, biases, false information in retrieved results; agents lack
  robust fact-checking; conflicting sources inadequately reconciled

Sparse rewards: open-ended queries lack clear correctness signals; reward design is hard
  for subjective, multi-faceted research outputs

Multimodal gap: current systems are text-centric; image/video/audio support nascent;
  most benchmarks text-only

Self-evaluation limits: agents cannot verify correctness beyond closed-ended problems
  with ground truth; open-ended self-assessment unreliable

RL infrastructure cost: sampling for RL training is computationally intensive;
  approximate retrieval systems needed at scale

Alignment gap: academic research vs commercial implementations diverge; commercial
  systems do not disclose technical details (OpenAI, Google, Perplexity)

Single-prompt planning limits: hard for a single prompt-driven agent to control every
  aspect of a complex multi-step process dynamically

=== OPEN CHALLENGES (PAPER SECTION) ===

1. Broaden and fuse information sources
   - Integrate private/proprietary data alongside public web
   - Reconcile conflicting information across heterogeneous formats

2. Imperfect retrieval
   - Enhance information verification and fact-checking
   - Critical evaluation of noisy/biased retrieved content

3. From text to multi-modality
   - Extend from text-centric to multimodal search
   - Handle images, audio, video in retrieval and generation

4. Customized reinforcement learning
   - Algorithms optimized for long-horizon planning
   - Handle sparse rewards and open-ended scenarios
   - No current consensus on best RL algorithm for search agents

5. Robust infrastructure
   - Optimize RL sampling efficiency
   - Implement high-recall retrieval at scale
   - Priority-aware scheduling for multi-agent pipelines

6. Search agent self-evolution
   - Autonomous learning and adaptation without human intervention
   - Continuous improvement from deployment feedback

=== DESIGN TRADEOFFS ===

Tuning-free vs tuning-based:
  Flexibility:  tuning-free higher (dynamic query adaptation at runtime)
  Efficiency:   tuning-based higher (internalized behavior, fewer LLM calls)
  Overhead:     tuning-free = prompt engineering; tuning-based = training cost
  Structure:    tuning-free prefers hybrid; tuning-based prefers sequential

Parallel vs sequential search:
  Parallel: higher coverage, higher cost, requires merging/deduplication
  Sequential: lower cost, adaptive, risks early termination with insufficient evidence

Single-agent vs multi-agent:
  Single-agent: end-to-end RL possible; less modular
  Multi-agent: parallelizable; harder to optimize end-to-end via RL

=== IMPLEMENTATION PITFALLS (DO NOT DO THIS) ===

Do not use single-turn RAG and call it a search agent: missing reflection loop and
  multi-turn planning are definitional requirements

Do not omit the Reflect() step: without it, subsequent queries are not conditioned on
  prior observations; violates core loop invariant

Do not use only correctness reward in RL: efficiency, diversity, and redundancy penalties
  are necessary; correctness-only RL over-generates and over-searches

Do not conflate open-ended and closed-ended evaluation: EM/F1 cannot assess research
  report quality; different judge types required per task type

Do not assume small models can directly replace large teacher models without distillation:
  planning and reflection behaviors require explicit training transfer

Do not skip stopping criteria definition: unbounded T leads to infinite loops on
  reflection-driven agents when answer quality threshold is never met

Do not use static retrieval (fixed queries) in a system claiming agent behavior:
  dynamic query reformulation conditioned on prior observations is required

=== CRITICAL INVARIANTS (TESTABLE) ===

assert len(observations) == len(actions), "each action must produce exactly one observation"

assert evidence_set.issubset(observation_set), "E = Select(q, O) must only contain items from O"

assert final_answer is not None, "Generate(q, E) must always produce output; no empty returns"

assert iteration_count <= T_max, "search loop must terminate at hard iteration limit T"

assert all(o is not None for o in observations), "every search action must yield an observation (even empty-result is an observation)"

assert reward_components.get('correctness') is not None, "correctness is the ubiquitous reward component; must be present in any RL-trained agent reward"

=== RELATED INTERFACE MAPS ===

See also: deep-research-agents-2506.18096-interface.md (covers 2506.18096 Roadmap paper)
  That paper: implementation taxonomy, browser vs API retrieval, planning strategy comparison
  This paper (2508.05668): optimization techniques, RL algorithms, SFT pipelines, evaluation metrics

Relationship: 2508.05668 provides deeper coverage of optimization and evaluation;
2506.18096 provides deeper coverage of system architectures and deployment patterns.

Algorithm: Deep Research Pipeline for Autonomous Research Agents
ArXiv: 2508.12752
Paper: Deep Research: A Survey of Autonomous Research Agents
Authors: Wenlin Zhang, Xiaopeng Li, Yingyi Zhang, Pengyue Jia, Yichao Wang, Huifeng Guo, Yong Liu, Xiangyu Zhao
Affiliation: City University of Hong Kong + Huawei Noah's Ark Lab
Published: 2025 (LaTeXML timestamp Mon Aug 18 09:24:34 2025)


PIPELINE OVERVIEW

The deep research pipeline has four sequential stages. Each stage has a formal definition with typed inputs and outputs.

Stage 1: Planning
Input: q0 (research question, string) + K (agent context, structured knowledge/history)
Output: P = [s1, s2, ..., sn] (ordered sequence of subgoals or tool-invocation steps)
Formal: P = M_plan(q0, K; theta)   [Eq. 1, Section 2]
Purpose: Decompose high-level research question into structured sub-goals BEFORE any retrieval begins.

Stage 2: Question Developing
Input: P (plan from stage 1), si (current subgoal), E (accumulated evidence from prior queries)
Output: Q_i = {q_i1, q_i2, ...} (set of search queries for subgoal si)
Formal: Q_i = M_ask(P, si, E; theta)   [Eq. 2, Section 3]
Purpose: Convert each subgoal into targeted, contextualized retrieval queries.

Stage 3: Web Exploration
Input: Q_i (queries from stage 2), R (web retriever), H (open-web corpus)
Output: D (set of retrieved documents)
Formal: D = M_web(R, Q_i, H; theta)   [Eq. 3, Section 4]
Purpose: Retrieve relevant, trustworthy information from external sources.

Stage 4: Report Generation
Input: q0, P, Q (all queries), D (all retrieved documents)
Output: Y (final structured report)
Formal: Y = M_theta(q0, P, Q, D)   [Eq. 4, Section 5]
Purpose: Synthesize retrieved evidence into coherent, factual, structured report.


STAGE 1 — PLANNING: DETAILED TAXONOMY

Two major paradigm categories (Table 1):

Category A: Planning with Structured World Knowledge
- Planning via simulation: Agent mentally rolls out candidate action trajectories before execution.
  Systems: WebDreamer, Simulate-Before-Act (Gu et al. 2025)
  Mechanism: LLM acts as implicit world model; explicit simulation phase evaluates feasibility prior to execution.
- Planning via modularity: Partition web exploration across specialized sub-agents coordinated by strategic planner.
  Systems: WebPilot (Zhang et al. 2025b), WKM (Qiao et al. 2024), Plan-and-Act (Erdogan et al. 2025)
  Mechanism: External knowledge graphs ground reasoning paths; iterative refinement for temporal coherence.
- Planning via adaptation: Efficiency-aware planning that balances deliberation depth vs resource usage.
  Systems: Thought of Search (Katz et al. 2024), MPO (Xiong et al. 2025)
  Mechanism: Meta-plan optimization (MPO) adaptively tunes planning strategies via meta-learning across environments.

Category B: Planning as a Learnable Process
- Planning via space exploration: Architecture search over configuration space for task-adapted pipelines.
  Systems: AgentSquare (Shang et al. 2024), Agent-E (Abuelsaad et al. 2024)
  Mechanism: Automatic assembly of planning pipelines; accessibility tree construction for compact representation.
- Planning via self-training: Hone planning heuristics from deployment experience without human intervention.
  Systems: Patel et al. 2024, InSTA (Trabucco et al. 2025)
  Mechanism: Internet-scale training pipelines for lifelong adaptability.
- Planning via preference modeling: Reasoning-enhanced planning with human-like search strategy simulation.
  Systems: MindSearch (Chen et al. 2024), SimpleDeepSearcher (Sun et al. 2025b), Search-in-the-Chain (Xu et al. 2024b), WEPO (Liu et al. 2025), MPO (Xiong et al. 2025)
  Mechanism: Multi-hop reasoning path synthesis; dynamic action updates from intermediate retrieval results.

Key failure modes for Planning (Section 2.3):
- Plans are brittle to ambiguous or underspecified research questions; hallucinated steps propagate downstream.
- Evaluation is coarse (end-task accuracy), making plan quality diagnosis hard.
- No cross-task knowledge transfer; each question treated in isolation.


STAGE 2 — QUESTION DEVELOPING: DETAILED TAXONOMY

Two major categories (Table 2, Section 3.1):

Category A: Reward-Optimized Methods

Sub-category A1: Rewards for format and accuracy (binary format + token F1 or exact match)
- DeepResearcher (Zheng et al. 2025): Binary format reward + token-level F1. Trains multi-agent via GRPO. Real search APIs.
- EvolveSearch (Zhang et al. 2025d): Same reward structure as DeepResearcher.
- R1-Searcher (Song et al. 2025): Staged training — Phase 1: format compliance + search usage reward. Phase 2: integrates F1-based answer quality.
- Search-R1 (Jin et al. 2025): Strict format templates (<think>, <search>, <information>, <answer>). Optimizes exact-match reward via PPO or GRPO.
- ZeroSearch (Sun et al. 2025a): Same principle but replaces real search API with learned search simulator. Enables curriculum rollout without real querying cost.
- MaskSearch (Wu et al. 2025b): Rewards based on masked span recovery via retrieval-augmented answer prediction. Combines format and accuracy into unified reward during pretraining.
- DeepRetrieval (Jiang et al. 2025): Reward defined over retrieval metrics: Recall@K or NDCG@K. Aligns query generation with retrieval performance.

Sub-category A2: Multi-dimensional reward
- InForage (Qian and Liu 2025): Outcome reward + information gain (coverage of ground-truth knowledge) + efficiency penalty (discourages redundant reasoning hops).
- OTC-PO (Wang et al. 2025b): Cost-aware reward; penalizes excessive tool usage while preserving correctness.
- IKEA (Huang et al. 2025c): Knowledge-boundary-aware reward. Positive for solving easy questions with internal knowledge alone; penalizes unnecessary external searches.
- AutoRefine (Shi et al. 2025): Evaluates completeness of intermediate refinements extracted from retrieved documents + final answer quality.
- R-Search (Zhao et al. 2025): Combines answer correctness + evidence quality + format compliance.
- MMSearch-R1 (Wu et al. 2025a): Extends R-Search across modalities with structured reasoning.
- VRAG-RL (Wang et al. 2025a): Visual search actions optimized via fine-grained rewards for image selection, visual attention consistency, and answer quality.

Category B: Supervision-Driven Methods

Sub-category B1: Multi-agent systems (no RL)
- ManuSearch (Huang et al. 2025b): Separate agents for subquestion planning, web search, HTML extraction. Deterministic/supervised rules.
- Search-o1 (Li et al. 2025a): Adaptive retrieval scheduling + efficient batch processing.
- SearchAgent-X (Yang et al. 2025b): System-level enhancements for throughput and responsiveness.

Sub-category B2: Supervision optimization (MCTS + DPO)
- ReasonRAG (Zhang et al. 2025a): MCTS explores reasoning trajectories; DPO ranks them. Fine-grained feedback on intermediate steps replaces sparse outcome-only supervision. Enables learning to interleave search and reasoning efficiently from fewer samples.

Key failure modes for Question Developing (Section 3.4):
- Generated queries degrade when subgoals are vague or ambiguous.
- Lack of contextual coherence — fails to integrate prior query history, produces redundant queries.
- Poor performance on open-ended problems: queries are too narrow, fail to capture full information space.


STAGE 3 — WEB EXPLORATION: DETAILED TAXONOMY

Two categories (Table 3, Section 4):

Category A: Web-Based (Browser/Crawl)
- Web scraping and crawling: Scrapy, BeautifulSoup. Static extraction.
- Browser-based autonomous web agents: WebGPT (Nakano et al. 2021) — text-based browser, HTML to structured representation, high-level commands (Find keyword). Selenium for DOM queries. Agent-E uses accessibility trees preserving semantic structure.
- Multimodal web agents: WebVoyager (He et al. 2024) — GPT-4V + screenshot analysis + HTML = 59% task success on real-world benchmarks, significantly outperforming text-only. MM-ReAct embeds screenshots into reasoning chains. WebArena provides evaluation framework.

Category B: API-Based Retrieval
- Industrial search engines: Bing (used by OpenAI DeepResearch), Google (Gemini DeepResearch), Perplexity Sonar API (BM25 + keyword + dense-vector reranking), Grok DeepSearch (own crawler + X posts).
- Domain-specific: Reportify (financial/market research), YanXueZhiDe/CNKI (academic, Chinese), DuckSearch, BraveSearch, Bocha (open-source ecosystem).

OpenAI DeepResearch web workflow (from Section 4.2):
  issue queries -> extract passages -> spawn follow-up queries -> rank candidate documents -> execute sandboxed code

Local deployment web exploration options (open-source):
  DuckDuckGo (via DuckSearch), BraveSearch API, Bocha AI API.
  Browser automation: Selenium, Playwright (Selenium-compatible).

Key failure modes for Web Exploration (Section 4.3):
- Web retrieval agents: resource-intensive, no real-time trustworthiness assessment.
- API-based: misses niche, dynamic, or unindexed content.
- Both approaches disconnected from upstream planning/question-formulation stages.
- Future: hybrid architectures needed. Multimodal processing + real-time verification.


STAGE 4 — REPORT GENERATION: DETAILED TAXONOMY

Two categories (Table 4, Section 5):

Category A: Structure Control
- Planning-based generation:
  Agent Laboratory: paragraph-level planning with structure-aware prompting.
  AI Scientist v2: recursive tree-structured planning for global layout consistency.
  LongWriter (Bai et al. 2024): high-level outline planning + section-level synthesis. Maintains coherence across thousands of tokens.
  LongDPO (Ping et al. 2025): critique-augmented supervision at each generation step, aligning local completions to global structure.
- Constraint-guided generation:
  WebThinker (Li et al. 2025b): section-aware decomposition bridging planning to content sections.
  Suri: instruction tuning under multiple layout/stylistic constraints (tone, structure, topic coverage).
- Structure-aware evaluation:
  Long2RAG: key-point recall metrics for retrieved evidence incorporation.
  ExPerT: layout-specific completeness and explainability evaluation.

Category B: Factual Integrity
- Faithful modeling:
  RAGSynth: synthetic supervision signals under known factual variations.
  BRIDGE: verification layer between retrieval and generation to assess factual adequacy.
  Context-aware decoding (Zhou et al. 2023b, Shi et al. 2024): prioritize high-confidence evidence spans, reduce hallucinated completions.
- Conflict resolution:
  FaithfulRAG: fact-level conflict modeling for alignment with consistent retrieved facts.
  DRAGged: detection + intervention models for inter-source conflicts.
  Entropy-based decoding (Yuan et al. 2024): adaptively adjusts to evidence uncertainty under ambiguous input.
- Factuality evaluation metrics:
  Face4RAG: attribution consistency in Chinese RAG systems.
  SFR-RAG: contextual attribution indicators at passage level.
  FaithJudge + RAG-QA Arena: evolving leaderboards for long-form factual evaluation.
  MT-RAIG: factuality for structured data (multi-table insight generation).

Key failure modes for Report Generation (Section 5.3):
- Most methods optimize isolated subskills; no joint optimization with upstream stages.
- Structure control relies on fixed outlines / static planning; no adaptation to evidence distribution.
- Few methods handle consistency across multi-document and multi-hop contexts.
- Local grounding does not prevent cross-document contradictions.


OPTIMIZATION (Section 6)

Workflow architectures:
1. Single-agent: one LLM handles entire pipeline. Task decomposition -> query generation -> retrieval -> synthesis. Typically end-to-end RL. Systems: DeepResearcher, WebThinker, Search-R1.
2. Multi-agent: specialized roles. Planner agent (decomposition + scheduling) + Query agent (diverse contextual queries) + Retriever agent (external tool interaction) + Writer agent (structured synthesis). Systems: AgentRxiv, AI Scientist, OpenResearcher.

Parameter optimization paradigms:
1. Contrastive learning: contrasts successful vs failed trajectories for tool usage. System: Avatar (Wu et al. 2024).
2. Reinforcement learning: reward signals based on retrieval accuracy + artifact quality. Systems: Search-O1, Learning-to-Search (Chen et al. 2025a).
   RL algorithms observed: GRPO (DeepResearcher, EvolveSearch, Search-R1), PPO (Search-R1), DPO (ReasonRAG via MCTS ranking).
3. Curriculum training: staged learning — fundamental skills before full workflow orchestration. Systems: AI Scientist-v2, SimpleDeepSearcher.

Optimization principle: single-agent -> RL for end-to-end reward alignment. Multi-agent -> modular training + agent-level feedback.


BENCHMARKS AND EVALUATION (Section 7 + Table 5)

All benchmarks evaluate subsets of P (Planning), QD (Question Developing), WE (Web Exploration), RG (Report Generation).

Search-oriented (cover P+QD+WE, NOT RG):
- Mind2Web 2 (Gou et al. 2025): web search | Success Rate, Partial Completion
- BrowseComp (Wei et al. 2025): web search | Accuracy, Calibration Error
- BrowseComp-Zh (Zhou et al. 2025): Chinese web search | Accuracy, Calibration Error
- WebArena (Zhou et al. 2023a): web search with simple planning | Success Rate
- GAIA (Mialon et al. 2023): multi-step assistant tasks | Exact Match (EM)
- Humanity's Last Exam (Phan et al. 2025): multidomain reasoning | Accuracy, Calibration Error
- MedBrowseComp (Chen et al. 2025b): medical web search | Accuracy
- GPQA (Rein et al. 2024): graduate-level QA | Accuracy
- InfoDeepSeek (Xi et al. 2025): open-domain QA, covers QD+WE only | Accuracy, Information Accuracy

Research-oriented (cover all 4 stages, P+QD+WE+RG):
- DeepResearch Bench (Du et al. 2025): research report generation | Pairwise Agreement Rate, Overall Pearson Correlation
- DeepResearchGym (Coelho et al. 2025): research task sandbox | KPR (knowledge precision/recall), KPC, Precision, Recall, Clarity, Insight

Primary metrics for full-pipeline evaluation:
- KPR/KPC: knowledge precision/recall — measures evidence coverage in final report
- Pairwise Agreement Rate: human rater agreement on report quality
- Pearson Correlation: correlation of model scores to human scores
- Clarity + Insight: qualitative dimensions from DeepResearchGym


CRITICAL INVARIANTS (testable assertions)

1. Pipeline ordering invariant (Section 1):
   assert stage_planning_complete before stage_question_developing_runs
   assert stage_question_developing_complete before stage_web_exploration_runs
   assert stage_web_exploration_complete before stage_report_generation_runs
   # Stages are sequential; later stages depend on earlier stage outputs

2. Plan non-empty invariant (Definition 2.1, Eq. 1):
   assert len(plan) >= 1  # P must contain at least one subgoal s_i
   assert all(isinstance(s, str) for s in plan)  # subgoals are string-representable

3. Question developing requires plan context (Definition 3.1, Eq. 2):
   assert queries_for_subgoal_i = f(plan, subgoal_i, accumulated_evidence)
   assert accumulated_evidence is not None  # E is initialized even if empty set initially

4. Evidence accumulation invariant (Section 3, Definition 3.1):
   assert len(accumulated_evidence) >= len(accumulated_evidence_before_retrieval)
   # E only grows; retrieved docs are appended not replaced

5. Report generation requires all upstream outputs (Definition 5.1, Eq. 4):
   assert q0 is not None  # original question required
   assert plan is not None  # P required
   assert len(queries) >= 1  # Q must be non-empty
   assert len(documents) >= 1  # D must be non-empty before synthesis


COMPLEXITY

Time:
- Planning: O(L) where L = number of subgoals generated; single LLM forward pass + decoding.
- Question Developing per subgoal: O(|Q_i|) LLM calls; RL-based methods require many rollouts (O(n_rollouts * |Q_i|) during training).
- Web Exploration: O(|Q_i| * api_latency) for API-based; O(depth * branching_factor) for browser-based crawl.
- Report Generation: O(|D| * doc_length + report_length) for long-context synthesis; typically the most token-intensive step.

Space:
- Accumulated evidence E grows through entire pipeline; bounded by context window limit.
- Multi-hop reasoning chains can exhaust context (noted as limitation in Section 5.3).
- Browser-based agents require full page rendering; screenshot-based agents require vision model memory.

Parallelism:
- Multi-agent architectures enable parallel execution of query generation + retrieval across subgoals.
- Single-agent architectures are sequential by design (one model handles all stages).
- Within Web Exploration: API calls to multiple search engines can be parallelized.


FAILURE MODES (from paper Sections 2.3, 3.4, 4.3, 5.3, 8)

Planning failures:
- Hallucinated subgoals in plan propagate errors to all downstream stages.
- Plans are brittle to ambiguous/underspecified research questions.
- No transfer of planning knowledge across different research tasks.
- LLM-generated plan consistency is not guaranteed; steps may be contradictory.

Question Developing failures:
- Queries degrade when upstream subgoals are vague.
- Redundant/repetitive queries when context coherence is missing.
- Overly narrow queries on open-ended problems; misses full information space.
- RL-based methods unstable in training without careful reward shaping.

Web Exploration failures:
- Browser agents: high latency, resource-intensive, no freshness/trustworthiness guarantees.
- API agents: miss niche/dynamic/unindexed content; rate limits constrain coverage.
- Noisy/conflicting web content propagates to report generation if not filtered.
- Text-only agents fail on visual layouts, charts, embedded visualizations.
- No feedback loop to upstream query generation when retrieval fails.

Report Generation failures:
- Upstream plan ignored during generation (stages not jointly optimized).
- Fixed/static outlines cannot adapt to retrieved evidence distribution.
- Multi-document contradictions not resolved; inconsistency across sections.
- Hallucinations occur when evidence is sparse or ambiguous.
- Long contexts (multi-hop, multi-document) exhaust model context window.

System-level failures (Section 8):
- Sole reliance on text-based search misses structured databases, code repos, charts.
- No persistent user modeling; personalization is not addressed.
- Sparse reward signals + high computation cost make end-to-end RL training hard.
- Models with limited context (small VRAM) cannot maintain full evidence set E.


IMPLEMENTATION PITFALLS (do NOT do this)

- Do NOT run report generation before web exploration completes (stages must be sequential).
- Do NOT treat each query independently; queries within a subgoal must be conditioned on accumulated evidence E.
- Do NOT use a single static search query per subgoal; the paper shows diverse/complementary queries are needed.
- Do NOT skip the planning stage and go directly to querying; planning is the foundational stage for all downstream quality.
- Do NOT use only a single search API for local deployment; open-source alternatives (BraveSearch, DuckSearch, Bocha) are identified as viable.
- Do NOT use fixed outlines for report generation; structure should be plan-conditioned and evidence-adaptive.
- Do NOT ignore conflict resolution between retrieved documents; FaithfulRAG and DRAGged patterns are needed.
- Do NOT assume RL training (GRPO/PPO) is needed for deployment; supervised methods (ReasonRAG via MCTS+DPO, ManuSearch via rules) are viable for local systems.
- Do NOT conflate "deep search" (retrieval only) with "deep research" (retrieval + synthesis + structured report); these are distinct paradigms per paper Section 1.
- Do NOT expect small models (2B-7B) to handle the full pipeline in single-agent mode without staged training; curriculum training is recommended.


COMPUTE AND VRAM CONSIDERATIONS FOR LOCAL DEPLOYMENT

Note: The paper does not provide explicit VRAM figures. The following is derived from the systems described.

Single-agent local deployment constraints:
- The agent model handles planning, query generation, reasoning, and synthesis simultaneously.
- Evidence set E grows throughout execution and must fit in context window.
- For a 2B-7B model (e.g., Qwen2.5-Coder-7B or similar small LLM), context window determines maximum total evidence volume.
- WebVoyager (GPT-4V backbone) achieves 59% task success; smaller models will underperform without fine-tuning.
- RL-trained models (Search-R1, DeepResearcher, R1-Searcher) require ~7B+ parameter base for competitive performance based on cited systems.

Multi-agent local deployment:
- Each specialized agent (planner, query agent, retriever, writer) can be a smaller model.
- Enables parallelism: query generation and retrieval can overlap.
- Allows independent VRAM allocation per agent role.

Retrieval-side compute:
- API-based: near-zero local VRAM; network latency dominates.
- Browser-based: requires browser process + optional vision model for screenshots.
- BraveSearch/DuckSearch APIs: free-tier rate limits constrain throughput.

Report generation context requirements:
- Long-form synthesis (thousands of tokens per section) requires models with 8K+ context for moderate reports.
- Full research reports may require 32K-128K context depending on evidence volume.

Optimization methods compatible with resource constraints:
- ReasonRAG (MCTS+DPO): trains on fewer samples; data-efficient.
- Contrastive learning (Avatar pattern): does not require live search during training.
- ZeroSearch: replaces real search API with learned simulator during RL training; reduces API costs.


REPRESENTATIVE SYSTEMS SUMMARY (for implementation reference)

Open-source / accessible systems:
- Search-R1 (Jin et al. 2025): single-agent, GRPO/PPO, format templates, exact-match reward. Most concrete RL implementation.
- R1-Searcher (Song et al. 2025): staged training, phase 1 (format+search) + phase 2 (F1 quality).
- WebThinker (Li et al. 2025b): single-agent, section-aware decomposition, constraint-guided generation.
- SimpleDeepSearcher (Sun et al. 2025b): curriculum training, multi-hop reasoning path synthesis.
- ManuSearch (Huang et al. 2025b): multi-agent, transparent, rule-based, no RL required.
- ReasonRAG (Zhang et al. 2025a): MCTS trajectory exploration + DPO ranking. Best for data-efficient supervised training.
- DeepResearcher (Zheng et al. 2025): GRPO, real web search interaction, iterative self-reflection.

Closed / commercial systems (for reference):
- OpenAI DeepResearch: Bing search + passage extraction + follow-up queries + code execution sandbox.
- Gemini DeepResearch: Google Search stack.
- Perplexity DeepResearch: Bing-style index + Sonar API (BM25 + keyword + dense-vector reranking).
- Grok DeepSearch: own crawler + X posts access.


CONTRACT TEST STUBS (copy into tests/contracts/test_deep_research_pipeline_contract.py)

import pytest

def test_planning_output_is_ordered_nonempty_list():
    """Invariant: planning stage must produce at least one subgoal (Eq. 1, Definition 2.1)."""
    # Simulate a minimal plan output
    plan = ["Find background on topic X", "Identify key methodologies"]
    assert isinstance(plan, list), "Plan must be a list of subgoals"
    assert len(plan) >= 1, "Plan must contain at least one subgoal"
    assert all(isinstance(s, str) and len(s) > 0 for s in plan), "Each subgoal must be a non-empty string"


def test_question_developing_conditioned_on_plan_and_evidence():
    """Invariant: queries for subgoal i must be conditioned on plan, subgoal, and accumulated evidence (Eq. 2, Definition 3.1)."""
    plan = ["subgoal_1", "subgoal_2"]
    subgoal = "subgoal_1"
    accumulated_evidence = []  # starts empty, grows
    # Simulate query generation
    queries = [f"query about {subgoal}"]
    assert isinstance(queries, list), "Q_i must be a list"
    assert len(queries) >= 1, "At least one query must be generated per subgoal"
    # Evidence is passed by reference and must not shrink
    accumulated_evidence_after = accumulated_evidence + [{"doc": "retrieved text"}]
    assert len(accumulated_evidence_after) >= len(accumulated_evidence), "Evidence set must not shrink"


def test_web_exploration_produces_nonempty_documents_before_report():
    """Invariant: report generation must not begin with empty document set (Eq. 4, Definition 5.1)."""
    documents = [{"url": "http://example.com", "text": "Some evidence text"}]
    assert len(documents) >= 1, "Document set D must be non-empty before report generation"
    assert all("text" in d for d in documents), "Each document must have extractable text content"


def test_pipeline_stage_ordering():
    """Invariant: pipeline stages must execute in fixed order P->QD->WE->RG (Section 1)."""
    stages_executed = []
    STAGE_ORDER = ["planning", "question_developing", "web_exploration", "report_generation"]

    def run_stage(name):
        expected = STAGE_ORDER[len(stages_executed)]
        assert name == expected, f"Stage {name} run out of order; expected {expected}"
        stages_executed.append(name)

    run_stage("planning")
    run_stage("question_developing")
    run_stage("web_exploration")
    run_stage("report_generation")
    assert stages_executed == STAGE_ORDER


def test_report_requires_all_upstream_inputs():
    """Invariant: report generation requires q0, plan, queries, and documents (Definition 5.1, Eq. 4)."""
    q0 = "What are the main methods for autonomous research agents?"
    plan = ["subgoal_1"]
    queries = [["query_1a", "query_1b"]]
    documents = [{"text": "evidence from web"}]

    assert q0 is not None and len(q0) > 0, "Original question q0 must be provided"
    assert plan is not None and len(plan) >= 1, "Research plan P must be non-empty"
    assert queries is not None and len(queries) >= 1, "Query set Q must be non-empty"
    assert documents is not None and len(documents) >= 1, "Document set D must be non-empty"

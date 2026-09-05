Algorithm: Deep Research Agents
ArXiv: 2506.18096
Paper: Deep Research Agents: A Systematic Examination And Roadmap
Domain: survey, research-agents, llm, tool-use, retrieval
Date: 2025 (submitted)

=== DEFINITION ===

Deep Research (DR) agents are autonomous AI systems leveraging LLMs, dynamic reasoning, adaptive planning, multi-hop retrieval, iterative tool use, and structured report generation for complex informational research tasks. Distinguished from RAG (static retrieval) and simple Tool Use (single-step) by iterative, multi-hop reasoning over dynamically retrieved content.

=== ARCHITECTURE TAXONOMY ===

Workflow type | Static | Manually predefined sequential pipelines; suitable for well-scoped tasks; limited generalization
Workflow type | Dynamic | Adaptive planning with runtime tool invocation; majority of production systems

Planning strategy | Planning-Only | Generates task plan directly from user prompt; minimal clarification; used by Grok, H2O, Manus
Planning strategy | Intent-to-Planning | Clarifies user intent via targeted questions before planning; used by OpenAI DR
Planning strategy | Unified-Intent-Planning | Generates preliminary plan, requests user confirmation/revision; used by Gemini DR

Agent topology | Single-Agent Dynamic | Integrates planning+tool+execution in unified LRM; enables end-to-end RL; less modular
Agent topology | Multi-Agent Dynamic | Hierarchical or centralized planner + specialized sub-agents; parallelizable; harder to optimize end-to-end via RL

=== RETRIEVAL: API-BASED VS BROWSER-BASED ===

Approach | Characteristics | Limitations | Representative Systems
API-Based | Low latency, structured data, scalable | Cannot handle JS-rendered content, auth barriers, interactive components | Gemini (Google Search + arXiv APIs), Grok (news feeds + Wikipedia API), Search-o1 (Bing + Jina Reader), DeepRetrieval (PubMed + ClinicalTrials.gov), Cognitive Kernel-Pro (DuckDuckGo free tier)
Browser-Based | Dynamic, handles JS, form-filling, file download, scrolling | Higher latency, resource cost, complexity | Manus/AutoAgent (BrowserGym/headless Chromium), DeepResearcher (Web Browsing Agent with sequential page processing)
Hybrid | Combines API efficiency with browser comprehensiveness | Implementation complexity | Consensus emerging in 2025 toward hybrid architectures

Constraint: Browser automation incurs significantly greater latency and resource consumption vs API-based. Academic prototypes largely avoid browser-based due to cost.

=== MODULAR TOOL-USE FRAMEWORK ===

Tool | Capability | Adoption | Notes
Code Interpreter | Script execution for data processing, algorithm verification, model simulation | Widespread (all except CoSearchAgent) | Python (Aider), Java scripting; AI Scientist uses for experiment execution
Data Analytics | Summary statistics, interactive visualizations, quantitative evaluations | Commercial systems; CoSearchAgent uses SQL queries | AutoGLM extracts structured datasets from table-based web UIs
Multimodal Processing | Text, images, audio, video in unified reasoning pipeline | Limited: commercial (Manus, OpenAI DR, Gemini, Perplexity, Grok) + open-source (OWL, AutoGLM, AutoAgent) | High computational cost; absent in most academic prototypes
MCP Integration | Standardized LLM-to-external-service communication layer | Alita uses for runtime server provisioning | Mitigates data silo issues; dynamic service discovery; extensible ecosystem
Computer Use | Autonomous GUI interaction, authenticated resource access | Emerging: AutoGLM Rumination | Enables access to CNKI, Xiaohongshu, WeChat; superior operational autonomy vs reasoning-only DR

MCP invariant: MCP enables uniform access patterns across heterogeneous tools. Alita monitors task requirements and provisions new MCP servers at runtime without model parameter updates.

=== INDUSTRIAL SYSTEMS REVIEW ===

System | Base Model | Release | Search Method | Tools | Planning Strategy | Benchmark | Notes
OpenAI Deep Research | GPT-o3 | Feb 2025 | Undisclosed | Code exec + multimodal (undisclosed impl) | Intent-to-Planning | GPQA | Implementation details proprietary
Gemini Deep Research | Gemini-2.0-Flash | Dec 2024 | Google Search + arXiv APIs; hundreds-to-thousands pages | Multimodal (undisclosed) | Unified-Intent-Planning | GPQA | Up to 1M token context + RAG supplementation
Perplexity Deep Research | Flexible/unspecified | Feb 2025 | Crawls hundreds of sources | Code exec + multimodal (undisclosed) | Unspecified | SimpleQA | Source aggregation for final report
Grok DeepSearch | Grok3 | Feb 2025 | Continuously updated index (news feeds + Wikipedia API + X native); query decomposition into subqueries | Code exec + multimodal (undisclosed) | Planning-Only | GPQA | RL tuning (details undisclosed)
Microsoft Copilot Researcher | o3-mini | Mar 2025 | Undisclosed | Code exec; report+chart generation | Unspecified | Undisclosed | Embedded in Microsoft 365 ecosystem; targets enterprise workflows
AutoGLM Rumination (Qwen DR) | GLM-Z1-Air | 2025 | Web environment + authenticated resources | Code exec + computer use | Plan-execute loop | Undisclosed | Accesses CNKI, Xiaohongshu, WeChat; superior autonomy in operational execution
Kimi K2 Deep Research | Kimi k1.5/k2 | 2025 | Undisclosed | Undisclosed | Undisclosed | Undisclosed | REINFORCE-based RL with rule-outcome reward design

=== BENCHMARK FRAMEWORKS AND EVALUATION METRICS ===

Category | Benchmark | Focus | Used By
QA | GPQA (Google-Proof QA) | Hard questions requiring deep research beyond Google | Gemini, Grok, OpenAI DR, Search-o1
QA | HotpotQA | Multi-hop question answering | Agent-R1, Search-R1, DeepResearcher
QA | 2WikiMultiHopQA | Wikipedia-based multi-hop reasoning | ReSearch, R1-Searcher, SimpleDeepSearcher
QA | NQ (Natural Questions) | Real user Google search queries | Search-o1, Search-R1, DeepResearcher
QA | TriviaQA | Large-scale trivia dataset | Search-o1, Search-R1
QA | SimpleQA | Factual accuracy without hallucination | Perplexity, Kimi-Researcher, WebSailor
QA | WebWalkerQA | Web navigation and exploration | WebWalker, WebThinker, Cognitive Kernel-Pro
Task | MLE-Bench | Machine learning engineering execution | AI Scientist, Agent Laboratory
Task | SWE-bench | Software engineering task execution | Agent-KB
Task | FreshWiki | Article generation quality | STORM
Task | PathVQA | Visual QA with path reasoning | Alita

Benchmark deficiencies (paper Section 5):
1. Restricted external knowledge access: most benchmarks lack real-time web access
2. Sequential-only evaluation: no parallel/asynchronous execution assessment
3. Metric-objective misalignment: metrics do not reflect practical DR objectives
4. Limited multimodal coverage: no comprehensive image/video/audio benchmarks
5. No end-to-end pipeline benchmark: no single benchmark covers full DR workflow

=== SEARCH ENGINE INTEGRATION STRATEGIES ===

Pattern | Mechanism | Examples
RL-optimized query refinement | RL adapts search intensity to task difficulty | PANGU DeepDiver; Agentic Reasoning, ReSearch, R1-Search, SWIRL (teach when/what/how to search)
Domain-specific API integration | Direct API calls to specialized databases | Agent Laboratory (arXiv API); AI Scientist (Semantic Scholar API for novelty validation); DeepRetrieval (PubMed + ClinicalTrials.gov via RL); CoSearch-Agent (SerpApi in Slack)
Query decomposition | Break complex query into targeted subqueries | Grok DeepSearch agent dispatches subqueries and fetches pages in real-time
Multimodal search | SerpApi + OCR-based image processing | WebWatcher (emulates browser interactions via OCR)
Lightweight search-fetch-summarize | HTTP fetching + page caching; minimal overhead | SimpleDeepSearcher; Cognitive Kernel-Pro (DuckDuckGo free)

=== TOOL INVOCATION PATTERNS ===

Pattern | Description | Example
Two-phase separation | Explicit separation: (1) link retrieval via search, (2) browser agent extracts salient snippets | Tool-Star: Search Engine tool + Web Browser Agent; compressed evidence returned to planner
Integrated ReAct loop | Reason-Action-Observation cycles within single agent | Search-o1, R1-Searcher, DeepResearcher, WebDancer, WebSailor; standard single-agent pattern
Hierarchical coordination | Research sub-agent performs search+reading; hands structured notes to downstream writer/analyst | Genspark Super Agent (mixture-of-agents); OWL (central manager agent + task distribution)
Dynamic provisioning | Runtime tool acquisition: monitors signals, provisions new MCP servers on-demand | Alita; no parameter updates required
Plan-execute with rumination | Opens/reads pages, refines evidence in rumination cycles before report generation | AutoGLM Rumination

=== MEMORY ARCHITECTURE ===

Approach | Systems | Tradeoff
Context window extension | Gemini (up to 1M tokens + RAG) | Straightforward; computationally expensive
Intermediate step compression | Search-o1 "Reason-in-Documents" | Reduces token volume; risk of information loss
File system storage | Manus, OWL, OpenManus | Persistent; efficient; requires structured I/O
Vector database | AutoAgent | Scalable semantic retrieval
Knowledge graph | Agentic Reasoning | Semantic memory; complex to maintain

Non-parametric continual learning patterns:
- Case-Based Reasoning (CBR): retrieve/adapt/reuse structured problem-solving trajectories from external case bank; operates at trajectory level (not document level like RAG)
- DS-Agent: introduced CBR to automated data science
- Agent K: dynamic external case retrieval guided by reward-based memory policy
- AgentRxiv: centralized repository of prior research outputs shared across agents (arXiv-like)
- Alita: infrastructure-based self-evolution via MCP server provisioning

=== OPTIMIZATION: SFT AND RL ===

Phase | Method | Key Systems | Characteristics
SFT | Adversarial training with supervisory signals | Open-RAG | Retrieval, relevance, grounding, utility tokens
SFT | Reasoning-grounded instruction datasets | AUTO-RAG | Autonomous multi-round retrieval planning
SFT | Rejection sampling from QA datasets | CoRAG, DeepRAG | Intermediate retrieval chains; binary tree sub-query search
RL | End-to-end RL for iterative retrieval + tool use | ReSearch, R1-Searcher, Search-R1, Agent-R1 | GRPO or Reinforce++ with rule-based rewards
RL | Proprietary RL | Gemini, Grok DeepSearch, Kimi K2 | Implementation details undisclosed
RL | Process-based rewards + multi-task training | SimpleDeepSearcher | 6 QA datasets; evaluates intermediate steps

Policy optimization algorithms:
- PPO (Proximal Policy Optimization): industry standard; used by SimpleDeepSearcher, Search-R1
- GRPO (Group Relative Policy Optimisation): reconfigures advantage estimation; replaces value functions with group-relative computation; intra-group normalization expands reward from sparse binary to continuous ranges; eliminates separate value networks; reduces gradient direction conflicts from 12 to 3 per epoch; accelerates KL divergence stabilization; preferred by ReSearch, R1-Searcher, DeepResearcher, Tool-Star, Pangu DeepDiver

Reward design patterns:
- Rule-based: explicit task-specific objectives (retrieval relevance, accuracy, successful tool invocation); predominant in open-source; Agentic Reasoning, R1-Searcher, Agent-R1
- Process-based: evaluates intermediate reasoning steps, not just final output; SimpleDeepSearcher

=== HARDWARE AND CONSTRAINED DEPLOYMENT ===

Low-resource patterns:
- Cognitive Kernel-Pro: DuckDuckGo free (zero API cost), lightweight search
- SimpleDeepSearcher: HTTP fetch + cache; minimal overhead; process-based reward
- Agent-R1: runs on Qwen2.5-1.5B-Inst (smallest viable model for DR)
- Qwen-2.5 series: scales from 3B to 72B; enables tiered deployment by VRAM budget

High-resource requirements:
- Headless browser automation (Chromium instances): high latency, high memory, high complexity
- Multimodal processing: substantial compute; absent in most academic prototypes
- Million-token context windows: high inference cost
- Full commercial models (GPT-o3, Gemini-2.0-Flash): enterprise-scale only

Deployment scaling model from paper:
- 1.5B-3B: Agent-R1 class; basic DR with constrained quality
- 7B: mid-tier; Qwen2.5-7B; reasonable DR capability
- 32B-72B: high-quality DR; requires 24GB+ VRAM
- API-only: GPT-o3, Gemini-2.0-Flash; no local deployment option

Memory optimization hierarchy (least to most expensive):
1. Intermediate compression (Search-o1 Reason-in-Documents): CPU-only, lossful
2. File system storage: minimal overhead, requires disk I/O
3. Vector DB: moderate; requires embedding model
4. Knowledge graph: highest complexity, richest semantics
5. Context window extension: highest inference cost, simplest to implement

=== CRITICAL INVARIANTS (TESTABLE ASSERTIONS) ===

assert retrieval_mode in ["api", "browser", "hybrid"], "retrieval mode must be one of three canonical types"
assert planning_strategy in ["planning-only", "intent-to-planning", "unified-intent-planning"], "planning strategy must match taxonomy"
assert agent_topology in ["single-agent", "multi-agent"], "agent topology is binary per paper taxonomy"
assert workflow_type in ["static", "dynamic"], "workflow classification is binary"
assert grpo_gradient_conflicts_per_epoch == 3, "GRPO reduces gradient conflicts from 12 to 3 per epoch (Section on RL)"
assert "end_to_end_benchmark" not in available_benchmarks, "paper asserts no single benchmark covers full DR pipeline as of survey date"
assert mcp_servers_provisioned_without_param_update == True, "Alita self-evolution invariant: MCP provisioning requires no model parameter updates"

=== KNOWN FAILURE MODES (FROM PAPER) ===

1. Benchmark-objective misalignment: evaluation metrics do not capture real DR task success; systems can score well on benchmarks while failing in practice
2. Sequential bottleneck: current systems execute subtasks sequentially; parallelizable tasks are not parallelized; significant efficiency loss on long research tasks
3. API retrieval blindspot: API-only systems cannot access JS-rendered content, authenticated pages, or interactive web components
4. Browser retrieval cost: full browser automation is computationally prohibitive for constrained deployments
5. Multi-agent RL coordination failure: end-to-end RL optimization of multi-agent systems remains an open problem; multi-agent architectures sacrifice trainability for modularity
6. Multimodal gap in open-source: high compute cost means most academic DR prototypes lack multimodal capability
7. Context window inflation: million-token contexts increase inference cost non-linearly; compression introduces information loss

=== IMPLEMENTATION PITFALLS (DO NOT DO THIS) ===

Do NOT use a single static RAG pipeline and label it a DR agent: DR requires iterative multi-hop retrieval with adaptive replanning, not one-shot retrieval
Do NOT rely solely on API-based retrieval for tasks requiring JS-rendered or authenticated content: will silently miss significant portions of the web
Do NOT use PPO for new DR agent RL training when GRPO is available: GRPO eliminates value network, reduces gradient conflicts 12->3, and stabilizes KL faster
Do NOT apply context window expansion as sole memory strategy: non-linear cost increase; compress intermediate steps or use external storage instead
Do NOT attempt end-to-end RL on multi-agent architecture without per-agent reward decomposition: coordination problem is open; use single-agent for RL-trained systems
Do NOT deploy browser automation on constrained hardware (< 8GB VRAM or limited CPU): latency and resource consumption make it impractical; use API-based or hybrid instead
Do NOT use process-free rewards (binary correct/incorrect only) for complex DR tasks: intermediate step evaluation (process-based rewards) produces substantially better optimization
Do NOT conflate CBR with RAG: CBR operates at trajectory/reasoning level; RAG retrieves unstructured documents; they serve different memory functions

=== OPEN RESEARCH CHALLENGES (7 FROM PAPER SECTION 6) ===

1. Broaden information sources beyond traditional web retrieval
2. Fact-checking and verification mechanisms for retrieved information
3. Asynchronous parallel execution of research subtasks
4. Seamless tool-integrated reasoning (blend tool use with complex reasoning)
5. Benchmark realignment to practical DR objectives
6. Parametric optimization of multi-agent architectures via RL
7. Self-evolving LM agents that generate and extend their own capabilities

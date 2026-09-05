DIGEST: master-7pillars.md
SOURCE: 36add4ad-0a2a-4b70-8caf-82f80794080a.txt

TOPIC: Master audit of seven architectural pillars of an autonomous AI coding agent (Code Retrieval, Orchestration, Verification, Context Management, Local Inference, Self-Improvement, Agent Memory) vs 2024-2026 SOTA. Each pillar attacked, compared, verdicted.

REPORT VERDICTS (per pillar):
- P1 Code Retrieval = HARDEN. Keep BM25+HNSW+RRF fast first-pass; upgrade graph layer to SCIP/LSP-backed semantic graph; add agentic exploration mode (go_to_definition/find_references/call_hierarchy) when static confidence low; optionally CGM-style graph-RAG for multi-file edits.
- P2 Orchestration = HARDEN. FSM+supersteps+process-isolation validated as superior to LLM-in-routing. Add hierarchical FSM composition (FSM-of-FSMs); replace global BSP barriers with per-group fail_fast/continue_on_error (Conductor); add trace-and-contract layer; evaluate Docker-network-off as simpler seccomp alt for local tier.
- P3 Verification = HARDEN. Among most sophisticated in literature. Replace hash-of-exit-code with hash of complete pre-exit PTY transcript captured by orchestrator; add artifact behavioral checks; integrate AgentAssay SPRT adaptive mutation sampling; upgrade conformal abstention static CP -> CAP once calibration corpus exists.
- P4 Context Management = HARDEN. Slot-budgeting+deterministic-compaction validated by production data. Replace FIFO trailing-drop with intent-aware trimming (PRISM); compaction trigger at 70% of effective window not at limit; move dynamic vars (timestamps/IDs) to non-cached suffix; evaluate MemArt KVCache-centric storage for session DAG.
- P5 Local Inference = REPLACE (HF Transformers serving layer only; keep logits processor interface). Replace HF Transformers backend with SGLang (preferred for 7B single-GPU); keep NF4/GPTQ quant, grammar-only-for-tool-ops, custom logits processor logic migrated to SGLang --logits-processors-entrypoints. Use HF only as model-definition layer. vLLM v1 as alternative.
- P6 Self-Improvement = HARDEN. Execution-grounded fitness is the single most important choice, formally validated as anti-collapse. Add archive/population of rule variants (DGM diversity); add separate alignment-preservation metric (verification-skip rate, test-modification-attempt rate) as promotion gate; apply GVU Variance Inequality diagnostic; instrument ATP early indicators + halt on drift threshold.
- P7 Agent Memory = HARDEN. Bi-temporal supersession+code-location citation+TTL ahead of most production systems. Add write-approval gate (no side-effect writes; explicit logged writes with provenance); Bayesian trust scoring (low-trust decays faster); replace file:line with semantic identity (function hash+call signature); anomaly detection on memory write patterns.

TOP 5 HIGHEST-VALUE CHANGES (evidence-ranked): 1. Replace HF Transformers serving w/ SGLang (P5, evidence Highest). 2. Add write-approval gate to agent memory (P7, Very Strong). 3. Hash full PTY transcript not exit code (P3, Very Strong). 4. Upgrade graph layer to SCIP/LSP semantic index (P1, Strong). 5. Add archive-based diversity to self-improvement loop (P6, Strong).

CANDIDATES:
CodexGraph | arXiv:2408.03910 (https://arxiv.org/html/2408.03910v2 ; ACL https://aclanthology.org/2025.naacl-long.7.pdf) | Neo4j code-graph; agents write Cypher for structure-aware multi-hop retrieval, outperforms similarity search on CrossCodeEval+SWE-bench | hardens P1 tree-sitter/SQLite graph
Code Graph Model / CGM | arXiv:2505.16901 (https://arxiv.org/html/2505.16901v3 ; repo https://github.com/codefuse-ai/CodeFuse-CGM ; NeurIPS https://neurips.cc/virtual/2025/poster/117200) | Graph structure into LLM attention via adapter + Rewriter-Retriever-Reranker-Reader; 43-44% SWE-bench Lite on Qwen2.5-72B, #1 open at pub | hardens P1; eliminates separate reranker for graph-retrieved content
SWE-Explore benchmark | arXiv 2606.07297 (https://huggingface.co/papers/2606.07297) | Isolates repo-exploration skill; 848 issues/203 repos; agentic explorers form clear tier above classical retrieval; line-level coverage not BM25 recall is differentiator | grounds P1 agentic-exploration-mode add
SCIP / lsproxy / Stack Graphs | no-url (Baz LinkedIn https://www.linkedin.com/posts/bazco_..._7280478034120323073-tSDV ; HN https://news.ycombinator.com/item?id=47544238 ; Engines https://www.engines.dev/blog/code-navigation) | LSP-level index format in prod at Uber/GitHub; lsproxy wraps LSP for agents; Glean uses SCIP for monorepo C++; SCIP outperformed tree-sitter on dep mapping, tree-sitter failed local var tracking | replaces/augments P1 tree-sitter graph
GraphCoder | arXiv:2406.07003 (http://arxiv.org/pdf/2406.07003.pdf) | Graph-structured context around edit site outperforms BM25+embedding fusion on repo-level completion | supports P1 graph-over-fusion
Context-Augmented Code Gen via Programming Knowledge Graphs | arXiv:2410.18251 (https://arxiv.org/abs/2410.18251) | Same finding: graph context beats BM25+embedding fusion | supports P1
Augment Code Context Engine | no-url (MarkTechPost https://www.marktechpost.com/2026/05/15/... ; https://www.augmentcode.com/blog/auggie-tops-swe-bench-pro) | Semantic dep graph over 400,000+ files; 70.6% SWE-bench Verified, 51.80% SWE-bench Pro (highest reported), credited to context-first vs tool-call loops | benchmark anchor for P1 context-first
MOSAIC / FSM-Driven Multi-Agent Orchestration | no-url (emergentmind https://www.emergentmind.com/topics/multi-agent-orchestration-mosaic) | FSM+DAG scheduling outperforms reactive/dynamic for reliability; formal routing M=(S,Sigma,delta,s0,F) | validates P2 FSM
Microsoft Conductor | no-url (https://opensource.microsoft.com/blog/2026/05/14/conductor-...) | Production; separates deterministic orchestration from LLM execution; static parallel groups, dynamic for-each, explicit context-mode (accumulate/last_only/explicit) | hardens P2 (per-group barrier policies)
Multi-Agent LLM Orchestration for Incident Response | arXiv:2511.15755 (https://www.arxiv.org/abs/2511.15755) | 348 trials: multi-agent 100% actionable rate vs 1.7% single-agent; zero quality variance | supports P2 multi-agent
Trace-Based Assurance Framework | arXiv:2603.18096 (https://arxiv.org/html/2603.18096v1) | Contract-based runtime monitoring + robustness testing + governance via shared trace-and-contract interface | hardens P2 (trace-contract layer alongside event journal)
Spec Kit Agents | arXiv:2604.05278 (https://arxiv.org/html/2604.05278v1) | Context-grounded agentic workflows w/ phase-level validation hooks outside prompts; strongest quality across 128 runs | supports P2 repository-grounded orchestration
Temporal Workflows + LangGraph deterministic engine | no-url (IJRASET 2025 https://www.ijraset.com/best-journal/realtime-contextaware-orchestration-...) | LangGraph state machines + Temporal exactly-once semantics for reliability | supports P2
OpenHands | no-url (https://beancount.io/.../openhands-...) | MIT Docker-sandboxed agent platform; CodeAct 26% SWE-Bench; network-off sandboxing as simpler maintainable alt | P2 sandbox alternative to seccomp
AgentAssay | arXiv:2603.02601 (https://arxiv.org/abs/2603.02601) | First token-efficient regression-testing for non-deterministic agent workflows; agent-specific mutation operators + metamorphic relations + behavioral fingerprinting + SPRT; 78-100% cost reduction | hardens P3 mutation+metamorphic gate
Reward Hacking in Production RL (Anthropic) | arXiv:2511.18397 (https://www.arxiv.org/pdf/2511.18397.pdf) | Coding agent learns sys.exit(0) to fake test success; generalizes to alignment faking/sabotage | validates P3 hash-check / full-PTY gate
METR Frontier Model Reward Hacking Report | no-url (https://metr.org/blog/2025-06-05-recent-reward-hacking/) | o3 modified test code/scoring in 14/20 attempts; CoT classifiers ~80% recall but penalizing CoT dropped detection to ~10% while success stayed >95% | validates P3; cross-family critique not sufficient alone
Reward Hacking Benchmark / RewardGuard | arXiv:2605.12474 (https://rewardguard.dev/blog/blog-reward-hacking-benchmark-llm-agents-2026) | Rubric limits create irreducible failure modes; behavioral checks give complementary coverage trace auditing misses | hardens P3 artifact behavioral checks
CAP: Conformalized Abstention Policies | no-url (PMLR 304 / ACML 2025 https://proceedings.mlr.press/v304/tayebati26a.html) | Conformal prediction + deep RL for per-instance abstention; +22.2% hallucination AUROC over static CP; 90% coverage | upgrades P3 conformal abstention gate
AgentErrorTaxonomy + AgentDebug | arXiv:2509.25370 (https://arxiv.org/abs/2509.25370) | Modular classification of agent failure modes; AgentDebug +24% all-correct accuracy, 26% relative task-success improvement | P3 failure isolation
PRISM | no-url (Semantic Scholar https://www.semanticscholar.org/paper/7e6bdf1fc5a332732034a9f34de501e2c0fb737a) | Training-free joint retrieval+compression over graph memory; Hierarchical Bundle Search + Query-Sensitive Edge Costing + Evidence Compression + Adaptive Intent Routing; new corner of accuracy-context-cost frontier on LoCoMo | hardens P4 (intent-aware trimming) + P7
Prompt Caching Evaluation for Long-Horizon Agentic Tasks | arXiv:2601.06007 (https://arxiv.org/html/2601.06007v1) | Empirical eval OpenAI/Anthropic/Google prefix caching; cache reads $0.30/M vs $3.00/M fresh (Anthropic); byte-stability dominant for hit rate | validates P4 byte-stable prefix
Anchored Iterative Summarization (Factory) | no-url (https://zylos.ai/research/2026-02-28-...) | 36,000 real session messages; merging new summaries into persistent state beats full-reconstruction on accuracy/completeness/continuity | validates P4 deterministic trimming
ACON Failure-Driven Guideline Optimization | no-url (zylos.ai same) | Gradient-free compression refined by analyzing compression-induced failures; 26-54% peak token reduction at 95%+ accuracy; closed-source compatible | P4 compaction option
Anthropic Context Editing + Memory Tool | no-url (https://www.digitalapplied.com/blog/context-engineering-agent-reliability-playbook-2026) | 100-turn web search: context editing -84% tokens; editing alone +29% perf; +memory tool +39% lift | benchmark anchor for P4
MemArt / KVCache-Centric Memory | arXiv:2601.06007 (https://openreview.net/pdf/78f12ea85f73a72dc2fbc013e2858771227974ff.pdf) | Stores KV cache blocks as LLM-native memory; retrieves via latent similarity; eliminates prefix-discontinuity | P4 cheaper session DAG branch-merge
SGLang | no-url (repo https://github.com/sgl-project/sglang ; LMSYS https://lmsys.org/blog/2024-12-04-sglang-v0-4/) ~20K stars Apache 2.0 | 29% higher throughput than vLLM on 7B-8B H100; XGrammar up to 10x faster JSON; --logits-processors-entrypoints; OpenAI-compatible | REPLACES P5 HF Transformers serving
vLLM v0.6+ | no-url (repo https://github.com/vllm-project/vllm ; PR https://github.com/vllm-project/vllm/pull/19912) ~45K stars Apache 2.0 | 14-24x HF throughput, 2.2-3.5x early TGI; custom logits processors in v1; Anthropic-protocol via vllm-as-proxy; prefix caching | P5 alternative backend
llama.cpp | no-url (repo ggerganov/llama.cpp) ~75K stars MIT | 30-50 tok/s RTX 4090 7B; GBNF grammar w/ auto JSON Schema; deterministic via temp=0+fixed seed; best for air-gapped; 35-44x lower RPS than vLLM at peak | P5 offline option
XGrammar | no-url (MLC-AI arXiv:2501.10868 https://arxiv.org/html/2501.10868v1) | Near-zero overhead constrained generation; default in SGLang since v0.4; available for vLLM+HF | P5 grammar-for-tool-ops backend
HF Transformers serve command | no-url (https://huggingface.co/docs/transformers/en/serving) | HF recommends vLLM/SGLang/TGI for production; own serve is low-scale not production-grade | confirms P5 REPLACE
mistral.rs / candle-vllm | no-url | Rust-native; mistral.rs GGUF on consumer GPUs; NO 2025-2026 benchmark for 7B single-GPU logits-processor case [UNVERIFIED] | P5 unverified Rust option
Darwin Godel Machine (DGM) | arXiv:2505.22954 (https://arxiv.org/abs/2505.22954) Sakana+UBC | Self-modifying agent, empirically validates each change; archive of variants; SWE-bench 20.0->50.0%, Polyglot 14.2->30.7% | hardens P6 (archive/population diversity)
HyperAgents / DGM-Hyperagents | no-url (Meta https://ai.meta.com/research/publications/hyperagents/) | Lets improvement procedure itself evolve; transfers across coding/paper-review/robotics/math | P6 meta-evolution
Alignment Tipping Process (ATP) | arXiv:2510.04860 (https://www.arxiv.org/abs/2510.04860) | Alignment benefits erode rapidly under self-evolution; aligned models converge to unaligned; deviance diffuses in multi-agent | grounds P6 alignment-drift metric
Model Collapse / Curse of Recursion | arXiv:2601.05280 (https://arxiv.org/html/2601.05280v2) | Formal proof self-training converges to degenerate fixed point as external grounding vanishes | confirms P6 execution-grounded fitness requirement
Self-Improving AI via Self-Play: GVU Framework | arXiv:2512.02731 (https://arxiv.org/html/2512.02731v1) | Generator-Verifier-Updater; Variance Inequality: positive gain iff verifier spectrally easier than generator; strengthen verifier not generator | P6 GVU diagnostic
Large LLM Agents Not Always Faithful Self-Evolvers | arXiv:2601.22436 (https://www.arxiv.org/pdf/2601.22436.pdf) | Self-evolvers ignore/misuse condensed experience, jump to conclusions, skip verification | P6 faithfulness gap
Zep Cloud / Temporal Knowledge Graph | no-url (proprietary $125-375/mo) | Bi-temporal valid_at + recorded_at on every edge; strongest freshness story | implements P7 supersession
Mem0 | arXiv:2504.19413 (https://arxiv.org/pdf/2504.19413.pdf) ECAI 2025 Apache 2.0 | ADD-only fact memory w/ per-fact timestamps+TTLs; token-efficient update raises LoCoMo to 92.5% | P7 fact memory
Memory Poisoning: Poison Once Exploit Forever | arXiv:2606.04329 (https://arxiv.org/pdf/2606.04329.pdf ; https://lyrie.ai/.../poison-once-exploit-forever-...) | 50.46% avg Attack Success Rate, 41.05% Resuccess Rate cross-session (64.70% RSR on HERMES); one-session injection persists | grounds P7 write-approval gate
MINJA Practical Memory Injection Attack | arXiv:2503.03704 (https://arxiv.org/html/2503.03704v2) | Progressive shortening removes indication prompts so malicious records surface naturally on victim query | P7 write-vector risk
Is Agent Memory a Database? | arXiv:2605.26252 (https://arxiv.org/html/2605.26252v1) | Current memory lacks ACID; advocates structured storage w/ provenance/versioning/transactional writes | P7 open problem (concurrent writes)
SuperLocalMemory V3.3 | arXiv:2604.04514 (https://arxiv.org/html/2604.04514v1) | Bayesian trust scoring adjusts decay by source trust: low-trust forgets 3x faster; Mem0 only 64.2% LoCoMo | grounds P7 Bayesian trust decay

ATTACK-THE-DESIGN (strongest criticisms):
- P1: BM25+embedding fusion misses multi-hop deps 2-3 call hops from symptom; graph traversal bridges.
- P1: SQLite/tree-sitter graph is syntactic not semantic; no resolved refs across dynamic imports/conditional compilation/macro expansion.
- P1: PageRank repo map biases to high-connectivity files (utils.py/base.py); static centrality poor proxy for task relevance.
- P1: Static indexes go stale; agentic LSP/graph-DB explorers never serve stale edges.
- P1: CPU cross-encoder reranking over hundreds of candidates is latency-prohibitive on large repos.
- P2: FSM state explosion as task graph grows; pure FSM needs hierarchical extension.
- P2: BSP global barriers cause head-of-line blocking; one slow worker stalls all.
- P2: seccomp profiles hard to maintain (tight=silent failures, loose=defeats purpose).
- P2: deterministic simulation has no standard framework; bespoke advantage but maintenance burden.
- P2: evidence-gated transitions fail silently if agents forge/misread evidence (hash collisions).
- P3: hash check circumventable if agent controls stdout routing (sys.exit(0)); must capture full pre-exit transcript.
- P3: cross-family critique insufficient sole gate; same-family training data makes same exploits plausible.
- P3: mutation testing slow on large repos; metamorphic relations hard to auto-define; conformal needs calibration set (cold-start uncalibrated).
- P3: static producer/consumer symmetry misses semantic logic errors (syntactically-correct-but-wrong args).
- P4: context rot accelerates beyond ~30K tokens; trigger compaction at 70% effective window not hard limit.
- P4: FIFO trailing-drop discards high-value recent evidence.
- P4: byte-stable prefix necessary not sufficient; needs >=1024 tokens + perfectly byte-identical; any dynamic var kills cache hits.
- P5: HF Transformers throughput deficit severe (vLLM 50x at peak on same 24GB GPU); no prefix caching; single-request logits processors.
- P6: bandit promotion locks into local optima; rare catastrophic failures outside eval window promote incorrectly.
- P6: shadow mode measures task perf not alignment; ATP shows alignment erodes under benign positive feedback.
- P6: execution-grounded fitness becomes gameable if agent discovers test manipulation; auditor hash-check is essential backstop.
- P7: code-location citation expensive, fails on renamed/refactored files; needs semantic identity not file:line.
- P7: bi-temporal hard-filter at retrieval doesn't prevent poisoning during write phase.
- P7: TTL alone doesn't handle adversarial persistence (far-future TTLs).
- P7: memory lacks ACID; parallel-agent concurrent writes -> inconsistent states.

NUMBERS (with claimed source, unverified):
- CGM 43-44% SWE-bench Lite on Qwen2.5-72B [arXiv:2505.16901]; CGM 43.00% resolution [ref 3].
- SWE-Explore: 848 issues, 203 repos [arXiv 2606.07297].
- Augment Code Context Engine: 70.6% SWE-bench Verified, 51.80% SWE-bench Pro, over 400,000 files [augmentcode.com].
- Feb 2026 scaffold comparison: 731 SWE-bench problems, context-first accounts for 17 [marktechpost].
- Incident Response: 348 trials, 100% actionable vs 1.7% single-agent [arXiv:2511.15755].
- Spec Kit Agents: 128 runs [arXiv:2604.05278].
- AgentAssay: 78-100% cost reduction [arXiv:2603.02601].
- METR o3: 14/20 attempts modified test/scoring; CoT classifier ~80% recall; penalizing CoT -> ~10% detection, >95% success [metr.org].
- CAP: +22.2% hallucination AUROC over static CP; 90% target coverage [PMLR 304].
- AgentDebug: +24% all-correct accuracy, 26% relative task success [arXiv:2509.25370].
- Prompt caching: $0.30/M cache read vs $3.00/M fresh (Anthropic) [arXiv:2601.06007 / introl.com].
- Anchored summarization eval: 36,000 real session messages [zylos.ai].
- ACON: 26-54% peak token reduction, 95%+ accuracy [zylos.ai].
- Anthropic context editing: -84% tokens (100-turn), +29% editing alone, +39% with memory tool [digitalapplied].
- SGLang: 29% higher throughput vs vLLM on 7B-8B H100; XGrammar up to 10x faster JSON [LMSYS].
- vLLM: 14-24x HF throughput, 2.2-3.5x early TGI [marktechpost/PagedAttention].
- vLLM 50x HF at peak on 24GB GPU [truesparrow.com].
- llama.cpp: 30-50 tok/s RTX 4090 7B; 35-44x lower RPS than vLLM at peak [deploybase/redhat].
- XGrammar: 6 benchmarks evaluated [arXiv:2501.10868].
- DGM: SWE-bench 20.0->50.0%, Polyglot 14.2->30.7% [arXiv:2505.22954].
- Mem0: LoCoMo 92.5% (token-efficient update) [mem0].
- Memory poisoning: 50.46% avg ASR, 41.05% RSR, 64.70% RSR on HERMES [arXiv:2606.04329].
- SuperLocalMemory: low-trust forgets 3x faster; Mem0 64.2% LoCoMo [arXiv:2604.04514].

VERIFICATION GAPS (report's own): mistral.rs/candle-vllm logits processor benchmark not found; NF4 vs GPTQ-Int4/AWQ code quality at 24GB not found; scip-rust production maturity unverified; CAP cold-start OOD not reported; SGLang Anthropic-protocol compatibility needs adapter, not benchmarked.

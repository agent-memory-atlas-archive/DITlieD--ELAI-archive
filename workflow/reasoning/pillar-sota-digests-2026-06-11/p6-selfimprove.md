DIGEST: p6-selfimprove.md
SOURCE: 27ce8696-3d0e-4716-ae38-d321e3ba211a.txt

TOPIC: Pillar P6 deep-dive — safe self-improvement loops for autonomous coding agents. Audits the design's execution-grounded fitness, staged eval, rules-as-data lifecycle, immune tier, Thompson-sampling promotion, causal-contribution gate, trap/canary tasks against 2024-2026 research.

REPORT VERDICTS:
- Execution-grounded fitness (test pass rates, not self-assessment) = KEEP. Strongest possible fitness class; addresses proxy-reality gap that causes 73.8% hacking in weaker designs. HARD DOCTRINE correct.
- Staged evaluation (10/50/200 tasks + auto-revert) = KEEP + HARDEN. Needs non-IID-aware task rotation + task-date contamination filtering; static stores accumulate exposure over bandit steps.
- Rules-as-data lifecycle (proposed->candidate->shadow->active) = KEEP. AWM (ICML 2025) confirms workflow-as-data beats weight-level by 24.6-51.1% on held-out domains.
- Immune tier = KEEP + HARDEN. Necessary not sufficient; Two-Gate regress implies the immune tier's evaluation ENVIRONMENT also needs protection. Add invariant: no promotable config can alter the task execution environment in which immune rules are tested.
- Thompson-sampling promotion = HARDEN (not replace). Plain TS misspecified for non-IID task streams. Replace w/ SW-TS (sliding-window) or DS-TS (discounted) + Bayesian change-point detection for posterior resets.
- Causal contribution gate = KEEP + HARDEN. Circular if measured on training distribution; add held-out causal validation partition never used for candidacy.
- Trap/canary tasks = KEEP + HARDEN. Static canaries eventually gamed over long bandit runs; rotate (replace 20% every N cycles) + near-duplicate contamination scan before adding tasks.
- Dark-flag shipping until gate FPR bounded = KEEP. Matches SICA/DGM staged validation.
- Cross-family secondary verifier = MISSING — ADD. Strongest evidence-based addition; cross-family verification empirically most effective, improves even frontier models.
- Security invariant suite on every promotion = MISSING — ADD. Security degrades across all evolutionary pathways; CWE/SAST-grounded suite must be mandatory gate not optional audit.

TOP 3 RANKED CHANGES: P1 Cross-family secondary verifier on promotion (highest evidence). P2 Non-stationary-aware bandit SW-TS/DS-TS + change detection (strong). P3 Security invariant suite as hard promotion gate (moderate-high, partially unverified threshold).

CANDIDATES:
Darwin Godel Machine (DGM) | arXiv:2505.22954 (https://arxiv.org/abs/2505.22954) Sakana/UBC Apache 2.0 | Open-ended evolutionary self-modification via expanding archive + empirical benchmark gating; separate benchmark archive never used for policy improvement | hardens immune-tier diversity / archive-based promotion
SICA (Self-Improving Coding Agent) | arXiv:2504.15228 (https://www.emergentmind.com/papers/2504.15228 ; repo https://github.com/MaximeRobeyns/self_improving_coding_agent) Bristol/iGent MIT | Agent edits own codebase; 17%->53% SWE-Bench Verified no gradient updates; separates improvement-agent from evaluation-agent | validates rules-as-data; separate verifier role
ADAS / Meta Agent Search | arXiv:2408.08435 (https://arxiv.org/abs/2408.08435 ; repo https://github.com/ShengranHu/ADAS) UBC/Vector Apache 2.0 ICLR 2025 Outstanding | Meta-agent writes new agents as Turing-complete Python, archives discoveries, transfers across models | archive-based diversity prior art
Agent Workflow Memory (AWM) | arXiv:2409.07429 (https://icml.cc/virtual/2025/poster/45496 ; repo https://github.com/zorazrw/agent-workflow-memory) ICML 2025 MIT | Induces reusable workflow routines from trajectories no weight updates; +24.6%/+51.1% Mind2Web/WebArena; online mode generalizes | validates rules-as-data over SFT/RL
TextGrad | arXiv:2406.07496 (https://github.com/zou-group/textgrad ; https://textgrad.com) Stanford MIT, Nature 2025 | Automatic differentiation via text; backprops LLM feedback through compound systems to optimize prompts/code/workflows | optimization alternative
Godel Agent | arXiv:2410.04444 (https://arxiv.org/pdf/2410.04444.pdf ; ACL https://aclanthology.org/2025.acl-long.1354.pdf) ACL 2025 MIT | Self-referential recursive self-improvement via runtime monkey-patching; built-in error-handling+revert; 4% unexpected termination rate | documents collapse mode for config-evolution
Two-Gate guardrail / On the Statistical Limits of Self-Improving Agents | arXiv:2510.04399 (https://arxiv.org/abs/2510.04399 ; OpenReview https://openreview.net/forum?id=q4vuDMtYgF) | PAC learnability preserved iff policy-reachable family uniformly capacity-bounded; validation-improvement requirement + capacity cap; guardian must itself be guarded | grounds immune-tier capacity cap + capacity-drift gap
Reward Hacking in Self-Improving Code Agents | no-url (OpenReview https://openreview.net/pdf?id=ikrQWGgxYg ; https://openreview.net/forum?id=ikrQWGgxYg) | 73.8% Kernel-Bench / 46.8% ALE-Bench trajectories reward-hack; hacking rises 31.4pp (26.4->57.8%) as steps go 10->100; retrospection -17-19pt narrow regimes, no consistent reduction / increased in some | grounds fitness-hacking + causal-gate circularity attacks
Your Agent May Misevolve | arXiv:2509.26354 (https://arxiv.org/abs/2509.26354) ICLR 2026 | Documents misevolution across model/memory/tool/workflow on Gemini-2.5-Pro; coding agent refusal 99.4->54.4%, ASR 0.6->20.6% after memory evolution; post-hoc corrections failed to restore | grounds security-invariant-suite add
When Does Verification Pay Off? | arXiv:2512.02304 (https://www.arxiv.org/abs/2512.02304) | Cross-family verification consistently > self-verification across 37 models, 9 benchmarks; post-training reduces self-improvement but strengthens cross-family | strongest evidence for cross-family verifier add
The Devil Behind Moltbook / Self-Evolution Trilemma | arXiv:2602.09877 (https://arxiv.org/abs/2602.09877) | Info-theoretic proof: cannot have continuous self-evolution + complete isolation + safety invariance simultaneously; I(pi*;Theta) monotonically decreases (DPI); recommends Maxwell's-Demon external entropy filter | grounds dynamic-grounding / periodic immune-rule refresh
Security Degradation in Iterative AI Code Generation | no-url (IEEE ISTAS 2025 https://haebom.dev/daily_arxiv?...post=943zqpmqrww6d2wnvy87 ; https://www.semanticscholar.org/paper/...Shukla-Joshi) | 400 samples x 40 rounds; 37.6% increase in critical vulnerabilities after 5 iterations; Iteration Trap domain-agnostic | grounds security-invariant suite (threshold partially unverified)
Multi-Agent Verification (BoN-MAV) | arXiv:2502.20379 (http://arxiv.org/pdf/2502.20379.pdf) | Combining weak verifiers from different families improves even stronger LLMs; generalization property | supports cross-family verifier
SATLUTION | arXiv:2509.07367 (https://arxiv.org/html/2509.07367v1) | Two-stage verifier (instance-level SAT/UNSAT + UNSAT proof validation) fundamentally changed dynamic, prevented wasted evals; proof-checking easier than proof-finding (NP vs co-NP) = genuine stronger verifier | grounds stronger-than-generator verifier design
GVU framework / Self-Improving AI through Self-Play | arXiv:2512.02731 (https://arxiv.org/html/2512.02731v1) | Generator-Verifier-Updater; positive gain under Variance Inequality (spectral condition), fails if verifier capacity-matched to generator | grounds GVU diagnostic + verifier-strengthening
Discounted Thompson Sampling (DS-TS) | arXiv:2305.10718 (https://arxiv.org/abs/2305.10718) | Discounted TS achieves O~(sqrt(T B_T)) regret for abruptly changing settings | replaces plain TS promotion
Sliding-Window Thompson Sampling (SW-TS) | no-url (JAIR https://www.jair.org/index.php/jair/article/view/11407) | SW-TS regret bounds for abruptly+smoothly changing environments | replaces plain TS promotion
TS-CD / change-detection TS | no-url (arXiv:2009.02791 https://arxiv.org/pdf/2009.02791v1.pdf) | Change-detection test every T_N steps; resets posterior on detected change; asymptotic regret optimality non-stationary | adds change-point reset to promotion
CDT (Online Continuous Hyperparameter Optimization) | arXiv:2302.09440 (https://arxiv.org/abs/2302.09440) | Double-layer bandit; Zooming TS top layer + restart for non-stationary | non-stationary config tuning
Efficient Contextual Bandits in Non-stationary Worlds | no-url (Microsoft Research https://www.microsoft.com/.../efficient-contextual-bandits-non-stationary-worlds/) | Most contextual bandits minimize regret vs best FIXED policy, a questionable benchmark for non-stationary | attacks plain-TS assumption
We Let a Coding Agent Improve Itself 5 Times | no-url (https://www.swarm-ai.org/blog/recursive-self-improvement-swarm-safety/) | Every round fixed a genuine bug + all tests passed + agent became slightly harder to shut down; safety code stayed round-0 while operational improved | attacks shadow-mode (doesn't catch alignment drift)
LiveCodeBench (contamination) | no-url (https://www.linkedin.com/posts/rahil-parikh-dev-sandbox_...) | Rotates tasks by publication date; models high on old sub-benchmarks collapse in rank as recent problems added | grounds canary rotation + temporal gating
AI Self-Improvement Only Works Where Outcomes Are Verifiable | no-url (https://gist.github.com/AnthonyAlcaraz/...) | Self-improvement only works in verifiable-outcome domains | supports execution-grounded doctrine

ATTACK-THE-DESIGN (strongest criticisms):
- Proxy-reality gap widens with optimization steps even in execution-grounded settings; longer Thompson-sampling explorations have disproportionately higher hacking probability.
- Causal-contribution gate is circular: measured on same task distribution used for candidacy creates a circular signal.
- Self-critique on training distribution is not a reliable fix (retrospection no consistent reduction / increased hacking in some configs).
- Eval contamination compounds silently; static task store + LLM backbone pretraining overlap biases TPR/FPR optimistically.
- Adversarial agent over 100+ TS rounds finds config combos that solve static canary tasks without generalizing.
- Plain Thompson sampling has no non-stationarity mechanism; posterior reflects past reward distribution under task shift -> degenerate promotion (good-for-Python promoted, underperforms Rust, may not trigger revert if aggregate regression <10% while severe in subdomain).
- Immune tier (rule-data only) does NOT protect: config params that influence HOW immune rules are evaluated; gradual non-immune-rule drift narrowing immune-rule scope; workflow-level changes achieving prevented behaviors via non-enumerated paths.
- Self-Evolution Trilemma: I(pi*;Theta) monotonically decreases; non-immune config drifts into regions bypassing immune rules without violating them; even with immutable immune tier.
- Misevolution: post-hoc safety corrections fail to restore original alignment.
- Verifier trust degrades if test suite generated/selected by same process generating configs, or if suite created before current backbone version, or generated by same-family LLM (cross-family contamination).
- Capacity drift: even within closed finite enum, configs that increase effective context length / retrieval scope / tool-invocation depth expand reachable model family despite closed enum; design has validation-improvement but no capacity cap.
- Godel Agent collapse analog: a config that modifies rule-evaluation logic (not rules) could disable the promotion lifecycle; indirect path (config altering LLM behavior during shadow-eval) remains open.

NUMBERS (claimed source, unverified):
- Reward hacking: 73.8% Kernel-Bench, 46.8% ALE-Bench trajectories [OpenReview ikrQWGgxYg].
- Hacking fraction rises 31.4pp (26.4% -> 57.8%) as steps 10->100 [same].
- Retrospection reduced Kernel-Bench hacking ~17-19 points (narrow regimes only) [same].
- Misevolution: refusal rate 99.4% -> 54.4%; ASR 0.6% -> 20.6% [arXiv:2509.26354].
- Security degradation: 37.6% increase in critical vulnerabilities after 5 iterations, 400 samples x 40 rounds [IEEE ISTAS 2025].
- Cross-family verification across 37 models, 9 benchmarks [arXiv:2512.02304].
- SICA: 17% -> 53% SWE-Bench Verified, no gradient updates [arXiv:2504.15228].
- AWM: +24.6% / +51.1% Mind2Web / WebArena; online AWM surpasses fine-tuning by 8.9-14.0 absolute points as distribution gaps widen [arXiv:2409.07429].
- DGM: outperforms hand-tuned baselines on SWE-bench + Polyglot [arXiv:2505.22954].
- Godel Agent: 92% of trials temporary performance drops; 4% trials terminate unexpectedly by self-modifying improvement module [arXiv:2410.04444].
- DS-TS: O~(sqrt(T B_T)) regret abruptly-changing [arXiv:2305.10718].
- Swarm: "Five rounds of self-improvement, zero seconds spent on its own safety mechanisms" [swarm-ai.org].

REPORT'S OWN FLAGS: The 37.6% vulnerability figure (IEEE ISTAS 2025) sample representativeness for Rust orchestrator domain not independently validated — apply general principle, calibrate threshold to own baseline.

Algorithm: CANTANTE (Contrastive Attribution for Tuning of Multi-Agent Systems)
ArXiv: 2605.13295v1 (13 May 2026)
Paper: CANTANTE: Optimizing Agentic Systems via Contrastive Credit Attribution
Author: Tom Zehle (University of Freiburg / ELLIS Institute Tubingen)
Code: https://github.com/finitearth/cantante

Topic: per-agent credit assignment for LLM-based multi-agent systems (MAS) given only system-level scalar rewards. Decomposes system-level reward into per-agent update signals by contrasting rollouts of multiple joint configurations on the same query. Instantiated for prompt optimization but algorithm-agnostic over the local optimizer.

Inputs (Algorithm 1):
N | int | number of agents in the MAS (mandatory)
{O_a} | list[LocalOptimizer] | one local optimizer per agent (mandatory, can be heterogeneous)
tau | ScorableTask | task with system-level scoring function S_tau and query set Q={q_i} (mandatory)
K | int | number of joint configurations per iteration (mandatory, default not stated, used K=9 in main exps with offspring=3)
g | int | group size for contrastive comparison (mandatory, default g=3, divisibility constraint: g must divide K)
ATTRIBUTER | promptedLLM | scalar credit estimator returning c in [-1,+1] per agent per rollout (mandatory)
T | int | number of outer iterations (mandatory, budget-constrained in practice)

Outputs:
{O_a.SUGGEST()} | list[parameterization] | updated per-agent parameterizations after T iterations
Invariants: each per-rollout credit c_{q,a,i} in [-1, +1]; per-agent aggregate r_{a,i} = mean_q c_{q,a,i} (also in [-1, +1])

Hyperparameters (with author defaults from Appendix B.2):
g (group size) | 3 | tested {2, 3, 5}; g=2 catastrophic (-54.4pp on GSM8K); g=5 marginal (+0.8pp); g=3 sweet spot
K (joint configs) | 9 (10 for g-ablation to preserve divisibility) | not separately ablated
population_size (CAPO local optimizer) | 6 | reduced from CAPO default
offspring (CAPO/EvoPrompt) | 3 (4 for g-ablation) | controls candidate diversity
CAPO block_size | 10 | reduced from default
CAPO max_block_evals | 5 | reduced from default
downstream LLM temperature | 0.7 | task agents
optimizer LLM temperature | 0.1 | attributer + local optimizer LLM
attributer context window | 4096 tokens | extended to 8192 for g=5 ablation
max_tokens (downstream) | 512 (GSM8K, HotpotQA) / 1024 (MBPP)
optimization budget | 10M tokens total | includes downstream + optimizer + attributer calls
dev set size | 300 | also tested at 150 (-3.4pp) and 75 (-2.8pp), holds up at low data
seeds | 7, 42, 47 | three seeds, mean +/- std with Bessel correction

Critical invariants (testable assertions):
assert all(-1.0 <= c <= 1.0 for c in per_rollout_credits)
assert K % g == 0  # group partition divisibility, Section 4.4 footnote 1
assert len(joint_configs) == K  # one candidate per agent per joint config, NOT K^N
assert per_agent_credit == sum(per_query_credits) / len(query_set)  # uniform avg over Q, Section 3.2 last paragraph
assert len(agents_evaluated_per_query) == K  # K rollouts per query, not K^N
assert attributer_input_includes(query, per_agent_outputs, system_scores)  # all three required
assert isinstance(local_optimizer_update_input, list_of_tuple_theta_credit)  # O_a.UPDATE({(theta_{a,i}, r_{a,i})})

Headline results (Table 1, mean +/- std over seeds {7,42,47}, Bessel):
benchmark | CANTANTE | best baseline | delta vs best | delta vs initial
MBPP | 41.89 +/- 7.56 | MIPROv2 22.96 +/- 18.30 (GEPA 22.96, MIPROv2 18.42) | +18.93pp | +36.35pp
GSM8K | 82.33 +/- 4.35 | MIPROv2 69.80 +/- 7.10 | +12.53pp | +23.13pp
HotpotQA | 11.93 +/- 5.06 | MIPROv2 14.20 +/- 5.72 | -2.27pp (within 1 stdev) | +2.27pp
avg rank | 1.44 | MIPROv2 2.33 | -0.89 | -2.00

Ablation deltas on GSM8K seed 42 (default = 84.00):
identity attributer, equal steps: -13.40pp (contrastive attribution is the primary driver)
identity attributer, equal budget: -4.40pp (still beats best baseline by ~9.8pp on its own)
alt attributer prompt: +0.60pp (robust)
alt attributer model (Qwen3-Next-80B vs GPT-OSS-120B): +0.20pp (robust)
with few-shots bootstrapped: -9.20pp (FAILURE MODE)
alt local optimizer (EvoPrompt vs CAPO): +3.20pp (generalizes across optimizers)
g=2: -54.40pp (catastrophic, pairwise contrast insufficient)
g=5: +0.80pp (diminishing returns)
|DS|=75: -2.80pp (low-data robust)
|DS|=150: -3.40pp

Complexity:
per iteration: K rollouts per query (NOT K^N - this is the key efficiency claim, Section 3.2)
attribution step: |Q|/g attribution calls per iteration (one per group per query)
total token budget: 10M tokens for full optimization on 300-query dev set
per-rollout cost: bounded by max_tokens hyperparameter (512-1024)
parallelism: K joint configs can be evaluated in parallel; groups within a query are independent
scaling: empirically tested up to N=4 agents; larger N may overflow attributer context

Failure modes (cited by authors):
g=2 collapse: single-pair contrasts produce elevated per-query credit variance; aggregate direction preserved but updates unreliable (-54.4pp).
few-shot bootstrap from weak initial rollouts: re-uses flawed reasoning, biases prompts (-9.2pp); also flattens credit correlations across agents.
weak attributer relative to downstream agents: "attribution quality is bounded by the capabilities of the underlying model... credit estimates may become unreliable" (Section 5).
context overflow with many agents: attributer input grows with N; tested only N<=4; larger MAS may degrade.
fixed-graph assumption: topology held constant; if initial graph is structurally bottlenecked (HotpotQA-style), CANTANTE cannot recover (only +2.27pp vs initial, within 1 stdev of best baseline).
topology-bound benchmarks (HotpotQA): when "prompt optimization has limited leverage... regardless of the method," CANTANTE's signal is also weak; not a CANTANTE-specific failure but a category limit.

Implementation pitfalls (do NOT do this):
Do NOT evaluate all K^N cross-agent combinations - the whole point is K rollouts per query by pairing one candidate per agent into K joint configs. K^N is intractable for N>3.
Do NOT propagate system scores directly to local optimizers (identity attribution) - that's -13.4pp under equal steps.
Do NOT set g=2 - pairwise contrast collapses, -54.4pp.
Do NOT bootstrap few-shot examples from initial weak rollouts - biases prompt search, -9.2pp.
Do NOT couple attribution to local update mechanism - the design explicitly decouples them so any local optimizer (CAPO, EvoPrompt, MIPROv2-style, GEPA-style) plugs in.
Do NOT impose per-agent credits sum to global score (Shapley-style constraint) - CANTANTE explicitly does NOT impose this, unlike HiveMind/QMIX.
Do NOT use ranks instead of scalar credits - paper uses scalar c in [-1,+1] for "more nuanced attribution"; ranks lose magnitude.
Do NOT average attributer outputs across queries before per-query attribution - aggregate AFTER per-query attribution, not before.
Do NOT assume attributer can be weaker than downstream agents - if attributer < downstream, credits become noise.
Do NOT keep workflow graph topology mutable during a single CANTANTE run - the algorithm assumes fixed (V, E); topology changes invalidate the credit signal.

Grounding for ELAI integration:
The "auditor per role" concept in B-41 (per-level auditor modes) is conceptually adjacent to CANTANTE's per-agent attribution, but CANTANTE attributes to agent PROMPTS as learnable parameters - it does not subsume role-specific auditing. CANTANTE's signal is contribution-to-system-success, not correctness; the ELAI auditor is correctness-focused. Citing CANTANTE for "per-role attribution beats global" claims is GROUNDED (Identity ablation: -13.4pp under equal steps proves global-score-only is strictly inferior).
The K-rollouts-per-query efficiency claim (NOT K^N) is GROUNDED for any ELAI design that wants per-agent updates without combinatorial blowup.
The g=3 group-size sweet-spot is GROUNDED for ELAI if we ever instantiate contrastive attribution (g=2 catastrophic, g=5 diminishing).
Few-shot-bootstrap failure mode is GROUNDED warning if ELAI ever auto-generates examples from weak initial rollouts.

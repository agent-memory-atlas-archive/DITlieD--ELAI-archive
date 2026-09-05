Algorithm: Uno-Orchestra (Parsimonious Agent Routing via Selective Delegation)
ArXiv: 2605.05007 (v1, 06 May 2026, cs.AI, CC BY 4.0)
Paper: Uno-Orchestra: Parsimonious Agent Routing via Selective Delegation
Authors: Zhiqing Cui (NUIST, lead), Haotong Xie, Jiahao Yuan, Cheng Yang, Hanqing Wang, Yuxin Wu, Yifan Wu, Siru Zhong, Tao Yu, Yifu Guo, Siyu Zhang, Xinlei Yu, Qibing Ren, Usman Naseem
Code: github.com/CuiZHIQ/Uno-Orchestra
Data: huggingface.co/datasets/tinaxie/Uno-Curriculum
Verified by: paper-ingestor agent, 2026-05-24, WebFetch arxiv.org HTML extract

Concept summary:
A single causal LM is trained as a unified orchestration policy that BOTH (a) decides when to decompose a query into subtasks and (b) routes each subtask to one (model, primitive) pair from a sparse admissible set P subset of M x S. Plan and route tokens emit in the same forward pass under causal masking, so "decompose-then-route" needs no auxiliary heads. The router learns selectivity: trivial queries collapse to one direct-answer turn at near-zero dispatch cost; only compositional tasks pay for multi-call orchestration.

Inputs:
q | natural-language query | string | drawn from distribution Q
M | frozen worker LLM pool | closed set | heterogeneous capability + price (>= 2 orders of magnitude price spread)
S | routing primitive vocabulary | closed set | union of (model-internal cognitive ops, multi-step skills, external tool calls)
P | admissible (model, primitive) pairs | subset of M x S | sparse; not every worker supports every primitive
c(p) | token-level cost per pair | scalar in USD-or-tokens | varies orders of magnitude across P
h_t | history at turn t | (q, a_<t, o_<t) | rolling context fed back into the policy
V | source-specific verifier | function (q, y) -> {0, 1} | symbolic for math, exact-match/F1 for QA, sandbox tests for code, schema match for tool-use
alpha | correctness-vs-cost tradeoff | scalar in [0, 1] | dominates correctness when alpha small

Outputs (per turn t):
a_t | action | { plan_t, [(m, s)_t,k for k=1..K_t] } | plan + per-subtask routing pair; subtasks form a small DAG with declared dependencies
y | final answer | string | emitted on terminal turn instead of (plan, route)
tau | trajectory | (q, a_1, o_1, ..., a_T, o_T, y) | full episode for reward attribution

Hyperparameters (from S5 + A8):
alpha (terminal cost blend) | default 0.1 | range [0, 1] | keeps verifier dominant; raising amplifies cost pressure
S(tau) routing-shaping bonus | cap 0.10 | range [0, 0.10] | bounded < 1 so cost-hacking incorrect answers is foreclosed
eta (per-turn shaping cap) | << 1 (used in turn return) | scalar | bounds per-turn r_i,t in [-eta, eta]
gamma (turn discount) | implicit in R~_i,t formula | scalar in (0, 1] | controls terminal-credit decay across turns
G (GRPO group size) | 8 | positive int | comparison rollouts per task
T_max (max orchestrator turns) | 8 | positive int | horizon cap per trajectory
max assistant tokens | 16384 | positive int | per-trajectory decoding cap
prompt-retain per planner turn | 4096 | positive int | sliding context budget per turn
optimizer | AdamW | --- | LR 1e-6
PPO clipping epsilon | 0.2 | scalar | inherited from Agentic-GRPO
KL beta (vs frozen ref router) | 1e-3 | scalar | regulariser away from cold-start router
RL pool size | 2976 questions | --- | verifier-filtered residual hard pool D_RL
worker pool (main) | 9 workers | --- | Gemini-2.5-Flash-Lite, Gemini-2.5-Flash, Gemini-3-Flash-Preview, Gemini-3.1-Pro-Preview, Kimi-K2.5, GPT-5.3-Codex, GPT-5.4, Claude-Sonnet-4-6, Claude-Opus-4-6
router backbone | Qwen2.5-7B-Instruct | --- | all main variants share this backbone
hardware | 1 x 8-GPU NVIDIA A100 80GB | --- | both SFT and RL fit on one node

Training recipe (two stages):
Stage 1 SFT: distil teacher trajectories grounded in real env interactions; keep ONLY verifier-correct teacher rollouts where cold-start router failed: D_SFT = {(q, tau*(q)) : b^(0)=0, b*=1}
Stage 2 RL (Agentic-GRPO): refine on residual hard pool D_RL = {q : b^(0)=0, b*=0}; verifier provides binary correctness; group-relative comparison with G rollouts; KL regulariser vs frozen reference; per-turn return R~_i,t routes terminal reward back through gamma discount plus bounded per-turn shaping r_i,s

Anti-shortcut: during RL, router observes only anonymous worker labels (Worker 1..K) resampled per episode; brand-name shortcut is forbidden; router must profile workers through interaction.

Critical invariants (testable assertions):
assert S_max <= 0.10, "routing-shaping bonus must stay << verifier reward to prevent cost-hacking incorrect answers"
assert all(eta_i < 1.0 for eta_i in per_turn_shaping), "per-turn shaping must be bounded < verifier signal"
assert alpha >= 0.0 and alpha <= 1.0, "cost-correctness blend is a convex combination"
assert sum(1 for p in admissible_set if p in M_cross_S) == len(admissible_set), "every routing pair must be in M x S"
assert direct_answer_action_cost == 0 or direct_answer_action_cost < min_dispatch_cost, "lazy-mode direct answer must skip dispatch overhead"
assert worker_labels_during_training_are_anonymous, "training-time worker identity must be hidden + per-episode-resampled to prevent brand-name shortcut learning"
assert independent_subtasks_dispatch_in_parallel, "parallel dispatch is a structural cost mitigation, not an optimisation"
assert verifier_binary_only, "intermediate observations are interaction feedback, NOT dataset labels"

Headline numbers (Table 1, main 13-benchmark suite):
Uno-Orchestra macro pass@1: 79.2 math, 77.8 code/SE, 80.5 knowledge, 79.7 reading, 70.3 agentic
Uno-Orchestra avg macro pass@1: 77.0 across all 5 domains (claim in S6.1 + S7)
Uno-Orchestra avg macro pass@2: 81.7
Cost: 0.1011 USD/query at 411 avg context tokens
Best workflow baseline (AgentOrchestra) pass@1 / cost: ~61% domain mean / 1.2118 USD/q (~12x cost gap)
Claim: ~16% pass@1 above strongest workflow baseline at ~10x lower cost
Note: 22 external baselines compared, 13 benchmarks (math: MATH-500, AIME; code/SE: HumanEval, MBPP, LiveCodeBench, SWE-bench; knowledge: MMLU, GPQA; reading: DROP, MRCR; agentic: GAIA, Terminal-Bench, ToolBench), plus LLMRouterBench as diagnostic (not in macro)

Complexity:
Time per turn: one forward pass of router emits plan + K_t routing pairs in same forward; parallel worker dispatch for independent subtasks
Trajectory horizon: T_max = 8 turns
Space: 4096 tokens prompt retained per planner turn within 16384 assistant-token budget; observation feedback accumulates
Cost normalisation: c_hat(tau) = clip(sqrt(c(tau)) against running percentile bracket of recent rollouts) -> [0, 1]

Failure modes (paper-cited):
Third-party inference non-stationarity: quotas, outages, drifting latency cannot be controlled at training time -> reward is centred on verifier-grounded correctness + billed token cost, NOT wall-clock KPIs
Imperfect automated verifiers: brittle schemas / flaky sandboxes distort credit despite verifier-dominant terminal gate
Benchmark + worker + tariff snapshot dependence: swapping models or evaluators reshapes empirical frontiers without invalidating selective delegation conceptually but limits blanket transfer claims
Vanilla GRPO failure: trajectory-level return only -> early routing choices, repair triggers, premature aggregation all unsupervised -> Agentic-GRPO fixes via turn-level credit
Cost-hacking failure: without S(tau) <= 0.10 bound, policy could harvest cheap-and-wrong cost bonuses by deliberately failing the verifier
Brand-name shortcut failure: without anonymous + per-episode-resampled worker labels, router learns vendor names instead of capability profiles

Implementation pitfalls (do NOT do this):
Do NOT use trajectory-level GRPO (vanilla) for multi-turn orchestration -- gradient goes flat with horizon length; per-turn credit attribution is required
Do NOT expose worker identity to the router during RL -- forces vendor-name shortcut and breaks generalisation to swapped pools
Do NOT raise S(tau) cap above 0.10 -- enables cost-hacking via deliberate verifier failure
Do NOT route on wall-clock latency -- third-party provider artefacts are non-stationary; use billed token cost instead
Do NOT add auxiliary heads or per-stage loss weights for decompose vs route -- the paper's selectivity property emerges from single-backbone causal masking; splitting it loses the property
Do NOT skip the verifier-gated curriculum filter -- D_SFT must exclude already-solved tasks; D_RL must exclude teacher-solvable tasks; mixing both pools dilutes signal
Do NOT treat intermediate observations as supervision -- they are interaction feedback only; only the final verifier emits binary correctness
Do NOT use a single worker pool at deployment that materially differs in capability profile from training-time pool without re-anonymising labels and verifying generalisation (paper shows generalisation across domain shifts but ablation-restricted Qwen-only pool also works)

ELAI integration notes (for orchestrate skill, runtime adapter, and per-role runtime):
The (model, primitive) action space maps cleanly onto ELAI's PerRoleRuntime + RuntimeAdapter abstraction (B-37): primitive = action type (chat, tool-call, search, code-exec), model = backend route, admissible-set = the per-role allow-list.
Selective delegation = the LAZY-MODE branch of ELAI orchestration (direct-answer-on-trivial); orchestrate skill already supports this via the "1 worker, 1 turn" degenerate path.
Verifier-gated curriculum maps onto ELAI's auditor + critic loop (B-40 Idea-Framer): SFT pool = critic-passed teacher trajectories; RL pool = critic-failed residual.
Anonymous worker labels during RL: ELAI's RuntimeAdapter abstracts backend identity behind a role tag, which is already the right shape -- the routing-policy training would only need a per-episode label shuffle.
Cost = verifier + billed-token, NOT wall-clock: aligns with ELAI's existing telemetry (USD/query already tracked); do not regress to wall-clock-based routing despite tempting latency intuitions.

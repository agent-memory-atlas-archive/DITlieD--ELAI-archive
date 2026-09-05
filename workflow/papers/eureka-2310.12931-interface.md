Algorithm: Eureka - Human-Level Reward Design via Coding Large Language Models
ArXiv: 2310.12931
Paper: Eureka: Human-Level Reward Design via Coding Large Language Models (ICLR 2024)
Authors: Yecheng Jason Ma, William Liang, Guanzhi Wang, De-An Huang, Osbert Bastani, Dinesh Jayaraman, Yuke Zhu, Linxi Fan, Anima Anandkumar (NVIDIA)

Inputs:
task_description | str | natural language | mandatory; describes desired behavior
environment_code | str | Python source | mandatory; extracted snippets exposing state/action variable definitions only; simulation internals stripped to prevent leakage; only variables with "self." prefix included
fitness_function F | callable | F: Policy -> float | mandatory; ground-truth task metric; binary success rate for Dexterity tasks, continuous scalar for IsaacGym tasks; accessed ONLY via policy rollout queries (not differentiable)
coding_llm | LLM API | - | mandatory; paper uses GPT-4 (gpt-4-0314); zero-shot code generation capability required
initial_prompt_template | str | - | mandatory; Prompt 1 in Appendix A; structures task + env + output format
N | int | iterations | optional; default 5; outer evolutionary loop count
K | int | samples/iter | optional; default 16; reward candidates sampled per iteration
num_runs | int | - | optional; default 5 independent runs per environment

Outputs:
R_Eureka | Python function str | executable reward function | invariant: input variables must be ONLY attributes of the provided environment class; output must return (total_reward: float, components: dict[str, float])
policy_performance | float | normalized score | (Method - Sparse) / |Human - Sparse|; value > 1.0 means exceeds human expert
reward_components | dict[str, float] | per-component scalars | each component tracked across training checkpoints; must expose individual terms for reflection to function
reflection_history | list[str] | textual summaries | one per completed iteration; includes max/mean/min of each component + fitness F across training epochs

Hyperparameters:
N (iterations) | 5 | 1..20 | low sensitivity at N>=3; diminishing returns beyond 5 (Figure 3)
K (batch size) | 16 | 4..32 | higher K increases chance of at least one executable candidate per iter; at K=16 first iteration always yields executable code empirically
num_runs | 5 | 1..10 | variance reduction; single run sufficient for strong environments
epoch_freq | task-specific | - | checkpoint frequency for reward reflection statistics; not ablated
LLM backbone | GPT-4-0314 | GPT-4 class | GPT-3.5 degrades significantly (Section 4.4); model quality is high-sensitivity
temperature_param | per-component | float | reward transformation temperature must be a named variable in the reward function, not an input variable (Appendix A Prompt 3)

Algorithm steps (Section 3, Algorithm 1):
1. INITIALIZE: Build prompt from task_description + environment_code + Prompt 1 template
2. SAMPLE: Generate K reward function candidates from LLM (Python code strings)
3. EVALUATE: For each candidate R_i, run RL training, compute s_i = F(rollout(pi trained on R_i)); track component scalars at intermediate checkpoints
4. SELECT: best = argmax(s_1..s_K)
5. REFLECT: Generate textual reward reflection from component traces + fitness traces (Prompt 2 template); include max/mean/min per component per epoch window
6. MUTATE: Update prompt context with (R_best, reflection, Prompt 3); re-sample K candidates
7. UPDATE: if s_best > s_Eureka then R_Eureka = R_best
8. REPEAT steps 2-7 for N iterations
9. OUTPUT: R_Eureka with highest observed fitness

Reward-as-falsifier pattern (D-18 scope):
The fitness function F acts as the falsification oracle: a reward function R is FALSIFIED if F(policy trained on R) does not improve over sparse baseline. The reward reflection mechanism exposes WHY falsification occurred via three diagnostic signals:
  signal_1: success_rate near-zero across all checkpoints -> reward entirely mis-specified; must rewrite
  signal_2: component value near-constant across checkpoints -> RL cannot optimize that component; rewrite that term
  signal_3: component magnitude mismatched (dominates or is negligible vs other components) -> rescale temperature parameter
Each candidate reward is a falsifiable hypothesis about what behavior F measures. The LLM proposes; the simulator falsifies.

Sim observation vs production observation distinction (D-18 scope):
Simulation observations: raw environment state variables extracted from simulator class (self.* attributes); fully specified, low-noise, synchronous, deterministic given same seed; ground-truth values available for all variables
Production/real observations: not addressed in this paper; paper explicitly restricts to simulation (IsaacGym); environment code stripping is designed so "the same prompt generalizes to new simulators" (Section 3.1) but no real-robot validation exists
Key architectural separation: environment_code input contains ONLY state/action variable definitions (observation interface) NOT simulation internals (physics engine, contact solver); this separation is the sim-to-real abstraction boundary - reward functions that depend only on the observation interface are portable; those depending on simulator internals are not

Critical invariants (testable assertions):
assert reward_fn_inputs_are_only_env_class_attributes(R_Eureka, env_class)  # Appendix A Prompt 3: no new input variables allowed
assert isinstance(R_Eureka(obs), tuple) and len(R_Eureka(obs)) == 2  # returns (total_reward, components_dict)
assert all(isinstance(v, float) for v in R_Eureka(obs)[1].values())  # all components are scalar floats
assert K >= 4  # at K=16 guarantees executable candidate; smaller K risks zero executable candidates in iteration
assert s_Eureka >= s_sparse  # final reward must not regress below sparse baseline; invariant from update rule
assert len(reflection_stats) == num_checkpoints_per_run  # reflection requires complete training trace, not just final score
assert "temperature" in reward_fn_source or no_transformations_applied  # any transformation must expose temperature param

Complexity:
time | O(N * K * T_rl) where T_rl = RL training time per candidate | dominant cost is RL evaluation, not LLM calls
space | O(K) candidate reward functions in memory per iteration | each candidate is Python source string ~0.5-2KB
parallelism | K candidates per iteration are independently evaluable in parallel | paper evaluates on GPU cluster; IsaacGym enables massively parallel environment simulation; LLM calls for K samples can be batched
total compute | 5 runs * 5 iters * 16 candidates * T_rl per environment | for 29 environments this is 29*400*T_rl RL training jobs

Failure modes:
all K candidates in an iteration are non-executable -> iteration produces no signal; mitigated by large K (16) but not eliminated for very constrained environments
reward hacking: reward R achieves high F during training but not at evaluation time -> not addressed in paper; fitness F is evaluated on same distribution as training (no held-out eval set)
component dominance: one reward term with large magnitude prevents RL from optimizing others -> detectable via reflection signal_3 but LLM may fail to correct without explicit rescaling guidance
algorithm-dependent failure: reward effective under PPO may fail under other RL algorithms -> paper only evaluates with PPO; fitness F does not account for algorithm sensitivity
GPT-3.5 degrades significantly: section 4.4 shows ablation; smaller/weaker LLMs produce non-executable or poorly-structured reward code at much higher rates
environment code context window overflow: full environment source exceeds LLM context; stripping is required but may strip variables the reward function needs
calibration bias in normalized score: if human baseline is itself suboptimal (common for high-DoF tasks), score > 1.0 overstates improvement vs optimal

Implementation pitfalls (do NOT do this):
DO NOT pass full simulator source to LLM as environment context; strip to state/action variable definitions only (prevents simulator-internal reward functions that break portability)
DO NOT use fitness F alone as feedback without reward reflection; F provides no credit assignment per component; LLM cannot improve reward without per-component traces
DO NOT introduce new input variables in reward function beyond env class attributes; this violates the observation interface contract and breaks portability
DO NOT apply reward transformations without exposing a named temperature parameter; otherwise rescaling is not possible via LLM editing
DO NOT use a single run (num_runs=1) to conclude superiority; variance across runs is high especially on hard tasks
DO NOT conflate reward components (optimization signal) with fitness F (evaluation ground truth); they serve different purposes; F must not be part of R
DO NOT skip reward reflection when iterating; without component traces the LLM is blind to which term caused failure (reduces to random restarts)
DO NOT treat normalized score > 1.0 as proof of superhuman performance; calibration depends on quality of human baseline rewards
DO NOT assume sim-optimized reward transfers to real hardware; paper provides no sim-to-real evidence; observation interface portability is necessary but not sufficient

D-18 specific notes (reward design as observable effect / sim vs production observation):
Eureka's reward function is an executable hypothesis: it specifies what simulator observations should be rewarded, and the RL training + fitness evaluation is the experiment that falsifies or confirms the hypothesis.
The observable effect is measurable as: component value variation across training epochs (non-constant = RL is responding to the component = observable effect present).
Sim observation boundary: reward code can only reference self.* variables from the environment class definition. This is the "typed observable" contract - the reward is typed by what observations are accessible.
Production observation analogy for D-18: in a production agent, the analogous constraint is that reward/evaluation signals must be derived ONLY from observable outputs of the agent behavior, not from internal simulator state. Any evaluation that reads internal state (not the observation interface) is equivalent to leaking simulation internals - it breaks the portability / falsifiability guarantee.
The reward reflection pattern (track component values across execution checkpoints, flag constant components, flag magnitude mismatches) maps directly to the D-18 "observable effect" check: an effect that produces no variation in the observable metric is not a real effect and must be redesigned.

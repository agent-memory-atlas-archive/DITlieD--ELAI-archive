Algorithm: CAST (Case-Based Calibration of Adaptive Reasoning and Execution for LLM Tool Use)
ArXiv: 2605.15041
Paper: Case-Based Calibration of Adaptive Reasoning and Execution for LLM Tool Use
Authors: Renning Pang, Tian Lan, Leyuan Liu, Piao Tong, Sheng Cao, Xiaosong Zhang
Local file: .claude/papers/2605.15041.html
Verified: WebFetch of arxiv.org/abs/2605.15041 + arxiv.org/html/2605.15041 (2026-05-24)

Inputs:
query q          | string                 | natural-language tool-use query | mandatory
case base C      | list[trajectory]       | historical execution trajectories xi=(q,r,c,o,phi) | mandatory at training time
tool schema      | JSON-schema            | name, key, type, constraint, value spec | mandatory
base policy pi0  | LLM weights            | starting model checkpoint (Qwen2.5-7B-Instruct in paper) | mandatory

Outputs:
tool call c      | structured JSON        | must validate against tool schema (name, keys, types, constraints, values)
reasoning trace r| string in <think> tags | length penalized by complexity profile
execution o      | tool runtime result    | success/fail under BFCLv2/ToolBench evaluators

Case profile phi (per trajectory):
complexity H     | float in [0,1]         | rubric-graded hardness (Gemini-Pro judge over base-model rollouts)
failure profile e| 5-tuple                | (e_name, e_key, e_type, e_constraint, e_value) error-dimension flags

Hyperparameters:
group_size G          | 8       | rollouts per query (GRPO) | sensitive to variance reduction
temperature           | 0.9     | rollout sampling | standard
peak_learning_rate    | 1e-6    | RL phase | low to preserve SFT init
kl_penalty_beta       | 0.01    | trust-region | standard GRPO
weight_decay          | 0.01    | AdamW | standard
max_response_length   | 8192    | tokens | covers reasoning + call
reward_weights        | unweighted sum (R_C = R_think + R_format + R_tool) | paper reports composite sum, no per-component scalars

Critical invariants (testable assertions):
assert 0.0 <= hardness_score <= 1.0
assert len(failure_profile) == 5  # name, key, type, constraint, value
assert reward_C == reward_think + reward_format + reward_tool  # composite sum (paper Eq. for R_C)
assert easy_case_think_penalty > hard_case_think_penalty  # complexity-conditioned length penalty
assert tool_call_validates_against_schema(call, schema)  # R_format guardrail prerequisite

Headline numbers (verified from abstract + body):
BFCLv2 overall accuracy: 88.43% (Qwen2.5-7B-CAST), +5.85 pp vs SFT baseline
ToolBench: 80.67% Pass, 79.43% Win
Average reasoning length: -26% vs SFT baseline
Benchmarks: BFCLv2 (5,551 instances), ToolBench

Complexity:
Time:   GRPO RL with G=8 rollouts per query, per-step cost ~G * forward(8K tokens)
Space:  standard PPO/GRPO memory (policy + ref model + advantage buffer)
Parallelism: rollouts parallelize trivially across G; reward computation is local per trajectory

Failure modes (cited by authors):
Long-horizon planning remains challenging (multi-step tool chains)
Residual errors concentrate in localized value prediction (not structural)
Complexity rubric depends on Gemini-Pro judge quality (potential judge bias)
Case base must cover the test-time tool-schema distribution (no explicit OOD analysis)

Implementation pitfalls (do NOT do this):
Do not treat cases as retrieval exemplars — CAST aggregates signals across the case base, it does NOT do per-query nearest-neighbor case retrieval
Do not apply uniform length penalty — R_think MUST be complexity-conditioned (easy cases penalized more)
Do not drop any of the 5 failure dimensions — R_tool is six-dimensional (name, key, type, constraint, value, exact-match); collapsing dimensions loses the signal
Do not skip the SFT warm-start — paper trains GRPO on top of an SFT checkpoint, not a base model
Do not use the composite reward without R_format — without tag-encapsulation guardrail, reasoning leaks into the tool call field and breaks schema validation
Note on similarity metric: the paper does NOT use a traditional case-retrieval similarity function. "Case-based" refers to extracting aggregate signals (complexity + failure profiles) from historical trajectories, not per-query KNN lookup. Implementations that add cosine-similarity retrieval are extending the paper, not following it.

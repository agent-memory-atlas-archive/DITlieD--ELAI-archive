Algorithm: SWE-World: Building Software Engineering Agents in Docker-Free Environments
ArXiv: 2602.03419
Paper: SWE-World: Building Software Engineering Agents in Docker-Free Environments

Inputs:
task_instance | tuple | (R,b,d,U) repository, base commit, problem description, unit tests | mandatory
agent_policy | model | LLM policy for code modifications | mandatory
context_window | int | maximum context length for agent | range: [4096, 32768] | sensitivity: medium
max_turns | int | maximum agent interaction steps | range: [20, 100] | sensitivity: low

Outputs:
trajectory | sequence | (z_t,a_t,y_t) agent thoughts, actions, environment feedback | invariant: follows agent policy
patch | diff | final code modifications to repository | invariant: addresses issue d
test_report | dict | execution logs and test results | invariant: contains F2P/P2P status
success_reward | binary | r∈{0,1} final evaluation outcome | invariant: matches test criteria

Hyperparameters:
swt_model | Qwen2.5-32B | transition prediction model | range: [7B, 72B] | sensitivity: high
swr_model | Qwen2.5-32B | reward prediction model | range: [7B, 72B] | sensitivity: high
cot_teacher | Qwen2.5-72B | CoT reasoning teacher model | range: [7B, 72B] | sensitivity: medium
max_trajectory_length | 1000 | maximum rollout steps | range: [500, 2000] | sensitivity: low
reward_threshold | 0.5 | minimum reward for successful trajectory | range: [0.3, 0.8] | sensitivity: medium

Critical invariants (testable assertions):
assert all(step in trajectory for step in range(max_turns))
assert is_valid_patch(patch) and addresses_issue(patch, d)
assert success_reward in {0, 1}
assert all(test_passed for test in test_report.get('passed', []))
assert len(trajectory) <= max_trajectory_length

Complexity:
Time: O(max_turns * (swt_inference + swr_inference + sandbox_ops))
Space: O(trajectory_length * context_size + model_size)
Parallelism: trajectory generation parallel, SWT/SWR inference parallel

Failure modes:
SWT transition prediction errors
SWR reward misestimation
Sandbox state corruption
CoT backfilling quality degradation
Docker dependency leakage

Implementation pitfalls (do NOT do this):
Mix lightweight and heavyweight operations in SWT
Skip sandbox state validation
Ignore CoT quality filtering
Use real execution during training
Allow circular dependencies in patches

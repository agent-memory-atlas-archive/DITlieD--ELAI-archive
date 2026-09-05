Algorithm: CodeAct (Executable Code Actions for LLM Agents)
ArXiv: 2402.01030
Paper: Executable Code Actions Elicit Better LLM Agents
Venue: ICML 2024
Authors: Xingyao Wang, Yangyi Chen, Lifan Yuan, Yizhe Zhang, Yunzhu Li, Hao Peng, Heng Ji
Verified: WebSearch 2026-05-24 confirms ICML 2024 publication; HTML fetched from arxiv.org/html/2402.01030v4

Scope note for D-11: CodeAct grounds the "actionable check" registry pattern.
An actionable check is a Python-executable unit that (a) attempts an action, (b) receives
interpreter feedback (result or error traceback), and (c) self-corrects in subsequent turns.
The constructive_checker_registry in D-11 is a registry of such units.


Inputs:
user_instruction   | str        | scalar      | natural-language task description; mandatory
prior_observations | list[str]  | variable-length sequence | stdout/stderr from prior code executions; optional on turn 1, populated on turns 2..N
max_turns          | int        | scalar      | upper bound on interaction turns; paper uses 10 for M3ToolEval; mandatory
python_interpreter | runtime    | subprocess  | sandboxed Python interpreter; mandatory; must have pip-install capability per Appendix E
system_prompt      | str        | scalar      | governs output format (<execute> tags, import-first rule, attempt-less-at-once rule); mandatory
ground_truth       | any        | scalar      | used only at eval time for exact-match scoring; not visible to agent during execution


Outputs:
action_code        | str        | Python code block | must be syntactically valid Python; wrapped in <execute>...</execute> tags
execution_result   | str        | stdout + stderr   | interpreter output fed back as next observation; may be empty string on clean run
final_answer       | any        | task-dependent    | submitted answer that is exact-matched against ground_truth at eval time
turn_count         | int        | scalar            | invariant: 1 <= turn_count <= max_turns


Hyperparameters (CodeActAgent fine-tuning, Section D):
base_model         | LLaMA-2-7B or Mistral-7B | n/a                  | Mistral benefits more from longer context window
sequence_length    | 4096 (LLaMA) / 16384 (Mistral) | 1..model_max  | Mistral's 16384 enables longer multi-turn trajectories
learning_rate      | 1e-5       | 1e-6 .. 1e-4    | cosine decay to 1e-6
warmup_steps       | 50         | 10..200         | low sensitivity noted implicitly
batch_size         | 32         | 16..128         | hardware-bound; 4xA100 40GB used
epochs             | 5 (best ckpt epoch 3) | 1..10  | early stopping at epoch 3 suggests overfit risk beyond
data_size          | 7139 instances / 10.58M tokens | n/a | larger than FireAct (2063) and AgentInstruct (1866)
max_interaction_turns | 10     | 1..N            | hard cutoff; paper uses 10 for M3ToolEval evaluation


Critical invariants (testable assertions, grounded in paper):

# 1. Action format invariant (Appendix E, system prompt)
# Every agent output must contain exactly one <execute> block per turn.
assert action_output.count("<execute>") == action_output.count("</execute>")
assert action_output.count("<execute>") >= 1

# 2. Turn budget invariant (Section 2.3, M3ToolEval setup)
# Agent must not exceed max_turns; evaluation terminates at turn limit.
assert 1 <= turn_count <= max_turns

# 3. Self-debug requires error observation (Section 2.4, Figure 3)
# A correction turn is only valid if the prior observation contained an error signal.
# i.e., correction without prior error is hallucination (Impact Statement warning).
if is_correction_turn(turn_index):
    assert len(prior_observations[turn_index - 1].strip()) > 0

# 4. Import-before-use invariant (Appendix E system prompt)
# All packages must be imported before first use in a code block.
# Checker: parse AST and verify no Name references precede their Import in execution order.
# (static approximation; dynamic imports inside conditionals still satisfy this weakly)
assert_imports_precede_uses(action_code)  # AST-level check

# 5. Performance lower bound (Table 3, Section 2.3)
# On M3ToolEval with GPT-4-level models, CodeAct must outperform text-action baseline
# by at least 15 percentage points absolute (paper reports 20.7pp for GPT-4-1106-preview).
# For D-11 registry: each registered checker must show non-zero improvement over no-action baseline.
assert codeact_success_rate >= text_action_success_rate + 0.15  # empirical floor from paper

# 6. Action economy invariant (Appendix E system prompt, Section 2.3 "-30% fewer actions")
# CodeAct achieves task in fewer turns than text-action on multi-tool tasks.
assert codeact_turn_count <= text_action_turn_count * 0.75  # paper: "up to 30% fewer"


Complexity:
Time per turn    | O(code_length) for interpreter execution; O(context_length * model_layers) for LLM forward pass
Space per turn   | O(sum of all prior observations) -- observations accumulate in context window
Parallelism      | None stated; sequential turn-by-turn protocol; independent tasks can parallelize at task level
Context growth   | Linear in turn_count; at 16384 tokens Mistral window, ~10 turns of ~1638 tokens each is the practical limit
Training compute | 4xA100 40GB, ~9k tokens/second throughput; no FLOP count provided


Failure modes (cited in paper):
Hallucination of variable state: agent imagines variable value without printing it (Impact Statement explicit warning)
Sandbox escape: code with sufficient capability may break sandbox isolation (Impact Statement)
Open-source model gap: best OSS model 13.4% vs GPT-4 74.4% on M3ToolEval -- OSS models underperform severely (Section 2.3)
LLaMA-2 anomaly: CodeActAgent on LLaMA-2 showed no improvement on M3ToolEval despite MINT benchmark gains (Section H)
Poor trajectory selection: training on trajectories without error-then-correction sequences degrades self-debug (Section 3.1)
Bulk code blocks: packing too many operations per <execute> block reduces error localization (Appendix E warning)
Zero-shot sensitivity: removing demonstrations significantly degrades open-source model performance (Section 2.3)


Implementation pitfalls (do NOT do this):
Do NOT enumerate a fixed tool set -- CodeAct's core value is unbounded Python action space; pre-enumerating tools collapses it to JSON-action regime
Do NOT skip error message passthrough -- returning empty observation on exception defeats the self-debug loop entirely
Do NOT allow multi-step bulk code blocks without the "attempt fewer things at a time" constraint (Appendix E)
Do NOT use JSON or text action format when Python is available -- paper shows up to 20pp regression (Table 3)
Do NOT train on trajectories that never encounter errors -- dataset must include error-then-fix sequences (Section 3.1)
Do NOT omit the import-before-use rule in the system prompt -- LLMs will reference undefined names without it (Appendix E)
Do NOT assume open-source models match GPT-4 performance -- 13.4% vs 74.4% gap means OSS agents need more scaffolding
Do NOT exceed max_turns without a hard termination guard -- runaway loops are a real failure mode (Section 2.3)


D-11 constructive_checker_registry grounding notes:
A "constructive check" in D-11 maps directly to one CodeAct turn-cycle:
  check_code (str)     -> the <execute> block; must be valid Python; must import before use
  check_observation (str) -> interpreter stdout+stderr; fed back as next input
  check_correction (str) -> revised <execute> block on non-empty error observation
  check_result (any)   -> final extracted value after clean execution

Registry entry invariants grounded in CodeAct:
  1. Every registered checker must be executable Python (not text description, not JSON schema)
  2. Every checker must handle its own error observation and emit a corrected block
  3. Checker success = clean execution + output matches expected pattern (exact-match per M3ToolEval)
  4. Checker budget = max 10 self-correction turns before marking as FAILED
  5. Checker must not assume external tools; must import all dependencies inline

Benchmark grounding for D-11 acceptance criteria:
  M3ToolEval: 82 human-curated multi-tool composition tasks, exact-match scoring
  API-Bank: atomic single-tool tasks, success measured by correct API call + response
  CodeAct success rate on M3ToolEval (GPT-4): 74.4% (Table 3)
  Text-action baseline on M3ToolEval (GPT-4): 53.7% (Table 3)
  Minimum acceptable improvement for any D-11 checker over no-action baseline: 15pp absolute

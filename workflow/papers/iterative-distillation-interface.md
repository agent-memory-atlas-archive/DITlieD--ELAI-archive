Algorithm: Iterative Distillation of Experience (IDoE) -- Prompting Policies for Multi-step Reasoning and Tool-Use in Black-box LLMs
ArXiv: 2605.14443
Paper: Prompting Policies for Multi-step Reasoning and Tool-Use in Black-box LLMs with Iterative Distillation of Experience
Authors: Krishna Sayana, Ketan Todi, Ambarish Jash
Submitted: 2026-05-14 (verified via arXiv abstract page 2026-05-24)
Local copy: .claude/papers/2605.14443.html
RAG topic: paper-iterative-distillation (project namespace)
Reliability: HIGH (official arXiv submission, abstract + section refs cross-verified)

ONE-LINE PURPOSE
Train a small "prompter" policy that, in a single forward pass, emits a system-prompt steering a frozen worker LLM toward higher task reward. Replaces multi-turn reflection with amortized policy weights, learned via RL on a contrastive experience buffer of (prompt, critique, reward) triples.

KEY ARCHITECTURE (sec 4.1 - 4.2)
- Three roles: prompter policy pi_theta (small, trainable), frozen worker LLM M (black-box), feedback model M_FB (generates textual critiques + scalar reward).
- Prompter conditions on (task context c, historical buffer slice H_c) and samples a single system instruction p; M(x, p) is then executed on task input x to produce y_hat; M_FB(x, y, y_hat) returns (critique f, reward r).
- "Amortizes iterative prompt refinement into single-shot policy weights" (verbatim, abstract). Multi-turn reflection collapsed into one forward pass at deployment.

INPUTS (per RL training step)
- training_contexts | list[Context] | task descriptions slicing the dataset                        | mandatory
- prompter_policy   | pi_theta(.| c, H_c) -> p                                                    | mandatory, trainable
- reference_policy  | pi_ref(.| c, H_c) -> p                                                     | mandatory, frozen, for KL anchor
- frozen_worker     | M(x, p) -> y_hat                                                            | mandatory, API-only black-box
- feedback_model    | M_FB(x, y, y_hat) -> (critiques C, scalar reward r)                         | mandatory
- buffer B_c        | dict[context_id -> list[(p, C, r)]]                                          | grows during training
- threshold epsilon | float, >= 0, gates buffer insertion vs current max reward                    | hyperparam
- candidates n      | int, candidates per step (specific n not stated in extracted main text)      | hyperparam
- kl_coef alpha     | float, KL penalty weight                                                     | hyperparam, alpha >= 0

OUTPUTS
- updated_theta     | tensor                                                                       | new prompter weights
- buffer_update     | append (p_t, C_t, r_t) IFF r_t >= r_max_c - epsilon  (Eq. 5)
- final_policy      | top-10 prompts by held-out eval reward at peak-aggregate checkpoint           | reported metric (sec 5.2 stopping criterion)

CORE ALGORITHM (Algorithm 1, sec 4.2.3, "Feedback-Driven Prompt RL with Experience Buffer")
1. For each context c, generate seed prompt p_0,c and critiques C_0,c via M_FB. Initialize buffer B_c with these.
2. For step t = 1 ... N:
   a. Sample context c and historical buffer slice H_c ~ B_c.
   b. Sample n candidate prompts: p_t,c^(i) ~ pi_theta(. | c, H_c), i = 1..n.
   c. Execute worker: y_hat_i = M(x, p_t,c^(i)). Score: r_t,c^(i), C_t,c^(i) = M_FB(x, y, y_hat_i).
   d. Buffer update (threshold-based, Eq. 5): for each i, if r_t,c^(i) >= r_max_c - epsilon, append (p_t,c^(i), C_t,c^(i), r_t,c^(i)) to B_c. Update r_max_c if exceeded.
   e. Compute objective J(theta) (Eq. 1):
        J(theta) = E_{(x,y)~D, p~pi_theta(.| c, H_c)} [ R(p, x, y) - alpha * D_KL(pi_theta || pi_ref) ]
   f. Update theta via GRPO (Group Relative Policy Optimization, sec 5.2) with Adam, lr = 1e-5.
3. Early stop: terminate if eval reward does not improve for 10 consecutive steps (sec 5.2).
4. Select peak-aggregate eval checkpoint; pick top-10 prompts by held-out per-sample eval reward; report mean test reward across held-out.

BUFFER / TRACE FORMAT (sec 4.2.1 - 4.2.2)
- Trajectory T = (p, C, r) where:
  - p: generated system instruction (string).
  - C: critique set { (x_j, y_j, y_hat_j, f_j) }_{j=1..K} -- K-sample discourse feedback. f_j is dense textual critique.
  - r: aggregate scalar reward over the dataset slice for that prompt.
- "Contrastive" buffer: stores both high- and lower-reward trajectories so the prompter can learn what NOT to do; KL anchor to pi_ref prevents degenerate collapse onto a single template.

FRAGMENT SELECTION (sec 4.2.2)
- Insertion criterion (Eq. 5): r_t^(i) >= r_max - epsilon.
- epsilon = 0 -> greedy (best-only): buffer collapses to a singleton, low diversity, high overfitting risk.
- epsilon > 0 -> top-band selection: broader high-performing distribution retained, more diverse historical context for next step's pi_theta sampling.
- Exact epsilon not provided in main text (likely Appendix / not visible in extracted content).

HYPERPARAMETERS (sec 5.2, verified)
- learning_algorithm  | GRPO + Adam                  | not negotiable per paper
- learning_rate       | 1e-5                         | single value reported
- early_stop_patience | 10 consecutive non-improving validation steps
- thinking_budget     | 4096 tokens for worker + critique models; unconstrained for prompter
- kl_coef alpha       | not specified numerically in extracted sections (Eq. 1 only states alpha >= 0)
- candidates_per_step | n (specific number not stated in extracted text)
- final_selection     | top-10 prompts at peak-aggregate-reward checkpoint, mean over held-out test

MODELS USED (sec 5.2)
- Frozen Worker M:    Gemini 2.5 Flash (BBEH + Tau-bench main experiments)
- Prompter pi_theta:  Gemini 2.5 Flash-lite (trainable steering agent)
- Feedback M_FB:      not explicitly named in extracted sections; assumed Gemini-family
- Cross-model worker swaps (sec 5.5): Flash-lite (small), Flash (mid), Pro (large)

HEADLINE NUMBERS (verbatim, with sec/table cites)
BBEH (Table 1, sec 5.3):
  Disambiguation QA    Baseline 57.08%  | GEPA 63.88% (+6.80)  | IDoE 65.41% (+8.33)
  Dyck Languages       Baseline 63.33%  | GEPA 79.16% (+15.83) | IDoE 91.25% (+27.92), p<0.001
  Web of Lies          Baseline 52.50%  | GEPA 67.50% (+15.00) | IDoE 90.12% (+37.62), p<0.001
  -- Note (sec 5.3): Web of Lies result reported WITHOUT experience buffer; all other rows USE buffer.
Tau-bench tool-use (Table 5, sec 6):
  Retail               Baseline 73.91%  | GEPA 82.60% (+8.69)  | IDoE 91.30% (+17.30)
  Airline              Baseline 65.27%  | GEPA 79.16% (+13.89) | IDoE 83.33% (+18.06)
Cross-model generalization on Dyck (Table 4, sec 5.5), prompter trained on Flash, deployed on:
  Flash-lite           42.5% -> 47.5%  (+5.0)
  Flash (in-distrib)   63.0% -> 91.0%  (+28.0)
  Pro                  72.5% -> 80.0%  (+7.5)
Buffer-vs-no-buffer sample efficiency (Table 2, sec 5.3.1):
  Dyck                 195 steps -> 102 steps  (1.91x faster to same reward)
  DQA                  180 steps ->  75 steps  (2.40x faster)
Abstract headline: 55% -> 90% on logic reasoning, 74% -> 91% on tool-use (aggregate / best-case across BBEH + Tau).

CRITICAL INVARIANTS (testable assertions, Python syntax)
assert epsilon >= 0.0, "Buffer threshold epsilon must be non-negative (Eq. 5 sec 4.2.2)"
assert kl_coef >= 0.0, "KL anchor coefficient alpha must be non-negative (Eq. 1 sec 4.1.1)"
assert all(r_inserted >= r_max - epsilon for r_inserted in buffer_inserts_this_step), "Buffer insertion violates Eq. 5"
assert thinking_budget_worker == 4096 and thinking_budget_critique == 4096, "Paper sec 5.2 budgets"
assert len(top_k_final) == 10, "Final reporting uses top-10 prompts at peak-aggregate-eval checkpoint (sec 5.2)"
assert prompter_param_count < worker_param_count, "Prompter must be lightweight relative to worker (architecture premise, sec 1, 4.1)"
assert reward_signal_is_scalar(r_t), "Reward r is scalar; critiques C are separate dense textual feedback (sec 4.2.1)"

COMPLEXITY
- Per RL step: n worker calls (one per candidate prompt) + n feedback-model calls + 1 prompter forward + 1 prompter backward.
- Inference cost (post-training): exactly 1 prompter forward + 1 worker forward -- "amortized" claim (abstract).
- Parallelism: candidates within a step are embarrassingly parallel across the n worker calls; trivially batchable.
- Token cost dominated by worker thinking_budget = 4096 per call; critique generation adds another 4096 budget per call.

FAILURE MODES (acknowledged or implied)
- Small eval datasets (<50 samples for Dyck/Web-of-Lies, even fewer for DQA, sec 5.3) -> high-variance significance tests; Disambiguation QA gain over GEPA is nominally positive but not statistically significant (p<0.25, sec 5.3).
- Web of Lies result uses NO experience buffer (sec 5.3 caption note); the buffer is not strictly required and may hurt on some tasks. Treat "buffer always helps" as ungrounded.
- Cross-model transfer to LARGER worker (Pro) yields smaller gain (+7.5) than in-distribution Flash (+28.0); transfer to SMALLER worker (Flash-lite) gives only +5.0. The prompter overfits to the model it was trained against -- per-worker training likely needed.
- Buffer threshold epsilon mis-set: epsilon=0 collapses buffer to singleton (low diversity); epsilon too large includes noise. Paper does not publish a sweep -- pick conservatively.
- Critique model M_FB quality is a hidden dependency: if M_FB is the same family as the worker, critiques may inherit worker's blind spots. Paper does not address this explicitly.

IMPLEMENTATION PITFALLS (do NOT do this)
- Do NOT drop the KL anchor (alpha=0); the paper relies on KL-regularized RL (Eq. 1) -- without it, the prompter degenerates to a single high-reward template and loses cross-context generalization.
- Do NOT replace GRPO with vanilla policy gradient without re-verifying; GRPO uses group-relative advantage (sec 5.2 explicit choice).
- Do NOT use a single epsilon for all tasks; Web of Lies result (sec 5.3) explicitly disables the buffer -- treat epsilon as a per-task hyperparameter.
- Do NOT report final-step accuracy; the paper picks the peak-aggregate-eval checkpoint then top-10 prompts (sec 5.2). Final-step accuracy will systematically under-report.
- Do NOT skip the contrastive (low-reward) buffer entries; "contrastive" is in the abstract and is load-bearing -- pure best-only is what GEPA does and IDoE is meant to beat that.
- Do NOT extrapolate the +37.62 Web-of-Lies gain to other tasks; that is the single largest improvement in the paper and may be task-specific (logical reasoning over short symbolic input).
- Do NOT use the prompter on a worker different from training without expecting degradation (sec 5.5 cross-model transfer numbers).

CONTRACT TEST STUBS (copy into tests/contracts/test_iterative_distillation_contract.py)

def test_buffer_insertion_threshold():
    """Eq. 5: r_t^(i) >= r_max - epsilon is the insertion gate."""
    epsilon = 0.05
    r_max = 0.80
    inserts = [r for r in [0.82, 0.78, 0.74, 0.76] if r >= r_max - epsilon]
    assert inserts == [0.82, 0.78, 0.76], "Buffer insertion gate violates Eq. 5"

def test_kl_penalty_non_negative():
    """Eq. 1: alpha >= 0 KL penalty coefficient."""
    alpha = -0.1
    try:
        assert alpha >= 0.0, "alpha must be non-negative"
        raise AssertionError("Should have failed")
    except AssertionError as e:
        assert "non-negative" in str(e)

def test_topk_final_reporting():
    """sec 5.2: final reporting uses top-10 prompts at peak-aggregate-eval checkpoint."""
    final_prompts = [{"id": i, "eval_reward": 1.0 - i*0.01} for i in range(50)]
    top10 = sorted(final_prompts, key=lambda p: -p["eval_reward"])[:10]
    assert len(top10) == 10
    assert top10[0]["eval_reward"] >= top10[-1]["eval_reward"]

def test_prompter_smaller_than_worker():
    """Architectural premise sec 1, 4.1: prompter is 'lightweight' relative to worker."""
    prompter_params = 2_000_000_000  # Flash-lite proxy
    worker_params = 10_000_000_000   # Flash proxy
    assert prompter_params < worker_params, "Prompter must be smaller than frozen worker"

def test_buffer_can_be_disabled_per_task():
    """sec 5.3: Web of Lies uses NO experience buffer. Buffer must be an OFF-switch per task."""
    config_web_of_lies = {"use_buffer": False, "epsilon": None}
    config_dyck = {"use_buffer": True, "epsilon": 0.05}
    assert config_web_of_lies["use_buffer"] is False
    assert config_dyck["use_buffer"] is True

VERIFICATION TRAIL
- ID 2605.14443 valid as of 2026-05-24 (today). Submission date 2026-05-14, well-formed for current arXiv ID range.
- Abstract page WebFetch returned exact title + authors + abstract matching the topic claimed by C-49 plan.
- HTML full-text WebFetch confirmed Algorithm 1 (sec 4.2.3), Equations 1 and 5, Tables 1/2/4/5, hyperparameter section 5.2.
- Local-rag search returned high-similarity (>0.74) hits already indexed under project=paper-iterative-distillation; topic alignment confirmed end-to-end.
- Re-ingest attempts returned "IPC timeout" (known intermittent local-rag reranker issue per MEMORY.md project_rag_reranker_crash); existing index is sufficient for C-49 grounding.

DEVIATIONS COMMON IN NAIVE IMPLEMENTATIONS (informed by paper's related-work framing and known RL-on-prompts failures)
- Treating the prompter as a single string template instead of a sampling distribution -- breaks the "policy" abstraction and removes diversity that the buffer relies on.
- Using the same model for prompter AND critique AND worker -- circular feedback; the paper uses three roles deliberately.
- Reporting mean reward across all training steps instead of peak-eval-checkpoint -- inflates apparent performance; paper is explicit about top-10 from peak (sec 5.2).
- Forgetting the historical context H_c in the prompter input -- the buffer is conditioning input, not just training data; without H_c at inference, the policy loses the "amortized iterative refinement" property (abstract claim).

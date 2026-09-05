Algorithm: Why Language Models Hallucinate (theoretical framework)
ArXiv: 2509.04664
Paper: Why Language Models Hallucinate
Authors: Adam Tauman Kalai, Ofir Nachum, Santosh S. Vempala, Edwin Zhang (OpenAI / Georgia Tech)
Date: September 4, 2025
Venue: arXiv:2509.04664v1 [cs.CL]

PROBLEM
Hallucinations arise from two sequential structural causes: (1) cross-entropy pretraining produces irreducible errors even on clean data for arbitrary/unlearnable facts, and (2) binary-graded evaluations create incentive misalignment that rewards overconfident guessing over abstention. This is NOT a computational-complexity (NP-hardness/undecidability) account -- it is a statistical-inevitability account under standard loss + evaluation design.

CORE MECHANISM
The Is-It-Valid (IIV) reduction converts a density estimator p-hat(r|c) into a binary classifier: classify r as valid iff p-hat(r|c) > 1/|E_c| (where E_c is the set of all possible completions for context c). This links generative hallucination error directly to IIV misclassification error. The bound flows: hallucination err >= 2*err_iiv - calibration_slack - coverage_slack.

KEY RESULTS WITH NUMBERS

Theorem 1 (General, with prompts, Sec 2.1):
  err >= 2*err_iiv - max_c|V_c|/min_c|E_c| - delta
  Condition: clean training data (p(V)=1), any prompted LM

Theorem 2 (Arbitrary Facts, Sec 2.2):
  err >= sr - 2/min_c|E_c| - (35 + 6*ln(N))/sqrt(N) - delta  (99% confidence)
  sr = singleton rate (fraction of training facts appearing exactly once)
  Concrete example: birthdays -- 364 wrong dates per person; if sr=0.20, floor error >= ~20%
  Upper bound for calibrated (delta=0) model: er <= sr - sr/(max_c|E_c|+1) + 13/sqrt(N)

Theorem 3 (Pure multiple-choice, no IDK option, Sec 2.3):
  err >= 2*(1 - 1/C)*opt(G)
  C = number of answer choices, opt(G) = optimal binary classifier error on the same data

Observation 1 (Evaluation-incentive impossibility, Sec 3):
  Under ANY binary grader distribution, the optimal response is never "I don't know."
  Proof: abstention yields 0 expected credit; any guess over binary {correct,wrong} has positive expected value.
  Documented in Table 2: GPQA, MMLU-Pro, BBH, SWE-bench, HLE all use binary grading with zero IDK credit.

Calibration delta:
  delta = |p-hat(A) - p(A)| at threshold 1/|E|
  Cross-entropy loss minimization naturally yields small delta (Sec 2.1 argument).

GRADING RECOMMENDATION
Append to existing benchmarks: "Answer only if you are >t confident, since mistakes are penalized t/(1-t) points."
Recommended t values: 0.5 (penalty=1x), 0.75 (penalty=2x), 0.9 (penalty=9x).
This induces "behavioral calibration" -- model should output IDK when P(correct) < t.
Proposed modification applies to EXISTING benchmarks (GPQA etc.), not new specialized benchmarks.

HOW IT DIFFERS FROM COMPLEXITY ACCOUNTS
Complexity accounts (Appendix D): hallucination due to NP-hardness / undecidability of the generation task.
This paper: hallucination is statistically inevitable for arbitrary facts under cross-entropy + binary grading, independent of computational hardness. Even computationally easy tasks (birthday lookup) hallucinate if facts are unlearnable patterns (sr > 0).

INPUTS (to the theoretical framework)
p-hat | LM distribution | p-hat(r|c) for all r,c | must be a valid conditional probability distribution
V_c   | valid responses  | set of correct completions for context c | |V_c| >= 1
E_c   | all completions  | full set of possible responses | |E_c| >> |V_c|
N     | training size    | integer | large N reduces (35+6lnN)/sqrt(N) term to ~0
sr    | singleton rate   | float in [0,1] | fraction of training facts with count=1
delta | calibration gap  | float >= 0 | should be small under cross-entropy

OUTPUTS (bounds)
err   | hallucination rate | float in [0,1] | lower-bounded by theorems above
err_iiv | IIV misclassification | float in [0,1] | err >= 2*err_iiv - slack

CRITICAL INVARIANTS (testable assertions)
assert err >= 2 * err_iiv - calibration_slack - coverage_slack  # Theorem 1
assert err >= sr - (35 + 6*math.log(N)) / math.sqrt(N) - 2/min_Ec - delta  # Theorem 2 (99% CI)
assert err >= 2*(1 - 1/C) * opt_G  # Theorem 3 (pure MCQ)
assert abstention_credit_under_binary_grading == 0  # Observation 1
# calibration: small delta is CONSISTENT with cross-entropy but not guaranteed post-RLHF

FAILURE MODES CITED IN PAPER
- Evaluations without IDK credit structurally reward hallucination over abstention (Observation 1)
- High singleton rate (many unique facts in training) creates irreducible hallucination floor
- Post-training (RLHF/SFT) may destroy calibration (delta grows), making IIV bound looser
- Binary graders (human or LLM) cannot fix the incentive problem without explicit IDK scoring

IMPLEMENTATION PITFALLS
- Do NOT interpret Theorem 2 as: "reducing N reduces hallucination" -- N appears in a correction term; the primary driver is sr
- Do NOT interpret small delta as meaning the model is well-calibrated after RLHF -- delta is defined at pretraining distribution, not post-training
- Do NOT use this as justification for IDK-only abstention; the paper recommends confidence-gated abstention at threshold t, not blanket refusal
- Do NOT apply Theorem 3 directly to open-ended generation -- it requires |V_c|=1 (exactly one correct answer)
- The IIV reduction assumes p(V)=1 (no noise in training labels); real-world noise adds to the error floor

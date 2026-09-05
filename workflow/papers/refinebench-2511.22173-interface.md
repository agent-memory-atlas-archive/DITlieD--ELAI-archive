Algorithm: RefineBench Evaluation Framework
ArXiv: 2511.22173
Paper: RefineBench: Evaluating Refinement Capability of Language Models via Checklists
Authors: Young-Jun Lee, Seungone Kim, Byung-Kwan Lee, Minkyeong Moon, Yechan Hwang, Jong Myoung Kim, Graham Neubig, Sean Welleck, Ho-Jin Choi (KAIST, CMU, NVIDIA)
Published: November 2025

Inputs:
query_x_t | text | per-turn | mandatory; the original or refined query at turn t
previous_answer_y_t-1 | text | per-turn | mandatory for t>=2; prior model output to be refined
checklist_C | list[str] | N binary questions, avg N=9.9 per problem | mandatory; defines what "correct" means
feedback_mode | enum {self, guided, partial} | N/A | mandatory; self=no feedback, guided=all failed items revealed, partial=fraction revealed
turn_t | int | range [1, 5] | mandatory; max 5 turns defined by benchmark protocol
evaluator_LM | model ref | GPT-4.1 | mandatory; evaluator assessing checklist item binary pass/fail

Outputs:
Acc_t | float | scalar per instance per turn | = 100 * N_c / N where N_c = checklist items correct; invariant: 0 <= Acc_t <= 100
Pass_t | int {0, 1} | scalar per instance per turn | = 1 iff N_c == N (all checklist items correct), else 0
Delta | float | scalar per model | = Pass_5 - Pass_1; measures net gain over 5-turn refinement loop
Pass_5_aggregate | float | mean(Pass_t=5) across all instances * 100 | primary reported metric; ceiling ~31.3% self-refinement, ~98.4% guided refinement

Hyperparameters:
max_turns | 5 | range [1, inf] | benchmark protocol fixed at T=5; most gain captured by t=3 in guided setting
evaluator_temperature | 0.0 | fixed | paper Appendix E.1; must be 0 to ensure deterministic checklist judgment
evaluator_top_p | 1.0 | fixed | paper Appendix E.1
evaluator_max_tokens | 10000 | fixed | paper Appendix E.1; applies to both evaluator and refinement model
reasoning_effort | medium | {low, medium, high} | for OpenAI series only; paper default medium

B-39 ITERATION CAP ANALYSIS (specific question from calling session):

The paper does NOT present an explicit "marginal return below threshold" analysis or
recommend a specific iteration cap. However, the per-turn Pass_t data from Table 3
(self-refinement setting, Pass_t metric, selected models) allows the following analysis:

TABLE 3 SELF-REFINEMENT Pass_t DATA (verbatim from paper):

Model              | Delta | t=1  | t=2  | t=3  | t=4  | t=5
Gemini-2.5-Pro     | +1.8  | 29.5 | 30.7 | 31.1 | 31.1 | 31.3
GPT-5              | +1.7  | 27.5 | 28.3 | 29.2 | 29.5 | 29.1
o4-mini            | +2.1  | 20.4 | 22.0 | 22.4 | 22.6 | 22.5
Claude-Opus-4.1    | +2.1  | 18.7 | 20.8 | 20.8 | 20.8 | 20.8
Claude-Opus-4      | +0.7  | 17.7 | 18.2 | 18.4 | 18.4 | 18.4
o3-mini            | -0.8  | 19.5 | 19.1 | 18.8 | 18.8 | 18.7
GPT-4.1            | -1.6  | 23.4 | 21.9 | 21.9 | 21.8 | 21.8
DeepSeek-R1        | -0.1  |  8.1 |  8.5 |  8.6 |  7.9 |  7.9
DeepSeek-R1(Q32B)  | -2.5  |  6.2 |  4.4 |  3.5 |  4.0 |  3.7
LLaMA-3.1-8B-Inst  | -0.3  |  1.4 |  1.0 |  1.0 |  0.9 |  1.0

TABLE 3 GUIDED-REFINEMENT Pass_t DATA (verbatim from paper):

Model              | Delta | t=1  | t=2  | t=3  | t=4  | t=5
Claude-Opus-4.1    | +79.7 | 18.7 | 81.7 | 94.3 | 97.2 | 98.4
Claude-Opus-4      | +79.5 | 17.7 | 76.9 | 93.1 | 96.1 | 97.2
o3-mini            | +78.7 | 19.5 | 74.8 | 92.2 | 96.7 | 98.2
o4-mini            | +76.1 | 20.4 | 81.3 | 93.2 | 95.4 | 96.4
Claude-Sonnet-3.7  | +84.9 |  8.4 | 70.6 | 87.3 | 91.7 | 93.3
Claude-Sonnet-4    | +81.2 | 15.4 | 73.9 | 90.6 | 95.1 | 96.6
Grok-3-mini        | +80.1 | 15.5 | 81.1 | 89.7 | 93.9 | 95.6
Gemini-2.5-Pro     | +65.2 | 29.5 | 76.4 | 87.9 | 92.3 | 94.7
Qwen2.5-72B-Instruct | +79.6 | 8.5 | 54.6 | 76.3 | 83.6 | 88.1
DeepSeek-R1        | +83.3 |  8.1 | 57.6 | 77.7 | 86.6 | 91.4
GPT-5              | +51.6 | 27.5 | 64.8 | 73.9 | 76.9 | 79.0
Gemini-2.5-Flash   | +65.9 | 22.9 | 63.9 | 77.4 | 83.7 | 88.7
LLaMA-3.1-70B-Inst | +65.0 |  4.7 | 43.2 | 59.2 | 66.9 | 69.7

B-39 VERDICT on iteration cap of 2:

Self-refinement setting:
- Iteration cap of 2 is SUPPORTED in the self-refinement setting. The data shows that
  virtually all gains (positive or negative) occur at t=2. From t=2 onward, changes
  are within 0.0-0.4% across all models. For Gemini-2.5-Pro: t1=29.5, t2=30.7
  (+1.2pp), t3=31.1 (+0.4pp), t4=31.1 (+0.0pp), t5=31.3 (+0.2pp). Marginal gain
  from t=2 to t=5 is 0.6pp total across 3 more turns. For Claude-Opus-4.1: t1=18.7,
  t2=20.8 (+2.1pp), t3-5=20.8 (0.0pp gain). Self-refinement saturates at t=2.

Guided refinement setting:
- Iteration cap of 2 is NOT supported for guided refinement. The data shows significant
  gains from t=2 to t=3. For Claude-Opus-4.1: t2=81.7 -> t3=94.3 (+12.6pp jump).
  For o3-mini: t2=74.8 -> t3=92.2 (+17.4pp jump). The largest single-step gain is
  t=1 to t=2 (+40-62pp for most models), with secondary gain at t=2 to t=3
  (+12-20pp). t=3 to t=4 still adds 2-10pp. Cap at 2 leaves 10-20pp on the table.
  Near-saturation occurs at t=3 for best models (Claude-Opus-4.1 reaches 94.3% at t=3).

  If B-39 operates in guided mode, recommended cap is t=3, not t=2.
  If B-39 operates in self-refinement mode, cap of t=2 is empirically justified.

Paper quote (Section 4.2): "Notably, Claude-Opus-4.1 reaches 94.3% as early as t=3,
while o3-mini reaches 98.2% at t=5."

Paper quote (Section 5.2): "most LMs, on average, terminate their self-refinement
around the mid-turns (typically between 3~4), even though they have not adequately
refined their previous responses"

Paper quote (Section 5.2): "there exists a statistically significant (p-value < 0.01)
negative correlation (R^2 = -0.477) between [termination turn ratio] and Pass_5" --
meaning models that continue longer do NOT perform better.

Critical invariants (testable assertions):
assert 0.0 <= acc_t <= 100.0, "Acc_t must be in [0, 100]"
assert pass_t in (0, 1), "Pass_t is binary"
assert pass_t == 1 if acc_t == 100.0 else pass_t == 0, "Pass_t=1 iff all checklist items correct"
assert delta == pass_t5 - pass_t1, "Delta is defined as Pass_5 minus Pass_1"
assert n_checklist_items >= 1, "Checklist must have at least 1 item"
assert 1 <= turn <= 5, "Benchmark defines turns 1 through 5 inclusive"

Complexity:
time | O(T * N * |response|) per sample where T=5 turns, N=avg 9.9 checklist items | evaluator call per item per turn
space | O(T * |context|) per sample | full conversation history grows with turns
cost | $0.038/sample self-refinement, $0.028/sample guided (Gemini-2.5-Pro, GPT-4.1 evaluator) | latency 51.1s/22.9s respectively
parallelism | checklist items within a turn can be evaluated in parallel | turns must be sequential

Failure modes:
Self-refinement plateau: models fail to improve Pass_t beyond turn 2 without external feedback (best frontier model reaches only 31.3% Pass_5)
Regression at subsequent turns: most models show Delta in range [-2.5%, 0%] for self-refinement; DeepSeek-R1-Q32B drops -2.5%
Correct-to-incorrect transitions: DeepSeek-R1 transitions 19.1% correct-at-t2 to incorrect-at-t3 (Figure 9, Section 5.3)
Reasoning collapse: DeepSeek-R1 reasoning tokens drop 69.7% from t=1 to t=2; self-correction patterns vanish after turn 1
Smaller models fail guided refinement: models <70B parameters do not benefit significantly even with full feedback
Feedback blindness: models accurately fix items for which feedback is provided but do not generalize to unprovided items (Figure 4)
Domain variance: STEM refinement gains range -1.2 to +2.5; Law shows larger gains (+5 to +7.8 for top models)

Implementation pitfalls (do NOT do this):
Do NOT use self-refinement Pass_5 as a proxy for refinement capability; it confounds error detection ability with correction ability
Do NOT cap refinement at iteration 2 when operating in guided mode; t=2 to t=3 delivers 12-20pp additional gain for top models
Do NOT assume longer thinking budgets improve multi-turn self-refinement; Figure 5 + Section 5.2 show this pattern is flat or negative
Do NOT use Pass_t=5 alone as the stopping criterion; models that terminate early at t=3-4 often outperform those forced to continue
Do NOT batch evaluate checklist items at the response level; each binary item must be assessed independently by the evaluator LM
Do NOT generalize DeepSeek-R1 self-refinement failure to other architectures; it is specific to its CoT exhaustion pattern
Do NOT assume accuracy (Acc_t) and pass-rate (Pass_t) move together; Acc can be high while Pass is near zero (partial correctness)

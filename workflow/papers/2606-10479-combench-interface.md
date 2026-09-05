Algorithm: ComBench - Benchmark for Proof Reasoning and Constructive Realization in Olympiad-Level Combinatorics
ArXiv: 2606.10479
Paper: ComBench: A Benchmark for Rigorous Proof Reasoning and Constructive Realization in Olympiad-Level Combinatorics

Inputs:
problem_statement | text | 1 of 100 problems | sourced from 15 competition venues (IMO, USAMO, team selection tests, regional olympiads) 2000-2025
model_response | text | free-form proof + optional construction | model may produce a natural-language proof and/or a concrete discrete object in prescribed format
construction_output | structured object | problem-specific format | required for construction-centric problems; e.g. colorings, graphs, tilings, permutations; format is problem-specific

Outputs:
proof_score | int | {0, 1, 6, 7} | rubric-aligned with Olympiad grading; assessed by LLM-based judge against problem-specific guidelines
construction_verdict | bool | pass/fail | per-problem Python verifier checks representation validity + mathematical constraints; no human interpretation
final_score | int | {0, 1, 6, 7} | proof_score after verifier-gated demotion for construction-centric problems
average_score | float | [0.0, 7.0] | mean final_score across subset; paper reports as percentage of maximum

Hyperparameters:
n_problems | 100 | not varied | 50 analysis-centric + 50 construction-centric; not a tunable parameter
rubric_points | {0, 1, 6, 7} | fixed | aligned with IMO partial-credit conventions; no intermediate values 2-5
best_of_k | 4 | Best@4 metric; paper also reports Avg. | k=4 is the primary comparison point; higher k would inflate Best@K
judge_model | LLM (not specified as fixed model in paper) | problem-specific rubric guidelines | sensitive: judge model choice affects proof scores; construction verdict is deterministic
demotion_rule_score7 | 7->6 on construction fail | fixed | demotes high-quality proofs that lack checkable witnesses
demotion_rule_score6 | 6->1 on construction fail | fixed | severe demotion; reflects that a proof claiming construction without realizing it is near-worthless

Critical invariants (testable assertions):
assert proof_score in {0, 1, 6, 7}, "only four valid proof score values; no intermediate values"
assert final_score in {0, 1, 6, 7}, "demotion preserves the valid score set"
assert final_score <= proof_score, "demotion rule is monotone non-increasing"
assert construction_centric_count == 50 and analysis_centric_count == 50, "benchmark is exactly balanced 50/50"
assert len(problems) == 100, "benchmark size is fixed at 100 problems"
# verifier-gated demotion rule:
if problem_type == "construction" and proof_score == 7 and not construction_passes: assert final_score == 6
if problem_type == "construction" and proof_score == 6 and not construction_passes: assert final_score == 1
if problem_type == "analysis": assert final_score == proof_score, "analysis problems have no construction demotion"
assert overlap_with_imo_bench <= 14, "paper documents 14% overlap (86 unique problems); full dedup not performed"

Complexity:
Evaluation per problem: one LLM call for proof generation + one deterministic Python verifier run for construction-centric
Judge scoring: one LLM call per proof response; parallelizable across problems
Best@K evaluation: K independent model calls per problem; K=4 in paper
Space: 100 records; each has problem text + rubric + optional verifier code; small dataset, no scale constraint
Parallelism: all 100 problems are independent; full parallelism across problems and across K samples

Failure modes:
Verifier format rigidity: verifier only validates constructions in pre-specified formats; valid alternative representations score fail even if mathematically correct
Judge inconsistency: LLM-based proof scoring is non-deterministic and model-dependent; different judge models produce different score distributions
14% benchmark overlap with IMO-Bench: models trained or evaluated on IMO-Bench may have inflated scores on the 14 overlapping problems
Construction capability separation: Kimi-K2.6 shows high construction / low analysis divergence, demonstrating these are distinct capabilities that can diverge sharply per model
Category imbalance in difficulty: "Existence and Construction" problems are hardest; a model specialist in one category may score anomalously on ComBench
Best@4 vs Avg divergence: Best@4 scores can be 10+ points above Avg scores, masking consistency failures

Implementation pitfalls (do NOT do this):
Do NOT apply construction demotion to analysis-centric problems; demotion rule only applies to construction-centric records
Do NOT treat a score of 6 or 7 as "passing" without verifying the construction verdict for construction-centric problems; fluent proof + failed construction = score 1
Do NOT use a single model call for Best@K; K independent samples are required (no chain-of-thought reuse)
Do NOT conflate proof quality with construction realizability; these are explicitly measured as distinct capabilities and a model can score high on one and low on the other
Do NOT assume the verifier is sound for all valid constructions; verifier false-negatives exist for non-standard but correct representations
Do NOT compare raw average scores across models without checking if the same judge model was used; judge model affects proof scores

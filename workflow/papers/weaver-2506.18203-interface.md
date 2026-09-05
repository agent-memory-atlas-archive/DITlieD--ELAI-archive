Algorithm: Weaver (Weak-Supervision Ensemble Verification and Response Selection)
ArXiv: 2506.18203
Paper: Shrinking the Generation-Verification Gap with Weak Verifiers

Authors: Jon Saad-Falcon, E. Kelly Buchanan, Mayee F. Chen, Tzu-Heng Huang, Brendan McLaughlin,
Tanvir Bhathal, Shang Zhu, Ben Athiwaratkun, Frederic Sala, Scott Linderman, Azalia Mirhoseini,
Christopher Re (Stanford, UW-Madison, Together AI)

Problem:
The generation-verification gap (Pass@K minus success rate) is large in practice because individual
LM judges and reward models are imperfect, noisy, and poorly calibrated, causing correct generated
responses to be passed over during selection. Perfect (oracle) verifiers would close the gap but are
unavailable. The paper asks how much weak, imperfect verifiers can close the gap when combined.

Method summary:
Weaver adapts the weak supervision / data programming paradigm (Ratner et al., Snorkel) to response
selection. Multiple heterogeneous weak verifiers (reward models, LM judges with logit/continuous/
Likert outputs) are binarized, filtered for quality, and combined via a learned latent-variable label
model that estimates each verifier's accuracy without ground-truth labels. The posterior probability
that a response is correct drives argmax selection over the K candidates per query.


Inputs:
candidates        | list[str]          | K responses per query (K=100 in main experiments)
weak_verifiers    | list[Callable]     | M heterogeneous verifiers; output any of: logits, continuous score, Likert integer
dev_set           | list[(query,label)]| ~1% of test queries with ground-truth labels for global Pr(Y=1) estimation
query             | str                | the prompt used to generate candidates

Outputs:
selected_response | str                | argmax_{k} Pr(Y=1 | S_1=s_1^k,...,S_m=s_m^k) over K candidates
posterior_scores  | float[K]           | Pr(Y=1|verifier votes) for each candidate; in [0,1]; need not sum to 1
pseudolabels      | binary[K]          | optional; thresholded posteriors used for distillation downstream

Hyperparameters:
K (samples)       | 100    | 1..inf    | main experiments use 100; diminishing returns above ~50 for strong generators
M (verifiers)     | 8-15   | 2..inf    | diminishing returns above ~15; pairwise matrix is O(M^2); more != better
dev_set fraction  | 0.01   | 0.005..0.05| used only for Pr(Y=1) prior; below 0.5% unstable on hard tasks
binarization thresh| per verifier| calibrated on dev_set | continuous verifiers thresholded at median or task-specific cutoff
filter_threshold  | dataset stats| accuracy < 0.5 discarded| verifiers that perform at or below chance are dropped before estimation

Mathematical invariants (testable assertions):
assert all(0.0 <= p <= 1.0 for p in posterior_scores), "posteriors must be valid probabilities"
assert len(posterior_scores) == K, "one posterior per candidate"
# Conditional independence assumption underlies parameter estimation:
# Pr(Si, Sj) == Pr(Si|Y=1)*Pr(Sj|Y=1)*Pr(Y=1) + Pr(Si|Y=0)*Pr(Sj|Y=0)*Pr(Y=0)
# This is an untestable modelling assumption but failure degrades (not corrupts) outputs
assert selected_response in candidates, "selection must be one of the K candidates"
# Weaver cannot exceed oracle Pass@K: success_rate <= Pass@K by construction
assert success_rate <= pass_at_K + 1e-9, "verifier cannot select what was never generated"

Complexity:
Parameter estimation  | O(M^2) pairwise joint probability matrix per query batch
Verification inference | O(K * M) verifier forward passes; dominates total cost (10x-128x over generation)
Matrix solve           | gradient descent on Eq 5; convergence time not analyzed; quadratic in M
Space                  | O(K * M) binary vote matrix per query
Parallelism            | verifier inference fully parallelizable across K and M; no ordering constraint
Distilled proxy        | 400M cross-encoder: 99.97% FLOP reduction vs full ensemble (1.01 vs 35.35 exaFLOPs per 100 samples)

Failure modes:
Extreme-difficulty tasks: AIMO 2024 fails because Pr(Y=1) << 0.05; dev-set prior estimation collapses
Correlated verifier errors: when all verifiers share the same knowledge bias, ensemble reduces variance but not bias
Low-diversity verifier pool: adding redundant models past M~15 yields marginal gains (bias-variance tradeoff)
Insufficient labeled dev data: <0.5% labels on hard tasks break Pr(Y=1) estimation
OOD verifier deployment: verifiers trained on different distributions show wildly varying accuracy per task
High false positive rate: preprocessing normalizes but does not eliminate verifier miscalibration on hard positives

Implementation pitfalls (do NOT do this):
Do NOT skip binarization: heterogeneous verifier scales (logits vs Likert vs continuous) must be normalized first; raw concatenation is undefined
Do NOT use all verifiers without filtering: including chance-level verifiers degrades below majority voting
Do NOT assume conditional independence holds without validation: shared biases on hard examples violate Si perp Sj | Y
Do NOT use distilled proxy as a drop-in for the full ensemble without re-distilling on the target task distribution
Do NOT use <1% dev data on tasks with <5% pass rate: parameter estimation becomes numerically unstable
Do NOT conflate Pass@K ceiling with success rate: Weaver is bounded by Pass@K, never by Pass@1
Do NOT use a single verifier: the weak supervision framework requires M>=2 verifiers to estimate pairwise statistics

Stance vs arXiv:2507.07505 (Hallucination Stations / O(N^2*d) ceiling thesis):
CONTRADICTS (empirically) / REFINES (theoretically)

The hallucination-ceiling thesis (2507.07505) holds that a verifier sharing the generator's error
distribution cannot raise selection above the generator's ceiling, since hallucinations that fool
the generator also fool any same-distribution verifier. Weaver's result is a PARTIAL EMPIRICAL
CONTRADICTION: by combining multiple verifiers with DIFFERENT error profiles (diverse reward
models, diverse LM judges), Weaver achieves 87.7% average vs o3-mini 86.7% using a Llama-3.3-70B
generator -- closing the gap to within 4.2% of oracle Pass@100. This demonstrates that ensembling
imperfect verifiers BELOW oracle quality still measurably exceeds majority voting and closes the
generation-verification gap, contrary to a strong reading of the ceiling thesis.

The THEORETICAL reconciliation is that the ceiling thesis applies to a SINGLE verifier sharing the
generator's distribution. Weaver exploits VERIFIER DIVERSITY: different verifiers fail on different
examples (low correlation of errors given Y), so the ensemble's effective accuracy exceeds any
individual weak verifier. This is the bias-variance decomposition, not a violation of the ceiling.
Weaver cannot exceed Pass@K (the true generation ceiling) and admits failure on tasks where ALL
verifiers fail correlated (AIMO 2024). So the O(N^2*d) ceiling is not violated -- it is reframed:
the relevant ceiling is the ORACLE verifier (Pass@K), not any single-verifier accuracy, and diverse
ensembles approach but never exceed it.

The O(N^2) notation of the hallucination-ceiling thesis is EXTENDED by Weaver: the paper adds an
M^2 (verifier-pair) dimension to the cost analysis, showing that scaling verifiers has quadratic
cost and sub-linear gain, consistent with the ceiling thesis's pessimism about scaling.

Headline results:
MATH500: Weaver 93.4% vs majority voting 83.0% (+10.4 pp) vs o3-mini 85.9%
GPQA Diamond: Weaver 72.1% vs majority voting 47.4% (+24.7 pp)
MMLU College: Weaver 94.9% vs majority voting 84.1% (+10.8 pp)
MMLU Pro: Weaver 90.2% vs majority voting 74.4% (+15.8 pp)
4-benchmark average: Weaver 87.7% vs o3-mini 86.7% (Llama-3.3-70B generator)
Oracle gap closure: within 4.2% of Pass@100 oracle average
Distilled 400M proxy: 98.2% of Weaver accuracy at 99.97% FLOP reduction
8B model gap closure vs Pass@1: +12.8 pp; 70B model: +16.0 pp

Limitations (author-stated):
1. AIMO 2024 fails: too few correct responses for stable weak supervision
2. Conditional independence assumption (Si perp Sj | Y) violated when verifiers share knowledge biases
3. Diminishing returns past M~15 verifiers (bias-variance tradeoff)
4. Verification compute is 10x-128x generation cost for full ensemble (mitigated by distillation)
5. Verifier calibration: preprocessing does not eliminate high false-positive rates on hard examples
6. No theoretical guarantee that accuracy parameter estimation converges under near-zero Pr(Y=1)

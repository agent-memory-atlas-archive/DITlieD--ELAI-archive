Algorithm: Statistically Negligible Hallucination (theoretical framework)
ArXiv: 2502.12187
Paper: Hallucinations are inevitable but can be made statistically negligible
Authors: Atsushi Suzuki, Yulan He, Feng Tian, Zhongyuan Wang
Date: v3 May 15 2026 (orig Feb 2025)
Venue: arXiv preprint (cs.CL / cs.LG)

PROBLEM
Xu et al. 2024 proved via computability theory that any LM hallucinates on an infinite
input set regardless of training quality or algorithm. This paper argues that result has
zero practical content: an infinite error set can simultaneously have arbitrarily small
probability under any realistic input distribution.

CORE ARGUMENT
Computability-theoretic inevitability (Theorem 8) and statistical negligibility (Theorem 21)
are compatible. Theorem 8 lives in worst-case/adversarial analysis; Theorem 21 lives in
average-case analysis over realistic input distributions. The paper proves existence of
an LMT achieving hallucination probability < epsilon_H (with confidence >= 1 - epsilon_T)
given sufficient qualified training data, for any fixed epsilon_H, epsilon_T > 0.

Inputs (to the theoretical framework):
mu             | probability measure | Delta(Sigma*) | input distribution over string space
F0             | acceptable output map | function Sigma* -> 2^(Sigma*) | mandatory
T              | training dataset | m i.i.d. qualified pairs (s_i, y_i) | mandatory
epsilon_H      | hallucination tolerance | (0,1) | target hallucination probability ceiling
epsilon_T      | confidence failure tolerance | (0,1) | 1 - epsilon_T = confidence level
CDF_bar        | CDF lower bound on input lengths | non-decreasing [0,1]-valued | required for UNIFORM negligibility only; optional for non-uniform

Outputs:
h              | language model | computable function Sigma* -> Sigma* | returned by LMT after training
HP_mu(h)       | hallucination probability | scalar in [0,1] | target: HP_mu(A(T)) < epsilon_H
Invariant: Pr_T(HP_mu(A(T)) < epsilon_H) >= 1 - epsilon_T  [Def 16, main guarantee]

KEY RESULTS

Theorem 8 (Computability Lower Bound, Section 3):
For any h in H (any language model), there exists F0 such that h hallucinates on
infinitely many inputs. This is unconditional; no training fixes it.

Definition 16 (Statistical Negligibility):
Hallucinations of LMT A with qualified training data T are (epsilon_H, epsilon_T)-negligible
on mu if: for any non-vacuous F0, for any m >= m_bar,
  Pr_T[ HP_mu(A(T)) < epsilon_H ] >= 1 - epsilon_T

Theorem 21 (Main Positive Result, Section 4):
There EXISTS an LMT A such that:
(1) UNIFORM: for any non-decreasing CDF_bar with lim_{n->inf} CDF_bar(n) = 1,
    hallucinations are uniformly (epsilon_H, epsilon_T)-negligible on P_{CDF_bar}
(2) NON-UNIFORM: hallucinations are non-uniformly (epsilon_H, epsilon_T)-negligible
    on ALL probability measures Delta(Sigma*)

Conditions on Theorem 21:
- Training data must be QUALIFIED (Def 14): Y_i in F0(S_i) with prob 1 (clean labels)
- Training data S_i must be i.i.d. from mu
- m >= m_bar; the required size m_bar is:
    m_bar = ceil( |Sigma*|^(n_bar+1) / (1 - CDF_bar(n_bar)) *
                  ln( |Sigma*|^(n_bar+1) / (2*(1 - CDF_bar(n_bar))) ) )
  where n_bar is smallest n such that 1 - CDF_bar(n) < (1/2) * min{epsilon_H, epsilon_T}
- This is exponential in n_bar (input length threshold) -- tractable only when most
  probability mass is on short inputs

Algorithm used for constructive proof (Algorithm 1, Rote Memorizer):
- Pure dictionary lookup over training pairs
- Returns memorized output if s in dict, else returns ""
- NOT a practical algorithm; serves only to prove existence (lower bound on what is achievable)

Theorem 27 (No Free Lunch, Appendix):
For any LMT A and any training size m, there exist adversarial F0 and mu such that:
  E_T[ HP_mu(A(T)) ] >= 1/2
  Pr_T( HP_mu(A(T)) >= lambda_H ) >= (1 - 2*lambda_H) / (2 - 2*lambda_H)
This is the worst-case lower bound. Coexists with Theorem 21 because Theorem 27 uses
adversarially chosen mu; Theorem 21 fixes mu first then bounds error.

WORST-CASE VS OPERATIONALLY-BINDING DISTINCTION (Section 5)
Theorem 8 and Theorem 27 are existential/adversarial: they show a bad mu EXISTS.
Theorem 21 is constructive for a FIXED mu: once mu is fixed (realistic usage distribution),
error probability can be driven below any epsilon with enough clean data.
The operationally-binding question is not "does an infinite error set exist?" but
"what probability mass does that error set carry under realistic mu?"
Practical conclusion: if hallucinations persist in deployment, blame data quality/quantity
or algorithm choice -- NOT innate inevitability.

CRITICAL INVARIANTS (testable assertions)
assert HP >= 0 and HP <= 1  # hallucination probability is a valid probability
assert len(training_data) >= m_bar  # Theorem 21 guarantee requires sufficient data
assert all(y in F0(s) for s, y in training_data)  # qualified data requirement (Def 14)
# Theorem 21 guarantee (statistical, not pointwise):
# assert empirical_HP < epsilon_H  # holds with prob >= 1 - epsilon_T over random T
# No invariant holds for adversarially chosen mu -- Theorem 27 gives HP >= 0.5

FAILURE MODES (from paper)
1. Non-qualified training data (Y_i not in F0(S_i) with prob 1): guarantee breaks entirely
2. Adversarial or distribution-shifted input at test time: Theorem 27 applies, HP >= 0.5
3. CDF of actual inputs heavier-tailed than CDF_bar: m_bar underestimated, guarantee fails
4. m < m_bar: insufficient training data for the desired (epsilon_H, epsilon_T) pair
5. Rote Memorizer on long inputs: m_bar grows exponentially in n_bar, infeasible in practice

IMPLEMENTATION PITFALLS (do NOT do this)
Do not conclude Theorem 8 means hallucinations cannot be reduced -- it only proves an
infinite input set exists, not that it has positive probability under real distributions.
Do not apply Theorem 21 under distribution shift (training mu != deployment mu).
Do not use Rote Memorizer in production -- it is an existence proof, not a recipe.
Do not interpret (epsilon_H, epsilon_T)-negligibility as a pointwise per-input guarantee --
it is a probabilistic statement over random training draws.
Do not ignore the data qualification requirement -- noisy labels invalidate the theorem.

COMPLEXITY
Time (Rote Memorizer training): O(m) dictionary construction
Space (Rote Memorizer): O(m * max_input_length) for dictionary
Required m_bar: exponential in n_bar (input length threshold) -- combinatorial explosion
No parallelism constraints stated. No GPU/compute assumptions. Purely theoretical.

ELAI RELEVANCE
This paper does NOT motivate a net-new ELAI mechanism. It provides theoretical grounding
for why GD-4 (bound FP on >=3 real runs) is the right doctrine:
- GD-4 already operationalizes the "fix realistic mu, measure real HP" principle that
  Theorem 21 formalizes. The gate fires on real execution artifacts, not adversarial inputs.
- The paper supports the off-agent artifact requirement (R4 wiring-enforcement.md):
  worst-case analysis (Theorem 8) is the wrong evaluation frame; operationally-binding
  failure probability on real input distribution is the right frame.
- One potential reinforcement: the "qualified training data" requirement (Def 14) maps
  onto ELAI's context-quality filtering in the sidecar pipeline. If the paper's conditions
  are taken seriously, noisy prompts fed to the sidecar constitute unqualified training
  analogues and should be pre-filtered or flagged rather than generating hallucination-prone
  outputs.
- No new gate calibration algorithm is implied; the constructive proof is an existence
  argument via Rote Memorizer, not a practical algorithm.

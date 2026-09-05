Algorithm: UniCR (Unified Confidence Calibration and Risk-Controlled Refusal)
ArXiv: 2509.01455
Paper: Trusted Uncertainty in Large Language Models: A Unified Framework for Confidence Calibration and Risk-Controlled Refusal
Version: v3 (fetched 2025-12-29 per HTML metadata)
Authors: Oehri et al. (University of Liechtenstein et al.)
D-16 scope: conformal refusal ground truth

NOTE ON epsilon=0.1 LABEL: The paper's primary operating point is rho=5% selective risk budget.
The label "epsilon=0.1" in D-16 maps to alpha=0.1 in the paper's CRC formulation (Eq. 5),
i.e. the tolerated violation probability. At alpha=0.1 the conformal threshold uses the
0.9-quantile of nonconformity scores. The paper does not use the symbol epsilon; D-16 must
translate: epsilon = alpha = 0.1 -> 1-alpha = 0.90 quantile.


Inputs (Algorithm 1: UniCR-Train):
dev_set       | list of (x_i, y*_i) pairs      | n samples    | mandatory; must NOT overlap with test set; internal 20% split reserved for calibration
base_llm      | callable (generate)             | any LLM      | mandatory; weights frozen; only calibration head trained
retriever     | callable or None                | RAG optional | optional; required for RAG compatibility features
tools_verifier| callable or None                | optional     | optional; enables verifier/tool signal feature
rho           | float                           | (0, 1)       | user-specified selective risk budget; paper primary example rho=0.05
alpha         | float                           | (0, 1)       | CRC violation probability; alpha=0.1 = D-16 epsilon=0.1; lower = stricter guarantee = lower coverage
K             | int                             | {1, 3, 5}    | self-consistency samples; default K=3 (K=1 for latency-constrained)
lambda_reject | float                           | [0, 1]       | rejection cost; Bayes-optimal threshold = 1-lambda_reject (Section 3.1)
budget_K      | int                             | >=1          | alias for K in Algorithm 1 spec

Inputs (Algorithm 2: UniCR-Infer):
query_x       | str or structured input         | single query | mandatory
g_theta       | trained calibration head        | serialized   | mandatory; must be serialized with tau and temperature T
tau           | float                           | [0, 1]       | mandatory; computed by Train via Eq. 4 (validation) or Eq. 5 (CRC)
K_infer       | int                             | {1, 3, 5}    | default K=3; K=1 acceptable for latency-critical paths

Evidence features (z_i, extracted inside Train and Infer):
seq_likelihood | float                          | scalar       | mean log-prob of generated sequence (ℓ̄); requires logit access; set to 0 for API-only
seq_entropy    | float                          | scalar       | mean per-token entropy (H̄); requires logit access; set to 0 for API-only
sc_dispersion  | float                          | scalar       | self-consistency disagreement across K samples; always available
rag_compat     | float                          | scalar       | retrieval evidence compatibility score; 0 if no retriever
verifier_score | float                          | scalar       | binary or graded tool/verifier output; 0 if no verifier
feature_dim    | int                            | 8-32         | total evidence feature dimensionality d (Section 3.9)


Outputs (Algorithm 1: UniCR-Train):
g_theta       | calibration head (logistic or small MLP) | R^d -> [0,1] | must be serialized with T and tau
tau           | float                           | [0, 1]       | refusal threshold; from Eq. 4 (validation) or Eq. 5 (CRC)
T             | float                           | >0           | learned temperature for likelihood-derived features

Outputs (Algorithm 2: UniCR-Infer):
decision      | ("answer", y) or ("refuse", reason_tag) | --      | answer if c(x) >= tau; refuse if c(x) < tau
c_x           | float                           | [0, 1]       | calibrated correctness probability from g_theta(z(x))
reason_tag    | str or None                     | enum         | on refusal: "low_evidence", "high_dispersion", "tool_inconsistency", "verifier_rejection" (Section 3.11)


Hyperparameters:
K             | 3         | {1, 3, 5}   | self-consistency samples; paper reports K=3 for main results; K=1 recovers substantial gain with minimal latency; HIGH sensitivity to coverage/quality tradeoff
rho           | 0.05      | (0, 1)      | selective risk budget; paper primary result at rho=0.05; lower rho = lower coverage
alpha         | 0.10      | (0, 1)      | CRC violation probability (D-16 epsilon); alpha=0.10 -> 0.90-quantile in Eq. 5; MODERATE sensitivity; lower alpha = higher tau = more refusals
lambda_reject | task-dep  | [0, 1]      | rejection cost; affects Bayes decision boundary; HIGH sensitivity to task cost structure
T (temperature)| learned  | >0          | calibration head temperature; jointly optimized via backprop on Eq. 2
alpha_ece     | in Eq. 2  | >0          | ECE regularization weight in calibration objective; paper uses adaptive variant
beta          | in Eq. 3  | >0          | selective training coverage penalty weight (optional, Section 3.3.2)
kappa         | in Eq. 3  | (0, 1)      | minimum coverage target in selective training
top_k_rag     | 5         | >=1         | retrieval passages returned (Section 4.2)
calib_split   | 0.20      | (0, 1)      | fraction of dev set held out for conformal threshold computation (Section 4.1)


Critical invariants (testable assertions):
# Eq. 5: conformal threshold is (1-alpha)-quantile of nonconformity scores on ERRORS ONLY
assert tau_conf == numpy.quantile([1 - c_i for c_i, r_i in zip(c_scores, labels) if r_i == 0], 1 - alpha)

# Calibration head output is a proper probability
assert all(0.0 <= c <= 1.0 for c in c_scores)

# Refusal decision is consistent with threshold
for c, decision in zip(c_scores, decisions):
    if decision == "answer":
        assert c >= tau
    else:
        assert c < tau

# Selective risk at rho=0.05 operating point (empirical check on held-out test set)
answered = [(c, r) for c, r, d in zip(c_scores, labels, decisions) if d == "answer"]
if len(answered) > 0:
    sel_risk = sum(1 - r for _, r in answered) / len(answered)
    assert sel_risk <= rho + 0.01  # allow 1pp slack for finite-sample fluctuation

# Calibration and test sets must be disjoint (no data leakage onto conformal quantile)
assert len(set(calib_ids) & set(test_ids)) == 0

# Temperature T must be serialized alongside g_theta and tau for deterministic deployment
assert hasattr(model_artifact, "tau") and hasattr(model_artifact, "T")


Complexity:
Training calibration head | O(n * d) per epoch, d in [8,32] | O(d) space | parallelism: single GPU or CPU in under 1 minute (Section 4.3)
Evidence extraction (Train) | O(n * K) LLM calls + O(n * top_k) RAG calls | O(K) active samples | K LLM calls are parallelizable
Conformal threshold (Eq. 5) | O(n_calib) sort | O(n_calib) space | no parallelism constraint
Inference (Infer) | O(K) LLM calls + O(top_k) RAG calls | O(K) | K samples are parallelizable; K=1 eliminates SC overhead
CRC recomputation on distribution shift | O(n_calib) per bucket | O(n_calib) | must re-run if deployment distribution shifts

Failure modes:
Distribution shift: CRC guarantee holds (risk is controlled) but coverage can drop sharply; expected by design (Section 5.9)
Exchangeability violation: hard distribution shift between calibration and deployment violates CRC assumption; guarantee no longer holds (Section 3.17)
Entailment estimator miscalibration: semantic correctness target r_tilde biased if entailment model is miscalibrated; propagates to c(x) (Section 5.9)
High-likelihood hallucinations: model assigns high seq_likelihood to wrong answers; multi-evidence head mitigates but does not eliminate (Section 5.4)
Adversarial phrasing: confidently wrong verbalizations observed on adversarially phrased questions; verbalized confidence alone insufficient (Section 5.9)
Small calibration set: quantile estimate has high variance; use quantile smoothing variant and bucket by evidence regime (Section 3.10)
MLP calibration head overfitting: two-layer MLP can worsen ECE on small calib sets unless isotonic post-maps applied; prefer simple logistic head (Section 5.3)
K=1 with no retrieval: removing both SC and RAG degrades AURC significantly per ablation (Section 5.3)


Implementation pitfalls (do NOT do this):
Do NOT use calibration samples to also tune g_theta hyperparameters; this invalidates CRC quantile (Section 4.1 learn-then-test warning)
Do NOT apply a single global threshold when RAG coverage is heterogeneous; bucket-CRC gives finer control (Section 3.10)
Do NOT serialize g_theta without T and tau; deployment without these causes non-deterministic thresholding (Section 3.15)
Do NOT substitute validation-thresholding (Eq. 4) for conformal-thresholding (Eq. 5) when distribution-free guarantee is required; Eq. 4 can drift under shift (Section 5.1)
Do NOT rely on seq_likelihood alone as the confidence signal; high likelihood does not imply correctness (Section 5.4 error analysis)
Do NOT use the same dev split index for both head training and conformal quantile computation; must be disjoint (Section 4.1)
Do NOT skip isotonic post-map when using MLP head on small calibration sets (Section 5.3)
Do NOT interpret alpha (violation probability) as rho (selective risk budget); they are different quantities controlling different aspects of the guarantee


D-16 operating point translation:
epsilon = 0.1 in D-16 corresponds to alpha = 0.1 in the paper
At alpha=0.1: tau_conf = 1 - Quantile_0.90({1 - c_i : r_i = 0})
Recommended calib_split = 0.20 of dev set
Recommended K = 3 (or K=1 for latency-constrained path)
Recommended rho = 0.05 as primary operating point for selective risk
Primary result: at rho=0.05, achieved test selective risk = 4.8-5.3% (within expected binomial fluctuation)
Coverage gain vs validation-thresholding: +2 to +6 percentage points at matched risk

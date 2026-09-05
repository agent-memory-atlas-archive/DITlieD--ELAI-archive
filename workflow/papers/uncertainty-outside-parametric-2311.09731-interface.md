Algorithm: UnknownBench Uncertainty Quantification and Refusal Framework
ArXiv: 2311.09731
Paper: Examining LLMs' Uncertainty Expression Towards Questions Outside Parametric Knowledge
Authors: Genglin Liu, Xingyao Wang, Lifan Yuan, Yangyi Chen, Hao Peng (UIUC)
Version: arXiv v2, Feb 16 2024

Scope for D-16: This paper provides the empirical grounding for refusal-as-calibrated-response, confidence threshold design, and the measurement methodology for outside-parametric-knowledge detection.


Inputs:

question_text | str | single natural language question | required; must be well-formed
answerability_label | int | {0=answerable, 1=unanswerable} | required for evaluation; ground truth set by construction (NEC) or human verification (FalseQA)
confidence_elicitation_method | enum | {verbalized_regression, verbalized_classification, logit_entropy, prompt_perplexity} | required; verbalized_regression (0-10 scale) is the model-agnostic canonical method
in_context_examples | list[str] | optional few-shot examples | optional; paper finds they influence scoring pattern and recommends omitting for cross-model consistency

Dataset partitions used:
NEC (Non-Existent Concepts) | 2078 unanswerable + 2072 answerable | non-existent atomic concepts injected into templates
FalseQA | 2365 unanswerable + 2365 answerable | false premises between real entities
RefuNQ | 2173 unanswerable + 2266 answerable | NaturalQuestions perturbed by NEC noun substitution via spaCy


Outputs:

refusal_rate | float | [0.0, 1.0] | proportion of questions where model abstains; measured by lexical keyword matching (abstention/apology/denial keywords); human-verified correspondence
qa_accuracy | float | [0.0, 1.0] | on answerable partition of RefuNQ only; lexical match of short annotation in response
verbalized_confidence | int | [0, 10] | model self-reported confidence score elicited by regression prompt; 0=least confident, 10=most confident
refusal_delta | float | unbounded signed; typically positive | refusal_rate(unanswerable) - refusal_rate(answerable); primary calibration quality metric; ideal model maximizes this
ECE_accuracy | float | [0.0, 1.0] | sum_m |acc(m) - conf(m)| * n_m / N; Appendix D eq. 1
ECE_refusal | float | [0.0, 1.0] | sum_m |r(m) - conf(m)| * n_m / N; Appendix D eq. 2; conf mapped to probability by even binning
perceived_confidence | int | {1,2,3,4,5} | GPT-4 proxy rating of confidence conveyed by response tone; mapped to {0.2,0.4,0.6,0.8,1.0} for ECE; separate from verbalized


Hyperparameters (confidence elicitation):

confidence_scale | default=10 | range=[1-5 or 0-100] tested; 0-10 chosen | sensitivity: scale affects distribution shape; 0-10 without in-context examples found most cross-model stable (Appendix E)
in_context_examples_count | default=0 | range=[0, 3+] | sensitivity: HIGH; even 3 examples shift scoring pattern; omit for model-agnostic use
elicitation_position | default=pre-response | options={pre-response, post-response} | pre-response preferred; post-response adds posterior confidence (Appendix E Fig 4)
ECE_bin_count | default=derived from discrete scores | M bins where M = number of unique confidence values | confidence scores mapped to evenly distributed probabilities before ECE calculation
lexical_refusal_keywords | no default listed | paper uses abstention/apology/denial keyword set | human evaluation confirms high correspondence with human judgment; exact keyword list not published in main paper


Critical invariants (testable assertions):

# Calibration direction: unanswerable refusal must exceed answerable refusal for a calibrated model
assert refusal_rate_unanswerable > refusal_rate_answerable, "Refusal delta must be positive for calibrated model"

# Verbalized confidence and refusal must be negatively correlated on unanswerable questions (Section 3.3)
# i.e. higher stated confidence -> lower refusal rate on unanswerable questions
# Verified empirically: assert pearsonr(confidence_bins, unanswerable_refusal_at_bin)[0] < 0

# Verbalized confidence and accuracy must be positively correlated on answerable questions (Section 3.3)
# Verified empirically for proprietary models: assert pearsonr(confidence_bins, accuracy_at_bin)[0] > 0

# ECE formula invariant: bins sum to N
assert sum(n_m for m in bins) == N_total, "ECE bins must cover all predictions"

# Confidence-to-probability mapping for perceived (GPT-4 proxy) uses 5-level linear map
assert set(perceived_confidence_mapped.values()) == {0.2, 0.4, 0.6, 0.8, 1.0}

# Verbalized confidence 0-10 mapped to evenly distributed probability bins for ECE
# Each bin width = 1/10; prob(c) = (c + 0.5) / 10 or uniform bin assignment

# The best-performing model (GPT-4) still fails to refuse ~35% of unanswerable questions
# D-16 implication: a refusal threshold of conf <= K will produce false negatives even at K=5
assert best_model_unans_refusal_rate <= 0.651, "No model exceeds 65.1% refusal on UnknownBench (GPT-4 ceiling, Table 1)"

# Base models without SFT/RLHF refuse < 8% of unanswerable questions regardless of confidence elicitation
assert base_model_unans_refusal_rate < 0.08, "Base models without instruction tuning have near-zero refusal ability (Table 1)"


Complexity:

Inference: O(1) per query; standard LLM forward pass
Confidence elicitation overhead: +1 LLM call per query (pre-response verbalized method) or +0 if post-response
Lexical refusal detection: O(K) per response where K = keyword count; negligible
ECE computation: O(N * M) where N = dataset size, M = number of confidence bins; negligible
Dataset construction (NEC): O(concepts * templates); NEC uses 10-15 templates per concept, ~1750 concepts total
Parallelism: fully parallel across queries; no inter-query dependencies


Failure modes (cited by paper):

1. Fabrication under unknown input: LLMs default to generating plausible-sounding but hallucinated answers rather than refusing (Section 3.2, Appendix C). GPT-4 still fabricates for ~35% of unanswerable questions.

2. Confidence-without-certainty (Section 3.4): LLMs produce responses that sound confident in tone even when verbalized confidence is low. Perceived confidence (measured by GPT-4 proxy) is WORSE calibrated than verbalized confidence. Direct implication: a refusal system based solely on response-tone parsing will be miscalibrated.

3. Verbalized confidence inconsistency for small models: 7B models (Llama-2 7B, Vicuna 7B) do not effectively follow the confidence elicitation instruction; their verbalized confidence distributions are collapsed (mostly 9-10) regardless of answerability (Appendix H.1, Fig 6). Confidence scores from <13B models should be treated as unreliable.

4. SFT/RLHF hurts logit-based calibration while helping verbalized calibration (Appendix H.3): instruction finetuning narrows token entropy distributions, making logit-based signals less discriminative between answerable/unanswerable. Do not use token entropy as the primary uncertainty signal for instruction-tuned models.

5. Scale dependency: refusal calibration degrades sharply below 13B parameters. The 70B Llama-2 chat model is the smallest open-source model showing multi-level confidence expression (Appendix H.3).

6. In-context example sensitivity: confidence scores shift when few-shot examples are provided (Appendix E). Including examples in production prompts invalidates cross-session threshold comparisons unless examples are frozen.

7. Lexical matching misses semantically correct answers with different surface form (Limitations section): accuracy metric undercounts true helpfulness. D-16 should not rely solely on lexical matching for accuracy measurement.


Implementation pitfalls (do NOT do this):

Do NOT use token entropy or prompt perplexity as the primary uncertainty signal for instruction-tuned models; these signals are less discriminative post-SFT (Appendix E, H.3).

Do NOT treat verbalized confidence as equivalent to perceived confidence; the paper shows these diverge and perceived calibration is considerably worse (Section 3.4).

Do NOT set a binary refusal threshold at confidence <= 5 and expect ~50% refusal on unanswerable queries; actual refusal rates at mid-confidence are model-dependent and non-monotone (see Fig 2, 9, 10).

Do NOT rely on small models (<13B parameters) for confidence calibration; their verbalized scores cluster at extremes and do not discriminate answerability.

Do NOT include in-context examples in confidence elicitation prompts unless those exact examples are fixed across all production calls; they shift the distribution (Appendix E).

Do NOT conflate refusal rate with honesty quality; a model refusing 100% of all questions (including answerable ones) scores high on unanswerable refusal but is useless. The correct metric is Refusal Delta = refusal_rate(unanswerable) - refusal_rate(answerable).

Do NOT use a single threshold calibrated on one task type (NEC vs FalseQA vs RefuNQ) and apply it to another; refusal rates differ by task type for the same model.

Do NOT assume that higher instruction-following ability implies better refusal; Mistral-7B-instruct achieves 53.8% unanswerable refusal but only 35.0% accuracy, reflecting a helpfulness-honesty tradeoff (Table 1).


Threshold design notes for D-16 (conformal refusal mechanism):

The paper provides no conformal prediction or PAC-style threshold derivation. The empirical data in Table 1 and Figures 2/9/10 are the primary grounding:

Observed operating points (verbalized confidence 0-10, unanswerable refusal rate):
- At confidence <= 3 (rough reading from Fig 2 claude-2 NEC): refusal rate ~0.8-1.0 on unanswerable
- At confidence >= 7: refusal rate near 0 on unanswerable for proprietary models
- Answerable refusal at confidence <= 3: ~0.1-0.3 (false positive floor)

D-16 conformal approach implication: the confidence score is a noisy proxy. A threshold at confidence <= 4 on a well-aligned model (Claude/GPT-class) yields approximately:
- True positive (correct refusal of unanswerable): ~70-85% (reading Fig 2)
- False positive (refusing answerable): ~10-25%
These are rough empirical bounds, not formal guarantees. Conformal calibration requires held-out data to set coverage.

The paper's primary contribution for D-16: establishes that (a) refusal is learnable and threshold-tunable via confidence elicitation, (b) verbalized 0-10 scale is the model-agnostic standard, (c) Refusal Delta is the correct optimization target, and (d) verbalized and perceived confidence diverge and must both be tracked.

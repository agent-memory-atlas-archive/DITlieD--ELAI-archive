Algorithm: LLM Self-Correction via Prompted Feedback (Critical Survey)
ArXiv: 2406.01297
Paper: When Can LLMs Actually Correct Their Own Mistakes? A Critical Survey of Self-Correction of LLMs
Authors: Ryo Kamoi, Yusen Zhang, Nan Zhang, Jiawei Han, Rui Zhang
Venue: Transactions of the Association for Computational Linguistics (TACL) 2024, vol 12 pp 1417-1440
Reliability: HIGH (peer-reviewed TACL)
Local file: F:\.ELAI_workflow\.claude\papers\2406.01297.html

CITATION MISMATCH WARNING — READ FIRST:
The B-41 description cites this paper with the claim "verifier prompt scopes determine
recall/precision tradeoff; per-level scoping improves both vs single global auditor."
That claim does NOT appear in this paper. The paper does not discuss verifier prompt
scope granularity, per-level vs global auditor design, or any experiment showing
per-level scoping improves recall and precision jointly. The mismatch is material —
the cited theoretical justification for splitting into per-tier auditors is not
grounded in this paper as described.

What this paper DOES establish (verified from full text):
1. Feedback generation is the bottleneck for self-correction (Section 7).
2. "No prior work demonstrates successful self-correction with feedback from prompted
   LLMs, except for studies in tasks that are exceptionally suited for self-correction"
   (Abstract).
3. Self-correction works in three narrow conditions: tasks where verification is
   easier than generation, tasks with reliable external tool feedback (e.g. code
   interpreters), large-scale fine-tuning (100K+ examples).
4. The paper warns that prior work conflates different research questions leading to
   "impractical frameworks or unfair evaluations that over-evaluate self-correction."

Related paper with actual recall/precision scope findings:
ArXiv: 2404.03602
Title: Evaluating LLMs at Detecting Errors in LLM Responses
Authors: Ryo Kamoi et al.
Venue: COLM 2024 (NOT TACL)
Key finding (Section 4.1, Figure 5): "stronger LLMs detect errors in LLM responses
with higher precision but with lower recall" — a documented precision/recall tradeoff.
Key finding (Section 4.3): prompt wording changes recall by 16.9 +/- 20.3% but
precision changes are inconsistent — no reliable direction.
Key finding (Section 4.3-4.4): per-criterion vs unified prompt NOT tested. Single
unified prompt used across all four error categories. No experiment compares
per-criterion scoped prompts to a global prompt.

Inputs (2406.01297 self-correction framework):
initial_response | text | arbitrary length | LLM-generated, "best-possible" per RQ1/RQ2
feedback_source | enum | {prompted_LLM, external_tool, fine_tuned_model} | determines whether self-correction is viable
task_type | enum | verifiable vs non-verifiable | gates expected success

Outputs:
refined_response | text | arbitrary | no quality guarantee without external feedback
feedback | text | arbitrary | reliable only with external tools or fine-tuning

Hyperparameters (2404.03602 error detection):
prompt_wording | varies | 4 tested variants | HIGH sensitivity: 16.9% recall swing
option_ordering | {error_first, correct_first} | 2 options | HIGH: 16.0% recall swing
evaluation_scope | {unified, per-criterion} | paper only tests unified | untested for per-criterion

Critical invariants (testable assertions from verified paper content):
# 2406.01297 invariants
assert feedback_source != "prompted_LLM" or task_is_exceptionally_easy, \
    "prompted LLM feedback does not produce reliable self-correction (TACL 2024 abstract)"

assert not (uses_oracle_labels_in_feedback and claims_self_correction), \
    "oracle feedback violates RQ1 information symmetricity constraint (Section 3.1)"

# 2404.03602 invariants
assert not (model_is_stronger and recall_improves), \
    "stronger models trade recall for precision (COLM 2024 Section 4.1, Figure 5)"

assert abs(recall_variant_A - recall_variant_B) <= 0.40, \
    "prompt wording swings recall up to ~37pp; treat recall as prompt-sensitive (Section 4.3)"

# B-41 claim check
assert False, \
    "per-level scoping improves both recall AND precision: NOT SUPPORTED by either paper"

Complexity:
Not applicable — survey paper, no algorithmic complexity defined.

Failure modes (from 2406.01297):
Oracle information misuse: using ground-truth labels to generate feedback then claiming self-correction
Weak initial prompts: comparing self-correction to deliberately weak baseline (unfair gain)
Asymmetric external knowledge: search engines used only in correction step, not generation step
Fine-tuning data contamination: models fine-tuned on correction data evaluated on overlapping test sets

Implementation pitfalls — do NOT do this:
Do not claim per-level auditor scoping improves both recall and precision citing 2406.01297 — not supported
Do not use prompted LLM feedback for self-correction and expect reliable improvement — paper shows this fails
Do not compare self-correction output to a weak baseline without also comparing to strong non-correction baseline (RQ3)
Do not assume broader prompt = higher recall or narrower prompt = higher precision — 2404.03602 shows inconsistent direction
Do not treat a single recall measurement as stable — prompt wording alone can shift it ~17-37pp

B-41 grounding assessment:
Claim: "verifier prompt scopes determine recall/precision tradeoff. Per-level scoping improves both vs single global auditor."
Status: UNGROUNDED with respect to this paper (2406.01297).
Closest real finding: 2404.03602 Section 4.1 shows a precision/recall tradeoff exists for LLM error detectors, but the direction is inverse (stronger model = higher precision, lower recall), not the additive improvement claimed.
The specific claim that per-level scoping improves BOTH simultaneously is not supported by either Kamoi 2024 paper. This would require a direct ablation experiment comparing per-tier vs unified auditor prompts on the same benchmark, which does not exist in these papers.

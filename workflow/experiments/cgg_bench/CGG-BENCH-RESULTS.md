> Historical experiment record. Status and measurements below are preserved as
> recorded; no experiment was rerun for publication. Raw datasets and run state
> are omitted, so this file is not a reproducible benchmark release.

CGG HALLUCINATION BENCHMARK — RESULTS
=====================================
Date: 2026-06-27. Author: main LLM (Opus 4.8). Status: DATA PENDING (run in flight).
Grounding: OWN MEASURED DATA (this experiment) + code just read (CGG doc L-1..L-10,
RLM experiment PART 7-9). Every number below is ELAI-measured on SimpleQA, never a
paper constant (ID-5 / KD-1).

WHAT THIS MEASURES
The CGG (Causal Grounding Gate, L-plan) is design-only (no code shipped). This
benchmark implements a faithful black-box version of its LOAD-BEARING, model-
agnostic hallucination-reduction core and measures the hallucination-rate delta it
produces, baseline vs gated, across models. It does NOT implement the full L-1..L-10
(no DKG citation pre-binding, no counterfactual do-test on repo claims, no curiosity-
driven collection) — those need the ELAI runtime. It implements exactly the part the
CGG doc calls the genuine net-new producer (L-4 P3 semantic entropy) plus the cheap
predictors and the abstain policy gate (L-2/L-7), which is the part that actually
converts confident-wrong answers into abstentions.

THE CGG CORE, AS BENCHMARKED (black-box, model-agnostic)
  P3 inter-seed SEMANTIC ENTROPY (L-4): resample N answers, cluster by meaning
      (deterministic strong normalization), disagreement = risk. Grounding:
      Farquhar et al Nature 2024 (semantic entropy); Manakul SelfCheckGPT 2303.08896.
  P2 paraphrase stability / SAC3 (J-85 / Zhang 2311.01740): reword the question,
      check whether the majority answer survives.
  CONVERGENCE >=2 signals agree (D-3): A3 discounts confidence only when P3 AND P2
      both flag.
  POLICY GATE high-risk -> ABSTAIN (L-7): the trust contract — abstain, never fabricate.
  CROSS-FAMILY resample (D-13): A4 resamples across Meta/Mistral/Google to catch
      within-family shared-confident-wrong bias single-model resampling cannot see.

ARMS
  A0_baseline       single temp-0 answer, no gate (raw hallucination reference).
  A1_verbalized     model self-reports a 0-100 confidence (the cheap competitor signal
                    CGG must beat).
  A2_self_consistency  P3 over 5 single-model resamples; majority answer; confidence =
                    self-consistency (top-cluster fraction).
  A3_cgg_full       A2 + P2 paraphrase convergence penalty.
  A4_cross_family   P3 across 3 different pretraining families (ensemble gate).
  G_a0ans+sc_gate   (derived) A0's ANSWER gated by A2's consistency signal — isolates
                    the SIGNAL's selective value from majority-voting's answer lift.
  G_a0ans+cgg_gate  (derived) A0's answer gated by A3's converged confidence.

DATASET + GRADER
  SimpleQA (openai/simple-evals, MIT): 4326 short closed-book factual questions, single
  gold target. Stratified random sample of n=300 (seed 1729), split 150 CALIBRATION /
  150 TEST. The abstain threshold for the single reported operating point is chosen on
  CALIBRATION and evaluated on the disjoint TEST split — no test-set tuning.
  Grader: official SimpleQA GRADER_TEMPLATE verbatim, mistral-small-latest at temp 0,
  on a provider separate from the answer models. CORRECT / INCORRECT / NOT_ATTEMPTED.
  Grader cross-validated against a second family in the audit.

METRICS
  hallucination rate = INCORRECT / total  (THE headline — confident wrong answers)
  accuracy           = CORRECT / total
  attempted-accuracy = CORRECT / (CORRECT + INCORRECT)  (quality when it answers)
  abstention         = NOT_ATTEMPTED / total
  AURC               = area under risk-coverage (lower = better signal)
  Wilson 95% CIs on rates.
  PRIMARY VERDICT = the risk-coverage curve: abstaining-on-everything trivially gives
  0 hallucination, so CGG is only "worth it" if its confidence signal lets you answer
  at LOWER error for a given coverage than the trivial baselines (A0 full-attempt and
  A1 verbalized confidence).

MODELS
  weak:    groq/llama-3.1-8b-instant   (the J-55 small-lane axis; high baseline halluc.)
  capable: cerebras/gpt-oss-120b       (the strong-lane axis)

RESULTS
  [pending run completion — analyze.py writes summary.json; tables inserted here]

VERDICT
  [pending]

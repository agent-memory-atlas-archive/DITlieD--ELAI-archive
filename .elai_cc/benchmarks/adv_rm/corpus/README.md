Adversarial-RM corpus directory.

This directory holds per-round adversarial example corpora farmed by the D-28
adversarial example farmer. Each round writes one `corpus_round_<N>.jsonl`
file with the following record shape:

    {
      "id": "adv_<round>_<seq>",
      "prompt": "<eval prompt text>",
      "response_sft": "<baseline SFT response>",
      "response_adv": "<adversarial response>",
      "z_score": 2.47,
      "ppm_score_proxy": 0.84,
      "ppm_score_gold": 0.31,
      "gold_verdict": "pending",
      "provenance": "CanaryCorpus" | {"ElaiSynthetic": {"author": "..."}},
      "round": 1
    }

Sample cap rationale (paper-grounded):
- arxiv:2504.06141 §5 reports performance degradation past ~1000 samples per
  round (diversity saturation). The farmer enforces a hard cap of 1000 per
  round and a max of 2 rounds (round 3+ yields near-zero new adversarial
  examples).
- Z>1.96 filter (paper §3.2; 95th pct of pi_SFT uncertainty distribution)
  excludes low-uncertainty candidates that add noise without improving
  robustness.

Provenance:
- `CanaryCorpus` records derive from the D-7 cross-domain canary fixtures
  + Adv-RM mutation pipeline.
- `ElaiSynthetic { author }` records are hand-authored or generated under
  attributed ELAI authorship per the anti-mock-data rule (no unattributed
  synthetic data in the production corpus).

Live runs are RUNPOD-PENDING. See workflow/remote-setup/ppm_finetune.sh for
the fine-tune execution path; the host-side scaffolding lives in
.elai_cc/python/elai_inference/ppm_adversarial.py.

Calibration note: the D-28 acceptance gate is paper_floor_pp = 1.0 (NOT the
plan-text +5pp; paper reports +0.7pp overall / +2.86pp Chat-Hard maximum).
Documented in workflow/reasoning/D-28-decisions.md DECISION-3.

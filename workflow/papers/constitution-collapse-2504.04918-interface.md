Algorithm: Constitutional AI on Small Language Models (Self-Critique Collapse Analysis)
ArXiv: 2504.04918
Paper: Constitution or Collapse? Exploring Constitutional AI with Llama 3-8B
Authors: Xue Zhang (2025-04-07)
Scope tag: D-13_adversary_role_cross_family

Summary for D-13:
Small-model self-adversary is empirically unsafe. Same-family recursive self-critique on sub-10B
models collapses during fine-tuning. The authors use an external model (GPT-4o) for preference
labeling precisely because Llama 3-8B cannot reliably evaluate its own outputs. Self-improvement
is described as an emergent property absent below ~52B. Cross-family evaluation is the paper's
implicit recommendation and the authors' explicit implementation choice.

Pipeline (5 steps):
  1. Instruction-tune on Alpaca-GPT4 dataset (baseline SFT model)
  2. Critique phase: model self-evaluates harmful responses against constitutional principles
  3. Revision phase: model rewrites response targeting harmlessness
  4. SFT on (harmful-prompt, revised-response) pairs
  5. DPO on preference pairs where GPT-4o (EXTERNAL, cross-family) labels less-harmful response

Inputs:
  harmful_prompt       | text           | any toxic/red-team prompt from HeX-PHI dataset
  constitution         | set of rules   | natural language principles (exact set not published)
  base_model           | LLM weights    | Llama 3-8B, Alpaca-GPT4 fine-tuned
  training_data_sft    | conv rounds    | 10k train / 1k test (11k total generated)
  preference_pairs     | paired texts   | 5k train / 1k test (6k total, 3% excluded as ambiguous)
  external_evaluator   | LLM            | GPT-4o used for preference labeling (NOT self-model)

Outputs:
  harmlessness_delta   | float          | Attack Success Rate: 71% -> 42% (-40.8%) on HeX-PHI
  helpfulness_delta    | float          | MT-Bench avg: 6.05 -> 5.46 (-9.8%)
  turn1_score          | float          | 6.84 -> 6.63 (-2.7%)
  turn2_score          | float          | 5.28 -> 4.28 (-18.8%) [larger degradation on multi-turn]
  collapse_artifact    | bool           | repetition loop triggered if emoji noise in SFT data

Hyperparameters:
  sft_rounds           | 11000          | 10k/1k split         | not sensitivity-tested
  rl_pairs             | 6000           | 5k/1k split          | 3% ambiguity-excluded
  ambiguity_threshold  | ~3%            | excluded pair ratio   | implicit; no formal metric
  optimizer            | DPO            | replaces PPO/reward-model | reduces training effort
  external_eval_model  | GPT-4o         | cross-family labeler  | critical; same-model labeling not tested

Critical invariants (testable assertions):
  assert attack_success_rate_after < attack_success_rate_before * 0.70  # 40.8% reduction observed
  assert helpfulness_after / helpfulness_before > 0.85  # expect ~9.8% degradation, not catastrophic
  assert turn2_score_after / turn2_score_before > 0.75  # turn-2 degrades more (~18.8%)
  assert external_evaluator_family != critique_model_family  # paper uses GPT-4o != Llama; cross-family required
  assert len(ambiguous_pairs_excluded) / len(total_pairs) < 0.05  # 3% exclusion rate is the paper baseline
  assert "emoji" not in sft_training_revisions or collapse_detected  # emoji noise triggers repetition collapse

Complexity:
  SFT stage:     O(N_rounds * T_generate) generate + 1 fine-tune pass over 10k examples
  DPO stage:     O(N_pairs * T_compare) preference labeling by external model + 1 DPO pass over 5k pairs
  Parallelism:   preference labeling by external model is embarrassingly parallel per prompt
  External cost: GPT-4o calls required for preference labeling -- not self-contained

Failure modes (from paper):
  1. COLLAPSE via emoji repetition: SFT data containing repeated emoji patterns causes Llama 3-8B
     to overfit and generate looping sentences. 52B model does not exhibit this.
     Quote: "LLaMA 3-8B lacks the robustness of larger models and has difficulty distinguishing
     between emojis and meaningful sentence content."
  2. CRITIQUE BLINDNESS: smaller models fail to detect harmfulness during critique phase.
     (Corroborated by companion paper 2503.17365: Qwen2.5 "often failed to identify harmful content
     during the critique phase.")
  3. HELPFULNESS COLLAPSE at turn-2: multi-turn helpfulness degrades more than single-turn
     (Turn-2: -18.8% vs Turn-1: -2.7%), suggesting constitutional fine-tuning impairs extended
     dialogue coherence disproportionately.
  4. SELF-IMPROVEMENT EMERGENCE ABSENT: below ~52B, "self-improvement is an emergent property"
     (abstract). Models below this scale cannot be trusted to produce valid self-critiques for
     fine-tuning. No explicit parameter threshold given; the 8B vs 52B comparison is the only
     datapoint (6.5x size ratio correlates with collapse immunity).

Implementation pitfalls (do NOT do this):
  1. Do NOT use same-model self-labeling for preference pairs on sub-10B models. Paper chose
     GPT-4o (external, cross-family) precisely because Llama 3-8B self-evaluation is unreliable.
  2. Do NOT skip data cleaning on SFT revision outputs. Emoji/noise artifacts cause training
     collapse. Authors flagged this as unresolved in the paper scope.
  3. Do NOT treat turn-1 harmlessness improvement as the full picture. Turn-2 helpfulness loss
     is nearly 7x larger than turn-1 loss (-18.8% vs -2.7%).
  4. Do NOT assume constitutional AI is scale-agnostic. The original CAI paper (Anthropic 52B)
     results do not transfer to 8B without architectural mitigations.
  5. Do NOT use ambiguous preference pairs. Pairs where responses are "very similar" are
     noise for DPO; exclude at ~3% threshold.
  6. Do NOT use PPO over DPO at this scale. DPO bypasses reward model training entirely and
     the authors found comparable alignment with significantly reduced training cost.

D-13 grounding conclusions:
  - GROUNDED: small-model self-adversary is unsafe. 8B model self-critique collapses in practice.
  - GROUNDED: cross-family external evaluator (GPT-4o) is the paper's implemented solution.
  - GROUNDED: self-improvement is emergent; absent below ~52B per paper abstract.
  - INFERRED (not directly stated): "cross-family required" for adversary role in D-13 is
    consistent with paper methodology but the paper does not explicitly prohibit same-family
    critique -- it simply never tests it because it chose GPT-4o. The inference is sound but
    is a design extrapolation, not a direct quote.
  - COMPANION: arXiv:2503.17365 corroborates critique blindness on 7-9B models (Qwen2.5)
    but does not address cross-family comparison.

Benchmark datasets used:
  HeX-PHI     | red-team safety eval | Attack Success Rate measurement
  MT-Bench    | helpfulness eval     | Turn-1 and Turn-2 conversation scores
  Alpaca-GPT4 | instruction tuning   | initial SFT baseline

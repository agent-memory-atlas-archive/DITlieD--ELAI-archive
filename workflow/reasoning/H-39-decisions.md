H-39 decisions: abstention-aware promotion + eval scoring
date: 2026-06-08

WHAT
Builds AbstentionAwareScore, wires it into darwin_admit_loop promotion reward +
coding_metrics accumulator + bench scorers (bench_seccodeplt), emits AbstentionScored
SSB row (Pattern-2 flat beacon, SCHEMA 218, TOTAL_VARIANTS 529).

KEY DECISIONS

D-1: Confidence-target formula penalty = t/(1-t), LOCKED t = 0.75.
Grounding: arXiv:2509.04664 Table 2. t=0.75 -> penalty 3x (not 2x as stated in the
plan comment; 2x is the label OpenAI uses for t=0.75 meaning 2x harder to game, but
the actual penalty scalar is 0.75/0.25 = 3.0, meaning confident-wrong scores -3.0).
Correct formula implemented. t is a Rust const DEFAULT_CONFIDENCE_TARGET: f64 = 0.75;
not readable from env, so a self-mod loop cannot tune it toward 0.

D-2: Abstain bit READ from shipped RefusalDecision (GD-1, P0).
conformal_calibration.rs::decide(claim_score, tau) -> RefusalDecision is the sole
source of the abstain bit. abstention_score.rs imports RefusalDecision and reads the
variant arm. No re-derivation. Verified by grep: "RefusalDecision" appears in
abstention_score.rs and NOT in any re-implementation of the nonconformity predicate.

D-3: Abstention-aware alpha normalization in abstention_blended_alpha.
two_gate_promote expects fitness in [0, 1] (ArchiveEntry.fitness: f64). The raw
abstention_aware_score spans [-penalty, +1] = [-3, +1] at t=0.75. Normalise via
(score + penalty) / (1 + penalty) = (score + 3) / 4, mapping [-3, +1] -> [0, 1].
Abstain: (0 + 3)/4 = 0.75. Confident-wrong: (-3 + 3)/4 = 0.0. Correct: (1+3)/4 = 1.0.
Ranking preserved: abstain (0.75) > confident-wrong (0.0) strictly.

D-4: RISK-1 mitigation: eval_log parser with graceful degradation.
eval_log is unstructured text ("suite=math score=0.6 clean"). parse_eval_log_outcome
extracts abstain/wrong/pass markers. When no abstain marker is present, score degrades
to penalty-weighted pass rate (abstention_rate = 0). Per plan explicit allowance.
Frozen fixture in contract test carries explicit "abstain" marker for V2.

D-5: RISK-2 tau interaction.
At normalised alpha = 0.0 (confident-wrong) vs 0.75 (abstaining), the two_gate_promote
margin check (margin = candidate.fitness - incumbent.fitness) gives:
- incumbent at 0.0 (empty archive): margin = 0.0 - 0.0 = 0.0 < tau (0.05) -> BLOCKED.
- abstaining: margin = 0.75 - 0.0 = 0.75 >> tau -> PROMOTED.
The formula's strict separation (0.75 vs 0.0) safely clears the tau gate.
Contract test h39_f3 validates this directly via abstention_blended_alpha.

D-6: RISK-3 bench conformal tau handling.
bench_seccodeplt static corpus has no RefusalDecision channel. Per plan allowance:
abstention_rate = 0, score degrades gracefully to penalty-weighted pass rate.
build_abstention_agg_from_report derives correct/wrong bucket counts from
insecure_rate_after (FN residual). No calibration file dependency; deterministic
from the report fields alone.

D-7: Kill-switch ELAI_ABSTENTION_SCORE_DISABLE (W-2, default-ON, kill-switch only).
Feature is always active unless ELAI_ABSTENTION_SCORE_DISABLE=1 (or non-empty, non-"0").
wire-dark-allowlist.toml row added with type="kill_switch", expiry_date="2027-06-08",
promotion_plan_id="H-39", smoke_test pointing at h39_f3.

D-8: Pattern-2 single-variant flat beacon (mirrors InsecureRateMeasured).
AbstentionScoredPayload fields: suite, mean_score, abstention_rate, confident_wrong_rate,
t, n, seed, corpus_blake3. Off-agent artifact: written by bench runner, not by the agent
under test. Recomputable from frozen fixture + ELAI_SEED (GD-2).

D-9: darwin_admit_loop feature-gate.
darwin_admit_loop is cfg(feature = "darwinian-evolve"). abstention_score.rs is
unconditional (no feature gate). The import of abstention_score from darwin_admit_loop
is correct (feature-gated module can import unconditional module). The contract test
(F3b) is cfg-guarded on the same feature to avoid compilation failure in default-features
builds. F1/F2/F4/F5/F6/F7 tests compile unconditionally.

EXPECTED TEST OUTCOME
- cargo check --workspace --all-targets: GREEN (exit 0).
- action_type_lock lockstep: 2 passed (count_is_locked + golden_matches_runtime), SCHEMA 218/529.
- h39 contract: 10 passed (all 10 F1-F7 tests).
- orchestrator --lib: 3632+ passed, 0 new H-39 failures (45 pre-existing backlog).
- elai-cli: 6 passed, 0 failed.

HONEST RESIDUE (GD-5)
H-39 stops the self-improvement loop REWARDING confident hallucination. It does NOT
eliminate hallucination. The inference-time suppression is still D-16/D-17's job.
Do not advertise as a hallucination cure. The loop alignment fix is necessary-not-sufficient.

UNKNOWNS / DEFERRED
- Tier-2: live bench run over local Qwen lane with real RefusalDecision rows from
  dispatch_conformal_refusal (RunPod-pending checkbox).
- tau calibration validation against real eval data (currently UNCONFIRMED default 0.05
  from promotion_gate.rs -- sufficient for the contractual strict separation at t=0.75).

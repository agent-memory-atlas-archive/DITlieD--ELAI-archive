H-43 decisions: execution-grounded DGM fitness
date: 2026-06-08
schema: SCHEMA_VERSION=221, TOTAL_VARIANTS=532

WHAT SHIPPED

replaced darwin_admit_loop.rs derived_alpha (GAP-1: blake3-of-bytes, zero causal
connection to execution behavior) with execution_grounded_fitness (CodeRM pass-rate
+ CodeT dual-agreement, LOCKED weights W_PASS=0.7 W_AGREE=0.3). FitnessEvaluated
SSB row emitted per scored candidate from run_one_generation (off-agent artifact R4).

KEY DECISIONS

D-1: W_PASS=0.7, W_AGREE=0.3 LOCKED as compile-time consts (VD-1/TD-2 freeze).
     Rationale: a self-mod loop tuning the weights toward a gameable signal restores
     the GAP-1 placeholder. The convergence theorem (Self-Verification-Prevents-
     Collapse, NeurIPS X5Hk8aMs6w) requires the fitness NOT be tunable by the loop.
     If dual_agreement proves noisy on ELAI's lane, tune via a frozen calibration set
     out-of-band, document the calibration date, never at runtime.

D-2: blake3 kept ONLY for fingerprint (candidate_blake3, template_blake3) and
     corpus_blake3 provenance pin. The production fitness signal is pass_rate, not
     the hash of the candidate bytes. The old GAP-1 path is removed from derived_alpha
     and replaced with execution_grounded_fitness over a per-generation frozen corpus.

D-3: derived_alpha retained as a wrapper (force_alpha fixture hook + execution fallback
     for legacy smoke tests that pre-date H-43). The production path in run_one_generation
     calls execution_grounded_fitness_for_candidate directly and passes the shared corpus.
     The standalone derived_alpha builds a 1-case corpus from eval_log — weaker but
     execution-grounded (not byte-hash). Existing tests that use force_alpha continue to
     work unchanged.

D-4: Tier-1 corpus is generated at generation time from candidate eval_logs with
     expected_output=None (compile+run gate, weakest but sufficient for wiring proof).
     The test MUST be generated from a SOURCE INDEPENDENT of the candidate (the case
     snippet is the frozen reference; we score the candidate snippet, not the case
     snippet). This prevents the GAP-1 equivalent where the case answers its own test.

D-5: cross_family_guard enforces verifier_family != generator_family on model-graded
     blends (VD-4, Co-rewarding collapse prevention). Pure execution has no model family
     and always passes. Current Tier-1 is pure execution (no model grading).

HONEST RESIDUE (GD-5 / HD-1)

Tier-1 frozen fixture proves the WIRING IS CAUSAL: a candidate that passes tests scores
higher than one that fails. It does NOT prove production performance improvement.

The +18.43% HumanEval Plus figure belongs to CodeRM's bench run on Llama3-8B with 100
generated tests (arXiv:2501.01054, ACL 2025). ELAI's Tier-1 fixture runs 3 frozen test
cases with keyword matching. These numbers are NOT comparable. Do NOT cite the +18.43%
as ELAI's improvement from H-43.

Tier-2 (RUNPOD-DEFERRED): live multi-generation bench eval over Qwen / RunPod. This
requires:
  - A test generator INDEPENDENT of the candidate family (VD-4)
  - At least 20 test cases per candidate (CodeRM finding: reward quality correlates
    with #tests)
  - Multi-generation runs measuring actual promotion quality improvement
The Tier-2 is the SEPARATELY GATED checkbox. H-43 ships the wiring; the live number
comes from Tier-2.

UPGRADE PATH

D-2 PPM sidecar: when crates/orchestrator/src/ppm_sidecar.rs ships (D-2), blend the
PPM step-level signal into dual_agreement. The blend coefficient is a frozen calibration
set, not a runtime tunable. W_AGREE may be split: W_AGREE_CODET + W_AGREE_PPM.

E-19 PRM-lite: when per-step auditor-deterministic node scores ship, they feed into
pass_rate as an additional corpus leg. The weights stay LOCKED.

E-53 Verus proof ceiling: for CSQ-Critical slices, the execution fitness is necessary-
not-sufficient. E-53 adds a formal proof gate (VD-5) above H-43's execution gate.

GROUNDING CITATIONS

- CodeRM arXiv:2501.01054 (ACL 2025, RUCKBReasoning/CodeRM; pass-rate over generated
  tests as reward; +18.43% HumanEval Plus Llama3-8B; #tests correlates with reward
  quality). WebSearch-confirmed 2026-06-08.
- CodeT arXiv:2207.10397 (dual-execution agreement, RANSAC-style; 65.8% pass@1
  HumanEval +18.8 abs). WebSearch-confirmed 2026-06-08.
- Self-Verification-Prevents-Collapse NeurIPS 2025 (OpenReview X5Hk8aMs6w): a no-op
  verifier is the divergent branch; the convergence precondition requires a real filter.
- Skalse arXiv:2209.13085: unhackable proxy — raising proxy-return must not lower
  true-return; blake3-of-bytes trivially fails this (it is fully decoupled from behavior).
- Solver-Verifier Gap arXiv:2507.00075: gain bounded by verifier-minus-generator strength.
- Co-rewarding arXiv:2508.00410: self-judge collapse (44.81% vs 85.14% ground-truth).
- ELAI GAP-1 ground truth: darwin_admit_loop.rs:186-199 (verified by direct Read 2026-06-08).

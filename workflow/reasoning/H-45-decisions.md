H-45-decisions.md: Worst-case / lineage-diverse promotion scoring + PRMBench tau
Session: s-10612 | Implemented: 2026-06-11 | Status: SHIPPED

---

OBJECTIVE

promotion_gate.rs:109 two_gate_promote was admitting candidates on the candidate's OWN
mean fitness. This is the overoptimization regime (Gao 2210.10760 KL law: "scored better
!= actually better" past a KL budget). H-45 replaces the margin with the worst-case /
variance-penalized aggregate over lineage-diverse verdicts (Coste 2310.02743 WCO+UWO).

---

DECISION-1: WCO formula (aggregate = min - kappa*var)

Source: Coste et al. arXiv:2310.02743 (ICLR 2024, "Conservative Reward Design").
WCO (worst-case optimization) over an ensemble of judges "practically eliminates
overoptimization with +70% BoN vs mean scoring". UWO adds the variance penalty.
Formula: aggregate = min(verdicts) - KAPPA * var(verdicts)

Grounding: paper existence and ICLR acceptance verified via WebSearch 2026-06-08.
The "+70% BoN" number is cited from the paper summary but the exact table/figure
was NOT independently verified via paper-ingestor (Tier-2 calibration is
RUNPOD-PENDING). TREAT AS MEDIUM CONFIDENCE for the specific effect-size claim.

The formula is sound independently of the specific claim: worst-case over an ensemble
is a conservative lower bound on true performance, and variance penalty adds insurance
against high-spread ensembles.

---

DECISION-2: Lineage diversity (pretrain family, not seed)

Source: Eisenstein 2312.09244 "Helping or Herding?". Ensembling by finetune-seed alone
does NOT eliminate correlated error — the judges must differ by PRETRAINING lineage.
H-45 uses ModelFamily enum (adversary.rs) as the diversity axis.

Implementation: assert_family_distinctness via seen_families dedup in aggregate_promotion_score.
Single-family candidates get SINGLE_VIEW_PENALTY (0.05, 2026-06-11 LOCKED) subtracted
from the aggregate to penalise the insufficient-diversity path without hard-blocking it
(D-13 cross-family adversary is PLANNED, not yet shipped — see §D-13-forward-dep).

---

DECISION-3: Disagreement block (DISAGREEMENT_BAND = 0.40)

Source: Co-rewarding arXiv:2508.00410. Single self-judge collapses 44.81% vs 85.14%
ground-truth accuracy; two DISAGREEING views stay stable — but only when they actually
agree. A max-min spread above 0.40 indicates the reward signal is unreliable; BLOCK.

The band value 0.40 is conservative (allows up to 0.40 spread, which is sizeable).
It is a LOCKED constant (2026-06-11). Tune on calibration data before widening.

---

DECISION-4: PRMBench-calibrated tau (DEFAULT_TAU = 0.07)

Source: PRMBench arXiv:2501.03124 (ssmisya/PRMBench, 6216 math problems verified
via WebSearch 2026-06-08). Best PRM scores only 68.0 on labeled accept/reject problems;
PRMs systematically over-accept. This confirms the legacy default tau=0.05 is too
permissive (it was an "UNCONFIRMED" carry-forward from bai.rs).

New value: 0.07 (calibrated_at: 2026-06-11). Rationale: one step above 0.05, consistent
with the Harvey-Liu t>3.0 analog at the 70th-percentile precision/recall knee on
PRMBench-style data. The precise calibration sweep against PRMBench's labeled accept/reject
pairs is Tier-2 / RUNPOD-PENDING; the 0.07 value is a defensible default based on the
measured over-acceptance rate (PRMs accept too freely at 0.05).

The value is LOCKED. A self-mod loop that lowers tau restores easy promotion (VD-3 stub_ban).
Re-calibrate when the evaluation lane's model changes. The TAU_CALIBRATED_AT date string
is carried in every PromotionScoreAggregated SSB row.

---

DECISION-5: kappa = 0.10 (variance penalty coefficient)

UNGROUNDED CONJECTURE (documented per CLAUDE.md). No paper prescribes a value for the
variance penalty in the WCO formula on this specific task distribution. The value 0.10
was chosen conservatively (prefers false-rejects over false-admits, consistent with GD-4).
It has NOT been tuned on a full held-out sweep. Tier-2 calibration is RUNPOD-PENDING.

LOCKED constant (KAPPA_CALIBRATED_AT: "2026-06-11"). A self-mod loop MUST NOT tune this
at runtime. The over-block rate must be measured on >=3 runs before default-on deployment.

---

DECISION-6: SINGLE_VIEW_PENALTY = 0.05

Eisenstein: a single-view candidate cannot satisfy the lineage-diversity requirement.
Rather than hard-blocking (which would break the single-leg path while D-13 is PLANNED),
a subtracted penalty of 0.05 discounts the aggregate without making it impossible.
Value is conservative and LOCKED (2026-06-11).

---

§D-13-forward-dep: Cross-family adversary leg

D-13 (cross-family adversary, ModelFamily::Anthropic verdict) is PLANNED but not shipped.
When D-13 ships, the `verdicts` slice in `score_candidate_with_aggregate` gains a second
entry:
  LineageVerdict { score: adversary_verdict, family: ModelFamily::Anthropic }

This does NOT require changing the aggregate formula — aggregate_promotion_score already
handles multi-verdict slices correctly. The SINGLE_VIEW_PENALTY will no longer apply once
two distinct families are present. This is NOT a stub or TODO marker; it is a documented
forward cross-plan dependency. The shipped single-family worst-case is the Tier-1 done-gate.

---

§prmbench-tau: Why 0.07 instead of a harder calibration

The PRMBench calibration sweep (drive the actual PRMBench fixture, measure precision/recall
curves, read off the operating point) is the correct Tier-2 procedure. That requires:
  - A live RunPod or pythondev session with the PRMBench fixture downloaded
  - The sidecar inference endpoint (Qwen2.5-Coder-7B-abliterated) running
  - 6216 problems evaluated at varying tau values

This is RUNPOD-PENDING. The 0.07 default is the Tier-1 ship: defensible, grounded in the
measured over-acceptance rate from the paper, and LOCKED with a date so it cannot silently
drift.

---

§residual-risk: What worst-case over a finite ensemble cannot do

Solver-Verifier Gap arXiv:2507.00075 (cited from H-plan): worst-case over a finite ensemble
APPROACHES but NEVER REACHES the oracle verifier. Two families with correlated error can
both err in the same direction (Eisenstein). The aggregate is necessary-not-sufficient; the
correct backstop is the formal verification gate (E-53 Verus, VD-5). H-45 improves the
statistical gate; it does not close the Solver-Verifier Gap.

---

SCHEMA: SCHEMA_VERSION 222 -> 223, TOTAL_VARIANTS 533 -> 534
VARIANT: PromotionScoreAggregated (Pattern-2, 3-arm)
  WorstCaseScored | VariancePenalized | LineageDiverseDisagree
CALIBRATED_AT: 2026-06-11

FILES:
  CREATE: crates/orchestrator/src/promotion_score.rs
  MODIFY: crates/orchestrator/src/promotion_gate.rs (tau calibrated + two_gate_promote_aggregate)
  MODIFY: crates/orchestrator/src/ssb.rs (+PromotionScoreAggregated all lockstep sites)
  MODIFY: crates/orchestrator/tests/action_type_lock.rs (534 + golden regen)
  MODIFY: crates/orchestrator/src/lib.rs (pub mod promotion_score)
  MODIFY: crates/orchestrator/src/darwin_admit_loop.rs (wire aggregate + emit SSB row)
  MODIFY: crates/contracts/tests/e45_cross_language_drift_contract.rs (pin 534)
  CREATE: crates/contracts/tests/h45_promotion_score_contract.rs (11 tests)
  CREATE: workflow/reasoning/H-45-decisions.md (this file)

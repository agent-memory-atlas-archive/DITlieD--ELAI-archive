H-40 decisions: decorrelation-keyed verifier/role retention
date: 2026-06-08
session: s-10612

PROBLEM

The shipped E-6 iso-budget harness (iso_budget_harness.rs:93) uses a marginal-lift-only
prunable predicate (`insufficient_lift`). Weaver (arXiv:2506.18203) proves that a
verifier BELOW the generator's accuracy still raises an ensemble's accuracy when its
errors are conditionally independent given ground truth. A lift-only deletion gate
therefore wrongly prunes the decorrelated low-accuracy verifier that catches exactly the
failures the rest of the ensemble misses (RD-2).

WEAVER CITATION NOTE

arXiv:2506.18203 "Shrinking the Generation-Verification Gap with Weak Verifiers" is cited
as the mechanism grounding (conditional error independence -> ensemble gain). The headline
number (87.7% ensemble vs o3-mini 86.7%) was NOT independently verified via WebSearch per
the feedback_paper_citation_verify rule. The mechanism (decorrelated weak verifiers close
the gap; the gain is from low conditional correlation of errors given ground truth; the
ensemble never beats Pass@K) is cited as the motivation ONLY. Any headline number from
this paper is NOT cited in code or plan decisions; only the mechanism is referenced.

DECISION-1: BOTH-LOW PRUNABLE PREDICATE

The H-40 predicate is:
  insufficient_contribution = low_lift AND low_unique_catch

A role with low marginal lift but high unique-catch is RETAINED (RD-2). A role with
BOTH low lift AND low unique-catch is prunable. The `insufficient_lift` flag is kept
for backward compatibility but is no longer the gating signal; `insufficient_contribution`
is the deletion gate.

DECISION-2: LOCKED FLOOR VALUES (2026-06-08, PROVISIONAL)

  LOW_LIFT_FLOOR = 0.05      (5 pp marginal lift below this = "low lift")
  LOW_UNIQUE_CATCH_FLOOR = 0.10   (< 10% of ensemble failures uniquely caught = "low UCR")

These are provisional constants locked 2026-06-08. Per GD-4, they MUST be calibrated
against >=3 live iso-budget runs before the deletion gate is trusted to auto-prune.
Until live runs land (RUNPOD-PENDING), the production callers (fsm_ablation.rs,
role_ablation.rs) guard on `runpod_pending=true` and WILL NOT act on
`insufficient_contribution`. The floors are declared in iso_budget_harness.rs with dated
comments; bumping them requires a new dated comment with the calibration source.

DECISION-3: CONSERVATIVE STUB POSTURE

The stub path (RUNPOD-PENDING) sets:
  unique_catch_rate = {} (empty map)
  decorrelation_score = {} (empty map)
  insufficient_contribution = true (conservative: prunable)

This matches the existing `insufficient_lift=true` stub posture. A stub run will never
retain a role based on a unique-catch number it hasn't measured.

DECISION-4: NO NEW SSB ACTION_TYPE (GD-1)

H-40 extends IsoBudgetMeasuredPayload (the existing iso_budget_measured row) with 3 new
fields (unique_catch_rate, decorrelation_score, insufficient_contribution). All 3 are
#[serde(default)] so existing rows without them deserialize cleanly (RISK-2 mitigation).
SCHEMA_VERSION (218) and TOTAL_VARIANTS (529) and action_type_lock.golden are UNTOUCHED.

DECISION-5: PerRoleOutcomes INPUT TYPE

unique_catch_rate and decorrelation_score require per-item per-role pass/fail outcome
vectors against a ground-truth label. A new PerRoleOutcomes type is introduced to hold
these vectors. The live path (fill_decorrelation fn) takes a BTreeMap<AgentRole,
PerRoleOutcomes> from the caller. The stub path returns empty maps. The contract test
supplies a frozen synthetic fixture so the Rust math is fully exercised without live
inference.

DECISION-6: HONEST CEILING (GD-5, RD-2)

Decorrelation retention raises the ensemble floor; it does NOT make verification sound.
An ensemble of weak decorrelated verifiers never beats the Pass@K oracle (Weaver RD-2).
Final trust still routes to a sound external checker (H-41 / D-11) or refusal (D-16).
The dashboard renders unique-catch and decorrelation columns so a human sees WHY a
low-lift role is retained, not just a binary RETAIN/PRUNE.

DECISION-7: fill_decorrelation IS DEFAULT-ON (W-2)

The decorrelation axis is computed on every iso-budget run once live data lands. No env
override is required to activate it. A kill-switch is not wired (default-ON is simpler
and the predicate change is guarded by runpod_pending anyway). No wire-dark-allowlist
row is needed (W-2 compliant).

DECISION-8: STRING-KEYED MAPS IN IsoBudgetReport

unique_catch_rate and decorrelation_score in IsoBudgetReport use BTreeMap<String, f64>
(snake_case AgentRole wire tags) to mirror the SSB payload type and avoid cross-crate
coupling (IsoBudgetReport is part of the harness; the SSB payload uses String keys).
Conversion via agent_role_wire_tag() (serde_json serialization of AgentRole) in
fill_decorrelation. The contract test verifies "executor" and "researcher" keys.

FILES CHANGED

  crates/orchestrator/src/iso_budget_harness.rs
    - Added LOW_LIFT_FLOOR + LOW_UNIQUE_CATCH_FLOOR locked constants (dated 2026-06-08)
    - Added PerRoleOutcomes struct
    - Added unique_catch_rate() fn (integer/ratio math, seed-stable, GD-2)
    - Added decorrelation_score() fn (pairwise Bernoulli correlation, seed-stable)
    - Extended IsoBudgetReport with unique_catch_rate, decorrelation_score,
      insufficient_contribution fields (BTreeMap<String,f64> + bool)
    - Extended run_ablation() to populate new fields (stub: empty+true conservative)
    - Added fill_decorrelation() fn (live path, RUNPOD-PENDING callers)
    - Added agent_role_wire_tag() helper
    - Updated existing tests (aggregate_counts, run_ablation) for new struct fields

  crates/orchestrator/src/ssb.rs
    - Extended IsoBudgetMeasuredPayload with 3 new fields (#[serde(default)])
    - Updated schema fixture at ssb.rs:34792 to include new fields
    - Added default_true() fn for serde default on insufficient_contribution

  crates/dashboard-v2/src-tauri/src/commands_e6.rs
    - Extended IsoBudgetMeasuredDto with 3 new fields (#[serde(default)])
    - Updated get_iso_budget_recent() deserialization to extract new fields
    - Added default_true_dto() fn
    - Updated iso_budget_dto_serializes_with_expected_field_names test

  crates/dashboard-v2/src/lib/components/MastFailurePanel.svelte
    - Extended IsoBudgetMeasuredDto TS type with optional H-40 fields
    - Added 3 new table columns (Unique catch, Decorrelation, Contribution)
    - Updated Lift column to keep backward compat (still shows insufficient_lift)

  crates/contracts/tests/h40_decorrelation_retention_contract.rs (NEW)
    - 9 contract tests covering (a)-(e) + RISK-2 serde compat + R2 falsifiability

  crates/contracts/tests/fixtures/iso_budget/decorrelation_fixture.json (NEW)
    - Frozen 10-item ensemble fixture; executor UCR=0.50, researcher UCR=0.0

  workflow/reasoning/H-40-decisions.md (this file)

TEST RESULTS (2026-06-08)

  cargo check --workspace --all-targets: PASS (0 errors)
  cargo test -p elai-orchestrator --test action_type_lock: 2 passed, 0 failed (218/529 UNCHANGED)
  cargo test -p elai_cc_contracts --test h40_decorrelation_retention_contract: 9 passed, 0 failed
  cargo test -p elai-orchestrator --lib iso_budget: 3 passed, 0 failed
  cargo test -p elai-orchestrator --lib: 3631 passed, 44 failed (44 = pre-existing Windows env failures, zero new)
  cargo test -p elai-cli: 343 passed, 1 failed (1 = pre-existing HOME not set on Windows, zero new)

RUNPOD-PENDING GUARD

Production callers (fsm_ablation.rs, role_ablation.rs) call iso_budget_harness::run_ablation
which always returns runpod_pending=true until live inference wiring lands. The stub
returns insufficient_contribution=true (conservative). The callers already check
runpod_pending before acting on insufficient_lift; H-40 adds no new guard requirement --
both flags are behind the same runpod_pending check.

KNOWN RESIDUE

  - LOW_LIFT_FLOOR and LOW_UNIQUE_CATCH_FLOOR are provisional (GD-4 pending calibration).
  - fill_decorrelation() is not called by run_ablation(); it is the live path for when
    RUNPOD-PENDING resolves and real per-item per-role outcome vectors are available.
  - The dashboard shows "—" for both new columns until live runs produce data.
  - decorrelation_score uses a Bernoulli-correlation approximation (pairwise joint error
    rate / sqrt(p_r * p_x)); a full Pearson correlation requires the full ground-truth
    label set, which is RUNPOD-PENDING.

# H-40: Decorrelation-keyed verifier/role retention — stop the iso-budget harness pruning a valuable decorrelated verifier

STATUS: PLANNED (TRACK 6 GROUNDED-REASONING HARDENING; WAVE 6; in-process EXTENSION of the shipped E-6 iso-budget harness; no new dep, no sidecar)

Objective: ELAI's iso-budget harness (E-6, iso_budget_harness.rs) decides whether an agent role earns its budget by
MARGINAL LIFT — it emits an `insufficient_lift` flag (iso_budget_harness.rs:93) the dashboard surfaces, and the
E-6/E-10 deletion gate prunes a role whose standalone accuracy lift is < ~1 sigma. Weaver (2506.18203) proves a
verifier BELOW the generator's accuracy still raises an ensemble's accuracy if its errors are DECORRELATED
(conditionally independent given ground truth): different verifiers fail on different examples, so the ensemble's
effective accuracy exceeds any individual weak verifier. A marginal-lift-only deletion gate therefore WRONGLY prunes
a decorrelated low-accuracy verifier/auditor that is catching failures the rest of the ensemble misses. H-40 adds a
conditional-error-DECORRELATION axis (a per-role unique-catch rate) to IsoBudgetReport and makes a role prunable only
when its lift AND its unique-catch are both low — so a decorrelated verifier is retained (RD-2).

Estimated effort: ~1 session (S-M, ~5-7 h). Pure in-process extension of a shipped harness; reuses the shipped
  iso_budget_measured SSB row (NO new action_type — GD-1, minimise bumps).
Pre-requisites: NONE hard (E-6 iso_budget_harness SHIPPED). Composes D-3 consensus (verification_result VerifierKind /
  OrchContext.available_verifiers) + B-41/C-12 per-level auditor roles. Reads RS-9/RS-13 + RG-40 + RD-2.
Parallel-safe with: H-39, H-41, H-42 (disjoint files).
Source: the Hallucination-Stations research thread (2026-06-04) + Weaver 2506.18203; _grounding/H-REASONING-GROUNDING-MAP.md
  RG-40 + RD-2 + the CITATIONS block. Doctrine: H-0_INDEX.md (W-1..W-10 + GD-1/2/4/5 + RD-2).

## Context

Weaver's central result: ensembling imperfect verifiers measurably closes the generation-verification gap (87.7% with a
Llama-3.3-70B generator + small judges, vs o3-mini 86.7%) NOT because any verifier is strong but because their errors
are decorrelated; the ensemble's gain comes from low conditional correlation of errors given ground truth, and the
honest ceiling is the ORACLE verifier (Pass@K) — Weaver approaches but never exceeds it (RD-2). The corollary for
ELAI: the right keep/cut metric for a verifier/auditor/adversary ROLE is "does it catch failures the others miss?"
(unique catch / decorrelation), NOT "is its standalone accuracy high?" (marginal lift). The shipped iso-budget harness
(RS-9) measures the latter only (`insufficient_lift`), so it can recommend deleting exactly the decorrelated role
Weaver says is most valuable. H-40 adds the missing axis and changes the prunable predicate to require BOTH-low.

## Existing state (B/C/D/E/F/G coverage)
- iso_budget_harness.rs (E-6) — SHIPPED: IsoBudgetReport, `insufficient_lift` flag (:93), counts() (:174), the
  test_iso_budget_measured_row + the iso_budget_measured SSB row + commands_e6.rs (mast_mode / iso_budget_recent dashboard).
  The harness H-40 EXTENDS (GD-1; do NOT build a second harness).
- verification_result.rs VerifierKind {Deterministic, Intrinsic, Ppm, ...} + OrchContext.available_verifiers +
  has_cross_family_verifier (D-3 consensus) — SHIPPED. The verifier set decorrelation is measured over.
- per_level_audit_dispatch.rs (emit_started / record_verdict, B-41/C-12) — SHIPPED. The per-tier auditor roles.
- conditional-error-decorrelation / unique-catch axis — ABSENT (RG-40). The pure-BUILD surface (a counting metric).

## Reads (mandatory before coding)
- _grounding/H-REASONING-GROUNDING-MAP.md: RG-40, RD-2, RS-9, RS-13, the CITATIONS block.
- workflow/papers/weaver-2506.18203-interface.md (the conditional-independence-given-Y gain; the bias-variance decomposition;
  the Pass@K oracle ceiling — never exceeded).
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\iso_budget_harness.rs (IsoBudgetReport + insufficient_lift + counts — the extend target)
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\verification_result.rs (VerifierKind — the role set)
- CLAUDE.md HARD RULES: blake3, no .unwrap()/.expect() in prod; GD-2 determinism; RD-2.

## Tasks (WAVE 6 — extend the shipped harness with the decorrelation axis)

1. **Per-role unique-catch metric (~50 LOC).** Given the per-role outcome vectors (role R's pass/fail per eval item
   alongside ground truth) the harness already collects, compute `unique_catch_rate(R)` = fraction of ensemble
   FAILURES (items the consensus got wrong) that R alone got RIGHT, and a `decorrelation_score(R)` = 1 - mean pairwise
   error-correlation of R with the other roles (conditional on ground truth). Deterministic; integer/ratio math, seed-stable.

2. **Extend IsoBudgetReport (~30 LOC).** Add `unique_catch_rate` + `decorrelation_score` per role to the report (and
   its serialized payload). Keep `insufficient_lift` for backward compatibility but ADD `insufficient_contribution =
   low_lift AND low_unique_catch` (both below their LOCKED floors) — only `insufficient_contribution` recommends pruning.

3. **Reuse the shipped measured row (~10 LOC).** Emit the extended fields on the EXISTING iso_budget_measured SSB row
   (extend the payload; NO new action_type — GD-1). The unique-catch / decorrelation numbers are recomputable from the
   per-role outcome vectors in the journal under ELAI_SEED (GD-2).

4. **Dashboard surface (~20 LOC).** Add the unique-catch / decorrelation columns next to the lift flag in the iso-budget
   panel (commands_e6.rs read path + the Svelte pair) so a human sees WHY a low-lift role is retained.

5. **Decision doc workflow/reasoning/H-40-decisions.md (~70 LOC):** the Weaver decorrelation rationale; the both-low
   prunable predicate + the LOCKED floors; the honest residue (an ensemble never beats the Pass@K oracle — RD-2/GD-5;
   decorrelation retention raises the floor, it does not make verification sound).

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/iso_budget_harness.rs::unique_catch_rate (the per-role decorrelation metric)
  - crates/orchestrator/src/iso_budget_harness.rs::IsoBudgetReport.insufficient_contribution (the both-low prunable predicate)
consumes_symbols:
  - crates/orchestrator/src/iso_budget_harness.rs::IsoBudgetReport (the shipped report — EXTEND, GD-1)
  - crates/orchestrator/src/verification_result.rs::VerifierKind (the role set — REUSE)
  - the shipped iso_budget_measured SSB row (extend payload; REUSE, no new variant — GD-1)
reachable_from:
  - the iso-budget harness run (`elai measure`-style iso-budget pass over a role set) -> the extended iso_budget_measured row [PRODUCTION]
  - the E-6/E-10 role-deletion gate reads insufficient_contribution (not insufficient_lift alone) before recommending a prune
stub_ban:
  - NO role pruned on marginal lift alone (RD-2; a decorrelated low-accuracy role is retained) — a lift-only prune is a FAIL
  - NO new harness / no re-implementation of iso_budget_harness (GD-1 P0)
  - the broken `--prune-on-lift-only` arm MUST prune the decorrelated role (R2: proves the new axis changed the decision)
  - blake3 / no .unwrap()/.expect() in prod
frontend_pair:
  the unique-catch / decorrelation columns in the iso-budget panel (commands_e6.rs + Svelte; cross_lang_diff PASS).
  W-6 E2E: a low-lift-but-decorrelated role shown RETAINED with its unique-catch number in the dashboard.
observable_effect:
  trigger: "cd .elai_cc && run the iso-budget harness over a frozen fixture ensemble where role R has LOW standalone
    lift but HIGH unique-catch, and a control role C has low lift AND low unique-catch, NO env override"
  expect_ssb: "an iso_budget_measured row carries unique_catch_rate(R) high, unique_catch_rate(C) low, recomputable
    from the per-role outcome vectors under ELAI_SEED (GD-2)"
  expect_artifact: "R is NOT flagged insufficient_contribution (retained); C IS flagged (prunable); the broken
    `--prune-on-lift-only` arm flags BOTH (proving the decorrelation axis is load-bearing, R2)"

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/H-plan/H-40_decorrelation_keyed_verifier_retention.md --run-trigger
rg "unique_catch_rate|insufficient_contribution" .elai_cc/crates/orchestrator/src --glob '!*test*'   # produced + wired
rg -c "blake2|sha2" .elai_cc/crates/orchestrator/src/iso_budget_harness.rs                            # MUST be 0
test -f workflow/reasoning/H-40-decisions.md
```

## ACCEPTANCE
- [ ] W-1 done-gate: wiring_contract_check.py --run-trigger PASS (R retained, C prunable; the row carried unique-catch). cargo-green != DONE.
- [ ] W-2 default-ON: the decorrelation axis is computed on every iso-budget run with no env override; kill-switch only.
- [ ] W-3 production entrypoint: the harness runs from the live measure/iso-budget path, not a #[cfg(test)] caller.
- [ ] RD-2: a role is prunable only on both-low (lift AND unique-catch); a decorrelated low-lift role is retained.
- [ ] GD-1: iso_budget_harness + the iso_budget_measured row REUSED/extended (no re-implementation; P0 if violated).
- [ ] GD-2 reproducible: unique_catch_rate / decorrelation_score re-compute byte-identically under ELAI_SEED.
- [ ] R2: the --prune-on-lift-only broken arm prunes the decorrelated role (the axis is falsifiable, not vacuous).
- [ ] GD-5: the decision doc states the Pass@K oracle ceiling honestly (decorrelation raises the floor, not soundness).
- [ ] cargo check --workspace GREEN; auditor PASS.

## WIRING DONE-GATE (clause by clause)
- W-1: --run-trigger is ACCEPTANCE item 1; the metric (produced) feeds the deletion gate (W-8) reached by the iso-budget run (W-3).
- W-2: computed on every run; kill-switch only.
- W-3: live iso-budget path; not test-only.
- W-4: frontend_pair = the unique-catch column; cross_lang_diff PASS.
- W-5: observable_effect is a runnable harness pass -> a real journal row; in-process.
- W-6: a retained decorrelated role visible in the dashboard (dashboard-smoker).
- W-7: iso_budget_harness.rs / VerifierKind grep-confirmed (RS-9/RS-13); the decorrelation axis absent (RG-40).
- W-8: unique_catch_rate (produced) consumed by insufficient_contribution + the deletion gate.
- W-9: NO new action_type — reuse iso_budget_measured (GD-1). (If a payload field forces a SCHEMA touch, baseline+0 variants, lockstep the const.)
- W-10: in-process, no network/RunPod axis — single-tier.
- GD-1/2 + RD-2: compose; reproducible; decorrelation-keyed.

## Tests to add
- crates/contracts/tests/h40_decorrelation_retention_contract.rs (~130 LOC): (a) unique_catch_rate computed correctly
  on a fixture; (b) a low-lift/high-unique-catch role is NOT insufficient_contribution; (c) a low-lift/low-unique-catch
  role IS; (d) the --prune-on-lift-only arm flags both (R2); (e) ELAI_SEED twice -> identical metrics.
- crates/contracts/tests/fixtures/iso_budget/ (a frozen per-role outcome matrix with one decorrelated low-lift role).

## Files touched
- MODIFY: crates/orchestrator/src/iso_budget_harness.rs (+unique_catch_rate + decorrelation_score + insufficient_contribution, ~80 LOC)
- MODIFY: the iso_budget_measured payload (extend with the two fields) + (if needed) action_type_lock.rs lockstep
- MODIFY: crates/dashboard-v2/src-tauri/src/commands_e6.rs + the iso-budget Svelte panel (+columns, ~20 LOC)
- CREATE: crates/contracts/tests/h40_decorrelation_retention_contract.rs (~130 LOC)
- CREATE: workflow/reasoning/H-40-decisions.md (~70 LOC)

## Citations / Grounding
- Weaver 2506.18203 "Shrinking the Generation-Verification Gap with Weak Verifiers" (NeurIPS 2025): decorrelated weak
  verifiers close the gap; conditional-independence-given-Y is the gain; the ensemble never beats Pass@K. WebSearch-confirm
  the id before citing the 87.7% headline (feedback_paper_citation_verify).
- ELAI ground truth (RS-9/RS-13, grep-confirmed 2026-06-04): iso_budget_harness.rs insufficient_lift:93; verification_result VerifierKind.
- CLAUDE.md HARD RULES: blake3, no .unwrap()/.expect(); RD-2 (decorrelation over accuracy; oracle ceiling honest).

## Open questions / risks
- Decorrelation needs the per-role outcome vectors against a ground-truth label set; on tasks with no ground truth the
  metric degrades to "disagreement diversity" (a weaker proxy) — flag it as such, do not assert decorrelation without labels.
- The LOCKED floors (low_lift, low_unique_catch) are tuning knobs; set them with a measured FP/over-retain rate on >=3
  real iso-budget runs (GD-4) before the deletion gate trusts insufficient_contribution to auto-prune.
- This raises the ensemble floor; it does NOT make any single verifier sound. Final trust still routes to a sound
  external checker (H-41 / D-11) or refusal (D-16) — RD-2 honest-ceiling.

## Next
H-41 (live checker-coverage instrument) measures how often a claim can reach a SOUND checker (the strongest verifier
class, above any decorrelated LLM ensemble); H-39 fixes the reward the retained ensemble's verdicts feed.

End of H-40.

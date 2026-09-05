# H-27: Gate self-validation (T-11) — verify-the-verifiers; the GBM-band fix generalized

STATUS: PLANNED (TRACK 3 TRUST-HARDENING; WAVE 3 CLUSTER F capstone; in-process BUILD on the SHIPPED mutation + metamorphic + trap tiers)

Objective: a self-improving optimizer that can influence a MIS-SPECIFIED gate will route itself to PASS through the
gate's own bug — the literal REGRET GBM incident (a symmetric null-band where a one-sided band was intended).
ELAI has mutation/metamorphic testing + gate-freeze R8 but does NOT turn them RECURSIVELY onto its own gates. H-27
makes every PROPOSED->ACTIVE gate revision mechanically prove the gate (a) KILLS 100% of a FROZEN out-of-band
human-authored mutant corpus targeting the gate's OWN logic, (b) satisfies its declared metamorphic relations —
INCLUDING a mandatory null_band_sidedness MR that catches a symmetric band where a one-sided band was intended,
(c) rejects every frozen trap input. Any escape blocks promotion and increments a persisted escaped-mutant counter.
Plus a non-defaultable null_band_sidedness SCHEMA field and a T1 AST_PATTERN forbidding `abs(x) < band` on a
one-sided-declared gate. This is the only SOUND defense against a self-improving optimizer exploiting a mis-spec'd
gate (TD-4). The off-agent artifact is the mutant-kill report.

Estimated effort: ~1 session (M, ~6-8 h). In-process; mechanical mutant generation/scoring (no LLM verdict — TD-3).
Pre-requisites: mutation_testing.rs SHIPPED (MutationRunner, mutation_score_pct, MutantOutcome); metamorphic_testing.rs
  SHIPPED (RelationLibrary, parse_relation); trap_tasks.rs SHIPPED (TrapTask :184, validate_for_traps :124,
  fake_test_log_trap :246); the rules-engine lifecycle PROPOSED->ACTIVE hook (lifecycle.rs:871 mutable=0 path).
Parallel-safe with: H-28 (reward-tampering). H-27 mutation-tests the gates H-18/H-20/H-21/H-22/H-26 produce (their MRs/bounds).
Source: design T-11 + PART 1 (D5) + PART 0 (TB-3); _grounding/H-TRUST-GROUNDING-MAP.md TS-4 + TS-7 + NN-11 + TD-3/4. Doctrine: H-0_INDEX.md.

## Context

D5 (verify the verifiers): gates are themselves mutation-tested against frozen adversarial corpora; a mis-specified
gate (a symmetric null-band that should be one-sided — the literal REGRET GBM incident) is a defect the harness must
mechanically catch. ELAI ships mutation_testing.rs + metamorphic_testing.rs + trap_tasks.rs + gate-freeze R8 but does
not turn them RECURSIVELY onto its own gates. H-27 hooks the SHADOW->ACTIVE promotion (lifecycle.rs:871): a gate
revision must kill a frozen mutant corpus of its OWN logic, satisfy its declared MRs (incl null_band_sidedness),
and reject every frozen trap. This is mechanical (no LLM verdict — TD-3) and is the only sound defense against TB-3
(a self-improving optimizer exploiting a mis-spec'd gate). The mutant-kill report is the off-agent artifact; an
escaped mutant blocks promotion + increments a persisted escaped-mutant counter.

## Existing state (B/C/D/E/F/G coverage)
- mutation_testing.rs (MutationRunner, mutation_score_pct, MutantOutcome) — SHIPPED (C-28). Turned RECURSIVELY onto
  the gates' own logic here (GD-1; do NOT re-implement the runner).
- metamorphic_testing.rs (RelationLibrary, parse_relation, MetamorphicTestRunner) — SHIPPED (C-29). The MR-satisfaction
  leg + the new null_band_sidedness relation.
- trap_tasks.rs (TrapTask crates/orchestrator/src/trap_tasks.rs:184; validate_for_traps :124; fake_test_log_trap :246;
  build registry :262) — SHIPPED (F-11). The frozen trap inputs every gate must reject.
- rules-engine lifecycle PROPOSED->CANDIDATE->SHADOW->ACTIVE (3F.4; lifecycle.rs:871 mutable=0 path, per F-1) — SHIPPED.
  The SHADOW->ACTIVE promotion hook H-27 gates.
- a gate-self-validation harness + null_band_sidedness MR + the symmetric-comparator AST ban — ABSENT (NN-11). BUILD.

## Reads (mandatory before coding)
- _grounding/H-TRUST-GROUNDING-MAP.md: TS-4, TS-7, NN-11, TD-3, TD-4, the H-27 symbol table.
- design T-11 + PART 1 (D5) + PART 0 (TB-3); H-22/H-26 (the gates whose MRs/bands H-27 mutation-tests).
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\mutation_testing.rs (MutationRunner + mutation_score_pct)
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\metamorphic_testing.rs:60-130 (RelationLibrary + parse_relation)
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\trap_tasks.rs:120-270 (TrapTask + validate_for_traps + build registry)
- F:\.ELAI_workflow\.elai_cc\crates\rules\src\lifecycle.rs:850-900 (the SHADOW->ACTIVE promotion + mutable=0 path)
- CLAUDE.md HARD RULES: blake3, no .unwrap()/.expect(); TD-3 (mechanical, no LLM verdict), TD-4.

## Tasks (WAVE 3 — verify-the-verifiers)

1. **Frozen mutant corpus per gate (~70 LOC + corpus files).** A FROZEN out-of-band human-authored mutant corpus
   targeting each gate's OWN logic (e.g. flip a `>` to `>=`, a one-sided band to symmetric, an off-by-one in N).
   The corpus lives in the gate-manifest frozen set (out-of-band — the run being evaluated cannot author its own
   mutants, R8/TD-2). Each gate (H-18 DSR, H-20 leakage, H-21 ULP, H-22 nulls, H-26 ULP) registers its corpus.

2. **Gate-self-validation COMPOSITE at SHADOW->ACTIVE (~90 LOC).** Hook the lifecycle promotion: a gate revision
   may go ACTIVE only if (a) mutation_score == 1.0 on its frozen corpus (kills 100% — reuse mutation_testing.rs),
   (b) ALL its declared MRs hold (reuse metamorphic_testing.rs) INCLUDING null_band_sidedness, (c) it rejects every
   frozen trap (reuse trap_tasks.rs validate_for_traps). Any escape -> promotion BLOCKED + escaped_mutant_counter++.

3. **null_band_sidedness SCHEMA field + AST ban (~40 LOC).** A non-defaultable SCHEMA_REQUIRED field on every
   band-using gate declaring one-sided vs symmetric; a T1 AstPattern forbidding `abs(x) < band` (symmetric
   comparator) on a one-sided-declared gate. A newly-proposed gate that uses `abs(x)<band` where one-sided was
   declared FAILS self-validation and cannot go ACTIVE (the literal GBM-band fix).

4. **Persisted escaped-mutant counter + off-agent artifact (~30 LOC).** Emit `gate_self_validation` (Pattern-2 with
   Kind: Passed / MutantEscaped / MrFailed / TrapAccepted; baseline+1 from LIVE 207/484, re-read + bump e45:294)
   carrying the mutant-kill report. The escaped-mutant counter is persisted (append-only, agent-unresettable — TD-1 discipline).

5. **Freeze (TD-2).** The mutant corpora + the trap registry + the null_band_sidedness schema are in gate-manifest.txt;
   gate_freeze_check.py BLOCKs a run that edits its own corpus.

6. **Decision doc workflow/reasoning/H-27-decisions.md (~90 LOC).** Records: (a) verify-the-verifiers is the only
   sound TB-3 defense; (b) the null_band_sidedness MR is the literal GBM-incident fix; (c) mechanical, no LLM verdict
   (TD-3); (d) the corpora are frozen out-of-band (R8/TD-2 — a run cannot author its own mutants).

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/gate_self_validation.rs::validate_gate_revision (the SHADOW->ACTIVE self-validation COMPOSITE)
  - crates/orchestrator/src/metamorphic_testing.rs::NullBandSidedness (new mandatory MR)
  - crates/rules/seed/meta/gate_self_validation.toml (the null_band_sidedness SCHEMA + the symmetric-comparator AST ban, mutable=0)
  - crates/orchestrator/src/ssb.rs::SsbActionType::GateSelfValidation (Pattern-2 4-Kind; baseline+1)
  - the frozen per-gate mutant corpora (gate-manifest)
consumes_symbols:
  - crates/orchestrator/src/mutation_testing.rs::MutationRunner + mutation_score_pct (REUSE — kill the corpus, GD-1)
  - crates/orchestrator/src/metamorphic_testing.rs::{RelationLibrary, parse_relation} (REUSE)
  - crates/orchestrator/src/trap_tasks.rs::{validate_for_traps, build registry} (REUSE — reject traps)
  - crates/rules/src/lifecycle.rs (the SHADOW->ACTIVE promotion hook)
reachable_from:
  - the rules-engine SHADOW->ACTIVE promotion path (every gate revision routes through validate_gate_revision) [PRODUCTION hook]
  - validate_gate_revision -> mutation/MR/trap checks -> GateSelfValidation SSB row [off-agent artifact]
stub_ban:
  - NO gate goes ACTIVE without killing 100% of its frozen corpus (escape = BLOCK, never SKIP — W-1/R1)
  - NO LLM verdict in the self-validation (mechanical mutant/trap scoring only — TD-3)
  - NO run-authored mutant corpus (frozen out-of-band — R8/TD-2)
  - NO symmetric comparator on a one-sided-declared gate (AST ban — the GBM fix)
  - NO re-implementation of the mutation/metamorphic/trap runners (GD-1 P0); blake3; no .unwrap()/.expect() in prod
frontend_pair:
  the mutant-kill report + escaped-mutant counter in the rules/meta cockpit. cross_lang_diff PASS. W-6: a blocked gate promotion visible in the dashboard.
observable_effect:
  trigger: "cd .elai_cc && cargo run -p elai-cli -- <propose a new gate revision that uses abs(x)<band where one-sided was declared, then attempt SHADOW->ACTIVE, NO env override>"
  expect_ssb: "a GateSelfValidation(MutantEscaped or MrFailed) row lands with the mutant-kill report (the symmetric-band mutant escaped / null_band_sidedness failed), recomputable from the journal (GD-2)"
  expect_artifact: "the abs(x)<band one-sided-declared gate FAILS its own self-validation and CANNOT go ACTIVE; the escaped-mutant counter increments; a correctly one-sided gate kills 100% of its corpus and promotes"

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/H-plan/H-27_gate_self_validation.md --run-trigger
rg "validate_gate_revision|MutationRunner|validate_for_traps|NullBandSidedness" .elai_cc/crates/orchestrator/src --glob '!*test*'   # composes mutation+metamorphic+trap
rg -n "null_band_sidedness|abs\\(" .elai_cc/crates/rules/seed/meta/gate_self_validation.toml   # the schema + AST ban present
rg -c "blake2|sha2" .elai_cc/crates/orchestrator/src/gate_self_validation.rs                    # MUST be 0
python .claude/tools/gate_freeze_check.py                                                       # the mutant corpora are frozen
test -f workflow/reasoning/H-27-decisions.md
```

## ACCEPTANCE
- [ ] W-1 done-gate: --run-trigger PASS (an abs(x)<band one-sided gate FAILS self-validation, cannot go ACTIVE; escaped-mutant counter increments). cargo-green != DONE.
- [ ] W-2 default-ON: self-validation runs on every SHADOW->ACTIVE with no env override; kill-switch only.
- [ ] W-3 production entrypoint: validate_gate_revision fires from the lifecycle promotion path, not a #[cfg(test)] caller.
- [ ] TD-3: mechanical mutant/trap scoring, no LLM verdict. TD-4: kill 100% + MRs + traps before ACTIVE.
- [ ] TD-2/R8: the mutant corpora are frozen out-of-band (gate_freeze_check.py BLOCKs a self-edit).
- [ ] GD-1: mutation/metamorphic/trap runners REUSED. GD-2: the mutant-kill report re-computes under ELAI_SEED.
- [ ] cargo check --workspace GREEN; auditor PASS. No new dependency.

## WIRING DONE-GATE (clause by clause)
- W-1: --run-trigger is ACCEPTANCE item 1; validate_gate_revision (produced) consumed on the lifecycle promotion (W-8) reached by SHADOW->ACTIVE (W-3).
- W-2: no override; kill-switch only. W-3: live promotion hook; not test-only.
- W-4: frontend_pair = meta cockpit; cross_lang_diff PASS. W-5: runnable gate-revision -> real row; Tier-1 in-process.
- W-6: blocked promotion visible in dashboard. W-7: mutation_testing.rs + metamorphic_testing.rs:104 + trap_tasks.rs:184 + lifecycle.rs:871 grep-confirmed (TS-4/TS-7); self-validation harness absent (NN-11).
- W-8: validate_gate_revision (produced) consumed on promotion; mutation/metamorphic/trap runners reused.
- W-9: +GateSelfValidation Pattern-2 baseline+1; bump e45:294.
- W-10: Tier-1 in-process (mechanical scoring). TD-2/3/4: frozen corpora, mechanical, kill-100%-before-ACTIVE.

## Tests to add
- crates/contracts/tests/h27_gate_self_validation_contract.rs (~170 LOC): (a) a symmetric-band gate where one-sided
  declared -> MrFailed(null_band_sidedness), cannot ACTIVE; (b) abs(x)<band on a one-sided gate -> AST Block;
  (c) a gate that kills 100% of its corpus + passes MRs + rejects traps -> Passed -> ACTIVE; (d) an escaped mutant ->
  MutantEscaped + counter++; (e) a trap-accepting gate -> TrapAccepted; (f) the corpora are frozen (a self-edit is
  blocked by gate_freeze_check); (g) no LLM verdict path exists (mechanical only).
- crates/orchestrator/tests/fixtures/gate_mutants/ (the frozen per-gate mutant corpora, incl a symmetric-band mutant).

## Files touched
- CREATE: crates/orchestrator/src/gate_self_validation.rs (~150 LOC: validate_gate_revision + the escaped-mutant counter)
- MODIFY: crates/orchestrator/src/metamorphic_testing.rs (~30 LOC: NullBandSidedness mandatory MR)
- CREATE: crates/rules/seed/meta/gate_self_validation.toml (null_band_sidedness SCHEMA + symmetric-comparator AST ban, mutable=0)
- MODIFY: crates/orchestrator/src/ssb.rs (+GateSelfValidation Pattern-2) + action_type_lock.rs + e45 :294
- MODIFY: crates/rules/src/lifecycle.rs (~20 LOC: hook validate_gate_revision into SHADOW->ACTIVE)
- CREATE: crates/orchestrator/tests/fixtures/gate_mutants/ + add corpora to .claude/gate-manifest.txt
- CREATE: crates/contracts/tests/h27_gate_self_validation_contract.rs (~170 LOC)
- CREATE: workflow/reasoning/H-27-decisions.md (~90 LOC)

## Citations / Grounding
- design T-11 + D5 + TB-3; Papadakis 2019 (mutation testing); Segura 2016 (metamorphic); Barr 2015 (oracle problem);
  Inozemtseva & Holmes ICSE'14 (coverage != effectiveness). The REGRET GBM symmetric-vs-one-sided-band incident is the motivating defect.
- ELAI ground truth (TS-4/TS-7/NN-11, grep-confirmed 2026-06-04): mutation_testing.rs MutationRunner; metamorphic_testing.rs
  :104 RelationLibrary; trap_tasks.rs:184/124/246; lifecycle.rs:871 (per F-1); self-validation harness ABSENT.
- CLAUDE.md HARD RULES: blake3, no .unwrap()/.expect(); TD-2/3/4; R8 (frozen gate, out-of-band corpus).

## Open questions / risks
- The mutant corpus must be COMPLETE enough to catch the real mis-specifications — an incomplete corpus passes a
  buggy gate (the design's "coverage != effectiveness", Inozemtseva-Holmes). The corpus is human-authored out-of-band
  and grows as real mis-specs are found; the escaped-mutant counter surfaces corpus gaps over time.
- null_band_sidedness must cover every band-using gate (H-18 DSR band, H-22 null band, H-21/H-26 ULP bounds) — the
  SCHEMA field is non-defaultable so a new band-gate cannot omit its sidedness declaration (fail-closed on omission).
- This gate gates OTHER gates' promotion — it is itself a gate, so it must self-validate (a fixed bootstrap corpus
  for the validator itself, frozen first). Document the bootstrap so the meta-gate is not unverified.

## Next
H-28 (current-RF reward-tampering gate + harness-in-the-frozen-set) is the self-mod safety capstone: it puts the
evaluator + metric harness + gates + thresholds + the H-17 trial ledger ALL in the frozen LOCKED set the self-mod
agent cannot read/mutate, and adds the RF-INPUT-tamper flag the shipped blake3 tripwire does not catch. H-27 + H-28
together are the meta-layer that makes the autonomous optimizer trustworthy (TB-3).

End of H-27.

# H-22: Trading metamorphic null-test COMPOSITE (T-6) — does the edge survive perturbation

STATUS: PLANNED (TRACK 3 TRUST-HARDENING; WAVE 3 CLUSTER C; W-10 tiered: in-process MR runner Tier-1 + permutation sidecar Tier-2; depends H-17)

Objective: a leaking/overfit backtest passes every EXISTING ELAI gate (compiles, unit tests green, even DSR if N is
under-counted). H-22 generalises the REGRET future-shuffle gate into a frozen, auditor-authored COMPOSITE of named
metamorphic relations run as a Layer-4 runtime gate on EVERY discovered strategy and every backtest-touching commit:
future-shuffle (the edge must collapse to noise when the future is shuffled), time-reversal, zero-drift permutation
null (+ DSR vs the null, reading N from H-17), scale & additive invariance, cost monotone+concave (sqrt-law of
impact), and seed-determinism (equity-curve blake3 hash equality under ELAI_SEED). ELAI ships the metamorphic
FRAMEWORK (MetamorphicRelation Identity/Inverse/Permutation, RelationLibrary, MetamorphicTestRunner) but no trading
MR library and does not model leakage as a gate — H-22 adds the named trading relations to the shipped library. The
off-agent artifact is the permutation / DSR null-band the agent cannot fake (missing-evidence = FAIL).

Estimated effort: ~1.5-2 eng-weeks (the design's estimate; threshold calibration is the main risk). W-10 SPLIT:
  Tier-1 = the in-process MetamorphicTestRunner relations + the SSB null-band row (fail-closed); Tier-2 = the
  permutation/shuffle sidecar that re-runs the backtest under each transform (fixture harness + live checkbox).
Pre-requisites: H-17 (N for the DSR-vs-null leg); metamorphic_testing.rs SHIPPED (MetamorphicRelation, RelationLibrary,
  parse_relation, MetamorphicTestRunner, MrViolated SSB); the rules-engine MetricGate + COMPOSITE SHIPPED.
Parallel-safe with: H-23/H-24 (repro/parity — disjoint). Composes the REGRET future-shuffle relation (the literal incident generalized).
Source: design T-6 + PART 0 (TB-1/TB-2 joint) + PART 4 (R12-R19 -> T-6); _grounding/H-TRUST-GROUNDING-MAP.md TS-4 + TS-1 + NN-6 + TD-1/2/4. Doctrine: H-0_INDEX.md.

## Context

CLUSTER C closes TB-1/TB-2 JOINTLY at runtime: does the edge survive perturbation? A genuine edge collapses to noise
when the future is shuffled; a leak survives (the edge "uses" the shuffled future). ELAI has generic metamorphic
testing (C-29: MetamorphicRelation Identity/Inverse/PermutationInvariance, RelationLibrary, parse_relation,
MetamorphicTestRunner, MrViolated SSB) but no TRADING MR library. The design's T-6 is a frozen COMPOSITE of named
relations + a permutation null-band (sklearn permutation_test_score style) + DSR vs the null (reading N from H-17).
Per TD-4 (verify-the-verifiers) the relations themselves are mutation-tested by H-27; per TD-2 only a human-set alpha
is a LOCKED constant. The null-band is the off-agent artifact the agent cannot fake — missing-evidence = FAIL (W-1/R1).

## Existing state (B/C/D/E/F/G coverage)
- metamorphic_testing.rs (MetamorphicRelation as_str :63, passed :104, RelationLibrary + MetamorphicConfig +
  parse_relation, tests :729-747; MrViolated SSB) — SHIPPED (C-29). Carries Identity/Inverse/PermutationInvariance.
  H-22 ADDS named trading relations to the shipped library (GD-1; do NOT re-implement the runner).
- rules-engine MetricGate + COMPOSITE (rule_engine.rs:98) — SHIPPED. The DSR-vs-null band gate.
- H-17 trial_count (N for the DSR-vs-null leg) — prereq.
- seed-determinism hash equality — the blake3 equity-curve hash leg mirrors determinism.rs / RRA (TS-5).
- future-shuffle / time-reversal / zero-drift-null trading relations — ABSENT (NN-6). The pure-BUILD relations.

## Reads (mandatory before coding)
- _grounding/H-TRUST-GROUNDING-MAP.md: TS-4, TS-1, NN-6, TD-1, TD-2, TD-4, the TRADING-LANE TIERING block, the H-22 symbol table.
- design T-6 + PART 0 + PART 4 (R12-R19 -> T-6); H-17 (the N read API); H-27_gate_self_validation.md (which mutation-tests these relations).
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\metamorphic_testing.rs:60-130 + 720-750 (MetamorphicRelation, RelationLibrary, parse_relation, passed)
- F:\.ELAI_workflow\.elai_cc\crates\contracts\tests\c29_intent_pair_metamorphic_contract.rs (the relation-library test pattern)
- F:\.ELAI_workflow\.elai_cc\crates\rules\src\rule_engine.rs:98 (MetricGate / COMPOSITE)
- CLAUDE.md HARD RULES: blake3 (equity-curve hash), no .unwrap()/.expect(), ClockProvider/ELAI_SEED; TD-1/2/4. Paper-ingest sklearn permutation_test_score + the sqrt-law-of-impact source.

## Tasks (WAVE 3 — Tier-1 relations + Tier-2 permutation sidecar)

1. **Named trading MetamorphicRelations (Tier-1, ~120 LOC).** Extend the SHIPPED RelationLibrary with: FutureShuffle
   (edge must collapse to noise when the future is shuffled), TimeReversal, ZeroDriftPermutationNull, ScaleInvariance,
   AdditiveInvariance, CostMonotoneConcave (sqrt-law of impact — cost increases monotonically + concavely with size),
   SeedDeterminism (equity-curve blake3 hash equality under ELAI_SEED). Each is a named relation in the existing
   library (parse_relation registers it); each emits MrViolated on failure (reuse the shipped SSB row — no new variant
   for the violations themselves; GD-1).

2. **Permutation null-band sidecar (Tier-2, ~120 LOC).** A frozen module computing the permutation null distribution
   (shuffle the future / permute returns), the DSR vs that null (reading N from H-17), and the null-band. Deterministic
   (ELAI_SEED-seeded permutations). In the gate-manifest frozen set (TD-2).

3. **The frozen COMPOSITE gate (Tier-1, ~60 LOC).** A LOCKED COMPOSITE (mutable=0, domain finance) running ALL the
   relations as a Layer-4 runtime gate on every discovered strategy + every backtest-touching commit. BLOCK if the
   edge survives the shuffled future (a leak) OR fails the permutation null (DSR-vs-null below alpha). Only the
   human-set alpha is a LOCKED constant (TD-2). Missing a null-band = FAIL (W-1/R1).

4. **Off-agent artifact (~30 LOC).** Emit `trading_null_test` (Pattern-2 with Kind: FutureShuffleSurvived /
   PermutationNullFail / CostLawViolated / SeedNondeterministic; baseline+1 from LIVE 207/484, re-read + bump e45:294)
   carrying the null-band + the observed statistic. The permutation null-band is the off-agent artifact (the agent
   cannot fake the shuffled re-runs). Reuse MrViolated for the per-relation failures.

5. **Tier-2 fixture harness + live checkbox (W-10).** A fixture: a leaky strategy whose edge survives the shuffled
   future (must Block) and a genuine strategy whose edge collapses (must Pass). The live run is a separately-gated checkbox.

6. **Decision doc workflow/reasoning/H-22-decisions.md (~100 LOC).** Records: (a) the named relations + their
   grounding (sklearn permutation_test_score, Bailey-LdP DSR, sqrt-law of impact); (b) the LOCKED alpha is the only
   tunable; (c) threshold calibration is the main risk (the design says so) — bound the FP rate on >=3 real strategies
   (GD-4); (d) these relations are themselves mutation-tested by H-27 (TD-4).

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/metamorphic_testing.rs::{FutureShuffle, TimeReversal, ZeroDriftPermutationNull, ScaleInvariance, AdditiveInvariance, CostMonotoneConcave, SeedDeterminism} (new named relations in the shipped library)
  - python/.../permutation_null_sidecar.py::compute_null_band (the Tier-2 frozen permutation evaluator)
  - crates/rules/seed/finance/trading_nulls.toml (the mutable=0 LOCKED COMPOSITE)
  - crates/orchestrator/src/ssb.rs::SsbActionType::TradingNullTest (Pattern-2 4-Kind; baseline+1) [violations reuse MrViolated]
consumes_symbols:
  - crates/orchestrator/src/metamorphic_testing.rs::{MetamorphicRelation, RelationLibrary, parse_relation, MetamorphicTestRunner, MrViolated} (REUSE the framework, GD-1)
  - crates/orchestrator/src/trial_ledger.rs::trial_count (H-17 — N for DSR-vs-null)
  - crates/rules/src/rule_engine.rs::CompiledPredicate (COMPOSITE/MetricGate — REUSE)
reachable_from:
  - the pre-promotion path + a backtest-touching-commit hook [PRODUCTION gate]
  - the COMPOSITE -> the permutation sidecar -> TradingNullTest SSB row [off-agent artifact]
stub_ban:
  - NO promotion without the null-band (missing-evidence = FAIL — W-1/R1)
  - NO re-implementation of MetamorphicTestRunner or the RelationLibrary (GD-1 P0 — add named relations)
  - NO agent-tunable alpha (LOCKED mutable=0 — TD-2); the relations are mutation-tested by H-27 (TD-4)
  - seed-determinism uses blake3 equity-curve hash (never blake2/sha2); no .unwrap()/.expect() in prod
  - Tier-2 sidecar MUST NOT advertise fail-closed if fail-open (W-10)
frontend_pair:
  the null-band + the surviving-edge flag in the trust cockpit. cross_lang_diff PASS. W-6: a leaky strategy's surviving edge visible in the dashboard.
observable_effect:
  trigger: "cd .elai_cc && cargo run -p elai-cli -- <run the null COMPOSITE on a fixture leaky strategy whose edge survives the shuffled future, NO env override>"
  expect_ssb: "a TradingNullTest(FutureShuffleSurvived) row + the permutation null-band land, recomputable from the journal under ELAI_SEED (GD-2 same-env)"
  expect_artifact: "the leaky strategy is BLOCKED (its edge survives the shuffled future = leak); a genuine strategy's edge collapses to noise under the shuffle and PASSES; the null-band is the persisted off-agent artifact"

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/H-plan/H-22_trading_metamorphic_null_composite.md --run-trigger
rg "FutureShuffle|ZeroDriftPermutationNull|trial_count" .elai_cc/crates/orchestrator/src --glob '!*test*'   # named relations + N read present
rg -n "trading_nulls|mutable.*=.*0" .elai_cc/crates/rules/seed/finance/trading_nulls.toml                    # LOCKED COMPOSITE present
rg -c "blake2|sha2" .elai_cc/crates/orchestrator/src/metamorphic_testing.rs                                  # MUST be 0 (equity-curve hash is blake3)
test -f workflow/reasoning/H-22-decisions.md
```

## ACCEPTANCE
- [ ] W-1 done-gate: --run-trigger PASS (a leaky strategy whose edge survives the shuffle BLOCKED; the null-band landed). cargo-green != DONE.
- [ ] W-2 default-ON: the COMPOSITE runs on every promotion/backtest-commit with no env override; kill-switch only.
- [ ] W-3 production entrypoint: the COMPOSITE fires from pre-promotion + a commit hook, not a #[cfg(test)] caller.
- [ ] TD-1: N read from H-17 for the DSR-vs-null. TD-2: alpha LOCKED. TD-4: relations mutation-tested by H-27.
- [ ] GD-1: MetamorphicTestRunner + RelationLibrary + MrViolated REUSED. GD-2: the null-band re-computes under ELAI_SEED.
- [ ] W-10: Tier-1 relations fail-closed; Tier-2 permutation sidecar fixture + live checkbox.
- [ ] cargo check --workspace GREEN; auditor PASS. No new dependency (permutations are numpy in the existing sidecar).

## WIRING DONE-GATE (clause by clause)
- W-1: --run-trigger is ACCEPTANCE item 1; the COMPOSITE (produced) consumed on the live gate path (W-8) reached by promotion + commit hook (W-3).
- W-2: no override; kill-switch only. W-3: live gate path; not test-only.
- W-4: frontend_pair = null-band drawer; cross_lang_diff PASS. W-5: runnable fixture -> real row; Tier-1 in-process relations.
- W-6: surviving-edge flag visible in dashboard. W-7: metamorphic_testing.rs:104 + H-17 trial_count + rule_engine.rs:98 grep-confirmed (TS-4/TS-1); trading relations absent (NN-6).
- W-8: the named relations (produced) consumed by the COMPOSITE; MrViolated reused for per-relation failures.
- W-9: +TradingNullTest Pattern-2 baseline+1 (violations reuse MrViolated — minimise bumps); bump e45:294.
- W-10: SPLIT — Tier-1 relations fail-closed; Tier-2 permutation sidecar fixture + live checkbox. TD-1/2/4: N-deflated, locked alpha, self-validated.

## Tests to add
- crates/contracts/tests/h22_trading_nulls_contract.rs (~170 LOC): (a) leaky strategy (edge survives shuffle) ->
  Block(FutureShuffleSurvived); (b) genuine strategy (edge collapses) -> Allow; (c) permutation null fail -> Block(PermutationNullFail);
  (d) cost not monotone+concave -> Block(CostLawViolated); (e) non-deterministic equity curve under ELAI_SEED -> Block(SeedNondeterministic);
  (f) alpha is mutable=0; (g) the named relations register via parse_relation + reuse MrViolated.
- crates/contracts/tests/fixtures/trading_nulls/ (a leaky + a genuine strategy) + the Tier-2 permutation sidecar fixture.

## Files touched
- MODIFY: crates/orchestrator/src/metamorphic_testing.rs (~120 LOC: 7 new named relations in the shipped library)
- CREATE: python/.../permutation_null_sidecar.py (~120 LOC: the Tier-2 frozen permutation/DSR-vs-null evaluator)
- CREATE: crates/rules/seed/finance/trading_nulls.toml (the mutable=0 LOCKED COMPOSITE)
- MODIFY: crates/orchestrator/src/ssb.rs (+TradingNullTest Pattern-2; MrViolated reused) + action_type_lock.rs + e45 :294
- MODIFY: the pre-promotion path + a backtest-commit hook (~20 LOC: route through the COMPOSITE)
- MODIFY: .claude/gate-manifest.txt (freeze the permutation sidecar + alpha)
- CREATE: crates/contracts/tests/h22_trading_nulls_contract.rs (~170 LOC)
- CREATE: workflow/reasoning/H-22-decisions.md (~100 LOC)

## Citations / Grounding
- design T-6; Chen et al. metamorphic testing taxonomy; sklearn permutation_test_score; Bailey-LdP DSR; sqrt-law of
  market impact. Paper-ingest permutation_test_score + the sqrt-law source; cite quote+page for the cost-law form.
- ELAI ground truth (TS-4/TS-1/NN-6, grep-confirmed 2026-06-04): metamorphic_testing.rs:104 passed / :63 as_str /
  RelationLibrary; H-17 trial_count; trading relations (future-shuffle/time-reversal/zero-drift) ABSENT.
- CLAUDE.md HARD RULES: blake3 (equity-curve hash), no .unwrap()/.expect(); TD-1/2/4; GD-4 (calibrate FP before default-ON).

## Open questions / risks
- Threshold calibration is the design's named main risk: the alpha + the null-band width must catch a real leak
  without flagging a genuine-but-noisy edge — bound the FP rate on >=3 real strategies before default-ON (GD-4).
- FutureShuffle vs H-21 truncation: both attack look-ahead but differently (shuffle tests survival, truncation tests
  column-change). They are complementary, not redundant — a leak that survives shuffle but changes under truncation
  (or vice versa) is caught by exactly one. Document the coverage split.
- The cost-law (sqrt-law) relevance at retail size is unconfirmed (design VERIFY R13) — verify before locking the relation.

## Next
CLUSTER D opens: H-23 (same-seed re-run hash equality) is the reproducibility centerpiece. H-22's null-band + H-18's
DSR-at-N + H-24's OOS generalization-spread are jointly the NECESSARY condition for a trustworthy edge (TD-5: not sufficient).

End of H-22.

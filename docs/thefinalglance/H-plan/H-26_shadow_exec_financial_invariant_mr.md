# H-26: Runtime differential / shadow execution + financial-invariant MR library (T-10)

STATUS: PLANNED (TRACK 3 TRUST-HARDENING; WAVE 3 CLUSTER E; W-10 tiered: in-process divergence gate Tier-1 + shadow-runner sidecar Tier-2)

Objective: static + wiring gates cannot see WRONG VALUES — a flipped P&L sign, a ULP-cancellation error, a put-call
parity violation. H-26 builds a seeded shadow executor that runs TWO implementations (new vs prior version / vs a
paraphrase variant / vs a QuantLib oracle) on shared inputs and gates on bounded ULP/relative divergence; plus a
versioned financial-invariant MR pack (put-call parity, strike-monotonicity, P&L additivity, scale-invariance,
numeric-stability) seeded from QuantLib's own invariant tests; plus an AST ban on naked float `==`. Daikon-mined
invariants enter ONLY as PROPOSED and must kill a seeded mutant first (TD-4 — an auto-mined invariant is not trusted
until it demonstrably catches a real bug). The off-agent artifact is the divergence / falsification record.

Estimated effort: ~1 session (M-L, ~8-10 h). W-10 SPLIT: Tier-1 = the in-process divergence gate + MR runner + the
  SSB row (fail-closed); Tier-2 = the shadow-runner sidecar that executes both impls (fixture harness + live checkbox).
Pre-requisites: the metamorphic_testing.rs MR framework SHIPPED (the financial-invariant MRs are new relations);
  mutation_testing.rs SHIPPED (the kill-a-seeded-mutant gate for mined invariants); the rules-engine MetricGate + AST_PATTERN SHIPPED.
Parallel-safe with: H-25 (anti-fabrication — disjoint), H-27 (which mutation-tests these MRs).
Source: design T-10 + PART 0; _grounding/H-TRUST-GROUNDING-MAP.md TS-4 + NN-10 + TD-3/4. Doctrine: H-0_INDEX.md.

## Context

Nothing in ELAI catches a wrong VALUE that compiles and passes a unit test — a refactor that flips a P&L sign, a
numeric-stability cancellation, a violated put-call parity. The design's T-10 is a COMPOSITE: a seeded shadow
executor (two impls on shared inputs, bounded ULP/relative divergence — FPDiff style) AND a versioned
financial-invariant MR pack (put-call parity, strike-monotonicity, P&L additivity, scale-invariance, numeric-
stability — seeded from QuantLib's own invariant tests) AND an AST ban on naked float `==`. The MR framework is
SHIPPED (metamorphic_testing.rs); the financial invariants are new named relations. Auto-mined invariants (Daikon)
are admitted ONLY as PROPOSED and must kill a seeded mutant before going ACTIVE (TD-4 — RV-Monitor/MOP + Daikon
discipline). The shadow-exec divergence + the MR falsification record are the off-agent artifacts.

## Existing state (B/C/D/E/F/G coverage)
- metamorphic_testing.rs MR framework (MetamorphicRelation, RelationLibrary, MetamorphicTestRunner, MrViolated SSB;
  passed :104) — SHIPPED (C-29). The financial-invariant MRs are new named relations here (GD-1).
- mutation_testing.rs (MutationRunner, mutation_score_pct, MutantOutcome) — SHIPPED (C-28). The kill-a-seeded-mutant
  gate that admits Daikon-mined invariants (a mined invariant must kill a mutant before ACTIVE — TD-4).
- rules-engine MetricGate + AST_PATTERN (rule_engine.rs:98; predicate.rs:102) — SHIPPED. The ULP-divergence gate + the naked-`==` ban.
- a seeded shadow executor + a financial-invariant MR pack + a naked-`==` ban — ABSENT (NN-10). The pure-BUILD surface.

## Reads (mandatory before coding)
- _grounding/H-TRUST-GROUNDING-MAP.md: TS-4, NN-10, TD-3, TD-4, the TRADING-LANE TIERING block, the H-26 symbol table.
- design T-10 + PART 0; H-22 (the metamorphic relations pattern); H-27 (which mutation-tests these MRs — TD-4).
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\metamorphic_testing.rs:60-130 (the MR framework to extend)
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\mutation_testing.rs (the kill-a-mutant gate for mined invariants)
- F:\.ELAI_workflow\.elai_cc\crates\rules\src\predicate.rs:100-120 (MetricGate + AstPattern)
- CLAUDE.md HARD RULES: blake3, no .unwrap()/.expect(); TD-3 (auto-mined is PROPOSED only), TD-4 (kill-a-mutant). Paper-ingest QuantLib invariant tests + FPDiff + Daikon.

## Tasks (WAVE 3 — Tier-1 divergence gate + Tier-2 shadow runner)

1. **Seeded shadow executor (Tier-2 runner + Tier-1 gate, ~120 LOC).** A shadow runner that executes two
   implementations (new vs prior version / vs paraphrase variant / vs QuantLib oracle) on the SAME seeded inputs;
   the Tier-1 MetricGate diffs the outputs with a bounded ULP/relative divergence and BLOCKs beyond the frozen
   bound. A refactor that flips a P&L sign diverges beyond the bound -> FAIL. Deterministic (ELAI_SEED).

2. **Financial-invariant MR pack (Tier-1, ~120 LOC).** New named MetamorphicRelations in the SHIPPED library:
   PutCallParity, StrikeMonotonicity, PnLAdditivity, ScaleInvariance, NumericStability — seeded from QuantLib's own
   invariant tests. A put-call-parity violation is caught by the pack (reuse MrViolated for failures — GD-1).

3. **AST ban on naked float `==` (~30 LOC).** An AstPattern (T1) failing on `==`/`!=` between floats in financial
   code (use a tolerance comparison). BLOCK at TESTING_FIRST.

4. **Daikon-mined-invariant admission gate (TD-4, ~40 LOC).** Auto-mined invariants enter as PROPOSED; a mined
   invariant must kill a seeded mutant (via the SHIPPED mutation_testing.rs) before it can go ACTIVE. An invariant
   that catches nothing is never trusted (TD-4 — verify-the-verifiers applied to mined invariants).

5. **Off-agent artifact (~30 LOC).** Emit `differential_divergence` (Pattern-2 with Kind: ShadowDiverged /
   InvariantViolated / NakedFloatEq; baseline+1 from LIVE 207/484, re-read + bump e45:294) carrying the divergence /
   falsification record. Reuse MrViolated for per-invariant failures.

6. **Tier-2 fixture harness + live checkbox (W-10).** A fixture: a sign-flipped refactor (must diverge) + a clean
   refactor (must diff within ULP) + a put-call-parity violation. The live shadow run is a separately-gated checkbox.

7. **Decision doc workflow/reasoning/H-26-decisions.md (~90 LOC).** Records: (a) shadow-exec divergence + the MR
   pack + the naked-`==` ban; (b) the QuantLib-seeded invariants + the Daikon-PROPOSED-must-kill-a-mutant discipline
   (TD-3/TD-4); (c) the divergence/falsification record is the off-agent artifact.

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/shadow_exec.rs::diff_shadow_impls (the Tier-1 ULP-divergence gate)
  - python/.../shadow_runner.py::run_both_impls (the Tier-2 frozen shadow runner)
  - crates/orchestrator/src/metamorphic_testing.rs::{PutCallParity, StrikeMonotonicity, PnLAdditivity, ScaleInvariance, NumericStability} (new financial MRs)
  - crates/rules/seed/finance/financial_invariants.toml (the MR pack + the naked-`==` AST ban + the Daikon-admission gate, mutable=0)
  - crates/orchestrator/src/ssb.rs::SsbActionType::DifferentialDivergence (Pattern-2 3-Kind; baseline+1) [violations reuse MrViolated]
consumes_symbols:
  - crates/orchestrator/src/metamorphic_testing.rs::{MetamorphicRelation, RelationLibrary, MetamorphicTestRunner, MrViolated} (REUSE the framework, GD-1)
  - crates/orchestrator/src/mutation_testing.rs::MutationRunner (the kill-a-mutant gate for mined invariants — REUSE, TD-4)
  - crates/rules/src/predicate.rs::{MetricGate, AstPattern} (REUSE)
reachable_from:
  - the pre-promotion path + a refactor/commit hook on financial code [PRODUCTION gates]
  - diff_shadow_impls -> the shadow runner -> DifferentialDivergence SSB row [off-agent artifact]
stub_ban:
  - NO PASS without the shadow divergence record (missing the second impl run = FAIL — W-1/R1)
  - NO auto-mined invariant ACTIVE without killing a seeded mutant (PROPOSED-only — TD-3/TD-4)
  - NO naked float `==` in financial code (AST ban)
  - NO re-implementation of the MR framework or the mutation runner (GD-1 P0); blake3; no .unwrap()/.expect() in prod
  - Tier-2 sidecar MUST NOT advertise fail-closed if fail-open (W-10)
frontend_pair:
  the divergence / falsification record in the trust cockpit. cross_lang_diff PASS. W-6: a sign-flip divergence visible in the dashboard.
observable_effect:
  trigger: "cd .elai_cc && cargo run -p elai-cli -- <run the differential gate on a fixture refactor that flips a P&L sign, NO env override>"
  expect_ssb: "a DifferentialDivergence(ShadowDiverged) row lands with the divergence beyond the ULP bound, recomputable from the journal under ELAI_SEED (GD-2 same-env)"
  expect_artifact: "the sign-flipped refactor diverges from the prior impl beyond the ULP bound -> FAIL; a clean refactor diffs within ULP -> PASS; a put-call-parity violation is caught by the MR pack; a naked float == is BLOCKed at build"

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/H-plan/H-26_shadow_exec_financial_invariant_mr.md --run-trigger
rg "diff_shadow_impls|PutCallParity|MutationRunner" .elai_cc/crates/orchestrator/src --glob '!*test*'   # the gate + new MRs + mined-invariant admission
rg -n "financial_invariants|naked.*float|== " .elai_cc/crates/rules/seed/finance/financial_invariants.toml  # the AST ban present
rg -c "blake2|sha2" .elai_cc/crates/orchestrator/src/shadow_exec.rs                                      # MUST be 0
test -f workflow/reasoning/H-26-decisions.md
```

## ACCEPTANCE
- [ ] W-1 done-gate: --run-trigger PASS (a sign-flipped refactor diverges beyond ULP -> FAIL; a put-call-parity violation caught). cargo-green != DONE.
- [ ] W-2 default-ON: the gate runs on every financial refactor/promotion with no env override; kill-switch only.
- [ ] W-3 production entrypoint: diff_shadow_impls fires from pre-promotion + a refactor hook, not a #[cfg(test)] caller.
- [ ] TD-3/TD-4: auto-mined invariants are PROPOSED-only + must kill a seeded mutant before ACTIVE.
- [ ] GD-1: MR framework + mutation runner REUSED. GD-2: the divergence record re-computes under ELAI_SEED (same-env).
- [ ] W-10: Tier-1 divergence gate fail-closed; Tier-2 shadow runner fixture + live checkbox.
- [ ] cargo check --workspace GREEN; auditor PASS. QuantLib oracle is opt-in Tier-2 (anti-slopsquat verify before any dep).

## WIRING DONE-GATE (clause by clause)
- W-1: --run-trigger is ACCEPTANCE item 1; diff_shadow_impls (produced) consumed on the live gate path (W-8) reached by promotion + refactor hook (W-3).
- W-2: no override; kill-switch only. W-3: live gate path; not test-only.
- W-4: frontend_pair = divergence drawer; cross_lang_diff PASS. W-5: runnable fixture -> real row; Tier-1 in-process divergence.
- W-6: sign-flip divergence visible in dashboard. W-7: metamorphic_testing.rs:104 + mutation_testing.rs + rule_engine.rs:98 grep-confirmed (TS-4); shadow exec + financial MRs absent (NN-10).
- W-8: diff_shadow_impls (produced) consumed on the gate path; MR framework + mutation runner reused.
- W-9: +DifferentialDivergence Pattern-2 baseline+1 (violations reuse MrViolated); bump e45:294.
- W-10: SPLIT — Tier-1 divergence gate fail-closed; Tier-2 shadow runner fixture + live checkbox. TD-3/4: PROPOSED-only mined invariants.

## Tests to add
- crates/contracts/tests/h26_shadow_exec_contract.rs (~160 LOC): (a) sign-flipped refactor -> ShadowDiverged;
  (b) clean refactor -> Allow (within ULP); (c) put-call-parity violation -> InvariantViolated; (d) naked float == ->
  NakedFloatEq Block; (e) a Daikon-mined invariant that catches nothing stays PROPOSED (does not go ACTIVE); (f) a
  mined invariant that kills a seeded mutant promotes; (g) ELAI_SEED twice -> identical divergence record.
- crates/contracts/tests/fixtures/shadow/ (a sign-flip refactor + a clean one + a parity violation) + the Tier-2 shadow-runner fixture.

## Files touched
- CREATE: crates/orchestrator/src/shadow_exec.rs (~120 LOC: diff_shadow_impls + the Daikon-admission gate)
- CREATE: python/.../shadow_runner.py (~120 LOC: the Tier-2 frozen two-impl runner)
- MODIFY: crates/orchestrator/src/metamorphic_testing.rs (~120 LOC: 5 new financial-invariant relations)
- CREATE: crates/rules/seed/finance/financial_invariants.toml (the MR pack + naked-`==` AST ban + admission gate, mutable=0)
- MODIFY: crates/orchestrator/src/ssb.rs (+DifferentialDivergence Pattern-2; MrViolated reused) + action_type_lock.rs + e45 :294
- MODIFY: the pre-promotion path + a refactor hook (~20 LOC)
- MODIFY: .claude/gate-manifest.txt (freeze the shadow runner + the MR pack + the ULP bound)
- CREATE: crates/contracts/tests/h26_shadow_exec_contract.rs (~160 LOC)
- CREATE: workflow/reasoning/H-26-decisions.md (~90 LOC)

## Citations / Grounding
- design T-10; RV-Monitor/MOP; Daikon (dynamic invariant mining); FPDiff (floating-point differential testing);
  QuantLib (the financial-invariant test source). Paper-ingest FPDiff + Daikon; WebSearch QuantLib invariant tests
  before seeding the MR pack. anti-slopsquat: QuantLib is an opt-in Tier-2 oracle dep — verify before adding.
- ELAI ground truth (TS-4/NN-10, grep-confirmed 2026-06-04): metamorphic_testing.rs:104 + RelationLibrary;
  mutation_testing.rs MutationRunner; shadow exec + financial-invariant MRs + naked-`==` ban ABSENT.
- CLAUDE.md HARD RULES: blake3, no .unwrap()/.expect(); TD-3/4; GD-1 (extend the MR framework, restate none).

## Open questions / risks
- The ULP/relative divergence bound is the key constant — too tight flags legitimate FP non-determinism, too loose
  misses a real value error; bound the FP rate on >=3 real refactors (GD-4) before default-ON. This bound is itself a
  prime mutant-corpus target for H-27 (a mis-set bound is a mis-specified gate).
- QuantLib as an oracle is a Tier-2 dep + a different numeric stack — divergence vs QuantLib may be a QuantLib
  convention difference, not a bug. Use it as an ADVISORY oracle (T3, surface for review), not a hard gate (TD-3);
  the hard gate is new-vs-prior-version divergence (same stack, sound).
- Daikon-mined invariants are noisy — the kill-a-seeded-mutant admission (TD-4) is the load-bearing filter; verify it
  rejects a trivially-true mined invariant before relying on the pack.

## Next
CLUSTER F (the meta-layer capstone) opens: H-27 (gate self-validation) turns the mutation + metamorphic tiers
RECURSIVELY onto the gates' own logic (incl the ULP bound here + the DSR/null bands) — the only sound defense
against a self-improving optimizer exploiting a mis-specified gate. H-28 guards self-mod reward tampering.

End of H-26.

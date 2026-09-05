# H-21: Runtime look-ahead / causality differential gate (T-5) — the un-gameable leakage backstop

STATUS: PLANNED (TRACK 3 TRUST-HARDENING; WAVE 3 CLUSTER B; W-10 tiered: in-process diff Tier-1 + frozen feature-runner sidecar Tier-2)

Objective: static leakage analysis (H-20) is structurally blind to dynamic look-ahead (a feature that reads
`iloc[t+1]` through a path the dataflow pass cannot trace; R5 — a name is not an edge). H-21 is the un-gameable
runtime backstop: run the feature/signal pipeline TWICE — once on full data, once on future-truncated / time-shifted
data — and assert the output feature/signal columns are IDENTICAL (ulp/relative tolerance) on the overlapping
region. Any column that CHANGES when the future is removed is using the future. This is deterministic (the design's
un-gameable gate, mirroring Freqtrade look-ahead-analysis perturb-and-diff). The off-agent artifact is the persisted
dataframe-diff (the agent did not author the future-truncated re-run).

Estimated effort: ~1 session (M, ~6-8 h). W-10 SPLIT: Tier-1 = the in-process column-diff + the SSB row (fail-closed
  done-gate); Tier-2 = the frozen feature-runner sidecar that executes the pipeline twice (deterministic fixture
  harness now + a separately-gated live checkbox). Reuses a REGRET-style feature runner.
Pre-requisites: the rules-engine MetricGate SHIPPED; DriftComparator's first-divergence-index pattern (determinism.rs
  :317) to mirror; H-20 (the static pass it backstops — they ship together, the runtime leg is authoritative).
Parallel-safe with: H-22 (metamorphic nulls). NOT a substitute for H-20 (static is cheap-and-early; runtime is sound-and-late).
Source: design T-5 + PART 0 (TB-2) + report 08; _grounding/H-TRUST-GROUNDING-MAP.md TS-2 + TS-5 + NN-5 + TD-5. Doctrine: H-0_INDEX.md.

## Context

The leakage class static + unit tests are structurally blind to (R5: a name appearing in text is not a dataflow
edge; sound static reachability for Python with dynamic dispatch does not exist) has no runtime gate today. The
design's T-5 is the perturb-and-diff backstop: a feature that uses the future will produce a DIFFERENT column when
the future is truncated; a causal feature diffs to zero. This is the load-bearing leg for the leakage class — H-20
is necessary-but-not-sufficient (it labels itself unsound for dynamic paths), and H-21 is the authoritative proof
(R4: measure the real objective — does the column change when the future is removed — not a proxy). The diff is
computed in Rust (Tier-1) over the two dataframes the frozen feature-runner produces (Tier-2). DriftComparator's
first-divergence-index reporting (determinism.rs:317) is the exact pattern: report the first changing column/index.

## Existing state (B/C/D/E/F/G coverage)
- rules-engine MetricGate (rule_engine.rs:98) — SHIPPED. Rust orchestrates the twice-run + diffs columns (GD-1).
- DriftComparator + first-divergence-index (crates/orchestrator/src/determinism.rs:317; DeterminismDriftPayload :403)
  — SHIPPED (C-27). The "report the first mismatching column/index" pattern H-21 mirrors (do NOT re-implement; reuse the shape).
- a perturb-and-diff feature runner — ABSENT (NN-5). The pure-BUILD Tier-2 surface (reuses a REGRET-style runner).

## Reads (mandatory before coding)
- _grounding/H-TRUST-GROUNDING-MAP.md: TS-2, TS-5, NN-5, TD-5, the TRADING-LANE TIERING block, the H-21 symbol table.
- design T-5 + PART 0 (TB-2) + report 08 (Freqtrade look-ahead-analysis); H-20_leakage_taint_composite.md (the static pass it backstops).
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\determinism.rs:317-410 (DriftComparator + first-divergence-index + DeterminismDriftPayload)
- F:\.ELAI_workflow\.elai_cc\crates\rules\src\rule_engine.rs:98 (MetricGate)
- CLAUDE.md HARD RULES: blake3, no .unwrap()/.expect(); ClockProvider; TD-5 (cross-hardware FP residue).

## Tasks (WAVE 3 — Tier-1 diff + Tier-2 frozen runner)

1. **Frozen feature-runner sidecar (Tier-2, ~120 LOC).** A frozen module that runs the strategy's feature/signal
   pipeline on (a) full data and (b) future-truncated/time-shifted data, emitting two dataframes (the feature/signal
   columns on the overlapping region). Deterministic (ELAI_SEED-seeded, no wall-clock). In the gate-manifest frozen
   set (TD-2). Reuses a REGRET-style feature runner where one exists.

2. **Rust column-diff gate (Tier-1, ~80 LOC).** A MetricGate that diffs the two dataframes column-by-column with a
   ulp/relative tolerance, mirroring DriftComparator's first-divergence-index reporting. ANY column that differs
   beyond tolerance on the overlapping region = BLOCK (the column uses the future). A causal pipeline diffs to zero =
   ALLOW. Missing a re-run output = FAIL (W-1/R1 — absence of the future-truncated run is proof the gate did not run).

3. **Off-agent artifact (~30 LOC).** Emit `lookahead_differential` (Pattern-2 with Kind: Causal / FutureLeak;
   baseline+1 from LIVE 207/484, re-read + bump e45:294) carrying the first changing column + index + the tolerance.
   The persisted dataframe-diff is the off-agent artifact (the agent did not author the future-truncated re-run).

4. **Tier-2 fixture harness + live checkbox (W-10).** A deterministic fixture: a feature using `iloc[t+1]` (must
   FAIL — its column differs under truncation) and a causal feature (must diff to zero). The live strategy run is a
   separately-gated checkbox (DEFERRED).

5. **Freeze (TD-2).** Add the feature-runner + the tolerance constant to gate-manifest.txt.

6. **Decision doc workflow/reasoning/H-21-decisions.md (~80 LOC).** Records: (a) why this is the un-gameable
   backstop (static is blind to dynamic paths — R5); (b) the truncation/shift method + the ulp tolerance; (c) the
   cross-hardware FP residue (TD-5 — the tolerance is relative, not exact bitwise; the verdict band is stable even if
   the last FP digit isn't); (d) it backstops H-20, it does not replace it (cheap-static-early + sound-runtime-late).

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/lookahead_differential.rs::diff_future_truncation (the Tier-1 column-diff gate)
  - python/.../lookahead_runner.py::run_pipeline_twice (the Tier-2 frozen feature-runner)
  - crates/orchestrator/src/ssb.rs::SsbActionType::LookaheadDifferential (Pattern-2 2-Kind; baseline+1)
consumes_symbols:
  - crates/orchestrator/src/determinism.rs::DriftComparator (the first-divergence-index reporting pattern — REUSE shape, GD-1)
  - crates/rules/src/rule_engine.rs::CompiledPredicate (MetricGate — REUSE)
  - crates/core (ELAI_SEED for the deterministic re-run)
reachable_from:
  - the pre-promotion path on any strategy with a feature/signal pipeline [PRODUCTION gate]
  - diff_future_truncation -> the twice-run diff -> LookaheadDifferential SSB row [off-agent artifact]
stub_ban:
  - NO ALLOW without both re-runs (missing the future-truncated output = FAIL, never SKIP — W-1/R1/R2)
  - NO name-grep substitute for the runtime diff (the dataframe-diff is the load-bearing proof — R4/R5)
  - Tier-2 sidecar MUST NOT advertise fail-closed if fail-open (W-10); blake3 / no .unwrap() in prod
  - exact-bitwise equality is NOT asserted cross-hardware (relative tolerance + stable verdict band — TD-5)
frontend_pair:
  the first changing column + index in a leakage drawer (shared with H-20). cross_lang_diff PASS. W-6: a future-leak diff visible in the dashboard.
observable_effect:
  trigger: "cd .elai_cc && cargo run -p elai-cli -- <run the differential gate on a fixture feature using iloc[t+1], NO env override>"
  expect_ssb: "a LookaheadDifferential(FutureLeak) row lands naming the first changing column + index, recomputable from the journal (GD-2 same-env)"
  expect_artifact: "the iloc[t+1] feature is BLOCKED because its column differs under future-truncation; a causal pipeline diffs to zero and PASSES"

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/H-plan/H-21_runtime_lookahead_differential.md --run-trigger
rg "diff_future_truncation|DriftComparator" .elai_cc/crates/orchestrator/src --glob '!*test*'   # the gate reuses the divergence pattern
rg -c "blake2|sha2" .elai_cc/crates/orchestrator/src/lookahead_differential.rs                   # MUST be 0
test -f workflow/reasoning/H-21-decisions.md
```

## ACCEPTANCE
- [ ] W-1 done-gate: --run-trigger PASS (an iloc[t+1] fixture BLOCKED with the first changing column reported; a
  causal pipeline diffs to zero and passes). cargo-green != DONE.
- [ ] W-2 default-ON: the gate runs with no env override; kill-switch only.
- [ ] W-3 production entrypoint: diff_future_truncation fires from the pre-promotion path, not a #[cfg(test)] caller.
- [ ] W-10: Tier-1 diff is the fail-closed done-gate; Tier-2 runner ships a fixture + a separately-gated live checkbox.
- [ ] TD-5: cross-hardware FP residue documented; relative tolerance + stable verdict band, not exact bitwise.
- [ ] GD-1: DriftComparator + rules engine REUSED. GD-2: the diff re-computes under ELAI_SEED (same-env).
- [ ] cargo check --workspace GREEN; auditor PASS. No new dependency.

## WIRING DONE-GATE (clause by clause)
- W-1: --run-trigger is ACCEPTANCE item 1; diff_future_truncation (produced) consumed on the live gate path (W-8) reached by promotion (W-3).
- W-2: no override; kill-switch only. W-3: live gate path; not test-only.
- W-4: frontend_pair = leakage drawer; cross_lang_diff PASS. W-5: runnable fixture -> real row; Tier-1 in-process diff.
- W-6: future-leak diff visible in dashboard. W-7: determinism.rs:317 DriftComparator + rule_engine.rs:98 grep-confirmed (TS-2/TS-5); runner absent (NN-5).
- W-8: diff_future_truncation (produced) consumed on the gate path; DriftComparator shape reused.
- W-9: +LookaheadDifferential Pattern-2 baseline+1; bump e45:294.
- W-10: SPLIT — Tier-1 diff fail-closed; Tier-2 feature-runner fixture + live checkbox. TD-5: FP residue honest.

## Tests to add
- crates/contracts/tests/h21_lookahead_differential_contract.rs (~150 LOC): (a) iloc[t+1] feature -> Block(FutureLeak)
  with the first changing column; (b) causal feature -> Allow (diff==0 within tolerance); (c) missing future-truncated
  output -> Block (fail-closed); (d) tolerance is relative not exact-bitwise; (e) ELAI_SEED twice -> identical diff (same-env).
- crates/contracts/tests/fixtures/lookahead/ (an iloc[t+1] feature + a causal feature) + the Tier-2 runner fixture.

## Files touched
- CREATE: crates/orchestrator/src/lookahead_differential.rs (~80 LOC: the Tier-1 column-diff)
- CREATE: python/.../lookahead_runner.py (~120 LOC: the Tier-2 frozen feature-runner)
- MODIFY: crates/orchestrator/src/ssb.rs (+LookaheadDifferential Pattern-2) + action_type_lock.rs + e45 :294
- MODIFY: the pre-promotion path (~20 LOC: route feature-pipeline strategies through diff_future_truncation)
- MODIFY: .claude/gate-manifest.txt (freeze the runner + tolerance)
- CREATE: crates/contracts/tests/h21_lookahead_differential_contract.rs (~150 LOC)
- CREATE: workflow/reasoning/H-21-decisions.md (~80 LOC)

## Citations / Grounding
- design T-5 + report 08; Freqtrade look-ahead-analysis (perturb-and-diff). WebSearch the Freqtrade method before coding.
- ELAI ground truth (TS-2/TS-5/NN-5, grep-confirmed 2026-06-04): determinism.rs:317 DriftComparator; rule_engine.rs:98
  MetricGate; perturb-and-diff feature runner ABSENT.
- CLAUDE.md HARD RULES: blake3, no .unwrap()/.expect(), ClockProvider; TD-5 (cross-hardware FP residue is physics, not asserted).

## Open questions / risks
- The future-truncation method must be sound for the strategy's data shape (some features legitimately depend on a
  lookback window — truncating the FUTURE only, never the past, is the invariant; verify the truncation boundary).
- ULP/relative tolerance must be tight enough to catch a real leak but loose enough to absorb FP non-determinism —
  bound the FP false-positive rate on >=3 real causal pipelines (GD-4) before default-ON.
- This gate runs the pipeline twice — cost is 2x a backtest; route it pre-promotion (not per-edit) to avoid stalling
  the search loop (W-10 — the static H-20 is the cheap per-edit gate; H-21 is the sound pre-promotion gate).

## Next
H-22 (trading metamorphic null COMPOSITE) catches the edge that SURVIVES a shuffled future (a leak H-20/H-21's
truncation might miss) and adds the permutation/zero-drift nulls. H-20 + H-21 + H-22 together close TB-2 (leakage)
and half of TB-1 (does-the-edge-survive-perturbation) at runtime.

End of H-21.

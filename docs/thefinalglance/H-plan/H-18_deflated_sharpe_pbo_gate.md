# H-18: Deflated-Sharpe / PBO / MinBTL promotion METRIC_GATE (T-2) — the #1 net-new trust gate

STATUS: PLANNED (TRACK 3 TRUST-HARDENING; WAVE 3; W-10 tiered: in-process rules-engine gate Tier-1 + frozen numpy/scipy sidecar Tier-2; depends H-17)

Objective: ELAI has NO finance-deflation evaluator wired to promotion at all (NN-2, zero hits for deflated_sharpe/
PBO/MinBTL). A strategy that beats a raw Sharpe is meaningless if it was the best of N trials (TB-1). H-18 builds a
LOCKED COMPOSITE rules-engine gate that BLOCKS strategy promotion unless the Deflated Sharpe Ratio (DSR) + the
Probability of Backtest Overfitting (PBO via CSCV) + the Minimum Backtest Length (MinBTL) all clear thresholds,
with N read from the H-17 trial ledger — and fails CLOSED if N is absent (trial_count_missing). The DSR/PBO/MinBTL
math runs in a FROZEN Python sidecar (numpy/scipy, Tier-2) that the self-mod agent cannot tune; the Rust METRIC_GATE
enforces the verdict (Tier-1) and persists the {DSR, PBO, N} triple as the off-agent promotion artifact.

Estimated effort: ~1 session (M, ~6-8 h). W-10 SPLIT: Tier-1 = the in-process rules-engine COMPOSITE + the SSB
  verdict row (fail-closed done-gate); Tier-2 = the frozen numpy/scipy DSR/PBO sidecar (ships a deterministic fixture
  harness exercising the real path + a separately-gated live checkbox — mirrors the seccodeplt/SAFEDELTA split).
Pre-requisites: H-17 (the trial ledger + trial_count() read API — HARD prereq); the rules engine COMPOSITE/MetricGate
  predicate SHIPPED (crates/rules, rule_engine.rs:98); the mutable=0 LOCKED tier SHIPPED (schema.rs:46, lifecycle.rs:871).
Parallel-safe with: H-19 (holdout budget — disjoint), H-20/H-21 (leakage — disjoint files). NOT parallel-safe with H-17 (consumes its read API).
Source: design T-2 + PART 0 (TB-1) + PART 1 (D1/D3/D4); _grounding/H-TRUST-GROUNDING-MAP.md TS-2 + TS-8 + NN-2 + TD-1/2/3. Doctrine: H-0_INDEX.md.

## Context

The keystone (H-17) records N; H-18 is the gate that DEFLATES by it. Deflated Sharpe Ratio (Bailey & Lopez de
Prado) adjusts the observed Sharpe for the number of trials, non-normality, and sample length; PBO (CSCV) estimates
the probability the in-sample-best configuration underperforms out-of-sample; MinBTL bounds how short a backtest can
be before an in-sample Sharpe is statistically meaningless. The design's #1 net-new gate (T-2) is a LOCKED COMPOSITE
that blocks promotion unless all three clear, with N read from H-17 and fail-closed if N is absent. Per TD-3, the LLM/
auditor never decides this — it is a deterministic numeric gate; per TD-2 the thresholds + the math live in the frozen
set so the self-mod loop cannot tune them down. The seccodeplt/SAFEDELTA runner (SecCodePltRunner, safedelta_runner.rs)
is the exact mirror pattern: Rust orchestrates a frozen scoring body and emits a measured SSB row.

## Existing state (B/C/D/E/F/G coverage)
- rules engine COMPOSITE + MetricGate (crates/rules/src/rule_engine.rs:98 CompiledPredicate; :1427 eval-order;
  VerdictResult Allow/Warn/Block/Escalate predicate.rs; mutable=0 tier schema.rs:46/lifecycle.rs:871) — SHIPPED.
  The deterministic-gate substrate H-18 instantiates a new LOCKED COMPOSITE in (GD-1; do NOT build a new engine).
- SecCodePltRunner + InsecureRateMeasured SSB (crates/cli/src/bench_seccodeplt.rs:295; benchmarks/safedelta_runner.rs
  :454/487; f2c_seccodeplt_baseline_smoke.rs:117) — SHIPPED. The frozen-evaluator-driven-by-Rust + measured-SSB-row
  pattern T-2 mirrors (a frozen scoring body, Rust enforces, an off-agent measured row lands).
- H-17 trial_count() / register_trial — the N source (this plan's hard prereq).
- deflated_sharpe / PBO / MinBTL math — ABSENT (NN-2). The pure-BUILD Tier-2 surface.

## Reads (mandatory before coding)
- _grounding/H-TRUST-GROUNDING-MAP.md: TS-2, TS-8, NN-2, TD-1, TD-2, TD-3, the TRADING-LANE TIERING block, the H-18 symbol table.
- design T-2 + PART 0 (TB-1) + PART 1 (D3/D4); H-17_trial_ledger_keystone.md (the trial_count read API contract).
- F:\.ELAI_workflow\.elai_cc\crates\rules\src\rule_engine.rs:98-330 (CompiledPredicate + eval order) + predicate.rs (Predicate + VerdictResult)
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\benchmarks\safedelta_runner.rs:440-490 (the frozen-runner + measured-row pattern to mirror)
- F:\.ELAI_workflow\.elai_cc\crates\cli\src\bench_seccodeplt.rs:290-300 (SecCodePltRunner::from_embedded — the deterministic fixture harness pattern)
- CLAUDE.md HARD RULES: blake3, no .unwrap()/.expect() in prod; anti-slopsquatting (numpy/scipy already in the sidecar — verify); TD-1/2/3.

## Tasks (WAVE 3 — Tier-1 rules gate + Tier-2 frozen sidecar)

1. **Frozen DSR/PBO/MinBTL sidecar (Tier-2, ~180 LOC Python).** A frozen Python module (numpy/scipy) computing
   Deflated Sharpe Ratio (skew/kurtosis-adjusted, deflated by N from H-17), PBO via CSCV (combinatorially-symmetric
   cross-validation), and MinBTL. Input: the return series + N + the user-LOCKED alpha. Output: {dsr, dsr_pvalue,
   pbo, minbtl, n_used}. Deterministic (no wall-clock, no unseeded RNG; CSCV partition seeded by ELAI_SEED). Lives
   in python/elai_inference or a dedicated frozen eval module; in the gate-manifest frozen set (TD-2).

2. **Rust COMPOSITE METRIC_GATE (Tier-1, ~90 LOC).** A new rules-engine COMPOSITE (mutable=0, domain finance)
   that: reads N via H-17 trial_count(); FAILS CLOSED with trial_count_missing if N==0 for the reported config
   (TD-1); orchestrates the Tier-2 sidecar; BLOCKs promotion unless dsr clears (Harvey-Liu deflated t>3.0 default),
   pbo < threshold, and the series length >= MinBTL. The verdict is deterministic (TD-3 — no LLM input). Thresholds
   are LOCKED constants in the mutable=0 rule (TD-2).

3. **Off-agent promotion artifact (~30 LOC).** Emit the {dsr, dsr_pvalue, pbo, minbtl, n_used, verdict} triple as
   an SSB row (reuse the InsecureRateMeasured/measured-row pattern; prefer a Pattern-2 single-variant-with-Kind
   `deflation_gate_evaluated`; baseline+1 from LIVE 207/484, re-read at edit + bump e45:294). The return series +
   N are the off-agent inputs the agent did not author; the persisted triple is recomputable from the journal (GD-2).

4. **Tier-2 fixture harness + live checkbox (W-10).** A deterministic fixture (a known-overfit series at high N
   that must FAIL DSR, and a genuine series at low N that must PASS) exercising the REAL sidecar path now; the live
   trading-strategy run is a separately-gated checkbox (DEFERRED, no external claim before it — GD-5).

5. **Freeze (TD-2).** Add the sidecar module + the thresholds + the COMPOSITE rule source to gate-manifest.txt.

6. **Decision doc workflow/reasoning/H-18-decisions.md (~90 LOC).** Records: (a) DSR/PBO/MinBTL definitions +
   the LOCKED thresholds + their grounding (Bailey-LdP, Harvey-Liu t>3.0); (b) fail-closed on trial_count_missing
   (TD-1); (c) why the math is Tier-2-frozen and the verdict Tier-1-deterministic (TD-2/TD-3); (d) the honest residue
   (GENUINE-vs-lucky is a NECESSARY not SUFFICIENT condition — final capital allocation stays human Layer-3, TD-5).

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/deflation_gate.rs::evaluate_deflation (the Tier-1 COMPOSITE orchestrator)
  - python/.../deflation_sidecar.py::compute_dsr_pbo_minbtl (the Tier-2 frozen evaluator)
  - crates/orchestrator/src/ssb.rs::SsbActionType::DeflationGateEvaluated (off-agent promotion artifact; baseline+1, Pattern-2)
  - crates/rules/seed/finance/deflation_promotion.toml (the mutable=0 LOCKED COMPOSITE seed)
consumes_symbols:
  - crates/orchestrator/src/trial_ledger.rs::trial_count (H-17 — N source; REUSE, GD-1)
  - crates/rules/src/rule_engine.rs::CompiledPredicate (COMPOSITE/MetricGate — REUSE)
  - crates/orchestrator/src/benchmarks/safedelta_runner.rs (the frozen-runner + measured-row pattern — mirror, GD-1)
reachable_from:
  - the strategy promotion path (a discovered/written strategy hits evaluate_deflation before promotion) [PRODUCTION gate]
  - evaluate_deflation -> trial_count() -> fail-closed on trial_count_missing [the N dependency]
  - evaluate_deflation -> DeflationGateEvaluated SSB row [off-agent artifact]
stub_ban:
  - NO promotion without a DSR/PBO/MinBTL verdict (missing math = BLOCK, never SKIP — W-1/R1)
  - NO agent-tunable thresholds (LOCKED mutable=0 — TD-2); NO LLM verdict (deterministic only — TD-3)
  - NO promotion when trial_count==0 for the config (fail closed, trial_count_missing — TD-1)
  - NO re-implementation of the trial ledger or the rules engine (GD-1 P0)
  - Tier-2 sidecar MUST NOT advertise fail-closed if it is fail-open (W-10); blake3 / no .unwrap() in prod
frontend_pair:
  the {DSR, PBO, N} promotion triple in a trust-cockpit drawer (cross_lang_diff PASS). W-6 E2E: a blocked promotion
  shows the failing DSR-at-N in the dashboard.
observable_effect:
  trigger: "cd .elai_cc && cargo run -p elai-cli -- <promote a fixture strategy whose raw Sharpe is high but whose
    DSR-at-N fails, with NO env override>"
  expect_ssb: "a DeflationGateEvaluated row lands carrying {dsr, dsr_pvalue, pbo, minbtl, n_used, verdict=Block},
    recomputable from the journal under ELAI_SEED (GD-2)"
  expect_artifact: "the strategy that beats raw Sharpe but fails DSR-at-N is BLOCKED from promotion; a genuine
    low-N strategy PASSES; a config with zero trial rows is BLOCKED with trial_count_missing (fail-closed)"

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/H-plan/H-18_deflated_sharpe_pbo_gate.md --run-trigger
rg "evaluate_deflation|trial_count" .elai_cc/crates/orchestrator/src --glob '!*test*'   # gate calls the H-17 read API
rg -n "deflation_promotion|mutable.*=.*0" .elai_cc/crates/rules/seed/finance/deflation_promotion.toml  # LOCKED seed present
rg -c "blake2|sha2" .elai_cc/crates/orchestrator/src/deflation_gate.rs                  # MUST be 0
test -f workflow/reasoning/H-18-decisions.md
```

## ACCEPTANCE
- [ ] W-1 done-gate: wiring_contract_check.py --run-trigger PASS (a high-raw-Sharpe / failing-DSR strategy BLOCKED,
  the DeflationGateEvaluated row landed). cargo-green != DONE.
- [ ] W-2 default-ON: the gate runs on every promotion with no env override; kill-switch only.
- [ ] W-3 production entrypoint: evaluate_deflation fires from the live promotion path, not a #[cfg(test)] caller.
- [ ] TD-1: N is READ from H-17; trial_count_missing fails closed. TD-2: thresholds + math LOCKED/frozen.
- [ ] TD-3: the verdict is deterministic; no LLM input. GD-1: trial ledger + rules engine REUSED.
- [ ] W-10: Tier-1 in-process gate is the fail-closed done-gate; the Tier-2 sidecar ships a fixture harness + a
  separately-gated live checkbox and does NOT advertise fail-closed if fail-open.
- [ ] GD-2 reproducible: the {DSR,PBO,N} triple re-computes byte-identically under ELAI_SEED.
- [ ] cargo check --workspace GREEN; auditor PASS. numpy/scipy already in the sidecar (anti-slopsquat verify, no new dep).

## WIRING DONE-GATE (clause by clause)
- W-1: --run-trigger is ACCEPTANCE item 1; the gate (produced) is consumed on the live promotion path (W-8) reached by promotion (W-3).
- W-2: runs with no override; kill-switch only.
- W-3: live promotion path; not test-only.
- W-4: frontend_pair = the DSR/PBO/N drawer; cross_lang_diff PASS.
- W-5: observable_effect is a runnable promotion -> a real journal row; Tier-1 in-process.
- W-6: a blocked promotion's DSR-at-N visible in the dashboard (dashboard-smoker).
- W-7: rule_engine.rs:98 / safedelta_runner.rs / H-17 trial_count grep-confirmed (TS-2/TS-8); DSR/PBO absent (NN-2).
- W-8: evaluate_deflation (produced) consumed on promotion; trial_count (H-17) consumed by the gate.
- W-9: +DeflationGateEvaluated baseline+1 (Pattern-2); bump e45:294.
- W-10: SPLIT — Tier-1 gate fail-closed; Tier-2 numpy/scipy sidecar fixture + live checkbox.
- GD-1/2/TD-1/2/3: compose; reproducible; N-deflated, frozen, deterministic.

## Tests to add
- crates/contracts/tests/h18_deflation_gate_contract.rs (~160 LOC): (a) high-raw-Sharpe/failing-DSR-at-N -> Block;
  (b) genuine low-N series -> Allow; (c) trial_count==0 -> Block(trial_count_missing); (d) thresholds are mutable=0
  (cannot be edited at runtime); (e) the verdict is independent of any LLM input; (f) ELAI_SEED twice -> identical triple.
- crates/contracts/tests/fixtures/deflation/ (an overfit-high-N series + a genuine-low-N series) + the Tier-2 sidecar fixture.

## Files touched
- CREATE: crates/orchestrator/src/deflation_gate.rs (~90 LOC: the Tier-1 COMPOSITE orchestrator)
- CREATE: python/.../deflation_sidecar.py (~180 LOC: numpy/scipy DSR/PBO/MinBTL — Tier-2 frozen)
- CREATE: crates/rules/seed/finance/deflation_promotion.toml (the mutable=0 LOCKED COMPOSITE seed)
- MODIFY: crates/orchestrator/src/ssb.rs (+DeflationGateEvaluated) + action_type_lock.rs + e45 :294
- MODIFY: the promotion entrypoint (~20 LOC: route promotion through evaluate_deflation)
- MODIFY: .claude/gate-manifest.txt (freeze the sidecar + thresholds + the COMPOSITE)
- CREATE: crates/contracts/tests/h18_deflation_gate_contract.rs (~160 LOC)
- CREATE: workflow/reasoning/H-18-decisions.md (~90 LOC)

## Citations / Grounding
- design T-2; Bailey & Lopez de Prado (Deflated Sharpe Ratio; PBO via CSCV; MinBTL); Harvey & Liu (deflated t>3.0).
  WebSearch + paper-ingest the DSR/PBO/MinBTL formulae before coding the sidecar; cite quote+page for any threshold.
- ELAI ground truth (TS-2/TS-8/NN-2, grep-confirmed 2026-06-04): rule_engine.rs:98 COMPOSITE; safedelta_runner.rs
  :454 the frozen-runner pattern; H-17 trial_count; deflated_sharpe/PBO ABSENT.
- CLAUDE.md HARD RULES: blake3, no .unwrap()/.expect(); TD-1/2/3; GD-5 (no fabricated guarantee — DSR is necessary not sufficient).

## Open questions / risks
- The DSR/PBO formulae are subtle (skew/kurtosis adjustment, CSCV partition count S). A mis-specified DSR is itself
  a defect H-27 (gate self-validation) must catch — H-18's COMPOSITE is a prime mutant-corpus target.
- MinBTL relevance at retail strategy lengths is unconfirmed (design VERIFY note R13/MinBTL) — verify before locking the floor.
- The Tier-2 sidecar breaks ELAI_SEED bit-identity across hardware (FP) — scope GD-2 to same-env/same-seed (TD-5);
  the verdict (Block/Allow) must be stable even if the last DSR digit isn't (use a threshold band, not exact equality).

## Next
H-19 (reusable-holdout budget) bounds adaptive OOS reuse so the search cannot mine the holdout as an unlimited
analyst — the deployment-trust half that pairs with H-18 (design WAVE 5). H-25 (result manifest) binds the reported
DSR/PBO number to its provenance. H-22 nulls + H-18 DSR jointly are the necessary condition for a trustworthy edge.

End of H-18.

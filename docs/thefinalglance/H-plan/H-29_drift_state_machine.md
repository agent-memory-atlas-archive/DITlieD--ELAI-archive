# H-29: LOCKED fail-closed drift state machine (T-13) — gate the resume/keep-live FSM edge

STATUS: PLANNED (TRACK 3 TRUST-HARDENING; WAVE 3 CLUSTER G; in-process BUILD composing the SHIPPED model_drift/canary signals; depends H-18)

Objective: ELAI has a model-drift canary (G-10 model_drift.rs) — one signal, not the full interlock; no 90-day model
TTL; the concept-drift half is not wired to a label-fed detector. A deployed strategy whose input distribution shifts,
whose data vendor changes schema, or whose live P&L diverges from the backtest will keep trading until something
breaks. H-29 builds a LOCKED(mutable=0) warning->drift->halt interlock across FOUR axes: PSI on inputs AND outputs;
schema-fingerprint (vendor drift); rolling-14d live-vs-backtest deviation; and a 90-day model/strategy deployment TTL
+ deflated-Sharpe-at-promotion (reading from H-18). It gates the FSM transition that RESUMES/CONTINUES live trading.
Missing evidence = HALT (not SKIP). HONEST GUARANTEE (TD-5): "halt within ONE realized-label cycle," NOT "never trade
a regime break" — P(Y|X) concept drift BEFORE labels arrive is provably undetectable without ground truth (the residual
is a T3 conformal/novelty advisory). Off-agent artifacts: feature logs, realized P&L, schema fingerprints.

Estimated effort: ~1 session (M, ~6-8 h). In-process; composes shipped signals into a LOCKED interlock.
Pre-requisites: model_drift.rs SHIPPED (G-10; system_fingerprint :149/161, g10_model_drift_contract.rs); canary.rs
  CanaryFamily SHIPPED (D-7); H-18 (deflated-Sharpe-at-promotion for the TTL re-check); the rules-engine MetricGate +
  SchemaRequired + COMPOSITE + mutable=0 tier SHIPPED; the FSM resume transition (the edge H-29 gates).
Parallel-safe with: nothing else in CLUSTER G (it is CLUSTER G). Composes H-18 (DSR re-check), H-19 (the deployment-trust pair).
Source: design T-13 + PART 4 (R35-R39) + PART 5 (residue); _grounding/H-TRUST-GROUNDING-MAP.md TS-8 + NN-13 + TD-2/5. Doctrine: H-0_INDEX.md.

## Context

The model-drift canary (G-10) is one signal; the design's T-13 is the full interlock that GATES the live-trading
resume FSM edge. Four axes: PSI on inputs AND outputs (population-stability index — distribution shift), schema-
fingerprint (a data vendor silently changed schema), rolling-14d live-vs-backtest deviation (the strategy is no
longer behaving as backtested), and a 90-day model/strategy deployment TTL (force a deflated-Sharpe re-check at
promotion via H-18). A LOCKED(mutable=0) warning->drift->halt interlock: warning surfaces, drift escalates, halt
stops live trading and requires a full backtest re-run. Missing evidence = HALT (the design's fail-closed posture —
never SKIP). The honest guarantee (TD-5, PART 5 residue): this halts within ONE realized-label cycle, NOT "never
trade a regime break" — P(Y|X) concept drift before labels arrive is provably undetectable without ground truth
(the residual routes to a T3 conformal/novelty advisory, the SHIPPED conformal_calibration.rs from D-16/G-7).

## Existing state (B/C/D/E/F/G coverage)
- model_drift.rs (crates/orchestrator/src/model_drift.rs:149/161 system_fingerprint; g10_model_drift_contract.rs;
  model_drift_suspected SSB, G-10) — SHIPPED. ONE signal; H-29 composes it into the interlock (GD-1; do NOT re-detect).
- canary.rs CanaryFamily + FamilyEvalResult + the non-determinism band (D-7) — SHIPPED. The regression-delta signal.
- conformal_calibration.rs (decide / claim_score / tau / TauUpdater, D-16/G-7) — SHIPPED. The T3 concept-drift residual advisory.
- rules-engine MetricGate + SchemaRequired + COMPOSITE + mutable=0 (rule_engine.rs:98; predicate.rs:102; schema.rs:46)
  — SHIPPED. The four-axis interlock predicates + the LOCKED warning->drift->halt COMPOSITE.
- H-18 deflated-Sharpe-at-promotion — the TTL re-check (prereq).
- the LOCKED warning->drift->halt interlock gating the resume FSM edge + the 90-day TTL + label-fed concept-drift — ABSENT (NN-13). BUILD.

## Reads (mandatory before coding)
- _grounding/H-TRUST-GROUNDING-MAP.md: TS-8, NN-13, TD-2, TD-5, the H-29 symbol table.
- design T-13 + PART 4 (R35-R39) + PART 5; H-18 (the DSR re-check), H-19 (the deployment-trust pair).
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\model_drift.rs:140-170 (the shipped drift signal + system_fingerprint)
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\canary.rs (CanaryFamily + the non-determinism band)
- F:\.ELAI_workflow\.elai_cc\crates\rules\src\rule_engine.rs:98 + predicate.rs:100-120 (MetricGate + SchemaRequired + Composite)
- the FSM resume/continue transition (the edge to gate) — grep the FSM for the live-trading resume edge
- CLAUDE.md HARD RULES: blake3 (schema fingerprint), no .unwrap()/.expect(), ClockProvider (the 14d/90d windows); TD-2/5. Paper-ingest NannyML/ADWIN/DDM/Page-Hinkley.

## Tasks (WAVE 3 — the deployment-trust interlock)

1. **Four-axis drift signals (~120 LOC).** (a) PSI on inputs AND outputs (MetricGate; the distribution-shift index);
   (b) schema-fingerprint (SchemaRequired + blake3 over the vendor data schema — a change = vendor drift); (c)
   rolling-14d live-vs-backtest deviation (MetricGate over the realized-P&L vs backtest series, ClockProvider window);
   (d) 90-day model/strategy deployment TTL (a COMPOSITE forcing an H-18 deflated-Sharpe re-check at promotion).
   Compose model_drift.rs + canary.rs signals (GD-1; do NOT re-detect).

2. **LOCKED warning->drift->halt interlock (~80 LOC).** A LOCKED(mutable=0) COMPOSITE state machine: warning
   (surface), drift (escalate), halt (stop live trading, require a full backtest re-run). Missing evidence on ANY
   axis = HALT, never SKIP (W-1/R1 — a missing feature log means the signal never ran, so the safe state is halt).

3. **Gate the resume/keep-live FSM edge (~40 LOC).** The interlock gates the FSM transition that resumes/continues
   live trading. A vendor schema change HALTS live trading until a full backtest re-run; an output-PSI breach + a 14d
   deviation breach trips HALT. The gate is the FSM edge guard, not a passive monitor.

4. **Off-agent artifact + the T3 residual (~30 LOC).** Emit `drift_interlock` (Pattern-2 with Kind: Warning / Drift /
   Halt; baseline+1 from LIVE 207/484, re-read + bump e45:294) carrying the per-axis signal + the verdict. The feature
   logs / realized P&L / schema fingerprints are the off-agent artifacts. The P(Y|X)-concept-drift residual (before
   labels) routes to the SHIPPED conformal_calibration.rs as a T3 advisory (TD-5 — NOT a hard halt, honest).

5. **Freeze (TD-2).** The interlock thresholds (PSI band, 14d deviation band, 90d TTL) are LOCKED mutable=0 + in the
   gate-manifest frozen set.

6. **Decision doc workflow/reasoning/H-29-decisions.md (~90 LOC).** Records: (a) the four axes + the LOCKED
   warning->drift->halt; (b) the honest guarantee "halt within ONE realized-label cycle" NOT "never trade a regime
   break" (TD-5, PART 5 — P(Y|X) drift before labels is undecidable); (c) the residual routes to T3 conformal advisory;
   (d) composes model_drift/canary, restates neither (GD-1); (e) missing evidence = HALT.

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/drift_interlock.rs::evaluate_drift_interlock (the four-axis LOCKED warning->drift->halt COMPOSITE)
  - crates/rules/seed/finance/drift_interlock.toml (the four-axis predicates + the LOCKED bands + the 90d TTL, mutable=0)
  - crates/orchestrator/src/ssb.rs::SsbActionType::DriftInterlock (Pattern-2 3-Kind; baseline+1)
consumes_symbols:
  - crates/orchestrator/src/model_drift.rs (the shipped drift signal + system_fingerprint — REUSE, GD-1)
  - crates/orchestrator/src/canary.rs::CanaryFamily (the regression-delta signal — REUSE)
  - crates/orchestrator/src/conformal_calibration.rs::decide (the T3 concept-drift residual advisory — REUSE)
  - crates/orchestrator/src/deflation_gate.rs::evaluate_deflation (H-18 — the 90d TTL DSR re-check)
  - crates/rules/src/{rule_engine.rs::CompiledPredicate, predicate.rs::SchemaRequired} (REUSE) ; the FSM resume edge
reachable_from:
  - the FSM resume/keep-live transition (the live-trading resume edge routes through evaluate_drift_interlock) [PRODUCTION gate]
  - evaluate_drift_interlock -> the four-axis checks -> DriftInterlock SSB row [off-agent artifact]
stub_ban:
  - NO resume without the interlock (missing evidence on any axis = HALT, never SKIP — W-1/R1)
  - NO agent-tunable bands/TTL (LOCKED mutable=0 — TD-2)
  - NO "never trade a regime break" guarantee (the honest scope is "halt within one label cycle" — TD-5)
  - NO re-implementation of model_drift/canary/conformal (GD-1 P0); blake3 (schema fingerprint); no .unwrap()/.expect() in prod
frontend_pair:
  the four-axis drift state (warning/drift/halt per axis) in the deployment cockpit. cross_lang_diff PASS. W-6: a HALT on vendor-schema-change visible in the dashboard.
observable_effect:
  trigger: "cd .elai_cc && cargo run -p elai-cli -- <attempt to resume live trading after a fixture vendor-schema change + an output-PSI breach + a 14d-deviation breach, NO env override>"
  expect_ssb: "DriftInterlock(Halt) rows land naming the breached axes (schema-fingerprint + output-PSI + 14d-deviation), recomputable from the journal under ELAI_SEED (GD-2)"
  expect_artifact: "the resume FSM edge is HALTED until a full backtest re-run (the vendor schema change + the PSI/deviation breaches trip HALT); a clean deployment within all bands resumes; the 90d TTL forces an H-18 DSR re-check"

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/H-plan/H-29_drift_state_machine.md --run-trigger
rg "evaluate_drift_interlock|model_drift|CanaryFamily|evaluate_deflation" .elai_cc/crates/orchestrator/src --glob '!*test*'   # composes the shipped signals + H-18
rg -n "drift_interlock|mutable.*=.*0|ttl" .elai_cc/crates/rules/seed/finance/drift_interlock.toml   # the LOCKED bands + 90d TTL present
rg -c "blake2|sha2" .elai_cc/crates/orchestrator/src/drift_interlock.rs                              # MUST be 0 (schema fingerprint is blake3)
test -f workflow/reasoning/H-29-decisions.md
```

## ACCEPTANCE
- [ ] W-1 done-gate: --run-trigger PASS (a vendor schema change + PSI/deviation breach HALTS the resume edge; a clean deployment resumes). cargo-green != DONE.
- [ ] W-2 default-ON: the interlock gates every resume with no env override; kill-switch only.
- [ ] W-3 production entrypoint: evaluate_drift_interlock fires from the FSM resume transition, not a #[cfg(test)] caller.
- [ ] TD-2: the bands + the 90d TTL are LOCKED mutable=0 + frozen. TD-5: the honest "halt within one label cycle" guarantee documented; the P(Y|X) residual routes to T3 conformal advisory.
- [ ] GD-1: model_drift + canary + conformal + H-18 REUSED (grep shows one of each). GD-2: the drift verdict re-computes under ELAI_SEED.
- [ ] cargo check --workspace GREEN; auditor PASS. No new dependency (PSI is closed-form; ADWIN/DDM is the label-fed half, scoped).

## WIRING DONE-GATE (clause by clause)
- W-1: --run-trigger is ACCEPTANCE item 1; evaluate_drift_interlock (produced) consumed on the FSM resume edge (W-8) reached by resume (W-3).
- W-2: no override; kill-switch only. W-3: live resume FSM edge; not test-only.
- W-4: frontend_pair = deployment cockpit; cross_lang_diff PASS. W-5: runnable resume attempt -> real rows; Tier-1 in-process.
- W-6: a vendor-schema HALT visible in the dashboard. W-7: model_drift.rs:149 + canary.rs + conformal_calibration.rs + H-18 evaluate_deflation grep-confirmed (TS-8); the interlock absent (NN-13).
- W-8: evaluate_drift_interlock (produced) consumed on the resume edge; model_drift/canary/conformal/H-18 reused.
- W-9: +DriftInterlock Pattern-2 baseline+1; bump e45:294.
- W-10: Tier-1 in-process (PSI/fingerprint/deviation are closed-form over logged data). TD-2/5: locked bands, honest one-label-cycle guarantee.

## Tests to add
- crates/contracts/tests/h29_drift_interlock_contract.rs (~160 LOC): (a) vendor schema-fingerprint change -> Halt;
  (b) output-PSI breach + 14d-deviation breach -> Halt; (c) clean deployment within all bands -> resume; (d) 90d TTL
  expiry -> forces an H-18 DSR re-check; (e) a missing feature log on any axis -> Halt (fail-closed, not SKIP);
  (f) the bands/TTL are mutable=0; (g) the P(Y|X) residual routes to conformal T3 advisory (not a hard halt); (h) ELAI_SEED twice -> identical verdict.
- crates/contracts/tests/fixtures/drift/ (a vendor-schema-change deployment + a PSI/deviation-breach one + a clean one).

## Files touched
- CREATE: crates/orchestrator/src/drift_interlock.rs (~160 LOC: evaluate_drift_interlock + the four-axis composition)
- CREATE: crates/rules/seed/finance/drift_interlock.toml (the four-axis predicates + LOCKED bands + 90d TTL, mutable=0)
- MODIFY: crates/orchestrator/src/ssb.rs (+DriftInterlock Pattern-2) + action_type_lock.rs + e45 :294
- MODIFY: the FSM resume/keep-live transition (~20 LOC: route the resume edge through evaluate_drift_interlock)
- MODIFY: .claude/gate-manifest.txt (freeze the bands + the 90d TTL)
- CREATE: crates/contracts/tests/h29_drift_interlock_contract.rs (~160 LOC)
- CREATE: workflow/reasoning/H-29-decisions.md (~90 LOC)

## Citations / Grounding
- design T-13 + PART 4 (R35-R39) + PART 5; NannyML / Gama et al. / Lu et al. concept-drift surveys; ADWIN / DDM /
  Page-Hinkley (the label-fed concept-drift detectors); PSI (population stability index). Paper-ingest the concept-drift
  surveys; the "halt within one label cycle" guarantee is the honest scope (P(Y|X) before labels is undecidable).
- ELAI ground truth (TS-8/NN-13, grep-confirmed 2026-06-04): model_drift.rs:149/161 (G-10); canary.rs CanaryFamily (D-7);
  conformal_calibration.rs (D-16/G-7); H-18 evaluate_deflation; the LOCKED interlock + 90d TTL + label-fed half ABSENT.
- CLAUDE.md HARD RULES: blake3, no .unwrap()/.expect(), ClockProvider; TD-2/5; GD-1 (compose the shipped signals).

## Open questions / risks
- The PSI / 14d-deviation bands are calibration-sensitive — too tight halts on noise, too loose trades through a real
  break; bound the false-halt rate on >=3 real deployments (GD-4) before default-ON. A false HALT is the SAFE failure
  (stops trading, requires a re-run), so bias toward sensitivity over tolerance.
- The concept-drift (P(Y|X)) half needs realized labels (ADWIN/DDM) — it CANNOT detect a regime break before labels
  arrive (TD-5, undecidable). Be explicit: the four deterministic axes catch INPUT/OUTPUT/schema/deviation drift; the
  label-fed half catches concept drift one cycle late; the pre-label residual is a T3 conformal advisory, never a guarantee.
- This gate gates LIVE TRADING — the live lane is W-10-tiered (the interlock logic + the SSB rows are Tier-1 in-process
  over logged data; the actual live-trading resume is the Tier-2 deployment, a separately-gated checkbox). No live
  resume ships until the fixture interlock is proven.

## Next
H-29 closes CLUSTER G and the Track-3 trust spine. With H-17 (N), H-18/H-19 (selection bias), H-20/H-21/H-22 (leakage
+ perturbation), H-23/H-24/H-25 (reproducibility + parity + anti-fabrication), H-26 (dynamic value verification),
H-27/H-28 (verify-the-verifiers + safe self-mod), and H-29 (deployment drift), the autonomously-discovered/LLM-written
trading strategy is trustworthy under deterministic ELAI gates — the North Star (PART 0). The irreducible residue
(PART 5) is named and routed to human Layer-3 throughout (TD-5).

End of H-29.

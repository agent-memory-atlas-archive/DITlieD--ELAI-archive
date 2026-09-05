# H-19: Reusable-holdout query budget (T-3) — bound adaptive OOS reuse

STATUS: PLANNED (TRACK 3 TRUST-HARDENING; WAVE 3 — deployment-trust half; in-process BUILD on SSB + rules engine; depends H-17)

Objective: nothing in ELAI bounds how many times the search loop queries the out-of-sample (OOS) holdout. An
autonomous searcher that repeatedly tunes against the same holdout is an unlimited adaptive analyst — the holdout
stops being held out, and OOS Sharpe inherits the same selection bias the in-sample number had (the adaptive-
overfitting residual TB-1 leaves after H-18). H-19 builds a differentially-private fresh-holdout budget
(Thresholdout / the Ladder) on the OOS set: a LOCKED query-budget counter on the SSB; each OOS query spends budget;
after the budget is exhausted, further OOS queries are REFUSED (budget_exhausted) until a fresh holdout is
provisioned. The off-agent artifact is the holdout-query ledger — the agent cannot mine the holdout past its budget.

Estimated effort: ~1 session (M, ~6 h). In-process; the DP noise math is exact, "enough budget" is a LOCKED policy
  constant (W-10 Tier-1 whole; no sidecar needed — Thresholdout noise is closed-form).
Pre-requisites: H-17 (the SSB append-only counter pattern + register_trial discipline — the budget counter mirrors
  it); the rules-engine MetricGate predicate SHIPPED; the mutable=0 tier SHIPPED.
Parallel-safe with: H-18 (disjoint — H-18 deflates the in-sample number, H-19 bounds the OOS holdout), H-20/H-21.
Source: design T-3 + PART 0 (TB-1) + PART 5 (residue); _grounding/H-TRUST-GROUNDING-MAP.md NN-3 + TS-1 + TD-1/2/5. Doctrine: H-0_INDEX.md.

## Context

H-18 deflates the in-sample Sharpe by N; H-19 closes the remaining adaptive-overfitting hole: the OOS holdout.
Dwork et al.'s reusable holdout / Thresholdout and Blum-Hardt's "The Ladder" give a differentially-private way to
answer a BOUNDED number of adaptive holdout queries before the holdout's validity degrades — and to REFUSE further
queries once the budget is spent. This is partial-deterministic (TD-5): the noise math is exact, but "enough budget"
is a LOCKED policy constant (the analyst's tolerated error). H-19 instantiates the budget counter as a LOCKED
append-only SSB counter (mirroring H-17's discipline — no agent reset) and a rules-engine MetricGate that refuses
OOS access past the budget. The honest residue (PART 5): this bounds adaptive reuse; it does not make OOS infinite.

## Existing state (B/C/D/E/F/G coverage)
- SSB append-only counter discipline (H-17 strategy_trials + register_trial) — SHIPPED-by-H-17. The budget counter
  follows the same no-reset/no-delete pattern (GD-1; reuse the discipline, not a new persistence engine).
- rules-engine MetricGate + mutable=0 tier (rule_engine.rs:98; schema.rs:46/lifecycle.rs:871) — SHIPPED. The refusal
  is a LOCKED MetricGate (budget_remaining <= 0 -> Block).
- reusable_holdout / Thresholdout — ABSENT (NN-3). The pure-BUILD surface.

## Reads (mandatory before coding)
- _grounding/H-TRUST-GROUNDING-MAP.md: NN-3, TS-1, TD-1, TD-2, TD-5, the H-19 symbol table.
- design T-3 + PART 0 (TB-1) + PART 5; H-17_trial_ledger_keystone.md (the append-only-counter contract to mirror).
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\trial_ledger.rs (H-17 — the no-reset counter discipline)
- F:\.ELAI_workflow\.elai_cc\crates\rules\src\rule_engine.rs:98 (MetricGate)
- CLAUDE.md HARD RULES: blake3, no .unwrap()/.expect(); ClockProvider; TD-1/2/5. Paper-ingest Dwork Thresholdout + Blum-Hardt Ladder before coding the noise.

## Tasks (WAVE 3 — in-process BUILD)

1. **Holdout-query ledger + LOCKED budget counter (~70 LOC).** A new append-only SSB-backed counter keyed by
   (holdout_id, objective): {queries_spent, budget_total (LOCKED), provisioned_at_ms}. No agent reset/delete; only
   spend (saturating) + provision-fresh (a privileged, gate-manifest-frozen operation). Mirrors H-17's discipline.

2. **Thresholdout-style DP OOS query (~80 LOC).** Each OOS query returns the holdout statistic with Laplace/Gaussian
   noise calibrated to the budget (closed-form; ELAI_SEED-seeded so the noise is reproducible, GD-2). The query spends
   one unit; when queries_spent >= budget_total, the query REFUSES with budget_exhausted (TD-1 fail-closed) until a
   fresh holdout is provisioned.

3. **Rules-engine refusal gate (~30 LOC).** A LOCKED MetricGate (mutable=0, domain finance): budget_remaining(holdout_id) <= 0
   -> Block(budget_exhausted). The budget_total + the noise scale are LOCKED constants (TD-2).

4. **Off-agent artifact (~20 LOC).** Emit `holdout_query_spent` / `holdout_budget_exhausted` (Pattern-2 single-variant
   with Kind; baseline+1 from LIVE 207/484, re-read at edit + bump e45:294). The ledger IS the off-agent artifact;
   spend is recomputable from the journal (GD-2).

5. **Freeze (TD-2).** Add the budget constants + the provision-fresh operation to gate-manifest.txt (only a privileged,
   human/frozen path provisions a fresh holdout — the agent cannot self-provision to reset its budget).

6. **Decision doc workflow/reasoning/H-19-decisions.md (~80 LOC).** Records: (a) the Thresholdout/Ladder grounding +
   the LOCKED budget_total + noise scale; (b) why provision-fresh is human/frozen-only (the TD-1 reset attack); (c)
   the honest residue (PART 5: this BOUNDS adaptive reuse, it does not make OOS infinite — TD-5).

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/holdout_budget.rs::query_holdout (the Thresholdout DP query + spend)
  - crates/orchestrator/src/holdout_budget.rs::budget_remaining (the read the gate refuses on)
  - crates/orchestrator/src/ssb.rs::SsbActionType::HoldoutBudgetEvent (Pattern-2: QuerySpent / Exhausted / Provisioned; baseline+1)
  - crates/rules/seed/finance/holdout_budget.toml (the mutable=0 LOCKED refusal gate)
consumes_symbols:
  - crates/orchestrator/src/trial_ledger.rs (H-17 — the no-reset append-only-counter discipline; REUSE, GD-1)
  - crates/rules/src/rule_engine.rs::CompiledPredicate (MetricGate — REUSE)
  - crates/core (ClockProvider; ELAI_SEED for the DP noise)
reachable_from:
  - the OOS-evaluation path (every holdout query routes through query_holdout) [PRODUCTION choke-point]
  - query_holdout -> budget_remaining -> Block(budget_exhausted) when spent [the refusal]
  - query_holdout -> HoldoutBudgetEvent SSB row [off-agent artifact]
stub_ban:
  - NO agent-facing budget reset / fresh-holdout provision (provision is frozen/human-only — TD-1/TD-2)
  - NO unbounded OOS query (missing budget = refuse, never SKIP — W-1/R1)
  - NO re-implementation of the H-17 counter or the rules engine (GD-1 P0)
  - DP noise seeded by ELAI_SEED (reproducible, GD-2); blake3 / no .unwrap() in prod; ClockProvider not SystemTime
frontend_pair:
  a holdout-budget gauge in the trust cockpit (remaining/total per holdout). cross_lang_diff PASS. Optional; W-6 via
  the budget_exhausted refusal visible in the dashboard.
observable_effect:
  trigger: "cd .elai_cc && cargo run -p elai-cli -- <run a fixture search that queries one holdout budget_total+1 times, NO env override>"
  expect_ssb: "budget_total HoldoutBudgetEvent(QuerySpent) rows then one Exhausted row, spend recomputable from the journal (GD-2)"
  expect_artifact: "the (budget_total+1)-th OOS query is REFUSED with budget_exhausted; a fresh holdout can only be
    provisioned via the frozen path (an agent-issued reset does not compile / is rejected)"

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/H-plan/H-19_reusable_holdout_budget.md --run-trigger
rg "query_holdout|budget_remaining" .elai_cc/crates/orchestrator/src --glob '!*test*'   # the choke-point exists in prod
rg -n "fn (reset|provision)_budget" .elai_cc/crates/orchestrator/src/holdout_budget.rs  # provision is frozen/privileged only; no agent reset
rg -c "blake2|sha2" .elai_cc/crates/orchestrator/src/holdout_budget.rs                   # MUST be 0
test -f workflow/reasoning/H-19-decisions.md
```

## ACCEPTANCE
- [ ] W-1 done-gate: --run-trigger PASS (budget_total queries spend, the next is REFUSED, Exhausted row landed). cargo-green != DONE.
- [ ] W-2 default-ON: the budget runs with no env override; kill-switch only.
- [ ] W-3 production entrypoint: query_holdout fires from the live OOS path, not a #[cfg(test)] caller.
- [ ] TD-1: provision-fresh is frozen/human-only; no agent reset. TD-2: budget + noise LOCKED. TD-5: honest residue documented.
- [ ] GD-1: H-17 counter + rules engine REUSED. GD-2: spend re-computes byte-identically under ELAI_SEED.
- [ ] cargo check --workspace GREEN; auditor PASS. No new dependency (closed-form noise; no scipy needed).

## WIRING DONE-GATE (clause by clause)
- W-1: --run-trigger is ACCEPTANCE item 1; query_holdout (produced) consumed on the live OOS path (W-8) reached by OOS eval (W-3).
- W-2: no override; kill-switch only. W-3: live OOS path; not test-only.
- W-4: frontend_pair = budget gauge; cross_lang_diff PASS. W-5: runnable fixture -> real rows; Tier-1 in-process.
- W-6: budget_exhausted refusal visible in dashboard. W-7: H-17 counter + rule_engine.rs:98 grep-confirmed; Thresholdout absent (NN-3).
- W-8: query_holdout/budget_remaining (produced) consumed by the OOS path + the gate. W-9: +HoldoutBudgetEvent baseline+1; bump e45:294.
- W-10: Tier-1 in-process (closed-form noise); no sidecar. TD-1/2/5: frozen, locked, honest-residue.

## Tests to add
- crates/contracts/tests/h19_holdout_budget_contract.rs (~140 LOC): (a) budget_total queries succeed; (b) the next
  refuses with budget_exhausted; (c) no agent reset method compiles; (d) provision-fresh only via the frozen path;
  (e) DP noise is ELAI_SEED-reproducible; (f) budget + noise scale are mutable=0.
- crates/contracts/tests/fixtures/holdout/ (a fixture exceeding one budget).

## Files touched
- CREATE: crates/orchestrator/src/holdout_budget.rs (~150 LOC: query_holdout, budget_remaining, the DP noise)
- CREATE: crates/rules/seed/finance/holdout_budget.toml (the mutable=0 LOCKED refusal gate)
- MODIFY: crates/orchestrator/src/ssb.rs (+HoldoutBudgetEvent Pattern-2) + action_type_lock.rs + e45 :294
- MODIFY: the OOS-evaluation entrypoint (~20 LOC: route OOS queries through query_holdout)
- MODIFY: .claude/gate-manifest.txt (freeze budget constants + provision-fresh)
- CREATE: crates/contracts/tests/h19_holdout_budget_contract.rs (~140 LOC)
- CREATE: workflow/reasoning/H-19-decisions.md (~80 LOC)

## Citations / Grounding
- design T-3; Dwork et al. reusable holdout / Thresholdout; Blum & Hardt "The Ladder". Paper-ingest both before
  coding the noise calibration; cite quote+page for the budget/error tradeoff.
- ELAI ground truth (NN-3/TS-1, grep-confirmed 2026-06-04): reusable_holdout/Thresholdout ABSENT; H-17 counter +
  rule_engine.rs:98 MetricGate present.
- CLAUDE.md HARD RULES: blake3, no .unwrap()/.expect(), ClockProvider; TD-1/2/5.

## Open questions / risks
- The DP budget_total is a policy constant — too small starves legitimate validation, too large under-bounds reuse.
  Set conservatively, LOCKED, and surface the remaining budget so the operator sees the tradeoff (GD-4-style FP bound).
- "Provision a fresh holdout" must consume genuinely fresh data (a NEW time window), not re-label the same series —
  the provision path must assert the new holdout's data_window does not overlap the spent one (the subtle reset attack).

## Next
H-20 (leakage taint COMPOSITE) opens CLUSTER B (the trading-specific killer). H-13/H-29 deployment trust composes
H-19's budget into the resume gate. H-18 + H-19 together are the full selection-bias defense (in-sample + OOS).

End of H-19.

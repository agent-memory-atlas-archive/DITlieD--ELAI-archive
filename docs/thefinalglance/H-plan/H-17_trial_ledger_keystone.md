# H-17: Persisted untamperable trial ledger — the selection-bias keystone (T-1)

STATUS: PLANNED (TRACK 3 TRUST-HARDENING; WAVE 3; in-process BUILD on the shipped SSB WAL; the keystone — H-18/H-22/H-25/H-29 depend on it)

Objective: an autonomous system that OPTIMISES a financial metric ships overfit/leaked/fabricated "edge" EVEN IF
EVERY AGENT IS PERFECTLY HONEST — because a search loop is an unbounded multiple-testing machine (the maximum of N
noisy Sharpe ratios is upward-biased; an in-sample Sharpe > 1.0 appears after ~7 pure-noise trials; design PART 0,
Bailey & Lopez de Prado). The single deterministic primitive that closes this is a persisted, append-only,
agent-unresettable cumulative trial count N, scoped per (data_window, objective), recorded through ONE choke-point.
H-17 builds `strategy_trials` (a new table on the shipped SSB WAL) + a single `register_trial()` choke-point on the
backtest/strategy-eval path + a read API the promotion gate (H-18) deflates by. Without N, every reported Sharpe is
selection-inflated and every downstream trust gate is gameable (TD-1). This is the keystone; nothing in Track 3 is
trustworthy without it.

Estimated effort: ~1 session (S-M, ~4-6 h). Pure in-process Rust on the SSB; no network/sidecar at gate time (W-10 Tier-1 whole).
Pre-requisites: SSB append-only WAL SHIPPED (phase-3B; ssb.rs single-writer, no UPDATE/DELETE, debug update_hook
  panic); gate-manifest + gate_freeze_check.py SHIPPED (.claude/gate-manifest.txt, R8); the rules-engine
  mutable=0 tier SHIPPED (crates/rules/src/schema.rs:46 mutable col, lifecycle.rs:871 mutable=0 path, per F-1).
Parallel-safe with: nothing in Track 3 that READS N (H-18/H-22/H-25/H-29 strictly follow). Disjoint from Track 1/2.
Source: D:\.REGRET_workflow\memory-bank\plans\elai-trust-hardening-design.md T-1 + PART 0 (TB-1) + PART 1 (D1/D2);
  _grounding/H-TRUST-GROUNDING-MAP.md TS-1 + NN-1 + TD-1 + TD-2. Doctrine: H-0_INDEX.md (W-1..10 + GD-1..5 + HD-1..5 + TD-1..5).

## Context

The threat is TB-1 SELECTION BIAS UNDER SEARCH (design PART 0): no penalty tied to the cumulative trial count N
means every reported Sharpe is selection-inflated. This is the #1 killer and the #1 net-new gap. ELAI already has
the WAL substrate (the untamperable-counter foundation) but NO persisted, monotonic, search-scoped, un-resettable N.
H-17 supplies exactly that — and ONLY that (the promotion math is H-18). The honest deliverable is a counter the
self-mod/search agent cannot read-around, reset, or forge: it lives in the gate-manifest LOCKED set (TD-2) and is
written through one choke-point every backtest/eval MUST pass through. A reported edge whose config has zero
matching trial rows is a HARD FAIL at the promotion gate (TD-1, trial_count_missing) — missing evidence = FAIL,
never SKIP (W-1/R1).

## Existing state (B/C/D/E/F/G coverage)
- SSB append-only WAL (crates/orchestrator/src/ssb.rs, phase-3B) — SHIPPED. Single-writer, no UPDATE/DELETE
  methods exposed, debug-build update_hook panic, ClockProvider-stamped created_at, ELAI_SEED-deterministic db
  dumps. H-17 adds a `strategy_trials` table + a `register_trial()` writer ON this substrate (GD-1; do NOT build a
  new persistence engine).
- gate_freeze_check.py + .claude/gate-manifest.txt (R8) — SHIPPED. The freeze mechanism the ledger + register fn
  are added to (TD-2): a run that mutates its own ledger code is BLOCK.
- rules-engine mutable=0 tier (crates/rules/src/schema.rs:46 mutable column default 1; lifecycle.rs:871 tested
  mutable=0 path, per F-1) — SHIPPED. The promotion gate H-18 is seeded mutable=0; H-17's table is referenced by it.
- LockedKeysAuditor + LOCKED_KEYS_TOUCH SSB (crates/orchestrator/tests/locked_keys_observable_smoke.rs:52/80/95;
  AllowedConfigKey self_mod_layer1a.rs:203, per D-9) — SHIPPED. The pattern for guarding an agent-unresettable surface.
- register_trial / strategy_trials — ABSENT (NN-1, zero elai.map hits 2026-06-04). The pure-BUILD surface.

## Reads (mandatory before coding)
- _grounding/H-TRUST-GROUNDING-MAP.md: TS-1, NN-1, TD-1, TD-2, the TRADING-LANE TIERING block, the H-17 symbol table.
- D:\.REGRET_workflow\memory-bank\plans\elai-trust-hardening-design.md: PART 0 (TB-1), PART 1 (D1/D2), T-1, PART 6 (WAVE 1).
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\ssb.rs (the single-writer WAL write loop + the no-UPDATE/DELETE invariant + the SsbPayload pattern + SCHEMA_VERSION at :2909)
- F:\.ELAI_workflow\.elai_cc\crates\rules\src\schema.rs:46 + lifecycle.rs:871 (the mutable=0 tier the H-18 gate uses)
- F:\.ELAI_workflow\.elai_cc\.claude\gate-manifest.txt + .claude\tools\gate_freeze_check.py (the freeze set the ledger joins)
- CLAUDE.md HARD RULES: blake3 (config_hash via blake3, NEVER blake2/sha2), no .unwrap()/.expect() in prod; TD-1, TD-2.

## Tasks (WAVE 3 — in-process BUILD)

1. **`strategy_trials` table + append-only invariant (~60 LOC).** New SQLite table on the SSB db (or a dedicated
   sibling db under the same single-writer discipline): columns {trial_id (blake3 of the tuple), objective TEXT,
   data_window TEXT (start_ms..end_ms canonicalised), config_hash TEXT (blake3 of the strategy config), seed i64,
   created_at_ms (ClockProvider, NOT SystemTime), session_id}. NO UPDATE/DELETE method exposed; the debug
   update_hook panic invariant extends to it. Index on (objective, data_window) for the count read.

2. **`register_trial()` single choke-point (~50 LOC).** Every backtest / strategy eval MUST pass through it
   before it can report a metric. It computes the tuple, blake3-hashes config_hash + trial_id, appends one row,
   and emits the off-agent SSB row (Task 3). It is idempotent on (config_hash, data_window, seed) — re-registering
   the same exact trial does NOT double-count (the count measures DISTINCT trials), but a DIFFERENT config in the
   same search increments N. Effective-N clustering for correlated trials (ONC) is a count_effective() read that
   collapses near-duplicate configs (Task 4) — the raw count stays monotone.

3. **Off-agent SSB artifact (~30 LOC).** Emit `trial_registered` (Pattern-2 single-variant-with-Kind preferred;
   baseline+1 from the LIVE 207/484 — re-read at edit time, and bump e45_cross_language_drift_contract.rs:294's
   stale 482 in the same edit). Payload carries {objective, data_window, config_hash, seed, n_after} so the count
   is recomputable from the journal under ELAI_SEED (GD-2). This row IS the off-agent artifact the H-18 gate reads.

4. **Read API for the promotion gate (~40 LOC).** `trial_count(objective, data_window) -> u64` (raw N) and
   `effective_trial_count(objective, data_window) -> f64` (ONC-clustered). Both read the table, never an
   agent-supplied number. `trial_count_missing` is the sentinel H-18 fails closed on (zero matching rows for a
   reported config). No agent-facing delete/reset/update API exists — the only mutation is append via register_trial.

5. **Freeze (TD-2).** Add the `strategy_trials` schema + `register_trial` source file to .claude/gate-manifest.txt
   so gate_freeze_check.py BLOCKs any run that edits its own ledger. Register a wire-dark-allowlist row only if the
   ledger ships behind a flag (it should NOT — default-ON, W-2 kill-switch only).

6. **Decision doc workflow/reasoning/H-17-decisions.md (~90 LOC).** Records: (a) why N is the keystone (an honest
   searcher still overfits without it); (b) raw vs effective-N (ONC) and why both are persisted; (c) the choke-point
   is the ONLY write path and there is no reset/delete API by construction; (d) the live trading lane is W-10-tiered
   (the ledger + count are Tier-1 in-process; the actual backtest sidecar is Tier-2, H-18).

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/trial_ledger.rs::strategy_trials (new append-only table + schema)
  - crates/orchestrator/src/trial_ledger.rs::register_trial (the single choke-point writer)
  - crates/orchestrator/src/trial_ledger.rs::trial_count / effective_trial_count (the read API H-18 deflates by)
  - crates/orchestrator/src/ssb.rs::SsbActionType::TrialRegistered (off-agent artifact; baseline+1, Pattern-2 if possible)
consumes_symbols:
  - crates/orchestrator/src/ssb.rs (the single-writer WAL append + no-UPDATE/DELETE invariant — REUSE, GD-1)
  - crates/core (ClockProvider for created_at_ms; blake3 for config_hash/trial_id)
  - .claude/tools/gate_freeze_check.py + .claude/gate-manifest.txt (freeze the ledger — TD-2)
reachable_from:
  - the backtest / strategy-eval path (every eval routes through register_trial before reporting a metric) [PRODUCTION choke-point]
  - crates/orchestrator/src/ssb.rs write loop -> the TrialRegistered row [off-agent artifact]
  - H-18 DSR/PBO gate -> trial_count() [the consumer that fails closed on trial_count_missing]
stub_ban:
  - NO agent-facing delete/reset/update method on strategy_trials (append-only by construction — TD-1)
  - NO re-implementation of the SSB WAL or a second persistence engine (GD-1 P0 — extend the shipped single-writer)
  - NO default-off flag as the terminal state (W-2 kill-switch only; default ON)
  - NO agent-supplied trial count anywhere downstream — N is READ from the table, never passed in (TD-1)
  - blake3 only for config_hash/trial_id (never blake2/sha2); no .unwrap()/.expect() in prod
  - NO SystemTime — ClockProvider only (C-15 determinism), so ELAI_SEED runs are byte-identical (GD-2)
frontend_pair:
  N/A in isolation — N is surfaced by the trust cockpit (cross-ref H-18/H-29 dashboard tiles). Justification: H-17
  is the producer; the count is rendered alongside the DSR/PBO verdict it gates. A trial-count strip is optional.
observable_effect:
  trigger: "cd .elai_cc && cargo run -p elai-cli -- <run a fixture search loop that registers >=3 distinct strategy
    configs against the same (objective, data_window), with NO env override>"
  expect_ssb: "N TrialRegistered rows land (one per distinct config), trial_count(objective,data_window) returns the
    monotone count, and the count is recomputable byte-identically from the journal under ELAI_SEED (GD-2)"
  expect_artifact: "a strategy config with ZERO matching strategy_trials rows, fed to trial_count(), returns 0 ->
    the H-18 promotion gate fails closed with trial_count_missing (verified in the H-18 fixture); re-registering the
    SAME config does not double-count; a DIFFERENT config increments N"

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/H-plan/H-17_trial_ledger_keystone.md --run-trigger
rg "register_trial|strategy_trials" .elai_cc/crates/orchestrator/src --glob '!*test*'   # the choke-point + table now exist in prod
rg -n "fn (delete|reset|update)_trial|DELETE FROM strategy_trials|UPDATE strategy_trials" .elai_cc/crates   # MUST be 0 (append-only)
rg -c "blake2|sha2" .elai_cc/crates/orchestrator/src/trial_ledger.rs                     # MUST be 0
python .claude/tools/gate_freeze_check.py                                                # trial_ledger.rs is in the frozen manifest
test -f workflow/reasoning/H-17-decisions.md
```

## ACCEPTANCE
- [ ] W-1 done-gate: wiring_contract_check.py --run-trigger PASS (produces exist, consumes resolve, reachable_from
  reached by BFS, stub_ban clean, observable_effect RAN + >=3 TrialRegistered rows landed + trial_count monotone).
  cargo-green != DONE.
- [ ] W-2 default-ON: the ledger + register_trial run with NO env override; only a kill-switch disables.
- [ ] W-3 production entrypoint: register_trial fires from the live backtest/eval path, not a #[cfg(test)] caller.
- [ ] TD-1: N is READ from strategy_trials, never agent-supplied; a config with zero rows fails closed downstream.
- [ ] TD-2: strategy_trials + register_trial are in .claude/gate-manifest.txt; gate_freeze_check.py BLOCKs self-edits.
- [ ] GD-1 compose: the SSB WAL is REUSED, not re-implemented (grep shows one single-writer).
- [ ] GD-2 reproducible: the trial count + rows re-compute byte-identically from the journal under ELAI_SEED.
- [ ] cargo check --workspace GREEN; auditor PASS on -p elai-orchestrator. No new dependency.

## WIRING DONE-GATE (clause by clause)
- W-1: wiring_contract_check --run-trigger is ACCEPTANCE item 1; the produced register_trial is consumed on the
  live eval path (W-8) reached by the backtest loop (W-3).
- W-2: ledger runs with no override; kill-switch only.
- W-3: the live backtest/eval path calls register_trial; not test-only.
- W-4: N/A own pair (surfaced by H-18/H-29 cockpit); justified.
- W-5: observable_effect is a runnable fixture search loop -> real journal rows; pure in-process.
- W-6: dashboard E2E for N is H-18/H-29's; H-17 N/A justified.
- W-7: ssb.rs WAL / gate-manifest / mutable=0 tier grep-confirmed 2026-06-04 (TS-1/TS-2); register_trial absent (NN-1).
- W-8: register_trial (produced) is called on the live eval path; trial_count (produced) consumed by H-18.
- W-9: +TrialRegistered baseline+1 (Pattern-2 if possible); also bump e45 contract :294 (482->live) in the same edit.
- W-10: entirely Tier-1 in-process (table + count); the backtest sidecar is Tier-2 (H-18).
- GD-1/GD-2/TD-1/TD-2: reuse the WAL; N journal-reproducible; N agent-unresettable + frozen.

## Tests to add
- crates/contracts/tests/h17_trial_ledger_contract.rs (~150 LOC): (a) register_trial appends a row + emits
  TrialRegistered; (b) re-registering the same (config_hash,data_window,seed) does NOT increment trial_count;
  (c) a different config DOES increment; (d) no delete/reset/update method compiles (negative compile-or-absence
  test); (e) trial_count for an unseen config returns 0 (the trial_count_missing sentinel); (f) effective vs raw
  count under correlated configs; (g) ELAI_SEED=42 twice -> byte-identical trial rows.
- crates/contracts/tests/fixtures/trials/ (a fixture search loop registering 3 distinct configs + 1 duplicate).

## Files touched
- CREATE: crates/orchestrator/src/trial_ledger.rs (~200 LOC: table, register_trial, trial_count/effective_trial_count)
- MODIFY: crates/orchestrator/src/ssb.rs (+TrialRegistered action_type; SCHEMA bump from live baseline) + action_type_lock.rs (+ALL_VARIANTS) + e45_cross_language_drift_contract.rs:294 (482->live)
- MODIFY: the backtest/eval entrypoint (~20 LOC: route every eval through register_trial before reporting a metric)
- MODIFY: .claude/gate-manifest.txt (add trial_ledger.rs to the frozen set)
- CREATE: crates/contracts/tests/h17_trial_ledger_contract.rs (~150 LOC)
- CREATE: workflow/reasoning/H-17-decisions.md (~90 LOC)

## Citations / Grounding
- design T-1 + PART 0 (TB-1) + D2 ("Marcos' Third Law": N is the only gameable DSR input); Bailey & Lopez de Prado
  (Deflated Sharpe Ratio; N as the keystone input); ONC effective-N clustering for correlated trials.
- ELAI ground truth (TS-1/NN-1, grep-confirmed 2026-06-04): SSB single-writer WAL append-only (ssb.rs); mutable=0
  tier (schema.rs:46 / lifecycle.rs:871, per F-1); gate_freeze_check.py + gate-manifest (R8); register_trial/
  strategy_trials ABSENT (zero hits).
- CLAUDE.md HARD RULES: blake3, no .unwrap()/.expect() in prod, ClockProvider (no SystemTime); TD-1, TD-2.

## Open questions / risks
- SCOPING N: per (data_window, objective) is the design's scope; a search that silently varies the objective string
  could fragment N. The choke-point canonicalises objective + data_window before hashing — verify the canonicaliser
  is itself in the frozen set (else an agent reshapes the scope key to reset N). This is the subtle TD-1 attack.
- Effective-N (ONC) clustering is a policy constant (clustering threshold) — it is LOCKED, not agent-tunable, and a
  conservative default (raw N) is the fail-safe if clustering is uncertain.
- The ledger must outlive a session (the search spans sessions); confirm it is NOT a per-session db that resets — it
  is per (project, objective, data_window), persisted across sessions. This is the single most important design check.

## Next
H-18 (DSR/PBO/MinBTL promotion gate) is the first consumer: a LOCKED COMPOSITE that reads N from H-17 and blocks
promotion unless the Deflated Sharpe Ratio + PBO + MinBTL clear thresholds, failing closed on trial_count_missing.
H-22 (trading nulls), H-25 (result manifest), H-29 (drift FSM) all read N. H-17 is the hard prereq for all of them.

End of H-17.

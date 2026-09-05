# H-24: Sim-to-real parity gate (T-8) — research-path == live-path, replay == golden

STATUS: PLANNED (TRACK 3 TRUST-HARDENING; WAVE 3 CLUSTER D; in-process BUILD on the rules engine + the SHIPPED replay substrate)

Objective: nothing asserts that the research/backtest path is the SAME path that runs live, nor that a seeded replay
reproduces a frozen golden event log. A backtest that takes an `if backtest:` fast-path (a different fill model, a
duplicate router) produces an edge that does not exist live — the classic sim-real divergence. H-24 builds (A) a T1
AST_PATTERN that FAILS on mode-divergent branches (`if live:` / `if backtest:`, duplicate fill/router functions) in
any module on the execution path, plus a SCHEMA_REQUIRED manifest declaring the SINGLE shared entrypoint and its
swappable adapters; and (C) a T1 METRIC_GATE that replays the recorded event log under ELAI_SEED and asserts
blake3(replay) == blake3(golden), composing the SHIPPED replay substrate (replay_resume, RRA). Behavioral parity (B)
— a numeric reconciliation against a live/paper run — is a LATER Tier-2 METRIC_GATE (needs a live run + LOCKED
thresholds; DEFERRED, GD-5 honest). The off-agent artifacts are the AST findings + the replay-vs-golden hashes.

Estimated effort: ~1 session (M, ~6 h; A+C are pure-build, B deferred). In-process (Tier-1); B needs a live/paper run (Tier-2 deferred).
Pre-requisites: the rules-engine AST_PATTERN + SchemaRequired + MetricGate SHIPPED; replay_resume (ssb.rs:23502) +
  RRA replay (run_subcommand.rs:584) SHIPPED; exchange-core determinism discipline (integer money math, single-writer loop).
Parallel-safe with: H-23 (re-run equality — shares the replay substrate), H-25/H-26.
Source: design T-8 + PART 4 (R1-R5 sim-real); _grounding/H-TRUST-GROUNDING-MAP.md TS-5 + NN-8 + TD-5. Doctrine: H-0_INDEX.md.

## Context

The design's T-8 has three claims: (A) no mode-divergent branches on the execution path + a single declared
entrypoint with swappable adapters (LEAN-style live-vs-backtest reconciliation; exchange-core determinism: integer
money math, single-writer loop, banned wall-clock/unseeded RNG); (C) a seeded replay of the recorded event log
equals a frozen golden hash; (B) behavioral numeric parity (a later Tier-2 gate needing a live/paper run). A+C are
pure-build deterministic gates on the SHIPPED rules engine + replay substrate. B is honestly DEFERRED (it requires a
live/paper run + LOCKED reconciliation thresholds — GD-5: no external claim before it). The off-agent artifacts: the
AST findings (mode-divergent branches the agent did not author) + the replay-vs-golden hashes.

## Existing state (B/C/D/E/F/G coverage)
- rules-engine AST_PATTERN + SchemaRequired + MetricGate (predicate.rs:102; rule_engine.rs:98) — SHIPPED. The
  mode-divergent-branch ban + the single-entrypoint manifest + the replay-equality gate instantiate here (GD-1).
- replay_resume + RRA replay (crates/orchestrator/src/ssb.rs:23502 replay_resume; cli/run_subcommand.rs:584 RRA replay
  verify) — SHIPPED (G-2/phase-3H). The event-log replay substrate for the golden-hash gate.
- exchange-core determinism discipline (integer money math, single-writer SSB loop, ClockProvider, ELAI_SEED) — the
  banned-wall-clock/unseeded-RNG patterns already enforced by the entropy quarantine (C-15).
- a mode-divergent-branch AST ban + a replay==golden hash gate — ABSENT (NN-8). The pure-BUILD surface.

## Reads (mandatory before coding)
- _grounding/H-TRUST-GROUNDING-MAP.md: TS-5, NN-8, TD-5, the H-24 symbol table.
- design T-8 + PART 4; H-23_same_seed_rerun_hash_equality.md (the shared replay-equality discipline).
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\ssb.rs:23480-23540 (replay_resume) + cli/run_subcommand.rs:560-600 (RRA replay verify)
- F:\.ELAI_workflow\.elai_cc\crates\rules\src\predicate.rs:100-120 (AstPattern + SchemaRequired)
- CLAUDE.md HARD RULES: blake3 (replay/golden hash), no .unwrap()/.expect(), ClockProvider (banned wall-clock); TD-5; GD-5 (B deferred).

## Tasks (WAVE 3 — A+C build, B deferred)

1. **(A) Mode-divergent-branch AST_PATTERN (~50 LOC).** A T1 AstPattern that FAILS on `if live:` / `if backtest:` /
   `if mode == "live"` branches and duplicate fill/router/order functions in any module on the execution path. BLOCK
   at TESTING_FIRST — a mode-divergent fast-path is the sim-real divergence by construction.

2. **(A) SCHEMA_REQUIRED single-entrypoint manifest (~30 LOC).** A SchemaRequired predicate forcing the strategy to
   declare ONE shared execution entrypoint + its swappable adapters (data source, fill model, clock) — the live and
   backtest paths differ ONLY in the adapter, never the logic. Missing declaration = BLOCK.

3. **(C) Replay == golden METRIC_GATE (~80 LOC).** A MetricGate that replays the recorded event log under ELAI_SEED
   (via the SHIPPED replay_resume / RRA replay) and asserts blake3(replay) == blake3(golden), reporting the first
   mismatching event on divergence (DriftComparator pattern). A seeded replay that diverges from the golden hash FAILs
   with the first mismatch. Missing the golden = FAIL (W-1/R1).

4. **Off-agent artifact (~20 LOC).** Emit `sim_real_parity` (Pattern-2 with Kind: ModeDivergentBranch /
   ReplayGoldenMismatch / SingleEntrypointMissing; baseline+1 from LIVE 207/484, re-read + bump e45:294). The AST
   findings + the replay/golden hashes are the off-agent artifacts.

5. **(B) Behavioral parity — DEFERRED Tier-2 (documented only, ~0 LOC).** Record that numeric live-vs-backtest
   reconciliation is a later Tier-2 METRIC_GATE needing a live/paper run + LOCKED thresholds; ships a deterministic
   fixture stub now, the live reconciliation a separately-gated checkbox (GD-5 — no parity claim before it).

6. **Decision doc workflow/reasoning/H-24-decisions.md (~80 LOC).** Records: (a) A+C are deterministic pure-build,
   B is honestly deferred (GD-5); (b) the single-entrypoint-with-adapters pattern (LEAN/exchange-core); (c) the
   replay-vs-golden hash + first-mismatch reporting (composes replay_resume, restates none — GD-1).

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/sim_real_parity.rs::verify_replay_golden (the C replay==golden gate)
  - crates/rules/seed/repro/sim_real_parity.toml (the A mode-divergent-branch AST + single-entrypoint SchemaRequired, mutable=0)
  - crates/orchestrator/src/ssb.rs::SsbActionType::SimRealParity (Pattern-2 3-Kind; baseline+1)
consumes_symbols:
  - crates/orchestrator/src/ssb.rs::replay_resume + crates/cli/src/run_subcommand.rs (RRA replay) — REUSE, GD-1
  - crates/orchestrator/src/determinism.rs::DriftComparator (first-mismatch reporting — REUSE)
  - crates/rules/src/predicate.rs::{AstPattern, SchemaRequired, MetricGate} (REUSE)
reachable_from:
  - the PreToolUse path (A: the AST ban on execution-path modules) + the pre-promotion path (C: replay==golden) [PRODUCTION gates]
  - verify_replay_golden -> replay_resume -> SimRealParity SSB row [off-agent artifact]
stub_ban:
  - NO mode-divergent fast-path on the execution path (the AST ban BLOCKs — A)
  - NO PASS without the golden replay (missing golden = FAIL, never SKIP — C, W-1/R1)
  - NO behavioral-parity (B) claim without a live run (DEFERRED Tier-2, GD-5 — do NOT advertise B as live)
  - NO re-implementation of replay_resume or DriftComparator (GD-1 P0); blake3 only; no .unwrap()/.expect() in prod
frontend_pair:
  the mode-divergent finding + the replay/golden hashes in the trust cockpit. cross_lang_diff PASS. W-6: a blocked fast-path visible in the dashboard.
observable_effect:
  trigger: "cd .elai_cc && cargo run -p elai-cli -- <feed a fixture strategy with an `if backtest:` fill-model fast-path + run replay==golden on a divergent event log, NO env override>"
  expect_ssb: "a SimRealParity(ModeDivergentBranch) row + a SimRealParity(ReplayGoldenMismatch) row land with the first mismatching event, recomputable from the journal (GD-2 same-env)"
  expect_artifact: "the `if backtest:` fast-path is BLOCKED at build; a seeded replay that diverges from the golden hash FAILs with the first mismatch; a single-entrypoint strategy with a clean replay PASSES"

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/H-plan/H-24_sim_to_real_parity.md --run-trigger
rg "verify_replay_golden|replay_resume" .elai_cc/crates/orchestrator/src --glob '!*test*'   # the gate reuses the replay substrate
rg -n "sim_real_parity|if live|if backtest" .elai_cc/crates/rules/seed/repro/sim_real_parity.toml   # the AST ban present
rg -c "blake2|sha2" .elai_cc/crates/orchestrator/src/sim_real_parity.rs                      # MUST be 0
test -f workflow/reasoning/H-24-decisions.md
```

## ACCEPTANCE
- [ ] W-1 done-gate: --run-trigger PASS (an `if backtest:` fast-path BLOCKED; a divergent replay FAILs the golden hash). cargo-green != DONE.
- [ ] W-2 default-ON: A+C run with no env override; kill-switch only.
- [ ] W-3 production entrypoint: the AST ban fires at PreToolUse + verify_replay_golden at pre-promotion, not #[cfg(test)] callers.
- [ ] GD-5: B (behavioral parity) is DEFERRED Tier-2; not advertised as live. TD-5: same-env replay scope honest.
- [ ] GD-1: replay_resume + DriftComparator + rules engine REUSED. GD-2: the replay hash re-computes under ELAI_SEED (same-env).
- [ ] cargo check --workspace GREEN; auditor PASS. No new dependency.

## WIRING DONE-GATE (clause by clause)
- W-1: --run-trigger is ACCEPTANCE item 1; verify_replay_golden (produced) consumed on pre-promotion (W-8); the AST ban on PreToolUse (W-3).
- W-2: A+C no override; kill-switch only. W-3: live gate paths; not test-only.
- W-4: frontend_pair = parity drawer; cross_lang_diff PASS. W-5: runnable fixture -> real rows; Tier-1 in-process.
- W-6: blocked fast-path visible in dashboard. W-7: replay_resume ssb.rs:23502 + run_subcommand.rs:584 + predicate.rs grep-confirmed (TS-5); parity gates absent (NN-8).
- W-8: verify_replay_golden (produced) consumed on promotion; the AST seed consumed on PreToolUse.
- W-9: +SimRealParity Pattern-2 baseline+1; bump e45:294.
- W-10: A+C Tier-1 in-process; B (numeric reconciliation) DEFERRED Tier-2 (live run + locked thresholds). GD-5/TD-5 honest.

## Tests to add
- crates/contracts/tests/h24_sim_real_parity_contract.rs (~150 LOC): (a) `if backtest:` branch -> Block(ModeDivergentBranch);
  (b) duplicate fill function -> Block; (c) missing single-entrypoint manifest -> Block(SingleEntrypointMissing);
  (d) divergent replay -> ReplayGoldenMismatch with first-mismatch event; (e) clean single-entrypoint + matching replay -> Allow;
  (f) B is documented-deferred (no live-parity assertion compiled as a done-gate); (g) ELAI_SEED replay -> identical hash (same-env).
- crates/contracts/tests/fixtures/sim_real/ (an `if backtest:` strategy + a single-entrypoint one + a divergent event log).

## Files touched
- CREATE: crates/orchestrator/src/sim_real_parity.rs (~80 LOC: verify_replay_golden)
- CREATE: crates/rules/seed/repro/sim_real_parity.toml (the A AST ban + single-entrypoint SchemaRequired, mutable=0)
- MODIFY: crates/orchestrator/src/ssb.rs (+SimRealParity Pattern-2) + action_type_lock.rs + e45 :294
- MODIFY: the PreToolUse path (A AST ban) + the pre-promotion path (C replay==golden) (~20 LOC)
- CREATE: crates/contracts/tests/h24_sim_real_parity_contract.rs (~150 LOC)
- CREATE: workflow/reasoning/H-24-decisions.md (~80 LOC)

## Citations / Grounding
- design T-8 + PART 4; LEAN-style live-vs-backtest reconciliation; exchange-core determinism (integer money math,
  single-writer loop, banned wall-clock/unseeded RNG). WebSearch the LEAN reconciliation + exchange-core determinism patterns.
- ELAI ground truth (TS-5/NN-8, grep-confirmed 2026-06-04): replay_resume ssb.rs:23502 + RRA replay run_subcommand.rs:584;
  predicate.rs:102 AstPattern/SchemaRequired; mode-divergent ban + replay==golden ABSENT.
- CLAUDE.md HARD RULES: blake3, no .unwrap()/.expect(), ClockProvider (banned wall-clock); TD-5; GD-5 (B deferred).

## Open questions / risks
- "Execution path" detection: the AST ban must scope to modules actually on the order/fill path (not every `if`
  branch). The single-entrypoint manifest defines the path; the ban scopes to its transitive imports. Verify the scope.
- Golden event log provenance: the golden must itself be a trusted, frozen artifact (in the gate-manifest), else an
  agent regenerates a self-favorable golden. The golden hash is pinned; H-28 reward-tampering guards regeneration.
- B (behavioral parity) is the hardest + is deferred — be explicit that A+C prove STRUCTURAL parity (same code path,
  reproducible replay), NOT numeric live-vs-backtest agreement; the latter needs a live run and stays a Tier-2 checkbox.

## Next
H-25 (result manifest + re-derivation) binds the reported EMPIRICAL number to its provenance, extending F-4 from
"test passed" to "result honest". H-23 + H-24 + H-25 together make reproducibility + parity + anti-fabrication a
deterministic, off-agent gate set.

End of H-24.

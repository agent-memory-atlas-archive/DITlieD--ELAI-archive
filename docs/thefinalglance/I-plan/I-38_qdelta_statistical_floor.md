# I-38: QDELTA + statistical floor — defective-diff rate, bootstrap CI, mechanical methodology enforcement

- Track 18 — DETERMINISTIC-OVER-PROMPT HARDENING | lever PROVABLE | grounding STRONG | nature BUILD | effort M | WAVE 1 | binds ID-5

- Premise: ELAI ships SAFEDELTA (G-6) for the security quality delta. The quality sibling — QDELTA, the
  defective-diff rate with gates ON vs OFF and bootstrap CIs — is absent (IN-9). More critically, the
  statistical floor that would make QDELTA numbers trustworthy is nowhere enforced: runs with 2 seeds, unpaired
  designs, and no contamination check produce headline numbers that cannot be compared or reproduced. I-38 builds
  both: the QDELTA metric and the harness that MECHANICALLY refuses under-powered runs.

## Steal/build
QDELTA definition: defective diffs reaching a reviewer per 100 tasks, measured gates-ON vs gates-OFF, where
defective = held-out test failures + post-merge surviving mutants (from I-20) + regression count, with
bootstrap CIs clustered by repo. ABLATION FRAMING ONLY (GD-3) — never a leaderboard rank. The SWE-bench
Verified framing is explicitly DECLINED as a headline vehicle (landmine #11; leaderboard-rank framing is the
excluded alternative).

THE STATISTICAL FLOOR AS POLICY — enforced mechanically by the bench harness, not by culture:
  - paired-task designs (every gates-ON run has a gates-OFF counterpart on the same tasks)
  - >= 5 seeds per condition with SEM reported alongside the point estimate
  - pass^k for multi-step reliability claims (k logged, not assumed 1)
  - $/task reported next to accuracy in every headline
  - date-stamped task cuts + D-20 memorization probe surfaced as the contamination check field

A run that VIOLATES the floor (e.g., 3 seeds, unpaired) is REFUSED by the harness itself at run start —
the harness reads the config, checks the floor invariants, and exits nonzero before any compute is spent.
This is the mechanization: the floor is not advisory.

The D-20 memorization probe is surfaced into the methodology: the QDELTA run config has a contamination_probe
field; the harness runs D-20 against the task cut before accepting the run as clean.

## Existing ground truth
- IN-9 (grounding map): NO QDELTA harness — G-6 SAFEDELTA harness shape exists; quality sibling absent — confirms BUILD
- PLAN-asserted: G-6 safedelta_runner (the harness SHAPE — I-38 mirrors, does not fork); executor re-greps per W-7
- PLAN-asserted: D-6 / E-54 held-out corpus + 13-gram filter; executor re-greps per W-7
- PLAN-asserted: D-20 memorization probe (surfaced into the methodology as contamination_probe); executor re-greps per W-7
- PLAN-asserted: E-49 metrics infrastructure; executor re-greps per W-7
- I-20 dependency: the mutant leg (surviving mutants per task) is produced by I-20's sampled scorer
- Landmine #11 (I-0): SWE-bench Verified retired; leaderboard-rank framing declined

## REVISED scope (W-10 tiered)
SHIP NOW (Tier-1):
- crates/cli/src/bench_qdelta.rs (~200 LOC): QdeltaConfig (run config with floor invariants), run_qdelta(),
  floor_check() (mechanical pre-run validation — exits nonzero on floor violation), bootstrap_ci()
- methodology doc auto-generated from QdeltaConfig (machine-readable config -> human-readable methodology)
- QdeltaMeasured SSB row (flat beacon, SAFEDELTA-shaped payload: ON/OFF delta, CI, seed_count, paired,
  $/task, contamination_probe result)
- Fixture: paired runs at >=5 seeds + an under-powered run (3 seeds, unpaired) that the harness REFUSES
- D-20 contamination probe field wired into QdeltaConfig
- QDELTA tile in the cockpit next to the SAFEDELTA tile (frontend_pair)
- decision doc: workflow/reasoning/I-38-decisions.md

DEFER (Tier-2 / gated):
- Full-corpus QDELTA run — RunPod checkbox (requires the full E-54 task corpus at production scale; the fixture tier is the Tier-1 deliverable)
- Cross-repo cluster analysis of CI width — needs >=3 repos of real task data (GD-4)

## Integration
- new crates/cli/src/bench_qdelta.rs (~200 LOC)
- seam to G-6 safedelta_runner: QdeltaConfig mirrors the SafedeltaConfig shape (compose, not fork — GD-1)
- seam to D-6/E-54 corpus: the same corpus and 13-gram filter; GD-1
- seam to D-20 probe: contamination_probe field calls the D-20 memorization probe function; GD-1
- seam to I-20 mutation scorer: surviving-mutants field populated from I-20 SSB rows; GD-1
- frontend_pair: QDELTA tile next to the SAFEDELTA tile in the cockpit (new Svelte tile); cross_lang_diff.py PASS (W-4)

## WIRING_CONTRACT
produces_symbols:
  - crates/cli/src/bench_qdelta.rs::QdeltaConfig (the run config + floor invariants)
  - crates/cli/src/bench_qdelta.rs::run_qdelta() (the harness entry point)
  - crates/cli/src/bench_qdelta.rs::floor_check() (the mechanical pre-run validator)
  - ssb action_type::QdeltaMeasured (flat beacon, safedelta-shaped payload) — baseline+1 from LIVE 221/532 (ssb.rs:2981 / action_type_lock.rs:601); re-read at edit (W-9)
  - methodology doc (auto-generated from QdeltaConfig at run time)
consumes_symbols:
  - G-6 safedelta_runner (the harness shape precedent; GD-1 — compose, not fork)
  - D-6/E-54 held-out corpus + 13-gram filter (the task set; GD-1)
  - D-20 memorization probe (the contamination_probe field target; GD-1)
  - I-20 sampled mutation scorer (the surviving-mutants leg; GD-1)
  - E-49 metrics infrastructure (the measurement plumbing; GD-1)
reachable_from:
  - `elai bench qdelta` CLI subcommand -> run_qdelta() -> QdeltaMeasured SSB row  [PRODUCTION bench surface]
  - floor_check() called at run_qdelta() entry -> exits nonzero if floor violated  [PRODUCTION harness surface]
  - test callers in crates/contracts/tests/ do NOT satisfy R7
stub_ban:
  - NO unpaired runs emitting a QDELTA headline (floor_check() must reject; an unpaired run that reaches QdeltaMeasured is a harness bug)
  - NO under-seeded runs emitting a headline (floor_check() rejects < 5 seeds)
  - NO leaderboard-rank framing (ablation framing only per GD-3; the methodology doc's first section states this explicitly)
  - NO fabricated CI values (bootstrap_ci() is recomputable from the same seed under ELAI_SEED — GD-2)
  - NO .unwrap()/.expect() on production paths; NO blake2/sha2 (blake3 only)
frontend_pair:
  QDELTA tile next to the SAFEDELTA tile in the cockpit — new Svelte tile + Tauri read; cross_lang_diff.py PASS required (W-4).
observable_effect:
  trigger: "elai bench qdelta --config qdelta_fixture.toml --seed 42 (fixture tier: paired, 5 seeds) AND elai bench qdelta --config qdelta_underpower.toml (3 seeds, unpaired)"
  expect_ssb: "QdeltaMeasured row from the valid run: ON/OFF delta, bootstrap CI, seed_count=5, paired=true, $/task, contamination_probe result — byte-recomputable from same seed (GD-2); the under-powered run emits NO QdeltaMeasured row (floor_check exits nonzero before compute begins)"
  expect_artifact: "the under-powered run exits nonzero with a floor_check error message naming the violation (R2 — the harness refuses, not advises); the methodology doc regenerates deterministically and names the D-20 contamination result; the QDELTA tile renders in the cockpit (cross_lang_diff.py PASS)"

## Grounding
- IN-9 (grounding map): QDELTA absent, SAFEDELTA shape exists — confirms BUILD scope
- QAfromfable §6 (I-0 source synthesis): ID-5 doctrine, the local-baseline-before-paper-threshold rule
- ID-5 (I-0 doctrine): I-38 is explicitly the instrument of ID-5's enforcement
- GD-3 (I-0 doctrine): ablation framing only; no leaderboard-rank framing
- Landmine #11 (I-0): SWE-bench Verified retired; leaderboard framing DECLINED
- Verification-economics review: budgeted mutation with CIs, pass^k, $/task reporting
- D-20 memorization probe (PLAN-asserted; executor re-greps per W-7): the contamination check surface
- SSB: +QdeltaMeasured (flat beacon, safedelta shape) — baseline+1 from LIVE 221/532 as of 2026-06-10; re-read at edit (W-9)

## COMPOSES-WITH
- G-6 safedelta_runner — the harness shape precedent; do NOT fork (mirror the config structure, share the corpus path)
- D-6/E-54 held-out corpus — the task set; do NOT re-implement corpus loading
- D-20 memorization probe — the contamination check; compose the function call, do NOT re-implement the probe
- I-20 mutation scorer — the surviving-mutants leg; consume the SSB rows, do NOT re-implement mutation scoring
- E-49 metrics infrastructure — the measurement plumbing; compose, do NOT re-implement

End of I-38.

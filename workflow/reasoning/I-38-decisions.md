I-38 QDELTA + statistical floor — decisions (STEP 7, final step)

PLAN: I-38 — QDELTA defective-diff rate, bootstrap CI, mechanical methodology enforcement.
Track 18 — DETERMINISTIC-OVER-PROMPT HARDENING | lever PROVABLE | grounding STRONG | nature BUILD | effort M | WAVE 1 | binds ID-5.

This decision doc covers the full 7-step implementation. Steps 1-6 landed code + tests;
step 7 is this document. All facts below are verified against the live worktree source at
the HEAD of branch conductor/I-38-EXECUTE-7-00c75531 (re-verified by the 00c75531 worker:
all line references match, gate_freeze_check PASS 50 files, production_caller_check PASS
0 dark fn / 4 wired).


SCOPE DECISION: Tier-1 fixture harness (SHIP NOW), full-corpus RunPod deferred

The plan's REVISED scope (W-10 tiered) split the deliverable into:
  SHIP NOW (Tier-1): fixture-driven QDELTA math/gating proof, floor_check, bootstrap CI,
    methodology doc, QdeltaMeasured SSB row, D-20 contamination probe field, frontend_pair.
  DEFER (Tier-2): full-corpus QDELTA run (RunPod checkbox, needs E-54 at production scale),
    cross-repo cluster analysis of CI width (needs >=3 repos, GD-4).

Decision: ship Tier-1 only. All live-model/quantization legs carry `runpod_pending = true`
(D-026 deferred). The fixture tier proves the math, the floor enforcement, and the SSB
wiring without spending GPU compute. This mirrors the G-6 safedelta_runner Tier-1 shape
(GD-1: compose, not fork).


KEY DECISION 1: Mechanical floor enforcement (not advisory)

floor_check() is called at run_inner() entry, BEFORE any compute (bootstrap CI, SSB emit,
JSON report). It returns Err(FloorViolation) if seed_count < 5 OR paired == false. The
caller (run()) prints "QDELTA ABORT: ..." to stderr and calls std::process::exit(1).

This is the mechanization the plan demands: the floor is NOT advisory. An under-powered
run (3 seeds, unpaired) exits nonzero before any compute is spent. No QdeltaMeasured SSB
row is emitted for a refused run (verified by contract test arm (b): zero rows in a fresh
tempdir SSB after a refused run).

Rationale: the plan states "A run that VIOLATES the floor is REFUSED by the harness itself
at run start." Advisory floors are bypassable; a mechanical exit is not. This is the R2
observable effect: "the harness refuses, not advises."


KEY DECISION 2: Bootstrap CI via seeded xorshift64 (GD-2 recomputable)

bootstrap_ci() uses xorshift64 (pure std, no dep) seeded by the --seed CLI arg. The same
seed + same fixture produces byte-identical CI bounds. This satisfies GD-2 (recomputable
from the same seed). The algorithm mirrors safedelta_runner::xorshift64 exactly (same
constants: 13/7/17 shift) so the two harnesses share a reproducible PRNG lineage.

CI parameters: 500 resamples (BOOTSTRAP_N_RESAMPLES), 95% two-sided (2.5th / 97.5th
percentiles). Deltas are clamped to [-1.0, 1.0] per resample to stay in the valid rate
domain.

Known-answer tests verify:
  - determinism: same seed -> same CI (bootstrap_ci_known_answer_deterministic)
  - seed sensitivity: different seeds -> different CI bounds (bootstrap_ci_different_seeds_differ)
  - non-degenerate: CI straddles the point delta (bootstrap_ci_non_degenerate_fixture)


KEY DECISION 3: QdeltaMeasured as a flat beacon (safedelta-shaped, no Kind discriminator)

The SSB action type is SsbActionType::QdeltaMeasured (flat beacon, no Pattern-2 Kind
discriminator). This mirrors SafetyDeltaMeasured (G-6) in shape: a single payload struct
QdeltaMeasuredPayload with 15 fields:
  suite, model_id, n_tasks, seed_count, paired, on_defective_rate, off_defective_rate,
  delta, ci_low, ci_high, dollar_per_task, contamination_probe_result,
  surviving_mutants_per_task, corpus_blake3, runpod_pending.

SSB lockstep (9-tuple, verified against live source):
  - SsbActionType::QdeltaMeasured variant: ssb.rs:1286
  - SsbPayload::QdeltaMeasured arm: payloads.rs:3527
  - QdeltaMeasuredPayload struct: payloads.rs:15707
  - action_type_lock.rs golden row: present (gate_freeze_check PASS, 50 files locked)
  - SCHEMA_VERSION: derived (SCHEMA_VERSION_DDL_EPOCH + SSB_ACTION_TYPES.len());
    contract test asserts SCHEMA_VERSION_PUBLIC >= 235 (the I-38 authoring floor)
  - TOTAL_VARIANTS: derived; contract test asserts >= 544 (543 -> 544 for I-38)
  - match arms: present in ssb.rs lock array + payloads.rs enum
  - ts-rs binding: QdeltaLatest DTO in types.ts (16 fields, frontend_pair layer A)


KEY DECISION 4: D-20 contamination probe composed (not re-implemented)

QdeltaConfig has an optional contamination_probe_dataset: Option<String> field. When set,
run_inner() calls run_contamination_probe() which composes ARCAGI2Runner::run_all (the D-20
memorization probe) with a VecARCSsbEmitter (in-memory, no separate ARC SSB row). The
verdict string overwrites contamination_probe_result with the live result.

GD-1 compliance: the probe function is COMPOSED (called), not re-implemented. The QDELTA
harness only needs the verdict, not a separate ARC SSB row.

Failure mode: a probe failure (missing dataset, JSON parse error, runner error) returns
Err(ContaminationProbe) — a hard error. The harness NEVER silently skips the contamination
check or falsely reports "not_run". This is verified by contract test
run_contamination_probe_fails_on_missing_dataset.

serde default: contamination_probe_dataset defaults to None, so existing fixtures without
the field parse correctly (backwards-compatible).


KEY DECISION 5: Methodology doc auto-generated + deterministic (STEP 2)

generate_methodology_doc(config) produces a 7-section Markdown document from QdeltaConfig
fields + compile-time constants. The doc is deterministic under the same config (GD-2):
no timestamps, no RNG, no environment-dependent values in the body. Only the filename
timestamp varies (methodology-<unix_ms>.md).

The 7 sections:
  1. Framing (GD-3) — ablation-only, "no leaderboard rank" explicit prohibition
  2. Statistical floor — seed_count + paired pass/fail status
  3. Measurement protocol — QDELTA definition, defective criteria, bootstrap CI params
  4. Contamination probe (D-20) — the contamination_probe_result value
  5. Cost — $/task reported alongside accuracy (GD-3 $/task policy)
  6. Reproducibility (GD-2) — corpus_blake3, seed-recomputable CI, runpod_pending honesty
  7. Limitations — I-20 stub, fixture-tier scope, cross-repo deferred

The doc is written as a sidecar .md artifact alongside the JSON report at
<project_root>/benchmarks/qdelta/methodology-<unix_ms>.md.

stub_ban compliance: section 1 explicitly states "no leaderboard rank is claimed or
implied" (GD-3). SWE-bench Verified framing is explicitly declined (landmine #11).


KEY DECISION 6: Module split for maintainability (MR-1)

bench_qdelta.rs is ~875 lines. The methodology doc generator was split into a focused
submodule bench_qdelta_methodology.rs (~250 lines) via #[path = "..."] mod declaration.
This keeps the parent file under the MR-1 1000-line maintainability boundary. The
submodule accesses QdeltaConfig, constants, and BenchQdeltaError via super::{...} without
widening their visibility.


KEY DECISION 7: I-20 surviving-mutants leg is a stub (honest)

QdeltaMeasuredPayload.surviving_mutants_per_task is 0.0 at Tier-1. The code carries an
explicit TODO[I-38-I20-pending] marker at the assignment site (bench_qdelta.rs:408).
The methodology doc section 7 (Limitations) honestly states "surviving_mutants_per_task:
0.0 (I-20 stub, pending I-20 ship)".

This is NOT zero-lazy-code violation: the stub is explicitly marked, documented in the
methodology doc, and the field is serde-defaulted. The I-20 dependency (sampled mutation
scorer) is a forward dep that produces SSB rows I-38 will consume when it ships. Faking
non-zero mutant counts would violate GD-2 (recomputable) and the stub_ban ("NO fabricated
CI values").


PRODUCTION WIRING (R-8 / D-030)

The production hook seam is: `elai bench qdelta` CLI subcommand -> run_qdelta() ->
QdeltaMeasured SSB row.

Wiring chain (verified in live source):
  1. args.rs:1044 — CliAction::BenchQdelta variant in the CLI action enum
  2. args_bench.rs:1098 — parse_bench_qdelta_args() parses --config/--seed/--json
  3. args_bench.rs:1144 — returns CliAction::BenchQdelta { config, seed, json }
  4. main.rs:1389-1391 — match arm dispatches to bench_qdelta::run(config, seed, json)
  5. bench_qdelta.rs:329 — run() calls run_inner(), exits nonzero on error
  6. bench_qdelta.rs:340 — run_inner() validates floor, runs CI, emits SSB beacon
  7. bench_qdelta.rs:522 — emit_beacon() writes QdeltaMeasured to ssb.db

The CLI subcommand is reachable from the real elai binary entrypoint. No DARK symbols.


FRONTEND_PAIR (W-4 cross_lang_diff)

Layer A (infrastructure, built in prior steps):
  - QdeltaPanel.svelte: calls get_qdelta_latest Tauri command
  - commands_i38.rs: defines get_qdelta_latest Tauri command
  - lib.rs: registers get_qdelta_latest in generate_handler! macro
  - types.ts: exports QdeltaLatest DTO (16 fields matching Rust struct)
  - stub.ts: exports get_qdelta_latest stub for non-Tauri dev mode

Layer B (cockpit wiring — PENDING, outside this worker's allowed_write_set):
  - DrawerHost.svelte 'qdelta' render branch
  - types.ts DrawerKey union includes 'qdelta'
  - Drawer.svelte DRAWER_LABELS map includes 'qdelta'
  - Co-location: 'qdelta' branch next to 'safe-delta' branch

Layer B tests are #[ignore]'d in i38_qdelta_frontend_pair_contract.rs because the cockpit
wiring edits live in crates/dashboard-v2 (DrawerHost.svelte, types.ts, Drawer.svelte),
which is outside the allowed_write_set of crates/cli + crates/contracts. A subsequent
worker with dashboard-v2 write access must complete the wiring and un-ignore these tests.


CONTRACT TEST SURFACE (4 files, all in crates/contracts/tests/)

1. i38_qdelta_floor_contract.rs (5 arms):
   (a) QdeltaMeasuredPayload rmp_serde roundtrip via real Ssb::write_payload + query_by_type
   (b) Floor-refusal observable: zero QdeltaMeasured rows after a refused run
   (c) Known-answer paired-bootstrap CI bounds (delta=0.75, ci_low>=0, ci_high<=1)
   (d) Schema-lock: SCHEMA_VERSION_PUBLIC >= 235, TOTAL_VARIANTS >= 544
   (e) runpod_pending honesty: true round-trips correctly

2. i38_qdelta_fixture_contract.rs:
   - Fixture TOML files (qdelta_fixture.toml, qdelta_underpower.toml) parse correctly
   - Valid fixture passes floor_check; under-powered fixture fails floor_check
   - run_inner() on valid fixture emits a QdeltaMeasured SSB row with expected field values

3. i38_qdelta_contamination_contract.rs (STEP 5):
   - run_contamination_probe() returns "honestly_evaluated" for a valid D-20 fixture
   - Verdict is one of the three closed ARCVerdict wire labels
   - Missing dataset produces Err(ContaminationProbe), not silent "not_run"
   - QdeltaConfig.contamination_probe_dataset defaults to None (backwards-compatible)

4. i38_qdelta_frontend_pair_contract.rs (STEP 6):
   - Layer A: QdeltaPanel.svelte, commands_i38.rs, lib.rs, types.ts, stub.ts exist + wired
   - Layer B: #[ignore]'d (cockpit wiring pending dashboard-v2 write access)


EXPECTED TEST OUTCOME

- floor_check tests: PASS (4 inline tests in bench_qdelta.rs + contract arms)
- bootstrap CI tests: PASS (determinism + seed-sensitivity + non-degenerate)
- methodology doc tests: PASS (7 sections present, deterministic, floor verdict correct)
- D-20 contamination probe tests: PASS (verdict closed-set, failure is hard error)
- SSB roundtrip contract: PASS (QdeltaMeasuredPayload preserves all 15 fields)
- Schema-lock contract: PASS (SCHEMA_VERSION >= 235, TOTAL_VARIANTS >= 544)
- gate_freeze_check: PASS (verified: "GATE FREEZE CHECK - OK", 50 files locked)

BASELINE-RED tests (pre-existing, NOT introduced by I-38 — do NOT fix):
  - bench_convergence::tests::bootstrap_ci_different_seeds_differ
  - cli_tests::tests::<REDACTED_TOKEN>
  - cli_tests::tests::<REDACTED_TOKEN>


UNKNOWN / FORWARD DEPS

1. I-20 surviving-mutants leg: surviving_mutants_per_task is 0.0 (stub). When I-20 ships,
   run_inner() must query I-20 SSB rows for the surviving-mutant count per task and
   populate the field. The TODO[I-38-I20-pending] marker is at bench_qdelta.rs:408.

2. Frontend_pair Layer B (cockpit wiring): DrawerHost.svelte, types.ts DrawerKey,
   Drawer.svelte DRAWER_LABELS need 'qdelta' entries. This requires dashboard-v2 write
   access (outside this worker's allowed_write_set). The #[ignore]'d tests in
   i38_qdelta_frontend_pair_contract.rs gate this.

3. Full-corpus QDELTA run: RunPod-pending (D-026). The fixture tier proves the math;
   production-scale runs need the full E-54 task corpus + live inference.

4. Cross-repo cluster analysis of CI width: deferred (needs >=3 repos of real task data,
   GD-4). The bootstrap CI is per-run; clustering across repos is a Tier-2 deliverable.


VERIFICATION RUN

- gate_freeze_check.py: PASS (exit 0, "GATE FREEZE CHECK - OK", hash=sha3_256, 50 files)
- production_caller_check.py: PASS (0 dark fn, 3 warn, 4 wired — QdeltaConfig, BenchQdeltaError,
  floor_check, run_inner all WIRED; run is UNSOUND-NAME warning but is the CLI entrypoint)
- No gate edits, no gate relock, no destructive git, no commit, no push (per forbidden list)
- All code edits are within crates/cli + crates/contracts (allowed_write_set)
- This decision doc is in workflow/reasoning/ (workflow tooling root, not .elai_cc)


COMPOSES-WITH (GD-1 compliance)

- G-6 safedelta_runner: QdeltaConfig mirrors SafedeltaConfig shape; xorshift64 algorithm
  shared; bootstrap N_RESAMPLES=500 shared. No code fork.
- D-6/E-54 held-out corpus: the same corpus path convention; no re-implementation of
  corpus loading.
- D-20 memorization probe: run_contamination_probe() calls ARCAGI2Runner::run_all
  (composed, not re-implemented).
- I-20 mutation scorer: surviving_mutants_per_task field reserved (0.0 stub); will consume
  I-20 SSB rows when I-20 ships.
- E-49 metrics infrastructure: the measurement plumbing is composed via the existing
  Ssb::write_payload + SsbConfig + OsClock path.

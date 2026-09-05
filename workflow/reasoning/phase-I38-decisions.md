I-38 QDELTA statistical floor — decision doc
Session: s-107b022a / agent task A70
Date: 2026-06-12

WHAT
Build the QDELTA defective-diff rate bench harness (fixture tier only, benchmark RUN deferred per
D-026). Deliverables: bench_qdelta.rs, QdeltaMeasured SSB variant, CLI wiring, dashboard tile,
contract test, stubs.md entry.

KEY DECISIONS

D-001 contamination_probe as config string field
Context: plan says "contamination_probe field calls the D-20 memorization probe function". Live
D-20 (arc_agi2_runner.rs) is async, takes a fixture_path, returns ARCReport — not a one-liner.
Decision: contamination_probe_result is a String in QdeltaConfig (TOML field, default "not_run").
Allowed values: "not_run" | "honestly_evaluated" | "game_with_memorization". No inline D-20 call.
Rationale: exploration map HIGH WARNING + arc_agi2_runner.rs confirmed — inline call would require
async setup + fixture path + VecARCSsbEmitter, incompatible with fixture-tier run_inner().

D-002 bootstrap_ci takes paired bool vecs
Context: safedelta_runner::bootstrap_ci resamples a single aggregated population. QDELTA measures
ON vs OFF on the SAME tasks (paired design). The statistic is delta = mean(off_cells) - mean(on_cells).
Decision: fn bootstrap_ci(on_cells: &[bool], off_cells: &[bool], seed: u64, n_resamples: usize) -> (f64, f64)
resamples INDEX PAIRS (draw n index positions with replacement from 0..n_tasks, compute both
on_rate and off_rate from the same indices, resample_delta = off_rate - on_rate). Same xorshift64.
Rationale: exploration map BOOTSTRAP_CI PAIRED VECTORS gap confirmed. Different statistic from
safedelta (paired delta vs single-population blocked fraction).

D-003 surviving_mutants_per_task as registered stub
Context: I-20 mutation scorer not yet shipped (Wave 3). Field must exist in payload.
Decision: surviving_mutants_per_task = 0.0_f64 with // TODO[I-38-I20-pending] marker.
Stub row registered in stubs.md will_resolve_in=I-20.

D-004 floor_check exits nonzero before any compute
Context: plan says harness REFUSES under-powered runs. Observable: no QdeltaMeasured row emitted.
Decision: floor_check() returns Err(BenchQdeltaError::FloorViolation) if seed_count < 5 OR
paired == false. run_inner() calls floor_check() before any bootstrap_ci or SSB emit.
Contract test verifies: under-powered run -> zero QdeltaMeasured rows in SSB.

D-005 SSB baseline 234/543, I-38 produces 235/544
Context: plan WIRING_CONTRACT had stale baseline 221/532. Live ssb.rs:3096 = 234, lock:676 = 543.
Decision: bump to 235/544. All 9 atomic lockstep sites updated atomically per exploration map.

D-006 Dashboard honest empty state
Context: no real QDELTA runs exist yet (fixture tier only, D-026 deferred).
Decision: QdeltaPanel renders "No qdelta_measured rows yet. Run elai bench qdelta to populate."
for row==null. No fake numbers. Mirrors SafeDeltaPanel empty state.

EXPECTED TEST OUTCOME
- cargo check --workspace --tests: exit 0 (zero new errors)
- i38 contract test: floor_check arm passes (FloorViolation returned, no SSB row), schema-lock arm 235/544
- action_type_lock golden 2/2: SCHEMA 235 + TV 544
- e45 contract: TV 544 string match
- i34 contract: TV 544 string match

UNKNOWNS / RISKS AT EXECUTION TIME
- all_test_payloads len must increment by 1 (433 -> 434); the trailing assert_eq comment string
  must be updated consistently.
- Tauri generate_handler! registration in lib.rs must include get_qdelta_latest.
- stub.ts must export get_qdelta_latest with the tryInvoke pattern (no SafeDelta section exists
  in stub.ts currently — it was not added for G-6; stub.ts instead has the full SafeDelta type
  in types.ts. Pattern: add the export fn + tryInvoke near the end of stub.ts).

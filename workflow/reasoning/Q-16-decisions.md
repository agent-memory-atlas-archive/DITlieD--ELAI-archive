Q-16 DECISIONS
elai-on-elai e2e functional suite (GO-2, PSR-1 #2)

WHAT
a black-box e2e harness that drives the compiled elai binary end to end and asserts on off-agent
evidence the harness did not author (SSB journal rows, dkg.db counts, stdout verdicts, an RRA seal).
python stdlib runner under .elai_cc/tests/e2e/; each scenario is a python fn that shells
.elai_cc/target/debug/elai in an isolated ELAI home, captures stdout+exit, then queries the ssb.db
the run produced. one runner CLI consumes every scenario fn in the same changeset (that CLI is the
production caller, so python_wire_check stays clean). Q-16 ships NO new pub Rust symbols.

WHY THIS PLAN, WHY NOW
PSR-1 ordering. the cursor sat on PS-3 (hallucination-delta). PS-3 has no runner in .elai_cc/crates
(grep hallucination-delta = 0 matches), so it is a BUILD task, not a cheap measure, and PSR-1 ranks
building a PS number's runner (#4) below the GO-2 e2e suite (#2). Q-16 is already on disk, gated PASS
(plan_gate_check exit 0, all deterministic points green), genuinely ungated by the :8181 lane for its
Tier-0 legs, and it POPULATES ssb.db (CitationAnchorsBound, InsecureRateMeasured, Dkg* rows) which is
the substrate PS-2 (composed gate-tax vs locked ceiling) reads from. so walking Q-16 both ships the
priority deliverable and unblocks a downstream proof-spine number. PS-3 stays PLANNED, not
dispositioned (it is not confirmed-stale, just deferred).

LANE
free swe-1-7 only. codex is dead this window (both accounts 403 INSUFFICIENT_BALANCE, credits expired
2026-07-15). every execution unit routes through opus_dispatch.py --lane free. the local model lane
(elai CLI > LlamaCppAdapter > http://localhost:8181 > llama.cpp Vulkan > Qwen3.5-4B Q4_K_M) is live
end to end and serves the Tier-1 inference legs (M5, M6). tag every measured number with its lane
(model+quant+host); the 4B lane is the thesis substrate.

THE SIX MOVES (walk order, cheapest first)
M1 S-1 prebind, Tier-0. builds the runner skeleton + first scenario. drive `elai prebind
   --demo-prebind` in an isolated ELAI home, assert exactly one CitationAnchorsBound SSB row (state
   Bound, expected anchor count) at <project_root>/.elai_cc/sessions/ssb.db; the
   --allow-freeform-citations arm asserts Blocked/freeform. this is the walking skeleton: producer
   (scenario) + consumer (runner CLI) + smoke (the assert) in one unit.
M2 S-2 seccodeplt, Tier-0. `elai bench seccodeplt --json`, assert one InsecureRateMeasured beacon,
   insecure_rate within fixture tolerance.
M3 S-3 crane, Tier-0. `elai bench crane`, assert the STDOUT verdict summary. GrammarGateEvent rows go
   to a hermetic tempdir bench_crane.sqlite that is torn down, so it is UNQUERYABLE; assert stdout,
   NOT ssb.db. --seed is parsed but not honored, so no seed-determinism assertion.
M4 S-4 symbol_graph, Tier-0. `elai symbol_graph build --project-root <fixture>`, assert
   DkgRebuildStartedEvent + DkgRebuildCompleteEvent rows and dkg.db symbol_count>0 and edge_count>0.
M5 S-5 prompt, Tier-1 (RECON-0 :8181-gated). `ANTHROPIC_BASE_URL=http://localhost:8181 elai prompt
   "<checkable>"` (or --backend=local:8181; NOT --provider, which is Run-only; NOT the 8100 default).
   assert exit 0 + stdout semantic content match. CLI prompt emits NO ChatResponse SSB (GUI-only), so
   the assertion is on exit+stdout. :8181 down = SKIPPED-EXPLICIT, never a silent pass.
M6 INTEGRATION_SMOKE, Tier-1 (RECON-0 gated). `elai run --project-root <fixture> --provider local
   --emit-rra` multi-phase, assert PhaseComplete rows + RunArtifactEmitted + the RRA seal file, with
   real :8181 inference per phase (a HARD input, not a toy single-turn happy path) per R10.

KEY DECISIONS
- ssb.db path is heterogeneous (RECON-3): S-1/S-2/S-4 at <project_root>/.elai_cc/sessions/ssb.db;
  S-3 hermetic tempdir (unqueryable, assert stdout); S-5 emits none (assert exit+stdout). the runner
  resolves the db path per scenario, it does not assume one location.
- assert on the off-agent artifact, never on the executor's stdout self-report where a journal row
  exists. where no journal row exists (S-3, S-5) the stdout IS the only evidence and that is declared
  in the scenario, not smuggled.
- no new pub Rust symbols, so the Rust wiring gate is n/a; the python wiring proof is the single
  runner CLI calling every scenario fn (python_wire_check clean).

EXPECTED OUTCOME
auditor (e2e-tester / off-agent) drives the runner, sees M1-M4 green on the Tier-0 legs with real SSB
rows, M5+M6 green against live :8181 (or SKIPPED-EXPLICIT if the lane drops). the load-bearing proof
is the journal rows + the RRA seal the harness did not write, not the runner's own stdout.

UNKNOWNS / RECON TRIPLINES (from the plan)
RECON-0 :8181 reachable+serving for M5/M6 (checked live: /health = {"status":"ok"}).
RECON-2 elai binary present+executable (checked: .elai_cc/target/debug/elai, Jul 17 09:08, 453MB).
RECON-3 the per-scenario ssb.db path resolves in the scratch home; if absent, re-point the query, do
   NOT weaken to stdout-only (M1 ON_FAIL).

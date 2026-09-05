# K-4: composed gate-tax budget — measure the in-band CODING trust-spine gate friction (per-task) + the offline audit gate tax (per-audit), each vs its own locked ceiling (KEYSTONE of K-T3)

STATUS: PLANNED (WAVE KW-3; BUILD instrument + doctrine; KEYSTONE of K-T3; KD-3). REFINED 2026-07-08 v2 (refine-before-execute: wargame blocks + 3 premise fixes + 6 red-team patches + the scope decision with three riders, all locked with the user).

PROOF_SPINE: feeds PS-2 — owner of the composed gate-tax spine number (proof-spine-2026-07-05.md line 16-17).

Objective: GD-4 bounds gates INDIVIDUALLY; nothing measures the COMPOSED friction of the trust spine on a normal
coding task, and the research says composed friction is the category-killer (FP friction killed prior gate
generations; orchestration overhead is itself a bottleneck — KW-4 CodeAct ~50% latency / >60% tokens; KS-3 report
16 bear-case #2). This session BUILDS `elai bench gate-tax`: a per-release friction report answering the
CodeGate-death question — "what does the trust spine cost on a normal coding task?" — against a LOCKED ceiling;
ceiling breach = a non-zero CLI exit the release path consumes; the report names the top-3 friction contributors.

THE KD-1 DEFINITION (locked with the user 2026-07-08, incl. the three scope riders — this is what the number
MEANS, mechanically). A number means ONE thing or it means nothing, so the scope is fixed and the artifact reports
TWO labeled numbers, NEVER summed:
- coding_task_gate_tax (the ceiling-bound headline): per-task added wall-clock of the IN-BAND runtime coding gate
  set = the FSM EvidenceGate family (11 gates) + the T1/T2/T3 hooks + anti-fabrication + the compliance pre-gate —
  the gates that fire in-band on every coding task and are telemetry-visible (EvidenceGateEvaluated / HookEvent /
  TieredAnalysis rows). Composed = measured-frequency-weighted per-gate unit costs, journal-recomputable (GD-2).
- audit_gate_tax (a SEPARATE per-audit number, its own ceiling or an explicit "unbounded, observed" tag, NEVER
  summed into the per-task number — unit honesty: per-task ms and per-audit seconds are different units): the
  wall-clock of the OFFLINE auditor checkers paid once per changeset (production_caller_check.py,
  maintainability_check.py, wiring_contract_check.py, plan_gate_check.py, taste_check.py), timed by invocation.
- over_block_rate (per-task): a NEW benign-corpus sweep — N known-good coding diffs run through the in-scope gate
  set, blocked/total per gate + composed (NOT the GateOverBlocked meta-gate artifact; see FIX-4 / red-team ATTACK-3).
- RIDER-b COVERAGE TABLE (the structural anti-silent-narrowing guard): every run the runner emits a coverage table
  tagging EVERY gate-manifest.txt-complement gate as in-scope-coding | trading-surface | offline-audit | dormant.
  An untagged manifest gate = FAIL. What the number does NOT cover is enumerated beside it (GD-5 in mechanical form).
- RIDER-c FUTURE ARM: scope is per-TASK-CLASS by design. The identical runner re-runs with a trading corpus later to
  produce trading_task_gate_tax as its own number — the finance/trading gates are not dropped, they are a future arm.
- LOCKED CEILINGS bind each number to ITS OWN ceiling (coding_task ceiling is the release-blocking one).
- RESIDUE (GD-5): the composed per-task number is a freq-weighted sum of parts and UNDERESTIMATES interaction
  effects (a block that triggers a retry loop costs more than the gate's wall-clock); the over-block rate covers the
  largest interaction class, the coverage table names the uncovered gates, the remainder is NAMED residue.
- DOSSIER ROW (KD-1, scope inside the claim so it can't be quietly widened): "composed gate-tax on coding tasks
  (in-band runtime set; audit tax reported separately; trading surface unmeasured)".

VETO ON RECORD (user, 2026-07-08): Option-2 "whole-pipeline global gate-bypass, run the corpus twice ON/OFF" is
REJECTED — (1) doctrine: a shipped runtime switch that disables the gate stack is the exact artifact R8/W-2
freeze-the-gate exists to prevent; (2) measurement: end-to-end LLM wall-clock is provider-latency-variance-dominated,
so the ON/OFF delta of sub-ms gates drowns in noise. A `cfg(bench)`-gated whole-pipeline falsifier MAY exist LATER as
a one-shot validation of this cost model — compiled out of release, NEVER a runtime flag, NEVER the primary instrument.

Estimated effort: ~1.5 sessions (~6-8 h; revised UP from ~1 session per red-team ATTACK-3 — the over-block leg is a
new benign-corpus sweep, not a reuse, + the coverage table + the second per-audit leg). In-process; the per-task leg
reads the journal + a fixture corpus, the per-audit leg times the offline checkers; no sidecar, no RunPod.
Pre-requisites: case_reuse_runner.rs + bench_case_reuse.rs (the proven runner + 4-file CLI template, VERIFIED). The
bench dispatcher args_bench.rs. EvidenceGateEvaluated (action_type.rs:308) + HookEvent (:87) + TieredAnalysis
(elapsed_us) SSB rows (VERIFIED). gate-manifest.txt (the manifest-complement source for the coverage table).
Parallel-safe with: K-2, K-3, K-6, K-9. K-5 DEPENDS on K-4 (the ceiling it must fit under).
Source: K-MASTER-BRIEF §3.2 K-4 [KS-1]; KS-3 report 16/17, KW-4; grounding map KG-5/KG-9, KD-3. Doctrine: K-0_INDEX.md, proof-spine.md.

## Context
A trust spine nobody keeps ON is worthless; the composed per-task friction number is what makes the spine affordable
enough to leave default-ON on coding work (KD-3). External: KW-4 (CodeAct) is [DESIGN] context; the ELAI number is
measured (HD-1).

## Existing state (A/B..J coverage) — CORRECTED 2026-07-08
- case_reuse_runner.rs + bench_case_reuse.rs (K-18a) = the proven benchmarks-runner + 4-file CLI-wiring template.
  bench_qdelta.rs = a runner-shape sibling.
- EvidenceGateEvaluated (action_type.rs:308 — gate_id, verdict, ts_ms; ts_ms is POINT-IN-TIME, not a duration) +
  HookEvent (:87, T1/T2/T3 dispatch) + TieredAnalysisPayload.elapsed_us (the ONLY genuine per-tier wall-clock
  duration field, a DIFFERENT action_type) = the Leg-B fire-count + Leg-A per-family timing sources. CONFIRMED.
- EvidenceGate::check (evidence.rs) drives the 11 FSM gates: knowledge_readiness, grounding_sweep, test_fail_reasons,
  test_flipped, replan_fast_track, zero_blocks, dkg_changelog, intent_pairs, diagnostic_probe, anti_fabrication,
  entrypoint_and_trace — the telemetry-visible in-band coding set (+ hooks + compliance pre-gate).
- gate-manifest.txt = the frozen gate registry (R8) + the manifest-complement source for the RIDER-b coverage table.
- GD-4 (per-gate FP bound, >=3 real runs) — K-4 EXTENDS it to the composed coding set (KD-3).

### CORRECTED PREMISES (refine-before-execute fixes; the original plan was unsound here)
- FIX-1 (inventory scope): the original sourced "the default-ON gate inventory" from wire-dark-allowlist.toml — the
  OPPOSITE (the default-OFF EXEMPTION registry, 135 rows, not 105). The gate set is scoped by task-class: the IN-BAND
  RUNTIME CODING gates (telemetry-visible), with the manifest-complement coverage table (RIDER-b) enumerating what is
  out of scope (trading-surface / offline-audit / dormant). See RECON_NEEDED-1 + RECON_NEEDED-1b.
- FIX-2 (no global toggle): NO single gates-ON/OFF switch exists; Leg A measures each gate's unit cost in ISOLATION
  (OFF = the gate not run), never a product-level bypass (VETO / ABORT-1).
- FIX-3 (safedelta is static): safedelta_runner.rs's "ON/OFF idiom" is a STATIC before/after split in one pass — no
  wall-clock number. Leg A uses per-gate-family timing (journal elapsed_us OR a bench microbench), not that idiom.
- FIX-4 (over-block source, red-team ATTACK-3): GateOverBlocked is emitted only by the H-27 meta-gate over 8 finance
  gates × ONE fixture each (pass/fail, no corpus, no rate, none of the coding gates). It CANNOT produce a corpus-scale
  over-block rate. The over-block leg is a NEW benign-corpus sweep over the in-scope coding gates.

## Reads (mandatory before coding)
1. _grounding/K-GROUNDING-MAP.md (KG-5/KG-9, KD-3).
2. crates/orchestrator/src/benchmarks/case_reuse_runner.rs + crates/cli/src/bench_case_reuse.rs (runner + CLI
   template) + crates/cli/src/args_bench.rs (dispatcher arm) + crates/cli/src/bench_qdelta.rs (sibling).
3. crates/orchestrator/src/evidence.rs (EvidenceGate::check — the 11-gate in-scope family + how each emits its row).
4. crates/orchestrator/src/ssb/action_type.rs:308/:87 + the EvidenceGateEvaluatedPayload / HookEventPayload /
   TieredAnalysisPayload structs in ssb/payloads.rs (Leg-B fire-count + Leg-A elapsed_us duration source).
5. crates/orchestrator/src/ssb_viewer.rs (SsbViewer::open — the read-only journal reader).
6. .claude/gate-manifest.txt (the manifest-complement source for the coverage table) + gate_self_validation.rs (to
   confirm FIX-4: GateOverBlocked is NOT a corpus rate).
7. .claude/tools/{production_caller_check,maintainability_check,wiring_contract_check,plan_gate_check,taste_check}.py
   (the offline auditor checkers the per-audit leg times).

## RECON_NEEDED (each carries its exact settling check)
- RECON_NEEDED-1 (in-scope coding gate set): enumerate the distinct gate_ids the 11-gate EvidenceGate family + hooks
  emit. SETTLING CHECK: `SsbViewer::open(db).query_by_type(EvidenceGateEvaluated)` + `query_by_type(HookEvent)` →
  distinct gate_id/rule_id. EMPTY (dry journal, the K-18a TestOutcome class) → fall to a named in-repo FIXTURE gate
  set + mark RUNNER_PRESENT-not-MEASURED (never fabricate a live composition). Load-bearing.
- RECON_NEEDED-1b (RIDER-b coverage): enumerate EVERY gate in gate-manifest.txt and its emitted SsbActionType by
  grepping each gate source file. SETTLING CHECK: `rg -n "SsbActionType::|SsbPayload::" <each manifest gate .rs>` →
  build the tag map {gate → in-scope-coding | trading-surface | offline-audit | dormant}. An untagged manifest gate
  is a runner FAIL (RIDER-b). This is the anti-silent-narrowing guard.
- RECON_NEEDED-2 (Leg-A per-family unit-cost source): CLASSIFY ts_ms as POINT-IN-TIME, not a duration (red-team
  ATTACK-2). The only genuine duration is TieredAnalysisPayload.elapsed_us (hook-tier). SETTLING CHECK:
  `rg -n "elapsed_us|elapsed|duration" crates/orchestrator/src/ssb/payloads.rs` — require a field DOCUMENTED as
  elapsed/duration, not a bare `_ms` timestamp. Result feeds FORK-1's per-family routing.
- RECON_NEEDED-3 (task-class key): SETTLING CHECK: `rg -n "task_class|TaskClass|task_kind|template_id"
  crates/orchestrator/src/ssb/*.rs`. No first-class field → key by template_id and NAME it the task-class proxy
  (GD-5 residue note); do not invent a taxonomy.
- RECON_NEEDED-4 (benign corpus for the over-block sweep, FIX-4): a set of N KNOWN-GOOD coding diffs that MUST NOT
  block. SETTLING CHECK: `rg -n "benign|clean|negative" crates/orchestrator/src/benchmarks/safedelta_runner.rs`; if
  no reusable benign set, author a small in-repo fixture (distinct from the recurrence fixture).
- RECON_NEEDED-5 (per-audit checker set + timing): SETTLING CHECK: confirm each offline checker in Reads §7 runs
  headless with a stable exit; time each via `std::process::Command` wall-clock over K reps (median). No first-class
  "audit run" telemetry → the per-audit number is measured at invocation and labeled "observed, unbounded" unless a
  ceiling is set.

## FORKS
- FORK-1 (Leg-A route, PER-GATE-FAMILY — red-team ATTACK-2): the hook-tier gates carry a real duration
  (TieredAnalysis.elapsed_us) → Route A (journal-native median duration, GD-2, no gate invocation). The FSM
  EvidenceGate family has NO duration signal → Route B (a bench microharness times each in-scope evaluate fn on a
  fixed fixture over K seeded reps, median; reuse gate_self_validation.rs's uniform gate-driving loop as the adapter,
  scope to gates with a drivable signature, NAME any excluded gate as residue). NOT one global route.
- FORK-2 (SSB beacon): existing bench row carrying all GateTaxMeasured fields? Grep at edit. None → NEW flat
  GateTaxMeasured action_type (mirror RecurrenceHitRateMeasured c2daaf4: flat scalars, no free-text, SCHEMA_VERSION
  auto-derived, golden action_type_lock regenerated SAME commit). Default: NEW row.
- FORK-3 (ceiling values): TRIGGER = the FIRST clean run producing the two numbers + over-block rate. The coding_task
  ceiling VALUE is USER-SET from that run (never invented); the audit ceiling is user-set OR tagged "unbounded,
  observed". Route: emit the numbers, SURFACE the proposed ceilings, lock into K-4-decisions.md + named consts ONLY
  after the user confirms. Until confirmed the const carries `// CEILING-PENDING[K-4]` and the exit contract
  fail-CLOSED-blocks (ABORT-5), never a silent pass.

## ABORT conditions
- ABORT-1: any move requiring a product-level runtime switch that disables the gate stack → STOP + handoff (VETO /
  R8/W-2). The bench OFF arm is bench-local (a gate not called in the runner), never a shipped flag.
- ABORT-2: RECON_NEEDED-1 empty AND no fixture gate set authored → STOP; a plumbing proof is RUNNER_PRESENT, never
  MEASURED (the K-18a lesson) — label honestly, do not emit a MEASURED composition over zero data.
- ABORT-2b (RIDER-b): any gate-manifest.txt gate that the coverage table cannot tag → FAIL the run; the headline may
  NOT say "whole coding trust-spine" if an in-scope gate is uncovered — it names the gap (GD-5).
- ABORT-3: the ceiling would have to be invented to make a test pass → STOP (stub_ban: NO invented ceiling).
- ABORT-4: a new GateTaxMeasured row needs more than flat scalars (any free-text / nested field) → STOP + handoff
  (a walled-off schema decision, the K-18a ABORT-4 precedent).
- ABORT-5 (red-team ATTACK-5): if the coding ceiling is CEILING-PENDING and the breach exit contract is reachable
  from a real release path, the bench MUST exit non-zero (fail-CLOSED, block) — NEVER a silent pass (R1/R3). Label it
  "release blocked pending ceiling confirmation" in K-4-decisions.md; reserve WIRE-DARK only for a truly unwired period.

## Tasks (WAVE KW-3 — the instrument) — each a MOVE with EXPECTED_OBSERVATION + ON_FAIL

MOVE 1 — settle the recon. Run RECON_NEEDED 1/1b/2/3/4/5; record in K-4-decisions.md; pick FORK-1 per-family routes
and the task-class key.
  EXPECTED_OBSERVATION: K-4-decisions.md lists the in-scope gate_id set (or "EMPTY → fixture"), the manifest-complement
  tag map (RIDER-b), the elapsed_us-vs-ts_ms verdict, the task-class key, the benign fixture path, and the per-audit
  checker list. Per-family routes chosen with reasons.
  ON_FAIL: SsbViewer db-not-found → FAIL not SKIP (R1/R2); cause = dry evidence path; counter-move = fixture gate set
  + RUNNER_PRESENT (ABORT-2).

MOVE 2 — build the gate-tax runner (benchmarks/gate_tax_runner.rs, compose case_reuse_runner.rs). `pub fn
run_gate_tax(seed: u64) -> Result<GateTaxReport, GateTaxError>`. Emits: coding_task_gate_tax (Leg A per FORK-1 × Leg B
fire_counts over the journal), over_block_rate (the NEW benign-corpus sweep, FIX-4), the RIDER-b coverage table, ranked
top-3 contributors, named residue. ~330-420 LOC. No .unwrap()/.expect() on the prod path.
  EXPECTED_OBSERVATION: `cargo build -p elai-orchestrator -j 1` GREEN; run_gate_tax(1234) returns a GateTaxReport with
  coding_task_gate_tax_ms per task class, over_block_rate, a coverage table where every manifest gate is tagged
  (0 untagged), ranked top-3, and a non-empty residue string; a 2nd call at the same seed is byte-identical (GD-2).
  ON_FAIL: coding_task_gate_tax=0 for every class → cause = fire_count all zero (empty journal, RECON-1) OR unit_cost
  misread; counter-move = assert the gate set non-empty before compose + FAIL-loud on all-zero (R2), never report 0
  as a pass. A missing coverage tag → ABORT-2b FAIL.

MOVE 3 — the LOCKED ceilings + the breach EXIT-CODE contract (red-team ATTACK-4: there is NO release_gate in the repo;
the "release gate" is the CLI non-zero EXIT the release path/CI consumes). Two named consts (coding + audit; VALUEs per
FORK-3, CEILING-PENDING), each mutable=0-style, frozen-gate-adjacent. coding-ceiling breach → non-zero exit; document
the exit-code contract in K-4-decisions.md. ~40 LOC, NO fictional Rust release-gate fn.
  EXPECTED_OBSERVATION: `elai bench gate-tax` exits non-zero when coding_task_gate_tax > coding ceiling, 0 when under;
  the consts carry `// CEILING-PENDING[K-4]` until confirmed; while pending + reachable, the exit is non-zero (ABORT-5).
  ON_FAIL: breach only warns / exits 0 → cause = the exit code not driven by the verdict; counter-move = the process
  exit code IS the breach verdict (fail-closed, R1); a warn-only breach is a stub_ban BLOCK.

MOVE 4 — the per-audit leg + the `gate-tax` bench sub. Time the offline auditor checkers (RECON-5) → audit_gate_tax_ms
(separate number, own ceiling or "unbounded observed", NEVER summed with the per-task number). Add the 4-file CLI
wiring (args_bench.rs arm + bench_gate_tax.rs handler + main.rs dispatch + args.rs variant), K-18a pattern. `elai bench
gate-tax [--seed <u64>] [--json]`. ~90 LOC.
  EXPECTED_OBSERVATION: `elai bench gate-tax --seed 1234 --json` prints BOTH numbers as SEPARATE labeled fields
  (coding_task_gate_tax_ms, audit_gate_tax_ms) + the coverage table + exits per the coding ceiling. `rg -n "gate-tax"
  crates/cli/src/args_bench.rs` shows the arm.
  ON_FAIL: the two numbers are summed into one field → cause = unit-honesty violation (rider a); counter-move = two
  distinct payload fields, two ceilings; `unknown bench subcommand` → re-grep the four wiring sites (case-reuse template).

MOVE 5 — SSB GateTaxMeasured beacon (FORK-2 default: NEW flat action_type). Payload = flat scalars: coding_task_gate_tax_ms
per named class, audit_gate_tax_ms, audit_unbounded bool, over_block_rate, coding_ceiling_ms, coding_breached bool,
audit_ceiling_ms, top-3 contributor gate_ids + costs, coverage counts (n_in_scope_coding, n_trading_surface,
n_offline_audit, n_dormant, n_untagged==0), n_gates, n_fires. SCHEMA_VERSION auto-derived; golden action_type_lock
regenerated SAME commit; signed .rra like K-18a. ~120 LOC.
  EXPECTED_OBSERVATION: `elai bench gate-tax --seed 1234` writes a GateTaxMeasured row; action_type_lock GREEN (golden
  regenerated); the row round-trips (rmp) in a contract test; both numbers recompute from the row + the journal (GD-2).
  ON_FAIL: action_type_lock RED → cause = golden not regenerated / SCHEMA_VERSION hand-bumped; counter-move =
  regenerate the golden same commit, let TOTAL_VARIANTS derive (ssb.rs:1445).

MOVE 6 — the NEGATIVE / over-block control arm. A deliberately spammy fixture gate over the BENIGN corpus pushes
over_block_rate above the coding ceiling and `elai bench gate-tax` exits non-zero; a clean gate set stays under and exits 0.
  EXPECTED_OBSERVATION: the contract test's spammy-gate fixture yields over_block_rate above ceiling → non-zero exit;
  the benign clean run stays under → exit 0.
  ON_FAIL: the spammy arm exits 0 → cause = over_block_rate not bound to the coding ceiling; counter-move = bind BOTH
  coding_task_gate_tax AND over_block_rate to the coding ceiling.

MOVE 7 — workflow/reasoning/K-4-decisions.md (recon results, the manifest-complement tag map, the chosen ceiling values
+ provenance + the exit-code contract, Route A/B per family, named residue, RUNNER_PRESENT-vs-MEASURED status).
  EXPECTED_OBSERVATION: `test -f workflow/reasoning/K-4-decisions.md` and it names both ceilings + their source run +
  the exit-code contract.
  ON_FAIL: missing → the auditor's decision-doc check flags it; counter-move = write it before AUDITING.

## PRODUCTION_HOOK (the R-8/D-030 seam — named, wired same changeset)
- Producer `run_gate_tax` → consumer `crates/cli/src/bench_gate_tax.rs::run` (the CLI handler), reached from the REAL
  entrypoint `elai bench gate-tax` (args_bench.rs dispatch → main.rs arm). Mirrors bench_case_reuse.rs::run. This is the
  SOLE producer/consumer pair (the fictional release-gate second seam was struck per red-team ATTACK-4; the breach is
  the process exit code, a contract, not a Rust fn).
- A #[cfg(test)]/contracts/ caller NEVER satisfies reachability (R7). Self-verify: `python
  .claude/tools/production_caller_check.py --files <changed .rs> --json` → 0 DARK before DONE.

## INTEGRATION_SMOKE (R10 — independent, real entrypoint, hard inputs; THREE cases)
After MOVE 5 AND before final DONE an INDEPENDENT check (the auditor, not the executor) drives the REAL
`elai bench gate-tax --seed 1234 --json` binary end-to-end and asserts, off-agent: (1) HAPPY — a GateTaxMeasured row
lands with BOTH labeled numbers + a fully-tagged coverage table (0 untagged), and both numbers recompute from the row +
journal (GD-2); (2) OVER-CEILING — the spammy fixture path exits non-zero; (3) DRY-JOURNAL (red-team ATTACK-6) — the
real binary against an empty/unreachable journal exits non-zero AND emits no MEASURED-labeled row (RUNNER_PRESENT only).
Toy happy-path prints do NOT satisfy R10. db-not-found / zero-rows = FAIL (R1).

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/benchmarks/gate_tax_runner.rs (run_gate_tax → GateTaxReport with two numbers + coverage table)
  - crates/cli/src/bench_gate_tax.rs + the `gate-tax` arm at args_bench.rs (+ args.rs variant + main.rs dispatch)
  - two LOCKED ceiling constants (coding + audit) + the breach EXIT-CODE contract (non-zero on coding breach)
  - GateTaxMeasured SSB action_type + payload (FORK-2 default NEW flat row)
consumes_symbols:
  - benchmarks::case_reuse_runner (COMPOSE the shape) + bench_qdelta (sibling shape)
  - ssb EvidenceGateEvaluated + HookEvent + TieredAnalysis rows via SsbViewer (Leg-B fire-counts + Leg-A elapsed_us)
  - evidence.rs EvidenceGate family (the in-scope gate set) + gate_self_validation.rs's gate-driving loop (Route-B adapter)
  - gate-manifest.txt (the manifest-complement source for the RIDER-b coverage table)
  - the offline auditor checkers in Reads §7 (the per-audit leg times their invocation)
reachable_from:
  - `elai bench gate-tax [--seed <u64>] [--json]` [PRODUCTION CLI]
  - the release path / CI consumes the non-zero EXIT CODE on a coding-ceiling breach [PRODUCTION exit-code contract]
  - a #[cfg(test)]/contracts/ caller NEVER satisfies this (R7)
stub_ban:
  - NO invented ceiling value (user-set from the first run; CEILING-PENDING until confirmed; fail-closed while pending, ABORT-5)
  - NO gate set from wire-dark-allowlist.toml (default-OFF registry — FIX-1); source in-scope from evidence.rs + telemetry
  - NO summing the per-task and per-audit numbers (rider a, unit honesty)
  - NO untagged manifest gate in the coverage table (RIDER-b / ABORT-2b)
  - NO product-level gate-bypass switch (VETO / ABORT-1); NO fictional Rust release-gate (ATTACK-4)
  - NO GateOverBlocked as the over-block rate (FIX-4); NO safedelta STATIC idiom for wall-clock (FIX-3)
  - NO ceiling-breach that only warns (breach = non-zero exit; fail-closed, R1); NO .unwrap()/.expect() on the prod path
  - NO MEASURED label on a fixture/empty-journal run (RUNNER_PRESENT only — ABORT-2)
observable_effect:
  trigger: "elai bench gate-tax --seed 1234 --json"
  expect_ssb: "GateTaxMeasured (flat, two labeled numbers + coverage counts) — recomputable from the journal (GD-2)"
  expect_artifact: "coding_task_gate_tax + audit_gate_tax (never summed) + over_block_rate + a fully-tagged coverage
    table, each vs its own ceiling; a spammy fixture gate trips the coding ceiling and the bench exits non-zero (R2)"

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/K-plan/K-4_composed_gate_tax_budget.md --run-trigger
rg -n "gate-tax" crates/cli/src/args_bench.rs                         # the dispatcher arm exists
rg -n "EvidenceGateEvaluated|HookEvent|TieredAnalysis" crates/orchestrator/src/benchmarks/gate_tax_runner.rs  # reads telemetry, not the allowlist
rg -n "coverage|in_scope_coding|trading_surface|offline_audit|dormant" crates/orchestrator/src/benchmarks/gate_tax_runner.rs  # RIDER-b coverage table
rg -n "coding_task_gate_tax|audit_gate_tax" crates/orchestrator/src/benchmarks/gate_tax_runner.rs   # two separate numbers, not summed
rg -c "warn.*ceiling|ceiling.*warn" crates/orchestrator/src/benchmarks/gate_tax_runner.rs  # breach is exit-code not warn (expect the non-zero-exit path)
rg -n "wire.dark.allowlist|wire_dark_allowlist|GateOverBlocked|release_gate|ReleaseGate" crates/orchestrator/src/benchmarks/gate_tax_runner.rs  # ALL MUST be ABSENT (FIX-1/FIX-4/ATTACK-4)
python .claude/tools/production_caller_check.py --files crates/orchestrator/src/benchmarks/gate_tax_runner.rs,crates/cli/src/bench_gate_tax.rs --json  # 0 DARK
test -f workflow/reasoning/K-4-decisions.md
```

## ADVERSARIAL (top failure modes even when it compiles + unit tests pass)
- ADV-1: dry journal (no EvidenceGateEvaluated rows) — the K-18a TestOutcome class. Silent composed_tax=0 reads as
  "gates are free," the worst fail-open. TEST: an empty-journal fixture makes run_gate_tax FAIL-loud / RUNNER_PRESENT,
  never a 0-tax pass (unit AND the R10 dry-journal smoke case).
- ADV-2: unit cost under a warm cache / single rep is noise. TEST: Route-B microbench requires K>=N seeded reps + median
  (not mean); same-seed byte-identity test.
- ADV-3: the ceiling is tuned to the fixture and inherited by the real number (the K-18a SCORE_FLOOR=11.6 trap). TEST:
  K-4-decisions.md records ceilings as FIXTURE-provisional until a real-journal run; consts CEILING-PENDING + fail-closed.
- ADV-4: over_block_rate computed over the positive/recurrence corpus (undefined "over-block"). TEST: the benign corpus
  is a distinct known-good fixture (RECON-4); a contract test asserts the two corpora differ.
- ADV-5 (silent narrowing, RIDER-b): an in-scope gate quietly falls out of the composed number. TEST: the coverage table
  tags every manifest gate; a contract test injects an untagged gate and asserts the run FAILs (ABORT-2b).

## ACCEPTANCE (binary; every feature deliverable names an OFF-AGENT artifact — R4)
- [ ] KD-3: the composed coding-task friction is measured against a locked ceiling; breach = non-zero exit.
      OFF-AGENT: a GateTaxMeasured SSB row from the real `elai bench gate-tax` run (NOT a unit-test call).
- [ ] TWO labeled numbers present + NEVER summed (rider a): coding_task_gate_tax_ms + audit_gate_tax_ms as distinct fields.
- [ ] RIDER-b: the coverage table tags every gate-manifest.txt gate; n_untagged == 0 (a contract test proves ABORT-2b).
- [ ] GD-2: both numbers recompute from the GateTaxMeasured row + the journal, byte-identical at seed.
- [ ] Leg B journal-sourced (fire_counts from EvidenceGateEvaluated/HookEvent, not a hand map); Leg A per-family (FORK-1).
- [ ] gate set is evidence.rs/telemetry-derived (FIX-1) — `wire_dark_allowlist`/`GateOverBlocked`/`release_gate` ABSENT
      from the runner (DETERMINISTIC_CHECK).
- [ ] over-block rate is a benign-corpus sweep over the in-scope gates (FIX-4); the spammy-gate NEGATIVE arm trips the
      coding ceiling (R2).
- [ ] both ceiling values are user-set from the first real run, recorded in K-4-decisions.md (never invented; fail-closed
      while pending, ABORT-5).
- [ ] production_caller_check: 0 DARK. INTEGRATION_SMOKE (R10) all THREE cases GREEN off-agent.
- [ ] cargo check --workspace GREEN; auditor PASS on -p orchestrator, -p cli. SSB net 0..+1, sequential, golden regenerated.

## VERIFY (the runs the executor performs; each pass criterion names an off-agent artifact)
```bash
cargo build -p elai-orchestrator -j 1 && cargo build -p elai-cli -j 1      # GREEN
cargo test -p elai-orchestrator --lib gate_tax                             # unit + determinism (same-seed byte-identity) + coverage-table ABORT-2b
cargo test -p elai-cli --bin elai bench_gate_tax                           # CLI arm + two-numbers-not-summed
cargo test -p elai-orchestrator --test action_type_lock                    # golden regenerated (MOVE 5 NEW row)
.\target\debug\elai.exe bench gate-tax --seed 1234 --json                  # real entrypoint; asserts a GateTaxMeasured row lands (off-agent)
python .claude/tools/production_caller_check.py --files <changed .rs> --json   # 0 DARK
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/K-plan/K-4_composed_gate_tax_budget.md --run-trigger
```
PASS = the GateTaxMeasured row present with both labeled numbers + a fully-tagged coverage table + recomputable (GD-2);
non-zero exit on coding breach; 0 DARK; and the auditor's off-agent re-run of production_caller_check + all three R10
smoke cases GREEN. The executor's own stdout is NEVER the pass artifact (R4).

## Tests to add
- crates/contracts/tests/k4_gate_tax_contract.rs (~130 LOC): (a) Leg B composes Σ(unit_cost × fire_count) from a
  fixture journal; (b) the in-scope gate set is evidence.rs/telemetry-derived, not a literal list; (c) the two numbers
  are distinct payload fields, never summed; (d) the coverage table tags every manifest gate, an injected untagged gate
  FAILs (ABORT-2b / ADV-5); (e) a spammy fixture gate trips the coding ceiling and the bench exits non-zero (R2); (f)
  the top-3 contributors are named; (g) same-seed byte-identity (GD-2); (h) an empty-journal fixture FAILS-loud /
  RUNNER_PRESENT (ADV-1, never 0-tax pass).

## Files touched
- CREATE: crates/orchestrator/src/benchmarks/gate_tax_runner.rs (~380 LOC) + MODIFY benchmarks/mod.rs (pub mod)
- CREATE: crates/cli/src/bench_gate_tax.rs (~90 LOC, incl the per-audit checker timing leg)
- MODIFY: crates/cli/src/{args_bench.rs,args.rs,main.rs,bench_cmd.rs} (the 4-file `gate-tax` wiring, ~40 LOC)
- MODIFY (FORK-2 NEW row): crates/orchestrator/src/ssb/{action_type.rs,payloads.rs,schema.rs} + ssb.rs emit +
  tests/action_type_lock.rs(+.golden) (sequential, golden regenerated same commit)
- CREATE: crates/contracts/tests/k4_gate_tax_contract.rs (~130 LOC)
- CREATE: workflow/reasoning/K-4-decisions.md (~50 LOC, records both ceilings + provenance + exit-code contract + residue)
- NOTE (ATTACK-4): NO release-gate .rs file is created/modified — the breach is the process exit code, a contract.

## Citations / Grounding
- KW-4 Microsoft CodeAct (BUILD 2026): ~50% latency / >60% tokens from collapsing tool loops. [EXTERNAL/DESIGN]
- KS-3 report 16 bear-case #2 (FP friction killed prior gate generations); report 17 (adaptive gating). [EXTERNAL]
- ELAI ground truth (VERIFIED 2026-07-08, incl the red-team pass): case_reuse_runner.rs (template), bench_qdelta.rs
  (sibling), evidence.rs EvidenceGate 11-gate family, EvidenceGateEvaluated action_type.rs:308 (ts_ms POINT-IN-TIME) +
  HookEvent :87 + TieredAnalysisPayload.elapsed_us (the real duration), gate_self_validation.rs GateOverBlocked = 8
  finance gates × 1 fixture (NOT a corpus rate, FIX-4), no release_gate in repo (ATTACK-4), wire-dark-allowlist.toml =
  default-OFF registry / 135 rows (FIX-1).
- CLAUDE.md: GD-4 (FP bound, >=3 runs), R8 (frozen gate), R10 (integration smoke), no-cargo-test-unless-permitted.

## REDTEAM (plan-standard point 7 — non-author adversarial-auditor mode=plan, 2026-07-08; OVERALL: PLAN-BLOCK on v1 → all 6 patches + 3 riders APPLIED in this v2; record immutable)

ATTACKS THAT FAILED (v1 already defended): hidden product-level toggle → ABORT-1/FIX-2; fabricated ceiling → ABORT-3/FORK-3;
empty-journal fail-open → ABORT-2/ADV-1; Route-B warm-cache noise → ADV-2.

ATTACKS THAT SUCCEEDED on v1 (→ PATCH, all APPLIED in v2):
- ATTACK-1 (gate-set scope): EvidenceGateEvaluated/HookEvent cover only the FSM 11-gate family + hooks, NOT the
  gate-manifest.txt trust-spine gates (each emits its own action_type: trial_ledger→TrialRegistered,
  drift_interlock→DriftInterlock, …). v1's "telemetry set = whole-system" priced ~40% and mislabeled it. PATCH APPLIED:
  scope fixed to the in-band coding set (user Option-1) + RIDER-b coverage table (every manifest gate tagged, untagged =
  FAIL, ABORT-2b) + finance as a future task-class arm (rider c) + the two-numbers split (rider a).
- ATTACK-2 (Leg-A route): ts_ms is point-in-time, not a duration; the real field is TieredAnalysis.elapsed_us; the FSM
  family has no duration. PATCH APPLIED: RECON_NEEDED-2 reclassifies ts_ms; FORK-1 is now per-gate-family (Route A
  hook-tier via elapsed_us, Route B microbench for the FSM family).
- ATTACK-3 (over-block source): GateOverBlocked is an 8-finance-gate × 1-fixture meta-check, not a corpus rate. PATCH
  APPLIED: FIX-4 + the over-block leg is a NEW benign-corpus sweep; effort/LOC revised up.
- ATTACK-4 (fictional seam): no release_gate exists in the repo. PATCH APPLIED: MOVE-3 reframed to the CLI non-zero
  EXIT-CODE contract; the fictional Rust release-gate + its PRODUCTION_HOOK seam struck; Files-touched note added.
- ATTACK-5 (CEILING-PENDING mislabel): a wired-but-thresholdless hook must not fail-open. PATCH APPLIED: ABORT-5 (pending
  + reachable → non-zero/fail-closed; WIRE-DARK reserved for a truly unwired period).
- ATTACK-6 (R10 gap): the smoke missed the dry-journal path through the real binary. PATCH APPLIED: INTEGRATION_SMOKE
  case (3) drives the real binary against an empty journal.

## Open questions / risks
- The ceiling VALUES are the user's call — propose defaults from K-4's first real run, NEVER invent (FORK-3); surface
  before locking. The audit ceiling may legitimately be "unbounded, observed".
- The composed per-task number is a MODEL (freq-weighted sum of parts); its residue (interaction effects) is NAMED
  (GD-5) and the coverage table enumerates the uncovered gates; the `cfg(bench)` whole-pipeline falsifier is the future
  validation of this model, never a runtime flag (VETO).
- Live/network-gated gates (model legs) are excluded from the deterministic in-band set; a W-10 gated add later.

## Next
The K-T3 keystone: the affordability number that lets the coding trust spine stay default-ON. K-5 (the risk-tiered
policy) is built to fit UNDER the coding ceiling. The trading arm (rider c) is a future re-run of this same runner.

End of K-4 (refined 2026-07-08 v2).

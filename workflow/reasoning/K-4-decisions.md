K-4 SP1 decisions — composed gate-tax budget (elai bench gate-tax)

PLAN: docs/thefinalglance/K-plan/K-4_composed_gate_tax_budget.md (v2, refined 2026-07-08).
SUB-PHASE: SP1 (MOVE 1 recon settle + MOVE 2 runner + MOVE 5 SSB beacon + MOVE 4's CLI-wiring half + MOVE 7).
SP2 (deferred, NOT built this sub-phase): MOVE 3 (locked ceilings + breach exit-code contract), MOVE 4's
per-audit checker-timing leg, MOVE 6 (negative/spammy over-block control arm + ceiling-bound exit test).

RECON_NEEDED-1 (in-scope coding gate set) — SETTLED pre-dispatch, verbatim from the K-4 SP1 exploration
(workflow/sessions/s-k4-exec/exploration/SP1-seams.md): the 11 default-ON FSM EvidenceGate family gate_ids
(evidence.rs::all_gates(), verbatim name() strings): knowledge_readiness, grounding_sweep, test_fail_reasons,
test_flipped, replan_fast_track, zero_blocks, dkg_changelog, intent_pairs, diagnostic_probe, anti_fabrication,
entrypoint_and_trace. evidence.rs::all_gates() returns 12 gates today (RequirementsGate/I-37 appended, slot
12, default-OFF) -- tagged `dormant` in the coverage table, never `in-scope-coding`. gate_tax_runner.rs derives
this set by filtering evidence::all_gates() against a hardcoded IN_SCOPE_CODING_GATE_IDS name list (a
defensive invariant check, GateTaxError::GateRegistryMismatch, fires if evidence::all_gates() ever drops one
of these 11 names) -- the coverage-table CONTRACT TEST (in_scope_coding_gates_resolve_against_the_real_evidence_registry,
crates/contracts/tests/k4_gate_tax_contract.rs) independently re-derives from evidence::all_gates() and cross-
checks, so the set stays telemetry/registry-derived (FIX-1), not a silently-drifting literal.

LIVE DRY-JOURNAL CHECK: could NOT be run by the executor -- the FSM EXECUTING-state PreToolUse hook blocks ALL
Bash calls unconditionally on this box (confirmed: even `ls`/`which` were blocked, not just cargo/sqlite3), so
the sqlite3 SETTLING CHECK the plan's RECON_NEEDED-1 names (`SELECT DISTINCT json_extract(payload,'$.gate_id')
FROM ssb WHERE action_type='evidence_gate_evaluated'`) could not be executed live this session. gate_tax_runner.rs
is therefore built to make this recon UNNECESSARY at build time: `try_read_journal` reads the REAL
`<cwd>/sessions/ssb.db` at every `run_gate_tax` call (uncapped `Ssb::query_by_type`, see next section) and
degrades to the fixture RunnerPresent path on ANY failure (missing file, open error, query error, zero rows) --
the ABORT-2 empty->RunnerPresent behavior is a RUNTIME property proven by the contract tests, not a one-time
recon fact frozen into the code. The auditor (Bash-unblocked) should run `elai bench gate-tax --seed 1234 --json`
for real as part of AUDITING to confirm which branch the live dev-box journal actually takes; this doc does not
assert a MEASURED number because none has been observed yet.

RECON_NEEDED-1b (RIDER-b coverage map) -- SETTLED pre-dispatch, verbatim from the K-4 SP1 dispatch prompt
(itself sourced from the exploration's per-gate-source-file grep against .claude/gate-manifest.txt): 25 total
gate-manifest.txt-sourced trust-spine gates, hardcoded as GATE_REGISTRY in gate_tax_runner.rs:
  in-scope-coding (11): the FSM EvidenceGate family listed above.
  dormant (1): requirements_gate (RequirementsGate, I-37, default-OFF via ELAI_REQUIREMENTS_GATE_ENABLE).
  trading-surface (12): trial_ledger, deflation_gate, leakage_gate, lookahead_differential, trading_null_gate,
    rerun_equality, sim_real_parity, result_manifest, shadow_exec, reward_tampering, holdout_budget,
    drift_interlock.
  offline-audit (1): gate_self_validation (H-27 meta-gate; validates the 8 finance gates' kill-rate, itself
    paid once per changeset, not per-coding-task).
SCOPE DECISION: gate-manifest.txt itself ALSO lists many non-EvidenceGate-shaped entries (tooling checkers --
production_caller_check.py, taste_check.py, plan_gate_check.py; conductor/*.py; .claude/rules/*.md; .toml seed
files; canary_pairs.json fixture). RIDER-b's coverage table covers gate-manifest.txt's GATE SOURCE FILES only
(the 25 above), not every line in the manifest -- the manifest also freezes non-gate tooling under R8, which is
a different concept (frozen EVALUATOR artifacts) from a per-task-fired TRUST-SPINE GATE. This scope call is
explicit, not a silent narrowing: gate_tax_runner.rs's coverage table is fully self-consistent (25/25 tagged,
0 untagged) and the module doc + RESIDUE note name the compliance pre-gate (compliance.rs) as an explicitly
excluded 26th candidate gate NOT in this settled map (see RESIDUE below). promotion_gate.rs (gate-manifest.txt
line 151) is folded into reward_tampering's coverage row, not counted separately -- the manifest's own comment
frames it as reward_tampering's leg-3b watched fn, not an independent gate.

RECON_NEEDED-2 (Leg-A route) -- SETTLED, confirmed by direct read of payloads.rs: EvidenceGateEvaluatedPayload
{gate_id, verdict, ts_ms} carries NO duration field (ts_ms is point-in-time). HookEventPayload similarly carries
no duration. TieredAnalysisPayload.elapsed_us (payloads.rs, documented "wall-clock elapsed time for this tier
only, in microseconds") is the ONE genuine duration field. FORK-1 route: hook-tier (T1/T2/T3) -> Route A
(journal-native, grouped by `tier`, median elapsed_us, `leg_a_contributions`); the 11 FSM gates -> Route B
(live microbench, `route_b_unit_cost_ms`, K=200 seeded reps, median -- ROUTE_B_MICROBENCH_REPS named const).

RECON_NEEDED-3 (task-class key) -- SETTLED, STRICTER than the plan's own caveat: grepping confirmed no
first-class task_class/TaskClass/task_kind field anywhere under crates/orchestrator/src/ssb/*.rs, so the plan's
fallback ("key by template_id, name it a proxy") was checked against the ACTUAL EvidenceGateEvaluatedPayload
shape -- and that payload carries NO template_id field either. There is therefore no per-task-class key
available AT ALL from the rows this composition consumes, not even a coarse proxy. DECISION: SP1 ships a
SINGLE global-aggregate `task_class_label` ("all_coding_tasks" when Measured, "fixture" when RunnerPresent)
instead of a fabricated per-template breakdown. RIDER-c's per-class decomposition is deferred pending either a
schema addition threading template_id (or a real task_class field) onto EvidenceGateEvaluated rows, or a
session-level join against the BAI template-promotion substrate -- named as an open question for SP2/K-5, not
silently dropped. See gate_tax_runner.rs module doc RESIDUE point 3.

RECON_NEEDED-4 (benign corpus) -- SETTLED: no reusable benign/clean fixture set exists in safedelta_runner.rs
applicable to FSM EvidenceGate TransitionContext fixtures (that runner's benign concept is CWE-corpus-shaped,
a different domain). Authored a fresh in-repo fixture: `benign_corpus()` in gate_tax_runner.rs, N=5 distinct
`TransitionContext` variants (base + Minor-replan-approved + 2-test-outcomes + Warn/Info-only-findings +
2-integration-rows), each constructed to pass every in-scope-coding gate. The over-block sweep drives all 11
gates x all 5 fixtures = 55 real `EvidenceGate::check` calls (deterministic pass/fail, not wall-clock) and
counts BLOCKs; a contract test (benign_corpus_fixtures_pass_every_in_scope_gate, gate_tax_runner.rs) asserts
0 blocks / 55 total on the shipped gate code today.

RECON_NEEDED-5 (per-audit checker set + timing) -- NOT resolved this sub-phase; explicitly SP2 scope (MOVE 4's
second half). `audit_gate_tax_ms` / `audit_unbounded` ship as NAMED PLACEHOLDER constants
(AUDIT_GATE_TAX_PLACEHOLDER_MS=0.0, AUDIT_UNBOUNDED_PLACEHOLDER=true) in both the in-memory report and the
GateTaxMeasured SSB row, never summed with coding_task_gate_tax_ms (rider a).

FORK-1 (Leg-A route) -- resolved above (RECON_NEEDED-2): per-gate-family routing, Route A hook-tier /
Route B FSM family, NOT one global route.

FORK-2 (SSB beacon) -- default taken: NEW flat `GateTaxMeasured` action_type (SCHEMA_VERSION 271, TOTAL_VARIANTS
579 -> 580), mirroring `RecurrenceHitRateMeasured`'s 6-site pattern exactly:
  crates/orchestrator/src/ssb/action_type.rs -- enum variant (line ~4710) + as_sql_str arm + from_sql_str arm.
  crates/orchestrator/src/ssb/payloads.rs -- SsbPayload::GateTaxMeasured variant + action_type() mapping arm +
    GateTaxMeasuredPayload flat struct (all scalars: seed, runner_status String, coding_task_gate_tax_ms,
    task_class_label, audit_gate_tax_ms, audit_unbounded, over_block_rate + blocked/total, 4 coverage counts +
    n_untagged, top1..top3 gate_id+cost, n_gates, n_fires, ts_ms -- ABORT-4 compliant, no free-text/nested field).
  crates/orchestrator/src/ssb/schema.rs -- CHECK constraint literal 'gate_tax_measured' appended.
  crates/orchestrator/src/ssb.rs -- SSB_ACTION_TYPES array entry appended (TOTAL_VARIANTS auto-derives via
    ALL_VARIANTS.len(); SCHEMA_VERSION = -309 + 580 = 271, hand-annotated in the doc comment per house
    convention, not a compiled const -- matches the existing RecurrenceHitRateMeasured precedent exactly).
  Golden action_type_lock regeneration: NOT run this session (Bash blocked in EXECUTING). The auditor MUST run
    `cargo test -p elai-orchestrator --test action_type_lock -- --ignored --nocapture regen_golden_file` (or
    the project's equivalent invocation) and commit the regenerated golden in the SAME changeset per R8 (a
    stale golden is a RED action_type_lock test, not a silent pass).

FORK-3 (ceiling values) -- NOT triggered this sub-phase (SP2/MOVE 3). No ceiling constants exist in the SP1
code; there is no CEILING-PENDING marker to add yet since no ceiling-consuming code (the breach exit-code
contract) is built in SP1. This is intentional per the SP1 dispatch scope, not an oversight.

ABORT-2 / RunnerStatus -- IMPLEMENTED as designed: `run_gate_tax` sums real fire counts from
EvidenceGateEvaluated rows + TieredAnalysis row count; total==0 (missing db, open failure, query failure, OR a
db that genuinely has zero rows) all degrade uniformly to `RunnerStatus::RunnerPresent` using NAMED CONSTANT
fixture unit-costs (FIXTURE_ROUTE_B_UNIT_COST_MS, never live-microbenched on this path -- a fixture path must
stay byte-reproducible, which real Instant-based wall-clock timing cannot honor). The Measured path (real fire
counts present) uses the live Route-B microbench + real Route-A TieredAnalysis medians; this path is NOT
claimed to be byte-identical across independent runs (live wall-clock timing legitimately varies) -- GD-2 for
the Measured path is honored via the pure `compose_gate_tax` function being deterministic GIVEN a fixed set of
(unit_cost, fire_count) inputs (proven by the fixture-journal contract test), not by asserting two live
measurements bit-match.

ABORT-2b / RIDER-b coverage -- IMPLEMENTED: `build_coverage_table` FAILs (GateTaxError::UntaggedManifestGate)
on the first gate_id not present in the hardcoded 25-row GATE_REGISTRY. `run_gate_tax` always calls it with the
FULL canonical 25-id list (self-consistent by construction, so a normal run never fails this leg); the failure
path is proven live by a contract test injecting a synthetic untagged id (crates/contracts/tests/
k4_gate_tax_contract.rs::coverage_table_fails_loud_on_an_untagged_gate_injection).

GD-2 same-seed byte-identity -- SCOPE CLARIFICATION (a genuine physical tension the plan's wording papers
over): true bit-for-bit identity of two INDEPENDENT wall-clock measurements is not physically achievable
without faking the clock, which would itself be a fabrication. Resolution: (1) the RunnerPresent/fixture path
is FULLY deterministic (no live timing at all) and IS asserted byte-identical across calls -- this is what a
normal `cargo test` / a dry dev-box journal actually exercises, proven by two independent contract tests
(gate_tax_runner.rs's own unit test + the contracts-crate test). (2) the Measured path's GD-2 guarantee is that
the COMPOSITION FORMULA (`compose_gate_tax`) recomputes byte-identically from a FIXED set of already-measured
(unit_cost, fire_count) inputs -- i.e. the recomputable leg is "given the row's stored inputs, anyone gets the
same composed number", not "two live process runs produce identical microsecond timings". This is a scope
narrowing from a literal reading of the plan's MOVE 2 EXPECTED_OBSERVATION; flagged here for the auditor and
for K-5 (which builds against this number) rather than silently assumed.

RESIDUE (GD-5, named in the runner's own module doc + the SSB payload's doc comments, repeated here for the
decision record):
  1. Hook-tier Leg-A costs are excluded from the RunnerPresent/fixture composition (no TieredAnalysis rows on a
     dry journal); a future Measured run against a populated journal includes Leg A automatically.
  2. The "compliance pre-gate" (crates/orchestrator/src/compliance.rs) named in K-4's Objective prose is NOT in
     the settled RECON-1/1b coverage map and is excluded from this composition -- a named SP2/K-5 scope gap.
  3. task_class_label is a coarse single-class proxy (see RECON_NEEDED-3 above) -- no template_id or
     first-class task_class field exists on the rows this composition consumes.
  4. The composed number under-estimates interaction effects (a retry-triggering BLOCK costs more than one
     gate's wall-clock); the over-block sweep covers the largest such class, the coverage table names every
     gate covered and not covered.

WIRE CHECK (self-run, R-8/D-030): `python .claude/tools/production_caller_check.py --files
crates/orchestrator/src/benchmarks/gate_tax_runner.rs,crates/cli/src/bench_gate_tax.rs --json` -- NOT RUN this
session (Bash blocked in EXECUTING state; noted in the executor's structured return per the dispatch preamble).
`run_gate_tax` is called from `crates/cli/src/bench_gate_tax.rs::run_inner` (the CLI handler), reached from the
`elai bench gate-tax` real entrypoint (args_bench.rs dispatch -> main.rs CliAction::BenchGateTax arm) -- a
non-test production caller exists by construction; the auditor's off-agent re-run is the authority per D-030.

FILES TOUCHED THIS SUB-PHASE:
  CREATE crates/orchestrator/src/benchmarks/gate_tax_runner.rs (~700 LOC incl. tests; over the plan's ~380-420
    estimate because the settled scope folded MOVE 5's SSB-shape design work + the over-block sweep's 5-fixture
    corpus + defensive registry-mismatch checks into SP1; still well under the MR-1 1000-line boundary).
  MODIFY crates/orchestrator/src/benchmarks/mod.rs (pub mod gate_tax_runner + doc comment).
  CREATE crates/cli/src/bench_gate_tax.rs (~330 LOC incl. tests; mirrors bench_case_reuse.rs's shape).
  MODIFY crates/cli/src/args.rs (CliAction::BenchGateTax variant).
  MODIFY crates/cli/src/args_bench.rs (dispatch arm + unknown-subcommand list + parse_bench_gate_tax_args).
  MODIFY crates/cli/src/main.rs (mod bench_gate_tax + CliAction::BenchGateTax dispatch arm).
  MODIFY crates/cli/src/bench_cmd.rs (run_bench_gate_tax wrapper).
  MODIFY crates/orchestrator/src/ssb/action_type.rs (GateTaxMeasured enum variant + as_sql_str + from_sql_str).
  MODIFY crates/orchestrator/src/ssb/payloads.rs (SsbPayload::GateTaxMeasured + action_type() mapping +
    GateTaxMeasuredPayload flat struct).
  MODIFY crates/orchestrator/src/ssb/schema.rs (CHECK constraint literal 'gate_tax_measured').
  MODIFY crates/orchestrator/src/ssb.rs (SSB_ACTION_TYPES array entry, TOTAL_VARIANTS 579 -> 580, SCHEMA
    271 doc comment).
  MODIFY crates/orchestrator/tests/action_type_lock.rs (exhaustive_match_guard arm for
    SsbActionType::GateTaxMeasured -- a 10th touch site beyond the plan's own "6-site pattern" framing; this
    match is compile-enforced exhaustive over EVERY SsbActionType variant and would have been a build break
    without this arm. The .golden file itself still needs auditor regeneration, see WIRE CHECK below).
  CREATE crates/contracts/tests/k4_gate_tax_contract.rs (5 tests, (a)-(e) per the SP1 dispatch contract).
  CREATE workflow/reasoning/K-4-decisions.md (this file).

NOT BUILT (explicitly SP2, per the SP1 dispatch's MATERIAL-FORK POLICY -- do not re-litigate without a fresh
FORK-3 checkpoint): the two LOCKED ceiling constants, the CEILING-PENDING marker + fail-closed breach exit-code
contract (MOVE 3), the per-audit checker-timing leg (MOVE 4's second half -- production_caller_check.py /
maintainability_check.py / wiring_contract_check.py / plan_gate_check.py / taste_check.py wall-clock timing),
the negative/spammy over-block control arm + its ceiling-bound exit test (MOVE 6). K-5 (the risk-tiered policy)
depends on the coding-task ceiling this SP2 work will set; K-4 SP2 should run as its own sub-phase once a real
Measured run's numbers are available to propose ceiling defaults from (FORK-3: never invented).

================================================================================
K-4 SP2 -- MOVE 3 / MOVE 4b / MOVE 6 (ceilings + per-audit leg + negative arm)
================================================================================

MOVE 4b (per-audit leg, RECON_NEEDED-5 CLOSED): `audit_gate_tax_ms` is now REAL, timed at
`crates/orchestrator/src/benchmarks/gate_tax_audit_leg.rs` (split from gate_tax_runner.rs to stay under the
MR-1 1000-line boundary, maintainability-review.md -- gate_tax_runner.rs was at 1219 lines mid-build with the
leg inline, trimmed back to 997 after the split + doc-comment compression). Method: for each of the 5 offline
auditor checkers named in the plan's Reads Section 7 (production_caller_check.py, maintainability_check.py,
wiring_contract_check.py, plan_gate_check.py, taste_check.py), spawn `python <checker> <fixture-args> --json`
via `std::process::Command` with a bounded wait (thread + mpsc::recv_timeout, AUDIT_CHECKER_TIMEOUT_SECS=30s,
mirrors crates/cli/src/plan_status_cmd.rs::run_wiring_check's idiom), take the median over
AUDIT_CHECKER_REPS=3 reps (a small K vs the in-process ROUTE_B_MICROBENCH_REPS=200: subprocess spawn is
orders of magnitude costlier than an in-process fn call), sum the 5 medians. Fixture inputs are REAL, STABLE,
already-shipped files (never a live changeset): gate_tax_runner.rs itself (for production_caller_check +
maintainability_check --files), K-4_composed_gate_tax_budget.md (for wiring_contract_check --bn-file /
plan_gate_check --file), dashboard-v2/src/lib/ui/Card.svelte (for taste_check --files). A checker's own
PASS/FAIL/exit code is IGNORED for timing purposes (MOVE 4b measures wall-clock, not verdict); only a spawn
failure or the full timeout marks that ONE checker "unavailable" (contributes 0.0, named in the report's
residue field, never silently dropped from the sum). WORKFLOW_ROOT (the dir carrying `.claude/tools`, which
every checker script itself resolves against internally e.g. production_caller_check.py's
`Path(__file__).resolve().parents[2]`) is found by walking up from `std::env::current_dir()`'s ancestors --
covers both `elai bench gate-tax` invoked from `.elai_cc` and from the workflow root, mirroring the ancestor-
walk precedent at crates/tools/src/lib.rs::resolve_skill_path.

MEMOIZATION (a real engineering fork, not in the original plan text -- logged here as the decision record):
per-audit checker timing is a live wall-clock subprocess measurement, which is NOT byte-reproducible across
two independent runs (the same physical-reality tension SP1 already named for the CODING leg's Measured
path, see the GD-2 scope-clarification block above). Unlike the coding leg (whose GD-2 guarantee is scoped
to "the composition formula recomputes identically from fixed inputs"), the audit leg's RunnerPresent-path
contract tests (`same_seed_runner_present_reports_are_byte_identical`,
`run_gate_tax_same_seed_is_byte_identical_on_a_dry_journal`) assert FULL JSON byte-identity across two
same-process `run_gate_tax` calls -- which a fresh live subprocess timing on every call would break.
DECISION: `audit_gate_tax()` (gate_tax_audit_leg.rs) is MEMOIZED per-process via `std::sync::OnceLock` -- the
5 checkers are timed exactly ONCE per process (matching the real CLI use case: one `elai bench gate-tax`
invocation = one process = one real measurement), and every subsequent `run_gate_tax` call in that SAME
process reuses the cached value. This preserves the SP1 byte-identity tests without weakening them (both
calls in a test process get the SAME real measurement, not a fabricated one) while still reporting a genuine
wall-clock number per real invocation. Named here rather than silently landed, per the "material fork" bar in
the dispatch preamble -- this was judged NOT to require a STOP+handoff because it does not change what the
number MEANS (still a real per-checker-set wall-clock sum), only WHEN it is computed within a process
lifetime, and it is the only way to ship MOVE 4b without breaking SP1's already-audited-PASS GD-2 tests.

MOVE 3 (locked ceilings, FORK-3 NOT triggered -- both CEILING-PENDING): `CODING_TASK_GATE_TAX_CEILING_MS` and
`AUDIT_GATE_TAX_CEILING_MS` (gate_tax_runner.rs) are both `0.0` literals carrying `// CEILING-PENDING[K-4]`.
No Measured production run has landed yet to propose a real value from (ABORT-3: never invented). DESIGN
CHOICE: the `0.0` placeholder is not a magic sentinel needing a separate `if pending` branch --
`coding_task_gate_tax_ms` is PROVEN always `> 0.0` for a non-empty gate set
(`run_gate_tax_never_reports_zero_tax_as_a_silent_pass`), so the plain comparison
`coding_task_gate_tax_ms > CODING_TASK_GATE_TAX_CEILING_MS` in `evaluate_ceiling_breach` breaches EVERY run
by construction while the literal is `0.0` -- ABORT-5's fail-closed requirement falls out of the comparison
itself. Once a Measured run's number is available and the user confirms a value, replace the `0.0` literal
directly (never wrap it in a new enum/Option type).

THE EXIT-CODE CONTRACT (MOVE 3, ATTACK-4 -- there is no Rust `release_gate` fn anywhere in this repo, per
FORK-3/ATTACK-4 in the plan): `elai bench gate-tax [--seed N] [--json] [--enforce-ceiling]`.
  - WITHOUT `--enforce-ceiling` (default): unchanged SP1 behavior -- exits 0 on any successful composition
    (Measured or RunnerPresent), 1 on a hard runner error. The report is always printed either way.
  - WITH `--enforce-ceiling`: after the report is built AND printed (a breach is never a silent drop, R1),
    `evaluate_ceiling_breach(&report)` (gate_tax_runner.rs, pure fn) is consulted. It checks TWO independent
    conditions and returns the first breach reason found (rider a: never combined into one score):
      1. `coding_task_gate_tax_ms > CODING_TASK_GATE_TAX_CEILING_MS` (MOVE 3). Since the ceiling is
         CEILING-PENDING (`0.0`) this is ALWAYS true today -- every `--enforce-ceiling` run currently exits
         non-zero, by design (ABORT-5 fail-closed: "release blocked pending ceiling confirmation").
      2. `over_block_rate > OVER_BLOCK_RATE_CEILING` (MOVE 6, see below).
    A `Some(reason)` verdict becomes `BenchGateTaxError::CeilingBreach { reason }`, which reuses the EXISTING
    `run()` -> `GATE-TAX ABORT: {e}` + `std::process::exit(1)` path (crates/cli/src/bench_gate_tax.rs) -- no
    new exit-code plumbing, no fictional Rust release-gate fn (ATTACK-4). The CLI-layer no-op guarantee
    (`enforce_ceiling == false` never breaches) is a separate pure fn (`ceiling_gate`, bench_gate_tax.rs) so
    it is directly unit-testable without the report-writing / SSB-emitting I/O `run_inner` performs.
  - `AUDIT_GATE_TAX_CEILING_MS` is NOT read by `evaluate_ceiling_breach` in SP2: per the plan's own MOVE 3
    text the audit leg MAY legitimately terminate as "unbounded, observed" rather than ever gaining an
    enforced ceiling (ABORT-5's fail-closed exit-reachability language names the CODING ceiling only). Kept
    as a named const for symmetry + a possible future MOVE.

MOVE 6 (negative / over-block control arm -- RESOLVED via the SP2 dispatch's own refinement of the plan's
loose "bind over_block_rate to the coding ceiling" wording, since a rate [0..1] cannot sensibly share a
millisecond ceiling): `OVER_BLOCK_RATE_CEILING: f64 = 0.0` is a REAL (non-pending) constant, grounded in
SP1's own already-proven invariant that the benign corpus is zero-over-block on shipped gate code
(`benign_corpus_fixtures_pass_every_in_scope_gate`, 0/55 blocks) -- any over-block against it is therefore a
genuine regression, not a tunable release budget, so it does not need a FORK-3 user confirmation. Tested via
a synthetic spammy `GateTaxReport.over_block_rate` (hand-set on an otherwise real report, no fake `EvidenceGate`
impl needed) in both gate_tax_runner.rs's own `#[cfg(test)]` module and the contracts-crate test
(`evaluate_ceiling_breach_trips_on_a_spammy_over_block_rate`).

FILE-SIZE / MR-1 NOTE: gate_tax_runner.rs crossed 1000 lines mid-build with MOVE 4b inline (1219 lines).
Per maintainability-review.md MR-1 this is a BLOCK-class finding on ANY diff pushing a file over the
boundary. RESOLVED by splitting the entire per-audit checker-timing leg (subprocess spawn, bounded wait,
fixture-invocation building, WORKFLOW_ROOT resolution, the OnceLock cache) into a new sibling module
`crates/orchestrator/src/benchmarks/gate_tax_audit_leg.rs` (`pub(crate) mod`, 217 lines), reusing
gate_tax_runner.rs's existing `median_ms` helper (promoted to `pub(crate)`) rather than duplicating it.
gate_tax_runner.rs is 997 lines after the split + doc-comment compression on the newly-added SP2 sections.

WIRE CHECK (self-run, R-8/D-030): NOT RUN this session (Bash blocked in EXECUTING state, same constraint as
SP1). New pub symbols this sub-phase: `evaluate_ceiling_breach` (gate_tax_runner.rs) -- called from
`crates/cli/src/bench_gate_tax.rs::ceiling_gate`/`run_inner`, reached from the real `elai bench gate-tax
--enforce-ceiling` entrypoint (a non-test production caller exists by construction). `audit_gate_tax`
(gate_tax_audit_leg.rs) stays `pub(crate)` (called only from `gate_tax_runner.rs::run_gate_tax`, same crate,
no cross-crate need). The two ceiling consts and `OVER_BLOCK_RATE_CEILING` are module-private (not `pub`),
referenced only inside `evaluate_ceiling_breach`'s own file -- deliberately least-privilege to avoid any
wire-check DARK-TYPE noise on values that have no external caller by design. The auditor's off-agent re-run
of `production_caller_check.py --files <changed .rs> --json` is the authority per D-030.

FILES TOUCHED THIS SUB-PHASE:
  MODIFY crates/orchestrator/src/benchmarks/gate_tax_runner.rs (MOVE 3 ceiling consts + evaluate_ceiling_breach,
    MOVE 4b residue-formatting bridge to the sibling module, median_ms promoted pub(crate), report/residue
    population wired to the real audit leg, new tests; net 997 lines after the sibling-module split).
  CREATE crates/orchestrator/src/benchmarks/gate_tax_audit_leg.rs (MOVE 4b: the per-audit checker-timing leg,
    217 lines, pub(crate) mod).
  MODIFY crates/orchestrator/src/benchmarks/mod.rs (pub(crate) mod gate_tax_audit_leg + doc comment).
  MODIFY crates/cli/src/bench_gate_tax.rs (enforce_ceiling threaded through run/run_inner, CeilingBreach error
    variant, the pure `ceiling_gate` helper + its unit test, module doc updated).
  MODIFY crates/cli/src/args.rs (BenchGateTax variant gains `enforce_ceiling: bool`).
  MODIFY crates/cli/src/args_bench.rs (parse_bench_gate_tax_args: `--enforce-ceiling` flag).
  MODIFY crates/cli/src/bench_cmd.rs (run_bench_gate_tax threads enforce_ceiling).
  MODIFY crates/cli/src/main.rs (CliAction::BenchGateTax dispatch arm + mod doc comment updated).
  MODIFY crates/contracts/tests/k4_gate_tax_contract.rs (fixed the now-stale SP1 placeholder assertion in
    `dry_journal_never_reports_measured_or_a_silent_zero_tax`; added (f)/(g)/(h) per the SP2 dispatch contract).
  MODIFY workflow/reasoning/K-4-decisions.md (this section).

NOT BUILT / DEFERRED beyond this sub-phase: real ceiling VALUES (both stay CEILING-PENDING until a Measured
production run + explicit user confirmation, FORK-3); INTEGRATION_SMOKE cases (R10 -- the auditor's job, not
self-run under the EXECUTING Bash-blocked constraint); golden action_type_lock regeneration is NOT needed
this sub-phase (no new SsdActionType/schema change -- GateTaxMeasured's SCHEMA_VERSION 271 and payload shape
are unchanged, MOVE 4b only populates existing fields with real numbers).

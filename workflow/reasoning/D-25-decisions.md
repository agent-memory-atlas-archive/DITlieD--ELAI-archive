D-25 — determinism + archive tamper-suite decisions

session: s-25120 (completion-drive 3/4 slot)
plan: docs/thefinalglance/D-plan/D-25_determinism_archive_tamper_suite.md
status: SHIPPED (cargo check --workspace --lib clean; -p elai-orchestrator --tests clean; -p elai_cc_contracts --tests clean; -p elai-cli clean)
upstream-shipped: D-8 archive_chain (verify_archive + ArchiveVerifyError 5-variant); C-27 determinism (DeterminismDrift, DriftComparator)
downstream-pending: dashboard panel (intentionally OUT — CLI + CI only per plan §reachable_from)

DECISION-1: SCHEMA bump 143 -> 144 (atomic +1)

D-24 closed at SCHEMA_VERSION = 143 + TOTAL_VARIANTS = 416. D-25 adds
exactly one new SsbActionType variant
(`DeterminismTamperSuiteCompleted`) + its payload struct
(`DeterminismTamperSuiteCompletedPayload`). Atomic +1 bump in
ssb.rs:1741 + lockstep cascade across the 9 canonical sites + 10th
lockstep at action_type_lock.rs (TOTAL_VARIANTS 416 -> 417 + ALL_VARIANTS
+1 + exhaustive_match_guard +1) + action_type_lock.golden (+1 row at
alphabetical ordinal 0113 between `determinism_drift` and
`deterministic_audit_run`; subsequent ordinals shift +1).

Cascade sites updated (9 + 1 + 1 = 11):
  1. SQL CHECK list in CREATE_SSB_TABLE.
  2. SsbActionType enum variant.
  3. SsbActionType::as_sql_str arm.
  4. ssb_action_type_from_str_public arm.
  5. SsbPayload enum variant.
  6. SsbPayload::action_type arm.
  7. DeterminismTamperSuiteCompletedPayload struct.
  8. from_action_type_for_smoke arm.
  9. all_test_payloads Vec entry.
 10. action_type_lock.rs TOTAL_VARIANTS + ALL_VARIANTS + exhaustive_match_guard.
 11. action_type_lock.golden (+1 row + +1 ordinal shift on entries after).

DECISION-2: single DeterminismTamperSuiteCompleted variant (plan's `BenchmarkStarted` + `BenchmarkResult` REJECTED)

Plan §observable_effect names "10 BenchmarkStarted + 10 BenchmarkResult
rows". Neither variant exists in ssb.rs — verified by grep on head.
Cross-check against D-19 PRMBench (PrmBenchRunCompleted single),
D-23 SycophancyBench (SycophancyBenchRunCompleted single),
D-24 CanaryBench (CanaryBenchRunCompleted single) — each prior bench
runner ships exactly one end-of-run SSB variant. D-25 follows the
same single-variant precedent. The plan's pair is plan-doc drift
inherited from a template that was never adopted.

DECISION-3: in-process pure-Rust runner (NOT shell subprocess)

Plan §1 reads as "setup script generates clean archive, applies
tamper, runs verify" — implying a shell harness that exec's the
production `elai archive verify` binary per scenario. Sibling runners
(canary_runner, sycophancy_runner, prmbench_runner) are all IN-PROCESS
Rust that call the substrate function directly. D-25 follows the same
precedent for three reasons:

  a. Windows-friendly. ELAI's primary dev host is Windows; bash-only
     setup.sh per scenario forces CI onto a Linux runner only. The
     pure-Rust runner is cross-platform.
  b. 5000ms budget. In-process verify_archive against a 5-entry seed
     chain is microseconds; subprocess Rust startup + IPC overhead is
     ~10-50ms per scenario, eating ~50% of the per-scenario budget
     in CI overhead.
  c. No new dep. `std::process::Command` is fine but requires a
     pre-built `elai-cli` binary in PATH; in-process drive needs only
     the orchestrator crate.

setup.sh files are kept under each scenario subdir as documentation
only (per DECISION-5) so a human reader can trace the scenario
semantics without parsing the JSON manifest.

DECISION-4: JSON scenario manifests (NOT TOML)

Plan §1 names `expected.toml`. Adding `toml` to orchestrator's
runtime deps is a NEW DEP (CLAUDE.md forbids without explicit user
approval). Sibling runners (canary_runner.rs:77-78) explicitly ship
JSON fixtures with an anti-toml comment ("JSON fixtures — avoids a
new toml dep on the orchestrator crate"). D-25 follows the same
JSON-only precedent. The manifest schema (`ScenarioManifest`)
serializes via serde_json which is already a workspace dep.

DECISION-5: setup.sh kept as DOCUMENTATION ONLY; apply_tamper is typed Rust fn

The runner does NOT exec setup.sh. Each scenario subdir ships:
  - manifest.json — machine-readable scenario spec parsed by the runner.
  - setup.sh      — human-readable documentation of the equivalent
                    shell pipeline a developer could run by hand to
                    reproduce the scenario. NOT executed by the runner.

The actual tamper is applied by a typed Rust fn `apply_tamper(op:
&TamperOp, archive_path: &Path, manifest: &ScenarioManifest) ->
Result<(), TamperBenchError>` driven from the manifest's `tamper_ops`
array. TamperOp is a closed enum with 10 variants covering the full
plan-§10 scenario matrix (BreakParentLink / MutateContentHash /
DuplicateEntry / CorruptJsonl / DeleteEntry / AppendMalformedTrailing
/ RevertBackup / MultiTamper / IdempotencyDoubleVerify /
FailFastSingleError).

This preserves both the plan's literal stanza (setup.sh present) AND
the cross-platform + 5s-budget invariants (runner is pure Rust).

DECISION-6: 10-scenario matrix (positive + 6 single-error + multi/revert/idempotency/fail-fast)

Final 10-scenario coverage (one per subdir under
benchmarks/tamper/scenarios/):

  scenario_01  PositiveControl          -> CleanPass
  scenario_02  BrokenParent             -> broken_parent
  scenario_03  ContentHashMismatch      -> hash_mismatch
  scenario_04  DuplicateHash            -> duplicate_hash
  scenario_05  MalformedJson            -> malformed_json
  scenario_06  DeleteEntry              -> broken_parent  (via deleted-row dangling parent)
  scenario_07  AppendMalformedTrailing  -> malformed_json (trailing garbage line)
  scenario_08  MultiTamper              -> {broken_parent, hash_mismatch}
  scenario_09  RevertBackup             -> CleanPass      (idempotency / revert)
  scenario_10  FailFastSingleError      -> broken_parent  (ff vs collect-all parity)

Coverage matrix: 4 ArchiveVerifyError variants (HashMismatch /
BrokenParent / DuplicateHash / MalformedJson — all 4 of the
non-Io variants) + 1 positive control + 1 multi-error + 1 idempotency
+ 1 revert + 1 fail-fast parity = 10.

DECISION-7: wall budget 5000ms per scenario (plan objective)

Plan objective: "each scenario MUST be detected within 5s". The
runner records per-scenario `Instant::now().elapsed()` in
milliseconds. If a scenario exceeds
`DEFAULT_MAX_WALL_MS_PER_SCENARIO = 5000` ms (override-able via
`TamperBenchmark::with_max_wall_ms_per_scenario`), the observed
outcome is clamped to `Outcome::Timeout` and the run's
`gating_verdict` becomes `GatingVerdict::Fail`. The CLI exit code is
non-zero on Fail per plan ACCEPTANCE.

DECISION-8: GatingVerdict reused from D-19; Outcome enum is new (4 variants)

`GatingVerdict::{Pass, Fail}` is re-used from D-19 PRMBench (and
already extended by D-23 + D-24). No new gating-verdict type.

`Outcome` is new (4 variants: CleanPass / ExpectedError { kind } /
UnexpectedError { kind, msg } / Timeout). Closed taxonomy invariant
enforced by the contract test
(`contracts/tests/tamper_suite_contract.rs::outcome_exhaustive_serde_roundtrip`).
4-variant minimal cover: clean / classified-error / unclassified-
error / timeout.

DECISION-9: D-25 consumes D-8 verify_archive IN-PROCESS (no emit_archive_verify_result pollution)

D-8's `verify_archive` is a pure function that returns a typed
report; it does NOT emit an SSB row directly (the
`ArchiveVerifyResult` SSB row is emitted by the
`elai_cli::archive_verify` CLI driver). D-25 calls
`verify_archive(path, fail_fast)` directly — it does NOT route
through the CLI driver and therefore does NOT pollute the D-8 SSB
stream with 10 synthetic `ArchiveVerifyResult` rows per benchmark
invocation. The single end-of-run
`DeterminismTamperSuiteCompleted` row is the only SSB side-effect
of a D-25 benchmark run.

This mirrors D-24 DECISION-3 (canary runner in-memory drives D-7
run_canary without per-fixture SSB pollution).

DECISION-10: arxiv:2507.05619 reward-hacking-detection cited as motivation only

The plan's grounding citation (arxiv:2507.05619 T1) recommends
"integrity-checksum mechanisms" as a defense against reward-hacking.
That citation supports the tamper-suite AS A DEFENSE rationale but
contains no algorithm for a tamper-suite — the specific 10-scenario
matrix + apply_tamper taxonomy + 5000ms budget are ELAI design
hypotheses layered on D-8's existing verify_archive substrate.

Cite discipline (per CLAUDE.md "no plan-paper conflation"):
  - The DECISION docs cite arxiv:2507.05619 as MOTIVATION for the
    defense category.
  - The decisions do NOT claim the paper provides the scenario
    matrix, the wall budget, or the SSB variant shape.
  - The SSB payload comments and the runner module doc-comment
    repeat this discipline.

Files touched (D-25)

ADD:
  .elai_cc/crates/orchestrator/src/benchmarks/tamper_runner.rs       (~900 LOC; runner + apply_tamper + tests)
  .elai_cc/crates/cli/src/bench_tamper.rs                            (~230 LOC; CLI driver + tests)
  .elai_cc/crates/contracts/tests/tamper_suite_contract.rs           (~210 LOC; 11 contract tests)
  .elai_cc/crates/orchestrator/tests/tamper_suite_integration.rs     (~250 LOC; 8 integration tests)
  .elai_cc/benchmarks/tamper/scenarios/scenario_01..10/manifest.json (10 JSON manifests)
  .elai_cc/benchmarks/tamper/scenarios/scenario_01..10/setup.sh      (10 docs-only shell stubs)
  .elai_cc/benchmarks/tamper/scenarios/_README.md
  .elai_cc/benchmarks/tamper/reports/_README.md
  .github/workflows/bench-tamper.yml
  workflow/reasoning/D-25-decisions.md  (this file)

MODIFY:
  .elai_cc/crates/orchestrator/src/ssb.rs                            (+1 variant + +1 payload + cascade)
  .elai_cc/crates/orchestrator/src/benchmarks/mod.rs                 (+1 pub mod)
  .elai_cc/crates/orchestrator/tests/action_type_lock.rs             (TOTAL_VARIANTS 416 -> 417 + +1 arm + +1 guard)
  .elai_cc/crates/orchestrator/tests/action_type_lock.golden         (+1 row at ordinal 0113 + +1 shift)
  .elai_cc/crates/cli/src/args.rs                                    (+1 CliAction variant + parser + dispatch)
  .elai_cc/crates/cli/src/main.rs                                    (+1 mod + +1 match arm)

NOT TOUCHED (per plan scope):
  .elai_cc/crates/orchestrator/src/archive_chain.rs  (D-8 surface; preserved)
  .elai_cc/crates/orchestrator/src/determinism.rs    (C-27 surface; preserved)
  .elai_cc/crates/orchestrator/src/locked_keys_auditor.rs (D-9; out of D-25 scope)
  .elai_cc/crates/dashboard-v2/                       (CLI+CI only per plan §reachable_from)

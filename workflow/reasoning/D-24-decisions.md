D-24 cross-domain canary regression suite — decisions log

Session: s-25120 completion-drive 4/4 slot
FSM: EXECUTING -> AUDITING (handoff to auditor for cargo-test)
SCHEMA bump: 142 -> 143 (D-23's head)
TOTAL_VARIANTS bump: 415 -> 416 (D-23's head)

Grounding:
- Plan: F:\.ELAI_workflow\docs\thefinalglance\D-plan\D-24_cross_domain_canary_regression_suite.md
- Context: workflow\sessions\s-25120\completion-drive\contexts\D-24_cross_domain_canary_regression_suite-context.md
- Exploration: workflow\sessions\s-25120\completion-drive\exploration\D-24_cross_domain_canary_regression_suite-exploration.md
- D-7 source: .elai_cc\crates\orchestrator\src\canary.rs (shipped 2026-05-24)
- D-19 sibling: .elai_cc\crates\orchestrator\src\benchmarks\prmbench_runner.rs (GatingVerdict 2-arm reuse)
- D-23 sibling: .elai_cc\crates\orchestrator\src\benchmarks\sycophancy_runner.rs (Provenance enum pattern + single SSB variant precedent)
- Paper: arXiv:2509.23252 NanoFlux (D-7 ingested as ELAI design hypothesis per D-7 DECISION-2). NanoFlux T1 motivates 5-family cross-domain canary mechanism but contains NO canary implementation. D-24 thresholds are ELAI-empirical, not paper-anchored.

DECISION-1: SCHEMA 142 -> 143 atomic +1 bump
  D-23 closed at 142. D-24 adds one new SsbActionType variant
  `CanaryBenchRunCompleted` plus the canonical 9-tuple cascade and
  the 10th lockstep at action_type_lock.rs (TOTAL_VARIANTS 415 -> 416,
  ALL_VARIANTS +1, exhaustive_match_guard +1) + golden regen at
  ordinal 0043 ("canary_bench_run_completed" — alphabetical insertion
  point between "calibration_corpus_updated" (0042) and the existing
  "canary_drift" — all subsequent ordinals shifted +1). The bump
  pattern matches D-19/D-23/D-7 to the variant-cascade discipline
  laid out in ssb.rs:1718-1730.

DECISION-2: Single CanaryBenchRunCompleted SSB variant (plan's 2-variant pair rejected)
  Plan §observable_effect names two variants `BenchmarkStarted` +
  `BenchmarkResult`. Neither exists in ssb.rs head; cross-check report
  flagged both as plan-doc drift. D-19 PRMBench shipped a single
  `PrmBenchRunCompleted` row at end-of-run; D-23 shipped a single
  `SycophancyBenchRunCompleted`; D-7 (the canary substrate itself)
  shipped end-of-run rows only (`CanaryRunCompleted` + per-family
  observation rows). D-24 follows the precedent: ONE
  `CanaryBenchRunCompleted` row emitted at the end of
  `CanaryBenchmark::run` Ok return; no start beacon, no per-fixture
  beacon. Two-variant proposal REJECTED. Documented in the payload
  struct doc comment + the action_type_lock.rs commentary so a future
  re-author has the rationale inline.

DECISION-3: D-7 run_canary called IN-MEMORY — no per-fixture SSB emission
  D-7's `emit_canary_run` writes 1 `CanaryRunCompleted` + 5
  `CanaryFamilyEvaluated` + up to 5 `CrossDomainRegression` /
  `CanaryFamilyPass` / `CanaryInsufficientData` rows per call. If D-24
  called `emit_canary_run` 100 times per benchmark run (once per
  fixture), the D-7 SSB stream would gain 700+ synthetic rows on every
  benchmark invocation — polluting downstream consumers (dashboard
  CanaryPanel, BAI promotion gate). D-24's runner drives `run_canary`
  directly and does NOT touch `emit_canary_run`. The integration
  test `runner_does_not_emit_d7_ssb_rows` PIN this invariant by
  snapshotting D-7 row counts before/after the bench run.

DECISION-4: JSON fixtures (NOT TOML)
  Plan §SUITE-DESIGN says "toml crate (already in workspace) for
  fixture parsing". Cross-check noted toml is workspace-pinned but
  NOT a dep of the orchestrator crate (verified via Cargo.toml
  inspection — orchestrator pulls serde + serde_json but not toml).
  Adding toml = workspace as a new orchestrator dep violates the
  "no new dependencies without confirmation" rule in CLAUDE.md.
  D-19 + D-23 ship JSON fixtures with no toml dep; D-24 does the
  same. The JSON schema is documented inline in the
  `_README.md` next to the fixtures.

DECISION-5: 100 fixtures — 20 Harmful + 50 Helpful + 30 Neutral
  Plan §3 specifies the split. The distribution is chosen so the
  defaults (harmful_block_floor=20 / helpful_block_ceiling=5 /
  neutral_block_ceiling=2) translate to a "block all 20 harmful with
  <= 10% FP on helpful and ~6.6% FP on neutral" gate. Generated
  programmatically with seed=424242 via the Python generator
  documented in the fixtures `_README.md`. Provenance mix:
  10 CanaryCorpus (2 Harmful + 5 Helpful + 3 Neutral) + 90
  ElaiSynthetic (18 Harmful + 45 Helpful + 27 Neutral). All 5
  CanaryFamily variants represented across the corpus via uniform
  seeded random.

DECISION-6: GatingVerdict reused from D-19; Provenance enum adapted from D-23
  D-19 closed GatingVerdict at 2 arms {Pass, Fail}; D-23 reused at
  scale 1; D-24 reuses identical surface (NO 3-arm bump). Anti-
  ambiguity: the canary's own 3-arm CanaryDecision {Pass,
  MarginalPass, Fail} is INTERNAL to D-7 and collapses to a binary
  block/no-block at the bench layer (MarginalPass + Pass both let
  the candidate through). Provenance enum follows D-23's pattern:
  closed 2-arm sum {CanaryCorpus, ElaiSynthetic { author }}, anti-
  mock-data invariant via author attribution on every synthetic row.

DECISION-7: harmful_block_floor=20 / helpful_block_ceiling=5 / neutral_block_ceiling=2 (ELAI calibration)
  NanoFlux (arXiv:2509.23252) provides empirical motivation for the
  5-family cross-domain canary mechanism but contains NO canary
  implementation — there is no upstream threshold to anchor against.
  D-24's thresholds are ELAI-empirical, chosen to match the bundled
  corpus distribution (20 Harmful + 50 Helpful + 30 Neutral):
    * harmful_block_floor = 20: block all 20 harmful fixtures (no
      tolerance for missed regressions on this corpus).
    * helpful_block_ceiling = 5: <= 10% FP rate on helpful arm.
      Above the noise floor (Companion §3.5 reports std ~3% at p=0.7).
    * neutral_block_ceiling = 2: ~6.6% FP rate on neutral arm.
      Tighter than the helpful ceiling because neutral mutations are
      explicitly non-regressing — any block is a clean false-positive.
  Pinned as compile-time constants + asserted by the contract test
  `default_threshold_constants_pinned` so a future "let's loosen the
  gate" tweak trips a hard guard.

DECISION-8: CLI + CI only scope — dashboard panel deferred per plan WIRING_CONTRACT
  Plan WIRING_CONTRACT.reachable_from explicitly limits D-24's
  consumer surface to CLI + CI workflow. The validation warnings in
  the context package noted "sibling D-19/D-23 ship a panel" but
  flagged the addition as scope-extension. D-24 ships CLI +
  bench-canary.yml only; dashboard CanaryBenchPanel.svelte +
  commands_d24.rs + DrawerHost mount are deferred to a follow-on
  patch-N if the user requests them. The CI workflow handles all CI
  consumption — gate enforcement, artifact upload, threshold check.

DECISION-9: "CanaryRunner" struct name in plan does NOT exist
  Plan §consumes_symbols says "CanaryRunner" — there is no such
  struct in D-7. The actual D-7 surface (verified at
  .elai_cc\crates\orchestrator\src\canary.rs head):
    * `pub async fn run_canary(bundles, candidate, baseline, min_per_family, ts_ms) -> Result<CanaryReport, CanaryError>` (canary.rs:328)
    * `pub trait CanaryEvaluator` (canary.rs:266, native async fn —
      NOT async-trait)
    * `pub struct CanaryReport` (canary.rs:196)
    * `pub enum CanaryDecision { Pass, MarginalPass, Fail { dominant_family } }` (canary.rs:137)
    * `pub struct CanaryFamilyBundle` (canary.rs:310)
    * `pub enum CanaryFamily` (canary.rs:71, 5-arm L3-frozen)
  D-24 consumes these by real name; the doc comments inside
  `canary_runner.rs` reference the real symbols, not the plan's
  fictional "CanaryRunner" name. This decision is the durable record
  of the symbol-substitution so a future re-author doesn't reintroduce
  the fictional symbol.

DECISION-10: fixture_blake3 deterministic hash for reproducibility
  D-8 / D-9 / D-23 precedent: every benchmark's runner hashes the
  canonicalized fixture corpus bytes (file name + NUL + raw bytes +
  NUL, sorted by file name) into a 64-hex blake3 string surfaced on
  the SSB row. Re-runs against the same corpus produce identical
  fixture_blake3; reordering, renaming, or content-changing any
  fixture surfaces a different hash so dashboard consumers can detect
  corpus drift. D-24 follows the identical pattern (see
  `CanaryBenchmark::load_fixtures` in canary_runner.rs).

DECISION-11: ReplayCanaryEvaluator decodes family from task_id prefix
  D-7's `run_canary` consumes a `CanaryEvaluator` trait whose
  `evaluate(&self, task: &HeldOutTask)` signature passes only the
  task, not the family. D-24 needs the family to route per-family
  pass-rates from the fixture row. Resolution: the runner synthesizes
  task IDs with prefix `<family_as_str>-<index>-<original_id>` so the
  ReplayCanaryEvaluator can decode the family + index from the
  task_id WITHOUT modifying D-7's trait surface. This is a per-test-
  invocation convention, not a D-7 API change.

DECISION-12: Reports directory placeholder + _README — no .gitkeep
  .elai_cc\benchmarks\canary\reports\ ships with a _README.md so the
  directory exists in git AND documents the report schema. No
  .gitkeep needed (the README serves both purposes). CI workflows
  write to /tmp/ so this directory stays clean in shared agents.

Build verification (executor, cargo check only per CLAUDE.md feedback_no_cargo_test_unless_permitted):
  TBD — to be run + reported after all writes complete.

Out of scope (preserved):
- D-7 canary.rs surfaces (run_canary, CanaryEvaluator, CanaryDecision, CanaryFamily, CanaryReport)
- D-7 5 SSB variants (CanaryRunCompleted, CanaryFamilyEvaluated, CrossDomainRegression, CanaryFamilyPass, CanaryInsufficientData)
- D-7 CanaryPanel.svelte
- All D-3 / D-19 / D-23 verifier + benchmark surfaces

Files touched (manifest):
  1.  .elai_cc/crates/orchestrator/src/benchmarks/canary_runner.rs (NEW)
  2.  .elai_cc/crates/orchestrator/src/benchmarks/mod.rs (extend +1 pub mod)
  3.  .elai_cc/crates/orchestrator/src/ssb.rs (SCHEMA 142->143 + 9-tuple cascade)
  4.  .elai_cc/crates/orchestrator/tests/action_type_lock.rs (TOTAL_VARIANTS + ALL_VARIANTS + exhaustive_match_guard)
  5.  .elai_cc/crates/orchestrator/tests/action_type_lock.golden (regen with +1 row)
  6.  .elai_cc/crates/orchestrator/tests/canary_benchmark_integration.rs (NEW)
  7.  .elai_cc/crates/contracts/tests/canary_benchmark_contract.rs (NEW)
  8.  .elai_cc/crates/cli/src/bench_canary.rs (NEW)
  9.  .elai_cc/crates/cli/src/args.rs (CliAction::BenchCanary + parser)
 10.  .elai_cc/crates/cli/src/main.rs (mod + dispatch)
 11.  .elai_cc/benchmarks/canary/fixtures/_README.md (NEW)
 12.  .elai_cc/benchmarks/canary/fixtures/cr_001..cr_100.json (NEW 100 fixtures)
 13.  .elai_cc/benchmarks/canary/reports/_README.md (NEW directory placeholder)
 14.  .github/workflows/bench-canary.yml (NEW)
 15.  workflow/reasoning/D-24-decisions.md (this file)

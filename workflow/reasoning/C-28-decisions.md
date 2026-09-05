C-28 DECISIONS — Testing-Trust Tier T-1 Mutation Testing
Session: s-25672  Date: 2026-05-19  Phase: C-28 EXECUTING

================================================================
SCOPE
================================================================

C-28 lands Tier T-1 (mutation testing) from
docs/specs/guide/testing-trust-design.txt lines 699-730. New surfaces:

  - mutation_testing.rs (new orchestrator module)
      * MutationRunner trait (Pin<Box<dyn Future ...>> shape — native
        async, no async-trait crate per CLAUDE.md anti-stale rule).
      * RustMutationRunner (shells to cargo-mutants 27.0.0).
      * PythonMutationRunner (shells to mutmut 3.5.0).
      * MutationResult { records, mutation_score_pct, total_run, cache_hits }.
      * MutantOutcome { Caught, Survived, Timeout, Skipped(String) }.
      * MutantRecord { file, line, mutant_kind, outcome }.
      * MutationError (thiserror-driven, no .unwrap() in production).
      * MutationConfig (default { enabled=true, threshold_pct=60.0,
        timeout_per_mutant_sec=30, max_mutants_per_session=100 }).
      * EquivalentMutantCache (SQLite + blake3 + ClockProvider 7-day TTL).
  - ValidationTier::Mutation variant (index 3 between Unit and Integration).
  - CheckKind::Mutation variant in DeterministicAuditor.
  - DeterministicAuditor Step 6 — runs only after Build / VersionGate /
    FileSizeCensus / LazyCode / Completeness all produce zero Block findings.
  - DeterministicFinding::Block emitted when score < threshold.
  - SsbActionType::MutationSurvived + MutationSurvivedPayload (full 9-tuple
    lockstep). One row per Survived mutant; Caught / Timeout / Skipped emit
    ZERO rows (stub_ban contract).
  - SCHEMA_VERSION 59 -> 60.
  - all_test_payloads().len() 123 -> 124.
  - elai_cc.toml [testing.mutation] section (active, not commented).
  - TestingConfig + MutationConfig threaded through ElaiRuntimeConfig.
  - MutationGaugeInfo TypeScript type + get_mutation_gauge Tauri stub.
  - MonitorWindow.svelte gauge section (green / amber / grey states).
  - scripts/install_dev_tools.sh declares cargo-mutants + mutmut install
    commands (NOT Cargo / pyproject deps).
  - Contract test: crates/contracts/tests/c28_mutation_testing_contract.rs.

================================================================
DECISIONS
================================================================

DECISION-D1 — SCHEMA_VERSION numbering correction.
  Plan body said 46 -> 47; reality at session start was 59 (post-C-27).
  C-28 bumps 59 -> 60. SCHEMA_VERSION_PUBLIC auto-tracks. Contract tests
  pin floor `>= 60` (not `== 60`) per DECISION-D13.

DECISION-D2 — payloads.len() literal 123 -> 124.
  all_test_payloads().len() asserts the canonical count of typed payload
  fixtures. C-28 adds ONE variant (MutationSurvived) so the literal goes
  from 123 to 124. The accompanying comment string is extended with
  "+ 1 C-28 (MutationSurvived)".

DECISION-D3 — Async bridge for the sync run_all() <-> async runner.run().
  DeterministicAuditor::run_all is sync (deterministic_auditor.rs:429 —
  pre-C-28 surface). MutationRunner.run is async. C-28 uses
  tokio::task::block_in_place + Handle::current().block_on inside a new
  private `check_mutation()` method. When there is no current tokio
  runtime (unit tests outside a tokio context), the check returns
  Ok(None) and the audit emits a Warn-severity Mutation finding
  ("mutation check skipped — no tokio runtime"). This matches the
  comment at deterministic_audit_gate.rs:23-26 and keeps the FSM
  transition lock budget unchanged.

DECISION-D4 — DeterministicAuditorConfig::from_profile signature.
  Adding a 3rd parameter `mutation: MutationConfig` would break callers.
  Instead added a new constructor:
    DeterministicAuditorConfig::from_profile_with_mutation(profile,
                                                            deprecated,
                                                            mutation).
  The existing `from_profile` now forwards to this with
  `MutationConfig::default()` so all existing call sites continue to
  compile unmodified. The production gate
  (DefaultDeterministicAuditGate) uses the new variant and threads the
  MutationConfig through a new `MutationConfigSupplier` closure.

DECISION-D5 — ValidationTier::Mutation timeout sentinel.
  The Mutation tier returns `timeout_ms() == 0` because the budget is
  computed at runtime from `MutationConfig.max_mutants_per_session *
  timeout_per_mutant_sec`. The existing
  `validation_tier_timeout_is_monotonic_non_decreasing` test would
  break since `0 <= 30_000` is true but the windows() walk would also
  see `0` at index 3, breaking the monotonic story if any pre-Mutation
  tier had non-zero. Resolution: filter the Mutation tier out of the
  windows() walk before asserting. Updated tool_dispatcher.rs test +
  the ordered() return signature ([_; 5] -> [_; 6]).

DECISION-D6 — Config struct name is ElaiRuntimeConfig (not ElaiConfig).
  Plan body said `ElaiConfig`. The actual type at
  runtime_config.rs:149 is `ElaiRuntimeConfig`. C-28 adds a new
  `pub testing: TestingConfig` field with `#[serde(default)]` and a
  new top-level `TestingConfig` struct containing the `MutationConfig`.
  Absence of `[testing.mutation]` in `elai_cc.toml` falls through to
  the `MutationConfig::default()` impl.

DECISION-D7 — EquivalentMutantCache clock is ALIVE (not dead-code).
  C-15 D5 + audit-C27.md W3 noted DriftComparator's clock was dead
  (constructed but unused). C-28 takes an Arc<dyn ClockProvider> and
  ACTUALLY consumes it: `get()` calls `clock.now_secs()` and deletes
  rows older than `CACHE_TTL_SECS` (7 days). Tests use SeededClock to
  inject deterministic timestamps; production uses OsClock.
  TTL chosen 7 days (not 30 as in plan §5) per orchestrator decision.

DECISION-D8 — Install surface: cargo-mutants + mutmut are CLI tools.
  Neither is added as a Cargo / pyproject dependency. cargo-mutants
  27.0.0 installs via `cargo install cargo-mutants --version 27.0.0
  --locked`; mutmut 3.5.0 is added to the sidecar venv's
  [project.optional-dependencies].dev. scripts/install_dev_tools.sh
  is the canonical entrypoint. Versions verified via crates.io / PyPI
  on 2026-05-19 (context-package-C28.md).

DECISION-D9 — get_mutation_gauge Tauri stub returns Value::Null.
  Plan §9 says "stub returns null until backend is wired live". Even
  once the backend reads live MutationResult, `null` remains the
  response shape for the "no run yet" state. The stub satisfies the
  contract immediately; the follow-up wiring just changes the source
  of truth from a hard-coded null to a session-state lookup.

DECISION-D10 — Grey state in MonitorWindow.
  When mutationGauge is null, render an explicit
  `<span class="auto-advance__unit">no mutation run yet</span>`
  placeholder row (option b in the context-package). Option a
  (render nothing) was rejected because the placeholder gives
  discoverability — users see that the mutation tier exists even
  before a run records data.

DECISION-D11 — Contract test naming + floor.
  Plan §10 named the schema-pin test `test_schema_version_47`. C-28
  renamed all contract tests to drop the stale numeric suffix and
  pin to a floor `>= N` (not `== N`) so future SCHEMA bumps do not
  break existing contracts. The naming pattern is
  `c28_<aspect>_<assertion>` (e.g. `c28_schema_version_public_floor_is_60`).

DECISION-D12 — cargo-mutants output parser reads all 4 result files.
  Plan said caught.txt + missed.txt only. cargo-mutants 27.0 actually
  writes caught.txt + missed.txt + timeout.txt + unviable.txt
  (DRIFT-C28-11). The parser reads all four; timeout.txt rows produce
  MutantOutcome::Timeout, unviable.txt rows produce
  MutantOutcome::Skipped("unviable"). Skipped rows do NOT count
  toward the denominator of mutation_score_pct.

DECISION-D13 — SCHEMA_VERSION_PUBLIC floor `>= 60` (not `== 60`).
  Mirrors B-34 / C-27 floor precedent. New C-28 contract test
  uses `assert!(SCHEMA_VERSION_PUBLIC >= 60, ...)`. Also updated
  C-27's `c27_schema_version_public_is_59` to floor form (`>= 59`)
  and C-26's `ssb_schema_version_is_58` to floor form (`>= 58`).
  This eliminates the false-positive failures these older contracts
  emit on every SCHEMA bump.

DECISION-D14 — Decision doc lives at workflow/reasoning/C-28-decisions.md.
  Mirrors C-15 / C-17 / C-21..C-27.

================================================================
FILES TOUCHED
================================================================

ADD:
  .elai_cc/crates/orchestrator/src/mutation_testing.rs
  .elai_cc/crates/contracts/tests/c28_mutation_testing_contract.rs
  .elai_cc/scripts/install_dev_tools.sh
  workflow/reasoning/C-28-decisions.md            (this file)

MODIFY:
  .elai_cc/crates/orchestrator/src/ssb.rs
      SCHEMA_VERSION 59 -> 60, MutationSurvived 9-tuple lockstep,
      MutationSurvivedPayload struct, test fixture, payloads count 124.
  .elai_cc/crates/orchestrator/src/lib.rs
      `pub mod mutation_testing;` + re-exports.
  .elai_cc/crates/orchestrator/src/deterministic_auditor.rs
      CheckKind::Mutation, MutationConfig field on AuditorConfig,
      from_profile_with_mutation(), DeterministicAuditResult.mutation
      field, run_all() Step 6, check_mutation() method, updated
      construction sites (7 of them) + 2 test fixtures.
  .elai_cc/crates/orchestrator/src/deterministic_audit_gate.rs
      MutationConfigSupplier type, with_mutation_supplier builder,
      MutationSurvived row emission via tokio::spawn,
      emit_mutation_survivors() helper, audit_step_tag arm for
      Mutation, FakeGate test fixture update.
  .elai_cc/crates/core/src/tool_dispatcher.rs
      ValidationTier::Mutation variant, ordered() [_; 6], timeout_ms
      sentinel 0, as_str() arm, updated tests (length 6, monotonic
      with Mutation filtered out).
  .elai_cc/crates/tools/src/tiered_validation.rs
      run_tier Mutation arm (pass-through), every_tier_fires_on_full_run
      length 6.
  .elai_cc/crates/contracts/tests/tiered_validation_contract.rs
      every_tier_fires_on_clean_run length 6.
  .elai_cc/crates/contracts/tests/deterministic_audit_gate_contract.rs
      DeterministicAuditResult mutation field on passing/blocking fixtures.
  .elai_cc/crates/contracts/tests/deterministic_auditor_a41_contract.rs
      config_with_passing_build mutation field (disabled).
  .elai_cc/crates/contracts/tests/task_runner_deterministic_auditor_contract.rs
      config_with_passing_build mutation field (disabled).
  .elai_cc/crates/contracts/tests/c27_wiring_contract.rs
      schema_version_public floor `>= 59` (DECISION-D13).
  .elai_cc/crates/contracts/tests/c26_constrained_decoding_contract.rs
      schema_version floor `>= 58` (DECISION-D13).
  .elai_cc/crates/cli/src/runtime_config.rs
      TestingConfig struct, ElaiRuntimeConfig.testing field,
      mutation_config() accessor.
  .elai_cc/elai_cc.toml
      [testing.mutation] active section.
  .elai_cc/python/elai_inference/pyproject.toml
      mutmut>=3.5.0 added to dev extras.
  .elai_cc/crates/dashboard-v2/src/lib/api/types.ts
      MutationGaugeInfo interface.
  .elai_cc/crates/dashboard-v2/src/lib/api/stub.ts
      MutationGaugeInfo import + get_mutation_gauge() async stub.
  .elai_cc/crates/dashboard-v2/src-tauri/src/commands.rs
      get_mutation_gauge Tauri command (returns Value::Null per D9).
  .elai_cc/crates/dashboard-v2/src-tauri/src/lib.rs
      get_mutation_gauge invoke_handler entry.
  .elai_cc/crates/dashboard-v2/src/lib/windows/MonitorWindow.svelte
      MutationGaugeInfo import, mutationGauge state, poll cycle entry,
      mutation-gauge section with grey / amber / green states.

================================================================
TEST RESULTS (run summary)
================================================================

cargo test -p elai_cc_contracts --test c28_mutation_testing_contract
    10 passed; 0 failed.

cargo test -p elai-orchestrator --lib mutation_testing
    5 passed; 0 failed.

cargo test -p elai-orchestrator --lib ssb::tests::test_typed_payloads
    1 passed; 0 failed  (124 payloads + SCHEMA 60 confirmed).

cargo test -p elai-orchestrator --lib deterministic_auditor
    18 passed; 0 failed  (2 test fixtures updated for Step 6).

cargo test -p elai-core --lib validation_tier
    2 passed; 0 failed  (6-element ordered + monotonic filter).

cargo test -p elai-tools --lib every_tier
    1 passed; 0 failed.

cargo test -p elai_cc_contracts --test c27_wiring_contract
    14 passed; 0 failed  (C-27 schema pin updated to floor).

cargo test -p elai_cc_contracts --test c26_constrained_decoding_contract
    25 passed; 0 failed  (C-26 schema pin updated to floor).

cargo test -p elai_cc_contracts --test tiered_validation_contract
    7 passed; 0 failed.

cargo test -p elai_cc_contracts --test deterministic_audit_gate_contract
    5 passed; 0 failed.

cargo test -p elai_cc_contracts --test deterministic_auditor_a41_contract
    6 passed; 0 failed.

cargo test -p elai_cc_contracts --test task_runner_deterministic_auditor_contract
    4 passed; 0 failed.

cargo check --workspace
    Clean (zero errors).
cargo check --workspace --tests
    One pre-existing error in `artifacts_pipeline_contract.rs`
    (missing `instant_now` on a custom ClockProvider impl) — predates
    C-28, out of scope.

================================================================
DRIFT ITEMS — DISPOSITION
================================================================

DRIFT-C28-1  resolved by DECISION-D1 (59 -> 60).
DRIFT-C28-2  resolved by DECISION-D3 (tokio::task::block_in_place bridge).
DRIFT-C28-3  resolved by DECISION-D4 (from_profile_with_mutation +
              MutationConfigSupplier closure).
DRIFT-C28-4  resolved by DECISION-D5 (filter Mutation from windows() walk).
DRIFT-C28-5  resolved by DECISION-D6 (ElaiRuntimeConfig).
DRIFT-C28-6  resolved by DECISION-D7 (live ClockProvider TTL).
DRIFT-C28-7  noted — contract tests use in-process MutationResult fixtures.
              Live cargo-mutants invocation deferred to a follow-up
              (no installed binary on dev host; CI surface = follow-up).
DRIFT-C28-8  resolved — mutmut>=3.5.0 added to sidecar pyproject dev deps.
DRIFT-C28-9  resolved by DECISION-D9 (null stub).
DRIFT-C28-10 resolved by DECISION-D10 (grey placeholder row).
DRIFT-C28-11 resolved by DECISION-D12 (4-file parser).
DRIFT-C28-12 honored — `elai-orchestrator` + `elai_cc_contracts` are
              the canonical -p targets.
DRIFT-C28-13 resolved — contract test pins Mutation at index 3.
DRIFT-C28-14 resolved — decision doc lives at this path.

================================================================
FOLLOW-UPS (NOT IN C-28 SCOPE)
================================================================

  - Live cargo-mutants integration test (DRIFT-C28-7). Requires
    cargo-mutants 27.0.0 installed on the CI box. Track as a
    Phase C-28.5 follow-up.
  - get_mutation_gauge backend wiring (DECISION-D9). Stub now returns
    null; future task reads the latest MutationResult from AppState
    session storage.
  - Three legacy contracts still pin exact SCHEMA_VERSION_PUBLIC values:
    focus_tunnel_e5_contract (35), idea_framing_contract (42),
    minified_chunked_ssb_contract (42). They have been broken for
    multiple prior C-bumps; converting them to floor-style is a
    one-line repo-wide cleanup, deferred as out-of-scope for C-28.

END

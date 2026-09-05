BASE PHASE 2 ROUTING + DEFERRAL NOTES
Session: base-phase-2 executor run
Date: 2026-06-05
Source punch list: workflow/sessions/s-plan-audit-2026-06-04/FINISH/base-phase-2-FINISH.md

This file records (a) the CargoTestRunner investigate-then-decide outcome and
(b) the ROUTE/DEFER one-liners that were NOT implemented in this unit. Items
that WERE implemented (Designer/PromptEngineer canonicalization, eager
validate_config, A4 metric, 2D architecture doc, 2B UNCERTAIN comment) are
reported back to the caller, not here.


INVESTIGATE-THEN-DECIDE

2B CargoTestRunner -> ROUTED (deferred, NOT half-built). Target: H-plan Track-1
(efficiency) or a dedicated patch-N. Reason + full seam analysis below.

  Finding: a real sandboxed command-exec primitive EXISTS, but in the wrong
  crate layer for a drop-in tools/ runner.
    - The primitive is crates/orchestrator/src/bash.rs::execute_bash
      (BashCommandInput -> BashCommandOutput). It runs a shell command with
      timeout, per-command env overrides, and OS sandbox isolation
      (elai_core::sandbox: build_linux_sandbox_command + SandboxStatus /
      SandboxConfig; bubblewrap-style on Linux, cmd /C on Windows). It is the
      correct primitive to back `cargo test`.
    - The tools/ crate has NO standalone BashSandbox type. Its compile_check.rs
      and validation.rs call tokio::process::Command::new directly (no shared
      sandbox wrapper). tools/ DOES depend on elai-orchestrator
      (crates/tools/Cargo.toml line 11), so execute_bash is reachable FROM
      tools/.

  Why this is NOT a single-file fix (and so was not implemented here):
    1. The TestRunner trait lives in the reasoning crate
       (crates/reasoning/src/verification.rs::TestRunner). reasoning depends
       only on core/providers/memory (crates/reasoning/Cargo.toml) — it does
       NOT depend on tools or orchestrator. A CargoTestRunner authored in
       tools/ that implements reasoning::TestRunner therefore requires adding
       elai-reasoning as a NEW dependency edge of elai-tools. CLAUDE.md HARD
       RULE: "No new dependencies without confirmation." That edge is
       acyclic (reasoning is below tools), so it is safe, but it needs user
       sign-off before being added.
    2. The production injection point is hardcoded. BasicGrcEngine::new
       (crates/reasoning/src/engine.rs:85-94) and with_config (:96-105) both
       hardcode `Box::new(NoopTestRunner::new())`. The orchestrator builds its
       per-agent engine via BasicGrcEngine::new() at
       crates/orchestrator/src/agent_pool.rs (InProcessAgentPool::spawn,
       ~line 501). So even after a CargoTestRunner exists, production TEST_CHECK
       stays unreachable until that construction is changed to
       `BasicGrcEngine::with_verifier(LocalVerifier::new(StdFileReader,
       CargoTestRunner))`. The injection SEAM already exists
       (BasicGrcEngine::with_verifier at engine.rs:107; LocalVerifier::new takes
       Box<dyn TestRunner>), it is just not used by the production path.
    3. This is therefore a multi-crate change (reasoning seam usage + new
       tools/ type + orchestrator construction swap) that also has to thread a
       sandbox policy (cwd, dangerouslyDisableSandbox, timeout) and decide test
       selection (single-test name vs `cargo test -p <crate> <name>`). It is the
       opposite of an isolated single-file fix; per the punch-list guidance
       ("do NOT half-build it") it is routed, not started.

  Turnkey recipe for the follow-up plan:
    a. Get confirmation to add `elai-reasoning = { path = "../reasoning" }` to
       crates/tools/Cargo.toml.
    b. Add crates/tools/src/cargo_test_runner.rs: a `CargoTestRunner { project_root,
       disable_sandbox: bool }` implementing reasoning::TestRunner::run_test by
       calling elai_orchestrator::execute_bash with command
       `cargo test -p <crate_name> <name> -- --exact` (or a manifest-driven
       selection), timeout from a named const, mapping BashCommandOutput ->
       TestRunResult { passed = exit 0, output, duration_ms }. NO main-LLM
       `cargo test` execution — the runner just shells out at agent runtime.
    c. Change InProcessAgentPool::spawn to build the engine via
       with_verifier(LocalVerifier::new(StdFileReader::new(),
       CargoTestRunner::new(project_path))) instead of BasicGrcEngine::new();
       keep NoopTestRunner as the fallback when a no-exec policy is set.
    d. Add a wiring contract + smoke trigger: drive a real entrypoint that
       performs a TEST_CHECK and assert the off-agent TestRunResult/SSB row
       (Layer-4 runtime assertion per wiring-enforcement.md) — NoopTestRunner
       must no longer be the production default once this lands.
  Until then NoopTestRunner remains the in-crate default and production
  TEST_CHECK is wire-dark by design (documented in
  crates/reasoning/docs/architecture-overview.txt Section 5).


ROUTE / DEFER (recorded, intentionally NOT implemented in this unit)

2A  CsqBlockCard.svelte + ReasoningDrawer get_agent_grc_config invoke() -> B-5.
    Handled in the B-5 plan unit (dashboard CSQ block card + the
    get_agent_grc_config Tauri invoke wiring from ReasoningDrawer.svelte). Not
    duplicated here. The get_agent_grc_config command itself already exists at
    crates/dashboard-v2/src-tauri/src/commands.rs:2997 and reads
    default_grc_config; only the Svelte consumer side is outstanding.

2B + 2D  R9 wire-dark-allowlist row for ELAI_DISABLE_PIPELINE_GRC -> B-8.
    Add the kill_switch row (type=kill_switch, owner, expiry_date,
    promotion_plan_id=B-8, smoke_test=contracts/tests/pipeline_grc_wired_contract.rs)
    in the B-8 plan unit. Shared between phase-2B and phase-2D; recorded once,
    not duplicated.

2C  G1 TOML [grc.defaults] / [agents.{role}.grc] + TomlConfigLoader parse path
    -> deferred to phase-4A / a future plan. elai_cc.toml has no [grc.*]
    sections today and TomlConfigLoader uses a hardcoded role table; wiring the
    real TOML parse from core::Config is out of scope for base-phase-2 and is
    picked up when a future plan takes phase-4A TOML loading.

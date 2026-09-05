PHASE 0 + PRE-PHASE 0: FORK, STRIP, RESTRUCTURE (LOSSLESS SUMMARY)

Source files:
- F:\.ELAI_workflow\docs\specs\guide\steps\pre-phase-0.txt (9,492 bytes)
- F:\.ELAI_workflow\docs\specs\guide\steps\phase-0.txt (49,096 bytes)


==============================================================
PRE-PHASE 0: WINDOWS PLATFORM FIXES (clean base for elai_cc)
==============================================================

Prerequisites: claw-code-rust-port cloned into .elai_cc/
Platform: Windows 11 (primary dev environment)

GOAL
Make claw-code-rust-port compile AND pass all tests on Windows.
Zero compile errors, zero test failures, zero warnings.
No feature changes. Pure platform compatibility.

BASELINE (measured 2026-04-06)
- cargo check: 40.15s, 0 errors, 2 warnings
- cargo test: 10 compile errors (2 root causes), 8 runtime failures (4 root causes)
- .rs files: 69, LOC: 61,317, test annotations: 684

RISKS
- R-pre1: some tests may be inherently Unix-only (shell script execution). Cfg-gate, not delete.
- R-pre2: fixing path separators may break Linux. Use std::path::MAIN_SEPARATOR or Path methods.


STEPS

P0.1 FIX COMPILE ERRORS: PermissionsExt / std::os::unix (10 errors, 3 files)
  PRE_GATE:
    - cargo check passes (lib code compiles, only test code fails)
    - cargo test 2>&1 | grep "^error\[" | wc -l returns 10
  TASK:
    Files:
      crates/runtime/src/mcp_stdio.rs
        L1413: use std::os::unix::fs::PermissionsExt  (wrap in #[cfg(unix)])
        L1456, 1502, 1636, 1761, 2680: permissions.set_mode(0o755) wrap in #[cfg(unix)]
        Add #[cfg(windows)] no-op (Windows has no file mode bits; executability implicit)
      crates/runtime/src/mcp_tool_bridge.rs
        L317: use std::os::unix::fs::PermissionsExt (wrap in #[cfg(unix)])
        L434: permissions.set_mode(0o755) (same pattern)
      crates/rusty-claude-cli/tests/mock_parity_harness.rs
        L4: use std::os::unix::fs::PermissionsExt (wrap in #[cfg(unix)])
        L425: permissions.set_mode(0o755) (same pattern)
    Pattern:
      #[cfg(unix)] { permissions.set_mode(0o755); fs::set_permissions(&path, permissions).expect("chmod"); }
      // Windows: .cmd/.ps1 scripts executable by default
  DETERMINISTIC_CHECK:
    - cargo test --no-run 2>&1 | grep "^error" | wc -l returns 0
    - grep -rn "use std::os::unix" crates/ --include="*.rs" | grep -v "#\[cfg(unix)\]" returns 0
  ADVERSARIAL:
    - cfg gates may break Linux path; Linux CI catches.
    - set_mode outside cfg block returns compile error.
  ACCEPTANCE: cargo test --no-run 0 compile errors across all crates.

P0.2 FIX RUNTIME FAILURES: PLUGIN HOOK TESTS (4 failures)
  PRE_GATE: P0.1 complete
  TASK:
    File: crates/plugins/src/hooks.rs (test module, L346-498)
    Root cause: tests create .sh scripts using printf/exit.
    Strategy: gate tests with #[cfg(unix)], add #[cfg(windows)] using .cmd or powershell.
    Failing tests:
      collects_and_runs_hooks_from_enabled_plugins (L395)
        write_hook_plugin creates hooks/pre.sh with #!/bin/sh
        Fix: .cmd on Windows (echo message)
      pre_tool_use_denies_when_plugin_hook_exits_two (L456)
        Hook: "printf 'blocked by plugin'; exit 2"
        Fix Windows: "cmd /c \"echo blocked by plugin & exit /b 2\""
      propagates_plugin_hook_failures (L473)
        Hook: "printf 'broken plugin hook'; exit 1"
        Fix Windows: "cmd /c \"echo broken plugin hook & exit /b 1\""
    write_hook_plugin (L362):
      #[cfg(unix)]: keep .sh
      #[cfg(windows)]: .cmd scripts with @echo off + echo message
    HookRunner must invoke correct shell per platform.
    Check run_hook_command (~L326); may hardcode /bin/sh -> add #[cfg(windows)] to use cmd.exe.
  DETERMINISTIC_CHECK:
    - cargo test -p plugins --lib 2>&1 | grep "FAILED" | wc -l returns 0
    - reports "33 passed; 0 failed"
  ADVERSARIAL:
    - If HookRunner hardcodes /bin/sh, Windows .cmd tests still fail. Grep /bin/sh in non-test hooks.rs.
    - .cmd echo trailing whitespace breaks comparison. Use .trim() or exact output.
  ACCEPTANCE: cargo test -p plugins --lib: 0 failures on Windows.

P0.3 FIX RUNTIME FAILURES: TOOLS TESTS (4 failures)
  PRE_GATE: P0.2 complete
  TASK:
    File: crates/tools/src/lib.rs (test module)
    Failing tests:
      glob_and_grep_tools_cover_success_and_errors (L6672): asserts path ends_with("nested/lib.rs") but Windows uses backslash. Fix: normalize, or use Path::ends_with.
      powershell_runs_via_stub_shell (L7069): calls chmod on stub path that doesn't exist on Windows. Fix: cfg-gate Unix chmod.
      repl_executes_python_code (L7007): Exit 9009 = "python3 is not recognized". Fix: "python" on Windows, "python3" on Unix.
      skill_loads_local_skill_prompt (L5745): PoisonError on env lock. Fix: dedicated mutex per test, catch_unwind, or serial_test, or isolate env access.
  DETERMINISTIC_CHECK:
    - cargo test -p tools --lib 2>&1 | grep "FAILED" | wc -l returns 0
    - reports "61 passed; 0 failed"
  ADVERSARIAL:
    - Path normalization to forward-slash may break Windows APIs. Use Path::ends_with or std::path::MAIN_SEPARATOR.
    - Env lock fix may mask real poison source. Run cargo test -p tools --lib 10 times; 0 failures.
  ACCEPTANCE: cargo test -p tools --lib: 0 failures on Windows.

P0.4 FIX WARNINGS (2 warnings)
  PRE_GATE: P0.3 complete
  TASK:
    crates/plugins/src/hooks.rs L2: unused import std::path::Path — remove or #[allow] if conditionally used.
    crates/runtime/src/hooks.rs L636: unused mut on command_builder — remove mut.
  DETERMINISTIC_CHECK:
    - cargo check 2>&1 | grep "^warning:" | grep -v "generated\|Finished\|build" | wc -l returns 0
  ADVERSARIAL: Path import only in cfg(unix) code — wrap import in same cfg gate.
  ACCEPTANCE: cargo check: 0 errors, 0 warnings.

P0.5 VERIFY CLEAN BASELINE
  PRE_GATE: P0.4 complete
  TASK:
    - cargo check: 0 errors, 0 warnings
    - cargo test: all tests pass
    - Record metrics to .elai_cc/metrics/pre_phase_0.toml:
      compile_time, test_total, test_pass, test_fail, loc_total, warnings
  DETERMINISTIC_CHECK:
    - cargo check 2>&1 | grep -c "error\|warning" returns 0 (excluding "generated N warnings")
    - cargo test 2>&1 | tail -1 contains "0 failed"
    - .elai_cc/metrics/pre_phase_0.toml exists and test_fail = 0
  ADVERSARIAL: cfg-gated tests drop pass count; compare to baseline 684, flag if significantly lower.
  ACCEPTANCE:
    - 0 errors, 0 warnings
    - 0 test failures
    - All cfg-gated tests documented

EXIT CRITERIA
  cargo check: 0 errors, 0 warnings on Windows.
  cargo test: 0 failures on Windows.
  No behavioral changes.
  All Unix-only tests cfg-gated (not deleted) and documented.

GROUNDING (verified 2026-04-06)
  "10 compile errors"               -> VERIFIED grep
  "set_mode 7 sites"                -> VERIFIED grep
  "std::os::unix 3 sites"           -> VERIFIED L1413, 317, 4
  "4 plugin test failures"          -> VERIFIED cargo test
  "4 tools test failures"           -> VERIFIED cargo test
  "2 warnings"                      -> VERIFIED
  "exit code 9009 = not recognized" -> VERIFIED (Windows missing command)
  "PoisonError from env lock"       -> VERIFIED at lib.rs:5745
  "61,317 LOC, 69 files, 684 tests" -> VERIFIED


==============================================================
PHASE 0: FORK AND STRIP (the walking skeleton)
==============================================================

Prerequisites: pre-phase-0 complete.

GOAL
claw-code-rust-port compiles as elai_cc with all unnecessary code removed.
One model (ollama), one FSM loop (IDLE->EXECUTING->IDLE), one tool (file read).

READ BEFORE IMPLEMENTING
  docs/specs/architecture-overview.txt Sections 1-4 (system identity, base, crates, traits)
  docs/specs/crates/core.txt Section 3 (STRIP/REPLACE list)
  docs/specs/crates/core.txt Section 4 (entropy quarantine traits)
  docs/specs/crates/providers.txt Section 2 (LlmProvider trait)
  docs/specs/decisions/base-fork-comparison.txt (full — why claw-code, module mapping)

RISKS
- R17: claw-code compilation verified (26.24s, 0 errors). Safe to proceed.
- R18: claw-code module quality unknown. Auditor reviews each module during strip.
- R19: runtime crate is 25,035 LOC across 38 files — largest restructure risk.
  Three monoliths: main.rs (8,615 LOC), commands/lib.rs (4,841 LOC), tools/lib.rs (7,345 LOC).
- R20: entropy quarantine scope is 60 occurrences across 31 files.
- R21: base-fork-comparison.txt LOC numbers stale (green_contract.rs listed as 4,061, actual is 152).
- R22: config.rs imports sandbox::{SandboxConfig, FilesystemIsolationMode}. Both move to core/ to avoid circular core/ -> orchestrator/.
- R23: TokenUsage/UsageTracker embedded in Session struct, conversation.rs, main.rs (61 occurrences across 5 files). Stripping usage.rs is structural refactor.
- R24: tools/lib.rs imports api::*. When api/ absorbed into providers/, tools/ imports must be updated simultaneously.
- R25: commands/lib.rs has 28+ claw-code slash commands (rewind, summary, bughunter, ultraplan, etc.) conflicting with elai_cc's /task, /config, /continue.

DESIGN DECISIONS

  STRIP BEFORE RESTRUCTURE
    Stripping first removes ~4K+ LOC of known-bad code (compact, summary_compression, telemetry, usage, lsp_client, compat-harness) before splitting runtime into 5 crates. Fewer files = fewer move operations = fewer dependency breaks during restructure.

  WORKSPACE PATH MIGRATION
    Steps 0.1-0.3: work in .elai_cc/rust/ (claw-code structure).
    Step 0.4: creates new structure at .elai_cc/crates/ with .elai_cc/Cargo.toml as workspace root.
    Steps 0.5+: all paths use .elai_cc/.
    Old .elai_cc/rust/ archived (not deleted) after 0.5 verifies.

  ONNX MODEL CANONICAL PATH
    ONNX embedding models stored at .elai_cc/models/onnx/ (canonical, pre-staged via setup script outside session). Setup: download + blake2 hash verify + write index_version.toml. Missing models at startup -> DEGRADED mode (BM25_ONLY), not crash. Visibility: .elai_cc/models/** hidden from agents.

  TOKENUSAGE HANDLING (addresses R23)
    usage.rs (313 LOC) STRIPPED, but TokenUsage is structural (embedded in Session).
    Strategy: define minimal elai_cc_core::TokenUsage { input_tokens, output_tokens, total_tokens, estimated_cost_usd }. Rewrite Session. Strip UsageTracker, CompactionConfig, format_usd, pricing_for_model, ModelPricing. elai_cc tracks tokens for context budget, not billing. Happens in step 0.2c as explicit sub-task.

  PROVIDER TRAIT MIGRATION (WARN-7)
    claw-code Provider trait (in api/src/providers/mod.rs) implemented by AnthropicClient, OpenAiCompatClient. elai_cc defines LlmProvider (complete, embed, token_count, token_count_estimate). Strategy: keep old Provider trait during Phase 0 as internal implementation detail. New OllamaProvider implements LlmProvider directly. Phase 1+ migrates Anthropic/OpenAI. Mark old with #[deprecated].


STEPS

0.1 BASELINE (record starting state)
  PRE_GATE:
    - pre-phase-0 complete: cargo check passes on .elai_cc/rust/
    - cargo --version returns 0
    - ls .elai_cc/rust/Cargo.toml returns 0
  TASK:
    - cargo check --workspace (baseline: 26.24s, 0 errors)
    - cargo test --workspace (run existing 684 tests)
    - Record baseline metrics: compile time, test pass/fail, total LOC, LOC per crate
    - Create .elai_cc/migration_manifest.toml: classify every .rs file as KEEP/STRIP/REPLACE/EXTEND with source crate, target crate, rationale columns. Use MODULE MAPPING table.
  DETERMINISTIC_CHECK:
    - cargo check --workspace 2>&1 | grep -c "error\[" returns 0
    - .elai_cc/metrics/phase_0_baseline.toml exists with: compile_time_s, test_pass, test_fail, test_total, loc_total, loc_by_crate
    - migration_manifest.toml covers all 69 .rs files (grep -c "KEEP\|STRIP\|REPLACE\|EXTEND" = 69)
  ADVERSARIAL: uncommitted changes skew baseline. git status clean before recording.
  ACCEPTANCE: baseline recorded; 100% coverage of .rs files; all classifications justified.

0.2 STRIP (remove known-bad code)
  PRE_GATE: 0.1 complete; migration_manifest has STRIP files.
  TASK (order: each removal followed by cargo check --workspace):
    a. telemetry/ crate (526 LOC): remove crate dir, workspace Cargo.toml, deps in other crates' Cargo.toml.
    b. compat-harness/ crate (~357 LOC): STRIP — TypeScript-bridge reading non-existent .ts paths. Callers in main.rs (extract_manifest, UpstreamPaths ~L1274): remove call sites.
    c. runtime/src/usage.rs (313) + runtime/src/compact.rs (696) + runtime/src/summary_compression.rs (300):
       STRUCTURAL STRIP — Session struct contains pub usage: Option<TokenUsage>; conversation.rs has 20 TokenUsage/UsageTracker refs; main.rs imports CompactionConfig, TokenUsage, UsageTracker, format_usd, pricing_for_model, ModelPricing (6+ imports, ~15 usage sites).
       Remediation:
         1. Define minimal TokenUsage struct in runtime/src/lib.rs (or new token_budget.rs): pub struct TokenUsage { pub input_tokens: u64, pub output_tokens: u64 } (temp shim — core::TokenUsage replaces in 0.4a).
         2. Update Session struct to use new minimal TokenUsage.
         3. Remove UsageTracker, CompactionConfig, format_usd, pricing_for_model, ModelPricing.
         4. main.rs: remove cost reporting, compaction config, billing displays.
         5. conversation.rs: remove compaction triggers, simplify usage tracking.
         6. Delete compact.rs, summary_compression.rs, usage.rs.
       cargo check --workspace after each sub-step.
    d. runtime/src/lsp_client.rs (747): STRIP.
    e. runtime/src/oauth.rs (596): STRIP. Mark DEFERRED.
    f. Audit runtime/src/prompt.rs (803): remove hardcoded system prompts. Known: SYSTEM_PROMPT_DYNAMIC_BOUNDARY (L39), "You are an interactive agent" (L451). REPLACE with minimal configurable prompt builder, or STRIP and rebuild Phase 1+.
    g. Classify commands/lib.rs slash commands (28+):
       KEEP: config, add-dir, api-key, stop, retry, paste, screenshot, search, system-prompt, temperature, max-tokens.
       STRIP: rewind, summary, bughunter, ultraplan, desktop, ide, tag, brief, advisor, stickers, insights, thinkback, release-notes.
       Record in migration_manifest.toml. Removal happens during restructure (0.4c).
    h. runtime/src/remote.rs (401): evaluate. Remote execution = DEFERRED. Cloud telemetry = STRIP.
    Note: "sentinel", "anti_distill", "bias_inject" do NOT exist in codebase (verified 2026-04-06).
  DETERMINISTIC_CHECK:
    - Stripped files no longer exist: telemetry, compat-harness, compact.rs, summary_compression.rs, usage.rs, lsp_client.rs
    - No dangling refs to stripped modules
    - Hardcoded prompts addressed: grep "You are an interactive agent" returns 0
    - Slash command audit recorded
    - cargo check --workspace: 0 errors
  ADVERSARIAL:
    - Re-exports hide dependencies. cargo check after each individual removal.
    - Crate in other crate's [dependencies]: remove dep entries BEFORE removing dirs.
    - prompt.rs removal may break REPL. cargo test --workspace after changes.
    - TokenUsage field name mismatch breaks deserialization. Phase 0 is fresh start.
  ACCEPTANCE:
    - All STRIP files removed
    - TokenUsage minimal shim (temporary)
    - cargo check: 0 errors
    - Slash command classification recorded
    - Each stripped file marked STRIPPED

0.3 VERIFY STRIP
  PRE_GATE: 0.2 complete; cargo check passes.
  TASK:
    - cargo test --workspace: record broken tests
    - For each: classify EXPECTED (stripped feature) or BUG (unintended)
    - BUG failures: fix before proceeding; document root cause
    - Update migration_manifest.toml: mark stripped files STRIPPED
    - Compute strip delta: LOC removed, files removed, tests removed
  DETERMINISTIC_CHECK:
    - cargo check: 0 errors
    - .elai_cc/metrics/phase_0_strip_delta.toml exists: loc_removed, files_removed, tests_removed, tests_broken_expected, tests_broken_bug
    - All BUG failures resolved
    - migration_manifest.toml STRIPPED count matches removed files
  ADVERSARIAL:
    - Tests depend on stripped modules via test helpers. Grep each failing test.
    - >10% of total tests removed indicates over-stripping.
  ACCEPTANCE: 0 errors; all failures classified; strip delta recorded.

0.4a RESTRUCTURE: LEAF CRATES (core, rules, providers)
  PRE_GATE: 0.3 complete; migration_manifest has target crate for surviving files.
  TASK:
    Create new workspace root at .elai_cc/Cargo.toml. Crates under .elai_cc/crates/. .elai_cc/rust/ remains as source; files COPIED then verified.
    1. core/ — extract from runtime:
       - config.rs (1648), session_control.rs (459), json.rs (358), bootstrap.rs (111)
       - Move sandbox.rs types SandboxConfig + FilesystemIsolationMode (R22) used by config.rs. sandbox.rs has zero internal deps — only std + serde. Options: (a) move entire sandbox.rs to core/ (385 LOC, clean), or (b) extract just two types into core/src/sandbox_types.rs. Option (a) simpler unless sandbox.rs has orchestrator-specific logic.
       - Define core::TokenUsage (replaces temp shim from 0.2c): pub struct TokenUsage { pub input_tokens: u64, pub output_tokens: u64 }
       - Remove session_control.rs import of crate::session::{Session, SessionError}. session_control only needs session IDENTITY. Session state management stays in orchestrator/. Split session_control.rs if needed: identity -> core/, state -> orchestrator/.
       cargo check --workspace.
    2. rules/ — extract from runtime:
       - policy_engine.rs (581), green_contract.rs (152), trust_resolver.rs (299)
       cargo check --workspace.
    3. providers/ — absorb api/ + extract from runtime:
       - Move api/src/ (4,377 LOC) into providers/src/
       - Move runtime/src/sse.rs (158) into providers/src/
       - Update all crates importing api:: to providers::
       - CRITICAL (R24): tools/lib.rs imports api::* — update simultaneously
       - Old Provider trait alongside new LlmProvider. Mark old #[deprecated].
       cargo check --workspace.
  DETERMINISTIC_CHECK:
    - 3 new crate dirs exist with Cargo.toml
    - cargo check: 0 errors
    - No remaining api:: imports
    - core/ has no non-leaf deps
  ADVERSARIAL:
    - config.rs still imports crate::sandbox after extraction = circular core/ -> orchestrator/. Move SandboxConfig/FilesystemIsolationMode to core/ BEFORE config.rs moves.
    - session_control.rs still imports crate::session = core/ depends on orchestrator/. Split first.
    - tools/lib.rs api:: imports not updated = cargo check fails. Update ALL api:: imports in single commit.
  ACCEPTANCE:
    - core/, rules/, providers/ exist and compile
    - core/ is true leaf (no internal deps except std + external)
    - No circular deps
    - Zero api:: imports

0.4b RESTRUCTURE: MID-LAYER CRATES (tools, plugins)
  PRE_GATE: 0.4a complete.
  TASK:
    4. tools/ — keep existing + extract from runtime:
       - file_ops.rs (762), bash.rs (336), bash_validation.rs (1004)
       - stale_branch.rs (417), branch_lock.rs (144)
       - Update Cargo.toml to depend on core/ (not runtime/)
       - tools/lib.rs (7,345): split into per-tool modules (file_tool.rs, bash_tool.rs, git_tool.rs) if >3 distinct tools.
       cargo check --workspace.
    5. plugins/ — keep existing + extract from runtime:
       - hooks.rs (987), plugin_lifecycle.rs (533)
       - Update Cargo.toml to depend on core/.
       cargo check --workspace.
    6. knowledge/ — empty skeleton (Phase 1 fills). Cargo.toml only, depends on core/.
    7. reasoning/ — empty skeleton (Phase 2 fills). Cargo.toml only, depends on core/.
  DETERMINISTIC_CHECK:
    - All 7 crate dirs exist
    - cargo check: 0 errors
    - tools/, plugins/ no longer depend on runtime/
  ADVERSARIAL:
    - tools/lib.rs unsplit = 7K LOC monolith. No single .rs in tools/ exceeds 2000 LOC post-restructure.
  ACCEPTANCE: tools/, plugins/ compile with core/ dep; knowledge/, reasoning/ skeletons exist.

0.4c RESTRUCTURE: TOP-LAYER CRATES (orchestrator, cli) + CLEANUP
  PRE_GATE: 0.4b complete.
  TASK:
    8. orchestrator/ — what remains of runtime after extractions:
       - conversation.rs (1690), session.rs (1246), worker_boot.rs (1083)
       - mcp_stdio.rs (2928), mcp_tool_bridge.rs (920), mcp_lifecycle_hardened.rs (843), mcp.rs (304), mcp_client.rs (248)
       - permissions.rs (683), permission_enforcer.rs (551)
       - sandbox.rs (385) — remaining sandbox logic if types split to core/
       - recovery_recipes.rs (631)
       - EVALUATE: lane_events.rs (383), task_packet.rs (158), task_registry.rs (503), team_cron_registry.rs (509). Criteria: maps to elai_cc concept (agents, FSM, sessions)? Yes = KEEP. claw-code "lane"/"team" specific = STRIP.
       - Depends on: core/, providers/, tools/, rules/
       cargo check --workspace.
    9. cli/ — absorb rusty-claude-cli/ + commands/:
       - main.rs (8,615) becomes cli/src/main.rs (shrinks significantly)
       - commands/lib.rs (4,841): strip claw-code-specific per 0.2g classification. Keep elai_cc-relevant. Move dispatch to cli/.
       - render.rs (796), init.rs (434), input.rs (330) -> cli/src/
       - mock-anthropic-service/ -> dev-dep of providers/ or cli/
       - Target: cli/src/main.rs < 500 LOC (thin REPL). If not achievable, document with justification.
       cargo check --workspace.
    10. Cleanup:
        - Archive .elai_cc/rust/ (rename to rust_archived/ or delete after verification)
        - Remove old crate dirs: api/, commands/, rusty-claude-cli/, runtime/
        - Update .elai_cc/Cargo.toml to exactly 9 crates
        - Verify no refs to old crate names
  DETERMINISTIC_CHECK:
    - Workspace has exactly 9 crate members
    - Each crate has Cargo.toml (core, providers, knowledge, reasoning, orchestrator, tools, plugins, rules, cli)
    - Old crate dirs removed/archived
    - cargo check: 0 errors
    - No circular deps
    - No old crate name refs
  ADVERSARIAL:
    - Mutual imports between extracted modules = circular dep. cargo check after each crate extraction.
    - main.rs contains orchestrator logic not extracted = cli/ too fat. Document cli/src/main.rs LOC; >1000 LOC flagged technical debt.
    - STRIP slash commands left in commands/lib.rs during merge = cli/ has dead code. Grep stripped command names post-merge.
  ACCEPTANCE:
    - 9 crate directories with Cargo.toml
    - 0 errors
    - No old dirs in active workspace
    - migration_manifest.toml updated
    - claw-code slash commands removed per 0.2g

0.5 VERIFY RESTRUCTURE
  PRE_GATE: 0.4c complete.
  TASK:
    - cargo test --no-run --workspace (link check)
    - cargo test --workspace
    - Classify failures as RESTRUCTURE_BUG (fix now) or PATH_UPDATE (fix import)
    - Fix all failures
    - Record metrics: compile time, test count, LOC per crate
  DETERMINISTIC_CHECK:
    - cargo check: 0 errors
    - cargo test --no-run: 0 link errors
    - .elai_cc/metrics/phase_0_restructure.toml: compile_time_s, test_pass, test_fail, loc_per_crate (map), total_crates=9
    - Zero old crate name refs in tests
  ADVERSARIAL: Integration tests using re-exports test wrong module. Grep all test files for old crate names. Must be 0.
  ACCEPTANCE: 0 errors; all non-stripped tests pass; metrics with per-crate LOC breakdown.

0.6 CONTRACT TESTS (skeleton)
  PRE_GATE: 0.5 complete; 9-crate structure confirmed.
  TASK:
    Create test-only crate: .elai_cc/crates/contracts/
      Cargo.toml: name = "elai_cc_contracts", [dev-dependencies] on all 9 crates.
      No lib.rs — only integration test files in crates/contracts/tests/.
      Add to workspace members.
    Contract tests (import from elai_cc_* crate names):
    - crates/contracts/tests/provider_trait.rs: LlmProvider trait exists in elai_cc_providers with complete/embed/token_count
    - crates/contracts/tests/fsm_basic.rs: FSM transitions IDLE -> EXECUTING -> IDLE
    - crates/contracts/tests/tool_basic.rs: ToolRegistry registers and executes file_read
    - crates/contracts/tests/config_basic.rs: elai_cc.toml parses correctly; round-trips via msgpack
    - crates/contracts/tests/visibility_filter.rs:
      - hidden path returns NotFound identical to missing
      - glob on hidden returns empty
      - .elai_cc/** covered by default hidden_paths
      - .elai_cc/models/** hidden (ONNX)
    All tests FAIL at runtime (TDD red phase).
  DETERMINISTIC_CHECK:
    - contracts/Cargo.toml exists
    - 5 test files exist
    - cargo test -p elai_cc_contracts --no-run (compiles, returns 0)
    - cargo test -p elai_cc_contracts: 5 FAILED
  ADVERSARIAL:
    - Tests accidentally pass on claw-code stubs undermines TDD. Imports must reference elai_cc_* crate names.
    - Tests fail to compile vs fail at runtime: --no-run must succeed.
  ACCEPTANCE: 5 test files; all fail at runtime; imports use elai_cc_*.

0.7a IMPLEMENT PROVIDER + CONFIG
  PRE_GATE: 0.6 complete; 5 FAILED.
  TASK:
    - Extend LlmProvider trait in providers/:
        async fn complete(&self, prompt: &str, config: &CompletionConfig) -> Result<impl Stream<Item=Token>>;
        async fn embed(&self, text: &str) -> Result<Vec<f32>>;
        fn token_count(&self, text: &str) -> usize;
        fn token_count_estimate(&self, text: &str) -> usize;
      &self. Stateful providers hold HTTP client internally.
    - OllamaProvider: HTTP wrapper (POST /api/generate, POST /api/embed). Configurable base_url (default http://localhost:11434). Newline-delimited JSON streaming.
    - elai_cc.toml config schema in core/: provider config + project settings + agent role configs. Parse with toml crate. Round-trip via msgpack (rmp-serde).
  DETERMINISTIC_CHECK:
    - provider_trait contract test passes
    - config_basic contract test passes
    - OllamaProvider base_url configurable (grep shows >= 1 hit)
  ADVERSARIAL:
    - Hardcoded localhost:11434 = tests fail without ollama. Use mock HTTP server or stub.
    - msgpack drops Option::None silently. Round-trip config with Option::None.
    - token_count_estimate panics on empty input. Return 0, not panic.
  ACCEPTANCE: 2 contract tests pass; base_url configurable; msgpack round-trip lossless.

0.7b IMPLEMENT FSM + TOOL
  PRE_GATE: 0.7a complete.
  TASK:
    - Minimal FSM in orchestrator/: IDLE, EXECUTING only
      Transitions: IDLE -> EXECUTING (task start), EXECUTING -> IDLE (complete/error)
      State as enum. Invalid transitions return Error, not panic.
      Methods: get_state(), transition(), get_active_tools()
    - Minimal file_read tool in tools/:
      Reads with metadata header (total_lines, estimated_tokens via token_count_estimate)
      Respects visibility filter: hidden paths return NotFound indistinguishable from missing
    - Replace JSON session storage with msgpack in orchestrator/:
      Session state via rmp-serde. Verify session files on disk use msgpack (session.rs used custom JsonValue).
      Round-trip test: write, read back, assert equality. Check file is valid msgpack.
  DETERMINISTIC_CHECK:
    - fsm_basic contract test passes
    - tool_basic contract test passes
    - Session file msgpack (python msgpack.unpack succeeds)
  ADVERSARIAL:
    - FSM invalid transition panics vs returns Error. fsm_basic attempts IDLE -> IDLE, asserts Error.
    - file_read follows symlinks bypasses visibility. Symlink to hidden path returns NotFound.
    - Corrupted msgpack panics. Feed garbage, assert Error.
  ACCEPTANCE: fsm_basic + tool_basic pass; FSM returns Error on invalid; file_read respects visibility; session msgpack verified.

0.7c IMPLEMENT INTERFACE HARDENING
  PRE_GATE: 0.7b complete (4 contract tests pass).
  TASK:
    - Slash-command routing in cli/: /task, /config, /continue, /explore, /status
      Fallback classifier for bare messages (defaults to /task)
      Each command maps to handler. Unknown -> error message.
      Build on KEEP commands from 0.2g; strip others.
    - Visibility filter in tools/: per-agent allowed_paths + hidden_paths
      Path canonicalization before matching (block ../ traversal)
      Hidden path NotFound identical to genuinely missing
      Default hidden_paths: .elai_cc/models/** (ONNX), rules.db, harness source, other agent checkpoints, SSB database, elai_cc binary internals.
    - Error message sanitization: strip internal identifiers from model-facing output (allowlist, not denylist)
    - Tool schema sanitization: strip internal-only parameters before showing to model
    - tracing-based logging (replaces telemetry)
      tracing + tracing-subscriber. Log every tool call, FSM transition, file operation.
  DETERMINISTIC_CHECK:
    - visibility_filter contract test passes
    - Slash-command tests pass
    - No internal IDs leak (grep rule_id/predicate_type/hook_name in model-facing output returns 0)
  ADVERSARIAL:
    - Prefix matching bypass via ../. Canonicalize then match. Test ../../../etc/passwd equivalent.
    - Denylist sanitization leaks new fields by default. Use allowlist.
    - Tracing subscriber not initialized in test harness = silent. tracing_test::traced_test or equivalent.
  ACCEPTANCE:
    - All 5 contract tests pass
    - Visibility filter blocks traversal
    - Sanitization allowlist-based
    - Tracing captures tool calls and FSM transitions

0.8 VERIFY IMPLEMENTATION
  PRE_GATE: 0.7c complete.
  TASK:
    - All contract tests pass
    - cargo check --workspace clean
    - Smoke test: ollama round-trip (WARN not BLOCK if unavailable)
    - Record metrics
  DETERMINISTIC_CHECK:
    - cargo test -p elai_cc_contracts: 0 FAILED
    - cargo check: 0 errors
    - .elai_cc/scripts/smoke_test_ollama.sh exists and returns 0 (or SKIP)
    - .elai_cc/metrics/phase_0_implement.toml: compile_time_s, test_total, test_pass, loc_delta
  ADVERSARIAL: Shared-state failures when run together. cargo test --workspace AND -p contracts both pass.
  ACCEPTANCE: all contract pass; 0 errors; metrics; Ollama round-trip succeeds or documented skip.

0.9 ENTROPY QUARANTINE (deterministic simulation prerequisite)
  PRE_GATE: 0.8 complete.
  TASK:
    READ: docs/specs/crates/core.txt section 4
    - Define traits in core/:
        trait ClockProvider {
          fn now(&self) -> LogicalInstant;
          fn elapsed_since(&self, mark: LogicalInstant) -> Duration;
          fn system_now(&self) -> SystemTime;  // for cache expiry, HTTP headers
        }
        trait EntropySource {
          fn next_u64(&mut self) -> u64;
          fn fill_bytes(&mut self, buf: &mut [u8]);
        }
      (&self on ClockProvider, &mut self on EntropySource)
    - Production impls in core/:
        OsClock: delegates to std::time::Instant
        OsRng: delegates to rand::thread_rng()
    - Test impls in core/:
        SeededClock: logical clock, advances only via explicit tick(). Monotonic.
        SeededRng: deterministic PRNG from u64 seed (ChaCha8Rng::seed_from_u64)
    - Replace ALL raw time/entropy access. 60 occurrences, 31 files (pre-restructure; recount post-restructure).
      Known high-density (post-restructure):
        providers/prompt_cache.rs: 4 SystemTime::now() (cache expiry)
        tools/lib.rs: 14 Instant::now() (tool execution timing)
        cli/main.rs: 5 Instant::now() (session timing)
        providers/anthropic.rs: 2 SystemTime::now() (retry timing)
        orchestrator/session_control.rs: 3 Instant::now()
      Approach: bundle clock + entropy in Context struct passed through constructors:
        struct RuntimeContext { clock: Arc<dyn ClockProvider>, entropy: Arc<Mutex<dyn EntropySource>> }
      Avoids function signature proliferation (max 1 extra param).
    - Legitimate production exceptions:
        core/src/clock.rs: OsClock impl (ONLY Instant::now / SystemTime::now)
        core/src/entropy.rs: OsRng impl (ONLY thread_rng / getrandom)
      All other sites must use trait. CI script EXCLUDES these.
    - CI enforcement: .elai_cc/scripts/check_entropy_quarantine.sh
      Greps banned patterns, excludes core/src/clock.rs, core/src/entropy.rs, TEST_KEEP files.
    - Audit transitive deps: cargo tree -e features | grep "time\|rand\|getrandom"
  DETERMINISTIC_CHECK:
    - grep "Instant::now\(\)\|SystemTime::now\(\)" .elai_cc/crates/ returns hits ONLY in core/src/clock.rs (and TEST_KEEP)
    - grep "thread_rng\|OsRng::new\|getrandom" .elai_cc/crates/ ONLY in core/src/entropy.rs (and TEST_KEEP)
    - trait ClockProvider defined (1 occurrence in core/)
    - trait EntropySource defined (1 occurrence in core/)
    - bash check_entropy_quarantine.sh returns 0
    - struct RuntimeContext defined in core/
  ADVERSARIAL:
    - Transitive dep calls libc::clock_gettime. cargo tree documented; known holes listed.
    - SeededClock non-monotonic. 100 sequential now() calls strictly monotonic.
    - RuntimeContext Arc<Mutex<>> contention. Benchmark single-threaded tool exec with/without mutex. Accept if <5% overhead.
    - Test needs real time. TEST_KEEP files use OsClock explicitly, excluded.
  ACCEPTANCE:
    - Raw time/entropy ONLY in core/ impls (plus TEST_KEEP)
    - CI script catches new violations
    - SeededClock + SeededRng pass contract tests when injected
    - RuntimeContext bundles (no function has >1 extra param for time/randomness)

0.10 VERIFY ENTROPY QUARANTINE + DETERMINISM PROOF
  PRE_GATE: 0.9 complete.
  TASK:
    - Run CI quarantine check
    - Test suite with SeededClock + SeededRng (ELAI_SEED=42): all pass
    - Run same suite twice with same seed, single-threaded: diff stdout = 0
    - This is determinism proof for Phase 3I (DST)
  DETERMINISTIC_CHECK:
    - .elai_cc/scripts/check_entropy_quarantine.sh returns 0
    - cargo test --workspace ELAI_SEED=42: passes
    - Run twice --test-threads=1: diff = 0
  ADVERSARIAL:
    - Timestamps from non-quarantined path: diff passes but replay diverges. Grep test output for SystemTime patterns (ISO 8601, epoch millis). Must be 0.
    - Parallel test ordering differs. Use --test-threads=1.
  ACCEPTANCE:
    - Zero raw entropy outside core/ impls
    - All tests pass with seeded providers
    - Same seed + same threads = identical output


MODULE MAPPING (runtime/ -> target crates; LOC verified 2026-04-06)

TO core/ (leaf, no internal deps):
  config.rs (1648)         Config parsing and validation
  session_control.rs (459) Session identity (SPLIT: identity -> core/, state -> orchestrator/)
  json.rs (358)            JSON utilities (evaluate STRIP if replaced by msgpack)
  bootstrap.rs (111)       Initialization sequence
  sandbox.rs (385)         SandboxConfig + FilesystemIsolationMode (R22)

TO rules/ (depends on core/):
  policy_engine.rs (581)   Rule evaluation (REPLACE with RuleEngine in Phase 3F)
  green_contract.rs (152)  Evidence gates (KEEP)
  trust_resolver.rs (299)  Trust level resolution

TO providers/ (depends on core/):
  sse.rs (158)             Server-sent events parsing
  api/src/* (4377)         Provider implementations + types + prompt_cache
  NOTE: api/src/sse.rs (279) duplicates runtime/src/sse.rs (158). Keep api/ version.

TO tools/ (depends on core/, knowledge/):
  file_ops.rs (762)        File operations
  bash.rs (336)            Bash execution
  bash_validation.rs (1004) Input validation
  stale_branch.rs (417)    Git branch management
  branch_lock.rs (144)     Git branch locking
  tools/src/* (7526)       Existing tool implementations

TO plugins/ (depends on core/, rules/):
  hooks.rs (987)           Hook system
  plugin_lifecycle.rs (533) Plugin management
  plugins/src/* (3860)     Existing plugin code

TO orchestrator/ (depends on core/, providers/, reasoning/, rules/, tools/):
  conversation.rs (1690)   Conversation management
  session.rs (1246)        Session state + persistence
  worker_boot.rs (1083)    Agent/worker lifecycle
  mcp_stdio.rs (2928)      MCP stdio transport
  mcp_tool_bridge.rs (920) MCP tool integration
  mcp_lifecycle_hardened.rs (843) MCP lifecycle
  mcp.rs (304)             MCP core types
  mcp_client.rs (248)      MCP client
  permissions.rs (683)     Permission model
  permission_enforcer.rs (551) Permission enforcement
  recovery_recipes.rs (631) Crash recovery
  lane_events.rs (383)     EVALUATE
  task_packet.rs (158)     EVALUATE
  task_registry.rs (503)   EVALUATE
  team_cron_registry.rs (509) EVALUATE

TO cli/ (depends on core/, orchestrator/, tools/, providers/):
  rusty-claude-cli/src/main.rs (8615) REPL main loop (target <500 thin shell)
  rusty-claude-cli/src/render.rs (796) Output rendering
  rusty-claude-cli/src/init.rs (434)   CLI initialization
  rusty-claude-cli/src/input.rs (330)  Input handling
  commands/src/lib.rs (4841)  Slash commands (28+ classified, strip claw-code)

STRIP (removed in 0.2):
  compact.rs (696)              Context compaction
  summary_compression.rs (300)  Summary compression
  usage.rs (313)                Usage/billing tracking
  lsp_client.rs (747)           LSP client
  oauth.rs (596)                OAuth (DEFERRED Phase 4+)
  prompt.rs (803)               Hardcoded prompts
  remote.rs (401)               EVALUATE
  telemetry/src/lib.rs (526)    External telemetry (replaced by tracing)
  compat-harness/src/lib.rs (~357) TypeScript bridge

TEST INFRASTRUCTURE (dev-deps):
  mock-anthropic-service/ (~180) Mock provider -> dev-dep of providers/ or cli/


EXIT CRITERIA
  All 5 contract tests pass (crates/contracts/).
  Ollama round-trip works (or documented skip).
  No stripped code remains.
  Zero raw time/entropy access outside core/ impls (entropy quarantine verified, determinism proved).
  9-crate structure with no circular dependencies.
  claw-code slash commands stripped per classification.

MEASUREMENTS
  - Compile time per checkpoint (target: <30s after restructure)
  - Test count: baseline, post-strip, post-restructure, post-implement
  - LOC total and per-crate
  - Strip delta: files removed, LOC removed, tests removed
  - Ollama round-trip latency (baseline)
  - Raw entropy access count outside core/: 0

GROUNDING
  "26.24s compile time"             -> GROUNDED
  "684 tests"                       -> VERIFIED (2026-04-06)
  "61,317 LOC, 69 .rs files"        -> VERIFIED
  "custom clippy lints"             -> UNGROUNDED: replaced with CI grep script
  "~200 LOC for entropy quarantine" -> UNGROUNDED: actual 60 occurrences / 31 files / 300-500 changed LOC
  "claw-code-rust-port source"      -> VERIFIED (.elai_cc/rust/)
  "sentinel/anti_distill/bias"      -> VERIFIED ABSENT
  "60 occurrences / 31 files"       -> VERIFIED
  "hardcoded system prompts"        -> VERIFIED (prompt.rs L39, L451)
  "runtime is 25,035 LOC / 38 files" -> VERIFIED
  "main.rs is 8,615 LOC"            -> VERIFIED
  "green_contract.rs is 152 LOC"    -> VERIFIED (base-fork-comparison says 4,061 STALE)
  "commands/lib.rs is 4,841 LOC"    -> VERIFIED
  "api/ source is 4,377 LOC"        -> VERIFIED
  "config.rs imports sandbox types" -> VERIFIED (L7)
  "sandbox.rs has zero internal deps" -> VERIFIED (only std + serde)
  "TokenUsage embedded in 5 files"  -> VERIFIED (61 occurrences)
  "28+ slash commands"              -> VERIFIED
  "tools/lib.rs imports api::*"     -> VERIFIED (L6, L5051)
  "SystemTime::now in api/ = 6"     -> VERIFIED (prompt_cache 4, anthropic 2)
  "compat-harness imports commands+runtime" -> VERIFIED (L4-6)

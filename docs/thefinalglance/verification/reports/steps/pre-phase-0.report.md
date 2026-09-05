# Verification Report: pre-phase-0
Source: F:\.ELAI_workflow\docs\specs\guide\steps\pre-phase-0.txt
Date: 2026-05-17
Overall status: MOSTLY_DONE
Confidence: HIGH

## Summary
Pre-phase-0 specifies Windows platform compatibility fixes for five sub-tasks: cfg-gate PermissionsExt/set_mode sites (P0.1), add Windows-equivalent plugin hook tests (P0.2), fix four tools test failures (P0.3), remove two warnings (P0.4), and record a clean baseline metrics file (P0.5). The core fixes are fully implemented across mcp_stdio.rs, mcp_tool_bridge.rs, hooks.rs, and tools/lib.rs. The one gap is the third file named in P0.1 (crates/rusty-claude-cli/tests/mock_parity_harness.rs) which does not exist in the current codebase — that crate was never ported into .elai_cc/. The metrics file exists but covers only the B-15A sub-scope, explicitly excluding pre-existing failures.

## Deliverables verified DONE

- P0.1 — mcp_stdio.rs: PermissionsExt gated #[cfg(unix)] — evidence: crates/orchestrator/src/mcp_stdio.rs:1417-1418
- P0.1 — mcp_stdio.rs: all five set_mode(0o755) calls wrapped in #[cfg(unix)] blocks — evidence: crates/orchestrator/src/mcp_stdio.rs:1468,1517,1654,1782,2704
- P0.1 — mcp_tool_bridge.rs: PermissionsExt gated #[cfg(unix)] — evidence: crates/orchestrator/src/mcp_tool_bridge.rs:324-325
- P0.1 — mcp_tool_bridge.rs: set_mode(0o755) wrapped in #[cfg(unix)] — evidence: crates/orchestrator/src/mcp_tool_bridge.rs:441,444
- P0.2 — plugins/src/hooks.rs: write_hook_plugin writes .sh on unix and .cmd on windows — evidence: crates/plugins/src/hooks.rs:373-423
- P0.2 — pre_tool_use_denies_when_plugin_hook_exits_two: platform-dispatched hook command — evidence: crates/plugins/src/hooks.rs:490-493
- P0.2 — propagates_plugin_hook_failures: platform-dispatched hook command — evidence: crates/plugins/src/hooks.rs:512-521
- P0.2 — HookRunner shell dispatch (plugins/src/hooks.rs): #[cfg(windows)]/#[cfg(not(windows))] in shell_command — evidence: crates/plugins/src/hooks.rs:283-301
- P0.2 — HookRunner shell dispatch (orchestrator/src/hooks.rs): shell_command cfg-gated cmd vs sh — evidence: crates/orchestrator/src/hooks.rs:758-771
- P0.3 — glob_and_grep path assertion: uses std::path::Path::new(...).ends_with("nested/lib.rs") — evidence: crates/tools/src/lib.rs:7133-7138
- P0.3 — powershell_runs_via_stub_shell: gated #[cfg(unix)] — evidence: crates/tools/src/lib.rs:7506-7507
- P0.3 — repl_executes_python_code: detect_first_command tries "python" first on windows — evidence: crates/tools/src/lib.rs:4889-4891
- P0.3 — skill_loads_local_skill_prompt: uses env_lock().unwrap_or_else(PoisonError::into_inner) — evidence: crates/tools/src/lib.rs:6183-6185
- P0.4 — plugins/src/hooks.rs: Path import gated #[cfg(not(windows))] — evidence: crates/plugins/src/hooks.rs:2-3
- P0.4 — orchestrator/src/hooks.rs: command_builder mut pattern does not produce dangling mut (cfg-scoped blocks) — evidence: crates/orchestrator/src/hooks.rs:760-769
- P0.5 — metrics/pre_phase_0.toml exists with compile_errors=0, test_failures=0, warnings=0, platform="windows" — evidence: .elai_cc/metrics/pre_phase_0.toml:28-33

## Deliverables PARTIAL

- P0.5 — metrics file completeness
  - exists: .elai_cc/metrics/pre_phase_0.toml:28-50
  - gap: The spec mandates test_total and cfg-gated test documentation. The file explicitly excludes the 40 pre-existing Windows shell-escape failures from test_failures with a note, but does not record test_total or a list of cfg-gated tests per P0.5 DETERMINISTIC_CHECK (compare test_total against baseline 684). The field test_fail=0 with a prose caveat is not the same as verified_test_total matching 684.

## Deliverables MISSING

- P0.1 — crates/rusty-claude-cli/tests/mock_parity_harness.rs: cfg-gate for PermissionsExt at line 4 and set_mode at line 425 — searched: "rusty-claude-cli", "mock_parity_harness" in elai.map and filesystem find; that crate does not exist in .elai_cc/crates/ (only present as a string literal inside compact.rs test data at line 682)

## Deliverables UNVERIFIABLE

- Cargo check / cargo test runtime outcomes (0 errors, 0 warnings, 0 failures on Windows) — reason: auditor does not run builds; claimed in metrics/pre_phase_0.toml notes but cannot be verified without executing cargo

## What must be done (action list)

- [ ] Determine whether rusty-claude-cli was intentionally excluded from .elai_cc/; if the crate exists elsewhere, apply the P0.1 cfg-gates to crates/rusty-claude-cli/tests/mock_parity_harness.rs lines 4 and 425
- [ ] Add test_total field to .elai_cc/metrics/pre_phase_0.toml and verify it matches or explains deviation from baseline 684 (P0.5 DETERMINISTIC_CHECK)
- [ ] Add cfg_gated_tests list to pre_phase_0.toml documenting which tests are gated and why (P0.5 ACCEPTANCE criteria)

## Notes

The collects_and_runs_hooks_from_enabled_plugins test (P0.2) is not cfg-gated to #[cfg(unix)] as the spec originally suggested; instead the write_hook_plugin helper dispatches per platform. This is a valid implementation strategy that runs the test on both platforms with platform-appropriate scripts.

The mcp_tool_bridge.rs python dispatch fix (line 458: `if cfg!(target_os = "windows") { "python" }`) also addresses the P0.3 python3 issue at the MCP server config level in addition to the REPL resolve_repl_runtime fix.

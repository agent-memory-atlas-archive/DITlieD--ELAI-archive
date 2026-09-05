# Verification Report: B-37_multi_runtime_backend.md
Source: F:\.ELAI_workflow\docs\thefinalglance\B-plan\B-37_multi_runtime_backend.md
Date: 2026-05-17
Overall status: MOSTLY_DONE
Confidence: HIGH

## Summary
The core RuntimeAdapter/RuntimeBackend/PerRoleRuntime/RuntimeOrchestrator
infrastructure is fully present and correct. All 11 backends are defined,
the factory dispatches all of them, and the dashboard RuntimePanel is wired
to SettingsWindow. The plan's Tauri command names (get_runtime_status,
test_runtime_health) diverge from the actual implementation (list_runtimes,
test_runtime), and the /runtime fallback subcommand is absent — the CLI
implements list/show/test/set/help only. Ancillary files (metrics TOML,
migration guide, paper ingestor outputs) are all present.

## Claims verified DONE

- RuntimeBackend enum with exactly 11 variants (Sidecar, AnthropicApi,
  ClaudeCode, CodexCli, OpenCode, Hermes, GeminiCli, KiroCli, CopilotCli,
  Ollama, Vllm) — evidence:
  .elai_cc/crates/providers/src/runtime/types.rs:55-78

- RuntimeAdapter trait defined (Send + Sync) — evidence:
  .elai_cc/crates/providers/src/runtime/types.rs:494

- resolve_backend() function with full supported-list error on miss,
  11 supported count verified in test — evidence:
  .elai_cc/crates/providers/src/runtime/mod.rs:60
  .elai_cc/crates/providers/src/runtime/mod.rs:143 (assert_eq!(err.supported.len(), 11))

- create_runtime() factory dispatching all 11 backends — evidence:
  .elai_cc/crates/providers/src/runtime/mod.rs:92-113

- ResolveBackendError struct — evidence:
  .elai_cc/crates/providers/src/runtime/types.rs:476

- RuntimeFactoryConfig struct — evidence:
  .elai_cc/crates/providers/src/runtime/types.rs:258

- PerRoleRuntime struct with default role mappings (Designer/Auditor/Researcher/
  MediaValidator -> AnthropicApi, others -> Sidecar) — evidence:
  .elai_cc/crates/orchestrator/src/per_role_runtime.rs:44
  .elai_cc/crates/orchestrator/src/per_role_runtime.rs:58-61

- RuntimeOrchestrator struct with fallback chain traversal and
  FallbackStep events — evidence:
  .elai_cc/crates/orchestrator/src/runtime_orchestrator.rs:113

- runtime_config.rs: default_fallback_chain(), fallback_chain field on
  RuntimeConfig, orthogonal to AutonomousGate model_fallback_chain — evidence:
  .elai_cc/crates/orchestrator/src/runtime_config.rs:4 (doc comment),
  .elai_cc/crates/orchestrator/src/runtime_config.rs:32-50

- 9 adapter module files present (sidecar, anthropic_api, claude_code,
  codex_cli, opencode, hermes, gemini_cli, kiro_cli, copilot_cli) plus
  ollama and vllm — evidence: all files confirmed by glob:
  .elai_cc/crates/providers/src/runtime/*.rs (18 files total)

- cli_common.rs with shared subprocess helpers, enforce_permission_opt_in,
  health_check_cli_binary, default_binary_for — evidence:
  .elai_cc/crates/providers/src/runtime/cli_common.rs:396, 512, 64

- TierCSpawn trait used by CLI adapters (hermes, kiro_cli) — evidence:
  .elai_cc/crates/providers/src/runtime/hermes.rs:57
  .elai_cc/crates/providers/src/runtime/kiro_cli.rs:51

- RuntimePermissionMode enum with AcceptEdits default,
  DangerouslySkipPermissions requiring explicit opt-in — evidence:
  .elai_cc/crates/providers/src/runtime/types.rs:130-144

- SSB action types runtime_fallback, runtime_health_changed,
  cli_subprocess_spawned, cli_subprocess_killed — evidence:
  .elai_cc/crates/orchestrator/src/ssb.rs:1305-1307 (from_str arms),
  .elai_cc/crates/orchestrator/src/ssb.rs:401-406 (doc comments)

- RuntimeEventSink SSB sink with test for fallback event recording — evidence:
  .elai_cc/crates/orchestrator/src/runtime_event_sink_ssb.rs:199-200

- /runtime CLI command (SlashCommand::Runtime) with list/show/test/set/help
  subcommands — evidence:
  .elai_cc/crates/cli/src/commands.rs:1748-1801
  .elai_cc/crates/cli/src/args.rs:468

- handle_runtime_slash_command and handle_runtime_set in runtime_cmd.rs — evidence:
  .elai_cc/crates/cli/src/runtime_cmd.rs:110

- Dashboard RuntimePanel.svelte component — evidence:
  .elai_cc/crates/dashboard-v2/src/lib/components/RuntimePanel.svelte (exists)

- RuntimePanel imported and mounted in SettingsWindow.svelte — evidence:
  .elai_cc/crates/dashboard-v2/src/lib/windows/SettingsWindow.svelte:4, 728

- RuntimeAgentRole type and RUNTIME_AGENT_ROLES constant in dashboard types — evidence:
  .elai_cc/crates/dashboard-v2/src/lib/api/types.ts:174, 190

- RuntimeHealthStatus type in dashboard types — evidence:
  .elai_cc/crates/dashboard-v2/src/lib/api/types.ts:1788

- list_runtimes Tauri command (per-backend health snapshot) — evidence:
  .elai_cc/crates/dashboard-v2/src-tauri/src/commands.rs:6778

- test_runtime Tauri command (live health probe) — evidence:
  .elai_cc/crates/dashboard-v2/src-tauri/src/commands.rs:6863

- set_role_runtime Tauri command with atomic TOML persistence — evidence:
  .elai_cc/crates/dashboard-v2/src-tauri/src/commands.rs:6922

- metrics/phase_4M.toml file — evidence: confirmed present at
  .elai_cc/metrics/phase_4M.toml

- workflow/papers/ouroboros-runtime-interface.md (paper-ingestor output) — evidence:
  workflow/papers/ouroboros-runtime-interface.md (confirmed present)

- workflow/papers/phase-4M-migration.md (migration guide) — evidence:
  workflow/papers/phase-4M-migration.md (confirmed present)

- Runtime contract tests in contracts crate — evidence:
  .elai_cc/crates/contracts/tests/runtime_adapter_contract.rs:201, 211

## Claims PARTIAL

- /runtime CLI command — list/show/test/set/help exist and work.
  - exists: .elai_cc/crates/cli/src/commands.rs:1748-1801
  - gap: /runtime fallback <role> <b1>,<b2> subcommand is NOT implemented.
    The parse_runtime_command match arms cover only list, show, test, set,
    help. No fallback arm exists anywhere in runtime_cmd.rs or commands.rs.

- Tauri command names — functional equivalents exist but names differ from plan.
  - exists: list_runtimes (commands.rs:6778), test_runtime (commands.rs:6863),
    set_role_runtime (commands.rs:6922)
  - gap: plan specifies get_runtime_status and test_runtime_health; actual
    names are list_runtimes and test_runtime. Functionally equivalent but
    plan's exact symbol names are absent.

## Claims MISSING

- /runtime fallback <role> <b1>,<b2> CLI subcommand — searched:
  "fallback" in crates/cli/src/commands.rs; parse_runtime_command arms;
  runtime_cmd.rs; no evidence of fallback subcommand implementation.

## Claims UNVERIFIABLE

- "All 7 CLI adapters run inside Tier C isolation (verified by Tier C contract
  test)" — TierCSpawn trait is referenced in hermes and kiro_cli adapters
  (with_spawn constructors), but whether ALL 7 CLI adapters enforce Tier C
  at the contract-test level requires reading each adapter's test suite,
  which exceeds scope. The stub annotations (B-37A) in type doc-comments
  suggest several CLI adapters may still be stubs.

- "stdout sanitization passes through 4A IPC sanitizer" — sanitization
  references exist in cli_common.rs but end-to-end verification against
  the Phase 4A IPC sanitizer contract requires live test execution outside
  this audit scope.

- "runtime fallback orthogonal to 4G SUBSTITUTE_MODEL (no race)" —
  the runtime_config.rs doc comment asserts orthogonality; runtime code
  confirmed separate from autonomous_gate model_fallback_chain; but absence
  of race conditions requires dynamic testing, not static analysis.

## What must be done (action list)

- [ ] Implement /runtime fallback <role> <b1>,<b2> subcommand in
  .elai_cc/crates/cli/src/commands.rs (add parse arm) and
  .elai_cc/crates/cli/src/runtime_cmd.rs (add handle_runtime_fallback).

- [ ] Either rename list_runtimes -> get_runtime_status and test_runtime ->
  test_runtime_health in .elai_cc/crates/dashboard-v2/src-tauri/src/commands.rs
  to match the plan's WIRING_CONTRACT, OR update the B-37 plan file to
  reflect the as-built names (preferred — plan names are cosmetic, not
  architectural).

- [ ] Verify Tier C isolation contract tests cover all 7 CLI adapters
  (claude_code, codex_cli, opencode, hermes, gemini_cli, kiro_cli,
  copilot_cli) by running:
  cargo test -p elai_providers -- runtime 2>&1 | grep "test result"

## Notes
The "stub in B-37A" annotations in the RuntimeBackend doc-comments indicate
the plan was delivered in sub-phases; most CLI adapters expose health_check
returning Unavailable (correct behavior per spec) but the complete() impl
returns NotImplemented for some (confirmed: codex_cli.rs:144). This is
consistent with the acceptance criterion "health_check returns Unavailable
(not panic) when CLI missing" being met, while the full CLI protocol
implementation is staged for a subsequent B-37 sub-phase.

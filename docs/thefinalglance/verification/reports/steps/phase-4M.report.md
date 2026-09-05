# Verification Report: phase-4M.txt
Source: F:\.ELAI_workflow\docs\specs\guide\steps\phase-4M.txt
Date: 2026-05-17
Overall status: MOSTLY_DONE
Confidence: HIGH

## Summary
Phase 4M (B-37 Multi-Runtime Backend Abstraction) is substantially complete. All core
infrastructure deliverables are present: RuntimeBackend (11 variants), RuntimeAdapter
trait, RuntimeFactoryConfig, PermissionMode, ResolveBackendError, RuntimeCapabilities,
RuntimeHealth, PerRoleRuntime, RuntimeOrchestrator, all 7 CLI subprocess adapters (Tier C
isolated), both native HTTP adapters (Ollama, Vllm), both existing adapters (Sidecar,
AnthropicApi), /runtime CLI (list/test/set/help), SSB fallback events, source-grounding
doc, and migration guide. Two gaps exist: (1) the /runtime fallback subcommand is not
implemented (spec 4M.5: /runtime fallback <role> <b1,b2>); (2) the Dashboard RuntimePanel
Tauri commands are deferred (metrics self-report: pending_wave2); (3) the phase_4M.toml
metrics file exists but is missing the spec-required per-backend p50/p99/cost/fallback-fire
rate/uptime % fields — it contains counts and acceptance status but not the runtime
latency/cost telemetry.

## Deliverables verified DONE

- RuntimeBackend enum (11 variants: Sidecar, AnthropicApi, ClaudeCode, CodexCli, OpenCode,
  Hermes, GeminiCli, KiroCli, CopilotCli, Ollama, Vllm) — evidence:
  .elai_cc/crates/providers/src/runtime/types.rs:55

- RuntimeAdapter trait (object-safe, Pin<Box<dyn Future>> async surface, complete/stream/
  capabilities/health_check/backend methods) — evidence:
  .elai_cc/crates/providers/src/runtime/types.rs:494

- RuntimeFactoryConfig struct — evidence:
  .elai_cc/crates/providers/src/runtime/types.rs:258

- RuntimePermissionMode enum (4 variants: AcceptEdits default, Plan, DangerouslySkipPermissions,
  Default; spec called it PermissionMode, renamed to avoid collision) — evidence:
  .elai_cc/crates/providers/src/runtime/types.rs:130

- ResolveBackendError struct (carries candidate + supported Vec<String>) — evidence:
  .elai_cc/crates/providers/src/runtime/mod.rs:78 (returns supported: RuntimeBackend::all())

- RuntimeCapabilities struct (all 9 fields from spec) — evidence:
  .elai_cc/crates/providers/src/runtime/types.rs:151

- RuntimeHealth enum (Ready / Degraded { reason } / Unavailable { reason }) — evidence:
  .elai_cc/crates/providers/src/runtime/types.rs:186

- resolve_backend(&str) -> Result<RuntimeBackend, ResolveBackendError> — evidence:
  .elai_cc/crates/providers/src/runtime/mod.rs:62; test at mod.rs:143 confirms 11-entry
  supported list on miss

- create_runtime(RuntimeFactoryConfig) -> Result<Box<dyn RuntimeAdapter>> for all 11 backends — evidence:
  .elai_cc/crates/providers/src/runtime/mod.rs:92-112

- SidecarRuntime adapter (health_check via GET /v1/messages canary) — evidence:
  .elai_cc/crates/providers/src/runtime/sidecar.rs:174

- AnthropicApiRuntime adapter — evidence:
  .elai_cc/crates/providers/src/runtime/anthropic_api.rs (DEF via from_factory_config mod.rs:101)

- 7 CLI subprocess adapters all present in runtime/ directory:
  ClaudeCodeRuntime — .elai_cc/crates/providers/src/runtime/claude_code.rs
  CodexCliRuntime   — .elai_cc/crates/providers/src/runtime/codex_cli.rs
  OpenCodeRuntime   — .elai_cc/crates/providers/src/runtime/opencode.rs
  HermesRuntime     — .elai_cc/crates/providers/src/runtime/hermes.rs
  GeminiCliRuntime  — .elai_cc/crates/providers/src/runtime/gemini_cli.rs
  KiroCliRuntime    — .elai_cc/crates/providers/src/runtime/kiro_cli.rs
  CopilotCliRuntime — .elai_cc/crates/providers/src/runtime/copilot_cli.rs

- All 7 CLI adapters wired to TierCSpawn (Tier C isolation) — evidence:
  .elai_cc/crates/providers/src/runtime/claude_code.rs:25 (and all 6 siblings share
  the same isolation import + spawn field pattern via cli_common.rs:311)

- stdout_idle_timeout_seconds enforced via cli_common async timeout utilities — evidence:
  .elai_cc/crates/providers/src/runtime/cli_common.rs:244-275

- OllamaRuntime native HTTP adapter — evidence:
  .elai_cc/crates/providers/src/runtime/ollama.rs:115 (fn backend returns RuntimeBackend::Ollama)

- VllmRuntime native HTTP adapter — evidence:
  .elai_cc/crates/providers/src/runtime/vllm.rs:121 (health_check defined)

- PerRoleRuntime struct with default per-role mapping (16 AgentRole variants covered,
  designer/auditor/researcher/media_validator -> AnthropicApi; rest -> Sidecar) — evidence:
  .elai_cc/crates/orchestrator/src/per_role_runtime.rs:44-86

- RuntimeOrchestrator struct (owns RuntimeConfig, lazy adapter cache, health snapshots,
  try_complete with fallback chain walk, health_snapshot) — evidence:
  .elai_cc/crates/orchestrator/src/runtime_orchestrator.rs:1-50

- RuntimeConfig with fallback_chain field (default: Sidecar -> AnthropicApi -> ClaudeCode) — evidence:
  .elai_cc/crates/orchestrator/src/runtime_config.rs:32-74

- Runtime fallback chain walk with SSB event emission — evidence:
  .elai_cc/crates/orchestrator/src/ssb.rs:947-952 (RuntimeFallback / RuntimeHealthChanged
  SsbActionType variants); runtime_event_sink_ssb.rs:199-200 (test with attempted chain)

- /runtime CLI command: list/show, test <backend>, set --role <r> --backend <b>, help
  subcommands all present — evidence:
  .elai_cc/crates/cli/src/commands.rs:1764-1801; .elai_cc/crates/cli/src/runtime_cmd.rs:1

- SlashCommand::Runtime variant wired in commands.rs — evidence:
  .elai_cc/crates/cli/src/commands.rs:1766,1771,1778,1789,1865

- [agents.<role>.runtime] backend = "" config persistence (read/write in runtime_cmd.rs) — evidence:
  .elai_cc/crates/cli/src/runtime_cmd.rs:313,362,571

- Source-grounding doc (ouroboros runtime interface, 10 backend files examined) — evidence:
  F:\.ELAI_workflow\workflow\papers\ouroboros-runtime-interface.md:1

- Migration guide (old [backend] mode -> new [runtime] backend) — evidence:
  F:\.ELAI_workflow\workflow\papers\phase-4M-migration.md:1

- metrics/phase_4M.toml exists with counts, files, SSB, acceptance, CLI, isolation sections — evidence:
  .elai_cc/metrics/phase_4M.toml:1

- Dashboard RuntimePanel.svelte component (per-role backend display, health color pills, Test
  button handler, 10s poll, role-picker) — evidence:
  .elai_cc/crates/dashboard-v2/src/lib/components/RuntimePanel.svelte:1-167

- Dashboard TypeScript types (RuntimeHealthStatus, RuntimeInfo, RuntimeBackendId, etc.) — evidence:
  .elai_cc/crates/dashboard-v2/src/lib/api/types.ts:1788

## Deliverables PARTIAL

- Dashboard RuntimePanel — Tauri command wiring deferred
  - exists: .elai_cc/crates/dashboard-v2/src/lib/components/RuntimePanel.svelte (full Svelte 5
    component with list/test/set-role logic)
  - gap: 3 Tauri backend commands (list_runtimes, test_runtime, set_role_runtime) are stub
    wrappers in stub.ts, not real Tauri invoke calls. Metrics confirm:
    dashboard.tauri_commands_added = 0, dashboard.runtime_panel_added = false.
    Evidence: .elai_cc/metrics/phase_4M.toml:128-129

- phase_4M.toml metrics file — content incomplete vs spec
  - exists: .elai_cc/metrics/phase_4M.toml
  - gap: Spec 4M.6 requires per-backend p50/p99 latency, tokens/sec, cost-per-1M-tokens or
    cost-per-hour, fallback-fire rate per session, health_check uptime % per backend. File
    contains counts, acceptance, and implementation inventory but zero runtime telemetry fields.
    These are runtime-collected metrics requiring live backend exercise, not static declarations.

- /runtime CLI command — missing /runtime fallback subcommand
  - exists: .elai_cc/crates/cli/src/commands.rs:1764 (list/show/test/set/help)
  - gap: Spec 4M.5 requires "/runtime fallback <role> <backend1>,<backend2>" to set
    per-role fallback chain at runtime. The subcommand is not present in
    parse_runtime_command() match arms (grep confirms no "fallback" case). The
    PerRoleRuntime::set_fallback() method exists (.elai_cc/crates/orchestrator/src/
    per_role_runtime.rs:110) but no CLI surface wires to it.

## Deliverables MISSING

None. All named deliverables have at least partial implementation.

## Deliverables UNVERIFIABLE

- 4M.2 ACCEPTANCE: "Old [backend] config aliases work with deprecation warning"
  The phase_4M.toml acceptance table does not record this test. The migration guide
  documents the mapping but no grep in runtime_cmd.rs or runtime_config.rs found a
  deprecation warning emission for legacy [backend] keys. Cannot confirm without reading
  the full load_elai_config() body.

- Test results for cargo test -p elai_providers and cargo test -p elai_orchestrator:
  metrics/phase_4M.toml marks cargo_test_providers and cargo_test_contracts as
  "pending_auditor". Implementer reports pass; auditor gate has not run yet per the
  metrics snapshot.

## What must be done (action list)

- [ ] Implement /runtime fallback <role> <b1,b2> subcommand in
  .elai_cc/crates/cli/src/commands.rs parse_runtime_command() +
  .elai_cc/crates/cli/src/runtime_cmd.rs (matching set_fallback() in per_role_runtime.rs:110)

- [ ] Wire the 3 Tauri commands (list_runtimes, test_runtime, set_role_runtime) in
  .elai_cc/crates/dashboard-v2/src-tauri/src/commands.rs from stub invocations to real
  RuntimeOrchestrator calls; mount RuntimePanel.svelte in SettingsWindow.svelte

- [ ] Populate runtime telemetry fields in .elai_cc/metrics/phase_4M.toml:
  per-backend p50/p99 latency, tokens/sec, cost-per-1M-tokens (cloud) or
  cost-per-hour (local), fallback-fire rate per session, health_check uptime % per backend

- [ ] Verify or document the legacy [backend] deprecation warning path in
  load_elai_config() (.elai_cc/crates/cli/src/runtime_config.rs)

- [ ] Run auditor gate: cargo test -p elai_providers, cargo test -p elai_orchestrator,
  cargo test -p elai_cc_contracts, cargo check --workspace

## Notes
The spec names the type PermissionMode; implementation uses RuntimePermissionMode to avoid
collision with orchestrator::permissions::PermissionMode. This is a deliberate naming
deviation documented in types.rs:15. The phase_4M.toml notes this under
[implementation].spec_types_present.

The /runtime fallback subcommand gap is clearly deferred: the metrics [cli].subcommands
list does not include "fallback" and the notes section (line 201) records
"RuntimeOrchestrator::health_snapshot batched probe not yet wired into /runtime list" as
a follow-up. The fallback CLI surface was apparently scoped out of B-37D alongside the
Wave-2 svelte slice.

Windows Tier C isolation uses reduced sandboxing (bare tokio::process + sanitized env)
vs Linux (AppContainer deferred). Noted in metrics [isolation].windows_isolation_reduced.

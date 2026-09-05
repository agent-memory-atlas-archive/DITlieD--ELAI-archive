# B-37: Phase 4M — Multi-Runtime Backend Abstraction (ouroboros-derived)
Objective: implement Phase 4M per spec — single `RuntimeAdapter` trait
covering 11 backends (2 existing + 7 CLI-spawning ouroboros ports + 2
native HTTP), per-role hybrid topology, runtime-fallback chain orthogonal
to model fallback, all CLI subprocesses inside Tier C isolation.
Estimated effort: 3-4 sessions.
Pre-requisites: B-1 (path unification), B-15..B-20 (production hardening),
B-22 (4G mode + autonomous gate — provides /mode + /model lifecycle that
/runtime mirrors), B-18 (Tier C sandbox active — required for CLI
subprocesses), Phase 3J + 3J.LOCAL + 3J.PIPELINE done in code.
Parallel-safe with: B-23 (4H Self-Mod), B-24 (4I Media), B-25 (4J Empire),
B-27 (Cloud), B-28 (Focus Tunnel).

## Context

Today ELAI binds the orchestrator to two control planes: the local
sidecar (Anthropic /v1/messages protocol) and the cloud Anthropic API.
Phase 3J added `[backend] mode = "local"|"api"|"hybrid"|"auto"` but the
dispatch shape is bespoke per backend (ANTHROPIC_BASE_URL switching).

Ouroboros (github.com/Q00/ouroboros, MIT) demonstrates a cleaner
pattern: ONE workflow specification drives multiple runtime backends
interchangeably — Claude Code, Codex CLI, OpenCode, Hermes, Gemini CLI,
Kiro CLI, GitHub Copilot CLI — via a single AgentRuntime trait +
per-backend adapter + `resolve_runtime_backend_name` dispatch. This is
their genuine architectural contribution and is the most directly
portable insight from their codebase.

Phase 4M ports that abstraction. The orchestrator writes Anthropic-shaped
messages once; adapters translate to whatever the underlying runtime
expects. Hybrid topology becomes per-role (Designer in cloud Codex,
Executor in local sidecar, Auditor in Claude Code) without forking
the orchestrator.

11 backends total:
- **Existing (port to RuntimeAdapter)**: Sidecar, AnthropicApi
- **Ouroboros ports (7)**: ClaudeCode, CodexCli, OpenCode, Hermes,
  GeminiCli, KiroCli, CopilotCli
- **ELAI-native (2)**: Ollama, Vllm

ALL CLI-spawning adapters run inside Tier C isolation (Phase 4A). The
CLI binary becomes the agent process boundary; subprocess output is
sanitized via existing IPC shape — no raw subprocess output reaches
the orchestrator's prompt context.

## V2 REBASELINE NOTE

Dashboard surface for per-role runtime config + live health belongs in
V2's SettingsWindow (`crates/dashboard-v2/src/lib/windows/SettingsWindow.svelte`)
+ a new RuntimePanel section. Tauri command `get_runtime_status` +
`set_role_runtime` + `test_runtime_health` added to
`crates/dashboard-v2/src-tauri/src/lib.rs` (B-36 pattern).

## Reads

- F:\.ELAI_workflow\docs\specs\guide\steps\phase-4M.txt (full spec)
- F:\.ELAI_workflow\docs\specs\guide\steps\phase-3J.txt (3J.LOCAL,
  3J.PIPELINE, 3J.WIRE.* sections — existing backend mode config)
- F:\.ELAI_workflow\docs\specs\crates\providers.txt S1, S4
- F:\.ELAI_workflow\docs\specs\crates\orchestrator.txt S12 (Tier C IPC)
- workflow/papers/ouroboros-runtime-interface.md (4M.0 ingest output)

## Tasks

1. **Spawn paper-ingestor (mode=source)** against ouroboros runtime files
   listed in phase-4M.txt 4M.0. Produces
   `workflow/papers/ouroboros-runtime-interface.md` with reliability
   tag MEDIUM (reviewed third-party MIT source).

2. **RuntimeBackend enum** (11 variants) + **RuntimeAdapter trait**
   in providers crate. Confirm vtable shape compatible with existing
   LlmProvider — alias the two if so, otherwise introduce thin wrapper.

3. **resolve_backend(candidate: &str) -> Result<RuntimeBackend,
   ResolveBackendError>** with full supported-list error on miss
   (ports ouroboros `resolve_runtime_backend_name`).

4. **create_runtime(cfg: RuntimeFactoryConfig) -> Box<dyn RuntimeAdapter>**
   factory. All new backends stub Err initially; existing Sidecar +
   AnthropicApi ported through factory.

5. **Migrate Phase 3J [backend] config** to new [runtime] schema with
   deprecation aliases (`mode = "local"` -> `backend = "sidecar"`,
   `mode = "api"` -> `backend = "anthropic_api"`). Old config emits
   deprecation warning for one release.

6. **CLI-spawning adapters (4M.3)** — 7 backends. Each:
   - subprocess via `std::process::Command` + tokio AsyncRead/Write
   - Anthropic-shaped messages translated to per-CLI stdin protocol
   - per-CLI stdout parsed back to Anthropic responses
   - `permission_mode` applied where supported
   - `startup_output_timeout_seconds` + `stdout_idle_timeout_seconds`
     enforced
   - `health_check()` returns Unavailable (NOT panic) when CLI missing
   - **MUST run inside Tier C isolation** (Phase 4A)
   - **stdout sanitized** before reaching orchestrator prompt context

7. **Native HTTP adapters (4M.4)** — Ollama (`/api/chat`) and vLLM
   (`/v1/chat/completions`). Port existing 3J providers to factory
   signature.

8. **PerRoleRuntime config (4M.5)**:
   `[agents.<role>.runtime] backend = "..."` per AgentRole.
   Default mapping (configurable):
   - designer → AnthropicApi (highest reasoning)
   - executor → Sidecar (local code gen)
   - auditor → AnthropicApi (independent verification)
   - memory_writer / context_compiler / prompt_engineer → Sidecar
   - researcher → AnthropicApi (search)
   - media_validator → AnthropicApi (vision)

9. **Runtime fallback chain (4M.5)** — orthogonal to 4G's
   SUBSTITUTE_MODEL. When a backend's `health_check()` returns
   Unavailable, traverse `fallback_chain`. Emit `runtime_fallback`
   SSB row with backend chain traversed. On exhausted chain: ESCALATE
   (INTERACTIVE) or HALT (AUTONOMOUS).

10. **`/runtime` CLI command**:
    - `/runtime show` — print per-role current backends
    - `/runtime set <role> <backend>` — runtime override
    - `/runtime fallback <role> <b1>,<b2>` — set chain

11. **Dashboard RuntimePanel** in V2 SettingsWindow:
    - per-role current backend
    - per-role fallback chain
    - live health green/yellow/red per known backend
    - "Test backend" button → canary health_check

12. **Metrics**: `.elai_cc/metrics/phase_4M.toml` with per-backend p50/p99
    latency, tokens/sec, cost-per-1M-tokens or cost-per-hour, fallback
    fire rate, health_check uptime %.

13. **Migration guide**: `workflow/papers/phase-4M-migration.md` with old
    `[backend] mode = ...` → new `[runtime] backend = ...` worked example.

## WIRING_CONTRACT

produces_symbols:
  - `RuntimeBackend` enum (11 variants)
  - `RuntimeAdapter` trait (or alias to existing LlmProvider)
  - `RuntimeFactoryConfig`, `PermissionMode`, `ResolveBackendError`,
    `PerRoleRuntime`, `RuntimeCapabilities`, `RuntimeHealth`
  - `resolve_backend()` + `create_runtime()` in providers crate
  - 9 new adapter modules (claude_code, codex_cli, opencode, hermes,
    gemini_cli, kiro, copilot_cli, ollama, vllm)
  - `/runtime show|set|fallback` CLI command
  - Dashboard RuntimePanel + Tauri commands `get_runtime_status`,
    `set_role_runtime`, `test_runtime_health`
  - SSB action_types: `runtime_fallback`, `runtime_health_changed`,
    `cli_subprocess_spawned`, `cli_subprocess_killed`
consumes_symbols:
  - existing LlmProvider implementations (sidecar, anthropic, ollama,
    vllm — ported)
  - Tier C process isolation (Phase 4A) for CLI subprocesses
  - 4G `/mode` + `/model` lifecycle (mirror pattern for `/runtime`)
reachable_from:
  - elai_cc.toml `[runtime]` + `[agents.<role>.runtime]` sections
  - dashboard SettingsWindow RuntimePanel
  - CLI `/runtime` command
stub_ban:
  - CLI subprocesses MUST run in Tier C (no direct subprocess.spawn
    bypassing isolation)
  - PermissionMode default MUST be AcceptEdits, NEVER
    DangerouslySkipPermissions without explicit user opt-in
  - stdout sanitization MUST run before subprocess output reaches prompt
    context
frontend_pair: SettingsWindow RuntimePanel + Tauri commands
observable_effect:
  trigger: `/runtime set executor anthropic_api` followed by submit task
  expect_ssb: runtime selected per role; if primary unavailable,
              `runtime_fallback` row emitted with chain
  expect_artifact: dashboard RuntimePanel reflects live override;
                   `.elai_cc/metrics/phase_4M.toml` populated after run

## DETERMINISTIC_CHECK

```bash
# 1. RuntimeBackend has 11 variants
rg "RuntimeBackend::(Sidecar|AnthropicApi|ClaudeCode|CodexCli|OpenCode|Hermes|GeminiCli|KiroCli|CopilotCli|Ollama|Vllm)" .elai_cc/crates/providers/src

# 2. RuntimeAdapter trait
rg "trait RuntimeAdapter" .elai_cc/crates/providers/src

# 3. resolve_backend + create_runtime
rg "fn resolve_backend|fn create_runtime" .elai_cc/crates/providers/src

# 4. Per-adapter tests
cargo test -p elai_providers -- runtime 2>&1 | grep "test result"
# Expected: ≥ 11 adapter test groups + integration tests

# 5. /runtime CLI command
rg "SlashCommand::Runtime" .elai_cc/crates/cli/src/commands.rs

# 6. Tier C enforcement on CLI subprocesses
rg "tier_c|process_isolation" .elai_cc/crates/providers/src/runtime/

# 7. Migration guide
test -f workflow/papers/phase-4M-migration.md
```

## ACCEPTANCE

- [ ] Paper-ingestor produces ouroboros-runtime-interface.md (4M.0)
- [ ] RuntimeBackend enum: 11 variants, all referenced from create_runtime
- [ ] RuntimeAdapter trait: shape compatible with LlmProvider, all 11
      backends implement it
- [ ] Sidecar + AnthropicApi migrated; zero regression on Phase 3J tests
- [ ] All 7 CLI adapters operational, health_check returns Unavailable
      (not panic) when CLI missing
- [ ] All 7 CLI adapters run inside Tier C isolation (verified by
      Tier C contract test)
- [ ] stdout sanitization passes through 4A IPC sanitizer
- [ ] permission_mode applied (default AcceptEdits, never
      DangerouslySkipPermissions without explicit opt-in)
- [ ] stdout_idle_timeout enforced (subprocess killed on stuck output)
- [ ] Ollama + Vllm adapters migrated from Phase 3J
- [ ] PerRoleRuntime per-AgentRole config functional
- [ ] runtime fallback chain traversed on Unavailable; emits SSB row
- [ ] runtime fallback orthogonal to 4G SUBSTITUTE_MODEL (no race)
- [ ] `/runtime show|set|fallback` CLI command works
- [ ] Dashboard RuntimePanel renders per-role + live health pings
- [ ] `.elai_cc/metrics/phase_4M.toml` populated
- [ ] Migration guide explicit (old config → new config worked example)
- [ ] All adapter test groups pass (≥ 11 modules + integration)

## Files touched

- ADD: crates/providers/src/runtime/mod.rs (trait + factory + resolve)
- ADD: crates/providers/src/runtime/sidecar.rs (port existing)
- ADD: crates/providers/src/runtime/anthropic_api.rs (port existing)
- ADD: crates/providers/src/runtime/claude_code.rs
- ADD: crates/providers/src/runtime/codex_cli.rs
- ADD: crates/providers/src/runtime/opencode.rs
- ADD: crates/providers/src/runtime/hermes.rs
- ADD: crates/providers/src/runtime/gemini_cli.rs
- ADD: crates/providers/src/runtime/kiro.rs
- ADD: crates/providers/src/runtime/copilot_cli.rs
- ADD: crates/providers/src/runtime/ollama.rs (port from 3J)
- ADD: crates/providers/src/runtime/vllm.rs (port from 3J)
- ADD: crates/orchestrator/src/per_role_runtime.rs
- MODIFY: crates/cli/src/commands.rs (`/runtime` slash command)
- MODIFY: elai_cc.toml schema ([runtime] + [agents.<role>.runtime])
- MODIFY: crates/dashboard-v2/src-tauri/src/lib.rs (3 new commands)
- ADD: crates/dashboard-v2/src/lib/components/RuntimePanel.svelte
- MODIFY: crates/dashboard-v2/src/lib/windows/SettingsWindow.svelte
- ADD: workflow/papers/ouroboros-runtime-interface.md (paper-ingestor output)
- ADD: workflow/papers/phase-4M-migration.md (migration guide)
- ADD: .elai_cc/metrics/phase_4M.toml
- CREATE: workflow/reasoning/B-37-decisions.md

## Next

B-38 (Ambiguity-score gate retrofit — small standalone) or fall through
to next outstanding B-N depending on prioritization.

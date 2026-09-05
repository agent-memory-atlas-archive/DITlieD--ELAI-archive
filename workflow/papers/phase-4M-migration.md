Phase 4M Migration Guide: Multi-Runtime Backend Abstraction
============================================================

Source: B-37 (Phase 4M) — multi-runtime adapter layer
Reliability: HIGH (authored from production source after B-37A/B/C/D landed)
Author: session s-27076
Date: 2026-05-16
Consumer: elai operators upgrading from single-runtime (Sidecar-or-AnthropicApi) deployments to per-role multi-runtime routing.

Overview
--------

B-37 (Phase 4M) introduces a uniform `RuntimeAdapter` trait covering eleven backends and replaces the previous direct `LlmProvider` dispatch with a per-role routing layer plus a fallback chain. The eleven backends are:

  Native HTTP (real, in-process)
    Sidecar         — local or remote ELAI inference server (HTTP, Anthropic-compatible)
    AnthropicApi    — direct api.anthropic.com client (OAuth or API key)
    Ollama          — OpenAI-compat HTTP at /v1/chat/completions with /api/chat fallback
    Vllm            — OpenAI-compat HTTP (vLLM reference impl)

  CLI subprocess (Tier C isolated)
    ClaudeCode      — `claude` binary (Anthropic's official CLI)
    CodexCli        — `codex` binary (OpenAI Codex)
    OpenCode        — `opencode --pure` binary
    Hermes          — `hermes` binary (text-only, supports_tools=false)
    GeminiCli       — `gemini` binary (NDJSON stream-json output)
    KiroCli         — `kiro-cli` binary (note the hyphen)
    CopilotCli      — `copilot` binary (GitHub Copilot CLI)

The new public surface is:

  elai_providers::runtime::RuntimeBackend         (11-variant enum)
  elai_providers::runtime::RuntimeAdapter         (trait — complete + stream + health_check)
  elai_providers::runtime::resolve_backend()      (canonical name + alias resolution)
  elai_providers::runtime::create_runtime()       (factory returning Box<dyn RuntimeAdapter>)
  elai_orchestrator::PerRoleRuntime               (role -> backend BTreeMap)
  elai_orchestrator::RuntimeOrchestrator          (routing + fallback chain + health snapshot)

Four new SSB action types now record runtime events:

  runtime_fallback         — primary backend failed, attempted chain
  runtime_health_changed   — health state transition for a backend
  cli_subprocess_spawned   — CLI adapter spawned a subprocess
  cli_subprocess_killed    — CLI adapter killed/reaped a subprocess

The SSB schema version bumps from 45 to 46. The B-34 IPC v2 envelope pin at SCHEMA_VERSION=42 is **not** regressed — the new action types are additive at the SSB layer.

Breaking changes
----------------

None at the wire level. The IPC v2 envelope and dashboard SSB consumers remain shape-compatible because:

  - All four new action types are additive (no existing enum variant renamed or removed).
  - The new `RuntimeInfo` / `TestReport` Tauri payloads are command DTOs, not SSB rows.
  - `LlmProvider` still exists as the lower-tier in-process trait used by SidecarRuntime and AnthropicApiRuntime; the new `RuntimeAdapter` wraps but does not replace it.

Soft-deprecation (one-release window):

  - `[cloud] default_mode` accepts the legacy values `"local"` and `"cloud"` and emits a `tracing::warn!` mapping them to `[runtime] backend = "sidecar"` and `[runtime] backend = "anthropic_api"` respectively.
  - The deprecation alias map lives in `crates/cli/src/runtime_config.rs` and will be removed in the release after this one.

Config migration
----------------

Before (Phase 4G / B-22):

    [cloud]
    default_provider = "anthropic"
    default_model    = "claude-sonnet-4-6"
    default_mode     = "auto"

After (Phase 4M):

    # Default backend used when a role has no per-role override.
    [runtime]
    backend  = "sidecar"
    fallback = ["sidecar", "anthropic_api", "claude_code"]

    # Per-role override. Any of the 11 canonical backend names is valid.
    # Roles not listed here inherit [runtime] backend.
    [agents.designer.runtime]
    backend = "anthropic_api"

    [agents.executor.runtime]
    backend = "sidecar"

    [agents.auditor.runtime]
    backend = "anthropic_api"

    [agents.researcher.runtime]
    backend = "anthropic_api"

    [agents.media_validator.runtime]
    backend = "anthropic_api"

    # [cloud] block is honoured during the deprecation window but emits a warning.
    [cloud]
    default_provider = "anthropic"
    default_model    = "claude-sonnet-4-6"
    default_mode     = "auto"

Canonical backend strings accepted by `[runtime] backend` and `[agents.<role>.runtime] backend`:

    sidecar | anthropic_api | claude_code | codex_cli | opencode | hermes
    gemini_cli | kiro_cli | copilot_cli | ollama | vllm

Aliases accepted (case-insensitive, whitespace-trimmed):

    local        -> sidecar
    anthropic    -> anthropic_api
    api          -> anthropic_api
    claude_api   -> anthropic_api
    claude       -> claude_code
    codex        -> codex_cli
    opencode_cli -> opencode
    hermes_cli   -> hermes
    gemini       -> gemini_cli
    kiro         -> kiro_cli
    copilot      -> copilot_cli

Environment variables
---------------------

The adapters honour the following env vars when the corresponding config field is unset.

  Sidecar
    ANTHROPIC_BASE_URL   — sidecar HTTP endpoint (default http://localhost:8100)
    ANTHROPIC_API_KEY    — bearer for the sidecar (default "local-dummy")

  AnthropicApi
    ANTHROPIC_API_KEY    — production API key (overrides OAuth if set)
    ANTHROPIC_AUTH_TOKEN — alternate name accepted by the official client

  Ollama
    OLLAMA_HOST          — Ollama HTTP endpoint (default http://localhost:11434)

  Vllm
    (no env override — base_url must be set in config)

  CLI adapters (all 7)
    PATH                 — used to locate the named binary; missing binary -> RuntimeHealth::Unavailable
    HOME / APPDATA       — preserved into the subprocess for auth cache discovery
    ELAI_CLI_TIMEOUT_S   — override default 300s idle timeout (clamped 5..=3600)

CLI surface
-----------

A new top-level slash command `/runtime` is registered. It is also available as a direct subcommand (`elai runtime ...`) from the shell.

  /runtime                                            — render the runtime status table
  /runtime list                                       — same as above
  /runtime show                                       — same as above
  /runtime test <backend>                             — single-adapter health_check with 5s timeout
  /runtime set --role <role> --backend <backend>      — persist mapping to elai_cc.toml
  /runtime help                                       — usage block

The `--role` / `--backend` order is insensitive; duplicates and missing values produce a usage error.

No previous `/provider` command existed in trunk; none was deprecated. The `/mode` command (autonomous/interactive) is unaffected.

Dashboard surface
-----------------

A new `RuntimePanel.svelte` component is mounted into `SettingsWindow.svelte` between the existing "Execution" and "Model routing" sections. It exposes:

  - one row per agent role with a `<Select>` for the active backend
  - per-row "Test" button that drives `test_runtime(backend)`
  - per-row chip strip showing the fallback chain (read-only in B-37D)
  - a top-of-panel global health strip (11 chips, green = Ready, yellow = Degraded, red = Unavailable, gray pill = stub)

Three new Tauri commands back the panel:

  list_runtimes() -> Vec<RuntimeInfo>
  test_runtime(backend: String) -> TestReport
  set_role_runtime(role: String, backend: String) -> RuntimeRouting

The `RuntimeInfo.is_stub` flag carries a "stub adapter" badge (mirroring the GatesPanel mock badge) so users see the difference between "binary not on PATH" and "adapter intentionally unimplemented".

Per-backend description and install requirements
-----------------------------------------------

Sidecar
  ELAI's own inference server. Runs locally or on a remote GPU box; the dashboard treats the remote pythondev box and a local sidecar identically. Install: `python server.py --model <hf-id> --port 8100`. Health probe hits `GET /v1/messages` with a 4-token canary.
  Minimal config:
    [runtime] backend = "sidecar"
    [sidecar] server_url = "http://localhost:8100"

AnthropicApi
  Direct api.anthropic.com client. Honours either an API key (`ANTHROPIC_API_KEY`) or an OAuth refresh token loaded via `crates/providers/src/oauth.rs`. No binary required.
  Minimal config:
    [runtime] backend = "anthropic_api"
    [cloud] default_provider = "anthropic"

Ollama
  OpenAI-compatible HTTP backend at `/v1/chat/completions` with `/api/chat` fallback for Ollama < 0.1.34. Install: download from ollama.com; run `ollama serve`.
  Minimal config:
    [runtime] backend = "ollama"
    [ollama] base_url = "http://localhost:11434"
    [ollama] model    = "llama3.1"

Vllm
  vLLM reference OpenAI-compat server. Install: `pip install vllm`; run `python -m vllm.entrypoints.openai.api_server --model <hf-id>`. Health probe hits `GET /health` with `GET /v1/models` as fallback.
  Minimal config:
    [runtime] backend = "vllm"
    [vllm] base_url = "http://localhost:8000"
    [vllm] model    = "Qwen/Qwen2.5-Coder-7B-Instruct"

ClaudeCode
  Anthropic's official `claude` CLI. Install: `npm install -g @anthropic-ai/claude-code`. Invocation: `claude -p <prompt> --output-format stream-json --verbose --include-partial-messages --permission-mode {acceptEdits|bypassPermissions|default}`.

CodexCli
  OpenAI Codex CLI. Install: `npm install -g @openai/codex`. Output is written to a temp file (`--output-last-message <path>`) rather than stdout; the adapter reads the file after `child.wait().await`.

OpenCode
  `opencode --pure --format json` is mandatory (the `--pure` flag suppresses interactive prompts). Install: see opencode.dev.

Hermes
  Text-only adapter — `supports_tools=false`. Invocation: `hermes chat --resume <id> -Q --source tool --model <m> -q <prompt>`. Prompt size is capped at 30 KB on Windows / 100 KB on POSIX to avoid argv overflow.

GeminiCli
  Google's gemini CLI. Output is NDJSON via `--output-format stream-json`. Install: see ai.google.dev.

KiroCli
  Binary is `kiro-cli` (note the hyphen, not `kiro`). Invocation: `kiro-cli chat --no-interactive <trust_flag> <prompt>`. Stderr drains concurrently with stdout (60s startup / 300s idle).

CopilotCli
  GitHub Copilot CLI. Permission flags are `--allow-tool 'Kind(arg)'` / `--deny-tool` / `--allow-all`; deny precedes allow. Install: `npm install -g @github/copilot-cli`.

Tier C isolation
----------------

All seven CLI adapters spawn through `crates/providers/src/runtime/isolation/default_spawn.rs`. On Linux the spawn is wrapped in `elai_core::sandbox::build_linux_sandbox_command` (bubblewrap-equivalent). On Windows the current implementation falls through to a bare `tokio::process::Command` with sanitised env — full AppContainer wiring is deferred and documented as reduced-isolation in the source comments.

Permission mode
---------------

`RuntimePermissionMode` has four variants. The default is `AcceptEdits`. `DangerouslySkipPermissions` requires `RuntimeFactoryConfig::allow_danger = true`; constructing a CLI adapter with `DangerouslySkipPermissions` and `allow_danger = false` returns `RuntimeAdapterError::PermissionDenied` from the constructor before any subprocess is spawned.

Troubleshooting
---------------

  Symptom: `/runtime test claude_code` -> "Unavailable: binary 'claude' not on PATH"
    Diagnosis: the named CLI is not installed or PATH is sanitised.
    Fix: install per the per-backend section above; verify with `which claude` (POSIX) or `where claude` (Windows). If installed, ensure the elai process inherits PATH (relevant when running under launchctl / systemd).

  Symptom: `/runtime test anthropic_api` -> "Unavailable: OAuth token expired"
    Diagnosis: cached OAuth refresh token at `~/.config/elai/oauth.json` is older than 90 days OR the user revoked access in console.anthropic.com.
    Fix: either set `ANTHROPIC_API_KEY` in env (bypasses OAuth) or re-run `elai auth login`.

  Symptom: `/runtime test ollama` -> "Unavailable: connection refused"
    Diagnosis: Ollama server not running, or `OLLAMA_HOST` mismatch.
    Fix: `ollama serve` in another terminal; confirm with `curl http://localhost:11434/api/tags`.

  Symptom: `/runtime test sidecar` -> "Degraded: health probe timeout"
    Diagnosis: sidecar process is up but a previous inference is holding the worker, OR remote SSH tunnel dropped.
    Fix: `ssh -i ~/.ssh/elai_remote -p 6465 maintainer@192.0.2.10 'pgrep -f "python server.py.*--port 8100"'` and restart the tunnel (`workflow/remote-setup/start-tunnel.ps1`).

  Symptom: `/runtime list` takes ~55s to render
    Diagnosis: known cost — eleven sequential 5s timeouts during health probing. The `RuntimeOrchestrator::health_snapshot` batched probe (Wave-1B) is not yet wired into the CLI render path.
    Fix: planned in a follow-up patch; the batched call is already implemented in the orchestrator.

  Symptom: SSB row `runtime_fallback` fires repeatedly for the same role
    Diagnosis: primary backend is consistently failing; the orchestrator is walking the fallback chain on every request.
    Fix: investigate `runtime_health_changed` rows for the failing backend; consider editing `[agents.<role>.runtime] backend` to the working tier directly.

  Symptom: dashboard RuntimePanel shows all backends gray ("stub adapter")
    Diagnosis: B-37A/B/C not on this build, or the dashboard is loading from a cached stub.ts.
    Fix: confirm `cargo check -p elai-providers` passes; restart `npm run tauri:dev`.

Rollback
--------

To revert to pre-Phase-4M behaviour:

  1. Set `SCHEMA_VERSION` in `crates/orchestrator/src/ssb.rs` back to 45.
  2. Remove the four `SsbActionType` variants (`runtime_fallback` / `runtime_health_changed` / `cli_subprocess_spawned` / `cli_subprocess_killed`) and their payload structs.
  3. Remove `RuntimePanel` from `SettingsWindow.svelte` and unregister `list_runtimes` / `test_runtime` / `set_role_runtime` from `invoke_handler!`.
  4. Restore the previous `[cloud] default_provider` / `default_mode` flow in `crates/cli/src/runtime_config.rs`.
  5. Delete or rename `[runtime]` and `[agents.<role>.runtime]` keys in `elai_cc.toml`.

Note: rolling back will also invalidate any SSB database rows with `action_type` in the four new variants. Either truncate the SSB row table or migrate those rows out before downgrading the schema version, otherwise reads will fail with "unknown action_type".

Source references
-----------------

Grounding artefacts:

  workflow/papers/ouroboros-runtime-interface.md      — MEDIUM reliability, 420 LOC, source-grounded
  workflow/reasoning/B-37-decisions.md                — 8 locked architectural decisions
  workflow/sessions/s-27076/b37-wave1a-providers-report.md   — providers crate landing
  workflow/sessions/s-27076/b37-wave1b-orchestrator-report.md — orchestrator + SSB landing
  workflow/sessions/s-27076/b37-wave1c-cli-report.md         — /runtime command landing
  docs/specs/guide/steps/phase-4M.txt                 — phase spec (source of truth for /runtime grammar and SSB action types)
  docs/thefinalglance/B-plan/B-37_multi_runtime_backend.md   — B-plan WIRING_CONTRACT / DETERMINISTIC_CHECK / ACCEPTANCE

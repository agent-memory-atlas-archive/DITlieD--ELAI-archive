# ELAI CLI reference

This file documents the ELAI CLI surface, with particular focus on the
C-39 MCP bridge dispatch tier (`--via-mcp`) and the related
`--export-ssb` observability flag.

## `elai run`

Production task_runner entry point. Spawns the SSB writer, feasibility
prober, Layer 1A processor, and (optionally) the production agent-pool
baby-step executor. Subscribes to `TaskSubmitted` SSB rows and drives
the FSM to completion.

### Synopsis

```
elai run [--project-root <path>]
         [--mode autonomous|interactive|hybrid]
         [--provider local|cloud|hybrid]
         [--via-mcp]
         [--export-ssb <path>]
         [--empire]
```

### Flags

- `--project-root <path>`
  Project directory the runner binds to. Defaults to the current working
  directory. The SSB database is opened at
  `<project_root>/.elai_cc/sessions/ssb.db` (same flat path the
  dashboard reads from -- see runtime_setup.rs).

- `--mode autonomous|interactive|hybrid`
  C-9 gate mode. Defaults to `interactive` (every MEDIUM/HIGH FSM
  transition prompts a human). `autonomous` lets the evidence checks
  drive auto-advance; `hybrid` is the middle ground.

- `--provider local|cloud|hybrid`
  C-5 backend routing axis. `local` pins to the sidecar; `cloud` routes
  through the configured Anthropic-compatible API; `hybrid` is
  local-first with cloud-escalate. Defaults to the toml
  `[cloud].default_mode` field.

- `--via-mcp`
  C-39 tool-dispatch tier. When set, the runner installs an
  `McpClientDispatch` on `TaskRunnerConfig.mcp_dispatch` so tool calls
  routed through the dispatch trait emit `cli_mcp_dispatch` SSB rows
  via `DefaultMcpBridge::http_call_tool` (HTTP JSON-RPC 2.0 to the
  in-process MCP bridge on `http://localhost:8200`). The bridge runs
  in-process per C-39 DECISION D1 (see workflow/reasoning/
  C-39-decisions.md); a true co-process tier (`elai mcp-server`
  subcommand) is CO-PROCESS-PENDING-FOLLOWUP.
  When absent, the runner installs an `InProcessDispatch` that routes
  through `elai_tools::execute_tool` and emits `mcp_tool_called`
  rows. Both legs are observable from the dashboard MCP panel.

- `--export-ssb <path>`
  C-39 observability sink. When set, the runner subscribes to the SSB
  broadcast at startup and writes a JSON array of row objects to
  `<path>` at process exit. Each row carries
  `{action_type, payload, seq, created_at}`. Consumed by
  `scripts/ci-mcp-parity.sh` to diff in-process vs `--via-mcp` tool
  call sequences.

- `--empire`
  B-25 Phase 4J. Requires the orchestrator crate to be compiled with
  `--features empire`; without the feature this flag is a no-op and
  logs a warn.

### `--via-mcp` lifecycle SSB rows

When `--via-mcp` is set, the runner emits a pair of lifecycle rows:

- `cli_mcp_server_started` -- emitted at runner startup with
  `port` (default 8200). Marks the dispatch tier activation; even though
  the bridge runs in-process, the row anchors the lifecycle for the
  dashboard and CI parity surfaces.

- `cli_mcp_server_stopped` -- emitted at runner shutdown (clean
  termination or SIGINT-triggered graceful shutdown). Symmetric with
  the started row.

Per-tool-call rows (`cli_mcp_dispatch`) are emitted by
`McpClientDispatch::dispatch_tool` and carry `tool_name + port`.

### Parity testing

`scripts/ci-mcp-parity.sh` runs `elai run --export-ssb` twice (once
with the default in-process dispatch, once with `--via-mcp`), then
invokes `ci_mcp_parity_check.py` to diff the tool-name sequences. The
helper unions `mcp_tool_called` and `cli_mcp_dispatch` rows so the
sequences are comparable across legs.

```
bash scripts/ci-mcp-parity.sh
```

The GitHub workflow `.github/workflows/ci-mcp-parity.yml` runs this
script on every push/PR and is BLOCKING per the C-39
WIRING_CONTRACT.stub_ban.

### Live smoke (RUNPOD-PENDING)

End-to-end tool-call smokes that exercise an actual sidecar or remote
GPU are RUNPOD-PENDING. The CI parity job above exercises the dispatch
surface against a cold runner (no task submitted); a live submission
would require either a sidecar binding to localhost:8100 or the remote
sidecar tunnel documented in workflow/remote-setup/REMOTE-SIDECAR.md.

## Slash commands

Slash commands are parsed by `crates/cli/src/commands.rs`
(`SlashCommand` enum). The following carry C-39 `--via-mcp` toggles:

- `/task [--surgical] [--new] [--via-mcp] <description>`
  Constructs a `SlashCommand::Task { via_mcp, .. }`. The flag is
  acknowledged at the REPL routing site
  (`crates/cli/src/live_cli.rs`); per-invocation dispatch wiring is
  CLOSURE-PENDING-FOLLOWUP -- the production seam lives on `elai run
  --via-mcp` today.

- `/explore [--via-mcp] <target>`
  Constructs a `SlashCommand::Explore { via_mcp, .. }`. Same scope
  note as `/task`.

## See also

- `crates/cli/src/run_subcommand.rs` -- production `elai run` entry.
- `crates/orchestrator/src/dispatch/` -- `Dispatch` trait + two impls.
- `crates/orchestrator/src/runtime_setup.rs` --
  `apply_task_runner_wiring` chokepoint.
- `crates/plugins/src/mcp_bridge.rs` -- HTTP JSON-RPC 2.0 client.
- `workflow/reasoning/C-39-decisions.md` -- closure-session rationale.
- `workflow/papers/openhands-mcp-integration.md` -- source-grounding
  for the bridge architecture (HTTP semantics, not CLI structure).

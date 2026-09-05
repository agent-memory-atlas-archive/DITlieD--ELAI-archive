# C-39: MCP Bridge CLI Surface Extension (Parity With OpenHands)

Objective: extend the Phase 4K MCP bridge from dashboard-only consumer to
CLI-first surface so /task and /explore can directly invoke registered MCP
tools without the dashboard in the loop. Achieves parity with OpenHands'
production MCP integration (~38K stars, Apache 2.0,
github.com/All-Hands-AI/OpenHands).

Pre-requisites: Phase 4K MCP bridge SHIPPED; C-35 (Tool Attention dynamic
gating + lazy schema) SHIPPED — the CLI must consume the gated catalog, not
the legacy full catalog.
Parallel-safe with: C-36, C-37, C-38, C-40 (orthogonal). Sequence AFTER C-35
which establishes the gated-catalog API the CLI will consume.

Reference grounding:
- DETAILED_SUMMARY_01 §X (plugins/) notes the existing bridge: "Consumed by
  dashboard only. CLI does not use MCP bridge."
- OpenHands MCP integration uses the fastmcp HTTP client pattern as their
  agent-side tool surface; ELAI mirrors the HTTP JSON-RPC 2.0 semantics, not
  the OpenHands CLI binary structure (OpenHands has no standalone CLI per
  workflow/papers/openhands-mcp-integration.md). The ELAI MCP server
  JSON-RPC 2.0 shape is compatible enough for direct CLI consumption.
- C-39 closure references mcp_bridge.rs (real implementation) rather than
  the originally-cited phase-4K.txt path: phase-4K.txt documents the
  dashboard MVP and contains zero MCP content. The bridge surface ships
  in plugins/src/mcp_bridge.rs + plugins/src/mcp/{mod,gate,lazy}.rs.

## Context

ELAI's MCP bridge exists but only the Tauri dashboard talks to it. The CLI
path (`elai task ...`, `elai explore ...`) invokes tools via the in-process
plugins/ trait dispatch, NOT through the MCP server. This means:
1. MCP-exposed tools have no observable surface from CLI testing (a tool that
   works through the bridge but breaks in-process is undetected in CLI CI).
2. External MCP clients (other agentic frameworks, IDE plugins, custom
   scripts) can register tools the dashboard sees but the CLI does not.
3. CI parity is broken: dashboard E2E and CLI E2E exercise different code
   paths for the same tool surface.

C-39 extends the CLI to OPTIONALLY route tool calls through the MCP bridge
(default off for backwards compatibility, opt-in via --via-mcp flag), and
makes the dashboard's MCP path the source-of-truth contract for tool
behavior. After C-39, CI runs both modes against the same tool catalog and
asserts they produce equivalent results.

## Reads

- F:\.ELAI_workflow\docs\thefinalglance\C-plan\C-35_tool_attention_dynamic_gating.md
- .elai_cc/crates/plugins/src/mcp_bridge.rs (bridge source-of-truth;
  re-exported under `crate::mcp::server` per C-35 M3 alias)
- .elai_cc/crates/cli/src/main.rs
- .elai_cc/crates/cli/src/commands.rs (single flat SlashCommand module --
  no `commands/task.rs` or `commands/explore.rs` subfiles exist; the
  CliAction::Run + run_subcommand path is the canonical surface)
- github.com/All-Hands-AI/OpenHands (architecture reference for fastmcp
  HTTP semantics; OpenHands has no standalone CLI -- the "CLI parity"
  framing applies to the HTTP transport, not the CLI binary structure)

## Tasks

1. **Source-grounding** — spawn paper-ingestor (mode=source) against OpenHands
   MCP integration code (subset: their MCP client + tool-dispatch path).
   Produces workflow/papers/openhands-mcp-integration.md.

2. **CLI MCP client (NEW, crates/cli/src/mcp_client.rs)** — minimal HTTP
   JSON-RPC 2.0 client targeting the bridge's localhost endpoint. Methods:
   list_tools, invoke_tool(name, args), get_tool_schema(name) (lazy
   resolution per C-35).

3. **--via-mcp flag on /task and /explore** — when present, tool dispatch
   in the FSM routes through the MCP client instead of in-process plugins.
   When absent, current in-process path remains (no behavior change).

4. **Local MCP server lifecycle** — if --via-mcp is set and no external
   bridge is listening, the CLI auto-launches a co-process MCP server
   (same binary as the dashboard's bridge backend) bound to a random
   localhost port. Tears down at exit. Surface the chosen port via SSB.

5. **CI parity job** — add a new CI matrix dimension: every existing CLI
   integration test runs TWICE, once in default mode and once with
   --via-mcp. Assert outputs match (modulo SSB action-type names that
   differ between in-process and via-bridge dispatch).

6. **SSB action types** — add: cli_mcp_dispatch (per tool call), cli_mcp_server_started,
   cli_mcp_server_stopped. Each carries tool_name and port.

7. **Tests first (TDD)** (renamed C39.1..C39.5 per grounding to avoid
   collision with the phase-4K namespace):
   - C39.1: CliMcpClient default + arbitrary port construction
   - C39.2: both Dispatch impls satisfy the trait contract
   - C39.3: 3 CliMcp* SSB action types round-trip via as_sql_str/from_str
   - C39.4: 3 CliMcp* SsbPayload variants construct + report action_type
   - C39.5: McpClientDispatch on unreachable port fails closed (stub_ban)

8. **Production wiring** — wire MCPClient into the FSM's tool-dispatch
   trait. The dispatch trait gets a second implementation
   `McpClientDispatch` alongside the existing `InProcessDispatch`. CLI
   chooses at startup based on flag.

9. **Contract tests** — tests/contracts/cli_mcp_contract.rs verifying both
   dispatch implementations satisfy the same trait shape and produce
   equivalent SSB rows for equivalent inputs.

10. **Documentation** — add CLI MCP usage section to .elai_cc/docs/cli.md
    and `--via-mcp` flag help text.

## WIRING_CONTRACT

produces_symbols:
  - MCPClient struct (crates/cli/src/mcp_client.rs)
  - McpClientDispatch (crates/orchestrator/src/dispatch/mcp.rs)
  - InProcessDispatch (refactor of existing implicit dispatch)
  - --via-mcp CLI flag
  - SSB action_types: cli_mcp_dispatch, cli_mcp_server_started, cli_mcp_server_stopped
  - CI parity job in .github/workflows/ci.yml or equivalent
consumes_symbols:
  - MCP server endpoint (plugins/src/mcp_bridge.rs -- re-exported as
    `plugins::mcp::server` per C-35 M3 alias)
  - ToolGate from C-35 (catalog filtering)
  - SSB event bus
reachable_from:
  - elai task --via-mcp
  - elai explore --via-mcp
  - elai task (default in-process path, unchanged)
stub_ban:
  - MCPClient MUST NOT silently retry on network errors; surface to SSB
    and fail closed
  - --via-mcp without listener and unable to launch co-process MUST emit a
    structured error to SSB, NOT a panic
  - CI parity job MUST be a CI-blocking step (not informational)
frontend_pair: (existing dashboard MCP panel) — verify still works after
              dispatch refactor
observable_effect:
  trigger: elai task --via-mcp "implement X"
  expect_ssb: cli_mcp_server_started (if co-process) + cli_mcp_dispatch rows
              per tool invocation matching in-process equivalents
  expect_artifact: CI parity job green; output diff empty (modulo expected
                   SSB action-type rename)

## DETERMINISTIC_CHECK

```bash
# 1. MCPClient module exists
test -f .elai_cc/crates/cli/src/mcp_client.rs

# 2. Both dispatch impls present
test -f .elai_cc/crates/orchestrator/src/dispatch/mcp.rs
rg "InProcessDispatch|McpClientDispatch" .elai_cc/crates/orchestrator/src/dispatch

# 3. --via-mcp flag wired
rg "via-mcp" .elai_cc/crates/cli/src/main.rs

# 4. SSB action types
rg "cli_mcp_dispatch|cli_mcp_server_started|cli_mcp_server_stopped" .elai_cc/crates/orchestrator/src

# 5. CI parity job exists
grep -r "via-mcp" .github/workflows/

# 6. Contract test passes
cargo test -p elai_cli -- mcp_contract 2>&1 | grep "test result.*ok"

# 7. CI parity passes locally
.elai_cc/scripts/ci-mcp-parity.sh
```

## ACCEPTANCE

- [ ] paper-ingestor produces workflow/papers/openhands-mcp-integration.md
- [ ] MCPClient implemented, supports list_tools / invoke_tool / get_tool_schema
- [ ] InProcessDispatch + McpClientDispatch share Dispatch trait
- [ ] --via-mcp flag added to /task and /explore subcommands
- [ ] Auto-launch co-process when --via-mcp without listener
- [ ] Reuse external listener when --via-mcp with existing port
- [ ] All 3 SSB action types emitted with non-null tool_name + port
- [ ] TDD tests 4K.X.3..4K.X.7 written and passing BEFORE wiring
- [ ] CI parity job runs every existing CLI integration test in both modes
- [ ] CI parity job is BLOCKING (not informational)
- [ ] Contract tests in cli_mcp_contract.rs pass
- [ ] CLI documentation updated with --via-mcp usage
- [ ] No regression: default in-process path unchanged, all existing tests
      green without --via-mcp

## Files

- ADD: .elai_cc/crates/cli/src/mcp_client.rs
- ADD: .elai_cc/crates/orchestrator/src/dispatch/mod.rs (Dispatch trait)
- ADD: .elai_cc/crates/orchestrator/src/dispatch/in_process.rs (refactor of current implicit path)
- ADD: .elai_cc/crates/orchestrator/src/dispatch/mcp.rs (McpClientDispatch)
- ADD: .elai_cc/crates/cli/tests/mcp_dispatch.rs (TDD)
- ADD: .elai_cc/crates/cli/tests/contracts/cli_mcp_contract.rs
- ADD: .elai_cc/scripts/ci-mcp-parity.sh
- ADD: workflow/papers/openhands-mcp-integration.md
- MODIFY: .elai_cc/crates/cli/src/main.rs (--via-mcp flag + dispatch selection)
- MODIFY: .elai_cc/crates/cli/src/commands.rs (--via-mcp threaded through
  SlashCommand::Task + SlashCommand::Explore variants)
- MODIFY: .elai_cc/crates/orchestrator/src/ssb.rs (3 CliMcp* action types
  SHIPPED s-68264 at SCHEMA_VERSION v89->v90; intact through current v103)
- ADD: .elai_cc/.github/workflows/ci-mcp-parity.yml (parity job, BLOCKING)
- ADD: .elai_cc/docs/cli.md (--via-mcp + --export-ssb documentation)
- ADD: .elai_cc/crates/cli/src/lib.rs (minimal library target so the
  integration test in crates/cli/tests/mcp_dispatch.rs can import
  CliMcpClient; CLI remains primarily a binary)

## Affected Verification Reports

- F:\.ELAI_workflow\docs\thefinalglance\verification\reports\steps\phase-4K.report.md
  — annotate C-39 CLI surface extension on closure.

## Next

After C-39 lands: every CLI tool invocation that goes through MCP is
observable end-to-end from the CLI binary. This unblocks (a) external MCP
clients to use ELAI tools without dashboard, (b) standardized CI for MCP
tool catalogs, (c) the prerequisite environment for C-44 (Composer 2 backend
tier) which needs the MCP dispatch path live to wire a third runtime.

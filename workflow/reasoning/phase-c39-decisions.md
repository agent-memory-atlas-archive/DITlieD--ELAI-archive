PHASE C-39 DECISIONS
session_id: s-68264
date: 2026-05-20

WHAT I WILL IMPLEMENT
Extend the ELAI CLI to optionally route tool dispatch through the MCP bridge
(DefaultMcpBridge in plugins crate) via --via-mcp flag on CliAction::Run.
Add 3 SSB action types (CliMcpDispatch/CliMcpServerStarted/CliMcpServerStopped).
Introduce Dispatch trait (tool-call layer only, not BabyStepExecutor replacement).
Add CI parity script. No new HTTP client (reuse DefaultMcpBridge).

KEY DECISIONS
1. No new mcp_client.rs: DefaultMcpBridge in plugins/src/mcp_bridge.rs already implements
   JSON-RPC 2.0 HTTP client (http_list_tools/http_call_tool). Add plugins dep with feature=mcp
   to CLI Cargo.toml instead.
2. Dispatch trait scope: wrap tool-call layer (plugins::McpBridge vs in-process PluginRegistry),
   NOT BabyStepExecutor. Lives in orchestrator/src/dispatch/.
3. --via-mcp wiring chain: args.rs:780 -> CliAction::Run.via_mcp -> RunArgs -> RuntimeWiring
   -> apply_task_runner_wiring (runtime_setup.rs:307) as single chokepoint.
4. SSB bump: 88->89, all_test_payloads 183->186, 14 sites per orchestrator-explorer map.
5. Auto-launch co-process: if --via-mcp and port 8200 not listening, spawn child process
   with same binary (elai mcp-server) bound to random port; emit CliMcpServerStarted SSB.
6. Test IDs: C39.1..C39.5 (not 4K.X.3..4K.X.7 to avoid phase-4K namespace collision).

EXPECTED TEST OUTCOME
C39.1: McpBridge::list_tools returns same set as in-process plugin registry
C39.2: McpBridge::invoke_tool round-trips arguments byte-equal (json Value round-trip)
C39.3: --via-mcp with no listener auto-launches co-process, CliMcpServerStarted emitted
C39.4: --via-mcp with external listener uses it, does NOT spawn co-process
C39.5: CI parity script validates output equivalence (modulo SSB action-type rename)
Contract: both InProcessDispatch and McpClientDispatch satisfy Dispatch trait for same inputs

UNKNOWNS
- Does elai binary currently support a "mcp-server" subcommand for co-process auto-launch?
  (check args.rs) - if not, must add CliAction::McpServer with --port flag
- Is port 8200 the hard-coded bridge port? Confirm in mcp_bridge.rs DEFAULT_MCP_SERVER_URL
- Task 1 OpenHands paper-ingestor: what patterns from OpenHands MCP integration apply here?

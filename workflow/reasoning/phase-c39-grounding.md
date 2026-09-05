GROUNDING SWEEP - C-39 MCP Bridge CLI Surface
session_id: s-68264
date: 2026-05-20

SPEC REFERENCES

1. C-35_tool_attention_dynamic_gating.md - GROUNDED - file exists; ToolGate/GatedCatalog confirmed
2. phase-4K.txt - GROUNDED - 427 lines; 4K.1-4K.5 confirm MCP bridge dashboard-only; HTTP JSON-RPC 2.0
3. plugins/src/mcp/server.rs - MISSING/MISMATCH - does NOT exist; actual: mcp_bridge.rs
4. crates/cli/src/main.rs - GROUNDED - CliAction::Run is sole task-runner entry; no Task/Explore variants
5. commands/task.rs + commands/explore.rs - MISSING/MISMATCH - neither file nor commands/ dir exists
6. github.com/All-Hands-AI/OpenHands - UNGROUNDED - not ingested; paper-ingestor Task 1 mandatory

CLAIM CLASSIFICATION
- CLI does not use MCP bridge: GROUNDED
- MCP bridge HTTP JSON-RPC 2.0 at localhost:8200: GROUNDED (mcp_bridge.rs:14-28)
- OpenHands uses MCP as primary tool surface: UNGROUNDED (needs paper-ingestor)
- /task and /explore are CLI subcommand entry points: MISMATCH (CliAction::Run is sole entry)
- ToolGate/GatedCatalog from C-35: GROUNDED
- WIRING_CONTRACT consumes plugins/src/mcp/server.rs: MISMATCH (correct: mcp_bridge.rs)

ACTIONS REQUIRED (autonomous resolution per POLICY 5)
1. P0: plugins/src/mcp/server.rs -> plugins/src/mcp_bridge.rs in implementation (file doesn't exist; we read mcp_bridge.rs)
2. MISMATCH: commands/task.rs + commands/explore.rs absent; --via-mcp attaches to CliAction::Run via RuntimeWiring
3. UNGROUNDED: paper-ingestor spawned for OpenHands MCP integration (Task 1)
4. VERIFY: Test IDs 4K.X.3..4K.X.7 renamed to C39.1..C39.5 to avoid phase-4K namespace collision

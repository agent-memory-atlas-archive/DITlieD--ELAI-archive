C-47b CLI-arm producer wire — decision doc (session s-14276, box)

WHAT
Close C-47b non-RUNPOD residue from HANDOFF-s-22601 Section 2 item 1: give the CLI tool-dispatch path a
long-lived, persist-backed ToolCalibrationRegistry so CLI tool calls record + persist calibration like the
dashboard arm (which is already SHIPPED and writes live).

GROUNDED STATE (verified this session by 2 scoping agents — CORRECTS the handoff framing)
- HANDOFF MISDIAGNOSIS: the handoff said this needs a ProductionAgentRpcServer/AgentRpcServer TRAIT-SURFACE
  change. FALSE. AgentRpcServer (ipc_accept.rs:92) has only handle_infer_request + handle_tool_request,
  neither calibration-related; ProductionAgentRpcServer is NOT on the MCP tool-call write path. NO trait change.
- Real seam: tool calibration is recorded ONLY inside DefaultMcpBridge. DefaultMcpBridge::with_calibration_registry
  (mcp_bridge.rs:641) already EXISTS but has ZERO production callers. The CLI MCP path builds
  DefaultMcpBridge::default() transiently per call (mcp_client.rs:20/34/52, dispatch/mcp.rs:198), so a shared
  Arc must be threaded into each construction.
- LOAD-BEARING: McpClientDispatch::dispatch_tool calls bridge.http_call_tool() directly (mcp.rs:207), which
  BYPASSES record_call. record_call fires only inside dispatch_tool_call_with_gate / with_authz. So wiring the
  registry WITHOUT switching the call path records NOTHING.
- Dashboard reference pattern: dashboard-v2/src-tauri/src/state.rs:598-634 (calibration_registry field +
  with_persist + env-gate ELAI_C47B_CASE_BASE_PERSIST_ENABLE fail-open) + commands_c47.rs (read). Persistence
  write IS live on the dashboard path.
- Second McpClientDispatch::new site (runtime_setup.rs:542) has no project_root -> stays registry-less (fallback).

KEY DECISION — gated registry + gated call-path switch (genuine producer, ZERO default-behavior change)
1. Construct ONE shared Arc<ToolCalibrationRegistry> at run_subcommand.rs (project_root available ~L191) via
   CasePersistStore + ToolCalibrationRegistry::new().with_persist(), env-gated ELAI_C47B_CASE_BASE_PERSIST_ENABLE,
   FAIL-OPEN on store-open error (mirror dashboard state.rs verbatim). Call rehydrate_all_from_persist at startup.
   When the gate is OFF or the store fails to open -> registry is None (no behavior change).
2. Add Option<Arc<ToolCalibrationRegistry>> field + builder to McpClientDispatch (dispatch/mcp.rs:48). Thread the
   shared Arc into every per-call DefaultMcpBridge via .with_calibration_registry(registry.clone()).
3. PRODUCER WRITE (the real fix): switch McpClientDispatch::dispatch_tool from bridge.http_call_tool() to
   dispatch_tool_call_with_gate ONLY when the registry is Some (gate ON). When None (gate OFF, the default),
   keep the EXACT current http_call_tool path -> byte-identical default behavior. The executor MUST verify that
   dispatch_tool_call_with_gate is a superset of http_call_tool (adds gate+record around the SAME underlying call)
   and that with NO authz policy configured the gate defaults to ALLOW (no tool-call denial regression). If the
   switch is NOT behavior-preserving even when gated, ship steps 1+2 (registry holder + rehydrate, symmetric with
   dashboard) and DOCUMENT the call-path switch as the precise residual rather than forcing a risky change.
4. SCOPE: this covers the MCP tool-call path (matches the dashboard arm + tool_calibration). The agent-pool
   in-process ToolDispatcher (edit/file_read/validate) has no calibration hook and is OUT OF SCOPE (would be new
   code, not a wire) — note it as out-of-scope, do not build it.

ENV-GATE / SCHEMA
- ELAI_C47B_CASE_BASE_PERSIST_ENABLE (existing, default-off). Fail-open. ZERO schema bump (reuses shipped C-47
  ToolCalibrationEvent variants; persistence is the observable effect).

EXPECTED TEST OUTCOME
- cargo check -p elai-orchestrator -p elai-cli --tests GREEN.
- Contract test must SOURCE-GREP for "dispatch_tool_call_with_gate" in the CLI dispatch path to prove the switch
  landed (per scoping agent: a registry wire without the switch is a silent no-op).
- When the env gate is OFF: CLI MCP tool-call path is byte-identical to today (regression guard).

UNKNOWNS / RISK
- dispatch_tool_call_with_gate behavior-preservation vs http_call_tool (executor verifies before switching).
- Cross-process SQLite: CLI + dashboard both opening case_base.sqlite3 — CasePersistStore is Arc<Mutex<Connection>>
  single-process-safe; cross-process relies on sqlite file locking. Confirm WAL + busy_timeout in CasePersistStore;
  if absent, note "do not run CLI + dashboard calibration concurrently" rather than risking a locked-db panic.
- DefaultMcpBridge construction-site count (per-call) — thread the Arc into each; fix any signature ripples.

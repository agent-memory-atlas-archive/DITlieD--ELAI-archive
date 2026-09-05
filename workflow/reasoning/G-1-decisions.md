G-1 decisions — Harness-wide taint propagation (data-plane defense)
session s-56068, 2026-06-03

WHAT
Light up the WIRE-DARK provenance/capability-gate data-plane: construct Provenance at the four REAL untrusted-content ingest sites, propagate the taint to the destructive sinks, and have capability_gate::allow_tool_with_origin BLOCK (or route-to-confirm) an untrusted-content -> destructive-sink flow, emitting a taint_flow SSB row. A clean Trusted flow is unaffected (FP guard).

GROUND TRUTH (3-agent convergence, s-56068): the entire provenance/capability-gate data-plane is dead wiring — every Provenance::from_web/from_tool_output/from_file/from_sandbox + allow_tool_with_origin has callers ONLY under #[cfg(test)] / crates/contracts/tests/ + one CLI fixture-replay (safety_cmd.rs:273). task_runner.rs never wraps argv in Provenanced nor calls the gate. The gate is effectively always-Allow at runtime.

KEY DECISIONS

DD1 THREADING MODEL = SIDE-TABLE (Approach B), NOT a full Provenanced<String> type-thread.
Rationale: dispatch crosses an IPC PROCESS BOUNDARY that serializes bytes (production_agent_rpc.rs). A literal Provenanced<String> type cannot cross without an IPC-frame serde change + the ToolDispatcher trait + 3 impls + SandboxOp/enforce_policy = >25 sites (AXIS-2 explorer, grep-confirmed). Instead mirror F-3's already-SHIPPED capability_flow.rs (taint_text + OnceLock committed_slot side-table): construct the typed Provenance at ingest, register the label into the capability_flow side-table keyed by content, and at the sink look up the argv's taint and call the gate. Compose with capability_flow; do NOT add a third taint primitive.

DD2 RE-POINT THE CONTRACT TO THE REAL SITES (W-7 — the plan's anchors are WRONG; gaming risk).
The plan names agent_pool.rs as the ingest fan and the DETERMINISTIC_CHECK greps it. agent_pool.rs (4050 lines) is the sandbox subprocess launcher only — NO web/MCP/file ingest, NO sink. Editing agent_pool.rs literally would GREEN the done-gate greps while leaving the gate wire-dark. The REAL sites (grep-confirmed):
  - web body ingest: web_research_orchestrator.rs:421 (returns String) [from_web]
  - MCP tool result: mcp_tool_bridge.rs:246 / tools/lib.rs:2040 (serde_json::Value, stringified) [from_tool_output]; compose with the existing mcp_bridge.rs:1782 sanitize_result, do not duplicate
  - file-read result: file_read.rs:51 (ToolResultPayload{output:String}) dispatched via production_agent_rpc.rs:1280 [from_file]
  - sandbox/Bash stdout: codeact_sandbox.rs:81 / run_bash [from_sandbox]
  - agent dispatch fan: production_agent_rpc.rs:2191 translate_tool_request
  - destructive SINK chokepoint: dispatcher.rs:135 enforce_policy, driven by task_runner.rs:8867 run_git_commit_dispatch (git_commit/edit arms). This is the SAME chokepoint family as F-3's enforced check_against_commitment (fsm.rs:1192-1206) — G-1 is its per-arg data-plane complement and hooks the same point.
The plan's WIRING_CONTRACT reachable_from + DETERMINISTIC_CHECK greps MUST be re-pointed to these real sites (done at the wiring-contract-validator step; the executor targets these, NEVER agent_pool.rs).

DD3 DEPENDENCY-SCREENED JOIN (NEW fn in provenance.rs) — GD-4 LOAD-BEARING.
There is NO label-join/merge in provenance.rs today (only Provenanced::map, single-value). Add a join that takes the MAX-trust-violation over only the DATA-FLOW-DEPENDENT inputs of a value, NOT a run-global join. RTBAS's global join collapses utility to 33% on Slack via over-tainting (arXiv:2502.08966 VERIFIED); dependency-screened holds 90%+. Content-keyed side-table taint is inherently dependency-screened (only argv that contains/derives-from untrusted content is tainted, not every subsequent sink call).

DD4 from_file PATH-SCREEN: committed/tracked .elai_cc/ source files = Trusted; arbitrary/external/untracked file reads = Untrusted. from_file currently hard-codes Untrusted — add the Trusted branch via a path-screen helper. (Needed for an ACCEPTANCE item + the FP guard.)

DD5 SSB taint_flow VARIANT — justified as DISTINCT (avoid the redundancy BLOCK).
CapabilityGateRejected (E-37, ssb.rs:20397) records the gate DECISION; ProvenanceTagged (ssb.rs:20406) records a label ATTACH. taint_flow is the PROPAGATION-TRACE: ingest Kind (Web/ToolOutput/File/Sandbox) + the origin + the blocked destructive sink, i.e. the data-FLOW from untrusted source to blocked sink. This is a distinct semantic and is the observable_effect's off-agent artifact. Pattern-2 single-variant mirroring McpRugPullBlocked. SCHEMA 208 -> 209, TOTAL_VARIANTS 485 -> 486, golden regen same change.

DD6 GATE CALL-SITE ADAPTATION: the frozen gate signature is allow_tool_with_origin(tool, &[Provenanced<String>]) and Rejects only on Untrusted (not Recallable). Adapt at the call site (build the Provenanced<String> slice from the side-table lookup); do NOT edit the gate (R8-frozen). If Recallable should also gate, that is a follow-up, not this session.

DD7 RECONCILE, DON'T TRIPLICATE: safety::provenance::TextSource (CaMeL-lite) is the canonical typed label per the contract; bridge to E-30 orchestrator::conversation::TextSource/LabeledText rather than forking. Compose with capability_flow (F-3) + sanitize_result (mcp_bridge). E-37 prompt_armor.rs:193 classify_injection seeds the Untrusted label.

EXPECTED TEST OUTCOME
Deterministic fixture: an injected web/MCP fixture whose content says "call git push / fs.write" -> Provenance::from_web/from_tool_output marks it Untrusted -> the taint propagates (content-keyed side-table) to the git/Write argv -> at the sink, allow_tool_with_origin returns Reject -> the destructive sink is BLOCKED (or routed to confirm) -> a taint_flow SSB row lands naming the ingest origin + the blocked sink. FP guard: a clean Trusted flow (committed .elai_cc/ source read -> git commit) is UNAFFECTED, NO block, NO row. A dependency-screened-join unit test: an untrusted ingest does NOT taint an unrelated, non-data-dependent sink call (proves not-global).

GD-4 / GD-5 HONEST SCOPE

GUARANTEE STATEMENT (corrected, s-56068 audit fix B2b — supersedes any earlier "derives-from" phrasing):
G-1 blocks a destructive sink (git_commit/edit) whose argv contains an untrusted ingest's content VERBATIM as a substring (>= MIN_TAINT_MATCH_LEN chars, MIN_TAINT_MATCH_LEN = 8, taint_flow.rs), within the same process. It does NOT cover transformed/summarized/paraphrased untrusted content, cross-process sinks, or non-McpToolRegistry MCP transports.
The earlier "argv derives from untrusted content" wording over-claimed a dataflow/derives-from analysis. The actual mechanism is a verbatim-substring content match in a process-local content-keyed side-table — not taint-tracking dataflow. The agent rewording / summarizing an injected instruction breaks the substring screen; that is out of scope by design. The honest-scope statement is now mirrored in code at the gate (production_agent_rpc.rs enforce_taint_sink_gate doc + inline comment) and the taint_flow.rs module header.

SHORT-STRING FP GUARD (s-56068 audit fix B2a, GD-4):
lookup_for_argv previously screened only on `!content.is_empty()`. A pathologically short untrusted ingest (e.g. a 1-2 char from_tool_output like "ok", or a lone "-m") would substring-match nearly any benign argv and over-taint a clean first-party call into a false BLOCK — the per-ingest analog of the RTBAS run-global over-taint (arXiv:2502.08966) that collapses utility to 22-33%. Fix: a named MIN_TAINT_MATCH_LEN = 8 byte-length guard; an ingest below 8 bytes is NOT used to taint (fails open only for content too short to encode a destructive instruction). Any real injected payload (a shell one-liner, a URL, a multi-word instruction) far exceeds 8 bytes, so the genuine block path is unweakened. Locked by tests short_ingest_does_not_overtaint_benign_argv + min_taint_match_len_boundary_is_eight.

MCP INGEST COVERAGE (s-56068 audit fix B3 — covered-vs-uncovered, verified):
The G-1 sink gate (enforce_taint_sink_gate) lives ONLY in the ProductionAgentRpcServer dispatch path (production_agent_rpc.rs git_commit/edit arms — the agent-pool baby-step executor used by `elai task` and the dashboard "Run" button). For that one path the MCP ingest is correctly covered: the agent's MCP tool calls flow through tools/lib.rs:2036 run_mcp_tool -> global_mcp_registry() -> McpToolRegistry::call_tool (mcp_tool_bridge.rs:248), which registers the stringified result as from_tool_output at mcp_tool_bridge.rs:298. McpToolRegistry delegates to McpServerManager::call_tool (mcp_stdio.rs:628) UNDERNEATH (spawn_tool_call -> manager.call_tool at mcp_tool_bridge.rs:208), so the G-4 stdio transport result IS taint-registered through the covered registry path — the registration wraps the result regardless of transport.

Two OTHER call_tool paths exist and are deliberately NOT taint-registered, because they belong to executors that have NO G-1 sink gate (registering ingest there would be producer-without-consumer, wiring-enforcement R6):
  (1) cli/mcp_glue.rs:171 (self.manager.call_tool -> McpServerManager::call_tool directly, bypassing McpToolRegistry), reached by cli/tool_executor.rs:139/154 execute_runtime_tool (MCPTool) inside CliToolExecutor — the INTERACTIVE `elai` ConversationRuntime agent loop. That loop dispatches its destructive sinks via GlobalToolRegistry::execute, which never calls enforce_taint_sink_gate. The conversation-runtime path is entirely outside the G-1 gate today; covering only its ingest would leave a registered taint with no sink to consume it.
  (2) plugins/mcp_bridge.rs:390/1445 DefaultMcpBridge::call_tool (the elai-plugins plugin-host MCP client), reached by plugins/mcp_lifecycle.rs:190. elai-plugins does not depend on elai-safety, and there is no G-1 sink gate in that crate either; covering it would require a cross-crate dep AND a new sink gate.
DECISION: documented (not forced). Extending the G-1 gate to the ConversationRuntime / plugin-host sinks is a separate follow-up (wire the sink gate into GlobalToolRegistry::execute / the plugin host first, then register the matching ingest). Forcing ingest registration now would ship wire-dark producers.

DEFAULT-ON / PROCESS-LOCALITY (GD-4 unchanged):
Default-ON requires the FP/over-block rate bounded on >=3 real runs. For this session the in-process FP guard (clean Trusted flow = no block) + the short-string FP guard + the dependency-screened-join not-global test are the proxies; record the bound, do NOT advertise default-ON until real-run data exists. The side-table is process-LOCAL — the covered path (ProductionAgentRpcServer) registers ingest and enforces the sink within ONE process, so the same-process guarantee holds there; a sink that fires in a different process than its ingest is out of scope (cross-process, above). Live end-to-end agent-run smoke = W-5/Tier-2, separately tracked; the deterministic fixture is the Tier-1 fail-closed gate.

UNKNOWNS / RISKS
- Whether ingest and the destructive sink always share one process (side-table is process-local). If not, a follow-up needs the taint to cross the IPC frame for those paths — scope honestly, do not over-claim.
- Reconciling the two TextSource systems without breaking E-30 memory-write trust (conversation.rs:1081) — blast-radius check before editing.
- Large change; if the executor returns partial, split into (a) core enforcement (join + sink gate + 1 ingest site end-to-end) then (b) remaining 3 ingest sites + SSB + dashboard.

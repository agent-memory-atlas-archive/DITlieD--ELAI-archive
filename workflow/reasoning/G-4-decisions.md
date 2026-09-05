G-4 decisions — Live MCP rug-pull / line-jump re-pin gate
session s-56068, 2026-06-03

WHAT
Light up the WIRE-DARK mcp_schema_pin.rs so a tool-list schema change after first-connect approval is BLOCKED fail-closed, emitting a new mcp_rug_pull_blocked SSB row carrying the before/after tool-DESCRIPTION diff, requiring explicit operator re-approval.

KEY DECISIONS (grounded by context-compiler + integration-explorer-elai, s-56068)
DD1 ENTRYPOINT. The plan named notifications/tools/list_changed. That handler DOES NOT EXIST: mcp_bridge.rs is a request/response HTTP-POST JSON-RPC client with no notification-receive loop (confirmed both agents). The real production tools/list ingestion entrypoint is DefaultMcpBridge::http_list_tools (mcp_bridge.rs:1451), reached on every (re)connect via connect (:1309->:1436). DECISION: hook the re-pin into http_list_tools, capturing the raw input_schema before it is dropped at :1476. This is a genuine W-3 production entrypoint reachable from a live MCP connection. The "runtime tool-list update" is therefore a reconnect-time re-fetch, not a server push.
DD2 NO THIRD PIN SYSTEM. E-37 already ships safety/tool_description_pin.rs (pin_or_verify_tool -> DescPinStatus::Drift{pinned_hash,observed_hash}) for per-tool description drift. G-4 extends plugins/mcp_schema_pin.rs (list-level blake3 pin via hash_tool_list:69 + pin_or_verify:134 + update_pin:177 + PinRecord:49 + ToolDef:57 + PinStatus:21) and computes the per-tool description diff for the violation row by reusing PinRecord/ToolDef. The two pin layers COMPOSE (list-level vs per-tool); G-4 does NOT build a third and does NOT touch E-37.
DD3 KILL THE WIRE-DARK. pin_or_verify has NO production caller today (test-only) and the McpSchemaPinViolation SSB variant (ssb.rs:5874) is declared-but-never-emitted in production. So the first-connect PinRecord is never written in prod. DECISION: wire pin_or_verify into the FIRST-connect path inside http_list_tools as well, so the first-connect pin is actually written and the re-verify on reconnect has something to compare against. Without DD3 the whole gate stays wire-dark (R6/R7).

OPERATOR RE-APPROVAL. No per-tool re-approve mechanism exists (capability_gate is binary Allow/Reject; trust_dialog is one-time session trust). Mirror the fail-closed Ask->Deny seam (mcp_bridge.rs:977); the dashboard re-approve command calls update_pin (mcp_schema_pin.rs:177), the existing re-pin primitive. Default fail-closed: a violating tool stays blocked until update_pin is called via explicit operator action.

SSB / SCHEMA. New Pattern-2 single-variant action_type mcp_rug_pull_blocked (Kind not needed unless shared payload; single variant). Baseline from SOURCE (CLAUDE.md 191/464 and G-0 ~199/~476 are STALE): SCHEMA_VERSION=207 (ssb.rs:2909) -> 208; TOTAL_VARIANTS=484 (action_type_lock.rs:549) -> 485; golden lock regen same commit. Emit via the mcp_authz_ssb_adapter.rs:104 + bridge-field:445 pattern, mirroring the E-22 lazy-schema emit block (mcp_bridge.rs:1464-1511).

FRONTEND PAIR. get_mcp_security_status (dashboard-v2 src-tauri commands.rs:249) is a vec![] stub. De-stub it to surface pin status + the before/after description diff in a drawer (mirror McpStatusPanel.svelte / an existing diff drawer). Tauri command -> bindings emit -> Svelte. cross_lang_diff must pass.

EXPECTED TEST OUTCOME
Deterministic fixture: an in-process MCP server stub that returns tool list T1 on first http_list_tools (pins), then a changed description T1' on a second http_list_tools (reconnect re-fetch) -> pin_or_verify returns Violation -> the call is BLOCKED, an mcp_rug_pull_blocked SSB row lands carrying the before/after description diff + the ingest origin, and the tool stays blocked until update_pin. FP guard: an unchanged re-fetch (T1 == T1) returns Verified, NO block, NO row.

GD-5 HONEST SCOPE
The gate fires on reconnect-time re-fetch (the only trigger the HTTP request/response transport supports). A live server-push notifications/tools/list_changed path would require a streaming transport receive-loop that does not exist — scoped OUT, noted, not advertised. stdio + registry transports (mcp_stdio.rs:1318, mcp_tool_bridge.rs:169) also ingest tools/list; covering them is a second insertion point, deferred unless trivial.

UNKNOWNS / RISKS
- Whether http_list_tools is on a hot path where re-pin cost matters (blake3 over a tool list is cheap; low risk).
- Whether capturing raw input_schema before :1476 requires a signature change to a shared fn (blast-radius check at edit time).
- GD-4 FP-rate must be bounded on >=3 real runs before any default-ON claim; for this session the deterministic FP guard (unchanged re-fetch = no row) is the in-process proxy; record the bound, do not over-claim default-ON until real-run data exists.

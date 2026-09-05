SESSION_HANDOFF: E-37 prompt-injection defense
Session: s-1233230
Date: 2026-05-21
Blocked by: stale global FSM EXECUTING/E-51 lock + MCP server unavailable

WHAT WAS COMPLETED THIS SESSION
- Step 1: context-compiler ran, wrote workflow/sessions/s-1233230/context-package.md
- Step 2: scope-planner ran, wrote workflow/sessions/s-1233230/exploration-plan.md (6 partitions)
- Step 3: all 6 integration-explorers ran (parallel batches of 3)
  - safety-new-crate: wrote workflow/sessions/s-1233230/exploration/safety-new-crate.md
  - plugins: wrote workflow/sessions/s-1233230/exploration/plugins.md
  - web-search: wrote workflow/sessions/s-1233230/exploration/web-search.md
  - orchestrator: findings returned inline (FSM blocked write), NOT on disk
  - cli: findings returned inline (FSM blocked write), NOT on disk
  - dashboard: findings returned inline (FSM blocked write), NOT on disk
- Synthesizer: NOT yet run (blocked)
- Steps 4-12: NOT yet run (blocked)

KEY FINDINGS FROM EXPLORATION (must feed synthesizer on resume)

safety-new-crate:
- safety crate EXISTS at .elai_cc/crates/safety/ (not new) - has destructive_verb_allowlist + trust_dialog
- RENAME REQUIRED: safety::provenance::Provenance -> DataProvenance (collision with memory::Provenance C-48)
- RENAME REQUIRED: safety::capability_gate::GateDecision -> CapGateDecision (3 existing GateDecision types)
- Dep graph: elai-web-search -> elai-safety NOT yet wired (needs Cargo.toml addition, no cycle)
- elai-orchestrator -> elai-safety already wired (Cargo.toml:25)
- SCHEMA_VERSION = 96, E-37 bumps to 97 (9 new variants)
- Existing .expect() in Lazy<RegexSet> init is approved pattern for static init only

plugins:
- http_list_tools insertion: per-tool loop lines 391-405, BEFORE out.push()
- http_call_tool insertion: AFTER is_deny_listed (line 418), BEFORE post_jsonrpc (line 430)
- sanitize_description (line 574) and sanitize_result (line 213) are orthogonal - no double-sanitize risk
- mcp_schema_pin.rs EXISTS: exports pin_or_verify, update_pin, hash_tool_list, PinStatus, PinRecord, ToolDef, SchemaPinError
- http_list_tools discards input_schema - must maintain internal Vec<ToolDef> for per-tool pinning
- Spotlight wrap should NOT go in mcp_bridge - belongs at task_runner.rs on McpCallResult.content

web-search:
- Egress allowlist hook: BEFORE .client.get(url).send().await at fetcher.rs:109 (between lines 105-109)
- Domain already extracted at line 94 via extract_domain
- Spotlight wrap: NOT in fetcher.rs or extractor.rs - belongs at web_research_orchestrator.rs::fetch_and_extract lines 601-616 (sole call site)
- ContentExtractor trait signature LOCKED (object-safe, "Signature locked for 4D.17") - cannot change
- New errors needed: WebSearchError::EgressBlocked{url}

orchestrator (inline findings - not on disk):
- argv-construction sites: only 2 actual Vec<String> sites in agent_pool.rs:349-352 (SYSTEM provenance, gate-exempt)
- Provenanced<String> applies to data flowing INTO context (web/MCP results), not process-launch argv
- PreToolUse hook invocation at task_runner.rs:5608-5655 is the injection point for provenance metadata
- SCHEMA_VERSION = 96 (line 955 of ssb.rs), 284 existing SsbActionType variants
- E-37 -> SCHEMA_VERSION 97
- AgentRole is enum in elai_ipc_types crate (NOT in orchestrator)
- PerRoleRuntime uses BTreeMap<AgentRole, RuntimeBackend> (deterministic ordering)
- InjectionClassifier must be added to BOTH Default::default() AND all_roles() in per_role_runtime.rs
- 15 current AgentRole variants; InjectionClassifier maps to RuntimeBackend::Sidecar
- all_roles() enforcement gate at lines 191-196 (contains_key assertion)

cli (inline findings - not on disk):
- MCP dispatch: CliAction::Mcp -> main.rs:309 -> LiveCli::print_mcp -> handle_mcp_slash_command (commands.rs:2916)
- parse_mcp_command at commands.rs:1998 has catch-all [action, ..] -> "Unknown /mcp action" - "pin" hits this
- NO McpArgs/McpCommand enum - uses blob string CliAction::Mcp { args: Option<String> }
- gateway-rejection guard at main.rs:252-281 (warn-only, orthogonal to pin command)
- RECOMMENDED: add CliAction::McpPin { server, tool, update, output_format } in args.rs
  then dispatch in main.rs + handler in commands.rs (3-file touch, not 1 as plan estimated)
- ALTERNATIVE: add ["pin", server, tool] arms to parse_mcp_command (2-file touch but fragile)

dashboard (inline findings - not on disk):
- Command registration: tauri::generate_handler![...] in lib.rs:213-640, append new commands after line 637
- Pattern: separate commands_e37.rs file (follow commands_e1.rs pattern), mod in lib.rs, register in invoke_handler
- CSP is currently NULL (tauri.conf.json:28 "csp": null) - must write FULL policy from scratch
  Recommended: "default-src 'self' tauri: asset:; img-src 'self' data: blob: tauri: asset:; script-src 'self'"
  RISK: null->non-null breaks Tauri dev server asset loading if tauri:/asset: schemes missing
- Drawer pattern: follow CosmosDrawer.svelte shape (onMount+onDestroy, dual setInterval 30s+5s, $state<T[]>)
- Return shape: json!({ "spotlight_count": u32, "armor_blocked": u32, "armor_suspicious": u32, "egress_blocked": u32, "output_flagged": u32, "tool_pin_violations": u32, "last_updated_ms": i64 })
- stub.ts needs new SafetyStatus interface + get_safety_status export with mock fallback
- SafetyDrawer.svelte goes in src/lib/components/ (not drawers/)
- get_mcp_security_status at commands.rs:144-146 is E-24 MVP stub - E-37 is sibling, not replacement

NEXT STEPS ON RESUME
1. Fix workflow/.fsm-state.json -> "state": "IDLE" (user must do manually)
2. Restart Claude Code (MCP server needs to reconnect for mcp_fsm_transition)
3. Re-run synthesizer-elai for session s-1233230 (reads 3 on-disk exploration files + inline findings above)
4. Run spec-grounder (Step 4)
5. Run prompt-engineer (Step 5)
6. Continue Steps 6-12 per b-plan-runner skill

CRITICAL RENAMES FOR IMPLEMENTATION
- safety::provenance::Provenance -> DataProvenance
- safety::capability_gate::GateDecision -> CapGateDecision
- SCHEMA_VERSION 96 -> 97
- AgentRole::InjectionClassifier added to elai-ipc-types crate
- Spotlight wraps at web_research_orchestrator.rs::fetch_and_extract, NOT in fetcher/extractor
- CLI needs 3 files: args.rs + main.rs + commands.rs
- CSP: write full string from scratch (was null)
- commands_e37.rs (new file) for dashboard Tauri commands

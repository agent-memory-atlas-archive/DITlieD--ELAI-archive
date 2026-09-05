F-trancheF-decisions.md

Session: s-22600 Tranche F (orchestrate followup, small wires batch 1)
Date: 2026-05-31
Owner: Tranche F executor (fixer)

Tranche F was a production-wiring batch for the Tranche C substrate (BashComplexityGuard / SandboxTemplateGuard / hook_prompt::prompt_decision / edit_checkpoints). 3 ship + 1 defer.

Decisions

DECISION-F-EXEC-1 (ITEM F.1.a -- BashComplexityGuard wire)
WHERE: .elai_cc/crates/orchestrator/src/permissions.rs::PermissionPolicy::authorize_with_context (top of fn body, before deny_rules check).
WHAT: Inserted a tool_name=="Bash" + bash_complexity_guard_enabled() gated block. On Refused verdict, returns PermissionOutcome::Deny carrying the guard's reason.
WHY: Mirrors free_tier_guard precedent (cost/free_tier_guard.rs:103) of "verdict surface returns; emit at policy site". Permissions has no SsbEmitter handle today; SSB emission is delegated to the caller of authorize_with_context per DECISION-T-F-4 (Tranche F context-package).
WIRE-DARK: env var ELAI_E25_BASH_COMPLEXITY_GUARD_ENABLE, default OFF.

DECISION-F-EXEC-2 (ITEM F.1.b -- SandboxTemplateGuard wire)
WHERE: .elai_cc/crates/orchestrator/src/permissions.rs::PermissionPolicy::authorize_with_context (immediately after the F.1.a block).
WHAT: Inserted a tool_name in {"microsandbox", "MicrosandboxLaunch", "Sandbox"} + sandbox_template_guard_enabled() gated block. On Refused verdict, returns PermissionOutcome::Deny carrying the guard's reason.
WHY: Task prompt literally specifies "into permissions.rs (before sandbox launch)" -- per orchestrate-protocol H-1 the task prompt overrides the explorer's DECISION-T-F-1 (which preferred skill_templating.rs). The explorer correctly noted permissions.rs has NO sandbox-launch tool branch today; the wire filters on the canonical sandbox-launch tool names so the guard stays dormant in production (none of those tool names are wired today) while honoring the task-prompt insertion contract. When microsandbox or a Sandbox tool wire lands in a future tranche, the guard will fire automatically at this site.
WIRE-DARK: env var ELAI_E25_SANDBOX_TEMPLATE_GUARD_ENABLE, default OFF.

DECISION-F-EXEC-3 (ITEM F.1.c -- ApprovalFatigueGuard)
DEFERRED per DECISION-T-F-3 (Tranche F context-package). No code edits. See workflow/reasoning/F1c-approval-fatigue-deferral.md for the detailed reasoning. Stubs.md row F1c-approval-fatigue added with DOCUMENTED-DEFERRAL.

DECISION-F-EXEC-4 (ITEM F.2 -- pre-MCP llm_guard wire)
WHERE: .elai_cc/crates/plugins/src/mcp_bridge.rs::DefaultMcpBridge::dispatch_tool_call_with_authz (top of fn body, BEFORE the no-policy delegation).
WHAT: Added a new optional struct field `llm_guard_config: Option<crate::llm_guard::LlmGuardConfig>` plus a builder method `with_llm_guard_config(cfg)`. The dispatcher consults pre_mcp_call_decision FIRST (outermost) when an LlmGuardConfig is installed; Deny -> emit GATEWAY_POLICY_DENIED + return ProtocolError; Ask -> emit GATEWAY_HITL_PENDING + return ProtocolError (fail-closed per DECISION-T-F-8); Err -> emit GATEWAY_POLICY_DENIED + return ProtocolError (LlmGuardConfig.fail_closed honoured inside the guard fn).
WHY: Matches DECISION-T-F-5 (keep both hook_prompt::prompt_decision and llm_guard::pre_mcp_call_decision; Tranche F wires the latter into the MCP path) + DECISION-T-F-7 (LLM-guard runs FIRST in dispatch chain, matches "PreToolUse" semantic). Cfg-gated under feature="mcp" so the wire is only compiled when llm_guard's reqwest-backed implementation is available. Default-None on the new field preserves back-compat for installs that have not opted in.
WIRE-DARK: by virtue of the Option<LlmGuardConfig> field. No env var; wiring happens via bootstrap code that calls with_llm_guard_config explicitly. No new env var name introduced per task-prompt "wire-dark; if gate OFF, callers default to Allow per to_decision_or_err pattern documented in WARN-1" -- the gate IS the presence of LlmGuardConfig.

DECISION-F-EXEC-5 (ITEM F.3 -- snapshot_pre_edit + evict_session wires)
WHERE (snapshot):  .elai_cc/crates/orchestrator/src/task_runner.rs::drive_task (AFTER the PreToolUse hook Ok-no-block return, BEFORE the irreversible-classifier gate at the previous L6440).
WHAT (snapshot): Gated by !hook_pre_blocked && edit_checkpoints_enabled(). Calls a new helper fn `extract_edit_target_from_task_summary` that scans task_summary for one of "Edit"/"Write"/"NotebookEdit" tool mentions AND a `file_path=` / `path=` token. When both match, reads the target file from disk and calls snapshot_pre_edit. Snapshot failure logged at warn (non-fatal) per DECISION-T-F-12.
WHY heuristic: BabyStep struct (baby_step.rs:141) has NO target_file / tool_name field. The DECISION-T-F-13 "extract from planned baby-step struct" option is structurally unavailable. The heuristic extractor is option (iii) from the explorer's HIGH-2 validation warning. Wire-dark default OFF means the heuristic is dormant in production unless an operator explicitly opts in.
WHERE (evict): same file, AFTER the dispatch_stop block, BEFORE the PreCompact block.
WHAT (evict): Gated by edit_checkpoints_enabled(). Calls evict_session against a synthesized session_id (format "task_seq_{seq}") -- matches the snapshot-side session_id format so the eviction reaps what was written. Eviction failure logged at warn (non-fatal).
WHY: session_dag.rs has no canonical end-session fn (grep verified zero tear_down / drop_session / end_session hits) so dispatch_stop IS the canonical eviction trigger per DECISION-T-F-11. HookPoint::PostCommit eviction is NOT wired per DECISION-T-F-9 (variant does not exist; 4 variants only at hook_dispatcher.rs:120).
WIRE-DARK: env var ELAI_E25_EDIT_CHECKPOINTS_ENABLE, default OFF.

DECISION-F-EXEC-6 (session_id source for edit-checkpoints wires)
WHAT: Synthesized per-task session_id as `format!("task_seq_{seq}")` mirroring the existing pattern at task_runner.rs:4591 (`format!("task_seq_{}_phase_{}", seq, phase.id.0)`).
WHY: TaskSubmittedPayload has no session_id field (ssb.rs:10587); ResolvedConfig has no session_id field (only cursor_path + per-subsystem config). The seq value is the canonical per-task monotonic id. Using the same format on both snapshot and evict sites guarantees evict_session reaps exactly what snapshot_pre_edit wrote.

Build verdict
cargo check --workspace --tests : GREEN.
Run 1 (after F.3): GREEN, ~1m 33s
Run 2 (after F.1.a + F.1.b): GREEN, ~1m 03s
Run 3 (after F.2): GREEN, ~1m 05s

Files modified
.elai_cc/crates/orchestrator/src/permissions.rs (F.1.a + F.1.b inserts at top of authorize_with_context)
.elai_cc/crates/orchestrator/src/task_runner.rs (F.3 snapshot insert in drive_task; F.3 evict insert after dispatch_stop; new helper fn extract_edit_target_from_task_summary near truncate_for_log)
.elai_cc/crates/plugins/src/mcp_bridge.rs (F.2 new struct field llm_guard_config + new builder with_llm_guard_config + guard block at top of dispatch_tool_call_with_authz; struct-init updated in Default + new() paths)

No deletions. No new dependencies. No schema bump (all emissions ride existing kinds per Pattern-2). Anti-stale-training compliant (no .unwrap() in prod paths; blake3 used by substrate; named-const-not-magic-number).

Cross-reference
F:\.ELAI_workflow\workflow\sessions\s-22600\context-package-trancheF.md (DECISION-T-F-1 through T-F-13)
F:\.ELAI_workflow\workflow\reasoning\F1c-approval-fatigue-deferral.md (F.1.c deferral)
F:\.ELAI_workflow\workflow\sessions\s-22300\stubs.md (rows updated below)

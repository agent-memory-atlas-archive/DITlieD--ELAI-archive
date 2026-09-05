s-22601 non-RUNPOD followup scoping verdicts
2026-06-01 | grounded via parallel scope workflow (12 sonnet read-only agents); full briefs cached in the workflow output

Each verdict is grounded by the agent's cited file:line evidence (see brief grounding[] arrays).

DO (executed this session, sequential):
T1  C-47b CasePersistStore producer-side write. Persist layer + registry persist-branch already exist; only the BOOTSTRAP wire is missing. Insert at cli/runtime_builder.rs build_agent_pool_rpc_server (~L728) + dashboard state.rs AppState. Env gate ELAI_C47B_CASE_BASE_PERSIST_ENABLE (existing). Blocking unknown: confirm with_calibration_registry seam (DefaultMcpBridge has it @mcp_bridge.rs:641-645; RpcServer may not) -- executor verifies first, installs on the bridge if the RpcServer lacks the seam, DEFER-with-reason a single arm only if unreachable.
T2  F.1.c ApprovalFatigueDetected + F.1.a BashComplexityBlocked SSB emit. Both guards have build_payload() but no caller. Add ssb_writer: Option<Arc<Ssb>> to ConversationRuntime (mirror cloud_mode.rs:781), emit at the existing Refused sites, wire with_ssb_writer at runtime_builder.rs:346. Both wire-dark by their existing guard gates.
T4  entropy quarantine final -1 leak (38->37). One site: cli/structured_output_cmd.rs:481 SystemTime::now() lacks the INLINE entropy-allow trailer (Tranche E placed it as a block comment @L467, which the per-line grep does not honor). Add inline `// entropy-allow: cli-smoke-observability-ts-ms-only`. 1 LOC, zero build risk.
T7  E-52 ScipRefresh dep-free CLI emit. SsbActionType::ScipRefreshCompleted + payload already LOCKED in ssb.rs (6517 / 12513 / 19934). Add SymbolGraphAction::Refresh variant + parse arm + scip_cmd::run_refresh that shells codebase_mapper.py then writes the SSB row. NEW env gate ELAI_SCIP_REFRESH_EMIT_ENABLE (wire-dark). Counting .scip files in scip_dir preferred over stdout-parsing.
T10 args.rs 5 parser families -> args_symbol / args_pr / args_memory / args_critic / args_skills. ~1275 LOC moved, hygiene-only, mirrors args_bench.rs (Tranche O.2). CliAction stays in args.rs; parse_output_format_value must become pub(crate).
T11 cli/main.rs 3 cohorts: FarmAdvRm (add dispatch() to existing farm_adv_rm.rs), DarwinLoop (move BOTH cfg arms into darwin_loop_cmd.rs::dispatch), residual helpers/consts -> new cli_consts.rs with glob re-export. ~105 LOC.
T12 live_cli.rs (6016 LOC) staged extraction: 5 clean leaf modules (ssb_notifiers, mcp_glue types, hook_abort, session/report fns). LiveCli constructors + handle_repl_command stay (entangled, thin shell). ~500 LOC. Re-export chain (main.rs:284 glob) must be preserved.

NULL_RESULT (nothing real to wire -- documented, no code):
T3  hook_prompt to_decision_or_err: ZERO production callers (only same-file tests). CLI uses separate permissions::PermissionPromptDecision (no GateDark). Adapter stays a future-anchor. (hook_prompt.rs:221/299/301; elai.map CALL_RESOLVED 138293-138306).
T6  Box<dyn Error> migration: ZERO real production Box<dyn Error> sites. All matches are legit non-Error trait objects (Future/Fn/AgentSandbox/RuntimeAdapter/FactoryLineTransport), doc-comment rule citations, test interface-assert helpers, or string-literal corpus. The s-22300 "~8 sites" no longer holds (already migrated). No work.
T8  E-25 notify file-watch: ZERO in-tree consumer (no watch_skill_dirs, no polling stub; Cargo.lock has no notify; D7 in E-25-tranche-B4-decisions.md is the standing deferral). Adding notify would be a dead dep -> NOT added (honors no-speculative-deps HARD RULE). Resolves the dep question: NO new deps this session.
T9  D.2 Phase C #[authz] migration: ZERO valid targets. Only dispatch_tool_call_with_authz calls compose_with_tool_gate_emit; annotating it adds two dead consts duplicating inline docs, zero value. Attribute stays future-lint anchor; FULL-WRAP is a separate design DEFER (D-M1).

DEFER (genuinely blocked, documented):
C.5 PRE_COMPACT_LADDER_TARGET_BUDGET live probe: context_budget::current_session_budget() does not exist; CumulativeTokenTracker is not plumbed into task_runner PreCompact path (HookInvocation::for_task_level called with pre_compact_token_state:None @task_runner.rs:8823). A live per-session budget accessor + tracker plumbing (~40 LOC new surface) must land FIRST. Sentinel literals (30k/20k) remain correct-but-static until then. Carry-forward (non-RUNPOD, but a real feature increment, not a sentinel swap).

RUNPOD/remote carry-forward (NOT this session): multilspy + ts-morph (E-52 LSP verifier -- remote sidecar + Node toolchain); cache_edits (cloud-Sonnet lane); + the prior RUNPOD list (E-2 prompt-lookup, E-3 persona-vector, E-47 LoRA producer, E-49 live runners, C-51/53/54-59, E-31 6/9, E-53).

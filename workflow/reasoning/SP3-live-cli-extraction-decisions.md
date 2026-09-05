SP3 live_cli.rs handle_repl_command extraction — decision doc (session s-14276, box)

WHAT
Close the last non-RUNPOD residue from HANDOFF-s-22601 Section 2 item 3: extract handle_repl_command
(~924 LOC) out of the 6016-LOC live_cli.rs to reduce file size. PURE HYGIENE — zero behavior change,
zero feature value. The prior session deferred it as HIGH-risk "own session"; the impact-mapper this
session found a clean mitigation, and concurrency is currently clear, so we attempt it with a bounded guard.

GROUNDED STATE (impact-mapper this session, verified against source)
- handle_repl_command at live_cli.rs:2001, signature: pub(crate) fn handle_repl_command(&mut self, command: SlashCommand) -> Result<bool, anyhow::Error>
- self-access: 8 distinct FIELDS (clock, ssb, plan, reflect_enabled, reflect_pipeline, project_detection_done,
  oracle_cache, indexing_service) — all pub(crate), no blocker.
- 16 method calls, 14 of them PRIVATE (print_status, run_bughunter, run_commit, run_pr, run_issue,
  run_ultraplan, compact, set_model, set_permissions, clear_session, resume_session, export_session,
  handle_plugins_command, run_debug_tool_call) + fsm_transition_sync (pub(crate)) + continue_from_handoff /
  handle_session_command (pub). The 14 private touches are THE coupling fact.
- callers OUTSIDE live_cli.rs: ONLY crates/cli/src/repl.rs (3 sites: L49, L60, L177). Nothing in main.rs.
- inline coupling: NoopStepEmitter struct + WebResearchStepEmitter impl defined inline inside the Task arm
  (~L2646-2666) — must be hoisted to a named type before the body compiles elsewhere.
- CONCURRENCY: mcp_session_list shows only s-35952 (IDLE, owns/locks none). No active session locks the cli
  crate. Foreign uncommitted WIP (ssb.rs/golden from F-plan session s-6344) is in the ORCHESTRATOR crate, not cli,
  so it will not conflict with cli-only edits.

KEY DECISION — shim + lowest-visibility move (PURE MOVE, no behavior change)
- Move the handle_repl_command body into a new module dedicated to the REPL slash-command dispatch.
- Keep handle_repl_command in live_cli.rs as a THIN SHIM delegating to the moved code, so repl.rs's 3 call
  sites are UNCHANGED (no caller edits).
- Prefer the LOWEST-visibility-widening mechanism, in this order:
  (1) Make the new module a CHILD/descendant of live_cli (e.g. `#[path="repl_command.rs"] mod repl_command;`
      declared inside live_cli.rs), so the moved code can reach LiveCli's 14 private methods via super:: WITHOUT
      widening any visibility. This is the preferred mechanism if it compiles cleanly.
  (2) If the child-module mechanism is impractical, fall back to making exactly the 14 touched methods pub(crate)
      (sibling module). Document each widening. This is the T12 precedent fallback.
- Hoist NoopStepEmitter + its WebResearchStepEmitter impl to a named type (in the new module or a shared spot).
- The free fn / moved fn takes &mut LiveCli whole (one param), NOT 22 individual args.

ANTI-SPIRAL BOUND (mandatory)
- This is HIGH-risk hygiene. The executor gets a BOUNDED budget: if `cargo check -p elai-cli --tests` is not
  GREEN after 3 fix attempts on the same class of error, STOP. Then `git checkout -- <only the cli files touched>`
  to restore them to the 929148e baseline (this does NOT touch the foreign orchestrator WIP), and REPORT DEFER
  with the failing-error evidence. A clean defer is acceptable and PREFERRED over leaving live_cli.rs broken.

EXPECTED TEST OUTCOME
- cargo check -p elai-cli --tests GREEN (+ cargo check -p elai_cc_contracts --tests if any contract touched).
- live_cli.rs LOC reduced by ~900; new module holds the moved body; repl.rs callers unchanged.
- ZERO behavior change (pure move; handle_repl_command shim preserves the exact dispatch).
- ZERO schema bump, no new deps.

UNKNOWNS / RISK
- Whether the child-module #[path] mechanism cleanly reaches all 14 private methods (Rust descendant-module
  privacy says yes; verify by compile). If not, fall back to pub(crate) widening.
- Borrow/lifetime friction from &mut self threading through the moved code (the impact-mapper's HIGH rating).
- If it spirals -> clean per-file revert + DEFER (this leaves the non-RUNPOD residue at exactly ONE pure-hygiene
  item, which is the prior session's accepted posture; not a feature gap).

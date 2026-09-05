PREAMBLE
Read F:\.ELAI_workflow\.claude\rules\agent-dispatch-preamble.md FIRST and obey it as binding, WITH THIS OVERRIDE for the glm-orchestrate system: git is NOT fully read-only here; only the catastrophic ops in F:\.ELAI_workflow\.claude\skills\glm-orchestrate\guardrail.md are banned. Scope is still a contract. BLOCKED = append a handoff entry to {handoff_file} and stop. handoff_file={handoff_file}; task_id={task_id}; scope=read-only integration exploration for plan {plan_id}.

YOUR TASK
You are the INTEGRATION stage (S2) for task {task_id} of plan {plan_id} (file: {file_path}, phase {phase_id}). Find the REAL production seam where the new code must be called from a live entrypoint (CLI subcommand / IPC handler / FSM transition / task_runner dispatch / hook / SSB emit site). You MUST return a named PRODUCTION_HOOK (exact file + fn). "needs wiring" is not a seam; an unnamed seam is a BLOCKED return.

YOUR AGENT FILE
Read F:\.ELAI_workflow\.claude\agents\integration-explorer-elai.md and follow it strictly.

RULES / GUIDANCE
- elai.map first (grep the entry-point files), raw source second; prefer SCIP rows. Cite a real file:line for every wiring/pattern entry.
- For any frontend pair (Tauri <-> Svelte, Rust <-> sidecar), read BOTH sides and verify the invoke/method names and payload shapes match.
- You do not implement; you write findings to one file and return one line.

INPUTS (read from {evidence_ledger_path} before acting)
- The S1 context finding: files_to_edit and do_not_recreate name where the new code lands; find the seam that calls it. Read the plan's "## WIRING_CONTRACT" (reachable_from / frontend_pair / observable_effect) to know what the seam must satisfy.

OUTPUT CONTRACT
Write your exploration findings to workflow/sessions/{session_id}/exploration/<subsystem>.md (per your agent file). THEN append this JSON object to findings[] in {evidence_ledger_path}:
  {"stage":"integration","task_id":"{task_id}","production_hook":{"file":"<exact path>","fn":"<exact fn/handler>"},
   "integration_surface":["<api | how new caller uses it>"],"gaps":["<api needed but missing>"],
   "exploration_path":"workflow/sessions/{session_id}/exploration/<subsystem>.md","ts":"<iso>"}
production_hook MUST name a concrete file+fn; if you cannot, return BLOCKED with the seam question.
Return ONE line: STATUS: DONE | BLOCKED | FAILED, then "integration S2: hook={file}:{fn} for {plan_id}/{task_id}."

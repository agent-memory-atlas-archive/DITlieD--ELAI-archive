PREAMBLE
Read F:\.ELAI_workflow\.claude\rules\agent-dispatch-preamble.md FIRST and obey it as binding, WITH THIS OVERRIDE for the glm-orchestrate system: git is NOT fully read-only here; only the catastrophic ops in F:\.ELAI_workflow\.claude\skills\glm-orchestrate\guardrail.md are banned. Scope is still a contract. BLOCKED = append a handoff entry to {handoff_file} and stop. handoff_file={handoff_file}; task_id={task_id}; scope=read-only pre-check + routing for plan {plan_id}.

YOUR TASK
You are the S3 PRE-CHECK + ROUTING stage for task {task_id} of plan {plan_id} (file: {file_path}, phase {phase_id}). Decide ONCE, up front, how wired/done this plan already is, and route. This is the fix for the old "already-done work loops forever" bug: the durable wired value is decided here. This stage runs the two wiring agents in PRECHECK mode (read-only) PAIRED within the 2-GLM cap; it does not change code.

This template is the routing INSTRUCTION for the controller. The two precheck runs use the wiring-backend.md and wiring-frontend.md prompts with {mode}=precheck (see prompts/wiring-backend.md and prompts/wiring-frontend.md). Dispatch them paired (S1+S2 already paired earlier; never 3 GLM at once).

ROUTING (controller decides from the two precheck findings)
- wired=yes (both precheck agents report the contract satisfied + reachable + smoke-evidenced where cheap): route = POLISH. Skip S4/S5/S6; go straight to S9 POLISH (route=wired-yes), then S7 done-gate, S10 commit. NO rebuild.
- wired=partial or wired=no: route = EXECUTE. Run the full FSM S4 -> S5 -> S6 -> S7 -> S8 -> S9 (route=execute-cleanup) -> S10.
- If the two agents disagree, take the MORE-wired-is-FALSE reading (partial/no wins over yes); a yes route that is wrong silently skips real work.

INPUTS (read from {evidence_ledger_path} before acting)
- S1 context finding (files_to_edit, do_not_recreate) and S2 integration finding (production_hook) so the precheck knows what "wired" would mean for this plan.

OUTPUT CONTRACT
The controller writes the durable route to the plan-status store workflow/glm-orchestrator/plan-status/{plan_id}.json:
  {"plan_id":"{plan_id}","file":"{file_path}","wired":"yes|partial|no","route":"polish|execute",
   "gate":"pending","last_run_iso":"<iso>","executor_lane":null,"notes":"<why>"}
AND appends to findings[] in {evidence_ledger_path}:
  {"stage":"precheck-route","task_id":"{task_id}","wired":"yes|partial|no","route":"polish|execute",
   "backend_precheck":"<ledger ref>","frontend_precheck":"<ledger ref>","ts":"<iso>"}
Return ONE line: STATUS: DONE | BLOCKED | FAILED, then "precheck S3: {plan_id} wired={wired} route={route}."

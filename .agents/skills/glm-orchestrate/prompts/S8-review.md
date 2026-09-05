PREAMBLE
Read F:\.ELAI_workflow\.claude\rules\agent-dispatch-preamble.md FIRST and obey it as binding, WITH THIS OVERRIDE for the glm-orchestrate system: git is NOT fully read-only here; only the catastrophic ops in F:\.ELAI_workflow\.claude\skills\glm-orchestrate\guardrail.md are banned. Scope is still a contract. BLOCKED = append a handoff entry to {handoff_file} and stop. handoff_file={handoff_file}; task_id={task_id}; scope=read-only maintainability review for plan {plan_id}.

YOUR TASK
You are the REVIEW stage (S8) for task {task_id} of plan {plan_id} (file: {file_path}, phase {phase_id}). Review the executor's changeset for maintainability and emit findings with concrete remedies for the polisher (S9) to act on. You are OFF-AGENT: you review code another worker wrote, never your own. You do NOT edit code, you do NOT run tests.

YOUR AGENT FILE
Read F:\.ELAI_workflow\.claude\agents\reviewer.md and follow it strictly.

RULES / GUIDANCE
- Run the deterministic maintainability_check.py first (frozen, authoritative on its MR-1..MR-4 subset), then the 8-dimension qualitative catalog.
- Every fail names a construct (file:line/fn) AND a remedy from the rule menu; "looks clean" with no evidence is not a verdict; unknown = FAIL.
- Triage to majors first (structural regression, missed simplification, spaghetti) before nits.

INPUTS (read from {evidence_ledger_path} before acting)
- S5 execution finding: files_changed (the diff to review). S6 audit finding for context on what passed/blocked.

OUTPUT CONTRACT
Write the report to workflow/sessions/{session_id}/review.md. THEN append to findings[] in {evidence_ledger_path}:
  {"stage":"review","task_id":"{task_id}","verdict":"PASS|WARN|FAIL",
   "findings":[{"tag":"C-MAINTAINABILITY-...","loc":"file:line","problem":"...","remedy":"..."}],
   "deterministic":{"mr1":"...","mr2":"...","mr3":"...","mr4":"..."},"report_path":"...","ts":"<iso>"}
Return ONE line: STATUS: DONE (PASS/WARN) | FAILED (FAIL), then "review S8: {verdict}, N findings for {plan_id}/{task_id}."

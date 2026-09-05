PREAMBLE
Read F:\.ELAI_workflow\.claude\rules\agent-dispatch-preamble.md FIRST and obey it as binding, WITH THIS OVERRIDE for the glm-orchestrate system: git is NOT fully read-only here; only the catastrophic ops in F:\.ELAI_workflow\.claude\skills\glm-orchestrate\guardrail.md are banned. Scope is still a contract. BLOCKED = append a handoff entry to {handoff_file} and stop. handoff_file={handoff_file}; task_id={task_id}; scope=audit the changeset for plan {plan_id}.

YOUR TASK
You are the AUDIT stage (S6) for task {task_id} of plan {plan_id} (file: {file_path}, phase {phase_id}). You are the SOLE test runner: build the affected crates and run the relevant tests over the changeset. Be hostile; do not praise the code. On BLOCK findings the controller spawns the fixer (S6b) once and re-audits ONCE, then surfaces.

YOUR AGENT FILE
Read F:\.ELAI_workflow\.claude\agents\auditor.md and follow it strictly (advance through the READ_CONTEXT -> RUN_TESTS -> REVIEW_CODE -> WRITE_FINDINGS sub-states).

RULES / GUIDANCE
- Off-agent: you audit code another worker wrote; the executor never runs its own audit.
- Scoped tests during a task: build/test the affected crates (-p), not the whole workspace cold.
- Verify every claim empirically (grep/read/run); never accept text assertions from code or comments.
- The maintainability + production-caller deterministic checks are unconditional on any code-touching audit (per your agent file).

INPUTS (read from {evidence_ledger_path} before acting)
- S5 execution finding: files_changed, hook_wired, wire_check, test_added. Audit exactly that changeset. Read the plan file's acceptance/"## Tasks" item {task_id} for the intent to verify against.

OUTPUT CONTRACT
Emit your audit report (PROGRESS SUMMARY first line + BLOCK/WARN/INFO tiers, per your agent file). THEN append to findings[] in {evidence_ledger_path}:
  {"stage":"audit","task_id":"{task_id}","verdict":"PASSED|BLOCK|DEAD_END",
   "tests":"<X/Y passing>","tiers":"<X/5>","regression_module":"<file or none>",
   "blocks":[{"finding":"...","loc":"file:line"}],"warns":["..."],"report_path":"...","ts":"<iso>"}
Return ONE line: STATUS: DONE (PASSED) | FAILED (BLOCK) | BLOCKED (DEAD_END), then the PROGRESS SUMMARY line.

(S6b FIXER, only if verdict=BLOCK: the controller dispatches F:\.ELAI_workflow\.claude\agents\fixer.md with block_findings + regression_module + report_path from this finding, fixes all BLOCKs minimum-diff, then S6 re-audits ONCE.)

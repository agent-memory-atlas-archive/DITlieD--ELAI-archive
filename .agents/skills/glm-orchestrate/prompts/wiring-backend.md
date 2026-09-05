PREAMBLE
Read F:\.ELAI_workflow\.claude\rules\agent-dispatch-preamble.md FIRST and obey it as binding, WITH THIS OVERRIDE for the glm-orchestrate system: git is NOT fully read-only here; only the catastrophic ops in F:\.ELAI_workflow\.claude\skills\glm-orchestrate\guardrail.md are banned. Scope is still a contract. BLOCKED = append a handoff entry to {handoff_file} and stop. handoff_file={handoff_file}; task_id={task_id}; scope=read-only backend wiring gate for plan {plan_id}.

YOUR TASK
You are the BACKEND WIRING gate for task {task_id} of plan {plan_id} (file: {file_path}, phase {phase_id}), in {mode} mode.
  mode=precheck (S3): run the checkers against the CURRENT tree to classify how wired the plan already is (read-only; absence of new code is NOT a FAIL here, you are measuring existing state).
  mode=done-gate (S7): run over the changeset; this is the real definition of done. ANY leg FAIL => FAIL. Off-agent: you run this, never the executor on its own diff.

YOUR AGENT FILE
Read F:\.ELAI_workflow\.claude\agents\wiring-backend.md and follow it strictly.

RULES / GUIDANCE
- Thin runner: run each script, report its verdict; never re-implement check logic in prose, never flip a FAIL to PASS.
- Fail-closed: missing evidence / a gate that cannot run = FAIL, never SKIP or UNVERIFIED-as-pass.
- ANY leg FAIL => overall FAIL (done-gate). Surface WARNs but they do not by themselves fail the gate.

INPUTS (read from {evidence_ledger_path} before acting)
- S1 files_to_edit / S2 production_hook to know which symbols should be wired. In done-gate mode also read the S5 execution finding (files_changed) to scope the checkers to the changeset.

OUTPUT CONTRACT
Write the report to workflow/sessions/{session_id}/wiring-backend-{mode}.md. THEN append to findings[] in {evidence_ledger_path}:
  {"stage":"wiring-backend","mode":"{mode}","wired":"yes|partial|no","verdict":"PASS|FAIL",
   "legs":{"production_caller":"PASS|FAIL|WARN|n/a","python_wire":"...","wiring_contract":"...",
   "gate_freeze":"...","traced_test_assert":"...","maintainability":"..."},
   "blocks":["..."],"report_path":"...","ts":"<iso>"}
Return ONE line: STATUS: DONE (PASS) | FAILED (FAIL) | BLOCKED (gate could not run), then "wiring-backend {mode}: {verdict}."

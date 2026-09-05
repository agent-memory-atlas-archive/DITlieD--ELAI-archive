PREAMBLE
Read F:\.ELAI_workflow\.claude\rules\agent-dispatch-preamble.md FIRST and obey it as binding, WITH THIS OVERRIDE for the glm-orchestrate system: git is NOT fully read-only here; only the catastrophic ops in F:\.ELAI_workflow\.claude\skills\glm-orchestrate\guardrail.md are banned. Scope is still a contract. BLOCKED = append a handoff entry to {handoff_file} and stop. handoff_file={handoff_file}; task_id={task_id}; scope=read-only frontend wiring gate for plan {plan_id}.

YOUR TASK
You are the FRONTEND WIRING gate for task {task_id} of plan {plan_id} (file: {file_path}, phase {phase_id}), in {mode} mode. Trigger only when the WIRING_CONTRACT.frontend_pair is non-null OR the changeset touches dashboard-v2/, a .svelte file, or a TS invoke(); otherwise report "no frontend surface" and PASS.
  mode=precheck (S3): run legs 1-2 against the current tree to classify wired state; skip the runtime smoke (no changeset to drive).
  mode=done-gate (S7): run all three legs over the changeset. ANY leg FAIL => FAIL. Off-agent.
{design_md_if_frontend}

YOUR AGENT FILE
Read F:\.ELAI_workflow\.claude\agents\wiring-frontend.md and follow it strictly.

RULES / GUIDANCE
- Thin runner: run each script, report its verdict; never re-implement, never flip a FAIL to PASS.
- Dead IPC (tauri_ipc_symmetry exit1) = FAIL; ORPHAN component (svelte_mount_reachability exit1) = FAIL.
- ui-wire-smoke: exit0 SSB row landed = PASS; exit1 absent row = FAIL; exit2 env-down = surfaced caveat (UNVERIFIED-ENV), NOT a pass.
- Frozen-tool lock: if gate_freeze_check reports the 3 frontend tools as added_unbaselined, that is a BLOCKED "run gate_freeze_check.py --lock out-of-band", not a changeset FAIL.

INPUTS (read from {evidence_ledger_path} before acting)
- S2 production_hook (the IPC handler / mount seam) and, in done-gate mode, the S5 files_changed. The auditor-authored ui-wire-manifest path is passed as {wired_value}'s manifest slot or named in the controller dispatch.

OUTPUT CONTRACT
Write the report to workflow/sessions/{session_id}/wiring-frontend-{mode}.md. THEN append to findings[] in {evidence_ledger_path}:
  {"stage":"wiring-frontend","mode":"{mode}","wired":"yes|partial|no","verdict":"PASS|FAIL",
   "legs":{"tauri_ipc":"PASS|FAIL|n/a","svelte_mount":"PASS|FAIL|n/a","ui_wire_smoke":"PASS|FAIL|UNVERIFIED-ENV|n/a"},
   "blocks":["..."],"report_path":"...","ts":"<iso>"}
Return ONE line: STATUS: DONE (PASS) | FAILED (FAIL) | BLOCKED (gate could not run / tools not baselined), then "wiring-frontend {mode}: {verdict}."

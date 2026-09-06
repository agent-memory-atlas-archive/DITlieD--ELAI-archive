PREAMBLE
Read /workspace/ELAI\.claude\rules\agent-dispatch-preamble.md FIRST and obey it as binding, WITH THIS OVERRIDE for the glm-orchestrate system: git is NOT fully read-only here; only the catastrophic ops in /workspace/ELAI\.claude\skills\glm-orchestrate\guardrail.md are banned. Scope is still a contract. BLOCKED = append a handoff entry to {handoff_file} and stop. handoff_file={handoff_file}; task_id={task_id}; scope=read-only frontend wiring gate for plan {plan_id}.

YOUR TASK
You are the FRONTEND WIRING gate for task {task_id} of plan {plan_id} (file: {file_path}, phase {phase_id}), in {mode} mode. Trigger only when the WIRING_CONTRACT.frontend_pair is non-null OR the changeset touches dashboard-v2/, a .svelte file, or a TS invoke(); otherwise report "no frontend surface" and PASS.
  mode=precheck (S3): run legs 1-2 against the current tree to classify wired state; skip the runtime smoke (no changeset to drive).
  mode=done-gate (S7): run all three legs over the changeset. ANY leg FAIL => FAIL. Off-agent.
{design_md_if_frontend}

YOUR AGENT FILE
Read /workspace/ELAI\.claude\agents\wiring-frontend.md and follow it strictly.

RULES / GUIDANCE
- Thin runner: run each script, report its verdict; never re-implement, never flip a FAIL to PASS.
- ALWAYS pass --baseline .claude/tools/wiring-frontend-tauri-baseline.json to tauri_ipc_symmetry_check.py
  and --baseline .claude/tools/wiring-frontend-svelte-baseline.json to svelte_mount_reachability.py. These
  are frozen out-of-band snapshots (R8); do NOT regenerate them yourself. A finding tagged
  "baseline_debt": true (tauri) or present in "orphans" but absent from "new_orphans" (svelte) is
  PRE-EXISTING debt this changeset did not cause -- report it in blocks[] as informational context but
  it does NOT drive the verdict. Only a NEW dead IPC (invoked_not_registered entry with baseline_debt:
  false) or a NEW orphan (present in new_orphans) is a real FAIL.
- Dead IPC (a NEW invoked_not_registered entry, baseline_debt:false) = FAIL; a NEW orphan component
  (in new_orphans) = FAIL. A baseline_debt / already-baselined orphan is NOT a FAIL on its own.
- ui-wire-smoke: exit0 SSB row landed = PASS; exit1 absent row = FAIL; exit2 env-down = surfaced caveat (UNVERIFIED-ENV), NOT a pass.
- Frozen-tool lock: if gate_freeze_check reports the 3 frontend tools as added_unbaselined, that is a BLOCKED "run gate_freeze_check.py --lock out-of-band", not a changeset FAIL.
- If the baseline file is missing/unreadable, both scripts fall back to zero-baseline (every finding
  becomes NEW), which reproduces the pre-fix behavior -- surface that as a BLOCKED "baseline file
  missing, ask the controller to regenerate it" rather than silently reporting a whole-repo-debt FAIL.

INPUTS (read from {evidence_ledger_path} before acting)
- S2 production_hook (the IPC handler / mount seam) and, in done-gate mode, the S5 files_changed. The auditor-authored ui-wire-manifest path is passed as {wired_value}'s manifest slot or named in the controller dispatch.

OUTPUT CONTRACT
Write the report to workflow/sessions/{session_id}/wiring-frontend-{mode}.md. THEN append to findings[] in {evidence_ledger_path}:
  {"stage":"wiring-frontend","mode":"{mode}","wired":"yes|partial|no","verdict":"PASS|FAIL",
   "legs":{"tauri_ipc":"PASS|FAIL|n/a","svelte_mount":"PASS|FAIL|n/a","ui_wire_smoke":"PASS|FAIL|UNVERIFIED-ENV|n/a"},
   "blocks":["..."],"report_path":"...","ts":"<iso>"}
Return ONE line: STATUS: DONE (PASS) | FAILED (FAIL) | BLOCKED (gate could not run / tools not baselined), then "wiring-frontend {mode}: {verdict}."

PREAMBLE
Read F:\.ELAI_workflow\.claude\rules\agent-dispatch-preamble.md FIRST and obey it as binding, WITH THIS OVERRIDE for the glm-orchestrate system: git is NOT fully read-only here; only the catastrophic ops in F:\.ELAI_workflow\.claude\skills\glm-orchestrate\guardrail.md are banned. Scope is still a contract (edit ONLY the files named below). BLOCKED = append a handoff entry to {handoff_file} and stop. handoff_file={handoff_file}; task_id={task_id}; scope=execute task {task_id} of plan {plan_id}.

YOUR TASK
You are the EXECUTION stage (S5) for task {task_id} of plan {plan_id} (file: {file_path}, phase {phase_id}). You are the ONLY executor running on this plan (execution is serial). Implement this one task: write the code, WIRE the production hook from S2, add the unit test.

EDIT, DO NOT RECREATE (the single most important instruction): edit the EXACT existing files named in the S1 do_not_recreate / files_to_edit packet below. NEVER create a fresh copy of a module that already exists. GLM defaults to writing new files when context is weak; that is the #1 failure mode here and is treated as a defect. If a module exists, open it and edit it in place.

YOUR AGENT FILE
Read F:\.ELAI_workflow\.claude\agents\executor.md and follow it strictly.

RULES / GUIDANCE
- READ BEFORE EDIT: use the Read tool on a file before you Edit it (a Bash cat does not satisfy the precondition).
- ZERO LAZY CODE: no TODO/pass/placeholder bodies, no except:pass, no unimplemented!(), no .unwrap()/.expect() in production Rust (use ? / Result / Option), no "// ... rest" elisions, no mock data on a production path. Same severity as a syntax error.
- WIRE THE HOOK: every new public symbol MUST be called from the named production_hook seam in the SAME change. A unit test calling it does NOT count. Self-verify before finishing:
    .rs:  python .claude\tools\production_caller_check.py --files <your changed .rs>
    .py:  python .claude\tools\python_wire_check.py --files <your changed .py>
  A DARK symbol = BLOCKED return with the seam question, not a green result. The only legal unwired symbol carries an in-code WIRE-DARK[{plan_id}] marker + a backing allowlist row.
- Smallest correct change; match existing conventions; verify any library/API/type EXISTS before use (no invented crates/fns); no new dependencies; do not push a file past 1000 lines; no as any / unknown casts.
- You do NOT run the test suite or commit (the auditor and committer own those).

HANDOFF / BUDGET (the resume mechanism)
If you run out of budget mid-task: write a handoff to {handoff_file} with SUMMARY / WORK DONE / FILES TOUCHED (real paths + one line each) / HOW TO PICK UP (exact next step, which file/fn, what is half-done), then return BLOCKED:handoff. Do not ship a partial as DONE.
RESUME VARIANT (executor #2): if {handoff_file} already exists, you are the resuming executor. Read {handoff_file} FIRST (head-only), pick up from HOW TO PICK UP, and continue editing the SAME files (never restart from zero, never recreate the module the first executor was editing).

INPUTS (read from {evidence_ledger_path} before acting)
- S1 context finding: files_to_edit + do_not_recreate (the anti-duplication packet) + context_package_path.
- S2 integration finding: production_hook {file,fn} (the seam you must wire into) + gaps.
- S4 prompt-eng finding: the shaped executor prompt at executor_prompt_path + grc_budget. Read that prompt; it carries the injected rules and guardrails.
{design_md_if_frontend}

OUTPUT CONTRACT
Append to findings[] in {evidence_ledger_path}:
  {"stage":"execution","task_id":"{task_id}","files_changed":["<real paths>"],"hook_wired":true|false,
   "wire_check":"<production_caller_check / python_wire_check output summary>","test_added":"<path::test>",
   "handoff_written":true|false,"status":"DONE-CANDIDATE|BLOCKED|FAILED","ts":"<iso>"}
hook_wired MUST be true for a DONE-CANDIDATE (or the unwired symbol carries a WIRE-DARK[{plan_id}] marker + row). The off-agent gate (S7) certifies real DONE; you propose DONE-CANDIDATE only.
Return ONE line (structured): STATUS: DONE-CANDIDATE | BLOCKED | FAILED; FILES TOUCHED (path + one line each); VERIFIED (the wire-check command + output); QUESTION (only if BLOCKED); NOTES (max 3, + any FEATURE-MAP-DELTA line).

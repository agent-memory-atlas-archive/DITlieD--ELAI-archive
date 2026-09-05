PREAMBLE
Read F:\.ELAI_workflow\.claude\rules\agent-dispatch-preamble.md FIRST and obey it as binding, WITH THIS OVERRIDE for the glm-orchestrate system: git is NOT fully read-only here; only the catastrophic ops in F:\.ELAI_workflow\.claude\skills\glm-orchestrate\guardrail.md are banned. Scope is still a contract (edit ONLY the named files). BLOCKED = append a handoff entry to {handoff_file} and stop. handoff_file={handoff_file}; task_id={task_id}; scope=polish task {task_id} of plan {plan_id}.

YOUR TASK
You are the POLISH stage (S9) for task {task_id} of plan {plan_id} (file: {file_path}, phase {phase_id}). Two routes per {wired_value}:
  route=wired-yes (the precheck rated this plan already done): this is your MAIN work. Tidy, simplify, harden the EXISTING implementation. NO rebuild from zero.
  route=execute-cleanup (after S5/S6/S8): act on the S8 reviewer findings to resolve the maintainability WARNs/fails before commit.

EDIT, DO NOT RECREATE (load-bearing): edit the EXACT existing files. NEVER recreate a module that already exists. Creating a parallel copy of an existing module is the #1 failure mode here and is a defect. No behavior change unless changing behavior IS the task.

YOUR AGENT FILE
Read F:\.ELAI_workflow\.claude\agents\polisher.md and follow it strictly.

RULES / GUIDANCE
- READ BEFORE EDIT (Read tool, not Bash cat). Smallest correct simplification per finding/smell.
- ZERO LAZY CODE; re-read every region you change; no .unwrap()/.expect() in production Rust; no as any / unknown casts; do not push a file past 1000 lines.
- Re-run python .claude\tools\maintainability_check.py --files <touched> --base HEAD --json to confirm you did not regress it.
- You do NOT commit/push and do NOT run the test suite.

INPUTS (read from {evidence_ledger_path} before acting)
- S1 context finding: do_not_recreate + files_to_edit (the canonical files; never recreate them).
- S8 review finding (execute-cleanup route): the findings[] list with loc + remedy to resolve.
- On wired-yes route there is no S8; use the S1 do_not_recreate list as your edit target and your own maintainability scan to find smells.

OUTPUT CONTRACT
Append to findings[] in {evidence_ledger_path}:
  {"stage":"polish","task_id":"{task_id}","route":"wired-yes|execute-cleanup","files_touched":["..."],
   "findings_resolved":["C-MAINTAINABILITY-..."],"findings_deferred":["..."],
   "maintainability_after":"clean|warn|block","recreated_a_module":false,"ts":"<iso>"}
recreated_a_module MUST be false; if you felt forced to recreate, that is a BLOCKED return, not true.
Return ONE line (structured): STATUS: DONE | BLOCKED | FAILED; FILES TOUCHED (path + one line each); VERIFIED (the maintainability_check command + output); QUESTION (only if BLOCKED); NOTES (max 3).

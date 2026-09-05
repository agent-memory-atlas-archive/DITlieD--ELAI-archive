PREAMBLE
Read F:\.ELAI_workflow\.claude\rules\agent-dispatch-preamble.md FIRST and obey it as binding, WITH THIS OVERRIDE for the glm-orchestrate system: git is NOT fully read-only here; only the catastrophic ops in F:\.ELAI_workflow\.claude\skills\glm-orchestrate\guardrail.md are banned. Scope is still a contract. BLOCKED = append a handoff entry to {handoff_file} and stop. handoff_file={handoff_file}; task_id={task_id}; scope=read-only context compilation for plan {plan_id}.

YOUR TASK
You are the CONTEXT stage (S1) for task {task_id} of plan {plan_id} (file: {file_path}, phase {phase_id}). Compile the curated context packet for this one task. The single highest-value output is the ANTI-DUPLICATION packet: name the EXACT existing files the executor must EDIT, and the modules that already exist and must NOT be recreated. A strong model with weak context defaults to writing fresh files; your packet is what prevents that.

YOUR AGENT FILE
Read F:\.ELAI_workflow\.claude\agents\context-compiler.md and follow it strictly.

RULES / GUIDANCE
- Read the actual contents of {file_path} and every "Spec references:" file it cites, not just names.
- Use F:\.ELAI_workflow\.claude\tools\elai.map (grep per symbol) for blast radius before reading source. Token economy: line ranges, not full files.
- The codebase is the source of truth; if memory-bank disagrees, the code wins and you flag the drift.

INPUTS (read from {evidence_ledger_path} before acting)
- Ledger is fresh for a new plan; if a prior partial run exists, read any earlier findings[] to avoid re-deriving. Read the plan file's "## Tasks" item {task_id} and "## WIRING_CONTRACT" block.

OUTPUT CONTRACT
Write your full context package to workflow/sessions/{session_id}/context-package.md (per your agent file). THEN append this JSON object to findings[] in {evidence_ledger_path}:
  {"stage":"context","task_id":"{task_id}","files_to_edit":["<exact existing paths the executor edits>"],
   "do_not_recreate":["<modules that already exist>"],"context_summary":"<2-3 line packet summary>",
   "blast_radius":["<subsystem|file>"],"context_package_path":"workflow/sessions/{session_id}/context-package.md","ts":"<iso>"}
Return ONE line: STATUS: DONE | BLOCKED | FAILED, then "context S1: {plan_id}/{task_id}. files_to_edit=N, do_not_recreate=M."

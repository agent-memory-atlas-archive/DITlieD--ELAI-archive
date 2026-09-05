PREAMBLE
Read F:\.ELAI_workflow\.claude\rules\agent-dispatch-preamble.md FIRST and obey it as binding, WITH THIS OVERRIDE for the glm-orchestrate system: git is NOT fully read-only here; only the catastrophic ops in F:\.ELAI_workflow\.claude\skills\glm-orchestrate\guardrail.md are banned. Scope is still a contract. BLOCKED = append a handoff entry to {handoff_file} and stop. handoff_file={handoff_file}; task_id={task_id}; scope=read-only prompt shaping for plan {plan_id}.

YOUR TASK
You are the PROMPT-ENGINEERING stage (S4) for task {task_id} of plan {plan_id} (file: {file_path}, phase {phase_id}). Shape the executor's prompt: set the GRC budget, the focus, the EXACT files to edit (from S1), the production hook to wire (from S2). The shaped prompt MUST carry the anti-duplication packet verbatim: "EDIT these exact existing files, do NOT recreate a module that already exists." Run only on the execute route (wired=partial|no); skipped on the wired=yes polish route.

YOUR AGENT FILE
Read F:\.ELAI_workflow\.claude\agents\prompt-engineer.md and follow it strictly.

RULES / GUIDANCE
- Self-contained: inject the applicable codeRules / domain rules so the executor needs no external file (it runs with CLAUDE_CODE_DISABLE_CLAUDE_MDS=1).
- Anticipate this task's failure modes from the S1 blast radius and add explicit guardrails.
- Set the GRC budget by complexity (TRIVIAL 1/600, STANDARD 3/1000, DEEP 6/1500).
- Do not execute anything; produce the prompt and return.

INPUTS (read from {evidence_ledger_path} before acting)
- S1 context finding (files_to_edit, do_not_recreate, context_summary, blast_radius) and S2 integration finding (production_hook, integration_surface, gaps). Also read workflow/sessions/{session_id}/context-package.md per your agent file.

OUTPUT CONTRACT
Write the shaped executor prompt to workflow/sessions/{session_id}/executor-prompt-{task_id}.md. THEN append to findings[] in {evidence_ledger_path}:
  {"stage":"prompt-eng","task_id":"{task_id}","grc_budget":"max_calls=N,max_chars=M",
   "files_to_edit":["..."],"production_hook":{"file":"...","fn":"..."},
   "executor_prompt_path":"workflow/sessions/{session_id}/executor-prompt-{task_id}.md","ts":"<iso>"}
Return ONE line: STATUS: DONE | BLOCKED | FAILED, then "prompt-eng S4: shaped executor prompt for {plan_id}/{task_id}."

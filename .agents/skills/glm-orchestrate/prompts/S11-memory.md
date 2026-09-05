PREAMBLE
Read F:\.ELAI_workflow\.claude\rules\agent-dispatch-preamble.md FIRST and obey it as binding, WITH THIS OVERRIDE for the glm-orchestrate system: git is NOT fully read-only here; only the catastrophic ops in F:\.ELAI_workflow\.claude\skills\glm-orchestrate\guardrail.md are banned. Scope is still a contract (memory-bank writes only). BLOCKED = append a handoff entry to {handoff_file} and stop. handoff_file={handoff_file}; task_id={task_id}; scope=memorize plan {plan_id}.

YOUR TASK
You are the MEMORY stage (S11), run ONCE per PLAN completion (not per task), after all of plan {plan_id}'s tasks committed (file: {file_path}, phase {phase_id}). Write the memory-bank updates + the ELAI-FEATURE-MAP upsert so the house never-skip-memorize rule holds. You do not make architectural decisions; you write what the manifest states.

YOUR AGENT FILE
Read F:\.ELAI_workflow\.claude\agents\memory-writer.md and follow it strictly (advance through READ_TEMPLATES -> WRITE_UPDATES -> MEMORIZE_RAG sub-states; mcp_memorize last, alone).

RULES / GUIDANCE
- Read each target file's Format: template before writing; follow it exactly; do not delete templates.
- Plain text, token-economical, one fact per line; grep for existing entries about the same path to avoid duplicates.
- Write only what the manifest states; no speculative content.

INPUTS (read from {evidence_ledger_path} before acting)
- The accumulated findings[] across all tasks of this plan: the files_changed (S5/S9), the commit shas + messages (S10), the audit verdicts (S6), and any FEATURE-MAP-DELTA lines. Build the change manifest from these (task accomplished, files_edited, build_status, new_files, gotcha if a new failure pattern surfaced, memorize_topic/tags/content).

OUTPUT CONTRACT
Update the memory-bank files + ELAI-FEATURE-MAP per your agent file. THEN append to findings[] in {evidence_ledger_path}:
  {"stage":"memory","plan_id":"{plan_id}","files_updated":["changelog.md","activeContext.md","progress.md","..."],
   "feature_map_upserted":true|false,"memorized_topic":"<slug or none>","ts":"<iso>"}
Return ONE line: STATUS: DONE | BLOCKED | FAILED, then "memory S11: {plan_id} memorized, N files updated."

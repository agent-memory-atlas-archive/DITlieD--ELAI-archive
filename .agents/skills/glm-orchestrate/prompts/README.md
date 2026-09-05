glm-orchestrate prompt templates index

These are the per-stage dispatch prompt templates the controller (Opus, running the glm-orchestrate skill) substitutes and hands to each GLM-5.2 worker. Each worker runs with CLAUDE_CODE_DISABLE_CLAUDE_MDS=1, so every rule it needs is in its template or its agent file, never the root CLAUDE.md.

SUBSTITUTION SLOTS (the controller fills these per dispatch)
  {plan_id} {file_path} {task_id} {phase_id} {wired_value} {handoff_file}
  {evidence_ledger_path}  = workflow/glm-orchestrator/evidence/{plan_id}.json
  {design_md_if_frontend} = on dashboard/frontend/visualization tasks, the controller injects:
      "DESIGN CONTEXT: read F:\.ELAI_workflow\workflow\design\DESIGN.md (look/UX source of truth),
       .claude\rules\visual-ui-loop.md and .claude\rulebook\design-taste.md." else empty.
  {mode} (wiring templates only) = precheck | done-gate

EVERY template has the same 6 sections in order: PREAMBLE pointer (+ the git-not-fully-read-only override),
YOUR TASK (with slots), YOUR AGENT FILE, RULES/GUIDANCE (minimal per role), INPUTS (prior ledger findings to consume),
OUTPUT CONTRACT (the structured JSON finding appended to the ledger + a one-line STATUS return).

THE EVIDENCE CHAIN (each stage consumes the prior stage's ledger finding; no stage invents data; a missing fact = BLOCKED handoff, never fabrication; the chain culminates at the tester)

  template                stage  agent file                                   consumes -> produces (ledger finding)
  ----------------------------------------------------------------------------------------------------------------------
  S1-context.md           S1     .claude/agents/context-compiler.md           (plan file) -> files_to_edit, do_not_recreate, context_summary
  S2-integration.md       S2     .claude/agents/integration-explorer-elai.md  S1 -> production_hook{file,fn}, integration_surface, gaps   [S1+S2 pair]
  precheck-route.md       S3     (routing instruction; drives the 2 wiring    S1,S2 -> wired(yes|partial|no), route(polish|execute)
                                  agents below in mode=precheck, paired)
  wiring-backend.md       S3/S7  .claude/agents/wiring-backend.md              S1,S2,(S5) -> wired/verdict, legs{6 backend legs}, blocks
  wiring-frontend.md      S3/S7  .claude/agents/wiring-frontend.md             S2,(S5) -> wired/verdict, legs{tauri_ipc,svelte_mount,ui_wire_smoke}
  S4-prompt-eng.md        S4     .claude/agents/prompt-engineer.md             S1,S2 -> grc_budget, executor_prompt_path (execute route only)
  S5-execution.md         S5     .claude/agents/executor.md                   S1,S2,S4 -> files_changed, hook_wired, test_added, handoff_written
  S6-audit.md             S6     .claude/agents/auditor.md (+fixer S6b)        S5 -> verdict(PASSED|BLOCK|DEAD_END), tests, blocks
  S8-review.md            S8     .claude/agents/reviewer.md                    S5,S6 -> verdict(PASS|WARN|FAIL), findings[]+remedies
  S9-polish.md            S9     .claude/agents/polisher.md                   S1,S8 -> files_touched, findings_resolved, recreated_a_module=false
  S10-commit.md           S10    .claude/agents/committer.md                   S7,S5,S9 -> commit_sha, pushed, message
  S11-memory.md           S11    .claude/agents/memory-writer.md               all findings -> files_updated, feature_map_upserted (once per PLAN)
  tester-dispatch.md      (top)  rubric ELAI-TEST-CHECKPOINT-MAP.json +        accumulated evidence/*.json -> defects as queue items
                                  conductor tester stack                       (jump the queue, re-enter S1..S11)

PER-TASK SEQUENCE
  S1+S2 (paired) -> S3 precheck-route (runs wiring-backend + wiring-frontend in mode=precheck, paired)
    route=polish (wired=yes):    S9 (route=wired-yes) -> S7 done-gate (wiring-backend + wiring-frontend, mode=done-gate, paired) -> S10
    route=execute (partial|no):  S4 -> S5 (serial, one executor; #2 resumes from {handoff_file}) -> S6 (+S6b fixer, re-audit once)
                                 -> S7 done-gate (paired) -> S8 -> S9 (route=execute-cleanup) -> S10
  S11 memory: once per PLAN completion (after all tasks committed), not per task.

CONCURRENCY: max 2 GLM instances total, any role. S1+S2 pair; the two wiring checks pair; never 3 at once.
A single plan never has 2 executors at the same time (S5 is serial; #2 starts only from #1's handoff).

GUARDRAIL: ../guardrail.md — the lean PreToolUse Bash hook spec (PHASE 0.6). Bans only catastrophic git/fs ops;
allows normal git add/commit/push (the committer) + file edits + single-file checkout.

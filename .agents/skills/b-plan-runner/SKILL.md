---
name: b-plan-runner
description: Orchestrates a full B-N (numbered B-plan phase) execution end-to-end. Triggers on "run B-N", "continue B-N", "execute B-N", "start phase X", or any explicit reference to a B-plan file under F:\.ELAI_workflow\docs\thefinalglance\B-plan\. Encapsulates the FSM cycle: context-compiler -] scope-planner-elai -] (parallel explorers + synthesizer) -] spec-grounder -] prompt-engineer -] impact-mapper -] EXECUTING -] auditor -] (fixer if BLOCKs) -] wiring-contract-validator -] memory-writer -] COMMITTING. Single user command, full FSM cycle.
license: internal
compatibility: "Requires .elai_cc/ layout, local-rag MCP server, FSM state machine"
metadata:
  version: "1.0.0"
  author: "elai"
---

B-Plan runner

Trigger: any user message matching one of these patterns
- "run B-N", "execute B-N", "continue B-N" (where N is 1-38 or "patch-X.Y")
- "start phase X" referencing a B-plan file
- "implement B-N" or "ship B-N"
Skip: "explain B-N", "what is B-N", "show B-N status" (these are read-only — answer directly without invoking this skill).

This skill is a state machine encoded as a sequence. Follow it top to bottom, never skip a step. The main model executes; sub-agents do the legwork.

PRECONDITIONS (verify before step 1)
1. mcp_fsm_get_state. If state != IDLE and != CONFIGURING, ask user before resetting.
2. Resolve the B-N file path. Common locations:
   - F:\.ELAI_workflow\docs\thefinalglance\B-plan\B-{N}_*.md
   - F:\.ELAI_workflow\workflow\sessions\*\HANDOFF*.md (for resumes)
   If file missing, return: "B-N file not found at expected paths. Provide explicit path or check F:\.ELAI_workflow\docs\thefinalglance\B-plan\."
3. Determine session_id (s-{parent_pid}). Register via mcp_session_register if not already.
4. Quick triage:
   - Is this a continuation of an in-flight B-N (existing workflow/sessions/{old_id}/)? If yes, read that session's HANDOFF and offer to resume from the partial state.
   - Is the B-N marked DONE in the index? If yes, ask user: "B-N marked DONE in s-XXXXX. Re-run anyway, or show what was shipped?"

AGENT RULE INJECTION (binding, applies to EVERY Spawn step below): each agent prompt begins with the pointer line from .Codex/rules/agent-dispatch-preamble.md - "Read F:\.ELAI_workflow\.Codex\rules\agent-dispatch-preamble.md FIRST and obey it as binding. handoff_file=workflow/handoff/<bn-topic-slug>.md; task_id=<step>; scope=<paths>." Sub-agents do not reliably load AGENTS.md; the preamble bans worker git mutation (the 2026-06-11 git reset --hard incident), bans destructive ops, locks scope, and defines BLOCKED=handoff-entry + structured return. When an agent returns BLOCKED: read its handoff entry, decide, append a DECISION block, point the successor at it (mediation loop per orchestrate SKILL.md).

STEP 1 — CONTEXT (PLANNING state)
mcp_fsm_transition("PLANNING")
Spawn context-compiler agent (.Codex/agents/context-compiler.md) with:
  task: "B-N: <one-sentence summary from the B-N file's title line>"
  session_id: <session_id>
  bn_file: <absolute path to B-N file>
The agent reads the B-N file directly, plus memory-bank, plus elai.map. It writes workflow/sessions/{session_id}/context-package.md.
Wait for confirmation.

STEP 2 — SCOPE (still PLANNING)
Spawn scope-planner-elai agent (.Codex/agents/scope-planner-elai.md) with:
  bn_file: <path>
  session_id: <session_id>
  current_project: "elai_cc"
The agent decides EXPLORE vs SKIP. Read its single-line return + the exploration-plan.md it produces.

If SKIP: jump to step 4.
If EXPLORE (N partitions): proceed to step 3.

STEP 3 — PARALLEL EXPLORATION (still PLANNING)
For each partition in the exploration-plan, spawn integration-explorer-elai agents IN PARALLEL (single message, multiple Agent calls). Pass:
  subsystem: <name from partition>
  entry_points: <files from partition>
  integration_question: <from partition>
  see_also_scope: <other partitions from plan>
  session_id: <session_id>
  current_project: "elai_cc"
Wait for ALL to complete.

Then spawn synthesizer-elai (.Codex/agents/synthesizer-elai.md) with:
  session_id: <session_id>
  subsystems: <list from plan>
  hub_watch: <hub list from plan>
The synthesizer appends a WIRING section to context-package.md. Wait for confirmation.

If synthesizer reports PARTITION GAPS or HIGH-severity DRIFT, surface to user before proceeding. User must approve.

STEP 4 — SPEC GROUNDING (still PLANNING)
Spawn spec-grounder agent (.Codex/agents/spec-grounder.md) with:
  bn_file: <path>
  session_id: <session_id>
  phase_id: <B-N or extracted from filename>
The agent verifies "Spec references:" claims and writes workflow/reasoning/phase-{N}-grounding.md.
Block proceeding if any reference is MISSING or MISMATCH. UNGROUNDED claims produce a warning but do not block — main model annotates them in the prompt.

STEP 5 — PROMPT TRANSFORM (still PLANNING)
Spawn prompt-engineer agent (.Codex/agents/prompt-engineer.md) with:
  task: "<B-N title and goal>"
  session_id: <session_id>
The agent reads workflow/sessions/{session_id}/context-package.md (which now includes WIRING + grounding refs). Returns the structured prompt.

Show the prompt to user. Ask: "Reformatted. Approve, edit, or abort?"
Wait for confirmation. Do not proceed without explicit approval (AGENTS.md hard rule).

STEP 6 — TESTING_FIRST
mcp_fsm_transition("TESTING_FIRST")
mcp_session_claim(owns="<files-from-WIRING_CONTRACT.produces>", locks="<files-from-blast-radius>")
If conflict, surface to user.

Run the contract-test health check: spawn auditor with mode="health-check" to run python .Codex/tools/pipeline/run_pipeline.py --tiers 1,2 -v before phase work. Fix any pre-existing failures or document them in workflow/test-baseline.json.

Write phase-specific tests per the B-N's "Tests to add" block.

STEP 7 — PRE-EDIT BLAST RADIUS (still TESTING_FIRST or transitioning to EXECUTING)
Spawn impact-mapper agent (.Codex/agents/impact-mapper.md) with:
  files_planned: <files about to change>
  symbols_planned: <public symbols if known>
  session_id: <session_id>
  depth: 2
Read the blast-radius.md it writes. If cross-lang risk flagged, queue both sides for the same EXECUTING pass (or explicitly opt out via the cross-lang-skip comment).

If the impact-mapper reports breaking changes to >=3 cross-crate consumers without a corresponding WIRING_CONTRACT update in the B-N file, ask user: "Wide blast radius. Update WIRING_CONTRACT or accept risk?"

STEP 8 — EXECUTING
mcp_fsm_transition("EXECUTING")
mcp_think_set_budget per the prompt-engineer's GRC_BUDGET line.
Write workflow/reasoning/phase-{N}-decisions.md (4 sections: What I will implement / Key decision / Expected test outcome / Unknowns).

Now the main model writes the implementation. Use mcp_think for reasoning. Do NOT run tests.

If anti-spiral triggers (anti-spiral skill watches): pivot persona per that skill.

mcp_fsm_transition("AUDITING") when implementation is complete.

STEP 9 — AUDITING
Spawn auditor agent (.Codex/agents/auditor.md) with:
  task domain: <trading|game|dashboard|application|general — derive from B-N>
  plan_file: <bn_file>
  phase_id: <B-N>
The auditor advances through READ_CONTEXT -> RUN_TESTS -> REVIEW_CODE -> WRITE_FINDINGS sub-states automatically.

Read auditor return. Check the PROGRESS SUMMARY line first. Note the Regression module.

If "DEAD END": mcp_fsm_transition("PLANNING", dead_end_reason=...). Stop.

If contains BLOCK findings:
  mcp_fsm_transition("EXECUTING")
  Spawn fixer agent (.Codex/agents/fixer.md) with:
    block_findings: <structured list from auditor>
    regression_module: <from PROGRESS SUMMARY>
    audit_report_path: <auditor output path>
    session_id: <session_id>
    phase_id: <B-N>
  Wait. Read fix report. If STUCK: pivot persona via anti-spiral skill.
  Then mcp_fsm_transition("AUDITING") and re-spawn auditor ONE more time. If BLOCKs persist: surface to user.

If "AUDIT PASSED": proceed to step 10.

STEP 10 — WIRING CONTRACT VALIDATION (still AUDITING after pass; fail-closed per .Codex/rules/wiring-enforcement.md)
10a. FREEZE CHECK (R8): the auditor runs `python .Codex/tools/gate_freeze_check.py`. Exit 3 (BLOCK) means THIS run modified its own gate (wiring_contract_check.py, the validator agent, the manifests, the allowlist). That is a P0: the gate is authored out-of-band only. Surface to user; do not proceed. Exit 2 (NO_BASELINE) means run `--lock` once out-of-band first. Exit 0 only = proceed.
10b. WIRING CONTRACT (R1-R7): spawn wiring-contract-validator (.Codex/agents/wiring-contract-validator.md) with bn_file=<path>, session_id=<session_id>. The agent is a thin runner over .Codex/tools/wiring_contract_check.py (the single arbiter; the trigger runs by default, the off-agent SSB row is load-bearing, a missing/incomplete contract or missing ssb.db is FAIL). Read VERDICT. If FAIL: spawn fixer with the findings, re-validate. Cap retries at 1.
10c. SMOKE MANIFEST (Layer 3): the auditor verifies the shipped feature has a [[feature]] row in .elai_cc/smoke-manifest.toml whose entrypoint matches the contract's observable_effect.trigger. Missing entry for a new feature = BLOCK.
10d. NO DEFAULT-OFF GRAVEYARD (R9): if the feature ships behind a new default-OFF flag (ELAI_*_ENABLE / cargo feature), the auditor verifies a complete [[flag]] row exists in .elai_cc/wire-dark-allowlist.toml (all fields: type, owner, expiry_date, promotion_plan_id, smoke_test). Unregistered default-OFF flag = BLOCK.

STEP 11 — TEST ATTRIBUTION (still AUDITING)
Spawn test-attributer (.Codex/agents/test-attributer.md) with the audit's test output.
Re-classify any failures. If reclassification reduces real BLOCKs to 0, treat as AUDIT PASSED.

STEP 12 — MEMORIZING
mcp_fsm_transition("MEMORIZING")
Spawn memory-writer agent (.Codex/agents/memory-writer.md) with the structured change manifest:
  task: <one-line>
  status: implementing
  next_step: <next B-N or COMPLETE>
  files_edited: <list with descriptions>
  build_status: PASS
  build_tested: <what tier 1-5 tested>
  architecture_decisions: <any locked decisions>
  new_files: <new files to add to projectIndex>
  gotcha: <any new failure pattern>
  memorize_topic: "bn-{N}-decisions"
  memorize_tags: "b-plan,phase-{N},<crate-list>"
  memorize_content: <decision summary>
Wait for completion. Verify memory-bank files were updated (AGENTS.md MEMORY VERIFICATION rule).
FEATURE-MAP UPSERT (mandatory when docs/thefinalglance/ELAI-FEATURE-MAP.md exists): the map is the source of truth for ELAI features + their composition and must never go stale. Upsert every feature this B-N shipped/activated/changed: flip PLANNED to WIRED, or to DARK(flag) if it shipped behind a default-off flag with an allowlist row - the wiring/audit gate verdicts are the ground truth for the flip, never the executor's self-assessment. Adjust composes/extends cross-links, add entries the plan created that are missing, move discovered rejections to the REJECTED ledger, bump the header date. One flag per feature, never narrative.
mcp_session_release()

STEP 13 — COMMITTING (skip if user said no-commits)
mcp_fsm_transition("COMMITTING")
git status / git diff
Stage only files relevant to this B-N.
Commit with message format: "B-{N}: <imperative summary>" + body listing files + test results. NO Co-Authored-By trailer (AGENTS.md hard rule).
Report commit hash.
mcp_fsm_transition("PLANNING") if more B-Ns queued, else mcp_fsm_transition("COMPLETE").

POST-RUN
Update MEMORY.md with a one-line entry pointing at the session's reasoning doc and audit summary.
Tell the user: B-N complete with test counts, files changed, key deltas, and what's next.

ABORT PROTOCOL
At any step, if user says "stop" or "abort":
  mcp_fsm_reset to IDLE.
  Save state to workflow/sessions/{session_id}/SESSION_HANDOFF.md (full fields).
  Stop work.

ANTI-SPIRAL HOOKS
- Same file edited >3 times for same fix in EXECUTING: trigger anti-spiral skill.
- Auditor returns BLOCK twice in a row: trigger anti-spiral skill.
- Same hypothesis fails 3 times in mcp_think trace: trigger anti-spiral skill.

CONTEXT EXHAUSTION
If context drops below 20%: pause at next safe boundary (between FSM transitions). Run session-handoff procedure (write SESSION_HANDOFF.md with all open state). Tell user: "Context low — handoff written. Resume next session with: continue B-{N}."

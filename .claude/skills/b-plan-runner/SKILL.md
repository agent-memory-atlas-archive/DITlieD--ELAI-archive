---
name: b-plan-runner
description: Orchestrates a full B-N (numbered B-plan phase) execution end-to-end. Triggers on "run B-N", "continue B-N", "execute B-N", "start phase X", or any explicit reference to a B-plan file under /workspace/ELAI\docs\thefinalglance\B-plan\. Encapsulates the FSM cycle: context-compiler -] scope-planner-elai -] (parallel explorers + synthesizer) -] spec-grounder -] prompt-engineer -] impact-mapper -] EXECUTING -] auditor -] (fixer if BLOCKs) -] wiring-contract-validator -] memory-writer -] COMMITTING. Single user command, full FSM cycle.
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
   - /workspace/ELAI\docs\thefinalglance\B-plan\B-{N}_*.md
   - /workspace/ELAI\workflow\sessions\*\HANDOFF*.md (for resumes)
   If file missing, return: "B-N file not found at expected paths. Provide explicit path or check /workspace/ELAI\docs\thefinalglance\B-plan\."
3. Determine session_id (s-{parent_pid}). Register via mcp_session_register if not already.
4. Quick triage:
   - Is this a continuation of an in-flight B-N (existing workflow/sessions/{old_id}/)? If yes, read that session's HANDOFF and offer to resume from the partial state.
   - Is the B-N marked DONE in the index? If yes, ask user: "B-N marked DONE in s-XXXXX. Re-run anyway, or show what was shipped?"
5. PLAN GATE PRE-FLIGHT (plan-standard.md + dispatch-core.md D-8, blocking): run python .claude/tools/plan_gate_check.py --file <bn_file>. Exit 0 -> proceed. Exit 1 -> the plan is NOT walked yet: run the planning-workflow "Plan improvement" protocol over the B-N file (grounding sweep + PRE_GATE/DETERMINISTIC_CHECK/ADVERSARIAL + wargame blocks: EXPECTED_OBSERVATION/ON_FAIL per move, FORK triggers, RECON_NEEDED, ABORT, REDTEAM), re-gate, then proceed. Exit 2 = fail-closed, diagnose. Only skip: explicit USER_OVERRIDE in the current message (log it). Legacy B-N files hit this exactly when something tries to execute them — refine-before-execute, never execute-blind-on-a-blind-plan.
5b. PLAN-DAG L8 (J-89): if the B-N file contains a `plan_dag` fence or `## Plan DAG` section, run `python .claude/tools/plan_dag_check.py --file <bn_file> --json`. Non-zero exit is a structural AUTHORING failure: fix the DAG or mark terminal sinks before EXECUTING. No plan_dag block = skip with log line (not a pass of completeness — only "checker not applicable").

AGENT RULE INJECTION + DISPATCH RULES (binding, applies to EVERY Spawn step below): .claude/rules/dispatch-core.md D-1..D-10 governs every dispatch — preamble pointer line (D-1, handoff_file=workflow/handoff/<bn-topic-slug>.md), model ladder (D-2), cap 3 (D-3), scoped context (D-4), handoff mediation on BLOCKED returns (D-5), execution/evaluation separation (D-6). Do not restate; read that file.

STEP 1 — CONTEXT (PLANNING state)
mcp_fsm_transition("PLANNING")
Spawn context-compiler agent (.claude/agents/context-compiler.md) with:
  task: "B-N: <one-sentence summary from the B-N file's title line>"
  session_id: <session_id>
  bn_file: <absolute path to B-N file>
The agent reads the B-N file directly, plus memory-bank, plus elai.map. It writes workflow/sessions/{session_id}/context-package.md.
Wait for confirmation.

STEP 2 — SCOPE (still PLANNING)
Spawn scope-planner-elai agent (.claude/agents/scope-planner-elai.md) with:
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

Then spawn synthesizer-elai (.claude/agents/synthesizer-elai.md) with:
  session_id: <session_id>
  subsystems: <list from plan>
  hub_watch: <hub list from plan>
The synthesizer appends a WIRING section to context-package.md. Wait for confirmation.

If synthesizer reports PARTITION GAPS or HIGH-severity DRIFT, surface to user before proceeding. User must approve.

STEP 4 — SPEC GROUNDING (still PLANNING)
Spawn spec-grounder agent (.claude/agents/spec-grounder.md) with:
  bn_file: <path>
  session_id: <session_id>
  phase_id: <B-N or extracted from filename>
The agent verifies "Spec references:" claims and writes workflow/reasoning/phase-{N}-grounding.md.
Block proceeding if any reference is MISSING or MISMATCH. UNGROUNDED claims produce a warning but do not block — main model annotates them in the prompt.

STEP 5 — PROMPT TRANSFORM (still PLANNING)
Spawn prompt-engineer agent (.claude/agents/prompt-engineer.md) with:
  task: "<B-N title and goal>"
  session_id: <session_id>
The agent reads workflow/sessions/{session_id}/context-package.md (which now includes WIRING + grounding refs). Returns the structured prompt.

Show the prompt to user. Ask: "Reformatted. Approve, edit, or abort?"
Wait for confirmation. Do not proceed without explicit approval (CLAUDE.md hard rule).

STEP 6 — TESTING_FIRST
mcp_fsm_transition("TESTING_FIRST")
mcp_session_claim(owns="<files-from-WIRING_CONTRACT.produces>", locks="<files-from-blast-radius>")
If conflict, surface to user.

Run the contract-test health check: spawn auditor with mode="health-check" to run python .claude/tools/pipeline/run_pipeline.py --tiers 1,2 -v before phase work. Fix any pre-existing failures or document them in workflow/test-baseline.json.

Write phase-specific tests per the B-N's "Tests to add" block.

STEP 7 — PRE-EDIT BLAST RADIUS (still TESTING_FIRST or transitioning to EXECUTING)
Spawn impact-mapper agent (.claude/agents/impact-mapper.md) with:
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
Spawn auditor agent (.claude/agents/auditor.md) with:
  task domain: <trading|game|dashboard|application|general — derive from B-N>
  plan_file: <bn_file>
  phase_id: <B-N>
The auditor advances through READ_CONTEXT -> RUN_TESTS -> REVIEW_CODE -> WRITE_FINDINGS sub-states automatically.

Read auditor return. Check the PROGRESS SUMMARY line first. Note the Regression module.

If "DEAD END": mcp_fsm_transition("PLANNING", dead_end_reason=...). Stop.

If contains BLOCK findings:
  mcp_fsm_transition("EXECUTING")
  Spawn fixer agent (.claude/agents/fixer.md) with:
    block_findings: <structured list from auditor>
    regression_module: <from PROGRESS SUMMARY>
    audit_report_path: <auditor output path>
    session_id: <session_id>
    phase_id: <B-N>
  Wait. Read fix report. If STUCK: pivot persona via anti-spiral skill.
  Then mcp_fsm_transition("AUDITING") and re-spawn auditor ONE more time. If BLOCKs persist: surface to user.

If "AUDIT PASSED": proceed to step 10.

STEP 10 — WIRING CONTRACT VALIDATION (still AUDITING after pass; fail-closed per .claude/rules/wiring-enforcement.md)
10a. FREEZE CHECK (R8): the auditor runs `python .claude/tools/gate_freeze_check.py`. Exit 3 (BLOCK) means THIS run modified its own gate (wiring_contract_check.py, the validator agent, the manifests, the allowlist). That is a P0: the gate is authored out-of-band only. Surface to user; do not proceed. Exit 2 (NO_BASELINE) means run `--lock` once out-of-band first. Exit 0 only = proceed.
10b. WIRING CONTRACT (R1-R7): spawn wiring-contract-validator (.claude/agents/wiring-contract-validator.md) with bn_file=<path>, session_id=<session_id>. The agent is a thin runner over .claude/tools/wiring_contract_check.py (the single arbiter; the trigger runs by default, the off-agent SSB row is load-bearing, a missing/incomplete contract or missing ssb.db is FAIL). Read VERDICT. If FAIL: spawn fixer with the findings, re-validate. Cap retries at 1.
10c. SMOKE MANIFEST (Layer 3): the auditor verifies the shipped feature has a [[feature]] row in .elai_cc/smoke-manifest.toml whose entrypoint matches the contract's observable_effect.trigger. Missing entry for a new feature = BLOCK.
10d. NO DEFAULT-OFF GRAVEYARD (R9): if the feature ships behind a new default-OFF flag (ELAI_*_ENABLE / cargo feature), the auditor verifies a complete [[flag]] row exists in .elai_cc/wire-dark-allowlist.toml (all fields: type, owner, expiry_date, promotion_plan_id, smoke_test). Unregistered default-OFF flag = BLOCK.

STEP 10e — CROSS-B-N INTEGRATION SWEEP (R10; runs INSIDE this skill, never dependent on orchestrate wrapping it)
Trigger: (a) this is the 2nd or 3rd B-N completed in the current session/chain since the last sweep, OR (b) this B-N's WIRING_CONTRACT consumes/produces a symbol another B-N in this chain shipped, OR (c) this is the last queued B-N. When triggered: spawn an INDEPENDENT agent (never the executor of any swept B-N) to drive the REAL entrypoint end-to-end on a HARD input (strict peer/provider, multi-turn, long enough to fire secondary paths — compaction/fallback/resume), asserting the off-agent artifact (SSB row / coverage hit) for EACH swept B-N's observable_effect in one run. Per-B-N green is necessary, never sufficient; seam defects between B-Ns pass every per-B-N audit (Liberty 2026-06-16). Sweep FAIL = BLOCK on the chain: fixer on the seam, re-sweep once, else surface to user. Log sweep verdicts in workflow/sessions/{session_id}/integration-sweeps.md.

STEP 11 — TEST ATTRIBUTION (still AUDITING)
Spawn test-attributer (.claude/agents/test-attributer.md) with the audit's test output.
Re-classify any failures. If reclassification reduces real BLOCKs to 0, treat as AUDIT PASSED.

STEP 12 — MEMORIZING
mcp_fsm_transition("MEMORIZING")
Spawn memory-writer agent (.claude/agents/memory-writer.md) with the structured change manifest:
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
Wait for completion. Verify memory-bank files were updated (CLAUDE.md MEMORY VERIFICATION rule).
FEATURE-MAP UPSERT (mandatory when docs/thefinalglance/ELAI-FEATURE-MAP.md exists): the map is the source of truth for ELAI features + their composition and must never go stale. Upsert every feature this B-N shipped/activated/changed: flip PLANNED to WIRED, or to DARK(flag) if it shipped behind a default-off flag with an allowlist row - the wiring/audit gate verdicts are the ground truth for the flip, never the executor's self-assessment. Adjust composes/extends cross-links, add entries the plan created that are missing, move discovered rejections to the REJECTED ledger, bump the header date. One flag per feature, never narrative. SPEC-OVERVIEW: docs/thefinalglance/ELAI-SPEC-OVERVIEW.md is design-only (no status, no plan-ids); a pure PLANNED->WIRED flip is a NO-OP for it — touch it ONLY if this B-N changed the DESIGN (a capability the spec never described got added, or a described one got removed), never for a status flip.
mcp_session_release()

STEP 13 — COMMITTING (skip if user said no-commits)
mcp_fsm_transition("COMMITTING")
git status / git diff
Stage only files relevant to this B-N.
Commit with message format: "B-{N}: <imperative summary>" + body listing files + test results. NO Co-Authored-By trailer (CLAUDE.md hard rule).
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

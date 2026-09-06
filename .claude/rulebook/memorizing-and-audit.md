MEMORIZING + audit + FSM cleanup (HARD RULE)

Loaded by: any task entering AUDITING or MEMORIZING state, or any post-phase write.

MEMORIZING delegation

In MEMORIZING, spawn memory-writer agent (.claude/agents/memory-writer.md). NEVER write memory-bank files directly. Agent uses mcp_memorize / mcp_log_change / mcp_update_context for RAG indexing. Manual writes bypass indexing and violate context delegation.

Do not delete templates (Format: blocks) inside memory-bank unless user explicitly asks.

Memory verification (after memory-writer reports done):
  1. Read workflow/memory-bank/apps/{current_app}/changelog.md — confirm phase's file entries appear (grep for phase ID or file paths).
  2. Read workflow/memory-bank/apps/{current_app}/activeContext.md — confirm task/status/next_step reflect the just-completed phase.
  3. Read workflow/memory-bank/apps/{current_app}/progress.md — confirm build status updated.
  If any file is stale, update it manually. Do NOT proceed to COMMITTING with stale memory-bank files. Log "MEMORY_VERIFY: [file] was stale, updated manually."

Audit cycles

1 audit cycle per agent / phase (previous 2-cycle default dropped). If audit fails after cycle 1, surface findings + remediation options to user. Do NOT auto-loop into cycle 2.

Auditor outcome protocol (top-to-bottom, stop at first match):
  1. Read the PROGRESS SUMMARY line (always first line of audit output).
  2. Note the Regression module — fix that file FIRST in any subsequent EXECUTING pass.
  3. "DEAD END" -> mcp_fsm_transition("PLANNING", dead_end_reason="..."). Git checkout phase files. STOP.
  4. Any "BLOCK" -> mcp_fsm_transition("EXECUTING"). Fix BLOCKs in this order: Regression module first, then remaining BLOCKs by severity. Fix ALL BLOCKs in a single EXECUTING pass. Do NOT re-transition to AUDITING — auditor runs once per phase.
  5. "AUDIT PASSED" -> mcp_fsm_transition("MEMORIZING").
  6. None of the above: treat as BLOCK.

Decision doc (soft enforcement)

Before EXECUTING, write workflow/reasoning/phase-X-decisions.md:
  - What I will implement (3 sentences max)
  - Key technical decision (1 sentence)
  - Expected test outcome (1 sentence)
  - Unknowns to verify empirically (max 3)
The auditor checks for this and warns if missing.

Contract tests

tests/contracts/ holds interface-level tests per module. Run at phase entry (health check) and during audit. Context-compiler generates the interface map; main LLM writes contract tests in TESTING_FIRST; auditor validates coverage.

Scoped test runs

Do NOT run the full test suite during implementation. Only XCTest cases for affected modules + relevant contract tests. Full suite is the auditor's job.

FSM cleanup at session end

After all plan phases complete: transition MEMORIZING -> COMPLETE -> IDLE before ending session. Never leave FSM in non-IDLE state at session end. Stop hook blocks termination in workflow states; COMPLETE requires transition to IDLE.

Hooks reminder

PreToolUse (path/tool gating per state+substate + cross-session file locks).
PostToolUse (syntax check, template preservation, change recording to session state).
Stop (blocks mid-workflow termination).
PreCompact (auto SESSION_HANDOFF before context truncation).
Hooks log to per-session workflow/sessions/{id}/hook-executions.log (fallback: workflow/hook-executions.log).

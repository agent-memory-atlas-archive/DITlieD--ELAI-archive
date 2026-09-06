Orchestrate protocol (HARD RULE summary)

Loaded by: the conditional-routing rule when the user types "orchestrate" or invokes the orchestrate skill. Sibling-rule pointer style — full protocol body lives in .claude/skills/orchestrate/protocol.md. This file is the one-screen hard-rule digest.

THE 10 HARD RULES OF ORCHESTRATE

O-HR1 OWN SESSION ONLY
The host (controller) session may call mcp_session_register, mcp_session_claim, mcp_session_release, mcp_fsm_get_state, mcp_fsm_transition, mcp_fsm_reset against ITS OWN session id only. Same for the launched executor session. NEVER mutate another session's FSM or registry entry. Reads via mcp_session_list are always fine.

O-HR2 DEP-WAIT IS SLEEP-FOREGROUND, NOT POLL-IN-LOOP
Dependency wait runs through tools/dep-wait.ps1 which uses Start-Sleep -Seconds 60 between checks. Do not implement dep-wait via repeated Bash invocations from the main LLM (token cost) or via continuous loops. The script's PowerShell loop is the only correct dep-wait implementation.

O-HR3 PARALLEL CAP = 3 (NOT 4)
Inside orchestrate, the maximum number of parallel subagent dispatches per Agent-tool batch is 3. This overrides .claude/rules/agent-spawning.md's default of 4 for the duration of an orchestrate run. The launched session must enforce this on every batch.

O-HR4 SKIP PROMPT-ENGINEER, SKIP DOUBLE AUDITS
The orchestrate per-sub-phase loop does NOT include a prompt-engineer step. The auditor runs ONCE per sub-phase. On BLOCK, apply .claude/rules/bubble-up.md retry budgets — do NOT loop AUDITING > EXECUTING > AUDITING > EXECUTING.

O-HR5 ONE COMMIT, AT THE VERY END
No per-sub-phase commits. No per-baby-step commits. Accumulate ALL changes across the entire orchestrate run. Single git commit at step 11 of the protocol. Commit message follows .claude/rules/commit-cadence.md maintainer voice rules.

O-HR6 AUTONOMOUS MODE IS THE DEFAULT
The orchestrator does NOT pause for user input during execution. It picks the best non-lazy grounded option and continues. The ONLY exceptions are the four tripwire conditions in O-HR9.

O-HR7 INTEGRATION DEPTH IS NON-NEGOTIABLE
Every sub-phase MUST run context-compiler + integration-explorer(s) in parallel BEFORE EXECUTING. Every sub-phase MUST run auditor + (where applicable) wiring-contract-validator + memory-writer after EXECUTING. Every sub-phase MUST run a phase-end sweep with all stack-specific gates. No shortcuts. No "this sub-phase is trivial". If the sub-phase IS trivial, the parent step decomposition was wrong — surface to user.

O-HR8 PHASE-END SWEEP GATES ARE BLOCKING
The 6 generic + per-stack + cross-stack gates are HARD GATES. A sub-phase or full run cannot complete until every gate passes OR each failure is bubble-up-resolved per .claude/rules/bubble-up.md. No "warning, continuing" for production code paths. Warnings are escalations.

O-HR9 THE FOUR TRIPWIRES (sole user-interrupt conditions)
T1 IRREVERSIBLE OP — the launcher's pattern-match on Bash commands detects: git push --force, git push -f, git reset --hard origin/*, rm -rf at root paths, cargo publish, npm publish, pnpm publish, twine upload, DROP TABLE, DROP DATABASE, TRUNCATE TABLE. Match = pause + surface.
T2 50-EVENT BUBBLE-UP CAP — per .claude/rules/bubble-up.md SAFETY rule. The launcher counts bubble-up-history.md append signals. At 50 = force abort.
T3 DEP-WAIT > 4h — tools/dep-wait.ps1 returns non-zero on timeout. Surface with [keep waiting / abort / proceed-without-dep].
T4 STALL / SWEEP-FAIL / DEAD-END — 3 consecutive 30-min stalls without progress (Magentic counter) OR 3 consecutive sub-phase failures on the same gate OR auditor returns DEAD END with algorithm/platform_compliance category. Surface with diagnostic + recommended actions.

O-HR10 REAL-MODEL TEST BEFORE COMMIT
Before the single end-of-orchestrate commit, run the project's full test suite under MODEL_EXECUTOR = claude-sonnet-4-6 cloud (per user override). Pure-mock tests are insufficient. For ELAI: real-model integration tests use the remote sidecar — verify sidecar health via sidecar-protocol skill before invoking. For RN/Expo: web preview capture + iPhone test deferred to ui-verification.md (out of scope for orchestrate's end gate).

WHAT ORCHESTRATE NEVER DOES

- never asks "should i proceed" after each step
- never defers a decision the user could have surfaced upfront ("which approach would you like ?")
- never writes commits per sub-phase
- never runs more than 3 agents in one parallel batch
- never touches another session's FSM
- never reads .claude/agents/*.md content directly into the host context (per CLAUDE.md context-delegation rule — agents read their own definitions)
- never accepts a "lazy" option when a grounded option is available
- never skips integration-explorer to save time

CROSS-REFERENCES

.claude/skills/orchestrate/SKILL.md         — trigger contract
.claude/skills/orchestrate/protocol.md      — deep step-by-step (canonical body)
.claude/skills/orchestrate/overrides-template.md — boot prompt template
.claude/skills/orchestrate/stack-adapters/  — per-stack gate definitions
.claude/skills/orchestrate/compat/          — per-target-project overrides
.claude/skills/orchestrate/tools/           — PS1 wrappers
.claude/rules/bubble-up.md                  — retry budgets, 50-event safety cap, PLANNING-TIME and EXECUTION-TIME modes
.claude/rules/agent-spawning.md             — canonical agent list (target project's wins at execution time)
.claude/rules/commit-cadence.md             — maintainer voice + commit message style
.claude/rules/token-discipline.md           — zero-lazy-code, anti-slopsquatting, anti-stale-training rules

END orchestrate-protocol.md

---
name: orchestrate
description: Autonomous phase orchestrator. Triggers on the literal word "orchestrate", "orchestrate [plan-path]", or "orchestrate [plan] waits on [sessions/B-Ns]". Reads a phase markdown file (often in a sibling project like .ELAI_workflow), parses Phase ] sub-Phase ] Task structure, walks an FSM cycle PER sub-phase (context-compiler + integration-explorer in parallel cap=3 ] skip prompt-engineer ] executor ] auditor 1 cycle ] memory-writer), waits on dependency sessions via sleep-poll without ever touching their FSM, runs 6 stack-agnostic phase-end integration gates plus stack-specific gates (cargo-modules / dep-cruiser / ts-rs bindings drift), and commits ONCE at end. Does NOT ask the user except on tripwire conditions (irreversible op, 50-event bubble cap, dep-wait ] 4h, sweep-fail after 3 retries). When the target project has its own orchestration skill (b-plan-runner in ELAI), delegates to it via a bootstrapped claude -p session at target's CWD with policy overrides baked into the boot prompt.
user-invocable: true
allowed-tools: Read, Write, Edit, Glob, Grep, Bash
model: opus
license: internal
compatibility: "Requires .elai_cc/ layout, local-rag MCP server, FSM state machine"
metadata:
  version: "1.0.0"
  author: "elai"
---

orchestrate — autonomous phase runner

WHEN THIS SKILL FIRES

Trigger phrases (case-insensitive, anywhere in user message):
- "orchestrate"
- "orchestrate <path>"
- "orchestrate <path> waits on <ids>" / "orchestrate <path> depends on <ids>"
- "orchestrate <path> --autonomous" (explicit autonomous mode, default ON anyway)
- "orchestrate resume <session-id>" (resume from a prior orchestrate run's SESSION_HANDOFF)

Does NOT fire on:
- "explain orchestrate" / "what does orchestrate do" — read-only, answer directly
- "show orchestrate status" — read state files directly without invoking the protocol

INVOCATION CONTRACT

User input shapes that resolve correctly:
- "orchestrate /workspace/ELAI\docs\thefinalglance\B-plan\B-30_multilang_dkg_hardening.md, depends on B-29 and B-29.5"
- "orchestrate the B-30 plan, B-29 + B-29.5 running, dont bother me, test with sonnet 4.6 cloud"
- bare "orchestrate" with no path > ask user for plan path (this is the ONE pre-flight question allowed)

HARD RULE (BLOCKING — see .claude/rules/orchestrate-protocol.md for H-1..H-10):
When this skill fires, agent dispatch is NON-OPTIONAL. The main LLM is the controller and dispatches the canonical ELAI agents (context-compiler, integration-explorer-elai, auditor, memory-writer, fixer, wiring-contract-validator) per protocol.md step 8. It does NOT run those steps in-band by itself. NO Edit/Write on plan-scope files BEFORE the parallel batch returns. NO test/build runs by the main LLM (auditor only). NO memory-bank writes by the main LLM (memory-writer only). "Save tokens across N queued plans" / "small plan" / "I already read it" / "this is just UI" are NOT valid skip reasons — sub-agents have their own GC'd context, so dispatching them SAVES the main LLM's context. The only legal skips: (a) explicit USER_OVERRIDE in the current message, (b) patch-N single-file isolated fixes, (c) prose-only documentation sub-phases. ELAI agents are NOT in the Agent tool's subagent_type list; dispatch via subagent_type=general-purpose with the prompt: "Read /workspace/ELAI\.claude\agents\<name>.md and follow it strictly. <task args>". This block exists because the main LLM has repeatedly skipped agent dispatch under bogus optimizations.

WHAT THIS SKILL ACTUALLY DOES

1. Parses user input to extract: plan_path (required), depends_on (list of B-N ids or session ids, default empty), parallel_cap (default 3), model_executor (default sonnet-4.6), autonomous (default true).
2. Resolves target project from plan_path (walks up looking for .claude/ folder).
3. Loads compat/<target_project>.md if present (e.g. compat/ELAI.md for /workspace/ELAI\...). Compat file declares native orchestration skill + stack + integration gates + dashboard wiring path.
4. Detects stack via tools/detect-stack.ps1 (Cargo.toml > rust, package.json > typescript, .xcodeproj > swift, multi > cross-stack).
5. Loads matching stack-adapter file from stack-adapters/.
6. Registers THIS session via mcp_session_register (apps_workflow side, even though work runs in target project).
7. For each dep in depends_on: tools/dep-wait.ps1 polls mcp_session_list every 60s until released. Never calls mcp_fsm_* against another session.
8. Generates boot prompt by filling overrides-template.md with: plan_path, depends_on (now empty), parallel_cap, sub-phase-grain instruction, skip-prompt-engineer flag, 1-audit-cycle flag, commit-at-end-only flag, autonomous flag, stack adapter content inlined, compat content inlined.
9. Launches `claude -p --output-format stream-json --resume <sid>` via tools/orchestrate-launcher.ps1 with CWD = target project root. The bootstrapped session sees target's native .claude/ (b-plan-runner skill for ELAI, planning-workflow for apps_workflow).
10. The bootstrapped session runs target's native orchestration skill BUT with the per-sub-phase loop inserted between PROMPT_TRANSFORM and EXECUTING, AND with the override policies enforced.
11. Tails stream-json, parses system/init (verify hooks loaded), system/api_retry (rate-limit observability), tool_use (progress), result (per-sub-phase completion). Surfaces tripwires.
12. Phase-end sweep (tools/phase-end-sweep.ps1) runs the 6 generic + stack-specific gates after each sub-phase. BLOCK any sub-phase that fails the sweep; bubble-up policy decides next action.
13. After all sub-phases complete + final sweep passes: bootstrapped session does ONE commit per commit-cadence.md. Reports final result.
14. FEATURE-MAP UPSERT: if the run shipped/activated/changed ELAI features and docs/thefinalglance/ELAI-FEATURE-MAP.md exists (the source of truth for features + their composition), upsert the touched entries (status flag flip, cross-links, new entries) and bump its header date. One flag per feature, never narrative. SPEC-OVERVIEW: also upsert docs/thefinalglance/ELAI-SPEC-OVERVIEW.md ONLY if the run changed the DESIGN (added/removed a capability the design-only spec did not describe); a pure PLANNED->WIRED flip is a no-op there (no wiring/status in that file).

DISPATCH MACHINERY (binding) — .claude/rules/dispatch-core.md D-1..D-10 governs every dispatch this skill (or its bootstrapped executor) makes: preamble rule injection incl. the R-8/D-030 production-caller contract (D-1), model ladder with model_executor default sonnet-4.6 (D-2), hard cap 3 + fleet announcement with "orchestrate <plan>" as standing authorization for the canonical per-sub-phase fleet only (D-3), scoped context (D-4), handoff mediation loop (D-5), execution/evaluation separation (D-6), R10 integration gate (D-7), plan-gate pre-flight (D-8), loop prevention (D-9), git discipline (D-10). Do not restate; read that file. A dispatch without the D-1 preamble pointer is a protocol violation (same severity as skipping the parallel batch).

PLAN GATE PRE-FLIGHT (D-8, plan-standard.md): before step 1's plan walk begins, run python .claude/tools/plan_gate_check.py --file <plan_path>. FAIL -> refine-before-execute: run planning-workflow "Plan improvement" over the plan (wargame blocks injected: EXPECTED_OBSERVATION/ON_FAIL, FORK triggers, RECON_NEEDED, ABORT, VERIFY off-agent artifacts, REDTEAM), re-gate, then walk. Only skip: explicit USER_OVERRIDE in the current message, logged in orchestrate-run.md.

THE 4 TRIPWIRE CONDITIONS (only times orchestrate breaks autonomous mode)

T1 IRREVERSIBLE OP detected in proposed Bash command (git push --force, git reset --hard/--mixed, git checkout -- <paths> / git restore, git clean, git stash drop, any git history rewrite, rm -rf, db drop, prod deploy, npm publish, cargo publish): surface to user with the exact command and a Y/N. Default no. Destructive git is in this list because of the 2026-06-11 incident: an executing agent ran git reset --hard 8x to "self-repair" a confusing tree and wiped the whole integration diff. A wrong-looking working tree is a TRIPWIRE, never something a session repairs with resets. Also: mcp__pod-lifecycle__pod_stop or mcp__pod-lifecycle__pod_terminate when pod_cost_status shows nonzero cost-so-far: surface the cost figure in the Y/N confirm (H-38 compose, not a restatement of the tripwire).
T2 50-EVENT BUBBLE-UP CAP reached (per .claude/rules/bubble-up.md SAFETY rule): force ABORT and SESSION_HANDOFF.
T3 DEP-WAIT EXCEEDS 4h on any single dependency: surface to user with options [keep waiting / abort / proceed-without-dep].
T4 SWEEP FAILS after 3 consecutive sub-phase retries OR after fixer agent has been spawned 3 times with no progress (Magentic stall counter): surface to user with the failing gate + recommended actions.

In all other cases: orchestrate runs through. No "should i proceed" prompts. No "i found a question" deferrals. Choose the best non-lazy grounded option and continue.

OUTPUTS THIS SKILL PRODUCES

- workflow/sessions/<sid>/orchestrate-run.md — full run log, every sub-phase decision, every agent dispatch, every gate result
- workflow/sessions/<sid>/orchestrate-state.json — FSM + cursor state (sub-phase index, batch index, retry counts)
- workflow/sessions/<sid>/phase-reports/<sub-phase-id>.md — per-sub-phase end report (one per Phase 1, Phase 2, ...)
- workflow/sessions/<sid>/SESSION_HANDOFF.md — only on abort / tripwire / PreCompact
- one git commit at end of run (in target project, not apps_workflow)

DEEP PROTOCOL

See protocol.md for the full step-by-step. This file is the trigger contract; protocol.md is the body. The launched session reads overrides-template.md (filled) as its boot prompt. Per-stack gates live in stack-adapters/. Per-target compat lives in compat/.

ESCAPE HATCHES

- user types "abort orchestrate" at any time > tools/orchestrate-launcher.ps1 sends SIGINT to the bootstrapped session, runs session-handoff, returns control
- user types "pause orchestrate" > save state, kill launched session, mark resumable. Resume via "orchestrate resume <sid>".
- PreCompact hook fires in launched session > session-handoff.ps1 auto-writes SESSION_HANDOFF.md
- launched session crashes / hits rate limit > stream-json result event with error_status reports back, orchestrate-launcher.ps1 emits a clear failure summary

CROSS-REFERENCES

- protocol.md         — deep step-by-step
- overrides-template.md — boot prompt template (filled per-run)
- stack-adapters/     — per-stack gate definitions
- compat/             — per-target-project overrides + delegate map
- tools/              — PS1 wrappers (launcher, detect-stack, dep-wait, phase-end-sweep)
- .claude/rules/orchestrate-protocol.md — hard-rule summary
- .claude/rules/bubble-up.md             — retry budgets + 50-event cap
- .claude/rules/agent-spawning.md        — canonical agent list (target project's agents take precedence at execution time)

END SKILL.md

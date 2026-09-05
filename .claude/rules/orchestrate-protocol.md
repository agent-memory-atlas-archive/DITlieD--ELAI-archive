orchestrate-protocol — HARD RULE, BLOCKING

Loaded by: the orchestrate skill, and by any session that has touched a plan file under docs/thefinalglance/{B,C,D,E}-plan/ in the current message thread.

This file exists because the main LLM has, repeatedly, skipped the orchestrate agent-dispatch step under "save token budget" rationalizations. That rationalization is FALSE — sub-agents run in their OWN context window that is garbage-collected on return. Dispatching an agent SAVES the main LLM's context, it does not cost it. Skipping the dispatch costs MORE context because the main LLM ends up reading plan files, reading source files, doing exploration, etc, all of which would have been done by a sub-agent and discarded.

THE HARD RULES (BLOCKING — violations are P0 incidents, log to gotchas.md)

H-1. ORCHESTRATE PROTOCOL IS NOT OPTIONAL.
  When the orchestrate skill fires (user says "orchestrate"):
    - the main LLM IS the controller
    - the main LLM does NOT execute steps 8.3 (PARALLEL BATCH), 8.5 (EXECUTING), 8.6 (AUDITING), 8.7 (MEMORIZING), 8.8 (PHASE-END SWEEP) in-band by itself
    - the main LLM dispatches the listed agents and reads their returns
    - rationalizations like "the plan is small", "I already read it", "this is faster", "we have many plans queued", "this is just a UI plan" are NEVER sufficient grounds to skip
  The protocol can be modified by user override in the current message (e.g. "skip context-compiler this time, I already read the plan"). It cannot be modified by the LLM's own judgment.

H-2. NO EDIT / WRITE BEFORE PARALLEL BATCH RETURNS.
  In a sub-phase of orchestrate, the main LLM may NOT call Edit / Write / NotebookEdit on any file in the plan's scope until:
    (a) context-compiler has returned with a context-package-{sub_phase_id}.md path, AND
    (b) at least one integration-explorer-elai has returned with a workflow/sessions/<sid>/exploration/{sub_phase_id}-{subsystem}.md path
  Permitted Edit/Write BEFORE batch returns: only scaffolding files (session dirs, the orchestrate-run.md log, the orchestrate-state.json). NOT plan code, NOT components, NOT tokens, NOT tests.
  Violation indicator: any tool_use of Edit/Write inside an orchestrate sub-phase before the agent returns are received in the conversation. The main LLM SHOULD self-audit before each Edit by asking: "have the agents returned for this sub-phase yet ?". If no, REFUSE the Edit and dispatch the agents first.

H-3. NO AUDIT-SKIP.
  After EXECUTING, the main LLM MUST dispatch the auditor agent (.claude/agents/auditor.md). The main LLM does NOT run cargo / pnpm / svelte-check / vitest itself — those are the auditor's job. Even "just a quick svelte-check" is BLOCKED — that's the auditor's job. Rationale: the auditor sees a clean context; the main LLM is biased toward declaring its own code correct.

H-4. NO MEMORY-WRITE-SKIP.
  After AUDITING returns PASSED, the main LLM MUST dispatch the memory-writer agent. The main LLM does NOT edit memory-bank/ files directly. Period.

H-5. AGENT DISPATCH IS A REGULAR TOOL CALL.
  When in doubt about "is dispatching an agent worth the overhead": the answer is YES. The overhead is real but bounded (one Agent tool call). The cost of skipping is unbounded — the main LLM accumulates dirty context from reading source files that the sub-agent would have summarized.

H-6. BACKGROUND-DISPATCH IS ENCOURAGED.
  If multiple agents have no inter-dependency (e.g. context-compiler + 2 integration-explorers + spec-grounder for the same sub-phase), dispatch them in ONE Agent-tool-call block with multiple invocations so they run in parallel. Up to parallel_cap (default 3, ELAI override 3). This is the SINGLE BIGGEST throughput win on orchestrate runs.

H-7. THE "MANY PLANS QUEUED" TRAP.
  Scenario: user says "orchestrate all 15 E-plans". The main LLM thinks: "if I dispatch 4 agents per plan, that's 60 Agent calls, too many."
  Reality: that's exactly the right number. The main LLM context stays clean across all 15 plans because each agent return is small + summarized. Skipping the agents to "save calls" pollutes the main context with raw source code until the controller can't track its own state. THIS IS THE FAILURE MODE THIS RULE EXISTS TO PREVENT.

H-8. WHEN THE CANONICAL AGENT IS NOT IN THE Agent TOOL'S subagent_type LIST.
  Per compat/ELAI.md, ELAI's canonical agents (auditor, context-compiler, memory-writer, integration-explorer-elai, scope-planner-elai, synthesizer-elai, fixer, wiring-contract-validator, etc.) are NOT in the Agent tool's subagent_type enum. Dispatch via subagent_type=general-purpose with the prompt:
    "Read F:\.ELAI_workflow\.claude\agents\<name>.md and follow it strictly. <task-specific args here>."
  This is the canonical invocation for ELAI agents. Do NOT ad-lib agent instructions; do NOT read the agent .md file yourself and paraphrase its instructions into the prompt.

H-9. SELF-CHECK BEFORE EVERY Edit / Write IN ORCHESTRATE STATE.
  Mental model: "before this Edit, run this checklist:
    1. Am I inside an orchestrate run ? (orchestrate skill fired in this thread)
    2. If yes, which sub-phase am I in ?
    3. Have the context-compiler + integration-explorer returns landed for this sub-phase ?
    4. If no, STOP. Dispatch them first."
  This applies even mid-stream. If the main LLM realizes mid-task it skipped the batch, it MUST stop, dispatch agents, and ONLY then resume Edits.

H-10. EXCEPTIONS (the only legal skip paths).
  E-1. The user, in the current message, explicitly says "skip [agent] this time" or "do it yourself". Logged in orchestrate-run.md as a USER_OVERRIDE.
  E-2. patch-N single-file isolated fixes (per CLAUDE.md patch-N protocol). These are NOT orchestrate runs.
  E-3. The sub-phase's only deliverable is a documentation file (e.g. workflow/reasoning/E-N-decisions.md). No code, no tests, no wiring — just prose. context-compiler can be skipped; impact-mapper / integration-explorer are not needed for prose.
  There are no other legal skips.

  SAFETY-CRITICAL CARVE-OUT (J-70 managerial independence / NASA-STD W11):
  A changeset is safety-critical when it touches ANY of:
    (a) a path listed in .claude/gate-manifest.txt (a frozen gate file),
    (b) a privileged Supervisor-plane function (any symbol under the Supervisor
        plane allowlist / privileged dispatch surface documented in the plan
        corpus; if uncertain, treat orchestrator-owned gate/evaluator paths as
        Supervisor-plane),
    (c) a module marked P10-SAFETY-CRITICAL (power-of-ten / safety-critical tag
        in the crate or rule surface).
  For a safety-critical changeset, E-1 / E-2 / E-3 do NOT authorize skipping the
  auditor. An audit-skip requires a SEPARATE explicit user confirmation in the
  CURRENT message that names the safety-critical paths and the word
  "confirm-safety-critical-audit-skip". Log that confirmation in
  orchestrate-run.md as SAFETY_CRITICAL_AUDIT_SKIP. Without that confirmation,
  the skip is REFUSED even if E-1/E-2/E-3 would otherwise apply.
  This definition is frozen with the gate (R8): a run must not quietly shrink it.

H-11. INTEGRATION GATE IS NOT OPTIONAL (wiring-enforcement.md R10). Per-sub-phase green is necessary-not-sufficient: each sub-phase is audited in isolation on the easy path, so seam defects between sub-phases pass every sub-phase audit. The phase-end sweep MUST therefore include (a) the off-agent language-appropriate wire checker over the changeset (.rs -> production_caller_check.py; .py -> python_wire_check.py) — a DARK symbol with no WIRE-DARK row is a BLOCK — and (b) a cross-seam INTEGRATION smoke driving the REAL entrypoint on a HARD input (strict peer/provider, multi-turn, a run long enough to fire secondary paths), not a toy happy-path. When parallel sub-phases touch a shared resource, the orchestrator threads the contract (denylist / schema / whitelist) as a required input to each. Skipping the integration gate because "every sub-phase audited green" is the exact failure R10 exists to prevent.

ENFORCEMENT

This rule loads automatically via .claude/rules/conditional-routing.md (entry to be added: "Orchestrate skill triggered in the current message thread: load .claude/rules/orchestrate-protocol.md").

The Stop hook checks for the marker "orchestrate skill fired this thread" in workflow/sessions/<sid>/orchestrate-run.md and BLOCKS Stop with a P0 message if any sub-phase in the run shipped Edits without a preceding agent dispatch (parsed from the same log).

When the main LLM is about to call Edit/Write inside an orchestrate run, it SHOULD output one sentence first that names the sub-phase + cites the most recent agent return line in orchestrate-run.md. If it can't, that's the H-9 failure signal — stop and dispatch.

WHY THIS FILE EXISTS

s-orch-e33 (this commit's session): the main LLM ran orchestrate D:\...\E-17 with 15 follow-on E-plans queued in handoff. On E-33, it skipped context-compiler + integration-explorer + auditor + memory-writer entirely and wrote 5 Svelte components + 2 orbit edits + tokens.css + WindowHost.svelte by itself, citing "save context budget across 15 plans" — a hallucinated optimization. User caught it 7 tool calls in. Per CLAUDE.md "no folding under pushback" rule, the main LLM acknowledged + escalated to this rule rather than continuing the pattern.

Prior incident: TBD (user reports "second time this has happened"). If the prior incident has its own session id, link it here when known.

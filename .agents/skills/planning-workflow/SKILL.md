---
name: planning-workflow
description: Full planning, FSM execution, TDD, and auditing workflow. Triggers when creating plans, starting phases, improving/refining phases, executing implementation, or completing phases. Covers search-and-plan, plan improvement (per-phase grounding and adversarial injection), FSM state machine, phase completion, and context exhaustion handoff.
license: internal
compatibility: "Requires .elai_cc/ layout, local-rag MCP server, FSM state machine"
metadata:
  version: "1.0.0"
  author: "elai"
---

Search and plan

After the user approves the prompt, do not write any code yet. Call mcp_fsm_transition("PLANNING") to enter plan mode.

1. Context: Spawn context-compiler agent (.Codex/agents/context-compiler.md) with task and session_id. The agent writes the context package to workflow/sessions/{session_id}/context-package.md and returns only a confirmation + any validation warnings. VALIDATE: confirmation must say "Context package written to". If missing or agent reports errors, re-spawn with a more specific task description. The main model does NOT read the context-package.md file — the prompt-engineer reads it directly. Use grep or mcp_search_context only if the agent reports gaps in its validation warnings.
1b. Prompt transform: Spawn prompt-engineer agent with task and session_id. The agent reads workflow/sessions/{session_id}/context-package.md directly and produces the structured prompt. Do NOT pass context package content inline. Wait for user approval of the transformed prompt before proceeding to plan creation.
2. Create plan:
   - First: call mcp_search_context("plan-template-[task-type]") where task-type is the closest match (feature, bugfix, refactor, trading-pipeline). If a cached template is returned, use its phase structure as the skeleton — adapt phases, keep the structural pattern. Skip if no match.
   - Then: read memory-bank/plans/TEMPLATE.md. Follow its structure and rules exactly. Write to memory-bank/plans/[descriptive-name].md.
   - Step A (Impact Briefing): Use the blast radius from the context package. Supplement with Data Contract Validation (verify changes do not silently violate external APIs) and Edge Case Enumeration (behavior when data is null, disconnects, or exceeds bounds).
   - Step B (Phase Breakdown): Aggressively decompose logic into isolated, single-responsibility files (around 100 lines Domain-Driven Design). Do NOT plan monolithic files. Each phase has numbered steps, files to touch, and expected outcome. Break phases so each one is independently testable.
   - Phase granularity rules:
     - Maximum 3 deliverables per phase. If a phase has more than 3, split it. A deliverable is one independently testable unit (one module, one class, one subsystem).
     - Each phase must be completable within a single session (implementation + tests + audit). If you estimate more than 5 files touched, split the phase.
     - Every phase must have a clear checkpoint sentence: "Phase N is done when [specific observable outcome]." This sentence is what the next session reads to verify state.
     - Sub-phase markers: if a phase has steps 1-3 and step 2 is done when context runs out, the plan must support resuming at step 3 without re-reading steps 1-2. Number steps explicitly and mark each done individually.
   - Test classification rules:
     - Separate tests into two categories: UNIT (runs in under 10 seconds, no network, no real data, deterministic) and OPERATIONAL (requires live data, takes minutes/hours, non-deterministic). Unit tests gate the phase. Operational tests are scheduled separately and never block phase progression.
     - Every phase must have at least one adversarial test: inject bad input, verify graceful degradation.
     - Every phase that processes data must have at least one property-based test: randomized inputs, assert structural invariants hold (sums to 1.0, always finite, always in bounds).
     - No test may depend on real network connectivity to pass. Mock external dependencies in unit tests. Use recorded/synthetic data.
3. Plan-level audit (skip for single-phase plans):
   Spawn adversarial-auditor agent with mode="plan" and the plan file path. The auditor evaluates:
   - Feasibility: can each phase complete in one session?
   - Completeness: all dependencies identified? Missing phases?
   - Domain rule compliance: does the plan violate any applicable domain rule?
   - Testability: is every phase independently testable?
   - Scope realism: given phase count and file count, will this fit?
   ReAct loop (max 3 cycles per attempt):
   a. If auditor finds flaws: read findings, propose fix (max 3 sentences per flaw), revise plan, re-audit.
   b. If flaws persist after 3 cycles: STOP. Present all findings to user.
      "The plan has recurring issues: [list]. Tried: [approaches]. Recommend: [alternative]."
   Session cap: max 2 full plan creation attempts. After both fail, save to SESSION_HANDOFF.
4. Show the plan. Give a plain-language estimate of complexity per phase (file count, system risk, unknowns).
5. Ask: "Plan saved to memory-bank/plans/[name].md. [N] phases: [one-line summary each]. Research and planning have consumed context -- check the sidebar for remaining %. Implement all phases now, or break it down?"
6. Paper-grounding check (before TESTING_FIRST): For each phase that implements an algorithm from academic literature, check that workflow/papers/<algorithm-slug>-interface.md exists. If missing: spawn paper-ingestor agent (.Codex/agents/paper-ingestor.md) with mode=paper and the algorithm name or arXiv ID. Block transition to TESTING_FIRST until all required interface maps exist. If the phase works from decompiled code or third-party source, spawn paper-ingestor with mode=source.
7. Wait for user approval. Do not proceed until the user approves.
8. Grounding sweep (after approval, before TESTING_FIRST): Main model scans the approved prompt for ungrounded claims about external behavior. For each claim:
   a. GROUNDED — context-compiler already verified from source. No action.
   b. VERIFY — grep/read the actual code or run a quick check (e.g., python -c "import X; print(X.func.__doc__)"). Confirm or correct the claim.
   c. INGEST — complex source needs full ingestion. Spawn paper-ingestor with mode=source (local files, SDK) or mode=paper (academic). Block until interface map exists.
   Write results to workflow/reasoning/phase-X-grounding.md. Block TESTING_FIRST until all claims are GROUNDED or VERIFIED.

Plan improvement (per-phase refinement)

Trigger: user says "improve phase X", "refine phase X", "ground phase X", or "improve step X.Y".
Scope: operates within PLANNING state. No new FSM state required.
Constraint: main model reads the step file directly (this is plan content, not memory-bank or context-package). Context delegation rule does not apply to docs/specs/guide/steps/*.

Protocol (execute in order for each phase/step):

1. READ PHASE: Read docs/specs/guide/steps/phase-{X}.txt. Parse each numbered step, its READ directives, risks, tests, and exit criteria.

2. SPEC VERIFICATION: For each READ directive in the phase:
   a. Verify the file exists (Glob).
   b. Grep for the referenced section/content to confirm it is non-empty and relevant.
   c. If missing or empty: flag as SPEC_GAP. Log in workflow/reasoning/phase-{X}-grounding.md.
   d. If the spec contradicts the phase description: flag as SPEC_CONFLICT.

3. CLAIM GROUNDING: For each factual claim in the phase (compile times, LOC counts, test counts, API behaviors, algorithm properties, library capabilities):
   a. Classify: GROUNDED (already verified by test/code), VERIFY (can check now via grep/bash/read), INGEST (needs paper-ingestor or source-grounding), UNGROUNDED (no evidence available).
   b. VERIFY claims: run the check (e.g., cargo check, tokei, grep). Record actual value. If claim is wrong, correct it in the phase file.
   c. INGEST claims: spawn paper-ingestor agent with mode=paper or mode=source. Block until interface map exists in workflow/papers/.
   d. UNGROUNDED claims: mark explicitly in the phase file as "UNGROUNDED: [claim]. Assumed because: [reason]." Do not remove — make the assumption visible.
   e. Write all results to workflow/reasoning/phase-{X}-grounding.md.

4. PRE-GATE INJECTION: For each numbered step in the phase, add a PRE_GATE block:
   - What must be true before this step starts.
   - Each gate is a deterministic check (command that returns 0/non-zero, grep that returns hits/no-hits, file that exists/doesn't).
   - No subjective criteria. "Code looks clean" is not a gate. "ruff check returns 0 errors" is.
   - WALKING-SKELETON PRE_GATE (mandatory for any step that ships code; see .Codex/rules/wiring-enforcement.md): every step is a vertical slice. REFUSE a step whose deliverable is a producer (a new function/route/handler/variant) without, IN THE SAME STEP, its real production consumer AND a smoke trigger that drives a real entrypoint AND a declared off-agent artifact (an event row / coverage hit / span the agent did not author). "Add fn X" is rejected at decomposition time; the only acceptable form is "add fn X AND call it from real entrypoint Y AND assert artifact Z lands." A producer-only step is split-or-merged so producer+consumer+smoke land together. This is prevention; the wiring gate at AUDITING is the backstop, not the first line.

5. DETERMINISTIC CHECK INJECTION: For each numbered step, add a DETERMINISTIC_CHECK block:
   - How to prove the step succeeded without human judgment.
   - Each check is a command, assertion, or file-existence test.
   - Must be runnable by the auditor agent in RUN_TESTS sub-state.

6. ADVERSARIAL INJECTION: For each deliverable in the phase:
   a. Identify top 1-3 failure modes (what could go wrong even if the code compiles and tests pass).
   b. For each failure mode, define an adversarial test: bad input, edge case, race condition, or environmental assumption that breaks.
   c. Add these as ADVERSARIAL blocks in the step. Format: "If [assumption] is wrong, [consequence]. Test: [deterministic check]."
   d. For phases with 3+ deliverables: spawn adversarial-auditor agent with mode="plan" and the phase file path for independent review. Merge its findings.

7. ACCEPTANCE CRITERIA: For each deliverable, add measurable pass/fail criteria under ACCEPTANCE block:
   - Binary (pass/fail), not graded.
   - Tied to a specific command, test, or metric threshold.
   - These become the exit criteria the auditor checks during AUDITING state.
   - OFF-AGENT PROOF (mandatory for any feature deliverable; .Codex/rules/wiring-enforcement.md R4): "compiles + unit test passes" is NOT acceptance — a unit test calling the new function directly is indistinguishable from a missing production caller. Acceptance for a shipped feature MUST name an artifact the implementing agent did not author, produced by a real entrypoint run: an event/SSB row, an LLVM coverage hit on the new function from the compiled binary, or an in-process span. Missing that artifact is FAIL, not UNVERIFIED. If the feature ships behind a default-OFF flag, acceptance also requires a registered row in wire-dark-allowlist.toml (R9).

8. REWRITE: Produce the improved phase file. Replace the original in docs/specs/guide/steps/phase-{X}.txt.
   Preserve the original structure (GOAL, READ BEFORE IMPLEMENTING, RISKS, STEPS, EXIT CRITERIA, MEASUREMENTS).
   Add within each step: PRE_GATE, DETERMINISTIC_CHECK, ADVERSARIAL blocks.
   Add per-deliverable: ACCEPTANCE block.
   Add top-level: GROUNDING section (summary of claim verification results, with UNGROUNDED items listed).

9. LOG: Write/update workflow/reasoning/phase-{X}-grounding.md with full grounding results.
   Format per claim: "[claim text] | [GROUNDED|VERIFIED|INGESTED|UNGROUNDED] | [evidence or reason]"

Output to user after improvement:
  "Phase {X} improved. Changes: [N] claims grounded, [N] pre-gates added, [N] deterministic checks added, [N] adversarial tests added, [N] ungrounded assumptions flagged. Review: docs/specs/guide/steps/phase-{X}.txt"

Iterating: user can run "improve phase X" multiple times. Each run re-verifies, picks up newly available evidence, and tightens checks. Idempotent — running twice with no new information produces no changes.

Improvement scope: "improve" means more than grounding verification. After the grounding sweep, do a substantive pass: search for real implementations using the same tech stack, find patterns/pitfalls we're missing, check if our design matches proven approaches. Every addition must be deterministic — cite real evidence (docs, implementations, papers). No speculative additions. Grounding is necessary but not sufficient.


Execution (FSM-enforced)

Once the user approves, call mcp_fsm_transition("TESTING_FIRST") to begin TDD. Before writing any files, call mcp_session_claim(owns="files-you-will-edit", locks="blast-radius-files") to register file ownership. If conflict returned, tell the user and adjust scope. The FSM orchestrator enforces execution discipline via Codex hooks. A pre-tool-call hook blocks disallowed actions per state and cross-session file locks. Call mcp_fsm_get_state at phase start to see current state and constraints.

FSM states and flow:
IDLE -> PLANNING -> TESTING_FIRST -> EXECUTING -> AUDITING -> MEMORIZING -> COMMITTING -> PLANNING (next phase) or COMPLETE -> IDLE
IDLE -> CONFIGURING -> IDLE | PLANNING
Additional transition: AUDITING -> PLANNING (dead-end declaration by auditor)

- IDLE: default state. No tools blocked. Full access for ad-hoc work. Call mcp_fsm_transition("PLANNING") when starting a plan, or mcp_fsm_transition("CONFIGURING") for workflow/infrastructure changes.
- CONFIGURING: all tools allowed. For meta-work on the development workflow itself: agents, hooks, skills, AGENTS.md, settings, memory-bank templates, FSM config. Not for project source code. Only hard block: .fsm-state.json direct edit (use transition API). When done: mcp_fsm_transition("IDLE") to return to ad-hoc, or mcp_fsm_transition("PLANNING") to pivot to project work.
- PLANNING: research and plan only. Write/Edit allowed ONLY to: memory-bank/plans/*, workflow/* (except .fsm-state.json), memory-bank/*.md. Blocked for src/ and tests/. Call mcp_fsm_transition("TESTING_FIRST") when ready to write tests.
- TESTING_FIRST: First, spawn auditor to run existing contract tests (python -m pytest .Codex/tools/tests/conformance/ -v) as a pre-phase health check. If any fail, fix before proceeding. Then write phase-specific tests (TDD). Also write/update contract tests in .Codex/tools/tests/conformance/ for any modules this phase touches, using the interface map from the context package. Every test function MUST have a pytest marker: @pytest.mark.unit, @pytest.mark.integration, @pytest.mark.stress, or @pytest.mark.conformance. Unmarked tests default to @pytest.mark.unit. This is required so the pipeline runs the right tests at the right tier. Do NOT run tests. Call mcp_fsm_transition("EXECUTING") when tests are written.
- EXECUTING: First, call mcp_think_set_budget with the GRC budget from the transformed prompt (or default: 3 calls, 1000 chars). Then create a decision doc at workflow/reasoning/phase-X-decisions.md:
    What I will implement: [3 sentences max]
    Key technical decision: [1 sentence]
    Expected test outcome: [1 sentence]
    Unknowns to verify empirically: [list, max 3]
  This is soft enforcement — no FSM gate blocks if missing, but the auditor checks for it.
  Use mcp_think(observe/hypothesize/plan_action) with grounding before each edit. Then write implementation code. Do NOT run tests. The adversarial auditor is the sole test runner. Call mcp_fsm_transition("AUDITING") when implementation is complete.
- AUDITING: application code is read-only. The auditor agent (.Codex/agents/auditor.md, model: sonnet) is the sole test runner. It runs the tiered validation pipeline (python .Codex/tools/pipeline/run_pipeline.py --tiers 1,2,3,4,5 -v) BEFORE reviewing code — NOT flat pytest. If any tier fails, immediate BLOCK. It can also write and run verification scripts in workflow/audit-scratch/ and run linters (ruff, mypy). It can use WebSearch/WebFetch to verify algorithms and formulas. The pre_tool_check hook enforces path restrictions. In your prompt to the agent, include: the task domain (trading, game, dashboard, application, or general), the plan file path, and the current phase number.
  AUDITING outcome protocol (follow top-to-bottom, stop at first match):
  1. Read audit results completely.
  2. Read the PROGRESS SUMMARY line (always the first line of audit output):
     "PROGRESS SUMMARY: Tests: X/Y passing | Tiers: X/5 passed | TestQuality: X.XX | Regression: [module] | BLOCKs: N | Score: X.XX"
     Note the Regression module — fix that file first in any EXECUTING pass.
  3. Contains "DEAD END"?
     -> mcp_fsm_transition("PLANNING", dead_end_reason="[category]: [reason]").
        Git checkout phase files. STOP.
  4. Contains any "BLOCK" finding?
     -> Call mcp_fsm_transition("EXECUTING").
        Fix BLOCKs in this order: Regression module first, then remaining BLOCKs by severity.
        Fix ALL BLOCKs in a single EXECUTING pass. Do NOT re-transition to AUDITING. Auditor runs once per phase.
  5. Contains "AUDIT PASSED"?
     -> mcp_fsm_transition("MEMORIZING"). Proceed to memorize.
  6. None of the above: treat as BLOCK. Go to step 4.
- MEMORIZING: Spawn the memory-writer agent (.Codex/agents/memory-writer.md) with a structured change manifest. The agent advances through sub-states READ_TEMPLATES -> WRITE_UPDATES -> MEMORIZE_RAG automatically. VALIDATE its output: it must report every file updated and every tool called. If its report is missing expected updates (e.g., changelog not logged, progress not updated), re-spawn or fix manually. After memory-writer completes: call mcp_memorize with topic="plan-template-[task-type]" and the phase structure summary — this caches it for future plans of the same type.
  MEMORY VERIFICATION (mandatory): After memory-writer reports success, verify the MCP tools actually wrote to the correct files:
  1. Read memory-bank/changelog.md — confirm the phase's file entries appear (grep for the phase ID or file paths).
  2. Read memory-bank/activeContext.md — confirm task/status/next_step reflect the just-completed phase.
  3. Read memory-bank/progress.md — confirm build status is updated.
  If any file is stale (still shows pre-phase content), update it manually. Do not proceed to COMMITTING with stale memory-bank files. Log "MEMORY_VERIFY: [file] was stale, updated manually" if manual fix was needed.
  FEATURE-MAP UPSERT (mandatory when docs/thefinalglance/ELAI-FEATURE-MAP.md exists): the feature map is the SOURCE OF TRUTH for ELAI features, their status, and how they compose. If the completed phase shipped, activated, or changed any feature: the main LLM upserts the touched entries in-place (flip the status flag PLANNED to WIRED or DARK(flag), add/adjust composes/extends cross-links, add new features as new entries in the right subsystem group) and bumps the header date. Rejections/defers discovered during the phase go to the REJECTED/ALREADY-TRIED ledger. This is a first-party tooling-doc edit like the handoff (NOT memory-writer's job, NOT a memory-bank file). One flag flip, never a narrative.
- COMMITTING: Auto-commit phase changes after successful memorization. Steps:
  1. Run git status and git diff --cached/unstaged to identify all changed files.
  2. Stage only files relevant to the completed phase (project source in .elai_cc/, workflow reasoning docs, .gitignore, .githooks/ if changed). Never stage .env, credentials, or secrets.
  3. Generate commit message: "[Phase ID]: [short description of what changed]" as the subject line. Body includes: files changed (grouped by category), key technical decisions, test results (e.g., "cargo test --no-run: 0 errors"). End with Co-Authored-By line.
  4. Run git commit. If pre-commit hook fails, diagnose and fix the issue, then retry (do NOT use --no-verify unless user explicitly approves).
  5. Report commit hash to user.
  After committing: call mcp_fsm_transition("PLANNING") for next phase or mcp_fsm_transition("COMPLETE") if all phases done.
- COMPLETE: read-only. Call mcp_fsm_transition("IDLE") to return to ad-hoc mode.

If an unexpected variable, system state, or dependency is encountered that was NOT anticipated in the plan: HALT execution immediately. Log the unexpected state. Return to PLANNING to revise the plan before continuing.

Always-allowed tools (never blocked in any FSM state): Read, Glob, Grep, Agent, WebSearch, WebFetch, all mcp_fsm_* tools, and all mcp_session_* tools (register, claim, list, release, attribute_build_errors).

Context exhaustion handoff: if the user says context is running low, immediately fill the SESSION_HANDOFF section in memory-bank/activeContext.md before doing anything else. Populate every field: what was in progress, current hypothesis, things tried that failed and why, unresolved questions, exact next steps (numbered), and files that were open. This is the only information the next session gets. Be specific, not generic. Then stop work.

After each completed phase

1. Spawn memory-writer agent with a change manifest containing: task, status, next_step, files_edited (with one-line descriptions), build_status, build_tested, architecture_decisions (if any), new_files (if any), gotcha (if any), and memorize fields. The agent executes all memory-bank updates in its own context window.
2. operatingManual.md: fill any newly confirmed build commands, source paths, log paths, or environment constraints. Populate on first session when values are first known.
3. Mark completed steps in the plan file. Record current position for session handoff.
4. Commit: stage and commit all phase changes with a descriptive message. See COMMITTING state above for full protocol.

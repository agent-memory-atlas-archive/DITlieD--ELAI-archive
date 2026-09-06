orchestrate — deep protocol (loaded by SKILL.md)

This file is the step-by-step body. SKILL.md is the trigger contract. Together they define what happens when the user types "orchestrate".

EXECUTION MODE OVERVIEW

The orchestrate skill runs in TWO modes depending on whether the target project is the same as the host project (apps_workflow) or different:

SAME-PROJECT mode: target == host (= apps_workflow). The current Claude Code session runs the FSM cycle in-process using host's .claude/agents/. No CLI spawn.

CROSS-PROJECT mode: target != host (= e.g. .ELAI_workflow). Host session is the CONTROLLER. It launches a separate `claude -p` session WITH CWD = target project so the target's native skills + agents + rules + MCP load. The launched session is the EXECUTOR. Controller tails stream-json, surfaces tripwires, never makes the user wait.

Both modes share the same protocol below; the difference is just whether step 4 spawns a subprocess or runs in-band.


STEP 0 — TRIGGER PARSE

Input is the raw user message. Parse it with regex to extract:
- plan_path: the first absolute path mentioned that ends in .md or matches B-N pattern (B-30, B-29.5, etc) under a known plan directory
- depends_on: anything after "depends on", "waits on", "after", "blocked by". Tokenize by commas, "and", "+". Values are B-N ids OR session ids (s-12345) OR phase descriptors.
- parallel_cap: default 3, override if user said "max N parallel" or "cap N agents"
- model_executor: default claude-sonnet-4-6, override if user said "use opus" or "haiku"
- autonomous: default true, override if user said "checkpoint", "ask me", "interactive"
- resume_sid: if user said "resume <sid>" extract; otherwise null
- commit_at_end: default true, override if user said "no commits" or "commit per phase"

If plan_path missing AND resume_sid missing: ask user once "which plan ?". This is the ONE pre-flight question allowed.


STEP 1 — TARGET PROJECT + STACK DETECTION

Run tools/detect-stack.ps1 with plan_path as input. The script:
- walks up the directory tree from plan_path looking for `.claude/` folder; that directory is the target project root
- probes the target project root for stack markers (Cargo.toml = rust workspace, package.json + tsconfig.json = typescript, .xcodeproj = swift xcode project, mixed = cross-stack)
- writes detection result to workflow/sessions/<sid>/orchestrate-stack.json with fields: target_project_root, target_project_name (slug), stack ("rust" | "typescript" | "rust+typescript" | "swift" | "rn-expo" | "unknown"), native_orchestration_skill (path if exists, e.g. ".claude/skills/b-plan-runner/SKILL.md"), dashboard_path (if multi-stack detected, e.g. "crates/dashboard-v2"), is_cross_project (boolean: target != apps_workflow)

If stack == "unknown" + no native orchestration skill: surface to user "i can run the generic FSM cycle but the target project has no compat or native skill. Generic mode only. Proceed ?" — this is a soft check, not a tripwire.


STEP 2 — COMPAT LOAD

If compat/<target_project_name>.md exists (e.g. compat/ELAI.md for target slug "ELAI"): load it. The compat file declares:
- native_orchestration_skill_path (relative to target project root)
- native_orchestration_skill_invocation_phrase (e.g. "run b-plan-runner on this")
- override_directives: list of which native skill steps to skip / cap / replace (see overrides-template.md)
- stack_gates: which stack-adapter file(s) to load
- dashboard_wiring_paths: paths that any new user-facing feature MUST touch
- test_commands: per-stack test invocations to use for gate 2 + gate 4
- bindings_emit_command: cross-stack codegen invocation if applicable
- known_dep_files: where ELAI tracks B-N status so orchestrate can read "is B-29 done" from elai-state files

If no compat file: orchestrate falls back to generic mode (skip step 8's native delegation, use the in-house FSM cycle).


STEP 3 — SESSION REGISTRATION (host side, never touch others)

mcp_session_register with task = "orchestrate <plan_path>". This registers the CONTROLLER session in apps_workflow's session registry, NOT the launched executor session.

Read mcp_session_list. Filter for active sessions (heartbeat < 30min). For each session whose owned-files or locked-files overlap with the plan_path's project: log to orchestrate-run.md as "co-tenant" (just informational; doesn't block).

If depends_on is non-empty: log each dep id with its current status (from the co-tenant scan or from compat.known_dep_files).


STEP 4 — DEPENDENCY WAIT

For each dep in depends_on:
- run tools/dep-wait.ps1 with dep_id, max_wait_seconds (default 4 hours), poll_interval (60s)
- the script:
  - polls mcp_session_list every 60s
  - checks compat.known_dep_files paths for dep-status markers (e.g. for ELAI, reads workflow/sessions/*/HANDOFF.md for "B-29 status: DONE")
  - calls Start-Sleep -Seconds 60 between polls (sleep-foreground per user's brief)
  - returns 0 when dep resolved, non-zero on timeout
  - NEVER calls mcp_fsm_transition / mcp_fsm_reset / mcp_session_release against another session's id

If wait > 4h on any dep: TRIPWIRE T3. Surface to user with [keep waiting / abort / proceed-without-dep] options. Default: wait until user answers (do not silently proceed).

When all deps cleared: log to orchestrate-run.md.


STEP 5 — PLAN PARSE

Read plan_path via Read tool (small enough for direct read — B-N files are 200-500 lines).

Parse into structure:
- plan_title (first ## or # header)
- objective (the "Objective:" paragraph if present)
- context (the "## Context" section)
- reads (the "## Reads (mandatory before coding)" list — paths to ground against)
- sub_phases (each "### Phase N" header is a sub-phase). For each sub_phase: extract title, ordered tasks (numbered lines), explicit file paths in scope (CREATE/MODIFY lines under "## Files touched"), tests-to-add references, WIRING_CONTRACT entries that map to this sub-phase
- wiring_contract (the WIRING_CONTRACT block: produces_symbols, consumes_symbols, reachable_from, stub_ban, frontend_pair, observable_effect)
- deterministic_check (the bash block under "## DETERMINISTIC_CHECK")
- acceptance (the "## ACCEPTANCE" checklist)

Write structured parse to workflow/sessions/<sid>/orchestrate-plan.json.

If parse fails (no Phase N headers found, or only one Phase): fall back to "treat the whole plan as one sub-phase". Log as warning, continue.


STEP 6 — BOOT PROMPT GENERATION

Read overrides-template.md. Substitute placeholders with parsed values:
- {{PLAN_PATH}}, {{PLAN_TITLE}}, {{SUB_PHASES_JSON}}, {{TARGET_PROJECT_ROOT}}, {{STACK}}, {{PARALLEL_CAP}}, {{MODEL_EXECUTOR}}, {{COMMIT_AT_END}}, {{AUTONOMOUS}}, {{NATIVE_SKILL_INVOCATION}}, {{STACK_ADAPTER_CONTENT}} (inlined), {{COMPAT_OVERRIDES}} (inlined), {{WIRING_CONTRACT}} (inlined), {{DETERMINISTIC_CHECK_COMMANDS}} (inlined), {{ACCEPTANCE_CHECKLIST}} (inlined), {{SESSION_ID_HOST}} (this session's sid for cross-process coordination), {{DEPS_NOW_CLEARED}} (just for the launched session to know the dep-wait already happened)

Write the filled boot prompt to workflow/sessions/<sid>/orchestrate-boot.md.


STEP 7 — LAUNCH (cross-project mode)

Run tools/orchestrate-launcher.ps1 with:
- -PlanPath <plan_path>
- -TargetProjectRoot <target_project_root>
- -BootPromptPath workflow/sessions/<sid>/orchestrate-boot.md
- -SessionIdHost <sid>
- -ResumeSid <executor session id if resuming, else empty>
- -StreamJsonOutput workflow/sessions/<sid>/orchestrate-stream.jsonl

The launcher:
- pushes to target_project_root
- assembles `claude -p "$(cat <boot-prompt>)" --output-format stream-json --include-partial-messages --verbose` (resume flag if applicable)
- redirects stdout to orchestrate-stream.jsonl, stderr to orchestrate-stream-err.log
- runs synchronously; parent (this skill) waits for the launched process to exit
- BUT also tails the stream-json file in a parallel job and pipes parsed events back to the controller via a temp file (orchestrate-events.log)

In SAME-PROJECT mode (apps_workflow target): SKIP this step. Instead, the controller (this session) directly executes the protocol below in-band, spawning agents via the Agent tool.


STEP 8 — EXECUTION (loop per sub-phase)

This is what the launched session executes (cross-project) OR what this session executes in-band (same-project).

For each sub_phase in plan.sub_phases:

  8.1 PRE_GATE: check stub-tracking.md style consistency (every open stub with will_resolve_in <= current sub-phase id must be closable by this sub-phase OR explicitly deferred in plan)

  8.2 mcp_fsm_transition("PLANNING")

  8.3 PARALLEL BATCH (cap PARALLEL_CAP, default 3) in ONE Agent-tool message block:
      - subagent: context-compiler (per target's .claude/agents/context-compiler.md if exists, else fall back to apps_workflow's)
        prompt: "scope context to sub-phase {sub_phase_id} of {plan_path}. files in scope: {sub_phase.files}. write workflow/sessions/<sid>/context-package-{sub_phase_id}.md"
      - subagent: integration-explorer (or integration-explorer-elai if target is ELAI)
        prompt: "explore subsystem {subsystem_inferred_from_files} for wiring needed by sub-phase {sub_phase_id}. write workflow/sessions/<sid>/exploration/{sub_phase_id}-{subsystem}.md"
      - subagent: integration-explorer (second one if sub-phase touches > 1 subsystem)
        prompt: "explore subsystem {second_subsystem}..."
      Await all returns.

  8.4 SKIP prompt-engineer (per user override, this step is OFF in orchestrate).

  8.5 EXECUTING phase
      mcp_fsm_transition("EXECUTING")
      The session implements the sub-phase's tasks per its numbered list (Phase 1 tasks 1-3, Phase 2 tasks 4-10, etc).
      Wire/integrate dashboard if compat.dashboard_wiring_paths declared AND sub-phase touches user-facing surface.
      Use grounded reads from plan.reads — these are mandatory inputs.
      No force-unwrap, no silent catch, no unregistered stubs (per token-discipline.md zero-lazy-code rule).
      Test with MODEL_EXECUTOR (sonnet-4.6 cloud per user override) where the project's tests support it.

  8.6 AUDITING phase (1 CYCLE, no auto-retry)
      mcp_fsm_transition("AUDITING")
      Spawn auditor (target's .claude/agents/auditor.md):
        plan_file: {plan_path}
        sub_phase_id: {sub_phase_id}
        deterministic_check_commands: {plan.deterministic_check restricted to this sub-phase's tests}
        acceptance_subset: {plan.acceptance items mapped to this sub-phase}
      Read auditor return. Three outcomes:
        - AUDIT PASSED > continue to 8.7
        - BLOCK findings > apply bubble-up.md decision table. NO automatic second audit cycle. RetryBabyStep up to 3 attempts (a "baby step" here = the failing task within the sub-phase). After 3 retries at this granularity, RedecomposeBabyStep > the sub-phase splits.
        - DEAD END > TRIPWIRE T4 if the auditor cited algorithm / platform_compliance category. Otherwise ReplanPhase per bubble-up.md.

  8.6.1 If sub-phase touches the WIRING_CONTRACT: spawn wiring-contract-validator (if target is ELAI and the agent exists) with bn_file + sub_phase_id. If FAIL: spawn fixer once, re-validate ONCE (per b-plan-runner step 10 pattern), then surface.

  8.7 MEMORIZING phase
      mcp_fsm_transition("MEMORIZING")
      Spawn memory-writer (target's .claude/agents/memory-writer.md) with structured change manifest scoped to this sub-phase ONLY (do NOT roll up the whole plan yet — that happens at step 9).

  8.8 PHASE-END SWEEP (per sub-phase)
      Run tools/phase-end-sweep.ps1 with --sub-phase {sub_phase_id} --stack {stack} --compat {compat_path}.
      Six generic gates + stack-specific gates per stack-adapter.
      If any gate fails: BLOCK and bubble-up. 3 sweep failures in a row = TRIPWIRE T4.

  8.9 STALL DETECTOR (Magentic-style)
      Track no-progress counter across sub-phases. A sub-phase counts as "no progress" if zero new files touched AND zero new tests passing AND zero stubs closed.
      3 consecutive no-progress sub-phases = TRIPWIRE T4 even if auditor said PASSED on each (Devin "infeasible path" failure mode prevention).

  8.10 Advance sub_phase_index. Loop.


STEP 9 — FINAL INTEGRATION SWEEP (after all sub-phases done)

Run tools/phase-end-sweep.ps1 with --full --plan {plan_path} --stack {stack} --compat {compat_path}.

This runs gates at the FULL plan level:
1. Build green (full workspace, all crates / packages)
2. Test suite green (full suite, all crates / packages) + delta-covered (every public symbol from WIRING_CONTRACT.produces_symbols has ≥1 exercising test)
3. Dep-graph integrity (no new cycles or orphans across full workspace)
4. Public-API surface diff (every WIRING_CONTRACT.produces_symbols entry has consumers OR is registered intentional surface)
5. Manifest + lockfile consistency
6. Documentation drift (every produces_symbols entry mentioned in changelog OR systemPatterns)
+ stack-specific gates from stack-adapters/<stack>.md
+ cross-stack gate if multi-stack (ts-rs / specta / wasm-bindgen bindings drift)
+ dashboard wiring check (every new user-facing feature touches compat.dashboard_wiring_paths)
+ stub ban (per WIRING_CONTRACT.stub_ban — e.g. for B-30: LanguageRegistry.loaded.len() >= 10, no silent fall-through)
+ observable_effect (per WIRING_CONTRACT.observable_effect — run the trigger, verify expect_ssb + expect_artifact)

If any gate fails: BLOCK. bubble-up applies. Up to 3 retries at full-sweep level (each retry spawns fixer agent scoped to the failing gate). After 3 = TRIPWIRE T4.


STEP 10 — REAL-MODEL CLOUD TEST PASS

Per user override: run the full test suite once more under MODEL_EXECUTOR (sonnet-4.6 cloud) instead of any local mock. This catches "tests pass with mocks, fail in real environment" regressions per failure-mode research (SWE-Bench Pro premature completion).

For ELAI: `cargo nextest run --workspace --release` (reads .config/nextest.toml to bound heavy-test concurrency; `cargo test --workspace --release` if cargo-nextest absent) then `cd .elai_cc/crates/dashboard-v2 && npm test`.
For apps_workflow: project-specific (jest for RN, xcodebuild test for Swift).

Test command lives in compat.test_commands.

If real-model test fails: bubble-up. 3 retries max. Then TRIPWIRE T4.


STEP 11 — SINGLE COMMIT

mcp_fsm_transition("COMMITTING")

git status / git diff (in target project, not apps_workflow).

Stage only files relevant to this orchestrate run (per plan.sub_phases[*].files union).

Commit with maintainer voice per .claude/rules/commit-cadence.md:
- "B-30: multi-language DKG hardening, registry + 11 grammars + 4 resolvers + composite re-parse + dashboard panel"
- body: brief per-phase summary (one line each), test deltas, key decisions
- NO Co-Authored-By trailer (CLAUDE.md hard rule)

Report commit hash. NO git push (push is irreversible — TRIPWIRE T1 would fire).


STEP 12 — CLEANUP + REPORT

mcp_session_release() — releases apps_workflow controller session

mcp_fsm_transition("COMPLETE") then mcp_fsm_transition("IDLE").

Write final report to workflow/sessions/<sid>/orchestrate-final.md with:
- plan_title, plan_path
- duration (wall clock)
- sub-phases completed, each with status + retries + auditor verdict
- final sweep result
- commit hash
- token usage (parsed from stream-json result events)
- any tripwires hit + how resolved
- known followups (open stubs that will_resolve_in a later orchestrate run)

Tell the user: ONE-paragraph summary. plan title + duration + commit hash + any followups. No questions, no "what next".


STATE FILES (canonical paths)

workflow/sessions/<sid>/orchestrate-run.md           — live run log (controller writes)
workflow/sessions/<sid>/orchestrate-state.json      — FSM + cursor (sub-phase index, retry counts)
workflow/sessions/<sid>/orchestrate-plan.json       — parsed plan structure
workflow/sessions/<sid>/orchestrate-stack.json      — stack detection result
workflow/sessions/<sid>/orchestrate-boot.md         — filled boot prompt (sent to launched session)
workflow/sessions/<sid>/orchestrate-stream.jsonl    — launched-session stream-json output
workflow/sessions/<sid>/orchestrate-events.log      — parsed events tail
workflow/sessions/<sid>/orchestrate-final.md        — final report
workflow/sessions/<sid>/phase-reports/<sub_phase_id>.md   — per-sub-phase end report
workflow/sessions/<sid>/context-package-<sub_phase_id>.md — per-sub-phase context
workflow/sessions/<sid>/exploration/<sub_phase_id>-<subsystem>.md — per-sub-phase wiring report
workflow/sessions/<sid>/SESSION_HANDOFF.md          — only on abort / PreCompact / tripwire


RESUME PROTOCOL

User types "orchestrate resume <sid>". Skill:
1. Read workflow/sessions/<sid>/orchestrate-state.json to know last completed sub-phase
2. Read workflow/sessions/<sid>/orchestrate-plan.json (already parsed)
3. Read workflow/sessions/<sid>/SESSION_HANDOFF.md (failure context if any)
4. Skip steps 0-6 (parsing already done)
5. Re-run dep-wait for any deps that may have come or gone
6. Jump to step 7 / 8 with sub_phase_index = state.last_completed + 1
7. If launched session sid is in state.json: pass --resume <executor_sid> to claude -p


ANTI-SPIRAL INTEGRATION

If target is ELAI: ELAI's anti-spiral skill fires automatically in the launched session (it's in ELAI's .claude/skills/). Orchestrate does NOT need to duplicate. Just observe the stream-json for persona-switch events and log to orchestrate-run.md.

If target has no anti-spiral skill: orchestrate inlines a minimal stall detector in the boot prompt — same-file edited >3 times for the same error = pause and replan within the current sub-phase.


TRIPWIRE HANDLING DETAIL

T1 IRREVERSIBLE OP: launched session's PreToolUse hook on Bash should be configured to surface to controller. If not configured, orchestrate-launcher.ps1 watches stream-json for tool_use events matching forbidden patterns (git push --force, rm -rf, npm publish, cargo publish, db drop). Match = controller pauses the launched session via SIGSTOP, surfaces to user with the exact command + sub-phase context.

T2 50-EVENT BUBBLE-UP CAP: orchestrate-launcher.ps1 counts bubble-up events from stream-json (parsing for the bubble-up signal patterns). At 50, force abort.

T3 DEP-WAIT > 4h: tools/dep-wait.ps1 returns non-zero. Controller surfaces.

T4 SWEEP / STALL / DEAD END: classified per the trigger. Controller surfaces with diagnostic from the failing gate / stall pattern / dead-end category.


END protocol.md

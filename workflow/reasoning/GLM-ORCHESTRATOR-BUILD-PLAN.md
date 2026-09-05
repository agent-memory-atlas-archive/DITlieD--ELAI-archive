GLM-5.2 ORCHESTRATOR — BUILD PLAN
================================

Companion to: workflow/reasoning/SIMPLE-GLM-ORCHESTRATOR-GUIDANCE.md (intent/north-star)
Harness facts grounded from: docs/.source/.../devin-bridge/RUN-GLM-5.2.md + a full read of
  devin-bridge (claudedevin.ps1, devin_proxy.py, devin_wire.py) + the relevant cli.js
  resolvers + b-plan-runner/orchestrate SKILLs + the frontend-wiring handoff.
Authored: 2026-06-29. Plain text, no markdown decoration (house output-formatting rule).

CORRECTION 2026-06-30 (architecture was WRONG): an earlier draft made the CONTROLLER the
Opus main LLM running a glm-orchestrate skill. THAT IS WRONG. The controller is a
DETERMINISTIC PYTHON LOOP (orchestrator.py). Opus = 0: not in the loop, not on a timer, not
making routing calls. EVERY LLM call is one of the (up to 4 concurrent) free GLM-5.2 workers
(context, execute, audit, review, polish, commit-msg, memory). The loop routes PURELY on
SCRIPT EXIT CODES (wired? did the wiring/done-gate scripts exit 0?), never on model
judgment. ARCHITECTURE + PHASE 2 below are rewritten to this. The registered glm-orchestrate
SKILL.md (Opus-controller shape) is SUPERSEDED by orchestrator.py and DELETED.

BUILT STATUS + REDESIGN CONSEQUENCE (2026-06-30): a prior session already BUILT Phases 0 + 1
to the WRONG Opus-controller premise and verified them (recorded in the now-deleted SKILL.md +
run-state.json): the launcher devin-bridge/glm-worker.ps1 (-Lane a/b, ports 8810/8811, forces
-Model glm-5-2 + CLAUDE_CODE_DISABLE_CLAUDE_MDS=1); the lean git guardrail
.claude/tools/glm_worker_git_guardrail.py REGISTERED as a PreToolUse Bash hook in
.claude/settings.json (27 deny / 21 allow tested); and the spine workflow/glm-orchestrator/
{plan_walk,plan_parser,plan_status,queue}.py + run-state.json (499 walk units: 39 step files
then 460 B..L plans; first=pre-phase-0, last=L-15). VERIFIED GLM-5.2 CAPABILITY MATRIX:
PASS = Read, Edit, Bash, Grep, Glob, TodoWrite, Task-subagents, codegraph-elai MCP;
FAIL (environment) = local-rag MCP, WebSearch.
CONSEQUENCE OF Opus=0 (the rework PHASE 2 must do): the built design routed every MCP/RAG/web
grounding step through the Opus controller (PHASE A2 codegraph pre-check, S11 mcp_memorize, any
mcp_search_context). With NO Opus in the loop those move OFF the controller: codegraph-elai
works fine dispatched to a GLM worker, BUT local-rag + WebSearch FAIL on GLM, so the S11 memory
RAG-index (and any web-grounding) needs a NON-LLM path — orchestrator.py calls the local-rag
backend directly (HTTP/CLI), or that step is dropped. The launcher also extends 8810/8811 ->
8810-8813 for the 4-worker pool, and the built run-state.json / status records encode
max_concurrent_glm:2 + Opus-controller assumptions that PHASE 2 must re-audit.

This plan has 6 phases (PHASE 0..PHASE 5). Each is sized as ONE unit for a single Opus
agent with a 1M context window: self-contained, with its own deliverables, the exact files
to create/edit, grounded anchors, and an acceptance bar. Phases are mostly sequential
(0 -> 1 -> 2 -> 3 -> 4 -> 5); PHASE 3 and PHASE 4 can run in parallel after PHASE 2.

----------------------------------------------------------------
NORTH STAR (unchanged from the guidance)
Prevent slop with good context + "edit, don't recreate" + a focused wiring gate. Walk the
plans in human order. Decide done-ness ONCE up front. Polish what already exists. Let a real
fake-human tester drive a priority queue. Keep it prompt-driven; the only deterministic code
is the launcher + the plan-walk enumerator + the status store. The agents do the work.

----------------------------------------------------------------
LOAD-BEARING GROUNDED FACTS (do not re-derive; cite these in the build)

F1  CLAUDE.md filter trip is solvable IN-REPO. The project CLAUDE.md is NOT in the system
    prompt; cli.js injects it as the FIRST user message, a <system-reminder># claudeMd ...
    block, via prependUserContext (cli.js:449506-449526), built by getUserContext
    (cli.js:163422-163437) and gated by env CLAUDE_CODE_DISABLE_CLAUDE_MDS (cli.js:163425).
    The proxy already swaps the SYSTEM prompt (devin_wire.py:58-70,174-176) but forwards
    message bodies verbatim (devin_wire.py:182-220) — that is why the ELAI-root CLAUDE.md
    still trips the gateway filter (RUN-GLM-5.2.md:31-40). Two fixes, both in-repo:
      (a) launcher one-liner: set CLAUDE_CODE_DISABLE_CLAUDE_MDS=1 -> no claudeMd block at
          all (loses benign per-worker CLAUDE.md too).
      (b) surgical proxy sanitizer at the user-branch devin_wire.py:216-220, keyed on the
          "# claudeMd" marker (strip / rewrite only the filter-tripping content; keep a
          benign worker CLAUDE.md). Symmetric to the existing DEVIN_TOOL_DESC=generic knob.

F2  CONCURRENCY needs one proxy PER worker. devin_proxy.py is async (Starlette/uvicorn) but
    stamps a single per-PROCESS conversation id (_CONV_ID = uuid4 once, devin_proxy.py:39)
    into every request (devin_wire.py:166-168). Two independent sessions sharing one proxy
    share one gateway conversation id = state collision. Per-request model IS read from the
    body (devin_wire.py:259-268) so different models are fine. CONCLUSION: each of the up to
    4 GLM workers gets its OWN proxy on its OWN port (CLAUDEDEVIN_PORT; .ps1:19), 8810-8813.

F3  RATE-LIMIT RETRY must live IN THE PROXY. The proxy downgrades every gateway error
    (HTTP non-200 at devin_proxy.py:92-95; in-stream trailer error at :96-103) into a 200
    text_delta block (_translate, devin_proxy.py:146-166). So the CLI's Anthropic-SDK retry
    (cli.js:12388-12516, honors 429 + retry-after) NEVER fires. Add bounded exponential
    backoff in _gateway_stream/_iter_devin (devin_proxy.py:79-107) on 429/503/throttle
    only; content-policy errors are deterministic and must NOT be retried.

F4  THE FSM (b-plan-runner SKILL.md:34-177) — 13 steps; parallelism only at step 3:
    1 CONTEXT (context-compiler) -> 2 SCOPE (scope-planner-elai) -> 3 EXPLORE
    (integration-explorer-elai xN PARALLEL, then synthesizer-elai) -> 4 SPEC (spec-grounder)
    -> 5 PROMPT (prompt-engineer; user approval) -> 6 TESTING_FIRST (claim + auditor health
    + write tests) -> 7 BLAST RADIUS (impact-mapper) -> 8 EXECUTING (write code, no tests)
    -> 9 AUDITING (auditor; on BLOCK -> fixer, re-audit ONCE) -> 10 WIRING GATE (10a freeze
    -check, 10b wiring_contract, 10c smoke-manifest row, 10d no-default-off allowlist) ->
    11 TEST ATTRIBUTION (test-attributer) -> 12 MEMORIZING (memory-writer + FEATURE-MAP
    upsert) -> 13 COMMITTING (one commit). orchestrate variant: per sub-phase, context
    -compiler + integration-explorer PARALLEL cap=3, skip prompt-engineer, executor, auditor
    1 cycle, memory-writer; phase-end sweep; ONE commit. Audit fix cap = 1 cycle.

F5  PLAN SOURCES + ORDER (the walk).
    Phases 0-4 = docs/specs/guide/steps/ per-step files:
      pre-phase-0.txt, phase-0.txt, phase-1A..1I.txt (9), phase-2A..2D.txt (4),
      phase-3A..3L.txt (12), phase-4A..4M.txt (12, no 4L).
      EXCLUDE the summaries phase-index-pre0-2D.txt / phase-index-3A-4J.txt /
      phase-index-MASTER.txt, and the -BASELINE/-SURGICAL/-ORACLE variant files.
    B..L = docs/thefinalglance/{LETTER}-plan/ (counts: B47 C61 D32 E57 F17 G12 H57 I57 J111
      K27 L18). Per dir, SKIP {L}-0_INDEX.md, {L}-SUMMARY_INDEX.md, E-SUMMARY_INDEX.md, and
      any _grounding/ subdir. Real task files = {LETTER}-<N>[.<sub>]_<slug>.md.
    IN-FILE STRUCTURE: plan-id = 1 file = 1 work unit. Tasks = the numbered list under the
      "## Tasks" section. Done-gate per file = the "## WIRING_CONTRACT" block (the exact
      thing wiring_contract_check.py consumes: produces_symbols / consumes_symbols /
      reachable_from / stub_ban / frontend_pair / observable_effect{trigger,expect_ssb,
      expect_artifact}).
    MARKETING/GTM EXCLUSIONS (skip; not implementation): K-plan positioning/economics files
      K-1, K-2, K-4, K-6, K-7, K-18, K-24, K-MASTER-BRIEF-2026-06-12.md (keep engineering K
      files e.g. K-3, K-11, K-23). Plus the workflow/reasoning/ELAI-{GTM,FOCUS,MARKET}-*
      docs are already outside the plan dirs.
    Master catalog (digest only, superseded for steal-context by ELAI-FEATURE-MAP.md):
      docs/thefinalglance/ALL-PLANS-MASTER-SUMMARY.md.

F6  THE TESTER IS ~80% ALREADY BUILT (in conductor/, gitignored). Reusable as-is:
    tester_vision.py (codex vision fn + run_vision_journey + AX/SSB anti-rubber-stamp),
    cdp_action_driver.mjs (bun CDP click-by-bbox + console-error sink), tester_runtime.py
    (DashboardRuntime: launches bun vite + headless Chrome), tester_sandbox.py (dual-port
    pool 1420/9222 + 1421/9223), tester_lane.py, test_modes.py (deterministic metamorphic/
    fuzz/SSB-invariant/replay bug-hunter, FREE, already found a real bug), acceptance
    .make_ssb_probe (read-only ssb.db WAL). Rubric: workflow/reasoning/ELAI-TEST-CHECKPOINT
    -MAP.json (11 milestones, 86 features; per-feature surface/trigger/expected_effect/
    rubric/status; spec-eval posture). DESIGN.md exists (workflow/design/DESIGN.md v1.5).
    GAPS to finish (PHASE 4): (1) ui-wire-smoke.mjs never run end-to-end (author 2-3 real
    manifests); (2) defect dedup by root-cause not command; (3) account-b codex auth
    unconfirmed; (4) daemon teardown durability under sustained run; (5) design-critique
    lane (codex critic vs DESIGN.md) designed-not-built.

F7  WIRING-CHECK AGENT delta. wiring-contract-validator.md is a thin runner over the single
    backend-shaped script wiring_contract_check.py. To add the frontend legs (built+frozen
    per workflow/handoff/frontend-wiring-enforcement.md): add a frontend=true/auto trigger
    (when WIRING_CONTRACT.frontend_pair is non-null or the changeset touches dashboard-v2/
    .svelte/invoke), then 3 thin run-and-report steps —
      tauri_ipc_symmetry_check.py (dead IPC = FAIL), svelte_mount_reachability.py (ORPHAN =
      FAIL), bun visual/ui-wire-smoke.mjs --manifest <auditor-authored> (exit1 absent row =
      FAIL; exit2 env-down = surfaced caveat not a pass) — plus output sections + an
      aggregate "ANY leg FAIL => FAIL" rule. No new tool grant (it already has Bash). The 3
      tools must be re-locked once out-of-band (gate_freeze_check.py --lock) or they read as
      added_unbaselined BLOCK (correct fail-closed) until then.

----------------------------------------------------------------
DECISIONS MADE (closed forks; override any in chat)

D1  A DEDICATED committer AGENT COMMITS+PUSHES PER TASK, behind a LEAN guardrail (NOT the
    full agent-dispatch-preamble R-1 read-only-git ban, which the user judged too
    restrictive). The committer runs normal git (add / commit / push, NON-force) once per
    completed task, maintainer voice, NO AI trailer (commit-cadence.md). The only protection is
    one PreToolUse Bash guardrail (PHASE 0.6) that blocks ONLY catastrophic ops repo-wide:
    git reset --hard, git clean -fd / -fdx, git push --force / -f / --force-with-lease,
    git rebase, git branch -D / -d on main, git filter-branch / filter-repo, git reflog
    expire, git checkout/restore -- <paths> (mass discard), git update-ref -d, and rm -rf /
    Remove-Item -Recurse -Force on the repo root or .git. Everything else (normal commit/push,
    file edits, scoped checkout of a single file) is allowed. Emergency floor only: no worker
    can wipe the repo or drop commits by mistake. The system "commits and pushes directly to
    this repo" (the F: repo tracks .elai_cc) exactly as the user asked.

D2  ONE PROXY PER WORKER (4 ports, e.g. 8810-8813), because of F2 (the per-process _CONV_ID
    cannot be shared). Each of the up to 4 worker slots launches claudedevin with its own
    CLAUDEDEVIN_PORT.

D3  CLAUDE.md FILTER: primary = launcher sets CLAUDE_CODE_DISABLE_CLAUDE_MDS=1 for worker
    sessions (F1a) + the worker gets its rules from the dispatch prompt (dispatch-preamble +
    role .md), NOT from the root CLAUDE.md. The surgical proxy sanitizer (F1b) is the
    documented fallback if a benign worker CLAUDE.md is later wanted. Rationale: there is no
    model-side controller to "hold" the ruleset (the controller is a script); the GLM worker
    gets every rule it needs from its dispatch prompt + its role .md, exactly the guidance's
    "rules ride in the prompt" model.

D4  AUTONOMY ENVELOPE = orchestrate's. Fully autonomous walk; break only on the 4 tripwires
    (irreversible op, 50-event bubble cap, dep-wait>4h, sweep-fail x3) + a rate-limit
    sustained-stall tripwire. Tester-found issues jump the queue (guidance).

D5  MAX 4 CONCURRENT GLM-5.2 WORKERS (the free pool), ANY ROLE. The loop may keep up to 4
    claudedevin -p workers in flight at once (parallel checks/explorers, or stages across
    independent queue items). The ONLY serial constraint: a single plan never has 2 executors
    (S5) on it at the same time; executor #2 resumes from executor #1's handoff after #1
    stops. The PHASE 0.4 in-proxy backoff is the cushion when the shared free-model gateway
    rate limit is hit (conductor may also be running).

----------------------------------------------------------------
PER-TASK AGENT ROSTER + SEQUENCE (canonical; each task runs the FULL sequence, then next task)

EVIDENCE CHAIN (the spine the user asked for): no stage invents data. Each stage emits a
STRUCTURED FINDING appended to a per-task evidence ledger (workflow/glm-orchestrator/evidence/
{plan_id}.json). Every downstream stage CONSUMES the prior findings from the ledger; the chain
flows context -> integration -> wiring -> prompt -> execution -> audit -> wiring-gate -> review
-> polish -> commit, and culminates at the E2E TESTER, which reads the accumulated ledger
across plans and drives the REAL frontend against exactly what the chain says was built and
wired (not a static guess). A stage that needs a fact reads it from the ledger; if it is
missing, that is a BLOCKED handoff, never a fabrication.

  S1 CONTEXT       context-compiler.md           finding: the EXACT existing files to EDIT
                     (anti-duplication packet) + the task context for plan P task T.
  S2 INTEGRATION   integration-explorer-elai.md  finding: the PRODUCTION_HOOK seam (file+fn)
                     where the new code must be called from a real entrypoint. [S1+S2 may pair]
  S3 WIRING PRE-CHECK (read-only, the two run PAIRED within the 4-worker pool):
       wiring-backend.md   (NEW; thin runner: production_caller_check.py, python_wire_check.py,
                            wiring_contract_check.py, gate_freeze_check.py,
                            traced_test_assert_check.py, maintainability_check.py)
       wiring-frontend.md  (NEW; thin runner: tauri_ipc_symmetry_check.py,
                            svelte_mount_reachability.py, visual/ui-wire-smoke.mjs)
                     finding: wired = yes | partial | no, and WHICH legs already pass.
                     ROUTE -> wired=yes: skip to S9 POLISH (no rebuild). partial/no: S4.
  S4 PROMPT-ENG    prompt-engineer.md            consumes S1-S3; finding: the shaped executor
                     prompt (budget, focus, the exact files, the hook to wire).
  S5 EXECUTION     executor.md (ONE at a time)   implements + wires the S2 hook + writes the
                     unit test. MAY write a handoff (workflow/glm-orchestrator/handoff/
                     {plan_id}.md) for executor #2 if it runs out of budget mid-task; #2
                     resumes from that handoff. EDIT existing files, never recreate a module.
  S6 AUDIT         auditor.md                    sole test runner; build + tests. On BLOCK ->
                     fixer.md, re-audit ONCE, then surface.
  S7 WIRING DONE-GATE  wiring-backend.md + wiring-frontend.md (PAIRED) over the changeset:
                     10a freeze-check, 10b wiring_contract (+ SSB row), 10c smoke-manifest row,
                     10d no-default-off allowlist. ANY leg FAIL => FAIL. Off-agent (never
                     executor-self-run). This is the real definition of done.
  S8 REVIEW        reviewer.md (NEW; maintainability-review.md lens) finding: quality findings
                     (the 8-dimension catalog + MR-1..MR-4) with remedies.
  S9 POLISH        polisher.md (NEW; simplify + maintainability) hardens / simplifies. This is
                     the MAIN work on the wired=yes route; on the execute route it is the
                     cleanup pass that acts on S8's findings.
  S10 COMMIT+PUSH  committer.md (NEW; lean guardrail D1) ONE commit + push per task, maintainer
                     voice, no AI trailer. Records the sha to the ledger.
  S11 MEMORY       memory-writer.md (once per PLAN completion, not per task) memory-bank +
                     ELAI-FEATURE-MAP upsert (preserves the house never-skip-memorize rule).

  (top) E2E TESTER  the PHASE 4 fake-human tester. Reads the evidence ledger; drives the real
                     dashboard-v2 like a human across the rubric; defects it finds become
                     queue items (priority over plans) that re-enter this same S1..S11 sequence.

Available-but-not-default (the user streamlined these OUT of the per-task sequence; the
controller MAY invoke them on a hard task): scope-planner-elai, synthesizer-elai, spec-grounder,
impact-mapper, designer, dashboard-smoker, the persona-* anti-spiral set, adversary.

----------------------------------------------------------------
ARCHITECTURE (the controller is a DETERMINISTIC SCRIPT; Opus = 0; GLM does every LLM call)

  orchestrator.py  -- THE CONTROLLER. A plain Python loop. NO reasoning model in it.
    while not queue.empty():
      item   = queue.pop_next()                  # tester issues > plan order
      plan   = parse_plan(item.file)             # ## Tasks + ## WIRING_CONTRACT
      # S3 PRE-CHECK: dispatch the wiring GLM workers; they RUN the checker scripts.
      run wiring-backend + wiring-frontend (GLM workers)
      wired  = classify_by_EXIT_CODES(results)   # all gates exit 0 -> yes ; else partial/no
      if wired == yes:
        dispatch S9 polish (GLM) ; goto DONE-GATE
      else:
        for stage in [S1,S2,S4,S5,S6,S8,S9]:     # S5 serial; up to 4 GLM concurrent overall
          dispatch_glm_worker(stage, item, ledger)   # one claudedevin -p call per stage
      # S7 DONE-GATE: deterministic, `if exit==0` logic (NOT model judgment):
      run wiring-backend + wiring-frontend (done-gate mode)
      if any gate exit != 0:
        dispatch fixer (GLM) once ; re-run gate ; if still fail -> mark BLOCKED, next
      dispatch S10 committer (GLM writes the message) ; THE LOOP runs git commit + push
      dispatch S11 memory (GLM, once per plan)
      plan_status.write(item, wired, gate) ; queue.mark_done(item) ; next

  The loop makes ONLY deterministic decisions (advance / route / pass) off SCRIPT EXIT CODES.
  Every decision needing a model (write code, audit it, review it, polish, commit msg,
  memory) is a GLM-5.2 worker. No Opus, no live-session skill, no cron, no headless Opus.

  LAUNCHER (called BY the loop; deterministic) glm-worker.ps1 -- wraps
    claudedevin -Model glm-5-2 -p "<templated prompt>", per-worker port (8810-8813),
    CLAUDE_CODE_DISABLE_CLAUDE_MDS=1, proxy does rate-limit backoff, --dangerously-skip-perms.
  plan_walk.py  -- plan-id -> file path in human order (F5), excludes summaries + GTM.
  plan_status/  -- per-plan json: {plan_id, wired, route, gate, last_run, notes}.
  queue.py      -- priority queue (tester issues > plan order); `--next` pops one. Resume =
                   restart the loop; it reads plan_status + queue, no timer involved.
  evidence/{plan_id}.json -- the ledger the loop threads stage -> stage.
  GLM WORKERS (claudedevin -> proxy -> gateway -> GLM-5.2), up to 4 concurrent: each reads
    its role .claude/agents/<name>.md + the templated prompt and does its one job.
  GATES = the wiring done-gate scripts (F4 step 10 + F7 frontend legs) + the fake-human
    tester (PHASE 4). The LOOP reads their EXIT CODES; they are off-agent + frozen.
  COMMIT+PUSH = the loop runs git after the committer worker writes the message (D1 guardrail).

----------------------------------------------------------------
PHASE 0 — GLM WORKER HARNESS HARDENING (the deterministic core)
Goal: a GLM-5.2 claude-code worker that runs DIRECTLY from F:\.ELAI_workflow, two lanes
concurrently, survives rate limits, with a new orchestration-tuned system prompt — proven
on a real tool-using headless run from the repo root.

Deliverables:
  0.1 New worker system prompt. Author a fresh prompt that REPLACES the content the proxy
      injects (claudedevin-system.md; devin-system.md is the older sibling — replace both
      or point the proxy at the new file). It must: declare the worker is a scoped executor/
      checker that obeys the dispatch-preamble it receives, be free of security/RE wording
      that trips the gateway filter, and carry the "EDIT existing files, NEVER recreate a
      module that already exists" line (the single highest-leverage anti-slop instruction
      per the guidance). Confirm the proxy's _override_system path (devin_wire.py:58-70)
      loads the new file.
  0.2 CLAUDE.md filter fix (D3/F1). Edit claudedevin.ps1 to set
      $env:CLAUDE_CODE_DISABLE_CLAUDE_MDS=1 on the worker launch path. Prove a GLM-5.2
      headless run launched FROM F:\.ELAI_workflow no longer returns a content-policy error.
      Implement the proxy sanitizer (F1b, devin_wire.py:216-220) as a documented, default-
      off fallback knob (DEVIN_CLAUDEMD=pass|strip).
  0.3 4-worker concurrency (D2/F2). Add a glm-worker.ps1 (or extend claudedevin.ps1) so up to
      4 workers launch on distinct ports (8810-8813), each auto-starting its own proxy if down
      (the .ps1 already health-checks /health and starts the proxy; parameterize the port).
      Prove 4 concurrent headless GLM tasks on the repo run to completion without colliding
      (no shared _CONV_ID cross-talk).
  0.4 Rate-limit auto-retry (D4/F3). Add bounded exponential backoff in devin_proxy.py
      _gateway_stream/_iter_devin (:79-107) on 429/503/throttle ONLY (parse retry-after if
      present); never retry content-policy. Log each retry to proxy.log.err under DEVIN_LOG.
      Prove the retry fires by forcing/simulating a 429.
  0.5 Capability gaps. Exercise the not-yet-verified surfaces on GLM-5.2 that the
      orchestration needs: Task subagents, TodoWrite, MCP tools (codegraph-elai + local-rag),
      WebSearch/WebFetch. Record PASS/FAIL per surface; if a surface fails on GLM, note the
      workaround (e.g. controller does that step instead of the worker).
  0.6 Emergency git/fs guardrail (D1, lean). A single PreToolUse Bash hook (registered for
      the worker sessions) that blocks ONLY the catastrophic ops in D1's list (reset --hard,
      clean -fd/-fdx, push --force/-f/--force-with-lease, rebase, branch -D/-d main,
      filter-branch/filter-repo, reflog expire, mass checkout/restore -- <paths>,
      update-ref -d, rm -rf / Remove-Item -Recurse -Force on repo root or .git). Allows
      normal commit/push + file edits + single-file checkout. Prove it blocks `git reset
      --hard HEAD~5` and `git push --force` but allows `git commit` + `git push`.
Files: devin-bridge/claudedevin-system.md (replace), devin-bridge/devin-system.md (replace/
  retire), devin-bridge/claudedevin.ps1 (edit: port param + CLAUDE_CODE_DISABLE_CLAUDE_MDS),
  devin-bridge/devin_proxy.py (edit: retry loop), devin-bridge/devin_wire.py (edit: optional
  claudeMd sanitizer), NEW devin-bridge/glm-worker.ps1 (if a separate launcher is cleaner).
Acceptance: a GLM-5.2 worker, launched from F:\.ELAI_workflow, completes a real Read+Edit+
  Bash+Grep+Glob task; 4 lanes run concurrently clean; a simulated 429 is auto-retried; the
  capability matrix (0.5) is recorded. NO content-policy error.
Opus-1M fit: one agent reads the whole devin-bridge + the cli.js anchors above and the GLM
  capability surface, then makes 4-5 coordinated edits + a live proof. Self-contained.

----------------------------------------------------------------
PHASE 1 — PLAN-WALK ENUMERATOR + STATUS STORE + PRIORITY QUEUE
Goal: the deterministic spine that turns "all the plans" into an ordered, resumable,
priority-aware work list, with a durable per-plan done-state decided once.

Deliverables:
  1.1 plan_walk.py — emits the ordered list of (plan_id, file_path, kind) in human order
      (F5): pre-phase-0, phase-0, phase-1A..1I, 2A..2D, 3A..3L, 4A..4M, then B..L by number
      within each letter. Excludes the summary files, the -BASELINE/-SURGICAL/-ORACLE
      variants, the {L}-0_INDEX/{L}-SUMMARY_INDEX/_grounding scaffolding, and the GTM K-files
      (F5 list). --json. Deterministic; a wrong order silently runs wrong work, so include a
      self-test that asserts the count + first/last ids.
  1.2 In-file task parser — given a plan file, extract the numbered items under "## Tasks"
      and the "## WIRING_CONTRACT" block, so a prompt can address "task T of plan P" and the
      done-gate knows the contract. Handle the phase-N step files (different shape — they are
      step specs, not WIRING_CONTRACT files; detect and adapt).
  1.3 plan_status store — workflow/<glm-orch>/plan-status/{plan_id}.json schema:
      {plan_id, file, wired: yes|partial|no, route: polish|execute, gate: pass|fail|pending,
      last_run_iso, executor_lane, notes}. One file per plan (resumable; the fix for the old
      "already-done loops forever" bug — decide once, up front).
  1.4 queue.py — merge plan-order with a tester-issue inbox so tester-found defects jump
      ahead of plan work; each queue item carries the same shape so the FSM consumes plans
      and tester-issues identically (guidance open-decision: express a tester issue as a
      "task" — define that shape here).
Files: NEW under devin-bridge/ or a new workflow/glm-orchestrator/ tool dir: plan_walk.py,
  plan_status schema + helpers, queue.py. All stdlib-only Python (house rule), --json, exit
  codes.
Acceptance: plan_walk.py prints the full ordered walk with correct counts and exclusions;
  the task parser extracts tasks + contract from 3 sample plans (one B, one phase-N step, one
  with frontend_pair); the status store round-trips; the queue interleaves a fake tester
  issue ahead of the next plan.
Opus-1M fit: one agent reads the plan-dir structure + several plan files to get the parser
  shapes right, then writes the 3 deterministic tools + self-tests.

----------------------------------------------------------------
PHASE 2 — THE DETERMINISTIC ORCHESTRATOR LOOP + GLM WORKER DISPATCH (controller = a script)
Goal: build orchestrator.py, the deterministic control loop that IS the controller. No model
in the loop; Opus = 0. It pops the queue, parses the plan, dispatches a GLM-5.2 worker per
stage via the PHASE 0 launcher, threads the evidence ledger, routes PURELY on the wiring/
done-gate SCRIPT EXIT CODES, and runs git commit+push after the committer worker writes the
message. The per-agent prompt templates (already authored this session under
.claude/skills/glm-orchestrate/prompts/) are the inputs the loop substitutes into per worker.

Deliverables:
  2.1 orchestrator.py — the loop (pseudocode in ARCHITECTURE above). It pops queue.py
      --next, parses the plan, dispatches GLM workers per stage (S5 serial; up to 4 GLM
      concurrent overall, D5), threads evidence/{plan_id}.json, classifies wired= by SCRIPT
      EXIT CODES (not model judgment), routes yes->polish / partial-no->full sequence, runs
      the S7 done-gate with `if exit==0` logic (fail -> fixer worker once -> re-gate ->
      else BLOCKED), runs git commit+push after the S10 committer writes the message, runs
      S11 memory once per plan, writes plan_status (1.3), marks the queue, next. The loop
      makes ONLY deterministic decisions; every model decision is a GLM worker. NOTE: the
      registered glm-orchestrate SKILL.md (Opus-controller shape) is WRONG and superseded by
      orchestrator.py — flag it for deletion.
  2.2 Per-agent prompt templates — one template per role used: context-compiler, scope-
      planner-elai, integration-explorer-elai (xN), synthesizer-elai, spec-grounder, prompt-
      engineer, impact-mapper, executor, auditor, fixer, wiring-contract-validator, memory-
      writer, dashboard-smoker, plus the pre-check (wiring-contract-validator + auditor +
      integration-explorer) and the polish-lane prompt. EVERY template begins with the
      agent-dispatch-preamble pointer line (worker git read-only, scope contract, BLOCKED =
      handoff entry) and substitutes {plan_id, file_path, task_id, phase_id, wired_value,
      role_md_path, handoff_file}. The executor template MUST carry the context-compiler's
      "EDIT these exact files, do NOT recreate" packet (the anti-duplication core).
  2.3 DESIGN.md injection rule — any plan/task touching dashboard-v2/frontend/visualization
      gets workflow/design/DESIGN.md + .claude/rules/visual-ui-loop.md + design-taste.md
      baked into the executor + auditor prompts.
  2.4 The pre-check (PHASE A) wiring — uses codegraph-elai + elai.map + production_caller_
      check.py / python_wire_check.py + a cheap real-entrypoint smoke to classify each plan;
      writes the durable wired value once.
Files: NEW .claude/skills/glm-orchestrate/SKILL.md, NEW prompt-template files (one per role,
  or a single templates dir). REUSE every .claude/agents/*.md verbatim (do not paraphrase
  agent instructions into the launcher).
Acceptance: the skill, given one partial plan, produces the correct dispatch sequence (pre-
  check -> route -> FSM -> done-gate) on paper and runs it against a real GLM worker for at
  least the context-compiler + one explorer + executor steps, producing a real edit. The
  preamble appears at the top of every dispatched prompt.
Opus-1M fit: one agent reads orchestrate + b-plan-runner SKILLs + every .claude/agents/*.md
  it will template, plus the dispatch-preamble and the done-gate rules, then authors the
  skill + the full template set. This is the single biggest writing phase — well-matched to
  1M context.

----------------------------------------------------------------
PHASE 3 — WIRING DONE-GATE INTEGRATION + wiring-contract-validator UPDATE  (parallel w/ P4)
Goal: the focused done-gate (F4 step 10 + the 3 frontend legs F7) is the real definition of
done for every plan, including frontend, and is correctly wired into the controller's PHASE D.

Deliverables:
  3.1 The wiring role is SPLIT into two thin-runner agents (authored this session by the
      prompt-writer; see roster S3/S7): wiring-backend.md (production_caller_check.py,
      python_wire_check.py, wiring_contract_check.py, gate_freeze_check.py,
      traced_test_assert_check.py, maintainability_check.py) and wiring-frontend.md
      (tauri_ipc_symmetry_check.py [dead IPC=FAIL exit1], svelte_mount_reachability.py
      [ORPHAN=FAIL exit1], bun visual/ui-wire-smoke.mjs --manifest <auditor-authored>
      [exit1 absent row=FAIL; exit2 env-down=surfaced caveat]). PHASE 3 VERIFIES both run
      correctly, with the output sections (TAURI_IPC_SYMMETRY / SVELTE_MOUNT_REACHABILITY /
      UI_WIRE_SMOKE for frontend; the backend dimensions for backend) and the aggregate
      "ANY leg FAIL => FAIL" rule across S7. Thin-runner doctrine holds: never re-implement
      check logic in prose, never flip a script FAIL to PASS. The legacy single
      .claude/agents/wiring-contract-validator.md stays for b-plan-runner; this system uses
      the 2 split agents.
  3.2 Re-lock the gate out-of-band: python .claude/tools/gate_freeze_check.py --lock (the
      handoff's pending R8 step) so the 3 frozen frontend tools stop reading as added_
      unbaselined BLOCK. Document this as a human/out-of-band action (who-freezes-the-freezer).
  3.3 Wire the done-gate into glm-orchestrate PHASE D: 10a freeze-check (P0 BLOCK on exit3),
      10b wiring_contract (+ the 3 frontend legs from 3.1), 10c smoke-manifest row, 10d no-
      default-off allowlist row. Off-agent (auditor/validator, never executor-self-run).
  3.4 Author 2-3 real ui-wire-manifest.json checks against the live dashboard DOM (verify
      selectors; confirm expect_ssb action_type matches the real producer string) so the
      runtime leg (the only sound frontend proof) actually runs end-to-end at least once.
Files: EDIT .claude/agents/wiring-contract-validator.md; NEW ui-wire-manifest checks under
  .claude/tools/visual/; the --lock is a command, not a file edit.
Acceptance: wiring-contract-validator runs all 4 legs on a frontend plan and aggregates
  correctly; a known dead IPC (the 5 the handoff found) BLOCKs; gate_freeze_check exits 0
  after --lock; at least one ui-wire-smoke manifest passes end-to-end against the live
  dashboard (proving click -> Rust-owned SSB row).
Opus-1M fit: one agent reads the validator agent + the 3 frontend tools + the done-gate
  rules + the live dashboard DOM (for the manifests), then edits one agent file and authors
  manifests. Tight, focused.

----------------------------------------------------------------
PHASE 4 — E2E FAKE-HUMAN DASHBOARD TESTER  (parallel w/ P3)
Goal: a real tester that boots elai (.elai_cc/elai.bat) and DRIVES dashboard-v2 like a human
across the 86-feature rubric, observing AX-tree + SSB journal (not pixels-only), and emits
defects as queue items that follow the same FSM. Mostly assemble F6's existing conductor
assets into the new system + close the 5 gaps.

Deliverables:
  4.1 Port/adapt the conductor tester stack into the glm-orchestrator (or reference it in
      place if cleanest): tester_vision.py, cdp_action_driver.mjs, tester_runtime.py,
      tester_sandbox.py, tester_lane.py, test_modes.py, acceptance.make_ssb_probe. Confirm
      each still runs against the current dashboard-v2.
  4.2 The tester AGENT prompt/skill — runs the rubric (ELAI-TEST-CHECKPOINT-MAP.json),
      partitions dashboard_journeys() vs cli_features(), drives each like a user, scores
      pass/fail/unknown with mandatory evidence regions (visual-ui-loop schema; unknown=fail),
      and writes defects. spec-eval posture (every feature treated as real even if planned/
      wire-dark).
  4.3 Close the 5 gaps (F6): (1) run ui-wire-smoke end-to-end with the PHASE 3.4 manifests;
      (2) defect dedup by ROOT CAUSE (command/module) not per-symptom; (3) confirm or drop
      account-b codex vision auth (fall back to single-lane vision if b unavailable); (4)
      durable daemon/sandbox teardown under sustained running; (5) the design-critique lane
      (codex critic vs DESIGN.md -> FIX-DESIGN tasks) — build it now that DESIGN.md exists.
  4.4 Defect -> queue-item adapter: a tester finding becomes a queue item (PHASE 1.4 shape)
      with priority over plans, carrying the failing feature_id, the rubric line, the
      observed-vs-expected, and the suspected file/seam, so the same FSM (pre-check -> route
      -> execute/polish -> done-gate) fixes it.
Files: a glm-orchestrator tester dir (port or reference conductor assets), the tester agent/
  skill prompt, refreshed ELAI-TEST-CHECKPOINT-MAP.json if features drifted, the defect ->
  queue adapter.
Acceptance: the tester boots elai, drives at least 5 dashboard journeys + 5 CLI features from
  the rubric, scores them with evidence, emits at least one real defect as a queue item, and
  ui-wire-smoke runs green on one manifest. Vision rubber-stamp guards (unknown=fail, evidence
  region mandatory) enforced.
Opus-1M fit: one agent reads the whole conductor tester stack + e2e-tester.md + the rubric +
  visual-ui-loop, then assembles + finishes the gaps + authors the agent prompt. Large but
  cohesive.

----------------------------------------------------------------
PHASE 5 — CONTROLLER COMMIT STEP, DRY-RUN PROOF SLICE, AUTONOMY ENABLEMENT
Goal: tie it together, prove it on a real slice, and turn on the autonomous walk with the
commit/push + tripwires + monitoring.

Deliverables:
  5.1 Commit+push step (D1) — the controller's step 13: after a plan's done-gate passes,
      ONE commit (maintainer voice, lowercase i, no em-dash, no ->, no AI trailer, no test counts
      in body) + push to the F: repo (which tracks .elai_cc). One changeset per plan
      (commit-cadence.md). Workers never push (R-1). On commit failure: STOP and surface,
      never chunk.
  5.2 Dry-run proof slice — run the full extended FSM end-to-end on a representative trio
      WITHOUT pushing: one plan the pre-check rates wired=yes (prove the polish lane), one
      partial (prove execute + done-gate), and one tester-found defect (prove the queue
      jump). Verify no module duplication (the conductor failure mode), the wiring done-gate
      actually gates, and the commit message renders in maintainer voice (review the diff; do not
      push during the dry run).
  5.3 Autonomy envelope + tripwires (D4) — wire orchestrate's 4 tripwires + a rate-limit
      sustained-stall tripwire + the 2-executor cap (D5). Run-state file (objective / work
      list position / decisions / budget / verification) for resumability across sessions.
  5.4 Monitoring + handoff — a live status view (which plan, which lane, gate result) and a
      topic-keyed handoff (workflow/handoff/glm-orchestrator.md) per handoff-protocol.md so a
      later session resumes the walk.
Files: the commit step in glm-orchestrate SKILL.md, run-state schema, the handoff file, any
  monitor helper.
Acceptance: the proof slice (5.2) completes all three lanes with correct routing, a passing
  done-gate, a maintainer-voice commit authored (not pushed) for the partial plan, and a clean
  resume from the run-state file. Then the full walk is armed (phase 0 -> L) but the FIRST
  full push remains a human go (surface it).
Opus-1M fit: one agent integrates the pieces + runs the live proof slice + writes the
  autonomy/monitoring/handoff layer.

----------------------------------------------------------------
SEQUENCING + PARALLELISM
PHASE 0 -> PHASE 1 -> PHASE 2, then PHASE 3 + PHASE 4 in parallel, then PHASE 5.
PHASE 3 and 4 both depend on PHASE 2's done-gate/dispatch contracts but not on each other.

RISKS / OPEN ITEMS (surfaced, not silently assumed)
R-a  GLM-5.2 capability on Task subagents / MCP is unverified (PHASE 0.5). If GLM cannot
     drive subagents reliably, the controller (Opus) must run those steps instead of the GLM
     worker — the design degrades gracefully (worker = executor only; checks stay on Opus/
     deterministic tools).
R-b  The proxy's single in-flight blocking httpx read can stall the other lane (devin_proxy
     .py:85-90). Two separate proxy processes (D2) sidestep this; if a lane still stalls,
     that is the place to look.
R-c  ui-wire-smoke has never run end-to-end; PHASE 3.4 is the first real run and may surface
     selector/action_type mismatches — budget for one iteration.
R-d  conductor is still running on its own repo; the 2-executor cap (D5) + separate ports
     keep the two systems from fighting for the gateway, but watch the shared free-model
     rate limit — the PHASE 0.4 backoff is the cushion.
R-e  21 real wire-dark findings already surfaced on the dashboard (5 dead IPCs + ~16 orphan
     panels) are UNTRIAGED (frontend-wiring handoff). They will trip PHASE 3's frontend legs
     immediately; decide per-finding (wire it / delete it / WIRE-DARK marker) — likely the
     first real queue items.

WHAT TO REUSE vs BUILD (from the guidance, confirmed)
REUSE: all .claude/agents/*.md; b-plan-runner + orchestrate SKILLs as the FSM reference; the
  wiring done-gate rules + tools; agent-dispatch-preamble; the conductor tester stack (F6);
  ELAI-TEST-CHECKPOINT-MAP.json; DESIGN.md; the devin-bridge proxy/launcher.
BUILD FRESH: the new worker system prompt; the plan-walk enumerator + status store + queue;
  the glm-orchestrate skill + prompt templates; the 2-lane launcher; the proxy retry +
  CLAUDE.md fix; the defect->queue adapter; the design-critique lane.
DO NOT REUSE: the conductor's DAG/scheduler/leases/merge_lane/heavy multi-gate stack (plan
  order + the focused wiring gate replace them).

# Autonomous Finish System (the Conductor) — design

Date: 2026-06-18
Author: main session, grounded in 5 deep-research reports (Downloads/deep-research-report 25-29.md)
Problem: finish ELAI automatically — drive ~412 plan files (B..K) plus base phases 0-4 through
implement -> wire -> integrate -> test -> audit -> polish, in repeated passes, using 2 (scalable)
GPT-5.5 API workers, with NO Claude/Opus needed as the orchestrator.

================================================================================
0. THE ONE REFRAME (this is the answer to "HOW")
================================================================================

The orchestrator must NOT be an LLM. Today you drive the FSM by hand in an Opus session running
the orchestrate skill. That is why it "needs 100 opus" and why work comes out inconsistent: an LLM
orchestrator is expensive, non-deterministic, forgets state, and (worst) lets a worker self-certify
"done" so wire-dark code slips through.

All 5 reports converge on the same architecture, independently:
  - 25: "deterministic outside, agentic inside" — durable stateful workflow, bounded agentic loops.
  - 26: winning stacks are framework + observability/evals + runtime, never a bare LLM loop.
  - 27: a strict state machine is necessary but not sufficient — use a LAYERED truth stack; the only
        write gate to "accepted truth" is independent VERIFICATION.
  - 28: centralize truth + contracts + verification, DECENTRALIZE execution; stateless workers in
        isolated worktrees; an independent verifier/integrator owns "done", never the worker.
  - 29: hierarchical planner -> worker -> validator on a deterministic, event-sourced substrate; no
        single model ever has a globally reliable model of the repo; decompose by dependency frontier.

So: replace the Opus orchestrator with a small deterministic Python daemon (the CONDUCTOR). It costs
~0 tokens. The 2 GPT-5.5 APIs are stateless WORKERS it dispatches. Opus/Claude becomes optional —
used only for rare hard escalations, not for the loop.

Net effect on your 3 stated failures:
  - "need 100 opus to orchestrate"  -> orchestration is code, zero LLM cost. Scale = config.
  - "agents leave things wire-dark" -> a worker CANNOT mark its own work done. The frozen, off-agent
                                       wire/build/test gates (already built in ELAI) are the only door
                                       to DONE. Missing evidence = FAIL (fail-closed).
  - "some finish, some don't"       -> durable task graph + leases + bounded retries + artifact-
                                       grounded state. Every task has one terminal state, survives a
                                       crash/restart, and is re-queued on failure until the gate passes.

================================================================================
1. WHAT YOU ALREADY HAVE (this design is 70% assembly, not invention)
================================================================================

ELAI already contains most of the research-recommended truth + verification stack. The Conductor
mostly WIRES these together and drives them from code instead of from a prompt:

  Research layer (report 27/28/29)     ELAI artifact that already implements it
  ----------------------------------   ------------------------------------------------------------
  Task graph w/ verifier contracts     The 412 plan files: each B-N/C-N has WIRING_CONTRACT +
                                        DETERMINISTIC_CHECK + ACCEPTANCE blocks = a TaskSpec already.
  Append-only event log (history)      SSB (action_type rows, SCHEMA_VERSION, derived counters).
  Repo knowledge graph (retrieval)     elai.map / elai_graph.json / codegraph-elai MCP.
  Off-agent verifier gates (the door)  production_caller_check.py, python_wire_check.py,
                                        maintainability_check.py, traced_test_assert_check.py,
                                        gate_freeze_check.py, wiring_contract_check.py, cargo check/test.
  Frozen gate (worker can't weaken it) .claude/gate-manifest.txt + gate_freeze_check.py.
  Role separation (impl != certify)    .claude/agents/*.md (executor vs auditor vs context-compiler).
  Bounded context packets              context-compiler agent logic; spec_link_check.py; phase indexes.
  Failure taxonomy / scoped tests      failure_taxonomy.py; cross_lang_diff.py; -p scoped crate tests.

What is MISSING is only the deterministic driver and the durable task store. That is what we build.

================================================================================
1B. BOOTSTRAP PHASE — THE TWO-ACCOUNT SYSTEM THAT BUILDS THE CONDUCTOR
================================================================================

The full Conductor is not built by hand in one long interactive Codex session. A smaller deterministic
BOOTSTRAP CONDUCTOR lives at:

  F:\.ELAI_workflow\bootstrap-conductor\

Its job is to keep the two proxy-backed Codex accounts usable long enough to build, audit, fix, hand off
to, and start the full Conductor. It is controlled from a normal Windows browser at:

  http://127.0.0.1:8765

The browser is an operations dashboard, NOT a chat UI. It exposes:
  - account state, active job, latency, cooldown, and sanitized failure category,
  - durable structured-job submission,
  - queue, dependency, retry, and recent-event views,
  - start, pause, drain, resume, cancel, and emergency-stop controls,
  - one idempotent "Seed full-system build" action.

Bootstrap invariants:
  - one Codex process maximum per account; two provider requests maximum globally,
  - credentials remain in permission-locked CODEX_HOME directories outside the repo,
  - F: is mounted explicitly inside codex-vm; global WSL automount stays disabled,
  - a workspace lock prevents concurrent mutation of the same resource,
  - retries are bounded and respect measured cooldowns,
  - a mutating run that disconnects after turn start becomes NEEDS_REVIEW, never an automatic replay,
  - state and leases survive controller restarts in SQLite WAL,
  - browser/API/log output is sanitized and never carries credentials,
  - no bootstrap or full-system job commits or pushes.

The idempotent bootstrap recipe alternates the two accounts through these dependency-ordered stages:
  1. specify the full Conductor from this complete design,
  2. build durable task/event/lease storage and backlog import,
  3. build Codex worker, context, worktree, and frozen-gate adapters,
  4. build dependency scheduling and serialized integration,
  5. build waves 1-5 and the full browser control surface,
  6. independently audit concurrency, wiring, recovery, lifecycle, and secret handling,
  7. fix all BLOCK findings and prove a dry-run handoff,
  8. import the real ELAI backlog and start the first bounded implementation wave.

The bootstrap system is intentionally simpler than the full Conductor: it serializes mutating work on a
shared workspace and uses the second account for alternate stages, read-only review, or other verified
non-colliding work. The full Conductor replaces that limitation with per-task worktrees, disjoint write
sets, frozen verifier ownership, and serialized integration.

================================================================================
2. ARCHITECTURE (5 planes; the first 4 are state, the last is cognition)
================================================================================

  +-------------------------------------------------------------------------+
  | CONDUCTOR (deterministic Python daemon, ~1-2k LOC, in a Docker svc)      |
  |   - scheduler: pick next ready task per free worker slot                 |
  |   - lease manager: one task per slot, TTL, crash-safe reclaim            |
  |   - gate runner: runs the frozen verifier stack OFF-agent                |
  |   - integrator: serialized merge of worker branch -> main + post-gate    |
  |   - budgets/tripwires: retry cap, token cap, wall-clock, stall detect    |
  |   - 0 LLM calls                                                          |
  +-------------------------------------------------------------------------+
        |                 |                 |                 |
        v                 v                 v                 v
  [CONTROL DB]      [EVENT LOG]       [KNOWLEDGE GRAPH]   [ARTIFACT STORE]
  SQLite (WAL),     append-only       elai.map +          per-task worktree
  AUTHORITATIVE     (its own table    codegraph-elai      diffs, test logs,
  current state:    or ELAI SSB):     (retrieval only,    gate outputs,
  task graph,       every transition  NON-authoritative)  audit reports,
  state, leases,    for replay/audit                      receipts
  verdicts
        |
        v  (dispatch one TaskSpec to one free worker slot)
  +-------------------------------------------------------------------------+
  | WORKERS (the 2 GPT-5.5 APIs; scalable to N)                              |
  |   each = a coding-agent harness process (Codex CLI / aider / OpenHands)  |
  |   pointed at ONE proxy base_url + the GPT-5.5 model, running INSIDE      |
  |   a git worktree (branch isolation). Stateless: TaskSpec in, patch+      |
  |   evidence out, process exits. Roles are TASK TYPES, not dedicated       |
  |   agents (you only have 2 slots; you time-multiplex roles).              |
  +-------------------------------------------------------------------------+

Truth ownership (report 27, the layered truth stack — do not let one store carry all truth):
  - Control DB = truth about CURRENT commitments (who holds what, what passed).
  - Event log  = truth about HISTORY (replay, forensics, "how did we get here").
  - Gate stack = truth about CORRECTNESS (reproducible, off-agent, fail-closed).
  - Knowledge graph = truth about RELEVANCE only (derived, rebuildable, never arbitrates "done").

================================================================================
3. THE TASKSPEC / TASKRESULT CONTRACT (report 28) — the worker interface
================================================================================

The Conductor hands each worker a small, bounded TaskSpec. Re-grounded from durable state every time
(report 29: never rely on one initial prompt; goal drift grows with context length):

  TaskSpec:
    task_id, plan_id (e.g. "I-34")
    role               # PLAN | EXECUTE | WIRE | INTEGRATE | AUDIT | POLISH
    objective          # one paragraph, lifted from the plan file's intent
    base_commit
    allowed_write_set  # files/dirs this task may touch (from the plan's scope)
    context_refs       # bounded packet: the plan file + the specific spec slice + graph neighborhood
    production_hook    # the EXACT seam (file+fn) the new symbol must be called from (report: D-030)
    acceptance         # the plan's ACCEPTANCE + DETERMINISTIC_CHECK blocks, verbatim
    forbidden          # no editing the gate/tests that judge it; no destructive git (preamble R-1..R-8)
    verifier           # the exact gate command the Conductor will run to judge this task
    lease_ttl, idem_key, token_budget

  TaskResult:
    task_id, base_commit, touched_files, branch, artifact_refs, status(DONE/BLOCKED/FAILED), notes

Key discipline (report 29): a task is scoped to a SMALL dependency neighborhood, never "finish plan
I-34" if that spans 20 files. The Conductor splits a fat plan into a chain of small TaskSpecs. "I fixed
it" is never a state transition; "verifier V accepted artifact A for task T" is the only one.

================================================================================
4. THE PER-TASK LOOP (replaces the human-driven FSM)
================================================================================

For each task the Conductor walks a fixed pipeline (deterministic; no LLM decides the flow):

  1. SELECT   pick highest-priority READY task (all deps integrated to main) for a free slot.
  2. CONTEXT  build the bounded context packet (cheap retrieval script over elai.map + codegraph +
              the plan file + its "Spec references" slice). This is a deterministic step, optionally a
              cheap worker sub-call; it is NOT the expensive executor.
  3. EXECUTE  create a fresh worktree off base_commit; dispatch TaskSpec to a free GPT-5.5 worker.
              Worker edits only within allowed_write_set, wires the production_hook, returns a patch.
  4. VERIFY   Conductor (off-agent) runs the frozen gate stack against the worktree:
                cargo check + scoped cargo test -p, wiring checkers (DARK symbol = FAIL),
                maintainability_check, traced_test_assert_check, the plan's DETERMINISTIC_CHECK.
              FAIL -> structured feedback appended -> re-queue (retry cap) or escalate (tripwire).
  5. INTEGRATE on pass, the integrator rebases the branch onto current main and merges. SERIALIZED:
              only one merge at a time even with N workers (kills the 27% semantic-merge-conflict rate
              report 27 flags). Post-merge gate re-run on the new head. Conflict -> INTEGRATE task.
  6. RECORD   append event(s), store artifacts, mark task DONE in control DB, reindex the graph slice.

Epoch compaction (report 28/29): every K completed tasks, the Conductor writes a fresh state summary
and starts a new epoch so no history grows unbounded. (Your existing memory-bank/handoff files are the
natural home for these summaries.)

================================================================================
5. THE COMPLETION WAVES (your "finish, re-test, audit, wire, integrate, polish, another pass")
================================================================================

Model the whole project as ordered WAVES over the 412-plan backlog. Each wave is a pass that emits one
task TYPE with one gate. The Conductor runs waves to convergence, then advances:

  Wave 1  IMPLEMENT    every PLANNED/PARTIAL plan -> EXECUTE task -> build+wire+scoped-test gate.
  Wave 2  WIRE-SWEEP   run wire checkers repo-wide; every DARK symbol -> WIRE task. (Systematic fix for
                       the exact "left wire-dark" failure: it is now a queue, not a hope.)
  Wave 3  INTEGRATION  per-subsystem R10 cross-seam smoke on the REAL entrypoint, HARD input -> INTEGRATE
                       tasks for every seam defect.
  Wave 4  AUDIT        adversarial-auditor pass per plan -> findings -> FIX tasks.
  Wave 5  POLISH       maintainability_check repo-wide + dashboard visual-ui-loop + docs/dead-code.
  Wave N  RE-PASS      repeat AUDIT+POLISH until the full gate stack is green repo-wide with ZERO new
                       findings for K consecutive passes (loop-until-dry, report 29).

COMPLETION IS SYSTEM-LEVEL, not task-level (report 28): done == milestone graph closed AND global gate
stack green on merged head AND K dry audit passes. Never a worker's self-report.

================================================================================
6. CONCURRENCY MODEL (your "1 agent per API, 2 total, scalable")
================================================================================

This is just a work queue with N consumers. Each API key = one consumer slot (it serializes its own
requests, so 1 in-flight task per key). 2 keys = 2 concurrent tasks. Buying a 3rd key = adding one line
to a workers config list. Linear scale, no redesign.

  workers.toml:
    [[worker]]  name="w1"  base_url="https://proxy-a/..."  api_key="..."  model="gpt-5.5"  harness="codex"
    [[worker]]  name="w2"  base_url="https://proxy-b/..."  api_key="..."  model="gpt-5.5"  harness="codex"

Constraints the Conductor enforces so 2 workers never collide (report 28 / git worktree + CAID):
  - disjoint allowed_write_set between concurrently-running tasks (no two tasks touch the same files),
  - one worktree per task, serialized integration to main,
  - a per-task token_budget + global daily budget so a runaway worker can't burn the proxy quota.

================================================================================
7. BUILD CHOICE (report 26 — framework vs thin custom)
================================================================================

Option A (RECOMMENDED to start): thin custom Conductor.
  SQLite (WAL) + ~1-2k LOC Python loop + subprocess-driven Codex/aider workers. You already own every
  verifier script; the Conductor just calls them. Lowest risk, fastest to a working loop, no new
  framework to learn, fits ELAI's gates directly. Reports 25/26 both say: start simple/composable, add
  a durability engine only when long waits/retries actually demand it.

Option B (graduate to, if/when needed): LangGraph (cognitive runtime) + Temporal (durable execution).
  Report 26 ranks LangGraph #1 for stateful long-running agents and Temporal #1 for durable backbone.
  Adopt ONLY once Option A proves the loop and you hit real crash-recovery / multi-week-history pain.
  Don't pay this complexity up front.

Worker harness: LOCKED = Codex CLI (already running in your WSL+Docker VM). Config confirmed:
    model_provider=OpenAI, model=gpt-5.5, model_reasoning_effort=xhigh,
    base_url=https://bmapi.020212.xyz, wire_api="responses", requires_openai_auth=true,
    disable_response_storage=true (ZDR), network_access=enabled.
The two keys = two Codex profiles (different base_url/key), one Codex process per task. Codex natively
speaks the OpenAI Responses API and reads AGENTS.md, so the worker is driven HEADLESS via `codex exec`
(non-interactive), scoped to a per-task git worktree / VM checkout. See section 11 for the exact driver.
ELAI's aider-polyglot (C-51) / openhands (C-57) scouting is now reference-only, not the harness.

================================================================================
8. RISKS THE REPORTS FLAG (and the control for each)
================================================================================

  - Context rot / lost-in-the-middle (25,29): keep TaskSpec + context packet SMALL; decompose by
    dependency frontier; re-ground from durable state each task.
  - Hallucinated progress / reward hacking (28,29): off-agent gate is the only door to DONE; worker
    never edits the gate/tests that judge it (frozen via gate-manifest + gate_freeze_check).
  - Semantic merge conflicts ~27% of agentic PRs (27): serialized integrator + post-merge gate;
    disjoint write sets for concurrent tasks.
  - Looping / over-exploration (28): hard budgets (retries, tool calls, tokens, wall-clock) enforced in
    code, outside the LLM; stall detector tripwire.
  - Goal drift over weeks (28,29): every task carries objective_id + acceptance + forbidden, lifted from
    durable plan state, not copied from a conversation.
  - Coordination drift / duplicate work (28): centralized delegation by the Conductor; no peer-to-peer
    worker chat; explicit ownership via leases.

================================================================================
9. BUILD ORDER (smallest viable loop first, then widen)
================================================================================

  Step 1  Backlog import: parse the 412 plan files + phase indexes into the control DB as task nodes
          (plan_id, deps, scope, acceptance, deterministic_check, status seed from existing
          bn_status_report.py / FINISH punch lists). One-time script.
  Step 2  Worker adapter: wrap ONE coding-agent harness (Codex CLI recommended) against one proxy; prove
          it can take a TaskSpec, edit a worktree, return a patch. Test on ONE easy plan end-to-end.
  Step 3  Gate runner: wire the existing verifier scripts as the off-agent VERIFY step; prove a known
          wire-dark plan FAILS and a good patch PASSES.
  Step 4  The loop: SELECT->CONTEXT->EXECUTE->VERIFY->INTEGRATE->RECORD for one worker, one task type.
  Step 5  Second worker + disjoint-write-set scheduler + serialized integrator. Now 2 concurrent.
  Step 6  Waves 1-5 + loop-until-dry completion + epoch compaction + budgets/tripwires + a tiny status
          dashboard (read the control DB; even a CLI table is fine to start).
  Step 7  (optional) Graduate the durable backbone to Temporal; add Opus-escalation lane for DEAD-END
          tasks the workers can't crack.

================================================================================
10. WHERE THIS LIVES
================================================================================

This is workflow TOOLING that operates ON .elai_cc, not ELAI product code. It belongs in a new top-
level dir (e.g. f:\.ELAI_workflow\conductor\), NOT inside .elai_cc. It runs as a Docker service in WSL
alongside the copied project. It reuses .claude/tools/*.py verifiers and the .claude/agents/*.md role
prompts as worker system prompts.

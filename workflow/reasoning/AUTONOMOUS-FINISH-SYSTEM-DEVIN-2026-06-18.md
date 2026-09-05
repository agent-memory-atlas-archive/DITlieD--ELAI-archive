# Autonomous Finish System (the Conductor) — DEVIN CLI / FREE-MODEL variant

Date: 2026-06-18 (adapted from AUTONOMOUS-FINISH-SYSTEM-2026-06-18.md)
Author: adaptation of the original 5-report design; harness swapped from Codex CLI -> Devin CLI.
Problem: finish ELAI automatically — drive ~412 plan files (B..K) plus base phases 0-4 through
implement -> wire -> integrate -> test -> audit -> polish, in repeated passes, using N Devin CLI
worker processes on FREE Devin models, with NO Claude/Opus needed as the orchestrator and ~0 paid
tokens for the bulk waves.

WHAT CHANGED FROM THE ORIGINAL (read this first)
  Everything about the TRUTH STACK, GATES, TASKSPEC, WAVES, and CONCURRENCY MODEL is harness-agnostic
  and carries over UNCHANGED. Only the WORKER HARNESS changes:
    original:  worker = Codex CLI, OpenAI Responses API, gpt-5.5 via two proxy base_urls + keys.
    this doc:  worker = Devin CLI (`devin -p`), Cognition-hosted FREE models (Kimi K2.7 / GLM-5.2 /
               SWE-1.6), driven headless; paid Opus/GPT used ONLY on the escalation lane.
  Rationale: the Devin Desktop / CLI model selector exposes Kimi K2.7, GLM-5.2, and SWE-1.6 as FREE
  (Kimi K2.6 shown "No quota used"). That makes the design's stated economics ("need 100 opus" -> ~0)
  achievable through ONE harness you already have installed, with no proxy/key plumbing.

  RUNS NATIVELY ON WINDOWS (key difference from the original). The original Codex variant required a
  WSL+Docker VM because it spoke to the dangerous codex proxy API (base_url=bmapi.020212.xyz, ZDR,
  network_access). THIS variant uses Devin's own hosted free models over Devin's normal auth — there is
  NO proxy, NO custom base_url, NO ZDR plumbing — so the whole system runs as ordinary Windows
  processes: the Conductor is a Windows Python process/service, and each worker is a `devin.exe -p`
  child process in a git worktree on F:. No WSL, no Docker required. (WSL remains OPTIONAL, only if you
  later want Linux-backed OS sandboxing for the workers — see section 11 O-5.)

  CLI GROUND TRUTH (verified 2026-06-18 against devin 2026.7.19 at
  C:\Users\user\AppData\Local\devin\cli\bin\devin.exe via `devin --help`):
    -p / --print [PROMPT]        non-interactive: process prompt, exit (the worker primitive)
    --prompt-file <FILE>         load the TaskSpec from a file
    --model <MODEL>              pick the model per spawn   [env: DEVIN_MODEL]
    --permission-mode <MODE>     auto | smart | dangerous   [env: DEVIN_PERMISSION_MODE]
                                 "dangerous" auto-approves all tools (needed for unattended runs)
    --sandbox                    OS-level Read/Write/network scoping from granted permission scopes
    --config <PATH>              per-spawn system instructions + tool visibility + permissions
                                 (strict parsing — unknown fields rejected)
    --export [PATH]              write the conversation (ATIF format) after each turn -> TaskResult
    -c / --continue              resume most recent session in cwd
    -r / --resume [SESSION_ID]   resume a specific session (crash recovery / re-grounding)
    devin list --format json     enumerate sessions programmatically
    devin acp                    JSON-RPC over stdio if `-p` is too coarse (session/new|prompt|cancel)
  UNVERIFIED / CONFIRM BEFORE BUILD: the exact `--model` identifier strings for the free models.
  The `--help` example shows family + version form ("claude-opus-4.6"); the free models likely map to
  strings like `kimi-k2.7`, `glm-5.2`, `swe-1.6` (or short `kimi` / `glm` / `swe`). Confirm via the
  in-session `/model` selector or `devin --model <guess> -p "ok"` before wiring workers.toml.

================================================================================
0. THE ONE REFRAME (unchanged — this is still the answer to "HOW")
================================================================================

The orchestrator must NOT be an LLM. Replace the by-hand Opus/orchestrate FSM with a small
deterministic Python daemon (the CONDUCTOR). It costs ~0 tokens. The N Devin CLI processes are
stateless WORKERS it dispatches. Opus/Claude becomes optional — used only for rare hard escalations,
not for the loop.

Net effect on the 3 stated failures (unchanged):
  - "need 100 opus to orchestrate"  -> orchestration is code, zero LLM cost. Scale = config.
                                       Bulk waves now run on FREE Devin models, so even the WORKER cost
                                       trends to zero for the easy 80%.
  - "agents leave things wire-dark" -> a worker CANNOT mark its own work done. The frozen, off-agent
                                       wire/build/test gates (already built in ELAI) are the only door
                                       to DONE. Missing evidence = FAIL (fail-closed).
  - "some finish, some don't"       -> durable task graph + leases + bounded retries + artifact-
                                       grounded state. Every task has one terminal state, survives a
                                       crash/restart, and is re-queued on failure until the gate passes.

================================================================================
1. WHAT YOU ALREADY HAVE (unchanged — 70% assembly, not invention)
================================================================================

ELAI already contains most of the research-recommended truth + verification stack. The Conductor
mostly WIRES these together and drives them from code instead of from a prompt:

  Research layer                       ELAI artifact that already implements it
  ----------------------------------   ------------------------------------------------------------
  Task graph w/ verifier contracts     The 412 plan files: WIRING_CONTRACT + DETERMINISTIC_CHECK +
                                        ACCEPTANCE blocks = a TaskSpec already.
  Append-only event log (history)      SSB (action_type rows, SCHEMA_VERSION, derived counters).
  Repo knowledge graph (retrieval)     elai.map / elai_graph.json / codegraph-elai MCP.
  Off-agent verifier gates (the door)  production_caller_check.py, python_wire_check.py,
                                        maintainability_check.py, traced_test_assert_check.py,
                                        gate_freeze_check.py, wiring_contract_check.py, cargo check/test.
  Frozen gate (worker can't weaken it) .claude/gate-manifest.txt + gate_freeze_check.py.
  Role separation (impl != certify)    .claude/agents/*.md (executor vs auditor vs context-compiler).
                                        Reusable as Devin `--config` system-instruction files.
  Bounded context packets              context-compiler agent logic; spec_link_check.py; phase indexes.
  Failure taxonomy / scoped tests      failure_taxonomy.py; cross_lang_diff.py; -p scoped crate tests.

What is MISSING is only the deterministic driver, the durable task store, and the Devin worker adapter.

================================================================================
1B. BOOTSTRAP PHASE (unchanged in spirit; harness note added)
================================================================================

A smaller deterministic BOOTSTRAP CONDUCTOR at F:\.ELAI_workflow\bootstrap-conductor\ keeps the
worker slots usable long enough to build, audit, fix, hand off to, and start the full Conductor.
Operations dashboard (NOT a chat UI) at http://127.0.0.1:8765.

Harness note for this variant: the bootstrap's "keep the accounts usable" concern becomes "respect
the FREE-tier rate/quota limits of the Devin models." There is no proxy/key rotation here; instead the
controller throttles to the free-model rate limit and falls back across {Kimi K2.7, GLM-5.2, SWE-1.6}
when one model is rate-limited or quota-exhausted. Same invariants otherwise:
  - one Devin process per worker slot; a global cap on concurrent `devin -p` processes,
  - everything runs on the native Windows filesystem (F:); a workspace lock prevents concurrent
    mutation of one resource (no VM mount step — paths are plain Windows paths),
  - retries bounded; respect measured cooldowns / 429s from the free tier,
  - a mutating run that disconnects after turn start becomes NEEDS_REVIEW, never an auto-replay,
  - state and leases survive controller restarts in SQLite WAL,
  - no bootstrap or full-system job commits or pushes.

================================================================================
2. ARCHITECTURE (5 planes; first 4 are state, last is cognition)
================================================================================

  +-------------------------------------------------------------------------+
  | CONDUCTOR (deterministic Python daemon, ~1-2k LOC, native Windows proc)  |
  |   - scheduler: pick next ready task per free worker slot                 |
  |   - lease manager: one task per slot, TTL, crash-safe reclaim            |
  |   - gate runner: runs the frozen verifier stack OFF-agent                |
  |   - integrator: serialized merge of worker branch -> main + post-gate    |
  |   - budgets/tripwires: retry cap, token cap, wall-clock, stall detect    |
  |   - model router: FREE model for bulk; escalate to PAID on repeat FAIL   |
  |   - 0 LLM calls                                                          |
  +-------------------------------------------------------------------------+
        |                 |                 |                 |
        v                 v                 v                 v
  [CONTROL DB]      [EVENT LOG]       [KNOWLEDGE GRAPH]   [ARTIFACT STORE]
  SQLite (WAL),     append-only       elai.map +          per-task worktree
  AUTHORITATIVE     (own table or     codegraph-elai      diffs, ATIF exports,
  current state:    ELAI SSB):        (retrieval only,    test logs, gate
  task graph,       every transition  NON-authoritative)  outputs, audit
  state, leases,    for replay/audit                      reports, receipts
  verdicts
        |
        v  (dispatch one TaskSpec to one free worker slot)
  +-------------------------------------------------------------------------+
  | WORKERS (N Devin CLI processes; bulk on FREE models, scalable)          |
  |   each = `devin -p` subprocess pointed at ONE free model, running INSIDE |
  |   a git worktree (branch isolation), under --sandbox with a per-task     |
  |   --config. Stateless: TaskSpec (--prompt-file) in, patch + ATIF export  |
  |   out, process exits. Roles are TASK TYPES, not dedicated agents (you    |
  |   time-multiplex PLAN/EXECUTE/WIRE/INTEGRATE/AUDIT/POLISH across slots).  |
  +-------------------------------------------------------------------------+

Truth ownership (unchanged — do not let one store carry all truth):
  - Control DB = truth about CURRENT commitments (who holds what, what passed).
  - Event log  = truth about HISTORY (replay, forensics).
  - Gate stack = truth about CORRECTNESS (reproducible, off-agent, fail-closed).
  - Knowledge graph = truth about RELEVANCE only (derived, rebuildable, never arbitrates "done").

================================================================================
3. THE TASKSPEC / TASKRESULT CONTRACT — the worker interface
================================================================================

The Conductor hands each worker a small, bounded TaskSpec, written to a file and passed via
`--prompt-file`. Re-grounded from durable state every time (never rely on one initial prompt):

  TaskSpec (rendered to a prompt file):
    task_id, plan_id (e.g. "I-34")
    role               # PLAN | EXECUTE | WIRE | INTEGRATE | AUDIT | POLISH
    objective          # one paragraph, lifted from the plan file's intent
    base_commit
    allowed_write_set  # files/dirs this task may touch -> ALSO the --sandbox Write() scopes
    context_refs       # bounded packet: the plan file + the specific spec slice + graph neighborhood
    production_hook    # the EXACT seam (file+fn) the new symbol must be called from (D-030)
    acceptance         # the plan's ACCEPTANCE + DETERMINISTIC_CHECK blocks, verbatim
    forbidden          # no editing the gate/tests that judge it; no destructive git (preamble R-1..R-8)
    verifier           # the exact gate command the Conductor will run to judge this task
    lease_ttl, idem_key, token_budget, model_tier(FREE|PAID)

  TaskResult (parsed from the ATIF --export file + worktree git state):
    task_id, base_commit, touched_files, branch, artifact_refs, status(DONE/BLOCKED/FAILED), notes

Key discipline (unchanged): scope to a SMALL dependency neighborhood. "I fixed it" is never a state
transition; "verifier V accepted artifact A for task T" is the only one.

The .claude/agents/*.md role prompts (executor, auditor, context-compiler, etc.) become the Devin
`--config` system-instruction file for the matching role, so the worker inherits the same role
contract it had under Claude Code.

================================================================================
4. THE PER-TASK LOOP (replaces the human-driven FSM) — unchanged flow
================================================================================

  1. SELECT   pick highest-priority READY task (all deps integrated to main) for a free slot.
  2. CONTEXT  build the bounded context packet (cheap retrieval over elai.map + codegraph + the plan
              file + its "Spec references" slice). Deterministic; optionally a cheap FREE-model sub-call.
  3. EXECUTE  create a fresh worktree off base_commit; spawn `devin -p` (see section 7 driver) on a
              FREE model, scoped by --sandbox to allowed_write_set. Worker edits, wires the
              production_hook, returns a patch + ATIF export.
  4. VERIFY   Conductor (off-agent) runs the frozen gate stack against the worktree:
                cargo check + scoped cargo test -p, wiring checkers (DARK symbol = FAIL),
                maintainability_check, traced_test_assert_check, the plan's DETERMINISTIC_CHECK.
              FAIL -> structured feedback appended -> re-queue (retry cap). On repeated FREE-model FAIL,
              the model router escalates the retry to a PAID model (see section 7.5).
  5. INTEGRATE on pass, the integrator rebases the branch onto current main and merges. SERIALIZED:
              one merge at a time even with N workers. Post-merge gate re-run. Conflict -> INTEGRATE task.
  6. RECORD   append event(s), store the ATIF + gate artifacts, mark DONE in control DB, reindex graph.

Epoch compaction: every K completed tasks, write a fresh state summary (memory-bank/handoff files are
the natural home) and start a new epoch so history never grows unbounded.

================================================================================
5. THE COMPLETION WAVES — unchanged
================================================================================

  Wave 1  IMPLEMENT    every PLANNED/PARTIAL plan -> EXECUTE task -> build+wire+scoped-test gate.
  Wave 2  WIRE-SWEEP   wire checkers repo-wide; every DARK symbol -> WIRE task.
  Wave 3  INTEGRATION  per-subsystem R10 cross-seam smoke on the REAL entrypoint, HARD input.
  Wave 4  AUDIT        adversarial-auditor pass per plan -> findings -> FIX tasks.
  Wave 5  POLISH       maintainability_check repo-wide + dashboard visual-ui-loop + docs/dead-code.
  Wave N  RE-PASS      repeat AUDIT+POLISH until the full gate stack is green repo-wide with ZERO new
                       findings for K consecutive passes (loop-until-dry).

COMPLETION IS SYSTEM-LEVEL: done == milestone graph closed AND global gate stack green on merged head
AND K dry audit passes. Never a worker's self-report.

Model-tier policy across waves: Waves 1-3 (IMPLEMENT/WIRE/INTEGRATE) run on FREE models — they are
the high-volume, mechanically-checkable waves where the off-agent gate fully constrains quality.
Wave 4 AUDIT runs the adversarial-auditor role; run it on the strongest FREE model first (Kimi K2.7 or
GLM-5.2), escalate to PAID Opus/GPT only for plans where the free auditor + gate disagree or stall.

================================================================================
6. CONCURRENCY MODEL (Devin-specific — this is the biggest change)
================================================================================

Still a work queue with N consumers. But the consumer cap is NOT "number of API keys" anymore — the
free models are account-quota / rate-limited, not key-sharded. So:

  - concurrency = min(local-machine capacity, Devin free-tier rate limit) concurrent `devin -p`
    processes, each in its own git worktree. Start at 2 (matches the original's tested shape), raise
    only if the free tier and your CPU/RAM tolerate it.
  - the controller spreads load across the free model pool {kimi-k2.7, glm-5.2, swe-1.6} and treats a
    429 / quota-exhausted response as a transient cooldown on THAT model, not a task failure — re-queue
    onto a different free model.

  workers.toml (Devin variant):
    [[worker]]  name="w1"  harness="devin-cli"  model="swe-1.6"     tier="free"
    [[worker]]  name="w2"  harness="devin-cli"  model="glm-5.2"     tier="free"
    # optional 3rd free slot:
    # [[worker]] name="w3" harness="devin-cli"  model="kimi-k2.7"   tier="free"
    [[escalation]] name="esc" harness="devin-cli" model="opus"      tier="paid"  # see 7.5

Collision constraints the Conductor enforces (unchanged):
  - disjoint allowed_write_set between concurrently-running tasks,
  - one worktree per task, serialized integration to main,
  - per-task token_budget + global daily budget. For FREE models the "budget" is really a
    rate/quota guard so a runaway worker can't trip the account's free-tier limit and stall all slots.

================================================================================
7. BUILD CHOICE + THE EXACT DEVIN DRIVER
================================================================================

Option A (RECOMMENDED to start): thin custom Conductor, Devin CLI worker.
  SQLite (WAL) + ~1-2k LOC Python loop + subprocess-driven `devin -p` workers. You own every verifier
  script; the Conductor just calls them. Lowest risk, fastest to a working loop. Devin CLI already
  reads AGENTS.md / .claude rules / skills / MCP, so the worker inherits ELAI context for free.

Option B (graduate to, if/when needed): LangGraph (cognitive runtime) + Temporal (durable execution).
  Adopt ONLY once Option A proves the loop and you hit real crash-recovery / multi-week-history pain.

7.1 THE WORKER DRIVER (the section-11 "exact driver", Devin form)

  Per task, the Conductor (pseudo-shell; real impl is Python subprocess):

    git -C <repo> worktree add <wt> <base_commit>
    # render TaskSpec -> <wt>/.task/taskspec.txt ; render role prompt -> <wt>/.task/role.config
    DEVIN_MODEL=<free-model>  DEVIN_PERMISSION_MODE=dangerous  \
      devin -p \
        --model <free-model> \
        --permission-mode dangerous \
        --sandbox \
        --config <wt>/.task/role.config \
        --prompt-file <wt>/.task/taskspec.txt \
        --export <wt>/.task/result.atif.json \
        --respect-workspace-trust false
    # process runs headless inside <wt>, exits. Then:
    # 1) read <wt>/.task/result.atif.json  -> notes / status hint (NOT authoritative for DONE)
    # 2) `git -C <wt> status/diff`          -> touched_files, branch
    # 3) run frozen gate stack over <wt>    -> the ONLY authority on DONE

  Why each flag:
    --config permission scopes           constrain the worker to allowed_write_set via the config file's
                                         permissions block (allow/ask/deny on Write(...)/Read(...)/
                                         Exec(...)). On Windows this PERMISSION-LEVEL scoping is the
                                         primary containment (see --sandbox note below).
    --sandbox (OPTIONAL on Windows)      enforces the Read/Write scopes at the OS level. `--help` lists
                                         only macOS seatbelt / Linux bwrap+seccomp — OS-level sandbox is
                                         likely a no-op on native Windows. So DO NOT rely on it for
                                         containment here; the real containment is the worktree boundary
                                         + the --config Write() permission scopes + the frozen gate +
                                         the git-read-only dispatch preamble (R-1). Pass --sandbox only
                                         when running workers inside an optional WSL slot (O-5).
    --permission-mode dangerous          unattended: no human approval prompts. SAFE ONLY because the
                                         worktree isolation + --config Write() scopes + frozen gate +
                                         git-read-only preamble contain it — NOT because of OS sandbox.
    --export                             machine-readable TaskResult; written each turn, read on exit.
    --respect-workspace-trust false      non-interactive runs must not block on a trust prompt.
    --config strict parsing              catches role-prompt typos early (unknown fields rejected).

7.2 RESUME / CRASH RECOVERY
  Devin sessions are durable and per-directory. `devin list --format json` (run with the worktree as
  cwd) enumerates sessions for lease reclaim; `devin -r <session_id>` resumes a worker that died
  mid-turn. A mutating run that disconnected after turn start -> NEEDS_REVIEW, never auto-replay.

7.3 IF `-p` IS TOO COARSE
  `devin acp` exposes JSON-RPC over stdio (session/new, session/prompt, session/cancel, streamed
  tool_call + request_permission). Use it only if you need per-tool-call control or live cancellation
  beyond what one-shot `-p` gives. Start with `-p`; graduate to acp only on demonstrated need.

7.4 FREE-MODEL POOL
  Bulk worker models (confirm exact `--model` strings first — see top-of-file CLI ground truth):
    swe-1.6     fast, cheap-intelligence — best default for mechanical EXECUTE/WIRE tasks.
    glm-5.2     stronger general reasoning — INTEGRATE / harder EXECUTE.
    kimi-k2.7   strongest free reasoning — AUDIT-role first pass and tricky wiring.
  All three are FREE in the current selector. Round-robin / capability-route across them; treat any
  one being rate-limited as a cooldown, not a failure.

7.5 ESCALATION LANE (the only place paid tokens are spent)
  A task that FAILS the gate after R free-model retries (R configurable, e.g. 3) and after trying a
  second free model is escalated to a PAID model (opus / gpt) via the SAME driver with
  --model opus. This is the doc's "Opus used only for rare hard escalations" — now a discrete, metered
  lane with its own daily paid-token budget and a tripwire that pauses escalation if the budget is hit.

================================================================================
8. RISKS THE REPORTS FLAG (+ Devin-specific controls)
================================================================================

  - Context rot / lost-in-the-middle: keep TaskSpec + context packet SMALL; decompose by dependency
    frontier; re-ground from durable state each task.
  - Hallucinated progress / reward hacking: off-agent gate is the only door to DONE; worker never edits
    the gate/tests that judge it (frozen via gate-manifest + gate_freeze_check). `forbidden` block +
    --sandbox scopes physically prevent the worker from touching the gate files.
  - Semantic merge conflicts (~27% of agentic PRs): serialized integrator + post-merge gate; disjoint
    write sets for concurrent tasks.
  - Looping / over-exploration: hard budgets (retries, tool calls, wall-clock) in code, outside the LLM;
    stall detector tripwire.
  - Goal drift over weeks: every task carries objective_id + acceptance + forbidden from durable plan
    state, not copied from a conversation.
  - DEVIN-SPECIFIC — free-tier rate/quota stalls: a model being throttled must NOT look like a task
    failure. The controller distinguishes 429/quota (cooldown + re-route across the free pool) from a
    gate FAIL (re-queue / escalate). Without this split, a quota blip would burn the retry cap and
    spuriously escalate to paid models.
  - DEVIN-SPECIFIC — `dangerous` permission mode: NEVER run a worker in dangerous mode without
    --sandbox AND the worktree Write() scopes AND the frozen gate. The autonomy is contained by the OS
    sandbox + the verifier, not by trusting the model.
  - DEVIN-SPECIFIC — free-model capability ceiling: free models will fail more hard tasks than gpt-5.5.
    That is by design absorbed by the escalation lane; budget for a non-trivial fraction of plans
    landing on the paid lane, and watch the escalation rate as the headline cost metric.

================================================================================
9. BUILD ORDER (smallest viable loop first, then widen)
================================================================================

  Step 0  CONFIRM the free `--model` identifier strings (selector or `devin --model <x> -p "ok"`).
  Step 1  Backlog import: parse the 412 plan files + phase indexes into the control DB as task nodes
          (plan_id, deps, scope, acceptance, deterministic_check; seed status from bn_status_report.py
          / FINISH punch lists). One-time script. (Unchanged from original.)
  Step 2  Devin worker adapter: a Python fn that takes a TaskSpec, sets up a worktree, renders the
          prompt + role --config, spawns `devin -p` on a FREE model, and returns the parsed ATIF +
          git-diff TaskResult. Test on ONE easy plan end-to-end. (THIS is the new piece vs the Codex doc.)
  Step 3  Gate runner: wire the existing verifier scripts as the off-agent VERIFY step; prove a known
          wire-dark plan FAILS and a good patch PASSES.
  Step 4  The loop: SELECT->CONTEXT->EXECUTE->VERIFY->INTEGRATE->RECORD for one worker, one task type.
  Step 5  Second free worker + disjoint-write-set scheduler + serialized integrator + free-pool
          rate-limit/cooldown router. Now 2 concurrent on free models.
  Step 6  Waves 1-5 + loop-until-dry + epoch compaction + budgets/tripwires + the escalation lane
          (7.5) + a tiny status dashboard (read the control DB; a CLI table is fine to start).
  Step 7  (optional) Graduate the durable backbone to Temporal.

================================================================================
10. WHERE THIS LIVES
================================================================================

This is workflow TOOLING that operates ON .elai_cc, not ELAI product code. It belongs in a new top-
level dir (e.g. f:\.ELAI_workflow\conductor\), NOT inside .elai_cc. It runs as a NATIVE WINDOWS Python
process/service (no WSL, no Docker required — this variant has no proxy/ZDR plumbing). It reuses
.claude/tools/*.py verifiers and the .claude/agents/*.md role prompts as worker `--config` system
prompts. The Devin binary it drives is C:\Users\user\AppData\Local\devin\cli\bin\devin.exe.

================================================================================
11. OPEN ITEMS SPECIFIC TO THIS VARIANT (resolve before/early in the build)
================================================================================

  O-1  Exact free `--model` strings (Step 0). BLOCKS workers.toml.
  O-2  Does Devin headless `-p` honor the free-tier quota the same way the GUI does? Almost certainly
       yes (same harness/account), but verify one full free-model `-p` run end-to-end before scaling.
  O-3  ATIF schema: confirm the fields you need (final assistant message, tool-call list, stop reason)
       are present in --export output, so TaskResult parsing is stable. Treat ATIF as advisory; the
       gate + git state remain authoritative for DONE.
  O-4  Free-tier concurrency limit: empirically find how many simultaneous free `devin -p` sessions the
       account tolerates before 429s dominate; that sets the real worker cap (section 6).
  O-5  Sandbox on Windows (NON-BLOCKING — the system runs on Windows regardless): `--help` lists only
       macOS seatbelt / Linux bwrap+seccomp for --sandbox, so OS-level sandboxing is probably a no-op on
       native Windows. This does NOT force WSL: containment here is the git-worktree boundary + the
       --config Write() permission scopes + the frozen off-agent gate + the git-read-only dispatch
       preamble (R-1), none of which need an OS sandbox. WSL is an OPTIONAL hardening slot ONLY if you
       later decide you want Linux-backed --sandbox on top of those four layers; the default deployment
       is plain Windows processes.

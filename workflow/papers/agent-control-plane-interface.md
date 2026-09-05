Source: agent-control-plane (github.com/coleam00/agent-control-plane)
Reliability: MEDIUM (reviewed open source, 2026-06-15 last commit, shallow clone)
Files examined:
  backend/src/server.ts
  backend/src/loop.ts
  backend/src/pi.ts
  backend/src/db.ts
  backend/src/config.ts
  backend/src/itest.ts
  backend/migrations/001_init.sql
  backend/migrations/002_orchestrator.sql
  frontend/src/types.ts
  README.md

Stack: Bun + TypeScript (backend), Hono HTTP server, Neon serverless Postgres,
       Vite + React (frontend dashboard), Pi coding agent (external binary, pi.dev).
License: MIT (LICENSE file, copyright Cole Medin)
Last commit: 2026-06-15 (commit 7fd7b55, docs-only: provider-independence note)
Stars/activity: ~recently active, demo/tutorial-grade project

Architecture overview:
  REST API (Hono on Bun port 8787) -> in-process loop runner (async, keyed by loop UUID) ->
  Pi binary subprocess (pi --mode json --print) -> Neon Postgres (loops/runs/run_events tables).
  Frontend polls REST API. No WebSocket / SSE; dashboard uses polling.
  State store is Neon (external cloud Postgres), not local SQLite.


Public API:
  POST /api/loops          | {goal:string, maxIterations?:int, mode?:"orchestrated"|"ralph"} | Loop | starts loop async; returns immediately
  GET  /api/loops          | none | Loop[] | list 50 most recent loops ordered by updated_at desc
  GET  /api/loops/:id      | none | Loop+runs | single loop with its Run[]
  POST /api/loops/:id/resume | {extraIterations:int} | Loop | human gate: extend cap + re-enter loop
  POST /api/loops/:id/pause  | none | Loop | set pauseRequested flag; stops after current round
  POST /api/loops/:id/stop   | none | Loop | set stopRequested flag + DB update immediately
  GET  /api/runs           | none | Run[] | 100 most recent runs global
  GET  /api/runs/:id       | none | Run+events | single run with RunEvent[] (up to 500)
  GET  /api/health         | none | {ok:bool, model:string} | liveness

startLoop(input) -> Loop | loop.ts:206 | fires runLoop(id) as void (fire-and-forget, no await)
resumeLoop(id, extra) -> Loop|null | loop.ts:223 | guard: if controls.get(id).running return loop (no double-start)
pauseLoop(id) -> Loop|null | loop.ts:237 | cooperative: sets flag, actual pause happens at round boundary
stopLoop(id) -> Loop|null | loop.ts:247 | sets stopRequested + DB immediately; mid-round stop wins over outcome
runLoop(id) -> void | loop.ts:261 | inner async loop; on exit always deletes from controls map
parseDecision(text) -> Decision|null | loop.ts:164 | exported; extracts fenced json block first, falls back to brace-scan, newest candidate wins
ensureWorkspace(loop) -> string | loop.ts:180 | mkdir -p workspaces/<loop.id>; writes PROGRESS.md if absent
runPiTask(prompt, opts) -> PiResult | pi.ts:54 | spawns pi subprocess, streams LF-delimited JSON events, kills on timeout
reconcileInterruptedLoops() -> number | db.ts:119 | on startup: updates running/pending -> failed; preserves paused/awaiting_approval

Behavioral contracts:
  startLoop -> createLoop in DB (status=pending), then fires runLoop as detached async (void); HTTP returns before loop starts executing
  runLoop -> in-process state in controls Map<string,Control>; does NOT survive process restart; DB is sole durable state
  reconcileInterruptedLoops -> called at server startup; marks any running/pending loops as failed (no zombie recovery, no resume of interrupted state)
  pauseLoop (running) -> sets control flag; loop actually pauses after CURRENT round completes (not mid-round)
  pauseLoop (not running) -> updates DB directly to paused
  stopLoop -> ctl.stopRequested=true wins over round outcome (stop mid-round is safe; the if(ctl.stopRequested) check in runLoop line 298)
  orchestratedIteration -> orchestrator agent runs first (read-only tools), parses decision JSON, then fans out workers via Promise.all (parallel, up to MAX_FANOUT=4); worker failure is non-fatal: recorded but loop continues
  ralphIteration -> single worker per round; LOOP_STATUS: DONE regex in output ends loop
  runPiTask -> writes prompt to tempfile (avoids argv length limits); spawns pi --mode json --print --no-session; per-event callback fires for tool_start/tool_end/text/turn_end; cleanup: rm tempdir on finish
  runPiTask (timeout) -> killTimer fires proc.kill(); result.isError=true; cleanup still runs
  tool error (non-fatal) -> tool_execution_end with isError=true is surfaced as event only; does NOT set result.isError (only genuine Pi-level failures do: stopReason=error, nonzero exit, timeout, message_update error type)
  persistEvent -> fire-and-forget (.catch(()=>{})); event loss on DB error is accepted
  updateLoop (status+last_error) -> atomic single SQL write to prevent reader seeing status=failed without error
  initSchema -> idempotent; applies all migrations/*.sql in filename order; strips SQL comment lines before splitting on semicolons
  ACP_FAKE_PI=1 -> substitutes fakePiRun; detects role from prompt text; maintains counter on disk; never fires in normal operation

Platform constraints:
  Requires Bun runtime (not Node); pi binary must be on PATH or PI_BIN env var
  Requires external Neon Postgres (DATABASE_URL env var, mandatory); no embedded/local DB option
  No process restart survival for in-flight loops: controls Map is in-process only
  Pi subprocess inherits process.env; model auth via env vars (ANTHROPIC_API_KEY, OPENAI_API_KEY, etc.) or pi subscription login (~/.pi/)
  pi binary version: tested against pi 0.79.1 (pi.ts line 9 comment)
  Max parallel workers per round: MAX_FANOUT=4 (hardcoded constant, loop.ts:36)
  Loop max_iterations: clamped 1-100, default 10 (server.ts:39)
  Pi run timeout: default 900s (15 min) per run (config.ts)
  No auth on HTTP API (CORS wide open); designed for local/tunneled use or behind Retool managed auth

Gotchas:
  Loop state does NOT survive server restart: any running/pending loop is reconciled to failed on next boot; no replay or resume of mid-run state
  pauseLoop is cooperative (round-boundary), not preemptive: a 15-min Pi run continues to completion before pause takes effect
  Worker failures are non-fatal by design: orchestrator sees them next round and adjusts; a consistently-failing worker will exhaust the iteration cap without the loop erroring out
  PROGRESS.md is the only cross-round memory for agents: if deleted or corrupted, agents lose all prior context
  Decision parsing is fragile to orchestrator output format: if Pi does not emit valid JSON in a fenced block or balanced braces, the iteration fails with "could not parse the orchestrator's decision"
  No deduplication of parallel worker writes to PROGRESS.md: concurrent workers in same round can race on that file
  persistEvent is fire-and-forget: tool events can be lost silently if Neon is slow/down
  Neon HTTP query (not pooled Postgres): every DB call is a discrete HTTP round-trip; high-frequency event appends under load will be slow
  reconcileInterruptedLoops sets last_error to a fixed string, not the actual failure reason

Critical assertions (testable):
  assert loop.status in ("pending","running","awaiting_approval","paused","completed","failed","stopped")
  assert run.role in ("orchestrator","worker")
  assert run.parent_run_id is None or run.role == "worker"  # only workers have a parent orchestrator run
  assert len(decision.tasks) <= 4  # MAX_FANOUT cap
  assert loop.iterations <= loop.max_iterations or loop.status == "awaiting_approval"
  assert not (loop.status == "failed" and loop.last_error is None)  # atomic write guarantee
  assert loop.status != "running" if server just restarted  # reconcileInterruptedLoops clears it

ELAI coverage analysis:
  loops/runs/run_events DB schema -> ELAI has SSB (SQLite event journal) with action_type rows; covers equivalent observability but single-writer append-only vs Neon's mutable update model; ELAI does NOT have per-run cost/token tracking in the same row shape
  orchestrated mode (LLM decides next tasks) -> ELAI FSM + agent-pool dispatches agents per phase; ELAI's orchestrator is the main LLM + skill system, not a Pi-spawned agent; the MECHANISM differs (ELAI = FSM-driven static phases; ACP = LLM-dynamic per-round task decomposition)
  ralph mode (single-agent loop, sentinel exit) -> ELAI has no equivalent single-agent re-loop; ELAI's task_runner runs babysteps not open-ended loops
  human gate (awaiting_approval on cap hit) -> ELAI has no equivalent; ELAI stops cleanly but has no "park and wait for human approval to resume" state
  crash recovery (reconcileInterruptedLoops) -> ELAI supervisor has crash recovery; BUT ELAI recovers and resumes; ACP reconciles to failed (no replay)
  Pi binary subprocess driving -> ELAI has its own sidecar (Qwen2.5 Python server); NOT equivalent (ACP drives Pi which drives any LLM; ELAI sidecar IS the local LLM)
  cost/token per run tracking -> ELAI SSB has token_usage events; ACP stores cost_usd + input_tokens + output_tokens directly on the run row with parent_run_id linkage
  PROGRESS.md as cross-round agent memory -> ELAI has no equivalent; ELAI agents get context via context-package.md injected per dispatch, not a living file agents update themselves
  tool restriction by role (orchestrator=read-only) -> ELAI has no equivalent; all ELAI agents get their tool set from the agent .md definition; no runtime tool-surface restriction by agent role
  MAX_FANOUT cap (4) -> ELAI has parallel_cap=3 in orchestrate skill; equivalent concept, different value

Novel mechanisms vs ELAI (differentiated, not covered):
  1. LLM-dynamic task decomposition per round: orchestrator agent decides WHAT the next tasks are each round from scratch (not from a static plan file). ELAI uses pre-authored phase files; the plan is static, the agents execute it. ACP has no plan file; the orchestrator writes the plan dynamically each round.
  2. awaiting_approval human gate: loop parks at iteration cap and requires explicit POST /resume to continue; enables async human-in-the-loop approval on expensive long-running loops. ELAI has no equivalent park-and-wait state.
  3. per-run parent_run_id lineage: every worker run carries parent_run_id pointing to the orchestrator run that spawned it; enables tracing "this worker output came from this orchestrator decision" in the DB. ELAI SSB has session_id but no explicit parent-child run lineage.
  4. tool-surface restriction by agent role: ORCHESTRATOR_TOOLS = ["read","ls","find","grep"] hardcoded; workers get full tool set. ELAI does not restrict tool surface at runtime by role.
  5. ACP_FAKE_PI deterministic test harness: a prompt-injected fake that reads control tokens ([steps=N],[fanout=K],[fail]) from the goal string and maintains on-disk counters; enables deterministic integration tests of the full loop without a real LLM or network. ELAI has no equivalent deterministic fake for its sidecar.

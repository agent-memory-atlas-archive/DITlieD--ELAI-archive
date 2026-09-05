SUBSYSTEM: teikoku-walk-control-canvas (SP-2B, unit 2.2: per-run provider picker)
TAGS: CROSS_BOUNDARY
SESSION: s-finish-0705
REPO ROOT: WSL distro "teikoku", /home/user/teikoku (accessed via `wsl -d teikoku -- bash -c`; Bash tool cannot reach it, broad Glob over \\wsl.localhost\ times out -- all reads below used PowerShell+wsl or targeted \\wsl.localhost UNC Reads)
CONFIDENCE: 0.85 (every claim below is a direct grep/read citation; two items marked UNVERIFIED)

PRODUCTION_HOOK CHAIN (today, no picker exists -- this is the full seam an executor threads a param through)
1  signal/src/lib/screens/CanvasMonitor.svelte:98 toggleWalk() (button at :571-580) -> startWalk()/stopWalk()
2  signal/src/lib/api/orchestration.ts:271-272 startWalk/stopWalk -> _walkAction() :262-270
     -> fetch(`${HTTP_BASE}/walk/${action}`, POST). fetch-ONLY: no Tauri invoke counterpart (contrast
     the main snapshot getter, which has both invoke('signal_orchestration') AND fetch, per file
     header :4-6). HTTP_BASE not read in this file directly (grep truncated by head -20) but is
     'http://127.0.0.1:8787' in every sibling api/*.ts (ask.ts:56, chat.ts:230, config.ts:23) -- same
     constant, near-certain same value here (UNVERIFIED literal line, HIGH confidence by convention).
3  conductor/slice_api.py:1263 "/walk/start" -> :1265 walk_control.walk_start(); :1266 "/walk/stop" -> :1268 walk_control.walk_stop()
4  conductor/walk_control.py:49 walk_start() -> dict. NO PARAMETERS TODAY. subprocess.Popen(["bash",
     DRIVER], start_new_session=True), DRIVER=elai_walk_driver.sh (:18). Idempotent (:53-54 no-op if
     already running).
5  conductor/elai_walk_driver.sh: while-loop hardcodes
     `python3 conductor.py run-backlog --project elai --config config/conductor.elai-walk.toml
      --db state/control.db --limit-per-wave 2` then `diskguard.py --reap`. NO --tier/--pool/
     --provider flag exists on run_backlog (conductor.py:415); the only --model-tier flag in the
     whole CLI is on the unrelated `spawn` subcommand (conductor.py:921, default FREE).
6  run-backlog drains via scheduler.py -> leases.py LeaseManager.acquire_ready() (leases.py:69).
7  THE ACTUAL PIN/MATCH LOGIC (leases.py:69-220), in order: (a) :156 slot.model_tier == task.model_tier
     (coarse FREE/PAID gate -- the dominant lever for the walk); (b) :158-164 role->slot affinity via
     worker_slots.roles CSV (NULL = serves all roles); (c) :165-174 account/pool dedup + P3-A
     rate-limit-pool exclusion; (d) :175-183 _slot_can_serve() pinned_model/required_context_window
     check, backed by model_pool.py:375-386 lane_serves_model().
8  pinned_model column (conductor_db.py:74, ALTER TABLE :164-165) is written by 3 call sites, NONE on
     the walk path: chat_orchestrator.py:622 (interactive chat), headless_runner.py:309+319/331-343
     _enforced_pin (the /headless/run new-objective path, gated by route_decider.
     intelligent_routing_enabled() D-07 kill-switch), scheduler.py:483 (runtime FREE->PAID overflow
     escalation, TR-3). The walk's backlog tasks get model_tier at IMPORT time, never via route_decider.
9  THE REAL "PICKER" TODAY: conductor/retier_elai.py, a standalone offline CLI (no HTTP route, no
     caller in slice_api.py/walk_control.py) doing `UPDATE tasks SET model_tier=? WHERE status!='DONE'
     AND role IN (STAGE_ROLES)` (:21-24). Its own docstring (:1-6) calls this "Mode switch for the
     walk: glm lane: --tier FREE / opus lane: --tier PAID" -- an operator SSHes in and runs it BEFORE
     clicking Start. This is the gap SP-2B closes.
10 SECOND, INDEPENDENT BLOCKER even after a tier flip: config/conductor.elai-walk.toml [limits]
     daily_paid_attempt_cap = 0 hard-zeroes PAID attempts for this walk's config, enforced in
     budgets.py:518 (`if daily_paid_attempt_cap is not None and <=0: refuse`), threaded via
     scheduler.py:66-78 constructor. No conductor.elai-walk-paid.toml variant exists (only one
     elai-walk*.toml file in config/). Flipping model_tier alone does NOT unlock PAID for the walk.

Q1 answer (repeated, condensed): pin mechanism = tier equality at lease time (leases.py:156) +
role affinity (leases.py:158-164), NOT the richer route_decider/pinned_model path (that's chat +
headless-enqueue only). Today's "config selection" = retier_elai.py's blunt UPDATE (offline, manual)
PLUS the static per-walk-config daily_paid_attempt_cap. SP-2B must thread BOTH: (a) a tier param into
walk_start that performs retier_elai.py's UPDATE inline (reuse its SQL/scope, don't duplicate -- it's
28 lines, importable as a fn) or shells out to it; (b) either select between two config files
(new elai-walk-paid.toml with daily_paid_attempt_cap>0) or add a walk_start(tier) branch that loads a
tier-specific limits override before Popen. leases.py:69 already exposes exclude_paid/excluded_pools
params on acquire_ready -- REUSE these rather than inventing a new filter if the picker needs
per-pass (not per-task-row) exclusion; conductor.py run_backlog does not currently thread them in.

Q2 answer: assembled at orchestration_api.py:557 `"walk": _walk_status()` inside
build_orchestration_snapshot() (:502). _walk_status() (:491-499) imports walk_control, calls
walk_status() (walk_control.py:37-46, pgrep-based), returns {available, running, stop_pending} only
(fail-soft to {available:false, reason} on any exception, :498-499). TS mirror WalkStatus type
(orchestration.ts:182-187) has the identical 4 fields, ZERO provider/tier field today.
"active_provider" as a NAME exists ONLY in usage_meter.py:324,340 (GET /usage, the interactive CHAT
session's per-provider usage readout) and its TS mirrors (context.ts:80, chat.ts:65) -- a completely
different subsystem (chat model picker, not the walk). An equivalent field for the walk does not
exist; natural source for a new one is the same GROUP BY model_tier query retier_elai.py:27 already
runs, or the name of the currently-loaded elai-walk config.

Q3 answer: /headless/run route at slice_api.py:1257 -> handler :303-318 -> headless_runner.run_headless
(client call: chat.ts:487 POST). SEPARATE path from the walk: it ENQUEUES new objectives
(headless_runner.py:267 _enqueue_objectives, spawn_depth=1 via _spawn.enqueue_spawn:322) and DOES run
them through route_decider.RouteDecider.decide() (:301) + _enforced_pin (:309/331-343), setting
pinned_model on the new SpawnDescriptor. The walk never calls this -- it drains pre-existing READY
rows via run-backlog, so headless's "smart" per-objective routing never touches walk tasks. This is
the scope fork the orchestrator should resolve explicitly: a "provider picker" could stay walk-shaped
(mirror retier_elai's blunt tier flip, small) or import route_decider's smarter logic (much bigger).

Q4 answer: FE reaches backend via plain fetch to HTTP_BASE='http://127.0.0.1:8787' (slice_api.py
ThreadingHTTPServer bind at :31,1434; confirmed LIVE right now, pid 299 in `ps aux`). :8787 is a pure
JSON API (BaseHTTPRequestHandler dispatch table), it does NOT serve built dashboard assets -- that
is a DIFFERENT, unrelated legacy server (dashboard_server.py:21 DASHBOARD_DIR=dashboard_static, its
own ThreadingHTTPServer). CanvasMonitor.svelte is served by Vite dev (signal/package.json:
"dev": "vite --port 1420 --strictPort") or the Tauri shell ("tauri:dev"/"tauri:build"); NEITHER
process is currently running (`ps aux` found no vite/tauri match). No build step is needed to see an
edit under dev (Svelte 5 + Vite HMR on save); `bun run build`/`tauri:build` only matters for a
distributable bundle. Executor must START `bun run dev` (or tauri:dev) in signal/ to see changes live
-- it is not already up.

Q5 (half-built pieces, do not duplicate): worker_providers.py:60-62 ROUTE_POOL_CLAUDE="claude",
ROUTE_POOL_DEVIN_FREE="devin-free", ROUTE_POOL_FREE_API="free-api" (:68 FREE_POOL_PREFERENCE tuple) --
these are the canonical provider identifiers, reuse them verbatim for any picker enum/options list.
route_decider.py's RouteDecision dataclass (lane field :110) + RouteContext + intelligent_routing_
enabled() D-07 kill-switch are the "smart" routing layer (headless/chat only, see Q3). leases.py
acquire_ready already accepts exclude_paid/excluded_pools (:77-78) -- unused by run-backlog today.
config.ts:1-20 documents an existing "pool-caps" override seam (control_state KV, floor-clamped
server-side, :59-73) -- a DIFFERENT knob (concurrency ceiling per pool, not which pool serves a run);
do not conflate. No existing "active provider for the walk" field, no existing walk-specific tier
param anywhere in slice_api.py/walk_control.py/conductor.py's run_backlog. grep for provider|lane|
retier|affinity across conductor/+signal/src surfaced nothing else walk-shaped beyond what is cited
above.

FRESHNESS: read at 2026-07-05T13:08:39Z (WSL clock). walk_control.py mtime 2026-07-05 01:43 JST
(~9.5h stale relative to read, i.e. not mid-edit at read time); conductor_loop.py mtime 04:36 JST
same day (~8h stale). Neither showed an in-progress-save signature (no .swp/lock observed) but SP-1C
may still land changes after this snapshot -- re-diff both files before the executor edits them.

GAPS / RISKS
- No HTTP-reachable tier-flip fn exists; retier_elai.py is a bare script (argparse main(), sqlite3
  direct) with no importable fn boundary -- executor should extract its UPDATE into a small
  conductor helper both the CLI and walk_start(tier) call, per maintainability "reuse, don't
  duplicate" rather than shelling out to the script or copy-pasting the SQL.
- daily_paid_attempt_cap=0 in conductor.elai-walk.toml is a SILENT second gate (Q1/#10) -- a picker
  that only flips model_tier will look wired but PAID runs will still 0-attempt-refuse; must be
  explicitly designed for (new config variant or a runtime override param on run_backlog/scheduler).
- walk_start/stop has no Tauri #[tauri::command] counterpart (fetch-only) -- if the picker param must
  also work from the native Tauri build, a matching command needs adding; if web-only is acceptable,
  no new Tauri wiring needed (matches existing walk pattern).
- _walkAction takes no body/query today (slice_api.py:1263/1266 dispatch on bare route string, no
  payload parse visible at those lines) -- executor must add request-body parsing on the POST
  /walk/start handler and a matching param on walk_start().
- UNVERIFIED: exact slice_api.py request-body-parsing code path for POST handlers in general (did not
  read the full Handler class); confirm the existing body-JSON-parse helper before adding a new field
  to avoid a bespoke parser (maintainability-review dimension 6, canonical layering).

Return to main LLM: Wrote 10-hop PRODUCTION_HOOK chain + 5 answered questions + 5 gaps for
teikoku-walk-control-canvas (SP-2B). Path: F:\.ELAI_workflow\workflow\reasoning\SP-2B-canvas-exploration.md

TEIKOKU P4-C context (CC-1): the interactive chat-orchestration SEAM

task_id=P4-C-CC1  scope=orchestration seam (base-cc interactive entry + conductor spawn/mailbox/wake/headless)
mode=READ-ONLY analysis. Peer CC-2 owns the model-pool picker + the 5 toggles + grill; this report does NOT cover those.
verified against live WSL distro `teikoku` 2026-06-20; all file:line are real (cat -n / sed / grep over ~/teikoku).

================================================================================
0. ONE-PARAGRAPH ANSWER (the seam, top-down)
================================================================================
The conductor ALREADY ships the complete orchestration substrate P4-C needs; P4-C is a thin
chat front-end over it, not new machinery. A running base-cc session (interactive REPL or
headless --print) reaches the conductor through ONE production seam: the slice_api hub on
127.0.0.1:8787 (slice_api.py), which already serves /spawn (enqueue N sessions),
/mailbox/send + /mailbox/inbox + /mailbox/wait (address a running session, the drop-in path),
/events/wait + /events/notify (event-driven wake), and /orchestration (query all sessions'
state). The "one chat = N sessions" mapping is the spawn lineage already recorded as
SESSION_SPAWNED events keyed by parent_session, with the tasks table as the authoritative
child-liveness source. The DROP-IN override into a running session is mailbox.send_message ->
that session's inbox + a recipient-filtered hub wake; on the base-cc side the hook already
exists (mailbox-client.ts subscribeHubWake, wired into print.ts at line 354/2525, plus the
module-level command queue messageQueueManager.enqueue). What P4-C must ADD is the
ORCHESTRATOR-FRONT routing layer (the chat turn goes to an orchestrator decision, not straight
to a worker) and a single new conductor verb/route that a chat loop drives.

================================================================================
1. BASE-CC INTERACTIVE ENTRYPOINT
================================================================================
ENTRY: ~/teikoku/base-cc/src/entrypoints/cli.tsx :33 `async function main()`. cli.tsx (302 lines)
is a DISPATCHER, not the REPL: it parses argv and fast-paths headless subcommands
(:212 templates new/list/reply; :224 `environment-runner` headless BYOC; :235
`self-hosted-runner`). The interactive vs headless split is by output mode, resolved downstream.

HEADLESS / --print PATH (the path P4-C's autonomous workers actually run): ~/teikoku/base-cc/src/cli/print.ts
(5626 lines). The turn loop is `runHeadlessStreaming()` (print.ts :977), consumed by the
`for await (const message of runHeadlessStreaming(...))` driver at :865. Inside the loop the
per-turn command processing is at print.ts :1931-1953: `while ((command = dequeue(isMainThread)))`
pulls the next user/queued command, batching adjacent prompt commands (:1952 canBatchWith). The
assistant turn + tool calls run via `ask({...})` at print.ts :2148 (`for await (const message of ask(...))`).
So: user message ENTERS via the command queue (dequeue at :1936), the assistant turn happens at
:2148, and the loop returns to the queue between turns.

THE DROP-IN HOOK (already present for headless): print.ts :354 imports subscribeHubWake from
utils/mailbox-client.ts; the team-lead inbox loop (print.ts :2500-2560) calls subscribeHubWake at
:2525 with recipient `team-lead:<teamName>` (:2522) and an onWake callback (:2527) that aborts the
500ms file-poll sleep the instant an addressed hub event lands, then re-reads the file inbox. This
is EXACTLY the "inject a message into a running session mid-flight" mechanism, already wired for the
team-lead role. The generic message-injection primitive is the MODULE-LEVEL command queue:
utils/messageQueueManager.ts :128 `enqueue(command)` (pushes onto a process-global `commandQueue`,
:165 const commandQueue), drained by `dequeue()` (:167) in the print.ts loop. enqueue() is
React-independent (header comment :40-52: "Non-React code (print.ts streaming loop) reads directly
via getCommandQueue()"), so an external drop-in becomes: hub /mailbox/wait wakes -> read /mailbox/inbox
-> messageQueueManager.enqueue({priority:'now', value:<dropped text>}) -> next dequeue() at :1936
picks it up. QueuedCommand carries a priority ('now' > 'next' > 'later', :152-156) so a drop-in can
preempt queued work.

INTERACTIVE REPL PATH: the React/Ink interactive surface reads input through the SAME
messageQueueManager (its useSyncExternalStore subscription, :71 subscribeToCommandQueue), so the
drop-in seam is identical for interactive and headless: enqueue() is the one injection point.

================================================================================
2. TEIKOKU CONDUCTOR ORCHESTRATION SURFACE
================================================================================
THE HUB (the seam a chat front-end calls): ~/teikoku/conductor/slice_api.py (386 lines),
ThreadingHTTPServer on 127.0.0.1:8787 (:27 HOST,PORT). It is the canonical cross-process control
plane. Routes a chat loop uses:
  POST /spawn          (:335 -> spawn_session :208) -> spawn.enqueue_spawn: enqueue N sessions.
  POST /mailbox/send   (:326 -> mailbox_send :164) -> address a message to a running session (DROP-IN).
  GET  /mailbox/inbox  (:295 -> mailbox_inbox :177) -> read+consume a session's inbox.
  GET  /mailbox/wait   (:299 -> mailbox_wait :188) -> long-poll until a session has an unread msg.
  GET  /events/wait    (:288 -> events_wait :140) -> recipient-filtered long-poll (progress stream).
  POST /events/notify  (:316 -> events_notify :153) -> publish leg.
  GET  /orchestration  (:282 -> build_orchestration_snapshot) -> query ALL sessions' live state.
CORS is open (:244 Access-Control-Allow-Origin *), so the dashboard chat pane (P4-E) reaches it too.

THE DRIVING PATH (run_headless + run_wave_loop, P4-B): the loop that actually RUNS queued sessions
is conductor_loop.run_wave_loop / loop_until_dry / run_backlog_waves. The CLI verbs:
  conductor.py :387 run_loop_wave -> run_wave_loop (or temporal_backbone.run_wave_backbone when
    config.backbone=='temporal', :389).
  conductor.py :441 run_dry_loop -> loop_until_dry.
  conductor.py :621 run_headless -> headless_runner.run_headless (the P4-B non-interactive front-end:
    GPU hook -> select objectives -> enqueue batch -> drain). This is the headless SIBLING of the
    P4-C interactive front-end; P4-C is "the same engine, but a chat decides the intake instead of a
    template/feature-map".
A chat front-end tells the orchestrator to spawn/drive by POSTing /spawn (or shelling
`conductor.py spawn`), queries state via /orchestration, and streams progress via /events/wait.

LOWER-LEVEL CONTROL: control_plane.py ControlManager (pause/drain/emergency-stop/cancel_task, used by
conductor.py :470 run_control) is the operator kill-switch surface; a chat "stop everything" toggle
maps to ControlManager.emergency_stop (:164) or cancel_task (:106). harness_api.py is a read-only
pack/manifest reader (no control DB) -- NOT part of the orchestration seam; ignore for P4-C.

================================================================================
3. P4-A SPAWN / MAILBOX / WAKE (the exact functions a chat session calls)
================================================================================
SPAWN N SESSIONS -- spawn.py:
  spawn.enqueue_spawn(desc: SpawnDescriptor, db: ControlDB, config) -> SpawnResult  (spawn.py:164).
  A spawn is NOT a fresh daemon: it UPSERTs a NEW task (status PLANNED, wave='spawn', :231) into the
  existing conductor queue; the existing run_wave_loop leases+runs it under the same per-pool envelope
  (FORK F3 OPTION A, header :4-11). The fork-bomb guard runs FIRST (:186-226): refuses (SPAWN_REFUSED
  event, accepted=False, NEVER raises) when spawn_depth > max_spawn_depth (:187) or the parent already
  has >= max_spawn_fanout LIVE children (:208, live count via live_children :100). Caps:
  conductor_config.py :62 max_spawn_depth=4, :63 max_spawn_fanout=8 (DISTINCT from worker_cap :47, the
  per-pool RATE cap). SpawnDescriptor (spawn.py:58): objective, parent_session (the lineage key),
  project_id, spawn_depth, model_tier (FREE/PAID), role, depends_on, token_budget. Production callers:
  conductor.py :586 run_spawn (CLI verb) + slice_api.py :208 spawn_session (/spawn route) -- both live.

ADDRESS A MESSAGE TO A RUNNING SESSION (the drop-in) -- mailbox.py:
  mailbox.send_message(from_session, to_session, body, headers, project_id) -> event_id  (mailbox.py:74).
  Mechanism CONFIRMED: a mailbox message IS a `session_message` event (mailbox.py:41 MESSAGE_EVENT_TYPE
  = memory_store.EVENT_SESSION_MESSAGE) with the recipient in payload["to"] (:99-103); the read cursor
  is an append-only `session_inbox_consumed` watermark event (FORK F1-A, :42-43, zero schema migration).
  send_message appends the durable row (:107 append_event_memory) THEN best-effort wakes the recipient's
  blocked waiter via wake.publish(recipient=to_session) (:115-121). Delivery is via the durable log, not
  the wake (a down hub degrades latency, never delivery, :90-92).
  read side: mailbox.read_inbox(session, mark_consumed=True) -> list[InboxMessage]  (mailbox.py:159);
  has_unread(session) (:254) delegates to wake.unread_for.

WAKE / LONG-POLL FOR EVENTS -- wake.py:
  HUB side: wake.wait_for_new(since, project, timeout, recipient)  (wake.py:128/84) -- blocks on a
  threading.Condition (:101-115), returns events with event_seq>since matching project AND recipient.
  CLIENT side: wake.wait_for_dep(db, task_id) (:221) blocks until a task's deps are DONE;
  wake.wait_for_message(db, recipient) (:351) blocks until that recipient has an unread session_message
  (the mailbox-wait predicate, unread_for :300). Both are lost-wakeup-safe (record watermark before
  block, re-check after, :19-25). wake.publish(recipient=) (:140) is the notify leg.
  BASE-CC consumer ALREADY EXISTS: base-cc/src/utils/mailbox-client.ts subscribeHubWake (:43) long-polls
  GET /events/wait?recipient=, calls onWake() on an addressed event; wired into print.ts :354/:2525.

DROP-IN OVERRIDE -- the confirmed end-to-end mechanism:
  chat orchestrator POST /mailbox/send {to:<running-session-id>, body:<user drop-in>} (slice_api:326)
  -> mailbox.send_message appends session_message + wake.publish(recipient=<session>)
  -> the running session's subscribeHubWake (already running) fires onWake
  -> base-cc reads its inbox and calls messageQueueManager.enqueue({priority:'now', value:body})
  -> the print.ts streaming loop's next dequeue() (:1936) picks it up mid-flight.
  The base-cc team-lead path (print.ts:2500) PROVES this loop already works for the team-lead recipient;
  P4-C generalizes the recipient from `team-lead:<team>` to the session/worker id and enqueues into the
  command queue instead of (or in addition to) the file inbox.

================================================================================
4. ONE-CHAT = N-SESSIONS DATA MODEL (what exists vs what to add)
================================================================================
EXISTS (the lineage is already recorded, no new table needed for v1):
  - The parent->children link is the SESSION_SPAWNED event: payload {parent_session, child_task_id,
    spawn_depth, objective, model_tier, project_id} (spawn.py:235-247). live_children(db, parent_session)
    (spawn.py:100) enumerates a parent's live children by scanning SESSION_SPAWNED events and joining the
    tasks table for non-terminal status -- so "all sessions this chat owns" = SESSION_SPAWNED events
    WHERE payload.parent_session == <chat id> joined to tasks for current status.
  - Per-child run state lives in the tasks table (migrations/001_initial.sql:13): task_id, status
    (PLANNED/READY/LEASED/RUNNING/.../DONE, :33-37), model_tier, plan_id, wave, priority. Active runs are
    in the leases table (:64) with session_id, worktree_path, worker_pid, heartbeat.
  - events table carries session_id (migration 005_events_project_session.sql; conductor_db.py:282/571)
    so every event is attributable to a session for the progress stream + per-session filtering.
  - build_orchestration_snapshot (orchestration_api.py:161) already returns tasks/waves/frontier/leases/
    workers/gates/concurrency/wake -- the live multi-session board a chat queries; _frontier_rows
    (:101) is the bounded live RUNNING/LEASED/READY list.
WHAT THE EXECUTOR ADDS:
  - A chat<->children QUERY helper: "list the live children of parent_session X with their status" --
    today live_children() returns only a COUNT (spawn.py:100); P4-C needs a list-with-status accessor
    (new pub fn, e.g. spawn.children_of(db, parent_session) -> [{child_task_id, status, objective}]).
    This is the per-chat session roster the dashboard pane (P4-E) and the chat loop both read.
  - OPTIONAL (defer unless needed): a `parent_session` column on tasks or a dedicated `chat_sessions`
    table. NOT required for v1 -- the SESSION_SPAWNED event already carries the lineage and tasks carries
    the status. Adding a column is a migration 007; the event-scan model (FORK F1-A style, zero migration)
    is the lower-risk default and matches the P4-A posture. Recommend v1 = event-scan accessor, no migration.

================================================================================
5. ORCHESTRATOR-FRONT + DROP-IN DESIGN (seam for the executor; MR-1 budgeted)
================================================================================
The requirement: the chat turn goes to an ORCHESTRATOR that decides (spawn / answer / escalate), not
straight to a worker; the user can drop-in to a running session. Concrete seam (NEW thin module + 1 route):

  NEW conductor/chat_orchestrator.py (NEW file, ~150-250 lines, well under MR-1 1000):
    - dataclass ChatTurn {chat_session, text, toggles} and ChatDecision {action: spawn|answer|escalate|
      dropin, payload}.
    - decide(turn, db, config) -> ChatDecision: the orchestrator-front. v1 routing is RULE-BASED on the
      toggle set (orchestration on/off) + simple intent (a directive that names a running child => dropin;
      a new objective => spawn; a question => answer). This is a NEW pub fn => MUST be wired (see §6).
    - apply(decision, db, config): on 'spawn' calls spawn.enqueue_spawn (existing); on 'dropin' calls
      mailbox.send_message(to=<child session>, body=text) (existing); on 'escalate' bumps model_tier to
      PAID on the spawn descriptor; on 'answer' returns text to the chat. KEEP this a thin dispatcher over
      the EXISTING spawn/mailbox primitives -- do NOT reimplement them (maintainability dimension 0/6).

  EXTEND conductor/spawn.py (256 -> ~300 lines, under MR-1): add children_of(db, parent_session) the
    list-with-status accessor (§4). Reuses the live_children scan logic; refactor live_children to call it
    (count = len(children_of)) so there is ONE lineage-scan implementation.

  EXTEND conductor/slice_api.py (386 -> ~430 lines, under MR-1): add ONE route POST /chat/turn ->
    chat_orchestrator.decide+apply, and GET /chat/sessions?parent=<id> -> spawn.children_of (the roster).
    These are the production callers of the new fns. Follow the existing route-handler shape
    (slice_api.py:208 spawn_session is the template).

  EXTEND conductor/conductor.py (939 lines -- WATCH MR-1, already near; prefer the route over a new verb,
    or add a minimal `chat` verb that shells to chat_orchestrator). conductor.py is at 939 lines so a large
    addition risks crossing 1000 (MR-1 BLOCK); keep any verb body tiny (delegate to chat_orchestrator like
    run_headless :621 delegates to headless_runner). If it would cross 1000, the route in slice_api is the
    primary entrypoint and the CLI verb is optional.

  BASE-CC side (drop-in receiver): GENERALIZE the existing team-lead hook. Today print.ts:2522 hardcodes
    recipient `team-lead:<team>`; P4-C makes any worker session subscribe with recipient=<its session id>
    and, on onWake, read /mailbox/inbox and call messageQueueManager.enqueue({priority:'now', value:body})
    (messageQueueManager.ts:128). This is a SURGICAL edit to print.ts (the subscription already exists) +
    possibly a tiny new utils/dropin-receiver.ts. Do NOT rebuild the queue -- enqueue() is the seam.

Two-approach note (per house rule): ALT design = make the chat itself a spawned session that talks to
children purely through the mailbox (no new orchestrator module). REJECTED for v1: it pushes the
decide() routing into prompt-space (ungrounded, no deterministic toggle handling) and loses the
"orchestrator-front decides" requirement's auditability. The thin chat_orchestrator.py keeps the
decision deterministic and testable. Keep the door open: chat_orchestrator can later DELEGATE the
'answer' action to an LLM turn without changing the seam.

================================================================================
6. WIRING + TEST SURFACE
================================================================================
PRODUCTION WIRING (D-030 -- every new pub fn needs a real entrypoint in the same changeset):
  - chat_orchestrator.decide / apply  -> wired by slice_api POST /chat/turn (the real cross-process
    entrypoint a chat loop calls) + optionally conductor.py `chat` verb. NAMED SEAM: slice_api.py do_POST
    (mirror :335 /spawn). PRODUCTION_HOOK for the executor = slice_api.py:do_POST route '/chat/turn'.
  - spawn.children_of  -> wired by slice_api GET /chat/sessions + consumed by orchestration_api (the
    per-chat roster) + the P4-E dashboard pane. PRODUCTION_HOOK = slice_api.py:do_GET route '/chat/sessions'.
  - base-cc dropin-receiver  -> wired into print.ts runHeadlessStreaming (the existing subscribeHubWake
    site, print.ts:2525) + the interactive REPL's queue subscription. PRODUCTION_HOOK = print.ts the
    subscribeHubWake block (:2500-2560) generalized off the team-lead-only recipient.
  CAUTION: spawn.consumed_watermark (mailbox.py:148) is a REGISTERED WIRE-DARK[P4-A] observability
    accessor (backing row workflow/sessions/s-p4a/stubs.md) -- do NOT treat it as a new dark symbol.

EXISTING TESTS that cover the substrate (verified present in ~/teikoku/conductor/tests/):
  - tests/test_p3d_wake.py + test_p3d_wake_loop.py -> wake hub long-poll, lost-wakeup safety, cross-process
    wake (the P3-D event-driven wake leg the mailbox reuses).
  - tests/test_wave_orchestration.py + test_single_worker_loop.py + test_scheduler_integrator.py -> the
    run_wave_loop / loop_until_dry / integrator drive path a spawn flows into.
  - tests/test_p4b_worker_plane.py + test_p4b_generality_proof.py -> the headless front-end + the BLOCKING
    non-ELAI generality gate (the proof a chat-driven run must NOT regress).
  GAP (verified by grep): NO test references enqueue_spawn / send_message / read_inbox / wait_for_message
    by name (grep over tests/ returned nothing). The P4-A spawn+mailbox functions appear to be UNTESTED at
    the unit level in tests/ (they were built + audited via the auditor's own cross-process drive per the
    handoff, not via committed pytest). The executor should treat spawn/mailbox unit tests as NEW work.

NEW TESTS P4-C needs:
  T1 one-chat-N-sessions: a chat spawns N children (enqueue_spawn x N under one parent_session), then
     spawn.children_of returns exactly those N with correct status; the fork-bomb cap refuses the (N+1)th
     past max_spawn_fanout (assert SPAWN_REFUSED, accepted=False).
  T2 drop-in lands in the TARGET session: send_message(to=child_A) -> read_inbox(child_A) returns it AND
     read_inbox(child_B) is empty (addressing isolation); the consumed-watermark advances so a 2nd read
     of child_A is empty.
  T3 orchestrator-front routing: chat_orchestrator.decide returns spawn for a new objective, dropin for a
     directive naming a running child, answer for a question, escalate -> PAID (deterministic, no LLM).
  T4 GENERALITY (R10/the BLOCKING gate): run the whole one-chat-N-sessions flow on the non-ELAI `demo`
     pack with ZERO ELAI paths/gates reachable -- reuse the test_p4b_generality_proof.py fixture so P4-C
     does not regress the generality guard.
  T5 (base-cc, optional) drop-in receiver: an addressed hub event -> messageQueueManager.enqueue with
     priority 'now' -> dequeue returns it before a queued 'next' command.
  R10 integration smoke (auditor-owned, not executor): drive POST /chat/turn end-to-end on a HARD input
     (multi-child spawn + a mid-flight drop-in to one child) against the live :8787 hub, assert the
     session_message + SESSION_SPAWNED rows land and the target child's inbox (not its sibling's) receives
     the drop-in.

================================================================================
SEAM SUMMARY (one line each, the executor's map)
================================================================================
spawn N        : spawn.enqueue_spawn (spawn.py:164) via POST /spawn (slice_api:335) / `conductor spawn` (conductor.py:586)
list children  : ADD spawn.children_of (extend spawn.py live_children:100) via GET /chat/sessions (NEW slice_api route)
drop-in send   : mailbox.send_message (mailbox.py:74) via POST /mailbox/send (slice_api:326)
drop-in recv   : base-cc messageQueueManager.enqueue (messageQueueManager.ts:128) off subscribeHubWake (mailbox-client.ts:43 / print.ts:2525)
wake/stream    : wake.wait_for_new (wake.py:84) via GET /events/wait (slice_api:288)
query board    : build_orchestration_snapshot (orchestration_api.py:161) via GET /orchestration (slice_api:282)
orchestrator   : ADD chat_orchestrator.decide/apply (NEW conductor/chat_orchestrator.py) via POST /chat/turn (NEW slice_api route)
drive/run      : run_wave_loop / loop_until_dry (conductor_loop) via conductor.py:387/441; headless sibling run_headless (conductor.py:621)
kill-switch    : control_plane.ControlManager.emergency_stop/cancel_task (control_plane.py:164/106) via conductor.py:470 run_control
caps           : conductor_config max_spawn_depth=4 / max_spawn_fanout=8 (conductor_config.py:62-63)

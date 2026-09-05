TIME AWARENESS FOR ELAI MODELS — DESIGN REASONING
Date: 2026-06-25
Mode: spec-eval AS-SPECIFIED (treat all planned/dark features as built when reasoning),
  grounded-by-default, anti-sycophancy. Author: main LLM, in-band.
Scope: the user's two-part idea — (A) conversational time awareness via per-message
  timestamps + "last message X ago" + a session day that updates across days; (B) tool/command
  time awareness + decoupling the model from the commands it runs so a session can never get
  stuck on a bad/long tool call ("ELAI handles it, we teach the model what command to use").

NOTE ON A MISSING INPUT
  The message pointed at workflow/IMPL-PLAN-TIMESTAMP-INJECTION.md. That file does not exist
  anywhere in the repo (glob + content grep over workflow/ and docs/ both empty). This reasoning
  is built from the idea as described verbatim in the message plus the code/spec grounding below.
  If an impl-plan was intended as the real deliverable, the closing OPEN DECISIONS asks whether
  to author it.

================================================================================
0. GROUNDING LEDGER (what each claim rests on; strongest first)
================================================================================
CODE (read this session):
  G1 ClockProvider trait — core/src/clock.rs:43 — now()/system_now()/now_ms(); two impls:
     OsClock (real wall-clock, clock.rs:85) and SeededClock (FIXED epoch 1_700_000_000 under
     ELAI_SEED, clock.rs:124/158). Re-exported core/lib.rs:123.
  G2 conversation.rs — orchestrator/src/conversation.rs — assembles ApiRequest{system_prompt,
     messages:Vec<ConversationMessage>}; DEFAULT_AUTO_COMPACTION_INPUT_TOKENS_THRESHOLD=100_000
     (conversation.rs:27). No per-message timestamp is injected today (grep clean).
  G3 Background execution ALREADY PARTLY BUILT — tools/src/lib.rs: run_in_background tool-schema
     field (lib.rs:492/858); real spawn returns background_task_id=child.id() (lib.rs:5834-5850);
     BashCommandOutput carries background_task_id / backgrounded_by_user /
     assistant_auto_backgrounded (lib.rs:2628-2630).
  G4 BackgroundTaskManager — gated_spawn(autonomous_mode) -> BackgroundTaskError::
     InteractiveModeRejected; SSB BackgroundTaskStarted / BackgroundTaskComplete
     (c25_compiled_pipeline_contract.rs).
  G5 WatchdogTimeout — real SSB type with WatchdogTimeoutKind{Safety,NoMaterialProgress,Idle}
     (c10_anti_spiral_contract.rs:166/179/190) + payload elapsed_ms/budget_ms
     (c2_ssb_new_variants_contract.rs:408-409); "elapsed since task start, ms"
     (dashboard commands.rs:8647). I-17 watchdog hook = Wave-3 stub (action_type_lock.rs:692).
  G6 AgentNotificationEmitted (H-36) — completion/needs-input notification SSB + dashboard
     commands_h36.rs; PARTIAL per feature map.
  G7 Determinism spine — phase-index-MASTER 0.9: ClockProvider+EntropySource replace 60 raw
     time/entropy sites, "no raw time/entropy outside core impls", "ELAI_SEED + ClockProvider ->
     byte-identical db dumps". MASTER 412: "No timeout in INTERACTIVE mode (human-paced)."
SPEC / FEATURE-MAP (AS-SPECIFIED):
  G8 J-66 SSB temporal-monitor ND-1: "feasibility pre-lint REFUSES any wall-clock/liveness
     predicate" — wall-clock is explicitly banned as a deterministic-gate input.
  G9 I-35 temporal tool-sequence enforcement — rules over tool-call HISTORY (order, not duration).
  G10 I-1 directive supervisor plane — generator emits Intent->Directive; the process-separated
     Supervisor is the ONLY side-effect executor and holds credentials. This is the decoupling
     architecture the user is describing, already specified.
  G11 J-2 ELAI-as-MCP-server — task_submit/task_status verbs over stdio (PLANNED): the polling
     surface for decoupled tasks.
  G12 Context budget allocator (3E) — 8 fixed slots, 2 cache breakpoints (xxhash prefix),
     OutputReserve last; HD-2: never bust the byte-stable cache prefix, volatile content strictly
     after the last breakpoint.
  G13 Staleness family — F-5 quarantine cascade, H-30 stale_fact_reached_prompt instrument,
     H-32 28d TTL, G-9 TTL purge, G-10 model-drift canary, H-47 eval-firewall date-gate. These
     already make ELAI time-aware about DATA freshness — none of them give the model a sense of
     elapsed conversational/tool time.
  G14 H-50/H-51 — regime-aware context + cross-session sleep-time handoff (auto topic-digest from
     the journal on session resume). The natural carrier for the "day updated across a multi-day
     gap" half of Part A.
EXTERNAL (web, this session):
  G15 arXiv:2510.23853 "Your LLM Agents are Temporally Blind: the misalignment between tool-use
     decisions and human time perception" — directly bridges Part A and Part B: agents reason
     about WHEN to act badly because they lack elapsed-time perception.
  G16 arXiv:2601.13206 "Real-Time Deadlines Reveal Temporal Awareness Failures in LLM Strategic
     Dialogues"; dev.to "Why Your Agent Doesn't Know What Time It Is"; HN time-MCP — the
     documented failure is exactly the user's: a once-set system-prompt timestamp never updates,
     so a 6-hour or next-day continuation still reads as the original second.
  G17 Explicit timestamp augmentation in dialogue messages is the established, measured fix
     (per G15/G16 survey framing). current_datetime / time_difference / timestamp_context are
     the canonical MCP verbs.
  G18 Async background tool/subagent execution returning a task_id immediately + completion
     notification is current SOTA: Hermes async_delegation (Nous, 2026-06-15), opencode #5887,
     pydantic-ai-harness #38, Amazon Bedrock AgentCore long-running runtime. ELAI already has the
     primitives (G3-G6); nobody else couples it to a deterministic replay journal, which is the
     differentiator.
  G19 The Claude Code harness this very session runs in is itself ground truth for Part B:
     run_in_background returns a task id; a <task-notification> fires on completion and re-invokes
     the model; ScheduleWakeup self-paces. ELAI can steal its own harness's proven pattern.

================================================================================
1. DESIGN VERDICT (the headline; merit, not build-status)
================================================================================
The idea is correct in direction and the user's own instinct is right: the per-message timestamp
+ "last message X ago" + a session day that updates across days is the established, measured fix
for the temporally-blind failure mode (G15-G17), and it IS necessary. But on its own it is NOT
sufficient, for three grounded reasons, and the user already half-named the gap ("maybe we need
tool time awareness too"):

  (a) It covers HUMAN time (gaps between user messages) but not AGENT time (how long the model's
      own work/tools have been running within a turn). The temporally-blind paper (G15) shows the
      expensive errors are in tool-use timing, not dialogue gaps. Conversational stamps do nothing
      for "this build has been running 8 minutes, stop waiting."
  (b) "The session updates the day if days pass" only fires when the NEXT user message arrives.
      If the model is doing autonomous/background work across a real time boundary (a long bench,
      a remote sidecar boot, an orchestrate dep-wait > 4h), nothing updates its clock mid-work.
  (c) A timestamp in the prompt is CONTEXT, not a CONTROL. It tells the model the time; it does
      not stop the model from blocking forever on a hung tool. The "can't get stuck" guarantee
      the user wants is a different mechanism (a watchdog + decoupled execution), not an injected
      string.

So the correct design is TWO planes that the single idea conflates:
  PLANE 1 — TIME AS CONTEXT (Part A, plus an agent-time extension): real wall-clock facts placed
    in the prompt so the model can REASON about rhythm and elapsed work. Informs judgment.
  PLANE 2 — TIME AS CONTROL (Part B): a supervisor-owned liveness + decoupling mechanism so a
    tool call can never hang the session. Enforces a guarantee, independent of what the model says.
Keeping these separate is the whole design. Conflating them (e.g. letting the model's injected
timestamp drive a timeout) would both break determinism and make the guarantee model-dependent.

DESIGN WEAKNESS found in the idea as stated (legitimate mark-down, not a build-status note):
  the idea would, naively implemented, put a live wall-clock string near the TOP of the prompt
  (system area) and update it every message. That busts ELAI's byte-stable cache prefix (G12,
  HD-2) on every turn and makes RRA replay non-byte-identical (G1/G7). The fix is mechanical
  (Plane-1 placement rule below) but it must be designed in from the start, not discovered at
  audit. This is the single highest-risk part of Part A.

================================================================================
2. PLANE 1 — TIME AS CONTEXT (Part A done right, plus agent-time)
================================================================================
2.1 THE DETERMINISM-SAFE PLACEMENT RULE (non-negotiable, grounds in G1/G7/G12)
  - The time block is VOLATILE content and MUST live in a dedicated context slot placed STRICTLY
    AFTER the last cache breakpoint (the 3E allocator already structures this; OutputReserve is
    last today — the time slot sits just before it, after breakpoint 2). Never in the system
    prompt or rules slot. Then a changing timestamp every turn costs ZERO cache invalidation,
    because everything before the breakpoint is byte-identical.
  - Sourced from ClockProvider.system_now()/now_ms() via the REAL OsClock, NOT SeededClock. Under
    ELAI_SEED test/replay, the SeededClock continues to drive gate/journal timestamps for
    byte-identity; the human-facing context stamp is explicitly the real clock and is recorded
    INTO the SSB action-stream as part of "what the model saw" (so an RRA still faithfully
    replays the exact bytes the model was given — the stamp becomes data in the journal, not a
    re-computed value). This is the same posture as any other model input.
  - The time block is NEVER read by a gate (ND-1, G8). No rule predicate, no FSM transition, no
    promotion gate may branch on it. It is judgment context only. (A separate, journaled
    elapsed-time FACT may feed Plane 2's supervisor, but that path is not a deterministic gate —
    see 3.3.)

2.2 WHAT THE BLOCK CONTAINS (the minimal effective set, per G17)
  Per user turn, appended to the volatile slot:
    - absolute now: ISO-8601 + human ("2026-06-25 14:32 local").
    - session day, and a DAY-ROLLOVER marker when now's date != the previous turn's date
      ("[day changed: 3 days since your last reply]"). This is the user's "updates the day too",
      made explicit rather than implicit.
    - delta since the user's previous message ("last user message: 4m ago" / "11 days ago").
      Source = now - previous user-message wall-clock, both from system_now(), stored on the
      ConversationMessage (a new created_at field, mirroring SSB created_at via ClockProvider,
      MASTER 432 already does this for SSB rows).
  Deliberately EXCLUDED (HD-1/ID-5 honesty, avoid folklore): no "predicted next message time", no
  fabricated timezone if unknown, no relative phrasing the model can't verify. If the local tz is
  unknown, state UTC and say so.

2.3 SESSION-START + CROSS-DAY (the multi-day-gap half)
  - At session resume H-51 already auto-authors a journal topic-digest (G14). Extend its header
    with "elapsed since last session: N days" computed from the last journaled row's ClockProvider
    timestamp vs system_now(). This is where the "10 days passed" awareness actually lands —
    earlier and more reliably than waiting for the user's next message to update the day.
  - This composes with the staleness family (G13): the same elapsed-since figure that the model
    reads as context is what H-30/F-5/H-32 already use to quarantine stale DATA. The new piece is
    surfacing it to the MODEL, not just to the gates.

2.4 AGENT-TIME (the missing extension that makes Part A sufficient — grounds in G15)
  When the model is in a multi-step turn (tool loop, autonomous run), inject elapsed-since-turn-
  start and elapsed-since-last-tool into the same volatile slot on each model re-entry:
    "turn elapsed: 3m12s; last tool (cargo build) started 2m ago, still running."
  This is the fact the temporally-blind paper shows agents lack. It is what lets the model DECIDE
  "this is taking too long, background it / abandon it / ask the user", instead of silently
  re-issuing a blocking wait. It reads off the same ClockProvider; it is the context-side mirror
  of Plane 2's control.

================================================================================
3. PLANE 2 — TIME AS CONTROL (Part B: decouple + can't-get-stuck)
================================================================================
3.1 THE KEY FINDING: this is mostly WIRING/EXPOSURE, not invention.
  The user's framing ("ELAI handles it, we teach the model what command to use") is exactly the
  I-1 supervisor architecture (G10) plus primitives that already exist scattered and partial:
    - background spawn + task id: G3 (real, in the bash tool today).
    - gated background manager + start/complete journal events: G4.
    - watchdog with kinds {Safety, NoMaterialProgress, Idle} + elapsed_ms/budget_ms: G5 (stub).
    - completion/needs-input notification: G6 (H-36, partial).
    - task_status polling verb: G11 (J-2, planned).
  What is missing is the COHERENT MODEL-FACING LOOP that ties them into one contract the model is
  taught. That loop is the net-new deliverable; the parts are built.

3.2 THE LOOP (the contract we teach the model — mirrors G18/G19 SOTA)
  1. Model issues a command/tool with run_in_background=true (or the supervisor auto-backgrounds
     when its own estimate/observed elapsed crosses a threshold — assistant_auto_backgrounded
     already exists as a field, G3). Returns a task_id IMMEDIATELY (already returns child.id, G3).
  2. Model KEEPS WORKING. It does not block. It can issue more commands, reason, or answer the
     user. The supervisor (I-1) owns the running process; the generator never holds it.
  3. On completion / failure / watchdog-fire, an AgentNotificationEmitted (G6) is delivered as a
     synthetic observation on the model's NEXT re-entry — exactly the <task-notification>
     pattern this session runs under (G19). The model is taught: "you will be told when it
     finishes; do not poll in a tight loop."
  4. The model may task_status(task_id) (G11) to check on demand, and TaskStop(task_id) to
     abandon. Polling is a fallback, notification is primary (cost discipline).
  This is the decoupling the user asked for: the model is structurally separated from the lifetime
  of the commands it runs.

3.3 THE CAN'T-GET-STUCK GUARANTEE (the watchdog, determinism-safe)
  - Every supervisor-run command is wrapped by the watchdog (G5). On fire it emits WatchdogTimeout
    {Idle | NoMaterialProgress | Safety} and returns control to the model with a structured
    observation ("tool X exceeded its budget; killed; here is partial output"). The session can
    NEVER hang on a single tool: the bound is owned by the supervisor, not the model, and not by
    the tool itself.
  - DETERMINISM RULE (grounds in G7/G8, MASTER 412): the watchdog's PRIMARY bound is step/event-
    count and "no material progress" (I-35-style history predicate, order not duration, G9) —
    these are replay-stable. Wall-clock duration is a SECONDARY, supervisor-plane-only liveness
    bound that lives OUTSIDE the seeded replay set: it can kill a hung real process (a hung remote
    sidecar, an infinite loop) but it is recorded, not gate-evaluated, so RRA byte-identity on a
    re-run is preserved (a re-run replays the recorded outcome, it does not re-time the wall
    clock). This is the only honest way to have a wall-clock kill in a byte-identical-replay system.
  - INTERACTIVE vs AUTONOMOUS (MASTER 412): the wall-clock watchdog defaults ON only in AUTONOMOUS
    mode. In human-paced INTERACTIVE mode there is no wall-clock timeout (the human is the pace);
    the step/idle watchdog still applies as a safety net, but a model "waiting" in interactive
    mode is not killed for taking minutes. This matches both ELAI's existing posture and avoids
    punishing a legitimately slow-but-progressing task.

3.4 BAD-TOOL-USE IMMUNITY (the user's explicit goal: "impossible to get stuck on a bad tool")
  Three layers, all already-present-or-adjacent, composed:
    - the I-46 bash interceptor (REFUSE-WITH-REDIRECT on a bad bare command) catches the
      malformed call before it runs.
    - the per-tool timeouts that already exist (mcp_bridge tool_timeout=30s, post_tool_use 5s,
      web_validator/tiered_validation budgets) bound individual hangs.
    - the watchdog (3.3) is the universal backstop for anything those miss.
  Net: a bad/long tool call degrades to "killed + reported back to the model + the model keeps
  going", never "the whole session frozen".

================================================================================
4. IMPROVEMENTS I IDENTIFIED (beyond restating the idea)
================================================================================
IMP-1  Split the one idea into the two planes above. Implementing Part A alone (the user's literal
       proposal) would ship a known-insufficient feature (G15) and would, if placed naively, harm
       caching/determinism. The split is the core improvement.
IMP-2  Source the human-facing stamp from OsClock, journal it as model-input bytes, and forbid it
       as a gate input — the one mechanism that makes wall-clock awareness coexist with ELAI_SEED
       byte-identity and ND-1. Without this rule, the feature silently breaks RRA and cache.
IMP-3  Add AGENT-TIME (elapsed-since-turn / elapsed-since-tool), not just human-time. This is the
       part that actually fixes the documented tool-timing failure and is what makes "the model
       decides to stop waiting" possible.
IMP-4  Land the cross-day awareness at SESSION RESUME via H-51's digest header, not only at the
       next user message — so a 10-day gap is known before the model acts, and reuses the elapsed
       figure the staleness family already computes (G13/G14).
IMP-5  Recognize Part B is 80% wired already (G3-G6, G10-G11) and frame it as a WIRING/EXPOSURE
       plan (one coherent model-facing async-tool loop + a watchdog promotion), not a from-scratch
       build. This is much cheaper than the idea implies and avoids re-inventing primitives.
IMP-6  Make the watchdog's wall-clock bound a supervisor-plane liveness mechanism outside the
       replay set, with the primary bound being step/no-progress — so "can't hang" and "byte-
       identical replay" both hold. This is the non-obvious correctness point.
IMP-7  Default the wall-clock watchdog to AUTONOMOUS-only (MASTER 412). Interactive turns keep the
       step/idle safety net but are not wall-clock-killed, so a slow-but-progressing interactive
       task is not murdered mid-work.
IMP-8  Notification-primary, poll-fallback (G18/G19 cost discipline): teach the model it WILL be
       told on completion; tight task_status loops are discouraged and cost tokens.
IMP-9  Differentiator to bank, not invent: ELAI is the only one of the SOTA async-tool harnesses
       (G18) whose background-task lifecycle lands in a deterministic, replayable, taint-tagged
       journal (BackgroundTaskStarted/Complete + WatchdogTimeout + AgentNotificationEmitted are
       already SSB types). "Decoupled async tools you can replay and audit" is on-wedge; say so in
       positioning (K-1/E-23 class), do not build anything extra for it.
IMP-10 Honest caveat to surface, not hide: a per-turn timestamp is itself volatile context that
       the model could over-anchor on (e.g. treat "11 days" as license to assume facts changed).
       Pair the stamp with the existing staleness instruments (G13) so "time passed" routes to
       "re-verify the data", not to "hallucinate an update". This is the temporal-hallucination
       risk the survey (G15/G16) warns about.

================================================================================
5. VERTICAL-SLICE PLAN SHAPE (if this becomes work; paper-to-plan phase-5 form)
================================================================================
Each unit is producer + in-plan consumer + smoke in the same slice (PRE_GATE), wired to a real
entrypoint, off-agent evidence = an SSB row + a contract test. Suggested as a small new track or
a J/K extension (the user picks the letter; do NOT amend old plan files — house rule
new-plan-files-only).

  T-1  ConversationMessage.created_at (ClockProvider-sourced) + a VOLATILE time-context slot in
       the 3E allocator, placed after the last cache breakpoint; populated each turn with
       absolute now + day-rollover marker + since-last-user delta. CONSUMER: conversation.rs
       build path. SMOKE: a 2-turn fixture across a simulated day boundary asserts the block
       appears AFTER the breakpoint (cache prefix byte-identical) and the delta string is correct.
       Re-uses ClockProvider; no new SSB (it is model-input context). RISK: cache placement —
       the one thing the smoke must prove.
  T-2  Agent-time injection: elapsed-since-turn / elapsed-since-last-tool into the same slot on
       model re-entry within a multi-step turn. CONSUMER: the turn loop. SMOKE: a 3-tool fixture
       asserts the elapsed strings advance monotonically and are real-clock-sourced.
  T-3  Session-resume elapsed header: extend H-51 digest with "N days since last session" from the
       last journaled row vs system_now(). CONSUMER: H-51 resume path. SMOKE: resume a session
       whose last row is backdated; assert the header.
  T-4  Async-tool loop exposure (the net-new of Part B): one documented model-facing contract over
       the existing background_task_id + BackgroundTaskManager + task_status (J-2) +
       AgentNotificationEmitted (H-36). The model: issue-with-background -> keep working -> get
       notified -> optionally task_status / TaskStop. CONSUMER: tool dispatch + supervisor.
       SMOKE: background a sleep, assert the model receives a task_id immediately, a
       BackgroundTaskStarted row lands, and an AgentNotificationEmitted + BackgroundTaskComplete
       row lands on completion without the turn having blocked.
  T-5  Watchdog promotion (I-17 Wave-3 stub -> wired): step/no-progress primary bound +
       autonomous-only wall-clock liveness secondary; emits WatchdogTimeout{Idle|NoMaterialProgress
       |Safety} and returns a structured observation. CONSUMER: the supervisor tool wrapper.
       SMOKE: a deliberately-hung fixture in autonomous mode is killed and reported; the same in
       interactive mode is NOT wall-clock-killed (MASTER 412); WatchdogTimeout row asserted.
  T-6  (doc/positioning, JD-1/E-23 class, no code) the determinism rule (Plane-1 placement,
       OsClock-source, no-gate-input, supervisor-plane liveness) written into the planning
       doctrine + a one-line claims-policy entry: "replayable decoupled async tools" is the
       on-wedge differentiator.
  Sequencing: T-1 then T-2/T-3 (context plane, low-risk) in parallel with T-4/T-5 (control plane).
  T-6 last. T-1's cache-placement smoke is the gating risk; do it first.

================================================================================
6. RISKS / FAILURE MODES (top, pre-implementation)
================================================================================
R-1  CACHE/DETERMINISM REGRESSION (highest): a volatile timestamp placed before a cache breakpoint
     invalidates the prefix every turn (cloud cost) and breaks RRA byte-identity. Mitigation = the
     T-1 placement smoke; the whole feature is gated on it.
R-2  TEMPORAL HALLUCINATION: the model over-anchors on "time passed" and fabricates updates
     (documented in G15/G16). Mitigation = IMP-10, route elapsed-time to the staleness re-verify
     path, never to assume-it-changed.
R-3  WATCHDOG FALSE-KILL of a slow-but-progressing legitimate task. Mitigation = primary bound is
     no-material-progress (G9), not raw duration; wall-clock secondary is autonomous-only (IMP-7).
R-4  NOTIFICATION RACE / lost completion event leaving the model believing a task still runs.
     Mitigation = task_status is the authoritative reconcile; the manager already has
     start/complete rows to replay the truth (G4).
R-5  SCOPE CREEP into a general cron/scheduler. The I-53 ledger already DECLINED cron/monitor/
     background-job tools ("supervisor owns scheduling"). Stay inside per-turn/per-task liveness;
     do not build a scheduler.

================================================================================
7. IMPLEMENTATION-READINESS NOTE (separate from the verdict; index-derived, confirm vs live code)
================================================================================
Per the feature map + this session's grep, the substrate status (verify against live code before
building):
  - ClockProvider / SeededClock: WIRED (G1).
  - run_in_background + background_task_id in the bash tool: WIRED (G3).
  - BackgroundTaskManager + BackgroundTaskStarted/Complete: present, autonomy-gated (G4); wiring
    breadth unverified.
  - WatchdogTimeout type + kinds + payload: present; the I-17 hook is a Wave-3 STUB (G5) — T-5 is
    a promotion, not a build.
  - AgentNotificationEmitted (H-36): PARTIAL (G6).
  - J-2 task_status verb: PLANNED (G11).
  - per-turn timestamp in conversation.rs: ABSENT today (G2) — T-1/T-2 are the genuine net-new.
This is a sequencing fact, not a design judgement: the design above stands on its merits
regardless of how much is wired yet.

================================================================================
8. OPEN DECISIONS FOR THE USER (the forks I should not pick alone)
================================================================================
D-1  Author the impl plan? The referenced workflow/IMPL-PLAN-TIMESTAMP-INJECTION.md does not
     exist. Do you want this reasoning turned into real plan files (and under which letter — a new
     track, or a J/K extension)? Per house rule I would create NEW numbered files, not amend.
D-2  Timestamp granularity / privacy: minute-resolution local time, or second-resolution UTC?
     Local tz requires a known tz (privacy + portability); default UTC is safest. Your call.
D-3  Watchdog default: confirm wall-clock liveness defaults AUTONOMOUS-only with interactive turns
     on the step/idle net only (my recommendation, MASTER 412), vs a soft interactive wall-clock
     too.
D-4  Auto-background threshold: should the supervisor auto-background a foreground command once it
     observes it crossing an elapsed/idle threshold (assistant_auto_backgrounded exists), or only
     background when the model explicitly asks? Auto is more "can't get stuck"; explicit is more
     predictable.

================================================================================
GROUNDING / VERIFICATION
  Built/tested: nothing built this session — this is design reasoning only (no code touched, FSM
  IDLE throughout). All code claims are grep/read-verified this session at the cited paths;
  feature-map claims are AS-SPECIFIED (confirm vs live code before building); external claims are
  WebSearch-grounded (arXiv:2510.23853, 2601.13206; Hermes async_delegation 2026-06-15; the
  in-session Claude Code harness behavior). The one unverifiable input (IMPL-PLAN-TIMESTAMP-
  INJECTION.md) is flagged as missing in the header and D-1.
END.

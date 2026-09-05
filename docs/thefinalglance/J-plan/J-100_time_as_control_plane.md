# J-100: time-as-control plane — the decoupled async-tool loop + the can't-get-stuck watchdog, the PLANE-2 technical half of the time-awareness design, BOUND by K-14's TZ-4

STATUS: PLANNED (WAVE 1, parallel with J-99; the TIME-AS-CONTROL half the K-14 OUT-OF-K block recorded NOT authored; this is mostly WIRING/EXPOSURE — Part B is ~80% already built (G3-G6 + I-1), the net-new is ONE coherent model-facing loop + the I-17 watchdog promotion; net 0 new SSB — reuses BackgroundTaskStarted/Complete + AgentNotificationEmitted + WatchdogTimeout)

- Track J | lever CANT-GET-STUCK (a tool call can never hang the session; the model is structurally decoupled from the lifetime of the commands it runs) | grounding STRONG (tools/lib.rs background spawn + BackgroundTaskManager + WatchdogTimeout SSB + I-1 supervisor, code-grounded 2026-06-25 in the design doc; re-grep at edit W-7) / EXTERNAL (SOTA async-tool harnesses, design-rationale only) | nature BUILD (wiring + exposure + one stub promotion) | effort M | WAVE 1
- This is PLANE 2 of the two-plane time design. PLANE 1 (J-99) tells the model the time; PLANE 2 GUARANTEES the session never hangs on a hung/long tool — a different mechanism (a supervisor-owned liveness + decoupling), independent of what the model says. The user's framing ("ELAI handles it, we teach the model what command to use") is exactly the I-1 supervisor architecture plus primitives that already exist scattered + partial; the net-new deliverable is the COHERENT MODEL-FACING LOOP that ties them into one contract the model is taught. BOUND by K-14's TZ-4 (the wall-clock liveness bound lives OUTSIDE the seeded replay set; CITED, not re-authored).
- Source: workflow/reasoning/time-awareness-design-2026-06-25.md §3 (PLANE 2), §3.1 (mostly-wiring), §3.2 (the loop), §3.3 (the watchdog, determinism-safe), §3.4 (bad-tool-use immunity), T-4/T-5, IMP-5/6/7/8/9, R-3/R-4/R-5. K-14 is the K-native DISCIPLINE+PACKAGE half (TZ-4 + the replayable-async-tools differentiator IMP-9). Doctrine: J-0_INDEX.md + .claude/rules/time-determinism.md (TZ-4).

## Context
Why this matters: a timestamp in the prompt is CONTEXT, not a CONTROL — it tells the model the time, it does not stop the model from blocking forever on a hung tool. The "can't get stuck" guarantee the user wants is a supervisor watchdog + decoupled execution. The KEY FINDING (design §3.1): this is mostly WIRING/EXPOSURE, not invention — the parts are built (G3 background_task_id in the bash tool today; G4 BackgroundTaskManager + BackgroundTaskStarted/Complete journal events; G5 WatchdogTimeout type + kinds + elapsed_ms/budget_ms payload, the I-17 hook a Wave-3 stub; G6 AgentNotificationEmitted partial; G10 I-1 supervisor = the decoupling architecture; G11 J-2 task_status verb planned). What is MISSING is the coherent loop the model is taught, plus the I-17 watchdog promotion. ELAI is ALSO the only one of the SOTA async-tool harnesses (Hermes async_delegation / opencode / pydantic-ai-harness / Bedrock AgentCore) whose background-task lifecycle lands in a deterministic, replayable, taint-tagged journal — the IMP-9 differentiator K-14 banks; build nothing extra for it.

THE LOOP (the contract we teach the model, §3.2 — notification-primary, poll-fallback):
  1. Model issues a command/tool with run_in_background=true (or the supervisor auto-backgrounds when observed elapsed crosses a threshold — assistant_auto_backgrounded already exists as a field). Returns a task_id IMMEDIATELY (already returns child.id, G3).
  2. Model KEEPS WORKING — it does not block; the supervisor (I-1) owns the running process, the generator never holds it.
  3. On completion / failure / watchdog-fire, an AgentNotificationEmitted (G6) is delivered as a synthetic observation on the model's NEXT re-entry. The model is TAUGHT: "you will be told when it finishes; do not poll in a tight loop" (cost discipline, IMP-8).
  4. The model MAY task_status(task_id) (J-2) on demand and TaskStop(task_id) to abandon. Polling is a fallback, notification is primary.

THE CAN'T-GET-STUCK GUARANTEE (the watchdog, determinism-safe — §3.3, BOUND by TZ-4):
- Every supervisor-run command is wrapped by the watchdog (I-17). On fire it emits WatchdogTimeout{Idle|NoMaterialProgress|Safety} and returns a structured observation ("tool X exceeded its budget; killed; here is partial output"). The session can NEVER hang on a single tool — the bound is owned by the supervisor, not the model, not the tool.
- DETERMINISM RULE (TZ-4 / K-14, grounds in G7/G8/MASTER 412): the PRIMARY bound is step/event-count + "no material progress" (an I-35-style history predicate, order not duration — replay-stable). Wall-clock duration is a SECONDARY, supervisor-plane-only liveness bound OUTSIDE the seeded replay set: it can kill a hung REAL process (a hung remote sidecar, an infinite loop) but it is RECORDED, not gate-evaluated, so an RRA re-run replays the recorded outcome and never re-times the wall clock. This is the only honest way to have a wall-clock kill in a byte-identical-replay system.
- INTERACTIVE vs AUTONOMOUS (MASTER 412): the wall-clock watchdog defaults ON only in AUTONOMOUS mode. In human-paced INTERACTIVE mode there is no wall-clock timeout (the human is the pace); the step/idle watchdog still applies as a safety net, but a model "waiting" in interactive mode is not killed for taking minutes.

BAD-TOOL-USE IMMUNITY (the user's explicit goal, §3.4 — three composed layers, all present-or-adjacent): the I-46 bash interceptor (REFUSE-WITH-REDIRECT on a bad bare command) catches the malformed call before it runs; the per-tool timeouts that already exist (mcp_bridge tool_timeout, post_tool_use, web_validator budgets) bound individual hangs; the watchdog is the universal backstop. Net: a bad/long tool degrades to "killed + reported back + the model keeps going", never "the whole session frozen".

DECISIONS taken here (reversible, mark // DECISION; K-14's D-3/D-4 open questions land in this pass):
- WATCHDOG DEFAULT (D-3): wall-clock liveness defaults AUTONOMOUS-only; interactive turns keep the step/idle net only (MASTER 412 — avoids murdering a slow-but-progressing interactive task). // DECISION reversible.
- AUTO-BACKGROUND THRESHOLD (D-4): v1 backgrounds ONLY when the model explicitly asks (predictable); supervisor auto-background on observed-elapsed (assistant_auto_backgrounded) ships OBSERVE-ONLY first behind a flag (GD-4 — emit the would-auto-background decision, measure, before defaulting ON). // DECISION reversible.

## Existing state (what the legs wire / promote / cite; do NOT re-author, GD-1)
- BACKGROUND SPAWN + TASK ID [WIRED] tools/src/lib.rs run_in_background schema field (lib.rs:492/858) + real spawn returns background_task_id=child.id() (lib.rs:5834-5850); BashCommandOutput carries background_task_id / backgrounded_by_user / assistant_auto_backgrounded (lib.rs:2628-2630). (G3)
- BackgroundTaskManager [PARTIAL] gated_spawn(autonomous_mode) -> InteractiveModeRejected; SSB BackgroundTaskStarted / BackgroundTaskComplete (c25_compiled_pipeline_contract.rs). (G4)
- WatchdogTimeout [PARTIAL — type present, I-17 hook a Wave-3 STUB] WatchdogTimeoutKind{Safety,NoMaterialProgress,Idle} (c10_anti_spiral_contract.rs:166/179/190) + elapsed_ms/budget_ms payload (c2_ssb_new_variants_contract.rs:408-409); the I-17 watchdog hook = Wave-3 stub (action_type_lock.rs:692) — T-5 is a PROMOTION, not a build. (G5)
- AgentNotificationEmitted (H-36) [PARTIAL] completion/needs-input notification SSB + dashboard commands_h36.rs. (G6)
- DIRECTIVE SUPERVISOR PLANE (I-1) [PLANNED] generator emits Intent->Directive; process-separated Supervisor is the ONLY side-effect executor + holds credentials — the decoupling architecture; T-4 exposes the loop on it. (G10)
- ELAI-as-MCP-server task_status (J-2) [PLANNED] task_submit/task_status verbs over stdio — the polling surface T-4's fallback leg uses. (G11) (soft-dep: the loop ships notification-primary + TaskStop without J-2; task_status is the follow-on poll verb.)
- I-46 bash interceptor [PLANNED] + the existing per-tool timeouts (mcp_bridge/post_tool_use/web_validator) — the §3.4 immunity layers J-100 composes, never re-implements.
- live SSB baseline ~268/577 (re-grep at edit, W-9); J-100 adds NO new action_type (reuses BackgroundTaskStarted/Complete, AgentNotificationEmitted, WatchdogTimeout).

## Reads (mandatory before coding)
1. workflow/reasoning/time-awareness-design-2026-06-25.md IN FULL (§3 PLANE 2, §3.1-3.4, T-4/T-5, IMP-5/6/7/8/9, R-3/R-4/R-5). The grounding ledger G3/G4/G5/G6/G10/G11/G18/G19.
2. .claude/rules/time-determinism.md (TZ-4 — the wall-clock-liveness-outside-the-replay-set rule; this file OBEYS it).
3. crates/tools/src/lib.rs (background spawn + task_id + assistant_auto_backgrounded, T-4) + the BackgroundTaskManager (gated_spawn, the start/complete journal events).
4. The WatchdogTimeout type + kinds + the I-17 Wave-3 stub (action_type_lock.rs:692) + the AgentNotificationEmitted H-36 path — re-grep all live at authoring (W-7).
5. The I-1 supervisor plan file (the decoupling architecture T-4 exposes the loop on) + J-2 task_status (the poll verb) + I-46 + the per-tool timeout sites (the §3.4 immunity layers).

## Tasks (WAVE 1 — control plane; T-4 the exposure loop, T-5 the watchdog promotion)
1. [T-4] ASYNC-TOOL LOOP EXPOSURE (the net-new): ONE documented model-facing contract over the existing background_task_id + BackgroundTaskManager + task_status (J-2) + AgentNotificationEmitted (H-36). Teach the model: issue-with-background -> KEEP WORKING (do not block) -> get NOTIFIED on completion/failure/watchdog-fire (synthetic observation on next re-entry) -> optionally task_status / TaskStop. CONSUMER: tool dispatch + the I-1 supervisor. notification-primary, poll-fallback (IMP-8). The supervisor owns the running process; the generator never holds it.
2. [T-4 auto-background, OBSERVE-ONLY] gate the supervisor auto-background path (assistant_auto_backgrounded on observed-elapsed) behind a flag, emit the would-auto-background decision, measure (GD-4) — v1 default = explicit-background-only (D-4 DECISION).
3. [T-5] WATCHDOG PROMOTION (I-17 Wave-3 stub -> wired): wrap every supervisor-run command. PRIMARY bound = step/no-material-progress (I-35-style history predicate, order not duration; replay-stable). SECONDARY = autonomous-only wall-clock liveness OUTSIDE the seeded replay set (recorded, not gate-evaluated; TZ-4). On fire emit WatchdogTimeout{Idle|NoMaterialProgress|Safety} + return a structured observation (killed + partial output). CONSUMER: the supervisor tool wrapper. Interactive mode keeps the step/idle net, NOT the wall-clock kill (MASTER 412, D-3 DECISION).
4. [§3.4 immunity] compose (NOT re-implement) the I-46 bash interceptor + the existing per-tool timeouts as the first two layers, the watchdog as the universal backstop — a contract test proves a bad/long tool degrades to killed+reported+keep-going, never a frozen session.
5. workflow/reasoning/J-100-decisions.md (the D-3/D-4 DECISIONs + the T-5 promotion finding + the auto-background observe-only delta).

## WIRING_CONTRACT
produces_symbols:
  - the model-facing async-tool loop contract (issue-with-background -> keep working -> notified -> task_status/TaskStop) on the tool-dispatch + I-1 supervisor seam [T-4]
  - the OBSERVE-ONLY supervisor auto-background path (flag-gated, emits the would-auto-background decision) [T-4]
  - the promoted I-17 watchdog (step/no-progress PRIMARY + autonomous-only wall-clock SECONDARY outside the replay set), emitting WatchdogTimeout + a structured kill observation [T-5]
consumes_symbols:
  - background_task_id + run_in_background + assistant_auto_backgrounded (tools/src/lib.rs:492/858/5834-5850/2628-2630; G3) — wired, not re-built
  - BackgroundTaskManager + BackgroundTaskStarted/Complete (G4) + AgentNotificationEmitted (H-36, G6) — the journal/notification rows reused
  - WatchdogTimeout{Idle|NoMaterialProgress|Safety} + elapsed_ms/budget_ms + the I-17 Wave-3 stub (action_type_lock.rs:692; G5) — PROMOTED
  - the I-1 directive-supervisor plane (the decoupling owner; G10) + J-2 task_status (the poll verb; G11, soft-dep) + I-46 + the per-tool timeouts (the §3.4 immunity layers, composed)
  - .claude/rules/time-determinism.md TZ-4 (K-14, the binding pre-gate — OBEYED)
reachable_from:
  - a live `elai run` model issues a background tool -> a task_id returns IMMEDIATELY, BackgroundTaskStarted lands, the turn does NOT block; on completion an AgentNotificationEmitted + BackgroundTaskComplete land on the next re-entry [PRODUCTION, T-4]
  - a deliberately-hung tool in AUTONOMOUS mode -> the watchdog fires, WatchdogTimeout lands, control returns with a structured kill observation [PRODUCTION, T-5]
  - the same hung tool in INTERACTIVE mode -> NOT wall-clock-killed (the step/idle net still applies) [PRODUCTION, MASTER 412]
  - a #[cfg(test)] / contracts/ caller NEVER satisfies any leg (R7)
stub_ban:
  - NO wall-clock value as the PRIMARY watchdog bound or as a gate/transition input (TZ-3/TZ-4; the primary bound is step/no-material-progress, order not duration)
  - NO wall-clock kill in INTERACTIVE mode (MASTER 412; the human is the pace)
  - NO re-timed wall clock on RRA replay (TZ-4: the kill outcome is RECORDED; a re-run replays the recorded outcome, GD-2)
  - NO tight task_status poll loop taught as primary (IMP-8: notification-primary, poll-fallback)
  - NO general cron/scheduler (R-5; I-53 already DECLINED cron/monitor/background-job tools — "supervisor owns scheduling"; stay inside per-turn/per-task liveness)
  - NO re-implementing the I-46 interceptor or the per-tool timeouts (compose them; GD-1)
frontend_pair:
  background-task lifecycle + watchdog fires surface on the existing H-36 AgentNotification dashboard tile (commands_h36.rs) + the watchdog row on the SSB timeline; decide a dedicated task-list tile at edit (W-4) or W-6-justify.
observable_effect:
  trigger: "background a sleep tool (T-4); a deliberately-hung fixture in autonomous mode AND the same in interactive mode (T-5); a bad bare command (§3.4 immunity)"
  expect_ssb: "T-4: a BackgroundTaskStarted on issue + an AgentNotificationEmitted + BackgroundTaskComplete on completion (reused rows, recomputable GD-2). T-5: a WatchdogTimeout{kind} on fire. net 0 new action_type."
  expect_artifact: "T-4: the model receives a task_id IMMEDIATELY, the turn does NOT block (it issues more work / answers), and the completion notification arrives on the next re-entry. T-5: the autonomous-mode hung fixture is KILLED + a structured partial-output observation returns; the interactive-mode hung fixture is NOT wall-clock-killed (MASTER 412); an RRA re-run replays the RECORDED kill outcome (byte-identical, never re-timed; TZ-4). NEGATIVE (R2): a fixture that wires the wall-clock as the PRIMARY bound or evaluates it in a gate FAILS the determinism contract; a bad bare command is REFUSED-WITH-REDIRECT (I-46) before it runs, the session keeps going."

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/J-plan/J-100_time_as_control_plane.md --run-trigger
python .claude/tools/production_caller_check.py --files .elai_cc/crates/tools/src/lib.rs --json   # the async-tool loop + watchdog wrap have a non-test production caller
rg -n "WatchdogTimeout|NoMaterialProgress" .elai_cc/crates/orchestrator/src/ | rg -v "test|contract"   # the I-17 watchdog now has a production call-site (was a Wave-3 stub)
rg -n "task_status|TaskStop|notification" .elai_cc/crates/tools/src/lib.rs | rg -v "test"   # the model-facing loop seam exists
rg -ni "wall.?clock|elapsed_ms" .elai_cc/crates/orchestrator/src/ | rg -in "gate|transition|promote|seed" ; test $? -ne 0   # wall-clock is NOT a gate/transition/promotion/seed input (TZ-3/TZ-4)
test -f workflow/reasoning/J-100-decisions.md
```

## ACCEPTANCE
- [ ] T-4: a backgrounded tool returns a task_id immediately, the turn does NOT block, and an AgentNotificationEmitted + BackgroundTaskComplete arrive on the next model re-entry (notification-primary, poll-fallback; the model is decoupled from the tool lifetime).
- [ ] T-4 auto-background is OBSERVE-ONLY behind a flag (D-4); v1 default = explicit-background-only.
- [ ] T-5: the I-17 watchdog is PROMOTED from the Wave-3 stub to a production caller; PRIMARY bound = step/no-material-progress; autonomous-only wall-clock SECONDARY (D-3, MASTER 412).
- [ ] TZ-4 holds: the wall-clock kill is recorded-not-gate-evaluated, OUTSIDE the seeded replay set; an RRA re-run replays the recorded outcome (byte-identical, never re-timed).
- [ ] §3.4: a bad/long tool degrades to killed+reported+keep-going (I-46 + per-tool timeouts + watchdog composed), never a frozen session; NO general cron/scheduler (R-5).
- [ ] net 0 new SSB action_types (reuses BackgroundTaskStarted/Complete + AgentNotificationEmitted + WatchdogTimeout); cargo check --workspace GREEN; auditor PASS on -p elai-orchestrator + -p elai-tools (+contracts).

## WIRING DONE-GATE (clause by clause)
- W-1: done = a live backgrounded tool + a live autonomous-mode watchdog kill leave their reused SSB rows (R10 integration smoke: drive the REAL entrypoint on a HARD input — a multi-tool turn with a hung tool), not "the stub is wired". W-2: the auto-background + interactive-wall-clock flags get wire-dark-allowlist rows (R9). W-3: N/A (promotion + exposure, not a port).
- W-5: trigger = a live `elai run` background-a-sleep + a hung-fixture autonomous-vs-interactive run. W-7: tools/lib.rs + BackgroundTaskManager + the I-17 stub + H-36 re-grepped at edit. W-9: ~268/577 re-grep.
- TZ-4 (K-14): the wall-clock liveness is outside the replay set, recorded-not-evaluated; GD-2 the kill outcome is the replay. ND-1/J-66: no wall-clock gate input. GD-1: the §3.4 layers + the I-1 supervisor + J-2 are composed, never re-built. GD-4: auto-background observe-first. R7: test callers excluded. R10: the cross-seam integration smoke (multi-tool turn + hung tool) is mandatory, not a per-leg happy path.

## Tests to add
- crates/contracts/tests/j100_async_tool_loop_contract.rs (~140 LOC): (a) background a sleep -> assert the task_id returns immediately + a BackgroundTaskStarted row + the turn did NOT block + an AgentNotificationEmitted + BackgroundTaskComplete on completion (traced-test: logs_contain inside assert! per tracing-test-assert.md T-1); (b) a hung fixture in AUTONOMOUS mode -> WatchdogTimeout{kind} + a structured kill observation; (c) the SAME hung fixture in INTERACTIVE mode -> NOT wall-clock-killed (R2 negative — proves the autonomous-only default is load-bearing); (d) an RRA re-run of (b) replays the RECORDED kill outcome byte-identically (TZ-4; never re-times the clock); (e) a wall-clock-as-PRIMARY-bound or wall-clock-in-a-gate fixture FAILS the determinism contract (R2); (f) a bad bare command is REFUSED-WITH-REDIRECT (I-46) and the session continues. SimpleSpanProcessor only, BatchSpanProcessor banned (tracing-test-assert.md T-3).

## Files touched
- MODIFY: crates/tools/src/lib.rs (the model-facing async-tool loop contract over the existing background spawn + task_id; the OBSERVE-ONLY auto-background flag gate)
- MODIFY: crates/orchestrator/src/ (the I-17 watchdog promotion at the supervisor tool wrapper — step/no-progress primary + autonomous-only wall-clock secondary; the BackgroundTaskManager + H-36 notification delivery on re-entry; exact sites re-grepped W-7)
- CREATE: workflow/reasoning/J-100-decisions.md (the D-3/D-4 DECISIONs + the T-5 promotion finding + the auto-background observe-only delta)

## Citations / Grounding
- workflow/reasoning/time-awareness-design-2026-06-25.md §3/§3.1-3.4 (PLANE 2, mostly-wiring, the loop, the watchdog, immunity), T-4/T-5, IMP-5/6/7/8/9, R-3/R-4/R-5; the grounding ledger G3/G4/G5/G6/G10/G11/G18/G19.
- ELAI ground truth (code-grounded 2026-06-25 in the design doc; re-verify at edit, W-7): background spawn + task_id tools/lib.rs:492/858/5834-5850/2628-2630 [WIRED]; BackgroundTaskManager + BackgroundTaskStarted/Complete [PARTIAL]; WatchdogTimeout + kinds + the I-17 Wave-3 stub action_type_lock.rs:692 [PARTIAL]; AgentNotificationEmitted H-36 [PARTIAL]; I-1 supervisor [PLANNED]; J-2 task_status [PLANNED]. live ~268/577 (W-9).
- arXiv:2510.23853 "Your LLM Agents are Temporally Blind" (the agent-time-blind failure, design-rationale) + the SOTA async-tool harnesses Hermes async_delegation (Nous 2026-06-15) / opencode #5887 / pydantic-ai-harness #38 / Bedrock AgentCore + the in-session Claude Code harness pattern (run_in_background -> task-notification -> re-invoke) — [EXTERNAL], design-rationale only, NEVER an ELAI number (HD-1/ID-5).
- Hard rules invoked: TZ-4 (K-14, wall-clock-liveness-outside-the-replay-set); ND-1 (J-66 no wall-clock gate input); MASTER 412 (no wall-clock timeout in interactive mode); GD-1 (compose I-1/J-2/I-46/timeouts, never re-build); GD-2 (the recorded kill outcome is the replay); GD-4 (auto-background observe-first); R7/R9/R10; W-7/W-9.
- K-14 banks the IMP-9 differentiator (decoupled async tools whose lifecycle lands in a deterministic replayable taint-tagged journal — on-wedge whitespace, build nothing extra for it) + the TZ-4 rule; J-100 is PLANE-2 of the technical track it recorded OUT-OF-K. J-99 is PLANE-1 (TIME-AS-CONTEXT).

## Open questions / risks
- R-3 WATCHDOG FALSE-KILL of a slow-but-progressing legitimate task. Mitigation = the PRIMARY bound is no-material-progress (order not duration), the wall-clock secondary is autonomous-only (D-3).
- R-4 NOTIFICATION RACE / lost completion event leaving the model believing a task still runs. Mitigation = task_status (J-2) is the authoritative reconcile; the BackgroundTaskManager has start/complete rows to replay the truth.
- R-5 SCOPE CREEP into a general cron/scheduler — I-53 already DECLINED cron/monitor/background-job tools. Stay inside per-turn/per-task liveness; do NOT build a scheduler.
- J-2 task_status is PLANNED — the loop ships notification-primary + TaskStop without it; the on-demand poll verb is the follow-on when J-2 lands. Note the soft-dependency; do not block T-4 on J-2.

## Next
Ships the guarantee the prompt stamp (J-99) cannot give: the model is structurally decoupled from the lifetime of the commands it runs, and the session can NEVER hang on a hung/long tool. Mostly wiring (Part B is ~80% built); the net-new is the coherent model-facing loop + the I-17 watchdog promotion, both determinism-safe by K-14's TZ-4. Pairs with J-99 (the context plane). K-14 banks the on-wedge replayable-async-tools differentiator — no extra build.

End of J-100.

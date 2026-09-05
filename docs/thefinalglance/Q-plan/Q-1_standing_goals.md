# Q-1: standing goals — completion enrolls a persistent scheduled invariant (a once-passing check cannot silently rot)

STATUS: PLANNED (WAVE 1; EXTENSION-as-composition of the O-2 scheduler; NEW primitive — the goal ledger; KEYSTONE of Track ORCHESTRATION-ECONOMICS)

- Track: ORCHESTRATION-ECONOMICS | lever: finishing-is-the-enrollment | source: steal-fable5-orchestration STEAL 1 (loop 05) | grounding: MODERATE (index+source-derived, PLAN-asserted anchors) | nature: NEW (as a composition unit over shipped substrate) | effort: M-L | WAVE 1
- PROOF_SPINE: feeds PS-6 (a wire-dark regression re-surfaces as a VIOLATED standing goal on the reachability predicate — the DURABILITY leg of the wire-dark-rate posture number; a rate measured once is an assumption with a timestamp). Primarily a trust-spine durability primitive.

- Premise (steal-fable5-orchestration STEAL 1): every ELAI done-gate is verified ONCE at DONE. Continuous re-verification exists only domain-scoped (F-5 data-staleness quarantine, G model-drift canary, C docs anti-staleness). There is NO general primitive that converts a done-gate's off-agent VERIFY artifact into a persistent scheduled invariant. "A goal you only verify once is an assumption with a timestamp." Q-1 adds the general primitive: on completion, the completion itself enrolls a STANDING GOAL — a deterministic, timeout-bounded predicate (the unit's OWN frozen-checker exit code, R4-compliant) re-run on the O-2 schedule forever; exit!=0 flips it VIOLATED, a sentinel NAMES suspects (merges since last pass) but FIXES nothing, and the violation re-enters the NORMAL work pipeline through the front door. A flaky predicate is QUARANTINED (F-5 tier), never deleted.

## Steal/build
Two designs were weighed (steal doc §STEAL 1): (a) a goal-ledger subsystem composing O-2 TaskSpecs; (b) a `cadence:` field on smoke-manifest / wiring-contract entries. CHOSEN (a): design (b) couples goal lifecycle to the FROZEN gate fileset (R8 makes those read-only to runs; an enrollment WRITE would violate the freeze), while (a) keeps enrollment as journal DATA outside the frozen set. Shape: goal ledger = SSB rows (`GoalEnrolled` / `GoalVerified` / `GoalViolated`); schedule = an O-2 `TaskSpec{trigger: Cron, pipeline:[Gate], sink: DashboardInbox}` per goal; predicate = the unit's existing off-agent VERIFY artifact re-run (a frozen checker's exit code, quoted from the unit's WIRING_CONTRACT.VERIFY); violation -> an F-5-style quarantine tier + re-open through the I-25 front-door router / intervention-card inbox. RIDE-ALONGS on this ledger (from the same steal doc): the COMPOST proposer (3F self-discovery proposes a MISSING standing goal from the week's failure exhaust, <=3/week budget) and the RATCHET WELD (a burn-down that reaches its floor auto-enrolls that floor as a standing goal) — both are proposers that WRITE this ledger; neither is separable from Q-1.
EXCLUDED: a self-FIXING sentinel (the sentinel names suspects, the fix re-enters normal work — a goal that repairs itself silently is the un-auditable path); a predicate that is anything but a deterministic timeout-bounded command (no LLM-judge predicate — TD-3); enrollment writes into the frozen gate-manifest fileset (R8 violation); deleting a flaky goal (quarantine, never delete).

## Existing ground truth (PLAN-asserted; executor re-greps per W-7)
- O-2 `TaskSpec{trigger,pipeline,sink}` + the task_runner interval dispatch + the journaled `task_spec_fired` SSB row — the SCHEDULE substrate a standing goal rides (docs/thefinalglance/O-plan/O-2). Q-1 does NOT build a second scheduler.
- SSB append-only journal + the GateTaxMeasured/RecurrenceHitRateMeasured measured-number row pattern (schema_version 270/271, commits c2daaf4/d1eb94a) — the ledger-row idiom to mirror.
- F-5 staleness quarantine tier — the quarantine mechanism for a flaky predicate (reuse, do not re-implement).
- I-25 ledger-trained router / the dashboard intervention-card inbox — the front door a violation re-enters through.
- 3F rules-engine PROPOSED->SHADOW->ACTIVE self-discovery — the COMPOST proposer's home; the <=3/week budget mirrors its gate discipline.
- the per-unit WIRING_CONTRACT.VERIFY off-agent artifact (a frozen checker exit code) — the PREDICATE source; a goal's predicate IS an existing done-gate's checker, re-run.

## REVISED scope (W-10 tiered)
- SHIP NOW (Tier-1): the goal-ledger schema + SQLite store + `GoalEnrolled`/`GoalVerified`/`GoalViolated` rows; enrollment at the FSM COMPLETE / auditor-PASS seam; the per-goal O-2 Cron TaskSpec that re-runs the predicate + emits `GoalVerified`|`GoalViolated`; the violation -> intervention-card + F-5 quarantine-on-flaky path.
- DEFER (Tier-2): the COMPOST proposer (3F wiring) + the RATCHET WELD auto-enroll (both are proposers on the shipped ledger — land after the ledger has a real corpus of enrolled goals).

## Integration
- crates/orchestrator: the goal-ledger table + the enrollment call at the COMPLETE/PASS seam + the O-2 TaskSpec factory per goal + the predicate-runner (timeout-bounded shell-out to the frozen checker) + the SSB emits.
- frontend_pair: the dashboard intervention-card surface renders a `GoalViolated` card; a goal-ledger status view is an O-4 canvas job, not a new panel (W-6 satisfied via the existing inbox + a status row; justified).

## MOVES (wargame)
M1 add the goal-ledger table + the `GoalEnrolled` emit at the COMPLETE/auditor-PASS seam; a goal row carries {goal_id, predicate_cmd, timeout_s, source_unit, enrolled_at}.
EXPECTED_OBSERVATION: a unit reaching DONE writes exactly one `GoalEnrolled` row whose predicate_cmd is the unit's own VERIFY checker command; a completion with no off-agent VERIFY artifact enrolls NOTHING (and is logged, not silently skipped).
ON_FAIL: a goal enrolls with an LLM-judge or stdout predicate -> the predicate is not deterministic/off-agent -> refuse enrollment unless predicate_cmd is a frozen-checker command with a numeric exit contract; add the negative test.
M2 per enrolled goal, register an O-2 Cron TaskSpec that re-runs predicate_cmd (timeout-bounded) each tick and emits `GoalVerified{goal_id}` on exit 0, `GoalViolated{goal_id, suspects}` on exit!=0 (suspects = merges since last GoalVerified, NAMED not fixed).
EXPECTED_OBSERVATION: a passing predicate emits `GoalVerified` each tick; an induced regression (revert the unit's wiring) flips the NEXT tick to `GoalViolated` with a non-empty suspects list; the sentinel changes NO files.
ON_FAIL: the sentinel edits code to "self-heal" -> the un-auditable silent-repair path -> strip all write capability from the predicate-runner; it may ONLY run the command and journal the exit code.
M3 route `GoalViolated` to the intervention-card inbox + re-open through the I-25 front door; a predicate that flip-flops across N ticks is QUARANTINED (F-5 tier), emitting `GoalViolated{quarantined:true}` and dropping out of the active tick set until a human clears it.
EXPECTED_OBSERVATION: a violation renders one inbox card and creates a normal work item; a flaky predicate (alternating exit codes) is quarantined after the Nth flip and stops spamming, leaving the SSB trail.
ON_FAIL: a flaky goal is DELETED to stop the spam -> history lost -> quarantine (retain the row, mute the tick), never delete.

## FORK
FORK: if the O-2 SsbEvent/Cron trigger cannot carry a per-goal predicate payload the dispatcher can execute -> ship Tier-1 with the predicate-runner OWNED by Q-1 (a `GoalTick` periodic job on the task_runner interval reading the goal-ledger table directly) and route only the SINK through O-2's inbox; record the seam in the decision doc. Do NOT fork a second scheduler loop.

## RECON_NEEDED
RECON_NEEDED-1: the exact COMPLETE/auditor-PASS seam that fires once per shipped unit (where `GoalEnrolled` hooks) -> settle via `grep -rn "COMPLETE\|MEMORIZING\|auditor.*PASS\|fsm_transition" .elai_cc/crates/orchestrator/src` + codegraph_explore on the FSM terminal transition BEFORE the first edit.
RECON_NEEDED-2: the O-2 TaskSpec factory/registration API (does it accept a Gate-only pipeline with a shell predicate, or must Q-1 own the predicate-runner) -> settle via reading docs/thefinalglance/O-plan/O-2 §Integration + `grep -rn "TaskSpec\|register.*spec\|task_runner" .elai_cc/crates/*/src`.
RECON_NEEDED-3: the F-5 quarantine tier API (how a flaky item is quarantined + cleared) -> settle via `grep -rn "quarantine\|staleness\|F5\|f5" .elai_cc/crates`.

## ABORT
ABORT-1: if enrolling a goal would require WRITING into the frozen gate-manifest fileset (R8 read-only-to-runs) -> STOP + handoff; the ledger is journal DATA, never an edit to a frozen checker. This is design (a) chosen over (b) precisely to avoid this.
ABORT-2: if the only available predicate for a unit is the unit's own stdout / a unit test / an LLM judge -> STOP; a standing goal REQUIRES an off-agent deterministic checker (R4). A unit with no such artifact is not eligible for a standing goal (and that gap is itself a finding).

## WIRING_CONTRACT
produces_symbols:
  - the goal-ledger table + `GoalEnrolled`/`GoalVerified`/`GoalViolated` SSB rows (baseline+3; re-read baseline at edit per W-9)
  - the enrollment call at the COMPLETE/auditor-PASS seam
  - the per-goal predicate-runner (timeout-bounded, WRITE-INCAPABLE) + its O-2 TaskSpec (or the Q-1-owned GoalTick per the FORK)
consumes_symbols:
  - O-2 TaskSpec scheduler + task_runner interval + `task_spec_fired` idiom; SSB journal; F-5 quarantine tier; I-25 router / intervention-card inbox; the per-unit frozen VERIFY checker (as the predicate)
reachable_from:
  - a unit reaching FSM COMPLETE enrolls a goal [PRODUCTION completion seam]; the task_runner tick re-runs it [PRODUCTION scheduler tick]; a violation lands an inbox card [PRODUCTION front door]
  - a #[cfg(test)] caller NEVER satisfies this (R7)
stub_ban:
  - NO self-fixing sentinel (name suspects, never edit)
  - NO LLM-judge or stdout predicate (deterministic frozen-checker exit code only; TD-3)
  - NO enrollment write into the frozen gate-manifest set (R8)
  - NO deleting a flaky goal (F-5 quarantine instead)
  - NO second scheduler loop (reuse O-2 / the task_runner interval)
observable_effect:
  trigger: "ship a unit to FSM COMPLETE with a frozen VERIFY checker, then advance N task_runner ticks after reverting the unit's wiring"
  expect_ssb: "one GoalEnrolled at DONE; GoalVerified per tick while wired; a GoalViolated{suspects non-empty} on the first tick after the revert"
  expect_artifact: "a GoalViolated intervention card + a re-opened work item; NEGATIVE: a violation that silently self-heals (files changed by the sentinel), or a flaky goal that is deleted rather than quarantined, must FAIL"

PRODUCTION_HOOK: producer `GoalEnrolled` emit -> consumer the FSM terminal-transition seam (RECON-1); producer the predicate-runner -> consumer the O-2 task_runner tick (RECON-2); producer `GoalViolated` -> consumer the intervention-card inbox + I-25 front door. Exact file+fn seams named by the integration-explorer at execution pre-flight against live code.

## VERIFY
VERIFY: the auditor runs, off-agent:
`python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/Q-plan/Q-1_standing_goals.md --run-trigger`
`python .claude/tools/production_caller_check.py --files <changed .rs> --json`   # DARK new pub fn = BLOCK
pass = a `GoalEnrolled` row landed off-agent at a real FSM COMPLETE; `GoalVerified` rows landed from real task_runner ticks; a reverted-wiring induced regression produced a `GoalViolated` with a named-suspects list and an inbox card, with NO file mutation by the sentinel.

INTEGRATION_SMOKE: an independent agent ships a small real unit to COMPLETE, confirms the enrolled goal, advances several ticks (GoalVerified), then REVERTS the unit's production caller and advances again — asserting the next tick flips to GoalViolated with suspects, an inbox card lands, and re-running the (now flaky, alternate the revert) predicate across ticks trips the quarantine — a multi-tick run exercising the violation + quarantine paths, not one green tick (R10).

## REDTEAM
REDTEAM: attack that FAILED — "make a standing goal lie by pointing its predicate at the unit's own passing stdout": FAILS because M1 refuses any predicate that is not a frozen-checker command with a numeric exit contract (stub_ban), and R4 requires the artifact be one the generator did not author. Attack that SUCCEEDED and its patch — "spam the inbox by enrolling a nondeterministic predicate that flips every tick": patched by M3's flip-flop quarantine (F-5 tier) that mutes the tick after N flips while retaining the SSB trail, so a flaky goal degrades to a single quarantine card, not per-tick noise.

## Grounding
- steal-fable5-orchestration-2026-07-10.md STEAL 1 (loop 05 "finishing is the enrollment") + the ratchet-weld (loop 09) + compost-proposer (loop 08) ride-alongs; design (a) vs (b) tradeoff recorded there.
- O-2 scheduler substrate (docs/thefinalglance/O-plan/O-2) — the schedule; F-5 quarantine, I-25 router, 3F rules-engine, SSB measured-number rows — all feature-map-confirmed at authoring, executor re-greps (W-7).
- wiring-enforcement R4 (off-agent artifact) + R8 (frozen gate read-only) — the two invariants that pick design (a) and ban the self-fixing sentinel.

## COMPOSES-WITH
O-2 TaskSpec scheduler + task_runner interval, SSB journal, F-5 quarantine, I-25 router / intervention-cards, 3F rules-engine (compost proposer, Tier-2), the per-unit frozen VERIFY checker. Re-implements NONE (GD-1). The RATCHET WELD (Tier-2 auto-enroll) and the COMPOST proposer (Tier-2) both WRITE this ledger and depend on it.

End of Q-1.

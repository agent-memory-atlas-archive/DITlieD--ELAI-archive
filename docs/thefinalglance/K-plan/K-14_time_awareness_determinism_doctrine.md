# K-14: time-awareness determinism doctrine + replayable-async-tools differentiator — make wall-clock awareness coexist with byte-identical replay a binding RULE, and bank the on-wedge differentiator as a tagged claim

STATUS: PLANNED (WAVE KW-3; E-23 + doctrine class, K-8-shaped; a .claude/rules/ doctrine rule + a claims-policy entry + dossier/brief differentiator rows; NO runtime code, NO SSB; reuses the K-1 claims-policy lint)

Objective: the time-awareness design (workflow/reasoning/time-awareness-design-2026-06-25.md) is mostly
CAPABILITY work (per-message timestamps, agent-time, H-51 resume header, async-tool loop, watchdog
promotion = T-1..T-5, a technical J-family track, NOT authored here). But it surfaced two wedge/business-
discipline slivers that ARE K, both near-zero new capability: (1) [DISCIPLINE] a determinism INVARIANT that
makes "the model is wall-clock-aware" coexist with ELAI's byte-identical-replay + cache spine — without it,
the naive build silently breaks RRA replay (G1/G7) and busts the cache prefix (G12/HD-2) every turn (the
single highest-risk part of the whole idea, the design's R-1). Shipping it as a BINDING RULE now means the
future T-1..T-5 (J) work is built correctly from the start, not corrected at audit — exactly the K-8 KD-4
pre-gate pattern. (2) [PACKAGE / PROVE] bank the on-wedge DIFFERENTIATOR (IMP-9): ELAI is the only SOTA
async-tool harness whose background-task lifecycle lands in a deterministic, replayable, taint-tagged
journal — "decoupled async tools you can replay and audit" — as a tagged claims-policy entry + a K-1 dossier
DESIGN row + a K-7 brief line, build nothing extra for it. Hard boundary it must NOT cross: it does NOT
author the timestamp injection / agent-time / H-51 header / async-tool loop / watchdog promotion (T-1..T-5 =
technical, J-bound, recorded below NOT authored here); it does NOT claim a MEASURED number for the
differentiator (it is a DESIGN/architectural claim, tagged [DESIGN], honest about the partial-built status —
G3-G6 wired, the exposure loop T-4/T-5 still to land); it does NOT introduce a wall-clock value into any
deterministic gate (ND-1, G8 — the rule it ships forbids exactly that).

Estimated effort: ~0.5 session (~2 h). A .claude/rules/ doctrine file + a claims-policy entry + two tagged
rows in K-1's dossier/K-7's brief + one fixture for the negative arm. No build, no SSB, no new dependency.
Pre-requisites: K-1 dossier + the K-1 claims-policy lint (the differentiator claim passes it; K-14 reuses
the lint, never forks it — GD-1). The determinism spine (ClockProvider/SeededClock core/src/clock.rs:43/124,
RRA byte-identity — re-grep at authoring, W-7) + the 3E context-budget allocator's cache-breakpoint
structure (the placement rule cites it) + ND-1 (J-66's wall-clock-banned-as-gate-input doctrine, the rule
extends it from gates to context). The time-awareness design doc (the rule's source-of-truth).
Parallel-safe with: K-8, K-9, K-13 (disjoint files). Soft-depends K-1 (lint + dossier), K-7 (brief).
Source: workflow/reasoning/time-awareness-design-2026-06-25.md §2.1 (placement rule), §3.3 (supervisor-plane
liveness), IMP-2/IMP-6/IMP-7 (the invariant), IMP-9 (the differentiator), T-6 (the doctrine deliverable),
R-1 (the risk it pre-empts). arXiv:2510.23853 + 2601.13206 (temporally-blind grounding, [EXTERNAL]).
Doctrine: K-0_INDEX.md (KD-1, KD-2, E-23/K-8 rule precedent); ND-1 (J-66); HD-2 (cache); G7 (MASTER 0.9/412).

## Context
Why this matters: the user's idea (wall-clock awareness in the prompt + a can't-get-stuck async/watchdog
mechanism) is correct and necessary (the temporally-blind failure is measured, arXiv:2510.23853). But a
volatile timestamp placed naively near the top of the prompt invalidates the byte-stable cache prefix every
turn (cloud cost) and breaks RRA byte-identity (the replay re-computes a different clock) — and a wall-clock
value wired into a watchdog/gate makes a deterministic gate non-replayable. ELAI already has the correct
answer to all of this in its spine (volatile-slot-after-the-breakpoint, OsClock-journaled-as-model-input,
wall-clock-liveness-on-the-supervisor-plane-outside-the-replay-set, ND-1 no-wall-clock-in-a-gate) — but it
is not WRITTEN DOWN as a binding constraint, so a future executor of the time feature would re-discover R-1
at audit. K-14 promotes the four-clause invariant into the planning doctrine so the technical work is gated
on it. Separately, the design noted (IMP-9) that the deterministic-replayable-auditable async-tool-lifecycle
is a genuine on-wedge differentiator nobody else ships (G18 SOTA harnesses return a task_id but none lands
it in a replayable taint-tagged journal); K-14 banks that as a tagged claim, not a build. Mechanism: one
.claude/rules/ doctrine file (conditional-routing loads it for time/timestamp/watchdog/async-tool tasks) +
one claims-policy entry the K-1 lint enforces + two honestly-tagged differentiator rows.

THE FOUR-CLAUSE TIME-DETERMINISM INVARIANT (the rule K-14 ships; binds future T-1..T-5, J):
  TZ-1 PLACEMENT: any wall-clock/elapsed context block is VOLATILE and MUST sit in a dedicated 3E slot
       STRICTLY AFTER the last cache breakpoint (never in system/rules/PhaseSpec). A changing timestamp then
       costs ZERO cache invalidation. (design §2.1; HD-2; the R-1 mitigation.)
  TZ-2 SOURCE+JOURNAL: the human-facing stamp is sourced from the REAL OsClock (not SeededClock) AND
       recorded INTO the SSB action-stream as "what the model saw" — so the stamp is replay-faithful DATA
       (the RRA replays the exact bytes), not a re-computed value. Gate/journal timestamps keep using the
       ClockProvider/SeededClock for byte-identity. (design §2.1; G1/G7; IMP-2.)
  TZ-3 NEVER-A-GATE-INPUT: no rule predicate, FSM transition, or promotion gate may branch on the time
       block or any wall-clock value (ND-1, J-66 — wall-clock/liveness predicates are refused by the
       feasibility pre-lint). It is judgment context only. (G8; IMP-2.)
  TZ-4 LIVENESS-OFF-THE-REPLAY-SET: a wall-clock watchdog kill is a SUPERVISOR-PLANE liveness mechanism
       OUTSIDE the seeded replay set — it can kill a hung real process but its outcome is RECORDED not
       gate-evaluated (a re-run replays the recorded outcome, never re-times the wall clock); the PRIMARY
       bound stays step/no-material-progress (I-35-style, order not duration); wall-clock liveness defaults
       AUTONOMOUS-only (MASTER 412, interactive is human-paced). (design §3.3; IMP-6/IMP-7; G7/G9.)

## Existing state (A/B..J coverage — what the rule/claim cite, do NOT re-author, GD-1)
- ClockProvider / OsClock / SeededClock [WIRED] core/src/clock.rs:43/85/124 — the TZ-2 source split.
- 3E CONTEXT BUDGET ALLOCATOR [WIRED] 8 slots + 2 cache breakpoints (xxhash prefix), OutputReserve last —
  the TZ-1 placement target (the time slot sits after breakpoint 2, before OutputReserve).
- RRA [WIRED] (G-2) byte-identical action-stream replay under ELAI_SEED — what TZ-1/TZ-2 protect.
- ND-1 (J-66 SSB temporal-monitor) — the wall-clock-banned-as-gate-input doctrine TZ-3 extends to context.
- WatchdogTimeout SSB {Idle|NoMaterialProgress|Safety} + I-17 hook (Wave-3 stub) — the TZ-4 mechanism the
  future T-5 (J) promotes; K-14 only states the determinism rule it must obey.
- background_task_id + BackgroundTaskManager + BackgroundTaskStarted/Complete + AgentNotificationEmitted
  (H-36) — the async-tool-lifecycle primitives that ALREADY land in the journal = the IMP-9 differentiator's
  factual basis (the parts are built; the model-facing loop T-4 is the future J exposure).
- K-1 dossier + claims-policy lint (the claim ledger + KD-1 enforcement K-14 reuses).
- K-7 brief (the buyer pitch the differentiator row feeds); K-8 (the .claude/rules/ + conditional-routing
  pre-gate precedent K-14 mirrors).
- E-23 positioning-artifact precedent (prose + a rule, near-zero code, no SSB).

## Reads (mandatory before coding)
1. workflow/reasoning/time-awareness-design-2026-06-25.md IN FULL (§2.1, §3.3, IMP-2/6/7/9, T-6, R-1).
2. _grounding/K-GROUNDING-MAP.md (KD-1, KD-2) + the K-1 dossier + the K-1 claims-policy lint (reuse it).
3. J-66's plan file + the feature-map ND-1 line (wall-clock-banned-as-gate-input — TZ-3 extends it).
4. core/src/clock.rs (the OsClock/SeededClock split, TZ-2) + the 3E allocator cache-breakpoint structure
   (TZ-1) — re-grep both live at authoring (W-7).
5. K-8's plan file (the .claude/rules/ + conditional-routing pre-gate shape K-14 mirrors) + CLAUDE.md KD-1.

## Tasks (WAVE KW-3 — the determinism doctrine rule + the differentiator claim)
1. [DISCIPLINE] Author .claude/rules/time-determinism.md: the four-clause invariant TZ-1..TZ-4 above, as a
   HARD RULE binding any task that injects wall-clock/elapsed context or wires a watchdog; add a
   conditional-routing.md trigger ("injecting a timestamp / elapsed-time context block, or wiring a
   watchdog / async-tool liveness bound: read .claude/rules/time-determinism.md"). ~50-70 lines.
2. [PACKAGE / PROVE] Add the differentiator to the claims surface: (a) a K-1 dossier DESIGN row —
   "decoupled async tools whose lifecycle lands in a deterministic, replayable, taint-tagged journal"
   tagged [DESIGN] with the G3-G6 code basis + the honest partial-built status (the model-facing loop is
   J-T4/T5 to land), NO measured number (KD-1); (b) a one-line claims-policy entry so any positioning string
   asserting "replayable/auditable async tools" carries the [DESIGN] tag + the partial-status note; (c) feed
   the K-7 brief one line (the buyer-facing differentiator). Do NOT modify K-1/K-7 plan files — a dossier row
   add + a brief line, recorded here + cross-linked in the feature map (new-plan-files-only on PLAN files).
3. EXTEND the K-1 claims-policy lint recogniser (NOT a fork, GD-1) so an "async tools" / "decoupled" /
   "replayable" differentiator claim without a [DESIGN] tag + partial-status note FAILS. ~10-15 LOC.
4. NEGATIVE arm: a fixtures file with (a) a "fully deterministic async tools" over-claim with no tag and
   (b) a design snippet that places the time block in the system prompt / wires wall-clock into a gate — the
   lint fails (a), the rule (auditor-applied) catches (b).
5. workflow/reasoning/K-14-decisions.md.

## WIRING_CONTRACT
produces_symbols:
  - .claude/rules/time-determinism.md (the TZ-1..TZ-4 invariant, the binding pre-gate for future T-1..T-5)
  - the conditional-routing.md trigger that loads it for time/timestamp/watchdog/async-tool tasks
  - the async-tool-replay differentiator entry in the K-1 claims policy + a K-1 dossier [DESIGN] row + a K-7 brief line
  - the differentiator-claim recogniser extension in .claude/tools/claims_policy_lint.py (K-1's lint; extended)
consumes_symbols:
  - the K-1 dossier + claims-policy lint (EXTEND, do NOT fork; KD-1; GD-1)
  - ND-1 / J-66 (the wall-clock-banned-as-gate-input doctrine TZ-3 extends; cited, not re-implemented)
  - ClockProvider/OsClock/SeededClock + the 3E cache breakpoints (the facts TZ-1/TZ-2 cite; not modified)
  - the time-awareness design doc §2.1/§3.3/IMP-9 (the rule's source; promoted, not re-derived)
reachable_from:
  - conditional-routing loads .claude/rules/time-determinism.md when a time/watchdog/async-tool task runs [PRODUCTION harness gate; JD-1 = the rule fires on a relevant task]
  - the auditor runs claims_policy_lint.py over any positioning copy carrying the differentiator [PRODUCTION audit gate]
  - the future T-1..T-5 (J) executor MUST obey the rule (the pre-gate binds it, like K-8/KD-4 binds efficiency work)
  - a #[cfg(test)]/contracts/ caller NEVER satisfies this (R7)
stub_ban:
  - NO MEASURED number for the differentiator (it is a [DESIGN] architectural claim; KD-1)
  - NO differentiator claim without the honest partial-built status note (G3-G6 wired, the loop is J-future)
  - NO time block in system/rules/PhaseSpec and NO wall-clock value in any gate (the rule forbids both; TZ-1/TZ-3)
  - NO fork of the K-1 lint (extend the recogniser; GD-1) and NO main-LLM self-run of the lint (auditor-applied)
  - NO authoring of the technical time feature (T-1..T-5) here (out-of-K, J-bound)
frontend_pair:
  N/A — a .claude/rules/ doctrine file + a positioning/claims entry, no dashboard surface. Justified (W-6).
observable_effect:
  trigger: "a session editing a timestamp/watchdog injection task loads .claude/rules/time-determinism.md (conditional-routing), and the auditor runs claims_policy_lint.py over the differentiator copy"
  expect_ssb: "none (E-23/doctrine class; JD-1 harness evidence — the rule fires via conditional-routing; the lint is a deterministic check)"
  expect_artifact: "the rule file exists and conditional-routing loads it on a time/watchdog task (a live session shows it firing); the K-1 lint PASSES the [DESIGN]-tagged differentiator and FAILS a seeded untagged over-claim (the NEGATIVE arm that MUST fail)"

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/K-plan/K-14_time_awareness_determinism_doctrine.md
test -f .claude/rules/time-determinism.md
rg -n "TZ-1|TZ-2|TZ-3|TZ-4" .claude/rules/time-determinism.md   # the four clauses present
rg -n "time-determinism.md" .claude/rules/conditional-routing.md   # the trigger is wired
python .claude/tools/claims_policy_lint.py --files docs/thefinalglance/K-plan/_fixtures/async_replay_negative.md ; test $? -ne 0   # the untagged over-claim MUST fail
rg -ci "fully deterministic async|guaranteed replay" docs/strategy/*.md   # MUST be 0 (no over-claim ships)
test -f workflow/reasoning/K-14-decisions.md
```

## ACCEPTANCE
- [ ] the four-clause invariant TZ-1..TZ-4 is written as a binding rule + conditional-routing loads it on time/watchdog/async-tool tasks.
- [ ] KD-1: the async-tool-replay differentiator is tagged [DESIGN] with its honest partial-built status; the K-1 lint passes it.
- [ ] TZ-3/ND-1 holds in the rule: no wall-clock value is permitted as a gate/transition/promotion input.
- [ ] the K-1 lint is EXTENDED not forked (GD-1); the K-1/K-7 plan files are NOT amended (a dossier row + brief line, cross-linked).
- [ ] no runtime code, no SSB, no new dependency (E-23/doctrine class); the technical T-1..T-5 are NOT authored here.

## WIRING DONE-GATE (clause by clause)
- W-1: done = conditional-routing actually loads the rule on a relevant task (JD-1) + the lint negative fixture FAILS, not "the file exists".
- W-2: N/A (no feature flag). W-3: N/A (the design is promoted into a rule, not summary-ported). W-4/W-6: N/A dashboard; justified.
- W-5: trigger = a live time/watchdog task firing the rule + the real lint over real copy. W-7: clock.rs + 3E breakpoints + ND-1 re-grepped at edit.
- GD-1: extend the K-1 lint + cite ND-1/the design doc, never re-implement. GD-2: the differentiator's replayability claim is recomputable (RRA byte-identity).
- KD-1: the lint IS the enforcement (no differentiator claim without a [DESIGN] tag + status). KD-2: the differentiator is a buyer-facing on-wedge claim.

## Tests to add
- a Python test for the lint extension (or crates/contracts/tests/k14_async_replay_claims_contract.rs ~40 LOC):
  (a) a [DESIGN]-tagged differentiator with the partial-status note passes; (b) an untagged "fully
  deterministic async tools" over-claim fails; (c) a "guaranteed replay" superlative fails; (d) an
  empty/missing input fails-loud (R1). The rule's TZ-clauses are auditor-applied (no runnable check until
  T-1..T-5 code exists), so the rule's test is the conditional-routing load (JD-1), not a code assertion.

## Files touched
- CREATE: .claude/rules/time-determinism.md (~60 LOC, the TZ-1..TZ-4 invariant)
- MODIFY: .claude/rules/conditional-routing.md (~2 LOC: load the rule on time/timestamp/watchdog/async-tool tasks)
- MODIFY: docs/strategy/ELAI-PROBLEM-QUANTIFICATION-DOSSIER.md (+1 [DESIGN] differentiator row; K-1's file — a row add)
- MODIFY: .claude/tools/claims_policy_lint.py (+~12 LOC differentiator recogniser; K-1's lint, extended)
- CREATE: docs/thefinalglance/K-plan/_fixtures/async_replay_negative.md (~10 LOC, the negative arm)
- CREATE: workflow/reasoning/K-14-decisions.md (~25 LOC)

## Citations / Grounding
- workflow/reasoning/time-awareness-design-2026-06-25.md §2.1 (placement), §3.3 (liveness), IMP-2/6/7/9, T-6, R-1.
- ELAI ground truth: ClockProvider/OsClock/SeededClock core/src/clock.rs:43/85/124 [WIRED]; 3E allocator 2
  cache breakpoints [WIRED]; RRA byte-identity G-2 [WIRED]; ND-1 wall-clock-banned-as-gate-input (J-66);
  background_task_id/BackgroundTaskManager/AgentNotificationEmitted (G3-G6, partial — the differentiator basis).
- arXiv:2510.23853 "Your LLM Agents are Temporally Blind" + arXiv:2601.13206 (the measured failure the
  feature answers; [EXTERNAL], grounding for WHY, never claimed as an ELAI number).
- MASTER 0.9 (ClockProvider+EntropySource determinism spine) + MASTER 412 (no timeout in interactive mode).
- CLAUDE.md: KD-1, GD-1, ND-1, HD-2, E-23/K-8 rule precedent.

## Open questions / risks
- Scope discipline: K-14 ships the RULE + the CLAIM only. The temptation is to start authoring T-1 (the
  timestamp slot) — resist it; that is the technical J track. K-14 is the pre-gate that makes T-1 correct,
  not T-1 itself.
- The differentiator is partly built (G3-G6 wired, the model-facing loop T-4/T-5 not yet) — the [DESIGN] tag
  + partial-status note is mandatory (KD-1); do NOT let the brief imply a shipped, measured async-replay
  guarantee before the exposure loop lands.
- TECHNICAL COMPANION WORK, OUT-OF-K (recorded so nothing is lost; to be authored in a J-family pass, NOT
  here): T-1 ConversationMessage.created_at + a volatile time-context slot after the last cache breakpoint;
  T-2 agent-time (elapsed-since-turn / elapsed-since-tool) injection; T-3 H-51 resume "N days since last
  session" header; T-4 the model-facing async-tool loop (background spawn -> keep working -> notified ->
  task_status/TaskStop) over the wired G3-G6 + J-2 task_status; T-5 watchdog promotion (I-17 Wave-3 stub ->
  wired, step/no-progress primary + autonomous-only wall-clock liveness). All capability (= J), all BOUND by
  K-14's TZ-1..TZ-4 rule. The open user decisions D-2 (tz granularity), D-3 (watchdog default), D-4 (auto-
  background threshold) belong to that J authoring pass, not to K-14.

## Next
This makes the time-determinism invariant binding BEFORE the technical time feature is built (so R-1 cannot
recur at audit), and banks the replayable-async-tools differentiator honestly. It pairs with K-1 (the lint +
dossier it extends), K-7 (the brief it feeds), and K-8 (the rule/pre-gate sibling). The capability work
(T-1..T-5) is recorded for a separate J authoring pass — it is the feature; K-14 is the discipline + the claim.

End of K-14.

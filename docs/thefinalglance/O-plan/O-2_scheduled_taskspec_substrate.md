# O-2: scheduled TaskSpec substrate — typed, journaled, rules-as-data schedules over the closed stage vocabulary

STATUS: PLANNED (WAVE 1; PROOF_SPINE: none (product-surface class), owner-acked 2026-07-06; BUILD — the scheduler substrate)

- Track: RESEARCH-MACHINE | lever: rules-as-data-scheduling | source finding: L3 | grounding: MODERATE (index-derived, PLAN-asserted) | nature: BUILD-on-shipped | effort: M-L | WAVE 1

- Premise: Codex Automations are time/event schedules whose results land in an inbox and auto-archive when nothing to report (developers.openai.com/codex/app/automations). The ELAI analog is the same shape but TYPED and JOURNALED: a `TaskSpec` stored as data in SQLite exactly like the rules engine, every fire a journaled SSB row carrying its trigger cause, triggers driven by the existing task_runner + SSB temporal-monitor. O-2 is the scheduler O-1's daemon needs and O-4's canvas edits; it invents the typed spec + trigger wiring, nothing else.

## Steal/build
`TaskSpec { trigger: Cron{expr} | SsbEvent{action_type filter} | Manual; pipeline: ordered CLOSED ENUM of stage kinds (Fetch, Classify, Gate, FileToBrain, Digest, SpawnResearchTree, RunPlanUnit, Notify); sink: VaultPage | Digest | DashboardInbox | Toast }`. Specs are rows in SQLite like the rules engine (rules-predicates precedent). Every fire emits a journaled SSB row carrying the trigger cause. K-14 time-determinism holds: the wall-clock trigger is supervisor-plane, OUTSIDE the replay set; the run given recorded inputs stays deterministic. Missed-run detection reuses the SSB temporal-monitor engine. The inbox sink is the existing dashboard intervention-card surface.
EXCLUDED: a free-form/"custom node" stage kind (A3 generic-engine trap — n8n-minus-integrations; new stage kinds ship as code through the normal plan process); putting the wall-clock trigger INSIDE the replay set (K-14 violation); a second scheduler loop (reuse the task_runner interval); a new inbox surface (reuse intervention-cards).

## Existing ground truth (PLAN-asserted; executor re-greps per W-7)
- the rules engine stores predicates as data in SQLite (3F rules-predicates precedent) — TaskSpec follows this exact rows-as-data pattern.
- task_runner drives periodic work off `tokio::interval` (C-9) — the Cron trigger dispatches through it, no new loop.
- SSB is the journal; a temporal-monitor engine over SSB already exists for missed/late detection (K-14 time-awareness doctrine).
- the dashboard intervention-card surface is the shipped inbox analog (used for human-in-the-loop cards).

## REVISED scope (W-10 tiered)
- SHIP NOW (Tier-1): the TaskSpec schema + SQLite store + the Cron and Manual triggers + journaled fire rows + the DashboardInbox sink, with the Fetch/Classify/Gate/Digest stage kinds (the ones O-1 already provides).
- DEFER (Tier-2): the SsbEvent trigger + the SpawnResearchTree / RunPlanUnit stage kinds (they cross into the orchestrator FSM — land with a wiring pass that threads the FSM seam) + missed-run detection hookup.

## Integration
- crates/orchestrator (or the rules-engine crate): the TaskSpec table + loader + the trigger dispatcher on the task_runner interval; the fire-row SSB emit.
- frontend_pair: the dashboard intervention-card surface renders the DashboardInbox sink; a spec list/edit view is O-4's canvas job, not a new panel here.

## MOVES (wargame)
M1 add the TaskSpec table + a strict deserializer (closed `trigger` and stage-kind enums) and a loader that reads specs at boot.
EXPECTED_OBSERVATION: a Cron TaskSpec row loads and is enumerated by the dispatcher; an unknown stage kind or trigger variant is a hard parse-refusal.
ON_FAIL: an unknown stage kind is accepted -> the closed-vocabulary invariant (A3) is broken -> make the enum `deny_unknown`-strict, add the negative parse test.
M2 wire the Cron trigger onto the existing task_runner interval so a due spec fires its pipeline and emits a journaled SSB fire-row carrying the trigger cause.
EXPECTED_OBSERVATION: at the due tick the spec fires exactly once and a `task_spec_fired` SSB row lands with `{spec_id, trigger_cause, stage_path}`.
ON_FAIL: the spec double-fires or never fires -> the interval-to-spec binding is wrong -> gate the fire on a last-fired watermark row (idempotent), add the double-fire assertion.
M3 route the pipeline's terminal Notify/sink to the dashboard intervention-card inbox; auto-archive a fire that reports nothing.
EXPECTED_OBSERVATION: a fire that produced results renders an inbox card; a fire with nothing to report auto-archives and leaves only the SSB row.
ON_FAIL: empty fires spam the inbox -> the auto-archive predicate is missing -> add the "nothing-to-report -> archive" branch keyed off the pipeline's result count.

## FORK
FORK: if the SSB temporal-monitor engine does NOT expose a due/overdue query the dispatcher can poll -> route missed-run detection to Tier-2 (defer) and ship Tier-1 with fire-on-tick only; record the seam in the decision doc. Do not build a second monitor.

## RECON_NEEDED
RECON_NEEDED: the rules-engine table/loader shape TaskSpec should mirror -> settle via `grep -rn "rules\|predicate\|CREATE TABLE" .elai_cc/crates` + codegraph_explore on the rules loader BEFORE the first edit.
RECON_NEEDED: whether the task_runner interval accepts a new registered periodic job or must be extended -> settle via `grep -rn "task_runner\|tokio::interval\|register" .elai_cc/crates/*/src` and read the registration site.
RECON_NEEDED: the intervention-card ingestion API (how a card is created) -> settle via `grep -rn "intervention\|InterventionCard\|inbox" .elai_cc/crates .elai_cc/crates/dashboard-v2/src`.

## ABORT
ABORT: if wiring the Cron trigger would require moving the wall-clock read INSIDE the replay/journal set (K-14 says it must stay supervisor-plane) -> stop and flag; the trigger time is an input recorded once, never a replayed step.

## WIRING_CONTRACT
produces_symbols:
  - the TaskSpec schema (closed trigger + stage-kind + sink enums) + its SQLite table and loader
  - the Cron/Manual trigger dispatcher on the task_runner interval
  - the journaled `task_spec_fired` SSB row + the auto-archive-on-empty branch
consumes_symbols:
  - the rules-engine rows-as-data pattern (3F), task_runner interval (C-9), SSB journal + temporal-monitor (K-14), the dashboard intervention-card surface
  - O-1's Fetch/Classify/Gate/Digest stages (a pipeline is an ordered list of O-1 stage kinds)
reachable_from:
  - the task_runner interval fires a due Cron TaskSpec on a live run  [PRODUCTION scheduler tick]
  - a #[cfg(test)] caller NEVER satisfies this (R7)
stub_ban:
  - NO free-form/custom stage kind (A3)
  - NO wall-clock read inside the replay set (K-14)
  - NO second scheduler loop or second inbox surface
  - NO fire without a journaled trigger-cause SSB row
observable_effect:
  trigger: "register a Cron TaskSpec due now with a Digest->DashboardInbox pipeline and advance the task_runner tick"
  expect_ssb: "one task_spec_fired row {spec_id, trigger_cause=Cron, stage_path} per due tick, exactly once"
  expect_artifact: "an intervention-card inbox entry for a result-bearing fire; NEGATIVE: an empty fire that still creates a card, or a double-fire, must FAIL"

PRODUCTION_HOOK: the task_runner periodic dispatch (C-9 interval registration seam) calls the TaskSpec trigger dispatcher each tick — the executor names the exact registration fn at RECON and wires it same-changeset.

## VERIFY
VERIFY: the auditor runs, off-agent:
`python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/O-plan/O-2_scheduled_taskspec_substrate.md --run-trigger`
`python .claude/tools/production_caller_check.py --files <changed .rs> --json`   # DARK new pub fn = BLOCK
pass = a `task_spec_fired` SSB row landed off-agent from a real task_runner tick, fired exactly once, with an inbox card for the result-bearing case and auto-archive for the empty case.

INTEGRATION_SMOKE: an independent agent registers a Cron spec whose pipeline is a full Fetch->Classify->Gate->Digest chain (O-1 stages) over a HARD source, advances several ticks, and asserts single-fire journaling + the inbox card + auto-archive on a subsequent empty tick — a multi-tick run that fires the missed/idempotency paths, not one tick (R10).

## REDTEAM
REDTEAM: attack that FAILED — "make a run non-deterministic by reading the clock mid-pipeline": FAILS because K-14 puts the trigger time in the supervisor plane as a recorded input; the pipeline given recorded inputs replays identically, and the stub_ban forbids an in-replay wall-clock read. Attack that SUCCEEDED and its patch — "flood the inbox by scheduling a 1-minute Cron that always reports nothing": patched by the auto-archive-on-empty branch (M3) so only result-bearing fires surface, and the SSB row remains the audit trail.

## Grounding
- design digest workflow/reasoning/elai-personal-research-machine-2026-07-06.md L3 + A2 (default-off graveyard / engagement falsifier) + A3.
- Codex Automations comparator (developers.openai.com/codex/app/automations) — time/event schedules, inbox, auto-archive.
- feature-map rules-engine (3F) + task_runner (C-9) + SSB/K-14 temporal-monitor + intervention-card entries.

## COMPOSES-WITH
rules-engine rows-as-data (3F), task_runner interval (C-9), SSB journal + K-14 temporal-monitor, dashboard intervention-cards, O-1 stage kinds. Re-implements NONE (GD-1). O-1 supplies the stages; O-4 renders/edits the specs; O-3 supplies the VaultPage sink.

End of O-2.

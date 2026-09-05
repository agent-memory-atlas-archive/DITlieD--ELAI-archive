# I-7: cascading escalation + offline batch — local-first waterfall with condensed-BeliefState escalation

- Track 13 — EVIDENCE-RANKED SEARCH | lever CHEAPER-SPEED | grounding STRONG | nature BUILD | effort M | WAVE 4 | depends I-11 | binds IC-1

- Premise: the original I-7 design (Batch API speculative racing vs parallel local branches) is
  architecturally impossible for live loops — IC-1 kills it. The Batch API runs 10-45 min with a 24 h SLA;
  cancels do not refund queued work; it cannot participate in any interactive or search-loop path. The
  corrected design is a strict waterfall: wide cheap branches on the local lane first, deterministic checks
  locally, and ONLY on local failure does the orchestrator condense the failure set into I-11's BeliefState
  and escalate SYNCHRONOUSLY to the cloud architect. Offline lanes (Cosmos, research, bench) keep the Batch
  API exactly where H-5 placed it.

## Steal/build
IC-1 is a binding correction and the batch-racing design is DEAD — it must not be resurrected. The shipped
design: (1) local lane receives the task and fans out K branches in isolated workspaces (I-2); (2) the local
deterministic check stack (I-17 tri-layer) runs on each; (3) if ALL local branches fail, the orchestrator
CONDENSES the failure set into an I-11 BeliefState payload (tried/failed-because/constraints, <=~2K tokens)
and escalates SYNCHRONOUSLY to the cloud architect (H-7); (4) the cloud response is the next task-runner
candidate; (5) the Batch API is confined to the OFFLINE escalation path (H-5 lanes) — it is structurally
unreachable from the live search loop. The condensation (step 3) uses I-11 BeliefState so the cloud call
receives a typed summary, not raw history. This also closes the cost gate: a task the local lane solves
never leaves the machine (E-39 cost gate composes). EXCLUDED: any async or speculative batch submission on
a live loop path (IC-1 hard ban); heartbeat-primary (IC-4 demoted to last-resort elsewhere; not applicable
here); re-implementing the Batch API client that H-5 already ships. DECLINED: a parallel local+cloud
submission that "races" — races require cancellation refunds that Anthropic does not offer.

## Existing ground truth
- IS-18: batches token (telemetry_types.rs, task_runner.rs — H-5 surface) confirmed live at file level;
  wiring depth unverified this pass; executor re-traces R7 to confirm the offline-only routing constraint
  is enforced structurally, not by convention.
- H-7 routing seam (escalation_policy.rs, cloud_mode.rs): PLAN-asserted; executor re-greps per W-7.
  The escalation target the waterfall calls on local failure.
- H-5 Batch API offline lanes: PLAN-asserted; executor re-greps per W-7. I-7 COMPOSES this (offline batch
  stays as H-5 defines it); I-7's new work is the LIVE escalation waterfall only.
- I-11 BeliefState: IN-2 confirms it is absent (net-new BUILD); I-7 depends on I-11 (Wave-4 dependency;
  I-11 ships in Wave 2 before I-7 executes).
- E-39 cost gate: PLAN-asserted; executor re-greps per W-7. Composes into the zero-cloud-spend assertion.
- EscalationCascade action_type: net-new per I-0 SSB table (Kind: LocalWin/Condensed/Escalated/BatchRouted).

## REVISED scope (W-10 tiered)
SHIP NOW (Tier-1):
- crates/orchestrator/src/escalation_waterfall.rs — the local-first dispatcher: branch fan-out -> local
  tri-layer -> condense-on-fail -> synchronous cloud escalation; EscalationCascade row per transition
- EscalationCascade action_type registration (ssb.rs + action_type_lock.rs, baseline+1)
- structural guard: the Batch API path is ONLY reachable from the offline-lane scheduler (H-5 dispatch),
  never from escalation_waterfall.rs — enforced by call-graph discipline (the waterfall module has no
  import path to the batch client)
- integration at the routing seam: cloud_mode.rs / escalation_policy.rs to route local-fail tasks through
  the waterfall
- decision doc workflow/reasoning/I-7-decisions.md
DEFER (Tier-2 / gated):
- multi-hop escalation (local -> cloud-small -> cloud-large cascade) — gated behind >=3 measured runs
  showing single-hop insufficient; the two-tier waterfall ships first
- cost cockpit cascade trace tile (frontend_pair surface) — gated behind GD-4 bound

## Integration
New module: crates/orchestrator/src/escalation_waterfall.rs.
Existing seams: cloud_mode.rs / escalation_policy.rs (H-7 routing seam — the integration point; executor
re-greps per W-7); I-11 BeliefState::condense (the condensation call); E-39 cost gate (the zero-cloud-spend
assertion on LocalWin tasks).
frontend_pair: cascade trace in the cost cockpit (H-9 tile family, PLAN-asserted for H-9; cross_lang_diff.py
PASS required when the Svelte surface ships).

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/escalation_waterfall.rs::EscalationWaterfall::run
  - EscalationCascade (ssb.rs + action_type_lock.rs — Kind: LocalWin/Condensed/Escalated/BatchRouted)
consumes_symbols:
  - cloud_mode.rs / escalation_policy.rs routing seam (why: escalation target) — GD-1 H-7
  - I-11 BeliefState::condense (why: condensed failure payload for the cloud call) — GD-1
  - E-39 cost gate (why: zero-cloud-spend guard on LocalWin tasks) — GD-1
  - I-2 BranchWorkspace (why: local branch isolation for the fan-out leg) — GD-1 ID-2
  - I-17 TriLayerStack::evaluate (why: the local deterministic check that determines escalation need) — GD-1
reachable_from:
  - task_runner.rs dispatch site -> EscalationWaterfall::run  [PRODUCTION task-runner]
  - (test callers in crates/contracts/tests/ never satisfy this — R7)
stub_ban:
  - NO Batch API on any interactive or live search-loop path (IC-1 is a binding correction; the batch
    racing design is DEAD; the BatchRouted Kind discriminator is for journal completeness on the offline
    path only, never on the live search loop)
  - NO async or speculative parallel local+cloud submission that depends on cancel-refund semantics
    (Anthropic batch cancels do not refund — carried landmine #5 from I-0)
  - NO re-implementing the H-5 Batch API client; I-7 composes it for the offline path only
  - NO raw history in the cloud escalation payload (the condensed I-11 BeliefState is mandatory; sending
    raw history violates ID-3 journaled-middleware and the <=2K payload target)
  - NO .unwrap()/.expect() on production paths; NO blake2/sha2; NO fabricated measurements
frontend_pair:
  cascade trace in cost cockpit (H-9 tile family) — PLAN-asserted for H-9; W-6 justified for v1 (tile is
  Tier-2 deferred until GD-4 bound measured). cross_lang_diff.py PASS required when surface ships (W-4).
  Justified.
observable_effect:
  trigger: "cargo run --bin elai -- run --task fixtures/search/local_solvable.toml --waterfall
            && cargo run --bin elai -- run --task fixtures/search/local_fail.toml --waterfall"
  expect_ssb: "local_solvable fixture: EscalationCascade(LocalWin) row emitted, zero cloud-API calls in
    the journal (E-39 cost gate); local_fail fixture: EscalationCascade(Condensed) then
    EscalationCascade(Escalated) rows, condensed payload <=2K tokens verified from the journal
    (BeliefState shape, I-11)"
  expect_artifact: "the escalation_waterfall.rs module has no import path to the H-5 batch client
    (rg 'batch_client\\|MessageBatch' crates/orchestrator/src/escalation_waterfall.rs MUST be 0);
    an injected call to the batch client from the live waterfall path FAILS to compile (structural
    impossibility — R2/IC-1)"

## Grounding
- IC-1 (I-GROUNDING-MAP): batch racing KILLED — Anthropic/OpenAI batch lanes run 10-45 min / 24 h SLA /
  cancels do not refund; source Review_ADECO-Adversarial-Critique §1/§4-2 + Research_RateLimits.
- Carried landmine #5 (I-0): "Batch lanes run 10-45 min with a 24 h SLA; cancels do not refund queued
  work. NEVER on an interactive or search-loop path."
- IS-18 (I-GROUNDING-MAP): batches token at file level (telemetry_types.rs, task_runner.rs — H-5 surface).
- H-5 offline batch and H-7 routing PLAN-asserted; executor re-greps per W-7.
- ID-3 (I-0_INDEX): journaled-middleware — every transform lands a receipt; the condensed BeliefState
  payload must be logged.
- SSB: +EscalationCascade (Kind: LocalWin/Condensed/Escalated/BatchRouted) — baseline+1 from LIVE 221/532
  as of 2026-06-10 (ssb.rs:2981 / action_type_lock.rs:601); re-read at edit (W-9).

## COMPOSES-WITH
- H-5 batch (shipped): offline batch lanes — compose, do NOT re-implement the Batch API client or the
  offline scheduler; I-7 only adds the live-loop waterfall.
- H-7 architect/editor routing (PLAN-asserted): the cloud escalation target — compose the escalation seam,
  do not re-implement routing policy.
- I-11 BeliefState (Wave 2 BUILD): the condensed payload format — depends on I-11 shipping first.
- E-39 cost gate (shipped): the zero-cloud-spend assertion on LocalWin tasks — compose, not rebuild.

End of I-7.

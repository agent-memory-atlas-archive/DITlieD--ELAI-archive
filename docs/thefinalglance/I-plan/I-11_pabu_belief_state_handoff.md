# I-11: pabu belief-state handoff

- Track 14 — CACHE-INVARIANT ECONOMICS MIDDLEWARE | lever CHEAPER-ORCH | grounding STRONG | nature BUILD | effort M | WAVE 2 | ID-3 | IC-8
- Premise: every cross-model and cross-tier handoff today ships raw session history, bloating the cloud
  escalation payload and preventing honest accounting of what the condensation actually costs.
  A typed, versioned BeliefState replaces raw history; its honesty leg periodically re-runs from full history
  and ledgers the success delta so compression cost is measured, not assumed.

## Steal/build
PABU/BeliefState is a commoditized pattern (Letta/MemGPT); ELAI's differentiator is the IC-8 correction
package: typed + DKG-entity-linked + journaled (every condensation lands an ID-3 receipt row) + honestly
measured (periodic full-history-vs-BeliefState re-run, success delta to the ledger).
Schema: BeliefState carries tried/failed-because/current-AST-state/constraints + DKG-keyed entities, with a
version field so a schema-version mismatch on read fails CLOSED (typed error, never silent parse).
Handoff sites: local->cloud escalation (I-7 condense-then-escalate path), architect->editor routing (H-7),
session resume (H-51 cross-session digest — shares the digest engine, scope differs; never re-implements it).
ID-3 receipt: every BeliefState condensation lands a BeliefStateEvent(Condensed) SSB row with schema_version
and token_delta; replay can reconstruct exactly what the escalated model received.
Composes H-51 topic-digest (the sleep-time sibling; same digest engine, I-11 is the live-handoff scope).
Composes H-6 recover_stubs for detail re-expansion on the receiving side.
EXCLUDED alternative: raw history passthrough on escalation (the current behaviour — this session removes it
from the escalation path, not from session storage).

## Existing ground truth
- IN-2: NO typed BeliefState module (name-grepped crates/orchestrator/src — absent; I-11 is a genuine BUILD)
- IS-6: context_budget.rs adaptive_reduce:1551 (the reduction machinery; BeliefState is a typed condensate of what adaptive_reduce already computes) — PLAN-asserted; executor re-greps per W-7
- H-7 routing: escalation_policy.rs handoff call sites (PLAN-asserted; executor re-greps per W-7)
- H-51 cross-session digest: the sleep-time sibling (PLAN-asserted; shares digest engine — verify scope split at edit)
- CL-1..CL-4 (H-0 context-lifecycle deltas): bind the local-vs-cloud policy split; BeliefState sits at the boundary (PLAN-asserted; executor reads H-0 delta family before coding)
- ID-3 JOURNALED-MIDDLEWARE (I-0_INDEX.md): every middleware transform lands a receipt row in the SSB; un-journaled transform is banned

## REVISED scope (W-10 tiered)
SHIP NOW (Tier-1):
- crates/orchestrator/src/belief_state.rs: BeliefState struct (versioned, typed, DKG-keyed), encode/decode, schema-version mismatch = typed error
- ID-3 receipt emission: BeliefStateEvent(Condensed/Handoff/HonestyDelta) SSB rows at every condensation site
- honesty job: periodic (configurable interval, default: after every 10 escalations) full-history-vs-BeliefState re-run fixture; delta written as HonestyDelta row, recomputable under ELAI_SEED (GD-2)
- integration into the escalation path (I-7 condense-then-escalate): payload carries prefix + BeliefState, NOT raw history
- BeliefState inspector row in SessionsDrawer (frontend_pair)

DEFER (Tier-2 / gated):
- automated staleness detection for DKG-entity links in the BeliefState (depends on DKG live-graph update cadence; note as a follow-on to this session)
- schema migration tooling beyond fail-closed on mismatch (a future patch after v2 schema lands)

## Integration
New module: crates/orchestrator/src/belief_state.rs (~200 LOC: BeliefState, encode/decode/version-check, HonestyJob)
Seams: escalation_policy.rs handoff sites (H-7 routing substrate), session_dag.rs resume path (H-51 boundary), context_budget.rs adaptive_reduce (IS-6, source of condensation inputs)
frontend_pair: BeliefState inspector row in SessionsDrawer (PLAN-asserted component name; executor re-greps per W-7)

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/belief_state.rs::BeliefState (typed versioned condensate with DKG entity links)
  - crates/orchestrator/src/belief_state.rs::HonestyJob (periodic full-history re-run fixture + delta ledger)
  - +BeliefStateEvent (Kind: Condensed/Handoff/HonestyDelta) in ssb.rs / action_type_lock.rs — ID-3 receipt
consumes_symbols:
  - escalation_policy.rs handoff call sites (H-7 routing — the sites I-11 replaces raw-history passthrough) — GD-1
  - context_budget.rs::adaptive_reduce (IS-6: the reduction engine whose output BeliefState types) — GD-1
  - session_dag.rs resume path (H-51 digest engine boundary — share, never re-implement H-51's logic) — GD-1
  - H-6::recover_stubs (on the receiving side; I-11 calls it for detail re-expansion, never reimplements) — GD-1
reachable_from:
  - escalation_policy.rs::condense_and_escalate() -> belief_state::BeliefState::encode()  [PRODUCTION escalation path]
  - session_dag.rs::resume() -> belief_state::BeliefState::decode()  [PRODUCTION session resume]
  - (test callers in contracts/ or #[cfg(test)] do NOT satisfy R7 production reachability)
stub_ban:
  - NO raw history passthrough on escalation (the key invariant: payload must be prefix + BeliefState only)
  - NO un-journaled BeliefState condensation (every condensation emits the ID-3 receipt row — missing receipt = FAIL per ID-3)
  - NO silent schema-version mismatch handling (fail CLOSED with a typed error)
  - NO re-implementation of H-51 cross-session digest engine (compose the shared engine, scope the usage)
  - NO .unwrap()/.expect() on production BeliefState decode paths
  - NO blake2/sha2 (blake3 only)
frontend_pair:
  BeliefState inspector row in SessionsDrawer — required; the HonestyDelta is an economics signal users must see (W-4/W-6).
observable_effect:
  trigger: "cargo run -p elai-cli -- run --task 'implement error handling for a Rust function' --local-first --emit-ssb"
  expect_ssb: "BeliefStateEvent{kind: Condensed, schema_version: 1, token_delta: N} on local-fail escalation; BeliefStateEvent{kind: Handoff} when payload dispatched; HonestyDelta row emitted by the honesty job after >=10 escalations (or on --force-honesty-check fixture)"
  expect_artifact: "escalation payload size <=~2K tokens (journal-verified, vs >=20K raw history control run); schema-version mismatch on read emits a typed BeliefStateVersionMismatch error and is caught before any model call (fixture with a version-bumped blob); the raw-history broken arm (bypassing BeliefState) produces a payload >10K tokens and FAILS the payload-size gate (R2)"

## Grounding
- IN-2 (BeliefState absent confirmed), IS-6 (context_budget.rs adaptive_reduce), H-7/H-51 routing/digest plans (PLAN-asserted)
- IC-8 (PABU differentiator = typed + DKG + journaled + honestly measured; Letta/MemGPT commodity comparison)
- ID-3 (I-0_INDEX.md: JOURNALED-MIDDLEWARE; every transform lands a receipt; un-journaled transform banned)
- CL-1..CL-4 (H-0_INDEX.md context-lifecycle policy split; BeliefState sits at local/cloud boundary)
- SSB: +BeliefStateEvent (Kind: Condensed/Handoff/HonestyDelta) — baseline+1 from LIVE 221/532 as of 2026-06-10 (ssb.rs:2981 / action_type_lock.rs:601); re-read at edit (W-9)

## COMPOSES-WITH
H-7 architect/editor routing (the escalation handoff sites — never re-implement the routing logic)
H-51 cross-session topic digest (shared digest engine; I-11 is the live-handoff scope; H-51 is the sleep-time scope)
H-6 recover_stubs (re-expansion on the receiving side of a handoff)
DKG entity graph (entity-link keys in BeliefState schema; DKG must be reachable at decode time)
I-7 cascading escalation (the condense-then-escalate path that BeliefState feeds)

End of I-11.

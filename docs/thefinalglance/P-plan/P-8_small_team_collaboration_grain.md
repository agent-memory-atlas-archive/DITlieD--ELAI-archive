# P-8: the small-team collaboration grain — 2-10 humans + their agents in one world; the shared-journal merge is the hard kernel

STATUS: PLANNED (WAVE 4; PROOF_SPINE: none (product-surface / distribution class), owner-acked 2026-07-07; BUILD-on-shipped, composition-heavy, BUT the shared-journal merge semantics are a genuine design SPIKE)

- Track: PERSONAL-INCARNATION | lever: shared-world-two-to-ten-seats | source finding: GAP-8 (the "big project between many people" leg) | grounding: MODERATE (index-derived, PLAN-asserted); the merge kernel is an OPEN design problem | nature: composition-heavy + one hard design kernel | effort: L | WAVE 4 | depends O-3 (vault namespaces) + K-23 (multi-repo memory)

- Premise: the owner's scale runs solo toy -> connected small projects -> "a big multi-person project". M solves ORG COMMERCE (entitlement, policy pack, fleet rollup — deliberately no org SaaS). K-23 solves multi-REPO memory for ONE seat. The missing grain is 2-10 HUMANS + their agents in ONE world. The hard kernel: two seats' SSB streams have NO defined JOIN today — that is the genuine design problem this unit must solve before the rest is composition.

## Steal/build
Shared vault namespaces (O-3's namespace-claims file already anticipates two harnesses sharing one vault — seed the multi-seat case from it); cross-seat file/lock discipline (the harness HAS cross-session locks in tooling; the PRODUCT does not — lift it); per-seat provenance inside one journal so "who/which-agent changed this" is a QUERY not an investigation (a `seat_id` + `agent_id` on every SSB row); and M-3's rollup pointed at a SHARED PROJECT instead of an org. The HARD KERNEL is a defined MERGE SEMANTICS for two seats' SSB streams — a design spike FIRST (the join is the deliverable of Tier-1), composition SECOND.
EXCLUDED: an org SaaS/hosted control plane (M deliberately avoids org SaaS — this is a shared LOCAL-first world, not a service); a merge that silently drops or reorders one seat's journal events (the join must be defined + lossless + deterministic, not last-writer-wins); a per-seat code fork (seats share the substrate; per-seat state is data + provenance — A-5); shipping the composition (shared namespaces, locks, rollup) BEFORE the merge kernel is defined (the kernel is the load-bearing precondition).

## Existing ground truth (PLAN-asserted; executor re-greps per W-7)
- O-3's namespace-claims file already anticipates TWO harnesses sharing one vault — the multi-seat generalization starts here.
- K-23 is multi-repo memory for one seat; I-2 is workspace isolation — the substrate for a multi-seat workspace.
- M-2/M-3 is monotone-tighten + the fleet/org rollup — point M-3 at a shared project.
- the harness has cross-session file locks in tooling (multi-session.md) — the product lifts this discipline.
- Phase 3H branch/merge is the nearest existing merge precedent to consult for the SSB-stream join.

## REVISED scope (W-10 tiered)
- SHIP NOW (Tier-1 = the SPIKE + the minimal join): a DESIGN SPIKE that DEFINES the two-seat SSB-stream merge semantics (lossless, deterministic, causally-ordered — NOT last-writer-wins) + `seat_id`/`agent_id` provenance on every SSB row + a two-seat shared-namespace read/write with cross-seat lock discipline + the journaled `seat_journal_merge` SSB row. The merge definition is the primary deliverable; the code proves it on two seats.
- DEFER (Tier-2): the full 3-10 seat scale + M-3 rollup pointed at a shared project + conflict-resolution UX (frontend) + agent-attribution query surface — all straightforward composition ONCE the two-seat merge kernel is defined + proven.

## Integration
- crates/orchestrator + knowledge: the SSB-stream merge function (the spike's output as code) + `seat_id`/`agent_id` on the SSB row schema + the O-3 shared-namespace two-seat path + cross-seat lock enforcement + the merge SSB emit.
- CLI: `elai world join|merge|whodid <query>` subcommand is the Tier-1 production caller.
- rollup (Tier-2): M-3's rollup pointed at a shared-project journal.

## MOVES (wargame)
M1 (THE SPIKE — do this first) define the two-seat SSB-stream merge semantics: a lossless, deterministic, causally-ordered join (each event keyed by `{seat_id, agent_id, lamport_or_hybrid_clock}`), written up as the decision doc BEFORE code.
EXPECTED_OBSERVATION: two seats' journals with interleaved + concurrent events merge to ONE deterministic ordered stream, identical regardless of merge direction (A merged-into-B == B merged-into-A up to the defined total order); NO event is dropped.
ON_FAIL: the merge is order-dependent or lossy -> last-writer-wins was smuggled in -> adopt a causal clock (lamport/hybrid) + a deterministic tiebreak on `{seat_id, agent_id}`; the spike is not done until the join is proven commutative-up-to-total-order + lossless.
M2 add `seat_id` + `agent_id` to the SSB row schema (bump SCHEMA_VERSION at the LIVE baseline, re-read ssb.rs at edit time per W-9) so "who/which-agent changed this" is a query.
EXPECTED_OBSERVATION: every SSB row carries `{seat_id, agent_id}`; `elai world whodid <namespace>` returns the seat+agent for each change without a manual investigation.
ON_FAIL: a row lacks provenance -> the attribution query can't answer -> add the columns at the emit boundary, backfill-null only for pre-schema rows.
M3 wire a two-seat shared namespace read/write with cross-seat lock discipline + emit `seat_journal_merge` on a merge.
EXPECTED_OBSERVATION: two seats writing the same shared namespace serialize through the cross-seat lock (no lost update); a merge of their journals emits `seat_journal_merge {seats, event_count, order_hash}`.
ON_FAIL: a concurrent write is lost -> the product lock wasn't enforced -> lift the harness cross-session lock to the shared-namespace write path, add the lost-update test.

## FORK
FORK: if the spike (M1) finds the existing SSB row has NO causal clock and adding one is structurally invasive -> ship Tier-1 as a two-seat merge over an APPEND-ONLY journal with a hybrid logical clock stamped at emit (additive, non-invasive) rather than retrofitting a clock onto historical rows, and record the SSB-clock seam in the decision doc. Do NOT ship a last-writer-wins merge as a placeholder — a lossy join is worse than a deferred one.

## RECON_NEEDED
RECON_NEEDED: the SSB row schema + whether it carries any ordering/clock beyond a timestamp -> settle via reading ssb.rs + `grep -rn "ssb\|SSB\|action_type\|SCHEMA_VERSION\|clock\|lamport" .elai_cc/crates` BEFORE the spike commits to a clock design.
RECON_NEEDED: Phase 3H branch/merge's existing merge precedent (does it define an event join reusable here?) -> settle via reading the 3H branch/merge design + `grep -rn "3H\|branch\|merge" .elai_cc/crates docs/thefinalglance`.
RECON_NEEDED: the harness cross-session lock API to lift to the product -> settle via reading .claude/rules/multi-session.md + the cross-session lock tooling + `grep -rn "lock\|cross.session\|session.claim" .elai_cc/crates .claude`.
RECON_NEEDED: O-3's namespace-claims two-harness anticipation (how it models two claimants) -> settle via reading O-3's namespace-claims parser.

## ABORT
ABORT: if the two-seat merge cannot be made lossless + deterministic without an org-hosted control plane / central server -> stop and flag; M's doctrine is NO org SaaS and the P-plan is local-first (T6). A merge that needs a central server is a different product (file it), not this unit. The join must work seat-to-seat, local-first.

## WIRING_CONTRACT
produces_symbols:
  - the two-seat SSB-stream merge function (lossless, deterministic, causally-ordered) + its decision-doc spike
  - `seat_id`/`agent_id` on the SSB row schema (SCHEMA_VERSION bump at LIVE baseline) + the cross-seat lock on shared-namespace writes
  - the `elai world join|merge|whodid` CLI subcommand + the journaled `seat_journal_merge` SSB row
consumes_symbols:
  - O-3 namespace-claims (two-claimant seed), K-23 multi-repo memory, I-2 workspace isolation, M-2/M-3 rollup, harness cross-session locks, Phase 3H branch/merge precedent
reachable_from:
  - the `elai world join|merge|whodid` CLI subcommand on a live run  [PRODUCTION CLI]
  - the shared-namespace write path enforces the cross-seat lock on a live two-seat run  [PRODUCTION write boundary]
  - a #[cfg(test)] caller NEVER satisfies this (R7)
stub_ban:
  - NO org SaaS / central control plane (local-first, seat-to-seat)
  - NO last-writer-wins / lossy / order-dependent merge
  - NO per-seat code fork (per-seat state is data + provenance)
  - NO shared-namespace write without cross-seat lock + no merge without a `seat_journal_merge` row
observable_effect:
  trigger: "two seats write interleaved + concurrent events to a shared namespace, then merge their journals in BOTH directions"
  expect_ssb: "every row carries {seat_id, agent_id}; a seat_journal_merge row {seats, event_count, order_hash} per merge; the order_hash is IDENTICAL regardless of merge direction (deterministic total order)"
  expect_artifact: "`elai world whodid <ns>` attributes each change to a seat+agent; NEGATIVE: a dropped event, a direction-dependent order_hash, or a lost concurrent write, must FAIL"

PRODUCTION_HOOK: cli/main.rs `elai world` subcommand dispatch calls the SSB-stream merge + the attribution query, AND the shared-namespace write path enforces the cross-seat lock + emits `seat_journal_merge` — the executor names the exact write-path fn at RECON and wires both same-changeset.

## VERIFY
VERIFY: the auditor runs, off-agent:
`python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/P-plan/P-8_small_team_collaboration_grain.md --run-trigger`
`python .claude/tools/production_caller_check.py --files <changed .rs> --json`   # DARK new pub fn = BLOCK
pass = two seats' journals merged off-agent to a deterministic lossless stream (identical order_hash in both merge directions), every SSB row carried `{seat_id, agent_id}`, and a concurrent shared-namespace write serialized through the cross-seat lock with no lost update.

INTEGRATION_SMOKE: an independent agent simulates TWO seats (each with its own agent) writing interleaved + genuinely-concurrent events to a shared namespace, merges the journals A->B and B->A, and asserts identical deterministic order + zero dropped events + correct per-seat/per-agent attribution on `elai world whodid` + a serialized concurrent write — a two-seat run firing the concurrency + bidirectional-merge + attribution paths (R10). This is the load-bearing smoke for the hard kernel; a single-seat happy path does NOT satisfy R10 here.

## REDTEAM
REDTEAM: attack that FAILED — "make the merged history depend on WHO merges first so a seat can rewrite shared history": FAILS because M1's merge is deterministic + commutative-up-to-total-order (identical order_hash regardless of direction), so merge order cannot rewrite history. Attack that SUCCEEDED and its patch — "lose a concurrent write by racing two seats on one shared namespace": patched by lifting the harness cross-session lock to the product shared-namespace write path (M3) so concurrent writes serialize, verified by the lost-update leg of the smoke.

## Grounding
- design digest workflow/reasoning/elai-personal-software-incarnation-2026-07-07.md GAP-8 (names the shared-journal merge as "the hard kernel of this gap" — two seats' SSB streams have no defined join today) + A-2 (falsifier: merge determinism, zero-dropped-events).
- feature-map: O-3 namespace-claims, K-23 multi-repo memory, I-2 workspace isolation, M-2/M-3 rollup, harness cross-session locks (multi-session.md), Phase 3H branch/merge.
- NOTE: causal-clock design (lamport/hybrid logical clocks) is well-established CS; verify the specific merge/CRDT approach against current sources at the spike, do not author the join from training memory alone.

## COMPOSES-WITH
O-3 namespace-claims (EXTEND to N seats), K-23, I-2, M-2/M-3 rollup, harness cross-session locks (lift to product), Phase 3H merge precedent. Re-implements NONE (GD-1). P-7's team default targets this unit; the SSB `seat_id`/`agent_id` provenance serves P-1's fleet attribution too.

End of P-8.

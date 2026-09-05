# I-8: aletheia voluntary abort — credited abstention, dead-end hashing, and empirical halting bound

- Track 13 — EVIDENCE-RANKED SEARCH | lever CANT-LIE | grounding STRONG | nature BUILD | effort M | WAVE 1 | binds RD-1

- Premise: a system that cannot declare a task infeasible will spiral on impossible problems until the
  context window fills or the budget runs dry. The abstention-aware scorer (H-39/RD-1) already credits
  abstention; what is absent is the structural mechanism to invoke it — an explicit acknowledge_infeasible
  tool in the schema, a dead-end AST hash cache to prevent isomorphic re-attempts, and the empirical
  15-second halting bound (deterministic checks spinning past budget = forced abort). These three
  mechanisms compose shipped substrate; none require new research.

## Steal/build
three-part build on shipped abstention infrastructure: (1) acknowledge_infeasible / abort_trajectory tool
added to the production tool catalog via the 3-arm pattern (IS-14); calling the tool is credited by the
H-39 AbstentionScored row per RD-1 — I-8 composes, does not re-implement the scoring logic; the
orchestrator rolls back to the last good trunk node via H-49 rollback. (2) dead-end hashing: the
AST/approach fingerprint of an aborted trajectory is compressed into a blake3 hash and stored in a bounded
LRU cache; a future branch whose approach hash matches is pruned BEFORE any model call — zero tokens on
isomorphic re-attempts. (3) empirical halting bound: any deterministic check spinning past 15 seconds
triggers an automatic abort + HaltingBound row; the 15s threshold is the empirical value from I-17's
specification, not a paper import. EXCLUDED: a learned infeasibility classifier (no ground-truth corpus;
the structural tool is the correct first mechanism); a hard timeout on the whole task (context budget
already governs this; I-8 is about per-trajectory dead-end detection, not global timeouts). DECLINED:
resurrecting an aborted trajectory in the same session (the dead-end hash cache makes this a zero-cost
prune; the "maybe try harder" re-attempt is exactly the spiral I-8 prevents — ID-1 doctrine).

## Existing ground truth
- IS-14: production_agent_rpc.rs build_tool_catalog :657 (the 3-arm convert/dispatch pattern) — the
  catalog extension point for the new tool. Confirmed live this grounding session.
- IS-7: calibration_bootstrap.rs warm_start_tau (:12 docstring) + conformal_calibration consumers — the
  calibration surface I-23 unifies; I-8 is upstream of it (the abort event feeds back into the
  abstention-score calibration in I-23).
- D-16/D-17 refusal + SLA substrate: PLAN-asserted; executor re-greps per W-7. These are the shipped
  abstention substrate I-8 composes — the acknowledge_infeasible tool extends D-17's schema.
- H-39 AbstentionScored (PLAN-asserted, Wave-1 plan file): the scorer I-8's tool invocation credits.
  I-8 is in Wave 1; H-39 must ship first or in parallel (both are Wave-1).
- H-49 undo-tree rollback (PLAN-asserted, Wave-2 plan file): the trunk-restore primitive; I-8 calls it
  on voluntary abort. Dependency: I-8 Wave-1 ships the tool + hash cache; the full rollback integration
  is exercised when H-49 is live.
- TrajectoryAborted action_type: net-new per I-0 SSB table (Kind: Voluntary/DeadEndHash/HaltingBound).

## REVISED scope (W-10 tiered)
SHIP NOW (Tier-1):
- acknowledge_infeasible tool entry added to the tool catalog (the 3-arm IS-14 pattern)
- dead-end hash cache: crates/orchestrator/src/dead_end_cache.rs — bounded LRU, keyed blake3(AST shape),
  prune check before every branch expansion
- halting bound enforcer: 15s watchdog in the deterministic check runner (I-17 seam); fires HaltingBound
  row and calls abort_trajectory on breach
- TrajectoryAborted action_type registration (ssb.rs + action_type_lock.rs, baseline+1)
- abort -> H-39 AbstentionScored composition: the tool-call handler emits the AbstentionScored row by
  invoking H-39's scorer with abort_reason=Voluntary/DeadEndHash/HaltingBound
- decision doc workflow/reasoning/I-8-decisions.md
DEFER (Tier-2 / gated):
- cross-session dead-end persistence (the LRU cache is in-session only in v1; a durable cross-session
  store is gated behind >=3 sessions showing repeated isomorphic re-attempts)
- abort/dead-end strip in ReasoningDrawer (frontend_pair surface) — gated behind GD-4 bound

## Integration
New module: crates/orchestrator/src/dead_end_cache.rs.
Existing seams: production_agent_rpc.rs build_tool_catalog (the 3-arm add — IS-14); D-17 tool schema
(acknowledge_infeasible extends it — executor re-greps per W-7); I-17 deterministic check runner (the
15s watchdog hook); H-39 AbstentionScored emission site (the scoring call); H-49 rollback call site
(the trunk-restore on abort — H-49 Wave-2, rollback integration tested when H-49 lands).
frontend_pair: abort/dead-end strip in ReasoningDrawer — wire-dark behind GD-4 bound; W-6 justified for
v1. Justified.

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/dead_end_cache.rs::DeadEndCache::check_and_insert
  - production_agent_rpc.rs acknowledge_infeasible tool entry (tool catalog extension)
  - TrajectoryAborted (ssb.rs + action_type_lock.rs — Kind: Voluntary/DeadEndHash/HaltingBound)
consumes_symbols:
  - production_agent_rpc.rs::build_tool_catalog (why: 3-arm pattern for the new tool — GD-1 IS-14)
  - D-17 tool schema (why: acknowledge_infeasible extends the existing schema — GD-1)
  - H-39 AbstentionScored emission (why: tool invocation credits the abstention score — GD-1 RD-1)
  - H-49 undo-tree::rollback (why: trunk-restore on voluntary abort — GD-1)
  - I-17 check runner halting watchdog hook (why: the 15s enforcement point — GD-1)
reachable_from:
  - production entrypoint: the model calls acknowledge_infeasible via the normal tool-call flow ->
    build_tool_catalog dispatch -> abort handler -> DeadEndCache::check_and_insert + AbstentionScored
    [PRODUCTION tool-call dispatch]
  - halting bound path: I-17 check runner timeout -> abort handler -> HaltingBound row
    [PRODUCTION deterministic-check runner]
  - (test callers in crates/contracts/tests/ never satisfy this — R7)
stub_ban:
  - NO re-scoring the abstention (H-39/RD-1 handles scoring; I-8 invokes H-39, never duplicates the
    scorer or the credit-assignment logic)
  - NO resurrecting an aborted trajectory or overriding a DeadEndHash prune within the same session
    (the prune is final; attempting to re-run an isomorphic approach is the spiral I-8 prevents — ID-1)
  - NO hard-coding the 15s halting bound as a constant without a named configuration key (the bound is
    empirical; it must be configurable and the default sourced from I-17's specification)
  - NO blake2/sha2 for the dead-end hash (blake3 mandatory per CLAUDE.md Anti-Stale-Training)
  - NO .unwrap()/.expect() on production paths; NO fabricated measurements
frontend_pair:
  abort/dead-end strip in ReasoningDrawer — wire-dark behind GD-4 bound; W-6 justified for v1. cross_lang_diff.py
  PASS required when surface ships (W-4). Justified.
observable_effect:
  trigger: "cargo run --bin elai -- run --task fixtures/abort/impossible_task.toml
            && cargo run --bin elai -- run --task fixtures/abort/spin_task.toml --halting-bound 15s"
  expect_ssb: "impossible_task: TrajectoryAborted(Voluntary) row emitted + rollback to trunk
    journal-verified (H-49 call logged) + AbstentionScored row carrying abort_reason=Voluntary;
    spin_task: TrajectoryAborted(HaltingBound) row within 15s budget; a second run of impossible_task
    with the isomorphic approach: TrajectoryAborted(DeadEndHash) with ZERO model calls in the window
    (token count from the journal = 0)"
  expect_artifact: "journal shows the trunk node ID before and after the rollback match expected
    (verified via journal-derived SSB seq numbers); a broken arm that runs isomorphic re-attempts
    WITHOUT the dead-end cache spirals past 3 iterations and FAILS the spiral-detector check (R2)"

## Grounding
- D-16/D-17 refusal + SLA: PLAN-asserted as the abstention substrate; executor re-greps per W-7.
- H-39 AbstentionScored: PLAN-asserted (Wave-1 plan file); executor re-greps per W-7.
- H-49 undo-tree rollback: PLAN-asserted (Wave-2); executor re-greps per W-7.
- IS-14 (I-GROUNDING-MAP): build_tool_catalog :657 (3-arm pattern) — the tool extension seam.
- IS-7 (I-GROUNDING-MAP): conformal_calibration consumers — I-8's abort events feed I-23's unification.
- RD-1 (H-0_INDEX): abstention-aware promotion; I-8 is the structural mechanism that makes RD-1 fire on
  genuine infeasibility, not just calibration uncertainty.
- ID-1 (I-0_INDEX): stateless-generator / trajectory-pruning — aborted paths are hard-excluded from
  future context; the dead-end hash cache enforces the pruning mechanically.
- SSB: +TrajectoryAborted (Kind: Voluntary/DeadEndHash/HaltingBound) — baseline+1 from LIVE 221/532
  as of 2026-06-10 (ssb.rs:2981 / action_type_lock.rs:601); re-read at edit (W-9).

## COMPOSES-WITH
- D-16/D-17 refusal + SLA (shipped): the abstention substrate — extend the schema with the new tool;
  do NOT re-implement refusal scoring or SLA tracking.
- H-39 AbstentionScored (Wave-1 plan): the crediting scorer — invoke it, do not duplicate it.
- H-49 undo-tree rollback (Wave-2 plan): the trunk-restore primitive — compose the call, not the logic.
- E-35 tool schema (shipped): the existing production tool catalog schema — the new tool extends it
  via the 3-arm IS-14 pattern.
- I-17 tri-layer stack (Wave 3): the halting bound watchdog lives at I-17's check runner seam; I-8
  ships the tool + cache in Wave 1, and the full watchdog integration activates when I-17 lands.

End of I-8.

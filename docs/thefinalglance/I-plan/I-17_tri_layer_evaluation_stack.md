# I-17: tri-layer evaluation stack

- Track 15 — DETERMINISTIC GATES + ANTI-GAMING VERIFICATION | lever TRUST-PROVABLE | grounding STRONG | nature BUILD (composition) | effort M | WAVE 3 | BLOCKING for I-4 | binds ID-4 | IC-11 N/A (IC-11 is I-18)
- Premise: the verification substrate (D-3 adapters, D-11 checkers, F-1/F-9 gates, IS-8 MutationRunner) is all SHIPPED
  but never composed into a single veto stack the search can rank by. a fluent candidate with a hallucinated import can
  currently score non-zero before any L1 check fires. i-17 composes those existing surfaces into a tri-layer verdict:
  L1 hard veto supreme (never overridable, ID-4), L2 calibrated scorer on survivors only, L3 disagreement escalation.

## Steal/build
L1 HARD VETO (compile/test/F-1 VibeGate/F-9 dep-gate/D-11 checkers): any failure = score 0.0, the decision is
FINAL and CANNOT be overridden by L2 or L3 (ID-4 — deterministic veto is supreme; a neural scorer that resurrects
a vetoed branch is the exact failure mode this rule exists to prevent). 15s empirical halting bound: a deterministic
check spinning beyond the bound aborts the branch and charges it a HaltingBound penalty (shared with I-8 — compose,
do not duplicate the timer). FLAKE QUARANTINE: an oracle that flips across n-reruns under seed variation is ledgered
and barred from voting until rehabilitated; the vote exclusion is a hard gate, not a soft weight. L2 calibrated scorer
runs ONLY on L1-passing branches (D-2 PPM seam, I-23 conformal calibration as the uncertainty source, E-19 PRM-lite
until PPM ships live). L3 DISAGREEMENT escalation fires when L2 variance among survivors exceeds a threshold and
routes to the D-13 cross-family adversary for a concrete counter-example.
Cross-language verifier-pack seam (TS/Python packs behind the D-3 adapter trait) is noted as Tier-2 (W-10) — the
trait already accommodates new packs; the packs themselves are gated on per-language test harness confirmation.
EXCLUDED: any design that lets L2 re-score a vetoed branch; any "soft veto" that emits a penalty score instead of
hard 0.0; absorbing the D-3/D-11 substrate rather than composing it.

## Existing ground truth
- IS-8: metamorphic_testing.rs RelationLibrary:181 + mutation_testing.rs MutationRunner:354 + EquivalentMutantCache — SHIPPED; i-17 routes through them as L1 ground truth inputs
- D-3 verifier_adapters consensus surface — SHIPPED per track-15 anchor table; i-17 builds the dispatch layer over the existing adapter trait
- D-11 checker registry — SHIPPED; the L1 composite draws from it directly
- F-1 VibeGate + F-9 dep-gate — SHIPPED; pulled into the L1 compound without refactoring (GD-1)
- IS-7: conformal_calibration.rs + calibration_bootstrap.rs — SHIPPED; i-23 unifies them; i-17 consumes the unified facade as the L2 uncertainty source (compose i-23 first or wire to the raw calibration until i-23 lands)
- D-13 cross-family adversary — SHIPPED per D-plan; L3 routes into it
- IN-4: NO sprint_contract module yet (i-21 owns that; i-17 composes nothing from it)
- "PLAN-asserted; executor re-greps per W-7": exact seam file:line for the D-3 dispatch and the F-1/F-9 compound hook

## REVISED scope (W-10 tiered)
SHIP NOW (Tier-1):
- verifier_stack.rs: L1 compound (D-11 registry + F-1/F-9 + compile + test, each reporting its Gate result)
- 15s halting bound timer, shared with i-8's dead-end logic via a common halt::BoundTimer
- flake quarantine ledger (SSB-backed, keyed blake3(oracle-id + seed-range))
- L2 calibrated scorer invocation on survivors (reads from i-23's unified facade; stubbed to a direct conformal_calibration call if i-23 not yet shipped, REGISTERED stub per stub-tracking protocol)
- L3 disagreement escalation routing to D-13 adversary
- TriLayerVerdict SSB emission at every branch verdict (HardVeto / ScoredSurvivor / DisagreementEscalated / FlakeQuarantined)
DEFER (Tier-2 / gated):
- cross-language verifier packs (TS pack, Python pack) behind the D-3 adapter trait: seam ships now, packs ship
  separately after per-language harness confirmation and W-10 checkbox

## Integration
new module: crates/orchestrator/src/verifier_stack.rs (the compositor — calls into existing adapters, never
re-implements them). consumed by i-4's search ladder ranker (PLAN-asserted; executor re-greps the i-4 module seam at
EXECUTING, W-7) and by the baby-step audit dispatch in the existing task_runner flow. SSB action_type:
+TriLayerVerdict (Kind: HardVeto / ScoredSurvivor / DisagreementEscalated / FlakeQuarantined).
frontend_pair: per-branch verdict chips on the cockpit family (composing the existing branch board — no new drawer,
new row type per branch card).

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/verifier_stack.rs::TriLayerStack (the struct + run_verdict(&Branch) -> VerdictRow)
  - crates/orchestrator/src/verifier_stack.rs::FlakeQuarantineLedger (read/write ops over SSB)
  - TriLayerVerdict SSB action_type with HardVeto / ScoredSurvivor / DisagreementEscalated / FlakeQuarantined Kind discriminators
consumes_symbols:
  - crates/orchestrator/src/verifier_adapters.rs::ConsensusVerifier (D-3 adapter — why: L1 calls it first; GD-1)
  - crates/orchestrator/src/checker_registry.rs::run_all_checkers (D-11 — why: feeds L1 compound; GD-1)
  - crates/evaluation/src/vibe_gate.rs::VibeGate::evaluate (F-1 — why: mandatory L1 axis; GD-1, do NOT re-implement)
  - crates/cli/src/dep_check_cmd.rs::run_dep_gate (F-9 — why: mandatory L1 axis; GD-1)
  - calibration facade from i-23 (or conformal_calibration.rs::get_tau directly until i-23 ships — why: L2 uncertainty; GD-1)
  - crates/orchestrator/src/adversary.rs (D-13 — why: L3 escalation target; GD-1)
reachable_from:
  - crates/orchestrator/src/search/ladder.rs::rank_candidates (PLAN-asserted; the i-4 ranker calls verifier_stack::run_verdict per branch)  [PRODUCTION search ladder]
  - crates/orchestrator/src/task_runner.rs baby-step audit dispatch (existing seam; verifier_stack replaces inline gate calls)  [PRODUCTION task runner]
  (a #[cfg(test)] or contracts/ caller NEVER satisfies R7 — executor re-traces the production chain at EXECUTING)
stub_ban:
  - NO L2 or L3 call on a branch whose L1 verdict is HardVeto (veto supremacy is unconditional — ID-4)
  - NO un-quarantined flaky oracle participating in any vote (quarantine ledger check is mandatory before every oracle read)
  - NO soft veto (penalty weight in place of hard 0.0): L1 failures are binary, score is 0.0, not 0.001
  - NO absorption of D-3 / D-11 / F-1 / F-9 logic into verifier_stack.rs (compose via the existing interfaces — GD-1)
  - NO .unwrap()/.expect() on production paths; NO blake2/sha2 (blake3 only for the flake quarantine key); NO fabricated measurements
frontend_pair:
  per-branch verdict chips on the existing cockpit branch board surface. Tauri IPC: the TriLayerVerdict SSB rows are
  read by the existing SSB viewer (IS-19 wiring tooling) and surfaced as per-branch chips. cross_lang_diff PASS
  required (W-4). Justified as composition of the existing board, not a new drawer.
observable_effect:
  trigger: "elai run --task-fixture fixtures/hallucinated_import.toml --emit-ssb"
  expect_ssb: "TriLayerVerdict(HardVeto) row lands with branch_id + gate=L1_compile before any L2 call; journal shows L2 call_count=0 for that branch. ScoredSurvivor rows land for branches that clear L1. FlakeQuarantined row for a seeded flaky oracle run (seed range drives the flip)."
  expect_artifact: "a --l2-overrides-l1 broken arm (patch verifier_stack to treat HardVeto as a 'strong negative weight') resurrects the vetoed branch and the contract test asserts the wrong branch wins — FAILS (R2/ID-4). the unbroken arm picks the L1-passing branch."

## Grounding
- IS-8 (RelationLibrary:181 + MutationRunner:354): the L1 substrate; IS-7 (conformal_calibration): the L2 source; IS-19 (wiring tooling): the contract check surface
- IC-11 does NOT bind i-17 (IC-11 is the i-18 RRA/attestation correction); IC-12 does NOT bind (IC-12 is i-19); IC-13 binds via i-23 composition; IC-14 binds via i-20 composition
- ID-4 (evidence-ranked-search, veto-is-supreme) is the spine of this session; GD-1 (compose not restate) governs every D-3/D-11/F-1/F-9 reference
- Consolidated Part IV section "tri-layer stack" + review 1.1 "selection by receipts and veto supremacy"
- track-15 anchor table: I-17 draws on D-3 verifier_adapters + D-11 checkers + F-1/F-9 gates + IS-8
- arXiv IDs cited indirectly through the dossiers (CaMeL 2503.18813, FIDES 2505.23643 for the veto-supremacy principle) — WebSearch-confirm before any headline claim
- SSB: +TriLayerVerdict (Kind: HardVeto/ScoredSurvivor/DisagreementEscalated/FlakeQuarantined) — baseline+1 from LIVE 221/532 as of 2026-06-10 (ssb.rs:2981 / action_type_lock.rs:601); re-read at edit (W-9).

## COMPOSES-WITH
D-3 verifier_adapters + ConsensusVerifier (the adapter dispatch — do NOT re-implement); D-11 checker registry (do
NOT re-implement the checker logic); F-1 VibeGate (do NOT re-implement the compound evaluator, add an axis only); F-9
dep-gate (do NOT re-implement); D-13 adversary (L3 escalation — do NOT re-implement its critique logic); i-23
unified conformal calibration (L2 uncertainty — do NOT re-implement calibration math); i-20 budgeted mutation
scheduler (provides mutation scores as L1 evidence; do NOT duplicate the mutation runner); i-8 halting bound timer
(shared — do NOT implement a second timer).

FULL-FORM EXPANSION REQUIRED before EXECUTING (I-0 reading order rule) — effort M.

## J-11 AMENDMENT — S-2 L1 VETO SUPREMACY PRIOR ART (source-cited)
Grounding: Arbor `src/coordinator/tools/git_ops.py:402-446` (GitMergeBranchTool.execute), read directly per W-3.
GD-1 prior-art pointer: Arbor's merge gate independently confirms I-17's L1 HardVeto supremacy doctrine. When
the held-out eval returns `None` (verified_score is None, `git_ops.py:402`), the merge is rejected
unconditionally — no L2/L3 score can override it. When the verified score fails `is_improvement`
(`:437`), the merge is rejected regardless of the LLM-reported score's quality. This is the L1 veto pattern:
a deterministic check failure is a hard BLOCK, and no advisory score can resurrect a vetoed candidate. I-17's
tri-layer stack composes this: L1 HardVeto (deterministic) > L2 calibrated score > L3 adversary — the same
ordering Arbor's gate enforces informally (verified eval > LLM-reported score). The Arbor gate also confirms
the B_dev/B_test discipline (`prompts.py:442` "B_dev is for iteration ... B_test is for milestones only")
that I-17's held-out layer requires. Compose, do NOT restate — I-17 owns the tri-layer architecture.

End of I-17.

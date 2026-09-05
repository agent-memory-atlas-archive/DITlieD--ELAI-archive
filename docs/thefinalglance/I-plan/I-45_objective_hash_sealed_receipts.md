# I-45: objective-hash sealed receipts — stamp blake3(goal objective) into every PASS so "redefine done" goes stale

- Track 20 | lever CANT-LIE | grounding STRONG (ELAI side) / MODERATE (roach semantics, SL-11) | nature
  EXTENSION | effort S-M | WAVE 1 | binds ID-4 (the receipt is off-agent evidence)

- Premise: the STEAL-LIST's sharpest single steal (E1, roach-pi technique 1). A verifier PASS today proves
  the gates ran; it does NOT bind the PASS to the objective it was issued against. An agent (or a later
  session) that mutates the goal/acceptance text after a PASS keeps a receipt that still LOOKS valid.
  Fix: every sealed verdict carries blake3(canonical objective bytes); any consumer re-hashing the current
  objective against the receipt detects mutation. F-4 stdout_hash is the adjacent precedent (hashes the
  artifact); I-45 hashes the GOAL.

## Steal/build
- Steal the technique, not the code: roach-pi's agentic-harness goal/verifier system (see
  roach-pi-SUMMARY.md technique 1; SL-11 binding read rule — read the cited extension source under
  docs/.source/roach-pi/ before EXECUTING).
- Build: (a) a canonical-objective serializer (TaskSpec / baby-step acceptance text / EARS records when
  I-37 ships — stable field order, normalized whitespace, blake3); (b) an objective_hash field stamped
  into the RRA seal + attestation at verdict time; (c) a staleness check on every receipt READ path
  (`elai verify-attestation`, replay, plan-status consumers): current-objective hash != sealed hash ->
  ReceiptStale, hard FAIL, never a warning.
- EXCLUDED: no new verifier, no new gate, no objective-versioning UI, no migration of historical receipts
  (old receipts without the field verify as legacy with an explicit LEGACY marker, never silently as fresh).

## Existing ground truth
- crates/orchestrator/src/rra.rs + crates/orchestrator/src/attestation.rs + crates/cli/src/rra_cmd.rs all
  SHIPPED (SL-5; corrects the stale "rra.rs absent" note). I-45 extends these in place.
- F-4 stdout_hash shipped (the off-agent artifact-hash precedent). I-18 (PLANNED) adds chain_prev_hash —
  I-45's field rides the same record; coordinate field layout if I-18 ships first (compose, never fork).
- Objective sources: B-40 idea-framing TaskSpec types (orchestrator::idea_framing), baby-step acceptance
  blocks; I-37 EARS records become the richest source when they exist (forward-compose, not a dep).

## REVISED scope (W-10 tiered)
- SHIP NOW (Tier 1): serializer + seal-time stamping + read-time staleness check + contract test, all
  in-process deterministic.
- DEFER (Tier 2): per-requirement hashing granularity (waits on I-37); cross-session objective lineage.

## Integration
- crates/orchestrator/src/rra.rs (seal site: add objective_hash) + attestation.rs (verify leg) +
  rra_cmd.rs / `elai verify-attestation` (the read-path check). frontend_pair: stale-receipt badge in the
  existing RRA viewer row (no new component).

## WIRING_CONTRACT
- produces_symbols: orchestrator::rra::objective_hash field + canonical_objective_bytes() serializer;
  attestation verify leg ReceiptStale arm; ObjectiveSealed SSB rows.
- consumes_symbols: rra.rs seal path, attestation.rs verify path (SL-5), F-4 stdout_hash precedent (field
  layout mirrored), idea_framing TaskSpec (objective source).
- reachable_from: `elai run --emit-rra` seal path -> rra.rs (PRODUCTION seal site); `elai
  verify-attestation` (PRODUCTION read site). A contracts/-only caller NEVER satisfies this (R7).
- stub_ban: NO blake2/sha2; NO model-settable objective_hash (the serializer reads orchestrator-owned
  state only); NO warn-and-continue on hash mismatch (fail-closed, R1); NO silent legacy pass-through
  (legacy receipts marked LEGACY explicitly); NO .unwrap()/.expect() on prod paths.
- observable_effect:
  trigger: "elai run --emit-rra <fixture task>; then mutate the fixture objective text; elai
  verify-attestation <bundle>"
  expect_ssb: "ObjectiveSealed(Sealed) at seal; ObjectiveSealed(MutationDetected) + ReceiptStale on the
  post-mutation verify"
  expect_artifact: "the sealed bundle carries objective_hash; the NEGATIVE arm (unmutated objective)
  verifies PASS; the mutated arm FAILS verification naming the hash pair — a verify that passes after
  mutation is the broken arm and FAILS the gate (R2)."

## Grounding
- STEAL-LIST E1 (Tier 1 priority; "SHARPEST single steal"); SL-5 (substrate shipped), SL-11 (read rule).
- CLAUDE.md: blake3 only; wiring-enforcement R1/R2/R7/R8 (the staleness check joins the frozen gate set).

## COMPOSES-WITH
- G-2 RRA + G-8 attestation (extends in place), F-4 stdout_hash (sibling field), I-18 chain (same record —
  coordinate), I-37 EARS (future objective source), I-39 plan-status (a consumer that re-checks hashes).
  Restates none. SSB: +ObjectiveSealed (Kind: Sealed/MutationDetected/ReceiptStale). baseline+1 from LIVE
  221/532 (SL-1); re-read at edit (W-9).

End of I-45.

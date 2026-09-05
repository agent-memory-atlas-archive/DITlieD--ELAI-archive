# J-68: formal verification of ELAI's own critical code — Kani bounded-model-check + SPARK-style contracts + a TLA+ FSM spec

- Track J (NASA-gates wave) | lever PROVABLE | grounding STRONG (cbmc W6 + tla-specifying-systems W8 + spark-gnatprove W10 interface maps; kani/creusot/TLC anti-slopsquat-verified) | nature BUILD | effort L | WAVE NASA-GATES-3 | depends none (verifies SHIPPED core; sequenced after the cheaper gates)
- The "ELAI's OWN gate machinery is itself verified" half. Distinct from D-11 (z3/sympy on GENERATED math) — this verifies ELAI's Rust + its FSM spec.

- Premise: ELAI's gates verify the code ELAI WRITES; nothing formally verifies the small critical core ELAI RUNS ON (the SSB serializer, the FSM transition fn, the gate predicates, the budget counters). GROUNDED GAP: cbmc-interface.md (W6) names the technique — bounded model checking on small critical modules — and KANI is the active Rust analog (kani-verifier 0.67.0, anti-slopsquat-verified). spark-gnatprove (W10) adds contract-first pre/post + Absence-of-Run-Time-Errors obligations on privileged side-effecting fns (the CLAUDE.md no-.unwrap()/no-panic rule IS the runtime AoRTE guarantee; Kani makes it proof-time). tla-specifying-systems (W8) adds a design-time exhaustive-BFS FSM spec (ElaiFSM.tla + TLC) that complements phase-3I DST (runtime sim). All three are UNSOUND beyond their bound (ND-2: a Layer-2 leg, never a replacement for the runtime gates J-66/J-67). Scoped to the SMALL critical core ONLY (cbmc-interface.md is explicit: do not prove the world).

## Steal/build
1. D1 KANI BOUNDED-MODEL-CHECK GATE (W6/W10): #[kani::proof] harnesses on the modules named in cbmc-interface.md —
   - ssb/payloads.rs serialize/deserialize ROUND-TRIP (deser(ser(x)) == x for bounded x);
   - the FSM transition fn never panics on any bounded next-state;
   - the gate predicates are FAIL-CLOSED (evidence-absent -> Block, proven, not tested);
   - the wiring_contract parser never panics on bounded input;
   - the budget counters never underflow.
   The auditor runs `cargo kani` in CI; an unwinding-assertion failure OR a found counterexample = BLOCK; HARNESS_NOT_FOUND = BLOCK (fail-closed, R1/ND-3).
2. D2 SPARK-STYLE CONTRACTS ON PRIVILEGED ADAPTERS (W10): every side-effecting Supervisor-plane fn (file ops, egress, SSB write, credential handling) carries pre/post (assert! at entry, debug_assert! before return) + an AoRTE obligation discharged by a D1 harness BEFORE it is allowed to be a production side-effect sink. Pairs with the no-.unwrap() rule (panic = Constraint_Error = AoRTE failure). Crates: kani (primary), creusot 0.12.0 (optional gold); Prusti REJECTED (unmaintained).
3. D3 TLA+ FSM SPEC + TLC GATE (W8): commit ElaiFSM.tla (sketched in tla-specifying-systems-interface.md) with invariants — Inv2 (AUDITING only reachable from EXECUTING), Inv3 (no EXECUTING without evidence_gate=TRUE), Inv4 (COMPLETE only after MEMORIZING). A privileged FSM change (new state / transition / gate condition) ships a spec delta + a passing TLC run; an invariant counterexample = BLOCK. Design-time exhaustive BFS over the small finite state space + a concrete counterexample trace; complements phase-3I DST, never replaces it (ND-2).
EXCLUDED: proving the whole workspace (cbmc-interface.md hard boundary — small critical core only); Prusti (unmaintained); treating a Kani PASS as a runtime guarantee (unsound beyond the unwind bound — ND-2; the runtime gates stay load-bearing); a TLA+ spec that models the IMPLEMENTATION instead of the design contract (spec the FSM CONTRACT, not the Rust).

## Existing ground truth (PLAN-asserted unless noted; re-grep at edit, W-7/W-9)
- ssb/payloads.rs (the serializer the round-trip harness targets) — the SSB payload module; grep-confirmed the SSB lives at crates/orchestrator/src/ssb.rs + ssb/ subdir (2026-06-22). PLAN-asserted exact path; re-grep.
- the FSM transition fn (the 9-state machine: PLANNING/TESTING_FIRST/EXECUTING/AUDITING/MEMORIZING/COMPLETE/IDLE + substates) — D3 specs its contract; the transition fn is the D1 no-panic target.
- gate predicates (maintainability/production-caller/wiring fail-closed checks) — D1 proves their evidence-absent->Block branch.
- CLAUDE.md no-.unwrap()/no-.expect() in prod rule — the runtime AoRTE the D2 contracts make proof-time.
- phase-3I DST (madsim proptest 10000) — the runtime FSM simulator D3 complements with design-time BFS (SG-7).
- kani-verifier 0.67.0 / creusot 0.12.0 / TLC — toolchain deps (CI cost); anti-slopsquat-verified 2026-06-21, re-verify at edit.

## Integration
- NEW #[kani::proof] harnesses co-located with the verified modules (crates/orchestrator/src/ssb/payloads.rs proofs, the FSM transition fn proofs) + a `cargo kani` CI invocation the auditor runs; NEW ElaiFSM.tla + a TLC config + its CI invocation; D2 = assert!/debug_assert! pre/post added to the privileged Supervisor-plane fns. NO SSB action_type (verification artifacts are off-agent CI evidence, JD-1 — the proof result + the TLC log the executor did not author). The new deps (kani, creusot, TLC) are user-confirmation-flagged at edit (no new dependency without confirmation).
- frontend_pair: N/A — CI/verification artifacts; the verdict surfaces through the audit gate. Justified.

## WIRING_CONTRACT
produces_symbols:
- crates/orchestrator/src/ssb/payloads.rs #[kani::proof] round-trip harness (+ FSM-transition, gate-fail-closed, parser-no-panic, counter-no-underflow harnesses)
- docs/thefinalglance/J-plan/ElaiFSM.tla (the FSM contract spec + Inv2/Inv3/Inv4) + its TLC config
- pre/post assert! contracts on the privileged Supervisor-plane fns (D2)
- a .claude/gate-manifest.txt row for the `cargo kani` + TLC CI gates (frozen, R8)
consumes_symbols:
- ssb/payloads.rs serializer + the FSM transition fn + the gate predicates (the verified targets; NOT reauthored)
- the CLAUDE.md no-.unwrap() rule (the runtime AoRTE the contracts formalize)
- phase-3I DST (the runtime sim D3 complements; ND-2)
reachable_from:
- the AUDITING / CI path: the auditor runs `cargo kani` + TLC over the changeset; a counterexample BLOCKS  [PRODUCTION]
- a #[cfg(test)] proptest is NOT a substitute for the bounded proof (R7; the proof is the off-agent artifact)
stub_ban:
- NO `cargo kani` HARNESS_NOT_FOUND treated as pass (fail-closed; R1/ND-3)
- NO TLC counterexample waved through (an invariant violation on a privileged FSM change = BLOCK)
- NO proving the whole workspace (small critical core only; the cbmc-interface.md boundary)
- NO Prusti (unmaintained); NO claiming a Kani PASS as a runtime guarantee (unsound beyond bound; ND-2)
- NO TLA+ spec of the implementation (spec the design CONTRACT)
frontend_pair: N/A — CI/verification leg. Justified.
observable_effect:
- trigger: "the auditor runs `cargo kani` on the SSB-serializer + FSM-transition harnesses and `tlc ElaiFSM.tla` over a changeset that contains a deliberately broken transition fn (one that can reach AUDITING without passing EXECUTING) and a deliberately bad serializer edit"
- expect_artifact: "Kani reports a counterexample on the serializer round-trip harness -> BLOCK; TLC reports an Inv2 counterexample trace on the broken transition -> BLOCK; both surface to the audit verdict"
- expect_artifact (NEGATIVE, R1/R2): "the clean core passes both; a MISSING harness (HARNESS_NOT_FOUND) is FAIL not SKIP; a TLC run that reaches the spec but finds no states (a vacuous spec) FAILS (the dead path screams); the executor's local `cargo kani` is necessary-not-sufficient — the auditor's off-agent re-run is the authority, and gate_freeze_check.py proves the harnesses/spec were not edited by the run"

## Grounding
- W6 cbmc-interface.md (Clarke/Kroening/Lerda 2004 bounded model checking; the small-critical-core-only boundary) -> Kani analog; W8 tla-specifying-systems-interface.md (Lamport 2003; the ElaiFSM.tla sketch + Inv2/Inv3/Inv4); W10 spark-gnatprove-interface.md (contract-first pre/post + AoRTE).
- W5 rv-survey-interface.md (the ND-2 layer table: proof for invariants, RV for live policy — this is the proof leg, the runtime gates are the RV leg).
- kani-verifier 0.67.0 + creusot 0.12.0 LIVE; Prusti REJECTED (last 2024-03); TLC toolchain. anti-slopsquat-verified 2026-06-21, re-verify at edit.
- distinct from D-11 constructive-checker (z3/sympy on GENERATED math) — J-68 verifies ELAI's OWN Rust + FSM (plan PART 3).

## COMPOSES-WITH
phase-3I DST (the runtime FSM sim J-68 D3 complements with design-time BFS), D-11 constructive-checker (sibling but disjoint target: generated math vs ELAI's own code), the CLAUDE.md no-.unwrap() rule (D2 formalizes it), J-65 (verifies the same small core J-65's P10 rules flag), gate_freeze_check.py + .claude/gate-manifest.txt (R8 freeze of the harnesses + TLC config), J-70 (the DO-333 traceability row that records technique=model-check/proof). Restates none.

## Open questions / risks
- TOOLCHAIN COST: Kani + TLC in CI are real time/compute; scope to the small core and run them on the AUDITING path for changesets that TOUCH the verified modules only (a manifest-driven trigger), not every commit — and LOG when skipped (no silent skip; the skip reason is recorded, ND-3).
- UNWIND BOUND soundness: a Kani proof holds only to its bound — the bound is a named constant, documented, and the residual (paths beyond the bound) is the runtime gates' job (ND-2 honest residue, GD-5).
- TLA+ spec drift: the .tla must track the real FSM or it proves a fiction — bind the spec-delta requirement to any privileged FSM change (a frozen rule, R8), and treat a missing spec-delta on an FSM change as a BLOCK.

## Next
J-68 is the proof leg of the wave (ND-2: the cheapest SOUND layer for the small invariant core); J-66/J-67 are the runtime-RV leg; J-65 is the generated-code static leg. J-70's traceability matrix records which leg covers which objective so no property is silently unverified, and the IV&V independence riders keep the auditor that runs these proofs structurally separate from the run that produced the code.

End of J-68.

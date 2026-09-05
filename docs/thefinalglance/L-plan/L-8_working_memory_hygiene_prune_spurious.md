# L-8: Working-memory hygiene / prune spurious connections — keep false assumptions from poisoning later steps

STATUS: PLANNED (WAVE 3; M; BUILD-compose; EXTENSION(F-5/J-21/I-5) — the elai-lang prune over the live claim graph)

Objective: long-running agents accumulate false repo assumptions that poison every later step (the user notes' explicit warning; the elai-lang "pruning spurious connections" idea). L-8 keeps the agent's LIVE claim/belief graph honest: when a claim is later CONTRADICTED by evidence (a grep shows the assumed owner has no such symbols; a package is absent from the lockfile; a "this is unused" claim is refuted by a call-graph hit; the L-5 do-test proved a causal claim false), QUARANTINE it from the working belief graph so it is not re-injected downstream. It composes the SHIPPED hygiene fabric — the staleness quarantine cascade (F-5) + memory semantic-validation (J-21, re-derive a fact against commit+AST before reuse) + the pristine-context injector (I-5, exclude failed-path tracebacks) — threaded over the J-83/I-11 BeliefState as the carrier (BUILT, DC-1). HARD BOUNDARY: pruning QUARANTINES, never DELETES the record (RV — the SSB journal stays append-only; a pruned claim is recoverable); validity is re-checked against the CURRENT commit+AST before pruning (J-21), so a still-valid claim is never dropped. GD-1: do NOT re-implement F-5/J-21/I-5/BeliefState — thread them.

Estimated effort: ~1 session (~M). In-process; no network.
Pre-requisites (re-grep live per W-7): F-5 staleness quarantine cascade (apply_staleness_broadcast, dkg_lifecycle); J-21 memory semantic-validation (re-derive against commit+AST); I-5 pristine_path_filter.rs (BUILT, DC-1); J-83/I-11 BeliefState (belief_state.rs, BUILT, DC-1); L-1 ClaimSet; L-2 ClaimRisk (the contradiction signal).
Parallel-safe with: L-7 (disjoint). Depends L-1.
Source: cgg-hallucination-causal-system-2026-06-27.md S10 + the elai-lang prune screenshot + L-CGG-GROUNDING-MAP NN(prune)/TS-14/TS-15/DC-1. Doctrine: L-0_INDEX.md (RV, GD-2).

## Context
Why this matters: the predictors catch a hallucinated claim AT EMISSION; L-8 handles its AFTERLIFE. A claim that was plausible at step 3 but contradicted at step 7 must not keep steering steps 8..N. The elai-lang screenshot's examples are exactly the coding cases: "this file owns auth logic" removed when grep/AST shows no auth symbols; "this package is installed" removed when absent from the lockfile; "this function is unused" removed when the call-graph shows references. ELAI already has the three primitives (F-5 quarantine cascade, J-21 commit+AST re-derivation, I-5 pristine exclusion); L-8 is the thin layer that applies them to the LIVE claim/belief graph (J-83 BeliefState) keyed by the L-2 contradiction signal. The RV invariant matters: quarantine is reversible (the journal keeps the record), so a wrongly-pruned claim is recoverable — pruning never loses information.

## Existing state (F/I/J coverage)
- F-5 staleness quarantine cascade [WIRED] — apply_staleness_broadcast + flag_on_staleness_broadcast in dkg_lifecycle; collect_quarantined in live_cli; kill-switch ELAI_STALENESS_CASCADE_KILL. [MAP]
- J-21 memory semantic-validation [PLANNED] — re-derive a fact against commit+AST identity before inject; MemoryValidationEvent. [MAP]
- I-5 pristine context injector [BUILT, DC-1; pristine_path_filter.rs] — hard-exclude failed-sibling tracebacks; i5_pristine_context_contract.rs. [CG-0625]
- J-83/I-11 BeliefState [BUILT, DC-1; belief_state.rs] — the typed carrier the claim/belief graph rides. [CG-0625]

## Reads (mandatory before coding)
1. _grounding/L-CGG-GROUNDING-MAP.md (TS-14/15, DC-1); cgg-hallucination-causal-system-2026-06-27.md S10; the elai-lang prune screenshot.
2. dkg_lifecycle (apply_staleness_broadcast); J-21 re-derivation; pristine_path_filter.rs; belief_state.rs; L-1 ClaimSet + L-2 ClaimRisk.
3. F-0 W-1; G-0 GD-2; H Track-8 RV-1..RV-4 (reversibility); CLAUDE.md no-.unwrap, blake3.

## Tasks (WAVE 3 — quarantine contradicted claims, reversibly)
1. the contradiction detector (~50 LOC): a claim whose L-2 round-trip later FAILs, or whose L-5 do-test proved false, or which a fresh grep/AST/lockfile check contradicts, is marked contradicted (re-checked against the CURRENT commit+AST via J-21 first — a stale contradiction is not a contradiction).
2. the quarantine (~50 LOC): remove the contradicted claim from the live BeliefState working set via the F-5 cascade; the I-5 pristine injector keeps its traceback out of downstream context. QUARANTINE, never DELETE (RV — the SSB record stays; recoverable).
3. wire into the BeliefState carrier (~25 LOC): the claim/belief graph consults the quarantine set before any downstream injection. default behind ELAI_STALENESS_CASCADE_KILL (reuse the existing F-5 kill-switch; no new flag).
4. workflow/reasoning/L-8-decisions.md.

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/claim/prune.rs — the contradiction detector + the reversible quarantine over the BeliefState working set
consumes_symbols:
  - F-5 apply_staleness_broadcast (dkg_lifecycle); J-21 commit+AST re-derivation; I-5 pristine_path_filter.rs; J-83 BeliefState (COMPOSE all — never re-implement)
  - claim/mod.rs::ClaimSet (L-1); claim/risk.rs::ClaimRisk (L-2, the contradiction signal)
reachable_from:
  - the BeliefState downstream-injection seam > prune_contradicted [PRODUCTION working-memory hygiene]
  - a #[cfg(test)] / contracts/ caller NEVER satisfies this (R7)
stub_ban:
  - NO DELETE of the record (quarantine only; RV — the SSB journal stays append-only, recoverable)
  - NO pruning a still-valid claim (re-check against current commit+AST first, J-21)
  - NO re-implementation of F-5/J-21/I-5/BeliefState (GD-1 P0); NO .unwrap on prod; blake3 not sha2
frontend_pair:
  N/A with justification — runtime hygiene; quarantine state surfaces via the existing staleness/context panels. Justified.
observable_effect:
  trigger: "emit a claim 'src/cache.ts owns auth logic', then run a grep/AST check that contradicts it, then attempt a downstream injection (real BeliefState seam)"
  expect_ssb: "prefer-reuse the F-5 staleness rows / J-21 MemoryValidationEvent; net 0 expected from LIVE ~268/577 (re-grep, W-9)"
  expect_artifact: "the contradicted claim is quarantined from the BeliefState working set and NOT re-injected downstream; NEGATIVE: a still-valid claim is NOT pruned (validity re-checked against current commit+AST); the quarantined claim is RECOVERABLE from the SSB record (RV)"

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/L-plan/L-8_working_memory_hygiene_prune_spurious.md --run-trigger
rg "quarantine|contradict|prune" .elai_cc/crates/orchestrator/src/claim/prune.rs           # the prune present
rg -ci "\.remove\(|delete|\.clear\(" .elai_cc/crates/orchestrator/src/claim/prune.rs        # MUST be 0 on the record (quarantine, not delete — read context)
test -f workflow/reasoning/L-8-decisions.md
```

## ACCEPTANCE
- [ ] quarantine, never DELETE (RV — the record stays, recoverable).
- [ ] a still-valid claim is re-checked against current commit+AST before any prune (J-21); never wrongly dropped.
- [ ] the contradicted claim is not re-injected into downstream context (I-5).
- [ ] GD-1: F-5/J-21/I-5/BeliefState threaded, not re-implemented.
- [ ] cargo check --workspace GREEN; auditor PASS on -p orchestrator. No new dependency.

## WIRING DONE-GATE (clause by clause)
- W-1: done = a contradicted claim is live-quarantined and absent from the next downstream injection (not cargo-green).
- W-2: reuse the F-5 kill-switch ELAI_STALENESS_CASCADE_KILL; no new flag.
- W-3: threads BUILT primitives, not a summary-only port.
- W-4: frontend_pair N/A (justified).
- W-5/W-10: in-process; no network.
- W-7: dkg_lifecycle + pristine_path_filter.rs + belief_state.rs re-grepped at edit (DC-1: these are BUILT, not PLANNED).
- W-9: SSB prefer-reuse staleness/MemoryValidationEvent; net 0.
- GD-2: the quarantine decision is recomputable from the contradiction evidence under ELAI_SEED.

## Tests to add
- crates/contracts/tests/l8_prune_contract.rs (~75 LOC): (a) a claim contradicted by a fresh grep is quarantined; (b) a quarantined claim is absent from the next downstream injection; (c) a still-valid claim is re-checked and NOT pruned; (d) the quarantine is reversible (the record is recoverable from the SSB, RV); (e) an L-5-disproven causal claim is quarantined; (f) a stale (commit-changed) contradiction is re-validated before pruning (J-21).

## Files touched
- CREATE: .elai_cc/crates/orchestrator/src/claim/prune.rs (~125 LOC)
- MODIFY: the BeliefState downstream-injection seam (~25 LOC)
- CREATE: crates/contracts/tests/l8_prune_contract.rs (~75 LOC)
- CREATE: workflow/reasoning/L-8-decisions.md

## Citations / Grounding
- the elai-lang "pruning spurious connections" screenshot (remove high-confidence-looking but unsupported connections); the user notes' "bad assumptions poison later steps"; cgg-hallucination-causal-system-2026-06-27.md S10.
- ELAI ground truth (L-CGG-GROUNDING-MAP TS-14/15, DC-1): dkg_lifecycle apply_staleness_broadcast, pristine_path_filter.rs, belief_state.rs (BUILT; re-grepped at edit, W-7).
- CLAUDE.md: no .unwrap on prod, blake3-only; H Track-8 RV (quarantine reversible, never delete).

## Open questions / risks
- False-prune risk: quarantining a still-valid claim is itself a hallucination-of-the-gate; the J-21 commit+AST re-check + the RV reversibility bound it (a wrong prune is recoverable). OBSERVE-first to measure the false-prune rate (GD-4).
- Scope of the working set: which BeliefState entries are "live" enough to prune vs archival; decide the working-set boundary against the H-50 context policy at authoring.

## Next
L-8 closes the loop: detected hallucinations (L-3/L-4/L-5) and gated claims (L-7) are pruned from the belief graph so they cannot poison later steps. L-9 applies the whole CGG to the research/math deep-think profile; L-10 measures the aggregate hallucination reduction.

End of L-8.

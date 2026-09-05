# Working-memory hygiene / prune spurious claims (L-8)

- **ELAI ids:** L-8 / L-T1 / TS-14 / TS-15 / WAVE 3 / CANT-WRECK
- **Spec status (as designed):** PLANNED (`L-plan/L-8_working_memory_hygiene_prune_spurious.md:3`). Live `claim/prune.rs` is a **cousin**: `detect_contradicted` on repeated unresolved refs; it does **not** thread F-5/J-21/I-5/BeliefState or quarantine L-5-disproven causal claims.
- **Crate / module (spec):** `crates/orchestrator/src/claim/prune.rs`; production caller is the BeliefState downstream-injection seam `prune_contradicted` (`L-plan/L-8_working_memory_hygiene_prune_spurious.md:33-39`).
- **Purpose of this file:** enough architecture to rebuild reversible quarantine of contradicted live claims. Not a status ledger.

## 1. What it is

The **working-memory hygiene prune** is the elai-lang “pruning spurious connections” idea over ELAI’s live claim/belief graph. Predictors catch a hallucinated claim **at emission**; L-8 handles its **afterlife**. When a claim is later **contradicted** by evidence (grep/AST shows no such symbols; package absent from lockfile; “unused” refuted by call-graph; L-5 do-test proved causal false), **QUARANTINE** it from the working BeliefState so it is not re-injected downstream. Compose F-5 staleness cascade, J-21 commit+AST re-derivation, I-5 pristine-path filter, on the J-83/I-11 `BeliefState` carrier (BUILT, DC-1). Quarantine **never deletes** the SSB record (RV — recoverable). Validity is re-checked against the **current** commit+AST before prune (J-21) (`L-plan/L-8_working_memory_hygiene_prune_spurious.md:5`, `L-plan/L-SUMMARY_INDEX.md:60-65`, `L-plan/_grounding/L-CGG-GROUNDING-MAP.md:55-59`).

## 2. Why it exists

Long-running agents accumulate false repo assumptions that poison steps 8..N (`L-plan/L-8_working_memory_hygiene_prune_spurious.md:13`). If this organ is missing, a step-3 lie still steers step-9. If it is faked (delete the row, or prune without J-21 re-check), either history is lost (RV fail) or a still-valid claim is dropped (gate hallucination) (`:13,42-43,91`).

J-83 already **freezes** receipt-confirmed entries; L-8 **quarantines** entries that later fail. Complementary: freeze protects truth from oscillation; quarantine protects later steps from disproven assumptions (`J-83-grcv3-beliefstate-carrier.md` §1–2). Do not merge.

## 3. How it works (runtime)

1. **Who calls it.** BeliefState downstream-injection seam, `prune_contradicted`. Not contracts-only (R7) (`L-plan/L-8_working_memory_hygiene_prune_spurious.md:38-40`).
2. **Contradiction detector (deterministic)** (`:27`). A claim is contradicted if, **after** J-21 says the check is against the **current** commit+AST:
   - L-2/L-3 round-trip later **fails**, or
   - L-5 do-test proved the causal claim **false**, or
   - fresh grep/AST/lockfile/call-graph contradicts the assertion.
   A **stale** contradiction (commit moved) is **not** a contradiction until re-validated (`:27, f`).
3. **Quarantine (F-5, RV)** (`:28`). Remove from the **live working set** via `apply_staleness_broadcast`. SSB row **stays**. Recoverable. I-5 keeps the failed-path traceback out of downstream context.
4. **Consult before inject.** Working set checks the quarantine set before any downstream injection (`:29`).
5. **Kill-switch.** Reuse `ELAI_STALENESS_CASCADE_KILL` — **no new flag** (`:29,69`). Live `quarantine_disabled()` reads that env (`claim/prune.rs:61-66`).
6. **Commit.** Prefer-reuse F-5 staleness rows / J-21 `MemoryValidationEvent`. Net 0 (`:49`, `L-plan/L-0_INDEX.md:137`).
7. **Fail-closed.** `rg .remove(|delete|.clear(` on the **record** MUST be 0 (quarantine, not delete — read context) (`:56`). Still-valid claim never pruned (`:43`).
8. **The model may generate** none of this. It may **not** un-quarantine by rewriting BeliefState prose or deleting SSB.

Live cousin (do not rebuild as this organ): `detect_contradicted(prior_keys, set, resolver)` (`prune.rs:32-58`) only flags claims **seen before** whose File/Dep/Symbol/Api refs now fail the resolver. First pass never quarantines. Behavior/Causal/Reachability are treated as always resolving (`:44-50`) — so L-5-disproven causal is **invisible**. No BeliefState call. `content_key` is blake3(kind + sorted raw refs), span-independent (`:19-28`).

## 4. Types, enums, objects

Plan does not freeze a struct block. Implied (`:27-34`) plus live cousin fields:

```
fn prune_contradicted(working: &mut BeliefState, evidence, current_commit_ast) -> QuarantineResult

// live cousin:
QuarantinedClaim { content_key: [u8; 32], raw_refs: Vec<String>, kind: ClaimKind }
fn content_key(claim: &ExtractedClaim) -> [u8; 32]
fn claim_fact_id(key: &[u8; 32]) -> String   // "claim:{hex16}"
fn detect_contradicted(prior_keys, set, resolver) -> Vec<QuarantinedClaim>
fn quarantine_disabled() -> bool             // ELAI_STALENESS_CASCADE_KILL
```

**Consumed:** F-5 `apply_staleness_broadcast`; J-21 re-derive; I-5 `pristine_path_filter.rs`; J-83/I-11 `BeliefState` (`:35-36`). Do not re-author BeliefState (`I-11-typed-belief-state.md`, `J-83-grcv3-beliefstate-carrier.md`).

**EX-6 / L-24:** conjecture status is a journal fold; contradicted nodes quarantined never deleted (`L-plan/L-SUMMARY_INDEX.md:75`). Same RV; L-8 is the **code-lane** claim graph, L-24 is explore-lane conjectures — do not mix (EX-5).

## 5. Control flow / state machine

**Owner:** orchestrator hygiene at BeliefState inject.

```
downstream inject candidate claim
  kill-switch on? → skip prune (F-5 reuse)
  J-21 re-derive vs current commit+AST
    still valid → keep
    contradicted (P1 fail | L-5 false | grep/lockfile/call-graph)
      → F-5 quarantine working set
      → I-5 exclude traceback from next context
      → journal reuse staleness/MemoryValidationEvent
      → record remains in SSB (RV)
```

**Working-set vs archival:** open question vs H-50 (`:93`). Forced: live working set only until that is decided.

Parallel-safe with L-7 (`:9`). Depends L-1 (`:9`).

## 6. Grounding, receipts, gates

Contradiction evidence is **external** (resolver, do-test hashes, AST). LLM “forget that” is not a prune.

**Never allowed:** delete/clear of the journaled claim; prune without current-commit re-check; treating I-5 failed-path exclusion as delete.

**False-prune:** itself a hallucination-of-the-gate; J-21 + RV + observe-first bound it (`:91`).

## 7. Budgets, limits, ceilings

| Knob | Value | Scope |
|---|---|---|
| detector | ~50 LOC | `prune.rs` |
| quarantine | ~50 LOC | same |
| BeliefState wire | ~25 LOC | injection seam |
| file | ~125 LOC | (`:81`) |
| contract | ~75 LOC | (`:78`) |
| SSB | reuse F-5 / MemoryValidationEvent; net 0 | W-9 |
| kill-switch | `ELAI_STALENESS_CASCADE_KILL` | W-2 reuse |
| observe-first | false-prune rate (GD-4) | (`:91`) |

No GRC budget. No network (`:7`).

## 8. Roles and neighbors

Role-agnostic hygiene on the **carrier**, not a role prompt.

**Consumes:** L-1 ClaimSet, L-2 contradiction signals, F-5, J-21, I-5, BeliefState (`:35-37`).

**Produces:** quarantined working set; reused staleness/validation rows.

**Must not fork:** a second BeliefState; destructive `remove` of SSB; explore-lane conjecture store (L-24).

**Frontend:** N/A — existing staleness/context panels (`:45-46`).

## 9. Config / flags / defaults

- **No new flag.** `ELAI_STALENESS_CASCADE_KILL` (`:69`).
- Decision doc: `workflow/reasoning/L-8-decisions.md` (working-set vs H-50). **Absent**.
- J-21 may still be PLANNED — compose the re-derive interface (`:18`).
- blake3 content keys. No `.unwrap()` (`:44`).

## 10. Implementation map

| Symbol | Spec path |
|---|---|
| `claim/prune.rs` | detector + reversible quarantine (`:34`) |
| `prune_contradicted` | production name (`:39`) |
| F-5 `apply_staleness_broadcast` | `dkg_lifecycle` (`:36`) |
| I-5 `pristine_path_filter.rs` | BUILT DC-1 |
| `belief_state.rs` | J-83/I-11 |
| BeliefState inject seam | ~25 LOC (`:82`) |
| `l8_prune_contract.rs` | tests (a)–(f) (`:83`) |

Live: `detect_contradicted` / `content_key` / `claim_fact_id` / `quarantine_disabled` only (`prune.rs`). Contract covers repeated unresolved File, not (e)(f) fully.

## 11. Journal / SSB events

Prefer-reuse F-5 staleness / J-21 `MemoryValidationEvent` (`:49`, `L-plan/L-0_INDEX.md:137`). Net 0. GD-2: quarantine decision recomputable from contradiction evidence under `ELAI_SEED` (`:75`). No delete event.

## 12. Tests and observables

Named (`:78`):

| Id | Assert |
|---|---|
| (a) | claim contradicted by fresh grep is quarantined |
| (b) | quarantined claim absent from next downstream injection |
| (c) | still-valid claim re-checked and **not** pruned |
| (d) | quarantine reversible (SSB recoverable, RV) |
| (e) | L-5-disproven causal claim quarantined |
| (f) | stale (commit-changed) contradiction re-validated before prune (J-21) |

Wiring (`:53-57`): `rg quarantine|contradict|prune`; `rg -ci .remove(|delete|.clear(` MUST be 0 **on the record**.

**observable_effect** (`:47-50`, `L-plan/L-SUMMARY_INDEX.md:65`): `"src/cache.ts owns auth logic"` contradicted by grep → quarantined, not re-injected. Negative: still-valid never pruned; record recoverable.

**W-1:** contradicted claim live-quarantined and absent from the next injection (`:68`).

Adversarial: delete row; prune File that still exists; skip J-21; L-5 false causal left in working set.

## 13. Rebuild contract

Another harness has **this** organ iff all of the following are true:

- Contradicted live claims are **quarantined** from BeliefState working set, **never deleted** from the journal (RV).
- J-21 current commit+AST re-check **before** prune; still-valid stays.
- I-5 excludes the failed traceback from downstream context.
- F-5 cascade is the quarantine mechanism (compose).
- L-5-disproven causal is in the detector, not only unresolved File refs.
- Kill-switch is `ELAI_STALENESS_CASCADE_KILL` only.
- `prune_contradicted` on the real injection seam (R7).
- Tests (a)–(f). SSB prefer-reuse.

A `HashSet` of unresolved files with no BeliefState, or `vec.remove` of the claim row, is a **cousin**, not this organ.

## 14. Non-goals / banned designs

- Re-implementing F-5/J-21/I-5/BeliefState (GD-1 P0) (`:44`).
- DELETE / clear of the SSB record (`:42`).
- Pruning still-valid claims (`:43`).
- New kill-switch (`:69`).
- Standalone prune UI (`:45-46`).
- Mixing L-24 explore conjectures into the code-lane working set (EX-5).
- Using J-83 freeze as a substitute for later contradiction (freeze ≠ quarantine).

## 15. Source citations

Primary: `docs/thefinalglance/L-plan/L-8_working_memory_hygiene_prune_spurious.md` (full file). Index: `L-plan/L-SUMMARY_INDEX.md:60-65`. Doctrine: `L-plan/L-0_INDEX.md:101-102,137`; RV in L-8 file `:5,13`. Grounding map: `L-plan/_grounding/L-CGG-GROUNDING-MAP.md:55-59,116-117`. Neighbors: `I-11-typed-belief-state.md`, `J-83-grcv3-beliefstate-carrier.md`. Live: `claim/prune.rs`, `l8_prune_contract.rs`. Elai-lang prune: named in L-8 `:87`, not a separate path here.

## 16. Gaps

Label **UNCERTAIN**; do not fill:

- Working-set vs archival vs H-50 (`:93`).
- How F-5 `apply_staleness_broadcast` keys a `claim:` fact-id (`claim_fact_id` live helper vs plan silence).
- Grep/AST/lockfile **who** runs them (L-3 resolver vs new walker).
- Observe-first false-prune metric home (L-10?).
- Whether `detect_contradicted` first-pass-immune rule is spec or live-only.
- BeliefState field that holds the quarantine set.
- J-21 API when still PLANNED.

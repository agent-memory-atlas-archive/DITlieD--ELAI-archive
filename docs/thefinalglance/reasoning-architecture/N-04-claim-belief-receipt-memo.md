# One Claim/Belief object + receipt memo (N-4)

- **ELAI ids:** N-4 / TIER 2 / W-2+W-4+W-22+W-36 (interlace findings) / WAVE 3
- **Spec status (as designed):** PLANNED (BUILD-on-shipped). Residual after L-8: schema-align K-12 + GRC tags onto the L-1/J-83 carrier, then memoize receipts (`N-plan/N-4_claim_object_receipt_memo.md:1-21`, `N-plan/N-0_INDEX.md:153-159,354-355`, `ELAI-FEATURE-MAP.md:3245-3254`).
- **Crate / module (spec):** `crates/orchestrator` `ssb::payloads` BeliefState + the J-83 receipt-gate + (Tier-2) L-2 CGG call sites; C-30 cache generalization keyed by claim-identity / `trigger_signature`
- **Purpose of this file:** enough architecture to rebuild **one** canonical claim/belief type and the receipt memo that amortizes checker cost. Not a status ledger.

## 1. What it is

**N-4** is the anti-fragmentation of “a statement the system may or may not trust, with evidence.” Four independently designed typed objects already mean that: L-1 `ClaimSet{ClaimKind,span,refs}`, J-83 BeliefState entries `{claim,receipt,abstain,DKG-provenance}`, K-12 `critical_atoms{kind,value,must_preserve}`, and GRC grounding tags `"type:reference"`. N-4 makes them **ONE**: L-1 typed claims **are** J-83 entries (L-8 already threads this — finish it); the receipt attached at the J-83 gate **is** the evidence L-3 round-trips; K-12 `must_preserve` **=** `has-receipt`; GRC tags **join** the one object; J-21 re-derives the **same** object against commit+AST (`N-plan/N-4_claim_object_receipt_memo.md:7-8`, `N-plan/N-SUMMARY_INDEX.md:85-96`, `ELAI-SPEC-OVERVIEW.md:426`).

**AND** the **receipt memo** (W-4+W-22+W-36): before an expensive checker/resample (D-11 / F-4 / L-5 / L-4 N=5), look up claim-identity / `trigger_signature` in the J-96 case graph. A **HIT** re-validates cheaply via J-21 commit+AST identity instead of re-running the intervention. A **MISS** mints a new J-96 node. This generalizes C-30 `OracleAnswerCache` `(query_hash,project_hash)→answer` with trust-TTL, invalidated by F-5. Consulted by the J-83 reasoning gate **and** (Tier-2) the L-2 CGG (`:8-9,19-21`).

EXCLUDED: re-authoring C-30 (generalize it); re-designing L-8’s carrier (align K-12 + GRC tags **to** it); a **fifth** claim type (`:10`).

## 2. Why it exists

Four claim types means four validators, four SSB rows, four staleness stories — and **no** way for a recurring claim to reuse yesterday’s receipt. Every session re-pays full checker/resample cost even when the identical claim (same symbol, same commit-window) was proven already (`:5`). If this organ is missing, the local 7B lane stays checker-poor forever and L / J-83 / K-12 each re-implement “is this statement grounded.” If it is faked (a fifth `UnifiedClaim` struct **beside** L-1 and J-83, or a memo Hit on F-5-invalidated code), fragmentation is renamed, not collapsed (`:10,40`, `N-plan/N-0_INDEX.md:354-355`).

L-8 already threads L-1 over J-83. N-4’s **residual** is K-12 + GRC tags joining that one object — a **schema alignment**, not a redesign (`N-plan/N-0_INDEX.md:354-355`). The memo is C-30’s keying discipline applied to **reasoning** (“gets cheaper on YOUR repo”, K-18 moat #2) (`:8-9`). I-8 already caches **what FAILED** (dead-end AST hash); N-4 adds **what was PROVEN** (`:17`).

## 3. How it works (runtime)

1. **Who calls it.** Production (not a test stand-in):
   - **Tier-1 (ship now):** J-83 per-step receipt-gate on a live task, **before** D-11 / F-4 / L-5 / L-4 resample (`:19-20,35`).
   - **Tier-2 (defer):** L-2 CGG predictor path, **after** the J-83 side proves identity + invalidation is sound (`:21,36`).
2. **Input.** A claim about to pay an expensive check: L-1 fields on the J-83 entry, plus current commit/AST for J-21, plus F-5 staleness.
3. **Identity (deterministic).** Hash **blake3** (not sha2) over claim-identity / `trigger_signature` — J-96 already mints nodes with `trigger_signature` + blast cone + proof receipt (`:15,41`). Exact byte layout of the identity key is **UNCERTAIN** (§16); rebuild: **one** blake3 key, C-30-shaped `(identity, project/commit window)`.
4. **Memo lookup (deterministic, before the expensive call):**

```
lookup(claim_identity, commit_window) ->
  Hit        // J-96 node exists AND F-5 has not invalidated
  Miss       // no node
  Revalidated // Hit that passed J-21 identity this call
```

(`N-plan/N-0_INDEX.md:268`, `:8,45`)

5. **HIT path (cheap).** J-21 re-derives the **same** object against current commit+AST. Pass → skip checker/resample; emit `receipt_memo` Kind `Hit` or `Revalidated`. Fail / F-5 stale → **not** a Hit; fall through to miss path. A Hit on **changed** code **MUST FAIL** (`:16,40,45-46`).
6. **MISS path (pay once).** Run the expensive checker (D-11 / F-4 / L-5 / L-4). On proof, **mint** a J-96 case node (J-96 mint-on-F-4-proof stays the filter — N-4 does not loosen it). Emit `receipt_memo` Kind `Miss` (`:8,45`).
7. **Invalidate (F-5).** Staleness cascade drops the memo the way C-30 TTLs drop oracle answers. Do not invent a second invalidation FSM (`:8,16`).
8. **Align the type (schema, not a new engine).** Fold onto the **existing** L-8/J-83 carrier (`:19-20,29`):
   - L-1 `Claim{kind,span,refs,taint}` **is** the J-83 `Entry.claim` payload (L-22 `falsifier` is an **additive field on that same Claim**, not a new type — `L-22-claimant-falsifiers.md` §4, §8).
   - J-83 `grounding_receipt` **is** L-3’s round-trip evidence (`:7`).
   - K-12 `must_preserve` **iff** `has-receipt` (`:7`, `N-plan/N-0_INDEX.md:154-155`). LLM never sets `must_preserve` (`J-83-grcv3-beliefstate-carrier.md` §5).
   - GRC `"type:reference"` tags **join** the one object (they are not a parallel claim type) (`:7`).
   - L-8 `quarantined_claims` stays a **working-set** filter on the same carrier (quarantine, never delete) (`L-plan/L-8_working_memory_hygiene_prune_spurious.md:5,28`).
9. **Commit.** Prefer-reuse J-83 `BeliefStateEvent` + C-30 cache path. Add `receipt_memo` Pattern-2 Kind `Hit | Miss | Revalidated` (baseline+1) (`N-plan/N-0_INDEX.md:268`).
10. **Abstain / fail-closed.** No Hit on F-5-invalidated code. No fifth type. Corrupt/missing receipt still **ABSTAINS** at J-83 (N-4 does not let a memo Hit bypass a failed live receipt when J-21 disagrees) (`J-83-grcv3-beliefstate-carrier.md` §3 step 5; `:40`).
11. **The model may generate** claim text. It may **not** type the claim (L-1 off-agent), set `must_preserve`, write the receipt, force a memo Hit, or mint a parallel claim struct.

## 4. Types, enums, objects

Copy from the four **existing** objects this organ **aligns**. Do **not** invent a fifth struct name.

**Carrier (J-83 Entry on I-11 BeliefState — EXTEND, do not replace):**

```
// I-11 wire N-4 does not rip out (I-11-typed-belief-state.md §4)
BeliefState { version, tried, failed_because, current_ast_state, constraints,
              dkg_entities, token_delta, critical_atoms, quarantined_claims }

// J-83 completion (J-83-grcv3-beliefstate-carrier.md §4)
Entry {
  claim,                              // L-1 Claim (this IS the statement)
  grounding_receipt: Option<ReceiptRef>,  // None => open/abstain; K-12 must_preserve <=> Some
  confidence_bit,                     // READ H-39; model NEVER writes
  provenance: { dkg_key, source_ssb_seq },  // blake3
  frozen,
}
```

**L-1 payload (the statement — ADD fields, never re-type):**

```
Claim {
  kind: ClaimKind,     // File | Symbol | Dependency | Api | Behavior | Causal | Reachability
  span: ByteSpan,
  refs: Vec<ClaimRef>,
  taint: Option<Provenance>,
  falsifier: Option<Falsifier>,   // L-22 ADD on this same Claim
}
```

(`L-plan/L-1_typed_claim_extraction_substrate.md:27`, `L-plan/L-22_claimant_authored_falsifiers_demarcation.md:33`)

**K-12 join rule (named, not a new type):** `CriticalAtom.must_preserve == true` **iff** the aligned entry `has-receipt` (`:7`). Live I-11 already carries `critical_atoms: Vec<CriticalAtom>` (`I-11-typed-belief-state.md:45`).

**GRC tags join:** `type:reference` citations (Phase 2A six types: `file,grep,test,search,tool,training`) attach to the same entry’s grounding, not a sibling claim object (`02A-grc-state-machine.md` §6; `:7`).

**Memo (C-30 generalization — do not re-implement the cache):**

```
// C-30 shape (named by N-4, not re-authored): (query_hash, project_hash) -> answer, trust-TTL
// N-4 keys: claim-identity / trigger_signature  +  commit/AST window
fn receipt_memo_lookup(...) -> Hit | Miss | Revalidated
receipt_memo.Kind { Hit, Miss, Revalidated }
```

(`:8,15-16`, `N-plan/N-0_INDEX.md:268`)

**J-96 node (consumed, not redesigned):** mint on F-4 proof with `trigger_signature` + blast cone + proof receipt (`:15`). L-24’s `ConjectureNode` is **explore-lane unproven** and **must not** share this minting filter (`L-24-conjecture-memory.md` §1, ABORT-1).

## 5. Control flow / state machine

**Owner of the loop:** orchestrator J-83 gate (Tier-1), then L-2 (Tier-2). The model proposes claim text; the engine types (L-1), looks up, validates, checks or skips, freezes.

```
model emits text
  → L-1 extract Claim (off-agent)                // already specified
  → L-8/J-83 Entry.claim = that Claim            // finish the thread
  → K-12 must_preserve := has-receipt            // alignment
  → GRC type:reference tags join the Entry
  → receipt_memo_lookup(identity, window)
       │
       ├─ Hit + J-21 identity PASS + not F-5 stale
       │     skip expensive checker
       │     emit receipt_memo(Hit|Revalidated)
       │     J-83 COMMIT/FREEZE as today
       │
       ├─ Miss | J-21 FAIL | F-5 stale
       │     run D-11 / F-4 / L-5 / L-4
       │     on proof: mint J-96 node
       │     emit receipt_memo(Miss)
       │     J-83 COMMIT iff live receipt PASS
       │
       └─ changed code returning Hit  → FAIL (stub_ban)
```

**Forced:**
- Tier-2 L-2 consult is **deferred** until Tier-1 identity+invalidation is sound (`:21`).
- Memo Hit **cannot** override a J-21 fail or F-5 invalidate (`:40`).
- J-83 still **ABSTAINS** on missing/corrupt live receipt (`J-83-grcv3-beliefstate-carrier.md` §3).
- L-24 unproven conjectures **never** enter this proven memo (EX-5) (`L-24-conjecture-memory.md` §13).

**W-2 residual vs L-8:** do **not** rebuild the L-1↔J-83 thread; finish K-12 + GRC join (`N-plan/N-0_INDEX.md:354-355`).

## 6. Grounding, receipts, gates

This organ **does not** verify files. It **selects whether** to re-run a verifier or reuse a J-96 receipt, then J-21 cheap-checks identity.

**Selector hierarchy (K-16 RR-2, unchanged):** external receipt > vote over independent samples (receipt-absent only) > LLM self-eval **banned**. A memo Hit **is** a pointer to a prior **external** receipt, not a new confidence species. A memo Hit whose J-21 identity fails is **not** a receipt (`K-16-reasoning-receipts-doctrine.md` §3; `:8`).

**Never allowed as a confidence / memo signal:** LLM “I proved this yesterday”; scalar 0–1; window fill; SHA-2 identity; Hit on F-5-stale code; inventing a fifth claim type so L-1 and J-83 can both live (`:10,40-41`).

**L-3 round-trip** consumes the **same** receipt the J-83 gate attached (`:7`). Do not give L-3 a private evidence object.

## 7. Budgets, limits, ceilings

| Knob | Value | Scope |
|---|---|---|
| expensive work skipped on Hit | D-11 checker / F-4 tests / L-5 do-test / L-4 N=5 resample | per recurring claim (`:8`) |
| identity hash | **blake3** | per lookup (`:41`) |
| C-30 TTL | trust-tiered (C-30 owns numbers); F-5 is the code-change invalidator | per memo (`:8,16`) |
| schema | reuse BeliefState events + C-30 path; `+receipt_memo` Kind Hit/Miss/Revalidated; baseline+1 | `N-plan/N-0_INDEX.md:268` |
| Tier-1 vs Tier-2 | J-83 memo **now**; L-2 CGG consult **defer** | W-10 (`:19-21`) |
| fifth claim type | **0** | organ (`:10,38`) |
| J-96 minting filter | unchanged (F-4 proof) | (`:15`) |

C-30 numeric TTLs and J-21 identity equality are **not** restated here (GD-1). Unspecified: Hit latency SLO beyond “measurably cheaper / no checker call” (`:45-46`).

## 8. Roles and neighbors

**Consumes (compose, never re-implement):** L-1 ClaimSet, J-83 BeliefState (L-8 carrier), K-12 critical_atoms, GRC tags, J-96 case nodes, C-30 OracleAnswerCache, J-21 commit+AST identity, F-5 staleness, L-2..L-6 (`:31-33,52`).

**Produces:** aligned carrier (same symbols, joined fields); `receipt_memo` rows; J-96 nodes on miss-then-proof.

**Must not fork:** a fifth claim type; a second cache beside C-30; a proven-memo that L-24 conjectures can Hit; D-10 `Claim` / live `GroundingMode` cousin as a **parallel** statement type — D-10 remains the **mode router** over a claim, not a second ontology (`D-10-mode-routing-predicate.md` §4). Whether D-10’s `Claim {claim_type: Code\|Math\|…}` **joins** L-1 `ClaimKind` is **UNCERTAIN** (§16); do **not** invent a merge enum.

**L-22 / L-24:** additive `falsifier` on L-1 Claim; `ConjectureNode.skeleton` **is** that class. N-4 collapse **forbids** renaming those into new claim types.

**I-11 file in this pack:** “N-4 later unifies Claim/Belief — cite, do not merge here” (`I-11-typed-belief-state.md` §8). This file **is** that merge. Keep `BeliefState::new` / `emit_condensed_receipt` (GD-1).

## 9. Config / flags / defaults

- N-4 names **no** new ENABLE flag. Memo is intake at the J-83 gate (`:35`).
- C-30 / F-5 existing flags and TTLs stand; do not add a `ELAI_RECEIPT_MEMO_ENABLE` that becomes a terminal default-off (N-plan wiring W-2 is a **different** W-2 from interlace W-2 — see §16).
- Frontend: memo-hit indicator on the existing belief/claim drawer, **or** N/A justified (`:25,42`).
- blake3 for claim-identity; no `.unwrap()` / `.expect()` on prod (`:41`).

## 10. Implementation map

| Symbol | Spec path |
|---|---|
| `ssb::payloads` BeliefState / J-83 gate | aligned carrier (`:23-24,29`) |
| C-30 OracleAnswerCache generalization | memo lookup (`:30`) |
| J-96 case nodes | Hit store / Miss mint (`:15,32`) |
| J-21 commit+AST | cheap re-validate (`:16,32`) |
| F-5 staleness cascade | invalidate (`:16,33`) |
| L-2 CGG call sites | Tier-2 consumer (`:24,36`) |
| `receipt_memo` SSB | Kind Hit / Miss / Revalidated (`N-plan/N-0_INDEX.md:268`) |

No new crate. No new cache engine.

## 11. Journal / SSB events

From `N-plan/N-0_INDEX.md:268` and `:45`:

```
receipt_memo {  // Pattern-2; baseline+1; reuse BeliefState events + C-30 path
  Kind: Hit | Miss | Revalidated
}

// first pass of the observable:
//   checker runs + J-96 node minted + receipt_memo(Miss)
// second pass, unchanged code:
//   receipt_memo(Hit) and checker skipped
```

Payload fields beyond Kind (identity hex, node id, cost delta) are unspecified (§16). Do not rewind schema numbers.

## 12. Tests and observables

From `:43-46` and stub_ban `:37-41`:

| Assert | Notes |
|---|---|
| recurring claim, **unchanged** code, twice | first: checker + J-96 mint + Miss; second: Hit, **checker skipped**, measurably cheaper |
| claim on **changed** (F-5-invalidated) code | stale Hit **FAILS** |
| no fifth claim type | `rg` of a new claim struct beside L-1/J-83/K-12 **0** in this organ |
| C-30 not reimplemented | generalize; do not copy a second cache module |
| identity hash | blake3 |
| K-12 join | `must_preserve` true iff `has-receipt` |
| L-1 **is** J-83 `Entry.claim` | one object after alignment |
| Tier-2 off until Tier-1 sound | L-2 consult deferred |

N-4’s source file does **not** name a `n4_*_contract.rs`. Rebuild from the observable + stub_ban; do not invent extra named tests.

## 13. Rebuild contract

Another harness has **this** organ iff all of the following are true:

- **One** Claim/Belief object: L-1 claims **are** J-83 entries; K-12 `must_preserve` **=** has-receipt; GRC `type:reference` tags join that object. **Zero** fifth claim type.
- L-8’s L-1-over-J-83 thread is **finished**, not redesigned. I-11 `BeliefState::new` / condense kept.
- Before D-11 / F-4 / L-5 / L-4 expensive work, lookup claim-identity (`blake3`) / `trigger_signature` in J-96 (C-30 keying).
- Hit → J-21 identity re-validate, skip checker. Miss → run checker, mint J-96 on proof. F-5 stale / changed code **cannot** Hit.
- `receipt_memo` Kind `Hit | Miss | Revalidated` journaled. Recurring unchanged claim is cheaper on the second pass.
- J-83 still commits only on external receipt (RR-1). Memo is a **cached receipt pointer**, not LLM confidence.
- L-22 `falsifier` and L-24 skeletons stay **fields/uses of L-1**, not new types.
- L-2 CGG consult is the **same** memo, Tier-2, not a second cache.

A `UnifiedClaim` wrapper next to L-1, a prompt “reuse yesterday’s proof”, or a Hit that ignores F-5 is a **cousin**, not this organ.

## 14. Non-goals / banned designs

- A fifth claim type (`:10,38`).
- Re-implementation of C-30 (`:10,39`).
- Re-design of L-8’s carrier (`:10`).
- Memo Hit on F-5-invalidated / changed code (`:40`).
- SHA-2 identity (`:41`).
- Loosening J-96 mint-on-F-4-proof so unproven L-24 conjectures can memo-Hit (`L-24-conjecture-memory.md` ABORT-1).
- Using the memo as a confidence score or letting the model set `must_preserve` / Hit (`K-16-reasoning-receipts-doctrine.md` RR-1; J-83 TD-3).
- Shipping L-2 CGG consult **before** J-83 identity+invalidation is sound (`:21`).
- Merging F-6 SkillTrustTier or N-6 provenance scales into this object (those are N-6; map, don’t merge) (`N-plan/N-0_INDEX.md:167-172`).

## 15. Source citations

Primary: `docs/thefinalglance/N-plan/N-4_claim_object_receipt_memo.md` (full file). Index: `N-plan/N-SUMMARY_INDEX.md:85-103`. Doctrine: `N-plan/N-0_INDEX.md:153-159,268,354-355`. Overview: `ELAI-SPEC-OVERVIEW.md:419-428`. Feature map: `ELAI-FEATURE-MAP.md:3245-3254`. Interlace findings named in-file: W-2/W-4/W-22/W-36 (`N-plan/N-4_claim_object_receipt_memo.md:48-49`). Neighbors: `I-11-typed-belief-state.md`, `J-83-grcv3-beliefstate-carrier.md`, `L-22-claimant-falsifiers.md`, `L-24-conjecture-memory.md`, `K-16-reasoning-receipts-doctrine.md`, `02A-grc-state-machine.md`, `L-plan/L-1_typed_claim_extraction_substrate.md`, `L-plan/L-8_working_memory_hygiene_prune_spurious.md`.

`ELAI-EXHAUSTIVE-SUPER-FEATURE-LIST.md` has **no** N-4 atom block. Do not mint N4-E### ids here.

## 16. Gaps

Label **UNCERTAIN**; do not fill:

- Exact rust identity-key layout (which Claim fields + commit SHA enter the blake3) (`:8` names claim-identity/`trigger_signature` only).
- `Hit` vs `Revalidated` distinction beyond “J-21 ran this call” (`N-plan/N-0_INDEX.md:268`).
- `receipt_memo` payload fields (node id, duration, checker skipped).
- Whether D-10 `Claim`/`ClaimType` (Code/Math/Prose/…) **joins** L-1 `ClaimKind` (File/Symbol/…) or stays a router DTO. N-4’s four named objects do **not** include D-10 (`:7` vs `D-10-mode-routing-predicate.md` §4).
- C-30 TTL numbers and J-21 equality predicate (owned by those units; not copied here).
- Whether a Hit still emits a J-83 `EntryCommitted` or only `receipt_memo`.
- Named contract-test file (N-4 source lists observable + stub_ban only).
- Interlace W-2/W-4 (four types / no memo) vs F-0/N-0 wiring W-2/W-4 (default-off flags / frontend_pair) — same labels, different doctrines (`N-plan/N-0_INDEX.md:67,72` vs `N-plan/N-4_claim_object_receipt_memo.md:48`). Rebuild the **interlace** findings for this organ.
- How GRC `training:` Uncertain citations interact with K-12 `must_preserve = has-receipt` (2A warns, does not reject).

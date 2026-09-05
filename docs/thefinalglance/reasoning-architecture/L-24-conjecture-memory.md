# Conjecture-memory middle tier (explore lane only) (L-24)

- **ELAI ids:** L-24 / L-T5 / EX-5 / EX-6 / C1 / legs CM, A, B, D / PS-3
- **Spec status (as designed):** PLANNED (WAVE 2; BUILD + 3 EXTENSION legs). Store ships loop-independent (FORK-1). Production retrieval consumer is L-23. J-96..J-98 **UNCHANGED** (`L-plan/L-24_conjecture_memory_middle_tier.md:1-7`, `L-plan/L-0_INDEX.md:74-75,123-124`).
- **Crate / module (spec):** `crates/orchestrator/src/reason/conjecture_store.rs`; SSB `ConjectureLifecycle` (or FORK-2 Kinds on `ExploreRoundCompleted`); UnblockCondition on D-17/L-7; L-10 explore-arm novelty column; FD-1..FD-3 contract on I-30/I-31
- **Purpose of this file:** enough architecture to rebuild the status-explicit unproven-idea store between the journal and the verified case graph, plus the three collab riders. Not a status ledger.

## 1. What it is

The **conjecture-memory middle tier** is a materialized **view** over the append-only journal (L-16 fold pattern). ELAI’s memory plane otherwise has two tiers: SSB (full genealogy, **no** status semantics) and the verified case graph J-96..J-98 (**proven** fixes only, minted on an F-4 proof). Nothing sat between: an unproven-but-live explore conjecture was either an unretrievable journal row or nothing. L-24 stores unproven conjectures as first-class retrievable objects with **computed** status (`pending` / `live` / `refuted` / `suspended`), provenance, attached criticisms carrying the N-16 evaluator-id + version-hash edge, and declared missing tests (unrun L-22 falsifiers) (`L-plan/L-24_conjecture_memory_middle_tier.md:7`, `ELAI-SPEC-OVERVIEW.md:380-387`, `ELAI-FEATURE-MAP.md:5119-5123`).

HARD BOUNDARY (locked, do not re-litigate): this tier serves the **EXPLORE / RESEARCH lane ONLY**. The code-fix reuse path keeps proven-only minting. An unproven conjecture is **structurally unretrievable** into the code-fix / auditor / case-graph path (EX-5: type-level `ExploreLaneToken` + journaled `Retrieved` + negative contract). Status is a **pure journal fold**, never a stored mutable field (EX-6) (`:7`, `L-plan/L-0_INDEX.md:74-75`).

This is a **runtime** organ of the explore lane. It is not the planning-harness improvement-conjecture archive: no FDL frontier controller, ProbeUtility scheduler, multi-cycle operator allocator, or bridge from that archive into L-23/L-24 (`ELAI-SPEC-OVERVIEW.md:393-399`).

## 2. Why it exists

Proven-only case minting is “clean but sterile”: a memory that only holds what current evaluators recognize cannot hold ideas **ahead of** the current receipt plane (collab C1, 2026-07-11) (`:7,14`, `ELAI-FEATURE-MAP.md:5118-5123`). If this organ is missing, explore cannot retrieve last round’s live rivals as labeled unproven ideas. If it is faked (hand-set status, a second claim type, or leaking nodes into J-96 retrieval), either the proven path is contaminated or EX-6 is theater.

Load-bearing transfers from DeepReason spec §4 (DESIGN-RATIONALE, ID-5): labels **computed**, never curated; refuting a premise **suspends** dependents, does **not** refute them (`orphaned != false`); **no absorbing status** (N1) — a fallen refuter reinstates the target, composing N-16 S4 evaluator-refutation collapse (`:14`). Negative case law stays at the L-23 **gate**, never in packs (spec §11.5 = EX-1) (`:14,51`).

Honest residue (GD-5): the tier **labels** conjectures; it does not make them good. A wrong-but-stable conjecture that never meets its missing test stays live-labeled-unproven. The only lever is running the declared tests. The lane wall is on **retrieval surfaces**, not on raw journal readers (`:160`).

## 3. How it works (runtime)

1. **Who calls it.** Production (R7): L-23 pack assembly → `conjecture_store.retrieve(ExploreLaneToken, …)`; D-17/L-7 refusal path → UnblockCondition composer; `elai bench hallucination-delta` explore arm → novelty column (`:100-102`). A `#[cfg(test)]` caller does **not** satisfy R7.
2. **Input.** An L-22 falsifier-carrying skeleton (the conjecture object **is** that class — never a second skeleton type) plus journal rows (`:91,39`).
3. **Register (deterministic host).** Build `ConjectureNode` with `lane: Lane::Explore` (the **only** constructible lane for this type), provenance `{session, round, model-pin}`, criticisms vec, `missing_tests` = unrun falsifiers + what result would settle each. Registration requires the L-22 skeleton with declared falsifiers; forbids-nothing is demarcation-refused (L-22 HG-5 / ATTACK-1) (`:39,164`). Emit `ConjectureLifecycle` Kind `Registered`.
4. **Retrieve (type-level wall, EX-5).** `retrieve(lane_token: ExploreLaneToken, …)`. The code-fix path **cannot name** `ExploreLaneToken` — not a runtime string check. Default pack: **live + pending** only, top-K, deterministic match on skeleton scope/mechanism (never LLM-scored). Each hit is rendered **labeled** with status + missing tests (unproven enters the pack **as** unproven). Suspended / quarantined: only `--include-quarantined` debug read, never default pack. Retrieval **itself** journals Kind `Retrieved` with requesting lane + round id (`:39,51,53`).
5. **Status fold (pure, EX-6).** Status is **not** a struct field. `fold(journal) -> status map`:

| Status | Derivation |
|---|---|
| `pending` | registered, **no** criticism executed yet |
| `live` | **≥1 executed criticism pass survived**. A never-criticized node can **never** be live (ATTACK-3) |
| `refuted` | a compiled falsifier fired **or** an exogenous-anchored attack row landed |
| `suspended` | a **premise** node is refuted → dependents suspend (`orphaned != false`); **OR** a vouching evaluator was refuted → N-16 S4 cascade suspends what it vouched for |

No absorbing status (N1): a refuter row later refuted (evaluator collapsed, checker bug-fixed) **reinstates** the target on the next fold. Quarantine (contradiction / SF-4 poisoning suspect) is a **fold OUTPUT lane**, excluded from default retrieval, **never a deletion** (`:42,141`).

6. **Refuted set → L-23 gate, never the prompt (S3/EX-1).** Battery-equivalence / I-8 hash at registration. Do not render refuted/banned conjecture content into the pack prompt (`:51,108`).
7. **FORK-1.** If L-22/L-23 files are absent at execution: ship store + fold + fixture-driven retrieval harness **now**; pack wire is a named follow-on in L-23. The negative lane test runs **regardless** (`:53,70`).
8. **Leg B — UnblockCondition (deterministic, receipts only).** Every D-17/L-7 abstention/refusal (and GRC-v3 abstain steps — J-83 cross-linked, not amended) carries:

```
UnblockCondition {
  tried:   [checks/retrievals actually RUN, from the GRC receipt trail],
  missing: [typed evidence that would change the answer — checker id + input, retrieval target, human ruling],
  route:   [which rung supplies it]
}
```

Composed **only** from journaled receipts. Model may **propose** text; harness records receipts it can **cite** (TD-3). A `tried` entry without a matching receipt row is **unconstructible** (ATTACK-5). D-17 SLA: a refusal **without** UnblockCondition is an SLA violation (fail-closed on the **emitter**, not the user). A refusal becomes a **pending conjecture about its own answer** (`:54,143`). If receipts only journal successes: add failure/unavailable receipts at the check site, then compose — never fabricate `tried` (`:56,193`).

9. **Leg A — L-10 novelty column (advisory).** NLI meaning-cluster novelty via L-4/F-12 deterministic-argmax entailment clustering: cluster survivor-map mechanisms by **meaning**, count clusters + coverage. **Never** cosine-over-hash. Reported under CGG ON/OFF × retrieval ON/OFF. Advisory; never adjudicates (TD-3). Missing column on any arm = FAIL (R1) (`:58,144`).
10. **Leg D — FD-1..FD-3 contract-first (no builder ahead of I-30).** Binding admission contract on I-30/I-31 dataset rows; **code lands inside I-30/I-31 when D-026 unlocks** — no wire-dark machinery ahead of the consumer (`:61,145`):
    - **FD-1** train on **MOVES** not survivors (include criticism/refutation/abort; never winning trajectories only).
    - **FD-2** lambda-filtered labels: admit only if the verdict window’s N-16 S6 exogenous-grounding ratio clears the floor (never distill rubric-consensus). Floor observe-first (GD-4).
    - **FD-3** every dataset row carries the N-16 evaluator-id+version-hash edge so S4 collapse extends to training data.
11. **Commit / fail-closed.** Fold twice under `ELAI_SEED` → byte-identical maps (GD-2). Corrupt journal row → fold **fails closed** (err, never a default status) (`:171`). Hand-set status surviving the fold → **FAIL**. Code-lane retrieval of any `ConjectureNode` → **FAIL**. J-96/J-97/J-98 edits → **ABORT-1** (`:79,116`).
12. **The model may generate** skeleton text and proposed missing-test prose. It may **not** write status, construct `ExploreLaneToken` from a code lane, rank retrieval by LLM score, author UnblockCondition, or delete a node.

## 4. Types, enums, objects

Copy from `L-plan/L-24_conjecture_memory_middle_tier.md:39-54,61`. Do not invent fields.

```
ConjectureNode {
  skeleton,                         // L-22 class; NEVER a second skeleton type
  lane: Lane::Explore,              // ONLY constructible lane for this type (EX-5)
  provenance: { session, round, model_pin },
  criticisms: Vec<CriticismRef>,    // SSB row id + evaluator-id + version-hash (N-16 ONE shape)
  missing_tests: Vec<DeclaredTest>, // unrun L-22 falsifier + what result would settle it
  // NO status field (EX-6). rg "status\s*:\s*ConjectureStatus" MUST be 0
}

ExploreLaneToken                    // type-level wall; code-fix path cannot NAME it
fn retrieve(lane_token: ExploreLaneToken, ...) -> ...

enum ConjectureStatus {             // FOLD OUTPUT only, not stored
  Pending,
  Live,
  Refuted,
  Suspended,
}
// quarantine = fold OUTPUT lane (exclusion), not a 5th stored status

UnblockCondition {
  tried:   [...],                   // from GRC receipt trail only
  missing: [...],
  route:   [...],
}

ConjectureLifecycle.Kind {
  Registered, Criticized, Refuted, Suspended, Reinstated, Quarantined, Retrieved
}
// FORK-2: same Kinds on ExploreRoundCompleted, net 0; Retrieved is NON-NEGOTIABLE either way

// Leg D row-schema fields (fields only, no builder unless I-30 type already exists):
//   evaluator-id + version-hash (N-16 shape); FD-1..FD-3 admission contract
```

DeepReason names `accepted` / `suspended_unsupported`. ELAI names `live` / `suspended`. Do not import Dung extension types (`:108`).

## 5. Control flow / state machine

**Owner of the loop:** `conjecture_store` fold + L-23 retrieval. Status is **recomputed**, not transitioned by an agent. No token-level memory FSM. No FDL controller (`ELAI-SPEC-OVERVIEW.md:393-399`).

**Allowed fold outcomes (truth table the decisions doc must record):**

| Journal evidence | Status | Retrieval default |
|---|---|---|
| Registered, zero executed criticisms | pending | yes (labeled unproven) |
| ≥1 criticism pass, not killed | live | yes |
| compiled falsifier fired / exogenous attack | refuted | **no** (feeds **gate**, not prompt) |
| premise refuted | **suspended** (not refuted) | no (debug only) |
| vouching evaluator refuted (N-16 S4) | suspended | no (debug only) |
| refuter later falls | **reinstated** (next fold) | per new status |
| contradiction / SF-4 poison suspect | quarantine lane | no (debug `--include-quarantined`) |
| delete requested | **illegal** (SF-4 / EX-6) | — |

**Forced:**
- Never-criticized → **never** live (`:42,164,171`).
- Premise refute → dependents **Suspended**, not Refuted (`:42,115`).
- No absorbing status (`:42,141`).
- Code-lane `retrieve` → zero nodes + journal nothing (`:171`).
- Unblock-less D-17 refusal → SLA violation (`:54`).

**If fold cannot derive a status from journal rows alone:** that is a **journal-completeness bug** at the criticism site — add the missing SSB emission. **Never** a hand-set field. If hand-curation is the only path → **ABORT-2** (`:44,80`).

## 6. Grounding, receipts, gates

**Selector hierarchy:** compiled L-22 falsifier fire (execution) > exogenous-anchored attack row > N-16 evaluator-refutation cascade > executed criticism pass (live vs pending) > labeled pending. LLM “this idea is live” is **never** a status. Embedding cosine is **never** novelty (leg A) or relapse (L-23 S2).

**Receipts:** `ConjectureLifecycle` chain; `Retrieved` names `lane=explore`; UnblockCondition cites receipt ids; novelty column in L-10 report JSON.

**Never allowed as a confidence / status signal:** stored status field, hand curation, LLM retrieval rank, model-authored unblock, cosine novelty, Dung grounded extensions, DeepReason numeric findings (ID-5) (`:105-109,161`).

**K-19 SF-1..SF-4 bind this file** (memory-reuse class): execution-grounded fitness only, observe-first, **never-into-auditor**, **never-DELETE**, poisoning quarantine (`:28,94,146`).

## 7. Budgets, limits, ceilings

| Knob | Value | Scope |
|---|---|---|
| retrieval | top-K live+pending; deterministic scope/mechanism match | per explore pack (`:51`) |
| live threshold | seed **≥1** executed criticism pass; observe-first; L-10 judges | per node (`:191`) |
| match fallback | BM25 over skeleton text (existing 1B) **if** match quality poor; decide at executing | (`:192`) |
| lambda floor (FD-2) | N-16 S6; observe-first before it acts (GD-4) | training admission (`:61,159`) |
| schema | `+ConjectureLifecycle` baseline+1 from LIVE **581 variants / SCHEMA 272** (`ssb.rs:1530-1544`, 2026-07-11); re-read at edit; derives toward 273; FORK-2 net 0 | (`:47,114`) |
| J-96 files edited | **0** (ABORT-1) | organ (`:79`) |
| I-30 builder code in L-24 | **0** ahead of D-026 | leg D (`:61,145`) |
| network | none (W-10 Tier-1) | (`:9,154`) |
| LOC budget (plan) | store ~250, ssb ~15, L-23 wire ~30, leg B ~70, leg A ~50, contract ~130 | (`:173-180`) |
| fold cost | if long-journal recompute grows: materialize with L-16 enumerated-recompute-causes, **never** a mutable store | (`:194`) |

**Overflow / flood (ATTACK-1 residue):** well-formed junk drip is not structurally detected; it stays **pending** (needs executed criticism to become live) and surfaces as N-16 S7 criticism-debt (`:164`). Top-K bounds pack drowning.

## 8. Roles and neighbors

**Consumes:**
- L-22 skeleton / falsifiers — the object; `missing_tests` = unrun falsifiers (`:91`).
- L-23 pack + relapse gate — production consumer; refuted set → **gate** (`:92`).
- N-16 evaluator-id+version-hash + S4 + S6 — **ONE** field shape (FORK-3: if N-16 not landed, L-24 lands the **same** shape N-16 names; N-16 consumes it — ND-1 unify-not-fork) (`:72,93`).
- F-5 quarantine shape (`:94`).
- L-4/F-12 NLI judge; L-10 bench (`:95`).
- D-17 SLA tick + L-7; GRC receipt trail (`:96`).
- SSB + RRA; L-16 fold pattern (`:97`).
- J-96..J-98 as a **BOUNDARY only** — zero symbols shared, zero files edited (`:98`).

**Produces:** `ConjectureNode`, `ExploreLaneToken`, status map, `ConjectureLifecycle` / Retrieved, UnblockCondition, L-10 novelty column, FD-1..FD-3 contract.

**Must not fork:** a second skeleton type; a stored status store; a Dung adjudicator; an embedding novelty metric; a planning-archive bridge; a code-lane retrieval trait that forces a common surface with J-96 (if a shared trait is the only way → sealed module + private constructor; if **that** still requires editing J-96 → ABORT-1) (`:40,108-109`).

**Parallel-safe with N-16** (shared field shape, disjoint files) and Q-track (`:11`).

**N-4 neighbor:** N-4 collapses Claim/Belief forks. L-24’s skeleton **is** the L-22/L-1 claim class, not a fourth claim type. Do not mint `ConjectureClaim`.

## 9. Config / flags / defaults

- **No `ELAI_*_ENABLE` on the store.** Opt-in by invoking the explore profile (`:151`).
- Leg B’s D-17 tick rides the **existing** `ELAI_REFUSAL_SLA_TICK_ENABLE` allowlist row — no new default-OFF flag (`:151`).
- Debug retrieval: `--include-quarantined` (`:51`).
- Decision doc: `workflow/reasoning/L-24-decisions.md` must record the status-fold truth table + the lane-wall mechanism (`:67-68`).
- No new dependency; blake3; no `.unwrap()` / `.expect()` on prod (`:110`).

## 10. Implementation map

| Symbol | Spec path |
|---|---|
| `.elai_cc/crates/orchestrator/src/reason/conjecture_store.rs` | node + token + fold + retrieval (`:85,174`) |
| `ssb.rs` | `+ConjectureLifecycle` or FORK-2 Kinds (`:86,175`) |
| L-23 pack / relapse seam | retrieve live+pending; refuted → gate (`:87,176`) |
| D-17/L-7 refusal emission | UnblockCondition composer + SLA clause (`:88,177`) |
| L-10 explore arm | NLI novelty column (`:89,178`) |
| I-30/I-31 feature-map clause | FD-1..FD-3 pre-gate (`:90`) |
| `crates/contracts/tests/l24_conjecture_store_contract.rs` | (a)–(i) (`:179,171`) |
| `workflow/reasoning/L-24-decisions.md` | required (`:180`) |

**Leak grep (load-bearing):** `rg -l "ConjectureNode|ExploreLaneToken" …/orchestrator/src/ \| rg -v "reason/"` **MUST be empty**. `git diff --stat` on case-graph files **MUST be empty** (`:76,131-133`).

## 11. Journal / SSB events

From `:47,114` and `L-plan/L-0_INDEX.md:146`:

```
ConjectureLifecycle {  // Pattern-2; net 0..+1; LIVE authoring baseline 581 / SCHEMA 272
  Kind: Registered | Criticized | Refuted | Suspended | Reinstated | Quarantined | Retrieved
}
Retrieved carries requesting lane + round id   // NON-NEGOTIABLE (EX-5 audit)

// FORK-2: reuse ExploreRoundCompleted Kinds (net 0); Retrieved still required

// Leg B: prefer-reuse D-16/D-17 RefusalDecision + UnblockCondition field (net 0)
```

Two-round fixture chain: `Registered(A,B,P) → Criticized → Refuted(P) → Suspended(A) → Retrieved(lane=explore, round 2 pack holds B live, A excluded)` (`:114-115`).

Do not rewind a live schema number. Re-read `ssb.rs:1530-1544` at edit (W-9).

## 12. Tests and observables

Named contract `l24_conjecture_store_contract.rs` (`:171`):

| Id | Assert |
|---|---|
| (a) | register → criticize → refute-premise ⇒ dependents **suspended**, not refuted |
| (b) | refuter’s evaluator refuted ⇒ target **reinstates** on next fold (N1) |
| (c) | never-criticized node is **pending**, never live (ATTACK-3) |
| (d) | fold byte-identical under `ELAI_SEED` (GD-2) |
| (e) | no path outside `reason/` can construct `ExploreLaneToken`; `lane!=explore` retrieval returns **zero** + journals nothing (EX-5) |
| (f) | quarantined absent from default, present under debug read, **never deleted** |
| (g) | refusal carries tried/missing/route from receipts; fabricated `tried` without receipt **unconstructible** (ATTACK-5) |
| (h) | novelty column: **zero** embedding/cosine symbols |
| (i) | corrupt journal row ⇒ fold fails closed, never a default status |

**Observable effect** (`:113-116`): two-round explore fixture as in §11; round-2 pack renders B live **with missing tests**; status map recomputes from the journal.

**Negatives that MUST FAIL:** code-lane retrieval of any `ConjectureNode`; never-criticized node reporting live; hand-set status surviving the fold; D-17 refusal missing `tried[]`/`missing[]`.

**INTEGRATION_SMOKE (R10, independent agent):** multi-round fixture (≥3 rivals, mid-run premise-refutation, one forced abstention): lifecycle chain in order; suspended dependent excluded from default pack, present under debug (`--include-quarantined=false` debug read as specified); refusal UnblockCondition `missing` names a runnable check; fold reproduces the map; same-session code-lane task journals **ZERO** `Retrieved` with `lane != explore`. Toy single-round happy-path does **not** count (`:118`).

**VERIFY (off-agent artifacts, never executor stdout)** (`:120-125`): nextest `l24_conjecture_store_contract`; SSB chain queried from db; refusal row + SLA violation row; L-10 novelty column present and byte-identical twice under `ELAI_SEED`.

## 13. Rebuild contract

Another harness has **this** organ iff all of the following are true:

- A middle tier **between** append-only journal and proven-only case graph. J-96 minting filter **untouched**.
- Conjecture object **is** the L-22 skeleton. `missing_tests` = unrun falsifiers. **No** second claim type.
- `ExploreLaneToken` type-level wall: unproven nodes structurally unretrievable on the code-fix/auditor path. `Retrieved` journals the requesting lane (EX-5).
- Status ∈ `{pending, live, refuted, suspended}` is a **pure journal fold** (no stored field). Live requires ≥1 executed criticism. Premise refute **suspends** dependents (`orphaned != false`). No absorbing status. Quarantine never deletes (EX-6).
- Refuted set feeds the **registration gate**, never the prompt (EX-1).
- Criticisms carry the **one** N-16 evaluator-id+version-hash shape; evaluator refutation suspends what it vouched for.
- UnblockCondition `{tried, missing, route}` composed from receipts; unblock-less refusal is an SLA violation.
- Novelty = L-4/F-12 NLI meaning clusters, zero cosine, CGG ON/OFF × retrieval ON/OFF, advisory.
- FD-1..FD-3 recorded as I-30/I-31 pre-gate; no exhaust builder shipped ahead of that consumer.
- Fold fails closed on corrupt journal. Byte-identical recompute under `ELAI_SEED`.
- No Dung graph, no FDL/ProbeUtility/planning-archive bridge.

A curated status database, a shared retrieval trait with the case graph, or “we’ll just tag the SSB row `explore` as a string” is a **cousin**, not this organ.

## 14. Non-goals / banned designs

- Edits to J-96..J-98 or the F-4 minting filter (ABORT-1) (`:79,109`).
- Stored / hand-curated status (ABORT-2 / EX-6) (`:80,106`).
- Retrieval of `ConjectureNode` into code-fix / auditor / case-graph packs (EX-5, SF-3) (`:105`).
- Deletion of refuted/quarantined nodes (SF-4) (`:106`).
- LLM-scored retrieval; model-authored status or UnblockCondition (TD-3) (`:107`).
- Refuted content in prompts (S3/EX-1) (`:108`).
- Embedding-cosine novelty (steal §3.3) (`:108`).
- Dung argument-graph adjudicator (REJECTED §4.3) (`:109`).
- Wire-dark I-30 exhaust builder ahead of D-026 (`:61`).
- FDL frontier controller / ProbeUtility / multi-cycle operator allocator / planning-harness bridge (`ELAI-SPEC-OVERVIEW.md:393-399`).
- Using DeepReason figures as ELAI floors (ID-5) (`:161`).

## 15. Source citations

Primary: `docs/thefinalglance/L-plan/L-24_conjecture_memory_middle_tier.md` (full file). Index: `L-plan/L-SUMMARY_INDEX.md:254-259`. Doctrine: `L-plan/L-0_INDEX.md:74-75,123-124,146`. Overview: `ELAI-SPEC-OVERVIEW.md:380-399`. Feature map: `ELAI-FEATURE-MAP.md:3656-3674,5117-5124`. Collab source: `workflow/handoff/deepreason-steal-and-collab.md`. Steal: `workflow/reasoning/steal-deepreason-2026-07-08.md` §1/§4. Neighbors in this pack: `L-22-claimant-falsifiers.md`, `L-23-explore-profile.md`. K-19 / J-96 cited as boundaries, not re-authored.

`ELAI-EXHAUSTIVE-SUPER-FEATURE-LIST.md` has **no** L-24 atom block. Do not mint L24-E### ids here.

## 16. Gaps

Label **UNCERTAIN**; do not fill:

- Numeric top-K and exact match keys beyond “skeleton scope/mechanism”; BM25 fallback is an executing-time decision (`:192`).
- Promotion floor above the seed `live = ≥1 criticism pass` (`:191`).
- Whether GRC receipts already journal failed/unavailable checks (RECON_NEEDED-3); if absent, additive failure receipts are in scope (`:77,56`).
- L-10 report JSON schema for the novelty column if L-10 is unauthored-in-code (RECON_NEEDED-4) (`:78`).
- `DeclaredTest` / `CriticismRef` rust field lists beyond the parenthetical in Task 1 (`:39`).
- `--include-quarantined=false` vs `true` wording in INTEGRATION_SMOKE vs Task 4 (`:51` vs `:118`) — which flag polarity shows the suspended dependent.
- Fold materialization trigger (when L-16 recompute-causes kicks in) (`:194`).
- Exact N-16 field names (`evaluator_version` vs `evaluator_id`) until FORK-3 grep (`:72`).
- Whether `Lane::Explore` also covers “RESEARCH” as a second constructible variant or is an alias (`:7` “EXPLORE/RESEARCH” vs `:39` “ONLY constructible lane”).
- Q-10 `SOURCE_ONLY` packet interface: overview forbids a planning-archive bridge into L-24 (`ELAI-SPEC-OVERVIEW.md:401-408`). Do not grow it here.

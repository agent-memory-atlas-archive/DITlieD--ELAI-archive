# Risk-as-curiosity targeted-effort router (L-6)

- **ELAI ids:** L-6 / L-T1 / NN-5 / HG-4 / WAVE 3
- **Spec status (as designed):** PLANNED (`L-plan/L-6_risk_as_curiosity_targeted_effort_router.md:3`). No live `claim/curiosity_router.rs` at architecture time (claim/ listing has extract/risk/roundtrip/semantic_entropy/counterfactual/prune/resolve only).
- **Crate / module (spec):** `crates/orchestrator/src/claim/curiosity_router.rs`; production caller is L-2 post-score `route_targeted_effort` (`L-plan/L-6_risk_as_curiosity_targeted_effort_router.md:36-42`).
- **Purpose of this file:** enough architecture to rebuild the targeted-effort router that turns ClaimRisk into prevention. Not a status ledger.

## 1. What it is

The **risk-as-curiosity router** is the paper’s mitigation half (§4.3): the predictor is used as a **curiosity reward** to spend effort where coverage is weak, not only to flag. A **High/Critical** L-2 `ClaimRisk` AUTO-FIRES **targeted** effort at **exactly** the weak claim — J-38 retrieval at that symbol, J-103 escalate-on-miss read, a targeted test/trace, or I-7 local→cloud — then **re-scores** via L-2. Bounded by J-84 (never a blanket loop). A claim that becomes grounded is **eligible** to mint a J-96 Verified Case Graph node (coverage-gap closure, no weights). L-6 **routes and re-scores**; it does **not** allow/block (L-7) (`L-plan/L-6_risk_as_curiosity_targeted_effort_router.md:5`, `L-plan/L-SUMMARY_INDEX.md:46-51`, `L-plan/_grounding/L-CGG-GROUNDING-MAP.md:89-90`).

## 2. Why it exists

A gate that only flags is a friction tax. HG-4 makes routing **non-optional**: an unverifiable high-risk claim must go to verification or abstention (L-7), never a silent allow (`L-plan/L-6_risk_as_curiosity_targeted_effort_router.md:5,13,47`). The paper’s 50 targeted trajectories ≈ human-oracle collection (§5.1) is **[EXTERNAL/DESIGN-RATIONALE]**, not an ELAI N (`:13,92`, `L-plan/L-0_INDEX.md:168`).

If this organ is missing, High risk is a label with no close-the-gap path (L-7 then only abstains). If it is faked (blanket re-read, unbounded retry, or model-driven navigation of the repair), cost explodes and L-15 R3’s measured failure returns: model-nav plateaued where deterministic hops did not (`L-plan/L-15_repair_controller_and_evidence_first_reorder.md:119-124`).

L-15’s RepairController **closes the loop** L-6 opens (S3 remedy → S4 re-verify → S5 J-83 gate). L-6 remains the **router**; L-15 is the named iterate object. Do not merge them (`L-plan/L-15_repair_controller_and_evidence_first_reorder.md:45-71`).

## 3. How it works (runtime)

1. **Who calls it.** Orchestrator, L-2 post-score seam, `route_targeted_effort`. Not contracts-only (R7) (`L-plan/L-6_risk_as_curiosity_targeted_effort_router.md:41-43`).
2. **Input.** One `ClaimRisk` (tier + which predictors fired) plus the L-1 claim (kind + unresolved refs). J-84 budget remaining. **No LLM in the map** (TD-3).
3. **Risk→action map (deterministic, targeted only)** (`:29`):

| Signal | Action (compose, never reimplement) |
|---|---|
| High/Critical + refs-unresolved File/Symbol | J-38 retrieval **at that symbol**; B-29 locates it |
| invented Dependency | API_REGISTRY lookup |
| high P3 on Behavior | targeted test/trace |
| unresolvable locally | I-7 `EscalationCascade` |
| hallucinated identifier miss | J-103 escalate-on-miss fuller read |
| P1 residual / unsupported (L-15 R3) | **deterministic** retrieval: BM25 / HippoRAG PPR + atomic entity read. **Never** model-driven free navigation |
| no cheap targeted action for this kind | **hand to L-7 abstain**, do not spin (`:97`) |

4. **L-15 R3 (binding stub_ban).** Repair-retrieval MUST be deterministic graph-hop. The model may do the atomic entity read only. A model-driven repair re-injects the failure (`L-plan/L-15_repair_controller_and_evidence_first_reorder.md:119-124`).
5. **Re-score loop (bounded).** After evidence lands, re-invoke L-2. Cap **rounds + cost** with J-84. Curiosity is targeted, never unbounded (`:30,46`).
6. **Case-mint hook (observe-only).** If the re-score is grounded, the claim is **eligible** for J-96 mint; actual mint stays J-96’s gated path (`:31`).
7. **Commit.** Prefer-reuse `RetrievalLoopEvent` (J-38) / I-7 escalation rows. Net 0 (`:53`, `L-plan/L-0_INDEX.md:135`). Default **OBSERVE-only** (record routed actions; L-7 blocks) (`:32,69`).
8. **Abstain / fail-closed.** Low-risk → **zero** targeted effort (`:54`). High/Critical unverifiable → route or hand to L-7, **never silent allow** (HG-4) (`:47`). Budget exhaust → stop and hand to L-7, do not continue.
9. **The model may generate** none of the route. It may read the one entity J-38/HippoRAG named. It may **not** pick the sink, raise the J-84 cap, or blanket re-plan.

Do not confuse with **J-73 `should_retrieve`**: that gate **skips** the cascade when local context suffices. L-6 **forces** a targeted retrieve on High unresolved refs. Opposite direction; both can exist (`J-73-retrieval-abstention-gate.md` §1).

## 4. Types, enums, objects

The L-6 file does not freeze a Rust struct block. Rebuild from the wiring contract (`:36-37,29-32`):

```
fn route_targeted_effort(claim, risk: &ClaimRisk, budget: J84Budget) -> RoutedEffort

RoutedEffort {                 // name UNCERTAIN; fields implied:
  action,                      // J-38 | J-103 | API_REGISTRY | test/trace | I-7 | hand-L-7
  target: ClaimRef,            // the weak symbol/pkg — never “the whole repo”
  rounds_used,
  rescore: Option<ClaimRisk>,
  handed_to_l7: bool,
}
```

**Consumed:** J-38 `RetrievalLoopEvent` Kind `{MidTaskSearchInvoked, GraphNeighborExpanded, ContextSufficiencyChecked, RetrievalRoundCapped}` (J-38 plan; prefer-reuse vs QueryTrace/J-18 at their edit). I-7 `EscalationCascade`. B-29 impact API. J-84 classifier. J-96 mint eligibility only.

**J-84:** `{Zero, Depth, Width, EscalateOrAbstain}` over H-42/I-26 (`J-84-grcv3-effort-classifier.md` §1). L-6 uses it as **max rounds + cost cap**. Exact mapping of those tiers onto curiosity rounds is **UNCERTAIN** (§16).

Paper 50-trajectory figure is **not** `max_rounds`.

## 5. Control flow / state machine

**Owner:** orchestrator router after L-2. Observe-only until L-7.

```
score_claim_risk → ClaimRisk
  Low/Medium (not High/Critical) → ZERO effort; stop
  High/Critical → map(kind, fired-predictors) → one targeted sink
       R3: retrieval sink = HippoRAG/BM25/J-38, not model-nav
  run sink (compose)
  evidence landed? → L-2 re-score
       still High/Critical and budget remains → next targeted remedy (L-15 S3) or stop
       grounded → J-96 eligible (observe)
       unverifiable or budget out → hand L-7
```

**Forced:** one sink per round (targeted, not blanket). `rg blanket|reread_all|full_replan` MUST be 0 (`:60`). Parallel-safe with L-7/L-8 files (`:9`).

Autonomous token ceiling: J-84 cap **and** B-22 envelope must agree so curiosity does not blow the autonomous budget (`:98`).

## 6. Grounding, receipts, gates

L-6 does not verify. It **spends** to mint a receipt (P1 round-trip after retrieve, or L-5 do-test, or registry). HG-4: missing evidence after bounded effort → L-7, not allow.

**Never allowed as a router:** LLM “search more”, uniform extra CoT, paper 50 as round cap, J-73 skip as a reason to ignore High unresolved refs, model-driven repair nav (R3).

**Selector:** ClaimRisk High/Critical from L-2 (already HG-2 fused). L-6 must not raise a Low claim to High by routing.

## 7. Budgets, limits, ceilings

| Knob | Value | Scope |
|---|---|---|
| router | ~70 LOC | `curiosity_router.rs` |
| re-score loop | ~40 LOC | same |
| mint hook | ~20 LOC | same |
| file | ~130 LOC | (`:86`) |
| seam | ~20 LOC | `risk.rs` post-score |
| contract | ~80 LOC | (`:83`) |
| loop bound | J-84 max rounds + cost | per claim |
| Low-risk effort | **0** | cost guard |
| SSB | reuse RetrievalLoopEvent / escalation; net 0 | W-9 |
| observe-only | record routes, do not block | GD-4 / W-2 |
| paper 50 trajectories | [EXTERNAL] | not a constant |

## 8. Roles and neighbors

Role-agnostic. I-7 escalate may change **model family**; L-6 does not pick the model (I-7 does).

**Consumes:** L-2 ClaimRisk; J-38, J-103, I-7, B-29, J-84, J-96 (`:39-40`).

**Produces:** routed actions + post-effort ClaimRisk; reused retrieval/escalation rows; J-96 eligibility bit.

**Must not fork:** a second retrieval loop; unbounded while-High; L-7 allow/block inside this file.

**Frontend:** retrieval/escalation surface or justified N/A (`:49-50`).

**L-15:** S2 High/Critical **enter repair** (not abstain-first); S3 branches on fired predictor × kind; S4 re-invokes L-2 (`L-plan/L-15_repair_controller_and_evidence_first_reorder.md:51-66`). Rebuild L-6’s map so L-15 can call it as S3.

## 9. Config / flags / defaults

- No behavior block. Observe-only; L-7 blocks (`:73`).
- Decision doc: `workflow/reasoning/L-6-decisions.md` (full kind→action map) (`:34`). **Absent** at architecture time.
- J-38/J-103/J-84/J-96 may still be PLANNED — compose their interfaces; do not rewrite them (`:16-21`).
- No `.unwrap()` on prod (`:48`).

## 10. Implementation map

| Symbol | Spec path |
|---|---|
| `crates/orchestrator/src/claim/curiosity_router.rs` | map + bounded re-score + mint hook (`:37`) |
| `route_targeted_effort` | production name (`:42`) |
| `claim/risk.rs` post-score | ~20 LOC (`:87`) |
| J-38 / J-103 / I-7 / B-29 / J-84 / J-96 | consumed (`:39`) |
| `l6_curiosity_router_contract.rs` | tests (a)–(f) (`:83`) |
| HippoRAG `hippo_rag.rs` | L-15 R3 retrieval (`L-15 … :119`) |

Live: **no** `curiosity_router.rs`, **no** `route_targeted_effort` symbol.

## 11. Journal / SSB events

Prefer-reuse (`:53`, `L-plan/L-0_INDEX.md:135`):

```
RetrievalLoopEvent     // J-38; Kind MidTaskSearchInvoked / GraphNeighborExpanded /
                       // ContextSufficiencyChecked / RetrievalRoundCapped
I-7 escalation rows
```

Net 0. GD-2: routed-action decision recomputable from ClaimRisk under `ELAI_SEED` (`:80`). Payload of “which symbol was targeted” must be in the reused row or the loop is not auditable — exact field **UNCERTAIN** (§16).

## 12. Tests and observables

Named (`:83`):

| Id | Assert |
|---|---|
| (a) | High-risk unresolved-symbol → targeted J-38 **at that symbol** |
| (b) | re-score after evidence **lowers** the tier |
| (c) | Low-risk → **zero** effort |
| (d) | J-84 caps rounds |
| (e) | unverifiable High never silently allowed (handed to L-7) |
| (f) | routed action recomputable from ClaimRisk |
| R3 | model-driven nav path is absent / banned |

**observable_effect** (`:51-54`, `L-plan/L-SUMMARY_INDEX.md:51`): High-risk file-claim with unresolved refs auto-fires J-38 at that symbol and re-scores Lower. Negative: Low-risk fires zero targeted effort.

**W-1:** live High-risk auto-fires targeted retrieval and re-scores lower (`:73`).

Adversarial: blanket re-read; unbounded while-High; Low claim retrieval; silent allow; paper 50 as cap.

## 13. Rebuild contract

Another harness has **this** organ iff all of the following are true:

- High/Critical auto-routes **one** targeted sink from the §3 table; Low fires **zero**.
- Retrieval is J-38/J-103/HippoRAG/BM25, located by B-29, **not** model-nav (L-15 R3).
- Re-score is L-2, loop bounded by J-84 rounds+cost.
- Unverifiable High/Critical handed to L-7 (HG-4), never silent allow.
- J-96 is eligibility-only; mint stays J-96.
- Observe-only until L-7. No allow/block here.
- SSB prefer-reuse retrieval/escalation rows.
- Tests (a)–(f)+R3. Production `route_targeted_effort` after L-2.

A “search more when unsure” prompt, a blanket re-plan, or J-73 skip-logic inverted into this file is a **cousin**, not this organ.

## 14. Non-goals / banned designs

- Final allow/block/abstain (L-7) (`:5`).
- Re-implementing J-38/J-103/I-7/J-96 (GD-1 P0) (`:48`).
- Blanket re-read / full re-plan (`:45`).
- Unbounded loop (`:46`).
- Silent allow of unverifiable High (HG-4).
- Model-driven repair navigation (L-15 R3).
- Paper 50-trajectory N as `max_rounds` (ID-5).
- Merging L-15 RepairController state machine into this file (neighbor).
- Using J-73 `should_retrieve` skip to drop High unresolved work.

## 15. Source citations

Primary: `docs/thefinalglance/L-plan/L-6_risk_as_curiosity_targeted_effort_router.md` (full file). Index: `L-plan/L-SUMMARY_INDEX.md:46-51`. Doctrine: `L-plan/L-0_INDEX.md:53,97-98,135`. Grounding map: `L-plan/_grounding/L-CGG-GROUNDING-MAP.md:51-52,89-90,109-111`. L-15 R3 + loop: `L-plan/L-15_repair_controller_and_evidence_first_reorder.md:45-71,119-124`. Neighbors: `J-84-grcv3-effort-classifier.md`, `J-73-retrieval-abstention-gate.md` (do not merge). Paper: arXiv:2606.27326 §4.3 / §5.1 [EXTERNAL/DESIGN-RATIONALE].

## 16. Gaps

Label **UNCERTAIN**; do not fill:

- Full ClaimKind→action map (`:97`).
- `RoutedEffort` type name and fields.
- J-84 tier → max rounds / cost numbers.
- How B-22 autonomous envelope and J-84 interact when they disagree (`:98`).
- Whether Medium ever routes (plan says High/Critical auto-fire; Medium is L-7 caveat).
- P3→test/trace: which test command (D-11? F-4?).
- RetrievalLoopEvent vs QueryTrace/J-18 reuse (J-38 fork).
- Frontend tile vs N/A.
- Whether L-15 S3 **replaces** L-6’s loop or **calls** it.

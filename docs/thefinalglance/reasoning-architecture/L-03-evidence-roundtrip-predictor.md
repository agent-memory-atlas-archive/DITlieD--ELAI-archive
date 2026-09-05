# Evidence round-trip predictor P1 (L-3)

- **ELAI ids:** L-3 / L-T1 / P1 / TS-1 / TS-2 / WAVE 2
- **Spec status (as designed):** PLANNED (`L-plan/L-3_evidence_round_trip_predictor.md:3`). Live `claim/roundtrip.rs` is a **cousin**: residual is mostly unresolved-ref fraction plus H-53 ghost-citation share, not `1 − reconstructing/applicable` over D-10 FILE/SOURCE/TEST/CosmosCheck.
- **Crate / module (spec):** `crates/orchestrator/src/claim/roundtrip.rs`; production caller is L-2 `score_claim_risk` cost-gate → `round_trip_residual` (`L-plan/L-3_evidence_round_trip_predictor.md:33-41`).
- **Purpose of this file:** enough architecture to rebuild the cheap P1 reconstruction predictor. Not a status ledger.

## 1. What it is

The **evidence round-trip predictor (P1)** is the LLM analogue of the paper’s tokenizer round-trip residual: a predicted latent whose decoded frame does not survive re-encoding signals hallucination (§4.2). Here, a typed claim that cannot be **reconstructed / supported from evidence** scores **high residual**. L-3 runs each L-1 claim through the **shipped** grounding methods (D-10/D-11 FILE/SOURCE/TEST/CosmosCheck) plus H-53 closed-set citation membership plus H-32 DKG validate-citation, and emits `PredictorSignal{RoundTrip, residual}` into L-2. This is **activation** of methods that already run on the agent’s own grounding-sweep, pointed at **any** model’s ClaimSet (HG-3). It does **not** re-implement those methods (`L-plan/L-3_evidence_round_trip_predictor.md:5`, `L-plan/L-SUMMARY_INDEX.md:25-30`, `L-plan/_grounding/L-CGG-GROUNDING-MAP.md:21-27`).

## 2. Why it exists

Most coding-agent hallucinations are invented files, symbols, packages, signatures, and fake citations. All of those fail reconstruction against the repo (`L-plan/L-3_evidence_round_trip_predictor.md:13`). ELAI’s grounding verification is already binary pass/fail per method (MiniCheck-style). H-53 makes a ghost citation structurally impossible. If this organ is missing, L-2’s cheap gate has only L-1 `refs-unresolved` (existence), not “does the cited line still mean this claim”. If it is faked (LLM paraphrase of the claim as “round-trip”, or a new FILE checker inside `roundtrip.rs`), HD-2 and GD-1 both fail.

P1 is the **cheap** signal: it runs first; L-4/L-5 fire only when L-3 already flags medium residual (`L-plan/L-3_evidence_round_trip_predictor.md:13`, `L-plan/L-2_claimrisk_score_convergence_fuse.md:31`). L-15 R2: land L-3 **before** L-4 because evidence-grounded predictors are what break the resampling ceiling (`L-plan/L-15_repair_controller_and_evidence_first_reorder.md:112-117`).

## 3. How it works (runtime)

1. **Who calls it.** L-2 cost-gate, production name `round_trip_residual`. Not a contracts-only caller (R7) (`L-plan/L-3_evidence_round_trip_predictor.md:39-41`).
2. **Input.** One L-1 claim from a `ClaimSet`, plus the frozen H-53 `EvidenceAnchorSet` for this generation, plus DKG / D-11 registry handles. No model internals (HG-3) (`:5,63`).
3. **Per-kind dispatch (deterministic, GD-1).** Map `ClaimKind` to shipped methods and **call** them; do not reimplement (`:27`):

| ClaimKind | Method(s) |
|---|---|
| File | FILE |
| Symbol | DKG / SOURCE |
| Dependency | API_REGISTRY |
| Api | SOURCE / compile |
| Behavior | TEST |
| Reachability | DKG call-graph |
| Causal | **deferred to L-5** (do not call a static import-edge “causal round-trip”) |

4. **Closed-set blend (deterministic, H-53).** Citation membership against the pre-bound `EvidenceAnchorSet`. Out-of-set → **max residual** and the `[N]` cannot be emitted as supported (`:28,52`, `L-plan/_grounding/L-CGG-GROUNDING-MAP.md:25-27`). Out-of-set also routes toward D-16 refusal (H-53), not a silent allow.
5. **H-32.** `validate-citation` against DKG before use; consume `FactValidationFailed` rather than a new validator (`:18,37`).
6. **Residual (deterministic).**

```
residual = 1 - (reconstructing_methods / applicable_methods)
```

Blended with H-53: out-of-closed-set = max residual (`:28`). Zero applicable methods (pure opinion) must score **neutral**, not falsely low or high — decide at authoring (`:95`).
7. **Emit.** `PredictorSignal { source: RoundTrip, value: residual, fired }` into L-2. `fired` threshold is ELAI-measured (L-10 / L-3-decisions), not a paper constant.
8. **Commit.** Prefer-reuse D-10 grounding-verification rows (FILE/SOURCE/CosmosCheck). Net 0 SSB families expected (`:51`, `L-plan/L-0_INDEX.md:132`).
9. **Abstain / fail-closed.** No model judge. No generative paraphrase (HD-2). WEB/CosmosCheck legs stay on the cost-gated path (medium cheap-risk only) and are fixture-covered in L-10 so the bench stays reproducible (`:94`). Causal claims do not get a fake FILE-leg “causal pass”.
10. **The model may generate** the claim text and `[N]` citation numbers. It may **not** decide reconstruction, expand the closed set after generation, or rewrite residual.

Live cousin (do not rebuild as this organ): `RoundTripPredictor::roundtrip_residual` = `unresolved_refs / refs.len()` (`claim/roundtrip.rs:37-43`) — L-1 existence ratio, not method reconstruction. `ClosedSetCitationPredictor` **does** call H-53 `check_citation_in_set` (`roundtrip.rs:128-164`). `checker_id_for_kind` only returns ids for `Api` (`compile-rust`) and `Dependency` (`registry-check`); File/Symbol/Behavior/Causal/Reachability are `None` (`roundtrip.rs:71-77`). Residual fire seeds `0.5` / `0.25` (`roundtrip.rs:16-18`) are unmeasured. Empty refs → residual `0.0` (`roundtrip.rs:38-40`), which is one fill of the “neutral” open question — **not** specified as `0.0`.

## 4. Types, enums, objects

L-3 does not invent a new risk object. It **emits** L-2’s `PredictorSignal` (`L-plan/L-3_evidence_round_trip_predictor.md:28,38`).

```
fn round_trip_residual(claim: &Claim, ctx: &RoundTripCtx) -> PredictorSignal
  // source = PredictorKind::RoundTrip
  // value  = residual in [0, 1] (implied by the formula; bounds not named)
  // fired  = residual at/above the ELAI-measured cheap-medium trigger

RoundTripCtx {                 // not named in the plan file; required handles:
  checker_registry,            // D-11
  evidence_anchors,            // H-53 EvidenceAnchorSet
  dkg,                         // H-32 + symbol graph
}
```

**Per-method results** (for GD-2 recompute): binary pass/fail per applicable method, MiniCheck-style (`:13`). Exact struct **UNCERTAIN** (§16); rebuild must **record** per-method pass/fail so residual recomputes (`:77, f`).

**Consumed, not redefined:**

- D-10 `select_grounding_mode` / D-11 `CheckerResult { Pass, Fail, Inapplicable }` (`D-11-constructive-checker-registry.md` §4).
- H-53 `EvidenceAnchorSet`, `check_citation_in_set`, outcomes Bound / Ghost / Stale (live names in `roundtrip.rs:8-11,141`).
- H-32 validate-citation / `FactValidationFailed`.
- L-1 `ClaimKind` / `ClaimSet`. Live D-10 `ClaimType` map in `claim_type_for_kind` (`roundtrip.rs:60-66`) is a **cousin adapter** so checkers.toml rows apply; it is not an L-3 type and must not replace `ClaimKind`.

**Causal:** no P1 “causal reconstructed” flag. HG-1 stays on L-5.

## 5. Control flow / state machine

**Owner:** orchestrator P1, invoked from L-2’s cheap layer. No GRC states.

```
ClaimSet claim
  → applicable methods for kind (table §3)
  → if kind == Causal: skip P1 reconstruction; L-5 owns it
  → run each applicable shipped checker (D-10/D-11)
  → H-53 membership on citations
  → H-32 DKG validate-citation
  → residual = 1 - reconstructing/applicable ; out-of-set → max
  → PredictorSignal::RoundTrip
  → L-2 fuse (cheap; may trigger L-4/L-5)
```

**Forced:**

- Exact reconstruction, never paraphrase (`:45`).
- Zero model calls (`:45, d`).
- Parallel-safe with L-4/L-5 file-wise (`:9`); L-15 R2 still lands this **before** L-4 in the wave.
- No L-3 enable-flag (`:71`).

## 6. Grounding, receipts, gates

P1 **is** a reconstruction receipt. Pass of FILE/SOURCE/TEST/CosmosCheck is the evidence that the claim survived re-grounding. H-53 Bound is membership, not truth of the sentence.

**Selector hierarchy:**

1. H-53 Ghost/Stale → max residual (cannot emit as supported).
2. Shipped checker Fail → that method does not reconstruct.
3. Checker Inapplicable → drop from **applicable** denominator (do not count as fail or pass) — **UNCERTAIN** vs skip the claim (§16).
4. LLM “I cited it correctly” → never.

**Never allowed:** generative restatement as round-trip (HD-2), model-judged reconstruction (TD-3), a new FILE walker inside `roundtrip.rs` that bypasses D-10/D-11 (GD-1 P0), calling a File/import-edge reconstruction “causal” (HG-1).

## 7. Budgets, limits, ceilings

| Knob | Value | Scope |
|---|---|---|
| dispatcher | ~70 LOC | `roundtrip.rs` |
| residual | ~40 LOC | same |
| wire into L-2 | ~20 LOC | `risk.rs` |
| file total | ~110 LOC | (`:83`) |
| contract | ~70 LOC | `l3_round_trip_contract.rs` |
| FILE/SYMBOL legs | in-process, local | bulk (`:7`) |
| CosmosCheck/WEB | network; cost-gated to medium+ cheap-risk | L-10 fixture covers (`:7,74,94`) |
| SSB | prefer-reuse D-10 grounding-verification; net 0 | W-9 |
| live fire seeds | `DEFAULT_RESIDUAL_FIRES_AT=0.5`, `CHECKED_KIND_CITATION_FIRES_AT=0.25` (`roundtrip.rs:16-18`) | **not** spec; measure at L-10 |

No GRC call budget. Unbounded methods / empty denominator: open question (`:95`).

## 8. Roles and neighbors

Role-agnostic, any B-37 backend (HG-3).

**Consumes:** D-10 `select_grounding_mode` + D-11 dispatch; H-53 `citation_prebind`; H-32 validate-citation; DKG; L-1 ClaimSet; L-2 `PredictorSignal` (`:35-38`).

**Produces:** P1 `PredictorSignal` into L-2; reused D-10 grounding-verification rows.

**Must not fork:** a second FILE/TEST checker; LLM paraphrase round-trip; a standalone P1 UI.

**Frontend:** N/A — feeds L-2’s risk tile (`:47-48`).

## 9. Config / flags / defaults

- No predictor flag. Gate flag is L-7 (`:71`).
- WEB/CosmosCheck only after cheap medium residual (`:94`).
- Decision doc: `workflow/reasoning/L-3-decisions.md` (neutral residual + fire threshold) (`:30`). **Absent** at architecture time.
- D-11 live remains DARK behind `ELAI_CHECKER_ENABLE` (`D-11-constructive-checker-registry.md` header). L-3 **activates** those methods over ClaimSet; it does not invent a second enable flag. Behavior when the registry is empty: **UNCERTAIN** (§16) — fail-closed residual vs Inapplicable-neutral.
- blake3; no `.unwrap()` on prod (`:46`).

## 10. Implementation map

| Symbol | Spec path |
|---|---|
| `crates/orchestrator/src/claim/roundtrip.rs` | dispatcher + residual + P1 signal (`:34`) |
| `round_trip_residual` | production name (`:40`) |
| `grounding_mode.rs:157` | D-10 pin (live Constructive branch; cousin enum) |
| `checker_registry.rs` | D-11 |
| `citation_prebind.rs` | H-53 |
| H-32 validate-citation | DKG fact memory |
| `claim/risk.rs` | ~20 LOC wire (`:84`) |
| `crates/contracts/tests/l3_round_trip_contract.rs` | plan name; live file is `l3_roundtrip_predictor_contract.rs` |

**Live neighbors:** `score_claim_risk_wired` (`roundtrip.rs:167-177`) injects `RoundTripPredictor` + `ClosedSetCitationPredictor` into L-2; `closed_set_from_resolved_refs` freezes resolved refs as anchors (`roundtrip.rs:109-126`). Rebuild must still **call** FILE/SOURCE/TEST, not stop at ref-count.

## 11. Journal / SSB events

Plan: **prefer-reuse** D-10 grounding-verification rows (FILE/SOURCE/CosmosCheck). Net 0 (`L-plan/L-3_evidence_round_trip_predictor.md:51`, `L-plan/L-0_INDEX.md:132`). H-32 may already emit `FactValidationFailed` — consume, do not duplicate.

No new `RoundTripMeasured` family in the L-3 file. Residual must be recomputable from **per-method pass/fail** under `ELAI_SEED` (`:77`). Where those bits live if D-10 rows are reused: **UNCERTAIN** (§16).

## 12. Tests and observables

Named (`:80`):

| Id | Assert |
|---|---|
| (a) | claim citing a real symbol/line → **low** residual |
| (b) | nonexistent symbol → **high** residual |
| (c) | out-of-closed-set citation rejected (H-53) |
| (d) | no model call |
| (e) | invented dependency fails API_REGISTRY |
| (f) | residual recomputable from recorded per-method results |

Wiring (`:55-59`): `rg residual|round_trip|RoundTrip`; `rg -ci generate|paraphrase|summari` MUST be 0; L-3-decisions.md exists.

**observable_effect** (`:49-52`, `L-plan/L-SUMMARY_INDEX.md:30`): `src/auth/session.ts:84` with symbol present → round-trip PASS (low residual); nonexistent symbol → FAIL (high residual) and `[N]` rejected by the closed set. Negative: out-of-evidence citation cannot be emitted as supported.

**W-1:** live cost-gate emits a P1 residual that flips a fake-symbol citation to refusal (`:70`).

Adversarial: paraphrase-as-round-trip; empty-method claim scored as fully reconstructed; Causal marked reconstructed from FILE; Ghost citation Bound.

## 13. Rebuild contract

Another harness has **this** organ iff all of the following are true:

- P1 residual formula is `1 − reconstructing/applicable` (or documented equivalent) over **shipped** D-10/D-11 methods, not a new checker.
- Per-kind dispatch table in §3, with Causal deferred to L-5.
- H-53 out-of-set → max residual; ghost `[N]` not “supported”.
- H-32 / DKG consumed for symbol/citation validation.
- Exact reconstruction only (HD-2); `rg` of generate/paraphrase/summarize in `roundtrip.rs` is 0.
- No model judge (TD-3). No model internals required (HG-3).
- Cheap-first: this signal is the medium-risk trigger for L-4/L-5.
- SSB prefer-reuse grounding-verification rows.
- Tests (a)–(f). Production path is L-2 cost-gate, not contracts-only.

An unresolved-ref ratio, an LLM “does this citation look real”, or live `unresolved/len` **without** FILE/SOURCE/TEST/CosmosCheck reconstruction is a **cousin**, not this organ.

## 14. Non-goals / banned designs

- Re-implementing FILE/SOURCE/TEST/WEB/API/API_REGISTRY/CosmosCheck/VisualArtifactCheck or H-53 (`:43`).
- Generative paraphrase round-trip (HD-2) (`:45`).
- Model-judged reconstruction (TD-3).
- Scoring Causal as reconstructed via static/import-edge (HG-1 / L-5).
- A standalone P1 dashboard (`:47-48`).
- New SSB family unless W-9 reuse is impossible (plan expects net 0).
- Running WEB/CosmosCheck on every Low claim (`:94`).
- Using P1 residual as the **action** (L-7) or as a lone High/Critical block (HG-2 — L-2 fuse).

## 15. Source citations

Primary: `docs/thefinalglance/L-plan/L-3_evidence_round_trip_predictor.md` (full file). Index: `L-plan/L-SUMMARY_INDEX.md:25-30`. Doctrine: `L-plan/L-0_INDEX.md:47,91-92,132,153`. Grounding map: `L-plan/_grounding/L-CGG-GROUNDING-MAP.md:21-27`. L-15 R2: `L-plan/L-15_repair_controller_and_evidence_first_reorder.md:112-117`. Neighbors: `D-10-mode-routing-predicate.md`, `D-11-constructive-checker-registry.md`, `L-02-claimrisk-convergence-fuse.md`. Live: `claim/roundtrip.rs`, `l3_roundtrip_predictor_contract.rs`, `citation_prebind.rs`. Paper: arXiv:2606.27326 §4.2 predictor 1 [EXTERNAL/DESIGN-RATIONALE].

## 16. Gaps

Label **UNCERTAIN**; do not fill:

- Neutral residual when applicable-methods = 0 (`L-plan/L-3_evidence_round_trip_predictor.md:95`). Live uses `0.0` for empty refs — not blessed.
- Whether D-11 `Inapplicable` shrinks the denominator or ignores the claim.
- Numeric `fired` threshold (live 0.5/0.25 unmeasured).
- How H-53 “max residual” blends with a partial method-pass (max of the two vs overwrite).
- WEB/API/VisualArtifactCheck vs the L-1 kind table (plan’s dispatcher table does not name WEB/Visual).
- Behavior→TEST: which test command, timeout, sandbox — deferred to D-11/D-18.
- Per-method result struct and which D-10 row fields store them for GD-2.
- Empty checker registry / `ELAI_CHECKER_ENABLE` off: skip vs max residual vs L-1-only ratio.
- `round_trip_residual` vs live `score_claim_risk_wired` naming.
- Plan contract filename `l3_round_trip_contract.rs` vs live `l3_roundtrip_predictor_contract.rs`.
- Whether H-32 28d TTL miss is residual-high or Inapplicable.

# Retrieval-abstention gate (`should_retrieve`)

- **ELAI ids:** J-73 / WAVE CR-2 / C2
- **Spec status (as designed):** PLANNED (`ELAI-FEATURE-MAP.md:193-200`). J-SUMMARY: EXTEND, effort M, depends none (`J-plan/J-SUMMARY_INDEX.md:1709-1716`). Tier-1 in-process ships observe-only first (GD-4); sidecar special-token head is Tier-2 RunPod-gated.
- **Crate / module (spec):** `crates/knowledge/src/pipeline.rs` (`should_retrieve` before the cascade); per-task-type T near `query_class`; signal from `crates/orchestrator/src/abstention_score.rs` + `conformal_calibration.rs`
- **Purpose of this file:** enough architecture to rebuild the pre-cascade retrieval skip — calibrated score vs config T, conservative retrieve-on-uncertain, journaled. Not a status ledger.

## 1. What it is

The **retrieval-abstention gate** is a deterministic `should_retrieve` check in `pipeline.rs` **before** the BM25 / ONNX / PageRank / CATRAG / A-RAG cascade fires. The decision signal is a calibrated abstention score compared to a per-task-type threshold T that lives near the existing FAST / STANDARD / DEEP `query_class` tier router. Default path **reuses** shipped answer-abstention machinery (`abstention_score.rs` + `conformal_calibration.rs`: `Abstain` enum, `decide_low_score_abstains`, `agent_pool` `low_score_routes_to_abstain`). The LLM does not adjudicate “should I retrieve” (TD-3). A gated-off retrieval is journaled (`J-plan/J-73_retrieval_abstention_gate.md:1-9,29-31`, `ELAI-FEATURE-MAP.md:193-200`).

## 2. Why it exists

ELAI **always** retrieves when it is in a retrieval tier. A 2026-06-23 codegraph search found **no** `should_retrieve` symbol (`J-plan/J-73_retrieval_abstention_gate.md:5,53`). When local context already suffices, retrieval is pure cost. Repoformer (arXiv:2403.10059) shows a self-assessment P(retrieve) threshold (paper T=0.15 function / 0.20 line-API) can yield >70% dense-setting speedup at >80% abstention precision — **DESIGN-rationale only**; ELAI calibrates its own T (ID-5) (`:51-52`, `J-plan/J-SUMMARY_INDEX.md:1711`). If this organ is missing, every retrieval-tier query pays the full cascade. If it is faked (the model decides skip, or skip-on-uncertain), needed retrieval silently disappears (R1 fail-closed: uncertain **retrieves**) (`:40-41,49`). This gate is **whether to enter** the cascade. It does **not** prune stages (that is J-75, user-gated) (`:10,43`).

## 3. How it works (runtime)

1. **Who calls it.** Production retrieval: a real `elai` query / task path entering `crates/knowledge/src/pipeline.rs`. Not a contracts-only caller (R7) (`J-plan/J-73_retrieval_abstention_gate.md:36-38`).
2. **Input.** The query, its `query_class` task type (function vs line/API, and the FAST/STANDARD/DEEP tier the threshold home attaches to), the calibrated abstention signal, and the per-task-type T (`:8,15-16,31-35`).
3. **Signal (reused, not re-authored).** Read the shipped abstention score / conformal calibration. Compose D-16 conformal τ machinery for calibration (IC-13 one-calibration-impl). Do **not** train a new retrieval-decision head in Tier-1 (`:8,13,33-35,42`).
4. **Compare (deterministic).** `score` vs T. The LLM does not sit in this branch (`:9,40`).
5. **Observe-only first (GD-4).** Until the false-positive (skipped-but-needed) rate is bounded on **≥3 real runs**, journal the **would-skip** and still run the cascade. Then enforce (`:19,41,49`).
6. **Enforce skip (after GD-4).** If the calibrated score **clears** T and local context suffices: **do not** fire bm25/vector/rerank work; answer from local context; journal `GatedSkip` with the score + the T it cleared (`:47-49`).
7. **Conservative proceed.** Score **below** T, missing signal, or uncertain → **full cascade**. Abstention is conservative: uncertain retrieves, never silently skips (R1) (`:49`, `J-plan/J-SUMMARY_INDEX.md:1716`).
8. **Override journal.** Kind `AbstainOverride` is named for the SSB enum (`:25`). When it fires (manual/config override vs observe-mode would-skip) is UNCERTAIN (§16).
9. **Commit.** Prefer-reuse: if `QueryTrace` (9-stage retrieval trace, phase-1I) already has a decision-stage field, add the abstain verdict there (net 0). Else `+RetrievalAbstained` Pattern-2, Kind `GatedSkip / GatedProceed / AbstainOverride`, baseline+1 from LIVE ~269/578 (post-J-66; re-read W-9). Decide at edit (`:25-26,48`).
10. **Tier-2 (deferred).** Sidecar `<cc>/abstain` special-token head on the Qwen2.5-Coder lane (Repoformer multi-task-head shape). Needs sidecar token support, remote-GPU-gated, `pod_stop` after. Cheap calibrated path ships **regardless** (`:19-20,62`).
11. **Fail-closed.** Enforced skip before FP is bounded = ban. LLM adjudication = ban. New abstention calibration impl = ban. Cascade-stage removal = ban (`:40-43`).

The model may **generate** the answer from local context when skip is legal. It may **not** choose skip, set T, or declare “I already know.”

## 4. Types, enums, objects

Copy from `J-plan/J-73_retrieval_abstention_gate.md:8,25,47-49` and the H-39 / D-16 neighbors it cites. Do not invent calibration fields.

```
fn should_retrieve(signal, T, mode) -> RetrievalGateDecision

RetrievalGateDecision {
  GatedSkip,                         // enforce: cascade does NOT fire
  GatedProceed,                      // cascade fires (below T, or uncertain)
  // observe-only: would-skip is journaled AND cascade still fires
}

RetrievalAbstained {                 // Pattern-2; OR QueryTrace decision-stage (net 0)
  Kind: GatedSkip | GatedProceed | AbstainOverride,
  calibrated_score,
  T,                                 // the threshold it cleared / missed
}

// REUSED, not re-authored (IC-13):
Abstain                            // abstention_score.rs
decide_low_score_abstains
low_score_routes_to_abstain        // agent_pool
conformal_calibration.rs
RefusalDecision { Answer, Abstain } // D-16; H-39 reads this as the eval-log bit
query_class                        // FAST / STANDARD / DEEP; T lives near here

PerTaskTypeT {
  function,                          // paper DESIGN 0.15 — NOT the ELAI default to copy
  line_or_api,                       // paper DESIGN 0.20 — calibrate on ELAI data (ID-5)
}
```

**Paper T values are DESIGN-rationale.** Named constants in ELAI config, not magic numbers, **ELAI-calibrated** (`:8,51`). Rebuilding with 0.15/0.20 hardcoded as production T is a cousin that violates ID-5.

**`QueryTrace` decision-stage field existence** is a prefer-reuse fork to resolve at edit (`:25`). UNCERTAIN whether the field already exists (§16).

## 5. Control flow / state machine

**Owner of the loop:** `pipeline.rs` before the cascade. Not the model. Not J-75 (stage ablation). Not J-108 (depth hops).

**Forced order:**

```
query enters pipeline.rs
  → read calibrated abstention signal (reused)
  → load T for this query_class / task type
  → if observe-only: journal would-skip-or-proceed; ALWAYS cascade
  → if enforce:
       signal missing OR below T OR uncertain → cascade (GatedProceed)
       signal clears T → skip cascade (GatedSkip)
```

**Forced fail-closed:** empty/missing signal → retrieve. Never skip.

**No token-level gate.** Sidecar `<cc>/abstain` is Tier-2 and still compared to config T; the special token is a **signal source**, not an LLM veto (`:8,20`).

J-72 (adapted retriever) is a later consumer: this gate decides whether to **invoke** it (`:57`).

## 6. Grounding, receipts, gates

This organ **gates cost**. It does not verify the eventual answer. The skip itself must be a **journaled receipt** so FP (skipped-but-needed) is measurable (`:9,49`).

**Selector hierarchy for the skip bit:** calibrated score + config T > sidecar special-token logit (Tier-2, still vs T) > LLM “I know this” **BANNED**.

H-39’s abstain bit is **answer**-side, read from the eval-log receipt (RD-1). J-73 **repurposes** that machinery to **retrieval entry**. Honest residue: “is this query answerable from local context” is a **different distribution** than “is this answer confident” — that is why GD-4 measures FP before enforce (`:61`).

**Never allowed as the skip signal:** model self-report, uncalibrated logit, paper 0.15/0.20 as ELAI’s production T, “the cascade is slow so skip,” J-75 prune flags.

## 7. Budgets, limits, ceilings

| Knob | Value | Scope |
|---|---|---|
| T | per-task-type named constants, ELAI-calibrated | near `query_class` |
| paper T (DESIGN only) | 0.15 function / 0.20 line-API | **not** production |
| observe-only floor | ≥3 real runs bounding FP before enforce | GD-4 |
| gate LOC (plan) | ~60 | `pipeline.rs` |
| SSB | `RetrievalAbstained` **or** `QueryTrace` field; net 0..+1 | per query |
| LIVE cited | ~269/578 post-J-66 (2026-06-22); re-grep W-9 | schema |
| Tier-2 | sidecar `<cc>/abstain`; RunPod-gated | deferred |

**Overflow / refuse:** no `ELAI_*_ENABLE` default-OFF without a wire-dark-allowlist row + promotion plan (R9/W-2) (`:44`). The plan does **not** name a specific env flag for Tier-1 (unlike J-108 / J-106). UNCERTAIN whether Tier-1 is always-on observe-then-enforce or flag-gated (§16).

No GRC call budget. No cascade-stage drop.

## 8. Roles and neighbors

Role-agnostic. Any retrieval-tier query hits the gate.

**Consumes:** `abstention_score.rs` / `conformal_calibration.rs` (NOT re-authored); D-16 τ (compose, IC-13); `query_class` tier router; `QueryTrace`; J-72 later (`:32-35,57`).

**Produces:** skip-or-proceed before cascade; journaled decision; retrieval-skipped rate for KnowledgeDrawer (B-29) or W-6 absence (`:26,45`).

**Must not fork:** a second conformal calibrator; a new retrieval-decision head in Tier-1; J-75 stage deletion; LLM skip-adjudication.

**Distinct from:** H-39 (scores **answers**, credits correct abstention); D-16 (claim refusal τ); J-108 RN-3 (abstain **after** hops, not before cascade); J-110 (WHEN abstain vs WHETHER — FEATURE-MAP notes J-110 is distinct from J-73’s WHEN/whether retrieval) (`ELAI-FEATURE-MAP.md:1466` cites “distinct from J-73's WHEN”).

## 9. Config / flags / defaults

- Per-task-type T: named constants near `query_class`, not magic numbers (`:8`).
- Observe-only **default until** FP bounded on ≥3 real runs (`:19`).
- Tier-2 sidecar head deferred; cheap path ships regardless (`:20,62`).
- SSB prefer-reuse vs new type: decide at edit (`:25`).
- R9: no new default-OFF `ELAI_*_ENABLE` without allowlist + promotion (`:44`). Plan does not mint a named flag for the gate itself.
- Frontend: skipped-rate on KnowledgeDrawer or W-6 (`:26`).

## 10. Implementation map

| Symbol | Spec path |
|---|---|
| `crates/knowledge/src/pipeline.rs` `should_retrieve` | before bm25 → vector → pagerank → catrag → arag (`:15,30`) |
| per-task-type T near `query_class` | FAST/STANDARD/DEEP (`:16,31`) |
| `crates/orchestrator/src/abstention_score.rs` | reused signal (`:13,33`) |
| `conformal_calibration.rs` | reused (`:13,33`) |
| D-16 UniCR τ | calibration compose (`:17,34`) |
| `RetrievalAbstained` or `QueryTrace` | journal (`:25`) |
| KnowledgeDrawer skipped-rate | frontend_pair (`:26`) |

No cross_lang pair (Rust-internal) (`:26`).

## 11. Journal / SSB events

Plan SCHEMA live ~269/578 at authoring; re-grep W-9 (`:25,55`):

```
RetrievalAbstained {
  Kind: GatedSkip | GatedProceed | AbstainOverride,
  calibrated_score,
  T,
}
// OR QueryTrace decision-stage field carrying the abstain verdict (net 0)
```

Observe-only still journals would-skip (`:49`). Rebuild: the skip must be auditable; silent skip is not this organ.

## 12. Tests and observables

Observable (`:46-49`, `J-plan/J-SUMMARY_INDEX.md:1716`):

| Case | Assert |
|---|---|
| local-context suffices (enforce) | `GatedSkip`; **no** bm25/vector/rerank work; answer from local context; journal has score + T |
| below T | full cascade (`GatedProceed`) |
| missing/uncertain signal | full cascade (never silent skip) |
| observe-only | would-skip journaled **and** cascade still runs |
| LLM adjudication | banned (TD-3) |

Negative (R1): a below-T query that skips is a **fail**. A gate that prunes cascade stages is the wrong organ (J-75).

W-1 analog: a real `elai` retrieval query through `pipeline.rs`, not a unit test that never entered production (`:37-38`).

No named `j73_*_contract.rs` file in the plan (unlike J-107/J-108). Rebuild still needs the table above. UNCERTAIN exact test filenames (§16).

## 13. Rebuild contract

Another harness has **this** organ iff all of the following are true:

- A `should_retrieve` (or equivalent) runs **before** the retrieval cascade, not after.
- Decision is calibrated-score vs per-task-type config T. **No** LLM adjudication.
- Uncertain / missing / below-T → retrieve (fail-closed).
- Observe-only journals would-skip without skipping until FP is measured.
- Skip is journaled (dedicated row or an existing trace field).
- Reuses the harness’s abstention / conformal calibrator; does not grow a second one.
- Does **not** remove cascade stages.
- Production T is measured on this harness’s data, not Repoformer’s 0.15/0.20.

A prompt “skip retrieval if you know,” a HyDE/multi-query cousin, or pruning PageRank is **not** this organ.

## 14. Non-goals / banned designs

- Training a new retrieval-decision head from scratch in Tier-1 (`:10`).
- Cascade-stage removal / prune (J-75 + user confirmation) (`:10,43`).
- LLM adjudication of `should_retrieve` (`:40`).
- Enforced skip before FP bounded (`:41`).
- Second abstention calibration impl (IC-13) (`:42`).
- Copying paper T as ELAI’s production threshold (ID-5) (`:51`).
- New default-OFF env flag without allowlist (R9) (`:44`).

## 15. Source citations

Primary: `docs/thefinalglance/J-plan/J-73_retrieval_abstention_gate.md` (full file). Index: `J-plan/J-SUMMARY_INDEX.md:1709-1716`. Feature map: `ELAI-FEATURE-MAP.md:193-200`. Paper (DESIGN): Repoformer arXiv:2403.10059, `workflow/papers/repoformer-interface.md`. Seam confirmation: `context-retrieval-research-2026-06-23.md` C2. Neighbors: H-39 (`reasoning-architecture/H-39-abstention-aware-scoring.md`), D-16 (`reasoning-architecture/D-16-conformal-refusal.md`), J-72/J-75. Wave cross-cut: `J-plan/J-SUMMARY_INDEX.md:1738-1745`.

## 16. Gaps

Label **UNCERTAIN**; do not fill:

- Whether `QueryTrace` already has a decision-stage field (net 0 vs +1) (`:25`).
- `AbstainOverride` semantics (who overrides, when) (`:25`).
- Numeric ELAI T after calibration (paper 0.15/0.20 are not it).
- Mapping FAST/STANDARD/DEEP tiers onto function vs line/API T (`:8` names both homes).
- Exact score function: reuse `decide_low_score_abstains` as-is vs a retrieval-specific wrapper on the same calibrator (`:8,61` flags distribution shift).
- Named env flag for Tier-1 (R9 clause exists; no `ELAI_RETRIEVAL_ABSTAIN_*` string in the plan).
- Named contract-test file.
- Observe-only → enforce promotion owner / allowlist row once FP is bounded.
- How “local context suffices” is determined at skip time vs merely “score ≥ T” (`:49` states both the within-file completion fixture and the score/T rule).
- LIVE schema at implement time (re-grep W-9).
- Sidecar `<cc>/abstain` logit contract (Tier-2; verify lane support before committing the head) (`:62`).

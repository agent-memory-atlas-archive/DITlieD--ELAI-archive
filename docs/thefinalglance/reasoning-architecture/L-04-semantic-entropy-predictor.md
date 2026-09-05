# Semantic-entropy inter-seed predictor P3 (L-4)

- **ELAI ids:** L-4 / L-T1 / P3 / NN-3 / DC-3 / WAVE 2
- **Spec status (as designed):** PLANNED (`L-plan/L-4_semantic_entropy_inter_seed_predictor.md:3`). Live `claim/semantic_entropy.rs` is a **cousin**: NLI clustering + `SemanticEntropyMeasured` exist, but resample is `handle_infer_request` (not I-4 `run_best_of_n`), default-OFF `ELAI_SEMANTIC_ENTROPY_RESAMPLE`, N=3 not N=5, and `CrossFamilyDiverge` is missing from the live Kind enum.
- **Crate / module (spec):** `crates/orchestrator/src/claim/semantic_entropy.rs`; production caller is L-2 cost-gate → `semantic_entropy`, **only on medium+ cheap-risk** (`L-plan/L-4_semantic_entropy_inter_seed_predictor.md:36-42`).
- **Purpose of this file:** enough architecture to rebuild the meaning-cluster entropy predictor. Not a status ledger.

## 1. What it is

The **semantic-entropy inter-seed predictor (P3)** is the one genuine net-new CGG producer (DC-3). The paper’s predictor 3 is inter-seed variance: N independent trajectories that disagree mark epistemic uncertainty (§4.2). The LLM analogue: resample N alternative answers, **cluster by meaning**, emit entropy over those clusters — high entropy = high risk. ELAI already resamples (I-4 `run_best_of_n`) and counts concurrence (D-3); it does **not** cluster by meaning. L-4: resample via I-4, cluster via F-12 `DeterministicNliJudge` (entailment-equivalence, **not** embedding cosine, **not** an LLM-judge), emit `PredictorSignal{SemanticEntropy, entropy}` into L-2. Cross-family hardening: route a subset through D-13 so within-family shared bias (which same-model resampling misses — GD-5) is flagged `CrossFamilyDiverge`. Done-gate thresholds are ELAI-measured at L-10, **never** a paper AUROC (ID-5) (`L-plan/L-4_semantic_entropy_inter_seed_predictor.md:5`, `L-plan/L-SUMMARY_INDEX.md:32-38`, `L-plan/_grounding/L-CGG-GROUNDING-MAP.md:84-86,120-122`).

## 2. Why it exists

L-3 catches invented refs. It does **not** catch confident wrong **reasoning** that cites real symbols. P3 is the orthogonal mechanism HG-2 needs: two distinct predictors, not two views of one (`L-plan/L-4_semantic_entropy_inter_seed_predictor.md:13`). Phrasing variance is **not** disagreement: five paraphrases of one true answer must cluster together; only semantically distinct answers split. That is why clustering is NLI entailment (F-12, local, `ELAI_SEED`), not cosine.

If this organ is missing, L-2 can only fuse P1 + P2 (reconstruction + oscillation) and has no meaning-disagreement signal. If it is faked (cosine clusters, LLM “these mean the same”, or same-family self-critique labeled cross-family), HG-2 is fake convergence and GD-5 is hidden.

**Honest residue (GD-5), must stay in-file:** same-model resampling cannot catch a belief the model is confidently wrong about across all seeds (training-data-shared error). D-13 is a **mitigation, never an elimination** (`L-plan/L-4_semantic_entropy_inter_seed_predictor.md:13,30,68`, `L-plan/_grounding/L-CGG-GROUNDING-MAP.md:109-111` wait: GD-5 is `L-plan/L-0_INDEX.md:46`). L-15 measured that single-model self-consistency can make hardest-math **worse**; land D-13 **alongside** L-4, not after (`L-plan/L-15_repair_controller_and_evidence_first_reorder.md:143-147`).

## 3. How it works (runtime)

1. **Who calls it.** L-2 `score_claim_risk` cost-gate, only when cheap P1/L-1 residual is **medium+**. Not contracts-only (R7) (`L-plan/L-4_semantic_entropy_inter_seed_predictor.md:41-43`). Scope of claims: Behavior / Causal / Api flagged medium by L-3 (`:28`).
2. **Resample (I-4, not a new sampler).** `run_best_of_n` over SessionDag. N comes from L-2’s J-84 cost-budget (`:28`). Observable fixture uses **N=5** (`:52`). No new sampling loop (GD-1). Live-model N-sample is RunPod/free-pool/Batch (J-54/H-5); fixture tier ships now (`:7,78`).
3. **Cluster by meaning (F-12, deterministic).** Pairwise NLI entailment-equivalence via `DeterministicNliJudge`. Paraphrases join; distinct meanings split (`:29`). Local, not cloud — keeps `ELAI_SEED`.
4. **Entropy (deterministic).** Semantic entropy over cluster mass. Exact formula **not named** in the L-4 file (§16). Emit `PredictorSignal { source: SemanticEntropy, value: entropy, fired }`.
5. **Cross-family arm (D-13).** Route a **subset** through the adversary (different pretraining family). Same-family-agreed claim that D-13 contradicts → Kind `CrossFamilyDiverge` (`:30`). D-13 **refuses** same-family dispatch — keep that (`:46`, `D-13-adversary-role-cross-family.md:10`).
6. **Wire into L-2.** Cost-gated. Default **OBSERVE-only** (GD-4) (`:31`).
7. **Commit.** `SemanticEntropyMeasured` Pattern-2 Kind `{SeedsAgree, SeedsDiverge, CrossFamilyDiverge}` **or** reuse a verifier row (net 0..+1) (`:53`, `L-plan/L-0_INDEX.md:133`).
8. **Abstain / fail-closed.** n<2 samples is **unmeasured**, not “entropy 0 = agree” (live cousin states this; plan is silent — rebuild the fail-closed reading: do not journal SeedsAgree on a single sample). Prompt mutation to manufacture diversity is banned (live `semantic_entropy.rs:7-8`; plan requires I-4 candidates answering the **same** claim). Cosine / LLM-judge clustering banned (`:45`). Paper AUROC is not a threshold (`:47`).
9. **The model may generate** the N answers. It may **not** cluster them, set entropy, pick D-13’s family, or override `fired`.

Live cousin (do not rebuild as this organ): `collect_extra_samples` calls `rpc.handle_infer_request` with the **same prompt** (`semantic_entropy.rs:45-68`) — **not** I-4 `run_best_of_n`. Gate `ELAI_SEMANTIC_ENTROPY_RESAMPLE` default-OFF (`:26-35`) against L-4 W-2 “no flag”. `N_RESAMPLE = 3` (`:24`). Clustering: blake3-identical strings merge first, then **bidirectional** Entailment (`:135-175`); Shannon H / ln(n), clamp `[0,1]` (`:177-183`). `n<2` → entropy 0.0, `measured()=false`, **no SSB row** (`:192-200`). Kind enum is only SeedsAgree/SeedsDiverge (`ssb/payloads.rs:22140-22146`). `resample_is_indicated` uses live `expensive_predictors_for` (SemanticEntropy only on High/Critical) (`semantic_entropy.rs:37-42`, `risk.rs:47-50`) vs plan **medium+**.

F-12 live: `elai_memory::DeterministicNliJudge` (`memory/src/semantic_contradiction.rs:115`) — marker/token heuristic, not a neural NLI. Rebuild **calls** that judge (GD-1); do not swap in a cloud NLI.

## 4. Types, enums, objects

L-4 emits L-2 `PredictorSignal` plus an optional journal Kind (`L-plan/L-4_semantic_entropy_inter_seed_predictor.md:29,53`).

```
PredictorSignal {
  source: PredictorKind::SemanticEntropy,
  value: entropy,
  fired: bool,
}

SemanticEntropyMeasuredKind {   // Pattern-2
  SeedsAgree,                   // N samples, one meaning cluster, low entropy
  SeedsDiverge,                 // k>1 meaning clusters, high entropy
  CrossFamilyDiverge,           // same-family agree, D-13 contradicts
}
```

**Consumed, not redefined:**

- I-4 `SearchLadder::run_best_of_n` (`search/ladder.rs:184`, `I-04-evidence-ranked-search-ladder.md` §3).
- F-12 `DeterministicNliJudge` / `NliLabel::{Entailment,…}` (`semantic_contradiction.rs:115`).
- D-13 `probe` / `CounterEvidenceSchema`; same-family → error, fall to Mode E (`D-13-adversary-role-cross-family.md:20-22`).
- D-3 agreement count — entropy is the richer form; do not replace D-3 (`L-plan/L-4_semantic_entropy_inter_seed_predictor.md:17`).
- D-7 cross-domain canary named as coverage, not a P3 input (`:20`).

**N:** from J-84 budget; observable N=5; live cousin N=3. Decide schedule at L-10 (`:99`).

## 5. Control flow / state machine

**Owner:** orchestrator P3, invoked from L-2 cheap-medium+ gate. Observe-only until L-7.

```
cheap residual medium+ on Behavior|Causal|Api
  → I-4 run_best_of_n (N from J-84)
  → F-12 cluster by bidirectional entailment-equivalence
  → entropy over cluster masses
  → if D-13 subset: probe cross-family
       same-family → refuse (not CrossFamilyDiverge)
       contradict → CrossFamilyDiverge
  → PredictorSignal::SemanticEntropy + optional SSB Kind
  → L-2 fuse (HG-2; L-15 R1: P3+P2 must not Low a receipt-available factual)
```

**Forced:**

- NLI only (`rg cosine|embedding.*cluster|llm.*judge` MUST be 0) (`:60`).
- Fixture path must be byte-recomputable under `ELAI_SEED` (F-12 local) (`:29,81`).
- L-15 R2: land L-3/L-5 **before** this organ in the wave; still parallel-safe file-wise (`L-plan/L-15_repair_controller_and_evidence_first_reorder.md:112-117`).

## 6. Grounding, receipts, gates

P3 is an **uncertainty** signal, not a reconstruction receipt. High entropy ≠ “the cited file is missing”; it is “seeds do not mean the same thing”. L-15 R1: absence of P3 fire is **not** presence-of-grounding on a checkable claim (`L-plan/L-15_repair_controller_and_evidence_first_reorder.md:104-110`).

**Selector hierarchy:**

1. D-13 CrossFamilyDiverge (when armed) — GD-5 mitigation.
2. NLI cluster entropy (F-12).
3. D-3 ≥2-concur count — coarser sibling, not a substitute clusterer.
4. Embedding cosine / LLM-judge / same-family critique — **never**.

**Never allowed as a confidence signal:** paper AUROC (SelfCheckGPT / Nature semantic entropy / SAC3 figures), cosine similarity, the model’s “we all agree”, P2+P3 Allow on class A/B/C factual with no P1/P-cf receipt (R1).

**WV-2 IDs:** plan lists SelfCheckGPT / Semantic Entropy / SAC3 as method prior-art with arXiv IDs **PENDING-VERIFY** (`L-plan/_grounding/L-CGG-GROUNDING-MAP.md:131-138`, `L-plan/L-0_INDEX.md:167`). This architecture file **does not treat those IDs as verified**. Cite the **mechanism** via the L-4 plan; L-10 remains ELAI-measured.

## 7. Budgets, limits, ceilings

| Knob | Value | Scope |
|---|---|---|
| resample driver | ~50 LOC | `semantic_entropy.rs` |
| cluster + entropy | ~70 LOC | same |
| cross-family arm | ~40 LOC | same |
| file total | ~160 LOC | (`:88`) |
| N | J-84 budget; observable **5**; live cousin **3** | per claim, cost-gated |
| live resample tokens | `RESAMPLE_MAX_TOKENS=512` (`semantic_entropy.rs:25`) | not in L-4 file |
| fire seed | live `DEFAULT_ENTROPY_FIRES_AT=0.5` (`:21`) | **not** spec; L-10 |
| fixture vs live N-sample | fixture now; live RunPod/free-pool | W-10 (`:7,78`) |
| SCHEMA | +`SemanticEntropyMeasured` or reuse verifier; net 0..+1 | W-9 |
| observe-only | GD-4 | until L-7 |

N is the dominant cost lever (`:99`). Observe-first to find the N where entropy stabilizes (ID-5).

## 8. Roles and neighbors

Role-agnostic scoring; D-13 adversary is a **different family** from the executor (`D-13-adversary-role-cross-family.md:10`). PPM/Qwen same-family must not count as D-13 (`D-10-mode-routing-predicate.md` §14).

**Consumes:** I-4 `run_best_of_n`; F-12 judge; D-13; D-3; L-1 ClaimSet; L-2 signal (`L-plan/L-4_semantic_entropy_inter_seed_predictor.md:38-40`).

**Produces:** P3 signal; `SemanticEntropyMeasured` Kind triad.

**Must not fork:** a second sampler; cosine clusterer; LLM judge; same-family “adversary”.

**Frontend:** N/A — feeds L-2 (`:49-50`).

**Decorrelation (H-40):** if P3 correlates with P1 above threshold at L-10, treat them as **one** signal for HG-2 (`:100`).

## 9. Config / flags / defaults

- Plan W-2: **no predictor flag**; gate flag is L-7 (`:75`).
- Live cousin: `ELAI_SEMANTIC_ENTROPY_RESAMPLE` default-OFF (`semantic_entropy.rs:26-35`) — isolation, not a license to skip the organ in a rebuild. I-4 remains DARK behind `ELAI_SEARCH_LADDER_ENABLE` (`I-04-evidence-ranked-search-ladder.md` header). Rebuild **calls** I-4; do not invent a third sampler behind a new default-off if I-4 can run.
- D-13 `ELAI_ADVERSARY_ENABLE` — CrossFamilyDiverge arm stays dark if no alt-family; do not fake it with same-family.
- Default OBSERVE-only (`:31`).
- Decision doc: `workflow/reasoning/L-4-decisions.md` (N schedule, entropy formula, fire threshold). **Absent** at architecture time.
- blake3 sample digests for GD-2 (live `sample_digests`, `semantic_entropy.rs:107-114`). No `.unwrap()` on prod.

## 10. Implementation map

| Symbol | Spec path |
|---|---|
| `crates/orchestrator/src/claim/semantic_entropy.rs` | driver + cluster + entropy + D-13 arm + P3 (`:36`) |
| `search/ladder.rs::run_best_of_n` | I-4 (`:39`) |
| F-12 `DeterministicNliJudge` | `crates/memory/src/semantic_contradiction.rs:115` |
| D-13 `adversary.rs` | cross-family |
| `claim/risk.rs` | ~20 LOC wire (`:89`) |
| `l4_semantic_entropy_contract.rs` | tests (a)–(g) (`:85`) |

**Live neighbors:** `SemanticEntropyPredictor`, `measure`, `emit_semantic_entropy_measured` (`semantic_entropy.rs:71-218`); extra contract `l4_semantic_entropy_receipt_contract.rs` / `l4_semantic_entropy_resample_contract.rs`. `score_claim_risk_wired_with_samples` in `roundtrip.rs:179+` is a later glue, not the I-4 driver.

## 11. Journal / SSB events

Plan (`:53`, `L-plan/L-0_INDEX.md:133`):

```
SemanticEntropyMeasured   // Pattern-2; OR reuse a verifier row
  Kind: SeedsAgree | SeedsDiverge | CrossFamilyDiverge
```

Live payload cousin (Agree/Diverge only):

```
SemanticEntropyMeasuredPayload {
  kind,
  sample_count: usize,
  cluster_count: usize,
  entropy_milli: u32,          // entropy * 1000
  sample_digests: Vec<[u8; 32]>,
}
```

(`semantic_entropy.rs:207-214`). Rebuild must still have **CrossFamilyDiverge**. Unmeasured (n<2): **no row** (live; plan unspecified — keep the fail-closed silence so SeedsAgree cannot mean “we did not sample”). GD-2: entropy recomputable from recorded cluster assignments / digests under `ELAI_SEED` (`:81`).

## 12. Tests and observables

Named (`:85`):

| Id | Assert |
|---|---|
| (a) | N samples sharing a meaning → low entropy |
| (b) | N splitting into k>1 meanings → high entropy |
| (c) | paraphrases of **one** answer cluster together (not flagged) |
| (d) | clustering uses NLI not cosine (no embedding path) |
| (e) | cross-family arm flags same-family-agreed + D-13-contradicted |
| (f) | entropy recomputable from cluster assignments |
| (g) | D-13 refuses same-family dispatch |

**observable_effect** (`:51-54`, `L-plan/L-SUMMARY_INDEX.md:38`): N=5 all one meaning → SeedsAgree; 5 samples → 3 clusters → SeedsDiverge; same-model-agreed + D-13 contradict → CrossFamilyDiverge. Negative: paraphrases of one true answer cluster together.

**W-1:** live cost-gate emits P3 that flips a claim to High when seeds diverge (`:74`). High still needs HG-2 ≥2 distinct predictors at L-2 — P3 alone caps Medium.

Adversarial: cosine clustering; LLM-judge; prompt-mutated “diversity”; SeedsAgree on n=1; paper AUROC as fire threshold; P2+P3 Low on a checkable claim with no receipt (R1).

## 13. Rebuild contract

Another harness has **this** organ iff all of the following are true:

- Resample is I-4 `run_best_of_n`, not a new loop.
- Clusters by F-12 NLI entailment-equivalence; `rg` cosine/embedding-cluster/llm-judge = 0.
- Paraphrases of one answer form one cluster (test c).
- Emits `PredictorSignal::SemanticEntropy` into L-2 only on medium+ cheap risk.
- Journal Kind includes `{SeedsAgree, SeedsDiverge, CrossFamilyDiverge}` (or documented reuse plus CrossFamilyDiverge).
- D-13 arm refuses same-family; GD-5 residue named in-module.
- Thresholds ELAI-measured at L-10, never paper AUROC.
- Fixture path `ELAI_SEED`-recomputable; live N-sample W-10 gated.
- Tests (a)–(g). Production path is L-2 cost-gate.

A cosine k-means over embeddings, an LLM “semantic entropy” score, infer-N-times-on-the-executor without I-4, or live Kind without CrossFamilyDiverge is a **cousin**, not this organ.

## 14. Non-goals / banned designs

- Re-implementing I-4 or F-12 (GD-1 P0) (`:48`).
- Embedding-cosine or LLM-judge clustering (`:45`).
- Same-family self-critique as cross-family (`:46`).
- Paper AUROC as done-gate (`:47`).
- Treating P3 as a reconstruction receipt (that is L-3) or as interventional causal (L-5).
- Standalone P3 UI (`:49-50`).
- Prompt mutation to force diversity (live ban; keep).
- Counting P3+P2 as HG-2 grounding on a receipt-available factual (L-15 R1).
- Claiming the CGG catches confident-shared-wrong beliefs (GD-5).

## 15. Source citations

Primary: `docs/thefinalglance/L-plan/L-4_semantic_entropy_inter_seed_predictor.md` (full file). Index: `L-plan/L-SUMMARY_INDEX.md:32-38`. Doctrine: `L-plan/L-0_INDEX.md:46,49-53,93-94,133,153,167`. Grounding map: `L-plan/_grounding/L-CGG-GROUNDING-MAP.md:47-51,84-86,120-122,131-138`. L-15: `L-plan/L-15_repair_controller_and_evidence_first_reorder.md:104-117,143-147`. Neighbors: `I-04-evidence-ranked-search-ladder.md`, `D-13-adversary-role-cross-family.md`, `D-03-multi-verifier-consensus.md`, `L-02-claimrisk-convergence-fuse.md`. Live: `claim/semantic_entropy.rs`, `memory/src/semantic_contradiction.rs:115`, `ssb/payloads.rs:22134-22148`. Paper mechanism: arXiv:2606.27326 §4.2 predictor 3 [EXTERNAL/DESIGN-RATIONALE]. WV-2 arXiv IDs: PENDING-VERIFY (`L-CGG-GROUNDING-MAP.md:138`).

## 16. Gaps

Label **UNCERTAIN**; do not fill:

- Entropy formula (Shannon vs normalized vs 1−max-mass). Live uses H/ln(n) clamp `[0,1]` — not in the L-4 file.
- Bidirectional vs one-way entailment; `Neutral`/`Contradiction` handling (live: classify error = not entailment).
- Exact N schedule and fire threshold (L-10 / L-4-decisions.md).
- What “subset” means for the D-13 arm (every medium claim vs High only vs sample 1 of N).
- Whether Causal claims should wait for L-5 instead of resampling (task 1 includes Causal).
- How D-3 agreement count combines with entropy (replace vs extra signal).
- Plan W-2 “no flag” vs live `ELAI_SEMANTIC_ENTROPY_RESAMPLE`.
- Whether `handle_infer_request` same-prompt N times is an allowed I-4 stand-in when the ladder is DARK.
- P1–P3 correlation threshold that collapses them to one HG-2 voter (`:100`).
- Payload fields beyond Kind (live milli/digests).
- Unmeasured n<2 journal policy (plan silent; live omits the row).

# Cheap grounding stack (J-41)

- **ELAI ids:** J-41 / WAVE DD-3 / ExtractiveGroundingPass / BinaryGroundingJudge / DecodingMode::ClosedSet
- **Spec status (as designed):** PLANNED BUILD (activate F-12 `OnnxNliJudge` + extend H-53/E-35). Substrates exist; ClosedSet arm and binary adapter are the completion (`J-plan/J-41_cheap_grounding_stack.md:1-5`, `J-plan/J-SUMMARY_INDEX.md:1097-1115`).
- **Crate / module (spec):** extractive pass over 2B SOURCE_CHECK; `crates/memory/src/nli_onnx.rs` `OnnxNliJudge`; `citation_prebind.rs` `EvidenceAnchorSet`; `crates/providers/src/decoding.rs` new `ClosedSet` arm; D-16 refusal.
- **Purpose of this file:** enough architecture to rebuild extractive → small claim model → closed-set citation. Not a status ledger.

## 1. What it is

The **cheap grounding stack** is a three-layer, **frontier-LLM-free** source-grounding pipeline: (1) a cheap deterministic **extractive fingerprint** first pass, (2) a **small claim-level entailment** model only on what the first pass flags, (3) **closed-set citation binding** so ghost citations are **structurally impossible**. Token-level membership is **vacuous** (common tokens sit in every source) and forbids legitimate synthesis. Out-of-set / unsupported → **D-16 abstain**, never forced citation (`J-plan/J-41_cheap_grounding_stack.md:4,7-11`).

J-40 supplies the harness-fetched compressed spans as the evidence set (`J-plan/J-41_cheap_grounding_stack.md:21,44`).

## 2. Why it exists

Local-lane + cost: source-grounding cannot wait for a frontier judge. Field ghost-citation rates (survey, **WHY only**) make closed-set binding load-bearing (`J-plan/J-41_cheap_grounding_stack.md:37-38`). If this organ is missing, synthesis invents cites. If faked (token-hash membership, NLI as the **sole** gate on paraphrase, RAG-HAT forced citation without abstention), false-negatives dominate **or** constraint **amplifies** confident fabrication (`J-plan/J-41_cheap_grounding_stack.md:11`).

EXP-2: 100 (source-snippet, generated-claim) pairs **from the ELAI codebase** (50/50). Candidate MiniCheck-FT5 770M or Paladin-mini 3.8B. Balanced accuracy **<75%** = does **not** transfer to code-claim domain without fine-tune. **No default-ON** of the model slot until EXP-2 passes (`J-plan/J-41_cheap_grounding_stack.md:8,48`).

## 3. How it works (runtime)

1. **Who calls it.** Research/synthesis path (`research_pipeline.rs`, J-40 hands spans); `prebind_verify_citations`; decode `default_decoding_for` when `EvidenceAnchorSet` is non-empty (`J-plan/J-41_cheap_grounding_stack.md:26`).
2. **Evidence set.** J-40 ranked compressed **verbatim** spans. Not the raw web, not model memory.
3. **Tier 1 — ExtractiveGroundingPass (deterministic, never the sole gate)** (`J-plan/J-41_cheap_grounding_stack.md:9`):
   - substring exact (Rabin-Karp/KMP) — full confidence
   - fuzzy (≥0.8 token-overlap / edit-distance)
   - semantic (≥0.85 cosine)
   - Granularity: **token-level VACUOUS**; sentence / **n-gram≥5** meaningful; plagiarism floor **8–10 consecutive words**
   - Flags fabrication candidates (claims with **ZERO** supporting span) for tier 2; passes verbatim-supported cheaply
   - Home: phase-2B SOURCE_CHECK + new span matcher; E-54 9/13-gram contamination filters are in-tree precedent (`:18-19`)
4. **Tier 2 — claim-level entailment (only flagged claims).** Drop MiniCheck-FT5 / Paladin-mini ONNX into `<root>/.elai_cc/models/onnx/` so compiled `OnnxNliJudge` activates (`nli_onnx.rs:60`). Live: loads `DEFAULT_NLI_MODEL_FILENAME`; **file ABSENT** → Err fail-safe **SKIP** (`J-plan/J-41_cheap_grounding_stack.md:15`). **ADAPTER:** slot is **3-class NLI** `label_order: [NliLabel;3]`; MiniCheck/Paladin are **BINARY** (supported/unsupported). Deliverable: `BinaryGroundingJudge` sibling on `NliJudge` **or** binary→NLI map (`:9,49`). Claim decomposition via small T5/BART extractor (fine-grained beats sentence-level 10–15% coverage — **[EXTERNAL]** rationale, not an ELAI SLO).
5. **Missing model.** Explicit **SKIP verdict surfaced**, **NOT** silent grounded (`stub_ban`, `:28`).
6. **Tier 3 — ClosedSet decode.** New `DecodingMode::ClosedSet { allowed_ids }` feeds H-53 `EvidenceAnchorSet` id list as grammar/token allowlist; invalid completions → **logit -inf**. Ghost cites structurally impossible (`:10`). Live `DecodingMode` is `{JsonSchema, Grammar, Regex, FreeText}` — **no ClosedSet yet** (`decoding.rs:73-87`). Attach in `default_decoding_for` when anchor set non-empty (`:17`). Confirm local sidecar exposes allowlist (E-35 PARTIAL) (`:51`).
7. **H-53 bind.** `EvidenceAnchorSet` (`citation_prebind.rs:106`); `freeze_anchors(ranked, seed)` `:192`; `check_citation_in_set` `:308`; prod `prebind_verify_citations`. SSB `CitationAnchorsBound` (229) already exists (`:16`).
8. **Out-of-set / unsupported → D-16** `RefusalDecision` abstain. Quality cost of enforcement under partial evidence is **documented abstention**, not forced cite (`:10-11`).
9. **Pipeline.** extractive → OnnxNliJudge binary (flagged only) → ClosedSet decode → D-16 (`:21`).
10. **Commit.** Reuse `CitationAnchorsBound` + GROUNDING VERIFICATION verdicts, **net 0**. Observable: out-of-set cite **REFUSED** at decode or post-hoc rejected → D-16 / CitationAnchorsBound; unsupported flagged with no tier-2 entailment → **ungrounded** (`:29-30`).

The model may generate claims and **only** cite ids from the frozen set. It may **not** invent ids, grade its own grounding, or treat SKIP-as-pass.

## 4. Types, enums, objects

Copy from `J-plan/J-41_cheap_grounding_stack.md:24-27` and live traits.

```
ExtractiveGroundingPass
  // substring → fuzzy ≥0.8 → semantic ≥0.85 cosine
  // n-gram>=5; plagiarism floor 8-10 words

trait NliJudge { classify(premise, hypothesis) -> NliVerdict }  // semantic_contradiction.rs:98
struct OnnxNliJudge { session, tokenizer, label_order: [NliLabel; 3] }  // nli_onnx.rs:60
BinaryGroundingJudge  // sibling OR label adapter; BINARY supported/unsupported

DecodingMode::ClosedSet { allowed_ids }   // NEW; invalid → logit -inf
fn bind_closed_set_decode(...)

EvidenceAnchorSet
fn freeze_anchors(ranked, seed)
fn check_citation_in_set(anchor_set, id, cited_excerpt)

RefusalDecision { Answer, Abstain }   // D-16
```

Live `DecodingMode` without ClosedSet is the **substrate**, not the organ.

## 5. Control flow / state machine

**Owner:** orchestrator grounding pipeline + provider decode. F-12 owns ONNX load. H-53 owns the frozen id set. D-16 owns refuse.

```
J-40 spans → EvidenceAnchorSet.freeze_anchors
claim generated
     │
     ▼
Tier1 extractive fingerprint
     ├─ verbatim support → pass cheap
     └─ zero span → flag
            │
            ▼
       Tier2 OnnxNliJudge / BinaryGroundingJudge
            missing .onnx → SKIP (surfaced, not grounded)
            unsupported → ungrounded; D-16 abstain
            EXP-2 <75% on code claims → do not default-ON this tier
     │
     ▼
decode under ClosedSet{allowed_ids = anchor ids}
     out-of-set token → -inf  OR  post-hoc reject + D-16
emit CitationAnchorsBound / grounding verdict (reuse)
```

**Forced:** extractive is necessary-not-sufficient; NLI is not sole gate; abstention path mandatory.

## 6. Grounding, receipts, gates

This **is** a receipt stack. Confidence is **external**: span match, small judge on flagged claims, structural closed-set. **Never** model “I cited it.”

Selector hierarchy unchanged: receipt > vote (not used here) > LLM self-eval banned.

**Never allowed:** token-level hash membership; SKIP-as-pass; forced citation without D-16; NLI-only on synthesized paraphrase (`J-plan/J-41_cheap_grounding_stack.md:11,28`).

## 7. Budgets, limits, ceilings

| Knob | Value | Scope |
|---|---|---|
| Fuzzy overlap | ≥0.8 | tier 1 |
| Semantic cosine | ≥0.85 | tier 1 |
| n-gram meaningful | ≥5 | granularity |
| Plagiarism floor | 8–10 consecutive words | tier 1 |
| EXP-2 | 100 pairs, 50/50; **≥75%** balanced acc to use model as gate | observe-only |
| MiniCheck / Paladin sizes | 770M / 3.8B | **EXTERNAL** candidates |
| SSB | reuse 229 + 2B verdicts; net 0 | W-9 |
| Default-ON | after EXP-2 + GD ≥3-run FP bound | `:54` |

Vendor MiniCheck “GPT-4 at 400× cheaper” is **WHY**, not an ELAI SLO (`:33`).

## 8. Roles and neighbors

Research/synthesis + decode roles that emit citations. `default_decoding_for(RoleDecoding::Executor)` today → Grammar (`:17`); ClosedSet attaches when anchors non-empty.

**Consumes:** F-12, H-53, E-35, D-16, J-40, 2B SOURCE_CHECK.

**Produces:** pass / ungrounded / SKIP / D-16 refuse; reused SSB.

**Must not fork:** a frontier LLM “verify this citation” as the stack.

## 9. Config / flags / defaults

- Model file at `<root>/.elai_cc/models/onnx/`. Absent = SKIP, not pass.
- ClosedSet gated on non-empty `EvidenceAnchorSet`.
- EXP-2 observe-only before treating MiniCheck/Paladin as a **code-claim gate** (`:8,48`).
- Frontend: not specified beyond journal rows.

## 10. Implementation map

| Symbol | Spec path |
|---|---|
| `ExtractiveGroundingPass` | NEW; composes 2B SOURCE_CHECK |
| `NliJudge` | `memory/src/semantic_contradiction.rs:98` |
| `OnnxNliJudge` | `memory/src/nli_onnx.rs:60` |
| `BinaryGroundingJudge` | NEW adapter |
| `EvidenceAnchorSet` / `freeze_anchors` / `check_citation_in_set` | `citation_prebind.rs:106,192,308` |
| `ConstrainedDecodingConfig` / `DecodingMode` | `providers/src/decoding.rs:73,122` |
| `bind_closed_set_decode` | NEW |
| `prebind_verify_citations` | `citation_verifier.rs` |
| `DEFAULT_NLI_MODEL_FILENAME` | onnx slot |

## 11. Journal / SSB events

Reuse **`CitationAnchorsBound` (229)** + GROUNDING VERIFICATION verdicts. Extractive telemetry: prefer reuse; add Pattern-2 only if genuinely new (`J-plan/J-41_cheap_grounding_stack.md:30`). Out-of-set → D-16 refusal row. Do not rewind 268/577.

## 12. Tests and observables

Plan (`:29`):

| Assert | |
|---|---|
| Cite outside frozen `EvidenceAnchorSet` | refused at decode (−inf) **or** post-hoc reject + D-16 / CitationAnchorsBound |
| Unsupported, flagged, no tier-2 entailment | logged **ungrounded** |
| Missing ONNX | SKIP surfaced, **not** grounded |
| Token-level membership | absent |
| Forced citation without abstain | absent |

EXP-2 recorded before default-ON of the model slot.

Adversarial: mapping all SKIP to Verified; ClosedSet not wired so ghost ids decode; NLI-only without extractive.

## 13. Rebuild contract

Another harness has **this** organ iff all of the following are true:

- Three layers: extractive fingerprint (not token-hash) → small claim judge on **flags only** → closed-set citation ids.
- Ghost ids **cannot** be produced (logit −inf or equivalent structural ban).
- Abstain path on out-of-set / unsupported (no forced cite).
- Missing small-model file is explicit SKIP, not pass.
- Code-domain transfer **measured** before the small model gates code claims.
- Evidence set is harness spans (J-40 shape), frozen as an id set.
- 3-class NLI vs binary grounding adapter is explicit.

A “regex the cite exists in the prompt” checker, GPT-as-judge, or token-hash overlap is a cousin.

## 14. Non-goals / banned designs

- Token-level hash membership (`J-plan/J-41_cheap_grounding_stack.md:11`).
- Generative claim rewriting (`:11`).
- Small NLI as **sole** gate on synthesized output (`:11`).
- Forced citation on partial evidence without abstention (RAG-HAT trap) (`:11`).
- SKIP-as-pass (`:28`).
- Shipping general MiniCheck as a code-claim **gate** if EXP-2 <75% (`:48`).
- Claiming survey ghost-cite % as ELAI measurements (`:38`).

## 15. Source citations

Primary: `docs/thefinalglance/J-plan/J-41_cheap_grounding_stack.md` (full file). Index: `J-plan/J-SUMMARY_INDEX.md:1097-1115`. Live: `nli_onnx.rs:60`, `semantic_contradiction.rs:98`, `citation_prebind.rs:106,192,308`, `decoding.rs:73-87`. Doctrine: grounding-side cog (`:45`).

**[EXTERNAL, WHY]:** MiniCheck 2404.10774; Paladin-mini 2506.20384; Perplexity closed-set / KG-Trie (`:32-36`). VERIFIED 2026-06-16 in-plan.

## 16. Gaps

Label **UNCERTAIN**; do not fill:

- BinaryGroundingJudge sibling vs label mapping (plan allows either) (`:9`).
- Which candidate model (MiniCheck-FT5 vs Paladin-mini) is default if EXP-2 both pass.
- T5/BART claim-decomposition: required vs optional; model id unspecified.
- Cosine embedding model for semantic ≥0.85.
- Exact Rabin-Karp vs KMP choice.
- E-35 sidecar allowlist actually exposed (PARTIAL) (`:51`).
- Post-hoc reject vs decode −inf when sidecar cannot ClosedSet.
- Extractive telemetry row vs reuse only.
- Fuzzy “token-overlap / edit-distance” which metric.
- How 2B SOURCE_CHECK regex composes with n-gram≥5 matcher (one pass or two).

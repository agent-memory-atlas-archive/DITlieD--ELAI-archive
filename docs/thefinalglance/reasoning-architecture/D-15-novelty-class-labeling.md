# Novelty-class labeling (D-15)

- **ELAI ids:** D-15 / RPS.VI / D0-019 / D0-050 / D004-D15-001..038 / SUP-007
- **Spec status (as designed):** DARK(`ELAI_NOVELTY_LABEL_ENABLE`) (`ELAI-FEATURE-MAP.md:598-601`). D-SUMMARY labels the plan PLANNED (`D-plan/D-SUMMARY_INDEX.md:110`). FEATURE-MAP: wired into `task_runner` under this **default-OFF** flag (“the one clean 06-06 reachability flip”). Live: `task_runner.rs:3801` region + `novelty.rs`.
- **Crate / module (spec):** `crates/orchestrator/src/novelty.rs`; Mode A/D hook in `grounding_mode.rs`; queue under `workflow/sessions/{id}/discovery-queue/`
- **Purpose of this file:** enough architecture to rebuild discovery-vs-search honesty labels. Not a status ledger.

## 1. What it is

**NoveltyClass** is a closed three-bucket label applied to every **Mode A or Mode D** result: `Search`, `DistantSearch`, `DiscoveryCandidate`. `DiscoveryCandidate` routes to a **human-review queue**. The system **never autonomously claims discovery** (`D-plan/D-15_novelty_class_labeling.md:1-7,139-141`, `ELAI-SYSTEMS-OVERVIEW.md:237`). This is Mode C in the five-mode taxonomy: a **label**, not a runtime grade (`D-plan/D-10_mode_routing_predicate.md:25-27`).

## 2. Why it exists

FunSearch (Nature 2023) can be read as discovery **or** as large-scale search scored by checkers. Both readings are defensible (`D-plan/D-15_novelty_class_labeling.md:16-28`). Engineering rule: use checker-scored search; **do not market autonomous LLM discovery**; distinguish “verified by checker” from “search produced this”. ARC-AGI-2 honesty: no current 7B path to open-ended novelty. If this organ is missing, Mode A Pass is billed as discovery. If it is faked (LLM self-label), the honesty layer is gone.

## 3. How it works (runtime)

1. **Who calls it.** After D-10 `dispatch_by_mode` Constructive **or** OperationalFalsification produces a `CheckerResult`, call `classify`, attach `NoveltyClass` to **`ClaimResult` metadata**, emit `NoveltyLabeled` (`D-plan/D-15_novelty_class_labeling.md:83-88`). SUP-007: attachment target is **ClaimResult metadata**, not a field on `CheckerResult` (`ELAI-EXHAUSTIVE-SUPER-FEATURE-LIST.md:3796`).
2. **Input.** `result: &CheckerResult`, `dkg: &SymbolGraph`, `mistake_book: &MistakeBook`, `session_archive: &ArchiveChain`.
3. **`classify` (deterministic heuristic, first match):**
   - Near-duplicate (cosine ≥ **0.85** via existing embedding pipeline) in DKG **OR** MistakeBook **OR** session archive → `Search`.
   - Else all primitives referenced in the result exist in DKG → `DistantSearch`.
   - Else any primitive / relationship **not** in DKG → `DiscoveryCandidate`.
   (`:73-81,310-328` real-problem §4.8)
4. **Empty DKG.** Return `DiscoveryCandidate` when no references known; **never panic** (`:164-165`).
5. **DiscoveryCandidate side effects.** Write `workflow/sessions/{id}/discovery-queue/{claim_id}.toml` **and** emit `DiscoveryCandidateQueued`. Queue file is mandatory, not SSB-only (`:88-89,168`).
6. **Human review.** `elai discovery review` v1 **prints the list** (TUI deferred). Status `Pending` → `RatifiedHuman` / `RejectedHuman`; those emit `DiscoveryRatified` / `DiscoveryRejected` (`:97-110,161-162`).
7. **The model may generate** the candidate text being labeled. It may **not** pick `NoveltyClass`, auto-publish DiscoveryCandidate, or skip the queue write.

Live MVP (`novelty.rs:35-50`) ships a pure predicate over caller-supplied scalars and **defers** live DKG embeddings + queue files — a **partial cousin**. Rebuild the plan: embeddings + queue file are in-contract.

## 4. Types, enums, objects

Copy from `D-plan/D-15_novelty_class_labeling.md:64-99`. Do not invent fields.

```
enum NoveltyClass {  // closed, L3-locked
  Search, DistantSearch, DiscoveryCandidate
}

struct NoveltyClassification {
  class: NoveltyClass,
  dkg_match_count: u32,
  near_duplicate_threshold_hit: bool,
  primitives_in_dkg: u32,
  primitives_missing: u32,
  evidence: BoundedString<2048>,
}

fn classify(result, dkg, mistake_book, session_archive) -> NoveltyClassification

fn near_duplicate_check(result_embedding: &[f32], dkg: &SymbolGraph, threshold: f32) -> bool
  // top-k nearest in DKG; default threshold 0.85

struct DiscoveryQueueEntry {
  claim_id: ClaimId,
  classified_at: DateTime<Utc>,
  result_hash: Blake3Hash,
  evidence: BoundedString<2048>,
  status: ReviewStatus,
}

enum ReviewStatus {
  Pending,
  RatifiedHuman { reviewer: String, ratified_at: DateTime<Utc> },
  RejectedHuman { reviewer: String, reason: String },
}
```

Exact enum values `Search`, `DistantSearch`, `DiscoveryCandidate` are load-bearing (`ELAI-EXHAUSTIVE-SUPER-FEATURE-LIST.md` D0-050).

## 5. Control flow / state machine

**Owner:** orchestrator after Mode A/D. Human owns ratification.

```
Mode A or D CheckerResult
  → classify
  → NoveltyLabeled
  → if DiscoveryCandidate
        write queue TOML
        DiscoveryCandidateQueued
        wait human
          RatifiedHuman → DiscoveryRatified
          RejectedHuman → DiscoveryRejected
     else Search / DistantSearch: no auto-discovery claim
```

**Forced:** no auto-publish on N reviewers (deferred; conservative default) (`:223-225`). v1 single-reviewer.

**Not** a `select_grounding_mode` return value.

## 6. Grounding, receipts, gates

Receipts: `NoveltyClassification` counts, blake3 `result_hash`, queue file, SSB. Cosine ≥ 0.85 is the near-duplicate gate. Embeddings **must** use the existing knowledge-crate pipeline; do not reinvent (`:166-167,112-116`).

**Never allowed as a confidence signal:** LLM “this is novel”; marketing copy; auto-promoting DiscoveryCandidate to a published discovery. Context text wants a “confidence” distinguishing checker-verified vs uncheckable search — **no confidence field exists on the types** (D004-D15-033). Rebuild without inventing one.

## 7. Budgets, limits, ceilings

| Knob | Value | Scope |
|---|---|---|
| cosine near-duplicate | 0.85 | classify / `near_duplicate_check` |
| evidence | `BoundedString<2048>` | classification + queue |
| observable | 10 mixed synthetic claims | 10 NoveltyLabeled; queue files for candidates |
| SCHEMA_VERSION (plan) | 60 → 61 | SSB |
| unit tests | ≥8 | acceptance |

`[novelty]` is named for cosine threshold + queue path (`:162`) but **no concrete defaults/TOML block** in the plan (D004-D15-023, D004-D15-035). Live `NoveltyConfig` exists in `novelty.rs` — cousin unless a later plan pins it.

## 8. Roles and neighbors

Consumes D-11 `CheckerResult`, C-17 DKG `SymbolGraph`, C-24 `symbol_impact`, D-1/C-21 MistakeBook, D-1 ArchiveChain, D-10 dispatch A/D. D-12 is a prerequisite (procedures landed) and is **not parallel-safe** with D-12 (shared hook). Parallel-safe with D-14.

Human reviewer via CLI. D-20 ARC harness must **not** auto-publish DiscoveryCandidate (`ELAI-EXHAUSTIVE-SUPER-FEATURE-LIST.md` D0-024).

**Must not fork:** 5-bucket or 2-bucket taxonomies; LLM novelty judge; attaching the enum only on CheckerResult (SUP-007 loser).

## 9. Config / flags / defaults

- **`ELAI_NOVELTY_LABEL_ENABLE`** — default-OFF (`ELAI-FEATURE-MAP.md:598-601`, `wire-dark-allowlist.toml:871-876`). Copy this flag.
- Plan names `elai_cc.toml [novelty]` (cosine threshold, queue path) without a filled section.
- NoveltyClass L3-locked as a closed enum (`:67-68`).

## 10. Implementation map

| Symbol | Spec path |
|---|---|
| `novelty.rs` | `NoveltyClass`, `classify`, `near_duplicate_check`, queue types |
| `grounding_mode.rs` | A/D post-result classify |
| `ssb.rs` 60→61 | four discovery/novelty variants |
| `lib.rs` | `pub mod novelty` |
| `orchestrator/tests/novelty_integration.rs` | 3-symbol mock DKG |
| `workflow/sessions/{id}/discovery-queue/{claim_id}.toml` | artifacts |
| `workflow/reasoning/D-15-decisions.md` | three decisions |
| papers | funsearch-nature-2023, alphaevolve-2506.13131, chollet-arc-1911.01547 |
| live `novelty.rs:74` | enum present; embeddings/queue deferred in-module |

## 11. Journal / SSB events

Plan SCHEMA_VERSION **60 → 61**:

```
NoveltyLabeled { claim_id, class: String, dkg_match_count: u32,
                 primitives_in_dkg: u32, primitives_missing: u32 }
DiscoveryCandidateQueued { claim_id, queue_path: String }
DiscoveryRatified { claim_id, reviewer: String }
DiscoveryRejected { claim_id, reviewer: String, reason: String }
```

(`:103-109`). D-SUMMARY: `NOVELTY_LABELED`, `DISCOVERY_CANDIDATE_QUEUED` (`D-plan/D-SUMMARY_INDEX.md:112`).

## 12. Tests and observables

| Case | Assert |
|---|---|
| near-dup embedding in DKG | Search |
| all primitives in DKG, no near-dup | DistantSearch |
| novel primitive | DiscoveryCandidate + queue file |
| `near_duplicate_check` threshold | 0.85 behavior |
| empty DKG | DiscoveryCandidate, no panic |
| integration mock 3 symbols | all three classes; novel writes queue |
| cloud A | near-dup primitive → Search row |
| cloud B | novel composition of known primitives → DistantSearch |
| cloud C | not-in-DKG primitive → DiscoveryCandidate + TOML |

≥8 tests. Three paper interfaces.

## 13. Rebuild contract

Another harness has **this** organ iff:

- Closed `NoveltyClass { Search, DistantSearch, DiscoveryCandidate }`.
- Applied to every Mode A **and** Mode D `CheckerResult`, attached to **ClaimResult metadata**.
- Cosine ≥ 0.85 near-dup → Search; all-known primitives → DistantSearch; else DiscoveryCandidate.
- DiscoveryCandidate **writes a queue file** and waits for human ratify/reject. No autonomous discovery claim.
- Empty DKG → DiscoveryCandidate, not panic.
- Embeddings from existing knowledge pipeline, not a new model stack.
- `ELAI_NOVELTY_LABEL_ENABLE` default-OFF isolation allowed; when on, A/D results must be labeled.

An LLM “novelty score” or a two-bucket familiar/novel cousin is not this organ.

## 14. Non-goals / banned designs

- TUI for `elai discovery review` (deferred; v1 prints list).
- Multi-reviewer ratification (deferred).
- Auto-publish on N agreements (deferred; safety default conservative) (`:220-225`).
- Reinventing embeddings.
- Treating Mode C as a `select_grounding_mode` branch.

## 15. Source citations

Primary: `D-plan/D-15_novelty_class_labeling.md` (full file). Index: `D-plan/D-SUMMARY_INDEX.md:110-114`. Feature map: `ELAI-FEATURE-MAP.md:598-601`. Systems: `ELAI-SYSTEMS-OVERVIEW.md:233-237`. Exhaustive: D0-019, D0-050, D004-D15-001..038, SUP-007. Live: `novelty.rs:1-90`, `task_runner.rs` novelty hook, `wire-dark-allowlist.toml:871-876`.

## 16. Gaps

Label **UNCERTAIN**; do not fill:

- `classify` takes `CheckerResult`, but D-11 result has no candidate/embedding/primitives (D004-D15-032).
- No confidence field despite context prose (D004-D15-033).
- v1 “prints list” vs ratify/reject mutation, auth, persistence (D004-D15-034).
- `[novelty]` defaults, lock, loader (D004-D15-035).
- Primitive extraction, relationship match, top-k, embedding model/version, exact 0.85 boundary (D004-D15-036).
- Queue atomicity, idempotency, permissions, retention (D004-D15-037).
- `near_duplicate_check` signature searches only SymbolGraph; classify also needs MistakeBook + ArchiveChain adapters (D004-D15-038).
- Live MVP defers queue files + live embeddings (`novelty.rs:43-50`) vs plan contract.

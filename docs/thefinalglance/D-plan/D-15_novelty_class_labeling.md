# D-15: Discovery-vs-search labeling (NoveltyClass)

Objective: every Mode A or Mode D result gets a NoveltyClass label
{Search, DistantSearch, DiscoveryCandidate}; DiscoveryCandidate routes to
human-review queue. Realizes the FunSearch-vs-critics contradiction
resolution (real-problem §1 Mode C; §4.8 lines 310-328) as honesty layer —
system never autonomously claims discovery; human ratifies.
Estimated effort: 1-2 sessions.
Pre-requisites: D-11 (Mode A produces results to label), D-12 (procedures
  landed), C-17 (DKG resilience for near-duplicate detection). spec-grounder
  verifies.
Parallel-safe with: D-14 (sycophancy probe; orthogonal).

## Context

Real-problem §1 Mode C (lines 68-89). The grounded contradiction:
- FunSearch (Nature 2023, T1) — genuine discovery (cap-set bounds, bin-packing)
- FunSearch critics (Davis NYU T2, Marcus T3, Mouret hedged T1) — large-scale
  SEARCH through program spaces evaluated by automated checkers.

Both readings defensible. Engineering-actionable position:
1. Don't market autonomous LLM "discovery" — too contested.
2. DO use FunSearch / AlphaEvolve pattern for novel-output problems where
   scorer is writable.
3. Mark novel claims with confidence distinguishing "verified by checker"
   from "search produced this, no checker available".
4. Recognize ARC-AGI-2 24% vs 60-70% human: no current 7B path to closing
   genuine open-ended novelty. Be honest.

Real-problem §4.8 (lines 310-328): NoveltyClass enum + heuristic for labeling:
- **Search**: result has near-duplicate in DKG / MistakeBook / prior session
  archive.
- **DistantSearch**: result is novel COMPOSITION of known primitives, each
  in DKG.
- **DiscoveryCandidate**: result references primitives or relationships NOT
  in DKG; flag for human review.

Partial foundation present and usable:
- C-17 DKG resilience + Kotlin grammar + scale benchmarks.
- C-24 symbol_impact tool catalog (DKG query).
- MistakeBook (D-1 + C-21).
- CheckerResult (D-11) is the labeling target.

## Reads

- F:\.ELAI_workflow\docs\specs\guide\real-problem-solving-grounded-2026-05-19.md
  (§1 Mode C contradiction; §4.8 NoveltyClass — lines 68-89 + 310-328)
- F:\.ELAI_workflow\docs\specs\guide\reasoning-improvement-grounded-2026-05-19.md
  (§1.16 ARC-AGI-2 honesty)
- .elai_cc/crates/orchestrator/src/checker_registry.rs (D-11 CheckerResult)
- .elai_cc/crates/knowledge/src/symbol_graph/ (C-17 DKG types)

## Paper-ingestor pre-step

Spawn paper-ingestor with mode=paper for:
- FunSearch Nature 2023 doi:10.1038/s41586-023-06924-6
- arxiv:2506.13131 AlphaEvolve
- arxiv:1911.01547 Chollet On the Measure of Intelligence (ARC framing)

Verify workflow/papers/{funsearch-nature-2023,alphaevolve-2506.13131,chollet-
arc-1911.01547}-interface.md.

## Tasks

1. **NoveltyClass enum** in new file
   .elai_cc/crates/orchestrator/src/novelty.rs:
   - `enum NoveltyClass { Search, DistantSearch, DiscoveryCandidate }`
     (closed enum; L3-locked)
   - `struct NoveltyClassification { class: NoveltyClass, dkg_match_count:
     u32, near_duplicate_threshold_hit: bool, primitives_in_dkg: u32,
     primitives_missing: u32, evidence: BoundedString<2048> }`

2. **Heuristic labeling** in novelty.rs:
   - `fn classify(result: &CheckerResult, dkg: &SymbolGraph, mistake_book:
     &MistakeBook, session_archive: &ArchiveChain) -> NoveltyClassification`:
     - If result has near-duplicate (cosine similarity ≥ 0.85 via existing
       embedding pipeline) in DKG OR MistakeBook OR session_archive →
       Search.
     - Else if all primitives referenced in result are in DKG (e.g. all
       function-symbols used in result candidate exist in DKG) → DistantSearch.
     - Else (result references primitives NOT in DKG) → DiscoveryCandidate.

3. **Mode A/D integration** in
   .elai_cc/crates/orchestrator/src/grounding_mode.rs (D-10):
   - dispatch_by_mode Constructive AND OperationalFalsification branches:
     after CheckerResult is determined, call NoveltyClassification.classify;
     attach NoveltyClass to ClaimResult metadata; emit NoveltyLabeled SSB row.
   - If DiscoveryCandidate → enqueue to workflow/sessions/{id}/discovery-
     queue/; emit DiscoveryCandidateQueued SSB row.

4. **Discovery review queue**:
   - Directory: workflow/sessions/{id}/discovery-queue/{claim_id}.toml
     (per-claim; human reviewer ratifies or rejects).
   - `struct DiscoveryQueueEntry { claim_id: ClaimId, classified_at:
     DateTime<Utc>, result_hash: Blake3Hash, evidence: BoundedString<2048>,
     status: ReviewStatus }`
   - `enum ReviewStatus { Pending, RatifiedHuman { reviewer: String,
     ratified_at: DateTime<Utc> }, RejectedHuman { reviewer: String,
     reason: String } }`
   - CLI: `elai discovery review` — opens TUI for pending entries (deferred;
     v1 prints list).

5. **SSB action types** in ssb.rs (SCHEMA_VERSION 60→61):
   - `NoveltyLabeled { claim_id: ClaimId, class: String, dkg_match_count: u32,
     primitives_in_dkg: u32, primitives_missing: u32 }`
   - `DiscoveryCandidateQueued { claim_id: ClaimId, queue_path: String }`
   - `DiscoveryRatified { claim_id: ClaimId, reviewer: String }` (emitted by
     human-review CLI flow)
   - `DiscoveryRejected { claim_id: ClaimId, reviewer: String, reason: String }`

6. **Cosine-similarity helper** in novelty.rs:
   - Uses existing knowledge crate's embedding pipeline (per C-17); if not
     directly callable from orchestrator, use a stub interface and document
     the dependency.
   - `fn near_duplicate_check(result_embedding: &[f32], dkg: &SymbolGraph,
     threshold: f32) -> bool`: top-k nearest in DKG; threshold default 0.85.

7. **Unit tests** in novelty.rs:
   - classify: near-duplicate in DKG → Search (synthetic embeddings).
   - classify: all primitives in DKG but no near-duplicate → DistantSearch.
   - classify: novel primitive (not in DKG) → DiscoveryCandidate.
   - near_duplicate_check thresholding correct.

8. **Integration test** in
   .elai_cc/crates/orchestrator/tests/novelty_integration.rs:
   - Mock DKG with 3 known symbols + 3 known embeddings.
   - Test claim with near-duplicate embedding → Search.
   - Test claim with novel composition of known primitives → DistantSearch.
   - Test claim with novel primitive → DiscoveryCandidate + queue entry created.

9. **Decision doc** workflow/reasoning/D-15-decisions.md (3 decisions):
    - (a) Why 3-bucket classification (not 5 or 2): real-problem §4.8
      UC-4 — preserves engineering capability while flagging honest novelty;
      coarse is OK for human routing.
    - (b) Why cosine similarity threshold 0.85 (not 0.9 or 0.7):
      empirically tuned for natural-language + code embeddings; balances
      false-positive (over-classify as Search) vs false-negative (over-classify
      as DiscoveryCandidate).
    - (c) Why human review for DiscoveryCandidate (not auto-publish): per
      real-problem §1 Mode C — "the system never claims discovery
      autonomously; surfaces candidates for human ratification."

## WIRING_CONTRACT

produces_symbols:
  - NoveltyClass (closed enum) + NoveltyClassification
  - classify + near_duplicate_check
  - DiscoveryQueueEntry + ReviewStatus
  - SsbActionType::NoveltyLabeled + ::DiscoveryCandidateQueued +
    ::DiscoveryRatified + ::DiscoveryRejected + payloads
  - SCHEMA_VERSION bump 60→61
  - workflow/sessions/{id}/discovery-queue/ layout
consumes_symbols:
  - CheckerResult (D-11)
  - DKG SymbolGraph (C-17)
  - MistakeBook (D-1 / C-21)
  - ArchiveChain (D-1)
  - GroundingMode dispatch_by_mode (D-10)
reachable_from:
  - D-10 dispatch_by_mode (Mode A / Mode D branches)
  - human reviewer via `elai discovery review` CLI (deferred TUI)
  - elai_cc.toml [novelty] section (cosine threshold, queue path)
stub_ban:
  - classify MUST handle empty DKG (return DiscoveryCandidate when no
    references known; not panic)
  - near_duplicate_check MUST use existing knowledge embeddings; not
    reinvent
  - DiscoveryCandidateQueued MUST write file to queue path; not just SSB
observable_effect:
  trigger: 10 synthetic claims of mixed novelty
  expect_ssb: 10 NoveltyLabeled rows; some DiscoveryCandidateQueued
  expect_artifact: workflow/sessions/{id}/discovery-queue/*.toml present
                   for DiscoveryCandidate-classified claims

## DETERMINISTIC_CHECK

```bash
rg "SCHEMA_VERSION: i64 = 61" .elai_cc/crates/orchestrator/src/ssb.rs
rg "NoveltyLabeled|DiscoveryCandidateQueued|DiscoveryRatified|DiscoveryRejected" \
  .elai_cc/crates/orchestrator/src/ssb.rs | wc -l
rg "enum NoveltyClass|fn classify|fn near_duplicate_check" \
  .elai_cc/crates/orchestrator/src/novelty.rs
cargo check --workspace 2>&1 | tail -3
cargo test -p elai_orchestrator -- novelty 2>&1 | grep "test result"
test -f workflow/papers/funsearch-nature-2023-interface.md && \
  test -f workflow/papers/alphaevolve-2506.13131-interface.md && \
  test -f workflow/papers/chollet-arc-1911.01547-interface.md && echo OK
```

## ACCEPTANCE

- [ ] SCHEMA_VERSION_PUBLIC == 61
- [ ] 4 SSB variants round-trip
- [ ] NoveltyClassification + 3-bucket logic correct on synthetic embeddings
- [ ] near_duplicate_check correct on canonical inputs
- [ ] Discovery queue file creation works
- [ ] cargo check --workspace: zero new errors
- [ ] cargo test -p elai_orchestrator -- novelty: all pass, ≥8 tests
- [ ] Cloud smoke test A: claim with near-duplicate primitive in DKG →
      NoveltyClass::Search SSB row
- [ ] Cloud smoke test B: claim with novel composition of known primitives →
      NoveltyClass::DistantSearch SSB row
- [ ] Cloud smoke test C: claim referencing primitive NOT in DKG →
      NoveltyClass::DiscoveryCandidate + queue entry file created
- [ ] All 3 paper-ingestor outputs present
- [ ] Decision doc workflow/reasoning/D-15-decisions.md

## Files touched

- ADD: .elai_cc/crates/orchestrator/src/novelty.rs
- MODIFY: .elai_cc/crates/orchestrator/src/ssb.rs (SCHEMA_VERSION 60→61;
    4 new variants)
- MODIFY: .elai_cc/crates/orchestrator/src/grounding_mode.rs (D-10 wiring)
- MODIFY: .elai_cc/crates/orchestrator/src/lib.rs (pub mod novelty)
- ADD: .elai_cc/crates/orchestrator/tests/novelty_integration.rs
- ADD: workflow/reasoning/D-15-decisions.md
- ADD: workflow/papers/{funsearch-nature-2023,alphaevolve-2506.13131,
       chollet-arc-1911.01547}-interface.md

## Out-of-scope

- TUI for discovery-review CLI (DEFERRED; v1 prints list)
- Multi-reviewer ratification (DEFERRED; single-reviewer first)
- Auto-publish on N reviewer agreements (DEFERRED; out of scope; safety
  default is conservative)

## Next

D-16 (UniCR conformal calibration) is next; the formal Mode E refusal.

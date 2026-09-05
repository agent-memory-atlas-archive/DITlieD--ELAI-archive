D-15 — Discovery-vs-search NoveltyClass labeling — closure decisions
====================================================================

Plan: F:/.ELAI_workflow/docs/thefinalglance/D-plan/D-15_novelty_class_labeling.md
Re-check: F:/.ELAI_workflow/workflow/sessions/s-25120/cross-check-recheck/D-15_novelty_class_labeling-report.md
Session: s-25120

Status at closure entry: PARTIAL (substrate shipped at SCHEMA 83;
re-check found 11 MISSING + 1 NOT_WIRED of 20 claims, with the
single-variant consolidation precedent rejecting most of the missing
SSB variants).

Status at closure exit: SHIPPED for the orchestrator-scoped MVP
(production dispatch wire + CheckerResult projection + L3-locked toml
+ runtime config + TS mirror + 5 contract tests + 5 integration
tests). Plan-deferred items recorded under KNOWN-RESIDUAL.

DECISION-1 — NO SCHEMA bump
---------------------------
Plan §3.5 calls for `SCHEMA_VERSION 60 -> 61` for 4 new SSB variants.
Substrate already ships at SCHEMA 83 with a single consolidated
`NoveltyClassified` variant (ssb.rs:7752 + 13308). Current head is
SCHEMA 125 (ssb.rs:1424). The D-15 closure does NOT bump the schema
— the substrate already exists, the closure only wires production
dispatch + projection + runtime config. Contract floor is set at >=
83 (the shipped baseline).

DECISION-2 — Single NoveltyClassified preserved
-----------------------------------------------
Plan §3.5 proposed 4 SSB variants:
  NoveltyLabeled / DiscoveryCandidateQueued / DiscoveryRatified /
  DiscoveryRejected
The shipped substrate has ONE: `SsbActionType::NoveltyClassified` +
`NoveltyClassifiedPayload` (template_id, class, dkg_match_count,
near_duplicate_threshold, ts_ms). The 4-way split is REJECTED at
closure per the D-10/D-11/D-12/D-13/D-14 single-variant
consolidation precedent — one row family per producer surface,
class enum carries the verdict variant.

DECISION-3 — 3-arm NoveltyClass preserved
-----------------------------------------
The shipped enum is `{Search, DistantSearch, DiscoveryCandidate}`
(novelty.rs:74, L3-locked per arxiv:2406.05761 §4 empirical
3-bucket motivation + real-problem-solving §4.8 lines 310-328).

Plan vocab "CoreKnowledge / Acquired / Derivative" (Chollet
arXiv:1911.01547 framing) NOT adopted at closure — Chollet's
taxonomy describes BENCHMARK-TASK novelty (ARC-AGI problem-level
generalization), whereas ELAI labels CLAIM-level novelty produced
by reasoning templates. The 2406.05761 3-bucket is the directly
grounded design; the Chollet citation in the plan motivates the
ARC-AGI-2 honesty layer (real-problem §1 Mode C), not the
classifier-side enum.

DECISION-4 — classify signature preserved
-----------------------------------------
The shipped predicate is:

  classify(_template: &ReasoningTemplate, _archive: &ArchiveChain,
           evidence: NoveltyEvidence, config: NoveltyConfig)
    -> NoveltyClass

Plan §3.2 proposed:

  classify(result: &CheckerResult, dkg: &SymbolGraph,
           mistake_book: &MistakeBook, session_archive: &ArchiveChain)
    -> NoveltyClassification

The plan's signature is REJECTED at closure (no Mistake_book or
SymbolGraph in the predicate scope — the predicate must stay pure
and dyn-compatible). The bridging path is the NEW
`novelty_evidence_from_checker_result` projection function added
in this closure — callers with a `CheckerResult` produce a
`NoveltyEvidence` bundle, then call the shipped predicate.

DECISION-5 — Production dispatch in agent_pool/task_runner
-----------------------------------------------------------
Plan §3.3 wires dispatch in `grounding_mode.rs::dispatch_by_mode`.
The 5-precedent (D-10 / D-11 / D-12 / D-13 / D-14) places production
dispatch in `agent_pool.rs` + `task_runner.rs::drive_task` as
post-task hooks gated on a runtime + env var. D-15 closure follows
the 5-precedent: `dispatch_novelty_hook` lives in `task_runner.rs`
(layer-6 after `dispatch_sycophancy_probe_hook`), env-gated on
`ELAI_NOVELTY_LABEL_ENABLE=1`, runtime field
`ResolvedConfig.novelty_runtime: Option<Arc<NoveltyRuntimeState>>`.
Default is None (no-op).

DECISION-6 — discovery_candidate_threshold = 0.20 is implementation choice
--------------------------------------------------------------------------
The `[novelty].discovery_candidate_threshold = 0.20` default is an
ELAI calibration heuristic — NOT directly grounded in any cited
paper.

  * arXiv:2506.13131 (AlphaEvolve) — paper has NO scored novelty
    variable. AlphaEvolve uses MAP-Elites binning (Cully &
    Demarteau 2018) as a behavioural-diversity proxy; the "novelty"
    a MAP-Elites cell tracks is empty-bin first-population, not a
    cosine-similarity score. Citing arXiv:2506.13131 as the
    threshold ground would be a mis-citation.

  * Nature 2023 FunSearch — paper requires expert review (the
    authors and named domain experts) to declare a result a
    "discovery". Automated cosine-similarity is explicitly NOT a
    substitute. ELAI uses the cosine score as an automated
    heuristic AND tags the row with the "automated heuristic"
    marker; the human-review queue (D-27 virtual queue) is the
    real verdict surface.

  * arXiv:2406.05761 (LLM Novelty Detection) — provides the
    3-bucket EMPIRICAL motivation but does NOT publish a specific
    discovery-candidate cosine threshold (the paper's
    classification head is task-fine-tuned, not threshold-based).

The 0.20 default is therefore documented in the toml stanza as an
"implementation choice" so future operators can tune without
treating it as paper-locked. The L3-LOCK applies to the field
NAME + type, not to the default scalar.

DECISION-7 — Paper-finding labels (per D-13 DECISION-8 framing)
---------------------------------------------------------------
Each cited paper carries a finding label so future re-checks know
what is paper-grounded vs ELAI-extended:

  * FunSearch (Nature 2023) — finding: "automated novelty
    classifier ABSENT in paper; expert review is the verdict
    surface". ELAI's cosine-similarity proxy is an automated
    heuristic, NOT a paper-defined classifier.

  * AlphaEvolve (arXiv:2506.13131) — finding: "NO scored novelty
    variable in paper; MAP-Elites bin-population proxy is the
    closest analog". ELAI does not use MAP-Elites at the
    classifier surface (the 3-bucket taxonomy is a different
    abstraction); the citation grounds the BROADER
    discovery-vs-search framing, not the threshold values.

  * Chollet (arXiv:1911.01547) — finding: "framework taxonomy
    describes BENCHMARK-task novelty". The ARC-AGI Measure of
    Intelligence treats novelty at the problem-distribution level
    (developer-aware, experience-relative); ELAI's 3-arm bucket
    labels CLAIM-level novelty. The Chollet framing grounds the
    real-problem §1 Mode C honesty layer (system never
    autonomously claims discovery), not the per-claim classifier.

  * LLM Novelty Detection (arXiv:2406.05761) — finding: "3-bucket
    classification (familiar / recombination / OOD) is the
    directly-cited empirical motivation". The 3-arm enum
    (`Search / DistantSearch / DiscoveryCandidate`) is the ELAI
    rename of the paper's labels for our naming conventions; the
    bucket count and motivation are paper-grounded.

DECISION-8 — Plan-deferred items recorded as KNOWN-RESIDUAL
-----------------------------------------------------------
The following plan-named scope is NOT shipped in the D-15 closure
and is intentionally deferred:

  KR-1: queue-file writer for
        workflow/sessions/{id}/discovery-queue/{claim_id}.toml
        — D-27 virtual queue handles this surface; the
        physical-file writer is not needed and would conflict
        with the D-27 storage backend.

  KR-2: 3 plan-named SSB variants (DiscoveryCandidateQueued,
        DiscoveryRatified, DiscoveryRejected, ReviewQueued) —
        rejected per DECISION-2 single-variant precedent. If
        future plan work introduces a review-flow producer
        surface, it lands as a separate ssb variant cluster, not
        as a NoveltyClass extension.

  KR-3: ReviewStatus enum {Pending, RatifiedHuman, RejectedHuman}
        — review-flow vocabulary not modelled at closure; the
        D-27 virtual queue surface owns this if it lands.

  KR-4: near_duplicate_check(result_embedding: &[f32],
                              dkg: &SymbolGraph,
                              threshold: f32) -> bool — deferred
        per the novelty.rs module-level out-of-scope list. The
        MVP accepts a pre-computed `near_duplicate_threshold`
        scalar in `NoveltyEvidence` supplied by the caller. The
        live embedding pipeline lands in a follow-up patch when
        the knowledge crate's embedding callable surfaces.

  KR-5: CLI subcommand `elai discovery review` — deferred
        (planned as a v1 TUI; the dashboard
        `DiscoveryQueueDrawer.svelte` provides the equivalent
        surface today).

Files touched (closure)
-----------------------
- .elai_cc/crates/orchestrator/src/runtime_config.rs (added
  NoveltyRuntimeConfig + NoveltyRuntimeState +
  ELAI_NOVELTY_LABEL_ENABLE_ENV + 5 tests + 1 error enum)
- .elai_cc/crates/orchestrator/src/novelty.rs (added
  novelty_evidence_from_checker_result + 2 unit tests)
- .elai_cc/crates/orchestrator/src/task_runner.rs (added
  ResolvedConfig.novelty_runtime field + dispatch_novelty_hook
  + TaskRunnerNoveltyEmitter adapter + drive_task wire call)
- .elai_cc/crates/orchestrator/tests/novelty_integration.rs
  (added 5 new tests + 1 sync runtime-config test)
- .elai_cc/crates/contracts/tests/novelty_contract.rs (NEW;
  5 contract tests)
- .elai_cc/elai_cc.toml (added [novelty] L3-locked commented
  stanza with 5 documented knobs)
- .elai_cc/crates/dashboard-v2/src/lib/api/types.ts (added
  NoveltyClass + NoveltyClassifiedRow + NoveltyDistribution
  shared mirrors)
- workflow/reasoning/D-15-decisions.md (this file)

Out of scope (explicit)
-----------------------
- novelty.rs core impl (660+ LOC shipped at SCHEMA 83 baseline)
- SsbActionType::NoveltyClassified variant (shipped)
- commands_d15.rs + NoveltyPanel.svelte (shipped)
- DiscoveryQueueDrawer.svelte (D-27 shipped)
- queue-file writer (D-27 virtual queue owns this)

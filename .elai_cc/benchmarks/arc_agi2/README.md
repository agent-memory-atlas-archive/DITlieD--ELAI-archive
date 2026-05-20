D-20 ARC-AGI-2 OOD probe harness — genuine-novelty gating fixture
==================================================================

Purpose
-------

The ARC-AGI-2 benchmark (arcprize.org) is designed to resist both
training-data memorization and search-style retrieval. ELAI's D-20
runner does NOT attempt to "solve" ARC tasks; instead it gates the
ORCHESTRATOR-side honesty stack against ARC-style claims:

  (a) does D-10's `select_grounding_mode` predicate keep ARC claims
      out of `Constructive` (which assumes a deterministic checker
      exists; ARC explicitly resists that)?
  (b) does D-15's `classify` route ARC claims to either
      `DiscoveryCandidate` (genuinely novel structure) or
      `DistantSearch` (recombination of known primitives)?
  (c) does the runner refuse to auto-publish any DiscoveryCandidate
      (real-problem §1 Mode C honesty layer; human ratification only)?

This shifts the gate from "solve rate" (which would creep upward as
the underlying cloud model improves, masking framework regressions)
to "does ELAI's mode-routing + novelty-classifier behave honestly on
out-of-distribution claims" — measurable, framework-anchored.

Fixture format
--------------

`arc_mini_fixture.jsonl` — one JSON object per line, schema given by
[`crate::benchmarks::arc_agi2_runner::FixtureRow`]:

  task_id                  — ARC-AGI-2 task identifier (synthetic for
                             the mini fixture; real fixtures use the
                             arcprize.org task IDs).
  claim_text               — natural-language framing of the task; the
                             runner threads this through D-10's
                             routing predicate and D-15's classifier.
  claim_type               — snake-case `ClaimType` (almost always
                             `factual` or `hypothetical` for ARC).
  expected_novelty_class   — snake-case `NoveltyClass`
                             (`search` | `distant_search` |
                             `discovery_candidate`). For ARC the
                             expected distribution skews toward
                             `discovery_candidate` (genuine novelty)
                             and `distant_search` (recombination);
                             `search` should be rare and signals
                             contamination if it dominates.
  dkg_match_count          — synthetic DKG-overlap value the D-15
                             classifier sees. Mirrors the production
                             caller's evidence bundle.
  near_duplicate_threshold — synthetic cosine similarity vs the
                             archive centroid (0.0 = maximally
                             novel; 1.0 = literal match).

Honestly evaluated definition
-----------------------------

A fixture row is "honestly evaluated" iff D-15 classifies it as
`DiscoveryCandidate` AND D-10 routes it to one of
  - `FalsifiabilityCrossFamily`
  - `OperationalDeferred`
  - `ConformalRefusal`
(i.e. NOT `Constructive` — Constructive on an ARC claim is the
"memorization gaming" pattern: the orchestrator pretended a
deterministic checker existed for an OOD task).

Memorization-gaming detection
-----------------------------

If the row is classified `Search` AND routed to `Constructive`, that
is the explicit "memorization gaming" signature. The aggregate report
flags `GameWithMemorization` when the search-class mode-routing
concentration on `Constructive` exceeds the configured threshold
(default: > 60% of `Search`-class rows land on `Constructive`,
suggesting the model is leaking memorized solutions through what
should be a refusal path).

Mini fixture scope
------------------

The bundled `arc_mini_fixture.jsonl` (20-30 rows) is balanced across
the 3 `NoveltyClass` values + various claim types so the runner can
exercise every branch of the verdict logic in a single run. Real
fixtures (full ARC-AGI-2 evaluation subset) live alongside under
`fixtures/` (deferred — costs cloud Sonnet budget; see D-20 plan
out-of-scope).

Grounding
---------

  - arXiv:2412.04261 — ARC-AGI-2 dataset description (deferred
    paper-ingestor follow-up; the citation is dropped in module
    docs per the anti-citation-rot rule until verified).
  - docs/specs/guide/real-problem-solving-grounded-2026-05-19.md
    §1 Mode C — honest evaluation framing.
  - D-10 GroundingMode taxonomy (crates/orchestrator/src/grounding_mode.rs).
  - D-15 NoveltyClass classifier (crates/orchestrator/src/novelty.rs).

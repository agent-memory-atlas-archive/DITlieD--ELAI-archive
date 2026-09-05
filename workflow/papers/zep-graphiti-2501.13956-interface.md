Algorithm: Zep / Graphiti Temporal Knowledge Graph for Agent Memory
ArXiv: 2501.13956
Paper: Zep: A Temporal Knowledge Graph Architecture for Agent Memory

VERIFICATION STATUS: VERIFIED
The arXiv ID resolves to "Zep: A Temporal Knowledge Graph Architecture for Agent Memory"
by Preston Rasmussen et al. (Zep AI). Confirmed via arxiv.org/abs/2501.13956 and
ar5iv.labs.arxiv.org/html/2501.13956. Published January 2025.

TOKEN COST CLAIM VERIFICATION (+68% narration vs hard-filter):
  STATUS: NOT FOUND IN PAPER. UNGROUNDED.
  The paper contains NO specific percentage claim about token overhead when narrating
  superseded facts as [OUTDATED] vs hard-filtering them. The paper states:
    "The reduced prompt size also led to significant latency cost reductions."
  and the average context token count drops from 115K to 1.6K (98.6% reduction by filtering
  the conversation history down to graph-retrieved facts). The +68% number cited in H-30's
  stub_ban does NOT appear in this paper. It is fabricated or drawn from an unidentified
  other source. H-30 MUST NOT cite 2501.13956 for this number.
  Executor note: if the +68% overhead claim is load-bearing for H-30's token-cost rationale,
  it requires a different citation or must be labeled "Ungrounded conjecture: +68% overhead".

BI-TEMPORAL SEMANTICS (verified, load-bearing):
  Four timestamps tracked per edge:
    t_created, t_expired in T_prime: when facts entered/left the SYSTEM (transaction time)
    t_valid, t_invalid in T: when facts held true in the WORLD (valid time)
  Invalidation mechanism (verbatim):
    "When the system identifies temporally overlapping contradictions, it invalidates the
    affected edges by setting their t_invalid to the t_valid of the invalidating edge."
  This is SOFT invalidation, not deletion. The old edge is preserved with t_invalid set.
  The close semantics: invalid_at (= t_invalid) is set to the valid_at (= t_valid) of the
  superseding fact. H-30's cited "invalid_at = valid_at" captures this correctly.

Public API (from paper architecture):
  Graphiti.add_episode | (episode: Episode) -> None | ingests conversation turn, extracts edges
  Graphiti.search | (query: str, timestamp: datetime) -> list[Edge] | retrieves valid-at-time facts
  Edge.t_valid | datetime | when fact became true in world
  Edge.t_invalid | datetime | when fact ceased to be true in world (None if still valid)
  Edge.t_created | datetime | when fact entered system
  Edge.t_expired | datetime | when fact was removed from system (None if still present)

Behavioral contracts:
  add_episode -> LLM extracts entity-relation-entity triples from episode text
  add_episode -> LLM compares new edges against semantically related existing edges for contradiction
  add_episode -> on contradiction: set t_invalid on old edge to t_valid of new edge
  search -> only returns edges where t_invalid IS NULL (or t_invalid > query_timestamp)
  search -> reduces context from ~115K tokens (full conversation) to ~1.6K tokens (graph-retrieved)

Benchmark results (Deep Memory Retrieval, DMR):
  Zep + gpt-4-turbo: 94.8% accuracy
  Zep + gpt-4o-mini: 98.2% accuracy
  Both outperform MemGPT and full-conversation baselines marginally

LongMemEval results:
  Accuracy improvement up to 18.5% vs baseline
  Response latency reduction: 90% vs baseline

Inputs:
  episode | text | conversation turn or document chunk | no length constraint cited
  query | text | retrieval query
  timestamp | datetime | point-in-time for valid-at query | optional, defaults to now

Outputs:
  edges | list[Edge] | retrieved facts valid at query time | t_invalid must be null or future
  contradiction_report | internal | which edges were invalidated by new episode

Hyperparameters:
  semantic_similarity_threshold | not specified in paper | used for contradiction detection
  LLM_model | any | used for extraction and contradiction comparison

Critical invariants:
  assert edge.t_invalid is None or edge.t_invalid > query_timestamp, "only return currently-valid facts"
  assert new_edge.t_invalid is None, "a newly added edge is valid until superseded"
  assert superseded_edge.t_invalid == superseding_edge.t_valid, "close semantics: non-destructive"
  assert old_edge is not deleted, "soft invalidation only; historical record preserved"

Failure modes cited:
  Full-conversation baseline inflates context to 115K tokens causing latency and cost blowup
  MemGPT baseline: reranker fails on long contexts; Zep marginal improvement only on DMR
  No explicit failure mode for the narration-vs-filtering tradeoff documented in this paper

Implementation pitfalls (do NOT do this):
  Do NOT delete superseded edges; use soft invalidation (set t_invalid only)
  Do NOT return edges where t_invalid < query_timestamp in search results
  Do NOT cite this paper for a "+68% token overhead" narration-vs-filtering claim; it is absent
  Do NOT confuse transaction time (t_created/t_expired) with valid time (t_valid/t_invalid)

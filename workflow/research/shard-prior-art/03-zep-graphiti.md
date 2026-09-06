# Prior-Art Dossier: Zep / Graphiti

Pillar: memory governance (temporal knowledge graph)
Prepared for: ELAI (.elai_cc) — comparison against the SHARD paper's memory governance subsystem
Research mode: oracle (grounded-by-default, primary-source fetches)
Date compiled: 2026-06-01
Primary sources fetched: 11 (arXiv abstract + full HTML, 2x api.github.com repo, 1x github contents, 1x github releases, 1x github tags, raw graphiti.py source, 3x PyPI JSON, getzep.com homepage + product page)

NOTE ON SOURCE TRUST: Zep (the hosted product) and Graphiti (the open-source engine) are distinct artifacts from the same company (Zep AI / getzep). The 2501.13956 paper describes the HOSTED Zep system as of Jan 2025; Graphiti the OSS library has evolved past the paper. Where the paper and the current code disagree, both are cited and the divergence is flagged.

---

## 1. Identity & provenance

Engine (open source):
- Name: Graphiti — "Build Real-Time Knowledge Graphs for AI Agents" [T1: api.github.com/repos/getzep/graphiti]
- Repo: github.com/getzep/graphiti — created 2024-08-08, last push 2026-05-21 [T1]
- License: Apache-2.0 [T1: GitHub API license.spdx_id]
- PyPI package: graphiti-core, latest stable 0.29.1, license Apache-2.0, requires Python >=3.10,<4 [T1: pypi.org/pypi/graphiti-core/json]
- graphiti-core 0.29.1 uploaded 2026-05-21 (matches GitHub release v0.29.1 published 2026-05-21T03:32:50Z). The PyPI release history jumps 0.17.x -> 0.29.1 (a version realignment, not 12 missing minors). GitHub tags show v0.30.0pre0..pre5 prereleases on top of v0.29.1. [T1: PyPI releases list + api.github.com/repos/getzep/graphiti/tags]
  - FLAGGED MISREAD (now corrected): an earlier WebFetch of the PyPI JSON inferred a "2025-01-16" upload date for 0.29.1 from a serial number; the actual distribution-file upload_time_iso_8601 for 0.29.1 is 2026-05-21, consistent with the GitHub release date. Do not trust serial-derived dates.

Hosted product (closed):
- Name: Zep — homepage tagline "Agent memory, at enterprise scale ... a governed Context Lake of context graphs, served in milliseconds." [T3-vendor: getzep.com]
- The getzep/zep repo (github.com/getzep/zep) is now "Examples, Integrations, & More" — description "Zep | Examples, Integrations, & More", created 2023-04-29, last push 2026-04-09, Apache-2.0, not archived, ~4,626 stars. This is NOT the engine; the engine moved to graphiti-core. [T1: api.github.com/repos/getzep/zep]

Client SDKs (PyPI):
- zep-cloud — latest 3.22.0, uploaded 2026-05-04. license and summary fields are null in PyPI metadata (FLAG: license-not-declared in package metadata; treat as proprietary client SDK unless the repo states otherwise). Python <4.0,>=3.9.0. [T1: pypi.org/pypi/zep-cloud/json]
- zep-python — latest 2.0.2, license null, summary null. Description distinguishes a zep-cloud SDK branch from an OSS-compatible branch; NO explicit deprecation notice, but it is the older client surface. [T1: pypi.org/pypi/zep-python/json]

Paper:
- arXiv:2501.13956 RESOLVES. Title verbatim: "Zep: A Temporal Knowledge Graph Architecture for Agent Memory" [T1: arxiv.org/abs/2501.13956]
- Authors: Preston Rasmussen, Pavlo Paliychuk, Travis Beauvais, Jack Ryan, Daniel Chalef (Zep AI team) [T1]
- Submitted: 2025-01-20. Affiliations not printed in the arXiv abstract block; authors are the Zep AI founding/eng team (Daniel Chalef is the founder). [T1 for names; T3 for org attribution via getzep]

Docs:
- help.getzep.com is the documented home (GitHub homepage field = help.getzep.com/graphiti). NOTE: the specific path help.getzep.com/docs/graphiti/getting-started/overview returned 404 at fetch time — the docs IA has moved; cite help.getzep.com root, not that deep path. [T1: 404 observed]

License/version mismatch flags:
- graphiti-core (engine): Apache-2.0, clean. [T1]
- zep-cloud / zep-python: PyPI license field null — license posture for the CLIENT SDKs is undeclared in registry metadata. If ELAI ever vendored a client, this needs a repo-level LICENSE check before use. For prior-art study (no vendoring) this is informational only.

---

## 2. Problem it solves

Specific failure mode targeted (from the paper's own framing, abstract verbatim): "existing retrieval-augmented generation (RAG) frameworks for LLM-based agents are limited to static document retrieval, [while] enterprise applications demand dynamic knowledge integration from diverse sources including ongoing conversations and business data." [T1: 2501.13956 abstract]

Concretely, the failure modes Zep/Graphiti attack:
1. Static RAG can't represent CHANGE over time — a fact retrieved today may have been superseded yesterday, and flat vector RAG has no notion of "this was true then, is false now." [T1: abstract + method]
2. Batch-recomputed knowledge graphs (e.g. Microsoft GraphRAG-style) require re-summarizing the whole corpus when data changes, which is infeasible for streaming agent memory. Graphiti's differentiator is real-time INCREMENTAL update without batch recomputation. [T1: repo description "Real-Time Knowledge Graphs"; T3-vendor: getzep "served in milliseconds"]
3. Long-conversation / cross-session synthesis: full-context stuffing balloons latency and tokens (paper reports ~115k-token baseline contexts). [T1: results section]

---

## 3. How it works — architecture + KEY MECHANISMS

This is the core deliverable. All component names below are quoted from either the paper method section (arxiv.org/html/2501.13956v1) [T1] or the live source graphiti_core/graphiti.py [T1].

### 3.1 Three-tier subgraph hierarchy (paper notation)
Knowledge is organized into three hierarchical subgraphs [T1: paper method]:
- Episode subgraph (G_e): episodic nodes hold raw input — "messages, text, or JSON". Episodic edges connect an episode to the semantic entities it references. This is the immutable, lossless, non-destructive ingestion layer.
- Semantic entity subgraph (G_s): entity nodes "extracted from episodes and resolved with existing graph entities"; semantic edges are the entity-to-entity relationships (the facts/triples).
- Community subgraph (G_c): community nodes are "clusters of strongly connected entities" carrying "high-level summarizations."

In the live code these map to node/edge classes imported by graphiti.py: EpisodicNode, EntityNode, CommunityNode, EntityEdge, EpisodicEdge, CommunityEdge — plus newer SagaNode / HasEpisodeEdge / NextEpisodeEdge for ordered episode grouping ("sagas"), which post-date the paper. [T1: graphiti.py imports]

### 3.2 Bi-temporal model (the governance core — most relevant to SHARD)
Every edge carries TWO independent timelines [T1: paper method, verbatim labels]:
- Timeline T (event timeline): "the chronological ordering of events" — when the fact was true in the real world.
- Timeline T' (transaction timeline): "the transactional order of Zep's data ingestion" — when the system learned/recorded the fact.

Four timestamps per edge [T1: paper, exact field semantics]:
- t_valid in T — start of the range during which the fact held true.
- t_invalid in T — when the fact stopped being true (event-time).
- t'_created in T' — when the fact was written into the system.
- t'_expired in T' — when the fact was invalidated in the system (transaction-time).

This bi-temporal structure lets Zep answer "what did we believe at time X" AND "what was actually true at time X" separately — point-in-time reconstruction along either axis. This is the mechanism ELAI/SHARD's staleness model should be compared against (see §7).

### 3.3 Temporal edge invalidation (the "governance" action)
Mechanism, verbatim: "When the system identifies temporally overlapping contradictions, it invalidates the affected edges by setting their t_invalid to the t_valid of the invalidating edge." [T1: paper]
- Contradiction detection is SEMANTIC: when a newly extracted fact contradicts an existing edge between the same entity pair, the old edge is not deleted — it is marked invalid (t_invalid set) and the new one becomes the live edge. History is preserved (non-destructive).
- Entity/fact dedup is constrained: fact extraction + deduplication uses hybrid search "constrained to edges existing between the same entity pairs" — i.e. contradiction search is localized to the relevant entity pair, not a global scan. [T1: paper]

### 3.4 Ingestion pipeline (add_episode)
Live method signature [T1: graphiti.py]:
`add_episode(name, episode_body, source_description, reference_time, source=EpisodeType.message, group_id=None, uuid=None, update_communities=False, entity_types=None, excluded_entity_types=None, previous_episode_uuids=None, edge_types=None, edge_type_map=None, custom_extraction_instructions=None, saga=None, saga_previous_episode_uuid=None)`
Pipeline steps (paper) [T1]:
1. NER over "current message content and the last n messages" (n=4) for context.
2. A "reflection technique inspired by reflexion to minimize hallucinations" — a self-critique pass on extracted entities to reduce spurious extractions.
3. Entity resolution / dedup: hybrid search (cosine similarity + full-text) to find candidate matches, then an LLM comparison to decide merge vs new. [T1]
4. Fact (edge) extraction + dedup: same hybrid approach, scoped to the entity pair. [T1]
5. Temporal extraction: assign t_valid / t_invalid; run the invalidation step (§3.3).
- `add_triplet(source_node, edge, target_node)` allows direct insertion of a single fact triple bypassing LLM extraction (structured-data path). [T1: graphiti.py]

### 3.5 Retrieval (search) — hybrid + rerank
Two public entry points [T1: graphiti.py]:
- `search(query, center_node_uuid=None, group_ids=None, num_results=DEFAULT_SEARCH_LIMIT, search_filter=None, ...)` — "basic hybrid search returning edges".
- `search_(query, config=COMBINED_HYBRID_SEARCH_CROSS_ENCODER, ..., bfs_origin_node_uuids=None, ...)` — advanced, fully configurable strategy.
Three search functions composed [T1: paper, verbatim names]:
- phi_cos — cosine semantic similarity over embeddings.
- phi_bm25 — BM25 full-text via "Neo4j's implementation of Lucene".
- phi_bfs — breadth-first search that "identifies additional nodes and edges within n-hops" of seed nodes (graph traversal, the part flat RAG lacks).
Rerankers [T1: paper]:
- Reciprocal Rank Fusion (RRF) — combine the three result lists.
- Maximal Marginal Relevance (MMR) — diversity.
- "Graph-based episode-mentions reranker that prioritizes results based on frequency of entity or fact mentions."
- Node distance reranker — re-rank by graph proximity to a center node (the `center_node_uuid` arg).
- Cross-encoder LLM reranking (the default `COMBINED_HYBRID_SEARCH_CROSS_ENCODER` config; live code has a `cross_encoder/` module). [T1: graphiti_core dir listing]

### 3.6 Community detection
Algorithm, verbatim: "label propagation algorithm" (explicitly NOT Leiden). Dynamic extension implements "the logic of a single recursive step in label propagation" — a newly added node is assigned "the community held by the plurality of its neighbors," so communities update incrementally without a full re-cluster. `build_communities()` and the `update_communities` flag on `add_episode` drive this. [T1: paper + graphiti.py]

### 3.7 Storage backends
graphiti_core ships pluggable drivers: the `driver/` module + optional deps for neo4j (>=5.26), falkordb, kuzu, neptune. LLM and embedder are also pluggable (`llm_client/`, `embedder/`, `cross_encoder/` modules; optional anthropic/google-genai/groq/voyageai/sentence-transformers). [T1: graphiti_core dir listing + PyPI requires_dist]

---

## 4. Core API / usage

Minimal real surface, taken from the live Graphiti class [T1: graphiti.py]:

```python
from graphiti_core import Graphiti
from graphiti_core.nodes import EpisodeType
from datetime import datetime, timezone

graphiti = Graphiti("bolt://localhost:7687", "neo4j", "password")
await graphiti.build_indices_and_constraints()

# Ingest an episode (conversation turn / text / JSON)
await graphiti.add_episode(
    name="user-msg-1",
    episode_body="Alice was promoted to VP of Engineering.",
    source_description="slack",
    reference_time=datetime.now(timezone.utc),
    source=EpisodeType.message,
    group_id="org-acme",          # partition / namespace
    update_communities=False,
)

# Hybrid retrieval (returns EntityEdge facts)
results = await graphiti.search(
    query="What is Alice's role?",
    center_node_uuid=None,        # optional: node-distance rerank anchor
    num_results=10,
)

await graphiti.close()
```

Other real methods on the class: `retrieve_episodes(reference_time, last_n=EPISODE_WINDOW_LEN, ...)`, `add_episode_bulk(...)`, `add_triplet(source_node, edge, target_node)`, `build_communities(...)`, `get_nodes_and_edges_by_episode(...)`, `remove_episode(...)`, `summarize_saga(...)`, and a `token_tracker` property. [T1: graphiti.py — all verbatim]

---

## 5. Measured numbers / benchmarks

ALL paper figures below are VENDOR-REPORTED (authors = Zep AI), self-run, not independently reproduced. Treat as the vendor's own eval. [T1: 2501.13956 results]

DMR (Deep Memory Retrieval — the MemGPT team's own benchmark):
- Zep 94.8% vs MemGPT 93.4% with gpt-4-turbo (+1.4pp, marginal). [T1]
- Zep 98.2% with gpt-4o-mini (MemGPT figure for that model not reported). [T1]
- Sample: 500 multi-session conversations, 5 sessions x up to 12 messages (~60 messages total). [T1]
- Authors' OWN stated DMR weaknesses (rare honesty, worth quoting): "single-turn, fact-retrieval questions that fail to assess complex memory understanding"; "poorly represents real-world enterprise use cases"; conversations "easily fitting within current LLM context windows." [T1]

LongMemEval (the harder benchmark, ~115k-token contexts):
- gpt-4o-mini: full-context baseline 55.4% -> Zep 63.8% (+15.2% relative), latency 31.3s -> 3.20s (~90% reduction). [T1]
- gpt-4o: full-context baseline 60.2% -> Zep 71.2% (+18.5% relative — the headline number), latency 28.9s -> 2.58s (~90% reduction). [T1]
- Token reduction: ~115k -> ~1.6k context tokens. [T1]
- Per-question-type (gpt-4o, Zep vs baseline) [T1]:
  - single-session-preference: 20.0% -> 56.7% (+184% relative — the standout)
  - temporal-reasoning: 45.1% -> 62.4% (+38.4%)
  - multi-session: 44.3% -> 57.9% (+30.7%)
  - (gpt-4o-mini also: single-session-user 81.4% -> 92.9%)
- Sample: question count not stated in the extracted results; conversations average ~115k tokens. (FLAG: exact N for LongMemEval not captured — LongMemEval is publicly ~500 questions but that number is from the LongMemEval paper, not 2501.13956; do not attribute it to Zep's paper.)

Vendor homepage benchmarks (NEWER, different benchmark, NOT in the paper — VENDOR-REPORTED, methodology link only): [T3-vendor: getzep.com + getzep.com/product/agent-memory]
- LoCoMo: 94.7% accuracy, retrieval latency 155-187ms, context ~5,760 tokens.
- Sub-200ms retrieval claimed across graph sizes "tested at 10K to 100M graph size."
- Comparison baseline on that page is labeled only "LongMemEval 90.2% / 162ms / 4,408 tokens" with the competing SYSTEM unnamed (FLAG: the product page does not name which competitor it benchmarks against, and does not state independent verification — low transparency; treat LoCoMo numbers as marketing-grade until a methodology doc is read).

Reproducibility verdict: DMR + LongMemEval numbers are vendor self-run on public benchmarks (reproducible in principle; not independently reproduced here). LoCoMo homepage numbers are marketing claims without a named baseline.

---

## 6. Maturity signals

- Graphiti stars: 26,820; forks 2,674; open issues 422 (as of 2026-05 fetch). [T1: GitHub API]
- Last engine push: 2026-05-21 (active); current stable v0.29.1, prereleases v0.30.0pre0..5 in flight. Release cadence is high (0.1 -> 0.29 across ~21 months). [T1: tags + releases]
- getzep/zep (examples/integrations repo): 4,626 stars, last push 2026-04-09, not archived. [T1]
- Company: Zep AI, Y Combinator W24 batch. [T2: LinkedIn "Zep AI (YC W24)"; T3: Tracxn/Crunchbase]
- Funding: seed-stage. One secondary source (Tracxn via search) states ~$500K seed from Y Combinator (Apr 2024) and ~$1M revenue by mid-2024; NO Series A found. (FLAG: funding figures are T3 aggregator data, not a press release — treat as approximate.) [T3: Tracxn/getlatka search snippet]
- Production users (named on vendor homepage — vendor claim, not independently verified): Twin Health, Praktika.ai, Thrive AI Health, AGI Inc, Harper, FlockX, plus logos for AWS, Samsung, Writer, HoneyBook, and an unnamed "Fortune 500 Tech Company." A pull-quote attributed to S&P Global Market Intelligence appears on the homepage. [T3-vendor: getzep.com]

Overall maturity verdict: the ENGINE (Graphiti) is genuinely mature OSS (27k stars, Apache-2.0, weekly-cadence releases, pluggable backends, broad LLM provider support). The COMPANY is early-stage (seed, YC W24). The PAPER is a vendor whitepaper, not peer-reviewed (arXiv preprint).

---

## 7. Versus SHARD's memory governance (temporal knowledge graph) pillar

Comparison is mechanism-by-mechanism. SHARD specifics referenced per the task brief; Zep/Graphiti specifics per §3 (all T1).

7.1 Staleness / temporal validity
- SHARD: staleness is a SCALAR formula, staleness = (now - last_verified) / expected_lifetime, with thresholds flag>1.0, quarantine>2.0. One axis (transaction-time only: last_verified is a wall-clock recency stamp). Action is threshold-driven quarantine.
- Zep/Graphiti: BI-TEMPORAL — two independent axes (event-time T via t_valid/t_invalid, transaction-time T' via t'_created/t'_expired). No scalar score; validity is a RANGE, and a fact's "currentness" is derived structurally (is t_invalid set? is the invalidating edge's t_valid <= now?).
- Verdict: Graphiti is RICHER on temporal modeling — it can answer "what was true at time X" vs "what we believed at time X" separately, which SHARD's single recency ratio cannot express. SHARD is SIMPLER and cheaper — a divide-and-threshold with no graph traversal, easy to reason about and audit, and it bakes in an explicit decay assumption (expected_lifetime) that Graphiti has no equivalent of. Graphiti has no built-in "expected lifetime / auto-decay" notion — invalidation is contradiction-driven, not time-decay-driven. So SHARD models DECAY; Graphiti models CONTRADICTION. These are complementary, not redundant: Graphiti would never quarantine a fact merely because it got old with no contradicting evidence, whereas SHARD would.

7.2 Conflict detection
- SHARD: per the brief, the repo implements only SYNTACTIC conflict detection (and DECLARE-REVIEW-EXECUTE coordination claims SEMANTIC conflict detection at the design level — a design/impl gap).
- Zep/Graphiti: SEMANTIC contradiction detection at ingestion — a newly extracted fact is compared (via hybrid search + LLM comparison) against existing edges between the same entity pair; on temporally-overlapping contradiction it invalidates the old edge. This is genuine semantic conflict resolution running in production code (the dedup + invalidation pipeline, §3.3-3.4).
- Verdict: Graphiti is STRICTLY AHEAD here in shipped capability — it has the semantic-contradiction mechanism that SHARD's design describes but its repo only stubs syntactically. The mechanism SHARD lacks: localized (entity-pair-scoped) LLM-judged contradiction with non-destructive invalidation. ELAI could borrow Graphiti's "scope contradiction search to the same subject-pair, then LLM-compare" pattern to make SHARD's semantic detection real without a global scan.

7.3 Coordination / concurrency
- SHARD: DECLARE-REVIEW-EXECUTE multi-stage coordination with conflict detection between concurrent operations.
- Zep/Graphiti: NO explicit multi-agent coordination protocol. Concurrency is handled at the data layer (group_id namespacing, episode ordering via reference_time, optimistic graph writes). There is no DECLARE/REVIEW gate — episodes are ingested and reconciled, not negotiated.
- Verdict: SHARD's coordination layer is a CAPABILITY GRAPHITI DOES NOT HAVE. Graphiti assumes a single logical writer reconciling a stream; SHARD targets multiple agents proposing changes that must be reviewed before commit. Different problem framing — SHARD's is the harder multi-writer problem.

7.4 Trust tiers / provenance
- SHARD: explicit T1-T4 skill trust tiers.
- Zep/Graphiti: NO trust-tier system on facts. Provenance is tracked structurally (episodic edges link every fact back to its source episode + source_description string, so you CAN see where a fact came from), but there is no graded trust score or tier gate. All extracted facts are first-class once they survive the reflexion + dedup pass.
- Verdict: SHARD has an explicit trust model Graphiti lacks; Graphiti has finer-grained SOURCE PROVENANCE (every fact traces to an episode) that SHARD's tier system abstracts away. Complementary: ELAI could layer T1-T4 tiers ON TOP of Graphiti-style episodic provenance.

7.5 Self-improvement / self-healing
- SHARD: design describes safety-constrained self-improvement (.bak rollback, 0-3 scoring) and composition.py cross-subsystem self-healing cascade — BUT per the brief the SHARD repo has NO self-improvement loop implemented.
- Zep/Graphiti: NO self-improvement loop AT ALL (not even by design). It is a memory substrate, not a self-modifying agent. The only "self-correction" is the reflexion pass during extraction (reduces hallucinated entities) and contradiction-driven invalidation — neither is a scored, rollback-guarded self-improvement cascade.
- Verdict: NEITHER ships self-improvement. SHARD at least has the DESIGN (rollback + scoring + cascade); Graphiti doesn't claim it. So on this axis SHARD's vision is more ambitious but both are unimplemented/absent in code. Not a differentiator in Graphiti's favor.

7.6 Retrieval sophistication (bonus axis where Graphiti clearly leads)
- Graphiti's hybrid retrieval (cosine + BM25 + n-hop BFS, fused via RRF/MMR + cross-encoder + node-distance + episode-mention rerankers) is a substantially more developed retrieval stack than a temporal-KG governance pillar typically includes. If SHARD's TKG retrieval is simpler (e.g. embedding-only or single-strategy), Graphiti is the reference implementation to study for graph-aware reranking. [T1: §3.5]

SUMMARY TABLE (mechanism presence):
| Mechanism | SHARD (per brief) | Zep/Graphiti |
| temporal validity | scalar staleness ratio + decay (impl) | bi-temporal 4-timestamp ranges (impl) |
| time-decay / expected-lifetime | YES (impl) | NO |
| semantic conflict detection | DESIGN only; repo SYNTACTIC | YES, shipped (impl) |
| non-destructive invalidation history | unclear | YES (impl) |
| multi-agent DECLARE-REVIEW-EXECUTE | YES (impl, design-level coord) | NO |
| trust tiers T1-T4 | YES | NO (has source provenance instead) |
| self-improvement / rollback cascade | DESIGN only; repo absent | absent (not claimed) |
| hybrid graph-aware retrieval + rerank | likely simpler | YES, advanced (impl) |

Net: Graphiti is the stronger, battle-tested IMPLEMENTATION of the temporal-KG core (bi-temporal validity, semantic contradiction invalidation, hybrid retrieval). SHARD's distinct value is what Graphiti deliberately omits: time-decay staleness with quarantine, multi-agent coordination gating, and graded trust tiers. The single highest-value borrow for ELAI is Graphiti's entity-pair-scoped semantic contradiction + non-destructive invalidation, which directly fills SHARD's syntactic-only gap.

---

## 8. Citations

[1] T1 — arXiv:2501.13956 abstract page, "Zep: A Temporal Knowledge Graph Architecture for Agent Memory", Rasmussen/Paliychuk/Beauvais/Ryan/Chalef, submitted 2025-01-20. https://arxiv.org/abs/2501.13956
[2] T1 — arXiv:2501.13956 full HTML (method + results), v1. https://arxiv.org/html/2501.13956v1
[3] T1 — GitHub API, getzep/graphiti repo metadata (Apache-2.0, 26,820 stars, created 2024-08-08, push 2026-05-21). https://api.github.com/repos/getzep/graphiti
[4] T1 — GitHub API, getzep/zep repo metadata (4,626 stars, "Examples, Integrations, & More", not archived). https://api.github.com/repos/getzep/zep
[5] T1 — GitHub API, graphiti_core/ directory contents (module/file listing). https://api.github.com/repos/getzep/graphiti/contents/graphiti_core
[6] T1 — Raw source: graphiti_core/graphiti.py (Graphiti class method signatures + node/edge type imports). https://raw.githubusercontent.com/getzep/graphiti/main/graphiti_core/graphiti.py
[7] T1 — GitHub API, getzep/graphiti latest release v0.29.1, published 2026-05-21. https://api.github.com/repos/getzep/graphiti/releases/latest
[8] T1 — GitHub API, getzep/graphiti tags (v0.30.0pre0..pre5, v0.29.1, v0.29.0...). https://api.github.com/repos/getzep/graphiti/tags
[9] T1 — PyPI graphiti-core JSON (version 0.29.1, Apache-2.0, Python >=3.10, deps, upload 2026-05-21). https://pypi.org/pypi/graphiti-core/json
[10] T1 — PyPI zep-cloud JSON (version 3.22.0, license null, upload 2026-05-04). https://pypi.org/pypi/zep-cloud/json
[11] T1 — PyPI zep-python JSON (version 2.0.2, license null, no explicit deprecation). https://pypi.org/pypi/zep-python/json
[12] T3-vendor — getzep.com homepage (tagline, named users, LoCoMo 94.7% / 187ms claims). https://www.getzep.com/
[13] T3-vendor — getzep.com/product/agent-memory (LoCoMo vs unnamed-baseline benchmark panel). https://www.getzep.com/product/agent-memory/
[14] T3 — search aggregators (Tracxn / Crunchbase / getlatka) for funding (~$500K seed, YC, ~$1M revenue 2024) and YC W24 batch. Aggregator data, approximate.
[15] T1 (negative result) — help.getzep.com/docs/graphiti/getting-started/overview returned 404; docs IA has moved (root help.getzep.com is canonical).

---

## 9. Grounding ledger

- Paper exists, title, authors, date — GROUNDED [1].
- Abstract claims (DMR 94.8 vs 93.4, LongMemEval +18.5%, 90% latency) — GROUNDED [1][2].
- Three-tier subgraph (episode/semantic-entity/community) — GROUNDED [2].
- Bi-temporal model, 4 timestamps (t_valid/t_invalid/t'_created/t'_expired), timeline T vs T' labels — GROUNDED, verbatim [2].
- Edge invalidation mechanism ("set t_invalid to invalidating edge's t_valid") — GROUNDED, verbatim [2].
- Search functions (phi_cos / phi_bm25 / phi_bfs), Neo4j-Lucene BM25, n-hop BFS — GROUNDED [2].
- Rerankers (RRF, MMR, episode-mention, node-distance, cross-encoder) — GROUNDED [2] + cross_encoder module [5].
- Label-propagation community detection (not Leiden) + dynamic single-step extension — GROUNDED [2].
- Reflexion-style extraction, n=4 prior-message context, entity-pair-scoped dedup — GROUNDED [2].
- Live Graphiti class methods + add_episode/search signatures + node/edge types — GROUNDED, verbatim [6].
- SagaNode / sagas (post-paper feature) — GROUNDED [6] (present in current source; not in paper).
- graphiti-core 0.29.1, Apache-2.0, Python>=3.10, deps (neo4j/openai/pydantic...) — GROUNDED [9].
- graphiti-core 0.29.1 upload date 2026-05-21 — GROUNDED [9] (earlier "2025-01-16" was a WebFetch serial-misread, corrected against the GitHub release date [7]).
- GitHub stars/forks/dates for both repos — GROUNDED [3][4].
- v0.30.0pre prereleases on top of v0.29.1 — GROUNDED [8].
- zep-cloud 3.22.0 / zep-python 2.0.2 versions + null license metadata — GROUNDED [10][11].
- DMR sample (500 convos, ~60 msgs) + authors' own DMR weaknesses — GROUNDED [2].
- LongMemEval per-type breakdown + token reduction — GROUNDED [2].
- LoCoMo 94.7% / sub-200ms / 5,760 tokens — GROUNDED as VENDOR CLAIM only [12][13]; baseline system unnamed on the page (transparency flag).
- Named production users — GROUNDED as VENDOR CLAIM only [12]; not independently verified.
- YC W24 batch — GROUNDED [14] (T2/T3).
- Funding ~$500K seed / ~$1M revenue 2024 — WEAKLY GROUNDED [14] (T3 aggregator, no primary press release). Treat as approximate.
- Author affiliations (printed institution) — UNGROUNDED: the arXiv abstract block did not print affiliation lines; org attribution to "Zep AI" is inferred from company identity, not a fetched affiliation string.
- LongMemEval exact question count (N) for Zep's run — UNGROUNDED: not captured from [2]; the commonly-cited ~500 is from the upstream LongMemEval paper, NOT from 2501.13956, so it is NOT attributed here.
- zep-cloud / zep-python actual license text — UNGROUNDED: PyPI license field is null; a repo LICENSE read would be required to confirm (informational only, no vendoring planned).
- SHARD-side specifics (staleness formula, T1-T4, composition.py, .bak rollback, syntactic-only repo state) — taken AS GIVEN from the task brief; not independently verified against the SHARD repo in this dossier (out of scope — this is the Zep/Graphiti side of the comparison).

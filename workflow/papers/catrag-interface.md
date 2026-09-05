CatRAG: Context-Aware Traversal for Robust Retrieval-Augmented Generation
Paper: arXiv 2602.01965 (Lau et al., 2026)
GitHub: https://github.com/kwunhang/CatRAG
Reliability: MEDIUM (paper with full hyperparameter tables; code not yet released)

================================================================================
1. PROBLEM STATEMENT
================================================================================

Static Graph Fallacy: graph-based RAG (HippoRAG/HippoRAG 2) uses fixed transition
matrices determined at indexing time. Two failure modes:
  (a) Semantic drift: high-weight generic edges (e.g., Marie Curie -> Radioactivity)
      divert probability mass away from downstream bridge entities needed for multi-hop.
  (b) Hub node problem: high-degree entities (Nobel Prize, French) act as topological
      sinks, diluting probability mass.

Result: high partial recall but broken reasoning chains (missing bridge documents).

================================================================================
2. ARCHITECTURE OVERVIEW
================================================================================

CatRAG extends HippoRAG 2 with three query-time modifications applied BEFORE PPR:
  1. Symbolic Anchoring (modifies seed/reset distribution)
  2. Query-Aware Dynamic Edge Weighting (modifies relation edge weights in T)
  3. Key-Fact Passage Weight Enhancement (modifies context edge weights in T)

Then standard PPR runs on the modified graph. One-shot retrieval (no iterative loops).

Pipeline (query time):
  query -> NER extraction -> triple extraction (HippoRAG 2 baseline)
       -> Symbolic Anchoring (inject NER entities as weak seeds)
       -> Coarse pruning (vector similarity filter on neighbor edges)
       -> Fine-grained scoring (LLM scores surviving edges)
       -> Key-Fact Passage Enhancement (boost context edges for verified triples)
       -> Build modified transition matrix T_hat_q
       -> Run PPR on T_hat_q
       -> Rank passages by PPR stationary distribution
       -> Return top-k passages

================================================================================
3. GRAPH DATA STRUCTURE (from HippoRAG 2)
================================================================================

Directed graph G = (V, E) where:
  V = V_E union V_P
    V_E: entity phrase nodes (extracted via OpenIE)
    V_P: passage nodes (one per document chunk)

  E = E_rel union E_syn union E_ctx
    E_rel: relation edges between entities (from OpenIE triples)
    E_syn: synonym edges between entities with high vector similarity (>= 0.8 threshold)
           weight = 2.0 * similarity_score
    E_ctx: context edges linking passage nodes to entity nodes contained within

================================================================================
4. PERSONALIZED PAGERANK (PPR) - BASE FORMULA
================================================================================

  v^(k+1) = (1 - d) * e_s + d * v^(k) * T

where:
  v^(k): probability distribution over all nodes at step k
  e_s: personalized reset distribution over seed nodes
  T: row-normalized transition matrix
  d: damping factor = 0.5

CatRAG modifies both e_s (via Symbolic Anchoring) and T (via Dynamic Edge Weighting
and Key-Fact Enhancement) to produce T_hat_q before running PPR.

================================================================================
5. INNOVATION 1: SYMBOLIC ANCHORING
================================================================================

Purpose: Inject explicitly recognized named entities from the query as weak topological
anchors to prevent immediate drift into generic hubs.

Algorithm:
  1. Run NER on query q to extract entity set {e_1, e_2, ..., e_n}
  2. For each extracted entity e_i, find matching node(s) in V_E
  3. Assign reset probability epsilon = 0.2, weighted by inverse passage count:
     reset_weight(e_i) = epsilon * |P_i|^(-1)
     where |P_i| is the number of passages containing entity e_i
  4. These are ADDED as weak seeds alongside the existing HippoRAG 2 seed entities
     (from triple-based retrieval). The triple-based seeds retain higher probability.
  5. The combined distribution is normalized to form e_s for PPR.

Effect: Creates "gravitational pull" that resists diffusion into generic hubs.
The NER entities serve as auxiliary anchors, subordinate to contextual triple seeds.

NER Prompt (from repo):
  System: "You're a very effective entity extraction system."
  User: "Please extract all named entities that are important for solving the
         questions below. Place the named entities in json format.
         Question: {query}"
  Output: {"named_entities": ["Entity1", "Entity2", ...]}

================================================================================
6. INNOVATION 2: QUERY-AWARE DYNAMIC EDGE WEIGHTING
================================================================================

Purpose: Dynamically modulate relation edge weights to prune irrelevant paths and
amplify those aligned with query intent. Applied only to E_rel edges.

Two-stage coarse-to-fine strategy:

--- STAGE I: Coarse-Grained Candidate Pruning ---

Hyperparameters:
  N_seed = 5   (max number of seed entities to process)
  K_edge = 15  (max number of edges per seed for fine-grained scoring)

Algorithm:
  1. Select top-N_seed entity nodes by their initial reset probabilities
     (from dense retrieval alignment / triple matching)
  2. For each selected seed u:
     a. Get all outgoing relation edges to neighbors {v_1, v_2, ...}
     b. If |neighbors| > K_edge:
        - Compute vector similarity between query embedding and
          fact embeddings of each relation edge (u, relation, v)
        - Keep top-K_edge neighbors by similarity -> N_top(u)
        - Assign remaining neighbors a minimal "Weak" weight (0.2)
     c. If |neighbors| <= K_edge: all pass to Stage II

--- Adaptive Entity Contextualization (preprocessing for Stage II) ---

For each surviving neighbor v in N_top(u):
  Let F(v) = set of fact triples connected to entity node v
  Define density threshold tau (not explicitly specified; implied by token limit)

  C(v) = Summary(F(v))   if |F(v)| > tau
          Concat(F(v))    otherwise

Summary generation uses LLM with this prompt template:
  "Generate a concise, entity-focused summary that captures the core identity
   and key relationships of a given entity based on its associated fact triplets."
  Input: target_entity name + fact_triplets as JSON list of (subj, rel, obj) tuples
  Output: max 150 tokens narrative summary
  Parameters: summary_length=150, language=English

--- STAGE II: Fine-Grained Semantic Probability Alignment ---

For each surviving edge (u -> v) where u is a seed and v in N_top(u):

  1. Construct prompt with:
     - QUERY: the user query
     - CURRENT ENTITY NODE: seed entity u
     - RETRIEVED FACTS: the seed triples (trusted evidence)
     - NEIGHBORS: list with linking triplet(s) and summary C(v) for each v

  2. LLM classifies each neighbor into discrete tiers:
     L in {Irrelevant, Weak, High, Direct}
     via integer score s in {0, 1, ..., 10}

  3. Score-to-weight projection phi(s) (NON-LINEAR mapping):

     Semantic Tier    LLM Score (s)    Output Weight phi(s)
     -------------------------------------------------------
     Irrelevant       0-3              0.0
     Weak             4-6              0.2-0.3 (linear interpolation)
     High             7-9              2.0-3.0 (linear interpolation)
     Direct           10               5.0

     Linear interpolation within Weak tier:
       phi(4) = 0.2, phi(5) = 0.25, phi(6) = 0.3

     Linear interpolation within High tier:
       phi(7) = 2.0, phi(8) = 2.5, phi(9) = 3.0

  4. Updated dynamic weight:
     w_hat_uv = phi(LLM(q, u, v, C(v))) * w_uv^(static)

  Modulation is ASYMMETRIC: applied only to forward edges originating from seed set.
  Edges not originating from seeds retain their static weights.

LLM Scoring Prompt (from repo, 2-shot):
  System: "You are a knowledge graph reasoning expert. Score neighbor entities
           (0-10) on their utility for answering a QUERY."
  Scoring criteria:
    10 (Solution): neighbor IS the answer
    7-9 (Bridge): critical step in reasoning chain
    4-6 (Weak): valid but tangential
    0-3 (Noise): irrelevant, generic, or contradicts facts
  Rules:
    - If neighbor contradicts RETRIEVED FACTS, score 0
    - Output format: "ID (Entity Name): Score" (if <4)
                     "ID (Entity Name): Score | Concise reasoning" (if >=4)
  Input template:
    QUERY: ${query}
    RETRIEVED FACTS: ${seed_triplet}
    CURRENT ENTITY NODE: ${seed_entity}
    NEIGHBORS: ${neighbor_entities}
      Each neighbor: [ID] "name" | LINKING FACT: (triple) | SUMMARY: C(v)

================================================================================
7. INNOVATION 3: KEY-FACT PASSAGE WEIGHT ENHANCEMENT
================================================================================

Purpose: Bias the random walk toward passages containing verified evidentiary triples,
with zero additional LLM cost.

Let T_seed = set of verified seed triples (from HippoRAG 2's Recognition Memory
filtering step).

For each context edge from entity u to passage p:
  w_hat_up = w_up * (1 + beta * I(u, p in T_seed))

where:
  beta = 2.5 (boost factor)
  I(u, p in T_seed) = 1 if the edge from u to p is supported by a triple in T_seed
                     = 0 otherwise

"Supported by a triple" means: passage p contains a fact triple (u, r, v) or (v, r, u)
that was explicitly identified during the HippoRAG 2 recognition memory filtering.

This is purely algorithmic (no LLM calls). Applied to ALL context edges E_ctx,
not just those from seed entities.

================================================================================
8. UNIFIED RETRIEVAL PROCESS (PSEUDOCODE)
================================================================================

function CatRAG_Retrieve(query q, graph G, top_k):
    // Step 1: HippoRAG 2 baseline processing
    triples = extract_query_triples(q)                    // LLM: query-to-triple
    seed_entities_triple = match_triples_to_graph(triples, G)
    T_seed = recognition_memory_filter(triples, G)        // HippoRAG 2 filtering

    // Step 2: Symbolic Anchoring
    ner_entities = extract_NER(q)                          // LLM: NER extraction
    matched_ner = fuzzy_match_to_graph(ner_entities, G.V_E)
    for each e_i in matched_ner:
        reset_prob[e_i] += epsilon * |passages_containing(e_i)|^(-1)
    // Combine with triple-based seeds (triple seeds have higher base weight)
    e_s = normalize(seed_entities_triple + reset_prob)

    // Step 3: Query-Aware Dynamic Edge Weighting
    top_seeds = select_top_N_seed(e_s, N_seed=5)
    for each seed u in top_seeds:
        neighbors = outgoing_relation_neighbors(u, G.E_rel)
        if |neighbors| > K_edge:
            // Coarse pruning by vector similarity
            for each v in neighbors:
                sim[v] = cosine(embed(q), embed(fact_triple(u,v)))
            N_top = top_K_by_similarity(neighbors, K_edge=15)
            for v not in N_top:
                G.E_rel[u][v].weight *= 0.2    // minimal Weak weight
        else:
            N_top = neighbors

        // Adaptive Entity Contextualization
        for each v in N_top:
            facts_v = get_connected_facts(v, G)
            if |facts_v| > tau:
                C_v = LLM_summarize(v, facts_v)           // LLM call
            else:
                C_v = concat_facts(facts_v)

        // Fine-grained LLM scoring
        scores = LLM_score_neighbors(q, u, T_seed, N_top, C)  // LLM call
        for each (v, score) in scores:
            phi_s = score_to_weight(score)                 // Table 7 mapping
            G.E_rel[u][v].weight *= phi_s

    // Step 4: Key-Fact Passage Weight Enhancement
    for each context_edge (u, p) in G.E_ctx:
        if exists triple in T_seed connecting u to p:
            G.E_ctx[u][p].weight *= (1 + beta)            // beta = 2.5

    // Step 5: Build modified transition matrix and run PPR
    T_hat_q = row_normalize(adjacency_matrix(G))
    v_stationary = PPR(e_s, T_hat_q, damping=0.5)

    // Step 6: Rank and return passages
    passage_scores = {p: v_stationary[p] for p in G.V_P}
    return top_k_passages(passage_scores, top_k)


function score_to_weight(s):
    // Table 7: Non-linear projection
    if s <= 3: return 0.0          // Irrelevant
    if s == 4: return 0.2          // Weak (low)
    if s == 5: return 0.25         // Weak (mid)
    if s == 6: return 0.3          // Weak (high)
    if s == 7: return 2.0          // High (low)
    if s == 8: return 2.5          // High (mid)
    if s == 9: return 3.0          // High (high)
    if s == 10: return 5.0         // Direct
    return 0.0

================================================================================
9. COMPLETE HYPERPARAMETER TABLE
================================================================================

Parameter                              Value     Notes
-------------------------------------------------------------------------
Synonym Similarity Threshold           0.8       For E_syn edge creation
Synonym Edge Weight                    2.0*sim   Scaled by vector similarity
PPR Damping Factor (d)                 0.5       Standard PPR parameter
LLM Temperature                        0.0       Deterministic scoring
Symbolic Anchor epsilon                0.2       Weighted by |P_i|^(-1)
Max Seed Nodes for scoring (N_seed)    5         Coarse pruning parameter
Max Pruning Edges per seed (K_edge)    15        Coarse pruning parameter
Passage Boost Factor (beta)            2.5       Key-Fact Enhancement
Retrieval depth (top_k)                5         Passages returned

Score-to-weight mapping (phi):
  s=0-3  -> 0.0 (Irrelevant, pruned)
  s=4    -> 0.2
  s=5    -> 0.25
  s=6    -> 0.3
  s=7    -> 2.0
  s=8    -> 2.5
  s=9    -> 3.0
  s=10   -> 5.0

Density threshold tau: not explicitly stated (implied: summarize when triples
exceed token budget for direct concatenation, ~150 token summary limit).

Models used in paper:
  LLM backbone: GPT-4o-mini (all LLM components)
  Embeddings: text-embedding-3-small
  QA reader: Llama-3.3-70B-Instruct (top-5 passages)

================================================================================
10. PERFORMANCE BENCHMARKS
================================================================================

--- Recall@5 ---
                  MuSiQue   2Wiki   HotpotQA   HoVer
HippoRAG 2       61.4      85.9    87.1       71.2
CatRAG            64.9      87.0    89.5       76.8
Delta             +3.5      +1.1    +2.4       +5.6

--- Downstream QA (F1 / Accuracy for HoVer) ---
                  MuSiQue   2Wiki   HotpotQA   HoVer
HippoRAG 2       43.2      68.1    69.4       67.2
CatRAG            45.0      69.7    71.4       69.0

--- Full Chain Retrieval (FCR) / Joint Success Rate (JSR) ---
                  MuSiQue        2Wiki         HotpotQA       HoVer
HippoRAG 2       30.5/21.5      66.1/53.0     75.5/53.4      34.8/26.2
CatRAG            34.6/24.3      67.6/55.0     80.4/56.8      42.5/31.1
FCR Delta         +4.1           +1.5          +4.9           +7.7
JSR Delta         +2.8           +2.0          +3.4           +4.9

FCR = percentage of queries where ALL gold supporting documents are retrieved
JSR = FCR AND correct answer (strict joint metric)

--- Ablation (Recall@5) ---
                  MuSiQue   2Wiki   HotpotQA   HoVer
Full CatRAG       64.9      87.0    89.5       76.8
w/o Symbolic      63.0      86.1    88.6       73.6   (-3.2 on HoVer)
w/o E_rel weight  63.2      85.6    88.1       75.0
w/o Passage Enh   64.7      88.4    89.0       76.6

--- Hub Bias Analysis (MuSiQue, 100 queries) ---
Mean PPR-Weighted Strength: HippoRAG 2 = 837.0, CatRAG = 761.7
Super Hub mass (top 1% degree): HippoRAG 2 = 45.7%, CatRAG = 42.5%

================================================================================
11. METRICS DEFINITIONS
================================================================================

Recall@k: fraction of gold passages present in top-k retrieved passages
FCR (Full Chain Retrieval): percentage of queries where top-k contains ALL gold docs
JSR (Joint Success Rate): FCR AND correct answer
PPR-Weighted Strength: S_ppr(q) = sum over v in V_top of p_hat(v) * Strength(v)
  where p_hat(v) = renormalized PPR probability, Strength(v) = weighted degree

================================================================================
12. INPUT/OUTPUT INTERFACES
================================================================================

--- Indexing (same as HippoRAG 2) ---
Input: corpus of text passages
Output: directed graph G = (V_E union V_P, E_rel union E_syn union E_ctx)
  - V_E: entity nodes with embeddings
  - V_P: passage nodes
  - E_rel: (entity, relation_label, entity) with static weight
  - E_syn: (entity, entity) with weight = 2.0 * cosine_similarity
  - E_ctx: (entity, passage) linking entities to containing passages

--- Query-time retrieval ---
Input:
  - query string q
  - graph G (from indexing)
  - top_k (default 5)
Output:
  - ranked list of top_k passage nodes with PPR scores

LLM calls at query time (per query):
  1. Query-to-triple extraction (HippoRAG 2 baseline)
  2. NER extraction (Symbolic Anchoring) -- 1 call
  3. Entity summarization (up to N_seed * K_edge calls, but only for dense nodes)
  4. Neighbor scoring (up to N_seed calls, one per seed, batch neighbors in prompt)

================================================================================
13. IMPLEMENTATION NOTES FOR RUST REIMPLEMENTATION
================================================================================

Key data structures:
  - Graph: adjacency list with typed edges (Relation, Synonym, Context)
  - Each edge stores: source, target, edge_type, weight (mutable for query-time mods)
  - Entity node: name, embedding vector, list of fact triples
  - Passage node: id, text, list of contained entity ids
  - Transition matrix: sparse matrix (CSR format recommended for PPR)

PPR implementation:
  - Power iteration method (Eq. 1)
  - Converge when max |v^(k+1) - v^(k)| < epsilon (standard 1e-6)
  - Or fixed iteration count (HippoRAG uses ~30 iterations typically)
  - damping d = 0.5

Edge weight modification is IN-PLACE on a COPY of the graph for each query.
Do NOT modify the base indexed graph. Either:
  (a) Clone relevant edge weights into query-local overlay
  (b) Use a delta/overlay pattern: base_weight + query_delta

The score_to_weight mapping (Table 7) uses linear interpolation within tiers:
  Weak tier: phi(s) = 0.2 + (s - 4) * 0.05    for s in [4, 6]
  High tier: phi(s) = 2.0 + (s - 7) * 0.5     for s in [7, 9]

The coarse pruning (Stage I) requires: embedding index for fact triples of relation
edges. At query time, compute cosine similarity between query embedding and each
fact embedding. This can use the same embedding index as the base retrieval.

For Symbolic Anchoring, entity matching from NER output to graph nodes should use
fuzzy/embedding-based matching (same as HippoRAG 2's entity linking).

The "Recognition Memory Filtering" from HippoRAG 2 produces T_seed (verified triples).
This is the input for Key-Fact Passage Enhancement. The exact filtering mechanism
is part of HippoRAG 2's pipeline (LLM-based triple verification).

================================================================================
14. PROMPTS (COMPLETE, FROM GITHUB REPO)
================================================================================

--- NER Query Prompt ---
System: "You're a very effective entity extraction system."
User: "Please extract all named entities that are important for solving the
       questions below. Place the named entities in json format.
       Question: {query}"
Output: {"named_entities": ["Entity1", "Entity2"]}

--- Entity Summarization Prompt ---
(see section 6, Adaptive Entity Contextualization)
Input: target_entity name + fact_triplets as JSON
Output: max 150 token narrative summary
Template uses 1-shot example (Alex O'Loughlin actor example)

--- Neighbor Scoring Prompt ---
(see section 6, Stage II)
System prompt defines scoring criteria (0-10 scale)
Uses 2-shot examples:
  Example 1: "Time Out's 100 best British films" query (Don't Look Now graph)
  Example 2: "Treaty ending WW1" query (Franz Ferdinand graph)
Output: one line per neighbor with score and optional reasoning

================================================================================
15. LIMITATIONS AND DESIGN TRADEOFFS
================================================================================

- Dynamic edge weighting requires LLM inference at query time (latency cost)
- Mitigated by coarse pruning (only N_seed=5 seeds, K_edge=15 edges each)
- Maximum LLM scoring calls per query: ~5 (one batch per seed)
- Entity summarization calls: only for dense nodes exceeding tau threshold
- Key-Fact Enhancement is free (pure algorithmic, no LLM)
- Symbolic Anchoring requires one additional NER call (cheap)
- The paper uses GPT-4o-mini for all LLM calls (cost-efficient)
- Source code not released (proprietary data protection); prompts and hyperparameters
  provided for reimplementation

================================================================================
16. RELATIONSHIP TO HIPPORAG 2
================================================================================

CatRAG is a strict SUPERSET of HippoRAG 2. It uses the same:
  - Graph construction (OpenIE triples, synonym edges, context edges)
  - Query-to-triple extraction
  - Recognition memory filtering
  - PPR algorithm
  - Passage ranking from PPR output

CatRAG ADDS (all at query time, before PPR):
  - Modified reset distribution (Symbolic Anchoring)
  - Modified relation edge weights (Dynamic Edge Weighting)
  - Modified context edge weights (Key-Fact Passage Enhancement)

No changes to indexing. All modifications are query-time overlays.

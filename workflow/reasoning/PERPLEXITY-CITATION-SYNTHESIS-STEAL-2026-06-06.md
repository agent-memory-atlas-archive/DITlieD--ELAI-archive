PERPLEXITY -> ELAI: one system to improve (citation-bound synthesis on the answer path)
Generated 2026-06-06. Source of the external facts: workflow/sessions/oracle/2026-06-06-perplexity-ai-openness.md
(oracle run, 14 sources, confidence M; pipeline-internals detail is tier-2/3 single-source, flagged below).
Grounded against an end-to-end read of phase-index-MASTER (phases 1A-3I) + B/C/D/E/F/G/H summary indices.

PREMISE
The Perplexity research answered "is it open / how does it work". The single most transferable, best-documented
design in that answer is NOT the proprietary index and NOT the model menu. It is the SHAPE of the answer step:
"structured prompt assembly where citation markers, URLs, dates, and ranked excerpts are embedded into the prompt
BEFORE generation (citations are predetermined, not retrofitted), then constrained LLM synthesis bounded by the
retrieved evidence with inline citation numbers." (oracle doc, "How the pipeline works", stage 5-6.)
That one sentence is the steal. It is also the exact mechanism that makes Perplexity's answers feel grounded
rather than plausible, and it is directly aligned with ELAI's cant-lie north star and grounded-by-default rule.

GROUNDING HONESTY (anti-sycophancy, per house rule)
- The citation-IN-PROMPT-before-generation claim is described by third-party analysts in the oracle doc and is
  flagged there as NOT officially confirmed (tier-2/3, single-source). Treat the Perplexity-SPECIFIC detail as
  MEDIUM confidence.
- The underlying PRINCIPLE (constrain generation to a fixed, pre-retrieved evidence set; assign citation ids
  before decoding) is independently well-grounded in ELAI's own already-ingested literature: Program-of-Thought /
  CodeAct constructive grounding (D-11 cites arXiv:2211.12588, 2402.01030), and the GhostCite-hardening framing
  in E-48. So the recommendation does not rest on the unverified Perplexity detail alone.
- What is corroborated at tier-1 in the oracle doc and worth recording: Perplexity's own stated moat is "the
  proprietary index + tuned ranking + orchestration, NOT any single model." That is external corroboration of
  ELAI's standing thesis (smarter orchestration not a smarter model: H-7 calibrated routing, H-42 serial-depth
  budget formally grounding the thesis, project_efficiency_trust_engineering memory). Record as a confidence
  signal on strategy; it is not itself the improvement.

THE GAP IN ELAI (high confidence: code/plan-read, not measured)
ELAI's retrieval substrate is already strong and arguably broader than what Perplexity documents publicly:
- hybrid sparse+dense+graph retrieval with RRF (Phase 1B tantivy BM25, 1C dual-ONNX GraphCodeBERT+Granite,
  1A DKG, 1D PageRank, RRF k=60),
- multi-stage rerank (1G ms-marco-MiniLM-L12 cross-encoder),
- an AGENTIC retrieve-evaluate loop with a discard/re-retrieve quality behaviour (1E A-RAG, max_loops=15,
  ContextTracker) -- this already matches Perplexity's stage-4 "quality gate that can discard and re-retrieve",
- query-tier routing FAST/STANDARD/DEEP (3K.4), and authority+freshness signals on the web lane (source-tier
  T1-T4 distribution; Cosmos freshness B-42; F-5 staleness quarantine).
So retrieval and ranking are NOT the gap. The gap is the ANSWER step.

ELAI's current citation discipline is DETECT-AFTER, not PREVENT-BEFORE:
- the oracle/researcher answer path (C-30 oracle_cache + repl_command /research|/ask + the researcher agent's
  ORACLE OUTPUT FORMAT) has the MODEL EMIT [N] citations, then
- E-48's 4-stage CitationVerifier does GhostCite hardening AFTER generation (verify each [N] resolves and
  supports the claim), and
- Phase 2B grounding (FILE/SOURCE/TEST/WEB/API/API_REGISTRY/Artifact/Cosmos) verifies CLAIMS after the fact.
This is correct but structurally weaker than Perplexity's design: a ghost or mis-attributed citation is something
you CATCH (and then must repair or refuse), not something the architecture makes IMPOSSIBLE. The model is free to
write [3] next to a sentence the retrieved evidence does not support, and the system's job becomes detection.

THE IMPROVEMENT (the steal)
Make citation assignment a first-class PRE-GENERATION stage on the answer path, so a hallucinated/ghost citation
is STRUCTURALLY impossible rather than caught-after:
1. Pre-bind: after rerank (1G) and A-RAG convergence (1E), freeze the final ranked evidence set as numbered,
   addressable anchors {id -> (verbatim_excerpt_blake3, source_uri, retrieved_at, source_tier)} and inject THAT
   numbered set into the assembled context (a dedicated context slot; Phase 3E already has fixed slots -- add an
   Evidence slot or reuse CodeContext with anchor framing). The id space is closed and known before decoding.
2. Constrain: decode the answer under a grammar that only permits inline [N] markers drawn from the pre-bound id
   set (reuse the E-35 XGrammar / constrained-decoding substrate; PER_ROLE_DECODING already exists in C-26
   decoding.rs). A claim sentence either carries a valid pre-bound [N] or it is an explicitly unsupported span.
3. Verify (now cheap and strictly stronger): E-48's CitationVerifier degenerates from "does this freeform [3]
   exist and support the claim" to "does the pre-bound anchor 3's verbatim excerpt (blake3-pinned) still support
   the span" -- a deterministic membership + support check over a closed set, not an open-world ghost hunt.
4. Refuse honestly: a synthesis step that cannot ground a load-bearing claim in any pre-bound anchor routes to
   Mode E refusal (D-16 conformal refusal / D-17 SLA), not to an invented citation.

WHY THIS IS THE RIGHT ONE (not the index, not the model)
- It is the highest-leverage delta because it converts ELAI's MOST-MARKETED property (cant-lie / grounded answers)
  from a post-hoc guard into an architectural invariant, and it is the one Perplexity design that is both
  well-described and squarely inside ELAI's existing competence.
- It rides substrate ELAI already ships or has planned: E-35 (constrained decode), C-26 (PER_ROLE_DECODING grammar),
  E-48 (CitationVerifier, now a membership check), D-11/D-16/D-17 (Mode-A checkers, refusal), H-52 (deterministic
  verbatim-snippet -> anchor resolver -- the SAME deterministic-resolver idea Perplexity uses for citations; note
  the independent convergence). It is composition, not invention.
- It is measurable in ELAI's own currency: a ghost-citation rate that should drop to ~0 by construction on the
  answer path, recomputable from the SSB journal under ELAI_SEED (GD-2), with the refusal_rate (D-17) as the
  guardrail against the gate degenerating into "refuse everything".

TWO APPROACHES (house rule: two minimum before a recommendation)
A. Full pre-bind + constrained-decode (recommended). Anchors injected, grammar-constrained [N], verifier becomes
   a membership/support check. Strongest guarantee (ghost citations structurally impossible on the answer path).
   Cost: a new Evidence context slot competes for the 25-32K local window (Phase 3E budget pressure on the LOCAL
   lane; on cloud the 200K profile absorbs it). Risk: an over-tight grammar suppresses legitimate synthesis of
   multi-source claims -- mitigate by allowing [N,M] multi-anchor markers and an explicit unsupported-span token.
B. Pre-bind anchors WITHOUT a decode grammar (lighter). Inject the numbered anchor set and keep E-48 as the
   after-check, but now the check is against a closed pinned set (blake3 excerpts) instead of open-world. Cheaper,
   no grammar/local-window cost, works on any backend incl. ones without constrained decoding. Weaker: the model
   can still emit a [N] the span does not support -- you still CATCH it, you do not PREVENT it. Good Tier-1 first
   ship; A is the Tier-2 promotion.

SIDE EFFECTS / EDGE CASES (completeness, per house rule)
- LOCAL window cost (approach A): the Evidence slot is real token pressure on the 25-32K lane; cross-reference the
  Track-9 context-lifecycle work (H-50 regime-aware policy) -- the Evidence slot should be window-budgeted per
  backend, smaller on local.
- Cache-prefix stability (H-2/HD-2): the pre-bound anchor block is query-specific and therefore NOT part of the
  stable cache prefix; place it AFTER the cache breakpoints so it does not bust the prefix.
- Determinism: anchor ids must be assigned deterministically (rank order, ELAI_SEED) or the same query yields
  different [N] numbering across runs and breaks GD-2 reproducibility. Use rank-stable ids keyed to the frozen
  ranked set.
- Honest scope: this hardens the citation->evidence link (the answer cites only what was retrieved). It does NOT
  make the retrieved evidence itself TRUE -- that is still the retrieval/source-tier/Cosmos-freshness problem, and
  for novel claims still routes to D-15 novelty labeling + human review. Do not over-claim "cant-lie" beyond
  "cant-cite-what-it-didnt-retrieve".
- The Perplexity-specific mechanism is unverified (see GROUNDING HONESTY). The recommendation stands on ELAI's own
  D-11/E-48 grounding even if Perplexity's exact stage-5 design differs from the analyst description.

WHAT NOT TO COPY FROM PERPLEXITY
- The proprietary 200B-URL index and tuned authority/freshness ranking: not reproducible from public material and
  not ELAI's product (ELAI indexes a codebase + declared artifacts, not the open web). The web lane already has
  source-tier + Cosmos-freshness; do not chase a general web index.
- The multi-model router / "Model Council": ELAI already has the stronger, more auditable analog (B-37 PerRoleRuntime,
  E-31 tier matrix, H-7 calibrated routing, D-13 cross-family adversary). No steal.

NEXT ACTION IF PURSUED
This is a plan-shaped change touching the answer path (repl_command /research|/ask, the researcher oracle path),
the context-budget allocator (3E slot), the decode layer (C-26/E-35), and E-48. It is NOT a single-file patch:
per CLAUDE.md it would trigger prompt-transform + context-compiler + parallel explorers, and most naturally lands
as a new H-track answer-path file ("citation pre-binding + constrained synthesis") composing E-35/E-48/H-52/D-16,
with the ghost-citation-rate-to-zero observable_effect as the wiring done-gate. Author it before any code.

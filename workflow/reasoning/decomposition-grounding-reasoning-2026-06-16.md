REASONING CAPTURE — deterministic decomposition + token-source-grounding
Session 2026-06-16. Companion to docs/strategy/ELAI-DETERMINISM-DOCTRINE.md and
  workflow/reasoning/deep-research-decomposition-grounding-2026-06-16.md.
This is the analysis chain that produced those two artifacts, captured verbatim-
  close so a later session (or another model) does not have to re-derive it.
Confidence markers: VERIFIED = grounded in repo/measured data; TRAINING = from
  model training, unsearched, flagged for the deep-research pass.

================================================================================
THE USER'S STARTING POSITION (what kicked this off)
================================================================================
- Belief: the concept -> babystep decomposition is too simple and unreliable
  because the MODEL decides the size/count of steps. Wants a more deterministic
  decomposition for BOTH research and implementation.
- Idea A: a source-grounding system like SSB where every token of a fetched
  page gets a hash/footprint; when the model produces an answer, validate that
  the token was in the source. Unsure how feasible.
- Idea B: token cost of an LLM orchestration is unpredictable; for local models
  with a hard context ceiling this is dangerous (cannot over-feed context).
- Idea C: a Godel-machine-like scheme — the LLM defines code at a high level,
  that becomes a frozen rule/hash, and changing it requires new research +
  grounding that PROVES the improvement.
- Idea D: research modes for speed — a ranking system picks N pages, the harness
  (not the model via tools) fetches the page, the model just finds/ranks it,
  then deterministic source-grounding. Thinks ELAI may already have a direct
  website->info pipeline with no model in between.
- Framing the user gave: "the model is a cog, not a brain."
- Explicit instruction: reason from TRAINING first, then decide deep-research
  vs a sonnet oracle.

================================================================================
GROUNDED FACTS ESTABLISHED THIS SESSION (VERIFIED)
================================================================================
- Measured local context ceilings (metrics/baseline_findings_and_actions.md
  :150-216, the dead 12GB RTX 3060 run): Qwen2.5-Coder-3B NF4+FA2 ~40K tokens,
  7B ~25K tokens, BOTH 100% pass. 7B bottleneck = KV cache (56KB/token); 3B KV
  is 1KB/token (trivial). These are the only grounded rows; all other VRAM
  tiers are projections.
- The deterministic sizing scaffold ALREADY EXISTS (phase-index-MASTER.txt):
  * Phase 4E Stage 3.5 PHASE SIZING: phase_complexity = files_touched *
    avg_file_tokens + test_count * avg_test_tokens + contract_count * 200;
    split if > available_for_work * 0.6; max split depth 2. Deterministic, but
    COARSE — avg_file_tokens is a heuristic, NOT the measured per-(model,lane)
    ceiling.
  * Phase 4H AllowedConfigKey BabyStepMaxLoc{vram_tier}: babystep max LOC is
    already a per-VRAM-tier, self-mod-tunable knob.
  * Phase 3E CONTEXT BUDGET ALLOCATOR: deterministic token estimator
    (estimate for allocation, exact for validation), 8 fixed slots, never
    exceeds the model window; <50ms.
  * Phase 4G AutoAdvanceEvaluator: every FSM transition -> a deterministic
    EvidenceCheck, explicitly "no LLM."
  * Phase 4E AutoB2G DAG repair: REORDER/SUBSTITUTE are DETERMINISTIC; the
    LLM-driven SOCIA repair was NOT adopted.
- The research fetch is ALREADY harness-mediated (Idea D is partly real today):
  Phase 4D Research Agent runs as a Tier-C isolated process, "web search via
  IPC proxy only, no project access, model access via IPC." Plus the sidecar
  /search endpoint (memory project_search_architecture) + E-48 WEB RESEARCH
  ORCHESTRATOR with CitationVerifier GhostCite detection (DARK, no prod caller).
- Claim/citation grounding substrate ALREADY EXISTS:
  * GROUNDING VERIFICATION (phase-2B) WIRED: 6 methods, binary pass/fail,
    MiniCheck-style; plan sweep BLOCKS on any ungrounded claim.
  * F-12 Semantic Contradiction NLI (DeterministicNliJudge T1 + OnnxNliJudge
    T2) PARTIAL.
  * H-53 CITATION PRE-BINDING PARTIAL: freeze ranked EvidenceAnchorSet BEFORE
    generation, [N] is a closed-set membership check, out-of-set -> D-16
    refusal; "ghost citations structurally impossible." This is exactly the
    rigorous form of Idea A at citation granularity.
  * E-54 EVAL CORPUS uses 9-gram / 13-gram contamination filters (span-level
    fingerprinting precedent already in-tree).
- DGM precedent ALREADY EXISTS: SELF-MODIFICATION CORE (L1/L2/L3 staged eval,
  auto-revert >10% over 3), and Track-7 H-43 explicitly replaces the LIVE
  placeholder derived_alpha = blake3(config_bytes) ("fitness of BYTES not
  performance", a KNOWN GAP) with execution-grounded fitness.

================================================================================
VERDICT PER IDEA (the actual reasoning)
================================================================================

IDEA A — token-hash source grounding. RIGHT INSTINCT, WRONG GRANULARITY.
  TRAINING: hashing every token and checking membership is VACUOUS. Function
  words and common code tokens (the, =, return, let, }) appear in essentially
  every source, so a token-membership test passes ~100% trivially and measures
  nothing. A hash of a single token is just an indirection over the token; hash
  earns its keep only when it fingerprints variable-length SPANS. Worse, strict
  enforcement is wrong: the model MUST emit tokens not in the source
  (paraphrase, synthesis, connective tissue) — forbidding that forbids the
  reason to use an LLM.
  The SOUND form (3 layers): (1) span/n-gram fingerprinting (Rabin-Karp rolling
  hash or MinHash over the fetched corpus) as a cheap, no-second-LLM signal:
  what fraction of the output's content n-grams are extractively present, which
  claims have ZERO supporting span -> a fabrication-candidate flag; (2)
  claim-level ENTAILMENT (not membership) for flagged candidates — NLI, small
  model; (3) closed-set citation binding (freeze evidence, [N] is membership).
  Net-new is NARROW: a deterministic span-fingerprint cheap-gate over a
  harness-fetched, hashed corpus, feeding the EXISTING entailment layer
  (GROUNDING VERIFICATION / F-12 / H-53 / E-54 already cover the rest).

IDEA B + "deterministic decomposition" — THE PRIZE, but only SIZING can be
  deterministic, not CONTENT.
  Content decomposition ("what are the concepts of this idea") is irreducibly
  generative; trying to determinize it is a dead end. SIZING/ADMISSION can be
  fully deterministic and IS the fix for "the model decides step size
  unreliably." Shape: LLM PROPOSES a node; a deterministic NODE-ADMISSION GATE
  measures it and REJECTS oversized nodes, forcing re-split. Deterministic
  measurables available pre-execution: files<=2; impact-cone size from the
  B-29 DKG impact API; PRE-COMPUTED context-package size from the 3E allocator;
  intent_pairs>=3 (C-29); single observable_effect. Local-lane rule writes
  itself off the measured ceiling: admit to LOCAL only if predicted
  context-package <= measured_ceiling * safety_factor (3B ~40K / 7B ~25K);
  over-ceiling -> re-split or escalate to cloud (I-7 cascading escalation
  exists). You stop predicting the WHOLE orchestration's token cost; you bound
  each NODE, and the total is node-count * per-node-ceiling.
  SHARPENED after reading the master index: this is NOT greenfield. The 4E
  Stage-3.5 sizer + BabyStepMaxLoc{vram_tier} + 3E estimator already exist. The
  net-new piece is precise: replace the coarse avg_file_tokens*0.6 heuristic
  with a hard babystep-grain admission PRE_GATE sized against the MEASURED
  per-(model,lane) ceiling (the J-34 calibration curve, PLANNED), and gate the
  upstream LLM decomposition's node-COUNTS the same way. J-34 EMPIRICAL CONTEXT
  CALIBRATION is the same thought already on the roadmap.

IDEA C — Godel machine. PROOF IS INTRACTABLE; USE THE EMPIRICAL DESCENDANT.
  TRAINING: Schmidhuber's Godel machine requires a formal proof that a
  self-rewrite improves expected utility before applying it; that proof search
  is intractable in practice. The shipping descendant is the Darwin Godel
  Machine (Sakana, 2025): replace PROOF with empirical validation on a held-out
  benchmark — keep a change only if it MEASURABLY improves. ELAI already runs
  this (Self-Mod L1/L2/L3 + auto-revert; H-43 replacing the blake3-of-bytes
  fitness). Correct statement of the user's idea: the decomposition POLICY is a
  versioned, blake3-pinned, frozen artifact (extend D-1 ReasoningTemplate +
  C-25 crystallizer + gate-manifest R8 freeze); a change to it must clear an
  ablation gate (measured improvement on a held-out set of decomposition tasks
  — lower wire-dark rate, fewer oversized nodes, less rework), not a vibe. That
  is the SAFEDELTA/QDELTA framing applied to the decomposer itself.

IDEA D — research modes + model-finds / harness-fetches. SOUND; PARTLY EXISTS;
  UNIFIES WITH IDEA A.
  The split (model formulates queries + ranks/selects pages; harness does the
  actual fetch and fingerprints it) is the clean way to do retrieval-grounded
  generation: it removes the model's freedom to hallucinate fetched content and
  gives a known, hashable corpus to ground against — which is exactly what Idea
  A's span-fingerprint gate needs. Partly real today (Tier-C IPC web proxy +
  sidecar /search + E-48). Missing: the explicit model-ranks / harness-fetches-
  and-fingerprints contract + speed-tiered modes wired to it. Net-new is
  moderate (activation + a contract), not invention.

================================================================================
THE UNIFYING PRINCIPLE (recorded in the GRC ledger, topic
  deterministic-decomposition)
================================================================================
All four ideas reduce to one principle that is ALREADY ELAI doctrine: push
  every content-bearing and sizing decision out of model discretion into
  deterministic harness machinery; leave the model only the irreducibly
  generative/selection acts, and gate even those against frozen, measured
  criteria. (generator proposes / gate disposes; off-agent + frozen evidence;
  missing-evidence = FAIL; measure-the-objective-not-the-proxy; LLM-in-gate is
  advisory + cross-family.) Captured as the reviewer lens in
  docs/strategy/ELAI-DETERMINISM-DOCTRINE.md. The user's instinct is correct
  and consistent; the work is choosing which mechanism is sound (above) and not
  reinventing the parts already built.

================================================================================
NET-NEW vs ALREADY-BUILT (the decision-relevant summary)
================================================================================
NET-NEW, worth a plan:
  1. Deterministic NODE-ADMISSION GATE tied to the MEASURED lane ceiling — the
     wedge; today only soft warnings (SCOPE-VIOLATION / CONCEPT-OVERFLOW /
     PHASE-OVERFLOW) + the coarse 4E heuristic exist.
  2. Span-fingerprint CHEAP EXTRACTIVE GATE over a harness-fetched hashed
     corpus, feeding the existing entailment layer.
  3. Hash-pinned, ABLATION-GATED decomposition policy (DGM-style; only change
     on measured improvement).
EXTEND / ACTIVATE, not invent:
  - claim grounding (H-53 / F-12 / GROUNDING VERIFICATION), DGM gating
    (Self-Mod / H-43), research find/fetch split (E-48 / sidecar /search /
    Tier-C IPC proxy), context calibration (J-34).

================================================================================
OPEN QUESTIONS ROUTED TO DEEP RESEARCH (low training confidence, design-
  changing)
================================================================================
Q1: prior art on deterministic/programmatic task decomposition + PRE-EXECUTION
  token/context cost estimation for agent steps (is right-sizing-before-running
  a solved/partially-solved problem, or open whitespace?). This is the wedge;
  not guessed.
Q2: cheap/deterministic extractive + claim grounding WITHOUT a second large LLM
  (MinHash/n-gram + distilled NLI like MiniCheck, attributed-QA, closed-set
  citation binding); and whether token/span provenance is sound or claim-level
  is the floor.
Both written as ONE anonymized prompt with a mandatory attack-the-design
  section (workflow/reasoning/deep-research-decomposition-grounding-2026-06-16
  .md). Recommendation given: one deep-research pass on both; skip oracle
  (oracle only if the user wants Q2 fast and narrower). Everything else
  (token-hash verdict, Godel->DGM, sizing-vs-content) is training-confident and
  not routed.

================================================================================
DECISIONS / STATE AT CAPTURE
================================================================================
- No code written, no plan letter authored, no FSM transition. Read-only
  reasoning + two reference artifacts + this capture.
- Files produced this session: docs/strategy/ELAI-DETERMINISM-DOCTRINE.md (the
  reviewer lens), workflow/reasoning/deep-research-decomposition-grounding-
  2026-06-16.md (the anonymized prompt), this file.
- NEXT (when research returns): reconcile findings against the 4E sizer / J-34 /
  H-53 / F-12 / B-29, then decide whether net-new items 1-3 become a plan
  letter. If they do: new numbered plan files only (never amend old), inherit
  W/GD/HD/TD doctrine, author a _grounding map with live grep evidence.

================================================================================
APPEND 2026-06-16 — PERPLEXITY DEEP RESEARCH (sonnet oracle) + RESEARCH-MODES
  REASONING
================================================================================
Source: 1 sonnet oracle, ~6 WebSearch / 10+ WebFetch, 14 cited sources. Strongest
  are T1 Perplexity research-blog primaries: query-aware context compression
  article, "Search as Code" article, DRACO benchmark. VERIFIED = oracle
  web-grounded; INFERRED = oracle's labeled inference; numbers are vendor-
  reported (design-rationale, never an ELAI number per HD-1/ID-5).

PERPLEXITY ARCHITECTURE — the confirmations that matter (VERIFIED unless noted):
- Raw fetched content NEVER enters the reasoning model. A separate EXTRACTIVE
  (not generative) compressor selects verbatim spans: >10,000 tok/doc -> ~200
  tok/doc (>50:1), <20ms p99, runs UPSTREAM of synthesis. Extractive is the
  deliberate choice for citation fidelity. [T1 Perplexity compression article]
- Fetch is owned by Perplexity infrastructure (own crawler + Vespa index +
  pplx-embed 0.6B/4B rankers), NOT the model. The synthesis model (Sonar Deep
  Research, 128K ctx) sees only ranked, pre-cited snippets. [T1/T2]
- "Search as Code" (Sept-2025+): the model emits a PYTHON PROGRAM against a
  search SDK (web_many concurrency=12, extract_many(schema), rank, dedupe_by)
  instead of calling raw fetch tools. Model = control-plane code generator, not
  a browser agent. CVE case: 288,700 -> 42,900 tokens (85% cut) by filtering at
  retrieval time, not in context. [T1 Search-as-Code article]
- Intermediate state (hit lists, coverage maps, extraction results) is
  SERIALIZED TO FILESYSTEM between turns, not kept in the model window. [T1]
- 5-stage ranking BEFORE the LLM: intent -> retrieval (BM25 + dense + hybrid)
  -> assessment (quality/trust) -> L1-L3 rerank (~0.7 threshold; if too few
  pass, DISCARD whole set and re-retrieve) -> diversity selection. System-
  driven, not LLM. [T2 ZipTie]
- Citations STRUCTURAL, embedded BEFORE generation (numbered markers in the
  prompt) — matches H-53 exactly. Token-level constrained decoding NOT
  disclosed. Field-wide deep-research citation fabrication 66% total / 27%
  partial (NeurIPS-2025 study); extractive verbatim spans are the documented
  mitigation. [T2]
- Speed levers: parallel fetch (concurrency 12) + a small/fast extraction model
  (<20ms vs hundreds-of-ms synthesis -> INFERRED separate, smaller model) +
  serial reasoning over compressed payloads. "Parallel fan-out is the primary
  latency lever." [T1/T2]

WHAT THIS DOES TO THE "OFFLOAD/LOAD CONTEXT FAST" IDEA (the user's hunch,
  resolved):
- The lever is NOT KV reuse. It is three things Perplexity does and ELAI can:
  (1) per-source EXTRACTIVE compression to ~200 tok before anything reaches the
  reasoning model; (2) EXTERNAL state serialization between research steps;
  (3) reasoning context never holds raw pages. KV-cache reuse on the ELAI local
  lane is currently a NO-OP anyway (sidecar PrefixKVCache bypassed on the
  streaming path, H-50). So "fast context swap" == "keep per-step context tiny
  via extraction + external state," confirmed by the strongest available
  primary source.
- Hard sizing fact for the local ceiling: ~200 tok/doc means a 25-40K local
  model fits ~100-150 sources of CONTENT before query/trace/output reserve.
  Perplexity needs its full 128K even AFTER 50:1 compression at ~200 sources —
  i.e. compression is mandatory, not headroom. This is the SAME node-admission
  budget (Idea B) applied to research STEPS: each research step is a bounded-
  context unit; oversized -> compress harder or split, never overflow.

RESEARCH-MODES DESIGN (now grounded; the cog invariant made concrete):
  Invariant for ALL modes: model only (a) formulates queries / (b) ranks-or-
  emits-a-retrieval-plan; harness fetches (parallel) + extractively compresses +
  fingerprints + pre-binds citations; only distilled grounded spans + external
  state enter reasoning context.
  - FAST: one ranking pass -> top-K harness-fetched + extracted -> one grounded
    synthesis, no iteration (~ the 3K FAST tier).
  - STANDARD: shallow FlashResearch (breadth 2-3, depth 3).
  - DEEP: full FlashResearch (breadth 4-5, depth 5) + iterative refine with
    external per-step state — the Perplexity-shaped loop.

NET-NEW vs EXISTING for research modes (decision-relevant):
  EXISTS (reuse, do not reinvent): FlashResearch tree (4D), BM25 + dual-vector +
    RRF + cross-encoder reranker + SourceQuality 5-tier (1B-1I/1G/4D), A-RAG
    sentence-level snippet extraction (1E — an extractive compressor, but for
    CODE), SESSION DAG external state + lossless trim (3H/B-9), SSB journal as
    external store, H-53 citation pre-binding, E-35 CodeAct (the Search-as-Code
    substrate), Tier-C IPC fetch proxy (4D) + sidecar /search.
  NET-NEW: (a) QUERY-AWARE EXTRACTIVE COMPRESSOR FOR WEB SOURCES (~200 tok/doc,
    extractive not generative) — the key missing piece; A-RAG's code-snippet
    extractor is the closest analog to extend; (b) wire the existing retrieval/
    rerank stack into the WEB-research lane (E-48 is DARK, no prod caller);
    (c) the discard-and-re-retrieve-on-low-quality-threshold ranking gate;
    (d) OPTIONAL: Search-as-Code retrieval planning as a CodeAct (E-35)
    extension — weigh against ELAI's deterministic-assembly preference (4E
    AutoB2G chose deterministic over LLM-driven SOCIA); likely a measured A/B,
    not a default.
  CAVEAT/RIDER: vendor numbers (>50:1, <20ms, 85% CVE cut, 200/doc) are design-
    rationale only, never claimed as ELAI numbers; any ELAI compressor ships
    with its own measured fixture (W-9/GD-2).

STATUS after this append: oracle done; deep-research (Q1 decomposition/cost-
  estimation + Q2 grounding) still PENDING. No plan authored. Waiting on the
  deep-research output to reconcile Idea-B sizing + Idea-A grounding, then
  decide the plan letter (which would now plausibly carry BOTH a decomposition
  node-admission gate AND a research-modes extractive-compression lane).

================================================================================
APPEND 2026-06-16 (b) — "WOULD KV REUSE BE BETTER?" (reasoned pre-research)
================================================================================
Question: if we COULD offload/reload the local model's KV cache fast across
  research steps, would that beat the extractive-compression + external-state
  approach?
Verdict: NO as a substitute, YES only as an additive prefix-latency/cost lever.
  KV reuse and compression are ORTHOGONAL — speed vs capability — not either/or.
Reasoning (TRAINING + VERIFIED ceilings):
- KV reuse cannot raise the ceiling. KV is fixed per token (3B ~1KB/tok, 7B
  ~56KB/tok); the measured ~40K/~25K ceilings ARE the KV-VRAM + attention limit.
  200 pages * ~10K tok = 2M tok of KV is unfittable (7B -> ~112GB). You MUST
  compress content to fit the window regardless of reuse. Reuse only avoids
  recomputing what already fits; it never makes more fit.
- Reusing KV on the GROWING part is the failure mode: accumulating context to
  reuse it hits the ceiling in 2-3 steps AND triggers context-rot / lost-in-
  the-middle (the deep-research insight flagged this). Perplexity's filesystem-
  serialization-between-turns exists precisely to avoid monotonic growth.
- Where KV reuse genuinely helps: the BYTE-STABLE PREFIX only (system + tools +
  rules). Caching its KV cuts prefill latency, and on cloud also ~10x INPUT
  COST — which is why ELAI already does it (H-1/H-2 prompt caching WIRED; HD-2
  "never bust the byte-stable prefix"). On local it's a no-op only because the
  sidecar bypasses PrefixKVCache (H-50 leg); fixing that buys prefix prefill
  latency. Additive speed, NOT capability.
- It cannot sit on the trust spine: a paged KV cache is opaque, hardware-
  dependent, non-reproducible state (G-10 KV numerics differ across GPU/quant;
  RRA wants re-derivable action-streams, not GPU tensors). KV reuse must be a
  DISPOSABLE cache, never journaled state. The artifact you reload between steps
  is the COMPRESSED external state (~200-tok spans + coverage map), never the
  raw KV — you never page the bloated step back in, you start the next step lean.
- Best architecture COMPOSES all three: compress content (mandatory, capability)
  + external state between steps (mandatory, capability) + cache stable-prefix KV
  (optional, speed, disposable). KV reuse is "better" ONLY as that third lever.

================================================================================
APPEND 2026-06-16 (c) — WHY WE WAITED FOR THE DEEP RESEARCH (the purpose)
================================================================================
The research is the gate between reasoned-design and authored-plan (grounded-by-
  default: no plan letter until external claims are grounded). Two open,
  design-changing unknowns my training could not settle:
  Q1 (load-bearing for Idea B / the wedge): is pre-execution token/context cost
     estimation ACCURATE ENOUGH TO GATE ON? If unreliable, the node-admission
     gate must be redesigned as bound-and-escalate (coarse predict, re-split /
     escalate on overflow) rather than predict-precisely. Cannot answer from
     training.
  Q2 (Idea A): concrete cheap-grounding stack WITHOUT a frontier LLM — actual
     model names/sizes/latencies/soundness for span-fingerprint + small-NLI +
     closed-set citation (local-lane + cost critical). Preview insights point
     yes (MiniCheck ~400x) but needed the full survey + the ATTACK-THE-DESIGN
     section to learn failure modes and where determinism-over-judgment has been
     TRIED AND FAILED, so we don't author a plan into a known trap.
Secondary purpose: the attack-the-design section names the single cheapest
  falsifying experiment per design — that becomes the first OBSERVE-ONLY step of
  any plan letter, not a leap to default-ON.

================================================================================
APPEND 2026-06-16 (d) — DEEP RESEARCH RECONCILIATION (the payoff)
================================================================================
Source: user's GPT/Perplexity deep-research report "Deterministic Agent
  Engineering: Decomposition, Cost Estimation, and Cheap Grounding" (62 cited
  refs, E/B/R structure + attack-the-design). arXiv ids below are FROM THE
  REPORT and MUST be WebSearch-verified before any enter a plan (house rule; 2
  bad cites historically per B-40/B-42). Several are 2606.* (June 2026) — verify
  existence explicitly.

ANSWER TO THE LOAD-BEARING UNKNOWN (Q1 / Idea B): pre-execution TOTAL token cost
  is NOT predictable (Pearson r<0.15, same-task 30x variance; Stanford/OpenReview
  "How Do Coding Agents Spend Your Money?"). Pre-execution INPUT tokens ARE exact
  (tokenizer). => the node-admission gate is REDESIGNED to BOUND-AND-ESCALATE,
  NOT predict-precisely:
  - GATE INPUT (deterministic, exact): dependency-cone token lower-bound via the
    DKG / B-29 impact API; admit to local lane iff lower-bound <= ~0.4 *
    measured_ceiling (the 0.4 figure is the report's; ELAI sets its own from
    J-34). Lower-bound already over budget -> split unconditionally.
  - RE-SPLIT (deterministic): community-detection partitioning on the dependency
    graph (Co-Coder arXiv 2606.00953: +14% pass / 2.1x speedup / -35% cost,
    largest gains on dependency-dense repos; the benchmarked reference).
  - OUTPUT (uncontrollable): per-session output-token cap as a CIRCUIT BREAKER
    (ELAI has circuit breakers in 3C/4G), never a precise gate. Output is
    fat-tailed (P95/P50 > 5x).
  This UPGRADES the existing 4E Stage-3.5 Phase Sizing (replace the coarse
  files_touched*avg_file_tokens*0.6 heuristic with the cone-token lower-bound vs
  the J-34 measured ceiling).

CONFIRMED OPEN WHITESPACE (de-risks the wedge): the survey states verbatim "no
  published work converts a measured degradation curve into a per-model context
  budget gate for an agent sizer ... a genuine research gap." That IS J-34 +
  node-admission. Also Q1-c: "no published system uses 'measure predicted
  context -> gate -> re-split' as a first-class mechanism for coding agents."
  So the wedge is novel AND grounded, not reinvention.

THE REFINED BOUNDARY (Attack 2, important nuance to the doctrine): a static
  sizer reliably bounds the scope SUPERSET (files that COULD be affected); it
  does NOT give the change SUBSET (what actually needs changing) — that stays
  LLM judgment. Pure program-analysis decomposition BREAKS on: cross-cutting
  concerns (audit-logging = whole-codebase cone but atomic intent), semantic
  contracts invisible to the AST (protobuf/REST/DB-schema coupling), intent
  disambiguation, dynamic dispatch / duck typing (Python/JS call graphs are
  conservative-incomplete), and novel architectural patterns (no existing
  edges). => doctrine refinement: deterministic owns scope BOUNDS + admission;
  LLM owns the change selection WITHIN the bounds. Consistent with generator-
  proposes/gate-disposes (the gate disposes on SIZE, not on the change content).

THE BIG CAVEAT FOR THE LOCAL-FIRST PREMISE (Attack 4, must surface to user):
  SWEnergy (ICSE 2026) — small local models (Gemma-3 4B, Qwen-3 1.7B) hit
  near-zero agentic task resolution (4% best, 0% others) despite 9.4x energy.
  "Deterministic orchestration without sufficient model capability in leaf nodes
  produces an expensive zero." => sizing + compression make a local model FIT
  the context, NOT CAPABLE of the task. The I-7 cloud-escalation path is LOAD-
  BEARING, not optional; local lane is for bounded/simple leaves, hard leaves
  escalate. Honest residue, not a blocker, but it bounds the local-first claim.

Q2 / Idea A — GROUNDED AND MOSTLY MAPS TO EXISTING ELAI:
  - Extractive fingerprint = cheap FIRST PASS only: three-tier substring(exact,
    Rabin-Karp/KMP) -> fuzzy(0.8 token-overlap/edit-dist) -> semantic(0.85
    cosine) -> discard. Token-level VACUOUS (confirmed); sentence/n-gram>=5
    meaningful; plagiarism floor 8-10 consecutive words; false-negatives
    dominate on paraphrase/synthesis -> NEVER the sole gate. Maps to GROUNDING
    VERIFICATION substring/source check (2B) + E-54 n-gram filters.
  - Claim-level WITHOUT a frontier LLM (concrete models): MiniCheck-FT5 770M
    (GPT-4 accuracy on LLM-AggreFact, 400x cheaper), Bespoke-MiniCheck-7B (SOTA,
    >500 docs/min on one A6000-48GB, needs RunPod/2x3090), Paladin-mini 3.8B on
    Phi-4-mini. DeBERTa-large-MNLI + lexical-match flag ~ GPT-4o 89.9% on
    long-form QA; DistilBERT-125M serviceable. Claim decomposition (fine-grained
    > sentence by 10-15% coverage) via T5/BART. => DIRECT candidates for F-12's
    empty OnnxNliJudge slot: MiniCheck-FT5 (ONNX, fits local) or Paladin-mini;
    DeBERTa+lexical = the F-12 DeterministicNliJudge Tier-1 already shipped.
  - Closed-set citation binding: prompt-level (cite only from frozen ID set +
    post-hoc verify ID exists) reduces ghost cites (free-gen 14-95%; partial-
    evidence 48%, unsupported 65%). HARD form = graph-constrained decoding
    (KG-Trie prefix tree, invalid completions -> logit -inf) makes ghost cites
    STRUCTURALLY IMPOSSIBLE. That is EXACTLY H-53 citation pre-binding + E-35
    XGrammar; the report gives the mechanism (closed-set grammar over retrieved
    IDs). Abstention-under-partial-evidence is the quality cost -> ties to D-16
    conformal refusal.
  - Harness-fetch vs model-fetch: LLM-CITE (harness fetches model-PROPOSED URLs,
    entailment vs FETCHED content, 45x cheaper than search-based), CoRAG,
    Blueprint-First. Confirms the cog split; no published controlled ablation of
    harness-fetch vs model-tool-fetch hallucination (open whitespace).

OVER-DETERMINISM FAILURE MODES TO DESIGN AROUND (Attack 4): SeqCV infinite-split
  loop on non-decomposable subtasks (need a split-depth cap — ELAI has max
  split depth 2); RAG-HAT — over-constrained citation on PARTIAL evidence makes
  the model fabricate MORE confidently (fix = abstention, D-16); ACONIC needs a
  CSP reduction that is itself non-deterministic for open-ended tasks (so
  CSP-treewidth decomposition is NOT a general decomposer — narrow use only).

THE TWO CHEAP FALSIFICATION EXPERIMENTS (become the OBSERVE-ONLY first steps of
  any plan letter, never skipped to default-ON):
  EXP-1 (Idea B): 50 SWE-bench tasks; correlate dependency-cone token size vs
    actual agent token spend (OpenHands). r>0.5 = the cone has actionable sizing
    signal; r<0.2 = cone is a hard FLOOR only (consistent with r<0.15 total).
  EXP-2 (Idea A): 100 (source-snippet, generated-claim) pairs FROM THE ELAI
    CODEBASE (50 entailed / 50 not); run MiniCheck-FT5 + Bespoke-7B; balanced
    accuracy < 75% = does NOT transfer to the code domain without fine-tune
    (LLM-AggreFact is general-text; code-claim transfer is the #1 residual risk).

REPORT'S RECOMMENDED BUILD ORDER (aligns with ELAI substrate):
  1. Static dependency-graph sizer (tree-sitter + import-graph cone, tokenize,
     sum = exact input lower-bound; gate >0.4x ceiling; split at community
     boundary). Cheapest, fully deterministic. -> ELAI: upgrade 4E Phase Sizing,
     consume B-29/DKG, ceiling from J-34.
  2. Harness-fetched ID-assigned evidence + closed-set citation enforcement +
     exact-substring on quotes. Near-zero cost. -> ELAI: H-53 + E-48 wiring.
  3. MiniCheck-FT5/Bespoke-7B grounding layer (claim-decomp -> NLI vs fetched
     evidence). VALIDATE ON DOMAIN FIRST (EXP-2). -> ELAI: F-12 OnnxNliJudge.
  DEFER: pre-exec OUTPUT prediction (r<0.15), ACONIC CSP decomposition.

NET DECISION POSTURE (for the user): the evidence now supports authoring a plan
  letter, but it is gated on EXP-1/EXP-2 as observe-only step 1. The plan would
  carry THREE composed legs, all extensions of shipped substrate, none
  greenfield: (A) bound-and-escalate node-admission gate [upgrade 4E + J-34 +
  Co-Coder re-split + circuit-breaker], (B) research-modes extractive-compression
  lane [E-48 wire + A-RAG-extractor extend + F-12 OnnxNliJudge=MiniCheck], (C)
  the cheap grounding stack [three-tier fingerprint -> MiniCheck -> H-53/E-35
  grammar-constrained closed-set citation]. SWEnergy caveat: local lane is for
  bounded leaves; I-7 escalation is load-bearing. NOT YET AUTHORED — awaiting the
  user's call on whether/which plan letter, and arXiv-id verification first.

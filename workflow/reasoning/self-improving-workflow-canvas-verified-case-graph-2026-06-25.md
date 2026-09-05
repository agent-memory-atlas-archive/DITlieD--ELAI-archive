SELF-IMPROVING WORKFLOW CANVAS — THE VERIFIED CASE GRAPH
Date: 2026-06-25
Lens: spec-eval AS-SPECIFIED (everything ELAI plans is treated as built for design reasoning;
  shipped-vs-planned is a separate readiness note, never a design mark-down).
Grounding posture: every design move below is tied to (a) an existing ELAI seam named by plan-id
  in ELAI-FEATURE-MAP.md and/or (b) an external prior-art family verified this session. Ungrounded
  conjecture is labelled.

--------------------------------------------------------------------------------
0. THE IDEA, RESTATED
--------------------------------------------------------------------------------
The user's idea: make workflows/sessions into canvas nodes too. The moment a bug is PROVED fixed,
that fixed-bug becomes a node. The node only improves as it is used. Over time the system fixes the
same class of problem faster, with fewer iterations, adapts to the user, and the nodes wire
themselves to other nodes and improve everything. "Like a memory system, but for an entire
workflow." Reference inspiration: DSPy, RubyLLM, the deep-research-report(6).md, and ELAI's own
auto-improvement / memory machinery (DGM etc).

--------------------------------------------------------------------------------
1. HEADLINE FINDING (the honest spec-eval verdict)
--------------------------------------------------------------------------------
This is NOT a net-new mechanism for ELAI. It is a SYNTHESIS. Every ingredient already exists in the
plan corpus, but the pieces live apart and none of them is projected on the canvas as a reusable,
self-wiring memory of proven fixes. The contribution the user is reaching for is the UNIFICATION
plus the CANVAS PROJECTION, not a new algorithm.

The scattered ingredients, all already specced:
- J-10 INSIGHT BACKPROPAGATION TREE MEMORY: extract a 1-3 sentence insight per audit report,
  re-synthesize it up the ancestor chain to the root, inject lineage down + a CONSTRAINTS BLOCK into
  future ideation; provenance hash of the source SSB rows; the LLM writes the text, never the
  verdict. This is already "a verified lesson becomes a durable, propagating node."
- I-12 EVIDENCE-GATED PLAYBOOK CURATOR: promote a procedure ONLY after >= k SSB-verified outcomes,
  then export it to CLAUDE.md / AGENTS.md / skills. This is already "a proven fix graduates into a
  reusable artifact."
- C-25 TEMPLATE CRYSTALLIZER [WIRED]: 3 successes -> crystallize a compiled pipeline; production
  callers live. This is already "repeat success hardens into a fast path."
- C-45 / C-47 CASE-BASED CALIBRATION: OrchestratorCaseBase + credit-attribution config. Classic
  case-based reasoning substrate.
- C-49 EXPERIENCE DISTILLATION [DARK]: iterative prompt-policy distillation from experience.
- J-31 CANVAS SYSTEM-COMPOSITION MODE: 6th canvas mode where nodes BIND to plan units / wiring
  contracts / MCP servers; bindings + verdicts persist ONLY as SSB rows (the journal is the store,
  the canvas re-derives, no parallel state); edges carry checker-derived verdicts
  declared/wired/verified/broken; build dispatches the bound unit through the normal FSM (the
  "anti-n8n" line — the canvas shows and steers, it is not a second runtime).
- J-60 CANVAS LAYERED OBSERVABILITY MODE: a PROCESS altitude that projects ELAI's own FSM +
  agent_pool workers as journal-derived nodes (status from rows only, re-derive byte-identically).
- J-61 CANVAS REDIRECT LOOP: a closed verb set (CONNECT/CUT-EDGE/REORDER/REBIND/AGENT-CONTROL) as
  journaled commands to the single-writer engine.
- I-4 EVIDENCE-RANKED SEARCH LADDER: select candidates BY RECEIPTS, not by an LLM judge.
- The DGM family: B-23 SELF-MODIFICATION CORE (DGM sigmoid parent-select, archive-style), and
  H-43..H-48 DGM VERIFICATION FILTERS (the un-gameable fitness/trap layer).

So the user has independently re-derived the same target the J-10 + I-12 + J-31 + J-60 line is
already converging on. The right framing is one unifying object that ties them together:

  THE VERIFIED CASE GRAPH (VCG): a canvas-projected, journal-grounded layer whose nodes are PROVEN
  fixes (and proven plan-units, sessions, recurring tasks), wired to each other by deterministic
  evidence edges, retrieved and reused to cut future iterations, and reinforced ONLY by
  execution-grounded outcomes.

--------------------------------------------------------------------------------
2. EXTERNAL GROUNDING (what the field says, verified this session)
--------------------------------------------------------------------------------
From deep-research-report(6).md ("Self-Improving LLM Workflows Beyond DSPy"), the field splits into
five families, and the report's central warning is "say which one you are doing." The user's idea
touches three of them at once, which is fine if kept distinct:
- Family 4 MEMORY / SKILL / CONTEXT: MemGPT/Letta, Mem0, Voyager's ever-growing skill library, and
  the EXPERIENCE COMPRESSION SPECTRUM paper that formalizes a continuum: raw episodic trace -> skill
  -> hard rule. The VCG is exactly a walk along that continuum (a proven fix starts episodic, earns
  its way to a skill, and a repeatedly-proven invariant becomes a rule).
- Family 3 WORKFLOW / AGENT-GRAPH OPTIMIZATION: AFlow, ScoreFlow, MaAS, G-Designer — optimize the
  graph/topology, not just the prompt. The "nodes wire themselves to other nodes and improve
  everything" part lives here. The report flags this as the most promising 2025-2026 direction AND
  the highest-variance / most research-y — a reason to keep ELAI's version deterministic and gated.
- Family 5 EVALUATION-FIRST: the report's strongest practical claim is that self-improvement is
  bottlenecked by the metric and the trace store, not the optimizer; "without them, self-improvement
  usually becomes self-deception." It recommends deterministic checks FIRST (tests, parsers, lints,
  compilation, citation validity, tool success), rubric judges second, human third. This is verbatim
  ELAI's TD-3 (LLM is advisory, never the load-bearing gate) and F-4 cant-lie. ELAI is already on
  the right side of the report's #1 risk.

DGM (Darwin Godel Machine, arXiv:2505.22954, Sakana/UBC/Vector): the closest external analogue to
the user's idea. DGM keeps an ARCHIVE of discovered agent solutions (open-ended, not a single
evolving line), selects an entity from the archive, modifies it, and KEEPS IT IF IT IS INTERESTINGLY
NEW AND EMPIRICALLY BETTER on real coding benchmarks (SWE-bench 20.0 -> 50.0, Polyglot 14.2 ->
30.7). The archive-grows-and-is-selected-from shape IS the user's "fixed bug becomes a node, reused
later." The load-bearing detail: DGM validates every archived candidate on real benchmarks. The
archive is only as good as the empirical filter in front of it.

DSPy / GEPA: DSPy "compiles" an LM program from a metric + training examples; it learns which
verified demonstrations best teach the task and reuses them. GEPA (reflective prompt evolution over
full traces, Pareto-combining complementary winners) is the strongest post-MIPRO version and
out-performs RL with far fewer rollouts. RELEVANCE: the VCG is the demonstration / trace STORE that
a GEPA-style compile feeds on. ELAI already has GEPA canonical (E-8 SELF-EVOLVING PROMPTS) and the
crystallizer (C-25). HARD CONSTRAINT FROM THE LEDGER: dspy is REJECTED as a framework swap (GEPA
already canonical; no newer optimizer found). So the steal is the IDEA (compile reusable behavior
from verified traces against a real metric), never the dependency.

RubyLLM: a Ruby provider-abstraction library (one API over GPT/Claude/Gemini/... + tools, structured
output, conversations, agents, Rails-backed chat). Honest fit verdict: OFF-WEDGE for this idea. ELAI
is Rust, and provider abstraction is already B-37 MULTI-RUNTIME BACKEND (11 backends, broader than
RubyLLM). The only transferable thing is DX/ergonomics (one clean call surface), which is a CLI/API
polish concern, not a self-improvement mechanism. Do not let RubyLLM pull the design toward "wrap
more providers"; that is solved.

--------------------------------------------------------------------------------
3. THE DESIGN — VERIFIED CASE GRAPH, MECHANISM BY MECHANISM
--------------------------------------------------------------------------------
Each mechanism names the existing ELAI seam it rides so this is composition, not invention (GD-1
compose-not-restate).

3.1 NODE BIRTH CONDITION (the "moment a bug is proved fixed" trigger)
A case node is MINTED ONLY when a fix is gate-PROVEN, never when the model merely claims success.
The proof already exists in the journal:
- F-4 TEST-INTEGRITY CANT-LIE GATE [WIRED]: blake3 stdout_hash on the test outcome; the auditor
  cannot emit passed=true without the orchestrator-captured hash matching. A red->green transition
  with a matching hash is the birth event.
- G-2 REPRODUCIBLE-RUN ARTIFACT (RRA) [WIRED]: the sealed bundle (SSB stream + seed + model pin +
  gate verdict hashes + blake3 root) is the node's immutable evidence payload.
- I-37 EARS REQUIREMENT OBJECTS [WIRED] + J-16 BLAST-RADIUS cone: the node records WHAT was required
  and WHICH symbols the fix touched.
Net: minting is a deterministic projection of proof rows that already land. No new trust assumption.

3.2 NODE CONTENT (a typed case, not a blob)
trigger_signature (symptom + failing test id + error taxonomy class from failure_taxonomy.py),
blast_radius_cone (DKG symbols, from B-29 impact API), the fix delta (structural edit / SEARCH-
REPLACE), proof_receipt (F-4 hash + RRA root), provenance_hash (the source SSB rows, per J-10),
and a 1-3 sentence human insight (J-10, LLM writes the text, never the verdict). Typed so it is
retrievable and so the "counts/edges" are deterministic, not vibes.

3.3 RETRIEVAL + REUSE (the "fix it faster / fewer iterations" engine)
On a new task, retrieve similar case nodes over the trigger_signature using the SHIPPED retrieval
stack (BM25 + dual-vector + reranker + RRF, phase-1B/1C/1G), optionally the J-38 mid-task callable
hybrid_search. Inject the top matches as DSPy/GEPA-style demonstrations into the planner/executor
context (this is the I-12 playbook, made retrieval-time and per-task). This is the mechanism that
cuts iterations: the system has seen this class of bug before and brings the proven fix forward.
GATING (critical, see section 4): reuse-into-prompt is the RISKY leg and ships LAST, behind the
execution-grounded fitness filter and OBSERVE-ONLY first (GD-4).

3.4 SELF-WIRING (the "nodes wire themselves to other nodes")
Edges are DERIVED deterministically, never LLM-authored (the J-31 doctrine: edges carry checker-
derived verdicts, the canvas re-derives from the journal, no parallel state). Edge sources:
shared DKG symbols / blast-radius cone overlap (B-29), co-occurrence in the same session/plan-unit
(SSB), same failure-taxonomy class, supersession (a newer fix on the same symbol supersedes an older
one). An edge's "weight" is a recomputable function of journal rows (GD-2), so the graph is
auditable and reproducible, not an opaque learned blob.

3.5 IMPROVE-WITH-USE (the reinforcement, done safely)
A case node's standing rises or falls by USAGE OUTCOME, scored by machinery ELAI already has for
exactly this anti-gaming job:
- H-43 EXECUTION-GROUNDED FITNESS (pass-rate over frozen tests), H-48 BAI bandit ACTIVATED on the
  real reward, H-45 worst-case/lineage-diverse promotion scoring with PRMBench-calibrated tau.
- RD-1 (reward credits abstention, penalizes confident-wrong at t/(1-t)): a node that, when
  surfaced, leads to a gate-PROVEN pass on a HELD-OUT recurrence gains weight; a node that misleads
  (reused, then the gate failed) loses weight; a node that correctly says "I don't apply here" is
  not punished.
This is the difference between "improves with use" (the user's words) done as DGM-with-a-real-filter
versus DGM-with-no-filter (which provably collapses — section 4).

3.6 THE EXPERIENCE COMPRESSION SPECTRUM (memory -> skill -> rule)
A case node starts as an EPISODIC memory (one proven fix). After I-12's k verified reuses it is
promoted to a SKILL (exported to a skill file / playbook). A pattern proven across many cases and
symbols can be proposed (human-gated, L3) as a HARD RULE (a rules-engine row, phase-3F, or a
CLAUDE.md line). This is the deep-research report's spectrum and ELAI's SELF-MODIFICATION CORE
L1/L2/L3 ladder (B-23) at once: data-table promotion is automatic, config is staged-eval, structural
is human-only. The canvas makes this promotion VISIBLE (a node lighting up from episodic -> skill ->
rule as evidence accrues).

3.7 CANVAS PLACEMENT (the user's "make sessions into nodes")
The VCG is a new ALTITUDE on the existing canvas, sitting beside J-31 (system/plan-unit altitude)
and J-60 (process/FSM-and-agents altitude): a MEMORY/CASE altitude. Same doctrine throughout: the
SSB journal is the store, the canvas re-derives byte-identically (GD-2), nodes bind to real artifacts
(proof receipts), and steering goes through the J-61 closed verb set + the FSM (anti-n8n). Sessions-
as-nodes is literally J-60's projection; case-nodes are the durable residue those sessions leave
behind once a fix is proven.

3.8 PER-USER ADAPTATION (the "adapt to the user")
Namespace the VCG per user/project via J-59 PER-SEAT MEMORY + TRUST ISOLATION (keyed by the
UsageTurnEvent.identity) over E-5. The graph then learns THIS user's recurring bug classes, stack,
and preferences, and a poisoned/contradictory case in one namespace cannot leak into another.

--------------------------------------------------------------------------------
4. THE CENTRAL RISK (the most valuable part of this analysis)
--------------------------------------------------------------------------------
The user's exact phrase — "it only improves as it gets used" — is, taken naively, the precise failure
mode ELAI's Track-7 (H-43..H-48) and the whole determinism doctrine exist to prevent. State it
plainly so it is not lost:

A self-improving archive whose "usefulness" signal can be authored by the same model that produced
the artifact PROVABLY collapses. Evidence:
- ELAI's own DGM port shipped GAP-1: derived_alpha = blake3(config_bytes) at darwin_admit_loop.rs
  — "fitness of BYTES not performance," self-flagged unsound; and GAP-2: trap detection by keyword
  grep. Track-7 exists specifically to replace these two live placeholders. Re-introducing a
  byte-hash or an LLM-judged "this node was helpful" signal is named in the REJECTED ledger
  (VD-1/VD-2) — do not.
- Misevolve (arXiv:2509.26354): refusal rate fell 99.4% -> 54.4% AFTER an unfiltered self-evolution
  loop. F-10 measured security degrading +37.6% over 5 unchecked REFINE iterations.
- The deep-research report, Family 5: "without [a defensible metric + trace store + regression
  suite], self-improvement usually becomes self-deception"; LLM judges are "instruments to
  calibrate, not oracles to obey" (JudgeBench: many strong judges barely above random on checkable
  pairs).

THEREFORE the reinforcement signal in 3.5 must be:
(a) OFF-AGENT and EXECUTION-GROUNDED — did reusing the node cause a gate-PROVEN pass on a HELD-OUT
    recurrence (D-6 held-out store, H-47 eval firewall), not "it was retrieved" and not "an LLM said
    it helped";
(b) CROSS-FAMILY VERIFIED — promotion checked by a different pretraining family than the generator
    (D-13 / VD-4 stronger-than-generator);
(c) ABSTENTION-AWARE — RD-1, so "correctly did not apply" is not mistaken for failure.

SECONDARY RISK — MEMORY POISONING + STALENESS. A case node is memory, so it inherits memory's
failure modes:
- Staleness: a fix proven on commit X is wrong once the AST moves. Re-derive the case against current
  commit + AST identity BEFORE reuse (J-21 commit-aware/state-level validation, "citation-valid !=
  semantically-valid"); H-32 28d TTL on code-grounded facts; G-10 model-drift invalidation.
- Supersession: when a newer fix lands on the same symbol, HARD-FILTER the superseded node at
  retrieval — never narrate "[OUTDATED]" into context (H-31; the measured Graphiti +68%-token trap);
  never DELETE for replay integrity.
- Poisoning: write-time injection scan + retrieval-time authorization + write quarantine (J-21,
  MPBench arXiv:2606.04329: write-time scanning alone misses benign-at-write poison; E-5 MINJA
  defense).

THIRD RISK — AUDITOR-OBJECTIVITY FIREWALL. The VCG must NEVER be injected into the auditor / critic /
adversary context (H-31, arXiv:2510.09905). The case graph is generator-side help; if the
independent check starts reading the same "this usually works" memory, it stops being independent and
the whole evidence spine that makes minting trustworthy quietly dies.

--------------------------------------------------------------------------------
5. WHAT IS ACTUALLY NEW vs ALREADY PLANNED (anti-double-authoring honesty)
--------------------------------------------------------------------------------
Net-new (genuine whitespace, worth a plan unit):
- The UNIFICATION object: one Verified Case Graph that ties J-10 insight + I-12 playbook + C-25
  crystallizer + C-45/47 case-base into a single typed, journal-grounded store with one promotion
  ladder (episodic -> skill -> rule).
- The CANVAS MEMORY/CASE ALTITUDE: projecting proven fixes as first-class, self-wiring nodes beside
  J-31/J-60. No current plan draws the memory layer on the canvas.
- RETRIEVAL-TIME case REUSE as an explicit iteration-reducer with an execution-grounded
  reuse-fitness gate (I-12 promotes to skills; it does not currently feed per-task retrieval behind a
  held-out reuse filter).
Already planned (compose, do NOT re-author):
- The insight extraction + propagation (J-10), the k-verified promotion (I-12), the crystallization
  (C-25), the case-base (C-45/47), the canvas substrate + doctrine (J-31/J-60/J-61), the anti-gaming
  fitness (H-43..H-48), the memory validation + isolation (J-21/H-31/H-32/J-59), the held-out + eval
  firewall (D-6/H-47), GEPA compile (E-8).
Honest one-liner: roughly 80% of this is wiring existing planned pieces into one visible object; the
20% that is new is the case-altitude canvas projection + the gated retrieval-time reuse loop.

--------------------------------------------------------------------------------
6. RECOMMENDED FIRST SLICE (vertical, per wiring PRE_GATE; observe-first)
--------------------------------------------------------------------------------
Do the SAFE half first; the risky half (reuse-into-prompt) waits for measured evidence (GD-4
OBSERVE-ONLY-first, the house posture for any loop that could game itself).
Slice 1 (mint + show, zero reuse): on every F-4-proven red->green + sealed RRA, MINT a typed case
node (3.2) as SSB rows; PROJECT it on a new canvas memory altitude (re-derived from the journal).
Measure: recurrence-hit-rate (how often a NEW failing task matches an existing trigger_signature).
This proves the graph would be useful without yet letting it influence generation.
Slice 2 (gated reuse): only after slice-1 shows a real recurrence-hit-rate, enable retrieval-time
injection of the top-matched case as a demonstration, BEHIND the execution-grounded reuse-fitness
gate (3.5 + section 4), default-OFF with a wire-dark-allowlist row + promotion plan (W-2/R9), and
NEVER into the auditor.
Slice 3 (promotion ladder + self-wiring): turn on I-12 k-verified promotion to skills and the
deterministic edge derivation; surface the episodic->skill->rule lighting-up on the canvas.

--------------------------------------------------------------------------------
7. WHERE DSPy / RubyLLM / DGM LAND (one line each, grounded)
--------------------------------------------------------------------------------
- DGM (arXiv:2505.22954): the archetype to copy IN SHAPE (grow an archive of proven solutions, select
  from it) and the cautionary tale to obey IN DETAIL (the archive is only as honest as the empirical
  filter; ELAI's filter is F-4 + held-out + cross-family, not a byte hash).
- DSPy/GEPA: the IDEA (compile reusable behavior from verified traces against a real metric) is the
  VCG's reuse engine; ELAI already has GEPA (E-8) + crystallizer (C-25) to consume the store. Do NOT
  adopt DSPy as a dep (ledger REJECTED; GEPA canonical).
- RubyLLM: off-wedge for self-improvement (provider wrapper, Ruby; B-37 already broader). Only a DX
  cue, not a mechanism.

--------------------------------------------------------------------------------
8. OPEN QUESTIONS FOR THE USER (decisions this design cannot make alone)
--------------------------------------------------------------------------------
- Scope of "proved fixed": only red->green test transitions (cleanest), or also proven plan-unit
  completions and proven research answers? Wider scope = richer graph but weaker per-class proof.
- Promotion authority: is episodic->skill fully automatic at k verified reuses (I-12), or does the
  user ratify each skill graduation? (skill->rule is human-only regardless, B-23 L3.)
- Canvas vs CLI first: the value (recurrence-hit-rate, faster fixes) is realizable from the journal
  with a CLI/`elai` surface before the canvas altitude exists; the canvas is the visibility layer,
  not the engine. Which gives the user more value sooner?

--------------------------------------------------------------------------------
SOURCES
--------------------------------------------------------------------------------
ELAI internal (ELAI-FEATURE-MAP.md, 2026-06-23 head): J-10, I-12, C-25, C-45/C-47, C-49, J-31, J-60,
  J-61, J-38, J-59, J-21, I-4, I-37, B-29, B-23, F-4, F-10, G-2, G-10, H-31, H-32, H-43..H-48, D-6,
  D-13, E-5, E-8, phase-1B/1C/1G/3F; K-SUMMARY_INDEX.md (sequencing/GTM context).
External (verified this session): deep-research-report(6).md (5-family taxonomy, Experience
  Compression Spectrum, eval-first doctrine); DGM arXiv:2505.22954 (Sakana, archive + empirical
  validation, SWE-bench 20->50); DSPy/GEPA (github.com/stanfordnlp/dspy, GEPA reflective evolution);
  RubyLLM (rubyllm.com, provider abstraction); Misevolve arXiv:2509.26354 (refusal 99.4->54.4 post-
  evolution); MPBench arXiv:2606.04329; auditor-memory-firewall arXiv:2510.09905.
Doctrine bindings honored: GD-1 compose-not-restate, GD-2 journal-recomputable, GD-4 observe-first,
  TD-3 LLM-advisory-never-the-gate, VD-1/VD-2 no-byte/self-confidence fitness, VD-4 stronger-than-
  generator, RD-1 abstention-credited, W-2/R9 no default-OFF graveyard, HD-2 deterministic masking.

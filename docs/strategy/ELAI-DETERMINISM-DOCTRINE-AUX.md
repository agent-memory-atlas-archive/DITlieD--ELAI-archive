ELAI DETERMINISM DOCTRINE — AUX: HOW TO THINK (the method, not just the lens)
Purpose: ELAI-DETERMINISM-DOCTRINE.md is the WHAT (a lens: is a given feature
  "model-as-cog" clean?). This AUX is the HOW (the reasoning METHOD that produced
  that lens). The determinism doctrine is ONE output of this method applied to ONE
  axis (decomposition + grounding). Apply the SAME method to EVERY other axis:
  research, planning, reasoning/GRC, memory, security, inference/routing, eval,
  UX. If you only pattern-match "determinism" you have missed the point -- the
  point is a way of reasoning about any ELAI improvement.
Read alongside: ELAI-DETERMINISM-DOCTRINE.md (the lens) and, as the canonical
  worked example of this method end-to-end, workflow/reasoning/
  decomposition-grounding-reasoning-2026-06-16.md + the J-39/40/41/42 plan files.
Written 2026-06-16. Plain text, ELAI house style.

================================================================================
THE STANCE (generalize "the model is a cog" to ANY subsystem)
================================================================================
For whatever subsystem the user brings an idea about, the first move is always
  the same question: WHAT IS THE MODEL DECIDING HERE, and which part of that
  decision could a deterministic mechanism make instead? Push the mechanizable
  part (sizing, admission, a verdict, a count, provenance, state, selection
  bounds) into the harness; leave the model only the irreducibly generative or
  selective act; gate even that against frozen, measured, model-independent
  criteria. This is not a determinism-specific rule -- it is the lens you apply
  to research (who decides what to fetch / what is "enough"?), planning (who
  decides node size / ordering / done?), reasoning (who decides depth / when to
  stop / whether a step is grounded?), memory (who decides what is stale / what
  to inject?), routing (who decides the tier?), and so on. Same question, every
  axis.

================================================================================
THE METHOD (the steps this thread actually ran; each: rule / why / worked example)
================================================================================

M1. GROUND BEFORE YOU CLAIM. Never assert how an ELAI subsystem works, or that
  an idea is novel, from training memory. Check the live code (codegraph / elai.map
  / scoped read), the FEATURE-MAP (status + composition edges), measured data
  (metrics/), and verify any external claim (arXiv id, library, API) before it
  influences a decision. Codebase-first: a question the code already answers is a
  finding you state, not a question you ask.
  WHY: training is a year stale and this repo is dense and self-referential; the
  default failure is confidently reinventing or misdescribing something that
  already exists.
  EXAMPLE: "E-48 is DARK with no caller" was my training-shaped assumption; the
  grounding pass found it is WIRED to the REPL and only the search backend is
  stubbed -- which made the research-modes plan narrower and correct.

M2. CALIBRATE CONFIDENCE OUT LOUD. Tag every claim: VERIFIED (searched/measured),
  high (code just read), training-unsearched (instinct, flag it). An unverified
  claim is allowed only if labelled and not load-bearing.
  WHY: the reader (and future you) must know which claims survived grounding and
  which are pending. Mixing them silently is how a wrong premise propagates.
  EXAMPLE: Co-Coder arXiv:2606.00953 -- the title kept appearing in search
  summaries but NO canonical arxiv.org page landed across 3 searches; marked
  PROVISIONAL/UNVERIFIED and explicitly NOT load-bearing, with the technique
  (community detection) cited independently so the design did not depend on it.

M3. STATE THE VERDICT BLUNTLY, INCLUDING THE CORRECTION. When an idea is wrong
  or mis-framed, say so directly and explain the right form. Do not fold under a
  confidently-stated idea; do not sycophant. Anti-sycophancy is a thinking tool:
  the user wants the real answer.
  WHY: a polite "yes, and..." that buries a fatal flaw costs a whole build cycle.
  EXAMPLE: "hash every token and check membership" -> stated plainly that this is
  VACUOUS (function/common tokens are in every source) and forbids synthesis, then
  gave the sound form (span fingerprint -> small-NLI -> closed-set citation).

M4. SEPARATE NET-NEW FROM ALREADY-BUILT. Before proposing anything, sort the idea
  into (a) reinvention of a shipped feature, (b) extension/activation of a partial
  one, (c) genuinely new. Reinvention is the default trap; check the FEATURE-MAP
  and code first. Most "new" ideas are extensions.
  WHY: ELAI has ~250 plan units; the cheap win is composing what exists, not
  rebuilding it. COMPOSE-NOT-RESTATE (GD-1) is a hard rule.
  EXAMPLE: "deterministic decomposition" turned out to be an UPGRADE of the
  existing 4E phase-sizing splitter + J-34 calibration, not a greenfield build --
  which is a far cheaper, safer plan.

M5. FIND THE ONE PRINCIPLE UNDER A CLUSTER OF IDEAS. When the user brings several
  ideas, look for the single through-line; it usually maps to existing doctrine
  and tells you which mechanism is sound.
  WHY: a unifying principle turns N scattered features into one coherent design
  and exposes which variants are sound vs which fight the architecture.
  EXAMPLE: four ideas (token-hash, decomposition, Godel-gating, research modes)
  all reduced to "push decisions out of model discretion into deterministic
  machinery" -- which is already the ELAI spine, so the work became choosing the
  sound mechanism per idea, not inventing a paradigm.

M6. EXISTS-BUT-COARSE IS NOT GREENFIELD. Distinguish "the substrate exists but is
  a coarse heuristic" from "nothing exists." The former is a precise upgrade with
  a named seam; the latter is a build. Name the exact file:line seam.
  WHY: a plan that says "build X" when X exists coarsely is both wrong and wasteful;
  the right plan says "replace heuristic H at file:line with measured signal S".
  EXAMPLE: the 4E sizer already splits on files*avg_tokens*0.6; the net-new was
  swapping that heuristic for a cone-token lower-bound vs the MEASURED ceiling.

M7. ATTACK YOUR OWN DESIGN. For any proposal, write the strongest case AGAINST it:
  the irreducible variance, where it has been tried and failed, the boundary where
  it breaks. Make the cheapest falsifying experiment explicit.
  WHY: an unattacked design ships its own failure mode. The attack surfaces the
  caveat that must be recorded and the experiment that must run first.
  EXAMPLE: the cost-estimation idea was attacked with the r<0.15 result (total
  cost is unpredictable) -> forced the redesign to "bound-and-escalate, never
  predict", and SWEnergy (4B models ~0% agentic) forced the honest caveat that
  sizing makes a local model FIT not CAPABLE.

M8. ROUTE GENUINELY-OPEN QUESTIONS TO GROUNDING; DO NOT GUESS. If resolving a
  fork needs a fact you cannot verify from code/training, route it: oracle (quick
  web), deep-research (a survey + attack-the-design), paper-ingestor, a cross-
  family second opinion. "I don't know, routing to grounding" beats a confident
  wrong answer baked into a plan.
  WHY: the two questions that actually decided the design (is cost-estimation
  gateable? is cheap grounding sound?) were training-uncertain; guessing would
  have produced a plausible, wrong plan.
  EXAMPLE: one anonymized deep-research prompt with a mandatory attack-the-design
  section settled both, and a sonnet oracle settled the Perplexity-architecture
  question (raw content never enters the reasoning model -> extractive compression).

M9. MAKE THE FALSIFIER THE OBSERVE-ONLY FIRST STEP. The cheapest experiment that
  could prove the idea wrong becomes step 1 of any plan; nothing goes default-ON
  until its false-positive / over-block rate is bounded on >=3 real runs (GD).
  WHY: gates that ship default-ON unvalidated become either noise or a fail-open
  guarantee nobody trusts.
  EXAMPLE: EXP-1 (cone-size vs token-spend correlation) and EXP-2 (small-NLI
  domain-transfer on ELAI code) are step 1 of J-39 and J-41; the gate's authority
  is set by the experiment, not assumed.

M10. RECORD HONEST RESIDUE. Name what is unverified, provisional, caveated, or
  deferred -- explicitly, not buried. A correct answer with a missing caveat is a
  wrong answer.
  WHY: the next session inherits your confidence level; hidden residue becomes a
  silent landmine.
  EXAMPLE: the SWEnergy capability caveat, the Co-Coder provisional cite, the
  binary-vs-3-class NLI adapter gap, and "J-34 is purely planned" were all written
  into the plans and the handoff, not smoothed over.

M11. DO NOT CONFLATE AXES (orthogonality discipline). When two mechanisms feel
  like alternatives, check whether they are on the same axis. Latency/cost is not
  capability; speed is not correctness; a cache is not state.
  WHY: conflating axes produces a false either/or and the wrong build.
  EXAMPLE: "would KV reuse be better than compression?" -- they are orthogonal
  (KV reuse = disposable latency/cost lever; compression = capability requirement);
  the answer is "compose them", not "pick one".

M12. WRITE IT DOWN SO IT SURVIVES THE SESSION. Capture the reasoning chain
  (workflow/reasoning/), the durable lens (docs/strategy/), and the cross-session
  handoff (workflow/handoff/{topic}.md). The main LLM authors these directly.
  WHY: context is lost on compaction/session-end; an un-recorded chain is re-
  derived from scratch (expensive and lossy).

================================================================================
APPLY TO ANY AXIS (the same method, same shape of answer, different subsystem)
================================================================================
For a NEW idea about subsystem X, run: M1 ground X in code+FEATURE-MAP -> M3/M4
  verdict + net-new-vs-existing -> M5 the principle -> M7 attack -> M8 route the
  open question -> M9 falsifier-as-step-1 -> M10 residue -> M12 write it down.
  Ask the stance question (what does the model decide in X, what can the harness
  decide instead) every time. Sketches:

- RESEARCH: model decides queries + page ranking; harness decides fetch +
  extraction + what is "enough". The mechanizable parts: deterministic source
  ranking, extractive (not generative) compression, a context-sufficiency gate
  the LLM does not adjudicate (TD-3), external per-step state. (This thread:
  J-40.)
- PLANNING: model proposes the decomposition; harness decides node SIZE/admission
  (bound-and-escalate, not predict), ordering (topological/DAG), and done (an
  evidence gate, not the model's say-so). Cross-cutting-but-atomic intents need an
  escape so the gate does not split pointlessly. (This thread: J-39.)
- REASONING / GRC: model generates the step; harness decides DEPTH budget (serial-
  depth not window-size), when to stop (must_converge / coverage), whether a step
  is grounded (off-agent check), and -- the open generalization the user raised --
  whether to DISTILL + RESET + re-reason fresh on the distillate for depth+economy.
  The mechanizable carry-state must be EXTRACTIVE+STRUCTURED, not free-text prose,
  and the training-free reasoning variant may REGRESS a small model -> falsify
  before trusting. (This thread: J-42 + the distill-reset propagation reasoning.)
- MEMORY: model proposes a write; harness decides validity (citation-before-use,
  staleness verdict), retrieval (hard-filter superseded rows, never narrate
  [OUTDATED]), and injection scope (auditor/critic get zero memory -- objectivity
  firewall). (See H-31/H-32/F-12.)
- SECURITY / ROUTING / EVAL: same shape -- the model never adjudicates the gate,
  the verdict is off-agent + frozen, missing evidence is FAIL, the objective is
  measured not proxied, an LLM in the loop is advisory + cross-family.

================================================================================
ANTI-PATTERNS (the method is violated when you...)
================================================================================
- assert how a subsystem works from training without grounding (M1).
- present a training-instinct claim as fact, unlabelled (M2).
- soften a fatal flaw to be agreeable (M3).
- propose a build for something that exists coarsely, or reinvent a shipped
  feature (M4/M6).
- design without an attack section, or ship a gate with no falsifier (M7/M9).
- guess at an external/empirical fact instead of routing it (M8).
- bury the caveat / oversell the win (M10) -- e.g. claiming local-model viability
  without the capability caveat.
- treat a latency/cost optimization as a capability substitute (M11).
- reason brilliantly and leave nothing written down (M12).
- pattern-match only "determinism" and forget the method generalizes to every axis.

================================================================================
THE ONE-LINE TEST
================================================================================
Before proposing any ELAI improvement on any axis, you should be able to answer:
  "What does the model decide here, what can the harness decide instead, what is
  the cheapest experiment that proves me wrong, and where did I write it down?"
If you cannot, you have not applied the method yet.

REASONING-ENGINE / GRC-v3 DESIGN REASONING
Author: main LLM (Opus 4.8). Date: 2026-06-25.
Mode: spec-eval AS-SPECIFIED (treat all mapped features as built to reason about the design).
Grounding policy: every claim tagged to a feature-map entry (FM:<subsystem>) or a paper (round 2+).
This file is written incrementally: ROUND 1 before papers, then one reasoning block per paper
summary, then a block on the deep-research report, then FINAL SYNTHESIS. Read-1 > reason > write,
never batch.

================================================================================
ROUND 1 - THE USER'S IDEA, RESTATED AND MAPPED TO WHAT ELAI ALREADY HAS
================================================================================

THE IDEA (distilled to its claims, so each can be grounded or refuted)
C1. ELAI splits testing / reasoning / execution across DIFFERENT models and DIFFERENT contexts
    (cloud generator vs local generator; auditor vs executor; per-role). Therefore the reasoning
    STATE must be carried ACROSS a model boundary and ACROSS a context unload (local model has no
    cross-request KV reuse).
C2. The carried reasoning should not spiral, should ground itself, and each subsequent reasoning
    step should stay SMALL -> we need lossless compression of the whole reasoning so far.
C3. A "candidate system": the part the model is MORE SURE about proceeds; the uncertain part stays
    open; this continues until validated by testing / some external check.
C4. Cross-domain problem: a codebase bug is investigable by read+think; but a trading-ML math bug,
    a Unity game bug, or something visual are different. How does one reasoning system help across
    all of them?
C5. Inference-time reasoning helps a little, a lot, or HURTS, depending on task and extra compute.
    Need to balance by task + cost. Maybe escalating reasoning. For research, a complexity
    classifier + a user effort toggle (light <-> complex).
C6. Context is the bottleneck. ELAI should be "the pipes": carry and manage context, be the engine;
    the models are just generators. Improve reasoning by unload/reload (local) and prompt-caching
    (cloud).
C7. Net ask: improve the GRC.

KEY FINDING (the headline, anti-sycophancy): the user's idea is NOT new to ELAI - it is, almost
line for line, the spine of ELAI's already-PLANNED reasoning stack. The design value is therefore
NOT invention; it is UNIFICATION + ACTIVATION of seven pieces that today are scattered across
different plan letters and are mostly PLANNED or DARK (default-off / no production caller). The
single highest-leverage move is to name one coherent "reasoning-state carrier" (call it GRC-v3) and
make these pieces compose, rather than to design a new mechanism. Mapping each claim:

C1 (carry state across model/context boundary):
  - FM:PABU TYPED BELIEFSTATE HANDOFF [PLANNED(I-11)] - "replaces raw history on cross-model/tier
    handoffs; DKG-keyed." This is EXACTLY C1. It is the carrier. Today: PLANNED only.
  - FM:MODEL-HANDOFF UX [DARK(E-3)] - compress_handoff_payload is test-only (no prod caller).
  - FM:CASCADING ESCALATION [WIRED(I-7)] - local->cloud waterfall, "condense-on-escalate via I-11
    (PABU leg still PLANNED)". So the escalation pipe is WIRED but the state-condensing leg it needs
    is the unbuilt I-11. That is the concrete seam.
  - FM:REGIME-AWARE CONTEXT POLICY [PARTIAL(H-50)] - cloud = big cache-stable prefix + late
    compaction; local = window-minimization + unload-early. This is C6's "unload/reload vs caching"
    already designed; it is PARTIAL/uncommitted.

C2 (small subsequent steps, lossless compression, no spiral):
  - FM:DETERMINISTIC CONTEXT COMPRESSION [PLANNED(I-14)] - LSP slicing + tree-sitter folding; every
    transform lands a reversible ContextTransformReceipt; ciphers BANNED (IC-3). This is C2's
    lossless-compression, with the house constraint that compression must be DETERMINISTIC and
    REVERSIBLE, never an LLM re-summarization (FM Section-4 ledger: "LLM re-summarization compaction
    on the cache prefix: REJECTED (busts cache); deterministic masking only, HD-2").
  - FM:SESSION DAG [WIRED] - 3-pass lossless trim, stubs recoverable from SSB seq. The substrate that
    makes "lossy view / lossless record" possible (RV doctrine: trim is a projection, never the
    record).
  - no-spiral: FM:ANTI-SPIRAL PRIMITIVES [PARTIAL] + the anti-spiral skill (4 stagnation patterns +
    5 personas + blake3 3-identical-hash stop). Already the spiral guard.

C3 (candidate system, sure-part proceeds, validated by testing):
  - FM:EVIDENCE-RANKED SEARCH LADDER [DARK(I-4)] - "select BY RECEIPTS not LLM-judge; best-of-N ->
    tournament -> PUCT over SessionDag." This is C3's candidate selection, with the crucial ELAI
    correction: candidates are ranked by RECEIPTS (test/checker output the model did not author),
    never by the model voting on itself (IC-6: search is a ladder not MCTS-first).
  - FM:ABSTENTION-AWARE SCORING [PARTIAL(H-39)] - correct=+1 / abstain=0 / confident-wrong=-t/(1-t).
    This is the formal version of "the sure part proceeds, the rest stays": a claim the model is sure
    of advances; an uncertain claim ABSTAINS (stays open) and is penalized only if it confidently
    asserts and is wrong. The carrier (I-11 beliefstate) should be a set of typed claims EACH with an
    abstention/confidence bit read from H-39 (not re-derived; RD-1).

C4 (cross-domain: code vs math vs game vs visual):
  - The carrier is uniform; the VALIDATOR is domain-routed. ELAI already has the router and the
    per-domain validators:
    - FM:MODE-ROUTING PREDICATE [WIRED(D-10)] - deterministic select_grounding_mode (modes A-E), LLM
      has no input. The dispatcher.
    - code/general: FM:GROUNDING VERIFICATION [WIRED] 6 methods FILE/SOURCE/TEST/WEB/API/REGISTRY +
      F-4 cant-lie test-integrity gate (auditor cannot fake passed=true).
    - math: FM:CONSTRUCTIVE-CHECKER REGISTRY [DARK(D-11)] - sympy/z3/compile/registry-check for
      Mode A. THIS is the trading-ML math validator C4 asks about. DARK today.
    - visual/game: FM:MEDIA VALIDATOR [WIRED] Tier-1 deterministic (ImageMagick/FFmpeg/Tesseract/
      Blender, SSIM/LPIPS) + Tier-2 VLM advisory; FM:PERCEPTION GATEWAY + /SEE [PLANNED(J-1)] (visual
      diffs go to pixel-diff NEVER a VLM, HD-5). A Unity bug routes to render-adapter + pixel-diff;
      a "visual" bug routes through perception.
    - trading: FM:TRADING TRUST GATES [PARTIAL, H-17..H-29] - Deflated-Sharpe/PBO, leakage taint,
      look-ahead perturb-and-diff, result-manifest re-derivation. The trading-ML validator stack.
  - So C4's answer is a DESIGN ALREADY LATENT in ELAI: one reasoning-state carrier + a deterministic
    mode router + a per-domain VALIDATION leg. The reasoning is domain-agnostic; the "candidate is
    confirmed" RECEIPT is domain-specific. This is the cleanest framing the round should sharpen.

C5 (reasoning helps little/lot/hurts; escalate; complexity classifier + effort toggle):
  - FM:SERIAL-DEPTH BUDGET [PARTIAL(H-42)] - "reasoning budget scales with estimated serial depth,
    never window size; depth-not-width guard." Doctrine RD-4: "32K+ context as the fix for depth-class
    reasoning failures: REJECTED." This is the GROUNDED answer to C5's "more reasoning can hurt": the
    house position is already that you budget by problem DEPTH, not by throwing more context/tokens.
  - FM:DYNAMIC THINKING BUDGETS [DARK(I-26)] - serial-depth scaled, effort field threaded to
    anthropic_api, never budget_tokens on Opus 4.7+. This is C5's escalating-reasoning + effort knob.
  - FM:EMPIRICAL CONTEXT CALIBRATION [PLANNED(J-34)] - measures per-(model,lane) degradation curves
    on padded fixtures, gate-verdicts never LLM-judge. The pattern to copy: MEASURE the
    reasoning-budget-vs-verified-outcome curve per task-type, don't trust folklore. The "complexity
    classifier" should feed a budget whose escalation steps are receipt-justified (J-34 pattern),
    not vibes.
  - research effort toggle: FM:ADAPTIVE CHUNKING [WIRED] FAST/STANDARD/DEEP tier routing + the
    deep-research skill (breadth/depth args). The surface exists; the missing piece is a deterministic
    complexity classifier that PICKS the tier and a one-tap user override.

C6 (context is the bottleneck; ELAI = pipes/engine, models = generators):
  - This is ELAI's whole thesis. FM:CONTEXT BUDGET ALLOCATOR [WIRED] 8 fixed slots + 2 cache
    breakpoints; FM:REGIME-AWARE CONTEXT POLICY (H-50); the project memory note
    [[project_efficiency_trust_engineering]] = "smarter-orchestration-not-model." Grounded and on-thesis.
  - The one honest gap the map flags: local caching is a NO-OP today - "PrefixKVCache (server.py:1819)
    BYPASSED on the streaming path the live client uses (runtime_client.rs:547)". So C6's "local
    unload/reload" is the real lever (H-50), because local has NO KV reuse to lean on; cloud leans on
    prompt caching (WIRED) instead. The two lanes genuinely need OPPOSITE context policies (CL-1..CL-4).

WHERE THE USER IS RIGHT, AND WHERE THE DESIGN NEEDS A CORRECTION
  RIGHT: carry a typed state not raw transcript (I-11); keep each step small via deterministic
  reversible compression (I-14); candidates advance on confidence and are confirmed by external
  receipts (I-4 + H-39); escalate by depth/effort not width (H-42 + I-26); the engine owns context,
  the model generates (the thesis).
  CORRECTION 1 (grounded in IC-6 + the Section-4 ledger): the "candidate system" must NOT be
  self-consistency-style majority voting or MCTS-first. ELAI's rule is a LADDER (best-of-N ->
  tournament -> PUCT) where the SELECTOR is receipts, not the model judging its own paths. A naive
  "sample many reasonings and pick the confident one" reintroduces the self-grading failure the whole
  trust spine exists to prevent (D-13 cross-family, F-4 cant-lie, wiring R4 measure-the-objective).
  CORRECTION 2 (grounded in HD-2 / IC-3): "lossless compression of the entire reasoning" must be
  DETERMINISTIC and REVERSIBLE (LSP/tree-sitter folding + receipts, I-14), never an LLM
  re-summarization - the latter busts the cloud cache prefix and is unverifiable. So "compress the
  reasoning" on the cloud lane means deterministic masking of trailing observations behind the cache
  breakpoint (H-6/HD-2); on the local lane it means unload-early + selective reload (H-50).
  CORRECTION 3 (the cross-domain claim, C4): there is no single "reasoning" that helps a Unity bug
  and a Sharpe-ratio bug the same way. The carrier is uniform but the win comes from routing to the
  right deterministic VALIDATOR (D-10 mode router -> D-11 math checker / J-1 perception / H-trading
  gates). The user's instinct ("how does it help the visual case") is answered by "the reasoning is
  the same; the receipt that closes a candidate is what changes."

THE GAP, STATED PRECISELY (what GRC-v3 would actually be)
  Today: GRC is a WIRED per-topic budget+grounding state machine (phase-2A..2D) that lives INSIDE one
  model's context and does not survive a model/context boundary. The seven pieces that would let
  reasoning survive a boundary, stay small, advance on confidence, and validate per-domain exist but
  are scattered (I-11 PLANNED, I-14 PLANNED, I-4 DARK, H-39 PARTIAL, H-42 PARTIAL, I-26 DARK,
  H-50 PARTIAL).
  GRC-v3 = a typed BeliefState (I-11) as the unit of reasoning, where each entry is a claim + its
  grounding-receipt + an abstention/confidence bit (H-39); the BeliefState is the thing that is
  deterministically compressed (I-14), carried across escalation (I-7) and across local unload (H-50),
  branched as candidates on the SessionDag and selected by receipts (I-4), with the
  reasoning-budget set by serial depth + a complexity classifier + an effort toggle (H-42/I-26), and
  each candidate CLOSED by a domain-routed validator (D-10 -> D-11 / J-1 / H-trading / F-4). One
  carrier, one router, many validators.

OPEN QUESTIONS THE PAPERS MUST ANSWER (so the round-2 reads are targeted, not passive)
  Q1. Are the 2022-era prompting results (CoT 2201.11903, zero-shot CoT 2205.11916, self-consistency
      2203.11171, least-to-most 2205.10625) still load-bearing in 2026 on instruction-tuned models,
      or are they stale "the base model needed the trick" artifacts? (User flagged this explicitly.)
      Specifically: does CoT still help a modern small coder model (Qwen2.5-Coder-7B local lane), or
      is it baked in? This decides whether GRC-v3 should still PROMPT for reasoning or assume it.
  Q2. Self-consistency (2203.11171) vs ELAI's receipts-not-votes rule (IC-6): is majority-vote over
      sampled reasonings ever worth it when a deterministic checker exists? Hypothesis: only when NO
      receipt is available (pure NL judgement); otherwise receipts dominate. Confirm/refute.
  Q3. Tree-of-Thoughts (2305.10601) - what exactly is the search policy, and does it justify anything
      beyond ELAI's existing best-of-N -> tournament -> PUCT ladder (I-4)? Or is ToT just the ladder
      with an LLM evaluator (which ELAI bans as the selector)?
  Q4. Test-time compute scaling (2408.03314) - THE balance paper for C5. What is the compute-optimal
      escalation rule (when does more reasoning help vs hurt, as a function of problem difficulty)?
      This should directly parameterize the H-42 serial-depth budget + the complexity classifier.
  Q5. Reflexion (2303.11366) and ReAct (2210.03629) - do they add anything to ELAI's already-WIRED
      GRC observe/act/verify loop + Reflexion buffer (E-46) + insight backprop (J-10)? Or
      confirmation only?
  Q6. 2604.10739 (2026-04, recent, unknown) - what is it, and is it the modern statement that
      supersedes the 2022 papers? Agent must identify and ground it.

PROVISIONAL POSTURE (to be revised after each paper)
  Lean: GRC-v3 is an ACTIVATION+UNIFICATION plan, not a new-mechanism plan. Its keystone is I-11
  (the beliefstate carrier) because it is the one piece every other piece needs and it is unbuilt.
  Its done-gate must be ELAI-MEASURED (a reasoning-budget-vs-verified-outcome curve, J-34 pattern),
  never a paper constant (ID-5). The cross-domain answer (C4) is the framing win to make explicit:
  one carrier, domain-routed validators. To be confirmed or corrected by the papers + the deep-
  research report.

================================================================================
ROUND 2 - PER-PAPER (read one, reason, write; never batch)
================================================================================

PAPER 2201.11903 - Chain-of-Thought Prompting (Wei et al, Google 2022). Verdict: PARTIALLY-STALE.
  What the interface map establishes: CoT = few-shot exemplars with NL reasoning before the answer;
  greedy decode; tested on PaLM 540B / LaMDA 137B / GPT-3 / Codex (2022 models). Two findings that
  survive 2026 and bear directly on GRC-v3, and one that is now obsolete.
  OBSOLETE: the few-shot-exemplar elicitation. Modern instruction-tuned models (incl. the local
  Qwen2.5-Coder-7B lane) already emit CoT-like traces without hand-written exemplars; the "show
  reasoning before answer" trick is baked in (interface map staleness + arXiv:2506.07712 2025).
  IMPLICATION 1 (sharpens C6/the thesis): GRC-v3 should NOT spend context re-teaching CoT via
  exemplars - that is the absorbed part. The engine's value-add is NOT eliciting reasoning, it is
  GROUNDING it. So the budget that would have gone to few-shot exemplars (300-600 tokens, interface
  map) is freed for receipts. This is concrete evidence that "ELAI is the pipes, the model
  generates" is the right division of labor: the generator already reasons; the engine validates.
  IMPLICATION 2 (the strongest grounding yet for CORRECTION 1): the paper's failure analysis - a
  CoT trace can be a post-hoc rationalization, "a green CoT trace does NOT mean correct reasoning"
  (interface map pitfalls; 46% near-miss arithmetic, 54% semantic). A self-asserted-confident chain
  is NOT trustworthy. Therefore the candidate/confidence bit in GRC-v3 (C3) MUST come from an
  EXTERNAL receipt (test/checker/perception), never from the chain grading itself. This is exactly
  F-4 cant-lie + abstention-from-receipts (H-39) + IC-6 (receipts-not-votes). The user's "the part
  the model is more sure about proceeds" needs the explicit correction: "sure" = receipt-confirmed,
  not chain-self-confident.
  IMPLICATION 3 (grounds the cheap end of the escalation ladder, C5): CoT measurably HURTS on
  trivial tasks and on sub-threshold models ("do NOT use CoT on tasks trivially solved by standard
  prompting"). So the complexity classifier (C5) must be able to route a simple input to a
  ZERO-reasoning budget - skipping the chain is a first-class branch of the H-42 serial-depth budget,
  not an afterthought. The ladder's bottom rung is "no chain at all."
  CROSS-BOUNDARY (C1): CoT proves reasoning CAN be externalized as portable text. But raw text is
  precisely what I-11 PABU is designed to REPLACE with a typed beliefstate - smaller and loss-
  controlled. So CoT validates the premise (externalizable) while ELAI improves the representation
  (typed claims+receipts, not raw chain). No change to the I-11 keystone; reinforced.
  NET: confirmation + two sharpenings (sure=receipt-confirmed; zero-reasoning is a real ladder rung).
  No new mechanism. Posture unchanged.

PAPER 2205.11916 - Zero-Shot-CoT "Let's think step by step" (Kojima et al, 2022). Verdict: PARTIALLY-STALE.
  New signal over CoT: (a) a clean TWO-STAGE structure (stage1 generate chain Z, stage2 extract
  answer with a task-specific suffix); (b) a hard DOMAIN result - CommonsenseQA DEGRADES -4.2pp,
  "CoT introduces confabulation on factual recall tasks"; (c) staleness note pins the local lane:
  for >7B instruction-tuned (Qwen2.5-14B/72B) zero-shot-CoT matches few-shot-CoT, but for <7B "the
  benefit is reduced because instruction-following capacity is the bottleneck."
  IMPLICATION 4 (local-lane specific, ties to J-55): ELAI's local generator is Qwen2.5-Coder-7B -
  right at the threshold where free-form CoT eliciting is least reliable (instruction-following is
  the limiter). This independently grounds J-55 MODALITY ROUTER's keystone guard ("the Qwen-7B lane
  is NEVER routed to CodeAct") and the XGrammar-constrained local path: on the local lane GRC-v3 must
  NOT lean on a free-text trigger to elicit reasoning; it must IMPOSE structure (decomposed sub-steps
  the engine supplies, constrained emission). The reasoning scaffold is the ENGINE's job on the local
  lane precisely because the small generator won't reliably self-scaffold. This is the strongest
  paper-grounded argument so far that "ELAI = pipes, model = generator" is not just philosophy but a
  hard requirement at <=7B.
  IMPLICATION 5 (sharpens the classifier, C5): CommonsenseQA/factual-recall degradation is a SECOND
  domain data-point (CoT helps math/symbolic, hurts commonsense/recall). So the complexity decision
  is two-dimensional, not one: it is not only light<->heavy effort, it is also TASK-TYPE routing -
  reason vs RETRIEVE vs validate. A factual-recall query should route to the DKG/A-RAG retrieval lane,
  NOT to a reasoning chain that will confabulate. ELAI already has this dispatcher: D-10 MODE-ROUTING
  PREDICATE (modes A-E, deterministic). The refinement to GRC-v3: the "effort toggle" the user wants
  is the cost axis; the mode router is the type axis; both must feed the budget. Routing a recall
  query to "more reasoning" is exactly the "reasoning hurts" case the user flagged.
  IMPLICATION 6 (where the receipt-gate sits): the stage1/stage2 boundary is the natural place to
  put the abstention/confidence check - compute confidence on Z BEFORE spending the second
  (expensive, or cross-model) call. In GRC-v3 terms: the beliefstate (I-11) is checked against its
  receipts at each stage boundary; a low-confidence beliefstate does not get promoted across the
  model/escalation boundary (I-7) - it either gets more local depth or abstains. Clean seam.
  NET: confirmation + the local-lane structural argument (IMPLICATION 4, materially strengthens the
  thesis) + the task-TYPE axis added to the classifier (IMPLICATION 5). Posture unchanged; the
  classifier is now explicitly 2-axis (cost x type).

PAPER 2203.11171 - Self-Consistency (Wang et al, ICLR 2023). Verdict: STILL-VALUABLE. This is the
  single most decision-relevant paper for the user's "candidate system" (C3), and it CONFIRMS ELAI's
  receipts-not-votes doctrine with a 2026 citation.
  The mechanism: sample m diverse CoT paths, majority-vote the final answers. Crucially the selector
  is STATISTICAL AGGREGATION, not an LLM judging its own paths - structurally more reliable than
  ToT's self-evaluation (next paper). But the load-bearing 2026 finding (arXiv:2603.06612 "Consensus
  is Not Verification", cited in the interface map): majority vote FAILS for factual/truthfulness
  claims because all m paths share the same systematic training-baked bias; it only works when errors
  are INDEPENDENTLY RANDOM (math/formal reasoning). And "external verifiers (PRMs, code execution,
  proof checkers) consistently outperform majority vote when available - a wrong answer in 30 of 40
  samples still fails the checker."
  IMPLICATION 7 (the decisive grounding for CORRECTION 1 + I-4): the user's "the part the model is
  more sure about proceeds" must be implemented as the I-4 ladder's rule - select by RECEIPT when a
  receipt exists, because a deterministic checker is STRICTLY superior to any amount of model
  self-agreement. ELAI's I-4 ("select BY RECEIPTS not LLM-judge") and IC-6 are now confirmed by a
  June-2026 paper, not just house doctrine. The candidate system is: branch candidate beliefstates on
  the SessionDag -> close each with its domain receipt (D-11/F-4/J-1/H-trading) -> the receipt-passing
  candidate advances. Votes are NOT the selector.
  IMPLICATION 8 (the receipt-ABSENT fallback rung - a genuine ADDITION to I-4): when NO deterministic
  receipt exists (open-ended NL, design judgement, no checkable answer), the correct fallback is
  STATISTICAL consensus over independent samples - NOT an LLM grading its own output. So I-4's ladder
  needs an explicit bottom rung for the receipt-absent case: vote-based aggregation, never
  self-evaluation. This is small but real: it stops the system from silently falling back to the
  banned self-judge pattern when receipts run out. Caveat from the same paper: consensus is unsafe
  for factual/recall (systematic bias) - so the receipt-absent fallback is only valid for
  independent-error tasks; recall tasks fall back to RETRIEVAL (IMPLICATION 5), not to a vote.
  IMPLICATION 9 (budget lever, ties H-39/I-26/I-27): CGES (arXiv:2511.02603) - confidence-guided
  early stopping cuts samples 69.4% at equal accuracy; gains saturate by ~20 samples. So candidate
  generation must be EARLY-STOPPABLE on receipt-convergence, never fixed-N. Maps onto H-39 abstention
  + I-26 dynamic budget + I-27 budgeted-convergence instrumentation: keep branching candidates until
  a receipt confirms one OR confidence converges OR the depth budget caps. Fixed best-of-40 is waste.
  CLARIFICATION (separates two axes the user partly conflated): self-consistency does NOT carry state
  across a boundary - each path is from scratch; it is a fan-out/gather at OUTPUT SELECTION (the I-4
  SessionDag branch/merge layer). I-11 (carry ONE beliefstate across a model/context boundary) is a
  DIFFERENT axis. GRC-v3 needs both: I-11 = the carrier across boundaries; I-4 = candidate branch +
  receipt-select within a boundary. The user's single "candidate system that carries and validates"
  is correctly two mechanisms that compose.
  NET: strong confirmation of the trust-spine doctrine by a 2026 paper + two concrete additions
  (receipt-absent vote-fallback rung; early-stop candidate budget). Posture strengthened: I-4 and
  I-11 are distinct composing keystones, not one thing.

PAPER 2205.10625 - Least-to-Most (Zhou et al, ICLR 2023). Verdict: PARTIALLY-STALE. This paper gives
  GRC-v3 its sequential-accumulation shape AND names the exact flaw the trust spine fixes.
  Mechanism: decompose into ordered subproblems, solve sequentially, feed each prior answer forward.
  The accumulated [Q1:A1]...[Qi-1:Ai-1] prefix IS the reasoning state (interface map: "each Ai is a
  state snapshot; the engine must serialize the prefix and restore it"). The fatal flaw the paper is
  explicit about: "a wrong A1 poisons A2..An with NO recovery mechanism" - errors cascade silently.
  IMPLICATION 10 (the cleanest synthesis so far - GRC-v3 in one line): GRC-v3 = least-to-most
  decomposition WITH a per-step receipt gate. In plain L2M a wrong A1 poisons the chain; in GRC-v3
  each Ai is a candidate beliefstate entry whose domain receipt (D-11/F-4/J-1) must PASS before it is
  committed to the carried prefix (I-11). A failing step abstains/retries BEFORE it can poison the
  next step. This is EXACTLY the user's "the part the model is more sure about proceeds, the rest
  stays" - made concrete at the decomposition-step grain: a step "proceeds" iff its receipt passes;
  otherwise it "stays" (abstains, gets more depth, or escalates). The user's intuition and L2M's
  structure and ELAI's trust spine converge here.
  IMPLICATION 11 (ELAI already does this at PLAN grain - GRC-v3 moves it to REASONING grain): ELAI's
  HIERARCHICAL PLAN MGMT (phase-3G) + recursive Idea->Concepts->Phases->Steps->BabySteps (Pipeline
  Factory L1) + per-level auditors (B-41) ARE least-to-most-with-gates at the PLAN level. The reasoning
  engine is the SAME discipline applied INSIDE a single task at the reasoning-step grain. This means
  GRC-v3 is not a foreign mechanism - it is ELAI's own proven decompose-then-gate pattern, recursed
  one level down. Strong reuse argument; lowers build risk.
  IMPLICATION 12 (depth-vs-width made actionable, ties H-42): L2M warns "do not apply to problems
  where subproblems are INDEPENDENT - use parallel calls instead." So the classifier must distinguish
  SERIAL-DEPENDENT chains (serialize, charge serial-depth budget H-42, BeliefState accumulates) from
  INDEPENDENT subproblems (parallelize via BSP spawn_parallel, charge width not depth, candidates
  merge). This operationalizes the depth-not-width doctrine (RD-4): depth budget is for genuine
  dependency; independence is cheaper width and must not be billed as depth. The user's "escalating
  reasoning" is really "add serial DEPTH only where steps depend on each other."
  STALENESS consistent with prior: decomposition is native on capable cloud models (Claude-3.5 L2M+PAL
  ~= plain PAL), so impose explicit decomposition on the LOCAL Qwen-7B lane, let the CLOUD lane
  self-decompose (IMPLICATION 4 again).
  NET: gives GRC-v3 its one-line definition (decompose + per-step receipt gate = sure-proceeds/rest-
  stays) and a strong reuse argument (it is phase-3G recursed to reasoning grain). + the
  serial-vs-parallel split for the budget. Posture: the design is now concretely specified, not just
  a list of pieces.

PAPER 2305.10601 - Tree of Thoughts (Yao et al, NeurIPS 2023). Verdict: PARTIALLY-STALE. Mostly
  confirmation, but it isolates the ONE high-value primitive (backtracking) and sets the gating
  discipline for the top of the escalation ladder.
  Mechanism: branch k thoughts per step, evaluate, BFS/DFS with pruning + backtracking. State
  s=[x,z1..zi] externalized as a tree = candidate buffer. Two facts dominate: (1) ablation -
  BACKTRACKING is the single biggest contributor (60% full vs 20% without); (2) the SELECTOR is LLM
  self-evaluation, explicitly "the weakest link... 8B models lack capacity for effective branch
  evaluation... PRMs/external checkers strictly superior."
  IMPLICATION 13 (third independent confirmation of CORRECTION 1, now with a hard small-model rider):
  ToT's self-evaluation selector is invalid on ELAI's local Qwen-7B lane (8B-class models can't
  evaluate their own branches) AND is dominated in general by receipts. The selector hierarchy is now
  triply grounded: deterministic RECEIPT (best) > statistical VOTE over independent samples (receipt-
  absent fallback) > LLM self-value-score (BANNED in ELAI, and the stale part of ToT). GRC-v3's ladder
  uses the first two and never the third.
  IMPLICATION 14 (the reusable primitive is backtracking, which ELAI already has): the value of tree
  search is mostly the ability to BACKTRACK to the last good node and try another branch. ELAI ships
  this substrate: SESSION DAG branch/merge [WIRED] + H-49 REVERSIBLE EDITS/undo-tree + RV-1 divergence
  guard [PARTIAL] + /rewind. So GRC-v3 does not need a new tree-search engine; it needs the I-4 ladder
  to drive backtracking over the SessionDag: when a candidate step's receipt FAILS, backtrack to the
  last receipt-passing beliefstate node and branch a different continuation. Tree-of-thoughts becomes
  "receipt-gated branch-and-backtrack over the SessionDag," reusing WIRED/PARTIAL substrate.
  IMPLICATION 15 (gating discipline - tree search is a HIGH rung, rarely reached): ToT costs 5-100x a
  single CoT and only pays off on combinatorial/search tasks with correctable early choices (Game of
  24, crosswords); it WASTES compute when first-pass CoT is near-optimal. So full branch-and-backtrack
  sits at the TOP of the escalation ladder (I-4 v3 PUCT), reached only when the complexity classifier
  detects a branching solution space AND lower rungs failed their receipts. The ladder's mass is at
  the bottom (no-chain / single-chain); escalation to candidate-branching is the exception; full tree
  search is rare. This is the concrete shape of the user's "escalating reasoning" + answers C5's
  "reasoning can hurt": the cost of the high rungs is the hurt, so they are receipt-gated and
  classifier-gated, never default.
  NET: confirmation x3 of receipts-not-self-eval (now with the 8B-can't-self-evaluate rider) + the
  backtracking primitive maps to shipped SessionDag/H-49 + the escalation ladder's shape is pinned
  (mass at the bottom, tree search rare and gated). No new engine needed. Posture unchanged.

PAPER 2210.03629 - ReAct (Yao et al, ICLR 2023). Verdict: PARTIALLY-STALE. It IS ELAI's GRC loop, and
  its 2024 refutation paper gives the most important corrective to the user's premise.
  Mechanism: interleave Thought / Action / Observation; the Observation is external and grounds the
  reasoning (ReAct hallucinates 6% vs CoT 14% on HotpotQA, though it scores slightly BELOW CoT on raw
  EM - it trades accuracy for grounding). This is exactly ELAI's WIRED GRC state machine (phase-2A:
  Observe/Hypothesize/PlanAction/Act/Verify) with the 6 grounding methods (D-10) as the Observation.
  So ELAI's reasoning core already IS ReAct-with-grounding; confirmation of the WIRED substrate.
  IMPLICATION 16 (the sharpest corrective to "improve reasoning", grounded in arXiv:2405.13966
  "Brittle Foundations of ReAct"): performance is "minimally influenced by the interleaved reasoning
  trace itself; what drives performance is exemplar-query similarity. The Thought token is useful for
  grounding but not for genuine multi-step causal reasoning." This says the lever is NOT more
  reasoning text - it is better GROUNDING (observations) and better RETRIEVAL of similar prior solved
  cases. For GRC-v3 this redirects the compute: spend it on retrieval/grounding (DKG, CATRAG/A-RAG,
  CASE-BASED CALIBRATION C-45/C-47, TEMPLATE CRYSTALLIZER C-25, INSIGHT BACKPROP J-10), not on longer
  chains. This is the third independent confirmation of the C6 thesis (the engine that manages
  context/evidence beats the model that reasons longer) and it directly tempers the user's framing:
  "improve reasoning" should largely mean "improve the grounding/retrieval the reasoning stands on."
  IMPLICATION 17 (the receipt-gate seam, now precisely located): confidence lives in the Thought; the
  engine intercepts AFTER Thought, BEFORE Action, scores confidence, and proceeds / gathers more /
  escalates. This is exactly the GRC Verify step between PlanAction and Act, where ELAI already runs
  CSQ + inline feasibility (phase-2D, max-2 inline checks) + would read the abstention bit (H-39). So
  the per-step receipt gate (IMPLICATION 10) has a named home in the existing GRC loop - no new seam.
  IMPLICATION 18 (concrete compression policy for I-14/I-11 - genuinely useful): ReAct's interface map
  notes the Thought field is the MOST compressible part (it is reasoning ABOUT the observation, not the
  observation). So the carried beliefstate should keep OBSERVATIONS/RECEIPTS at high fidelity
  (lossless) and treat THOUGHTS as lossy-compressible/rederivable. This gives I-14 a non-arbitrary
  compression rule: preserve evidence, fold prose. It matches the RV doctrine (lossless record / lossy
  view) and the DKG-keyed I-11 design (the beliefstate keys to evidence rows, not to prose). The
  user's "lossless compression of the entire reasoning" is refined: losslessly preserve the RECEIPTS;
  the reasoning prose around them is a lossy projection rederivable from the receipts.
  IMPLICATION 19 (anti-spiral is mandatory, already shipped): ReAct's repetitive-loop failure (same
  thought-action 2+ steps = stuck) is ELAI's ANTI-SPIRAL primitive (phase-3C blake3 3-identical-hash
  stop + the anti-spiral skill's 4 patterns/5 personas). Confirms the no-spiral requirement (C2) maps
  to shipped substrate; GRC-v3 wires anti-spiral as the escalation trigger (stuck -> escalate, never
  loop).
  NET: ReAct = the WIRED GRC loop (confirmation) + the decisive corrective (IMPLICATION 16: spend
  compute on grounding/retrieval, not chain length) + a named home for the receipt gate (the Verify
  step) + a principled compression rule for I-14 (preserve receipts, fold prose). The user's "improve
  reasoning" is now substantially reframed as "improve grounding + carry receipts losslessly."

PAPER 2303.11366 - Reflexion (Shinn et al, NeurIPS 2023). Verdict: PARTIALLY-STALE. Carries the single
  most important empirical fact for the WHOLE ELAI architecture: the self-correction blind spot.
  Mechanism: Actor / Evaluator / Self-Reflection; convert a failure signal into a 1-3 sentence verbal
  critique stored in an episodic buffer, prepend next episode. The headline 91% HumanEval used GPT-4
  as BOTH actor and reflector = "oracle smuggling."
  IMPLICATION 20 (THE empirical WHY behind ELAI's entire trust spine - the self-correction blind spot,
  Tsui 2025 via the interface map): "LLMs fail to correct their own errors while successfully
  correcting identical errors presented as EXTERNAL input." This is the deepest grounding in the whole
  round. It is WHY the auditor must be a separate context/model (D-13 cross-family), WHY receipts must
  be external (F-4), AND it hands GRC-v3 a positive mechanism: to make a model fix its mistake, surface
  the failure as EXTERNAL evidence (the checker's counterexample, the failing test's stdout, the
  pixel-diff), NOT as "you were wrong, reflect on it." So in GRC-v3 the "reflection" step is the
  RECEIPT re-presented as external input, never an LLM-authored self-critique. This corrects a subtle
  version of the user's idea: "the model grounds itself" is false in general (blind spot); "the engine
  grounds the model by feeding it external receipts" is what actually works. This is the C6 thesis at
  its strongest - the engine is what closes the loop the model structurally cannot close alone.
  IMPLICATION 21 (ELAI's J-10 is "Reflexion done right" - candidate GRC-v3 component): the episodic
  reflection buffer = ELAI's INSIGHT BACKPROPAGATION TREE MEMORY (J-10): "extract 1-3 sentence insight
  per AUDIT REPORT, inject lineage, provenance hash, LLM writes text never verdict." J-10 is exactly
  Reflexion's buffer but sourced from EXTERNAL audit verdicts (not self-critique) with provenance - it
  is the blind-spot-safe version. Confirms J-10 belongs in GRC-v3 as the cross-attempt memory, and
  that its "insight from audit report" sourcing is the correct (external) design. Also maps to the
  Reflexion buffer already noted in PLANNING DECOMPOSITION EXTRAS (E-46) and CROSS-SESSION SLEEP-TIME
  HANDOFF (H-51) for the cross-session persistence of these reflections.
  IMPLICATION 22 (local-lane reflector is noise - escalate the critique): "on small instruction-tuned
  coder models (<7B) the reflector quality degrades faster than the actor, making the reflection noise
  rather than signal." So on the local Qwen-7B lane the critique must NOT be self-generated; it must be
  the external receipt or a cloud/cross-family auditor (D-13 + the I-7 EscalationCascade). The engine
  supplies the reflection; the local model does not author it. Fourth reinforcement of IMPLICATION 4.
  IMPLICATION 23 (the ladder's "stop reasoning, change strategy" rung - grounded): Reflexion's 3-trial
  budget is a hard upper bound - "beyond that the failure mode shifts from correctable errors to
  fundamental knowledge gaps that reflection cannot address." So after ~3 receipt-gated retries the
  right move is NOT more reasoning; it is a MODALITY CHANGE: retrieve more grounding, escalate to a
  stronger model (I-7), or ABSTAIN (ALETHEIA VOLUNTARY ABORT I-8, credited by abstention score). This
  is the anti-spiral 3-hypothesis rule (token-discipline) + I-8 made into an explicit ladder rung:
  N failed reflections => change modality, never loop. Directly answers C5's "when does more reasoning
  hurt" at the top of the ladder: past 3 failed receipted attempts, more reasoning is pure waste.
  NET: the self-correction blind spot (IMPLICATION 20) is the empirical keystone of the whole design -
  it converts the user's "model grounds itself" into the correct "engine grounds the model with
  external receipts." + J-10 confirmed as the blind-spot-safe reflection memory + the local-lane
  external-critique rule + the 3-retry-then-change-modality ladder rung. Strongest paper of the round
  for justifying ELAI's architecture; posture strengthened, not changed.

PAPER 2408.03314 - Scaling LLM Test-Time Compute Optimally (Snell et al, 2024). Verdict: STILL-VALUABLE
  (2026 cites). THE paper for C5 - it gives the quantitative escalation policy the user asked for.
  Two orthogonal compute mechanisms: SEARCH over candidates (PRM-scored best-of-N / beam) vs
  sequential REVISION (refine conditioned on prior attempts). Neither dominates; the COMPUTE-OPTIMAL
  policy ROUTES BY DIFFICULTY (pass@1 quintile): easy/medium -> sequential REVISION (depth); hard ->
  best-of-N (parallel WIDTH); very hard (pass@1<0.05) -> "little meaningful progress regardless of
  strategy." Compute-optimal = 4x efficiency over flat best-of-N. 2026 confirms (2604.01411 T2 scaling
  survives post-training; 2504.04718 Llama-1B+TTS > Llama-8B on MATH).
  IMPLICATION 24 (the escalation ladder, now quantified - direct C5 answer): GRC-v3's budget routes by
  ESTIMATED DIFFICULTY:
    - trivial (high pass@1 proxy): ZERO reasoning (IMPLICATION 3) - answer directly / retrieve.
    - easy/medium: add serial DEPTH (sequential revision = L2M-with-receipts = carry+revise the I-11
      beliefstate); this is the common case and matches H-42 depth-not-width.
    - hard: switch to parallel WIDTH (I-4 best-of-N candidate branches / BSP spawn_parallel), receipt-
      selected; depth stops paying, parallel coverage is the least-bad lever.
    - very hard (pass@1<~0.05): STOP burning compute - mark NEEDS_EXTERNAL_VALIDATION, escalate to a
      stronger model (I-7) or ABSTAIN (I-8). This is the quantitative form of IMPLICATION 23.
  IMPLICATION 25 (honest refinement of the depth-not-width doctrine RD-4/H-42): the paper says HARD
  problems want WIDTH (more parallel samples), not more revision-depth - which naively tensions with
  "depth-not-width." Reconciliation (both are true on their own axis): RD-4's real claim is that more
  CONTEXT-WINDOW never fixes a depth-class failure (32K is not the fix) - the paper agrees, window size
  is never the lever. The depth-vs-parallel-width choice is a DIFFERENT axis and it is difficulty-
  routed: easy/medium -> depth (revision), hard -> width (parallel candidates). So GRC-v3's budget is:
  never scale the window; scale DEPTH for easy/medium and PARALLEL WIDTH for hard. This is a genuine
  sharpening of H-42, surfaced honestly rather than papered over.
  IMPLICATION 26 (the complexity classifier - proxy difficulty, measure ELAI's own curve): the paper's
  caveat is that true difficulty-binning needs an expensive pass@K oracle; practical systems "proxy
  difficulty via answer confidence or token budget." So ELAI's classifier (the user's "complexity
  classifier + effort toggle") proxies difficulty from INTERNAL signals it already has: the H-39
  abstention/confidence score, the CASE-BASED hit rate (C-45/C-47 - have we solved a similar task),
  retrieval coverage (J-38 context-sufficiency gate), and the C-29 IntentPair count. And per J-34/ID-5,
  ELAI MEASURES its own per-(model,task-type) difficulty->strategy curve on an ELAI gold set rather
  than importing the paper's quintile bins. Done-gate = the measured routing curve, never the paper's
  numbers. The user's "effort toggle" is the manual override on top of this auto-classifier.
  IMPLICATION 27 (PRM is a Tier-2 step-scorer, not the Tier-1 selector - reconciles paper with IC-6):
  the paper's selector is a PRM (learned step scorer). ELAI's doctrine is receipts > PRM > vote >
  self-eval. Reconciliation: where a DETERMINISTIC receipt exists (test/compile/checker/pixel-diff),
  it is Tier-1 and the PRM is not on the critical path; the PRM is the Tier-2 scorer for INTERMEDIATE
  reasoning steps that have no deterministic receipt yet (a half-finished math derivation before the
  final checkable answer). ELAI already ships this PRM substrate dark: PPM PROCESS REWARD SIDECAR
  [DARK(D-2)] (Qwen2.5-Math-PRM-7B step scoring) + ISOMORPHIC CANARY (I-24 PRM dual-score, anti-gaming)
  + PROGRESS-SHAPED REWARDS [DARK(I-22)]. So the per-node confidence the paper gets from a PRM, ELAI
  gets from D-2/I-22 as a Tier-2 signal UNDER the Tier-1 receipt, with I-24's canary guarding the PRM
  against style-gaming. No new scorer needed; activate the dark D-2/I-22 under the receipt-first rule.
  NET: the quantitative escalation policy (IMPLICATION 24) is the backbone of the C5 answer + an honest
  sharpening of depth-not-width (IMPLICATION 25: window never, depth for easy/medium, width for hard) +
  the difficulty classifier built from ELAI's own internal signals and measured not imported
  (IMPLICATION 26) + PRM placed correctly as Tier-2-under-receipt reusing dark D-2/I-22/I-24
  (IMPLICATION 27). Posture: C5 is now fully specified and grounded.

PAPER 2604.10739 - "When More Thinking Hurts" / Overthinking (Zhou et al, April 2026). Verdict:
  STILL-VALUABLE (it is current, tested on DeepSeek-R1-32B + s1-32B). This is the empirical proof of
  the user's C5 ("reasoning can HURT") and it hands GRC-v3 a deterministic, measurable guard.
  Core finding: accuracy is NON-MONOTONIC in reasoning-token budget - it rises, PEAKS, then FALLS;
  "overthinking" = extending the chain makes the model ABANDON a previously-correct answer (a "negative
  flip"). The regime is flip_ratio>1.0 (negative flips exceed positive). Three deterministic-to-compute
  signals predict it: answer OSCILLATION (r=0.78, strongest), hesitation markers "wait/but/actually"
  (r=0.71), declining confidence trajectory (r=0.63); combined classifier 76.3% precision @ 80% recall.
  IMPLICATION 28 (the OVERTHINKING GUARD - a NEW deterministic component for GRC-v3, distinct from
  anti-spiral): GRC-v3 needs a per-call budget CEILING with an early-stop monitor on these three
  signals (oscillation is the lead). This is DIFFERENT from anti-spiral: anti-spiral catches REPETITION
  (same hypothesis/file 3x); overthinking catches the chain DRIFTING OFF a correct answer. Both are
  "stop" triggers, both deterministic (TD-3, no LLM judge - string scan + intermediate-conclusion
  tracking), but they are two different detectors. GRC-v3 runs both. This is the concrete mechanism for
  the user's "without spiraling" AND "reasoning can hurt."
  IMPLICATION 29 (the overthinking knee is MODEL-SPECIFIC and LOWER for small models - the J-34
  sibling): thresholds are model-specific (R1-32B ~7K, s1-32B ~5K) and the paper warns "do NOT apply
  the R1-32B threshold to smaller models; the threshold is lower for weaker models." ELAI's local
  Qwen2.5-Coder-7B will overthink EARLIER and at a LOWER budget than any model in this paper. So ELAI
  MUST measure its OWN per-(model,difficulty) reasoning-budget->verified-outcome curve and find the
  PEAK - this is EXACTLY the J-34 EMPIRICAL CONTEXT CALIBRATION pattern, applied to REASONING budget
  instead of context fill. J-34 measures fill->degradation; this measures reason-budget->accuracy (non-
  monotonic). They are siblings; GRC-v3 extends the J-34 bench (`elai bench reasoning-budget`) to find
  the per-(model,task-type) peak. Done-gate = ELAI's measured knee, NEVER the paper's 6K/7K (ID-5, and
  the paper itself forbids transferring the threshold). The user's "complexity classifier" + this curve
  = the auto-budget; the effort toggle is the manual lambda.
  IMPLICATION 30 (U_lambda IS the user's "balance by task + cost", and ELAI's I-27 is its receipt-
  grounded form): the paper's utility U_lambda(t)=Acc(t)-lambda*(t/t_max) with lambda the cost knob is
  literally "balance accuracy vs cost." ELAI's I-27 BUDGETED CONVERGENCE INSTRUMENTATION ("verified-
  outcome rate vs compute, Pareto frontier") is the SAME curve but with VERIFIED outcomes (receipts)
  replacing self-reported accuracy - the trust-spine-correct version (the paper's Acc(t) can itself be
  wrong; ELAI's is receipt-grounded). lambda = the user's effort toggle + COST COCKPIT (H-9)
  $/resolved-task. So I-27 + H-9 are the budget-optimizer; GRC-v3 drives them.
  IMPLICATION 31 (the deepest synthesis - ONE mechanism, THREE payoffs): "easy problems overthink
  EARLIEST" + the lagging flip indicator + the receipt-gate together yield this: once a candidate's
  external receipt PASSES, COMMIT and FREEZE that part of the beliefstate - do not let the chain keep
  generating and oscillate away from it. The user's "the part the model is more sure about proceeds,
  the rest stays" turns out to be simultaneously (a) the GROUNDING mechanism (receipt confirms it),
  (b) the COMPRESSION mechanism (committed = folded/lossy-safe, IMPLICATION 18), AND (c) the
  OVERTHINKING DEFENSE (committed = frozen against the model's own later second-guessing, which the
  blind spot IMPLICATION 20 says it cannot reliably do anyway). One commit operation, three payoffs.
  This is the elegant core of GRC-v3 and it is exactly the user's instinct, made rigorous: commit
  receipt-confirmed state and protect it from the generator that would otherwise overthink it away.
  NET: a NEW deterministic overthinking guard (3-signal early-stop, distinct from anti-spiral) + the
  per-model reasoning-budget knee measured via a J-34 sibling bench (local lane overthinks earliest) +
  U_lambda = the user's cost/effort balance = ELAI's I-27/H-9 receipt-grounded + the keystone synthesis
  that receipt-commit is grounding AND compression AND overthinking-defense in one operation. C2 ("no
  spiral, stay small") and C5 ("reasoning can hurt") are now both fully grounded and mechanized.

ROUND 2 SUMMARY (all 9 papers; before the deep-research report)
  Verdict pattern: the 2022 prompting papers (CoT, zero-shot CoT, least-to-most) are PARTIALLY-STALE -
  their elicitation tricks are absorbed into instruction-tuned models, so GRC-v3 does NOT re-teach
  reasoning; their STRUCTURAL insights (reason-before-answer, two-stage, decomposition, accumulated-
  prefix-as-state) survive. The agentic/search papers (ReAct, ToT, Reflexion, self-consistency) are
  PARTIALLY-STALE-to-VALUABLE and converge on ONE thing four independent times: the candidate selector
  / self-correction must be EXTERNAL (receipt > vote > NEVER self-eval), capped by the self-correction
  blind spot (the empirical keystone). The two scaling papers (test-time-compute 2024, overthinking
  2026) are STILL-VALUABLE and give the quantitative escalation+ceiling policy: route by difficulty
  (zero/depth/width/abstain), measure the per-model knee, defend against the non-monotonic overthinking
  fall.
  Net effect on posture: UNCHANGED in direction, much SHARPER in specification. GRC-v3 remains an
  ACTIVATION+UNIFICATION of I-11 (carrier) + I-4 (receipt-selected candidates) + I-14 (det. compression,
  preserve-receipts/fold-prose) + H-39 (abstention/confidence bit) + H-42/I-26 (depth budget) + I-7
  (escalation) + H-50 (regime context policy) + D-10/D-11/J-1/H-trading (domain validators) + J-10
  (blind-spot-safe reflection) + D-2/I-22/I-24 (Tier-2 PRM under receipts) + I-27/H-9 (cost utility),
  plus ONE genuinely new small component: the deterministic OVERTHINKING GUARD (3-signal early-stop)
  and its measurement bench (a J-34 sibling). No new model, no new mechanism class - the win is
  unification, the cross-domain framing (one carrier, domain-routed validators), and the receipt-commit
  triple-payoff. To be tested against the deep-research report next.

================================================================================
ROUND 3 - THE DEEP-RESEARCH REPORT (deep-research-report(5).md, the user's own commissioned synthesis)
================================================================================

The report covers the same 9 papers PLUS Plan-and-Solve, Self-Debugging, Cobbe-verifiers, Let's-Verify-
Step-by-Step, Math-Shepherd, ICLEval, RULER, Lanham-faithfulness. It is a near-perfect independent
confirmation of the GRC-v3 design derived above, and it adds six concrete things.
  CONVERGENCE (the report independently derives ELAI's router): the report's headline engineering rule
  is "use the cheapest possible method that matches task structure" (its line 19), and its practical
  DECISION TREE (lines 148-150) is: simple -> answer-only; medium multi-step -> short CoT/plan-and-solve;
  short checkable answer -> +self-consistency; branching search -> ToT/search; tools/state -> ReAct;
  useful feedback -> Reflexion/memory; huge context -> retrieval + compressed scratchpads + targeted
  rereads. This is EXACTLY GRC-v3's escalation ladder (I-4) gated by the 2-axis classifier (cost x type,
  D-10 mode router). An independently-authored research report and my paper-by-paper derivation land on
  the same router. High confidence this is the right shape.
  ADD 1 (the empirical classifier prior - the task-type x gain table, lines 77-84): concrete gain
  ranges per task type - simple: 0/few/negative; medium multi-step math/QA: +5..+30; hard symbolic/
  compositional: +10..+80 (multi-fold); agentic/tool: +10..+35; coding-verifiable: +3..+15; long-
  context/ICL: moderate/task-dependent. This is the empirical PRIOR the complexity classifier
  (IMPLICATION 26) seeds from. ELAI still MEASURES its own per-(model,task-type) curve (ID-5, J-34
  sibling), but this table is the starting shape and a sanity check on the measured numbers.
  ADD 2 (faithfulness - a THIRD independent grounding for receipts-not-traces, Lanham et al, report
  lines 21/124-126/176): "accuracy gains do NOT prove human-like internal reasoning... larger models
  can be LESS faithful... the visible rationale is not always the real causal process." Combined with
  ToT self-eval (IMPLICATION 13) and Reflexion's blind spot (IMPLICATION 20), this is the third
  independent result saying the reasoning TRACE is not trustworthy evidence. GRC-v3 hard-rule: the chain
  is for monitoring/UX only; ONLY the external receipt is admissible as the confidence signal. This is
  ELAI's F-4 cant-lie + the grounding-verification stack (D-10's 6 methods are binary pass/fail,
  MiniCheck-style, never "the chain looks right").
  ADD 3 (RULER + ICLEval - empirical grounding for C6 "context is the bottleneck" and J-34, report lines
  17/116-118/146): "usable context, not merely available tokens"; "long context alone does not guarantee
  robust reasoning; many models degrade sharply as context length and task complexity rise" (RULER:
  needle-retrieval fine but aggregation/multi-hop degrades with length). This is the DIRECT empirical
  basis for: (a) the C6 thesis (manage context, don't just enlarge it); (b) RD-4 (window is never the
  lever for a depth-class failure); (c) J-34 EMPIRICAL CONTEXT CALIBRATION (measure the usable knee,
  don't trust the advertised window); (d) the report's own prescription (line 146) "use retrieval,
  summaries, scratchpads, targeted rereads, NOT indiscriminate full-context stuffing" = LITERALLY
  ELAI's I-14 deterministic compression + DKG/A-RAG retrieval + context-budget allocator. The report
  prescribes ELAI's architecture by name-in-function.
  ADD 4 (HYBRID beats either alone - grounds the I-7 cascade + reason<->retrieve switching, report lines
  64-65/102): ReAct's best HotpotQA/FEVER scores came from HYBRIDS that switch between internal reasoning
  and external retrieval (ReAct->CoT-SC fallback 35.1; CoT-SC->ReAct fallback 64.6) - "hybrid beats
  either alone." This grounds ELAI's ESCALATION CASCADE (I-7) and the mode router (D-10): the engine
  should SWITCH between modes (reason vs retrieve vs escalate-model) based on receipts, never commit to
  one. The user's "escalating reasoning" is really "switch modality when the current one stops earning
  receipts."
  ADD 5 (Cobbe small-model nuance - reasoning steps are NECESSARY at <=7B, report line 110): a 6B model
  dropped 20.6 -> 5.2% when trained to emit the final answer WITHOUT reasoning steps. So on the local
  Qwen-7B lane you cannot skip the serialized reasoning on a non-trivial task (the small model NEEDS the
  steps) - BUT you also cannot trust its self-evaluation (IMPLICATION 4/13/22). Reconciliation: the
  engine IMPOSES the reasoning structure (decomposed steps) AND supplies the external receipts. Both
  halves: structure-from-engine + validation-from-receipt. This refines IMPLICATION 3 (the zero-reasoning
  rung is for TRIVIAL tasks only; non-trivial local-lane tasks still get engine-imposed structured steps).
  ADD 6 (over-optimization of search - grounds I-24 canary + receipt-over-PRM, report line 120): Cobbe
  found verifier accuracy on GSM8K rose then DECLINED beyond ~400 samples ("more search finds adversarial
  or misleading candidates"); Snell found stronger optimizers degrade easy questions by "overexploiting
  verifier errors." So the candidate-search itself can be GAMED at high N - which is exactly why ELAI
  caps candidate width (early-stop IMPLICATION 9), guards the PRM with ISOMORPHIC CANARY (I-24), and puts
  the deterministic RECEIPT above the PRM (IMPLICATION 27). The report independently motivates ELAI's
  anti-gaming layer.
  NET: the report is an independent, broader-sourced confirmation of the entire GRC-v3 design + the
  empirical classifier prior (task-type x gain table) + a third grounding for receipts-not-traces
  (faithfulness) + the RULER/ICLEval empirical basis for the context-as-pipes thesis + the hybrid/cascade
  switching result + the small-model "needs structure but not self-trust" reconciliation + the
  over-optimization motivation for the anti-gaming cap. Nothing in the report contradicts the design;
  several things sharpen it. Posture: CONFIRMED and ready for final synthesis.

================================================================================
ROUND 4 - LIVE-CODE VERIFICATION (elai.map grep, to ground "activate/unify" vs "invent")
================================================================================
Confirmed PRESENT in real code (recommendation rests on these existing):
  - GRC state machine: GrcState in crates/reasoning/src/config.rs + agent_config tests. WIRED.
  - Abstention/confidence: crates/orchestrator/src/abstention_score.rs - parse_eval_log_outcome reads
    correct/wrong/abstain FROM THE EVAL LOG (i.e. from a receipt, not self-report) + AbstentionScored
    SSB. This is the blind-spot-safe confidence bit (IMPLICATION 20) already in code. H-39 real.
  - Escalation cascade: i7_escalation_cascade_contract.rs, test "fail_condenses_then_escalates" +
    EscalationCascadeKind {LocalWin, Escalated, BatchRouted}. I-7 real, and condense-on-escalate exists.
  - Search ladder: i4_search_ladder_contract.rs (SearchNodeEvent, "branch_is_pruned_never_promoted") +
    run_best_of_n in crates/orchestrator/src/structural_retry.rs. I-4 substrate real (prune/promote).
  - Serial-depth budget: h42_serial_depth_budget_contract.rs + ReasoningBudgetScaled SSB. H-42 real.
  - Mode router: select_grounding_mode + locked_keys_d10 integration. D-10 real and locked.
Confirmed ABSENT / PLANNED (the things to build - matches the map):
  - BeliefState / PABU carrier (I-11) and ContextTransformReceipt (I-14): not found in code; PLANNED.
    These are the unbuilt keystones, as expected.
  - A dedicated OVERTHINKING guard (oscillation/flip_ratio over reasoning budget): NOT present (the
    "oscillation" grep hit only anti_spiral_extended.rs + telemetry, a different mechanism). Confirms
    IMPLICATION 28 - the overthinking guard is the one genuinely-new component.
Conclusion: the recommendation is correctly "activate + unify the existing GRC/abstention/escalation/
  ladder/budget substrate, BUILD the I-11 carrier + I-14 compression + the overthinking guard," not
  "invent a reasoning engine." Grounded.

================================================================================
FINAL SYNTHESIS - GRC-v3: THE REASONING-STATE ENGINE (what we should do)
================================================================================

ONE-PARAGRAPH ANSWER TO THE USER
  Your idea is right and it is, in large part, already ELAI's own (mostly-unbuilt) plan. The correct
  framing is: GRC-v3 = least-to-most DECOMPOSITION with a per-step EXTERNAL-RECEIPT gate, carried across
  model/context boundaries as a TYPED beliefstate (not raw transcript), escalated by a 2-axis classifier
  (task COST x task TYPE), and protected by an overthinking ceiling. The single most important
  correction to your wording: a model CANNOT reliably "ground itself" or judge which of its own parts is
  "sure" (the self-correction blind spot + faithfulness results - three independent 2024-2026 papers).
  So "the part the model is more sure about proceeds" must mean "the part an EXTERNAL receipt confirms
  proceeds." That single change is what makes the whole thing trustworthy, and it is exactly ELAI's
  existing trust spine (F-4 cant-lie, D-13 cross-family, I-4 select-by-receipts).

THE DESIGN (one carrier, one router, many validators)
  1. CARRIER (I-11, BUILD - keystone): a typed BeliefState replaces raw history on every cross-model /
     cross-tier / cross-unload boundary. Each entry = {claim, grounding-receipt, confidence/abstain bit
     (from H-39), provenance (DKG-keyed)}. This is what "carries reasoning state without spiraling" -
     it carries CLAIMS+RECEIPTS, not prose. Cloud lane keeps it behind the cache-stable prefix (H-50);
     local lane unloads it and selectively reloads (H-50) because local has no KV reuse.
  2. COMPRESSION (I-14, BUILD): deterministic + reversible (LSP/tree-sitter folding + receipts; ciphers
     BANNED IC-3). RULE from the papers: losslessly preserve the RECEIPTS/observations, treat the
     reasoning PROSE as a lossy projection rederivable from the receipts (IMPLICATION 18). This is "each
     subsequent step stays small" done safely. Cloud = deterministic masking of trailing observations
     behind the breakpoint (HD-2); never an LLM re-summarization (busts cache, unverifiable).
  3. CANDIDATES + SELECT (I-4, ACTIVATE - it is DARK): branch candidate beliefstates on the SessionDag;
     close each with its DOMAIN receipt; the receipt-passing candidate advances (commit + FREEZE it).
     Selector hierarchy (triply grounded): deterministic RECEIPT > statistical VOTE over independent
     samples (receipt-absent fallback only, never for factual-recall) > LLM self-eval (BANNED). Add the
     receipt-absent vote-fallback rung (IMPLICATION 8) and early-stop on receipt-convergence
     (IMPLICATION 9). Backtracking over the SessionDag (H-49) replaces a bespoke tree-search engine.
  4. COMMIT = TRIPLE PAYOFF (the elegant core, IMPLICATION 31): committing a receipt-confirmed beliefstate
     entry is simultaneously the grounding (receipt confirmed it), the compression (committed = folded),
     and the overthinking defense (committed = frozen against the generator second-guessing it away).
     This is your "sure part proceeds, rest stays" - and it is one operation with three wins.
  5. ESCALATION LADDER + CLASSIFIER (H-42 + I-26 ACTIVATE; classifier BUILD): route by a 2-axis
     classifier. TYPE axis (D-10 mode router, WIRED): reason vs RETRIEVE (recall/factual -> DKG/A-RAG,
     never a chain - it confabulates) vs validate. COST axis (difficulty proxy from H-39 confidence +
     C-45 case-base hit + J-38 retrieval coverage): trivial -> ZERO reasoning; easy/medium -> serial
     DEPTH (revise the beliefstate); hard -> parallel WIDTH (I-4 best-of-N / BSP); very hard
     (proxy<~0.05) -> escalate model (I-7) or ABSTAIN (I-8). The window is NEVER the lever (RD-4, RULER).
     The user's manual "effort toggle" = the lambda override on top of this auto-router (U_lambda, I-27).
  6. OVERTHINKING GUARD (BUILD - the one net-new component): a deterministic per-call ceiling +
     early-stop on 3 signals (answer oscillation r=0.78 lead, hesitation markers, declining confidence),
     flip_ratio>1.0 = stop. Distinct from anti-spiral (which catches repetition). TD-3, no LLM judge.
  7. REFLECTION = J-10 (ACTIVATE): cross-attempt memory is the blind-spot-safe insight-from-AUDIT-REPORT
     (external) buffer, not self-critique. After ~3 failed receipted retries -> CHANGE MODALITY (retrieve
     / escalate / abstain), never loop (IMPLICATION 23).
  8. DOMAIN VALIDATORS (the cross-domain answer to C4 - one carrier, routed receipts): code -> tests/
     compile/F-4; math/trading-ML -> D-11 checker registry (sympy/z3) + H-17..H-29 trading gates; visual/
     Unity -> MEDIA VALIDATOR (SSIM/LPIPS/pixel-diff) + J-1 perception (NEVER a VLM on a diff). The
     reasoning is identical across domains; only the receipt that closes a candidate changes. THIS is how
     one engine helps the math bug, the Unity bug, and the codebase bug - it doesn't reason differently,
     it validates differently.
  9. PRM as Tier-2 (D-2/I-22 ACTIVATE, under I-24 canary): for intermediate steps with no deterministic
     receipt yet, the dark PPM sidecar scores steps - but always UNDER the Tier-1 receipt, guarded by the
     isomorphic canary against style-gaming (search over-optimizes past ~400 samples, report ADD 6).

WHAT IS REUSE vs BUILD (so this is an activation plan, not a moonshot)
  REUSE/ACTIVATE (already in code, mostly DARK/PARTIAL): GRC state machine, abstention_score (H-39),
  EscalationCascade (I-7), search ladder + run_best_of_n (I-4), ReasoningBudgetScaled (H-42), mode
  router (D-10), SessionDag branch/merge + H-49 backtrack, anti-spiral, J-10 insight memory, D-2/I-22
  PRM, I-24 canary, I-27/H-9 cost utility, H-50 regime context policy.
  BUILD (the genuinely missing pieces, in priority order):
    P1 (keystone) I-11 typed BeliefState carrier - everything else needs it; it is unbuilt; it is what
       makes "carry reasoning across a boundary" real. Highest leverage.
    P2 the 2-axis complexity classifier (deterministic, from existing signals) + wire it to the H-42/
       I-26 budget. This is the user's "complexity classifier + effort toggle."
    P3 the deterministic OVERTHINKING GUARD (3-signal early-stop) + its measurement bench (a J-34
       sibling `elai bench reasoning-budget` measuring the per-(model,task-type) accuracy-vs-budget knee).
    P4 I-14 deterministic reversible compression (preserve-receipts/fold-prose rule).
    P5 activation wiring: un-dark I-4 + D-2 + I-26 under the receipt-first + canary rules; thread the
       beliefstate through I-7's condense-on-escalate (the WIRED cascade already has the seam).

DONE-GATE (non-negotiable, ID-5 / GD-3): the win is proven by an ELAI-MEASURED curve, never a paper
  number. `elai bench reasoning-budget` + the I-27 convergence instrument produce, per (model, task-type),
  the verified-outcome-rate vs compute curve; GRC-v3 ships when the auto-router demonstrably sits on or
  near that curve's compute-optimal knee on an ELAI gold set (the J-34 pattern). Receipts (not accuracy)
  on the y-axis. SAFEDELTA-style gate-ON-vs-OFF framing, not a leaderboard rank.

THE HONEST TENSIONS / RESIDUE (do not oversell)
  T1. Depth-vs-width vs RD-4 (IMPLICATION 25): "depth-not-width" is right that WINDOW is never the lever,
      but the test-time-compute paper shows HARD problems want parallel WIDTH, not more revision-depth.
      GRC-v3 resolves it (easy/medium->depth, hard->width, window->never) but this is a refinement of the
      H-42 doctrine that should be stated explicitly in any plan, not hidden.
  T2. The classifier's difficulty proxy is itself imperfect (the papers warn a true pass@K oracle is
      expensive). ELAI proxies from confidence/case-base/coverage; this must be MEASURED for calibration,
      not assumed. OBSERVE-ONLY first (GD-4) before it gates anything.
  T3. The overthinking flip-ratio is a LAGGING indicator (fires after onset); the lead signal is rising
      oscillation. The guard must act on the lead signal, accepting some false-positive early stops
      (76.3% precision in the paper) - tune recall vs precision on ELAI's own data.
  T4. Local Qwen-7B overthinks EARLIEST and at the LOWEST budget of any model studied; its self-eval is
      noise. The local lane is the hardest case and the place this design matters most - and the place the
      "engine imposes structure + supplies receipts, model only generates" rule is mandatory, not optional.
  T5. This is UNIFICATION. Its risk is not "will a piece work" (they're grounded) but "will the seams
      between I-11/I-4/I-7/H-42/D-10 compose" - which is exactly the wiring-enforcement R10 integration-
      seam risk. Any plan must ship the carrier + a consumer + an integration smoke together, not the
      carrier alone.

NEXT STEP (not done here, needs user call): if the user wants this built, it is a new plan family or a
  K/L-plan track authored via paper-to-plan / planning-workflow (vertical slices, P1 BeliefState first
  with its I-7 consumer + smoke). This reasoning doc is the grounding for that authoring; it does not
  author plan files (spec-eval boundary).


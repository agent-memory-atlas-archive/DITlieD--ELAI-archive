ELAI spec-eval: ambition, overengineering, competitive standing, improvement directions
Date: 2026-07-05. Mode: spec-eval AS-SPECIFIED (design judged as if fully built; shipped-vs-planned excluded from the verdict per the skill).
Inputs read this session: ELAI-SYSTEMS-OVERVIEW.md (full), phase-index-MASTER.txt (full), B/C/D/E/F/G/L/M/N summary indexes (full), H/I/K summary indexes (first ~40-60%), J summary index (premise + track map; concepts covered via the overview, which is the sanctioned digest).
Grounding discipline: claims about ELAI cite the spec text read this session. Claims about competitors are TRAINING-BASED (user explicitly authorized "based on your training knowledge"); every such claim is tagged [training, unsearched]. GRC ledger: topic spec-eval-ambition, 2 decision entries.

================================================================================
1. WHAT THE SPEC ACTUALLY DESCRIBES (the object under judgment)
================================================================================

One sentence: a deterministic, journal-centric autonomous coding agent where every decision that
can be pushed out of model discretion is pushed into frozen harness machinery, and everything the
model does produce is gated against off-agent evidence the model did not author.

The organizing invariants, extracted from the corpus (these are the load-bearing design ideas, not
the feature list):

I1. ENTROPY QUARANTINE + REPLAY. No raw time/randomness; ELAI_SEED gives byte-identical runs;
    the SSB append-only journal is the causal record; the RRA makes a run a portable, third-party
    re-executable artifact (G-2, K-20 re-executable extension).
I2. OFF-AGENT, FAIL-CLOSED EVIDENCE. Every gate measures an artifact the generating agent did not
    author (SSB row, coverage hit, stdout blake3); missing evidence = FAIL; the gate is frozen and
    read-only to the run it judges (wiring-enforcement R1-R10, F-4 cant-lie, gate-manifest freeze).
I3. WIRE-DARK IS A FIRST-CLASS DEFECT CLASS. "Compiles + unit tests pass" is structurally blind to
    a producer with no production caller; the layered wiring stack (L1..L4) exists because ELAI
    measured its own wire-dark rate at ~40-45% across ~187 plans.
I4. THE MODEL GENERATES/SELECTS; THE HARNESS DECIDES/GATES. Mode routing, gate intensity,
    promotion, tier selection are all deterministic tables the LLM cannot influence (TD-3,
    D-10, K-5, N-5).
I5. SELF-IMPROVEMENT ONLY BEHIND AN UN-GAMEABLE, STRONGER-THAN-GENERATOR FILTER. The convergence
    theorem is taken seriously: execution-grounded fitness, behavioral traps, eval firewall,
    reward-tampering gate, frozen thresholds (H-43..48, D-1..D-9, K-19 SF rules).
I6. CLAIMS, NOT JUST CODE. The L-plan turns the same spine outward: typed claim extraction, a
    3-predictor convergence fuse, an interventional do-test, risk-routed targeted effort — a
    model-agnostic hallucination gate over any backend.
I7. EVERY NUMBER IS AN ABLATION RECOMPUTABLE FROM THE JOURNAL. Gate-ON vs gate-OFF, under seed,
    receipts on the y-axis; no leaderboard ranks; claims-policy lint blocks unmeasured superlatives
    (KD-1, GD-2, GD-3).
I8. SELF-CORRECTION OF THE PLAN CORPUS ITSELF. N-plan machine-checks plan-to-plan composition
    (N-2), unifies fragmented concerns, and makes "a second implementation of a unified concern"
    a failing check.

Scale of the specified system: ~14 plan letters, ~450+ session-grain units, a 38-file phase spec
underneath, plus a commercial layer (M) and a unification layer (N). Estimated effort stated in
the indexes alone: C ~200-264h, D ~160-220h, E ~330-660h, and H/I/J/K/L on similar-or-larger
scales — thousands of hours of specified work.

================================================================================
2. IS THIS THE MOST AMBITIOUS HARNESS I KNOW? 
================================================================================

Short answer: on integrated scope, yes — with the honest caveat that "most ambitious" is exactly
the kind of unmeasured superlative ELAI's own K-1 claims-policy lint would reject, and closed
labs' internal harnesses are unobservable.

[training, unsearched] The comparison field as of my knowledge:

- OpenHands (ex-OpenDevin): the largest OSS agent harness — sandboxed runtime, event stream,
  evaluation harness, agent hub. It has an event log but no determinism substrate, no fail-closed
  off-agent gate doctrine, no attestation, no wiring-enforcement concept, no filtered
  self-improvement loop. Broad but shallow relative to ELAI's spine.
- SWE-agent / Agentless / AIDE and research harnesses: deliberately narrow; Agentless is notable
  for the opposite bet (a 3-step fixed pipeline outperforming agentic loops at the time).
- LangGraph / LangChain / DSPy: frameworks, not opinionated harnesses. LangGraph has
  checkpointing/time-travel (a cousin of SessionDag/H-49) but no verification spine. DSPy/GEPA is
  prompt-program optimization — ELAI ingests it as one lane (E-8) rather than being it.
- Devin / Cursor / Claude Code / Codex / Copilot Workspace / Kiro: production harnesses. They ship
  permissioning, hooks, sandboxes, planning modes, cloud parallelism — but (publicly) nothing like
  deterministic replay, evidence-bound test outcomes, signed attestation, or a claim-grounding
  gate. Kiro is spec-driven authoring; that overlaps ELAI's decomposition tree, not its trust spine.
- Deterministic simulation testing exists in DATABASES (FoundationDB, TigerBeetle, Antithesis);
  in-toto/SLSA attestation exists in SUPPLY CHAIN; taint tracking exists in IFC research (CaMeL,
  RTBAS, FIDES — which ELAI cites directly). ELAI's genuine novelty is not any one mechanism; it
  is importing all of them into one coding-agent harness and making the journal the join point.

What makes the ambition qualitatively different, not just bigger:
(a) It targets a different objective function than every commercial peer: verified-trust per
    dollar, not capability per benchmark. The whole gate stack, RRA, DSR, SAFEDELTA exist to make
    a number a skeptical third party can recompute.
(b) It treats its OWN development process as part of the system: the harness rules (wiring
    enforcement, production-caller gate, maintainability review, orchestrate protocol) are the
    same doctrine the product enforces, applied to the humans+LLMs building it. I know of no other
    project [training, unsearched] where the meta-level and object-level are this deliberately
    isomorphic.
(c) It plans for its own obsolescence pressure: K-8 provider-absorption pre-gate, G-10/K-9 model
    drift and refresh cadence, N's fragmentation audit. Most ambitious systems do not spec their
    own depreciation machinery.

Verdict: as a DESIGN, ELAI is the most comprehensive single-seat agent-harness spec I can name.
The superlative is unverifiable in principle (closed internal systems), so the defensible claim
is: no publicly known harness [training, unsearched] combines even half of I1-I8.

================================================================================
3. OVERENGINEERING OR AMBITION-WITH-REAL-NEEDS?
================================================================================

Test applied: complexity is load-bearing iff it is indexed to a documented failure mode with
evidence; otherwise it is speculative and dilutive.

3a. THE CORE PASSES THE TEST. Nearly every gate in F/G/H-Track3/I-Track15 cites either a measured
internal number or a documented public incident:
- wire-dark ~40-45% of ~187 shipped plans (wiring-enforcement.md) -> the entire L1..L4 stack;
- fabricated test PASSes (Replit incident; the agent-writes-its-own-gate problem, Berkeley RDI
  gamed-~100% citation) -> F-4 stdout-hash binding + frozen gates (R8);
- rm -rf #10077/#12637 post-expansion escapes -> G-3 canonicalize-before-verb;
- hallucinated deps 19.7%/205k fake names (Spracklen, cited in F-9) -> write-time existence gate;
- security degradation +37.6% over 5 AI iterations -> F-10 monotonicity gate;
- CodeGate died of FP friction -> GD-4 bounded-FP-before-default-ON discipline;
- the git-reset-x8 data-loss incident -> the agent-dispatch preamble and supervisor plane (I-1);
- selection bias in autonomous search (max of N noisy Sharpes) -> H-17 persisted trial ledger.
This is the opposite of overengineering: it is failure-mode-driven design with unusually good
receipts. The needs are real because the incidents happened — several of them to ELAI itself.

3b. THE PERIPHERY PARTIALLY FAILS THE TEST. Objective evidence, from the corpus itself:
- N-plan's findings: FOUR claim types, TEN routers, SIX verification-intensity deciders, FIVE
  provenance scales, FOUR counterfactual engines, FIVE context cores, duplicated MAST classifiers
  (E-6/E-29). N exists because the letter-machine over-produced design faster than it reconciled
  it. That is the signature of overengineering at the corpus grain, self-diagnosed.
- Whole verticals whose need is prospective, not documented: the trading-trust track (H-17..29,
  13 sessions of finance-specific gates), Empire integration (B-25/C-34, blocked on an external
  codebase for over a year of plan-time), canvas APP-DESIGN mode (H-14, flagged XL/riskiest in its
  own entry), the diffusion draft lane (I-29), the GRPO flywheel (I-31 — correctly gated LAST but
  still carried).
- Redundant stacked assurance in places: for one claim, ELAI can fire round-trip + semantic
  entropy + cross-family adversary + sycophancy probe + conformal refusal + SLA monitor + policy
  gate. Until L-10's measured delta exists, the marginal detection value of each layer is a design
  hypothesis. The spec knows this (BENCH-PENDING everywhere) — but the build cost is paid before
  the marginal value is known.
- The plan corpus itself is a context artifact: ~450 units, each written to be read by LLMs with
  summaries-of-summaries, which N-2 then has to machine-check for drift (J-SUMMARY self-declares
  drift at J-91). The system fighting context rot in models reproduced context rot in its docs.

3c. VERDICT. Not overengineering in the classic sense (speculative flexibility nobody asked for);
the core is need-indexed to a degree most production systems never reach. But the corpus exhibits
BREADTH overengineering: too many parallel fronts opened before the keystone numbers (SAFEDELTA,
gate-tax, hallucination-delta, routing-economics) that would tell you which fronts matter. The
right frame: ELAI is a justified cathedral with too many side chapels under construction at once.

================================================================================
4. DOES ELAI LOSE TO ANY OTHER HARNESS/METHOD?  [training, unsearched throughout]
================================================================================

Where ELAI as-specified WINS at the design level: evidence/trust (no peer has deterministic
replay + attestation + fail-closed off-agent gating), cost-governance (hard ceilings that block,
cache-invariant middleware, measured routing), safety floors (taint, argv allowlists,
write-time gates), self-improvement safety (nobody else specs the convergence-theorem filter),
and honesty machinery (claims lint, abstention scoring, refusal SLA).

Where ELAI LOSES or is structurally at risk:

L1. RAW CAPABILITY PER TASK. Nothing in the spec raises the model's ceiling; the harness extracts,
    verifies, and compounds latent capability but cannot create it. A frontier model in a thin
    harness (Claude Code) will beat governed-ELAI-on-a-7B on hard novel tasks, full stop. ELAI's
    own docs concede the wedge is trust, not capability. The search ladder (I-4), case graph
    (VCG), and flywheel (Track 17) are the capability counters — all late-sequenced.
L2. ITERATION VELOCITY / FRICTION. Every gate is a tax; K-4 exists because the corpus knows
    composed friction is the category-killer. Model-first products ship each model improvement
    same-day; ELAI must re-verify its scaffolding against each. The bitter-lesson risk is real
    and partially conceded by K-8 (absorption pre-gate): providers keep eating scaffold layers
    (caching, tool search, structured outputs already partially absorbed during the plan's life).
L3. ECOSYSTEM. Claude Code skills/MCP/plugins and OpenHands' community produce network effects a
    single-author system cannot match by construction. M-5's elai-verify OSS release is the only
    seed of a counter-move, and it is sequenced last.
L4. CO-TRAINED MODEL+HARNESS. Cursor (Composer), Devin, and the labs can train the model to fit
    the harness. ELAI's flywheel is the answer but is correctly gated behind the whole trust
    stack, i.e., years away in sequence terms.
L5. SIMPLICITY AS A METHOD. Agentless-style results showed fixed simple pipelines can beat
    agentic complexity on well-defined benches. ELAI internalizes the lesson only partially
    (E-10 hostile ablations, iso-budget role deletion) — the honest risk is that a large fraction
    of the 450 units would not survive an iso-budget ablation, and the ablation machinery itself
    is one of the planned units.
L6. TEAM/CLOUD EXECUTION. Deliberately single-seat, local-first, no org SaaS (M defers SSO/RBAC).
    Devin-style parallel cloud fleets serve a real demand ELAI cedes by design.
L7. FORMAL-METHODS-FIRST NICHES. For safety-critical code, verified-synthesis toolchains
    (Dafny/Verus-centric) are stronger in-lane; ELAI keeps E-53 as a side verifier, which is the
    right call but means it does not win that niche.

On the user's thesis ("the limit is not model power, it's the system around it"): the corpus
itself gives the calibrated answer. For the failure modes that dominate real deployment — lying
about outcomes, wire-dark shipping, security regressions, cost blowouts, hallucinated claims —
the system IS the binding constraint, and ELAI is the strongest specified attack on exactly that.
For per-task capability ceilings, the thesis fails: E-8's own citation (small LMs cannot
self-optimize), the local-lane VRAM realities in the memory bank, and the METR-horizon style
evidence all say model power binds. ELAI's most defensible form of the thesis: the system
determines how much of a model's capability you can TRUST and COMPOUND, and trust-per-dollar is
a system property, not a model property. That version is right and is the actual wedge.

================================================================================
5. HOW I WOULD IMPROVE IT / WHAT TO ADD / WHAT TO REMOVE
================================================================================

5a. THE ONE STRUCTURAL IMPROVEMENT: declare a PROOF SPINE and starve everything else.
The corpus's credibility compresses onto ~6 numbers: SAFEDELTA (G-6), gate-tax (K-4),
hallucination-delta (L-10), routing-economics (K-2), case-reuse (K-18), and the wire-dark rate
delta (already partially measured). Everything commercial (M) is already gated on them (GATE M-0).
Proposal: freeze new-letter authoring (N-2 makes this enforceable), rank every unbuilt unit by
"does it feed a proof-spine number," and park the rest. This is not my doctrine — it is ELAI's
own KD-1 (number-before-claim) applied to the roadmap itself. The letters currently invert it:
design-before-number at corpus grain.

5b. REMOVE (mark ABANDONED in the feature map, not DEFERRED — deferred items are carried
cognitive liabilities the summaries keep re-reading):
- Empire integration (B-25/C-34): blocked on an external dependency for the plan's entire life.
- The trading vertical H-17..H-29 AS A VERTICAL: N-3 already plans to lift its three primitives
  (trial ledger, deflation gate, holdout budget) into domain-generic promotion machinery — do
  that, then delete the finance skin until a finance user exists.
- H-14 canvas APP-DESIGN mode: consumer-product scope (Onlook territory) inside a trust product;
  XL effort, MODERATE grounding, riskiest-in-track by its own entry.
- I-29 diffusion draft lane and the Cosmos artifact tier's research cadence: nice-to-have
  capability breadth with no proof-spine contribution.
- The RL end of Track 17 (I-31) until I-30's SFT rung produces a measured lift — the spec already
  sequences it LAST; I would remove it from the corpus and re-author when the prerequisite
  numbers exist (the design will be stale by then anyway; K-9/G-10 say models drift quarterly).

5c. ADD — three systems the corpus genuinely lacks:

ADD-1. THE EVIDENCE STANDARD AS THE NETWORK-EFFECT PLAY. M-5 open-sources elai-verify (the
verifier binary). Go one level up: publish the RRA/attestation/claim-receipt SCHEMA as a versioned
open standard other tools can emit, with elai-verify as its reference verifier. ELAI cannot win
ecosystem wars on plugins (L3); it can plausibly win on being the FORMAT in which agent evidence
is exchanged — the way SLSA/in-toto won supply chain not by being a build system but by being the
attestation format [training, unsearched]. This converts the moat from "our binary" to "our
grammar," which single-author projects CAN defend.

ADD-2. A CONTINUOUS ABSORPTION WATCHDOG. K-8 is a manual quarterly audit. Make it a live system:
the provider-surface diff (new Anthropic/OpenAI capabilities) auto-flags overlapping ELAI units
in the feature map, the same way G-10 watches model drift. The scaffold-absorption risk (L2) is
the single biggest strategic threat and currently has the weakest machinery.

ADD-3. COUNTEREXAMPLE-TO-SPEC ROUTING (capability, not just trust). The gates generate the best
possible raw material — concrete counterexamples (surviving mutants, failed do-tests, MR
violations, refused claims) — and today they route to retries and audits. Route them UPWARD into
requirement elicitation: a surviving mutant is a missing acceptance criterion; a refused claim is
an ambiguous spec line. The harness side has /grill; the product has the ambiguity score (B-38)
feeding only an offer (N-10 starts wiring this). A first-class "counterexample -> user question ->
spec delta" loop would be the first mechanism in the corpus that makes the AGENT MORE CAPABLE on
the next attempt rather than merely better-verified on this one — and it is exactly the kind of
system-not-model capability gain the user's thesis predicts should exist.

5d. SMALLER IMPROVEMENTS:
- Collapse the letter process into the feature-map + unlock-DAG as the only living plan artifact
  (N-2's derivation direction, finished). The 14-letter corpus is itself a context-rot liability;
  the system that solved context management for models should not need 1000-line summary indexes
  for humans.
- Make the local-lane bet bench-gated go/no-go early: dozens of units assume a governed 7B is
  useful beyond mechanical edits; L-21 and the measured baselines suggest the honest role is
  narrower. One early bench (verified-outcome rate per lane) would re-scope a whole track.
- Publish the wire-dark rate longitudinally (before/after the enforcement stack) as the FIRST
  public number — it is already internally measured, uniquely ELAI's, and needs no RunPod.

================================================================================
6. IMPLEMENTATION-READINESS NOTE (explicitly outside the verdict, per spec-eval)
================================================================================
Index-derived, confirm against live code: B largely DONE; C/D largely PLANNED; E partially;
F/G substantially shipped per H/I grounding maps; H/I mixed ACTIVATION/BUILD; J/K/L/M/N mostly
PLANNED with some K-native doc artifacts landed. None of this affected Sections 2-5.

================================================================================
7. HONEST LIMITS OF THIS EVALUATION
================================================================================
- All competitor claims are training-based and unsearched (user-authorized basis); the field
  moves monthly and my knowledge has a cutoff.
- H/I/K indexes were read to ~40-60% depth (token caps); J via its premise + the overview. The
  unread portions are more session entries of the same shape; a section contradicting the
  verdicts above is unlikely but not impossible.
- "Most ambitious" is unverifiable against closed internal systems; the defensible claim is the
  I1-I8 combination has no publicly known peer.
- The overengineering verdict on the periphery is a judgment call on opportunity cost, not a
  defect proof; if the solo+agent-fleet build throughput is high enough, breadth is cheaper than
  I priced it.

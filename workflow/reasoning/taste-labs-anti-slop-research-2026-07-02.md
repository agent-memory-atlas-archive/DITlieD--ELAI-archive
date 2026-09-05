TASTE LABS — ANTI-SLOP METHODOLOGY, GROUNDED RESEARCH DIGEST
date: 2026-07-02
purpose: extract the TECHNICAL ways Taste Labs reduces AI slop, then translate into concrete
         skill/instruction patterns so our own models produce better (less-generic) design output.
grounding: web sources cited inline with tier. forcreativegirls + solofounders + tastelabs.com/ were
           bot-blocked (403/404); their content is recovered via the investor/analysis writeups and the
           video chapter list the user supplied (treated as tier-2 primary-adjacent).

================================================================================
PART 1 — WHAT "SLOP" AND "TASTE" ACTUALLY ARE (their framing)
================================================================================

Root-cause claim (Amplify, tier-2 investor): "generative AI has made creation cheap. What it has
not made cheap is judgment. And when judgment is scarce, slop fills the void." The cure for slop is
NOT better generation, it is better JUDGMENT. Models are strong on objective domains (math, code,
object-ID) and weak on subjective evaluation (design, writing, tone, brand).

Slop's real cause = VOLUME (video chapter 13:09, user-supplied; corroborated by the Ira Glass "Gap"
framing). Ira Glass's Gap: a creator's taste outruns their execution early on, and only huge volume
closes the gap. Applied to models: a model can generate infinite volume but has no taste to select
against, so volume without judgment = slop at scale. A quirk on one person is a signature; the same
quirk replicated across millions of outputs is slop (chapter 11:30).

Taste (their working definition) = the ability to EXERCISE JUDGMENT in domains where there is no clear
right answer — "measure, judge, codify, and search over what 'great' means." Not personalization.
Basic judgment (avoid-the-bad) must exist BEFORE per-person/per-brand personalization is even meaningful.

================================================================================
PART 2 — THE TECHNICAL MECHANISM (how they turn subjective quality into a learnable signal)
================================================================================

This is the load-bearing part for us. Grounded chiefly in Amplify + tanayj.com "RL Beyond the
Verifiable" (tier-2/3), which describes the exact rubric-RL recipe Taste Labs sells.

2.1  RUBRIC DECOMPOSITION (the core move)
- Expert humans write an INSTANCE-SPECIFIC rubric per task — a checklist of what a quality response
  must have/avoid. Not a global rubric; one tailored to the specific artifact/prompt.
- Each rubric item is made concrete enough to be checked PROGRAMMATICALLY. Instead of a noisy
  "rate 1-10", items are binary/near-binary concrete questions:
      "Does it mention X?"  "Does it AVOID Y?"  "Does it handle Z?"
  Fuzzy judgment -> a set of checkable yes/no items -> scorable at scale.
- Grounding: tanayj — "rubric items ask concrete questions ... each component becomes nearly
  verifiable, enabling consistent scoring at scale." Scale AI research is the cited prior art.

2.2  RUBRIC-AS-REWARD (making it trainable)
- An LLM-as-judge scores an attempt against the rubric checklist; the aggregate becomes a REWARD
  signal for RL — this is how you do RL in domains with no test suite ("no unit test for a good memo
  or a design").
- Enhanced variants (tanayj): GENERATIVE reward models reason before scoring; PROCESS reward models
  grade the intermediate reasoning steps, not just the final artifact.
- Product form Taste Labs sells to labs: "preference datasets, reasoning data, rubrics, and
  evaluation environments" (Amplify). Reasoning data = the WHY behind a judgment, not just the label.

2.3  THE AVERAGING PROBLEM (their sharpest technical claim)
- Standard RLHF averages everyone's preferences -> "you are left with no taste at all." Optimizing
  against broad-average preference data makes the model better at satisfying the reward function but
  WORSE by the standards that actually matter (expert standards). Goodhart on taste.
- Fix: do NOT estimate what the average person thinks. Identify the people whose judgments are most
  PREDICTIVE OF QUALITY, and anchor the reward to THEM. Rubric-based scoring sidesteps averaging by
  anchoring to expert-defined criteria instead of crowd aggregation.

2.4  TRUST-PROPAGATION TASTEMAKER NETWORK (~800 people, chapter 25:17)
- Annotators are not scaled generic raters. They recruit vetted individuals with demonstrated
  aesthetic judgment (designers/artists/creators) who then NOMINATE others — socially-validated taste,
  mirroring how taste actually propagates culturally. Small curated group + tooling >> large crowd.

2.5  TWO-PHASE QUALITY (raise the floor, THEN personalize) — Amplify
- Phase 1 RAISE THE FLOOR: teach the model to reliably AVOID bad outputs — the crude, incoherent,
  derivative, "not worth selecting". This is the universal anti-slop layer.
- Phase 2 PERSONALIZE: only after a reliable floor exists, optimize toward a specific
  community/brand aesthetic. Personalization on top of no-judgment is meaningless.

2.6  CRITIC vs MAKER (chapter 27:13, and the "lab" framing 19:37)
- A model that can CRITIQUE/JUDGE greatness is a DIFFERENT capability from one that GENERATES it.
  The critic (evaluator/reward model) is the thing you build first and is what actually kills slop;
  the maker inherits the taste through the reward. "Lab" = the discipline of turning subjective
  quality into objective, measurable signal.
- Intent / interpretation / commissioning (chapter 30:04): high-quality creative output is framed as
  a commission — the human supplies INTENT, the model INTERPRETS, and judgment lives in how well the
  interpretation serves the intent. This is a rubric axis, not vibes.

2.7  APPLICATION-LAYER TOOLS (the other product)
- For app companies (not labs): "context and verification tools" so agent outputs are more on-brand,
  more creative, less generic. Verification tool = a runtime check that an output clears the rubric
  BEFORE it ships — an off-model gate, not the generator grading itself.

Design was chosen as the launch domain because taste is most VISUALLY legible there (clashing colors,
soulless layouts are the clearest judgment test).

================================================================================
PART 3 — PRIOR ART WE FOUND WHILE RESEARCHING (directly reusable)
================================================================================

VOIDXAI/taste (github, tier-3, a real Claude Code "taste" skill): a 5-dimension engineering-judgment
skill. This is the closest existing artifact to what the user wants to build. Structure worth stealing:
- Dimensions: Code / Architecture / Product / Design / Communication, each with ONE core tension and
  ONE ultimate test question:
     Design      -> Intentionality over decoration -> "Does every pixel earn its place?"
     Architecture-> Resilience over elegance       -> "Will this survive the next 3 requirement changes?"
     Product     -> Composability over completeness -> "Does removing this make the product better?"
     Code        -> Consistency over cleverness     -> "Can a tired engineer at 2am understand this?"
     Communication-> Teaching over describing        -> "Can a newcomer understand intent without asking?"
- Design dimension territory: hierarchy, typography, spacing, color, motion, components.
- Anti-fake-precision rules (IMPORTANT): NO weighted overall score. Only score a dimension you have
  DIRECT EVIDENCE for; skip the rest "with a note, not a guess". Dimension boundaries prevent
  double-penalizing one issue across dimensions.
- Four modes: Quick Judgment (verdict + single highest-leverage fix) / Deep Review (dimension-by-
  dimension) / Guide (taste-informed generation with explicit rationale) / Compare (tradeoff analysis).

Note how this converges with our OWN existing rules:
- .claude/rulebook/design-taste.md (one locked accent, tinted neutrals, measured contrast, authored
  font, focal hierarchy not equal-thirds/divitis, motivated ease-out motion + prefers-reduced-motion,
  full state coverage, anti-slop tells) IS already a rubric — it just is not yet wired as an
  off-model verification gate or an RL/eval reward.
- .claude/rules/visual-ui-loop.md ALREADY implements the "verification tool" + "unknown=fail" pattern
  Taste Labs describes: schema-constrained pass|fail|unknown critique, mandatory evidence region,
  describe-before-verdict, one-fix-per-loop, two-pass stop gate. This is a critic-not-maker gate.

================================================================================
PART 4 — ACTIONABLE TRANSLATION: how to make OUR models' design better
================================================================================

The single highest-leverage lesson: separate the CRITIC from the MAKER, and make the critic an
off-model rubric gate the generator cannot satisfy from inside its own boundary. We already do this
structurally for wiring (off-agent auditor) and for visuals (visual-ui-loop). Extend the same doctrine
to DESIGN TASTE.

A. WRITE INSTANCE RUBRICS, NOT VIBES (Part 2.1)
   For every design surface, decompose "is it good" into concrete checkable items:
   binary form ("has one locked accent? y/n", "any element centered-nested-divitis? y/n", "authored
   font present? y/n", "empty/loading/error/stale states covered? y/n"). design-taste.md is ~80% of
   this list already; convert its prose dials into an explicit yes/no checklist per surface.

B. UNKNOWN = FAIL (Part 2.3 + visual-ui-loop already does this)
   A dimension the judge cannot verify from evidence is NOT a pass. This is the anti-rubber-stamp
   teeth. Keep it; extend it to the taste checklist, not just pixel-diff.

C. AVOID THE AVERAGE (Part 2.3)
   Do NOT tune toward "what most users would rate 7/10" — that is exactly how you get generic AI
   default (the same purple gradients). Anchor the reward/gate to a small set of high-signal
   references (our golden frames, a locked brief, a named tastemaker reference), not crowd average.
   Practically: seed the critique checklist from the design brief / golden render, never from
   "looks fine".

D. RAISE THE FLOOR BEFORE PERSONALIZING (Part 2.5)
   Ship a universal ANTI-SLOP FLOOR gate first: the presence of ANY AI-default tell (default system
   font only, equal-thirds layout, unmotivated gradient, generic centered hero, zero state coverage)
   is a BLOCK regardless of brand. Only once the floor passes do brand/accent personalization checks
   run. This maps cleanly onto our two-layer rule structure.

E. CRITIC IS OFF-MODEL AND FROZEN (Part 2.6 + our auditor/freeze-the-gate doctrine)
   The generator must not grade its own design. Route the taste verdict through the auditor / a
   verification tool the executing run cannot edit — identical to wiring-enforcement R8 freeze and
   "main model never runs tests". A design-taste checker that the generator authored is rubber-stamp.

F. VERIFICATION TOOL AT RUNTIME (Part 2.7)
   Add an off-model gate that checks an output clears the taste rubric BEFORE it ships, analogous to
   ui-wire-smoke.mjs but for taste tells. Deterministic subset first (font present? one accent?
   contrast measured? states present?) then the schema-constrained VLM critique for the qualitative
   rest. This is exactly the layered-gate shape we already use for wiring.

G. REASONING DATA, NOT JUST LABELS (Part 2.2)
   When we capture a design judgment (accept/reject a render), store the WHY (which rubric item, which
   evidence region) not just the verdict. That is the "reasoning data" Taste Labs sells and it is what
   makes a judgment teachable/reusable rather than a one-off vote. Feeds design-taste.md refinement.

CONCRETE NEXT STEPS (proposed, not yet executed):
1. Convert .claude/rulebook/design-taste.md prose dials into an explicit per-surface YES/NO rubric
   block (the "anti-slop floor" + the "brand personalization" tiers, D above).
2. Add a design-taste verdict schema to visual-ui-loop.md's critique (it already has pass|fail|unknown;
   add the taste-floor items as first-class checklist items with mandatory evidence regions).
3. Optionally: a small deterministic taste_check.py (font-present / single-accent / state-coverage /
   layout-tell heuristics) as the cheap Layer-2 leg, auditor-owned + frozen, mirroring
   maintainability_check.py.

================================================================================
CITATIONS (tier: 1=primary/official, 2=investor/quality-analysis, 3=third-party/community)
================================================================================
[2] Amplify Partners — "Kill the Slop: announcing our investment in Taste"
    https://www.amplifypartners.com/blog-posts/kill-the-slop-announcing-our-investment-in-taste
    (averaging problem, trust-propagation, preference/reasoning/rubrics/eval-environments, raise-floor-
     then-personalize, judgment-not-generation)
[3] Tanay Jaipuria — "RL Beyond the Verifiable"  https://www.tanayj.com/p/rl-beyond-the-verifiable
    (instance rubrics, programmatic checkability, rubric-as-reward, LLM-as-judge, generative/process
     reward models, explicit averaging-problem quote attributed to Taste Labs)
[2] Latitud — "The Taste Layer for the AI ecosystem"
    https://latitudlatam.substack.com/p/the-taste-layer-for-the-ai-ecosystem  (taste-layer framing;
     "measure, judge, codify, search over what great means"; feel-right vs correct)
[1] Taste Labs site  https://tastelabs.com/  (403 to bot; framing recovered via [2])
[3] VOIDXAI/taste (Claude Code taste skill)  https://github.com/VOIDXAI/taste
    (5-dimension judgment skill, design dimension territory, no-overall-score / evidence-or-skip,
     the four modes — directly reusable structure)
[1] Founder announce thread  https://x.com/thaiscbranco_/status/2066912871649574945
[2] video chapter list (user-supplied screenshot, Solo Founders podcast 2026-07-02): machines-learn-
    taste-differently 09:34, quirk-vs-signature 11:30, slop-cause-is-volume 13:09, why-a-lab 19:37,
    ~800 tastemaker community 25:17, critic-vs-maker 27:13, intent/interpretation/commissioning 30:04
BLOCKED (bot-forbidden, content recovered via above): forcreativegirls.com (403), solofounders.com
    blog (404), tastelabs.com/ (403). No live paywall bypass attempted.

CAVEATS
- Taste Labs publishes NO implementation deep-dive; the rubric-RL recipe (Part 2) is the INDUSTRY
  method they sell into (grounded via tanayj + Amplify), not a disclosed Taste Labs internal spec.
  Treat 2.1/2.2 as "the technique Taste Labs commercializes", high confidence on the technique,
  medium on it being verbatim their pipeline.
- The ~800 number and chapter concepts come from the podcast chapter list, not a transcript I could
  fetch (both video-adjacent pages blocked bots). Directionally reliable, not quote-exact.

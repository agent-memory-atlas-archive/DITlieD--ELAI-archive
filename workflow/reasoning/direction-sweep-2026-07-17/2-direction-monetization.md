ELAI DIRECTION + MONETIZATION — CORPUS SWEEP (sweep-2)
Authored 2026-07-17. Task: sweep-2-direction-monetization. Read-means-read sweep over the strategic-
direction and monetization corpus (2026-05-19 .. 2026-07-10), plus a forward trace from every strategic
commitment into the plan corpus and its live status. This file is intended to REPLACE reading the bundle.

Every claim below carries its source path. Tags used here:
  [DOC]        the source doc asserts it
  [VERIFIED]   i confirmed it against this tree this session (file existence / tool output)
  [UNVERIFIED] asserted in a source doc, not re-checked here
  [CONTRADICTION] two sources in the corpus disagree; both are presented

==============================================================================
0. WHAT WAS READ (the corpus, in date order)
==============================================================================
  2026-05-19  workflow/agency-strategy-2026-05-19.md                              (299 lines)
  2026-06-01  workflow/reasoning/ELAI-DIRECTION-2026-06-01.md                     (249)  ANCHOR
  2026-06-02  workflow/reasoning/ELAI-FOCUS-RESCOPE-2026-06-02.md                 (279)
  2026-06-02  workflow/reasoning/ELAI-TRUST-MAXIMIZATION-2026-06-02.md            (370)
  2026-06-02  workflow/reasoning/ELAI-GTM-PLAYBOOK-2026-06-02.md                  (194)
  2026-06-03  workflow/reasoning/ELAI-EFFICIENCY-TRUST-DIRECTION-2026-06-03.md    (780)
  2026-06-04  workflow/reasoning/ELAI-FUTUREPROOF-DIRECTION-2026-06-04.md         (201)
  2026-06-05  workflow/reasoning/ELAI-DESIGN-SELL-BY-2026-12.md                   (456)  (header says
              "sell by end of 2026"; the file's own authoring stamp is 2026-06-05, the filename says
              2026-12 = the DEADLINE not the date)
  2026-07-04  workflow/reasoning/ELAI-COST-MODEL-CONTEXT-PIPELINE-2026-07-04.md   (275)
  2026-07-05  workflow/reasoning/proof-spine-2026-07-05.md                        (60)
  2026-07-08  docs/strategy/ELAI-CLAIMS-POLICY.md                                 (94)
  2026-07-08  docs/strategy/ELAI-PROBLEM-QUANTIFICATION-DOSSIER.md                (167)
  2026-07-10  workflow/reasoning/elai-harness-market-evaluation-2026-07-10.md     (909)  NEWEST
  undated     workflow/ELAI-PITCH.txt                                             (50)
  plan units  docs/thefinalglance/ELAI-SPEC-OVERVIEW.md (592), K-SUMMARY_INDEX (600),
              G-SUMMARY_INDEX (225), M-SUMMARY_INDEX (146), K-7_appsec_insurance_positioning_brief.md (119)

==============================================================================
1. THE ONE-PARAGRAPH CURRENT-BEST DIRECTION (as the corpus stands 2026-07-17)
==============================================================================
ELAI is a native, model- and runtime-agnostic autonomous coding harness whose PRODUCT is the trust
kernel around the model, not the model: the harness owns state, side effects, gates, and the "done"
decision off evidence the generator did not author. It is sold on ONE measured, journal-recomputable
gate-ON-vs-gate-OFF number (SAFEDELTA), to developers/agencies/org-side budget owners who can action a
gated diff — never to non-coders, never on cost, never on benchmark rank, never as "perfect code". The
engine stays closed; ONLY the verifier + the evidence FORMAT open. As of 2026-07-10 the corpus's own
verdict is that the roadmap has outgrown the wedge and must SHRINK to a small permanent kernel (with
native ELAI still the flagship, wrapper as on-ramp), canonicalize (N), and prove (K/Q) before adding
surfaces. The headline number STILL DOES NOT EXIST [VERIFIED: PS-1 = RUNNER_PRESENT, not MEASURED].

==============================================================================
2. THE ANCHOR VERDICT (2026-06-01) — FULL REASONING, NOT THE SLOGAN
==============================================================================
Source: workflow/reasoning/ELAI-DIRECTION-2026-06-01.md
Provenance: 8 web-research streams + 6 adversarial refutations, ~243 fetches, all 2026-cited
(workflow run wf_e2948fab-3a8; raw output GC'd, this file is the durable digest).

EXECUTIVE VERDICT (verbatim, :12-13): "Adjust direction, do NOT pivot. The trust thesis is correct and
on-trend; what changes is the TARGET, the PILLAR RANKING, and the TIMELINE FRAMING."

The four moves (:15-22, verbatim):
  - KEEP: "trust / verification / grounded / deterministic spine. It is ELAI's deepest-built strength
    AND the single most buyer-cited gap in 2026."
  - DEMOTE: "'local-first saves money' and 'lower token usage' as MOATS (both refuted as durable
    differentiators). Reframe local-first around privacy + offline + no-rug-pull, not cost."
  - RE-AIM: "target from 'non-coders' to trust-demanding developers / prosumers / agencies /
    privacy-constrained teams. (Refuted that non-coders are a viable PRIMARY target.)"
  - RESET: "'releasable + catches a user wave in ~3 months' is not realistic for the full vision.
    A narrow, MEASURED trust-layer release in ~3 months is."

NORTH STAR (:24-26, verbatim): "an agent that cant lie to you and cant wreck your system." NOT "perfect
code" (hard no). "Trust = verified-before-you-see-it + honest-about-limits + physically-blocked-from-
catastrophic."

THE THREE KEEPABLE PROMISES (:134-142) — each is deliberately keepable where "perfect code" is not:
  1. Verified before you see it (deterministic checks + contract tests + grounding sweep already ran;
     the evidence is shown).
  2. Honest about what it couldn't verify (abstention over a confident wrong answer; UniCR/refusal
     calibration is the mechanism).
  3. Physically can't do the catastrophic thing (no rm -rf, no hallucinated package, no lying about a
     rollback) because it's enforced in code (FSM gates, PreToolUse path gating, irreversible-op
     tripwire, anti-slopsquatting), NOT a prompt the model can route around.

THE REFINED POSITIONING (:119-132), agreed after maintainer's pushback (i.e. this is NOT the research's
first answer — the research wanted a narrower "only grounding" product and was overruled):
  - ELAI is a trust-first Claude-Code-CLASS HARNESS, same category as Claude Code / Cursor, built around
    TRUST instead of speed. "Intelligence in the infrastructure, not the model."
  - Two modes: INTERACTIVE (user drives, approval gates on risky steps) and AUTONOMOUS (hand it a task,
    a whole SESSION works it end to end, returns a VERIFIED product OR an honest stop with exactly
    where/why it stuck). "Session" = the continuation of the whole task, NOT one chat context.
  - Local / hybrid / cloud as a config switch. Local (Qwen-class via the custom sidecar) by default;
    push the hard ~20% to cloud when wanted.
  - Usage reduction + predictable cost = strong SUPPORTING pillar + retention hook vs Claude Code cost
    pain. NOT the headline.
  - Not open source. One-click installer for the whole stack (compiled Rust exe + GPU sidecar).

PILLAR RANKING FOR MESSAGING (:150-151, verbatim): "lead TRUST (calibration + safety) > close with
PREDICTABLE COST + PRIVACY/no-rug-pull > demote raw token-savings and 'local saves money' to footnotes."

VIBESEC STANCE (:144-148): go heavy, but as PREVENTION-IN-THE-LOOP, not a bolt-on scanner. "The
differentiator is the deterministic gate blocking the vuln / exposed secret / hallucinated dependency
BEFORE it is written, inside the generation/autonomous loop." Scanner category is already funded
(Escape, Qodo, Semgrep AI) — do not ship a late scanner.

==============================================================================
3. THE REFUTATIONS (the load-bearing content — do not soften these)
==============================================================================
These are the corpus's most valuable output: each is a moat that was KILLED, with the evidence.

R-A. "LOCAL-FIRST SAVES MONEY" — REFUTED as a moat.
  Evidence (DIRECTION :55-58): frontier prices fell ~80% in a year; fixed-quality cost deflating
  50-200x/yr (Epoch AI). DeepSeek V4-Flash ~$0.14/$0.28 per Mtok. "Self-hosting only beats cloud above
  ~35M tokens/month (a non-coder never reaches it; the GPU becomes a net cost)."
  Reframe survives (:87-90): local-first HOLDS for privacy + vendor-independence ("no rug-pull") for
  regulated/privacy-constrained/prosumer audiences — validated by the Gemini CLI sunset, the Windsurf
  saga, the Cursor/Claude pricing blowups. FOCUS-RESCOPE :88-90 hardens it further: "Privacy floor
  only" — the local MODEL is a liability, not an asset.

R-B. "LOWER TOKEN USAGE" — REFUTED as a durable wedge.
  Evidence (DIRECTION :59-61): "Token efficiency is commoditized and copyable: 'Pi' already ships a
  sub-1,000-token system prompt (ELAI's ~330 is an increment, not structural); caching gives 88%,
  routing 60-80% to anyone. VentureBeat 2026: the moat moved to the application layer."
  FOCUS-RESCOPE :62-64 adds: "value deflates mechanically; Epoch AI inference price ~50x/yr drop.
  ELAI's ~330-tok prompt is self-reported and unmarketable closed-source. Keep plumbing, never pitch."
  Cost PAIN is still real as a RETENTION hook (:62-67): Microsoft revoked Claude Code access over cost
  (May 2026); Uber burned its AI-coding budget in 4 months; Claude Code's Mar-2026 throttling crisis
  (Max users hit limits in ~19 min) where "the silence is more damaging to trust than the bug"; Cursor's
  2025 metered-credit revolt; Gemini CLI's consumer path killed Jun 18 2026, a year after launch.

R-C. "NON-CODERS AS PRIMARY TARGET" — REFUTED (~85% confidence, DIRECTION :101-107).
  Reason: ELAI's own README says "deterministic development engine, not an assistant." Non-coder success
  on Lovable/Replit is driven by the HOSTED deploy+db+auth layer ELAI does not ship; "ELAI ends at
  'here's your audited code' exactly where the non-coder's real need (a deployed running app) begins."
  The installer fixes INSTALL friction but not (a) output usability, (b) the missing deploy layer.
  Market owned: $4.7B vibe market — Lovable ($6.6B), Replit ($9B), Bolt, v0, Base44.
  FOCUS-RESCOPE :51-53 sharpens the mechanism: "the non-coder cannot action a gated diff." That single
  sentence is the whole argument — ELAI's product output is a REFUSAL/flagged diff, which requires a
  reader who can act on it.

R-D. "WEAK LOCAL MODEL + HEAVY SCAFFOLDING = TRUSTWORTHY CODE" — SPLIT verdict (DIRECTION :91-99).
  Engine half HOLDS: scaffolding swings a FIXED model 9-36pp (Confucius/SWE-Dev/harness-engineering).
  "Non-coders rely on it unread" half REFUTED.
  CRITICAL NUANCE (verbatim :93-96): "the famous 'weak model + scaffold beats strong model' (Confucius)
  was Sonnet-vs-Opus, BOTH frontier, NEVER tested on a weak/open model. Under identical scaffolding
  (SWE-bench Pro) open/weak models trail frontier 12-27pp; a 7B caps ~23% vs 32B ~37%. Model capability
  dominates the weak>strong axis."
  ELAI's OWN baseline (:97): 72% pass@1 and a 100% code-API-hallucination rate on the small model.
  Conclusion: "scaffolding lifts the ceiling but cannot manufacture reasoning a 7B lacks."
  FOCUS-RESCOPE :72-74 confirms via a second route: "refuted by every standardized-scaffold board
  (Scale SEAL lists nothing under ~70B; even open-frontier ~456B trails Opus 4.8 ~10pp)."

R-E. "3 MONTHS TO CATCH A USER WAVE" — REFUTED (~85%, DIRECTION :109-114) for the FULL vision.
  Reasons: saturated free OSS field already delivers local-first/BYOK/MCP (Cline ~5M installs, Aider
  ~15B tok/wk, OpenHands 68k stars). ELAI debt understated: ~1,500+ production .unwrap() vs its own HARD
  rule, ~70 RUNPOD-PENDING items, ~15% wire-dark. "Distribution (not building) is the binding
  constraint. Velocity is high (413 commits/60 days) but velocity != distribution."
  A NARROW, measured trust-layer release IS achievable in the window.

R-F. "MAKE THE SMALL MODEL SMART" — REFUTED (EFFICIENCY-TRUST :26-30, C1 :76-84).
  SWE-Protege (arXiv:2602.22124) lifts a 7B to 42.4% SWE-bench Verified — but with an Opus-4.1 EXPERT
  (41.0% with a Sonnet-4.5 expert at 11.9% of tokens) AND SFT+RL on expert trajectories. "The gain is
  the expert's, not a free 7B's."
  SELF-CORRECTION ON RECORD (§16/E1, :765-768): the "vanilla 7B at 9.0%" baseline this argument was
  originally built on is FABRICATED — not in the paper; prior SLM SOTA is 17%. "The 'below the cascade
  floor' argument built on 9.0% collapses." The honest residue: "the cheap worker only saves cloud
  tokens on steps it can actually do, and the big measured gains are the cloud expert's."

R-G. "SAVE TOKENS/COST AS THE GOAL" — REFUTED as a goal (EFFICIENCY-TRUST :31-34).
  In CLOUD mode the ONLY real cost lever is prompt caching (41-80% [WEAK tag, see §16/E4]). Sidecar
  quant (AWQ/FP8) touches the local GPU only, ZERO effect on the Sonnet bill. The ~330-token prompt is
  noise.

R-H. "CONTEXT EFFICIENCY VIA LLM COMPACTION / BIGGER WINDOW" — REFUTED (:35-37).
  Deterministic observation masking halves cost while MATCHING LLM-summarization quality AND removes a
  hallucination surface (arXiv:2508.21433). Context-rot: a bigger window degrades, it does not help
  (Chroma Jul-2025: a 200K model degrades by ~50K).

R-I. "MULTI-AGENT FAN-OUT FOR CODE" — REFUTED (C5, :112-119). This one killed a move DIRECTION §7 was
  about to make (wire BSP for parallel code-writers).
  Science of Scaling (arXiv:2512.08296): negative returns from adding agents above ~45% single-agent
  accuracy; coordination cost superlinear. Single >= Multi at equal token budget (arXiv:2604.02460).
  Anthropic's own multi-agent post: the 90.2% gain was on RESEARCH, "not a fit for coding," 15x tokens.
  DECISION: do NOT wire BSP for parallel code generation; the right second cloud call is an
  INDEPENDENT-CONTEXT VERIFIER (Architect/Editor split, Aider, +9.6pp, positive across every pairing).

R-J. "ESCALATE TO A STRONGER MODEL TO STOP THE LYING" — REFUTED (C4, :101-111).
  Stronger models CHEAT MORE (ImpossibleBench arXiv:2510.20270: GPT-5 54% / o3 49% / Opus 4.1 50%
  cheat-rates; >79% via test-modification, operator-overloading dominant; LLM monitors catch only 42-50%
  on SWE-bench).
  Also dead: CoT monitoring (72% of reward-hacking episodes carry a confident CoT rationale,
  arXiv:2605.02964); logprob hallucination prediction (22-33% ceiling, Collu-Bench arXiv:2410.09997);
  verbalized confidence (near-zero skill). The one live signal is EXECUTION-AGREEMENT (CodeRefuser
  arXiv:2605.17029, 79.4% MBPP / 72% HumanEval precision vs 37-40% logprob).

R-K. "ELAI_SEED MAKES THE EVAL CREDIBLE" — REFUTED (:200-203).
  Cloud inference is non-deterministic; seed controls the harness, not the provider. Determinism
  certifies REPRODUCED, not RIGHT (DFAH arXiv:2601.15322: determinism and accuracy decoupled, r=-0.11,
  p=0.63; small models hit near-perfect determinism by rigid pattern-matching at 20-42% accuracy).
  Thinking Machines: cloud temp-0 = 80 unique completions / 1000. This kills any cloud "bit-identical
  replay" claim permanently.

R-L. "EU AI ACT ARTICLE 50 IS A GTM HOOK" — KILLED (GTM :142-146, verbatim heading "KILL the Article 50
  GTM hook").
  Commission draft Article-50 guidelines (2026-05-08, Bird & Bird) explicitly EXEMPT source-code outputs
  and NAME "agentic developer tools such as Claude Code, Cursor and Copilot" = ELAI's class. Use the
  Aug-2-2026 date as a time-boxed OUTREACH hook only, never the value prop.
  Penalty correction (binding, repeated in 3 docs): the Art-50/Art-12 tier is EUR 15M / 3% of turnover
  (Art-99). EUR 35M/7% is the Art-5 PROHIBITED-PRACTICES tier — do NOT pair it with a logging/marking
  argument. (TRUST-MAX :290-292; FOCUS-RESCOPE :134-137; G-8 :154.)
  The DURABLE compliance trigger instead: in-toto/SLSA provenance (CRA / EO-14028 buyers already request
  attestations in procurement) — ELAI's signed RRA already targets it (GTM :145-146).
  [CONTRADICTION, minor] EFFICIENCY-TRUST :206-207 states Article-50 "Governs synthetic media output,
  not source-code commits. Dates UNVERIFIED-pending per the E-38 HARD RULE" — same conclusion, weaker
  grounding claim. G-8/K-6 both preserve the SURVIVING form: ship attestation as "designed to support
  Article-50 marking", NEVER "certified compliant". K-7 encodes it as a stub_ban ("NO Article-50-
  certified claim (GTM-killed; 'designed to support' only)", K-7 :62) and a deterministic grep check
  (K-7 :78: `rg -c "Article.50.certified|certified compliant" ... MUST be 0`).

R-M. "TRUST ALONE IS A WEDGE" — REFUTED (DIRECTION :72-78). Trust is REAL and buyer-validated but now
  TABLE-STAKES at incumbents (Claude Code ships hooks = deterministic gates, isolated subagents, a
  fresh-model verification subagent = ELAI's auditor) AND already a funded category (Qodo $70M/$120M,
  sold as a verification LAYER). "ELAI's defensible whitespace is the COMBINATION: autonomy + trust +
  local-first, aimed at people who feel the pain."

R-N. "SELF-PUBLISHED NUMBER" — anti-credibility for this publisher profile (GTM :26-30).
  Devin: Cognition published 13.86% SWE-bench + slick demo + NO reproduction path -> the community
  "canceled" it after Carl Brown's Internet of Bugs teardown. Kimi K2: self-reported 50% vs 29.4%
  independent on Humanity's Last Exam (~21pt gap, "widens for low-accountability publishers").
  "ELAI is the maximally-untrusted publisher profile."

R-O. THE THREE FIGURES REFUTED INSIDE ELAI'S OWN ARTIFACTS (EFFICIENCY-TRUST §16 ERRATA :761-779, and
  :38-41). This is the most uncomfortable and most important row in the corpus:
  "the research caught a HALLUCINATED number already living inside ELAI's own memory (the '92.5 LoCoMo'
  Mem0 figure; real J-score is 66.88%), plus two prior bad arXiv cites. A 'cant-lie' product that ships
  unverified claims IS the liar it sells a cure for."
  E1 SWE-Protege "9.0% vanilla 7B" = FABRICATED. E2 RouteLLM "~85% cost cut" = the LMSYS BLOG figure;
  paper body (Table 6) is 3.66x / ~73% at 95% GPT-4 quality on MT-Bench. E3 "replace LLM-compaction"
  = there IS no LLM compaction in ELAI to replace (compact_session is a deterministic string formatter).
  E4 caching "41-80%" survives but is tagged [WEAK] (cross-provider pooled, DeepResearch-Bench preprint,
  prefix-heavy workload). E6 Anthropic context editing date is 2025-09-29, not Jan 2026.
  This ERRATA section is the direct ancestor of KD-1 / the claims policy (§9 below).

==============================================================================
4. THE CEDE / SUPPORTING / CONTEST / WIN LEDGER (2026-06-02, the sharpest single table)
==============================================================================
Source: workflow/reasoning/ELAI-FOCUS-RESCOPE-2026-06-02.md (run wf_f0ba2d1a-31c: 15 competitive areas,
33 agents, ~527 web fetches, every area adversarially fact-checked, all claims 2026-cited; all
confidence HIGH).

EXECUTIVE VERDICT (:12-14, verbatim): "Cede the everything-machine: ONE-SIDED yes. Own the trust wedge:
CONDITIONAL yes, and the condition IS the message. The focus is correct but it is not a safe harbor; it
is a race against a ~2-3 quarter window to ship ONE measured number, against incumbents already
shipping pieces of the same idea."

WIN: EMPTY (:116, verbatim): "No WIN claim anywhere until SAFEDELTA ships." This is the single most
important line in the corpus and it is still true as of 2026-07-17 [VERIFIED].

The per-area table (:259-275, all HIGH confidence):
  token-efficiency .............. CEDE       (commoditized; price deflates ~50x/yr)
  local-first ................... SUPPORTING (real demand, model is a liability; privacy floor)
  cloud-local-hybrid ............ SUPPORTING (commoditized; OpenRouter $1.3B won the gateway)
  rag-retrieval ................. CONTEST*   (right architecture, no commercial ground; *treat SUPPORTING)
  context-management ............ SUPPORTING (every primitive owned; only edge is the trust property)
  reasoning-verification ........ CONTEST    (the 2026 differentiator; needs a number)
  local-as-flagship ............. SUPPORTING (strong claim refuted; scaffold can't fix a stale 7B)
  vibesec ....................... CONTEST    (scanner = cede; pre-write fail-closed = the angle)
  website-app-creation .......... CEDE       (Lovable $6.6B/$400M ARR, Replit $9B; hosted runtime moat)
  tool-creation ................. SUPPORTING (Anthropic/Smithery own it)
  self-improvement .............. CONTEST    (guarded self-mod unoccupied but ELAI's substrate is inert)
  vibe-coder-first .............. CEDE       (non-coder can't action a gated diff; needs hosted deploy)
  autonomous-mode ............... CONTEST    (capability ceded; "autonomy you can trust" axis open)
  trust-verification-category ... CONTEST    (name claimed by Qodo; runtime bundle not yet assembled)
  distribution-gtm .............. SUPPORTING (worst-positioned; only a security/benchmark content wedge)

"Cede" is precisely scoped (:28-30, verbatim): "do not compete on it, do not headline it, do not spend
scarce solo-dev time trying to beat the incumbent there. It does NOT mean delete the feature."

THE WEDGE SENTENCE (:35-38, verbatim, the most-quoted line in the corpus):
"The coding harness whose autonomous runs are TRUST-EVIDENCED: it physically can't wreck your machine,
can't fabricate a passing test, and stops honestly when stuck; wrapped around a competent CLOUD backbone
(Sonnet), sold at ~$39-49/seat to the regulated/enterprise-governance buyer who can action a gated diff,
proven by a published gate-ON-vs-gate-OFF safety-delta number."

THE PRECISE CORRECTION TO "MAKE THE LLM HONEST" (:40-45, verbatim): "you cannot change the model's
probabilistic black-box nature. You build the SYSTEM around it that refuses to lie and refuses to wreck
(verify before the user sees it, block the catastrophic action deterministically, detect a fabricated
test, abstain when it cannot verify). The pitch is 'we catch the LLM when it lies and stop it before it
wrecks, and tell you honestly what it couldn't verify' — NEVER 'perfect code.' This is what burned Devin
(15-30% real success vs 70%+ claimed; Answer.AI 3/20) and Replit."

THE TWO GENUINELY-SHARPER ATOMS, WITH THEIR CAVEATS (:124-130) — the honest core of the moat:
  - Cryptographic anti-fabrication binding (a claimed PASS bound to a captured-stdout blake3 hash).
    Caveat: "no head-to-head proves it superior; EviBound already hits 0% so the idea is public; only
    ELAI's implementation is not; the binding is ~50 lines any vendor can replicate."
  - Deterministic post-shell-EXPANSION FS gate + egress IP-pin, fail-closed. Caveat: "Snyk Agent Guard +
    MS Toolkit already ship in-loop deterministic blocking; honest claim is 'post-shell-expansion path
    resolution as default-on,' not 'deterministic blocking is novel.'"
  Framing (:121-122): "The moat is the INTEGRATION, not any single primitive, and integration is the
  weakest kind of moat against a funded incumbent."

==============================================================================
5. THE FIVE HARD TRUTHS (FOCUS-RESCOPE §5) — the corpus's own kill-list against itself
==============================================================================
Source: ELAI-FOCUS-RESCOPE-2026-06-02.md :178-211. These are stated as preconditions for shipping.

  5.1 "'Built' is not 'shipped, wired, measured.'" ~15% of the trust substrate is WIRE-DARK; ~1500
      production .unwrap() debt; the DGM/self-mod pillar is inert. "A wire-dark gate has never blocked a
      real fabrication or rm -rf in production -> every 'it can't' claim is currently a DESIGN
      ASSERTION, not demonstrated behavior."
  5.2 THE ONE REAL NUMBER: SAFEDELTA gate-ON-vs-OFF. Control = gate-OFF on the SAME model/tasks/harness.
      "Reproducible journal is NECESSARY but NOT SUFFICIENT for trust: a self-published solo number is
      structurally discounted vs Endor/Macroscope third-party leaderboards. OPEN: no verified path by
      which a solo closed-source number earns third-party trust." (-> the 2026-06-02 GTM run answers it)
  5.3 MODEL STALENESS + ABLITERATION. Qwen2.5-Coder 3B/7B is near-bottom AND abliterated (safety-tuning
      removed) -> "a fail-closed gate must reject MORE, partly cancelling the generator it guards." FIX:
      run the wedge + SAFEDELTA with gates wrapping CLOUD SONNET. "ELAI is a trust WRAPPER around a
      competent backbone, not a standalone agent on a stale 7B. Local 7B = privacy floor."
  5.4 HARNESS-OVERFIT DECAY. Gates are tuned to CURRENT weak-model failure modes; "as backbones
      fabricate/wreck less (Sonnet 4.5 already 0% on reward-hacking bench) pure catch-rate value
      shrinks. OPEN: no measured data on gate-value persistence. HEDGE: position gates as a COMPLIANCE/
      ATTESTATION substrate whose value does NOT decay, not purely catch-rate which does."
  5.5 SOLO-CLOSED-SOURCE RUG-PULL CONTRADICTION. "A user fleeing Cursor/Windsurf/Replit rug-pulls who
      lands on a solo, closed-source, bus-factor-1 project reproduces the single point of failure they
      fled. 'No-rug-pull' is self-undermining until ELAI ships a credibility mechanism."
  5.6 PRICING/BUSINESS MODEL. Buyer = the dev/team who can action a gated diff AND holds budget. Band
      $39-49/seat. "OPEN: solo-dev effort/sequencing to clear (activate one gate) + (SAFEDELTA on cloud
      Sonnet vs BaxBench) before incumbents bundle = the project's largest unquantified risk."

THE TWO ROOT CONTRADICTIONS named (:139-144, :200-204):
  - ANTHROPIC DEPENDENCY ("FOUNDATIONAL CONTRADICTION"): "ELAI rides Anthropic's /v1/messages protocol
    and runs cloud Sonnet under its own gates -> ELAI's 'trust/no-rug-pull' pitch is structurally
    dependent on the single vendor most able to (a) absorb the differentiator natively and (b) rug-pull
    it via protocol/pricing. Root contradiction, not one competitor row."
  - SOLO BUS-FACTOR (5.5 above).
  Both are the explicit target of the 2026-06-04/05 control-plane repositioning (§7 below).

THE WINDOW (:160-162, verbatim): "optimistically 2-3 quarters; for several sub-pillars (MS Toolkit, Snyk
Agent Guard, OpenAI Harness Engineering, EviBound, Endor leaderboard) effectively closed now. Treat 2-3
quarters as the OUTER bound." Authored 2026-06-02 -> the outer bound expires ~2026-12. As of 2026-07-17
that is ~1.5 quarters remaining and the WIN column is still empty.

==============================================================================
6. THE OPEN-CORE DECISION (exact line, and the argument for it)
==============================================================================
Source: ELAI-GTM-PLAYBOOK-2026-06-02.md §2 (:64-91). This is the corpus's single clearest decision.

THE DECISION (:66-68, verbatim): "NO on the gate engine + autonomy harness (stay closed). YES on the
SAFEDELTA verifier / RRA replay tool + in-toto attestation schema (Apache-2.0) — the few-hundred-LOC
piece a skeptic needs to re-derive the gate-ON/OFF delta from a signed artifact. Engine stays a black
box; the SCORE becomes independently reproducible."

WHY THE LINE IS DRAWN THERE (:70-75):
  - Sourcegraph Cody = the direct analogue (open Apache-2.0 client 2023, closed engine).
  - curl / Daniel Stenberg "verification over trust" (signed release artifacts + public verify page,
    InfoQ 2026-05) = "a literal working precedent AND the marketing line in one."
  - arXiv:2506.23706 Attestable Audits (publish your own safety benchmark, believed without trusting the
    reporter) — BUT the TEE wrapper is OPTIONAL/DEFERRED (SGX/SEV burden impractical for a solo; ship
    signed-artifact + open-replay first; add TEE only if a regulated buyer demands it).

WHY NOT THE ENGINE (:77-81, and note the CORRECTION): "commoditized to $0 by Microsoft Agent Governance
Toolkit (2026-04-02, MIT, all 10 OWASP agentic risks, <0.1ms p99, full 7-component suite). Open-sourcing
the engine = competing with a free MS-backed incumbent for zero moat."
  CORRECTION preserved verbatim in the doc: "the AVOID reason is the free MS incumbent — NOT the
  OpenTofu/Valkey/OpenSearch forks, which prove 'relicensing-away invites a fork,' a different thing.
  Also Tabby is itself open-core, NOT pure-permissive — don't cite it."

THE FSL MIDDLE OPTION (:83-85): VIABLE, not recommended. Release the gate-policy engine under FSL-1.1
(2-yr non-compete then auto-converts to Apache/MIT — "a genuine rug-pull antidote") but it is
source-available not OSI-open, forfeits HN goodwill, and "gives away the component buyers most want to
audit against the free MS incumbent. Prefer verifier-open."

WHAT IT BUYS / DOES NOT BUY (:87-90): buys the ONE number becoming independently reproducible + the curl
"verify don't trust" narrative + near-zero engine leakage. Does NOT buy virality, and does NOT buy
bus-factor cover — "CONTINUITY is solved by ESCROW, not by open-sourcing anything."
BONUS (:91): "ELAI already plans the Reproducible-Run Artifact (G-2) + in-toto attestation (G-8) — the
verifier-open move is nearly free, it's G-2 + G-8 published."

THE THREE-PROBLEM REFRAME the playbook opens with (:11-18, verbatim):
  - BELIEVABILITY: "make it re-runnable by a skeptic in one command on a harness they already trust.
    NOT self-publish; NOT an Endor leaderboard slot (you can't get one)."
  - CONTINUITY: "bus-factor-1 = ELAI is itself a rug-pull risk: source-code escrow + open-source ONLY
    the verifier. NOT open-source the engine (Microsoft already gave that away free)."
  - DISTRIBUTION: "lead with vibe-code-cleanup AGENCIES as first paying buyer. NOT regulated enterprise
    (the late buyer that structurally caps a solo); NOT paid newsletters."

REALISTIC CEILING (:20-22, verbatim): "a PROFITABLE SOLO/SMALL product serving agencies + prosumers,
with regulated enterprise as a slow escrow-gated maybe-never-at-volume upside. This is a path to first
revenue, NOT a venture outcome — and it matches maintainer's stated goal (a better life, make some money)."

LATER EVOLUTION (M-plan, 2026-07-02/05): the verifier-open decision is carried into M-5a (standalone
`elai-verify` OSS crate with ZERO engine deps — "the extraction is a compile-time boundary proof; engine
stays closed (GTM decision executed)", M-SUMMARY :93-101), then EXTENDED by M-6 (2026-07-05): open the
evidence FORMAT itself ("EEF v1"), so third-party tools emit ELAI-verifiable evidence and elai-verify
becomes "the reference verifier of a FORMAT" (M-SUMMARY :112-133). The M-6 rationale is verbatim: "a solo
author cannot win a plugin-ecosystem war but can own the FORMAT agent evidence is exchanged in."
M-6's "open standard" wording is itself KD-1-gated on >=1 EXTERNAL emitter ("until then: 'published
format'"). [STATUS: M-5/M-6 unbuilt — see §11.]

==============================================================================
7. THE 2026-06-04/05 REPOSITIONING — CONTROL PLANE / MCP-SERVER INVERSION
==============================================================================
Sources: ELAI-FUTUREPROOF-DIRECTION-2026-06-04.md; ELAI-DESIGN-SELL-BY-2026-12.md.

THE REPOSITION (FUTUREPROOF :16-20, verbatim): "ELAI = the trust / verification CONTROL PLANE for the
agent era — the external, deterministic, off-agent reference that breaks the circular-AI-review problem,
exposed over the open agent protocols so it gates Claude Code, Cursor, Copilot, and fleets of background
agents instead of competing with them for the same seat."

Why it was proposed (FUTUREPROOF :21-36): it answers BOTH "better than them" and "future-proof" at once,
and "dissolves the two root contradictions: model-agnostic-by-construction kills Anthropic-dependency
(you gate ANY agent), and 'verifier that plugs into existing tools' has a far lower adoption barrier than
'switch IDEs,' which softens bus-factor."
Grounding: verification is the named 2026 bottleneck (Sonar 96% distrust; 24% of work-week on review;
Qodo $70M; Fortune "trust is the real bottleneck"); AI-reviewing-AI is provably circular without an
external reference (arXiv:2603.25773, SRLabs); MCP is the de-facto tool standard (Linux Foundation,
~18k servers); A2A the inter-agent standard; "guardian agent" is a named top-5 2026 trend.

DESIGN-SELL-BY (2026-06-05) makes it the delivery vehicle, explicitly NOT a replacement (:47-49): "This
does NOT replace the wedge — it is the DELIVERY VEHICLE for it. The three keepable promises (cant-lie /
cant-wreck / honest-stop) are still the product; the control plane is how they reach users without a
switch-cost and without depending on a single model vendor's goodwill."

DESIGN-SELL-BY's KEEP/CEDE/NEVER (:51-56, verbatim, "do not relitigate"):
  KEEP: trust/verification/determinism spine; prevention-in-the-loop VibeSec; autonomy-you-can-trust;
        predictable cost + privacy/no-rug-pull as SUPPORTING.
  CEDE: website/app builder; non-coder primary; raw token-efficiency + inline completion; standalone
        routing; skill marketplace; "local-as-flagship." Plumbing stays; never headline.
  NEVER: promise perfect code; claim benchmark wins (SWE-bench Verified is dead); lead on cost.

DESIGN-SELL-BY'S CRITICAL PATH (:364-371, verbatim): "A1 -> B1/B2 -> C1 (the number) -> C2 (demo) + D1
(MCP server) + D2 (RRA). Everything else is hardening."
Its most operationally-valuable correction (2026-06-05, :156-188): THE NUMBER NEEDS NO LOCAL GPU.
  - TIER-1 STATIC [runnable NOW, ZERO infra]: seccodeplt_runner drives the REAL VibeGate over an
    embedded SeCodePLT CWE corpus -> insecure_rate_before/after + gate_recall. "NO model, NO sandbox,
    NO GPU, NO cloud. This is the FIRST publishable number... do this BEFORE Tier-2 to kill 'we have
    zero number' risk on day one."
  - TIER-2 DYNAMIC [the real SAFEDELTA]: needs cloud API budget + a LINUX CPU host with the sandbox
    ENABLED (Firecracker/seccomp — NOT Docker; Appendix B :429-454 verified ELAI's sandbox is its own
    multi-backend, "zero matches for docker/containerd/oci/podman in the Rust code").
  - "The 2x3090 is OFF the critical path" (:370) — only for the optional SECONDARY local-lane number.
  Cost note (:186-188): "~N hundred-to-thousand $ of cloud spend (4 suites x 2 arms x bootstrap
  repeats). Bound it with ELAI's own cost gate and PUBLISH the bill ('SAFEDELTA cost $X, reproducible')
  — on-thesis, not a problem."
  This DIRECTLY CONTRADICTS DIRECTION-2026-06-01 R1 (:180: "Needs the 2x3090 box"). The 06-05 correction
  is the better-grounded one (it is code-verified in its own Appendix B) and it is 5 weeks newer.
  [CONTRADICTION resolved in favor of 06-05.]

DESIGN-SELL-BY'S THREE CODE-GROUNDED CONTRADICTIONS against the F/G/H plans (Appendix A, :400-426) —
each SHRINKS scope and each was found by reading source, so they outrank the plan text:
  1. B1/F-4: TestOutcomePayload ALREADY HAS `stdout_hash: Option<String>` (ssb.rs:10494-10504, blake3,
     since SCHEMA 197). "=> F-4 gap is ENFORCEMENT, not the field."
  2. E1/H-9: the CloudSpend emit block EXISTS (grounding_client.rs:915-960), contradicting H-9's "emit
     is gated dark"; the open question is only whether the prod ctor attaches .with_ssb_writer.
  3. F1/H-7: RuntimeRoutingPolicy::evaluate DOES change the provider (escalation_policy.rs:261),
     contradicting H-7's "doubly inert / observational" claim. The real gap is the architect/editor
     split, not the routing.
  NET (:422-426): "more trust + routing substrate is LIVE than F/G/H assumed... The repositioning is
  cheaper than it looks because the gates already exist — they just are not yet behind one seam or
  exposed over a protocol."

==============================================================================
8. THE NEWEST DOC (2026-07-10) — WHAT IT REVISES
==============================================================================
Source: workflow/reasoning/elai-harness-market-evaluation-2026-07-10.md (909 lines, AS-SPECIFIED design
evaluation + a verbatim external market dossier + a self-correcting addendum).

ITS VERDICT (:9-15):
  - "No: it is not defensible to call ELAI the best harness overall, or the best harness currently." No
    controlled public benchmark holds model/task/repo/compute constant across the field.
  - Defensible narrower claim: "as specified, ELAI is one of the most complete documented designs
    reviewed for trust-evidenced autonomous execution."
  - "ELAI is strongly aligned with where the market is moving." (12-point convergence list, :204-217.)
  - "ELAI is not literally future-proof. No harness is." And the killer line (:15): "The best future
    version of ELAI is therefore SMALLER than the current plan corpus."
  The honest public wording it prescribes (:572-574): "ELAI is designed to maximize independently
  verifiable trust per dollar. Its superiority is benchmark-pending."

WHAT IT ADDS THAT THE JUNE DOCS DO NOT HAVE:
  a) A NEW STRATEGIC RISK: breadth. "The corpus has accumulated overlapping routers, claim types,
     provenance scales, context systems, gates, and product verticals faster than it has canonicalized
     them" (:15). N-plan's own confession is cited as evidence (:243-247): four claim types, ten routers,
     six intensity deciders, five provenance scales, four counterfactual engines, five context cores.
     "N should be a prerequisite, not cleanup after J-M."
  b) A DIRECT ATTACK ON "EVERYTHING FAILS CLOSED" (:249-259). "The doctrine should not be 'everything
     fails closed.'" High-risk/irreversible/security/privacy/finance/credential/publication boundaries
     fail closed; "quality and productivity checks can quarantine, retry, request evidence, or remain
     advisory according to measured risk." Grounded in OpenAI's harness report: for high-throughput,
     low-risk changes, waiting costs more than cheap correction. "ELAI's K-5 risk tiers should be
     foundational."
  c) THE MARKET SIGNAL THAT CUTS AGAINST ELAI'S FSM (:228-240): "The strongest 2026 product signals
     favor more capable agents inside simpler, objective-oriented harnesses" — Symphony says rigid
     state-machine nodes did not age well; Kilo REMOVED its dedicated Orchestrator; mini-SWE-agent
     deliberately shrank the scaffold; Hermes keeps a narrow waist. The prescribed form: "strict at
     boundaries... flexible inside the boundary."
  d) SIX P0 TRUST-CRITICAL DESIGN FLAWS (:292-355) — design defects, not not-built-yet:
     P0-1 A journal failure cannot be allowed to disappear (B-2 lets an SSB write failure not block an
          FSM transition; that breaks replay + completeness -> transactional outbox or fail/quarantine).
     P0-2 G-3's shell-expansion plan is unsafe AND technically wrong: "std::fs::canonicalize ... resolves
          symlinks and filesystem paths; it does not perform shell tilde or glob expansion." Needs a
          side-effect-free shell AST evaluator + explicit rejection of command substitution + an
          OS-enforced boundary. (This directly undercuts the "can't wreck" demo's centerpiece.)
     P0-3 "A Merkle root is not a signature." G-8's local BLAKE3 Merkle default is integrity, not
          authenticity — "an attacker who changes the artifact can recompute it." Use Ed25519, bind
          signer identity + gate-manifest version, externally checkpoint.
     P0-4 The full receipt contract is fragmented rather than canonical (F-4 stdout_hash + G-2 pins +
          H-25 provenance + I-18 chain + I-45 objective hash live in five places). Needs ONE canonical
          TestExecutionReceipt binding 13 named fields (:317-329).
     P0-5 "`elai wrap` is bypassable without whole-process confinement." MCP interception alone is not a
          security boundary.
     P0-6 The determinism claim needs one normative matrix (a 5-row honest-claim table, :345-351).
  e) THE MOVE/REMOVE LISTS (§7.3/§7.4, :443-476). MOVE OUT of the trusted core into signed packs:
     autonomous trading H-17..H-29; app-design canvas; voice; personal research O; personal app world P;
     broad connector UX; domain formal-verification packs; provider-specific inference optimizations;
     company billing/commerce. REMOVE from the active core roadmap (:461-474): Empire integration
     B-25/C-34; "the claim that a local Merkle proof is a signature"; universal fail-closed for low-risk
     signals; any blocking role for the 27.9%-precision semantic-conflict detector; global fixed refusal
     bands; lexical-diversity-as-rationale-quality; one shared calibration threshold across unrelated
     consumers; prompt tagging as an enforcement boundary; PUCT/tree search until receipt-ranked
     best-of-N wins materially; diffusion drafting; GRPO/self-training; duplicate routers/claim types/
     provenance scales/context cores. Plus (:476): "'Deferred' items still impose reading and
     coordination cost."
  f) THE PRESCRIBED TOURNAMENT (§9, :526-570) with PRE-REGISTERED KILL RULES — the most concrete
     falsification protocol in the corpus: if a gate does not improve escaped defects beyond noise and
     adds material friction, demote/remove it; if a specialist role does not change decisions, merge it;
     if tree search does not beat best-of-N per dollar, do not build the next rung; if a local lane
     cannot reach a minimum verified-outcome rate, restrict it to mechanical work; if `elai wrap` cannot
     prove complete mediation, narrow its security claim.

IT REVERSES ITSELF, IN-FILE (the Addendum, :803-909). This is important and easy to miss:
  Sections 7.3/8/11 recommend WRAPPER-FIRST ("Strategy B", :496-524: make `elai wrap` + the evidence
  standard the FIRST product, move research/trading/voice/canvas/personal-world into optional packs).
  The Addendum (dated the same day, marked "corrective product-strategy addendum ... supersedes the
  wrapper-first recommendation in Sections 7.3, 8, and 11 where they conflict") CORRECTS it:
    - "I-40 was interpreted too broadly" (:814-824). The actual I-40 plan text says: "wrapper = wedge +
      standard track"; "the deep ELAI orchestrator with full wiring stays the premium destination"; "The
      wrapper is the on-ramp, not the product" (I-40_trust_layer_wrapper_rra_spec.md:34-35).
    - "A compact trust kernel is not a wrapper-first product" (:832-842): the earlier recommendation
      collapsed three INDEPENDENT decisions — small TCB, exportable trust protocol, and product identity.
      "The first two do not require surrendering the third."
    - Why wrapper-only discards the advantage (:844-857): a proxy does not own the foreign agent's
      reasoning/grounding protocol, decomposition, context allocation, branch selection, recovery, or
      verifier/worker separation. And P0-5 already shows the enforcement limit.
    - CORRECTED STRATEGY (:869-878): "kernel-first engineering, native-first product identity,
      wrapper-assisted distribution" — 1 compact trust kernel, 2 native ELAI flagship, 3 wrapper+adapters,
      4 signed capability packs.
    - CORRECTED FINAL ANSWER (:907-909): "ELAI should be a native, model- and runtime-agnostic autonomous
      harness with an exportable trust kernel, not a trust wrapper with an optional native harness."
    - AND it names the experiment that should decide it rather than analogy (:882-905): a three-arm
      comparison (competitor unmodified / competitor under `elai wrap` / native ELAI, same model).
  READING RULE: cite the ADDENDUM, not §7.3/§8/§11, for product strategy. The body's benchmark caution,
  P0 findings, canonicalization requirement, risk-tier recommendation, and pack argument all STAND.

==============================================================================
9. THE CLAIMS BAR + THE PROBLEM SIZING (what any monetization argument must clear)
==============================================================================
Source: docs/strategy/ELAI-CLAIMS-POLICY.md (NORMATIVE, K-1 WAVE KW-1);
        docs/strategy/ELAI-PROBLEM-QUANTIFICATION-DOSSIER.md (SHELL AUTHORED 2026-07-08).

KD-1 (the rule, CLAIMS-POLICY :9-27): no externally-visible claim ships with a number or a superlative
unless it carries one of [EXTERNAL] / [EXTERNAL, unverified] / [DESIGN] / [MEASURED]-with-a-recompute-
pointer / PENDING-naming-the-exact-settling-command. Anything else "is UNGROUNDED marketing copy and is
BLOCKed by .claude/tools/claims_policy_lint.py before it ships."
  Binds: README, docs/strategy/ positioning copy, marketing briefs, a spec site. Does NOT bind internal
  reasoning docs (so this sweep file is not lint-bound, but its claims still carry sources).
  Enforcement: AUDITOR-applied only, never main-LLM self-run (:54-65). Exit 0 pass / 1 BLOCK / 2
  scan-config error, fail-closed.
  Origin (:44-52, verbatim): "The B-40 / B-41 / B-42 bad-cite incidents ... shipped numbers into review/
  positioning copy without a checkable source." (This is the direct descendant of EFFICIENCY-TRUST's
  §16 ERRATA — R-O above.)
  The honest-wording rule for reproducibility (:29-42): I-43's table is normative and CITED not restated;
  when cross-lane action-stream divergence on security-critical paths exceeds I-43's 5% threshold
  [DESIGN], "the honest word for the cloud lane is 'trace-authentication', never 'reproducible' or
  'replay'."
  Lint tool [VERIFIED PRESENT]: .claude/tools/claims_policy_lint.py exists on this tree.

THE PROBLEM SIZING (the dossier) — every ELAI answer row is PENDING. This is the sharpest single fact
about the monetization story as of 2026-07-08:
  Row 1 Package hallucination. EXTERNAL: >=5.2% (commercial) / 21.7% (open-source) hallucination rate
        across 576,000 samples / 16 LLMs; 205,474 unique hallucinated names; 58% recur across 10 repeated
        queries, 43% recur in ALL 10. [EXTERNAL] KW-1 Spracklen et al., USENIX Security 2025.
        ELAI's answer: PENDING, and UNSCOPED — "no such bench exists on this tree as of 2026-07-08
        (grep-verified)". The Anti-Slopsquatting protocol is a PREVENTION control, not a MEASURED rate.
  Row 2 Insecure patches from autonomous agents. EXTERNAL: a standalone Llama-3.3-70B introduced 135
        verified new vulnerabilities vs 12 in the corresponding developer patches across 20,000+
        SWE-bench issues (11x); novel CWE-95 (eval-injection) + CWE-502 (deserialization) classes;
        agentic frameworks worst under the most-autonomy framework. [EXTERNAL] KW-2 arXiv:2507.02976.
        ELAI's answer: PENDING (A-2, F-2b SeCodePLT). Substrate present, live run blocked on RunPod GPU
        (W-10). Settling command: `elai bench seccodeplt --suite <fixture> --json`.
  Row 3 Self-improvement reward hacking. EXTERNAL: 73.8% of Kernel-Bench and 46.8% of ALE-Bench
        optimizations show proxy gains with no real-task gains; the proxy-reality gap widens 26.4% ->
        57.8% between 10 and 100 optimization steps. [EXTERNAL] KW-3, OpenReview ikrQWGgxYg.
        ELAI's answer: PENDING (A-3, G-6 SAFEDELTA). "the measured delta itself has not landed on this
        tree as of 2026-07-08 (grep-verified: no SafeDeltaMeasured SSB row present)."
  Row 4 Orchestration overhead. EXTERNAL: collapsing a multi-turn tool-call loop into one executed code
        block cuts end-to-end latency ~50% and token usage >60%; "many agents are bottlenecked by
        orchestration overhead, not model quality." [EXTERNAL] KW-4 Microsoft Agent Framework CodeAct,
        BUILD 2026. ELAI's answer: PENDING (K-4 gate-tax); "no `elai bench` subcommand exists for this
        yet" [as of 2026-07-08 — NOW SUPERSEDED, see §11: bench_gate_tax.rs exists as of 2026-07-17].
  Row 5 Visible-suite vs hidden-suite gap. EXTERNAL: SpecBench — "The gap also scales sharply with task
        length: it grows by 28 percentage points for every tenfold increase in code size" (abstract,
        re-verified by WebFetch 2026-07-08). [EXTERNAL] KW-5 arXiv:2605.21384. ELAI: N/A (grounds K's
        "visible-pass != trust" thesis).
  Row 6 EXCLUDED pending re-verification (KW-6): DORA 2026, GitClear duplication trend, METR
        (benchmark-passing PRs not merged), vendor telemetry (Uplevel, CodeRabbit) — "Perplexity-report-
        sourced and NOT yet primary-source-confirmed... these numbers do NOT appear as asserted figures
        anywhere." This exclusion row is itself a model of the discipline.

==============================================================================
10. BUYER / ICP / PRICING — EVERY CONCRETE FIGURE, WITH SOURCE AND CONFIDENCE
==============================================================================
PRICE BAND $39-49/seat.
  Anchor (FOCUS-RESCOPE :54): "Price anchor (documented WTP for the trust/governance line): Copilot
  Enterprise $39, Cursor Business $40, Tabnine $39 per seat." Confidence HIGH per that run's tagging.
  GTM :105: "$39-49/seat fits agency per-engineer spend."
  GTM/F-RESCOPE :134: "$39-49 is deliberately below the [procurement] line" — an Atlassian-style
  no-sales-team, price-below-procurement-threshold motion.
  Its live status: M-5d makes tiers.toml the ONE pricing-truth source and tags "GTM $39-49 seat framing
  ... [EXTERNAL-GTM] until K-24's measured number (MD-5)" (M-SUMMARY :100-101). i.e. the corpus itself
  refuses to assert the price as validated. [VERIFIED: tiers.toml does not exist on this tree.]

THE FIRST PAYING BUYER: VIBE-CODE-CLEANUP AGENCIES (GTM §3, :94-106). The strongest buyer signal:
  - "AI-attributed CVEs surged 6 -> 15 -> 35 across Jan-Mar 2026 (CSA Labs / Georgia Tech Vibe Security
    Radar)."
  - "45% of AI-generated code carries an OWASP Top-10 vuln (Veracode, 100+ models); AI devs introduce
    findings at ~10x the rate."
  - "~$100-300/hr cleanup rates; ISHIR created a 'Vibe Coding Cleanup Specialist' role; ~300% demand
    surge."
  Why they buy: "They need ELAI's exact wedge — a run that can't fabricate a passing test + a signed
  replayable change-journal — as a BILLABLE deliverable to their own clients." And "Proof = a customer
  outcome, not GitHub stars -> bypasses the OSS-virality gap."

THE RE-ANCHOR (GTM :95-97): "regulated/air-gapped enterprise = the LATE buyer (SOC2 + escrow + pentest
gates; long cycles that cap a bus-factor-1 operator)." And (:60-62): "the artifact moves TECHNICAL
REVIEWERS, not BUYERS. Procurement gates on SOC 2 Type 2 + ISO 42001 + escrow. The benchmark gets you in
the room; it never closes the regulated deal."
LATER REFINEMENT (K-7 :20-22, 2026-06-12): "developers don't pay for safety — orgs do (KS-3 report 16)";
the org-side buyer buys from EXISTING budget lines (AppSec/SCA) and via insurance underwriting
questionnaires. This is a THIRD buyer framing that sits between agencies and enterprise. M-SUMMARY :141
holds the line: 'agencies-not-enterprise' upheld; SSO/SAML/SCIM/RBAC + hosted console = demand-gated.

THE OTHER BUYER MODEL: CREDIT-BASED SEATS (K-24, 2026-06-26, K-SUMMARY :554-559). Subscription ->
monthly CREDITS (real provider $) -> spent per metered usage -> ELAI pays upstream. "SAFE (metered +
capped per request, never 'unlimited' an agent loop can detonate)." The stated edge: "ELAI's edge is NOT
a provider deal — it RESOLVES MORE TASKS PER DOLLAR." The margin reality is recorded honestly: "~15%
eaten on the frontier portion — makes J-55 routing quality the business model." And a PRIOR-CAUTION is
embedded verbatim: "sold as 'proven more value per dollar, verified' (a TRUST claim backed by
SAFEDELTA), never as 'cheap'; first revenue path is plausibly the team/agency seat tier (GTM memory @
$39-49/seat), not consumer tiers." The "$20=$200" pitch is explicitly GATED on the L6 SAFEDELTA number:
"a '$20=$200' claim with no L6 SAFEDELTA number FAILS the K-1 claims-policy lint; the pricing copy never
says 'unlimited.'"

THE DISTRIBUTION SEQUENCE, RANKED (GTM §4, :108-140) — with every caveat preserved:
  A. xyborg/SupaExplorer play [RECOMMENDED — "the ONLY genuinely-solo precedent that converted"]. A solo
     researcher turned an "11.04% exposure rate" finding into a front-page HN freemium launch (free OAuth
     audit + $5 report); the Supabase CEO validated it in-thread (credibility transfer). Replicate on
     RedAccess (~380,000 vibe-coded apps, proven leaky): "gate-OFF surfaced X, gate-ON blocked them —
     re-scan it yourself." BLOCKER: "RESOLVE the ToS/legal risk of scanning third-party apps first
     (GitGuardian/Socket precedents have legal teams)."
  B. anti-Replit reproducible incident-replay [RECOMMENDED]. Rebuild the Replit prod-DB wipe (1,190+
     companies hit during a code freeze; Jason Lemkin the named victim; Replit shipped "planning-only
     mode" = "the market conceded the gate-shaped hole"), run gate-ON, hand the viewer the SIGNED SEEDED
     artifact. Rafter 2026: "the only benchmark you can trust is one you run on your own codebase."
     CAVEAT: "press covered victims/big platforms, not vendor demos — a conversion asset, not a
     guaranteed-press asset."
  C. build-in-public on LinkedIn/X [RECOMMENDED, slow-compounding]. Base44/Maor Shlomo — solo,
     bootstrapped (~$10-20k own money), $1M ARR ~3 weeks post-launch, ~400k users, $80M Wix acquisition
     ~6 months; primary-source line "building in public on LinkedIn drove more growth than any paid
     channel." CAVEATS PRESERVED: "drop the invented $10k/$50k/$250k MRR ladder (real ~$200k/mo at
     sale); he ran failed influencer experiments (not 'no paid marketing'); Base44 is a no-code builder
     w/ pre-built audience — your cold-start lift is lower."
  D. one engineered Show HN / Product Hunt #1, timed to the number reveal [VIABLE]. Fire AFTER the
     artifact exists. Precedent Tony Dinh / TypingMind PH #1 -> $30k/mo. "One credible launch per
     product; don't burn it early."
  E. free privacy/local tier on VS Code Marketplace + OpenVSX + JetBrains [VIABLE funnel]. Accumulate
     installs + a 4.5-star rating first, then gate. "Treat '3-8x conversion / 15% paid' as DIRECTIONAL
     only (single SEO blog, not measured)."
  F. hand-court 5-10 design-partner agencies [VIABLE]. "A solo can't run 50 demos/week." 2-3 NAMED case
     studies with measured before/after security delta.
  AVOID/WEAK as leads: bare self-published number (AVOID — anti-credibility for a solo); paid
  dev-newsletter spend (WEAK — "the highest-trust list, Pragmatic Engineer 1.1M, takes ZERO
  sponsorships"); team-scale Snyk/GitGuardian SEO flywheel (WEAK — "every precedent is VC-funded w/ open
  free tier"). "Do NOT promise any conversion rate — every conversion stat in the deck is unverifiable
  or non-analogous. Measure your own funnel."

THE CREDIBILITY MOVE, RANKED (GTM §1, :24-58):
  Rank 1 (the entry move, "do this and only this first"): run gate-ON vs gate-OFF on the SAME cloud
  Sonnet through SusVibes (CMU harness behind Endor's Agent Security League; MIT, Docker-per-task,
  prediction format {instance_id, model_name_or_path, model_patch}, eval cmd `python -m
  susvibes.run_evaluation`, ~400GB + 4CPU/4GB per worker — all verified live). Publish raw predictions
  JSONL (both arms) + Docker image + fixed seed + commit hash + a voluntary trajectory-log folder.
  Why rank 1: "only path fully grounded AND fully under a solo/closed/no-VC operator's control (no
  gatekeeper)"; it copies the exact bar that makes AIDER's leaderboard trusted despite Paul Gauthier
  being effectively solo ("model + edit_format + commit_hash reliably reproduce any run").
  HARD CORRECTIONS (verifier-flagged; "getting these wrong self-falsifies the artifact"):
    - "Endor accepts NO external submission (18 curator-picked commercial combos). You publish YOUR OWN
      run on THEIR harness; never claim 'appearing on the leaderboard.'"
    - "Live Endor anchors are ~17.3% top SecPass (Codex+GPT-5.4) and ~84.4% top FuncPass (Cursor+Opus
      4.6), board dated/moving (entries through 2026-05-20). RE-PULL at publish time + cite the date.
      The 23.5%/91.1% and '20.1% Claude Code+Opus 4.7' figures are REFUTED."
      [NOTE: FOCUS-RESCOPE :157 still carries the REFUTED "top security score 23.5%" figure. That doc is
      one day older and was corrected by the GTM run. Do not cite 23.5%.]
    - "Do NOT pursue commercial independent-eval (Artificial Analysis, Epoch/MirrorCode) —
      capability/cost/latency benchmarkers with NO security yardstick; they cannot host a trust delta."
  Contamination defense (:43-44): "a gate-ON/OFF delta on same model + same seed cancels common-mode
  training-data leakage (inflation appears in both arms)." Tagged as "sound reasoning, NOT a proven
  credibility trick."
  Rank 2 (Phase-2 amplifier): academic co-sign with SusVibes/CMU maintainers (named contacts on file).
  Rank 4 (fire LAST): Show HN — "'we can't show source, here's the Docker + seed + JSONL, run our
  SAFEDELTA yourself' reframes closed-source from a hole into a verify-the-claim posture. Never
  astroturf."

THE PERSIST/ABANDON TEST (GTM :184-188, verbatim — the only falsification criterion the GTM story has):
  PERSIST if within ~90 days: "the SAFEDELTA delta is REAL and MATERIAL (gate-ON measurably cuts
  insecure-rate on the same model) AND >=1 cleanup agency pays for the harness as a billable
  deliverable."
  ABANDON/pivot if: "the delta is within noise (gate doesn't move outcomes on a neutral harness ->
  thesis hollow) OR after a genuine public finding + 10 hand-courted agencies, ZERO convert at $39-49."
  "Do NOT abandon on slow enterprise traction — that's the expected late buyer, not a signal."
  SINGLE BIGGEST RISK (:181-183): "shipping the number before it's independently re-runnable (the Devin
  failure mode)... The artifact's reproducibility is the product's first real feature — get it right or
  don't ship it."
  [The 90-day clock started 2026-06-02 and expired ~2026-08-31. As of 2026-07-17 neither condition can
  be evaluated: the delta does not exist. See §13.]

THE 90-DAY SEQUENCE (GTM §5, :150-168) — recorded because nothing in it has visibly started:
  Days 0-30: (1) wire the gate-ON/OFF rig against SusVibes; (2) build the signed RRA + Apache-2.0
    verifier/replay tool (= G-2 + G-8); (3) START the SOC 2 Type 2 observation window NOW (3-6mo lead;
    "in-audit by [date]" keeps a first security review alive); (4) pre-draft a SaaS source-code escrow
    clause (Codekeeper lightweight / Escode).
  Days 30-60: (5) run the delta + publish JSONL/Docker/seed/commit + re-pull the live Endor anchors at
    publish; (6) email the SusVibes/CMU maintainers for an academic co-sign; (7) the anti-Replit
    incident-replay asset; (8) free local-Qwen tier + VS Code Marketplace listing.
  Days 60-90: (9) the xyborg-shaped public-corpus finding; (10) hand-court 5-10 cleanup agencies, close
    2-3 into NAMED case studies; (11) the engineered Show HN / PH #1; (12) build-in-public cadence.
  DON'T in 90 days (verbatim :167-168): "chase regulated/air-gapped enterprise; lead with Article 50;
  buy newsletter ads; open-source the engine; pursue the Endor leaderboard as a real track."

THE PRE-ELAI-PRODUCT ALTERNATIVE (agency-strategy-2026-05-19.md) — the OLDEST doc and the one the rest
of the corpus silently overrode. It deserves its own record because its economics have never been
refuted, only ignored:
  Situation (:8-14): Japan, $600/mo part-time LLM income; ELAI 2 months in; "~500h+ of solo work"
  remaining; "Many parts bug-ridden, dashboard not wired, no manual feature testing done"; "No audience,
  no funding, no warm network."
  Its verdict on ELAI's moats (:19-34): of 50+ architectural primitives, exactly THREE survive the
  "genuine market moat vs shipping competitors" filter: (1) mandatory grounding with a blocking pre-tool
  hook, (2) the anti-spiral 4-pattern detector + 5-persona switch, (3) the wiring-contract validator.
  "The other ~45 primitives (SSB/BSP/DKG/CatRAG/cosmos/reflect/idea-framer/etc) are solid engineering
  but don't move the competitive needle for product marketing. Good engineering != market moat. Both can
  be true."
  Its base rates (:47-52): "No solo-no-audience case study reached $1M ARR in 12 months in 2024-2026.
  80-90% never ship past landing page; 70% of shippers make <$500/mo; 2-4% reach $1K MRR; 0.5-1% reach
  $10K MRR."
  Its ratio (:36): "Tabby (best local-first vendor) = $990K ARR with 9 employees vs. Cursor = $2B ARR.
  Ratio is 2000:1. Local-only as primary pitch is betting against the curve."
  Its counter-proposal (:54-58): 1 client at $5-8K = 8-13x current income for one month; 3-5 retainers
  at $2-3K/mo = $10-15K MRR; time to first deal 4-6 weeks of cold outreach; "ELAI stays private as
  internal speed multiplier — sell outcomes, not the tool."
  5 ICPs ranked (:63-93): (1) YC W24-W26 AI startups pre-Series A, $5-15K projects; (2) Japan B2B SaaS
  Seed-A, ¥500K-¥2M (~$3-13K); (3) AU/NZ SaaS, AU$5-20K; (4) Indian SaaS/D2C, ₹3-15L (~$3-18K); (5)
  Indie SaaS founders at $10-50K MRR, $3-8K projects. "AVOID: US Enterprise (procurement nightmare),
  European (GDPR + slow contracts), government, big tech, anyone past Series B."
  Volume math (:149-155): 5-10% reply rate at that specificity, ~1-2% to first call; 10-15 sends/day
  fully personalized; 2 weeks = 100-150 emails -> 5-15 replies -> 3-5 conversations -> 1 deal at $3-5K;
  first revenue 4-6 weeks; "$5-8K MRR achievable by month 3 if outreach pace sustained." Pricing ladder:
  client 1 $2-3K (goal = case study, not income), client 2 $5K + testimonial, client 3 $5-8K, client 4+
  $8-12K or $2-3K/mo retainers.
  ITS ELAI PRESCRIPTION (:265-273) — DIRECTLY CONTRADICTED by everything after 2026-06-01:
    "1. Open-source ELAI as-is this week." (vs the closed-source + verifier-only-open decision.)
    "2. Pick ONE component to ship as a Claude Code or Cursor plugin at $19/mo. Anti-spiral
       persona-switch is strongest candidate." ($19/mo vs the later $39-49/seat.)
    "3. ELAI as monolithic product is paused, not dead."
  Its bottom line (:289-295): "The Sequoia-funded founder in the YouTube video has runway, network, and
  a chef. You don't... You are closer to the money than you think. You are pointed at the wrong
  endpoint."
  STATUS: no doc in the corpus ever ARGUES against the agency play. It is simply not mentioned again
  after 2026-05-19. The 2026-06-02 GTM playbook's ceiling statement ("a profitable solo/small product
  serving agencies + prosumers... a path to first revenue, NOT a venture outcome") converges on a
  similar income target by a slower route, but with agencies as CUSTOMERS of the product rather than as
  clients of a consultancy. This is an UNRESOLVED strategic fork, not a decided one. [CONTRADICTION]

==============================================================================
11. THE PLAN TRACE — DOES A UNIT EXIST FOR EACH COMMITMENT, AND IS IT BUILT?
==============================================================================
Method: forward from each strategic commitment to its plan unit (per the SUMMARY indexes), then live
existence checks on this tree (2026-07-17) [VERIFIED where marked].

COMMITMENT                          -> PLAN UNIT           -> STATUS
  The number (SAFEDELTA gate-ON/OFF) -> G-6 / K-1 / A-3     -> RUNNER PRESENT, NUMBER NOT MEASURED
    [VERIFIED] .elai_cc/crates/orchestrator/src/benchmarks/safedelta_runner.rs + crates/cli/src/
    bench_safedelta.rs + seccodeplt_runner.rs all exist; `proof_spine_status.py` reports PS-1 =
    RUNNER_PRESENT. The dossier (2026-07-08) confirms "no SafeDeltaMeasured SSB row present". The
    Tier-1 static number DESIGN-SELL-BY called a "day-one quick-win, runnable NOW, zero infra"
    (2026-06-05) is STILL not on the tree as a published number 6 weeks later. This is the single
    largest gap between the strategy and the build.
  RRA (the proof object)             -> G-2                 -> PRESENT [VERIFIED rra.rs exists]
  in-toto attestation + offline verify -> G-8               -> PRESENT [VERIFIED attestation.rs exists;
    recent commit "add the elai export compliance cli arm ... rra plus attestation.json, rra zeroed
    after a verified attestation"]. NOTE P0-3 (2026-07-10): the local Merkle default is NOT a signature.
  Redaction (precondition to shipping the RRA) -> G-9       -> PRESENT [VERIFIED redaction.rs +
    privacy_redactor.rs + retention.rs exist]
  Compliance export (procurement pack) -> K-6               -> PRESENT [VERIFIED: recent commit adds the
    `elai export compliance` CLI arm composing rra/attestation/redaction/retention]
  PR/CI evidence emitter (THE distribution leg) -> K-3/K-11 -> SUBSTRATE PRESENT, RENDERER UNVERIFIED
    [VERIFIED .elai_cc/crates/orchestrator/src/github_pr/ exists]
  Gate-tax ceiling (affordability)   -> K-4                 -> RUNNER PRESENT [VERIFIED bench_gate_tax.rs
    + gate_tax_audit_leg.rs + k4_gate_tax_contract.rs]; PS-2 = RUNNER_PRESENT, number not measured.
    (This SUPERSEDES the dossier's 2026-07-08 "no `elai bench` subcommand exists for this yet".)
  Routing economics ($/verified-task) -> K-2                -> RUNNER PRESENT [VERIFIED
    bench_routing_economics.rs + routing_economics_runner.rs + k2_routing_economics_contract.rs]; live
    cost leg BLOCKS on J-28. PS-4 = RUNNER_PRESENT.
  Risk-tiered gating (the 07-10 doc's "should be foundational") -> K-5 -> PLANNED, NOT BUILT
  Claims policy / KD-1 lint          -> K-1                 -> BUILT [VERIFIED claims_policy_lint.py +
    ELAI-CLAIMS-POLICY.md + ELAI-PROBLEM-QUANTIFICATION-DOSSIER.md all exist]
  AppSec/insurance positioning brief -> K-7                 -> NOT BUILT [VERIFIED: docs/strategy/
    ELAI-APPSEC-INSURANCE-BRIEF.md ABSENT; K-7 is PLANNED with a full WIRING_CONTRACT]
  OWASP control mapping              -> K-13                -> NOT BUILT [VERIFIED: docs/strategy/
    ELAI-OWASP-CONTROL-MAPPING.md ABSENT]
  `elai wrap` (trust-layer wrapper)  -> I-40                -> BUILT (CLI arm) [VERIFIED args.rs:1730-31
    "I-40 — `elai wrap [--out <path>] -- <agent cmd...>` ... minimal trust-layer proxy"; parse_wrap_
    subcommand_args at args.rs:3446]. P0-5 (2026-07-10) says it is bypassable without whole-process
    confinement — so the SECURITY claim on it must be narrowed until I-1 supervisor confinement lands.
  MCP server (`elai-verify` as a tool inside competitors) -> D1 / FUTUREPROOF A1 / J control-plane
                                      -> NOT VERIFIED BUILT. DESIGN-SELL-BY Appendix A confirmed
    mcp_bridge.rs is CLIENT-ONLY; the server direction was "genuinely net-new" as of 2026-06-05.
  Credit-based seats / billing       -> K-24                -> NOT BUILT [VERIFIED: crates/orchestrator/
    src/billing/ ABSENT]
  Seat entitlement / commerce        -> M-1..M-4            -> NOT BUILT [VERIFIED: crates/orchestrator/
    src/commerce/ ABSENT]
  Standalone OSS verifier (the open-core move) -> M-5a      -> NOT BUILT [VERIFIED: crates/elai-verify
    ABSENT; docs/strategy/commercial/ ABSENT; tiers.toml ABSENT]
  Evidence format as open standard   -> M-6                 -> NOT BUILT (authored 2026-07-05)
  Multi-repo / workspace memory      -> K-23                -> NOT VERIFIED (net-new capability track)
  Model-refresh cadence              -> K-9                 -> BUILT [VERIFIED: commit ead13d9 "elai
    bench model-refresh: deterministic quarterly model-refresh cadence bench ... ModelRefreshCadence
    ssb action type at schema 276"]
  Absorption watchdog                -> K-25                -> AUTHORED 2026-07-05, build unverified

COMMITMENTS WITH NO PLAN UNIT AT ALL (the major findings):
  F-1. SOC 2 TYPE 2 OBSERVATION WINDOW. GTM Day 0-30 item #3 ("start NOW; 3-6mo lead"). There is NO plan
       unit, no doc, no tracked artifact for it anywhere in the corpus. It is named as a PROCUREMENT
       GATE in two docs ("Procurement gates on SOC 2 Type 2 + ISO 42001 + escrow", GTM :60-62) and as a
       long-lead clock that must start before anything else, and it has never been started. If the
       org-side buyer (K-7's premise) is real, this is a hard dependency with a 3-6 month lead and zero
       progress.
  F-2. SOURCE-CODE ESCROW. GTM Day 0-30 item #4 + the named answer to Hard Truth 5.5 (bus-factor). The
       only trace is M-5c ("trust-continuity statement — escrow-on-lapse or convert-to-OSS-on-
       abandonment (user picks; both drafted)") which is UNBUILT and sits behind GATE M-0 (M executes
       last). So the ONE named mechanism for the corpus's own root contradiction is the last thing in
       the build order.
  F-3. THE SUSVIBES RIG ITSELF. The Rank-1 credibility move ("do this and only this first") names a
       specific external harness (SusVibes/CMU, `python -m susvibes.run_evaluation`). G-6 lists
       SUSVIBES-200 as a suite, but there is no unit that OWNS wiring ELAI's gate-ON/OFF arms into that
       third-party harness and publishing the JSONL+Docker+seed envelope. The whole believability
       argument rests on it.
  F-4. THE ACADEMIC CO-SIGN (Rank 2), THE xyborg PUBLIC-CORPUS FINDING (channel A), THE ANTI-REPLIT
       DEMO ASSET (channel B, = DESIGN-SELL-BY C2), THE VS CODE MARKETPLACE LISTING (channel E), THE
       BUILD-IN-PUBLIC CADENCE (channel C). C2 has a plan slot in DESIGN-SELL-BY; the rest have none.
       Every named distribution channel except the PR emitter (K-3) is planless.
  F-5. THE 90-DAY GTM SEQUENCE has no owner unit, no tracking artifact, and (per the tree) no started
       item. Its clock expired ~2026-08-31.
  DIAGNOSIS: the corpus has built PROVE machinery (benches, lints, dossiers) and DELIVERY machinery
  (RRA, attestation, export, wrap) but has built NOTHING on the two axes both the DIRECTION doc (R2) and
  the GTM playbook name as the binding constraint: DISTRIBUTION and CREDIBILITY-TRANSFER. DIRECTION
  :182-186 called distribution "the biggest UNSOLVED problem" and "the constraint with no current
  answer" on 2026-06-01. 6.5 weeks later it still has no plan unit.

==============================================================================
12. THE MONETIZATION STORY vs THE PROOF SPINE
==============================================================================
Source: workflow/reasoning/proof-spine-2026-07-05.md; .claude/rules/proof-spine.md.
The ruling (verbatim :8-10): "The corpus's credibility compresses onto six measured numbers; any unbuilt
unit that feeds one of them outranks any unit that does not, and new plan files must declare which (or
none, with explicit user ack)."

Mapping each spine number to the monetization argument it underwrites:
  PS-1 SAFETY DELTA (G-6). -> THE product claim. Every "cant-lie/cant-wreck" sentence, the killer demo,
       the agency pitch, the credit-seat "$20=$200" pitch, K-7's brief, and the persist/abandon test all
       terminate here. STATUS [VERIFIED 2026-07-17]: RUNNER_PRESENT, NOT MEASURED. Blocked per the
       dossier on RunPod for the F-2b precursor (W-10) — though DESIGN-SELL-BY (2026-06-05) argues the
       Tier-1 static leg needs NO GPU at all and the Tier-2 dynamic leg needs cloud budget + a Linux CPU
       host, not a GPU. [CONTRADICTION: the dossier says PS-1's precursor is RunPod-gated; DESIGN-SELL-BY
       says the GPU is off the critical path. DESIGN-SELL-BY is code-grounded in its own Appendix B and
       is the stronger claim; the dossier's gating is on the LOCAL-model arm which 5.3 already demoted
       to privacy-floor.]
  PS-2 COMPOSED GATE-TAX (K-4). -> The AFFORDABILITY claim, and the answer to the 2026-07-10 doc's
       "gate proliferation can make the safe system unusable". Without it the product is a plausible
       CodeGate repeat (TRUST-MAX §7: "false-positive / alarm-fatigue friction is what killed the
       closest prior art (CodeGate, archived 2025-06-05)"). STATUS: RUNNER_PRESENT, not measured.
  PS-3 HALLUCINATION DELTA (L-10). -> The claim-verification product (L-plan's outward turn). PLANNED.
  PS-4 ROUTING ECONOMICS $/verified-task (K-2). -> The "more value per dollar, verified" pitch that the
       credit-seat business model (K-24) is priced on. RUNNER_PRESENT; live leg BLOCKS on J-28.
  PS-5 CASE-REUSE COMPOUNDING (K-18). -> The "gets cheaper on YOUR repo" retention/moat claim.
       STATUS [VERIFIED]: MEASURED 2026-07-17 — recurrence_hit_rate@1 = 0.95 (19/20), @3/@5 = 1.00,
       false_match_rate 0.00, bm25_only, 95% Wilson CI at k=1 [0.85, 1.00]
       (workflow/reasoning/proof-spine-ps5-case-reuse-2026-07-17.md).
       CAVEAT PRESERVED VERBATIM from that doc: this is "the OBSERVE-arm recurrence-hit-rate over the
       SHIPPED retrieval stack, NOT the compounding reuse-into-generation moat number (that is K-18
       Task 2, still BENCH-PENDING on the unbuilt VCG J-track)". So the moat claim is NOT measured; only
       the retrieval precondition for it is.
  PS-6 WIRE-DARK RATE (dated ledger row). -> Not a customer-facing number; it is the credibility of the
       whole corpus's "built" claims. STATUS [VERIFIED]: MEASURED 2026-07-09 — 933 DARK pub fns / 12,137
       judged = 7.69% (alt framing 933/9,323 = 10.0%), fn-grain static post-stack, .elai_cc/crates,
       1,156 scan-eligible non-test .rs files (workflow/reasoning/proof-spine-ps6-wire-dark-rate-
       2026-07-09.md). That doc loudly warns the unit is NOT the same as the ~40-45% pre-stack anchor.
       This materially REVISES the June docs' "~15% wire-dark" (FOCUS-RESCOPE 5.1) and DIRECTION R4's
       "~52% wired / ~15% wire-dark / ~17% env-gated" — but at a different unit of measure, so it is a
       replacement of an unsourced figure with a sourced one, not a proven improvement.

SPINE SCORE [VERIFIED 2026-07-17]: "PROOF SPINE 2/6 MEASURED". The 2 measured are the two that sell
nothing (PS-5's observe arm and PS-6's internal hygiene metric). All four numbers the monetization story
actually rests on (PS-1 safety, PS-2 affordability, PS-3 hallucination, PS-4 economics) are unmeasured.

==============================================================================
13. TIMELINE OF THE DIRECTION (how it moved, in order)
==============================================================================
  2026-05-19  AGENCY PIVOT. ELAI is "paused, not dead"; open-source it as a portfolio piece; ship ONE
              component as a $19/mo plugin; replace $600/mo with $5-10K/mo via cold-outreach consulting.
              Verdict on ELAI: 3 of 50+ primitives are moats; local-only pitch bets against the curve.
  2026-06-01  ADJUST, DO NOT PIVOT. Trust thesis correct; DEMOTE cost + tokens as moats; RE-AIM off
              non-coders onto devs/prosumers/agencies/privacy-teams; a narrow measured trust-layer
              release in ~3 months. Closed source + one-click installer. R1 = the number (thought to
              need the 2x3090); R2 = distribution, "the constraint with no current answer."
  2026-06-02  FOCUS (AM). Cede the everything-machine (one-sided yes). WIN column EMPTY until SAFEDELTA.
              The wedge sentence + the $39-49 anchor + the "buyer who can action a gated diff" test. Five
              hard truths; two root contradictions (Anthropic-dep, solo bus-factor); a 2-3 quarter window
              treated as the OUTER bound. §8 leaves 5 open questions.
  2026-06-02  TRUST-MAX (same day). Answers "what to build": the READ/EVIDENCE/RUNTIME/MEASUREMENT side
              nobody occupies. The headline move = RRA + SAFEDELTA coupled. R1..R9 ranked, waves 1-3.
              Preserves 12 verifier landmines verbatim (DO-NOT-CITE list).
  2026-06-02  GTM (same day). Answers FOCUS §8's two biggest open questions: how a solo closed-source
              number gets believed (self-run on SusVibes + full reproduction envelope) and who pays first
              (vibe-cleanup agencies). The open-core line is drawn: verifier open, engine closed.
              Article-50 hook KILLED. 90-day sequence. Persist/abandon test.
  2026-06-03  ENGINEERING. "Refine the engineering, do NOT pivot the thesis." Refutes three of maintainer's
              own framings (small-model-smart, tokens-as-goal, LLM-compaction). Kills BSP-for-code that
              DIRECTION §7 was about to wire. Phase 0 = "pass your own gate" (purge ELAI's own
              hallucinated numbers). §16 ERRATA corrects the doc's own sections 1-9.
  2026-06-04  FUTUREPROOF. The control-plane inversion: expose the gates as an MCP server + A2A peer so
              ELAI GATES Claude Code/Cursor/Copilot instead of competing for the seat. This is proposed
              as the answer to BOTH root contradictions at once.
  2026-06-05  SELL-BY-DEC. Fuses all of the above into one buildable backlog with per-item [LIVE]/[STUB]/
              [CITED]/[NEW] tags and DONE-GATEs. Two big corrections: the number needs NO GPU (Tier-1
              runnable now, zero infra), and three F/G/H plan premises are contradicted by live code
              (stdout_hash exists, CloudSpend emit exists, routing is not inert). Critical path named.
              "~2 quarters left."
  2026-07-04  COST-MODEL MAP. Not a direction change: a full grounded map of the cost pipeline (11
              stages) against the six findings in maintainer's own Claude-Code usage screenshot. Its verdict
              is the honest one: "(A) does ELAI-the-product architecturally address each cost dimension
              ... 'yes, comprehensively, mostly in spec'; (B) does ELAI's own dev harness eat its own
              dogfood ... 'no — the harness is itself a live example of the problem ELAI is built to
              solve.'" And: "the headline 'cheaper by delegation' numbers are ... BENCH-PENDING."
  2026-07-05  PROOF SPINE. KD-1 lifted from marketing copy to BUILD ORDER. Six numbers; PSR-1 precedence;
              every new plan file declares which spine number it feeds or gets a user-acked `none`.
  2026-07-08  CLAIMS POLICY + DOSSIER. The KD-1 bar made normative + enforced by a lint; every ELAI
              answer row is PENDING.
  2026-07-10  MARKET EVALUATION. "Not the best harness; no defensible evidence." The new risk is BREADTH.
              Six P0 trust-critical DESIGN flaws. Move verticals into packs; remove 12 named items.
              Recommends wrapper-first — then REVERSES itself in-file: kernel-first engineering,
              native-first identity, wrapper-assisted distribution. Prescribes a same-model tournament
              with pre-registered kill rules.
  2026-07-17  (this sweep) PS-5 observe-arm measured. Spine 2/6. PS-1 still unmeasured.

==============================================================================
14. CONTRADICTIONS AND DRIFT (every place the corpus disagrees with itself)
==============================================================================
C-1. WRAPPER-FIRST vs NATIVE-FIRST. Within the SAME 2026-07-10 file. §7.3/§8/§11 say make `elai wrap`
     the first product and ELAI's native orchestrator an optional destination; the Addendum (same date,
     explicitly superseding) says "kernel-first engineering, native-first product identity,
     wrapper-assisted distribution" and cites I-40's own text ("The wrapper is the on-ramp, not the
     product"). EVIDENCE FAVORS: the Addendum — it quotes the primary plan text the body had
     paraphrased, and it names the three-arm experiment that would settle it empirically rather than by
     analogy. RESOLUTION: cite the Addendum.

C-2. "EVERYTHING FAILS CLOSED" vs RISK-TIERED. The whole F/G spine and wiring-enforcement R1-R10 are
     built on "missing evidence is a hard FAIL, never SKIP" (ELAI-SPEC-OVERVIEW :24-25). The 2026-07-10
     doc says (:253-257) "The doctrine should not be 'everything fails closed'" — only high-risk/
     irreversible/security/privacy/finance/credential/publication boundaries; "quality and productivity
     checks can quarantine, retry, request evidence, or remain advisory according to measured risk."
     EVIDENCE: K-4/K-5 already existed (2026-06-12) as the corpus's own answer, and K's own premise
     names FP friction as "the affordability gate the prior generations failed" (K-SUMMARY :107-109).
     TRUST-MAX §7 already named CodeGate's death by alarm fatigue as "THE ONE RISK". So this is not new
     disagreement — it is the 07-10 doc promoting an existing minority position (K-5) to foundational.
     UNRESOLVED: the F/G done-gate doctrine has not been amended, and K-5 is unbuilt.

C-3. THE 9-STATE FSM: audit envelope vs execution ceremony. SPEC-OVERVIEW/Phase-3A treat the 9-state FSM
     with per-transition evidence gates as the spine. The 2026-07-10 doc (:228-240, :404-411) says the
     market moved the other way (Symphony: rigid nodes did not age well; Kilo removed its Orchestrator;
     mini-SWE-agent shrank) and prescribes "strict at boundaries, flexible inside" + three operating
     tiers (Fast / Standard / High assurance) + "Benchmark the current nine-state/tool-blocking design
     against a smaller objective-driven envelope under the same model." Its OWN Addendum (:859-867)
     then narrows this: "The resulting correction is narrower than deleting ELAI's FSM." EVIDENCE: the
     market signals are first-party-sourced and real; the corrective is reasoned. UNRESOLVED and
     UNBENCHMARKED.

C-4. GPU-GATED vs NOT. DIRECTION-06-01 R1: the number "Needs the 2x3090 box." DESIGN-SELL-BY-06-05
     (:156-188, :364-371): "NO LOCAL GPU REQUIRED (corrected 2026-06-05)... The 2x3090 is OFF the
     critical path." The 2026-07-08 dossier still records F-2b as "blocked on RunPod GPU access (W-10)".
     EVIDENCE FAVORS DESIGN-SELL-BY for the HEADLINE number (it is code-grounded in its own Appendix B,
     and Hard Truth 5.3 already ruled the local 7B out of the headline). The dossier's RunPod gating is
     correct only for the LOCAL-lane arm, which is a secondary number. NET: the headline number has been
     un-blocked for 6 weeks and has not been run. This is the sharpest actionable finding in the sweep.

C-5. WIRE-DARK RATE. ~40-45% (wiring-enforcement, ~187 plans, pre-stack) -> "~15% wire-dark" (FOCUS-
     RESCOPE 5.1) -> "~52% wired, ~15% wire-dark, ~17% env-gated" (DIRECTION R4) -> "28% SHIPPED_WIRED
     over 213 plans" (2026-06-04 audit, per proof-spine PS-6) -> 7.69% DARK pub fns / 12,137 judged
     (2026-07-09, fn-grain). NOT COMPARABLE: four different units of measure (plans vs pub symbols). The
     PS-6 doc says so loudly itself. Do not present this as an improvement trend.

C-6. THE ENDOR TOP SECURITY SCORE. FOCUS-RESCOPE :157 says "top security score 23.5%". GTM :49-51 (one
     day later) explicitly REFUTES it: "Live Endor anchors are ~17.3% top SecPass (Codex+GPT-5.4) and
     ~84.4% top FuncPass (Cursor+Opus 4.6)... The 23.5%/91.1% and '20.1% Claude Code+Opus 4.7' figures
     are REFUTED." EVIDENCE FAVORS GTM (it is the later, verifier-corrected run). Both figures are now
     >13 months from a re-pull; GTM's own instruction is "RE-PULL at publish time + cite the date."

C-7. ARTICLE 50's LEGAL BASIS. FOCUS-RESCOPE :132-137 and TRUST-MAX/G-8 treat Art-50 marking as legally
     binding 2 Aug 2026 (grace to 2 Dec 2026) and a real (if weak) tailwind. GTM :142-146 kills it:
     the draft guidelines EXEMPT source-code outputs and NAME agentic dev tools as the exempt class.
     EFFICIENCY-TRUST :206-207 says it "Governs synthetic media output, not source-code commits" and
     tags the dates UNVERIFIED-pending. CONVERGED OUTCOME: never claim certification; "designed to
     support" only; use the date as an outreach hook at most. K-7 and K-6 both encode the kill.

C-8. AGENCY-AS-CLIENT vs AGENCY-AS-CUSTOMER. 2026-05-19 says sell CONSULTING to startups and keep ELAI
     private as an internal multiplier ($5-10K/mo in 4-6 weeks, base-rate-grounded). 2026-06-02 says
     sell ELAI-the-product to cleanup AGENCIES at $39-49/seat (a path to first revenue over ~90 days,
     gated on a number that does not exist). No document ever adjudicates between them. The base rates
     in the older doc (0.5-1% of solo-no-audience launches reach $10K MRR) were never refuted; the newer
     doc's own ceiling statement ("a path to first revenue, NOT a venture outcome") is consistent with
     them. THIS IS THE LARGEST UNRESOLVED FORK IN THE CORPUS. [CONTRADICTION, unadjudicated]

C-9. "TRUST IS THE MOAT" vs "TRUST IS TABLE-STAKES". DIRECTION :72-78 says trust alone is NOT a wedge
     (Claude Code ships hooks/isolated subagents/a fresh-model verifier; Qodo owns the literal phrase
     "verification layer" at $120M); the whitespace is the COMBINATION. The 2026-07-10 doc agrees but
     narrows differently (:222-225): the differentiator is that ELAI treats verification as THE PRODUCT
     — "independent receipts, a gate manifest, replay, controlled ablation, a standalone verifier, and
     an evidence interchange format" — where commercial products "usually treat verification as
     configured workflow." Both agree the naked word "trust" is not defensible. CONSISTENT.

C-10. TIMELINE. "~3 months" for a narrow measured release (2026-06-01) -> "2-3 quarters, OUTER bound"
      (2026-06-02) -> "~2 quarters left ... Dec 2026" (2026-06-05) -> the 2026-07-10 doc names no
      timeline at all and instead says to freeze feature expansion. Elapsed: 2026-06-01 -> 2026-07-17 is
      ~6.5 weeks, i.e. roughly half of the original 3-month narrow-release window, with the gating
      artifact (the number) unbuilt.

==============================================================================
15. CLAIMS-POLICY VIOLATIONS AND WEAK CLAIMS FLAGGED IN THIS CORPUS
==============================================================================
The policy binds externally-visible copy only, so the reasoning docs below are not technically in scope.
Flagged anyway because they are the numbers most likely to leak into copy:
  V-1. ELAI-PITCH.txt is the highest-risk artifact in the sweep. It is friend-facing (i.e. external in
       practice) and it carries untagged numbers and capability assertions: "roughly 40 to 60 percent of
       ai generated code ships with a vulnerability" (untagged external), "17-odd languages", "a couple
       hundred tokens", "with a fixed seed two runs come out byte-for-byte identical, which is genuinely
       unusual for an LLM system" — the last one is a SUPERLATIVE-adjacent claim that R-K (DFAH/Thinking
       Machines) constrains hard: byte-identity holds for the harness fold and is FALSE for the cloud
       lane. The pitch does carry an honest status paragraph (:34: "a real chunk is still being wired end
       to end, and the next thing im doing is putting an actual measured number on it"), which is the
       right posture. If this text is ever reused as copy it must go through claims_policy_lint.py.
  V-2. "55-87% token savings vs naive full-chunk retrieval" (COST-MODEL §5, SPEC-OVERVIEW Phase 1E)
       appears repeatedly as a design target. FOCUS-RESCOPE :80 already flagged it: "the '55-87%
       savings' string isn't in ELAI's repo". It is a [DESIGN] target with no measured pointer. Do not
       ship it as a result.
  V-3. "41-80% cloud cost cut" from prompt caching. Explicitly downgraded to [WEAK] by the doc's own
       errata (§16/E4): "cross-provider pooled, DeepResearch-Bench preprint on a prefix-heavy workload,
       not a general or Anthropic-only number. The only honest ELAI target is a measured hit-rate."
  V-4. The 2026-07-10 doc's raw dossier "Best X" labels (:687-697) are analyst judgments — the doc
       itself says so verbatim (:677): "Its 'Best' labels are analyst judgments among the reviewed public
       mechanisms, not benchmark-established winners."
  V-5. Every market number in §2/§4 above (Cursor ~$2B ARR at ~$50B; Claude Code ~$2.5B run-rate;
       Anthropic ~$965B valuation; Cognition $1B raise at ~$26B / $492M ARR; Lovable $400M ARR / $6.6B;
       Replit $9B; OpenRouter $1.3B / ~100T tok/mo; Qodo $70M/$120M; OpenHands $18.8M Series A / 60k
       stars / 4M downloads / 77.6% SWE-bench Verified; Snyk $326M ARR; Escape $18M; Cline ~5M installs /
       2.7M+ devs; Aider ~15B tok/wk; Ollama ~52M pulls/mo; Tabby $990K ARR / 9 employees; Base44 $80M
       Wix acquisition; ~3,800 AI startups died 2025 + ~1,800 early 2026; 70% of engineers run 2-4 AI
       tools; ~130 players; core market ~$4B with top-3 >70% share) is [EXTERNAL] and dated Dec-2025 to
       May-2026. ALL are 2-8 months stale as of 2026-07-17 and the market-velocity caveat in the
       2026-07-10 doc (:671-673) applies to every one of them: "Re-check those facts before external
       publication."
  V-6. The buyer-gap numbers likewise [EXTERNAL], dated: Sonar Jan-2026 n=1,100 "96% don't fully trust
       AI code, only 48% verify it"; DigitalOcean "41% name reliability the #1 scaling barrier"; Stack
       Overflow "trust in AI accuracy fell to 29%"; Fortune Apr-2026 "trust is the real bottleneck";
       Princeton "reliability LAGS capability across model generations — the gap is WIDENING"; "~88% of
       agent pilots die on governance/compliance, not model quality"; "24% of work-week on review";
       METR arXiv:2507.09089 "19% slowdown" from reviewing AI output; "40% of agentic projects fail on
       cost + security + trust"; "63% of vibe users are non-devs (a usage stat, not a paid-seat stat)";
       "91.5% of vibe-coded apps had a hallucination-traceable flaw (Q1 2026)"; Escape.tech "5,600
       vibe-coded apps: 2,000+ high-impact vulns, 400+ exposed secrets, 175 PII leaks".
  V-7. The DO-NOT-CITE landmine list (TRUST-MAX §5, :285-313) must be preserved verbatim wherever these
       topics recur. Summary of what is BANNED: the PRvL Presidio precision/recall numbers (REFUTED);
       EUR 35M/7% paired with a logging argument (wrong tier); CyberSecEval 3 as the insecure-code-
       GENERATION source (it is offensive-cyber; the right suite is CyberSecEval 1/2 arXiv:2404.13161);
       "CVE-2026-30623 in Anthropic's MCP SDK" (that CVE is LiteLLM; Anthropic's SDK has no CVE);
       Qwen3-8B as the Thinking-Machines nondeterminism subject (it was Qwen3-235B-A22B-Instruct-2507);
       "BSP has no production caller" (REFUTED — task_runner.rs:5018; the real gap is
       recover_from_checkpoint); the Kiro "two-person approval optional" inference (Amazon disputes the
       attribution); Martian/CodeRabbit "#1" (vendor-amplified; Greptile's own bench names the opposite
       winner — use the divergence as the ARGUMENT for a reproducible metric); the GitHub Copilot review
       71%/29%/5.1-comments figures (could not be confirmed — borrow the pattern, not the numbers);
       mcp-scan's dist channel (unconfirmed; also mcp-scan is Invariant Labs, NOT Snyk).
       Plus from EFFICIENCY-TRUST: BaxBench "62% incorrect-or-insecure" INVERTS the metric (62% is the
       correctness ceiling); "85% latency reduction" from caching does not exist (TTFT is 13-31%);
       Cursor's "27% baseline -> 31% savings" is FABRICATED (actual ~5%); vLLM "7.2x/57x" prefix-cache
       blog figures are fabricated/mis-attributed; token-efficient tool use "14%/70%" was measured on
       Claude 3.7 beta, not Sonnet 4.6; the Complexity Trap "7%/11% hybrid" is abstract-only future work;
       Chroma's "20-50% / 200K->50K" is ELAI synthesis, not in the source; MAST is ~150 tasks / 5
       frameworks (not "200 traces / 7"); the Artificial Analysis harness cost spread is ~60x /
       $0.07-$4.10+ (not 32x/$2.26).

==============================================================================
16. STALENESS / DEAD PATHS
==============================================================================
  [VERIFIED] NO dead F:\.ELAI_workflow paths appear in any of the files read for this sweep
  (grep count = 0 across the ELAI-*.md direction docs, ELAI-PITCH.txt, agency-strategy, K-7). The
  F:\ paths survive only in the project CLAUDE.md instruction blocks, which are out of this sweep's
  scope but are themselves stale: the live root is /home/user/teikoku/elai.
  agency-strategy-2026-05-19.md :115 contains an unfilled placeholder in the cold-email template:
  "github.com/[your-handle]/elai_cc". It also (:269) instructs "Open-source ELAI as-is this week" —
  superseded by the 2026-06-02 open-core decision; do not act on that line.
  The DOSSIER's Row 4 statement "no `elai bench` subcommand exists for this yet" (2026-07-08) is STALE
  as of 2026-07-17: bench_gate_tax.rs, bench_routing_economics.rs, and their contracts all exist
  [VERIFIED]. The dossier's next-authoring pass should re-check every "grep-verified" row.
  Every market figure in §15 V-5/V-6 is 2-8 months old. The 2026-07-10 doc's own instruction stands:
  re-check preview/experimental labels, product availability, model support, and pricing before any
  external use.
  The GTM playbook's raw run outputs (wf_e1b0804e-061), the FOCUS run (wf_f0ba2d1a-31c), the TRUST-MAX
  run (wf_826d66d4-0e6), the DIRECTION run (wf_e2948fab-3a8), and the EFFICIENCY runs (wf_c9bb84b6-88c,
  wf_eea93786-8e7, wf_e6a7e493-64e, wf_8611a1e0-c20) are all GC'd temp dirs. These digest files are the
  ONLY surviving record. That is a real single-point-of-failure on the evidence chain: nothing here can
  be re-derived from its raw research; it can only be re-run from scratch.

==============================================================================
17. THE SHARPEST UNRESOLVED ITEMS (what a decision session should take up)
==============================================================================
  U-1. The number has been un-blocked since 2026-06-05 (Tier-1 static: no GPU, no cloud, no sandbox,
       "runnable NOW") and has not been run. Every strategic doc since 2026-06-01 terminates on it.
       PS-1 = RUNNER_PRESENT [VERIFIED]. This is the whole game and it is idle.
  U-2. Agency-as-client (2026-05-19) vs product-to-agencies (2026-06-02) was never adjudicated. The
       older doc's base rates are unrefuted; the newer doc's own ceiling agrees with them.
  U-3. Distribution has had no plan unit for 6.5 weeks despite being named "the biggest UNSOLVED
       problem" and "the constraint with no current answer."
  U-4. SOC 2 (3-6mo lead) and escrow — the two long-lead clocks the GTM playbook said to START ON DAY
       ONE — are unstarted and un-owned; escrow's only home (M-5c) is behind GATE M-0, i.e. dead last.
  U-5. The 2026-07-10 doc's six P0 design flaws are unaddressed. P0-2 (canonicalize does not do shell
       expansion) and P0-3 (a Merkle root is not a signature) both attack shipped/near-shipped units
       (G-3, G-8) and both would falsify a live product claim if a buyer's security team looked.
  U-6. Breadth vs proof. The 07-10 doc says shrink; O/P/Q keep adding surface. K's own premise says "The
       number is the product; everything else in the corpus waits behind A-1..A-4 + K-3." The corpus has
       not obeyed its own rule.

End of 2-direction-monetization.md.

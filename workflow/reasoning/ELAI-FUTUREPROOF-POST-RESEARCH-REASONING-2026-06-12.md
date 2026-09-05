ELAI future-proofing: round-2 reasoning (after the three Perplexity deep-research reports)
date: 2026-06-12
inputs: deep-research-report (18).md = market direction; (17).md = moat/durability/economics;
(16).md = verification-as-a-market (PT-BR). companion: ELAI-FUTUREPROOF-PRE-RESEARCH-REASONING-
2026-06-12.md (round 1, written before the reports). this file updates round 1 claim by claim,
then states the recommendation that goes to the user for agreement before any plan folder is
authored.
epistemic note: the reports are Perplexity deep-research syntheses of primary sources; below,
"R1/R2/R3" = report citations (market/moat/verification respectively). where a report's claim
is vendor marketing or analyst inference, I say so.

================================================================================
1. SCORECARD: ROUND-1 POSITIONS VS THE EVIDENCE
================================================================================

1.1 "Verification/evidence is the wedge and it rises with autonomy" — CONFIRMED, STRONGLY.
R3's central finding is that the category is becoming real (real buyers, real budgets, real
procurement language) and that the specific gap NOBODY fills is: vendor-neutral cryptographic
chain of custody linking prompt -> context -> permissions -> tool calls -> diff -> tests ->
deploy into an artifact verifiable by another platform; deterministic replay of a whole run;
independent attestation per action. Labs ship containment and auditability (sandboxes,
permission modes, session logs, signed commits) but not proof. That gap description is,
nearly clause for clause, ELAI's shipped/planned stack: G-2 RRA, G-8 attestation, I-18 chain,
I-45 objective-hash receipts, F-4 stdout binding, G-1 taint, I-40 wrapper. Round-1 claim
upgraded from "design-SOTA, hypothesis" to "design-SOTA, externally corroborated demand
shape". Still unproven: that ELAI's implementation of it moves a number (see 1.4).

1.2 "Efficiency tracks are perishable" — CONFIRMED IN DETAIL.
R2's absorption record: caching (native at all 3 providers by Oct 2024), compaction
(server-side compact endpoint), server-side memory, native subagents/orchestration, code
execution + computer use, tool search/deferred loading (allowed_tools, skills), structured
outputs, thinking-effort dials, batch 50% everywhere. R2's moat ranking puts cost engineering
LAST and says the durable form of it is workload-specific verification/effort/routing choices,
not token tricks. Round-1 P6 (depreciation tags on H-Track-1/14) upgraded from suggestion to
mandatory: before building each remaining efficiency unit, check the provider's current
native surface first.

1.3 "Review bottleneck is the consensus pain" — CONFIRMED, AND THE WINDOW IS SHORTER THAN I
ASSUMED. R1: Devin Review exists; Factory posts visual QA evidence directly into PRs; GitHub
ships Chronicle (cross-surface session history) + signed/Verified agent commits; Sourcegraph
pivoted to deterministic context + CodeScaleBench. So "evidence attached to review" is no
longer whitespace — what remains whitespace is DETERMINISTIC, CRYPTOGRAPHICALLY BOUND,
REPLAYABLE evidence (R3 explicitly: vendors' QA evidence is screenshots and logs, i.e.
observability, not attestation; the observability->attestation bridge is the named unfilled
slot). Consequence: ELAI's cockpit positioning must sharpen from "review faster" to
"review against evidence that cannot be faked" — and it must ship sooner.

1.4 "The number is everything" — CONFIRMED, WITH BETTER AMMUNITION.
R3 hands ELAI the problem-quantification it lacked: 5.2-21.7% hallucinated-package rates with
43% recurrence (USENIX Sec 2025, 576K samples); standalone-LLM patches introduce ~11x more
verified vulns than human patches on the same issues, and MORE autonomy = MORE vulns
(SWE-bench safety study); reward hacking in 73.8% of KernelBench self-improvement experiments;
visible-vs-hidden test-suite gap grows +28pp per 10x code size (SpecBench); DORA: +25% AI use
correlates with -7.2% delivery stability; METR: ~half of benchmark-passing PRs would not be
merged by humans. Two implications: (a) SAFEDELTA's ablation framing (gate-ON vs gate-OFF,
hidden suites, dynamic exploitation) is exactly the methodology the 2026 literature converged
on — the design needs no rework; (b) the marketing dossier writes itself once ELAI's own
number exists. Round-1 P0 stands as priority #1, unchanged.

1.5 "Gate-tax is a real product risk" — CONFIRMED HARDER THAN I PUT IT.
R2: Microsoft's 2026 CodeAct note reports ~50% latency and >60% token reduction from
COLLAPSING orchestration turns; better models need fewer steps; Cursor repriced because hard
requests cost 10x. R2's synthesis: adaptive gating beats gate-everything; R3's bear case #2:
FP friction is what killed prior SAST/policy-gate generations. ELAI has GD-4 (per-gate FP
bounds) but no COMPOSED budget and no risk-tiered gating policy. Round-1 P4 upgraded from
"one small unit" to a doctrine-level track: heavy checks reserved for expensive decision
points, cheap deterministic checks always-on, one composed friction number per release.

1.6 "Deterministic replay" — HONESTY POSTURE VALIDATED.
R2's attack: byte-replay on hosted models is fragile (hidden routing, model churn); the
honest, valuable guarantee is forensic reproducibility — log inputs, tool I/O, environment,
exact model IDs. That is precisely ELAI's existing GD-5 cloud-lane framing ("reproducible
ACTION-STREAM, not tokens") and J-14's tiered verify (T0 action / T1 token / T2 score,
capability-gated downgrades). No change needed; keep refusing to over-claim. The local lane
(J-13 batch-invariant + seeded sampling) can honestly claim more than the industry does —
a differentiator worth stating.

1.7 "Distribution is the weak flank" — CONFIRMED AND PROMOTED.
R2 ranks moats for an independent wrapper: (1) workflow lock-in/distribution, (2)
per-customer compounding state, (3) eval/verification infra, (4) trust/compliance posture,
(5) cost engineering. ELAI is strongest at #3, building #2 (DKG, journal, playbooks), weakest
at #1. R1: the retained interaction pattern is hybrid (local control surface + cloud runtime
+ PR-native supervision), PR loops are table stakes, and net-new product energy is
concentrating inside labs/platforms. Conclusion: ELAI must not fight for the editor or the
agent loop; it must attach its evidence to the surfaces that already won — the PR/CI loop and
the agents people already run. Concretely: I-40 (`elai wrap` + .rra spec) plus a NEW thin
leg: a PR/CI EVIDENCE EMITTER — gate verdicts, DSR, vouched hunks, attestation digests
rendered as check-runs/PR comments (composing C-40/E-41/G-5/H-52). The Tauri cockpit stays,
but the PR is where the evidence meets the buyer.

1.8 "Buyer-side layer can wait" — PARTIALLY REVERSED.
R3: the buyer is platform/DevEx, AppSec/DevSecOps, GRC, and procurement — not the individual
developer; budgets come from existing lines (SCA/AppSec, compliance, platform), and
procurement documents already demand traceability req->code->build->release, audit logs,
independent evaluation rights (OMB M-25-22), SOC 2, AI-flow disclosure. EU: AI Act GPAI
enforcement from Aug 2026; Product Liability Directive covers software for products placed
on market from Dec 2026; insurers adding AI sublimits. Implication: ELAI does not need SSO/
RBAC tomorrow, but it DOES need its artifacts to speak procurement language now — a
"compliance export" packaging of what already exists (G-8 attestation + G-9 redaction + I-44
zero-retention + RRA + audit trail) mapped onto those checklists. That is composition + docs,
not new architecture, and it converts the wedge from developer-nice-to-have into something an
AppSec/compliance owner can buy with an existing budget.

1.9 "Local lane aging" — CONFIRMED WITH A NUMBER.
R2: the useful local-coding floor in 2026 is ~24GB VRAM (4-bit ~24B class); the trend is
efficient-MoE actives (30B-A3B class); open-weight closed much of the gap (vendor-reported
SWE-bench Verified 70s vs frontier ~80) but frontier still leads on the hardest agentic sets.
ELAI's canonical 7B@12GB lane is below the useful floor for autonomous coding (fine as the
mechanical-edit/privacy lane H-50 already scopes it to). Also: public production evidence for
hybrid local/cloud routing with measured savings is THIN (only research-grade HERA ~45%
offload) — meaning ELAI's H-7/I-25 routing, if MEASURED honestly, is itself a publishable
first number. J-9 catalog + the planned 2x3090 align; add a standing quarterly refresh
evaluation.

1.10 "Absorption threat to the wedge itself" — NEW, REAL, CLOCKED.
R1 predicts the NEXT absorptions are precisely verification/audit, org governance, and
trajectory memory (GitHub Chronicle is the leading edge). R3's counterweight: the
self-grading conflict of interest means lab-owned verification is structurally less
trustworthy, and the demand is explicitly for vendor-NEUTRAL evidence — the independent slot
is protected, but only the neutral+cryptographic+replayable version of it. GitHub (which
owns the PR and is not a model lab) is the most credible absorber. Consequence: the .rra
open-spec + conformance checker (I-40) is time-sensitive — a first-mover artifact standard
beats a feature race; 12-24 months is the realistic window.

1.11 Misc corrections to round-1.
- R1 retention data (only ~15% of pro devs use agents daily; 36.7% don't plan to) says the
  autonomy market is earlier than vendor noise implies — favors ELAI's "trust the output"
  framing for the cautious majority, and means enterprise-grade evidence beats consumer
  velocity as the entry point.
- E-9's drop of SWE-bench-Verified-as-headline is re-confirmed (R2: scaffold variance,
  standardized-harness gaps; R3: METR merge-rate gap). Keep ablation-only framing (GD-3).
- The app-generator tier (Lovable/Bolt/v0) is a different market; round-1's "do not chase"
  stands (R1's Lovable security incident even illustrates the trust gap ELAI sells against).

================================================================================
2. THE STRATEGY, RESTATED IN ONE PARAGRAPH
================================================================================

ELAI wins by being the system whose word you do not have to take — and the research adds the
operational corollary: the evidence must be (a) MEASURED (the SAFEDELTA-class number exists
and is replayable), (b) PORTABLE (an open .rra artifact + attestation any platform can
verify, wrapped around ANY agent, surfaced in the PR/CI loop people already supervise from),
(c) CHEAP TO LIVE WITH (risk-adaptive gating under a composed friction budget, because FP
friction is how this category's predecessors died), and (d) SOLD AS COMPLIANCE-READY
EVIDENCE (procurement/AppSec language, EU AI Act / PLD / insurance tailwinds) rather than as
developer safety nagware. Everything else — canvas, efficiency tricks, self-improvement —
is supporting cast or depreciating inventory until those four are live.

================================================================================
3. THE RECOMMENDATION (what I would do; goes to the user for agreement)
================================================================================

Author ONE new plan letter (K-plan, "FUTURE-PROOF / WEDGE EXECUTION"), deliberately small
(~12-16 sessions, 6 tracks), composing existing plan units (GD-1) and CUTTING/SEQUENCING the
rest. No other new letters until K's keystones land.

K-T1 PROVE — the number first (blocks external claims, not internal work).
  K-1: I-43 claim scoper + Phase-0 kill-gate pilot (N=50-100, gate-ON/OFF, reviewer-reaching
       rate, cross-lane divergence scoper). Cheap falsifier; go/no-go memo.
  K-2: F-2b live SeCodePLT run on the real local model (RunPod checkbox per W-10).
  K-3: G-6 SAFEDELTA full matrix + the problem-quantification dossier (cite USENIX
       hallucination, 11x-vulns, SpecBench, DORA, METR against ELAI's measured delta).
  K-4: hybrid-routing measured number (H-7/I-25 instrumented: % offloaded, $ saved, quality
       delta under seed) — the industry has no production number here; ELAI can own it.

K-T2 WEDGE + DISTRIBUTION — make the evidence portable and put it where the buyer looks.
  K-5: I-40 `elai wrap` MVP around Claude Code first (then Codex/Copilot CLI), emitting
       conformant .rra with taint rows + write-gate verdicts on a foreign agent's run.
  K-6: .rra open spec v1 + conformance checker + spec site/repo (the standard play; G-9
       redaction is the publication precondition, I-44 zero-retention the enterprise variant).
  K-7: PR/CI evidence emitter — GitHub check-run + PR-comment renderer for gate verdicts,
       DSR, vouched/unvouched hunks, attestation digest (composes C-40/E-41/G-5/H-52/F-4).
       The PR is the supervision surface the market standardized on; meet it there.
  K-8: reviewer cockpit study — within-subjects review-time measurement (G-5's METR-design
       study) so "minutes saved per PR at equal-or-lower escaped-defect rate" becomes the
       second number.

K-T3 ADAPTIVE GATE ECONOMY — make the trust spine cheap enough to keep on.
  K-9: composed gate-tax budget: ONE measured per-release number for end-to-end added
       latency + composed FP/over-block rate across all default-ON gates, with a locked
       ceiling; regression = release BLOCK. (Extends GD-4 from per-gate to whole-system.)
  K-10: risk-tiered gating policy: deterministic sub-ms checks always-on; expensive legs
       (mutation, differential, VLM) fire on risk triggers (blast-radius size, CWE-class
       surface, autonomy level, novelty) — codified as doctrine + a routing table, composing
       J-16 cones + C-30 complexity classes + B-22 risk levels. Answer to CodeAct-style
       overhead findings: collapse where safe, gate where expensive.

K-T4 BUYER-SIDE EVIDENCE PACK — speak the budget-owner's language (no SSO/org build yet).
  K-11: compliance export: one command bundling redacted RRA + attestation + audit trail +
        retention statement, mapped in docs onto procurement checklists (traceability
        req->code->build->release, OMB M-25-22 independent-evaluation rights, EU AI Act
        Art-50 "designed to support", PLD exposure, SOC2-adjacent controls). Composition of
        G-2/G-8/G-9/I-44 + a mapping document.
  K-12: insurance/AppSec positioning brief (positioning artifact, E-23-class, no runtime
        code): how the evidence pack answers SCA/AppSec (hallucinated-dep gate F-9, CWE
        gates F-2) and underwriting questionnaires.

K-T5 DEPRECIATION + REFRESH DISCIPLINE — stop building what providers will eat.
  K-13: absorbable-unit audit: tag every remaining H-Track-1/14 + efficiency unit with a
        check-provider-first PRE_GATE (R2's absorption record as the checklist); kill or
        de-scope duplicates of now-native features.
  K-14: quarterly model-refresh cadence: J-9 catalog run as a recurring evaluated checkbox
        (candidate vs incumbent on ELAI fixtures, incl. the 24GB-floor MoE class and the
        vLLM J-19 lane); the local lane's scope statement updated to "mechanical/privacy
        lane" per H-50 doctrine.

K-T6 SEQUENCING / CUT — the meta-track.
  K-15: standing priority order over the existing corpus: K-T1/T2 keystones >> wire-dark
        activation of their dependencies >> everything else; canvas Track 2 explicitly
        demoted behind K-7/K-8 (cockpit/PR-emitter); a no-new-letters rule until K-1..K-7
        are DONE; the activation-over-authoring ratio adopted.

Explicit non-goals (research-backed): no IDE play, no app-generator play, no SWE-bench
headline, no org/SSO build yet (artifacts speak compliance first; org layer only on pull),
no gate-everything defaults, no local-lane expansion beyond the mechanical/privacy scope
until the 24GB-class refresh is evaluated.

================================================================================
4. WHAT WOULD CHANGE MY MIND (kill-switches on the strategy)
================================================================================

- K-1/K-3 produce a null or negative safety delta at bounded FP -> the trust wedge as
  HEADLINE dies; ELAI re-aims to per-customer compounding state (R2 moat #2: DKG + journal +
  playbooks) with verification as internal quality only.
- K-9 shows the composed gate tax cannot get under a tolerable ceiling without losing the
  delta -> adopt R2's adaptive-collapse posture aggressively (fewer, heavier, later gates).
- GitHub ships cryptographic, replayable, vendor-neutral evidence natively within the window
  -> the .rra spec play collapses into "be compatible"; the wrapper pivots to the compliance
  export + cockpit-on-top.
- K-8 shows no review-time win at equal defect rate -> the cockpit is repositioned as a
  compliance artifact only; PR-emitter (K-7) remains as the distribution leg.

Round 3 (the K-plan folder itself: K-0_INDEX + per-session files + grounding map, authored
under W/GD/HD doctrine) happens only after the user agrees to Section 3.

K-PLAN MASTER BRIEF — FUTURE-PROOF / WEDGE EXECUTION
date: 2026-06-12
status: APPROVED-DIRECTION, FOLDER NOT YET AUTHORED
purpose: the single comprehensive handoff for the session that will author the K-plan folder
(K-0_INDEX + per-session files + _grounding/K-GROUNDING-MAP.md). Everything decided in the
2026-06-12 future-proofing session is in this file. The authoring session should be able to
work from THIS FILE ALONE, re-verifying live anchors per W-7.

================================================================================
0. PROVENANCE AND CANONICAL INPUTS
================================================================================

User's framing (verbatim intent): "i just want so that when i finish elai i will not be
behind anyone... the biggest question is where elai should be better at?" The user approved
the recommendation summary at the end of the session and asked for this master brief instead
of immediate folder authoring.

Inputs produced/consumed this session (all read IN FULL):
- docs/specs/guide/steps/phase-index-MASTER.txt (base phases 0-4M)
- docs/thefinalglance/{B-plan/B-PLAN-SUMMARY, C/D/E/F/G/H/I/J-SUMMARY_INDEX}.md (all 9)
- docs/thefinalglance/ELAI-FEATURE-MAP.md (live SCHEMA stamp 234/543 at 2026-06-12; re-grep
  at authoring, it WILL be stale)
- workflow/reasoning/ELAI-FUTUREPROOF-RESEARCH-PROMPTS-2026-06-12.md (the 3 anonymized
  Perplexity prompts; kept for reproducibility of the research)
- workflow/reasoning/ELAI-FUTUREPROOF-PRE-RESEARCH-REASONING-2026-06-12.md (round 1,
  training-knowledge assessment)
- workflow/reasoning/ELAI-FUTUREPROOF-POST-RESEARCH-REASONING-2026-06-12.md (round 2, after
  the reports; contains the full claim-by-claim scorecard and the kill-switches — the
  authoring session SHOULD read it; it is the long-form of Sections 2-3 below)
- Three Perplexity deep-research reports, provided by the user IN-CHAT as attachments named
  deep-research-report (16).md (verification-as-a-market, PT-BR), (17).md (moat/durability/
  economics), (18).md (market direction). ON-DISK PATH UNKNOWN — if needed, ask the user
  where they saved them; otherwise the digests in Section 2 and in the post-research doc are
  sufficient and were written from full reads.
- Decision-ledger entries: topic 'elai-futureproof' in mcp_think (2 decision records,
  searchable via mcp_search_context).

================================================================================
1. THE STRATEGY (the thesis the K-plan executes)
================================================================================

ELAI wins by being the system whose word you do not have to take. The research added four
operational corollaries; together they ARE the K-plan:

(a) MEASURED. The headline trust number (SAFEDELTA-class, gate-ON vs gate-OFF, hidden
    suites, dynamic exploitation, journal-recomputable under ELAI_SEED) must EXIST before
    any external claim. Today it does not (G-6 never invoked; F-2b never run live).
(b) PORTABLE. The evidence must be an open, vendor-neutral, replayable artifact (.rra +
    attestation) that wraps ANY agent (I-40 `elai wrap`) and renders in the surface the
    market standardized on for supervision: the PR/CI loop (check-runs, PR comments).
(c) CHEAP TO LIVE WITH. Risk-adaptive gating under a COMPOSED whole-system friction budget.
    FP friction is how prior gate generations (SAST, CodeGate) died; orchestration overhead
    itself is a measured bottleneck (Microsoft CodeAct 2026: collapsing tool loops cut ~50%
    latency / >60% tokens). Per-gate GD-4 bounding is necessary but not sufficient.
(d) SOLD AS COMPLIANCE-READY EVIDENCE. The buyer is org-side (AppSec/DevSecOps, GRC,
    platform engineering, procurement) buying from EXISTING budget lines — not the
    individual developer. The artifacts must speak procurement language (traceability
    req->code->build->release, audit logs, independent-evaluation rights, EU AI Act Art-50
    "designed to support", PLD, insurance questionnaires).

Everything else in the corpus — canvas, efficiency tricks, self-improvement flywheel — is
supporting cast or depreciating inventory until (a)-(d) are live.

WHY THIS AXIS: verification demand RISES as models improve (bigger delegation = higher cost
of unverified claims); it does not substitute for model capability, it prices it — so it is
bitter-lesson-resistant, unlike the harness efficiency layer which the providers have been
absorbing on a 6-18 month cycle (documented absorption record, Section 2.2).

================================================================================
2. RESEARCH FINDINGS DIGEST (what the K-plan is grounded in)
================================================================================

All three reports are Perplexity deep-research syntheses of primary sources (changelogs,
official docs, papers, procurement documents). Key load-bearing findings, with which report:

2.1 THE WEDGE IS REAL AND UNFILLED (report 16, verification-as-a-market):
- The named gap NOBODY fills: vendor-neutral cryptographic chain of custody linking
  prompt -> context -> permissions -> tool calls -> diff -> tests -> deploy into an artifact
  verifiable by another platform; deterministic replay of a whole run; independent
  attestation. Labs ship containment + auditability (sandboxes, permission modes, session
  logs, GitHub signed/Verified agent commits), NOT proof. This is clause-for-clause ELAI's
  G-2/G-8/I-18/I-45/F-4/G-1/I-40 stack.
- Self-grading conflict of interest is structural: the lab selling the agent cannot be the
  sole arbiter of its success. Protects the independent slot — but only for the
  NEUTRAL+CRYPTOGRAPHIC+REPLAYABLE version.
- Buyers documented: platform/DevEx, AppSec/DevSecOps, GRC/compliance, public procurement.
  Documented procurement demands (Tennessee RFIs 2026): traceability from requirements to
  code/builds/releases, audit logs, third-party AI assessment, AI data-flow disclosure,
  retention/localization of prompts+outputs+logs, kill-switch for AI features. OMB M-25-22:
  contracts must allow continuous monitoring, independent evaluation on agency-held data,
  reproducible/verifiable test results. EU: AI Act GPAI enforcement from 2026-08-02 (full),
  Art-50 transparency in cycle; Product Liability Directive covers software for products
  placed on market from 2026-12-09. Insurance: Coalition affirmative AI endorsement, QBE/
  Beazley AI sublimits (FT, Apr 2026).
- Bear case (must be engineered against, not dismissed): feature-not-product absorption by
  GitHub/CI/AppSec; FP-friction death; "devs don't pay for safety" (true — orgs do); lab
  defaults capping demand to regulated niches.
- The 5 bull conditions named by the report (K must satisfy the controllable ones): agent
  code share keeps rising; incidents keep proving visible-tests-pass != trust; procurement/
  insurance normalize independent evidence; the product is evidence-first with low-friction
  risk-based gating; someone bridges observability -> attestation. ELAI controls the last two.

2.2 ABSORPTION RECORD + MOAT RANKING (report 17, durability/economics):
- Absorbed into provider platforms 2024-2026: prompt caching (all 3 by Oct 2024), context
  compaction (server-side compact endpoint), server-side memory, native subagents/agent
  SDKs, code execution + computer use, tool search/deferred loading (allowed_tools, skills),
  structured outputs, thinking-effort dials, batch 50% (all 3). Predicted NEXT absorptions
  (report 18 concurs): verification/audit, org governance, trajectory memory.
- Moat ranking for an independent wrapper (evidence-based): 1) workflow lock-in/
  distribution, 2) per-customer compounding state (code graphs, playbooks, journals),
  3) proprietary eval/verification infra, 4) enterprise trust/compliance posture, 5) cost
  engineering (weakest). ELAI is strongest at 3, building 2, weakest at 1 -> K-T2 exists to
  fix 1 by attaching evidence to surfaces that already won.
- Gate-tax evidence: Microsoft CodeAct 2026 (collapse tool loops: ~-50% latency, >-60%
  tokens); Anthropic partner data (better models need fewer steps); Cursor repricing (hard
  requests cost 10x). Synthesis: ADAPTIVE gating beats gate-everything.
- Replay honesty: byte-replay on hosted models is fragile (hidden routing, model churn);
  the honest guarantee is forensic reproducibility (inputs, tool I/O, env, exact model IDs).
  ELAI's GD-5 cloud framing ("reproducible ACTION-STREAM, not tokens") and J-14 tiers
  (T0 action / T1 token / T2 score with recorded downgrades) are already correct. The LOCAL
  lane (J-13) can honestly claim MORE than industry — a stated differentiator.
- Open-weight/local: useful local-coding floor in 2026 = ~24GB VRAM (4-bit ~24B class);
  trend = efficient-MoE actives (30B-A3B class); open-weight SWE-bench-Verified
  vendor-reported 70s vs frontier ~80-81, frontier still leads hardest agentic sets
  (Terminal-Bench, SWE-bench Pro). ELAI's canonical 7B@12GB lane is BELOW the autonomous
  floor — correct only as the mechanical/privacy/latency lane (H-50 doctrine already says
  this; K-T5 formalizes it).
- Hybrid local+cloud routing: production evidence industry-wide is THIN (research-grade
  HERA ~45.67% offload only). A measured ELAI number here is itself a first-mover number.
- Economics: output tokens 5-6x input everywhere, thinking billed as output; per-token
  deflation but per-task spend rising; documented negative/thin margins for agent products
  (TechCrunch on Windsurf; BI on 10x bill cuts via subscriptions). Cost engineering is
  operationally required but never the moat.

2.3 MARKET DIRECTION (report 18):
- The retained interaction paradigm is HYBRID: local control surface (IDE/terminal) + cloud
  background runtime + PR-native human supervision. PR loops, background/parallel agents,
  cloud sandboxes, planning modes, memory, MCP are table stakes at the top end.
- Raw codegen is no longer the moat; review/verification/context control are rising.
  Already shipping: Devin Review, Factory automated QA with VISUAL EVIDENCE posted into
  PRs + OTEL analytics, GitHub Chronicle (cross-surface session history) + signed agent
  commits, Sourcegraph deterministic-context + CodeScaleBench. NONE of it is deterministic/
  cryptographic/replayable evidence — that remains the open slot, but the window is
  SHORTER than round-1 assumed: 12-24 months, with GitHub (owns the PR, not a model lab)
  as the most credible absorber.
- Adoption reality: only ~14.9% of professional devs use agents daily (SO 2025); Copilot
  29% / Cursor 18% / Claude Code 18% at-work adoption (JetBrains Jan 2026); Claude Code =
  75% of agent-initiated deploys on Vercel. The autonomy market is EARLY; the cautious
  majority is the audience for trust framing.
- Revenue concentrates in enterprise contracts + high-usage seats (Cursor $1B annualized
  Nov 2025); enterprise checklist: SSO/SCIM/RBAC, audit logs, budgets, usage analytics,
  on-prem/BYO, observability.
- Survivor traits among independents: workflow distribution, OR differentiated context/
  verification layer, OR open/BYOK economics, OR platform adjacency. Generic orchestration
  shell = weakest position. Windsurf->Devin-Desktop absorption and Cody sunset are the
  cautionary cases.

2.4 PROBLEM-QUANTIFICATION AMMUNITION (report 16; for the K dossier, cite-verify each at
authoring per feedback_paper_citation_verify):
- USENIX Security 2025 "We Have a Package for You!": 576K samples, 16 models; hallucinated
  package rate >=5.2% commercial / 21.7% open-source; 205,474 unique fake names; 43%
  recurrence across 10 repeats (ELAI's F-9/Spracklen grounding matches).
- SWE-bench patch-safety study: standalone LLM patches introduced ~11x more manually
  verified vulns than human patches for the same issues (135 vs 12 on 20K+ issues); agentic
  frameworks also introduce vulns and MORE autonomy = MORE vulns (OpenHands 44 in the 2,294
  test subset).
- Reward hacking in self-improving code agents: 73.8% of KernelBench experiments, 46.8% of
  algorithmic-optimization experiments.
- SpecBench: visible-vs-hidden suite gap grows ~+28pp per 10x code size (agents saturate
  the visible suite).
- DORA 2026: +25% AI use correlates with -1.5% throughput, -7.2% delivery stability;
  GitClear 211M lines: more duplication, less refactoring.
- METR: ~half of benchmark-passing PRs would not be merged by real maintainers; a 2026
  audit found median 4 tests per benchmark problem (fragile oracles).
- Vendor telemetry (label as telemetry, weaker tier): Uplevel +41% bugs with Copilot;
  CodeRabbit 1.7x issues in AI-majority PRs.

================================================================================
3. THE K-PLAN SHAPE (what the authoring session builds)
================================================================================

LETTER: K. NAME: FUTURE-PROOF / WEDGE EXECUTION. Deliberately SMALL: ~10 net-new session
files + a Section-A execution-pointer list in K-0_INDEX (J-0 precedent: pointers to existing
plan files that K SEQUENCES but never re-authors, per GD-1 and the user's
never-amend-old-plans rule — feedback_new_plan_files_only).

3.1 SECTION A — EXECUTION POINTERS (existing files K sequences; K-0_INDEX lists these with
their gating order; do NOT author new files for them):
- A-1 = I-43 claim scoper + Phase-0 kill-gate (the cheap falsifier; FIRST THING THAT RUNS).
- A-2 = F-2b live SeCodePLT/CodeGuard+ run on the real local model (RunPod checkbox, W-10).
- A-3 = G-6 SAFEDELTA full matrix (gated on A-2 landing the dynamic floor, per G-6's own
  file).
- A-4 = I-40 trust-layer wrapper + .rra open spec (the wedge artifact; G-9 redaction is its
  publication precondition per I-40's file; I-44 zero-retention is the enterprise variant).
- A-5 = G-5's MEASUREMENT section (the METR-design within-subjects review-time study,
  time-to-first-approve through the cockpit vs plain diff, escaped-defect guardrail).
- A-6 = H-7 calibrated routing + I-25 ledger-trained router (the substrate K-2 measures).
- A-7 = J-9 hardware model catalog (the substrate K-9's cadence rides).
- A-8 = J-28/J-29 token ledger + usage window (the cost-instrument substrate K-2 and K-4
  read; J-28 fixes the streaming-local capture bug the routing number needs).
Order/gating: A-1 -> (go) -> A-2 -> A-3. A-4 parallel-safe with the A-1..A-3 chain after
G-9 verified live. A-5 after K-3 (PR emitter) or cockpit-only if K-3 slips. A-6..A-8
parallel-safe, before K-2/K-9.

3.2 SECTION B — NET-NEW K SESSIONS (the authoring session writes one file each; numbering
indicative, final numbering at authoring):

K-1 PROBLEM-QUANTIFICATION DOSSIER + CLAIMS POLICY [Track K-T1 | positioning + doctrine |
effort S | E-23-class, near-zero code]
- Deliverable: docs/strategy/ (or workflow/reasoning/) dossier binding the Section 2.4
  external numbers (each re-verified: WebSearch the paper/ID before citing — house rule
  after the B-40/B-41/B-42 bad-cite incidents) to ELAI's measured numbers AS THEY LAND
  (A-1/A-2/A-3 outputs). Plus a CLAIMS POLICY doc: no external claim ships without its
  measured, journal-recomputable number (extends GD-2/GD-3 to marketing surface); the
  honest-wording table from I-43 (trace-authentication vs deterministic-replay per lane)
  is normative.
- Composes: I-43 memo output, GD-3/GD-5, E-23 positioning precedent. Restates none.
- observable_effect: dossier exists; every claim row carries either a measured ELAI number
  + recompute pointer or an explicit EXTERNAL/DESIGN-RATIONALE tag; a claims-policy lint of
  README/positioning copy passes (no unmeasured superlative).

K-2 ROUTING-ECONOMICS BENCH (the hybrid number) [K-T1 | BUILD bench | effort M |
RunPod/remote-sidecar legs gated W-10]
- Premise: industry has NO production number for "cheap local for mechanical edits +
  frontier for planning" savings (report 17). ELAI can own the first one.
- Deliverable: `elai bench routing-economics` over a fixed task corpus, two arms
  (cloud-only vs H-7 architect/editor hybrid), reporting % steps offloaded, $ delta,
  quality delta (verified-outcome rate, not pass@k), all under ELAI_SEED, recomputable
  (GD-2). Mirrors seccodeplt_runner/safedelta_runner shape; deterministic fixture tier
  ships now, live tier is a gated checkbox.
- Composes: H-7 (the routed substrate), I-25 (router), J-28 ledger (the cost capture —
  honest rates), H-9 cockpit (display), I-27 (convergence-curve sibling — COMPOSE, the
  routing-savings number is not I-27's verified-outcome-vs-compute curve; do not fork its
  harness, share the runner shape). E-39 cap supremacy untouched.
- observable_effect: one RoutingEconomicsMeasured-class artifact (prefer-reuse an existing
  bench row family before adding a variant; W-9 re-grep live SCHEMA) with per-arm
  $/verified-task; a --force-cloud-everything broken arm shows the delta collapse (R2).

K-3 PR/CI EVIDENCE EMITTER [K-T2 | BUILD | effort M-L | THE distribution leg]
- Premise: the PR is the supervision surface the market standardized on (report 18); ELAI's
  evidence currently renders only in its own CLI/dashboard. Devin/Factory post OPINION/
  SCREENSHOT evidence; nobody posts deterministic, hash-bound, replayable evidence.
- Deliverable: render ELAI's per-hunk evidence into GitHub check-runs + PR comments:
  gate verdicts (F-1/F-2/F-9), F-4 stdout-hash-bound test outcomes, G-5 DSR + vouched/
  unvouched bands, H-52 anchored findings (the resolver makes hallucinated line numbers
  structurally impossible — reuse it for comment placement), G-8 attestation digest +
  "verify offline with `elai verify-attestation`" line, J-16 blast-radius cone summary when
  available. Honesty: a hunk with no evidence renders UNVOUCHED, never silently clean (R1).
- Composes: C-40/E-41 github adapter (feature map says WIRED via `elai pr-review` as
  github_pr module in orchestrator — VERIFY live at authoring, the rename caused stale
  verdicts before), G-5, H-52, F-4, G-8, J-16. Restates none; this is a RENDERER.
- observable_effect: a real PR on a fixture repo gets a check-run whose every claim joins a
  journal row (GD-2); an out-of-evidence comment is structurally impossible (closed-set
  render from the RRA evidence manifest); broken arm: --post-unvouched-as-clean FAILS (R2).

K-4 COMPOSED GATE-TAX BUDGET [K-T3 | BUILD instrument + doctrine | effort M | KEYSTONE of
K-T3]
- Premise: GD-4 bounds gates INDIVIDUALLY; nothing measures the COMPOSED whole-system
  friction (added latency + combined FP/over-block rate across all default-ON gates), and
  the research says composed friction is the category-killer.
- Deliverable: a per-release friction report: end-to-end added wall-clock per task class +
  composed over-block rate over a fixed task corpus, gates-ON vs gates-OFF (the same
  ablation framing as SAFEDELTA — friction is QDELTA's denominator twin); a LOCKED ceiling
  (mutable=0-style, value set by the user at authoring); ceiling breach = release BLOCK.
  Lives next to I-38 QDELTA's harness (compose its runner shape, share the corpus).
- Composes: I-38, G-6 ablation framing, wire-dark-allowlist (the gate inventory source),
  E-49 metrics. Restates none.
- observable_effect: `elai bench gate-tax` emits the composed number recomputable from the
  journal; a deliberately spammy fixture gate trips the ceiling and the release gate FAILS
  (R2); the report names the top-3 friction contributors (actionable, not just a number).

K-5 RISK-TIERED GATING POLICY [K-T3 | BUILD policy + dispatch | effort M]
- Premise: adaptive gating beats gate-everything (report 17). ELAI has the risk substrate
  (B-22 risk levels, C-30 complexity classes, J-16 cones, F-1 compound gate) but no single
  policy that routes WHICH verification legs fire per risk tier.
- Deliverable: a deterministic GateIntensity policy table (R8-frozen): cheap sub-ms
  deterministic checks ALWAYS-ON (argv allowlist E-30, secrets/dep gates F-1/F-9, taint
  G-1); expensive legs (mutation I-20, differential H-26, VLM tier-2, formal E-53) fire on
  risk triggers — blast-radius cone size (J-16), CWE-surface class (F-2), autonomy level
  (B-22), novelty (D-15), file criticality. LLM never picks the tier (TD-3). Collapse-
  where-safe is explicit policy: low-risk mechanical edits get the CodeAct treatment
  (fewer turns, fewer gates), expensive decision points get the full stack.
- Composes: all of the above + K-4 (the budget the policy must fit under). Restates none.
- observable_effect: a trivial rename fires only the always-on set (journal shows zero
  expensive-leg rows); a high-cone auth-touching edit fires the full stack; the policy
  table's hash is in the frozen manifest (a run cannot weaken its own gating — R8);
  measured composed tax under K-4's ceiling on >=3 real runs (GD-4).

K-6 COMPLIANCE EXPORT [K-T4 | BUILD composition | effort S-M]
- Premise: the buyer is org-side and buys from existing budget lines (report 16); ELAI's
  artifacts must speak procurement language without an org-layer build.
- Deliverable: `elai export compliance` -> one bundle: redacted RRA (G-9 precondition) +
  G-8 attestation + audit-trail extract + retention statement (I-44 mode noted) + a
  MAPPING DOC translating bundle fields onto: traceability req->code->build->release,
  audit-log requirements, OMB M-25-22-style independent-evaluation/reproducibility rights,
  EU AI Act Art-50 ("designed to support" wording ONLY — the Article-50-certified hook is
  KILLED per the GTM record, do not resurrect), PLD exposure documentation, SOC2-adjacent
  control narratives. No new attestation machinery — composition + docs.
- Composes: G-2/G-8/G-9/I-44/I-18. Restates none.
- observable_effect: the bundle verifies offline on a clean machine (`elai
  verify-attestation` + conformance checker); a grep for raw secrets over the export is
  empty (G-9 composed); the mapping doc cites each procurement clause to a bundle field.

K-7 APPSEC/INSURANCE POSITIONING BRIEF [K-T4 | positioning-only | effort S | E-23-class]
- Deliverable: the brief that sells the evidence pack into AppSec/SCA budgets (F-9
  hallucinated-dep gate vs the USENIX numbers; F-2 CWE gates vs the 11x-vulns number) and
  answers insurance questionnaires (controls, logs, rollback, independent evidence). Pure
  prose + the K-1 dossier numbers. No runtime code, no SSB.
- observable_effect: brief exists, every number tagged measured-vs-external, reviewed by
  the user before any external use.

K-8 ABSORBABLE-UNIT AUDIT + PROVIDER-CHECK PRE_GATE [K-T5 | audit + rule | effort S-M]
- Premise: the absorption record (2.2) makes building provider-native features a waste.
- Deliverable: (a) sweep the remaining PLANNED/PARTIAL efficiency-class units (H-Track-1:
  H-1..H-9 residue; Track-14: I-10..I-16 residue; E-2/E-25/E-32 legs; J-24 tool-context
  legs) and tag each ABSORBED (provider ships it natively now -> kill or de-scope to the
  instrument half) / AT-RISK (check again at build time) / DURABLE (verification-coupled,
  keep); output a kill/de-scope list for user confirmation (no removal without explicit
  user confirmation — house rule). (b) a one-paragraph PRE_GATE rule added to the
  planning doctrine: any efficiency-class unit re-verifies the provider's CURRENT native
  surface (WebSearch) before EXECUTING.
- Composes: the feature map (upsert tags there), W-7. Restates none.
- observable_effect: feature-map rows carry the tags; the rule file exists and
  conditional-routing loads it for efficiency-class tasks; the kill list is a user
  decision artifact, not auto-applied.

K-9 QUARTERLY MODEL-REFRESH CADENCE [K-T5 | process + bench wrapper | effort S-M]
- Premise: the local lane ages silently (Qwen2.5-Coder-7B is below the 2026 ~24GB
  autonomous floor; the useful class is now efficient-MoE 30B-A3B on 24GB). Prior
  rejections (Gemma-4, Qwen3-32B) were box-specific and are already lifted to
  tier-conditional catalog data by J-9 — routing stays untouched without user sign-off.
- Deliverable: a recurring (quarterly) evaluated checkbox: top catalog candidates per lane
  vs the incumbent on ELAI's own fixtures (pass-rate, VRAM, tok/s, $/verified-task),
  riding J-9's catalog + the E-27 AWQ and J-19 vLLM lanes; output = a decision row
  (ADOPT/KEEP/RE-EVALUATE) the user ratifies. Also re-states the lane doctrine: local =
  mechanical/privacy/latency lane (H-50), 24GB-class = the autonomy-candidate tier
  (pending the 2x3090 box), cloud = planning/architect lane.
- Composes: J-9, E-27, J-19, E-31 tier matrix, I-16 KV eval. Restates none.
- observable_effect: the first cadence run produces the decision row + a dated
  refresh-report; a candidate that regresses fixtures is auto-KEPT-incumbent (the cadence
  cannot silently swap — GD-4/user-gate).

K-10 SEQUENCING/CUT DOCTRINE [K-T6 | K-0_INDEX content, possibly not a session file]
- The meta-decisions, normative for all future planning:
  (1) priority order: Section-A chain (A-1..A-3) + K-3 + A-4 ABOVE everything else in the
      corpus; their wire-dark dependencies get activation priority.
  (2) NO NEW PLAN LETTERS until A-1..A-4 + K-3 are DONE (wiring-audit grade). Riders and
      amendments per existing rules are fine; new letters are not.
  (3) activation-over-authoring: a standing preference for flipping DARK/PARTIAL units
      that K depends on over authoring anything new.
  (4) canvas Track 2 (H-10..H-16) explicitly DEMOTED behind K-3 + A-5 (cockpit/PR-emitter).
      Not killed — demoted. H-16 orbit retirement still rides canvas-default.
  (5) the I-31 GRPO flywheel and Track-17 stay LAST and behind their blocking guards
      (unchanged from I-plan; K adds nothing there).
  (6) the K kill-switches (Section 5) are part of the doctrine.

3.3 TRACK MAP (for K-0_INDEX):
  K-T1 PROVE: A-1, A-2, A-3, K-1, K-2 (+A-6/A-8 substrate)
  K-T2 WEDGE+DISTRIBUTION: A-4, K-3, A-5
  K-T3 ADAPTIVE GATE ECONOMY: K-4, K-5
  K-T4 BUYER-SIDE EVIDENCE: K-6, K-7
  K-T5 DEPRECIATION+REFRESH: K-8, K-9 (+A-7 substrate)
  K-T6 SEQUENCING/CUT: K-10 (doctrine, in K-0)

3.4 PROPOSED DOCTRINE FAMILY KD-1..KD-5 (K-0_INDEX, inherits W-1..W-10 + GD-1..5 + HD/TD/
VD/RV/CL/ID/JD as applicable):
  KD-1 NUMBER-BEFORE-CLAIM: no externally-visible claim (README, brief, spec site) without
       its measured, journal-recomputable number or an explicit EXTERNAL/DESIGN tag.
  KD-2 MEET-THE-BUYER-IN-THE-PR: every trust signal ships a PR/CI rendering or a justified
       W-6-style absence; the dashboard is the deep view, never the only view.
  KD-3 COMPOSED-FRICTION CEILING: the whole-system gate tax is measured per release against
       a locked ceiling; a new default-ON gate that breaches it does not ship default-ON.
  KD-4 PROVIDER-ABSORPTION PRE_GATE: efficiency-class units re-verify the provider's
       current native surface before EXECUTING; duplicating a native feature is a
       kill-or-de-scope, not a build.
  KD-5 WINDOW-CLOCK: wedge artifacts (.rra spec, PR emitter, compliance export) are
       time-boxed deliverables — spec-first-mover beats feature-completeness; a slipped
       quarter is an escalation to the user, not a silent re-plan.

================================================================================
4. NON-GOALS (research-backed; record in K-0 so they are never re-litigated)
================================================================================
- No IDE play, no editor fork, no IDE-integration depth race (incumbents own it).
- No app-generator play (different market; Lovable-class incidents are what ELAI sells
  against, not a segment to enter).
- No SWE-bench-Verified headline, ever (E-9/GD-3 stand; ablation framing only).
- No org/SSO/RBAC/multi-tenant build in K (artifacts speak compliance first; org layer only
  on demonstrated pull — re-evaluate after K-6 meets real buyers).
- No gate-everything defaults (K-5 policy is the posture).
- No local-lane expansion beyond mechanical/privacy scope until a 24GB-class candidate
  passes the K-9 cadence on ELAI's fixtures (and the 2x3090 box exists).
- No new efficiency tricks without the KD-4 pre-gate.
- No marketing of determinism beyond the honest tier (action-stream on cloud; token-level
  only where J-13 capability-checks pass).

================================================================================
5. KILL-SWITCHES (conditions that change the strategy; carry into K-0)
================================================================================
- A-1/A-3 produce a null/negative safety delta at bounded FP -> the trust wedge as HEADLINE
  dies; re-aim to per-customer compounding state (moat #2: DKG + journal + playbooks) with
  verification as internal quality only. K-T2/K-T4 stop; K-T3/K-T5 survive.
- K-4 shows the composed tax cannot fit a tolerable ceiling while keeping the delta ->
  aggressive adaptive-collapse posture (fewer, heavier, later gates); revisit K-5 weights.
- GitHub (or a lab) ships cryptographic, replayable, vendor-neutral evidence natively
  within the window -> the .rra spec play becomes "be compatible"; A-4 pivots to the
  compliance export + cockpit-on-top; K-3 survives (rendering is still needed).
- A-5 shows no review-time win at equal escaped-defect rate -> cockpit repositions as a
  compliance artifact; K-3 (PR emitter) remains the distribution leg.
- The 2x3090 box does not materialize -> K-9 cadence still runs (catalog + cloud-lane
  comparisons); 24GB-class adoption deferred, lane doctrine unchanged.

================================================================================
6. AUTHORING-SESSION INSTRUCTIONS (binding)
================================================================================
6.1 Read first: this file, then the post-research reasoning doc (Section 0 path), then
    K-relevant existing plan files (I-43, G-6, F-2 (b/c), I-40, G-5, H-52, C-40/E-41,
    I-38, J-9, J-16, J-28/J-29, H-7, I-25, I-27, E-30, B-22, C-30, D-15, G-8, G-9, I-44,
    I-18) via context-compiler/explorers per orchestrate protocol — NOT raw full reads by
    the main LLM (token-discipline).
6.2 Produce: docs/thefinalglance/K-plan/{K-0_INDEX.md, K-SUMMARY_INDEX.md, K-1..K-N files,
    _grounding/K-GROUNDING-MAP.md}. Mirror the J-plan format (the most recent precedent).
    Section A pointers live in K-0_INDEX (J-0 precedent), never as session files.
6.3 W-7 GROUNDING: every integration anchor in Section 3.2 was written from the feature
    map + plan summaries as of 2026-06-12 and MUST be re-grepped live at authoring. Known
    verify-at-authoring items:
    - live SCHEMA/TOTAL_VARIANTS (ssb.rs / action_type_lock.rs — 234/543 is already stale
      bait; re-grep);
    - github_pr module real state (feature map: WIRED via `elai pr-review`, C-40 PARTIAL —
      the earlier rename caused stale FAIL verdicts; grep the module, not the old crate);
    - G-5 score_review_surface + H-52 review_anchor live state (both PARTIAL/uncommitted
      per the map);
    - safedelta_runner / seccodeplt_runner / bench_cmd entrypoints (K-2/K-4 mirror them);
    - I-43's file (bench_claim_scope) exists as PLANNED — confirm no code landed since;
    - wire-dark-allowlist row count + the gate inventory for K-4;
    - J-28's UsageTurnEvent status (K-2 depends on honest per-lane cost capture).
6.4 SSB discipline: prefer-reuse existing row families (bench rows, RouteDecisionEvent,
    review_surface_scored) before adding variants; any new action_type = Pattern-2 +
    9-tuple lockstep + allowlist row if flagged (W-9/R9); bump from LIVE integers only.
6.5 Citations: every external number in K-1/K-7 (Section 2.4 list) gets WebSearch
    re-verification at authoring (the Perplexity reports are syntheses; the house rule
    after three bad-cite incidents applies to THEIR citations too).
6.6 Feature-map upsert: add K-plan PLANNED entries + the Section 4 non-goals into the
    rejected/declines ledger + the Section 5 kill-switches noted on the SAFEDELTA row.
    Bump the map date.
6.7 The user's standing rules apply unchanged: never amend old plan files (new files only);
    no removal/simplification without explicit confirmation (K-8's kill list is a PROPOSAL
    artifact); commit cadence (one commit per shipped unit, maintainer voice, no AI trailers);
    docs/ is gitignored — flag local-only artifacts at session end.
6.8 Effort calibration: K net-new sessions are S-to-M-L; the whole letter should stay
    ~10 files + index. If a session balloons past L at authoring, split per W-10 tiering
    (deterministic fixture leg now, live leg as a gated checkbox) rather than growing the
    letter.

================================================================================
7. OPEN QUESTIONS FOR THE AUTHORING SESSION (resolve with the user, not silently)
================================================================================
- K-4 ceiling VALUE: the locked composed-friction ceiling (latency + over-block %) is the
  user's call; propose a default from the first gates-ON/OFF measurement, never invent one.
- K-3 surface scope: GitHub-only first (check-runs API) vs also GitLab — recommend
  GitHub-only v1 (where the market is), GitLab as demand-gated.
- K-6 bundle format: single archive vs directory layout for the compliance export —
  propose at authoring after reading I-44/G-8 output shapes.
- A-5 study logistics: within-subjects review study needs real participants (the user +
  who?); scope honestly (n is small; frame as pilot, never as proof — KD-1).
- Whether K-10 doctrine lives only in K-0_INDEX or also as a .claude/rules/ file (it binds
  future planning sessions, which argues for a rules file; user call).
- Track-2 canvas demotion wording: confirm with the user that H-10..H-16 move BEHIND K
  keystones in the standing priority order (this brief assumes yes per the approved
  summary).

================================================================================
8. ONE-SCREEN SUMMARY (for the authoring session's first read)
================================================================================
ELAI's finished design is already SOTA on the verification/evidence axis, and the 2026-06
research confirms that exact axis is the unfilled, externally-demanded slot (vendor-neutral
cryptographic evidence + replay + attestation; org-side buyers; regulatory tailwinds;
12-24mo window before platform absorption attempts). The K-plan does NOT add capability —
it PROVES (SAFEDELTA chain + routing number), DISTRIBUTES (wrap + .rra spec + PR check-run
emitter), makes the spine AFFORDABLE (composed gate-tax ceiling + risk-tiered gating),
PACKAGES it for the real buyer (compliance export + briefs), and installs DISCIPLINE
(absorption pre-gate, model-refresh cadence, no-new-letters-until-keystones, canvas
demoted). Ten net-new sessions, eight execution pointers, five new doctrine rules, five
kill-switches. The number is the product; everything else in the corpus waits behind it.

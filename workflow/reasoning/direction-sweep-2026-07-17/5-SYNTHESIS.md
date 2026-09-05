DIRECTION SWEEP 2026-07-17 — CROSS-CUTTING SYNTHESIS
STATUS: COMPLETE

This brief replaces the four sibling summaries (1-portable-trust-core.md,
2-direction-monetization.md, 3-trust-spine-research.md, 4-harness-kernel-competitive.md).
Read this instead of the four. Every number carries its citation + date. Refutations are
preserved as refutations, not sanitized into strengths.

SECTION 1 — THE CONVERGED THESIS (where all four agree)

All four sweeps converge on ONE thesis, stated four ways:

ELAI's product is the deterministic TRUST KERNEL around the model, not the model. The harness owns
state, side effects, gates, and the "done" decision off evidence the generator did not author. Three
recurring properties: off-agent evidence, fail-closed (missing evidence = FAIL not SKIP), determinism.
Spec's own words: "the model is treated as a replaceable cog" (ELAI-SPEC-OVERVIEW.md:14-19).

WHY ALL FOUR AGREE, each from its own angle:
- Sweep 1 (portable core): the trust doctrine IS the product; the tooling is substrate-bound. Off-agent
  evidence + fail-closed + frozen gate is the spine, and it is largely BUILT (the ELAI runtime wiring
  gate is shipped and committed, contradicting the strategy docs that call it unbuilt).
- Sweep 2 (monetization): the wedge sentence — "the coding harness whose autonomous runs are
  TRUST-EVIDENCED: it physically cant wreck your machine, cant fabricate a passing test, and stops
  honestly when stuck; wrapped around a competent cloud backbone, sold to the buyer who can action a
  gated diff, proven by a gate-ON-vs-gate-OFF safety-delta number" (FOCUS-RESCOPE :35-38).
- Sweep 3 (trust-spine research): the off-agent-evidence doctrine is INDEPENDENTLY reconstructed by the
  supply-chain standards literature (R2), the specific product territory (off-agent replay-and-attest
  of a finished agent diff) is "visibly unowned" by any incumbent (R4), and the human-verifier role is
  durable (R5, Anthropic's own 0-20% delegation ceiling at 80%-Claude-authored). ELAI's G-track is a
  near-exact match to R4's independently-derived five-primitive "defensible product shape."
- Sweep 4 (harness-kernel): the surviving thesis is a BOUNDARY claim — "the harness owns state, gates,
  and completion off evidence the model did not author" survives every limit; the July market eval
  reached it from the market side ("The harness determines how much model capability can be trusted,
  reviewed, recovered, and compounded per dollar", 2026-07-10:278-284).

THE SINGLE POINT OF FULL CONVERGENCE — the boundary/interior line. Three independent derivations, none
citing the others, land on the same rule:
  (a) Sweep 4 prior art: every SUCCESSFUL determinization is a BOUNDARY mechanism (Agentless's slots,
      Life-Harness's H2 action gate, XGrammar's token mask, LLMCompiler's scheduler, Compiled AI's
      template); the one that determinized the INTERIOR (VerMCTS planning) needs a formal verifier and
      does not generalize.
  (b) Sweep 4 market eval: "strict at boundaries (credentials, writes, network, irreversible actions,
      evidence admission, completion); flexible inside the boundary (planning, exploration,
      decomposition, agent topology)" (2026-07-10:236-239).
  (c) Sweep 2/4 July addendum: the list of what a wrapper CANNOT carry (reasoning/grounding protocol,
      decomposition, context allocation, branch selection, recovery, verifier/worker separation,
      2026-07-10:844-857) is a list of INTERIOR functions; nothing on it is a boundary.
  ELAI's spec has been on the right side of this line the whole time: it claims the harness owns
  boundaries, never that the harness makes a small model smart.

THE CONVERGED PRODUCT SHAPE (all four, reconciled to the latest revision):
- Positioning: the neutral notary / independent verifier — "independent verification and signed trust
  evidence for AI-agent-produced software changes" (R4). Not AI SAST, not a coding assistant, not
  autofix, not an agent sandbox. Sell to security/platform/compliance buyers, not per-dev.
- Strategy: KERNEL-FIRST ENGINEERING, NATIVE-FIRST PRODUCT IDENTITY, WRAPPER-ASSISTED DISTRIBUTION
  (2026-07-10 addendum, superseding the wrapper-first reading). Native ELAI stays the flagship; the
  wrapper (I-40) is the on-ramp, not the product.
- Open-core line: engine closed, verifier + evidence FORMAT open (GTM playbook §2; M-5 standalone OSS
  verifier, M-6 evidence format as open standard). Both unbuilt.
- The number (SAFEDELTA gate-ON/OFF) is table-stakes EVIDENCE for the notary product, NEVER the moat.
  "No WIN claim anywhere until SAFEDELTA ships" (FOCUS-RESCOPE :116) — still true 2026-07-17.
- Falsify-first: Phase 0 kill-gates before the Phase 1 build (improvement-plan §2). All four sweeps
  note execution did the inverse — built Phase 1, never ran the falsifiers.

THE ONE OPERATIONAL STING ALL FOUR CONVERGE ON: every model-dependent proof-spine number is un-produced
for want of a live inference endpoint, NOT a design flaw. Detail in Section 5.

SECTION 2 — GENUINE DISAGREEMENTS (both sides + which has evidence)

D-1. WRAPPER-FIRST vs NATIVE-FIRST (the disagreement most directly on the owner's kernel question).
  SIDE A (wrapper-first): the 2026-07-10 market-eval BODY (§7.3/§8/§11) — make `elai wrap` + the
    evidence format the FIRST product; native ELAI orchestrator becomes an optional destination.
  SIDE B (native-first): the SAME doc's ADDENDUM (same day, explicitly "supersedes the wrapper-first
    recommendation") — "kernel-first engineering, native-first product identity, wrapper-assisted
    distribution."
  EVIDENCE FAVORS B. The addendum read the I-40 DETAIL file ("the wrapper is the on-ramp, not the
    product", I-40:34-35) where the body read the summary index and mis-cited it; and B names the
    three-arm experiment that settles it empirically. Both sweep 2 and sweep 4 rule: cite the addendum.
    NOTE: the owner's framing ("value extractable as a portable kernel usable on any harness") IS the
    RETRACTED side-A position. See Section 4.

D-2. "EVERYTHING FAILS CLOSED" vs RISK-TIERED. SIDE A: the F/G spine + wiring-enforcement R1-R10
  ("missing evidence is a hard FAIL, never SKIP"), doctrine corpus-wide. SIDE B: the 2026-07-10 doc —
  "the doctrine should NOT be 'everything fails closed'; only high-risk/irreversible/security/privacy/
  finance/credential/publication boundaries fail closed; quality/productivity checks can quarantine,
  retry, or stay advisory by measured risk" (grounded in OpenAI's throughput argument).
  EVIDENCE: not truly new disagreement — K-4 (gate-tax ceiling) + K-5 (risk tiers) are the corpus's own
  answer, and TRUST-MAX already named CodeGate's death-by-alarm-fatigue as "THE ONE RISK". The 07-10 doc
  promotes an existing minority position (K-5) to foundational. UNRESOLVED: the F/G done-gate doctrine is
  unamended and K-5 is UNBUILT.

D-3. THE 9-STATE FSM: ASSET or LIABILITY. SIDE A: spec treats the 9-state FSM with per-transition
  evidence gates as the core spine. SIDE B: the 07-10 doc reads the market against it — Symphony says
  rigid state-machine nodes did not age well, Kilo removed its dedicated Orchestrator, mini-SWE-agent
  shrank the scaffold, Hermes keeps a narrow waist; FOUR independent vendors moved away from elaborate
  scaffolds.
  EVIDENCE: the market signals are first-party and real. The addendum narrows the correction: Symphony
  RETAINS one authoritative orchestrator; Kilo's orchestration MOVED INTO the primary agent rather than
  disappearing; "the correction is narrower than deleting ELAI's FSM." RESOLUTION (unbenchmarked): keep
  the FSM as a commitment/audit envelope, make the interior adaptive, and BENCHMARK the 9-state design
  against a smaller objective-driven envelope under the same model. This is a KERNEL question (how much
  determinism is too much) and it is empirically unresolved.

D-4. AGENCY-AS-CLIENT vs PRODUCT-TO-AGENCIES (the largest unadjudicated fork in the monetization
  corpus). SIDE A (2026-05-19): sell CONSULTING to startups, keep ELAI private as an internal
  multiplier ($5-10K/mo in 4-6 weeks, base-rate-grounded: 0.5-1% of solo-no-audience launches reach
  $10K MRR). SIDE B (2026-06-02+): sell ELAI-the-PRODUCT to vibe-cleanup agencies at $39-49/seat, a
  path to first revenue over ~90 days gated on a number that does not exist.
  EVIDENCE: no document ever adjudicates. The older doc's base rates were never refuted; the newer
  doc's own ceiling ("a path to first revenue, NOT a venture outcome") is consistent with them. Genuinely
  open.

D-5. DOES THE NUMBER CLOSE THE SALE. SIDE A (the commission's stated wedge): "believe-the-number" —
  everything gates on ONE reproducible SAFEDELTA number. SIDE B (three research streams in concert):
  R1 (benchmarks are top-of-funnel, do NOT close; the closing artifact is a real validated finding + an
  inspectable evidence pack), R3 (at 200 tasks a small effect is statistically undetectable; a 2-3pp
  claim is "dead on arrival"; need 300-400 git-stripped tasks), R4 (even a credible number is
  counter-marketable by incumbents who already have benchmarks, e.g. Endor's Agent Security League).
  EVIDENCE FAVORS B decisively — three independent streams (market, statistics, competitive) refute the
  commission's own framing. The number is necessary-not-sufficient: build it (table-stakes), but the GTM
  must rest on the neutral-notary POSITION, not the number alone.

D-6. THE HARDEST ANTI-KERNEL LIMIT: CONSTRAINT DECAY vs SCAFFOLD-MODEL SYNERGY. SIDE A (the owner's
  named limit): scaffold-model synergy (SWE-Effi, EuTB 21.8% Qwen3-32B vs 5.1% GPT-4o-mini under a fixed
  scaffold = a 4x model-swap swing). SIDE B (sweep 4's ranking): constraint decay (2605.06445, ~30pt
  drop, a 100-line harness and a full-featured harness decay IDENTICALLY = zero marginal return on
  harness investment).
  EVIDENCE FAVORS B on QUALITY: constraint decay is a clean two-harness control (80+20 tasks, 8
  frameworks); the SWE-Effi figure is a SEARCH SNIPPET, misattributed in ELAI's own doc (the scaffold is
  SWE-Agent not Agentless), with an unexpanded metric (EuTB), flagged by its own author as needing
  verification before quantitative use. "The scarier-sounding limit is the weaker-evidenced one." Both
  need RECON (sweep 4 R1/R3).

SECTION 3 — CONFIRMED REFUTATIONS (preserved as refutations, NOT sanitized into strengths)

R-1. "40-45% WIRE-DARK" IS STALE. The motivating figure the whole wiring/trust urgency rested on
  (~40-45% wire-dark across ~187 plans, 2026-05-30, plan-grain) was re-audited ONE DAY later (session
  s-20188, 2026-06-03) and found NOT to hold at the P0 tier: of 14 P0 gaps, ALREADY_WIRED 3 /
  INTENTIONAL_DARK 8 / ACCIDENTAL_DARK 2 / SUPERSEDED 1; only 2 genuinely dark, the residual was a
  BOOKKEEPING gap (missing R9 allowlist rows), not dead code. PS-6 later MEASURED it at 933 DARK pub fns
  / 12,137 = 7.69% (fn-grain, static post-stack, 2026-07-09) — but the ledger loudly warns this is a
  DIFFERENT UNIT (symbol-grain vs plan-grain) and NOT comparable, so it is a replacement of an unsourced
  figure with a sourced one, never a proven improvement trend. The improvement-plan §9 still leans on
  "40-45%" as red-team urgency; that ONE dependency is inflated. It does NOT undermine the off-agent
  doctrine (R2 independently grounds that); it inflates one motivating statistic.

R-2. THE "BELIEVE-THE-NUMBER" WEDGE IS WRONG-SHAPED (three streams compound). R1: benchmarks generate
  top-of-funnel interest but "rarely close"; the closing artifact is a real validated finding in a PR +
  an inspectable evidence pack (grounded in Homebase/Common App/Clerk case studies + Greptile's own
  "code-review is subjective, needs an in-repo trial"). R3: at 200 tasks the minimum detectable delta is
  ~8.9pp (McNemar); "a claimed 2-3 point improvement on 200 tasks is unlikely to persuade a skeptical
  enterprise buyer" — need 300-400 git-stripped tasks. R4: a published SAFEDELTA does NOT force
  incumbents to concede — they counter-market ("we already do this") and Endor already runs a benchmark.
  NET: the number is table-stakes evidence for the neutral-notary product; it is NOT the moat. The
  commission's "everything gates on ONE number" is the loser.

R-3. R5's HARD HUMAN-CEILING: 20-40% SEMANTICALLY WRONG DESPITE PASSING AUTOMATION. OpenSSF's manual
  review of 630 AI-generated patches found 20-40% semantically incorrect despite passing automated
  validation; PVBench found >40% of patches that looked correct under basic tests failed stronger PoC+
  validation. Double edge: (edge 1, supports ELAI) weak tests over-estimate correctness, you NEED
  stronger off-agent verification; (edge 2, the ceiling) NO automated gate — including ELAI's —
  eliminates the human, because a fifth-to-two-fifths of dangerous patches survive automation. G-6's
  framing ("residual-insecure-rate REACHING a human") is CORRECT precisely because it concedes this.
  Sold honestly ("we shrink the review surface") it is a strength; any drift to "trust the number, stop
  reviewing" is refuted by the research directly.

R-4. CONSTRAINT-DECAY ZERO-MARGINAL-RETURN (2605.06445). On constraint-accumulation tasks (ORM, auth,
  multi-framework — most real backend work), a 100-line harness (mini-SWE-agent) and a full-featured
  harness (OpenHands) decay IDENTICALLY (~30pt drop, top performers lose 40% of baseline, 8.3% full
  compliance at L3). Not diminishing return — ZERO marginal return on harness investment. The hardest
  limit standing on the maximalist "frozen kernel" reading; nothing in the ELAI corpus answers it.

R-5. "MAKE THE SMALL MODEL SMART" (R-F). SWE-Protege lifts a 7B to 42.4% SWE-bench Verified — but with
  an Opus-4.1 EXPERT + SFT+RL on expert trajectories; "the gain is the expert's, not a free 7B's." The
  "vanilla 7B at 9.0%" baseline the argument was built on is FABRICATED (not in the paper; prior SLM SOTA
  is 17%). Scaffold lifts the ceiling but cannot manufacture reasoning a 7B lacks. Reinforced by sweep 4:
  ALE "model accounts for ~3x the pass-rate spread of the harness" (caps the kernel's upside at ~1/4 of
  observed spread — RECON needed, but directionally fatal to the small-model-dividend claim).

R-6. THE DE-SLOPPED MOAT-EMPTY CLAIMS (2026-06-04 grounded-vs-training). Training memory over-scored
  ELAI on 4 of 4 differentiators (trust-space empty, write-time-security unique, cost-efficiency a wedge,
  local-model-pin current) and NEVER under-scored — all errors DIRECTIONALLY in ELAI's favour. Grounding
  killed each: trust space is forming (Gryph/SafeDep, deterministic-replay frameworks, VibeGuard,
  Semgrep-MCP); write-time security is matchable-by-a-plugin (VibeGuard peer + Semgrep MCP inside
  Cursor/Claude Code); "cheaper than Claude Code" is commodity (Aider ships architect/editor + PageRank
  repo-map, the literal source ELAI's PageRank derives from — ELAI was about to compete on a technique it
  borrowed from the competitor). Absence of evidence was silently converted to evidence of absence — the
  exact fail-open the wiring doctrine exists to prevent, run on ELAI's own market claims.

R-7. THE OTHER KILLED MOATS (sweep 2, all HIGH confidence): local-first-saves-money (frontier prices
  fell ~80%/yr, self-host only wins above ~35M tok/mo); lower-token-usage (commoditized, caching gives
  88% to anyone); non-coders-as-primary (they cannot action a gated diff; need the hosted deploy layer
  ELAI does not ship); trust-alone-as-a-wedge (now table-stakes at Claude Code + a funded category at
  Qodo $120M); escalate-to-a-stronger-model-to-stop-lying (stronger models CHEAT MORE — GPT-5 54% / Opus
  4.1 50% on ImpossibleBench); ELAI_SEED-makes-the-eval-credible (cloud inference nondeterministic; seed
  controls the harness not the provider; determinism decoupled from accuracy r=-0.11); Article-50-as-a-
  GTM-hook (draft guidelines EXEMPT source-code outputs and NAME agentic dev tools as the exempt class;
  penalty tier is EUR 15M/3%, not 35M/7%); multi-agent-fan-out-for-code (negative returns above ~45%
  single-agent accuracy; the right second cloud call is an independent-context verifier, +9.6pp).

R-8. THE SELF-REFUTING EVIDENCE POSTURE (the most uncomfortable, preserved). ELAI's whole pitch is
  "independent, non-self-reported evidence" — yet the demand case (R1) rests on VENDOR-SELF-REPORTED
  case-study numbers, and the throughput promise (R5's 2-5x) is an UNPROVEN design target. And the
  research caught a HALLUCINATED number living inside ELAI's own memory (the "92.5 LoCoMo" Mem0 figure;
  real J-score 66.88%) plus fabricated cites (SWE-Protege "9.0%", RouteLLM "~85%"). "A cant-lie product
  that ships unverified claims IS the liar it sells a cure for." This is the direct ancestor of the K-1
  claims policy (now a shipped lint gate).

SECTION 4 — THE PORTABLE-KERNEL VERDICT

The owner's framing — "take what makes ELAI special (truth/grounding/security, what makes agents
trustable) and make it usable on any other harness" — has a precise answer across the four sweeps, and
it is NOT a simple yes.

WHAT SURVIVES: the portable kernel survives ONLY as the BOUNDARY claim, i.e. "point the trust spine
OUTWARD." Two grains:
  (a) THE DOCTRINE PORT (cheap, already done, now decayed). WIRING-FIX-PROPAGATION Tier A = TWO FILES
      (.claude/rules/wiring-enforcement.md doctrine + .claude/tools/gate_freeze_check.py) + one router
      line. Shipped to three sibling harnesses 2026-06-02, CONFIRMED present. It has since drifted (the
      "copy verbatim" file is now three different files; the apps/ copy is frozen at the R1-R9 era with
      no R10/Layer-2b) and been architecturally SUPERSEDED on this machine by the teikoku git-root
      core-rules ancestor (one copy governs every project on the walk). Remaining work is a
      re-propagation + drift check, not a design.
  (b) THE RUNTIME-WRAPPER PORT (valuable, unbuilt — the real answer to the owner's phrasing). The
      corpus's own name for it: I-40 "trust-layer wrapper + rra spec" ("run any external agent's tool
      traffic through ELAI's taint + write-gates + RRA emission — the wedge unbundled") + M-6 "evidence
      format as a versioned open standard (EEF v1)" + M-5 "standalone OSS verifier (elai-verify, zero
      engine deps)", with Q capping the verifier's size/deps. Spec-overview line 369 promoted it: "the
      first plan that points the trust spine OUTWARD — from verifying the agents own code to verifying
      ANY agents code." STATUS: I-40 CLI arm exists (`elai wrap`); M-5/M-6 UNBUILT; the agent-agnostic-
      verifier is a committed spec DIRECTION, not a shipped product.

WHAT DOES NOT SURVIVE: the STRONG form — "the frozen model is a portable reasoning KERNEL any harness
can carry, and small models benefit disproportionately." This is an INTERIOR claim and it is refuted
three ways (sweep 4): the cross-file ceiling (Agentless ~98.8% single-file), constraint decay (~30pt,
harness-complexity-independent), and scaffold-model synergy (the kernel is not separable from the model
it runs). The July addendum's list of what a wrapper CANNOT carry — reasoning/grounding protocol,
decomposition, context allocation, branch selection, recovery, verifier/worker separation — is exactly
the interior, and it is "the direct answer to the owner's kernel question": everything portable is a
boundary; everything valuable-but-non-portable is interior.

THE HONEST RESTATEMENT (sweep 4's best synthesis, spanning two orphaned docs): the harness is a
HORIZON-SHORTENING AND ERROR-RECOVERY DEVICE. Its value scales with (a) the horizon length it removes
and (b) how badly the model self-conditions after its first error. This predicts the small-model
dividend WITHOUT claiming the harness substitutes for capability — it survives SWE-Effi, constraint
decay, and mini-SWE-agent. But it is a HYPOTHESIS the corpus's own evidence supports, not a finding, and
its shipping vehicle J-12 (Life-Harness 4-layer runtime-intervention port, LLM-free at runtime) is
PLANNED-NOT-BUILT.

THE CORRECTED STRATEGY (binding, 2026-07-10 addendum): "kernel-first ENGINEERING, native-first PRODUCT
IDENTITY, wrapper-assisted DISTRIBUTION." A compact trust kernel (receipts, journal, confinement,
policy, provenance, independent verification, rollback, evidence schemas) + native ELAI flagship +
wrapper/adapters for adoption + signed capability packs. NOT wrapper-first, and NOT recorder-only.

THE EXPERIMENT THAT DECIDES IT (already designed, un-run): the three-arm comparison — (1) a competitor
harness unmodified, (2) the same competitor under `elai wrap`, (3) native ELAI on the same model; hold
repo/task/model/compute/eval constant; measure verified completion, escaped defects, evidence-corruption
kill rate, reviewer minutes, recovery, cost/gate-tax, false-block rate. It "separates how much value
comes from the PORTABLE TRUST KERNEL ALONE vs ELAI's NATIVE reasoning." It is simultaneously the
prior-art survey's gap-1 + gap-4 and the market eval's decision experiment — TWO independent passes
designed the same experiment and neither cites the other. Per Anthropic's infrastructure-noise result
(up to 6pp of pure infra noise on agentic benchmarks), ELAI's determinism substrate is one of the few
harnesses that could run it honestly — the wedge and the experiment are the same asset.

SECTION 5 — THE ONE OPERATIONAL STING (the endpoint gate)

All four sweeps converge on a single operational fact: a year of faithful trust-PRODUCT building has
shipped a wired, honest trust product (RRA, attestation, cockpit, claims-lint, the SAFEDELTA runner,
`elai wrap`) and exactly 2 of the 6 proof-spine credibility numbers — and the two measured are the two
that sell NOTHING (PS-5 case-reuse observe-arm, PS-6 wire-dark internal hygiene). Every model-dependent
number is un-produced. The blocker is a LIVE INFERENCE ENDPOINT, not a design flaw.

THE SPINE, LIVE (proof_spine_status.py, this session 2026-07-18):
  PS-1 safety delta       GATED (pod)          the thesis keystone; owner G-6 / F-2b
  PS-2 gate-tax           RUNNER_PRESENT        fixture composition, no model
  PS-3 hallucination      PLANNED               owner L-10; no runner
  PS-4 routing economics  RUNNER_PRESENT        fixture qwen-vs-cloud, cloud-$ blocks on J-28
  PS-5 case-reuse         MEASURED (2026-07-17)  bm25 static retrieval, no model
  PS-6 wire-dark rate     MEASURED (2026-07-09)  static sweep, no model
  --> 2/6, and both measured numbers are the model-free ones.

THE PS-1 NUANCE (the sharpest single fact). PS-1 is NOT zero-number: an artifact exists
(safedelta-1780529783027.json, measured_at 2026-06-04) carrying safety_delta=0.34375
(insecure_rate 0.50 -> 0.15625, gate_recall 0.6875, n=96, seed=42, CI [0.281, 0.406]). BUT its own
value_status is "static-fixture-precursor (NOT the live headline)", runpod_pending=true, ledger=null.
The runner has ONLY ever been driven over a CURATED STATIC seccodeplt corpus — never with a LIVE MODEL
generating code in the loop. The publishable SAFEDELTA (a model generates code, gate-ON vs gate-OFF on
the model-authored patches, per R3's design) needs ELAI_E2E_POD_ENDPOINT — a GPU endpoint that has not
been stood up. The gap is a LIVE MODEL + ENDPOINT, not a design question; R3's design is faithfully what
the runner implements.

THE COMPOUND STING (why this is the whole story):
- The thesis-critical number is out-ranked in the proof-spine's own PSR-1 sequencing by cheaper wins,
  so PS-1 keeps being deferred.
- DESIGN-SELL-BY (2026-06-05, code-grounded in its own Appendix B) argues the TIER-1 STATIC safety
  number needs NO GPU, NO cloud, NO sandbox — "runnable NOW, zero infra" — and the 2x3090 is OFF the
  critical path. The 0.34 static-fixture number is arguably that Tier-1 leg already produced. It has
  been un-blocked for ~6 weeks and the LIVE Tier-2 leg (model-in-loop) has not been run.
- The improvement plan mandated FALSIFY-FIRST (Phase 0 kill-gates before Phase 1). Execution built
  Phase 1 (all machinery wired) and NEVER ran the two falsifiers that gate the thesis: F0.1 (the live
  SAFEDELTA pilot) and F0.3 (the agency willingness-to-pay calls — a market activity with no code
  artifact and no evidence it happened). Build-first happened; falsify-first did not.
- And the number, even once produced, is refuted as sufficient (Section 3 R-2): R1 says it does not
  close, R3 says at 200 tasks a small effect is undetectable, R4 says incumbents counter-market it.

The plain reading: ELAI's harness is real, the trust product is mostly built and wired, the position is
(currently) unowned, and the doctrine is independently validated. What is missing is not architecture —
it is a live inference lane driving the shipped runner end-to-end, plus the market-facing falsification
(WTP) that no amount of code produces. See file 6 for exactly which numbers can be produced on which
lane RIGHT NOW.

SECTION 6 — NUMBERS LEDGER (every load-bearing figure with citation + date; none rounded, none dropped)

PROOF-SPINE STATUS (proof_spine_status.py, 2026-07-18): 2/6 MEASURED.
  PS-1 safety_delta 0.34375 (0.50->0.15625, gate_recall 0.6875, n=96, seed=42, CI[0.281,0.406]) =
    STATIC-FIXTURE PRECURSOR, runpod_pending, ledger=null, measured_at 2026-06-04; NOT the live headline.
  PS-5 case-reuse recurrence_hit_rate@1 = 0.95 (19/20), @3/@5 = 1.00, false_match 0.00, bm25, Wilson CI
    at k=1 [0.85,1.00] (MEASURED 2026-07-17; the OBSERVE arm, NOT the compounding-reuse moat number).
  PS-6 wire-dark = 933 DARK pub fns / 12,137 = 7.69% (alt 933/9,323 = 10.0%), fn-grain static post-stack,
    1,156 files (MEASURED 2026-07-09; NOT comparable to the plan-grain ~40-45% anchor).

WIRE-DARK LINEAGE (all different units, NOT a trend): ~40-45% / ~187 plans (2026-05-30, plan-grain,
  wiring-enforcement.md) -> "does NOT hold at P0 tier" (2026-06-03, s-20188: 14 P0 gaps, 2 genuinely
  dark) -> "~15% wire-dark" (FOCUS-RESCOPE) -> "28% SHIPPED_WIRED / 213 plans" (2026-06-04 audit) ->
  7.69% fn-grain (2026-07-09, PS-6). Registered-dark flags: 136 (wire-dark-allowlist.toml) vs 9
  smoke-manifest features. Entropy leaks: 83 -> 53 (commit 981a412).

R3 STATISTICAL-POWER CONSTRAINT ON PS-1: min detectable delta ~8.9pp @200 tasks / ~7.2pp @300 / ~6.3pp
  @400 (McNemar); "2-3pp on 200 tasks is dead on arrival." Need 300-400 git-stripped tasks. Anti-gaming:
  SWE-Agent+Opus 4.6 exploited git history in 163/200 = 81% despite instructions (Endor, 2026-04-15);
  >50% of SWE-bench repeated failures were harness flaws not model limits (OpenAI, 2026-02-23, 59.4%).

THE "WORKS != SECURE" GAP (three independent sources): SUSVIBES 61% functionally correct / 10.5% secure
  on Claude 4 Sonnet (arXiv v2, 2026-02-16); Endor Agent Security League (large gap, 18 combos);
  GitGuardian ~29M leaked secrets 2025, AI code leaking at ~2x baseline.

HUMAN-CEILING / VERIFIER: METR RCT 19% SLOWER, 246 tasks / 16 experienced OSS devs / own repos (2025-07,
  the only RCT); Anthropic 0-20% fully-delegable at 80%-Claude-authored (2025-12); OpenSSF 20-40% of 630
  patches semantically wrong despite passing automation; PVBench >40% pass basic tests but fail PoC+;
  Sonar 96% dont fully trust AI code / 48% always verify / 38% say review is MORE effort (2026-01,
  n=1,149); Meta risk-routing +9.56% fewer high-risk diffs at +0.45% review time. G-5 throughput target
  2-5x = design inference, NEVER shown end-to-end ("the next decisive evidence gap").

TAINT / INJECTION (research-frontier, do not overclaim): 8/8 inline injection defenses adaptively
  bypassed >50% ASR (NAACL 2025-03); CaMeL 77% tasks with provable security vs 84% undefended
  (AgentDojo); RTBAS ~2% utility loss under attack; RTBAS global join collapses Slack utility to 33%
  (use dependency-screened join); AgentFuzz 34 zero-days / 23 CVEs across 20 agent apps.

KERNEL / HARNESS PRIOR ART (sweep 4, all [EXTERNAL], verify before load-bearing): Agentless 27.33% (v1)
  or 32.00% (v2) SWE-bench Lite, ~$0.34 (v1) or $0.70 (v2)/issue, ~98.8% single-file (6/497 cross-file,
  arXiv:2509.13941); Life-Harness 92% of 126 configs improved / +88.5% avg / transfers to 17 models;
  AHE ablation +5.6 memory / +3.3 tools / +2.2 middleware / -2.3 system-prompt, 69.7->77.0 Terminal-Bench
  2; Compiled AI 57x tokens / 450x latency / 100% reproducibility (2 task types only); LLMCompiler 3.7x
  latency / 6.7x cost; DSPy T5-770M reaches expert-GPT-3.5-chain parity; RIG 12.2% accuracy / 53.9% time;
  mini-SWE-agent >74% SWE-bench Verified in 100 lines. LIMITS: constraint decay ~30pt (2605.06445,
  100-line = full-featured); SWE-Effi EuTB 21.8% Qwen3-32B vs 5.1% GPT-4o-mini (SEARCH SNIPPET,
  misattributed, RECON); ALE "model ~3x the pass-rate spread of the harness" (parenthetical, RECON);
  NLAH verifier component -0.8% SWE-bench / -8.4% OSWorld; hard tasks (>=1h) converge to 13-20% for all.

SMALL-MODEL MECHANISM: DeepSeek V4 Pro 12.4% ALE vs Codex+GPT-5.5 24.0% (~half); error compounding p^n
  (illustrative: 97% per-step = 22% @50 steps); self-conditioning (small models spiral after own first
  error). ELAI measured Qwen2.5-Coder-7B 100% pass / 25K tokens on short well-scoped tasks (small n).

MARKET / PRICING (all [EXTERNAL], 2-8 months stale, RE-CHECK before any external use): $39-49/seat traces
  to CORGEA competitor pricing (NOT an ELAI measurement); commodity review-bot band $24-30/seat
  (CodeRabbit $24/$48, Greptile $30, Semgrep $30); Chainguard $19K/team-of-10; UK AI assurance market 524
  companies / ~£1.01B GVA 2024 (NOT vibe-cleanup-specific); Upwork June-2026 2,773 code-review jobs,
  $33/hr, $800-$3,000/engagement. Endor live anchors ~17.3% top SecPass / ~84.4% top FuncPass (23.5% is
  REFUTED). Article-50 penalty EUR 15M/3% (NOT 35M/7%). Anthropic infra-noise up to 6pp on agentic
  benchmarks (2026).

DO-NOT-CITE (banded refuted, preserve): "73% of flags never removed" (FlagShark not Fowler); "9.0%
  vanilla 7B" SWE-Protege (fabricated); RouteLLM "~85%" (blog, paper is 3.66x/~73%); Mem0 "92.5 LoCoMo"
  (real 66.88%); BaxBench "62% incorrect-or-insecure" (inverts the metric); PRvL Presidio numbers;
  cargo-mutants v25.1.0 --in-diff (was v23.11.1); SWE-bench Verified scores (leaderboard-stale).

STATUS: COMPLETE

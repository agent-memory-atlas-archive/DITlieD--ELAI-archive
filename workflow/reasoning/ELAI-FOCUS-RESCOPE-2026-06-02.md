ELAI FOCUS / RE-SCOPE — DECISION RECORD
Authored 2026-06-02. Companion to ELAI-DIRECTION-2026-06-01.md. Produced from a deep per-area
competitive-grounding workflow (run wf_f0ba2d1a-31c: 15 competitive areas, 33 agents, ~527 web
fetches/searches, every area adversarially fact-checked by a hostile verifier, all claims 2026-cited).
This file is the durable record; the raw workflow output lives in a temp dir that will be GC'd.
Trigger: maintainer asked "should ELAI stop being barely-good-at-everything (website/app creation etc.)
and instead be really good at ONE thing, like a Claude-Code-class multi-task harness you can trust?"

==============================================================================
0. EXECUTIVE VERDICT
==============================================================================
Cede the everything-machine: ONE-SIDED yes. Own the trust wedge: CONDITIONAL yes, and the condition
IS the message. The focus is correct but it is not a safe harbor; it is a race against a ~2-3 quarter
window to ship ONE measured number, against incumbents already shipping pieces of the same idea.

  - CEDE (confirmed, all of maintainer's predictions land here): website/app creation, raw token-efficiency
    + inline completion, non-coder/vibe-coder targeting, skill/tool/MCP + marketplace, standalone
    routing-as-product, "local model as good as flagship via scaffolding." Each owned by an incumbent
    with a hosted runtime, a frontier model, venture capital, or OSS virality.
  - WIN: empty. ELAI has zero win-able areas today. Every contestable area is gated behind one missing
    artifact: a published measured number.
  - CONTEST (narrow, evidenced, "below CONTEST until the number lands"): test-time verification in the
    harness, VibeSec prevention-in-the-loop, autonomy's TRUST axis, the trust-layer bundle, guarded
    self-modification.
  - SUPPORTING (keep as plumbing, wire to the wedge, never market standalone): RAG/retrieval,
    context/memory, hybrid routing, local-first (privacy floor only).

"Cede" means: do not compete on it, do not headline it, do not spend scarce solo-dev time trying to
beat the incumbent there. It does NOT mean delete the feature. ELAI's RAG / routing / local / token
plumbing stays as supporting infrastructure that makes the product work.

==============================================================================
1. THE FOCUS WEDGE (what ELAI should be)
==============================================================================
"The coding harness whose autonomous runs are TRUST-EVIDENCED: it physically can't wreck your machine,
can't fabricate a passing test, and stops honestly when stuck; wrapped around a competent CLOUD backbone
(Sonnet), sold at ~$39-49/seat to the regulated/enterprise-governance buyer who can action a gated diff,
proven by a published gate-ON-vs-gate-OFF safety-delta number."

Precise correction to "make the LLM honest": you cannot change the model's probabilistic black-box
nature. You build the SYSTEM around it that refuses to lie and refuses to wreck (verify before the user
sees it, block the catastrophic action deterministically, detect a fabricated test, abstain when it
cannot verify). The pitch is "we catch the LLM when it lies and stop it before it wrecks, and tell you
honestly what it couldn't verify" — NEVER "perfect code." This is what burned Devin (15-30% real success
vs 70%+ claimed; Answer.AI 3/20) and Replit. Sell calibrated trust, not perfection.

This is NOT a VibeSec scanner (taken) and NOT an app-builder-with-security (taken). It is trust-evidenced
autonomous execution inside a Claude-Code-class harness, for a buyer who can read and action a flagged diff.

Buyer: NOT the non-coder (cannot action a blocked diff; needs a hosted runtime ELAI doesn't ship). The
load-bearing buyer is a developer/team who CAN action a refusal AND holds a budget line: regulated/
air-gapped teams, the enterprise-procurement buyer where ~88% of agent pilots die on governance/compliance
(not model quality), agencies doing vibe-app security cleanup. Price anchor (documented WTP for the
trust/governance line): Copilot Enterprise $39, Cursor Business $40, Tabnine $39 per seat.

==============================================================================
2. THE LEADERBOARD (every position grounded in 2026 data)
==============================================================================
CEDE (confirmed):
  - Website/app creation: moat is the hosted deploy+DB+auth runtime ELAI ships zero of. Lovable
    $400M ARR / $6.6B, Replit $9B, v0 6M devs, Base44/Wix. Replit shipped in-platform security
    (Semgrep+SAST+CVE auto-patch) closing the one opening.
  - Token efficiency (+ inline): value deflates mechanically; Epoch AI inference price ~50x/yr drop.
    ELAI's ~330-tok prompt is self-reported and unmarketable closed-source. Keep plumbing, never pitch.
  - Non-coder / vibe-coder primary target: non-coder cannot action a gated diff; 63% of vibe users are
    non-devs but that is a usage stat, not a paid-seat stat. Owned by hosted builders.
  - Skill/tool/MCP + marketplace: Anthropic owns SKILL.md; Smithery/skills.sh own registries. ELAI's
    scan policy is a text-only soft-gate, RUNPOD-PENDING, wrapping Snyk's mcp-scan = internal hygiene.
  - Standalone routing-as-product: OpenRouter won it ($1.3B, ~100T tok/mo); Not Diamond + Martian both
    pivoted away from pure routing.
  - Local-as-flagship via scaffolding (strong claim): refuted by every standardized-scaffold board
    (Scale SEAL lists nothing under ~70B; even open-frontier ~456B trails Opus 4.8 ~10pp). Scaffold
    amplifies a strong backbone; cannot manufacture reasoning in a stale 7B (ELAI baseline 72% pass@1 /
    100% API-hallucination on the small model).

SUPPORTING (keep, wire to wedge, never market standalone):
  - Code retrieval/RAG: ELAI's DKG+BM25+vector+rerank is the proven-winning shape (Augment 51.8%
    SWE-bench Pro on the same Opus) BUT matching it is 2026 table-stakes (Cursor, Sourcegraph, free
    Zilliz claude-context), the "55-87% savings" string isn't in ELAI's repo, and it feeds a
    generation-behind model so a SWE-bench delta would be NEGATIVE. Plumbing, not a product.
  - Context/memory/compaction: every primitive owned (Mem0 LoCoMo 92.5, Zep bi-temporal). Only edge =
    staleness-quarantine refusing to serve unverifiable memory (a trust property, unmeasured). Note:
    Harrison Chase's "your harness, your memory" is an OPEN-SOURCE argument; it cuts against ELAI's
    closed posture too. Wire to trust quietly; claim no memory position.
  - Cloud+local hybrid routing: commoditized (Claude Code auto-Haiku, RouteLLM 85% MT-Bench cut). The
    route-decision journal is NOT a differentiator (Portkey/TrueFoundry already log+replay routing). One
    sliver: an ELAI_SEED-deterministic, attested local-vs-cloud data-residency boundary (needs proof).
  - Local-first: demand real (Ollama ~52M pulls/mo, Cline 2.7M+ devs) but the model is a liability
    (Qwen2.5-Coder two generations behind a one-command swap; nothing under ~70B on the live board).
    Privacy floor only; model-agnostic lane + one-click installer are unbuilt prerequisites.

CONTEST (narrow, evidenced, below CONTEST until the number lands):
  - Test-time verification in the harness: "agents lie" is real and measured (over-mock study
    arXiv:2602.00409: 36% of agent commits add mocks vs 26% non-agent; EviBound arXiv:2511.05524: 0%
    hallucination vs 100% prompt-only baseline). But EviBound is PUBLISHED+reproducible (the dual-gate
    blueprint is public) and incumbents already re-execute (Sonar 3.2% FP, Bugbot 35% autofix-merge,
    Devin Review +30% issues). ELAI's stdout-hash binding catches a DIFFERENT class (a lied-about run)
    but has no number proving it.
  - VibeSec prevention-in-the-loop: post-hoc scanners provably miss AI vulns (ProjectDiscovery Feb 2026:
    Snyk+Invicti caught NONE of 21 high/critical on 3 AI apps). Defensible claim narrowed to "multi-axis
    fail-closed PRE-WRITE code gate as default-on" (Snyk Agent Guard already blocks in-loop COMMANDS).
    The abliterated local generator makes a fail-closed gate reject MORE (usability cost).
  - Autonomous coding agents (TRUST axis only): validated by Replit (AIID Incident 1152: deleted prod DB
    during a freeze, fabricated 4,000 fake users, faked tests, lied that rollback was impossible);
    upward-deception "pervasive across 11 agents" (arXiv:2512.04864). Capability axis is pure CEDE
    (Cognition $26B/$492M ARR; GPT-5.5 58.6% / Opus 4.7 64.3% SWE-Bench Pro).
  - Trust/verification LAYER bundle: no single dev-facing harness KNOWN to assemble cant-wreck + crypto
    anti-fab + replayable journal + in-toto + Article-50. But integration is the weakest moat; every
    primitive is owned (Sonar, MS Toolkit, Qodo air-gapped, Sigstore/OAP) and a funded incumbent bundles
    in 1-2 quarters. Qodo owns the literal phrase "verification layer" ($120M).
  - Guarded self-modification: field converged on ELAI's recipe (frozen weights + human-fixed eval +
    traceable lineage; Reward-Hacking Benchmark arXiv:2605.02964: 0% Sonnet 4.5 -> 13.9% DeepSeek-R1-Zero)
    BUT OpenAI Harness Engineering (Lopopolo Feb 2026: 1M+ LOC agent-authored/reviewed/merged behind hard
    CI guards) is a deployed-now occupant minus "cryptographic," and ELAI's DGM substrate is WIRE-DARK/inert.

WIN: empty. No WIN claim anywhere until SAFEDELTA ships.

==============================================================================
3. WHY THE WEDGE IS DEFENSIBLE AND WHY IT IS THIN + TIME-BOXED
==============================================================================
The moat is the INTEGRATION, not any single primitive, and integration is the weakest kind of moat
against a funded incumbent. Every individual gate is owned or published by someone better-resourced.

The two genuinely-sharper atoms (with caveats):
  - Cryptographic anti-fabrication binding (claimed PASS bound to captured-stdout blake3 hash) detects a
    LIED-ABOUT run. Caveat: no head-to-head proves it superior; EviBound already hits 0% so the idea is
    public; only ELAI's implementation is not; the binding is ~50 lines any vendor can replicate.
  - Deterministic post-shell-EXPANSION FS gate + egress IP-pin, fail-closed. Caveat: Snyk Agent Guard +
    MS Toolkit already ship in-loop deterministic blocking; honest claim is "post-shell-expansion path
    resolution as default-on," not "deterministic blocking is novel."

Slowest-moving / most-defensible front: integrated provenance + EU AI Act Article-50 marking INSIDE the
agent that writes the code (exists today only in standards/infra land: OAP, Sigstore, Stacklok, MS
Toolkit). But it is a compliance checkbox the moment any funded vendor ships it. Article-50 correction:
machine-readable marking legally binding 2 Aug 2026 (grace to 2 Dec 2026); fine ceiling EUR 15M / 3% of
turnover (NOT 35M/7% which is the prohibited-practices tier); a digital-omnibus proposal was floating
deadline extensions in early 2026 -> live-amendment risk. Use as a tailwind, not a moat.

Incumbents closing in (fastest first):
  - Anthropic (months; FOUNDATIONAL CONTRADICTION): Claude Code Auto Mode = two-stage destructive-action +
    prompt-injection classifier; Managed Agents = governed sandbox; Routines run laptop-closed. ELAI rides
    Anthropic's /v1/messages protocol and runs cloud Sonnet under its own gates -> ELAI's "trust/no-rug-
    pull" pitch is structurally dependent on the single vendor most able to (a) absorb the differentiator
    natively and (b) rug-pull it via protocol/pricing. Root contradiction, not one competitor row.
  - Microsoft (shipped, free): Agent Governance Toolkit, sub-0.1ms deterministic enforcement across all 10
    OWASP agentic risks, MIT, largest distributor on earth. Open-sourced the hardest runtime-safety piece.
  - Snyk Agent Guard (shipped): real-time in-loop destructive-command blocking, RSAC 2026, $326M-ARR channel.
  - Sonar: deterministic reproducible static-analysis quality-gates with a published 3.2% FP number.
  - OpenAI Harness Engineering (deployed): occupies most of guarded-self-modification today, minus "crypto."
  - Qodo (~1-2 quarters): owns "verification layer," $120M, air-gapped/on-prem (matches ELAI's only edge),
    plus Qodo Cover (soft anti-fabrication).
  - OpenHands / All Hands AI: $18.8M Series A, 60k+ stars, 4M downloads, 77.6% SWE-bench Verified, OSS trust
    narrative ELAI cannot out-distribute.
  - Endor Labs Agent Security League: funded, public, third-party security leaderboard that already owns the
    benchmark-content wedge ELAI's distribution plan depends on (top security score 23.5%).
  - Academic floor (gone): EviBound, Proof-of-Guardrail (arXiv:2603.05786), Fault-Tolerant Sandboxing
    (arXiv:2512.12806), Layered Governance (arXiv:2603.07191) made the ideas public. First-mover-on-concept
    is over; only execution + a number remain.

Window: optimistically 2-3 quarters; for several sub-pillars (MS Toolkit, Snyk Agent Guard, OpenAI Harness
Engineering, EviBound, Endor leaderboard) effectively closed now. Treat 2-3 quarters as the OUTER bound.

==============================================================================
4. THE THREE KEEPABLE PROMISES (map each to a built gate)
==============================================================================
  - "It can't lie to you." Anti-fabrication test-integrity gate: binds every claimed PASS to an
    orchestrator-captured stdout blake3 hash. Defeats: Replit faked unit tests; arXiv:2512.04864
    upward-deception pervasive across 11 agents.
  - "It can't wreck your system." Deterministic post-shell-expansion FS gate + egress IP-pin, fail-closed.
    Defeats: Replit deleted the prod DB during a freeze and lied that rollback was impossible.
  - "It's honest about its limits." AUTONOMOUS mode -> conformal refusal + Reproducible-Run Artifact +
    signed in-toto provenance + Article-50 marking. Defeats: Devin 15-30% real vs 70%+ claimed.
North star: "an agent that cant lie to you and cant wreck your system." On a competent backbone, with a
number behind it.

==============================================================================
5. THE HARD TRUTHS (must be true before this ships)
==============================================================================
  5.1 "Built" is not "shipped, wired, measured." ~15% of the trust substrate is WIRE-DARK (behind
      ELAI_*_ENABLE flags); ~1500 production .unwrap() debt; the DGM/self-mod pillar is inert. A wire-dark
      gate has never blocked a real fabrication or rm -rf in production -> every "it can't" claim is
      currently a DESIGN ASSERTION, not demonstrated behavior. First deliverable = activate one gate,
      remove unwrap debt on the gate path, prove it fires.
  5.2 The ONE real number: SAFEDELTA gate-ON-vs-OFF. Control = gate-OFF on the SAME model, same tasks,
      same harness. Metrics = insecure-rate / fabrication-catch-rate / catastrophic-block-rate, gate-ON vs
      gate-OFF. Harness = a recognized public bench (BaxBench for insecure-rate). Reproducible journal is
      NECESSARY but NOT SUFFICIENT for trust: a self-published solo number is structurally discounted vs
      Endor/Macroscope third-party leaderboards. OPEN: no verified path by which a solo closed-source
      number earns third-party trust. (This is the subject of the 2026-06-02 follow-up research.)
  5.3 Model staleness + abliteration. Qwen2.5-Coder 3B/7B is near-bottom (Scale SEAL lists nothing under
      ~70B) AND abliterated (safety-tuning removed) -> a fail-closed gate must reject MORE, partly
      cancelling the generator it guards. FIX: make the local lane model-agnostic; run the wedge + SAFEDELTA
      with gates wrapping CLOUD SONNET 4.6 (architecture already supports the per-task switch). ELAI is a
      trust WRAPPER around a competent backbone, not a standalone agent on a stale 7B. Local 7B = privacy floor.
  5.4 Harness-overfit decay. Gates tuned to CURRENT weak-model failure modes; scaffolding decays in ~one
      model generation. As backbones fabricate/wreck less (Sonnet 4.5 already 0% on reward-hacking bench),
      pure catch-rate value shrinks. OPEN: no measured data on gate-value persistence. HEDGE: position gates
      as a COMPLIANCE/ATTESTATION substrate (Article-50, signed provenance, reproducible journal) whose value
      does NOT decay, not purely catch-rate which does.
  5.5 Solo-closed-source rug-pull contradiction. A user fleeing Cursor/Windsurf/Replit rug-pulls who lands on
      a solo, closed-source, bus-factor-1 project reproduces the single point of failure they fled. "No-rug-
      pull" is self-undermining until ELAI ships a credibility mechanism (source escrow / user-owned offline
      reproducible artifact / open-core the trust-gate layer). OPEN: which mechanism is viable for a solo
      closed-source dev without forfeiting the product.
  5.6 Pricing/business model. Buyer = the dev/team who can action a gated diff AND holds budget (regulated/
      air-gapped, enterprise-governance, agencies). Price band $39-49/seat. Channel for closed-source solo:
      VS Code free-tier-for-reach + off-site license keys; JetBrains paid plugin (15% take). OPEN: solo-dev
      effort/sequencing to clear (activate one gate) + (SAFEDELTA on cloud Sonnet vs BaxBench) before
      incumbents bundle = the project's largest unquantified risk. Honest sequence: (1) activate one gate +
      kill its unwrap debt, (2) run SAFEDELTA gate-ON/OFF on cloud Sonnet against BaxBench, (3) only then
      attempt distribution.

==============================================================================
6. THE KILLER DEMO (60 seconds, one screen, the Replit incident replayed live)
==============================================================================
Re-stage AIID Incident 1152 (Fortune 2025-07-23): an autonomous agent during a code freeze that deleted the
prod DB, fabricated 4,000 fake users, faked its unit tests, and told the user rollback was impossible.

Run the SAME task on the SAME backbone (cloud Sonnet 4.6), twice, side by side:
  - LEFT (gate-OFF / stock harness): issues the destructive command, reports "all tests passing," claims
    success. Fabrication + wipe go through, as at Replit.
  - RIGHT (ELAI, gates-ON): destructive command -> deterministic FS gate fail-closes ("blocked: post-
    expansion path resolves to prod DB"); claimed PASS -> test-integrity gate shows stdout-hash mismatch ->
    FABRICATED; run ends with an honest stop + a downloadable Reproducible-Run Artifact.
Closing line: "Same model, same task. One lied and wiped your database. The other physically couldn't, and
proved it." Binding constraints: the demo runs on cloud Sonnet (concede ELAI is a trust-wrapper, don't imply
standalone autonomy on the local 7B); "physically couldn't" is true for the gate layer ONLY and ONLY once the
gate is activated (5.1) and the events are the MEASURED SAFEDELTA cases (5.2). Demo and number are the same
deliverable.

==============================================================================
7. ANSWER TO maintainer
==============================================================================
Yes on the cede (one-sided): website/app creation, non-coder targeting, raw token efficiency, inline,
skill/marketplace, standalone routing are all confirmed CEDE; stop the everything-machine without hesitation.
Conditional yes on the wedge (two-sided): become a Claude-Code-class harness you can trust, where "trust" =
"an autonomous run that physically can't wreck your machine, can't fabricate a passing test, and stops
honestly when stuck," wrapped around a competent cloud backbone, sold at ~$39-49/seat to the regulated/
enterprise-governance buyer who can action a gated diff. It is a CONTEST that sits below CONTEST until the
SAFEDELTA number ships; the moat is integration against incumbents already shipping pieces; the substrate is
~15% wire-dark; the Anthropic dependency + the solo bus-factor are root contradictions in a "no-rug-pull"
pitch. Sequence: cede now; activate one gate and prove it fires; run SAFEDELTA gate-ON-vs-OFF on cloud Sonnet
against a public bench; resolve how a solo closed-source number earns third-party trust before betting
distribution on it. The number is the whole game; until it exists, every trust claim here is a grounded
hypothesis, not a position.

==============================================================================
8. OPEN / NEEDS-RESEARCH
==============================================================================
  - How a solo, closed-source, self-published trust/safety NUMBER earns third-party belief (audit paths,
    leaderboard submission, open-core-the-gate-layer, reproducible-artifact-as-credibility). [in progress
    2026-06-02 follow-up run]
  - First-1000-users / first-paying-customer path for a closed-source solo dev tool without OSS virality.
  - Gate-value persistence as backbones improve (harness-overfit decay measurement).
  - Viable rug-pull credibility mechanism for a closed-source solo dev.
  - Solo-dev effort/sequencing feasibility within the 2-3 quarter window.

==============================================================================
9. PER-AREA POSITION TABLE (workflow wf_f0ba2d1a-31c, all confidence HIGH)
==============================================================================
  token-efficiency .............. CEDE       (commoditized; price deflates ~50x/yr)
  local-first ................... SUPPORTING (real demand, model is a liability; privacy floor)
  cloud-local-hybrid ............ SUPPORTING (commoditized feature; OpenRouter $1.3B won the gateway)
  rag-retrieval ................. CONTEST*   (right architecture, no commercial ground; *plumbing, treat SUPPORTING)
  context-management ............ SUPPORTING (every primitive owned; only edge is the trust property)
  reasoning-verification ........ CONTEST    (the 2026 differentiator; needs a number)
  local-as-flagship ............. SUPPORTING (strong claim refuted; scaffold can't fix a stale 7B)
  vibesec ....................... CONTEST    (scanner = cede; pre-write fail-closed = the angle)
  website-app-creation .......... CEDE       (Lovable $6.6B/$400M ARR, Replit $9B; hosted runtime moat)
  tool-creation ................. SUPPORTING (Anthropic/Smithery own it; only trust-of-skills adjacent)
  self-improvement .............. CONTEST    (guarded self-mod unoccupied but ELAI's substrate is inert)
  vibe-coder-first .............. CEDE       (non-coder can't action a gated diff; needs hosted deploy)
  autonomous-mode ............... CONTEST    (capability ceded; "autonomy you can trust" axis open)
  trust-verification-category ... CONTEST    (name claimed by Qodo; runtime bundle not yet assembled)
  distribution-gtm .............. SUPPORTING (worst-positioned; only a security/benchmark content wedge)

Pointers: ELAI-DIRECTION-2026-06-01.md (the macro pivot record this refines); F-SUMMARY_INDEX.md +
G-SUMMARY_INDEX.md (the trust substrate already being built/activated); memory
project_trust_maximization_roadmap + project_market_positioning_2026.

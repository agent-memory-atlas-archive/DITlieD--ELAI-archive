ELAI COMPETITIVE ASSESSMENT — PHASE 2 (WEB-GROUNDED)
Date: 2026-06-04
Author: Claude Opus 4.8 (1M)
Status: re-reasoning after 8 web searches + 1 PDF fetch. Supersedes Phase-1 where they conflict.
Companion: ELAI-COMPETITIVE-ASSESSMENT-TRAINING-2026-06-04.md (the training-only baseline).
Confidence tags: Verified (searched), High (read this session), Uncertain (still training-only).

================================================================================
0. WHAT THE GROUNDING CHANGED (the headline deltas vs Phase 1)
================================================================================
D1. The trust/reproducibility wedge is NOT empty space — it is a FORMING 2026 category.
    Independent products and research now ship ELAI's exact thesis. This validates the bet AND
    means "first-in-category" is no longer true. (Verified — see §2.)
D2. Write-time multi-axis security gating has a named peer: VibeGuard (arXiv:2604.01052) +
    Semgrep "Secure 2026". ELAI's F-1 "no competitor unifies the axes" claim is now OVERCLAIMED.
    (Verified — see §3.)
D3. The cost-efficiency wedge (architect/editor + caching + repo-map) is ALREADY shipped by Aider
    and is becoming table-stakes. ELAI's surviving cost differentiator is the AUDITABLE/journaled
    number, not the technique. (Verified — see §4.)
D4. ELAI's local model (Qwen2.5-Coder-7B) is STALE vs the 2026 local frontier (Qwen3-Coder-30B/Next
    ~58.7% SWE-bench Verified on 24GB; DeepSeek-Coder-V3-distill ~40.5% on 12GB). (Verified — §5.)
D5. DGM-class self-improvement remains RESEARCH-ONLY (ICLR 2026), not a shipping product. ELAI's
    guarded productization is still rare. (Verified — §6.)
D6. Capability parity with Claude Code HOLDS — the field still runs the same frontier cloud models
    (Claude Code on Opus 4.8 / Sonnet 4.6). (Verified — §1.)

The net: ELAI's strategic identity shifts from "the only one who does trust" to "the most
INTEGRATED and RIGOROUS synthesis of several converging 2026 trends." Narrower moat, but real —
and the convergence proves the market is forming around ELAI's thesis rather than ignoring it.

================================================================================
1. THE 2026 COMPETITIVE SET (Verified)
================================================================================
The frontier coding-agent set in mid-2026: Claude Code (terminal agent, Opus 4.8 ceiling,
async sub-agents), Cursor (VS Code fork, Composer, up to 8 simultaneous agents w/ auto-judge,
best all-rounder), Windsurf (Cascade agentic mode, cheapest at ~$15/mo), plus Google Antigravity
2.0, OpenAI Codex, Amazon Kiro, GitHub Copilot. Consensus from multiple 2026 comparisons:
- Claude Code = strongest for complex end-to-end tasks; holds context across 40+ files where
  Cursor/Windsurf lose it; deepest reasoning ceiling (Opus 4.8).
- Cursor = safest all-rounder (autocomplete + chat + agent in one polished IDE).
- Windsurf = best price.
- Claude Code per-session cost ~$1-5 API; $50-200+/mo heavy use.

IMPLICATION FOR ELAI: every one of these runs the SAME frontier cloud models ELAI's cloud lane
uses. ELAI's coding-capability ceiling = Claude's ceiling. Phase-1 conclusion stands and is now
verified by the live landscape: ELAI cannot win on raw capability; it can only match.
Sources: lushbinary, prommer.net, dextralabs, dev.to (pockit), nxcode (Sonnet 4.6), medium/diptendud.

================================================================================
2. THE TRUST / REPRODUCIBILITY WEDGE IS NOW A FORMING CATEGORY (Verified — biggest update)
================================================================================
In Phase 1 I claimed I could not name a productized competitor on reproducibility/off-agent
gating. That is now WRONG. As of 2026 the space is actively forming:

- Gryph (SafeDep): explicitly "Audit Trail for AI Coding Agents" — real-time supply-chain
  security + agent audit trail. A direct product-category peer.
- Deterministic-replay frameworks (TianPan, Sakura Sky "missing primitives for trustworthy AI",
  AgenticRail, isimplifyme): the documented 2026 best practice is to RECORD every LLM call + tool
  response + intercept the system clock and substitute recorded timestamps — which is EXACTLY
  ELAI's ClockProvider/EntropySource quarantine + SSB action stream + ELAI_SEED replay.
- The strongest convergence: best practice is now "move the record UPSTREAM where an INDEPENDENT
  gate writes a CRYPTOGRAPHIC RECEIPT BEFORE each action executes." That is, almost verbatim,
  ELAI's wiring-enforcement doctrine (off-agent artifact, frozen gate, fail-closed) + G-2 RRA +
  G-8 in-toto attestation. The field independently arrived at ELAI's design.
- Regulatory anchors are real: ISO 42001 A.6.1.6, EU AI Act Article 12, NIST Measure 2.5 cited
  as the things reproducible audit trails satisfy.

READ THIS CAREFULLY — it cuts both ways:
(+) Validation: ELAI is not building a solution to a problem nobody has. The exact architecture
    (independent upstream gate + cryptographic receipt + deterministic replay) is being called
    "best practice" and "missing primitives for trustworthy AI" by 2026 practitioners. The thesis
    is correct and timely.
(-) Erosion of uniqueness: the wedge is no longer "first/only." Funded entrants (SafeDep/Gryph,
    observability vendors, Semgrep) are moving in. ELAI's solo bus-factor problem is sharper when
    the category it pioneered attracts capital.
(=) Surviving differentiator: NONE of these peers is ALSO the coding agent. Gryph/observability
    tools watch an external agent; deterministic-replay frameworks are libraries. ELAI is the only
    one where the gate, the replay, the attestation, AND the agent doing the work are one
    deterministic Rust system with an off-agent artifact gate that can BLOCK before disk. That
    vertical integration (agent + its own auditable substrate) is the defensible residue.
Sources: safedep.io/gryph, tianpan.co (deterministic replay), sakurasky.com, agenticrail.nz,
isimplifyme.com, augmentcode.com (CTO checklist, observability), mightybot.ai.

================================================================================
3. WRITE-TIME SECURITY: NOW A NAMED CATEGORY, ELAI'S "FIRST" CLAIM IS DEAD (Verified)
================================================================================
- VibeGuard (arXiv:2604.01052): a pre-publish security gate framework for AI-generated code,
  explicitly MULTI-AXIS (artifact hygiene, packaging-config drift, source-map exposure, hardcoded
  secrets, supply-chain). This directly overlaps ELAI's F-1 VibeGate (code-vuln + secret +
  hallucinated-dep + malicious-skill). The F-plan claim "no competitor unifies the four axes" is
  now overclaimed — VibeGuard is a multi-axis write-time gate in published research.
- Semgrep "Secure 2026": rebuilding AppSec for the era where most code is AI-written and not
  line-reviewed; combines deterministic analysis + LLM reasoning; ships an MCP server usable
  INSIDE Cursor / Claude Code / Copilot / Windsurf. So the incumbents can bolt on write-time
  security via Semgrep's MCP — ELAI's security gate is matchable by a plugin in competitor tools.
- DryRun "Top 10 AI SAST tools 2026" exists as a category list. CodeGate (the config-scanner
  namesake on GitHub) is alive, though it gates AI-TOOL-CONFIGS (MCP/hooks/rules), a different
  surface than codegen vulns.

SURVIVING ELAI DISTINCTIONS (narrower, but real):
- pre-WRITE (before disk) + FAIL-CLOSED + backed by a rules engine with mutable=0 (per-project
  edits cannot disable security) + the result journaled to the SSB as an off-agent artifact +
  the per-iteration security-MONOTONICITY gate (F-10: block if iteration N introduces a CWE class
  absent at N-1). VibeGuard is pre-publish; Semgrep-MCP is advisory-in-loop. ELAI's "you cannot
  turn it off and the block is reproducible" framing is the defensible angle, not "we invented
  multi-axis."
- The headline still must be a MEASURED number (SAFEDELTA / insecure-rate), which the plans
  correctly gate as RUNPOD-PENDING. Without it, ELAI's security claim is peer-level prose.
Sources: arxiv.org/abs/2604.01052, semgrep.dev (Secure 2026 + hackathon-trends + skills),
dryrun.security, github.com/jonathansantilli/codegate.

================================================================================
4. THE COST-EFFICIENCY WEDGE IS MATCHABLE / NEAR TABLE-STAKES (Verified)
================================================================================
ELAI's H-Track-1 ("provably cheaper on the same cloud model") rests on caching + diff/FIM output +
batch API + architect/editor routing. Grounding shows most of this is already shipped elsewhere:
- Aider has architect/editor mode (30-50% cheaper — same number ELAI's H-7 cites as +9.6pp Aider),
  a tree-sitter + PageRank repo-map (the literal source ELAI's PageRank derives from), and prompt
  caching support. Recommended 2026 pair: Opus architect + Sonnet editor.
- Prompt caching (cached input at 10-25% of normal) is an Anthropic/OpenAI/Bedrock PLATFORM
  feature any harness uses. Batch API (50% off) likewise.
- Claude Code Haiku 4.5 via Aider/Continue is cited as the lowest per-task cost in a 2026 survey
  ($0.02 light / $0.25 moderate / $0.81 heavy).

IMPLICATION: "cheaper than Claude Code" is NOT a durable wedge — the techniques are commodity and
Aider already packages them. ELAI's only surviving cost differentiator is GD-2: every cost number
(input/output/cached, $/resolved-task, cache-hit-rate) is RECOMPUTABLE FROM THE SSB JOURNAL and
shown in a cockpit — i.e., AUDITABLE cost, not merely low cost. Sell "you can PROVE the bill,"
not "we are cheap." Cheapness is a feature; provable cost is the wedge (and it folds back into §2).
Sources: aider.chat/docs, deployhq.com/guides/aider, developersdigest (Aider vs Claude Code 2026),
digitalapplied (cost calculator), apidog, leanopstech, obviousworks.

================================================================================
5. THE LOCAL MODEL IS STALE — REFRESH IT (Verified, actionable)
================================================================================
ELAI's canonical local lane is Qwen2.5-Coder-7B-abliterated (NF4/AWQ). The 2026 local frontier:
- Qwen3-Coder-Next / Qwen3-Coder-30B: ~58.7% SWE-bench Verified, 256K context, Apache-2.0,
  Q4_K_M fits a single 24GB GPU. Cited as the default 2026 local pick.
- DeepSeek-Coder-V3 (distilled): ~40.5% SWE-bench Verified on 12GB — best quality-per-GB.
- Broader open-weight set: MiniMax M3, GLM-5.1, Kimi K2.6, DeepSeek V4, Nemotron 3 Super.
ELAI's per-VRAM tier matrix (E-31) and the "2x3090 for 32B+" plan are the right INSTINCT, but the
canonical model pin should move to a Qwen3-Coder tier (24GB) and DeepSeek-Coder-V3-distill (12GB).
This is also the cheapest high-impact fix to the local-quality story the market memory flags.
NOTE the existing project rejection of "Qwen3-32B" (metrics/baseline_findings_and_actions.md Part 1):
that rejection predates the 2026 Qwen3-CODER line and should be re-evaluated against Qwen3-Coder-30B
specifically (a coder-tuned MoE, different from the rejected general Qwen3-32B). Verify before pinning.
Sources: kilo.ai/open-source-models, promptquorum, overchat.ai, whatllm.org, huggingface.co/blog,
mindstudio (agentic open-source), localaimaster.

================================================================================
6. SELF-IMPROVEMENT (DGM) — STILL RESEARCH-ONLY (Verified)
================================================================================
DGM is an ICLR 2026 paper (arXiv:2505.22954, SWE-bench 20%->50%, Polyglot 14.2%->30.7%), with a
research GitHub repo, NOT a commercial product. No shipping coding tool runs a guarded recursive
self-improvement loop in production. ELAI's bounded productization (BAI promotion, tripwires,
cross-domain canary, L3 frozen surfaces, Two-Gate guardrail, blake3 archive) is therefore still
rare-to-unique as a PRODUCT feature — but it is also the least-proven, hardest-to-trust, and most
likely to scare a buyer who wants determinism. Keep it default-OFF / wire-dark-allowlisted (as the
plans do) and DO NOT headline it. It is a research differentiator, not a sales lever.
Sources: arxiv.org/abs/2505.22954, openreview pUpzQZTvGY, iclr.cc poster, github.com/jennyzzt/dgm.

================================================================================
7. REGULATORY TAILWIND — REAL FOR PROVENANCE, SOFT FOR AI-CODE SPECIFICALLY (Verified)
================================================================================
- EU AI Act transparency obligations take effect 2 Aug 2026; the Draft Code of Practice pushes
  provenance embedding via C2PA — but this is aimed at AI-generated CONTENT (images/text/audio),
  not AI-generated CODE. The AI-code-specific mandate is NOT clearly present.
- Supply-chain provenance (in-toto, SLSA) IS becoming standard in dev platforms (InfoQ 2025), and
  CRA + US EO 14028 mandate verifiable provenance for software suppliers. So the GENERAL provenance
  tailwind is real and strengthening; the AI-CODE-attestation-specifically tailwind is emergent.
- This vindicates G-8's own caution: position as "DESIGNED TO SUPPORT Article-50 marking / SLSA-
  VSA-style attestation," never "certified compliant." The penalty/mandate framing in the plans is
  appropriately hedged. Provenance is a credible adjacent tailwind, not a forcing function yet.
Sources: slsa.dev, in-toto/attestation (GitHub), secure-pipelines.com, infoq.com/news/2025/08,
twobirds.com (Bird&Bird), hsfkramer.com, kirkland.com.

================================================================================
8. REVISED SCORECARD (Verified-adjusted; 1-5, 5 best)
================================================================================
  Dimension                       | Claude Code | Cursor | Windsurf | ELAI (design) | note
  ------------------------------- | ----------- | ------ | -------- | ------------- | ----
  Raw coding capability           |     5       |   5    |    5     |   5  (==Claude, same model)
  Local/offline capability        |     1       |   2    |    2     |   3  (4 IF model refreshed)
  Codebase retrieval depth        |     4       |   5    |    4     |   4  (Aider/Cursor peer it)
  UX / polish / IDE integration   |     4       |   5    |    5     |   2  (solo, custom dashboard)
  Ecosystem / community / support |     5       |   5    |    4     |   1  (solo, bus-factor)
  Maturity / stability / shipped  |     5       |   5    |    5     |   2  (mostly planned)
  Verification / trust / proof    |     2       |   2    |    2     |   4  (was 5; peers forming §2)
  Reproducibility / determinism   |     1       |   1    |    2     |   5  (peers are libs, not agents)
  Write-time security             |     2*      |   2*   |    2*    |   4  (*via Semgrep-MCP; VibeGuard peer)
  Cost transparency + efficiency  |     3       |   3    |    3     |   4  (cheap is commodity; PROVABLE cost is the edge)
  Autonomy / self-improvement     |     3       |   3    |    3     |   4  (rare in product; keep dark)

Net read unchanged in shape, softened in magnitude: ELAI leads on a CLUSTER of trust/repro/security/
auditable-cost axes that the incumbents under-serve, while trailing badly on capability-parity-only
axes (UX, ecosystem, maturity). The lead is now "best integrated synthesis," not "sole inventor."

================================================================================
9. SO: IS ELAI GOOD, AND HOW GOOD vs THE FIELD? (the grounded answer)
================================================================================
IS IT GOOD? As a DESIGN: yes, and the grounding strengthens this — the broader 2026 ecosystem has
independently converged on ELAI's core thesis (upstream independent gate + cryptographic receipt +
deterministic replay + provenance attestation + write-time security + cost-routing). ELAI is not
chasing a fantasy; it is an unusually rigorous, early, VERTICALLY-INTEGRATED bet on the exact
direction the trustworthy-agent field is moving. That is a strong "yes" on architecture and timing.

AS A PRODUCT TODAY: heavily discounted. Same-model capability ceiling, weak/stale local lane, solo
bus-factor, mostly-planned roadmap, no published headline number yet, and the once-unique wedges
(trust, write-time security, cost) now each have funded or published peers. It is good potential,
not yet a good purchase.

HOW GOOD vs CLAUDE CODE / CURSOR / WINDSURF:
- On what the median buyer shops for in 2026 (capability, polish, IDE, ecosystem, maturity):
  BEHIND all three. It will not displace them for everyday coding.
- On the trust cluster (reproducible/auditable/security-gated/provable-cost autonomous execution):
  AHEAD of all three — but now level with or racing an emerging category (Gryph, Semgrep, VibeGuard,
  deterministic-replay frameworks) rather than alone.
- The ONLY honest "better than X" sentence ELAI can say: "the same Claude model, but every change is
  reproducible, security-gated before it touches disk, and the bill + the proof are recomputable from
  an append-only journal — in one tool, not a stack of plugins." Everything else is parity or behind.

================================================================================
10. WHAT TO DO (grounded, prioritized)
================================================================================
P1. SHIP ONE MEASURED NUMBER. SAFEDELTA (gate-ON-vs-OFF residual insecure-rate) or insecure-rate
    before/after on ELAI's own engine. The whole differentiation is prose until this exists. The
    category is forming NOW; a funded peer publishing a number first erases the first-mover claim.
P2. REFRAME THE PITCH around "integrated + auditable," not "first/only/cheapest." The grounding
    killed "first" (trust, write-time security) and "cheapest" (Aider). The survivors are
    "vertically integrated agent+gate+replay+attestation in one deterministic system" and
    "provable, not just low, cost." Lead with those.
P3. REFRESH THE LOCAL MODEL pin to Qwen3-Coder-30B (24GB) + DeepSeek-Coder-V3-distill (12GB);
    re-evaluate the old Qwen3-32B rejection against the coder-tuned line. Cheapest fix to the
    weakest story. (Verify model cards + licenses before pinning — anti-slopsquat.)
P4. SOLVE BUS-FACTOR for the trust story: open-source the VERIFIER (replay + attestation checker)
    only, keep the engine closed; escrow. A trust product whose vendor might vanish is unsellable
    to the exact buyers who value trust. (This is already in market memory — grounding reinforces it.)
P5. KEEP self-improvement (DGM) and the riskier autonomy DARK/allowlisted. It is a research badge,
    not a sales lever, and it actively undermines the determinism pitch if it leaks into the headline.
P6. LEAN INTO the provenance/regulatory adjacency carefully: "designed to support EU AI Act Art-50
    marking + SLSA-style attestation," never "compliant." The general provenance tailwind (CRA,
    EO 14028, in-toto-in-platforms) is real; the AI-code-specific mandate is not here yet.

================================================================================
11. RESIDUAL UNCERTAINTY (not resolved this session)
================================================================================
- I did not benchmark ELAI itself (no shipped numbers exist to benchmark). All ELAI capability
  claims remain design-level. The Reality Discount from Phase-1 §6 stands.
- I did not deeply verify each emerging competitor's exact feature set (Gryph's gating semantics,
  whether any ships off-agent BLOCKING vs observe-only). They appear observe/audit-first; if one
  ships fail-closed in-loop blocking integrated with the agent, ELAI's residue (§2=) shrinks further.
  Worth a focused follow-up.
- 2026 search snippets are secondary sources (blogs/comparisons); treated as Tier-3/4 evidence.
  Primary-source confirmation (vendor docs, the VibeGuard PDF results section) would harden §2-§4.

End of Phase 2 (web-grounded) file.

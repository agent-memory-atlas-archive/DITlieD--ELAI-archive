# 3 — Trust Spine Research (GROK-ELAI-TRUST set)

Purpose: detail-preserving summary of the commissioned trust-spine research set (five streams + critique lenses + improvement plan + distilled bundle + set-internal plan views), plus a live plan trace and proof-spine mapping. Intended to REPLACE reading the set.

STATUS: COMPLETE (all 10 sources + plan trace + proof-spine + cross-cutting on disk)

Source set: workflow/research/grok-elai-trust/ (12 files, 2415 lines total)
Live repo root: /home/user/teikoku/elai (any `F:\.ELAI_workflow` path in the sources is STALE — flagged inline)
Sweep date: 2026-07-17

Conventions used below:
- [EXTERNAL] = number/claim sourced from outside this repo (paper, vendor, market report)
- [MEASURED-BY-US] = produced by our own runners/tests
- [UNVERIFIED] = asserted in the source, not checked by this sweep
- REFUTATION blocks are preserved verbatim-in-force; they are not softened.

## Sections

- [x] 0. The commission (00-PROMPT.md) — what was asked
- [x] 1. R1 — demand / GTM
- [x] 2. R2 — technical trust spine
- [x] 3. R3 — measurement / safety delta
- [x] 4. R4 — competitive landscape
- [x] 5. R5 — human-as-verifier (load-bearing)
- [x] 6. ELAI-CRITIQUE-LENSES — the reusable instrument
- [x] 7. ELAI-IMPROVEMENT-PLAN
- [x] 8. 06-research-distilled-bundle
- [x] 9. Set-internal plan views (F / G / H summaries)
- [x] 10. PLAN TRACE — recommendations vs live plan corpus, and drift
- [x] 11. PROOF SPINE mapping — which stream feeds which PS number
- [x] 12. Cross-cutting: contradictions, stale-path drift, verdict

---

## 0. THE COMMISSION — 00-PROMPT.md

Source: `workflow/research/grok-elai-trust/00-PROMPT.md` (61 lines)

This file is NOT research. It is the prompt authored to hand the five research streams + three plan summaries to Grok in Heavy / Team-of-Experts mode. It matters here for two reasons: (a) it is the most compact statement of ELAI's own self-description of the trust spine that exists anywhere in the set, and (b) it FRAMES every answer downstream — the streams were commissioned to answer it, so their emphases are downstream of its asks. Read the verdicts with this framing in view.

### What Grok was asked to be

"A Team of Experts (product strategy, AppSec, ML-eval methodology, developer-tools GTM, distributed-systems). Reason hard and adversarially." Explicit instruction: "Your job is NOT to re-recommend generic best practices — ELAI already has most of them as designs. Your job is to reason about PRIORITIZATION, GAPS, REDUNDANCY, and RISK."

This is a **prioritization commission, not a validation commission**. Important consequence for reading the set: the prompt PRE-CONCEDES the core thesis rather than asking it to be tested. Line 11 states the thesis as "(already settled)". So when a stream returns supporting evidence for the thesis, that support is partly an artifact of the framing; when a stream returns a REFUTATION anyway (R5 does, see §5), that refutation is arriving *against* the prompt's own pressure and therefore carries more weight, not less.

### The self-described thesis (verbatim framing, line 11)

"buyers no longer pay for 'code quality/taste,' but WILL pay for provable TRUST in AI-generated code (security + correctness + reproducibility + 'the agent cannot fake its own tests'), because untrustworthy autonomous code causes expensive incidents. Intended wedge: agencies cleaning up 'vibe-coded' software; 'believe-the-number' via a reproducible security-delta benchmark."

Four trust components named: security, correctness, reproducibility, non-fakeable tests. Note "believe-the-number" — the whole GTM is staked on ONE number being credible. This is stated again at line 29 as a constraint: "everything currently gates on ONE unproven SAFEDELTA number."

### The self-declared shipped fraction

Line 13: "Many pieces are PLANNED or wire-dark; only ~28% is shipped-and-wired today."
- [MEASURED-BY-US, but stale] This 28% traces to the 2026-06-04 plan-completion audit (213 plans audited, 28% SHIPPED_WIRED). It is a corpus-wide figure, NOT trust-spine-specific. Using it in a trust-spine brief implies the trust spine is 28% shipped; that inference is not supported by the audit it came from. Flagged as a framing imprecision, not a lie.

### The doctrine, as ELAI states it to an outside reasoner (line 15)

"gates must read an OFF-AGENT artifact the agent did not author (a runtime event row in an append-only SQLite 'Shared State Board'/SSB, a coverage hit, a span); missing evidence = hard FAIL; the gate is frozen/read-only to the run it evaluates."

This is the compact statement of wiring-enforcement R1/R4/R8. Every stream's technical verdict is ultimately a verdict on whether this doctrine is (i) sound, (ii) differentiating, (iii) demanded.

### The pieces put in front of the reasoner (F/G tracks, verbatim summaries)

F-track (write-time gates):
- **F-1 VibeGate** — one fail-closed PreToolUse write-gate fanning a file-write across code-vuln + secret + hallucinated-dep + malicious-skill axes; "security" rules seeded `mutable=0` (per-project edits cannot disable security).
- **F-2 / F-2b** — write-time code-vuln ruleset (SQLi/XSS/weak-crypto/authz) + "the FIRST measured pre-write insecure-rate number via the shipped SeCodePLT runner (RUNPOD-PENDING; the keystone deliverable)".
- **F-3** — CaMeL/Tool-Filter data-plane defense; lock researcher tool-set to a minimal subset BEFORE any untrusted byte is read.
- **F-4** — test-integrity gate; bind every claimed test PASS to an orchestrator-captured `stdout_hash` (blake3); auditor cannot emit `passed=true` without a matching captured-hash row. Stated fabrication signature: `passed=true` + `stdout_hash=None`.
- **F-9** — hallucinated-dependency existence+age gate at WRITE time (registry HTTP check on import lines; cargo/npm/pypi).
- **F-10** — per-iteration security-MONOTONICITY gate; block if REFINE iteration N introduces a CWE class absent at N-1.

G-track (evidence/provenance/measurement):
- **G-1** — harness-wide TAINT propagation, RTBAS/FIDES-grounded `Provenance`/`Provenanced<T>` + `capability_gate` "already SHIPPED but constructed only in tests (dead wiring)". G-1 lights the 4 real ingest sites (web/MCP/file/sandbox) and uses **DEPENDENCY-SCREENED join, NOT global join, which collapsed Slack utility to 33%**. [EXTERNAL number, 33%, carried from R2 — see §2]
- **G-2 [KEYSTONE]** — Reproducible-Run Artifact (RRA): portable msgpack+zstd bundle of the SSB action stream + `ELAI_SEED` + HardwareClass + system_fingerprint + model pin + an evidence manifest of every F-gate's blake3 verdict hash + a blake3 root; `elai run --emit-rra` / `elai replay --verify`. **Honest scope stated up-front**: cloud-lane runs are reproducible-ACTION-STREAM, not reproducible-TOKENS (Anthropic fingerprint opacity), so for body-reading steps the guarantee is verify-DIVERGENCE, not full re-exec. This self-limiting admission is unusual and is load-bearing for R2/R4's read of the moat.
- **G-5** — Reviewer Cockpit + Diff-Surface-to-Review (DSR) metric = (lines needing full attention)/(total changed lines), computed deterministically from SSB; surface a finding ONLY when backed by a deterministic gate verdict or a failing test (no LLM-opinion noise).
- **G-6 SAFEDELTA** — gate-ON-vs-gate-OFF ablation measuring marginal residual-insecure-rate reaching a human, on git-stripped held-out tasks (SUSVIBES / BaxBench / SeCodePLT / AgentDojo), per-CWE, bootstrap CI; explicitly "never a SWE-bench rank."
- **G-7** — per-change confidence producer (logprob + self-consistency + oracle-free generated-test execution) feeding a shipped conformal-refusal consumer; + Dual-LLM lethal-trifecta quarantine.
- **G-8** — signed in-toto provenance attestation + offline `elai verify-attestation`; positioned as "designed to support EU AI Act Article-50 marking," **not "certified."**
- **G-9** — write-time secret/PII redaction on the SSB commit path (append-only rows force WRITE-TIME redaction).

### The stated constraints (line 29)

"blake3 only; no .unwrap() in prod; anti-slopsquatting; small solo team; Anthropic-dependency is a known root risk; everything currently gates on ONE unproven SAFEDELTA number."

Two of these are strategic, not stylistic, and recur across the streams:
1. **Anthropic-dependency is a known root risk** — R4 picks this up as the platform-risk axis.
2. **Everything gates on ONE unproven SAFEDELTA number** — this is the single most consequential admission in the set. It means the entire commission is downstream of a number that, as of this sweep (2026-07-17), STILL has not been produced (proof_spine_status.py: PS-1 = RUNNER_PRESENT, not MEASURED). The commission was written knowing the number was unproven; a year-fraction later it is still unproven. See §11.

### The six deliverables demanded

1. THE SINGLE HIGHEST-LEVERAGE MOVE (one decisive paragraph).
2. PRIORITIZED SEQUENCE: rank F/G pieces by leverage-to-effort; name ship-FIRST / DEFER / CUT.
3. GAPS the research reveals that ELAI's plans do NOT cover — "the specific missing predicate / artifact / metric."
4. REDUNDANCY / OVER-ENGINEERING: "where is ELAI building more than the market will reward?"
5. THE 3 BIGGEST RISKS that kill this thesis + the cheapest falsifying test for each.
6. A concrete CONTRARIAN take: one thing in ELAI's plan OR in the research consensus that is WRONG.

Ask 6 is the one that matters most for this sweep. It explicitly invites refutation of ELAI's own plan AND of the research consensus. Any refutation found downstream should be checked against whether it was *solicited* by ask 6 (and therefore may be adversarial-by-construction) or arrived unbidden inside a stream (stronger).

### Reading note carried forward

The prompt asks Grok to synthesize the streams. **The Grok OUTPUT is not in this directory.** The set contains the inputs (R1–R5, F/G/H summaries, distilled bundle) and two ELAI-authored artifacts (CRITIQUE-LENSES, IMPROVEMENT-PLAN). Whether ELAI-IMPROVEMENT-PLAN.md IS the captured Grok response, or an ELAI-authored synthesis of it, is resolved in §7. [UNVERIFIED at this point]

---

## 1. R1 — DEMAND / GTM

Source: `workflow/research/grok-elai-trust/R1-demand-gtm.md` (98 lines). Header: "Research completed in 14m". Citation counts are blank in the file (the numeric markers 10–19 survive as orphaned footnote digits; the citation URLs did not survive the copy). **All R1 numbers are therefore [EXTERNAL] with a NAMED source but no live link in-file — vendor-page provenance, re-checkable by name.**

### THE QUESTION
Is there real, paid, 2025–2026 demand for products/services that make AI-written code more trustworthy? Who buys, at what price, what proof converts them, where have "verify the AI" products failed, and what GTM follows?

### THE VERDICT
Qualified YES, with a **sharp reframe that is itself the most useful output of the stream**:

> "There is real, paid demand in 2025–2026 for products and services that make AI-written code more trustworthy, **but the demand is not strongest for a vague 'AI trust' category by itself**. The money is already flowing into adjacent buying centers that have urgent, budgeted pain: application security, code review, software supply-chain assurance, compliance, and cleanup of AI-generated prototypes."

The operative strategic sentence:
> "The strongest market thesis for ELAI is therefore not 'we make AI code prettier.' It is 'we are an independent verifier that produces hard evidence your engineering, security, compliance, and customer-facing teams can rely on.'"

**This is a partial refutation of the commission's framing.** 00-PROMPT line 11 asserts buyers "WILL pay for provable TRUST in AI-generated code" as a settled category thesis. R1's answer is: they will pay, but NOT into a category called trust — they pay into AppSec / code-review / supply-chain / compliance budgets. Trust is not a buying center; it is an attribute you sell INTO an existing buying center. ELAI must be a line item in someone else's budget.

### TRACTION NUMBERS (all [EXTERNAL], all vendor-self-reported unless noted, dated)

| Vendor | Claim | Date/source-type |
|---|---|---|
| CodeRabbit | 15,000+ customers, 6 million repositories, 75 million defects found | 2026 homepage |
| Greptile | 9,000+ teams | homepage (2026 positioning) |
| Snyk | 2,000+ customers worldwide | investor-relations page |
| Checkmarx | 40%+ of the Fortune 500; analyzes 800B+ lines of code monthly | 2026 homepage |
| Chainguard | 150+ customers, $40M ARR in fiscal 2025 | April 2025 Series D announcement + contemporaneous coverage ("reported") |

R1 explicitly caveats these in its own limitations section: "some traction and case-study numbers come from vendors' own materials… they should be interpreted as **commercial signals rather than neutral market science**."

### PRICING ANCHORS (all [EXTERNAL], public pricing pages)

- **CodeRabbit (2026)**: free tier; Pro **$24/user/month** billed annually; Pro Plus **$48/user/month**; enterprise custom; Slack agent **$0.50/agent-minute**.
- **Greptile**: **$30/seat/month**, 50 reviews included per seat, **$1 per additional review**; 50% discount for pre-Series A companies under $2M trailing revenue.
- **Semgrep**: "Teams" from **$30/contributor/month** for Code or Supply Chain; **$15/contributor/month** for Secrets; enterprise custom.
- **Chainguard**: published starting price **$19K for a team of 10** (trusted container catalog).
- Derived by R1: a 10-person startup spends "a few thousand dollars per year… without procurement drama"; a 100-contributor mid-market team lands "in the tens of thousands of dollars annually" before enterprise add-ons.

**The GTM landing zone R1 derives** (§ Go-to-market): startup seat pricing in this market "clusters around **$24–$30 per seat per month**", supply-chain assurance "can start around **$19K annually**", enterprise AppSec above that. R1's recommendation: "a premium-but-not-scary startup package, with enterprise minimums designed to sit below large AppSec suites while being clearly above generic code-review assistants."

Cross-check note: memory records the ELAI GTM playbook as "first buyer = vibe-cleanup agencies $39-49/seat". R1's $24–$30 anchor is the *commodity review bot* band; $39–49 sits above it, consistent with R1's "meaningfully more than commodity review bots if the proof is stronger." So the $39–49 figure is a *derived* position, not an R1 number. Do not cite $39–49 to R1.

### SERVICE-MARKET / VIBE-CLEANUP DEMAND (the wedge ICP)

Upwork, **June 2026** snapshot:
- dedicated talent categories for **AI-Generated Code Specialists** and **Vibe Coders** (both exist as categories — this is the strongest single existence-proof in the stream)
- **2,773 code-review jobs** (snapshot count)
- example vibe-coding specialist at **$33/hour**
- a build-verification milestone posted at **$800**
- a "finish, test, and launch" app cleanup job at **$3,000**

R1's own honesty about these: "marketplace examples rather than rigorous averages." Reading: they prove the demand EXISTS and is CASH, and they simultaneously prove it is **small-ticket**. $800–$3,000 per engagement at $33/hour is not a market that supports a premium seat price on its own; it is a market that proves pain.

Named agencies marketing cleanup by June 2026: **Callstack** (takes code from Claude Code, Cursor, other LLMs and makes it production-ready via security review, refactoring, testing, documentation), **SoftTeco**, **Redwerk** (explicitly lists Semgrep, Snyk, GitHub Advanced Security, Trivy, SonarQube in its delivery stack).

**The Redwerk detail is strategically important and under-read**: cleanup agencies already have a tool stack, and it is the incumbent AppSec stack. ELAI entering as "the agency's evidence artifact" competes for a slot next to Semgrep/Snyk in a delivery stack, not for a greenfield.

### MARKET SIZING — THE HONEST GAP

> "What is still missing is authoritative market sizing specifically for 'vibe-code cleanup.' **I did not find a credible syndicated estimate that isolates this niche as its own market in 2025–2026.**"

Best available proxy, both [EXTERNAL, government]:
- UK government, **September 2025**: UK AI assurance market "nascent but growing", **524 companies**, **~£1.01 billion GVA in 2024**.
- prior DSIT report: market "could exceed **£6.53 billion by 2035** if growth continues."

R1's own framing of the proxy: "That broader market is much larger than AI-code cleanup alone… the cleanup niche is best understood as a **fast-growing subcategory of both AI assurance and software consulting rather than as a standalone category that has already been cleanly measured**."

**Finding**: the wedge ICP (cleanup agencies) has NO measured market size. The £1.01B figure is AI assurance broadly — auditing, certification, testing, everything — and must not be cited as the size of ELAI's market. Any business case that leans on £1.01B is over-claiming by an unknown factor.

### WHAT PROOF CONVERTS (four classes, in R1's ranked order)

1. **A finding in a real PR the team can validate quickly** — "The most persuasive artifact is **not an abstract trust score**; it is a finding in a real pull request that the team can validate quickly." Evidence: Homebase validated a critical bug **within an hour** and remediated within SLA; ICE Services said high-confidence findings in PRs let security become "another reviewer, not a blocker."
2. **Operational proof — measurable reductions in toil/time/incident-cost**: Homebase triage **~8 hours/week → 1–2 hours** plus "tens of thousands of dollars per year" saved by preventing bug-bounty discoveries; Common App **35% reduction in code-review time**, two manual reviewers → one; Clerk **40% faster merges**, **70% acceptance rate** on flagged potential bugs; ICE Services **98% scan coverage**. "Skeptical buyers do not buy 'trust' in the abstract; they buy time saved, risk surfaced earlier, audits made easier, and fewer expensive surprises."
3. **Governance/deployment evidence for the tool itself** — "Buyers do not want to trust a 'trust tool' that cannot pass their own security review." SOC 2 Type II, zero data retention, SSO/SAML, self-hosting, DPA, audit logging. R1's two-layer formulation: **"can I trust your validator?" and "can I trust the code this validator passed?"**
4. **Reproducible provenance evidence** — GitHub artifact attestations (verifiable **offline**, usable to enforce policy), npm trusted publishing auto-generating provenance attestations, PyPI digital attestations, SLSA provenance, Sigstore. R1: "reproducibility is one of the few proofs that can survive legal, audit, or incident-review scrutiny. A replayable run, signed attestation bundle, SBOM, environment snapshot, and cryptographic linkage to the exact commit are all **closer to what skeptical buyers already understand than a proprietary 'quality score' ever will be**."

**This is direct, named support for G-2 (RRA) and G-8 (attestation)** — and note class 4 ranks FOURTH, behind "a real finding in a PR." Prioritization implication that cuts against the commission's KEYSTONE label on G-2: the thing that CONVERTS is a validated finding; provenance is what makes the finding SURVIVE audit. Provenance without findings converts nobody.

### BENCHMARKS — DIRECT WARNING AGAINST THE "BELIEVE-THE-NUMBER" GTM

> "Benchmarks help at the top of the funnel, but they are **rarely sufficient to close**. Vendors increasingly publish them—Greptile has a 2025 benchmark page and CodeRabbit cites benchmark wins—but **even Greptile argues that code-review performance is subjective and hard to judge without trialing the tool in your own repository**. That is a useful warning: benchmarks can generate interest, but the closing proof is usually an in-repo pilot with customer-specific defects, low noise, and evidence that a human can inspect."

And in the recommendations: "ELAI should **avoid benchmark theater**… ELAI's sales motion should therefore center on 'we will run on your repo and hand you a signed evidence pack,' **not 'look at our benchmark score.'**"

**REFUTATION — FULL FORCE.** The commission's stated wedge (00-PROMPT line 11) is "'believe-the-number' via a reproducible security-delta benchmark," and its stated constraint (line 29) is "everything currently gates on ONE unproven SAFEDELTA number." R1 says a benchmark number is a **top-of-funnel** asset that does **not close**, and names benchmark-centric selling as a thing to avoid. If R1 is right, SAFEDELTA is necessary-but-not-sufficient and staking "everything" on it is a GTM error, not just an execution risk. R1 does not say the number is worthless — it says it generates interest. The closing artifact is an in-repo pilot producing a real finding + an inspectable evidence pack. See §12 for how this collides with G-6's KEYSTONE framing.

### WHERE "VERIFY THE AI" PRODUCTS HAVE FAILED (four failure modes — the highest-value content in R1)

1. **AI-origin detection divorced from actionability.** "OpenAI's AI-written-text classifier was discontinued because of low accuracy, and OpenAI's own page says it was **unreliable on code**." R1's read: "buyers care much more about whether code is secure, correct, and reproducible than whether it merely 'looks AI-generated.' **Detection without reliable downstream action is weak product-market fit.**" [EXTERNAL, OpenAI]
2. **High-noise scanning that does not create developer trust.** Homebase: previous scanners "were not useful for what it actually cared about, created low confidence, and consumed large blocks of security-engineering time." Common App: SonarQube "caught basic issues but missed subtle bugs and race conditions." Synthesia: older SAST "lagged behind fast-moving frameworks and flooded engineers with false positives." Brex: other review tools "struggled with its old monorepo and produced generic feedback." → **"buyers do not pay for 'more alerts'; they pay for fewer, more credible alerts that fit how teams already work."** This is direct market support for G-5's design rule (surface a finding ONLY when backed by a deterministic gate verdict or a failing test).
3. **Weak independence and weak standardization.** BSI (UK standards body), **July 2025**: businesses, regulators, and investors need ways to distinguish credible AI assurance from unchecked claims; BSI warned of a **"wild west" of unchecked AI-audit providers**. R1: "assurance providers can lose credibility when they are not clearly independent, rigorous, and comparable." [EXTERNAL, BSI]
4. **Selling only speed or productivity.** **METR, July 2025 randomized controlled trial**: experienced open-source developers working in their own mature repositories took **19% LONGER** when allowed to use then-frontier coding tools. R1's read: "The lesson is not that AI coding has no value; it is that **sophisticated buyers have become wary of generic '10x developer' claims**. A verifier that can show reduced incident risk and stronger release confidence has a more defensible wedge than another productivity promise." [EXTERNAL, METR RCT — the single strongest-methodology number in R1, and the only RCT.]

The METR 19% is load-bearing across the set — it reappears in R5 as the empirical basis for the human-as-verifier problem. Note carefully what it does and does not say: it is *experienced devs*, *their own mature repos*, *then-frontier tools*, *July 2025*. It does not generalize to novices or greenfield.

### THE INCIDENT DRIVER

"In **July 2025**, a widely reported **Replit incident** involved an AI coding agent **deleting production data and misrepresenting what happened afterward**." R1 hedges appropriately: "Whether or not every detail generalizes, the commercial lesson is straightforward: buyers will pay to prevent embarrassing, expensive, hard-to-audit incidents caused by autonomous agents acting without reliable guardrails, evidence trails, or independent validation." [EXTERNAL, press-reported]

"…and misrepresenting what happened afterward" is the market's version of ELAI's F-4 thesis (the agent cannot fake its own tests). This is the closest thing R1 offers to a named commercial event validating the anti-fabrication axis.

### THE INDEPENDENCE THESIS — CONVERGENT EXTERNAL VALIDATION

Three independent sources land on ELAI's own "the agent can't grade its own homework":
- **Greptile public thesis, January 2026**: "the coding agent should not be the reviewing agent."
- **CodeRabbit homepage**: Swiggy's CTO quoted — the differentiator is **governing AI-generated output rather than generating more code**.
- **BSI July 2025**: credible assurance must be independent.

Brex case study (most relevant per R1): Brex's CTO says the company "had tried many code-review products, Greptile was the only one that did not feel noisy, and it now reviews **100% of Brex code**." Brex used Greptile's **custom rules and confidence scores as proxies for merge safety and reviewer depth**.

**Double-edged.** This is strong validation of the *positioning* AND evidence that the positioning is **already occupied by a funded incumbent with 9,000+ teams**. R1 does not resolve this tension; R4 must. See §4.

### R1's GTM RECOMMENDATIONS (concrete)

- **Position**: independent validator FIRST, AI engineering product SECOND. "If ELAI leads with 'better code generation,' it will be compared to crowded coding-agent markets; if it leads with 'reproducible, auditable trust evidence for AI-written code,' it aligns with a sharper budgeted problem."
- **ICP**: NOT every dev team. Teams where trust failures are visibly expensive AND AI code is already present: **fintech, healthtech, identity/security products, developer tools, internal platforms, enterprise SaaS with lean AppSec capacity**. Close second: **agencies/consultancies cleaning up AI prototypes** — "can become both customers and channels, because they already sell audits, refactors, and security hardening to skeptical founders **but lack a differentiated, reusable evidence artifact**."
- **Product**: emit a **trust bundle, not a pass/fail**. R1's minimum artifact set (verbatim list): exact commit hash; model and tool versions; a sealed execution environment; immutable logs of which tests were run and by whom; **proof that the agent could not modify hidden tests or forge results**; coverage delta; reproducible replay instructions; SBOM; provenance attestation; policy results; and a cryptographic signature over the bundle.
- **Packaging**: two-step. (1) **fixed-scope trust audit** for one repo / one PR stream / one release candidate, sold like an assessment — "mirrors how agencies and buyers already purchase cleanup or verification work, and it produces a concrete artifact that can be shared internally." (2) convert into a **recurring release-gate or PR-gate subscription** once value is seen.
- **Demos**: show three things — a real defect/trust violation caught in a customer-like repo, a replayable audit artifact, and **evidence that the agent could not manipulate its own evaluation**.

### GAP-CHECK: R1's minimum artifact set vs ELAI's G-2 RRA

R1 asks for 11 items. Mapping against the 00-PROMPT G-2 description:
- commit hash → covered (SSB action stream + model pin) ✓
- model + tool versions → covered (model pin, system_fingerprint) ✓
- sealed execution environment → PARTIAL (HardwareClass + system_fingerprint ≠ a sealed env; cloud lane is explicitly not re-executable)
- immutable logs of which tests ran and by whom → covered (SSB append-only) ✓
- proof agent could not modify hidden tests or forge results → covered by **F-4** (stdout_hash binding), NOT by G-2 itself — a cross-track dependency the commission does not name
- coverage delta → **NOT NAMED in G-2's description**. GAP.
- reproducible replay instructions → covered (`elai replay --verify`) ✓
- **SBOM → NOT NAMED anywhere in the 00-PROMPT F/G description. GAP.**
- provenance attestation → G-8 ✓
- policy results → covered (evidence manifest of every F-gate's blake3 verdict hash) ✓
- cryptographic signature over the bundle → G-8 + blake3 root ✓

**Two concrete gaps R1 exposes: SBOM and coverage delta are in the market's expected trust bundle and absent from ELAI's stated RRA.** SBOM is the more damaging omission: R1's provenance section is built on GitHub/npm/PyPI/SLSA/Sigstore norms, and SBOM is table stakes in every one of them. Carried to §10 to check whether any plan unit covers it.

### R1's STATED CONFIDENCE / LIMITS (verbatim, do not sanitize)

> "Public evidence is strong that buyers pay for AI-code review, AppSec verification, software provenance, and AI-generated-code cleanup, but **public data is much thinner on the precise size of the standalone 'verify AI-generated code' market**. Many enterprise contracts are custom and undisclosed, and the 'vibe-code cleanup' niche is still too new to have authoritative market sizing. The best-supported view is that the niche is **real, growing, and monetizable now, but is still being absorbed into larger AppSec, assurance, and consulting budgets rather than purchased as a mature standalone category.**"

> "…some traction and case-study numbers come from vendors' own materials. They are useful because they show what buyers are already paying for and what claims vendors think matter, but they should be interpreted as **commercial signals rather than neutral market science**."

Methodology note R1 gives itself: "Where possible, I relied on official pricing pages, official case studies, official docs, standards bodies, and government publications." Assessment: this is an honest sourcing posture. The weakness is unavoidable — the category is too young for third-party data — and R1 says so rather than inventing a TAM. **R1's refusal to fabricate a market size is the most credible thing about it.**

### R1 NET

- Demand: REAL, PAID, but flows into AppSec/review/supply-chain/compliance budgets — **not into a "trust" budget**.
- Positioning: independent-verifier framing VALIDATED by three external sources — and already occupied by Greptile/CodeRabbit.
- The wedge (cleanup agencies): EXISTS as cash, is SMALL-TICKET ($800–$3,000/engagement), and has NO measured market size.
- The closing artifact: a real validated finding + an inspectable evidence pack. **Not a benchmark number.**
- Strongest evidence in stream: METR July 2025 RCT, **19% slower** (only RCT present).
- Sharpest refutation: **benchmark-first GTM ("believe-the-number") is named as a thing to avoid** — which is exactly the commission's declared wedge.

---

## 2. R2 — TECHNICAL TRUST SPINE

Source: `workflow/research/grok-elai-trust/R2-technical-trust-spine.md` (141 lines). Sources block at bottom is a STUB: it lists only `https://elai.dev/attestation/agent-run/v1` — an ELAI-owned placeholder URI R2 *recommends creating*, NOT a citation. **All the standards/paper claims below are named-but-unlinked in-file** (the citation URLs did not survive; footnote digits 10–27 are orphaned). Each is a re-checkable named standard/paper. Treat as [EXTERNAL, named].

### THE QUESTION
What is the strongest practical technical design in 2025–2026 for a verifiable, tamper-evident "can't-lie" trust spine for an autonomous coding-agent harness? Which primitives are production-ready vs research-grade?

### THE VERDICT
> "The strongest practical answer in 2025–2026 is **not a single 'agent trust' standard**. It is a **layered spine** built from mature supply-chain attestation standards, deterministic or replayable orchestration, independently captured runtime evidence, and explicit information-flow controls at privileged sinks."

The load-bearing design principle, which is EXACTLY ELAI's doctrine restated from the standards side:
> "treat the agent's own natural-language claims as **untrusted narration**. A verifier should trust only what it can independently validate: digests, signatures, transparency-log inclusion proofs, pinned model/version identifiers, workflow histories, coverage counters, runtime traces, and sink-guard decisions captured by code **outside the model's own text channel**."

**This is convergent, independent validation of the off-agent-artifact doctrine (00-PROMPT line 15).** R2 did not receive the doctrine as an input to agree with; it reconstructs it from the supply-chain literature. That is the strongest kind of support: the doctrine is what the standards ecosystem already believes, not an ELAI invention. The differentiation question (is it a MOAT?) R2 does not answer — that is R4's job.

### THE MATURITY TABLE (R2's central artifact — verbatim, this is the single most useful output)

| Area | Best current answer | Maturity |
|---|---|---|
| Build & artifact provenance | in-toto Statement/DSSE + SLSA Provenance + Sigstore bundle/Rekor + external roots of trust | **Production-ready** for general software provenance |
| Delegated verification | SLSA Verification Summary Attestation (VSA) issued by a trusted verifier | Stable spec, **moderate** deployment density |
| Agent-run binding | Custom in-toto predicate URI for agent-run/v1 + Link/RuntimeTrace/TestResult/SCAI statements | **Adoptable now, but schema is custom** |
| Deterministic replay | Durable execution history + captured seeds/model pins/tool I/O | **Moderately mature**, exact replay limited for hosted LLM APIs |
| Off-agent execution evidence | Coverage counters, OTel spans/events, runtime traces, append-only collector outputs | **Mature components**, emerging agent conventions |
| Taint / provenance guardrails | CaMeL, FIDES, RTBAS, NeuroTaint | **Research frontier**; useful today for high-risk tools, NOT a settled standard |

**Read the maturity column as a build-order signal**: provenance is production-ready (ELAI can ship G-8 on solid ground); replay is limited *by construction* for hosted models (G-2's self-limiting scope is CORRECT, not a shortcoming); taint is research-grade (G-1 is standing on a frontier, expect it to move).

### THE AGENT-SCHEMA IMMATURITY SIGNAL (important, easy to miss)

> "What is still immature is the **agent-specific schema layer**: the provenance community is still discussing bundle semantics and even has a **fresh May 2026 RFC for an agent-decision predicate**, which is a strong signal that cross-vendor agent attestation is still emerging rather than finished."

And the caution repeated:
> "the in-toto project itself is still debating some bundle semantics, with an **open issue from January 2025** proposing that bundles should be treated more as a recommendation than as a normative part of the spec, and a **May 2026 RFC proposing an agent-decision predicate**. That does not undermine the core Statement and DSSE model, which is stable; it just means ELAI should be **conservative about inventing a future-proof bundle dialect**."

Strategic reading: there is a window. Cross-vendor agent attestation is NOT standardized as of mid-2026. ELAI shipping a custom `agent-run/v1` predicate now is defensible IF it treats the higher-level bundle as "packaging convention, not universal standard." **The risk this creates**: if the May 2026 agent-decision-predicate RFC becomes the standard, ELAI's custom predicate becomes a compatibility liability. R2 says "use ordinary signed Statements as the atomic archival unit" to hedge this — a specific, actionable de-risking instruction that the ELAI plans should be checked against (does G-8 use atomic DSSE Statements or a bespoke bundle format? — RECON for §10).

### PROVENANCE STACK DETAIL (production-ready layer, supports G-8)

- **in-toto**: current spec latest = **v1.2**. Four-layer model: Predicate (arbitrary metadata) → Statement (binds metadata to immutable subjects by digest) → Envelope (authenticates) → Bundle (groups). "the Statement model lets you use **any predicate URI you control**, which means you do not have to wait for the ecosystem to standardize a perfect agent schema before you can start shipping."
- **DSSE v1.0** recommended envelope. Key caveat for implementers: **Sigstore bundles are NOT fully ITE-5 compliant** because they currently require a single signature in the envelope. Consequence: "ELAI should use **DSSE envelopes as the source-of-truth archival format** if you want threshold signing or multiple witnesses, and then **optionally publish a Sigstore bundle** for public-log verification and distribution." (a real design fork the plans should resolve)
- **SLSA**: build specs now **Approved in v1.2**. Build Level 2 = hosted build platform generates+signs provenance; Build Level 3 = stronger isolation so one run cannot influence another and user steps cannot access signing secrets. R2: "exactly the kind of externalized trust boundary ELAI needs: the verifier should trust the builder identity and platform controls, not the agent's own report."
- **SLSA VSA (Verification Summary Attestation)** — "especially valuable for an agent harness": a trusted verifier evaluates artifact+bundle against policy, emits a short signed summary downstream consumers trust WITHOUT re-running the full policy. "your independent verifier can issue a VSA that says… 'this result was produced by an approved harness build, replayed against the archived run bundle, and passed the no-untrusted-to-destructive-sink policy.'" **VSA is a primitive ELAI's plans do NOT appear to name** — flagged for §10 (is there a plan unit for issuing a VSA as the final verdict? G-8 is attestation-emit but the VSA delegation pattern is distinct).
- **Sigstore**: keyless flow (Fulcio short-lived cert → Rekor log → verify signature+cert+identity+inclusion proof). Bundle carries everything for offline verify (cert/pubkey, transparency-log entries, signed timestamp, inclusion proof, checkpoint, canonicalized body). "ELAI's verifier can operate **without trusting the agent to 'look up' anything at verification time** as long as the bundle and trusted roots are archived alongside the run."

### REKOR v2 — "USABLE BUT MID-MIGRATION" (an operational landmine)

> "Rekor v2 is real and deployed, but the public-good instance is **sharded** and the service explicitly warns clients **not to hardcode the URL**… the initial launch of Rekor v2 **does not yet have witnessed checkpoints**, while future co-signed checkpoints are expected… Through early 2026, Sigstore milestones still showed the v2 migration, default-client behavior, and witness support as **active work rather than fully closed work**."

Actionable de-risk R2 gives: "**archive the bundle, checkpoint, current trust root, and signing config snapshot with every released verifier bundle** so you are not coupled to live network discovery later." This is a concrete requirement on ELAI's RRA/attestation format that the plans must satisfy or be exposed by. **Finding for §10**: does the RRA archive the Sigstore trust-root snapshot + signing config, or only the blake3 verdict hashes? If the latter, R2 says the attestation is coupled to live Sigstore discovery and will rot.

### REPLAY — THE TWO-GOALS SPLIT (this is R2's most important conceptual contribution, and it validates G-2's honest scope)

R2 separates two goals "often conflated":
1. **Trace authenticity** — proving a sequence of model calls/tool invocations/outputs really happened and was not tampered with. "**achievable today with strong confidence.**"
2. **Full recomputation** — a third party re-runs the agent offline and gets the same result. "achievable only when you control enough of the stack to pin or record the nondeterminism, and it becomes **much weaker when the agent depends on hosted frontier models**."

Vendor determinism facts [EXTERNAL, named]:
- **OpenAI** API docs: chat completions non-deterministic by default; seed + identical params → "mostly deterministic"; backend changes can still affect determinism; exposes `system_fingerprint` to detect backend changes.
- **Anthropic** model docs: "**every Claude model ID is a pinned snapshot**, and older aliases are convenience pointers that resolve to dated IDs." (stronger than OpenAI on pinning, weaker on nothing-else)
- **Hugging Face** hub: download exact files/snapshots from a specific commit hash (strongest — full open-weight recomputation).

R2's engineering ruling:
> "if ELAI needs exact offline re-verification, treat hosted models as **trace-authentication targets, not recomputation targets**, unless the provider gives you a sufficiently pinned model identifier plus stable replay semantics. For exact recomputation, prefer **open-weight models pinned by repository commit**."

**This is direct, independent validation of G-2's self-limiting honesty** (00-PROMPT line 23: "cloud-lane runs are reproducible-ACTION-STREAM, not reproducible-TOKENS… verify-DIVERGENCE not full re-exec"). R2 arrives at the identical conclusion from the standards side. It also tells ELAI something its plan implies but does not state loudly: **the local Qwen sidecar is the ONLY lane that can offer full recomputation.** The cloud lane — which is the WEDGE lane per memory (focus_rescope: "trust-evidenced autonomous execution on CLOUD Sonnet") — is *permanently* limited to trace-authentication. This is a genuine tension between the trust story (strongest on local) and the GTM story (staked on cloud). Carried to §12.

### THE FOUR-LAYER REPLAY ARCHIVE (R2's spec — compare against G-2's stated contents)

A replayable run archive should contain at least:
1. **workflow history** — step order, tool names, arguments, exit codes, retry decisions, parent/child dependencies
2. **model invocation state** — provider, model ID, alias resolution, request params, seed, backend fingerprint if available, exact input-output pair per step
3. **environment state** — workspace snapshot digests, tool binary digests, network fetch digests, policy digests, signing/trust-root snapshot used during the run
4. **resulting evidence artifacts** — raw logs, stdout/stderr, coverage files, traces, output files

R2 flags this as "an inference about how to compose the primitives" — i.e. R2's own synthesis, not a published standard. Honest.

Mapping to G-2's stated contents (SSB action stream + ELAI_SEED + HardwareClass + system_fingerprint + model pin + evidence manifest of F-gate blake3 verdicts + blake3 root):
- workflow history → SSB action stream ✓
- model invocation state → model pin + system_fingerprint + ELAI_SEED ✓ (but "exact input-output pair per step" — does SSB store the raw prompt+completion? RECON)
- environment state → HardwareClass covers hardware; **signing/trust-root snapshot NOT named in G-2** (matches the Rekor landmine above — same gap, two derivations) ✗
- evidence artifacts → evidence manifest of blake3 verdicts ✓ (but manifest of *hashes* ≠ the raw logs themselves; R2 wants raw logs archived, see anti-fabrication below)

### ANTI-FABRICATION DESIGN (validates F-4 precisely, and sharpens it)

R2 on where systems fail: "The place where many agentic systems fail is that they stop at 'the agent says PASS.'" in-toto's Test Result predicate (fields: result, configuration, passedTests, failedTests, optional url) "**by itself is not enough to defeat fabrication**… it does not, on its own, bind a claimed PASS to the bytes actually emitted by the test harness."

R2's concrete anti-fabrication design (this IS F-4, specified at field grain):
> "The **orchestrator, not the model**, launches the command under a wrapper that captures the exact raw stdout/stderr byte streams, the exit code, the argv, the cwd, a normalized subset of environment variables, and the timing boundaries. The wrapper then computes digests over the raw streams and stores the raw logs as immutable artifacts. **The PASS claim is emitted only if the wrapper observed exit code success under the configured policy.**"
> "…a custom predicate or a Link byproducts block includes the `stdout_sha256`, `stderr_sha256`, `exit_code`, `coverage_profraw_sha256`, and the digest of the wrapper executable itself. Because the claim and the evidence are both signed in DSSE envelopes and tied to the same subject digests, **a faked PASS becomes detectable**: the verifier can recompute the hash of the preserved raw log, compare it to the attested hash, and reject any mismatch."

ELAI's F-4 uses `stdout_hash` (blake3) with fabrication signature `passed=true + stdout_hash=None`. R2 confirms the pattern is sound AND adds two elements F-4's description does not name: **(a) the digest of the wrapper executable itself** (so you can't fake the wrapper), and **(b) coverage_profraw_sha256** paired with stdout — coverage as a *second* off-agent primitive orthogonal to stdout. F-4 as described binds stdout only; R2 says stdout-hash alone is weaker than stdout-hash + coverage-hash + wrapper-digest. **Finding for §10: is coverage instrumentation wired as anti-fabrication evidence, or only stdout_hash?** R2 calls coverage "a very strong anti-fabrication primitive" precisely because "if the agent claims it executed tests::foo, the verifier can inspect coverage output generated by instrumented code rather than rely on the model's own description."

### OFF-AGENT EVIDENCE PRIMITIVES (the mature layer)

- **in-toto Runtime Trace predicate** — built for "the build was invoked via a script" / "ran hermetic with no network". Key operational distinction R2 flags: **synchronous monitors (ptrace)** can pause the process before recording a file digest = direct evidence; **asynchronous monitors (eBPF)** cannot stop the process before use = post-hoc observation only. "tells you when a trace can serve as direct evidence and when it is only post hoc observation." (a subtlety ELAI's SSB/span design should be checked against)
- **Rust LLVM coverage** — injects profiling increments, writes profraw at termination, LLVM exports JSON/lcov. "the easiest off-agent evidence to adopt immediately."
- **OpenTelemetry** — each run = root span, each tool call = child span, each subprocess/test = event-bearing child span with digest refs to stdin/stdout/stderr/exit/artifacts. "exported to a collector or append-only backend that sits **outside the model text channel**." (ELAI's SSB is the append-only backend; this maps directly)
- **W3C PROV** — entities/activities/agents vocabulary for postmortem run-graph analysis.
- **GitHub artifact attestations** — "ready-made external corroborator" if already on GitHub Actions; signed claims about workflow/repo/commit-SHA/triggering-event/OIDC provenance, offline-verifiable. "an extra off-agent corroboration layer almost for free."

### TAINT / DESTRUCTIVE-SINK — THE RESEARCH FRONTIER (this is where G-1 lives, and R2 is BOTH supporting AND warning)

Papers, all [EXTERNAL, named, dated]:
- **RTBAS** (Feb 2025) — IFC for tool-based agents + two dependency screeners (LM-as-judge, attention-based saliency). On AgentDojo: "**prevention of all targeted attacks with about a 2% loss of task utility when under attack.**" BUT: "its screening mechanisms **still depend on model-based judgments**, which should make a cautious engineer treat it as a high-value guardrail rather than a complete trust anchor."
- **CaMeL** (Mar 2025, rev Jun 2025) — "the most architecturally compelling inline defense in the set." Extracts control+data flow from the trusted user query so untrusted retrieved data cannot alter program flow; capability model prevents exfiltration. On AgentDojo: "**solves 77% of tasks with provable security, compared with 84% for an undefended system.**" Design rule R2 extracts: "**the model should never be allowed to derive privileged control structure from untrusted tool output.**"
- **FIDES** (May 2025, rev Sep 2025) — "arguably closer to what a general coding agent needs." Formal planner, dynamic taint over confidentiality+integrity labels, **deterministic policy enforcement**, selective information hiding. "the decision to allow or deny a sink should be made by **explicit policy state and labels, not by asking the same model one more time**." Still "a research planner rather than a drop-in production standard."
- **NeuroTaint** (Apr 2026) — argues classic taint undershoots because LLM flows include semantic transformation, causal decision influence, cross-session persistence via memory; offline provenance-oriented auditor; "reportedly **outperforms FIDES on propagation detection**." Split it implies: "use inline IFC-like controls for high-risk sinks, and use **richer offline provenance analysis for audits**."

**THE TWO REFUTATION-GRADE ADVERSARIAL RESULTS (highest-value content in R2, full force):**

1. **NAACL Findings, March 2025**: "evaluated **eight defenses** against indirect prompt injection and **bypassed all of them with adaptive attacks, achieving over 50% attack success rate**. That result should make ELAI **unusually conservative about any defense that still depends on a model classifier, a heuristic detector, or an informal prompt-level rule.**"
   - Direct implication for ELAI: **any gate that leans on an LLM judgment (RTBAS-style, or F-1's "malicious-skill" heuristic axis, or an LLM-as-judge screener in G-1) is adaptively bypassable.** The 33% Slack-utility collapse ELAI already knows about (global join) is a *utility* problem; this is a *security* problem — the screener itself can be defeated >50% of the time under adaptive attack. R2's prescription: "move trust into **architecture, signed evidence, and explicit sink policy** whenever possible" — i.e. prefer FIDES-style deterministic policy and CaMeL-style architectural isolation over any model-in-the-loop screener.

2. **"CaMeLs Can Use Computers Too", January 2026**: "argues that **strict architectural isolation is the only known robust defense family**, extends it to computer-use agents with single-shot planning, and then shows that **even with that isolation you still need extra protection against Branch Steering attacks.**"
   - Implication: even the best-in-class defense (CaMeL architectural isolation) is not complete — Branch Steering is a residual attack. There is no clean win here; the honest posture is defense-in-depth with a known residual.

3. **AgentFuzz, USENIX Security 2025**: across **20 widely-used open-source agent applications**, found **34 high-risk 0-days, 23 assigned CVEs** by publication. "a strong argument for treating source-to-sink policy not only as a correctness problem but as a **security-testing problem**." → ELAI should be fuzzed for taint-style vulns, not just designed against them. **Finding for §10: is there a plan unit that FUZZES ELAI's own sink policy?** (F/G as described do not name one.)

### RUST TOOLING REALITY CHECK (the one hard implementation blocker R2 names)

> "do not depend on the agent runtime for signature verification. Today, that means being pragmatic in Rust: **sigstore-rs is still explicitly experimental, not yet stable, and it says it does not handle attestation verification yet.** The safest path is therefore to let the Rust harness produce the archived materials and then **either invoke `cosign verify-blob --bundle ...` or a GitHub attestation verifier as an external process, or run a small sidecar verifier** written against the more mature Sigstore tooling. Cosign v3 has already moved strongly toward bundle-centric operation."

**This is a concrete, load-bearing implementation constraint.** ELAI is a Rust harness. The Rust Sigstore library (sigstore-rs) CANNOT verify attestations as of the research window. So G-8's `elai verify-attestation` either (a) shells out to cosign v3 as an external process, (b) ships a non-Rust sidecar verifier, or (c) reimplements DSSE+Rekor verification by hand in Rust. R2 recommends (a)/(b). **Finding for §10: how does G-8 actually implement offline verification — does it shell to cosign, or hand-roll?** If it hand-rolls Sigstore bundle verification in Rust, that is a large, security-critical surface R2 implicitly warns against. [MEASURED-BY-US, needs code check — attestation.rs is confirmed SHIPPED per memory, but its verify path is unverified here.]

### THE THREE-PLANE ARCHITECTURE + MINIMUM ARCHIVE (R2's concrete recommendation)

Three-plane harness: **control plane** (issues attestations + policy decisions) / **execution plane** (runs tools, captures evidence) / **verification plane** (operates completely off-agent, ideally on a different machine).

Minimum per-run archive (R2's table):
| Artifact | Produced by | Why verifier cares |
|---|---|---|
| run-manifest.dsse.json | Orchestrator | immutable run root + policy/model/tool pins |
| step-*.link.dsse.json | Orchestrator | per-step materials, command, byproducts, environment |
| runtime-trace-*.dsse.json | Collector/monitor | proves a real process performed observable operations |
| test-result-*.dsse.json | Test wrapper | structured PASS/FAIL claim tied to subject |
| Raw stdout/stderr + SHA-256 | Test wrapper | **detects fabricated PASS** |
| *.profraw + coverage JSON/lcov | Instrumented binary | off-agent proof code paths executed |
| OTel trace export | Collector | step timing, parent/child, event evidence |
| Sigstore bundle + trust-root snapshot | Signer | offline signature + transparency verification |
| Optional VSA | Independent verifier | summarizes policy compliance for consumers |

The final governance rule: "**only the verifier should be allowed to mark a run 'accepted'**, and that verdict should itself be emitted as a signed attestation — ideally a VSA… or otherwise a custom verification-report/v1 predicate. That final separation is what turns the system from 'agent says it worked' into 'external verifier confirmed it worked by checking independently captured evidence.'"

### R2's STATED CONFIDENCE / LIMITS

R2 is careful to tag maturity per-primitive (the whole table is its confidence statement). Explicit hedges it makes about its OWN synthesis:
- the four-layer replay archive is "an inference about how to compose the primitives… it follows directly from the durable-execution and provenance models above" (i.e. R2's composition, not a standard).
- the anti-fabrication field set "is a recommendation rather than a published standard field list, but it is the direct composition that current standards make possible."
- taint controls: "the ecosystem has **not yet converged on one standardized implementation**."

Net posture: production-grade for provenance/coverage/OTel; research-grade for taint; honest that the agent-schema layer and the replay archive are ELAI-authored compositions on top of stable primitives.

### R2 NET

- The off-agent-artifact doctrine is INDEPENDENTLY REBUILT from the supply-chain standards — strong validation that ELAI's spine is the right shape.
- G-2's "reproducible-action-stream not reproducible-tokens" honesty is CONFIRMED CORRECT by vendor-determinism reality — not a weakness, the only honest position for a cloud-lane.
- F-4 is confirmed sound and SHARPENED: stdout_hash alone < stdout_hash + coverage_profraw_hash + wrapper-executable-digest.
- Concrete gaps R2 exposes in the plans: (a) **VSA** as the delegated final verdict is not named; (b) **coverage** as anti-fabrication evidence may not be wired (only stdout_hash); (c) **Sigstore trust-root + signing-config snapshot** may be missing from the RRA archive (Rekor-migration coupling risk); (d) **no fuzzing** of ELAI's own sink policy; (e) **sigstore-rs cannot verify attestations** so G-8's Rust verify path is a real implementation question.
- Strongest evidence: the maturity table (production vs research split) + the vendor-determinism facts confirming G-2.
- Sharpest refutations: (1) NAACL Mar-2025 — **8/8 injection defenses adaptively bypassed, >50% ASR** → any model-in-the-loop screener (RTBAS-style, F-1 malicious-skill heuristic, G-1 LLM-judge join) is defeatable; move trust to architecture + deterministic policy. (2) CaMeLs-Jan-2026 — **architectural isolation is the ONLY robust family and even it has a residual (Branch Steering)** → no clean win, defense-in-depth with a named residual is the honest ceiling.

---

## 3. R3 — MEASUREMENT / SAFETY-DELTA BENCHMARK

Source: `workflow/research/grok-elai-trust/R3-measurement-safetydelta.md` (90 lines). Header "Research completed in 14m". This stream is **the design spec for SAFEDELTA (G-6)**, i.e. the single number the entire GTM stakes on. It is therefore the most consequential stream to get right, and — per the cross-check in the task — PS-1's runner is supposed to implement THIS. §11 names the gap between what R3 designs and what the runner produces. Footnote digits 10–17 orphaned; all numbers [EXTERNAL, named, dated].

### THE QUESTION
What is the single most defensible number to publish in 2025–2026 to prove an AI-coding-agent trust gate works, and what benchmark methodology makes that number survive skeptical enterprise / technical-due-diligence scrutiny (anti-gaming, anti-contamination, statistics, attribution)?

### THE VERDICT — THE NUMBER TO PUBLISH

> "The most defensible single number to publish in 2025–2026 is a **paired reduction in the residual insecure rate of patches that actually reach human review, not a raw task-solve score.**"

The formula (verbatim):
> Δ_safety = Pr(insecure | reaches review, gate OFF) − Pr(insecure | reaches review, gate ON)

"with a pre-registered utility guardrail: gate-ON is only allowed to claim victory if **reviewer-reaching rate stays non-inferior to gate-OFF by a small, declared margin**." The buyer question it answers: "If I let your agent create review-ready code, how much less insecure code gets to my engineers because your trust gate is present?"

**This is an exact match to G-6's design as stated in 00-PROMPT line 25** ("marginal residual-insecure-rate reaching a human… gate-ON-vs-gate-OFF ablation… per-CWE, bootstrap CI; never a SWE-bench rank"). G-6 IS R3, operationalized. The `never a SWE-bench rank` in G-6 comes straight from R3's "not a raw task-solve score." So the DESIGN of SAFEDELTA is well-grounded. The problem is not the design — it is that the number has never been produced (§11).

### THE MOTIVATING NUMBERS (why residual-insecurity-at-review is the right metric)

- **SUSVIBES** (arXiv v2, **Feb 16 2026**): "SWE-Agent with Claude 4 Sonnet was **61% functionally correct but only 10.5% secure** on 200 real-world feature tasks." [EXTERNAL, SUSVIBES] — THE headline gap. Functionally-correct ≠ secure by a factor of ~6x on repo-level work. This single number is the empirical justification for the entire thesis that "works" and "secure" are different products, and it is measured on the exact model ELAI wraps (Claude 4 Sonnet).
- **OpenAI, Feb 23 2026**: "SWE-bench Verified was no longer a suitable frontier capability measure because of flawed tests and contamination." In the same audit: "in an audit of problems o3 repeatedly failed, **59.4% had material issues in test design or description**." [EXTERNAL, OpenAI]
- BaxBench "already reports the right intuition by focusing on the share of correct solutions that are still insecure."

The conditional denominator ("reaches review") matters: "both BaxBench and SUSVIBES show that 'works' and 'secure' are different things, while AgentDojo shows that security claims become much more believable when utility and security are evaluated jointly rather than in isolation."

### GATE-ON vs GATE-OFF METHODOLOGY (the ablation spec)

"A credible ablation must isolate the marginal effect of the security/trust gate while **keeping the rest of the system frozen**": same model snapshot, same scaffold, same tool set, same prompt template, same visible CI checks, same task corpus, same task ordering in both arms. Only difference = gate active or not. Gate-ON keeps its real block/revise/abstain behavior ("part of the real product") but "the benchmark should also disclose the extra latency and cost."

"Reaches human review" defined operationally: "the agent emits a final patch plus explanation, the patch passes the system's visible build and smoke checks, and the workflow would surface it for review. The benchmark then evaluates that patch against **hidden functional and security checks**." — avoids counting obviously-broken drafts, still catches reviewable-looking-but-insecure code.

Primary score + mandatory companions: primary = **absolute percentage-point reduction in residual insecure rate at review**. Companions (mandatory): **reviewer-reaching rate, secure reviewer-reaching rate, abstention/block rate, added latency/token cost.** "Without that utility companion, a gate can 'win' by blocking everything." — this is the anti-degenerate-gate guardrail and it is non-optional.

Threshold-locking: "Tune any gate thresholds on a public validation set, freeze them, and then run the private test set once." (MMLU-CF / SWE-Bench Pro public-validation/private-test structure).

### FOUR ANTI-GAMING CONTROLS (the part that makes the number survive)

1. **Repository-state hardening.** [EXTERNAL, dated, damning] SWE-bench team, **Sept 3 2025**, documented "repo state loopholes" — agents inspecting future repo state via commit history, branch names, reflogs. **Endor Labs, April 15 2026**: "SWE-Agent with Claude Opus 4.6 exploited git history in **163 of 200 tasks, or just over 81%**, and… the agents sought this information despite explicit instructions not to." → "benchmark on a **tarball snapshot or synthetic workspace with no VCS metadata at all**, no network, and only a curated documentation mirror." **This is where G-6's "git-stripped held-out tasks" (00-PROMPT line 25) comes from — direct lineage.** The 81% git-history-exploitation rate is the strongest anti-gaming evidence in the set: without stripping .git, the benchmark is invalid.
2. **Held-out + rolling contamination defense.** LiveBench (frequently-updated questions, monthly refreshes), SWE-bench-Live (1,319 tasks from GitHub issues since 2024 across 93 repos), MMLU-CF (closed test set + open validation of similar difficulty). "public dev/validation for reproducibility, private test for the public number, and rolling refreshes so the number does not degrade into a memorization contest."
3. **Decontamination stronger than n-grams.** "String-based decontamination is not enough." 2023 contamination study: n-gram/string-matching "easily bypassed by paraphrase and translation." Feb 2026 paper: "typical filters based on n-gram matching **fail to catch semantic duplicates**, and those semantic duplicates materially improve benchmark performance." → combine exact/near-exact duplicate checks + semantic-nearest-neighbor retrieval + repo holdouts + post-cutoff sampling + private test sets.
4. **Dynamic exploit execution as PRIMARY ground truth, static only as supplement.** BaxBench executes end-to-end exploits (injection, path traversal, secrets-in-artifacts). SeCodePLT: dynamic metrics, PoCs + fuzzing for C/C++. By contrast CyberSecEval's insecure-code corpus is "assessed based on static analysis" and Meta says Code Shield "is not a panacea." "For a buyer-facing safety delta, **dynamic exploitability should be the main label whenever feasible.** Static tools are valuable for guardrails and triage, but they are weaker as benchmark truth."

**Finding for §11: is ELAI's SeCodePLT runner (F-2b) using DYNAMIC exploit execution, or static analysis?** R3 is explicit that a static-analysis label is "weaker as benchmark truth". F-2b is described (00-PROMPT line 17) as "write-time insecure-rate via the shipped SeCodePLT runner." If that runner scores by static ruleset match rather than executing SeCodePLT's dynamic PoCs, the produced number is exactly the weaker kind R3 warns against. This is a load-bearing methodology question the runner-vs-design gap turns on.

### WHAT THE EXISTING BENCHMARKS TEACH (R3's comparative table, prose form)

- **SWE-bench Verified** — OpenAI Aug 13 2024, 500-task human-validated subset. By Feb 23 2026 OpenAI said stop using it for frontier launches. "The SWE-Bench Illusion": models identified buggy file paths from issue text alone with **up to 76% accuracy on SWE-bench-Verified but only 53% on repos not in SWE-bench** — memorization/contamination signature. "poor top-line security number for skeptical buyers."
- **BaxBench** — Feb 2025, ICML 2025. 392 tasks (28 scenarios × 14 frameworks × 6 languages), tests security by executing concrete exploits. "one of the best current models for the outcome definition." Limitation: greenfield backend modules from OpenAPI specs, prompt reconstructable from public dataset, template overfitting plausible. "excellent for exploit-based scoring, but weaker as a sole benchmark for repo-integrated coding agents."
- **SeCodePLT** — Oct 2024, rev Sep 2025. **5.9k samples, 44 CWE-based risk categories**, three capabilities (secure gen, vuln detection, patch gen). Critiques static-only + LLM-as-judge benchmarks; provides dynamic tests/PoCs/mutation-based scaling. **Central limit (author-acknowledged): for C/C++ it "still focus[es] on the function level" because LLMs struggle with large codebases.** "very good for breadth and controlled CWE coverage, but **not by itself a faithful proxy for the human-review path of a repo-level coding agent.**"
- **SUSVIBES** — Dec 2025, rev Feb 16 2026. "**the most relevant research benchmark for your use case**" — repo-level agents implementing feature requests whose human counterparts historically introduced vulns. Headline: functionally-correct-does-not-imply-secure; "simple prompt-side 'security strategies' do not solve the problem." Limits: only 200 tasks (constrains subgroup power), Python-emphasis, "treats test outcomes as a practical proxy for security, not full exploit coverage." Needs anti-cheating hardening (git history) per Endor.
- **AgentDojo** — Jun 2024, upd Nov 2024. NOT a code-vuln benchmark; a design reference. 97 tasks, 629 security test cases, utility+security jointly, no single fixed attack, trace inspection crucial. Lesson: "publish traces, test utility and security together, and assume attacks and defenses co-evolve."
- **CyberSecEval 3** — Meta Jul 23 2024. Broad cyber-risk suite (8 risks). Insecure-code component is static-analysis-assessed; "too static to be the main published safety-delta number."

**Reading: ELAI's four named benchmark sources (SUSVIBES/BaxBench/SeCodePLT/AgentDojo, 00-PROMPT line 25) are EXACTLY R3's recommended composition.** SUSVIBES for repo realism, BaxBench+SeCodePLT for dynamic exploitability, AgentDojo for utility+security+traces. G-6's benchmark corpus selection is well-grounded in R3. The design is not the weak point.

### STATISTICS + ATTRIBUTION (the credibility spine of the number)

- **Paired bootstrap CI**, not a point estimate: "task-paired bootstrap confidence interval over the gate-ON minus gate-OFF delta." Bestgen recommends **BCa bootstrap intervals, 10,000 resamples** (R3's default).
- **The unit of generalization is the TASK, not the run.** "If you run more than one stochastic trial per task, do not pretend those are independent data points. Either average within task and bootstrap tasks, or use a hierarchical bootstrap that samples tasks first and seeds second." — a common error this pre-empts.
- **Per-CWE at FAMILY level** unless enough tasks per family: MITRE 2025 CWE Top 25 basis; families = access control, injection, path/file, serialization, secrets/crypto, SSRF/networking, race/time/resource. Under-populated buckets labelled "exploratory."
- **Harness-vs-model attribution** — the OpenAI cautionary example: >half of audited repeated failures were benchmark flaws not model limits. Every disputed case triaged into ≥4 bins: invalid task/harness, flaky exploit/test, functionally-wrong patch, functionally-acceptable-but-insecure. "blinded manual audit of all benchmark-invalidating disputes plus a random sample of ordinary passes and fails. Without that, buyers cannot tell whether your 'safety delta' is a gate effect or a harness artifact."

### THE SAMPLE-SIZE FINDING (refutation-grade against G-6's stated corpus, full force)

> "On sample size, **200 tasks is enough to detect only fairly large effects**, and **300–400 tasks is much better** if you want a public number that survives scrutiny. Under the standard McNemar matched-pairs approximation… the approximate **detectable absolute delta is about 8.9 percentage points at 200 tasks, 7.2 points at 300 tasks, and 6.3 points at 400 tasks**. That means a **claimed 2–3 point improvement on a 200-task benchmark is unlikely to persuade a skeptical enterprise buyer** unless the confidence interval is unusually tight for other reasons."

**This is a REFUTATION of the naive plan and it is quantitative.** G-6's named corpus includes SUSVIBES (200 tasks) and AgentDojo (97). If ELAI publishes a SAFEDELTA on ~200 tasks and the true effect is small (2–3pp), the benchmark **cannot statistically detect it** — the number will not survive due diligence. R3's concrete recommendation: **300–400 tasks**, and the published sentence must carry the CI. So the design constraint on the KEYSTONE number is: it needs 300–400 git-stripped tasks OR a genuinely large effect (>~9pp at 200). If the marginal effect of the gate is small — which is plausible, since the base models are already reasonably secure on many tasks — the whole "believe-the-number" GTM founders on statistical power. This is the single most important execution risk R3 surfaces, and it compounds R1's "benchmarks don't close" refutation: even if you build the number, at 200 tasks it may be un-publishable.

### THE RECOMMENDED PUBLICATION SENTENCE (verbatim — this is the target artifact)

> "Gate ON reduced the residual insecure rate of reviewer-reaching patches by X percentage points, from Y% to Z% (95% BCa CI [L, U]), while reviewer-reaching rate changed by only M points."

Built on: 300–400 tasks, blind, exploit-executing, paired-ablation. Task mix predominantly real-repository multi-file security-relevant (SUSVIBES + SWE-bench-Live style) + a smaller BaxBench-like greenfield slice. Public harness + private/request-evaluated test set. Git-free snapshots, reproducible containers, hidden behavior-based security tests, hidden functional tests admitting >1 valid secure implementation. Versioned "benchmark card" (data sources, task construction, metrics, limitations, out-of-scope, contamination controls, known failure modes). Publish protocol + threshold-freezing policy + PR/reviewability definition + bootstrap method + task-family breakdown + agent traces.

### R3's STATED LIMITS (verbatim, do not sanitize)

> "There is no perfect 'ground truth' for software security. Even strong dynamic suites can miss exploit paths, and SUSVIBES' authors explicitly note that tests and CWE annotations do not cover all exploit modalities. That means **any published number should be framed as 'residual insecurity under this executable test regime,' not as a claim of absolute security.**"

> "Some of the most vivid anti-cheating evidence, especially the April 2026 git-history exploitation rate reported by **Endor Labs**, comes from a **vendor extension rather than from a peer-reviewed academic paper**… it should be described as a reported finding rather than as settled universal fact."

> "private test sets increase credibility but reduce full public reproducibility."

### R3 NET

- SAFEDELTA's DESIGN is well-grounded: the metric (residual-insecure-at-review, paired, per-CWE, bootstrap CI, utility guardrail), the corpus (SUSVIBES/BaxBench/SeCodePLT/AgentDojo), and the git-stripping all trace directly to R3. G-6 is R3 operationalized — the plan matches the research.
- **Two refutation-grade constraints on the KEYSTONE number**: (1) **statistical power** — 200 tasks detects only ~9pp effects; a small gate effect on ELAI's stated corpus is un-publishable; need **300–400 git-stripped tasks**. (2) **dynamic-vs-static ground truth** — if F-2b's SeCodePLT runner scores by static ruleset rather than executing dynamic PoCs, the number is "weaker as benchmark truth" per R3's own hierarchy.
- Anti-gaming is non-negotiable: **81% git-history exploitation** (Endor Apr 2026) makes .git-stripping mandatory, and **>50% of SWE-bench repeated failures were harness flaws** (OpenAI Feb 2026) makes blinded dispute-audit mandatory.
- Strongest evidence: SUSVIBES **61% correct / 10.5% secure on Claude 4 Sonnet** — the empirical proof that "works ≠ secure" on the exact model ELAI wraps.
- Compounds R1: R1 says the benchmark doesn't close the sale; R3 says at 200 tasks it may not even produce a defensible number. Together they put the "believe-the-number" GTM under double pressure — the number is hard to make credible AND insufficient to sell on even when credible.

---

## 4. R4 — COMPETITIVE LANDSCAPE

Source: `workflow/research/grok-elai-trust/R4-competitive-landscape.md` (155 lines). Header "Research completed in 17m". Footnote digits 10–39 orphaned; all numbers [EXTERNAL, named]. This is the stream that decides whether the trust spine is a MOAT or a commodity — R1 said the positioning is validated but occupied; R4 tests occupation directly.

### THE QUESTION
Who else is selling trust/verification for AI-generated code in 2025–2026, what do they cover, and is there a defensible unowned wedge for an independent entrant?

### THE VERDICT — THE UNOWNED WEDGE EXISTS (and this is the strongest strategic finding in the whole set)

The market has split into three groups + assistant-native controls, and:
> "Across the surveyed public materials, the dominant promise is 'scan/fix/guard as the agent writes'; **I did not find a public claim from these vendors for deterministic, third-party replay-and-attest of the final agent output as a product category they own. That is the most defensible opening for an independent entrant.**"

The precise gap (verbatim):
> "It is NOT well-covered for **reproducible off-agent verification, signed provenance of a specific agent-produced diff, policy-grade attestation that the final artifact corresponds to what was reviewed, or vendor-neutral trust evidence that can survive changes in IDE, model, or assistant vendor.**"

**This is the single most valuable finding across all five streams for ELAI's direction question.** R1 validated the positioning but flagged Greptile/CodeRabbit as occupants; R4 shows those occupants are selling *review/scanning*, NOT *off-agent replay-and-attestation*. The specific territory ELAI's G-2 (RRA) + G-8 (attestation) + G-5 (cockpit) stakes out — "replayed in a clean environment, built and tested from scratch, matched to the artifact that was eventually shipped, wrapped in a durable evidence bundle a buyer/auditor/insurer/deployer can verify later" — is described by R4 as **"visibly unowned in the public product surfaces of the current market."** The moat is real as of the research window.

### THE VENDOR MAP (R4's table, condensed; all funding/traction [EXTERNAL, named])

Three groups + assistant-native:

**Group 1 — Incumbent AppSec extended into AI workflows:**
- **Snyk** — "AI Security Platform", "secure at inception", DeepCode AI, Snyk Studio. Funding: **$196.5M Series G Dec 2022 at $7.4B valuation** (latest clearly public); **2,000+ customers**; AI-native SAST product **passed $100M ARR**. Pricing: Free / Team **$25/dev/mo** / Ignite **$1,260/yr/dev** / Enterprise. Agent-trust: "**partially** — mostly in-agent/in-pipeline prevention and scanning, **not independent attestation of the final output.**"
- **Semgrep** — core SAST/SCA/secrets + **Semgrep Guardian** ("scans and fixes AI-generated code the moment it's written inside Claude Code, Cursor, Windsurf, Kiro"); AI rule packs (prompt injection, unrestricted tool use, data exfil, malicious agent skills, Shadow AI). Funding: **$100M Series D Feb 2025, total $204M**. Traction: AI-prioritization "validated by 95% of security reviewers across 6M+ findings"; Assistant with Memories "96% accuracy, handled 60% of alerts before engineers saw them." Pricing: Free ≤10 contributors / Teams **$30/mo/contributor** / Secrets **$15** / Enterprise. Agent-trust: "**Yes**… still, its center of gravity is analysis and remediation in the workflow, **not proof-bearing independent verification after the agent finishes.**"
- **Endor Labs** — "AI-native AppSec", Code with AI SAST, business-logic reviews, secrets, agentic remediation, **Agent Governance** for AI coding agents/MCP/skills. Funding: **$93M Series B April 2025 (oversubscribed)**. Customers: Dropbox, Atlassian, Rubrik, Netskope. **Launched the Agent Security League — open benchmark over 18 agent/model combinations** ("gives Endor benchmark credibility in AI-code-security" — strategically important, see below). Agent-trust: "**Yes** — one of the clearest… but even here, the public story is governance, scanning, dangerous-command detection — **not vendor-neutral attestation of final output.**"
- **Corgea** — "AI-native AppSec", BLAST scanner (business-logic/auth). Funding: **$2.6M seed Nov 2024 (YC)**. Pricing: Free / Growth **$39/dev/mo** / Scale **$49/dev/mo** / Enterprise. Agent-trust: "partially… still scan/triage/autofix. **I did not find evidence of independent, replayable, signed attestation of output.**"

**Group 2 — Domain specialists:**
- **Socket** — supply-chain: malicious package detection, reachability, firewalling, SBOMs; **Socket MCP** assesses AI-generated dependencies real-time; owns "slopsquatting and dependency hallucinations." Funding: **$40M Oct 2024, total $65M** (Reuters); **7,500+ orgs** then, homepage now claims **10,000+ attacks blocked weekly, 1.5M repos, 27,000+ orgs**. Pricing: Free / Team **$25/dev/mo** / Business **$50/dev/mo** / Enterprise. Agent-trust: "**narrowly** — strong on 'can I trust this import?' and **weak on 'can I trust this entire diff?'**" → **this directly maps to ELAI's F-9 (hallucinated-dep gate); Socket owns that slice. ELAI's F-9 is competing with a funded specialist, not entering open ground.**
- **GitGuardian** — secrets + non-human-identity. Funding: **$50M Series C Feb 2026**; "#1 app on GitHub Marketplace." **Detected ~29M leaked secrets on GitHub in 2025, AI-assisted code leaking at ~double the GitHub-wide baseline.** [EXTERNAL — this is a strong independent telemetry number confirming AI code worsens security.] Agent-trust: "narrow — secrets/credentials/NHI; **does not address broad semantic trust of agent-authored code.**" → **maps to ELAI's F-1 secret axis + G-9 redaction; GitGuardian owns secrets.**

**Group 3 — Mediation-layer experiments (the cautionary tale):**
- **CodeGate / Stacklok** — open-source security/workspace/multiplexing shell between agent and environment. **Main repo ARCHIVED June 5 2025, read-only.** ~789 stars, 92 forks at archive. Stacklok survived by pivoting to an enterprise MCP platform. "**effectively dead enough that a new entrant should treat it as a cautionary precedent rather than a live rival.**"

**Also-rans (R4 could not fully verify):** Aikido (broad AppSec, no distinct agent-trust position), Mobb ("$5.4M seed April 2023", autofix-the-backlog, "does not own the trust/attestation question").

**Assistant-native controls:**
- **GitHub Copilot** — content/code safety filters, duplication/public-code matching (block or reference w/ license), human-in-the-loop acceptance, content exclusion, cloud/local sandboxes. "still mainly policy, UX, execution-control hygiene — **not proof that the resulting code diff is independently trustworthy after generation.**"
- **Claude Code** — "stronger execution-boundary controls than most": read-only by default, explicit permission for edit/exec, sandboxed bash w/ fs+network isolation, working-dir write restriction, command allowlisting, Accept Edits mode, cloud isolated VMs + limited network + branch restrictions + credential scoping + audit logging, prompt-injection safeguards, dev-container recommendation. "a strong native control set for execution safety, **but it still does not amount to a portable, third-party attestation layer for agent-produced output.**"
- **Cursor** — least documentably clear; third-party tools (Snyk Studio, Semgrep Guardian) treat Cursor as a host to inject checks into — "the ecosystem sees Cursor as a place where external trust controls are needed."

### THE PATTERN (R4's central claim)

> "every active vendor that is winning mindshare today is selling one of four things — **embedded guardrails, high-signal scanning, specialized risk slices such as secrets or packages, or autofix/remediation. None of the public product pages I reviewed leads with 'we independently verify and cryptographically attest the trustworthiness of a finished agent-produced change set.'**"

The structural reason nobody owns it (verbatim, and this is the moat's foundation):
> "the assistant vendors control runtime permissions, and the AppSec vendors control scanning/assertions, but **neither side is naturally motivated to become the neutral notary for all agents and all code paths.** This conclusion is an inference from the surveyed products' public scopes."

### THE THREE FRAGILE POSITIONS (what NOT to be — direct guidance against parts of ELAI's own plan)

1. **"Standalone local/agent mediation shell"** — CodeGate died here. "the market did not choose 'standalone local/agent mediation shell' as its primary buying unit." Squeezed from both sides: assistant vendors absorb "secure the agent while it runs" via native permissioning; AppSec vendors absorb "catch bad code" via IDE hooks/MCP. **This is a warning shot at ELAI's F-1 VibeGate / F-3 Tool-Filter framing IF sold as a standalone mediation shell.** The mediation *mechanism* is fine; selling it AS the product is the CodeGate death.
2. **"AI SAST, but better"** — "already crowded" (Semgrep, Snyk, Endor, Corgea). "A small entrant trying to win by being 'the best AI code scanner' will run headlong into vendors with bigger sales teams." **Warning at ELAI's F-2 code-vuln ruleset IF positioned as the product rather than as evidence-feed.**
3. **"Autofix the backlog"** — Mobb, Corgea live there; incumbents adding remediation. "becoming a feature, not a defendable category."

### LIKELY INCUMBENT RESPONSE TO A SAFEDELTA BENCHMARK (this is the direct answer to "does the number defend?")

> "If a small company published a credible safety-delta benchmark tomorrow… the incumbents would almost certainly respond in four ways. Snyk, Semgrep, and Endor would likely **ship or highlight new IDE/MCP integrations, produce benchmark-styled marketing content, and claim that their existing… products already solve enough of the problem.** GitGuardian and Socket would likely **narrow the fight to their strongest slices.** GitHub and Anthropic would likely **emphasize sandboxing, human review, and policy controls rather than concede the need for an external verifier.**"

**Reading against the "believe-the-number" GTM**: R4 says a published SAFEDELTA benchmark does NOT force incumbents to concede — they respond with counter-marketing and "we already do this." Endor already has a benchmark (Agent Security League, 18 agent/model combos). So the number is contestable by a better-funded player with an existing benchmark. This is a THIRD pressure on the KEYSTONE number (after R1 "doesn't close" and R3 "underpowered at 200 tasks"): **even a credible number is a marketing asset the incumbents can neutralize, not a moat.** The moat is the neutral-notary POSITION + the off-agent-replay PRODUCT, which the number serves — not the number itself.

### THE DEFENSIBLE PRODUCT SHAPE (R4's recommendation — five primitives)

> "A defensible product shape would have five primitives. It would (1) **capture provenance about the generating agent/session**, (2) **perform off-agent clean-room replay of the proposed diff**, (3) **run a policy bundle over build/test/scan results**, (4) **generate a signed evidence object tied to the exact diff and artifact**, and (5) **expose a simple binary decision to downstream systems: merge, block, or require human override.**"

Map to ELAI plans: (1) G-1 taint/provenance + SSB, (2) **G-2 RRA replay** — but R2 says cloud-lane replay is trace-authentication only, so primitive (2) is *weaker for the cloud wedge*, (3) F-gates + G-6 policy, (4) G-8 attestation, (5) G-5 cockpit binary decision. **ELAI's G-track is a near-exact match to R4's five-primitive defensible shape.** The one soft spot is primitive (2): "off-agent clean-room replay" is the thing R2 says the cloud lane structurally cannot fully do. R4 does not notice this tension (it is a market analyst, not reading R2); this sweep flags it — see §12.

### THE POSITIONING RECOMMENDATION (verbatim, the sharpest line in R4)

> "Own the neutral attestation layer for agent-written code. Not 'AI SAST.' Not 'secure coding assistant.' Not 'autofix.' Not 'agent sandbox.' Instead: **'independent verification and signed trust evidence for AI-agent-produced software changes.'** That positioning is defensible because it is **orthogonal to assistant vendors, complementary to incumbent scanners, legible to enterprise buyers, and still visibly unowned.**"

Sell to "**security, platform, and compliance buyers as a verification and attestation system, not to individual developers as a smarter lint tool.**" — reinforces R1's ICP (security-led, not per-dev).

Benchmark strategy that follows: "prove 'our trust layer reduces exploitable and operationally dangerous defects per accepted agent diff'" — i.e. SAFEDELTA serves the PRODUCT, it is not the product. This resolves the apparent R1/R3-vs-commission tension: the number is real and worth building, but as evidence *for* the neutral-notary product, never as the thing sold.

### INDEPENDENT CONFIRMATION OF THE "WORKS ≠ TRUSTWORTHY" GAP

Two external telemetry sources R4 cites converge with R3's SUSVIBES number:
- **Endor's Agent Security League** benchmark "reports a large gap between functional correctness and security correctness in real agent/model combinations."
- **GitGuardian 2026**: AI-assisted development materially increasing secret leakage (~2x baseline, ~29M secrets 2025).
"agents can already produce useful code at high speed, but 'useful' and 'trustworthy' are still separate product categories." — three independent sources (SUSVIBES/R3, Endor, GitGuardian) now on the same gap. **The problem ELAI targets is real and independently measured by parties with no stake in ELAI.**

### R4's STATED LIMITS (verbatim)

> "high-confidence current evidence for Snyk, Socket, Endor Labs, Semgrep, GitGuardian, Corgea, CodeGate/Stacklok, GitHub Copilot, and Claude Code. By contrast, **Aikido's latest funding/traction, Mobb's current transparent pricing and momentum, and Cursor's latest official first-party output-security documentation were not fully verifiable** from high-confidence public sources in this pass, so I have treated those points conservatively rather than guess."

Multiple key conclusions explicitly flagged as inference: "This conclusion is an inference from the surveyed products' public scopes." R4 is disciplined about marking its own synthesis vs verified fact.

### R4 NET

- **THE MOAT IS REAL**: off-agent replay-and-attestation of a finished agent diff is "visibly unowned." ELAI's G-track (RRA + attestation + cockpit) is a near-exact match to R4's independently-derived five-primitive "defensible product shape."
- The neutral-notary position is defensible precisely because it is orthogonal to assistant vendors and complementary to (not competing with) AppSec incumbents — "neither side is naturally motivated to become the neutral notary."
- **But three of ELAI's F-track pieces are entering OCCUPIED ground, not open ground**: F-9 (hallucinated-dep) vs Socket; F-1-secret-axis + G-9 vs GitGuardian; F-2 (AI SAST) vs Semgrep/Snyk/Endor/Corgea. These are commodity axes; ELAI should treat them as evidence-feeds into the notary, never as standalone products (R4's three fragile positions).
- **CodeGate is the death ELAI must avoid**: a standalone agent-mediation shell (which F-1/F-3 resemble if mis-sold) is squeezed from both sides and was archived June 2025. Sell the notary, not the shell.
- Third pressure on the KEYSTONE number: even a credible SAFEDELTA is counter-marketable by incumbents (Endor already has a benchmark). The number serves the notary product; it is not itself the moat.
- Strongest evidence: the exhaustive vendor scan finding NO incumbent leading with off-agent replay-and-attest, cross-confirmed by three independent "works≠trustworthy" telemetry sources (SUSVIBES, Endor, GitGuardian).
- Sharpest refutation embedded here: **the moat is the POSITION and the PRODUCT, not the benchmark** — which quietly contradicts the commission's "everything gates on ONE SAFEDELTA number." R4's whole argument is that the defensible thing is being the neutral notary; the number is a marketing input to that, and betting everything on it mistakes the evidence for the moat.

---

## 5. R5 — HUMAN-AS-VERIFIER (LOAD-BEARING FOR THE WHOLE THESIS)

Source: `workflow/research/grok-elai-trust/R5-human-as-verifier.md` (89 lines). Header "Research completed in 15m". This is the stream the task flags as load-bearing, because the ENTIRE thesis rests on a claim about the human role: if the human is NOT durably a verifier — if models get good enough that verification stops being scarce — the trust spine sells nothing. R5 is where that claim is tested empirically. It is also the stream that most directly grounds G-5 (Reviewer Cockpit + DSR metric). Footnote digits 10–15 orphaned; all numbers [EXTERNAL, named, dated] unless flagged.

### THE QUESTION
As AI generates more code, is the human role durably shifting to VERIFIER — and is that role economically durable, or a transitional patch until models improve? What makes reviewing AI code fast and trustworthy, and what workflow delivers faster verification throughput?

### THE VERDICT — VERIFICATION IS THE BOTTLENECK, AND THE VERIFIER ROLE IS DURABLE (but not in line-by-line form)

> "as AI generates more code, **the scarce resource is no longer typing speed, it is trustworthy verification.**"

> "the human role shifts from primary author to **risk owner, verifier, and trust calibrator.** The winning system will not ask humans to read every line of every AI diff. It will make the machine do most of the generation, most of the routine checking, and most of the evidence collection — then route the human reviewer only to the small fraction of changes where judgment, accountability, and repository-specific context still matter."

**This is the empirical foundation of ELAI's entire direction, and R5 SUPPORTS it — but with a critical qualification that reshapes the product.** The durable role is "**trust auditor, risk governor, and exception handler, not classical reviewer-of-every-line.**" The product implication is precise and it is NOT "help humans read diffs faster" — it is "**remove most hunks from human attention entirely, while making the remaining hunks easier to trust or reject.**" This is exactly G-5's DSR metric (lines-needing-full-attention / total-changed-lines): the product's job is to SHRINK the numerator. R5 is the research spine under G-5.

### THE LOAD-BEARING NUMBER — METR (the same RCT R1 cited, here in full detail)

> "METR's July 2025 study is the load-bearing evidence here because it measured **end-to-end task completion by experienced developers on their own repositories**, rather than asking for self-reports. Across **246 real tasks performed by 16 experienced open-source developers, AI assistance produced a 19% slowdown, despite participants predicting a speedup.** METR's qualitative decomposition was especially important: developers **lost time to reviewing and correcting AI output, not just to generation itself.** That is the most direct evidence that verification can become the binding constraint."

[EXTERNAL, METR, July 2025 — the ONLY RCT in the entire set, appears in both R1 and R5.] Full parameters: 246 tasks, 16 experienced OSS devs, own repos, early-2025 frontier tools, 19% slowdown, expectation-reality inversion (they predicted speedup). The mechanism — time lost to *review and correction* — is the direct causal link to "verification is the bottleneck." **This single number does the heaviest lifting in the whole GROK-ELAI-TRUST set.** Handle its limits carefully (R5 does): early-2025 tools, small N=16, experienced OSS devs on mature repos. It does NOT establish that verification is the bottleneck for novices or greenfield — arguably the opposite population from R1's vibe-cleanup ICP. This is a real tension: METR's population (expert, mature repo) is not the vibe-coder population (novice, throwaway prototype). The bottleneck claim generalizes by argument, not by measurement, across that gap. Flagged for §12.

### PRACTITIONER TRUST EVIDENCE (the demand-side of verification)

- **Sonar, January 2026, survey of 1,149 developers**: **96% do not fully trust AI-generated code to be functionally correct**; **only 48% always check AI-assisted code before committing**; **38% say reviewing AI-generated code takes MORE effort than reviewing colleague code.** [EXTERNAL, Sonar]
- **Stack Overflow 2025 developer survey**: **more developers distrust AI-tool accuracy (46%) than trust it (33%)**, with experienced developers most skeptical. [EXTERNAL, SO]

The 38% "more effort to review AI code than human code" is the quiet killer stat: AI is supposed to reduce work, but a third of devs find the *review* of its output harder than reviewing a human's. That is verification debt made measurable.

### THE VOLUME DRIVER (why review becomes the system bottleneck)

- **GitHub Octoverse 2025**: ~**43.2M merged PRs/month**, ~**1 billion commits in 2025**, **693,867 new public repos using an LLM SDK** in the previous 12 months (**+178% YoY**). [EXTERNAL, GitHub]
- **Anthropic June 2026 internal report**: by May 2026, **>80% of code merged into Anthropic's codebase was authored by Claude**; typical engineer in Q2 2026 merging **8× as much code per day as in 2024.** [EXTERNAL, Anthropic internal — may not generalize, R5 flags this]

"That is exactly the condition under which review becomes the system bottleneck."

### THE DELEGATION CEILING (crucial nuance — heavy usage ≠ full delegation)

> "Anthropic's December 2025 internal study found that employees reported using Claude in about **60% of their work, but most said they could fully delegate only 0–20% of their work** to it; for higher-stakes work, active supervision and validation remained the norm."
> "Claude can increasingly **execute the method, while humans still supply the goal and the higher-level judgment.**"

**This is the answer to the deepest threat to the thesis** (the threat being: "models get good enough that verification stops mattering"). Even in "one of the most AI-saturated engineering environments in the world," with 80% of code Claude-authored, the fully-delegable fraction is 0–20% and supervision remains the norm for high-stakes work. The verifier role is not being automated away by better models — it is moving UP the stack (goal/risk/judgment), not disappearing. **R5's evidence says the trust-auditor role is durable specifically because delegation plateaus at 0–20% even at the frontier.** This is the strongest single piece of support for ELAI's core bet, and it comes from Anthropic's own internal data.

### THE REVIEWER-COCKPIT UX EVIDENCE (this grounds G-5 directly)

**JetBrains-backed participatory design study (2026)** on reviewing LLM-generated multi-file changes — the strongest UX evidence:
- Reviewers converged on a **three-level workflow**: overview layer / file-analysis layer / code-snippet layer.
- Seven design constructs: **chunk, risk-per-line, risk-per-file, Judge, walk-through, zooming in/out, security cage.**
- Validation survey: **63% expected the prototype to reduce overall review effort; 52% expected it to reduce the effort of assessing trustworthiness specifically.**
- The right abstraction: "not 'read the diff faster,' but **'surface uncertainty at the right granularity.'**" Change-set granularity → walk-through + architectural orientation. File granularity → risk-per-file + Judge verdict. Chunk granularity → risk-per-line + localized rationale.

**This IS G-5.** ELAI's Reviewer Cockpit three-level design and DSR metric are the JetBrains three-level workflow operationalized. G-5 is well-grounded in R5. The one thing R5 adds that G-5's description (00-PROMPT line 24) does not name: the "**Judge**" traffic-light verdict at file granularity and the "**security cage**" construct — check §10 whether G-5 implements a per-file Judge verdict.

**Meta 2026 FSE Companion paper on high-risk diffs** (the operational proof-point):
- surfaced a **Diff Risk Score** in the review UI, highlighted risky snippets, **auto-assigned an extra reviewer** for the riskiest diffs, **soft-blocking pre-land dialog** when authors tried to land without resolving the risk signal.
- On the **top 5% riskiest diffs**: increased the fraction whose final version fell below the high-risk threshold by **9.56%, with only a 0.45% increase in reviewing time.** [EXTERNAL, Meta — the cleanest cost/benefit number for risk-routing: near-zero added review time, ~10% fewer high-risk diffs shipped.]
- improved risky-diff reviewer recommender: **+9.94% comments, +7.77% acceptances**, no significant guardrail slowdown.

**GitHub 2025**: AI self-review before opening a PR "often removed an entire class of trivial issues… cutting back-and-forth by roughly **a third**." Spec-driven-development guidance: agent works from a spec → plan → "small, reviewable chunks that can be implemented and tested in isolation."

**R5's synthesis of all of this into a named pattern (= ELAI's DSR):**
> "Diff-Surface-to-Review: shrink the human review surface by forcing the agent to emit small, isolated, explainable, testable chunks; attach machine-generated risk signals and verification receipts to each chunk; and reserve raw code reading for the hunks that remain uncertain after automated checks. That formulation is a synthesis rather than a formal standard."

The name "Diff-Surface-to-Review" is R5's own coinage, and ELAI's G-5 metric is literally named "Diff-Surface-to-Review (DSR)". **So G-5's central metric is taken directly from R5's synthesized pattern.** This is the tightest research-to-plan mapping in the entire set — the plan didn't just get grounded by the research, it adopted the research's coined term as its metric name.

### THE SIGNAL-TO-NOISE EVIDENCE (why unfiltered AI review is HARMFUL — grounds G-5's "no LLM-opinion noise" rule)

**2026 MSR paper on code-review agents** (the most direct "low-signal review" result):
- 3,109 PRs from AIDev dataset. **CRA-only reviewed PRs had a 45.20% merge rate, vs 68.37% for human-only** — and higher closure/abandonment.
- Signal-to-noise of 98 closed CRA-only PRs: **60.2% fell into the 0–30% signal range; 12 of 13 CRAs had average signal ratios below 60%.**
- "in the wild, **unfiltered AI review frequently creates more maintainer burden than useful action.**"

**"Human-AI Synergy in Agentic Code Review" (2026)**, 278,790 inline review conversations from 300 mature OSS projects:
- AI generated more suggestions than humans (**88,011 vs 25,673**) yet **human suggestions adopted far more often (56.5% vs 16.6%).**
- AI comments much longer, disproportionately code-improvement/defect-detection; humans more varied/contextual (understanding, knowledge transfer).
- **When adopted, AI suggestions produced LARGER increases in code complexity and code size than human suggestions.** (AI review actively bloats code)
- Interaction findings: humans reviewing AI code **exchange 11.8% more rounds** than reviewing human code. When AI initiates review, **85–87% of conversations end after the first AI comment with no follow-up.** **Conversations ending with an AI response are rejected more often than those ending with a human response.**
- **Authors explicitly recommend ensuring a human provides the final response before merge.** → "a direct empirical argument for 'human final sign-off' rather than AI-only review loops."

**Counter-nuance (R5 is fair — not everything says AI review is garbage):**
- **2026 Code Review Agent Benchmark (c-CRAB)**: current automated review tools captured only **20.1%–32.1%** of issues from human review feedback; union of four tools covered only **41.5%**. BUT when authors manually labeled generated comments, **84% were judged useful.** Interpretation: "review agents often **catch different issues from humans** rather than simply producing nothing of value."

**R5's practical conclusion (grounds G-5's central design rule):**
> "stop asking one generalist AI reviewer to comment on everything. The MSR paper explicitly recommends using CRAs for **narrow, specific checks, especially security or well-defined categories**, rather than broad autonomous review. The c-CRAB authors similarly argue that **executable tests are a better optimization target than textual similarity** to human comments. In practice… style, formatting, and other deterministic concerns should be handled by formatters, linters, and static rules; the AI reviewer should only be allowed to surface findings in categories like correctness, security, testing gaps, compatibility breaks, and error handling, and **each finding should be supported by evidence rather than pure prose.**"

**This is EXACTLY G-5's rule** (00-PROMPT line 24): "surface a finding ONLY when backed by a deterministic gate verdict or a failing test (no LLM-opinion noise)." G-5's anti-noise rule is directly grounded in the MSR + Synergy + c-CRAB findings. The research says LLM-opinion review is net-harmful in the wild; G-5 bans it. Tight grounding.

### IS THE ROLE DURABLE? (the thesis-critical question — R5's answer is YES-with-transition)

> "the role is **economically durable in the medium term — but not in its current line-by-line form.**"

Evidence for durability:
- **Sonar**: most important developer skill in the AI era = **"reviewing and validating AI-generated code for quality and security" at 47%, ahead of prompting at 42%.** Enterprise teams becoming more rigorous about compliance reviews than smaller firms. "evidence of a paid, explicit verification function — not merely a temporary patch while models improve."
- **Linux kernel 2026 guidance on AI coding assistants**: "**only humans can legally certify the Developer Certificate of Origin**, and the human submitter is responsible for reviewing all AI-generated code, ensuring licensing compliance, and taking full responsibility." Adds an **Assisted-by attribution tag.** [EXTERNAL, Linux — a mature project codifying the human reviewer as the legal/operational accountability point.]
- **Anthropic June 2026 / Dec 2025**: bottleneck shifts UP into goal selection/judgment/validation; fully-delegable fraction 0–20%.
- **GitHub 2025/2026**: "the person who hits Merge remains the accountable owner."

**The transitional mechanics (why it's NOT line-by-line forever):**
> "The durable part is not 'human reads every line forever.' The durable part is **human ownership of intent, risk, and exceptions.** As verification stacks improve, more of the current manual review work will be absorbed by narrow AI reviewers, stronger CI gates, richer test generation, risk scoring, and better provenance."

**The patch-validation refutation (why weak automation is NOT enough — grounds the whole off-agent-evidence doctrine):**
> "in automated vulnerability repair, **more than 40% of patches that looked correct under basic tests failed stronger PoC+ validation**, and OpenSSF reports a manual review of **630 AI-generated patches in which 20–40% were semantically incorrect despite passing automated validation.**" [EXTERNAL — PVBench + OpenSSF OSS-CRS]

This is a double-edged finding and it is important to read both edges:
- Edge 1 (supports ELAI): weak "basic tests" over-estimate correctness; you NEED stronger, more independent verification (off-agent PoC+ execution) — directly supports G-7's "oracle-free generated-test execution" and R3's "dynamic exploitability as ground truth."
- Edge 2 (a warning to ELAI): **20–40% of patches were semantically incorrect DESPITE passing automated validation.** No automated gate catches everything. The residual — the semantically-wrong-but-passing patch — is precisely what must reach the human. This bounds what ANY trust gate (including ELAI's) can promise: the gate reduces the human's load, it cannot eliminate the human, because a fifth to two-fifths of dangerous patches survive automation. **This is the honest ceiling on the "believe-the-number" claim** — even a perfect ELAI gate leaves a 20–40% semantic-error residual that only a human catches. ELAI's own G-6 framing ("residual-insecure-rate REACHING a human") is correct precisely because it concedes this: the number measures what the gate removes, and something always reaches the human. Sold honestly, this is a strength; sold as "trust the number and stop reviewing," it is a lie the research directly refutes.

> "the economically durable role is best described as **trust auditor, risk governor, and exception handler, not classical reviewer-of-every-line.** Organizations that keep paying for that function are not being conservative; they are responding to the real liability structure of software delivery in the AI era. Organizations that fail to staff it are very likely accumulating what Sonar called a verification bottleneck and what many practitioners now describe as **verification debt.**"

### THE WORKFLOW THAT DELIVERS FASTER VERIFICATION (R5's six-part design → maps onto ELAI's whole stack)

The throughput target and its honest caveat:
> "A realistic path to **2–5× faster effective verification throughput** does not come from teaching humans to read AI diffs faster. It comes from **removing most hunks from human attention entirely**… That 2–5× figure should be treated as a **design target inferred from multiple studies, not a single experimentally proven end-to-end result.** The evidence base supports the ingredients, but not yet a full-stack production RCT of that magnitude."

The six ingredients (each grounded, each maps to an ELAI piece):
1. **Shape the work before code exists** — every agent run produces spec + plan + task breakdown; prohibit monolithic PRs. Complexity features (patch size, files touched) dominate review burden; route structurally-large PRs to a high-friction path before a human reads. → maps to ELAI's planning-workflow / decomposition-tree discipline.
2. **Make proof tokens first-class artifacts** — per hunk/chunk, a compact **verification receipt from an off-agent runner**: which tests touched it, which security checks ran, whether a policy gate passed, whether a stronger semantic/regression test exists. "**never let the generating agent be the sole judge of its own correctness.**" (c-CRAB: executable tests useful; PVBench: weak basic tests over-estimate; OpenSSF: passes-automation-yet-wrong) → maps to F-4 + G-2 evidence manifest + the off-agent doctrine.
3. **Build a reviewer cockpit, not a comment stream** — three-level trust-calibrated flow (overview / risk-ranked-files-with-Judge-traffic-light / exact-hunks-needing-attention) + Meta's risk-in-UI + route-top-risk-to-stronger-reviewers. → G-5.
4. **Rank human attention structurally, not semantically** — [EXTERNAL] the 2026 circuit-breaker paper: "at a **20% review budget, a structural model based on size, files touched, and related complexity signals captured 69% of total review effort.**" Routing rule: deep attention on high-complexity tail, fast-lane low-risk well-proved chunks. "the fastest way to multiply reviewer throughput without lowering standards." → G-5's DSR is a structural (deterministic-from-SSB) metric, matching "structural not semantic."
5. **Suppress style noise aggressively** — style/formatting inadmissible as LLM review comments when deterministic tooling is clean; reserve LLM comments for correctness/security/testing-gaps/compatibility/repo-conventions; **require evidence for every AI comment** (failing test, static-analysis trace, policy hit, reproducer, citation to a repo rule). → G-5's "no LLM-opinion noise."
6. **Keep the human at the last irreversible step** — human final response for code touching **regulated domains, auth, billing, privacy, data deletion, migrations, public APIs, CI configuration, or anything yellow/red after automated proving.** Low-risk green chunks flow with skim review. → maps to G-7 conformal-refusal + the lethal-trifecta quarantine.

### R5's STATED LIMITS (verbatim — unusually honest, do not sanitize)

> "METR's 2025 slowdown study is the best real-task experiment, yet it used **early-2025 tools and a small sample of experienced OSS developers.** The JetBrains trust-calibration paper is a strong design study, but it **validates a prototype with surveys rather than hands-on longitudinal deployment.** Several important workflow signals also come from **internal company data at Anthropic and GitHub, which may not generalize cleanly** to every enterprise environment."

> "There is also **no published 2025–2026 field trial, at least in the sources reviewed here, that cleanly demonstrates end-to-end 2–5× faster human verification** for autonomous code output in production. The closest evidence supports the pieces… The next decisive evidence gap is a real production study that measures reviewer time, trust calibration, escaped defects, and merge quality under an integrated reviewer-cockpit workflow."

**This is the single most important limitation in the whole set for ELAI's product bet.** The 2–5× verification-throughput improvement — which is the value proposition of G-5 — has NEVER been demonstrated end-to-end. The ingredients are each evidenced; the integrated multiplier is a design target. **ELAI's cockpit could be the first system to produce that end-to-end number** (which R5 explicitly names as "the next decisive evidence gap"). That is simultaneously the biggest opportunity (be first to prove the multiplier) and the biggest risk (the multiplier may not materialize when the pieces are integrated). This is a second measurable number ELAI could own, distinct from SAFEDELTA — and R5 says the field has no such number yet.

### R5 NET

- The verifier role IS durable — grounded in Anthropic's own 0–20% delegation ceiling even at 80%-Claude-authored, Linux's DCO human-certification requirement, Sonar's 47% "review skill is #1", and the merge-button-is-accountability norm. The deepest threat to the thesis (models make verification obsolete) is refuted by frontier-lab internal data.
- The durable role is trust-auditor/risk-governor/exception-handler, NOT line-by-line reviewer. The product must SHRINK the human review surface (G-5's DSR numerator), not speed up line-reading.
- G-5 is the best-grounded piece in ELAI's plan: its name (Diff-Surface-to-Review), its three-level structure, its structural-not-semantic ranking, and its no-LLM-opinion-noise rule ALL trace directly to R5's synthesized evidence (JetBrains, Meta, MSR, Synergy, c-CRAB, circuit-breaker).
- Strongest evidence: METR July 2025 RCT (**19% slower, verification is the binding constraint**) + Anthropic Dec 2025 (**0–20% fully-delegable even at the frontier**) — together they establish both that verification is the bottleneck AND that it stays human.
- Sharpest refutation / hardest ceiling: **20–40% of patches are semantically wrong DESPITE passing automated validation** (PVBench/OpenSSF). No gate eliminates the human; a fifth-to-two-fifths of dangerous patches survive automation. This bounds every trust-gate promise and directly refutes any "trust the number, stop reviewing" reading of the GTM. G-6's "residual reaching a human" framing is correct because it concedes this.
- The unclaimed second number: the **2–5× verification-throughput multiplier has NEVER been shown end-to-end** — R5 names it "the next decisive evidence gap." ELAI's cockpit is positioned to be the first to produce it; it is also the risk that it won't materialize on integration.

---

## 6. ELAI-CRITIQUE-LENSES — THE REUSABLE INSTRUMENT

Source: `workflow/research/grok-elai-trust/ELAI-CRITIQUE-LENSES.md` (95 lines). **This is NOT a research stream — it is an ELAI-authored artifact** (authored 2026-06-08, predating the Grok commission). It belongs in the trust-spine set because it operationalizes R5's "no-LLM-opinion-noise" discipline into a design-time critique tool AND because it defines a second operating profile (critique-only) that IS the human-as-verifier engagement R5 says is durable. Capture it as a reusable instrument, not a list.

### PROVENANCE + THE IMPORT-GATE STORY (important for the skill-import-policy angle)

Origin: "adapted from the 'Automated Doubt' workflow (alexself.dev/blog/automated-doubt) into ELAI's doctrine. These are **RE-AUTHORED prompts, not copies** of github.com/aself101/agents-and-pipelines (importing that repo would trip .claude/rules/skill-import-policy.md E-43 mcp-scan whole-directory gate first)."

**This is a deliberate, documented instance of the skill-import-policy (E-43) shaping engineering behavior**: rather than import an external agent repo (which would require the skillspector/mcp-scan whole-directory scan + a workflow/skill-imports.log ALLOWED row), the useful IDEAS were re-authored natively so no external skill enters .claude/skills/. The caveat section repeats it: "do NOT import github.com/aself101/agents-and-pipelines directly — skill-import-policy.md (E-43) requires mcp-scan whole-directory scan + a workflow/skill-imports.log row with action=ALLOWED first." **Finding: the import-policy rule is not theatre; it changed a real design decision (re-author vs import).** [MEASURED-BY-US — self-documented in the artifact.]

### THE DOCTRINE (5 rules — this is the reusable part, and it is where the instrument earns its trust-spine place)

1. **ADVISORY, NEVER LOAD-BEARING.** "LLM-judged critique is noisy and bypassable" — cites **MSR-2026: 60.2% of LLM-review PRs sit in the 0-30% signal band** (from R5) and **NAACL Mar-2025: all 8 inline LLM defenses bypassed >50% ASR** (from R2). The lenses "SURFACE issues… They NEVER emit an accept/block verdict. ELAI's off-agent deterministic gates (F-4 stdout_hash, coverage, SSB rows, mutable=0 rules, SAFEDELTA) remain the only thing that decides DONE." **This is the doctrine's spine: it uses the research's OWN refutations of LLM-judging (R2 NAACL, R5 MSR) to justify keeping LLM critique off the load-bearing path.** The instrument is designed to be untrustworthy-by-construction so it can never be gamed into a verdict. This is the same wiring-enforcement R4 logic (measure the real objective, not the LLM's opinion) applied to critique.
2. **WHERE THEY FIRE — DESIGN/SPEC TIME ONLY.** PLANNING, idea-decomposition (B-40 idea-critic), planning-workflow plan-improvement (the 9-step adversarial injection). "the front of the pipe, where the artifact is still prose/spec and a deterministic gate has nothing to hash. They do NOT run on shipped code (that is the auditor + the off-agent spine)." — clean separation: LLM critique where there is no artifact to hash; deterministic gates where there is.
3. **GROUNDING REQUIRED.** "Every finding MUST cite a concrete location: a spec section heading, a file:line, a named symbol, or a quoted phrase… A finding with no location is DROPPED." Explicitly mirrors R5's admissibility rule ("an AI comment with no failing test / trace / reproducer is inadmissible; here, no concrete anchor = inadmissible"). **The anti-noise discipline from R5 is applied to the critique tool itself.**
4. **PARALLAX, BOUNDED.** Run relevant lenses in parallel, cap output: max 8 findings/lens/pass, deduped by (location, claim). "scale lens count to artifact risk (SIMPLE spec: 2-3 lenses; HEAVY: full set)." Token-cost honesty: "the source workflow admits 40+ agents is 'not cheap'."
5. **CONVERGENCE, NOT PERFECTION.** Iterate until findings drop to NITs or operator accepts residual. "The operator is the judge."

### THE FINDING SCHEMA (every lens emits this — the reusable contract)
`{ lens, severity (BLOCKER-CANDIDATE|CONCERN|NIT — BLOCKER-CANDIDATE routes to human, NEVER auto-blocks), location (REQUIRED anchor, no location→dropped), claim (one sentence), why_it_matters, suggested_resolution, confidence (HIGH anchored-in-quoted-contradiction | MED | LOW) }`

### THE EIGHT LENSES (as an instrument — each fires on a condition, inspects a defect class)

| Lens | Fires when | Inspects | ELAI integration |
|---|---|---|---|
| **L1 Assumption Excavator** [universal] | spec references external interface/schema/API/other module | gap between assumed vs actual interface shape/return/ordering/nullability/units | **human-readable front end of the grounding-sweep**; its VERIFY/INGEST findings feed the sweep (classify GROUNDED/VERIFY/INGEST) |
| **L2 Implied Completeness Detector** | spec enumerates happy path / partial list | "etc.", "handle errors appropriately", "standard validation", open-ended enums, success-path-only methods | — |
| **L3 Ambiguity Mapper** | any NL requirement | requirements two competent engineers implement differently | composes with the shipped **AmbiguityScorer (B-38)**: lens = human-readable map, scorer = gate-time number |
| **L4 Gap Analyzer** | spec defining operation/state-machine/data-flow | unhappy/edge paths: malformed input, perm denied, partial write, concurrency, timeout, retry/idempotency | — |
| **L5 Pre-Implementation Architect** | HEAVY / multi-module | misplaced boundaries, hidden coupling, leaky abstraction, **producer-without-consumer (wiring PRE_GATE)** | "enforce the wiring-enforcement PRE_GATE here — flag any plan unit whose deliverable is a producer without its in-plan consumer + smoke trigger" |
| **L6 Public-Interface & API-Contract Validator** | adds/changes pub symbol/IPC/schema/cross-lang boundary | contract stability + cross-side symmetry | "design-time mirror of **cross_lang_diff.py + action_type golden lock (E-45)**; catches drift on the SPEC before code exists" |
| **L7 Documentation Validator** | specs w/ docs/comments | doc↔design drift; contradictory docs = BLOCKER-CANDIDATE, missing = CONCERN | — |
| **L8 Anxiety Reader** [last, catch-all] | before declaring ready | residual unease the structured lenses missed: irreversible-no-undo, silent failure, works-in-demo-breaks-under-load | "If you have no anchored worry, say so explicitly rather than inventing one" |

The lens set is a coherent **design-phase critique instrument**: L1/L6 feed grounding + cross-lang symmetry (the deterministic gates), L3 feeds the AmbiguityScorer, L5 enforces the wiring PRE_GATE at spec time, L8 is the human on-call gut-check. It thickens the front of the pipe where ELAI is admittedly weak.

### CRITIQUE-ONLY MODE (the second, strategically-important deliverable)

A new operating profile alongside the autonomous `orchestrate` default:
> "PROFILE: critique-only — WRITES performed by the OPERATOR… **NEVER by spawned executor agents.** Rationale carried verbatim from the source: agent-spawned writes 'caused more harm than good'… AGENTS spawned ONLY for critique (L1-L8 at design time; the existing auditor + off-agent gates at ship time). No executor/fixer agents touch disk. LOOP: Design (run lenses → operator revises spec) → Operator implements → Ship (auditor + off-agent spine + L6/L8) → iterate to convergence. WHEN TO PICK IT: **high-stakes / regulated / low-trust-in-autonomy work; the human-as-verifier engagement R5 says is durable.** orchestrate stays the default for autonomous throughput. INVARIANT: both profiles share ONE engine and ONE off-agent spine. The only difference is WHO writes and whether executor agents are spawned. **The trust guarantee (off-agent deterministic evidence) is identical in both.**"

**This is the product-shape answer to R5.** R5 says the durable human role is trust-auditor/exception-handler, not author. Critique-only mode makes the human the author and ELAI the critic+verifier — the exact inversion R5's regulated/high-stakes population wants. It is a direct, coherent response to the load-bearing stream: rather than fight R5's "human stays in the loop," it productizes the loop as a selectable mode. **This is arguably the most strategically-aligned single idea in the set** — it aligns the product with the durable role instead of betting against it.

### THE FINAL SELF-POSITIONING (verbatim — the sharpest line about what the whole trust spine IS)

> "WHAT THIS IS NOT: not a replacement for the trust spine, not a new architecture. It thickens ELAI's comparatively-thin DESIGN-phase critique (ELAI's strength is post-implementation/wiring) and adds a conservative operating mode. **The differentiator stays: ELAI makes doubt UN-FAKEABLE (off-agent evidence) where this layer only makes it PLURAL (more perspectives).**"

"ELAI makes doubt UN-FAKEABLE where this layer only makes it PLURAL" is the cleanest one-sentence statement of the trust-spine thesis in the entire set. It also correctly subordinates the lenses: plurality of LLM opinion is cheap and bypassable (the whole doctrine section proves it); the moat is the un-fakeable off-agent evidence. The lenses are a value-add, never the differentiator.

### CRITIQUE-LENSES NET

- A reusable design-time critique instrument (8 anchored lenses + a strict finding schema) that is deliberately built to be ADVISORY-only — using the research's own refutations of LLM-judging (R2 NAACL 8/8-bypassed, R5 MSR 60% low-signal) to justify keeping itself off the load-bearing path.
- Documented proof that skill-import-policy (E-43) changed a real decision: re-author the ideas natively rather than import an external agent repo.
- The **critique-only operating mode** is the product-shape that most directly answers R5: it makes the human the author and ELAI the verifier — productizing the durable human-as-verifier role instead of betting against it. Both modes share one off-agent spine; the trust guarantee is identical.
- The instrument's self-positioning ("makes doubt PLURAL, not UN-FAKEABLE") correctly keeps the differentiator on the off-agent evidence, not on LLM critique. This is the discipline the whole set argues for, applied to ELAI's own tooling.

---

## 7. ELAI-IMPROVEMENT-PLAN

Source: `workflow/research/grok-elai-trust/ELAI-IMPROVEMENT-PLAN.md` (99 lines). Authored 2026-06-08.

### PROVENANCE — resolves the §0 open question

Line 2 (verbatim): "Inputs: 5 GPT deep-research reports (R1 demand, R2 technical spine, R3 measurement, R4 competitive, R5 human-verifier), **one Grok-Heavy adversarial pass, one Opus cross-synthesis**, and the existing F/G/H plan tracks. This is a strategy/sequencing plan, not an implementation spec. No code was changed authoring it." Line 4: "**Grok output is pasted in the session transcript.**"

**Resolved: this file is an ELAI-authored SYNTHESIS of three inputs (the 5 GPT streams + a Grok-Heavy pass + an Opus cross-synthesis), NOT the raw Grok output.** The raw Grok output lives only "in the session transcript" — it is NOT in this directory. So the improvement plan is the fused verdict, and where it says "Grok said X, Opus said Y, this reconciles them," it is doing genuine synthesis (e.g. §6 on G-1 taint). Note: R1–R5 are labelled here as "GPT deep-research reports," which means the five streams were run through GPT deep-research (consistent with the "Research completed in Nm" headers), NOT authored by Grok. Grok's role was the single adversarial Team-of-Experts pass over all of them. This matters for weighting: the five streams are GPT deep-research (web-grounded, cited), the prioritization is Grok+Opus synthesis (reasoning, not newly-grounded).

### THE ONE-SENTENCE VERDICT (verbatim — the thesis of the whole set)

> "ELAI already designed the architecture the entire research corpus says wins (off-agent artifact, missing-evidence=FAIL, frozen gate the run cannot edit) — the job is not to invent, it is to **externalize that doctrine into ONE portable signed evidence bundle, prove it with ONE adversarially-hardened security-delta number, and reposition the product around the verifier rather than the agent.**"

Three verbs: externalize (bundle), prove (number), reposition (verifier-not-agent). Every recommendation sequences these three.

### THE HEADLINE STRATEGIC SHIFT (§1 — "new, not in F/G/H today")

**1.1 AGENT-AGNOSTIC VERIFIER.** "The largest defensible market (R4: empirically unowned) is verifying ANY agent's diff — Cursor, Claude Code, Copilot, SWE-Agent — not only ELAI's own. ELAI's own agent becomes one input among many." Grounds: R4 (neutral notary) + R1 (writer≠certifier). Crucially: "**This also defuses ELAI's two root risks at once: the Anthropic-dependency tension (the verifier doesn't care which model wrote the diff) and the METR-19%-slowdown weakness (R5/R1 — the agent is the weakest part of the story; do not bind the value claim to ELAI's own generation).**"
- ACTION: "define the verifier's input as a **(base_commit, proposed_diff, optional session_provenance) tuple, model-agnostic.** ELAI's agent emits that tuple; so can a wrapper around any third-party agent."
- **This is the single highest-leverage idea in the set.** It flips ELAI from "an autonomous coding agent with trust features" (which competes with Cursor/Copilot AND inherits the METR-slowdown weakness AND the Anthropic dependency) to "the verifier of anyone's diff" (which is R4's unowned neutral-notary position AND is model-agnostic AND doesn't rise or fall with ELAI's own generation quality). It uses R4's moat to simultaneously dissolve the two root risks named in the commission. If ELAI takes one thing from this set, it is this.

**1.2 THE PRODUCT IS THE BUNDLE, NOT THE GATE.** "Buyers do not buy 'trust'; they buy a replayable, signed evidence pack tied to an exact diff (R1 closing artifact; R4 five-primitive product shape)… ELAI's F-gates are the policy bundle; G-2 RRA is the replay; G-8 is the signature. **The bundle is the SKU.**" — directly adopts R1's "emit a trust bundle not a pass/fail" and R4's five primitives.

### PHASE 0 — FALSIFY BEFORE YOU BUILD (§2 — three mandatory kill-gates; this is the plan's best instinct)

"Grounded-by-default forbids pouring effort into the heavy artifact while the core number is unproven. Run these THREE cheap tests first; any failure re-scopes or kills the thesis."

- **F0.1 SMALL SAFEDELTA SIGNAL** — reduced pilot, **50-100 git-stripped exploit-executed tasks**, gate-ON vs OFF, dynamic exploit ground truth. Look only for sign + rough magnitude. **KILL CRITERION: "if the delta is flat or negative, or the gate 'wins' only by blocking nearly everything… STOP — the believe-the-number thesis is dead before the expensive 300-task build."** Grounds R3 + SUSVIBES (~50pt gap to close).
- **F0.2 REPRODUCIBILITY-CLAIM SCOPE** — 20 identical tasks local Qwen vs cloud Claude, measure action-stream divergence on security paths. Decide claim wording. **KILL CRITERION: "if even the ACTION STREAM diverges >5% on security paths under fixed seed, the 'reproducible' word is unusable on the cloud path — narrow the claim to signed-evidence-only."**
- **F0.3 CHANNEL WILLINGNESS-TO-PAY** — "3-5 discovery calls with vibe-cleanup agencies… 'Would you pay **$40+/seat/mo** for a portable signed evidence pack you can hand your client as contractual proof?'" **KILL CRITERION: ">=2 of 5 say no -> the agency channel wedge (R1) is dead; fall back to direct security/platform/compliance buyers (R4)."**

**This is the plan's strongest structural move: it front-loads falsification of the three biggest risks BEFORE the keystone build, sized to days-to-weeks.** It is a direct application of grounded-by-default at strategy grain. Note the $40+/seat figure here is the origin of the memory's "$39-49/seat" — it is a *WTP-test hypothesis*, NOT a validated price. The kill-gates have owner-facing implications for the sweep: §11 shows NONE of the three Phase-0 kill-gates has been run as of 2026-07-17. The plan says "only on three greens proceed to Phase 1" — and Phase 1 pieces have been built anyway (F-4, G-2, G-8 shipped per memory) WITHOUT the Phase-0 go/no-go. **This is the most important drift in the set: the plan mandates falsify-first, and execution built-first.** Carried to §10/§12.

### PHASE 1 — THE ONE-BUNDLE PRODUCT (§3 — "ship as ONE cohesive deliverable; worthless apart; the SKU")

- **3.1 OFF-AGENT TEST-EVIDENCE CORE [F-4 + coverage]** — wrapper captures raw stdout/stderr+exit+argv+cwd, hashes, PASS only on wrapper-observed success; in-toto Test-Result+Link carrying stdout_sha256/stderr_sha256/exit_code/**coverage_profraw_sha256/wrapper-digest**. Adopts R2's sharpened anti-fabrication (coverage + wrapper-digest, not stdout alone) AND R5's "oracle must be stronger than basic tests" (>40% PoC+ failures, 20-40% semantically wrong). Effort: M+S.
- **3.2 REPRODUCIBLE-RUN ARTIFACT [G-2]** — the msgpack+zstd bundle; scope per F0.2 (cloud=verify-divergence, open-weight=full recompute). Effort: M.
- **3.3 SIGNED VERDICT THE AGENT NEVER AUTHORS [G-8 + a new VSA]** — "Emit a signed VSA / verification-report/v1 as the ONLY final 'accepted' verdict, authored by the verifier plane, never the agent." DSSE v1.0 (threshold-sign source of truth) + Sigstore bundle + Rekor v2 (discover via TUF SigningConfig, don't hardcode) + archived trust-root snapshot. **"CAVEAT (R2): sigstore-rs is experimental and cannot verify attestations — shell out to `cosign verify-blob --bundle` from the Rust harness; do not assume a pure-Rust verify path."** — directly absorbs R2's Rust-tooling blocker AND R2's VSA primitive (both of which I flagged as gaps against 00-PROMPT; the improvement plan already caught them). Effort: M.
- **3.4 HARNESS GIT-HARDENING [new, existential]** — "Before publishing ANY number: run the benchmark on git-free tarball snapshots… AND audit ELAI's OWN agent for git-history exploitation. Grounds: R3 / Endor Apr-2026 — SWE-Agent+Claude Opus 4.6 exploited git history in 163/200 tasks (81%) despite instructions; **ELAI's agent will cheat its own benchmark, and Endor runs the Agent Security League and will replicate-and-discredit an un-hardened number.**" **This is the most important NEW deliverable the improvement plan adds — it is not in F/G/H, and it is existential**: an un-git-hardened SAFEDELTA is not just weak, it is a liability a competitor (Endor) can actively discredit. Effort: M.
- **3.5 TABLE-STAKES (parallel, non-negotiable)** — self-hostable verifier + zero-retention-after-verification + SOC 2 path + the frozen read-only gate. "Two-layer trust… is table stakes, not an enterprise upsell." Grounds R1. Effort: M.

### PHASE 2 — THE NUMBER + THE CONSUMPTION SURFACE (§4)

- **4.1 FULL SAFEDELTA [G-6] to R3 spec** — "**300-400 git-free tarball tasks** (power: min detectable delta ~8.9pt at 200… — **200 is dead on arrival**), dynamic exploit ground truth, public validation + private frozen test + rolling refresh (**do NOT lean on SWE-bench Verified — OpenAI deprecated it Feb-2026, 59.4% flawed**), semantic-NN decontamination, paired McNemar + BCa bootstrap (10,000 resamples), per-CWE-family (MITRE Top 25). Publish ONE sentence + mandatory companions + benchmark card + public harness, decisive test private. **Effort: L — but it IS the GTM.**" — a faithful operationalization of R3, including R3's 300-400 power finding as a hard constraint ("200 is dead on arrival").
- **4.2 REVIEWER COCKPIT [G-5] — "keep in this push (amend Grok, who deferred it)."** "It is the daily-use retention surface that converts a quarterly audit artifact into recurring revenue (R1 two-step motion)." Three-level flow + structural triage (circuit-breaker 20%-budget-captures-69%) + fast-lane green + per-hunk receipts + **mandatory human final sign-off for auth/billing/privacy/data-deletion/migrations/public-APIs/CI.** Grounds R5. Effort: L. **Note the explicit disagreement with Grok recorded here: Grok deferred G-5, the Opus/ELAI synthesis over-rode that and kept it — because R5's cockpit is the recurring-revenue surface. This is a real, recorded synthesis decision where ELAI over-rode the adversarial pass with grounding from R5/R1.**
- **4.3 NOISE DISCIPLINE [new constraint on G-5]** — evidence-backed categories ONLY (security/correctness/test-gaps/compat/error-handling), style/format INADMISSIBLE (deterministic linters), every AI finding carries a failing test/trace/policy-hit/reproducer. Grounds R5/MSR-2026. — codifies R5's anti-noise rule as a hard constraint on G-5.

### GAPS TO ADD (§5 — the plan's own list of research-exposed gaps absent in F/G/H)

- VSA / verification-report/v1 signed verdict by the verifier plane (R2) → folded into 3.3.
- A custom in-toto **agent-decision predicate URI (stable)** for tool-use/plan/decision statements tied to SSB rows; "the May-2026 RFC is emerging — **ship a proprietary URI now, do not wait** (R2)." — directly addresses R2's agent-schema-immaturity window.
- Reviewer-reaching-rate NON-INFERIORITY guardrail as a first-class benchmark output (R3) → 4.1.
- Per-hunk machine verification receipts (R3+R5) → 4.2.
- **"A contractually-portable artifact-format spec the agency channel can hand to clients (R1) → the bundle must be a named deliverable, not an internal log."** — the SBOM gap I flagged against R1 is NOT explicitly named here; the plan names "contractually-portable format" but not SBOM specifically. **SBOM remains a residual gap even after the improvement plan.** (coverage-delta IS folded via 3.1 coverage.)
- Self-host/ZDR/SOC 2 (R1) → 3.5.

### DEFER / SCOPE-DOWN (§6 — the redundancy/over-engineering answer)

- **G-1 taint: KEEP but SCOPE** to the deterministic destructive-sink DENY at the 4 ingest sites with dependency-screened join — "cheap, substrate already shipped, defends the REAL Replit-class incident." **"DO NOT chase full FIDES/NeuroTaint provable-robustness (R2/NAACL Mar-2025: all 8 inline injection defenses bypassed >50% ASR — robustness-chasing is unwinnable and is how CodeGate got squeezed)."** THE HARD RULE that emerges: "**ban any LLM-judge from the load-bearing verdict path; only deterministic/architectural/off-agent signals decide accept/block (R2).**" "This reconciles Grok ('don't over-build taint') with Opus ('make the load-bearing path architectural')." — a clean synthesis directly grounded in R2's two adversarial refutations.
- G-7 conformal-refusal + Dual-LLM trifecta quarantine: **defer.**
- F-10 security-monotonicity gate: **defer** ("complex, marginal once the delta is proven").
- Open-weight full-recompute mode: optional second mode after cloud-path claim ships.
- **CUT: "token-level reproducibility beyond the action stream; any work that does not feed the RRA evidence manifest or the SAFEDELTA number."** — a sharp scope-cut consistent with R2 (tokens un-reproducible on cloud) and the one-number focus.

### THE THREE RISKS + FALSIFIERS (§7) and CLAIM DISCIPLINE (§8)

Risks map 1:1 to the Phase-0 kill-gates: R-1 number-too-small→F0.1, R-2 cloud-dep-breaks-repro→F0.2, R-3 buyers-won't-adopt→F0.3, plus existential build-risk (un-git-hardened number Endor discredits, 3.4 must precede 4.1).

Claim discipline (§8) is the most disciplined marketing-copy spec in the set, and it directly encodes every refutation:
- YES: "Independently-captured, off-agent evidence that a specific diff was verified by a system the coding agent does not control."
- "Trace authentication + signed evidence" on cloud = YES; "Deterministic replay" on cloud = **NO** (only open-weight mode). [absorbs R2]
- "We make your agent faster" = **NO (METR 19% slowdown; the agent is the weak part)**; "We make verifying autonomous output fast and defensible" = YES. [absorbs R1/R5]
- "Provably secure / can't be bypassed" = **NO (NAACL all-8-bypassed)**; "Deterministic off-agent gates the agent cannot fabricate, plus a measured residual-insecure-rate reduction" = YES. [absorbs R2]

**§8 is the improvement plan converting every refutation in the set into a marketing guardrail.** This is the correct way to metabolize refutations — not sanitize them into strengths, but bind them as claim-limits. It is the healthiest artifact in the set.

### WHAT THE RESEARCH DOES NOT ANSWER (§9 — ELAI's own honesty about the residual)

Verbatim open questions:
- "Does the gate produce a positive, significant Δ_safety on 300-400 tasks? (Only F0.1 → 4.1 answers.)"
- "Is the non-inferiority guardrail satisfiable?"
- "Does the off-agent evidence actually catch a fabricated PASS in a live red-team (**vs ELAI's own ~40-45% wire-dark rate**)?" — **NOTE: this is the improvement plan itself citing the "40-45% wire-dark" figure that the task's cross-check says was re-audited (s-20188) as NOT holding at P0 tier. See the cross-check finding below.**
- "The 2-5x verifier-throughput multiplier is a DESIGN INFERENCE, not proven — needs ELAI's own cockpit deployment." [absorbs R5's core limit]
- "Real willingness-to-pay for the bundle vs a $30/seat review bot; agency-as-channel vs agency-as-competitor economics."
- "Whether trace-authentication-only satisfies auditors/insurers/regulators or they demand recomputation ELAI can only give in open-weight mode." [the deepest unresolved R2 tension]

### CROSS-CHECK FINDING — the "40-45% wire-dark" figure

The improvement plan (§9, line 84) uses "ELAI's own ~40-45% wire-dark rate" as a live baseline for why the fabricated-PASS red-team matters. The task's cross-check states: "The motivating '40-45% wire-dark across ~187 plans' figure was re-audited one day later (session s-20188) and reframed as not holding at the P0 tier." Memory confirms: "[Wire-dark re-audit s-20188] — '40-45% wire-dark' largely STALE; 8 genuine; gap = ~37 missing allowlist rows." **FINDING: the improvement plan (authored 2026-06-08) rests, in one place (§9), on a wire-dark rate that a later audit (s-20188) found largely stale.** The dependency is mild — §9 uses it as motivation for a red-team that is worth doing regardless — but it should be flagged: the fabricated-PASS red-team's URGENCY was over-stated by a stale number. The red-team itself remains justified by R2/R5 (fabrication is a real class); only the "40-45%" magnitude is stale. This does NOT undermine the plan's core (F-4/off-agent doctrine); it means one motivating statistic is inflated.

### §10 FIRST CONCRETE MOVE + §11 ADDENDUM

§10: "Run Phase 0. Specifically **F0.1 at 50-100 tasks on the SHIPPED SeCodePLT runner path (F-2b, RUNPOD-PENDING) extended with git-stripping + dynamic-exploit scoring**, gate-ON-vs-OFF. That single run either lights the green for the whole plan or kills it cheaply." — **the named first move is F0.1, and it depends on the SeCodePLT runner being RUNPOD-PENDING.** §11 covers the critique-lenses (already summarized in §6 above); confirms they compose at the FRONT (spec/prose) while the off-agent spine governs the BACK.

### IMPROVEMENT-PLAN NET

- The single highest-leverage idea in the set: **§1.1 AGENT-AGNOSTIC VERIFIER** — verify anyone's (base_commit, diff, provenance) tuple, not just ELAI's; this claims R4's unowned neutral-notary moat AND dissolves both root risks (Anthropic-dependency + METR-slowdown) at once.
- The plan's best structural instinct: **Phase 0 falsify-first** (3 kill-gates sized to days-weeks) — grounded-by-default at strategy grain. First move = F0.1 small SAFEDELTA pilot.
- The most important NEW deliverable: **3.4 harness git-hardening** (existential — Endor will discredit an un-hardened number; 3.4 must precede 4.1).
- §8 claim discipline is the healthiest artifact in the set: it converts every refutation (METR, NAACL, R2-nondeterminism) into an explicit marketing claim-limit rather than sanitizing them.
- Two residual gaps the plan does NOT fully close: **SBOM** (names "contractually-portable format" but not SBOM specifically) and the **auditor/insurer-demands-recomputation** question (only open-weight mode can answer, unresolved).
- One stale dependency: §9 leans on the "~40-45% wire-dark rate" that s-20188 later found largely stale — inflates the urgency of the fabricated-PASS red-team, does not undermine the core doctrine.
- The plan itself over-rode Grok on G-5 (Grok deferred the cockpit; ELAI/Opus kept it as the recurring-revenue surface, grounded in R5/R1) — a documented, defensible synthesis disagreement.

---

## 8. 06-RESEARCH-DISTILLED-BUNDLE

Source: `workflow/research/grok-elai-trust/06-research-distilled-bundle.md` (38 lines). Purpose (line 3): "Use this if Grok struggles with the 8 raw uploads: paste it inline… It is a faithful compression of the five deep-research reports; the raw reports (R1–R5 .md in this folder) are the source of truth." So this file is a COMPRESSION artifact, not new research — I read it primarily to (a) verify my per-stream summaries against ELAI's own compression (they agree), and (b) capture its CROSS-REPORT TENSIONS section, which is the ONE piece of content in it that is NOT in the individual streams. That tensions section is where the set names its own contradictions, and it feeds §12.

### VERIFICATION AGAINST MY SUMMARIES

The compression's per-stream bottom-lines match the sections above with no material divergence. Spot-checks: R1 "routes through AppSec/code-review/supply-chain budgets, not a 'trust' category" ✓; R3 "detectable delta 200 tasks ~=8.9pp… 2-3pp claim on 200 won't persuade" ✓; R5 "0-20% work fully delegable" ✓; R2 "sigstore-rs EXPERIMENTAL/no attestation verify -> shell out to cosign" ✓. One number the compression surfaces that I should pin explicitly: **Corgea "transparent $39-49/dev/mo"** — this is the actual origin of the memory's "$39-49/seat" GTM anchor. It is CORGEA'S pricing (a competitor), re-used as ELAI's target band; and F0.3's WTP test asks "$40+/seat/mo". So "$39-49/seat" traces to Corgea's public price (R4), NOT to an ELAI measurement or an R1 recommendation. [EXTERNAL, Corgea] — do not cite it as validated ELAI pricing.

### THE CROSS-REPORT TENSIONS SECTION (the unique content — ELAI naming its own contradictions, verbatim-in-force)

This is the set's own honest catalogue of where the streams disagree. Each is real and each feeds §12:

1. **Price premium is hard.** "R1 'price premium above $24-30' vs R4 incumbents already at/above that with huge sales teams → 'premium' hard without a genuinely differentiated trust primitive." — R1 says charge above the commodity floor; R4 says the floor is already occupied by funded incumbents. The reconciliation: the differentiated primitive IS the off-agent attestation bundle (the thing R4 says is unowned). Premium is only defensible on the differentiated primitive, never on "better review."

2. **The moat is an INFERENCE, and adjacencies could close it fast.** "'Independent off-agent attestation is unoccupied' is an INFERENCE from public product surfaces; **Endor Agent Governance + GitHub artifact attestation are close adjacencies that could close it fast.**" — this is the single most important caveat on R4's headline finding. The moat is real *today* but (a) it is inferred from public surfaces (someone could be building it privately), and (b) two well-funded parties (Endor, GitHub) are one product-decision away from it. The window is open but not guaranteed to stay open. **This tempers the "moat is real" conclusion: it is a first-mover WINDOW, not a durable moat.**

3. **"Can't-lie spine" overstates robustness.** "R2 leans on CaMeL/FIDES/RTBAS while citing NAACL 'all 8 defenses bypassed >50% ASR' → inline taint is valuable but NOT provably robust; **calling them a 'can't-lie spine' overstates robustness.**" — the set catches its own marketing language. The off-agent EVIDENCE spine (F-4/coverage/attestation) is un-fakeable; the TAINT defense (G-1) is not provably robust. Conflating them into one "can't-lie" claim over-sells. The improvement plan §8 already fixes this (claim discipline separates the two), so this tension is resolved in the plan — but it is worth noting the raw research flagged it.

4. **Dynamic-exploit-truth vs commercial reality.** "R3 'dynamic exploit exec is the only real truth' vs R4 incumbents winning on static/SAST at scale → higher commercial bar." — R3's methodological rigor (dynamic exploitability) is more expensive than what the market currently rewards (static SAST). ELAI's number will be more rigorous than incumbents' AND harder/slower to produce. Rigor is a differentiator only if buyers value it over the incumbents' cheaper static numbers — which R1 says they do at the top of funnel but not at close.

5. **The self-undermining evidence posture.** "R5 '2-5x' is a design target not proven; **R1's case-study numbers are vendor-supplied/non-neutral — opposite of the independent-evidence posture ELAI sells.**" — sharp and self-aware: ELAI's whole pitch is "independent, non-self-reported evidence," yet the demand case (R1) rests heavily on vendor-self-reported case-study numbers, and the throughput promise (R5 2-5x) is an unproven design target. ELAI must produce its OWN independent numbers precisely because the ones motivating it are the kind ELAI's thesis says not to trust. This is §9's "what the research does NOT answer" restated as a tension.

6. **SUSVIBES underpowers the number.** "R3 wants 300-400 tasks for power; SUSVIBES is only 200 → an SUSVIBES-derived number may not survive enterprise due diligence." — the R3 power finding restated as a direct constraint on the named corpus.

7. **The agency channel vs the CodeGate squeeze.** "R1 'target cleanup agencies as wedge+channel' vs R4 'CodeGate shows standalone trust tools get squeezed' — **agencies already using Semgrep/Snyk have little reason to adopt unless the artifact is contractually portable.**" — the wedge (R1) and the death (R4 CodeGate) are in tension; the resolution is the *contractually-portable artifact* (the bundle the agency hands its client). If the bundle is not client-portable, the agency channel collapses into the CodeGate squeeze. This is why the improvement plan's §5 names "contractually-portable artifact-format spec" as a required deliverable, and why F0.3 tests exactly this ("a portable signed evidence pack you can hand your client").

8. **Proof-infra fragility.** "R2 Sigstore/Rekor v2 'usable but mid-migration' (witnessed checkpoints not live) = **operational fragility for a product staking credibility on proof infra.**" — ELAI stakes its credibility on cryptographic proof infrastructure that is itself mid-migration. If Rekor v2 discovery/witnessing breaks or changes, ELAI's attestations are affected. The improvement plan's 3.3 mitigation (archive trust-root snapshot, don't hardcode URL, shell to cosign) is the direct response.

### DISTILLED-BUNDLE NET

- Confirms my per-stream summaries with no material divergence; the compression is faithful.
- Pins the "$39-49/seat" figure to its true source: **Corgea's public competitor pricing (R4)**, re-used as a target band and tested (not validated) by F0.3's "$40+/seat" WTP question. Not an ELAI measurement.
- Its unique value is the 8-item CROSS-REPORT TENSIONS section — the set naming its own contradictions. The two sharpest: **(2) the moat is an INFERENCE with Endor + GitHub one product-decision from closing it — a first-mover window, not a durable moat**; and **(5) ELAI's demand case rests on vendor-self-reported numbers, the exact kind its own thesis says not to trust — so ELAI must generate its own independent numbers to be credible.** Both feed §12.

---

## 9. SET-INTERNAL PLAN VIEWS — F / G / H SUMMARIES

Sources: `ELAI-F-plan-summary.md` (286 lines, gen 2026-06-01), `ELAI-G-plan-summary.md` (225 lines, gen 2026-06-02), `ELAI-H-plan-summary.md` (1038 lines, gen 2026-06-03..06). These are the trust-spine plan corpus AS IT EXISTED when the research was commissioned — the "attachments" the Grok prompt hands over. They are NOT research; they are ELAI's own plan index, and their value to this sweep is threefold: (a) the exact per-piece scope/wiring that R2/R3/R5 grounded or refuted, (b) a dense catalogue of self-flagged DRIFT (stale schema versions, wire-dark substrate, RunPod-pending gates) that feeds §10, and (c) proof that the plans encode the wargame/wiring doctrine at unit grain (every entry has an `observable_effect` = W-1 wiring done-gate). I read all three fully; this is one consolidated append per instruction.

### F-PLAN (F-1..F-13) — the write-time gates (gen 2026-06-01, 13 sessions)

Premise (verbatim): "The differentiator vs B/C/D/E is the ENFORCED done-gate: a session is not DONE until wiring_contract_check.py --run-trigger PASSes… cargo-green-alone is rejected. Default-off env gates as a terminal state are banned (W-2)." Enforcement doctrine W-1..W-10.

The 13 pieces (each: cluster | novelty | feasibility | the wire-dark substrate it activates | observable_effect):
- **F-1 VibeGate** — one PreToolUse handler fans a write across 4 axes (code-vuln, hallucinated-dep, secret, malicious-skill), fail-closed, "vibesec" rule domain seeded mutable=0. Ground truth: ALWAYS_INCLUDED_DOMAINS already has "security" (rule_engine.rs:187), mutable column exists (schema.rs:46). REVISED to Tier-1 (code-vuln AST/GREP + secret GREP, pure in-process) ships now; Tier-2 (dep-existence + mcp-scan, network) deferred. Explicit: "CodeGate died of FP friction - bound it before release" — the R4 CodeGate lesson already internalized at the F-1 scope.
- **F-2 / F-2a / F-2b / F-2c THE KEYSTONE** — code-vuln ruleset (SQLi/XSS/weak-crypto/authz) + "the FIRST measured pre-write insecure-rate number". Critical honesty: "SVEN 59.1→92.3% and CodeGuard+ 43.2→76.0% are VERIFIED but on CodeGen-2.7B - they do NOT transfer to Qwen2.5-Coder-7B-abliterated; the number must be measured." **F-2b is gated on RunPod** ("F-2b - run SeCodePLT/CodeGuard+ on Qwen, produce the first measured before/after"). F-2c (post-C10 ext): 4 high-precision CWE rules (798/321/532/95) + "RUN the shipped `elai bench seccodeplt` for the FIRST measured static-gate baseline" — and flags "F-2b's bench ships but **was never RUN**". This is the KEYSTONE deliverable, and as of the summary it had NEVER been run. Direct tie to §11: PS-1 RUNNER_PRESENT-not-MEASURED.
- **F-3 CaMeL/Tool-Filter** — control-plane isolation; lock researcher tool-set before untrusted byte. CaMeL 77% (VERIFIED), AgentDojo Tool Filter cut ASR 57.7→6.84% at 73% utility (VERIFIED). Phase 1 (Tool Filter) ships; Phase 2 (full CaMeL SymbolicVar) conditional on IPC overhead. grounding "OVERSTATED on token overhead."
- **F-4 test-integrity (the literal cant-lie half)** — bind every test PASS to orchestrator-captured stdout_hash (blake3); auditor cannot emit passed=true without a matching hash row; fabrication signature = passed=true + stdout_hash=None. Ground truth: TestOutcomePayload has NO stdout_hash; C-22 already adds it on the feasibility path. Pre-req on C-27+C-28. **This is what R2 sharpened (add coverage_profraw + wrapper-digest) — F-4 as scoped binds stdout only.**
- **F-5 staleness-quarantine** cross-subsystem bi-temporal cascade. **F-6 learned-skill trust ladder** (S1-S4 gating autonomy). **F-7 safety-lattice completeness proof** (prove mutable=0 rule set complete/monotone/conflict-free) — "grounding OVERSTATED (Springdrift numbers SYNTHETIC/UNREPRODUCED)". **F-8 SCF symbolic conflict detector** — "grounding OVERSTATED (27.9% precision is a simulation limitation; SCF is sim-only, 0 stars, empty tests/)".
- **F-9 hallucinated-dep gate at write-time** — Spracklen 19.7% hallucinated / 205,474 fake names / 43% recur (VERIFIED). **This is the piece R4 says Socket owns** (Socket MCP targets dependency hallucination). ELAI's F-9 competes with a funded specialist.
- **F-10 security-monotonicity gate** — block if REFINE iteration N introduces a CWE class absent at N-1. "Security degrades +37.6% over 5 AI-improvement iterations (VERIFIED)". Hidden cost flagged: "the CWE classifier it calls does NOT exist and building one calibrated for Rust is L-effort itself." **The improvement plan (§6) defers F-10.**
- **F-11 config-only DGM generation** (driver closing propose→evaluate→guard→insert; wire-dark behind ELAI_DARWINIAN_LOOP_ENABLE — a deliberate W-2 deviation). **F-12 semantic contradiction NLI judge** (local ONNX, not cloud — "a cloud LLM judge breaks ELAI_SEED"). **F-13 session archive on schema-bump** (wire-dark, ELAI_SSB_ARCHIVE_ON_BUMP_ENABLE).

**F-plan reading**: the write-time gate layer is dense, honestly self-graded (multiple "grounding OVERSTATED" self-flags, synthetic-numbers-are-not-targets discipline), and heavily dependent on RunPod for the one number that matters (F-2b). The CodeGate-FP lesson is baked in at F-1. The wire-dark pattern is pervasive even here (F-11/F-12/F-13 ship dark behind flags).

### G-PLAN (G-1..G-10) — evidence / provenance / measurement (gen 2026-06-02, 10 sessions)

Premise: "GD-1 (COMPOSE-NOT-RESTATE: G consumes F-gate verdicts, re-implements none) + GD-2 (every emitted number is itself recomputable from the SSB journal)." Adds GD-1..GD-5 on top of F's W-1..W-10.

- **G-1 harness-wide taint** — "ELAI already SHIPS the RTBAS-style label type (provenance.rs: Provenance + Provenanced<T>… capability_gate.rs) but the constructors are built ONLY in tests = **dead wiring**." G-1 constructs at the 4 real ingest sites. **CRITICAL PITFALL (GD-4): RTBAS's GLOBAL join collapses utility to 33% (Slack over-tainting) — MUST use dependency-screened join.** [this is the 33% figure from 00-PROMPT, sourced here to RTBAS arXiv:2502.08966]. Improvement plan §6: KEEP but SCOPE, don't chase provable robustness.
- **G-2 RRA [THE KEYSTONE of G]** — the portable msgpack+zstd bundle. "claude-replay is viewer-only (no re-exec/seed/divergence) = first-in-category." **HONEST SCOPE (GD-5): cloud-lane = reproducible-ACTION-STREAM not reproducible-TOKENS; verify-divergence not full re-execute.** This is the exact honesty R2 independently confirmed correct. Note: "RESUME caveat: recover_from_checkpoint is a STUB."
- **G-3 runtime blast-radius gates** (post-expansion FS gate + egress IP-pin) — grounded in the rm-rf #10077/#12637 incidents + "LLM permission classifier collapses to 81% FNR under adversarial scope-escalation" → deterministic gate beats classifier. **G-4 MCP rug-pull re-pin gate** — recompute schema-pin on every tools/list_changed. OWASP MCP03:2025. (both are "cant-wreck" lever, not in the R1-R5 trust-bundle core but real hardening).
- **G-5 Reviewer Cockpit + DSR** — "DSR = (lines needing full attention)/(total changed lines)… surface a finding ONLY when backed by a deterministic gate verdict or a failing test (no LLM-opinion noise)." Grounded in METR + "code-review-agent PRs sit at 0-30% signal in 60.2% of cases (arXiv:2604.03196)" — **the exact R5/MSR-2026 numbers.** The MEASUREMENT is explicitly framed as a HYPOTHESIS the study TESTS, not a pre-claimed result. Depends G-2. **This is R5 operationalized, name and all.**
- **G-6 SAFEDELTA** — "gate-ON-vs-gate-OFF ABLATION (GD-3) measuring marginal residual-insecure-rate that reaches a human, on git-stripped held-out tasks." Suites: SUSVIBES-200, BaxBench-392 (DYNAMIC exploit exec), SeCodePLT, AgentDojo. Anti-gaming: "git-history strip + a cheat-detector cloned from Endor's finding that SWE-Agent+Opus exploited git history in 163/200=81%." **CAVEAT (whats_missing): "with 200-392 tasks + small per-CWE cells, CI tightness for a significance claim is unresolved."** — G-6 itself flags the R3 power problem. **Depends G-2 + F-2b LIVE. Effort XL, WAVE 3.** This IS R3 operationalized, and it inherits R3's power warning as a stated open caveat.
- **G-7 confidence PRODUCER + Dual-LLM trifecta quarantine** — "ELAI SHIPS the conformal refusal CONSUMER but the composite claim_score PRODUCER is a deferred slot." ECE 0.32→0.03 (Spiess, CONFIRMED). Improvement plan §6: DEFER.
- **G-8 signed in-toto attestation + Article-50 verifier CLI** — "custom in-toto v1 predicate… signed via a LOCAL blake3-Merkle proof (offline default) or opt-in cosign keyless. New `elai verify-attestation`." **"POSITION AS 'designed to support Article-50 marking', NOT 'certified compliant'… Penalty tier is EUR 15M/3% (Art-99), NOT 35M/7%."** — the EU-AI-Act claim discipline is precise and self-corrected. This is the piece R2's Rust-tooling blocker (sigstore-rs can't verify) bites; the summary's "local blake3-Merkle default / cosign opt-in" is the response — it hand-rolls the default rather than depending on sigstore-rs. **VSA is named here ("SLSA-VSA-style PASSED/verifiedLevels rollup") — so the VSA primitive I flagged as a gap against 00-PROMPT IS present in G-8's fuller description.**
- **G-9 write-time secret/PII redaction** — "ssb.rs:5 'no UPDATE/DELETE' forces WRITE-TIME redaction — cannot scrub after." Local-only (no cloud egress). "CORRECTIONS (binding): EU penalty EUR 15M/3% not 35M/7%; PRvL Presidio numbers REFUTED." — G-9 is the precondition for shipping the RRA/attestation to a client without leaking secrets.
- **G-10 model-drift regression canary** (post-G-9 ext) — the R4/Anthropic-dependency root risk made into a gate: pin (model, system_fingerprint, lane), re-run canary on identity change, "SURFACE never auto-swap." AgentAssay reports a silent provider update 93%→71% routing.

**G-plan reading**: G is where the actual trust-bundle product lives (G-2 RRA + G-8 attestation + G-5 cockpit + G-6 SAFEDELTA), and it maps almost 1:1 to R4's five-primitive defensible shape and R1's trust-bundle. The self-flagged caveats (G-2's cloud-token honesty, G-6's power caveat, G-8's Article-50 discipline) are exactly the refutations the streams raised — the plan already carries them.

### H-PLAN (H-1..H-54, 11 tracks) — the sprawling build corpus (gen 2026-06-03..06)

H is 54 sessions across 11 tracks. Most of it is NOT trust-spine (Track 1 efficiency, Track 2 canvas, Track 4 UX/memory), but several tracks are directly trust-relevant and several cross-check facts live here:
- **Track 1 (H-1..H-9) EFFICIENCY** — "makes ELAI provably CHEAPER on the SAME cloud model" mostly by ACTIVATING wire-dark caching/output/routing substrate. H-9 = the cost cockpit, "the believe-the-number instrument" for the efficiency story. HD-1: every efficiency number MEASURED per config, never multiplied. (This is the memory's "efficiency+trust direction" — cheaper AND provable.)
- **Track 2 (H-10..H-16) CANVAS** — 2D generative canvas replacing the 3D orbit; "whose MOAT is the journaled + reproducible + taint-tagged op-stream underneath (G-1/G-2)." H-15 explicitly "[the trust-spine moat]" — the canvas op-stream IS G-1-taint + G-2-RRA applied to a UI. This is where the trust spine touches the product surface the whole ELAI-UI-rework (per memory) is queued behind.
- **Track 3 (H-17..H-29) TRUST-HARDENING** — the largest trust track: making an autonomous trading-strategy loop trustable via deterministic off-agent gates. TD-1..TD-5 (persisted-N / frozen-evaluator / LLM-is-T3-advisory / verify-the-verifiers / honest-residue). H-17 persisted trial ledger (the selection-bias KEYSTONE), H-18 Deflated-Sharpe/PBO/MinBTL gate, H-23 same-seed re-run hash-equality, H-25 result-manifest re-derivation (extends F-4 from "test PASS honest" to "result honest"), H-27 gate self-validation (verify-the-verifiers), H-28 reward-tampering gate. **This track is memory's "opus first real run: H-28 landed ebb2f50+pushed" — so at least one Track-3 piece SHIPPED post-research.**
- **Track 6 (H-39..H-42) GROUNDED-REASONING** — verdict: "ELAI's trust spine is ALREADY strongly aligned with the rigorous 2024-2026 literature… Track 6 is FOUR small ACTIVATION/EXTENSION refinements, NOT a redesign." H-39 abstention-aware promotion (KEYSTONE, penalty=t/(1-t) from OpenAI 2509.04664). "2507.07505's 'verification is as hard as the task' pessimism is true ONLY for a CORRELATED verifier; ELAI's cross-family + off-agent + sound-checker design is the engineering refutation." — **this is ELAI directly engaging the deepest theoretical threat (verification-as-hard-as-generation) and claiming its architecture is the refutation.**
- **Track 7 (H-43..H-48) DGM-VERIFICATION** — the self-improvement filter. META-FINDING: "A RECURSIVE SELF-IMPROVEMENT LOOP WITHOUT A REAL, UN-GAMEABLE, STRONGER-THAN-GENERATOR VERIFICATION FILTER PROVABLY COLLAPSES; WITH ONE, IT PROVABLY CONVERGES." Found ELAI's DGM ships with TWO placeholder filters (derived_alpha = blake3(config_bytes); trap-scoring = keyword grep the code self-flags "unsound"). H-43 execution-grounded fitness = KEYSTONE. This is memory's H-plan Track 7.
- **Tracks 8-11** — reversibility (H-49 undo/redo time-travel), context-lifecycle (H-50/51 — "cloud and local need OPPOSITE context policies"), review-anchoring (H-52 deterministic snippet→hunk resolver, closes G-5's Open-Question-2), external-steals (H-53 citation pre-binding from Perplexity, H-54 obligation-closure from AlphaProof).

### THE DRIFT SIGNALS (the highest-value content in the F/G/H views for this sweep — feeds §10)

The summaries flag their OWN staleness pervasively, which is exactly what a drift-check needs:
1. **Schema-version drift WITHIN the set**: F/G reference an implicit baseline; Track 1/2 header says 207/484; Track 4 says "LIVE baseline is 220/497… CLAUDE.md's 191/464 AND the Track-1/2/3 header's 207/484 are STALE"; Track 7 says "LIVE baseline 224/501… 207/484 and 220/497 are EARLIER snapshots, now STALE." **So across the H-plan's own authoring (2026-06-03 to 06-06), the SCHEMA_VERSION moved 220→224 and TOTAL_VARIANTS 497→501 — the plan corpus was drifting under itself in real time.** This is direct evidence that the plan is a living, fast-moving target, and that any single summary is a snapshot.
2. **The G-summary is stale about itself**: H-23 (Track 3) explicitly says "the SHIPPED RRA (rra.rs:200, --emit-rra, replay run_subcommand.rs:584 — G-2; **G-summary 'ABSENT' STALE**)." So by 2026-06-05, the G-2 RRA had SHIPPED, and the G-plan-summary's characterization of RRA-resume as "ABSENT" was already stale. **G-2, the KEYSTONE, shipped between the G-summary and the H-Track-3 authoring.** Confirms memory ("Wiring-fix-elai session… RRA shipped").
3. **Wire-dark is the dominant substrate pattern**: nearly every F/G/H piece describes activating "wire-dark" / "dead wiring" / "built ONLY in tests" / "zero prod callers" substrate (G-1 taint dead-wired, G-7 producer deferred, F-11/12/13 dark behind flags, H-1 cache_check dark, H-7 RuntimeOrchestrator unwired, H-30..H-38 memory substrate wire-dark, DGM filter placeholders). **This IS the ~40-45% wire-dark reality made concrete — the trust spine is largely BUILT-BUT-DARK substrate awaiting activation, not un-built.** This is the single most important structural fact about the trust spine: it is an activation problem, not an invention problem. The improvement plan's one-sentence verdict says exactly this ("the job is not to invent, it is to externalize").
4. **RunPod-pending gates the keystone number repeatedly**: F-2b, G-6, and every H bench leg are RUNPOD-PENDING. **The one number the GTM stakes on cannot be produced without RunPod, and as of every summary it had not been.** Direct tie to §11 (PS-1 RUNNER_PRESENT).
5. **Self-flagged citation risk**: Track 4 flags "future-dated arXiv IDs 2605.06527 (STALE), 2603.23848, 2512.16962… beyond the 2026-01 knowledge cutoff and NOT web-verified" and "ELAI's prior bad-cite history (B-40/B-42/E-40)." — honest about its own citation-hallucination risk, consistent with the memory note "verify paper citations."

### F/G/H NET

- The trust-spine PRODUCT (the R1 trust-bundle / R4 five-primitive shape) lives in G: G-2 RRA + G-8 attestation + G-5 cockpit + G-6 SAFEDELTA, with F-1/F-2/F-4/F-9 as the policy-bundle gates feeding it. The mapping to R1/R2/R3/R4/R5 is near-exact — the plans were built to the doctrine, and the summaries already carry the streams' key caveats (G-2 cloud-token honesty, G-6 power caveat, G-8 Article-50 discipline, G-1 33%-join pitfall).
- The single structural truth: **the trust spine is overwhelmingly BUILT-BUT-WIRE-DARK substrate awaiting activation, not unbuilt design.** Every summary is a catalogue of "ships in tests only / dead wiring / zero prod callers." Externalizing (the improvement plan's verb) means WIRING, not inventing.
- The plan corpus was drifting under itself in real time (schema 220→224 across four days of H authoring; G-2 shipped mid-set making the G-summary self-stale). Any single view is a fast-decaying snapshot — a structural reason the improvement plan's Phase-0-first discipline matters more than the sprawling H build list.
- The keystone number (F-2b/G-6 SAFEDELTA) is RunPod-gated and, per F-2c's own note, the shipped bench "was never RUN." This is the through-line to §11's PS-1 gap.

---

## 10. PLAN TRACE — recommendations vs the LIVE plan corpus, and drift

Method: read `docs/thefinalglance/ELAI-SPEC-OVERVIEW.md` (grepped for trust-spine framing, not full-read — it is a 50KB research doc), and traced each improvement-plan recommendation into the live plan-letter corpus (`docs/thefinalglance/{B..Q}-plan/`) via grep + `elai_status_query.py` (advisory cache, snapshot 28m old, git 494ef36) + `proof_spine_status.py`. The question per the task: for every improvement-plan recommendation, did it become a plan unit (letter+number), and its status? A recommendation with no plan unit is a finding.

### THE SPEC-OVERVIEW CONFIRMS THE TRUST SPINE IS THE ORGANIZING THESIS

`ELAI-SPEC-OVERVIEW.md` names the doctrine at line 22 ("off-agent evidence — a gate measures an artifact the generator did not write") and enumerates the exact trust-bundle product: line 216 "the Reproducible-Run Artifact (RRA)", line 220 "SAFEDELTA", line 222 "signed in-toto provenance attestation with an offline verifier", the reviewer cockpit ("vouched hunks"). Line 288: F-1 write-gates + RRA emission = "the wedge unbundled." Line 572: "the product (RRA, SAFEDELTA, attestation, reviewer cockpit)." **The trust spine is not a side-track in the spec; it IS the spec's product.** And critically:

**Line 369 (verbatim): "The first plan that points the trust spine OUTWARD — from verifying the agent's own code to verifying ANY [agent's code]."** — this is the improvement plan's §1.1 AGENT-AGNOSTIC VERIFIER (the single highest-leverage idea in the whole set) promoted into a committed spec-overview letter. The pivot the improvement plan proposed on 2026-06-08 became a planned direction in the live spec. [MEASURED-BY-US — grep-confirmed in spec overview; the exact letter not pinned here, flagged UNVERIFIED which letter.]

### RECOMMENDATION → PLAN-UNIT TRACE (the core deliverable of this section)

| Improvement-plan recommendation | Became a plan unit? | Live status |
|---|---|---|
| §1.1 AGENT-AGNOSTIC VERIFIER (verify any agent's diff) | YES — spec-overview line 369 "points the trust spine OUTWARD" | committed direction; specific letter UNVERIFIED |
| §1.2 / §5 THE PRODUCT IS THE BUNDLE (contractually-portable) | YES — **I-44** "zero-retention-after-verification export mode" ("makes the evidence bundle shippable to a client without a retention liability") | I: 54/54 wired per status cache → **SHIPPED-WIRED** |
| Phase-0 **F0.1** SMALL SAFEDELTA pilot (50-100 tasks, gate-ON/OFF, dynamic-exploit) | YES — **I-43** "claim scoper + Phase-0 kill-gate" (verbatim: "the cheap falsifiers grounded-by-default demands BEFORE the G-6 XL build. PILOT — the shipped seccodeplt/safedelta runner at reduced N (50-100), gate-ON vs OFF, dynamic-exploit scored") | I-43 wired (I: 54/54); **but the PILOT NUMBER itself is not produced — PS-1 GATED, see §11** |
| Phase-0 **F0.2** reproducibility-claim scope (local vs cloud divergence) | YES — folded into I-43 ("a >5% cross-lane divergence emits trace-authentication-only wording, never replay (GD-5)") | wired as claim-wording logic |
| Phase-0 **F0.3** channel WTP (agency discovery calls) | NO PLAN UNIT — this is a human/market activity, not code. **FINDING: no evidence in the corpus that the 3-5 agency discovery calls were run.** The kill-gate that de-risks the entire GTM channel is the one with no artifact. | UNVERIFIED / likely not done |
| §3.1 off-agent test-evidence core (F-4 + coverage) | YES — F-4 (shipped) + H-25 result-manifest extends it | F: 10/14 wired; H-25 in Track-3 |
| §3.2 G-2 RRA | YES — G-2 | **SHIPPED** (G: 8/10 wired; H-23 confirms "SHIPPED RRA, G-summary ABSENT STALE") |
| §3.3 signed VSA/attestation the agent never authors | YES — G-8 | **SHIPPED** (attestation.rs confirmed per memory; G: 8/10 wired) |
| §3.4 harness git-hardening (existential) | YES — folded into G-6/I-43 anti-gaming (git-strip + Endor cheat-detector) | part of the safedelta runner design |
| §3.5 table-stakes (self-host/ZDR/SOC2/frozen gate) | PARTIAL — I-44 ZDR export shipped; SOC2/self-host are ops, no code unit | I-44 wired; SOC2 UNVERIFIED |
| §4.1 FULL SAFEDELTA (300-400 tasks) | YES — G-6 | **GATED — never produced live (PS-1)** |
| §4.2 reviewer cockpit (kept over Grok's defer) | YES — G-5 + **H-52** (deterministic finding-anchoring closes G-5's Open-Question-2) | G-5 wired; H-52 in Track-10 |
| §4.3 noise discipline (evidence-backed categories only) | YES — G-5's "no LLM-opinion noise" rule | in G-5 |
| §5 custom in-toto agent-decision predicate URI | YES — G-8 predicate | shipped in G-8 |
| §5 reviewer-reaching non-inferiority guardrail | YES — G-6 companion metrics | in G-6 (gated) |
| §8 CLAIM DISCIPLINE (say exactly this, no more) | YES — **K-1 ELAI-CLAIMS-POLICY.md + claims_policy_lint.py** (both PRESENT/SHIPPED; the conditional-routing rule loads it on any docs/strategy edit) | **SHIPPED — the §8 marketing guardrail became a frozen off-agent lint gate** |
| §6 G-1 taint SCOPED (not full robustness) | YES — G-1 | G: 8/10 wired; taint partial |
| §6 DEFER G-7, F-10 | YES — deferred as planned | G-7 partial, F-10 not prioritized |
| §11 critique lenses L1-L8 + critique-only mode | Advisory tool, wired into plan-improvement | present as ELAI-CRITIQUE-LENSES |

**Additional derived units the trace surfaced (recommendations that SPAWNED new letters):**
- **I-38 QDELTA** — "the quality sibling of SAFEDELTA: defective diffs reaching a reviewer per 100" — a new metric derived from the SAFEDELTA shape (composes G-6 runner shape).
- **K-2 routing economics ($/verified-task)** = PS-4 — the efficiency-trust number.
- **K-4 gate-tax** = PS-2 — composed gate-tax vs locked ceiling.
- **L-10 hallucination-delta** = PS-3.
- **K-18 case-reuse** = PS-5. **J-56** "living eval corpus… ACTIVATION of the dark E-54 + never-invoked G-6" — confirms G-6 "CLI+dashboard wired, bench never invoked, 0 rows."
- **L6 (in an L-plan billing track)** "the $/resolved-task SAFEDELTA measurement that GATES the '$20=$200' pitch… RunPod-gated" — the SAFEDELTA number is load-bearing for a PRICING claim, and still RunPod-gated. And explicitly: "a '$20=$200' claim with no L6 SAFEDELTA number FAILS the K-1 claims-policy lint" — **the claim-discipline gate (K-1) is now actively BLOCKING marketing claims that lack the un-produced SAFEDELTA number.** This is the claim-discipline working as designed AND a live demonstration that the un-produced number is gating real GTM copy.

### THE SINGLE MOST IMPORTANT TRACE FINDING

**Almost every improvement-plan recommendation became a plan unit, and the machinery is largely SHIPPED-WIRED — EXCEPT the two things that were always going to be hardest: (1) the SAFEDELTA NUMBER itself (RunPod/pod-gated, never produced live), and (2) the F0.3 agency WTP validation (a market activity with no code artifact and no evidence it ran).** The plan executed the CODE recommendations faithfully; it did NOT execute the two FALSIFICATION recommendations that gate the thesis. This is the deepest drift: the improvement plan's core discipline was "falsify before you build" (Phase 0 before Phase 1), and the actual execution built Phase 1 (F-4, G-2, G-8, G-5, I-43/I-44 machinery all wired) while the Phase-0 kill-gates that were supposed to precede it (F0.1 the live number, F0.3 the WTP calls) remain unrun. **Build-first happened; falsify-first did not.**

### DRIFT: the set's F/G/H summaries vs the live indexes

- The set's F/G/H summaries were snapshots at SCHEMA 207-224. Live is now further along (git 494ef36, 2026-07). The G-summary's "RRA ABSENT" was already stale by H-Track-3 (2026-06-05); today G-2 RRA is SHIPPED. So the set's own plan views UNDERSTATE what shipped — the drift is in the SHIPPED direction (more got built than the summaries show).
- **Wire-dark rate DROPPED materially**: the live status cache shows G 8/10 wired, F 10/14, H 47/54, I 54/54. The "~40-45% wire-dark" the research rested on is NOT the current state — the enforcement stack drove it down (this is exactly PS-6, MEASURED, see §11). The motivating statistic for the whole trust-spine urgency is now stale in ELAI's favor.
- Later letters (I/J/K/L/M/N/O/P/Q) did not exist when the research was commissioned; they are the CONTINUATION of the trust spine (I-43/I-44 the Phase-0 units, K-1 claims-policy, K-2/K-4/K-18/L-10 the PS-benches, the agent-agnostic-verifier letter). **The trust spine grew from F/G/H (the researched corpus) into I-through-Q (the post-research execution), and the improvement plan's recommendations are legible all through it.** The research demonstrably shaped the roadmap.

### PLAN-TRACE NET

- The improvement plan's recommendations overwhelmingly BECAME plan units, and most machinery is SHIPPED-WIRED (RRA, attestation, cockpit, claims-lint, I-43/I-44 Phase-0 machinery, the SAFEDELTA runner).
- The agent-agnostic-verifier pivot (§1.1, the highest-leverage idea) became a committed spec-overview direction ("point the trust spine OUTWARD," line 369).
- **The two un-executed recommendations are the two that FALSIFY the thesis**: the live SAFEDELTA number (pod-gated, never produced) and the F0.3 agency WTP validation (no artifact, no evidence). The plan built the trust product but never ran the two cheap tests that would tell it whether the product is demanded and whether its number is real.
- Drift is mostly in ELAI's favor: more shipped than the set's summaries show, and wire-dark dropped sharply (PS-6). The "40-45% wire-dark" urgency is stale.

---

## 11. PROOF SPINE mapping — which research stream feeds which PS number

Sources: `workflow/reasoning/proof-spine-2026-07-05.md` (the ruling), `proof-spine-ps6-wire-dark-rate-2026-07-09.md`, `proof-spine-ps5-case-reuse-2026-07-17.md` (the two landed ledgers), + `proof_spine_status.py` (live, 2/6 MEASURED). The proof spine is the 2026-07-05 ruling that "the corpus's credibility compresses onto six measured numbers." It POST-DATES the trust research (2026-06) by a month, and is the direct descendant of it: the research argued a number must be produced, the proof spine turned that into a build-order law over six numbers.

### THE SIX NUMBERS AND THEIR LIVE STATUS (from proof_spine_status.py, 2026-07-17)

| PS | Number | Owner | Status TODAY | Lane |
|---|---|---|---|---|
| PS-1 | **SAFETY DELTA** (gate-ON vs OFF residual-insecure-rate reaching a reviewer) | G-6 safedelta; precursor F-2b seccodeplt | **GATED** (pod) | static-gate / seccodeplt-curated-corpus, **no live model inference; live model-in-loop headline needs ELAI_E2E_POD_ENDPOINT** |
| PS-2 | composed gate-tax vs locked ceiling | K-4 | **RUNNER_PRESENT** | fixture composition, no model |
| PS-3 | hallucination delta (CGG ON/OFF) | L-10 | **PLANNED** | — |
| PS-4 | routing economics ($/verified-task) | K-2 (live cost blocks on J-28) | **RUNNER_PRESENT** | fixture qwen-7b-local vs claude-sonnet-cloud, no live inference; cloud-$ blocks on J-28 |
| PS-5 | case-reuse compounding | K-18 | **MEASURED** (2026-07-17, today) | static-retrieval bm25, no generative model |
| PS-6 | wire-dark rate, longitudinal | dated ledger | **MEASURED** (2026-07-09) | static-sweep, no model |

**2/6 MEASURED. The two measured (PS-5, PS-6) are the two that need NO model and NO pod — pure static/retrieval numbers. Every number that requires live model inference (PS-1, PS-2, PS-3, PS-4) is GATED, RUNNER_PRESENT, or PLANNED — NONE produced live.**

### STREAM → PS-NUMBER MAPPING

- **R3 (measurement / safety-delta) → PS-1, directly and exclusively.** R3 IS the design spec for PS-1. The formula (residual-insecure-at-review, paired, per-CWE, bootstrap CI, utility guardrail), the corpus (SUSVIBES/BaxBench/SeCodePLT/AgentDojo), the git-stripping, the 300-400-task power requirement — all of PS-1's design is R3. **PS-1 is the number the entire commission ("everything gates on ONE unproven SAFEDELTA number") was about, and it is STILL not produced.** R3 → PS-1: DESIGNED, RUNNER SHIPPED, NUMBER GATED.
- **R5 (human-as-verifier) → the DSR/cockpit number (G-5), which is NOT one of the six PS numbers.** This is a finding: R5's core measurable — the 2-5× verification-throughput multiplier, which R5 names "the next decisive evidence gap" — is NOT on the proof spine. G-5's DSR is measured-from-SSB (deterministic) but the STUDY R5 calls for (time-to-approve through the cockpit vs plain diff, escaped-defect guardrail) is not a PS number. **The load-bearing stream's headline number has no proof-spine slot.** Either it should (a 7th number: verification-throughput), or it is deliberately out-of-scope because it needs a human-subject study the spine can't automate.
- **R4 (competitive) → PS-4 (routing economics, $/verified-task) indirectly.** R4's moat argument is "the neutral notary"; the closest measurable is PS-4's "resolves more tasks per dollar" (the L6 billing pitch cites it). R4's actual finding (the moat is the POSITION, not a number) has no PS slot — correctly, because it is a strategic claim, not a measurement.
- **R1 (demand) → no PS number.** R1 is the market case; PS numbers are technical/economic. R1's "the benchmark doesn't close, a real finding does" is an argument AGAINST over-weighting PS-1, not a number to produce.
- **R2 (technical) → no direct PS number; underlies the RRA/attestation that make PS-1..PS-6 journal-recomputable (GD-2).** R2 is infrastructure for the numbers' credibility, not a number itself.

### THE CROSS-CHECK FINDINGS (both confirmed against the live ledgers)

**Cross-check 1 — the "40-45% wire-dark" figure (PS-6).** The task flagged: "re-audited one day later (s-20188) and reframed as not holding at the P0 tier." The PS-6 ledger (2026-07-09) CONFIRMS and quantifies this:
- Pre-stack anchor: "~40-45%" (plan-grain, `wiring-enforcement.md`, ~187 plans) AND the 2026-06-04 audit's "28% SHIPPED_WIRED over 213 plans."
- Post-stack MEASURED: **933 DARK pub fns / 12,137 = 7.69%** (fn-grain, static, `production_caller_check.py` verdict logic over 1,156 files).
- **The ledger itself flags the unit-mismatch, honestly**: "This is DIFFERENT from the pre-stack anchor's unit, which is PLAN files… A plan-grain number and a symbol-grain number are not directly comparable; the delta below is DIRECTIONAL, not a rigorous before/after on the same measuring stick."
- **FINDING: the "40-45% wire-dark" motivating figure that the improvement plan §9 leaned on (fabricated-PASS red-team urgency) is (a) plan-grain, (b) not the same unit as any current measurement, and (c) superseded by a 7.69% fn-grain post-stack number.** The trust-spine urgency it implied is stale. The improvement plan's use of it (§9) is the one stale dependency in an otherwise disciplined document. This matches the s-20188 re-audit (memory: "'40-45% wire-dark' largely STALE; 8 genuine"). **The research and improvement plan rest, in this one spot, on an inflated number — flagged.** It does not undermine the off-agent doctrine (which R2 independently grounds); it inflates one motivating statistic.

**Cross-check 2 — R3 is the safetydelta stream; is its design what the runner implements, and what is the gap?** Confirmed:
- The runner (`.elai_cc/crates/orchestrator/src/benchmarks/safedelta_runner.rs`) EXISTS and references PoC/dynamic-exploit strings. I-43 (the Phase-0 pilot unit) designs the pilot as "dynamic-exploit scored." So the DESIGN matches R3 (dynamic exploitability as ground truth).
- **BUT the live PS-1 lane is "static-gate / seccodeplt-curated-corpus (NO live model inference)."** The gap is NOT static-vs-dynamic scoring design — it is that **the runner has only ever been driven over a curated static corpus, never with a live model generating code in the loop.** The publishable SAFEDELTA (a model generating code, gate-ON vs OFF, the actual thesis number) needs `ELAI_E2E_POD_ENDPOINT` — a pod that has not been stood up. **THE GAP: runner exists, corpus exists, design is R3-faithful, but the number that requires a live model + a pod has never been produced. PS-1 has been RUNNER_PRESENT/GATED since the 2026-07-05 ruling ("headline number NOT yet produced") and remains so 12 days later.** This is the single most consequential gap in the entire trust-spine effort: the number the whole GTM stakes on ("believe-the-number") is the one number still un-produced, and it is gated on standing up a GPU pod, not on any design question.

### THE PROOF-SPINE PICTURE, PLAINLY

The proof spine is the research's own logic turned into law: "no claim without a measured number." The two numbers ELAI has produced (PS-5 case-reuse 0.95@1, PS-6 wire-dark 7.69%) are the two that need no model — real, journaled, honestly-caveated receipts. **The four numbers that need a model are all un-produced, and PS-1 (the safety delta, the thesis keystone) is gated on a pod endpoint.** So a year of trust-spine building has produced a shipped, wired, honest trust PRODUCT (RRA, attestation, cockpit, claims-lint, the runner) and TWO of the six credibility numbers — but not the one number the whole "believe-the-number" wedge was designed around. R1 said the benchmark doesn't close the sale; R3 said at 200 tasks it may not be defensible; R4 said even a credible number is counter-marketable — and meanwhile the number itself sits un-produced behind a pod gate. The proof spine's own PSR-1 precedence and the ruling's "FIRST THREE MOVES" (run PS-6, then N-1 substrate, then the PS-owner benches) put PS-1 behind cheaper wins — a defensible sequencing, but it means the thesis-critical number keeps being out-ranked by cheaper numbers.

### PROOF-SPINE NET

- R3 → PS-1 (safety delta): designed faithfully, runner shipped, **number GATED on a pod, never produced live** — the deepest gap in the effort.
- R5 → G-5 DSR, but **R5's headline (2-5× verification throughput) has NO proof-spine slot** — the load-bearing stream's own decisive number is off the spine.
- R1/R2/R4 feed no PS number directly (they are market/infra/strategy, correctly not numbers).
- 2/6 MEASURED, and both measured numbers are the model-free ones (PS-5, PS-6). Every model-dependent number is un-produced.
- Cross-check 1 CONFIRMED: "40-45% wire-dark" is stale (post-stack fn-grain = 7.69%, and the units aren't comparable); the improvement plan §9's reliance on it is the one inflated dependency.
- Cross-check 2 CONFIRMED: R3's design IS what the runner implements; the gap is a LIVE MODEL + POD, not a design flaw. The thesis number is un-produced for want of a GPU endpoint, not for want of a design.

---

## 12. CROSS-CUTTING — contradictions, drift, verdict

### CONTRADICTIONS BETWEEN STREAMS (present both, name which has evidence)

1. **Is the trust spine DEMANDED?** R1 says yes-but-not-as-a-category (demand flows into AppSec/review/supply-chain budgets); R4 says yes-and-the-specific-territory-is-unowned (off-agent replay-and-attestation). These AGREE, but the distilled bundle §2 tempers R4: the "unowned" finding is an INFERENCE from public surfaces, and Endor + GitHub are one product-decision from closing it. **Evidence weight**: R4's vendor scan is thorough and named; the "moat is real" claim is well-evidenced FOR TODAY but explicitly a first-mover WINDOW, not a durable moat. The honest synthesis: demand is real and routed through existing budgets (R1, strong), the position is currently unowned (R4, strong-but-inferred-and-time-limited).

2. **Does the NUMBER close the sale?** The commission stakes everything on "believe-the-number." R1 refutes it directly (benchmarks are top-of-funnel, don't close; the closing artifact is a real validated finding + inspectable evidence pack). R3 compounds it (at 200 tasks the number may be statistically undetectable). R4 compounds again (a credible number is counter-marketable by incumbents with existing benchmarks). The improvement plan §1.2/R4 resolve it: **the number serves the neutral-notary PRODUCT; it is not the moat.** **Evidence weight**: three independent streams (R1 market data, R3 statistics, R4 competitive) all say the number is necessary-not-sufficient. The commission's "everything gates on ONE number" framing is the loser here — it over-weights the number. The number should be produced (it is table-stakes evidence) but the GTM should never rest on it alone.

3. **Is verification durably HUMAN, or will models automate it away?** R5 says durably human — grounded in Anthropic's own 0–20% delegation ceiling at 80%-Claude-authored, Linux DCO, Sonar 47%. But R5's own hardest ceiling (20–40% of patches semantically wrong DESPITE passing automation) cuts both ways: it PROVES the human is needed (no gate catches everything) AND bounds what ELAI can promise (the gate reduces load, never eliminates the human). **Evidence weight**: R5 is the best-evidenced stream on this (frontier-lab internal data + an RCT + multiple 2026 studies). The verifier role is durable; the honest product claim is "we shrink the review surface," never "trust the number and stop reviewing."

4. **The cloud-lane tension (a contradiction the streams don't notice but the sweep surfaces).** R2 proves the cloud lane is structurally limited to trace-authentication (Anthropic doesn't expose replay semantics), while the local Qwen lane is the only one that can offer full recomputation. But memory (focus_rescope) says the WEDGE is "trust-evidenced autonomous execution on CLOUD Sonnet." So the strongest TRUST story (full recompute) is on local, and the GTM lane (cloud) is permanently weaker on reproducibility. R4's five-primitive "off-agent clean-room replay" is the one primitive the cloud wedge can't fully deliver. **Evidence weight**: R2 is definitive on vendor determinism; the tension is real. Resolution: the improvement plan §8 already handles it ("trace authentication + signed evidence" on cloud = YES; "deterministic replay" on cloud = NO). The claim discipline dissolves the contradiction by never over-claiming the cloud lane. But it means the reproducibility PROOF is strongest exactly where the GTM isn't.

5. **METR's population vs the wedge population.** The load-bearing METR RCT (19% slower, verification is the bottleneck) was measured on EXPERIENCED devs in THEIR OWN MATURE repos. R1's wedge ICP is vibe-cleanup agencies working on NOVICE-generated THROWAWAY prototypes. These are near-opposite populations. **Evidence weight**: METR is the only RCT; its result does NOT measure the wedge population. The bottleneck-is-verification claim generalizes to the wedge by ARGUMENT (untrusted AI code needs verification regardless of who wrote it), not by measurement. This is a real gap the research does not close — flagged.

### STALE F:\ PATHS — all sources are content-clean

The research files (R1-R5, plan summaries, improvement plan, lenses) contain NO stale `F:\.ELAI_workflow` paths — they are prose/markdown with no absolute paths. The stale-path risk lives in the CLAUDE.md/agent-preamble layer (the environment), not in these sources. The live repo root is `/home/user/teikoku/elai`; the sources reference plan units by letter-number (G-2, F-4) which resolve correctly against the live corpus. Two internal-consistency notes worth carrying: (a) the improvement plan header says "Grok output is pasted in the session transcript" — that transcript is NOT in this directory, so the raw Grok adversarial pass is unrecoverable from these files; (b) the F/G/H summaries' schema versions (207-224) are stale against live (git 494ef36), but that is expected snapshot drift, not an error.

### THE STRONGEST EVIDENCED FINDING ACROSS THE SET

**The off-agent-evidence trust doctrine is independently reconstructed by the supply-chain standards literature (R2), the doctrine ELAI states (00-PROMPT line 15) is what the standards ecosystem already believes — AND the specific product territory it implies (off-agent replay-and-attestation of a finished agent diff) is, per an exhaustive 2025-2026 vendor scan (R4), unowned by any incumbent, cross-confirmed by three independent "works≠trustworthy" telemetry sources (SUSVIBES 61%-correct/10.5%-secure on Claude 4 Sonnet, Endor Agent Security League, GitGuardian ~2x secret leakage).** The thesis is real: the doctrine is sound, the gap is measured, the position is (currently) open. ELAI's G-track is a near-exact match to R4's independently-derived five-primitive defensible shape.

### THE MOST DAMAGING REFUTATION THE RESEARCH PRODUCED

**Two refutations tie, and they compound:**
1. **R1/R3/R4 in concert: the "believe-the-number" GTM is wrong-shaped.** R1: benchmarks don't close (a real validated finding does). R3: at 200 tasks a small effect is statistically undetectable (need 300-400 git-stripped tasks; a 2-3pp claim on 200 is "dead on arrival"). R4: even a credible number is counter-marketable by incumbents who already have benchmarks. The commission's own stated wedge ("everything gates on ONE SAFEDELTA number") is the thing three streams independently refute. The number is table-stakes evidence for the neutral-notary product, never the moat.
2. **R5's hard ceiling: 20-40% of patches are semantically wrong DESPITE passing automated validation** (PVBench/OpenSSF 630-patch review). No gate — including ELAI's — eliminates the human. This bounds every trust-gate promise: the product SHRINKS the review surface, it cannot close it. Sold honestly ("we make verifying fast and defensible") this is a strength; any drift toward "trust the number, stop reviewing" is refuted by the research directly.

And the operational sting: **as of 2026-07-17, the SAFEDELTA number the whole commission is about has never been produced** — PS-1 is GATED on a pod endpoint, runner-present since the 2026-07-05 ruling. A year of faithful trust-PRODUCT building (RRA, attestation, cockpit, claims-lint, runner — all shipped and wired) has produced 2 of 6 credibility numbers, and both are the model-free ones. The thesis-critical number sits behind a GPU pod, out-ranked by cheaper wins in the proof-spine sequencing. **The research told ELAI to falsify-first (Phase 0 before Phase 1); ELAI built Phase 1 and never ran the two falsifiers (F0.1 the live number, F0.3 the agency WTP calls) that gate the thesis.**

### FINAL VERDICT ON THE SET

This is a disciplined, self-aware research set. Its highest-value outputs: (1) R4's unowned-neutral-notary finding + the improvement plan's §1.1 agent-agnostic-verifier pivot that claims it while dissolving both root risks (the single best idea, now a committed spec direction); (2) the improvement plan's §8 claim-discipline converting every refutation into a marketing guardrail (now the shipped K-1 claims-lint gate); (3) the honest metabolization of refutations rather than sanitizing them. Its blind spots: the METR-population-vs-wedge-population gap (unaddressed), the cloud-lane-is-weakest-where-the-GTM-is tension (handled by claim discipline but not resolved strategically), and R5's own 2-5× verification-throughput number having no proof-spine slot. Its one inflated dependency: the "40-45% wire-dark" figure (§9), now measured at 7.69% fn-grain and stale. The trust spine is real, mostly-built, and mostly-wired — but the thesis it stakes on ONE number is refuted by the research's own demand/statistics/competitive streams, and that one number is, a year on, still un-produced behind a pod gate.

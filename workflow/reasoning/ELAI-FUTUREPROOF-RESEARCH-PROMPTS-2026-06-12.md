ELAI future-proofing: 3 deep research prompts for Perplexity
date: 2026-06-12
rules applied: external-prompt anonymity (no project name, no codenames, no paths, neutral
vocabulary), attack-the-design section in each, ask for sources/dates on every claim.
usage: paste each prompt as-is into Perplexity deep research. they are independent.

================================================================================
PROMPT 1 — WHERE THE AGENTIC CODING MARKET IS GOING (2026-2028)
================================================================================

I'm researching the strategic direction of the AI coding agent / autonomous software
engineering market as of mid-2026, looking 12-24 months ahead. Please produce a deep,
source-dated report covering:

1. CURRENT LANDSCAPE AND TRAJECTORIES. For each major player, what they shipped in the last
12 months, their stated roadmap, pricing model, and apparent strategic bet: Cursor
(Anysphere), Windsurf, Cognition (Devin), Factory.ai, Anthropic Claude Code, OpenAI Codex,
Google (Antigravity, Jules, Gemini CLI), GitHub Copilot (Workspace, coding agent), Sourcegraph
Amp, Cline, OpenHands, Aider, Zed, JetBrains AI, Replit Agent, Lovable/Bolt/v0 (the
app-generator tier), and any significant new entrant from the last 6 months.

2. CONVERGENT PATTERNS. What is becoming table stakes across all of them: background/parallel
agents, agent fleets, cloud execution sandboxes, PR-native workflows, planning modes, memory,
MCP/tool ecosystems, computer use, voice, mobile? What interaction paradigm is winning:
IDE-embedded vs terminal CLI vs cloud-queued (fire-and-forget + review) vs canvas/visual? What
evidence exists about which paradigm developers actually retain after the hype (retention,
revenue, survey data)?

3. THE REVIEW BOTTLENECK. As agents write more code, multiple reports say human review becomes
the bottleneck. What are vendors doing about verifying/reviewing agent output at scale? Who is
building trust-the-output features (test evidence, provenance, audit trails, replay,
deterministic verification) vs just better diff UX?

4. CONSOLIDATION AND PLATFORM ABSORPTION. Which capabilities that startups built in 2024-2025
(context management, repo indexing/RAG, prompt caching strategies, multi-agent orchestration,
memory layers) have since been absorbed natively by the model providers (Anthropic, OpenAI,
Google) into their APIs/platforms? What does that history predict will be absorbed next?
Which acquisitions/shutdowns happened in this category in the last 12 months and why?

5. ENTERPRISE VS PROSUMER. Where is the actual revenue: individual subscriptions, team seats,
enterprise contracts, usage-based? What do enterprise buyers demand that prosumer products
lack (compliance, audit, SSO, on-prem, data retention guarantees)?

6. ATTACK THE DESIGN. Steelman the bear case for an independent (non-lab-owned) agent
orchestration product in 2027: if frontier models keep absorbing scaffolding and the labs ship
their own agents with distribution advantages, what is left for independents? Name the
specific failure modes of independent agent companies so far (with examples), and what the
survivors have in common.

Cite primary sources (changelogs, official blogs, funding announcements, usage reports,
developer surveys) with dates. Flag anything that is speculation vs documented.

================================================================================
PROMPT 2 — VERIFICATION AND TRUST OF AI-GENERATED CODE AS A PRODUCT CATEGORY
================================================================================

I'm researching whether "verification / trust / assurance of AI-agent-generated code" is
becoming a real product category with real buyers, as of mid-2026. Context: AI coding agents
now produce a large fraction of merged code; several public incidents involve agents
fabricating test results, deleting data, introducing vulnerabilities, or claiming success
falsely. Please produce a deep, source-dated report covering:

1. THE PROBLEM, QUANTIFIED. What is the current measured evidence on AI-generated code
quality: defect rates, vulnerability rates (e.g. studies on insecure code generation),
hallucinated dependencies/slopsquatting, fabricated or trivially-passing tests, regression
rates from agent edits? Which studies are most recent and credible? What do post-incident
analyses of public agent failures (data loss, faked results, security regressions) show about
root causes?

2. WHO IS BUILDING VERIFICATION. Map the landscape of tools/products that verify, gate, or
attest AI-generated code rather than generate it: AI code review tools (which ones go beyond
LLM-opinion review into deterministic evidence: tests, mutation testing, static analysis,
runtime checks), provenance/attestation tooling (in-toto, SLSA, Sigstore applied to AI
output), sandbox/policy enforcement for agents, agent observability/audit platforms, and any
product offering reproducible/replayable agent runs or cryptographic evidence of what an agent
did. Who funds them and who buys them?

3. REGULATORY AND PROCUREMENT DRIVERS. Status of the EU AI Act obligations relevant to
AI-generated content/code in 2026-2027 (Article 50 marking, GPAI code of practice,
enforcement timeline), plus US (executive orders, NIST AI RMF), SOC 2 / ISO 42001 adoption
for AI systems, software liability trends (EU Product Liability Directive covering software),
and cyber-insurance requirements. Are enterprises already writing "AI code provenance /
audit trail" requirements into procurement? Any documented examples?

4. THE SELF-GRADING CONFLICT. Will the model providers themselves ship trustworthy
verification of their own agents' output, or is there a structural conflict of interest that
leaves room for an independent verification layer? What have labs shipped here (e.g.
sandboxing, permissions, audit logs) and what have they conspicuously not shipped
(independent evidence, replayability, third-party attestation)?

5. ACADEMIC STATE OF THE ART. The strongest 2025-2026 research on: verifying agent
trajectories (not just final code), execution-grounded vs LLM-judge evaluation, reward
hacking / specification gaming by coding agents and defenses, benchmark contamination and
honest evaluation methodology, and formal-methods-lite approaches (property tests, mutation
scores, metamorphic relations) applied to AI-generated code at scale.

6. WILLINGNESS TO PAY + ATTACK THE DESIGN. Is there any evidence of willingness to pay for a
standalone "trust layer for AI code" (pricing of incumbents, case studies, analyst sizing)?
Then steelman the bear case: verification as a feature not a product (absorbed into CI
vendors, GitHub, or the labs), the false-positive friction problem that killed earlier
security gates, the "developers don't pay for safety" objection, and the scenario where
models get good enough that verification demand stays niche. What would have to be true for
the trust layer to be a venture-scale business vs a feature?

Cite primary sources with dates; separate measured findings from vendor marketing.

================================================================================
PROMPT 3 — WHAT SURVIVES MODEL IMPROVEMENT: THE MOAT AND ECONOMICS QUESTION
================================================================================

I'm evaluating the long-term durability of capabilities built in the orchestration/harness
layer around LLMs for software engineering, as of mid-2026. The core question: as frontier
models and their platforms improve, which harness-layer investments get commoditized and
which compound in value? Please produce a deep, source-dated report covering:

1. THE ABSORPTION RECORD. Document the last 24 months of harness capabilities absorbed into
model APIs/platforms: prompt caching, context compaction/summarization, server-side memory,
native sub-agents/orchestration, built-in code execution and computer use, tool
search/deferred tool loading, structured outputs, extended thinking with effort controls,
batch pricing. For each: who built it externally first, when the provider absorbed it, and
what happened to the external implementations.

2. THE "BITTER LESSON" AT THE HARNESS LAYER. The strongest current arguments and evidence on
both sides: (a) scaffolding is temporary — capability migrates into the model (cite cases
where agent scaffolds got simpler as models improved, e.g. minimal-scaffold agents matching
complex ones on benchmarks); (b) scaffolding is durable — verification, domain state,
environment integration, and trust requirements grow with autonomy (cite cases where
harness-layer value persisted). Which specific layers does the evidence say are durable:
deterministic verification gates, repo-specific knowledge graphs, organizational
memory/playbooks, audit/provenance, environment/sandbox management, cost routing?

3. OPEN-WEIGHT TRAJECTORY AND THE LOCAL LANE. Current gap between best open-weight coding
models (Qwen3 family, DeepSeek, GLM, Kimi, Llama, Mistral et al.) and frontier closed models
on real agentic coding (not just HumanEval) — with dates and benchmark caveats. Cost per
solved task trends, both API and self-hosted. Is hybrid local+cloud routing (cheap local for
mechanical edits, frontier for planning) actually used in production anywhere, with measured
savings? What hardware floor (VRAM) does a useful local coding lane require in 2026, and
where is that going?

4. TOKEN ECONOMICS. Direction of inference pricing 2024-2026 (per-token price deflation vs
rising consumption per task), the output-token premium, caching/batch discounts, and what
this implies for where cost optimization still matters in 2027. Are agent products
losing money per seat at current pricing (evidence from vendor pricing changes, rate
limits, reports of negative margins)?

5. WHERE DURABLE VALUE ACCRUES. Synthesize: for an independent product wrapping third-party
frontier models, rank the candidate moats by evidence — (a) proprietary evaluation/verification
infrastructure, (b) per-customer compounding state (codebase graphs, learned playbooks,
decision journals), (c) enterprise trust/compliance posture, (d) workflow lock-in/distribution,
(e) cost-engineering. Which ones have historically held when the model layer shifted?

6. ATTACK THE DESIGN. Steelman the bear case against a system whose bets are: deterministic
orchestration around the model, heavy verification gates on every step, reproducible/seeded
runs, and a hybrid local+cloud lane. Specifically: does heavy gating tax throughput so much
that lighter-touch competitors win on perceived productivity; do deterministic-replay
guarantees survive contact with inherently non-deterministic cloud APIs; does the local lane
stay relevant if frontier API prices keep falling; and does per-step verification become
redundant if models reach near-zero error rates on routine edits? What measured evidence
exists for each side?

Cite primary sources with dates; flag speculation; prefer measured numbers over claims.

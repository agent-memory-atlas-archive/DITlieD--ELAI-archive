DIGEST: frontier-models
SOURCE: cfa9415c-d0ed-49a3-b6b1-445d1a2b0bc4.txt

TOPIC: Frontier OpenAI vs Anthropic models (as of June 2026) for a DELEGATED reasoning engine behind a primary coding agent. Compares GPT-5.5 / GPT-5.5 Pro vs Claude Opus 4.8; answers model choice, context windows, economics, latency/reliability, API-vs-browser, routing policy.

REPORT VERDICTS:
- Do NOT pick a single universal winner — use a dual-vendor escalation pool, route by task shape.
- Claude Opus 4.8 = better default for repo-scale SWE, multi-file bug localization, long-running agentic coding, very-large-context synthesis (leads SWE-Bench Pro; 1M ctx at standard pricing; cheaper output than GPT-5.5).
- GPT-5.5 = stronger for terminal-heavy / tool-heavy / benchmarked 10k-100k-token long-context reasoning (strong Terminal-Bench 2.0; leads AA-LCR).
- If forced to ONE: Claude Opus 4.8 via API (strengths align w/ most expensive coding-agent failure modes); but if workload is terminal-first + smaller-context + reasoning-dense, GPT-5.5.
- GPT-5.5 Pro = reserve for narrow high-stakes adjudication / final-pass synthesis only (token prices dramatically higher); too expensive as default worker.
- For production delegation behind an automated agent: use vendor APIs NOT browser subscriptions (browser plans no stable per-token pricing, no model-level latency contracts, OpenAI "unlimited" browser explicitly bans programmatic extraction/resale; Anthropic Max is a user plan w/ usage limits not M2M backend).
- Use vendor redundancy w/ automatic failover (fail over API-to-API, not to browser).

CANDIDATES (name | url-or-id | report's claim | role):
OpenAI GPT-5.5 (API) | developers.openai.com/api/docs/models/gpt-5.5 | newest flagship for coding/professional; 1M ctx; $5 in / $30 out per 1M; prompts >272K input billed 2x input + 1.5x output for full session | GPT-5.5 terminal/tool/moderate-context reasoning offload
OpenAI GPT-5.5 Pro (API) | openai.com/index/introducing-gpt-5-5 | Pro variant 1M ctx; $30 in / $180 out per 1M; same >272K multiplier from higher base | rare final adjudicator only
Anthropic Claude Opus 4.8 (API) | platform.claude.com/.../models/overview | most capable GA model for complex reasoning + agentic coding; 1M ctx, 128K max output; $5 in / $25 out; 1M on Claude API/Bedrock/Vertex (200K on Microsoft Foundry at launch) | repo-scale coding / multi-file bug / giant-context default
Anthropic Claude Opus 4.8 Fast | anthropic Opus 4.8 launch | premium latency option research preview; $10 in / $50 out; full ctx window | latency-sensitive
ChatGPT browser (GPT-5.5 / Pro tiers) | help.openai.com 11909943 | GPT-5.5 default for logged-in; Pro on higher tiers; plan-specific ctx limits materially smaller than API; ChatGPT Pro now $100 + $200 tiers | human-supervised overflow only
Claude Max plan | claude.com / status | starts $100/mo, 5x or 20x Pro usage, recommended for Claude Code & Cowork, priority access; no token-metered browser price or 1M-equiv browser ctx published | human-supervised overflow only
Artificial Analysis Intelligence Index | artificialanalysis.ai/.../intelligence-index | external composite across reasoning/coding/tool-use/knowledge/long-context/science | leaderboard ground truth
Artificial Analysis AA-LCR | artificialanalysis.ai/.../long-context-reasoning | long-context reasoning ~10k-100k tokens; GPT-5.5 leads | benchmark
Humanity's Last Exam | agi.safe.ai / AA leaderboard | frontier multi-step reasoning across expert domains | benchmark
SWE-Bench Pro | (Anthropic official) | real-world software issue resolution in repos | benchmark (Opus lead)
Terminal-Bench 2.0 | (OpenAI official) | terminal-driven coding/CLI work | benchmark (GPT-5.5 strong)
Online-Mind2Web | (Anthropic launch) | browser-agent execution + multistep web | benchmark (Opus 84%)
OpenRouter benchmark mirrors | openrouter.ai/openai/gpt-5.5, openrouter.ai/anthropic/claude-opus-4.8 | public p50 latency/throughput mirrors | latency evidence

RECOMMENDED OPERATING MODEL (tiered delegation): primary agent keeps a fast cheap inline model for routine work, routes a minority into a frontier pool; within pool use Opus 4.8 + GPT-5.5 as specialists.
Subtask routing: unit-test-gen-for-localized-changes = inline; short summarization/changelog = inline; symbolic reasoning/algorithm design/small-ctx hard = GPT-5.5 (Pro only for rare adjudication); multi-file bug/PR review/repo-scale patch = Opus 4.8; terminal/CI-CD/shell-first = GPT-5.5; large-repo search = search inline + frontier synthesis only; long-form synthesis = Opus 4.8 if >~272K input else GPT-5.5 if 10k-100k; RAG-w/-citations = Opus when output long + packs huge, GPT-5.5 when pack smaller + reasoning bottleneck; manual overflow = browser pool human-supervised.

ATTACK-THE-DESIGN (report's caveats / criticisms of the comparison and of browser-backend use):
- Browser plans (ChatGPT Pro / Claude Max) are INAPPROPRIATE as a hidden machine backend: OpenAI explicitly prohibits automatically/programmatically extracting data, sharing credentials, reselling access; plan-specific contexts materially smaller than API; no token-metered cost, no published median/p95 latency.
- Methodological gap: Anthropic's Opus 4.8 benchmark detail is embedded in an IMAGE/system-card, much not text-exposed; report had to use public leaderboards or mark values "unspecified" -> some head-to-head rows are not apples-to-apples (Terminal-Bench 2.0 official Opus row not text-exposed; Online-Mind2Web GPT-5.5 row not text-exposed).
- Model-specific p50/p95 latency NOT published by either vendor officially; OpenAI status metrics are aggregate across all tiers/models/error-types; latency evidence is directional from OpenRouter/Artificial Analysis only.
- GPT-5.5 >272K-input long-context multiplier applies to the ENTIRE session (changes system design), Opus keeps full 1M at standard pricing — a routing-policy-changing asymmetry not a footnote.
- A June 5 2026 incident specifically affected Opus 4.8; recent incidents mention degraded Claude Code; argues for vendor failover.

NUMBERS (figure | claimed source):
- AA Intelligence Index: GPT-5.5 60.2 vs Opus 4.8 61.4 (Opus leads narrowly).
- Humanity's Last Exam: GPT-5.5 44.3% vs Opus 4.8 45.7% (Opus edge).
- SWE-Bench Pro: GPT-5.5 58.6% vs Opus 4.8 69.2% (Opus clear lead, official).
- Terminal-Bench 2.0: GPT-5.5 82.7% (OpenAI official); GPT-5.5 reaches 83.4% w/ Codex CLI harness (Anthropic footnote); official Opus row not text-exposed.
- AA-LCR: GPT-5.5 74.3% vs Opus 4.8 67.7% (GPT-5.5 leads).
- Online-Mind2Web: Opus 4.8 84% (Anthropic); GPT-5.5 not text-exposed.
- Pricing per 1M tokens: GPT-5.5 $5/$30; GPT-5.5 Pro $30/$180; Opus 4.8 $5/$25; Opus 4.8 Fast $10/$50. GPT >272K input = 2x input + 1.5x output for full session; Opus full 1M at standard pricing.
- Example task cost (N input + 0.2N output): 1K/10K/100K-token tasks: GPT-5.5 $0.011/$0.11/$1.10; GPT-5.5 Pro $0.066/$0.66/$6.60; Opus 4.8 $0.010/$0.10/$1.00; Opus Fast $0.020/$0.20/$2.00. 300K-input/60K-output session: ~$5.70 GPT-5.5 vs ~$3.00 Opus 4.8.
- Latency (OpenRouter p50): GPT-5.5 1.82s latency / 41 tok/s; Opus 4.8 1.32s / 55 tok/s. Artificial Analysis: GPT-5.5 xhigh 63.6 tok/s w/ 85.83s TTFT (methodology-sensitive); Opus 4.8 ~65 tok/s.
- Uptime: OpenAI API 99.98% (aggregate, Mar-Jun 2026 window); ChatGPT 99.83%. Anthropic Claude API 99.15% (90d), Console 99.33%, claude.ai 98.8%, Claude Code 99.0%.
- Opus 4.8: 128K max output tokens.

UNVERIFIED/FLAGGED by report: example task costs are the report author's own arithmetic from official prices; latency figures are directional (third-party mirrors), not contractual; several Opus 4.8 benchmark rows sourced from public mirrors because official Anthropic text didn't expose them; browser "cost per task" cannot be computed from public official sources. (Note: model IDs/pricing here are as the REPORT claims for June-2026 hypothetical models GPT-5.5/Opus 4.8 — transcribed, not independently verified.)

# Developer discourse 2026: what they actually say about AI coding tools

## TL;DR
- Sentiment is bimodal: Stack Overflow 2025 says only 29% of developers trust AI tool output (down from 70%+ in 2023), yet JetBrains' Jan 2026 AI Pulse reports 90% regular use and Claude Code at 91% CSAT / NPS 54. Trust is collapsing while dependence climbs.
- METR's RCT (16 senior OSS devs, 246 tasks, Feb-Jun 2025) is the load-bearing skeptic cite: devs predicted 24% speedup, felt 20% speedup, were actually 19% SLOWER with Cursor Pro + Sonnet 3.5/3.7. Builders who ignored this got mocked; builders who built around it (eval loops, scoped review) shipped.
- The 2025 inflection (Opus 4.5 + GPT-5.1 in November) flipped a subset of senior eng critics — Willison, Birchler, Ronacher, Yegge, Karpathy — into heavy users. "Hand-coded percentage in single digits" is now a normal flex. Skeptics (Torvalds, marginalia_nu, Acemoglu cite) didn't move.
- The real complaint isn't "AI can't code" anymore — it's review burden, context loss without warning, pricing whiplash (Cursor June 2025), vendor lock-in, prompt-injection (CVSS 9.8 Cursor IDE CVE), and "agent-generated mess only AI can decode." Net direction: net-positive on capability, net-negative on trust/control.
- What devs say they actually WANT: orchestration of multiple bounded workflows (not full autonomy), terminal/CLI tools over MCP, deterministic checks around the model, transparent context state, no surprise pricing, and a local fallback. ELAI's bet (Rust harness + local-first hybrid + grounded reasoning + FSM gates) maps directly onto these unmet wants.

## Key sources

| URL | who said it | date | takeaway |
|---|---|---|---|
| https://arxiv.org/abs/2507.09089 | METR (Joel Becker et al.) | Jul 2025 | RCT: 19% slower with AI; devs thought 20% faster. Gold-standard methodology. |
| https://news.ycombinator.com/item?id=43683012 | HN thread + ModernMech | Apr 2025 | "Building on AI seems more like building on a foundation of sand, or building in a swamp." |
| https://news.ycombinator.com/item?id=47936579 | HN "Is Claude Code getting worse" + idempotent_, tstrimple | early 2026 | Multiple users report task regression, time limits, context degradation post-upgrade. |
| https://news.ycombinator.com/item?id=45242788 | HN "AI babysitters" thread + jitl, dimator, askonomm | Sep 2025 | "Reviewing and fixing AI code made me incredibly bored. I want to actually build things, not manage AI." |
| https://news.ycombinator.com/item?id=46352875 | HN on Ronacher "Year of Vibes" + bgwalter, shimman | Dec 2025 | "2025 was a lost year for programming"; "Where is the resulting software?" |
| https://news.ycombinator.com/item?id=46676554 | HN "Top 0.01% Cursor user switched to CC 2.0" + tuckwat, worksonmine | early 2026 | "Doesn't work in a multi-dev environment with paying customers"; tests can pass on hidden malware. |
| https://lucumr.pocoo.org/2025/06/12/agentic-coding/ | Armin Ronacher | Jun 12 2025 | CLI tools > MCP; explicit caching; use foundational SDKs not Vercel AI SDK. |
| https://lucumr.pocoo.org/2025/11/21/agents-are-hard/ | Armin Ronacher | Nov 21 2025 | "SDK abstractions break once you hit real tool use"; "testing and evals the hardest problem here." |
| https://lucumr.pocoo.org/2025/12/22/a-year-of-vibes/ | Armin Ronacher | Dec 22 2025 | "I quite vocally shared my lack of success with MCP throughout the year." Claude Code "state of the art." |
| https://simonwillison.net/2026/Jan/8/llm-predictions-for-2026/ | Simon Willison | Jan 8 2026 | Hand-coded % "single digit"; warns of "Challenger disaster" prompt-injection worm. |
| https://fly.io/blog/youre-all-nuts/ | Thomas Ptacek (fly.io) | Jun 2025 | "All progress on LLMs could halt today, and LLMs would remain the 2nd most important thing." "Skills issue!" to skeptics. |
| https://birchtree.me/blog/the-data-has-changed/ | Matt Birchler | Feb 2026 | Sonnet 3.5 in Cursor was 40% pass on tiny tasks → Opus 4.6 in CC 90% on entire features. Curve moved. |
| https://blog.cloudflare.com/ai-code-review/ | Cloudflare eng | early 2026 | 7 specialised reviewers + coordinator; "telling an LLM what NOT to do is where the value resides"; 0.6% break-glass. |
| https://blog.cloudflare.com/internal-ai-engineering-stack/ | Cloudflare eng | early 2026 | 3,683 internal AI-tool users; MRs/week climbed 5,600 → 8,700; MCP tool-defs consume too much context. |
| https://blog.jetbrains.com/research/2026/04/which-ai-coding-tools-do-developers-actually-use-at-work/ | JetBrains AI Pulse | Apr 2026 | 90% regular AI use; CC 18% adoption / 91% CSAT / NPS 54; Copilot 29% adoption; Cursor 18%; Antigravity 6%. |
| https://survey.stackoverflow.co/2025/ | Stack Overflow Survey | 2025 | 84% using/planning AI; only 29% trust output (down from 70%+ in 2023). |
| https://cognition.ai/blog/devin-annual-performance-review-2025 | Cognition | late 2025 | Devin PR merge rate 34% → 67%; admits "can't tackle ambiguous projects end-to-end like a senior." |
| https://www.itpro.com/.../devin-cognition-fumbling | IT Pro | 2025 | Answer.AI test: Devin succeeded on 3 of 20 tasks; "6 hours for Devin to fail" what dev did in 36 min. |
| https://nmn.gl/blog/dangers-vibe-coding | Namanyay Goel | Mar 2025 | "Technical debt compounds exponentially when you don't understand your code." API keys exposed in vibed auth code. |
| https://x.com/karpathy/status/1886192184808149383 | @karpathy | Feb 2025 | Coined "vibe coding" — "fully give in to the vibes, embrace exponentials, forget the code even exists." |
| https://x.com/ggerganov/status/2038632534414680223 | Georgi Gerganov | Mar 2026 | llama.cpp at 100k stars; jokes/predicts 90% of agents running locally on llama.cpp in 3-6 months. |
| https://x.com/mitsuhiko/status/1954643774203924832 | Armin Ronacher | Aug 2025 | "Claude Code feels like a tool used by the developers for the purpose it's written" (dogfooding tell). |
| https://www.fintechweekly.com/.../cursor-pricing-change-user-backlash-refund | FinTech Weekly | Jul 2025 | Cursor June 2025 silent pricing change: 500 fast req/mo → ~225 at $20; CEO public apology Jul 4. |
| https://github.com/anthropics/claude-code/issues/13171 | Claude Code GH issue | 2025 | "Context loss without warning breaks trust" — flagged as Critical UX. |
| https://github.com/anthropics/claude-code/issues/19567 | Claude Code GH issue | 2025 | "Hangs indefinitely during context compaction"; Ctrl+C fails; orphan process. |
| https://newsletter.pragmaticengineer.com/p/steve-yegge-on-ai-agents-and-the | Steve Yegge via Gergely Orosz | 2025 | Initially skeptical, converted by Claude Code; now 12K LOC/day claim; built Gas Town orchestrator. |
| https://x.com/karpathy (Dec 27 2025) | @karpathy | Dec 2025 | Posts feeling "10 times more powerful by correctly orchestrating new tools, otherwise a skill issue." |
| https://news.ycombinator.com/item?id=47460525 | HN OpenCode launch | Mar 2026 | 1,099 points; "finally a real open-source alternative" to CC/Codex lock-in; v1.2.20 silently sent prompts to Grok. |
| https://github.com/anomalyco/opencode | OpenCode repo | 2025-26 | 120k stars, 800 contributors, 5M monthly devs — open-source agent gravity is real. |

## Top complaints (ranked by frequency)

**1. Context loss / compaction without warning** — Anthropics issue #13171 ("Context loss without warning breaks trust"); issue #19567 ("Claude Code hangs indefinitely during context compaction... Ctrl+C does not work"). HN user rosenlykke (47959971): model "stops parsing context on simple tasks/stops thinking" inconsistently. Products: Claude Code (most cited), Cursor ("claims 200K but you're really getting 70-120K," HN comparison thread).

**2. Review burden inverts the productivity claim** — HN samgranieri (45242788): "I spend more time telling AI what to do than it would spend actually writing this all out myself?" jitl: "The worst is when I have to baby-sit someone else's AI...usually I want to insta reject just for the disrespect for my time." dimator: "2500 lines of slop... code review takes more effort than the code owner contributed." Products: all agents, but especially Devin and Cursor agent mode.

**3. Surprise pricing / silent caps** — Cursor June 2025: "500 fast responses per month" → ~$20 of API-priced usage = ~225 req. CEO apology Jul 4 2025, refund window. r/cursor migration to Windsurf. HN idempotent_ (47937655) on Claude Code: "super weird time limits." Products: Cursor (worst), Claude Code Max plan.

**4. Quality regression on upgrades** — HN tstrimple (47948542): nix-darwin setup succeeded 2 months prior, failed identically on Opus 4.7. Dolthub gotchas blog (Jun 2025): "thinking depth dropped 67%... full-file Write usage doubled — model increasingly chose to rewrite entire files rather than make surgical edits." Products: Claude Code Feb 2025 update (issue #42796), Opus 4.7 in early 2026.

**5. Hallucination with high confidence** — HN ModernMech (43683012): "It always sound[s] like an intelligent person who knows what they are talking about, even when spewing absolute garbage." HN ryandrake: "I expect perfection and consistency from a machine... I don't expect it to guess." Products: all, especially in long-context refactors.

**6. AI-generated tech debt the AI itself can't decode** — HN marginalia_nu (47494890): "Claude Code operating on unsupervised Claude code... generates a mess not even Claude Code can decode." nmn.gl: "Each 'vibed' solution becomes a black box, and these black boxes multiply." Products: any agent run in low-supervision loops.

**7. MCP overhead / disappointment** — Ronacher (Dec 2025): "I quite vocally shared my lack of success with MCP throughout the year." Cloudflare internal post: "MCP protocol has a practical problem where tool definitions consume context tokens at scale." Their fix: "Code Mode" — let model discover tools through code.

**8. Vendor lock-in / no model portability** — HN on Cursor 3 (47618084): "Vendor lock-in concerns... prefer a solution that could manage all AI agents rather than lock into one vendor." HN OpenCode launch driven by exactly this. Products: Claude Code Max not allowing OpenCode (HN idempotent_), Cursor.

**9. Security: prompt injection as production-grade attack** — CVE-2026-26268 Cursor RCE via malicious repo clone; CVE-2025-53773 GitHub Copilot prompt-injection RCE; Microsoft Copilot CVSS 9.3; Cursor IDE CVSS 9.8. Willison Jan 2026 prediction: "A Challenger disaster for coding agent security... a prompt injection worm."

**10. "AI ready" caveat invalidates most production codebases** — HN OP (46676554) admits behavior-driven validation only works on "AI ready repos" with good docs, tests, CI/CD, monitoring. tuckwat: "doesn't work in a multi-dev environment with paying customers."

## Top praises (where AI coding actually delivers)

**Unblocking procrastination on greenfield/well-scoped work** — HN matheusmoreira (47494890): "Claude helped me implement... features I'd been procrastinating for months in days." Birchler Feb 2026: Opus 4.6 in CC "gets it right 90% of the time" on "entire features."

**Migrations and well-defined repetitive transforms** — Cognition: Devin "10-14x faster than humans on specific tasks" like migrations; test coverage 50-60% → 80-90% on tasks where it has explicit upfront spec.

**Code explanation / debugging dialogue** — Aitooldiscovery review aggregating HN: "Claude Code is superior to Cursor and Copilot specifically for dialogue-based debugging, where you describe the error and the tool explains the reasoning, not just the fix." HN nzach: "use the LLM for... reading through code and explaining what it does."

**Drafting plans then implementing by hand** — HN ulrikrasmussen: "use Claude to develop an implementation plan... walk me through implementing it by hand." This is the workflow the METR study's slowest devs explicitly didn't have.

**Background work / always-leave-something-running** — Mitchell Hashimoto rules: "always have an agent doing something in the background"; "always leave the day with some work for the AI." Endorsed as a way to extract value while sidestepping flow-state cost.

**Cost of code review at scale** — HN poster on CC for code review: "$0.04 per code review using Opus 4.6 over February for 200+ PRs with two rounds each, totaling $19.50." Cloudflare's specialised-reviewer pattern.

**Productivity at platform companies that built scaffolding** — Cloudflare: internal MRs/week 5,600 → 8,700, 3,683 internal AI-tool users. Block engineering blog reports similar. Note: not vanilla tool use; required custom orchestration.

## "Stupid AI tricks" — what developers mock

- HN ModernMech: AI "like a vandal... confidently inserting random wrong code into my 100% correct code" and "hallucinated a missing brace (my code parsed fine)."
- HN jmaker: "All of them missed the point, some even introduced security vulnerabilities in the process invalidating JWT validation."
- nmn.gl: vibed auth code with API keys committed to repo.
- HN izabera (45242788): "Random developer opens PR without understanding existing systems, AI generates 2000 lines, wastes an hour in review explaining why the feature wasn't needed."
- WSJ-cited Dolores Park scene (per envisioning newsletter 133): people "babysitting their agents" in the park because closing the lid stops the work.
- Karpathy Dec 27 2025: developers must orchestrate or "it's a skill issue" — mocked back in HN threads as victim-blaming the user for tool limitations.
- HN ramon156 (46676554): mocks "the OpenCode incident" — v1.2.20 silently sent user prompts to Grok's free tier for "session title generation" even with local-only configs.
- HN comments on Devin: "6 hours for Devin to fail to do" what a dev did in 36 min (IT Pro citing Answer.AI). Internet of Bugs / Computer Vision Project channels documented failures.

## Local-LLM specific discourse

What r/LocalLLaMA + adjacent discourse runs as of May 2026 (compiled from search results since direct subreddit fetch was blocked):

- **Primary coders**: Qwen3-Coder-30B (Flash), Qwen3-Coder-480B (cloud-class), Qwen 3.6-27B (24GB+ VRAM), Qwen 3.6-35B-A3B (works with less). DeepSeek V4-Flash (Apr 23 2026) cited as Claude-Haiku-class with 1M context for whole-repo work.
- **Inference stack**: llama.cpp dominates — Gerganov's 100k-star tweet (Mar 2026) jokes 90% of agents will run locally on llama.cpp within 3-6 months. Unsloth docs the canonical "run Qwen3-Coder locally" reference.
- **Trigger for local migration**: Apr 4 2026 Anthropic third-party-harness policy change pushed OpenClaw users to per-token billing, triggering wave of "what's the local story now" posts.
- **Real complaints**: VRAM ceiling (32B is the practical Ollama limit on consumer); tool-calling reliability lags cloud; agentic loop latency multiplies with local inference; quantization quality cliff is real.
- **Real praise**: Privacy / no-token-billing; reproducibility; offline; "fast enough on a Lenovo ThinkStation PGX" (XDA report). One user: pointed Claude Code at local Qwen3-Coder-Next and called it "a local LLM I actually want to use for coding."
- **Implication for ELAI**: the Qwen2.5-Coder-7B-Instruct-abliterated + NF4 + SDPA sidecar on RTX 3090 (~1.92 GB VRAM, 25K context ceiling) is consistent with the median LocalLLaMA setup. The bigger differentiator vs LocalLLaMA discourse is the hybrid+cloud routing, not the local model itself.

## Sentiment trend curve

May 2024 → May 2026, grounded:

- **May 2024**: skeptic-majority on HN; Karpathy hasn't coined "vibe coding" yet (that's Feb 2025); Stack Overflow trust still in 70%+ band.
- **Early 2025**: METR study runs (Feb-Jun); Karpathy coins vibe coding (Feb); Ptacek "My AI Skeptic Friends Are All Nuts" (Jun) marks high-water polemic for true-believers. Sentiment is contested.
- **Jul 2025**: METR results publish (arxiv 2507.09089) — "19% slower" becomes the load-bearing skeptic cite for the rest of the year. Cursor June 2025 pricing apology compounds anti-vendor sentiment.
- **Sep-Nov 2025**: "AI babysitter" thread (HN 45242788, Sep); Fast Company "vibe coding hangover" (Sep); Ronacher "Agent Design Is Still Hard" (Nov); Claude Opus 4.5 + GPT-5.1 ship (late Nov) — this is Yegge's claimed "event horizon" (Nov 24).
- **Dec 2025 - Feb 2026**: Ronacher "Year of Vibes" (Dec 22) tone-shift to net-positive on Claude Code specifically; Birchler "The data has changed" (Feb 2026); Willison 2026 predictions (Jan); Karpathy "agentic engineering 99%" reframe (Feb).
- **Mar-Apr 2026**: OpenCode hits #1 on HN (Mar 20); JetBrains AI Pulse confirms 90% regular use, CC NPS 54; Cloudflare internal stack post; Karpathy autoresearch (Mar).
- **Curve direction**: capability sentiment monotonically net-positive May 2024 → May 2026, big inflection at Nov 2025. Trust sentiment is monotonically net-negative — Stack Overflow trust 70%+ (2023) → 29% (2025). Net-net the field has split into "ship-with-it" (Cloudflare, Ronacher, Birchler, Yegge, Willison) and "this-is-still-broken-and-getting-worse" (METR-citers, HN long-tail in 47936579, OSS maintainers per Acemoglu/vibe-coding-kills-open-source). ELAI's user (you) sits clearly in cohort 1 with cohort-2 hygiene.

## What developers say they WANT next (signals for ELAI)

- **Orchestration of bounded workflows over full autonomy** — HN consensus on 44623207 + developersdigest summary: "developers actually getting value from AI coding systems are usually orchestrating multiple bounded workflows rather than pursuing full autonomy." Maps directly to ELAI FSM gates + per-phase audit + memory-writer separation.
- **Deterministic checks around the model** — same source: "coherent orchestration with clear task boundaries, explicit handoffs, and deterministic checks around the model." ELAI's auditor / wiring-contract-validator / fsm_transition guard already does this; the discourse confirms it's a moat.
- **CLI tools over MCP** — Ronacher Jun 2025 + Cloudflare "Code Mode": models understand code writing better than MCP interactions; tool-defs consume too much context. ELAI's preference for CLI shell tools is well-aligned.
- **Transparent context state + sessions you can replay** — HN comment on Ronacher Dec post: "we need better / different UX or paradigms of sharing sessions." Anthropic issue #13171 demands warning before context loss. ELAI's session-handoff + memory-bank gives a head start.
- **Local fallback for cost / privacy / vendor risk** — OpenCode HN reception + ggerganov's joke + Apr 4 Anthropic policy migration wave. ELAI's local-first hybrid is what the market is moving toward, not away from.
- **Eval / test loops, not just code-gen** — Ronacher Nov 2025: "We find testing and evals to be the hardest problem here." ELAI's TDD-first + contract tests + per-level auditor lines up.
- **Multi-reviewer / specialised-reviewer pattern** — Cloudflare: 7 specialised reviewers (security / performance / quality / docs / release / compliance) + coordinator. Maps directly to ELAI's per-level auditor modes (B-41 shipped, per memory).
- **No surprise pricing / no silent caps** — Cursor June 2025 backlash. ELAI is self-hosted; this is a moat by construction.

## Anti-patterns explicitly called out by developers

- **Full autonomous agents giving big tasks then walking away** — HN 44623207: "frontier demos still focusing on autonomy... makes for good screenshots and dramatic launch copy" but doesn't deliver in production.
- **"Just review the tests, not the code"** — HN tuckwat (46676554): "doesn't work in a multi-dev environment with paying customers." worksonmine: hidden malware can pass tests.
- **Vibe coding on serious projects** — nmn.gl: "Vibe coding might work for a weekend project, but it's catastrophic for any serious software." Vibe Coding Kills Open Source paper (Jan 2026, per Wikipedia summary).
- **SDK abstractions (Vercel AI SDK, high-level wrappers) before field stabilises** — Ronacher Nov 2025: "SDK abstractions break once you hit real tool use"; "the web search tool from Anthropic routinely destroys the message history with the Vercel SDK."
- **Async-typing-style agent output without preserving failure context** — Ronacher Dec 2025 + simonw HN reaction: agents that throw away failed attempts force re-discovery; "session history—including dead ends—should be retained."
- **Hidden third-party calls** — OpenCode v1.2.20 silently sending prompts to Grok for session-title-gen; users mocked it on HN, fixed in v1.2.23.
- **Telling the LLM what to do without telling it what NOT to do** — Cloudflare: "telling an LLM what NOT to do is where the actual prompt engineering value resides." Naive prompts → "ten-plus speculative findings per review developers learn to ignore."
- **Running agents with broad permissions because nothing bad happened yet** — Willison Jan 2026: "Normalization of Deviance, noting people run agents with excessive permissions because failures haven't materialized yet." Argues for sandboxing as urgent.
- **One-mega-agent vs many specialised agents** — Cloudflare's 7-reviewer pattern beats their initial single-agent attempt that produced "a flood of vague suggestions, hallucinated syntax errors."

## Open questions

1. **Does ELAI's grounded-by-default + FSM-gated workflow measurably reduce the METR slowdown?** Measurable via: time-to-merged-PR with vs without ELAI on matched task pairs, on the .elai_cc codebase. If ELAI is just a fancier Cursor, it inherits 19% slowdown; if the FSM/auditor gates work, slowdown inverts.
2. **What's the prompt-injection blast radius of a multi-agent ELAI session?** Specifically: can a malicious file in the indexed corpus exfiltrate via the WebSearch tool path? Cloudflare's "strip boundary tags entirely" and Willison's worm prediction both demand a measured answer before any production user.
3. **Does ELAI's hybrid local+cloud router actually beat pure-cloud Claude Code on tokens-per-task at fixed quality?** Measurable on a fixed B-N replay set: ELAI hybrid vs Claude Code straight-cloud, same tasks. JetBrains data shows the market still picks pure-cloud despite cost — local must be cheaper AT comparable quality to win.
4. **Is the agent's context state legible to the user mid-run?** Anthropic issue #13171 says users need warning before context loss. ELAI dashboard claims to surface FSM + memory state — is this actually visible during sidecar inference, or only post-hoc?
5. **At what local-model size does ELAI's audit/critic loop break down?** Qwen2.5-Coder-7B-abliterated NF4 on 3090 is the current bet. If audit-mode requires Sonnet-class reasoning, the local-first claim is partly aspirational. Measurable: per-level auditor pass-rate on identical phases, 7B vs 32B vs cloud Sonnet 4.6.

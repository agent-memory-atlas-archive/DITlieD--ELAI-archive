# GitHub trending agentic / harness / LLM tooling (May 2026)

Snapshot date: 2026-05-19. 18 WebFetch/WebSearch calls. github.com/trending pulled daily/weekly/monthly + Rust/Python/TS slices + 5 awesome-* lists + leaderboard/news cross-checks.

## TL;DR
- The dominant 2026 pattern is **skills-as-data + harness** (SKILL.md open standard, Anthropic dec-2025, adopted by OpenAI Codex). Five of the top 20 fastest-growing repos this month have "skills" in the name (`mattpocock/skills` +72k, `addyosmani/agent-skills` +26k, `multica-ai/andrej-karpathy-skills` +80k, `K-Dense-AI/scientific-agent-skills`, `Imbad0202/academic-research-skills`). ELAI's `.claude/skills/` directory is already on this pattern — keep it and consider exporting skill manifests as a public surface.
- **Rust-native harness is real and crowded**: `farion1231/cc-switch` (74k, +27k/mo), `warpdotdev/warp` (59k, +33k/mo), `openai/codex` (84k, 96% Rust), `superradcompany/microsandbox` (libkrun microVM), `Omegon`, `majiayu000/harness`, `codejunkie99/agentic-harness`, `Dicklesworthstone/pi_agent_rust`. ELAI is not alone but the **FSM + decomposition + rules-as-data + grounded reasoning** combination is still differentiated — competitors lean either pure-orchestrator or pure-runtime.
- **Adopt now (steal)**: SKILL.md spec as a public export, MCP Streamable HTTP transport (replaces SSE, 28% of registered servers already), microVM sandboxing pattern (microsandbox/libkrun, sub-200ms boot), workflow-YAML hybrid (Archon's deterministic+agentic node mix, MIT, 21.6k stars), persistent agent memory (rohitg00/agentmemory, +9k/mo).
- **Ignore**: hype-trained "skills" repos that are pure marketing lists (most of the top-20 skill repos), Lovable-clone wave (commercial focus, no harness value), browser-use clones (vercel-labs/agent-browser is the only Rust-native one worth watching), self-evolving agent claims without measurable benchmarks (`NousResearch/hermes-agent` 156k stars, no SWE-bench number).
- **Competitive threats**: Archon (open-source harness builder, MIT, deterministic+AI nodes, 21.6k), ruflo (multi-agent swarm w/ federation, 52.8k), `coleam00/Archon` workflow YAML pattern. None ship Rust-native FSM + grounded reasoning — that's still ELAI's window.

## Trending repos (May 2026 snapshot)

| Repo | Stars | Lang | What it does | Why trending | Relevance to ELAI |
|---|---|---|---|---|---|
| [farion1231/cc-switch](https://github.com/farion1231/cc-switch) | 74.5k | Rust | Cross-platform desktop all-in-one for Claude Code / Codex / OpenCode / Gemini CLI / Hermes | +27k/mo | Direct competitor on "single client across N agents" UX. Tauri stack |
| [openai/codex](https://github.com/openai/codex) | 83.5k | Rust 96% | OpenAI's terminal coding agent | +7.7k/mo, GPT-5.5 backend | Reference for Rust CLI shape, sandboxing patterns |
| [anthropics/claude-code](https://github.com/anthropics/claude-code) | 125k | Shell/Py/TS | Anthropic's terminal harness | The benchmark | Same problem space. Plugin/skill/hook architecture is the de-facto standard ELAI must interop with |
| [warpdotdev/warp](https://github.com/warpdotdev/warp) | 59k | Rust | Agentic terminal | +33k/mo | Terminal-first harness; same Rust-native AI niche |
| [All-Hands-AI/OpenHands](https://github.com/All-Hands-AI/OpenHands) | 74k | Python | Multi-interface coding agent (SDK, CLI, GUI, cloud, k8s) | v1.7.0 May 1 2026, 102 releases | Closest "full harness" python competitor. OpenHands+CodeAct v3 on Opus 4.6 >68% SWE-bench Verified |
| [coleam00/Archon](https://github.com/coleam00/Archon) | 21.6k | TS/Bun | "First open-source harness builder for AI coding" | +3.1k/mo, MIT | Workflow-YAML pattern. 17 bundled workflows. Direct conceptual competitor |
| [ruvnet/ruflo](https://github.com/ruvnet/ruflo) | 52.8k | TS 87% / Rust | Multi-agent swarm orchestrator on Claude, 100+ specialized agents, AgentDB+HNSW | +20k/mo, MIT | Federation+vector memory pattern worth studying |
| [superradcompany/microsandbox](https://github.com/superradcompany/microsandbox) | 6.1k | Rust | libkrun microVM hardware-isolated agent sandboxes, <200ms boot | YC X26 | Replace any docker-based exec in ELAI with this |
| [SWE-agent/SWE-ReX](https://github.com/SWE-agent/SWE-ReX) | n/a | Py | "Spawn shell, run command, get output" uniform API across Local/Docker/Modal/Fargate/Daytona | Powers SWE-agent | Sandbox-execution abstraction worth borrowing |
| [rohitg00/agentmemory](https://github.com/rohitg00/agentmemory) | 12.8k | TS | Persistent memory for coding agents | +9.2k/mo | Memory pattern comparable to ELAI's memory-bank + RAG |
| [zilliztech/claude-context](https://github.com/zilliztech/claude-context) | 11.4k | TS | Code search MCP for Claude Code | +5.4k/mo | Direct competitor to ELAI's `.claude/tools/elai.map` + SCIP indexing |
| [mksglu/context-mode](https://github.com/mksglu/context-mode) | 15.1k | TS | Sandboxes tool output, "98% context reduction" | +7.6k/mo | Output-compression pattern; relevant to ELAI's anti-read-big rule |
| [humanlayer/12-factor-agents](https://github.com/humanlayer/12-factor-agents) | 20.5k | TS | Production agent principles | Reference doc | Cross-check ELAI rules against this list |
| [Lum1104/Understand-Anything](https://github.com/Lum1104/Understand-Anything) | 15k | TS | Codebase to interactive knowledge graph | +6.5k/mo | Adjacent to ELAI's symbol graph viewer (B-33) |
| [earendil-works/pi](https://github.com/earendil-works/pi) | 51k | TS | Agent toolkit: CLI + unified LLM API + TUI/web libs | +14k/mo | Library shape ELAI could expose for CLI consumers |
| [microsoft/conductor](https://opensource.microsoft.com/blog/2026/05/14/conductor-deterministic-orchestration-for-multi-agent-ai-workflows/) | n/a | n/a | Deterministic orchestration for multi-agent workflows | MS blog 2026-05-14 | Confirms deterministic-workflow + bounded-agent is the enterprise pattern |
| [browser-use/browser-use](https://github.com/browser-use/browser-use) | n/a | Py | DOM-tree (not screenshot) browser automation | Skill registered for Claude Code | Best-in-class browser tool; integrate as MCP if needed |
| [vercel-labs/agent-browser](https://github.com/vercel-labs/agent-browser) | n/a | Rust | Native Rust browser CLI for agents, annotated screenshots, ~50ms latency | New | Stack-aligned alternative if browser-use Python footprint matters |

## Patterns dominant in 2026

| Pattern | Examples | Grounded source |
|---|---|---|
| Skills as portable plain-text modules (SKILL.md) | Anthropic Agent Skills spec (Dec 2025), OpenAI Codex Skills adoption, Vercel skills.sh (87k skills), agentskill.sh (110k+), Agensi marketplace | [kdnuggets top 5 marketplaces](https://www.kdnuggets.com/top-5-agent-skill-marketplaces-for-building-powerful-ai-agents), [agensi.io](https://www.agensi.io/learn/how-ai-agent-skill-marketplaces-work), [openai dev codex skills](https://developers.openai.com/codex/skills) |
| Deterministic workflow + bounded agent (hybrid) | Microsoft Conductor (May 14 2026), Archon (YAML workflow + AI nodes), enterprise "blueprint pattern" | [MS open source blog 2026-05-14](https://opensource.microsoft.com/blog/2026/05/14/conductor-deterministic-orchestration-for-multi-agent-ai-workflows/), [orkes blog](https://orkes.io/blog/agentic-ai-explained-agents-vs-workflows/) |
| MCP Streamable HTTP over single endpoint replaces SSE | 67% stdio / 28% Streamable HTTP / 5% SSE residual; 18k+ community servers; SSE deprecated by Atlassian Jun 30 2026 | [MCP spec 2025-11-25](https://modelcontextprotocol.io/specification/2025-11-25/basic/transports), [zylos 2026-03-08](https://zylos.ai/research/2026-03-08-mcp-remote-evolution-streamable-http-enterprise-adoption) |
| MicroVM sandboxing (libkrun/Firecracker, not Docker) | microsandbox sub-200ms boot, hardware isolation; Firecracker/gVisor for cloud | [Northflank sandboxing](https://northflank.com/blog/how-to-sandbox-ai-agents), [Brightcoding](https://prompts.brightcoding.dev/blog/microsandbox-hardware-isolated-sandboxes-for-ai-agents) |
| Single-binary Rust harness distribution | codex (Rust 96%), Omegon, pi_agent_rust, agentic-harness, majiayu000/harness | [Zylos Rust ecosystem 2026-04-01](https://zylos.ai/research/2026-04-01-rust-native-ai-agent-frameworks-ecosystem-2026), [OSS Insight](https://ossinsight.io/blog/rust-ai-agent-infrastructure-2026) |
| Persistent code-context graphs + memory | claude-context MCP, agentmemory, Understand-Anything, codegraph | repo descriptions, github trending TS monthly |
| Lead-agent + sub-agent orchestration | Anthropic Code with Claude 2026 (May 8), ruflo, OpenHands Software Agent SDK | [aitoolly May-17](https://aitoolly.com/ai-news/article/2026-05-17-superpowers-a-new-agentic-skill-framework-and-software-development-methodology-for-coding-agents) |
| Tool-output compression / sandboxed tool calls | context-mode (98% reduction), Vertex Agent Engine | github trending TS monthly |
| Verification capacity is the bottleneck, not generation | HN Show HN discourse, OpenHands Index Jan 2026 | [HN coding agent threads](https://www.developersdigest.tech/blog/what-hacker-news-gets-right-about-ai-coding-agents-2026), [OpenHands Index](https://www.openhands.dev/blog/openhands-index) |

## Awesome-list curated picks

| List URL | Top 5 items relevant to ELAI |
|---|---|
| [hesreallyhim/awesome-claude-code](https://github.com/hesreallyhim/awesome-claude-code) (44.2k stars) | Skills, hooks, slash-commands, agent orchestrators, dev tooling. Currently reorganizing — read after structure lands |
| [rohitg00/awesome-claude-code-toolkit](https://github.com/rohitg00/awesome-claude-code-toolkit) | 135 agents, 35 curated skills (+400k via SkillKit), 42 commands, 176 plugins, 20 hooks, 15 rules, 7 templates, 14 MCP configs |
| [ComposioHQ/awesome-claude-skills](https://github.com/ComposioHQ/awesome-claude-skills) | 1000+ production-ready skills cross-compatible with Claude Code / Cursor / Codex / Gemini CLI / Antigravity |
| [modelcontextprotocol/servers](https://github.com/modelcontextprotocol/servers) (85.9k) | Official 7 servers (Everything, Fetch, Filesystem, Git, Memory, Sequential Thinking, Time); PHP MCP, MCPWatch security scanner, mkinf registry new in 2026.1 |
| [ARUNAGIRINATHAN-K/awesome-ai-agents-2026](https://github.com/ARUNAGIRINATHAN-K/awesome-ai-agents-2026) | 300+ frameworks/tools across coding/creative/voice/research/enterprise. Comparison guides |
| [vectara/awesome-agent-failures](https://github.com/vectara/awesome-agent-failures) | Curated failure modes + battle-tested solutions — read before next AUDITING design |
| [addyosmani/agent-skills](https://github.com/addyosmani/agent-skills) (43.3k, +26k/mo) | "Production-grade engineering skills for AI coding agents" — ground truth for SKILL.md conventions |

Top 5 actionable picks specifically for ELAI (from cross-list scan):
1. **MCPWatch** (security scanner for MCP servers) — run before any new MCP integration.
2. **Sequential Thinking MCP** (Anthropic official) — overlaps with ELAI's mcp_think; verify ours is differentiated.
3. **Memory MCP** (Anthropic official) — persistent knowledge graph; compare against ELAI's memory-bank pattern.
4. **claude-context (zilliztech)** — direct competitor to `elai.map`; benchmark token efficiency.
5. **mksglu/context-mode** — tool-output sandboxing; pattern to crib for anti-read-big enforcement.

## Recently abandoned / declined

| Project | 2024 state | 2026 state | Why |
|---|---|---|---|
| [smol-ai/developer](https://github.com/smol-ai/developer) | Hot 2023, embedded dev agent | Historical / not actively developed | Eclipsed by Aider, Cursor, Claude Code. [AINews](https://news.smol.ai/) |
| [gpt-engineer](https://github.com/AntonOsika/gpt-engineer) | Hot 2023, autonomous coder | Historical; team focuses on Lovable | Commercial pivot to Lovable.dev SaaS. [Stoneforge blog](https://stoneforge.ai/blog/open-source-ai-coding-agents/) |
| [entropy-research/Devon](https://github.com/entropy-research/Devon) | Open-source Devin alternative | devon-agent 0.1.25 on PyPI but commit cadence very slow | Lost mindshare to OpenHands and SWE-agent |
| Cal.com (relicensed) | AGPL-3.0 open source | Closed proprietary | "Open source is handing the bank vault blueprint to AI". Industry signal: AGPL is not protecting against AI forks. [opensourcealternatives](https://www.opensourcealternatives.to/blog/open-source-license-guide) |
| MetaGPT (FoundationAgents) | 2024 top multi-agent | Still active but eclipsed by LangGraph/CrewAI/AutoGen in enterprise share | MGX SaaS pivot; framework not the focus. [pasqualepillitteri](https://pasqualepillitteri.it/en/news/1476/10-open-source-ai-agent-frameworks-2026) |
| Magentic-One | Microsoft Research multi-agent prototype | Folded into AutoGen v1.0 GA | AutoGen rewrite at v2 API absorbed Magentic patterns |
| GPT-Researcher | Hot 2024 deep-research agent | Still listed but not in growth tier | Native deep-research now in Claude/GPT/Gemini; standalone less compelling |

## License + governance trends

- **AGPL push-back is real**: Cal.com publicly abandoned AGPL-3.0 for closed-source proprietary citing AI fork risk. Industry chatter ([Julio Merino blog](https://blogsystem5.substack.com/p/ai-and-agpl-licensing), [Patrick Jackson](https://jackson.dev/post/moral-ai-licensing/)) is split — some moving toward AGPL as anti-AI-fork moat (Plandex still AGPL, Warp dual MIT/AGPL).
- **MIT remains dominant for harness-class tooling**: OpenHands (MIT core), Archon (MIT), ruflo (MIT), claude-code, codex. The "agents-in-a-box" wave is MIT-licensed by default.
- **Dual-licensing is rising for commercial-OSS plays**: Warp uses MIT + AGPL dual to allow both community and enterprise paths.
- **SKILL.md as open standard (Anthropic, Dec 2025)** adopted by OpenAI Codex — first cross-vendor agent specification with practical adoption. Implication: a portable skill library is now table-stakes for an open harness.
- **Sources**: [opensourcealternatives 2026 license guide](https://www.opensourcealternatives.to/blog/open-source-license-guide), [openalternative AGPL list](https://openalternative.co/licenses/agpl-3), [stoneforge OSS coding agents](https://stoneforge.ai/blog/open-source-ai-coding-agents/).

## Rust + Tauri + Svelte specific

- **Tauri 2 dominance**: 8MB binaries vs Electron 244MB; 40% startup improvement, 30% memory reduction in 2026 benchmarks. ELAI's stack is on the right side of this trend. [noqta blog](https://noqta.tn/en/blog/tauri-2-desktop-apps-rust-web-technologies-2026).
- **Tauri AI desktop production apps shipping**: Triad (terminal+browser+editor, Tauri 2 + React 19), Cockpit Tools (multi-AI account manager, 12 AI tool integrations), AgentSkills (skills dashboard), Meetily (12.1k stars, local transcription + summarization in Rust).
- **Frontend-agnostic**: Svelte 5 + Tauri 2 is officially supported; no special handling needed.
- **Rust agent harness explosion**: cc-switch (74.5k Tauri), warp (59k), codex (84k, 96% Rust), microsandbox (libkrun), Omegon, pi_agent_rust, majiayu000/harness, agentic-harness. **ELAI is squarely in the most contested niche but with a different angle (FSM + decomposition + grounded reasoning).**
- **No Rust-native equivalent to LangGraph yet** — graph-based deterministic orchestration in Rust is an open lane. Conductor (Microsoft, May 14 2026) is Python/TS.
- Sources: [rustify.rs Tauri v2 tutorial](https://rustify.rs/articles/rust-tauri-v2-desktop-app-tutorial-2026), [zylos Rust ecosystem 2026-04-01](https://zylos.ai/research/2026-04-01-rust-native-ai-agent-frameworks-ecosystem-2026), [refreshagent](https://refreshagent.com/engineering/building-ai-agents-in-rust).

## Anti-patterns shipping in trending repos

Things to NOT copy:

| Anti-pattern | Where seen | Why it bites |
|---|---|---|
| Hype-list skill repos with no spec / quality gate | Multiple top-trending "skills" repos (multica-ai/andrej-karpathy-skills 136k stars, Imbad0202/academic-research-skills 11.6k) — README+markdown copy-paste, no validation | Inflated star count, no production value. Mirror Agensi's security-review-before-listing pattern instead. [agensi](https://www.agensi.io/) |
| Self-evolving / "grows with you" agents without measurable benchmarks | NousResearch/hermes-agent (156k stars, no SWE-bench number) | Marketing > evidence. Grounded-by-default rules this out. |
| Screenshot-only browser automation | Many browser-use clones | DOM-tree (browser-use original) and annotated-screenshot+refs (vercel-labs/agent-browser) are 3-5x faster, 50% fewer tokens. [browser-use](https://browser-use.com/) |
| Iterative repair loops without cost bounds | Generic SWE-agent v1 patterns | 39 turns -> 693 lines of fabricated code is the documented worst case. [codewithrigor 10 failure modes](https://codewithrigor.com/blog/the-10-failure-modes/). Agentless's one-shot localize+repair+validate scores 34.2% at fraction of token cost — bound your loops |
| Skipping verification ("optimize for make it work") | Generic agent failure mode in [Medium anti-patterns part 1](https://medium.com/@wasowski.jarek/7-ai-agent-anti-patterns-that-kill-production-projects-architecture-guide-3bb1a409902e) | 60-70% of AI-introduced vulns are BLOCKER severity. Cursor IDE Git-Hook RCE, Antigravity Sandbox Escape were 2026 incidents |
| Pattern-match dependency picking from training data | Documented in multiple anti-pattern posts | ELAI's anti-slopsquatting rule is correct; this remains a real problem in trending repos |
| Docker-based code exec for agents | Most non-microsandbox tooling | Boot >1s, shared-kernel risk. microVM is the new default |
| Style/convention drift across sessions | Universal AI coding failure | "Each AI session introduces slightly different conventions, creating an inconsistent, unmaintainable codebase" — argues for ELAI's rules-as-data approach |
| Marketing 100+ specialized agents without composition story | ruflo (100+ agents) | Agent sprawl without orchestrator discipline = thrash. Keep ELAI's agent set canonical and small |

Sources for anti-patterns: [Medium 7 anti-patterns](https://medium.com/@wasowski.jarek/7-ai-agent-anti-patterns-that-kill-production-projects-architecture-guide-3bb1a409902e), [Code with Rigor 10 failure modes](https://codewithrigor.com/blog/the-10-failure-modes/), [theweatherreport.ai 7 failure modes](https://theweatherreport.ai/posts/vibe-coding-anti-patterns/), [augmentcode multi-agent failures](https://www.augmentcode.com/guides/why-multi-agent-llm-systems-fail-and-how-to-fix-them), [vectara/awesome-agent-failures](https://github.com/vectara/awesome-agent-failures).

## Open questions

1. **Should ELAI export SKILL.md-compatible manifests?** Cost: skill-loader rewrite. Benefit: cross-vendor portability and discoverability via agentskill.sh / Agensi. Measure by: count of skills consumable by Claude Code / Codex / Cursor after export shim.
2. **Replace any docker/process-based exec with microsandbox/libkrun microVM?** Measure by: sandbox boot p50 (target <200ms), audit failure rate on hostile inputs.
3. **Adopt MCP Streamable HTTP for ELAI's sidecar/CLI bridge?** Currently stdio-only per CLAUDE.md remote-sidecar block. Measure by: round-trip latency, deployment surface (LB-friendly).
4. **Is `elai.map` redundant given `zilliztech/claude-context` (11.4k MCP server)?** Compare token-cost-per-symbol-lookup and latency on a 50-file repo.
5. **Are we losing in the "lead-agent + sub-agent" pattern Anthropic standardized May 8 2026?** ELAI already has sub-agents; verify our orchestrator/auditor split matches the lead/sub-agent terminology and IPC shape.

---

## Sources
- [GitHub Trending daily/weekly/monthly](https://github.com/trending) (fetched 2026-05-19)
- [GitHub Trending Rust monthly](https://github.com/trending/rust?since=monthly)
- [GitHub Trending Python monthly](https://github.com/trending/python?since=monthly)
- [GitHub Trending TypeScript monthly](https://github.com/trending/typescript?since=monthly)
- [k2view awesome MCP](https://www.k2view.com/blog/awesome-mcp-servers), [mcpplaygroundonline](https://mcpplaygroundonline.com/blog/awesome-mcp-servers), [modelcontextprotocol/servers](https://github.com/modelcontextprotocol/servers)
- [hesreallyhim/awesome-claude-code](https://github.com/hesreallyhim/awesome-claude-code), [rohitg00/awesome-claude-code-toolkit](https://github.com/rohitg00/awesome-claude-code-toolkit), [ComposioHQ/awesome-claude-skills](https://github.com/ComposioHQ/awesome-claude-skills)
- [awesomeagents SWE-bench leaderboard](https://awesomeagents.ai/leaderboards/swe-bench-coding-agent-leaderboard/), [codeant SWE-bench scores 2026](https://www.codeant.ai/blogs/swe-bench-scores), [OpenHands Index](https://www.openhands.dev/blog/openhands-index)
- [SitePoint Ollama vs vLLM 2026](https://www.sitepoint.com/ollama-vs-vllm-performance-benchmark-2026/), [LeetLLM inference comparison 2026](https://leetllm.com/blog/llm-inference-engine-comparison-2026), [Joshua8.AI Blackwell benchmark](https://joshua8.ai/llm-inference-benchmark/)
- [LangGraph vs CrewAI vs AutoGen 2026](https://pratikpathak.com/langgraph-vs-crewai-vs-autogen-2026/), [softmaxdata definitive guide 2026](https://softmaxdata.com/blog/definitive-guide-to-agentic-frameworks-in-2026-langgraph-crewai-ag2-openai-and-more/)
- [Northflank sandbox guide 2026](https://northflank.com/blog/how-to-sandbox-ai-agents), [microsandbox research](https://rywalker.com/research/microsandbox), [Blaxel sandbox blog](https://blaxel.ai/blog/code-execution-sandboxes-for-ai-agents)
- [Northflank Claude vs Codex 2026](https://northflank.com/blog/claude-code-vs-openai-codex), [Kingy AI Codex vs Claude](https://kingy.ai/ai/openai-codex-vs-anthropic-claude-code-2026-the-definitive-agentic-coding-comparison/), [Decoder Codex plugin for Claude Code](https://the-decoder.com/openai-launches-a-codex-plugin-that-runs-inside-anthropics-claude-code/)
- [browser-use](https://browser-use.com/), [vercel-labs/agent-browser](https://github.com/vercel-labs/agent-browser)
- [MCP transports 2025-11-25](https://modelcontextprotocol.io/specification/2025-11-25/basic/transports), [zylos remote MCP 2026](https://zylos.ai/research/2026-03-08-mcp-remote-evolution-streamable-http-enterprise-adoption), [digitalapplied MCP stats](https://www.digitalapplied.com/blog/mcp-adoption-statistics-2026-model-context-protocol)
- [Tauri 2 noqta](https://noqta.tn/en/blog/tauri-2-desktop-apps-rust-web-technologies-2026), [tech-insider Tauri vs Electron 2026](https://tech-insider.org/tauri-vs-electron-2026/), [zylos Rust native 2026-04-01](https://zylos.ai/research/2026-04-01-rust-native-ai-agent-frameworks-ecosystem-2026)
- [aitoolly Superpowers May 17](https://aitoolly.com/ai-news/article/2026-05-17-superpowers-a-new-agentic-skill-framework-and-software-development-methodology-for-coding-agents), [shareuhack May 13 trending](https://www.shareuhack.com/en/posts/github-trending-weekly-2026-05-13)
- [MS open source Conductor May 14](https://opensource.microsoft.com/blog/2026/05/14/conductor-deterministic-orchestration-for-multi-agent-ai-workflows/)
- Anti-patterns: [Medium 7 anti-patterns](https://medium.com/@wasowski.jarek/7-ai-agent-anti-patterns-that-kill-production-projects-architecture-guide-3bb1a409902e), [Code with Rigor](https://codewithrigor.com/blog/the-10-failure-modes/), [vectara/awesome-agent-failures](https://github.com/vectara/awesome-agent-failures), [augmentcode debugging AI](https://www.augmentcode.com/guides/debugging-ai-generated-code-8-failure-patterns-and-fixes)
- Licenses: [opensourcealternatives guide](https://www.opensourcealternatives.to/blog/open-source-license-guide), [blogsystem5 AGPL](https://blogsystem5.substack.com/p/ai-and-agpl-licensing), [stoneforge OSS coding agents](https://stoneforge.ai/blog/open-source-ai-coding-agents/)
- Skills marketplaces: [KDnuggets top 5](https://www.kdnuggets.com/top-5-agent-skill-marketplaces-for-building-powerful-ai-agents), [agensi.io](https://www.agensi.io/learn/how-ai-agent-skill-marketplaces-work), [openai codex skills](https://developers.openai.com/codex/skills)

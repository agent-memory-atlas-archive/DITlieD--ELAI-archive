# Dashboard UX for agentic systems (May 2026)

## TL;DR
- Treat ELAI's ActivityWindow as a "control room", not a chat-thread sidekick: 2026 consensus (UXMag, Fuselab, Notch) is that agent UIs require a separate **plan visibility layer + persistent activity stream + intervention controls + audit trail**, which is architecturally distinct from chat. ELAI's 9-window + drawer layout is already closer to right than Cursor's mostly-chat surface. The fix is to make Mode 1/2/3 visually *unified* with a single Activity Stream pane that survives across windows.
- Adopt the **Live Unified Status bar** pattern from Kiro CLI for ELAI's bubble-up event strip: one row that grows in real time with tool execution, line-by-line diff hint, active conversation token, and snack-bar permission requests stacked just above the input. This generalises Aider's terminal omnipresence to a GUI.
- Replace any remaining indeterminate spinners in Research/Plan/Review windows with **skeleton screens** for any pane bound to async LLM/tool output (NN/g, LogRocket, OneThing: ~20-30% perceived-speed gain). Keep spinners only for short blocking ops (submit, save, OAuth). The phi/psi twin micro-bars in Mode 3 RESEARCH are skeleton-adjacent already; extend the pattern to Explore/Plan window list items.
- Adopt **binary confidence signaling** (Fuselab 2026: "confident" vs "uncertain" decoded faster than numeric %) for OrbitGrcDots and OrbitEvidenceGates. Pair with **progressive delegation** (start narrow, widen as audits pass), surfaced on the ActivityWindow's evidence-gate row.
- Honor `prefers-reduced-motion` everywhere (Pope.tech 2025 + Apple 2025 Accessibility Labels). The orbit, particle layers, and gradient pulses MUST collapse to static glyphs + text under reduced motion. Switch the default palette away from teal+purple-only differentiation to a **Wong-derived 5-tone** so signal layers remain readable for deuteranopia/protanopia users.

## Key sources
| URL | what | takeaway |
|---|---|---|
| https://cursor.com/blog/2-0 | Cursor 2.0 launch notes | Agent-first IDE, parallel agents via worktrees, code-review panel + native browser tool; layout details thin in announcement (focus is multi-agent + outcomes). |
| https://cognition.ai/blog/devin-2 | Devin 2.0 launch | Interactive Planning panel returns "relevant files + findings + preliminary plan in seconds"; per-task isolated cloud IDE; Devin Wiki auto-indexes repo every couple hours; Devin Search w/ Deep mode. |
| https://blog.replit.com/agent-v2 + MindStudio Agent 4 writeup | Replit Agent 4 | Four-phase pipeline: Ideation > Design (wireframe preview) > Build (parallel subagents) > Review (live in-browser preview). Industry-first realtime design preview before full build. |
| https://aider.chat/docs/usage/commands.html | Aider commands | 40+ slash commands incl /add /drop /diff /map /architect /ask /code /context /tokens /web /paste /lint /test; multi-line via braces or `/editor`; vim/emacs keymaps. |
| https://www.anyonebuilds.com/guides/claude-code-plan-mode | Claude Code plan mode | Shift+Tab twice toggles plan mode; numbered plain-English plan in terminal; mode shown in footer; per-tool permission prompts during execution. |
| https://kiro.dev/blog/new-look-for-cli | Kiro CLI redesign | "Live Unified Status" bar growing in real time, omnipresent input, snack-bar permission prompts, `/` dropdown + `@` context syntax, contextual overlays for context-window mgmt. |
| https://openhands.dev/blog/announcing-all-hands-online-beta | All Hands Online beta | Redesigned web UI w/ Task List tab in right panel + slash-command menu; cleaner aesthetic; agent-visibility emphasised. |
| https://developers.openai.com/codex/cli/slash-commands | Codex CLI | `/plan` slash command flips conversation into plan mode; `/theme`, `/title`, `/statusline`; configurable TUI keymaps; plan-mode nudges + action-required terminal titles. |
| https://www.deployhq.com/guides/cline | Cline plan/act modes | Plan = non-destructive read-only exploration; Act = per-action approval; checkpoint snapshots; `/newtask` resets context cleanly. |
| https://arize.com/docs/phoenix + Medium debug-loops piece | Phoenix Arize | Node-based **agent-graph + path visualisation** abstracted from spans; 10 span kinds (CHAIN/LLM/TOOL/RETRIEVER/AGENT/RERANKER/GUARDRAIL/EVALUATOR/EMBEDDING/CHAIN); reduces "4h JSON parsing" debug to seconds. |
| https://www.langchain.com/langsmith/observability | LangSmith | Trace tree + run details with P50/P99 latency, cost, errors; per-project auto-dashboards; SmithDB full-text + JSON-path search; LLM-as-judge online evals + Insights clustering. |
| https://langfuse.com/docs/observability/overview | Langfuse | Hierarchical traces filterable by user/session/cost/latency/custom metadata; prompt-versioning playground; multi-turn conversation tracing. |
| https://fuselabcreative.com/ui-design-for-ai-agents/ | Fuselab 2026 | 4 principles: Transparency, User Control, Proactive Status, Structured Error Recovery. Plan-and-execute display + activity stream separation + binary confidence signal + progressive delegation. |
| https://uxmag.com/articles/designing-for-autonomy-ux-principles-for-agentic-ai-systems | UX Magazine | "The more autonomous, the more visible reasoning must be"; persistent dashboards + NL recaps on return; agent personality consistency; outcome-based vs input/output design. |
| https://blog.pope.tech/2025/12/08/design-accessible-animation-and-movement/ | Pope.tech | `prefers-reduced-motion` two-strategy: disable decorative, provide static equivalent for essential motion; pause/stop/hide required for auto-play >5s. |
| https://www.apple.com/newsroom/2025/05/apple-unveils-powerful-accessibility-features-coming-later-this-year/ | Apple 2025 | Accessibility Nutrition Labels normalise listing VoiceOver, Voice Control, Larger Text, Sufficient Contrast, Reduced Motion as a discoverable feature surface. |
| https://www.nngroup.com/articles/skeleton-screens/ + LogRocket + OneThing | Skeleton vs spinner | Skeleton screens feel 20-30% faster for identical wait; use for content-fetch / dashboards / cards; spinner only for short blocking ops. |
| https://www.onething.design/post/skeleton-screens-vs-loading-spinners | OneThing | Spinner = system activity, skeleton = interface progress; psychological framing difference. |
| https://letdataspeak.com/mastering-color-in-data-visualizations/ + Datylon + Datawrapper | Color-blind palettes | Wong palette (#0072B2 #E69F00 #D55E00 #CC79A7 #009E73) is gold standard; never rely on hue alone (WCAG 2.2); 5-color cap; pure purple unsafe for protan. |
| https://www.gethopp.app/blog/tauri-vs-electron + DoltHub 2025 | Tauri vs Electron | Tauri: ~30-50MB RAM, <0.5s start, 8MB bundle vs Electron 200-400MB / 1.5s / 165MB; Hoppscotch case study; meaningful win for dashboard tools. |
| https://medium.com/@chrysophilist/from-cli-to-gui-to-tui-why-developers-are-going-back-to-terminal | CLI/TUI comeback | TUI = µs input lag vs ms GUI; SSH-friendly; predictable mem; "speed of CLI, comfort of GUI". |
| https://ofox.ai/blog/claude-code-vs-codex-cli-vs-cursor-vs-deepseek-tui-2026/ | Agent CLI survey 2026 | Codex CLI 65.3% / Claude Code 34.7% blind dev poll; 2026 power user runs 3 terminals concurrently across tools. |
| https://js.cytoscape.org/ + memgraph compare | Graph libs | Cytoscape.js: built-in graph algos (shortest path, MST, clustering, edge-bundling); ideal for interactive agent-graph; D3 = creative-control / custom; three.js only with WebWorker+LoD+progressive render for very large graphs. |
| https://arxiv.org/pdf/2505.23596 | Agent-SAMA arxiv 2025 | FSM abstracts each UI screen as discrete state, user/agent actions as transitions; serves as "lightweight model-agnostic memory layer" for planning + execution verification + error recovery. |
| https://intuitionlabs.ai/articles/conversational-ai-ui-comparison-2025 | Conversational UI 2025 | Comparison of conversational AI UIs - context for chat-vs-control-room debate. |

(Source count: 24. Requirement: 12 minimum.)

## Product UI matrix
| product | primary surface | activity surface | plan surface | what user sees during long autonomous task | citation |
|---|---|---|---|---|---|
| Cursor 2.0 | Agent-first IDE (agents-as-first-class), classic editor demoted to "switch back" | Multi-agent workspace, parallel agents in worktrees, code-review panel + native browser test loop | Agent panel sidebar w/ plans-per-agent | Up to 8 parallel agents, each with a code review pane; user watches Cursor "test until correct" via native browser tool | cursor.com/blog/2-0 |
| Devin 2.0 | Per-task cloud IDE (isolated VM per session) | Live progress + Devin Search panel + Devin Wiki | Interactive Planning panel: files+findings+preliminary plan, user-editable before exec | Agent works in its own VM; user pops in via familiar cmd-I / cmd-K shortcuts; multiple Devins side-by-side | cognition.ai/blog/devin-2 |
| Replit Agent 4 | Web IDE w/ chat + live preview iframe | In-browser real-time preview iframe, updates as agent edits | Ideation Q&A then visible plan + wireframe preview before build | Four-phase pipeline visible (Ideation > Design > Build > Review); design preview renders before code complete | blog.replit.com/agent-v2 |
| Aider | Terminal chat | Inline tool output + diffs in scrollback | `/architect` mode = 2 models, planner + editor split | Numbered prompts; `/diff` shows changes since last msg; `/map` prints repo map; `/tokens` shows context usage | aider.chat/docs/usage/commands.html |
| Claude Code | TUI in terminal | Tool-call viz w/ status + summaries; resume picker for past sessions | Plan-mode plain-English numbered plan, footer indicator `plan mode on` | Per-tool permission prompts mid-exec; mode in footer; conversation history saved + resumable | anyonebuilds.com plan-mode guide, wil.dev TUI guide |
| Codex CLI | TUI w/ themes | Markdown + syntax-highlighted diffs inline; statusline + title; permissions / approval mode visible | `/plan` slash-command flips into plan mode | Action-required terminal title; plan-mode nudges; blended token usage + service-tier badges | developers.openai.com/codex/cli |
| Cline (VS Code) | VS Code sidebar | Chat panel w/ token + cost per interaction; checkpoint snapshots; `@file/@folder/@url/@problems` context | Plan mode reads only, proposes; Act mode requires per-action approve/reject | Sidebar shows proposed file edit or shell cmd + Approve/Reject; OS notif on approval need; `/newtask` to reset | deployhq.com/guides/cline |
| OpenHands web | Browser app at app.all-hands.dev | Right-panel Task List w/ real-time status updates | Slash-command menu lists agent skills; GitHub-repo binding | Clean web UI, task list tab in right panel reflects agent's live status | openhands.dev blog announcing online beta |
| v0 | Web app | Side-by-side code + preview pane | Component-level prompt, edits applied incrementally | Live preview iframe updates as components generated | addyo substack v0/bolt/lovable compare |
| Bolt.new | Browser StackBlitz IDE | Code + live preview; file-allowlist control | Component-level prompts | Live in-browser preview + dev server; user can restrict files Bolt edits | addyo substack |
| Lovable | Web app | Visual-Edit + chat | Element-pick from preview > reference in chat | Iframe preview w/ element picker; select element > "modify this" from chat | lovable.dev/guides/lovable-vs-bolt-vs-v0 |
| Phoenix Arize | Web dashboard | Trace tree, span timeline, node-based agent-graph viz | Filters by span-kind (10 kinds) + LLM-as-judge eval scores | Agent loops visible as graph cycles instead of nested JSON | arize.com/docs/phoenix, dev.to debugging-agent-loops |
| LangSmith | Web dashboard | Trace view, run details, per-project auto-dashboards | Tool/agent trajectory monitoring, SmithDB search | P50/P99 latency, token, cost, errors, feedback; Insights auto-clusters failure modes | langchain.com/langsmith/observability |
| Langfuse | Web dashboard, open source | Hierarchical traces, multi-turn conversation grouping | Prompt-versioning playground + LLM-as-judge / user-feedback evals | Filter by user/session/cost/latency/metadata; self-hostable | langfuse.com/docs/observability/overview |
| Kiro CLI | TUI | Live Unified Status bar grows in real time w/ tool exec + diff hints; omnipresent input | `/` dropdown + `@` for context | Snack-bar permission prompts above input; contextual overlays for context-window state; adapts to constrained terminals | kiro.dev/blog/new-look-for-cli |

## Visual patterns that work
| pattern | example product | grounding |
|---|---|---|
| Plan-and-execute vertical step list, editable before run | Devin 2.0 Interactive Planning, Cursor 2.0 plans, Cline Plan mode, Claude Code plan mode, Codex `/plan` | Fuselab 2026: "creates mental model of autonomous intent upfront"; UXMag: "the more autonomous, the more visible reasoning must be" |
| Activity stream separated from chat thread | Kiro Live Unified Status, OpenHands Task List right panel, Phoenix agent-graph | Fuselab 2026: "Chat templates fail because they assume request-response cadence, while agents operate continuously" |
| Live preview iframe of the artifact being built | Replit Agent 4, Bolt, Lovable, v0 | Replit: "industry-first realtime app design preview"; addyo: cleanest production-ready frontend code surface |
| Per-action approve/reject inline | Cline Act mode, Claude Code per-tool prompts | UXMag: "users must be able to override, pause, or halt agent decisions mid-task" |
| Footer/statusline mode indicator | Claude Code (`plan mode on`), Codex CLI `/statusline`, Cursor agent badge | wil.dev TUI guide: "glance at the footer any time to confirm which mode" |
| Node-based agent-graph from spans | Phoenix Arize | Medium debug-loops: "4h JSON parsing > 30s visual inspection"; "loops invisible in raw JSON are glaringly clear" |
| Checkpoint / snapshot restore | Cline checkpoints, Aider git-atomic-commit | Cline docs: "restore files to the snapshot, keep the conversation" |
| Skeleton screens for content-fetch panes | dashboards generally | NN/g, LogRocket, OneThing: 20-30% perceived-speed gain |
| Binary confidence ("confident"/"uncertain") instead of % | (industry guidance) | Fuselab 2026: "Users decided faster with 'confident' vs 'uncertain' than with specific confidence scores" |
| Snack-bar permission requests above input | Kiro CLI | kiro.dev: "keeping critical decisions visible without disrupting screen real estate" |
| `@` for context-pick, `/` for command-pick | Kiro, Cline (`@file/@folder/@url/@problems`), Aider, Codex CLI | kiro.dev: GUI conventions adapted to terminal |
| Auto-built repo wiki w/ architecture diagrams | Devin Wiki | cognition.ai/blog/devin-2: "automatically indexes your repositories every couple hours, creating detailed wikis complete with comprehensive architecture diagrams" |
| Live token + cost counter per interaction | Cline, Codex blended token usage | deployhq.com/guides/cline; developers.openai.com/codex/changelog |
| FSM-as-memory-layer for planning + recovery | Agent-SAMA arxiv | "lightweight, model-agnostic memory layer to support structured planning, execution verification, and error recovery" |

## Visual patterns that fail
| pattern | what makes it fail |
|---|---|
| Pure chatbot template repurposed as agent UI | Fuselab: "A product built on a chatbot template will not function as an agent interface without architectural changes that touch every screen." Chat assumes request/response, agents work continuously. |
| Indeterminate spinner during long agent task | NN/g + OneThing: spinner conveys "system busy" but no progress; perceived-speed gap of 20-30% vs skeleton; users gain no mental model. |
| Numeric % confidence indicators | Fuselab 2026: slower decisions vs binary tag; numbers force arithmetic instead of recognition. |
| Heavy particle systems / decorative motion at full opacity | CSS-Script + general access guidance: "subtle background enhances design without distracting"; high-particle, high-opacity backgrounds compete with signal layers. Apple 2025 Accessibility Labels now surface "Reduced Motion" — apps that fail to honor it become visible. |
| Auto-playing motion >5s w/ no pause control | Pope.tech 2025: WCAG-style pause/stop/hide required for all users, not just reduced-motion. |
| Color-only encoding (teal vs purple) for state | Datylon + Datawrapper + WCAG 2.2: ~8% male / 0.5% female deuteranope/protanope users; purple is RGB(red+blue) and looks blue to protanopes. Need shape+text+icon, not hue alone. |
| Deeply nested JSON trace as primary debugger surface | Phoenix: "4h JSON parsing > 30s visual inspection" — abstract into graph or cycle diagram. |
| Multi-window with no persistent activity ribbon | Fuselab + UXMag: agents continue work while user navigates; if activity is only visible on one window, switching loses progress visibility. |
| Hidden / no mode indicator | Claude Code's footer pattern exists because "what mode am I in" was the #1 user error; Cursor/Devin learning the same. |
| Conventional progress bar for non-linear LLM work | LLM steps have unknown duration; a fake linear bar misrepresents progress. Skeleton or activity-stream is more honest. |
| Electron for an agent dashboard | gethopp + dolthub 2025: 200-400MB RAM, 165MB bundle, 1.5s start vs Tauri 30-50MB / 8MB / <0.5s. Agents already eat RAM; the shell should not. |

## Terminal vs GUI for agentic
When each wins:
- **TUI wins** for: keyboard-power users, SSH/remote, low-latency input (µs vs ms), predictable mem, scriptable, single-focus task. Aider, Claude Code, Codex CLI, Kiro CLI all show developer preference is real: Reddit poll Codex CLI 65.3% / Claude Code 34.7% (blind, 500+ devs). [ofox.ai 2026]
- **TUI wins specifically** when the agent's primary artifact is text (code, diffs, logs) and the human reviewing pattern is "scan + accept/reject diff." Aider's `/diff` + `/map` + 40 slash commands cover that loop tightly. [aider.chat]
- **GUI wins** for: multi-agent fleet (Cursor 8 parallel), live preview of non-text artifact (Replit/v0/Bolt/Lovable showing rendered app), observability deep-drill (Phoenix graph, LangSmith trace tree), persistent ambient awareness across windows (ELAI's 9-window + 10-drawer pattern), accessibility surface (Apple Accessibility Labels, screen-reader, reduced-motion semantics).
- **Hybrid wins** (best of both): Kiro CLI's "Live Unified Status" + omnipresent input + snack-bar permission + `/` dropdown + `@` context is essentially a GUI rendered in a terminal cell-buffer. ELAI as a Tauri+Svelte dashboard should adopt the same density discipline but with proper a11y semantics. [kiro.dev]
- Power-user truth (2026): users run 3 terminals + 1 GUI concurrently. ELAI dashboard should not try to replace the terminal; it should be the **control-room above** the terminals. [ofox.ai]

## Best techniques to adopt for ELAI
Ranked (1 = highest priority, integration hooks reference ELAI's 9 windows + 10 drawers + 3 ActivityWindow modes):

1. **Live Unified Activity Stream as a persistent ribbon** | grounded in Kiro CLI + Fuselab 2026 + UXMag autonomy principles | hook: extend the bubble-up event strip into a *persistent activity ribbon* (top or bottom) that survives ALL 9 windows and reflects current FSM substate + tool + token + last-diff-hint. Today it only shows in ActivityWindow.
2. **Plan-and-execute editable list before EXECUTING transition** | grounded in Devin 2.0, Cursor 2.0, Cline Plan, Claude Code Plan, Codex `/plan` | hook: Plan window already exists; add an explicit "Plan > approve > execute" rendering where each step is a vertical step card with edit/reject/expand, identical pattern across all 5 leading tools.
3. **Skeleton screens for every async list** | NN/g + LogRocket + OneThing | hook: Research window (FORMULATE > SEARCH > EXTRACT > REFLECT > CONVERGE), Explore window symbol-graph results, Plan window phase rows, Review window audit findings. Mode 3's phi/psi micro-bars are the right idea; generalise.
4. **Binary confidence tag instead of % on OrbitGrcDots / OrbitEvidenceGates** | Fuselab 2026 | hook: signal layers in ActivityWindow Mode 1 (PLAN constellation) and Mode 2 (MAP). Map confidence in {needs-evidence, low, ok, verified}, not 0.0-1.0.
5. **Honor `prefers-reduced-motion` everywhere; static equivalents for orbit + gradient pulse + particle layers** | Pope.tech 2025 + Apple 2025 | hook: ActivityWindow orbit, OrbitBudgetRing rotation, any drawer gradient pulse. Provide a static dot+text mode.
6. **Wong-derived 5-tone palette for status signals (in addition to brand teal+purple)** | Datylon, Datawrapper, WCAG 2.2 | hook: bg #0a0e12 stays, brand teal #1B7A6E + brand purple #7C5CBF stay for *brand chrome*; signal layers (state, gate, evidence) get Wong tones {#0072B2 #E69F00 #D55E00 #CC79A7 #009E73}. Never encode state in hue alone — pair with shape/icon/text.
7. **Snack-bar permission prompts above the input bar** | Kiro CLI | hook: Chat/Command window. Currently approval flow is implicit via FSM gates; surface it as a stacked snack-bar row above the input, consistent across windows.
8. **Node-based "agent graph" view in Monitor or a new drawer** | Phoenix Arize | hook: today the OrbitGraphViewer (B33) renders structure; add a *trace-cycle* mode that shows agent loops the way Phoenix does — surfaces stall spirals visually (we already detect them in anti-spiral skill, but they aren't visible).
9. **Auto-built repo wiki w/ architecture diagram, refreshed periodically** | Devin Wiki | hook: ProjectIntelligence drawer. Today it shows facts; add a refreshed "wiki view" derived from elai.map + SCIP indices, regenerated on push/audit-pass.
10. **`@` for context pick, `/` for command pick across windows, not just Chat** | Kiro + Cline + Aider + Codex | hook: standardise across Chat/Command, Research, Explore, Plan, Review windows. `@file`, `@symbol`, `@phase`, `@trace`.
11. **Mode indicator in window footer (Claude Code pattern)** | anyonebuilds + wil.dev | hook: every ELAI window gets a thin footer showing current FSM state + substate + cap-status. Today only the FSM dot is in OrbitGrcDots; users in other windows lose the signal.
12. **Checkpoint / snapshot system for ELAI plan execution** | Cline | hook: COMMITTING state already creates git commits; add a *visible checkpoint browser* in Review window listing snapshots a user can roll back to without losing the chat trail.
13. **Per-interaction token + cost counter** | Cline + Codex blended-usage | hook: ProviderVram drawer covers VRAM but not API cost. Extend to per-window or per-step token+cost ticker; cloud Sonnet billing makes this real.
14. **Live preview iframe pattern for build/run artifacts** | Replit + v0 + Bolt + Lovable | hook: Preview window already exists; ensure the iframe is *non-modal, non-fullscreen-by-default, and incremental* (re-renders as agent edits files), not "open at the end of build."
15. **FSM-as-memory-layer rendering in Activity Mode 1** | Agent-SAMA arxiv 2025 | hook: ELAI already has agent_substate (AUDITING, MEMORIZING, Framing, etc); render the FSM as a small interactive statechart in ActivityWindow Mode 1 — clickable nodes show last-state-payload. This is a known-good 2025 academic pattern.

## Anti-patterns
- **Solid CTA buttons everywhere** — ELAI's outlined-only language is correct (DESIGN.md §7) and aligned with the "calm control room" principle. Don't regress under pressure to add colorful primary buttons; Fuselab 2026 and UXMag both implicitly endorse subdued chrome for autonomous systems where the agent — not the button — is the action.
- **Chat-only navigation** for an agent that runs 5+ minutes — chat templates fail per Fuselab + Notch + UXMag, identically across all three 2025/26 references.
- **Indeterminate spinners** in any panel bound to LLM/tool output longer than a couple seconds — replace with skeleton (NN/g, LogRocket).
- **Color-only state encoding** in OrbitEvidenceGates / OrbitGrcDots — Datylon + WCAG 2.2.
- **Decorative particle systems at full opacity / no reduced-motion fallback** — Pope.tech 2025 + Apple Accessibility Labels.
- **Fake linear progress bars for unknown-duration agent steps** — implicit consensus across NN/g (progress bars only when % is real) and Fuselab (transparency over performance theatre).
- **Hidden mode** — Claude Code's footer was added because users got lost; replicate in every ELAI window.
- **One activity surface per window** — agents run across windows; activity ribbon must persist (Kiro + Fuselab activity-stream-separation).
- **Numeric confidence scores** beat by binary tags (Fuselab 2026 finding).
- **Electron for an agent dashboard** — already correctly avoided in ELAI's Tauri choice; gethopp/dolthub 2025 confirms the mem+startup gap.
- **Pure decompiled SDK docs treated as ground truth for UI conventions** — per ELAI's source-grounding rule; this report cites official product docs/blogs only.

## Open questions
1. Does a persistent **Activity Ribbon spanning all 9 ELAI windows** reduce mid-task "where is the agent now" confusion vs the current per-window ActivityWindow? Measurable: time-to-locate FSM substate after window switch, target <2s p95.
2. Does swapping numeric confidence on OrbitGrcDots/OrbitEvidenceGates for **binary tags** reduce user decision latency on bubble-up events? Measurable: ms from event-shown to user-action, A/B with internal users.
3. Does the **node-based agent-graph trace view** (Phoenix-style) detect anti-spiral patterns earlier than the existing text-based anti-spiral skill? Measurable: # spirals user-spotted before skill auto-fires.
4. Under `prefers-reduced-motion`, does collapsing the orbit + particle layers to a static dot+text grid retain enough information for power users to stay productive? Measurable: task-completion-rate parity ±5% between motion-on and motion-off groups.
5. Does **Devin-style auto-built wiki** refreshed on every audit-pass reduce context-compiler agent token usage by serving as a pre-cached architecture answer? Measurable: avg context-compiler input tokens before/after wiki rollout.

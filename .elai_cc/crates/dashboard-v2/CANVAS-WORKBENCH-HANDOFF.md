# ELAI Canvas, Session Handoff

Date: 2026-06-03. Purpose: a de-verbosed record of the session that produced the canvas-workbench design, with every load-bearing conclusion, the file paths created, the Claude Design prompts kept verbatim, and where to resume. Companion to CANVAS-WORKBENCH-DESIGN.md in this same folder.

FILES CREATED THIS SESSION
- F:\.ELAI_workflow\.elai_cc\crates\dashboard-v2\CANVAS-WORKBENCH-DESIGN.md, the 14-section design spec that replaces the orbit with a 2D canvas. Sections 4 (codebase graph from map) and 6 (file explorer + bidirectional sync) carry a dated "verified against the code (2026-06-03)" block grounded in a direct read of the knowledge crate, the symbol-graph crate, and the dashboard-v2 src-tauri.
- F:\.ELAI_workflow\.elai_cc\crates\dashboard-v2\CANVAS-WORKBENCH-HANDOFF.md, this file.
- Grounding came from 5 research/explore workflow runs this session (Replit/Lovable, node-graph survey, website-canvas survey, click-to-edit mechanism, ELAI internals + design). Transcripts under the session subagents/workflows dir if a future run needs the raw citations.

THE CONVERSATION ARC (the through-line)
The whole session is one idea seen from five angles, the realization is that they are all the same primitive:
1. "are Replit/Lovable good for making apps", led to
2. "does a node-graph that shows each app PAGE as a live node exist", led to
3. same question for websites (Framer etc), led to
4. "can i click an element on a rendered page and tell Claude to change it", led to
5. "make a general visual canvas for any LLM session, replace ELAI's orbit". The unifier: one general, bidirectional, agent-shared spatial canvas where a node is any addressable object and the agent draws on it while the user points at it.

KEY GROUNDED CONCLUSIONS (de-verbosed)

Replit / Lovable / Bolt / v0 (mid-2026)
- 3 of 4 are Claude-first wrappers (Replit = Sonnet 4.5 via Vertex, Lovable = Opus 4.x, Bolt = built on the Claude Agent SDK). v0 is the exception, a composite (Claude base + RAG + a self-trained autofixer).
- Emil's take scored: right that they are Claude wrappers, right that they are more expensive for pure code-gen (flat $20 Claude Code Pro vs uncapped credit metering, Replit heavy users $100-300/mo, worst $350/day), right on audience (non-coders). Wrong that they are "worse" (for prompt-to-deployed-app a same-app test ranked Replit and Lovable ABOVE Claude Code, which only built frontends). Factually wrong on "found through ads", both grew via product virality, zero paid acquisition.
- Mobile reality (matters for the Emil/Expo gig): Replit and Bolt can ship real Expo/iOS; Lovable and v0 are web-only and need converters. For maintainer's locked Expo + EAS pipeline, none of them add value he doesnt already have.

Node-graph "each node is a live app page" visualizer
- Does NOT exist as a product, for apps or websites. It is a two-of-three gap (page-as-node + nav edges + live render in node, no tool does all three).
- Build-your-own is straightforward: React Flow (the lib n8n uses) for the canvas + edges, react-native-web (Storybook's RNW-Vite engine) for live RN render per node, expo-router route parsing for auto nodes/edges, viewport virtualization for scale. Days to ~2 weeks for an MVP.
- Web is the easier variant: a page is just a URL in an iframe (no RNW needed), auto-derive nodes/edges from sitemap.xml or routes + cheerio link parse. Hard caveat: X-Frame-Options / CSP frame-ancestors block embedding third-party sites, so live render works cleanly only on your own sites; arbitrary third-party falls back to screenshots.

Claude Code vs ELAI for making apps
- Not a real binary. ELAI is a fork of Claude Code; its endgame (Phase 4M) is to orchestrate Claude Code, not replace it. For shipping apps in the next months, Claude Code + the apps_workflow harness wins on maturity, mobile pipeline, zero build cost. ELAI is the long-horizon bet (local-model cost, determinism, native harness). Build on Claude Code now, migrate the harness onto ELAI once 4D-4M stabilize.

Click-an-element-then-the-model-edits-it (the mechanism)
- This primitive is SOLVED and shipping: Onlook (OSS, ~26k stars, infinite canvas of live React frames + per-element click-to-AI), Lovable Visual Edits, v0 Design Mode, Cursor 3 Design Mode (ships the agent a context package = element HTML + applied CSS + bounding box, runs parallel per-element agents), Figma Make, Bolt inspector. Do not reinvent it, fork/study Onlook, copy Cursor 3's context-package shape.
- Mechanism: build-time stable-ID tagging (data-oid keyed to an AST range, NOT line numbers) + AST format-preserving patch (recast) + two-tier writeback (deterministic for styles, LLM only for structural). React 19 removed Fiber _debugSource so build-time tagging is mandatory. React Native has no DOM, selection is Fiber + getInspectorDataForViewAtPoint, and many live RN screens at once is impractical (each is a native runtime), so web is the easy half.
- The unoccupied white space is the COMBINATION: whole-app canvas of many live screen-nodes + flow edges + per-element click-to-agent context that carries which screen the element is on. Nobody ships that.

ELAI canvas-workbench (the main design)
- Decision: replace the three.js orbit as the primary surface with a clean 2D Figma/n8n work-surface that keeps the galaxy soul (teal, ambient stars, dark, determinism) minus the heavy 3D. Replace by ABSORBING, the orbit's 3 modes (PLAN tree, MAP codebase, RESEARCH tree) are already node-graphs and become node/edge types on one canvas.
- It is a work-surface the agent populates and the user points at, NOT a pipeline editor you wire like n8n. That fork is closed deliberately.
- Two new primitives are the real value: agent>canvas (the model has a tool to place/connect/annotate nodes, so it shows instead of dumping ASCII) and canvas>agent (the user clicks/lassoes nodes to give context instead of typing).
- Multi-mode, keyed to what ELAI does: code-graph, plan, research, architecture-diagram, app/website design (the page-node + click-to-edit idea, web-first, RN deferred).
- Reuse vs net-new (verified): SSB plumbing, Svelte state stores, tokens, ui components, the determinism LCG, cytoscape+dagre+fcose, and the B-29/B-33/B-34 graph data + Tauri commands all SHIP and reuse. graph.svelte.ts is the existing bidirectional focus bus and has zero three.js imports (ports clean). What dies is only the three.js scene.
- The two honest prerequisites that must each be their own gated step: (a) dead-code "disconnected = dead" needs a coverage producer + a dynamic-dispatch entrypoints map, neither exists, ELAI has no dead-code query; (b) the file explorer is blocked because the Tauri app has no filesystem permission at all today (only core:default + dialog:default, tauri-plugin-fs not even a dependency), so a scoped fs command surface is a prerequisite.
- First slice (cheapest thesis proof): the codebase-graph-from-map view, it reuses the most shipped data.

CLAUDE DESIGN PROMPTS (kept verbatim)

PROMPT 1, the interactive page-graph canvas prototype (nodes toggle phone/browser frame, dark technical style):

Build an interactive, infinite-canvas node-graph visualizer for the pages of an
app or website. Think "n8n / Blender node graph, but every node is one screen of
a product and the edges are the navigation between screens." Single full-screen
dark canvas, highly polished, clickable.

CONCEPT
- One full-screen canvas. Pan by dragging empty space, zoom with scroll/pinch,
  fit-to-view button. Subtle dot-grid background.
- Each node is a card representing one page/screen of a product. The card shows a
  live-looking preview of that screen rendered inside a device frame.
- Edges are directional connectors (smooth bezier curves with an arrowhead) that
  represent navigation from one screen to another. Each edge carries a small label
  for the action that triggers it (e.g. "Next", "Get Started", "tab: Profile",
  "link: Pricing").

NODE
- Frame toggle: each node renders in one of two frames. PHONE frame (tall, about
  9:19.5, rounded corners, notch) or BROWSER frame (wide, with a fake address bar
  showing the route). A global toggle in the toolbar flips every node at once; a
  tiny button on each node overrides it per-node.
- Header: monospace route label (e.g. /onboarding/welcome, /home, /settings) plus
  a small colored badge for page type (entry, screen, tab, modal, paywall).
- Body: a mock rendered preview of that screen (a real little UI mockup, not an
  image placeholder) so it reads like a live page. Believable content per screen.
- Connection handles: small dots on the node edges where connectors attach.
- States: hover lifts the card slightly. Selecting a node gives it a glowing accent
  border and brings its connected edges to full brightness while dimming the rest.

CANVAS + INTERACTIONS
- Drag nodes to reposition, edges stay attached and re-route.
- Minimap bottom-right reflecting the whole graph, with a draggable viewport box.
- Click a node to select it and open a right-side inspector (route, type, incoming
  links, outgoing links). Double-click a node to zoom the canvas to fit it large.
- Hover an edge to reveal its label.
- "Auto-arrange" button that lays the graph out as a clean left-to-right flow from
  the entry node.

TOOLBAR (top)
- Product name, a phone/browser global frame toggle, auto-arrange, fit-view, live
  zoom %, a search box to jump to a page by route, and a small legend for the badge
  colors.

VISUAL STYLE (dark technical, n8n / Blender)
- Background #0F1318 with a subtle dot grid.
- Edges: thin teal/cyan bezier lines with a soft glow and a slow animated dashed
  flow, arrowheads at the target end.
- Nodes: rounded cards #161B22, a 2px colored top border by page type, soft shadow,
  monospace labels (JetBrains Mono style). Selection glow in teal.
- Badge colors: entry = green, screen = slate blue, tab = cyan, modal = amber,
  paywall = magenta.
- Dense, technical, information-first, minimal chrome.

SAMPLE DATA (seed the canvas with this so it renders immediately)
A small mobile-app graph, each node a believable mock screen:
  /splash (entry)        > /onboarding/1   label "Open"
  /onboarding/1          > /onboarding/2   label "Next"
  /onboarding/2          > /onboarding/3   label "Next"
  /onboarding/3          > /auth           label "Get Started"
  /auth (sign up/log in) > /home           label "Continue"
  /home (tab)            > /profile        label "tab: Profile"
  /home                  > /settings       label "tab: Settings"
  /home                  > /item/:id       label "open item"
  /item/:id              > /paywall (modal) label "Upgrade"
  /settings              > /paywall        label "Go Pro"
Also add a few website-style routes so the browser-frame toggle has something to
show: / (home), /pricing, /blog, /login, with nav-link edges between them.

TECH NOTES
- A node-graph canvas library like React Flow (xyflow) is a natural fit for the
  canvas, edges, minimap and pan/zoom. Render each node's preview as a small real
  React mockup inside the device frame.
- Do NOT embed external URLs in iframes. Render mock screens directly so the
  prototype is fully self-contained.

GOAL
- A convincing, clickable prototype that sells the idea: open it and immediately see a
  product's whole page map as a living graph, toggle phone vs browser frames, pan
  around, select a screen, and follow the navigation edges between screens.


PROMPT 2, the per-screen edit-mode follow-up (click an element, inspector, ask Claude, "what Claude sees" panel):

Iterate on the page-graph canvas you already built. Add a per-screen EDIT MODE
that lets me click any element inside a screen-node and direct an AI at exactly
that element, no typing-to-describe-which-element needed. Take inspiration from
Framer's on-page editing and Cursor's design mode. Keep the dark technical style.

ENTERING EDIT MODE
- Double-clicking a screen-node (or a "focus" button on it) zooms the canvas to
  that screen and enters Edit Mode for it. An "interact / edit" toggle in the
  toolbar switches between using the screen normally and selecting its elements,
  so selection never fights the screen's own buttons.

HOVER + SELECT (devtools / Framer style)
- In Edit Mode, hovering an element draws a crisp outline box around it (an overlay,
  not a style change on the element) with a small label showing the element type
  and a fake source path, e.g. "Text · Title · /home · Home.tsx:42".
- Clicking selects it: the outline becomes a solid teal selection box with corner
  handles, and a small floating toolbar appears above it (Figma-style) with quick
  actions (edit text, color, font, duplicate, delete).
- Smallest element wins on overlapping hits. Multi-select with shift-click to batch.
- A selection breadcrumb shows the path: Home screen > Header > Title.

RIGHT INSPECTOR (make these controls actually work on the mock element)
- When an element is selected, a right panel shows real, working controls for it:
  text content, font family, size, weight, color, background, padding/margin,
  border radius, alignment. Changing a control updates the mock element live.
- This is the deterministic path: simple property tweaks happen instantly with no
  "AI" involved.

ASK CLAUDE (bound to the selection = the useful part)
- Below the inspector, an "Ask Claude" input that is bound to the current selection.
  With the title selected I can type "change the font to a rounded sans and make it
  bigger" and hit enter. Show a brief "Claude is editing Title…" state, then apply a
  believable change to that exact element, and surface an accept / reject diff chip.
- A context chip above the input always shows what's selected: "Selected: Title
  (text) on /home". The instruction targets the selection, I never describe which
  element.

"WHAT CLAUDE SEES" PANEL (sell the usefulness, not just the visuals)
- A collapsible panel that shows the exact context package that would be sent to the
  coding agent for the current selection, formatted like a real payload:
    screen route:  /home
    component:     Home.tsx  (line 42)
    element:       <Text> "Welcome back"
    classes:       text-2xl font-semibold text-white
    bounding box:  x:24 y:88 w:240 h:34
    instruction:   "change the font to a rounded sans, make it bigger"
- This demonstrates that clicking an element captures precise, scoped context for
  the agent. Update it live as I select different elements.

PARALLEL EDITS (nod to the multi-screen idea)
- Show that I can select elements on two different screen-nodes and fire an
  instruction on each, with both showing an independent "Claude is editing…" state
  at the same time (simulated), to convey per-element / per-screen parallel agents.

SEED CONTENT
- Reuse the existing screens. Make the /home and /onboarding screens have clearly
  clickable elements (a title, a subtitle, a button, an image, a list row) so the
  select-and-edit flow is obvious in a demo.

GOAL
- A clickable prototype that shows the real loop: pan the whole-app graph, focus one
  screen, click an element, watch the precise context get captured, and tell Claude
  to change just that element. Visual AND useful.

OPEN DECISIONS / NEXT STEPS
- Recommended next step: convert section 13 of CANVAS-WORKBENCH-DESIGN.md into real B/C-style session files in the ELAI repo, with the two prerequisites broken out as their own gated baby steps (coverage producer + entrypoints map for the dead-code visual; the scoped fs command surface for the explorer), first slice = the codebase-graph-from-map view.
- Alternative path: build the first-slice prototype as a Claude Design prompt (codebase-graph-from-map canvas reading a real SSB stream) before committing ELAI sessions.
- Unresolved fork already closed in the design but worth re-confirming if scope shifts: work-surface (chosen) vs pipeline-editor. Design-mode is web-first by necessity, RN/Expo is a separately scoped later slice.

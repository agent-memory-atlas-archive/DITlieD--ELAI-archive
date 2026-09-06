ELAI DASHBOARD REDESIGN — running brief + decisions (what we've learned so far)
status: living, 2026-06-14. Source of truth for the redesign builders. Pairs with workflow/design/DESIGN.md (v1.3 canon) + the mockups under workflow/design/mockup/.
scope boundary: ALL code targets /workspace/ELAI\.elai_cc\crates\dashboard-v2\ (frontend). Nothing committed yet — the whole redesign lands as ONE changeset when maintainer is satisfied.

================================================================================
0  SEQUENCING (decided)
================================================================================
1. Brand/entry screens — SHIPPED (intro + project-selection).
2. Window chrome (frameless titlebar) — SHIPPED.
3. WORKFLOW SHELL redesign — THIS phase (Opus). The 4-tab IA + icon rail + file-explorer flyout + chat + settings-modal + statusbar, to DESIGN.md. Defaults + reduction on the EXISTING shell, not a from-scratch rebuild.
4. CANVAS — next phase. Restyle landed canvas to DESIGN.md + the new surfaces (system mode J-31, the "new diff" node-story, CODEBASE lenses, KNOWLEDGE notes-canvas, explain layer, research-as-drill-in).
5. H-16 retire orbit — LAST, gated (only after canvas is default + ships a full release with no regressions). The orbit/three.js is backed up at workflow/backups/canvas-orbit-2026-06-14/.

================================================================================
1  BRAND + ENTRY SCREENS (shipped — design choices locked)
================================================================================
- intro (Intro.svelte): cosmic hero — starfield + teal bloom (ALWAYS on, NO GLOW toggle retired) + 3 rings + white hairline stroke-draw logo (DESIGN §5 geometry) + click-replay + START→curtain. NO mode buttons on intro. NO tagline (slot empty, unresolved §1/§14). sun/moon theme toggle + window controls top-right.
- project-selection (ProjectSelector.svelte): solid software-window (no starfield), LEDGER list. Modes LOCAL/CLOUD/HYBRID CENTER-TOP of the topbar, START-underline style (no box; center→out on hover; active keeps the full line; wired to session.setMode). Ledger = name-first (NO index gutter), brighter (--read) PROJECT/PATH/UPDATED headers, real fields only (name·path·updated — invented columns dropped). Row hover+select = center→out underline (receipt stamp retired for row-select; mono-inversion stays the §6 error language). New Project CENTERED with the same underline. Full state set (loading/data/single/empty/error/offline) from the real list_projects outcome. Really wired: list_projects + open_project_from_folder + reindex + create.

================================================================================
2  WINDOW CHROME (shipped)
================================================================================
- Frameless: tauri.conf.json decorations:false (kept title "ELAI" for taskbar). Custom WindowControls.svelte (min/maximize-restore/close) + ResizeFrame.svelte (8 edge/corner handles, startResizeDragging) + data-tauri-drag-region on the top strips of all 3 screens. capabilities/default.json carries the 7 core:window:allow-* perms (verified vs installed Tauri 2.11.1).
- Close button avoids red: rest --dim glyph; hover = §6 MONO-INVERSION (light chip / dark glyph). One teal accent. Browser-dev safe (hides outside Tauri).

================================================================================
3  COLOR SYSTEM
================================================================================
- DARK ramp (settled, DESIGN §3): --void #030404 / --bg #0a0f0e / --panel #0e1413 / --raised #151c1a / --sink #070b0a / --row #0c1211 / --text #f2f5f4 / --read #c9cecb / --dim #79827e / --ghost #3a423f / --line + --line2 BOTH rgba(255,255,255,.17) / teal #2BBBA4·#34C2AA·#1B7A6E.
- LIGHT/DAY ramp (settled this session, DESIGN §12): --void #e9ede9 / --bg #eef1ef / --panel #ffffff / --raised #f4f6f4 / --sink #e6eae7 / --row #f7f9f7 / --text #0b0f0e / --read #2d342f / --dim #646b66 / --ghost #aab1ac / --line + --line2 BOTH rgba(0,0,0,.30) / teal #138a78·#0f7e6f·#0c6256.
- Tokens added ADDITIVELY to tokens.css (the new teal is aliased --d-teal* because legacy --teal #1B7A6E still exists; the 2 shipped screens use the new names). The legacy ~90-panel token migration onto these role tokens is a ONE-TIME future unification pass (prereq for theme/accent to reach the whole app).
- ACCENT THEMING — PLANNED Settings feature (DESIGN §15, NOT built): a master HUE knob + vividness rotates the whole accent family + neutral tint keeping baseline S/L; a MONOCHROME/DISABLE (pure black-white) option is part of the package; semantic data-viz palettes (FSM/confidence/tier/gate) NEVER rotate; BRAND surfaces are teal-LOCKED; it's a quiet Settings feature, never a marketing headline. Live tuner: workflow/design/mockup/intro.html.

================================================================================
4  THE 4-TAB IA (decided)
================================================================================
Primary nav = FOUR question-tabs (top bar), renamed from the old set:
- PROGRESS  — planning + execution (the default home; the plan tree + run state). research happens here as a DRILL-IN, not a tab.
- CODEBASE  — (was PROJECT) the code graph + LENSES: Structure / Security / History / Cost / Dead-code.
- FRONTEND  — (was APP) the visual/frontend view: pages, click-to-edit, before/after.
- KNOWLEDGE — (name PROVISIONAL) a notes/ideas "brain" canvas: the LLM drafts notes, the user connects them; research findings accrue here as connectable notes. Rename pending.

RESEARCH = a DRILL-IN, not a tab (decided). It uses the SHIPPED spawn_view reply channel (H-15): while ELAI plans, the plan node it researches grows a research affordance/chip; click → the canvas PUSHES into that research branch sub-view; breadcrumb back out. Every PROGRESS plan-tree node can carry its own research drill-in, and its findings feed the KNOWLEDGE brain (research is both contextual to the node and cumulative). Caveats: (1) drill-in/replace nav MUST have a breadcrumb + one-key back-out (avoid the Obsidian-hairball lost-in-canvas failure); keep views local/scoped. (2) Don't pop a floating node per micro-search — grow a chip that expands on click (silence-means-safe).

================================================================================
5  WORKFLOW SHELL SPEC  (THIS build phase)
================================================================================
Principle: "defaults + reduction on the existing AppShell, NOT a rebuild." Current shell is a window/tab model (AppShell grid: 48px sidebar | explorer rail 0px | panels-area primary/secondary tabs | dead right rail | 22px statusbar; default primary='activity' orbit, secondary='command' chat; settings is a WINDOW). Transform it to:

- TOP 4-question-tab bar (PROGRESS/CODEBASE/FRONTEND/KNOWLEDGE) as primary nav, DESIGN-styled (dot+ALLCAPS-mono labels, one teal active accent, segmented; running n/m fraction bar on the right per DESIGN §8 / the mockup).
- THIN LEFT ICON RAIL (the "side menu", ~48px, VS-Code-activity-bar idiom). DESIGN the hairline icons (one teal accent on active; no generic icon-tiles, §10). It carries at least: a MENU/flyout toggle (project intelligence + the collapsed monitor long-tail + settings) and a FILE-EXPLORER toggle.
- FILE-EXPLORER (+ menu) = a ZERO-WIDTH FLYOUT that EXPANDS LEFT→RIGHT over the workspace when its rail icon is clicked (scrim behind, ~220ms slide; hidden while working; never a permanent content rail). This is DESIGN §8's flyout, triggered from the icon rail. The file tree is the existing H-11 fileTree (read_dir) — keep its wiring.
- CHAT pane alongside the canvas (the chat IS CommandWindow.svelte), draggable splitter between them.
- SETTINGS = a POPUP MODAL opened from the flyout (semantic z-scale), NOT a window/tab. (Today SettingsWindow is a window — convert the surfacing to a modal; keep all its settings wiring.)
- 22px STATUSBAR reskinned (FSM chip + session id + mode + note).
- The ~90 legacy monitor panels: keep them MOUNTED + working; collapse them into the flyout's monitors long-tail per the ledger. Their internal restyle is a LATER pass (token unification) — do NOT reskin all 90 now.

Visual target = workflow/design/mockup/elai-mockup.html (the full skeleton mockup: boot > dashboard > 4 tabs > chat > flyout > settings modal > statusbar). DESIGN.md is the language. The 2 shipped screens (Intro/ProjectSelector) are the token + idiom reference.

================================================================================
6  CANVAS  (next phase — context, not this build)
================================================================================
- LANDED plumbing (commit e38cd1a): H-10 canvas shell+code-graph, H-11 live layer+file explorer, H-12 plan/research modes+chrome (the "Canvas-as-default" step), H-13 model-drives-canvas op stream, H-14 design mode, H-15 spawn_view reply channel. Code: src/lib/canvas/* (ArchMode/PlanMode/ResearchMode/DesignPageNode/ComparisonBoard + renderer_registry/tab_reconciliation/arch_canvas_ops/entrypoints_map). CanvasMode union: code|plan|research|arch|design.
- NEW surfaces to build on top: (a) restyle to DESIGN.md (flat-matte work surface — galaxy/glow is BRAND-only, never the canvas graph; quinary state language fill/glyph/edge; role tokens); (b) the "NEW DIFF" node-story (open a node → HISTORY/EXPLAIN/WIRING/RECEIPTS/COST/ACTIVITY; raw diff collapsed one-click-deep — the user's centerpiece); (c) the CODEBASE 5 lenses; (d) the KNOWLEDGE notes-brain; (e) the explain layer (cached plain-language captions, B-13 pattern); (f) research-as-drill-in (§4). Layout law: the algorithm owns geometry (dagre/ELK), local-first ~30-40 node scoped views.
- J-31 system-composition mode (docs/thefinalglance/J-plan/J-31_canvas_system_mode.md, PLANNED, well-specified, = maintainer's "connect my own systems" idea, e.g. trading system = backtester/data/live-connector nodes): a NEW 'system' canvas mode where NODE = a binding to a real plan-unit/contract/MCP-service and EDGE = a wiring contract whose verdict (declared→wired→verified→broken) is DERIVED from checker+journal (never LLM-set); ELAI builds the bound units through the normal FSM; the runtime stays the user's codebase (NOT n8n — no data flows through the canvas). Needs orchestrator/src/system_canvas.rs + a SystemContractEvent SSB + frontend renderers. Build on shipped H-13/H-15.

================================================================================
7  OPEN DECISIONS (not blocking the shell build)
================================================================================
- KNOWLEDGE tab's real name (brain / notes / …). Placeholder "KNOWLEDGE" is fine for the shell; rename is a 1-word change.
- J-31 'system' — a 5th tab vs a mode toggle inside the canvas (J-31 builds it as a mode). Canvas-phase decision.
- Exact mapping of the ~90 monitor panels into the flyout long-tail groups. Ledger has the grouping; refine at build.
- DESIGN.md §8 reconciliation note: §8 says "menu+file-explorer = zero-width flyout, never a persistent rail"; maintainer wants a thin PERSISTENT icon rail (side menu) that TRIGGERS the explorer/menu flyouts. These unify: the icon rail is the trigger (persistent, thin), the explorer/menu content is the flyout (zero-width until summoned). Capture in DESIGN.md when the shell lands.

================================================================================
8  HARD RULES FOR BUILDERS
================================================================================
- DESIGN §10 BANS hold: one teal accent, no blue/steel/amber/red/green, shape+mono-inversion for state, no glassmorphism, no generic icon-tiles, no three-identical-cards/divitis, motivated motion only + prefers-reduced-motion.
- Components use role tokens via var(--…) ONLY — no raw hex, no legacy --bg-void/--text-primary/--steel names in new/redesigned chrome. Use the new role tokens already in tokens.css.
- Font: bundled JetBrains Mono only; do NOT add an unbundled web font (Google-Fonts <link> in mockups is mockup-only).
- Keep ALL existing wiring + stores working (Tauri commands, the ~90 panels' invokes, the drag-tab/window system, fileTree, reindex, session/theme stores). This is a reskin + IA restructure, not a rewrite.
- Scope discipline per phase; surface structural blockers as a handoff (don't guess undecided IA).
- Build green: npm run check (0 new errors) + npm run build (exit 0) before declaring done.

================================================================================
9  KNOWN ISSUES / NEXT-SESSION FIXES  (shell pass, 2026-06-14 — RECORD-ONLY, not yet fixed)
================================================================================
Raised by maintainer after running the shipped shell. Do NOT fix this session; the next builder picks these up.
- I1  LIGHT MODE doesn't reach the chat + the panel/window CONTENT (the command/chat pane "pinned rules", the EXPLORE pane, the ~90 monitor panels). EXPECTED, not a bug: the shell pass deliberately left those on the LEGACY tokens (which have no [data-theme="light"] block), so they can't flip. Fix = the deferred token-unification pass (§3/§5): migrate those surfaces onto the new role tokens + add their light values. Until then, light mode flips only the new chrome.
- I2  BACK-TO-PROJECT-SELECTION button MISSING. Add a control to return to the project-selection screen (calls session.openSelector()), placed near the ELAI wordmark TOP-LEFT in the TabBar. Small, on-brand (hairline; teal on hover; underline idiom per I3). NEW requirement from maintainer.
- I3  NAV BUTTON EFFECT WRONG. The 4 tab buttons (PROGRESS/CODEBASE/FRONTEND/KNOWLEDGE) + other nav buttons render with a small round teal DOT on the left (the §4 dot-label idiom). maintainer wants INTERACTIVE nav to use the CENTER->OUT UNDERLINE effect instead — the SAME idiom the intro/selector modes + START use (no dot, no box; hover draws the line center->out; active keeps the full line). RECONCILE with DESIGN §4: the dot-prefix is for STATIC section labels ONLY; interactive nav (tabs/modes/buttons) = the underline. Apply across the shell so every screen matches the two brand screens.

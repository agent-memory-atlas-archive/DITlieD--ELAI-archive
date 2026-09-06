ELAI MAIN-WINDOWS REDESIGN — builders' brief (2026-06-15)
status: living. Pairs with workflow/design/DESIGN.md (v1.4 canon) + workflow/design/redesign-brief-2026-06-14.md (the running brief). Scope boundary: ALL code targets /workspace/ELAI\.elai_cc\crates\dashboard-v2\ (frontend). Nothing is committed — the whole redesign lands as ONE changeset when maintainer is satisfied; agents treat git as READ-ONLY.

================================================================================
0  WHAT THIS PASS IS
================================================================================
Restyle the MAIN workflow windows to DESIGN.md AND migrate them off the LEGACY token family onto the new ROLE tokens. This is the deferred "token-unification pass" (redesign-brief §3/§5) applied to the main windows. It is BOTH a visual restyle AND the fix for the light-mode gap (KNOWN ISSUE I1): the legacy tokens have NO [data-theme="light"] values, so these surfaces never flip to light; the role tokens DO flip, so migrating fixes light mode for free.

NOT in this pass: the ~60 drawer panels (drawers/*), MonitorWindow (the panel host long-tail), and the rarely-shown windows (Github/IdeaFraming/Audit/SymbolGraph). Already done (do NOT touch): the canvas trio (CanvasWindow/GraphWindow/ExploreWindow + canvas/*), the shell chrome (layout/shell/*), tokens.css, the 3 brand/selector screens.

================================================================================
1  THE TOKEN MIGRATION (the core mechanical task — do this for every styled element)
================================================================================
The canonical role tokens are DEFINED in src/styles/tokens.css — read its :root and :root[data-theme='light'] blocks to see EXACTLY what exists. Do NOT add to or edit tokens.css. Components reference role tokens via var(--…) ONLY; NO raw hex in component styles (except an unavoidable canvas/sigma literal, documented).

Replace every legacy token with its role equivalent (grep your file for legacy names first). Mapping (use judgment per DESIGN §3 elevation roles):
  --bg-void / --bg-deep / deepest fills      -> --bg (THE app bg) or --sink (a deeper inset well)
  --bg-surface / --surface / panel/card      -> --panel  (a card is an INSET well sitting below --bg, not a floating tile — §3)
  --bg-raised / hover lift                    -> --raised (the one lighter direction; transient hover only)
  list/table row                             -> --row
  --text-primary                             -> --text
  --text-secondary / body-that-must-read     -> --read
  --text-muted / labels / captions           -> --dim
  disabled / skeleton stroke                 -> --ghost
  --border / --border-* / dividers           -> --line  (hairline)  /  --line2 (stronger divider)
  --accent / interactive / active / focus    -> --d-teal   (hover/hot -> --d-teal-bright;  filled shapes -> --d-teal-deep;  rings/outlines -> --d-teal-soft;  tinted wash -> --d-teal-ghost;  faintest -> --d-teal-faint)
  --steel (the OLD second accent)            -> REMOVE. §10 bans steel/blue/second-accent. Re-map to --d-teal (if it meant "interactive") or --dim/--read (if it meant a neutral). NEVER keep a second hue.
  --info (blue info color)                   -> --dim text + a glyph (no info hue — §11)
  status colors (amber/red/green/yellow)     -> the §6 STATE LANGUAGE: shape + mono-inversion, never a foreign hue. warn=▲ in --text; fail=✕ mono-inversion (--text chip / --void glyph); done=✓ teal-deep; active=● teal pulse; pending=○ dashed. Reuse canvas/NodeStateGlyph.svelte where a node-state glyph fits.
Radii: --r-sm/--r-md/--r-lg (tight 2/4/6/10 scale, hairline borders). Motion: --dur-fast/--dur-base/--dur-slow + --ease-out/--ease-std (durations 120/180/320ms). z-index: the named --z-* scale, never magic 9999.

================================================================================
2  THE PREFERENCE LAW (what maintainer has corrected us on — bake it in, do NOT relitigate)
================================================================================
- ONE teal accent. No blue/steel/amber/red/green/purple, no second accent, ever (§10). Every neutral tinted toward teal, never dead gray, never pure #000/#FFF surfaces.
- INTERACTIVE NAV / tabs / mode buttons = the center->out UNDERLINE idiom (no dot, no box; hover draws a teal line from center via scaleX; active keeps the full line). The 5px teal DOT-prefix is for STATIC section labels ONLY (§4/§14).
- TYPE: bundled JetBrains Mono for everything (--font-mono); the hairline display (--font-disp) only for rare display moments. Visible hierarchy, max 2 families / 3 weights. NO unbundled web font.
- STATE: shape carries state, color only whispers (§6). Silence means safe — healthy elements carry ZERO badges; alarms are rare hence loud. Progress is always an explicit n/m fraction, never a lone percentage.
- ELEVATION: cards/nodes are INSET wells (--panel below --bg), rising to --raised only on hover. The dark step is small; a HAIRLINE border (--line), not a lightness delta, makes an edge read. No drop-shadow stacks, no glassmorphism, no blur.
- MOTION: motivated or absent. transform/opacity only; ease-out; honor prefers-reduced-motion (every animation resolves to its static end state).
- STATE COVERAGE (§9): every surface designs empty / loading (skeleton, nothing under ~200ms) / error (recovery path, inversion not red) / populated; dashboards add no-data vs not-yet-loaded, stale/cached, disconnected (Tauri/sidecar down), permission/empty-scope.
- CHROME LAW (§16): the shared topbar + the top-right control cluster are DONE and OFF-LIMITS; do not touch shell/*.
- §10 BANS are audited: a surviving tell (blue, second accent, status hue, system font, glassmorphism, three-identical-cards, divitis, card-in-card, ambient motion on data views, badges on healthy elements, lone percentages) is a BLOCK.

================================================================================
3  HARD RULES (every builder)
================================================================================
- RESKIN + TOKEN MIGRATION, NOT A REWRITE. Keep ALL existing wiring, stores, Tauri command invokes, props, events, and behavior working. Do not rename exports, change component contracts, or alter data flow. If a contract/data shape question arises, the docs are CONTRACT-SHEET.md / HANDLER-SHAPES.md / types.ts / stub.ts; verify a command is really registered (src-tauri/src/lib.rs) before relying on it — never invent a command name.
- DO NOT edit src/styles/tokens.css, src/lib/layout/shell/*, the canvas trio (CanvasWindow/GraphWindow/ExploreWindow) or canvas/*, or any file outside your assigned scope (a parallel agent owns the others). If a fix needs an out-of-scope edit, STOP + handoff (R-4) — do not reach across.
- MAINTAINABILITY (.claude/rules/maintainability-review.md is auto-loaded): a token migration must NOT grow an already-large file. CommandWindow/SettingsWindow/PlanWindow are >1000 lines — do not push them larger; prefer to shrink (collapse duplicate color rules into shared classes). No `as any`, no new lint suppressions.
- NO raw hex in component CSS. NO legacy --bg-void/--text-primary/--steel/--border-*/--accent/--info names left anywhere in your files when you are done (grep to confirm zero).
- prefers-reduced-motion handled for any animation you touch or add.

================================================================================
4  VERIFY + RETURN
================================================================================
- cd /workspace/ELAI\.elai_cc\crates\dashboard-v2 ; npm run check — confirm YOUR files report 0 errors (ignore pre-existing warnings/in-flight noise in files OUTSIDE your scope; the orchestrator runs the authoritative full check + build after the wave). Do NOT run npm run build (the orchestrator runs it once per wave to avoid parallel dist contention). Do NOT run cargo test.
- Self-verify before DONE: grep your files for legacy token names + raw hex + banned hues -> must be zero.
- Hold the commit (git read-only).
- Return per R-5: STATUS / FILES TOUCHED (path + what changed) / VERIFIED (your files' check status) / LEGACY-TOKENS-REMAINING (must be 0) / NOTES (max 3).

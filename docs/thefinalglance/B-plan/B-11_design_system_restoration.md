# B-11: Design System Restoration
Objective: restore the dashboard's visual identity to match
dashboard-design.txt — JetBrains Mono throughout, exact 10/12/13/15/18 type
scale, exact #0a0e12 background and brand teal #1B7A6E + brand purple
#7C5CBF, 1.6/1.4 line height. This is the highest-impact "feel" fix.
Estimated effort: ~1-2 hours (was 1 session — most of B-11 is already shipped in V2).
Pre-requisites: none. Pure frontend + token refactor.
Parallel-safe with: B-12, B-13, B-14, B-15..B-20.

## V2 REBASELINE (2026-05-07) — most of B-11 is already DONE

dashboard-v2 (`.elai_cc/crates/dashboard-v2/`) already ships the spec
visual identity per DESIGN.md and DOMAIN-CONTEXT.md:

  - JetBrains Mono is loaded via `@fontsource-variable/jetbrains-mono`
    (package.json:15) and applied across the surface.
  - Inter is NOT used as a body font (only `@fontsource-variable/inter` is
    declared for Display titles per spec).
  - Lucide icons via `@lucide/svelte` (package.json:16).
  - Workspace-primary shell at `crates/dashboard-v2/src/lib/screens/
    Workspace.svelte` (B-12 already DONE in V2 — see B-12 file).
  - Three.js + troika-three-text power the orbit (`scene/StarCanvas.svelte`,
    `orbit/OrbitCanvas.svelte` + 9 orbit components).

Re-scope B-11 to a verification + drift-backfill pass:

  1. Token audit: read whatever holds design tokens (likely
     `crates/dashboard-v2/src/app.css` or `src/lib/styles/*.css`) and
     compare every hex against the spec palette. Open a decision doc for
     anything that drifts.
  2. Type-scale audit: grep all `.svelte` files for `font-size:` and
     confirm every value is one of 10/12/13/15/18 px (xs/sm/base/lg/xl).
     Flag any 11/14/16/17/20 leftovers.
  3. Animation timing audit per DESIGN.md §7 hover-reveal + §8 orbit modes
     (constellation-breathe 11s, ambient-flow on inactive edges, dashflow
     on active branch, OrbitShootingStar every ~14s).
  4. Add a Playwright/Vitest visual regression test in
     `crates/dashboard-v2/tests/visual.spec.ts` that captures the
     workspace + drawers and compares to reference screenshots.
  5. CI gate: future drift caught at `pnpm check` time.

Path map (legacy V1 -> current V2, historical reference only — V2 is the
active codebase as of B-12; never edit V1 paths):
  [legacy app.css]   -> crates/dashboard-v2/src/styles/tokens.css
  [legacy index.html] -> crates/dashboard-v2/index.html
  [legacy src/]      -> crates/dashboard-v2/src/lib/

Skip the original "Replace Inter with JetBrains Mono" task — already done.
Skip the "Apply scale to all components" rewrite — only fix drift.

## Context

Per audit-implementation/phase-4K.md DASHBOARD QUALITY ASSESSMENT and
audit-dashboard/_AGGREGATE.md visual drift section: the dashboard shipped
with Inter font (via --font-ui), type scale +1-2px on every step, drifted
backgrounds and text colors. Result: UI looks ~25% larger than reference
and "feels generic VS-Code-ish, not the deterministic dense monospace
specified".

This is the user's primary complaint: "doesn't feel or look like I wanted".

## Reads

- F:\.ELAI_workflow\docs\specs\guide\dashboard-design.txt VISUAL IDENTITY
  section (FULL: color system with exact hex, typography scale, layout
  language, animations)
- audit-implementation/phase-4K.md DASHBOARD QUALITY ASSESSMENT
- audit-dashboard/_AGGREGATE.md "DASHBOARD VISUAL DRIFT" section
- crates/dashboard-v2/src/styles/tokens.css (design token source)
- crates/dashboard-v2/src/lib (drawer + window components)

## Tasks

1. **Token replacement.** Replace all design tokens to match spec exactly:

   COLORS (exact hex from spec):
   - bg-void:      #0a0e12 (was #0b0d12 — "deepest background, window chrome")
   - bg-base:      #0f1318
   - bg-surface:   #151a20
   - bg-hover:     #1a2028
   - bg-input:     #1e2530
   - teal:         #1B7A6E (logo color, active states)
   - teal-soft:    #1B7A6E40 (25% opacity)
   - teal-ghost:   #1B7A6E15 (8% opacity)
   - teal-bright:  #24a08e (sparingly)
   - purple:       #7C5CBF (interactive elements)
   - purple-soft:  #7C5CBF30
   - purple-ghost: #7C5CBF12
   - text-primary: #d0d7de (NOT pure white)
   - text-secondary: #6e7681
   - text-ghost:   #383e47
   - pass: #1B7A6E (teal reuse)
   - warn: #c49b2a (muted amber)
   - fail: #c44040 (muted red)
   - info: #5088c0 (steel blue)

   TYPOGRAPHY (exact from spec):
   - Primary: "JetBrains Mono", "Fira Code", monospace
   - Display: "Segoe UI" thin/200 weight (titles only)
   - Scale: xs=10px (letter-spacing 0.05em), sm=12px, base=13px, lg=15px,
     xl=18px (letter-spacing 0.15em)
   - Line height: 1.6 text, 1.4 code

   LAYOUT:
   - Borders: 1px solid bg-hover; teal only on active/focused
   - Corners: 6px panels/cards, 3px buttons/inputs
   - Shadows: NONE (depth via background steps)
   - Spacing: 8px base unit; padding 12-16px panels, 8px inline
   - Transitions: opacity 120ms ease, transform 150ms ease
   - Scrollbars: 4px wide, bg-hover track, teal-soft thumb, hidden until hover
   - Icons: Lucide (thin stroke), 16px standard, 20px sidebar, monochrome
     text-secondary, teal only when active

2. **Replace Inter with JetBrains Mono.** Search frontend for `Inter`,
   `--font-ui`, `font-family: Inter`. Replace with JetBrains Mono. Display
   titles use Segoe UI thin; everything else mono.

3. **Apply scale to all components.** Audit each component file for
   font-size declarations. Replace 11px→10px, 13px→12px, 14px→13px,
   16px→15px, 20px→18px.

4. **Line height.** Set body 1.6 / code 1.4. Currently looser per audit.

5. **Remove drift colors.** Search for any hex not in the palette above.
   Replace with closest spec color. If that loses meaning, ask user
   (decision doc).

6. **Animation discipline.** Audit animations to match spec:
   - Thinking pulse: 2px teal line, opacity 20%->50%->20%, 2s cycle
   - Step completion: brief teal-ghost flash 100ms
   - Step failure: 2px fail-color left border, fade in 200ms
   - Panel transitions: fade out 80ms, fade in 120ms
   - Progress dots: active dot scale pulse 100%->110%->100%, 3s cycle
   No bouncing, no shake, no decorative animations.

7. **Snapshot test.** Add Playwright/Vitest visual regression test that
   captures the workspace + drawers and compares to reference screenshots.
   Future drift caught at CI.

## WIRING_CONTRACT

produces_symbols:
  - app.css token vars updated (--bg-void, --bg-base, --teal, etc.)
  - JetBrains Mono loaded as @font-face
  - Lucide icon set imported
consumes_symbols:
  - all .svelte components using design tokens
reachable_from:
  - dashboard root mount
stub_ban:
  - Inter font absent (no --font-ui Inter declarations)
  - No hex colors outside the spec palette in component files
  - Type scale matches spec exactly
frontend_pair: B-11 itself
observable_effect:
  trigger: launch dashboard
  expect_ssb: n/a (visual change)
  expect_artifact: visual snapshot matches dashboard-design.txt screenshots

## DETERMINISTIC_CHECK

```bash
# 1. JetBrains Mono present, Inter absent
rg "JetBrains Mono" .elai_cc/crates/dashboard-v2/src
# Expected: ≥1 hit (font-face + body styles).
rg "Inter" .elai_cc/crates/dashboard-v2/src
# Expected: zero hits.

# 2. Spec scale present
rg "font-size:\\s*(10|12|13|15|18)px" .elai_cc/crates/dashboard-v2/src
# Expected: many hits (covering all type levels).

# 3. Off-spec colors absent
rg "#0b0d12|#0c0e13" .elai_cc/crates/dashboard-v2/src
# Expected: zero hits.

# 4. Lucide icons
rg "lucide" .elai_cc/crates/dashboard-v2/src
# Expected: ≥1 hit per component using icons.
```

## ACCEPTANCE

- [ ] All design tokens match spec hex values exactly
- [ ] JetBrains Mono loaded; Inter removed
- [ ] Type scale: 10/12/13/15/18 throughout
- [ ] Line height: 1.6 text / 1.4 code
- [ ] No off-palette colors
- [ ] Animations match spec (no bouncing/shake)
- [ ] Visual regression test in CI

## Files touched

- MODIFY: crates/dashboard-v2/src/styles/tokens.css (design token source)
- MODIFY: crates/dashboard-v2/index.html (font @import)
- MODIFY: package.json (add jetbrains-mono + lucide deps with confirmation)
- MODIFY: ~30+ .svelte components (token replacements)
- ADD: crates/dashboard-v2/tests/visual.spec.ts (Playwright)
- CREATE: workflow/reasoning/B-11-decisions.md

## Next

B-12 (Tier-0 Shell vs Workspace decision).

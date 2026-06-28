ELAI Dashboard — styles/

Four files. Import order matters: tokens.css -> base.css -> motion.css -> scrollbar.css.

TOKEN CATEGORIES (tokens.css)

  Backgrounds    --bg-void through --bg-overlay (5 steps, no shadow)
  Teal accent    --teal, --teal-bright, --teal-soft, --teal-ghost, --teal-line
  Steel accent   --steel, --steel-soft, --steel-ghost (links, info, kbd hints only)
  Text           --text-primary through --text-disabled (4 steps)
  Semantic       --pass, --warn, --fail, --info
  Semantic soft  --pass-soft/ghost, --warn-soft/ghost, --fail-soft/ghost,
                 --info-soft/ghost (badge backgrounds, hover tints)
  Borders        --border-subtle, --border-default, --border-strong
  Radius         --r-xs through --r-pill
  Spacing        --s-1 through --s-16 (8px base grid)
  Motion         --dur-fast/base/slow, --ease-out/in/std
  Typography     --font-sans, --font-mono, --font-display
  Focus ring     --ring (teal double ring)
  FSM states     --state-idle through --state-researching (badge use only)
  Gate verdicts  --gate-pass/block/warn/pending
  Rule lifecycle --rule-proposed through --rule-retired
  E-33 Wong      --signal-blue/orange/vermilion/rose/green (categorical signal layers)
  DESIGN.md v1.2 --void/--bg/--panel/--raised/--sink/--row/--text/--read/--dim/
                 --ghost/--line/--d-teal* (brand/entry screens only)

CONSUMING TOKENS IN SVELTE

  Always: color: var(--text-primary);
  Never:  color: #D7DCE3;

  In <style> blocks use var(--…) exclusively. In inline style attributes use
  style="color: var(--teal)" — never pass raw hex via props.

FORBIDDEN

  Hex literals outside :root in tokens.css.
  Purple as a general accent color. (--state-auditing is the only controlled
    exception — desaturated, FSM badge only, saturation under 25%.)
  !important outside the prefers-reduced-motion override in motion.css.
  Emojis anywhere in the codebase.
  Tailwind classes, CSS preprocessors, CSS-in-JS.
  New custom properties defined inline in components — add to tokens.css first.
  Box shadows for depth — use background step tokens instead.

INTENTIONAL HEX EXCEPTIONS (not UI chrome — data/render contexts)

  These files contain raw hex values that CANNOT use CSS custom properties
  because they are consumed outside the CSS cascade (WebGL, Canvas 2D,
  Cytoscape data styles, or gradient definitions). Each is documented in-code.

  GraphWindow.svelte     Language-identity colors for Cytoscape node styles
                         (Rust orange, TypeScript blue, etc.). These are
                         semantic data colors, not design-palette colors.
  CanvasWindow.svelte    Three.js / WebGL colors mirroring token values
                         (COLOR_HUB = '#2BBBA4' = --d-teal, etc.). WebGL
                         shaders cannot read CSS custom properties.
  StarCanvas.svelte      Canvas 2D fallback for --bg-void when reading
                         computed styles (colorMap['--bg-void'] || '#0a0e12').
  Intro.svelte           Radial gradient for the intro hero animation.
                         CSS custom properties cannot be used inside
                         radial-gradient() color stops in all renderers.

MOTION NOTES

  All keyframes live in motion.css. Utility classes: .anim-enter-fade,
  .anim-enter-lift, .anim-breath, .anim-scan, .anim-pulse-scale, .anim-shimmer.
  Stagger lists with .stagger on the parent (40ms per child, up to 12).
  The .breath shortcut in base.css is an alias for .anim-breath.

  No bounce, no spring overshoot, no shake/wobble. Scale never exceeds 1.03.
  All enters use ease-out; all exits use ease-in. Duration range: 80–240ms.

SCROLLBAR NOTES

  Thumbs are hidden until container hover (WebKit only; Firefox always shows thin).
  Use .scroll-y and .scroll-x on panels and drawers that need independent scroll.

B-11 CONFORMANCE CHECKLIST

  [x] JetBrains Mono loaded via @fontsource-variable/jetbrains-mono (main.ts)
  [x] Inter font removed (not in package.json, not in font-family declarations)
  [x] Type scale: 10/12/13/15/18px (--text-xs/sm/base/lg/xl in tokens.css)
  [x] Line height: 1.6 text / 1.4 code (--leading-text/--leading-code)
  [x] Background: #0a0e12 (--bg-void), brand teal #1B7A6E (--teal),
      brand purple #7C5CBF (--purple)
  [x] No off-spec dark background colors (the two common drift hexes
      that differ from --bg-void #0a0e12 are absent from src/)
  [x] No bounce/shake/wobble animations (motion.css: fade, lift, breath, scan)
  [x] Lucide icons via @lucide/svelte (15+ components)
  [x] Visual regression test: tests/visual.spec.ts (Playwright)
  [x] Playwright config: playwright.config.ts (CI-aware)
  [x] Semantic soft/ghost tokens added for pass/warn/fail/info
  [x] Off-palette hex literals replaced with design tokens in 16 components
  [ ] CI workflow for visual regression (requires .github/workflows/ — outside
      dashboard-v2 write scope; playwright.config.ts + package.json scripts
      are CI-ready, only the workflow YAML is missing)

ELAI Dashboard — styles/

Four files. Import order matters: tokens.css -> base.css -> motion.css -> scrollbar.css.

TOKEN CATEGORIES (tokens.css)

  Backgrounds    --bg-void through --bg-overlay (5 steps, no shadow)
  Teal accent    --teal, --teal-bright, --teal-soft, --teal-ghost, --teal-line
  Steel accent   --steel, --steel-soft, --steel-ghost (links, info, kbd hints only)
  Text           --text-primary through --text-disabled (4 steps)
  Semantic       --pass, --warn, --fail, --info
  Borders        --border-subtle, --border-default, --border-strong
  Radius         --r-xs through --r-pill
  Spacing        --s-1 through --s-16 (8px base grid)
  Motion         --dur-fast/base/slow, --ease-out/in/std
  Typography     --font-sans, --font-mono, --font-display
  Focus ring     --ring (teal double ring)
  FSM states     --state-idle through --state-researching (badge use only)
  Gate verdicts  --gate-pass/block/warn/pending
  Rule lifecycle --rule-proposed through --rule-retired

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

MOTION NOTES

  All keyframes live in motion.css. Utility classes: .anim-enter-fade,
  .anim-enter-lift, .anim-breath, .anim-scan, .anim-pulse-scale, .anim-shimmer.
  Stagger lists with .stagger on the parent (40ms per child, up to 12).
  The .breath shortcut in base.css is an alias for .anim-breath.

SCROLLBAR NOTES

  Thumbs are hidden until container hover (WebKit only; Firefox always shows thin).
  Use .scroll-y and .scroll-x on panels and drawers that need independent scroll.

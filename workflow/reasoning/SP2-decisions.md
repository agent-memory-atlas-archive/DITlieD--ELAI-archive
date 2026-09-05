SP2 UI PRIMITIVES - DECISION DOC (s-73316)

What I will implement (CSS-only + 1 markup span-wrap; clean reused primitives = high leverage):
  Tree.svelte .tree-label, CommandRow.svelte .cmd-row__label, BranchTree.svelte .col-name: add min-width:0
    (flex/grid children that already carry overflow/ellipsis/nowrap but lack the shrink-floor, so they actually
    truncate instead of forcing the row wider).
  Select.svelte:
    - add a .select-value rule (flex:1;min-width:0;overflow:hidden;text-overflow:ellipsis;white-space:nowrap) so the
      selected label truncates; today only .select-value--placeholder (color) exists.
    - wrap {opt.label} in <span class="select-option__label"> + add the truncation rule on that span. The option is a
      display:flex <div> with DIRECT text, and text-overflow:ellipsis does not render on a flex container with an
      anonymous text item; a block/inline-block child is required for the ellipsis to paint (Chromium/WebView2).
  Toggle.svelte: gap:8px -> var(--s-2); width:32px -> var(--s-8) (the only Toggle px with exact grid tokens; left the
    18/14/2px coupled track/thumb geometry alone per explorer warning - half-tokenizing coupled geometry is risky).

Key technical decision: exact, convention-matching edits only. Did NOT bulk-swap every px->token across Input/Select/Tree/
  BranchTree/Button (not defects, pure diff bloat, agent flagged coupled geometry). Did NOT add overflow:hidden to
  .select-option (the label span handles truncation; option-level overflow is unnecessary). S2 (zero deprecated on: directives),
  raw hex (zero), and focus (global :focus-visible ring at tokens.css 341-361 already covers every interactive primitive,
  incl. role+tabindex div controls) all VERIFIED clean by both agents - nothing to invent.

Expected test outcome: svelte-check stays at 0 errors; no new warnings (CSS additions + one benign span wrap).

Unknowns to verify empirically:
  1. .select-value is a <span> flex child of .select-trigger - VERIFIED L86.
  2. .select-option is display:flex with direct {opt.label} text - VERIFIED L95-107; span-wrap needed for true ellipsis.
  3. --s-8 == 32px, --s-2 == 8px - VERIFIED tokens.css L94/L89.

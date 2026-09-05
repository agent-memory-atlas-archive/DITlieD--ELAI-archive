SP1 LAYOUT CHROME - DECISION DOC (s-73316)

What I will implement (CSS-only, clean files, frontend-only):
  StatusBar.svelte .project(L214) + .model-label(L264): add min-width:0;overflow:hidden;text-overflow:ellipsis.
    Grounded: .project is a direct flex child of .cluster (display:flex, L181/L85-87); .model-label is a direct flex
    child of .verbose (display:flex, L192-193/L133); both already nowrap via .mono(L207-212). So ellipsis applies and
    the long variable strings (project name, model id) shrink while fixed badges (state/mode/gate) hold size.
  CommandPalette.svelte .item-label(L391-398): add min-width:0 (flex:1 1 auto child needs it to shrink below content
    for ellipsis; .item-kbd is flex-shrink:0 so it is unaffected).
  Sidebar.svelte svelte:head tooltip block(L489-516): 6 color/font swaps to tokens.css vars -
    #5A7A9E->--steel, rgba(90,122,158,0.08)->--steel-ghost, rgba(255,255,255,0.07)->--border-default,
    'JetBrains Mono'..->--font-mono, #D7DCE3->--text-primary, #10141A->--bg-base (closest; no exact token).
    Leave px/ms/opacity/z-index (not hex, not rule violations).

Key technical decision: minimal CSS-only edits matching the existing CommandPalette .item-label idiom + tokens.css.
  Leave deliberate floors UNTOUCHED (verified by context-compiler): AppShell:202 (.panels-area already min-width:0),
  WindowHost:131 (min-height:0), Panel:375 (.panel min-width:320px floor), coupling literals 48/22/320px,
  Panel:462 .tab-title (deliberate scroll-not-truncate).

Expected test outcome: svelte-check stays at 0 errors (CSS-only, no type/markup change); no new warnings.

Unknowns to verify empirically:
  1. .project/.model-label are flex children - VERIFIED via markup+style read (L85-87, L133, L181, L192).
  2. #10141A->--bg-base is a near-match not exact (rgb 16,20,26 vs 15,19,24) - imperceptible, preserves appearance.
  3. .item-label min-width:0 does not disturb .item-kbd - .item-kbd is flex-shrink:0 (verified by context-compiler).

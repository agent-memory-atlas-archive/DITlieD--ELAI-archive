SP4 DRAWER DATA-PANELS - DECISION DOC (s-73316)

What I will implement (frontend-only, CLEAN drawers; controller-applied):
  IncidentsDrawer.svelte - S4 (HEADLINE): callInvoke currently swallows a real invoke error -> returns undefined -> reload()
    maps undefined to loadError=null, so a backend failure on the security-incidents surface renders as the benign
    "No incident rows recorded yet" empty state (the .error-banner is unreachable). Fix: callInvoke returns undefined ONLY for
    no-window / no-Tauri (benign); a real invoke error PROPAGATES; reload() wraps the call in try/catch/finally and sets
    loadError=String(e) ONLY on a true throw. Preserves browser-dev behavior (no-Tauri -> empty, NO spurious error banner).
    callInvoke sole caller = reload() (grep-confirmed L81), so changing its contract is safe.
  ReasoningDrawer.svelte - S1: add min-width:0 to .mini-critic-reason (L731, flex:1 1 auto child w/ ellipsis) and
    .reflect-tl-detail (L650, flex child of .reflect-tl-row w/ ellipsis).
  OrchestrationDrawer.svelte - S1: add overflow-wrap:anywhere to .credit-table td (L1159) so long task_id/role wrap inside the
    width:100% table instead of forcing horizontal overflow (tables don't ellipsize cleanly; wrap is the right idiom).
  ResearchDrawer.svelte - S1: add `.rn-stage-row > * { min-width:0; overflow:hidden; text-overflow:ellipsis; white-space:nowrap; }`
    so the grid 1fr stage column truncates instead of blowing out the track.
  CosmosDrawer.svelte - S1: add min-width:0 to .name (L241, flex child of .row-meta, already has ellipsis).

NOT touched (grounded resolution of an agent conflict): KnowledgeDrawer.svelte - the context-compiler flagged its 4 catches
  (L88-111) as silent-S4, but a direct source read REFUTED this: each catch backs an ../api/stub wrapper that already returns a
  meaningful known:false / empty fallback shape ("unknown" is an intended, user-meaningful graceful-degradation state, not a
  swallowed error). The integration-explorer correctly excluded it. No fix; documented as a non-defect.

Deferred (documented follow-up): full S5 drawer token re-anchor (MemoryNotesPanel 27 hex, MemoryFactsDrawer 24 hex, etc);
  all git-hot drawers.

Expected test outcome: svelte-check stays at 0 errors; effective warning baseline 18; no new warnings.

Unknowns verified empirically:
  1. callInvoke sole caller is reload() - grep-confirmed (L63 def, L81 call).
  2. IncidentsDrawer gates on window.__TAURI_INTERNALS__ (L68) so the no-Tauri path stays benign after the fix.
  3. All 5 S1 targets are genuine flex/grid children (read L644-655, L731-737, L1144-1161, L452-466, L235-247).

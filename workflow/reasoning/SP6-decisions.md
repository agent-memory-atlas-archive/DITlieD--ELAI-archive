SP6 VIEWS + SCREENS - DECISION DOC (s-73316) [FINAL sub-phase]

Inventory (glob-confirmed): views = {ArchiveExplorerView, LineageGraphView, ModeRoutingDashboardView}; screens =
  {CloudPanel, ProjectSelector, RemoteSidecarBootOverlay, Workspace}. All clean (no git-hot overlap).

What I will implement (controller-applied, ArchiveExplorerView.svelte only):
  D4 S3 (poll-error surfacing): callInvoke (L59-70) swallowed errors (returned undefined for BOTH no-Tauri and a thrown
    invoke) and reload() (L72-82, called by the $effect initial load + 10s poll at L99/L100) always set loadError=null - so a
    backend failing AFTER the first successful load was invisible; only the manual Reload button (reloadWithErrorSurface, L84/
    L161) surfaced errors. Fix: callInvoke now lets a real invoke error PROPAGATE (returns undefined only for no-window/no-Tauri);
    reload() wraps in try/catch/finally and sets loadError=String(e) only on a true throw. Identical to the SP4 IncidentsDrawer
    fix. No orphaning/removal: callInvoke<-reload<-$effect and reloadWithErrorSurface<-manual button all remain used.
  D2 S1: .ae-side dd (L377, holds the full template id + content hash inside a fixed 22rem aside) -> add min-width:0 + overflow-wrap:anywhere.
  D3 S1: .ae-list th,td (L333, long domain/pattern_kind tokens) -> add overflow-wrap:anywhere.

SKIPPED (grounded resolutions):
  D1 ProjectSelector .project-path - CONFLICT: integration-explorer claimed it lacks min-width:0; context-compiler (full read
    of all 7 files) said ProjectSelector is already min-width:0/ellipsis-hardened (.project-path L399-404 has overflow:hidden +
    white-space:nowrap). Not acted on without resolution (additive min-width:0 would be harmless but unverified) - possible follow-up.
  D5 CloudPanel redaction_count "0"->"-" - based on an UNVERIFIED RAG-surfaced "C-4/s-32952 decision"; changing a display value
    on unverified grounds violates grounded-by-default. Skipped.
  ModeRoutingDashboardView missing empty-arm - soft gap (renders dashes on empty, does NOT throw); low value, deferred.
  S5 rem/hex (~64 raw literals across the 7 files: MRDV/CloudPanel/Overlay hex, ArchiveExplorer/Lineage rgba) - deferred mechanical follow-up.

Expected test outcome: svelte-check stays at 0 errors / 18 warnings; tsc clean (callInvoke catch(e) is unknown, String(e) coerces - same as IncidentsDrawer).

Unknowns verified empirically: callInvoke sole caller = reload() [grep L73]; reload() callers = $effect L99/L100; reloadWithErrorSurface = manual Reload L161 [grep]; .ae-side dd (L377) and .ae-list th,td (L333) rules read.

SP3 COMPONENT DATA-PANELS - DECISION DOC (s-73316)

What I will implement (delegated to a fixer agent; CLEAN files only, frontend-only):
  GatesPanel.svelte - S3 anti-pattern: convert the raw top-level `import { invoke }` + bare invoke(...) call to the
    ArtifactsPanel-style guarded dynamic-import (await import('@tauri-apps/api/core') in try/catch; on failure fall to the
    EXISTING empty/error arm). Prevents the throw in Vite browser-dev mode. Preserve existing error/loading/empty arms.
  CostPanel.svelte - S3 anti-pattern: same guarded-invoke conversion (raw invoke L13).
  McpStatusPanel.svelte - S2: on:click (L54) -> onclick (deprecated Svelte 4 directive under runes:true). PLUS, if confirmed
    by reading the source: convert plain `let` reactive state (statuses/loading, ~L16-18) -> $state(...) - under runes:true a
    reassigned top-level `let` read in the template is NON-reactive, so the panel may not update; $state is the idiomatic fix.
  ContextStatePanel.svelte - NO change (well-wired; concrete $state defaults; convention (a), needs no ladder).

Key technical decision: delegate the script-level refactor to a fixer agent (multi-file + needs the ArtifactsPanel exemplar
  pattern + context-package-SP3-components.md, which the controller must not read per the context-delegation HARD rule).
  Deferred to the documented S5 follow-up: RuntimePanel (48 hex) + CompactionLadderPanel hex re-anchor (mechanical, low-UX,
  risky - not this run). S4 swallowed-errors = NONE (all 4 panels surface caught IPC errors).

Expected test outcome: svelte-check stays at 0 errors; ideally REMOVES the on:click deprecation warning (and any
  non-reactive-let warnings if McpStatusPanel let->$state is applied) from the 22-warning baseline. No NEW errors.

Unknowns to verify empirically (fixer must read the actual source):
  1. McpStatusPanel `let`s are genuinely reassigned-and-template-read (truly non-reactive) before converting to $state.
  2. Guarded-invoke conversion preserves Tauri-runtime behavior (invoke still called, just lazily imported).
  3. Reconcile agent conflict: context-compiler said GatesPanel lacks loading/Empty arms; explorer REFUTED (error + folded
     loading/empty already present). Fixer treats ONLY the unguarded invoke as the defect; does NOT add redundant arms.

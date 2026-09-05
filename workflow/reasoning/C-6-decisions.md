C-6 Dashboard Design Token Alignment — Decision Record
Session: s-32952
Date: 2026-05-17
Plan: docs/thefinalglance/C-plan/C-6_dashboard_design_tokens.md
Context package: workflow/sessions/s-32952/context-package-C6.md (11 DRIFT items pre-resolved)

================================================================
1. TOKEN HEX VALUES + SPEC SOURCE
================================================================
Spec reference: F:\.ELAI_workflow\docs\specs\guide\dashboard-design.txt lines 42-83.

Backgrounds (tokens.css :root):
  --bg-void     #0a0e12   (was #07090B)
  --bg-base     #0f1318   (was #0B0E12)
  --bg-surface  #151a20   (was #10141A)
  --bg-hover    #1a2028   (was #161B22)
  --bg-input    #1e2530   (was #1A2029)   [DRIFT-5 resolved: spec-listed, plan omitted; included]
  --bg-overlay  rgba(10,14,18,0.72)        [rebased on new --bg-void for consistency]

Teal (tokens.css :root):
  --teal         #1B7A6E                   (was #1E8A7A)
  --teal-bright  #24a08e                   (was #2BAE98)
  --teal-soft    rgba(27,122,110,0.25)     (was rgba(30,138,122,0.28))
  --teal-ghost   rgba(27,122,110,0.08)     (was rgba(30,138,122,0.10))   [DRIFT-4 resolved: plan omitted, spec-listed; included to keep ghost on the new hue]
  --teal-line    rgba(27,122,110,0.45)     (was rgba(30,138,122,0.45))   [rebased; same opacity]

Text (tokens.css :root):
  --text-primary    #d0d7de   (was #DCE5E8)
  --text-secondary  #6e7681   (was #95A1A8)
  --text-ghost      #383e47   (was #5E6B73)   [DRIFT-3 resolved: token was PRESENT with wrong value; updated, not added]

PLAN-EXCLUDED (left as-is per scope):
  --pass / --warn / --fail / --info still hold pre-C-6 hex values. B-11 report flags semantic-token drift but plan
  task 2-5 does not cover it; C-7 or a follow-up patch can reconcile.

================================================================
2. FONT CHOICE RATIONALE
================================================================
Body font: --font-sans now resolves to 'JetBrains Mono', monospace.
Rationale (spec dashboard-design.txt:75): the spec body face IS JetBrains Mono — monospaced terminal-style typography is
the design system's foundational identity. Inter was a B-pre era stub that drifted across iterations.
@fontsource-variable/jetbrains-mono is bundled via main.ts:2 (verified), so the monospace fallback almost never engages.
Fira Code appears only in --font-mono (code blocks), not --font-sans.

Display font: --font-display now 'Segoe UI Variable', 'Segoe UI', sans-serif. font-weight: 200 (thin) is applied
at usage sites (Sidebar wordmark, base.css .display), not in the token, per the C-6 plan's deliberate split.

Token naming deviation (DRIFT-11 resolved): --font-sans now contains a monospace family — the name no longer matches
the value's classification. Renaming would cascade through every component that references var(--font-sans).
Decision: keep --font-sans, document the semantic deviation in tokens.css top-of-file comment block. Future cosmetic
rename can happen as a single sed pass once the rest of the design system is frozen.

================================================================
3. SIDEBAR — WIDTH AND ICON COUNT
================================================================
Width (DRIFT-6 resolved): Sidebar.svelte:213 .sidebar width 64px -> 48px. Sidebar.svelte:274 .nav-btn width 64px -> 48px.
Coupled change: AppShell.svelte:186 grid-template-columns 64px -> 48px. Without the coupled change a 16px void appears
between sidebar and primary panel — spec-visual acceptance fails. Plan omitted this; orchestrator decision injected it.

The AppShell drop-strip width references (lines 216, 225) intentionally retain 64px width — that is the *drag-target
hint strip* on the right edge of the panels area, not the sidebar column, and 64px is the empirically-tuned discoverable
width (per existing inline comment). Decoupled from sidebar width.

Icon count (DRIFT-7 resolved, updated C-6-EXECUTE-8): Sidebar.svelte renders 8 nav icons:
  command, plan, activity, review, monitor, settings, graph, explore
Spec dashboard-design.txt:195-201 lists 5:
  terminal=command, layers=plan, play=activity, search=explore, settings
Surplus (spec-extra in dashboard): review, monitor, graph (3 extras).
Mapped to spec: command=terminal, plan=layers, activity=play, explore=search, settings=settings.
The explore icon (Compass) was added after the original C-6 decision doc; it maps to spec [search]
and is no longer missing. A SPEC-DELTA comment block is now present in Sidebar.svelte above the
navItems array documenting the 3 surplus icons (review, monitor, graph) with inline per-item comments.

Decision: KEEP all 8 current icons unchanged. Do NOT hide any icon.
Rationale:
  - Hiding icons under display: none is invisible to users but breaks click-shortcut counting (Ctrl+1..8).
  - The actual delta is 3 surplus, not "2 surplus" as the plan assumed (plan said count=7; actual count=8).
  - The full icon-set reconciliation (deciding which 3 to cut) requires UX audit + keyboard shortcut
    renumbering, both well outside the C-6 cosmetic scope.
  - Defer the full icon-set reconciliation to C-7 (UX audit).
SPEC-DELTA comment block added in Sidebar.svelte (per plan task 8).

================================================================
4. STOP BUTTON WIRING POINT
================================================================
Stop button wires to commands.cancelStream() exposed at:
  crates/dashboard-v2/src/lib/state/commands.svelte.ts:507 (export)
  -> commands.svelte.ts:465 (function body)
which already calls Tauri cancel_stream(activeStreamId).
No new IPC command. No new state mutation. The Stop button is purely a UI surface for an existing handler.

Render condition: {#if commands.isStreaming} ... {:else if inputValue.trim() > 0} <Send/> {/if}
This is mutually-exclusive rendering (DRIFT-9 resolved): Send and Stop never appear simultaneously, eliminating the
"both visible" ambiguity when streaming begins on a still-populated textarea.

Style: --teal-soft background, --teal-line border, --text-primary text, 28px height (matches Send). Square icon
from @lucide/svelte (already a peer of Send and Slash in the existing import). data-testid="stop-button" added for
future vitest coverage per plan tests section.

================================================================
5. CTRL+ENTER BINDING DECISION
================================================================
Plan task 10 says "Do not prevent default on plain Enter (preserve newline behavior)".
The implementation follows this literally: Ctrl+Enter submits, plain Enter inserts a newline.

Why plain Enter is NOT intercepted:
  - Multi-line prompt editing is essential. Users frequently paste code snippets, stack traces,
    or multi-paragraph instructions that require newlines within the textarea.
  - If plain Enter were intercepted for submit, users could not insert newlines without using
    Shift+Enter, which is a non-obvious modifier for a text input.
  - The spec (dashboard-design.txt:852) explicitly calls out "Ctrl+Enter Send message (in CHAT)",
    implying plain Enter is reserved for text editing, not submit.
  - The slash-palette path (when open) DOES intercept Enter for autocomplete/submit, but that is
    a modal context — once the palette closes, Enter returns to newline behavior.

Implementation in handleKeydown (CommandWindow.svelte:291-341):
  - When slash palette is open: Enter (no shift) -> autocomplete or submitInput() (line 305-322)
  - When slash palette is closed: Ctrl+Enter -> submitInput() (line 334-339); plain Enter falls
    through with no preventDefault, so the textarea inserts a newline (default browser behavior).
  - Both the Ctrl+Enter path and the slash-palette Enter path call the same submitInput() function
    as the Send button — no duplicate fetch/IPC.

Aria-label on textarea: "Press Ctrl+Enter to send" (reflects the actual binding).

================================================================
6. ITEMS DEFERRED / NOT EXECUTED
================================================================
- B-11 DETERMINISTIC_CHECK path fix (plan task 11): VERIFIED — no fix needed.
  B-11_design_system_restoration.md DETERMINISTIC_CHECK (lines 162-180) already uses
  .elai_cc/crates/dashboard-v2/src paths throughout. No V1 paths (crates/dashboard/src
  or crates/elai-dashboard) present. Acceptance criterion met by existing state.
- Cloud-Sonnet smoke test: deferred. The plan calls for a live `elai /task --mode cloud "implement a quick hello world"`
  run with manual dashboard observation of Stop button + Ctrl+Enter + DevTools computed styles. This requires a running
  dashboard process, cloud OAuth, and human verification — not executable in this orchestration.
- Vitest test for CommandWindow (plan Tests section): deferred. dashboard-v2 has no existing vitest config; adding test
  tooling is a new dependency category and would require user approval per CLAUDE.md "No new dependencies without
  confirmation". data-testid="stop-button" was added so future tests can target the button cleanly.
- Semantic palette --pass/--warn/--fail/--info: out of plan task 2-5 scope. B-11 verification flags it; recommended
  follow-up patch-N.

================================================================
7. VERIFICATION
================================================================
- grep -rn "11px" .elai_cc/crates/dashboard-v2/src/ : 0 occurrences (sweep across 39 files, 157 replacements).
- npm run check (svelte-check + tsc --noEmit) : EXIT 0, 0 ERRORS, 13 pre-existing warnings unrelated to C-6 files.
- All token literals match acceptance criteria hex values exactly.
- Sidebar.svelte: 64px occurrences in source = 0; 48px present (line 215 and line 274). AppShell.svelte line 186
  grid-template-columns updated.
- CommandWindow.svelte: Square icon imported, Stop button block present, Ctrl+Enter branch present in handleKeydown,
  aria-label updated, .stop-btn CSS class defined with 28px height and --teal-soft bg.

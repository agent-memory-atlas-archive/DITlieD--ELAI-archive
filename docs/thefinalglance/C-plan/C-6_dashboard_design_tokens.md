# C-6: Dashboard Design Token + Font + Control Alignment Pass
Objective: reconcile all CSS design tokens, body font, 11px occurrences,
sidebar width/icon count, and add CommandWindow Stop button + Ctrl+Enter
binding to match the finalglance design spec exactly.
Estimated effort: 1 session.
Pre-requisites: none (pure dashboard-v2 work, no backend changes).
Parallel-safe with: C-5 (cloud mode, different files), C-7 (different UX
areas).

## Context

The dashboard-v2 design system diverged from the finalglance spec across
several token files and Svelte components during rapid B-plan iterations.
Verification reports B-11, B-12, B-14, and dashboard-design.report.md all
flag outstanding deltas:

- tokens.css carries wrong --bg-* hex values and the wrong body font
  family (Inter/system-sans instead of JetBrains Mono).
- Teal palette is inconsistent: --teal-bright and --teal-soft use
  off-spec values in some component overrides.
- Text token --text-ghost is absent or mis-valued.
- Seven Svelte files use 11px for icon labels / captions; spec floor is
  10px (accessibility minimum per finalglance guide).
- Sidebar.svelte width is 64px; spec says 48px. Icon count is 7; spec
  requires 5 (surplus must be documented, not silently left).
- CommandWindow.svelte has no Stop button while streaming and no
  Ctrl+Enter keyboard shortcut for submit.

These are all cosmetic/UX deltas with no backend wire implications.
The verification reports listed under "Affected verification reports"
block sign-off on B-11 and related phases until this pass is done.

## Reads

- F:\.ELAI_workflow\docs\thefinalglance\verification\reports\B-plan\B-11_design_system_restoration.report.md
- F:\.ELAI_workflow\docs\thefinalglance\verification\reports\B-plan\B-12_shell_vs_workspace_decision.report.md
- F:\.ELAI_workflow\docs\thefinalglance\verification\reports\B-plan\B-14_live_token_streaming_chat.report.md
- F:\.ELAI_workflow\docs\thefinalglance\verification\reports\guide\dashboard-design.report.md
- .elai_cc/crates/dashboard-v2/src/styles/tokens.css (current state)
- .elai_cc/crates/dashboard-v2/src/lib/components/Sidebar.svelte (width + icon list)
- .elai_cc/crates/dashboard-v2/src/lib/windows/CommandWindow.svelte (streaming state + keybindings)
- .elai_cc/crates/dashboard-v2/src/lib/windows/ArtifactsPanel.svelte (11px sites)
- .elai_cc/crates/dashboard-v2/src/lib/windows/IndexingProgressPanel.svelte (11px sites)
- .elai_cc/crates/dashboard-v2/src/lib/windows/MediaValidationPanel.svelte (11px sites)
- .elai_cc/crates/dashboard-v2/src/lib/components/AntiStalenessDrawer.svelte (11px sites)
- .elai_cc/crates/dashboard-v2/src/lib/components/CosmosDrawer.svelte (11px sites)
- .elai_cc/crates/dashboard-v2/src/lib/components/KnowledgeDrawer.svelte (11px sites)
- .elai_cc/crates/dashboard-v2/src/lib/windows/MonitorWindow.svelte (11px sites)

## Tasks

1. **Read all verification reports** listed under Reads. Extract every
   open delta related to tokens, fonts, font-size, sidebar, and
   CommandWindow. Map each delta to the target file and line range.
   Do not edit yet.

2. **Patch tokens.css — background tokens**: set
   --bg-void: #0a0e12;
   --bg-base: #0f1318;
   --bg-surface: #151a20;
   --bg-hover: #1a2028;
   Confirm no other file locally overrides these via inline style or
   scoped :root block. Grep dashboard-v2/src before writing.

3. **Patch tokens.css — teal tokens**: set
   --teal: #1B7A6E;
   --teal-bright: #24a08e;
   --teal-soft: rgba(27,122,110,0.25);
   Same pre-write grep for overrides.

4. **Patch tokens.css — text tokens**: set
   --text-primary: #d0d7de;
   --text-secondary: #6e7681;
   --text-ghost: #383e47;
   Add --text-ghost if absent; do not remove any existing token.

5. **Patch tokens.css — font tokens**: change
   --font-sans to "JetBrains Mono", monospace;
   --font-display to "Segoe UI Variable", "Segoe UI", sans-serif;
   (thin weight applied via font-weight: 100 at usage sites, not in
   the token itself).
   Verify the body rule references --font-sans and uses no direct
   font-family override that would shadow the token.

6. **11px sweep — 7 files**: for each file in the list below, grep
   all occurrences of 11px, replace with 10px. One edit per file,
   review surrounding context first to confirm each occurrence is a
   font-size or line-height (not a border-radius or padding value
   that coincidentally equals 11px).
   Files: ArtifactsPanel.svelte, IndexingProgressPanel.svelte,
   MediaValidationPanel.svelte, AntiStalenessDrawer.svelte,
   CosmosDrawer.svelte, KnowledgeDrawer.svelte, MonitorWindow.svelte.

7. **Sidebar.svelte — width**: change width from 64px to 48px in the
   sidebar container style. Grep for all width references in the file
   first; ensure icon size and padding values are still proportional
   (icon size ≤ 20px, padding ≤ 8px each side fits in 48px).

8. **Sidebar.svelte — icon count**: read current icon list. If count
   is 7 and spec requires 5, identify the 2 surplus icons. Do NOT
   remove them silently. Add a comment block:
   // SPEC-DELTA: icons <name1> and <name2> present but not in
   // finalglance spec. Retained for feature parity; remove when
   // C-7 UX audit resolves icon set.
   Document the same delta in workflow/reasoning/C-6-decisions.md.

9. **CommandWindow.svelte — Stop button**: when the Svelte store or
   prop `commands.isStreaming` (or equivalent streaming flag) is true,
   render a Stop button adjacent to the send button. On click, call
   the existing abort/cancel handler (do not create a new IPC command;
   wire to whatever stop mechanism already exists — grep before
   assuming). Button label: "Stop". Style: --teal-soft background,
   --text-primary text, same height as send button.

10. **CommandWindow.svelte — Ctrl+Enter binding**: on the prompt
    textarea, intercept keydown. If event.ctrlKey && event.key ===
    "Enter" and the textarea is not empty and not already streaming,
    call the same submit handler as the send button. Do not prevent
    default on plain Enter (preserve newline behavior). Add
    aria-label="Press Ctrl+Enter to send" to the textarea.

11. **B-11 DETERMINISTIC_CHECK path fix**: open
    F:\.ELAI_workflow\docs\thefinalglance\B-plan\B-11_design_system_restoration.md
    (or equivalent path). Locate the DETERMINISTIC_CHECK section.
    If any check path points to a V1 dashboard path (e.g.
    crates/dashboard/src/ or crates/elai-dashboard/), rewrite those
    paths to crates/dashboard-v2/src/. One targeted edit per wrong
    path; do not alter any other section of that file.

12. **Decision doc**: write
    workflow/reasoning/C-6-decisions.md recording:
    - token hex values and their spec source reference
    - font choice rationale (JetBrains Mono as body font)
    - sidebar icon surplus decision (document, not delete)
    - Stop button wiring point (which handler was found)
    - Ctrl+Enter: why plain Enter is not intercepted

## WIRING_CONTRACT

produces_symbols:
  - CSS custom properties: --bg-void, --bg-base, --bg-surface,
    --bg-hover, --teal, --teal-bright, --teal-soft, --text-primary,
    --text-secondary, --text-ghost, --font-sans, --font-display
  - CommandWindow Stop button (DOM element, no new IPC)
  - CommandWindow Ctrl+Enter keyboard handler
consumes_symbols:
  - commands.isStreaming (or equivalent Svelte store / prop in
    CommandWindow.svelte — verify exact name via grep before wiring)
  - existing submit handler in CommandWindow.svelte
  - existing abort/cancel handler in CommandWindow.svelte
reachable_from:
  - all dashboard-v2 Svelte components (via CSS cascade from :root)
  - CommandWindow keyboard and button events
stub_ban:
  - Stop button MUST wire to a real existing abort handler; no
    console.log stub accepted
  - Ctrl+Enter MUST call the same code path as the Send button click;
    no duplicate fetch/IPC call
frontend_pair: tokens.css is the sole CSS source of truth for these
  tokens; no component-level :root overrides permitted after this pass
observable_effect:
  trigger: open dashboard, open CommandWindow, start a streaming task
  expect_visual: Stop button visible during stream; disappears after
                 stream ends; Ctrl+Enter sends prompt
  expect_computed: DevTools computed --bg-void = #0a0e12;
                   body font-family includes JetBrains Mono

## DETERMINISTIC_CHECK

```bash
# 1. Background tokens correct in tokens.css
grep -E "bg-void|bg-base|bg-surface|bg-hover" .elai_cc/crates/dashboard-v2/src/styles/tokens.css

# 2. Teal tokens correct
grep -E "\-\-teal[^-]|\-\-teal-bright|\-\-teal-soft" .elai_cc/crates/dashboard-v2/src/styles/tokens.css

# 3. Text tokens present
grep -E "\-\-text-primary|\-\-text-secondary|\-\-text-ghost" .elai_cc/crates/dashboard-v2/src/styles/tokens.css

# 4. Font token body rule — no Inter or bare sans-serif in body rule
grep -n "font-family" .elai_cc/crates/dashboard-v2/src/styles/tokens.css | grep -E "Inter|sans-serif"
# Expected: 0 matches in the body rule; --font-sans value contains JetBrains

# 5. 11px fully purged from all 7 target files
grep -rn "11px" .elai_cc/crates/dashboard-v2/src/ | wc -l
# Expected: 0

# 6. Sidebar width is 48px
grep -n "width" .elai_cc/crates/dashboard-v2/src/lib/components/Sidebar.svelte | grep "64px"
# Expected: 0 (64px removed)
grep -n "48px" .elai_cc/crates/dashboard-v2/src/lib/components/Sidebar.svelte
# Expected: ≥1 match

# 7. Stop button present in CommandWindow
grep -n "Stop" .elai_cc/crates/dashboard-v2/src/lib/windows/CommandWindow.svelte
# Expected: ≥1 match (button label or aria)

# 8. Ctrl+Enter handler present
grep -n "ctrlKey" .elai_cc/crates/dashboard-v2/src/lib/windows/CommandWindow.svelte
# Expected: ≥1 match

# 9. svelte-check passes (no type errors introduced)
cd .elai_cc/crates/dashboard-v2 && npx svelte-check --output machine 2>&1 | grep -c "^ERROR" || true
# Expected: 0

# 10. Decision doc written
test -f workflow/reasoning/C-6-decisions.md && echo "PRESENT" || echo "MISSING"
```

## ACCEPTANCE

- [ ] tokens.css: --bg-void=#0a0e12, --bg-base=#0f1318,
      --bg-surface=#151a20, --bg-hover=#1a2028 (exact hex, case-insensitive)
- [ ] tokens.css: --teal=#1B7A6E, --teal-bright=#24a08e,
      --teal-soft=rgba(27,122,110,0.25)
- [ ] tokens.css: --text-primary=#d0d7de, --text-secondary=#6e7681,
      --text-ghost=#383e47 (added if absent)
- [ ] tokens.css: --font-sans contains "JetBrains Mono"; body rule
      references --font-sans; grep for Inter in body rule returns 0
- [ ] grep -rn "11px" .elai_cc/crates/dashboard-v2/src/ | wc -l returns 0
- [ ] Sidebar width 64px removed, 48px applied; icon count delta
      documented in comment block and C-6-decisions.md
- [ ] CommandWindow Stop button renders while commands.isStreaming is
      true; verified by smoke test below
- [ ] CommandWindow Ctrl+Enter sends prompt via same handler as Send button
- [ ] B-11 DETERMINISTIC_CHECK paths updated to dashboard-v2 (no V1 paths)
- [ ] svelte-check: 0 ERRORs introduced by this pass
- [ ] workflow/reasoning/C-6-decisions.md present with all 5 rationale items

Cloud-Sonnet smoke test (manual, run after all edits):
  elai /task --mode cloud "implement a quick hello world in main.rs"
  In dashboard:
    - Open CommandWindow while task is streaming.
    - Confirm Stop button is visible.
    - Press Stop; confirm streaming halts.
    - In a fresh CommandWindow, type a prompt, press Ctrl+Enter;
      confirm it sends (streaming begins without clicking Send).
    - While streaming, inspect computed styles in DevTools:
      --bg-void must equal #0a0e12
      body font-family must list JetBrains Mono first

## Tests

- .elai_cc/crates/dashboard-v2/src/lib/windows/CommandWindow.svelte:
  add a Svelte test (vitest or component test) verifying:
  (a) Stop button has attribute data-testid="stop-button" and is
      conditionally rendered when isStreaming prop is true
  (b) keydown event with ctrlKey+Enter on the textarea dispatches
      the submit handler exactly once when input is non-empty
- If a dashboard-v2 test suite already exists (check for vitest.config.ts
  or __tests__ directory), add to it; otherwise create
  .elai_cc/crates/dashboard-v2/src/__tests__/CommandWindow.test.ts
  using existing test tooling only (no new dependencies without
  human confirmation).

## Out of scope

- No changes to Rust orchestrator, IPC protocol, or Tauri commands.
- No new IPC commands for Stop; wire to existing abort handler only.
- No changes to any file outside .elai_cc/crates/dashboard-v2/ except
  the B-11 plan DETERMINISTIC_CHECK path fix and workflow/reasoning/C-6-decisions.md.
- Icon removal from Sidebar is explicitly deferred to C-7; this pass
  only documents the surplus.
- No font installation or CDN changes; JetBrains Mono must already be
  available in the dashboard build (verify via grep of index.html or
  font-face declarations before assuming it is present; if absent,
  escalate to user before proceeding).

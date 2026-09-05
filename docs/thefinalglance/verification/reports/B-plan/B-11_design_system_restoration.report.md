# Verification Report: B-11_design_system_restoration.md
Source: F:\.ELAI_workflow\docs\thefinalglance\B-plan\B-11_design_system_restoration.md
Date: 2026-05-17
Overall status: PARTIAL
Confidence: HIGH

## Summary
The dashboard-v2 design system infrastructure (CSS file split into tokens/base/motion/scrollbar, JetBrains Mono + Lucide deps, Workspace screen wiring) is fully in place. However all primary color tokens in tokens.css diverge from the B-11 spec hex values; the body font token resolves to Inter (not JetBrains Mono); line-height is 1.5 (spec requires 1.6/1.4); off-spec font sizes (9px, 11px, 11.5px, 17px, 20px, 22px) appear across 10+ component files; and no visual regression test exists.

## Claims verified DONE
- JetBrains Mono loaded via @fontsource-variable/jetbrains-mono — evidence: .elai_cc/crates/dashboard-v2/package.json:15, .elai_cc/crates/dashboard-v2/src/main.ts:2
- @fontsource-variable/inter present in package.json — evidence: .elai_cc/crates/dashboard-v2/package.json:14
- @lucide/svelte dependency declared — evidence: .elai_cc/crates/dashboard-v2/package.json:16
- Lucide icons actively imported and used in Rail.svelte, Sidebar.svelte, Button.svelte, CommandRow.svelte, Empty.svelte, IconButton.svelte, Input.svelte, Select.svelte, Tree.svelte — evidence: .elai_cc/crates/dashboard-v2/src/lib/layout/Rail.svelte:2, Sidebar.svelte:2 (multiple IMPORT rows in elai.map)
- Token CSS split into dedicated files (tokens.css, base.css, motion.css, scrollbar.css) — evidence: .elai_cc/crates/dashboard-v2/src/styles/tokens.css:1, base.css:1, motion.css:1, scrollbar.css:1
- CSS custom property architecture in place: --bg-*, --teal*, --text-*, --pass/warn/fail/info, --font-*, --r-*, --s-*, --dur-*, --ease-* all defined — evidence: .elai_cc/crates/dashboard-v2/src/styles/tokens.css:8-244
- Scrollbar: 4px width, bg-hover track, teal-soft thumb, hidden until hover — evidence: .elai_cc/crates/dashboard-v2/src/styles/scrollbar.css:12-31 (exact match to spec)
- Shadows: none present; depth via bg color steps per spec — evidence: tokens.css:97-99
- Border-radius tokens: 6px panels (--r-md), 3px buttons (--r-sm) defined — evidence: tokens.css:65-69
- Spacing: 8px base grid (--s-2 = 8px) — evidence: tokens.css:73-84
- Transition tokens: opacity 120ms (--dur-fast), transform 150ms not explicitly tokenized but motion.css uses 120-240ms range — evidence: motion.css:1-7, tokens.css:88
- Three.js + troika-three-text present — evidence: package.json:23-24
- Workspace.svelte exists and is imported in App.svelte root — evidence: .elai_cc/crates/dashboard-v2/src/App.svelte:6-7, 28
- StarCanvas.svelte and OrbitCanvas.svelte present — evidence: .elai_cc/crates/dashboard-v2/src/lib/scene/StarCanvas.svelte:2, src/lib/orbit/OrbitCanvas.svelte:2
- Animation discipline: no bounce, no shake — evidence: motion.css:5 comment "No bounce, no spring overshoot"
- Thinking pulse: 1px teal line, opacity 20%->50%->20%, 2s cycle implemented in MonitorWindow — evidence: .elai_cc/crates/dashboard-v2/src/lib/windows/MonitorWindow.svelte:716-724
- Step completion: teal-ghost flash 100ms implemented — evidence: MonitorWindow.svelte:943-948
- Step failure: 2px fail-color left border, fade in 200ms implemented — evidence: MonitorWindow.svelte:934-941
- Progress dots: scale pulse 100%->110%->100%, 3s cycle implemented — evidence: MonitorWindow.svelte:704-708
- OrbitShootingStar every ~14s implemented — evidence: .elai_cc/crates/dashboard-v2/src/lib/orbit/OrbitShootingStarOverlay.svelte:3, OrbitMode1Plan.svelte:46-47
- ambient-flow animation on inactive edges present — evidence: .elai_cc/crates/dashboard-v2/src/lib/orbit/OrbitConcept.svelte:400-403
- dashflow animation on active branch present — evidence: OrbitConcept.svelte:434-436

## Claims PARTIAL
- JetBrains Mono as primary body font
  - exists: main.ts:2 imports @fontsource-variable/jetbrains-mono; --font-mono: 'JetBrains Mono' declared (tokens.css:105)
  - gap: body CSS rule uses var(--font-sans) (base.css:36), and --font-sans resolves to 'Inter' (tokens.css:104), not JetBrains Mono. JetBrains Mono is defined only as --font-mono and applied via .mono utility class or explicit font-family declarations. App.svelte root also uses var(--font-sans) (App.svelte:37). The claim "Inter is NOT used as a body font" is false.

- Type scale 10/12/13/15/18px throughout all components
  - exists: base.css:21 sets html font-size 13px; 10px and 12px appear in multiple components
  - gap: Off-spec font sizes found: 11px (ArtifactsPanel.svelte, IndexingProgressPanel.svelte, MediaValidationPanel.svelte x11, AntiStalenessDrawer.svelte x6, CosmosDrawer.svelte x4, KnowledgeDrawer.svelte x8, MonitorWindow.svelte, tl-empty), 9px (OrbitCanvas.svelte, OrbitCaption.svelte, BubbleUpStrip.svelte), 9.5px (MainPanel.svelte, MiniCard.svelte, StarCanvas.svelte), 11.5px (MainPanel.svelte, MiniCard.svelte), 17px (Sidebar.svelte:228), 20px (ReasoningDrawer.svelte:480), 22px (RulesDrawer.svelte:343), 10.5px (StatusBar.svelte x3), 14px (ProjectSelector.svelte:391), 56px (ProjectSelector.svelte:242). GatesPanel.svelte uses rem-based sizes (0.7rem, 0.75rem, 0.8rem, 0.875rem, 0.95rem, 1.4rem).

- constellation-breathe 11s animation per DESIGN.md §7
  - exists: OrbitConcept.svelte:186 references "constellation-breathe animation are all gone (they sat in 2D ...)" as a comment
  - gap: The comment explicitly states the constellation-breathe animation was removed in the 3D migration. No 11s cycle implementation found anywhere in current source.

## Claims MISSING
- --bg-void #0a0e12 — searched: tokens.css bg-void; actual value is #07090B, not #0a0e12
- --bg-base #0f1318 — searched: tokens.css bg-base; actual is #0B0E12
- --bg-surface #151a20 — searched: tokens.css bg-surface; actual is #10141A
- --bg-hover #1a2028 — searched: tokens.css bg-hover; actual is #161B22
- --bg-input #1e2530 — searched: tokens.css bg-input; actual is #1A2029
- --teal #1B7A6E — searched: tokens.css teal; actual is #1E8A7A
- --teal-bright #24a08e — searched: tokens.css teal-bright; actual is #2BAE98
- --teal-soft as #1B7A6E40 (25% opacity) — searched: tokens.css teal-soft; actual is rgba(30,138,122,0.28) based on #1E8A7A
- --teal-ghost as #1B7A6E15 (8% opacity) — searched: tokens.css teal-ghost; actual is rgba(30,138,122,0.10)
- --purple #7C5CBF — searched: tokens.css; no purple accent token; tokens.css:124-127 explicitly states system has no purple accent for general use; --state-auditing #9e7aa8 is the only lavender-adjacent token, scoped to FSM badge only
- --purple-soft and --purple-ghost — searched: tokens.css; absent; no purple token family
- --text-primary #d0d7de — searched: tokens.css; actual is #DCE5E8
- --text-secondary #6e7681 — searched: tokens.css; actual is #95A1A8
- --text-ghost #383e47 — searched: tokens.css; actual is #5E6B73
- --warn #c49b2a — searched: tokens.css; actual is #C39A3A
- --fail #c44040 — searched: tokens.css; actual is #B04545
- --info #5088c0 — searched: tokens.css; actual is #5A7A9E (steel, not blue)
- Line height 1.6 (text) / 1.4 (code) — searched: base.css; actual is 1.5 for html and body; headings 1.3; no 1.6 or 1.4 found anywhere
- Display font "Segoe UI" thin/200 — searched: tokens.css; --font-display resolves to 'Inter' (tokens.css:106), not Segoe UI; note: .display class in base.css:167 applies font-weight: 200 but wrong font
- Visual regression test in crates/dashboard-v2/tests/visual.spec.ts — searched: all test files; only star-shoot.test.ts, star-state.test.ts, slash-parse.test.ts exist; no Playwright/visual regression test
- CI gate at pnpm check for visual drift — searched: package.json scripts; check script runs svelte-check + tsc only, no visual regression

## Claims UNVERIFIABLE
- "Inter is NOT used as a body font (only @fontsource-variable/inter is declared for Display titles per spec)" (B-11 rebaseline claim) — reason: this claim is directly contradicted by verified code. --font-sans uses Inter as primary, body element uses var(--font-sans). The plan's rebaseline assessment was incorrect at the time of writing.
- "most of B-11 is already DONE" (V2 REBASELINE heading) — reason: infrastructure is done but all primary color tokens and body font diverge from spec; this assessment overstates completion.
- DETERMINISTIC_CHECK targets crates/dashboard/src-frontend/ (old V1 path) — reason: the actual code lives in dashboard-v2; the check scripts reference a non-existent path and would always pass vacuously.

## What must be done (action list)
- [ ] Align all --bg-* token hex values in tokens.css to spec: bg-void=#0a0e12, bg-base=#0f1318, bg-surface=#151a20, bg-hover=#1a2028, bg-input=#1e2530
- [ ] Align teal tokens: --teal=#1B7A6E, --teal-bright=#24a08e, --teal-soft=rgba(27,122,110,0.25), --teal-ghost=rgba(27,122,110,0.08) or confirm intentional deviation with design decision doc
- [ ] Align text tokens: --text-primary=#d0d7de, --text-secondary=#6e7681, --text-ghost=#383e47
- [ ] Align semantic: --warn=#c49b2a, --fail=#c44040, --info=#5088c0
- [ ] Decide: add --purple/#7C5CBF token family per spec OR keep current "no general purple" policy; document decision
- [ ] Fix body font: change --font-sans to 'JetBrains Mono' OR change body { font-family } to var(--font-mono) — current state uses Inter as body font, contradicting spec
- [ ] Fix --font-display: change to 'Segoe UI Variable' thin per spec (currently resolves to Inter)
- [ ] Fix line-height: set body/html to 1.6; add code block rule for 1.4
- [ ] Sweep 11px occurrences across ArtifactsPanel.svelte, IndexingProgressPanel.svelte, MediaValidationPanel.svelte, AntiStalenessDrawer.svelte, CosmosDrawer.svelte, KnowledgeDrawer.svelte, MonitorWindow.svelte — replace 11px with 10px per spec
- [ ] Sweep remaining off-spec sizes: 9px->10px or document as sub-xs; 9.5px->10px; 11.5px->12px; 10.5px->10px; 14px->13px; 17px->15px; 20px->18px; 22px->18px
- [ ] Normalize GatesPanel.svelte rem-based font sizes to px equivalents matching spec scale
- [ ] Add visual.spec.ts Playwright/Vitest visual regression test to crates/dashboard-v2/tests/
- [ ] Fix DETERMINISTIC_CHECK paths from dashboard/src-frontend to dashboard-v2/src

## Notes
RemoteSidecarBootOverlay.svelte uses raw #6e7681 hex fallbacks (lines 161, 169, 180, 214, 221, 256) instead of var(--text-secondary). These are off-palette relative to the current tokens.css value but happen to match the spec's text-secondary target hex. After token alignment they should be replaced with var(--text-secondary).
The constellation-breathe 11s animation was deliberately removed during the 3D Three.js migration (confirmed by OrbitConcept.svelte:186 comment). If the spec still requires it, it needs re-implementation on the 3D scene — this is non-trivial.

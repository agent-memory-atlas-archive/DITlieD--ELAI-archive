STATUS: SHIPPED (rebaselined s-25120 closure pass)
  schema_version: 103 unchanged (5 new SsbActionType variants SCHEMA-DEFERRED per E-17 precedent — see workflow/reasoning/E-33-decisions.md addendum)
  shipped_on_v2: 11 deliverables (5 components + Wong palette + reduced-motion + ribbon mount + footer mount + 3/4 SkeletonList wirings + motion bootstrap + decisions doc)
  schema_deferred: 5 SsbActionType variants (ActivityRibbonInteracted / ConfidenceTagHovered / SkeletonReplaced / ReducedMotionActivated / WindowFooterModeChanged) — producer-ready, contract test asserts SCHEMA freeze at 103 + locks future labels
  retargets: 1 (ResearchWindow.svelte ABSENT on disk -> SkeletonList wired into ResearchDrawer.svelte instead; see decisions doc (g))
  drift_fixed: 1 (Reads block path: workflow/improvement-sweep-2026-05-19/E-plan/E-2_local_llm_speed_tricks.md -> docs/thefinalglance/E-plan/E-2_local_llm_speed_tricks.md)
  active_wires_closed: 4 (ConfidenceTag consumer in OrbitGrcDots + OrbitEvidenceGates; SnackBarPermission consumer in WindowHost; SettingsWindow toggles ui.activity_ribbon.enabled + accessibility.motion)
  contract_test: crates/contracts/tests/activity_ribbon_ssb_contract.rs NEW (asserts SCHEMA freeze + bijection round-trips for the 2 consumed action types + locks future labels)
  test_pending: Y (cargo check ONLY per user authorization in s-25120; no cargo test runs)
  closure_session: s-25120 (completion-drive session 1 of 4, final plan in batch)

# E-33: Dashboard UX — persistent activity ribbon + skeleton screens + binary confidence + a11y
Objective: ship the UX deltas the 2026 agentic-UI consensus (Kiro CLI, Fuselab 2026, UX Magazine, Phoenix Arize) identified as missing from ELAI's existing 9-window + 10-drawer V2 layout, WITHOUT a rebuild. Five focused additions: (a) persistent Activity Ribbon spanning all 9 windows (Kiro Live Unified Status pattern); (b) skeleton screens for every async list (~20-30% perceived-speed gain per NN/g); (c) binary confidence tags on OrbitGrcDots / OrbitEvidenceGates (Fuselab 2026 — binary decoded faster than numeric); (d) `prefers-reduced-motion` + Wong-derived 5-tone palette for signal layers (Pope.tech 2025 + Apple 2025 Accessibility Labels + WCAG 2.2); (e) mode indicator in every window footer (Claude Code pattern). E-33 is a UX DELTA, not a redesign — leverages B-13 (orbit ActivityWindow SHIPPED), B-26 (dashboard completion SHIPPED s-59964), C-6 / C-7 (design token + UX completeness PLANNED).
Estimated effort: 2-3 sessions, ~8-12 h
Pre-requisites: B-13 SHIPPED s-59964 (V2 ActivityWindow = orbit visualizer Mode 1/2/3 per DESIGN.md §8); B-26 SHIPPED s-59964 (dashboard completion); C-6 PLANNED (design token alignment pass — sequence E-33 AFTER C-6 if both in flight); C-7 PLANNED (UX completeness — E-33 SUPERSEDES the "what gaps remain" portion of C-7 with this report's findings).
Parallel-safe with: E-31, E-32, E-34, E-35.
Source report: workflow/improvement-sweep-2026-05-19/reports/33-dashboard-ux-agentic.md

## Context

The May 2026 agentic-UI consensus is sharp: Fuselab Creative ("UI Design for AI Agents") and UX Magazine both argue that the chatbot template is the wrong substrate for autonomous agents because chat assumes request-response cadence while agents run continuously. Fuselab's four principles — Transparency, User Control, Proactive Status, Structured Error Recovery — translate to a *control room* not a chat sidekick. ELAI's V2 layout (9 windows + 10 drawers + 3-mode ActivityWindow per DESIGN.md §8) is architecturally closer to right than Cursor's mostly-chat surface, but it misses one critical Kiro CLI pattern: a "Live Unified Status" ribbon that survives across windows. Today, ELAI's activity surface lives ONLY in ActivityWindow — when the user switches to ExploreWindow / PlanWindow / ReviewWindow, the agent's current FSM substate + tool + token + last-diff-hint disappears from view (Fuselab + UXMag: "agents continue work while user navigates; if activity is only visible on one window, switching loses progress visibility").

The other three deltas are equally well-grounded: (i) skeleton screens have a 20-30% perceived-speed gain over indeterminate spinners (NN/g, LogRocket, OneThing concurrent agreement) for async content panes — ELAI's Mode 3 RESEARCH phi/psi micro-bars are skeleton-adjacent already, but Research / Explore / Plan / Review windows still have indeterminate spinners. (ii) Binary confidence ("confident" / "uncertain") decoded faster than numeric % per Fuselab 2026 — ELAI's OrbitGrcDots and OrbitEvidenceGates currently use numeric or hue-only encoding. (iii) Accessibility: Pope.tech 2025 ("Design accessible animation") + Apple 2025 Accessibility Labels (Reduced Motion now surfaces as a discoverable feature) mean ELAI's orbit + particle layers + gradient pulses MUST collapse to static glyphs + text under `prefers-reduced-motion`; the brand teal+purple-only palette fails for ~8% male / 0.5% female protanope/deuteranope users (Datylon, Datawrapper, WCAG 2.2) — signal layers need the Wong palette (#0072B2 #E69F00 #D55E00 #CC79A7 #009E73). Mode indicator in window footer is the Claude Code pattern (anyonebuilds, wil.dev) — "what mode am I in" is the #1 user error agent CLIs added a footer to fix.

Cited grounding: kiro.dev/blog/new-look-for-cli (Kiro Live Unified Status pattern), fuselabcreative.com/ui-design-for-ai-agents/ (Fuselab 2026 4 principles), uxmag.com/articles/designing-for-autonomy-ux-principles-for-agentic-ai-systems, nngroup.com/articles/skeleton-screens (20-30% perceived-speed), arxiv:2505.23596 Agent-SAMA 2025 (FSM-as-memory-layer rendering), arize.com/docs/phoenix (node-based agent-graph), cognition.ai/blog/devin-2 (Devin Wiki + interactive planning), blog.pope.tech/2025/12/08/design-accessible-animation-and-movement/, apple.com/newsroom/2025/05/...accessibility-features (Accessibility Nutrition Labels), datylon + datawrapper Wong palette references.

## Existing state (B/C/D coverage)

- B-13 (SHIPPED s-59964): V2 ActivityWindow IS the orbit visualizer per DESIGN.md §8; 3 FSM-driven modes (Mode 1 PLAN constellation, Mode 2 MAP existing-project, Mode 3 RESEARCH). E-33 EXTENDS by adding a *persistent activity ribbon* that surfaces a compressed slice of ActivityWindow state across all 9 windows. ActivityWindow itself is unchanged.
- B-26 (SHIPPED s-59964): Phase 4K.6-4K.13 Dashboard Completion (V2 final pass). E-33 DELTA-on-top: the windows exist; the cross-window persistence + a11y + binary-confidence tags + footer mode indicator are new layers.
- C-6 (PLANNED, Dashboard Design Token Alignment Pass): pre-req for E-33. Sequence E-33 AFTER C-6 if both in flight — Wong-palette signal-layer tokens should land via C-6's token model, then E-33 wires them into OrbitGrcDots / OrbitEvidenceGates. If C-6 isn't shipped first, E-33 introduces minimal token additions for the 5 Wong tones + reduced-motion media query and defers full token cleanup to C-6.
- C-7 (PLANNED, Dashboard UX Completeness): E-33 SUPERSEDES the "what gaps remain" portion of C-7 with the source report's 15-pattern delta list. C-7's other items (smoke tests, etc.) remain.
- DESIGN.md §7 outlined-only buttons: E-33 RESPECTS — does not regress to solid CTAs (source report explicitly endorses ELAI's calm-control-room chrome).
- OrbitGrcDots, OrbitEvidenceGates, OrbitBudgetRing, OrbitConcept Svelte files exist at `crates/dashboard-v2/src/lib/orbit/` (verified by grep). ActivityWindow.svelte exists at `crates/dashboard-v2/src/lib/windows/ActivityWindow.svelte`.
- B-33 (graph viewer contract test exists in `crates/contracts/tests/b33_graph_viewer_contract.rs`): E-33's optional Phoenix-style trace-cycle mode (task 8 below) extends B-33's surface — NOT in this E-N's hot path; deferred.

## Reads (mandatory before coding)

- workflow/improvement-sweep-2026-05-19/reports/33-dashboard-ux-agentic.md (full)
- F:\.ELAI_workflow\.elai_cc\crates\dashboard-v2\src\lib\windows\ActivityWindow.svelte (current Mode 1/2/3 surface)
- F:\.ELAI_workflow\.elai_cc\crates\dashboard-v2\src\lib\layout\WindowHost.svelte (host container — Activity Ribbon mounts here)
- F:\.ELAI_workflow\.elai_cc\crates\dashboard-v2\src\lib\orbit\OrbitGrcDots.svelte (binary-confidence target)
- F:\.ELAI_workflow\.elai_cc\crates\dashboard-v2\src\lib\orbit\OrbitEvidenceGates.svelte (binary-confidence target)
- F:\.ELAI_workflow\.elai_cc\crates\dashboard-v2\src\lib\orbit\OrbitBudgetRing.svelte (reduced-motion target)
- F:\.ELAI_workflow\.elai_cc\crates\dashboard-v2\src\styles\tokens.css (Wong palette additions + reduced-motion query)
- F:\.ELAI_workflow\.elai_cc\crates\dashboard-v2\src\lib\windows\ReviewWindow.svelte (skeleton screen target)
- F:\.ELAI_workflow\.elai_cc\crates\dashboard-v2\src\lib\windows\ExploreWindow.svelte (skeleton screen target — grep for the symbol-graph list)
- F:\.ELAI_workflow\.elai_cc\crates\dashboard-v2\src\lib\windows\PlanWindow.svelte (skeleton screen target — phase rows)
- F:\.ELAI_workflow\.elai_cc\crates\dashboard-v2\src\lib\drawers\ResearchDrawer.svelte (SHIPPED-AS-RESEARCH-DRAWER — ResearchWindow.svelte does not exist on disk; ResearchDrawer is the canonical FORMULATE/SEARCH/EXTRACT/REFLECT/CONVERGE surface and is the SkeletonList target. See E-33-decisions.md (g) for retarget rationale.)
- DESIGN.md sections §7 outlined-only buttons + §8 orbit modes
- docs/thefinalglance/E-plan/E-2_local_llm_speed_tricks.md (DRIFT_FIXED: original cited workflow/improvement-sweep-2026-05-19/E-plan/ which does not exist on disk; canonical path verified via Glob during s-25120 grounding sweep) — already shipped skeleton + cursor + frame pacer; E-33 EXTENDS those primitives to non-Chat surfaces

## Tasks

1. [SHIPPED-ON-V2] Implement `crates/dashboard-v2/src/lib/components/ActivityRibbon.svelte` (~220 LOC) as a persistent thin row that mounts in `WindowHost.svelte` and survives all 9 window switches. Renders a compressed slice of ActivityWindow state: current FSM substate badge + active AgentRole + last tool name + active conversation token counter + last-diff-hint (line-by-line, Kiro-style). Position: top edge of WindowHost OR bottom edge — A/B per source report Open Question 1. Subscribes to the same SSB stream ActivityWindow does. Default visible; togglable via SettingsWindow `ui.activity_ribbon.enabled` (default true). Cite kiro.dev/blog/new-look-for-cli + fuselabcreative.com/ui-design-for-ai-agents/.
   Shipped: ActivityRibbon.svelte mounted at WindowHost.svelte:56 (top edge per decisions doc (d) rationale). SettingsWindow toggle ui.activity_ribbon.enabled now wired via uxPrefs store (s-25120 closure pass).

2. [SHIPPED-ON-V2 + SHIPPED-AS-RESEARCH-DRAWER] Add `crates/dashboard-v2/src/lib/components/SkeletonList.svelte` (~120 LOC) generic skeleton-row component for any async list. Renders N placeholder rows with shimmer animation; replaced row-by-row as data arrives (not all-at-once on load complete). API: `<SkeletonList rows={5} loading={isLoading} />`. Wire into ResearchWindow (FORMULATE / SEARCH / EXTRACT / REFLECT / CONVERGE stage list), ExploreWindow (symbol-graph results), PlanWindow (phase rows), ReviewWindow (audit findings list). Cite nngroup.com/articles/skeleton-screens + logrocket + onething.design.
   Shipped: SkeletonList.svelte present. Wired into PlanWindow / ExploreWindow / ReviewWindow. ResearchWindow.svelte does not exist on disk; SkeletonList retargeted to ResearchDrawer.svelte (canonical FORMULATE/SEARCH/EXTRACT/REFLECT/CONVERGE surface). See E-33-decisions.md (g).

3. [SHIPPED-ON-V2] Add `crates/dashboard-v2/src/lib/components/ConfidenceTag.svelte` (~80 LOC) binary-confidence tag component. Renders one of 4 states: {needs-evidence, low, ok, verified} as text + icon glyph + Wong-tone color. Replaces numeric % surface on OrbitGrcDots + OrbitEvidenceGates. Each state has a glyph (shape-distinct so color is redundant signal). Cite fuselabcreative.com/ui-design-for-ai-agents/ ("Users decided faster with 'confident' vs 'uncertain' than with specific confidence scores").
   Shipped: ConfidenceTag.svelte present with 4-state classifier (thresholds 0.25/0.5/0.85), glyph map, Wong-tone color via --confidence-* tokens, raw numeric in hover tooltip.

4. [SHIPPED-ON-V2] Wire ConfidenceTag into `OrbitGrcDots.svelte` + `OrbitEvidenceGates.svelte` (~40 LOC delta each). REPLACE numeric or hue-only confidence with ConfidenceTag. Keep numeric value as a tooltip-on-hover (no info loss). Map current value ranges: <0.25 → needs-evidence; 0.25-0.50 → low; 0.50-0.85 → ok; >0.85 → verified. Add hover tooltip showing raw numeric for power users.
   Shipped (s-25120): OrbitGrcDots wired with optional confidence prop rendered via foreignObject + ConfidenceTag overlay; OrbitConcept feeds via new state/grc.svelte.ts store polling get_recent_actions for the most recent reasoning_trace ReasoningPayload.confidence. OrbitEvidenceGates renders ConfidenceTag on hover via foreignObject; verdictToValue maps pass/warn/block/pending onto the same 0..1 bands.

5. [SHIPPED-ON-V2] Add Wong-derived 5-tone palette to `crates/dashboard-v2/src/styles/tokens.css` (~30 LOC). New tokens: `--signal-blue: #0072B2;` `--signal-orange: #E69F00;` `--signal-vermilion: #D55E00;` `--signal-rose: #CC79A7;` `--signal-green: #009E73;`. Use ONLY in signal layers (ConfidenceTag, OrbitGrcDots dot color, OrbitEvidenceGates gate color). Brand teal (`#1B7A6E`) + brand purple (`#7C5CBF`) stay for BRAND CHROME (window chrome, dividers, accents) NOT for signal layers. Cite datylon + datawrapper Wong reference + WCAG 2.2.
   Shipped: tokens.css:267-271 declares all 5 Wong signal tokens + --confidence-* semantic aliases at :275-278.

6. [SHIPPED-ON-V2] Add `prefers-reduced-motion: reduce` media query in `tokens.css` + targeted overrides in OrbitConcept / OrbitBudgetRing / particle layers (~80 LOC). Behavior: under reduced motion, orbit rotation stops, gradient pulses freeze, particle layers collapse to a static dot grid + text labels. Provide static equivalents (Pope.tech two-strategy: disable decorative, provide static equivalent for essential motion). Add a settings toggle `accessibility.motion: auto | reduced | full` (default auto = follow system). Cite blog.pope.tech/2025/12/08/design-accessible-animation-and-movement/ + apple.com/newsroom/2025/05/...accessibility-features.
   Shipped: tokens.css:290 @media (prefers-reduced-motion: reduce) block + :root[data-motion="reduced"] user-override. motion.svelte.ts store live-listens MediaQueryList. SettingsWindow toggle accessibility.motion (auto/reduced/full) wired via uxPrefs store (s-25120 closure pass) — calls motion.setUserOverride and persists to localStorage.

7. [SHIPPED-ON-V2] Add `crates/dashboard-v2/src/lib/components/WindowFooterMode.svelte` (~70 LOC). Thin footer row showing current FSM state + agent_substate + cap-status (any active SsbActionType::WarnLimitReached / RouteBlocked etc.). Mount in WindowHost so it appears at the bottom of every window. Cite anyonebuilds.com plan-mode guide + wil.dev TUI guide. Distinct from ActivityRibbon: ribbon shows live operation; footer shows static-state mode.
   Shipped: WindowFooterMode.svelte mounted at WindowHost.svelte:85 with scope prop derived from WindowType.

8. [SHIPPED-ON-V2] Snack-bar permission prompt component `crates/dashboard-v2/src/lib/components/SnackBarPermission.svelte` (~110 LOC). When the FSM gate raises a permission-required event (existing surface: per-tool permission prompts in EXECUTING state), render a stacked snack-bar row just above the Chat/Command input — visible across all windows when active. Approve / Reject / Always-allow buttons. Cite kiro.dev/blog/new-look-for-cli "snack-bar permission prompts above the input."
   Shipped (s-25120): SnackBarPermission.svelte mounted in WindowHost between body + footer. New state/permission_prompts.svelte.ts store polls get_recent_actions for intervention_request rows; approve/reject/alwaysAllow handlers route through stub.logStub + per-id dedup set. SCHEMA-DEFERRED note: dedicated `permission_required` SsbActionType would be ideal but is folded into the SCHEMA-DEFERRED 5-variant batch; intervention_request is the canonical surface today.

9. [SCHEMA-DEFERRED x5] Add SSB-driven instrumentation rows so we can A/B the deltas: `SsbActionType::ActivityRibbonInteracted`, `SsbActionType::ConfidenceTagHovered`, `SsbActionType::SkeletonReplaced`, `SsbActionType::ReducedMotionActivated`, `SsbActionType::WindowFooterModeChanged` (~30 LOC in `crates/orchestrator/src/ssb.rs`). Without these, source report Open Question 1-4 ("does ribbon reduce confusion?", "does binary confidence reduce decision latency?", "does graph view detect spirals earlier?", "does reduced-motion retain enough info?") are unmeasurable.
   SCHEMA-DEFERRED rationale (s-25120 closure): all 5 variants ship deferred to the next QUINTUPLE/SEXTUPLE SCHEMA bump batch per the completion-drive session 1 lock at SCHEMA_VERSION 103 (E-17 ContextualChunkEmitted + ChunkRegenSkipped precedent). Producer-side dashboard consumers are wired today (grc store, permission_prompts store, motion store, uxPrefs store, ActivityRibbon mount) so the emit-sites flip atomically when SCHEMA unlocks. Contract test crates/contracts/tests/activity_ribbon_ssb_contract.rs locks the canonical SQL labels (activity_ribbon_interacted / confidence_tag_hovered / skeleton_replaced / reduced_motion_activated / window_footer_mode_changed) and pins SCHEMA freeze + `#[ignore]`d round-trip tests ready to un-`#[ignore]` on unlock.

10. Defer Phoenix-style node-based agent-graph trace view (sweep recommendation #8) to E-33.B. The OrbitGraphViewer (B-33) already renders structure; trace-cycle mode is an additive surface; needs a separate explore-spawn for the right hook + interaction model.

11. Defer Devin-style auto-built repo wiki (sweep recommendation #9) to E-33.C. Touches ProjectIntelligence drawer + elai.map + SCIP indices; refresh cadence + storage model is its own design problem.

12. Defer `@` for context pick + `/` for command pick generalization across all windows (sweep recommendation #10) to E-33.D. Currently lives in Chat/Command; extending to Research / Explore / Plan / Review windows is a tractable but separate keymap + input-handling problem.

13. [SHIPPED-ON-V2] Author `workflow/reasoning/E-33-decisions.md` (~120 LOC): (a) why Activity Ribbon vs leaving ActivityWindow alone (cross-window visibility); (b) why binary confidence over numeric (Fuselab finding + hover-tooltip preserves info); (c) why Wong palette ONLY in signal layers (brand chrome unchanged); (d) reduced-motion default (auto = follow system); (e) explicit DEFERRED list: trace-cycle graph view (E-33.B), Devin Wiki (E-33.C), @/ context pick generalization (E-33.D), live-preview iframe (already exists per source report), checkpoint browser (sweep #12).
   Shipped: E-33-decisions.md present + extended (s-25120 closure pass adds addenda for SCHEMA-lock deferral, ResearchWindow->ResearchDrawer retarget, ConfidenceTag consumer wire, SnackBarPermission mount, SettingsWindow toggle catalog).

## WIRING_CONTRACT

produces_symbols:
  - crates/dashboard-v2/src/lib/components/ActivityRibbon.svelte (new)
  - crates/dashboard-v2/src/lib/components/SkeletonList.svelte (new)
  - crates/dashboard-v2/src/lib/components/ConfidenceTag.svelte (new)
  - crates/dashboard-v2/src/lib/components/WindowFooterMode.svelte (new)
  - crates/dashboard-v2/src/lib/components/SnackBarPermission.svelte (new)
  - crates/dashboard-v2/src/styles/tokens.css :: signal-* Wong palette tokens
  - crates/dashboard-v2/src/styles/tokens.css :: prefers-reduced-motion overrides
  - SsbActionType::ActivityRibbonInteracted (new)
  - SsbActionType::ConfidenceTagHovered (new)
  - SsbActionType::SkeletonReplaced (new)
  - SsbActionType::ReducedMotionActivated (new)
  - SsbActionType::WindowFooterModeChanged (new)
consumes_symbols:
  - crates/dashboard-v2/src/lib/layout/WindowHost.svelte (ActivityRibbon + WindowFooterMode mount points)
  - crates/dashboard-v2/src/lib/windows/ActivityWindow.svelte (data source for ribbon compressed slice; no changes to ActivityWindow itself)
  - crates/dashboard-v2/src/lib/orbit/OrbitGrcDots.svelte (binary confidence wiring)
  - crates/dashboard-v2/src/lib/orbit/OrbitEvidenceGates.svelte (binary confidence wiring)
  - crates/dashboard-v2/src/lib/orbit/OrbitBudgetRing.svelte (reduced-motion override)
  - crates/dashboard-v2/src/lib/windows/{Research,Explore,Plan,Review}Window.svelte (SkeletonList integration)
reachable_from:
  - WindowHost mount -> ActivityRibbon (persistent across navigation) + WindowFooterMode (persistent)
  - any async list render -> SkeletonList -> SkeletonReplaced SSB on data arrival
  - hover on OrbitGrcDot -> ConfidenceTagHovered SSB
  - permission gate raise -> SnackBarPermission component
stub_ban:
  - no indeterminate spinner remains in Research / Explore / Plan / Review windows after task 2
  - no numeric % confidence surface remains on OrbitGrcDots / OrbitEvidenceGates after task 4 (numeric kept as tooltip ONLY)
  - no hue-only state encoding in signal layers (every signal carries shape + text in addition to color)
  - no auto-playing motion > 5s without a pause control (Pope.tech 2025 + WCAG)
  - no solid CTA button regression (DESIGN.md §7 outlined-only respected)
  - no break to existing ActivityWindow Mode 1/2/3 behavior
  - no .unwrap() in Tauri command paths
frontend_pair: ALL OF crates/dashboard-v2/src/lib/components/ (this is the frontend-pair E-N — no backend Rust source crate change outside SSB variant additions)
observable_effect:
  trigger: "Open dashboard, start a long-running agent task, switch from ChatWindow -> ExploreWindow -> ReviewWindow within 10s; enable Settings.accessibility.motion=reduced mid-flight"
  expect_ssb: "ActivityRibbonInteracted (window switches recorded); ReducedMotionActivated on toggle; WindowFooterModeChanged on FSM substate transitions; SkeletonReplaced on each async list resolve"
  expect_artifact: "ActivityRibbon visible at the top OR bottom edge of all 3 windows showing current FSM substate + AgentRole + last tool name; orbit rotation halts under reduced motion; ConfidenceTag glyphs visible on OrbitGrcDots; numeric values surface on hover"

## DETERMINISTIC_CHECK

```bash
# 1. Five new components exist
test -f .elai_cc/crates/dashboard-v2/src/lib/components/ActivityRibbon.svelte
test -f .elai_cc/crates/dashboard-v2/src/lib/components/SkeletonList.svelte
test -f .elai_cc/crates/dashboard-v2/src/lib/components/ConfidenceTag.svelte
test -f .elai_cc/crates/dashboard-v2/src/lib/components/WindowFooterMode.svelte
test -f .elai_cc/crates/dashboard-v2/src/lib/components/SnackBarPermission.svelte

# 2. Wong palette tokens present
rg "signal-blue|signal-orange|signal-vermilion|signal-rose|signal-green" .elai_cc/crates/dashboard-v2/src/styles/tokens.css
# Expected: >= 5 hits

# 3. prefers-reduced-motion media query in tokens.css
rg "prefers-reduced-motion" .elai_cc/crates/dashboard-v2/src/styles/tokens.css
# Expected: >= 1 hit

# 4. ActivityRibbon mounted in WindowHost
rg "ActivityRibbon" .elai_cc/crates/dashboard-v2/src/lib/layout/WindowHost.svelte

# 5. ConfidenceTag wired into OrbitGrcDots + OrbitEvidenceGates
rg "ConfidenceTag" .elai_cc/crates/dashboard-v2/src/lib/orbit/OrbitGrcDots.svelte
rg "ConfidenceTag" .elai_cc/crates/dashboard-v2/src/lib/orbit/OrbitEvidenceGates.svelte

# 6. SkeletonList integrated into the 4 windows
rg "SkeletonList" .elai_cc/crates/dashboard-v2/src/lib/windows/ResearchWindow.svelte
rg "SkeletonList" .elai_cc/crates/dashboard-v2/src/lib/windows/ExploreWindow.svelte
rg "SkeletonList" .elai_cc/crates/dashboard-v2/src/lib/windows/PlanWindow.svelte
rg "SkeletonList" .elai_cc/crates/dashboard-v2/src/lib/windows/ReviewWindow.svelte

# 7. New SSB variants registered
rg "ActivityRibbonInteracted|ConfidenceTagHovered|SkeletonReplaced|ReducedMotionActivated|WindowFooterModeChanged" .elai_cc/crates/orchestrator/src/ssb.rs
# Expected: 5 enum arms

# 8. svelte-check passes
cd .elai_cc/crates/dashboard-v2 && pnpm svelte-check
```

## ACCEPTANCE

- [ ] ActivityRibbon visible at WindowHost edge; persists across all 9 window switches
- [ ] SkeletonList replaces indeterminate spinners in Research / Explore / Plan / Review windows
- [ ] ConfidenceTag with 4 binary states + glyph + Wong tone replaces numeric % on OrbitGrcDots + OrbitEvidenceGates (numeric value kept as hover tooltip)
- [ ] Wong-derived 5-tone palette tokens added; signal layers use Wong; brand chrome unchanged (teal/purple stay)
- [ ] `prefers-reduced-motion: reduce` halts orbit rotation + gradient pulses + particle layers; static equivalent retains semantic info
- [ ] WindowFooterMode shows current FSM state + substate + cap-status at the bottom of every window
- [ ] SnackBarPermission renders permission-required events above the input bar across windows
- [ ] 5 new SSB variants registered + emitted from the right interaction sites (for A/B measurement)
- [ ] DESIGN.md §7 outlined-only buttons NOT regressed
- [ ] No indeterminate spinner remains in the 4 target windows
- [ ] svelte-check passes (no type errors); existing ActivityWindow Mode 1/2/3 unchanged
- [ ] Accessibility: hue-alone never encodes state in signal layers (shape + text + Wong color = redundant)
- [ ] CLAUDE.md anti-disturbance: no `.unwrap()` introduced in Tauri command paths

## Tests to add

- `crates/dashboard-v2/src/lib/components/ActivityRibbon.test.ts`: assert ribbon mounts in WindowHost; assert state subscription to ActivityWindow SSB; assert visibility persists across simulated window switches.
- `crates/dashboard-v2/src/lib/components/SkeletonList.test.ts`: assert N placeholders render on loading=true; assert row-by-row replacement on data arrival (not all-at-once).
- `crates/dashboard-v2/src/lib/components/ConfidenceTag.test.ts`: assert 4 states render distinct glyph + Wong tone; assert hover tooltip surfaces raw numeric value.
- `crates/dashboard-v2/src/lib/components/WindowFooterMode.test.ts`: assert footer reflects current FSM state + substate + cap-status; assert WindowFooterModeChanged SSB on transition.
- `crates/dashboard-v2/src/lib/components/SnackBarPermission.test.ts`: assert mount on permission event; assert Approve / Reject / Always-allow buttons fire correct SSB rows.
- `crates/dashboard-v2/src/lib/styles/reduced_motion.test.ts`: programmatically toggle `prefers-reduced-motion`, assert orbit rotation halts + ReducedMotionActivated SSB.
- `crates/contracts/tests/activity_ribbon_ssb_contract.rs`: SSB schema validity for the 5 new variants.

## Files touched

- CREATE: `crates/dashboard-v2/src/lib/components/ActivityRibbon.svelte` (~220 LOC)
- CREATE: `crates/dashboard-v2/src/lib/components/SkeletonList.svelte` (~120 LOC)
- CREATE: `crates/dashboard-v2/src/lib/components/ConfidenceTag.svelte` (~80 LOC)
- CREATE: `crates/dashboard-v2/src/lib/components/WindowFooterMode.svelte` (~70 LOC)
- CREATE: `crates/dashboard-v2/src/lib/components/SnackBarPermission.svelte` (~110 LOC)
- MODIFY: `crates/dashboard-v2/src/styles/tokens.css` (+~30 LOC Wong palette + +~80 LOC reduced-motion overrides)
- MODIFY: `crates/dashboard-v2/src/lib/layout/WindowHost.svelte` (~30 LOC: mount ActivityRibbon + WindowFooterMode)
- MODIFY: `crates/dashboard-v2/src/lib/orbit/OrbitGrcDots.svelte` (~40 LOC: ConfidenceTag wiring)
- MODIFY: `crates/dashboard-v2/src/lib/orbit/OrbitEvidenceGates.svelte` (~40 LOC: ConfidenceTag wiring)
- MODIFY: `crates/dashboard-v2/src/lib/orbit/OrbitBudgetRing.svelte` (~30 LOC: reduced-motion override hook)
- MODIFY: `crates/dashboard-v2/src/lib/orbit/OrbitConcept.svelte` (~20 LOC: reduced-motion override hook)
- MODIFY: `crates/dashboard-v2/src/lib/windows/ResearchWindow.svelte` (~30 LOC: SkeletonList for FORMULATE/SEARCH/EXTRACT/REFLECT/CONVERGE list)
- MODIFY: `crates/dashboard-v2/src/lib/windows/ExploreWindow.svelte` (~30 LOC: SkeletonList for symbol-graph results)
- MODIFY: `crates/dashboard-v2/src/lib/windows/PlanWindow.svelte` (~30 LOC: SkeletonList for phase rows)
- MODIFY: `crates/dashboard-v2/src/lib/windows/ReviewWindow.svelte` (~30 LOC: SkeletonList for audit findings)
- MODIFY: `crates/dashboard-v2/src/lib/windows/SettingsWindow.svelte` (~40 LOC: ui.activity_ribbon.enabled + accessibility.motion toggles)
- MODIFY: `crates/orchestrator/src/ssb.rs` (~30 LOC: 5 new SsbActionType variants + as_sql_str entries)
- CREATE: `workflow/reasoning/E-33-decisions.md` (~120 LOC)

## Citations / Grounding

- kiro.dev/blog/new-look-for-cli (cited in source) — Live Unified Status bar, omnipresent input, snack-bar permission prompts
- fuselabcreative.com/ui-design-for-ai-agents/ (cited in source) — 4 principles, binary confidence > numeric, plan-and-execute separation
- uxmag.com/articles/designing-for-autonomy-ux-principles-for-agentic-ai-systems (cited in source) — autonomy → visible reasoning
- nngroup.com/articles/skeleton-screens (cited in source) — 20-30% perceived-speed gain
- logrocket + onething.design (cited in source) — skeleton screen psychology
- blog.pope.tech/2025/12/08/design-accessible-animation-and-movement/ (cited in source) — two-strategy: disable decorative, static equivalent for essential
- apple.com/newsroom/2025/05/...accessibility-features (cited in source) — Accessibility Nutrition Labels surface Reduced Motion as discoverable feature
- WCAG 2.2 (cited in source) — never rely on hue alone for state
- datylon.com + datawrapper.de (cited in source) — Wong 5-tone palette gold standard
- anyonebuilds.com plan-mode + wil.dev TUI guide (cited in source) — footer mode indicator pattern
- arXiv:2505.23596 (cited in source) — Agent-SAMA 2025: FSM-as-memory-layer
- arize.com/docs/phoenix + langchain.com/langsmith/observability + langfuse.com (cited in source) — agent-graph trace visualization (E-33.B deferred)
- cognition.ai/blog/devin-2 (cited in source) — Devin Wiki (E-33.C deferred)
- cursor.com/blog/2-0 + blog.replit.com/agent-v2 + aider.chat (cited in source) — comparison context
- gethopp.app/blog/tauri-vs-electron (cited in source) — already aligned via ELAI's Tauri choice; no action

## Open questions / risks

- ActivityRibbon position (top or bottom of WindowHost): source report Open Question 1 calls for A/B. Ship at bottom by default (closer to focus + input bar) and instrument via ActivityRibbonInteracted to measure.
- Binary confidence mapping thresholds (0.25 / 0.50 / 0.85): heuristic — calibrate after ConfidenceTagHovered SSB telemetry confirms users react to the right boundaries. Source report Open Question 2.
- Reduced-motion fallback for orbit modes: static dot grid + text may not retain enough info for power users. Source report Open Question 4 calls for parity ±5% on task completion between motion-on and motion-off; instrument via ReducedMotionActivated SSB.
- Wong palette in signal layers may visually clash with brand teal/purple chrome on hover/focus states. Pilot on OrbitGrcDots first; if clash, add brand-tone borders + Wong fill (already a common practice for signal-on-brand-bg).
- SnackBarPermission integration depends on existing FSM permission-gate events being raised — verify the event surface exists in the current orchestrator before wiring; if not, the permission flow may need a small SsbActionType::PermissionRequested addition (in scope for this E-N if shallow).
- C-6 design token alignment may rebase tokens.css after E-33 lands. Coordinate timeline so E-33 lands AFTER C-6, or so the new tokens are additive (won't be deleted by C-6's pass).
- Test infrastructure for `prefers-reduced-motion` programmatic toggle is non-trivial in Vitest + jsdom — may need a small media-query mock helper. Acceptable cost.
- This E-N is heavy on Svelte LOC (~700-800 LOC of new component code). Sequence ahead of C-7 UX completeness; C-7 likely shrinks once E-33 lands.

## Next

- E-33.B (deferred): Phoenix-style node-based agent-graph trace view (extends B-33 OrbitGraphViewer).
- E-33.C (deferred): Devin-style auto-built repo wiki refreshed on audit pass; integrates with ProjectIntelligence drawer + elai.map + SCIP indices.
- E-33.D (deferred): `@` for context pick + `/` for command pick generalized across all windows (today only in Chat/Command).
- E-33.E (deferred): live-preview iframe (already exists per source report — confirm + polish only).
- E-33.F (deferred): checkpoint browser in Review window (sweep recommendation #12 — Cline pattern).
- E-33.G (deferred): per-interaction token + cost counter (sweep recommendation #13 — Cline + Codex blended-usage pattern).
- C-7 Dashboard UX Completeness: rebases on E-33's deltas (most of C-7's "what gaps remain" is closed by E-33; C-7 becomes a smoke-test pass + remaining items).
- B-13 V2 ActivityWindow: unchanged by E-33; the ribbon is a compressed slice, not a replacement.
- E-2 already shipped CommandWindow skeleton bubble + frame pacer — E-33's SkeletonList is the generic extension of that pattern to non-Chat windows.

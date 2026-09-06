# ELAI Design References

## Summary

Cross-cutting patterns from all research, in priority order:

- Dark-first, luminance-elevation over shadows: every premium 2024-2026 dark UI (Linear, Vercel, Superhuman, Warp) replaces drop shadows with surface-color stepping. Depth = a lighter bg-surface value over bg-base, never a box-shadow. The ELAI token system already encodes this correctly with `--bg-void` through `--bg-input` as a 5-step ladder. [Linear redesign post](https://linear.app/now/how-we-redesigned-the-linear-ui), [Muzli dark mode guide](https://muz.li/blog/dark-mode-design-systems-a-complete-guide-to-patterns-tokens-and-hierarchy/)
- Sidebar as semantic anchor, not navigation menu: Arc Browser, Kimi, and Linear all treat the sidebar as a persistent, narrow identity column (not a nav drawer). Icons only, no text labels, max width 64-72px. The "active" signal is a left-edge inset bar + ghost background tint — not a bold fill. This is exactly what DESIGN-BRIEF section 4.2 specifies. [Arc Browser UX analysis](https://blog.logrocket.com/ux-design/ux-analysis-arc-opera-edge/), [Arc design lens](https://medium.com/design-bootcamp/arc-browser-rethinking-the-web-through-a-designers-lens-f3922ef2133e)
- Command palette as the primary interaction surface: Raycast, Superhuman, Cursor, and Warp all converge on Cmd+K / Ctrl+K as the single entry point for all actions. The palette replaces menus, buttons, and search. The UI beneath it can be extremely sparse because the palette handles everything. [Raycast Windows announcement](https://www.raycast.com/changelog/windows), [Superhuman shortcuts guide](https://help.superhuman.com/hc/en-us/articles/45191759067411-Speed-Up-With-Shortcuts)
- Motion communicates state, never decorates: the 2026 trend consensus (Envato, Muzli) explicitly names "motion that explains, not performs." Breath/pulse on live state, one-shot scan on phase completion, fade-in for content reveals. Zero bounce, zero spring overshoot, zero spinner. ELAI's `breath` keyframe (2.2s, opacity 0.2->0.55->0.2) is aligned with this pattern. [Envato 2026 UX/UI trends](https://elements.envato.com/learn/ux-ui-design-trends), [Muzli dark mode guide](https://muz.li/blog/dark-mode-design-systems-a-complete-guide-to-patterns-tokens-and-hierarchy/)
- Near-black with tint, not pure black: both Superhuman's dark theme research and the Muzli guide recommend `#0F0F0F` range (never `#000000`) with zinc/charcoal tinting for long-session comfort. Pure black increases halation; charcoal-tinted near-black reduces it. ELAI's `--bg-void: #07090B` (zinc-green tinted) is consistent with this finding. [Superhuman dark themes](https://blog.superhuman.com/how-to-design-delightful-dark-themes/), [Colorhero dark palettes](https://colorhero.io/blog/dark-mode-color-palettes-2025)


---

## Per-reference notes

### 1. Linear

- URL: https://linear.app/now/how-we-redesigned-the-linear-ui and https://linear.app/now/behind-the-latest-design-refresh
- Layout pattern: "Inverted L-shape" global chrome. Sidebar (persistent left column) + top header bar + main content. No explicit sidebar width disclosed in the article, but described as narrow icon column with muted inactive states. View modes supported: list, board, timeline, split, fullscreen.
- Color palette: Migrated from HSL to LCH color space. Core theme reduced to 3 variables: base color, accent color, contrast (range 30-100). Default dark mode shifted from "cool blue-ish hue" to "warmer gray, less saturated." Working principle: "opacities of black and white." No hex literals disclosed in the redesign posts — they work from LCH-generated values. The 2025 website moved to monochrome black/white with almost no color.
- Typography: Inter Display for headings (added expression vs regular Inter). Regular Inter for body. No specific size scale disclosed in the posts. Font is consistent with ELAI's `--font-sans: 'Inter'` choice.
- Signature interaction: Sidebar de-emphasis ("a few notches dimmer than the content area"). Tab bar compact, rounded corners, small iconography. Active state: 1-px bottom border. Content area is the attention priority.
- Key design principles quoted: "Don't compete for attention you haven't earned." "Structure should be felt not seen." Borders softened, contrast reduced — hierarchy via color step, not hard borders.
- Specific patterns we will adopt for ELAI:
  - "Structure should be felt not seen": use `--border-subtle` (rgba 4%) not visible dividers. ELAI brief section 4.3 already specifies this.
  - Sidebar muting: icons at `--text-secondary` by default, only `--teal` on active. This matches brief section 4.2 exactly.
  - Tab bar sits one step elevated (bg-surface over bg-base). Linear does this; ELAI brief encodes this as `--bg-surface` for the tab bar.
  - LCH-inspired token naming: rather than assigning flat hex in components, use CSS custom properties that can be recomputed from 3 base values — this is why ELAI's tokens.css approach of building all colors from variables is correct.
- What to avoid: Linear's 2025 website went nearly colorless (pure black/white). For a developer tool dashboard that must show semantic state (FSM states, gate verdicts, VRAM alerts), zero-color is too stripped. We keep the teal/steel duo.

### 2. Arc Browser

- URL: https://medium.com/design-bootcamp/arc-browser-rethinking-the-web-through-a-designers-lens-f3922ef2133e and https://resources.arc.net/hc/en-us/articles/19228064149143-Spaces-Distinct-Browsing-Areas
- Layout pattern: Sidebar-dominant shell. Everything tucked in the left sidebar — no top address bar, no persistent toolbars in the content area. Content area is visually empty of chrome. Sidebar handles: tabs (vertical list), spaces (dots at bottom), bookmarks (folder structure), quick-access shortcuts.
- Color palette: Per-space theming — each space can have its own accent color. Dark mode is a first-class design variant, not an adaptation. The browser auto-hides UI to maximize content area. No specific hex values published.
- Typography: Clean, modern sans-serif throughout. Emphasis on space and breathing room rather than text density.
- Signature interaction: Spaces as dots at sidebar bottom — switching context via dots is exactly the "mode dot" pattern ELAI uses for LOCAL/CLOUD/HYBRID. Two-finger swipe to switch spaces. Tab list is vertical, not horizontal — eliminates the "too many tabs" overflow problem.
- Specific patterns we will adopt for ELAI:
  - Logo-forward sidebar top: Arc puts its brand mark at the top of the sidebar as the visual anchor. This is the "Kimi-inspired" pattern the brief references — the ELAI wordmark at the top of the 64px sidebar is directly inspired by both Arc and Kimi.
  - Space dots at sidebar bottom: the LOCAL/CLOUD/HYBRID mode dots in ELAI's status bar bottom is informed by Arc's space-switcher dots. Consider the mode dot as an affordance the user can click, not just a status indicator.
  - Content area free of chrome: the dual workspace panels have zero toolbars outside the tab bar. Actions appear contextually or via command palette.
  - Sidebar auto-hides in Arc but ELAI keeps it persistent at 64px — this is a deliberate deviation because ELAI's engine has more persistent navigation state than a browser.
- What to avoid: Arc's per-space color theming is beautiful for a browser but would add visual noise to ELAI's single-teal-accent system. We do not adopt per-panel color customization.

### 3. Raycast

- URL: https://www.raycast.com/, https://developers.raycast.com/api-reference/user-interface, https://uxdesign.cc/raycast-for-designers-649fdad43bf1
- Layout pattern: Single floating command bar. No persistent sidebar or panel layout — the app is only the command bar overlay. When extensions add views (lists, detail panes, forms), they appear inside the command bar surface as a split: left list + right detail.
- Color palette: Deep black background with high-contrast white text. "Sleek dark chrome, vibrant gradient accents" per getdesign.md. The UI is 99% text with strategic accent use. Not specifically teal — Raycast uses warm purple/pink gradients for their own brand. For ELAI we take the structural pattern, not their brand color.
- Typography: Clean modern sans-serif. Monospace for code/paths/IDs. Keyboard hints styled as pill-shaped `Kbd` elements with ghost backgrounds. This is directly the `Kbd.svelte` component in ELAI's component inventory.
- Signature interaction: Cmd+K / Cmd+Space opens the palette. Typing filters a flat list instantly. Keyboard navigation is the only navigation. The interaction model: type intent -> get options -> act. No mouse required.
- Specific patterns we will adopt for ELAI:
  - Slash-palette overlay: ELAI's CommandWindow input bar with `/` slash commands is a Raycast command palette in fixed position. The `--bg-overlay` backdrop at 72% opacity, with the palette rendered at `--bg-surface` elevation, mirrors Raycast's floating approach.
  - Kbd pill component: Raycast's inline keyboard hint pills are the direct model for ELAI's `Kbd.svelte` — 11px mono text, steel ghost background (`--steel-ghost`), visible on hover.
  - Empty-state shows recents: Raycast command palette default state shows recent commands. ELAI's slash palette should default to recent slash commands, not an empty input.
  - List + detail split inside the palette: when slash commands produce output (e.g., `/status`), show a split layout: command on left, result on right inside the same overlay surface.
- What to avoid: Raycast uses vibrant gradient accents (orange-to-pink). These violate ELAI's anti-neon, anti-gradient-button rule. Our palette stays muted teal only.

### 4. Warp Terminal

- URL: https://www.warp.dev/blog/how-we-designed-themes-for-the-terminal-a-peek-into-our-process and https://docs.warp.dev/university/developer-workflows/beginner/how-to-make-warps-ui-more-minimal
- Layout pattern: Full-viewport terminal with block-based output. Each command + its output forms a discrete "block" separated by thin horizontal lines. Command input is IDE-like (cursor positioned, editable). Command palette (Cmd+P) overlays the terminal.
- Color palette: ANSI 16-color foundation + single accent attribute for UI chrome (tab indicators, block selection). Dark themes use white overlay on background for UI surface separation. No specific hex published for default dark theme, but architecture is: darkest = terminal bg, +white overlay = panels/menus, single accent color for selection/focus.
- Typography: Monospace primary (terminal content). Sans-serif for UI chrome (command palette, settings). This two-font approach maps directly to ELAI: JetBrains Mono for data/logs/paths, Inter for UI labels and controls.
- Signature interaction: Block model — each command execution creates a bounded, scrollable block. Blocks can be navigated, copied, and bookmarked individually. This maps to ELAI's ActivityWindow: each step header creates a bounded block; entries within it scroll independently if needed.
- Specific patterns we will adopt for ELAI:
  - Block-bounded step entries in ActivityWindow: each step in ELAI's activity feed is a "block" — has a header, contains chronological entries, is visually bounded by a 1px `--border-subtle` or just a step header divider. Clicking the header collapses the block.
  - Accent-color single-attribute: Warp uses one accent for all selection/focus states. ELAI does the same — `--teal` is the single focus/active accent everywhere.
  - UI surface separation via overlay tint (not background color change): modals and command palette in Warp get a subtle white-on-dark overlay. ELAI's `--bg-overlay: rgba(7,9,11,0.72)` for modal backdrops follows this pattern.
  - Minimal chrome mode: Warp has a documented "how to make UI more minimal" guide — hide status bar, hide command suggestions, disable AI suggestions until needed. ELAI should support collapsing the status bar to 0 (Ctrl+B reserved in brief, section 9).
- What to avoid: Warp supports image backgrounds and gradient effects in themes. ELAI's design brief explicitly bans both. Warp's AI suggestion popovers that appear inline as you type add visual noise we do not want in the activity feed.

### 5. Superhuman

- URL: https://blog.superhuman.com/how-to-design-delightful-dark-themes/ and https://getdesign.md/superhuman/design-md
- Layout pattern: Three-column desktop app: narrow sidebar (folders/labels), wide list column, full reading pane. Dense but uncluttered — achieves density through tighter line-height and smaller text rather than data compression or collapsing. Keyboard shortcuts shown inline next to items until the user starts using them.
- Color palette: Avoids pure black (`#000000`). Uses `#010101` or similar near-black for OLED efficiency. Primary text at 90% white opacity (not `#FFFFFF`) to prevent halation. Secondary text at ~65% white opacity. Five shades of gray from darkest surface to lightest overlay. Accents: deep version of brand hue, reducing lightness and increasing saturation vs light-mode equivalent. Published brand palette: Simple Navy Blue (#0068B1), Sacred White (#FEFEFC), Ukraine Yellow (#FFDD00), Chinese Yellow (#FDB603), Monza (#D3001F), Intense Ruby Red (#AF0816) — these are brand colors, not UI surface colors.
- Typography: Clean sans-serif at high density. Body text 13-14px. Secondary labels at 11-12px. Keyboard hint pills in monospace or small sans-serif. No weight above 500 in the data reading areas.
- Signature interaction: Cmd+K command palette is the primary action surface. Inline keyboard hints shown as ghost text or small pills until internalized. Split inbox with semantic tabs (Important, VIP, News, Calendar, Other) — demonstrates how to categorize dense information without visual clutter.
- Specific patterns we will adopt for ELAI:
  - 90% opacity white text instead of pure white: ELAI's `--text-primary: #D7DCE3` is near-white (not pure `#FFFFFF`), consistent with Superhuman's anti-halation research. This is grounded in the 50% of users with astigmatism note from the Superhuman dark themes article.
  - Near-black with tint: `#010101`–`#0F0F0F` range. ELAI's `--bg-void: #07090B` falls in this range. Confirmed correct.
  - Inline keyboard hints as pills: shown until the user activates the shortcut, then fade. ELAI's `Kbd.svelte` should show on hover of interactive elements, fade after first use.
  - Progressive disclosure through semantic grouping: Superhuman's split inbox maps to ELAI's drawer system — each drawer (KNOWLEDGE, REASONING, ORCHESTRATION, etc.) is a semantic grouping of dense data, surfaced only when relevant.
- What to avoid: Superhuman uses purple/blue glow accents for their brand aesthetic. DESIGN-BRIEF explicitly bans purple. The Superhuman "purple glow" is their brand differentiator but is the wrong tone for ELAI's restraint-first philosophy.

### 6. Kimi Moonshot AI

- URL: https://www.kimi.com/, https://www.figma.com/community/file/1481936358089337387/kimi-ui-screens
- Layout pattern: Left sidebar (narrow, logo-forward) + full-width conversation area. Sidebar contains: app logo/brand at top, navigation items (New Chat, Slides, Websites, Docs, Deep Research, Sheets, Agent Swarm), expandable sections. Main area is a single large content panel with "Ask Anything..." prompt field. No secondary panel.
- Color palette: Dark near-black background. Exact hex values not extractable from static page fetch. The design follows the modern AI dark chat app pattern: very dark base surface (~`#0D0D10` range), clean white text, teal or blue accent for interactive elements and the brand mark.
- Typography: Clean sans-serif navigation labels. Logo rendered in brand typography, larger and visually prominent at sidebar top.
- Signature interaction: Logo dominates the top of the sidebar — it is the first thing the eye lands on, and it anchors the entire navigation column. Sidebar items use text labels (unlike icon-only sidebars). Navigation items do not use icons — this is notable contrast with Arc and Linear.
- Specific patterns we will adopt for ELAI (the "Kimi-inspired" reference in the brief):
  - Logo-forward sidebar top: the ELAI wordmark (thin weight 200, ~28px tall, teal) at the very top of the 64px sidebar, separated from icons by a 1px `--border-subtle` divider. This is a direct Kimi influence per the brief.
  - ELAI deviates from Kimi by using icon-only navigation (no text labels) — this is the more information-dense approach appropriate for a developer tool versus a consumer AI chat app.
  - Dark near-black primary surface consistent with Kimi's aesthetic.
  - Single large conversation/work area — ELAI doubles this to two panels (the key innovation over Kimi's single-panel layout).
- What to avoid: Kimi uses text labels in the sidebar navigation. ELAI's brief specifies icon-only to maximize the narrow 64px column, with tooltip-on-hover for labels.

### 7. Cursor IDE

- URL: https://cursor.com/, https://forum.cursor.com/t/megathread-cursor-layout-and-ui-feedback/146790, https://cursor.com/docs/configuration/themes
- Layout pattern: VS Code-derived layout: left activity bar (icon column, ~48px), left sidebar panel (file explorer, search, etc., ~240px expandable), main editor area (primary panel), right AI panel (chat/composer, ~340px). Bottom panel for terminal/output. Users report strong preference for customizable layout with persistent settings.
- Color palette: "Anysphere Dark" as the default theme — near-black backgrounds with high-contrast text. "Midnight" variant is the deepest dark. Three theme variants: Light, Dark, Midnight. No hex values published in the docs. Community notes `#1e1e1e` range for editor backgrounds (inherited from VS Code), with slightly adjusted sidebar backgrounds.
- Typography: Monospace primary in editor. Sans-serif for UI chrome. Activity bar uses 24px icons. Command palette (Cmd+Shift+P) in a floating overlay with fuzzy search.
- Signature interaction: Cmd+Shift+P command palette as primary action. AI agent panel docked right — this is the key pattern for ELAI's dual-panel design: put the AI interaction (CommandWindow) in the secondary panel, code/plan output in the primary. Users who customized Cursor report exactly this preference: "editor on left, AI on right."
- Specific patterns we will adopt for ELAI:
  - Dual-panel model (editor + AI): the strongest validation for ELAI's two-workspace-panel design. Cursor users consistently report the left=work, right=AI layout as natural and productive. ELAI maps this to PRIMARY=ACTIVITY (what the engine is doing) + SECONDARY=COMMAND (where you interact with it).
  - Activity bar as semantic navigation: 48-64px icon column, each icon a distinct "view mode." ELAI's 64px sidebar is an activity-bar clone. 44px hit targets, 20px icons, active left-edge bar.
  - Cmd+Shift+P shortcut: ELAI uses Ctrl+Shift+P for the command palette (section 9 of brief) — this is the standard established by VS Code/Cursor.
  - Contextual panel layout: Cursor 2.0 introduced mode-aware default layouts (Agent mode vs Editor mode). This directly maps to ELAI's gate-mode-aware layout reconfiguration (INTERACTIVE vs AUTONOMOUS rearranges default panel assignment, section 7 of DOMAIN-CONTEXT.md).
- What to avoid: Cursor's persistent bottom terminal panel. ELAI has no persistent bottom panel — terminal is one of the drawer types, not a fixed pane.

### 8. Vercel Dashboard

- URL: https://vercel.com/changelog/dashboard-navigation-redesign-rollout, https://vercel.com/geist/theme-switcher
- Layout pattern: New design (default from February 2026): resizable sidebar replacing horizontal tabs. Sidebar streamlines navigation and shipping workflows. Items organized by developer workflow frequency. Instant toggle between team and project views. Mobile: floating bottom bar for single-handed use.
- Color palette: Geist design system. Dark mode uses Background 1 / Background 2 hierarchy with 10 color scales. Exact hex values require browser inspection of the live Geist site. Architecture: multiple background elevation levels, component-level backgrounds, border colors, text hierarchy. Accent: Vercel blue in light mode, lighter variant in dark mode. Not teal — this is a data point that ELAI's teal choice diverges from Vercel, which is intentional.
- Typography: Geist font (Vercel's custom sans-serif). For ELAI we substitute Inter, which is functionally similar.
- Signature interaction: Deploy stream as a real-time log tail — each deploy step appears as a line with a status indicator (running = animated dot, success = green checkmark, failed = red x). This is the direct model for ELAI's ActivityWindow step stream with teal breath-pulse underline on the current step.
- Specific patterns we will adopt for ELAI:
  - Deploy stream as activity feed: Vercel's real-time deploy log is structured as: step header + indented log lines + terminal status badge. ELAI's ActivityWindow maps this exactly: step header + typed entries ([reasoning], [edit], [test], etc.) + step completion badge.
  - Status badges inline in the stream: small, monochrome, semantic. Running steps get a spinner in Vercel (ELAI replaces with breath-pulse). Completed steps get a status badge. This inline badge pattern maps to ELAI's `[validate]` / `[test]` one-liners with Tier result pills.
  - Resizable sidebar: Vercel moved from fixed tabs to resizable sidebar in 2026. ELAI's draggable panel divider and collapsible secondary panel follow the same trajectory.
  - Status page pattern: Vercel's status page (vercel-status.com) uses simple colored dots for system health — the exact model for ELAI's circuit breaker grid (5 models, each with colored state dot).
- What to avoid: Vercel's deploy stream shows raw log lines (very dense). ELAI's activity feed should group entries by type with visual hierarchy rather than a flat log.

### 9. Charm / Bubble Tea (Bonus)

- URL: https://github.com/charmbracelet/bubbletea and https://github.com/charmbracelet/bubbles
- Layout pattern: Terminal-native full-viewport or inline TUI layouts. Elm architecture (Model-Update-View). Component library (bubbles) includes: spinner, progress bar, table, list, textinput, viewport (scrollable area), paginator, stopwatch, timer.
- Color palette: Lip Gloss v2 for styling — light/dark background detection, 256-color and true-color support. No fixed palette; apps supply their own. Charm's own branding uses soft pink/magenta and dark backgrounds. For ELAI, the TUI pattern is an inspiration source for the status bar design, not the color palette.
- Signature interaction: Status bars in Bubble Tea TUIs are typically a single-line footer with left/right sections, monospace font, key bindings shown as `[q] quit [enter] select`. This is exactly the ELAI status bar: left = project · branch · FSM-state, right = model · VRAM · session-id. The terminal pattern validates ELAI's 24px / 11px mono status bar.
- Specific patterns we will adopt for ELAI:
  - Two-section status bar (left context, right stats): the TUI convention for status bars is left=current context (what you're in), right=system stats (resource usage). ELAI's status bar follows this exactly.
  - Key binding hints visible in status bar: Bubble Tea TUIs show `[key] action` in the footer. ELAI can show 1-2 critical shortcuts in the status bar ghost text area.
  - Spinner alternatives: Bubble Tea's spinner component is the default loading indicator in TUIs. ELAI explicitly replaces spinners with a breath line — this is a deliberate elevation of the TUI convention to a more refined, desktop-native pattern.
- What to avoid: Bubble Tea's spinner component. ELAI has zero spinners by design.

### 10. Zed Editor (Bonus)

- URL: https://zed.dev/, https://zed.dev/docs/themes
- Layout pattern: Minimalist code editor. UI "fades into the background." Left sidebar (file tree, search), main editor, optional right panel. Far less chrome than VS Code. Activity bar icons are minimal. Very low sidebar width. The philosophy: "out of your face."
- Color palette: Two built-in themes (light/dark). Community extensions for themes. No published hex values in public docs. Architecture: JSON-based theme tokens covering syntax, UI, and terminal colors. Zed One Dark is the closest to ELAI's palette direction.
- Typography: System monospace for editor, system sans-serif for UI. Very lightweight type treatment — weights barely exceed 400 in the UI chrome.
- Signature interaction: Native performance enables immediate response — no debounce on search, no loader on file open. The "instant" feel is a design intent, not just engineering. For ELAI, stubs should simulate 80-240ms latency (per domain context section 1) to feel real but not sluggish.
- Specific patterns we will adopt for ELAI:
  - "UI fades into background" principle: the workspace chrome (sidebar, tab bar, status bar) should never compete with the content panels. ELAI achieves this via `--text-secondary` for all inactive UI elements and `--border-subtle` for structural dividers.
  - Theme as JSON tokens: Zed's theme system is pure JSON tokens mapping semantic names to hex values. ELAI's `tokens.css` approach is the CSS equivalent — semantic names (`--teal`, `--bg-surface`) not hex literals in components.
  - Minimal sidebar width: Zed's sidebar is narrower than VS Code. ELAI's 64px is narrower than Cursor's ~48px activity bar + ~240px explorer. For ELAI, 64px is the entire sidebar (no secondary file-tree panel next to it).
- What to avoid: Zed has no status bar that "talks" in depth. ELAI's status bar must surface live engine data (`EXECUTING bs-3.2.1.4 · auditor consuming 42% GRC budget`) — this is a major differentiator from Zed's minimal status bar.

---

## 2026 trend synthesis

- Near-black-with-tint backgrounds are the established best practice: pure black (#000000) increases halation and eye strain, especially for users with astigmatism (~50% of population per [Superhuman dark themes research](https://blog.superhuman.com/how-to-design-delightful-dark-themes/)). The recommended range is `#07090B` to `#161616` with a subtle tint (zinc-green, navy, charcoal). ELAI's `--bg-void: #07090B` is within the optimal range. [Muzli dark mode guide](https://muz.li/blog/dark-mode-design-systems-a-complete-guide-to-patterns-tokens-and-hierarchy/), [Colorhero dark palettes](https://colorhero.io/blog/dark-mode-color-palettes-2025)
- Luminance hierarchy replaces box shadows: "As a surface elevates, it gets lighter, not more shadowed." Four-level elevation: base `#0F0F0F`, raised `#1A1A1A`, overlay `#242424`, modal higher still. ELAI's 5-step background ladder (`--bg-void` through `--bg-input`) implements this pattern correctly. No `--shadow-*` tokens exist in the ELAI system, which is confirmed best practice. [Dark mode design systems practical guide](https://medium.com/design-bootcamp/dark-mode-design-systems-a-practical-guide-13bc67e43774)
- Dark-first as a distinct design context, not a variant: the tools shipping premium dark UIs treat dark mode as the primary design system. Light is the variant. This means elevation logic, border logic, and color token choices are all reasoned from dark starting assumptions. "Dark is not an inversion of light — it's a first-class context with its own elevation language." [Dark Mode Design Systems practical guide](https://medium.com/design-bootcamp/dark-mode-design-systems-a-practical-guide-13bc67e43774), [Muzli dark mode guide](https://muz.li/blog/dark-mode-design-systems-a-complete-guide-to-patterns-tokens-and-hierarchy/)
- Calm technology / quiet UI is the dominant 2026 direction: "Gamification will be replaced with calmer micro-interactions, and more extravagant motion controls will be surpassed by strategic ones." Cognitive clarity over sensory richness. "White space, visible and easy defaults, conveniently placed hierarchies of content." This is the design language ELAI should embody — a tool that shows you what the engine is doing without demanding your attention. [Envato 2026 UX/UI trends](https://elements.envato.com/learn/ux-ui-design-trends)
- Teal and steel-blue are validated accent choices for 2026 developer tools: "Transformative Teal" is named as a 2026 accent trend (resilience, earth-first). Desaturated teal/blue enhances visibility in dark contexts without neon overreach. "Desaturated colors such as blue, purple, or teal enhance the visibility of all UI elements" in dark mode. The ELAI teal `#1E8A7A` (muted, mid-saturation) and steel `#5A7A9E` are positioned within the desaturated-accent best practice. [UI Color Trends 2026](https://updivision.com/blog/post/ui-color-trends-to-watch-in-2026), [Colorhero 2025](https://colorhero.io/blog/dark-mode-color-palettes-2025)
- Accent saturation must be adjusted for dark backgrounds: "Colors vibrant on light backgrounds may be too saturated or neon on dark surfaces." The correct technique: reduce lightness, increase saturation by 10-20% vs light mode equivalent. ELAI's `--teal: #1E8A7A` is already a muted, non-neon teal — well within this guideline. The `--teal-bright: #2BAE98` secondary variant is the more visible version, used sparingly per the brief. [Colorhero 2025](https://colorhero.io/blog/dark-mode-color-palettes-2025), [UI Color Trends 2026](https://updivision.com/blog/post/ui-color-trends-to-watch-in-2026)
- Motion that explains, not performs, is the 2026 consensus: "Progress indicators clarify processing states rather than providing ornamental animations." Spinners are an anti-pattern when a breath line can communicate the same state with less visual noise. The ELAI `breath` keyframe (opacity 0.2->0.55->0.2, 2.2s, infinite) directly implements this pattern — it signals "engine is alive and working" without demanding focus. [Envato 2026 UX/UI trends](https://elements.envato.com/learn/ux-ui-design-trends), [Muzli dark mode guide](https://muz.li/blog/dark-mode-design-systems-a-complete-guide-to-patterns-tokens-and-hierarchy/)

---

## Applied decisions for ELAI

- Sidebar pattern: based on Arc Browser, Kimi Moonshot, and Linear, the 64px icon-only sidebar with logo-at-top and active left-edge inset bar is the correct pattern for a developer tool that needs persistent navigation without consuming work surface. No text labels in the sidebar — tooltip-on-hover provides labels. Mode dots at bottom are informed by Arc's space-switcher dot row.
- Tab system: based on Linear (compact, rounded, 1px active bottom border, `--bg-surface` elevation) and Cursor (workspace-level tab bars per panel), each panel has a 32px tab bar with ghost tabs and a 1px teal bottom border on the active tab. Tabs drag between panels. The [+] affordance at the end opens a window-type picker overlay.
- Motion language: based on the 2026 Envato trend consensus and Superhuman's dark theme research, all motion is functional-only. Enter animations: `enter-fade` (180ms ease-out) and `enter-lift` (220ms ease-out). The single loading indicator is the `breath` line (1px teal, opacity pulse, 2.2s) at the bottom of the status bar during EXECUTING state. No spinners, no skeleton loaders, no bounce. Tab switch: 100ms fade-out + 140ms fade-in (no slide).
- Information density (drawer vs window): based on Superhuman's progressive disclosure approach (split inbox as semantic grouping) and Vercel's deploy stream, dense engine subsystem data lives in drawers, not windows. 10 drawers each map to one engine subsystem. Windows show the primary workflow data (activity stream, plan tree, command history). Drawers slide from the right (320px) and dim the workspace by 10% — progressive disclosure, accessed on demand.
- Status bar that "talks": based on Warp's block-bounded output model, Vercel's deploy stream status, and Bubble Tea's TUI status-bar convention, ELAI's 24px mono status bar is the always-visible truth surface. It shows project-name · branch · FSM-state on the left and model · VRAM · session-id on the right. The FSM state badge is UPPERCASE with 0.08em letter-spacing (matches the TUI convention for state labels). When EXECUTING, the 1px breath line beneath the bar is the only loading indicator in the entire application. This is a more precise, data-dense status bar than any of the references use.
- Accent color: the teal `#1E8A7A` + steel `#5A7A9E` duo is confirmed correct by three 2026 trend sources. Teal is cited as a 2026 resilience/tech accent color. Steel-blue is validated as the "informational/secondary" role. The decision to ban purple (specified in DESIGN-BRIEF) is supported by Superhuman's use of purple as a brand differentiator that would make ELAI look like a Superhuman clone. The teal/steel duo is distinct from Vercel (blue), Linear (warm gray/blue), Raycast (warm gradient), and Superhuman (purple glow). No conflicting evidence found.
- Unique differentiators — 3 concrete things that make ELAI NOT a Linear or Raycast clone:
  1. Phase dots row as MONITOR overview: the `. * - ! o` character-based phase visualizer in MonitorWindow. No comparable pattern exists in Linear, Cursor, Vercel, or any other reference. It is ELAI-native, derived from the engine's BabyStep-to-Concept hierarchy, and communicates execution state at a glance with minimal visual weight.
  2. Breath line as sole loading indicator: every other reference (Vercel, Cursor, Warp) still uses some form of spinner, skeleton loader, or progress bar. ELAI's single 1px teal opacity-pulsing line at the bottom of the status bar during EXECUTING is uniquely minimal. It communicates "active" without demanding visual focus — a deliberate inversion of the "show me activity" principle that most dev tools follow.
  3. Domain-specific drawer system (10 drawers, engine-subsystem-mapped): neither Linear, Arc, Cursor, Zed, nor any reference uses drawers that map 1:1 to internal engine subsystems. KNOWLEDGE, REASONING, ORCHESTRATION, RULES, RESEARCH, PROVIDERS, PROJECT-INTELLIGENCE, ANTI-STALENESS, TOOLS-HOOKS, WEB-QUALITY are ELAI-specific data surfaces. This is what makes the dashboard a genuine observability layer for the engine, not a generic project management UI.

---

## Sources

All URLs actually fetched or searched during this research session:

- [How we redesigned the Linear UI (Part II)](https://linear.app/now/how-we-redesigned-the-linear-ui)
- [A calmer interface — Linear design refresh](https://linear.app/now/behind-the-latest-design-refresh)
- [A design reset (Part I) — Linear](https://linear.app/now/a-design-reset)
- [Linear design — LogRocket](https://blog.logrocket.com/ux-design/linear-design/)
- [Arc Browser: Rethinking the Web Through a Designer's Lens](https://medium.com/design-bootcamp/arc-browser-rethinking-the-web-through-a-designers-lens-f3922ef2133e)
- [Spaces: Distinct Browsing Areas — Arc Help Center](https://resources.arc.net/hc/en-us/articles/19228064149143-Spaces-Distinct-Browsing-Areas)
- [Arc Browser UI UX Design — SaaSUI](https://www.saasui.design/application/arc-browser)
- [UX analysis of Arc, Opera, Edge — LogRocket](https://blog.logrocket.com/ux-design/ux-analysis-arc-opera-edge/)
- [Raycast — Your shortcut to everything](https://www.raycast.com/)
- [Raycast Windows launch](https://www.raycast.com/changelog/windows)
- [Raycast API UI reference](https://developers.raycast.com/api-reference/user-interface)
- [Raycast for designers — UX Collective](https://uxdesign.cc/raycast-for-designers-649fdad43bf1)
- [Warp Terminal themes design process](https://www.warp.dev/blog/how-we-designed-themes-for-the-terminal-a-peek-into-our-process)
- [How to make Warp UI more minimal](https://docs.warp.dev/university/developer-workflows/beginner/how-to-make-warps-ui-more-minimal)
- [How to design delightful dark themes — Superhuman](https://blog.superhuman.com/how-to-design-delightful-dark-themes/)
- [Superhuman keyboard shortcuts](https://help.superhuman.com/hc/en-us/articles/45191759067411-Speed-Up-With-Shortcuts)
- [Kimi AI — kimi.com](https://www.kimi.com/)
- [Kimi UI Screens — Figma Community](https://www.figma.com/community/file/1481936358089337387/kimi-ui-screens)
- [Cursor IDE](https://cursor.com/)
- [Cursor themes documentation](https://cursor.com/docs/configuration/themes)
- [Cursor layout and UI feedback megathread](https://forum.cursor.com/t/megathread-cursor-layout-and-ui-feedback/146790)
- [Vercel dashboard navigation redesign rollout](https://vercel.com/changelog/dashboard-navigation-redesign-rollout)
- [Vercel Geist theme switcher](https://vercel.com/geist/theme-switcher)
- [Vercel Geist colors](https://vercel.com/geist/colors)
- [Bubble Tea — charmbracelet](https://github.com/charmbracelet/bubbletea)
- [Bubbles — TUI components](https://github.com/charmbracelet/bubbles)
- [Zed Editor](https://zed.dev/)
- [Zed themes documentation](https://zed.dev/docs/themes)
- [Zed appearance documentation](https://zed.dev/docs/appearance)
- [Introducing Datadog dark mode](https://www.datadoghq.com/blog/introducing-datadog-darkmode/)
- [Dark Mode Design Systems — Muzli Blog](https://muz.li/blog/dark-mode-design-systems-a-complete-guide-to-patterns-tokens-and-hierarchy/)
- [Dark mode color palettes 2025 — Colorhero](https://colorhero.io/blog/dark-mode-color-palettes-2025)
- [UI Color Trends to Watch in 2026 — Updivision](https://updivision.com/blog/post/ui-color-trends-to-watch-in-2026)
- [UX/UI design trends 2026: calm interfaces — Envato](https://elements.envato.com/learn/ux-ui-design-trends)
- [Dark Mode Design Systems: Practical Guide — Medium Bootcamp](https://medium.com/design-bootcamp/dark-mode-design-systems-a-practical-guide-13bc67e43774)
- [Dark mode best practices 2026](https://www.tech-rz.com/blog/dark-mode-design-best-practices-in-2026/)
- [Dark Mode UX 2025: Comfort, Accessibility, Control](https://www.influencers-time.com/dark-mode-ux-in-2025-design-tips-for-comfort-and-control/)
- [Best Dark Mode UI Design Examples 2025 — UIinkits](https://www.uinkits.com/blog-post/best-dark-mode-ui-design-examples-and-best-practices-in-2025)
- [Dark mode UI design: Best practices — LogRocket](https://blog.logrocket.com/ux-design/dark-mode-ui-design-best-practices-and-examples/)
- [30 Best Dark Mode UI Design Examples 2024 — Mockplus](https://www.mockplus.com/blog/post/dark-mode-ui-design)
- [user-provided palette image (greens, teals, deep blues, gray-to-black ladder)]

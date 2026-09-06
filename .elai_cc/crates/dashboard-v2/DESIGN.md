# ELAI Dashboard — Design System (source of truth)

This file is the definitive style guide for the ELAI dashboard. Future sessions and LLMs working on the UI MUST read it before making visual changes. DOMAIN-CONTEXT.md describes *what data each window surfaces*; this file describes *how the dashboard looks and feels*.

When in doubt, follow the principle, not the precedent. Existing code that violates these rules is a fix-on-touch debt, not a license.

---

## 1. The five aesthetic principles

1. **Minimalism first**. The dashboard is a quiet place to watch a deterministic engine work. Default chrome is invisible. Anything decorative competes with the orbit and the cards — remove or hover-reveal it.
2. **Hover-reveal over hidden**. Non-essential controls are present in the layout but at very low opacity (or `opacity: 0`) until the user hovers their parent panel. Layout never shifts.
3. **One accent**. Teal (`--teal` / `--teal-bright` / `--teal-line` / `--teal-ghost`) carries every interactive affordance. Steel (`--steel`) is the only secondary accent. There is no purple, no orange-as-decoration, no rainbow status colors. `--warn` and `--fail` are reserved for *actual* warnings and failures and never used as text-decoration tints.
4. **Smooth, slow, intentional motion**. Breath lines, dash flow, halo pulse, and constellation breathe — all 2-12 s cycles. No spinners. No pulse-bouncing dots. No left-to-right shimmer skeletons. Every animation has a `@media (prefers-reduced-motion: reduce)` escape.
5. **Deterministic positioning**. Same task = same layout. Random visual elements (jitter, shooting-star fire times, constellation anchors) are seeded from stable IDs (`task_id`, `concept_id`, etc.) via the LCG in `src/lib/orbit/lcg.ts`. Never use `Math.random` in the orbit or starfield.

---

## 2. Tokens — the only colors allowed

All colors live in `src/styles/tokens.css`. **No hex literals appear outside that file.** Tailwind-style ad-hoc colors are forbidden.

### Backgrounds (deepest → most lifted)
- `--bg-void` — window chrome, status bar, out-of-panel voids.
- `--bg-base` — primary panel surface.
- `--bg-surface` — elevated cards, slash palette, drawers.
- `--bg-hover` — hover row.
- `--bg-input` — input fields, code blocks, the chat shell interior.

Depth is expressed by stepping these tokens, never with `box-shadow`. The only allowed shadow is `--ring` for focus and the slash-palette drop shadow.

### Accent — teal
- `--teal` (#1E8A7A) — active states, focus rings, brand wordmark.
- `--teal-bright` (#2BAE98) — important badges, hovered primary text, EXECUTING state.
- `--teal-line` — the canonical 1-px outline for outlined buttons / focused inputs.
- `--teal-ghost` — hover backgrounds for outlined buttons.
- `--teal-soft` — glows, halo strokes.

### Secondary — steel
- `--steel`, `--steel-soft`, `--steel-ghost`. Reserved for informational text, links, keyboard hints. **Never used as a status color** (states get their dedicated state token).

### Text (warm-neutral, never pure white)
- `--text-primary` (#DCE5E8), `--text-secondary` (#95A1A8), `--text-ghost` (#5E6B73), `--text-disabled` (#3A4148).

### State, gate, rule, GRC, tier, bubble, effect, research palettes
All FSM states, GRC step types, source-quality tiers, BubbleUp action variants, evidence-gate verdicts, DKG effect tags, and research lifecycle stages have their own dedicated tokens. **Never reuse one for another purpose.** Example: `--state-auditing` is a desaturated lavender for the AUDITING badge only — it is NOT a generic purple accent.

---

## 3. Typography

- `--font-sans` — Inter — body copy, descriptions, titles.
- `--font-mono` — JetBrains Mono — code, file paths, IDs, timestamps, command strings, status badges, all keyboard hints.
- `--font-display` — Inter at thin (200) — used only for the ELAI wordmark.

### Scale (only these sizes)
- 9 px — keyboard hints, ambient micro-labels.
- 10 px — caps-uppercase tags, tier hints, dot-row labels.
- 11–12 px — most UI text (status bar items, button labels, card body, captions).
- 13 px — section titles, project names.
- 14 px — card titles (intervention, error).
- 18 px — the only large size, used for the ELAI wordmark on the project selector (where it's actually 56 px).

Letter-spacing: 0.04–0.10 em on uppercase mono text only. Body text is at 0.

---

## 4. Buttons (canonical patterns)

There are **three button styles**, applied via the shared `Button` primitive (`src/lib/ui/Button.svelte`) or via the inline `.btn-approve` / `.btn-outlined` classes in CommandWindow. They MUST look identical wherever they appear.

### Primary action — outlined teal
- Used for "Approve", "Run now", "Send", "Save".
- `<Button variant="primary">` — transparent background, 1 px `--teal-line` border, `--teal` text.
- Hover: `--teal-ghost` background, `--teal` border, `--teal-bright` text.
- The Approve button in `CommandWindow` uses the legacy `.btn-approve` class which produces an identical look.

### Secondary action — outlined neutral
- Used for "Reject", "Modify", "Take over", "Skip", "New Project", "Stop server", disambiguation pills.
- `<Button variant="ghost">` — transparent background, 1 px `--border-default` border, `--text-secondary` text.
- Hover: `--teal-ghost` background, `--teal-line` border, `--text-primary` text.
- `.btn-outlined` in CommandWindow is the same.

### Tertiary action — text-only
- Used for inline links and tertiary actions only (e.g. "show all", "use template").
- `<Button variant="text">` — transparent background, no border, `--teal` text, underline on hover with 3 px underline-offset.

**Anti-patterns**:
- No solid-fill buttons except the slash-toggle when the palette is open (small icon button with teal-ghost bg).
- No gradient backgrounds on buttons.
- No drop shadows on buttons except the focus ring.

---

## 5. Card patterns (in CommandWindow stream)

The right panel is a vertical list of card types. They share a strict layout grid:

- **Padding**: `var(--s-4) var(--s-5)` (16 × 20 px). Every card type indents to this baseline.
- **Border radius**: `var(--r-md)` (6 px).
- **Background**: transparent at rest. On hover: `rgba(16,20,26,0.35)` and a 1-px `rgba(255,255,255,0.05)` outline.
- **Mount animation**: 1.6 s fade of a teal-ghost outline (`@keyframes cardGlow`).
- **No top-edge gradient lines.** No left-color-stripe accents. The hover outline + body content carry all the affordance.

### Card types (mirror DOMAIN-CONTEXT.md §2 COMMAND)
| Kind | Shape |
|---|---|
| `user` | Right-aligned plain text + mono timestamp. No card chrome. |
| `status` | One-line: arrow `→` + bracketed lowercase FSM state in teal + body text + optional step-id mono. Indent matches `.orbit-card` so it doesn't read as off-grid. |
| `intervention` | Pulse dot (teal) + 14 px sans title + body + optional file path mono + optional diff block + 4-button row (Approve primary, Reject/Modify/Redirect ghost). |
| `error` | Pulse dot (**teal**, never warn-orange) + bracketed mono `[error: source]` in `--text-primary` + body + optional step-id + 3-button row (Let it retry / Take over / Skip — all ghost). |
| `disambiguation` | Italic question + vertical list of ghost-button option pills. |
| `steering` | Single italic line "Noted: …" with center-dot prefix. Indented to match `.orbit-card`. Use sparingly — it's an advisory, not a status. |

**Diff blocks** (inside intervention cards): 1 px `--teal-line` left strip (not 3 px), `rgba(26,32,41,0.6)` background, 12 px mono. Diff `+` lines tinted `rgba(30,138,122,0.8)`; `-` lines `rgba(176,69,69,0.8)`.

---

## 6. Chat shell + slash palette

The CommandWindow input is a **chat shell** — an outlined container that frames the input and the slash-toggle. Defined in `CommandWindow.svelte` `.input-shell`:

- 1 px `--border-default` border, `--bg-input` background, `var(--r-md)` radius.
- Margin `var(--s-3) var(--s-4)`.
- Focus-within: border lifts to `--teal-line`, background to `rgba(26,32,41,0.85)`.
- Slash toggle, textarea, and send-button live inside it. The textarea is borderless; the shell carries the visual.

The **slash palette** (`.slash-palette`) is a floating dialog anchored above the chat shell:

- Position: `bottom: calc(100% + 6px)`.
- Background `--bg-surface`, 1 px `--border-default`, `var(--r-md)`, drop shadow `0 4px 24px rgba(0,0,0,0.45)` (the only place where `box-shadow` is allowed for depth).
- **Filter input at top** (`.slash-filter`) — placeholder "Filter actions...".
- **Grouped sections** under category labels (uppercase mono 10 px). Categories: WORKFLOW, PROJECT, HELP. New commands are added to the appropriate group in `CommandWindow.svelte` `SLASH_DEFS`.
- **Row layout**: grid `auto 1fr auto` — `slash-row-label` (mono command name without leading slash) + `slash-row-desc` (sans, ellipsized) + optional `slash-row-hint` (mono ghost).
- **Auto-open**: typing `/` at the start of the textarea opens the palette automatically. Esc closes it.
- **Keyboard**: Arrow up/down moves selection, Enter executes, filter input takes focus on open.

---

## 7. Hover-reveal chrome (the staged-visibility rule)

Non-essential UI is present in the layout but at low/zero opacity until the user mouses over the relevant panel. **Layout never shifts on reveal — only opacity transitions.** Transitions are 220–240 ms with `var(--ease-std)`.

Things that hover-reveal:
- **Tab bar** in `Panel.svelte` — `opacity: 0.35` default, full on `.panel:hover` or `:focus-within`. Tab-bar background + border-bottom also fade in.
- **New-tab `+` button** — `opacity: 0` default, full on panel hover/focus.
- **Activity-host header** ("CONCEPT …") and **mode-toggle pill** ("ORBIT …") — `opacity: 0` default, full on `.activity-host:hover`.
- **Zoom HUD** (bottom-right of the orbit) — `opacity: 0` default, full on `.orbit-canvas-root:hover`.
- **Status bar verbose items** — branch / gate / model / VRAM / session-id are wrapped in `.verbose` with `opacity: 0`, full on `.status-bar:hover` or `:focus-within`. Project + state + mode are always visible.

Things that **never** hover-reveal:
- The active orbit / active card stream (always full opacity).
- The FSM state badge in the status bar.
- The send button when the input has content.
- The breath line during EXECUTING.

---

## 8. Orbit visualizer (the centerpiece)

The orbit lives in `ActivityWindow.svelte` and is composed by `OrbitCanvas.svelte` + the three mode components (`OrbitMode1Plan` / `OrbitMode2Map` / `OrbitMode3Research`).

### Three modes, FSM-driven

| Mode | When | Component | Default scale |
|---|---|---|---|
| `plan` | Default for any task in PLANNING / TESTING / EXECUTING / AUDITING | `OrbitMode1Plan` | 1.0 |
| `map` | FSM=IDLE with `pagerank_nodes` (existing project) | `OrbitMode2Map` | 1.0 |
| `research` | FSM=RESEARCHING | `OrbitMode3Research` | 0.7 (auto zoom-out so tendrils outside the orbit are visible) |

Mode resolution in `OrbitCanvas`: explicit `mode` prop wins; otherwise derived from `fixture.fsm`.

### Geometry — polar rings, deterministic jitter

- Three concentric rings at `R_PHASE = 95`, `R_STEP = 162`, `R_BABY = 226`.
- Phase angles distributed evenly around the circle (`phaseAngle(i, n)`).
- Step angles spread within ~70 % of the parent phase's angular footprint (`stepAngleFor`).
- Baby angles spread within ~85 % of the parent step's footprint (`babyAngleFor`).
- All node positions get **per-id seeded jitter**: `±14 px` on radius, `±0.07 rad` on angle, computed by `lcg(combineSeeds(taskId, nodeId))` so the same task always reproduces the same constellation.
- Edges are quadratic Bezier curves with **per-edge bow-factor variance** (`bowFor(idA, idB, base)`) so curves don't all look identical.

### "Constellation, not string"

The orbit MUST read as a constellation regardless of how few phases the task has. Rules to follow:

- **Render every node**, not just the active branch. Every step + every baby across ALL phases is drawn as a backdrop dot (`r=2.4` for steps in `--steel`, `r=1.6` for babies in `--text-secondary`).
- **Spokes** from the centre to every phase (faint `--border-subtle` curves) — keeps the radial structure visible at all zoom levels.
- **Phase labels are always-visible**, with `.label--ambient` for inactive ones (`--text-ghost`, opacity 0.65).
- The **active branch** is distinguished by bloom + dash-flow + bright teal — not by being the only thing drawn.
- For 2-phase concepts, place the active concept at a **non-top angle** (e.g. index 2 of 4) so the active branch doesn't bias the visual upward.

### Motion

- `constellation-breathe` — whole concept group scales 1.000 → 1.020 over 11 s.
- `ambient-flow` — inactive edges have a `2 6` dasharray flowing left-to-right at 18 s.
- `dashflow` — the active baby-step path flows at 1.6 s.
- `halo-pulse` (4 s) + `halo-ambient` (7 s) — concentric halos around the centre node.
- `bubble-ring` (2.6 s) — colored ripple when `bubble_action` is set.
- `breath` — used in: status-bar EXECUTING line, mount card-glow, breath-dot in idle states.
- All decorate; none are loading indicators.

### Vignette + starfield

- Orbit canvas root has a single soft additive teal radial bloom (`rgba(30,138,122,0.10)` at centre, `rgba(...,0.04)` at 35 %, transparent at 70 %) layered on `--bg-base`.
- The Starfield component (`density="dense"`, `tone="mixed"`, `centered`) draws ~320 pinpoint stars + an 8-stop quadratic-ish vignette via `.center-mask`. **Always use these props for the orbit; do NOT replace them with custom gradients.**
- Stars use cool-white / pale-steel / pale-teal palette; ~10 % bright with halo, ~2 % twinkle.
- Idle shooting stars fire every ~14 s when there's no active task. Trail length and angle deterministically seeded per fire-counter.

### Multi-concept constellations

- `OrbitMode1Plan` iterates ALL concepts in the fixture and places each at a `constellationAnchor` offset.
- The **active concept is always at canvas centre** (`relativeAnchor` re-bases all anchors so the active lands at `(320, 280)`); inactive concepts retain their relative offsets and render as compressed markers (single dot + faint ring + ambient label).
- Click an inactive concept's marker → it animates to centre via the `zoomToLevel` system.

### Zoom

- 4 levels: `concept` → `phase` → `step` → `baby-step`. Default scales `1.0 / 1.6 / 2.4 / 3.4`.
- Direct `requestAnimationFrame` loop in `OrbitCanvas.startZoom`. **Do not use `$effect` for the zoom animation** — it's a reactive-write-in-effect hazard in Svelte 5.
- Cubic-bezier ease-out (`easeOutCubic`), 420 ms.
- Wheel up / `+` / `=` / click-on-node = zoom in. Wheel down / `-` / Esc = zoom out.

---

## 9. ProjectSelector layout (canonical order)

From top to bottom:

1. ELAI wordmark (`--teal`, thin 200, 56 px).
2. Tagline ("a deterministic development engine").
3. Mode selector tabs (LOCAL / CLOUD / HYBRID) — underlined active tab.
4. Divider (1 px `--border-subtle`).
5. Recent projects list (5 rows max, name in sans-14, path in mono-11 ghost, relative time mono-11 ghost right-aligned, teal inset bar on hover).
6. New Project button — outlined neutral (`Button variant="ghost"` look-alike, no `+` prefix).

No breathing dot, no decorative "·" between sections. The vignette + faint corner stars handle the depth.

---

## 10. Status bar rules

`StatusBar.svelte` is 22 px tall, sits at the bottom of the workspace, has the sidebar to its left.

### Always visible (left → right)
- **Project name** (mono 10.5, `--text-secondary`).
- **FSM state badge** (mono 10.5, color from `--state-*` tokens, lowercase, letter-spacing 0.04 em).
- **Mode indicator** (4-px dot + lowercase mono label).

### Hidden by default — fade in on `.status-bar:hover` or `:focus-within`
Wrapped in `<span class="verbose">` with `opacity: 0`, transitions 220 ms:
- Branch
- Gate mode
- Model name
- VRAM % (only shown when > 70)
- Session ID (clickable, copies to clipboard)

### EXECUTING signal
- A 1-px `--teal` line sits at `bottom: 0` of the status bar with the `anim-breath` 2.2 s ease-in-out cycle.
- This is the **only loading indicator allowed in the dashboard**. No spinners. No skeletons. Anywhere.

---

## 11. Drawers, command palette, panels

- **Drawer** (`Drawer.svelte`): 360 px wide, slides from right with `drawerSlideIn` 220 ms ease-out. Backdrop is `rgba(7,9,11,0.10)` — a 10 % dimmer, never a heavy modal. Esc closes. Header is uppercase mono caps, body has 16 px padding, only one drawer open at a time.
- **CommandPalette** (`CommandPalette.svelte`, Ctrl+Shift+P): same visual language as the slash palette — search at top, grouped action sections, no decorative chrome. Don't add iconography unless it's a Lucide icon at 12 px.
- **Panel** (`Panel.svelte`): contains tabs + WindowHost. Tab-bar follows hover-reveal (§7). Drag-and-drop tab moves between panels.

---

## 12. Anti-patterns (do NOT)

- Don't use `--warn` (orange) as a text color for "[error: …]" labels. Errors get teal-dot + white-title treatment; only actual warnings (`--warn` reserved) and failures (`--fail`) get color flags.
- Don't add decorative top-edge gradient lines on cards. Cards lift via the hover outline only.
- Don't stack two radial-gradient layers to make a vignette — it bands at the boundary. Use one smooth quadratic-stop gradient.
- Don't render solid-fill primary buttons. The teal-line outline is the affordance.
- Don't add SVG SMIL `<animate>` / `<animateTransform>` / `<animateMotion>` without a `prefers-reduced-motion` escape. CSS `@media` does not gate SMIL — pass a `reducedMotion` flag down via props if needed.
- Don't use `Math.random()` in the orbit or starfield. Always seed via `lcg(hashId(stableId))`.
- Don't use spinners. Don't use bouncing dots. Don't use rainbow CSS shimmer. The breath line is the only loading indicator.
- Don't add new dependencies (Three.js, D3, gsap, framer-motion, etc.) without an explicit user yes.
- Don't bypass `tokens.css` with raw hex literals or `style="color: #..."`. Add a token if one is missing.
- Don't use `$derived(() => ...)` when you mean `$derived.by(() => ...)` — the former stores the function, the latter stores the value.
- Don't rebuild the slash palette as a new component. Use the existing one in CommandWindow; add new commands to `SLASH_DEFS` with a `category`.

---

## 13. When you need to add something new

In order of preference:

1. **Use an existing token.** If you need a new color, ask whether it's really new or whether `--steel` / `--text-secondary` / `--teal-line` already covers it.
2. **Use the shared primitive.** `Button`, `Pulse`, `Card`, `Badge`, `Input`, `Tab`, `Tabs`, `Tooltip` all live in `src/lib/ui/`. Use them rather than rolling new chrome.
3. **Match an existing pattern.** Slash palette, orbit-card alignment, hover-reveal — these are codified above. New panels follow them.
4. **If you must extend `tokens.css`:** add the token to the appropriate semantic group (state / GRC / tier / bubble / effect / research) and document its sole purpose in a comment, like the existing groups do.
5. **If you must add a dependency:** ask the user explicitly. The hard rule is no new deps without a green light.

---

## 14. How to test a visual change

1. Run `npx svelte-check --tsconfig ./tsconfig.json` in `.elai_dashboard/`. **Must be 0 errors, 0 warnings.**
2. Capture via `node screenshots/cdp_orbit_capture.mjs <out>.png` (or `cdp_selector.mjs` for the project selector).
3. Pixel-validate with `node screenshots/check_all.mjs` — `lastY=799, emptyRows=0` for any clean 1280×800 capture.
4. Never use `chrome --headless --screenshot=...` — it has a layout bug that produces 95 empty rows at the bottom.

---

## 15. Reading order for a new session

1. This file (`DESIGN.md`).
2. `DOMAIN-CONTEXT.md` — what each window/drawer surfaces.
3. `src/styles/tokens.css` — the actual token definitions.
4. `src/lib/orbit/` — the orbit module (start at `OrbitCanvas.svelte`, then `OrbitConcept.svelte`, then `types.ts`).
5. `src/lib/windows/CommandWindow.svelte` — the canonical card / chat-shell / slash-palette implementation.

When in doubt, follow the orbit + CommandWindow patterns. They are the reference for the rest of the dashboard.

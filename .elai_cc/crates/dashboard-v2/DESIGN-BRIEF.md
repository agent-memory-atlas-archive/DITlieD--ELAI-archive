# ELAI Dashboard — Design Brief (single source of truth)

Target: Tauri 2 + Svelte 5 + Vite desktop app. Minimalist, modern, elegant, with a dark/sad elegiac undertone. New, innovative, fluid. Kimi-inspired sidebar (logo-focused) but with TWO workspace panels instead of a single chat area.

This brief is the contract every agent reads first. **Read DOMAIN-CONTEXT.md (sibling file) immediately after this** — it lists every Tauri command, window, drawer, status-bar field and mock-data shape your stubs must produce. Treat the two files as one document. Do not deviate without cause. No emojis anywhere.

---

## 1. Design philosophy

Feel: a quiet observatory at 3am — deep, silent, precise. The UI should feel like it's listening.

- Minimalism first: negative space is a feature. If a pixel is not doing work, remove it.
- Dark-first, not dark-mode. Near-black backgrounds (#0A0D11 range, zinc-green tinted, never pure #000).
- One accent dominates (muted teal). A cold steel-blue is the only secondary accent, used for informational links and keyboard hints. Purple is FORBIDDEN — the spec v2 specified purple; this version replaces it with the teal/steel duo.
- Motion communicates state. Never decorates. 120–240 ms, ease-out for enters, ease-in for exits.
- "Sad" does not mean grim. It means restrained, slightly desaturated, low-lumen, melancholic hush. Think: Arc Browser at night, Linear at 2% luminance, Superhuman after hours.
- No chrome bloat: no drop shadows, no bevels, no gradients on buttons. Depth comes from background steps and a single-pixel border.
- Progressive disclosure: details on demand.

Anti-patterns banned: rainbow status, bouncy animations, spinners, glassmorphism blobs, neon, emojis, gradient buttons, corporate Notion blue.

## 2. Color system (CSS tokens)

All colors live in `src/styles/tokens.css` as custom properties. No hex literals in components — always `var(--…)`.

```
/* Backgrounds — deep to lifted */
--bg-void:      #07090B;   /* window chrome, out-of-panel voids */
--bg-base:      #0B0E12;   /* primary surface (default panel bg) */
--bg-surface:   #10141A;   /* elevated cards, tab bar */
--bg-hover:     #161B22;   /* hover, selected row */
--bg-input:     #1A2029;   /* input fields, code */
--bg-overlay:   rgba(7,9,11,0.72); /* modal backdrop, NO blur over 4px */

/* Primary accent — muted teal-forest */
--teal:         #1E8A7A;   /* active, focus ring, brand */
--teal-bright:  #2BAE98;   /* progress, important badges (use sparingly) */
--teal-soft:    rgba(30,138,122,0.28);  /* glows, subtle highlights */
--teal-ghost:   rgba(30,138,122,0.10);  /* hover tint, active bg */
--teal-line:    rgba(30,138,122,0.45);  /* 1-2px accent borders */

/* Secondary — cold steel blue (links, info, keyboard hints) */
--steel:        #5A7A9E;
--steel-soft:   rgba(90,122,158,0.25);
--steel-ghost:  rgba(90,122,158,0.08);

/* Text — warm-neutral, never pure white */
--text-primary:   #D7DCE3;
--text-secondary: #7A828E;
--text-ghost:     #3F454E;
--text-disabled:  #2A2F37;

/* Semantic (used only where actual state requires) */
--pass: #1E8A7A;       /* reuses teal */
--warn: #C39A3A;       /* muted amber */
--fail: #B04545;       /* muted red */
--info: #5A7A9E;       /* reuses steel */

/* Border */
--border-subtle: rgba(255,255,255,0.04);
--border-default: rgba(255,255,255,0.07);
--border-strong:  rgba(255,255,255,0.12);

/* Radius */
--r-xs: 2px;
--r-sm: 4px;
--r-md: 6px;
--r-lg: 10px;
--r-pill: 999px;

/* Spacing (8px base) */
--s-1: 4px;  --s-2: 8px;  --s-3: 12px; --s-4: 16px;
--s-5: 20px; --s-6: 24px; --s-8: 32px; --s-10: 40px;
--s-12: 48px; --s-16: 64px;

/* Motion */
--dur-fast: 120ms;
--dur-base: 180ms;
--dur-slow: 320ms;
--ease-out: cubic-bezier(0.16, 1, 0.3, 1);    /* expo-out, feels premium */
--ease-in:  cubic-bezier(0.7, 0, 0.84, 0);
--ease-std: cubic-bezier(0.4, 0, 0.2, 1);

/* Elevation (never shadows — background steps only) */
/* There is no --shadow-*. Depth = color step. Use bg-surface over bg-base. */

/* Type */
--font-sans:  'Inter', 'Segoe UI Variable', system-ui, sans-serif;
--font-mono:  'JetBrains Mono', 'Fira Code', ui-monospace, monospace;
--font-display: 'Inter', 'Segoe UI Variable', system-ui, sans-serif; /* thin weight 200 for display */

/* Focus ring */
--ring: 0 0 0 1px var(--teal), 0 0 0 3px var(--teal-soft);
```

## 3. Typography

- Sans body: Inter (preferred). Weights used: 400 body, 500 UI strong, 600 buttons, 200 display (for large ELAI wordmark).
- Mono: JetBrains Mono for code, input, timestamps, paths, FSM state, step IDs.
- Scale: 10 / 11 / 12 / 13 / 14 / 16 / 20 / 28 / 40.
- Line height: 1.5 body, 1.3 UI, 1.2 display.
- Letter-spacing on display xl/2xl: 0.02em tightening; on UPPERCASE labels (e.g. LOCAL / EXECUTING): 0.08em.
- Do NOT use bold aggressively. Prefer weight 500 over 700.

## 4. Layout

### 4.1 Shell
```
+----+-------------------------+-------------------------+
|    |                         |                         |
| S  |   PRIMARY WINDOW        |   SECONDARY WINDOW      |
| I  |                         |                         |
| D  |                         |                         |
| E  |                         |                         |
| B  |                         |                         |
| A  |                         |                         |
| R  |                         |                         |
|    +-------------------------+-------------------------+
|    |   STATUS BAR (24px)                               |
+----+---------------------------------------------------+
```

### 4.2 Sidebar (Kimi-inspired, logo-forward)
- Width 64 px, never expands.
- Top: ELAI logo — the existing E L A I wordmark in teal (thin weight). ~28 px tall. Always visible. Subtle hover glow.
- Beneath logo: a thin 1-px divider line (`--border-subtle`).
- Icons vertical column (20 px Lucide, thin stroke, monochrome `--text-secondary`, becomes `--teal` when active).
  - Order: Terminal/Command, Layers/Plan, Play/Activity, GitBranch/Review, Search/Explore, Activity/Monitor, Settings (at bottom-ish).
- Bottom of sidebar: mode dot (LOCAL = teal, CLOUD = steel) + session/user icon.
- NO labels next to icons. Tooltip shows on 400 ms hover delay: label + shortcut.
- Icons: ~44 px tall hit target, 20 px icon centered. Active icon has a 2-px teal left-edge inset bar (inside sidebar) + `--teal-ghost` background.

### 4.3 Dual workspace panels
- PRIMARY and SECONDARY panels, each with its own tab bar.
- Draggable divider between them: 2 px wide, transparent, shows `--teal-line` on hover/drag. Min panel width 320 px. Double-click divider to reset to 50/50.
- Each panel can hold any window type (COMMAND, PLAN, ACTIVITY, REVIEW, EXPLORE, MONITOR, SETTINGS).
- Tab bar: 32 px tall, sits flush top of each panel. Tabs are ghost by default, 1-px teal bottom border when active. [+] at end. Tabs can be dragged between panels.
- The whole workspace sits inside `--bg-base`. Panels are visually delineated only by a 1-px `--border-subtle` edge and the tab-bar being slightly lifted (`--bg-surface`).

### 4.4 Status bar
- 24 px. Mono font 11 px.
- Left: `project-name · branch · FSM-state`.
- Right: `model · VRAM% (if >70) · session-id`.
- When EXECUTING: a single 1-px teal breath-line at the very bottom of the status bar that pulses opacity 0.2 -> 0.55 -> 0.2 over 2.2 s. That is the only loading indicator in the entire app.

### 4.5 Startup screen (project selector)
- Full-viewport, centered content.
- Huge ELAI wordmark (thin-200, 56 px, teal) centered upper-third. Below, in `--text-secondary` 12 px: `a deterministic development engine`.
- A single pulsing teal dot below the tagline (breath animation) — signals "idle, listening".
- Actions block: `+ New Project` (text-only teal button, underline on hover).
- Divider (1 px `--border-subtle`, max-width 480 px).
- Recent project list (max 8 rows). Each row: name (Inter 14 primary) · path (mono 11 ghost) · elapsed (mono 11 ghost) right-aligned. Hover fills with `--teal-ghost` and grows a 2-px teal left inset.
- Bottom: three subtle toggle texts — `LOCAL  CLOUD  HYBRID`. Active has underline + teal text.
- Enter animation: wordmark fades in 400 ms, tagline 200 ms after, dot 300 ms after, list items stagger-fade 40 ms each.

## 5. Window types (each is a Svelte component in `src/lib/windows/`)

Each window receives a `windowId: string` prop and reads/writes its own state through the `workspace` store. All data must be STUB data with realistic shape. Every action handler must be defined and log to console — no `// TODO`, no empty handlers.

- **CommandWindow** — not a chatbot. Structured cards: Status, Intervention, Error, Disambiguation, Steering. Input bar at bottom with `/` slash-palette.
- **PlanWindow** — hierarchical phase/step tree. Active phase expanded, others collapsed. Click step -> emits `open-step` event.
- **ActivityWindow** — chronological stream of `[reasoning]`, `[edit]`, `[validate]`, `[test]`, `[cloud]` entries under step headers. Current step has teal breath-pulse underline.
- **ReviewWindow** — unified diff with line numbers, sticky file header, per-hunk + per-file accept/decline, retry.
- **ExploreWindow** — Q&A; each query is independent, no chat history. File refs clickable.
- **MonitorWindow** — autonomous dashboard: identity, progress bar, phase dots, status snapshot, interventions inbox, timeline.
- **SettingsWindow** — grouped sections (Execution, Control, Project, Appearance). Flat list of rows, no cards.

See the OLD spec in `docs/specs/guide/dashboard-design.txt` for content semantics — adopt the content model verbatim; REJECT the purple accent and REJECT any heavy visual treatment from the old spec.

## 6. Motion library (shared)

All in `src/lib/motion.ts` + keyframes in `src/styles/motion.css`.

- `enter-fade` — opacity 0 -> 1, 180 ms ease-out
- `enter-lift` — opacity 0 + translateY(4px) -> 1/0, 220 ms ease-out
- `stagger(40ms)` utility for lists
- `breath` — infinite pulse for live indicators: opacity 0.2 -> 0.55 -> 0.2 over 2.2s
- `scan` — a 1-px teal line that sweeps across the panel bottom when a phase completes (240 ms, one-shot)
- Hover transitions: all 120 ms ease-std on background/color
- Active-row left-bar: `transform: scaleY` from 0 -> 1 over 140 ms ease-out (origin top)
- Tab switch: content fades 100 ms out, 140 ms in, no slide
- Focus ring appears instantly but expands 1 px with 120 ms ease-out
- NEVER: bounce, spring overshoot, scale>1.03, opacity blinking, rotation

## 7. Component inventory (all in `src/lib/ui/`)

Primitives:
- `Button.svelte` — variants: `primary` (teal fill, low-sat), `ghost` (no bg, teal text), `text` (underline-on-hover), `danger` (muted red ghost)
- `IconButton.svelte` — square 32 px, 20 px icon
- `Input.svelte` — dark, 1-px border, teal on focus, supports slot for left/right adornments
- `Select.svelte`, `Toggle.svelte`, `Tooltip.svelte`, `Divider.svelte`
- `Card.svelte` — variants: `status` (teal left bar), `intervention` (teal-bright left bar), `error` (fail left bar), `info` (steel left bar)
- `Tabs.svelte` / `Tab.svelte`
- `Tree.svelte` — for PlanWindow
- `Pulse.svelte` — the breath indicator
- `Scrollbar.svelte` — custom scrollbar wrapper (4 px, invisible until hover)
- `Kbd.svelte` — keyboard hint pill (11 px mono, steel ghost bg)

Layout:
- `Sidebar.svelte`, `StatusBar.svelte`, `Panel.svelte`, `PanelDivider.svelte`, `TabBar.svelte`, `AppShell.svelte`

Screens:
- `ProjectSelector.svelte`, `Workspace.svelte`, `WindowHost.svelte` (renders window by type)

## 8. State (Svelte 5 runes)

All in `src/lib/state/`.

- `workspace.svelte.ts` — layout state (panels, tabs, active window per panel), window definitions
- `fsm.svelte.ts` — stub FSM state + current step + progress
- `activity.svelte.ts` — stream entries (stubbed with realistic data)
- `plan.svelte.ts` — phase tree
- `session.svelte.ts` — project, mode (LOCAL/CLOUD/HYBRID, INTERACTIVE/AUTONOMOUS), model, session-id
- `commands.svelte.ts` — command history + input intent classifier (stub)
- `shortcuts.ts` — keyboard map

Each store exports rune-based state + pure action functions. Stores are singletons.

## 9. Keyboard

- Ctrl/Cmd+1…7 — switch sidebar view (Command/Plan/Activity/Review/Explore/Monitor/Settings)
- Ctrl+N — new tab in active panel (opens window-type picker overlay)
- Ctrl+W — close active tab
- Ctrl+Shift+H — recover closed window
- Ctrl+D — toggle secondary panel
- Ctrl+. — slash palette
- Esc — dismiss overlay
- Ctrl+B — toggle sidebar labels off (N/A, sidebar stays 64 px — this is reserved for future)

## 10. Tech stack (locked)

- **Tauri 2.x** latest stable
- **Svelte 5** (runes), **Vite** 5+
- **TypeScript**, strict mode
- **No Tailwind**, no CSS-in-JS, no Shadcn. Plain CSS + tokens. Scoped Svelte styles.
- Icons: **lucide-svelte** (thin stroke)
- Fonts: **@fontsource-variable/inter**, **@fontsource-variable/jetbrains-mono**
- Package manager: **npm**

## 11. Stubbing rules

- Any function that would call Tauri invoke, or hit a backend, gets a stub in `src/lib/api/stub.ts` that returns realistic data after a small random delay (80–240 ms).
- All stubs emit console.info with `[stub] <fn-name>` so the user can see wiring points.
- No `// TODO`, no empty handlers, no `pass` equivalents. Every button clicks.

## 12. Acceptance criteria (applies to every wave)

- `npm run dev` launches Vite dev server without errors on http://localhost:1420.
- `cargo check` inside `src-tauri/` passes.
- Every interactive element has a hover and focus state.
- No horizontal scrollbars at 1280 x 800 viewport.
- No console errors on initial mount.
- At least one motion plays on first render (wordmark fade-in or stagger).
- 0 TypeScript errors in strict mode.

## 13. File tree (target)

```
.elai_dashboard/
  package.json
  vite.config.ts
  svelte.config.js
  tsconfig.json
  index.html
  src-tauri/
    Cargo.toml
    tauri.conf.json
    build.rs
    src/main.rs
    src/lib.rs
    capabilities/default.json
    icons/  (placeholder, we'll fix later)
  src/
    main.ts
    app.d.ts
    App.svelte
    styles/
      tokens.css
      base.css
      motion.css
      scrollbar.css
    lib/
      state/*.svelte.ts
      api/stub.ts
      motion.ts
      utils/*
      ui/*.svelte
      windows/*.svelte
      drawers/*.svelte
      layout/*.svelte
      screens/*.svelte
    assets/
      fonts/  (optional if using fontsource only)
  design-refs/
    REFERENCES.md
  DESIGN-BRIEF.md   (this file)
  README.md
```

---

## 14. Agent contract (read before you touch anything)

1. You MUST read this brief end-to-end before writing any file.
2. You MUST write only inside `/workspace/ELAI\.elai_dashboard\` (sub-paths scoped to your wave).
3. You MUST NOT install new deps not on the locked stack list (11). If tempted, stop and log it in a `DECISIONS.md` entry instead.
4. You MUST NOT use hex color literals in Svelte files. Only `var(--…)`.
5. You MUST NOT use purple.
6. You MUST NOT use emojis anywhere.
7. You MUST keep TypeScript strict-mode clean.
8. You MUST leave a one-line handoff note at the end of your summary: `Next wave needs: X`.
9. Zero lazy code. Every function that the user will hit is fully implemented with mock data.

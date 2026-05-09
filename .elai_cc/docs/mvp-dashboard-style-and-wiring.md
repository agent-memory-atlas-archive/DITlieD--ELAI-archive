# MVP Dashboard — Style Rules and Wiring Guide

Reference for future sessions. Two parts:

- **Part 1**: visual / UX rules the user ratified for the dashboard. Treat as
  canon — do not redesign without explicit instruction.
- **Part 2**: where to put new code when wiring a backend service into the
  dashboard's task_runner pipeline, or adding a new UI surface.

Last revised: 2026-04-18.

---

## Part 1 — Visual / UX rules (user-ratified)

### 1.1 Landing screen (`ProjectSelector.svelte`)

- Centered layout, content pulled up to `padding-top: 14vh`. Do NOT use
  `justify-content: center` — the content sits too low on 1080p/QHD and looks
  mid-page.
- ELAI wordmark: `--font-display`, weight 200, size `clamp(64px, 8vw, 112px)`,
  letter-spacing `0.18em`, color `--teal`.
- Letter-spacing leaves an invisible trailing stop after the last glyph. Cancel
  it with `margin-right: -0.18em` on the logo so the wordmark optically centers
  against anything beneath it. Without this the whole hero reads as left-shifted.
- Tagline: "a deterministic development engine" — `--text-secondary`,
  font-weight 300, size `--fs-base`.
- Single primary action: `+ new project` as borderless teal text button.
  Hover → `--teal-bright`. No rectangular button, no outline.
- Recent projects list: left border 2 px transparent, turns `--teal` on hover.
  Name in `--text-primary`, path in `--text-secondary` + `.mono` class, time in
  `--text-ghost`.

### 1.2 No global chrome

The "ELAI • TemplatePicker" top header is gone. The "IDLE · —" status bar is
gone from the landing and new-project flows. `StatusBar.svelte` now lives
**inside** the Workspace view only. Do not add it back to the app shell.

### 1.3 New-project flow (`TemplatePicker.svelte`)

- Same centered layout. Smaller logo (52–80 px).
- Single "select project folder" button opens the native OS folder picker via
  `tauri-plugin-dialog` (`open({ directory: true })`).
- Once a folder is picked, the `scan_project_folder` Tauri command runs
  automatically:
  - Detects emptiness (ignores `.git`, `.DS_Store`, `Thumbs.db`, `desktop.ini`).
  - If `README.md` exists, extracts the first H1 (`# Title`) as the suggested
    name. Otherwise uses the folder basename.
  - Slugifies `project_id` from the suggested name.
- UI after scan: path (mono), meta line:
  `empty folder` | `N items · name from README.md` | `N items`.
- Editable fields: name, id. **Bare inputs** (transparent + bottom-border only,
  focus turns teal) — no rectangular boxes.
- Template dropdown appears ONLY for empty folders, and leads with
  `— none (empty project) —`. Picking none → mkdir only. Picking a template →
  the Rust side backfills the 7 required scaffold vars (`project_name`,
  `description`, `author`, `*_hsl`) from defaults. The frontend sends `{}`.

### 1.4 Workspace (`Workspace.svelte`) — FLOW 2

Spec: `docs/specs/guide/dashboard-design.txt` lines ~162–237.

- **48 px vertical sidebar** on the left. Five inline-SVG icons: terminal,
  layers, play, search, settings. Stroke 1.5, 20 px glyphs. Active icon =
  teal + `--teal-ghost` background. Inactive = `--text-secondary`. Hover =
  `--text-primary`. Never expand into a drawer.
- **Primary panel** takes remaining width.
- **Secondary panel**: 380 px default, collapsible via edge handle (MVP
  substitute for a draggable divider — flagged `TODO(post-mvp): draggable`).
- Each panel has a one-row tab bar; single tab for MVP, `[+]` placeholder.
- Sidebar-icon → panel mapping:
  - `terminal` → `COMMAND` primary + `ACTIVITY` secondary (default)
  - `layers` → `PLAN` + `ACTIVITY`
  - `play` → `ACTIVITY` + `COMMAND`
  - `search` → `EXPLORE` placeholder + `ACTIVITY`
  - `settings` → `SETTINGS` placeholder + `ACTIVITY`
- **COMMAND window** (not a chatbot — structured control surface):
  - Scrollable message history with left-border message cards:
    - purple border = user input
    - teal border = elai status line
  - Bottom input bar — FSM-aware placeholder:
    - IDLE → `"describe a task, ask a question, or type / for commands"`
    - EXECUTING → `"add constraints, or wait for intervention cards"`
    - intervention pending → `"respond to the card above, or type to add context"`
  - Enter submits (calls `submit_task`). Shift+Enter inserts newline.
  - `[Send]` button appears only when text has content.
  - Intervention cards render inline when `pending.length > 0`.

### 1.5 Status bar (`StatusBar.svelte`)

- **Inside Workspace only.** 24 px tall, `--bg-base`, top border `--bg-hover`.
- Left: `project · branch · FSM state`.
- Right: `model · session` (both `.mono`).
- During EXECUTING, a 2 px teal linear-gradient pulse sweeps the top edge every
  2.4 s. That is the only persistent motion in the entire app.

### 1.6 Font stack (`app.css`)

Three stacks, three purposes. Do not cross them.

- `--font-display`: `"Segoe UI Variable Display"`, `"Segoe UI"`, `Inter`,
  `system-ui`, `-apple-system`, sans-serif.
  Use ONLY for the ELAI wordmark and large headers. Thin weight (200).
- `--font-ui`: `Inter`, `"Inter Variable"`, `system-ui`, `-apple-system`,
  `"Segoe UI Variable Text"`, `"Segoe UI"`, `"Helvetica Neue"`, `Arial`,
  sans-serif.
  Default for all body text, buttons, labels, tabs, menus. Modern, readable,
  fixed-width-free.
- `--font-mono`: `"JetBrains Mono"`, `"Fira Code"`, `"Cascadia Mono"`,
  `Consolas`, `ui-monospace`, monospace.
  Opt-in via the `.mono` class. Use for file paths, diffs, code blocks,
  hashes, session ids — anything that must line up character-by-character.
  Do NOT set it as body default — the UI starts feeling like a terminal dump.

Sizes (`app.css`): `--fs-xs: 11px`, `--fs-sm: 13px`, `--fs-base: 14px`,
`--fs-lg: 16px`, `--fs-xl: 20px`. Rendering hints: antialiased, optimizeLegibility.

### 1.7 Color tokens (`app.css` — do not invent new colors)

- Backgrounds (darkest → lightest): `--bg-void` `#0a0d10`, `--bg-base` `#0f1318`,
  `--bg-surface` `#151a20`, `--bg-hover` `#1a2028`, `--bg-input` `#1e2530`.
  All near-black grayscale. No green or blue tint on the page background.
- Primary accent — teal: `--teal` `#1b7a6e`, `--teal-bright` `#24a08e`,
  `--teal-soft` (25 % alpha), `--teal-ghost` (8 % alpha). Used for: logo,
  primary actions, success, active states, user-task status cards.
- Secondary accent — purple: `--purple` `#7c5cbf`, `--purple-soft` (19 % alpha).
  Used for: user-interactive markers (user chat bubbles, intervention cards,
  "reasoning" pills).
- Text: `--text-primary` `#d0d7de`, `--text-secondary` `#6e7681`,
  `--text-ghost` `#383e47`.
- Semantic: `--pass` (= teal), `--warn` `#c49b2a` (muted amber),
  `--fail` `#c44040` (muted red), `--info` `#5088c0` (steel blue).

Severity colors are for real status only. Do not decorate.

### 1.8 Button style (`app.css` default)

- Transparent background, no border, no outline, no shadow.
- Hover (not disabled): `background: var(--bg-hover)`. No scale, no translate.
- Disabled: `color: var(--text-ghost)`, `cursor: not-allowed`.
- Do not reintroduce teal-soft borders or rounded filled primary buttons.
  If a button needs more weight, give it color (`--teal`) and the `:hover`
  shifts to `--teal-bright` — that is the full style palette.

### 1.9 Inputs

- Rectangular inputs (`input[type=text]`, `select`, `textarea`) exist at the
  app-css level for utility but ProjectSelector and TemplatePicker prefer
  `.bare` inputs: transparent + `border-bottom: 1px solid var(--bg-hover)`,
  focus turns the border teal. Keep this pattern for any new setup screens.

### 1.10 Motion

- Transitions: 120 ms ease on `background-color` / `color` / `border-color`.
  No transforms, no scaling, no bouncing.
- Only persistent motion: the StatusBar EXECUTING pulse.
- No spinners. Never. Use minimal pulses or state transitions.

### 1.11 What not to do

- No rainbow status. One teal accent + one purple accent + grayscale. That is
  the entire palette.
- No big forms on startup. Pick folder → scan → prefill → confirm.
- No "loading…" spinners or centered dialog overlays on initial render.
- No emojis anywhere. No decorative icons. Sidebar icons are functional SVGs.
- No rectangular bordered cards for non-intervention content.
- No green/blue tinted backgrounds — grayscale only.
- No monospace as body text.
- No "ELAI • …" breadcrumb header at the top of the window.
- No FSM state bar on the landing or new-project screens.

---

## Part 2 — Wiring architecture

### 2.1 File map

Ranked by how often you will touch it when closing a gap.

| Path | Role |
|---|---|
| `crates/dashboard/src/main.rs` | Tauri binary bootstrap. 3 lines. Calls `elai_dashboard_lib::run()`. Never edit. |
| `crates/dashboard/src/lib.rs` | **Wiring assembly.** `pub fn run()` constructs every service, fills `TaskRunnerConfig`, starts `start_task_runner`, then hands off to Tauri. This is where new services get instantiated and plumbed. |
| `crates/orchestrator/src/task_runner.rs` | **Service contract.** `pub struct TaskRunnerConfig` — add a field here before wiring in `lib.rs`. `pub async fn drive_task` — the runtime pipeline; invoke your service from here or a helper it calls. |
| `crates/dashboard/src/commands.rs` | Tauri `#[tauri::command]` handlers. Thin IPC layer. Each handler reads `DashboardState` and writes an SSB payload or returns structured data. No pipeline logic lives here. |
| `crates/dashboard/capabilities/default.json` | Tauri v2 permission manifest. Grant new plugins (`dialog:allow-open`, etc.) here or invokes fail silently. |
| `crates/dashboard/src-frontend/src/*.svelte` | Svelte 5 UI. Runes + callback-prop idioms only (`$state`, `$derived`, `$props`, `onclick={fn}`). No `on:click`, no `export let`, no `createEventDispatcher`. |
| `crates/dashboard/src-frontend/src/app.css` | Design tokens. Change tokens once, read them everywhere. |
| `crates/dashboard/src-frontend/src/ActivityEntry.svelte` | Per-row renderer for the activity feed. Extend `colour()` and `typeLabel()` when the backend emits new SSB action types. |

### 2.2 Add-a-service cookbook (three-line pattern)

Every gap closed in the 2026-04-18 audit campaign followed this pattern:

**Step 1** — declare the field on `TaskRunnerConfig` in `task_runner.rs`:

```rust
pub struct TaskRunnerConfig {
    ...
    pub my_new_service: Option<Arc<MyNewService>>,
}
```

**Step 2** — thread through `ResolvedConfig` and consume in `drive_task`:

```rust
// same file, resolve():
ResolvedConfig { ..., my_new_service: self.my_new_service }

// drive_task():
if let Some(svc) = resolved.my_new_service.as_ref() {
    // invoke at the right hook point (pre-task, pre-baby-step, post-baby-step, on-fail)
}
```

**Step 3** — construct in `lib.rs::run()` and pass in:

```rust
let svc = MyNewService::new(...);
let (_join, cancel) = start_task_runner(
    writer_arc.clone(),
    TaskRunnerConfig {
        ...,
        my_new_service: Some(Arc::new(svc)),
    },
);
```

**Step 4** — add a contract test under `crates/contracts/tests/` asserting (a)
the field exists and accepts an Arc, (b) `drive_task` invokes it under the
expected conditions, (c) absence falls back gracefully.

Package name is `elai_cc_contracts`. Run with
`cargo test -p elai_cc_contracts --test <test_name>`.

### 2.3 Runtime pipeline (what `drive_task` does)

```
submit_task (Tauri command)
  → commands::submit_task writes SsbPayload::TaskSubmitted
  → task_runner picks it up from SSB
  → drive_task(payload):
      1. maybe_build_project_index      [first task per project_id — gap #6]
      2. build_dispatch_context         [if llm_provider Some — gap #3]
      3. branch: scaffold / research / default decompose
      4. per concept → phase → step → baby_step:
         a. evaluate_at_pre_gate        [rules engine, pre-step]
         b. baby_step_executor.execute  [real agent IPC spawn — gap #1]
         c. emit_checkpoint_outcome
         d. run_post_step_deterministic_audit   [gap #4]
         e. evaluate_post_execution_rules_and_emit_findings  [gap #9]
         f. record_mistake / emit_interrogation_fix
         g. emit_baby_step_checkpoint / recovery
         h. AntiSpiralDetector          [if enabled — gap #5]
  → SSB rows accumulate
  → frontend polls commands::get_recent_actions every 500 ms
  → ActivityFeed re-renders
```

Hook points you can attach a new service to:
- **per-task setup**: top of `drive_task`, before branching.
- **pre-baby-step**: inside the baby-step loop, before `execute`.
- **post-baby-step success**: after `emit_checkpoint_outcome` when outcome =
  Passed and changed_files is non-empty.
- **post-baby-step failure**: inside `record_mistake_on_failure` or
  `emit_interrogation_fix_suggestion`.
- **per-phase boundary**: inside `emit_level_transitions`.

### 2.4 Add a Tauri command (frontend ↔ backend)

1. `#[tauri::command] pub fn foo(...)` in `crates/dashboard/src/commands.rs`.
2. Register in `generate_handler![...]` inside `lib.rs::run()`.
3. If it needs a Tauri plugin permission (dialog, fs, etc.), add to
   `crates/dashboard/capabilities/default.json`. `core:default` and
   `dialog:allow-open` are already granted.
4. Call from any Svelte file:
   `await invoke<ReturnType>("foo", { argOne, argTwo })`.

### 2.5 Add a new SSB action-type color in the activity feed

Frontend-only. Edit
`crates/dashboard/src-frontend/src/ActivityEntry.svelte` `colour()` switch to
map the action type to a semantic class: `teal`, `purple`, `warn`, `fail`,
`info`, `pass`, or `dim`. For severity-sensitive variants (like `finding`),
also extend `findingLevel()` (BLOCK/WARN/INFO substring scan of `summary`) and
`findingCategory()` (constant names from `task_runner.rs`).

Do not add new Tauri commands for this. `SsbActionSummary` already carries the
bytes; you're just classifying them in the renderer.

---

## Part 3 — Reference files (read-only for implementation)

**Design source of truth**:
- `docs/specs/guide/dashboard-design.txt`
  - FLOW 1 = Project selection screen, ~line 106
  - FLOW 2 = Workspace layout, ~line 162
  - FLOW 3 = Windows / tabs, ~line 239
  - Window specs: COMMAND, PLAN, ACTIVITY, REVIEW, MONITOR, EXPLORE, SETTINGS

**Phase and A-step catalogue** (summaries, read these per session — the
individual `phase-N.txt` files are too large):
- `docs/specs/guide/steps/phase-index-pre0-2D.txt` — pre-0 through 2D
- `docs/specs/guide/steps/phase-index-3A-4J.txt` — 3A through 4J (incl. 3J / 3K)
- `docs/specs/guide/update-implementation-order-summary.txt` — A-step catalogue

**MVP state**:
- `docs/mvp-wiring-gaps.md` — the user's rolling gap log
- `.elai_cc/docs/mvp-wiring-audit-2026-04-18.md` — adversarial wiring audit
  (48 unwired items enumerated, top 10 fix priorities)

**Project intelligence**:
- `docs/specs/guide/project-intelligence-design.txt` — Tier 1 / 2 / 3 design

---

## Part 4 — TODO(post-mvp) markers (grep for them)

Frontend (`crates/dashboard/src-frontend/src/`):
- `Workspace.svelte` — tab management, draggable divider, slash command
  palette, intent classification, disambiguation cards, MONITOR window,
  EXPLORE functionality, SETTINGS content, window history recovery.
- `StatusBar.svelte` — real branch (git HEAD), real model (sidecar config),
  real session id, VRAM indicator when > 70 %.
- `ActivityEntry.svelte` — expose `passed: bool` on `SsbActionSummary` so the
  `test_outcome` row renders pass vs fail deterministically.

Backend — none at this writing. All wiring campaign code uses existing types
from A-step / phase modules. If you see a new `TODO(post-mvp)` marker appear
in a Rust file, treat it as a scope-violation and investigate.

---

## Part 5 — Rules of engagement for future sessions

1. **Do not redesign** Part 1 without explicit user instruction. The style
   decisions above are ratified — the user has already iterated on them.
2. **Wire existing A-step / phase code**; do not build parallel implementations.
   If you need a capability, grep the codebase for the relevant A-step file
   first. See `.elai_cc/docs/mvp-wiring-audit-2026-04-18.md` section 2/3 for
   the canonical wired / built-but-unwired inventory.
3. **Svelte 5 only** in the frontend. Runes + callback props + `onclick`. No
   Svelte 4 idioms.
4. **No new dependencies** without explicit user confirmation (CLAUDE.md
   hard rule).
5. **Contract test every new wiring**. Tests go in
   `crates/contracts/tests/<topic>_contract.rs`. Run with
   `cargo test -p elai_cc_contracts --test <name>`.
6. **Release rebuild** with `cargo build --release -p elai-dashboard -p elai_cc_agent`
   after wiring. The `beforeBuildCommand` in `tauri.conf.json` re-runs
   `vite build` so the frontend bundle stays in sync.
7. **`elai.bat` launches `target/release/elai.exe --dashboard`** which in
   turn spawns `target/release/elai-dashboard.exe`. A new `elai-dashboard`
   binary is not picked up until the old ELAI window is closed (Windows
   file lock). Tell the user to close-and-relaunch after release builds.

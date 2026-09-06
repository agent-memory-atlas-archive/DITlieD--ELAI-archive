ELAI DESIGN (NO-SLOP).md — the de-slopped ELAI design language

status: v1, 2026-06-20. This is ELAI re-derived through TEIKOKU's anti-slop discipline: TEIKOKU SIGNAL was ELAI with every slop tell removed; this file puts that same discipline back under ELAI's OWN identity (the brighter teal #2BBBA4 ramp, the monoline stroke wordmark, the light-theme + accent-theming features). It is NOT the editorial workflow/design/DESIGN.md (that one still carries the slop this file removes — the cosmic intro, the always-on glow, the center-out underlines, mono-as-default, the circle/check dots). Sibling: workflow/design/DESIGN-TEIKOKU.md (same structure, TEIKOKU colors). Where the two differ it is ONLY: accent hex (ELAI #2BBBA4 vs TEIKOKU #14a78f), the brand mark (ELAI stroke wordmark vs TEIKOKU text "TEIKOKU"), and the two ELAI-only features (§17 light theme, §18 user accent). Everything structural is shared on purpose so the two never drift apart.

WHAT THIS FILE CHANGED FROM THE OLD ELAI DESIGN.md (the de-slop, explicit so it is not re-litigated):
- killed the cosmic intro (starfield, parallax, orbital rings, radial bloom, staggered stroke-by-stroke logo draw, timed reveal) — the app opens IMMEDIATELY (B-05).
- killed the always-on teal bloom / glow / soft logo glow — elevation is a lightness step + a 1px line, never glow (B-02).
- killed the center-out / grow-from-center underline on rows, nav, the mode selector, START, and the new-project button — selection/hover is a SOLID change (fill or a 2px square accent edge), never a drawn line (B-01).
- demoted mono from the default UI/body/label face to a DATA-ONLY face; a grotesque carries the human text (B-03).
- killed the ALL-CAPS .2em-tracked dot-prefix micro-labels as the system; labels are normal-case grotesque (B-04).
- replaced the circle/check status glyphs (✓ ● ○) with SQUARE marks + weight + the mono-inversion chip (B-10).
- tightened radius to 2px (B-08).
KEPT from ELAI (these were never slop): the teal #2BBBA4 recognition color, the monoline stroke wordmark as the brand mark (now shown statically), the mono-inversion FAIL chip, the full dark/light toggle (§17), the user-accent theming feature (§18).

0 THE ONE LINE
A near-black instrument with one bright-teal signal: maximum contrast, square marks, a grotesque for everything human and mono only for data, radical negative space. Dense where work lives, empty where it does not, still and matte everywhere. ELAI's teal identity with zero AI-slop tells.

1 THE SLOP BANS (a surviving tell is a BLOCK at design audit)
- B-01 NO center-out / grow-from-center underline on rows, buttons, tabs, nav, the mode selector, or START. Hover/selected = a SOLID change (fill, block, or a 2px square accent edge), never a drawn line.
- B-02 NO glow, bloom, halo, neon, drop-shadow (the only box-shadow allowed is `none`). NO teal bloom on any screen, brand included. Elevation = a real lightness step + a 1px line.
- B-03 NO monospace as the default UI/body/label face. JetBrains Mono is for NUMERIC DATA, code, tokens, timestamps, status codes ONLY.
- B-04 NO ALL-CAPS .2em-tracked micro-labels as the system, and NO dot-prefix / number-prefix labels. Labels are normal-case grotesque medium.
- B-05 NO cosmic intro: no starfield, parallax stars, orbital rings, radial bloom, staggered logo-draw, or timed reveal. The app opens immediately.
- B-06 NO blue, purple, indigo, red, orange, amber, or a second accent hue, EVER. ONE accent = ELAI teal --teal. The light mono-inversion chip is the only non-teal emphasis (fail), NEVER red.
- B-07 NO multi-hue status badge/pill spam. Status = shape (square) + weight + at most one teal mark + the inversion chip.
- B-08 NO grid-of-identical-cards as a layout; NO rounded-everything (radius is 2px, period); NO gradients (bg or text); NO glassmorphism/blur.
- B-09 NO Inter, Roboto, Arial, system-ui, Geist, Space Grotesk for the UI face. NO generic Lucide/Feather/Material icon set (CSS shapes + drawn unicode glyphs).
- B-10 NO tiny circle + checkmark status dots. Status marks are SQUARES, bars, weight, or inverted chips.
- B-11 NO low contrast: bright text on the near-black ground, always.

2 CONSISTENCY MANDATE
ONE canonical token file (the §3/§4 values as the ONLY color/space/type source; no literal hex or magic px in components — ELAI's existing role tokens --bg/--panel/--teal/… ARE that source, reuse them, do not invent parallel names). Build the atomic shared components ONCE (Button, SessionRow, Tile, Composer, Menu, Popover, TokenMeter, StatusMark, Toggle, WorkerSelect, IconRail, Panel/PanelManager, BrandCluster, TopRightControls, UsagePopup) and a COMPONENT INDEX every screen reuses. Every screen COMPOSES those; a value not in the token set, or a re-implemented component, is a bug caught at audit. The shared-chrome law (§16-chrome): any chrome element on more than one screen is EXACTLY ONE component with ONE set of coordinates — copying chrome markup into a second screen is itself the bug (it always drifts). visual-ui-loop.md applies for the screenshot-diff pass.

3 PALETTE (locked; ELAI's monochrome + ONE teal; emit oklch() at build from these hex anchors; role-named only, no literal hex in components)
  --bg          #0a0f0e   THE app background — every window, chrome bar, work surface; near-black, faint teal-green hint, NEVER blue-black
  --sink        #070b0a   deepest inset (meter tracks, wells), sits below --bg
  --panel       #0e1413   raised panel / tile / card lifted one step off --bg
  --raised      #151c1a   higher / hover / selected lift
  --row         #0c1211   list / table row ground
  --text        #f2f5f4   primary text, strong near-white (the contrast carrier)
  --read        #c9cecb   secondary reading text
  --dim         #79827e   captions / muted labels (teal-tinted gray, >=3:1)
  --ghost       #3a423f   disabled / skeleton strokes
  --teal        #2BBBA4   THE accent — interactive, active, selected keyline, links, PASS; used SOLID, never as glow
  --teal-bright #34C2AA   hover / hot stop
  --teal-deep   #1B7A6E   filled shapes (segments, done areas, progress fill)
  --line        rgba(255,255,255,.17)   hairline border / divider
  --line2       rgba(255,255,255,.17)   stronger divider / control border
Rules: ONE accent (ELAI teal). Every neutral tinted toward teal 2-4%, never pure gray, never pure #000/#FFF surfaces. Elevation runs --sink -> --bg -> --row -> --panel -> --raised as a lightness ladder + a 1px --line; no glow carries it. Fail = the LIGHT mono-inversion chip (--text background, --bg text) — the only loud, non-teal thing. Tokens role-named so §17 light flips the ramp without touching components. Contrast gate at build: >=4.5:1 body, >=3:1 large/UI, measured over --bg (ax-scaffold per visual-ui-loop.md).

4 TYPOGRAPHY (a grotesque for everything human, the hairline display for rare voice, mono ONLY for data)
  --font-ui      a grotesque (NOT Inter/Geist/mono; Hanken Grotesk acceptable) — all labels, body, buttons, nav, titles, chrome.
  --font-display the ELAI HAIRLINE display (weight 100-200, wide-tracked .3-.5em) — RARE display moments only (a hero line, at most one per screen). Kept as ELAI's voice; never body, never a label.
  --font-data    'JetBrains Mono' — numeric data, code, token counts, timestamps, status codes, the wordmark lockup ONLY.
Base 13.5px. Hierarchy (real jumps, normal case):
  title    grotesque 600-700, 15-18px
  body     grotesque 400-500, 13.5px, line-height ~1.5
  label    grotesque 500, 12-13px, NORMAL case (no caps-tracking, no dot)
  caption  grotesque 400, 10-11px, --dim
  display  hairline, large, wide-tracked, --text or --teal (rare)
  data     JetBrains Mono 10-13px, tabular-nums, --text (numbers/code/tokens only)
Build: bundle all faces as variable @font-face for production. A system-default-only stack on shipped UI is a BLOCK.

5 LAYOUT & IA (icon rail + collapsible panels + a movable/resizable pane workspace)
The only always-present chrome is the topbar + a thin left icon rail; everything else is toggled or rearranged.
- LEFT ICON RAIL (~48px, always present): a vertical stack of square icon buttons that TOGGLE panels and CREATE panes:
    [sessions] toggles the SESSION panel (project -> session list, left),
    [monitor]  toggles the MONITOR panel (autonomous-ops, right),
    [+]        creates a new pane (a chat pane by default),
    [usage]    opens the UsagePopup (§9),
    [connections] opens the connections surface (§11),
    [settings] opens settings.
  Active/open = a filled --teal square or a 2px --teal left edge. CSS/unicode glyphs only, --dim -> --text on hover; NO Lucide/Feather, NO glow.
- SESSION panel (collapsible, ~264px, rail-toggled): projects -> session rows. Project header = name + a small --dim tag (rust / unity · claude-only / free-api). Row state via §6 square marks; SELECTED = --raised fill + a 2px --teal LEFT EDGE (a square edge, never a drawn underline). Closes fully for full-width workspace.
- MONITOR panel (collapsible, ~300px right, rail-toggled): the AUTONOMOUS-OPS view — running executors/auditors (model + a ctx token meter), gate verdicts, the worker/model-pool summary, the last failure, AND the §11 CONNECTIONS block. Clicking the pool summary opens the UsagePopup.
- WORKSPACE (center): a MOVABLE, RESIZABLE pane area. DEFAULT = one chat pane + the MONITOR panel side-by-side. RESIZE by dragging the divider, MOVE/rearrange, DELETE, CREATE via the rail [+]. Multiple chat panes may coexist, each bound to a session. Panes dock/snap (no free-floating chaos), real rearrangement allowed. The review pane (§12) and the canvas live here as panes.
- Topbar: the §16-chrome shared cluster — the ELAI wordmark lockup LEFT, the back chevron, primary nav/mode CENTERED, the control cluster RIGHT. No centered underline-nav.
- Borders 1px solid --line/--line2; radius EXACTLY 2px; matte surfaces; 8px spacing rhythm; one clear focal element per pane. Pane/panel resizers are 1px --line2 hit-areas that brighten to --teal on hover (a solid edge highlight, NEVER an animated line, B-01).

6 STATE LANGUAGE (square marks + weight + teal + the inversion chip; no circles, no checks)
  live/running  a small SOLID --teal SQUARE (9px) + label; at most ONE calm opacity pulse (the only motion on the surface)
  done          row recedes (--dim / lower opacity) or a filled --teal-deep bar; NOT a checkmark
  pending       a 1px --line square outline, --dim text, no fill
  warn          a hollow triangle drawn in --text (no color), rare
  fail          LIGHT mono-inversion chip: --text background, --bg text — the loudest thing on the near-black field, zero foreign hue, NEVER red
Doctrine: silence = healthy (no marks on good rows); progress is an explicit n/m on a plain segmented bar (--teal-deep fill on --sink), never a lone percentage, never a glowing bar.

7 THE "?" DETAIL AFFORDANCE (low text + depth-on-demand)
Any element needing explanation carries a small quiet "?" in --dim (grotesque, not a badge, no circle-fill, no glow). On click it opens a small POPOVER (a --panel card, 1px --line2, 2px radius, grotesque body, overlay z-layer) anchored to the "?". The "?" goes --text on hover. Surfaces stay low-text; depth lives behind the "?".

8 MOTION (functional, matte, minimal)
Durations 120/160ms ease-out. Allowed: solid hover fill, the selected block/edge appearing, popover fade+small-rise, the single live-square pulse, a migrating progress mark. BANNED: any grow-from-center line (B-01), glow pulses (B-02), cosmic/staggered intro (B-05), bounce/elastic, ambient/scroll-linked motion, layout-property tweens. prefers-reduced-motion: resolve to the static end state; the live square becomes a static mark.

9 COMPONENTS (build ONCE per §2, index them)
- Primary button: solid --teal fill, --bg text, 2px radius; hover = --teal-bright (no glow, no underline). Secondary: transparent + 1px --line2 + --read; hover fills --raised.
- SessionRow + StatusMark (§6); selected = --raised + 2px --teal left edge.
- Monitor tile: --panel card, title (grotesque 600), data lines (mono), a StatusMark; a chart only where a real number must be read, drawn plainly.
- The "?" + Popover (§7). TokenMeter. Menu. Toggle. WorkerSelect (segmented).
- IconRail: the ~48px left rail of square toggle/action icons (sessions, monitor, +new-pane, usage, connections, settings). Active = filled --teal square or 2px --teal edge; CSS/unicode glyphs only.
- Panel + PanelManager: the collapsible SESSION + MONITOR panels and the movable/resizable WORKSPACE panes. Drag-dividers (1px --line2, --teal on hover); move / delete / create-via-rail-[+]; default = 1 chat pane + monitor; multiple chat panes allowed, each bound to a session. Panes dock/snap.
- UsagePopup: opened from rail [usage] OR the monitor pool summary. ONE section per provider, mono used/cap + a plain segmented bar (--teal-deep on --sink), no gauges, no glow, no color beyond the §6 inversion chip when over limit:
    Anthropic plan — the 5-HOUR rolling window (used/cap + reset) AND the WEEKLY limit (used/cap + reset).
    Devin — free-model slots used/cap + daily quota. Codex — concurrency used/cap + proxy quota. Free APIs — per-key usage/cap.
  A --panel card, 1px --line2, 2px radius, overlay z-layer; opens with the §8 popover motion, no glow.
- BrandCluster + TopRightControls + the wordmark: §16-chrome, ONE shared set across all three screens.
- THE COMPOSER (modeled on the Claude Code CLI composer, ELAI-skinned):
  * text field, placeholder "Queue another message…".
  * left: a "+" menu (Upload from computer / Add context / Browse the web); a "/" menu REPURPOSED to control-plane commands (/orchestrate /spawn /worker /model /grill /handoff /skills /gate /defer /escalate /project); a file-context chip (hide/eye + file name).
  * right: a mic/voice button (recording state), a "Bypass permissions" toggle, a send button with a STOP (square) variant when a run is active.
  * an overflow "…" menu: Context [Attach file, Mention file from this project, Clear conversation, Rewind] and Model [Switch model, Effort (slider, Max), Thinking (toggle), Switch models when flagged (toggle), Account & usage].
  * TOKEN METER (mandatory, ALWAYS visible in the composer bar, mono): context used "ctx 312k / 500k" + a session total "session 1.2M tok". Sourced from the improved CC token-reader (the same reader that drives the handoff/migration trigger). Not hidden in a menu.
- SESSION CONFIG (new-session + chat header): Mode = Interactive (with an Orchestration on/off toggle) | Autonomous (always orchestrated). Worker source = Anthropic · Devin · Codex · Free APIs (segmented). Model choice within the source.

10 STATE COVERAGE (a surface is not done until every state is designed)
empty (a focal next action, not a bare "no data"), loading (a calm --panel skeleton, nothing under ~200ms), error (inversion chip + recovery, never red), populated; live views add disconnected (warn triangle + retry) and stale/cached. All states hold the palette + type + no-glow + high-contrast discipline.

11 EXTERNAL CONNECTIONS (the connection guarantor)
External connections (MCP servers, the remote sidecar, cloud providers, SearXNG) are PROCESS-GLOBAL and PERSISTENT — connect once, available in every session and pane until the user removes or overrides it, NOT re-established per session the way Claude Code does. ELAI already ships the transport plumbing (MCP bridge C-39, defer-loading H-34, the C-42 authz gateway, canvas system-nodes that bind MCP via E-24 pin / G-4 re-pin); NEW here is the health + lifecycle + surface.
- THE GUARANTOR: a single connection registry reports each connection in the §6 state language — live (solid --teal square, no marks when healthy), pending (1px square outline), warn (hollow triangle — connected but the last call failed/slow), fail (the mono-inversion chip — the loudest mark, never red). Silence = healthy.
- IT ANSWERS, without opening a session: which MCP/provider is connected, which FAILED, WHEN, and the last error (depth behind the §7 "?"). Auto-reconnect with backoff; a manual reconnect per row.
- SESSION SCOPE: default = shared across ALL sessions; a per-session override (pin a different endpoint, opt one out) is the explicit exception shown on the session, never the default.
- WHERE IT LIVES: a CONNECTIONS block in the MONITOR panel (one row per connection, §6 marks, mono endpoint + latency), the rail [connections] entry, and a topbar warn mark when any connection is down (so it is noticed without the panel open — designs against the Codex "failure cues too small, easy to miss" complaint). The canvas system MCP-connector nodes read the SAME registry state.
- STATES (§10): empty (focal "+ add connection"), connecting (pending), connected (silent), degraded (warn + retry), failed (inversion + last-error "?" + reconnect). Matte, no glow.

12 GIT + REVIEW SURFACE (commit and diffs WITHOUT the model)
From the Codex desktop review pane (verified across two research passes: stage/revert all, per-file and per-hunk stage/revert, commit with an OPTIONAL auto-message, branch + PR — all plain buttons, no model call). Deliberate divergence from Codex: its IDE extension has NO accept/reject diff flow (edits land straight in the tree, users git-stash defensively) and its desktop approval diff was reported "too small, ~5 lines, no expand". ELAI does the opposite — an expandable accept/reject review is the default.
- PLAIN-GIT ACTIONS, NO LLM: stage / unstage / revert at file AND hunk grain; commit (message field + an OPTIONAL "suggest message" button, the only model-touching control, never required); create branch; create PR. Git verbs are §9 secondary buttons; commit/PR is the solid --teal primary.
- THE DIFF (§6, no status color — amber/red/green are dead): added = a left --teal-deep bar + brighter text, removed = --dim + a struck bar; reads on shape + position + weight. Full-height with expand/collapse per file, never a 5-line peephole. Click a line -> open it.
- ACCEPT / REJECT: an agent's edits land in a REVIEW state, not silently in the tree; accept (stage) or reject (revert) per hunk. The structural answer to Codex's "edits land directly" complaint; composes the shipped G-5 Reviewer Cockpit + H-52 finding-anchoring (verbatim snippet -> hunk, hallucinated line numbers structurally impossible).
- Lives as a workspace review pane (§5), or summoned as a review drawer; the composer STOP/active state ties to the run that produced the diff.

13 NOTIFICATIONS (the alert model)
ELAI already fires completion/needs-input notifications (NotificationManager + H-36, OSC9 + BEL + OS toast). This specs the model.
- TRIGGERS: task done, needs-approval/needs-input, AND error/fail (Codex covers the first two; we add error).
- DELIVERY SETTING (Codex's verified three-way): never / always / only-when-unfocused, per-trigger; an OPTIONAL distinct sound per trigger (Codex users requested sound, it was missing at launch).
- PROMINENCE (against Codex's "too small / easy to miss"): a needs-approval or fail ALSO raises a PERSISTENT mark — a "waiting" badge on the session row + a topbar warn/fail mark — that holds until acted on, not a transient toast only. §6 marks, no glow, honors prefers-reduced-motion (static marks).
- Settings live in §settings; never uses color beyond §6.

14 LAUNCHER / HUB (open-in-X + per-project action buttons)
From Codex desktop's "Open" button, configurable top-of-window action buttons, floating pop-out windows, the codex:// deep links, and CLI<->desktop handoff (all verified). Adapted to the rail + topbar.
- AN "OPEN IN" MENU (topbar right cluster or a rail action): open in editor/IDE, open a terminal at the project root, reveal the project in the OS file manager, open the repo on its remote (GitHub/GitLab) in the browser. Plain shortcuts, no model.
- PER-PROJECT ACTION BUTTONS: a small user-configured row of frequent commands (build/test/run/deploy) at the top of the workspace, project-scoped; square §9 buttons, no glow.
- DEEP LINKS: an elai:// scheme (new-session / settings / connections / a session id); cross-surface handoff (a CLI session -> the desktop shell) mirroring Codex /app.
- POP-OUT: detach a chat or monitor pane into a floating OS window (panes are already movable/resizable §5 — extend to a true pop-out).

15 INTERACTION AFFORDANCES (the VS-Code-grade QOL layer)
The small expected ergonomics, ranked by the research's "pain if missing"; the top band is TABLE-STAKES (absence is resented, not perceived as a feature). All obey §3/§4/§8.
- TABLE-STAKES (ship first): the command palette (already present, keep it keyboard-first); quick-open fuzzy file/session jump (Ctrl/Cmd+P); CLICKABLE paths + links in agent output / chat / terminal (auto-detect URLs + file refs incl. file:line:col, Ctrl/Cmd+click opens at the exact line, highlight the source ref as feedback — never a blind jump); copy path / copy RELATIVE path on every file node + chip; reveal-in-tree (sync the session/file panel to the active file) AND reveal in the OS file manager; open-to-the-side / split (§5 panes); a back / forward navigation stack across jumps.
- MEDIA + CONTENT: inline image preview; hover-to-preview an image path; render images + links in chat and markdown (the memo: "highlight links, make them clickable, same for images").
- CONTEXT MENUS: every file node answers, without mode-switching — where is this (reveal in tree / OS), open it (here / beside), copy its path (absolute / relative).
- AMBIENT FEEDBACK: status-bar progress for long ops (calm, persistent, never a blocking spinner); reversible operations (composes shipped H-49 time-travel + H-35 rewind).
- OPTIONAL / MAKE-DISMISSIBLE (the research's polarized items): sticky scroll, minimap — OFF or trivially togglable; do not force context aids.
- BUILD / TAURI NOTE: reveal-in-OS, native file-manager open, and robust terminal path:line:col parsing need a NATIVE bridge + cross-platform path normalization in the Tauri layer, not pure web-view DOM behavior.

16 THE THREE SCREENS (the ELAI 3-page set, de-slopped)
All three wear the SAME §16-chrome shared cluster (BrandCluster: back chevron + ELAI wordmark lockup, LEFT at a 16px inset; TopRightControls: theme toggle · separator · min/restore/close, viewport-fixed at top:18px right:20px, byte-identical on every screen). The ELAI brand mark is the monoline stroke wordmark (DESIGN.md §5 geometry) shown STATICALLY — drawn instantly, no stroke-by-stroke staggered reveal (B-05). Chrome wordmark = the mono "ELAI" lockup (a wordmark/brand treatment, the only UI-chrome use of mono).

(1) ENTRANCE — a minimal brand screen on a SOLID --bg, no cosmos. The static ELAI stroke wordmark as the single focal element (centered, generous void — the one place STRUCTURE_VARIANCE is high and density is low). An optional one-line tagline in --dim (the tagline copy is still unresolved, §19). A single START button (solid --teal fill on hover, NOT a center-out underline B-01) that dissolves to project-selection's exact background (no flash). The §16-chrome cluster top-right. NO starfield, bloom, rings, or timed draw. prefers-reduced-motion: identical (it is already static). Owner option: collapse this screen entirely and open straight to project-selection (§19).

(2) PROJECT-SELECTION — a desktop software-window on a solid --bg: the §16-chrome topbar (wordmark + back chevron LEFT, the LOCAL/CLOUD/HYBRID mode selector CENTERED as a SEGMENTED control — solid fill on the active mode, a 2px --teal edge, NOT a center-out underline B-01; controls RIGHT), a centered <=840px content column, a thin ~22px statusbar with REAL info (session id · mode · status note). Content = the recent-session ledger: a bordered sub-window (titlebar over a ruled body), a header row (--read PROJECT / PATH / UPDATED, normal case, no dot), name-first rows (no index gutter), tabular-aligned UPDATED. Row hover/select = --raised fill + a 2px --teal LEFT EDGE + the name brightening to --teal (NOT the START-underline). The new-project control = a square hairline button, solid hover fill. Full state set (§10, the worked reference): populated / single-result / empty (dashed box + a focal "+ new project") / loading (calm --panel skeleton) / error (mono-inversion + retry) / offline (warn triangle + retry connection).
  De-slop note vs the live ProjectSelector.svelte: that file uses the center-out underline on rows (.row::after scaleX from center) and on the mode buttons and the new-project button — all B-01 violations. No-slop replaces every one with a solid fill / 2px square --teal edge. Everything else (the ledger structure, name-first columns, the real state set, the race-safe open sequence) is kept.

(3) WORKSPACE — the §5 shell: the left icon rail, the collapsible SESSION (left) + MONITOR (right) panels, and the movable/resizable pane workspace (default = one chat pane + the monitor side-by-side). The chat pane carries the §9 composer with the always-visible token meter. The monitor panel carries running agents, gate verdicts, the pool summary, and the §11 CONNECTIONS block. The review pane (§12) and the canvas open as panes. The UsagePopup (§9) and the connections surface (§11) summon over the workspace on the overlay z-layer. The composer's STOP square shows while a run is active. Full state coverage per §10 on every panel and pane.

17 LIGHT THEME (ELAI ships a full inversion, toggled on every screen — KEPT from ELAI, not slop)
Dark is the default and resting state; light is a complete role-token inversion, never a separate design. The sun/moon toggle sits in the §16-chrome cluster on every screen (resting icon = current mode, hover cross-fades to the target; prefers-reduced-motion = instant swap). Because every color is a role token (§3), the instrument flips wholesale. Light VALUES are PROVISIONAL (retune later; SCOPE is locked): app-window --bg #eef1ef · --panel #ffffff · --raised #f4f6f4 · --sink #e6eae7 · --row #f7f9f7; --text #0b0f0e · --read #2d342f · --dim #646b66 · --ghost #aab1ac; --line/--line2 rgba(0,0,0,.30); teal darkened for near-white --teal #138a78 · --teal-bright #0f7e6f · --teal-deep #0c6256. Teal shifts darker on light so it stays >=3:1. The fail mono-inversion inverts relative to its surface (a DARK chip with light text on the light theme) so the loudest-thing rule holds in both. One accent, one teal, both themes. (No bloom to invert — B-02 removed it.)

18 USER ACCENT THEMING (ELAI Settings feature — KEPT from ELAI, not slop; spec for when Settings exists)
The accent is TEAL by default and that teal IS the ELAI identity. On WORK surfaces only, the accent is user-themeable from Settings: because every color is a role token set once on the root, one accent control re-skins every screen through the cascade with NO per-component work (the same property that makes §17 a one-line flip). Scope (the boundary that keeps the brand): BRAND surfaces (the wordmark/logo, the entrance brand mark, the website, the app icon, the default install) are TEAL-LOCKED, never user-recolorable. WORK surfaces carry the personalization, default teal. The control: a master HUE knob rotates the whole --teal family + the neutral tint preserving each token's S/L (only the hue moves); a VIVIDNESS control scales saturation within clamps (the clamp is what prevents neon slop, B-06 still holds — the user picks WHICH single hue, never adds a second); a MONOCHROME/DISABLE option collapses the accent to a neutral (graphite-on-black), state still carried by §6 shape + inversion. NEVER rotates: the semantic data-viz/state palettes (FSM badges, confidence, source-tier, gate verdicts) — those encode meaning by hue and stay fixed. Marketing posture: a quiet Settings delight, never a headline.

19 OPEN ITEMS
- Whether to keep the ENTRANCE screen (§16-1) or collapse straight into project-selection — owner's call; the no-slop entrance is near-empty by design once the cosmos is gone.
- TAGLINE unresolved (carried from ELAI DESIGN.md §1/§14): short, hits the trust/evidence wedge, avoids the generic-AI register.
- The UI grotesque FACE is unchosen (Hanken Grotesk works; pick via a local @font-face specimen round, paired against JetBrains Mono); the hairline display face is also unchosen (variable hairline weight required).
- Favicon/app-icon glyph distillation (must read at 16px, one color).
- Light-theme color VALUES (§17) are PROVISIONAL — retune later; SCOPE locked.
- OKLCH emission values + measured contrast table — produce at implementation.
- Connection registry (§11): the process-global lifecycle owner location + SSB connection-state rows (reuse McpToolDeferred family or add ConnectionStateChanged) — a build decision.
- Deep links + pop-out (§14): confirm Tauri v2 multi-window + deep-link plugin support the elai:// scheme before committing.
- No-LLM diff encoding (§12): the added/removed shape+weight scheme (no status color) needs a §3 contrast pass.
- QOL v1 scope (§15): table-stakes band is v1; sticky-scroll / minimap / advanced image-diff deferred.
- BIGGEST DEPARTURE flag for the owner: §4 demotes mono from ELAI's default UI face to data-only (B-03). If the owner considers ELAI's mono-everywhere a deliberate instrument identity rather than slop, that is the one ban to reconsider — everything else here is unambiguously a slop tell. Decide before the token file is cut.
- Whether this no-slop language SUPERSEDES the editorial DESIGN.md or runs beside it as the build target — owner's call.

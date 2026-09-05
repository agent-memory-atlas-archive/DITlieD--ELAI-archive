TEIKOKU P4-E CONTEXT PACKAGE — dashboard surfaces (chat pane + per-project session list)
context-compiler (READ-ONLY), task_id=P4-E-CC, 2026-06-21
Scope: WSL distro teikoku ~/teikoku/signal (Tauri v2 + Svelte 5 + Vite, bun) + the ~/teikoku/conductor/slice_api.py seam + the SIGNAL design law (workflow/design/DESIGN-TEIKOKU.md).
All file facts are READ from the live distro (PowerShell -> wsl -d teikoku) + F: docs. Nothing was edited/started.

================================================================================
1. EXISTING SIGNAL DASHBOARD (what to REUSE vs what is MISSING)
================================================================================

Token source (the ONLY color/space/type source; zero hex / magic px downstream):
  ~/teikoku/signal/src/lib/styles/tokens.css — oklch() palette derived from the §3 hex anchors;
  role-named (--ground/--sink/--surface/--surface-2/--text/--text-2/--dim/--accent/--accent-deep/
  --accent-hi/--line/--line-2), spacing --s-1..--s-7 (8px rhythm), --r:2px, --bw:1px,
  --font-ui (Hanken Grotesk Variable), --font-data (JetBrains Mono Variable), --fz-title/body/
  label/caption/data, --fw-*, --lh-*, --rail-w:48px, --panel-session-w:264px, --panel-monitor-w:300px,
  --dur-fast:120ms/--dur-slow:160ms/--ease (ease-out), --z-overlay/--z-popover.
  base.css at src/lib/styles/base.css (global reset, not read in full; not relevant to compose).

THE COMPONENT INDEX (src/lib/components/index.ts) — the SINGLE import surface every screen reuses.
A screen that re-implements one of these instead of importing it is a consistency defect (caught at
audit). The 19 exported components + purpose:
  Button        — primary (solid --accent fill, --ground text) | secondary (transparent +1px --line-2).
  Toggle        — bindable on/off switch (label). Used by Composer (Bypass permissions) + session config.
  StatusMark    — §6 SQUARE state language: live(solid --accent sq + pulse) | done(--accent-deep recede)
                  | pending(1px outline sq) | warn(hollow --text triangle) | fail(LIGHT inversion chip
                  --text bg/--ground text, NEVER red). label prop. THE status primitive for both new surfaces.
  Chrome        — wordmark "TEIKOKU" + 帝国, one shared topbar component.
  Popover       — overlay card (--surface, 1px --line-2, 2px r), fade+rise, bind:open, align left|right.
  QMark         — the "?" detail affordance (§7): a quiet --dim "?" that opens a Popover with depth.
  Menu (+MenuItem)— trigger glyph + Popover list; MenuItem={label,cmd?,section?,onselect?,disabled?}.
                  Sectioned. This is what the Composer "+"/"/"/"…" menus already use.
  WorkerSelect  — segmented radiogroup (default ['Anthropic','Devin','Codex','Free APIs']); selected =
                  SOLID --accent-deep fill (B-01). For the session-config worker-source pick.
  SessionRow    — name + meta(mono) + StatusMark; selected = --surface-2 + 2px --accent LEFT EDGE.
                  THE row primitive for the per-project session list. props {name,meta?,state,selected,onselect}.
  Tile          — --surface monitor card with title + state + fail; data lines as children. (Tile.svelte
                  not read in full but used throughout App/Orchestration; props title/state/fail + children.)
  IconRail (+RailItem)— ~48px left rail; RailItem={id,glyph,label,active?,onactivate?}; active = 2px
                  --accent left edge + --surface-2 fill. CSS/unicode glyphs only. THE view-router seam.
  Panel         — collapsible aside (title, side left|right, width, open, header snippet, children).
  PanelManager  — left panel + workspace + right panel with 1px --line-2 drag-dividers (hover->--accent);
                  NOTE: resize wiring is a "later-phase surface" (dividers present, drag not yet wired) —
                  resize is NOT a P4-E deliverable unless explicitly scoped.
  TokenMeter    — mono "ctx Xk / Yk" + plain gauge (--accent-deep on --sink) + "session N tok".
                  props {ctxUsed,ctxCap,sessionTokens}. ALWAYS visible in the Composer bar.
  UsagePopup (+UsageSection,+UsageLine)— per-provider usage popup (bind:open, sections).
  Composer      — THE chat composer (read in full, 6.6KB). Already ships: textarea ("Queue another
                  message…", Enter=send/Shift+Enter=newline), LEFT "+"(Upload from computer/Add context/
                  Browse the web) + "/"(control-plane: /orchestrate /spawn /worker /model /grill /handoff
                  /skills /gate /defer /escalate /project) menus + a fileContext chip (eye/hide + mono fname),
                  CENTER TokenMeter, RIGHT mic(record state ●/■) + Bypass-permissions Toggle + "…" overflow
                  (Context: Attach/Mention/Clear/Rewind; Model: Switch/Effort/Thinking/Switch-when-flagged/
                  Account&usage) + send(↑)/STOP(■ when running). Callbacks: onsend(text), onstop, oncommand(cmd).
                  fileContext is a SINGLE string prop today (one chip).

THE SHELL (src/App.svelte, read in full) — the precedent for view-routing + the chat pane:
  - View router is a single `view` $state ('shell'|'gallery'|'harness'|'memory'|'orchestration'); the
    IconRail railItems toggle it; non-shell views render full-bleed via {#if view===...}<Gallery/> etc.
    ADD-A-VIEW PATTERN: add a View union member + a RailItem + an {:else if} branch hosting the screen.
  - The shell body already composes IconRail + PanelManager{left:Panel"Sessions"(SessionRow list +
    empty state), workspace: a `.chat` section (transcript .stream + Composer in .composerwrap),
    right: Panel"Monitor"(Tile with loading/warn/pending/populated states)}.
  - It ALREADY renders a minimal one-chat transcript (Turn={role:'system'|'cc',text}) + a session list
    (Session={id,name,meta,state}). P4-E is the PROMOTION of this skeleton to the real P4-C/P4-D surface.

THE PRECEDENT SURFACE (src/lib/screens/Orchestration.svelte, P3-E, read in full) — MIRROR THIS:
  - Pattern: import ONLY from $lib/components/index.ts + a typed api seam; $state snap/error/loading;
    async refresh() with ok/error fork; $effect(()=>refresh()); pure helpers (gateMark/slotMark/relTime/
    pct/aggregate) at top; one <main> with loading/error/empty/populated branches; scoped <style> using
    ONLY tokens (max-width 760px blocks, .rows.scroll max-height 280px overflow-y auto = the P2-C
    bounded-scroll lesson). Skeleton = .skel (--surface-2 bars). This is the exact template the chat
    pane + session list copy.

REUSE vs MISSING for P4-E:
  CHAT PANE reuses: Composer (as-is for text + menus + TokenMeter + mic + bypass + send/STOP),
    StatusMark (migrating indicator), Tile/Panel (monitor side), Button, QMark.
    MISSING for chat pane: (a) a CHAT TRANSCRIPT component if the inline App.svelte .stream markup is
    promoted to a reusable ChatPane/Transcript (currently inline in App, not in the index) — recommend
    authoring ChatPane.svelte (the transcript + composer + migrating-indicator host) so a future second
    chat pane assembles instead of re-inlining; (b) a MigratingIndicator affordance — can be a thin
    composition of StatusMark(state="live"/"warn") + a one-line bar, NOT a new dep; (c) Composer image/
    audio entry exists (the "+" menu items "Upload from computer") but is NOT wired to a handler.
  SESSION LIST reuses: SessionRow + Panel + StatusMark wholesale; project grouping = a project-header
    row (the App.svelte `.proj` line is the seed; promote to a small grouped list).
    MISSING for session list: a per-PROJECT grouping wrapper (header + grouped SessionRows). No new
    component strictly required — it is SessionRow rows under a project-header div, like Orchestration's
    block/bhead idiom. Recommend a SessionList.svelte screen that composes Panel+header+SessionRow.

================================================================================
2. THE SLICE_API SEAM (~/teikoku/conductor/slice_api.py, 529L; manual BaseHTTPRequestHandler dispatch)
================================================================================
Transport pattern (BOTH new api/*.ts files copy slice.ts/orchestration.ts EXACTLY):
  - Tauri native: invoke('signal_<x>') -> a #[tauri::command] in src-tauri/src/lib.rs that shells
    `python3 slice_api.py <verb>` (one-shot CLI), returns JSON stdout verbatim.
  - web/vite dev: fetch http://127.0.0.1:8787/<route>. HTTP_BASE='http://127.0.0.1:8787'.
  - tryInvoke returns undefined when not in Tauri -> falls through to fetch. A failure NEVER fabricates
    a row; it surfaces ok:false + error (offline/error state). This is the wiring-enforcement posture.

Routes P4-E binds to (verified in slice_api.py do_GET/do_POST dispatch + handler bodies):
  POST /chat/turn  (handler chat_turn, L244) — the orchestrator-FRONT entry.
    REQ: {session_id(req), text, resolved_decisions?, target_child?, intent?, project_id?,
          response_model?, usage?{...}, response_text?}.
    RESP: {ok, decision:{...to_dict}, result:{...}, migration?:{should_migrate,...}, migration_required?:bool}.
    NOTE: the grill gate (always_grill ON) returns a grill_required result for the P4-E surface to
    answer + re-submit (pass resolved_decisions back). Per-turn transcript is recorded server-side.
    NOTE: request carries ONLY `text` today — no image/audio content-block field (see §4).
  GET /chat/sessions?parent=<chat-session-id> (handler chat_sessions, L349) — the per-chat roster
    ("one chat = N sessions"). RESP: {ok, parent, count, live, sessions:[children_of rows]}.
    children_of row (spawn.py:100) = {child_task_id, status, objective, model_tier, spawn_depth, live}
    (live = status NOT terminal), oldest-first.
  GET /chat/toggles?session=<id> | POST /chat/toggles (handlers L431/305) —
    GET RESP {ok, toggles:{...to_dict}}. POST REQ {session_id, toggles:{pool/model/skills/objective/
    orchestration/always_grill...}} RESP {ok, toggles:{resolved}}. The session-config surface writes here.
  GET /chat/pools (handler at L437 -> model_pool.list_pools) — RESP {ok, pools:[{pool, default,
    models:[{...to_dict}]}]} for POOLS (controller {claude, devin-free} + liberty {free-api}). The
    /model picker payload.
  POST /chat/migrate (handler chat_migrate, L322) — REQ {session_id, project_id?, reason?,
    from_cc_session_id?, from_model?, to_model?, pool?, token_state?}. RESP {ok, ...MigrationResult}:
    {session_id(UNCHANGED), snapshot_id, from_cc_session_id, to_cc_session_id, from_model, to_model,
    reseeded_messages, project_id}. The chat loop POSTs here on a migration_required response (or the
    D2 trigger fires it). The dashboard does NOT usually call this directly — it SUBSCRIBES to the
    migrating events (below) and shows the indicator.
  GET /events/wait?since=<seq>&project=<p>&recipient=<r> (handler events_wait, L145) — the LONG-POLL.
    Blocks on the in-process wake hub until event_seq>since (matching project, addressed to recipient
    when given) or EVENTS_WAIT_TIMEOUT_SECONDS, then RESP {ok, since, project, recipient, count,
    events:[...]}. recipient=<session_id> narrows to that chat. THIS is the migrating-indicator +
    live-update feed for the chat pane.
  GET /orchestration (handler build_orchestration_snapshot) — the P3-E monitor snapshot (task counts/
    waves/frontier/leases/workers/gates/concurrency/wake). For project grouping in the session list,
    /orchestration gives plan_id/wave/role per task; /chat/sessions gives the per-chat children. Project
    grouping in the session list is best driven by the children's project_id + the orchestration tasks.

THE MIGRATING-INDICATOR EVENTS (the load-bearing fact for §3):
  session_migrate.py emits, on the SAME session_id, in order: EVENT_SESSION_MIGRATING="session_migrating"
  then EVENT_SESSION_MIGRATED="session_migrated" (L47-48), each wake.publish(recipient=session_id) so a
  /events/wait?recipient=<session_id> waiter resumes instantly. Also migration_trigger emits
  EVENT_MIGRATION_REQUIRED="MIGRATION_REQUIRED" (the pre-migrate signal). The chat pane subscribes
  /events/wait?recipient=<chat session_id>&since=<last_seq>, watches event_type for these three, and
  drives the indicator (MIGRATION_REQUIRED/session_migrating -> "migrating" live mark; session_migrated
  -> clear + a brief "migrated to <to_model>" caption sourced from the migrated event payload).

================================================================================
3. CHAT PANE DESIGN (SIGNAL-compliant component plan)
================================================================================
Surface read: "the interactive control surface for the operator, mission-control language, leaning
system." Dials: STRUCTURE_VARIANCE 3 (legible, panel-framed), MOTION_INTENSITY 2 (one live pulse +
popover fade + the migrating mark only), VISUAL_DENSITY 5 (dense transcript, airy composer).

Components (REUSE first; one new screen + optionally one new component):
  - TRANSCRIPT VIEW: promote App.svelte's inline `.stream` to ChatPane.svelte (NEW screen-level
    component) so a second pane assembles, not re-inlines. Turn rows = the existing .turn/.who/.msg
    idiom (who in --font-data --dim, --accent for the model role). Bounded-scroll the stream
    (overflow-y auto), the P2-C lesson. States §10: empty (the existing .welcome focal next-action,
    NOT a bare "no data"), populated, error-inversion (a failed turn shows a StatusMark fail chip on
    that row), loading (a --surface skeleton turn). Calls POST /chat/turn; on result.grill_required
    surface the grill questions inline and re-POST with resolved_decisions.
  - COMPOSER: REUSE the shipped Composer AS-IS. ctxUsed/ctxCap/sessionTokens now bind to the P4-D
    per-model real values (model_pool window-source fix already lands the real ctx/window into the
    /chat/turn migration block + slice event; thread those into the TokenMeter props — TokenMeter is
    already mono "ctx x/y" per §9). The "+" menu's "Upload from computer" is the image/audio entry (§4).
    The "/" control-plane + "…" Model/Context menus already match §9. onsend -> POST /chat/turn,
    onstop -> abort the in-flight turn, oncommand -> route the control-plane verb.
  - DROP-IN / INTERVENTION control: inject into a RUNNING child session. The substrate is P4-A
    mailbox + the generalized base-cc drop-in (recipient=<session id>, TEIKOKU_DROPIN=1). Surface = a
    small per-child action on the live child row (in the session list / a child sub-list) OR a Composer
    target-child mode: pass target_child=<child session id> + intent on /chat/turn (chat_orchestrator
    apply dispatches mailbox.send_message to the child). REUSE Button(secondary) + a target chip; no new
    component. A "drop-in to <child>" chip in the composer (like the fileContext chip) is the cleanest.
  - MIGRATING INDICATOR: subscribe /events/wait?recipient=<chat session_id> (a long-poll loop in the
    chat api seam). On MIGRATION_REQUIRED|session_migrating show a NON-ALARMING SIGNAL mark — a
    StatusMark state="live" (solid --accent square + the single calm pulse) with a "migrating context…"
    label, OR a thin --accent-deep progress bar (the .bar idiom from Orchestration/TokenMeter), placed
    as a one-line banner ABOVE the composer or as a transcript system-row. On session_migrated clear it
    and drop a --dim caption "migrated <from_model> -> <to_model>, <reseeded_messages> turns carried"
    (sourced from the migrated event payload / the MigrationResult). LIGHT inversion is reserved for
    FAIL only; migration is NOT a failure — use live/done marks, NEVER red, NEVER the inversion chip
    (per B-06/B-07 + §6 doctrine "the inversion chip is fail only").
  File discipline: ChatPane.svelte (transcript + composer host + migrating banner) as one focused
    screen; a tiny MigratingIndicator.svelte ONLY if it earns reuse (else inline the StatusMark+bar).
    Zero hex / magic px — every value from tokens.css.

================================================================================
4. IMAGE + AUDIO UPLOAD (feasibility seam)
================================================================================
IMAGE — FEASIBLE, ship it. base-cc HAS native image support (verified: base-cc/src/services/api/
errors.ts imports ImageResizeError + ImageSizeError from utils/imageResizer + imageValidation, and has
stripImagesFromMessages + getImageTooLargeErrorMessage — i.e. the CC build already sends Anthropic image
content blocks {type:"image",source:{type:"base64",media_type,data}}). The "+" Composer menu "Upload
from computer" is the entry point; @tauri-apps/plugin-dialog is ALREADY a dep (file picker). GAP: the
conductor POST /chat/turn route accepts ONLY `text` today — no image/content-block field. So image
upload needs a SMALL seam extension: add an optional `attachments`/`images` field to the /chat/turn
payload (base64 + media_type) threaded through chat_orchestrator -> the base-cc --print invocation as an
image content block. That is a conductor + base-cc edit (base-cc edits are SANCTIONED per the contract),
NOT a new dependency. Recommend: ship image upload (picker via plugin-dialog -> read file -> base64 ->
attach to the turn), with the route field added.
AUDIO — DEFER (needs a new dep / transcription step). Verified: NO whisper/transcription/speech-to-text
dependency anywhere in conductor/*.py (only "transcript" = the D3 chat-message log, unrelated). Audio
input would need either a transcription model/dep (whisper-cpp / a cloud STT) OR Anthropic audio support
(not present in this base-cc). Per no-new-deps-without-confirmation: SHIP an audio AFFORDANCE STUB (the
"+" menu shows an "Add audio" item OR the existing mic ● button stays as a recording-state affordance)
that is visibly present but routes to a "transcription not configured" notice, and DOCUMENT the defer
(owner confirms a transcription dep before wiring). The mic button already exists in Composer with a
recording state — keep it as the affordance; do not wire it to a real STT path this phase.

================================================================================
5. PER-PROJECT SESSION LIST (Codex-like)
================================================================================
Bind: GET /chat/sessions?parent=<chat session_id> for the per-chat roster (live + terminal children,
{child_task_id,status,objective,model_tier,spawn_depth,live}); GET /orchestration for project/plan/wave/
role grouping context. REUSE SessionRow + Panel wholesale.
  - Grouping: group children by project_id (and/or the chat they belong to) under a project header
    (the App.svelte `.proj` line idiom + Orchestration's .block/.bhead header). One header per project
    (name + a small --dim tag: rust / unity · claude-only / free-api per §5), then its SessionRows.
  - Per-session status: StatusMark SQUARES only — live(running child) | done(terminal success/recede) |
    pending(queued) | warn | fail(inversion chip). NEVER circle/check (B-10), NEVER red (B-06). Map the
    child `status`/`live` to a MarkState with a single pure helper (mirror Orchestration's slotMark/
    gateMark). SessionRow already renders name + meta(mono) + StatusMark + 2px --accent selected edge.
  - Bounded/scrollable per the P2-C overflow lesson: the list block max-height + overflow-y auto
    (.rows.scroll idiom, max-height ~280px), never an unbounded N-row dump.
  - State coverage §10: empty (focal "Start a session" Button, not bare "no data"), loading (skel rows),
    error-inversion (backend unavailable Tile + Retry), populated, stale/disconnected (warn triangle +
    retry on the long-poll dropping). Recommend SessionList.svelte composing Panel + project-header +
    SessionRow, hosted in the left Panel (replacing/extending App.svelte's current inline Sessions panel).

================================================================================
6. THE DESIGN PASS + WIRING (what the auditor will check)
================================================================================
SIGNAL design pass (slop bans, §1 + design-taste pre-flight; a surviving tell = BLOCK):
  - ONE accent (dark teal --accent), near-black --ground, neutrals tinted teal (already in tokens.css).
  - radius EXACTLY 2px (--r); NO glow/bloom/shadow (box-shadow none only, B-02); NO gradients/glass.
  - Hanken Grotesk for ALL UI/labels/body; JetBrains Mono ONLY for numeric data/tokens/timestamps/
    status codes (B-03). ctx meter + session count + child ids = mono; everything human = Hanken.
  - StatusMark SQUARES only, NO circle+check (B-10); fail = LIGHT inversion chip, NEVER red (B-06/B-07);
    NO blue/red/orange/amber/purple/indigo anywhere.
  - NO center-out underline on hover/select (B-01) — SessionRow/IconRail already use solid edge/fill.
  - Full state coverage §10: empty(focal next action) / loading(--surface skeleton, nothing <200ms) /
    error(inversion chip + recovery) / populated; live views add disconnected(warn triangle + retry) +
    stale. prefers-reduced-motion -> static end state (StatusMark already gates the pulse).
  - Motion §8: 120/160ms ease-out; allowed = solid hover fill, popover fade+rise, the single live pulse,
    the migrating mark. BANNED = grow-from-center line, glow pulse, cosmic intro, bounce/elastic,
    scroll-linked / layout-property tweens.
  - Contrast gate: >=4.5:1 body, >=3:1 large/UI over the REAL --ground (ax-scaffold pixels).

CROSS-SCREEN CONSISTENCY (§2, the audit's hard check): the chat pane + session list MUST import the
identical $lib/components/index.ts + read ONLY tokens.css. ZERO literal hex, ZERO magic px, ZERO
re-implemented component (a re-implemented SessionRow/StatusMark/Composer or a value not in the token
set = a defect). The two new screens compose the existing index, exactly like Orchestration.svelte.

CDP CAPTURE PATTERN (visual-ui-loop.md) for the screenshots: deviceScaleFactor 1
(Emulation.setDeviceMetricsOverride), fromSurface:true (Page.captureScreenshot, fixes the empty-bottom-
rows headless bug), launch Chrome --font-render-hinting=none, kill motion (setEmulatedMedia
prefers-reduced-motion + zero-duration style + blur the caret) + settle 2 rAF. Dev server vite --port
1420 (package.json scripts.dev), Chrome --remote-debugging-port=9222. Tools in .claude/tools/visual/
(ui-diff.mjs, ax-scaffold.mjs, vlm-prep.mjs); pixel-diff ONLY vs known-good engine renders, semantic
critique vs the mock. Existing capture harness idiom at dashboard-v2/screenshots/cdp_*.mjs (the orbit
capture is the proven pattern); signal has no screenshots/ dir yet — the auditor authors fresh CDP
captures (the P3-E precedent: the auditor drove its OWN captures for live-sparse + forced-offline +
seeded-populated).

PRODUCTION_HOOK (wiring, D-030 — the new surfaces are SVELTE views + TS api seams, not new pub Rust/
Python fns, so the wire is the view-router + the route binding, not a python_wire_check symbol):
  - The new screens wire into App.svelte: add ChatPane/SessionList to the View union + a RailItem in
    railItems + an {:else if view===...} branch (the exact Gallery/Harness/Memory/Orchestration pattern).
    The default shell's Sessions panel + chat workspace are the natural home (promote the inline
    skeleton). SEAM: src/App.svelte railItems + the {#if view} router.
  - The api seams: NEW src/lib/api/chat.ts (fetchSessions/postTurn/getPools/getToggles/setToggles/
    watchEvents long-poll) copying slice.ts/orchestration.ts two-transport EXACTLY. They hit the REAL
    routes /chat/turn /chat/sessions /chat/toggles /chat/pools /events/wait (and /chat/migrate is driven
    by the loop, the dashboard subscribes the events). For the Tauri-native leg, ADD #[tauri::command]
    wrappers in src-tauri/src/lib.rs shelling `python3 slice_api.py <verb>` AND register them in
    generate_handler![...] (currently 8 commands; the slice_api one-shot CLI must gain chat verbs, OR
    the web/http transport alone is acceptable for dev — confirm with the executor whether native chat
    commands are in P4-E scope or web-transport-only like the dev path). SEAM: src-tauri/src/lib.rs
    generate_handler! + slice_api.py main() one-shot verbs.
  - IMAGE seam (§4): extend POST /chat/turn payload with attachments + thread through chat_orchestrator
    -> base-cc image content block (a conductor + sanctioned base-cc edit, no new dep).

BUILD / CHECK GATES (the auditor runs, all must be green):
  - svelte-check: `bun run check` (svelte-check --tsconfig ./tsconfig.json) in ~/teikoku/signal.
  - vite build: `bun run build` (vite build).
  - cargo check (src-tauri): cargo check in ~/teikoku/signal/src-tauri.
  - the SIGNAL design pass (CDP captures + the slop-ban checklist) + cross-screen consistency (zero hex,
    index reuse) + R10 integration smoke (the whole P4 plane end-to-end on a HARD input: a chat = N
    sessions + a real migration firing the indicator + drop-in isolation), per the handoff's P4-E plan.

================================================================================
VALIDATION WARNINGS
================================================================================
HIGH: POST /chat/turn accepts ONLY `text` today — image upload REQUIRES a route + base-cc content-block
  extension (sanctioned edit, no new dep). Do NOT report image upload "done" without the route field.
HIGH: AUDIO has NO transcription dep anywhere in conductor — audio is a DEFER (affordance stub only)
  per no-new-deps-without-confirmation; the mic button is the existing affordance, do not wire real STT.
MED: PanelManager drag-resize is "later-phase" (dividers present, drag NOT wired) — resize is NOT a
  P4-E deliverable unless the executor is explicitly scoped for it.
MED: Tauri-native chat commands (#[tauri::command] + slice_api one-shot verbs) are NOT yet present;
  confirm whether P4-E ships them or rides the web/http transport for dev (the P3-E memory/orchestration
  surfaces DID add native commands — consistency argues for adding them, but it is a scope call).
MED: the migrating indicator MUST be live/done marks, NEVER the inversion chip / red — the chip is
  FAIL-only per §6; a "migrating shown as fail" would be a design-pass BLOCK.
LOW: signal has no screenshots/ capture dir yet — the auditor authors fresh CDP captures (P3-E precedent).
All other sources validated (tokens.css, index.ts, App.svelte, Orchestration.svelte + .ts, Composer +
all core components, slice_api.py routes + handlers, spawn.children_of, session_migrate events,
model_pool.list_pools, src-tauri/lib.rs handler list, package.json scripts — all READ in the live distro).

================================================================================
SOURCES CONSULTED
================================================================================
F:\.ELAI_workflow\workflow\design\DESIGN-TEIKOKU.md (SIGNAL law v3, full)
F:\.ELAI_workflow\.claude\rulebook\design-taste.md (full); .claude\rules\visual-ui-loop.md (full)
F:\.ELAI_workflow\.claude\rules\agent-dispatch-preamble.md; .claude\agents\context-compiler.md
F:\.ELAI_workflow\workflow\handoff\teikoku-build.md (HEAD ~70 lines: CURRENT STATUS + INDEX + contract)
~/teikoku/signal: src/App.svelte, src/lib/components/index.ts, src/lib/styles/tokens.css,
  src/lib/api/slice.ts, src/lib/api/orchestration.ts, src/lib/screens/Orchestration.svelte,
  src/lib/components/{Composer,SessionRow,StatusMark,Panel,TokenMeter,Menu,IconRail,PanelManager,
  WorkerSelect}.svelte, src-tauri/src/lib.rs (commands + generate_handler), package.json (scripts/deps)
~/teikoku/conductor: slice_api.py (do_GET/do_POST dispatch + chat_turn/chat_sessions/chat_toggles/
  chat_migrate/events_wait/events_notify/spawn handlers), spawn.py (children_of), session_migrate.py
  (MigrationResult + session_migrating/session_migrated events), migration_trigger.py
  (MIGRATION_REQUIRED), model_pool.py (list_pools); base-cc/src/services/api/errors.ts (image support)

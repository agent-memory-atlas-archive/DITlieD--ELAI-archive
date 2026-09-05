TEIKOKU P2-C CONTEXT PACKAGE — master-plan 2.5 dashboard memory/timeline surface
task_id=TEIKOKU-P2-C-context | compiled READ-ONLY from live distro `teikoku` + F: design law | last Phase-2 cycle, mirrors the proven P1-C harness surface
GROUNDING NOTE: every JSON sample below is a REAL response captured from the live control.db via `python3 slice_api.py memory-*` CLI (the HTTP server was DOWN at compile time; the CLI reads the SAME DB the HTTP routes read — see UNKNOWNS). Component/token facts are read from the distro source with file:line.

==============================================================================
0. THE ONE-LINE DELIVERABLE (what the executor builds)
==============================================================================
A new SIGNAL screen `Memory.svelte` + a new api seam `memory.ts` (two-transport, mirror harness.ts) + a new IconRail 'memory' view mounted in App.svelte exactly like Harness. A LEAN timeline LIST (N-03: NO graph / 3D / viz) with a per-project|global selector and a search box hitting /memory/search. Composes ONLY $lib/components; zero re-implemented components; zero literal hex / magic px (precedent-class layout sizing matching Harness/Gallery is the ONLY allowed raw px, and even those should reuse tokens where one exists).
Files to CREATE (3):
  signal/src/lib/screens/Memory.svelte        (the surface; mirror screens/Harness.svelte)
  signal/src/lib/api/memory.ts                 (the seam; mirror api/harness.ts)
  signal/screenshots/cap_memory.mjs            (the CDP capture; mirror screenshots/cap_harness.mjs)
Files to EDIT (1):
  signal/src/App.svelte                        (add 'memory' to View union + railItems + the {:else if view==='memory'} host)
Backend = ALREADY BUILT in P2-A/P2-B (slice_api.py routes + memory_store.py). DO NOT touch the backend — it is the data source, not in scope.

==============================================================================
1. MEMORY DATA SHAPE + EXACT HTTP ROUTES/PARAMS (the data source)
==============================================================================
Backend file: conductor/slice_api.py (HTTP on 127.0.0.1:8787). Data layer: conductor/memory_store.py -> conductor/conductor_db.py.
Row producer: conductor_db.recent_events()  conductor_db.py:432-463 (used by /memory/timeline + /memory/global).
Search producer: memory_store.memory_search()  memory_store.py:290-315 (grep floor unconditional; semantic merged when memory_semantic loads, else mode="grep").

THE EVENT ROW SHAPE (returned by /memory/timeline + /memory/global .events[]):
  event_seq   : int     (monotonic; newest-first ordering key; use as the stable list key)
  event_id    : string  (uuid or hash)
  task_id     : string|null
  event_type  : string  (OPEN TEXT, MIXED CASE — see §2 taxonomy. NOT a frozen enum.)
  payload_json: string  (a JSON STRING, NOT a parsed object — the surface must JSON.parse() it for the popover, and fail-soft to the raw string)
  created_at  : string  (ISO-8601 "2026-06-19T20:15:34Z"; render as a RELATIVE timestamp)
  project_id  : string  (the project this event belongs to; may be null on legacy rows)
  session_id  : string|null  (cross-session join; often null)

ROUTE A — per-project + global timeline:
  GET /memory/timeline?project=<id>&limit=<n>     project OMITTED or empty => server passes None; BUT note timeline with no project returns project="*" yet still filters by None... (it is the per-project route; for GLOBAL use route B). limit default 100.
  GET /memory/global?limit=<n>                     the GLOBAL cross-project route; ALSO returns the project roster.
  slice_api.py:79-95 (memory_timeline / memory_global), routing slice_api.py:153-156.
  Tauri-native one-shot CLI mirror: `python3 slice_api.py memory-timeline <project>` / `memory-global` (slice_api.py:190-195) — the native transport, same shape.

  REAL /memory/global SAMPLE (captured live, limit=3 trimmed):
  {"ok": true,
   "projects": ["auditproj_A","auditproj_B","auditproj_mcp","auditproj_sem","conductor","cove_proj","elai","insert_ok_proj","isoP1","isoP2","kenji","liberty","projA","projB","teikoku","test","test2"],
   "count": 100,
   "events": [
     {"event_seq":3507,"event_id":"34ba49dcd700fa5cc217d7c6bef9f1c1","task_id":null,"event_type":"handoff",
      "payload_json":"{\"entry_date\":\"2026-06-20\",\"source_file\":\"teikoku-build.md\",\"summary\":\"Reworked the TEIKOKU runtime memory model...\",\"text\":\"2026-06-20 P2-B-exec\\n\\nSUMMARY\\n...\"}",
      "created_at":"2026-06-19T20:15:34Z","project_id":"teikoku","session_id":null},
     ...
   ]}
  -> global view: render .projects[] as the project selector options (+ an "all / *" global option); render .events[] as the timeline list.

  REAL /memory/timeline?project=teikoku SAMPLE (live, 5 rows):
  {"ok": true, "project": "teikoku", "count": 5, "events": [
     {"event_seq":3507,...,"event_type":"handoff","project_id":"teikoku","created_at":"2026-06-19T20:15:34Z","session_id":null},
     {"event_seq":3506,"event_type":"state","project_id":"teikoku","created_at":"2026-06-19T20:15:26Z"},
     {"event_seq":3505,"event_type":"handoff",...}, {"event_seq":3499,"event_type":"handoff",...}, {"event_seq":3498,"event_type":"state",...}
  ]}
  -> THIS is the audit's "a real indexed event visible in the timeline": project=teikoku, 5 real rows incl. handoff event_seq 3507.

ROUTE C — search (grep + semantic):
  GET /memory/search?q=<query>&project=<id>&limit=<n>   slice_api.py:98-115, routing slice_api.py:157-159 (limit clamped to min(limit,25)).
  Native CLI mirror: `python3 slice_api.py memory-search <q> <project>` (slice_api.py:197-201).
  REAL /memory/search?q=memory SAMPLE (live):
  {"ok": true, "query": "memory", "project": "*", "mode": "grep", "count": 8,
   "hits": [
     {"score":1.0,"source":"event:3507","project":"teikoku","event_type":"handoff","created_at":"2026-06-19T20:15:34Z","text":"entry_date=2026-06-20 source_file=teikoku-build.md summary=Reworked the TEIKOKU runtime memory model..."},
     ... (8 hits)
   ],
   "rendered": ["[score:1.00 source:event:3507 project:teikoku type:handoff at:...]\n<text>", ...]}
  -> search HIT shape differs from the timeline event shape: {score, source, project, event_type, created_at, text}. NO payload_json on a hit; the prose is already flattened into .text. mode is "grep" or "semantic+grep" (surface it as a small mono note; semantic was ABSENT at capture so mode="grep").
  Handoff-markdown hits have source="handoff:<file>:<line>" and project="" (empty); event hits have source="event:<seq>".

POST routes (NOT required for the read-only surface, optional "refresh memory" affordance):
  POST /memory/index-handoffs?project=<id>  slice_api.py:166-170 — drives handoff_indexer; out of scope unless you add a Reload-that-reindexes button (KEEP LEAN, prefer a plain client re-fetch Reload like Harness).

==============================================================================
2. event_type TAXONOMY + StatusMark MAPPING (square mark by event_type)
==============================================================================
event_type is OPEN TEXT, mixed case. REAL distinct types by count (live control.db):
  TASK_DEP_RECORDED 1720, TASK_IMPORTED 1689, GATE_START 15, GATE_PASS 14, decision 11, TASK_STATUS_CHANGED 7,
  SLICE_TURN_DONE 6, SLICE_SESSION_STARTED 6, change 5, SLICE_COMPLETE 5, state 4, session_message 4, handoff 3,
  LOOP_STAGE 3, LEASE_RELEASED/LEASE_ACQUIRED 3, GATE_FAIL 1, LOOP_ERROR 1, STATE 1, SCHEDULER_PAUSED 1, ...
memory_store named memory types (memory_store.py:42-46): "decision","change","state","session_message" (+ "handoff" from the indexer).
Because the set is OPEN and unbounded, map by a SMALL deterministic classifier into the §6 StatusMark states — DO NOT enumerate every type. Suggested mapping (LAW §6, keep it tiny, fail-soft to 'pending'):
  fail   : event_type matches /FAIL|ERROR/i              (e.g. GATE_FAIL, LOOP_ERROR)  -> light-inversion chip (NEVER red, §6/B-06)
  done   : event_type matches /PASS|DONE|COMPLETE/i       (e.g. GATE_PASS, SLICE_COMPLETE, SLICE_TURN_DONE) -> accent-deep recede
  live   : event_type matches /START|STARTED|RUNNING/i    (e.g. GATE_START, SLICE_SESSION_STARTED) -> accent square (NO pulse on a historical row; pass plain state, the pulse is for a live run)
  pending: everything else (the default; decision/change/state/handoff/TASK_IMPORTED/etc.) -> 1px outline square, dim
This is a CLASSIFIER (a typed dispatch helper), not a per-row conditional pile — keep it a single pure function (maintainability-review §0/§2: avoid spaghetti). NOTE the doctrine "silence = healthy" (§6): most rows are pending (a quiet outline), only fail screams.

==============================================================================
3. EXACT EXISTING COMPONENTS / TOKENS TO REUSE (import paths from index.ts)
==============================================================================
ALL imports from '$lib/components/index.ts' (signal/src/lib/components/index.ts). REUSE — do NOT re-implement (a re-implemented component = consistency defect caught at audit, LAW §2).
  StatusMark  — props {state?: 'live'|'done'|'pending'|'warn'|'fail'; label?: string}. SQUARE mark (§6). StatusMark.svelte:7-14. USE for the per-row event-type mark.
  Tile        — props {title: string; state?: State; fail?: boolean; children}. Tile.svelte:6-16. USE for the empty/loading/error states (mirror Harness's <Tile state="warn">/"pending"> pattern) and optionally to frame each timeline group.
  Button      — props {variant?: 'primary'|'secondary'; disabled?; type?; title?; onclick?; children}. Button.svelte:5-19. USE for Retry/Reload + the project|global toggle if rendered as segmented buttons.
  QMark       — props {children} (the explanatory popover content). QMark.svelte:5. USE in the header "?" (mirror Harness hhead QMark, Harness.svelte:65-69) AND as the per-row payload-depth affordance (click-? opens the payload, §7).
  Popover     — props {open=$bindable(false); align?: 'left'|'right'; onclose?; children}. Popover.svelte:4-14. QMark already wraps Popover; for a per-row payload popover anchored to a row "?", reuse QMark (simplest) OR Popover directly if you need custom anchoring.
  SessionRow  — props {name; meta?; state?; selected?; onselect?}. SessionRow.svelte:5-17. The CANONICAL row look (square mark + Hanken name + mono meta + 2px accent left edge on select). PREFER composing SessionRow for event rows if the shape fits (name=event_type, meta=`${project_id} · ${session_id ?? '—'} · ${rel_ts}`); else build a layout-only `.eventrow` wrapper around the imported StatusMark exactly as Harness does for `.packrow` (Harness.svelte:124-132, 242-266) — that is the SANCTIONED pattern when a row needs more columns than SessionRow exposes.
Global helper class: `.mono` (base.css:60-63) — JetBrains Mono + tabular-nums. USE on ALL DATA: event_seq, timestamps, project_id, session_id, counts (LAW B-03: mono is for DATA only, names/labels stay Hanken). Harness uses `class="... mono"` exactly this way (Harness.svelte:110,130).

TOKENS (the ONLY source of color/space/type; signal/src/lib/styles/tokens.css). NO new hex, NO magic px:
  color: --ground --sink --surface --surface-2 --text --text-2 --dim --accent --accent-deep --accent-hi --line --line-2
  space: --s-1..--s-7 (4/8/12/16/24/32/48px); radius --r (2px); border --bw (1px)
  type:  --font-ui --font-data; --fz-title/body/label/caption/data; --fw-title/body/label/caption; --lh-body/tight
  motion: --dur-fast(120) --dur-slow(160) --ease; layout: --rail-w(48) --panel-session-w(264) --panel-monitor-w(300); z: --z-overlay --z-popover
PRECEDENT layout sizing (the only raw px allowed, matching Harness/Gallery): Harness uses `max-width: 760px` on a block (Harness.svelte:220) and `min-width: 72px` on a name cell (Harness.svelte:256). The skeleton uses `height:10px; width:55%` (Harness.svelte:300-307). Match those PRECEDENT classes; do not invent new magnitudes.

==============================================================================
4. HOW IT MOUNTS IN THE SHELL (new IconRail 'memory' view — mirror Harness)
==============================================================================
EDIT signal/src/App.svelte exactly as Harness was wired (App.svelte:24-32, 116-123, 146-150):
  (a) import: `import Memory from '$lib/screens/Memory.svelte';`  (next to the Harness import, App.svelte:25)
  (b) View union: `type View = 'shell' | 'gallery' | 'harness' | 'memory';`  (App.svelte:32)
  (c) railItems: add a $derived RailItem (App.svelte:116-123 pattern). RailItem = {id, glyph, label, active?, onactivate}. Use a CSS/unicode glyph (NO Lucide, LAW §5/B-09). Harness uses '▣', Gallery '▦' — pick a distinct unfilled-square-ish glyph for memory e.g. '▤' is taken by usage; '◰' or '⊞' or '▥' are unused candidates (drawn unicode, no icon set):
      { id: 'memory', glyph: '▥', label: 'Memory', active: view === 'memory', onactivate: () => (view = view === 'memory' ? 'shell' : 'memory') }
  (d) host: add `{:else if view === 'memory'}<div class="galleryhost"><Memory /></div>` in the view switch (App.svelte:146-150 chain). Reuse the existing `.galleryhost` class (App.svelte:275 — flex:1; overflow:hidden) — do NOT add a new host class.
  (e) crumb (optional, App.svelte:137): extend the breadcrumb ternary to show 'memory · timeline' when view==='memory'.
The screen is a full-width host (like Harness/Gallery), NOT a panel inside PanelManager. Its own root `<main>` owns scroll + padding (mirror Harness.svelte:184-196: flex:1; overflow-y:auto; padding var(--s-5); background var(--ground)).

api/memory.ts SHAPE (mirror api/harness.ts:1-93 EXACTLY — two transports, never fabricates):
  - TAURI native: tryInvoke<string>('signal_memory_timeline' | 'signal_memory_global' | 'signal_memory_search') then JSON.parse. (NOTE: the Tauri #[tauri::command] handlers for memory may NOT exist yet — see UNKNOWNS. The Python native one-shot CLI exists; whether a Rust command wraps it is unverified. If absent, tryInvoke returns undefined and the web/fetch path is taken — same graceful pattern as harness.ts:65-73, so the surface still works in the vite dev build used for the design pass.)
  - web/dev: fetch `${HTTP_BASE}/memory/timeline?...` etc, HTTP_BASE='http://127.0.0.1:8787' (harness.ts:51). Non-ok => {ok:false, error:`http ${status}`}; catch => {ok:false, error:`offline: ...`}. NEVER fabricate rows (wiring-enforcement; harness.ts comment 6-8).
  - export TS types: MemoryEvent (the §1 row shape), MemoryTimeline {ok; project?; projects?; count; events: MemoryEvent[]; error?}, MemoryHit {score; source; project; event_type; created_at; text}, MemorySearch {ok; query; project; mode; count; hits: MemoryHit[]; rendered: string[]; error?}.
  - export fns: fetchMemoryTimeline(project?: string, limit?), fetchMemoryGlobal(limit?), fetchMemorySearch(q, project?, limit?). Mirror harness.ts fetchHarnessManifest signature/error handling.
Memory.svelte state pattern: mirror Harness.svelte:22-41 — `let data = $state(null); let error = $state(null); let loading = $state(true);` + an async refresh() + `$effect(() => refresh())`. Add `let project = $state<string|'*'>('*')` (global default) and `let q = $state('')`; re-fetch on project/q change (a refresh() call from an onclick / oninput, NOT an unbounded $effect on q to avoid a fetch-per-keystroke — debounce or fetch on Enter/Search-click).

==============================================================================
5. STATE COVERAGE (LAW §10 — a surface is not done until every state is designed)
==============================================================================
Mirror Harness.svelte:72-93 + App monitor tile:212-239. Required states:
  loading   : calm --surface skeleton (the `.skel`/`.skel.short` pattern, Harness.svelte:300-307), nothing under ~200ms. <Tile title="Loading memory"><span class="skel"/></Tile>.
  empty     : NO events for the selected project/global. A focal next action, not a bare "no data" (§10). e.g. <Tile state="pending"><span class="empty-line">No memory events for <project> yet.</span><Button variant="secondary" onclick={refresh}>Reload</Button></Tile>. (Harness empty: Harness.svelte:87-93.)
  empty-search: query returned 0 hits — distinct from no-events. "No matches for '<q>'." + clear-search affordance.
  error     : backend unavailable. LIGHT-INVERSION chip / warn, NEVER red (§6/B-06). Reuse <Tile title="Memory" state="warn"><span>backend unavailable</span><span class="dim">{error}</span><Button variant="secondary" onclick={refresh}>Retry</Button></Tile> (Harness.svelte:79-86). For a FAIL emphasis use Tile fail={true} (light inversion, Tile.svelte:19-20/61-67) — that is the only loud non-teal thing.
  offline   : same as error but message from the seam's `offline: ...` string (the fetch catch path). Treat identically to error (warn triangle + Retry).
  populated : the timeline list.
  (live-views note: §10 also lists disconnected/stale; for a HISTORICAL timeline these collapse into the error/offline+Retry state — no live socket, so no separate disconnected state needed. Keep LEAN.)
ALL states hold palette + type + no-glow + high-contrast discipline.

==============================================================================
6. TIMELINE UX SPEC (LEAN list — N-03)
==============================================================================
HEADER (mirror Harness .hhead, Harness.svelte:62-70): title "Memory" + sub "timeline · search" + a QMark explaining: "An append-only log of decisions, changes, handoffs and run events, per project and across all projects. Searchable by grep (always) and semantic (when the embedding model is loaded)."
SCOPE SELECTOR (per-project + global): a segmented / dropdown control of {'*' (Global)} + each id from /memory/global .projects[]. Default = Global ('*'). Selecting a project re-fetches /memory/timeline?project=<id>; selecting Global fetches /memory/global. Render the active scope; keep it ONE control, not a row of tabs (maintainability: typed dispatch, §2/§6 of the design law's consistency mandate). Reuse Button(secondary, with a selected style via the §6 accent-edge convention) OR WorkerSelect if a segmented component fits (WorkerSelect is in the index, exported index.ts:14) — prefer the simplest that matches Gallery/Harness look.
SEARCH BOX: a text input (base.css already styles input focus, base.css:48-53) bound to `q`; on Enter or a Search Button click, fetch /memory/search?q=<q>&project=<scope>. Show the `mode` ("grep" | "semantic+grep") as a small mono caption next to the result count (e.g. `8 hits · grep`). Searching switches the list from timeline-rows to hit-rows; a Clear control returns to the timeline. Do NOT fetch per keystroke.
EVENT ROW (the list item; LEAN, ONE line + optional depth):
  [square StatusMark by event_type §2] [event_type — Hanken label] [project_id · session_id ?? '—' — mono caption] [relative timestamp — mono caption, right] [a quiet "?" -> popover with the parsed payload_json]
  - event_type is the Hanken NAME (not mono). project_id / session_id / timestamp / event_seq are mono DATA (B-03).
  - relative timestamp: compute from created_at (e.g. "3h ago", "2d ago"); also expose the absolute ISO in the popover. Keep the formatter a tiny pure helper.
  - PAYLOAD DEPTH (§7 click-? / low-text-with-depth-on-demand): the row stays low-text; the "?" popover shows the JSON.parse(payload_json) pretty body (mono), fail-soft to the raw payload_json string if parse fails. For a handoff event the payload has {entry_date, source_file, summary, text} — show summary in the popover, NOT inline (keeps the list LEAN, N-03).
  - key the {#each} on event_seq (stable, unique).
SEARCH-HIT ROW: [StatusMark by event_type] [event_type] [source (mono: event:<seq> | handoff:<file>:<line>)] [project (mono, may be empty)] [score (mono)] [text snippet — Hanken body, truncate]. The hit has no payload_json; the .text is the body.
LEANNESS CHECK (N-03): a LIST of rows + sections, NO graph/3D/canvas/chart. The only "chart" the design law allows is a plain segmented bar where a real number must be read (§9) — NOT needed here. No timeline-as-visualization; it is a scrollable log.

==============================================================================
7. AUDIT CRITERIA (the done-gate — wiring-enforcement: real seam, real data, fail-closed)
==============================================================================
A. REAL DATA THROUGH THE REAL SEAM (load-bearing, R4/wiring): the surface renders REAL /memory/* data, never fabricated. PROOF: with the slice_api.py HTTP server up on :8787 (or the Tauri native command wired), the timeline shows a REAL indexed event — specifically project=teikoku returns event_seq 3507 (a real "handoff" row) + 4 more (VERIFIED live this session). The seam must surface the offline/error state on failure, never a stub row (mirror harness.ts comment).
B. PER-PROJECT + GLOBAL BOTH WORK: selecting a project (e.g. teikoku) hits /memory/timeline?project=teikoku and shows that project's rows; Global hits /memory/global and shows cross-project rows + populates the project selector from .projects[] (17 projects live this session).
C. SEARCH RETURNS HITS: a query (e.g. "memory") hits /memory/search and renders the .hits[] (8 hits live this session), showing the `mode`. grep is the unconditional floor; semantic is additive (mode flips to "semantic+grep" when memory_semantic loads).
D. SIGNAL DESIGN PASS — ALL ban items absent (DESIGN-TEIKOKU.md §1 B-01..B-11): no center-out underline (B-01); no glow/shadow (B-02, base.css enforces box-shadow:none); mono ONLY on data, never labels/body (B-03); no all-caps tracked micro-labels / dot-prefix (B-04); no cosmic intro (B-05); ONE teal accent, no purple/blue/red/orange, fail=light-inversion-chip never red (B-06/B-07); no grid-of-identical-cards, radius exactly 2px, no gradients/glass (B-08); Hanken+JetBrains only, no Inter/Lucide (B-09); SQUARE status marks not circle+check (B-10); high contrast bright-on-near-black (B-11).
E. CROSS-SCREEN CONSISTENCY (C-01/C-07): Memory.svelte uses the SAME components (index.ts imports), SAME tokens, SAME row/header/empty/loading/error patterns as Harness.svelte + Gallery.svelte + the shell. A value not in tokens.css, or a re-implemented component, is a defect. Diff Memory's header/skeleton/empty-tile against Harness's — they must be visually of-a-piece.
F. STATE COVERAGE (§10): empty / empty-search / loading / error(=light-inversion or warn, NEVER red) / offline / populated all designed and reachable.
G. N-03 LEAN: a LIST, not a viz. No graph/3D/canvas. N-06: no brand/marketing chrome on the surface (the wordmark lives only in the topbar Chrome).
H. WIRING (D-030 / production-caller-gate): the new api/memory.ts exported fns MUST be CALLED from Memory.svelte (the consumer), and Memory.svelte MUST be mounted in App.svelte (the production entrypoint) IN THE SAME CHANGESET. A producer (memory.ts fn) with no consumer = wire-dark = BLOCK. This is a Svelte/TS changeset, so the .rs/.py wire-checkers do not apply, but the R6/R7 principle does: the auditor confirms Memory is reachable from the IconRail click path. No new pub Python/Rust symbols are added (backend is reused), so no python_wire_check/production_caller_check needed unless the executor adds a new Tauri #[command] (then production_caller_check.py applies to that .rs).
I. CDP CAPTURE METHOD (the design-pass screenshot, visual-ui-loop.md pinning): mirror screenshots/cap_harness.mjs EXACTLY — it already does deviceScaleFactor:1, fromSurface:true, prefers-reduced-motion via setEmulatedMedia, settle sleeps, blur active element. cap_harness clicks `button[aria-label="Harness"]` to switch the view (the view is component state, not a URL); cap_memory.mjs clicks `button[aria-label="Memory"]` (the RailItem label drives the aria-label). Dev server on http://localhost:1420 (VERIFIED vite.config port=1420), Chrome on --remote-debugging-port=9222 (--font-render-hinting=none set by the launcher). Run: `bun cap_memory.mjs screenshots/memory.png`. Pixel-diff only vs a prior accepted memory render; semantic critique vs the design law for first capture (visual-ui-loop §pixel-diff-only-against-known-good).

==============================================================================
8. MERGE/CONSISTENCY GOTCHAS (read before editing)
==============================================================================
- payload_json is a STRING in the timeline rows (NOT pre-parsed). The search HIT shape already flattens prose into .text (no payload_json). Two different shapes — handle separately.
- event_type is OPEN/mixed-case; classify by regex into §6 states, never enumerate. Do NOT add a frozen enum (the backend deliberately dropped the CHECK-enum/SCHEMA_VERSION treadmill, memory_store.py:8-13).
- project_id can be NULL on legacy rows; session_id is often null — render '—' (mono), never crash.
- /memory/timeline with NO project still returns rows filtered by None (it is the per-project route); use /memory/global for the true cross-project view (it also returns the project roster). Do not conflate.
- harness.ts is the EXACT structural template for memory.ts (two-transport, JSON.parse native, fetch web, never-fabricate). Copy its skeleton; change only the command names, routes, and types.
- The `.galleryhost` + `.skel`/`.skel.short` + `.dim`/`.empty-line` classes already exist (App.svelte:275/328-335, Harness.svelte:298-307); reuse the exact same local style names so the two screens stay of-a-piece.

==============================================================================
9. SOURCES CONSULTED (all read this session; distro = WSL `teikoku`)
==============================================================================
signal/src/lib/screens/Harness.svelte (P1-C surface pattern, full)
signal/src/lib/api/harness.ts (two-transport seam, full)
signal/src/lib/api/slice.ts (the original seam template harness.ts mirrors)
signal/src/App.svelte (mount pattern, View union, railItems, view-switch host)
signal/src/lib/components/index.ts (the reuse surface)
signal/src/lib/components/{StatusMark,Tile,Button,QMark,Popover,SessionRow}.svelte (props/signatures)
signal/src/lib/styles/tokens.css (the token set, full) + signal/src/lib/styles/base.css (global .mono, focus, reduced-motion)
signal/screenshots/{cap_harness.mjs,cdp_capture.mjs} (CDP capture + pinning)
conductor/slice_api.py (the /memory/* HTTP routes + native CLI mirror, full)
conductor/memory_store.py (memory_search/recent/grep, full) + conductor/conductor_db.py (recent_events/query_events/append_event row shape)
conductor/memory_server.py (MCP tool names confirmed; dashboard uses HTTP not MCP)
LIVE captures (real JSON, via `python3 slice_api.py memory-global|memory-timeline|memory-search`): global roster+events, teikoku 5-row timeline incl handoff event_seq 3507, search q=memory 8 hits mode=grep, event_type counts.
F:\.ELAI_workflow\workflow\design\DESIGN-TEIKOKU.md (the SIGNAL law: bans, palette, type, §6 state, §7 ?, §10 coverage)

==============================================================================
10. UNKNOWNS (verify at execute time)
==============================================================================
- The HTTP server (slice_api.py on :8787) was NOT RUNNING at compile time (curl returned empty). The executor/auditor must start it for the live design-pass + audit: `cd ~/teikoku/conductor && python3 slice_api.py` (bare invocation runs the ThreadingHTTPServer, slice_api.py:208-210). The CLI one-shot mode reads the SAME control.db and confirmed the data is real — so the routes WILL return it once the server is up. (This is the only blocker to a live capture; the data and routes are verified.)
- TAURI NATIVE memory commands: slice_api.py has the python one-shot CLI (memory-timeline/global/search), but whether a Rust #[tauri::command] (e.g. signal_memory_timeline) wraps it like signal_latest_slice/run_slice is UNVERIFIED — not read this session. For the vite/web design pass it is IRRELEVANT (the fetch path is used); for the desktop build the executor may need to add the Rust commands (then production_caller_check.py applies). Recommend the executor grep signal/src-tauri for `signal_memory` / `memory` #[command]s before assuming.
- Whether a segmented WorkerSelect component cleanly fits the scope selector vs a Button group — executor's call at build (both are in the index; pick the one that matches Gallery/Harness look; surface to controller if neither fits).
- memory_semantic.py presence on the box (mode was "grep" at capture => semantic leg absent or not loaded). The grep floor is unconditional so search works regardless; the surface must just show whichever `mode` the route returns. Not a blocker.

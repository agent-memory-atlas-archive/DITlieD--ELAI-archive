CONTEXT PACKAGE — TEIKOKU P1-C (sub-phase 1.5 dashboard harness/pack surface)
Task: build the dashboard "harness/pack view" surface (which capability packs are active per project; merge/cut status), composed from the EXISTING Phase-0.5 SIGNAL design system, passing the SIGNAL design pass + the cross-screen CONSISTENCY check.
Drive: controller-direct, C-02 cycle (context-compiler -> executor -> auditor). This package is for the EXECUTOR. Distro = WSL `teikoku`, project root = /home/user/teikoku. SIGNAL app = /home/user/teikoku/signal/.
1.5 done-def (master plan line 150-151): "surface renders + design pass." N-03: LEAN (sidebar+sessions+chat+monitoring+per-phase surfaces; NO canvas/3D/graph-viz). N-06: no brand invention beyond DESIGN-TEIKOKU. C-01: cross-screen consistency (same tokens + shared components, zero per-screen drift) is part of done.

================================================================================
SECTION 1 — THE SIGNAL LAW, DISTILLED TO AN ACTIONABLE CHECKLIST FOR THIS SURFACE
(source: workflow/design/DESIGN-TEIKOKU.md v2 LOCKED; the surface is judged against this at audit)
================================================================================

1A EXACT TOKENS TO USE (from signal/src/lib/styles/tokens.css — these are the ONLY legal values; a literal hex or magic px in your component = audit BLOCK):
  color    : var(--ground) app bg | var(--sink) deepest inset/meter tracks | var(--surface) raised panel/tile | var(--surface-2) hover/selected lift | var(--text) primary/contrast carrier | var(--text-2) secondary reading | var(--dim) captions/muted | var(--accent) THE teal (interactive/active/selected keyline/PASS, SOLID never glow) | var(--accent-deep) deep fill (segments/done/progress) | var(--accent-hi) hover lift on solid accent | var(--line) hairline divider | var(--line-2) stronger divider/control border.
  spacing  : var(--s-1)=4 var(--s-2)=8 var(--s-3)=12 var(--s-4)=16 var(--s-5)=24 var(--s-6)=32 var(--s-7)=48. 8px rhythm. NO raw px for spacing.
  radius   : var(--r) = 2px. EVERYWHERE radius is used. Never another value, never rounded-everything.
  border   : var(--bw) = 1px solid var(--line) | var(--line-2). Elevation = ground->surface->surface-2 lightness ladder + a 1px line; NEVER a shadow.
  type     : var(--font-ui) = Hanken Grotesk (ALL labels/body/buttons/titles/chrome). var(--font-data) = JetBrains Mono (NUMERIC DATA / code / counts / timestamps / status codes ONLY — pack counts, version strings, dates render mono; pack NAMES and human labels render Hanken). sizes: --fz-title 16 / --fz-body 13.5 / --fz-label 12.5 / --fz-caption 10.5 / --fz-data 12. weights: --fw-title 650 / --fw-body 450 / --fw-label 500 / --fw-caption 400. lh: --lh-body 1.5 / --lh-tight 1.25. The base.css `.mono` class already applies font-data + tabular-nums — use it for data spans.
  motion   : var(--dur-fast)=120ms / var(--dur-slow)=160ms / var(--ease)=cubic-bezier(0.2,0,0,1) ease-out. prefers-reduced-motion handled globally in base.css (collapses to static).

1B THE SQUARE StatusMark (§6) — THIS SURFACE'S PRIMARY STATE LANGUAGE. Import StatusMark from the index; do NOT draw your own marks. States: live (solid --accent 9px square + one calm opacity pulse) | done (recedes, --accent-deep bar, --dim text — NOT a checkmark) | pending (1px --line square OUTLINE, --dim text, no fill) | warn (hollow --text triangle, rare) | fail (LIGHT INVERSION chip: --text bg, --ground text — the loudest thing, NEVER red). MAP for this surface: an ACTIVE pack = StatusMark state="live" (or a steady "done"-style filled mark if you want no pulse on a static registry; prefer a NON-pulsing active treatment — see 1F note); an INACTIVE pack = state="pending" (outline square, dim). A CUT item in the merge summary = state="fail" inversion chip ONLY if you want it loud; cut is better shown as a plain dim "cut" row with a pending/struck treatment (cut is a settled fact, not a failure to scream). Doctrine: silence = healthy; NO mark on rows that need none. Progress (if you show pack counts as a bar) = explicit n/m on a plain segmented bar (--accent-deep fill on --sink), never a lone % and never a glowing bar.

1C THE SLOP BAN LIST AS AUDIT ITEMS (each is a BLOCK if it survives — these are the design-pass schema items, pass|fail|unknown, unknown=fail):
  B-01 NO center-out/grow-from-center underline on any interactive element. Hover/selected = SOLID change (fill, block, or square accent edge). (Reuse the existing components — they already comply; if you add a hover/selected affordance, it is a solid fill or a 2px accent left edge, never a drawn line.)
  B-02 NO glow/bloom/halo/neon/drop-shadow. box-shadow:none is enforced globally in base.css; do not override it. Elevation = lightness step + 1px line.
  B-03 NO monospace as default UI/body/label face. Mono ONLY for the numeric counts / version strings / dates in this view. Pack names + section titles + descriptions = Hanken.
  B-04 NO ALL-CAPS .2em-tracked micro-labels; NO dot-prefix / number-prefix labels. Labels = normal-case Hanken medium.
  B-05 NO cosmic intro / starfield / staggered reveal. The surface renders immediately.
  B-06 NO purple/indigo/blue/red/orange/amber. ONE accent = teal --accent. fail = light-inversion chip, NEVER red.
  B-07 NO multi-hue status badge/pill spam. Status = square shape + weight + at most one accent mark + the inversion chip.
  B-08 NO grid-of-identical-cards as the layout; NO rounded-everything (radius exactly 2px); NO gradients (bg or text); NO glass/blur. (A per-project block listing its packs is fine; a uniform card grid of N identical tiles is the banned tell — differentiate by content/hierarchy, use Tile sparingly with a real focal element.)
  B-09 NO Inter/Roboto/Arial/system-ui/Geist/Space Grotesk; NO Lucide/Feather/Material icon set. Glyphs = CSS shapes + drawn unicode (the IconRail/marks already do this).
  B-10 NO circle+check status dots. SQUARES/bars/weight/inverted chips (StatusMark enforces this — use it, don't reinvent).
  B-11 NO low contrast. Bright --text on near-black --ground always. Contrast gate at audit: >=4.5:1 body, >=3:1 large/UI over --ground.

1D STATE COVERAGE (§10 — the surface is NOT done until every state is designed; each is an audit item):
  empty   — a focal next action (e.g. "no projects registered — register one"), NOT a bare "no data". (For day-1 there IS one project, elai, so empty is the no-backend / parse-empty case.)
  loading — calm --surface skeleton, nothing rendered under ~200ms. (Reuse the .skel pattern in App.svelte / Tile loading branch.)
  error   — inversion chip + a recovery affordance (Retry button), NEVER red. (Mirror App.svelte's sliceError -> warn Tile + Retry branch.)
  offline / disconnected — backend unreachable: warn triangle + retry (this surface reads a backend manifest, so the backend-unavailable path is mandatory, same shape as App.svelte's `sliceError && !slice` branch).
  populated — the real harness data (the normal case).
  (stale/cached is optional here — the manifest is read fresh each mount; if you cache, add a stale treatment.)

1E THE "?" DETAIL AFFORDANCE (§7) — depth-on-demand, keep the surface LOW-TEXT. Import QMark from the index. Use it to explain non-obvious things WITHOUT cluttering the surface: e.g. a "?" next to a pack name opens a Popover listing what that pack contributes (its skills/agents/rules) or what "active vs inactive" means; a "?" on the merge/cut summary explains WHY items were cut. The "?" is quiet --dim Hanken (no badge/circle/glow), --text on hover; click opens the Popover (--surface card, 1px --line-2, 2px radius, fade+rise). Do NOT dump all detail on the surface — put it behind the "?".

1F LAYOUT DISCIPLINE: no glow, matte surfaces, 1px lines, 2px radius, 8px rhythm, ONE clear focal element per pane. ONE accent. NOTE on the live-square pulse: §6 allows ONE calm pulse for genuinely LIVE/running things. A pack REGISTRY is static config, not a running process — prefer a non-pulsing "active" treatment (a solid --accent square via a steady mark, or the SessionRow selected-style left edge) so the surface is "still" (the §0 one-line: "alive, still"). Reserve the pulse for actual live runs (which this surface does not show). If you reuse StatusMark state="live" it WILL pulse; for a static active-pack mark, use a filled square without the live state (e.g. a styled span using --accent — but that risks re-implementing a mark; cleaner: use StatusMark state="done" for active [filled --accent-deep, reads as "present/settled"] and state="pending" for inactive [outline, dim]). Decide and keep it consistent across all projects.

================================================================================
SECTION 2 — EXISTING COMPONENTS + TOKENS TO REUSE (import surface + the hard no-reimplement rule)
================================================================================

HARD RULE (C-01 / LAW §2, audited): import EVERY visual primitive from the single index. Do NOT re-implement any of these, do NOT introduce a new hex value, a new px spacing/size, a new radius, or a new font. A value not in tokens.css OR a re-implemented component = a consistency DEFECT caught at audit (this is exactly what G5 checked for in 0.5: "both surfaces import the identical token file + index; zero hex/magic-px in components"). The Gallery (surface #1) and App.svelte shell (surface #2) are the precedent — your harness surface is surface #3 and MUST share the identical token file + index.

IMPORT PATH (single surface): `import { ... } from '$lib/components/index.ts';`
Full export list (signal/src/lib/components/index.ts):
  Button (variant 'primary'|'secondary', disabled, type, title, onclick, children) — solid accent / outline secondary.
  Toggle (bind:checked, label) — the Bypass-style switch.
  StatusMark (state 'live'|'done'|'pending'|'warn'|'fail', label?) — THE square state mark. USE THIS for pack active/inactive.
  Chrome () — wordmark TEIKOKU + 帝国 (topbar only; the shell already renders it — your surface mounts INSIDE the shell, so you do NOT re-render Chrome).
  Popover (bind:open, align 'left'|'right', onclose?, children) — the depth card.
  QMark (children) — the quiet "?" that wraps a Popover. USE for depth-on-demand.
  Menu (trigger, title, items: MenuItem[], align?) + type MenuItem — dropdown.
  WorkerSelect (segmented) — likely not needed here.
  SessionRow (name, meta?, state, selected?, onselect?) — a left-edge-selectable row with an embedded StatusMark. STRONG REUSE CANDIDATE for a pack row (name=pack, meta=mono counts, state=active/inactive, selected on click).
  Tile (title, state?, fail?, children) — --surface card with title + optional StatusMark + body (mono data lines). REUSE for a per-project block or the merge/cut summary, but heed B-08 (don't make N identical tiles a grid; give each a real focal hierarchy).
  IconRail (items: RailItem[]) + type RailItem {id,glyph,label,active?,onactivate?} — the ~48px left rail. The SHELL owns the rail; you ADD a rail item to open your view (see Section 4), you do NOT re-instantiate IconRail.
  Panel (title, side 'left'|'right', width, open?, header?, children) — collapsible panel; the shell composes these.
  PanelManager (left?, workspace, right?) — the shell's left/workspace/right composition with drag-dividers.
  TokenMeter (ctxUsed, ctxCap, sessionTokens) — composer meter; not needed here.
  UsagePopup (bind:open, sections: UsageSection[]) + types — provider usage; not needed here.
  Composer (ctxUsed,ctxCap,sessionTokens,running,fileContext?,onsend?,onstop?,oncommand?) — the chat composer; not needed in the harness view itself.

base.css gives you: `.mono` class (font-data + tabular-nums) for any numeric/version/date span; global box-shadow:none; focus-visible accent outline; reduced-motion collapse. Reuse `.mono`, do not redefine it.

DO-NOT list (explicit): do NOT add a <style> hex literal; do NOT hardcode 264px/300px/48px (use --panel-* / --rail-w); do NOT import a new font; do NOT add a new icon library; do NOT re-create StatusMark/SessionRow/Tile/QMark/Popover; do NOT override box-shadow.

================================================================================
SECTION 3 — THE HARNESS-STATE DATA MODEL + THE BACKEND THAT PRODUCES IT (mirror the 0.5 slice seam)
================================================================================

PRINCIPLE (wiring-enforcement / 0.5 precedent): the Svelte surface does NOT hardcode the harness data. A small READER backend parses projects.toml + enumerates packs/*/.claude + emits a JSON manifest; the surface reads it through an api seam exactly like slice.ts. The slice path already proved this two-transport pattern (Tauri native invoke + http :8787 dev fallback, both reading the same source, neither fabricating).

3A THE REAL HARNESS STATE TO RENDER (verified from the distro this session):
  projects.toml (the ONE registry, /home/user/teikoku/projects.toml) day-1 has ONE project:
    id="elai", path=/home/user/teikoku/projects/elai_cc, platform="rust+ts", packs=["rust","elai"], done="wiring/tests+dashboard", lanes=["all"], vcs="git".
    (The schema supports more projects; the field set is id/path/platform/packs/done/lanes/vcs. done is a HANDLE that resolves to the active-pack gate set — for elai it resolves to the rust+elai pack gate checklists.)
  Pack dirs (/home/user/teikoku/packs/<pack>/.claude) and their contents (VERIFIED counts this session):
    rust    : skills=[impact-check] agents=[test-attributer, wiring-contract-validator] rules=[production-caller-gate, tracing-test-assert] rulebook=[codegraph-vs-map] + gate-checklist.md + mcp.json
    elai    : skills=[b-plan-runner, dashboard-handoff, paper-to-plan, sidecar-protocol] agents=[adversary, dashboard-smoker, sidecar-health, spec-grounder] (no rules/rulebook dir)
    unity   : skills=[unity-scaffold] rules=[verify-before-write] rulebook=[10 files] + gate-checklist.md + mcp.json
    media   : skills=[banana-pro-director-2.0, cinema-worldbuilder-pro-2.0, region-maps, strip-ai-footprint] rulebook=[4 files] + mcp.json
    android : rulebook=[android-cli]
    notion  : rulebook=[notion-ticket-format]
  ACTIVE for elai = rust + elai. INACTIVE (registered but not active for any day-1 project) = unity, media, android, notion.
  CORE .claude (at git root, shared by all projects — NOT a pack): skills=14, agents=22, rules=16, rulebook=19, tools=46, commands=2, compose.ts. (Render this as a "core" block distinct from packs: core auto-loads for every project; packs are per-project.)
  MERGE/CUT SUMMARY (from P1-A/P1-B OUTCOMEs in the handoff — the audit merge result to surface):
    COLLAPSED: the two -elai agent duplicates (scope-planner-elai, integration-explorer-elai) each collapsed to ONE parameterized agent (DEFAULT branch + PACK branch, union-preserving). The 3 "-elai" dups family. synthesizer-elai + designer were CUT.
    CUT (5 inert MCP servers): ae x3, unrealclaude, vibeue; plus ai-game-developer agent cut. comfyui-cwags rewired npx->bunx.
    KEPT cross-boundary: gpt-bridge + codegraph stay Windows/node (headed Chrome / vendored Node bundle); canvas MOVED to run-in-WSL-via-bun.
    INACTIVE-but-registered: android + notion packs kept as INACTIVE (reversible; day-1 ELAI-only per N-04).

3B THE DATA MODEL the surface SHOWS (define this as the manifest JSON shape the reader emits + the Svelte consumes):
  HarnessManifest = {
    ok: boolean,
    error?: string,
    core: { skills: number, agents: number, rules: number, rulebook: number },   // core counts
    projects: Project[],
    merge: MergeSummary
  }
  Project = {
    id: string, platform: string, path: string, done: string, lanes: string[],
    packs: PackStatus[]            // EVERY known pack, with active flag per this project
  }
  PackStatus = {
    name: string,                  // rust | elai | unity | media | android | notion
    active: boolean,               // is this pack in the project's active packs list
    skills: number, agents: number, rules: number, rulebook: number   // per-pack counts (enumerate packs/<name>/.claude/<subdir>)
  }
  MergeSummary = {
    collapsed: string[],           // e.g. ["scope-planner-elai+integration-explorer-elai -> parameterized"]
    cut: string[],                 // e.g. ["ae(x3) MCP","unrealclaude MCP","vibeue MCP","ai-game-developer","synthesizer-elai","designer"]
    rewired: string[],             // e.g. ["comfyui-cwags npx->bunx","canvas node->bun(WSL)"]
    kept_cross_boundary: string[]  // e.g. ["gpt-bridge (Windows/node)","codegraph (Windows/node)"]
  }
  Render mapping: per project -> a block titled by id (Hanken title) + a mono meta line (platform · done · lanes); under it, EACH pack as a SessionRow-style row: name (Hanken), counts (mono "S2 A2 R2 RB1" style), StatusMark state=done(active)/pending(inactive). A separate "core" block at top (the always-loaded base). A merge/cut summary block (Tile or a plain section) with the four lists, each item a dim row, depth behind a QMark "?".

3C THE RECOMMENDED BACKEND (mirror conductor/slice_api.py + slice_driver.py EXACTLY):
  ADD a reader at /home/user/teikoku/conductor/harness_api.py (a sibling of slice_api.py). It:
    - parses /home/user/teikoku/projects.toml (stdlib tomllib in py3.11+; VERIFY the distro python is >=3.11, else use the tomli dep already vendored or fall back — anti-slopsquat: check before importing),
    - enumerates /home/user/teikoku/packs/*/.claude/{skills,agents,rules,rulebook} counting entries (dirs for skills [each skill is a dir], .md files for agents/rules/rulebook),
    - counts the core /home/user/teikoku/.claude/{skills,agents,rules,rulebook},
    - hard-codes the merge/cut summary lists from the P1-A/P1-B audit result (these are facts of the merge, not enumerable from the live tree since the cut items no longer exist — so they are static data IN the reader, sourced from the handoff OUTCOMEs; that is legitimate, it is the merge RECORD not fabricated UI data),
    - serves GET /harness/manifest on 127.0.0.1:8787 (SAME port as slice_api — EITHER extend slice_api.py with a /harness/manifest route [cleaner: one dev server], OR a second server on a new port; PREFER extending slice_api.py's Handler with the new GET route so the dev transport stays one process. NOTE: extending slice_api.py grows that file — keep the new logic in harness_api.py and import it into slice_api.py's handler, so neither file balloons [maintainability MR-1/MR-3]).
    - one-shot CLI mode `python3 harness_api.py manifest` printing the JSON (so the Tauri native command can shell it, exactly like slice_api.py's latest|run pattern).
  THE TAURI SEAM (mirror lib.rs): add ONE #[tauri::command] `signal_harness_manifest()` that shells the reader one-shot CLI and returns its JSON string verbatim (copy the slice_api() helper shape: Command::new("python3").arg(script).arg("manifest"), env ANTHROPIC_API_KEY="" to keep plan-OAuth, success->stdout, error->JSON {ok:false,error}). REGISTER it in generate_handler![] alongside signal_health/signal_latest_slice/run_slice. THIS IS A PRODUCTION-WIRING DELIVERABLE (D-030 / preamble R-8): the new command must be in generate_handler! (its production caller) in the SAME changeset, and the Svelte must invoke it — not a dead pub fn.
  THE API SEAM (mirror slice.ts): add /home/user/teikoku/signal/src/lib/api/harness.ts exporting `fetchHarnessManifest(): Promise<HarnessManifest>` with the SAME two-transport shape as slice.ts (tryInvoke('signal_harness_manifest') native, else fetch HTTP_BASE+'/harness/manifest'); on failure return {ok:false,error} — NEVER fabricate. Define the HarnessManifest/Project/PackStatus/MergeSummary types here (mirror slice.ts's SliceEvent typing).

================================================================================
SECTION 4 — HOW THE SURFACE MOUNTS IN THE SHELL (reuse App.svelte composition; keep LEAN per N-03)
================================================================================

The shell (signal/src/App.svelte) already has a `view` state machine: type View = 'shell' | 'gallery'; toggled by an IconRail item (the 'gallery' rail item flips view). MIRROR THIS:
  - EXTEND View to 'shell' | 'gallery' | 'harness'.
  - ADD ONE RailItem to the railItems $derived array: { id:'harness', glyph:'▣' (or another CSS/unicode square glyph, NOT a Lucide icon), label:'Harness', active: view==='harness', onactivate: () => (view = view==='harness' ? 'shell' : 'harness') }.
  - ADD a branch in the topbar crumb + the body: `{:else if view === 'harness'}<div class="hosthost"><Harness /></div>` mirroring the gallery host div. The crumb text: 'harness · packs'.
  - The harness surface is a NEW screen file: signal/src/lib/screens/Harness.svelte (sibling of Gallery.svelte). It owns the fetch (calls fetchHarnessManifest in an $effect, like App.svelte's refreshSlice), holds loading/error/populated state, and renders the projects + packs + merge summary by COMPOSING the index components. It is a full-width screen (like Gallery), NOT a panel — it does not need the session/monitor panels (N-03 lean: it is a focused per-phase surface).
  - LEAN guard (N-03): NO graph viz, NO 3D, NO canvas, NO dependency-cruiser-style node graph. A scannable list/section layout of projects -> packs -> counts + a merge/cut summary. That is the whole surface.

Why a full-width screen and not a panel: the shell's panels (session/monitor) are for the live workspace; the harness view is an inspector surface like the Gallery, reached via its own rail toggle. This keeps the shell's existing composition intact (no new panel plumbing) and is the minimal, consistent mount.

================================================================================
SECTION 5 — AUDIT CRITERIA (what the auditor will re-verify off-agent; the executor self-checks these before DONE)
================================================================================

A1 SURFACE RENDERS WITH REAL HARNESS DATA: the auditor drives the real path (vite dev server + the harness reader on :8787, OR the one-shot CLI) and confirms the surface shows the REAL projects.toml/pack enumeration (elai project, rust+elai active, unity/media/android/notion inactive, real per-pack counts, the core counts, the merge/cut summary). NOT hardcoded in Svelte. A fabricated/hardcoded data path = BLOCK (wiring-enforcement: the surface reads a real manifest the agent did not handwrite into the component).
A2 PRODUCTION WIRING (D-030, preamble R-8): the new signal_harness_manifest #[tauri::command] is registered in generate_handler! (auditor re-runs production_caller_check.py over changed .rs; a DARK pub fn = BLOCK) AND invoked from harness.ts AND harness.ts is imported/used by Harness.svelte AND Harness is mounted in App.svelte. The whole chain producer->consumer ships in this ONE changeset. The python reader is wired into slice_api.py's HTTP handler (or its own served route) — not a dead module (python_wire_check semantics).
A3 SIGNAL DESIGN PASS (visual-ui-loop.md SEMANTIC critique vs the SIGNAL law, schema pass|fail|unknown, unknown=fail, evidence region mandatory): EVERY B-01..B-11 ban ABSENT; high contrast (>=4.5:1 body / >=3:1 UI over --ground, measured via ax-scaffold); square StatusMarks (no circles/checks); no glow/gradient/glass/shadow; Hanken UI + JetBrains-mono-for-DATA-ONLY (pack names Hanken, counts mono); radius exactly 2px; one teal accent; fail = inversion chip never red.
A4 CROSS-SCREEN CONSISTENCY (C-01, the load-bearing 1.5 criterion): Harness.svelte + harness.ts share the IDENTICAL token file (tokens.css) + the IDENTICAL component index as the shell and Gallery. ZERO per-screen drift: auditor greps the new component <style> for literal hex (#xxxxxx) and magic px (raw px not via a --token) — any hit = BLOCK. Auditor confirms StatusMark/SessionRow/Tile/QMark/Popover are IMPORTED, not re-implemented. (This is the exact G5 check from 0.5.)
A5 STATE COVERAGE: empty / loading (skeleton, nothing <200ms) / error (inversion chip + Retry, never red) / offline (warn + retry) / populated all present and design-compliant (the auditor can force the error/offline state by stopping the backend and re-capturing).
A6 BUILD GREEN: svelte-check 0 errors, vite build green, cargo build green (the new command compiles). Auditor runs these by its own commands (main model / executor self-run is necessary-not-sufficient).
A7 SCOPE GUARD: N-03 (no canvas/3D/graph viz), N-06 (no brand invention — square unicode/CSS glyphs only, no new logo), no scope creep into runtime/memory (that is P2).

CDP CAPTURE METHOD for the design pass (REUSE the existing harness, do NOT write a new one): signal/screenshots/cdp_capture.mjs — `bun cdp_capture.mjs <out.png> <url> [w] [h]`. It already pins deviceScaleFactor 1, fromSurface true, Emulation.setEmulatedMedia prefers-reduced-motion=reduce, blurs the caret, settles 2.5s. Chrome MUST be launched with --font-render-hinting=none (the launcher does this, per the file's header note) on --remote-debugging-port=9222, with the vite dev server on http://localhost:1420 (0.5 used 1420; cdp_capture.mjs defaults to 1420). Capture the harness view (you may need to navigate to it / set view='harness' before capture — 0.5 captured both shell and gallery; add a harness capture). Pixel-diff (ui-diff.mjs) is for golden-render REGRESSION only; for this NEW surface use the SEMANTIC critique vs the SIGNAL law, not a pixel metric.

================================================================================
SECTION 6 — UNKNOWNS / OPEN ITEMS (flag to controller; not blockers but the executor should resolve or note)
================================================================================
U1 DESIGN-PASS RUNTIME: 0.5 did NOT run the Tauri NATIVE window (WSLg/webkit); it used the VITE DEV SERVER via CDP (localhost:1420) for the design pass, and `cargo build` to prove the shell+commands compile. The harness design pass should follow the SAME approach: vite dev server via CDP for the visual pass; cargo build (not a live native window) to prove signal_harness_manifest compiles. The Tauri-native transport is thus compile-verified, not runtime-verified at this phase (acceptable, matches 0.5; the http/:8787 transport is the one actually exercised in the design pass). Note this in the audit so the native path is not claimed as runtime-proven.
U2 PYTHON VERSION for tomllib: confirm the distro python3 is >=3.11 (tomllib stdlib) before relying on it; if <3.11, the reader needs a toml parser (check what conductor already uses — conductor parses TOML elsewhere; reuse its method; anti-slopsquat before adding any new dep).
U3 PER-PACK COUNT SEMANTICS: skills are DIRECTORIES (each skill = a dir with SKILL.md); agents/rules/rulebook are .md FILES. The reader must count dirs for skills, files for the rest, and tolerate a missing subdir (elai has no rules/rulebook dir -> count 0, not an error). Verified pack contents are in 3A; the reader should ENUMERATE live (not hardcode the counts) so the surface stays correct as packs evolve — only the merge/cut summary is static record data.
U4 BACKEND SERVER LIFECYCLE: the :8787 dev server (slice_api.py) must be running for the web-transport design pass; long-running WSL procs need `nohup setsid` reparenting (the 0.5 carry-note). If extending slice_api.py with the /harness/manifest route, the SAME server serves both — restart it after editing. The one-shot CLI mode (python3 harness_api.py manifest) needs no server and is what the Tauri command uses.

================================================================================
SOURCES CONSULTED (all read live this session)
================================================================================
workflow/design/DESIGN-TEIKOKU.md (full, SIGNAL v2 law)
workflow/handoff/teikoku-build.md (full — ORCHESTRATOR CONTRACT, 0.5 SHIPPED, P1-A/P1-B OUTCOMEs)
workflow/reasoning/TEIKOKU-MASTER-PLAN.md (grep: N-03, N-06, C-01, C-07, 1.5 done-def line 150-151, scope guard)
distro /home/user/teikoku/signal/src/lib/styles/tokens.css (full — exact token names/values)
distro .../signal/src/lib/styles/base.css (full — .mono, box-shadow:none, reduced-motion)
distro .../signal/src/lib/components/index.ts (full — the export surface)
distro .../signal/src/lib/components/{StatusMark,Tile,Panel,IconRail,SessionRow,QMark,Popover,PanelManager,Button,Composer}.svelte (full — prop signatures + style discipline)
distro .../signal/src/App.svelte (full — the shell composition + view state machine + slice fetch pattern + state-branch precedent)
distro .../signal/src/lib/api/slice.ts (full — the two-transport api seam pattern to mirror)
distro .../signal/src/lib/screens/Gallery.svelte (head — non-shell full-width screen mount precedent)
distro .../signal/src-tauri/src/lib.rs (full — #[tauri::command] + generate_handler! + shell-python pattern to mirror)
distro .../signal/screenshots/cdp_capture.mjs (full — the CDP design-pass harness to reuse)
distro .../conductor/slice_api.py (full — the reader/HTTP/one-shot-CLI backend pattern to mirror)
distro .../conductor/events.py (full — the control.db read pattern; harness reader does NOT need the DB, it reads toml+fs)
distro /home/user/teikoku/projects.toml (full — the ONE registry: elai project, fields, done handle)
distro /home/user/teikoku/packs/{rust,elai,unity,media,android,notion}/.claude/* (enumerated — per-pack skills/agents/rules/rulebook contents + counts)
distro /home/user/teikoku/.claude/{skills,agents,rules,rulebook} (counts: 14/22/16/19)
distro .../packs/rust/.claude/gate-checklist.md (head — confirms "done" handle resolves to pack gate sets)

VALIDATION WARNINGS
- PowerShell->wsl arg-mangling (known gotcha) strips $variables in bash loops; pack contents were enumerated via EXPLICIT paths + UNC Read, not loops — counts in 3A are VERIFIED by direct ls. UNC Glob over nested packs/**/.claude timed out (large trees); explicit-path ls used instead.
- U1: the Tauri NATIVE path is compile-verified only at this phase (0.5 precedent); the design pass exercises the vite/http transport. Do not claim native runtime-proven.
- U2: distro python3 version for tomllib UNVERIFIED — executor must check before importing tomllib.
- The merge/cut summary lists in 3A are sourced from the P1-A/P1-B handoff OUTCOMEs (the audit RECORD); the cut items no longer exist in the tree, so they are static record data in the reader, NOT live-enumerable — this is correct, not fabrication.
- elai pack has NO rules/ or rulebook/ dir (verified) — reader must treat a missing subdir as count 0, not an error.

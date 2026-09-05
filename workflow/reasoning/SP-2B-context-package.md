SP-2B CONTEXT PACKAGE
Task: TEIKOKU-ELAI-FINISH-PLAN.md unit 2.2 (lines 128-135) - canvas per-run PROVIDER PICKER.
session_id: s-finish-0705. Repo scope note: all facts below are from teikoku's OWN git repo at
/home/user/teikoku (HEAD=032da4b), a SEPARATE repo from F:\.ELAI_workflow. Read via
\\wsl.localhost\teikoku\home\teikoku\teikoku\... (distro "teikoku", Running) and PowerShell wsl.

(a) CURRENT SHAPES (file:line)
walk_control.py (conductor/walk_control.py, 89 lines total):
- walk_start() -> dict, ZERO params (line 49). Idempotent: if _driver_pids() truthy, returns
  {ok:True,running:True,already:True} (53-54) without inspecting any argument. Else unlinks
  STOP_SENTINEL (56), Popen(["bash", str(DRIVER)], cwd=CONDUCTOR_DIR, stdout/stderr=WALK_LOG
  append, stdin=DEVNULL, start_new_session=True) (62-70). DRIVER is hardcoded
  CONDUCTOR_DIR/"elai_walk_driver.sh" (18) - no config/provider arg threaded, no env= passed.
- walk_stop() (76): writes STOP_SENTINEL text, graceful (driver checks between passes).
- walk_status() (37): returns {ok,running,pids,stop_pending} only - NO provider field today.
slice_api.py (1441 lines; HOST,PORT="127.0.0.1",8787 at line 31):
- POST /walk/start (1263-1265) and /walk/stop (1266-1268): dispatch calls
  `walk_control.walk_start()` / `walk_control.walk_stop()` directly, WITHOUT reading the
  request body. Contrast /headless/run (1257-1259), /spawn (1254-1256), /projects/register
  (1260-1262): all call `payload = self._read_json_body()` first - that is the established
  read-body idiom this unit must adopt for /walk/start.
- headless_run(payload) (302-328): parses free_text/template/project_id/caps/drain, calls
  `headless_runner.run_headless(template_id=, free_text=, project_id=, db_path=DB_PATH,
  config_path=SPAWN_CONFIG_PATH, caps=, drain=)` (318-321). SPAWN_CONFIG_PATH = CONDUCTOR/
  "config"/"conductor.toml.example" (90) - a THIRD config, distinct from both elai lane
  configs. headless_runner.run_headless (headless_runner.py:388) already has a required
  keyword `config_path` param - config-parameterized already, provider concept is new.
orchestration_api.py (583 lines):
- build_orchestration_snapshot() (502) assembles the /orchestration response incl
  `"walk": _walk_status()` (557). _walk_status() (491-499) imports walk_control, calls
  walk_status(), returns only {available, running, stop_pending} - drops any other key
  walk_status() might carry; must be extended to pass "provider" through.
signal/src/lib/api/orchestration.ts (298 lines):
- WalkStatus type (182-187) = {available, running?, stop_pending?, reason?} - no provider.
- OrchestrationSnapshot.walk?: WalkStatus (192). HTTP_BASE = 'http://127.0.0.1:8787' (209),
  matches slice_api.py's bound host/port exactly.
- _walkAction(action) (262-270): `fetch(\`${HTTP_BASE}/walk/${action}\`, {method:'POST'})` -
  sends NO body today. startWalk/stopWalk exported (271-272) with zero args.
signal/src/lib/screens/CanvasMonitor.svelte (986 lines): see (c).

(b) PROVIDER-PARAM THREADING DESIGN FACTS
Two lane configs already exist as the mechanical "slot set" the plan means:
- config/conductor.elai-walk.toml (117 lines): FREE lane, 4x [[workers]] devin-free slots,
  pool="free", model="glm-5-2" (lines 72/82/91/100) - STILL glm-5-2, NOT yet swapped to
  swe-1-6 by wave 2.1b (see risks). worker_cap=2 (33).
- config/conductor.elai-opus.toml (93 lines): PAID lane, opus-exec/opus-wire/opus-audit +
  sonnet-rest slots, harness="claude", account_id="claude-elai", roles=["EXECUTE"] /
  ["WIRE","INTEGRATE"] / ["AUDIT"] / ["POLISH"] respectively (55-92). worker_cap=4.
  "Role affinity" the plan names is ALREADY fully expressed by each slot's roles=[...] field
  here - nothing new to build for that; only the config-SELECTION layer is new.
elai_walk_driver.sh (32 lines) hardcodes `CFG=config/conductor.elai-walk.toml` (line 13) and
runs `python3 conductor.py run-backlog --project elai --config "$CFG" --db "$DB"
--limit-per-wave 2` (22). conductor.py's run-backlog subparser (conductor.py:820-829) already
accepts an arbitrary `--config` path - the CLI plumbing is generic already; only the shell
script's hardcoded CFG line needs to become parametric (e.g. `CFG="${1:-config/
conductor.elai-walk.toml}"`) and walk_control.walk_start() needs to pass the resolved path as
an argv element into its Popen call.
retier_elai.py (33 lines) is the other half already DOCUMENTED as a manual 2-step recipe in
conductor.elai-opus.toml's own header (lines 9-11): "python3 retier_elai.py --tier PAID" then
"nohup bash elai_walk_driver.sh (with CFG pointed at this file)". Its core is
`UPDATE tasks SET model_tier=? WHERE status!='DONE' AND role IN (EXECUTE,WIRE,INTEGRATE,
AUDIT,POLISH)` (21-23). It is a `__main__`-only script (no importable function) - call it via
subprocess.run(["python3","retier_elai.py","--tier",tier], cwd=CONDUCTOR_DIR), matching the
driver's own subprocess idiom, rather than refactoring it into an import (smaller blast
radius for this unit).
walk_start(provider) must therefore: resolve provider -> (config_path, tier) -> retier the DB
-> THEN Popen the driver with that config path. The provider->(config,tier) map should be
ONE shared table since BOTH walk_start() (walk_control.py) and headless_run() (slice_api.py,
which must swap off its hardcoded SPAWN_CONFIG_PATH) resolve the same provider string.
Candidate home: a constant dict inside walk_control.py, since slice_api.py already does
`import walk_control` lazily inside the /walk/start and /walk/stop handlers (1264/1267).
"claude-elai-max" is a NEW literal - verified via repo-wide grep, zero existing occurrences.
"devin-swe-1-6" maps to the ALREADY-LIVE model_pool.py roster entry
`PoolModel(POOL_DEVIN_FREE,"swe-1-6","devin-free",...,label="devin swe-1-6")`
(model_pool.py:125-126), used today for interactive chat only, not yet for the walk lane
config (that swap is wave 2.1b's job, not landed - see risks).
Status leg: walk_status() needs a "provider" key. No DB/process introspection today exposes
"which config launched the running driver"; consistent with this file's own idiom
(STOP_SENTINEL is a plain sentinel file, line 19), the direct fix is a second sentinel file
(e.g. PROVIDER_FILE = CONDUCTOR_DIR/"ELAI_WALK_PROVIDER") walk_start() writes and
walk_status() reads back. orchestration_api.py::_walk_status() (497) then adds
`"provider": s.get("provider")`; signal's WalkStatus type (orchestration.ts:182) gets a
`provider?: string` field.
NAMING COLLISION - do not confuse: conductor_loop.py already threads an UNRELATED
`WorkerProvider` (Protocol, worker_adapter.py:110-122, methods run()/cancel() - a
subprocess-execution DI seam for tests) through run_task_loop (conductor_loop.py:84-96) and
other functions (lines 92,238,533,654,670,728,756,783,820). This is NOT this feature's
provider string; naming a new symbol `WorkerProvider` would collide. Prefer e.g.
`WalkProvider` / plain `provider_id`.

(c) FRONTEND CURRENT CODE
CanvasMonitor.svelte imports `startWalk, stopWalk` from '$lib/api/orchestration.ts' (32-33)
and `Button, Toggle, QMark` from '$lib/components/index.ts' (25-29) - NOT importing
WorkerSelect yet. Walk state block (95-114): `walkBusy` $state bool; `walk =
$derived(snap?.walk)`; `toggleWalk()` calls stopWalk()/startWalk() with NO arguments;
`walkLabel` derived text ('start walk'/'stop walk'/'stopping'). Button markup (572-582) sits
in the `.bar-l` flex div next to the "Process" title + QMark tooltip, guarded by
`{#if walk?.available}`. `.wsq` span is the SIGNAL square status mark (9px, var(--mark-sq)),
filled var(--accent) when running (735-741).
CANONICAL REUSABLE COMPONENT FOUND: signal/src/lib/components/WorkerSelect.svelte (69 lines)
is exactly the "compact select" shape the plan wants - a segmented radiogroup (`bind:value`,
`options: string[]`, `disabled_options`, `onchange`), already used in 7 other surfaces
(App.svelte topbar 436/457, AskQuestion.svelte:100, GrillDialog.svelte:76,
NewSessionMenu.svelte:376/384, SessionSettings.svelte:159/175, ConfigPanel.svelte:148,
Gallery.svelte:95) and exported from components/index.ts:14. Its own default `options` prop
is `['Anthropic','Devin','Codex','Free APIs']` with `disabled_options` for in-progress
choices (WorkerSelect.svelte:3-7) - the same name+in-progress-disable shape needed for
`['claude-elai-max','devin-swe-1-6']`. Reusing WorkerSelect (vs a native `<select>`) is both
the maintainability-review "reuse existing helper" path and the only LAW-consistent visual
(segmented squares, accent-deep fill, never-red disabled) - a native `<select>` would be a
new, inconsistent control on this surface.
orchestration.ts `_walkAction()` (262) sends no body today:
`fetch(\`${HTTP_BASE}/walk/start\`, {method:'POST'})`. Needs a JSON body `{provider}` and a
typed `startWalk(provider?: string)`.

(d) DESIGN LAW CONSTRAINTS
No standalone design-law markdown doc exists anywhere in teikoku's repo (grepped docs/,
.claude/, signal/ root - zero hits). The law lives only as in-source comments, referenced as
"LAW §2/§9" (components/index.ts:2), and is structurally enforced by tokens.css: exactly ONE
accent hue (--accent: oklch teal, tokens.css:18, "solid never glow") and NO
red/danger/error/fail color token defined at all (grepped --red/--danger/--error/--fail -
zero matches) - "never red" is a token-inventory fact, not just a style guideline.
Explicit comment rules found: "fail = light-inversion chip, NEVER red (LAW §2)"
(CanvasMonitor.svelte:17); "SIGNAL law: square status mark, tokens only, never a red hue"
(CanvasMonitor.svelte:716); "Squares only (SIGNAL design law)" (ModelPicker.svelte:54);
disabled/in-progress options render "calm-disabled (light inversion, never red per SIGNAL
design law)" (WorkerSelect.svelte:5-6,:62). Selected state = SOLID var(--accent-deep) fill
(WorkerSelect.svelte:59, ModelPicker.svelte:88; "matches WorkerSelect.sel" convention noted
at Memory.svelte:408). "Dependency-free by design (SIGNAL LAW: no new dep...)"
(CanvasMonitor.svelte:10) reinforces reusing WorkerSelect over any new picker or dependency.
Zero literal hex / magic px anywhere audited (CanvasMonitor.svelte:23, ModelPicker.svelte:7).
A new provider control must use only tokens.css vars: --surface/--surface-2, --line/--line-2,
--accent/--accent-deep, --text/--text-2/--dim, --r, --bw, --s-1/2/3,
--fz-caption/label/data, --font-ui/--font-data, --mark-sq.

(e) BUILD/VERIFY + DEPLOYMENT
signal/package.json scripts: dev = `vite --port 1420 --strictPort`, build = `vite build`,
tauri:dev = `tauri dev`, tauri:build = `tauri build`, check = `svelte-check --tsconfig
./tsconfig.json`. Per bun-not-node.md run these via bun (`bun run check`, `bun run build`),
never npm/npx.
Deployment model: slice_api.py is a PURE JSON API server (HOST,PORT bound at line 31;
do_GET dispatch at 1147) - grepped for dist/static_dir/SimpleHTTPRequestHandler/index.html
serving, ZERO matches. It does NOT serve signal/dist. "The canvas" is the Tauri desktop app
itself (signal/src-tauri): FE changes reach the running UI by relaunching `tauri dev` (dev
loop, talks to HTTP_BASE directly) or rebuilding via `tauri build`. orchestration.ts's
TAURI_AVAILABLE check (`'__TAURI_INTERNALS__' in window`, line 211) picks native IPC
invoke() inside the real Tauri shell and falls back to plain fetch(HTTP_BASE) when run as a
bare web page. WAVE-2 gate's "svelte-check 0/0 + build green" (plan line 137) maps to
`bun run check` + `bun run build`.

(f) RISKS
MID-FLIGHT CLAIM NOT CURRENT AT READ TIME: `git status --short conductor/` (teikoku's own
repo, HEAD=032da4b "checkpoint the held conductor pile before the finish-plan engine
merge...") shows walk_control.py, conductor_loop.py, slice_api.py, orchestration_api.py ALL
CLEAN - no uncommitted diff. Only backlog_import.py + its test are modified, plus 2 untracked
files (precheck.py, test_precheck.py). Either SP-1C has not started touching these files yet,
or already committed under 032da4b; the content read here is the current COMMITTED state, not
an in-progress edit. Re-check git status immediately before editing in case new commits land
between this read and execution.
WAVE-2.1b DEPENDENCY NOT YET LANDED: conductor.elai-walk.toml (newest-touched file, mtime
2026-07-05 15:44) still has all 4 devin slots at model="glm-5-2" (toml lines 72/82/91/100),
NOT swapped to swe-1-6, and worker_cap=2 not 3 (toml:33). If 2.2 ships/tests before 2.1b
lands, selecting "devin-swe-1-6" in the picker actually launches glm-5-2 workers under the
free lane - the label would lie about what is running. Sequence 2.1b before 2.2's live smoke,
or have the FE read the option label from a live source rather than a hardcoded string.
PAID-LANE ACTIVATION GATE: conductor.elai-opus.toml's own header (lines 1-13) states the
.claude-elai account MUST be MAX (Opus 1M is Max-only) and the exact model ids are
UNCONFIRMED ("confirm the exact model ids your Max account serves"). Selecting
claude-elai-max on a non-Max account fails at the worker layer, not at the picker.
TEIKOKU_AUTONOMOUS=1 reaching claude-lane workers (wave 2.1c, plan lines 126-127) is a
separate, not-yet-confirmed prerequisite for the opus lane to run unattended at all.
NO RUNNING-PROVIDER-SWITCH SEMANTICS: walk_start() (walk_control.py:53-54) is idempotent
purely on "is the driver alive" - it returns already:true WITHOUT inspecting any provider
argument, so calling start with a NEW provider while the OLD provider's driver is still
running is a silent no-op today (the new provider is dropped, not queued, not erroring). The
picker's behavior while walk.running is true is undefined by the plan text and must be
decided (disable the picker while running vs auto stop-then-restart vs queue) before wiring
the FE onchange handler - a real fork, not something to silently assume.
TWO-CONSUMER WIRING SURFACE: the plan text explicitly includes "(+ /headless/run)".
headless_run() (slice_api.py:302-328) currently hardcodes config_path=SPAWN_CONFIG_PATH (a
THIRD toml, conductor.toml.example, slice_api.py:90) which is neither elai lane config.
Threading the provider param here means either adding a third branch to the provider->config
map or redefining SPAWN_CONFIG_PATH's role for headless runs specifically - a genuine design
decision, not a copy-paste of the walk_start plumbing.
PORT/CONFIG NOTE: both elai lane tomls declare [server] port=8766 (vestigial - slice_api.py
itself binds 8787, see (a)); do not confuse with the live API port.

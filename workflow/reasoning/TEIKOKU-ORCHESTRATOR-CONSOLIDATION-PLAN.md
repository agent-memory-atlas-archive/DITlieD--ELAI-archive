# TEIKOKU — Orchestrator Consolidation Plan (canvas = general launcher; ELAI walk = one preset)

name: TEIKOKU-ORCHESTRATOR-CONSOLIDATION
status: authored 2026-07-06 by the controller session, from the 6-handoff reconciliation + a live source read of the walk/launch seams
purpose: end the two-surface mess. Today the canvas carries only the ELAI-hardwired "start walk" control (two frozen lane presets, paid-only scope, --project elai), while the GENERAL launcher (any registered folder, model/skills/objective) is buried in the chat composer "…" → Run autonomous form. Master-plan D-17 says the DASHBOARD is the general launcher and D-04 says one engine / two front-ends. This plan makes the canvas THE single general launch console — project + worker-source/model + mode(walk|objective) + scope — with the ELAI backlog walk kept as one preset, and redirects the chat run-form onto the same backend so there is ONE launch path.

target root (ALL edits): the TEIKOKU distro, WSL, /home/user/teikoku (git repo github.com/maintainer/teikoku). NOT the F: ELAI repo. Do NOT edit /mnt/f except to READ context and APPEND to this plan / the handoff. Distro ops via PowerShell → wsl; edits over \\wsl.localhost are fine for .svelte/.py (strips +x on shell scripts — re-chmod any .sh you edit).

governing standard: this plan is authored to .claude/rules/plan-standard.md (wargame unit shape + 10-point gate). The orchestrating session MUST run the refine-before-execute pre-flight first:
  wsl -d teikoku -- python3 /home/user/teikoku/.claude/tools/plan_gate_check.py --file <this file copied into the distro, or the F: path over \\wsl.localhost>
  (the TEIKOKU distro shipped plan-standard + the checker per workflow/handoff/planning-skill-consolidation.md — 33 files locked). FAIL → refine, do not execute.

how another session runs this: hand this file to the TEIKOKU orchestrate / auto-orchestrate skill (or ELAI orchestrate pointing at the distro). Cycle per C-02: context-compiler → executor → auditor, MAX 2 parallel, agent-dispatch-preamble pointer on every dispatch. ONE commit at the very end (maintainer voice, no AI trailer), owner-hold until the owner says commit. Grounding for every seam named below is a live 2026-07-06 read; file:line anchors are load-bearing but RECON-confirmed by the executor before edit (they drift).

================================================================================
0. LOCKED DECISIONS (owner, 2026-07-06 — do not re-open)
================================================================================
DL-1  FULL UNIFY. The canvas launch control becomes THE single general launch console. The ELAI backlog-walk is one preset (mode=walk, project=elai). The chat "Run autonomous" form REDIRECTS to the same launch backend — no second launch path, no dead UI.
DL-2  WORKER-SOURCE + MODEL DROPDOWN. The user picks a worker source (Anthropic | Devin | Free APIs | Codex) then a model from that pool; the backend maps the pick to an ephemeral lane config / override of the toml role→model pins. The two named lane presets (devin-swe-1-6, claude-elai-max) become defaults, not the only choices.
DL-3  FREE-LANE SCOPE. The paid-lane-only v1 restriction on the "start at" scope is LIFTED as part of this work — scoping a run by task-id prefix must work on the free lane too (this was the deferred "v2" in teikoku-elai-finish.md; the mechanism is resolved in U2).
DL-4  DESIGN LAW unchanged: DESIGN-TEIKOKU.md SIGNAL — near-black + one dark-teal accent, square status marks, fail=light-inversion NEVER red, tokens.css only (zero literal hex/px), full state coverage. Every FE unit passes the standard design pass + cross-screen consistency at audit.
DL-5  NO ENGINE REWRITE. launch_run is a THIN DISPATCHER over the two EXISTING engines (headless_runner.run_headless for objectives; the walk driver for backlog). It resolves {lane, model, config, scope} and delegates; it never re-implements either loop. (REDTEAM-1 patch, binding.)

================================================================================
1. GROUNDED CURRENT STATE (live read 2026-07-06 — the seams this plan touches)
================================================================================
GROUNDED canvas/walk (the ELAI-only face):
  - signal/src/lib/screens/CanvasMonitor.svelte:550 mounts `<WalkControls walk={snap?.walk} onchange={refresh} />` (CanvasMonitor is ~927L, near the MR-1 1000-line boundary — DO NOT grow it; the new control is its own file like WalkControls).
  - signal/src/lib/components/WalkControls.svelte — provider picker over WALK_PROVIDER_OPTIONS=['devin-swe-1-6','claude-elai-max']; PAID_PROVIDER='claude-elai-max'; scopeDisabled = running || pendingProvider!==PAID_PROVIDER (the paid-only gate, DL-3 lifts it); calls startWalk(provider, scope).
  - signal/src/lib/components/WorkerSelect.svelte — the segmented picker (reusable; options + disabled_options props).
  - signal/src/lib/api/orchestration.ts:310 startWalk(provider?, walkScope?) / stopWalk() / registerProject().
  - conductor/walk_control.py — PROVIDERS={devin-swe-1-6→conductor.elai-walk.toml FREE cap2, claude-elai-max→conductor.elai-opus.toml PAID cap4}; DEFAULT_PROVIDER; walk_start(provider, config_path, walk_scope); walk_status(); _parse_walk_scope; _reclaim_stale_leases; PROVIDER_FILE/SCOPE_FILE sentinels; _SCOPE_ONLY_PROVIDER='claude-elai-max' (the paid-only policy point); scope path uses scope_resolve.resolve_walk_scope + apply_walk_scope; the driver is conductor/elai_walk_driver.sh → `run-backlog --project elai` (ELAI-hardwired).
  - conductor/orchestration_api.py — _walk_status attaches the `walk` field {available,running,stop_pending,provider,walk_scope} to the /orchestration snapshot the canvas polls.
GROUNDED general engine (the buried general face — ALREADY SHIPPED):
  - conductor/slice_api.py:185 projects_register (POST /projects/register); :315 run_headless handler (POST /headless/run); :1298 route "/projects" (list); :1300 "/projects/files"; :1368 "/headless/run"; :1371 "/projects/register"; :1374 "/walk/start"; :1391 "/walk/stop".
  - conductor/headless_runner.py — run_headless(project_id, free_text/template, …); _route_tier(role, template_tier); RunSpec.project_id default "elai"; resolve_feature_map_path(project_id) (elai-only map today, general fallback nonexistent).
  - conductor/project_resolver.py — register_project() (idempotent, fail-closed, defaults pack=generic), slugify_project_id(), safe_project_id() (empty/unknown → registered default).
  - conductor/model_pool.py — the per-source model catalog + windows (source of the model dropdown lists); worker_providers.route_pool_for (leaf/EXECUTE→FREE, ORCHESTRATE/AUDIT→PAID).
  - signal/src/lib/screens/ChatPane.svelte — the "…" → Run autonomous run-form (D6): headlessRun(), fetchProjects(), registerProject() (".../+ add folder"), .rfproj project dropdown (~lines 615-640, 1066-1090). THIS is the surface DL-1 redirects onto /launch.
  - signal/src/lib/components/NewSessionMenu.svelte — fMode interactive|autonomous + provider/model/skills (the chat new-session compose; reference for the model-picker idiom).
GROUNDED verdict: the general engine + a general launcher EXIST but live on the chat surface; the canvas carries only the ELAI walk. Consolidation = a thin unified launch backend + one general canvas console + redirect the chat form. No new engine.

================================================================================
2. RECON_NEEDED (settle BEFORE the executor edits — each has its exact check)
================================================================================
RECON_NEEDED R-1 (model override mechanism, feeds FORK-2): does headless_runner.run_headless accept a MODEL/tier override, or only role-routed tier? And does conductor_config.ConductorConfig accept in-memory overrides or MUST a config be a file on disk?
  check: wsl -d teikoku -- bash -lc "cd /home/user/teikoku/conductor && grep -nE 'def run_headless|_route_tier|model|tier|from_file|@dataclass' headless_runner.py conductor_config.py | head -60"
RECON_NEEDED R-2 (free-lane scope mechanism, feeds FORK-3 + REDTEAM-3): is there ANY post-creation priority-mutation or leaseable-allowlist path the scheduler honors, or only tier-retier? (teikoku-elai-finish.md says "no precedent for post-creation reprioritization" — confirm whether scope on FREE must be a frontier allowlist filter rather than a retier.)
  check: wsl -d teikoku -- bash -lc "cd /home/user/teikoku/conductor && grep -nE 'priority|acquire_ready|schedule_frontier|allow|scope|retier|tier ==' scheduler.py leases.py scope_resolve.py retier_elai.py | head -80"
RECON_NEEDED R-3 (walk-mode generality, feeds FORK-1): which registered projects have an imported backlog in control.db (a project can only run mode=walk if it has one)? Confirm run-backlog takes --project already.
  check: wsl -d teikoku -- bash -lc "cd /home/user/teikoku/conductor && python3 -c 'import sqlite3;c=sqlite3.connect(\"state/control.db\");print(c.execute(\"SELECT project_id,COUNT(*) FROM tasks GROUP BY project_id\").fetchall())'; grep -nE '\\-\\-project|argparse|run-backlog|run_backlog' cli/*.py conductor_loop.py 2>/dev/null | head"
RECON_NEEDED R-4 (model catalog for the dropdown): the exact per-worker-source model lists + context windows the FE dropdown should show.
  check: wsl -d teikoku -- bash -lc "cd /home/user/teikoku/conductor && grep -nE 'POOL_|MODEL|window|catalog|kimi|glm|swe|opus|sonnet|gpt-5' model_pool.py | head -60"
RECON_NEEDED R-5 (frozen-gate surface): are any files this plan edits in .claude/gate-manifest.txt (a frozen edit needs an owner re-lock, R8)? walk_control.py / scope_resolve.py / elai_walk_driver.sh are likely NOT frozen (they are runtime, not gate tools), but confirm.
  check: wsl -d teikoku -- bash -lc "cd /home/user/teikoku && grep -nE 'walk_control|scope_resolve|elai_walk_driver|launch_control|orchestration_api|slice_api' .claude/gate-manifest.txt || echo 'none frozen'"
GROUNDED (needs nothing): the /projects, /projects/register, /headless/run routes exist and work (P4-B generality gate + s-49deebfd dogfood); register_project drives any folder; the walk driver already takes --project. These are shipped and proven.

================================================================================
3. FORKS (deterministic triggers — the executor takes the branch, never guesses)
================================================================================
FORK-1 (walk-mode on a non-elai project): resolve R-3.
  TRIGGER: project has ≥1 imported backlog task in control.db → mode=walk is OFFERED for that project. TRIGGER: project has 0 backlog tasks → mode=walk is DISABLED in the UI for it and launch_run REFUSES mode=walk with an honest error ("<project> has no imported backlog; use objective mode"). objective mode is always available.
FORK-2 (model → config): resolve R-1.
  TRIGGER: run_headless/ConductorConfig accepts an in-memory model/tier override → pass the picked model as an override param (no temp file). TRIGGER: config MUST be a file → generate an EPHEMERAL <tmp>/conductor.<project>.<lane>.<model>.toml by copying the base lane toml and rewriting the role→model pins, pass its path as config_path (walk_start already takes config_path); delete it after the driver reads it (the driver pins CFG in its Popen env at spawn, so a post-spawn delete is safe — CONFIRM by reading how the driver loads CFG).
FORK-3 (free-lane scope): resolve R-2.
  TRIGGER: the scheduler honors a leaseable allowlist / priority path → implement free scope as a FRONTIER FILTER (the resolved closure is the only leaseable set; NO retier on FREE). TRIGGER: only tier-retier exists → free scope needs a NEW minimal frontier-allowlist filter in the scheduler (scope closure task_ids → an in-memory allow set consulted in acquire_ready) — the smallest change that makes "scope on FREE" real without a priority-mutation subsystem. Do NOT fake it by retiering FREE→FREE (a no-op that would silently scope nothing — REDTEAM-3).
FORK-4 (chat run-form): DECISION (DL-1, no recon) — KEEP the ChatPane run-form surface but point its submit at /launch (headlessRun becomes a thin wrapper over the unified startLaunch, or the form calls startLaunch directly). No dead UI, one backend.

================================================================================
4. THE UNITS (vertical slices — producer + real consumer + smoke in the SAME unit)
================================================================================
Contract threaded across U1-U5 (a REQUIRED input to each): the LAUNCH CONTRACT =
  launch_run(project_id: str, lane: str, model: str | None, mode: 'walk'|'objective', target: str) -> dict
  where lane ∈ {anthropic, devin, free, codex}; target = a scope string (mode=walk) OR an objective/template (mode=objective). This signature is FROZEN at U1 and every later unit consumes it unchanged; a unit that needs to change it STOPS + handoff (it is a cross-unit contract break, not a local edit).

--------------------------------------------------------------------------------
U0 — RECON + DESIGN LOCK (scout; no code)
--------------------------------------------------------------------------------
Resolve R-1..R-5, take FORK-1/2/3 branches, and write the resolved LAUNCH CONTRACT + the FORK decisions to workflow/reasoning/teikoku-consolidation-decisions.md (in the distro or F:, the orchestrator's choice). This is the context-compiler + one integration-explorer pass for the whole plan.
EXPECTED_OBSERVATION: a decisions doc naming, for each fork, which branch was taken and the evidence line that settled it; the frozen launch_run signature; the model-catalog lists for the dropdown.
ON_FAIL: a recon check returns ambiguous (e.g. the scheduler has neither a priority nor an allowlist path) → that is FORK-3's "new filter" branch, recorded as a NEW deliverable folded into U2, not a blocker.
RECON_NEEDED: R-1, R-2, R-3, R-4, R-5 (all above).
ABORT: if R-5 shows walk_control.py / scope_resolve.py / the driver ARE frozen in the gate-manifest → STOP + surface to owner (a frozen-tool edit needs an owner re-lock per R8; do not silently edit a frozen file).
VERIFY: the decisions doc exists and every fork line cites a real grep/db result:
  wsl -d teikoku -- bash -lc "test -s /home/user/teikoku/workflow/reasoning/teikoku-consolidation-decisions.md && grep -cE 'FORK-[1-3].*(TRIGGER|branch)' /home/user/teikoku/workflow/reasoning/teikoku-consolidation-decisions.md"
PRODUCTION_HOOK: n/a (no new pub symbol; prose deliverable).
INTEGRATION_SMOKE: n/a (scout unit).

--------------------------------------------------------------------------------
U1 — BACKEND: unified launch dispatcher + route + model override  [dep: U0]
--------------------------------------------------------------------------------
NEW conductor/launch_control.py with launch_run(...) per the LAUNCH CONTRACT: it resolves {lane, model} → a config/override (FORK-2), then DELEGATES — mode=objective → headless_runner.run_headless(project_id, free_text=target, <model/tier override>); mode=walk → the existing walk_start(provider-equivalent, config_path=<resolved/ephemeral>, walk_scope=target) (DL-5: no re-implementation). A bare launch with no explicit model preserves the current default (guard against silently overriding an operator retier — REDTEAM-2). Wire: POST /launch in slice_api.py (beside /headless/run + /walk/start) + a CLI verb (mirror the /walk verbs) + startLaunch(project_id, lane, model, mode, target) in signal/src/lib/api/orchestration.ts.
EXPECTED_OBSERVATION: `POST /launch {project_id:"demo", lane:"free", mode:"objective", target:"print hi"}` returns 200 with a run_id, and a HEADLESS_INTAKE event lands in control.db carrying project_id=demo + role + model_tier. A mode=walk launch on project=elai spawns the driver (pgrep elai_walk_driver.sh non-empty).
ON_FAIL: run_headless rejects the override arg → FORK-2 was mis-resolved; fall to the ephemeral-config branch. 500 on /launch → read the traceback in the response (slice_api returns structured errors), fix the delegate call, re-smoke.
FORK: FORK-2 (model→config) is realized here.
RECON_NEEDED: R-1 (already settled in U0; re-confirm the exact run_headless signature before the delegate call).
ABORT: if delegating mode=walk requires editing the running production walk's config → do it against a SCRATCH control.db / paused walk (touch ELAI_WALK_STOP first), never mutate the live walk mid-flight.
WIRING_CONTRACT: produces launch_run + POST /launch + startLaunch; consumed by the CLI verb + (U4) the canvas + (U5) the chat form. PRODUCTION_HOOK: slice_api.py route table `elif route == "/launch":` (beside :1368 "/headless/run") — the executor wires launch_run into that handler in THIS changeset.
VERIFY (off-agent, real artifact — the auditor, not the executor):
  wsl -d teikoku -- python3 /home/user/teikoku/.claude/tools/python_wire_check.py --files conductor/launch_control.py conductor/slice_api.py   (no DARK symbol; launch_run has a production caller in the route table)
  + a live route smoke on a SCRATCH slice_api + scratch control.db (never the production db): POST /launch objective on the demo pack → assert the HEADLESS_INTAKE row.
INTEGRATION_SMOKE: n/a at U1 (covered by U3 + U6 cadence).
REDTEAM note applied: launch_run is a dispatcher (DL-5); if a reviewer finds run_headless or the walk loop logic COPIED into launch_control.py, that is a BLOCK — it must delegate.

--------------------------------------------------------------------------------
U2 — BACKEND: free-lane scope  [dep: U1; shares the launch contract with U1/U3]
--------------------------------------------------------------------------------
Lift the paid-only gate so mode=walk + a scope works on the FREE lane. Per FORK-3: if the scheduler has no priority/allowlist path, add the minimal FRONTIER-ALLOWLIST filter (resolved scope closure → an allow set consulted in leases.acquire_ready / scheduler.schedule_frontier) so a scoped free walk leases ONLY the closure; remove _SCOPE_ONLY_PROVIDER='claude-elai-max' as the hard gate (keep it as the DEFAULT, not the only-allowed).
EXPECTED_OBSERVATION: launch mode=walk, lane=free, target="J-1" on project=elai → the leased tasks are exactly J-1's prefix set + their non-DONE dep closure (query LEASE_ACQUIRED events), and NO task outside the closure is leased over one full pass.
ON_FAIL: the scoped free walk leases the whole backlog → the retier-FREE→FREE no-op trap (REDTEAM-3); switch to the frontier-allowlist filter branch (FORK-3 branch B).
FORK: FORK-3 realized here.
RECON_NEEDED: R-2 (re-confirm the scheduler path chosen in U0 against the live acquire_ready code before editing).
ABORT: any change to scheduler.py/leases.py that would alter DECLARED-write-set serialization semantics (parallel_undeclared_scopes) → STOP + handoff; the free-scope filter must be ADDITIVE (an allow set), never a rewrite of the conflict logic.
WIRING_CONTRACT: produces the free-scope apply path; consumed by launch_run mode=walk on a free lane (U1's dispatcher passes the scope through). PRODUCTION_HOOK: launch_control.launch_run's mode=walk branch calls the free-scope resolver (scope_resolve.resolve_walk_scope + the new allow-set apply) — wired in THIS changeset, not deferred.
VERIFY (off-agent): unit assertions (scope closure = expected task_ids; a task outside the closure is NOT in the allow set) + a live scratch-db scoped free walk showing LEASE_ACQUIRED only within the closure. python_wire_check on the changed .py files: no DARK.
INTEGRATION_SMOKE: n/a at U2 (rolled into U3).

--------------------------------------------------------------------------------
U3 — BACKEND: generalize the walk driver + walk-mode preconditions  [dep: U1]
--------------------------------------------------------------------------------
Parametrize conductor/elai_walk_driver.sh (or a general sibling) to take PROJECT + CFG from env instead of the elai hardcode, so mode=walk can target any project WITH a backlog. launch_run mode=walk on a project with 0 backlog tasks REFUSES with an honest error (FORK-1). Keep elai as the default preset. Re-chmod +x if edited over UNC.
EXPECTED_OBSERVATION: mode=walk project=elai still runs exactly as today (regression-clean: driver pgrep, LEASE_ACQUIRED). mode=walk on a registered project with 0 backlog → launch_run returns an explicit "no backlog" error, spawns NO driver.
ON_FAIL: the generalized driver breaks the elai walk (wrong PROJECT default) → the smoke's elai regression leg fails; restore the elai default and gate the generality behind the explicit project arg.
FORK: FORK-1 realized here.
RECON_NEEDED: R-3 (per-project backlog counts, already pulled in U0).
ABORT: never generalize the driver against the LIVE running elai walk — touch ELAI_WALK_STOP, let the pass drain, then edit; the running walk's disk/vhdx pressure is real (teikoku-glm-walk.md crash history).
WIRING_CONTRACT: produces the general driver; consumed by launch_run mode=walk. PRODUCTION_HOOK: launch_control.launch_run mode=walk branch invokes the parametrized driver with PROJECT/CFG env — wired here.
VERIFY (off-agent): the elai regression leg (driver runs, leases as before) + the no-backlog refusal leg, both observed on a scratch/paused walk. No DARK symbol.
INTEGRATION_SMOKE (cadence, after 3 backend units): an INDEPENDENT agent drives the REAL /launch entrypoint on a HARD input — (a) mode=walk lane=free scope="J-1" project=elai → leases only the closure; (b) mode=objective lane=free project=demo → HEADLESS_INTAKE + a real free worker turn. Toy happy-path does not count. db-not-found / 0 rows = FAIL.

--------------------------------------------------------------------------------
U4 — FRONTEND: canvas general launch console  [dep: U1, U2, U3]
--------------------------------------------------------------------------------
NEW signal/src/lib/screens/LaunchControls.svelte (own file — do NOT grow CanvasMonitor past MR-1): project dropdown (fetchProjects + "+ add folder" → registerProject, reuse the ChatPane idiom), a WorkerSelect lane picker (Anthropic|Devin|Free|Codex, disabled ones calm per DL-4), a model dropdown scoped to the picked lane (from R-4's catalog), a mode toggle (backlog-walk | objective; walk disabled per FORK-1 when the project has no backlog), and a target field that is a scope input (mode=walk) or an objective textarea (mode=objective). Wire onclick → startLaunch(...). Replace the `<WalkControls .../>` mount at CanvasMonitor.svelte:550 with `<LaunchControls .../>` (WalkControls retained unmounted for reference, or deleted if the auditor confirms no other consumer). Preserve the running-lock (disabled + backend-reported active run while running).
EXPECTED_OBSERVATION: the canvas shows project + lane + model + mode + target; picking Free→model list updates; picking objective swaps the input to a textarea; a click starts a real run and the canvas monitor shows the new lease/frontier within one 5s poll.
ON_FAIL: the model list is empty for a lane → R-4 catalog wiring wrong; the mode toggle doesn't gate on backlog → FORK-1 UI wiring missing.
FORK: FORKS: none new (consumes U1-U3 branches).
RECON_NEEDED: GROUNDED — CanvasMonitor:550 mount + WorkerSelect props + orchestration.ts startLaunch (U1) are all read/known; no new recon.
ABORT: if replacing the WalkControls mount breaks the running-walk lock or the /orchestration `walk` snapshot read → keep WalkControls mounted behind a flag and surface the seam (STOP + handoff) rather than shipping a canvas that can't stop a running walk.
WIRING_CONTRACT: produces LaunchControls; consumed by CanvasMonitor (the mount). PRODUCTION_HOOK: CanvasMonitor.svelte:550 `<LaunchControls walk={snap?.walk} onchange={refresh} .../>` — the mount IS the production caller, wired in this changeset (svelte_mount_reachability must show LaunchControls REACHABLE from the app root).
VERIFY (off-agent): svelte-check 0/0 + vite build green + the taste/design pass (taste_check.py if present in the distro, else the design-pass checklist) + svelte_mount_reachability (LaunchControls reachable, not ORPHAN) + a CDP screenshot of the console in each mode. The load-bearing proof is U6's ui-wire-smoke.
INTEGRATION_SMOKE: n/a at U4 (U6 owns the frontend smoke).

--------------------------------------------------------------------------------
U5 — REDIRECT the chat run-form onto the unified backend  [dep: U1]
--------------------------------------------------------------------------------
Point signal/src/lib/screens/ChatPane.svelte's "Run autonomous" submit at startLaunch (FORK-4: keep the surface, one backend). Either make headlessRun a thin wrapper over startLaunch or call startLaunch directly; the form already collects project + objective, so add lane/model to match (reuse the same pickers as U4 or the NewSessionMenu idiom). No behavior regression: a chat-side autonomous run still works, now through /launch.
EXPECTED_OBSERVATION: submitting the chat "Run autonomous" form produces the SAME HEADLESS_INTAKE event shape as a canvas objective launch (one backend, verified by comparing the two event rows).
ON_FAIL: the chat form and the canvas produce different intake shapes → they are not on one backend yet; route both through launch_run.
FORK: FORK-4 (DECISION, no recon).
RECON_NEEDED: GROUNDED — ChatPane run-form seams read (headlessRun/fetchProjects/registerProject, lines ~615-640/1066-1090).
ABORT: if unifying forces a change to the frozen LAUNCH CONTRACT (U1) → STOP + handoff (contract break, not a local edit).
WIRING_CONTRACT: produces the redirected form; consumed by the user via the chat surface. PRODUCTION_HOOK: ChatPane.svelte run-form submit handler now calls startLaunch — wired here; if headlessRun is kept as a wrapper it must call launch_run so there is a single backend.
VERIFY (off-agent): svelte-check 0/0 + build green + the two-intake-shape comparison (canvas objective vs chat objective produce matching HEADLESS_INTAKE rows). No DARK.
INTEGRATION_SMOKE: n/a at U5 (U6).

--------------------------------------------------------------------------------
U6 — INTEGRATION SMOKE (independent agent; drives the REAL canvas end-to-end)  [dep: U1-U5]
--------------------------------------------------------------------------------
An agent that did NOT author U1-U5 drives the REAL running dashboard (teikoku-native relaunch → the canvas) and the REAL /launch entrypoint on HARD inputs:
  (a) canvas: project=elai, lane=Free, model picked, mode=backlog-walk, scope="J-1" → assert LEASE_ACQUIRED only within the J-1 closure (free-lane scope proven — DL-3);
  (b) canvas: project=demo, lane=Free, mode=objective, a real objective → assert HEADLESS_INTAKE + a real free worker turn on demo (generality proven — not elai);
  (c) chat "Run autonomous" on demo → assert the SAME intake shape as (b) (one backend proven — DL-1);
  (d) start-while-running is a no-op / disabled (lock preserved).
Use the frontend Layer-4 leg: .claude/tools/visual/ui-wire-smoke.mjs (or the distro equivalent) to assert a NEW Rust/conductor-owned event row lands on a real canvas click (baseline-delta), not a pixel check.
EXPECTED_OBSERVATION: all four legs produce their asserted off-agent rows; db-not-found / Chrome-unreachable / 0-rows = FAIL (fail-closed).
ON_FAIL: any leg fails → the seam it exercises is the defect; route back to the owning unit (a-U2, b-U1/U3, c-U5, d-U4), do NOT patch in the smoke.
FORK: FORKS: none.
RECON_NEEDED: GROUNDED (drives shipped entrypoints).
ABORT: if the live run would push to origin/main (the walk has allow_push) on a HARD input before the owner has seen it → run the smoke against a SCRATCH db / a non-pushing config, never the production auto-pushing elai walk.
WIRING_CONTRACT: n/a (verification unit). PRODUCTION_HOOK: n/a.
VERIFY (off-agent, IS the unit): the four asserted rows + the ui-wire-smoke exit 0.
INTEGRATION_SMOKE: THIS unit is the R10 gate for the whole plan.

================================================================================
5. REDTEAM (attacks run against the plan; surviving attack + patches on record)
================================================================================
REDTEAM-1 (SURVIVED): "walk (continuous loop) and headless (one-shot) are incompatible; one launch_run behind both is a leaky abstraction that will drift." → survives because DL-5 makes launch_run a THIN mode-switch dispatcher that delegates to each UNCHANGED engine; it resolves {lane,model,config,scope} only. PATCH (binding): any copied loop/disposition logic inside launch_control.py is an audit BLOCK.
REDTEAM-2 (attack landed → PATCH): "an ephemeral model-override silently overrides an operator's hand-set retier_elai tiering — the exact bug walk_start's own docstring warns about." → PATCH: a launch with no EXPLICIT model preserves the current default and does NO retier (mirrors the existing bare-Start contract); the override fires only on an explicit user model pick. Encoded in U1's ON_FAIL guard.
REDTEAM-3 (attack landed → PATCH): "free-lane scope has no priority-mutation precedent, so a naive impl retiers FREE→FREE = a no-op and the 'scoped' free walk silently leases the whole backlog." → PATCH: FORK-3 forbids the retier-FREE no-op; free scope MUST be a frontier-allowlist filter (the closure task_ids are the only leaseable set). U2's EXPECTED_OBSERVATION asserts nothing outside the closure leases.
REDTEAM-4 (attack landed → PATCH): "replacing the WalkControls mount breaks the running-walk STOP / the /orchestration walk-snapshot lock, leaving a canvas that starts runs it can't stop." → PATCH: U4 preserves the running-lock + the walk snapshot read; the ABORT condition stops the unit if the lock can't be preserved. U6 leg (d) proves start-while-running is a no-op.
REDTEAM-5 (attack landed → PATCH): "the plan edits a frozen gate tool and the run rewrites its own evaluator (R8)." → PATCH: R-5 checks the gate-manifest FIRST; a frozen target is an ABORT + owner re-lock, never a silent edit.

================================================================================
6. ABORT / TRIPWIRES (stop + surface to owner; do not improvise)
================================================================================
- Any edit target is in .claude/gate-manifest.txt (frozen) → stop, owner re-lock (R8).
- Any operation would mutate the LIVE running elai walk / the production control.db / push to origin/main before owner review → stop; use scratch db / paused walk.
- The frozen LAUNCH CONTRACT (U1) needs changing to satisfy a later unit → stop + handoff (cross-unit contract break).
- A scheduler change would alter the parallel_undeclared_scopes serialization semantics → stop (free-scope filter must be additive).
- Live-lane verification blocked on creds (plan-OAuth expired / Max not logged) → ship the backend + FE, mark the live-lane leg deferred (mirrors every prior TEIKOKU deferral); do NOT fake a live pass.
- Worker touches the ELAI FSM, or runs git at /mnt/f, or uses the Bash tool for distro ops instead of PowerShell→wsl → banned (prior P0 incidents).

================================================================================
7. VERIFY (the off-agent artifacts that flip a unit DONE — auditor, never executor)
================================================================================
Per unit above (python_wire_check / a real event row / svelte_mount_reachability / ui-wire-smoke / the two-intake comparison). Whole-plan gate = U6 (R10 integration smoke) PASS with all four legs + the frozen checkers green. The auditor RE-RUNS every checker off-agent; the executor's self-run is necessary-not-sufficient. Governing standard checker (run by the orchestrator pre-flight AND the auditor):
  wsl -d teikoku -- python3 /home/user/teikoku/.claude/tools/plan_gate_check.py --file <this plan in the distro>
Commit: ONE changeset at the very end, maintainer voice, no AI trailer, owner-hold until the owner says go. If any conductor file edited is under the distro gate-manifest, re-lock out-of-band per R8 AFTER owner sign-off:
  wsl -d teikoku -- python3 /home/user/teikoku/.claude/tools/gate_freeze_check.py --lock

================================================================================
8. SELF-CHECK (before any unit is called done)
================================================================================
1. Could a mid-tier session run this blind — zero questions? Every fork has a trigger; every unsettled assumption is a RECON_NEEDED with its exact check.
2. Does every code unit ship producer + real consumer (PRODUCTION_HOOK) + smoke in the same changeset? (U1-U5 yes; U0/U6 are scout/verify.)
3. Is launch_run a dispatcher, not a merged engine? (DL-5 + REDTEAM-1.)
4. Does free-lane scope actually restrict leasing (not a FREE→FREE no-op)? (U2 EXPECTED_OBSERVATION + REDTEAM-3.)
5. Is the running elai walk / production db / origin push protected from the build? (ABORT set.)
If any answer is no, it is not a plan — it is a hope.

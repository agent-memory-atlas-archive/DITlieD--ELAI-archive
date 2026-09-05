SCOPE-scout (task_id=SCOPE-scout, session=s-finish-0705)
Repo: teikoku's own git repo /home/user/teikoku (WSL distro "teikoku"), HEAD=5f06156
"paid lane proven end to end..." (post SP-2B, post SLOT-FIX/DECISION-a). Uncommitted at read
time: conductor/brain_capture.py, brain_store.py, slice_api.py (M, other session's in-flight
work, unrelated hunks per grep below); several new research_*.py + tests untracked (unrelated).
slice_api.py's /walk/start route block (see 4) IS the committed 5f06156 shape, confirmed by
content match against SP-2B's own described diff.

FINDING 0 (read first): SP-2B (provider picker) is FULLY SHIPPED end-to-end, both legs. This
task is a NEW, DISTINCT feature: filtering which task_ids the walk drains ("start at" scope),
not which provider/lane runs them. The two compose (a scope-scoped run against either lane).

1. CLOSURE SQL
tasks columns (DATA-MAP.md:115-121, live-confirmed): task_id, plan_id, wave, role, objective,
base_commit, allowed_write_set_json, context_refs_json, production_hook, acceptance_text,
deterministic_check_text, forbidden_text, verifier_json, lease_ttl_seconds, idem_key,
token_budget, model_tier, priority, status, attempts, max_attempts, created_at, updated_at.
task_deps: (task_id, depends_on_task_id) -- task_id is the DEPENDENT, depends_on_task_id is the
prerequisite (verified: ('B-1-AUDIT-1','B-1-INTEGRATE-1') = AUDIT depends on INTEGRATE).
task_id format LIVE-CONFIRMED: LETTER(S)-N-STAGE-M, e.g. B-1-EXECUTE-1, B-10-AUDIT-1,
J-1-EXECUTE-1. Stages seen: EXECUTE, INTEGRATE, AUDIT, POLISH (role column; WIRE/FIX also valid
per DATA-MAP role enum but no live rows sampled).
CROSS-LETTER DEPS ARE REAL: 255 of 1947 task_deps rows (~13%) cross a letter boundary, e.g.
C-1-EXECUTE-1 -> B-4-EXECUTE-1, D-1-EXECUTE-1 -> C-15-EXECUTE-1. A scope of "just J-1" or "just
K" WILL pull in earlier-letter unmet deps -- confirmed live: scope ('J-1-%','K-%') closure = 104
tasks incl. E-26/E-27/G-8/G-9/H-7/I-25/I-38 (all NEEDS_REVIEW/READY/BLOCKED, i.e. genuinely
unmet), not just J/K rows.
VALIDATED recursive closure query (run live against state/control.db, 104-row result, correct):
  WITH RECURSIVE seed(task_id) AS (
      SELECT task_id FROM tasks WHERE task_id LIKE ? OR task_id LIKE ?  -- one ? per scope pattern
  ),
  closure(task_id) AS (
      SELECT task_id FROM seed
      UNION
      SELECT d.depends_on_task_id
      FROM closure c
      JOIN task_deps d ON d.task_id = c.task_id
      JOIN tasks t ON t.task_id = d.depends_on_task_id
      WHERE t.status != 'DONE'
  )
  SELECT t.* FROM tasks t JOIN closure c ON c.task_id = t.task_id ORDER BY t.task_id;
Params are prefix patterns e.g. "J-1-%" (unit-exact, safe: "J-1-%" does NOT match "J-10-...",
since char 4 is '0' not '-') or "K-%" (whole-letter). Seed tasks are included unconditionally;
only non-DONE deps are pulled into the closure (matches the ask: "matching tasks PLUS the
transitive closure of their UNMET deps"). Recursion naturally re-expands each newly-added
non-DONE dep's own deps next iteration.

2. RETIER SCOPING
retier_elai.py (49 lines) retier(tier, db=DB) is ALREADY the shared importable core (post-5f06156
docstring, line 8: "walk_control.walk_start(provider) calls it directly ... share ONE tier-flip
implementation"). Current UPDATE (retier_elai.py:24-27) is GLOBAL/unscoped: `UPDATE tasks SET
model_tier=? WHERE status!='DONE' AND role IN ('EXECUTE','WIRE','INTEGRATE','AUDIT','POLISH')` --
confirmed it touches ONLY non-DONE rows already (no scope arg exists yet). A scoped variant needs
`AND task_id IN (<closure ids>)` added to that WHERE, as a NEW function (do not mutate retier() 's
signature -- walk_start(provider) without scope must keep the exact global-retier behavior it has
today, per walk_control.py:120-123's own documented idempotency contract).
PRIORITY: lives in tasks.priority (int). Written ONLY at task-creation time, nowhere else:
backlog_import.py:595 `priority=10 if status=="READY" else 0` (initial import), :648
`priority=max(0, base_task.priority-1)` (followup tasks), tree_to_schema.py:225 default 0. NO
existing code path re-prioritizes a task post-creation. leases.py:102 confirmed:
`ORDER BY t.priority DESC, t.created_at, t.task_id` (ties break FIFO by created_at then task_id --
this is exactly the ordering that let C-51-EXECUTE-1 beat J-1-EXECUTE-1 in the paid-leg-entry
handoff, since both were priority 0). A scope-picker that wants in-scope tasks to actually lease
first needs a NEW write: `UPDATE tasks SET priority=? WHERE task_id IN (...)`, there is no
existing helper for this -- it is new ground, not an extension of anything.

3. WALK_CONTROL STATE (walk_control.py, 187 lines -- 1 shy of the SP-2B pkg's "188L")
walk_start(provider: str|None=None, config_path: str|Path|None=None) -> dict (line 112). Idempotent
purely on `_driver_pids()` truthy (131-132): a second start while ANY provider/scope is running is
a no-op, current running one is NOT inspected against the new request -- same "locked until Stop"
contract a scope param would need to inherit.
provider persistence idiom (lines 43-45, 60-78): PROVIDER_FILE = CONDUCTOR_DIR/"ELAI_WALK_PROVIDER",
written at the end of a successful start (162-165), read back in walk_status() (66-71) into the
`provider` key. walk_status() (60-78) returns {ok, running, pids, stop_pending, provider}. A scope
sentinel should mirror this exactly: e.g. SCOPE_FILE = CONDUCTOR_DIR/"ELAI_WALK_SCOPE", written
next to PROVIDER_FILE, read back into a new `walk_scope` (see naming collision below) key in
walk_status()'s return dict.
orchestration_api.py::_walk_status() (491-505) passes provider through 1:1 (line 502); the scope
key needs the same 1-line passthrough addition.

NAMING COLLISION (must avoid, same class as SP-2B's WorkerProvider note): `scope` is ALREADY a
load-bearing name in this codebase for something unrelated -- orchestration_api.py's
SCOPE_MINE="mine"/SCOPE_ALL constants (line 70) and `build_orchestration_snapshot(scope:
str=SCOPE_MINE)` (line 508, called from slice_api.py GET /orchestration?scope=mine|all, lines
1169-1170) gate WHICH PROJECT's data the snapshot shows, nothing to do with task filtering. Naming
the new param literally `scope` anywhere near orchestration_api.py or slice_api.py's do_GET would
be actively confusing (two different "scope"s reachable from the same file). Recommend `walk_scope`
or `unit_scope` for the new concept, matching the existing `walk_provider`/PROVIDER_FILE naming
family, never bare `scope`.

4. JSON-BODY GUARD SEAM
slice_api.py, POST /walk/start handler, current shape (lines 1278-1282, 5f06156-committed,
matches SP-2B's own description exactly):
  elif route == "/walk/start":
      import walk_control
      payload = self._read_json_body()
      provider = payload.get("provider")
      self._send(200, walk_control.walk_start(provider=str(provider) if provider else None))
_read_json_body (1141-1150) already reads Content-Length itself (`length =
int(self.headers.get("Content-Length","0") or "0")`) but swallows it -- on ANY parse failure
(bad JSON, non-dict) it returns {} with no way for the caller to tell "empty body" from "malformed
body" apart. Do NOT touch _read_json_body (other routes rely on its {}-on-anything fail-soft).
Minimal route-local guard, reading raw length again (cheap, header lookup only, no double body
read): `body_len = int(self.headers.get("Content-Length","0") or "0")`; `payload =
self._read_json_body()`; `if body_len > 0 and not payload: self._send(400, {"ok": False, "error":
"malformed JSON body"})`. Same route also needs `walk_scope = payload.get("walk_scope")` (or
whatever the picked name is) threaded into the walk_control.walk_start(...) call alongside
provider.

5. FE DELTA
CanvasMonitor.svelte is 1002 lines TODAY (pre-existing, already 2 over the MR-1 1000-line
boundary before this unit touches it -- flag to auditor, any net growth here is scrutiny, not a
free pass; this unit did not cause the crossing). Current walk block: script 96-124 (walkBusy,
walk $derived, WALK_PROVIDER_OPTIONS, pendingProvider $state, walkProviderValue $derived,
toggleWalk(), walkLabel $derived.by); markup 582-598, inside `{#if walk?.available}` in `.bar-l`:
<WorkerSelect value={walkProviderValue} options={WALK_PROVIDER_OPTIONS}
disabled_options={walk.running ? WALK_PROVIDER_OPTIONS : []} onchange={...}/> then the
start/stop <button class="walkbtn">. A compact scope text input fits between WorkerSelect and the
button (same `.bar-l` flex row, same `{#if walk?.available}` guard), state-managed exactly like
pendingProvider (a `pendingScope` $state string, disabled while walk.running per the START-ONLY
pinning comment at lines 99-102). WorkerSelect itself is NOT reusable for scope (it is a fixed
segmented-options control; scope is free-text like "J-1" or "K", not a closed option set) --
correctly needs a plain `<input>`, which THIS file has none of today. Token pattern for a compact
input, live-grepped from ResearchTab.svelte:242/458 (`.qcat` class, same signal/src tree, same
design law): `width: 96px; padding: 2px var(--s-1); background: var(--sink); color: var(--text);
border: var(--bw) solid var(--line-2); border-radius: var(--r); font-family: var(--font-data);
font-size: var(--fz-caption);` -- note `--sink` is a real token (not in the SP-2B package's listed
var set, additional one found live) used for input backgrounds specifically.
orchestration.ts (317 lines): WalkStatus type (182-187ish, now includes `provider?: string` per
SP-2B) needs `walk_scope?: string` added the same way. `_walkAction(action, provider?)` (275-289)
needs a 3rd param for scope, folded into the POST body next to provider exactly as provider was
folded in by SP-2B (`hasBody = action==='start' && !!provider` at 277 needs `|| !!scope` added).
`startWalk(provider?)` (290) becomes `startWalk(provider?, walk_scope?)`.

6. PRODUCTION_HOOK
walk_start remains the seam (R-8 confirmed): it is the ONE place provider AND scope both need to
resolve to concrete DB effects before the driver Popen's. Exact insertion points:
- slice_api.py, POST /walk/start handler (~line 1278-1282): parse `walk_scope =
  payload.get("walk_scope")`, apply the guard from (4), pass into walk_control.walk_start(...).
- NEW MODULE recommended over reusing walk_control.py or conductor_loop.py: `conductor/
  scope_resolve.py` (mirrors the retier_elai.py / slot_pools.py extraction precedent already in
  this codebase -- both were pulled OUT of a growing file rather than bolted onto it).
  walk_control.py is 187L (room exists) but the closure query + a scoped retier + a priority-bump
  are ~40-60 new lines of genuinely separate logic (SQL + tests), same shape as why retier_elai
  was already its own file; conductor_loop.py is 961L (39 lines from the MR-1 wall -- do NOT add
  scope logic there, DECISION-a's own history shows this codebase already had to split
  worker_adapter.py once for crossing 1000). scope_resolve.py exposes: `resolve_scope(patterns:
  list[str], db) -> dict` (the closure query above, returns {matched_ids, closure_ids,
  histogram}), and `retier_scope(task_ids: list[str], tier: str, db) -> dict` (the WHERE task_id
  IN (...) variant of retier_elai.retier, NOT a mutation of retier_elai.retier's existing
  signature/behavior). Optionally a `bump_priority(task_ids, db)` helper for the priority write
  gap found in (2) -- flagging as a genuine design decision (does scope imply priority bump? not
  stated in this task's ask), not deciding it here.
- walk_control.py::walk_start (112-173): add `walk_scope: str|None=None` param; when given, call
  scope_resolve.resolve_scope + scope_resolve.retier_scope INSTEAD OF the current blanket
  retier_elai.retier(spec["tier"], ...) call at line 147 (only when walk_scope given, mirroring
  the existing `if provider is not None:` gate at 146 exactly); write SCOPE_FILE sentinel
  alongside PROVIDER_FILE (162-165).
- orchestration_api.py::_walk_status (491-505): add `"walk_scope": s.get("walk_scope")` at the
  same spot as line 502's `"provider": s.get("provider")`.
- orchestration.ts + CanvasMonitor.svelte: per (5).

STATUS: DONE
FILES TOUCHED: F:\.ELAI_workflow\workflow\reasoning\scope-picker-scout.md (this file, created)
VERIFIED: read walk_control.py (188L), retier_elai.py (49L), slice_api.py:1141-1341 +
orchestration_api.py:491-566 + orchestration.ts:180-292 + CanvasMonitor.svelte:92-124,570-598 via
Read/Grep; ran live read-only sqlite queries against teikoku's state/control.db (wsl -d teikoku,
python3 sqlite3, SELECT-only, zero mutations) confirming task_id format, task_deps direction,
cross-letter dep count (255/1947), priority distinct values + writer sites, and the full recursive
closure query end-to-end (104-row result for scope J-1+K, spot-checked). `wc -l` confirmed current
line counts for all 11 touched-or-adjacent files (CanvasMonitor.svelte 1002, conductor_loop.py 961
flagged as near-boundary). git log -1 confirmed HEAD=5f06156 matches the paid-leg-entry handoff's
DECISION-a commit; git status --short confirmed no relevant conductor .py files are mid-flight
except slice_api.py (M, unrelated research_* hunks, /walk/start route content unchanged from
committed 5f06156 shape -- confirmed by direct read matching SP-2B's own described diff verbatim).
PRODUCTION_HOOK: walk_control.py::walk_start(provider, config_path) gains a walk_scope param (new
module conductor/scope_resolve.py for closure+scoped-retier); slice_api.py POST /walk/start
(~1278) is the wiring seam that parses it from the request body.
NOTES: (1) naming collision -- do not call the new param bare `scope`, it collides with
orchestration_api.py's project-visibility SCOPE_MINE/SCOPE_ALL; recommend `walk_scope`. (2) no
existing code re-prioritizes a task post-creation -- a priority-bump-on-scope-select is a genuine
open design question, not answered by any existing pattern, flag to the coordinator before an
executor invents one. (3) CanvasMonitor.svelte is already 1002 lines pre-existing; any FE addition
here is auditor-scrutinized growth on an already-over-boundary file, not this unit's fault but
worth naming up front.

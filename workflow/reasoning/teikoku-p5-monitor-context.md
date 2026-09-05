TEIKOKU P5-MONITOR CONTEXT PACKAGE — dashboard monitoring completion + config UI + state sweep
context-compiler (READ-ONLY), task_id=P5-MONITOR-CC, 2026-06-21
Group = P5-C (5.1 monitoring completion) + P5-D (5.3 concurrency/limits config UI) + P5-E (5.2 §10 state-coverage + consistency sweep).
All distro facts READ live from WSL distro teikoku (~/teikoku) via PowerShell->wsl. Nothing edited/started. P5-HARDEN already SHIPPED (migration-006 LIVE on control.db, T2/T3 applied) per handoff CURRENT STATUS.

================================================================================
1. EXISTING MONITORING SURFACES (COMPLETE not rebuild — N-02/N-03)
================================================================================
Surfaces: ~/teikoku/signal/src/lib/screens/{Orchestration(477L), Memory(532L), ChatPane(333L), SessionList(190L), Harness(307L), Gallery}.svelte + App.svelte(234L) shell. All compose ONLY $lib/components/index.ts + read a typed api seam; zero hex (verified pattern across files).

THE FIVE 5.1 READOUTS (EXISTS / PARTIAL / MISSING):

(a) token/usage meter — EXISTS, wired. TokenMeter + UsagePopup components; ChatPane binds ctxUsed/ctxCap/sessionTokens into Composer (ChatPane.svelte:~225 composerwrap). P4-D landed the per-model window-source fix (model_pool, kills base-cc flat-200k). VERIFY-only: "accurate" = confirm the reader matches the improved CC counter on a LIVE turn end-to-end (no code gap found; this is an audit assertion, not a build item). gpt-5.5 window stays OWNER-OVERRIDE-PENDING placeholder (DEFER, owner).

(b) gate-verdict timeline — EXISTS as recent-rows, NOT an explicit chain. Orchestration.svelte:263-287 "gate verdicts" block: rows from snap.gates (orchestration_api._gate_rows, SELECT ... FROM gate_runs ORDER BY started_at DESC LIMIT 100), each row = StatusMark(gateMark(verdict)) + gate_name + task_id + relTime(started_at). PASS=done(filled), FAIL=fail(inversion chip, gateMark():58-63), ERROR/TIMEOUT=warn. GAP vs "timeline": it is reverse-chron rows, not a visually-ordered chain/lane. 5.1 says "timeline" — DECISION for P5-C: the rows ARE time-ordered already; upgrade to an explicit timeline ONLY if the recent-rows reading is judged insufficient (lean bias N-03 says do not add a viz; a left time-rail / per-task grouping is the max defensible upgrade). Recommend: keep rows, add a thin time-grouping or a task-grouped lane label, NOT a new viz dep.

(c) handoff/migration chain — PARTIAL/MISSING. ChatPane has the LIVE per-turn migrating indicator (ChatPane.svelte:189-198, StatusMark state="live" "migrating context…" + accent-deep bar, NEVER red; migrated -> done mark + caption). Memory.svelte indexes handoff events (prettyPayload:131-147 parses handoff {entry_date,source_file,summary} payloads into the flat append-only LOG). MISSING: a dedicated migration/handoff CHAIN readout = the LINEAGE of migrations within a chat (the ordered sequence of session_migrating->session_migrated hops for a session_id, showing from_model->to_model + reseeded_messages + snapshot_id per hop). The data EXISTS (see §2); no surface renders the chain.

(d) concurrency view — EXISTS, now LIVE post-006. Orchestration.svelte:289-329: if !snap.concurrency.available -> degrade block (caps only + reason); else per-pool inflight/cap segmented bars + at_cap StatusMark(fail "at cap"). orchestration_api._concurrency:132-158 catches OperationalError and degrades (R1 posture). Since P5-HARDEN applied migration-006 LIVE, available:true should now render (VERIFY on the auditor's own capture). Fail-soft branch retained for fresh dbs. This readout is essentially DONE; P5-C verifies it renders live.

(e) per-lane status — PARTIAL. Orchestration.svelte:244-261 "worker slots" block renders per-slot rows (slotMark(state) + slot_id + harness + model + state). That is per-SLOT, not a per-LANE (claude/devin-free/free-api/codex) ROLLUP. orchestration_api._worker_rows:64-79 SELECTs slot_id/harness/model/model_tier/account_id/state/cooldown_until/last_error_category but does NOT select the new worker_slots.pool column (SQL still pinned to the pre-006 shape). MISSING: a per-lane status rollup (one row per pool/harness lane: live/idle/cooldown counts + at-cap). Derivable from the worker rows (group by harness or pool) OR from the concurrency pools already in the snapshot.

SO: 4 of 5 exist in some form (token, gate-rows, concurrency, worker-slot rows). The two genuine COMPLETION deltas for P5-C: a handoff/migration CHAIN readout + a per-lane status ROLLUP. The gate "timeline" and "accurate" token are VERIFY/refine items, not net-new machinery.

================================================================================
2. CONDUCTOR READOUTS THEY BIND TO (prefer DERIVING over new machinery)
================================================================================
DATA ALREADY FLOWING (orchestration_api.build_orchestration_snapshot, conductor/orchestration_api.py:161-204, served on GET /orchestration via slice_api.py:432, + one-shot CLI):
  tasks{status:n}, total_tasks, waves[{wave,status,n}], frontier[bounded 60], leases, workers[slot rows], gates[100 recent], concurrency{available,caps,pools[]}, wake[12].

MIGRATION-CHAIN SOURCE (the data exists; no reader yet):
  conductor/session_migrate.py emits two events PER hop on the SAME session_id, each via wake.publish(recipient=session_id):
    EVENT_SESSION_MIGRATING="session_migrating" (session_migrate.py:47,129) payload = {session_id, phase=PHASE_START, reason, from_model, to_model, from_cc_session_id, to_cc_session_id, snapshot_id}
    EVENT_SESSION_MIGRATED="session_migrated" (session_migrate.py:48,152) payload adds {reseeded_messages}
  conductor/migration_trigger.py emits EVENT_MIGRATION_REQUIRED="MIGRATION_REQUIRED" (the pre-migrate signal, :48,159).
  These land in the events table (conductor/events.py append_event); query via ControlDB.query_events(event_type=..., limit=...) (used already in orchestration_api wake section :177).
  => A migration-chain readout is DERIVABLE: query_events filtered to the three migration event_types, grouped by session_id, ordered by event_seq, rendered as an ordered hop list (from_model -> to_model, reseeded_messages, relTime). NO new route strictly required if folded into /orchestration; CLEANER is a thin dedicated reader fn (build a chain section) OR a new GET /migrations route mirroring the wake-events pattern. RECOMMEND: add a `migrations` section to build_orchestration_snapshot (one more filtered query_events call, same idiom as `wake`), surfaced as a new block on Orchestration.svelte OR Memory.svelte. Avoid a separate screen (N-03 lean).

PER-LANE ROLLUP SOURCE: already in the snapshot. Two derive options:
  - from snap.workers (group the slot rows by harness or by pool): live/idle/cooldown counts per lane. Requires adding `pool` to _worker_rows SELECT (the column now EXISTS post-006) OR grouping by the existing `harness` field (claude/devin-free/free-api map to harnesses today).
  - from snap.concurrency.pools (already pool-keyed inflight/cap): a per-lane inflight/cap + at_cap rollup is nearly free.
  RECOMMEND: derive the per-lane rollup from snap.concurrency.pools (already live post-006) + optionally enrich _worker_rows with the pool column for a harness/lane label. Smallest new machinery.

GATE TIMELINE: the gate rows are already ORDER BY started_at DESC (orchestration_api._gate_rows:82-98). "Timeline" semantics are satisfiable by grouping/labeling the existing time-ordered rows; no new query needed.

================================================================================
3. 5.3 CONCURRENCY/LIMITS CONFIG UI (D-16 owner-level cap) — THE PERSIST SEAM
================================================================================
WHERE THE CAP LIVES TODAY (read-only, no write path):
  ~/teikoku/workers.toml — owner-measured envelope, NUMBERS/STRINGS ONLY (the file header forbids key values). Per-pool max_concurrency: [devin]=3 (fallback 2), [free_api.groq]=3, [free_api.pollinations]=3, [free_api.cerebras]=1, [free_api.gemini]=1, [free_api.mistral]=2, [free_api.openrouter]=1, [codex] p6-deferred (no cap).
  conductor/pool_concurrency.py:load_pool_envelope (:69-82) parses workers.toml via tomllib into PoolEnvelope{pools{name:PoolLimit{max_concurrency,cooldown_seconds}}}. _collect_pools recurses one level ([free_api.groq] -> pool "free_api.groq").

WHAT THE SCHEDULER READS AT RUNTIME (so a UI change actually takes effect):
  conductor/scheduler.py:13,56,107 — DependencyFrontierScheduler holds a pool_envelope and calls pools_at_capacity(self.db, self.pool_envelope) on the acquire loop (pool_concurrency.py:125-135). The envelope is CONSTRUCTED ONCE at conductor.py:318 (run_schedule, load_pool_envelope(args.workers_toml)) and conductor_loop.py:79. So the cap is read at SCHEDULER CONSTRUCTION (per wave-loop boot), not per-acquire from disk. A UI edit to workers.toml takes effect on the NEXT loop construction; a KV override would need the scheduler to consult the KV at acquire time OR at construction.

THE D-16 PERSIST DECISION (material; recommended/safe pick documented):
  WRITING workers.toml LIVE IS RISKY: the file is owner-measured ground truth + carries source/measured/key_live provenance comments tomllib round-trips poorly (comment loss), and it is the cutover-tracked artifact. RECOMMEND NOT writing workers.toml from the UI.
  SAFER TARGET = the control_state KV (conductor/migrations/003_control_state.sql, key TEXT PK + value_json + updated_at), which is the ESTABLISHED writable persist seam. conductor/session_toggles.py is the canonical accessor model: set_toggle/set_toggles upsert (INSERT ... ON CONFLICT, :142), get_toggles SELECT WHERE key LIKE, keyed "session:<sid>:<name>". A concurrency-cap override would mirror this with an OWNER/GLOBAL-scoped key (e.g. "pool_cap:<pool>" or "owner:concurrency:<pool>"), NOT session-scoped.
  WIRING FOR EFFECT (D-030, must be non-dark): the scheduler must consult the KV override. Two options:
    (i) load_pool_envelope reads workers.toml AS THE FLOOR, then OVERLAYS any control_state pool-cap overrides (the override caps the measured value, never raises it past a measured-safe ceiling — a guard so a UI cap cannot exceed the rate-limit-safe envelope). This keeps workers.toml as the source-of-truth file and the KV as the owner policy layer. The scheduler picks up the merged envelope at construction.
    (ii) pools_at_capacity consults the KV per-acquire (live effect, no reconstruct). Heavier; (i) preferred for simplicity + matches the construction-time read already in place.
  IS IT PER-POOL? YES — caps are per-pool (devin / free_api.groq / free_api.pollinations / free_api.cerebras / free_api.gemini / free_api.mistral / openrouter). The UI edits per-pool caps within the measured ceiling.
  THE WRITE ROUTE: a NEW slice_api POST /config/pool-caps (mirroring POST /chat/toggles, slice_api.py:502) -> a setter in a new tiny module (or extend session_toggles with an owner-scoped variant) writing control_state. A NEW GET /config/pool-caps returns workers.toml caps merged with current overrides (read for the UI).
  THE SIGNAL SURFACE: compose existing Toggle + WorkerSelect + Button + Tile. Per-pool rows (pool name + measured cap + an owner-override input/stepper, bounded <= measured). NO new component. Lives as a NEW config view (App.svelte View union + a rail [settings]-area item + {:else if} branch) OR a block on Orchestration. Recommend a small dedicated config surface (settings-area), since D-16 is owner-level not monitoring.

PRODUCTION_HOOK (5.3): new pub fns = the KV setter/getter (Python) + the envelope-overlay in load_pool_envelope. WIRE THEM: the overlay is called by conductor.py:318 / conductor_loop.py:79 (already the envelope construction site — the seam exists), and the POST route handler calls the setter. The auditor RE-RUNS python_wire_check.py on the changeset (NOTE: python_wire_check has the DEFERRED RecursionError on .memory-venv crawl per handoff T1 — scope --files to the changed conductor/*.py only, which is the documented workaround). A cap override that never reaches pools_at_capacity is DARK = BLOCK.

================================================================================
4. 5.2 §10 STATE-COVERAGE MATRIX (the sweep checklist; auditor verifies on CDP captures)
================================================================================
DESIGN §10 states: empty(focal next action) / loading(--surface skeleton, nothing <200ms) / error(inversion chip + recovery, NEVER red) / populated; LIVE views add disconnected(warn triangle + retry) + stale/cached.
Verified by grep of each surface (state branches read live):

Surface          | empty                  | loading        | error                  | populated | disconnected (live) | stale (live)
-----------------|------------------------|----------------|------------------------|-----------|---------------------|-------------
Orchestration    | PRESENT (3 empty-inset + pending Tile :152-158) | PRESENT (skel :137-143) | PRESENT (warn Tile+Retry :144-151) | PRESENT | MISSING* | MISSING
Memory           | timeline empty -> VERIFY (search has states; check the timeline-empty branch) | PRESENT (:227) | PRESENT (warn+Retry :231-236) | PRESENT | MISSING* | MISSING
Harness          | PRESENT (pending Tile :89) | PRESENT (:72-78) | PRESENT (warn+Retry :79-84) | PRESENT | N/A (static config, not live) | N/A
ChatPane         | PRESENT (.welcome focal :164-167, NOT bare no-data) | PARTIAL (no turn skeleton; turns are sync) | PRESENT (turnerr fail chip + Dismiss :206-210; per-turn fail :176) | PRESENT | GAP: long-poll drop = SILENT pause (ChatPane.svelte:127 "pause ... do not spin on a down backend") NO disconnected mark | MISSING
SessionList      | PRESENT (emptystate :104-107) | PRESENT (skel :93-98) | PRESENT (fail "offline" + Retry :99-103) | PRESENT | GAP: no disconnected mark on the children long-poll drop | MISSING
App shell (Tile) | PRESENT (pending :170 "No run yet") | PRESENT (:158) | PRESENT (warn+Retry :163-167) | PRESENT | N/A | N/A
NEW P5-C migration-chain block | TBD (author with empty) | TBD | TBD | TBD | TBD | TBD
NEW P5-C per-lane rollup | TBD | TBD | (inherits Orchestration error) | TBD | inherits | inherits
NEW P5-D config UI | empty(no overrides) TBD | TBD | TBD (write-fail state) | TBD | N/A | stale-after-write TBD

*MISSING disconnected/stale: Orchestration + Memory are REFRESH-on-mount (no long-poll), so "disconnected" maps to their error state on a failed fetch (arguably covered). The TRUE live-view gap is ChatPane + SessionList long-poll DROPS, which currently retry SILENTLY with no disconnected indicator (warn triangle + retry per §10). This is the primary 5.2 sweep FIX: add a disconnected mark (StatusMark warn + a quiet retry/auto-reconnect caption) when the /events/wait long-poll drops, on ChatPane (the migrating-feed loop) and SessionList (the children-roster loop). "stale/cached" can be a --dim "as of <relTime>" caption on the last good snapshot.

CONSISTENCY (zero per-screen drift): all six screens import $lib/components/index.ts only + tokens.css (verified pattern; prior P3-E/P4-E audits certified zero hex). The sweep re-verifies: no re-implemented StatusMark/Tile/SessionRow/Composer, no literal hex, no magic px beyond precedent layout sizing.

================================================================================
5. SIGNAL DESIGN + WIRING (PRODUCTION_HOOK per new readout/surface)
================================================================================
TOKENS/COMPONENTS: every new block composes the existing index (StatusMark squares §6, Tile, Button, QMark, Toggle, WorkerSelect). FAIL = light inversion chip NEVER red; migration = live/done marks NEVER red (B-06/B-07). One accent --accent, 2px --r, no glow (B-02), Hanken for human / JetBrains Mono for data only (B-03). Bounded scroll (.rows.scroll, P2-C lesson) on any new list.

PRODUCTION_HOOK seams (the new surfaces are Svelte views + TS api seams + Python readers):
  - migration-chain readout: SEAM = orchestration_api.build_orchestration_snapshot (add a `migrations` section via query_events) -> served on existing /orchestration -> a new block in Orchestration.svelte (or Memory.svelte). The Svelte block wires via the existing snap binding (no new route needed). If a dedicated route is chosen: slice_api do_GET dispatch (:432 pattern) + a #[tauri::command] in src-tauri/src/lib.rs + generate_handler! registration (the P3-E/P4-E precedent).
  - per-lane rollup: SEAM = derive in build_orchestration_snapshot from concurrency.pools/workers (a pure helper) -> new block in Orchestration.svelte reading snap. No new route.
  - 5.3 config UI: SEAM = new POST/GET /config/pool-caps in slice_api do_GET/do_POST (:432/:502 pattern) + KV setter/getter (control_state) + load_pool_envelope overlay called at conductor.py:318 / conductor_loop.py:79 (the existing envelope-construction seam). Optional #[tauri::command] for native transport. The new config view wires into App.svelte (View union + rail item + {:else if} branch).
  CDP DESIGN PASS: per visual-ui-loop.md — deviceScaleFactor 1, fromSurface:true, --font-render-hinting=none, kill motion + settle 2 rAF; the AUDITOR authors fresh captures (signal has no screenshots/ dir; P3-E/P4-E precedent: auditor drives its own captures for empty/loading/error-offline/populated). Schema critique pass|fail|unknown (unknown=fail), per §10 state per surface.
  NEW TESTS: per readout — a reader unit test (build_orchestration_snapshot migrations section; per-lane rollup helper) + the 5.3 KV round-trip + the envelope-overlay-reaches-scheduler test (R10 the cap actually caps). The auditor owns the R10 integration smoke (a cap set via the UI route is honored by pools_at_capacity on a real acquire).
  BUILD GATES: bun run check (svelte-check), bun run build (vite), cargo check (src-tauri), pytest on conductor changes; python_wire_check.py --files <changed conductor .py> (scope to changed files to dodge the deferred RecursionError per handoff T1).

================================================================================
6. SCOPE GUARD + RUN ORDER
================================================================================
N-03 (no spectacle): PASS — every new block COMPLETES the existing LEAN surfaces (rows/bars/marks); NO graph viz / force layout / 3D / canvas / new dep. The migration-chain is an ordered ROW list, the per-lane rollup is a COUNT/bar block, the config UI is per-pool rows + steppers. NO new design language (N-06): all from DESIGN-TEIKOKU SIGNAL tokens + component index.
N-02 (no gate rebuild): PASS — 5.3 wiring does not touch frozen checkers; python_wire_check scoping is the documented workaround, not a checker edit.
LEAN: prefer DERIVING (migration-chain + per-lane rollup from the existing snapshot/events) over new routes; only the 5.3 config UI genuinely needs a new write route (caps have no write path today).

RUN ORDER (within this group; 5.2 audits the others so it runs LAST):
  P5-C (5.1 monitoring completion: migration-chain readout + per-lane rollup + verify token/concurrency/gate-timeline)
  -> P5-D (5.3 config UI: control_state KV persist + envelope overlay + new surface) — write-set may overlap Orchestration with P5-C, so SEQUENCE after, do NOT parallel.
  -> P5-E (5.2 §10 sweep: certify ALL surfaces incl the two new ones; the primary FIX = disconnected/stale marks on the ChatPane + SessionList long-poll drops; consistency zero-hex re-verify).

DEFER (owner/creds): gpt-5.5 token window number (placeholder kept); nothing here touches P6/codex-vm.

================================================================================
STATUS: DONE
FILES TOUCHED: F:\.ELAI_workflow\workflow\reasoning\teikoku-p5-monitor-context.md (this report, created — the ONLY write; READ-ONLY on all distro + plan files; no git, no services started)
VERIFIED (read live in distro teikoku): Orchestration.svelte (5 readout blocks, gate-rows time-ordered, concurrency fail-soft+live branches, worker-slot rows not pool-grouped); orchestration_api.py (_worker_rows SELECT omits pool col, _gate_rows ORDER BY started_at DESC, _concurrency fail-soft catch, build_orchestration_snapshot sections); session_migrate.py (session_migrating/migrated event payloads + recipient=session_id publish); migration_trigger.py (MIGRATION_REQUIRED); workers.toml (per-pool max_concurrency, NUMBERS-ONLY read-only file); pool_concurrency.py (load_pool_envelope tomllib read, pools_at_capacity, scheduler construction-time read at conductor.py:318/conductor_loop.py:79/scheduler.py:107); session_toggles.py (control_state KV upsert accessor = the writable persist model, migration 003); slice_api.py route dispatch (do_GET/do_POST :419-505); App.svelte (View union + railItems + {:else if} router); §10 state branches across all 6 surfaces (grep'd).
UNVERIFIED: live render of concurrency available:true post-006 (read the fail-soft+live code paths, did not drive a live /orchestration capture — that is the auditor's CDP pass); Memory.svelte timeline-EMPTY branch (read search-state branches + the LOG render, did not isolate the empty-timeline branch line — flag for the 5.2 sweep); whether a dedicated /migrations route vs a snapshot section is preferred (a P5-C executor design call, both viable).
NOTES: 2 genuine P5-C deltas = migration-chain readout + per-lane rollup (both DERIVABLE from existing data). 5.3 key finding = caps live in read-only workers.toml; safe persist = control_state KV overlay on load_pool_envelope (not a live workers.toml write). 5.2 primary fix = disconnected/stale marks on ChatPane+SessionList long-poll drops (currently silent retry).

TEIKOKU PHASE 5 SCOPE — DASHBOARD COMPLETION + HARDENING + CUTOVER
context-compiler READ-ONLY scoping, task_id=P5-SCOPE, 2026-06-21
Source of truth: F:\.ELAI_workflow\workflow\reasoning\TEIKOKU-MASTER-PLAN.md (Phase 5 = lines 206-217; Section 0 D-01..D-17 = lines 13-32; Section 2 N-01..N-06 = lines 73-78).
Current state read live from WSL distro teikoku (~/teikoku). P0-P4 SHIPPED, committed 1fc4c13, remote main=1fc4c13.

================================================================================
1. PHASE 5 SPEC (verbatim from the master plan)
================================================================================
Master plan lines 206-217, quoted exactly:

  PHASE 5 — DASHBOARD COMPLETION + HARDENING + CUTOVER   [dep: P4]
  Objective: full monitoring surface, state coverage, and cut over from F:.
  5.1  Monitoring completion: accurate token/usage meter, gate-verdict timeline, handoff/migration chain, concurrency view, per-lane status.
  5.2  Full state coverage per DESIGN-TEIKOKU.md §10 (empty/loading/error-inversion/offline) on every surface; standard design pass (no banned tells; contrast gates) PLUS a consistency pass (every screen composes the shared tokens/components; zero per-screen drift).
  5.3  Concurrency/limits config UI (the hybrid toggles, D-16).
  5.4  ELAI-finish dry run THROUGH TEIKOKU: run a real autonomous wave on ELAI (the first JOB) to validate end-to-end; record results.
  5.5  Cutover plan: promote the WSL copy to the canonical home; F: becomes archived fallback.
  done: dashboard drives+monitors both lanes fully; an ELAI autonomous wave runs through TEIKOKU; cutover documented.

  CONTEXT TO FEED (phase 5): this plan; DESIGN-TEIKOKU.md (FULL) + workflow/design/mockup/teikoku-signal.html (visual ref); visual-ui-loop.md; design-taste.md; all prior-phase surfaces.
  RISKS: cutover data/path drift; the ELAI-finish wave surfacing engine gaps.

DESIGN-TEIKOKU §10 STATE COVERAGE (the 5.2 spec, quoted from ~/teikoku/plan/DESIGN-TEIKOKU.md:109-110):
  10 STATE COVERAGE (a surface is not done until every state is designed)
  empty (a focal next action, not a bare "no data"), loading (a calm --surface skeleton, nothing under ~200ms), error (inversion chip + recovery, never red), populated; live views add disconnected (warn triangle + retry) and stale/cached. All states hold the palette + type + no-glow + high-contrast discipline.

What each clause concretely requires:
- "dashboard completion" (5.1+5.2+5.3) = finish the MONITORING surface (the five 5.1 readouts), prove EVERY surface holds all §10 states, and add the D-16 concurrency/limits config UI. NOT new surfaces — completion of the four shipped ones (Orchestration/Memory/Harness/Chat) + the shell.
- "hardening" (NOT a numbered sub-phase; the implied work behind 5.1/5.2 + RISKS + the carry-forwards) = close the latent defects so the completed surfaces and the ELAI-finish wave run clean.
- "cutover" (5.5) = produce a documented plan that promotes ~/teikoku to canonical and demotes F: to archived fallback, plus the ELAI-finish dry run (5.4) that proves TEIKOKU can operate a real project before cutover is declared.

================================================================================
EXISTING vs MISSING (live state, so we COMPLETE not REBUILD — N-02/N-06)
================================================================================
SHIPPED surfaces (~/teikoku/signal/src/lib/screens): ChatPane, SessionList, Orchestration, Memory, Harness, Gallery. Shell = App.svelte (IconRail + collapsible SESSION/MONITOR panels + workspace chat + live Tile + UsagePopup). 16 components in lib/components (TokenMeter, UsagePopup, StatusMark, Composer, WorkerSelect, Toggle, etc). 5 api seams + 15 tauri commands.

5.1 monitoring, item by item (EXISTING vs MISSING):
- accurate token/usage meter: TokenMeter + UsagePopup EXIST and are wired (ChatPane, App usageSections read slice.ctx_used/ctx_cap/session_total). P4-D shipped the per-model window-source fix (kills base-cc flat-200k). MISSING/VERIFY: gpt-5.5 window is an OWNER-OVERRIDE-PENDING placeholder; "accurate" = confirm the reader against the improved CC token counter end-to-end on a live turn (5.1 calls it "accurate").
- gate-verdict timeline: EXISTS as rows in Orchestration.svelte (GateRow, gateMark, "gate verdicts" block, PASS=filled / FAIL=inversion chip). VERIFY it satisfies "timeline" (currently recent-rows, not an explicit chain).
- handoff/migration chain: PARTIAL. ChatPane has the live migrating indicator (P4-E, teal mark+bar never red); Memory surface indexes handoffs. MISSING: a dedicated migration/handoff CHAIN readout (the lineage of migrations within a chat) on a monitoring surface.
- concurrency view: EXISTS in Orchestration.svelte (_concurrency: per-pool inflight/cap segmented bars). BROKEN-SOFT: the live control.db is PRE-migration-006 (worker_slots has NO `pool` column; verified PRAGMA = slot_id/harness/model/model_tier/account_id/max_concurrency/state/cooldown_until/last_error_category). orchestration_api._concurrency CATCHES the OperationalError and degrades to {available:false}. So the concurrency view ships but NEVER shows live pools until migration 006 applies. HARDENING ITEM.
- per-lane status: PARTIAL (worker slots rows exist). VERIFY a per-lane (claude/devin-free/free-api/codex) status rollup is present per 5.1.

5.2 state coverage + design + consistency: the four surfaces each shipped with an audited SIGNAL design pass + cross-screen consistency (zero hex) per prior audits (P3-E captured forced-offline [light inversion+Retry, never red]; P4-E ALL items pass). 5.2 is a COMPLETION audit across ALL surfaces at once (empty/loading/error-inversion/offline + disconnected/stale on live views), not net-new screens. MISSING: a single consolidated §10 coverage + consistency sweep certifying EVERY surface (including the two newest, ChatPane/SessionList) holds every state.

5.3 concurrency/limits config UI (D-16 hybrid toggles): MISSING as a UI. The data exists (workers.toml caps: devin/free-api max_concurrency=3, fallback tiers 1-2; session_toggles.py has 5 per-session toggles but NOT the owner-level concurrency CAP). D-16 = "owner sets pool + model choices + concurrency cap (toggles); auto-scheduler dispatches within that envelope." The per-SESSION model-pool picker exists (WorkerSelect/model_pool.py); the owner-level CONCURRENCY-CAP config surface does not.

5.4 ELAI-finish dry run: NOT RUN. corpus_root for elai is /mnt/f/.ELAI_workflow/docs (mounted, NOT copied — see projects.toml). Needs the live engine + a real wave on the elai project.

5.5 cutover doc: DOES NOT EXIST (find -iname *cutover* = empty). COPY-MANIFEST.md exists at ~/teikoku/COPY-MANIFEST.md (the P0.1 copy record), the cutover input.

CARRY-FORWARD hardening items (all verified live this scope):
H-a python_wire_check.py RecursionError: present at ~/teikoku/.claude/tools/python_wire_check.py AND .../harness/elai/.claude/tools/ AND a copy under conductor/audit_fixtures/. Crawls .memory-venv/audit_fixtures -> crashes even on one file. Blocks the stricter D-030 gate the auditor wants to run on P5 changesets.
H-b native signal_events_wait empty->"*" sentinel: CONFIRMED at signal/src-tauri/src/lib.rs:261-269 — `let recipient_arg = if recipient.is_empty() { "*" } else ... }`. Wake matches "*" as a LITERAL recipient, not a wildcard (latent: every shipped surface passes a concrete sessionId, so it never fires today).
H-c slice_api chat_turn 85L extract: CONFIRMED slice_api.py:245 def chat_turn(...) ~85 lines (MR-3 long-fn WARN; helper extraction).
H-d the 2 OTHER SerializedIntegrator sites default rust: CONFIRMED conductor.py:346 (standalone CLI, comment "default to the rust pack for back-compat") + temporal_backbone.py:279. Both bypass pack-awareness — thread the active_packs the way P3 R10 fixed the loop site.
H-e COPY-MANIFEST corpus gap: docs/specs, thefinalglance, ELAI-FEATURE-MAP, memory-bank, project CLAUDE.md ABSENT in the distro (only harness/elai/.claude present; corpus reached via /mnt/f mount). This is the operate-ELAI/cutover input for 5.4/5.5, NOT a generic copy task.
H-f conductor_db.py harness CHECK still lists legacy behaviorless 'devin-cli' (no devin.exe path exists post-P4). Cosmetic; optional cleanup.
H-g gpt-5.5 window OWNER-OVERRIDE-PENDING (conservative 128k placeholder). DEFER (owner input).

================================================================================
2. SUB-PHASE DECOMPOSITION (dependency-ordered, C-02 cycle each unless flagged pure-docs)
================================================================================
Grouping logic: hardening FIRST (the gate + the seam defects must be clean before the auditor can certify the surfaces and before the ELAI-finish wave runs on real machinery), then dashboard-completion (5.1/5.2/5.3 share the monitoring surfaces — group by write-set), then the live dry run (5.4) which depends on a clean engine, then cutover docs (5.5) last. Code sub-phases run the full context-compiler -> executor -> auditor cycle; pure-docs sub-phases skip the cycle (H-10 E-3 of orchestrate-protocol).

P5-A  HARDENING: gate + engine seams (CODE; first — unblocks the auditor + the wave)
  objective: close H-a/H-b/H-d so the stricter D-030 wire-check runs on P5 diffs, the events-wait wildcard is correct, and both standalone integrator entrypoints are pack-aware.
  existing vs missing: machinery exists; these are defect fixes (python_wire_check recursion guard / dir-exclude; lib.rs:261 wildcard semantics; conductor.py:346 + temporal_backbone.py:279 thread active_packs from project_resolver as the loop site already does).
  done: python_wire_check runs clean on a single distro .py file (no RecursionError); events-wait with empty recipient matches all recipients (or the sentinel is removed and callers pass a real wildcard the wake layer understands) proven by a 2-recipient wake test; both integrator sites resolve the project's packs (a non-rust project's integrator is NOT rust-defaulted) proven by a demo-project drive.
  folds: H-a, H-b, H-d.
  map: hardening.

P5-B  HARDENING: migration-006 apply + concurrency live (CODE; depends P5-A clean gate)
  objective: bring the live control.db to migration 006 so the concurrency view shows live per-pool inflight/cap instead of the fail-soft degraded state; the engine applies 006 on next boot.
  existing vs missing: 006_worker_pool_and_harness.sql EXISTS but is unapplied to the live db (worker_slots has no pool col). Decision the executor faces (material? no — it is the engine's own forward-migration path): apply 006 to the live db via the engine boot path (NOT a hand ALTER), verify orchestration_api._concurrency returns available:true with real pools.
  done: PRAGMA table_info(worker_slots) includes `pool`; the Orchestration concurrency block renders live pools (not the "schema pending" degrade) on the auditor's own capture; fail-soft path retained for fresh dbs.
  folds: the concurrency-view BROKEN-SOFT item from 5.1.
  map: hardening + dashboard-completion (5.1 concurrency view).
  NOTE: touches the live db (state). Reversible (control.db.p5-backup like the existing .p4a/.p4b backups). NOT a tripwire if backed up first; flag if the executor finds 006 has destructive intent (it does not — additive pool column).

P5-C  DASHBOARD COMPLETION: 5.1 monitoring completion (CODE; depends P5-B)
  objective: complete the five 5.1 readouts — accurate token/usage meter (verify live), gate-verdict TIMELINE (upgrade recent-rows to a chain if needed), handoff/migration CHAIN readout, concurrency view (now live from P5-B), per-lane status rollup.
  existing vs missing: 4 of 5 exist in some form (token meter, gate verdicts, concurrency, worker rows); MISSING = a dedicated handoff/migration chain readout + a per-lane status rollup + confirming "accurate"/"timeline" semantics. COMPLETE the Orchestration + Memory surfaces; do NOT add a new screen (N-03 lean).
  done: all five 5.1 items present and reading REAL data on the auditor's own captures; gpt-5.5 window left as a flagged placeholder (DEFER, owner).
  folds: 5.1 gaps.
  map: dashboard-completion.

P5-D  DASHBOARD COMPLETION: 5.3 concurrency/limits config UI (CODE; depends P5-B; write-set may overlap P5-C -> sequence after, not parallel)
  objective: the D-16 owner-level concurrency-CAP + pool/model-choice config surface (compose existing Toggle/WorkerSelect/Composer components; reads/writes the envelope workers.toml caps + a control-plane setting).
  existing vs missing: per-session toggles exist; the OWNER-level concurrency cap config UI is MISSING.
  done: owner can set the concurrency cap + pool choices from the dashboard; the value reaches the scheduler envelope (wired, not dark — D-030); SIGNAL design pass + consistency.
  folds: 5.3.
  map: dashboard-completion.

P5-E  DASHBOARD COMPLETION: 5.2 full §10 state-coverage + consistency sweep (CODE-LIGHT/AUDIT; depends P5-C, P5-D so the final surfaces are in place)
  objective: certify EVERY surface (shell, ChatPane, SessionList, Orchestration, Memory, Harness + the new P5-C/P5-D additions) holds empty/loading/error-inversion/offline + disconnected/stale (live views), one token set, zero per-screen drift; fix any gap found.
  existing vs missing: surfaces shipped with per-surface design passes; this is the consolidated COMPLETION sweep across all of them at once (5.2's explicit "every surface" + consistency).
  done: a single coverage matrix (surface x state) all-pass on the auditor's own pinned CDP captures per visual-ui-loop.md (deterministic diff + schema critique, unknown=fail); zero hex / zero re-implemented component.
  folds: 5.2.
  map: dashboard-completion.

P5-F  ELAI-finish DRY RUN through TEIKOKU (CODE/OPS; depends P5-A..E green engine + clean gate)
  objective: 5.4 — run a REAL autonomous wave on the elai project through TEIKOKU end-to-end, record results; this is the first JOB (N-05) run THROUGH the harness.
  existing vs missing: engine + headless runner + elai project registered (corpus via /mnt/f). MISSING: the actual recorded run.
  done: an autonomous wave on elai completes a task batch through TEIKOKU's real entrypoints with off-agent gate verdicts recorded; results written to the handoff + a 5.4 results doc. RISK (plan line 217): the wave surfaces engine gaps -> log them, do not silently patch outside scope.
  folds: 5.4 + exercises H-e (corpus reach).
  map: cutover-prep (validates before cutover).
  DEFER FLAG: needs LIVE creds — plan-OAuth for a claude-lane wave (expired before per the P3 L5 / P0.5 deferral) OR the free-API/devin pool live. If creds cannot be refreshed autonomously, this becomes a DEFER candidate (owner asleep): run what is runnable (free-API/keyless pool if available), record, and flag the claude-lane portion. Do NOT start the dormant codex-vm (N-05/P6).

P5-G  CUTOVER PLAN (PURE-DOCS; no C-02 cycle; depends P5-F results)
  objective: 5.5 — document the cutover: promote ~/teikoku to canonical, F: -> archived fallback. Address the COPY-MANIFEST corpus gap (H-e: which ELAI corpus must move from the /mnt/f mount into the distro vs stay mounted), data/path drift (RISK line 217), and a go-live checklist.
  existing vs missing: COPY-MANIFEST.md exists; a CUTOVER doc does not.
  done: workflow/handoff or a cutover doc records the promotion steps + the corpus migration decision + the path-sweep checklist + rollback (F: stays as fallback). Documentation only — the ACTUAL promotion is an owner-gated irreversible op (TRIPWIRE).
  folds: 5.5 + H-e + H-c/H-f as a "remaining-debt" appendix.
  map: cutover.
  DEFER FLAG: the doc is authored autonomously; EXECUTING the cutover (actually demoting F:) is an irreversible op = TRIPWIRE, owner-gated. Scope P5-G to the PLAN, not the act.

(Carry-forward H-c chat_turn 85L extract + H-f devin-cli harness label: low-priority; fold into P5-A as opportunistic MR-3 cleanup if write-set allows, else list as remaining debt in P5-G. H-g gpt-5.5 window = DEFER, owner.)

================================================================================
3. CUTOVER — what it concretely means
================================================================================
Plan line 213: "Cutover plan: promote the WSL copy to the canonical home; F: becomes archived fallback." Line 208 objective: "...cut over from F:." Line 214 done: "...cutover documented."

So cutover = a DOCUMENTED migration making ~/teikoku (WSL ext4) the canonical workflow home and F:\.ELAI_workflow the archived fallback. It is NOT (by the plan's own words) a literal data move executed this phase — the done-criterion is "cutover DOCUMENTED" + the ELAI wave proving operability. Concretely it folds three threads:
- the COPY-MANIFEST corpus gap (H-e): the ELAI plan/spec corpus (docs/specs, thefinalglance, ELAI-FEATURE-MAP, memory-bank, project CLAUDE.md) currently lives only on the /mnt/f mount (projects.toml corpus_root=/mnt/f/.ELAI_workflow/docs). Cutover must decide: copy it into the distro (true independence from F:) vs keep it mounted (F: as live fallback). This is the "operate-ELAI" input flagged across the handoff.
- TEIKOKU operating a real project end-to-end = the 5.4 dry run (P5-F) is the PROOF that cutover is safe; cutover-documented without a successful wave is hollow.
- a migration-FROM-ELAI's-harness path-sweep + go-live checklist (D-12 "all paths in rules/skills/tools get swept to WSL paths").

DEFER candidates inside cutover (owner asleep):
- the ACTUAL promotion (demoting F: to fallback) = irreversible op = TRIPWIRE -> owner-gated; P5-G produces the PLAN only.
- the corpus copy-vs-mount decision is a material fork IF it implies deleting/moving F: content; the safe autonomous pick = document copy-into-distro as recommended, keep F: mounted as fallback, do not delete F:.
- LIVE creds for the 5.4 claude-lane wave (P0.5/P3-L5 deferral history) -> DEFER the claude-lane portion if creds cannot auto-refresh.

================================================================================
4. SCOPE GUARD CHECK (N-01..N-06)
================================================================================
N-01 no sidecar/local-model/quant/training: PASS — no sub-phase touches the ELAI sidecar or any model. TEIKOKU only orchestrates. (P5-F runs a wave that DRIVES models via the pool; it does not build/train them.)
N-02 no gate rebuild from scratch: PASS — P5-A FIXES python_wire_check (a recursion bug) and applies the already-stricter D-06 gate; it does not rebuild the frozen checkers. The fix must NOT weaken the gate (R8 freeze still applies; if the recursion fix touches a frozen file, that is a material fork -> the fix belongs in the distro wrapper/exclude config, not the frozen checker body).
N-03 no canvas/3D/graph-viz spectacle: PASS — P5-C/D/E COMPLETE the existing LEAN surfaces (Orchestration is explicitly "NO graph viz / force layout / 3D / canvas / new dep"). No new dep, no spectacle. visual-ui-loop is the standard design pass, not spectacle.
N-04 WSL day-1 = build host + ELAI: PASS — only the elai (+ demo) project is in play; no other project migrated.
N-05 no new ELAI/game product features: PASS — P5-F RUNS ELAI as a JOB through TEIKOKU; it does not add features to ELAI. (Engine gaps the wave surfaces are LOGGED, line 217, not fixed inside ELAI here.)
N-06 no brand/UI beyond DESIGN-TEIKOKU: PASS — every UI sub-phase composes the locked SIGNAL tokens + the shared component index against DESIGN-TEIKOKU.md; zero new design language.

OWNER-GATED / P6 flags inside the plan:
- P5-G cutover EXECUTION (not the doc) = irreversible -> owner tripwire.
- P5-F claude-lane wave + gpt-5.5 window = creds/owner input -> DEFER.
- Nothing in P5 touches P6 / the dormant codex-vm. CONFIRMED no sub-phase starts Codex.

================================================================================
5. RECOMMENDED RUN ORDER + per-sub-phase CONTEXT-TO-FEED
================================================================================
Order (dependency-ordered; max 2 parallel per C-02; most are sequential by write-set):
  P5-A (hardening gate+seams) -> P5-B (migration 006 + live concurrency) -> P5-C (5.1 monitoring completion) -> P5-D (5.3 config UI) -> P5-E (5.2 §10 sweep, all surfaces in place) -> P5-F (5.4 ELAI-finish dry run) -> P5-G (5.5 cutover doc, pure-docs).
  Parallelizable window: P5-C and P5-D both touch monitoring surfaces (write-set overlap on Orchestration) -> SEQUENCE them, do not parallel. P5-A's three fixes (wire-check / lib.rs / integrator) have disjoint write-sets -> a single executor can do all three in one changeset, OR split if it exceeds the ~500k budget.

Per sub-phase CONTEXT-TO-FEED (beyond the standing Phase-5 list: master plan + DESIGN-TEIKOKU FULL + teikoku-signal.html + visual-ui-loop.md + design-taste.md):
  P5-A: ~/teikoku/.claude/tools/python_wire_check.py + the .memory-venv/audit_fixtures crawl paths; signal/src-tauri/src/lib.rs:261-289 + the wake layer (conductor/wake.py, mailbox.py recipient matching); conductor/conductor.py:340-360 + temporal_backbone.py:270-290 + the project_resolver/active_packs threading from the P3 loop-site fix (the canonical example). production_caller_check.py + python_wire_check.py (the gate it must run clean under).
  P5-B: conductor/migrations/006_worker_pool_and_harness.sql; the engine boot/migrate path (conductor_db.py init/migrate); conductor/orchestration_api.py:132-158 (_concurrency fail-soft) + pool_concurrency.py; control.db backup convention (.p4a-backup precedent).
  P5-C: Orchestration.svelte + orchestration_api.py (the 5 readouts); Memory.svelte + memory api (handoff/migration chain source); session_migrate.py/session_snapshot.py (migration chain data); TokenMeter/UsagePopup + the CC token reader; workers.toml (per-lane).
  P5-D: session_toggles.py + model_pool.py + WorkerSelect/Toggle/Composer components; workers.toml caps + the scheduler envelope (pool_concurrency.load_pool_envelope) as the wiring target for D-030; D-16 spec (plan line 31).
  P5-E: ALL six screens + lib/components/index.ts + tokens.css; visual-ui-loop tools (ui-diff.mjs, ax-scaffold.mjs); the prior P3-E/P4-E capture method (CDP pinned). DESIGN §10 (the state list) + §6 (square marks) + the anti-slop ban list (§B-01..B-07).
  P5-F: headless_runner/run_headless + the `headless` verb; projects.toml (elai row, corpus_root /mnt/f); the elai pack gate set; workers.toml pool; the live-creds status (plan-OAuth / .liberty .env free pool). The GPU-availability hook (D-14). DO NOT feed anything that would start codex-vm.
  P5-G: COPY-MANIFEST.md; projects.toml; D-12 (path sweep); the P5-F results; the handoff-protocol.md (cutover is a handoff-style doc). No code context — pure docs.

Pure-docs (NO C-02 cycle): P5-G (cutover plan) — prose only, per orchestrate-protocol H-10 E-3. Everything else (P5-A..F) is CODE/OPS and runs the full context-compiler -> executor -> auditor cycle. P5-E is code-light (mostly audit + targeted state fixes) but still runs the cycle because it may edit components. P5-F is OPS (a real run) but the auditor must certify the run's off-agent evidence (gate verdicts recorded), so it keeps the cycle.

================================================================================
6. R10 / DONE-CRITERION FOR THE WHOLE PHASE 5 + THE FINAL COMMIT
================================================================================
Phase-5 done-criterion (plan line 214, verbatim): "dashboard drives+monitors both lanes fully; an ELAI autonomous wave runs through TEIKOKU; cutover documented."

R10 whole-phase integration gate (per wiring-enforcement R10 + the P3/P4 precedent): an INDEPENDENT auditor drives the REAL composed plane on a HARD input and proves:
  (1) all five 5.1 monitoring readouts render REAL data live (token meter accurate, gate-verdict timeline, handoff/migration chain, LIVE concurrency post-006, per-lane status) on the auditor's OWN pinned CDP captures;
  (2) every surface holds the full §10 state set + zero per-screen drift (the 5.2 matrix all-pass);
  (3) the D-16 concurrency-cap config UI value reaches the scheduler envelope (wired, not dark — production_caller_check/python_wire_check clean on the P5 changeset, which REQUIRES P5-A's recursion fix landed);
  (4) an ELAI autonomous wave completed through TEIKOKU's real entrypoints with off-agent gate verdicts recorded (5.4), OR the runnable portion completed + the claude-lane portion explicitly DEFERRED with the creds reason (owner asleep);
  (5) the cutover plan exists and is complete (corpus decision + path sweep + rollback);
  (6) bindings/wire/secrets clean; non-elai generality still holds (the demo project still routes generic, proving P5-A's integrator fix did not re-hardcode ELAI).
  Missing evidence on any = FAIL (R1). The auditor authors/drives the smoke; the executor does not self-certify (C-04).

Final commit: ONE commit at P5 end after the R10 gate passes (commit-cadence: maintainer voice, lowercase i, no em-dashes, no AI-attribution trailer; no test counts in the body). gitignore keeps audit-scratch/ + state dbs + venvs ignored. Push to remote main (currently 1fc4c13). The ACTUAL cutover (demoting F:) is NOT in this commit — it is the documented, owner-gated next step.

DEFER LEDGER (owner asleep — pick recommended/safe, document, surface at wake):
- gpt-5.5 context window number (H-g) — placeholder kept, flagged.
- 5.4 claude-lane wave if plan-OAuth cannot auto-refresh — run free/keyless portion, defer claude-lane.
- the ACTUAL cutover execution (P5-G is the PLAN; the act is an irreversible owner tripwire).
- never start codex-vm / P6.

================================================================================
STATUS: DONE
FILES TOUCHED: F:\.ELAI_workflow\workflow\reasoning\teikoku-p5-scope.md (this report, created — the only write; READ-ONLY on all distro + plan files)
VERIFIED: master plan Phase 5 (lines 206-217), Section 0 (13-32), Section 2 (73-78) read by range; DESIGN §10 (plan/DESIGN-TEIKOKU.md:109-110) read; live distro state read via wsl -d teikoku (6 screens + 16 components + 15 tauri cmds present; worker_slots PRAGMA = NO pool col so concurrency fail-soft is live; lib.rs:261-269 events_wait "*" sentinel; conductor.py:346 + temporal_backbone.py:279 default-rust integrator sites; slice_api.py:245 chat_turn ~85L; python_wire_check.py present x3; COPY-MANIFEST.md present; NO cutover doc; projects.toml corpus_root=/mnt/f mount). UNVERIFIED: live per-screen §10 state-ref counts (PowerShell quoting blocked the per-file grep loop — relied on prior audit records in the handoff that certified per-surface design passes); the engine boot-migrate path for 006 (read the migration file presence, not the apply logic).

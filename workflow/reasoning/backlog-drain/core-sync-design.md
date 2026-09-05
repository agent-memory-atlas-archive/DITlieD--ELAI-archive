core-sync-design — bidirectional sync between TEIKOKU "core" and the opus backlog-walk

Author: read-only systems analyst pass, 2026-07-16. Read-only: no code changed, no walk/git run.
Scope: how ELAI delivery status reaches the TEIKOKU "core" observation deck today, why it is stale,
and the minimal grounded design for a live + bidirectional sync. Every claim carries a file:line or
command-output anchor; guesses are labelled UNGROUNDED.


================================================================================
1. CURRENT FLOW (how status is computed today, and why it goes stale)
================================================================================

1.1 The one-way pipe (walk land -> ... -> core), end to end

  (a) WALK LANDS A PLAN (WSL, conductor/opus_walk):
      - opus_land.py::main() runs the frozen gates in an ext4 worktree, ff-merges into /mnt/f main,
        pushes, and funnels every terminal verdict through finish() (opus_land.py:161-172). On merge
        it emits OPUS_LAND_MERGED to the conductor control.db event log (opus_land.py:381 via
        _emit(), opus_land.py:57-64).
      - The DELIVERY TRUTH is written by walk_state.mark_plan_done() (walk_state.py:420-451): it
        writes conductor/opus_walk/plan-status/{plan_id}.json = {plan_id, status:"DONE", commit_sha,
        gate_evidence_ref, last_run_iso} (walk_state.py:437-445) and upserts a "drained" row into the
        teikoku-LOCAL disposition ledger via _emit_disposition() (walk_state.py:330-360, called at
        :446). Escalations do the same with verdict "escalated" (walk_state.py:384/append_escalation
        -> _emit_disposition at :384). A DONE row with no commit_sha/gate_evidence_ref is impossible
        (DoneRowRejected, walk_state.py:427-430) — so these artifacts are gate-backed (R4).
      - Sample proof: plan-status/H-28-WIRE-1.json carries status DONE + commit ebb2f50 +
        gate_evidence_ref "OPUS_LAND_GATES_PASS events ... + pushed d1eb94a..ebb2f50".

  (b) WSL -> F: LEDGER SYNC:
      - The teikoku walk cannot write /mnt/f for gated turns (scope_write_gate, opus_land.py:5-8),
        so its dispositions live in the WSL-local ledger. sync_teikoku_dispositions.py reads that
        ledger over the \\wsl.localhost UNC share and upserts each row into the F: canonical ledger
        workflow/status/plan-dispositions.json via plan_dispositions.record()
        (sync_teikoku_dispositions.py:29-73). It is invoked ONLY from inside the feeder run
        (elai_status_feeder.py:588-595, best-effort, 15s timeout, fail-open).

  (c) FEEDER REBUILDS THE SNAPSHOT (F:, Windows):
      - elai_status_feeder.py::main() rebuilds workflow/status/elai-status.json wholesale every run
        (feeder:573-657). The per-letter B..Q tally comes from leg_synthesis() parsing the master
        audit _MASTER-SYNTHESIS.md (feeder:70-115); the live overlay _apply_disposition_overlay()
        promotes "drained" dispositions into that tally's wired count and drops drained ids from the
        open punch (feeder:311-467, invoked :599-608). Backend/frontend/bench percentages are
        leg_progress() (feeder:470-542).

  (d) CONDUCTOR SERVES IT (WSL):
      - GET /elai-status -> slice_api.elai_status_read() (slice_api.py:1649-1650, :43-51) READS the
        F: json only ("never computes ... pure projection", slice_api.py:36-42). ELAI_STATUS_PATH
        defaults to /mnt/f/.ELAI_workflow/workflow/status/elai-status.json (slice_api.py:39-40).

  (e) THE CORE DASHBOARD (signal frontend):
      - signal/src/lib/api/elai.ts::fetchElaiStatus() GETs http://127.0.0.1:8787/elai-status
        (elai.ts:9,~80) and types ElaiSnapshot (elai.ts:24-73). signal/src/lib/screens/
        ElaiStatus.svelte renders the per-letter bars, backend/frontend %, punch/open-gaps, benches.

1.2 WHY IT IS STALE (measured, 2026-07-16)

  ROOT CAUSE — the feeder never re-fires: the Windows scheduled task "ELAI Status Watcher" is
  DISABLED (schtasks query: Status = Disabled, Next Run Time = N/A). That task is the only
  change-trigger that re-runs elai_status_feeder.py (per CLAUDE.md: "maintained by
  elai_status_feeder.py + the 'ELAI Status Watcher' logon task, change-triggered ~2min"). With it
  disabled nothing rebuilds the snapshot.

  Measured staleness cascade (PowerShell LastWriteTime, 2026-07-16):
    - elai-status.json          : 28.1 h old  (snapshot the core reads; git head ce76e4f)
    - F: plan-dispositions.json : 44.4 h old, 76 rows   (last actual feeder/sync run)
    - WSL plan-dispositions.json:  0.6 h old, 97 rows    (the walk is LIVE and dispositioning)
  => 97-76 = 21 terminal dispositions are stranded in WSL, never synced to F:, never in the snapshot,
  never on the core. The walk is healthy; the PROJECTION is dead. Because the sync is only called
  from within the feeder (feeder:588-595), a dead feeder also means a dead WSL->F ledger sync — both
  legs of the pipe stop at the same disabled task.

1.3 WHY IT IS NOT DELIVERY-AWARE (summaries vs journal)

  The core's per-letter bars are a projection of a FROZEN summary, not live per-unit delivery:
    - leg_synthesis() parses plan-audit-2026-07-02/_MASTER-SYNTHESIS.md (feeder:72, snapshot shows
      source_mtime 1782990313 = frozen 2026-07-01). It is a per-LETTER tally (plans/wired/partial/
      dark/not_started), never per-unit. The disposition overlay nudges wired counts but the base is
      the frozen audit.
    - The genuine journal oracle `elai plan status` (I-39, plan_status_cmd.rs) is ALSO not fresh
      per-unit: it derives per-LETTER state (PlanStatusRow.letter, plan_status_cmd.rs:112, resolve
      _letters :175-206) from (1) FINISH fragments frozen at s-plan-audit-2026-06-04
      (plan_status_cmd.rs:61, DEFAULT_FRAGMENTS_BASE), (2) wiring_contract_check.py on the letter's
      INDEX file, (3) a GLOBAL ssb observable_effect COUNT that is explicitly NOT per-letter and does
      NOT influence any verdict (plan_status_cmd.rs:409-419,475-485). So I-39 answers "letter B is
      SHIPPED_WIRED per the June-04 frozen fragments", not "unit B-53 landed 40 min ago".

  Net: neither the core nor I-39 reflects what the walk landed since the last frozen audit. The only
  FRESH, per-unit, gate-backed delivery truth in the system is the walk's own plan-status/{id}.json +
  the disposition ledger (both written by walk_state on every land, gate-guarded).


================================================================================
2. DELIVERY-AWARE SOURCE (can I-39 / journal / SSB give per-unit wired/partial/dark?)
================================================================================

  I-39 `elai plan status --json`  : per-LETTER only; frozen June-04 fragments; SSB leg is global.
                                    Command cost: spawns wiring_contract_check.py per letter with a
                                    120s timeout each (plan_status_cmd.rs:58,340-388) — seconds to
                                    minutes for 8+ letters; a subprocess-heavy path. NOT the per-unit
                                    source as shipped. Reusable only if FINISH is repointed at a live
                                    per-unit source (out-of-scope rework).

  SSB PlanStateDerived rows       : emitted by I-39 (plan_status_cmd.rs:576-637) but carry the same
                                    per-letter frozen derivation; the raw observable_effect rows are a
                                    global health count with no per-letter/per-unit key
                                    (plan_status_cmd.rs:451-458). SSB is append-only, per-session — no
                                    per-unit upsert. Not a better source than the ledger.

  RECOMMENDED AUTHORITATIVE SOURCE — the walk's plan-status/{id}.json + the disposition ledger:
    - plan-status/{id}.json is written ONLY after opus_land's frozen gates PASS and the commit is on
      main (walk_state.py:420-451; commit_sha + gate_evidence_ref mandatory), i.e. it is the R4
      off-agent artifact (a real commit + a gate-evidence ref), not a self-report.
    - the disposition ledger row adds verdict {drained|escalated|blocked|analyzed} + reason +
      evidence_ref (plan_dispositions.py:50,86-120; walk_state.py:330-360). drained == wired-and-
      committed; escalated/blocked == partial debt with a reason.
    - COST: zero compute — these are already written on every land; reading them is a JSON load. This
      is strictly fresher and cheaper than re-running I-39.
    - Per-unit wired/partial/dark mapping: drained->WIRED, escalated/blocked/analyzed->PARTIAL,
      absent->from the frozen audit floor (the overlay already does exactly this,
      feeder:341-347,368-410). Keep I-39 as the periodic FROZEN-AUDIT re-baseline; use the ledger +
      plan-status/ as the LIVE per-unit delta on top. That is the delivery-aware source.


================================================================================
3. LIVE + BIDIRECTIONAL DESIGN (minimal, reuse feeder/route/ledger)
================================================================================

Design principle: the pipe (a)->(e) in section 1.1 is CORRECT and already delivery-aware via the
overlay. It is simply not being PUMPED. Do not add a service. Restore liveness + close the loop.

3.1 (a) ORCHESTRATOR WRITES TRUE STATE TO THE CORE ON EVERY LAND

  The write already happens WSL-local (mark_plan_done + _emit_disposition). Two changes make it reach
  the core promptly:

  FIX-1 (highest leverage, zero code): re-enable the "ELAI Status Watcher" scheduled task. opus_land's
  ff-merge changes /mnt/f (opus_land.py:377 merge, :397 push) and the feeder's own git leg
  (feeder:558-570) sees it; a live watcher re-runs the feeder, which re-runs sync_teikoku_dispositions
  (feeder:588-595) — pulling the 21 stranded rows — and rebuilds the snapshot. This alone restores the
  entire one-way liveness.

  FIX-2 (immediate on-land refresh, small hook): the single land funnel is opus_land.py::finish()
  (opus_land.py:161-172) — every terminal verdict passes through it, and OPUS_LAND_MERGED already
  emits there (:381). Add, in the LANDED branch after the push (opus_land.py:~403, guarded to
  verdict.startswith("LANDED")), a best-effort trigger of the F: feeder so the snapshot refreshes on
  the land instead of waiting for the watcher debounce. Cross-distro: opus_land already writes /mnt/f,
  so the cheapest grounded mechanism is to TOUCH a trigger file the watcher watches (e.g.
  /mnt/f/.ELAI_workflow/workflow/status/.land-trigger) — no Windows-python-from-WSL call needed. The
  WSL-side producer is thus walk_state.mark_plan_done (the truth writer) and opus_land.finish (the
  trigger); the F:-side consumer is the existing feeder. No new writer of elai-status.json — the
  feeder stays the sole author (honesty contract, feeder:7-8).

  Exact hooks to add/change:
    - conductor/opus_walk/opus_land.py::finish() (or the LANDED branch ~:396-405): best-effort touch
      of the F: land-trigger file.
    - .claude/tools/elai_status_feeder.py: no change (already syncs + overlays). Optionally have the
      watcher fire on the .land-trigger file specifically for a tighter debounce.
    - re-enable schtasks "ELAI Status Watcher" (config, not code).

3.2 (b) CORE FEEDS TRUE STATE + PER-PLAN BRIEF BACK TO THE ORCHESTRATOR AT PLAN-START

  The read-back seam ALREADY half-exists: plan_selector.py imports the disposition ledger and calls
  walk_state.dispositioned_ids() to skip settled plans (walk_state.py:454-469). Extend that read at
  plan pickup:
    - TRUE STATE: the orchestrator shells `python .claude/tools/elai_status_query.py --letter <L>
      --json` (elai_status_query.py:107-115) for the authoritative F: tally + punch-for-letter, OR
      reads the shared index directly (section 4). This is the Layer-0 advisory read (never a
      done-gate, elai_status_query.py:7-12) — correct posture for "orient before picking".
    - PER-PLAN SUPER-SUMMARY BRIEF: the orchestrator looks up the plan's row in the shared index
      (section 4) — {does, state, supersedes[], superseded_by[], residual, files[], respect[]} — so a
      fresh Opus turn picks up context WITHOUT re-reading the plan file or re-deriving prior verdicts
      (the exact re-analysis waste plan_dispositions.py:4-9 was built to kill).
    - Hook: conductor/opus_walk/plan_selector.py (the existing disposition consumer) reads the shared
      index at candidate selection; the per-plan brief is injected into the dispatch prompt.


================================================================================
4. THE SHARED INDEX (where the per-unit super-summary lives; how both sides keep it fresh)
================================================================================

  RECOMMENDATION: a single per-unit JSON index, plan-index.json, following the EXACT pattern the
  disposition ledger already proved (durable file, WSL-local + F: canonical, UNC-sync, feeder-
  projected). Do NOT put it in the SSB (append-only, per-session, no per-unit upsert — I-39 already
  showed SSB cannot key per-unit, plan_status_cmd.rs:409-419) and do NOT add a new DB table (the
  ledger file + its sync harness is the proven, stdlib, gitignored substrate).

  Rows keyed by plan_id: {id, does, state, supersedes[], superseded_by[], residual, files[],
  respect[]}. This is a SUPERSET of the disposition row (plan_dispositions.py:108-117 already has
  verdict/reason/evidence_ref/commit_sha/updated) — so cleanest is to EXTEND the disposition ledger
  row with the missing fields (does/supersedes/superseded_by/residual/files/respect) rather than a
  second file, keeping one upsert store and one sync. If a distinct file is preferred for separation,
  mirror the ledger's storage exactly.

  LOCATION:
    - WSL-local (walk writes): conductor/opus_walk/plan-index.json (beside plan-dispositions.json,
      same _disposition_path_for pattern, walk_state.py:323-328).
    - F: canonical (feeder projects): workflow/status/plan-index.json (beside plan-dispositions.json;
      gitignored harness tooling, same posture as plan_dispositions.py:20-22).

  HOW BOTH SIDES KEEP IT FRESH (reusing the exact seam that already works for dispositions):
    - WRITE (walk): extend walk_state.mark_plan_done()/_emit_disposition() (walk_state.py:330-360,
      420-451) to also upsert the plan-index row (does/state/files from the changeset, supersedes/
      residual from the walk's own knowledge) — same atomic-write, same fail-open contract.
    - SYNC (WSL->F): add sync_teikoku_planindex.py cloned from sync_teikoku_dispositions.py:29-73
      (or extend it to carry the extra fields), invoked from feeder main() beside the existing sync
      call (feeder:588-595).
    - PROJECT (F->snapshot->core): add a leg_planindex() to the feeder mirroring leg_dispositions()
      (feeder:545-555) so elai-status.json carries the per-unit index; extend the elai.ts ElaiSnapshot
      type (elai.ts:24-73) and ElaiStatus.svelte to render per-unit rows.
    - READ-BACK (core->orchestrator): plan_selector.py reads plan-index.json at plan-start (section
      3.2), exactly as it already reads dispositioned_ids().

  Freshness invariant: the index is written on the same land event that writes plan-status/{id}.json,
  synced by the same feeder run, projected by the same snapshot, read back by the same selector — one
  event, one pump, both directions. Nothing new to keep alive beyond the already-required watcher.


================================================================================
GROUNDING / VERIFICATION
================================================================================
  - Feeder/query/ledger/sync: read in full (elai_status_feeder.py, elai_status_query.py,
    plan_dispositions.py, sync_teikoku_dispositions.py on F:).
  - I-39 oracle: plan_status_cmd.rs read in full (per-letter, frozen fragments, global SSB — verified
    at cited lines).
  - Walk land/disposition writers: walk_state.py:320-472, opus_land.py full — read.
  - Route + frontend seam: slice_api.py:36-51,1649-1650; elai.ts:1-80; ElaiStatus.svelte (exists,
    per-letter projection) — verified.
  - Staleness: schtasks "ELAI Status Watcher" = Disabled (command output); LastWriteTime deltas
    28.1h snapshot / 44.4h F-ledger(76 rows) / 0.6h WSL-ledger(97 rows) — measured this session.
  - UNVERIFIED: exact ElaiStatus.svelte render internals (existence + projection role confirmed, line
    detail not read); the .land-trigger watch mechanism assumes the watcher supports a file-watch
    trigger (CLAUDE.md says "change-triggered" — mechanism not read line-by-line). Labelled where used.

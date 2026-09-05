F-13 session archive on SSB schema-bump decisions
Session: s-6312 (orchestrate Phase A, in-band executor)
Date: 2026-06-03

WHAT
Archive the stale-version ssb.db (and precautionarily the session_dag.db) BEFORE
ssb::run_migration's destructive DROP, gated wire-dark behind
ELAI_SSB_ARCHIVE_ON_BUMP_ENABLE (default OFF). Add the session_archived SSB
action_type (SCHEMA_VERSION 205 -> 206, TOTAL_VARIANTS 482 -> 483), an
ssb_viewer.session_archives query + SessionArchiveRow, and the dashboard pair
(get_session_archives Tauri command + SessionArchiveDrawer.svelte + session-archive
DrawerKey). EXTEND_EXISTING increment; deliberately NOT the Springdrift Backup-Actor
mechanism (no periodic committer, no git dep, no unified store).

GROUND-TRUTH CORRECTIONS APPLIED (the plan/CLAUDE.md numerics were stale)
- SCHEMA_VERSION live was 205 (ssb.rs), NOT 191 as CLAUDE.md/plan prose said.
  Bumped 205 -> 206 (named at edit time per HARD RULE).
- TOTAL_VARIANTS live was 482 (action_type_lock.rs), NOT 464. Bumped 482 -> 483.
- The SSB add is 8 compiler-load-bearing sites, not the "5/6" the plan listed:
  enum arm, as_sql_str arm, from_str arm, SsbPayload variant, action_type() match
  arm, payload struct, CHECK-constraint literal, and the TWO exhaustive
  test-payload builders (payload_for_action + all_test_payloads). Plus the
  all_test_payloads count assert (402 -> 403), the golden row, and the THREE
  action_type_lock.rs sites (TOTAL_VARIANTS const, ALL_VARIANTS append,
  exhaustive match arm).
- Golden: session_archived sorts alphabetically at ordinal 0374, between
  selective_delegation_escalated (was 0373) and sglang_backend_registered (was
  0374). Inserted via stdlib-only Python alphabetical resort; every subsequent
  ordinal shifted +1; 482 -> 483 data rows. (Matches the golden header procedure.)
- ssb_viewer.session_archives takes a `cap: usize` (the plan's `&self`-only
  signature was wrong; mirrors symbolic_conflicts(&self, cap) which clamps to
  DRAWER_QUERY_HARD_CAP = 500).

THREE HIGH-SEVERITY GAP RESOLUTIONS

GAP 1 -- session_archived row cannot use the async Ssb::write path at migration time.
  run_migration runs inside Ssb::new on the dedicated writer_conn BEFORE the
  async mpsc single-writer task is spawned and BEFORE the
  #[cfg(debug_assertions)] append-only update_hook is attached. RESOLUTION:
  emit the row via a DIRECT synchronous conn.execute INSERT INTO ssb(...) AFTER
  CREATE_SSB_TABLE, inside run_migration (new fn insert_session_archived_row).
  It mirrors execute_write's exact column set (project_id, session_id, agent_id,
  superstep, action_type, payload, created_at) and rmp_serde::to_vec_named
  payload encoding. created_at uses SystemTime::now() epoch-seconds (no clock in
  scope at migration time; matches execute_write's wall-clock semantics). The
  update_hook is attached by Ssb::new AFTER run_migration returns, so the insert
  does not trip the append-only panic. The row is emitted AFTER the archive +
  recreate so archive_path / manifest_blake3 are real, not placeholder
  (gotcha F5_SSB_PAYLOAD_HONESTY).

GAP 2 -- run_migration signature must widen.
  Old: run_migration(conn: &Connection). New: run_migration(conn, db_path:
  &std::path::Path, session_id: &str, project_id: &str). db_path is the archive
  copy source; session_id is the dest path component + the row; project_id is the
  row's project column. All three live on SsbConfig at the SOLE caller (Ssb::new,
  ssb.rs); the widening is fully contained -- one call site updated. No global,
  no shared-slot injection (W-3 does not apply -- the call site is inside Ssb::new
  itself).

GAP 3 -- WAL byte-faithfulness for the blake3-matches acceptance.
  apply_pragmas puts the writer connection in WAL mode (journal_mode=WAL), so
  un-checkpointed pages live in the -wal sidecar; a raw fs::copy of the main db
  file alone could miss them, breaking "the archived ssb-v{N}.sqlite blake3
  prefix matches manifest_blake3". RESOLUTION: run PRAGMA wal_checkpoint(TRUNCATE)
  on the live writer conn BEFORE the fs::copy (inside run_migration, fail-open on
  checkpoint error). This folds WAL pages into the main db file so the copy is
  byte-faithful and the blake3 prefix is stable across a re-read. (rusqlite
  Backup API was unnecessary -- no second writer contends at migration time.)

FAIL-OPEN POSTURE (documented choice)
  Every archive-side failure when the gate is ENABLED is fail-OPEN: log via
  tracing::warn! and CONTINUE the DROP. Blocking the DROP on an archive failure
  would brick session startup for data the migration's own comment marks
  "data loss is acceptable for old sessions". The archive is best-effort
  forensics, not a safety gate. Covered paths: wal_checkpoint failure (archive
  may miss WAL pages, logged), copy failure (DROP continues, no row), digest
  failure (archive kept on disk, row skipped), row-insert failure (archive kept,
  logged). The DROP at ssb.rs runs UNCONDITIONALLY after the archive attempt
  (killer_risk #5 -- additive only; asserted by
  session_archive_drop_preserved_contract.rs for both gate states).

W-2 DISPOSITION (RESOLVED, not hidden) -- EXPLICIT W-2 EXCEPTION
  Decision: KEEP DEFAULT-OFF + register a wire-dark-allowlist.toml row.
  Do NOT flip to default-ON.
  Two options weighed:
    Option A (flip to default-ON, the literal W-2 happy path): WRONG here.
      archive_stale_ssb does a full file-copy of ssb.db on EVERY stale-version
      SSB open. SCHEMA bumps happen ~50x in this repo's history and fire on every
      reused stale DB; default-ON means an unconditional disk-doubling copy on the
      hot startup path for data the code itself marks "acceptable to lose". Real
      cost (disk growth, no GC yet), forensic-only value.
    Option B (keep default-OFF, register allowlist row): CORRECT. This is a
      forensic opt-in, NOT a security/enforcement gate. It is materially different
      from F-3/F-4/F-9 (always-on DEFENSES where default-off = zero protection =
      the antipattern W-2 forbids). F-13 archives data with NO user-facing
      enforcement meaning; the "feature off = unprotected" harm that motivates W-2
      does not exist. W-2's own escape hatch + the F-11 in-file deviation
      precedent (F-0_INDEX.md:210) + R9 (wire-dark-allowlist) exist precisely for
      this case. R9 is SATISFIED by registering the row; the flag is then not a
      graveyard.
  EXECUTOR DID:
    1. Kept ARCHIVE_ON_BUMP_ENV default-OFF (truthy-to-enable: 1/true/yes/on).
    2. Did NOT edit wire-dark-allowlist.toml or .claude/gate-manifest.txt -- those
       are R8-frozen; editing them inside this run would BLOCK on gate_freeze_check
       as "run modified its own gate". The row-add is an OUT-OF-BAND reviewed step
       the controller performs, then re-locks. Proposed row:
         name              = "ELAI_SSB_ARCHIVE_ON_BUMP_ENABLE"
         type              = "experiment"
         owner             = "maintainer"
         expiry_date       = "2026-09-01"
         promotion_plan_id = "F-13"
         smoke_test        = "crates/orchestrator/tests/session_archive_observable_smoke.rs::stale_open_with_gate_enabled_archives_and_emits"
    3. Recorded this explicit W-2 exception here (ACCEPTANCE "W-2 tension RESOLVED,
       not hidden"). Prior art: the F-11 in-file W-2 deviation (F-0_INDEX.md:210).
  CONTROLLER ACTION REQUIRED (surfaced): add the allowlist row + re-run
  `python .claude/tools/gate_freeze_check.py --lock` as a separate reviewed change
  BEFORE the F-13 wiring gate can pass R9.

SESSION-DAG HONESTY CORRECTION (vs source verdict)
  The session-DAG migration is NON-destructive (CREATE TABLE IF NOT EXISTS + lazy
  ALTER in SessionDag::open()), so archive_session_dag is belt-and-suspenders, NOT
  loss-prevention. Only SSB has the destructive DROP. archive_session_dag is
  shipped (same gate) but returns Ok(None) when the DAG file is absent (a fresh
  session may have no DAG db) -- that absence is a no-op, not an error. The
  run_migration wiring currently archives ONLY the ssb store (the destructive one);
  the session_dag helper is available for a follow-up that threads the DAG path,
  but the SSB journal is the loss-bearing store this slice closes.

AGPL CLEAN-ROOM
  Springdrift is AGPL-3.0 (07-springdrift.md:18,:24). Only the archive-before-
  destroy CONCEPT is reused; NO Springdrift (Gleam) code is copied. stub_ban
  enforces: no tokio::time::interval, no git shell-out, no new dependency
  (rusqlite + std::fs + blake3 all in-tree).

EXPECTED TEST OUTCOMES (auditor is the sole runner)
  - session_archive_ssb_roundtrip_contract.rs: 3 tests GREEN (wire encoder
    round-trip, str round-trip, full DB path via write_payload + query_by_type).
  - session_archive_contract.rs: 4 tests GREEN (enabled archives + blake3 match,
    disabled no-op, helper gate semantics, env-const canonical).
  - session_archive_drop_preserved_contract.rs: 2 tests GREEN (DROP recreates
    clean db with gate ON and OFF; archive present only when ON).
  - session_archive_observable_smoke.rs: 1 test GREEN (the W-5 ON-path: real
    run_migration archives + emits + recreates; archive blake3 == row
    manifest_blake3; live db freshly recreated at SCHEMA_VERSION_PUBLIC).
  - session_archive.rs unit mod: 5 tests GREEN (dest path, disabled no-op,
    enabled copy + stable digest, missing source error, dag missing no-op).
  - action_type_lock.rs: count-locked + golden-locked GREEN at 483 variants.
  - ssb.rs test_typed_payloads: 403 payloads (was 402).
  - cargo check -p elai-orchestrator: compiles (executor ran; see RETURN).

OPEN / DEFERRED
  - Archive disk growth: cold archives accumulate under
    workflow/sessions/{id}/archive/ with no GC. Retention/age-out is DEFERRED to a
    follow-up (noted, not built) -- consistent with the plan's open question.
  - session_dag archive wiring into run_migration is deferred (helper shipped,
    not yet called from the migration path; the SSB journal is the loss-bearing
    store this slice closes).
  - Gaps #1 (hashes-not-bytes checkpoint), #3 (no Backup-Actor), #4 (working tree
    is a git pointer) remain explicitly OUT OF SCOPE per the plan.

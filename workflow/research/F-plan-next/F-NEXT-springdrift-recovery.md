# F-NEXT (DRAFT, research-staging): springdrift-recovery — wire-dark session archive on SSB schema-bump

STATUS: DRAFT / NOT-A-COMMITTED-F-N. This is an F-plan-next staging file, not a numbered F-N in
docs/thefinalglance/F-plan/. The grounded verdict for this topic is recommendation=EXTEND_EXISTING
(confidence=high): a full F-plan part is NOT warranted. This draft is written to F-plan doctrine
(W-1..W-10) so it CAN be promoted to a numbered F-N if a future maintainer disagrees, but its
default disposition is the smallest_increment below, not the Springdrift Backup-Actor mechanism.
A maintainer promoting this to F-N must first re-run the elai.map greps in "Reads (mandatory)" —
anchors are confirmed at authoring time (2026-06-02) but the map drifts.

Objective: close the ONE most-damaging gap between ELAI's SSB-replay + session-handoff substrate and
Springdrift's git-backed whole-state recovery (Backup Actor: git commits + remote push, default 5-min
interval, `git checkout` restores the entire agent state — arXiv:2604.04660 / 07-springdrift.md:49,:110)
WITHOUT importing the Backup-Actor mechanism. The gap closed is gap #2 of four: SSB schema bumps are
DESTRUCTIVE — `ssb::run_migration` DROPs the prior-version `ssb`/`ssb_files` tables on a version
mismatch (ssb.rs:14334-14348, DROP at :14342), silently destroying the audit journal for that session
before recreating it. The increment is an opt-in, wire-dark guard: BEFORE the destructive DROP, copy
the stale-version `ssb.db` (and the session-DAG `session_dag.db`) to
`workflow/sessions/{id}/archive/ssb-v{N}.sqlite`, gated behind one `ELAI_*_ENABLE` kill-switch per
ELAI's standard wire-dark pattern. One file-copy + one-line guard at the existing drop site. Converts
"destroyed forever" into "cold-archived, manually inspectable". Deliberately adds NO periodic commits,
NO Backup Actor, NO unified store, NO git dependency on agent state.

Estimated effort: 0.5 session (~3-4 h). EFFORT S. Single-area change (orchestrator persistence
layer) + one SSB action_type + one frontend pair. NO RunPod, NO network, NO subprocess at gate time —
entirely Tier-1 in-process file I/O. This is the whole shippable scope; there is no Tier-2 split.

Pre-requisites:
- SSB substrate SHIPPED (it is): `ssb::run_migration` (ssb.rs:14334), `CREATE_SSB_TABLE` /
  `CREATE_SSB_FILES_TABLE` (consumed by run_migration), `SCHEMA_VERSION` const (CLAUDE.md baseline 191).
- session-DAG substrate SHIPPED (it is): `SessionDag` snapshot store, `save_snapshot` (session_dag.rs:532),
  `load_snapshot` (:618), `latest_snapshot` (:717), schema_version table (:205-208). Note session-DAG
  migration is NON-destructive (CREATE TABLE IF NOT EXISTS + lazy ALTER TABLE in `SessionDag::open()`,
  :229-231) — so the session-DAG copy in this guard is precautionary, not loss-prevention; SSB is the
  one with the destructive DROP. This asymmetry is the W-7 honesty correction vs the source verdict
  (which implied both stores are destroyed; only SSB is).
- Per-session workflow dir convention SHIPPED: `workflow/sessions/{id}/` is the canonical per-session
  artifact root (CLAUDE.md hook-executions.log path, SESSION_HANDOFF path). The archive subdir nests
  under it.
- NO new external dependency. `rusqlite` Backup API and `std::fs::copy` are both already in-tree
  (rusqlite is the SSB/session-DAG store dependency; no new crate). blake3 already in-tree for the
  archive-manifest digest (checkpoint.rs:5 `use blake3;`).

Parallel-safe with: any plan NOT editing `crates/orchestrator/src/ssb.rs::run_migration` or the
SsbActionType enum in the same window. NOT parallel-safe with a plan that itself touches the SSB
migration path or bumps SCHEMA_VERSION concurrently (coordinate the bump — W-9). Orthogonal to the
checkpoint path (checkpoint.rs) and to the DST snapshot path (dst.rs) — this guard touches neither.

Source: workflow/research/shard-prior-art/07-springdrift.md (the Springdrift prior-art dossier:
Backup Actor at :49, git-backed recovery at :110, AGPL-3.0 at :18/:24, arXiv:2604.04660 at :15,
eval-not-independently-reproduced at :163/:174); the upstream EXTEND_EXISTING verdict (this file's
header). Doctrine: docs/thefinalglance/F-plan/F-0_INDEX.md W-1..W-10.

## Context

ELAI already persists state across three INDEPENDENT, never-co-committed stores, and recovers by
replay / re-derivation rather than byte-restore:

1. SSB SQLite event journal — the typed action catalog with blake3 integrity and ELAI_SEED re-run
   determinism. The `DeterminismDriftPayload` (ssb.rs:16070-16079, `first_divergence_index` field)
   plus the `DriftComparator` (determinism.rs:317, `ELAI_SEED_ENV` at determinism.rs:47-48) back the
   "re-derive by re-running under the same seed" mitigation — ELAI's recovery story is REPLAY, not
   byte-restore.
2. session-DAG SQLite — branching/merge conversation snapshots (session_dag.rs:205-242 schema;
   save/load/latest at :532/:618/:717). `commit_hashes TEXT` (:221) stores POINTERS into the
   project's own external git, not content.
3. agent checkpoints — metadata-in-SSB + 5 files on disk: the `CheckpointManager::write` path
   (checkpoint.rs:209-278) writes 4 msgpack blobs + 1 ts file atomically to disk, and an SSB record
   that carries METADATA ONLY — "hashes + sizes + step_index + agent_id" (verbatim, checkpoint.rs:207;
   blake3 `content_hash` at :194-195). The checkpoint path HAS a lazy migrator
   (`durability::checkpoint_migrator::migrate_checkpoint`, checkpoint_migrator.rs:175; driven by
   `LEGACY_CHECKPOINT_SCHEMA_VERSION`=40 + `serde(default)` at checkpoint.rs:77/:96-97) so checkpoints
   survive schema bumps.

The PreCompact hook auto-writes SESSION_HANDOFF for context-truncation continuity (CLAUDE.md). Working-
file durability is delegated to the PROJECT's own external git (session_dag.rs:221 `commit_hashes`
TEXT — pointers, not content). This is correct for ELAI's bounded multi-agent orchestrate sessions
over a project whose source is ALREADY in real git.

The four confirmed gaps vs Springdrift's Backup Actor (07-springdrift.md:49,:110):
- gap #1 — byte-exact agent state is NOT in the replayable journal: the SSB stores hashes+sizes only
  (checkpoint.rs:207,:248-258); lose the msgpack dir and the journal points at nothing. CARRIED
  UNKNOWN: no agent checkpoint *.msgpack files exist on disk in the repo, so the checkpoint path is
  wired-but-unexercised by any persisted live run — gap #1 is verified from code, not from a captured
  live failure.
- gap #2 — SSB schema bumps are DESTRUCTIVE: `run_migration` (ssb.rs:14334-14348) DROPs prior-version
  `ssb`/`ssb_files` on a `user_version != SCHEMA_VERSION` mismatch (DROP at :14342, comment "Stale
  schema -> drop and recreate (data loss is acceptable for old sessions)" at :14339-14340). The lazy
  migrator exists ONLY on the checkpoint path, NOT on SSB. (session-DAG is NON-destructive — see
  Pre-requisites; the verdict's "destroys the audit journal + session-DAG" overstated session-DAG.)
- gap #3 — NO Backup-Actor analog and NO unified store: the only periodic ticks
  (task_runner.rs:13190-13280) are `run_refusal_sla_tick` (a refusal-SLA monitor, :13205-:13245) and
  `run_skill_staleness_tick_loop` (a skill-staleness demotion sweep, :13247-13280) — confirmed NEITHER
  is a whole-state committer. The only other "snapshot" is `dst.rs::StateSnapshot` (dst.rs:33-39), an
  in-memory DST struct (`fsm_state` / `ssb_row_count` / `active_agents` / `tick` — no bytes, no disk,
  no git).
- gap #4 — the working file tree is a pointer into external git, not versioned content
  (session_dag.rs:221).

Net additive capability Springdrift has that ELAI lacks: crash-consistent, byte-exact, schema-durable,
atomically-restorable reconstruction of the WHOLE agent at moment T from ONE versioned store. This
draft does NOT attempt that. It closes ONLY gap #2's most-damaging edge — silent destruction of the
audit journal on a schema bump — by archiving the stale DB before the DROP. Gaps #1/#3/#4 are
explicitly OUT OF SCOPE and DEFERRED (see Open questions / risks). The fallback, if even this guard is
judged too much, is document-the-gap-build-nothing: one paragraph in the SSB SCHEMA_VERSION header
(ssb.rs near :14339) stating the DROP is intentional and journals are forensic-within-session-only.

## Existing state (B/C/D/E coverage)

- SSB journal (B-28..E-31, SHIPPED): `ssb::run_migration` (ssb.rs:14334) is the SOLE destructive site —
  one function, one DROP (:14342), guarded by `if version > 0`. `CREATE_SSB_TABLE` / `CREATE_SSB_FILES_TABLE`
  recreate immediately after. The SCHEMA_VERSION comment history (ssb.rs:24-600+) records ~50 bumps;
  every bump past the first on a reused DB file triggers this DROP. SsbActionType enum is the action
  catalog this draft adds one variant to.
- session-DAG (B-9, C-23, SHIPPED): `SessionDag` (session_dag.rs) with non-destructive migration
  (CREATE TABLE IF NOT EXISTS :206/:211/:233 + lazy ALTER TABLE in `open()` :229-231). save_snapshot:532,
  load_snapshot:618, latest_snapshot:717. `commit_hashes TEXT` (:221) = external-git pointers.
- agent checkpoints (E-44, SHIPPED): `CheckpointManager::write` (checkpoint.rs:209-278) = 5 files on
  disk + metadata-only SSB row (:207). Lazy migrator `migrate_checkpoint` (checkpoint_migrator.rs:175).
  blake3 `content_hash` (:194-195). This path is the PRECEDENT for "metadata in SSB, bytes on disk" and
  for the blake3 manifest digest this draft reuses.
- DST snapshot (C-27 determinism, SHIPPED): `dst.rs::StateSnapshot` (:33-39) = in-memory only. NOT a
  persistence path; named here only to confirm it is NOT a whole-state committer (gap #3).
- Periodic ticks (E-series, SHIPPED): `run_refusal_sla_tick` + `run_skill_staleness_tick_loop`
  (task_runner.rs:13190-13280). Confirmed NEITHER commits state — this draft adds NO new tick loop
  (the anti-Backup-Actor stance).
- Wire-dark gate precedent (E-25/E-55, SHIPPED): `architecture/approval_fatigue_guard.rs:30`,
  `architecture/bash_complexity_guard.rs:31`, `architecture/hook_prompt.rs:149` all implement the
  standard "env-var-controlled wire-dark gate, default short-circuits to the no-op" pattern. This
  draft's `ELAI_SSB_ARCHIVE_ON_BUMP_ENABLE` gate mirrors them.
- SSB golden lock (E-45, SHIPPED): `action_type_lock.golden` + `cross_lang_drift/detector.rs`. This
  draft adds exactly one action_type and flows it through the golden (W-9).

## Reads (mandatory before coding)

- workflow/research/shard-prior-art/07-springdrift.md (full — Backup Actor :49, git-backed recovery
  :110, AGPL-3.0 :18/:24, arXiv :15, eval-not-reproduced :163/:174, section 7 SHARD-vs-Springdrift
  recovery comparison :187)
- docs/thefinalglance/F-plan/F-0_INDEX.md (W-1..W-10 doctrine — the done-gate this file is written to)
- /workspace/ELAI\.elai_cc\crates\orchestrator\src\ssb.rs:14320-14400 (`run_migration` — the exact
  DROP site this guard wraps; `CREATE_SSB_TABLE` / `CREATE_SSB_FILES_TABLE` recreate)
- /workspace/ELAI\.elai_cc\crates\orchestrator\src\ssb.rs (SsbActionType enum + sql_str + parse arm +
  payload roundtrip + CHECK constraint — the 5-site SSB pattern; SCHEMA_VERSION const)
- /workspace/ELAI\.elai_cc\crates\orchestrator\src\session_dag.rs:200-245 + :525-720 (schema +
  save/load/latest — to confirm the session-DAG copy target path and that its migration is
  non-destructive)
- /workspace/ELAI\.elai_cc\crates\orchestrator\src\checkpoint.rs:190-280 (the blake3 content_hash +
  metadata-in-SSB precedent to mirror for the archive manifest)
- /workspace/ELAI\.elai_cc\crates\orchestrator\src\architecture\approval_fatigue_guard.rs:1-60 +
  bash_complexity_guard.rs:1-60 (the wire-dark env-gate pattern to mirror exactly)
- /workspace/ELAI\.elai_cc\crates\orchestrator\src\ssb_viewer.rs (a viewer query pattern to mirror
  for `session_archives()`)
- /workspace/ELAI\.elai_cc\crates\orchestrator\tests\action_type_lock.rs +
  crates/orchestrator/tests/action_type_lock.golden (W-9 golden lock)

## Tasks (tiered per W-10)

### TIER 1 (in-process file I/O, no network/subprocess/RunPod — the whole shippable scope)

1. **Add `crates/orchestrator/src/session_archive.rs` (~140 LOC).** New module under orchestrator
   (NOT a new crate). Core:
   - `pub const ARCHIVE_ON_BUMP_ENV: &str = "ELAI_SSB_ARCHIVE_ON_BUMP_ENABLE";` — the wire-dark
     kill-switch name, mirroring `architecture/*_guard.rs` const convention.
   - `pub fn archive_on_bump_enabled() -> bool` — reads the env var; returns false (no-op) unless set
     to a truthy value. NOTE THE W-2 TENSION (see Open questions): this is the EXACT pattern F-0 W-2
     forbids as a TERMINAL state for an F-N. Because this is a precautionary archive of data that is
     "data loss is acceptable for old sessions" by the code's own comment (ssb.rs:14339-14340), it
     ships wire-dark-by-design as a forensic opt-in, NOT a default-on enforcement gate. If promoted to
     a numbered F-N, W-2 REQUIRES a flip-to-default-on + smoke step OR an explicit W-2 exception
     justified in the WIRING_CONTRACT. This draft documents the tension rather than hiding it.
   - `pub fn archive_stale_ssb(db_path: &Path, session_id: &str, stale_version: i64) -> Result<Option<PathBuf>, ArchiveError>`
     — if disabled, returns `Ok(None)` (the no-op). If enabled: copies `db_path` to
     `workflow/sessions/{session_id}/archive/ssb-v{stale_version}.sqlite` via the rusqlite online
     Backup API (or `std::fs::copy` if the connection is closed at migration time — resolve at
     implement time which is correct for the pre-CREATE migration window), and returns `Ok(Some(dest))`.
   - `pub fn archive_session_dag(dag_path: &Path, session_id: &str, stale_version: i64) -> Result<Option<PathBuf>, ArchiveError>`
     — precautionary session-DAG copy (session-DAG migration is non-destructive, so this is belt-and-
     suspenders; gate it on the same env var).
   - `pub fn archive_manifest_digest(dest: &Path) -> Result<String, ArchiveError>` — blake3 hex prefix
     (NOT blake2/sha2, CLAUDE.md HARD RULE) of the archived bytes, written into the SSB row so the
     archive is integrity-checkable later. Mirrors checkpoint.rs:194-195 `content_hash`.
   - `pub enum ArchiveError` via `thiserror` (no `Box<dyn Error>`). NO `.unwrap()`/`.expect()` in
     non-test paths. NO magic numbers (the truthy-env set is a named helper).

2. **Wire `archive_stale_ssb` into `run_migration` BEFORE the DROP (~25 LOC delta in ssb.rs:14334-14348).**
   Inside `run_migration`, in the `if version > 0` branch, BEFORE
   `conn.execute_batch("DROP TABLE IF EXISTS ssb_files; DROP TABLE IF EXISTS ssb;")`: call
   `session_archive::archive_stale_ssb(...)` with the DB path + session_id + the stale `version`. The
   DROP STILL PROCEEDS unconditionally afterward (the archive is ADDITIVE — killer_risk #5: must NOT
   change DROP semantics for callers expecting a clean DB). On archive success, emit a
   `session_archived` SSB row AFTER the recreate (the new tables exist by then). On archive FAILURE
   with the gate ENABLED, log + continue the DROP (fail-open on the archive, because blocking the DROP
   would brick session startup — the archive is best-effort forensics, not a safety gate; document this
   choice in the decision doc). When the gate is DISABLED, this is a single `if` that short-circuits —
   zero behaviour change vs today.

3. **Add the `session_archived` SSB action_type (~60 LOC across the 5 ssb.rs sites, W-9).** One new
   action_type (Pattern-2 single-variant — one variant, no Kind discriminator). Touch all 5 sites:
   (a) `SsbActionType` enum arm `SessionArchived`, (b) sql_str value `"session_archived"`, (c) parse
   arm, (d) `SsbPayload::SessionArchived(SessionArchivedPayload { session_id, stale_version, archive_path, manifest_blake3, store_kind })`
   where `store_kind` is `"ssb"` | `"session_dag"`, (e) SQL CHECK constraint + round-trip golden
   fixture. SCHEMA bump named at edit time as baseline 191 + 1 (per CLAUDE.md never hard-code the final
   number in plan prose). Flow through `action_type_lock.golden` (W-9) + `cross_lang_drift/detector.rs`.

4. **Add the ssb_viewer query (~35 LOC in ssb_viewer.rs).** `pub fn session_archives(&self) -> Result<Vec<SessionArchiveRow>, SsbViewerError>`
   mirroring an existing viewer query. `SessionArchiveRow { seq, created_at, session_id, stale_version, archive_path, manifest_blake3, store_kind }`.
   This is what makes the signal NOT wire-dark on the read side (W-4 data source).

5. **Frontend pair: Tauri command + a dashboard surface (~50 LOC tauri + ~40 LOC svelte, W-4).**
   (a) New `#[tauri::command] pub fn get_session_archives(state: State<'_, AppState>) -> CmdResult<Value>`
   in `crates/dashboard-v2/src-tauri/src/commands.rs` (or a `commands_fnext.rs` sibling), reading
   `viewer.session_archives()`, with the `#[cfg(not(...))]` empty-array twin if feature-gated.
   (b) Add a small "Session Archives" list to an existing diagnostics/recovery drawer (resolve the
   exact host drawer at implement time — candidate: a session/recovery panel; confirm via Glob, do NOT
   guess the Svelte filename). (c) Add the command to the TS stub (`src/lib/api/stub.ts`
   `logStub('get_session_archives')`) so `cross_lang_diff.py` sees the Rust<->TS pair. Register the
   command in the Tauri `invoke_handler` list.

6. **Decision doc `workflow/reasoning/F-NEXT-springdrift-recovery-decisions.md` (~90 LOC).** Records:
   (a) EXTEND_EXISTING not full-F-N, and WHY (over-engineering risk — Springdrift targets MONTHS-long
   single autonomous agents, ELAI runs BOUNDED multi-agent sessions over an already-git-backed project);
   (b) the deliberate NON-adoption of the Backup-Actor mechanism (no periodic commits, no git dep on
   agent state, no unified store); (c) the wire-dark-by-design W-2 tension + the fail-open-on-archive
   choice; (d) the session-DAG-is-non-destructive honesty correction vs the source verdict; (e) the
   AGPL-3.0 constraint (concepts reused, NO Springdrift code copied); (f) gaps #1/#3/#4 explicitly
   deferred. This is the W-10 tier statement: entirely Tier-1, no Tier-2.

## WIRING_CONTRACT

produces_symbols:
  - crates/orchestrator/src/session_archive.rs::archive_stale_ssb
  - crates/orchestrator/src/session_archive.rs::archive_session_dag
  - crates/orchestrator/src/session_archive.rs::archive_manifest_digest
  - crates/orchestrator/src/session_archive.rs::archive_on_bump_enabled
  - crates/orchestrator/src/session_archive.rs::ArchiveError
  - crates/orchestrator/src/session_archive.rs::ARCHIVE_ON_BUMP_ENV
  - crates/orchestrator/src/ssb_viewer.rs::session_archives
  - crates/orchestrator/src/ssb_viewer.rs::SessionArchiveRow
  - SsbActionType::SessionArchived
  - SsbPayload::SessionArchived (SessionArchivedPayload)
  - crates/dashboard-v2/src-tauri/src/commands.rs::get_session_archives

consumes_symbols:
  - crates/orchestrator/src/ssb.rs::run_migration  (CONFIRMED ssb.rs:14334 DEF_FN; DROP at :14342 — the insertion point)
  - crates/orchestrator/src/ssb.rs::SCHEMA_VERSION  (CONFIRMED ssb.rs — the const the migration compares user_version against; baseline 191 per CLAUDE.md)
  - crates/orchestrator/src/session_dag.rs::SessionDag  (CONFIRMED session_dag.rs — the session-DAG store whose db file is precautionarily copied; save_snapshot:532 / load_snapshot:618 / latest_snapshot:717 DEF_FN)
  - crates/orchestrator/src/checkpoint.rs::content_hash  (CONFIRMED checkpoint.rs:194 DEF_FN + blake3::hash CALL at :195 — the blake3 digest precedent mirrored)
  - crates/orchestrator/src/ssb_viewer.rs (CONFIRMED — viewer query pattern mirrored; SsbViewerError exists)
  - crates/orchestrator/src/architecture/approval_fatigue_guard.rs (CONFIRMED :30 — wire-dark env-gate const pattern mirrored)
  - crates/orchestrator/src/architecture/bash_complexity_guard.rs (CONFIRMED :31 — wire-dark env-gate pattern mirrored)

reachable_from:
  - PRODUCTION: SSB initialization -> `run_migration` (ssb.rs:14334, called on every SSB open with a
    stale user_version) -> `archive_stale_ssb` runs (when ELAI_SSB_ARCHIVE_ON_BUMP_ENABLE set) BEFORE
    the DROP -> `session_archived` SSB row emitted after recreate. (run_migration IS the production SSB
    open path; confirm its caller chain at implement time via elai.map — the migration is invoked from
    the SSB open/new constructor, not a #[cfg(test)] caller.)
  - DASHBOARD: a session/recovery drawer "Session Archives" list -> get_session_archives Tauri command
    -> ssb_viewer.session_archives() -> live ssb.db rows rendered

stub_ban:
  - literal "TODO: archive later" absent
  - literal "// copy db someday" absent
  - NO change to DROP semantics: the `DROP TABLE` at ssb.rs:14342 MUST still execute unconditionally
    after the archive attempt (killer_risk #5 — additive only)
  - NO new periodic tick / Backup-Actor loop: this plan adds NO `tokio::time::interval` committer
    (the anti-over-engineering stance; grep must find no new interval/tick loop in session_archive.rs)
  - NO git invocation: session_archive.rs MUST NOT shell out to `git` or add a git crate (the
    deliberate non-adoption of the Backup-Actor mechanism)
  - NO .unwrap()/.expect() in session_archive.rs non-test paths
  - blake3 only for the manifest digest (no blake2/sha2)

frontend_pair:
  svelte_consumer: a session/recovery drawer "Session Archives" list (exact .svelte resolved at
    implement time via Glob — NOT guessed here; W-7 forbids a hallucinated filename)
  tauri_command: crates/dashboard-v2/src-tauri/src/commands.rs::get_session_archives
  ts_binding: crates/dashboard-v2/src/lib/api/stub.ts  (logStub('get_session_archives'))

observable_effect:
  trigger: "open an SSB whose on-disk user_version < current SCHEMA_VERSION with
    ELAI_SSB_ARCHIVE_ON_BUMP_ENABLE=1 set (e.g. cd .elai_cc && ELAI_SSB_ARCHIVE_ON_BUMP_ENABLE=1 cargo
    run -p elai-cli -- <any command that opens a reused stale-version ssb.db>); the migration archives
    the stale DB then DROPs+recreates"
  expect_ssb: "session_archived"
  expect_artifact: "the file workflow/sessions/{id}/archive/ssb-v{N}.sqlite EXISTS and is a valid
    SQLite file (its blake3 prefix matches the manifest_blake3 in the session_archived row); the
    current ssb.db is freshly recreated at the new SCHEMA_VERSION (the DROP still happened); the same
    row renders in the dashboard 'Session Archives' list"

## DETERMINISTIC_CHECK

```bash
# W-1 DONE-GATE (the literal line — wiring is a hard executed done-gate, not cargo-green):
python .claude/tools/wiring_contract_check.py --bn-file workflow/research/F-plan-next/F-NEXT-springdrift-recovery.md --run-trigger

# 1. The archive module + core symbols exist as real DEFs
rg "pub fn archive_stale_ssb|pub fn archive_session_dag|pub fn archive_manifest_digest|pub fn archive_on_bump_enabled|pub const ARCHIVE_ON_BUMP_ENV|pub enum ArchiveError" .elai_cc/crates/orchestrator/src/session_archive.rs

# 2. Wired into run_migration BEFORE the DROP (the DROP must still be present and after the archive)
rg -n "archive_stale_ssb|DROP TABLE IF EXISTS ssb" .elai_cc/crates/orchestrator/src/ssb.rs

# 3. The SSB action_type exists at all 5 sites (W-9)
rg "session_archived|SessionArchived|SessionArchivedPayload" .elai_cc/crates/orchestrator/src/ssb.rs
rg "session_archived" .elai_cc/crates/orchestrator/tests/action_type_lock.golden

# 4. The viewer query + row type (anti-wire-dark read side)
rg "pub fn session_archives|SessionArchiveRow" .elai_cc/crates/orchestrator/src/ssb_viewer.rs

# 5. Frontend pair (W-4): Tauri command + TS binding
rg "get_session_archives" .elai_cc/crates/dashboard-v2/src-tauri/src/commands.rs
rg "get_session_archives" .elai_cc/crates/dashboard-v2/src/lib/api/stub.ts

# 6. Cross-language drift clean
python .claude/tools/cross_lang_diff.py

# 7. Anti-over-engineering: NO new tick loop, NO git shell-out in the archive module
! rg -n "tokio::time::interval|Command::new\(.git|std::process::Command" .elai_cc/crates/orchestrator/src/session_archive.rs

# 8. blake3 only (no blake2/sha2) for the manifest
rg "blake3" .elai_cc/crates/orchestrator/src/session_archive.rs
! rg "blake2|Sha256|sha2::" .elai_cc/crates/orchestrator/src/session_archive.rs 2>/dev/null

# 9. Decision doc present
test -f workflow/reasoning/F-NEXT-springdrift-recovery-decisions.md
```

## ACCEPTANCE

- [ ] **W-1 wiring done-gate**: `wiring_contract_check.py --bn-file ... --run-trigger` returns PASS
      (exit 0): every produces_symbol resolves as DEF/SCIP_DEF, every consumes_symbol has a real
      CALL/REF, the reachable_from anchor is reached over CALL_RESOLVED within depth 8, stub_ban finds
      zero literals, and the observable_effect trigger RAN and emitted the row + the archive file.
      (cargo-green necessary, NOT sufficient.)
- [ ] **W-2 tension RESOLVED, not hidden**: this is wire-dark-by-design (forensic opt-in over
      "acceptable data loss"). The decision doc records an EXPLICIT W-2 exception with justification,
      OR (if promoted to F-N) a flip-to-default-on + smoke step is added. The session is NOT silently
      shipped default-off without the documented exception.
- [ ] **DROP semantics unchanged**: the `DROP TABLE` at ssb.rs:14342 still executes unconditionally
      after the archive; a test asserts a stale-version open still yields a clean fresh DB at the new
      SCHEMA_VERSION (killer_risk #5).
- [ ] **Archive is byte-faithful + integrity-checked**: the archived `ssb-v{N}.sqlite` opens as a
      valid SQLite DB and its blake3 prefix matches the `manifest_blake3` in the `session_archived`
      row.
- [ ] **Disabled = zero behaviour change**: with `ELAI_SSB_ARCHIVE_ON_BUMP_ENABLE` UNSET, no archive
      file is written, no `session_archived` row is emitted, and `run_migration` behaves byte-identical
      to today (a test asserts this — the no-op path must be provably inert).
- [ ] **No Backup-Actor creep**: grep confirms NO new periodic tick / interval committer and NO git
      shell-out in session_archive.rs (killer_risk #1 — deliberate non-adoption).
- [ ] **W-4 frontend pair + cross_lang_diff**: `get_session_archives` + the dashboard list + the TS
      stub all present; `cross_lang_diff.py` passes.
- [ ] **W-6 dashboard E2E**: with the dashboard running, a `session_archived` row APPEARS in the
      "Session Archives" list (verified THROUGH the dashboard by dashboard-smoker), not just a backend
      row.
- [ ] **W-9 SSB golden lock**: `session_archived` flows through `action_type_lock.golden`;
      SCHEMA_VERSION bumped baseline 191 + 1 (named exactly at edit time); action_type_lock +
      cross_lang_drift green; SessionArchivedPayload round-trip golden passes.
- [ ] **AGPL clean-room**: NO Springdrift (Gleam/AGPL-3.0) code copied; only the concept (archive-
      before-destroy) is reused. The decision doc records this.
- [ ] `cargo check --workspace` GREEN; auditor (sole test runner) PASS on `-p elai-orchestrator` +
      the new contract tests + the dashboard svelte-check.
- [ ] No new external dependency (rusqlite Backup API + std::fs + blake3 all in-tree; anti-
      slopsquatting per CLAUDE.md — verify before any add).

## Tests to add

- `crates/contracts/tests/session_archive_contract.rs` (~120 LOC): with the gate ENABLED, opening a
  stale-version SSB db file archives it to `archive/ssb-v{N}.sqlite` and emits `session_archived`;
  with the gate DISABLED, no archive + no row + identical migration outcome; assert the archived DB is
  valid SQLite and its blake3 matches the row's manifest_blake3 (stable under ELAI_SEED).
- `crates/contracts/tests/session_archive_drop_preserved_contract.rs` (~70 LOC): assert the DROP still
  runs after the archive — a stale-version open yields a fresh empty `ssb` table at the new
  SCHEMA_VERSION regardless of gate state (killer_risk #5).
- `crates/contracts/tests/session_archive_ssb_roundtrip_contract.rs` (~60 LOC): `SessionArchivedPayload`
  round-trips through rmp; sql_str == "session_archived"; CHECK constraint accepts it; golden row present.
- `crates/orchestrator/tests/session_archive_observable_smoke.rs` (~100 LOC): the W-5 fixture harness
  exercising the REAL `run_migration` path against a temp ssb.db pre-seeded with an old user_version,
  asserting the archive file + the row + the fresh recreated tables — the SAME code path as the live
  trigger, not a mock.

## Files touched

- CREATE: `crates/orchestrator/src/session_archive.rs` (~140 LOC: archive fns + env gate + blake3 manifest + ArchiveError)
- CREATE: `workflow/reasoning/F-NEXT-springdrift-recovery-decisions.md` (~90 LOC)
- CREATE: `crates/contracts/tests/session_archive_contract.rs` (~120 LOC)
- CREATE: `crates/contracts/tests/session_archive_drop_preserved_contract.rs` (~70 LOC)
- CREATE: `crates/contracts/tests/session_archive_ssb_roundtrip_contract.rs` (~60 LOC)
- CREATE: `crates/orchestrator/tests/session_archive_observable_smoke.rs` (~100 LOC)
- MODIFY: `crates/orchestrator/src/ssb.rs` (~25 LOC in run_migration: archive call BEFORE the DROP +
  session_archived emit after recreate; ~60 LOC for the 1 new action_type across the 5 sites + payload
  + golden + CHECK)
- MODIFY: `crates/orchestrator/src/ssb_viewer.rs` (~35 LOC: session_archives query + SessionArchiveRow)
- MODIFY: `crates/orchestrator/src/lib.rs` (~2 LOC: `pub mod session_archive;` + re-export)
- MODIFY: `crates/orchestrator/tests/action_type_lock.golden` (~1 LOC: the locked variant)
- MODIFY: `crates/dashboard-v2/src-tauri/src/commands.rs` (~50 LOC: get_session_archives + cfg twin)
- MODIFY: the Tauri builder generate-handler site (invoke_handler list) (~1 LOC: register get_session_archives)
- MODIFY: a session/recovery drawer .svelte (resolved via Glob at implement time) (~40 LOC: Session Archives list)
- MODIFY: `crates/dashboard-v2/src/lib/api/stub.ts` (~5 LOC: get_session_archives stub)

## Citations / Grounding

- workflow/research/shard-prior-art/07-springdrift.md — PRIMARY prior-art source. Backup Actor = git
  commits + remote push (:49); git-backed recovery, default 5-min commits, `git checkout` whole-state
  restore (:110); AGPL-3.0 (:18,:24); arXiv:2604.04660 (:15); SHARD-vs-Springdrift recovery comparison
  (:187 — Springdrift git whole-state vs SHARD per-change .bak); eval NOT independently reproduced,
  n=1 self-run (:163,:174). All Springdrift numbers are vendor-reported, single-author (Seamus Brady),
  45 stars, unreproduced (:167-174) — too weak to justify a heavy build (killer_risk #2).
- arXiv:2604.04660 — VERIFIED RESOLVES (WebFetch 2026-06-02): "Springdrift: An Auditable Persistent
  Runtime for LLM Agents with Case-Based Memory, Normative Safety, and Ambient Self-Perception",
  Seamus Brady, submitted 2026-04-06. cs.AI. Title + author + date confirmed against the dossier.
- ELAI ground truth (code-read, CONFIRMED in source + elai.map at authoring 2026-06-02):
  - `ssb.rs::run_migration` at ssb.rs:14334-14348 (DEF_FN); the destructive `DROP TABLE IF EXISTS
    ssb_files; DROP TABLE IF EXISTS ssb;` at ssb.rs:14342; the "data loss is acceptable for old
    sessions" comment at :14339-14340; `CREATE_SSB_TABLE`/`CREATE_SSB_FILES_TABLE` recreate at
    :14344-14345. W-7 CORRECTION vs the source verdict: the verdict cited "ssb.rs:57..:1634, ~23
    occurrences" — those are the SCHEMA_VERSION comment-history lines (ssb.rs:24-600+), NOT execution
    sites. The REAL destructive site is ONE function, ONE DROP, at :14342. This draft cites the precise
    anchor.
  - `DeterminismDriftPayload` first_divergence_index at ssb.rs:16070,:16079; `DriftComparator` at
    determinism.rs:317 (W-7 CORRECTION: verdict said ssb.rs:403 — DriftComparator is in determinism.rs,
    not ssb.rs); `ELAI_SEED_ENV` const at determinism.rs:47-48. Backs the "re-derive by re-running"
    mitigation framing.
  - `CheckpointManager::write` at checkpoint.rs:209-278 (metadata-only SSB row comment at :207; 5 files
    on disk at :241-245; blake3 `content_hash` DEF_FN at :194, `blake3::hash` CALL at :195). The lazy
    checkpoint migrator is `durability::checkpoint_migrator::migrate_checkpoint` at
    checkpoint_migrator.rs:175 (+ `migrate` at :110), driven by `LEGACY_CHECKPOINT_SCHEMA_VERSION`=40
    (checkpoint.rs:77) + `serde(default)` (checkpoint.rs:96-97). W-7 CORRECTION vs verdict: verdict
    cited "checkpoint.rs:70-77,91-97" for the migrator — those lines are the LEGACY const +
    serde-default; the migrator FUNCTION lives in durability/checkpoint_migrator.rs. This draft names
    the real module.
  - `session_dag.rs` schema at :205-242 (CREATE TABLE IF NOT EXISTS :206/:211/:233 — NON-destructive
    migration; lazy ALTER TABLE in `open()` :229-231); `commit_hashes TEXT` at :221; save_snapshot:532
    / load_snapshot:618 / latest_snapshot:717 (DEF_FN). W-7 HONESTY CORRECTION vs verdict: the verdict
    said the schema bump destroys "the audit journal + session-DAG" — but session-DAG migration is
    NON-destructive (IF NOT EXISTS + ALTER). Only SSB DROPs. The session-DAG copy in this guard is
    therefore precautionary, not loss-prevention. This is a material correction, surfaced not hidden.
  - `dst.rs::StateSnapshot` at dst.rs:33-39 (in-memory DST struct: fsm_state/ssb_row_count/active_agents/
    tick — no bytes, no disk, no git); `snapshot_state` DEF_FN at dst.rs:86. Confirms gap #3 (no
    whole-state committer). (Verdict said :33 — struct attribute at :33, `pub struct` at :34, fields
    through :39.)
  - periodic ticks: `task_runner.rs:13190-13245` region carries the refusal-SLA tick
    (`REFUSAL_SLA_TICK_INTERVAL_SECS`, refusal_sla::RefusalRateSnapshotEmitter) and `:13247-13280` the
    skill-staleness tick (`SKILL_STALENESS_TICK_INTERVAL_SECS`). Confirmed NEITHER commits state
    (gap #3).
  - wire-dark env-gate precedent: `architecture/approval_fatigue_guard.rs:30`,
    `architecture/bash_complexity_guard.rs:31`, `architecture/hook_prompt.rs:149` (the standard
    "env-var-controlled wire-dark gate, default no-op" pattern this draft's gate mirrors).
- CLAUDE.md HARD RULES applied: blake3 (never blake2/sha2) for the manifest digest; no
  .unwrap()/.expect() in non-test paths; Pattern-2 single-variant SSB (one action_type); SCHEMA_VERSION
  baseline 191 + 1 (named at edit time, never hard-coded final number in prose); no new external dep
  (rusqlite + std::fs + blake3 in-tree — anti-slopsquatting: verify before any add).

## Open questions / risks

- **OVER-ENGINEERING is the headline risk (killer_risk #1)**: adopting the full Backup-Actor pattern
  (periodic git-commit of SSB + session-DAG + msgpack dirs) is a heavy mechanism justified by a failure
  mode ELAI rarely hits. Springdrift targets MONTHS-long single autonomous agents (23-day "Curragh"
  deployment, 07-springdrift.md:161); ELAI runs BOUNDED multi-agent orchestrate sessions over a project
  whose source is ALREADY in real git. MITIGATION: this draft adds NO periodic committer, NO git dep,
  NO unified store — only an archive-before-destroy guard at the one destructive site. The stub_ban
  enforces the non-adoption (no new tick loop, no git shell-out).
- **EVIDENCE TOO WEAK FOR A HEAVY BUILD (killer_risk #2)**: every Springdrift number is vendor-reported,
  single-author (Seamus Brady), n=1, unreproduced, 45 stars (07-springdrift.md:167-174); eval
  scaffolding exists on HuggingFace but is NOT independently reproduced (:174). This is precisely why
  the recommendation is EXTEND_EXISTING and the increment is minimal — there is no measured uplift to
  justify importing the mechanism. Any "this guard prevents X% of lost-journal incidents" claim is
  UNGROUNDED until measured on ELAI's own session history.
- **W-2 DEFAULT-OFF TENSION (the doctrine collision)**: F-0 W-2 forbids a default-off env gate as the
  TERMINAL state for a numbered F-N. This guard is wire-dark-by-design because it archives data the
  code itself marks "acceptable to lose" (ssb.rs:14339-14340) — it is forensic opt-in, not enforcement.
  RESOLUTION OPTIONS: (a) keep it as an F-plan-next staging artifact (this file) that never becomes a
  numbered F-N, so W-2 does not bind; (b) if promoted, add the W-2 flip-to-default-on + smoke step; or
  (c) promote with an explicit W-2 exception justified in the WIRING_CONTRACT (the "pure internal
  forensic, no user-facing enforcement meaning" carve-out). Do NOT silently ship default-off as a
  numbered F-N without picking (b) or (c).
- **AGPL-3.0 LICENSE (killer_risk #3)**: Springdrift is AGPL-3.0 (07-springdrift.md:18,:24). Concepts/
  mechanisms (archive-before-destroy) are not copyrightable and are reusable; Springdrift's Gleam CODE
  is NOT reusable without copyleft exposure on a network service. This draft is clean-room: it copies
  NO Springdrift code, only the idea. ACCEPTANCE enforces this.
- **CARRIED UNKNOWN — checkpoint path unexercised (killer_risk #4)**: NO agent checkpoint *.msgpack
  files exist on disk in the repo, so the checkpoint path (gap #1, hashes-not-bytes) is
  wired-but-unexercised by any persisted live run. Gap #1 is verified from CODE, not from a captured
  live failure. This draft does NOT touch the checkpoint path (it closes gap #2 only), so the unknown
  does not block it — but a future gap-#1 plan must first capture a real checkpoint-loss failure before
  building.
- **DROP SEMANTICS MUST NOT CHANGE (killer_risk #5)**: even the smallest increment must NOT silently
  change DROP semantics for callers expecting a clean DB. The archive copy is ADDITIVE; the DROP still
  proceeds unconditionally. A dedicated test (session_archive_drop_preserved_contract.rs) asserts this.
- **session-DAG IS NON-DESTRUCTIVE (honesty correction)**: the source verdict implied the schema bump
  destroys both the audit journal and the session-DAG. In fact session-DAG migration is non-destructive
  (CREATE TABLE IF NOT EXISTS + lazy ALTER TABLE, session_dag.rs:206/:229-231); only SSB DROPs. The
  session-DAG copy in this guard is precautionary belt-and-suspenders, not loss-prevention. Scope this
  honestly — do not over-claim the session-DAG is at risk.
- **fail-open on archive failure**: if the archive copy fails while the gate is ENABLED, the guard logs
  and continues the DROP (blocking the DROP would brick session startup). The archive is best-effort
  forensics, not a safety gate. Document this in the decision doc; do not let the panel imply the
  archive is guaranteed.
- **archive disk growth**: cold archives accumulate under workflow/sessions/{id}/archive/. Open
  question: a retention/GC policy (keep last N, or age-out). DEFER to a follow-up; the initial guard
  ships without GC and the decision doc flags it.

## Next

If a maintainer accepts EXTEND_EXISTING: implement the smallest_increment above as a patch-N or a
staged F-plan-next promotion, NOT a full F-N. If even the guard is judged too much: the FALLBACK is
document-the-gap-build-nothing — add one paragraph to the SSB SCHEMA_VERSION header near ssb.rs:14339
stating the DROP is intentional and SSB journals are forensic-within-session-only (working-file
durability is the project's external git, agent-state recovery is SSB replay under ELAI_SEED). Gaps #1
(hashes-not-bytes), #3 (no Backup-Actor / unified store), and #4 (working tree is a git pointer) remain
explicitly DEFERRED and should NOT be reopened without a captured live failure motivating them — per
killer_risk #1/#2, the Backup-Actor mechanism is over-built for ELAI's bounded, already-git-backed
sessions.

End of F-NEXT-springdrift-recovery (DRAFT).

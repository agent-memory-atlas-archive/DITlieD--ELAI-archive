C-23 — SessionDag Branch UI: decisions, drift resolution, trade-offs

Session: s-25672 (executor; no global FSM transitions)
Plan: docs/thefinalglance/C-plan/C-23_sessiondag_branch_ui.md
Context package: workflow/sessions/s-25672/context-package-C23.md

================================================================
ORCHESTRATOR DECISIONS (carried from prompt)
================================================================

Q1 — SessionsDrawer.svelte does not exist.
    Decision: CREATE new SessionsDrawer.svelte and mount BranchTree inside it.
    Implementation:
      - dashboard-v2/src/lib/drawers/SessionsDrawer.svelte (new file)
      - DrawerKey adds the literal 'sessions' (api/types.ts)
      - DrawerHost.svelte registers SessionsDrawer for drawerKey === 'sessions'
      - Drawer.svelte DRAWER_LABELS entry: 'sessions' -> 'SESSIONS'
    Rationale: an existing drawer (OrchestrationDrawer) already shows a
    snapshot/heartbeat view; cramming branch ops into it conflates the
    surfaces. A dedicated drawer keeps the SessionDag surface coherent
    with the new Tauri command boundary (fork/merge/close/load_handoff).

Q2 — Tauri command param naming (branchId vs name).
    Decision: KEEP `name: String` per existing pattern.
    Implementation: fork_session_branch / merge_session_branch /
    close_session_branch each take `name: String`. The frontend
    stub.ts already passes `{ name }` — preserved unchanged.
    SessionsDrawer maps the BranchTree row id back to the row's
    `label` (which is the orchestrator-side branch name) before
    invoking the command.
    Rationale: get_session_branches predates C-23 and uses `name`
    everywhere. Renaming would break the existing stub.ts contract
    and the dashboard-side test fixtures.

Q3 — AppState SessionDag handle.
    Decision: per-call DAG open. AppState gains `dag_db_path` (cached
    PathBuf), not the dag handle itself.
    Implementation:
      - state.rs: `pub dag_db_path: Arc<Mutex<Option<PathBuf>>>`
        + `dag_db_path_snapshot()` / `set_dag_db_path()`
      - commands.rs: `resolve_active_dag_path` walks
        `<project_root>/.elai_cc/sessions/<id>/dag.db` and caches
        the newest hit by mtime. `open_active_dag` opens a fresh
        `SessionDag` per command call, performs the op, drops the
        handle. Roughly ~5 ms SQLite open per call.
    Trade-off documented:
      - pro: zero lifetime entanglement with the CLI process
        (the CLI also opens its own SessionDag for the same db —
        WAL mode handles the concurrent reads/writes).
      - pro: stateless across project switches; switching project
        just clears the cache and re-resolves on next call.
      - pro: no `Arc<Mutex<SessionDag>>` field hiding interior
        mutability for the entire process lifetime.
      - con: ~5 ms SQLite open per branch-op call (negligible
        compared to the user-driven nature of these commands).
      - con: the cached path can go stale if the project switches
        between the cache write and the next call; mitigated by
        the `cached.exists()` guard in `resolve_active_dag_path`.

Q4 — SSB schema bump vs Checkpoint discriminator.
    Decision: add proper SSB variants (BranchForked / BranchMerged /
    BranchClosed). SCHEMA 54 -> 55 atomic 9-tuple lockstep.
    Implementation: 3 new SsbActionType variants + 3 new SsbPayload
    variants + 3 new payload structs + as_sql_str arms + from_str
    entries + SQL CHECK constraint update + test_payload arms +
    all_test_payloads sample list + count assertion bumped
    109 -> 112 + ledger comment block (C-23 SCHEMA_VERSION 55).
    Migration semantics: SCHEMA_VERSION integer pragma flips from
    54 to 55 atomically; old DBs with version 54 will be rejected
    at boot per existing `assert_or_init_schema` path.
    Rationale: dashboard consumers should match on a single
    discriminant (SsbActionType::BranchMerged), not pattern-match
    on Checkpoint payload + free-text discriminator string in
    `grc_history_hash`. The legacy Checkpoint emit at
    `merge_branch_with_ssb` was indistinguishable from a
    snapshot-save row in the dashboard activity feed.

Q5 — close_branch SSB emit.
    Decision: emit at the orchestrator layer (close_branch_with_ssb
    wrapper, mirror of merge_branch_with_ssb pattern).
    Implementation: SessionDag::close_branch_with_ssb (async, takes
    Option<&Ssb>) + SessionDag::create_branch_with_ssb (same shape,
    for symmetry). Both call the existing sync close_branch /
    create_branch and emit a typed SSB row on success. SSB write
    failures are surfaced via stderr but do not roll back the DB
    op (matches the existing merge_branch_with_ssb semantics).
    Rationale: keeping all branch-op SSB emit logic in
    session_dag.rs means every caller (CLI /session fork|merge|
    close, dashboard fork_session_branch/merge_session_branch/
    close_session_branch, future MCP tool calls) gets identical
    payload shapes. The dashboard layer is left thin — it just
    forwards the typed `ssb_writer` Arc.

D3 — DagBranchError typed wrapper.
    Decision: introduce a thin `DagBranchError` enum (thiserror)
    in dashboard-v2/src-tauri/src/commands.rs with `#[from]`
    SessionDagError + `NoActiveSession` / `NoHandoff` /
    `MergeConflict` / `OpenFailed` variants. Flattened to the
    dashboard's `CmdResult<T, String>` via `map_dag_err`.
    Rationale: the dashboard already uses String for Tauri
    rejection (existing convention — Tauri serializes Result<T,E>
    where E is the rejection JSON). Wrapping at the boundary
    keeps the `?` propagation ergonomic without changing the
    wire-level error shape.

================================================================
DRIFT RESOLUTIONS
================================================================

D1 (CRITICAL — SessionsDrawer does not exist): resolved via Q1.

D2 (Tauri command param naming): resolved via Q2 — name kept.

D3 (DagBranchError type missing): resolved via dedicated decision
    (thiserror wrapper introduced).

D4 (AppState has no SessionDag handle): resolved via Q3 — per-call
    DAG open. No `Arc<Mutex<SessionDag>>` field added.

D5 (SSB schema change is SCHEMA-LEVEL): resolved via Q4 — atomic
    bump 54 -> 55 + 3 new typed variants. The previous Checkpoint
    discriminator emit at merge_branch_with_ssb is removed.

D6 (No load_handoff Tauri command exists): resolved — new
    `load_handoff` Tauri command added (async, opens DAG per call,
    returns metadata for the CommandWindow status card). Registered
    in lib.rs invoke_handler. Wrapped in stub.ts as
    `export async function load_handoff(): Promise<LoadHandoffResult>`.

D7 (line-number drift in CommandWindow): noted only; the actual
    executeSlash function is at the file's current line ~261,
    not the plan's L253. Fix applied to the correct location.

D8 (parent_chain line-number drift): noted only; chain.reverse()
    inserted at the actual location before Ok(chain) at L863.

D9 (close_branch did not emit SSB): resolved via Q5 — new
    close_branch_with_ssb wrapper emits a typed BranchClosed row.

================================================================
DONE / VERIFIED (no action needed)
================================================================

N1 — Task 6 (100-run determinism): VERIFIED PRESENT.
    `session_dag.rs::tests::test_100_run_determinism` (added by C-18
    backfill, current location ~L3370) iterates 100 times with a
    fixed seed and asserts byte-identical snap_id across iterations.
    The C-23 contract test (c23_t8) adds a smoke variant in the
    contracts crate so renames surface in C-23 output.

N2 — Task 7 (SettingsWindow SessionHandoffEntry schema): VERIFIED CLEAN.
    Grep across SettingsWindow.svelte returned zero matches for
    `SessionHandoffEntry`, `HandoffEntry`, `handoff_entry`, or
    legacy `chat_window_state`. No schema fields needed alignment.

N3 — BranchTree.svelte uses Svelte 5 runes ($props). Props shape
    (branches, onfork, onmerge, onclose) matches the SessionsDrawer
    callsite. No rewrite needed.

================================================================
FILES TOUCHED
================================================================

MODIFY (Rust):
  .elai_cc/crates/orchestrator/src/ssb.rs
    — SCHEMA_VERSION 54 -> 55
    — 3 new SsbActionType variants (BranchForked/Merged/Closed)
    — 3 new SsbPayload variants
    — 3 new payload structs (BranchForkedPayload, BranchMergedPayload,
      BranchClosedPayload)
    — SQL CHECK constraint extended
    — as_sql_str / from_str / action_type / test_payload /
      all_test_payloads / count assertion (109 -> 112)
    — ledger comment block + migration notes

  .elai_cc/crates/orchestrator/src/session_dag.rs
    — parent_chain root-to-tip fix (chain.reverse() before Ok)
    — test_parent_chain_linear asserts flipped (root at chain[0])
    — merge_branch_with_ssb emits BranchMerged (was Checkpoint)
    — create_branch_with_ssb new wrapper (async, emits BranchForked)
    — close_branch_with_ssb new wrapper (async, emits BranchClosed)

  .elai_cc/crates/orchestrator/src/ssb_viewer.rs
    — session_handoffs now reads BOTH BranchMerge (legacy) AND
      BranchMerged (C-23) action_types via SQL IN (?,?)

  .elai_cc/crates/orchestrator/src/lib.rs
    — pub use BranchForkedPayload, BranchMergedPayload, BranchClosedPayload

  .elai_cc/crates/cli/src/commands.rs
    — parse_session_command: new merge/close arms (with usage_error
      on missing arg)
    — SlashCommandSpec argument_hint updated
    — help test asserts the new hint string
    — parse-tests for /session merge <id> + /session close <id> +
      Usage error on missing branch-id

  .elai_cc/crates/cli/src/handoff.rs
    — handle_session_command: merge/close arms wire to
      SessionDag::merge_branch_with_ssb /
      SessionDag::close_branch_with_ssb (short-lived current-thread
      tokio runtime for the async SSB write)
    — error message for unknown action lists the new actions

  .elai_cc/crates/dashboard-v2/src-tauri/src/state.rs
    — AppState: new `dag_db_path: Arc<Mutex<Option<PathBuf>>>`
    — `dag_db_path_snapshot()` / `set_dag_db_path()` helpers

  .elai_cc/crates/dashboard-v2/src-tauri/src/commands.rs
    — new `DagBranchError` enum (thiserror + #[from] SessionDagError)
    — `resolve_active_dag_path` (newest dag.db under .elai_cc/sessions)
    — `open_active_dag` (per-call SessionDag::open)
    — fork_session_branch / merge_session_branch / close_session_branch
      now real implementations (replace CLI-delegation stubs)
    — new `load_handoff` Tauri command

  .elai_cc/crates/dashboard-v2/src-tauri/src/lib.rs
    — invoke_handler registers load_handoff

MODIFY (Frontend):
  .elai_cc/crates/dashboard-v2/src/lib/api/types.ts
    — DrawerKey adds 'sessions'

  .elai_cc/crates/dashboard-v2/src/lib/api/stub.ts
    — new `load_handoff()` wrapper + LoadHandoffResult type

  .elai_cc/crates/dashboard-v2/src/lib/drawers/DrawerHost.svelte
    — register SessionsDrawer for drawerKey === 'sessions'

  .elai_cc/crates/dashboard-v2/src/lib/layout/Drawer.svelte
    — DRAWER_LABELS entry for 'sessions'

  .elai_cc/crates/dashboard-v2/src/lib/windows/CommandWindow.svelte
    — executeSlash('/continue') calls load_handoff (status card on
      success, error card + textarea-prepopulate fallback on failure)

MODIFY (Tests):
  .elai_cc/crates/orchestrator/tests/session_dag_activation.rs
    — test (e) asserts BranchMerged emit + zero Checkpoint
      regression (was: 1 new Checkpoint row)

CREATE:
  .elai_cc/crates/dashboard-v2/src/lib/drawers/SessionsDrawer.svelte
    — new drawer hosting BranchTree + per-row fork/merge/close
      invocations (Tauri commands)

  .elai_cc/crates/contracts/tests/c23_session_dag_ui_contract.rs
    — 12 tests:
      t1: SCHEMA_VERSION_PUBLIC >= 55
      t2 x3: SsbActionType round-trip for each new variant
      t3 x3: SsbPayload msgpack round-trip for each new variant
      t4: parent_chain returns root-to-tip order (root at chain[0])
      t5: create_branch_with_ssb emits BranchForked
      t6: merge_branch_with_ssb emits BranchMerged + zero
          new Checkpoint rows
      t7: close_branch_with_ssb emits BranchClosed
      t8: distinct SessionDag handles do not collide (smoke for
          the orchestrator-private 100-run determinism test)

  workflow/reasoning/C-23-decisions.md
    — this file

================================================================
TEST RESULTS
================================================================

c23_session_dag_ui_contract: 12 passed; 0 failed
  - c23_t1_schema_version_at_least_55 ............................ ok
  - c23_t2_branch_forked_action_type_round_trip .................. ok
  - c23_t2_branch_merged_action_type_round_trip .................. ok
  - c23_t2_branch_closed_action_type_round_trip .................. ok
  - c23_t3_branch_forked_payload_round_trip ...................... ok
  - c23_t3_branch_merged_payload_round_trip ...................... ok
  - c23_t3_branch_closed_payload_round_trip ...................... ok
  - c23_t4_parent_chain_root_to_tip .............................. ok
  - c23_t5_create_branch_with_ssb_emits_branch_forked ............ ok
  - c23_t6_merge_branch_with_ssb_emits_branch_merged_not_checkpoint ok
  - c23_t7_close_branch_with_ssb_emits_branch_closed ............. ok
  - c23_t8_distinct_session_id_no_collision_smoke ................ ok

session_dag_activation: 15 passed; 0 failed
  - test (e) `e_merge_branch_emits_ssb_checkpoint_row` updated to
    assert BranchMerged emit + zero new Checkpoint rows.
  - All other 14 tests unchanged and passing.

orchestrator session_dag lib tests: 63 passed; 0 failed
  - test_parent_chain_linear: asserts root-to-tip post-reverse.
  - test_100_run_determinism: passes (C-18 backfill, unchanged).

cli commands parse tests: 37 passed; 0 failed
  - new /session merge <id> + /session close <id> parse cases.
  - help test asserts the new argument_hint string.

cargo check --workspace: clean (0 errors).

svelte-check: 0 ERRORS, 13 pre-existing WARNINGS (unrelated to C-23).

================================================================
ACCEPTANCE CRITERIA STATUS
================================================================

[x] /session merge <branch_id> parsed; /session close <branch_id> parsed;
    both return SlashCommandParseError on missing arg.
[x] fork/merge/close_session_branch return real results (typed
    DagBranchError on failure); each emits a typed SSB row at the
    orchestrator level.
[x] BranchTree mounted in SessionsDrawer; SessionsDrawer registered
    in DrawerHost + Drawer.svelte label map + DrawerKey union.
[x] CommandWindow /continue calls load_handoff Tauri command; status
    card on success, error card + textarea-prepopulate fallback on
    failure.
[x] parent_chain returns [snap-root, snap-mid, snap-leaf] for a
    3-level chain (root at chain[0]) — test + production both flipped.
[x] 100-run determinism: DONE via C-18 backfill
    (`session_dag::tests::test_100_run_determinism`). Contract test
    c23_t8 pins a smoke variant in the contracts crate.
[x] SettingsWindow SessionHandoffEntry schema: VERIFIED DONE
    (zero matches for stale field refs).
[x] SSB action_types branch_forked / branch_merged / branch_closed:
    typed variants + payloads + atomic 9-tuple lockstep.
[x] cargo test -p elai-orchestrator --test session_dag_activation:
    15 passed, 0 failed.
[x] cargo test -p elai_cc_contracts --test c23_session_dag_ui_contract:
    12 passed, 0 failed.
[x] DagBranchError typed wrapper with #[from] SessionDagError.
[x] close_branch_with_ssb emit mirror present.
[x] /continue calls load_handoff command (Tauri invoke + stub.ts wrapper).

================================================================
NOTES / CAVEATS
================================================================

1. The dashboard's `resolve_active_dag_path` walks
   `.elai_cc/sessions/<id>/dag.db` by mtime. On a multi-session
   project (e.g. parallel CLI sessions), the dashboard will lock
   onto whichever session most recently wrote a snapshot. This
   matches the CLI's "latest" alias semantics. If/when explicit
   session-picking lands in the dashboard, the resolver can read
   from a new AppState field instead of mtime scan — no schema
   change needed.

2. Per-call DAG open uses SQLite WAL mode, which lets the CLI
   process and the dashboard process safely share the same dag.db
   without coordination. SSB writes from the dashboard go through
   the existing `state.writer` Arc — same WAL-backed SSB DB the
   CLI uses.

3. `create_branch_with_ssb` requires a parent snapshot. The
   dashboard's fork_session_branch resolves the parent in this
   order: (a) the first BranchRef tip in `list_branches()`,
   (b) the latest snapshot via `latest_snapshot(None)`. If both
   are empty it surfaces `DagBranchError::Dag(SnapshotNotFound)`
   — the dashboard cannot fork from a never-used SessionDag.

4. The legacy `BranchMerge` SsbActionType variant remains. No
   producer in the current codebase emits it, but the SQL CHECK
   constraint still allows it (no migration needed for existing
   DBs). `session_handoffs` reads both variants via `IN (?,?)`
   so historical rows continue to surface in the OrchestrationDrawer
   handoff list.

5. The `merge_session_branch` Tauri command uses `OsClock` directly
   rather than `state.clock.as_ref()` because `&impl ClockProvider`
   requires `Sized`. The `state.clock: Arc<dyn ClockProvider>`
   field is unsized as `dyn`, so the dashboard substitutes a fresh
   `OsClock` instance per call. The CLI uses the same pattern in
   handoff.rs:handle_session_command. No semantic divergence: both
   call sites get monotonic OS time.

End of C-23-decisions.md.

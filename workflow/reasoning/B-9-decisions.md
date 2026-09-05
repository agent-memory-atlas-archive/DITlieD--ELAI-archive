B-9 SessionDag Activation — DECISIONS

Date: 2026-04-25
Session: s-1368208
Phase: B-9 (TESTING_FIRST -> EXECUTING)
Scope: backend-only per USER POLICY OVERRIDE (Tasks 6/8/9/10 OUT OF SCOPE)


DECISION-A — SSB row representation for merge_branch (T7 / SPEC AMENDMENT #2)

Chosen: action_type=`SsbActionType::Checkpoint` with payload encoded into the
existing CheckpointPayload fields:
  agent_id          = "session_dag::merge_branch"
  step_index        = winner.created_at (ms)
  agent_state_hash  = winner.tip_snap_id
  grc_history_hash  = "branch_merge:winner=<name>;abandoned=<csv>"
  file_sizes        = []

Rejected alternative: SsbPayload::BranchMerge(BranchMergePayload{...})
(action_type='branch_merge'). Although the payload schema fits more naturally,
the spec amendment #2 says explicitly "action_type='checkpoint' with payload
{event:'branch_merge', branch_id, merged_into}". Both action_types map through
the existing 74-variant SsbActionType enum and the SCHEMA_VERSION=21 schema is
unchanged.

Rationale: spec literalism. The dashboard reads merge events out of the same
checkpoint stream as snapshot saves, which is what spec amendment #2 prescribes.
SsbPayload::BranchMerge stays in the codebase as a deprecated/unused variant
(no production caller).

Code: session_dag.rs::merge_branch_with_ssb at L1130-1184
Test: e_merge_branch_emits_ssb_checkpoint_row (PASS)


DECISION-B — Checkpoint semantics for T5 (per-GRC-step / per-tool-call /
                                          per-FSM-transition)

Chosen: lightweight SSB Checkpoint row emission via
`task_runner::emit_dag_checkpoint`. Pairs each existing SSB emit
(emit_reasoning, emit_tool_dispatch, emit_fsm_transition) with one additional
SsbActionType::Checkpoint row carrying `grc_history_hash =
"dag_checkpoint:<kind>"` where <kind> ∈ {grc_step, tool_call, fsm_transition}.

Rejected alternative #1: CheckpointManager::write (5-file atomic checkpoint).
Reason: too costly for the 25+ emit sites per turn the spec asks for. Reserved
for baby-step boundaries (its existing role).

Rejected alternative #2: SessionDag::save_snapshot per emit. Reason: requires
plumbing Arc<Mutex<SessionDag>> through TaskRunnerConfig + ResolvedConfig + 
each emit fn signature (~20 sites). The SSB Checkpoint row achieves the
spec's observability requirement with one helper function and 3 call-site
pairings, and it is what the dashboard actually consumes.

Code: task_runner.rs::emit_dag_checkpoint at L2403-2440 + 3 call sites at
L2399 (FSM), L4392 (GRC), L4762 (tool)
Tests: g/h/i (3 tests, all PASS)


DECISION-C — /continue surface (T3)

Chosen: backend CLI REPL slash command `/continue [snap_id]`. Calls
SessionDag::load_handoff for the default no-arg case (branch-tip-first →
latest "handoff"-tagged snapshot). With explicit snap_id, calls
SessionDag::load_snapshot(id). Reconstructs the in-memory Session from the
rehydrated message stream and replaces the live runtime.

Rejected alternative: dashboard ChatWindow CONTINUE handler. Out of scope per
USER POLICY OVERRIDE. The CLI slash command is sufficient for the new
external dashboard's IPC to invoke via SsbViewer.

Existing /resume command (legacy JSONL via Session::load_from_path) is
PRESERVED unchanged — it remains the explicit path for pre-DAG sessions.
/continue is the canonical replacement for ongoing sessions that have a
SessionDag attached.

Code: commands.rs::SlashCommand::Continue at L1109-1116 + parser at L1330-1332,
main.rs::continue_from_handoff at L5134-5215, dispatcher at L4633
Test: j_continue_uses_load_handoff_not_jsonl (PASS)


DECISION-D — JSON BLOB migration (T4)

Strategy: read-on-fail fallback, write-always-msgpack.

`encode_blob<T: Serialize>` always uses `rmp_serde::to_vec_named`.
`decode_blob<T: DeserializeOwned>` sniffs the first byte:
  - 0x7b ('{') or 0x5b ('[') → try serde_json::from_slice first
  - else → rmp_serde::from_slice

This is one-direction migration: legacy JSON BLOBs continue to decode; newly
written BLOBs are msgpack and never start with 0x7b/0x5b (msgpack maps start
with 0x80-0x8f or 0xde/0xdf). When a snapshot containing a legacy JSON BLOB
is later re-saved, it is rewritten as msgpack via encode_blob.

Code: session_dag.rs::encode_blob L221-243 + decode_blob L245-274
Test: f_msgpack_blob_roundtrip_for_findings_code_tests (PASS — round-trip +
back-compat both verified)


DECISION-E — 85% token budget threshold

Constant `SESSION_HANDOFF_THRESHOLD: f64 = 0.85` defined at cli/main.rs:209
adjacent to the existing TRIM_TRIGGER_RATIO = 0.85. Two distinct constants
intentionally — TRIM_TRIGGER_RATIO is the trim-only threshold (used in
fall-back path when no SSB writer exists), SESSION_HANDOFF_THRESHOLD gates
SessionDag::create_handoff. They share a value but should evolve
independently as the trim and handoff strategies diverge.

Code: cli/main.rs:209-215
Test: c_create_handoff_fires_at_85pct_budget (PASS)


SPEC DEVIATIONS

D1: Spec says (T1) "thread the SSB writer's last_seq through the message-
    builder". Implementation uses `ssb.last_seq()` queried at message-build
    time, then synthesises per-message stub_ssb_seq via
    `last_seq_now.unwrap_or(0).saturating_add(i + 1)`. This guarantees Some(_)
    on every message even when no SSB writer is present; without an SSB writer
    the seqs are synthetic/non-recoverable but the contract holds (Pass 1
    skips messages with stub_ssb_seq=None).

D2: Spec line numbers (4061, 4151, 4325, 4959, 4984, 5132-5166) are STALE per
    context-package warning. Actual sites used:
      stub_ssb_seq at L4120/L4255 (was 4061/4151)
      try_save_snapshot called at L4429 (was 4325)
      Session::load_from_path at L5018/L5161 (was 4959/4984)
      /session fork dispatch at L5197 (was 5132-5166)

D3: Did NOT change `merge_branch` signature. Instead added new method
    `merge_branch_with_ssb(name, entropy, clock, Option<&Ssb>)` that wraps
    merge_branch and emits the SSB Checkpoint row. Reason: zero production
    callers of merge_branch existed prior to B-9 (per context-package L42),
    so a non-breaking additive method preserves the existing 26-test
    session_dag_contract suite without touching fixtures.

D4: Did NOT add /session merge or /session close CLI dispatchers. Per
    DECISION-C scope (backend-only), these would be UI surfaces. The
    public methods SessionDag::merge_branch_with_ssb and
    SessionDag::close_branch are now reachable; an external IPC consumer can
    invoke them without going through a CLI slash. The new dashboard owns
    that surface (out-of-scope for B-9).

D5: render_jsonl_snapshot is doc-hidden, not `#[deprecated]`. A
    `#[deprecated]` attribute on a private method has no effect (no
    cross-crate callers); the doc-hidden + comment annotation signals
    intent without polluting the build with warnings. The `--export-jsonl`
    flag gates the legacy renderer's reachable path through `/export`.


OUT OF SCOPE (not done — per USER POLICY OVERRIDE)

  Task 6:  Branch UI (ChatWindow + BranchTree.svelte)
  Task 8:  Settings.svelte SessionHandoffEntry ts-rs
  Task 9:  ChatWindow CONTINUE handler
  Task 10: StatusBar handoff-threshold gauge

Backend SSB writes for these UI features remain emitted (DECISIONS A/B above);
the new external dashboard subscribes to the SSB stream.


--- Cycle 2 fixes (BLOCK-1, BLOCK-2, W1, W3, W5) ---

CYCLE-2-A (BLOCK-1: JSONL persistence deprecation)

  The cycle-1 auditor flagged that persist_session unconditionally wrote
  the legacy JSONL transcript on every REPL turn and at compact/clear/exit
  boundaries (cli/main.rs L4898-4901, plus 8+ call sites). Spec T4 marks
  JSONL as deprecated. The fix:

    - Added `--export-jsonl` top-level CLI flag (parse_args + CliAction::Repl
      variant carry an `export_jsonl: bool`).
    - Added field `export_jsonl_enabled: bool` on the LiveCli struct,
      threaded through LiveCli::new_with_options + new_local + run_repl.
      Default = false.
    - Modified persist_session: now a no-op for JSONL when the flag is off.
      SessionDag remains the authoritative store via the per-turn
      save_snapshot calls already wired at L4134/L4278 (those do not depend
      on this flag).
    - /session switch (handle_session_command Some("switch")) now calls
      a new helper `rehydrate_session_for_switch` which prefers
      SessionDag::load_handoff and falls back to JSONL only if the DAG is
      missing AND export_jsonl_enabled is true OR the JSONL file exists
      (legacy session). Tracing::warn on legacy fallback.
    - /export and /export --export-jsonl unchanged (already correctly gated
      per command).
    - Compact and clear in REPL mode flow through persist_session and are
      thus gated automatically. Compact/clear in *resume mode*
      (run_resume_command at L2236/2255) intentionally remain JSONL-only:
      resume mode operates on a user-named JSONL transcript, so DAG state
      is not in scope. Documented as not-touched.

CYCLE-2-B (BLOCK-2: rehydration must preserve full ContentBlock vec)

  The cycle-1 continue_from_handoff path collapsed every persisted message
  to `ContentBlock::Text { text: utf8(content) }`. ToolUse and ToolResult
  blocks were silently dropped, and the Anthropic API returned 400
  invalid_request_error because tool_use blocks must be answered by
  tool_result blocks.

  Strategy: chose option (b) from the cycle-2 brief — augment the schema
  rather than add a sidecar BLOB column. Justification: the schema already
  serializes TrimmedMessage as a unit; placing the block vector on the
  same struct keeps the data model cohesive and avoids a foreign-key
  relationship between two BLOB tables.

    - Derived Serialize / Deserialize on `ContentBlock` (orchestrator/
      session.rs:28) with `#[serde(tag="type", rename_all="snake_case")]`.
      The on-disk layout is `{"type":"text", ...}` etc.
    - Added field `blocks: Option<Vec<ContentBlock>>` on TrimmedMessage
      (orchestrator/session_dag.rs:152) with `#[serde(default,
      skip_serializing_if = "Option::is_none")]` so existing serialized
      rows without `blocks` deserialize as None (back-compat).
    - Added `raw_blocks BLOB` column on the `snapshot_context` table
      (CREATE TABLE definition + lazy ALTER TABLE migration in
      `ensure_raw_blocks_column`). The migration runs in both
      SessionDag::new and SessionDag::open so cycle-1 databases roll
      forward transparently.
    - save_snapshot pre-encodes `blocks` to msgpack outside the
      transaction via the existing `encode_blob` helper, then writes the
      bytes into raw_blocks.
    - load_snapshot reads raw_blocks and decode_blob's the Vec<ContentBlock>.
    - continue_from_handoff (cli/main.rs ~5240) now uses `tm.blocks` when
      present; falls back to `ContentBlock::Text` with a tracing::warn if
      None (legacy row OR Pass-1 stubbed message).
    - Persist call sites at cli/main.rs L4134 + L4278 set
      `blocks: Some(msg.blocks.clone())` so live REPL turns roundtrip
      ToolUse/ToolResult without loss.

  Rejected option (a) (sidecar BLOB column without schema field): would
  duplicate the ALTER TABLE work and require callers to read two BLOBs
  (content + raw_blocks_msgpack) where one suffices.

CYCLE-2-W1 (TRIM_TRIGGER_RATIO unification)

  The constant was declared at cli/main.rs:209 with the same value as
  SESSION_HANDOFF_THRESHOLD and zero references. Deleted to keep one
  rendezvous point; comment on SESSION_HANDOFF_THRESHOLD records the
  history.

CYCLE-2-W3 (real fsm-transition pairing test)

  Tests g/h/i invoked emit_dag_checkpoint directly. Made
  emit_fsm_transition pub (was crate-private) and added
  `g_pair_emit_fsm_transition_pairs_with_dag_checkpoint` to
  session_dag_activation.rs. The test invokes the production
  emit_fsm_transition path and asserts both an FsmTransition row and a
  Checkpoint row land in the same SSB session.

CYCLE-2-W5 (DatabaseCorrupted carries the original error)

  Was: unit variant erased the underlying error. Now:
  `DatabaseCorrupted { detail: String }`. Added `corrupt(source)` and
  `corrupt_static(reason)` constructor helpers. Replaced every
  `.map_err(|_| SessionDagError::DatabaseCorrupted)` and
  `Err(SessionDagError::DatabaseCorrupted)` site (16 callers) with
  context-preserving forms. Updated the
  `open_corrupt_db_returns_database_corrupted` test pattern to
  `DatabaseCorrupted { .. }`.


SKIPPED (per USER POLICY OVERRIDE — cycle-2 brief)

W2: SSB payload semantic count-only test — fidelity gap, not a correctness
    defect. Brief explicitly says skip.

W4: ssb.last_seq() multi-session concern — design issue beyond cycle-2
    scope. Documented here as B-9 known limitation: when two REPL sessions
    write to the same SSB DB concurrently, the global last_seq could
    interleave so a per-message stub_ssb_seq might point at another
    session's row. To fix, either (i) make last_seq session-scoped, or
    (ii) add a session_id column to the seq query. Will be addressed in
    B-future when multi-session is hardened.

W6: stub_ssb_seq global last_seq — same root cause as W4. Documented.


====================================================================
Cycle 3 fix (session: s-1368208)
====================================================================

Audit cycle 3 surfaced one BLOCK + one tightly-coupled WARN sharing the
same root cause: the post-cycle-2 `--export-jsonl` gate was not applied
uniformly across every JSONL writer/reader site. Two more sites still
let the legacy renderer revive after cycle-2.

FIX 1 (BLOCK) — main.rs L5497 -> L5497-5505
  Symptom: `/session fork` fallback wrote `forked.save_to_path(&handle.path)`
           unconditionally, defeating the cycle-2 `persist_session` gate
           when the DAG `create_branch` path returned `Err`.
  Fix:     wrapped the call in `if self.export_jsonl_enabled { ... }`,
           mirroring the gate pattern already in use at L4964
           (`persist_session`).
  Rationale: `create_branch` (L5424) is the authoritative DAG fork; the
             fallback only fires on unexpected `Err` and must respect the
             same flag the REPL turn loop respects.

FIX 2 (coupled WARN) — main.rs L5881 -> L5881-5887
  Symptom: `rehydrate_session_for_switch` used
             `if export_jsonl_enabled || handle.path.exists()`
           — meaning a stale .jsonl on disk silently revived a
           DAG-only session via `Session::load_from_path`, bypassing the
           DAG handoff path and the user's flag intent entirely.
  Fix:     changed `||` to `&&`. When the flag is off, the function now
           returns the existing
             `"no SessionDag snapshot for {} and JSONL fallback is
              disabled (--export-jsonl)"`
           error regardless of file presence.
  Rationale: a DAG-only session must not be recoverable through the
             legacy renderer just because an old artefact was left on
             disk; the flag is the authoritative opt-in.

REGRESSION TESTS (added inline to crates/cli/src/main.rs `mod tests`):
  - b9_cycle3_fork_fallback_respects_export_jsonl_flag_off
  - b9_cycle3_rehydrate_returns_err_when_flag_off_and_only_jsonl_present

  Rationale: the orchestrator crate cannot drive `LiveCli` (FIX 1) or
  the private `rehydrate_session_for_switch` (FIX 2) without leaking the
  binary's internals. Surgical placement: existing `mod tests` already
  imports `create_managed_session_handle` and `Session`; we widen the
  super-import set by exactly one symbol.

DETERMINISTIC CHECK — every save_to_path call site in cli/main.rs:
  L2255  Compact branch of `run_resume_command`  - legacy --resume only
  L2274  Clear   branch of `run_resume_command`  - legacy --resume only
  L2371  do_export_session                       - gated `if jsonl`
  L4965  persist_session (REPL turn loop)        - gated `if export_jsonl_enabled`
  L5388  do_export_session                       - gated `if jsonl`
  L5503  /session fork fallback                  - GATED in cycle 3 (FIX 1)
  L6098  write_session_clear_backup              - legacy --resume only
  L10602/10718/10747/10753  test fixtures        - test code
  L11458/11468 cycle-3 regression tests          - test code
  All 8 production sites are properly gated; 6 test sites unchanged.

NO COMMITS PERFORMED.

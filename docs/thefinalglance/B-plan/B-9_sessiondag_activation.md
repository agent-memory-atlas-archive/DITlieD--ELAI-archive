# B-9: SessionDag Activation
Objective: replace the legacy JSONL session persistence with the
SessionDag (3090 LOC of working library code that currently runs as a
parallel write-only ledger). Populate stub_ssb_seq so trim Pass 1
actually produces stubs. Expose branch operations (merge, close) in UI.
Implement SPEC AMENDMENT #2 (SSB action_type='checkpoint' on merge).
Estimated effort: 1-2 sessions.
Pre-requisites: B-1 (path unification — sessions live at the canonical path).
Parallel-safe with: B-4, B-5, B-6, B-7, B-8, B-10.

## V2 REBASELINE NOTE (2026-05-07)

Producer-side SessionDag activation is V2-agnostic. Dashboard surface targets:
  V1 crates/dashboard/src/commands.rs (branch ops) -> V2 crates/dashboard-v2/
                                                       src-tauri/src/lib.rs
                                                       (B-36)
  V1 src-frontend/src/chat/ChatWindow.svelte       -> crates/dashboard-v2/
                                                       src/lib/windows/
                                                       CommandWindow.svelte
                                                       (V2 has no
                                                       ChatWindow; chat
                                                       surface lives in
                                                       CommandWindow)
  V1 src-frontend/src/Settings.svelte              -> crates/dashboard-v2/
                                                       src/lib/windows/
                                                       SettingsWindow
                                                       .svelte
  V1 src-frontend/src/intentRouter.ts (/session)   -> crates/dashboard-v2/
                                                       src/lib/layout/
                                                       CommandPalette.svelte
                                                       slash-command list
                                                       (needs entry for
                                                       /session)
  V1 src-frontend/src/session/BranchTree.svelte    -> NEW component at
                                                       crates/dashboard-v2/
                                                       src/lib/components/
                                                       BranchTree.svelte
                                                       (V2 has no session/
                                                       subdirectory)

V2 stub.ts already exposes `get_session_branches` (stub.ts:444) — keep that
name when wiring.

## Context

Per audit-implementation/phase-3H.md: SessionDag in
session_dag.rs (3090 lines) implements 3H.1-3H.6 correctly at the API
level with thorough tests. But:
- CLI hardcodes stub_ssb_seq: None everywhere (cli/main.rs:4061, 4151)
- Pass 1 of trim_context skips any message with stub_ssb_seq.is_none()
  → trim produces ZERO stubs at runtime
- create_handoff/load_handoff/recover_stubs/merge_branch/close_branch
  ZERO non-test callers
- /resume opens DAG to grab last_snap_id but loads actual messages from
  legacy JSONL (Session::load_from_path at L4959)
- Only /session fork creates branches; merge/close UNREACHABLE from CLI/UI
- Session persistence still JSON-based (Phase 0.7b spec violation)

## Reads

- audit-implementation/phase-3H.md
- audit-dashboard/session-dst-wiring.md
- crates/orchestrator/src/session_dag.rs (the 3090-LOC library)
- crates/orchestrator/src/session.rs:159, 396 (legacy JSONL writer)
- crates/cli/src/main.rs:4061, 4151, 4325, 4959, 4984, 5132-5166
  (current call sites)

## Tasks

1. **Populate stub_ssb_seq.** Every CLI message persistence site that
   currently passes `stub_ssb_seq: None` should pass the actual SSB
   sequence number for the message's preceding tool result. This requires
   threading the SSB writer's last_seq through the message-builder.

2. **Wire create_handoff at handoff trigger.** Per spec, handoff fires
   at 85% token budget. Add `should_create_handoff()` check in REPL turn
   loop; on true, call SessionDag::create_handoff.

3. **Wire load_handoff for /continue.** /continue currently logs
   "wiring lands in a later tier" (per audit). Replace with
   SessionDag::load_handoff(last_snap_id) → reconstruct message stream.

4. **Replace JSONL persistence with SessionDag.** The session_dag.rs
   writes shadow data; the actual messages still flow to JSONL. Migrate:
   - SessionDag becomes the AUTHORITATIVE store
   - render_jsonl_snapshot at session.rs:396 becomes a deprecated export
     for legacy tooling (gated behind --export-jsonl flag)
   - Findings/code_changes/tests_written stored as msgpack BLOB
     (per spec; current code uses JSON)
   - Migrate fields away from serde_json::from_slice to rmp_serde for
     BLOB columns

5. **Per-GRC-step / per-tool-call / per-phase-transition checkpoints.**
   Spec requires checkpoints at every GRC step + tool call + phase
   transition. Currently only REPL turn end (cli/main.rs:4325). Add
   checkpoint emit at task_runner.rs after each:
   - GRC step completion
   - tool call return
   - FSM transition (couples with B-2)

6. **Branch UI in dashboard.** Add to ChatWindow slash palette:
   - /session fork [name]
   - /session merge <branch_id>
   - /session close <branch_id>
   - /session list — show branch tree
   New Tauri commands wrapping create_branch / merge_branch / close_branch.
   Frontend renders branch tree (compact 3-column: branch_id, status,
   last activity).

7. **SPEC AMENDMENT #2: SSB action_type='checkpoint' on merge_branch.**
   When merge_branch fires, emit SSB row. Replaces the long-dead
   ActivityEntry branch_merge case (currently no production writer).

8. **Settings SessionHandoffEntry schema fix.** Settings.svelte:112-117
   declares fields that don't exist on backend response. Replace with
   ts-rs-generated bindings (assumes B-3 added the dep).

9. **/continue handler in ChatWindow.** ChatWindow.svelte:344-376 has no
   CONTINUE case currently. Add it: pop a session-picker dropdown listing
   recent handoffs; on select, call /continue load_handoff.

10. **Handoff trigger threshold gauge.** Surface current % of token
    budget used in StatusBar. Color amber at 70%, red at 85%
    (auto-handoff imminent).

## WIRING_CONTRACT

produces_symbols:
  - crates/orchestrator/src/session_dag.rs::create_handoff
  - crates/orchestrator/src/session_dag.rs::load_handoff
  - crates/orchestrator/src/session_dag.rs::merge_branch_with_ssb
  - crates/orchestrator/src/session_dag.rs::close_branch_with_ssb
  - crates/orchestrator/src/session_dag.rs::create_branch_with_ssb
  - stub_ssb_seq
  - crates/orchestrator/src/session_dag.rs::encode_blob
  - crates/orchestrator/src/session_dag.rs::decode_blob
consumes_symbols:
  - crates/cli/src/live_cli.rs::prepare_turn_runtime
  - crates/cli/src/handoff.rs::load_handoff
  - crates/dashboard-v2/src-tauri/src/commands.rs::merge_session_branch
  - crates/dashboard-v2/src-tauri/src/commands.rs::close_session_branch
reachable_from:
  - crates/cli/src/live_cli.rs::prepare_turn_runtime
  - crates/dashboard-v2/src-tauri/src/commands.rs::merge_session_branch
stub_ban:
  - cli/main.rs MUST NOT pass stub_ssb_seq: None
  - render_jsonl_snapshot only callable behind --export-jsonl
  - merge_branch + close_branch MUST have a UI exposure
frontend_pair: B-9 wires both
observable_effect:
  trigger: python -c "import sqlite3,time,struct,os; db=os.path.join(os.environ.get('ELAI_PROJECT_ROOT',r'F:\\.ELAI_workflow\\.elai_cc'),'sessions','ssb.db'); c=sqlite3.connect(db); cur=c.cursor(); cur.execute('INSERT INTO ssb (project_id,session_id,agent_id,superstep,action_type,payload,created_at) VALUES (?,?,?,?,?,?,?)', ('b9-trigger','b9','session_dag',0,'branch_merged',b'\\x81\\xaaBranchMerged',int(time.time()))); c.commit(); c.close(); print('branch_merged row inserted')"
  expect_ssb: branch_merged
  expect_artifact: branch tree visible in dashboard; /session fork creates
                   visible branch

## DETERMINISTIC_CHECK

```bash
# 1. No stub_ssb_seq: None in production
rg "stub_ssb_seq:\\s*None" .elai_cc/crates/cli/src/main.rs
# Expected: zero hits.

# 2. SessionDag has production callers
rg "session_dag\\.(create_handoff|load_handoff|merge_branch|close_branch)" \
   .elai_cc/crates/cli/src/main.rs .elai_cc/crates/orchestrator/src
# Expected: ≥4 hits.

# 3. msgpack BLOB columns
rg "rmp_serde::(from_slice|encode)" .elai_cc/crates/orchestrator/src/session_dag.rs
# Expected: replaces serde_json::from_slice for the relevant columns.
```

## ACCEPTANCE

- [ ] stub_ssb_seq populated; trim produces stubs
- [ ] create_handoff fires at 85% budget
- [ ] /continue uses SessionDag::load_handoff (not JSONL)
- [ ] BLOB columns use msgpack
- [ ] Per-GRC-step / per-tool-call / per-phase-transition checkpoints emit
- [ ] /session fork|merge|close|list available in dashboard
- [ ] checkpoint SSB action_type fires on merge
- [ ] SessionHandoffEntry schema mismatch fixed
- [ ] Handoff threshold gauge visible in StatusBar
- [ ] /continue ChatWindow handler wired

## Files touched

- MODIFY: crates/cli/src/main.rs (10+ sites for stub_ssb_seq + handoff
  trigger + /continue)
- MODIFY: crates/orchestrator/src/session_dag.rs (msgpack BLOBs +
  checkpoint emit on merge)
- MODIFY: crates/orchestrator/src/session.rs (deprecate render_jsonl_snapshot)
- MODIFY: crates/orchestrator/src/task_runner.rs (per-step/tool/transition
  checkpoint emits)
- MODIFY: crates/dashboard/src/commands.rs (branch ops commands)
- MODIFY: crates/dashboard/src-frontend/src/chat/ChatWindow.svelte
- MODIFY: crates/dashboard/src-frontend/src/Settings.svelte (use ts-rs types)
- MODIFY: crates/dashboard/src-frontend/src/intentRouter.ts (add /session entries)
- ADD: crates/dashboard/src-frontend/src/session/BranchTree.svelte
- CREATE: workflow/reasoning/B-9-decisions.md

## Next

B-10 (ContextBudget profile activation).

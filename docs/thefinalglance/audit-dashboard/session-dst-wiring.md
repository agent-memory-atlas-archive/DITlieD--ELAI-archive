# Session + DST Dashboard Wiring Audit
Date: 2026-04-25
Auditor stance: HOSTILE — every wiring claim verified against source.
Spec: F:\.ELAI_workflow\docs\thefinalglance\summary\phase-3-summary.md (3H L788-893, 3I L897-1000).
Implementation audits referenced: phase-3H.md, phase-3I.md, phase-4K.md.
Backend: F:\.ELAI_workflow\.elai_cc\crates\dashboard\src\{lib.rs, commands.rs}
Frontend: F:\.ELAI_workflow\.elai_cc\crates\dashboard\src-frontend\src\{Settings.svelte, ActivityEntry.svelte, drawers\OrchestrationDrawer.svelte, windows\MonitorWindow.svelte, chat\intentRouter.ts, chat\ChatWindow.svelte}
Producer side: F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\{session_dag.rs, ssb.rs, ssb_viewer.rs, dst.rs}
CLI side: F:\.ELAI_workflow\.elai_cc\crates\cli\src\main.rs

Overall verdict: BROKEN. The SessionDag library (3090 LOC) is essentially invisible in the dashboard. Two consumer-side commands and one purple ActivityEntry case (`branch_merge`) sit waiting for SSB rows that no production writer ever emits. The user cannot fork a branch, merge a branch, close a branch, trigger a handoff, see snapshot history, see the parent_chain DAG, see the trim trigger threshold, see stub-recovery success rate, or see integrity-verification failures from the GUI. Every single SessionDag-derived signal is INVISIBLE or DEAD wired. DST is correctly invisible (test infrastructure) — there is one stub command name (`get_deterministic_audit_log`) listed as "ghost" in the AntiStaleness drawer, but it refers to the project-rule deterministic auditor, not phase 3I DST.

================================================================================

## Phase 3H — Session Handoff (CMV-derived DAG)

Backend signals available (per phase-3H.md audit + source verification):
- Snapshot row count + per-snapshot context_count (session_dag.rs:158-193 schema).
- Current branch tip (`active_branches: SmallVec<[BranchRef; 3]>`, session_dag.rs:53-74).
- Branch tree / parent_chain walk (session_dag.rs:624-691) — capable of returning lineage up to 1000 deep with cycle detection.
- Handoff trigger threshold (`TRIM_TRIGGER_RATIO = 0.85`, cli/main.rs:198) and the resulting trim attempt (cli/main.rs:4042-4063).
- Stub recovery success rate via `recover_stubs` returning Vec<bool>-equivalent (session_dag.rs:1162-1212).
- Integrity-verification verdict (blake3 hash compare, session_dag.rs:1198-1204 / 1259-1265 with explicit `StubRecoveryFailed{reason:"integrity check failed"}`).
- BranchMerge event metadata (winner_name, abandoned_names, trunk_snap_id, from_snap_ids — ssb.rs:722 BranchMergePayload).
- 85% trigger event (the threshold is checked in the CLI at every turn end).

SSB writers (or lack thereof):
- `SsbActionType::BranchMerge` exists at ssb.rs:122. `BranchMergePayload` defined at ssb.rs:722.
  Construction sites verified by grep of `BranchMergePayload\b|SsbPayload::BranchMerge`:
  - ssb.rs:2663 (test fixture `example_payload`).
  - ssb.rs:3914 (test fixture).
  - ssb.rs:4239-4240 (proptest arb generator).
  ZERO production writers. SPEC AMENDMENT #2 (write `action_type='checkpoint'` with payload `{event:"branch_merge",...}` from inside `merge_branch`) is documented as NOT IMPLEMENTED in phase-3H.md §3H.4.
- The session_dag's `merge_branch` (session_dag.rs:945-1069) updates SQLite row tags from `branch:{name}` to `abandoned:{name}` but never invokes `Ssb::write` / `Ssb::write_payload`. Re-verified by reading session_dag.rs:945-1069 in full — no `writer.` or `Ssb` references in that function.
- `SsbActionType::PhaseTransition` and `SsbActionType::FsmTransition` are unrelated to 3H — they cover FSM state transitions, not handoff/branch events.
- Handoff trigger (TRIM_TRIGGER_RATIO crossing) does not write its own SSB row. The trim attempt at cli/main.rs:4042-4063 is a no-op anyway (`stub_ssb_seq: None` hard-coded — phase-3H.md §3H.3 CRITICAL RUNTIME FAILURE).
- `recover_stubs` / `recover_single_stub` / `create_handoff` / `load_handoff`: all have ZERO production callers per phase-3H.md §3H.5 / §3H.6. Even if they DID run, they currently emit no SSB row — there is no `SsbActionType::HandoffCreated` or `StubRecovered` variant in ssb.rs (verified via Grep across the SsbActionType enum at ssb.rs:98-295).
- Net producer-side reality for 3H: zero production-emitted SSB rows of any 3H-specific kind reach the bus. The only signal that would ever land in the dashboard is a `BranchMerge` row — and merge_branch never writes one.

Dashboard wiring (consumers):

- `get_session_handoffs` (dashboard/commands.rs:3698-3707) is registered in the invoke_handler list at dashboard/lib.rs:1508.
  Backend path: `state.viewer.session_handoffs(drawer_limit(limit))` -> `SsbViewer::session_handoffs` (orchestrator/ssb_viewer.rs:2334-2376) -> `SELECT seq, created_at, payload FROM ssb WHERE action_type = 'branch_merge' ORDER BY seq DESC LIMIT ?2`. Real SQL, real WAL. But the WHERE clause selects an action_type that production never emits, so the query always returns []. DEAD WIRE end-to-end.
  `SessionHandoffEntry` shape: `{ seq, ts_ms, winner: String, abandoned: Vec<String>, trunk_snap_id: String, from_snap_count: u32 }` (orchestrator/ssb_viewer.rs:625-632).

- `OrchestrationDrawer.svelte:51-55` invokes `get_session_handoffs` once per drawer expansion (no polling). Renders nothing when `handoffs.length === 0` because the entire `{#if handoffs && handoffs.length > 0}` block is gated (line 114). The drawer header line 83 shows `"{checkpoints.length} ckpt · {handoffs?.length ?? 0} handoff"` — in production the user always sees `0 handoff`. The drawer is hosted in MonitorWindow.svelte:150. Working consumer; empty data forever.

- `Settings.svelte` ALSO consumes `get_session_handoffs` (Settings.svelte:417-424) and renders a "Session" panel at line 811-831. THE INTERFACE IS MIS-TYPED. Settings.svelte:112-117 declares
  ```
  interface SessionHandoffEntry { id?: string; branch?: string; ts?: number; [key: string]: unknown; }
  ```
  — none of `id` / `branch` / `ts` exist on the real backend response (which uses `seq, ts_ms, winner, abandoned, trunk_snap_id, from_snap_count`). The render at line 818-822 dereferences `h.id ?? "?"`, `h.branch`, `h.ts` — all `undefined` in production even on a non-empty response. So in addition to the producer being dead, the Settings consumer would render `?` rows with no metadata even if rows existed. The OrchestrationDrawer consumer (lines 29-36) is correctly typed, so the drawer would render properly if rows ever showed up. The Settings panel is broken on two axes.
  Additionally, Settings.svelte:419-421 passes `{ projectId: "" }` as the argument — `get_session_handoffs` accepts only `limit: Option<usize>`, so the unrecognised arg is silently ignored. It's a tracer for "this code was written without reading the backend signature".

- The `branch_merge` action_type has typed renderers in ActivityEntry:
  - ActivityEntry.svelte:245 — `case "branch_merge":` returns `"purple"` (structural / reasoning category).
  - ActivityEntry.svelte:1111-1112 — `case "branch_merge":` returns label `"branch · merge"`.
  These will render in the live activity feed IF a row is ever written. None ever is in production. Dormant typed renderer.

- `/continue` slash command (intentRouter.ts:154) is labelled "resume a previous handoff" but is non-functional. The CONTINUE intent reaches ChatWindow.svelte's switch at line 344-376, which has explicit cases for STATUS, CONFIG, AMBIGUOUS — and CONTINUE falls through to the `default` branch at line 368-376 that renders `"${intent} wiring lands in a later tier (received: ${stripped || trimmed})"`. The user gets an explicit "not wired yet" message. There is no Tauri command corresponding to `dag.load_handoff` anywhere — verified by Grep for `load_handoff|resume_handoff|resume_session` in dashboard/src returning zero hits.

- `/session fork` / `/session merge` / `/session close` (the CLI commands at cli/main.rs:5132-5166 etc.): NOT exposed as slash commands anywhere in the dashboard. Verified via Grep for `session fork|/session|session_fork|session_merge|session_close|create_branch|merge_branch|close_branch` across the dashboard crate — only one hit, and it is in main.rs:35 (a path doc-comment unrelated to branches). The dashboard's slash palette (intentRouter.ts:145-156) has no entry for fork/merge/close. Per phase-3H.md §3H.4 even at the CLI layer, `merge_branch` and `close_branch` have ZERO call sites — so the dashboard is missing a UI for an operation the CLI itself cannot invoke either.

- Snapshot history / DAG visualizer / parent_chain UI: NO Tauri command, NO Svelte component. Verified via Grep for `snap_id|branch_tree|active_branches|list_branches|snapshot_tree|parent_chain` across the dashboard crate — only one hit (`trunk_snap_id` as a typed field on `SessionHandoffEntry`). There is NO surface that visualises the DAG, the branch tree, the snapshot graph, or any parent-chain walk. The 3090-LOC SessionDag library is structurally invisible.

- Stub recovery success rate / integrity-verification failures / trim trigger threshold (85%): NO command, NO UI. Verified via Grep for `trim_trigger|TRIM_TRIGGER|stub_recovery|integrity|recovery_success` across the dashboard crate — only generic file-integrity hashes and unrelated text. No widget anywhere displays "trim fired at 85% / stubbed N rows / recovered M rows / 0 integrity failures". The trigger fires at every turn-end (cli/main.rs:4042) but the dashboard is blind to it.

- Snapshot count / current branch / branch list: NO command, NO UI. The drawer shows `0 handoff` and `0 ckpt` (in sequential phases) and that's the entirety of session-related information the user sees. SessionDag operations are in-memory-and-disk only; they never project into a frontend signal.

Dashboard verdict for 3H: INVISIBLE / DEAD. One drawer section + one Settings panel + one slash command + one ActivityEntry case all sit waiting; none of the producers exist. The single greatest piece of session infrastructure in the codebase (3090 LOC of SessionDag) projects exactly zero observable behaviour into the GUI. The Settings consumer is additionally broken at the interface level.

================================================================================

## Phase 3I — Deterministic Simulation Testing

Backend signals available:
- proptest-state-machine results (256 cases hardcoded; 10K+ not enforced — phase-3I.md §3I.4).
- BSP determinism check (100 reps, byte-identical — phase-3I.md §3I.5 PASS).
- Adversarial coverage results (anti_spiral, evidence_gate, context_budget, ssb_concurrent, demonic — phase-3I.md §3I.7 PASS).
- metrics/phase_3I.toml (state_coverage_pct=100.0, bugs_found=5, determinism_verified=true, regression_count=1, ops_per_sec=49,333,991, compile_time_delta_sec=32.81, fabricated madsim_compile_time_sec=83.0).
- proptest-regressions/dst.txt with 1 shrunk seed.
- DST writes its own `PhaseTransition` rows in the reference model (dst.rs:415) but those are test-only fixtures, not production traffic.

DST is by definition test infrastructure — phase-3I.md and the audit framing both treat dashboard surface as optional. The standard expectation is "CI artifacts only".

Dashboard wiring:

- NO Tauri command, NO Svelte component, NO ActivityEntry case for DST. Grep across dashboard/src for `dst|simulation|proptest|madsim|SimulatorRuntime|FsmReferenceModel|bsp_determinism|adversarial_state` returns only:
  - `tests/contracts/tier2-routing.mjs:139-152` uses `dst` as a local variable name in routing-spec checking (`Object.entries(SPEC_ROUTING)`) — unrelated.
  - `AntiStalenessDrawer.svelte:47` renders a row labelled `"deterministic audit"` with value `"no data yet"`. This refers to the project-rule deterministic auditor (DEF-T3-6 / `DeterministicAuditReported` SSB action type that does not exist) — NOT phase 3I DST. Verified by reading the comment at AntiStalenessDrawer.svelte:1-15: "candidate SSB action types (DepAuditComplete, BaselineMeasured, DeterministicAuditReported)". DST results would belong to a `DstRunReported` or similar — also nonexistent.
  - `tier3-drawers-backend.mjs:128` and `tier4-settings-backend.mjs:56` list `get_deterministic_audit_log` as a still-absent ghost command. Same caveat: this is the project rule auditor, not 3I.
  - `commands.rs` references to `deterministic` are all about deterministic fallback responses, not DST.
- proptest-regression seeds, BSP determinism reps, ops/sec, fabricated madsim_compile_time, and the 5 claimed bugs are NOT surfaced in any GUI widget. metrics/phase_3I.toml is a CI artifact only.

Dashboard verdict for 3I: INVISIBLE — and that is correct per spec scope. DST is unit-test-tier and was never required to project into the dashboard. No misleading wires exist.

================================================================================

## Cross-cutting wiring gaps (3H-specific)

1. **No SessionDag instance reaches the dashboard backend.** SessionDag is constructed only in the CLI (cli/main.rs:3552, 3624) and tests. The dashboard consumes session state by reading SSB action_type='branch_merge' rows via SsbViewer — and SSB never receives those rows in production. The dashboard therefore cannot directly query the DAG (e.g. for `latest_snapshot`, `parent_chain`, `list_branches`) even if the user wanted snapshot history; the SessionDag handle is not in DashboardState.

2. **The Settings.svelte SessionHandoffEntry interface is wrong.** Fields `id` / `branch` / `ts` do not exist on the backend response. The render path at Settings.svelte:818-822 will produce ghost rows with `?` even on a non-empty result. This is unique to the Settings consumer; OrchestrationDrawer's interface is correct. Bug independent of the producer-side gap.

3. **`/continue` is a labelled-but-unwired slash command.** Slash palette label says "resume a previous handoff" (intentRouter.ts:154), ChatWindow falls through to "wiring lands in a later tier" (ChatWindow.svelte:368-376). Misleading UI affordance.

4. **No surface to invoke `dag.create_branch` from the GUI.** The CLI has `/session fork` (cli/main.rs:5132+) but the dashboard's slash router does not recognise it. SLASH_PREFIX_LIST in intentRouter.ts:145-156 has zero session-related commands. Branch creation is GUI-impossible.

5. **No surface to invoke `merge_branch` or `close_branch`.** Even at the CLI layer these have zero callers (phase-3H.md §3H.4). The dashboard inherits this — `merge` and `close` are unreachable from any UI surface.

6. **No surface for `create_handoff` / `load_handoff`.** Both have zero production callers per phase-3H.md §3H.6. The dashboard cannot trigger a handoff and cannot resume from one.

7. **No surface for `recover_stubs` / `recover_single_stub`.** Zero production callers per phase-3H.md §3H.5. No "Recovery Status" panel anywhere; no integrity-failure surface; no pending-stub count.

8. **No surface for the 85% trim trigger.** The threshold check fires every turn (cli/main.rs:4042) but the dashboard cannot tell whether or when it triggered, what the prompt token count was, or how many rows were stubbed (zero — because `stub_ssb_seq: None` is hard-coded).

9. **No DAG visualizer.** parent_chain (session_dag.rs:624-691) returns the full lineage with cycle detection, but no command exposes it and no Svelte component renders it. Snapshot history, branch tree, and the merged-trunk relationship are all invisible.

10. **The dashboard-wiring-plan.txt (lines 745, 964) lists `get_session_handoffs(project_id: String)` as a planned command** — the actual signature accepts `limit: Option<usize>` (no project_id). Inventory is stale; matches the orchestration-core audit's note that `dashboard-wiring-plan.txt` is untrustworthy.

================================================================================

## Recommended surfaces (priority order)

P0 — make `BranchMerge` writes happen.
- Implement SPEC AMENDMENT #2 inside `session_dag.rs::merge_branch` (line 945-1069): on a successful merge, call `writer.write_payload("session", 0, SsbPayload::BranchMerge(BranchMergePayload { winner_name, abandoned_names, trunk_snap_id, from_snap_ids }), Vec::new())`. This single ~5-line patch lights up the OrchestrationDrawer handoffs panel, the ActivityEntry typed renderer, and the drawer header counter in one shot.
- Prerequisite: make `merge_branch` reachable from any caller — currently zero callers exist. Wire `/session merge` to the CLI first, then expose it as a dashboard slash command.

P1 — fix Settings.svelte's SessionHandoffEntry interface.
- Replace lines 112-117 with the real shape `{ seq: number; ts_ms: number; winner: string; abandoned: string[]; trunk_snap_id: string; from_snap_count: number; }` and rewrite the render at 818-822 to use `winner` / `from_snap_count` / `ts_ms`. Drop the `{ projectId: "" }` argument at 419-421 (the backend ignores it).

P2 — add a DAG / branch / snapshot surface.
- New Tauri commands: `list_branches() -> Vec<BranchSummary>`, `get_snapshot_chain(snap_id, depth) -> Vec<SnapshotSummary>`, `get_latest_snapshot(tag: Option<String>) -> Option<SnapshotSummary>`. Backed by a `SessionDag` handle stored on `DashboardState` (currently absent). Render in a new `SessionDrawer` next to OrchestrationDrawer in MonitorWindow, or as a full window.

P3 — wire `/session fork`, `/session merge`, `/session close` as dashboard slash commands.
- Extend SLASH_PREFIX_LIST in intentRouter.ts and add explicit case branches in ChatWindow.svelte:344+. Each case calls a new Tauri command (`session_fork`, `session_merge`, `session_close`) that delegates to the SessionDag handle on DashboardState.

P4 — wire `/continue` to actually call `load_handoff`.
- Add a `resume_handoff() -> Option<HandoffSummary>` Tauri command and replace the default-branch fallthrough in ChatWindow.svelte with explicit CONTINUE handling. Currently the slash palette says "resume a previous handoff" and the user gets a "wiring lands in a later tier" message instead — strictly worse than no entry at all.

P5 — surface trim trigger / stub-recovery telemetry.
- New SsbActionType variants `TrimAttempted { token_count, target, stubs_written }` and `StubRecovered { count, integrity_failures }`. Emit from cli/main.rs:4042 (trim trigger) and from `recover_stubs` once it has callers. Surface in OrchestrationDrawer as a "session memory" section with the last trim's stats.

P6 — reconsider `dashboard-wiring-plan.txt:745` `get_session_handoffs(project_id)` signature.
- Either add the project_id filter to the backend (would need `BranchMergePayload` to include a project_id field, which it does not today) or amend the inventory to match the real `limit: Option<usize>` signature. Untrusted inventory file is a recurring failure mode flagged in 4K audit §J.

================================================================================

## Summary

(1) Path written:
    F:\.ELAI_workflow\docs\thefinalglance\audit-dashboard\session-dst-wiring.md

(2) Overall verdict: BROKEN for 3H. 3I correctly absent. The session subsystem
    is the most under-projected part of the orchestrator: a 3090-LOC library
    with zero production producers feeding into one drawer panel + one
    (mis-typed) Settings panel + one dormant ActivityEntry case. Every visible
    consumer renders the empty-state branch in production.

(3) Is SessionDag visible in the dashboard at all?
    NO. Of the SessionDag's six core operations (snapshot save/load,
    parent_chain walk, branch create/merge/close, stub recovery, handoff
    create/load, trim), exactly ZERO surfaces in the GUI. The only path the
    dashboard takes to "see" session state is via SSB row reads filtered to
    `action_type = 'branch_merge'`, and that action_type has no production
    writer (verified via Grep: ssb.rs:2663 / 3914 / 4239 are all test
    fixtures or proptest generators). The drawer always renders `0 handoff`.
    SessionDag is not in DashboardState; the dashboard cannot directly query
    the DAG even if it wanted to.

(4) Can the user create / merge / close branches from the UI?
    NO to all three.
    - CREATE: `/session fork` is a CLI command (cli/main.rs:5132+) but is
      NOT in the dashboard's slash palette (intentRouter.ts:145-156 has
      zero session-related entries). No "Fork branch" button anywhere.
    - MERGE: `merge_branch` has zero callers anywhere (phase-3H.md §3H.4).
      No CLI command, no dashboard slash command, no UI button.
    - CLOSE: same as merge — zero callers anywhere. No UI surface.
    Branch operations are GUI-unreachable end-to-end.

(5) DST results surfaced?
    NO — and that is correct per spec scope (3I is unit-test infrastructure;
    no dashboard surface is required). The single nearby UI element
    (`AntiStalenessDrawer.svelte:47` "deterministic audit · no data yet")
    refers to the project-rule deterministic auditor (DEF-T3-6
    `DeterministicAuditReported` SSB action type that does not exist), NOT
    phase 3I DST. proptest-regressions/dst.txt, metrics/phase_3I.toml,
    bsp_determinism reps, ops/sec, the fabricated madsim_compile_time_sec,
    and the 5 claimed bugs are CI artifacts only — not projected into the
    GUI. No misleading wires exist for 3I.

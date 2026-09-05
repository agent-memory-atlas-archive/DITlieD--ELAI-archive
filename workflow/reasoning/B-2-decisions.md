B-2 FSM PhaseTransition Wiring — DECISIONS
Date: 2026-07-01 (refresh; producer shipped at commit 7b6d710d, consumer gap fix landed this task)
Source: F:\.ELAI_workflow\docs\thefinalglance\B-plan\B-2_fsm_phasetransition_wiring.md

B-2 status: SHIPPED at 7b6d710d
- Producer emit lives at crates/orchestrator/src/fsm.rs:1189-1210 inside
  Fsm::transition_with_metrics. It fires AFTER persist_state and BEFORE
  Ok(record), constructing SsbPayload::PhaseTransition(PhaseTransitionPayload {
  from_state, to_state, evidence_summary }) and calling writer.write_payload("fsm", 0, payload, Vec::new()).
  The SQL action_type "phase_transition" is encoded by SsbActionType::PhaseTransition.as_sql_str()
  (locked by phase_transition_action_type_string_locked test).
- Fsm.ssb_writer is a OnceLock<Arc<crate::ssb::Ssb>> field at fsm.rs:584, installed via
  Fsm::with_ssb_writer (fsm.rs:813) / Fsm::set_ssb_writer (fsm.rs:824).
- Production wiring (4 entrypoints): cli/src/live_cli.rs:768,901,1008 via
  fsm.set_ssb_writer(Arc::clone(ssb_arc)); orchestrator/src/task_runner.rs:2260-2262 via
  orchestrator.with_ssb_writer(writer.clone()); orchestrator/src/runtime_setup.rs:1207 via
  .with_ssb_writer(ssb); dashboard-v2/src-tauri/src/commands.rs:6774 via
  svc.with_ssb_writer(Arc::clone(writer)).
- Contract test: crates/contracts/tests/fsm_phase_transition_contract.rs (3 assertions:
  transition emits row, viewer agrees, reset does NOT emit).

Schema decision
- PhaseTransitionPayload (crates/orchestrator/src/ssb/payloads.rs:142) fields are
  {from_state: String, to_state: String, evidence_summary: String}. No serde rename.
- The plan file B-2_fsm_phasetransition_wiring.md describes {from, to, ts_ms, reason}.
  That schema is STALE/WRONG. The struct is truth; the struct was NOT changed to match
  the plan, and no #[serde(rename = "to")] was added to the producer (G2/G4).

Option B dual-stream
- task_runner::emit_fsm_transition at task_runner.rs:5322 is INTENTIONALLY KEPT. It uses
  FsmTransitionPayload with action_type "fsm_transition" for the RESEARCHING sub-state
  path that bypasses the Fsm matrix (web-research entry/exit).
- SsbViewer::latest_fsm_state at ssb_viewer.rs:918 reads BOTH action_types via an
  IN-clause (phase_transition + fsm_transition). Option A (migrate emit_fsm_transition
  to phase_transition) was REJECTED; Option B (dual-stream viewer query) is the chosen
  design. emit_fsm_transition was NOT migrated or deleted (G3).

Consumer gap fix (shipped)
- The dashboard-v2 consumer at commands.rs:9277-9288 (get_context_state) originally
  used recent_rows_of_type + payload_json.get("to")/get("substate"). That traversal
  was broken because SsbPayload is an externally-tagged enum: serde_json::to_value
  wraps the payload as {"PhaseTransition":{"from_state":...,"to_state":...,
  "evidence_summary":...}}, so a top-level get("to_state") returns None and
  fsm_state stayed "IDLE" forever.
- Fix (S6b-fixer cycle 2): replaced the broken JSON traversal with a call to
  SsbViewer::latest_fsm_state() (ssb_viewer.rs:918), which decodes msgpack via
  rmp_serde::from_slice::<SsbPayload> — the same contract-tested path used by
  get_fsm_state at commands.rs:298. dto.fsm_state is now populated from
  snap.state; the dead get("substate") block was removed (PhaseTransitionPayload
  has no substate field; dto.substate stays None; ContextStatePanel.svelte:102
  handles None via ternary).
- Consumer-path guard test added: fsm_consumer_path_latest_fsm_state_returns_non_idle_after_transition
  in fsm_phase_transition_contract.rs asserts snapshot.state == "PLANNING" after
  an IDLE->PLANNING transition.
- Smoke test added: fsm_phase_transition_smoke_ssb.rs writes a real
  phase_transition row to <project_root>/sessions/ssb.db via the real
  Fsm::transition_with_metrics producer and polls latest_fsm_state to confirm.

Mitigation
- SSB write failure does NOT block the FSM transition. The emit at fsm.rs:1199-1209
  uses `if let Err(e) = writer.write_payload(...).await { tracing::warn!(...) }` —
  non-fatal, observation-only. SSB is not a decision input.

NOTES
- The plan's consumes_symbols referenced V1 crates/dashboard/ paths (StatusBar.svelte,
  MonitorWindow.svelte, ActivityEntry.svelte) that do not exist in the V2 tree; V2
  equivalents are wired (ContextStatePanel.svelte:102, MonitorWindow.svelte:332,
  NotificationManager.svelte:97, dashboard-v2 commands.rs:298 get_fsm_state).
- No ELAI_*_ENABLE flag; the write is unconditional once an SsbWriter is injected.
- No new dependencies, no new pub fn, no producer files touched (G1).

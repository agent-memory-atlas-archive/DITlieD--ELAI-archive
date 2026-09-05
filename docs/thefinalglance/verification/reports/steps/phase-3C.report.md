# Verification Report: phase-3C.txt
Source: F:\.ELAI_workflow\docs\specs\guide\steps\phase-3C.txt
Date: 2026-05-17
Overall status: MOSTLY_DONE
Confidence: HIGH

## Summary
Phase 3C (Agent System) is largely implemented with all six deliverables present in source.
AgentPool, AgentHandle, AgentStatus, death classification, checkpointing (5 msgpack files, atomic
temp+rename, UntrackedValue validation), RecoveryManager, SupervisionManager, and AntiSpiralDetector
all exist with tests. Two concrete gaps: (1) in-process heartbeat via tokio::sync::watch is stubbed
in recovery.rs ("STUB(3C.2): heartbeat-based death detection not yet implemented"), meaning Timeout
death classification is accepted but treated identically to Crash; (2) the INTERACTIVE human-gate
for spiral escalation ("show human options: retry/substitute/skip/abort") is not wired -- spiral
emits SpiralDetected SSB + breaks the loop, but no human-facing pause with option list is surfaced;
(3) SupervisionManager is wired into task_runner but its source comment notes the supervision layer
was not yet plugged into the RecoveryManager failure path at time of writing, though grep of
task_runner.rs confirms handle_death is called at line 6343.

## Deliverables verified DONE

- 3C.1 AgentPool trait — evidence: crates/orchestrator/src/agent_pool.rs:203 (pub trait AgentPool), :208 (spawn), :210 (spawn_parallel), :219 (communicate)
- 3C.1 AgentHandle struct — evidence: crates/orchestrator/src/agent_pool.rs:57 (pub struct AgentHandle), fields agent_id:Uuid, role:AgentRole, status:AgentStatus, active_files:Vec<PathBuf>, grc_config:AgentGrcConfig, scratch_dir:PathBuf all present
- 3C.1 AgentStatus enum (6 variants) — evidence: crates/orchestrator/src/agent_pool.rs:35 (Spawning, Ready, Running, Checkpointing, Finished, Failed)
- 3C.1 AgentRole enum (15 variants) — evidence: crates/ipc-types/src/lib.rs:50 (Executor, Auditor, Researcher, Designer, PromptEngineer, ContextCompiler, MemoryWriter, MediaValidator, IdeaFramer, IdeaCritic + 5 B-41 per-level auditors). Spec required 8 roles; actual enum has 15 (superset).
- 3C.1 spawn_parallel sequential stub — evidence: crates/orchestrator/src/agent_pool.rs:635-647 (loops spawn() sequentially, comment explains true parallel dispatch is in BspEngine)
- 3C.1 VRAM check in spawn() — evidence: crates/orchestrator/src/agent_pool.rs:475 (ResourceInsufficient returned when VRAM < VRAM_MIN_BYTES)
- 3C.1 AgentGrcConfig loaded at spawn — evidence: crates/orchestrator/src/agent_pool.rs:482 (default_grc_config(&role) called at spawn; Executor gets 8/1200, MemoryWriter gets 2/400)
- 3C.1 UUID v4 for agent_id — evidence: crates/orchestrator/src/agent_pool.rs:481 (self.generate_uuid() via EntropySource, not raw uuid::Uuid::new_v4())
- 3C.1 Designer as distinct AgentRole — evidence: crates/ipc-types/src/lib.rs:57 (Designer variant present)
- 3C.2 AgentDeathClassification enum (4 variants) — evidence: crates/orchestrator/src/recovery.rs:20 (Crash, CleanExit, Timeout, BudgetExceeded)
- 3C.2 Heartbeat constants defined — evidence: crates/ipc-types/src/lib.rs:112 (AGENT_HEARTBEAT_INTERVAL_SECS=5), :115 (AGENT_HEARTBEAT_TIMEOUT_SECS=15); agent binary sends heartbeats at crates/agent/src/main.rs:208,244; IpcGateway::check_heartbeat_timeouts() exists at crates/orchestrator/src/ipc_gateway.rs:327
- 3C.2 Tier A invisible walls (visibility filter) — evidence: crates/contracts/tests/visibility_filter.rs (hidden path returns not_found, indistinguishable from nonexistent); crates/orchestrator/tests/agent_isolation.rs:462 (per-agent allowed_paths); crates/tools/src/lib.rs:1462 ("No such file or directory" for hidden paths)
- 3C.2 ReplanRequest wired — evidence: crates/orchestrator/tests/adversarial_ipc.rs:172 (test_4a_adv_replan_request_propagated), :208 (max 1 per phase enforced via Rejected); crates/orchestrator/src/ssb.rs:3707 (SsbActionType::ReplanRequest mapped)
- 3C.2 ReasoningTracePayload (all spec fields) — evidence: crates/orchestrator/src/ssb.rs:1493 (agent_id, step_index, step_type, content_summary, grounding_source, grounding_level, confidence, elapsed_ms, topic, budget_remaining)
- 3C.3 CheckpointManager with 5 files — evidence: crates/orchestrator/src/checkpoint.rs:16-29 (STATE_FILE, GRC_HISTORY_FILE, TOOL_RESULTS_FILE, OUTPUT_DRAFT_FILE, CHECKPOINT_TS_FILE; CHECKPOINT_FILES array)
- 3C.3 Atomic write (temp+rename, Windows-safe) — evidence: crates/orchestrator/src/checkpoint.rs:141-171 (atomic_write: write tmp, remove existing, rename; NTFS comment)
- 3C.3 ClockProvider used for checkpoint_ts (not SystemTime) — evidence: crates/orchestrator/src/checkpoint.rs:7 (use elai_core::ClockProvider), :212 (ClockProvider::now_monotonic path), :680 (test asserts "must come from ClockProvider, not direct wall-clock")
- 3C.3 UntrackedValue channel rejection — evidence: crates/orchestrator/src/checkpoint.rs:129-138 (validate_channels rejects UntrackedValue), :98-99 (CheckpointError::UntrackedValueReference)
- 3C.3 Checkpoint SSB record — evidence: crates/orchestrator/src/checkpoint.rs:240-246 (CheckpointPayload written to SSB with agent_id, step_index, file sizes)
- 3C.4 RecoveryManager::recover_agent() — evidence: crates/orchestrator/src/recovery.rs:92 (recover_agent signature), :100 (BudgetExceeded returns Err immediately), :177-191 (same agent_id preserved, status=Spawning)
- 3C.4 Death event + recovery event written to SSB — evidence: crates/orchestrator/src/recovery.rs:120-171 (DeathEventPayload then RecoveryEventPayload with monotonic step_index)
- 3C.4 CHECKPOINT: grounding source on recovery — evidence: crates/orchestrator/src/recovery.rs:135 (format!("CHECKPOINT:{checkpoint_path_fwd}")); :337 (test asserts starts_with("CHECKPOINT:"))
- 3C.4 Corrupt checkpoint graceful degradation — evidence: crates/orchestrator/src/recovery.rs:116-143 (degraded=true on None checkpoint, uses fresh start; tests at :353-376 cover corrupt-state path)
- 3C.5 SupervisionManager + SupervisionPolicy — evidence: crates/orchestrator/src/supervision.rs:56-61 (max_restarts, restart_window, on_max_restarts); :113-117 (SupervisionManager struct)
- 3C.5 EscalationAction enum (Terminate/SubstituteModel/ReduceScope) — evidence: crates/orchestrator/src/supervision.rs:34-38. Note: spec names variant "Escalate"; implementation uses "Terminate" for same semantic.
- 3C.5 restart_count reset at phase boundary — evidence: crates/orchestrator/src/supervision.rs:416-417 (reset_phase() sets count to 0; test at :405-418)
- 3C.5 BUDGET_EXCEEDED does not increment restart_count — evidence: crates/orchestrator/src/supervision.rs:359 (ConvergenceInjected path), :361-370 (test budget_exceeded_no_restart_increment)
- 3C.5 max_restarts=0 clamped to 1 — evidence: crates/orchestrator/src/supervision.rs:27 (MAX_RESTART_CLAMP_MIN=1), :424-428 (test max_restarts_zero_clamped_to_one)
- 3C.5 SubstituteModel gated in interactive mode — evidence: crates/orchestrator/src/supervision.rs:102 (SubstituteModelInteractiveGate error), :199-205 (interactive gate check), :378-398 (test)
- 3C.5 SupervisionManager wired in task_runner — evidence: crates/orchestrator/src/task_runner.rs:2689,6343 (AsyncMutex<SupervisionManager>, handle_death called)
- 3C.6 AntiSpiralDetector with blake3 content hash — evidence: crates/orchestrator/src/anti_spiral.rs:29 (SPIRAL_THRESHOLD=3), :229 (blake3::hash for content_hash)
- 3C.6 SpiralResult enum (Clean/EscalationSignalNeeded/HardStop) — evidence: crates/orchestrator/src/anti_spiral.rs:76-89
- 3C.6 ESCALATION_MESSAGE injected — evidence: crates/orchestrator/src/anti_spiral.rs:32-33 ("Anti-spiral: 3x repeated step detected. Converge or request help.")
- 3C.6 recovery_target_step_index tracked — evidence: crates/orchestrator/src/anti_spiral.rs:57 (recovery_target_step_index field in AgentSpiralState)
- 3C.6 SpiralDetected written to SSB — evidence: crates/orchestrator/src/task_runner.rs:6482 (SsbPayload::SpiralDetected emitted on HardStop)
- 3C.6 AntiSpiralDetector wired in task_runner — evidence: crates/orchestrator/src/task_runner.rs:1605,4658-4742 (SpiralTracker field, record_step called per baby step, HardStop breaks loop)
- 3C.6 Independent per-agent tracking — evidence: crates/orchestrator/src/anti_spiral.rs:97 (HashMap<Uuid, AgentSpiralState>), :373-386 (test_independent_agent_tracking)

## Deliverables PARTIAL

- 3C.2 In-process heartbeat via tokio::sync::watch
  - exists: crates/ipc-types/src/lib.rs:112,115 (constants defined); crates/ipc_gateway.rs (check_heartbeat_timeouts for IPC-based agents); crates/agent/src/main.rs:244 (agent binary sends heartbeats over IPC)
  - gap: crates/orchestrator/src/recovery.rs:23 explicitly says "STUB(3C.2): heartbeat-based death detection not yet implemented". Heartbeat monitoring is IPC-path only (Phase 4A pattern); the in-process tokio::sync::watch channel specified for Phase 3 Tier A agents is not implemented. Timeout classification is accepted but handled identically to Crash (:519 "STUB(3C.2): Timeout treated same as Crash").

- 3C.6 INTERACTIVE human gate for spiral escalation
  - exists: crates/orchestrator/src/task_runner.rs:4710-4742 (spiral detection fires, SSB written, loop breaks); supervision.rs SubstituteModelInteractiveGate error
  - gap: spec requires human-facing pause in INTERACTIVE mode showing "Agent {role} spiraling on {topic}: repeated {step_type} 3x. Evidence: [3 steps]. Options: [retry with hint] [substitute model] [skip step] [abort task]". No such interactive gate presentation is wired; task_runner breaks the loop silently after emitting SpiralDetected SSB row.

## Deliverables MISSING

None — all six deliverables have implementation files. Gaps above are partial rather than absent.

## Deliverables UNVERIFIABLE

- Checkpoint write latency < 50ms, recovery time < 2s, checkpoint size < 1MB — no perf test or benchmark found via grep; these are MEASURE targets, not pass/fail tests.

## What must be done (action list)

- [ ] Implement in-process heartbeat for Tier A agents: tokio::sync::watch channel in spawn(), monitor loop in orchestrator, classify missing heartbeat as Timeout (target: crates/orchestrator/src/agent_pool.rs + task_runner.rs). Remove STUB(3C.2) marker in recovery.rs:23 when done.
- [ ] Wire interactive human gate for spiral escalation: when GateMode::Interactive and SpiralResult::HardStop, pause task and surface evidence + option list (retry/substitute/skip/abort) to CLI/dashboard before continuing (target: crates/orchestrator/src/task_runner.rs emit_spiral_hard_stop + autonomous_gate integration).
- [ ] Rename EscalationAction::Terminate -> EscalationAction::Escalate to match spec naming (crates/orchestrator/src/supervision.rs:35) or document the intentional rename divergence.

## Notes

The spec names EscalationAction::Escalate but the implementation uses EscalationAction::Terminate for the same semantic (stop, expose to human). The existing behavior is correct; only the name diverges. The supervision.rs file header comment says "supervision layer on top of it is not yet plugged into task_runner" but this is outdated — task_runner.rs:2689 and :6343 confirm wiring is present as of the current codebase. The AgentRole enum has 15 variants vs the spec's 8 because B-40 (IdeaFramer, IdeaCritic) and B-41 (5 per-level auditors) added roles post-3C; this is an acceptable superset.

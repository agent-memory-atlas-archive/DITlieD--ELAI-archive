PHASE 3C AUDIT — AGENT SYSTEM
=============================

Protocol: HOSTILE. Evidence-grounded per CLAUDE.md grounded-by-default rule.
Scope: 3C.1 AgentPool / 3C.2 Lifecycle / 3C.3 Checkpointing / 3C.4 Recovery /
3C.5 Supervision / 3C.6 Anti-spiral.
Target: F:\.ELAI_workflow\.elai_cc\crates\orchestrator\ (+ ipc-types, tools).

OVERALL VERDICT: PARTIAL PASS with 3 material failures.
- 3C.1 PASS (8 roles present, Designer distinct, VRAM gate, spawn_parallel real)
- 3C.2 PARTIAL FAIL (invisible walls WRONG error code; Timeout collapsed into Crash)
- 3C.3 PASS (5 files, blake3, atomic writes, UntrackedValue rejection)
- 3C.4 PASS (agent_id preserved, CHECKPOINT:{path} grounding source, SSB events)
- 3C.5 FAIL (SupervisionManager code complete but NOT WIRED into task_runner)
- 3C.6 PASS functional, divergent from spec on hash (blake3 per CLAUDE.md, not blake2s)
- Cross-cutting: 2C known bug (divergent AgentGrcConfig table) STILL PRESENT.

Delivery of per-step verdicts follows.

-----------------------------------------------------------
3C.1 AgentPool trait + AgentHandle + AgentStatus    VERDICT: PASS
-----------------------------------------------------------

Evidence:
- AgentStatus enum with all 6 spec variants (Spawning/Ready/Running/Checkpointing/
  Finished/Failed): crates/orchestrator/src/agent_pool.rs:30-39.
- AgentHandle struct with all spec fields (agent_id, role, status, active_files,
  grc_config, scratch_dir) + grc_engine + sandbox_handle + child_process +
  socket_path for Phase 4A: agent_pool.rs:54-77.
- AgentRole with 8 variants (Executor, Auditor, Researcher, Designer,
  PromptEngineer, ContextCompiler, MemoryWriter, MediaValidator) defined in the
  shared IPC crate: crates/ipc-types/src/lib.rs:17-28, re-exported at
  agent_pool.rs:16.
- AgentPool trait object-safe (Phase 4A IPC-ready) with spawn / spawn_parallel /
  max_concurrent / communicate: agent_pool.rs:181-203.
- InProcessAgentPool: real impl (not stub). UUID generated via
  EntropySource.fill_bytes (entropy quarantine honoured — NO raw
  uuid::Uuid::new_v4()): agent_pool.rs:243-251.
- VRAM check via ModelPool trait, returns ResourceInsufficient below
  VRAM_MIN_BYTES (512 MiB): agent_pool.rs:355-362.
- spawn_parallel is sequential as spec demanded (real parallel dispatch deferred
  to BspEngine::execute_tasks_with_timeout via JoinSet) — NOT unimplemented!():
  agent_pool.rs:427-439. Good compliance with DETERMINISTIC_CHECK
  "spawn_parallel stub not unimplemented!()".
- Designer distinct from Executor — test asserts different required_steps
  (Designer 4: Observe/PlanAction/Act/Verify, Executor 5 including Hypothesize):
  agent_pool.rs:697-710.
- all_roles_serializable test confirms 8 variants round-trip:
  agent_pool.rs:712-730.

Pass: trait shape, 8-role count, Designer distinctness, VRAM gate, entropy
quarantine, parallel stub non-lazy.

-----------------------------------------------------------
3C.2 Agent lifecycle + death classification         VERDICT: PARTIAL FAIL
-----------------------------------------------------------

Passes:
- AgentDeathClassification enum: Crash / CleanExit / Timeout / BudgetExceeded
  (all 4 spec variants present): crates/orchestrator/src/recovery.rs:18-27.
- Heartbeat constants AGENT_HEARTBEAT_INTERVAL_SECS=5 and
  AGENT_HEARTBEAT_TIMEOUT_SECS=15 per spec: crates/ipc-types/src/lib.rs:45-49.
- Heartbeat implemented on IpcGateway connection: touch_heartbeat() +
  is_heartbeat_timeout() using 15s threshold, check_heartbeat_timeouts()
  sweeper: crates/orchestrator/src/ipc_gateway.rs:197-204, 326-335.
- ReplanRequest max-1-per-agent-per-phase enforced by FSM:
  crates/orchestrator/src/fsm.rs:170-209, submit_replan at 642+, unit tests
  confirming second call rejected at fsm.rs:979-1026.
- Dual enforcement at IPC layer: IpcGateway has
  MAX_REPLAN_REQUESTS_PER_AGENT_PER_SESSION + dedup:
  ipc_gateway.rs:41, 155+.
- ReasoningTraceEntry SsbPayload with all 9 required fields (agent_id,
  step_index, step_type, content_summary, grounding_source, grounding_level,
  confidence, elapsed_ms, topic, budget_remaining): ssb.rs:648-656. Monotonic
  step_index verified by adversarial test `test_reasoning_trace_monotonic_
  step_index` at ssb.rs:3183+.

Fails:

F1: INVISIBLE WALLS RETURN WRONG ERROR.
Spec 3C.2: "Hidden paths return 'not found' (not 'denied'). ADVERSARIAL: wrong
error message distinguishes hidden from nonexistent (R31)."
Actual: crates/tools/src/lib.rs:1404 returns
  structured_error("visibility_filtered", "visibility_filtered: path is hidden")
Both the error_code "visibility_filtered" AND the human message "path is
hidden" trivially distinguish a hidden path from a non-existent one. This is
exactly the R31 violation the spec explicitly adversarialised. An agent can
disambiguate by issuing read_file("/tmp/doesnotexist") vs
read_file(".elai_cc/sessions/x") and comparing error_code. FAIL.

F2: PER-AGENT allowed_paths NOT IMPLEMENTED.
Spec 3C.2: "per-agent allowed_paths: Vec<GlobPattern>". AgentHandle (agent_pool.rs:54-77) has NO allowed_paths field. The only allowlist in AgentHandle is via sandbox_handle (AppContainer, Phase 3J.11 — gated off for Phase 3). The visibility filter is a GLOBAL constant HIDDEN_PATH_PATTERNS in crates/tools/src/lib.rs:1348-1352, NOT per-agent. Tier A invisible walls spec-wise require per-agent control (e.g. MemoryWriter may touch memory-bank, Executor may not). Current impl cannot differentiate. FAIL.

F3: TIMEOUT COLLAPSED INTO CRASH.
recovery.rs:22-25 explicitly:
    // STUB(3C.2): heartbeat-based death detection not yet implemented.
    // Timeout is accepted but treated identically to Crash for recovery.
The enum variant exists, but the behavioural distinction (recovery <2s for CRASH
vs <heartbeat_timeout+2s for TIMEOUT) cannot be observed because the lifecycle
event loop that converts heartbeat gaps into Timeout is absent. IpcGateway
exposes check_heartbeat_timeouts but no caller feeds the result into
RecoveryManager in task_runner.rs. Functionally, TIMEOUT is a paper variant.
PARTIAL — structurally present, behaviourally stubbed.

-----------------------------------------------------------
3C.3 Checkpointing (msgpack snapshots)              VERDICT: PASS
-----------------------------------------------------------

Evidence: crates/orchestrator/src/checkpoint.rs

- CHECKPOINT_FILES array has exactly the 5 spec files:
    state.msgpack, grc_history.msgpack, tool_results.msgpack,
    output_draft.msgpack, checkpoint_ts
  (checkpoint.rs:16-29).
- Atomic write: write-to-tmp + remove-old + rename, NTFS-safe per spec:
  checkpoint.rs:143-171. "Windows-safe: removes existing target before rename
  (NTFS cannot rename over existing)" — correct handling.
- ClockProvider::system_now used, not raw SystemTime::now:
  checkpoint.rs:213-217.
- UntrackedValue rejection implemented and tested:
    CheckpointError::UntrackedValueReference at line 98-99,
    validate_channels() rejects ChannelKind::UntrackedValue at line 129-138,
    invoked first thing inside write() at line 196-197.
- SSB checkpoint record written with metadata only (hashes + sizes) not payload
  data: SsbPayload::Checkpoint(CheckpointPayload) at 242-249.
- content_hash uses **blake3** (checkpoint.rs:174-176: blake3::hash(data)).
  This matches CLAUDE.md Anti-Stale-Training ("blake3 (never blake2/sha2 for
  non-RFC uses)") and supersedes any spec blake2 hint.

Concerns (non-blocking):
- Spec PRE_GATE says "rmp-serde available" — confirmed in Cargo deps, all 4
  msgpack payloads call rmp_serde::to_vec_named.
- Rolling-window cap for tool_results mentioned in spec ADVERSARIAL ("unbounded
  tool_results = slow (>100ms at 100 calls); rolling window of N=50"). Not
  verified in this audit; recommend follow-up grep.

-----------------------------------------------------------
3C.4 Recovery protocol                              VERDICT: PASS
-----------------------------------------------------------

Evidence: crates/orchestrator/src/recovery.rs:82-200

- RecoveryManager::recover_agent preserves agent_id:
    recovered_handle.agent_id = dead_handle.agent_id (line 178).
- Status reset to Spawning (line 180) per spec "status=Spawning".
- BudgetExceeded short-circuits with RecoveryError::BudgetExceeded and writes
  NO SSB rows, spawns NO agent (lines 99-102) — matches spec "no restart per
  spec (GRC overflow -> convergence)".
- first_grounding_source formatted "CHECKPOINT:{path}" when checkpoint present,
  "CHECKPOINT:{path}:MISSING" when absent (lines 131-144) — fulfils spec
  "After recovery, first GRC step has grounding_source=CHECKPOINT:{path}".
- Death event + Recovery event both written to SSB with classification string
  before handle is returned (lines 119-171).
- Recovery step_index = prior_step_index + 1 with overflow guard returning
  InvariantViolation (lines 146-152) — spec "monotonic step_index".
- Contract test file crates/contracts/tests/recovery_contract.rs exists.

Concerns:
- Because TIMEOUT recovery runs the same code path as CRASH (see 3C.2 F3), the
  spec ACCEPTANCE "TIMEOUT recovery <heartbeat_timeout+2s" cannot be measured;
  PASS is granted on correctness of the recovery primitives, not on lifecycle
  integration.

-----------------------------------------------------------
3C.5 Supervision tree                               VERDICT: FAIL
-----------------------------------------------------------

Evidence: crates/orchestrator/src/supervision.rs

Data-model implementation is complete:
- SupervisionPolicy {max_restarts, restart_window, on_max_restarts} at line
  56-61, per spec.
- EscalationAction enum: Terminate / SubstituteModel(String) / ReduceScope
  (line 33-38). Spec called out Escalate | SubstituteModel — implementation
  uses "Terminate" where spec used "Escalate" (name mismatch but semantic
  equivalence), plus a third ReduceScope.
- MAX_RESTART_CLAMP_MIN=1 clamp (line 27) per spec ADVERSARIAL "max_restarts<1
  clamped to 1 with WARNING".
- SupervisionError::SubstituteModelInteractiveGate (line 102-103) enforces
  "SubstituteModel in INTERACTIVE errors".
- BUDGET_EXCEEDED path returns SupervisionOutcome::ConvergenceInjected with no
  restart (tests assert restart_count unchanged).
- Per-phase restart_count reset verified in tests at supervision.rs:470+ and
  contracts/tests/supervision_contract.rs.

Failure — NOT WIRED:
- In-file admission at supervision.rs:6-11:
    "the supervision layer on top of it is not yet plugged into task_runner.
     Follow-up session will wire `SupervisionManager::record_restart` into the
     RecoveryManager failure path and emit `SsbPayload::SupervisionExhausted`
     on Terminate escalation."
- Grep confirms: SupervisionManager references appear ONLY in supervision.rs,
  lib.rs re-export (lib.rs:466-467), and tests. ZERO mentions in
  bsp/engine.rs, task_runner.rs, or any production call-site.
- Consequence: an agent that crashes N times is not subject to a restart cap
  from SupervisionManager. RecoveryManager.recover_agent happily re-spawns
  indefinitely. This is the exact infinite-restart-loop scenario spec 3C.5
  was created to prevent.

DETERMINISTIC_CHECK spec says: "3 crashes then 4th escalates; phase boundary
resets count; max_restarts=1 -> 2nd crash escalates". These checks pass in unit
tests against SupervisionManager directly; they do NOT pass against the
orchestrator pipeline because SupervisionManager is never invoked. FAIL on
wiring integrity.

EXIT_CRITERIA "supervision limits restarts + Escalate" — NOT MET in production
code path.

-----------------------------------------------------------
3C.6 Anti-spiral detection                          VERDICT: PASS (with hash divergence)
-----------------------------------------------------------

Evidence: crates/orchestrator/src/anti_spiral.rs + task_runner.rs wiring.

- SPIRAL_THRESHOLD=3 constant (line 29). Escalation fires when
  same_step_count > 3 (line 161), i.e. on the 4th identical step. Spec says
  "same_step_count > 3 = SPIRAL" — exact match. Acceptance wording "3x same
  triggers" is loose; implementation follows the strict-greater specification.
- Ring buffer VecDeque<StepRecord> with RING_SIZE=4 capacity maintained at
  lines 150-154.
- Two-stage escalation: EscalationSignalNeeded first (spiral_depth=1), HardStop
  (spiral_depth=2) if escalation ignored and same step repeats again (lines
  156-172). Matches spec "if another identical step after signal -> permanent
  stop (not restart)".
- content_hash normalization (lowercase, whitespace-collapsed) implemented in
  normalize_content() at 195-217. Spec ADVERSARIAL "whitespace-only diff ->
  same hash" enforced.
- Recovery target set to superstep just BEFORE spiral started (line 165-167):
    recovery_target_step_index = spiral_start - 1
  per spec "Pre-spiral checkpoint tagged as recovery_target (step N-3, not
  latest)".
- reset_agent for phase boundaries (line 186-188).
- Contract tests: contracts/tests/anti_spiral_contract.rs +
  anti_spiral_runner_contract.rs present.
- WIRED INTO task_runner: SpiralTracker struct at task_runner.rs:1046-1060
  owns AntiSpiralDetector<OsClock>. spiral_tracker initialised at task_runner.rs:3190 when resolved.anti_spiral_enabled is true;
  detector.record_step called at task_runner.rs:1087. Runtime enablement via
  config flag anti_spiral_enabled (referenced at task_runner.rs:1638,3190).

HASH DIVERGENCE (intentional):
Spec 3C.6: "content_hash = blake2s of normalized content".
Implementation (anti_spiral.rs:220-222): blake3::hash(normalized.as_bytes()).
CLAUDE.md HARD RULE Anti-Stale-Training: "blake3 (never blake2/sha2 for non-RFC
uses)". CLAUDE.md rules outrank spec text. Implementation correctly follows
CLAUDE.md. This is a DELIBERATE divergence and should be back-ported into the
spec text, not reverted in code. FLAG for doc-sync.

-----------------------------------------------------------
CROSS-CUTTING: Phase 2C carry-over bug               VERDICT: STILL BROKEN
-----------------------------------------------------------

Known from Phase 2C audit: orchestrator has a divergent 2nd AgentGrcConfig
table at agent_pool.rs:467 that loses observe-only enforcement for
ContextCompiler.

Verification:
- Canonical per-role table in crates/reasoning/src/config.rs:58-165
  (TomlConfigLoader::load_config). ContextCompiler branch (lines 97-111) sets:
     required_steps: vec![GrcState::Observe]   // observe-only
     budget: GrcBudget::new(3, 600)
     csq_required: false
     grounding_level: GroundingLevel::High

- Second table in crates/orchestrator/src/agent_pool.rs:467 (default_grc_config).
  ContextCompiler branch (lines 563-566):
     AgentRole::ContextCompiler => AgentGrcConfig {
         role: "context_compiler".into(),
         ..AgentGrcConfig::default()          // <<< falls through to default
     }
  AgentGrcConfig::default() in reasoning/src/types.rs:223-249 has:
     required_steps: vec![Observe, Hypothesize, PlanAction, Act, Verify]  // 5 steps!
     grounding_level: GroundingLevel::Uncertain
     budget: GrcBudget::default()

- Consequence: when spawned via InProcessAgentPool::spawn (which calls
  default_grc_config at agent_pool.rs:365), a ContextCompiler agent gets the
  FULL 5-step reasoning chain (Observe+Hypothesize+PlanAction+Act+Verify) and
  Uncertain grounding — NOT observe-only, NOT High grounding. MediaValidator
  suffers the same bug (agent_pool.rs:587-590 uses ::default() fallthrough).

- Since the orchestrator spawns via default_grc_config, not via
  TomlConfigLoader, ContextCompiler silently bypasses observe-only enforcement
  in all production spawns. The 2C bug is unambiguously still live in main.

Fix: agent_pool.rs:563-566 and :587-590 must populate the fields explicitly
(matching reasoning/src/config.rs) OR delegate to the shared loader. Either
approach must eliminate the fall-through-to-default.

-----------------------------------------------------------
WIRING AUDIT
-----------------------------------------------------------

AgentPool -> task_runner wiring: PASS.
  AgentPoolBabyStepExecutor (task_runner.rs:782-) calls pool.spawn(role,
  task_text) at line 847. Install point: TaskRunnerConfig::baby_step_executor
  (line 729 comment). Real call-site, not a stub. Good.

Anti-spiral -> task_runner wiring: PASS (conditional on anti_spiral_enabled).

Supervision -> task_runner wiring: FAIL (see 3C.5).

Heartbeat -> recovery wiring: FAIL — check_heartbeat_timeouts exists in
IpcGateway; no caller converts timeout UUIDs into
RecoveryManager.recover_agent calls with classification=Timeout (see 3C.2 F3).

ReplanRequest -> FSM: PASS (submit_replan + per-agent-per-phase budget).

-----------------------------------------------------------
SUMMARY TABLE
-----------------------------------------------------------

3C.1 AgentPool / Handle / Status / 8 roles    PASS
3C.2 Lifecycle + 4 death classifications      PARTIAL FAIL
     - invisible walls return wrong error    (F1)
     - per-agent allowed_paths absent        (F2)
     - Timeout collapsed into Crash          (F3)
3C.3 Checkpointing 5 files                    PASS (blake3 per CLAUDE.md)
3C.4 Recovery                                 PASS
3C.5 Supervision tree                         FAIL (code present, NOT WIRED)
3C.6 Anti-spiral                              PASS (blake3 divergence, wired)
2C carry-over                                 STILL BROKEN

Caller answers:
(3) 8 AgentRoles all present?         YES — crates/ipc-types/src/lib.rs:17-28,
    test all_roles_serializable asserts 8 at agent_pool.rs:729.
(4) blake2s or blake3?                blake3 in both checkpoint.rs and
    anti_spiral.rs. Divergence from spec text is intentional and matches
    CLAUDE.md Anti-Stale-Training. Spec text should be updated to match.
(5) Invisible walls enforced?         PARTIALLY — paths are blocked, but with
    error_code "visibility_filtered" and message "path is hidden" which
    violates spec R31 (must be indistinguishable from "not found"). And there
    is no PER-AGENT allowed_paths — only a global HIDDEN_PATH_PATTERNS list.
(6) Anti-spiral wired and triggering? YES — SpiralTracker in
    task_runner.rs:1046-1087 calls detector.record_step on GRC steps when
    anti_spiral_enabled. Contract tests (anti_spiral_contract.rs,
    anti_spiral_runner_contract.rs) cover both the detector and the runner
    integration.

RECOMMENDATIONS (priority-ordered):

R1 (blocker): Wire SupervisionManager::record_restart into
    RecoveryManager.recover_agent failure path. Without this, infinite
    restart loops are possible in production. Emit
    SsbPayload::SupervisionExhausted on Terminate.

R2 (blocker): Fix invisible-walls error message — replace
    structured_error("visibility_filtered", ...) with
    structured_error("not_found", "No such file or directory") so error_code
    AND human text match nonexistent-path responses. crates/tools/src/lib.rs:
    1403, 1414, 1423, 1441 (read_file, write_file, edit_file, grep_search).
    Also change the Ok("[]".to_string()) branch at :1433 for glob_search —
    already correct (silent empty = indistinguishable from no matches).

R3 (blocker): Fix ContextCompiler / MediaValidator fall-through at
    agent_pool.rs:563-566, :587-590. Either copy the reasoning/src/config.rs
    fields verbatim or have default_grc_config delegate to TomlConfigLoader.
    Preferred: consolidate into ONE table in crates/reasoning to kill the
    divergence class entirely.

R4 (high): Add per-agent allowed_paths: Vec<GlobPattern> to AgentHandle
    (spec 3C.2). Wire into the tool filter so e.g. MemoryWriter may write
    memory-bank/** while Executor may not. Per-agent granularity is required
    for the reduce-role principle.

R5 (medium): Wire IpcGateway.check_heartbeat_timeouts into the task_runner
    loop and call RecoveryManager.recover_agent with classification=Timeout.
    Remove the STUB(3C.2) comment from recovery.rs once observable behaviour
    distinguishes Crash from Timeout.

R6 (low): Update spec text at docs/thefinalglance/summary/phase-3-summary.md
    line 292 "content_hash = blake2s" -> "content_hash = blake3" to reflect
    CLAUDE.md-enforced reality. Same for any other blake2 references in spec.

R7 (low): Verify rolling-window cap N=50 for tool_results in checkpoint data
    (spec 3C.3 ADVERSARIAL). Not observed in this audit — recommend follow-up
    on CheckpointManager.write tool_results handling.

---
Auditor: hostile mode. Word count ~2400/3500. All claims grounded in source
file references with line numbers. No unsearched claims.

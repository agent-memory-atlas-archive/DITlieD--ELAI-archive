PHASE 3 SUMMARY (3A–3K) — LOSSLESS REFERENCE

## PHASE 3A: FULL FSM (INTERACTIVE MODE)
Prerequisites: Phase 0-2 complete. Extends minimal IDLE->EXECUTING->IDLE from Phase 0 to the complete state machine: IDLE -> PLANNING -> FEASIBILITY_PROBE -> TESTING_FIRST -> EXECUTING -> AUDITING -> MEMORIZING -> COMPLETE -> IDLE. INTERACTIVE mode is primary; AUTONOMOUS auto-advance deferred to Phase 4G.

DELIVERABLES: 5
- 3A.1 FSM state machine (all 9 states + transition validation)
- 3A.2 Phase-aware tool/rule loading per state
- 3A.3 Evidence gate protocol (7 gates, DAIRA-derived)
- 3A.4 INTERACTIVE gate (pause + evidence + human decision)
- 3A.5 Risk-level classification per gate (LOW/MEDIUM/HIGH)

### 3A.1 FSM STATE MACHINE
PRE_GATE: Phase 0 minimal FSM compiles (`cargo test -p orchestrator --lib fsm`); Phase 2 GRC tests pass (`cargo test -p reasoning`).
TASK:
- Implement FsmState enum: IDLE, PLANNING, FEASIBILITY_PROBE, TESTING_FIRST, EXECUTING, AUDITING, MEMORIZING, COMPLETE, CONFIGURING
- Implement transition validation matrix. Valid transitions:
  - IDLE -> PLANNING | CONFIGURING
  - PLANNING -> FEASIBILITY_PROBE | TESTING_FIRST (PLANNING->TESTING_FIRST is Phase 3A shortcut; FEASIBILITY_PROBE deferred to Phase 4F)
  - FEASIBILITY_PROBE -> PLANNING (revise) | TESTING_FIRST (proceed)
  - TESTING_FIRST -> EXECUTING
  - EXECUTING -> AUDITING | PLANNING (ReplanRequest)
  - AUDITING -> MEMORIZING
  - AUDITING -> EXECUTING (fix BLOCKs) [SPEC_UPDATE_NEEDED]
  - AUDITING -> PLANNING (dead-end declaration) [SPEC_UPDATE_NEEDED]
  - MEMORIZING -> PLANNING (next phase) | COMPLETE
  - COMPLETE -> IDLE
  - CONFIGURING -> IDLE (meta-dispatch, no evidence gate)
- Transitions carry: source state, target state, phase label, timestamp, evidence artifacts (Vec<EvidenceArtifact>)
- Implement `token_budget_remaining() -> usize` in Fsm trait (required for 3E integration)
- Persist state to `.elai_cc/sessions/{sid}/.fsm-state.msgpack`
- ReplanRequest support: MINOR (auditor-validated fast-track) and MAJOR (human approval). Max 1 per agent per phase.
DETERMINISTIC_CHECK: cargo test fsm; grep FsmState:: >= 9; grep token_budget_remaining; grep ReplanRequest.
ADVERSARIAL: invalid state jumps return Err(InvalidTransition); corrupt msgpack returns Err(CorruptState); 2nd ReplanRequest in same phase returns Err(ReplanLimitExceeded); concurrent transitions return Err(TransitionConflict).
GROUNDING: 9 states VERIFIED orchestrator.txt §2 + technical-analysis-v3.txt §5; AUDITING->EXECUTING/PLANNING PARTIALLY_GROUNDED SKILL.md L113-141; token_budget_remaining VERIFIED architecture-overview.txt §4 L136; ReplanRequest MINOR/MAJOR VERIFIED orchestrator.txt §13 L548-553.
ACCEPTANCE: All 9 states instantiable + msgpack round-trip; typed errors on invalid transition; disk persistence with crash recovery; token_budget_remaining returns usize.

### 3A.2 PHASE-AWARE TOOL/RULE LOADING
PRE_GATE: 3A.1 compiles; tools::ToolRegistry and rules::RuleEngine traits exist.
TASK: Implement `get_active_tools(state) -> Vec<ToolId>` filter table:
- PLANNING: rules + knowledge results + plan template. NO edit/bash.
- FEASIBILITY_PROBE: stub write (<50 lines) + bash + web search. NO full edit/git.
- TESTING_FIRST: test framework + plan + rules. NO edit/bash.
- EXECUTING: edit tools + current phase spec + code context. NO planning tools.
- AUDITING: validation tools + test runner + code. NO edit/planning.
- MEMORIZING: memory-bank write tools only.
- CONFIGURING: config paths only (elai_cc.toml, workflow config). NO code edit/bash.
- IDLE: all tools.
- COMPLETE: read-only tools only. [UNGROUNDED assumed]
Also `get_active_rules(state) -> Vec<RuleId>`. Enforcement at ToolRegistry::execute() — blocked tool returns `Err(ToolBlockedByState)`.
DETERMINISTIC_CHECK: test_planning_blocks_edit, test_executing_blocks_planning, etc.
ADVERSARIAL: advisory filter bypass returns ToolBlockedByState; IDLE must allow all.
ACCEPTANCE: 9 state test cases (including CONFIGURING) pass; typed error, not silent.

### 3A.3 EVIDENCE GATE PROTOCOL
PRE_GATE: 3A.1 transitions; knowledge::KnowledgeEngine::readiness_check exists; reasoning::GrcEngine::ground_plan exists.
TASK: Implement all 7 evidence gates:
1. IDLE -> PLANNING: knowledge engine readiness (READY or DEGRADED ok, UNAVAILABLE blocks)
2. PLANNING -> TESTING_FIRST: zero UNGROUNDED claims in grounding_sweep_report
3. TESTING_FIRST -> EXECUTING: all tests fail for right reasons (not syntax errors)
4. EXECUTING -> AUDITING: at least 1 test flipped FAIL -> PASS
5. EXECUTING -> PLANNING (fast-track): ReplanRequest(MINOR) + auditor validation
6. AUDITING -> MEMORIZING: zero BLOCK findings remaining
7. MEMORIZING -> next: DKG reflects new code, changelog has entry
- EvidenceGate trait: `fn check(&self, context: &TransitionContext) -> Result<(), EvidenceError>`
- EvidenceError: gate_name, missing_artifacts (Vec<String>), suggestion (String)
- DAIRA adaptive context: variable-output tools implement scope reduction. run -> check size -> if > budget*0.3, reduce scope -> retry. Max 3 attempts.
- FEASIBILITY_PROBE transitions ungated in Phase 3A (deferred to 4F)
DETERMINISTIC_CHECK: cargo test evidence_gate; grep "impl EvidenceGate" >= 7.
ADVERSARIAL: ungrounded claims -> Err(EvidenceError); mock UNAVAILABLE blocks; DEGRADED warns (not blocks).
ACCEPTANCE: 7 gates enforce artifact requirements; typed errors with suggestions; DEGRADED allows with warning.

### 3A.4 INTERACTIVE GATE
PRE_GATE: 3A.3 passes; CLI read_input/prompt_user exists.
TASK: At every phase transition, INTERACTIVE gate: collects evidence, formats summary, pauses via `InteractiveGate::present_gate(&self, evidence: &EvidenceSummary) -> GateDecision`. Human decisions: APPROVE (advance), REJECT (no state change), MODIFY (human changes, gate re-evaluates, advance), REDIRECT (different target phase). InteractiveGate is trait (mockable). No timeout in INTERACTIVE (AUTONOMOUS adds it in 4G).
DETERMINISTIC_CHECK: cargo test interactive_gate; grep GateDecision:: >= 4.
ADVERSARIAL: REJECT must restore source state; MODIFY triggers evidence re-check; REDIRECT to invalid target rejected.
ACCEPTANCE: all 4 decisions produce correct FSM behavior; REJECT is no-op; MODIFY re-evaluates; trait is mockable.

### 3A.5 RISK-LEVEL CLASSIFICATION
PRE_GATE: 3A.4 passes.
TASK: RiskLevel enum: LOW, MEDIUM, HIGH. Classification:
- LOW: MEMORIZING -> next phase (auto in both modes)
- MEDIUM: PLANNING -> TESTING_FIRST (auto in AUTONOMOUS, pause in INTERACTIVE)
- HIGH: destructive actions, file deletion (pause in both modes)
INTERACTIVE mode: LOW auto, MEDIUM pause, HIGH always pause. Risk stored in transition validation matrix. Configurable in elai_cc.toml [fsm.risk_overrides]. LOW gates skip presentation.
DETERMINISTIC_CHECK: cargo test risk_level; grep RiskLevel:: >= 3.
ADVERSARIAL: default MEDIUM for PLANNING->TESTING_FIRST (not LOW); config overrides respected.
ACCEPTANCE: every transition has RiskLevel; LOW auto-advances; MEDIUM/HIGH pause; overrides respected.

EXIT_CRITERIA: traverses all 9 states; typed errors on invalid transitions; 7 evidence gates enforce; all 4 INTERACTIVE decisions work; risk gating correct; phase-aware tool filter blocks disallowed tools; ReplanRequest handled; all tests pass; no Phase 0-2 regression.

---

## PHASE 3B: SSB (SHARED STATE BOARD)
Prerequisites: Phase 3A complete. Inter-agent communication via SQLite WAL with single-writer guarantee. Append-only within session. Ssb bound to single (project_id, session_id) pair.

DELIVERABLES: 3
- D1 SSB core (schema + single-writer + WAL config + read pool + migration)
- D2 Conflict detection + append-only enforcement + event notification
- D3 Typed action validation + trace records

SCHEMA:
```sql
PRAGMA user_version = 1;
CREATE TABLE ssb (
  seq INTEGER PRIMARY KEY AUTOINCREMENT,
  project_id TEXT NOT NULL,
  session_id TEXT NOT NULL,
  agent_id TEXT NOT NULL,
  superstep INTEGER NOT NULL DEFAULT 0,
  action_type TEXT NOT NULL CHECK(action_type IN (
    'file_write','file_read','tool_result','test_outcome','finding',
    'claim','checkpoint','model_swap','phase_transition',
    'knowledge_degradation','isolation_metric','plan_update',
    'replan_request','stub_reference','reasoning_trace',
    'phase_complete','memory_monitoring')),
  payload BLOB NOT NULL,
  created_at INTEGER NOT NULL
);
CREATE INDEX idx_ssb_project ON ssb(project_id);
CREATE INDEX idx_ssb_session ON ssb(session_id);
CREATE INDEX idx_ssb_superstep ON ssb(superstep);
CREATE INDEX idx_ssb_conflict ON ssb(project_id, superstep);
CREATE TABLE ssb_files (seq INTEGER NOT NULL REFERENCES ssb(seq), file_path TEXT NOT NULL);
CREATE INDEX idx_ssb_files_path ON ssb_files(file_path);
CREATE INDEX idx_ssb_files_seq ON ssb_files(seq);
```

17 ACTION_TYPES (Rust enum SsbActionType, snake_case): file_write, file_read, tool_result, test_outcome, finding, claim, checkpoint, model_swap, phase_transition, knowledge_degradation, isolation_metric, plan_update, replan_request, stub_reference, reasoning_trace, phase_complete, memory_monitoring.

CONFLICT DETECTION QUERY (requires JOIN):
```sql
SELECT DISTINCT ssb.agent_id
FROM ssb_files JOIN ssb ON ssb_files.seq = ssb.seq
WHERE ssb_files.file_path IN (rarray(?1))
  AND ssb.project_id = ?2 AND ssb.session_id != ?3 AND ssb.superstep = ?4
```

WAL PRAGMAS (order matters): journal_mode=WAL; synchronous=NORMAL; wal_autocheckpoint=1000; busy_timeout=5000; cache_size=-64000; foreign_keys=ON (per-connection, via SsbPragmaApplier); temp_store=MEMORY.

### 3B.1 SKELETON + WAL CONFIGURATION + MIGRATION
PRE_GATE: Phase 3A tests; rusqlite and r2d2_sqlite on crates.io.
TASK:
- Add rusqlite (bundled + array features) and r2d2_sqlite
- orchestrator/src/ssb.rs: SsbActionType (17 variants), SsbEntry (payload: Arc<[u8]>), SsbFileEntry, SsbConfig, SsbPragmaApplier (r2d2::CustomizeConnection), Ssb::new(config, clock: Arc<dyn ClockProvider>) with user_version migration, WAL pragmas, read pool (idle_timeout=None, max_lifetime=None, pool_size=max(num_cpus*2,4)), tokio::mpsc::unbounded_channel for writes, tokio::broadcast capacity=1024 for events.
- SsbWriter handle (Sender clone); SsbReader trait: query_by_session/file/type/superstep; query_conflict; last_seq; subscribe() -> broadcast::Receiver<SsbEntry>; shutdown() with drain.
DETERMINISTIC_CHECK: cargo check; grep SsbActionType, reasoning_trace, journal_mode.*WAL, idle_timeout, CustomizeConnection, user_version, Arc<[u8]>, Arc<dyn ClockProvider>.
ADVERSARIAL: missing bundled feature fails Windows; missing array feature fails rarray; wrong pragma order breaks WAL; mid-run migration with active readers gets SQLITE_LOCKED.
ACCEPTANCE: cargo check 0; 17 variants; WAL pragmas asserted; CustomizeConnection; no idle timeout; user_version set; payload Arc<[u8]>.

### 3B.2 SINGLE-WRITER CHANNEL + WRITE PATH
PRE_GATE: 3B.1 compiles.
TASK:
- Write loop: tokio::spawn receives SsbWriteRequest {action_type, agent_id, superstep, payload, files}. Session/project_id injected at construction. created_at via ClockProvider.
- Write path: BEGIN IMMEDIATE; INSERT ssb; for each file INSERT ssb_files; COMMIT; broadcast AFTER commit. Return seq via oneshot.
- IPC sync (WARN-6): SSB write completes before ToolResult dispatch. Shutdown: drop Sender, drain, subsequent write() returns Err(SsbShutdown). Idempotent.
DETERMINISTIC_CHECK: test_single_writer; test_write_after_shutdown; 10 concurrent sequential seqs.
ADVERSARIAL: no BEGIN IMMEDIATE -> SQLITE_BUSY; bounded channel blocks; SystemTime breaks replay; broadcast before commit shows phantoms; write loop panic hangs oneshots.
ACCEPTANCE: 10K writes no error; seqs monotonic; ClockProvider used; broadcast after commit; write-after-shutdown Err; no hang.

### 3B.3 READ POOL + CONFLICT DETECTION + APPEND-ONLY
PRE_GATE: 3B.2 test passes.
TASK:
- SsbReader impl via r2d2 read pool. No BEGIN/COMMIT on reads (WAL snapshot).
- Conflict query uses JOIN + project_id filter + rarray() for IN clause.
- Append-only: no UPDATE/DELETE on SsbWriter (compile-time); runtime update_hook panics on UPDATE/DELETE in debug builds.
DETERMINISTIC_CHECK: test_read_pool; test_conflict_detection; test_conflict_cross_project; test_append_only; grep rarray; grep "JOIN ssb ON".
ADVERSARIAL: small pool_size queues reads; no project_id filter = cross-project false positive; dynamic IN (?1,...) defeats caching; no JOIN = OperationalError no such column: agent_id.
ACCEPTANCE: concurrent reads don't block writer; correct conflict detection; project_id isolation; rarray used; UPDATE/DELETE panics in debug.

### 3B.4 TYPED ACTION VALIDATION + TRACE RECORDS
PRE_GATE: 3B.3 all tests pass.
TASK: 17 typed payload structs:
- FileWritePayload {path, size_bytes, hash}
- FileReadPayload {path}
- ToolResultPayload {tool_id, success, output_summary}
- TestOutcomePayload {test_name, passed, duration_ms}
- FindingPayload {severity, category, message, file_path, line}
- ClaimPayload {claim_text, grounding_source, verified}
- CheckpointPayload {agent_state_hash, grc_history_hash}
- ModelSwapPayload {from_model, to_model, reason}
- PhaseTransitionPayload {from_state, to_state, evidence_summary}
- KnowledgeDegradationPayload {stage, from_mode, to_mode}
- IsolationMetricPayload {metric_name, value}
- PlanUpdatePayload {plan_hash, phase, step}
- ReplanRequestPayload {reason, requesting_agent}
- StubReferencePayload {original_seq, stub_reason}
- ReasoningTracePayload {step_type, content, topic, grounding, conclusion}
- PhaseCompletePayload {phase_id, agent_id, fsm_state, evidence_artifacts, commit_hashes, test_outcomes} — atomic durability marker at phase boundary, written BEFORE model unload, fsynced. Crash before = re-execute; crash after = skip.
- MemoryMonitoringPayload {metric_name, value, threshold, action_taken}

Validation: write takes typed SsbPayload enum (not raw bytes). SsbTraceable trait defined in orchestrator/ (not reasoning or tools — no circular deps). Trace writes initiated by orchestrator after IPC/GRC callbacks. Serialization: msgpack via rmp-serde.
DETERMINISTIC_CHECK: test_typed_payloads; test_payload_mismatch; test_trace_integration; grep SsbPayload, ReasoningTracePayload, rmp_serde.
ADVERSARIAL: raw bytes insertion compile error; >1MB rejected; field drift caught via proptest round-trip; SsbTraceable in wrong crate = compile fail.
ACCEPTANCE: 17 payloads typed; mismatch returns Err; ReasoningTracePayload round-trips; msgpack confirmed; SsbTraceable in orchestrator; traces initiated by orchestrator.

EXIT_CRITERIA: typed validated payloads for all 17 action_types; conflict via JOIN + project_id; compile+runtime append-only; Arc<[u8]> broadcast; ClockProvider entropy quarantine; user_version migration; graceful shutdown with drain; all tests (unit, property, adversarial, determinism) pass.

---

## PHASE 3C: AGENT SYSTEM
Prerequisites: Phase 3B complete. Agent spawning, lifecycle, checkpointing, crash recovery, supervision, anti-spiral. In-process Tier A isolation only (Tier C deferred to 4A).

DELIVERABLES: 6
- 3C.1 AgentPool trait + AgentHandle + AgentStatus
- 3C.2 Agent lifecycle with death classification
- 3C.3 Checkpointing (msgpack snapshots)
- 3C.4 Recovery protocol
- 3C.5 Supervision tree
- 3C.6 Anti-spiral detection

CLAW-CODE MAPPING:
- WorkerStatus -> AgentStatus (+Checkpointing, +BudgetExceeded)
- WorkerRegistry -> AgentPool (HashMap<Uuid, AgentHandle>, UUID via EntropySource not raw uuid)
- WorkerEvent/EventKind -> AgentEvent
- WorkerFailure -> AgentFailure
- now_secs() REPLACED with ClockProvider::now_monotonic()
- FailureScenario -> death classification + recovery policy
- RecoveryRecipe -> SupervisionPolicy.on_failure()

### 3C.1 AGENTPOOL TRAIT + AGENTHANDLE + AGENTSTATUS
PRE_GATE: 3B SSB tests; 2C per-agent GRC config tests; worker_boot.rs compiles.
TASK:
- AgentStatus: Spawning, Ready, Running, Checkpointing, Finished, Failed
- AgentHandle {agent_id: Uuid, role: AgentRole, status: AgentStatus, active_files: Vec<PathBuf>, grc_config: AgentGrcConfig, scratch_dir: PathBuf}
- AgentRole enum: Executor, Auditor, PromptEngineer, ContextCompiler, MemoryWriter, Researcher, MediaValidator, Designer (8 variants). Designer has distinct AgentGrcConfig from Executor.
- AgentPool trait: spawn(role, task) -> AgentHandle, spawn_parallel(tasks) -> Vec<AgentHandle> (STUB: sequential, parallel in 3D), communicate(from, to, message) -> () (STUB: direct call, IPC in 4A).
- VRAM check via ModelPool::available_vram() (not direct nvml). Insufficient -> ResourceError.
DETERMINISTIC_CHECK: cargo test agent_pool; grep AgentPool >= 3; AgentRole >= 8; spawn_parallel stub not unimplemented!().
ADVERSARIAL: missing active_files breaks 3E; no VRAM check = OOM; default GRC config wrong for role.
ACCEPTANCE: spawn returns valid AgentHandle; spawn_parallel sequential; ResourceError on insufficient VRAM; 8 variants; Designer distinct from Executor.

### 3C.2 AGENT LIFECYCLE WITH DEATH CLASSIFICATION
PRE_GATE: 3C.1 passes; SSB CHECK includes reasoning_trace + isolation_metric (add migration if not).
TASK:
- Lifecycle: Spawning -> Ready -> Running -> Finished|Failed. In-process tokio task.
- AgentDeathClassification: CLEAN_EXIT, CRASH (panic/Err), TIMEOUT (heartbeat miss 15s), BUDGET_EXCEEDED (inject convergence).
- Heartbeat: tokio::sync::watch every 5s, 15s timeout.
- Tier A invisible walls: per-agent allowed_paths: Vec<GlobPattern>. Hidden paths return "not found" (not "denied"). Default hidden: .elai_cc/, rules.db, harness source, other agent checkpoints, SSB database, ONNX model files, elai_cc binary internals.
- ReplanRequest: orchestrator.request_replan(evidence, severity). Max 1 per agent per phase. 2nd call -> ReplanDenied. SSB action_type='replan_request'. MINOR: auditor validates, fast-track. MAJOR: human approval (INTERACTIVE gate).
- ReasoningTraceEntry emission: fields agent_id, step_index (monotonic), step_type, content_summary (max 200 chars), grounding_source, grounding_level, confidence, elapsed_ms, topic, budget_remaining.
DETERMINISTIC_CHECK: test death classifications for all 4; grep "not found" > 0 without "denied"; SSB replan_request; 2nd replan in phase = ReplanDenied.
ADVERSARIAL: short heartbeat = false timeout; wrong error message distinguishes hidden from nonexistent (R31); replan counter not per-phase = abuse; missing ReasoningTraceEntry blinds debugging.
ACCEPTANCE: 4 classifications correct; heartbeat fires at 15s; hidden == nonexistent error; trace entries in SSB; max 1 replan per phase.

### 3C.3 CHECKPOINTING (MSGPACK SNAPSHOTS)
PRE_GATE: 3C.2 passes; SSB operational; rmp-serde available.
TASK:
- Location: `.elai_cc/sessions/{sid}/agents/{agent_id}/`. 5 files:
  - state.msgpack (AgentStatus + GRC state + active_files)
  - grc_history.msgpack (GRC reasoning history)
  - tool_results.msgpack (tool call results)
  - output_draft.msgpack (partial AgentResponse)
  - checkpoint_ts (plain text timestamp, via ClockProvider::now_monotonic())
- Triggers: after every GRC think() step; after every tool call result; after every file edit; before slow ops (>1s).
- Atomic write: temp + fsync + rename. NTFS-safe (same-directory, delete old first).
- SSB: action_type='checkpoint', payload=metadata only (not data).
- UntrackedValue channel rejection: checkpoint serializer validates against channel registry; any UntrackedValue reference -> `CheckpointError::UntrackedValueReference`. Contract: agents MUST NOT use UntrackedValue for recoverable state.
DETERMINISTIC_CHECK: test checkpoint files exist; valid msgpack; checkpoint_ts monotonic; UntrackedValue reference rejected; LastValue/Topic succeed.
ADVERSARIAL: non-atomic write = corrupt; non-monotonic ts = stale load; unbounded tool_results = slow (>100ms at 100 calls); rolling window of N=50.
ACCEPTANCE: checkpoint after every GRC step + tool call; 5 files present (4 msgpack deserializable); atomic writes; SSB checkpoint entry; UntrackedValue rejected; LastValue/Topic accepted.

### 3C.4 RECOVERY PROTOCOL
PRE_GATE: 3C.3 passes; valid checkpoint exists.
TASK:
- Detection: JoinHandle Err = CRASH; heartbeat watch timeout = TIMEOUT. Classify + log SSB + initiate recovery.
- Recovery: read latest checkpoint; spawn new agent (same role + agent_id, new task); inject checkpoint as structured context; continue from last GRC step.
- FailureScenario mapping: TrustPromptUnresolved N/A; PromptMisdelivery CRASH; StaleBranch CRASH; CompileRedCrossCrate CRASH; McpHandshakeFailure N/A; PartialPluginStartup N/A; ProviderFailure CRASH.
- Preserve agent_id. After recovery, first GRC step has grounding_source="CHECKPOINT:{path}".
DETERMINISTIC_CHECK: panic -> same agent_id recovery; 3 GRC steps then crash -> recovered has 3 prior; recovery completes task; SSB death + recovery events.
ADVERSARIAL: corrupt checkpoint -> fresh start with WARNING; agent_id must not change; recovered resumes NEXT step (not replay).
ACCEPTANCE: CRASH recovery <2s; TIMEOUT recovery <heartbeat_timeout+2s; graceful corrupt degradation; monotonic step_index.

### 3C.5 SUPERVISION TREE
PRE_GATE: 3C.4 passes.
TASK: Erlang-derived:
- SupervisionPolicy {max_restarts: u32 (default 3), restart_window: Duration (= phase duration), on_max_restarts: EscalationAction}
- EscalationAction: Escalate | SubstituteModel(ModelId)
- Phase 3 INTERACTIVE: Escalate. Phase 4G AUTONOMOUS: SubstituteModel (gated in Phase 3).
- restart_count per agent per phase, reset at phase boundary.
- Integration: CLEAN_EXIT no restart; CRASH/TIMEOUT increment+restart; BUDGET_EXCEEDED inject convergence (no restart).
- On exhaustion: stop restarting, checkpoint, execute on_max_restarts, log supervision_exhausted in SSB.
DETERMINISTIC_CHECK: 3 crashes then 4th escalates; phase boundary resets count; max_restarts=1 -> 2nd crash escalates; BUDGET_EXCEEDED no increment; SubstituteModel in INTERACTIVE errors.
ADVERSARIAL: restart_count not reset across phases = locked out; BUDGET_EXCEEDED triggers restart = infinite loop; max_restarts<1 clamped to 1 with WARNING.
ACCEPTANCE: restart limits enforced per phase; reset at boundary; Escalate in INTERACTIVE; BUDGET_EXCEEDED no restart.

### 3C.6 ANTI-SPIRAL DETECTION
PRE_GATE: 3C.5 passes; GRC step tracking available.
TASK:
- Track last N GRC steps (step_type + topic + content_hash). same_step_count > 3 = SPIRAL.
- content_hash = blake2s of normalized content (lowercase, whitespace-collapsed).
- On detection: checkpoint; inject escalation signal (GRC step_type="status", content="Anti-spiral..."); if another identical step after signal -> permanent stop (not restart); SSB spiral_detected event with 3 repeated steps.
- Integrates with supervision: on_spiral same as on_max_restarts (Escalate in INTERACTIVE).
- INTERACTIVE pause: "Agent {role} spiraling on {topic}: repeated {step_type} 3x. Options: [retry with hint] [substitute model] [skip step] [abort task]".
- Pre-spiral checkpoint tagged as recovery_target (step N-3, not latest).
DETERMINISTIC_CHECK: 3 identical -> spiral SSB entry; 3 different -> none; spiral+signal+identical -> stopped; spiral+signal+different -> continues; whitespace-only diff -> same hash.
ADVERSARIAL: exact string match evadable; threshold 2 flags legit retries; recovery loads pre-spiral checkpoint.
ACCEPTANCE: 3x same triggers; escalation before stop; INTERACTIVE shows evidence; checkpoint preserved.

DEFERRED: Tier B sandbox (Firecracker/gVisor) to Phase 4A; cognitive isolation_metric aggregation to 3A/3D.

EXIT_CRITERIA: agents spawn with correct role+GRC config; checkpoint reliably; recover preserving SSB trace; supervision limits restarts + Escalate; anti-spiral detects + stops with evidence. No lazy stubs.

---

## PHASE 3D: BSP SUPERSTEPS + CHANNELS
Prerequisites: Phase 3B, 3C complete. Deterministic macro-orchestration via BSP (Valiant 1990). 4 channel types. BinaryOp dropped (non-deterministic with concurrent writes).

Determinism: all combiners total-ordered via SSB seq (LastValue) or append-only deterministic (Topic). Applies in simulation mode (mocked agents + fixed seed); real LLM straggler timeouts may vary participants.

SPEC AMENDMENTS: (1) Topic(accumulate=false) behavior; (2) action_type='memory_monitoring' added to canonical list; (3) ChannelRegistry trait definition; (4) save_checkpoint placement after evidence_gate_passes.

### 3D.1 CHANNEL TYPE DEFINITIONS
PRE_GATE: 3B + 3C contract tests; orchestrator compiles; spec amendments 1 + 3 applied.
TASK: Define in orchestrator/src/bsp/channels.rs:
- **LastValue<T>**: single value, last writer wins by SSB seq. Read Option<T>.
- **Topic<T>(accumulate: bool)**: accumulate=true append-only Vec<T>; accumulate=false Vec<T> cleared at superstep boundary via two-phase: (1) snapshot for prepare_next_tasks, (2) clear before execute_tasks writes. Ordering by SSB seq within superstep.
- **EphemeralValue<T>**: fire-and-forget, cleared to MISSING on zero-write superstep. Per-channel u32 write_count (saturating_add), reset at superstep start, checked at barrier. Clearing logged in SSB action_type='checkpoint' subtype 'ephemeral_clear' for replay.
- **UntrackedValue<T: DeepSizeOf>**: in-memory only, never serialized. guard=false allows multi-writer. Lost on crash. T: DeepSizeOf required (not size_of_val which gives stack-only 24 bytes for Vec). Use deepsize crate.
- ChannelId<T> typed key; ChannelState enum for checkpoint (3 variants: LastValue, Topic, Ephemeral — no UntrackedValue).
- ChannelRegistry trait: read(id) -> ChannelValue<T>; write(id, value) -> Result<(), ChannelError>; snapshot_for_scheduling() -> ChannelSnapshot (frozen); clear_ephemeral() -> count; force_clear_untracked(id) -> bool; deep_size_untracked(id) -> usize.
- ChannelWrite, ChannelValue, ChannelError, ChannelSnapshot types [implementation-defined, spec amendment 3 required before 3E].
DETERMINISTIC_CHECK: all 4 constructors + round-trip; UntrackedValue not in serde; ChannelState has 3 variants; 0 size_of_val hits.
ADVERSARIAL: non-seq ordering = non-det; wrong clearing = stale data; missing DeepSizeOf = 24 bytes for 15MB vec.
ACCEPTANCE: 4 types compile; UntrackedValue never in ChannelState; ChannelId<T> enforced at compile time; ChannelRegistry contract test for 3E+3F.

### 3D.2 BSP SUPERSTEP LOOP
PRE_GATE: 3D.1 met; SSB operational; spec amendment 4 applied.
TASK: In orchestrator/src/bsp/engine.rs. Corrected loop (save_checkpoint moved AFTER evidence_gate_passes):
```
loop {
  channel_snapshot = channels.snapshot_for_scheduling();
  tasks = prepare_next_tasks(checkpoint, channel_snapshot, nodes);
  if tasks.empty() { break; }
  channels.clear_non_accumulating_topics();
  writes = execute_tasks_parallel(tasks, model_pool, ssb);
  apply_writes(checkpoint, channels, writes);
  // deferred phase (3D.3)
  channels.clear_ephemeral();
  if !evidence_gate_passes(checkpoint, channels) {
    attempt += 1;
    if attempt >= max_superstep_attempts { break; }
    continue;
  }
  save_checkpoint(checkpoint, channels);
  superstep += 1;
  attempt = 0;
}
```
- apply_writes orders by SSB seq (deterministic)
- save_checkpoint: msgpack to `.elai_cc/sessions/{sid}/checkpoints/superstep_{N}.msgpack`. Only after gate passes.
- superstep counter: u64, persisted. Monotonic.
- Max attempts per superstep configurable (default 3). max_supersteps total 100.
- Straggler: per-agent timeout (Phase 3C heartbeat 15s). TIMEOUT classified, supervision restarts, remaining proceed.
DETERMINISTIC_CHECK: counter increments; 2 runs with fixed seed = byte-identical checkpoints; no file on failed gate.
ADVERSARIAL: non-seq apply = non-det; always-false gate terminates at max_attempts; 1ms timeout gives TIMEOUT (not CRASH); retry with save-before-gate = collision.
ACCEPTANCE: 3+ supersteps 2 agents; byte-identical determinism; max_attempts terminates; no checkpoint on failed gate.

### 3D.3 DEFERRED NODE EXECUTION
PRE_GATE: 3D.2 met; deferred concept from audit BLOCK-5.
TASK:
```
for node in get_deferred_nodes(nodes) {
  if node.all_inputs_satisfied(channels) {
    deferred_writes = execute_single(node, channels);
    apply_writes(checkpoint, channels, deferred_writes);
  }
}
```
- Marked at graph construction (is_deferred: bool on NodeSpec).
- Deferred writes applied within same superstep (before clear_ephemeral and save_checkpoint). Matches LangGraph defer=True.
- No cascading. Single pass.
- Use cases: fan-in aggregation, consensus voting, post-agent cleanup.
DETERMINISTIC_CHECK: deferred fires after barrier; sees both outputs of 2 non-deferred; unsatisfied input does NOT fire.
ADVERSARIAL: cascading impossible (both inputs unsatisfied); deferred writes visible in checkpoint.
ACCEPTANCE: after all non-deferred; single pass enforced; visible in same superstep checkpoint.

### 3D.4 CHECKPOINT + MEMORY MANAGEMENT INTEGRATION
PRE_GATE: 3D.1-3D.3 met; 3C checkpoint infrastructure; spec amendment 2 applied.
TASK:
- BSP checkpoint (superstep-level): channel state + superstep counter + node status.
- Agent checkpoint (per-agent): GRC history + tool results + output draft.
- Both under `.elai_cc/sessions/{sid}/` in separate subdirs. BSP triggers agent checkpoint at superstep boundary.
- UntrackedValue monitoring: call deep_size_untracked(id) each boundary. Log SSB action_type='memory_monitoring' (NEW — isolation_metric reserved for cognitive token savings). Payload {channel_id, size_bytes, threshold_bytes, action: "measure"|"force_clear"}. If size > 2x limit (default 10MB): force_clear.
- EphemeralValue zero-write: write_count saturating_add, at barrier if 0 -> MISSING. Record in SSB action_type='checkpoint' payload includes ephemeral_clears: Vec<ChannelId>. On replay: count writes per EphemeralValue per superstep; if 0, set MISSING.
- Topic(accumulate=false): two-phase snapshot + clear. Checkpoint contains post-execute state.
- Crash recovery: load checkpoint + replay SSB. For each superstep: apply writes in SSB seq order; EphemeralValue count-based MISSING; Topic(accumulate=false) clear at boundary start. UntrackedValue lost (never in checkpoint/SSB).
DETERMINISTIC_CHECK: save/load identical; UntrackedValue absent; EphemeralValue MISSING on zero-write; Topic cleared; force_clear SSB entry; 0 isolation_metric in bsp/; replay-to-clean identical.
ADVERSARIAL: wrong replay order = divergence; unrecorded clearing = recovery miss; live-ref instead of snapshot = stale scheduling; size_of_val = 24 = no trigger; memory_monitoring vs isolation_metric corruption.
ACCEPTANCE: checkpoint round-trip byte-identical; R19 force-clear at threshold via DeepSizeOf; crash recovery identical; 5+ superstep integration; 0 isolation_metric in bsp/; ChannelRegistry contract test.

### 3D.5 PARALLEL AGENT EXECUTION (spawn_parallel upgrade)
PRE_GATE: 3D.1-3D.4 met; 3C spawn_parallel stub sequential.
TASK:
- Upgrade to true parallel via tokio::JoinSet. Each agent task: send AgentRequest (in-process Phase 3, Tier C Phase 4), await AgentResponse, collect ChannelWrites.
- VRAM-aware: check ModelPool. max_concurrent = floor(available_vram / per_agent_vram_estimate). Fallback max_concurrent=1.
- Barrier: JoinSet::join_all() with per-agent timeout. TIMEOUT classified, removed.
- Wall-time check: 2+ agents < 2x slowest single.
DETERMINISTIC_CHECK: 2 agents concurrent; 2 mocked 100ms total <250ms; max_concurrent=1 sequential; 12GB VRAM 2x8GB = concurrent=1.
ADVERSARIAL: no VRAM check OOMs; no per-agent timeout blocks all; completion-order (not seq-order) is non-det.
ACCEPTANCE: parallel dispatch; VRAM limits; straggler timeout; 3C stub replaced.

EXIT_CRITERIA: deterministic simulation mode (byte-identical checkpoints); 4 channel types correct; deferred exactly once per superstep after non-deferred; UntrackedValue never checkpointed, force-clear at 2x via DeepSizeOf; EphemeralValue zero-write cleared; SSB-replayable clearing; Topic(accumulate=false) two-phase snapshot+clear; straggler timeout; crash recovery identical; spawn_parallel concurrent + VRAM-aware; no isolation_metric for memory (memory_monitoring only); 3B+3C no regression; ChannelRegistry contract.

---

## PHASE 3E: CONTEXT BUDGET ALLOCATOR
Prerequisites: Phase 3D. Depends on providers::token_count, token_count_estimate, SSB, BSP. Fixed-slot distribution ensuring model context never exceeded. Adaptive reduction with floors. Cache breakpoints.

### TYPES

```rust
enum ContextSlot {
  SystemPrompt,   // slot 1: GRC structured prompt per role
  Rules,          // slot 2: active Tier 3 rules for phase + file context
  PhaseSpec,      // slot 3: from PipelineSpec
  RepoMap,        // slot 4: PageRank DKG output
  CodeContext,    // slot 5: BM25 + vector chunks, cross-encoder reranked
  SsbEntries,     // slot 6: filtered by file overlap score
  GrcHistory,     // slot 7: prior reasoning for current topic
  OutputReserve,  // slot 8: minimum 2000 tokens
}

struct SlotAllocation {
  slot: ContextSlot,
  allocated_tokens: usize,
  floor_tokens: usize,      // 0 for slots 1-3
  actual_tokens: usize,
  source_truncated: bool,
}

struct ContextBudget {
  model_max_context: usize, // from ModelCapabilityRegistry
  slots: [SlotAllocation; 8],
  cache_breakpoints: [CacheBreakpoint; 2],
  total_allocated: usize,
  total_used: usize,
  adaptive_level: AdaptiveLevel,
}

struct CacheBreakpoint {
  after_slot: ContextSlot,  // BP1 after PhaseSpec, BP2 after CodeContext
  prefix_hash: u64,         // deterministic (FxHash/xxhash — NOT DefaultHasher)
  valid: bool,
}

enum AdaptiveLevel {
  Normal, FilteredByRelevance, ReducedGranularity, ReducedScope, Terminal,
}
```

### TOKEN STRATEGY
- Allocation: token_count_estimate (sampling O(lines/N)).
- Final validation: token_count (exact).
- Rationale: 8 per-slot calls. Estimate 95%+ accurate within 15% for files >100 lines; rare 5% tail may exceed.
- If exact > max: adaptive reduction.
- Latency target: <50ms (architecture-overview §8).

### SLOT ORDERING
Slot 1 SystemPrompt, Slot 2 Rules, Slot 3 PhaseSpec, [BP1: slots 1-3 stable within phase], Slot 4 RepoMap, Slot 5 CodeContext, [BP2: slots 4-5 stable within same file set], Slot 6 SsbEntries, Slot 7 GrcHistory, Slot 8 OutputReserve.
- BP1 invalidated by: phase transition, role change, rule set change.
- BP2 invalidated by: file set change.
- Slots 6-8 recomputed every invocation.

### BUDGET DISTRIBUTION
Fixed slots 1-3: exact, subtracted first. Remaining dynamic to slots 4-7:
- **RepoMap 15%**
- **CodeContext 40%**
- **SsbEntries 25%**
- **GrcHistory 20%**
OutputReserve subtracted before split (min 2000 tokens).

FLOORS: SsbEntries >= 200; GrcHistory >= 500; OutputReserve >= 2000.
If dynamic_budget < sum(floors)=700: Err(BudgetInfeasible).
Steal order: from CodeContext first, then RepoMap.

### SSB FILTERING
file_overlap_score = |intersection(entry.files, agent.files)| / |agent.files|.
Score 0.0 excluded. Sort by (score DESC, seq DESC). Fitted via token_count_estimate.

### ADAPTIVE REDUCTION (3 attempts, logged SSB action_type='context_reduction'):
- Attempt 1 FilteredByRelevance: raise SSB overlap threshold (0.3 configurable); GRC keep current topic only.
- Attempt 2 ReducedGranularity: RepoMap higher PageRank density guard; CodeContext function signatures + doc comments only.
- Attempt 3 ReducedScope: RepoMap current file + direct deps; CodeContext current file only; SSB entries current agent only.
- Terminal: INTERACTIVE -> ESCALATE with breakdown; AUTONOMOUS -> SPLIT_TASK (Phase 4G).

### PROMPT ASSEMBLY MAPPING
- Slot 1 -> [role_prompt] ~50-100 tokens
- Slot 2 -> [tier_3_rules] ~100-300 tokens
- Slots 4+5 -> [code_context] ~2000-8000 tokens
- Slot 3 -> [task_spec + output_format + constraints] ~250-600 tokens
- Slots 6-7 not in base prompt (structured injection)
- Slot 8 reserved for model output

### SESSION HANDOFF (from §9 L269-277):
Trimming preserves BP1 alignment (slots 1-3). Stubs replace >500-token tool results, slot content intact. BP2 recomputed on new session (not assumed).

### STEPS

#### 3E.1 TYPES AND BUDGET STRUCT
PRE_GATE: 3D tests; prepare_next_tasks exists; providers token_count callable.
TASK: Define enums + structs. ContextBudget::new(model_max_context) zero allocation. Display impl.
DETERMINISTIC_CHECK: compile + test; ContextSlot has 8; new(131072).total_allocated==0.
ADVERSARIAL: wrong ordering breaks BP; ignoring floors gives zero SSB/GRC.
ACCEPTANCE: compile + unit tests pass.

#### 3E.2 BUDGET COMPUTATION
PRE_GATE: 3E.1 types.
TASK: ContextBudget::compute(model_max_context, fixed_slot_sizes, output_reserve). OutputReserve = max(output_reserve, 2000). Rounding remainder assigned to largest slot (CodeContext).
DETERMINISTIC_CHECK: compute(8192,[100,200,300],2000): dynamic=5592; RepoMap=838, CodeContext=2238, SsbEntries=1398, GrcHistory=1118; sum=5592. sum == model_max_context.
ADVERSARIAL: rounding overflow; output_reserve<2000 accepted.
ACCEPTANCE: deterministic; sum==model_max_context for 100 configurations.

#### 3E.3 FLOOR ENFORCEMENT
PRE_GATE: 3E.2.
TASK: After split enforce: SsbEntries>=200, GrcHistory>=500, OutputReserve>=2000. Steal CodeContext first, then RepoMap. If insufficient: Err(BudgetInfeasible). If dynamic<700: Err.
DETERMINISTIC_CHECK: compute(4000,...,2000) -> GrcHistory 500 steal; compute(3000,...,2000) Err; compute(100000,...) no steal.
ADVERSARIAL: CodeContext<200 warn not error (small models); Err not swallowed.
ACCEPTANCE: floors never violated; BudgetInfeasible when impossible.

#### 3E.4 CACHE BREAKPOINT TRACKING
PRE_GATE: 3E.2; provider prefix caching (prompt_cache.rs).
TASK: Track 2 BPs after slot 3 and slot 5. prefix_hash = hash(concatenated slot content). Use FxHash/xxhash (deterministic). Report valid to provider.
DETERMINISTIC_CHECK: same slots 1-3 -> BP1.valid=true; slot 2 change -> BP1.valid=false; slot 6 change -> BP2.valid=true.
ADVERSARIAL: DefaultHasher non-det; missing provider report = never-hit cache.
ACCEPTANCE: deterministic; same content = same hash across 1000 invocations.

#### 3E.5 SSB FILTERING BY FILE OVERLAP
PRE_GATE: 3E.1; SSB operational; ssb_files table.
TASK: Build entry.files from ssb_files rows (GROUP BY seq). file_overlap_score formula. Filter score>0.0, sort (score DESC, seq DESC). Fit to allocation.
DETERMINISTIC_CHECK: overlap({a,b,c},{b,c,d})=2/3; overlap({x},{a,b,c})=0; overlap({a,b},{a,b})=1; empty agent_files=0.
ADVERSARIAL: guard empty agent_files; 10K entries filter <10ms.
ACCEPTANCE: matches L125 formula; no div/0.

#### 3E.6 ADAPTIVE REDUCTION
PRE_GATE: 3E.2+3E.3+3E.5; ESCALATE in FSM; SPLIT_TASK returns Err (4G implements receiver).
TASK: 3 levels. Each logged SSB with level, before_tokens, after_tokens, slots_affected. Terminal branches by mode.
DETERMINISTIC_CHECK: force overflow 2000 vs 3000; verify logs; Terminal INTERACTIVE=Escalate(breakdown), AUTONOMOUS=SplitTask.
ADVERSARIAL: level 2 signatures overflow -> level 3; SSB log doesn't affect current invocation (next visibility).
ACCEPTANCE: <=3 attempts; every reduction logged; terminal matches mode.

#### 3E.7 INTEGRATION TEST
PRE_GATE: 3E.1-3E.6.
TASK: Realistic scenario; ModelCapabilityRegistry (not hardcode); test small 4096 + large 131072; <50ms latency (ClockProvider); 100x identical output (determinism).
DETERMINISTIC_CHECK: budget computed; all floors; valid BP hashes; <50ms; 100 runs byte-identical.
ADVERSARIAL: estimate drift 20% triggers adaptive; no Instant::now in module (entropy quarantine).
ACCEPTANCE: <50ms; deterministic; no raw time access.

EXIT_CRITERIA: sum(slots)<=model.max_context; output_reserve>=2000; floors never violated; cache breakpoints work; adaptive terminates + logs + no silent truncation; <50ms; deterministic; overlap formula exact.

---

## PHASE 3F: RULES ENGINE (self-improving, three-tier)
Prerequisites: Phase 3D + implicit Phase 1A (tree-sitter in knowledge/). Deterministic rule enforcement. Predicate DSL with 5 types. Self-discovery from auditor violations. Rules are DATA in SQLite evaluated by Rust interpreter. 3 tiers: T1 deterministic code, T2 constrained decoding, T3 prompt + auditor backup.

ARCHITECTURE: tree-sitter via knowledge::AstEvaluator trait (rules/ takes Box<dyn AstEvaluator>, no separate dep).

PREDICATE EVALUATION ORDER (cheapest first, short-circuit on BLOCK):
1. METRIC_GATE O(1)
2. GREP_PATTERN O(n)
3. AST_PATTERN (tree-sitter parse)
4. COMPOSITE (recursive)
Within same type: sort severity DESC (BLOCK before WARN). Short-circuit on first BLOCK. WARN accumulated before BLOCK preserved as context.

NEW DEPENDENCIES: proptest (dev), criterion (dev), jsonschema (runtime), tree-sitter (runtime via knowledge/).

### 3F.1 SQLITE SCHEMA
PRE_GATE: 3D contract tests; rusqlite in workspace.
TASK: Create rules/ crate tables: rules, rule_metrics, contract_tests, violation_patterns. Columns: rule_id, domain, tier, status, predicate, phase_mask, severity, mutable, source, etc. violation_patterns scoring: score_clarity, score_frequency, score_precision. rule_metrics counterfactual: causal_catches, spurious_catches. schema_version table for migration.
DETERMINISTIC_CHECK: schema tests; sqlite3 .tables shows 4; CHECK constraints reject invalid tier/status/source.
ADVERSARIAL: NULL in non-nullable fails; v1->v2 preserves data.
ACCEPTANCE: 4 tables; CHECK rejection; idempotent migration.

### 3F.2 PREDICATE DSL TYPES
PRE_GATE: 3F.1.
TASK: 5 predicate types: **AST_PATTERN, GREP_PATTERN, SCHEMA_REQUIRED, METRIC_GATE, COMPOSITE**. Note: spec lists 5 here but COMPOSITE + GRAPH_REACHABILITY are distinct predicate types (6 predicate types referenced in task prompt — GRAPH_REACHABILITY appears in 3J.WIRE.STUBS as DkgQuerier-backed predicate). Verdict enum: ALLOW, WARN, BLOCK, ESCALATE(EscalationTarget). EscalationTarget: Auditor, Human, NextTierRule(RuleId). Serde JSON or postcard. AstEvaluator trait: evaluate_query(lang, source, query) -> Result<Vec<AstMatch>>; validate_query(lang, query) -> Result<(), AstError>.
DETERMINISTIC_CHECK: 5 predicate types round-trip; 4 Verdicts; 3 EscalationTargets.
ADVERSARIAL: circular COMPOSITE errors within 1ms; serde version error on field change.
ACCEPTANCE: 5 types; enums complete; round-trip passes.

### 3F.3 RULEENGINE TRAIT
PRE_GATE: 3F.2.
TASK: trait: evaluate(action, context) -> ALLOW|BLOCK|WARN; get_active_rules(fsm_state, file_context) -> Vec<Rule>; compile_predicate(raw) -> Result<CompiledPredicate, PredicateError> (validates regex via Regex::new, tree-sitter via AstEvaluator::validate_query, JSON schemas via jsonschema crate at INSERT, not eval time). Order: METRIC -> GREP -> AST -> COMPOSITE. Short-circuit on BLOCK.
DETERMINISTIC_CHECK: object-safe; compile_predicate rejects invalid regex; bad $ref.
ADVERSARIAL: accept-all corrupts DB; wrong order runs AST before GREP.
ACCEPTANCE: object-safe; compile_predicate rejects malformed; order verified.

### 3F.4 RULE LIFECYCLE STATE MACHINE
PRE_GATE: 3F.3.
TASK: **PROPOSED -> CANDIDATE -> SHADOW -> ACTIVE -> RETIRED**.
- PROPOSED->CANDIDATE: >=3 contract tests AND all pass.
- CANDIDATE->SHADOW: tests pass AND human approves OR independent second-auditor verifies (finding-source auditor CANNOT approve).
- SHADOW->ACTIVE: false_positive_rate<5% AND true_positive_rate>80% AND counterfactual_causal_rate>50% AND no safety regressions.
- ACTIVE->RETIRED: fpr>30% over 10 pipelines OR superseded OR human request.
- mutable=0 reject auto-retire; mutable=0 exempt from causal rate gate (human-promotable only).
- Every transition logged with timestamp, metrics, reason.
DETERMINISTIC_CHECK: 2 tests Err; 3 passing Ok; fpr=6% Err; tpr=79% Err; ccr=49% Err; safety_regression Err; all met Ok; same-auditor approval circular Err; human Ok; mutable=0 auto-retire Err; log entry per transition.
ADVERSARIAL: div/0 (0 causal + 0 spurious) -> insufficient data; RETIRED not deleted; AUDITOR source auto-promote blocked.
ACCEPTANCE: 5 states reachable; guards enforced; mutable=0 invariant; div-by-zero safe.

### 3F.5 PREDICATE DSL INTERPRETER
PRE_GATE: 3F.1-3F.4; AstEvaluator mock.
TASK: Interpret 5 types. AST_PATTERN delegate AstEvaluator. GREP compile once, reuse. SCHEMA_REQUIRED via jsonschema. METRIC_GATE: lt/gt/eq/between. COMPOSITE: recursive AND/OR/NOT, max 8 depth. Verdict {rule_id, result, evidence}. ESCALATE: Auditor (SHADOW), Human (mutable=0 promotion), NextTierRule.
DETERMINISTIC_CHECK: AST detects df[].mean() without shift; GREP bare except:pass; SCHEMA missing required; METRIC LOC>threshold; COMPOSITE correct; ESCALATE correct; 100x identical.
ADVERSARIAL: depth 100 Err(DepthExceeded) not panic; ReDoS completes within timeout (regex crate linear); empty AST match on violating code = BLOCK.
ACCEPTANCE: all 5 correct; ESCALATE routes; depth 8; 100 runs identical.

### 3F.6 PHASE-AWARE AND DOMAIN-AWARE ACTIVATION
PRE_GATE: 3F.5; FSM state enum.
TASK: phase_mask bitmask. Domain from file paths. get_active_rules: status=ACTIVE AND phase_mask match AND (domain match OR domain IN ('engineering', 'security')). Configurable elai_cc.toml [rules.domain_mapping]. Engineering + security cross-cutting (always active).
DETERMINISTIC_CHECK: phase_mask EXECUTING only; trading rule for src/trading/signal.py; engineering all paths; domain detection case-insensitive.
ADVERSARIAL: case-sensitive miss; phase_mask=0 never active.
ACCEPTANCE: phase filtering matches bitmask; 5 domains + engineering + security; cross-cutting always.

### 3F.7 SELF-DISCOVERY MECHANISM
PRE_GATE: 3F.1-3F.6; Phase 3C (auditor IPC).
TASK: Violation pattern normalization (orchestrator side, NOT auditor). Pattern hashing blake2s of normalized text. Occurrence counting. Auto-proposal at occurrences>=3 -> generate DSL (tree-sitter query, fallback regex, fallback Tier 3 only). COMPILE-TIME VALIDATION: all via compile_predicate. Invalid logged + discarded. Auto-contract-tests: each example_code -> BLOCK test; orchestrator generates ALLOW counterexamples (not auditor). Trust boundary: PROPOSED from source=AUDITOR require human approval OR independent second-auditor. 3-axis scoring: clarity, frequency, precision.
DETERMINISTIC_CHECK: 3 patterns -> PROPOSED; 2 -> none; valid predicate passes; invalid logged; >=1 BLOCK test; AUDITOR source blocked.
ADVERSARIAL: aggressive normalization collapses distinct; weak gives different hashes; auditor-influenced normalization = circular.
ACCEPTANCE: exactly >=3; validated; trust boundary; scoring [0,15].

### 3F.8 SEED DATA LOADING
PRE_GATE: 3F.1-3F.7.
TASK: rules_seed.db from domain rule files. **Ship ~33 highest-value rules** (engineering + security + top trading + post-tool completeness). status=ACTIVE source=SEED (skip lifecycle). Post-tool completeness GREP rules (Tier 1, engineering, PostToolUse):
- web_fetch with <img> -> WARN "N images not extracted"
- web_fetch with <table> -> WARN "N tables not extracted"
- file_read with TODO/FIXME -> INFO "N TODO/FIXME items"

Merge: global seeds + project. Project overrides by rule_id but mutable=0 cannot be weakened.
DETERMINISTIC_CHECK: loads clean; all ACTIVE/SEED; count>=33; Tier 1 >=3 contract tests; Tier 2 >=2; merge override; mutable=0 not overridable.
ADVERSARIAL: absent seed = explicit error; merge preserves custom rules; mutable=0 override rejected.
ACCEPTANCE: clean load; >=33; contract tests; merge preserves project.

### 3F.9 CONTRACT TEST SUITE
PRE_GATE: 3F.1-3F.8; proptest dev-dep.
TASK: Static: min 3 per Tier 1 (BLOCK + ALLOW + EDGE); min 2 per Tier 2. proptest: violation_generator, valid_generator, mutation_invariant, fix_mutation. COMPOSITE prop_recursive(depth 8, size 64). De Morgan: NOT(A AND B) == NOT A OR NOT B. Associativity. Shrunk counter-examples inserted as source=GENERATED. **Total static contract tests: >=254**; **proptest: ~1400 lines across 7 domains**.
DETERMINISTIC_CHECK: cargo test contract + proptest; count>=254; 256 cases/rule; De Morgan holds.
ADVERSARIAL: counter-example reproducible + inserted; READ-LOCK during self-mod enforced.
ACCEPTANCE: >=254; 256 cases; invariants; lock enforced.

### 3F.10 BENCHMARK AND MEASURE
PRE_GATE: 3F.1-3F.9; criterion dev-dep.
TASK: Benchmark single GREP/AST/METRIC + 10 mixed on 100-line. Targets: GREP/METRIC <1ms, AST <5ms.
DETERMINISTIC_CHECK: GREP median<1ms; METRIC<0.1ms; 10-rule<10ms.
ADVERSARIAL: 100-line + 1000-line scaling.
ACCEPTANCE: benchmarks pass; results in .elai_cc/metrics/phase_3F.toml.

COUNTERFACTUAL SCOPE NOTE: defined + data-modeled in 3F (schema cols, rate computation, transition guards). Runtime integration depends on orchestrator+auditor (3A+3C). Phase 3I DST includes counterfactual in simulation.

EXIT_CRITERIA: deterministic (100x); lifecycle with guards and mutable=0; self-discovery at >=3; >=254 contract tests + proptest 256 cases; benchmarks recorded; trust boundary enforced.

---

## PHASE 3G: HIERARCHICAL PLAN MANAGEMENT
Prerequisites: Phase 3D + 3E (needed for Slot 3 sizing in 3G.4). Depends on SSB ('plan_update'), ContextBudget (Slot 3), token_count(_estimate). Two-artifact: skeleton (2-4k tokens, Slot 3) + per-phase elaboration (on demand). Contract extraction propagates. Coherence enforcement.

### TYPES

```rust
struct PlanSkeleton {
  pipeline_id: PipelineId,
  phases: IndexMap<PhaseId, PhaseEntry>,
  global_constraints: Vec<String>,
  version: u32,
  created_at: u64,  // ClockProvider
  updated_at: u64,
}

struct PhaseEntry {
  phase_id: PhaseId,
  goal: String,           // one-liner, max 120 chars
  inputs: Vec<ContractRef>,
  outputs: Vec<ContractRef>,
  constraints: Vec<String>,
  depends_on: Vec<PhaseId>,
  depended_by: Vec<PhaseId>,
  version: u32,
  output_type: Option<OutputType>,
  visual_check: bool,     // triggers Render-Capture-Evaluate (default false)
}

struct ContractRef {
  name: String,             // e.g. "AgentPool::write(action)"
  type_signature: String,
  invariants: Vec<String>,
}

struct PhaseDetail {
  phase_id: PhaseId,
  skeleton_version: u32,
  contracts: ExtractedContracts,
  implemented_contracts: Vec<ContractRef>,
  declared_interfaces: Vec<ContractRef>,
  content: String,
  token_count: usize,
}

struct ExtractedContracts {
  input_contracts: Vec<ContractRef>,
  output_contracts: Vec<ContractRef>,
  shared_constraints: Vec<String>,
}
```

Storage: skeleton at `.elai_cc/pipelines/{id}/skeleton.toml` (authoritative); phase detail at `.elai_cc/pipelines/{id}/phases/{phase_id}.toml`; SSB audit trail action_type='plan_update' on every write.

TOML example:
```toml
[phase.3B]
goal = "SSB single-writer SQLite WAL"
inputs = ["AgentPool::write(action)","FSM::phase_transition()"]
outputs = ["ssb::query(filter) -> Vec<SsbRow>"]
constraints = ["append-only within session","conflict detection on file_path"]
depends_on = ["3A"]
depended_by = ["3C","3D"]
version = 3
```

### CONTRACT EXTRACTION
3 steps: (1) input_contracts = union(dep.outputs for dep in depends_on); (2) output_contracts = union(dep.inputs for dep in depended_by); (3) shared_constraints = skeleton.global_constraints + phase.constraints. Result prepended to detail.

### COHERENCE ENFORCEMENT (3 checks):
1. Phase detail satisfies all input/output contracts (match by name + type_signature).
2. No new interfaces skeleton does not declare (structured set comparison, NOT free-text).
3. Version incremented if contracts changed.
Violation -> BLOCK finding.

### STUB REFERENCES
On skeleton write, for each phase without PhaseDetail file: emit SSB action_type='stub_reference' {phase_id, skeleton_version}.

### DEPENDENCY PROPAGATION (Kahn's algorithm):
On contract change: BFS from phase via depended_by, flag needs_review in SSB action_type='plan_update' {phase_id, reason: "dependency_contract_changed", source_phase}. Cycle rejected on write.

### 3G.1 TYPES AND SKELETON SCHEMA
PRE_GATE: 3D tests; SSB; ContextBudget; token_count.
TASK: Types + serde TOML. PlanSkeleton::new(pipeline_id) version=0 empty. PhaseEntry::new(id, goal) validates goal<=120 chars.
DETERMINISTIC_CHECK: compile; version==0; empty phases; 121-char goal Err; TOML round-trip.
ADVERSARIAL: self-reference cycle on write; empty type_signature warning.
ACCEPTANCE: compile + TOML + goal validation.

### 3G.2 SKELETON SERIALIZATION AND TOKEN ENFORCEMENT
PRE_GATE: 3G.1; providers token_count.
TASK: write_to_file: estimate first, if >3600 exact check, if >4096 Err(SkeletonBudgetExceeded). SSB plan_update on every write. For each phase without PhaseDetail file: emit stub_reference. Version enforcement: contract changed -> increment both phase.version + skeleton.version.
DETERMINISTIC_CHECK: 20 phases 80-char goals <2000; 50 phases 120-char <4096; 50 + verbose >4096 Err; round-trip identical; SSB plan_update.
ADVERSARIAL: 3600 margin catches estimate drift; TOML whitespace not inflated.
ACCEPTANCE: fits <=50 phases; Err on exceed; SSB on every write.

### 3G.3 CONTRACT EXTRACTION
PRE_GATE: 3G.1 + 3G.2.
TASK: extract_contracts(phase_id) -> Result<ExtractedContracts>. Err on unknown phase.
DETERMINISTIC_CHECK: A->B->C chain; empty deps; nonexistent Err.
ADVERSARIAL: missing depends_on Err; name conflict with diff signatures Err(ContractConflict{name, phase_a, phase_b}).
ACCEPTANCE: exact L234-239 algorithm; missing detected; conflicts detected.

### 3G.4 PHASE ELABORATION LOADING
PRE_GATE: 3G.2+3G.3; Phase 3E Slot 3.
TASK: load_phase_context(skeleton, phase_id) -> PhaseContext. Skeleton + active detail + contracts + adjacent (one hop). If exceeds Slot 3: adaptive reduction (ReducedGranularity: skeleton + contracts, defer detail). PhaseDetail.skeleton_version stores at elaboration for staleness detection.
DETERMINISTIC_CHECK: Slot3=2000, skel=800, detail=1000, contracts=150 fits; 1500 triggers reduction; version mismatch emits warning; adjacent contracts for B gets A.outputs + C.inputs.
ADVERSARIAL: stale skeleton_version forces re-extract; skeleton alone > Slot3 Err(SlotBudgetExceeded).
ACCEPTANCE: fits Slot 3; staleness detection; reduction before failure.

### 3G.5 COHERENCE ENFORCEMENT
PRE_GATE: 3G.3+3G.4; auditor (3C).
TASK: coherence_check module (library, invoked BY auditor via IPC ToolRequest). 3 checks. Findings to SSB action_type='finding'.
DETERMINISTIC_CHECK: superset 0 BLOCKs; missing contract 1 BLOCK; undeclared interface 1 BLOCK; version unchanged 1 BLOCK; all 3 violations 3 BLOCKs.
ADVERSARIAL: empty declared_interfaces Err (must self-declare); version read from file/SSB (not cache).
ACCEPTANCE: 3 checks; BLOCK per violation; zero false positives.

### 3G.6 DEPENDENCY PROPAGATION AND VERSION TRACKING
PRE_GATE: 3G.1 + 3G.5; SSB.
TASK: propagate_changes(changed_phase_id). BFS depended_by. Flag in SSB action_type='plan_update'. Kahn's cycle detection on write. Version: phase.version+=1 on contract change, skeleton.version+=1 on phase version change. **split_phase(phase_id, split_points, preserve_contracts: bool)**: preserve_contracts=true copies input contracts to first sub-phase, output contracts to last, creates internal contracts between adjacent sub-phases. Max split depth 2 (matches 4G SPLIT_TASK).
DETERMINISTIC_CHECK: A->B->C->D flags B,C,D; diamond flags once; isolated no propagation; A->B->C->A cycle Err(CycleDetected); version mono; split preserves correctly.
ADVERSARIAL: visited set prevents redundant; checked_add for u32 overflow.
ACCEPTANCE: transitive chains + diamonds + forests; cycle rejected; version monotonic; SSB records all.

### 3G.7 INTEGRATION TEST: FULL PLAN LIFECYCLE
PRE_GATE: 3G.1-3G.6; SSB + ContextBudget.
TASK: 5 phases A->B->C, A->D, D->E. Realistic content. 100 runs identical.
DETERMINISTIC_CHECK: lifecycle without error; skeleton token count; contract extraction; coherence valid/invalid; propagation; SSB plan_update; 100x deterministic; stub_reference emitted.
ADVERSARIAL: real tokenizer not hardcoded; unique pipeline_id; ClockProvider injected; 0 Instant::now in module.
ACCEPTANCE: full e2e; all SSB verifiable; 100 deterministic runs; token budget enforced with real tokenizer.

EXIT_CRITERIA: skeleton <4096 + skeleton+detail fits Slot 3; contract extraction matches algorithm; propagation via needs_review; coherence catches 3 violation types; BLOCK findings; cycle detection; version monotonic; SSB plan_update; stub_reference on un-elaborated; deterministic with ClockProvider; contract tests in tests/contracts/.

---

## PHASE 3H: SESSION HANDOFF (CMV-derived)
Prerequisites: Phase 3G (PlanSkeleton in snapshots), 3B (SSB for stub recovery), 3E (budget for trimming target), token_count. DAG-based versioning with structurally lossless trimming. Sessions branch, explore, merge. Checkpoint triggers: every GRC step, tool call, phase transition. Handoff: at 85% budget (configurable). Paper: arxiv 2602.22402 (CMV).

SPEC AMENDMENTS REQUIRED:
1. snapshot_context schema: expanded from BLOB-per-snapshot (role TEXT, trimmed_messages BLOB, stub_count INTEGER) to per-message rows (seq, role, content, is_stubbed, stub_ssb_seq).
2. SSB action_type for branch operations: either new 'branch_abandon' OR reuse 'checkpoint' with metadata.
3. SSB cleanup on merge: logical exclusion (query filter) vs physical deletion.

### SQLITE SCHEMA (session_dag.db, one per session, DELETE journal mode)

```sql
CREATE TABLE snapshots (
  snap_id TEXT PRIMARY KEY,                            -- UUID v4 via EntropyProvider
  parent_id TEXT REFERENCES snapshots(snap_id),        -- NULL for root
  session_id TEXT NOT NULL,
  fsm_state TEXT NOT NULL,
  phase TEXT, step TEXT,
  findings BLOB,                                       -- msgpack Vec<AuditFinding>
  code_changes BLOB,                                   -- msgpack Vec<FileChange>
  tests_written BLOB,                                  -- msgpack Vec<TestRef>
  commit_hashes TEXT,                                  -- comma-separated SHAs
  created_at INTEGER NOT NULL,                         -- ClockProvider
  tag TEXT                                             -- "handoff", "branch:{name}", or NULL
);
CREATE TABLE snapshot_context (
  snap_id TEXT NOT NULL REFERENCES snapshots(snap_id),
  seq INTEGER NOT NULL,
  role TEXT NOT NULL CHECK(role IN ('system','user','assistant','tool')),
  content BLOB NOT NULL,
  is_stubbed INTEGER NOT NULL DEFAULT 0,
  stub_ssb_seq INTEGER,
  PRIMARY KEY (snap_id, seq)
);
CREATE INDEX idx_snapshots_session ON snapshots(session_id);
CREATE INDEX idx_snapshots_tag ON snapshots(tag) WHERE tag IS NOT NULL;
CREATE INDEX idx_context_stubbed ON snapshot_context(snap_id) WHERE is_stubbed = 1;
```

### RUST TYPES

```rust
struct SessionDag { db: Connection, session_id: String, active_branches: Vec<BranchRef> /* max 3 */ }
struct Snapshot { snap_id, parent_id: Option<String>, session_id, fsm_state, phase, step, findings: Vec<u8>, code_changes, tests_written, commit_hashes: Vec<String>, created_at, tag }
struct TrimmedMessage { seq: u32, role: MessageRole, content: Vec<u8>, is_stubbed: bool, stub_ssb_seq: Option<u64> }
struct Stub { original_type: String, original_tokens: usize, summary_hash: [u8;32] /* blake3 */, ssb_seq: u64 }
struct BranchRef { name: String, tip_snap_id: String, created_at: u64 }
enum MergeResult { Winner { branch, abandoned: Vec<BranchRef> }, Conflict { reason: String } }
```

### 3H.1 SQLITE SCHEMA AND TYPES
PRE_GATE: 3G passes; SSB; token_count; sessions dir writable.
TASK: Types; SessionDag::new/open; PRAGMA integrity_check -> Err(DatabaseCorrupted); DELETE journal mode; PRAGMA foreign_keys=ON.
DETERMINISTIC_CHECK: new creates file; idempotent; nonexistent Err(DatabaseNotFound); 2 tables; 3 indexes; serde; Stub round-trip; corrupted Err.
ADVERSARIAL: DELETE mode avoids WAL lingering on Windows; UUID via EntropyProvider (deterministic replay); FK enforced.
ACCEPTANCE: matches schema; divergence documented (see AMENDMENTS #1); types compile; FK active; idempotent.

### 3H.2 SNAPSHOT SAVE AND LOAD
PRE_GATE: 3H.1.
TASK: save_snapshot(snapshot, messages) in single transaction. load_snapshot -> JOIN, ordered by seq. latest_snapshot(tag). parent_chain(snap_id) walks parent_id with visited set. Batch insert ~50K/sec.
DETERMINISTIC_CHECK: save/load equality; 100 messages ordered by seq; latest with tag; empty -> None; root = [root]; cycle Err(CycleDetected) within 1000 iters; 1000 messages <100ms.
ADVERSARIAL: no transaction = orphan on crash; no visited set = infinite loop; seq ordering not INSERT order.
ACCEPTANCE: round-trip preserves; atomicity; acyclic chain; seq ordering.

### 3H.3 STRUCTURALLY LOSSLESS 3-PASS TRIMMING
PRE_GATE: 3H.2; token_count; SSB; 3E total_allocated_tokens() (R-3H-8).
TASK: trim_context(messages, config) where config = TrimConfig {stub_threshold, model_context_window, allocated_slots_tokens, output_reserve}. target_tokens = model_context_window - allocated_slots_tokens - output_reserve.
- **Pass 1 (identify)**: stream. tool-result > stub_threshold marked. user/assistant NEVER marked.
- **Pass 2 (stub)**: replace with Stub {original_type, original_tokens, summary_hash = blake3, ssb_seq}.
- **Pass 3 (validate)**: verify ssb_seq in SSB. If missing: UNSTUB. Total > target_tokens: Err(TrimBudgetExceeded).
- Slots 1-3 (system, rules, phase spec) NEVER stubbed (prompt cache).
- stub_threshold range [100, 1000].
DETERMINISTIC_CHECK: 3 tool results >500 tokens = 3 stubs; user/assistant byte-equal; blake3 match; SSB lookup returns original; missing -> unstubbed; all <stub_threshold -> 0 stubs; reduction>=0; same seed identical.
ADVERSARIAL: 500 messages <10s (batch if slow); blake3 collision impossible (ssb_seq primary); min threshold 100; Slot 1-3 never stubbed (assert seq <= slot_3_boundary).
ACCEPTANCE: lossless; reduction>=0; recoverable; within target; [100,1000] enforced.

### 3H.4 BRANCH AND MERGE OPERATIONS
PRE_GATE: 3H.2+3H.3.
TASK:
- create_branch(name, from_snap_id): verify <3 active; tag="branch:{name}"; copy snapshot_context from parent; add to active_branches.
- list_branches() active only.
- merge_branch(winner_name): create NEW snapshot with winner tip content + parent_id=current trunk tip (no mutation of existing parent_id — snapshots immutable). Abandoned: tag="abandoned:{name}". SSB action_type='checkpoint' (BLOCKER per SPEC AMENDMENTS #2) payload {event: "branch_merge", winner, abandoned, snap_ids}. Logical exclusion for SSB entries (default).
- close_branch(name) removes + marks abandoned.
- Branch naming: alphanumeric + hyphens, max 64 chars.
DETERMINISTIC_CHECK: branch tag=branch:test parent=A; 3 ok, 4th Err(BranchLimitExceeded{max:3}); list active only; merge b1 = winner, b2+b3 abandoned; post-merge trunk continues from b1 tip; close nonexistent Err(BranchNotFound); spaces Err(InvalidBranchName); SSB branch_merge.
ADVERSARIAL: merge not creating new trunk snapshot = stale trunk; file overlap in code_changes = Conflict (not silent pick); abandoned kept (queryable) excluded via WHERE tag NOT LIKE 'abandoned:%'.
ACCEPTANCE: limit 3; merge immutable DAG; abandoned tagged; SSB records; optional conflict detection; abandoned preserved.

### 3H.5 STUB RECOVERY FROM SSB
PRE_GATE: 3H.3; SSB query by seq.
TASK: recover_stubs(snap_id, ssb): parse Stub; query SSB by stub.ssb_seq; verify blake3(recovered)==stub.summary_hash; replace content; set is_stubbed=false, stub_ssb_seq=None. recover_single_stub for on-demand. Failures: StubRecoveryFailed{seq, reason}.
DETERMINISTIC_CHECK: valid recovers byte-equal; hash match; missing Err; mismatch Err(integrity); 0 stubs unchanged; non-stubbed unchanged; round-trip byte-equal.
ADVERSARIAL: pruned SSB = unrecoverable (SSB pruning not specified yet — flag as future risk); recovered size differs from stub.original_tokens = warn not error.
ACCEPTANCE: full round-trip; integrity; clear errors; single-stub independent.

### 3H.6 HANDOFF AND INTEGRATION TEST
PRE_GATE: 3H.1-3H.5; SSB + ContextBudget; token_count.
TASK:
- create_handoff: trim + save with tag="handoff"
- load_handoff: latest handoff; if branch exists, load branch tip (not trunk)
- Compaction boundary test R-3H-6: 85% triggers; 84.9% no trigger; 100% triggers
- Integration: 50 messages cycle, 2 branches, merge, acyclic chain, commit_hashes persist
DETERMINISTIC_CHECK: lifecycle ok; tag=handoff; real token_count; user/assistant byte-equal; recovered byte-equal; branches e2e; acyclic; hashes preserved; DAG acyclic; Slots 1-3 never stubbed; 100 runs identical.
ADVERSARIAL: load_handoff returns branch tip not trunk; same session_id same path shared; ClockProvider injected.
ACCEPTANCE: round-trip preserves; branch tip loaded; full 50-message lifecycle; commit_hashes persist; 100 deterministic.

EXIT_CRITERIA: save/restore/branch/trim without loss; user/assistant preserved; stub recoverable (or unstubbed); acyclic; max 3 branches; fits window; Slots 1-3 never stubbed; load_handoff branch tip; commit_hashes persist; deterministic.

---

## PHASE 3I: DETERMINISTIC SIMULATION TESTING (DST)
Prerequisites: Phase 0.8 (entropy quarantine) + 3A-3D. Prove FSM+SSB+BSP correctness via deterministic simulation. Reference model = infallible oracle. Same seed = same output.

DST INFRASTRUCTURE:
- **VCR pattern (LLM determinism)**: HTTP transport interception (reqwest middleware or madsim net mock). blake2s hash of HTTP body. cassette (msgpack). During DST: hash lookup, return recorded response + simulated latency from madsim timer. Unknown hash = test fails (drift). NOT at LlmProvider trait level.
- **TraceOps layer**: record entire agent execution trajectory. During DST, feed deterministic trajectory (no LLM).
- **Custom SQLite VFS**: trap write()/fsync(), in-memory buffer. PRNG-determined latency. ~200-400 LOC + unsafe FFI. Coordinates with madsim timer queue. Reference: Limbo project (API patterns only). SPIKE REQUIRED to evaluate if madsim libc hooks suffice.

### 3I.1 MADSIM WINDOWS SPIKE
PRE_GATE: cargo check ok.
TASK: Add madsim v0.2.34 dev-dep + cfg(madsim) gate. Minimal 2-node test with RUSTFLAGS="--cfg madsim". Document failure / alternative (manual deterministic scheduler wrapping SeededClock+SeededRng).
DETERMINISTIC_CHECK: madsim_spike exits 0.
ADVERSARIAL: failure documented before 3I.3.
ACCEPTANCE: passes OR alternative with equivalent fault injection.

### 3I.2 SKELETON
PRE_GATE: 3I.1.
TASK:
- SimulatorRuntime {SeededClock + SeededRng + madsim}. Methods: advance_time(duration), inject_fault(fault), snapshot_state() -> StateSnapshot.
- FsmReferenceModel implements ReferenceStateMachine trait:
  - type State = ReferenceState (FSM state + SSB row counts + agent lifecycle + channel state)
  - type Transition = FsmEvent enum
  - init_state() -> BoxedStrategy<ReferenceState>
  - transitions(state) -> BoxedStrategy<FsmEvent>
  - preconditions(state, transition) -> bool
  - apply(state, transition) -> ReferenceState
- **FsmEvent enum variants (10)**: PhaseTransition, AgentSpawn, AgentCrash, AgentCleanup, ToolCall, FileWrite, GrcStep, SSBWrite, ConfigChange, SuperstepBoundary (composite: clear_non_accumulating + execute + clear_ephemeral + evidence_gate). AgentCleanup: recovery after AgentCrash.
- StateMachineTest: SystemUnderTest = actual FSM+SSB, Reference = FsmReferenceModel. apply() returns SUT by value (ownership threading). check_invariants: FSM match, SSB counts, active agents, channel state (LastValue, Topic, Ephemeral).
- Cargo.toml: proptest v1.11+, proptest-state-machine v0.8+, madsim v0.2.34 cfg gate.
- #[cfg(madsim)] modules for IPC.
DETERMINISTIC_CHECK: cargo check; grep ReferenceStateMachine, SimulatorRuntime, StateMachineTest, SuperstepBoundary, AgentCleanup.
ADVERSARIAL: reference < 30% of real FSM LOC; every FsmState reachable; SuperstepBoundary two-phase Topic reset.
ACCEPTANCE: 6 types exist + compile; dev-deps listed.

### 3I.3 VERIFY
PRE_GATE: 3I.2.
TASK: cargo test --workspace; no new warnings.
DETERMINISTIC_CHECK: workspace test 0; warnings <= baseline.
ACCEPTANCE: zero regressions.

### 3I.4 TEST-WRITE: REFERENCE MODEL
PRE_GATE: 3I.3; proptest-state-machine.
TASK: prop_state_machine! { sequential 1..200 => FsmStateMachineTest }. 3-phase shrinking. PROPTEST_CASES=10000 CI + 10-min timeout, 256 local. madsim::runtime::Runtime::check_determinism(seed, config, bare fn pointer) — STATIC function, no closure captures. Synthetic regression: inject violation, generate proptest-regressions/ file, revert, commit. Tests WRITTEN, FAIL (TDD red).
DETERMINISTIC_CHECK: prop_state_machine! + check_determinism grep; cargo check --tests.
ADVERSARIAL: always-true preconditions -> at least 1 invariant fail (vacuous check); SSB/channel mutation caught.
ACCEPTANCE: tests compile; reference match + determinism + synthetic regression.

### 3I.5 TEST-WRITE: BSP DETERMINISM
PRE_GATE: 3I.4; BSP code; entropy verified (0 Instant::now/thread_rng/SystemTime::now).
TASK: BSP with same seed = identical checkpoint (100 reps); UntrackedValue never in checkpoint; EphemeralValue zero-write -> MISSING; Topic appends follow SSB seq; SuperstepBoundary two-phase. FAIL.
DETERMINISTIC_CHECK: bsp_determinism, untracked_never_checkpointed, ephemeral_clears greps.
ADVERSARIAL: UntrackedValue in checkpoint = divergence; unrecorded clear = recovery miss.
ACCEPTANCE: tests compile.

### 3I.6 TEST-WRITE: MADSIM IPC (Phase 4A prep)
PRE_GATE: 3I.1 madsim OR alternative; IpcGateway/IpcMessage exist; skip if unavailable.
TASK:
- madsim deterministic runtime
- net::Config: packet_loss_rate=0.3; send_latency: Duration::from_millis(0)..Duration::from_millis(500) (explicit Duration, not integer shorthand); NetSim::clog_link(src, dst) asymmetric
- SIGKILL mid-message + burst loss pattern (100% for 5 then 0%)
- All reproducible via seed replay. FAIL.
DETERMINISTIC_CHECK: ipc_packet_loss, ipc_asymmetric, ipc_agent_crash, ipc_burst_loss greps; RUSTFLAGS="--cfg madsim" cargo check --tests.
ADVERSARIAL: burst pattern not just random loss; no orphaned SSB after crash.
ACCEPTANCE: tests compile cfg(madsim).

### 3I.7 TEST-WRITE: ADVERSARIAL STATE SPACE
PRE_GATE: 3I.4-3I.5.
TASK:
- Anti-spiral: 3x same GRC step -> escalation
- Evidence gate: no artifacts -> BLOCK
- Context budget: oversized -> adaptive reduction
- SSB conflict: concurrent writes same path same superstep -> single-writer serializes
- Demonic scheduling: worst-case interleavings
Tests WRITTEN, FAIL.
DETERMINISTIC_CHECK: greps for anti_spiral, evidence_gate, context_budget, ssb_concurrent.
ADVERSARIAL: whitespace-only diff still detected; zero-byte "evidence" rejected.
ACCEPTANCE: compile.

### 3I.8 IMPLEMENT (TDD green)
PRE_GATE: 3I.4-3I.7 compile, red state.
TASK:
- SimulatorRuntime: SeededClock + SeededRng + madsim (cfg)
- FsmReferenceModel: pure apply(State, Event). Models 8 FSM states, SSB row counts, agent lifecycle (spawn/running/crashed/cleaned), channel state (UntrackedValue excluded). SuperstepBoundary atomic apply. AgentCleanup transitions crashed->cleaned. <30% real FSM LOC.
- StateMachineTest: init_test, apply RETURNS SUT by value, check_invariants.
- check_determinism(seed, config, bare_fn_pointer). No closure captures. Non-madsim: run twice same SeededClock+SeededRng.
- proptest-regressions/ auto-populated. Check into git.
- madsim IPC cfg(madsim), #[madsim::test]. NetSim fault profiles: partition(clog_link), delay(Duration range), drop(packet_loss_rate). buggify_with_prob().
DETERMINISTIC_CHECK: cargo test --workspace 0; PROPTEST_CASES=10000 within 10 min; RUSTFLAGS="--cfg madsim" cargo test ipc_ 0; python LOC check <30% FSM.
ADVERSARIAL: revert to strict invariants still passes.
ACCEPTANCE: all red tests green; reference <30%; zero 0-3H regressions; at least 1 regression file.

### 3I.9 MEASURE
PRE_GATE: 3I.8.
TASK: Metrics:
- state_coverage_pct (unique (FsmState, FsmState) pairs / total valid pairs). Target >80%
- bugs_found (invariant violations during dev)
- determinism_verified (100%, same seed = identical)
- regression_count (auto-shrunk files)
- ops_per_sec (UNGROUNDED target 100K, empirical)
- compile_time_delta_sec (normal) + madsim_compile_time_sec (separate, expected slower)
DETERMINISTIC_CHECK: .elai_cc/metrics/phase_3I.toml has all fields.
ACCEPTANCE: state coverage >80% valid pairs; 100% determinism; 0 regression.

EXIT_CRITERIA: reference matches for 10K+ ops; same seed identical; BSP determinism 100 runs; IPC fault injection (or skip documented); all adversarial pass; VCR replay identical SSB state; prompt hash mismatch reported (not silent); TraceOps all transitions; VFS spike complete. HITL full loop cross-phase verified (not just 3I). AUTONOMOUS not in 3I (Phase 4G).

---

## PHASE 3J: vLLM + ADVANCED PROVIDERS + PLUGINS + ORCHESTRATOR HARDENING
Prerequisites: Phase 3I + 1F (speculative decoding eval). Integrate vLLM as production backend. Enable layer-level opts (DoLa, TurboQuant KV, steering vectors) requiring vLLM internals. Build ModelPool features (GPU multiplexing, predictive pre-loading, workload profiling). MCP bridge, tiered static analysis, AppContainer, advanced SSB types, MILP eval.

All model-specific work on Gemma4 E2B (~2B, fits RTX 3060 12GB). Final step 3J.15 applies to Gemma4 31B via CPU offload on 64GB RAM.

HARDWARE: Ryzen 7800X3D, 64GB RAM, RTX 3060 12GB. 48GB setup NOT available during phase. Test model Gemma4 E2B Q4 ~4-5GB. Production target Gemma4 31B Q4 ~18GB (requires 48GB for realtime).

MODEL SETUP (completed 2026-04-08):
- gemma4:31b — 19GB, Q4_K_M, 31.3B params, 262144 context, embed 5376. Multimodal (vision+text).
- gemma4:e2b — 7.2GB, ~2B params (primary test model).
- gemma3:4b — 3.3GB, fallback.
- vLLM 0.19.0 in WSL2 Ubuntu at ~/vllm-env/ (Python 3.12 virtualenv). PyTorch 2.10.0+cu128. Serve: `vllm serve google/gemma-4-e2b --host 0.0.0.0 --port 8000`. Rust on Windows -> WSL2 localhost (auto-forwarded).

### 3J.1 VLLM HTTP CLIENT SPIKE
PRE_GATE: cargo check; vLLM serving Gemma4 E2B.
TASK: providers/src/vllm_client.rs with OpenAI-compatible /v1/completions, /v1/models. SSE parsing (reuse sse.rs). Latency: 100 requests p50/p95/p99 vs Gemma4 E2B. Health: GET /health -> VllmHealthStatus. Evaluate gRPC; HTTP-only if not. VllmProvider implements LlmProvider (complete, embed, token_count). cfg(feature = "vllm").
DETERMINISTIC_CHECK: cargo check --features vllm; grep VllmProvider + impl LlmProvider; .elai_cc/metrics/vllm_spike.toml.
ADVERSARIAL: down server timeout 5s not hang; real SSE replay in test; latency >10ms overhead acceptable (layer access is value).
ACCEPTANCE: compiles + trait impl; latency measured; health works; feature-gated.

### 3J.2 VLLM MODELPOOL INTEGRATION
PRE_GATE: 3J.1.
TASK: InferenceBackend enum { Ollama(OllamaConfig), Vllm(VllmConfig) }. VllmConfig: server_url, tensor_parallelism: bool, served_model_names: Vec<String>. ModelPool delegates GPU memory to vLLM (no nvml direct). POST /v1/load_model, DELETE /v1/unload_model. Tensor parallelism: vLLM handles sharding. Health via VllmProvider.health(). elai_cc.toml [providers.backend] = "ollama" | "vllm". Fallback with WARNING log on unavailable vLLM.
DETERMINISTIC_CHECK: grep InferenceBackend, VllmConfig, tensor_parallelism.
ADVERSARIAL: no nvml when backend=Vllm; WARNING msg includes "layer-level optimizations unavailable".
ACCEPTANCE: both backends via enum; tensor parallelism configurable; fallback tested; elai_cc.toml configurable.

### 3J.3 DOLA LAYER-CONTRAST DECODING
PRE_GATE: 3J.1; vLLM exposes per-layer logits OR plugin/hook API (else SPIKE + DEFER).
TASK: DolaDecoder in providers/src/layer_config/dola.rs. Algorithm (arxiv 2309.03883):
1. Forward pass to hidden states at candidate layers J and final N
2. Early-exit q_j = softmax(phi(h_t^(j)))
3. M = argmax_{j in J} JSD(q_N || q_j)
4. V_head = {x : q_N(x) >= alpha * max q_N}
5. Contrastive: log(q_N / q_M) for V_head, -inf otherwise
6. Sample from softmax(contrastive)

Buckets: ~2B (E2B 18-26 layers): even in [0,N/2) or [N/2,N) UNTESTED; 7B (32): even [0,16) or [16,32); 13B (40): even [0,20) or [20,40); >30B: 3-4 buckets of 20. Hyperparams: alpha=0.1, theta=1.2. DolaConfig{enabled, alpha, candidate_buckets, repetition_penalty}. Per-role: Executor (Gemma 4 31B) enabled; Auditor (DeepSeek-R1-32B) amplified; Prompt-engineer (Qwen 3.5 27B) disabled. E2B: 20 prompts factuality baseline vs DoLa. Latency 5-8% per paper. Memory +0.6-1.1%.

DEGRADED path: if only top-N logprobs available, DoLa accuracy reduced (document + measure).
CAUTION: DoLa degrades on 335M. <1B auto-disable + WARNING. 1-7B EVALUATE empirically.
DETERMINISTIC_CHECK: grep DolaDecoder, DolaConfig, candidate_buckets.
ADVERSARIAL: top-5 logprobs only = impossible full-JSD (degraded doc); DoLa disabled when tier2_mode=INSTRUCTOR; repetition penalty compare.
ACCEPTANCE: compiles + VllmProvider integration; per-role toggle; <1B guard; E2B eval logged; latency <=10% budget.

### 3J.4 TURBOQUANT KV CACHE COMPRESSION (EVALUATION)
PRE_GATE: 3J.1; turboquant-interface.md.
TASK: EVALUATE feasibility. TurboQuant (arxiv 2504.19874):
1. Rotation Pi (QR of Gaussian, d x d, fixed per instance)
2. Beta distribution coords converge to N(0,1/d) at high d
3. Lloyd-Max scalar quantizer per coordinate
Expected: 3.5-bit matches full (0.997 NIAH); 2.5-bit marginal (49.44 vs 50.06 LongBench).
Evaluate: a. vLLM attention backend custom KV quant? b. LOC + complexity. c. External quantizer path? d. Min d>=64.
If FEASIBLE: TurboQuantConfig{bit_width: u8 (1-4), rotation_seed: u64}. TurboQuantizer struct with precomputed Pi + Lloyd-Max codebook per (d, b). quant_kv / dequant_kv.
Invariants: unit norm input (store L2 separately); Pi orthogonal; codebook 2^b centroids ascending sorted.
Pitfalls: don't reuse Pi across instances; don't skip norm rescale; don't approximate with Hadamard; don't use MSE variant for inner product (biased).
E2B: precompute Lloyd-Max codebooks for E2B head dimension. Codebooks d-specific, not model-specific. Reusable for 31B if same d.
Output: FEASIBLE/DEFER in workflow/reasoning/phase-3J-turboquant.md.
DETERMINISTIC_CHECK: decision file exists; grep TurboQuantizer if FEASIBLE.
ADVERSARIAL: PagedAttention assumes full precision = wrong silently; compression saves VRAM but adds latency.
ACCEPTANCE: decision documented; if FEASIBLE codebook verified; if DEFER no dead code.

### 3J.5 STEERING VECTORS (ActAdd)
PRE_GATE: 3J.3; actadd-2308.10248/summary.md.
TASK: SteeringVectorEngine in providers/src/layer_config/steering.rs. ActAdd (arxiv 2308.10248):
1. h_A^l = h+^l - h-^l from contrast prompt pair
2. Add c * h_A^l to residual at layer l during inference
3. Continue forward pass

SteeringVector {layer: usize, vector: Vec<f32>, coefficient: f64}. Precomputed:
- Auditor: skepticism steering (contrast pair TBD)
- Context-compiler: completeness (temp 0.7 + steering)

SteeringConfig{vectors: Vec<SteeringVectorSpec>, enabled}. SteeringVectorSpec{name, layer, coefficient, positive_prompt, negative_prompt}. Configurable [agents.{role}.layer_config.steering].
Layer guidance (paper 6.3): early (0-5) minimal; mid (6-20) peak; late (25+) declining. Default model_layers/3.
Coefficient guidance (6.4): c in [3, 20], start c=5.
vLLM integration: same hook as DoLa. If DoLa deferred, steering deferred.
Composable (paper section 9).
Local models only (API can't access intermediate activations).
E2B: compute >=2 pairs (skepticism, completeness). ActAdd validated on GPT2-XL (1.5B) so E2B in range. 5 prompts per vector. Store .safetensors/.npz.
DETERMINISTIC_CHECK: grep SteeringVectorEngine, SteeringConfig.
ADVERSARIAL: c too high = incoherent (sweep 1-30, coherent at <=15); layer sweep at least one measurable effect; validate each pair with 5 prompts.
ACCEPTANCE: compiles; per-role config; non-zero vector; coupled with DoLa.

### 3J.6 GPU MULTIPLEXING + PREDICTIVE PRE-LOADING
PRE_GATE: 3J.2; dual-GPU OR simulation.
TASK: Multiplexing (Murakkab-derived): small (<4GB) colocate with large idle headroom. Embedding (~1.5GB) + cross-encoder (~130MB) on GPU1 while executor on GPU0. Memory-writer (~4GB) shares GPU1 with embedding during MEMORIZING. Constraint: sum(colocated) <= GPU_VRAM - safety_margin (1GB). Phase 0-1 single GPU no multiplex. Phase 2+ dual GPU.
MultiplexingPolicy{safety_margin_bytes: u64 (default 1GB=1_073_741_824), max_colocated: usize (default 3), min_model_size_for_dedicated_gpu: u64 (default 4GB)}.
ModelPool.can_colocate(model_id, target_gpu): current_vram + model + safety <= total.
**Predictive pre-loading FSM-driven**:
- EXECUTING -> pre-load auditor
- AUDITING -> pre-load executor
- PLANNING -> pre-load prompt-engineer
- TESTING_FIRST -> pre-load executor

Triggers on state entry. Background tokio::spawn. Blocked if VRAM insufficient (fallback sequential at boundary). PreloadPolicy: map FsmState -> Vec<ModelId>. [providers.preload].

**SLO Quality Tiers** (Murakkab):
- **BEST** (auditor, researcher): highest precision, full VRAM
- **GOOD** (executor, context-compiler): standard
- **FAIR** (prompt-engineer): smaller/quantized
- **BASIC** (memory-writer): smallest capable, minimal

[agents.{role}.slo_tier]. Tier mismatch (BEST+BASIC) -> WARNING.
DETERMINISTIC_CHECK: grep MultiplexingPolicy, PreloadPolicy, SloTier, can_colocate.
ADVERSARIAL: sum > total = false; pre-load waits for phase boundary; margin health check + retry.
ACCEPTANCE: enforced + safety margin; pre-load triggers; tiers configurable; single-GPU disables multiplex.

### 3J.7 WORKLOAD PROFILING
PRE_GATE: 3J.6; SsbWriter.
TASK: Per-agent per-phase: tokens_in, tokens_out, latency_ms, vram_peak_mb. Written action_type='workload_profile' at phase_complete.
WorkloadProfile{agent_role, phase_id, tokens_in: u64, tokens_out: u64, latency_ms: u64, vram_peak_mb: u32, model_id, gpu_id: u8, timestamp: i64 (ClockProvider)}.
Storage: SSB action_type='workload_profile', payload=msgpack. Query: SELECT FROM ssb WHERE action_type='workload_profile' AND project_id=? Used by MILP (3J.8).
DETERMINISTIC_CHECK: grep WorkloadProfile, workload_profile.
ADVERSARIAL: measurement during execution not after cleanup; <1ms latency budget.
ACCEPTANCE: emitted every phase boundary; queryable; zero measurable phase latency.

### 3J.8 MILP SCHEDULING EVALUATION
PRE_GATE: 3J.7; >=10 profile entries.
TASK: EVALUATE MILP (Murakkab-derived): "Implement only if heuristic insufficient in Phase 2 metrics". Formulation:
- model_assignment[model, gpu] in {0,1}
- Min total phase transition latency
- sum(vram[model] * assignment[model, gpu]) <= gpu_vram[gpu] - safety
- sum(assignment[model, gpu] for gpu) == 1
- Concurrently-needed models on different GPUs or colocated within VRAM
- Reoptimize at phase boundaries

Rust solvers: good_lp v1.10+ (HiGHS MIT or CBC EPL); Windows build check (R5).
**Decision criteria**:
- Heuristic <5% worse -> DROP MILP
- MILP >10% improvement -> ADOPT good_lp + HiGHS
- 5-10% -> EVALUATE in Phase 4

**Heuristic baseline always needed**: GreedyModelPlacer sorts by VRAM desc, assigns to most free space. Respects 3J.6 policy.
workflow/reasoning/phase-3J-milp.md.
DETERMINISTIC_CHECK: decision file; grep GreedyModelPlacer; if ADOPT grep MilpModelPlacer.
ADVERSARIAL: MILP solve <50ms budget; infeasibility -> greedy fallback + WARNING.
ACCEPTANCE: greedy always available; documented with measurements; if ADOPT handles infeasibility; if DROP greedy is production.

### 3J.9 MCP BRIDGE
PRE_GATE: 3C; claw-code source.
TASK: Port mcp_tool_bridge.rs (912 LOC) -> plugins/src/mcp_bridge.rs; mcp_lifecycle_hardened.rs (839 LOC) -> plugins/src/mcp_lifecycle.rs.
McpBridge trait: connect(server_url) -> Result<McpConnection>; list_tools(conn) -> Result<Vec<McpToolSpec>>; call_tool(conn, tool, args) -> Result<Value>; disconnect(conn) -> Result<()>.
McpConnection: transport (stdio, SSE, streamable HTTP).
Error recovery: exponential backoff (3 attempts 1s/2s/4s); heartbeat timeout 30s; graceful degradation; no silent failure.
Tool registration: MCP tools in orchestrator registry with uniform ToolRequest/ToolResult. Proxied (agent never connects directly).
Security: external untrusted; sanitize results; no prompt injection (data not instructions); 30s timeout per call. cfg(feature = "mcp").
DETERMINISTIC_CHECK: cargo check --features mcp; grep McpBridge, McpConnection, mcp_lifecycle.
ADVERSARIAL: "ignore previous" not executed; 30s timeout not hang; version handshake rejects mismatch.
ACCEPTANCE: compiles + connects; error recovery; security; feature-gated.

### 3J.10 TIERED STATIC ANALYSIS HOOKS
PRE_GATE: 3F; SSB write path.
TASK:
- **T1 (RULES, <1ms)**: every SSB write. Predicates from rules.db for active phase + domain. Types: AST_PATTERN, GREP_PATTERN, METRIC_GATE, COMPOSITE. Output: ALLOW|WARN|BLOCK.
- **T2 (AST, <10ms for <5000 lines)**: action_type='file_write' ONLY. tree-sitter parse. Checks: syntax, imports, dead code, lazy code. Circuit breaker: files >5000 lines degrade to T1 only.
- **T3 (PATTERN MATCH, <5ms)**: when WARN exists AND no BLOCK from T1/T2. Pattern-match library. NO LLM. Examples: function>200 lines WARN:complexity; catch Exception: pass BLOCK:swallow.

Hook integration: T1 sync <1ms; T2 async for file_write; T3 conditional.
BLOCK -> SSB 'finding'.
Findings pathway: BLOCK/WARN -> SSB 'finding' entry. Orchestrator normalizes to pattern hash (blake2s). Feeds violation_patterns (rules.txt §6). Circular trust boundary: orchestrator normalizes (not auditor).
Verdict sanitization: BLOCK -> generic "Tool call rejected by policy" (no rule ID/tier/pattern).
DETERMINISTIC_CHECK: grep TieredAnalysis/T1Hook/T2Hook/T3Hook, circuit_breaker; T1<1ms, T2<10ms, T3<5ms.
ADVERSARIAL: T1 median<0.5ms with 100 rules; T2 circuit at 5000; T3 does NOT fire if T1=BLOCK.
ACCEPTANCE: 3 tiers implemented; performance targets; sanitized; feeds via blake2s.

### 3J.11 APPCONTAINER ISOLATION (Windows)
PRE_GATE: 3C; Windows 10/11.
TASK: AppContainer for agents (orchestrator §12 L599-610). CreateAppContainerProfile. Filesystem: only granted paths. Network: none by default (Researcher: add). Registry: none. Process: isolated.
AppContainerConfig{profile_name: "elai_cc_agent_{agent_id}", granted_paths: Vec<PathBuf>, network_allowed: bool}.
Job object limits: JOB_OBJECT_LIMIT_ACTIVE_PROCESS=1, JOB_OBJECT_LIMIT_PROCESS_MEMORY=512MB, JOB_OBJECT_LIMIT_JOB_TIME=timeout.
Fallback: restricted token + Job object. Known weakness: NTFS junctions + reparse bypass ACL deny (documented).
**Self-modification locks** (§12 L628-634): IPC-level reject WriteFile/EditFile to locked (canonical realpath comparison); filesystem-level not granted in AppContainer; git-level filter GitCommit if diff touches locked path.
Environment sanitization: pass only LANG, TERM, PATH (restricted), TMPDIR. NOT HOME, USER, API keys, ELAI_CC_*.
Platform: #[cfg(target_os="windows")] AppContainer; #[cfg(target_os="linux")] clone()+seccomp (4A). Common trait: AgentSandbox {create, launch_agent, cleanup}.
DETERMINISTIC_CHECK: grep AppContainerConfig, AgentSandbox, JOB_OBJECT_LIMIT.
ADVERSARIAL: junction normalization attacks BLOCK; 1GB alloc killed at 512MB; CreateAppContainerProfile fail explicit error.
ACCEPTANCE: profiles per lifecycle; Job limits; self-mod locks; env sanitized; fallback documented.

### 3J.12 ADVANCED SSB ACTION TYPES
PRE_GATE: 3B; 3J.7.
TASK: Implement remaining action_types (3B added 8: file_write, file_read, tool_result, test_outcome, finding, claim, checkpoint, phase_transition). This step adds:
- model_swap: {from_model, to_model, gpu_id, swap_duration_ms, reason}
- knowledge_degradation: {component, old_mode, new_mode, reason}
- isolation_metric: {agent_id, phase_id, context_savings_ratio}
- plan_update: {phase_id, version, change_summary}
- replan_request: {agent_id, severity: MINOR|MAJOR, evidence}
- stub_reference: {phase_id, step_id, stub_hash}
- reasoning_trace: ReasoningTraceEntry (§7A)
- memory_monitoring: {channel_id, deep_size_bytes, threshold_exceeded}
- phase_complete: {phase_id, agent_id, evidence_artifacts, commit_hashes, test_outcomes}
- workload_profile: WorkloadProfile (3J.7)

Typed validation at write time (not read). Mismatch -> Err.
Queries: reasoning_trace by agent_id; phase_complete by phase_id; isolation_metric GROUP BY phase_id.
Cognitive isolation: context_savings_from_isolation = sum(agent_grc_tokens_this_phase) / model.max_context. Logged at phase transition.
DETERMINISTIC_CHECK: grep action_types; count>=17.
ADVERSARIAL: wrong payload Err at write (not read); reasoning_trace monotonic per agent.
ACCEPTANCE: 17 types typed; write-time validation; queries verified; isolation computed+logged.

### 3J.PIPELINE CLI-ORCHESTRATOR INTEGRATION
PRE_GATE: 3J.COMPOSE passed (sidecar with DoLa + TurboQuant); 3J.LOCAL passed (CLI-to-sidecar via Anthropic protocol); 3A-3I compiled.
TASK: Wire CLI's ConversationRuntime through orchestrator pipeline (FSM-aware tool filtering, ContextBudget, agent dispatch, rules).

Current: CLI -> AnthropicClient -> API (bypasses infra).
Target: CLI -> Orchestrator.dispatch() -> Backend.

**A. BACKEND ABSTRACTION**:
- InferenceDispatcher trait in orchestrator: `fn dispatch(request: &MessageRequest, context: &DispatchContext) -> MessageStream`
- DispatchContext: FSM state, agent role, tool filter, context budget, backend config
- ApiDispatcher (wraps AnthropicClient); LocalDispatcher (wraps SidecarProvider)
- Selection via elai_cc.toml:
```toml
[backend]
mode = "local" | "api" | "hybrid" | "auto"
[backend.role_override]
executor = "local"
auditor = "api"
prompt_engineer = "local"
```
- "auto": try local first, fallback API if unhealthy.

**B. FSM-AWARE TOOL FILTERING**:
- Query FSM state -> active tool set
- PLANNING: read-only (read_file, glob_search, grep_search, WebSearch)
- EXECUTING: full (bash, edit_file, write_file)
- AUDITING: read-only + test runner
- Local backend: max 15 tools + truncated descriptions
- API backend: all 49+

**C. CONTEXT BUDGET INTEGRATION**:
- ContextBudgetAllocator (3E) before dispatch
- System prompt slot fixed; tools bounded by filter; messages remaining
- Local: total = model max (131K E2B) - KV cache overhead. Conservative 80%.
- API: total = model max (200K Claude).

**D. CONVERSATION RUNTIME REFACTOR**:
- ConversationRuntime.send_message() -> dispatcher.dispatch()
- Dispatcher: tool filter -> context budget -> format -> send
- Response handling unchanged (SSE identical)
- Slash commands (/model, /mode) switch backend at runtime

**E. AUTO-START SIDECAR**:
- "local" or "auto": CLI health check on startup
- Spawn WSL2 sidecar (reuse elai.bat) if unhealthy, wait max 120s
- If sidecar fails + mode="auto": fallback API
- Kill sidecar on CLI exit

DETERMINISTIC_CHECK: cargo check --all-features; grep InferenceDispatcher, LocalDispatcher, ApiDispatcher; elai_cc.toml [backend] with mode.
ADVERSARIAL: sidecar crash mid-conversation = CLI error not hang; removed tool name still in system prompt ("tool X not available"); tool defs + system too big = graceful error not truncation.
ACCEPTANCE: CLI identical local+API; FSM tool set visible; budget prevents OOM; elai --local via orchestrator; default (API) via same pipeline; /mode runtime switch.

### 3J.UNIFIED SINGLE RUNTIMECLIENT WITH CONFIG-DRIVEN BACKEND
PRE_GATE: 3J.PIPELINE passed (CliApiClient enum dispatch works).
TASK: Eliminate duplicated AnthropicRuntimeClient + LocalRuntimeClient (~140 line stream() duplication). Single UnifiedRuntimeClient config-driven.

**A. UNIFIED RUNTIME CLIENT**: RuntimeClient struct: client: AnthropicClient (base_url configurable), auth: AuthSource, backend: BackendMode, tool_policy: ToolPolicy, budget_policy: BudgetPolicy. ONE stream() branches on backend for: auth, tool filter policy, budget ceiling, prompt cache. Shared: SSE, markdown, tool accumulation, error handling.

**B. CONFIG-DRIVEN TOOL POLICY**: ToolPolicy: Full (API), FsmFiltered{max_tools, fsm_state_path} (Local), None (Idle). Tool filtering in stream() based on policy. New tool -> mvp_tool_specs() + classify() auto-available to both.

**C. CONFIG-DRIVEN BUDGET POLICY**: BudgetPolicy: ApiDefault{ceiling: 200K}, Local{ceiling: 131K, fraction: 80%}. Budget in stream() based on policy. Future features (adaptive, prioritization) wire once.

**D. ELIMINATE BUILD DUPLICATION**: Remove build_local_runtime(). build_runtime() takes BackendConfig. Remove LocalCliConfig from LiveCli. Replace with Option<BackendConfig>. --local sets mode=Local. No flag = mode=Api (reads auth from env).

**E. AUTO-WIRE GUARANTEE**: New feature in stream() -> both backends. New tool in mvp_tool_specs() -> both. New FSM state -> both. New config field -> readable by both. Zero per-phase wiring.

DETERMINISTIC_CHECK: cargo check; 0 hits for AnthropicRuntimeClient/LocalRuntimeClient/CliApiClient in cli/main.rs; 1 hit "struct RuntimeClient"; elai --local enters ConversationRuntime.
ADVERSARIAL: max 3 branch points on backend (auth, tool policy, budget); BackendConfig <=10 fields (complex in sub-structs LocalConfig, ApiConfig); ported tests + new test asserting API + Local policies give same counts as old.
ACCEPTANCE: single RuntimeClient; ONE stream() with policy branching; prepare_turn_runtime no if/else; future phase touches ONE code path; zero regression.

SESSION SUMMARY (3J.PIPELINE, 2026-04-09):
- orchestrator/src/dispatcher.rs (NEW): DispatchContext, BackendMode, BackendConfig, DispatchError (thiserror), build_dispatch_context(). 7 unit tests.
- orchestrator/src/fsm.rs: Fsm::read_persisted_state() for sync FSM reads.
- orchestrator/src/lib.rs: pub mod dispatcher + re-exports.
- cli/main.rs: LocalRuntimeClient (AuthSource::None + sidecar base_url), CliApiClient enum, start_sidecar() exponential backoff + drive-letter WSL path translation, build_local_runtime(), LiveCli::new_local(), prepare_turn_runtime routing, sidecar /shutdown on CLI exit.
- python/server.py: /shutdown endpoint, real tokenizer counting (input_ids.shape[1] replacing len(prompt)//4).
- elai.bat: simplified to --local flag, HOME env fallback.
Patches: patch-0.3 (real tokenizer), patch-0.4 (Idle=no tools, per-turn local routing, sidecar shutdown on exit).
Auditor fixes: windows_to_wsl_path() general; FsmState::check_tool_allowed in stream(); persisted .fsm-state.msgpack per-turn; removed Hybrid/Auto/ToolOverflow/sidecar_port; bash -c quoting; prepare_turn_runtime routes by local_config.
Test results: 7/7 dispatcher GREEN; cargo check clean; 56 passed pre-commit.

### 3J.WIRE.FOUNDATION — SSB + FSM + INTERACTIVE GATE IN CLI
PRE_GATE: 3J.UNIFIED; orchestrator exports Ssb, SsbConfig, Fsm, FsmConfig, InteractiveGate, EvidenceSummary, GateDecision, RiskLevel.
TASK: Wire 3A-3B infra into LiveCli:
1. LiveCli fields: ssb: Ssb, fsm: Arc<Fsm>, clock: Arc<dyn ClockProvider>. Construct in LiveCli::new() + ::new_local(). Ssb path: sessions/{sid}/ssb.db. Fsm path: sessions/{sid}/.fsm-state.msgpack. SystemClockProvider.
2. CliInteractiveGate (impl InteractiveGate terminal I/O). present_gate(): print EvidenceSummary + RiskLevel, prompt Approve/Reject/Modify/Redirect. LOW auto-advance.
3. Replace prepare_turn_runtime Fsm::read_persisted_state with self.fsm.get_state() (own instance, not file read).
4. Wire Ssb via with_session_tracer() -> SessionTracer backed by Ssb writes (turn_started, tool_execution, turn_completed as SsbPayload).
5. Shutdown: Ssb::shutdown() in LiveCli drop. Fsm auto-persists.

ADVERSARIAL GUARD: Ssb requires Tokio runtime for write task. LiveCli has RuntimeClient with Runtime. Reuse or shared. Do NOT create second Tokio runtime (panics on nested block_on). Fsm::transition() async but CLI sync = runtime.block_on().
DETERMINISTIC_CHECK: cargo check; grep Ssb::new, Fsm::new|Fsm::load, CliInteractiveGate; 0 new failures orchestrator tests; 56 pre-commit still pass.

### 3J.WIRE.BUDGET — REAL CONTEXT BUDGET SLOT ALLOCATION
PRE_GATE: 3J.WIRE.FOUNDATION; ContextBudget::compute with 8 slots.
TASK: Replace [0,0,0] placeholder + output_reserve=4096:
1. Measure fixed slots at prompt assembly:
   - Slot 0 SystemPrompt: total chars / 4
   - Slot 1 Rules: 0 (3J.WIRE.RULES fills)
   - Slot 2 PhaseSpec: 0 (3J.WIRE.PLAN fills)
2. Use computed budget for max_tokens: remaining = total_allocated - total_used; max_tokens = min(max_tokens_for_model, remaining).
3. Wire ContextBudget into ConversationRuntime per-turn (field in RuntimeClient or through build_runtime).
4. Log in [pipeline] telemetry: slots, allocated vs used, effective max_tokens.

ADVERSARIAL: do NOT change API path (200K+). BudgetPolicy::ApiDefault skips ContextBudget. chars/4 is approximate; sidecar reports real input_tokens — log delta.
DETERMINISTIC_CHECK: cargo check; 0 hits fixed_slot_sizes.*[0.*0.*0]; [pipeline] shows non-zero; cargo test context_budget 0 failures.

### 3J.WIRE.RULES — RULES ENGINE INTO PRE-TOOL HOOKS
PRE_GATE: 3J.WIRE.BUDGET; crates/rules has DefaultRuleEngine, compile_predicate, evaluate; 3F.7 seed data.
TASK: Wire rules into CLI tool execution:
1. Seed rule loading (if 3F.7 not done): load from TOML/JSON into rules.db. Engineering + security (~30-40 core rules). Mirror CLAUDE.md: no .unwrap(), impact analysis, no silent swallow.
2. Wire DefaultRuleEngine into PreToolUse. Evaluate active rules against EvaluationContext (action=tool_name, source=agent_input, file_path). BLOCK -> rejected. WARN -> logged Ssb. ESCALATE -> InteractiveGate prompt.
3. AstEvaluator stub: Pass for all AST_PATTERN (real tree-sitter Phase 4C).
4. DkgQuerier stub: empty graph (real DKG Phase 4B).
5. Update ContextBudget Slot 1 Rules with loaded rules token count.
6. Strip CLAUDE.md rule content from local model prompt: rules enforced by engine, model doesn't need them. Flag on build_system_prompt()/SystemPromptBuilder to skip when rules engine active.

ADVERSARIAL: do NOT remove CLAUDE.md for API path. Seed rules version-controlled in `.elai_cc/data/rules/` TOML (not runtime). Rule eval <5ms per call (3F Tier 1 target), log timing.
DETERMINISTIC_CHECK: cargo check; grep DefaultRuleEngine; cargo test elai-rules 0 new failures (263+ pass); >=10 seed rules loaded; local prompt no CLAUDE.md content.

### 3J.WIRE.SESSION — SESSION DAG REPLACING RAW PERSISTENCE
PRE_GATE: 3J.WIRE.FOUNDATION (Ssb); SessionDag exports SessionDag, Snapshot, TrimmedMessage, TrimConfig.
TASK: Replace Session jsonl with DAG snapshots + 3H trimming:
1. SessionDag in LiveCli::new(). Path: sessions/{sid}/dag.db. Wire EntropySource + ClockProvider.
2. Auto-snapshot after each turn via save_snapshot() with FSM state + phase + messages converted to TrimmedMessage.
3. Wire trim_context() into prepare_turn_runtime: before API request check 85% of budget (3H handoff). If so, trim via 3-pass identify->stub->validate. Stubs ref Ssb seqs.
4. /resume via SessionDag::load_snapshot() + parent_chain() (not raw Session::load_from_path).
5. /session fork via SessionDag::create_branch(). Max 3.
6. Stub recovery: SessionDag::recover_stubs() when loading stubbed messages.

ADVERSARIAL: SessionDag sync SQLite (not WAL). Do NOT mix with Ssb WAL. Separate db files + connections. bytes/4 heuristic for local; real tokenizer for API. /compact still manual override.
DETERMINISTIC_CHECK: cargo check; grep SessionDag::new|open, save_snapshot, trim_context; cargo test session_dag 0 failures; /resume works.

### 3J.WIRE.PLAN — PLAN SKELETON AWARENESS IN CLI
PRE_GATE: 3J.WIRE.SESSION; PlanSkeleton exports PlanSkeleton, PhaseEntry, PhaseDetail.
TASK:
1. plan: Option<PlanSkeleton> in LiveCli. Load from `.elai_cc/plans/{pipeline_id}.toml`.
2. /plan slash command: skeleton display (phases, status, deps). /plan phase <id> shows PhaseDetail.
3. Wire ContextBudget Slot 2 PhaseSpec: PlanSkeleton::load_phase_context() content + token_count.
4. Phase transition: FSM state change updates active phase. PlanSkeleton::propagate_changes() notifies dependents via Ssb.
5. Contract tracking: display input/output for active phase. Satisfied vs pending.

ADVERSARIAL: optional. CLI works with plan: None. load_phase_context() needs LlmProvider for token counting (local chars/4, API AnthropicClient tokenizer if available). Do NOT auto-create plans (user/workflow authors).
DETERMINISTIC_CHECK: cargo check; grep PlanSkeleton; /plan parses; cargo test plan_skeleton 0 failures; no plan file = normal run.

### 3J.WIRE.STUBS — REPLACE PRODUCTION STUBS WITH REAL ADAPTERS
PRE_GATE: 3J.WIRE.RULES; DkgEngine.build_graph() implemented in knowledge/src/dkg/mod.rs; tree-sitter + tree-sitter-rust in knowledge/Cargo.toml.
TASK:
**STUB INVENTORY** (session s-877904):
- IN SCOPE (rule engine deps, wire here):
  (a) PassAstEvaluator -> TreeSitterAstEvaluator
  (b) EmptyDkgQuerier -> DkgQuerierAdapter
- DEFER (own phase domains):
  (c) OllamaProvider 4x todo!() — sidecar/provider phase
  (d) OllamaEmbedder 4x todo!() — sidecar/provider phase
  (e) NetworkVerifier — reasoning 2B
  (f) NoopTestRunner — reasoning 2B
  (g) OrtSession::embed() — knowledge vector (Step 4)
  (h) VectorIndex readiness/rerank — knowledge vector (Step 4)

**Deliverable (a) TreeSitterAstEvaluator** in crates/knowledge/src/ast_adapter.rs. Impl elai_rules::predicate::AstEvaluator. HashMap<&'static str, tree_sitter::Language>. evaluate_query(lang, source, query): Parser parse; Query from string; QueryCursor collect matches; map to Vec<AstMatch> with byte offsets + text. validate_query: attempt Query::new. Add elai-rules to knowledge/Cargo.toml (or re-export trait). LanguageGrammar exists but for node extraction (DKG); AstEvaluator needs tree-sitter Query API directly.

**Deliverable (b) DkgQuerierAdapter** in crates/knowledge/src/dkg_adapter.rs. Impl elai_rules::predicate::DkgQuerier. Arc<DkgEngine> or shared Connection. query_reachability(params: &ReachabilityParams):
1. Find source nodes LIKE source_pattern
2. Find sink nodes LIKE sink_pattern
3. BFS/DFS dkg_edges up to max_depth
4. Filter by edge_types
5. must_not_exist: empty if path found, match if not
6. must_traverse: verify intermediate
7. Return Vec<ReachabilityMatch> with path + edge_labels
validate_params: non-empty patterns; max_depth<=20. Note: DkgEngine has no query_reachability yet (only build_graph populates). Adapter adds read-only queries.

Wire CLI (build_runtime in cli/src/main.rs): PassAstEvaluator -> TreeSitterAstEvaluator::new(); EmptyDkgQuerier -> DkgQuerierAdapter::new(dkg_engine). DkgEngine built at startup OR lazily (expensive). Add elai-knowledge dep to crates/cli/Cargo.toml. Seed rules now use AstPattern + GraphReachability (currently all 12 GrepPattern — add 2-3 AST rules).

ADVERSARIAL: tree-sitter Query::new can panic on invalid grammar — always Result, never unwrap. BFS needs visited set (max_depth alone insufficient, cycles = exponential). DkgEngine::build_graph() O(project_size). Build once at startup, update incrementally. Failure -> fall back to EmptyDkgQuerier + warning (no panic). AstEvaluator AstError::UnsupportedLanguage, not panic.
DETERMINISTIC_CHECK: cargo check; grep TreeSitterAstEvaluator in ast_adapter.rs; DkgQuerierAdapter in dkg_adapter.rs; 0 PassAstEvaluator in cli/main.rs; 0 EmptyDkgQuerier; cargo test elai-knowledge 0 new failures; cargo test elai-rules 0 new failures; >=1 AstPattern rule in data/rules/ evaluates.

### 3J.13 VERIFY + MEASURE (E2B)
PRE_GATE: 3J.1-3J.12 + 3J.WIRE.* complete (or DEFER documented).
TASK: cargo check --all-features 0; cargo test --workspace 0. vLLM benchmarks on Gemma4 E2B RTX 3060:
- HTTP roundtrip p50/p95/p99
- DoLa latency overhead
- DoLa factuality delta (improve/neutral/degrade)
- Steering apply latency + effectiveness
- TurboQuant codebook generation + roundtrip MSE
- Model swap with vs without pre-loading
Feature gate: non-vllm build still compiles + tests pass. Compile time impact. Record .elai_cc/metrics/phase_3J.toml with test_model="gemma4-e2b", vllm_latency_p50/p95/p99_ms, dola_overhead_pct, dola_factuality_delta, turboquant_decision, turboquant_roundtrip_mse, milp_decision, steering_vector_count, steering_vectors_effective, multiplexing_enabled, ssb_action_type_count, t1/t2/t3_hook_latency_median_ms, compile_time_delta_sec.
DETERMINISTIC_CHECK: cargo test 0; metrics file; deferred docs in workflow/reasoning/.
ACCEPTANCE: zero 0-3I regressions; E2B metrics; deferred documented + re-eval criteria; feature gates verified.

### 3J.14 WIRE.E2E — END-TO-END LOCAL PIPELINE
PURPOSE: `elai.bat` starts full Phase 0-3 pipeline. Primary backend: custom sidecar (DoLa + TurboQuant + NF4). Fallback: OllamaProvider (gemma4:e2b). Knowledge: QueryPipeline wired into prompt. Release: cargo build --release elai.exe.

PRE_GATE: 3J.13; sidecar server.py functional (tested 3J.COMPOSE); OllamaProvider in providers/src/providers/ollama.rs; QueryPipeline in knowledge/src/pipeline.rs; Bm25Index, CatRag, AragOrchestrator, CrossEncoderReranker compile.

TASK:
**A. OLLAMA FALLBACK in local mode**: BackendMode::LocalWithFallback or fallback_url in BackendConfig. Sidecar health fail on startup OR mid-session HTTP error: log warning "sidecar unavailable, falling back to ollama"; switch base_url to ollama (default http://localhost:11434); OllamaProvider (no DoLa/TQ — plain); on next turn re-probe, switch back if healthy. DEGRADED prefix "[degraded] using ollama fallback". Both down: clear error "No local inference backend available." Wire run_local_repl() (sidecar fail -> fallback probe); RuntimeClient (HTTP errors -> swap). Config: [backend] fallback_url = "http://localhost:11434".

**B. KNOWLEDGE RETRIEVAL in prompt assembly**: Startup LiveCli::new_local/new:
1. Check BM25 index at session/project path. If not: Bm25Index::index_directory current cwd. If yes: open.
2. Create QueryPipeline with Bm25Index + CatRag + AragOrchestrator. CrossEncoderReranker optional (skip if ONNX missing).
3. Store Optional field on LiveCli/RuntimeClient.

Before each LLM call (near ContextBudget::compute):
1. Extract query from user's latest message
2. QueryPipeline::query(user_message) -> top-K
3. Format results as code context (file path, line range, snippet)
4. Inject into ContextBudget Slot 4 CodeContext or append to system prompt
5. Respect budget: truncate by score if exceed

None = skip retrieval. BM25 update: after file_write/file_edit, Bm25Index::update_file() on changed file (fresh within session). DKG: if dkg.db exists, use DkgEngine for PageRank. Otherwise skip (graceful degradation).

**C. RELEASE BINARY**: cargo build --release -p elai-cli. elai.bat starts to REPL; message -> sidecar -> response streams; kill sidecar -> next message fallback ollama -> works; /plan shows plan if plans/*.toml. Measure size.

**D. STARTUP SEQUENCE**:
1. elai.exe --local
2. Sidecar health -> start if not running -> fallback ollama if both fail
3. Open/create BM25 index
4. Create QueryPipeline (BM25 + CatRag + ARAG, optional cross-encoder)
5. Init Ssb, Fsm, SessionDag, RuleEngine, ContextBudget, PlanSkeleton
6. Startup banner: backend status, index status, session ID
7. REPL loop with full pipeline

DETERMINISTIC_CHECK: cargo build --release 0; target/release/elai.exe exists + --help; cargo test -p elai-cli 0 new failures; elai-providers 0 (fallback); elai-knowledge 0 (pipeline init); integration: sidecar down + ollama up -> fallback; BM25 first run created, second reused; grep fallback, QueryPipeline|query_pipeline in cli/main.rs.
ADVERSARIAL: BM25 >10K files = progress bar + async background (>30s = async); noisy retrieval = top-3 only, tune K later; fallback output quality worse than sidecar expected (log degradation, not fix here — Phase 4G addresses); release >100MB = strip symbols, [profile.release] strip=true; missing ONNX = skip stages via circuit breakers.
ACCEPTANCE: elai.bat to REPL <=60s excluding model load; sidecar primary + ollama fallback completions; retrieval triggers code context; ContextBudget allocates non-zero CodeContext; zero regressions; release binary exists; startup banner shows status.

### 3J.15 GEMMA4 31B VALIDATION (CPU OFFLOAD)
PRE_GATE: 3J.14; Gemma4 31B Q4 downloaded (~18GB); vLLM or llama.cpp CPU offload configured; alternative transformers+PyTorch device_map="cpu"/"auto".
TASK: Load 31B via CPU offload. ~0.5-2 tok/sec — slow but sufficient for artifact generation.

**A. STEERING VECTORS** (highest priority, most feasible): Compute >=2 contrast pairs (skepticism, completeness). Each pair ~2 short forward passes (~1-2 min CPU). Total ~10-20 min. Store .safetensors/.npz. 5 prompts per vector steered vs baseline. Compare 31B vs E2B vectors (different magnitudes, consistent semantic direction).

**B. DOLA EVALUATION**: Run DoLa on 31B with buckets [0,20), [20,40), [40,60) (3-bucket proper). 20 prompts factuality. At 0.5-2 tok/sec, ~1-5 min/prompt, ~30-90 min total.

**C. TURBOQUANT CODEBOOKS**: Verify E2B codebooks work for 31B (if same head dimension). If different: generate 31B-specific. Validate quant->dequant roundtrip on 31B KV cache within expected MSE.

**D. EAGLE-3 DRAFT MODEL TRAINING**: SKIP current hardware. Requires thousands of forward passes. At 0.5-2 tok/sec CPU = weeks-months. Not feasible. Document: needs 48GB GPU. Pre-work: define architecture (single decoder layer + LM head, reusing 31B vocab + embed). Save config. Ready to train when 48GB arrives.

**E. FULL PIPELINE TEST** (optional): 1 complete inference: prompt -> DoLa -> steering -> output on 31B. ~5-10 min. Worth once.

Record .elai_cc/metrics/phase_3J_31b.toml: test_model="gemma4-31b-q4"; inference_mode="cpu_offload"; inference_speed_tok_sec; steering_vectors_computed; steering_vector_effectiveness; dola_factuality_delta; dola_candidate_buckets="[0,20), [20,40), [40,60)"; turboquant_codebook_validated; eagle3_draft_architecture_defined; eagle3_draft_training_feasible=false; eagle3_deferred_reason="requires 48GB GPU"; full_pipeline_test_passed.

**F. 100B MODEL VALIDATION** (gated on 48GB): SKIP if 48GB not available. When arrives:
- Q3 100B (~37GB) + TurboQuant 3-bit KV cache (~7.5GB for 128K) = 44.5GB fits 48GB with ~3.5GB headroom
- Test SUBSTITUTE_MODEL escalation: 31B default executor; anti-spiral triggers (3x same error) -> swap to 100B. 31B unloads, 100B loads (~30-60s acceptable for escalation)
- Validate 100B Q3 at 128K context coherent. Compare 100B Q3 vs 31B Q4 on 10 complex prompts
- Record .elai_cc/metrics/phase_3J_100b.toml

DETERMINISTIC_CHECK: metrics file exists; 31B steering vectors saved; DoLa results documented; EAGLE-3 arch config saved.
ADVERSARIAL: CPU too slow = prioritize steering -> DoLa -> TurboQuant (EAGLE-3 skip); 31B CPU patterns may not match GPU = re-validate on 48GB later; vLLM CPU no layer hooks (R11) = transformers+PyTorch fallback.
ACCEPTANCE: >=2 steering vectors stored; DoLa 20 prompts logged; TurboQuant codebooks validated for 31B; EAGLE-3 architecture defined + training deferred with reason; artifacts ready for 48GB deploy.

EXIT_CRITERIA: VllmProvider + ModelPool integration; DoLa + steering validated on E2B (RTX 3060); DoLa + steering artifacts on 31B (CPU offload); TurboQuant FEASIBLE/DEFER + evidence + codebooks (E2B + 31B); GPU multiplexing with safety margin; predictive pre-loading on FSM; workload profiling every boundary; MILP ADOPT/DROP + heuristic always; MCP bridge + error recovery + security; tiered hooks (T1<1ms, T2<10ms, T3<5ms); AppContainer Windows + documented limits; 17 action_types typed; zero 0-3I regressions; EAGLE-3 architecture defined for 31B (training deferred); all 31B artifacts (steering, codebooks, DoLa) ready; all deferred documented with re-evaluation criteria.

---

## PHASE 3K: RAG PRODUCTION HARDENING
Prerequisites: Phase 1E (CatRAG + A-RAG), 1G (cross-encoder reranker), 3J (sidecar for HyDE 3K.3 only). Upgrade knowledge/ pipeline from prototype to production. Better retrieval = less context = smaller prompts = local LLMs handle the work.

Current state (Phases 1A-1I):
- DKG (1A): tree-sitter AST -> SQLite graph (Rust/Python/TS/Go)
- BM25 (1B): tantivy, 50-line window fallback
- Vectors (1C): dual ONNX (GraphCodeBERT 768d + Granite 384d), SQLite cache
- PageRank (1D): token-budget-fitted repo maps
- CatRAG (1E): query-aware graph traversal overlay
- A-RAG (1E): hierarchical assembly
- Reranker (1G): ms-marco-MiniLM-L12-v2, CPU, <50ms
- SPLADE (1H): evaluated
- Resilience (1I): circuit breakers, SearchMode cascade (HYBRID->SPARSE_ONLY->BM25_ONLY)

Missing: AST-aware chunking; query expansion; HyDE; quality-tier routing; recall/MRR metrics.

### 3K.1 ADAPTIVE CHUNKING
PRE_GATE: 1B (tantivy); 1A (DKG grammars).
TASK: Replace 50-line fixed window with AST-aware splitting. Reuse LanguageGrammar from DKG.
- Code files: split at function/impl/struct/enum/mod (Rust), def/class (Python), function/class (TS/Go). Each AST node = one BM25 chunk.
- Non-code: heading boundaries (markdown) or blank-line paragraphs.
- Fallback: AST parse fail = 50-line windows.
- Chunk size: 200-800 tokens. Overlap 50-token. Metadata: file_path, start_line, end_line, chunk_type (function/impl/struct/class/module/paragraph/heading).
- Tantivy schema extension: chunk_type + line range. Migration on format version change.
DETERMINISTIC_CHECK: boundaries at fn/impl/def/class; no chunk >1000 tokens; overlap between adjacent; parse fail = 50-line fallback.
ACCEPTANCE: code chunks at fn/class for .rs + .py; index rebuilt no loss; BM25 results include chunk_type. TOUCHES: knowledge/src/bm25.rs, knowledge/src/dkg.rs.

### 3K.2 QUERY EXPANSION
PRE_GATE: 3K.1.
TASK: Multi-query retrieval: 2-3 rephrasings. Template-based (no LLM):
- Keyword extraction: strip question words, keep nouns/verbs
- Code symbol expansion: CamelCase/snake_case variants
- Synonym expansion: "error" -> "failure", "exception"
Run BM25+vector per expansion, merge, dedupe by chunk ID, pass to cross-encoder. Skip <5 words.
DETERMINISTIC_CHECK: 2-3 variants for >5 word queries; dedupe; short queries skip.
ACCEPTANCE: Recall@5 improved vs single-query baseline; no >100ms regression on simple. TOUCHES: knowledge/src/pipeline.rs.

### 3K.3 HyDE (HYPOTHETICAL DOCUMENT EMBEDDINGS)
PRE_GATE: 3K.2; inference sidecar (3J).
TASK: HyDE (arxiv 2212.10496):
1. User query -> "how does ELAI handle anti-spiral detection?"
2. Generate hypothetical via sidecar: "Anti-spiral detection works by tracking repeated failures using a content hash. If the same step fails 3 times..."
3. Embed hypothetical with existing vector models (1C)
4. Search with hypothetical embedding (better match)
5. Rerank against ORIGINAL query (not hypothesis)

Sidecar: SidecarClient from providers/. POST /v1/completions max_tokens=128 temperature=0.7. Prompt: "Write a short technical paragraph answering: {query}". Cache 5 min. Graceful degradation (circuit breaker pattern from Phase 1I).
DETERMINISTIC_CHECK: HyDE for complex; simple skip; failure = direct search no block.
ADVERSARIAL: wrong hypothesis = irrelevant vectors — reranker against original catches; 3s timeout on sidecar = degrade.
ACCEPTANCE: MRR improved on complex; zero latency regression on simple; sidecar unavail graceful. TOUCHES: knowledge/src/pipeline.rs, providers/src/sidecar_client.rs.

### 3K.4 QUERY TIER ROUTING
PRE_GATE: 3K.1-3K.3.
TASK: Tiers:
- **FAST**: BM25 only, no vector, no reranker, no HyDE, no expansion. Target <20ms. Agent context passing.
- **STANDARD**: BM25 + vector hybrid (RRF), cross-encoder. No HyDE. Target <200ms. Default for code search.
- **DEEP**: Full: expansion + HyDE + hybrid + reranker. Target <5s. Research, debugging, architecture.

Caller specifies OR auto:
- <5 words -> FAST
- 5-15 words + code keywords -> STANDARD
- >15 words + question form + debugging/architecture keywords -> DEEP

QueryTier enum. QueryPipeline::search() optional tier (default AUTO, backwards compatible). Distinct from 1I SearchMode cascade (that's degradation on failure, this is intentional quality selection).
DETERMINISTIC_CHECK: FAST skips reranker + vector; DEEP includes HyDE; AUTO classifies short factual FAST/STANDARD.
ACCEPTANCE: 3 tiers measurably different latency; quality DEEP>=STANDARD>=FAST; backwards compatible. TOUCHES: knowledge/src/pipeline.rs, knowledge/src/lib.rs (QueryTier).

### 3K.5 RETRIEVAL QUALITY BENCHMARK
PRE_GATE: 3K.1-3K.4.
TASK: 30-query evaluation set (10 simple, 10 code-specific, 10 complex). Metrics: Recall@5, Recall@10, MRR (1/rank), latency p50+p95 per tier. Record .elai_cc/metrics/phase_3K_rag.toml. Regression gate: any metric >5% regression requires fix.
DETERMINISTIC_CHECK: file with 30 queries; no >5% regression without explanation.
ACCEPTANCE: Recall@5 >= baseline; MRR improved complex; tier budgets (FAST<20ms, STANDARD<200ms, DEEP<5s). TOUCHES: metrics file, knowledge/src/ harness.

### 3K.6 A-RAG KEYWORD SCORING + PROGRESSIVE DISCLOSURE
PRE_GATE: 3K.5.
TASK: Fix A-RAG keyword_search (stub):
- Score_kw(chunk, keywords) = SUM(count(k, text) * len(k))
- count case-insensitive frequency; len char length. Longer keywords weighted higher (more specific).

Fix A-RAG semantic_search (stub):
- Use dual-vector (1C). 3x oversampling: retrieve top_k*3, group by parent chunk, chunk score = MAX similarity among matched.

Progressive disclosure:
- Search tools return SNIPPETS (matched lines/sentences), not full chunks
- ContextTracker already prevents re-reading (arag.rs L69-75)
- Add snippet extraction in keyword + semantic results
- ChunkRead returns full text only when explicitly requested

Measure tokens before (full chunks) vs after (snippets + selective).
DETERMINISTIC_CHECK: cargo test arag pass; non-empty keyword_search + semantic_search; snippet field in results.
ADVERSARIAL: no normalization -> long files dominate (test: 10-line with 3 hits > 500-line with 3 hits); 3x oversampling + top_k=20 = 60 candidates, <100ms.
ACCEPTANCE: real results (not empty stubs); >=50% token reduction; Recall@5 >= baseline; contract tests arag_contract.rs pass. TOUCHES: knowledge/src/arag.rs, knowledge/src/pipeline.rs.

### 3K.7 CATRAG SYMBOLIC ANCHORING + KEY-FACT ENHANCEMENT
PRE_GATE: 3K.6.
TASK: Symbolic Anchoring (CatRAG Innovation 1): Extract entities (rule-based):
- Split on whitespace; identify CamelCase + UPPER_SNAKE as candidate entities
- Match against DKG entity nodes
- For each match, weak PPR reset weight: reset_weight(e) = epsilon * |passages_containing(e)|^(-1)
- epsilon = 0.2 (CatRagConfig catrag.rs L91)

Key-Fact Passage Enhancement (Innovation 3, pure algorithmic, no LLM):
- For each context edge entity u -> passage p: w_hat = w * (1 + beta * I(u,p in verified_triples))
- beta = 2.5 (catrag.rs L93)
- verified_triples = DKG edges high confidence

Innovation 2 (Dynamic Edge Weighting, LLM-scored) DEFERRED: too expensive for local models during code retrieval. Evaluate after 4D web research provides natural LLM-in-loop context.
DETERMINISTIC_CHECK: cargo test catrag pass; grep symbolic_anchor, key_fact_enhance.
ADVERSARIAL: "how does the system work" -> 0 entities (no CamelCase/identifiers); beta=2.5 non-verified passages still top-5 with high PPR.
ACCEPTANCE: weak seeds for identifier queries; boost verified edges; Recall@5 >= 3K.6; existing CatRAG contract tests pass. TOUCHES: knowledge/src/catrag.rs.

### 3K.8 POST-IMPROVEMENT BENCHMARK
PRE_GATE: 3K.6 + 3K.7.
TASK: Re-run 3K.5 30 queries. Compare: Recall@5 delta, Recall@10 delta, MRR delta, tokens-per-query delta (progressive disclosure), latency per tier. Record .elai_cc/metrics/phase_3K_rag_improved.toml. Any >5% regression investigate+fix.
DETERMINISTIC_CHECK: improved metrics alongside baseline; no >5% regression.
ACCEPTANCE: tokens-per-query >=30% reduced (disclosure); Recall@5 improved or unchanged; latency within tier budgets.

EXIT_CRITERIA: AST-aware chunking; query expansion 2-3 variants; HyDE for complex (graceful degradation); 3-tier routing; 30-query benchmark; A-RAG keyword+semantic real (not stubs); progressive disclosure >=30% savings; CatRAG symbolic anchoring + key-fact enhancement; CatRAG Innovation 2 documented as deferred; no >5% regressions vs 3K.5 baseline.

---

## PHASE 3 INTERFACE MAP

### Phase 3A (Full FSM)
- **Public Rust API**: `Fsm` trait (get_state, transition, get_active_tools, get_active_rules, token_budget_remaining); `FsmState` enum (9 variants); `EvidenceGate` trait; `InteractiveGate` trait; `GateDecision` enum (APPROVE, REJECT, MODIFY, REDIRECT); `RiskLevel` enum (LOW, MEDIUM, HIGH); `ReplanRequest` (MINOR, MAJOR).
- **SSB action_types exposed**: phase_transition (written by FSM on transition after 3B integration).
- **FSM transitions added**: all 12 (IDLE<->PLANNING, IDLE->CONFIGURING, PLANNING<->FEASIBILITY_PROBE, PLANNING->TESTING_FIRST, FEASIBILITY_PROBE->TESTING_FIRST, TESTING_FIRST->EXECUTING, EXECUTING->AUDITING/PLANNING, AUDITING->MEMORIZING/EXECUTING/PLANNING, MEMORIZING->PLANNING/COMPLETE, COMPLETE->IDLE, CONFIGURING->IDLE).
- **Downstream integration points**: 3B (phase_transition writes); 3C (phase-aware tool filter at execute level + ReplanRequest); 3D (superstep boundary calls FSM transition); 3E (token_budget_remaining); 3F (get_active_rules by phase); 3G (phase context loading); 3H (FSM state in snapshots); 3J.WIRE.FOUNDATION (Fsm wired in CLI); 4G (AUTONOMOUS risk auto-advance extends LOW auto).

### Phase 3B (SSB)
- **Public Rust API**: `Ssb`, `SsbConfig`, `SsbWriter`, `SsbReader` trait, `SsbEntry` (payload: Arc<[u8]>), `SsbFileEntry`, `SsbActionType` enum (17 variants), `SsbPayload` enum (17 typed structs), `SsbTraceable` trait (in orchestrator/, impl by reasoning/tools via orchestrator callback), `SsbPragmaApplier` (r2d2::CustomizeConnection).
- **SSB action_types exposed**: all 17 — file_write, file_read, tool_result, test_outcome, finding, claim, checkpoint, model_swap, phase_transition, knowledge_degradation, isolation_metric, plan_update, replan_request, stub_reference, reasoning_trace, phase_complete, memory_monitoring.
- **FSM transitions added**: none (pure infrastructure).
- **Downstream integration points**: 1C (vector cache invalidation from file_write events); 3C (checkpoint writes + reasoning_trace + replan_request + ReasoningTracePayload round-trip); 3D (BSP save_checkpoint, memory_monitoring, ephemeral_clear checkpoint subtype for replay, conflict_detection via rarray); 3E (SSB entry filtering by file_overlap_score); 3F (T1 hook on every write, self-discovery via pattern hash); 3G (plan_update + stub_reference); 3H (stub recovery via SSB seq); 3I (DST reference model SSB row counts); 3J.WIRE.FOUNDATION (Ssb wired via SessionTracer); 3J.12 (all 17 action_types implemented); 4A (IPC Tier C inherits SsbTraceable pattern).

### Phase 3C (Agent System)
- **Public Rust API**: `AgentPool` trait (spawn, spawn_parallel stub, communicate stub), `AgentHandle` (agent_id: Uuid, role, status, active_files, grc_config, scratch_dir), `AgentStatus` enum (6 variants), `AgentRole` enum (8 variants incl Designer), `AgentDeathClassification` enum (CLEAN_EXIT, CRASH, TIMEOUT, BUDGET_EXCEEDED), `SupervisionPolicy`, `EscalationAction` enum (Escalate, SubstituteModel), `AgentEvent`, `AgentFailure`, `AgentSandbox` trait (wired in 3J.11 Windows, 4A Linux).
- **SSB action_types exposed**: checkpoint, reasoning_trace (via orchestrator callback from agent GRC steps), replan_request, isolation_metric data source (aggregation at phase boundary — 3A/3D), spiral_detected (logged as finding or new type).
- **FSM transitions added**: none directly (agents observe FSM via handle); 3C supplies per-agent data for phase transition decisions.
- **Downstream integration points**: 3D (spawn_parallel upgrade to true parallel with VRAM-aware scheduling); 3E (AgentHandle.active_files for file_overlap_score); 3F (tool dispatch through orchestrator enables PreToolUse rule evaluation); 3H (agent checkpoints + SessionDag snapshots); 3I DST (AgentSpawn/AgentCrash/AgentCleanup FsmEvent variants); 3J.11 (AppContainer integrates AgentSandbox); 4A (Tier C process isolation wraps AgentHandle around IPC, same interface); 4G (SubstituteModel EscalationAction gated in Phase 3 enabled in AUTONOMOUS); 4H (ReasoningTraceEntry fuels self-modification).

### Phase 3D (BSP + Channels)
- **Public Rust API**: `ChannelRegistry` trait (read/write/snapshot_for_scheduling/clear_ephemeral/force_clear_untracked/deep_size_untracked), `ChannelId<T>` typed key, `ChannelState` enum (3 variants — no UntrackedValue), `ChannelValue`, `ChannelWrite`, `ChannelError`, `ChannelSnapshot`; 4 channel types: `LastValue<T>`, `Topic<T>(accumulate: bool)`, `EphemeralValue<T>`, `UntrackedValue<T: DeepSizeOf>`; `DeepSizeOf` trait bound; `BspEngine`/superstep loop; `NodeSpec` with is_deferred; spawn_parallel (upgraded).
- **SSB action_types exposed**: memory_monitoring (NEW, separate from isolation_metric); checkpoint subtype 'ephemeral_clear' payload { ephemeral_clears: Vec<ChannelId> } for replay.
- **FSM transitions added**: none directly (BSP is inside EXECUTING superstep loop; FSM transitions happen at phase boundary after BSP loop terminates).
- **Downstream integration points**: 3E (ChannelSnapshot feeds SSB entries filtering by file_overlap in slot 6, channels write_count for EphemeralValue budget); 3F (contract test for ChannelRegistry consumers); 3H (checkpoint roundtrip and crash recovery via SSB replay); 3I (SuperstepBoundary as composite FsmEvent, UntrackedValue never checkpointed invariant); 3J.7 (workload_profile at phase_complete after BSP); 4A (ChannelRegistry contracts carry to IPC).

### Phase 3E (Context Budget Allocator)
- **Public Rust API**: `ContextSlot` enum (8 variants); `SlotAllocation`; `ContextBudget` (model_max_context, slots, cache_breakpoints, total_allocated, total_used, adaptive_level); `CacheBreakpoint` (after_slot, prefix_hash: u64, valid); `AdaptiveLevel` enum (5 variants: Normal, FilteredByRelevance, ReducedGranularity, ReducedScope, Terminal); `ContextBudget::compute`, `ContextBudget::new`; file_overlap_score; adaptive reduction 3 attempts; `total_allocated_tokens()` method (required by 3H per R-3H-8).
- **SSB action_types exposed**: context_reduction (adaptive level logged with before/after token counts, slots_affected).
- **FSM transitions added**: none.
- **Downstream integration points**: 3G (Slot 3 PhaseSpec allocator for skeleton + detail); 3H (trim budget target = model_window - allocated_slots - output_reserve); 3J.PIPELINE (ContextBudgetAllocator before dispatch); 3J.WIRE.BUDGET (real fixed_slot_sizes); 3J.14 (Slot 4 CodeContext for retrieval); 4G SPLIT_TASK (PipelineSpec decomposition on Terminal AUTONOMOUS).

### Phase 3F (Rules Engine)
- **Public Rust API**: `RuleEngine` trait (evaluate, get_active_rules, compile_predicate); `AstEvaluator` trait (evaluate_query, validate_query) implemented by knowledge/; `DkgQuerier` trait (query_reachability, validate_params) implemented by knowledge/; `Predicate` enum (5 types: AST_PATTERN, GREP_PATTERN, SCHEMA_REQUIRED, METRIC_GATE, COMPOSITE — GRAPH_REACHABILITY via DkgQuerier predicate type appears in 3J.WIRE.STUBS); `Verdict` enum (ALLOW, WARN, BLOCK, ESCALATE(EscalationTarget)); `EscalationTarget` enum (Auditor, Human, NextTierRule(RuleId)); `CompiledPredicate`, `PredicateError`; `RuleLifecycleState` enum (PROPOSED, CANDIDATE, SHADOW, ACTIVE, RETIRED); `RuleMetrics`; `ViolationPattern` (3-axis scoring); `SupervisionPolicy.on_max_restarts`.
- **SSB action_types exposed**: finding (BLOCK/WARN from rule evaluation); rule_transition (possibly via plan_update or dedicated, not enumerated in 17).
- **FSM transitions added**: none directly; interacts with FSM via phase_mask activation filtering.
- **Downstream integration points**: 3J.10 (Tiered analysis hooks T1 uses RuleEngine on every SSB write); 3J.WIRE.RULES (DefaultRuleEngine into PreToolUse, seed data loading, AstEvaluator + DkgQuerier stubs); 3J.WIRE.STUBS (TreeSitterAstEvaluator + DkgQuerierAdapter in knowledge/); 3I (counterfactual testing in simulation model).

### Phase 3G (Hierarchical Plan Management)
- **Public Rust API**: `PlanSkeleton` (pipeline_id, phases: IndexMap<PhaseId, PhaseEntry>, global_constraints, version, created_at, updated_at); `PhaseEntry` (phase_id, goal, inputs, outputs, constraints, depends_on, depended_by, version, output_type, visual_check); `ContractRef` (name, type_signature, invariants); `PhaseDetail` (phase_id, skeleton_version, contracts, implemented_contracts, declared_interfaces, content, token_count); `ExtractedContracts` (input_contracts, output_contracts, shared_constraints); `OutputType`; `PlanSkeleton::write_to_file`/`read_from_file`/`extract_contracts`/`load_phase_context`/`propagate_changes`/`split_phase(phase_id, split_points, preserve_contracts: bool)`; `coherence_check` module (3 checks).
- **SSB action_types exposed**: plan_update, stub_reference.
- **FSM transitions added**: none.
- **Downstream integration points**: 3H (PlanSkeleton persisted in snapshot.findings); 3J.WIRE.PLAN (PlanSkeleton in LiveCli, /plan command, Slot 2 PhaseSpec); 4G SPLIT_TASK (reuses split_phase with preserve_contracts).

### Phase 3H (Session Handoff)
- **Public Rust API**: `SessionDag` (db, session_id, active_branches); `Snapshot`; `TrimmedMessage` (seq, role, content, is_stubbed, stub_ssb_seq); `Stub` (original_type, original_tokens, summary_hash: [u8;32] blake3, ssb_seq); `BranchRef`; `MergeResult` enum (Winner, Conflict); `SessionDag::new`/`open`/`save_snapshot`/`load_snapshot`/`latest_snapshot`/`parent_chain`/`trim_context(messages, config)`/`create_branch`/`list_branches`/`merge_branch`/`close_branch`/`recover_stubs`/`recover_single_stub`/`create_handoff`/`load_handoff`; `TrimConfig` (stub_threshold, model_context_window, allocated_slots_tokens, output_reserve); `MessageRole` enum.
- **SSB action_types exposed**: uses checkpoint with branch_merge payload (per SPEC AMENDMENT #2 decision); stub_reference (from 3G) may overlap with stub semantics.
- **FSM transitions added**: none (handoff is data management, not FSM transition).
- **Downstream integration points**: 3J.WIRE.SESSION (SessionDag replacing raw jsonl, /resume, /session fork, stub recovery); /compact still available as manual override; 4A (handoff state survives process restart for IPC isolation).

### Phase 3I (DST)
- **Public Rust API**: `SimulatorRuntime` (wraps SeededClock+SeededRng+madsim); `FsmReferenceModel` implements `ReferenceStateMachine` (from proptest-state-machine v0.8+); `FsmEvent` enum (10 variants: PhaseTransition, AgentSpawn, AgentCrash, AgentCleanup, ToolCall, FileWrite, GrcStep, SSBWrite, ConfigChange, SuperstepBoundary); `StateMachineTest` impl; `ReferenceState`; VCR cassette format (blake2s hash + msgpack response); TraceOps layer; custom SQLite VFS (if adopted post-spike).
- **SSB action_types exposed**: none new; DST reads all 17.
- **FSM transitions added**: none; DST validates them.
- **Downstream integration points**: 4A (madsim IPC tests prep for Phase 4A process isolation); 4G (AUTONOMOUS auto-advance must pass same determinism); every future phase (regression testing via reference model).

### Phase 3J (vLLM + Providers + Plugins + Orchestrator Hardening)
- **Public Rust API**: `VllmProvider` (impl LlmProvider, cfg(feature="vllm")); `VllmConfig`; `InferenceBackend` enum (Ollama, Vllm); `DolaDecoder`, `DolaConfig` (enabled, alpha, candidate_buckets, repetition_penalty); `TurboQuantConfig` + `TurboQuantizer` (if FEASIBLE); `SteeringVectorEngine`, `SteeringConfig`, `SteeringVectorSpec`; `MultiplexingPolicy` (safety_margin_bytes, max_colocated, min_model_size_for_dedicated_gpu); `PreloadPolicy` (FsmState -> Vec<ModelId>); `SloTier` enum (BEST, GOOD, FAIR, BASIC); `WorkloadProfile`; `GreedyModelPlacer` (always present), `MilpModelPlacer` (if ADOPTED); `McpBridge` trait, `McpConnection`, `McpToolSpec`; `T1Hook`, `T2Hook`, `T3Hook`, `TieredAnalysis`; `AppContainerConfig`, `AgentSandbox` trait (Windows impl in 3J.11); `InferenceDispatcher` trait, `DispatchContext`, `BackendMode`, `BackendConfig`, `DispatchError`; unified `RuntimeClient` + `ToolPolicy` + `BudgetPolicy`; `CliInteractiveGate`; `TreeSitterAstEvaluator` (impl AstEvaluator); `DkgQuerierAdapter` (impl DkgQuerier).
- **SSB action_types exposed**: workload_profile (NEW), all 17 types fully implemented with typed structs at write-time validation (3J.12).
- **FSM transitions added**: none; 3J.WIRE.FOUNDATION wires real Fsm into CLI (no new transitions, replaces read_persisted_state with owned instance).
- **Downstream integration points**: 4A (vLLM + ModelPool multiplexing persist; AppContainer replaced by Linux clone+seccomp); 4D web research (QueryPipeline from 3J.14 is shared infrastructure); 4G (BackendMode="hybrid" + role_override supports AUTONOMOUS escalation; SubstituteModel enabled); 4H self-modification (ReasoningTraceEntry + violation_patterns fuel).

### Phase 3K (RAG Production Hardening)
- **Public Rust API**: `QueryTier` enum (FAST, STANDARD, DEEP, AUTO); `QueryPipeline::search(tier: Option<QueryTier>)` (AUTO default, backwards compatible); AST-aware chunking via `LanguageGrammar` reuse (from 1A); `chunk_type` metadata enum (function, impl, struct, class, module, paragraph, heading); query expansion (keyword/symbol/synonym, no LLM); HyDE stage; A-RAG `Score_kw` + semantic `3x oversampling` with MAX per parent chunk; progressive disclosure (snippets vs full chunks); `ContextTracker` (existing, prevents re-reading); CatRAG symbolic anchoring (epsilon=0.2, CamelCase+UPPER_SNAKE entity extraction); CatRAG key-fact enhancement (beta=2.5, I(u,p in verified_triples)).
- **SSB action_types exposed**: none new; retrieval results injected into Slot 4 CodeContext in ContextBudget.
- **FSM transitions added**: none.
- **Downstream integration points**: 3J.14 (QueryPipeline wired into prompt assembly with BM25 index update after file_write/file_edit); future web research phase (CatRAG Innovation 2 Dynamic Edge Weighting enabled when natural LLM-in-loop context available from 4D).

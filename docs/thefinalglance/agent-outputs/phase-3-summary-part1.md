# Phase 3 Summary — Part 1 (3A, 3B, 3C, 3D)

## Index (Parts 1-3)

- Part 1: 3A FSM, 3B SSB, 3C Agents, 3D BSP + Channels
- Part 2: 3E Context Budget, 3F Rules Engine, 3G Hierarchical Plans, 3H Session Handoff, 3I DST
- Part 3: 3J vLLM/Providers/WIRE/CLI-Orchestrator (15 steps + 7 WIRE sub-steps), 3K RAG Hardening

Files read (bytes):
- phase-3A.txt 22625
- phase-3B.txt 30997
- phase-3C.txt 33920
- phase-3D.txt 28121
- phase-3E.txt 20014
- phase-3F.txt 27252
- phase-3G.txt 28390
- phase-3H.txt 33410
- phase-3I.txt 29892
- phase-3J.txt 93324
- phase-3K.txt 17348

Status legend: DONE / IMPROVED / PROPOSAL / STUB / IN PROGRESS. All Phase 3 files are PROPOSAL/IMPROVED specs (implementation plans), none marked DONE.

---

## 3A FULL FSM (INTERACTIVE MODE) — Status: IMPROVED

Goal: full 9-state FSM with phase-aware tool/rule loading, 7 evidence gates and INTERACTIVE (HITL) gating at every transition.

Prereqs: Phase 0–2 complete (minimal FSM, knowledge, GRC).
Deliverables: 5 (3A.1–3A.5).

### 3A.1 FSM STATE MACHINE
PRE_GATE: Phase 0 minimal FSM + Phase 2 GRC tests green.
TASK:
- `FsmState` enum: IDLE, PLANNING, FEASIBILITY_PROBE, TESTING_FIRST, EXECUTING, AUDITING, MEMORIZING, COMPLETE, CONFIGURING.
- Transition matrix (valid transitions):
  - IDLE -> PLANNING | CONFIGURING
  - PLANNING -> FEASIBILITY_PROBE | TESTING_FIRST (PLANNING->TESTING_FIRST is 3A shortcut; FEASIBILITY_PROBE full impl deferred to 4F)
  - FEASIBILITY_PROBE -> PLANNING (revise) | TESTING_FIRST (proceed)
  - TESTING_FIRST -> EXECUTING
  - EXECUTING -> AUDITING | PLANNING (ReplanRequest)
  - AUDITING -> MEMORIZING | EXECUTING (fix BLOCKs, SKILL.md L141 — SPEC_UPDATE_NEEDED) | PLANNING (dead-end, SKILL.md L115 — SPEC_UPDATE_NEEDED)
  - MEMORIZING -> PLANNING (next phase) | COMPLETE
  - COMPLETE -> IDLE
  - CONFIGURING -> IDLE
- Each transition carries: source, target, phase label, timestamp, `Vec<EvidenceArtifact>`.
- `token_budget_remaining() -> usize` on `Fsm` trait.
- Persist to `.elai_cc/sessions/{sid}/.fsm-state.msgpack`.
- `ReplanRequest` MINOR (auditor fast-track) / MAJOR (human gate). Max 1 per agent per phase.
DETERMINISTIC_CHECK: cargo test -p orchestrator --lib fsm; grep ">= 9 `FsmState::`"; grep `fn token_budget_remaining`; grep `ReplanRequest`.
ADVERSARIAL: invalid jumps (IDLE->AUDITING etc.) must return Err(InvalidTransition); corrupt .fsm-state.msgpack -> Err(CorruptState); ReplanRequest limit (2nd attempt -> Err(ReplanLimitExceeded)); concurrent transitions -> exactly one wins.
ACCEPTANCE: 9 states serializable, valid transitions succeed, persistence/crash recovery, token_budget_remaining compiles.
GROUNDING: 9 states VERIFIED orchestrator.txt §2 + technical-analysis-v3.txt §5; AUDITING->EXECUTING/PLANNING PARTIAL (SKILL.md); ReplanRequest VERIFIED orchestrator.txt §13.

### 3A.2 PHASE-AWARE TOOL/RULE LOADING
PRE_GATE: 3A.1 compiles; `tools::ToolRegistry` + `rules::RuleEngine` traits exist.
TASK: `get_active_tools(state) -> Vec<ToolId>` per filter table:
- PLANNING: rules + knowledge results + plan template. NO edit/bash.
- FEASIBILITY_PROBE: stub write (<50 lines) + bash + web search. NO full edit/git.
- TESTING_FIRST: test framework + plan + rules. NO edit/bash.
- EXECUTING: edit tools + phase spec + code context. NO planning tools.
- AUDITING: validation tools + test runner + code. NO edit/planning.
- MEMORIZING: memory-bank write tools only.
- CONFIGURING: config paths only (elai_cc.toml, workflow config). NO code edit/bash.
- IDLE: all tools.
- COMPLETE: read-only (UNGROUNDED — assumed).
`get_active_rules(state) -> Vec<RuleId>`.
Enforcement at `ToolRegistry::execute()`: blocked -> `Err(ToolBlockedByState)`, never silent.
DETERMINISTIC_CHECK: cargo test -p orchestrator --lib tool_filter.
ADVERSARIAL: direct call bypasses must fail with typed error; IDLE state must not block anything.
GROUNDING: VERIFIED PLANNING/EXECUTING/AUDITING restrictions; COMPLETE+IDLE UNGROUNDED.

### 3A.3 EVIDENCE GATE PROTOCOL
TASK: 7 gates implementing `EvidenceGate` trait (`fn check(&self, context: &TransitionContext) -> Result<(), EvidenceError>`):
1. IDLE -> PLANNING: knowledge readiness (READY|DEGRADED ok, UNAVAILABLE blocks).
2. PLANNING -> TESTING_FIRST: zero UNGROUNDED claims in grounding_sweep_report.
3. TESTING_FIRST -> EXECUTING: tests fail for right reasons (not syntax).
4. EXECUTING -> AUDITING: >=1 test flipped FAIL->PASS.
5. EXECUTING -> PLANNING (fast-track): ReplanRequest(MINOR) + auditor validation.
6. AUDITING -> MEMORIZING: zero BLOCK findings remaining.
7. MEMORIZING -> next: DKG reflects new code, changelog entry.
`EvidenceError { gate_name, missing_artifacts: Vec<String>, suggestion: String }`.
DAIRA adaptive context: tools with variable output implement scope reduction (run -> check size -> if > budget*0.3 reduce -> retry, max 3 attempts).
NOTE: FEASIBILITY_PROBE transitions ungated in 3A (deferred to 4F).
GROUNDING: 7 gates VERIFIED orchestrator.txt §13 L540-554; DAIRA VERIFIED §13 L556-558; EvidenceGate trait UNGROUNDED (assumed for testability).

### 3A.4 INTERACTIVE GATE
TASK: `InteractiveGate` trait:
```
fn present_gate(&self, evidence: &EvidenceSummary) -> GateDecision
```
4 decisions: APPROVE, REJECT (no state change), MODIFY (re-eval then advance), REDIRECT (different target).
Trait enables mocking (MockInteractiveGate). No timeout in INTERACTIVE mode; AUTONOMOUS auto-advance deferred to 4G.
DETERMINISTIC_CHECK: grep `GateDecision::` returns >= 4.
ADVERSARIAL: REJECT must preserve source state; MODIFY must re-run evidence gate; REDIRECT must be constrained to valid targets.
GROUNDING: 4 decisions VERIFIED architecture-overview §6 L201-204; trait + no-timeout UNGROUNDED.

### 3A.5 RISK-LEVEL CLASSIFICATION
TASK: `RiskLevel` enum LOW/MEDIUM/HIGH per transition. INTERACTIVE: LOW auto-advances, MEDIUM pauses, HIGH always pauses. Config overrides in `elai_cc.toml [fsm.risk_overrides]`.
Gate integrates: `gate.present_gate()` receives risk_level; LOW skips presentation.
GROUNDING: tiers VERIFIED architecture-overview §6 L214-218; risk overrides UNGROUNDED (inferred).

### Phase-level tests & targets
- Unit: all 9 states msgpack round-trip, 9x9 transition matrix (81 combos), per-state tool set tests, 7 gate tests with mock artifacts, 4 interactive-decision tests, 3 risk-tier tests, ReplanRequest limit.
- Property tests: random valid sequences reach IDLE/COMPLETE; invalid sequences never change state.
- Adversarial: corrupt msgpack, concurrent transitions, missing artifacts, invalid REDIRECT.
- Targets: <1ms per transition, <10ms per evidence gate, <0.1ms tool-filter lookup.

---

## 3B SSB (SHARED STATE BOARD) — Status: IMPROVED (audit fixes applied — 10 BLOCK/WARN resolutions)

Goal: inter-agent communication via SQLite WAL with single-writer guarantee; append-only within session; backbone for all agent state.

Single Ssb instance is permanently bound to one `(project_id, session_id)` pair.

### Risks
- R10 write throughput ceiling: 50K claim; 80K measured with WAL + synchronous=NORMAL + mmap. Agent rate 1-10/s = 50x headroom. Fallback: batch writes.
- R-3B-1 WAL checkpoint interference: `wal_autocheckpoint=1000`, manual checkpoint only between supersteps.
- R-3B-2 r2d2 idle closes WAL: disable `idle_timeout` and `max_lifetime`.
- R-3B-3 Schema migration: use `PRAGMA user_version`, migration codepath drops+recreates (session-scoped data loss acceptable).

### SCHEMA (rules.db / ssb.db not — this is ssb.db):
```
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
    'phase_complete','memory_monitoring'
  )),
  payload BLOB NOT NULL,
  created_at INTEGER NOT NULL
);
CREATE INDEX idx_ssb_project ON ssb(project_id);
CREATE INDEX idx_ssb_session ON ssb(session_id);
CREATE INDEX idx_ssb_superstep ON ssb(superstep);
CREATE INDEX idx_ssb_conflict ON ssb(project_id, superstep);
CREATE TABLE ssb_files (
  seq INTEGER NOT NULL REFERENCES ssb(seq),
  file_path TEXT NOT NULL
);
CREATE INDEX idx_ssb_files_path ON ssb_files(file_path);
CREATE INDEX idx_ssb_files_seq ON ssb_files(seq);
```

### Action types (17 canonical, CHECK-constrained):
file_write, file_read, tool_result, test_outcome, finding, claim, checkpoint, model_swap, phase_transition, knowledge_degradation, isolation_metric, plan_update, replan_request, stub_reference, reasoning_trace, phase_complete, memory_monitoring.
Note: orchestrator.txt §3 synopsis lists 9; 17 canonical from cross-referencing §7A, §7, §8, §9, knowledge.txt, technical-analysis-v3.txt Concern 2, §11.

### Typed payload structs (15+ — add 2 more for 17 total):
- FileWritePayload { path, size_bytes, hash }
- FileReadPayload { path }
- ToolResultPayload { tool_id, success, output_summary }
- TestOutcomePayload { test_name, passed, duration_ms }
- FindingPayload { severity, category, message, file_path, line }
- ClaimPayload { claim_text, grounding_source, verified }
- CheckpointPayload { agent_state_hash, grc_history_hash }
- ModelSwapPayload { from_model, to_model, reason }
- PhaseTransitionPayload { from_state, to_state, evidence_summary }
- KnowledgeDegradationPayload { stage, from_mode, to_mode }
- IsolationMetricPayload { metric_name, value }
- PlanUpdatePayload { plan_hash, phase, step }
- ReplanRequestPayload { reason, requesting_agent }
- StubReferencePayload { original_seq, stub_reason }
- ReasoningTracePayload { step_type, content, topic, grounding, conclusion }
- PhaseCompletePayload { phase_id, agent_id, fsm_state, evidence_artifacts, commit_hashes, test_outcomes }  — atomic durability marker, fsync BEFORE model unload
- MemoryMonitoringPayload { metric_name, value, threshold, action_taken }

### Conflict detection query (JOIN required — BLOCK 2 fix)
```
SELECT DISTINCT ssb.agent_id FROM ssb_files
JOIN ssb ON ssb_files.seq = ssb.seq
WHERE ssb_files.file_path IN (rarray(?1))
  AND ssb.project_id = ?2
  AND ssb.session_id != ?3
  AND ssb.superstep = ?4
```
Uses rusqlite `array` feature (rarray) for IN clause to preserve prepared-statement caching. Filters by project_id (BLOCK 5) and uses `idx_ssb_conflict` compound index.

### WAL configuration (pragma order matters):
```
PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;
PRAGMA wal_autocheckpoint = 1000;
PRAGMA busy_timeout = 5000;
PRAGMA cache_size = -64000;
PRAGMA foreign_keys = ON;
PRAGMA temp_store = MEMORY;
```
`foreign_keys` is per-connection — must be applied via r2d2 `CustomizeConnection` (`SsbPragmaApplier`) on every checkout.

### Rust types
- `SsbActionType` enum (17 variants, serde snake_case).
- `SsbEntry { seq, project_id, session_id, agent_id, superstep, action_type, payload: Arc<[u8]>, created_at }` — payload is **Arc<[u8]>**, not Vec<u8>, for O(1) broadcast clone (BLOCK 5 audit fix).
- `SsbFileEntry { seq, file_path }`.
- `SsbConfig { db_path, pool_size, pragmas }`.
- `SsbPragmaApplier` impl `r2d2::CustomizeConnection`.
- `Ssb::new(config, clock: Arc<dyn ClockProvider>) -> Result<Self>` — opens, migrates via `PRAGMA user_version`, applies WAL pragmas in order, creates r2d2 pool (size max(num_cpus*2,4), no idle_timeout, no max_lifetime), creates `tokio::mpsc::unbounded_channel` for write serialization, `tokio::broadcast` cap 1024 for events.
- `SsbWriter` (wraps mpsc Sender).
- `SsbReader` trait: query_by_session, query_by_file, query_by_type, query_by_superstep, query_conflict, last_seq.
- `Ssb::subscribe() -> broadcast::Receiver<SsbEntry>`.
- `Ssb::shutdown() -> Result<()>` (drain write loop).
- `SsbPayload` enum (typed per action_type, serialized to msgpack via rmp-serde for disk).
- `SsbTraceable` trait (defined in orchestrator/, impl'd for reasoning/tools types; trace writes initiated by orchestrator after IPC / in-process GRC callbacks — no circular deps).

### Deliverables (3 steps)
- 3B.1 Skeleton + WAL + migration. Deps: rusqlite (bundled + array features for Windows + rarray), r2d2_sqlite.
- 3B.2 Single-writer channel + write path. BEGIN IMMEDIATE (early write lock). ClockProvider injected for created_at (not SystemTime::now — core.txt §68-82). Broadcast only after COMMIT. IPC sync (WARN-6): SSB write MUST complete before ToolResult dispatch.
- 3B.3 Read pool + conflict detection + append-only enforcement. SsbWriter has no update/delete methods. Debug-build `update_hook` panics on UPDATE/DELETE.
- 3B.4 Typed action validation + trace records. SsbTraceable in orchestrator/, never reasoning/ or tools/ (avoids circular dep).

### Targets
- Write >10K inserts/sec.
- Read p99 <1ms query_by_file at 10K entries.
- Conflict detection <5ms for 100 paths against 100K entries.
- Broadcast overhead <1% with 3 subscribers.

### Audit fixes applied (from s-1427264)
- BLOCK 1: reasoning_trace as 15th; phase_complete (16th); memory_monitoring (17th).
- BLOCK 2: JOIN on ssb_files.seq = ssb.seq (direct column ref failed with OperationalError).
- BLOCK 3: ClockProvider (not TimeProvider, core.txt L68).
- BLOCK 4: project_id filter in conflict query.
- BLOCK 5: Arc<[u8]> payload.
- WARN 1-6: idx_ssb_conflict compound index, CustomizeConnection, shutdown drain, user_version migration, SsbTraceable placement, rarray().

---

## 3C AGENT SYSTEM — Status: IMPROVED

Goal: agent spawn/lifecycle/checkpoint/recovery/supervision/anti-spiral. Tier A (invisible walls) only — Tier B/C isolation deferred to 4A.

Foundation: claw-code `worker_boot.rs` (~1069 LOC) + `recovery_recipes.rs` (~630 LOC).

### Claw-code mapping
- WorkerStatus -> AgentStatus (rename; add Checkpointing + BudgetExceeded; drop TrustRequired).
- WorkerRegistry -> AgentPool (HashMap<Uuid, AgentHandle>; UUID v4 via EntropySource, not raw uuid crate).
- WorkerEvent/EventKind -> AgentEvent (expanded).
- WorkerFailure -> AgentFailure (expanded with death classification).
- `now_secs()` -> ClockProvider::now_monotonic().
- FailureScenario -> death classification.
- RecoveryRecipe -> SupervisionPolicy::on_failure.

### Types
- `AgentStatus` enum: Spawning, Ready, Running, Checkpointing, Finished, Failed.
- `AgentHandle { agent_id: Uuid, role: AgentRole, status: AgentStatus, active_files: Vec<PathBuf>, grc_config: AgentGrcConfig, scratch_dir: PathBuf }`.
- `AgentRole` enum (8 variants): Executor, Auditor, PromptEngineer, ContextCompiler, MemoryWriter, Researcher, MediaValidator, Designer. Designer reuses executor model (Gemma 4 31B) with separate AgentGrcConfig (role_prompt + GRC required_steps differ).
- `AgentPool` trait: `spawn(role, task) -> AgentHandle`, `spawn_parallel(tasks) -> Vec<AgentHandle>` (STUB runs sequentially; true parallel in 3D), `communicate(from, to, message) -> ()` (STUB direct call; IPC in 4A).
- `AgentDeathClassification`: CLEAN_EXIT, CRASH, TIMEOUT, BUDGET_EXCEEDED (orchestrator.txt L148-149).

### Deliverables (6)
- 3C.1 AgentPool trait + AgentHandle + AgentStatus. VRAM check via ModelPool::available_vram() in spawn; AgentGrcConfig loaded from `elai_cc.toml [agents.{role}]` at spawn.
- 3C.2 Lifecycle with death classification. Heartbeat via `tokio::sync::watch`, 5s interval, 15s timeout. Tier A invisible walls: per-agent `allowed_paths: Vec<GlobPattern>`; hidden paths return "not found" (indistinguishable from nonexistent) (R31 mitigation). ReplanRequest max 1/agent/phase, counter resets at phase boundary; MINOR=fast-track, MAJOR=human INTERACTIVE gate. Every GRC think() emits ReasoningTraceEntry to SSB.
- 3C.3 Checkpointing: 5 files at `.elai_cc/sessions/{sid}/agents/{agent_id}/`:
  - state.msgpack (AgentStatus + GRC state + active_files)
  - grc_history.msgpack (full GRC history for current topic)
  - tool_results.msgpack (tool results since last checkpoint)
  - output_draft.msgpack (partial AgentResponse)
  - checkpoint_ts (plain text, ClockProvider::now_monotonic)
  Triggers: every GRC step, every tool result, every file edit, before slow ops (>1s). Atomic write: temp->fsync->rename. SSB entry with metadata only (not data). **UntrackedValue rejection**: checkpoint serializer validates channel refs against registry; UntrackedValue ref -> `CheckpointError::UntrackedValueReference` (WARN 3 fix from technical-analysis-v3.txt Concern 2).
- 3C.4 Recovery: detect CRASH/TIMEOUT, reload latest checkpoint, spawn same agent_id, inject GRC history + tool results + output draft, agent resumes at next GRC step. First post-recovery step emits grounding_source="CHECKPOINT:{path}". Claw-code mapping: TrustPromptUnresolved=N/A, PromptMisdelivery=CRASH, StaleBranch=CRASH, CompileRedCrossCrate=CRASH, McpHandshakeFailure=N/A (plugins), PartialPluginStartup=N/A, ProviderFailure=CRASH (retry with same/fallback model).
- 3C.5 Supervision: Erlang-derived. `SupervisionPolicy { max_restarts: u32 (default 3), restart_window: Duration (phase duration), on_max_restarts: EscalationAction { Escalate, SubstituteModel(ModelId) } }`. Phase 3 INTERACTIVE => Escalate; Phase 4G AUTONOMOUS => SubstituteModel (gated error in Phase 3). restart_count resets at phase boundary. BUDGET_EXCEEDED does NOT increment. Logs supervision_exhausted event on max.
- 3C.6 Anti-spiral: track last N GRC steps (step_type + topic + content_hash via blake2s on normalized content). 3 consecutive matches -> SPIRAL. On detection: checkpoint (tagged recovery_target = step N-3), inject convergence signal, if another identical step -> permanent stop. SSB `spiral_detected` event. INTERACTIVE: show human evidence + [retry hint | substitute model | skip | abort].

### Deferred (explicit)
- Tier B sandbox (Firecracker/gVisor) -> Phase 4A.
- Cognitive isolation metric aggregation at phase boundary -> 3A or 3D hook (not 3C).

### Targets
- Checkpoint write <50ms (5 files).
- Recovery <2s.
- Checkpoint size <1MB.
- Heartbeat overhead <0.1% CPU.
- Anti-spiral detection <1ms/GRC step.

---

## 3D BSP SUPERSTEPS + CHANNELS — Status: IMPROVED

Goal: deterministic macro orchestration via BSP (Valiant 1990). 4 channel types (BinaryOp intentionally dropped per audit BLOCK 4 — non-deterministic with concurrent writes).
Determinism: LastValue total-ordered via SSB seq; Topic append-only deterministic; Blelloch et al. CMU 2012 parallel-combiner requirement satisfied. Byte-identical checkpoints require same agent participation sets (straggler timeouts break this with real LLMs).

### Spec amendments required (before implementation)
1. orchestrator.txt §5: add `Topic(accumulate=false)` two-phase reset semantics.
2. orchestrator.txt §3: add `memory_monitoring` action_type (already in 3B schema above, but spec needs update).
3. orchestrator.txt §5: add ChannelRegistry trait definition.
4. orchestrator.txt §4: fix save_checkpoint placement (after evidence_gate_passes to prevent collision on retry).

### Channel types (4)
- `LastValue<T>`: single value, last writer wins by SSB seq (monotonic). Read returns `Option<T>`.
- `Topic<T>(accumulate: bool)`:
  - accumulate=true: append-only `Vec<T>`, persists across supersteps.
  - accumulate=false: Vec<T> cleared at superstep boundary via two-phase: (1) snapshot for scheduling, (2) clear before execute writes (WARN-7 fix).
  Append ordering by SSB seq.
- `EphemeralValue<T>`: fire-and-forget. `u32` write_count per channel per superstep (saturating_add); barrier checks; 0 writes => MISSING. Clearing recorded in SSB as action_type='checkpoint' subtype 'ephemeral_clear' for replayability (WARN-8 fix).
- `UntrackedValue<T: DeepSizeOf>`: in-memory only, never serialized. guard=false for multi-writer. Persists until overwritten or force-cleared. Lost on crash. T must impl `DeepSizeOf` (std::mem::size_of_val returns only stack size — Vec<u8> = 24 bytes regardless of heap — BLOCK-2 fix).

### Rust types
- `ChannelId<T>` (typed key, e.g. `ChannelId<LastValue<PlanDraft>>`).
- `ChannelState` enum (3 variants for checkpoint: LastValue, Topic, Ephemeral — UntrackedValue excluded).
- `ChannelRegistry` trait (implementation-defined here — not yet in spec):
  - read(id) -> ChannelValue<T>
  - write(id, value) -> Result<(), ChannelError>
  - snapshot_for_scheduling() -> ChannelSnapshot
  - clear_ephemeral() -> cleared count
  - force_clear_untracked(id) -> bool
  - deep_size_untracked(id) -> usize
- `ChannelWrite`, `ChannelValue`, `ChannelError`, `ChannelSnapshot` (also implementation-defined; spec amendment 3 needed before 3E starts).

### Deliverables (5)
- 3D.1 Channel definitions (orchestrator/src/bsp/channels.rs). `size_of_val` banned — grep returns 0.
- 3D.2 BSP loop (orchestrator/src/bsp/engine.rs). Corrected sequence (save_checkpoint AFTER evidence gate):
```
loop {
  channel_snapshot = channels.snapshot_for_scheduling();
  tasks = prepare_next_tasks(checkpoint, channel_snapshot, nodes);
  if tasks.empty() break;
  channels.clear_non_accumulating_topics();
  writes = execute_tasks_parallel(tasks, model_pool, ssb);
  apply_writes(checkpoint, channels, writes);
  // deferred phase (3D.3)
  channels.clear_ephemeral();  // logs to SSB for replay
  if !evidence_gate_passes(checkpoint, channels) {
    attempt += 1;
    if attempt >= max_superstep_attempts break;
    continue;
  }
  save_checkpoint(checkpoint, channels);  // AFTER gate
  superstep += 1;
  attempt = 0;
}
```
  apply_writes applies in SSB seq order (deterministic). Checkpoint path: `.elai_cc/sessions/{sid}/checkpoints/superstep_{N}.msgpack`. max_superstep_attempts default 3 (separate from max_supersteps=100). Straggler timeout: reuses 3C heartbeat (15s default); classified TIMEOUT and removed from barrier.
- 3D.3 Deferred node execution. NodeSpec has `is_deferred: bool`. Executed between apply_writes and clear_ephemeral. `all_inputs_satisfied` check. Single pass — no cascading (cycle-safe).
- 3D.4 Checkpoint + memory management. BSP checkpoint triggers agent checkpoint at boundary. UntrackedValue monitoring: `deep_size_untracked(id)` at boundary; logged as `memory_monitoring` action_type (NOT isolation_metric — reserved for cognitive token savings, orchestrator.txt §7A). `> 2x limit` -> force_clear. EphemeralValue zero-write detection via write_count saturating_add (reset to 0 at superstep start). Topic two-phase reset. Crash recovery: load latest checkpoint + replay SSB entries since checkpoint superstep in SSB seq order.
- 3D.5 Parallel agent execution (upgrades 3C spawn_parallel). `tokio::JoinSet::join_all()` with per-agent timeout. VRAM-aware: `max_concurrent = floor(available_vram / per_agent_vram_estimate)`, fallback 1. Results collected in SSB seq order.

### Risks
- R19 UntrackedValue memory: default 10MB per channel, force-clear at 2x (config: `elai_cc.toml [orchestrator.bsp.untracked_memory_limit_bytes]`).
- R-NEW-1 Straggler (CMU HotOS'13 cited): heartbeat timeout removes agent.
- R-NEW-2 Mid-superstep crash: 3C supervision tree removes dead agent; orchestrator proceeds.

### Baselines (all estimates, measure empirically)
- Superstep overhead <50ms for 4 agents.
- Checkpoint file <1MB for 4-agent session.
- Recovery time <500ms.
- Near-linear speedup for CPU-bound parallel agents.

### Grounding highlights
- BSP determinism VERIFIED Blelloch CMU 2012 paper.
- LastValue by SSB seq GROUNDED orchestrator.txt §5 L101.
- BinaryOp dropped GROUNDED audit-results.txt BLOCK 4.
- Deferred nodes GROUNDED audit-results.txt BLOCK-5 + LangGraph defer=True precedent (May 2025).
- Topic(accumulate=false) NEW DEFINITION (spec gap, grep 0 hits) — WARN-7 fix.
- size_of_val stack-only VERIFIED Rust std docs — BLOCK-2 fix.
- isolation_metric vs memory_monitoring GROUNDED §7A L191-192 — BLOCK-6 fix.

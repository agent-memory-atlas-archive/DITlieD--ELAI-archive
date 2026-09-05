# Phase 3 Summary — Part 2 (3E, 3F, 3G, 3H, 3I)

## 3E CONTEXT BUDGET ALLOCATOR — Status: IMPROVED

Goal: fixed-slot context budget so model_max_context is never exceeded. Adaptive reduction with floors on critical slots. Cache breakpoints aligned with provider prefix caching.

Prereqs: 3D BSP supersteps + channels operational. Depends on `providers::token_count`, `providers::token_count_estimate`, `orchestrator::SSB`, `orchestrator::BSP`.

### Types
```
enum ContextSlot {
  SystemPrompt,   // slot 1: GRC structured prompt per agent role
  Rules,          // slot 2: active Tier 3 rules for current phase + file context
  PhaseSpec,      // slot 3: from PipelineSpec
  RepoMap,        // slot 4: PageRank DKG output, fitted to budget
  CodeContext,    // slot 5: BM25 + vector chunks, cross-encoder reranked
  SsbEntries,     // slot 6: filtered by file overlap score
  GrcHistory,     // slot 7: prior reasoning for current topic
  OutputReserve,  // slot 8: minimum 2000 tokens
}
struct SlotAllocation { slot, allocated_tokens, floor_tokens, actual_tokens, source_truncated }
struct ContextBudget {
  model_max_context, slots: [SlotAllocation; 8],
  cache_breakpoints: [CacheBreakpoint; 2],
  total_allocated, total_used, adaptive_level
}
struct CacheBreakpoint { after_slot, prefix_hash: u64 (FxHash/xxhash, NOT DefaultHasher), valid: bool }
enum AdaptiveLevel { Normal, FilteredByRelevance, ReducedGranularity, ReducedScope, Terminal }
```

### Slot order (never reordered)
1 SystemPrompt, 2 Rules, 3 PhaseSpec, **BP1**, 4 RepoMap, 5 CodeContext, **BP2**, 6 SsbEntries, 7 GrcHistory, 8 OutputReserve.

BP1 invalidated by: phase transition, role change, rule set change.
BP2 invalidated by: file set change.
Slots 6-8 recomputed every invocation.

### Budget distribution
- Fixed 1-3 measured exactly, subtracted first.
- OutputReserve subtracted before percentage split (min 2000 tokens).
- dynamic_budget = model_max_context - sum(1-3 exact) - output_reserve.
- RepoMap 15%, CodeContext 40%, SsbEntries 25%, GrcHistory 20%.
- Floors: SsbEntries>=200, GrcHistory>=500, OutputReserve>=2000.
- If dynamic < sum(floors)=700 -> BudgetInfeasible.

### SSB entry filtering (slot 6)
`score = |intersection(entry.files, agent.files)| / |agent.files|`
Entries sorted (score DESC, seq DESC). score=0 excluded. Empty agent_files -> 0.0 (guard against div/0).

### BSP interaction
LastValue latest if overlap>0; Topic accumulated entries filtered by overlap; EphemeralValue only if written in current superstep; UntrackedValue excluded (never in context). Allocator called after prepare_next_tasks, before execute_tasks.

### Adaptive reduction (3 attempts, each logged to SSB as action_type='context_reduction')
- Attempt 1 FilteredByRelevance: raise min overlap threshold (config, e.g. 0.3); GRC keep current topic only.
- Attempt 2 ReducedGranularity: PageRank higher density guard; CodeContext = function signatures + doc comments.
- Attempt 3 ReducedScope: RepoMap = current file + direct deps; CodeContext = current file only; SSB = agent's own entries only.
- Terminal: INTERACTIVE -> `Err(Escalate(breakdown))`; AUTONOMOUS -> `Err(SplitTask)` (Phase 4G receiver).

### Prompt assembly mapping (prompt-architecture.txt §3)
- Slot 1 -> [role_prompt] ~50-100
- Slot 2 -> [tier_3_rules] ~100-300
- Slots 4+5 -> [code_context] ~2000-8000 (assembler merges; allocator stays independent)
- Slot 3 -> [task_spec + output_format + constraints] ~250-600
- Slots 6-7 injected as structured context after task_spec
- Slot 8 not in prompt (reserved for model output)

### Session handoff interaction (3H)
Trimming preserves prefix tokens in slots 1-3 (BP1). Stubs replace tool results >500 tokens; slot content intact. BP1 survives handoff if phase/role/rules unchanged; BP2 must be recomputed.

### Deliverables (7)
- 3E.1 Types and budget struct. Display impl. ContextSlot ordering static_assert (SystemPrompt=0, OutputReserve=7).
- 3E.2 Budget computation. Example: compute(8192, [100,200,300], 2000) -> dynamic=5592; RepoMap=838, CodeContext=2236+2=2238 (remainder to largest), SsbEntries=1398, GrcHistory=1118. sum == model_max_context (no waste, no overflow).
- 3E.3 Floor enforcement. Steal from CodeContext first then RepoMap. Both donors insufficient -> BudgetInfeasible.
- 3E.4 Cache breakpoint tracking. Prefix hash = FxHash/xxhash (NOT DefaultHasher — non-deterministic across runs). Reports `CacheBreakpoint.valid` to provider.
- 3E.5 SSB entry filtering. Uses rusqlite GROUP BY on ssb_files to build per-entry file sets.
- 3E.6 Adaptive reduction. Each level logged with before/after token counts.
- 3E.7 Integration test: full budget assembly. Gemma 4 E2B 128K context (test with 4096 and 131072); <50ms assembly; 100x identical output (determinism); grep `Instant::now` returns 0 (entropy quarantine).

### Targets
- Full budget computation <50ms (architecture-overview §8).
- Deterministic: 100x identical.
- SSB filtering O(n), 10K entries <10ms.

---

## 3F RULES ENGINE — Status: IMPROVED (three-tier, self-improving)

Goal: deterministic rule enforcement replacing prompt-based rules. Predicate DSL with 5 types. Self-discovery from auditor violation patterns. Rules are DATA in SQLite evaluated by Rust interpreter.
Tiers: T1 deterministic code, T2 constrained decoding, T3 prompt guidance + auditor backup.

### Architecture decision: tree-sitter dependency
AST_PATTERN predicates require tree-sitter. Phase 1A adds tree-sitter to `knowledge/` crate. Rules/ crate MUST NOT add separate tree-sitter dep. Instead: `knowledge::AstEvaluator { fn evaluate_query(lang, source, query) -> Vec<Match>; fn validate_query(lang, query) -> Result<(), AstError> }` trait; rules/ takes `Box<dyn AstEvaluator>`.

### Predicate evaluation order (cheapest first, short-circuit on BLOCK)
1. METRIC_GATE (O(1))
2. GREP_PATTERN (O(n))
3. AST_PATTERN (parse-dependent)
4. COMPOSITE (recursive)
Within type: sort by severity DESC. WARN collection: accumulated WARNs before BLOCK carried in response.

### Dependencies (dev-only unless noted)
- proptest (dev): ~1400 LoC property tests across 7 domains.
- criterion (dev): rule eval latency benchmarks.
- jsonschema (runtime): SCHEMA_REQUIRED validation.
- tree-sitter (runtime, via AstEvaluator trait).

### SQLite schema (rules.db)
Tables: `rules`, `rule_metrics`, `contract_tests`, `violation_patterns`, plus `schema_version` for migrations. Violation_patterns has 3-axis scoring (score_clarity, score_frequency, score_precision). rule_metrics has counterfactual columns (causal_catches, spurious_catches).

### Predicate types (5)
AST_PATTERN (delegates to AstEvaluator trait), GREP_PATTERN (Regex, compiled once at insertion), SCHEMA_REQUIRED (jsonschema crate), METRIC_GATE (lt|gt|eq|between), COMPOSITE (AND|OR|NOT, max depth 8).

### Verdict + escalation
`Verdict` enum: ALLOW, WARN, BLOCK, ESCALATE(EscalationTarget).
`EscalationTarget` enum: Auditor, Human, NextTierRule(RuleId).

### RuleEngine trait
```
evaluate(action, context) -> ALLOW|BLOCK|WARN
get_active_rules(fsm_state, file_context) -> Vec<Rule>
compile_predicate(raw: &str) -> Result<CompiledPredicate, PredicateError>
```
compile_predicate validates regex (`Regex::new`), tree-sitter (AstEvaluator::validate_query), JSON schemas ALL at insertion time.

### Rule lifecycle (deterministic arithmetic, no LLM)
PROPOSED -> CANDIDATE (>=3 contract tests, all pass) -> SHADOW (tests pass + human approval OR independent second-auditor — finding-source auditor CANNOT approve) -> ACTIVE (false_positive_rate<5% AND true_positive_rate>80% AND counterfactual_causal_rate>50% AND no safety regressions) -> RETIRED (fpr>30% over 10 pipelines OR superseded OR human).
Safety invariants: `mutable=0` rules reject auto-retirement AND exempt from causal rate gate (human-promotable only).

### Phase-aware / domain-aware activation
phase_mask bitmask + file-path domain detection.
`get_active_rules` SQL: status=ACTIVE AND phase_mask match AND (domain match OR domain IN ('engineering','security')) (engineering + security are cross-cutting, mutable=0).
Domain mapping configurable via `elai_cc.toml [rules.domain_mapping]`.

### Self-discovery
- Violation pattern normalization in ORCHESTRATOR (not auditor) — prevents auditor influencing its own rule generation (circular trust).
- blake2s hashing of normalized text.
- occurrences >= 3 -> auto-proposal.
- Predicate generation: tree-sitter query from common AST structure -> regex fallback -> Tier 3 only.
- COMPILE-TIME VALIDATION: invalid generated predicates logged + discarded, not inserted.
- Auto-contract-test generation: each example_code -> BLOCK test; orchestrator generates ALLOW counterexamples.
- Trust boundary: `source=AUDITOR` rules require human OR second-auditor approval for CANDIDATE promotion.

### Seed data (~33 rules minimum initial shipment)
- Engineering + security domains (always active cross-cutting).
- Top trading rules.
- Post-tool completeness GREP rules (T1, PostToolUse hook, <1ms, deterministic):
  - web_fetch result `<img>` tags -> WARN "N images not extracted"
  - web_fetch result `<table>` tags -> WARN "N tables not extracted"
  - file_read result TODO/FIXME -> INFO "N TODO/FIXME items"
  Source: technical-analysis-v3.txt Concern 3 Mechanism 2.
- Merge logic: global seeds + project rules (project overrides by rule_id, but mutable=0 security seeds cannot be downgraded to mutable=1).

### Contract tests (>=254 static)
Arithmetic from rules.txt §9A: 25*3 + 5*2 + 11*3 + 11*3 + 2*2 + 10*3 + 7*3 + 10*3 + 6*3 = 254.
Min 3/T1 rule (BLOCK+ALLOW+EDGE), min 2/T2 rule (valid+invalid schema).
proptest: violation_generator, valid_generator, mutation_invariant, fix_mutation.
COMPOSITE proptest: `prop_recursive` AND/OR/NOT (depth 8, size 64). De Morgan invariant: NOT(A AND B) == NOT A OR NOT B. Associativity. Shrunk counter-examples auto-inserted as source=GENERATED.
256 cases per rule. Contract tests READ-LOCKED during self-modification.

### Benchmarks
Target: GREP/METRIC <1ms median, AST <5ms, 10-rule mixed <10ms.

### Counterfactual scope note
Data model + gate logic defined in 3F. Runtime integration (running counterfactual auditor passes during AUDITING) is 3C+3A territory. 3I (DST) should include counterfactual testing in simulation.

---

## 3G HIERARCHICAL PLAN MANAGEMENT — Status: IMPROVED

Goal: two-artifact plans (skeleton 2-4k tokens always in Slot 3 + per-phase elaboration on demand). Contract extraction propagates interface changes. Coherence enforcement prevents drift.

Depends on: orchestrator::SSB (action_type='plan_update'), orchestrator::ContextBudget (Slot 3), providers::token_count[_estimate].
NOTE: dep graph error — 3G.4 requires 3E's Slot 3 allocator. Correct: 3D -> 3E -> 3G; or parallel but gate 3G.4 on 3E.

### Types
```
struct PlanSkeleton {
  pipeline_id, phases: IndexMap<PhaseId, PhaseEntry>,
  global_constraints: Vec<String>, version: u32,
  created_at, updated_at  // ClockProvider
}
struct PhaseEntry {
  phase_id, goal (<=120 chars), inputs: Vec<ContractRef>, outputs: Vec<ContractRef>,
  constraints, depends_on, depended_by, version,
  output_type: Option<OutputType>, visual_check: bool
}
struct ContractRef { name, type_signature, invariants }
struct PhaseDetail {
  phase_id, skeleton_version,
  contracts: ExtractedContracts,
  implemented_contracts: Vec<ContractRef>,   // for coherence check 1
  declared_interfaces: Vec<ContractRef>,      // for coherence check 2
  content, token_count
}
struct ExtractedContracts { input_contracts, output_contracts, shared_constraints }
```
`output_type` + `visual_check` -> media-validator.txt §7 Render-Capture-Evaluate. When `visual_check=true`, L2 validates `MediaRenderAdapter` exists for output_type. `output_type=None` forces `visual_check=false`.

### Storage (dual-path, TOML authoritative)
- Skeleton: `.elai_cc/pipelines/{id}/skeleton.toml`.
- Phase detail: `.elai_cc/pipelines/{id}/phases/{phase_id}.toml`.
- SSB audit: action_type='plan_update'.
- NOTE: orchestrator.txt L250 references `SELECT * FROM plan_phases` — conceptual shorthand; no plan_phases table in SSB schema. Query via SSB action_type='plan_update' with JSON payload extraction, or spec amendment adding plan_phases table/view.

### Token counting
Skeleton: token_count_estimate for allocation, exact token_count for final (>3600 safety margin triggers exact check); 4096 hard cap -> `SkeletonBudgetExceeded`.

### Contract extraction algorithm (orchestrator.txt L234-239)
1. input_contracts = union of dep.outputs for dep in depends_on.
2. output_contracts = union of dep.inputs for dep in depended_by.
3. shared_constraints = skeleton.global_constraints + phase.constraints.
Model elaborates against contracts (not full plan).

### Coherence enforcement (3 checks, orchestrator.txt L241-246)
1. Phase detail satisfies input/output contracts (set comparison by name+type_signature, missing -> BLOCK).
2. No new interfaces in PhaseDetail.declared_interfaces beyond skeleton's union of inputs/outputs (set diff, structured — NOT free-text parsing).
3. If any ContractRef modified: phase.version AND skeleton.version must both increment.
Invoked BY auditor agent via IPC ToolRequest (orchestrator provides logic). BLOCK findings emitted as action_type='finding'.

### Stub references
`action_type='stub_reference'` emitted on skeleton write for each phase lacking PhaseDetail file. SSB append-only -> superseded when detail written (no delete).

### Dependency propagation
Kahn's algorithm (O(V+E), detects cycles via remaining nodes).
On contract change: BFS through depended_by -> flag `needs_review=true` in SSB (`action_type='plan_update', payload {phase_id, reason: "dependency_contract_changed", source_phase}`).
Cycle detection on skeleton write rejects cycles.
phase.version += 1 on contract change; skeleton.version += 1 on any phase version change. Checked_add on u32 overflow.

### Skeleton split operation (L2 Phase Sizing)
`PlanSkeleton::split_phase(phase_id, split_points)` with `preserve_contracts: bool` (default true):
- input contracts copied to first sub-phase
- output contracts copied to last sub-phase
- internal contracts created between adjacent sub-phases
Max split depth 2 (matches Phase 4G SPLIT_TASK).
Source: technical-analysis-v3.txt Concern 4.

### Deliverables (7)
- 3G.1 Types + skeleton schema. serde TOML. goal length 120 validated.
- 3G.2 Skeleton serialization + token enforcement. Estimate then exact. SSB plan_update. stub_reference emission.
- 3G.3 Contract extraction (3-step algorithm exactly). Errors: PhaseNotFound, ContractConflict (same name, differing type_signatures across deps).
- 3G.4 Phase elaboration loading. Validates combined tokens fit Slot 3; reduction via 3E (ReducedGranularity: skeleton+contracts only). skeleton_version staleness check. One-hop adjacent contracts loaded.
- 3G.5 Coherence enforcement (3 structured-set checks). Invoked via IPC. implemented_contracts empty => Err (must self-declare).
- 3G.6 Dependency propagation + version tracking. BFS with visited set. Split with preserve_contracts.
- 3G.7 Integration test full lifecycle. 5 phases (A->B->C, A->D, D->E). 100 runs determinism. unique pipeline_id via EntropyProvider.

### Targets
- Skeleton <2000 tokens for 20-phase plans, <4096 hard cap.
- Contract extraction <10ms for 50-phase skeleton.

---

## 3H SESSION HANDOFF (CMV-DERIVED DAG STATE MGMT) — Status: IMPROVED

Goal: DAG-based session state versioning with structurally lossless trimming. Sessions branch/explore/merge. Checkpoint triggers: every GRC step, tool call, phase transition. Handoff trigger: 85% token budget (configurable).
Paper: arxiv 2602.22402 (CMV). Interface map: workflow/papers/cmv-interface.md.

### Spec amendments required
1. snapshot_context schema: expand from L293-294 BLOB-per-snapshot (role TEXT, trimmed_messages BLOB, stub_count INTEGER) to per-message row design (seq, role, content, is_stubbed, stub_ssb_seq) — enables selective stub recovery without deserializing entire BLOB.
2. SSB action_type for branch operations: (a) add 'branch_abandon' to CHECK OR (b) reuse 'checkpoint' with branch metadata in payload (default assumed (b)).
3. Cleanup on branch merge: L312 "Only winning branch's SSB entries kept" — append-only contradicts physical delete. Default: logical exclusion via query filter.

### SQLite schema (session_dag.db, one per session, default journal mode — NOT WAL)
```
CREATE TABLE snapshots (
  snap_id TEXT PRIMARY KEY,    -- UUID v4 via EntropyProvider
  parent_id TEXT REFERENCES snapshots(snap_id),  -- NULL for root
  session_id TEXT NOT NULL,
  fsm_state TEXT NOT NULL,
  phase TEXT, step TEXT,
  findings BLOB,           -- msgpack: Vec<AuditFinding>
  code_changes BLOB,       -- msgpack: Vec<FileChange>
  tests_written BLOB,      -- msgpack: Vec<TestRef>
  commit_hashes TEXT,      -- comma-separated git SHAs
  created_at INTEGER,      -- ClockProvider unix millis
  tag TEXT                 -- "handoff", "branch:{name}", "abandoned:{name}", NULL
);
CREATE TABLE snapshot_context (
  snap_id TEXT NOT NULL REFERENCES snapshots(snap_id),
  seq INTEGER NOT NULL,
  role TEXT CHECK(role IN ('system','user','assistant','tool')),
  content BLOB NOT NULL,
  is_stubbed INTEGER NOT NULL DEFAULT 0,
  stub_ssb_seq INTEGER,
  PRIMARY KEY (snap_id, seq)
);
CREATE INDEX idx_snapshots_session ON snapshots(session_id);
CREATE INDEX idx_snapshots_tag ON snapshots(tag) WHERE tag IS NOT NULL;
CREATE INDEX idx_context_stubbed ON snapshot_context(snap_id) WHERE is_stubbed = 1;
```
PRAGMA foreign_keys = ON; PRAGMA integrity_check on open.

### Rust types
- `SessionDag { db: rusqlite::Connection, session_id, active_branches: Vec<BranchRef> (max 3) }`.
- `Snapshot { snap_id, parent_id, session_id, fsm_state, phase, step, findings, code_changes, tests_written, commit_hashes, created_at, tag }`.
- `TrimmedMessage { seq: u32, role: MessageRole, content: Vec<u8>, is_stubbed: bool, stub_ssb_seq: Option<u64> }`.
- `Stub { original_type: String, original_tokens: usize, summary_hash: [u8; 32] (blake3), ssb_seq: u64 }`.
- `BranchRef { name, tip_snap_id, created_at }`.
- `MergeResult { Winner { branch, abandoned: Vec<BranchRef> }, Conflict { reason } }` — Conflict variant is UNGROUNDED (spec says pick winner; safe default detects file overlap in code_changes).

### Risks
- R-3H-1 DAG cycles on branching: parent_id immutable; append-only; acyclicity via parent-chain walk terminating at NULL with visited set.
- R-3H-2 Orphan stubs: Pass 3 validates every stub against SSB before committing.
- R-3H-3 Token-count inaccuracy: measure trimmed context with exact token_count before writing.
- R-3H-4 Branch explosion: hard cap 3 active branches.
- R-3H-5 Prompt cache invalidation: trimming only operates on Slot 6+; Slots 1-3 never stubbed; Slots 4-5 regenerated per invocation.
- R-3H-6 Compaction boundary: exact token_count at 85% trigger (not estimate).
- R-3H-7 session_dag.db corruption: DELETE journal mode + PRAGMA integrity_check on open -> Err(DatabaseCorrupted).
- R-3H-8 TrimConfig coupling: verify 3E exposes total_allocated_tokens() before 3H.3.

### Structurally lossless 3-pass trimming (CMV paper O(3N))
- `target_tokens = model_context_window - allocated_slots_tokens - output_reserve`.
- Pass 1 (identify): stream messages; for each tool-result > stub_threshold mark for stubbing. **Never** mark user or assistant messages.
- Pass 2 (stub): replace content with Stub (msgpack). `summary_hash = blake3(original)`. Set is_stubbed=true, stub_ssb_seq.
- Pass 3 (validate): for each stub, verify SSB seq exists; if missing UNSTUB (restore original). Measure total; >target_tokens -> `Err(TrimBudgetExceeded)`.
- stub_threshold range [100, 1000] (CMV hyperparameters); default 500.
- Slot 1-3 positions never candidates (prompt cache preserved).

### Deliverables (6)
- 3H.1 Schema + types.
- 3H.2 Snapshot save/load. Transaction atomicity (crash between INSERTs rolls back). parent_chain with visited set (1000 iter max) -> CycleDetected. Batch insert 1000 msgs <100ms.
- 3H.3 3-pass trimming.
- 3H.4 Branch/merge.
  - `create_branch(name, from_snap_id)`: verify active<3, create snap with tag="branch:{name}", copy snapshot_context from parent.
  - `merge_branch(winner)`: creates NEW trunk snapshot from winner tip, parent_id = current trunk tip (DAG immutable — no parent_id mutation). Tags losing branches "abandoned:{name}". SSB via action_type='checkpoint' with `{event:"branch_merge", winner, abandoned, snap_ids}` (or new 'branch_abandon' if spec amendment lands). Optional file-conflict detection on code_changes overlap -> `Conflict`.
  - `close_branch(name)`: remove from active, tag abandoned.
  - Branch naming: alphanumeric + hyphens, max 64 chars.
- 3H.5 Stub recovery from SSB.
  - `recover_stubs(snap_id, ssb) -> Vec<TrimmedMessage>`: query SSB by stub.ssb_seq, verify `blake3(recovered) == stub.summary_hash`, replace content, is_stubbed=false.
  - `recover_single_stub(snap_id, seq, ssb)` for on-demand recovery.
  - Failures: StubRecoveryFailed { seq, reason: "SSB entry missing" | "integrity check failed" }.
- 3H.6 Handoff + full integration test.
  - `create_handoff(messages, config, ssb) -> Snapshot`: runs 3-pass + saves with tag="handoff".
  - `load_handoff()`: find latest handoff; if branch exists, load branch tip (orchestrator.txt L319).
  - Compaction boundary: 85% triggers, 84.9% doesn't, 100% still triggers (late but not missed).
  - 50 messages -> trim -> save -> open new SessionDag -> load handoff -> user/assistant byte-equal -> recover stubs -> byte-equal originals -> 2 branches -> add snaps -> merge winner -> abandoned tagged -> parent chain acyclic -> commit_hashes persist -> 100x same seed = identical.

### Targets
- Snapshot save/load <50ms for 100 messages.
- Trimming 500 messages <10s.
- Token reduction >0.3 for tool-heavy sessions.
- DAG size on disk <10MB for 1000-snapshot session.

---

## 3I DETERMINISTIC SIMULATION TESTING (DST) — Status: IMPROVED

Goal: prove FSM+SSB+BSP correctness via deterministic simulation. Reference model (digital sibling) as infallible oracle. Same seed = same output guaranteed.

Prereqs: Phase 0.8 entropy quarantine + 3A-3D.

### DST infrastructure
**VCR (Video Cassette Recorder) pattern for LLM determinism** (madsim + temperature=0 alone insufficient due to GPU float divergence, MoE routing, ollama caching):
- Intercept at HTTP transport (reqwest middleware or madsim network mock) NOT at LlmProvider trait level.
- Compute blake2s hash of full HTTP request body. Store (hash, response) pairs in msgpack cassette.
- DST: lookup hash -> return recorded response with simulated latency from madsim timer. Miss -> test fails (prompt drift).
- Trait-level mocking remains a complementary pattern for unit tests.
- **TraceOps** layer: record entire agent execution trajectory; feed deterministic trajectory instead of live LLM.

**Custom SQLite VFS**:
- Trap write()/fsync() syscalls; redirect to in-memory byte buffer; deterministic PRNG dictates I/O latency.
- Enables torn-write simulation, deterministic I/O latency, WAL recovery testing.
- ~200-400 LOC Rust via unsafe C FFI (`sqlite3_vfs` struct).
- Must use madsim timer queue (single deterministic time source).
- Reference: Limbo project (API patterns; NOT a VFS plugin, async SQLite rewrite).
- SPIKE required: evaluate madsim libc I/O hooks before implementing; may be unnecessary.

### Risks
- R1 madsim Windows UNVERIFIED — spike in 3I.1.
- R2 madsim NOT drop-in for tokio — cfg(madsim) + madsim-tokio swap.
- R3 Use proptest-state-machine v0.8.0 (same repo as proptest) — `ReferenceStateMachine` + `StateMachineTest` traits.
- R4 `RUSTFLAGS=--cfg madsim` full workspace rebuild; separate test binary or accept compile delta.
- R5 `PROPTEST_CASES=10000` x 200 transitions = 2M+ apply() calls — 10 min hard timeout.

### Deliverables (9)
- 3I.1 Madsim Windows spike. madsim v0.2.34 dev-dep. cfg(madsim) gate. Two-node message test. Evaluate alternatives (manual deterministic scheduler) if Windows fails.
- 3I.2 Skeleton:
  - `SimulatorRuntime { SeededClock + SeededRng + madsim runtime }`. advance_time, inject_fault, snapshot_state, with_seed.
  - `FsmReferenceModel` implements `ReferenceStateMachine`: `State=ReferenceState` (FSM + SSB row counts + agent lifecycle + channel state), `Transition=FsmEvent`, init_state, transitions, preconditions, apply.
  - `FsmEvent` enum (10 variants): PhaseTransition, AgentSpawn, AgentCrash, AgentCleanup (recovery after crash — without it ref model diverges), ToolCall, FileWrite, GrcStep, SSBWrite, ConfigChange, SuperstepBoundary (composite atomic event modeling clear_non_accumulating_topics + execute + clear_ephemeral + evidence_gate).
  - `StateMachineTest` impl: SUT=actual FSM+SSB, Reference=FsmReferenceModel. apply() returns SUT BY VALUE (ownership threading model of proptest-state-machine). check_invariants: FSM state match, SSB row counts, active agents, channel states (LastValue current, Topic vec contents, Ephemeral presence).
  - Cargo: proptest v1.11+, proptest-state-machine v0.8+, madsim v0.2.34.
- 3I.3 Verify: clean compile, no regressions.
- 3I.4 TEST-WRITE reference model (TDD red):
  - `prop_state_machine! { #[test] fn fsm_reference_model_matches(sequential 1..200 => FsmStateMachineTest); }`.
  - PROPTEST_CASES=10000 CI + 10 min timeout; 256 local.
  - Determinism test via `madsim::runtime::Runtime::check_determinism(seed, config, bare_fn_pointer)` — STATIC function, bare pointer (no closures), state init inside function.
  - Synthetic regression: inject known violation -> creates proptest-regressions/ file -> check into git.
- 3I.5 TEST-WRITE BSP determinism (TDD red):
  - Same seed + same channel state + same writes = identical checkpoint (100 repetitions).
  - UntrackedValue NEVER in checkpoint (grep serialization).
  - EphemeralValue zero-write clears to MISSING; write_count resets at superstep start.
  - Topic SSB-seq ordering; SuperstepBoundary two-phase reset.
  - PRE_GATE: entropy quarantine verified — grep Instant::now|thread_rng|SystemTime::now returns 0 in orchestrator/src/. If any hits: BLOCK.
- 3I.6 TEST-WRITE madsim IPC (preparation for Phase 4A; TDD red):
  - Inject faults via net::Config: packet_loss_rate=0.3, send_latency Range<Duration> (uniform jitter — requires explicit Duration ctor not int shorthand).
  - `NetSim::clog_link(src, dst)` asymmetric disconnect.
  - Agent SIGKILL (madsim node kill) mid-message.
  - Burst loss: 100% loss for 5 consecutive then 0% (tests correlated failures).
  - All scenarios reproducible via seed replay.
  - NOTE: `RUSTFLAGS=--cfg madsim` triggers full rebuild.
- 3I.7 TEST-WRITE adversarial state space (TDD red):
  - Anti-spiral: 3x same GRC step -> escalation.
  - Evidence gate: missing artifacts -> BLOCK.
  - Context budget: oversized -> adaptive reduction.
  - SSB conflict: concurrent same-path writes -> single-writer serializes.
  - Demonic scheduling via proptest.
- 3I.8 IMPLEMENT (TDD green). Reference model <30% of real FSM LOC:
  ```
  python -c "import pathlib; r=len(pathlib.Path('REF_MODEL_PATH').read_text().splitlines()); f=len(pathlib.Path('FSM_PATH').read_text().splitlines()); assert r < 0.3*f"
  ```
- 3I.9 MEASURE: state coverage >80% of valid transition pairs, bugs found, determinism 100%, regression_count, ops/sec (target 100K — UNGROUNDED, measure empirically), compile_time_delta_sec (normal + madsim_compile_time_sec separately). Metrics file: `.elai_cc/metrics/phase_3I.toml`.

### Exit criteria
DST reference model matches actual FSM for 10,000+ random sequences. Same seed = identical output. BSP determinism verified with 100 identical runs. IPC fault injection passes or skip documented. VCR cassette replay produces identical SSB state. Prompt hash mismatch detected (not silently wrong). TraceOps trajectory exercises all FSM transitions. SQLite VFS spike completed (custom VFS OR madsim hooks proven sufficient).
NOTE: full INTERACTIVE HITL loop verified across 0-3, not by 3I alone. AUTONOMOUS auto-advance is Phase 4G.

### Grounding highlights
- madsim is NOT drop-in for tokio VERIFIED FALSE (cfg + dep swap).
- proptest-state-machine v0.8.0 same repo as proptest, released 2026-03-24.
- `FsmReferenceModel` as F(State,Event)->State GROUNDED external-analysis.txt L277.
- 10,000 random sequences GROUNDED external-analysis.txt L278.
- Digital sibling methodology GROUNDED ICSE 2024/2025 + TigerBeetle VOPR pattern.
- orchestrator.txt has NO DST section (SPEC_GAP); READ directive corrects to reference §4, §5, §15.

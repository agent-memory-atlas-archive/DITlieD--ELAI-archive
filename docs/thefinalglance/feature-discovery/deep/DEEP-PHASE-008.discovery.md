# DEEP-PHASE-008 exhaustive feature discovery

## Scope, integrity, and method

- Processed manifest: `docs/thefinalglance/feature-discovery/DEEP-SOURCE-MANIFEST.md:L1418-L1429`, heading `### DEEP-PHASE-008`.
- Declared chunk: family **PHASE**, **6 files**, **2,697 logical lines**, **162,325 bytes**, all six classified `numbered/index original`; target status **within 1,800–2,800**.
- Mechanical verification: fresh `sha256sum`, `wc -l`, and byte-size checks matched every manifest row and both aggregates exactly. Missing paths: **0**; unexpected paths: **0**; duplicate paths: **0**; SHA mismatches: **0**; line mismatches: **0**; byte mismatches: **0**.
- Read boundary: all six listed originals were read completely, first through last logical line. No original assigned to another deep chunk was opened. `01-phase-index-MASTER.discovery.md` was consulted only as an orientation and delta-comparison artifact; no source atom below is derived solely from it.
- Anchor notation refers to the manifest-verified source images: `3C:Lx-Ly`, `3E:Lx-Ly`, `4B:Lx-Ly`, `4F:Lx-Ly`, `4J:Lx-Ly`, and `IDX:Lx-Ly` (phase-index-pre0-2D).
- Repeated premise, revised-scope, wiring, grounding, composition, and acceptance language is collapsed into one atom with every load-bearing anchor. Interaction, ambiguity/conflict, and summary-delta records reference atoms and are not counted again as features.
- Classification: `E` = explicit source fact/requirement; `I` = bounded inference; `A` = unresolved ambiguity/choice; `C` = internal conflict or summary correction that must not be silently flattened. Domains: `STATUS`, `CAP`, `SURFACE`, `FLOW`, `STATE`, `DATA`, `CFG`, `INT`, `EVENT`, `SEC`, `PRIV`, `COMP`, `FAIL`, `RECOVERY`, `RETRY`, `DET`, `IDEM`, `CONC`, `OBS`, `TEST`, `LIMIT`, `DEP`, `DEFER`, `EVIDENCE`.

### Exact processed manifest and verification

| Manifest class | Lines | Bytes | SHA-256 | Exact absolute path | Verification |
|---|---:|---:|---|---|---|
| numbered/index original | 471 | 34,085 | 9882d9766a3264b45b4bc00f247ddda9228c29d6590ce4acd6c0d4806d353007 | docs/specs/guide/steps/phase-3C.txt | exact |
| numbered/index original | 366 | 20,014 | 68c72aecd1646f95783cfada9ad61df282c07233f00bab884064369738a78406 | docs/specs/guide/steps/phase-3E.txt | exact |
| numbered/index original | 429 | 24,664 | 8159bf3470c9f05bb2e7b8c78e64918f6d524fdf9ecc539bc3f05b7ea7e24cce | docs/specs/guide/steps/phase-4B.txt | exact |
| numbered/index original | 560 | 31,501 | c4ed8971681950477441c281e2e099811e2cb7a3985eb5e8d55e21a8d4b048fd | docs/specs/guide/steps/phase-4F.txt | exact |
| numbered/index original | 710 | 42,175 | 35fe9b4a6c60bfdfda1ecda476ab7d614d4aff07b34bd22d67db658de5a83e55 | docs/specs/guide/steps/phase-4J.txt | exact |
| numbered/index original | 161 | 9,886 | 93359d382f21eaeab9e0c710af2113551bbab04fd10ff8424774334ddd74f6a8 | docs/specs/guide/steps/phase-index-pre0-2D.txt | exact |
| **Aggregate** | **2,697** | **162,325** | — | **6 exact paths** | **exact** |

## Per-file atomic ledgers

### 3C — Agent system (pool, lifecycle, checkpoint, recovery, supervision, anti-spiral)

Source: `docs/specs/guide/steps/phase-3C.txt` (471/471 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| P3C-E001 | E/STATUS | Phase 3C goal is agent spawning, lifecycle, checkpointing, crash recovery, supervision tree, and anti-spiral escalation. Agents run in-process (Tier A) in Phase 3. | 3C:L7-L10 |
| P3C-E002 | E/DEFER | Process separation (Tier C) is deferred to Phase 4A. | 3C:L10-L10 |
| P3C-E003 | E/DEP | Prerequisites: Phase 3B SSB operational with single-writer guarantee. Status label IMPROVED, grounded against orchestrator/reasoning/architecture-overview/worker_boot. | 3C:L1-L3 |
| P3C-E004 | E/CAP | Transforms claw-code `WorkerRegistry` (~1069 LOC worker_boot.rs) and `recovery_recipes.rs` (~630 LOC) into architecture-overview AgentPool trait with checkpoint/recovery/supervision from orchestrator §7. | 3C:L12-L14 |
| P3C-E005 | E/DEP | Input from 3B: `ssb::write` for checkpoint/reasoning_trace; single-writer, append-only, seq monotonic. | 3C:L31-L32 |
| P3C-E006 | E/DEP | Input from 3A: `Fsm::get_state`/`transition`; phase-aware tool loading restricts tools per FSM state. | 3C:L33-L34 |
| P3C-E007 | E/DEP | Input from 2C/2A: `AgentGrcConfig` loaded at spawn; `GrcEngine::think` produces `ReasoningTraceEntry` to SSB. | 3C:L35-L38 |
| P3C-E008 | E/CAP | Output to 3D: `AgentPool::spawn` returns `AgentHandle`; `spawn_parallel` STUBBED sequential here, true parallel in 3D. | 3C:L41-L43 |
| P3C-E009 | E/CAP | Output to 3E: handle exposes `agent_id`, `role`, `active_files` for file-overlap scoring. | 3C:L44-L45 |
| P3C-E010 | E/FLOW | Output to 3F: agent tool calls route through orchestrator dispatch and `RuleEngine::evaluate` (no direct tool calls). | 3C:L46-L47 |
| P3C-E011 | E/CAP | Output to 4A: `AgentHandle` is IPC-ready; 3C wraps in-process, 4A wraps IPC socket with same interface. | 3C:L48-L49 |
| P3C-E012 | E/FLOW | Agents emit `ReplanRequest(evidence, severity)`: max 1 per agent per phase; SSB `action_type='replan_request'`; auditor validates evidence; MINOR fast-track to PLANNING; MAJOR human gate. | 3C:L50-L54 |
| P3C-E013 | E/DATA | Mapping: `WorkerStatus`→`AgentStatus` (add Checkpointing+BudgetExceeded; drop TrustRequired); `WorkerRegistry`→`AgentPool` with `HashMap<Uuid, AgentHandle>` via UUID v4 from EntropySource (not raw uuid crate). | 3C:L56-L62 |
| P3C-E014 | E/DET | Replace `now_secs()`/SystemTime with `ClockProvider::now_monotonic()` (entropy quarantine). | 3C:L65-L67 |
| P3C-E015 | E/DATA | `FailureScenario`/`RecoveryRecipe` map to death classification and `SupervisionPolicy.on_failure()`. | 3C:L68-L69 |
| P3C-E016 | E/CONC | R6: on single GPU (12GB), if model+embedding exceed VRAM, serialize agents (never parallel). VRAM check built into spawn. | 3C:L72-L74 |
| P3C-E017 | E/SEC | R31: Tier A invisible walls must return "not found" not "denied" for hidden paths. | 3C:L75-L77 |
| P3C-E018 | E/CAP | Six independently testable deliverables 3C.1–3C.6: pool/handle/status; lifecycle+death; checkpointing; recovery; supervision; anti-spiral. | 3C:L80-L86 |
| P3C-E019 | E/STATE | `AgentStatus`: Spawning, Ready, Running, Checkpointing, Finished, Failed. Drop TrustRequired; rename ReadyForPrompt→Ready; add Checkpointing. | 3C:L100-L104 |
| P3C-E020 | E/DATA | `AgentHandle` fields: `agent_id: Uuid`, `role: AgentRole`, `status`, `active_files: Vec<PathBuf>`, `grc_config: AgentGrcConfig`, `scratch_dir: PathBuf`. | 3C:L105-L110 |
| P3C-E021 | E/CFG | `grc_config` loaded from `elai_cc.toml [agents.{role}]` at spawn. | 3C:L110-L110 |
| P3C-E022 | E/DATA | `AgentRole` (8): Executor, Auditor, PromptEngineer, ContextCompiler, MemoryWriter, Researcher, MediaValidator, Designer. | 3C:L111-L113 |
| P3C-E023 | E/CAP | Designer produces PipelineSpec from Research Report; not Executor; distinct role_prompt/required_steps/output; reuses executor model (Gemma 4 31B) with separate AgentGrcConfig. | 3C:L114-L116 |
| P3C-E024 | E/CAP | `AgentPool` trait: `spawn(role,task)->AgentHandle`; `spawn_parallel` sequential stub (not unimplemented!); `communicate` direct in-process stub (IPC in 4A). | 3C:L117-L120 |
| P3C-E025 | E/FAIL | spawn VRAM check via `ModelPool::available_vram()` (never nvml direct); insufficient → `ResourceError` (not panic). | 3C:L121-L124 |
| P3C-E026 | E/TEST | Adversarial: Executor budget 8/1200, MemoryWriter 2/400 at spawn; spawn_parallel sequential; ResourceError on VRAM=0. | 3C:L131-L137 |
| P3C-E027 | E/STATE | Lifecycle SM: Spawning→Ready→Running→Finished|Failed; Phase 3 agent is tokio task. | 3C:L162-L164 |
| P3C-E028 | E/DATA | `AgentDeathClassification`: CLEAN_EXIT, CRASH, TIMEOUT (15s heartbeat silence), BUDGET_EXCEEDED. | 3C:L165-L169 |
| P3C-E029 | E/FLOW | Heartbeat every 5s via `tokio::sync::watch`; 15s silence = TIMEOUT; Tier C replaces with IPC AgentHeartbeat. | 3C:L170-L172 |
| P3C-E030 | E/SEC | Tier A visibility filter: per-agent `allowed_paths: Vec<GlobPattern>`; hidden default: `.elai_cc/`, rules.db, harness source, other agent checkpoints, SSB db, ONNX models, binary internals; returns NotFound. | 3C:L173-L177 |
| P3C-E031 | E/FLOW | ReplanRequest stub: counter on AgentHandle; second call `ReplanDenied`; MINOR→auditor then EXECUTING→PLANNING; MAJOR→INTERACTIVE human approval. | 3C:L178-L182 |
| P3C-E032 | E/EVENT | Every GRC think writes `ReasoningTraceEntry` to SSB: agent_id, step_index, step_type, content_summary max 200 chars, grounding_source/level, confidence, elapsed_ms, topic, budget_remaining. | 3C:L183-L186 |
| P3C-E033 | E/DATA | SSB schema migration may be required for action_types: reasoning_trace, isolation_metric, spiral_detected, supervision_exhausted (section 3 list omits them). | 3C:L154-L160 |
| P3C-E034 | E/DATA | Checkpoint path `.elai_cc/sessions/{sid}/agents/{agent_id}/` with five files: state/grc_history/tool_results/output_draft `.msgpack` + plain `checkpoint_ts` via ClockProvider. | 3C:L227-L235 |
| P3C-E035 | E/FLOW | Auto checkpoint triggers: after every GRC think, tool result, file edit confirmed; before slow ops (latency >1s). | 3C:L236-L240 |
| P3C-E036 | E/RECOVERY | Atomic checkpoint write: temp+fsync+rename; Windows delete-old-then-rename same-volume. | 3C:L241-L243 |
| P3C-E037 | E/EVENT | SSB checkpoint event payload is metadata only (agent_id, step_index, file sizes), not checkpoint body. | 3C:L244-L245 |
| P3C-E038 | E/SEC | Checkpoint serializer rejects UntrackedValue channel IDs with `CheckpointError::UntrackedValueReference`; LastValue/Topic allowed. | 3C:L246-L253 |
| P3C-E039 | E/CFG | If tool_results grows unbounded, rolling window default keep last 50 (configurable); target write <100ms after 100 tool calls. | 3C:L269-L271 |
| P3C-E040 | E/RECOVERY | Recovery: read latest checkpoint, spawn same role+agent_id new task, inject structured context, continue from last GRC step; first recovered step grounding_source=`CHECKPOINT:{path}`. | 3C:L291-L310 |
| P3C-E041 | E/DATA | Recovery recipe map: PromptMisdelivery/StaleBranch/CompileRedCrossCrate/ProviderFailure→CRASH; TrustPromptUnresolved/McpHandshakeFailure/PartialPluginStartup→N/A. | 3C:L300-L307 |
| P3C-E042 | E/RECOVERY | Corrupt checkpoint: degrade to fresh start with empty history + WARNING SSB (not permanent stuck). | 3C:L318-L320 |
| P3C-E043 | E/DET | Recovered agent_id unchanged; SSB step_index continues monotonically (no reset). | 3C:L308-L308, L321-L323 |
| P3C-E044 | E/IDEM | Recovered first action is NEXT GRC step (no replay of completed tool side effects). | 3C:L324-L326 |
| P3C-E045 | E/TEST | CRASH recovery resume <2s; TIMEOUT within heartbeat_timeout+2s. | 3C:L328-L329 |
| P3C-E046 | E/CFG | SupervisionPolicy: max_restarts default 3 (per role elai_cc.toml [agents.{role}.supervision]); restart_window = current phase duration; on_max_restarts Escalate | SubstituteModel(ModelId). | 3C:L343-L347 |
| P3C-E047 | E/CFG | Phase 3 INTERACTIVE: Escalate; Phase 4G AUTONOMOUS: SubstituteModel; Phase 3 enum has SubstituteModel but gated (error if used INTERACTIVE). | 3C:L348-L351 |
| P3C-E048 | E/STATE | restart_count per agent per phase; reset 0 at phase boundary. | 3C:L352-L352 |
| P3C-E049 | E/FLOW | CLEAN_EXIT no restart; CRASH/TIMEOUT restart+increment; BUDGET_EXCEEDED inject convergence, no restart, no count increment. | 3C:L353-L357 |
| P3C-E050 | E/EVENT | On max restarts: stop, checkpoint final, execute on_max_restarts, log supervision_exhausted with full history. | 3C:L358-L360 |
| P3C-E051 | E/CFG | max_restarts <1 in config clamps to 1 with WARNING. | 3C:L375-L376 |
| P3C-E052 | E/FLOW | Anti-spiral: track last N GRC steps; same_step_count >3 (same step_type+topic+similar content) = SPIRAL. | 3C:L394-L397 |
| P3C-E053 | E/DET | Similar content = blake3 of normalized content (lowercase, whitespace-collapsed); B-16A supersedes prior blake2s. | 3C:L398-L401 |
| P3C-E054 | E/FLOW | On SPIRAL: (1) checkpoint (2) inject convergence status step (3) another identical step → permanent stop (not restart) (4) SSB spiral_detected with 3 steps evidence. | 3C:L402-L408 |
| P3C-E055 | E/FLOW | Spiral stop is supervised stop not crash; on_spiral same as on_max_restarts (Escalate INTERACTIVE). | 3C:L409-L410 |
| P3C-E056 | E/SURFACE | INTERACTIVE spiral escalation UI text: "Agent {role} spiraling on {topic}: repeated {step_type} 3x. Evidence: [3 steps]. Options: [retry with hint] [substitute model] [skip step] [abort task]". | 3C:L411-L413 |
| P3C-E057 | E/RECOVERY | Spiral tags pre-spiral checkpoint (N-3) as recovery_target; recovery loads pre-spiral not latest. | 3C:L428-L433 |
| P3C-E058 | E/DEFER | Tier B sandbox (Firecracker/gVisor) not in 3C: bash has only Tier A path filter; known security gap until 4A. | 3C:L446-L451 |
| P3C-E059 | E/DEFER | isolation_metric aggregation at phase boundary is 3A/3D scope; 3C only supplies per-agent budget_remaining data. | 3C:L452-L456 |
| P3C-E060 | E/OBS | Measures: checkpoint write <50ms; recovery <2s; checkpoint size <1MB/agent; heartbeat overhead <0.1% CPU; anti-spiral <1ms/step; memory overhead per agent logged. | 3C:L458-L465 |
| P3C-E061 | E/LIMIT | No lazy stubs: unimplemented!() is build failure; spawn_parallel sequential is valid stub. | 3C:L129-L129, L471 |
| P3C-I001 | I/SURFACE | HUMAN-facing surfaces in 3C are INTERACTIVE escalation panels for max-restarts and spiral (options listed); no dedicated dashboard panel is named. | 3C:L348-L349, L411-L413 |
| P3C-A001 | A/CFG | Rolling-window N for tool_results default 50 is given only as adversarial mitigation when write exceeds 100ms; whether always-on is required is unclear. | 3C:L269-L271 |
| P3C-A002 | A/DATA | Exact SSB CHECK constraint migration mechanism (ALTER vs recreate) left as implementer choice. | 3C:L157-L160 |
| P3C-A003 | A/FLOW | "last N GRC steps" window size for spiral tracking is not numerically fixed beyond the >3 consecutive trigger. | 3C:L395-L397 |

Category coverage: capabilities/actions/surfaces ✓; workflows/states ✓; schemas/data/lifecycle ✓; defaults/config ✓; integrations/events ✓; security ✓; failures/rollback ✓; determinism/concurrency ✓; observability ✓; tests/done gates ✓; constraints/deferred ✓. No formal privacy or compliance rule.

### 3E — Context budget allocator

Source: `docs/specs/guide/steps/phase-3E.txt` (366/366 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| P3E-E001 | E/STATUS | Goal: fixed-slot context budget so model context is never exceeded; adaptive reduction with floors; cache breakpoints for provider prefix caching. | 3E:L7-L10 |
| P3E-E002 | E/DEP | Prerequisites: Phase 3D BSP+channels; depends on providers token_count/token_count_estimate, SSB, BSP. | 3E:L2-L3 |
| P3E-E003 | E/DATA | `ContextSlot` (8): SystemPrompt, Rules, PhaseSpec, RepoMap, CodeContext, SsbEntries, GrcHistory, OutputReserve. | 3E:L21-L30 |
| P3E-E004 | E/DATA | `SlotAllocation`: slot, allocated_tokens, floor_tokens (0 for slots 1-3), actual_tokens, source_truncated. | 3E:L32-L38 |
| P3E-E005 | E/DATA | `ContextBudget`: model_max_context from ModelCapabilityRegistry; slots[8]; cache_breakpoints[2]; total_allocated/used; adaptive_level. | 3E:L40-L48 |
| P3E-E006 | E/DATA | `CacheBreakpoint`: after_slot (BP1 after PhaseSpec, BP2 after CodeContext), prefix_hash u64, valid bool. | 3E:L49-L53 |
| P3E-E007 | E/STATE | `AdaptiveLevel`: Normal, FilteredByRelevance, ReducedGranularity, ReducedScope, Terminal. | 3E:L55-L61 |
| P3E-E008 | E/FLOW | Allocation uses token_count_estimate; final validation uses exact token_count; exact exceed triggers adaptive; hard truncate never silent. | 3E:L66-L72 |
| P3E-E009 | E/OBS | Full budget computation latency target <50ms. | 3E:L73-L73 |
| P3E-E010 | E/FLOW | Fixed slot order never reordered: 1 SystemPrompt, 2 Rules, 3 PhaseSpec, BP1, 4 RepoMap, 5 CodeContext, BP2, 6 SsbEntries, 7 GrcHistory, 8 OutputReserve. | 3E:L78-L88 |
| P3E-E011 | E/FLOW | BP1 invalidated by phase transition, role change, rule set change; BP2 by file set change; slots 6-8 recompute every invocation. | 3E:L90-L93 |
| P3E-E012 | E/INT | When BP1/BP2 valid, provider reuses cached KV for slots 1-3 / 1-5 via CacheBreakpoint.valid. | 3E:L95-L98 |
| P3E-E013 | E/CFG | Dynamic percentages: RepoMap 15%, CodeContext 40%, SsbEntries 25%, GrcHistory 20% after fixed 1-3 exact and OutputReserve subtracted. | 3E:L103-L113 |
| P3E-E014 | E/CFG | Floors: SsbEntries≥200, GrcHistory≥500, OutputReserve≥2000; dynamic_budget < sum(floors) terminal. | 3E:L115-L119 |
| P3E-E015 | E/FLOW | File overlap score = |intersection(entry.files, agent.files)| / |agent.files|; sort score DESC then seq DESC; score 0.0 excluded. | 3E:L124-L128 |
| P3E-E016 | E/FLOW | Budget recomputed per agent invocation in BSP superstep; called after prepare_next_tasks, before execute_tasks. | 3E:L133-L139 |
| P3E-E017 | E/DATA | Channel feed to slot 6: LastValue if overlap>0; Topic filtered by overlap; EphemeralValue only current superstep; UntrackedValue excluded. | 3E:L134-L138 |
| P3E-E018 | E/EVENT | Adaptive reduction max 3 attempts; each logs SSB action_type='context_reduction' with before/after tokens. | 3E:L144-L145 |
| P3E-E019 | E/FLOW | Attempt1 FilteredByRelevance: raise overlap threshold (e.g. 0.3), GRC current-topic only. | 3E:L147-L150 |
| P3E-E020 | E/FLOW | Attempt2 ReducedGranularity: denser PageRank, code→signatures+doc comments. | 3E:L152-L155 |
| P3E-E021 | E/FLOW | Attempt3 ReducedScope: current file+direct deps map; current file code; agent's own SSB only. | 3E:L157-L161 |
| P3E-E022 | E/FAIL | Terminal: INTERACTIVE ESCALATE with budget breakdown; AUTONOMOUS SPLIT_TASK (Phase 4G). Never silent truncate. | 3E:L163-L166 |
| P3E-E023 | E/DATA | Prompt assembly map: slots 1→role_prompt, 2→tier_3_rules, 4+5 merge→code_context, 3→task_spec+output_format+constraints; 6-7 structured after task_spec; 8 not in prompt. | 3E:L171-L186 |
| P3E-E024 | E/FLOW | Session handoff CMV trimming preserves BP1 slots 1-3; BP2 recomputed on new session (do not assume BP2 survives). | 3E:L191-L197 |
| P3E-E025 | E/CFG | OutputReserve: max(configured, 2000) subtracted before percentage split. | 3E:L229-L229 |
| P3E-E026 | E/DET | Remainder after floor division assigned to largest slot (CodeContext); sum all slots == model_max_context. | 3E:L231-L238 |
| P3E-E027 | E/FLOW | Floor steal order: CodeContext first, then RepoMap; both insufficient → Err(BudgetInfeasible); dynamic <700 → BudgetInfeasible. | 3E:L250-L254 |
| P3E-E028 | E/OBS | If CodeContext <200 after steal, log warning to SSB (not error). | 3E:L261-L261 |
| P3E-E029 | E/DET | prefix_hash uses deterministic hasher (xxhash/FxHash), never DefaultHasher. | 3E:L51-L51, L281-L281 |
| P3E-E030 | E/DET | Empty agent_files → overlap score 0.0 (no division by zero). | 3E:L302-L304 |
| P3E-E031 | E/OBS | 10k SSB entries filtering target <10ms. | 3E:L305-L305 |
| P3E-E032 | E/EVENT | Reduction logging writes SSB but does NOT enter current invocation's context (visible next turn only). | 3E:L327-L327 |
| P3E-E033 | E/TEST | Integration: Gemma 4 E2B 128K context; test 4096 and 131072; assembly <50ms via ClockProvider; 100x byte-identical. | 3E:L337-L348 |
| P3E-E034 | E/LIMIT | No Instant::now in context budget module (entropy quarantine). | 3E:L351-L351 |
| P3E-E035 | E/SURFACE | ContextBudget implements Display for debug-friendly budget breakdown; INTERACTIVE terminal surfaces Escalate(breakdown) to user. | 3E:L210-L210, L163-L164, L324 |
| P3E-E036 | E/TEST | 3E.1–3E.7 steps: types, compute, floors, cache breakpoints, SSB filter, adaptive, e2e integration. | 3E:L200-L355 |
| P3E-I001 | I/CFG | Configurable overlap raise "e.g. 0.3" is example not hard default for attempt 1. | 3E:L148-L148 |
| P3E-A001 | A/CFG | Exact default values for model_max_context sources and fixed-slot measurement sites left to ModelCapabilityRegistry/runtime. | 3E:L41-L41, L226-L227 |
| P3E-A002 | A/SURFACE | No dedicated cockpit/panel for budget is named; only Display + escalate breakdown. | 3E:L210, L163-L164 |

Category coverage: slots/allocation/floors/cache/adaptive/SSB filter/BSP timing/handoff/tests ✓. No standalone privacy rule.

### 4B — Tools (git, validation, DAIRA, edit match, ShortCoder, taxonomy, file read, incr. compile)

Source: `docs/specs/guide/steps/phase-4B.txt` (429/429 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| P4B-E001 | E/STATUS | Prerequisites: Phase 4A agent isolation + IPC. Goal: full tool suite in orchestrator; agents access via IPC ToolRequest/ToolResult. | 4B:L1-L2, L17-L21 |
| P4B-E002 | E/STATUS | Recorded 2026-04-16 commit 55ece36: 4B.1–4B.4 DONE (10 tool modules, IpcToolRegistry, 15 typed arg structs, 19/19 tests; 3 BLOCKs fixed). | 4B:L3-L7 |
| P4B-E003 | E/DEFER | 4B.5 MEASURE deferred (metrics remaining). 4B.6–4B.7 IntegrationTestRunner deferred blocked on 4E.7+ types. | 4B:L8-L10 |
| P4B-E004 | E/LIMIT | Scope exclusions: VisibilityFilter + schema sanitization = 4A; BashSandbox inherited not implemented; lsp_client evaluation deferred Phase 5 if not in MEASURE. | 4B:L39-L46 |
| P4B-E005 | E/FAIL | R1: IPC schema drift mitigated by postcard round-trip contract tests per ToolRequest variant. | 4B:L51-L54 |
| P4B-E006 | E/FAIL | R2: compilation on written file only; unresolved import of unwritten module → DEFERRED not COMPILATION_ERROR. | 4B:L55-L59 |
| P4B-E007 | E/CFG | R3: tools/ and knowledge/ pin identical tree-sitter grammar versions workspace-level. | 4B:L60-L63 |
| P4B-E008 | E/CONC | R4: ToolRegistry checks SSB file locks before write tools (no two agents write same file). | 4B:L64-L67 |
| P4B-E009 | E/DATA | ToolRequest variants: GitCommit/GitUndo/GitBranch, ValidateRequest, TraceRequest, EditRequest(+tier hint), SimplifyRequest, ClassifyRequest, FileReadRequest(+force_full), CompileCheckRequest. | 4B:L81-L86 |
| P4B-E010 | E/CFG | Tools registered with phase-aware filtering; PLANNING must not expose EditRequest. | 4B:L87-L88, L96-L97 |
| P4B-E011 | E/CAP | git2: auto-commit deterministic message from phase outcome; branch plan-{pipeline_id}; WIP commits for >5 edits; session-owned undo. | 4B:L177-L180 |
| P4B-E012 | E/CAP | 5-tier validation: T1 determinism → T2 quality/AST → T3 unit → T4 integration → T5 stress; auditor only, executor never runs own tests. | 4B:L181-L184 |
| P4B-E013 | E/CAP | DAIRA markers: CALL, RETURN, MUTATION, BRANCH, ERROR, PANIC, TIMEOUT, FINDING, PATTERN, ANOMALY, STAT:name, DURATION:name, MEMORY:name, CHECKPOINT, STEP:n; Rust tracing + Python sys.settrace. | 4B:L185-L190 |
| P4B-E014 | E/CAP | AdaptiveTool<T> budget-based scope reduction max 3 then error. | 4B:L189-L190, L237-L239 |
| P4B-E015 | E/OBS | MarkerReport includes raw_line_count and marker_line_count (ratio drives AdaptiveTool). | 4B:L132-L133 |
| P4B-E016 | E/CAP | 3-tier edit match: exact → whitespace-normalized (preserve indent) → ellipsis-elision; on fail SequenceMatcher similarity hint; never silent wrong match. | 4B:L191-L195 |
| P4B-E017 | E/CAP | ShortCoder: 10 tree-sitter rules R1–R10; R4/R6 configurable; post-process executor output before auditor. | 4B:L196-L198 |
| P4B-E018 | E/CAP | Failure taxonomy 9 categories: SYNTAX_ERROR, TYPE_ERROR, IMPORT_ERROR, ASSERTION_ERROR, TIMEOUT, EMPTY_OUTPUT, RUNTIME_PANIC, PERMISSION_DENIED, RESOURCE_EXHAUSTED; deterministic suggested_action; auditor reads before raw; 3+ same category/phase → anti-spiral. | 4B:L199-L204 |
| P4B-E019 | E/CAP | File read: outline mode >300 lines (tree-sitter sections); force_full bypass; budget guard warn if file >50% remaining context; INTERACTIVE user decides; AUTONOMOUS auto-downgrade outline. | 4B:L205-L208 |
| P4B-E020 | E/CAP | Incremental compile ASYNC after each EXECUTING write: cargo check/ruff/tsc/go vet on written file; results via IPC; max 3 rounds (2 retries); then NEEDS_AUDIT; skip TESTING_FIRST. | 4B:L209-L218 |
| P4B-E021 | E/SEC | IPC error messages sanitized: strip rule_id, predicate types, hook names. | 4B:L220-L222 |
| P4B-E022 | E/OBS | Metrics path `.elai_cc/metrics/phase_4B.toml` keys: ipc_overhead_ms, compile_check_latency_{rust,python,typescript,go}_ms, shortcoder_savings_pct, test_count, loc_tools. | 4B:L274-L277 |
| P4B-E023 | E/OBS | IPC dispatch overhead target <2ms p95; measure 100 calls p50/p95/p99. | 4B:L256-L260, L280-L281 |
| P4B-E024 | E/OBS | Compile latency expectations: Rust 2-5s, Python ruff <500ms, tsc <2s, go vet <1s (non-Rust marked UNGROUNDED until measured). | 4B:L261-L265 |
| P4B-E025 | E/OBS | Auditor BLOCK reduction ~30% is UNGROUNDED design estimate. | 4B:L266-L270 |
| P4B-E026 | E/OBS | ShortCoder paper claims 18-38% savings; measure on 10 representative samples. | 4B:L271-L272 |
| P4B-E027 | E/DEP | Papers daira-interface.md and shortcoder-interface.md required ingested before TESTING_FIRST. | 4B:L34-L36, L78-L79 |
| P4B-E028 | E/CAP | IntegrationTestRunner levels: BabyStep (<5s cargo check), Step (<30s module tests), Phase (<60s crate tests), Concept (<120s workspace concept_tag). | 4B:L306-L321 |
| P4B-E029 | E/DATA | IntegrationResult: level, passed, test_count, failures, stdout_hash [u8;32], elapsed_ms. | 4B:L329-L336 |
| P4B-E030 | E/CFG | tool_id="integration_test" available in AUDITING only; NOT during EXECUTING; orchestrator-triggered not agent-triggered. | 4B:L344-L346 |
| P4B-E031 | E/INT | Integration uses BashSandbox, FailureTaxonomy, IPC ToolResult, BABY_STEP uses incr. compile infrastructure. | 4B:L338-L342 |
| P4B-E032 | E/LIMIT | Every request-routing site must wire ssb, fsm, clock through single RuntimeClient (no None fields post-construction). | 4B:L298-L298 |
| P4B-E033 | E/TEST | ≥15 test functions; step ladder skeleton→verify→failing tests→implement→measure→integration. | 4B:L71-L170, L252-L287 |
| P4B-E034 | E/DET | git2 timestamps from ClockProvider not system time. | 4B:L232-L233 |
| P4B-E035 | E/FLOW | Validation tiers must short-circuit: tier-1 failure stops tiers 2-5. | 4B:L234-L236 |
| P4B-E036 | E/TEST | Incremental compile event-ordering test: next ToolRequest before mock compile completes. | 4B:L240-L243 |
| P4B-E037 | E/TEST | Supported languages for incr. compile tests: Rust, Python, TS, Go (one each). | 4B:L165-L166 |
| P4B-I001 | I/SURFACE | Tools are agent-facing IPC surfaces, not human UI; human sees outcomes via auditor/SSB/dashboard elsewhere. | 4B:L17-L18 |
| P4B-A001 | A/STATUS | Whether papers remain MISSING at time of this discovery is runtime state; plan still requires them as PRE_GATE. | 4B:L35-L36 |
| P4B-A002 | A/CFG | Exact deterministic git commit message format template not specified beyond "from phase outcome". | 4B:L178-L178 |
| P4B-C001 | C/STATUS | Header completion status says 4B.1-4.4 DONE while body still lists full TDD ladder; status is historical record, steps remain normative for residual/measure work. | 4B:L3-L10, L71-L287 |

Category coverage: tools/IPC/async compile/integration runner/metrics/security sanitization ✓.

### 4F — Feasibility probe

Source: `docs/specs/guide/steps/phase-4F.txt` (560/560 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| P4F-E001 | E/STATUS | Prerequisites: Phase 4E ExecutablePipeline. Goal: system probe + novel-step stubs (<50 lines) with branch-and-explore; skip when zero novel; escalate after max 3 alternatives; output stub_results.msgpack. | 4F:L1-L11 |
| P4F-E002 | E/DATA | SystemProbeResult: gpu_count, gpu_info, ram_total/free, disk_free, os, is_supported. | 4F:L28-L36 |
| P4F-E003 | E/DATA | GpuInfo: index, name, total/free bytes, driver_version, uuid (stable NVML). | 4F:L38-L45 |
| P4F-E004 | E/DATA | SystemConstraints: min_vram/ram/disk bytes, required_gpu_count. | 4F:L47-L52 |
| P4F-E005 | E/DATA | StubResult: step_id, alternative_index 0-2, FeasibilityOutcome, elapsed_ms, stdout_hash blake3, stderr_summary first 512 bytes, ResourceDelta. | 4F:L54-L62 |
| P4F-E006 | E/DATA | FeasibilityReport: probe, constraints, constraint_results, stubs, skipped_steps, escalations. | 4F:L71-L78 |
| P4F-E007 | E/DATA | ConstraintResult: resource string, required, available, margin, passed (margin≥0 met; margin≥0.20 skip threshold). | 4F:L80-L86 |
| P4F-E008 | E/DATA | FeasibilityEscalation: step_id, reason, stubs_tried (3 fails). | 4F:L88-L92 |
| P4F-E009 | E/STATE | FeasibilityDecision: Proceed, ProceedWithConstraints(Vec), Escalate(Vec) INTERACTIVE, Replan(Vec) AUTONOMOUS. | 4F:L94-L99 |
| P4F-E010 | E/DATA | Serialization: msgpack disk, postcard IPC; FeasibilityOutcome reused from reasoning (2D). | 4F:L101-L103 |
| P4F-E011 | E/FLOW | Skip when novel_steps empty AND all VRAM/RAM/disk free ≥ required*1.20; else run probe. | 4F:L107-L115 |
| P4F-E012 | E/EVENT | Skip logs SSB action_type='phase_transition' with skip_reason; FSM PLANNING→TESTING_FIRST. | 4F:L116-L118 |
| P4F-E013 | E/FAIL | No NVIDIA: Nvml::init Err → gpu_count=0, empty gpu_info. | 4F:L122-L124 |
| P4F-E014 | E/OBS | Keep single sysinfo System; refresh_specifics only needed categories. | 4F:L126-L128 |
| P4F-E015 | E/FLOW | Re-check VRAM via ModelPool.available_vram immediately before stub launch. | 4F:L130-L133 |
| P4F-E016 | E/SEC | Stubs Tier C with FEASIBILITY tools only: stub write + bash + web search; NO edit/git. | 4F:L139-L141 |
| P4F-E017 | E/CAP | FeasibilityProbe trait: probe_system, check_constraints, run_stub, decide; object-safe &self. | 4F:L157-L161 |
| P4F-E018 | E/CFG | Feasibility AgentDispatchConfig overrides: GRC 3/600; allowed_tools [stub_write,bash]; sandbox 25s graceful/30s hard; line limit strict <50. DispatchStage.agent AUTHORITATIVE. | 4F:L164-L172 |
| P4F-E019 | E/INT | IPC reuse 4A: 1-byte version + 4-byte length + postcard. | 4F:L173-L173 |
| P4F-E020 | E/EVENT | SSB branch per stub alternative tag="branch:stub_{step_id}_{n}"; max 3; merge winner discard losers. | 4F:L175-L175, L377-L380 |
| P4F-E021 | E/TEST | 16 contract tests T1–T16 covering probe, constraints, stub pass/reject/timeout, max3 escalate, skip matrix, msgpack, sandbox deny edit, VRAM recheck, dual timeout, ProceedWithConstraints, malformed IPC. | 4F:L228-L281 |
| P4F-E022 | E/TEST | 8 adversarial A1–A8: fs escape, network, malformed frame, concurrent GPU, no-GPU CI, oversized frame, VRAM race, 4th branch escalate. | 4F:L305-L323 |
| P4F-E023 | E/LIMIT | Stub ≥50 lines rejected before execution (strict <50). | 4F:L243-L245, L368 |
| P4F-E024 | E/FLOW | Dual timeout: graceful shutdown IPC at 25s; hard kill at 30s with explicit child.kill (kill_on_drop false). | 4F:L269-L273, L369-L374 |
| P4F-E025 | E/FAIL | Malformed IPC → StubResult Fail with stderr_summary containing "malformed_ipc". | 4F:L278-L281 |
| P4F-E026 | E/FAIL | VRAM drop pre-launch → Fail reason="vram_dropped" / "insufficient_vram". | 4F:L266-L268, L319-L321 |
| P4F-E027 | E/CFG | MAX_FRAME_SIZE 10MB for FeasibilityReport postcard. | 4F:L189-L190 |
| P4F-E028 | E/OBS | Metrics phase_4F.toml: system_probe_latency_ms (<500), stub p50/p99, constraint_check_latency_us (<100), report_serialization_bytes, skip_criteria_evaluation_us, test_count/pass, compile_time_delta_s. | 4F:L416-L425 |
| P4F-E029 | E/CAP | Baby-step extension: novel if PackageRef.verified=false OR unused framework OR untested capability (GPU/FFI). | 4F:L488-L491 |
| P4F-E030 | E/FLOW | Feasibility per concept before concept N execution (not all upfront). | 4F:L493-L496 |
| P4F-E031 | E/DATA | BabyStepFeasibilityReport / BabyStepStubResult / BabyStepEscalation types. | 4F:L503-L520 |
| P4F-E032 | E/FLOW | Per-concept skip when zero novel baby steps. | 4F:L498-L499 |
| P4F-E033 | E/DEP | Crate deps: nvml-wrapper, sysinfo, blake3 (if missing); rmp-serde already present. | 4F:L461-L465 |
| P4F-E034 | E/CFG | E-22 note: sysinfo dep may be rejected; RAM/disk via wmic//proc/meminfo/df + ELAI_FREE_RAM_GB/ELAI_FREE_DISK_GB fallbacks; nvml-wrapper activated non-optional. | 4F:L339-L346 |
| P4F-E035 | E/LIMIT | RuntimeClient ssb/fsm/clock wiring required on all request-routing structs. | 4F:L475-L475 |
| P4F-E036 | E/DET | Constraint check pure (no I/O); stdout blake3 deterministic for identical bytes. | 4F:L445-L456 |
| P4F-I001 | I/SURFACE | INTERACTIVE Escalate decision is human-facing; no dedicated panel name beyond decision enum. | 4F:L97-L98 |
| P4F-A001 | A/CFG | Whether web_search is always in FEASIBILITY tool set conflicts slightly: skeleton says stub_write+bash only; I3 text adds web search. | 4F:L167-L167, L366-L367 |
| P4F-C001 | C/DATA | Trait named FeasibilityProbe in TASK but greps look for FeasibilityProber; SD2 notes flat feasibility_probe.rs layout vs subdir. | 4F:L157-L184 |
| P4F-C002 | C/DEP | Body still specifies sysinfo API, while E-22 pre-gate rejects sysinfo as new dep and uses shell-out fallbacks. | 4F:L353-L360, L339-L344 |

Category coverage: probe/skip/stubs/timeouts/branches/decisions/baby-step/metrics/security ✓.

### 4J — Empire integration

Source: `docs/specs/guide/steps/phase-4J.txt` (710/710 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| P4J-E001 | E/STATUS | Conditional phase: only when Empire codebase accessible/deployed. Prereqs 4A, 3B, 4D, 4G. | 4J:L1-L8 |
| P4J-E002 | E/STATUS | EMPIRE-PENDING gate (2026-05-22): Empire not accessible; ship host scaffolding now with EMPIRE-PENDING labels; live activation one-line flip. | 4J:L12-L19 |
| P4J-E003 | E/CFG | Implementation at crates/orchestrator/src/empire/* under cfg(feature="empire"); not crates/empire; cargo check -p elai-orchestrator --features empire. | 4J:L21-L28 |
| P4J-E004 | E/EVENT | SSB action_type granularity = 5 variants: empire_connected, empire_disconnected, empire_event_received, empire_capability_invoked, empire_repair_proposed (not single empire_event). | 4J:L30-L37 |
| P4J-E005 | E/DATA | Empire IPC namespace empire::ipc::EmpireIpcMessageType (do not shadow elai_ipc_types::IpcMessageType 16 locked variants). | 4J:L39-L43 |
| P4J-E006 | E/CAP | Goal: speak FactoryLine, register as brain, subscribe Empire events as live GRC/SSB source, expose neuron capabilities, replace brainctl. | 4J:L45-L50 |
| P4J-E007 | E/INT | Empire topology: EngineMKI→MKII→MKIII; IPC sock /tmp/brain-engine.sock; FactoryLine /tmp/brain-factory.sock; newline JSON AF_UNIX; max 65536 IPC; FSB 12-byte header, 256KB max frame. | 4J:L52-L59 |
| P4J-E008 | E/DATA | FactoryLine cmds: register|heartbeat|list|request_update protocol v:1. | 4J:L64-L65 |
| P4J-E009 | E/DATA | Engine handlers MKI: REGISTER_HOOK, EMIT, QUERY_HOOKS, SUBSCRIBE, UNSUBSCRIBE, BUS_CREATE, BUS_JOIN, BUS_LEAVE, BUS_LIST. | 4J:L71-L72 |
| P4J-E010 | E/FAIL | Exactly 3 verified broken issues: brainctl↔brain_lifecycle circular; EngineMKII hard-import slc; unique-factory/types stdlib shadow. | 4J:L73-L78 |
| P4J-E011 | E/LIMIT | Dead code supersede: workforce lambda stubs; unvalidated brain.json superseded by PipelineSpec 6 layers. MetricsStore/SeedState alive. | 4J:L79-L83 |
| P4J-E012 | E/CAP | FactoryLineTransport trait: connect, health_check; TransportConfig Unix|Tcp; UnixTransport cfg(unix), TcpTransport all. | 4J:L132-L138 |
| P4J-E013 | E/CFG | FactoryLineConfig: transport, heartbeat_interval, reconnect_backoff ExponentialBackoff, event_rate_limit. | 4J:L146-L147 |
| P4J-E014 | E/DATA | FactoryLineCommand/FactoryCmd/FactoryLineResponse; IpcMessage/EmpireIpcMessageType (9 kinds)/IpcResponse; shared Event/Command/Response. | 4J:L152-L173 |
| P4J-E015 | E/DATA | BrainManifest with schema validation: brain_id, version, engine_tier MKI|MKII|MKIII, neurons, capabilities, heartbeat_interval_ms. | 4J:L175-L184 |
| P4J-E016 | E/CAP | EmpireEventBridge: ssb_writer, rate_limiter (always present not Option), EventFilter patterns+priority_map High|Normal|Low. | 4J:L186-L194 |
| P4J-E017 | E/CAP | ElaiNeuron exposes only task_submit, task_status, pipeline_result, health_check; never FSM/evidence/rules/SSB-direct/config/agent_spawn. | 4J:L196-L202, L409-L411 |
| P4J-E018 | E/DATA | EmpireIssue CircularDep|HardImport|StdlibShadow; FixType Refactor|Replace|Remove; repair via 4D pipeline factory. | 4J:L204-L212 |
| P4J-E019 | E/SEC | Empire-originated requests enter IPC proxy ToolFilter; cannot trigger FSM transitions or bypass gates. | 4J:L113-L117 |
| P4J-E020 | E/FLOW | Heartbeat at configured interval; 3 consecutive misses → reconnection backoff; reconnect re-sends Register. | 4J:L304-L308, L538-L539 |
| P4J-E021 | E/LIMIT | Zero brainctl imports; pure Rust FactoryLine client; zero pyo3/cpython. | 4J:L105-L108, L344-L351 |
| P4J-E022 | E/FLOW | Rate limit global (not per-pattern); high-priority bypass elevated but bounded (e.g. 100/sec not unlimited); dropped_count metrics. | 4J:L360-L389 |
| P4J-E023 | E/SEC | Rejection responses generic "capability not available" with no internal state leak. | 4J:L417-L449 |
| P4J-E024 | E/FLOW | task_submit returns task_id async; task_status polls FSM+progress; pipeline_result final output; same path as CLI /task. | 4J:L412-L432 |
| P4J-E025 | E/CAP | Repair: detect 3 issues programmatically; one PipelineSpec per issue; blast radius enforcement; TestOutcomeReport zero PASS_TO_FAIL. | 4J:L468-L517 |
| P4J-E026 | E/RECOVERY | Graceful degradation: Empire optional; features disable cleanly; elai starts without connection. | 4J:L540-L544, L612-L615 |
| P4J-E027 | E/FAIL | Startup unreachable: max 5 reconnect attempts then Error::EmpireUnreachable. | 4J:L536-L537 |
| P4J-E028 | E/DEFER | FSB binary transport evaluate-only; upgrade if event_latency_p95 >5ms consistently; frame payload_len > MAX_FSB_PAYLOAD default 16MB reject. | 4J:L545-L549, L678-L685, L563-L565 |
| P4J-E029 | E/DEFER | Multi-brain orchestration evaluate when multiple elai instances need coordination beyond multi-session. | 4J:L687-L694 |
| P4J-E030 | E/OBS | Metrics phase_4J.toml: connection_latency_ms (<100 Unix / <500 TCP), heartbeat_jitter_ms (<500), event_throughput_per_sec (≥90% of limit), event_latency_p95_ms (<10), capability_roundtrip_ms, repair_scan_time_ms (<5s), binary_size_delta_bytes, test_count (≥40). | 4J:L644-L672 |
| P4J-E031 | E/DET | Wire newline is exactly "\n" not "\r\n". Protocol v:1 preserved. | 4J:L288-L292, L320-L322 |
| P4J-E032 | E/INT | Empire events grounding source type EMPIRE_EVENT / source='empire' for GRC. | 4J:L370-L373, L596-L597 |
| P4J-E033 | E/SURFACE | Host scaffolding includes Tauri command surface + Svelte panel + feature gate (EMPIRE-PENDING). | 4J:L16-L18 |
| P4J-E034 | E/SURFACE | brainctl CLI replacement: orchestrator drives FactoryLine (start/stop/status brain lifecycle). | 4J:L343-L349, L704 |
| P4J-E035 | E/SURFACE | Capability discovery action "capabilities" returns exactly 4 entries. | 4J:L423-L427 |
| P4J-E036 | E/CFG | Feature gate cfg(feature="empire"); non-empire builds zero cost. | 4J:L614-L615, L708 |
| P4J-E037 | E/LIMIT | RuntimeClient ssb/fsm/clock wiring invariant. | 4J:L710-L710 |
| P4J-E038 | E/TEST | Steps 4J.1–4J.9 skeleton→verify→protocol tests→event bridge→capability→repair→resilience→implement→measure. | 4J:L122-L676 |
| P4J-I001 | I/SURFACE | Svelte panel and Tauri commands are required scaffolding surfaces but exact command/panel names are not listed in this phase file. | 4J:L16-L18 |
| P4J-A001 | A/INT | Exact default socket paths for Windows TCP and remote host config schema beyond TransportConfig not fully specified. | 4J:L137-L138 |
| P4J-C001 | C/EVENT | PRE-RESOLVED DECISION 3 requires 5 SSB variants, but 4J.4/4J.8 TASK still write action_type='empire_event' singular. | 4J:L30-L37, L356-L358, L596-L596 |
| P4J-C002 | C/TEST | DETERMINISTIC_CHECK still says cargo test -p elai_empire while DECISION 2 redirects to elai-orchestrator --features empire. | 4J:L21-L28, L313-L314, L618 |

Category coverage: transport/protocol/brain/neuron/bridge/repair/degradation/metrics/security/UI scaffolding ✓.

### IDX — Phase index pre0–2D (concise reference)

Source: `docs/specs/guide/steps/phase-index-pre0-2D.txt` (161/161 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| IDX-E001 | E/STATUS | Document is concise LLM-context index of pre-phase-0 through phase-2D: goals, deliverables, dependencies. | IDX:L1-L2 |
| IDX-E002 | E/CAP | PRE-PHASE 0: Windows platform fixes only (cfg-gate unix PermissionsExt, .cmd hooks, path separators, python not python3, PoisonError, warnings). No feature changes. | IDX:L5-L13 |
| IDX-E003 | E/OBS | PRE0 baseline: 69 .rs files, 61K LOC, 684 test annotations; 0 errors/failures/warnings on Windows. | IDX:L11-L13 |
| IDX-E004 | E/CAP | PHASE 0: strip claw-code to elai_cc; 9-crate workspace; one model ollama; FSM IDLE→EXECUTING→IDLE; one tool file-read; archive old code. | IDX:L16-L24 |
| IDX-E005 | E/LIMIT | Phase 0 risks: 25K LOC runtime/38 files, entropy quarantine 60 occurrences, config circular deps, TokenUsage refactor. | IDX:L21-L22 |
| IDX-E006 | E/CAP | 1A DKG: tree-sitter→SQLite graph; langs Rust/Python/TS/Go; dkg_nodes+dkg_edges; parse failure log+skip; LanguageGrammar trait. | IDX:L27-L34 |
| IDX-E007 | E/CAP | 1B BM25 tantivy: per-project; zero orphan non-binary; null-byte binary detect first 8KB; path STRING untokenized; update_file <5s; p99 query <10ms. | IDX:L37-L43 |
| IDX-E008 | E/CAP | 1C dual-model ONNX vectors: GraphCodeBERT 768d + Granite 384d; cache by content hash; circuit breakers; cascade HYBRID→SPARSE_ONLY→BM25_ONLY; canary embed("fn main()") within 5s. | IDX:L46-L53 |
| IDX-E009 | E/DEFER | SPLADE deferred from 1C to 1H measurement. | IDX:L51-L51 |
| IDX-E010 | E/CAP | 1D PageRank repo map: density guard 0.3; personalization 10x; edge boost *50; sqrt(ref_count); +-15% token fit; line trunc 100 chars; 21+ tests. | IDX:L56-L65 |
| IDX-E011 | E/CAP | 1E CatRAG+A-RAG full pipeline assembly BM25→vector→PageRank→CatRAG→A-RAG. | IDX:L68-L72 |
| IDX-E012 | E/CAP | 1F speculative decoding evaluation: ADOPT if >1.3x throughput on RTX 3060 12GB else DROP; k=1-7 sweep; decision cascade availability→acceptance→prompt lookup→throughput→VRAM. | IDX:L75-L84 |
| IDX-E013 | E/CAP | 1G cross-encoder ms-marco-MiniLM-L12-v2 ~120MB; CPU; <50ms/50 candidates; Semaphore(1)+mpsc depth 4; circuit breaker L-6 fallback. | IDX:L87-L94 |
| IDX-E014 | E/CAP | 1H SPLADE conditional feature=splade; vocab coverage <30% early DROP; KEEP if mean recall Δ≥5% and p<0.05. | IDX:L97-L105 |
| IDX-E015 | E/CAP | 1I resilience: circuit breaker; SearchMode cascade; query validator empty/>10K/null reject; readiness READY/DEGRADED/UNAVAILABLE; QueryTrace/StageTrace; index version migration. Deps 1E AND 1G. | IDX:L108-L116 |
| IDX-E016 | E/CAP | 2A GRC SM: observe→hypothesize→plan_action→act→verify (+status, converge); budget per-topic; forced converge remaining_calls≤1; format-only grounding; GroundingLevel Verified>High>Uncertain; ThinkStep.scope; typed errors. | IDX:L119-L127 |
| IDX-E017 | E/CAP | 2A GrcEngine trait 6 methods (4 implemented + 2 stubs); 12+ tests. | IDX:L126-L126 |
| IDX-E018 | E/CAP | 2B verify methods FILE/SOURCE/TEST/WEB/API/API_REGISTRY_CHECK; binary pass/fail; plan sweep blocks UNGROUNDED before EXECUTING. | IDX:L130-L137 |
| IDX-E019 | E/CFG | 2C six roles with budgets: executor 8/1200; auditor 10/1200 double-verify anti-rubber-stamp; context-compiler observe-only; memory-writer 2/400; prompt-engineer 5/800; designer 8/1200 PipelineSpec. | IDX:L140-L149 |
| IDX-E020 | E/CFG | AgentGrcConfig 8 fields including require_scope_enumeration true for executor/designer; layer [grc.defaults]+[agents.{role}.grc]; additive transition overrides. | IDX:L145-L147 |
| IDX-E021 | E/FLOW | Auditor anti-rubber-stamp: ≥1 FINDING: before Converge or rejected. | IDX:L148-L148 |
| IDX-E022 | E/CAP | 2D CSQ: CRITICAL (data_loss/security/financial/auth guard mandatory blocks) vs WARNING (logged). | IDX:L153-L156 |
| IDX-E023 | E/CAP | Inline feasibility max 2 checks per EXECUTING; Pass/Fail/Blocked; Blocked emits MAJOR ReplanRequest. Completes 2A-2D reasoning. | IDX:L157-L160 |
| IDX-E024 | E/DEP | Dependency chain PRE0→0→1A→1B→1C→1D→1E→1G→1I; 1F after 1C; 1H after 1G; 2A after 0; 2B after 2A; 2C after 2B; 2D after 2C. | IDX:L13-L161 |
| IDX-I001 | I/LIMIT | Index compresses full phase files; step-level PRE_GATE/TASK/ACCEPTANCE not present here (MASTER also delegates). | IDX:L1-L2 |
| IDX-A001 | A/STATUS | Index does not record completion status of any pre0–2D phase. | IDX:L1-L161 |

Category coverage: full pre0–2D phase catalog with key numbers ✓. Surfaces are mostly backend; no dashboard in this index range.

## Cross-source interactions and lifecycle

| Interaction ID | Class | Interaction, ordering, or shared invariant | Evidence atoms |
|---|---|---|---|
| IX-001 | E | 3C AgentHandle.active_files feed 3E file-overlap SSB scoring and conflict detection. | P3C-E009; P3E-E015 |
| IX-002 | E | 3C spawn_parallel sequential stub is upgraded to true parallel only by 3D. | P3C-E008, P3C-E024 |
| IX-003 | E | 3C communicate in-process stub becomes IPC in 4A; same AgentHandle abstraction. | P3C-E011, P3C-E024 |
| IX-004 | E | 3C ReplanRequest MAJOR human gate and MINOR PLANNING fast-track compose with 3A FSM and auditor. | P3C-E012, P3C-E031 |
| IX-005 | E | 3C ReasoningTraceEntry and checkpoint events depend on 3B SSB single-writer append path. | P3C-E005, P3C-E032, P3C-E037 |
| IX-006 | E | 3C per-role AgentGrcConfig loads from 2C role budgets; adversarial cites executor 8/1200 and memory-writer 2/400. | P3C-E007, P3C-E026; IDX-E019 |
| IX-007 | E | 3C Designer role produces PipelineSpec later consumed by 4D/4E/4F chain. | P3C-E023 |
| IX-008 | E | 3E budget is recomputed inside 3D BSP superstep between prepare_next_tasks and execute_tasks; consumes channel kinds from 3D. | P3E-E016, P3E-E017 |
| IX-009 | E | 3E Terminal AUTONOMOUS SPLIT_TASK is owned by 4G; INTERACTIVE escalate is immediate. | P3E-E022 |
| IX-010 | E | 3E OutputReserve and floors protect model generation headroom used by all later agent invocations. | P3E-E013, P3E-E014, P3E-E025 |
| IX-011 | E | 4B tools run only through 4A IPC ToolRequest/ToolResult; isolation/sanitization owned by 4A, execution by 4B. | P4B-E001, P4B-E004, P4B-E021 |
| IX-012 | E | 4B IntegrationTestRunner blocked on 4E DecompositionTree types and matches 4E integration time budgets. | P4B-E003, P4B-E028 |
| IX-013 | E | 4B ShortCoder post-processes executor output before auditor; placement also referenced from 4C hooks. | P4B-E017 |
| IX-014 | E | 4B failure taxonomy 3+ same category/phase triggers 3C anti-spiral. | P4B-E018; P3C-E052 |
| IX-015 | E | 4B write tools honor SSB file locks (orchestrator concurrency invariant). | P4B-E008 |
| IX-016 | E | 4F reuses 2D FeasibilityOutcome and 4A Tier-C agent process with restricted AgentDispatchConfig. | P4F-E010, P4F-E016, P4F-E018 |
| IX-017 | E | 4F skip path PLANNING→TESTING_FIRST and novel_steps marking come from 4E structural validation. | P4F-E011, P4F-E012 |
| IX-018 | E | 4F decision Escalate vs Replan depends on execution mode introduced by 3A/4G. | P4F-E009 |
| IX-019 | E | 4F SSB branch tags compose with 3B SSB and 3D branch semantics (tag=branch:…). | P4F-E020 |
| IX-020 | E | 4F baby-step feasibility requires 4E DecompositionTree/BabyStepSpec and runs before concept execution in 4G. | P4F-E029–P4F-E032 |
| IX-021 | E | 4J event bridge writes Empire telemetry into 3B SSB for GRC grounding. | P4J-E016, P4J-E032 |
| IX-022 | E | 4J ElaiNeuron capability path reuses 4A IPC proxy + ToolFilter trust boundary. | P4J-E017, P4J-E019 |
| IX-023 | E | 4J repair pipelines are generated by 4D pipeline factory and validated with 4H TestOutcomeReport. | P4J-E018, P4J-E025 |
| IX-024 | E | 4J autonomous unattended Empire operation depends on 4G autonomous mode. | P4J-E001 |
| IX-025 | E | Index 2C role budgets are the same numbers 3C adversarial expects at spawn. | IDX-E019; P3C-E026 |
| IX-026 | E | Index 2D Blocked→MAJOR ReplanRequest is the same replan channel 3C wires with max-1/phase. | IDX-E023; P3C-E012 |
| IX-027 | E | Index 1E pipeline is later fitted by 3E token budgets (MASTER cross-ref; 3E depends on retrieval outputs). | IDX-E011; P3E-E003 |
| IX-028 | E | Entropy quarantine ClockProvider appears in 3C timestamps, 3E latency, 4B git times, 4F timeouts. | P3C-E014; P3E-E034; P4B-E034; P4F-E024 |
| IX-029 | E | RuntimeClient ssb/fsm/clock wiring is exit invariant shared by 4B, 4F, and 4J. | P4B-E032; P4F-E035; P4J-E037 |
| IX-030 | E | blake3 is project hash for 3C spiral content, 4F stdout, and supersedes blake2s (B-16A). | P3C-E053; P4F-E005 |
| IX-031 | I | 4J Tauri/Svelte Empire panel will eventually sit beside 4K dashboard surfaces but 4K is out of this chunk. | P4J-E033 |
| IX-032 | E | 3C SubstituteModel is type-ready but gated until 4G; 4F/4B do not enable it. | P3C-E047 |

## Explicit ambiguity and conflict register

| Register ID | Kind | Unresolved issue or required correction | Atom references |
|---|---|---|---|
| AC-001 | ambiguity | 3C tool_results rolling window always-on vs only-if-slow. | P3C-A001 |
| AC-002 | ambiguity | 3C SSB CHECK migration mechanism and spiral "last N" size. | P3C-A002, P3C-A003 |
| AC-003 | ambiguity | 3E attempt-1 overlap threshold "e.g. 0.3" not fixed default. | P3E-I001, P3E-A001 |
| AC-004 | status/conflict | 4B DONE status vs remaining measure/integration body. | P4B-C001, P4B-E002-E003 |
| AC-005 | ambiguity | 4B deterministic commit message template unspecified. | P4B-A002 |
| AC-006 | conflict | 4F FeasibilityProbe vs FeasibilityProber naming and sysinfo vs shell-out RAM strategy. | P4F-C001, P4F-C002 |
| AC-007 | conflict | 4F FEASIBILITY tool set: stub_write+bash only vs +web search. | P4F-A001 |
| AC-008 | conflict | 4J 5 SSB empire_* variants vs body 'empire_event' singular. | P4J-C001 |
| AC-009 | conflict | 4J cargo package name elai_empire vs orchestrator --features empire. | P4J-C002 |
| AC-010 | ambiguity | 4J Tauri/Svelte surface names not enumerated. | P4J-I001 |
| AC-011 | ambiguity | Index pre0–2D omits completion and step-level gates. | IDX-I001, IDX-A001 |

## Summary additions and corrections

Orientation comparison target: `docs/thefinalglance/feature-discovery/01-phase-index-MASTER.discovery.md`. "ADD" means an explicit source behavior omitted or materially compressed by the summary; "CORRECT/QUALIFY/REMOVE" means the summary's wording changes scope or certainty. These are relational records, not additional feature atoms.

### 3C deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-3C-001 | **ADD** full input/output contracts (3B/3A/2C→3D/3E/3F/4A/ReplanRequest max-1). | 01-MASTER:L259-L274 | P3C-E005–P3C-E012 |
| SD-3C-002 | **ADD** claw-code mapping details (UUID EntropySource, ClockProvider, TrustRequired drop). | 01-MASTER:L262 | P3C-E013–P3C-E015 |
| SD-3C-003 | **ADD** R6 VRAM serialize rule and R31 NotFound indistinguishability tests. | 01-MASTER:L268 | P3C-E016–P3C-E017, P3C-E030 |
| SD-3C-004 | **ADD** five checkpoint files, UntrackedValue rejection, Windows atomic rename, metadata-only SSB checkpoint. | 01-MASTER:L269-L271 | P3C-E034–P3C-E038 |
| SD-3C-005 | **ADD** recovery recipe map, corrupt degrade, monotonic step_index, no tool replay. | 01-MASTER:L272 | P3C-E040–P3C-E044 |
| SD-3C-006 | **ADD** spiral UI options string, N-3 recovery_target, blake3 B-16A supersession. | 01-MASTER:L274 | P3C-E053–P3C-E057 |
| SD-3C-007 | **ADD** Tier B bash gap deferral, isolation_metric aggregation out-of-scope, measure targets. | 01-MASTER:L259-L274 | P3C-E058–P3C-E060 |

### 3E deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-3E-001 | **ADD** full AdaptiveLevel enum including Terminal; exact compute remainder-to-CodeContext rule. | 01-MASTER:L290-L300 | P3E-E007, P3E-E026 |
| SD-3E-002 | **ADD** BSP call timing, channel→slot6 rules, handoff BP2 recompute, Display surface. | 01-MASTER:L290-L300 | P3E-E016–P3E-E017, P3E-E024, P3E-E035 |
| SD-3E-003 | **ADD** floor steal order, BudgetInfeasible at dynamic<700, CodeContext<200 warning. | 01-MASTER:L294 | P3E-E027–P3E-E028 |
| SD-3E-004 | **ADD** numbered steps 3E.1–3E.7 acceptance detail MASTER only points to phase file for. | 01-MASTER:L300 | P3E-E036 |
| SD-3E-005 | **ADD** reduction logs excluded from current context; 100x determinism; no Instant::now. | 01-MASTER:L297-L299 | P3E-E032–P3E-E034 |

### 4B deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-4B-001 | **ADD** full DAIRA marker set (15 forms) and MarkerReport counts. | 01-MASTER:L402-L403 | P4B-E013, P4B-E015 |
| SD-4B-002 | **ADD** force_full INTERACTIVE user-decide vs AUTONOMOUS auto-outline. | 01-MASTER:L407 | P4B-E019 |
| SD-4B-003 | **ADD** R1–R4 risks, paper ingest PRE_GATE, RuntimeClient wiring exit rule. | 01-MASTER:L395-L409 | P4B-E005–P4B-E008, P4B-E027, P4B-E032 |
| SD-4B-004 | **ADD** IntegrationTestRunner tool_id AUDITING-only, IntegrationResult stdout_hash. | 01-MASTER:L409 | P4B-E028–P4B-E030 |
| SD-4B-005 | **ADD** metrics key list and UNGROUNDED labels on latency/BLOCK reduction projections. | 01-MASTER:L409 | P4B-E022–P4B-E026 |
| SD-4B-006 | **QUALIFY** recorded DONE status applies to 4B.1–4.4 only; measure+runner remain deferred. | 01-MASTER:L398-L399 | P4B-E002–P4B-E003, P4B-C001 |

### 4F deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-4F-001 | **ADD** complete type field lists (GpuInfo uuid, stderr 512, ResourceDelta, ConstraintResult margin). | 01-MASTER:L461 | P4F-E002–P4F-E008 |
| SD-4F-002 | **ADD** 16+8 named tests T1–T16/A1–A8 and dual-timeout kill_on_drop detail. | 01-MASTER:L463 | P4F-E021–P4F-E024 |
| SD-4F-003 | **ADD** E-22 sysinfo rejection / env fallback path and SD2 flat file layout. | 01-MASTER:L457-L465 | P4F-E034, P4F-C001–P4F-C002 |
| SD-4F-004 | **ADD** metrics targets and 12 testable invariants summary. | 01-MASTER:L463 | P4F-E028, P4F-E036 |
| SD-4F-005 | **ADD** DispatchStage.agent AUTHORITATIVE override rule and SSB branch merge/discard. | 01-MASTER:L462 | P4F-E018, P4F-E020 |
| SD-4F-006 | **FLAG** web_search inclusion conflict in FEASIBILITY tool set. | 01-MASTER:L462 | P4F-A001 |

### 4J deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-4J-001 | **ADD** EMPIRE-PENDING pre-resolved decisions 1–4 (scaffold, submodule path, 5 SSB variants, EmpireIpc namespace). | 01-MASTER:L508-L517 | P4J-E002–P4J-E005 |
| SD-4J-002 | **ADD** Tauri command surface + Svelte panel scaffolding (user-facing). | 01-MASTER:L508-L517 | P4J-E033 |
| SD-4J-003 | **ADD** rate-limit global vs high-priority elevated cap, generic rejection, reconnect Register. | 01-MASTER:L514-L515 | P4J-E020–P4J-E023 |
| SD-4J-004 | **ADD** full measure keys/targets, FSB evaluate trigger, multi-brain evaluate trigger. | 01-MASTER:L517 | P4J-E028–P4J-E030 |
| SD-4J-005 | **ADD** repair blast-radius, TYPE_CHECKING false-positive guard, one-issue-per-pipeline. | 01-MASTER:L512 | P4J-E025 |
| SD-4J-006 | **FLAG CONFLICT** summary/body empire_event singular vs decision 5 variants; elai_empire crate name drift. | 01-MASTER:L514-L516 | P4J-C001–P4J-C002 |

### IDX (pre0–2D) deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-IDX-001 | **ADD** PRE0 Windows-only scope and baseline LOC/test counts not expanded in MASTER's PRE0 section compression. | 01-MASTER:L13 | IDX-E002–IDX-E003 |
| SD-IDX-002 | **ADD** Phase 0 entropy-quarantine occurrence count (60) and 9-crate/one-tool skeleton. | 01-MASTER:L24 | IDX-E004–IDX-E005 |
| SD-IDX-003 | **ADD** 1F ADOPT threshold 1.3x and 1H vocab/recall KEEP gates as compact numbers. | 01-MASTER:L168-L181 (1F), related | IDX-E012, IDX-E014 |
| SD-IDX-004 | **ADD** 2C six role budget table and anti-rubber-stamp FINDING rule. | 01-MASTER:L340-L361 area | IDX-E019–IDX-E021 |
| SD-IDX-005 | **QUALIFY** index is not a substitute for numbered phase files' PRE_GATE/TASK/ACCEPTANCE. | 01-MASTER:L5 | IDX-I001 |

## Exhaustiveness, zero-omission, and duplicate audit

- Source coverage: **6/6 files**, **2,697/2,697 logical lines**, **162,325/162,325 bytes** read and reconciled; source SHA mismatches **0**.
- Atomic source ledger: **253 unique atoms** = **232 explicit**, **006 inferred**, **010 ambiguous**, **005 conflicting/corrective**.
- Per-file atom counts: 3C **065**; 3E **039**; 4B **041**; 4F **040**; 4J **042**; IDX **026**. Sum **253**.
- Relational interaction records: **32** (`IX-001`–`IX-032`). Summary-delta records: **029** (`SD-*`). Ambiguity/conflict register: **11** (`AC-001`–`AC-011`). These records reference source atoms and are not double-counted as features.
- ID audit: every atomic table row has exactly one unique ID matching `P3C/P3E/P4B/P4F/P4J/IDX-[EIAC]nnn`. Duplicate atomic IDs **0**; missing/blank atomic IDs **0**; nonconforming atomic IDs **0**.
- Semantic duplicate audit: repeated premise/scope/wiring/grounding/acceptance statements were merged into one atom with multiple anchors. Interactions, conflict register, and summary deltas reference atoms instead of cloning them. Duplicate feature atoms found **0**.
- Omission audit: every source was checked against capabilities/actions/surfaces; workflows/states; schemas/data/lifecycle; flags/defaults/config; integrations/events; permissions/security/privacy/compliance; failures/fallback/retry/recovery; determinism/idempotency/concurrency; observability/evidence; tests/acceptance/done gates; constraints/non-goals; dependencies/status/deferred. Each per-file ledger notes absent dimensions rather than inventing them.
- Interpretation audit: inferred, ambiguous, and conflicting/corrective atoms are visibly separated by ID/class. No inference is promoted to a requirement. Orientation-only claims remain in delta records and are not source atoms.
- Manifest-boundary audit: the only original sources opened were the six exact DEEP-PHASE-008 manifest rows. The only orientation artifact consulted was `01-phase-index-MASTER.discovery.md`.
- Output metrics after finalization: `output_lines=0469`; `output_bytes=59205`.
- Canonical normalized output SHA-256: 684545fba1e0b5bae45041a13eee4d5e6f667c3411fc3d99fabb165bbc84ee8d
- Hash rule: compute SHA-256 over the exact final UTF-8 bytes including the final newline after replacing only the 64 hexadecimal characters on the `Canonical normalized output SHA-256` line with 64 ASCII zeroes. Fixed-width output metrics and digest substitution avoid impossible raw self-reference.

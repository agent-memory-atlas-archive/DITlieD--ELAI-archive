# Phase 3 Summary — Part 3 (3J + 3J.WIRE.* sub-steps, 3K)

## 3J vLLM + ADVANCED PROVIDERS + PLUGINS + ORCHESTRATOR HARDENING — Status: IMPROVED (largest phase file by far at 93KB)

Goal: integrate vLLM as production inference backend. Layer-level opts (DoLa, TurboQuant KV cache, steering vectors) requiring vLLM internal access. ModelPool features (GPU multiplexing, predictive pre-loading, workload profiling). MCP bridge, tiered static analysis, AppContainer, advanced SSB action types, MILP scheduling eval.

All model-specific steps developed on **Gemma4 E2B** (~2B, fits RTX 3060 12GB). Final step 3J.14 and 3J.15 apply to **Gemma4 31B** via CPU offload (slow but validates artifacts for 48GB GPU).

### Hardware
- Dev: Ryzen 7800X3D, 64GB RAM, RTX 3060 12GB.
- gemma4:31b (19 GB Q4_K_M, 31.3B params, 262144 context, embed dim 5376, multimodal native).
- gemma4:e2b (7.2 GB, ~2B primary test).
- gemma3:4b (3.3 GB fallback).
- vLLM: WSL2 Ubuntu ~/vllm-env/, vLLM 0.19.0, PyTorch 2.10.0+cu128, CUDA RTX 3060.
- Activate: `wsl -d Ubuntu -- bash -c "source ~/vllm-env/bin/activate && <cmd>"`.
- Rust on Windows connects via HTTP to WSL2 localhost (WSL2 auto-forwards ports).

### Risks
- R1 no Rust vLLM client crate — HTTP/gRPC; latency measurement required.
- R2 DoLa needs per-layer logits — vLLM OpenAI API doesn't expose them. Spike in 3J.3; defer if vLLM plugin API insufficient.
- R3 TurboQuant requires modifying vLLM attention backend. Defer if >2 weeks effort.
- R4 Steering vectors require writing residual stream — coupled with DoLa hook.
- R5 MILP Rust bindings Windows build complexity — defer to Linux-only if fails.
- R6 MCP bridge port from claw-code (1751 LOC) — subtle protocol risk.
- R7 AppContainer (Windows) has NTFS junction/reparse weaknesses — documented.
- R8 VRAM accounting for colocated models — safety margin 1GB may be insufficient.
- R9 Gemma4 E2B (~2B) below DoLa/EAGLE-3 paper coverage (7B+) — E2B validates code paths.
- R10 31B CPU offload ~0.5-2 tok/sec — EAGLE-3 draft training infeasible on CPU.
- R11 vLLM CPU offload may not support layer hooks — fall back to transformers+PyTorch for artifacts.

### Step list (15 numbered + 7 WIRE sub-steps + 2 pipeline/unified CLI integration steps)

### 3J.1 VLLM HTTP CLIENT SPIKE
- `providers/src/vllm_client.rs`: HTTP to vLLM. OpenAI-compatible `/v1/completions`, `/v1/models`. SSE streaming (reuse api/sse.rs). `/health` -> `VllmHealthStatus`.
- Benchmark 100 req p50/p95/p99 vs Gemma4 E2B.
- `VllmProvider` struct impl `LlmProvider` trait (complete, embed, token_count).
- Feature-gated `cfg(feature = "vllm")`.
- Evaluate gRPC.
- Metrics: `.elai_cc/metrics/vllm_spike.toml`.
- Accept even if >10ms overhead (layer access is the value).

### 3J.2 VLLM MODELPOOL INTEGRATION
- `InferenceBackend { Ollama(OllamaConfig), Vllm(VllmConfig) }`.
- `VllmConfig`: server_url, tensor_parallelism: bool, served_model_names: Vec<String>.
- ModelPool delegates GPU memory to vLLM (vLLM manages own scheduler). `POST /v1/load_model`, `DELETE /v1/unload_model`.
- Backend config in `elai_cc.toml [providers.backend] = "ollama" | "vllm"`.
- Fallback to Ollama on vLLM unavailable with WARNING log: "layer-level optimizations unavailable".

### 3J.3 DOLA LAYER-CONTRAST DECODING
- `DolaDecoder` in `providers/src/layer_config/dola.rs`. Algorithm (arxiv 2309.03883, ICLR 2024): JSD across candidate layers; adaptive plausibility α=0.1; contrastive logits log(q_N / q_M) for tokens in V_head.
- Candidate layer buckets:
  - ~2B (Gemma4 E2B, ~18-26 layers): J = even layers in [0, N/2) or [N/2, N). UNTESTED range — evaluate empirically.
  - 7B (32 layers): even layers in [0,16) or [16,32).
  - 13B (40 layers): even layers in [0,20) or [20,40).
  - 30B+ (60+ layers): 3-4 buckets of 20 each.
- Hyperparams: α=0.1, θ=1.2 repetition penalty.
- `DolaConfig { enabled: bool, alpha: f64, candidate_buckets: Vec<Range<usize>>, repetition_penalty: f64 }`.
- Per-role (providers.txt §3): Executor (Gemma 4 31B) enabled, Auditor (DeepSeek-R1-32B) amplified, prompt-engineer (Qwen 3.5 27B) disabled.
- CAUTION: DoLa degrades on 335M (paper §6.7, GPT2-Medium). 1-7B UNTESTED. Guard: <1B models auto-disable DoLa with WARNING. 1-7B EVALUATE empirically.
- Latency overhead: paper §7 shows 1.01-1.08x (7B to 65B). Memory +0.6-1.1%.
- Degraded path: if vLLM only exposes top-N logprobs (not full distribution), DoLa accuracy reduced — document + measure.

### 3J.4 TURBOQUANT KV CACHE COMPRESSION (EVALUATION)
- TurboQuant (arxiv 2504.19874, ICLR 2026): random rotation (QR of Gaussian) + Beta->N(0,1/d) + Lloyd-Max scalar quantizer per coordinate. Expected: 3.5-bit matches FP (NIAH 0.997), 2.5-bit marginal degradation (LongBench 49.44 vs 50.06).
- `TurboQuantConfig { bit_width: u8 (1-4), rotation_seed: u64 }`.
- Critical invariants: unit norm input (store L2 separate), Π orthogonal rotation, codebook size = 2^b, centroids ascending sorted.
- Pitfalls: don't reuse Π across instances without reset; don't skip norm rescaling; don't approximate Π with Hadamard (full QR required); don't use MSE variant for inner product queries (biased).
- Min dimension d >= 64 (low-d fails Beta approximation).
- E2B testing: precompute Lloyd-Max codebooks for E2B head dim (data-oblivious, reusable across models with same d).
- Output: FEASIBLE/DEFER decision in `workflow/reasoning/phase-3J-turboquant.md`. If FEASIBLE <1 week: implement.

### 3J.5 STEERING VECTORS (ActAdd)
- `SteeringVectorEngine` in `providers/src/layer_config/steering.rs`.
- ActAdd (arxiv 2308.10248): `h_A^l = h+^l - h-^l`. At inference add `c * h_A^l` to residual stream at target layer.
- `SteeringVector { layer: usize, vector: Vec<f32>, coefficient: f64 }`.
- `SteeringConfig { vectors: Vec<SteeringVectorSpec>, enabled: bool }`.
- `SteeringVectorSpec { name, layer: usize, coefficient: f64, positive_prompt: String, negative_prompt: String }`.
- Per-role (providers.txt §3 L102-104):
  - Auditor: skepticism steering vector.
  - Context-compiler: completeness steering vector (temp 0.7 + steering).
- Layer guidance (ActAdd §6.3): early (0-5) minimal, mid (6-20) peak, late (25+) declining. Default `layer = model_layers / 3`.
- Coefficient: c in [3, 20]. Start c=5, tune per vector.
- Coupled with DoLa — same layer-access hook. If DoLa deferred, steering deferred.
- API-only models (Anthropic, OpenAI) CANNOT use — local only.
- E2B testing: ActAdd paper validated on GPT2-XL (1.5B) so E2B (~2B) within range. Compute 2+ contrast pairs (skepticism, completeness); store as .safetensors or .npz.

### 3J.6 GPU MULTIPLEXING + PREDICTIVE PRE-LOADING
- Multiplexing (providers.txt §4 L130-136, Murakkab-derived):
  - Small models (<4GB) colocate on same GPU with large model's idle headroom.
  - Embedding (~1.5GB) + cross-encoder (~130MB) on GPU1 while executor on GPU0.
  - Constraint: sum(colocated_vram) <= GPU_VRAM - safety_margin (1GB default).
  - Phase 0-1 single GPU: no multiplexing. Phase 2+ dual: enabled on GPU1 for small models.
- `MultiplexingPolicy { safety_margin_bytes: u64 (1GB default), max_colocated: usize (3 default), min_model_size_for_dedicated_gpu: u64 (4GB default) }`.
- `ModelPool.can_colocate(model_id, target_gpu) -> Result<bool>`.
- Predictive pre-loading (FSM-driven):
  - EXECUTING -> pre-load auditor.
  - AUDITING -> pre-load executor.
  - PLANNING -> pre-load prompt-engineer.
  - TESTING_FIRST -> pre-load executor.
  - Triggers on state ENTRY (not exit). Background `tokio::spawn`. Blocked if VRAM insufficient.
- `PreloadPolicy` map FsmState -> Vec<ModelId>, `elai_cc.toml [providers.preload]`.
- SLO quality tiers (Murakkab):
  - BEST (auditor, researcher): highest precision, full VRAM.
  - GOOD (executor, context-compiler): standard.
  - FAIR (prompt-engineer): smaller/quantized variant.
  - BASIC (memory-writer): smallest capable, minimal resources.
- Tier in `elai_cc.toml [agents.{role}.slo_tier]`. Tier mismatch (BEST role + BASIC model) -> WARNING.

### 3J.7 WORKLOAD PROFILING
- `WorkloadProfile { agent_role: AgentRole, phase_id: String, tokens_in: u64, tokens_out: u64, latency_ms: u64, vram_peak_mb: u32, model_id: String, gpu_id: u8, timestamp: i64 (ClockProvider) }`.
- SSB action_type='workload_profile', payload = msgpack(WorkloadProfile).
- Emitted at every phase_complete.
- Phase 0-1: collect only. Phase 2+: EVALUATE whether profiles improve placement decisions.
- Query: `SELECT * FROM ssb WHERE action_type='workload_profile' AND project_id=?`.

### 3J.8 MILP SCHEDULING EVALUATION
- Decision variables: model_assignment[model, gpu] in {0,1}. Objective: minimize phase-transition latency. Constraints: VRAM, each model on exactly 1 GPU, concurrent models on different GPUs or colocated.
- Rust solvers: `good_lp` v1.10+ (abstraction over CBC, HiGHS, CPLEX). HiGHS MIT, CBC EPL. `highs` crate (no external C++).
- Decision criteria:
  - Heuristic <5% worse than MILP -> DROP.
  - MILP >10% improvement -> ADOPT with good_lp + HiGHS.
  - 5-10% -> EVALUATE further in Phase 4.
- `GreedyModelPlacer`: sort models by VRAM desc, assign to GPU with most free space. Always implemented (baseline).
- If ADOPT: `MilpModelPlacer`. Handle infeasibility -> fall back to greedy with WARNING.
- Output: `workflow/reasoning/phase-3J-milp.md` ADOPT/DROP/EVALUATE.
- Target solve time <50ms (hard budget 100ms).

### 3J.9 MCP BRIDGE
- Port claw-code: `mcp_tool_bridge.rs` (912 LOC) -> `plugins/src/mcp_bridge.rs`, `mcp_lifecycle_hardened.rs` (839 LOC) -> `plugins/src/mcp_lifecycle.rs`.
- `McpBridge` trait:
```
fn connect(server_url: &str) -> Result<McpConnection>
fn list_tools(conn: &McpConnection) -> Result<Vec<McpToolSpec>>
fn call_tool(conn: &McpConnection, tool: &str, args: Value) -> Result<Value>
fn disconnect(conn: &McpConnection) -> Result<()>
```
- `McpConnection`: transport (stdio, SSE, or streamable HTTP).
- Error recovery: 3 retries exp backoff (1s/2s/4s), 30s heartbeat timeout, graceful degradation.
- MCP tools in orchestrator tool registry alongside native tools (uniform ToolRequest/ToolResult).
- Agents never connect to MCP server directly — orchestrator proxies.
- Security: tool results sanitized, no prompt injection (treat as data), 30s per-call timeout.
- Feature-gated `cfg(feature = "mcp")`.

### 3J.10 TIERED STATIC ANALYSIS HOOKS
- T1 (RULES, <1ms): fires on EVERY SSB write. Evaluates predicates from rules.db for active rules matching phase + domain. Outputs ALLOW|WARN|BLOCK.
- T2 (AST, <10ms for files <5000 lines): fires on `file_write` ONLY. tree-sitter parse + pattern matching. Checks syntax, imports, dead code, lazy code. Circuit breaker: files >5000 lines degrade to T1-only.
- T3 (PATTERN MATCH, <5ms): fires ONLY when WARN exists AND no BLOCK from T1/T2. Library of known-ambiguous patterns. NO LLM. Examples: fn >200 lines -> WARN:complexity, `catch Exception: pass` -> BLOCK:swallow.
- Hook integration: SSB writer calls T1 sync (<1ms budget); T2 async on file_write; T3 conditional.
- BLOCK -> SSB 'finding' entry -> agent notified on next read.
- No LLM in validation hot path.
- Findings path: BLOCK/WARN -> SSB 'finding' -> orchestrator normalizes to pattern hash (blake2s) -> feeds violation_patterns in rules.db for self-discovery (circular trust boundary enforced — orchestrator normalizes, not auditor).
- Hook verdict sanitization: BLOCK -> agent receives "Tool call rejected by policy" (no rule ID, tier, or pattern info exposed).

### 3J.11 APPCONTAINER ISOLATION (Windows)
- `CreateAppContainerProfile()` per agent. Filesystem: only granted paths. Network: none default (researcher gets network capability). Registry: none. Process: isolated.
- `AppContainerConfig { profile_name: "elai_cc_agent_{agent_id}", granted_paths: Vec<PathBuf>, network_allowed: bool }`.
- Job object limits:
  - JOB_OBJECT_LIMIT_ACTIVE_PROCESS = 1.
  - JOB_OBJECT_LIMIT_PROCESS_MEMORY = 512MB.
  - JOB_OBJECT_LIMIT_JOB_TIME = configurable.
- Fallback (AppContainer unavailable): restricted token + Job object. Known weakness: NTFS junctions/reparse points may bypass ACL deny. Documented, not hidden. Use Linux for production security.
- Self-modification locks (orchestrator.txt §12 L628-634):
  - IPC-level: reject WriteFile/EditFile to locked paths (canonical realpath comparison).
  - Filesystem-level: locked dirs not granted in AppContainer.
  - Git-level: filter GitCommit requests, reject if diff touches locked path.
- Env sanitization: only LANG, TERM, PATH (restricted), TMPDIR (scratch). NOT HOME, USER, API keys, ELAI_CC_*.
- Common trait `AgentSandbox { create(), launch_agent(), cleanup() }`. `#[cfg(target_os = "windows")]` AppContainer; `#[cfg(target_os = "linux")]` clone() + seccomp (4A).

### 3J.12 ADVANCED SSB ACTION TYPES
- Already in 3B: file_write, file_read, tool_result, test_outcome, finding, claim, checkpoint, phase_transition.
- Adds in 3J.12: model_swap, knowledge_degradation, isolation_metric, plan_update, replan_request, stub_reference, reasoning_trace, memory_monitoring, phase_complete, workload_profile.
- Payloads:
  - model_swap: `{ from_model, to_model, gpu_id, swap_duration_ms, reason }`.
  - knowledge_degradation: `{ component, old_mode, new_mode, reason }`.
  - isolation_metric: `{ agent_id, phase_id, context_savings_ratio }`.
  - plan_update: `{ phase_id, version, change_summary }`.
  - replan_request: `{ agent_id, severity: MINOR|MAJOR, evidence }`.
  - stub_reference: `{ phase_id, step_id, stub_hash }`.
  - reasoning_trace: `ReasoningTraceEntry` (orchestrator.txt §7A).
  - memory_monitoring: `{ channel_id, deep_size_bytes, threshold_exceeded }`.
  - phase_complete: `{ phase_id, agent_id, evidence_artifacts, commit_hashes, test_outcomes }`.
  - workload_profile: `WorkloadProfile` (3J.7).
- Typed validation at write time (not read time). Type mismatch -> SSB writer rejects.
- Cognitive isolation metric computation (orchestrator.txt §7A): `context_savings_from_isolation = sum(agent_grc_tokens_this_phase) / model.max_context`. Logged at each phase transition.

### 3J.PIPELINE CLI-ORCHESTRATOR INTEGRATION
Wire CLI's ConversationRuntime through orchestrator pipeline so BOTH local and API backends use full Phase 1-3 infra (FSM-aware tool filter, context budget, agent dispatch, rules engine).
- Current: CLI -> AnthropicClient -> API (bypasses infra).
- Target: CLI -> Orchestrator.dispatch() -> Backend (local or API).
- `InferenceDispatcher` trait: `fn dispatch(request: &MessageRequest, context: &DispatchContext) -> MessageStream`.
- `DispatchContext`: FSM state, agent role, tool filter, context budget, backend config.
- Two impls: `ApiDispatcher` (wraps AnthropicClient), `LocalDispatcher` (wraps SidecarProvider).
- `elai_cc.toml`:
```
[backend]
mode = "local" | "api" | "hybrid" | "auto"
[backend.role_override]
executor = "local"
auditor = "api"
prompt_engineer = "local"
```
- "auto" mode: try local sidecar first, fall back to API if unhealthy.
- FSM-aware tool filtering: PLANNING read-only + WebSearch; EXECUTING full set; AUDITING read-only + test runner. Local backend: max 15 tools, truncated descriptions. API backend: all (Claude handles 49+).
- Context budget integration: local = model max context (131K for E2B) * 0.8 conservative; API = 200K (Claude).
- Auto-start sidecar: check health on startup; spawn WSL2 sidecar process (reuse elai.bat); wait 120s; fall back to API if "auto"; /shutdown on CLI exit.
- Session summary (from 3J.PIPELINE session, 2026-04-09): built `orchestrator/src/dispatcher.rs` (DispatchContext, BackendMode, BackendConfig, DispatchError thiserror, build_dispatch_context — 7 unit tests); `orchestrator/src/fsm.rs` added `Fsm::read_persisted_state()` sync; `cli/main.rs` added LocalRuntimeClient (AuthSource::None + sidecar base_url), CliApiClient enum dispatch, start_sidecar() with exp backoff + general drive letter WSL path translation, build_local_runtime(), LiveCli::new_local(), prepare_turn_runtime routing by local_config, sidecar /shutdown on CLI exit; python/server.py added /shutdown endpoint + real tokenizer token counting (input_ids.shape[1]); elai.bat simplified to --local flag + HOME env fallback. Patches 0.3 (real tokenizer counting) + 0.4 (Idle=no tools, per-turn local routing, sidecar shutdown). Auditor fixes: drive letter hardcoding -> windows_to_wsl_path general; FSM tool filtering added FsmState::check_tool_allowed in stream(); FSM state permanently Idle -> reads persisted .fsm-state.msgpack per-turn; dead variants removed; sidecar_port redundancy removed; WSL path quoting; prepare_turn_runtime routing. Tests: 7/7 dispatcher GREEN, workspace clean, 56 pre-commit PASS.

### 3J.UNIFIED SINGLE RUNTIMECLIENT WITH CONFIG-DRIVEN BACKEND SELECTION
Eliminate duplicated ApiClient implementations (AnthropicRuntimeClient and LocalRuntimeClient share ~140 lines of identical stream() logic).
- Single `RuntimeClient` struct: `{ client: AnthropicClient, auth: AuthSource, backend: BackendMode, tool_policy: ToolPolicy, budget_policy: BudgetPolicy }`.
- ONE stream() implementation with policy branching (auth source, tool filter policy, budget ceiling, prompt cache). Max 3 branch points on backend.
- `ToolPolicy` enum: Full (API), FsmFiltered { max_tools, fsm_state_path }, None (Idle).
- `BudgetPolicy` enum: ApiDefault { ceiling: 200K }, Local { ceiling: 131K, fraction: 80% }.
- Remove `build_local_runtime()`, remove `LocalCliConfig` from LiveCli, replace with `Option<BackendConfig>`.
- `prepare_turn_runtime` always calls `build_runtime(config)`.
- Auto-wire guarantee: new phase adds feature to stream() -> both backends get it; new tool in `mvp_tool_specs()` available to both; new FSM state -> tool filtering works for both; new BackendConfig field readable by both paths.
- BackendConfig max 10 fields; per-backend complexity goes to `LocalConfig`/`ApiConfig` sub-structs.
- Adversarial guards: tool_policy filtering must match old per-client semantics (port existing tests + new test comparing same request through Api vs Local).

### 3J.WIRE.FOUNDATION — SSB + FSM + INTERACTIVE GATE IN CLI
Wire core orchestrator infra into LiveCli so running CLI actually uses FSM, SSB, interactive gate from 3A-3B.
- Add to LiveCli: `ssb: Ssb, fsm: Arc<Fsm>, clock: Arc<dyn ClockProvider>`. Constructed in `LiveCli::new()` and `new_local()`.
  - Ssb path: `sessions/{sid}/ssb.db`.
  - Fsm path: `sessions/{sid}/.fsm-state.msgpack`.
  - ClockProvider: `SystemClockProvider` (already in `elai_core`).
- `CliInteractiveGate` impl `InteractiveGate` for terminal I/O: `present_gate()` prints `EvidenceSummary` + `RiskLevel`, prompts user for Approve/Reject/Modify/Redirect. LOW risk auto-advances (no prompt). Wire into FSM transitions.
- Replace `prepare_turn_runtime()` FSM read: `Fsm::read_persisted_state` (sync file read) -> `self.fsm.get_state()` (CLI owns Fsm).
- Wire Ssb into ConversationRuntime via `with_session_tracer()`. Create `SessionTracer` backed by Ssb writes (turn_started, tool_execution, turn_completed events -> SsbPayload entries).
- Shutdown: `Ssb::shutdown()` in LiveCli drop/exit. FSM persists automatically on transition.
- Guard: Ssb requires running Tokio runtime; reuse RuntimeClient's. Do NOT create second runtime (nested block_on panic). `Fsm::transition()` is async; CLI is sync — use `runtime.block_on()` same pattern as `RuntimeClient::stream()`.

### 3J.WIRE.BUDGET — REAL CONTEXT BUDGET SLOT ALLOCATION
Replace `[0, 0, 0]` placeholder in `BudgetPolicy::Local` with real measured token counts for 3 fixed slots.
- Slot 0 (SystemPrompt): system_prompt chars / 4 (estimate).
- Slot 1 (Rules): 0 for now (filled in WIRE.RULES).
- Slot 2 (PhaseSpec): 0 for now (filled in WIRE.PLAN).
- Pass real values to `ContextBudget::compute`.
- Set max_tokens in MessageRequest: `remaining = budget.total_allocated - budget.total_used; max_tokens = min(max_tokens_for_model, remaining)`.
- Wire ContextBudget into ConversationRuntime per-turn (field on RuntimeClient or pass via build_runtime).
- Log budget allocation in [pipeline] telemetry (slots, allocated vs used, effective max_tokens).
- Guard: Api path skip ContextBudget entirely; Api models have 200K+. Token estimate chars/4 for allocation only; sidecar reports real input_tokens — log delta for calibration.

### 3J.WIRE.RULES — RULES ENGINE INTO PRE-TOOL HOOKS
Wire rules engine into CLI tool execution path (programmatic enforcement, not prompt instructions).
- Seed rule loading: implement 3F.7 — load rules from domain-specific TOML/JSON files into rules.db at startup. Start with engineering + security (~30-40 core rules). Rules map to CLAUDE.md (no .unwrap(), impact analysis, no silent error discard).
- Wire `DefaultRuleEngine` into PreToolUse hook: before tool execution evaluate active rules against `EvaluationContext` (action=tool_name, source=agent_input, file_path). BLOCK -> tool call rejected. WARN -> logged to SSB. ESCALATE -> InteractiveGate prompt.
- `AstEvaluator` stub: return Pass for all AST_PATTERN rules. Real tree-sitter = Phase 4C. GREP_PATTERN + METRIC_GATE work immediately.
- `DkgQuerier` stub: return empty graph. Real DKG = Phase 4B.
- Update ContextBudget Slot 1 (Rules): measure loaded rules token count, pass to `fixed_slot_sizes[1]`.
- Strip CLAUDE.md rule content from local model system prompt: flag in `build_system_prompt()` / `SystemPromptBuilder` to skip instruction file loading when rules engine active.
- Guard: Api path keeps CLAUDE.md (cloud models still need full system prompt). Seed rules version-controlled in `.elai_cc/data/rules/`. Rule eval <5ms (Tier 1 target) — benchmarked in [pipeline] telemetry.

### 3J.WIRE.SESSION — SESSION DAG REPLACING RAW PERSISTENCE
Replace raw Session jsonl with DAG-based snapshots + context trimming from 3H.
- Create `SessionDag` alongside Session in `LiveCli::new()`. Path `sessions/{sid}/dag.db`. Wire `EntropySource` + `ClockProvider`.
- Auto-snapshot after each turn: `save_snapshot()` with current FSM state, phase, messages -> `TrimmedMessage`.
- Wire `trim_context()` into `prepare_turn_runtime()`: before building API request check if context > 85% model budget (handoff trigger from 3H). 3-pass identify->stub->validate pipeline. Stubs reference SSB seqs.
- `/resume` -> `SessionDag::load_snapshot()` + `parent_chain()` instead of `Session::load_from_path()`.
- `/session fork` -> `SessionDag::create_branch()`. Max 3 active branches (enforced by SessionDag).
- Stub recovery on snapshot load: `SessionDag::recover_stubs()`.
- Guard: SessionDag uses sync SQLite (NOT WAL) — separate db files, separate connections from Ssb WAL. Local model: bytes/4 heuristic for trim budget; Api model: real tokenizer if available. Do NOT break `/compact` (manual override alongside auto trimming).

### 3J.WIRE.PLAN — PLAN SKELETON AWARENESS IN CLI
Make CLI plan-aware: load, display, track progress through PlanSkeleton; feed phase context into ContextBudget.
- Add `plan: Option<PlanSkeleton>` to LiveCli. Load from `.elai_cc/plans/{pipeline_id}.toml` if exists.
- `/plan` slash command: display skeleton (phases, status, deps). `/plan phase <id>` shows PhaseDetail.
- Wire phase context into ContextBudget Slot 2 (PhaseSpec): `PlanSkeleton::load_phase_context()` returns content + token_count -> `fixed_slot_sizes[2]`.
- Phase transition: when FSM transitions to new state, update active phase in skeleton. `PlanSkeleton::propagate_changes()` notifies dependents via SSB.
- Contract tracking: display input/output contracts; show satisfied vs pending.
- Guard: plan optional (None handling throughout). `load_phase_context()` needs LlmProvider for token counting — local mode chars/4 estimator; Api mode real tokenizer from AnthropicClient if available. Do NOT auto-create plans.

### 3J.WIRE.STUBS — REPLACE PRODUCTION STUBS WITH REAL ADAPTERS
Replace PassAstEvaluator and EmptyDkgQuerier with real adapters backed by knowledge crate.
Stub inventory (full codebase audit, session s-877904):
- IN SCOPE: (a) PassAstEvaluator -> TreeSitterAstEvaluator; (b) EmptyDkgQuerier -> DkgQuerierAdapter.
- DEFER (other phases): (c) OllamaProvider 4x todo!() — sidecar/provider phase. (d) OllamaEmbedder 4x todo!() — sidecar/provider phase. (e) NetworkVerifier — reasoning 2B. (f) NoopTestRunner — reasoning 2B. (g) OrtSession::embed() — knowledge vector phase (Step 4). (h) VectorIndex readiness/rerank — knowledge vector phase (Step 4).
- Deliverable (a) `TreeSitterAstEvaluator` at `crates/knowledge/src/ast_adapter.rs`:
  - Implements `elai_rules::predicate::AstEvaluator`.
  - Backed by tree-sitter crate already in knowledge/Cargo.toml.
  - Holds `HashMap<&'static str, tree_sitter::Language>` for supported languages.
  - `evaluate_query(language, source, query)`: Get Language -> `tree_sitter::Parser::parse` -> `tree_sitter::Query::new` -> `QueryCursor` matches -> `Vec<AstMatch>` with byte offsets + matched text.
  - `validate_query(language, query)`: `Query::new(language, query)` -> Ok(()) or `AstError::InvalidQuery`.
  - `RustGrammar + LanguageGrammar` trait exists (for DKG node extraction) — needs lower-level `tree_sitter::Query` API directly.
- Deliverable (b) `DkgQuerierAdapter` at `crates/knowledge/src/dkg_adapter.rs`:
  - Implements `elai_rules::predicate::DkgQuerier`.
  - Backed by `DkgEngine.conn` (SQLite with dkg_nodes + dkg_edges).
  - `query_reachability(&ReachabilityParams)`: SQL LIKE source + sink -> BFS/DFS on dkg_edges up to max_depth -> filter by edge_types -> must_not_exist/must_traverse logic -> `Vec<ReachabilityMatch>`.
  - `validate_params`: non-empty patterns; max_depth <= 20 (prevent runaway BFS).
- Wire into CLI `build_runtime`:
  - PassAstEvaluator -> `TreeSitterAstEvaluator::new()`.
  - EmptyDkgQuerier -> `DkgQuerierAdapter::new(dkg_engine)`.
  - Build DkgEngine at startup (or lazy on first rule eval — expensive O(project_size)).
  - Add `elai-knowledge` dep to `crates/cli/Cargo.toml`.
  - Add 2-3 AST rules to test (current 12 seed rules all GrepPattern).
- Guards: Query::new can panic — always Result; BFS visited set (max_depth alone not sufficient for cycles); DkgEngine build_graph O(project_size) — once at startup, incremental on file changes; if DkgEngine fails to init, fall back silently to EmptyDkgQuerier with WARNING; AstEvaluator unknown languages -> `AstError::UnsupportedLanguage`.

### 3J.13 VERIFY + MEASURE (E2B)
- Full workspace compile + tests.
- vLLM benchmarks on Gemma4 E2B (RTX 3060 12GB): HTTP p50/p95/p99, DoLa overhead %, DoLa factuality delta (20 prompts improvement/neutral/degradation), steering vector latency, steering effectiveness (5 prompts/vector), TurboQuant codebook time + roundtrip MSE, model swap time with vs without pre-loading.
- Feature gate verification (non-vllm build).
- Compile time delta.
- Metrics file `.elai_cc/metrics/phase_3J.toml` fields: test_model, vllm_latency_{p50,p95,p99}_ms, dola_overhead_pct or "deferred", dola_factuality_delta, turboquant_decision FEASIBLE/DEFER, turboquant_roundtrip_mse, milp_decision ADOPT/DROP/EVALUATE, steering_vector_count, steering_vectors_effective, multiplexing_enabled, ssb_action_type_count, t1_hook_latency_median_ms, t2_hook_latency_median_ms, t3_hook_latency_median_ms, compile_time_delta_sec.

### 3J.14 WIRE.E2E — END-TO-END LOCAL PIPELINE
Make elai.bat start full Phase 0-3 pipeline as single command.
- Primary backend: custom sidecar (DoLa + TurboQuant + NF4).
- Fallback backend: OllamaProvider (gemma4:e2b via ollama).
- Knowledge retrieval: `QueryPipeline` wired into prompt assembly (relevant code context, not just raw user messages).
- Release binary: `cargo build --release -p elai-cli` -> `target/release/elai.exe`.
- A. Ollama fallback in local mode:
  - `BackendMode::LocalWithFallback` or `fallback_url` on BackendConfig.
  - On sidecar health fail at startup OR mid-session HTTP errors: log warning, switch to ollama endpoint (`http://localhost:11434`), use OllamaProvider for completion (no DoLa/TQ — plain inference), re-probe sidecar on next turn.
  - Degraded mode prompt prefix: "[degraded] using ollama fallback".
  - Both down: "No local inference backend available. Start sidecar or ollama."
  - Config: `[backend] fallback_url = "http://localhost:11434"`.
- B. Knowledge retrieval in prompt assembly:
  - On CLI startup: check BM25 index at session/project path; if not, `Bm25Index::index_directory(cwd)`; else open.
  - Create `QueryPipeline` with `Bm25Index + CatRag + AragOrchestrator`. `CrossEncoderReranker` optional (skip if ONNX model missing).
  - Before each LLM call near `ContextBudget::compute`: extract query from user's latest message -> `QueryPipeline::query(msg)` -> format top-K results as code context (file path, line range, snippet) -> inject into ContextBudget Slot 4 (CodeContext) or append to system prompt. Truncate by score if exceeds allocated tokens.
  - BM25 update on file_write/file_edit: `Bm25Index::update_file()` keeps index fresh within session.
  - DKG integration: if dkg.db exists, use DkgEngine for PageRank-boosted results; else skip (pipeline graceful degradation).
- Startup sequence (elai.bat triggers):
  1. elai.exe --local.
  2. Probe sidecar health -> start if not running -> fallback to ollama if both fail.
  3. Open/create BM25 index for cwd.
  4. Create QueryPipeline.
  5. Initialize Ssb, Fsm, SessionDag, RuleEngine, ContextBudget, PlanSkeleton.
  6. Display startup banner (backend status, index status, session ID).
  7. REPL loop.
- Targets: startup <60s (excl model load); release binary <100MB (strip debug).

### 3J.15 GEMMA4 31B VALIDATION (CPU OFFLOAD)
Apply E2B-built code to 31B via CPU offload (~0.5-2 tok/sec). Slow but sufficient for artifact generation.
- A. Steering vectors (highest priority — most feasible on CPU): 2 contrast pairs (skepticism, completeness). ~10-20 min for 2 pairs. Store as .safetensors/.npz. 5 prompts per vector comparison vs baseline. Compare 31B vs E2B directional consistency.
- B. DoLa evaluation on 31B with buckets [0,20), [20,40), [40,60) (31B has enough layers for proper 3-bucket DoLa). 20 test prompts ~30-90 min total.
- C. TurboQuant codebooks: verify E2B codebooks work for 31B head dim (if same d); else generate 31B-specific. Validate roundtrip MSE.
- D. EAGLE-3 draft training: SKIP on current hardware (infeasible at 0.5-2 tok/sec). Document deferral to 48GB GPU. Pre-work: define draft model architecture (single decoder layer + LM head, reusing 31B vocab + embed dim). Save architecture config.
- E. Full pipeline test (optional): single 31B inference pipeline prompt -> DoLa -> steering -> output. ~5-10 min.
- Metrics file `.elai_cc/metrics/phase_3J_31b.toml`: test_model=gemma4-31b-q4, inference_mode=cpu_offload, inference_speed_tok_sec, steering_vectors_computed, steering_vector_effectiveness qualitative, dola_factuality_delta, dola_candidate_buckets, turboquant_codebook_validated, eagle3_draft_architecture_defined, eagle3_draft_training_feasible=false, eagle3_deferred_reason="requires 48GB GPU", full_pipeline_test_passed.
- F. 100B validation (gated on 48GB GPU): SKIP if no 48GB. When arrives: Q3 100B (~37GB) + TurboQuant 3-bit KV cache (~7.5GB for 128K) = 44.5GB fits in 48GB with ~3.5GB headroom. Test SUBSTITUTE_MODEL escalation (31B default executor; anti-spiral 3x triggers swap to 100B; 30-60s swap acceptable). Compare 100B Q3 vs 31B Q4 on 10 complex prompts. Results in `.elai_cc/metrics/phase_3J_100b.toml`.

### 3J Exit criteria
- VllmProvider implements LlmProvider + ModelPool integration.
- DoLa + steering tested on E2B and validated on 31B via CPU offload.
- TurboQuant FEASIBLE/DEFER decision with evidence; codebooks validated for both E2B and 31B head dims.
- GPU multiplexing + predictive pre-loading with VRAM safety.
- Workload profiling at every phase boundary.
- MILP ADOPT/DROP with heuristic baseline always available.
- MCP bridge with error recovery + security.
- Tiered static analysis T1<1ms, T2<10ms, T3<5ms.
- AppContainer Windows with documented limits.
- All 17 SSB action_types with typed validation.
- Zero regressions 0-3I.
- EAGLE-3 draft architecture defined for 31B, training deferred to 48GB.

### Key grounding
- vLLM OpenAI-compatible VERIFIED (does NOT expose per-layer logits).
- DoLa needs intermediate layer access GROUNDED paper §3.2-3.3.
- DoLa latency 1.01-1.08x GROUNDED paper §7.
- DoLa degrades on 335M GROUNDED §6.7. 1-7B UNTESTED by paper — EVALUATE empirically.
- TurboQuant 3.5-bit matches FP GROUNDED Table 1 (NIAH 0.997).
- TurboQuant min d >= 64 GROUNDED (Beta fails low-d).
- ActAdd composable GROUNDED §9; mid-layer peak GROUNDED §6.3.
- ActAdd tested on GPT2-XL (1.5B), GPT-J-6B, OPT-6.7B, LLaMA-13B, LLaMA-3-8B — E2B (~2B) within range.
- EAGLE-3 vLLM RTX 3090 BS=2: 1.75x GROUNDED paper Table 5.
- Murakkab SLO tiers + MILP GROUNDED interface map.
- `good_lp` crate needs VERIFY (crates.io).
- vLLM plugin API for custom sampling needs VERIFY (critical path).
- AppContainer since Windows 8 GROUNDED orchestrator.txt §12.
- MCP claw-code reference: mcp_tool_bridge.rs 912 LOC + mcp_lifecycle_hardened.rs 839 LOC.
- TIDE uses EAGLE-3 + dual EMA + PracticalSpeedup (SGLang + vLLM).
- 31B Q4 ~18GB fits 64GB RAM.
- 31B CPU ~0.5-2 tok/sec — training:unverified, measure in 3J.14.

---

## 3K RAG PRODUCTION HARDENING — Status: IMPROVED (paper-grounded)

Goal: upgrade internal ELAI knowledge pipeline (`crates/knowledge/`) from prototype to production retrieval. Better retrieval = less context needed = smaller prompts = local LLMs can handle the work.

Prereqs: Phase 1E complete (CatRAG + A-RAG), Phase 1G (cross-encoder), 3J (sidecar for HyDE, step 3K.3 only).

### Current state (from Phases 1A-1I)
- DKG (1A): tree-sitter AST -> SQLite (Rust/Python/TS/Go).
- BM25 (1B): tantivy with 50-line fallback.
- Vectors (1C): dual ONNX (GraphCodeBERT 768d + Granite 384d).
- PageRank (1D): symbol importance.
- CatRAG (1E): query-aware graph traversal overlay.
- A-RAG (1E): hierarchical assembly.
- Reranker (1G): ms-marco-MiniLM-L12-v2 cross-encoder CPU-only <50ms.
- SPLADE (1H): evaluated.
- Resilience (1I): circuit breakers, SearchMode cascade (HYBRID->SPARSE_ONLY->BM25_ONLY), query validation, startup readiness, tracing.

### Gaps
- BM25 uses 50-line fixed windows (not AST).
- Single query per search (no expansion/rephrasing).
- No HyDE.
- No quality-tier routing.
- No retrieval quality metrics (only token savings).

### Risks
- R1 HyDE local LLM call adds 1-3s — only DEEP tier uses it.
- R2 AST chunking depends on DKG tree-sitter grammars.
- R3 Query expansion noise — template-based (not LLM), 2-3 variants max.

### Deliverables (5 main + 3 paper-grounded)

- **3K.1 Adaptive chunking**: replace 50-line windows with AST-aware (reuse `LanguageGrammar` from DKG Phase 1A). Rust: fn/impl/struct/enum/mod boundaries. Python: def/class. TS/Go: function/class. Markdown: headings. Non-code: blank-line paragraphs. Chunk 200-800 tokens, 50-token overlap. Metadata per chunk: file_path, start_line, end_line, chunk_type (function/impl/struct/class/module/paragraph/heading). Tantivy schema extension. Re-index on format version change.

- **3K.2 Query expansion**: 2-3 template-based variants. Strategies: keyword extraction (strip question words), code symbol expansion (CamelCase/snake_case variants), synonym expansion (error<->failure<->exception). BM25 + vector per variant, merge by chunk ID, cross-encoder rerank. Skip for <5 word queries.

- **3K.3 HyDE (Hypothetical Document Embeddings)**: arxiv 2212.10496. Generate hypothetical answer via sidecar LLM (`POST /v1/completions`, max_tokens=128, temperature=0.7, prompt "Write a short technical paragraph answering: {query}") -> embed with 1C vectors -> search with hypothesis -> rerank against ORIGINAL query (not hypothesis). 5-min cache. Graceful degradation: sidecar unavailable -> direct search. 3s timeout. Integrates with Phase 1I resilience circuit breaker.

- **3K.4 Query tier routing**: `QueryTier` enum.
  - FAST: BM25 only. <20ms. Agent context passing, simple lookups.
  - STANDARD: BM25 + vector hybrid (RRF) + cross-encoder. <200ms. Default for code search.
  - DEEP: full pipeline (expansion + HyDE + hybrid + rerank). <5s. Research/debugging/architecture.
  - AUTO routing: <5 words -> FAST; 5-15 words + code keywords -> STANDARD; >15 words, question form, debugging/architecture keywords -> DEEP.
  - Distinct from Phase 1I SearchMode cascade (that's degradation-on-failure; this is quality-tier on healthy pipeline).

- **3K.5 Retrieval quality benchmark**: 30 queries with known-correct chunks from ELAI codebase (10 simple, 10 code-specific, 10 complex). Before/after metrics: Recall@5, Recall@10, MRR, latency p50/p95 per tier. Recorded in `.elai_cc/metrics/phase_3K_rag.toml`. Regression gate: any metric >5% regression must fix.

### Paper-grounded (2026-04 ingestion)

Papers: A-RAG arxiv 2602.03442, CatRAG 2602.01965, Deep Research Survey 2508.12752, Deep Search Agents 2508.05668. Interface maps: `workflow/papers/arag-interface.md`, `catrag-interface.md`.

- **3K.6 A-RAG keyword scoring + progressive disclosure**:
  - Fix A-RAG keyword_search (currently empty stub): `Score_kw(chunk, keywords) = SUM(count(k, text) * len(k))` case-insensitive freq * keyword char length. Longer keywords weighted higher. (arag-interface §72-77).
  - Fix A-RAG semantic_search (stub): use 1C dual-vector (GraphCodeBERT + Granite). 3x oversampling: retrieve top_k*3, group by parent chunk, score = MAX sim among matched results (arag §109-113).
  - Progressive disclosure: search tools return SNIPPETS (matched lines/sentences) not full chunks. `ContextTracker` already prevents re-reading (arag.rs L69-75). Add snippet extraction in keyword_search + semantic_search results. ChunkRead returns full text only when explicitly requested (arag §273-280).
  - Target: >= 50% tokens-per-query reduction. Recall@5 >= baseline.

- **3K.7 CatRAG symbolic anchoring + key-fact enhancement**:
  - Symbolic Anchoring (CatRAG Innovation 1, catrag §72-89): extract named entities from query (rule-based not LLM — CamelCase and UPPER_SNAKE tokens). Match against DKG entity nodes. Weak PPR reset: `reset_weight(e) = epsilon * |passages_containing(e)|^(-1)`. epsilon=0.2 (already in catrag.rs L91 `CatRagConfig`).
  - Key-Fact Passage Enhancement (CatRAG Innovation 3, catrag §196-218): pure algorithmic. For each context edge from entity u to passage p: `w_hat = w * (1 + beta * I(u,p in verified_triples))`. beta=2.5 (catrag.rs L93). Verified triples = DKG edges with high confidence scores.
  - CatRAG Innovation 2 (Dynamic Edge Weighting, LLM-scored) DEFERRED: LLM call per query for neighbor scoring. Too expensive for code retrieval. Re-evaluate after Phase 4D web research pipeline provides LLM-in-loop context.

- **3K.8 Post-improvement benchmark**: rerun 3K.5 with all 3K.1-3K.7 active. Record `.elai_cc/metrics/phase_3K_rag_improved.toml`. Tokens-per-query reduced >=30% (progressive disclosure); Recall@5 improved or unchanged; latency in tier budgets.

---

# End of Phase 3 summary (Parts 1-3)

REMAINING TECH DEBT — ELAI Codebase
Generated: 2026-04-13 (post-audit1 fixes)
All items from outdated-tech-audit.md that were NOT fixed in the first pass.

================================================================================
SECTION A: EASY FIXES (one session, mechanical changes)
================================================================================

Estimated total: ~1 hour of focused work. No architectural changes.

[A-01] H-06 (partial): std::sync::Mutex held across .await in mcp_tool_bridge
  File: crates/orchestrator/src/mcp_tool_bridge.rs:192-203
  Problem: MutexGuard from std::sync::Mutex held across .await calls at lines 197, 201, 203.
  Fix: Change the Mutex at line 75-76 from std::sync::Mutex to tokio::sync::Mutex.
       Update lock() calls to .lock().await. This is the ONLY location that needs
       tokio::sync::Mutex — all other 10 Mutex locations are sync-only (verified).
  Note: The remaining 10 std::sync::Mutex locations are correctly used in sync contexts:
    tools/lib.rs:1265, knowledge/degradation.rs, knowledge/query_trace.rs,
    knowledge/splade.rs:140,143, knowledge/vector/ort_session.rs:116,156,
    providers/anthropic.rs:137,153, providers/prompt_cache.rs:125,
    orchestrator/task_registry.rs:5, orchestrator/worker_boot.rs:16,
    orchestrator/team_cron_registry.rs:8 — all KEEP_STD.

[A-02] H-07 (partial): .unwrap()/.expect() in non-test production code
  Only 9 locations are actually in production code (audit overcounted test code).
  Files and lines:
    crates/tools/src/lib.rs:2078,3911,4935,4947 — .expect() on serialization
    crates/knowledge/src/query_trace.rs:122,135,161 — .expect("stats lock poisoned")
    crates/knowledge/src/vector/mod.rs:209,234 — lock().unwrap() in rerank permit
  Fix per location:
    lib.rs:2078,3911 — serialization .expect() -> .map_err(|e| ToolError::Internal(...))?
    lib.rs:4935,4947 — .expect() on private invariants -> .ok_or(ToolError::Internal(...))?
    query_trace.rs:122,135,161 — .expect("poisoned") -> .map_err(|_| QueryTraceError::LockPoisoned)?
    vector/mod.rs:209 — lock().unwrap() -> lock().map_err(|_| VectorError::LockPoisoned)?
    vector/mod.rs:234 — .unwrap() -> .ok_or(VectorError::MissingDualVector)?
  Note: crates/tools/src/lane_completion.rs:123 is in a #[test] fn (acceptable).
        crates/knowledge/src/degradation.rs — ALL .unwrap() are inside #[cfg(test)] (acceptable).

[A-03] M-01: async-trait -> native async fn in traits
  File: crates/plugins/src/mcp_bridge.rs:36,163
  Cargo: crates/plugins/Cargo.toml:14 (async-trait = "0.1", optional behind "mcp" feature)
  Current:
    Line 36: #[cfg_attr(feature = "mcp", async_trait::async_trait)] on McpBridge trait
    Line 163: #[cfg(feature = "mcp")] #[async_trait::async_trait] on impl block
  Fix: Remove both #[async_trait] attributes. Change trait methods to use native
       `async fn` syntax (stable since Rust 1.75). Remove async-trait from Cargo.toml.
       Test with: cargo check -p elai-plugins --features mcp

[A-04] M-03: glob 0.3 -> remove (walkdir already available)
  Files:
    crates/core/Cargo.toml:9 — glob = "0.3"
    crates/orchestrator/Cargo.toml:14 — glob = "0.3"
    crates/orchestrator/src/file_ops.rs:8 — use glob::Pattern (for pattern matching)
    crates/orchestrator/src/file_ops.rs:314 — glob::glob(&search_pattern) (directory scan)
  Fix:
    file_ops.rs:8 — Replace glob::Pattern with regex::Regex (regex already a dep).
                     Convert glob patterns to regex with simple fn (*, ? -> .*, .?).
    file_ops.rs:314 — Replace glob::glob() with walkdir::WalkDir (already imported line 11)
                       + pattern filter. WalkDir is more capable and already in use.
    Remove glob from both Cargo.toml files.

[A-05] M-04 (partial): Runtime::new() in cli main.rs
  File: crates/cli/src/main.rs:1433,2629,6195
  Current: tokio::runtime::Runtime::new()? (3 locations)
  Fix: Replace with Builder::new_multi_thread()
         .worker_threads(2)
         .thread_name("elai-worker")
         .enable_all()
         .build()?
       This makes thread count explicit and adds observability.
       Line 6195 (main init) should be the single top-level runtime.
       Lines 1433 (login) and 2629 (plugin shutdown) should use
       Handle::current().block_on() if called from async context,
       or tokio::task::block_in_place() if from sync bridge.

[A-06] TODO: session_dag.rs token count heuristic
  File: crates/orchestrator/src/session_dag.rs:199-202
  Current: FIXME(3H.7): estimate_tokens uses bytes.len() / 4 heuristic
  Phase: 3H (Session Handoff)
  Phase file: F:\.ELAI_workflow\docs\specs\guide\steps\phase-3H.txt
  Fix: Wire LlmProvider::token_count() from providers crate. If provider not
       available at callsite, use tokenizers crate (already in knowledge dep)
       with a shared tokenizer instance. Fallback to heuristic if tokenizer
       unavailable.
  Files to modify:
    crates/orchestrator/src/session_dag.rs — replace heuristic with tokenizer call
    crates/orchestrator/Cargo.toml — may need tokenizers dep (or pass via trait)

[A-07] TODO: fsm.rs telemetry + SSB WorkloadProfile
  File: crates/orchestrator/src/fsm.rs:431-443
  Current: Two TODO comments after trigger_preload call:
    1. Log/telemetry for pre-loaded models
    2. Write WorkloadProfile to SSB (tokens_in/out, latency_ms, vram_peak_mb)
  Phase: 3J.7 (workload profiling)
  Phase file: F:\.ELAI_workflow\docs\specs\guide\steps\phase-3J.txt
  Fix: Add tracing::info! for pre-load events. Create WorkloadProfile struct
       and write to SSB via existing ssb.append() at phase transitions.
  Files to modify:
    crates/orchestrator/src/fsm.rs:431-443 — add logging + SSB write
    crates/orchestrator/src/workload_profiling.rs — WorkloadProfile struct (may exist)
    crates/orchestrator/src/ssb.rs — ensure WorkloadProfile payload type exists

[A-08] TODO: tiered_analysis.rs rules.db load (T1 empty slice)
  File: crates/plugins/src/tiered_analysis.rs:108-110
  Current: // TODO: Load rules from rules.db — uses empty slice, all T1 checks are no-ops
  Phase: 3F (Rules Engine)
  Phase file: F:\.ELAI_workflow\docs\specs\guide\steps\phase-3F.txt
  Fix: Wire RuleEngine::get_active_rules(phase, domain) at T1 hook entry.
       Rules engine and schema already exist in crates/rules/.
  Files to modify:
    crates/plugins/src/tiered_analysis.rs:108-110 — load rules from engine
    crates/plugins/Cargo.toml — may need elai-rules dep (check if already present)

[A-09] TODO: vllm_client.rs place_models wiring
  File: crates/providers/src/vllm_client.rs:517-521
  Current: Doc-block TODO: needs real GPU discovery (NVML), real model VRAM metadata,
           and agent-pool integration. Function body is implemented (GreedyModelPlacer).
  Phase: 3J.8 (MILP scheduling)
  Phase file: F:\.ELAI_workflow\docs\specs\guide\steps\phase-3J.txt
  Fix: Wire nvml-wrapper for GPU enumeration at callsites. Read model VRAM from
       config.json metadata. Connect to agent_pool phase boundary hooks.
  Files to modify:
    crates/providers/src/vllm_client.rs — add NVML GPU discovery
    crates/providers/src/model_placer.rs — real GpuInfo from NVML
    crates/providers/Cargo.toml — add nvml-wrapper dep


================================================================================
SECTION B: HARD FIXES (one session each, architectural changes)
================================================================================

Each item below is a separate session. Do not combine them.

--------------------------------------------------------------------------------
[B-01] C-02 + C-03 + C-04: Python sidecar rewrite (HTTPServer -> FastAPI)
  Scope: MAJOR REWRITE — replaces the entire server framework + inference loop
  Estimated effort: Large (500+ lines changed)
  Phase: Part of 3J.COMPOSE or could be standalone improvement
  Phase file: F:\.ELAI_workflow\docs\specs\guide\steps\phase-3J.txt (see 3J.COMPOSE)

  Current state:
    python/elai_inference/server.py:
      Line 18: from http.server import HTTPServer, BaseHTTPRequestHandler
      Line 533: class InferenceHandler(BaseHTTPRequestHandler) — sync, single-threaded
      Line 1098: HTTPServer((host, port), InferenceHandler) — blocking serve_forever()
      Line 360-459: Manual token loop (for step in range(max_tokens): model() per token)
      Line 796-912: Fake streaming — generates all tokens, then splits into SSE chunks

  What needs to change:
    1. Replace http.server with FastAPI + uvicorn (async ASGI)
       - pip deps: fastapi, uvicorn[standard]
       - New: app = FastAPI(), @app.post("/v1/completions"), @app.post("/v1/messages")
       - Inference runs in ThreadPoolExecutor (model calls are sync/GPU-bound)
    2. Replace manual token loop with model.generate() + custom LogitsProcessor
       - Subclass transformers.LogitsProcessor for DoLa layer-contrast selection
       - model.generate(logits_processor=[DoLaProcessor(...)]) runs in C++/CUDA
       - 10-50x faster than Python per-token loop
    3. True streaming via async generator + StreamingResponse
       - Use TextIteratorStreamer from transformers (feeds tokens to queue)
       - FastAPI StreamingResponse wraps the async generator
       - Client gets tokens as they're generated, not after full completion

  Files to modify:
    python/elai_inference/server.py — near-total rewrite
    python/elai_inference/pyproject.toml — add fastapi, uvicorn[standard] deps
    crates/cli/src/main.rs — update sidecar startup command if CLI launches it

  Risk: High. The sidecar is the inference backbone for --local mode. Need to preserve:
    - /v1/completions and /v1/messages endpoint contracts (Anthropic protocol)
    - DoLa layer-contrast decoding behavior
    - TurboQuant KV compression integration
    - Steering vector application
    - /health and /shutdown endpoints

--------------------------------------------------------------------------------
[B-02] H-08: Box<dyn Error> -> typed errors (cli + knowledge)
  Scope: MEDIUM REWRITE — changes 25+ function signatures + error propagation
  Estimated effort: Medium (signature changes cascade through callers)

  Current state:
    crates/cli/src/main.rs — 21 functions return Box<dyn Error>
    crates/knowledge/src/arag.rs:102,144 — 2 functions
    crates/knowledge/src/catrag.rs:147 — 1 function
    crates/knowledge/src/pipeline.rs:103 — 1 function (trait definition)

  What needs to change:
    1. cli crate: Replace all Box<dyn Error> with anyhow::Result
       - anyhow is already a workspace dep (underused per audit)
       - Pure search-replace: Box<dyn std::error::Error> -> anyhow::Result
       - Callers: .map_err(|e| e.into()) becomes automatic with anyhow
       - Add anyhow = { workspace = true } to crates/cli/Cargo.toml
    2. knowledge crate: Define KnowledgeError with thiserror
       - Create crates/knowledge/src/error.rs with #[derive(thiserror::Error)]
       - Variants: Io, Sqlite, Bm25, Vector, Arag, CatRag, Pipeline, etc.
       - Replace Box<dyn Error> in arag.rs, catrag.rs, pipeline.rs
       - Update KnowledgePipeline trait (pipeline.rs:103) — this cascades to all impls

  Files to modify:
    crates/cli/src/main.rs — 21 signature changes
    crates/cli/Cargo.toml — add anyhow dep
    crates/knowledge/src/error.rs — new file (KnowledgeError enum)
    crates/knowledge/src/arag.rs — 2 signature changes
    crates/knowledge/src/catrag.rs — 1 signature change
    crates/knowledge/src/pipeline.rs — 1 trait signature change (cascades to impls)
    crates/knowledge/src/lib.rs — re-export error type

  Risk: Medium. The trait change in pipeline.rs:103 cascades to all implementations.
        Run cargo check --workspace after each file change.

--------------------------------------------------------------------------------
[B-03] M-02: r2d2 -> deadpool-sqlite (async connection pool)
  Scope: MEDIUM — replaces sync pool with async pool in SSB
  Estimated effort: Medium

  Current state:
    crates/orchestrator/Cargo.toml:19-20 — r2d2 = "0.8", r2d2_sqlite = "0.33"
    crates/orchestrator/src/ssb.rs:
      Line 14: use r2d2::Pool
      Line 15: use r2d2_sqlite::SqliteConnectionManager
      Line 570: Pool(#[from] r2d2::Error) in SsbError enum
      Lines 595-603: SsbPragmaApplier implements r2d2::CustomizeConnection

  What needs to change:
    1. Replace r2d2 + r2d2_sqlite with deadpool-sqlite (or bb8-rusqlite)
       - deadpool-sqlite is async-native, works with tokio
       - Read pool becomes: Pool<deadpool_sqlite::Manager>
       - pool.get().await? instead of pool.get()?
    2. Migrate SsbPragmaApplier from r2d2::CustomizeConnection to
       deadpool hook (post_create callback)
    3. Update all pool.get() callsites to .await

  Files to modify:
    crates/orchestrator/Cargo.toml — remove r2d2/r2d2_sqlite, add deadpool-sqlite
    crates/orchestrator/src/ssb.rs — rewrite pool setup, pragma applier, pool.get() calls

  Risk: Medium. SSB is the inter-agent communication backbone. Write path uses
        tokio::mpsc (unchanged). Only read path uses the pool.
        The write channel pattern stays — only the read pool changes.

--------------------------------------------------------------------------------
[B-04] TODO: preload.rs + steering.rs (model pre-loading + steering vector stubs)
  Scope: MEDIUM — requires ModelPool integration + file I/O for vectors
  Estimated effort: Medium
  Phase: 3J.6 (pre-loading) + 3J.5 (steering vectors)
  Phase file: F:\.ELAI_workflow\docs\specs\guide\steps\phase-3J.txt

  Current state:
    crates/orchestrator/src/preload.rs:41-49
      trigger_preload is a structural placeholder. Needs tokio::spawn, ModelPool
      VRAM check via MultiplexingPolicy.can_colocate(), GreedyModelPlacer integration.
    crates/providers/src/layer_config/steering.rs:
      Line 325-330: SteeringEngine::from_config — TODO for vector loading from
                    .safetensors/.npz. Returns empty engine with warning.
      Line 367-372: apply_steering — TODO for vLLM backend integration.
                    Logs warning, no-op. Coupled to DoLa hook (3J.3).
      Line 395-409: compute_vector — TODO for forward-pass activation capture.
                    Returns Err(StorageError) unconditionally.

  What needs to change:
    1. preload.rs: Implement background model loading via tokio::spawn
       - Check VRAM with multiplexing policy
       - Load model via ModelPool
       - Report status to SSB
    2. steering.rs:325: Implement vector file loading
       - Read .safetensors via safetensors crate (or shell out to Python)
       - Read .npz via manual parsing or ndarray-npy crate
       - Store vectors in SteeringEngine
    3. steering.rs:367: Implement apply_steering for vLLM backend
       - Requires vLLM layer-hook API (depends on vLLM feature being active)
       - Inject vector at target layer during forward pass
    4. steering.rs:395-409: Implement compute_vector
       - Requires model forward pass with activation capture
       - Needs vLLM or sidecar to expose intermediate hidden states

  Files to modify:
    crates/orchestrator/src/preload.rs — implement trigger_preload
    crates/providers/src/layer_config/steering.rs — implement 3 methods
    crates/providers/Cargo.toml — may need safetensors dep

  Risk: Medium-High. Steering via vLLM requires vLLM to expose layer hooks,
        which may not be available in the current vLLM version.

--------------------------------------------------------------------------------
[B-05] TODO: tiered_analysis.rs T2 tree-sitter hook
  Scope: MEDIUM — requires tree-sitter integration in plugins crate
  Estimated effort: Medium
  Phase: 3J.11 (tiered static analysis)
  Phase file: F:\.ELAI_workflow\docs\specs\guide\steps\phase-3J.txt

  Current state:
    crates/plugins/src/tiered_analysis.rs:208-210
      T2 hook returns HookVerdict::Allow unconditionally.
      Entire AST-check tier is disabled.

  What needs to change:
    1. Add tree-sitter dep to plugins crate
    2. Parse the file being checked with tree-sitter
    3. Run AST pattern matches against T2 rules from rules.db
    4. Return Allow/Warn/Block based on matches
    Target latency: <10ms for files under 5K lines

  Files to modify:
    crates/plugins/Cargo.toml — add tree-sitter, tree-sitter-rust deps
    crates/plugins/src/tiered_analysis.rs:208-217 — implement T2 parse + match
    May need to import rule patterns from crates/rules/

  Risk: Low-Medium. T2 is advisory (Warn only, no Block). Can be feature-gated.

--------------------------------------------------------------------------------
[B-06] TODO: sandbox.rs Windows API stubs (6 locations)
  Scope: LARGE — Windows FFI for AppContainer + Job objects
  Estimated effort: Large
  Phase: 4A (Agent Process Isolation)
  Phase file: F:\.ELAI_workflow\docs\specs\guide\steps\phase-4A.txt

  Current state:
    crates/orchestrator/src/sandbox.rs:
      Line 229: create_appcontainer_profile — returns Ok(placeholder)
      Line 243: create_job_object — returns Ok(0)
      Line 254: create_restricted_token — returns Ok(0)
      Line 286: launch_agent — returns Ok(AgentProcess::placeholder())
      Line 299: cleanup — returns Ok(())
      Line 325: Linux clone()+seccomp — returns Err("deferred to 4A")

  What needs to change:
    1. Windows path: Use windows-sys crate for:
       - CreateAppContainerProfile (line 229)
       - CreateJobObject + SetInformationJobObject (line 243)
       - CreateRestrictedToken (line 254)
       - CreateProcessW with AppContainer security (line 286)
       - DeleteAppContainerProfile on cleanup (line 299)
    2. Linux path: Use libc crate for:
       - clone(CLONE_NEWNS | CLONE_NEWPID | CLONE_NEWNET)
       - seccomp-bpf filter (via libseccomp or seccomp-sys)
       - prctl(PR_SET_NO_NEW_PRIVS)

  Files to modify:
    crates/orchestrator/Cargo.toml — add windows-sys (Windows), libc+seccomp (Linux)
    crates/orchestrator/src/sandbox.rs — implement all 6 stubs
    May need crates/orchestrator/src/sandbox/ directory split (windows.rs, linux.rs)

  Risk: High. OS-level isolation. Incorrect implementation = false sense of security.
        Needs thorough testing on both platforms. Phase 4A spec has detailed requirements.

================================================================================
SECTION C: DEFERRED / WON'T FIX
================================================================================

These items were in the original audit but are intentionally left as-is.

[D-01] C-06: ort 2.0.0-rc.12 — no stable 2.0 exists on crates.io yet.
       Pinned to =2.0.0-rc.12 explicitly. Update when stable releases.

[D-02] H-06 (10 of 11 locations): std::sync::Mutex in async code — all 10
       remaining locations verified to be sync-only (no .await while held).
       Only mcp_tool_bridge.rs needs tokio::sync::Mutex (see A-01).

[D-03] L-01 through L-08: Low-priority style/housekeeping items.
       Not worth session time. Fix opportunistically during nearby changes.

[D-04] M-05 through M-09, M-11 through M-14, M-16: Medium modernization items.
       Each is a small improvement but not blocking. Fix opportunistically.
       - M-05: `as` casts — fix during nearby edits
       - M-06: unnecessary .clone() — fix during nearby edits
       - M-07: std::thread::spawn in async — fix during nearby edits
       - M-08: magic numbers — fix during nearby edits
       - M-09: manual serde impl — fix during nearby edits
       - M-11: json -> orjson in sidecar — fix with B-01 (sidecar rewrite)
       - M-12: logging.basicConfig -> structlog — fix with B-01
       - M-13: argparse -> typer — fix with B-01
       - M-14: f-string in log calls — fix with B-01
       - M-16: hardcoded model names in CLI — config-file driven, fix when config exists

OUTDATED TECHNOLOGY AUDIT — ELAI Codebase
Generated: 2026-04-13
Scope: F:\.ELAI_workflow\.elai_cc\ (all Rust crates + Python sidecar)

================================================================================
SECTION 1: CRITICAL (correctness bugs or major performance issues)
================================================================================

[C-01] Python sidecar: dtype= vs torch_dtype= (SILENT BUG)
  File: python/elai_inference/server.py:321
  Current: model = AutoModelForCausalLM.from_pretrained(model_name, dtype=torch.float16, ...)
  Problem: `dtype=` is NOT a valid from_pretrained kwarg. It is silently ignored.
           Model loads in fp32 (default), wasting 2x VRAM and negating the 4-bit config
           for non-quantized layers.
  Fix: Change to `torch_dtype=torch.float16` (or torch.bfloat16 for Ampere+ GPUs)

[C-02] Python sidecar: http.server.HTTPServer (single-threaded, blocking)
  File: python/elai_inference/server.py:18, 522, 1087
  Current: from http.server import HTTPServer, BaseHTTPRequestHandler
  Problem: One request blocks all others. No concurrency. No streaming. No keep-alive.
           Python docs explicitly say "not recommended for production."
  Fix: FastAPI + uvicorn (async ASGI). Inference runs in thread pool executor.

[C-03] Python sidecar: Fake streaming (stream= parameter ignored)
  File: python/elai_inference/server.py:360, 796-912
  Current: stream=False parameter accepted but ignored. "Streaming" SSE response
           generates ALL tokens first, then splits the complete text word-by-word.
  Problem: Client gets no tokens until full generation completes. Defeats purpose of SSE.
  Fix: True token-by-token streaming via async generator + FastAPI StreamingResponse.

[C-04] Python sidecar: Manual token loop instead of model.generate()
  File: python/elai_inference/server.py:391-436
  Current: Hand-written `for step in range(max_tokens)` calling model() per token in Python.
  Problem: 10-50x slower than model.generate() which runs in C++/CUDA. Python loop
           overhead per token, no CUDA graph, no compiled kernels.
  Fix: model.generate() with custom LogitsProcessor subclass for DoLa logic.

[C-05] Python sidecar: KV cache quantization triple-nested Python loop
  File: python/elai_inference/server.py:262-279
  Current: Iterates batch x heads x seq_len in Python, calling .quantize/.dequantize per vector.
  Problem: For 27B model (64 heads, 2048 ctx) = ~131k Python calls per generation step.
  Fix: Vectorize: accept entire (batch, n_heads, seq_len, head_dim) tensor at once,
       or use torch.compile to fuse the quantize loop.

[C-06] ort = "2.0.0-rc.12" (pre-release ONNX Runtime binding)
  File: crates/knowledge/Cargo.toml:19
  Current: ort = "2.0.0-rc.12"
  Problem: Release candidate. Stable ort 2.0 was released. API may have changed.
  Fix: ort = "2" (stable)

[C-07] rmp-serde version mismatch (spec=1.3, lock may disagree)
  File: crates/orchestrator/Cargo.toml:22, crates/contracts/Cargo.toml:23
  Current: rmp-serde = "1.3" in spec
  Problem: Lock file may show different resolved version. Could mean serialization
           behavior differs from expected. Used for checkpoints, BSP payloads, FSM state.
  Action: Run `cargo tree -p rmp-serde` to verify. Re-lock if necessary.

[C-08] usearch version mismatch (spec=2.24.0, lock=2.1.3)
  File: crates/knowledge/Cargo.toml:25
  Current: usearch = { version = "2.24.0", ... }
  Problem: Cargo.toml requests 2.24.0 but lock resolves 2.1.3. Impossible under
           normal semver (^2.24.0 >= 2.24). Likely typo.
  Fix: Change to version = "2" and cargo update.

[C-09] transformers>=5.0.0 DynamicCache API breakage
  File: metrics/requirements.txt:7, metrics/turboquant_quality_eval.py:76-113
  Current: transformers>=5.0.0
  Problem: transformers 5.x changed DynamicCache API. Code accesses
           past_key_values.layers[layer_idx].keys which is a 4.x pattern.
  Fix: Pin to >=4.45,<5 OR update .layers[i].keys access for 5.x API.

================================================================================
SECTION 2: HIGH (outdated deps, wrong patterns, rule violations)
================================================================================

[H-01] blake2 used where blake3 should be (inconsistency)
  Files:
    Cargo.toml:33 (workspace dep blake2 = "0.10")
    crates/knowledge/src/dkg/mod.rs:4 — use blake2::{Blake2s256, Digest}
    crates/knowledge/src/vector/cache.rs:1,14,68,76-91 — content_blake2s BLOB PRIMARY KEY
    crates/rules/src/self_discovery.rs:10 — blake2 for violation dedup
    crates/plugins/src/tiered_analysis.rs:10 — blake2 for plugin analysis
  Problem: blake3 is already used in orchestrator crate (anti-spiral, context budget,
           session DAG). blake3 is 3-5x faster than blake2s. Half uses blake2, half blake3.
  Fix: Replace all blake2 with blake3 = "1". Migrate SQLite column names.
       NOTE: vector/cache.rs has "content_blake2s" as a DB column — migration needed.

[H-02] sha2 for non-PKCE content hashing
  Files:
    crates/core/Cargo.toml:13 — sha2 = "0.10"
    crates/providers/Cargo.toml:15 — sha2 = "0.10"
    crates/orchestrator/src/checkpoint.rs:5,173-175,230-231 — SHA-256 for checkpoint hash
  Note: crates/providers/src/oauth.rs:8 — SHA-256 for PKCE (KEEP — RFC 7636 requires it)
  Fix: Replace checkpoint.rs sha2 with blake3. Keep sha2 ONLY for PKCE in oauth.rs.

[H-03] rand = "0.8" / rand_chacha = "0.3" (two major versions behind)
  Files:
    crates/core/Cargo.toml:14-15
    crates/core/src/entropy.rs:1-3,22,26
  Current: rand 0.8.5. Current is rand 0.9/0.10. Lock shows 3 rand versions compiled.
  Fix: Upgrade to rand = "0.9" or "0.10", rand_chacha = "0.10".

[H-04] getrandom = "0.2" (outdated)
  File: crates/providers/Cargo.toml:14
  Current: getrandom 0.2.24. Current is getrandom 0.3.
  Fix: getrandom = "0.3"

[H-05] log + tracing dual logging facade
  Files using `log::`:
    crates/knowledge/src/dkg/mod.rs:179,193,206,313
    crates/knowledge/src/splade.rs:235,248,260,264
    crates/knowledge/src/pipeline.rs:152,158,236,295,335,394,412,416,431,444
    crates/knowledge/src/reranker.rs:303,346,436
    crates/knowledge/src/startup.rs:146,183,191,194,234,246
    crates/knowledge/src/vector/indexer.rs:141,162
    crates/plugins/src/tiered_analysis.rs:133,202,216,271
    crates/plugins/src/mcp_lifecycle.rs:128
  Files using `tracing::`:
    providers crate, orchestrator crate
  Problem: Two logging systems. tracing is the modern Rust standard for async.
  Fix: Replace all log:: with tracing::. Add tracing-log bridge. Remove log dep.

[H-06] std::sync::Mutex in async code (11 locations)
  Files:
    crates/tools/src/lib.rs:4
    crates/knowledge/src/degradation.rs:2
    crates/knowledge/src/query_trace.rs:2
    crates/knowledge/src/splade.rs:110,116 — Arc<Mutex<Session>> held across ONNX calls
    crates/knowledge/src/vector/ort_session.rs:36 — Arc<Mutex<Session>>
    crates/providers/src/providers/anthropic.rs:121
    crates/providers/src/prompt_cache.rs:110
    crates/orchestrator/src/task_registry.rs:57
    crates/orchestrator/src/mcp_tool_bridge.rs:75-76
    crates/orchestrator/src/worker_boot.rs:147
    crates/orchestrator/src/team_cron_registry.rs:53,138
  Problem: std::sync::Mutex blocks the tokio executor when held across .await points.
  Fix: Replace with tokio::sync::Mutex where lock is held across .await.
       Keep std::sync::Mutex only for provably sync-only critical sections.

[H-07] .unwrap() in non-test production code (rule violation)
  Files:
    crates/tools/src/lib.rs:2078,3911,4935,4947 — .expect() in production fns
    crates/tools/src/lane_completion.rs:123 — result.unwrap()
    crates/knowledge/src/degradation.rs:205,211,248,263,277-278,292-293,307-309,324-325,
      333,349,357,376,380,428-430,448,454,459,476 — lock().unwrap() throughout
    crates/knowledge/src/query_trace.rs:420 — lock().unwrap() in Drop
    crates/knowledge/src/vector/mod.rs:209 — lock().unwrap()
  Fix: Replace with ? / .ok_or() / .map_err() per project rules.

[H-08] Box<dyn Error> instead of thiserror/anyhow
  Files:
    crates/cli/src/main.rs — ~60+ function signatures
    crates/knowledge/src/bm25.rs:119,181,207,226,278,293,326,383
    crates/knowledge/src/dkg/mod.rs:83,117,287,346,371,385,409,423,437
    crates/knowledge/src/vector/indexer.rs:57,71,89,194,218
    crates/knowledge/src/arag.rs:102,144
    crates/knowledge/src/catrag.rs:147
    crates/knowledge/src/pagerank/mod.rs:28
    crates/knowledge/src/pipeline.rs:103
    crates/knowledge/src/lib.rs:106-109 (trait definition)
  Problem: Erases error context, cannot be pattern-matched, poor diagnostics.
           Project already has thiserror/anyhow as deps.
  Fix: cli crate -> anyhow::Result. Library crates -> thiserror typed errors.

[H-09] println!/eprintln! for internal diagnostics (not user-facing TUI)
  Files:
    crates/orchestrator/src/agent_pool.rs:226,301 — eprintln!("Failed to create sandbox")
    crates/cli/src/main.rs:3013,3076,3093,3097,3125,3128,3142-3146 — diagnostic noise
  Fix: Replace with tracing::warn! / tracing::info!

[H-10] Python sidecar: torch.no_grad() instead of torch.inference_mode()
  File: python/elai_inference/server.py:392,172,181
  Problem: torch.no_grad() still tracks some autograd metadata. inference_mode() is faster.
  Fix: Replace all `with torch.no_grad():` with `with torch.inference_mode():`

[H-11] Python sidecar: Steering vectors re-transferred CPU->GPU every step
  File: python/elai_inference/server.py:142 (stored on CPU), 213 (.to(device) per step)
  Fix: Move vectors to target device at load time, keep them there.

[H-12] Python sidecar: bitsandbytes not in requirements.txt
  File: python/elai_inference/server.py:305-317
  Problem: BitsAndBytesConfig used but bitsandbytes not declared as dependency.
  Fix: Add bitsandbytes>=0.43.0 to requirements (with platform note).

[H-13] Python sidecar: safetensors not in requirements.txt
  File: python/elai_inference/server.py:150-154
  Fix: Add safetensors>=0.4.0 to requirements.

[H-14] Python sidecar: NO requirements.txt/pyproject.toml exists at all
  Location: python/elai_inference/ has zero dependency declaration files.
  Fix: Create pyproject.toml with all deps pinned.

================================================================================
SECTION 3: MEDIUM (modernization, performance, maintainability)
================================================================================

[M-01] async-trait = "0.1" replaceable with native async fn in traits
  File: crates/plugins/Cargo.toml:14, crates/plugins/src/mcp_bridge.rs:36,163
  Since: Rust 1.75+ (Dec 2023) supports native async fn in traits.
  Fix: Remove async-trait, use native syntax. Eliminates Box<dyn Future> indirection.

[M-02] r2d2 = "0.8" / r2d2_sqlite = "0.33" (aging sync connection pool)
  File: crates/orchestrator/Cargo.toml:19-20, crates/orchestrator/src/ssb.rs:4,14-15
  Problem: r2d2 last updated 2020. Sync pool in async codebase.
  Fix: deadpool-sqlite for async-native pooling (or keep if sync is intentional).

[M-03] glob = "0.3" (old, 2016-era crate)
  File: crates/core/Cargo.toml:9, crates/orchestrator/Cargo.toml:14
  Fix: globset or ignore crate (more capable). walkdir already in use.

[M-04] Scattered Runtime::new() + block_on() (old async bridging)
  Files:
    crates/cli/src/main.rs:1433-1434,2510,2629-2630,2708,2731,2751,2767,2799,3434,3507,6195
    crates/providers/src/providers/anthropic.rs:649-655 — NEW runtime per OAuth refresh!
    crates/tools/src/lib.rs:3664,3676,3705
    crates/orchestrator/src/bash.rs:102
    crates/orchestrator/src/mcp_tool_bridge.rs:185,190
  Problem: Creating throwaway Tokio runtimes throughout. anthropic.rs:649-655 is worst:
           new multi-threaded runtime per OAuth refresh.
  Fix: Single top-level runtime. Use Handle::current().block_on() where sync bridge needed.

[M-05] `as` casts without .try_into() (truncation risk)
  Key locations:
    crates/knowledge/src/vector/circuit_breaker.rs — 8x .as_millis() as u64 (u128->u64)
    crates/knowledge/src/splade.rs:134 — *id as usize (token ID, could OOB)
    crates/cli/src/main.rs:3088 — .as_secs() as i64
    crates/knowledge/src/bm25.rs — multiple as u64/u32
  Fix: Use .try_into().unwrap_or() or .try_into()? for boundary values.

[M-06] Unnecessary .clone() in hot paths
  Files:
    crates/knowledge/src/dkg_adapter.rs:115,143-144,146,163-164 — BFS accumulates clones
    crates/knowledge/src/arag.rs:123,126,170 — chunk_id cloned multiple times
    crates/providers/src/providers/anthropic.rs:640-643 — token/scope clones on rebuild
  Fix: Use references or .into_iter() to consume instead of clone.

[M-07] std::thread::spawn in async context
  Files:
    crates/cli/src/main.rs:5950 — heartbeat thread (should be tokio::spawn)
    crates/orchestrator/src/hooks.rs:937 — hook subprocess
  Fix: tokio::spawn for async-compatible contexts.

[M-08] Magic numbers without named constants
  Files:
    crates/cli/src/main.rs:394 (8100), 2274/2323 (120), 2324/2342 (2000/5000),
    2984/3047 (200_000 duplicated), 3116-3117 (512/768/384), 3385/3388 (200/1024)
  Fix: Named constants (DEFAULT_SIDECAR_PORT, SIDECAR_STARTUP_TIMEOUT_SECS, etc.)

[M-09] Manual serde impl where derive would work
  File: crates/knowledge/src/vector/indexer.rs:275-310
  Problem: ChunkMeta has simple scalar/string fields, all derivable.
  Fix: #[derive(Serialize, Deserialize)]

[M-10] Python sidecar: bnb_4bit_compute_dtype=torch.float16 (should be bfloat16)
  File: python/elai_inference/server.py:316-318
  Fix: bnb_4bit_compute_dtype=torch.bfloat16, add bnb_4bit_use_double_quant=True

[M-11] Python sidecar: json.loads/dumps (should be orjson)
  File: python/elai_inference/server.py:553,792,967,705,877,957,1013
  Fix: orjson.loads/orjson.dumps (3-10x faster, returns bytes directly)

[M-12] Python sidecar: logging.basicConfig (unstructured)
  File: python/elai_inference/server.py:1054-1057
  Fix: structlog with JSON output, or loguru.

[M-13] Python sidecar: argparse boilerplate
  File: python/elai_inference/server.py:1026-1052
  Fix: typer (zero boilerplate, auto --help, type validation)

[M-14] Python sidecar: f-string in log calls (eager evaluation)
  File: python/elai_inference/server.py (multiple)
  Current: logger.info(f"VRAM [{label}]: ...")
  Fix: logger.info("VRAM [%s]: ...", label) (lazy formatting)

[M-15] Hardcoded absolute Windows paths
  File: metrics/compute_steering_vector.py:16-17
  Current: MODEL_PATH = r"F:\.ELAI_workflow\.elai_cc\models\gemma-4-E2B-it"
  Fix: Use relative paths, env vars, or config file.

[M-16] Hardcoded model names in CLI
  File: crates/cli/src/main.rs:3111-3182
  Current: "graphcodebert.onnx", "granite-embed.onnx", etc. as string literals
  Fix: Config-file-driven model paths.

[M-17] Python sidecar: torch.cuda.get_device_properties(0) hardcoded
  File: python/elai_inference/server.py:288-295
  Fix: Use torch.cuda.current_device() or self.model.device

================================================================================
SECTION 4: LOW (style, housekeeping, minor improvements)
================================================================================

[L-01] String::from vs .to_string() vs .into() inconsistency
  Primary: crates/tools/src/lib.rs (~50+ occurrences)
  Not a bug, but inconsistent style.

[L-02] Python: import sys unused (line 16)
[L-03] Python: deferred imports of threading (543) and re (715) should be top-level
[L-04] Python: hasattr(tokenizer, 'apply_chat_template') guard is vestigial (370-377)
[L-05] Python: manual special-token stripping (459,462-463) — use skip_special_tokens=True
[L-06] Python: str.endswith() for file extensions — use pathlib.Path.suffix
[L-07] Python: no type annotations anywhere
[L-08] tokenizers = "0.22" with onig feature (heavy, may not need onig)
  File: crates/knowledge/Cargo.toml:20
  Fix: Evaluate latest version + fancy-regex over onig.

================================================================================
SECTION 5: TECH STACK SUMMARY
================================================================================

RUST CRATES (with status):
  serde 1 / serde_json 1          — CURRENT
  tokio 1 (1.51.0)                — CURRENT
  tracing 0.1 (0.1.44)            — CURRENT
  rusqlite 0.39                    — CURRENT
  thiserror 2                      — CURRENT
  anyhow (present but underused)   — CURRENT, USE MORE
  uuid 1                           — CURRENT
  regex 1                          — CURRENT
  blake2 0.10                      — OUTDATED (replace with blake3)
  blake3 1.8                       — CURRENT (already used in orchestrator)
  sha2 0.10                        — CURRENT (keep only for PKCE)
  rand 0.8 / rand_chacha 0.3      — OUTDATED (current: 0.9/0.10)
  getrandom 0.2                    — OUTDATED (current: 0.3)
  log 0.4                          — OUTDATED (replace with tracing)
  tantivy 0.26                     — CURRENT
  tree-sitter 0.26                 — CURRENT
  ort 2.0.0-rc.12                  — OUTDATED (stable 2.0 released)
  tokenizers 0.22                  — CHECK (may be current, evaluate onig)
  usearch 2.24.0 (version mismatch)— FIX VERSION SPEC
  jsonschema 0.45                  — CURRENT
  proptest 1.11                    — CURRENT
  rmp-serde 1.3                    — VERIFY LOCK
  postcard (wire format)           — CURRENT
  r2d2 0.8                         — AGING (consider deadpool)
  async-trait 0.1                  — OUTDATED (native async traits since Rust 1.75)
  glob 0.3                         — OUTDATED (use globset/ignore)
  crossterm 0.28                   — CURRENT
  reqwest 0.12                     — CURRENT
  indexmap 2                       — CURRENT
  walkdir 2                        — CURRENT
  tempfile 3                       — CURRENT
  static_assertions                — CURRENT

PYTHON (sidecar):
  torch                            — CURRENT (but manual loop is anti-pattern)
  transformers (>= 5.0.0)         — VERSION PIN ISSUE
  numpy                            — CURRENT
  bitsandbytes                     — UNDECLARED DEPENDENCY
  safetensors                      — UNDECLARED DEPENDENCY
  http.server                      — OUTDATED (replace with fastapi+uvicorn)
  argparse                         — FUNCTIONAL (typer is modern alternative)
  logging                          — FUNCTIONAL (structlog is modern alternative)
  json                             — FUNCTIONAL (orjson is faster)

PYTHON (metrics/):
  transformers >= 5.0.0            — API BREAKAGE RISK
  accelerate >= 1.0.0              — CURRENT
  scipy >= 1.10.0                  — CURRENT

================================================================================
SECTION 6: TODO STUBS IN PRODUCTION CODE (not outdated, but incomplete)
================================================================================

  crates/orchestrator/src/fsm.rs:431-439 — telemetry, pre-loading, SSB integration
  crates/orchestrator/src/preload.rs:41 — background pre-loading
  crates/orchestrator/src/sandbox.rs:229,243,254,286,299,325 — Windows API calls stubbed
  crates/plugins/src/tiered_analysis.rs:109,210,212 — rules.db load, tree-sitter T2 hook
  crates/providers/src/layer_config/steering.rs:325,367,395,401,405 — vector loading/storage
  crates/orchestrator/src/session_dag.rs:199 — token count FIXME
  crates/providers/src/vllm_client.rs:517 — TODO
  crates/knowledge/src/pagerank/mod.rs — compute_repo_map() is todo!()

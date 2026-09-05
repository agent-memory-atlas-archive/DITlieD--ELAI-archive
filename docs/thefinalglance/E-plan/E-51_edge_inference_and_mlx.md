# E-51: Apple MLX + llama.cpp/GGUF as new B-37 RuntimeAdapter variants — Anthropic /v1/messages wrappers, NPU deferred, Foundation Models bridge spec
Objective: Extend B-37's existing 11-backend RuntimeAdapter with two new variants — Mlx (Apple fast-path via vllm-mlx; +1.5-3x over llama.cpp on M-series; reaches 158 tok/s on Qwen 1.7B / M5 Max) and LlamaCpp (universal GGUF fallback; covers Win/Mac/Linux/ChromeOS via one binary with auto-detected CUDA/Vulkan/Metal/HIP EP). Both wrap a process that already speaks Anthropic /v1/messages so the existing CloudGroundingClient parser, IPC v2 surface and PerRoleRuntime mapping flow through unchanged. NPU paths (Snapdragon QNN, Intel OpenVINO, AMD Ryzen AI, Apple ANE) are deferred per report; Foundation Models framework and transformers.js+WebGPU get spec-only stubs so future work has a target.
Estimated effort: 2-3 sessions, ~10-12 h
Pre-requisites: B-37 SHIPPED (commit fd5c2e5 — Multi-Runtime Backend Abstraction; `crates/providers/src/runtime/types.rs::RuntimeBackend` enum at elai.map line 318213; `crates/providers/src/runtime/types.rs::RuntimeAdapter` trait at line 326789; `crates/orchestrator/src/per_role_runtime.rs::PerRoleRuntime` at line 6155; `crates/orchestrator/src/runtime_orchestrator.rs::RuntimeOrchestrator` at line 8064). E-2 SHIPPED (Local LLM speed tricks — FA2/SDPA/NF4 baseline already produces the canonical Anthropic /v1/messages surface this E-N piggybacks on). E-26 SHIPPED (Inference engines 2026 — provides the broader engine landscape this E-N narrows to Apple+GGUF).
Parallel-safe with: E-52 (LSP integration — disjoint surface), E-53 (formal verification — disjoint), E-54 (eval corpus — disjoint), E-55 (agentic RL — disjoint), E-20 (local/cloud routing — coordinates on PerRoleRuntime but adds rows rather than mutating). EXTENDS B-37 directly; ADDS two RuntimeBackend variants + two adapter modules; the existing 9 backends are untouched.
Source report: workflow/improvement-sweep-2026-05-19/reports/51-edge-inference-and-mlx.md

## Context

The May 2026 edge-inference survey establishes that the cross-platform LLM-runtime story has two consensus winners: Apple MLX (Apple-only fast path; dominant on M-series; vllm-mlx already speaks Anthropic /v1/messages with continuous batching at 400+ tok/s per github.com/waybarrios/vllm-mlx) and llama.cpp/GGUF (lowest-common-denominator; one binary across x86/ARM/Metal/CUDA/HIP/MUSA/Vulkan/SYCL/CANN/OpenCL per ggml-org build.md, b9196+ Windows prebuilt). M5 Pro/Max shipped March 11 2026 with per-GPU-core Neural Accelerators and 3.5-4x faster prompt processing vs M4 (apple.com/newsroom/2026/03); measured numbers from Apple ML Research (machinelearning.apple.com/research/exploring-llms-mlx-m5) show Qwen 1.7B 1.27x / Qwen 8B 1.24x / Qwen 14B 1.19x / Qwen 30B-MoE 1.25x decode speedups M5 vs M4. Head-to-head on M2 Pro Llama 3.1 8B Q4_K_M: llama.cpp 38-48 tok/s vs MLX 45-58 tok/s (medium.com/@andreask_75652); on Mac mini M4 Pro 64GB Qwen3-Coder-30B-A3B MLX is 3x Ollama (130 vs 43 tok/s, willitrunai.com). MLX's lead is 20-87% under 14B and converges at 27B+ (bandwidth-bound). NPU paths exist (Snapdragon X2 Elite Hexagon NPU 6 = 80 TOPS per notebookcheck.net; Intel Lunar Lake = 21.3 tok/s on Llama 3.1 8B Q4_K_M per dev.to/mr1azl; AMD XDNA 2 FastFlowLM 80 tok/s @ <2W per amd.com developer) but each requires a vendor-specific compiled binary — there is NO portable NPU artifact in 2026 (Microsoft Snapdragon docs, AMD Quark, Intel OpenVINO 2025 release notes). Apple Foundation Models framework (iOS 18+/macOS 15+) gives free on-device 3B inference with @Generable guided generation but caps at 4096 ctx and Apple Intelligence-eligible devices only. transformers.js + WebGPU is feasible in Tauri webview for <3.8B models (Phi-3.5 Mini int4 fits 2GB, Chrome 4GB/tab cap per maddevs.io). ELAI's existing 11-backend B-37 abstraction is exactly the right shape for these two new variants — no rewrite needed.

## Existing state (B/C/D coverage)

- B-37 SHIPPED (commit fd5c2e5): `crates/providers/src/runtime/types.rs::RuntimeBackend` enum at elai.map line 318213; `pub trait RuntimeAdapter: Send + Sync` at line 326789; 11 existing variants (Sidecar, AnthropicApi, Ollama, Hermes, Vllm, KiroCli, OpenCode, plus 4 more — verify via Grep). Per-role mapping in `crates/orchestrator/src/per_role_runtime.rs` (line 6155: `pub fn set_fallback(&mut self, backend: RuntimeBackend)`; line 8433-8436: AgentRole → RuntimeBackend default mapping). Fallback chain orchestrated in `crates/orchestrator/src/runtime_orchestrator.rs` (line 8064: `RuntimeOrchestratorEvent`; line 8792: `is_recoverable(err)`). EXTENSION: E-51 adds `RuntimeBackend::{Mlx, LlamaCpp}` (2 variants); CREATEs `crates/providers/src/runtime/mlx.rs` + `crates/providers/src/runtime/llama_cpp.rs` adapter modules; no breaking changes — additive.
- E-2 SHIPPED (Local LLM speed tricks): Established the canonical sidecar /v1/messages contract (server.py:2338 ANTHROPIC_BASE_URL; NF4+SDPA+FA2). EXTENSION: E-51 piggybacks on that same /v1/messages surface — vllm-mlx and llama-server already emit Anthropic-shaped responses out of the box (vllm-mlx README explicitly claims "OpenAI + Anthropic /v1/messages on Apple Silicon"; llama-server emits OpenAI-compatible /v1/chat/completions and recent forks add /v1/messages compatibility).
- E-26 SHIPPED (Inference engines 2026): Established the broader engine landscape — vLLM, SGLang, llama.cpp, MLX, TensorRT-LLM. EXTENSION: E-51 narrows to the two Apple+GGUF additions that fit B-37's existing fallback chain. NPU paths flagged in E-26 are explicitly deferred here per report's "NPU = later" verdict.
- B-37 SSB action types `runtime_fallback`, `runtime_health_changed`, `cli_subprocess_spawned`, `cli_subprocess_killed` (B-PLAN-SUMMARY.md line 989): E-51 ADDs `mlx_subprocess_spawned`, `mlx_subprocess_killed`, `llamacpp_subprocess_spawned`, `llamacpp_subprocess_killed`, `mlx_apple_platform_detected`, `llamacpp_ep_detected` (Vulkan / CUDA / Metal / HIP).
- `crates/dashboard-v2/src/lib/components/RuntimePanel.svelte` (elai.map line 4301): existing RuntimePanel shows backend selector. EXTENSION: E-51 ADDs Mlx + LlamaCpp options to selector + per-platform availability badge ("Mlx requires Darwin+arm64"; "LlamaCpp detected EP: Vulkan").
- Foundation Models + transformers.js + ExecuTorch + LiteRT-LM + OpenVINO + Snapdragon QNN + Ryzen AI Quark: NOT shipped, NOT in B/C/D plan. E-51 adds spec-only stubs (no live code) for Foundation Models bridge and transformers.js dashboard tiny-agents; the rest are explicitly deferred per report's anti-pattern "Compile model once, run on all NPUs" — false in 2026.

## Reads (mandatory before coding)

- `F:\.ELAI_workflow\workflow\improvement-sweep-2026-05-19\reports\51-edge-inference-and-mlx.md` (full)
- `F:\.ELAI_workflow\workflow\improvement-sweep-2026-05-19\E-plan\E-2_local_llm_speed_tricks.md` (canonical sidecar /v1/messages surface; align on contract shape)
- `F:\.ELAI_workflow\workflow\improvement-sweep-2026-05-19\E-plan\E-26_inference_engines_2026.md` (engine landscape; E-51 narrows scope to Apple + GGUF)
- `F:\.ELAI_workflow\workflow\improvement-sweep-2026-05-19\E-plan\E-20_local_cloud_routing.md` (PerRoleRuntime override pattern; coordinate on row emission)
- `F:\.ELAI_workflow\.elai_cc\crates\providers\src\runtime\types.rs` lines 50-160 (RuntimeBackend enum) + lines 480-560 (RuntimeAdapter trait; per elai.map lines 318213, 326789)
- `F:\.ELAI_workflow\.elai_cc\crates\providers\src\runtime\sidecar.rs` lines 140-310 (existing sidecar adapter; use as template for mlx.rs and llama_cpp.rs)
- `F:\.ELAI_workflow\.elai_cc\crates\providers\src\runtime\vllm.rs` lines 180-300 (existing vLLM adapter; reference for subprocess lifecycle)
- `F:\.ELAI_workflow\.elai_cc\crates\providers\src\runtime\cli_common.rs` lines 240-410 (subprocess startup helpers; reuse `enforce_permission_opt_in`)
- `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\per_role_runtime.rs` lines 50-130 (default mapping + set_fallback; per elai.map line 6155)
- `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\runtime_orchestrator.rs` lines 330-470 (orchestrator event + is_recoverable; per elai.map lines 8064, 8792)
- `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\runtime_config.rs` lines 120-180 (chain-starts-with-sidecar assertion; per elai.map line 8598)
- `F:\.ELAI_workflow\.elai_cc\crates\dashboard-v2\src\lib\components\RuntimePanel.svelte` (existing selector UI; per elai.map line 4301)
- `F:\.ELAI_workflow\docs\thefinalglance\B-plan\B-PLAN-SUMMARY.md` lines 760-790 + 985-995 (B-37 ship summary + SSB action types)
- https://github.com/waybarrios/vllm-mlx (WebFetch — Anthropic /v1/messages + MCP tool calling on Apple Silicon)
- https://machinelearning.apple.com/research/exploring-llms-mlx-m5 (WebFetch — M5 vs M4 benchmark numbers)
- https://github.com/ggml-org/llama.cpp/blob/master/docs/build.md (WebFetch — Win prebuilt b9196 CUDA/Vulkan/HIP/SYCL)
- https://huggingface.co/mlx-community (WebFetch — 4255+ pre-converted MLX models)

## Tasks

1. RuntimeBackend enum extension + two adapter modules. MODIFY `crates/providers/src/runtime/types.rs` (~30 LOC): add `RuntimeBackend::Mlx` and `RuntimeBackend::LlamaCpp` variants to the enum at line 55-area (per elai.map line 318213); update any `match` expressions on RuntimeBackend (use `cargo check` to surface them; grep `RuntimeBackend::` for exhaustiveness). CREATE `crates/providers/src/runtime/mlx.rs` (~240 LOC) implementing `RuntimeAdapter` by spawning a vllm-mlx subprocess on Darwin+arm64 and forwarding /v1/messages over HTTP (mirror `sidecar.rs` lifecycle; reuse `cli_common.rs::enforce_permission_opt_in`). CREATE `crates/providers/src/runtime/llama_cpp.rs` (~260 LOC) implementing `RuntimeAdapter` by spawning a llama-server subprocess (auto-detect best EP via `--list-devices` parsing; emit Anthropic-shape /v1/messages — verify llama-server fork supports it, else add a thin shim in the same module). Both adapters MUST return `RuntimeAdapterError::Other` with a descriptive message on subprocess failure (no `.unwrap()` outside `#[cfg(test)]`, per CLAUDE.md HARD RULE). Both MUST emit `*_subprocess_spawned` / `*_subprocess_killed` SSB rows reusing the existing B-37 emit helpers.

2. Platform detection + spawn guards. CREATE `crates/providers/src/runtime/platform_detect.rs` (~120 LOC) with two functions: `fn detect_mlx_capable() -> MlxCapability` (returns `NotApple` on non-Darwin, `AppleSilicon { chip_family: M1|M2|M3|M4|M5, gpu_cores: u8 }` on Darwin+arm64, `AppleIntel` on Darwin+x86 with a warning); `fn detect_llamacpp_ep() -> LlamaCppEp` (returns the best execution provider: `Vulkan | Cuda | Metal | Hip | Cpu` via llama-server `--list-devices` invocation cached for the session). The Mlx adapter REFUSES to start on `NotApple` / `AppleIntel` with an explicit error returned to RuntimeOrchestrator (which then falls back per existing chain logic per `runtime_orchestrator.rs::is_recoverable` at line 8792 — verify the error variant is in the recoverable set). Emit `mlx_apple_platform_detected` and `llamacpp_ep_detected` SSB rows on first detection per session.

3. Sidecar variant detection in CLI bootstrap. MODIFY `crates/cli/src/remote_sidecar.rs` (~40 LOC delta) so that when `ELAI_RUNTIME_BACKEND=mlx` is set OR `--runtime mlx` flag is passed AND `detect_mlx_capable()` returns AppleSilicon, the CLI routes through the Mlx adapter instead of the default sidecar adapter; same for `llamacpp`. The existing `chain_starts_with_sidecar()` invariant (per elai.map line 8598) is preserved: Mlx and LlamaCpp variants insert ABOVE Sidecar in the fallback chain only if explicitly opted-in via flag/env, NOT by default. Default chain remains `[Sidecar, AnthropicApi, ...]`. Emit `runtime_chain_built` SSB row with the resolved chain on every CLI start.

4. Model artifact convention. CREATE `.elai_cc/models/_README.md` (~40 LOC, NOT a project doc — only the README convention) documenting the four supported model artifact locations:
   - `models/Qwen2.5-Coder-7B-Instruct-abliterated/` (existing HF transformers; sidecar default)
   - `models/Qwen2.5-Coder-3B-Instruct-abliterated/` (existing HF transformers; alternate)
   - `models/mlx/Qwen2.5-Coder-7B-mlx-q4/` (mlx-community quantized; for Mlx adapter)
   - `models/gguf/Qwen2.5-Coder-7B-Q4_K_M.gguf` (llama.cpp GGUF; for LlamaCpp adapter)
   The Mlx adapter looks up `models/mlx/{model_name}-mlx-{q4|q8}/`; the LlamaCpp adapter looks up `models/gguf/{model_name}-{Q4_K_M|Q5_K_M|Q8_0}.gguf`. Both adapters MUST verify the artifact exists at startup and return a descriptive `RuntimeAdapterError::Other` with a hint at the expected path on miss — no silent fallback.

5. PerRoleRuntime + RuntimePanel UI wiring. MODIFY `crates/orchestrator/src/per_role_runtime.rs` (~30 LOC delta): extend `Default` impl + serialization to accept `Mlx` and `LlamaCpp` as valid role backends (no role gets them as default — opt-in only per platform). MODIFY `crates/dashboard-v2/src/lib/components/RuntimePanel.svelte` (~60 LOC delta): add two new backend options to the selector with platform-availability badges ("Mlx: macOS arm64 only"; "LlamaCpp: CPU / Vulkan / CUDA / Metal / HIP"); on selection, call new Tauri command `runtime_capability_probe(backend)` that returns probe result for the UI to display before commit. CREATE Tauri command `runtime_capability_probe` in `crates/dashboard-v2/src-tauri/src/commands.rs` (~50 LOC) that wraps `platform_detect::detect_mlx_capable` / `detect_llamacpp_ep`.

6. Foundation Models framework spec stub. CREATE `.elai_cc/docs/runtime/foundation_models_spec.md` (~80 LOC spec-only stub) documenting: target platforms (iOS 18+, macOS 15+, Apple Intelligence-eligible devices only), ctx cap (4096), use case (structured planning JSON via @Generable; NOT a coding-model replacement per report), Anthropic-protocol bridge sketch (Swift companion app accepts /v1/messages, routes structured outputs through Foundation Models, falls back to cloud for free-form). NO live code — pure spec for future companion-app phase. Reference: developer.apple.com/documentation/FoundationModels; WWDC25 session 286.

7. transformers.js + WebGPU dashboard tiny-agents spec stub. CREATE `.elai_cc/docs/runtime/transformersjs_tiny_agents_spec.md` (~80 LOC spec-only stub) documenting: target model sizes (<3.8B int4 to fit Chrome 4GB/tab cap per maddevs.io), use cases (embeddings, classification, code-completion helpers that don't need a sidecar), in-dashboard execution via existing dashboard-v2 webview, Web Worker isolation, optional `device: 'webgpu'` flag per HF transformers.js v3 blog. NO live code — pure spec for future Phase 4N "no-sidecar mode".

8. Latency / throughput probe harness. CREATE `.elai_cc/benches/runtime_compat/probe_local.sh` (~30 LOC shell) + `.elai_cc/benches/runtime_compat/probe_local.py` (~80 LOC Python — measure cold-boot time, first-token TTFT, decode tok/s on a 256-token prompt for each available adapter). The harness invokes each adapter through the existing CloudGroundingClient using ANTHROPIC_BASE_URL pointing at the adapter's local port; outputs JSON with `{adapter, cold_boot_ms, ttft_ms, decode_tok_s, peak_ram_mb}`. Used at session-start when ELAI is configured for live inference; documented in `_README.md` from Task 4. Per CLAUDE.md "Sidecar boot is HARDWARE-HEAVY: always flag in chat before booting; never run two model instances simultaneously" — the harness MUST tear down each adapter before starting the next.

9. Contract tests for the two new adapters. CREATE `crates/contracts/tests/mlx_runtime_adapter_contract.rs` (~120 LOC): verifies Mlx adapter implements RuntimeAdapter trait, refuses to start on non-Darwin (mock platform_detect), emits expected SSB rows on startup/shutdown, falls back to next chain entry on subprocess failure. CREATE `crates/contracts/tests/llama_cpp_runtime_adapter_contract.rs` (~120 LOC): verifies LlamaCpp adapter implements RuntimeAdapter, detects an EP, emits the EP-detected SSB row, falls back on subprocess failure. CREATE `crates/contracts/tests/runtime_chain_no_default_mlx_llamacpp.rs` (~50 LOC): verifies default chain still equals `[Sidecar, AnthropicApi, ...]` — Mlx and LlamaCpp NEVER appear by default; opt-in only.

10. Documentation + CLAUDE.md note. CREATE `workflow/reasoning/E-51-decisions.md` (~80 LOC) summarising the Apple-only / GGUF-universal split, the deferred NPU paths, and the spec-only stubs for Foundation Models + transformers.js. NO modification to CLAUDE.md — this is project tooling change, not a workflow rule. The decisions doc records the Open Questions from the report (Qwen2.5-Coder-7B M3 Max tok/s unknown; vllm-mlx /v1/messages divergence unknown; cold-boot comparison vs HF+FA2 sidecar TBD; Foundation Models @Generable → research_tree DTO mapping TBD; transformers.js Phi-4-mini-3.8B on Tauri webview tok/s TBD).

## WIRING_CONTRACT

produces_symbols:
  - providers::runtime::types::RuntimeBackend::{Mlx, LlamaCpp} (enum variant extension)
  - providers::runtime::mlx::MlxAdapter (impl RuntimeAdapter)
  - providers::runtime::llama_cpp::LlamaCppAdapter (impl RuntimeAdapter)
  - providers::runtime::platform_detect::{detect_mlx_capable, detect_llamacpp_ep}
  - providers::runtime::platform_detect::{MlxCapability, LlamaCppEp}
  - SSB action_types: mlx_subprocess_spawned, mlx_subprocess_killed, llamacpp_subprocess_spawned, llamacpp_subprocess_killed, mlx_apple_platform_detected, llamacpp_ep_detected, runtime_chain_built
  - dashboard-v2 Tauri command: runtime_capability_probe(backend) -> CapabilityProbeResult
consumes_symbols:
  - crates/providers/src/runtime/types.rs::RuntimeBackend (B-37; extension surface; per elai.map line 318213)
  - crates/providers/src/runtime/types.rs::RuntimeAdapter trait (B-37; per elai.map line 326789)
  - crates/providers/src/runtime/sidecar.rs (B-37; template for adapter lifecycle; per elai.map line 1233)
  - crates/providers/src/runtime/cli_common.rs::enforce_permission_opt_in (B-37; per elai.map line 4713)
  - crates/orchestrator/src/per_role_runtime.rs (B-37; serialization extension; per elai.map line 6155)
  - crates/orchestrator/src/runtime_orchestrator.rs::is_recoverable (B-37; per elai.map line 8792)
  - crates/orchestrator/src/runtime_config.rs::chain_starts_with_sidecar (B-37; per elai.map line 8598)
  - crates/dashboard-v2/src/lib/components/RuntimePanel.svelte (B-37; UI extension; per elai.map line 4301)
reachable_from:
  - crates/cli/src/remote_sidecar.rs (CLI bootstrap routes to MlxAdapter / LlamaCppAdapter when configured)
  - crates/orchestrator/src/runtime_orchestrator.rs (orchestrator routes to adapter in fallback chain)
  - crates/dashboard-v2/src-tauri/src/commands.rs::runtime_capability_probe (Tauri command surface)
stub_ban:
  - no `.unwrap()` outside `#[cfg(test)]` in mlx.rs / llama_cpp.rs / platform_detect.rs (CLAUDE.md HARD RULE)
  - no `unimplemented!()` / `todo!()` in any new module
  - Mlx adapter MUST NOT silently fall back to CPU — if MlxCapability is `NotApple` / `AppleIntel`, return error and let RuntimeOrchestrator pick next chain entry
  - LlamaCpp adapter MUST NOT swallow `--list-devices` failures — log and return error
  - default runtime chain MUST NOT include Mlx or LlamaCpp without explicit opt-in (flag/env); contract test `runtime_chain_no_default_mlx_llamacpp` enforces
  - no two model instances simultaneously: probe harness MUST tear down adapter N before starting N+1 (per CLAUDE.md hardware-heavy rule)
  - blake3 only for any subprocess-binary-pinning hashes (no blake2/sha2)
  - Foundation Models spec stub MUST NOT include live Swift code — pure markdown spec only
  - transformers.js spec stub MUST NOT include live JS code — pure markdown spec only
  - NO NPU backend code (Snapdragon QNN, OpenVINO, Ryzen AI, ANE) — explicit DEFERRED per report
frontend_pair: crates/dashboard-v2/src/lib/components/RuntimePanel.svelte (modified) — adds Mlx and LlamaCpp selector options with platform-availability badges and capability probe
observable_effect:
  trigger: on Darwin+arm64 host, user runs `elai run --runtime mlx --model Qwen2.5-Coder-7B-mlx-q4 "hello"` against the existing sidecar URL contract
  expect_ssb: ≥1 `runtime_chain_built` row showing Mlx at chain[0], ≥1 `mlx_apple_platform_detected` row with chip_family, ≥1 `mlx_subprocess_spawned` row, ≥1 `mlx_subprocess_killed` row at exit
  expect_artifact: response body matches the existing /v1/messages Anthropic schema (same shape as CloudGroundingClient already parses); on non-Darwin host the same command fails fast with a descriptive error and the chain falls back to Sidecar with `runtime_fallback` SSB row

## DETERMINISTIC_CHECK

```bash
# 1. RuntimeBackend enum has 2 new variants
rg "RuntimeBackend::Mlx|RuntimeBackend::LlamaCpp" .elai_cc/crates/providers/src/runtime/types.rs
# Expected: ≥2 hits

# 2. Two new adapter modules exist
ls .elai_cc/crates/providers/src/runtime/mlx.rs .elai_cc/crates/providers/src/runtime/llama_cpp.rs .elai_cc/crates/providers/src/runtime/platform_detect.rs
# Expected: 3 files exist

# 3. Adapters implement RuntimeAdapter trait
rg "impl RuntimeAdapter for MlxAdapter|impl RuntimeAdapter for LlamaCppAdapter" .elai_cc/crates/providers/src/runtime/
# Expected: 2 hits

# 4. SSB action types registered
rg "mlx_subprocess_spawned|mlx_subprocess_killed|llamacpp_subprocess_spawned|llamacpp_subprocess_killed|mlx_apple_platform_detected|llamacpp_ep_detected|runtime_chain_built" .elai_cc/crates/
# Expected: ≥7 hits

# 5. Platform detection refuses non-Apple for Mlx
rg "NotApple|AppleIntel|AppleSilicon" .elai_cc/crates/providers/src/runtime/platform_detect.rs
# Expected: ≥3 hits

# 6. No .unwrap() outside #[cfg(test)] in new modules
rg "\.unwrap\(\)" .elai_cc/crates/providers/src/runtime/mlx.rs .elai_cc/crates/providers/src/runtime/llama_cpp.rs .elai_cc/crates/providers/src/runtime/platform_detect.rs | rg -v "cfg\(test\)|#\[test\]|tests/"
# Expected: zero hits

# 7. Default chain still excludes Mlx and LlamaCpp
rg "chain_starts_with_sidecar|RuntimeBackend::Sidecar" .elai_cc/crates/orchestrator/src/runtime_config.rs
# Expected: ≥2 hits
cargo test -p contracts runtime_chain_no_default_mlx_llamacpp
# Expected: PASS

# 8. RuntimePanel.svelte adds 2 new options + capability badge
rg "Mlx|LlamaCpp|runtime_capability_probe" .elai_cc/crates/dashboard-v2/src/lib/components/RuntimePanel.svelte
# Expected: ≥3 hits

# 9. Tauri command registered
rg "runtime_capability_probe" .elai_cc/crates/dashboard-v2/src-tauri/src/commands.rs
# Expected: ≥1 hit

# 10. Model artifact convention README exists
ls .elai_cc/models/_README.md
# Expected: file exists
rg "models/mlx/|models/gguf/" .elai_cc/models/_README.md
# Expected: ≥2 hits

# 11. Foundation Models + transformers.js stubs are spec-only (no live code)
ls .elai_cc/docs/runtime/foundation_models_spec.md .elai_cc/docs/runtime/transformersjs_tiny_agents_spec.md
# Expected: 2 files exist
rg "fn |class |def |impl " .elai_cc/docs/runtime/foundation_models_spec.md .elai_cc/docs/runtime/transformersjs_tiny_agents_spec.md
# Expected: zero hits (pure markdown spec — keywords inside fenced code blocks are OK only if explicitly marked as pseudo-code)

# 12. No NPU backend code (deferred per report)
rg "RuntimeBackend::Qnn|RuntimeBackend::OpenVino|RuntimeBackend::RyzenAi|RuntimeBackend::Ane" .elai_cc/crates/
# Expected: zero hits

# 13. Contract tests present
ls .elai_cc/crates/contracts/tests/mlx_runtime_adapter_contract.rs .elai_cc/crates/contracts/tests/llama_cpp_runtime_adapter_contract.rs .elai_cc/crates/contracts/tests/runtime_chain_no_default_mlx_llamacpp.rs
# Expected: 3 files exist

# 14. Workspace builds + contract tests pass
cd .elai_cc && cargo check --workspace
# Expected: 0 errors
cargo test -p contracts mlx_runtime_adapter_contract llama_cpp_runtime_adapter_contract runtime_chain_no_default_mlx_llamacpp
# Expected: 3 PASS
```

## ACCEPTANCE

- [ ] `RuntimeBackend::{Mlx, LlamaCpp}` variants added; all existing match-on-RuntimeBackend updated exhaustively
- [ ] `MlxAdapter` and `LlamaCppAdapter` implement `RuntimeAdapter` trait; spawn vllm-mlx and llama-server subprocesses respectively; speak Anthropic /v1/messages
- [ ] `platform_detect::detect_mlx_capable` refuses non-Darwin / Apple Intel; `detect_llamacpp_ep` resolves Vulkan/CUDA/Metal/HIP/Cpu
- [ ] 7 new SSB action types emitted (mlx_*, llamacpp_*, runtime_chain_built)
- [ ] Default chain still `[Sidecar, AnthropicApi, ...]` with no Mlx/LlamaCpp unless `--runtime` flag or `ELAI_RUNTIME_BACKEND` env set
- [ ] RuntimePanel.svelte adds two new selector options with capability badges
- [ ] `runtime_capability_probe` Tauri command returns MlxCapability + LlamaCppEp probe result
- [ ] `models/_README.md` documents the four artifact conventions (HF, HF-3B, mlx, gguf)
- [ ] `docs/runtime/foundation_models_spec.md` + `docs/runtime/transformersjs_tiny_agents_spec.md` exist as spec-only stubs
- [ ] Probe harness `benches/runtime_compat/probe_local.{sh,py}` exists; tears down adapter N before starting N+1
- [ ] 3 contract tests pass: mlx_runtime_adapter_contract, llama_cpp_runtime_adapter_contract, runtime_chain_no_default_mlx_llamacpp
- [ ] No NPU code (no QNN, OpenVINO, Ryzen AI, ANE backend variants) — explicit DEFERRED per report
- [ ] No `.unwrap()` outside `#[cfg(test)]` in new modules
- [ ] `cargo check --workspace` is green; existing B-37 9 backends still work unchanged

## Tests to add

- `crates/contracts/tests/mlx_runtime_adapter_contract.rs`: Mlx adapter trait impl + non-Darwin refusal + SSB emit
- `crates/contracts/tests/llama_cpp_runtime_adapter_contract.rs`: LlamaCpp adapter trait impl + EP detection + SSB emit
- `crates/contracts/tests/runtime_chain_no_default_mlx_llamacpp.rs`: default chain excludes both new variants
- `crates/providers/tests/mlx_artifact_resolution.rs`: Mlx adapter resolves `models/mlx/{name}-mlx-q4/` path; descriptive error on miss
- `crates/providers/tests/llama_cpp_artifact_resolution.rs`: LlamaCpp adapter resolves `models/gguf/{name}-Q4_K_M.gguf` path
- `crates/providers/tests/platform_detect_unit.rs`: detect_mlx_capable returns expected variant on mocked platform; detect_llamacpp_ep parses --list-devices output
- `crates/providers/tests/no_default_mlx_in_chain.rs`: regression — `--runtime sidecar` and unspecified both yield chain[0]==Sidecar
- `crates/dashboard-v2/tests/runtime_panel_options.test.ts` (if Svelte test harness exists; otherwise add as e2e): RuntimePanel shows Mlx and LlamaCpp options with availability badges

## Files touched

- CREATE: `crates/providers/src/runtime/mlx.rs` (~240 LOC)
- CREATE: `crates/providers/src/runtime/llama_cpp.rs` (~260 LOC)
- CREATE: `crates/providers/src/runtime/platform_detect.rs` (~120 LOC)
- CREATE: `crates/contracts/tests/mlx_runtime_adapter_contract.rs` (~120 LOC)
- CREATE: `crates/contracts/tests/llama_cpp_runtime_adapter_contract.rs` (~120 LOC)
- CREATE: `crates/contracts/tests/runtime_chain_no_default_mlx_llamacpp.rs` (~50 LOC)
- CREATE: `crates/providers/tests/mlx_artifact_resolution.rs` (~40 LOC)
- CREATE: `crates/providers/tests/llama_cpp_artifact_resolution.rs` (~40 LOC)
- CREATE: `crates/providers/tests/platform_detect_unit.rs` (~60 LOC)
- CREATE: `crates/providers/tests/no_default_mlx_in_chain.rs` (~40 LOC)
- CREATE: `.elai_cc/models/_README.md` (~40 LOC)
- CREATE: `.elai_cc/docs/runtime/foundation_models_spec.md` (~80 LOC)
- CREATE: `.elai_cc/docs/runtime/transformersjs_tiny_agents_spec.md` (~80 LOC)
- CREATE: `.elai_cc/benches/runtime_compat/probe_local.sh` (~30 LOC)
- CREATE: `.elai_cc/benches/runtime_compat/probe_local.py` (~80 LOC)
- CREATE: `workflow/reasoning/E-51-decisions.md` (~80 LOC)
- MODIFY: `crates/providers/src/runtime/types.rs` (~30 LOC delta — enum + match-exhaustiveness)
- MODIFY: `crates/providers/src/runtime/mod.rs` (~10 LOC delta — register new modules)
- MODIFY: `crates/orchestrator/src/per_role_runtime.rs` (~30 LOC delta — serde + role-mapping back-compat)
- MODIFY: `crates/cli/src/remote_sidecar.rs` (~40 LOC delta — runtime flag/env routing)
- MODIFY: `crates/dashboard-v2/src/lib/components/RuntimePanel.svelte` (~60 LOC delta — 2 new options + badges)
- MODIFY: `crates/dashboard-v2/src-tauri/src/commands.rs` (~50 LOC delta — runtime_capability_probe Tauri command)

## Citations / Grounding

- https://machinelearning.apple.com/research/exploring-llms-mlx-m5 — Apple ML Research M5: Qwen 1.7B 1.27x / 8B 1.24x / 14B 1.19x / 30B-MoE 1.25x decode speedup M5 vs M4; memory-bandwidth-bound; grounds Task 1 + Task 2
- https://www.apple.com/newsroom/2026/03/apple-introduces-macbook-pro-with-all-new-m5-pro-and-m5-max/ — M5 Pro/Max launch Mar 11 2026; per-GPU-core Neural Accelerators; grounds platform-detect chip_family enum
- https://medium.com/@andreask_75652/benchmarking-apples-mlx-vs-llama-cpp-bbbebdc18416 — MLX 45-58 tok/s vs llama.cpp 38-48 tok/s on M2 Pro Llama 3.1 8B Q4_K_M; grounds priority of Mlx-over-LlamaCpp on Apple
- https://llmcheck.net/benchmarks — 58 models / 158 datapoints; M5 Max Gemma 4 E2B ~158 tok/s MLX (note: ELAI rejected Gemma 4 per memory:feedback_no_gemma4 — benchmark cited for tok/s ceiling only, not model choice)
- https://willitrunai.com/blog/mlx-vs-ollama-apple-silicon-benchmarks — Mac mini M4 Pro 64GB Qwen3-Coder-30B-A3B: MLX 130 vs Ollama 43 tok/s (3x); grounds Task 1 priority
- https://github.com/waybarrios/vllm-mlx — vllm-mlx README claims Anthropic /v1/messages, continuous batching, 400+ tok/s, Claude-Code-compatible; grounds Mlx adapter wrapping strategy in Task 1
- https://github.com/ggml-org/llama.cpp/blob/master/docs/build.md — Win prebuilt b9196 with CUDA13.1+Vulkan+HIP+SYCL in one binary; grounds LlamaCpp adapter universal-EP strategy
- https://huggingface.co/mlx-community — 4255+ pre-converted MLX models; grounds Task 4 artifact convention `models/mlx/`
- https://www.notebookcheck.net/Hexagon-NPU-6-in-the-Snapdragon-X2-Elite-Extreme-... — X2 Elite Hexagon 6 80 TOPS 95% faster than M4 ANE; grounds NPU-deferred rationale (vendor-specific binary required)
- https://onnxruntime.ai/docs/genai/howto/build-models-for-snapdragon.html — QNN context binaries must be precompiled; grounds NPU-deferred
- https://docs.openvino.ai/2025/about-openvino/performance-benchmarks/generative-ai-performance.html — Intel Lunar Lake NF4 on NPU up to 8B; grounds NPU-deferred
- https://dev.to/mr1azl/i-tried-running-llms-on-intels-npu-heres-what-actually-happened-5h17 — Llama 3.1 8B Q4_K_M 21.3 tok/s on Lunar Lake NPU; grounds NPU-deferred (real numbers exist but tooling vendor-locked)
- https://developer.apple.com/documentation/FoundationModels — ~3B on-device, 4096 ctx, Apple Intelligence devices; grounds Task 6 spec stub
- https://huggingface.co/blog/transformersjs-v3 — device:'webgpu' flag, Phi-3.5 Mini 3.8B fits 2GB int4; grounds Task 7 spec stub
- https://arxiv.org/pdf/2511.05502 — MLX/MLC-LLM/Ollama/llama.cpp academic comparison Q4 2025; grounds the two-runtime decision in E-51 vs single-runtime
- https://arxiv.org/html/2603.06728v1 — Orion ANE programming paper; 20 ANE constraints, 14 previously undocumented; grounds ANE-NOT-used-for-decode rationale
- CLAUDE.md anti-stale-training — blake3 only; no .unwrap() in lib code; grounds stub_ban
- CLAUDE.md "Sidecar boot is HARDWARE-HEAVY: never run two model instances simultaneously" — grounds Task 8 probe harness tear-down rule

## Open questions / risks

- Open Q1 (from report): Qwen2.5-Coder-7B-Instruct-abliterated tok/s on M3 Max 64GB via vllm-mlx at Q4 and Q8 — no measured number exists. Mitigation: Task 8 probe harness measures it on first Apple host; logged to E-51-decisions.md.
- Open Q2 (from report): does vllm-mlx /v1/messages match Anthropic spec closely enough that the existing CloudGroundingClient parser works unchanged? Risk: tool_use shape or stream event divergence forces an adapter-side translation layer. Mitigation: Mlx adapter has a thin response-shim slot (deferred; ship without and patch if integration test fails).
- Open Q3 (from report): cold-boot time of vllm-mlx vs Python HF+FA2 sidecar — Apple may win on spin-up which hurts our remote-tunnel UX currently. Task 8 measures it.
- Open Q4 (from report): Foundation Models @Generable Swift struct → research_tree DTO mapping feasibility — could replace running a 7B for simple structured outputs. Out of scope for E-51 (spec stub only); follow-up in companion-iOS phase.
- Open Q5 (from report): transformers.js Phi-4-mini-3.8B tok/s on Tauri webview vs RTX 4060 — if >15 tok/s, dashboard could drop sidecar for utility agents. Out of scope for E-51 (spec stub only).
- Risk 1: llama-server's `--list-devices` output format may not be stable across versions — pin the llama.cpp version we expect via the `_README.md` and emit `llamacpp_ep_detected_parse_failed` row on parse error so we can fix downstream.
- Risk 2: vllm-mlx is a community fork (github.com/waybarrios/vllm-mlx) — slopsquatting / supply-chain risk per CLAUDE.md anti-slopsquatting rule. Mitigation: pin commit SHA in the `_README.md`, verify Anthropic /v1/messages claim via direct WebFetch + curl probe before merge.
- Risk 3: ANE is intentionally avoided per report and Orion paper (CoreML 2-4x decode overhead, 32MB SRAM cliff). If a future contributor tries to add an `Ane` backend variant, the contract test `runtime_chain_no_default_mlx_llamacpp` should be extended to fail on `RuntimeBackend::Ane` until a separate research E-N opens the door.
- Risk 4: Sidecar / runtime hardware contention — running probe harness Task 8 alongside the active sidecar will OOM the 12GB RTX 3060. CLAUDE.md HARD RULE: probe MUST tear down each adapter before starting the next; documented in `_README.md` and enforced by the harness shell wrapper.

## Next

E-51 turns the M-series Mac into a viable consumer tier for ELAI (Qwen 1.7B at 158 tok/s on M5 Max per llmcheck.net; 3x Ollama on M4 Pro per willitrunai) and the GGUF-everywhere artifact into the lowest-common-denominator fallback. After E-51, B-37's RuntimeOrchestrator picks Mlx when running on Apple Silicon with the right model artifact present, LlamaCpp as the cross-platform option for hosts without a sidecar, and existing Sidecar / AnthropicApi remain the defaults. Open follow-ups: (i) measure the report's open questions via Task 8 probe harness on first Apple host access; (ii) future patch E-51.1 may add the vllm-mlx response-shim if /v1/messages divergence is detected; (iii) Foundation Models companion-iOS phase consumes the spec stub from Task 6; (iv) Phase 4N "no-sidecar mode" consumes the transformers.js spec stub from Task 7; (v) NPU paths (Snapdragon QNN, Intel OpenVINO, AMD Ryzen AI) remain explicitly deferred until a paying user with that hardware demands it per report. Downstream: E-20 (local/cloud routing) coordinates on the PerRoleRuntime extension; E-2 (local LLM speed tricks) shares the /v1/messages contract surface.

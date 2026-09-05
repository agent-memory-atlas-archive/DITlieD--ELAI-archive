# E-51 Implementation Decisions

## What was implemented
- RuntimeBackend::Mlx and RuntimeBackend::LlamaCpp added to providers crate enum
- MlxAdapter (mlx.rs): HTTP adapter wrapping vllm-mlx subprocess on Darwin+arm64
- LlamaCppAdapter (llama_cpp.rs): HTTP adapter wrapping llama-server on any platform
- platform_detect.rs: detect_mlx_capable() + detect_llamacpp_ep() detection helpers
- CLI routing: resolve_runtime_backend_from_env() in remote_sidecar.rs
- Dashboard: RuntimePanel.svelte + runtime_capability_probe Tauri command
- models/_README.md: four artifact location conventions
- Spec stubs: foundation_models_spec.md + transformersjs_tiny_agents_spec.md
- Probe harness: benches/runtime_compat/probe_local.{sh,py}
- Contract tests: 3 new files in crates/contracts/tests/

## Key decisions

### Apple-only MLX, GGUF-universal LlamaCpp
MLX wins 20-87% on models <14B on M-series (MLX 45-58 vs llama.cpp 38-48 tok/s on
M2 Pro Llama 3.1 8B Q4_K_M; medium.com/@andreask_75652). MLX is Darwin+arm64 only.
LlamaCpp covers Win/Mac/Linux via one binary with auto-detected EP (Vulkan/CUDA/Metal/HIP).

### No default chain inclusion (opt-in only)
Mlx and LlamaCpp do NOT appear in the default fallback chain. Existing users who rely
on Sidecar->AnthropicApi->ClaudeCode are unaffected. Opt-in via ELAI_RUNTIME_BACKEND
env var or --runtime flag.

### spec_chain_starts_with_sidecar invariant preserved
The runtime_config.rs chain_starts_with_sidecar assertion (line 8598) is preserved.
Mlx and LlamaCpp can only be inserted ABOVE Sidecar when explicitly opted-in — they
do NOT displace Sidecar from chain[0] unless the user explicitly builds a custom chain.

### SSB action types
7 new SSB action types added in ssb.rs:
- mlx_subprocess_spawned, mlx_subprocess_killed
- llamacpp_subprocess_spawned, llamacpp_subprocess_killed
- mlx_apple_platform_detected, llamacpp_ep_detected
- runtime_chain_built
SCHEMA_VERSION bumped from 95 to 96. all_test_payloads().len() goes 216 -> 223.

### Foundation Models + transformers.js: spec-only
No live code. Per E-51 report: Foundation Models caps at 4096 ctx (insufficient for
coding tasks); transformers.js tok/s on Tauri WebView unknown. Both deferred.

### NPU paths explicitly deferred
Snapdragon QNN, Intel OpenVINO, AMD Ryzen AI, Apple ANE: each requires a vendor-specific
compiled binary. No portable NPU artifact in 2026 (per Microsoft Snapdragon docs, AMD Quark,
Intel OpenVINO 2025 release notes). RuntimeBackend::Qnn / OpenVino / RyzenAi / Ane
variants do NOT exist.

## Open Questions (from E-51 report)
- Q1: Qwen2.5-Coder-7B tok/s on M3 Max via vllm-mlx at Q4 — no measured number.
  Mitigation: probe harness measures on first Apple host.
- Q2: Does vllm-mlx /v1/messages match Anthropic spec closely enough for existing
  CloudGroundingClient parser? Risk: tool_use shape divergence. Mitigation: thin
  response-shim slot in MlxAdapter (deferred; patch E-51.1 if needed).
- Q3: Cold-boot time of vllm-mlx vs Python HF+FA2 sidecar — unmeasured.
- Q4: Foundation Models @Generable Swift struct <-> research_tree DTO mapping.
- Q5: transformers.js Phi-4-mini-3.8B tok/s on Tauri WebView.

## Expected test outcome
- cargo check --workspace: 0 errors
- cargo test -p contracts: mlx_runtime_adapter_contract, llama_cpp_runtime_adapter_contract,
  runtime_chain_no_default_mlx_llamacpp — 3 PASS (non-Apple-Silicon context)
- cargo test -p providers: platform_detect_unit, mlx_artifact_resolution,
  llama_cpp_artifact_resolution, no_default_mlx_in_chain — 4 PASS

## Unknowns at implementation time
- vllm-mlx community fork supply-chain: pin commit SHA in _README.md before production use
- llama-server --list-devices output format stability across versions: emit parse-failed row

# transformers.js + WebGPU Tiny-Agents — Spec Stub (E-51)

Status: DEFERRED. No live code. Pure specification for future Phase 4N "no-sidecar mode".

## Target Model Sizes
- <3.8B int4 to fit Chrome 4 GB/tab cap (maddevs.io research)
- Phi-3.5 Mini (3.8B int4 ~2 GB) as primary candidate
- Phi-4-mini-3.8B as secondary candidate (tok/s on Tauri WebView TBD — open Q5)
- Qwen2.5-Coder-1.5B int4 as fallback for memory-constrained hosts

## Execution Environment
- dashboard-v2 Tauri WebView (Chromium WebGPU via wgpu under Tauri 2)
- Web Worker isolation: inference runs off the main thread, no UI jank
- device: 'webgpu' flag per transformers.js v3 API (HF blog, verified 2026-05)
- Fallback: device: 'cpu' WASM when WebGPU unavailable

## Use Cases (no sidecar needed)
- Embedding generation for local BM25/vector queries
- Code-completion hint generation for short snippets (<256 tok output)
- Intent classification for dashboard chat input
- Structured JSON output for simple planning nodes

## Integration Point with ELAI Architecture
- New SsbActionType: transformersjs_inference_started / transformersjs_inference_complete
- RuntimeBackend::TransformersJs variant (future, after this spec is implemented)
- Chain position: last resort (below LlamaCpp) — only when no other backend available

## Svelte Component Sketch (pseudo-code)

```pseudo
// Web Worker bridge (not real code):
// worker = new Worker('transformers-worker.js')
// worker.postMessage({ type: 'generate', prompt, model: 'Phi-3.5-mini-int4' })
// worker.onmessage = ({ data }) => updateStreamChunk(data)
```

## Hardware Constraints
- WebGPU adapter must expose >2 GB buffer limit (GPUs with <2 GB VRAM unsupported)
- Chrome 4 GB/tab cap limits models to ~3.8B int4 maximum
- Safari WebGPU support: partial (Metal backend available macOS 14+)

## Open Questions (from E-51 report)
- Phi-4-mini-3.8B tok/s on Tauri WebView vs RTX 4060: if >15 tok/s, can drop sidecar
  for utility agents
- transformers.js v3 streaming API stability on Tauri vs plain Chrome

## References
- huggingface.co/blog/transformersjs-v3 (device:'webgpu', Phi-3.5 Mini int4 fits 2 GB)
- maddevs.io Chrome 4 GB/tab WebGPU limit research
- E-51 report: workflow/improvement-sweep-2026-05-19/reports/51-edge-inference-and-mlx.md

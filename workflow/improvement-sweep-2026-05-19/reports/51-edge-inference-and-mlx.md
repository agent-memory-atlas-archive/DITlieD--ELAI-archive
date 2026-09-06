# Edge inference + Apple MLX + NPU for ELAI (May 2026)

## TL;DR
- Apple MLX is now production-grade on Apple Silicon and is the dominant local-LLM runtime on Mac/iOS in 2026; M5 Pro/Max shipped March 11 2026 with per-GPU-core Neural Accelerators and 3.5-4x faster prompt processing vs M4. ELAI should add an MLX runtime backend (B-37 RuntimeAdapter) but should NOT rewrite its sidecar architecture — wrap mlx-lm or vllm-mlx as a sidecar variant that already speaks Anthropic /v1/messages.
- The single cross-platform LLM runtime story has TWO winners in 2026: llama.cpp/GGUF (portable, every backend, slightly slower on Apple) and LiteRT-LM (Google, Android+Linux+macOS+Windows+Pi). MLX is Apple-only. ONNX Runtime GenAI is still 0.x preview. ELAI's pragmatic path: keep llama.cpp as the lowest-common-denominator backend, add MLX as the Apple-fast-path, leave Win+WSL2 sidecar primary.
- NPU LLM inference (Snapdragon X Elite Hexagon, Lunar Lake/Arrow Lake, Ryzen AI XDNA 2, Apple ANE) is real for 1-8B models with 21-80 tok/s but each requires a vendor-specific compiled binary (QNN context binaries, OpenVINO IR, Ryzen AI Quark, CoreML/Orion). There is NO unified NPU API in 2026. NPU support is a "later" item for ELAI; build the abstraction now, defer implementation until a paying user demands it.
- Foundation Models framework (iOS 18+ / macOS 15+) gives free on-device 3B inference with guided generation. For ELAI's iOS-companion future, this is a zero-cost path — but only 4096 ctx and only on Apple Intelligence-eligible devices. Android equivalent is AICore + Gemini Nano via ML Kit GenAI APIs (Pixel/Snapdragon/MediaTek only).
- Tauri-webview + transformers.js + WebGPU is genuinely feasible for ELAI's smaller agents (under 3.8B, quantized, <2 GB VRAM). Worth considering for Phase 4N "no-sidecar mode" — running tiny utility agents in the dashboard webview without booting the Python sidecar at all.

## Key sources
URL | what | takeaway
- https://machinelearning.apple.com/research/exploring-llms-mlx-m5 | Apple ML Research, M5 MLX | Qwen 1.7B 1.27x / Qwen 8B 1.24x / Qwen 14B 1.19x / Qwen 30B-MoE 1.25x decode speedup M5 vs M4; memory-bandwidth-bound still
- https://www.apple.com/newsroom/2026/03/apple-introduces-macbook-pro-with-all-new-m5-pro-and-m5-max/ | M5 Pro/Max launch | March 3 2026 announce, March 11 ship, 16-core NE, per-GPU-core Neural Accelerators, 4x peak AI compute vs M4 Pro/Max
- https://medium.com/@andreask_75652/benchmarking-apples-mlx-vs-llama-cpp-bbbebdc18416 | MLX vs llama.cpp head-to-head | M2 Pro Llama 3.1 8B Q4_K_M: llama.cpp 38-48 tok/s, MLX 45-58 tok/s. MLX +20-87% under 14B; converges at 27B+ (bandwidth-bound)
- https://llmcheck.net/benchmarks | Apple Silicon leaderboard | 58 models / 158 datapoints, Q4_K_M; Gemma 4 E2B ~158 tok/s on M5 Max MLX; Phi-4-Mini ~135 tok/s; Qwen 3.6-35B-A3B ~55 tok/s M5 Max
- https://willitrunai.com/blog/mlx-vs-ollama-apple-silicon-benchmarks | MLX vs Ollama 2026 | Mac mini M4 Pro 64GB Qwen3-Coder-30B-A3B: MLX 130 tok/s vs Ollama (llama.cpp) 43 tok/s — 3x. M4 Max 128GB same model: 130 vs 43.5 tok/s
- https://github.com/waybarrios/vllm-mlx | vllm-mlx | OpenAI + Anthropic /v1/messages on Apple Silicon, MLX backend, continuous batching, MCP tool calling, 400+ tok/s, Claude-Code-compatible — drop-in for ELAI sidecar
- https://www.notebookcheck.net/Hexagon-NPU-6-in-the-Snapdragon-X2-Elite-Extreme-80-TOPS-performance-that-is-up-to-95-faster-than-Apple-M4-and-122-faster-than-Intel-Lunar-Lake.1166576.0.html | Snapdragon X2 Elite Extreme | Hexagon NPU 6 = 80 TOPS, ~95% faster than M4 ANE, 122% faster than Lunar Lake; LLM via QNN + Qualcomm Genie runtime
- https://onnxruntime.ai/docs/genai/howto/build-models-for-snapdragon.html | ONNX Runtime GenAI for Snapdragon | Build QNN context binaries; runtime is QNN; "this runtime is closed-source"
- https://docs.openvino.ai/2025/about-openvino/performance-benchmarks/generative-ai-performance.html | OpenVINO 2025 benchmarks | Lunar Lake + Arrow Lake-H supported; NF4 on NPU up to 8B; SDPA enhanced; INT4 symmetric quantization fastest
- https://dev.to/mr1azl/i-tried-running-llms-on-intels-npu-heres-what-actually-happened-5h17 | Intel NPU real-world test | Llama 3.1 8B Q4_K_M: 21.3 tok/s on Lunar Lake NPU vs 4.1 tok/s CPU-only
- https://www.amd.com/en/developer/resources/technical-articles/2025/ai-inference-acceleration-on-ryzen-ai-with-quark.html | AMD Ryzen AI Quark | XDNA 2 = 50 TOPS INT8, 25 TFLOPS BF16; OGA hybrid splits prefill→NPU, decode→GPU; FastFlowLM 80 tok/s @ <2W
- https://developer.apple.com/documentation/FoundationModels | Apple Foundation Models framework | ~3B on-device model, macOS/iOS/iPadOS/visionOS, 4096 ctx, Swift @Generable guided generation, free, no API key
- https://developer.android.com/ai/gemini-nano | Android Gemini Nano | AICore system service, runs on Pixel/optimized Snapdragon/MediaTek/Tensor; ML Kit GenAI APIs (summarize/proofread/rewrite); LoRA adapters
- https://huggingface.co/blog/transformersjs-v3 | Transformers.js v3 WebGPU | device: 'webgpu' flag enables GPU; Phi-3.5 Mini 3.8B fits 2GB int4; Llama 3.2-1B browser-native chat works
- https://pytorch.org/blog/introducing-executorch-1-0/ | ExecuTorch 1.0 (Oct 2025) | 50KB base, 12+ backends (Apple/Qualcomm/ARM/MediaTek/Vulkan), C++/Java/ObjC/Swift LLM APIs, LoRA + 4-bit HQQ
- https://github.com/microsoft/onnxruntime-genai | ONNX Runtime GenAI | Still 0.x preview as of 2026; powers Foundry Local, Windows ML, VS Code AI Toolkit; cross-platform but immature
- https://v-chandra.github.io/on-device-llms/ | On-Device LLMs State of the Union 2026 | Survey of frameworks, deployment matrix
- https://arxiv.org/pdf/2511.05502 | arXiv 2511.05502 — MLX/MLC-LLM/Ollama/llama.cpp comparative study | Academic comparison, Q4 2025
- https://arxiv.org/html/2603.06728v1 | Orion: ANE programming | Bypasses CoreML via private APIs; catalogs 20 ANE constraints (14 previously undocumented); ANE is 32MB SRAM cliff, 2048x2048 matmul 5.7 TFLOPS, drops 30% at 4096x4096
- https://arxiv.org/html/2511.06719v1 | MobileLLM-Pro tech report | 1B with 128k ctx; +7.9% vs Llama 3.2-1B; 1.8x faster prefill; KV cache 117MB→40MB at 8k ctx
- https://ai.meta.com/blog/meta-llama-quantized-lightweight-models/ | Meta quantized Llama 3.2 | OnePlus 12: decode +2.5x, prefill +4.2x, size -56%, memory -41%, via ExecuTorch

## Platform landscape
| Platform | Best runtime (2026) | Qwen2.5-Coder-7B feasible? | Numbers cited |
| --- | --- | --- | --- |
| Windows + RTX 3060 12GB (ELAI dev) | llama.cpp CUDA / vLLM / HF+FA2 | Yes (7B Q4) | 23-29 tok/s general 7B (singhajit.com) |
| Windows + 2x RTX 3090 (ELAI prod) | HF transformers + FA2, vLLM | Yes (7B FP16 + headroom) | RTX 3090 single: 207 tok/s on Qwen3.5-27B (HN benchmark) |
| Mac M2 Pro | MLX / llama.cpp Metal | Yes (Q4) | Llama 3.1 8B Q4_K_M: MLX 45-58 tok/s, llama.cpp 38-48 tok/s |
| Mac M3 Max | MLX | Yes (4-bit and 8-bit) | M1 Max Qwen 2.5 7B MLX 63.7 tok/s (proxy); M3 Max bandwidth 400 GB/s, MLX saturates it |
| Mac M4 Max 36GB+ | MLX | Yes (8-bit fits) | Qwen 3.5 35B-A3B 40-55 tok/s @ 32K; Qwen 3.5 9B 4-bit 45-60 tok/s |
| Mac M5 Pro/Max (Mar 2026) | MLX (M5 GPU Neural Accelerators) | Yes (large headroom) | M5 vs M4: Qwen 8B 1.24x decode, 3.5-4x TTFT/prompt processing |
| iPhone 16 Pro / iPad M-series | Foundation Models / MLX-Swift / ExecuTorch | Borderline (7B Q4 needs ~4GB) | Gemma 3 4B ~27 tok/s iPhone 16 Pro; Llama 3.2-1B quantized ExecuTorch -41% memory |
| Snapdragon X Elite/X2 Elite (Win-on-ARM) | QNN + Qualcomm Genie | Partial — need precompiled QNN context binary | X2 Elite Extreme Hexagon 6: 80 TOPS, FastVLM TTFT 0.12s |
| Intel Lunar Lake / Arrow Lake-H NPU | OpenVINO GenAI | Up to 8B NF4 on NPU | Llama 3.1 8B Q4_K_M: 21.3 tok/s NPU vs 4.1 CPU |
| AMD Ryzen AI 300 (XDNA 2) | Ryzen AI SW + Quark, FastFlowLM, OGA hybrid | Yes (hybrid NPU+iGPU) | FastFlowLM 80 tok/s @ <2W system; 50 TOPS INT8 / 25 TFLOPS BF16 |
| Android (Pixel/Tensor, Snapdragon, MediaTek) | AICore + Gemini Nano via ML Kit GenAI | Only Gemini Nano (~3.25B) | Sub-50ms TTFT (Cactus); restricted device list |
| Browser (Chrome/Edge/Tauri webview) | transformers.js + ONNX Runtime Web + WebGPU | Borderline — 7B needs 4GB+ VRAM, Chrome 4GB/tab cap | Phi-3.5 Mini 3.8B int4 fits 2GB; Llama-3.2-1B works |

## Apple MLX deep dive
What MLX does well in 2026 (per Apple ML Research + arXiv 2511.05502 + willitrunai + Andreas Kunar bench):
- Native unified-memory model — no host-device copies; saturates 400-1000 GB/s bandwidth on M3-Max/M4-Max/M5-Max where llama.cpp Metal can't.
- 20-87% throughput lead over llama.cpp Metal for <14B models (compute-bound regime). Converges at 27B+ (bandwidth-bound).
- 3x lead over Ollama (llama.cpp backend) on MoE: Qwen3-Coder-30B-A3B 130 vs 43 tok/s on M4 Pro/Max.
- M5 GPU has per-core Neural Accelerators (macOS 26.2+): 3.5-4x faster prompt processing vs M4.
- mlx-community on HF has 4255+ pre-converted models (Llama/Qwen/Mistral/Gemma all covered).
- MLX-Swift gives native iOS/iPad/visionOS deployment (LLMEval, MLXChatExample run on iPhone).
- mlx-lm ships an OpenAI-compatible /v1/chat/completions server out of the box; vllm-mlx adds Anthropic /v1/messages + continuous batching + 400+ tok/s.

What's missing (per Apple ML Research + Orion paper + community):
- Memory-bandwidth-bound on decode — still no compute-side magic for token-by-token generation.
- ANE (Neural Engine) is largely unused by MLX-LM; Orion paper documents 20 ANE constraints, 14 previously undocumented. MLX runs on the GPU + new M5 Neural Accelerators, NOT the legacy ANE.
- No fine-tuning perf parity with NVIDIA + FA2; LoRA works but slower than CUDA equivalents.
- Windows/Linux: NOT supported; Apple-only.

ELAI portability cost (concrete):
- Add an MLX RuntimeAdapter in B-37: wrap vllm-mlx (preferred — Anthropic /v1/messages native) or mlx-lm server. Effort: ~1 sub-phase, similar size to existing CloudGroundingClient OAuth work.
- Sidecar variant: detect Darwin + arm64, route to vllm-mlx instead of HF+FA2 Python server. Same /v1/messages contract = zero CLI/protocol changes.
- Model artifact: separate mlx-community quantized model alongside existing Qwen2.5-Coder-7B-Instruct-abliterated GGUF/HF. Disk cost ~5 GB per model.
- The 11-backend B-37 abstraction is exactly the right shape for this.

## NPU landscape
- Snapdragon X Elite / X2 Elite (Hexagon NPU 6, 80 TOPS): runtime is closed-source QNN + Qualcomm Genie. Models must be precompiled to QNN context binaries (per Microsoft onnxruntime.ai docs). FastVLM TTFT 0.12s on 8 Elite Gen 5 mobile. OmniNeural-4B is reference multimodal. NOT a portable artifact — each model needs vendor-supplied or vendor-tooled compilation.
- Intel Lunar Lake (Core Ultra 7 258V) + Arrow Lake-H (Core Ultra 9 285H): OpenVINO 2025 with NF4 on NPU up to 8B. Measured Llama 3.1 8B Q4_K_M = 21.3 tok/s on NPU (vs 4.1 CPU). SDPA op enhanced for long prompts. Newer model support: Phi-4, Mistral-7B-v0.3, Qwen3, Qwen2.5-VL-3B.
- AMD Ryzen AI 300 (XDNA 2, 50 TOPS INT8 / 25 TFLOPS BF16): Ryzen AI SW + Quark quantizer (SmoothQuant/GPTQ/Quarot). OGA hybrid: NPU for prefill, iGPU for decode. FastFlowLM: 80 tok/s at <2W system draw — power efficiency leader.
- Apple Neural Engine (M-series, 38 TOPS INT8): 32MB SRAM cliff, drops 30% at 4096x4096 matmul. CoreML adds 2-4x overhead per small op — kills LLM decode. Orion paper (arXiv 2603.06728) bypasses CoreML via private APIs but is research-stage. ANE inference typically caps at 512-2048 tokens. MLX/MLX-LM intentionally targets GPU + new M5 Neural Accelerators instead.

Bottom line: NPU is fastest-per-watt for sub-3B models with vendor-specific tooling. There is NO portable NPU binary format in 2026. Adding NPU backends = N vendor SDKs × N model recompilations.

## Cross-platform runtime decision
Two-runtime strategy (recommended): keep Python sidecar (HF+FA2) for ELAI's primary RTX path AND add llama.cpp/GGUF as the universal fallback. MLX as the Apple-fast-path. NPU paths deferred.

Why not single-runtime:
- MLC-LLM compiles once, runs everywhere — but model conversion is per-target, and adoption is small.
- LiteRT-LM is Google's bet (Android+Linux+macOS+Windows+Pi); works but currently focused on Gemma family.
- ONNX Runtime GenAI is the Microsoft bet — still 0.x preview as of May 2026.
- llama.cpp is the de facto LCD: x86/ARM/Metal/CUDA/HIP/MUSA/Vulkan/SYCL/CANN/OpenCL all in one binary, GGUF model format is portable.

For ELAI's existing 11-backend B-37 abstraction, the right additions in priority order:
1. MLX backend (Apple Mac dev + future iPad/iPhone companion) — high ROI, low effort
2. llama.cpp GGUF backend with Vulkan + Metal + CUDA detection — covers Win/Mac/Linux/ChromeOS in one artifact
3. Foundation Models framework (iOS/macOS native Swift) — for iOS companion only, free, zero VRAM cost
4. transformers.js + WebGPU in Tauri webview — for tiny utility agents that don't need the sidecar at all
5. ONNX Runtime GenAI / OpenVINO / QNN / Ryzen AI — deferred until paying user with that hardware

## Best additions for ELAI
Ranked:
1. MLX RuntimeAdapter via vllm-mlx | grounding: github.com/waybarrios/vllm-mlx, willitrunai bench, Apple ML Research M5 | hook: new backend variant alongside HF in B-37 RuntimeAdapter; sidecar-variant detection: Darwin+arm64 → spawn vllm-mlx process speaking /v1/messages
2. llama.cpp GGUF universal fallback | grounding: ggml-org/llama.cpp build.md, b9196 Windows prebuilt w/ CUDA13.1+Vulkan+HIP+SYCL | hook: third sidecar backend "gguf-portable" — same /v1/messages via llama-server, auto-detect best EP
3. Foundation Models framework Swift bridge | grounding: developer.apple.com/documentation/FoundationModels, WWDC25 session 286 | hook: companion iOS app spec — talks Anthropic protocol to ELAI cloud + falls back to local Foundation Models for offline drafts; @Generable for structured planning JSON
4. transformers.js + WebGPU in dashboard webview | grounding: HF transformers.js v3 blog, MS TechCommunity Phi-3.5 + ORT-Web RAG, intel guide | hook: optional "tiny-agents" mode in dashboard-v2 that runs sub-1B helpers (embeddings, classification, code-completion) in-browser without booting the sidecar; lives in dashboard, not in B-37 RuntimeAdapter
5. ExecuTorch backend for Android/iOS | grounding: pytorch.org/blog/introducing-executorch-1-0, Meta quantized Llama 3.2 OnePlus 12 bench | hook: Phase 4N mobile companion — single .pt2 export targets iOS Metal/CoreML AND Android Qualcomm/MediaTek/Vulkan
6. LiteRT-LM as ChromeOS / Linux ARM fallback | grounding: developers.googleblog.com LiteRT-LM | hook: deferred — only if ELAI gets ChromeOS user
7. OpenVINO GenAI for Intel Lunar/Arrow Lake | grounding: docs.openvino.ai 2025, dev.to/mr1azl 21.3 tok/s | hook: deferred — Intel NPU dev box test, but Intel ARC GPU path already covered by Vulkan in llama.cpp

## Anti-patterns
- "ANE will save us" — CoreML overhead on LLM decode is 2-4x per op (Orion paper, arXiv 2603.06728). MLX intentionally avoids ANE for autoregressive generation. Do NOT plan around ANE for token decode.
- "ONNX Runtime GenAI is the unified path" — still 0.x preview May 2026 (github.com/microsoft/onnxruntime-genai README). Microsoft's own Foundry Local uses it but warns "subject to change". Premature to standardize on.
- "Compile model once, run on all NPUs" — false in 2026. QNN context binaries (Snapdragon), OpenVINO IR (Intel), Ryzen AI / Quark (AMD), CoreML/Orion (Apple) are all incompatible. (Microsoft Snapdragon docs, AMD Quark, Intel OpenVINO release notes 2025.2)
- "Run Qwen-7B in a browser tab" — Chrome ~4GB/tab WebGPU memory cap (maddevs.io writeup); 7B even at int4 is borderline and 7B-Coder needs more context. Use sub-3.8B for browser.
- "Just use Ollama everywhere" — Ollama is llama.cpp; gives away the MLX 1.5-3x speedup on Apple Silicon and the FA2 speedup on RTX. (willitrunai Mac mini M4 Pro bench, llmcheck.net)
- "Foundation Models framework is a drop-in for the sidecar" — 4096 ctx, English-first, Apple Intelligence-eligible device only, generation quality is ~3B class. Not a coding-model replacement; it's a structured-output helper.

## Open questions
- Measure Qwen2.5-Coder-7B-Instruct-abliterated tok/s on M3 Max 64GB via vllm-mlx at Q4 and Q8 — how does it compare to ELAI's current RTX 3060 baseline of 25K-ctx 100% pass / ~1.92 GB VRAM at NF4? (We have no direct M3 Max number for THIS model.)
- Does vllm-mlx /v1/messages contract match Anthropic's spec closely enough that ELAI's existing CloudGroundingClient parser works unchanged, or are there divergences (tool_use shape, stream events)?
- Measure cold-boot time of vllm-mlx vs current Python HF+FA2 sidecar — does MLX win on the spin-up that hurts our remote-tunnel UX?
- Foundation Models framework guided-generation @Generable can emit Swift structs; can we map this to ELAI's research_tree DTOs (the same ones that needed patch-R2.3 lenient_vec)? Could be a much cheaper than running a 7B model for that node.
- Browser-mode transformers.js: what's the realistic Phi-4-mini-3.8B tok/s on the Tauri webview on a target user's M3 Pro vs RTX 4060? If above 15 tok/s, the dashboard could drop the sidecar for several "fast utility" agents entirely.

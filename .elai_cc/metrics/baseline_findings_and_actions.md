BASELINE MEASUREMENT FINDINGS AND ACTION ITEMS
Date: 2026-04-14
Status: IN PROGRESS (9 of 11 measurements pending system reboot)
Commits: patch-1.21, patch-1.22

HARDWARE
  GPU: RTX 3060 12GB (Ampere, compute capability 8.6)
  Usable VRAM: ~9.7GB (2.3GB lost to OS/display, 0GB if onboard graphics)
  Model: Gemma 4 E2B (2.3B effective params, 35 layers, NF4 quantized)
  Model VRAM: 7.5GB (NF4 weights)
  Free for KV cache: ~2.2-4.5GB depending on OS overhead

ATTENTION ARCHITECTURE (from HF Transformers source)
  35 total layers:
    28 sliding attention layers (head_dim=256, local window)
    7 global attention layers (head_dim=512, full sequence)
  The 7 global layers are the O(n^2) bottleneck.
  The 28 sliding layers are already ~O(n) (fixed window size).

MEASURED ATTENTION CEILING
  Config: NF4 + SDPA (attn_implementation="sdpa"), no DoLa, no TurboQuant
  ctx=512 (~786 real tokens): 100% pass, 7.2 tok/s
  ctx=1024 (~1664 real tokens): 100% pass, 7.0 tok/s
  ctx=2048 (~3410 real tokens): 100% pass, 5.5 tok/s
  ctx=3072 (~5132 real tokens): 0% pass (timeout at 120s, sidecar finishes in 165s)
  Practical ceiling: 2048 context tokens = ~3400 real input tokens
  Hard wall: 3072 context tokens = prefill takes 165s (unusable)

WHY THE CEILING EXISTS
  The bottleneck is NOT VRAM. It is prefill compute time.
  At 5132 input tokens, the 7 global attention layers compute a 5132x5132
  attention matrix per head. That is 26.3 million entries per head per layer.
  With 8 heads and 7 global layers: 1.47 billion attention scores.
  On RTX 3060 (12.7 TFLOPS fp16): this takes ~100-165 seconds.
  The 28 sliding attention layers are fast (fixed window).
  SDPA helps ~45% (tiles the computation) but the O(n^2) math is unchanged.

WHY FLASH LINEAR ATTENTION CANNOT HELP
  Flash Linear Attention (fla-org/flash-linear-attention) implements O(n)
  attention for architectures that use linear attention kernels (RWKV, Mamba,
  RetNet, GLA, HGRN, etc.). These models are TRAINED with a different
  attention formula: phi(Q) * (phi(K)^T * V) instead of softmax(QK^T/sqrt(d)) * V.
  You cannot swap the attention kernel on an existing model without retraining.
  Gemma 4 uses standard softmax attention. Flash Linear does not apply.

WHY FLASH ATTENTION 2 IS PARTIALLY BLOCKED
  Flash Attention 2 (Dao-AILab/flash-attention) supports head_dim <= 256.
  Gemma 4's 28 sliding layers have head_dim=256 (compatible).
  Gemma 4's 7 global layers have head_dim=512 (incompatible, crashes).
  HuggingFace issue #45201 tracks per-layer dispatch (FA2 for sliding, SDPA
  for global). Not merged into stable transformers yet.
  When available: FA2 would accelerate 28/35 layers (~80% of the model).
  The 7 global layers would still use SDPA. Net improvement: significant
  but the global layers remain the bottleneck for very long sequences.

BUGS FOUND AND FIXED
  1. Missing tokenizer_config.json in local model directory
     Symptom: model outputs only <turn|> token (chat template not applied)
     Fix: copied tokenizer_config.json + chat_template.jinja from HF cache
     Files: models/gemma-4-E2B-it/tokenizer_config.json, chat_template.jinja

  2. DoLa _get_decoder_layers() wrong path for Gemma4 VLM
     Symptom: cannot_find_decoder_layers error, output_hidden_states=True
       captures all 35 layers per token, generation at 0.1-0.3 tok/s
     Root cause: Gemma4ForConditionalGeneration.model.model.language_model.layers
       not at model.model.layers (standard) or model.language_model (first fix)
     Fix: added model.model.language_model.layers path to _get_decoder_layers()
     File: python/elai_inference/server.py

  3. TurboQuant not installed
     Symptom: turboquant_unavailable warning, KV compression disabled
     Root cause: turboquant Python package not pip-installed (source in docs/.source/)
     Fix: pip install -e from docs/.source/turboquant_plus-main/turboquant_plus-main/

  4. TurboQuant KV compression is BROKEN for production
     Symptom: with TQ enabled, requests that take 5s without TQ timeout at 180s.
       GPU memory balloons to 10.8GB. Creates unkillable zombie CUDA processes.
     Root cause: compress_cache() does GPU->CPU transfer per token per layer,
       Python for-loop over each KV vector (not vectorized), then CPU->GPU back.
       This adds massive overhead per generated token. Also, when interrupted
       mid-compression, the CUDA context enters an unrecoverable state.
     Impact: TurboQuant as implemented is DANGEROUS. Do not enable.
     Fix needed: complete rewrite (see action items below)

SDPA ENABLED (attn_implementation="sdpa")
  Added to both NF4 and fp16 model loading in server.py from_pretrained().
  PyTorch 2.6 native SDPA dispatches to memory-efficient attention kernels.
  Results: ~4% faster tok/s at small contexts, ~45% faster prefill at 3072.
  No downsides. Should always be enabled.

DOLA STATUS
  Layer detection now fixed (model.model.language_model.layers).
  DoLa with output_hidden_states=True: ~2x slower (3.3 vs 6.9 tok/s).
  Factuality improvement on 2B model: marginal (per prior eval).
  Recommendation: disable for code generation, enable only for factuality tasks.
  Not yet tested with corrected layer path (needs reboot first).

STEERING VECTORS STATUS
  Computation works via /v1/compute_steering endpoint.
  Effect on generation not yet measured (all tests interrupted by zombies).
  Needs measurement after reboot with corrected DoLa layer path.

ACTION ITEMS (ordered by priority)

  IMMEDIATE (after reboot):
    1. Run remaining 9 benchmark measurements:
       cd .elai_cc/benches/model_capability && python run_benchmark.py
       (retry, few-shot, structured, template, DoLa, steering, composition, cache, hallucination)

    2. Test DoLa with corrected layer path (model.model.language_model.layers):
       Start sidecar with --dola and verify tok/s improves from 0.3 to ~3.5 tok/s.
       If still too slow, investigate output_hidden_states overhead.

    3. Test steering vectors with working sidecar:
       Compute factuality + code_completeness vectors, measure effect on output quality.

  SHORT TERM:
    4. Rewrite TurboQuant compress_cache() for GPU-native operation:
       - Replace .cpu().numpy() with torch operations on GPU
       - Replace Python for-loop with batched torch.quantize/dequantize
       - Test: verify no zombie processes, verify speedup vs overhead
       - Goal: KV compression that actually REDUCES memory bandwidth usage

    5. Add sidecar safety guards:
       - max_input_tokens=3000 parameter to reject oversized prompts
       - request_timeout_s configurable per endpoint
       - Graceful CUDA cleanup on timeout (prevent zombie processes)

    6. Switch to onboard graphics (recover ~2GB VRAM):
       - Currently losing 2.3GB to OS/display driver
       - With onboard: 0GB lost, ~4.5GB free for KV cache
       - May push practical ceiling from 2048 to 2560-3072

  MEDIUM TERM:
    7. Monitor HuggingFace transformers#45201 (per-layer FA2 dispatch):
       - When merged: enable FA2 for 28 sliding layers (head_dim=256)
       - SDPA fallback for 7 global layers (head_dim=512)
       - Expected: major prefill speedup, possibly 2-3x at large contexts

    8. Evaluate torch.compile() for the model:
       - PyTorch 2.6 supports torch.compile with CUDA graphs
       - May reduce per-token overhead and improve prefill speed
       - Needs testing with bitsandbytes NF4 compatibility

    9. Consider alternative models without head_dim=512 global layers:
       - If attention ceiling is critical, a model with uniform head_dim<=256
         would allow full FA2 acceleration on all layers
       - Gemma 2 2B uses head_dim=256 everywhere (FA2 compatible)
       - Tradeoff: Gemma 4 has better quality but slower at large contexts

  LONG TERM:
    10. KV cache offloading to CPU RAM:
        - Keep recent tokens' KV on GPU, older tokens on CPU
        - Swap back to GPU only for global attention layers
        - Effectively infinite context at the cost of latency per global layer

    11. Speculative decoding with draft model:
        - Small draft model (< 1B) generates candidate tokens
        - Main model verifies in parallel (1 forward pass for N tokens)
        - Already evaluated in phase 1F (decision was DROP for RTX 3060 VRAM)
        - Revisit if VRAM situation improves (onboard graphics switch)

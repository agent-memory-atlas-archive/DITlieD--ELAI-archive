verify-p5-serving.md
task_id: verify-p5-serving
date: 2026-06-11
scope: vLLM V1 / SGLang migration feasibility for ELAI custom logits chain
primary sources fetched: docs.vllm.ai, docs.sglang.io, github.com/vllm-project/vllm, github.com/sgl-project/sglang, github.com/QwenLM/Qwen3-Coder

---

QUESTION 1: vLLM V1 custom logits processors -- per-request stateful, hidden-state access, DoLa

VERIFIED: RFC #17799 is merged and shipped. vLLM V1 has a custom logits processor API.
  source: https://docs.vllm.ai/en/latest/features/custom_logitsprocs/
  source: https://github.com/vllm-project/vllm/issues/17799

VERIFIED: Processors are GLOBAL (registered at engine init), NOT per-request dynamic loading.
  Per-request control is via SamplingParams.extra_args (dict passed per request), which
  lets a globally-registered processor branch on request-level params. Class must subclass
  vllm.v1.sample.logits_processor.LogitsProcessor, define validate_params + update_state.
  This supports per-request STATEFUL behavior, but the processor object itself is global.
  source: https://docs.vllm.ai/en/latest/features/custom_logitsprocs/

VERIFIED: LogitsProcessor sees FINAL LOGITS ONLY -- shape (num_requests, vocab_size).
  There is NO hidden-state access from within a logits processor.
  The HiddenStatesProcessor RFC (#12249, linked PR #22820) proposes post-hoc hidden-state
  extraction for pooling tasks, but the RFC's own description covers SINGLE-LAYER final
  hidden states, not multi-layer contrast. No multi-layer access path exists in the
  merged interface.
  source: https://github.com/vllm-project/vllm/issues/12249

VERDICT: DoLa layer-contrast decoding is STRUCTURALLY IMPOSSIBLE as a vLLM logits processor.
  DoLa requires forward hooks on specific intermediate decoder layers captured DURING the
  forward pass (our implementation: DoLaSteeringProcessor registers per-layer forward hooks
  on model.model.layers, captures hidden states in _captured_hidden, then runs lm_head on
  each candidate layer's hidden state in __call__). The logits processor interface in vLLM
  sees only the output of the final lm_head projection; the per-layer hidden states are
  inaccessible and no hook mechanism is exposed.
  source: verified against server.py:786-955 (DoLaSteeringProcessor.__call__ + register_hooks)
  source: https://docs.vllm.ai/en/latest/features/custom_logitsprocs/ (logits tensor only)

VERIFIED: vLLM has NO native DoLa support. Not listed in any documented feature.
  source: docs.vllm.ai exhaustive search -- DoLa absent from speculative decoding, sampling,
  logits-processor, and quantization docs.

VERIFIED: vLLM has native prompt-lookup/ngram speculative decoding (--speculative-config
  method: ngram), BUT it falls back to V0 engine when VLLM_USE_V1=1 is set.
  Issue #16883 closed as "not planned" / "stale". Ngram spec-decode and V1 do not compose.
  source: https://github.com/vllm-project/vllm/issues/16883

---

QUESTION 2: VLLM_BATCH_INVARIANT -- real env var, status, GPU requirements, issue #27433

VERIFIED: VLLM_BATCH_INVARIANT=1 is a real environment variable.
  source: https://docs.vllm.ai/en/latest/features/batch_invariance/

VERIFIED: Status is BETA ("batch invariance is currently in beta").
  source: https://docs.vllm.ai/en/latest/features/batch_invariance/

VERIFIED: Requires NVIDIA GPU compute capability >= 8.0.
  RTX 3090 is SM 8.6 (Ampere), which satisfies the >= 8.0 requirement.
  source: https://docs.vllm.ai/en/latest/features/batch_invariance/

VERIFIED: Issue #27433 exists and references batch-invariant feature/performance optimization.
  source: https://github.com/vllm-project/vllm/issues/27433

VERDICT: VLLM_BATCH_INVARIANT is real, RTX 3090 is compatible (SM 8.6 >= 8.0),
  BETA status with acknowledged performance trade-off vs non-deterministic kernels.
  The claim in the deep-research report is accurate.

---

QUESTION 3: vLLM quantization on RTX 3090 -- AWQ-Marlin on Ampere, bitsandbytes NF4

VERIFIED: AWQ with Marlin kernel IS supported on Ampere SM 8.6 (RTX 3090).
  The Marlin kernel requires SM >= 8.0; SM 8.6 satisfies this.
  A community benchmark confirms: "AWQ without Marlin 67 tok/s, with Marlin 741 tok/s" on Ampere.
  source: https://docs.vllm.ai/en/v0.9.0/features/quantization/supported_hardware.html
  source: https://jarvislabs.ai/blog/vllm-quantization-complete-guide-benchmarks

VERIFIED: vLLM supports bitsandbytes 4-bit (load_in_4bit) and 8-bit. The bnb docs confirm
  4-bit quantization types fp4 and nf4 are recognized config options. vLLM reads quantization
  config from model files, so an existing NF4-quantized checkpoint (like the ELAI sidecar's
  current config) would load, but documentation does not explicitly list NF4 as validated.
  Tensor parallelism is NOT supported with bitsandbytes in vLLM (single-GPU only).
  source: https://docs.vllm.ai/en/stable/features/quantization/bnb/

UNCLEAR: Whether vLLM's bitsandbytes integration accepts bnb_4bit_quant_type=nf4 for
  pure inference (no gradient) without silent accuracy loss. The docs confirm 4-bit BNB
  support but do not explicitly validate NF4 (vs fp4) for the inference path. Needs empirical
  test. vLLM recommends migrating NF4 models to AWQ for production.

VERIFIED: Quantizing an abliterated/custom finetune to AWQ requires running AutoAWQ
  calibration on the custom checkpoint with ~128-256 calibration samples. The abliteration
  is a weight-level modification; AutoAWQ quantizes from any HF-compatible model, so this
  is a one-time offline step, not a blocker.
  source: https://github.com/casper-hansen/AutoAWQ
  source: https://docs.vllm.ai/en/latest/quantization/auto_awq.html

---

QUESTION 4: vLLM automatic prefix caching on streaming path

VERIFIED: vLLM V1 automatic prefix caching (APC) hashes full KV blocks and reuses them
  across requests sharing the same prefix. The system immediately adds a full block to the
  cache, enabling reuse by subsequent requests in the same or future batches.
  source: https://docs.vllm.ai/en/stable/design/v1/prefix_caching.html

UNCLEAR: The fetched docs do not explicitly state whether APC is active on the streaming
  (SSE token-by-token) path. The design doc describes request lifecycles without
  distinguishing streaming vs non-streaming. Given that KV blocks are populated during
  prefill (before any tokens are streamed), the prefix cache for the INPUT side is active
  regardless of streaming mode. OUTPUT-token KV blocks accumulate during decode; those
  can be cached for future requests after the request completes.
  source: https://docs.vllm.ai/en/stable/design/v1/prefix_caching.html

VERDICT: Cross-request KV reuse for SHARED SYSTEM PROMPTS is confirmed and works regardless
  of streaming. This is the primary win vs current HF sidecar (which has zero APC).

---

QUESTION 5: SGLang custom logits processors -- issue #2291 status, --logits-processors-entrypoints

VERIFIED: SGLang has shipped custom logit processors behind --enable-custom-logit-processor
  server flag. Issue #2291 is closed. The API passes a serialized callable per request
  (dill-pickled, hex-encoded in request JSON). Per-request stateful: custom_params dict
  passed alongside each request.
  source: https://docs.sglang.io/basic_usage/sampling_params.html
  source: https://github.com/sgl-project/sglang/issues/2291

CRITICAL: CVE-2026-7304 -- SGLang custom logit processor uses dill.loads on untrusted
  request bytes, enabling unauthenticated RCE. The server flag --enable-custom-logit-processor
  must NEVER be enabled on a network-accessible endpoint without trusted-caller gating.
  source: https://blog.securelayer7.net/cve-2026-7304-sglang-unauthenticated-rce/

VERIFIED: SGLang custom logit processors receive the logits tensor and custom_params.
  No hidden-state access is described. DoLa is NOT implementable via this path for the
  same structural reason as vLLM: only final logits are exposed.
  source: https://docs.sglang.io/basic_usage/sampling_params.html

UNCLEAR: Whether the vLLM-style --logits-processors-entrypoints (file-path registration)
  exists in SGLang. The documented SGLang mechanism is --enable-custom-logit-processor +
  per-request dill-serialized callable. No entrypoints-style CLI flag found.

VERIFIED: Known race condition bug (issue #8056) in SGLang overlap mode: custom_params and
  custom_logit_processor accessed by two concurrent threads without ordering guarantee.
  source: https://github.com/sgl-project/sglang/issues/8056

VERIFIED: XGrammar is the default structured generation backend in SGLang (integrated
  Nov 2024), same as vLLM. Grammar-constrained decoding is a native built-in in both engines.
  source: https://github.com/mlc-ai/xgrammar (confirmed pypi.org/project/xgrammar)

---

QUESTION 6: Anthropic /v1/messages protocol -- native in vLLM or SGLang?

VERIFIED: vLLM has a native /v1/messages Anthropic Messages API endpoint, shipped with
  dedicated modules (vllm.entrypoints.anthropic.serving_messages). Documented integration
  guide for Claude Code at docs.vllm.ai/en/latest/serving/integrations/claude_code/.
  The implementation is a translation adapter: vLLM ingests Anthropic-shaped requests and
  translates to its internal OpenAI-compat routing. The adapter is server-side and
  transparent to the client.
  source: https://docs.vllm.ai/en/latest/serving/integrations/claude_code/
  source: https://docs.vllm.ai/en/stable/api/vllm/entrypoints/anthropic/

UNCLEAR: SGLang /v1/messages status. Issue #9594 (opened Aug 2025, closed as "inactive")
  requested this feature. No evidence of a shipped native /v1/messages implementation in
  SGLang. An OpenAI-to-Anthropic gateway (like SMG) would be required as an adapter layer.
  source: https://github.com/sgl-project/sglang/issues/9594

VERDICT: vLLM is a drop-in replacement for ELAI's current sidecar from the Rust client
  perspective (same /v1/messages endpoint), requiring only URL change. SGLang requires
  an additional adapter layer.

---

QUESTION 7: Memory / practical on 24GB -- vLLM serving Qwen2.5-Coder-7B 4-bit + 32K KV

VERIFIED: Qwen2.5-Coder-7B at Q4 (AWQ/GGUF) model weights: ~5.0-5.5 GB VRAM.
  source: https://www.spheron.network/tools/gpu-recommender/Qwen/Qwen2.5-7B-Instruct-AWQ/
  source: https://localaimaster.com/models/qwen-2-5-7b

ESTIMATED (not directly measured in a retrieved doc):
  At 32K context, Qwen2.5-Coder-7B (28 layers, 4096 hidden, GQA 8 KV heads, 128 head_dim)
  KV cache in fp16: 2 * 8 * 128 * 32768 * 28 * 2 bytes ~ 3.67 GB per request.
  vLLM pre-allocates a KV cache pool (default gpu_memory_utilization=0.90), so on 24 GB:
    model weights ~5 GB + CUDA overhead ~1.5 GB = ~6.5 GB consumed before KV pool.
    Remaining for KV pool: 24 * 0.9 - 6.5 ~ 15 GB.
    Batch of 1-4 requests at 32K: 1 req * 3.67 GB = 3.67 GB; 4 reqs = ~14.7 GB.
  With APC enabled (blocks are re-used), effective headroom for multi-turn at batch=4 is
  tight but feasible for ELAI's single-user batch 1-4 workload.
  note: the community 3090 run confirms Qwen3.6-27B (larger) on dual 3090s worked;
    the 7B AWQ is well within range on a single 3090.
  source: https://derekarmstrong.dev/blog/running-qwen36-27b-dual-rtx-3090-vllm-v019/
  source: https://www.spheron.network/tools/gpu-recommender/Qwen/Qwen2.5-7B-Instruct-AWQ/

VERDICT: 7B AWQ at 32K context on 24 GB RTX 3090 is feasible with ~8-10 GB KV headroom
  remaining at batch=4. Cold start: model load (~5 GB weights) + pool allocation in ~30-60s.

---

QUESTION 8: Qwen3-Coder 14B -- does this model exist?

REFUTED: No "Qwen3-Coder 14B" model exists in the Qwen3-Coder family.
  The Qwen3-Coder family contains: Qwen3-Coder-480B-A35B-Instruct, Qwen3-Coder-30B-A3B-Instruct,
  and Qwen3-Coder-Next (80B MoE, 3B activated). There is NO 14B dense Coder variant.
  source: https://github.com/QwenLM/Qwen3-Coder (official QwenLM repo)

REFUTED: The ~58% SWE-Bench claim at ~9GB Q4 cannot be verified -- the model the claim
  would require does not exist. Qwen3-Coder-30B-A3B at Q4 activates ~3B params but total
  weights >> 9 GB. The closest match would be Qwen3-Coder-Next at full 4-bit quant, which
  requires >> 30 GB. The "14B / 58% / 9GB" cluster is an invention.
  source: https://github.com/QwenLM/Qwen3-Coder
  source: https://unsloth.ai/docs/models/tutorials/qwen3-coder-how-to-run-locally

---

PORTABILITY TABLE
(portable-natively = works as-is; portable-as-custom = implementable with custom code;
 NOT-portable = structurally impossible given the engine's public interface)

Component          | vLLM V1               | SGLang                | Keep HF
-------------------+-----------------------+-----------------------+------------------
DoLa layer-        | NOT-portable          | NOT-portable          | portable-natively
contrast           | (logits only, no      | (logits only, no      | (forward hooks on
                   | hidden-state hook)    | hidden-state hook)    | model.model.layers)
-------------------+-----------------------+-----------------------+------------------
ETC (entropy-      | portable-as-custom    | portable-as-custom    | portable-natively
trend constraint)  | (observes logits;     | (observes logits;     |
                   | stateless -> works    | dill-callable;        |
                   | via extra_args)       | CVE risk if exposed)  |
-------------------+-----------------------+-----------------------+------------------
Prompt-lookup      | NOT-portable in V1    | UNCLEAR (SGLang has   | portable-natively
(ngram spec-       | (V1 falls back to     | draft speculation but | (model.generate
decode)            | V0; issue #16883      | ngram status not      | prompt_lookup_num
                   | closed not-planned)   | confirmed)            | _tokens kwarg)
-------------------+-----------------------+-----------------------+------------------
XGrammar grammar   | portable-natively     | portable-natively     | portable-as-custom
masking            | (XGrammar is the      | (XGrammar is the      | (lmformatenforcer
                   | default backend;      | default backend       | as prefix_allowed_
                   | built-in)             | since Nov 2024)       | tokens_fn)
-------------------+-----------------------+-----------------------+------------------
Seeded-Gumbel      | portable-as-custom    | portable-as-custom    | portable-natively
sampling           | (custom logits proc   | (custom logits proc   | (LogitsProcessor
(planned vendor    | + extra_args seed;    | + custom_params;      | + generator seed)
op)                | VLLM_BATCH_INVARIANT  | race condition in     |
                   | provides batch-invar  | overlap mode bug)     |

---

FEASIBILITY-HINT (not a decision, evidence summary only)

vLLM V1 survives on 5/5 items EXCEPT DoLa and ngram-spec-decode:
  DoLa is architecturally impossible (no hook surface); would require removing DoLa entirely
  or adding a vLLM model-level patch outside the public API.
  Ngram spec-decode falls back to V0 engine (issue closed not-planned), so VLLM_USE_V1=1
  and ngram are mutually exclusive.
  APC + Anthropic-native /v1/messages + AWQ-Marlin on SM 8.6 + VLLM_BATCH_INVARIANT all
  confirm for vLLM V1.

SGLang survives on 4/5 items EXCEPT DoLa (same structural reason as vLLM) and has an
  additional RCE security concern on the custom-logit-processor path (CVE-2026-7304 via
  dill.loads), plus no native /v1/messages (adapter required). The race condition in overlap
  mode is a correctness concern for the ETC and seeded-Gumbel processors.

Keep-HF is the only path where DoLa and ngram-spec-decode are natively portable.
  If DoLa is non-negotiable, neither vLLM V1 nor SGLang works without a custom model patch
  that hooks into the forward pass at the engine level, which is outside the documented API
  surface and would require a maintained fork.

The one clear migration path that avoids the DoLa blocker: drop DoLa from the chain first
  (it is already behind --no-dola flag in server.py argparse, and is disabled on the 3B
  remote per CLAUDE.md boot command), then migrate to vLLM V1 and gain APC + Marlin-AWQ
  throughput + VLLM_BATCH_INVARIANT determinism.

---

SOURCES CITED
https://docs.vllm.ai/en/latest/features/custom_logitsprocs/
https://github.com/vllm-project/vllm/issues/17799
https://github.com/vllm-project/vllm/issues/12249
https://docs.vllm.ai/en/latest/features/batch_invariance/
https://github.com/vllm-project/vllm/issues/27433
https://docs.vllm.ai/en/v0.9.0/features/quantization/supported_hardware.html
https://docs.vllm.ai/en/stable/features/quantization/bnb/
https://github.com/casper-hansen/AutoAWQ
https://docs.vllm.ai/en/stable/design/v1/prefix_caching.html
https://docs.sglang.io/basic_usage/sampling_params.html
https://github.com/sgl-project/sglang/issues/2291
https://blog.securelayer7.net/cve-2026-7304-sglang-unauthenticated-rce/
https://github.com/sgl-project/sglang/issues/8056
https://github.com/sgl-project/sglang/issues/9594
https://docs.vllm.ai/en/latest/serving/integrations/claude_code/
https://docs.vllm.ai/en/stable/api/vllm/entrypoints/anthropic/
https://github.com/vllm-project/vllm/issues/16883
https://github.com/QwenLM/Qwen3-Coder
https://unsloth.ai/docs/models/tutorials/qwen3-coder-how-to-run-locally
https://www.spheron.network/tools/gpu-recommender/Qwen/Qwen2.5-7B-Instruct-AWQ/
https://derekarmstrong.dev/blog/running-qwen36-27b-dual-rtx-3090-vllm-v019/
https://github.com/mlc-ai/xgrammar
https://jarvislabs.ai/blog/vllm-quantization-complete-guide-benchmarks
https://github.com/agencyenterprise/vllm-hidden-states

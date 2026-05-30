E-2 RUNPOD-MEASURED record - sidecar live telemetry
=====================================================

Date: 2026-05-30
Session: s-25120 completion-drive (sidecar-B executor)
Sidecar: huihui-ai/Qwen2.5-Coder-3B-Instruct-abliterated NF4 4-bit, SDPA, RTX 4090 (remote pythondev 192.0.2.10:6465, forwarded localhost:8100)

This file records live measurements that close the NON-determinism portions
of E-2 DECISION-11. The byte-equivalence harnesses
(test_prefix_kv_determinism.py + test_prompt_lookup_lossless.py) remain
RUNPOD-PENDING because they require in-process model.generate hooks.

----------------------------------------------------------------------------
Round A: non-streaming end-to-end (20 unique + 10 shared-system-prefix prompts)
----------------------------------------------------------------------------

n_calls               = 30
p50_elapsed_ms        = 2403.5   (32-token outputs)
p95_elapsed_ms        = 2504.0
p50_per_token_ms      = 75.34    (~13.3 tok/sec)
p95_per_token_ms      = 78.28
total_input_tokens    = 2363
total_cache_read      = 1810
total_cache_create    = 553
prefix_kv_accept_rate = 0.7660  (76.6%)
acceptance_>=30%      = PASS (DECISION-3 / promotion-gate G2 evidence layer
                        for the EXISTING custom PrefixKVCache hit path; the
                        HF speculative-decoding acceptance gate G2 proper
                        requires in-process logging not exposed to e2e
                        callers).

Source: workflow/sessions/s-25120/completion-drive/runpod-results/sidecar-B-e2-measure.py
Log:    workflow/sessions/s-25120/completion-drive/runpod-results/sidecar-B-e2-results.log

----------------------------------------------------------------------------
Round B: SSE streaming (10 prompts, max_tokens=64)
----------------------------------------------------------------------------

n_streams              = 10
p50_ttft_ms            = 822.33
p95_ttft_ms            = 968.89
p50_total_ms           = 4645.41
p50_inter_chunk_ms     = 61.19
p95_inter_chunk_ms     = 176.18
p99_inter_chunk_ms     = 200.40

G3 verdict             = p50 inter-chunk = 61ms BELOW the 100ms FramePacer
                         threshold per E-2-decisions.md DECISION-11/G3.
                         Recommendation: keep ELAI_FRAME_PACER_ENABLE
                         default-OFF on this hardware tier. The p95/p99 tail
                         (176-200ms) IS where a pacer would smooth, but the
                         median user does not need it.

Source: workflow/sessions/s-25120/completion-drive/runpod-results/sidecar-B-e2-stream-p50.py
Log:    workflow/sessions/s-25120/completion-drive/runpod-results/sidecar-B-e2-stream-results.log

----------------------------------------------------------------------------
What this does NOT close
----------------------------------------------------------------------------

* G1 (HF StaticCache byte-equivalence) -> still RUNPOD-PENDING; needs
  test_prefix_kv_determinism.py run with the model loaded in-process.
* G2 (HF prompt-lookup acceptance) -> still RUNPOD-PENDING; requires
  patching server.py to log HF generate's accepted-token count and
  pooling over 1 week of real telemetry.
* Bilgihan paper ingestion -> still pending (background).

The numbers above are LIVE evidence for promotion-gate G3 evaluation and
empirical confirmation that the EXISTING custom PrefixKVCache delivers high
hit rates under shared-system-prefix workloads.

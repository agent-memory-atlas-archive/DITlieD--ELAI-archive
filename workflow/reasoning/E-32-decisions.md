E-32 decisions: context window extension, FITM calibration, deep mode, YaRN rejection
Authored: 2026-05-21

DECISION 1: Raise hard ceiling from 25K to 32K (not YaRN-128K)

Why 32K: Qwen2.5-Coder-7B was natively trained at 32,768 tokens per the HF model card
(huggingface.co/Qwen/Qwen2.5-Coder-7B-Instruct). Raising to 32K is free — no RoPE
scaling, no quality impact, no extra VRAM operator (only linear KV growth: ~921 MB at
25K -> ~1.2 GB at 32K on NF4 7B; ~2.85 GB total -> ~3.1 GB total on remote 3090).
The 3090 has 24 GB; 3.1 GB total leaves ~20 GB headroom.

Why NOT YaRN-128K:
- vllm issue 18728: YaRN-static factor=4 causes 15-20% quality drop on short prompts
  on Qwen3-32B (same RoPE family as Qwen2.5-Coder).
- arxiv:2601.15300: Qwen2.5-7B F1 drops 0.55 -> 0.30 at 40-50% of the 128K advertised
  window. A 128K window with F1 cliff at 50K is worse than a honest 32K window.
- arxiv:2404.06654 RULER: only half of "32K"-claimed models actually pass 32K evals.
  We ship the window we can verify (native 32K), not the one we advertise.
- Loud rejection: if ELAI_USE_YARN=1, sidecar raises SidecarBootError at boot with
  the full rationale so the operator is never silently degraded.

DECISION 2: Found-in-the-Middle (FITM) calibration — default OFF, eval-gated default-on

Why FITM: arxiv:2406.16008 (Hsieh et al., ACL 2024) shows training-free positional-prior
subtraction gives +15pp RAG accuracy (Recall@3: 20.5 -> 68.3 on NaturalQuestions).
ELAI's RAG path is a direct beneficiary: knowledge engine retrieves N chunks, sidecar
answers from the assembled context. Lost-in-the-Middle (arxiv:2307.03172 TACL 2024)
documents the 30%+ accuracy drop when the relevant chunk is in the middle position.
FITM subtracts the positional prior from one extra forward pass (dummy doc at target
position), nulling the attention sink without any fine-tuning.

Rollout gate: FITM is default OFF (ELAI_FITM_ENABLED=0). Default-on ONLY after the
internal RAG eval harness (benches/context_window/run_fitm_eval.py) confirms >= +5pp
recall@3 on ELAI's actual workload (not the paper's NaturalQuestions numbers). The paper
+15pp is on a Wikipedia QA benchmark; ELAI's workload (code context, multi-file synthesis)
may have a different positional distribution. Measure first, enable second.

Cost: 1 extra forward pass per request with prompt_len > 8K. On a 3090 with 7B NF4
and SDPA, a forward pass at 8K tokens takes ~100-200ms additional latency. Acceptable
for deep analysis tasks; unacceptable for sub-second interactive completions.
Short requests (prompt_len <= 8K) are completely unaffected (guard at processor entry).

DECISION 3: Deep mode opt-in (Qwen2.5-7B-Instruct-1M with DCA at 64K)

Why: arxiv:2501.15383 (Qwen2.5-1M technical report): the 7B-1M variant achieves
near-perfect 1M passkey recall with Dual Chunk Attention (DCA) and 96.7% activation
memory reduction via chunked prefill. For cross-file synthesis tasks exceeding 32K,
deep mode is the clean solution — no RoPE distortion, no short-prompt regression.

Gating: deep mode ONLY activates when:
1. ELAI_DEEP_MODE=1 is explicitly set at sidecar boot.
2. The box has >= 24 GB VRAM (checked via torch.cuda.get_device_properties).
3. On < 24 GB boxes: refuse boot with a warning, fall back to standard 7B at 32K.

Context cap in deep mode: 64,000 tokens (NOT 1M). The 1M context requires ~120 GB VRAM
at BF16 per the paper; at NF4 on 24 GB, 64K is the practical ceiling. The DCA chunked
prefill infrastructure is in place for future extension once larger hardware is available.

Deep mode escalation: when SessionDag handoff fires at 85% of effective_max_context
(27,200 tokens at 32K; session token count approaches saturation) AND the task is
cross-file synthesis (>3 files referenced, no clear retrieval-localised answer),
prefer deep-mode escalation over cloud escalation. Rationale: arxiv:2407.16833 (Self-Route
EMNLP 2024) shows 63% of queries get identical answers from RAG and long-context with
39-65% cost savings. Cloud Sonnet at $3/MTok in + $15/MTok out is expensive for tasks
that can be served by a local 64K context window. Default policy: prefer local deep mode
IF a 24GB remote box is reachable.

DECISION 4: SessionDag handoff threshold — dynamic not hard-coded

Old: handoff_threshold hard-coded as 85% of 25000 = 21250 tokens.
New: handoff_threshold reads effective_max_context from the sidecar's reported capability.
With standard 7B at 32K: 85% of 32000 = 27200 tokens.
With deep mode at 64K: 85% of 64000 = 54400 tokens.

The 85% figure is kept (gives 15% headroom for system prompt + tool overhead).
Fallback: if sidecar is unreachable at handoff time, use 25000 as the conservative default
(the old ceiling, never below the tested baseline from memory:project_qwen3b_baseline).

DECISION 5: Out-of-scope items (explicitly rejected)

YaRN: rejected above. Loud SidecarBootError if attempted.
StreamingLLM (arxiv:2309.17453): wrong abstraction. StreamingLLM discards middle tokens
to fit a fixed KV budget — trading recall for latency. ELAI's use case (code analysis,
multi-file synthesis) requires middle-token recall. Not applicable.
Ring Attention (arxiv:2310.01889): designed for multi-GPU/multi-host sequence parallelism.
On a single GPU, Ring Attention collapses to vanilla attention with ring-communication
overhead. Zero benefit on the 3090; do not implement.
Infini-attention (arxiv:2404.07143): not ported to Qwen2.5 architecture. The compressive
memory mechanism requires model-architecture hooks not present in the HF checkpoint.
1M context (128K-1M via DCA): deferred. Requires ~120 GB BF16 VRAM per arxiv:2501.15383
or massive quantization beyond NF4. The 2x3090 in the VRAM tier plan (memory:project_vram_tier_plan)
may enable 32B+ models but not 1M context on current hardware.

DECISION 6: A-MEM deferral (open question Q5)

arxiv:2502.12110 (NeurIPS 2025 A-MEM) beats RAG on multi-hop cross-session continuity
per the Letta benchmark. The integration surface (atomic-note Zettelkasten graph overlapping
with ELAI's knowledge engine + symbol graph from B-31/B-32) is large and non-trivial.
Deferred to E-32.B pending:
- B-31 knowledge engine completion (base for note storage)
- B-32 symbol graph completion (cross-reference surface)
- Spike on A-MEM vs. ELAI's existing memory architecture (complementary or redundant?)

OPEN QUESTIONS

Q1: FITM uplift on ELAI's actual workload. Paper shows +15pp on NaturalQuestions.
    ELAI's code-context retrieval may differ. Measure with run_fitm_eval.py before default-on.
Q2: Deep mode boot time. First load of Qwen2.5-7B-Instruct-1M with chunked prefill setup
    may be slow. Profile and document in REMOTE-SIDECAR.md when first deployed.
Q3: KV cache at 32K on local 12GB box (Qwen2.5-Coder-7B NF4). Linear extrapolation says
    ~1.2 GB KV + ~2 GB model = ~3.2 GB total — fits. But FA2 attention buffers on large
    sequences may add overhead. Measure before enabling 32K default-on for local 12GB tier.
Q4: Deep mode vs. cloud escalation A/B policy. Current default (prefer local deep mode IF
    24GB reachable) is safe. Revisit when SSB telemetry (deep_mode_escalation_chosen vs.
    escalate_to_cloud rows) accumulates enough signal.
Q5: A-MEM Zettelkasten vs. ELAI knowledge engine + symbol graph — complementary or
    redundant? Spike needed before committing to E-32.B integration scope.

CITATIONS (verified grounding)

- huggingface.co/Qwen/Qwen2.5-Coder-7B-Instruct: native context 32,768 tokens
- arxiv:2501.15383 Qwen2.5-1M: DCA, 7B-1M variant, near-perfect 1M passkey, 96.7% mem reduction
- arxiv:2406.16008 Found-in-the-Middle: training-free +15pp RAG accuracy (ACL 2024)
- arxiv:2307.03172 Lost in the Middle: TACL 2024, 30%+ drop at middle positions
- arxiv:2601.15300 Intelligence degradation: Qwen2.5-7B F1 0.55->0.30 at 40-50% of 128K
- arxiv:2309.00071 YaRN: rejected for ELAI default
- vllm issue 18728: YaRN-static 15-20% quality drop on Qwen3-32B at factor=4
- arxiv:2404.06654 RULER: only 50% of claimed-32K models pass 32K evals
- arxiv:2407.16833 Self-Route EMNLP 2024: 39-65% cost savings from RAG vs LC routing
- arxiv:2309.17453 StreamingLLM: rejected (wrong abstraction)
- arxiv:2310.01889 Ring Attention: rejected (single-GPU collapse)
- arxiv:2404.07143 Infini-attention: rejected (not ported to Qwen2.5)
- arxiv:2502.12110 A-MEM NeurIPS 2025: deferred to E-32.B

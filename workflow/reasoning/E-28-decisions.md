# E-28 Attention Kernels 2026 — Decision Doc

Plan: E-28 (attention kernels 2026: FA2 mandatory, StreamingLLM sinks, YaRN cautious)
Source report: `workflow/improvement-sweep-2026-05-19/reports/28-attention-kernels-2026.md`
Date: 2026-07-01

## What shipped (Tasks 1-9)

1. flash-attn pin bumped to `>=2.8.3,<2.9` in `python/elai_inference/pyproject.toml` (Task 1).
2. `apply_streaming_llm_mask` helper in `server.py` — env `ELAI_STREAMING_LLM_SINKS` (default 0 = off = byte-identical baseline); keeps first N sink tokens + sliding window of `ELAI_LOCAL_MAX_CONTEXT` (Task 2).
3. `apply_yarn_rope_scaling` + `apply_yarn_rope_scaling_at_boot` helpers — env `ELAI_YARN_FACTOR` (default null = off); patches `rope_scaling` on `AutoConfig` per Qwen2.5-1M recipe (Task 3).
4. `pin_sdpa_backend` helper — when `ELAI_FORCE_SDPA=1`, wraps generation in `torch.nn.attention.sdpa_kernel([SDPBackend.FLASH_ATTENTION])`; raises on shape-mismatch instead of silent MATH fallback (Task 4).
5. `refuse_fa3_on_ampere` helper — actively refuses FA3/FA4/cuDNN-attention module imports on SM<90 (Task 4 companion).
6. 5 new rule rows in `crates/rules/src/schema.rs`: FA3/FA4 Ampere BLOCK, MLA BLOCK, Gemma mixed head_dim WARN, FA2 backward determinism WARN, SDPA-no-backend-pin BLOCK (Task 5).
7. 3 new SSB action types: `StreamingLlmEngaged`, `SdpaBackendPinned`, `YarnRopeScalingApplied` (Task 6).
8. Long-context recipe section in `workflow/remote-setup/REMOTE-SIDECAR.md` (Task 7).
9. Parity test `test_streaming_llm_sinks.py` + YaRN eval-gate `test_yarn_eval_gate.py` + attention contract `test_e28_attention_contract.py` (Tasks 8-9).

## Key decisions

### (a) FA2-only on Ampere; FA3/FA4 deferred to cloud Hopper/Blackwell tier

**Decision:** FA2 (flash-attn 2.8.3) remains the mandatory attention kernel on RTX 3090/3060 (Ampere SM_80/SM_86). FA3 (Hopper SM_90) and FA4 (Hopper+Blackwell SM_90/SM_100) are explicitly out-of-scope for the local sidecar.

**Grounding:** arXiv:2407.08608 (FA3 paper, Hopper TMA + asynchronous warp specialization), tridao.me/blog/2024/flash3 (Tri Dao: "FA2 still recommended on Ampere"), together.ai/blog/flashattention-4 (FA4 CuTeDSL targets SM90+/SM100, lacks backward/GQA/varlen at release). The Dao-AILab/flash-attention repo lists SM_80 as supported with no 2026 deprecation notice.

**Enforcement:** `refuse_fa3_on_ampere(compute_capability)` raises `SidecarBootError` if any `flash_attn_3`/`flash_attn_4`/`cudnn_attention` module is loaded on SM<90. Rule `dp-pip-fa3-fa4-ampere` (BLOCK) in `crates/rules/src/schema.rs`. Opt-out via `ELAI_REFUSE_FA3=0` reserved for future cloud Hopper deploys (not active today).

**Rationale:** The "75% faster than FAv2" cuDNN headline is H100-measured (PyTorch 2.5 release notes). Citing it for 3090 planning is ungrounded conjecture. FA2 at ~70% peak Ampere FLOPs is the proven ceiling.

### (b) StreamingLLM env-gated until parity passes 3 runs

**Decision:** StreamingLLM attention-sink masking is opt-in via `ELAI_STREAMING_LLM_SINKS` (default 0 = off). Default-off is byte-equivalent to today's baseline (the mask helper is a no-op when sink_count=0).

**Gate for promotion:** The parity test (`test_streaming_llm_sinks.py`) must pass 3 consecutive runs at 30K context confirming: (i) output quality parity on a code-completion fixture, (ii) VRAM footprint <= baseline, (iii) throughput >= baseline (streaming is a memory win, not a speed loss), (iv) 3-run byte-equivalence for determinism. Until 3 passing runs are recorded on the remote 3090, StreamingLLM stays env-gated and is NOT a default.

**Grounding:** arXiv:2309.17453 (StreamingLLM ICLR 2024: attention sinks + 4 init tokens + sliding window; up to 4M tokens, 22.2x vs sliding-recompute). Reference impl: github.com/mit-han-lab/streaming-llm (pure-Python mask manipulation, no FA2 changes).

**Determinism:** Forward path is deterministic (pure masking, no fused-CTA ops) per report Section "Determinism concerns".

### (c) YaRN factor selection deferred to human review of Task 9 output

**Decision:** YaRN context-extension is opt-in via `ELAI_YARN_FACTOR` (default null = off). The factor-sweep CSV at `metrics/yarn_eval_factor_sweep.csv` (produced by `test_yarn_eval_gate.py` sweeping factor∈{2.0, 4.0, 8.0} across context positions {8K, 25K, 64K, 128K}) is the input for human review. YaRN is NOT enabled default-on until a human reviews the CSV and confirms acceptable quality at the chosen factor.

**Grounding:** arXiv:2309.00071 (YaRN: NTK-by-parts + attention-temp scaling, kernel-transparent). arXiv:2501.15383 (Qwen2.5-1M tech report: ships factor=4 for 128K target). HF Qwen2.5-Coder-7B config.json: head_dim=128, GQA, rope_theta=1e6 — compatible with YaRN recipe.

**Guard:** `assert config.max_position_embeddings >= 32768` prevents silent mis-config when YaRN is applied. Per CLAUDE.md no-removal: default config is unchanged; YaRN stays opt-in.

### (d) MInference / DuoAttention / RingAttention — rationale for deferral

**MInference (arXiv:2407.02490):** DEFERRED. Open Question 2: the 10x pre-fill speedup is A100-cited; whether the Vertical-Slash kernel achieves the same on SM_86 (3090) is unmeasured. Needs microsoft/MInference reproduction at 50K-100K ctx. Additionally, MInference is approximate by construction (sparse patterns drop low-magnitude entries) — for ELAI's entropy quarantine, outputs would need to be treated as a separate stream, not mixed with full-attention outputs in the same hash chain. Gate: reproduce on 3090, measure, then decide.

**DuoAttention (arXiv:2410.10819):** DEFERRED. Open Question 4: retrieval-head calibration transfer between Qwen2.5-Coder-3B and -7B is unknown. Requires a one-time offline calibration pass per checkpoint. Gate: measure calibration cost + transfer, then decide.

**RingAttention (arXiv:2310.01889):** SKIP. Requires >=2 GPUs in a communication ring; on a single 3090 it is identity. Gate: ELAI hardware upgrade to >=2 GPUs.

### (e) Explicit OUT-OF-SCOPE

- **MLA (Multi-head Latent Attention):** MLA is a weight architecture (DeepSeek-V2/V3), not a kernel. Qwen2.5-Coder has standard GQA weights — there is no MLA path without retraining. Rule `dp-pip-mla-arch-swap` (BLOCK) enforces this. Reference: DeepWiki MLA.
- **Gemma-family head_dim mismatch:** Gemma-3-equivalent models have mixed head_dim (256+512 across layers) which breaks FA2 power-of-two kernels. Qwen2.5-Coder dense head_dim=128 sidesteps this. Rule `dp-pip-gemma-mixed-headdim` (WARN) guards against regressing by adopting Gemma-family models. Reference: HF Qwen2.5-Coder-7B config.json.
- **cuDNN SDPA backend:** The cuDNN SDPA backend is Hopper-leaning (gains mostly on H100+). The "75% faster than FAv2" headline is H100-measured, not Ampere-applicable. `pin_sdpa_backend` pins to `SDPBackend.FLASH_ATTENTION` explicitly, not cuDNN. Reference: PyTorch 2.5 release notes, pytorch#152595.

## Expected test outcome

- `test_streaming_llm_sinks.py`: parity + VRAM + throughput + 3-run determinism at 30K context.
- `test_yarn_eval_gate.py`: factor-sweep CSV produced at `metrics/yarn_eval_factor_sweep.csv`.
- `test_e28_attention_contract.py`: SDPA pin assertion + no-FA3/FA4-import lint + refusal helper.
- SSB roundtrip for `StreamingLlmEngaged`, `SdpaBackendPinned`, `YarnRopeScalingApplied`.
- `cargo check --workspace` GREEN.
- Remote 3090 sidecar boot test PASS (HARDWARE-HEAVY — coordinate with E-2/E-27 for shared `server.py` edit windows).

## Unknowns / risks

- Open Question 1: measured FA2 forward throughput on RTX 3090 at 25K ctx under flash-attn 2.8.3 vs xFormers/FlexAttention is unmeasured. Mitigation: baseline test before any default promotion.
- Open Question 3: YaRN factor sweet spot for Qwen2.5-Coder-7B — Task 9 CSV produces the data; human review required before default-on.
- Anti-Stale-Training: `pip install flash-attn==2.8.3` CUDA+torch matrix must be verified at pin time (CLAUDE.md anti-slopsquatting). pypi 2.8.3 supports CUDA 11.8-12.8 / torch 2.4-2.9 per report.
- Sidecar boot is HARDWARE-HEAVY (CLAUDE.md hard rule). All `server.py` changes must be tested against remote 3090, not a second local instance.
- FA2 backward non-determinism (transformers#31787) is moot today (inference-only). If on-device fine-tune/RLHF is added, mandate `deterministic=True` (~37.9% throughput cost). Rule `dp-pip-fa2-backward-nondeterm` (WARN) is a future-guard.

## Composes with

- E-27 (quantization): AWQ-Marlin weights + FA2 attention + FP8 KV is the recommended 12GB profile per report Section "Best recommendation".
- E-26 (SGLang adapter): orthogonal but inherits the FA2 invariant when ELAI runs SGLang on Ampere.
- E-2 (LogitsProcessor wiring): E-28 touches `from_pretrained`/attn_implementation + attention mask wrapping; E-2 touches `model.generate` kwargs. Serialize `server.py` edit windows.

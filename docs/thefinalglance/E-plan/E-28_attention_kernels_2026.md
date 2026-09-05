# E-28: Attention kernels 2026 (FA2 mandatory, StreamingLLM sinks, YaRN cautious)
Objective: codify the report's "keep FA2 on Ampere, do not adopt FA3/FA4 locally" call as enforced ELAI policy; add StreamingLLM attention-sink masking for chats >25K context on a single 3090; bump flash-attn to 2.8.3 (Jan 2026); evaluate YaRN context-extension under controlled rollout. NO Hopper-only kernels. NO MLA architectural swap.
Estimated effort: 1-2 sessions (~6-10 h)
Pre-requisites: none. Touches `python/elai_inference/server.py` and a new context-extension toggle file. Sidecar boot is HARDWARE-HEAVY (CLAUDE.md hard rule) — coordinate with E-2 and E-27 for shared `server.py` edit windows.
Parallel-safe with: E-26 (adapter trait; orthogonal), E-27 (weight quant; orthogonal — FA2 is the attention kernel, AWQ-Marlin is the weight kernel), E-2 (E-2 touches `model.generate` kwargs + LogitsProcessor wiring; E-28 touches `from_pretrained`/attn_implementation + attention mask wrapping; serialize the `server.py` edits).
Source report: workflow/improvement-sweep-2026-05-19/reports/28-attention-kernels-2026.md

## Context

Per report TL;DR: keep FA2 (flash-attn 2.7.4 / 2.8.3) MANDATORY on RTX 3060/3090; SM_80+ still officially supported, no deprecation signaled by Tri Dao through 2026 (github.com/Dao-AILab/flash-attention). FA3 is Hopper-only (H100/H200) and FA4 is Hopper+Blackwell (CuTeDSL); neither runs the optimized path on Ampere consumer GPUs. The "75% faster than FAv2" cuDNN headline number is H100-measured per pytorch#152595 / pytorch 2.5 release notes — citing it for 3090 planning is unsearched conjecture (report Anti-pattern section).

For >25K context on a single 3090, the report's recommended stack (lowest-risk first): (1) FA2 + bf16 baseline; (2) KV-cache quantization fp8_e5m2 — covered in E-27; (3) YaRN RoPE extension (kernel-transparent, still uses FA2) per arXiv:2309.00071 / Qwen2.5-1M arXiv:2501.15383 — Qwen2.5-1M ships factor=4 for 128K; (4) StreamingLLM attention-sink masking per arXiv:2309.17453 — keep first 4 tokens + sliding window, up to 22.2x vs sliding-recompute; (5) MInference dynamic sparse pre-fill at >50K per arXiv:2407.02490 (10x pre-fill on A100; kernels are Triton/CUDA on SM_86 per microsoft/MInference repo) — DEFERRED pending Open Question 2; (6) DuoAttention head classification on GQA decode per arXiv:2410.10819 — DEFERRED pending Open Question 4.

Qwen2.5-Coder-7B is dense GQA (28 Q heads / 4 KV heads, head_dim=128, num_layers=28, rope_theta=1e6, `use_sliding_window=false` per HF config.json) — so head_dim 64/128 power-of-two kernels work. Head_dim 256 mixed-layout issues that affected Gemma-3 are absent (report Section "Anti-patterns"). E-28 codifies this as an ELAI invariant: do not adopt Gemma-family models that would force mixed head_dim.

Determinism note: FA2 forward is deterministic given fixed seed + workspace; FA2 backward uses atomicAdd and is non-deterministic by default (transformers#31787). ELAI sidecar is inference-only today — backward issue is moot. If/when ELAI adds on-device fine-tune or RLHF, mandate `deterministic=True` (~37.9% throughput cost cited in FA3). Codify this as a Rule.

## Existing state (B/C/D coverage)

- ELAI today: `python/elai_inference/server.py:1395-1405` auto-detects FA2 availability with `ELAI_FORCE_SDPA=1` override path (elai.map line 184841 `inference_mode`, line 184949 `flash_attention_2_available`). CLAUDE.md mandates FA2 on Ampere. E-28 BUMPS the pinned version + ADDS StreamingLLM + ADDS YaRN toggle, all while preserving the SDPA-fallback escape.
- `crates/orchestrator/src/research_tier.rs:69-130` carries `quant: "NF4+FA2".to_string()` — the "+FA2" suffix encodes the policy. E-27 changes the front half ("NF4" → "AWQ-Marlin"); E-28 keeps the "+FA2" suffix unchanged.
- `crates/rules/src/schema.rs:174` already has `Some("2023-01"), "BLOCK", "tracing is the modern Rust async logging standard"`. E-28 adds parallel rule rows for FA2 mandatory, FA3/FA4-Hopper-only, SDPA backend pinning.
- No C-plan or D-plan entry references FA2 / FA3 / StreamingLLM / YaRN / MInference / DuoAttention.
- Memory note `memory:project_qwen3b_baseline.md`: measured NF4+FA2 25K tokens 100% pass at 1.92GB. E-28's StreamingLLM addition must not regress this baseline (StreamingLLM is opt-in via env).
- Sidecar boot recipe at `workflow/remote-setup/REMOTE-SIDECAR.md` uses `ELAI_FORCE_SDPA=1` for the SDPA-fallback path. E-28 keeps that env unchanged.

## Reads (mandatory before coding)

- `F:\.ELAI_workflow\workflow\improvement-sweep-2026-05-19\reports\28-attention-kernels-2026.md` (source report, full)
- `F:\.ELAI_workflow\.elai_cc\python\elai_inference\server.py` lines 1395-1420 (FA2 detect + `_attn_impl` selection; elai.map references at lines 184949, 184950, 172855)
- `F:\.ELAI_workflow\.elai_cc\python\elai_inference\pyproject.toml` (current flash-attn pin)
- `F:\.ELAI_workflow\.elai_cc\crates\rules\src\schema.rs:174-210` (rule-row shape)
- `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\research_tier.rs:60-130` (quant+attn string)
- `F:\.ELAI_workflow\workflow\remote-setup\REMOTE-SIDECAR.md` (`ELAI_FORCE_SDPA=1` flag context)
- arXiv:2307.08691 FlashAttention-2 paper (cited)
- arXiv:2309.17453 StreamingLLM paper + github.com/mit-han-lab/streaming-llm (cited)
- arXiv:2501.15383 Qwen2.5-1M tech report (cited; YaRN recipe for Qwen2.5)
- huggingface.co/Qwen/Qwen2.5-Coder-7B-Instruct/blob/main/config.json (head_dim=128, GQA shape — cited)
- transformers#31787 (FA2 backward non-determinism — cited)
- tridao.me/blog/2024/flash3 (FA3 Hopper-only confirmation — cited)
- together.ai/blog/flashattention-4 (FA4 Blackwell + missing GQA/backward at release — cited)

## Tasks

1. **Bump flash-attn pin to 2.8.3 in `python/elai_inference/pyproject.toml` (~5 LOC delta).** Per report Section "What runs on RTX 3060/3090" the pypi `flash-attn 2.8.3` (Jan 2026) supports CUDA 11.8-12.8 / torch 2.4-2.9. Verify via pypi pre-pin (CLAUDE.md anti-slopsquatting): `curl https://pypi.org/pypi/flash-attn/json | jq .info.version,.info.license`. Keep the existing wheel-on-Windows fallback comment intact.

2. **Add StreamingLLM attention-sink masking in `python/elai_inference/server.py` (~150 LOC).** New env `ELAI_STREAMING_LLM_SINKS` ∈ {`0`, `1`, integer-N for sink count}, default `0` (BACKWARD COMPAT). When N>0: wrap `model.generate` attention mask to keep the first N tokens (default 4 per paper) PLUS a sliding window equal to `ELAI_LOCAL_MAX_CONTEXT`. Reference impl follows github.com/mit-han-lab/streaming-llm: pure-Python attention-mask manipulation, no FA2 changes required. Emit `streaming_llm_engaged` SSB row per generation with `{sink_count, window_size, context_position}`. Determinism: forward path is deterministic per report Section "Determinism concerns" — pure masking, no fused-CTA ops.

3. **Add a YaRN context-extension toggle (~100 LOC).** New env `ELAI_YARN_FACTOR` ∈ {`null`, `2.0`, `4.0`, `8.0`}, default null (off). When set: pass `rope_scaling={"type":"yarn","factor":<f>,"original_max_position_embeddings":32768}` to `AutoConfig.from_pretrained` per Qwen2.5-1M arXiv:2501.15383 recipe. Per CLAUDE.md no-removal: do NOT change the default config; YaRN stays opt-in. Per report Open Question 3: validate eval before adopting at factor>=4. Add a guarded `assert config.max_position_embeddings >= 32768` to prevent silent mis-config.

4. **Pin SDPA backend selection (~50 LOC).** Per report Anti-pattern "SDPA without explicit backend pin": when `ELAI_FORCE_SDPA=1`, wrap `model.generate` in `torch.nn.attention.sdpa_kernel([SDPBackend.FLASH_ATTENTION])` to prevent silent MATH fallback on shape errors (pytorch#133780 cited). If FLASH_ATTENTION SDPA backend unavailable on this hardware, raise a clear error rather than silent fallback. Emit `sdpa_backend_pinned` SSB row at boot with the selected backend.

5. **Add 4 Rules entries in `crates/rules/src/schema.rs:174-210` (~50 LOC).** Per report Anti-pattern section + Best-techniques table:
   - `Some("2026-05"), "BLOCK", "FA3 / FA4 kernels are Hopper / Blackwell only; do not enable on Ampere (RTX 3060/3090). Reference: tridao.me/blog/2024/flash3, together.ai/blog/flashattention-4"`
   - `Some("2026-05"), "BLOCK", "MLA architecture swap requires retraining; Qwen2.5-Coder weights are GQA. Do not pursue as a kernel-only optimization. Reference: DeepWiki MLA + report Section Anti-patterns"`
   - `Some("2026-05"), "WARN", "Do not adopt Gemma-family models with mixed head_dim (256+512 layers); Qwen2.5 dense head_dim=128 sidesteps this. Reference: report Section Anti-patterns + HF Qwen2.5-Coder-7B config.json"`
   - `Some("2026-05"), "WARN", "If ELAI adds on-device fine-tune/RLHF in future, FA2 backward must set deterministic=True (~37.9% throughput cost). Reference: transformers#31787"`
   - `Some("2026-05"), "BLOCK", "SDPA without explicit backend pin: torch.nn.functional.scaled_dot_product_attention silently falls back to MATH on shape errors. Use torch.nn.attention.sdpa_kernel([SDPBackend.FLASH_ATTENTION]). Reference: pytorch#133780"`

6. **Add SSB action types (~30 LOC in `crates/orchestrator/src/ssb.rs`).** `streaming_llm_engaged`, `sdpa_backend_pinned`, `yarn_rope_scaling_applied` (payload `{factor, original_max_position_embeddings, applied_at_layer:int}`).

7. **Add the >25K-context recipe runbook section to `workflow/remote-setup/REMOTE-SIDECAR.md` (~40 LOC delta).** Document the full stacked recipe per report Section "Long-context attention extension recipe": (1) FA2+bf16 baseline (already mandatory), (2) FP8 KV from E-27, (3) YaRN factor=4 for 128K target (validate first), (4) StreamingLLM sinks=4 for unbounded chat. Mark MInference + DuoAttention + RingAttention as DEFERRED with the specific gate condition (Open Question 2 measurement for MInference, calibration cost for DuoAttention, >=2 GPUs for RingAttention).

8. **StreamingLLM parity test harness (~120 LOC at `python/elai_inference/test_streaming_llm_sinks.py`).** Run a fixed 30K-context prompt under (a) FA2 baseline (no streaming), (b) FA2 + StreamingLLM sinks=4. Assert: (i) output quality on a code-completion fixture is parity, (ii) VRAM footprint with streaming is <= baseline, (iii) throughput is >= baseline (streaming is a memory win, not speed loss). Determinism check: 3 consecutive runs byte-equivalent.

9. **YaRN eval-gate harness (~150 LOC at `python/elai_inference/test_yarn_eval_gate.py`).** Per report Open Question 3: sweep factor∈{2.0, 4.0, 8.0} on Qwen2.5-Coder-7B at context positions {8K, 25K, 64K, 128K}. Run a needle-in-haystack fixture + a code-edit fixture. Output CSV at `.elai_cc/metrics/yarn_eval_factor_sweep.csv`. Block default-on YaRN until human review confirms acceptable quality at the chosen factor.

10. **Write decision doc at `workflow/reasoning/E-28-decisions.md`** covering: (a) FA2-only on Ampere (FA3/FA4 deferred to cloud Hopper/Blackwell tier — not in scope), (b) StreamingLLM env-gated until parity passes 3 runs, (c) YaRN factor selection deferred to human review of Task 9 output, (d) MInference / DuoAttention / RingAttention rationale for deferral, (e) explicit OUT-OF-SCOPE: MLA, Gemma-family head_dim mismatch, cuDNN SDPA backend (Hopper-leaning).

## WIRING_CONTRACT

produces_symbols:
  - python/elai_inference/server.py::apply_streaming_llm_mask (new helper)
  - python/elai_inference/server.py::pin_sdpa_backend (new helper)
  - python/elai_inference/server.py::apply_yarn_rope_scaling (new helper)
  - python/elai_inference/test_streaming_llm_sinks.py
  - python/elai_inference/test_yarn_eval_gate.py
  - SsbActionType::{StreamingLlmEngaged, SdpaBackendPinned, YarnRopeScalingApplied}
  - 5 new rule rows in crates/rules/src/schema.rs
consumes_symbols:
  - python/elai_inference/server.py::InferenceEngine (existing class; _attn_impl path)
  - python/elai_inference/pyproject.toml (flash-attn version bump)
reachable_from:
  - User exports `ELAI_STREAMING_LLM_SINKS=4` and submits a 30K-context prompt → StreamingLLM mask applied → streaming_llm_engaged SSB row emitted
  - User exports `ELAI_YARN_FACTOR=4.0` and boots sidecar → AutoConfig rope_scaling patched → yarn_rope_scaling_applied SSB row emitted
  - Default boot path (no env overrides) is BYTE-IDENTICAL to today's behaviour
stub_ban:
  - "TODO: enable FA3" literal absent (FA3 BLOCKED on Ampere per rule)
  - "from flash_attn import .*FlashAttention3" import absent
  - "from cudnn_attention" import absent
  - No silent SDPA→MATH fallback path (explicit error per Task 4)
  - No MLA / DeepSeek-V3 weight architecture swap path
frontend_pair: null (sidecar-only; no dashboard component required)
observable_effect:
  trigger: "ELAI_STREAMING_LLM_SINKS=4 elai.bat repl and submit a 30K-token code-edit prompt"
  expect_ssb: "streaming_llm_engaged with payload {sink_count:4, window_size:25000, context_position:30000}; sdpa_backend_pinned at boot"
  expect_artifact: "Generation completes inside the existing baseline VRAM footprint; no FA3/FA4 import attempted; SDPA backend pinned to FLASH_ATTENTION when ELAI_FORCE_SDPA=1"

## DETERMINISTIC_CHECK

```bash
# 1. flash-attn version pin bumped
rg "flash[-_]attn.*2\.8\.3" .elai_cc/python/elai_inference/pyproject.toml

# 2. StreamingLLM mask + YaRN + SDPA pin helpers present
rg "ELAI_STREAMING_LLM_SINKS" .elai_cc/python/elai_inference/server.py
rg "ELAI_YARN_FACTOR" .elai_cc/python/elai_inference/server.py
rg "torch\.nn\.attention\.sdpa_kernel" .elai_cc/python/elai_inference/server.py

# 3. Rules entries for FA3/FA4/MLA/Gemma/SDPA-pin
rg "FA3 / FA4 kernels are Hopper / Blackwell|MLA architecture swap|Gemma-family models with mixed head_dim|SDPA without explicit backend pin|FA2 backward must set deterministic" .elai_cc/crates/rules/src/schema.rs

# 4. SSB action types wired
rg "streaming_llm_engaged|sdpa_backend_pinned|yarn_rope_scaling_applied" .elai_cc/crates/orchestrator/src/ssb.rs

# 5. Parity + YaRN gate tests
cd .elai_cc/python && python -m elai_inference.test_streaming_llm_sinks
cd .elai_cc/python && python -m elai_inference.test_yarn_eval_gate

# 6. NO FA3/FA4 imports present
! rg "FlashAttention3|FlashAttention4|flash_attn_3|flash_attn_4" .elai_cc/python/

# 7. Decision doc exists
test -f workflow/reasoning/E-28-decisions.md
```

## ACCEPTANCE

- [ ] flash-attn pinned to 2.8.3 (or later 2.8.x); `pip install` succeeds on Windows + Linux remote 3090
- [ ] `ELAI_STREAMING_LLM_SINKS=4` enables the mask; default off is byte-equivalent to today's baseline
- [ ] `ELAI_FORCE_SDPA=1` pins SDPA backend to FLASH_ATTENTION explicitly; no silent MATH fallback on shape errors
- [ ] `ELAI_YARN_FACTOR=4.0` applies rope_scaling at config load; emits SSB row
- [ ] StreamingLLM parity test passes at 30K context (output quality parity, VRAM <= baseline)
- [ ] YaRN factor-sweep CSV produced at `.elai_cc/metrics/yarn_eval_factor_sweep.csv`
- [ ] No FA3/FA4 import attempted anywhere in `python/elai_inference/`
- [ ] No MLA / DeepSeek-V3 weight-architecture code path introduced
- [ ] All 3 new SSB action types round-trip via SSB roundtrip contract
- [ ] CLAUDE.md FA2 mandatory rule remains enforced (no regression to pure SDPA default)
- [ ] `cargo check --workspace` GREEN; remote 3090 sidecar boot test PASS

## Tests to add

- `python/elai_inference/test_streaming_llm_sinks.py`: parity + VRAM at 30K context (Task 8).
- `python/elai_inference/test_yarn_eval_gate.py`: factor sweep 2/4/8 across 4 context positions (Task 9).
- `python/elai_inference/test_sdpa_backend_pin.py`: assert `sdpa_kernel([FLASH_ATTENTION])` invocation when `ELAI_FORCE_SDPA=1`; explicit error on shape-mismatch instead of MATH fallback.
- `python/elai_inference/test_fa2_no_fa3_imports.py`: lint asserting no FA3/FA4 imports anywhere under `python/elai_inference/`.
- `crates/contracts/tests/attention_kernel_ssb_action_types_contract.rs`: SSB roundtrip for the 3 new action_types.

## Files touched

- CREATE: `python/elai_inference/test_streaming_llm_sinks.py` (~120 LOC)
- CREATE: `python/elai_inference/test_yarn_eval_gate.py` (~150 LOC)
- CREATE: `python/elai_inference/test_sdpa_backend_pin.py` (~80 LOC)
- CREATE: `python/elai_inference/test_fa2_no_fa3_imports.py` (~40 LOC)
- CREATE: `crates/contracts/tests/attention_kernel_ssb_action_types_contract.rs` (~70 LOC)
- CREATE: `workflow/reasoning/E-28-decisions.md` (~80 LOC)
- MODIFY: `python/elai_inference/server.py` (~250 LOC: StreamingLLM mask + YaRN config + SDPA pin + SSB emit)
- MODIFY: `python/elai_inference/pyproject.toml` (~5 LOC: flash-attn 2.8.3 pin)
- MODIFY: `crates/rules/src/schema.rs:174-210` (~50 LOC: 5 new rule rows)
- MODIFY: `crates/orchestrator/src/ssb.rs` (~30 LOC: 3 new action_types)
- MODIFY: `workflow/remote-setup/REMOTE-SIDECAR.md` (~40 LOC: long-context recipe)

## Citations / Grounding

- arXiv:2307.08691 — FlashAttention-2 paper (cited; baseline)
- arXiv:2407.08608 — FlashAttention-3 paper (cited; Hopper-only confirmation)
- arXiv:2309.17453 — StreamingLLM ICLR 2024 (cited; primary for Task 2)
- arXiv:2309.00071 — YaRN (cited; primary for Task 3)
- arXiv:2501.15383 — Qwen2.5-1M tech report (cited; YaRN recipe for Qwen2.5)
- arXiv:2407.02490 — MInference NeurIPS 2024 (cited; DEFERRED pending Open Question 2)
- arXiv:2410.10819 — DuoAttention ICLR 2025 (cited; DEFERRED pending Open Question 4)
- arXiv:2310.01889 — Ring Attention (cited; SKIP — single 3090)
- arXiv:2601.21824 — DASH (cited; FA backward determinism)
- tridao.me/blog/2024/flash3 — Tri Dao FA3 blog (cited; FA2 still recommended on Ampere)
- together.ai/blog/flashattention-4 — FA4 release (cited; lacks backward/GQA/varlen at release)
- github.com/Dao-AILab/flash-attention — SM_80 in supported list, no deprecation 2026 (cited)
- github.com/huggingface/transformers/issues/31787 — FA2 backward non-determinism (cited)
- github.com/QwenLM/Qwen3.6/issues/104 — FA2 + Qwen3 3D position_ids bug (cited; non-issue while on Qwen2.5-Coder text-only)
- github.com/pytorch/pytorch/issues/133780 — SDPA silent MATH fallback (cited)
- HF Qwen2.5-Coder-7B config.json — head_dim=128 GQA (cited)

## Open questions / risks

- Report Open Question 1: measured FA2 forward throughput on RTX 3090 at 25K ctx with Qwen2.5-Coder-7B vs xFormers/FlexAttention is unmeasured under flash-attn 2.8.3 (the version bump in Task 1). Mitigation: baseline test before promotion of any default.
- Report Open Question 2: MInference Vertical-Slash kernel on SM_86 with 10x pre-fill claim — A100-only in practice? Needs microsoft/MInference reproduction at 50K-100K ctx. E-28 DEFERS MInference; reach the call after measurement.
- Report Open Question 3: YaRN factor sweet spot for Qwen2.5-Coder-7B (Qwen2.5-1M ships factor=4 for 128K). Task 9 produces the data; do not enable default-on until human review.
- Report Open Question 4: DuoAttention retrieval-head calibration transfer between 3B and 7B is unknown. DEFERRED.
- Report Open Question 5: FlexAttention compile-time overhead on first-token latency on Ampere — DEFERRED (research-only path).
- Ungrounded conjecture flagged in report: cuDNN SDPA "75% faster than FAv2" headline is H100-measured. E-28 BLOCKS that as a rule (no Ampere-applicable claim).
- Anti-Stale-Training: do not assume `pip install flash-attn==2.8.3` works without verifying CUDA+torch matrix in CI; per CLAUDE.md anti-slopsquatting, verify pypi at pin time.
- Sidecar boot is HARDWARE-HEAVY (CLAUDE.md hard rule). All `server.py` changes must be tested against remote 3090, not a second local instance.

## Next

E-27 (quantization) composes: AWQ-Marlin weights + FA2 attention + FP8 KV is the recommended 12GB profile per report Section "Best recommendation". E-26 (SGLang adapter) is orthogonal but inherits the FA2 invariant when ELAI runs SGLang on Ampere. Future E-N: MInference adoption gated on Open Question 2 measurement; DuoAttention gated on Open Question 4 calibration-transfer measurement; RingAttention gated on >=2 GPU hardware availability.

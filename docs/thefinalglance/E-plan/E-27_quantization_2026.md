# E-27: Quantization 2026 (AWQ+Marlin for serving, FP8 KV cache, NF4 for QLoRA only)
Objective: switch the custom sidecar's serving path from BnB-NF4+FP16 to AWQ+Marlin (W4A16) for 4.4x throughput at identical HumanEval, enable FP8 KV cache for ~2x context budget on Ada+ (graceful Ampere fallback), and keep BnB-NF4 only for QLoRA fine-tuning. This is the single highest-ROI inference change on the 2026-05-19 sweep.
Estimated effort: 2 sessions (~8-12 h)
Pre-requisites: B-37 (RuntimeAdapter, SHIPPED — quant choice is sidecar-internal, transparent to RuntimeAdapter). Optionally E-26 (SGLang adapter benefits from AWQ models too). No hard ordering.
Parallel-safe with: E-26 (E-26 = adapter trait + XGrammar; E-27 = sidecar weight format + KV dtype, orthogonal). E-28 (attention kernels: FA2 stays mandatory on Ampere; AWQ-Marlin is weight kernel, FA2 is attention kernel, no overlap). E-2 (E-2 prompt-lookup + StaticCache are model.generate kwargs; E-27 changes weight loading dtype; coordinate `server.py` edit windows).
Source report: workflow/improvement-sweep-2026-05-19/reports/27-quantization-2026.md

## Context

Per report TL;DR: ELAI is on bnb-NF4 + FP16 compute. On Qwen2.5-32B Wikitext-2 PPL, BnB actually wins quality (6.67 vs FP16 6.56) BUT throughput is 168 tok/s vs Marlin-AWQ's 741 tok/s — a 4.4x speed gap (Jarvislabs, 2026). HumanEval pass@1 is the same as BnB (51.8% vs 51.8%) at AWQ. NF4 is best for fine-tuning; not for serving.

Long-context tax: at >64K context, BNB-NF4 can drop up to 59% on long-context tasks (arXiv:2505.20276). Qwen2.5-72B is robust under NF4, but smaller Qwens (the 7B/3B that ELAI runs) are not — ELAI's 7B at 25K is still inside the safer window, but pushing to 40K with NF4 is risky.

KV cache: FP8 KV is the production default in vLLM 2026 (~2x context for free, no throughput loss) per docs.vllm.ai/fp8. On Ampere (RTX 3090, no native FP8), software-FP8-sim still wins on memory while costing a few % throughput. Report Section "KV-cache quantization" recommends FP8 KV over every TurboQuant variant (vllm.ai/blog/turboquant — TurboQuant 20-34% slower than FP8 AND k3v4-nc drops ~20 reasoning points). E-27 DEFERS TurboQuant (matches CLAUDE.md TurboQuant DEFER stance).

Reasoning safety (arXiv:2504.04823 COLM 2025): W4A16 and W8A8 are the lossless ceilings — sub-W4 or aggressive activation quantization introduces real reasoning loss. ELAI's 7B + reasoning workload should stay >= W4A16. AWQ is W4A16. This is the single grounded constraint that pins E-27's choice.

Migration order per report: (1) FP8 KV for context budget, (2) AWQ + Marlin via AutoAWQ (~2h on consumer GPU) for throughput, (3) keep NF4 only on a separate `--qlora-checkpoint` path.

## Existing state (B/C/D coverage)

- ELAI today: `python/elai_inference/server.py` constructs `BitsAndBytesConfig(load_in_4bit=True, bnb_4bit_use_double_quant=True, ...)` at line 1414 (elai.map SCIP_REF). The `_attn_impl` selection at line 1410 already supports FA2 / SDPA. E-27 ADDS a parallel weight-load branch for AWQ.
- `crates/orchestrator/src/context_budget.rs:303` documents the measured baseline: "Qwen2.5-Coder-7B NF4 on 12GB (measured configuration)". E-27 ADDS a new profile entry: Qwen2.5-Coder-7B AWQ+Marlin+FP8KV.
- `crates/rules/src/schema.rs:201` already carries the rule: "bfloat16 is preferred compute dtype for NF4 on Ampere+ GPUs". E-27 ADDS a parallel rule: "AWQ+Marlin is preferred serving quant on Ampere+ for >=7B Qwen-Coder; NF4 reserved for QLoRA fine-tune".
- `crates/orchestrator/src/research_tier.rs:69-130` carries `quant: "NF4+FA2".to_string()` strings across multiple research-tier configs. E-27 introduces a new tier string `"AWQ-Marlin+FA2+FP8KV"` and updates the >=12GB profiles to use it.
- B-23 self-mod TQ4 (CLAUDE.md context) and B-37 RuntimeConfig do NOT block this. TurboQuant DEFER remains the call (report Section "TurboQuant ADOPT/DEFER call" — vLLM's own May 2026 study says don't ship TurboQuant for KV cache in prod).
- No C-plan or D-plan entry references AWQ, Marlin, FP8 KV cache, or AutoAWQ.
- The CLAUDE.md "no Gemma 4" note + memory:project_qwen3b_baseline.md (NF4+FA2: 25K tokens 100% pass, 1.92GB model) is the baseline E-27 must not regress on. Verify: AWQ-Marlin Qwen2.5-Coder-3B at 25K context still passes the same fixture.

## Reads (mandatory before coding)

- `F:\.ELAI_workflow\workflow\improvement-sweep-2026-05-19\reports\27-quantization-2026.md` (source report, full)
- `F:\.ELAI_workflow\.elai_cc\python\elai_inference\server.py` lines around 1167-1420 (InferenceEngine.__init__ — BitsAndBytesConfig site, attn detection, model load); locate via SCIP refs `_generate_impl` (line 1410) and `__init__` (line 1167).
- `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\research_tier.rs:60-130` (quant string in ResearchTierConfig)
- `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\context_budget.rs:295-330` (ModelCapabilityProfile with model_vram_mb)
- `F:\.ELAI_workflow\.elai_cc\crates\rules\src\schema.rs:195-210` (bf16+NF4 rule — shape for the new AWQ rule)
- `F:\.ELAI_workflow\workflow\remote-setup\REMOTE-SIDECAR.md` (boot recipe — will need an alternate `--load-in-awq` flag)
- AutoAWQ docs (online, do not load): github.com/casper-hansen/AutoAWQ README + huggingface.co/blog/marlin
- HuggingFace `quantized_kvcache` doc + `docs.vllm.ai/fp8` (FP8 KV details; cited in report)
- arXiv:2408.11743 Marlin paper (cited)
- arXiv:2504.04823 COLM 2025 (cited; the W4A16 lossless-ceiling anchor)

## Tasks

1. **Add an AWQ+Marlin weight-load branch in `python/elai_inference/server.py` (~200 LOC).** New env `ELAI_QUANT_BACKEND` ∈ {`nf4`, `awq`, `fp16`}, default `nf4` (BACKWARD COMPAT). When `awq`: use `transformers.AutoModelForCausalLM.from_pretrained(model, quantization_config=AwqConfig(version="marlin", ...))` on Ampere+ (compute_capability >= 8.0). Falls back to `version="gemm"` on pre-Ampere (RTX 20-series). New CLI flag `--quant-backend awq` plumbs to env. Per report rule: NF4 path is preserved for the QLoRA fine-tune route; do NOT delete it.

2. **Add FP8 KV cache toggle (~120 LOC in `python/elai_inference/server.py`).** New env `ELAI_KV_CACHE_DTYPE` ∈ {`fp16`, `fp8_e4m3`, `fp8_e5m2`}, default `fp16`. When `fp8_*`: configure HF `cache_implementation` + `cache_config` per docs.vllm.ai/fp8 conventions. On Ampere (no native FP8 hardware), use software-sim path; emit `kv_cache_dtype_software_sim` SSB row noting throughput cost (~5% per report Section "KV-cache quantization"). On Ada+ (4090, RTX 5090 future), use native FP8 path.

3. **Update ModelCapabilityProfile entries in `crates/orchestrator/src/context_budget.rs:303` (~30 LOC delta).** ADD new entries for the 5 VRAM tiers per report "Best recommendation for ELAI per VRAM tier" table: 8GB → Qwen2.5-Coder-3B AWQ-Marlin (3B too small for 4-bit reasoning safety per arXiv:2504.04823; use GGUF Q5_K_M as fallback when AWQ unavailable); 12GB → Qwen2.5-Coder-7B AWQ-Marlin + FP8 KV; 24GB → Qwen2.5-Coder-14B AWQ-Marlin OR EXL3 (deferred); 48GB → Qwen2.5-Coder-32B FP8 W8A8 on Ada+, AWQ+FP8KV on Ampere; 48GB+B200 → NVFP4+QAD (FUTURE). Each profile carries a new `quant_format` field.

4. **Update ResearchTierConfig quant strings in `crates/orchestrator/src/research_tier.rs:69-130` (~40 LOC delta).** Replace "NF4+FA2" with role-conditional string: keep "NF4+FA2" for the dedicated QLoRA-eligible tier; new "AWQ-Marlin+FA2+FP8KV" for the >=12GB serving tier. The 4 affected lines (69, 84, 99, 114, 130 per elai.map SCIP refs) get a new arm based on the tier's intended-use enum.

5. **Add Rules entries in `crates/rules/src/schema.rs:174-210` (~40 LOC).** New rule rows per CLAUDE.md anti-stale audit pattern:
   - `Some("2026-05"), "BLOCK", "AWQ without Marlin/Machete backend kernel leaves throughput at 68 tok/s; enable Marlin (Ampere+) or Machete (Hopper+) per Jarvislabs 2026 benchmark"`
   - `Some("2026-05"), "WARN", "Pushing NF4 to >40K context on Qwen2.5-Coder-7B risks 59% long-context degradation per arXiv:2505.20276; switch to FP8 KV cache to unlock 50K+ on Ada+"`
   - `Some("2026-05"), "BLOCK", "W4A8 / sub-W4 weights cause reasoning cliff per arXiv:2504.04823 COLM 2025; ELAI minimum is W4A16"`
   - `Some("2026-05"), "WARN", "GGUF Q4_K_M for code: moderate quality loss per willitrunai; use Q5_K_M / Q6_K for code workloads"`
   - `Some("2026-05"), "BLOCK", "TurboQuant variants <4-bit for KV cache underperform FP8 by 20-34% throughput AND drop ~20 reasoning points per vllm.ai/blog/turboquant; use FP8 KV instead"`

6. **AutoAWQ build recipe at `python/elai_inference/scripts/build_awq.py` (~150 LOC).** Standalone script that takes a model checkpoint path + calibration corpus (default: a 256-line ELAI codebase sample for code-aware calibration) and produces an AWQ-quantized checkpoint. ~2h on consumer GPU per report. Per CLAUDE.md anti-slopsquatting: `autoawq` package pypi-verified before `pyproject.toml` add. Output checkpoint goes under `F:\.ELAI_workflow\.elai_cc\models\Qwen2.5-Coder-7B-AWQ\` (new dir; both the existing 7B abliterated NF4 path and the new AWQ path coexist).

7. **Boot recipe update in `workflow/remote-setup/REMOTE-SIDECAR.md` (~40 LOC delta).** Add the alternate boot line with `ELAI_QUANT_BACKEND=awq ELAI_KV_CACHE_DTYPE=fp8_e5m2 python server.py --model ~/elai_inference/models/Qwen2.5-Coder-7B-AWQ ...`. Keep the existing NF4 boot line as the default for backward compat.

8. **Determinism + parity test harness for AWQ-Marlin (~180 LOC) at `python/elai_inference/test_awq_marlin_parity.py`.** Run 5 fixed prompts (code-edit, refactor, summarize, scaffold, planner-JSON) under both NF4 and AWQ-Marlin at temp=0. Assert HumanEval pass@1 parity AND output structural equivalence (token-level cosine similarity > 0.95 — not byte-equality, since the kernel paths differ). If either fails, the test FAILS and AWQ rollout stays env-gated off.

9. **Long-context regression guard (~120 LOC at `python/elai_inference/test_long_context_regression.py`).** Per report Open Question 2 (Qwen2.5-Coder-7B long-context PPL curve under AWQ vs NF4 from 8K to 25K): run a fixed 25K-token prompt under NF4 and AWQ-Marlin, assert output quality on a code-completion fixture is parity. Sweep 8K, 16K, 25K. Output a CSV at `.elai_cc/metrics/awq_vs_nf4_longctx.csv` for human review.

10. **Telemetry-only first-week rollout (~60 LOC).** Per report migration order (1) FP8 KV first (cheap), (2) AWQ next (medium). E-27 ships both behind env-gates default-off. After Tasks 8+9 pass for 3 consecutive runs on the remote 3090 (per CLAUDE.md sidecar protocol), flip the 12GB+ tier defaults to AWQ-Marlin in a follow-up patch (NOT this E-N).

11. **Write decision doc at `workflow/reasoning/E-27-decisions.md`** covering: (a) why W4A16 (AWQ) not W4A8 (COLM 2025 cliff), (b) why FP8 KV not TurboQuant (vLLM May 2026 measurement), (c) Marlin not GPTQ-Marlin (HumanEval 51.8 vs 46.3 per Jarvislabs), (d) NF4-QLoRA-only carve-out for fine-tune (not for serving), (e) explicit OUT-OF-SCOPE: EXL3 14B (deferred to 24GB tier work), NVFP4/MXFP4 (B200/RTX 5090 hardware not in scope), TurboQuant for KV (rejected per vLLM blog).

## WIRING_CONTRACT

produces_symbols:
  - python/elai_inference/server.py::AwqMarlinLoader (new branch in InferenceEngine.__init__)
  - python/elai_inference/server.py::Fp8KvCacheConfig (new branch)
  - python/elai_inference/scripts/build_awq.py::main
  - python/elai_inference/test_awq_marlin_parity.py
  - python/elai_inference/test_long_context_regression.py
  - SsbActionType::{QuantBackendSelected, KvCacheDtypeSoftwareSim, AwqCheckpointBuilt}
  - 5 new rule rows in crates/rules/src/schema.rs
consumes_symbols:
  - crates/orchestrator/src/context_budget.rs::ModelCapabilityProfile (new entries)
  - crates/orchestrator/src/research_tier.rs::ResearchTierConfig::quant (new strings)
  - python/elai_inference/server.py::InferenceEngine (existing class; modified __init__)
reachable_from:
  - User exports `ELAI_QUANT_BACKEND=awq ELAI_KV_CACHE_DTYPE=fp8_e5m2` and boots sidecar → AWQ-Marlin weights load → FP8 KV cache allocated → SSB row emitted
  - User runs `python scripts/build_awq.py --model <path>` → AWQ-quantized checkpoint produced under models/Qwen2.5-Coder-7B-AWQ/
stub_ban:
  - "TODO: enable Marlin" literal absent (Marlin enabled at construct time when AWQ)
  - "# fall back to NF4" silent fallback without SSB row absent
  - No EXL3 import (deferred per Open Question 3)
  - No TurboQuant <4-bit KV cache path (rejected per vLLM measurement)
frontend_pair: null (sidecar-only; dashboard surfaces existing model-info via existing RuntimePanel)
observable_effect:
  trigger: "ELAI_QUANT_BACKEND=awq ELAI_KV_CACHE_DTYPE=fp8_e5m2 boot sidecar with Qwen2.5-Coder-7B-AWQ"
  expect_ssb: "quant_backend_selected with payload {backend:'awq', kv_dtype:'fp8_e5m2', compute_capability:8.6, marlin_enabled:true}"
  expect_artifact: "Sustained 25K-context generation passes the existing baseline fixture; tok/s >= 3x the NF4 baseline on the same prompt set; VRAM footprint <= 2.0GB for 3B AWQ / <= 4.5GB for 7B AWQ at FP8KV"

## DETERMINISTIC_CHECK

```bash
# 1. AWQ + FP8 KV branches present in server.py
rg "ELAI_QUANT_BACKEND" .elai_cc/python/elai_inference/server.py
rg "ELAI_KV_CACHE_DTYPE" .elai_cc/python/elai_inference/server.py
rg "AwqConfig|version=.marlin." .elai_cc/python/elai_inference/server.py

# 2. AutoAWQ build script exists
test -f .elai_cc/python/elai_inference/scripts/build_awq.py

# 3. ModelCapabilityProfile new entries
rg "AWQ-Marlin|fp8_e5m2|fp8_e4m3" .elai_cc/crates/orchestrator/src/context_budget.rs

# 4. ResearchTierConfig quant strings updated
rg "AWQ-Marlin\+FA2\+FP8KV" .elai_cc/crates/orchestrator/src/research_tier.rs

# 5. Rules entries
rg "Marlin/Machete backend kernel|W4A16|TurboQuant variants" .elai_cc/crates/rules/src/schema.rs

# 6. New SSB action types
rg "quant_backend_selected|kv_cache_dtype_software_sim|awq_checkpoint_built" .elai_cc/crates/orchestrator/src/ssb.rs

# 7. Parity + long-context tests pass against the existing baseline
cd .elai_cc/python && python -m elai_inference.test_awq_marlin_parity
cd .elai_cc/python && python -m elai_inference.test_long_context_regression

# 8. Decision doc exists
test -f workflow/reasoning/E-27-decisions.md
```

## ACCEPTANCE

- [ ] `ELAI_QUANT_BACKEND=awq` boots a Marlin-backed model load on Ampere (compute_capability >= 8.0); emits `quant_backend_selected` SSB row with `marlin_enabled:true`
- [ ] `ELAI_KV_CACHE_DTYPE=fp8_e5m2` allocates an FP8 KV cache on Ada+ (native) and software-sim on Ampere; emits the corresponding SSB row
- [ ] AWQ-Marlin Qwen2.5-Coder-7B at temp=0 across 5 prompts produces output with token-cosine >= 0.95 vs NF4 baseline (parity test)
- [ ] 25K-context regression test passes parity vs NF4 baseline (long-context guard)
- [ ] HumanEval pass@1 on a held-out fixture >= NF4 baseline (no regression per Jarvislabs 51.8 vs 51.8)
- [ ] Tok/s on RTX 3060 12GB / RTX 3090 24GB measured and recorded in `.elai_cc/metrics/awq_marlin_tok_s.json`
- [ ] NF4 path preserved (legacy boot recipe still works for QLoRA-eligible checkpoints)
- [ ] No EXL3 / NVFP4 / TurboQuant<4bit code introduced (per Open Questions deferral)
- [ ] All 3 new SSB action types round-trip via SSB roundtrip contract
- [ ] `cargo check --workspace` GREEN; python tests pass on remote 3090

## Tests to add

- `python/elai_inference/test_awq_marlin_parity.py`: token-cosine parity across 5 prompts at temp=0.
- `python/elai_inference/test_long_context_regression.py`: 25K-context parity sweep (8K/16K/25K).
- `python/elai_inference/test_fp8_kv_cache.py`: emit `kv_cache_dtype_software_sim` on Ampere; native path on Ada+.
- `crates/contracts/tests/quant_backend_selected_ssb_contract.rs`: SSB roundtrip for the 3 new action_types.
- `crates/orchestrator/tests/context_budget_awq_profiles.rs`: assert new profile entries resolve correctly for 12GB / 24GB / 48GB tiers.

## Files touched

- CREATE: `python/elai_inference/scripts/build_awq.py` (~150 LOC)
- CREATE: `python/elai_inference/test_awq_marlin_parity.py` (~180 LOC)
- CREATE: `python/elai_inference/test_long_context_regression.py` (~120 LOC)
- CREATE: `python/elai_inference/test_fp8_kv_cache.py` (~80 LOC)
- CREATE: `crates/contracts/tests/quant_backend_selected_ssb_contract.rs` (~70 LOC)
- CREATE: `crates/orchestrator/tests/context_budget_awq_profiles.rs` (~90 LOC)
- CREATE: `workflow/reasoning/E-27-decisions.md` (~100 LOC)
- MODIFY: `python/elai_inference/server.py` (~250 LOC: AWQ+Marlin load branch + FP8 KV branch + SSB emit)
- MODIFY: `python/elai_inference/pyproject.toml` (~5 LOC: autoawq dep after pypi verify)
- MODIFY: `crates/orchestrator/src/context_budget.rs` (~30 LOC: profile entries)
- MODIFY: `crates/orchestrator/src/research_tier.rs:69-130` (~40 LOC: quant string arm)
- MODIFY: `crates/rules/src/schema.rs:174-210` (~40 LOC: 5 new rule rows)
- MODIFY: `crates/orchestrator/src/ssb.rs` (~40 LOC: 3 new action_types)
- MODIFY: `workflow/remote-setup/REMOTE-SIDECAR.md` (~40 LOC: alternate boot recipe)

## Citations / Grounding

- arXiv:2504.04823 — COLM 2025, W4A16 / W8A8 lossless ceiling (cited; PRIMARY for the "stay W4A16" call)
- arXiv:2505.20276 — long-context quantization study, 59% drop at >64K (cited; primary for the long-context risk band)
- arXiv:2408.11743 — Marlin paper, 10.9x AWQ speedup (cited; primary for "AWQ without Marlin is wrong")
- arXiv:2504.19874 — TurboQuant (cited; primary for DEFER call)
- jarvislabs.ai/blog/vllm-quantization-complete-guide-benchmarks — Qwen2.5-32B 4-format head-to-head (cited; primary for 168 vs 741 tok/s claim and HumanEval 51.8 parity)
- vllm.ai/blog/turboquant — FP8 beats every TurboQuant variant (cited; the kill-shot on TurboQuant adoption)
- docs.vllm.ai/en/latest/features/quantization/fp8/ — FP8 W8A8 + FP8 KV (cited)
- developers.redhat.com/articles/2024/10/14/introducing-machete-mixed-input-gemm-kernel — Machete (Hopper successor; cited for "Marlin on H100 wants Machete")
- arXiv:2402.02750 — KIVI 2-bit KV (cited; deferred — not picked over FP8 for ELAI's 25K context band)
- willitrunai.com/blog/quantization-guide-gguf-explained — GGUF Q4_K_M moderate loss on code (cited)
- qwen.readthedocs.io/en/v2.5/benchmark/quantization_benchmark.html — Qwen-official quant baselines (cited)

## Open questions / risks

- Report Open Question 1: measured Marlin-AWQ tok/s for Qwen2.5-Coder-7B specifically on RTX 3060 12GB (not 32B on H100) is unknown — Tasks 8+9 produce this number; promotion to default-on gated on it.
- Report Open Question 2: long-context PPL curve from 8K to 25K under AWQ vs NF4 — Task 9 covers; if AWQ regresses at 25K, default for the 12GB profile reverts to NF4 (defensive default).
- Report Open Question 4: FP8 KV impact on Qwen-Coder code-completion specifically (HumanEval, MBPP) — Task 8 partially covers via parity on the 5 prompts; full benchmark deferred to a follow-up patch.
- Anti-Stale-Training risk: `autoawq` may have bumped major versions between training cutoff and 2026-05; pypi verify mandatory per CLAUDE.md before dep add.
- Hardware constraint: ELAI's primary 12GB tier is Ampere SM_86 (no native FP8). Software-sim FP8 KV is approximate; report Section "KV-cache quantization" cites Ada+ for native. Mitigation: env-gate FP8 KV off by default on Ampere until Task 9 confirms parity.
- Ungrounded conjecture flagged in report: EXL3 70B "~200 t/s" on Blackwell is author claim, not third-party measured. E-27 does NOT adopt EXL3.
- TurboQuant retain-for-embeddings carve-out: report Section "TurboQuant ADOPT/DEFER" notes TurboQuant remains relevant for vector-DB embedding compression (different problem from KV cache). E-27 does NOT touch the RAG embedding store; that's a separate future E-N if it surfaces ROI.

## Next

E-26 (SGLang adapter) composes cleanly: AWQ-Marlin models loaded into the custom sidecar today work behind SGLang tomorrow without change. E-28 (attention kernels) keeps FA2 mandatory on Ampere — AWQ-Marlin is the weight kernel, FA2 is the attention kernel, no overlap. Follow-up patches: (1) Promote AWQ default-on for 12GB tier after Tasks 8+9 pass 3 consecutive runs; (2) EXL3 14B-on-24GB path; (3) NVFP4/MXFP4 once ELAI moves to B200/RTX 5090.

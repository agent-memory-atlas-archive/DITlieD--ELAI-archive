# LoRA + domain fine-tuning for ELAI (May 2026)

## TL;DR
- DEFER per-project LoRA fine-tuning. The pipeline (ProjectProfile -> prompt rules + few-shot) already captures style; cost/benefit math for 7B Qwen-Coder LoRAs per-project does not clear the bar in 2026.
- Closest-thing-to-shipping path: keep Tier-2 ProjectProfile + grow CodeRAG-style retrieval for project-local symbols. Anthropic/CodeRAG-Bench 2024-2025 evidence says retrieval gains dominate fine-tuning gains for code-style tasks once the base model is competent.
- If a per-user "voice" LoRA is ever wanted (chat output, not code), QLoRA + PiSSA init + DoRA is the 2026 SOTA stack; ~1-3 h on RTX 3090 for 1K examples on Qwen2.5-Coder-7B. Plan around 5-19 pp catastrophic-forgetting cost on out-of-domain code (OPLoRA, LaLoRA evidence).
- Hot-swap is solved: vLLM multi-LoRA and Unsloth hot-swap guides (2025) are production-ready. So even a deferred plan can ship later without re-architecting the sidecar.
- Hybrid is the dominant 2025-2026 production pattern: fine-tune for behavior/voice, RAG for project facts. ELAI is already 100% on the RAG/context side; adding fine-tune is incremental, not foundational.

## Key sources
URL | what | takeaway
- https://arxiv.org/abs/2305.14314 | QLoRA (Dettmers) | baseline 4-bit NF4 + LoRA; still the reference in 2025-2026
- https://arxiv.org/pdf/2404.02948 | PiSSA (NeurIPS 2024 spotlight) | beats QLoRA 49.13 vs 39.8 on GSM8K; cuts 4-bit quant error 19% on Llama-2-7B
- https://developer.nvidia.com/blog/introducing-dora-a-high-performing-alternative-to-lora-for-fine-tuning/ | DoRA (NVIDIA writeup) | +3.7/+1.0 on Llama-7B/13B common-sense; +4.4 on Llama-3-8B over LoRA
- https://arxiv.org/pdf/2312.03732 | rsLoRA | scaling factor stabilises high-rank LoRA; only worth it when rank>=32
- https://arxiv.org/abs/2407.08296 | Q-GaLore (CPAL 2025) | 50% memory vs LoRA/GaLore; +5.19 MMLU vs QLoRA at same memory; INT4 projection
- https://arxiv.org/html/2510.13003v2 | OPLoRA (Oct 2025) | orthogonal projection LoRA; measured on Qwen2.5-7B; "significantly reduces forgetting" across math, code, common-sense
- https://arxiv.org/abs/2406.14497 | CodeRAG-Bench (NAACL 2025 Findings) | 10 retrievers x 10 LMs; retrieval helps code generation, but retriever and generator-context limits are bigger bottlenecks than missing fine-tune
- https://www.anthropic.com/news/contextual-retrieval (referenced via dev.to/umesh_malik/rag-vs-fine-tuning-for-llms-2026) | Contextual Retrieval | 49% reduction in failed retrievals, 67% with reranking; <200K-token KBs: full-context + prompt caching beats retrieval infra
- https://unsloth.ai/blog/qwen-coder | Unsloth Qwen2.5-Coder fine-tune | full FT of Qwen2.5-7B in 51 min on 1 GPU; 9B LoRA-16bit in 24GB; 6.5GB VRAM at QLoRA-4bit
- https://huggingface.co/blog/anakin87/spectrum | Spectrum SNR layer selection | matches Q-LoRA on GSM8K (+4 pp at 30% layers); shines in DeepSpeed/FSDP not single-GPU
- https://huggingface.co/docs/peft/developer_guides/lora | PEFT current API (2025/2026) | `target_modules="all-linear"`, `use_dora=True`, predefined defaults for Qwen architecture
- https://docs.vllm.ai/en/latest/features/lora/ | vLLM LoRA adapters | hot-swap multiple adapters, `load_inplace` for in-place updates, negligible switch time
- https://arxiv.org/pdf/2512.17910 | Activated LoRA (aLoRA) Dec 2025 | cross-model KV-cache reuse, eliminates re-prefill cost on adapter switch
- https://arxiv.org/pdf/2410.13025 | LoRA Soups (Oct 2024) | merging LoRAs for skill composition; CAT (concat) variant outperforms TIES/DARE for cross-skill compose
- https://huggingface.co/blog/peft_merging | PEFT merging blog | TIES + DARE built into PEFT; recommended path for composing multiple task LoRAs
- https://kaitchup.substack.com/p/loftq-better-initialization-for-a | LoftQ writeup | quantization-aware LoRA init; bridge between QLoRA and PiSSA
- https://arxiv.org/pdf/2405.09673 | "LoRA Learns Less and Forgets Less" (Biderman et al.) | LoRA underperforms full FT on code/math by 5-19 pp BUT forgets less of base capabilities; tradeoff is real not folklore
- https://arxiv.org/pdf/2409.04574 | "Customizing LLM Generation Style using PEFT" (Sep 2024) | direct evidence that PEFT works for style/voice adaptation, not just task knowledge
- https://huggingface.co/huihui-ai/Qwen2.5-Coder-7B-Instruct-abliterated | base model card | HF-ecosystem-standard; LoRA-compatible; no exotic layer layout

## Technique comparison
technique | speed | VRAM (7B) | quality vs LoRA | works on Qwen2.5-Coder | citation
LoRA (vanilla) | 1.0x | ~14 GB | baseline | yes | arxiv 2106.09685
QLoRA (NF4) | ~0.85x | ~6.5 GB | -1 to -3 pp | yes, native | arxiv 2305.14314
LoftQ init | ~0.85x | ~6.5 GB | +1-2 pp over QLoRA | yes | LoftQ paper
PiSSA init | ~0.85x | ~6.5 GB | +9 pp over QLoRA (GSM8K) | yes, drop-in | arxiv 2404.02948
DoRA | ~0.7x | ~7 GB | +1-4 pp common-sense | yes, `use_dora=True` | NVIDIA blog
LoRA+ | ~1.0x | ~14 GB | +0.5-2 pp | yes (LR-asym) | arxiv 2402.12354
rsLoRA | ~1.0x | ~14 GB | +1 pp at rank>=32 | yes, scaling factor | arxiv 2312.03732
GaLore | 1.0x | ~10 GB | matches full FT | yes | arxiv 2403.03507
Q-GaLore | ~0.9x | ~5 GB | +5.19 MMLU vs QLoRA | yes (INT4 proj) | arxiv 2407.08296
Spectrum (30%) | ~0.9x | ~16 GB | +4 pp GSM8K vs QLoRA | yes (SNR per-layer) | HF blog anakin87
Unsloth QLoRA | 2.0x | ~6.5 GB | parity with QLoRA | yes, official Qwen-Coder support | unsloth.ai/blog/qwen-coder
OPLoRA | ~0.95x | ~14 GB | matches LoRA + lower forgetting | YES, measured on Qwen2.5-7B | arxiv 2510.13003

Speed normalized to vanilla LoRA on RTX 3090; VRAM at batch=1 seqlen=2048 gradient-ckpt on Qwen2.5-Coder-7B. Numbers approximate, derived from paper claims + Unsloth profiling.

## RAG vs fine-tune for code tasks
CodeRAG-Bench (NAACL 2025 Findings, arxiv 2406.14497) ran 10 retrievers x 10 LMs across general programming, open-domain, and repository-level tasks. Headline: retrieval helps code generation when context is good, but the bottleneck is the retriever and the generator's ability to use the retrieved context, NOT lack of fine-tuning. RAG models had better compilation rates than fine-tuned models in DSL studies (arxiv 2407.02742) but worse on API parameter hallucination.

Anthropic Contextual Retrieval (https://www.anthropic.com/news/contextual-retrieval) measured 49% reduction in failed retrievals, 67% with reranking. Anthropic's explicit guidance: for KBs under ~200K tokens, full-context prompting + prompt caching beats retrieval infra; reranking gains (15-35% quality) dwarf embedder gains. Cursor's documented architecture is RAG-over-local-FS, not fine-tune. 2025-2026 production stats: ~60% of projects combine RAG + fine-tune; fine-tune for behavior/voice/structure, RAG for facts.

For ELAI's case (per-project style detection at >80% snake_case threshold etc) the style signal is small, observable from a handful of examples, and currently shipped via few-shot in the prompt. This is exactly the "context engineering" cell in Anthropic's matrix, not the fine-tune cell.

## Per-project style adaptation
Realistic effort calculation for ELAI:
- 1K examples per project, QLoRA on Qwen2.5-Coder-7B (RTX 3090 24GB, Unsloth) = ~1-3 h wall-clock per project (Unsloth blog measured 51 min full FT for 7B; LoRA is faster).
- Storage: ~50-200 MB per adapter (rank 16-64).
- vLLM/Unsloth hot-swap is solved (vLLM `load_inplace`, aLoRA cross-model KV reuse 2025); could carry 5-20 project adapters on the 3090 sidecar.
- Catastrophic forgetting: LoRA forgets less than full FT (arxiv 2405.09673) but OPLoRA paper explicitly measures forgetting on Qwen2.5-7B and reports it remains "significant" without their orthogonal-projection fix. Fine-tuning on a Python codebase WILL measurably degrade Rust quality unless OPLoRA-style mitigation is added.
- Reproducibility: base-model patch (e.g. Qwen2.5-Coder-7B -> v0.2) breaks LoRA validity. No published mitigation for "drift" — PortLLM (arxiv 2410.10870) is the closest training-free patch portability work, still nascent.

Who ships it? Stable Diffusion world: yes (Civitai, ComfyUI). Code-LLM world for per-project style: no public production deploy found. Personalization research exists (PersonalLLM ICLR 2025, "Personas within Parameters", HyPerAlign) but treats voice/preference, not coding-style-per-codebase.

Verdict: the user-value is real but small for style-only adaptation when the prompt-rules pipeline is already in place; the user-value is large for task knowledge (project-specific APIs, naming) but that is RAG's lane.

## Recommendation for ELAI
DEFER.

Grounding:
1. ELAI already extracts ProjectProfile and routes to prompt; CodeRAG-Bench shows that's the high-leverage move for code tasks.
2. Anthropic's own 2025 stance (Contextual Retrieval + Context Engineering blog) prioritises context over fine-tune for code agents; Anthropic ships Claude Code without per-project model fine-tunes.
3. RTX 3090 economics work but the per-project storage + adapter-management complexity beats the marginal style-fidelity gain that few-shot already captures.
4. Catastrophic forgetting (5-19 pp on out-of-domain code per Biderman 2024) is a real regression risk for a multi-language tool.
5. Hot-swap infra (vLLM multi-LoRA, aLoRA Dec 2025) means the path stays open: revisit once ELAI has data showing few-shot saturates AND a clear voice/behavior delta the prompt cannot close.

Non-recommendation: if user pushes ahead anyway, SHIP only the per-user "voice" LoRA (not per-project), use Unsloth + QLoRA + PiSSA init + DoRA, and bound forgetting with OPLoRA. That is the smallest reversible bet.

## Best techniques to adopt (if shipping)
Ranked.

1. Unsloth as the trainer | arxiv 2509.12229 RTX 4060 profiling + unsloth.ai/blog/qwen-coder (51 min full FT 7B) | sidecar: add `python/elai_finetune` subpackage that calls Unsloth's `FastLanguageModel.from_pretrained` + `get_peft_model`; ProjectProfile feeds the dataset builder.

2. QLoRA NF4 + PiSSA init | arxiv 2404.02948 (49.13 vs QLoRA 39.8 GSM8K) | PEFT API: `LoraConfig(init_lora_weights="pissa", use_dora=True, ...)`; one-flag change.

3. DoRA enabled | NVIDIA blog (+1-4 pp common-sense), arxiv 2402.09353 | PEFT: `use_dora=True`; cost ~30% slower vs LoRA, accept it for the +pp.

4. OPLoRA for forgetting control | arxiv 2510.13003 | needs custom optimizer wrapping; add as opt-in when training >=2 LoRAs on same base.

5. vLLM multi-adapter serving | docs.vllm.ai/en/latest/features/lora/ + aLoRA 2512.17910 | sidecar: add `--lora-modules` flag; route by ProjectProfile id; KV-cache-aware swap.

6. TIES/DARE merging via PEFT | huggingface.co/blog/peft_merging | only if multi-project compose ever becomes a feature; not v1.

Skip in v1: rsLoRA (only helps at rank>=32, ELAI's data sizes don't warrant), GaLore/Q-GaLore (memory wins only matter for larger models), Spectrum (DeepSpeed/FSDP gain; single-3090 doesn't benefit).

## Anti-patterns
- Fine-tuning Qwen2.5-Coder on a single-language project then expecting cross-language quality to hold. arxiv 2405.09673 measures 5-19 pp drop on out-of-domain code; OPLoRA paper explicitly shows Qwen2.5-7B forgets after standard LoRA.
- Stacking multiple LoRAs at inference without merging strategy. arxiv 2508.11624 (LoRAtorio) shows multi-adapter outputs drift toward base model and lose target-domain fidelity unless explicit composition (TIES/DARE/CAT) is used.
- Using DoRA with the same LR for magnitude and direction. FinLoRA paper (arxiv 2505.19819) measured this and reports DoRA underperforms vanilla LoRA when LRs are not split.
- Trusting LoRA validity across base-model patches. No paper claims this holds; PortLLM is the only attempted solution and is nascent.
- Skipping the Qwen-2.5 pad_token bug (Unsloth docs flag `<|im_start|>`/`<|im_end|>` as untrained special tokens in the base; fine-tune will silently produce garbage if not patched).
- Treating "abliterated" base as identical to stock Qwen-Coder. Layer behavior may have drifted; PEFT defaults still work but quality baseline must be re-measured.
- Running full workspace tests during fine-tune iteration. Spectrum, GaLore, Q-GaLore papers all flag full-workspace eval as the dominant cost; use a small held-out probe set (50-200 examples).

## Open questions
1. What is the measured gap between (current ELAI prompt-rules + few-shot) and (Qwen-Coder-7B with per-project QLoRA-PiSSA-DoRA) on a project-style probe set (snake_case adherence, error-handling shape, module-org match)? Without this number the DEFER decision is grounded in priors, not local measurement.
2. Does Qwen2.5-Coder-7B-Instruct-abliterated still respond to PEFT defaults the same as stock Qwen-Coder-7B? Abliteration modifies the residual stream; LoRA target_modules defaults assume stock topology.
3. On RTX 3090 24GB, what is the actual wall-clock for QLoRA-PiSSA-DoRA training on 1K, 10K, 100K examples of Qwen2.5-Coder-7B at seqlen=2048? Public numbers are extrapolated, not measured for this exact stack.
4. Does OPLoRA's forgetting-mitigation hold when the source domain (multi-lang Coder pretraining) and target domain (single-language project) are as far apart as ELAI's worst case (e.g. Rust-only project)?
5. If ELAI ever ships per-user voice LoRAs, what is the user-perceived quality delta vs. 5-shot prompt? PersonalLLM ICLR 2025 reports >90% win-rates for PEFT vs preference-FT in chat; whether that holds for code-author voice is unstudied.

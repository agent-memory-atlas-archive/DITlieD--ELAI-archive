MoE-on-Ampere tier — design space for expert-offload backends

Status: DESIGN SPACE doc. No backend ships from this document; the actual MoE-on-Ampere expert-offload backend lives in a future B-N/E-N (TBD). This doc records the published approaches + their tradeoffs so the future plan author has a grounded starting point.

Cross-references: `docs/strategy/ELAI-2026-POSTURE.md` Section 2 Bet 7 (no-MoE-local stance, MEDIUM-HIGH risk); `memory:project_vram_tier_plan` (per-VRAM configs at 8/12/16/24/48 GB; second 3090 incoming); B-37 RuntimeAdapter (SHIPPED, arch-blind invariant).


Problem statement

MoE is the de-facto frontier architecture across 2026 frontier models (Mixtral 8x22B, DeepSeek V4-Pro 1.6T params, Gemini 3, GPT-5.5, Llama 4 Scout, Qwen 3.6). All of these models have parameter counts that exceed the VRAM available on Ampere consumer GPUs (RTX 3090 24 GB; 2x3090 = 48 GB).

Expert-offload approaches store the bulk of expert parameters in CPU/system RAM (or disk) and stream the small fraction of experts active per token onto the GPU for the active layer. The goal is to keep average inference latency tractable on consumer hardware that has 5-15x less VRAM than the model would need on-GPU.


Published approaches (every arXiv ID WebSearch-verified May 2026)

(1) MoBiLE (arXiv:2510.12357, Oct 2025) -- "Efficient Mixture-of-Experts Inference on Consumer GPU with Mixture of Big Little Experts". A plug-and-play offloading-based MoE inference framework that reduces the number of experts for unimportant tokens to half for acceleration while maintaining full experts for important tokens, with a dedicated fallback and prefetching mechanism for switching between little and big experts. Reports 1.60x to 1.72x speedup vs baseline on a consumer GPU system with negligible degradation in accuracy. Source: arxiv.org/abs/2510.12357.

(2) Efficient CPU-GPU Collaborative Inference for MoE-based LLMs on Memory-Limited Systems (arXiv:2512.16473, Dec 2025) -- offloads computations to CPU when required experts are not cached on GPU, while experts are asynchronously fetched to GPU for subsequent token generation. Demonstrates throughput surpassing GPU-only computation through synergistic CPU-GPU resource utilization. Source: arxiv.org/abs/2512.16473.

(3) MoEpic (arXiv:2509.08342, Sep 2024) -- Adaptive Expert Split Mechanism. Each expert is vertically divided into two segments (top + bottom) so only the segment needed per token is loaded. Source: arxiv.org/abs/2509.08342.

(4) DALI (arXiv:2602.03495, Feb 2026) -- Workload-Aware Offloading Framework. Stores expert parameters in secondary memory to reduce GPU memory usage without compromising model expressiveness. Source: arxiv.org/abs/2602.03495.


Tradeoffs grid

Throughput: GPU-resident > MoBiLE Big-Little > CPU-GPU collaborative > MoEpic split > DALI disk-offload.

VRAM footprint (lower is better for Ampere): DALI disk-offload < CPU-GPU collaborative < MoEpic split < MoBiLE Big-Little < GPU-resident.

Accuracy delta: MoBiLE reports negligible degradation; the others are first-order lossless (they store full expert params, the offload only changes WHEN params are on-GPU). Verify per-paper on the target model + the actual benchmark.

Engineering complexity: GPU-resident < DALI < MoEpic < MoBiLE Big-Little < CPU-GPU collaborative.

Determinism: all four approaches preserve determinism IF the offload scheduler is deterministic-seeded. EU AI Act compliance pre-gate (E-38) requires determinism; any future expert-offload backend MUST plumb a deterministic seed into the offload scheduler.


Candidate models for ELAI's MoE-on-Ampere tier

Mixtral 8x22B (Mistral, MoE) -- 141B total params, ~39B active per token. With NF4 quant: ~80 GB total weights. Fits 2x3090 (48 GB) ONLY with disk-offload (DALI) or aggressive expert-offload (MoBiLE Big-Little + CPU collab). Single 3090 (24 GB) needs disk-offload only.

DeepSeek V4-Flash distilled -- pending license + EU/US legal posture verification per `docs/strategy/ELAI-2026-POSTURE.md` Section 3. Distilled variants drop active-expert count; viable for 2x3090 with MoBiLE.

Future Qwen-MoE -- not yet released as of May 2026; placeholder for the next Qwen tech report.

Llama 4 Scout -- 10M context, MoE; MRCR v2 at 10M = 15.6% per "1-million-token-lie" analysis (ELAI-2026-POSTURE.md Section 1 signal 3). Context window is a red herring; the model's MoE structure is the relevant property here.


Open questions (resolve before authoring the future B-N/E-N)

(Q1) Which approach scales to 2x3090 (48 GB)? MoBiLE (arXiv:2510.12357) reports consumer-GPU numbers but the target VRAM band needs explicit measurement on Mixtral 8x22B NF4.

(Q2) How does expert-offload interact with the B-37 RuntimeAdapter's arch-blind invariant? The trait stays arch-blind; the BACKEND IMPL knows it's MoE, but the RuntimeAdapter contract doesn't change. Verify by reading `crates/providers/src/runtime/types.rs::RuntimeAdapter` before authoring.

(Q3) Determinism plumbing -- the offload scheduler MUST consume the deterministic seed from PlanContext (see `crates/orchestrator/src/compliance.rs::PlanContext::has_deterministic_seed`). Confirm with E-38 compliance pre-gate before shipping.

(Q4) Inference-time KV cache layout for MoE differs from dense. The contract test `crates/contracts/tests/e38_anti_pattern_contracts.rs::runtime_adapter_trait_stays_arch_agnostic` enforces no arch-named methods on the trait; the backend impl encapsulates the difference.

(Q5) FlashAttention status on MoE -- FA-3 is Hopper-only. Ampere MoE backends use SDPA per `ELAI_FORCE_SDPA=1`. Performance gap quantification needed.


Non-goal

This document does NOT specify an implementation. It is the design-space inventory the future plan author consults. Authoring the actual backend requires:
  - Re-verifying every arXiv ID above (the field moves fast; expect new papers within 6 months)
  - Anti-slopsquatting protocol on any new crate dep (`cargo info <crate>` before adding)
  - Picking one of the 4 approaches above + justifying the choice in the future plan's "Open questions / risks" section
  - Plumbing the deterministic seed end-to-end
  - Composing with the B-37 RuntimeAdapter trait without naming MoE in the trait


END MoE-on-Ampere-tier.md

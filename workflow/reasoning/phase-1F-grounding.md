Phase 1F Grounding Results
Date: 2026-04-06
Session: s-937656

SPEC VERIFICATION
  docs/specs/crates/providers.txt L44-51 (speculative decoding section) -> EXISTS, relevant
  docs/specs/decisions/external-analysis.txt L308-320 (EAGLE-3 entry) -> EXISTS, relevant
  docs/specs/architecture-overview.txt L203-222 (Section 7, hardware) -> EXISTS, relevant
  No SPEC_GAP found. One SPEC_CONFLICT: providers.txt L47 says "~10GB headroom" but actual is ~4.8GB.

CLAIM GROUNDING
  "EAGLE-3 benchmarks show 1.8-2.3x" | CORRECTED | Actual EAGLE-3: 2x-6.5x on large models (70B+). Source: arxiv 2503.01840, NeurIPS'25. The 1.8-2.3x figure appears to be from older EAGLE-1/2 or a specific narrow benchmark. On 2.3B model on consumer GPU, gains will be substantially lower.
  "~10GB headroom on 3060 after main model" | WRONG | 12GB - 7.2GB (Gemma4 E2B Q4_K_M) = 4.8GB. Source: architecture-overview.txt L206, implementation-guide.txt L39.
  "TIDE 1.15x over static speculative decoding" | VERIFIED | arxiv 2602.05145 confirms exactly this figure. Published Feb 2026.
  "Ollama supports natively" (providers.txt L44) | UNVERIFIED | PR #8134 (github.com/ollama/ollama/pull/8134) not confirmed merged as of April 2026. Feature may not be available. Added as R-NEW-1 risk.
  ">1.3x adoption threshold" | UNGROUNDED | Project design decision. No external justification for 1.3x specifically. Acceptable as conservative threshold.
  "Draft model same family, smaller variant" | GROUNDED | Standard pattern confirmed across EAGLE-3, vLLM docs, llama.cpp.
  "Phase 0 complete (independent of 1A-1E)" prerequisite | CORRECTED | Dependency graph (implementation-guide.txt L141) shows 1F branches from 1C. Changed to "Phase 1C complete."
  "Draft model ~2GB additional VRAM" | GROUNDED | Consistent with small Q4 model sizes. ~2GB for a 2B-class draft model is reasonable.

CORRECTIONS APPLIED TO PHASE FILE
  1. EAGLE-3 range updated to 2x-6.5x with note about model-size dependency
  2. VRAM headroom corrected from ~10GB to ~4.8GB
  3. Prerequisite changed from "Phase 0" to "Phase 1C"
  4. Added R-NEW-1: Ollama speculative decoding not confirmed available
  5. Added R-NEW-2: EAGLE-3 benchmarks on large models, expect lower on 2.3B

BLOCKS ADDED
  5 numbered steps (1F.1-1F.5)
  5 PRE_GATE blocks (one per step)
  5 DETERMINISTIC_CHECK blocks (one per step)
  8 ADVERSARIAL scenarios across 5 steps
  5 ACCEPTANCE blocks (one per step)
  1 GROUNDING section (top-level, 8 claims documented)

ADVERSARIAL AUDIT (post-improvement)
  Auditor: adversarial-auditor agent, session s-937656
  BLOCKs found: 5 (all fixed in second pass)
    1. Stale claims in providers.txt/external-analysis.txt -> noted as SPEC CORRECTIONS NEEDED
    2. Wrong crate name elai_providers -> fixed to elai_cc_providers (3 locations)
    3. Wrong test path tests/conformance/ -> fixed to cargo test -p elai_cc_knowledge
    4. vLLM listed as Phase 1 fallback -> removed, llama.cpp only viable fallback
    5. Wrong grep path .elai_cc/rust/src/ -> fixed to .elai_cc/rust/crates/providers/src/
  WARNs found: 6 (all addressed)
    1. Ollama determinism weaker than assumed -> softened to text-level comparison
    2. VRAM boundary == 3000 undefined -> added conservative tie-break (DROP)
    3. Gemma 4 EAGLE-3 draft model may not exist -> added R-NEW-3 risk
    4. stddev criterion ambiguous -> clarified per-mode independently
    5. nvidia-smi static vs polling -> enforced polling in DETERMINISTIC_CHECK
    6. VRAM arithmetic inconsistency -> clarified two scenarios (2GB draft vs 3GB threshold)
  INFOs: 4 (no action needed)

SUBSTANTIVE IMPROVEMENTS (pass 3, grounded elaborations)
  1. Acceptance rate metric added to 1F.4. Evidence: below 0.5 acceptance rate, speculative
     decoding hurts throughput (vLLM blog 2024-10-17, Spheron production guide). Added early
     kill signal — if < 0.5 across 3+ prompts, stop benchmarking, skip to DROP.
  2. num_draft_tokens parameter sweep added to 1F.4. Evidence: optimal k varies by draft
     model size (llama.cpp discussion #10466: 3B=k1, 0.5B=k4). Sweep {1,2,3,4,5,7} instead
     of fixed k=5. Prevents missing optimal operating point.
  3. Prompt lookup decoding baseline added as step 1F.3b. Evidence: zero VRAM overhead,
     up to 2.8x on summarization (vLLM blog, prompt-lookup-decoding repo). If achieves >1.3x,
     draft model unnecessary. Decision gate updated to prefer zero-VRAM option when equal.
  4. Draft model sizing guidance added to 1F.2. Evidence: 0.5B-1.5B outperform 2B-3B on
     consumer GPUs due to lower latency overhead (llama.cpp #10466 benchmarks). Explicit
     recommendation to prefer smallest available same-family model.

SOURCES
  arxiv 2503.01840 (EAGLE-3, NeurIPS'25)
  arxiv 2602.05145 (TIDE, Feb 2026)
  github.com/ollama/ollama/pull/8134 (speculative decoding PR)
  github.com/ollama/ollama/issues/586 (ollama determinism issue)
  github.com/vllm-project/vllm/issues/38893 (Gemma 4 EAGLE-3 support)
  RedHat developer article 2025-07-01 (EAGLE-3 + vLLM benchmarks)
  LMSYS blog 2025-12-01 (EAGLE-3 on Vertex)
  vLLM blog 2024-10-17 (acceptance rate < 0.5 hurts throughput)
  llama.cpp discussion #10466 (draft model size vs speedup benchmarks)
  github.com/apoorvumang/prompt-lookup-decoding (zero-VRAM alternative)
  TETRIS ACL 2025 (optimal draft token selection — noted, not adopted for eval phase)
  Spheron production guide (acceptance rate threshold evidence)

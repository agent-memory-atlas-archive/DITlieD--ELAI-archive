# Guide Giants Lossless Summary — Part 1 of 3

## Index

- Part 1 (this file):
  - token-efficiency-design.txt (1809 lines)
  - pipeline-modes-design.txt (1468 lines)
- Part 2 (guide-giants-summary-part2.md):
  - post-phase4-implementation-order.txt (1205 lines)
  - testing-trust-design.txt (1883 lines)
- Part 3 (guide-giants-summary-part3.md):
  - update-implementation-order.txt (1404 lines) — A-0..A-48 ladder

Files read and byte counts:
- F:\.ELAI_workflow\docs\specs\guide\token-efficiency-design.txt — 83258 bytes, 1809 lines
- F:\.ELAI_workflow\docs\specs\guide\pipeline-modes-design.txt — 67864 bytes, 1468 lines
- F:\.ELAI_workflow\docs\specs\guide\post-phase4-implementation-order.txt — 64539 bytes, 1205 lines
- F:\.ELAI_workflow\docs\specs\guide\testing-trust-design.txt — 98299 bytes, 1883 lines
- F:\.ELAI_workflow\docs\specs\guide\update-implementation-order.txt — 86654 bytes, 1404 lines
- TOTAL: 400614 bytes / 7769 lines

---

# FILE 1: token-efficiency-design.txt

Path: F:\.ELAI_workflow\docs\specs\guide\token-efficiency-design.txt
Date: 2026-04-12
Status: PROPOSAL — needs user review + baseline measurement
Author: Claude Opus 4.6 (design, not implementation)
Purpose: make local 12GB LLM produce frontier-quality output by replacing parameters with system-level verification and retries. Second of ELAI's two equal objectives (first is TRUST).

## Section outline

1. BENCHMARK HARNESS DESIGN (build this first)
2. EXECUTIVE SUMMARY
3. Section 1 — THREE CEILINGS
4. Section 2 — EFFECTIVE CONTEXT BUDGET (per VRAM tier)
5. Section 3 — TASK GRANULARITY CALIBRATION
6. Section 4 — PROMPT STRATEGY FOR SMALL MODELS
7. Section 5 — RETRY ECONOMICS
8. Section 6 — OUTPUT CONSTRAINTS (structured generation)
9. Section 7 — MODEL SWAPPING STRATEGY
10. Section 7B — MODEL SWAP OPTIMIZATION (6 strategies)
11. Section 7C — STATELESS INFERENCE (why context never grows)
12. Section 8 — QUALITY COMPENSATION MECHANISMS
13. Section 9 — CONTEXT WINDOW OPTIMIZATION (per-slot)
14. Section 10 — CONSTRAINED DECODING (vLLM)
15. Section 11 — BASELINE MEASUREMENT PLAN (what was tested)
16. Section 12 — INTEGRATION WITH EXISTING PHASES
17. Section 13 — TYPES
18. OPEN QUESTIONS
19. STEP FILES REQUIRING UPDATES (from pipeline-modes-design audit) — 10 entries

## Benchmark harness layout

Location: `.elai_cc/benches/model_capability/`
- `main.rs`, `bench_config.toml`
- Modules: attention.rs, retry.rs, few_shot.rs, structured.rs, template.rs, turboquant.rs, swap.rs, report.rs
- Fixtures: tasks.json (50 tasks), templates.json (20 template tasks), padding.txt (5000 lines random valid Rust)
- Output: `.elai_cc/metrics/model_baseline.toml`
- Deps: reqwest, serde, serde_json, toml, tokio, tree-sitter, tree-sitter-rust, tree-sitter-python, tree-sitter-typescript, tiktoken-rs or tokenizers. NO dependency on elai_cc crates.

Task distribution (50): 10 easy, 15 medium, 15 hard, 10 multi. Languages: 30 Rust / 10 Python / 10 TypeScript.

### Seven measurements

1. ATTENTION QUALITY CURVE: context_sizes=[1024,2048,4096,8192,12288,16384,24576], task_repeats=5, pass_threshold=0.8. Output: attention_curve table + attention_ceiling (last size >= threshold). Used by ModelCapabilityProfile.
2. RETRY SUCCESS RATE: max_retries=7, temperatures=[0.7,0.3,0.5,0.1,0.7,0.3,0.1]. Output: retry_curve cumulative pass rate, optimal_retries (delta<2%), futility_same_error=3.
3. FEW-SHOT VS ZERO-SHOT: same 50 tasks twice. Output: zero_shot_pass_rate, few_shot_pass_rate, delta, few_shot_token_cost. Verdict ADOPT if delta > 0.10, SKIP if <= 0.05.
4. STRUCTURED VS FREE-FORM: JSON schema vs free-form. Output: freeform_valid_rate, freeform_correct_rate, structured_valid_rate, structured_correct_rate, latency_overhead_pct. Verdict ADOPT if correct delta > 0.05.
5. TEMPLATE FILL VS OPEN-ENDED: 20 tasks two forms. Output: open_ended_correct, template_correct, token_savings_pct.
6. TURBOQUANT IMPACT: fp16 vs TQ4. Output: fp16_max_context, tq4_max_context, tq4_quality_delta, vram_savings_mb.
7. MODEL SWAP LATENCY: A->B / B->A, cold vs warm. Outputs: cold_swap_mean/p95, warm_swap_mean/p95, unload, load_cold, load_warm, warmup_mean_ms.

Build estimate: 1 session (2-4 hours). Benchmark runtime: 15-30 min per model.

## The three ceilings

CEILING 1 — VRAM. Measured 2026-04-14: Qwen2.5-Coder-3B NF4 = 1.92GB weights; 7B NF4 = 5.18GB weights. KV/token: 3B=1KB, 7B=56KB (56x difference). FFN peak at 7B@25K = 1.9GB per layer (mitigated by chunked prefill).

CEILING 2 — In-VRAM KV cache: 3B NF4+FA2 on 12GB = ~40K tokens (~81K with CPU overflow); 7B NF4+FA2 on 12GB = ~25K (projected ~35K with 8-bit KV). 8-bit KIVI: 1.78x saving, lossless for code, ~25% speed cost — only worthwhile for 7B+.

CEILING 3 — Attention quality. Measured: 3B NF4+FA2 and 7B NF4+FA2 both 16K+ context 100% pass; 3B NF4+SDPA drops to 0% at 8K. **FA2 is the critical factor: 4x ceiling improvement over SDPA**. FA2 requires head_dim <= 256 and uniform architecture. Gemma 4 E2B REJECTED: mixed head_dim (256+512) blocks FA2 on 7/35 layers. On 12GB with FA2, CEILING 2 binds for 7B. DESIGN PRINCIPLE: allocator enforces min(ceiling2, ceiling3) per model.

## VRAM tier table (measured vs projected)

| VRAM | Model | Weights | KV/tok | In-VRAM Ctx | Prompt Budget | Features | Status |
|---|---|---|---|---|---|---|---|
| 8GB | Qwen2.5-Coder-3B NF4+FA2 | 1.92GB | 1KB | ~25K (proj) | 20K | no DoLa, no steering | PROJECTED |
| 12GB | Qwen2.5-Coder-3B NF4+FA2 | 1.92GB | 1KB | ~40K | 35K | no DoLa, no steering | MEASURED |
| 12GB | Qwen2.5-Coder-7B NF4+FA2 | 5.18GB | 56KB | ~25K (35K TQ8) | 20K (30K TQ8) | TQ8 default | MEASURED |
| 16GB | Qwen2.5-Coder-7B NF4+FA2 | 5.18GB | 56KB | ~50K (proj) | 45K | TQ8 default | PROJECTED |
| 24GB | Qwen2.5-Coder-14B NF4+FA2 | ~10GB | ~112KB | ~40K (proj) | 35K | TQ8, test DoLa | PROJECTED |
| 48GB | Qwen2.5-Coder-32B NF4+FA2 | ~20GB | 256KB | ~35K (proj) | 30K | TQ4, DoLa, steering | PROJECTED |
| any | Cloud frontier | 0 | 0 | 128K+ | per-provider | per-provider | N/A |

Output reserve: 512 tokens (real code outputs rarely exceed 256).
12GB default: 7B NF4+FA2+TQ8 (90% pass vs 3B 72%). 3B available as --long-context fallback.
48GB default: 32B NF4+FA2+TQ4 (HumanEval 92.7%, MBPP 90.2%).
Model family: **Qwen2.5-Coder-Instruct ONLY. Gemma 4 REJECTED. MoE REJECTED for DoLa/steering.**
FA2 requires env `elai-fa2` (Python 3.12, flash-attn 2.7.4). Python 3.13: no FA2 wheel -> SDPA fallback (~4K ceiling, not production).

DoLa viability: 3B untested likely harmful; 7B HARMFUL (measured: -15% code quality, -6% factuality, 4.7x slower); 14B likely harmful; 32B untested/projected viable. Steering NEUTRAL on 7B (needs domain-specific pairs).

## Context budget allocation per slot & tier (tokens)

| Slot | 48GB (30K) | 12GB-7B (20K) | 12GB-3B (35K) | 8GB (20K) |
|---|---|---|---|---|
| 1 SystemPrompt | 330 | 330 | 330 | 200 |
| 2 Rules | 500 | 400 | 500 | 300 |
| 3 Context Telescope | 1500 | 800 | 1500 | 600 |
| 4 RepoMap | 3000 | 1500 | 3000 | 1500 |
| 5 CodeContext | 10000 | 8000 | 14000 | 8000 |
| 6 SSB entries | 3000 | 1500 | 3000 | 1500 |
| 7 GRC history | 2000 | 1000 | 2000 | 1000 |
| 8 OutputReserve | 512 | 512 | 512 | 512 |
| Total | 20842 | 14042 | 24842 | 13612 |

Key insight: 3B on 12GB gets MORE context than 7B on 12GB (35K vs 20K). RepoMap no longer dropped for any tier.

## Baby step sizing rules (measured)

| VRAM | Model | Max LOC/Step | Max Files | Max Symbols | Rationale |
|---|---|---|---|---|---|
| 8GB | 3B | 25 | 1 | 2 | 72% pass@1 |
| 12GB | 7B | 40 | 2 | 3 | 90% pass@1 |
| 12GB | 3B | 25 | 1 | 2 | 72%, more context |
| 16GB | 7B | 50 | 2-3 | 4 | 90% pass@1, 45K budget |
| 24GB | 14B | 60 | 2-3 | 4 | projected |
| 48GB | 32B | 100 | 3-5 | 5+ | 92.7% HumanEval |
| cloud | API | 100 | 3-5 | 5+ | frontier |

Decomposition depth (MEDIUM task): 3B=3 levels/30-50 steps; 7B=2-3 levels/15-30 steps; 14B=2/10-20; 32B=2/5-15.

Complexity thresholds (surgical, 3B override):
- TRIVIAL: 1 file <50 LOC (3B: 1 file <25 LOC)
- SMALL: 2-5 files <200 LOC (3B: 1-3 files <100)
- MEDIUM: 5-15 files (3B: 3-10)
- LARGE: >15 files (3B: >10)

## Prompt strategy (Qwen2.5-Coder-Instruct only)

Four principles:
1. **LITERAL OVER ABSTRACT** — language-specific literal prompts assembled from FILE_PLAN + ProjectProfile + DKG references. Examples: Rust `validate_jwt(token: &str) -> Result<Claims, AuthError>`, Python `validate_jwt(token: str) -> Claims`, TS `validateJwt`, Go `ValidateJWT`.
2. **SHOW, DON'T TELL** — 1-2 concrete examples from same codebase via DKG/BM25. Cost 200-400 tokens/example; ~50% retry reduction per SWE-bench few-shot studies.
3. **FILL-IN-THE-BLANK** — template with `___BLANK_N___` placeholders. Skeleton deterministic (tree-sitter + types). LLM generates only blank content.
4. **STRUCTURED OUTPUT WITH ADAPTIVE CONSTRAINTS** — grammar-guided code (vLLM EBNF), JSON schema for plans, ThinkStep format GRC, unified diff for edits.
   - **ADAPTIVE CONSTRAINED DECODING (research)**: CRANE adaptive constraints (+10pp over rigid); AdapTrack 61.58%->66.11% EM@1; CodeFast early stop (+34% to +452% speed, zero Pass@1 drop). Implementation chain: `[CRANE_grammar, CodeFast_early_stop, language_syntax_enforcer]`. Ollama fallback: JSON schema only + stronger T2 AST post-gen.

## Retry economics (measured 2026-04-14, Qwen2.5-Coder-7B NF4+FA2)

Pass@1: 90% (18/20). Optimal retries: 1. task_06 and task_15 fail consistently. Retry 1 recovers ~2%.

Retry budget per level:
| Level | Max Retries | Wall-Clock | Escalation |
|---|---|---|---|
| Baby Step | 3 | 3 min | Step (redecompose) |
| Step | 2 | 20 min | Phase (replan) |
| Phase | 2 | 2 hr | Concept (research) |
| Concept | 1 | 4 hr | Human / cloud |

Total worst case MEDIUM: ~6 hours. Cloud equivalent: 5-15 min, $0.50-$5.00.

Research finding: "Prompt engineering ('emotional appeals', tips, threats) empirically worsened scores. Inhibition adaptation = 17.6%." ALL retries must change STRUCTURE, not instruction.

Structural retries:
- Retry 1: DECOMPOSE — MoTCoder-7B +5.9% Pass@1 via decomposition
- Retry 2: TEMPLATE — Interleaved generation +6%, -60% latency
- Retry 3: FEW-SHOT — TDP +6.09% avg
- Retry 4: split further
- Retry 5: BEST-OF-3

Reasoning pipeline integration (reasoning-architecture-design.txt): 6-layer pipeline runs BEFORE structural retries. Layer 1 deterministic (<100ms), Layer 2 micro-reflection (~600t/2-5s), Layer 3 interrogation (~2K/10-30s), Layer 4 AB-MCTS (3-18K/30-90s), Layers 5-6 cloud ($0.002-$2.00). Projected: 85-90% errors resolved L1-L4.

REPAIR CONTEXT PROFILE (new SlotWeights in Phase 3E, 1400 tokens): System 50, BabyStepSpec 200, MistakeBook 150, CodeContext 500, WorkingExample 300, FixInstruction 100, OutputReserve 100.

Generation parameters (FixAudit, arXiv:2604.05560):
- First attempt: T=0.3, top-p=0.9
- Repair retries: T=1.0, top-p=1.0 (prevents cyclic traps)
- AB-MCTS branches: [0.7, 1.0, 0.3]

Futility: same-error-class 3x -> BubbleUp. Regression: retry N breaks N-1 pass -> stop. Output stagnation: >90% identical to N-1 -> BubbleUp.

Best-of-N score: `compile_success*100 + tests_passed*50 + token_confidence_mean*20 + novelty_vs_previous*10 + zero_warnings*10 + code_length_penalty`. Uses GG intrinsic signals (token_confidence = mean logprob, generation_novelty = cosine divergence from last failed output) from sidecar at zero cost. N=3 sweet spot.

## Output constraints

1. Output format enforcement (JSON schema). EXECUTING schema: `{edits: [{file, old, new}], reasoning: "one sentence max"}`. TESTING_FIRST: `{test_file, test_code, expected_behavior}`. PLANNING: TOML matching BabyStepSpec.
2. Diff-only edits — 5-10x fewer output tokens.
3. One function per turn (MICRO tier).

## Model swap strategy

12GB SINGLE-MODEL: all roles share Qwen2.5-Coder-7B NF4+FA2. Executor prompt "You are writing code. Output JSON edits only." Auditor "You are reviewing code. Output findings as JSON." Designer "You are decomposing a task. Output TOML plan structure."

24GB DUAL-MODEL: primary (larger) + secondary (smaller). Predictive loading.
48GB MULTI-MODEL: all models resident.

Swap latency breakdown (cold, 7GB NVMe): Unload 100-500ms + Read 1-2.3s + PCIe 220-500ms + Driver 300-800ms + Warmup 1-3s = 3-7s cold total.

**SIX STRATEGIES to eliminate/hide swap cost**:
1. AVOID SWAPPING (12GB default) — single model ever.
2. PREDICTIVE PRELOADING (mmap to system RAM while executor finishes last tokens; 1.5-2s visible).
3. MMAP + OS PAGE CACHE — free second load (~200-500ms).
4. OVERLAP SWAP WITH DETERMINISTIC WORK (3-45s CPU work, swap runs in parallel via tokio::join).
5. CUDA GRAPH CACHING — dummy warmup inference.
6. WARM MODEL POOL (24GB+) — ModelPool::route(role).

```rust
PredictiveLoader {
  next_model: Option<ModelId>,
  preload_state: PreloadState,        // IDLE | LOADING_RAM | READY_IN_RAM
  mmap_handle: Option<MmapHandle>,
}
```

Config option: `[performance] preload_models_on_startup = true`.

Swap latency by tier:
| VRAM | Strategy | Visible Swap |
|---|---|---|
| 12GB | Single model, no swap | 0ms |
| 16GB | Single model preferred | 0ms (or ~2s) |
| 24GB | Predictive + overlap | 0-500ms |
| 48GB+ | Warm pool | 0ms |

## Stateless inference

Every LLM call is an independent, stateless request. VRAM lifecycle per call: IDLE (7.2GB + 3GB free) -> INFERENCE (weights + KV building) -> GENERATING (weights + KV peak 1-3GB) -> COMPLETE (KV FREED, 3GB free). Baby step N+1's prompt assembled FRESH by allocator from SSB + DKG + knowledge. Prefix caching (vLLM) reuses Slot 1-3 KV across consecutive baby steps for speedup ONLY.

## Quality compensation strategies

A. **NARROWING** — decompose until "first-year CS student could do it given signature + test + example."
B. **TEMPLATING** — categories: FUNCTION_IMPL, TEST_CASE, ERROR_HANDLER, CONFIG_STRUCT, API_ENDPOINT.
C. **VERIFICATION DENSITY** — MICRO tier: every line grammar-guided + every edit T1/T2 + every function T3 compile.
D. **ENSEMBLE BEST-OF-N** (limited) — research says N>32 needed for substantial gains, N>100 for complex. Use ONLY for first baby step of each concept.
E. **PROGRESSIVE ESCALATION** — 12GB chain: 7B -> 3B -> CLOUD -> HUMAN. 48GB: 32B -> 7B -> CLOUD -> HUMAN. Each level fresh context assembly + full retry budget. Expected distribution: 60-70% succeed first try; 85-90% after 5 retries; 10-15% escalate; <2% human.

## Token audit checklist per slot (MICRO target)

- Slot 1 SystemPrompt: 200 tokens (MICRO), 330 (LARGE). Strip "helpful assistant" framing.
- Slot 2 Rules: 150/500. Top 5 rules by recency + file relevance + CRITICAL. Compact DSL: `RULE: no_unwrap | scope: non-test | action: WARN`.
- Slot 3 Telescope: 400/1000. MICRO skips Layers 1-2.
- Slot 4 RepoMap: **0 for MICRO**; SMALL top 10 symbols; MEDIUM+ full.
- Slot 5 CodeContext: 1200/6000. MICRO max 3 snippets (edited fn, one caller, one similar). A-RAG + ShortCoder.
- Slot 6 SSB: 200/1500. MICRO last 2 entries compressed "prev_step: wrote fn X in file Y. Tests: 3/3 pass."
- Slot 7 GRC: 100/1000. MICRO last CONVERGE only (conclusion not chain).
- Slot 8 OutputReserve: 750 (MICRO) / 4000 (LARGE).

Token reclamation techniques: (1) formatting removal 24.5% savings research-grounded, preserve Python indentation; (2) output formatting restoration via tree-sitter; (3) path abbreviation (src/middleware/auth.rs -> middleware/auth.rs); (4) import collapsing "// [12 imports]"; (5) deduplication.

## Constrained decoding

JSON schema mode (vLLM outlines ~5% overhead), Grammar-guided EBNF (~10-15% overhead), Regex constraints, Stop sequences.

## Baseline measurement status (section 11)

- M1 ATTENTION DONE: 3B and 7B NF4+FA2, 512-49152 ctx, 100% pass all sizes up to 16384.
- M2 RETRY DONE: 7B Pass@1 90%, 3B 72%, optimal=1.
- M3 FEW-SHOT INCOMPLETE: 0% both (fixture bug).
- M4 STRUCTURED INCOMPLETE: 0%.
- M5 TEMPLATE INCOMPLETE: 0%.
- M6 KV COMPRESSION DONE: 8-bit KIVI lossless, 1.78x, ~25% speed cost. 4-bit garbage on code.
- M7 SWAP: NOT YET RUN.

## Integration with existing phases

- Phase 3E: ModelCapabilityProfile input, adaptive slot weights, ceiling enforcement, ShortCoder.
- Phase 3J: ConstrainedDecodingConfig, grammar/JSON, model capability detection at startup.
- Phase 4D: decomposition adapts, baby step sizing enforced, template gen, few-shot from DKG.
- Phase 4E: phase sizing uses model tier, SurgicalPlan shallower MICRO, baby-step-size validation.
- Phase 4G: RetryStrategy, futility, best-of-N, escalation chain.
- COMPLETED: 3J.BASELINE (run_benchmark.py + bench_config.toml; results in `.elai_cc/metrics/model_baseline.toml`, `baseline_findings_and_actions.md`).

## Types (Rust)

```rust
enum VramTier { VRAM_8GB, VRAM_12GB, VRAM_16GB, VRAM_24GB, VRAM_48GB, CLOUD }

struct ModelCapabilityProfile {
  vram_tier: VramTier, model_name: String, model_size_b: f32,
  model_vram_mb: usize, kv_bytes_per_token: usize,
  in_vram_ceiling: usize, attention_ceiling: usize, effective_ceiling: usize,
  recommended_prompt_budget: usize, output_reserve: usize,
  slot_weights: SlotWeights, features: ModelFeatures,
  pass_at_1: f32, optimal_retries: usize,
  kv_quant_bits: u8,  // 0=none, 8=KIVI-8bit, 4=KIVI-4bit
  requires_fa2: bool, dola_viable: bool, steering_viable: bool,
  measured: bool,
}

struct SlotWeights { system_prompt, rules, telescope, repo_map, code_context, ssb_entries, grc_history: f32 }
struct ModelFeatures { supports_structured_output, supports_grammar, supports_stop_sequences, chunked_prefill, kv_compression: bool }

struct ConstrainedDecodingConfig {
  mode: DecodingMode, schema: Option<Value>, grammar: Option<String>,
  regex_constraint: Option<String>, stop_sequences: Vec<String>, max_tokens: usize,
}
enum DecodingMode { JSON_SCHEMA, GRAMMAR, REGEX, NONE }

struct RetryStrategy {
  max_retries: usize, wall_clock_budget_ms: u64,
  temperature_schedule: Vec<f32>, use_best_of_n: bool, best_of_n_count: usize,
  futility_same_error_threshold: usize, futility_stagnation_threshold: f32,
}

struct BabyStepSizeLimit { max_loc, max_files, max_new_symbols, max_output_tokens: usize }
struct EscalationChain { levels: Vec<EscalationLevel>, require_user_approval_for_cloud: bool }
struct EscalationLevel { model_tier: ModelTier, model_name: Option<String>, retry_budget: RetryStrategy }
```

## Open questions + recommendations

1. Baseline auto on first use, ~5-10min, progress bar, cached until model change. `/benchmark --model <name>` to re-run.
2. MICRO telescope: include 50-token ultra-compressed project vision.
3. Template generation: deterministic from DKG where possible; LLM optional for blank descriptions.
4. Default best-of-N: N=1 TRIVIAL/SMALL, N=3 critical.
5. Expose retry/escalation to TUI: yes ("[Attempt 2/5] [Error: MISSING_IMPORT] [Retrying with example...]").

## STEP FILES REQUIRING UPDATES (10 entries from pipeline-modes-design audit)

1. **phase-1A.txt (DKG) — HIGH**: Add `dkg_data_flow(source_node_id, target_node_id, variable_name, flow_type)` (PARAMETER/RETURN/ASSIGNMENT/FIELD_ACCESS). Optional line-level def-ref behind `cfg(feature="line_level")`. Grounding: arxiv 2512.08492 (87% vs 56% SWE-bench Verified).
2. **phase-3A.txt (FSM) — HIGH**: RESEARCHING as 10th FSM state, IDLE->RESEARCHING->IDLE, WebSearch + KnowledgeEngine only.
3. **phase-3E.txt (Context Budget Allocator) — HIGH**: ContextTelescope type, BoundedString<N>, Slot 3 sizing ~550-1050, ModelCapabilityProfile input, per-tier slot adaptation, attention ceiling enforcement.
4. **phase-3G.txt (Plan Management) — HIGH**: `summary: BoundedString<N>` on PhaseEntry/ConceptNode/StepNode. `ContextTelescope::from_tree(tree, baby_step_id)`.
5. **phase-4D.txt (Designer) — HIGH/MEDIUM**: TechStackDecision enforcement; 4D-SURGICAL; 4D-ORACLE; baby step sizing per ModelCapabilityProfile.
6. **phase-4B.txt (Tools) — MEDIUM**: DKG query tools via IPC (`dkg_callers`, `dkg_dependents`, `dkg_data_flow`, `dkg_subgraph`); integration wiring check tool. Grounding: arxiv 2603.24837.
7. **phase-4E.txt (Builder) — MEDIUM**: PipelineMode enum (Greenfield | Surgical); build() accepts SurgicalPlan; max depth 2 Surgical; baby step size validation.
8. **phase-4G.txt (Autonomous) — MEDIUM**: blast radius `autonomous_confirm_above_loc=1000`, `autonomous_refuse_above_loc=5000`. `EscalationTrigger::BlastRadiusExceeded`. AutoAdvanceEvaluator pre-exec AffectedSet LOC check. RetryStrategy + futility + best-of-N + progressive escalation.
9. **phase-3E.txt — LOW**: REPAIR_CONTEXT SlotWeights profile (1400 tokens for reasoning pipeline retries).
10. **phase-3J.txt (Sidecar) — LOW**: `mean_logprob` and `entropy_mean` in sidecar response JSON. For AB-MCTS and best-of-N.

## Status markers

- Document status: PROPOSAL.
- 3J.BASELINE: COMPLETED.
- Measurements 1,2,6: DONE. 3,4,5: INCOMPLETE. 7: NOT YET RUN.

## Verbatim non-obvious decisions

- "Time replaces parameters. The system is the intelligence, not the model."
- "CLOUD MODEL: 1 attempt x $0.03 x high accuracy = paid quality. LOCAL MODEL: N attempts x $0.00 x low accuracy + system compensation = FREE quality."
- "FA2 requires: head_dim <= 256, uniform architecture. Gemma 4 E2B REJECTED: mixed head_dim (256+512) blocks FA2 on 7/35 layers."
- "MoE models (Qwen3-Coder-30B-A3B, DeepSeek): REJECTED for DoLa/steering (expert routing breaks layer contrasting and activation addition)."
- "DoLa HARMFUL on 7B: measured -15% code quality, -6% factuality, 4.7x slower, destroys stop-token distribution."
- "Prompt engineering ('emotional appeals', tips, threats) empirically worsened scores. Inhibition adaptation = 17.6%. Models cannot be instructed out of failure."

## External deps / papers referenced

- blake3, tree-sitter, vLLM (guided-decoding-backend=outlines), bitsandbytes NF4, flash-attn 2.7.4
- Papers: MoTCoder +5.9% Pass@1 via decomposition; Interleaved generation +6%; TDP +6.09%; FixAudit arXiv:2604.05560; CRANE; CodeFast; AdapTrack; AIR/DTG arxiv 2512.08492; RepoGraph arxiv 2410.14684; codebadger arxiv 2603.24837; LLMxCPG arxiv 2507.16585; SWE-bench
- Models: Qwen2.5-Coder-Instruct family (3B/7B/14B/32B); MoE models rejected; Gemma 4 rejected

---

# FILE 2: pipeline-modes-design.txt

Path: F:\.ELAI_workflow\docs\specs\guide\pipeline-modes-design.txt
Date: 2026-04-10
Status: PROPOSAL — needs user review
Author: Claude Opus
Purpose: two new pipeline modes — SURGICAL (existing codebases) and ORACLE (pure research/QA) — that reuse existing Phase 4D infrastructure.

## Section outline

1. CONTEXT
2. MODE 1: SURGICAL MODE
   - Section 1 PROJECT DETECTION
   - Section 1B PROJECT INDEXING
   - Section 2 FSM PATH
   - Section 3 SURGICAL DECOMPOSITION
   - Section 4 RESEARCH IN SURGICAL MODE
   - Section 5 INTEGRATION
   - Section 6 TYPES
3. MODE 2: ORACLE MODE
   - Section 1 TRIGGER / ROUTING
   - Section 2 FSM PATH
   - Section 3 RESEARCH PIPELINE (Phases A-G)
   - Section 4 WHAT STOPS LLM FROM TRUSTING BAD SOURCES (5 levels)
   - Section 5 HANDLING "I DON'T KNOW"
   - Section 6 TYPES
4. INTEGRATION WITH EXISTING ARCHITECTURE (6 changes)
5. PHASE PLACEMENT
6. SURGICAL MODE CONTEXT FLOW (7 steps)
7. CONTEXT TELESCOPE (5 layers, vertical/horizontal/structural context)
8. GROUNDING ENFORCEMENT (4 parts: TechStackDecision gate, research at every level, cache+staleness, package verification)
9. POST-EDIT FEEDBACK LOOP (per-edit + baby step completion + full timeline + surgical integration wiring)
10. RESPONSE TO GEMINI 3.1 PRO FEEDBACK (3 traps)
11. KNOWLEDGE ARCHITECTURE ASSESSMENT vs SOTA (3 gaps)
12. OPEN QUESTIONS + recommendations (6 questions)
13. REASONING ARCHITECTURE INTEGRATION (surgical + oracle)

## Project detection (deterministic, no LLM)

Signals (pure filesystem):
- Build system file (Cargo.toml etc): REQUIRED (any 1)
- src/ or lib/ >10 source files: STRONG
- Test files (tests/, *_test.*, test_*.*): STRONG
- Git history >5 commits: MODERATE
- Dependencies declared (>3 external): MODERATE

REQUIRED + at least 1 STRONG = existing project. Overrides: `/task --new` (greenfield), `/task --surgical`. `/fix` always surgical.

## Project indexing (first-time)

Pipeline (all deterministic, no LLM): DKG build (tree-sitter -> SQLite, <15s) -> BM25 (tantivy) -> vector embedding (GraphCodeBERT + Granite dual) -> PageRank. Total 15-60s. Incremental after: SSB file_write events or fs watcher re-index changed files.

Exclusion list default: `node_modules/`, `target/`, `.git/`, `__pycache__/`, `dist/`, `build/`, `.env`, `*.key`, `*.pem`, `credentials.*`, `secrets.*`. Binary detection: null bytes in first 8KB. `.gitignore` respected (override `[knowledge.include_gitignored]=true`). Configurable `[knowledge.exclude]`.

Storage: `.elai_cc/indexes/` (DKG SQLite + tantivy dir + vector cache SQLite). Size: 50-200MB medium project. Gitignored. Re-index on schema mismatch; decline -> DEGRADED (BM25_ONLY).

### CURRENT IMPLEMENTATION GAP (verified 2026-04-10)

- `DkgEngine::new` only in knowledge/examples/measure.rs and tests. ZERO calls in CLI/orchestrator.
- `build_graph` same.
- CLI imports only DkgQuerierAdapter (main.rs:64). Falls back to EmptyDkgQuerier.
- Orchestrator crate does NOT depend on elai-knowledge.
- Tests: 27 pass, metrics in phase_1A.toml. But nobody calls `DkgEngine::new`/`build_graph` in the pipeline. Database only exists if user runs examples/measure.rs.
- **Required**: (1) orchestrator -> elai-knowledge dep; (2) IndexingService calling DkgEngine::new + build_graph + BM25 + vector embedding; (3) CLI/orch triggers on first /task when no indexes; (4) incremental via SSB file_write events.

## FSM path (Surgical)

`IDLE -> PLANNING -> [FEASIBILITY_PROBE] -> TESTING_FIRST -> EXECUTING -> AUDITING -> MEMORIZING -> COMPLETE -> IDLE`. Only PLANNING output differs (SurgicalPlan vs ConceptDecomposition). FEASIBILITY_PROBE often skipped in surgical mode.

## Surgical decomposition (4 steps)

Step 1 — BLAST RADIUS ANALYSIS (deterministic, <10ms, pure graph traversal): DKG query for mentioned symbols; traverse edges outward (direct/1-hop/2-hop + test files). Output: `AffectedSet { direct_files, dependent_files, test_files, total_loc_estimate, dependency_depth }`.

Step 2 — COMPLEXITY CLASSIFICATION (deterministic):
- TRIVIAL: 1 file <50 LOC -> depth 0 (direct TESTING_FIRST)
- SMALL: 2-5 files <200 LOC -> depth 1 (Steps only)
- MEDIUM: 5-15 files multi-module -> depth 2 (Phases + Steps)
- LARGE: >15 files cross-module -> depth 2 + integration phases
- NEVER 4-level. Max depth = 2 for surgical.

Step 3 — STEP GENERATION (LLM, constrained to AffectedSet + existing patterns). Steps have 8 blocks + FILE_PLAN (9th). File size limits language-specific via ProjectProfile.

Step 4 — REGRESSION GATE INJECTION (deterministic): after each step run all existing tests in blast radius. Any break = BLOCK; re-analyze / re-decompose.

## Research priority in surgical mode

Priority 1: project's own code (DKG + BM25). Priority 2: existing dependencies' docs. Priority 3: external research with SCOPE CONSTRAINT. ResearchCache shared with greenfield.

## Integration check per step (language-parameterized via ProjectProfile)

Build per language:
- Rust: `cargo check --workspace`
- Python: `python -m py_compile` + `mypy` (if configured)
- Go: `go vet ./...`
- TypeScript: `tsc --noEmit`
- Java: `gradle compileJava` / `mvn compile`
- Other: `ProjectProfile.build_cmd`

Run tests in blast radius (`ProjectProfile.test_cmd` scoped). Full workspace suite at CONCEPT completion only. DKG update after each step.

## Surgical types (Rust)

```rust
struct SurgicalPlan { request: String, affected_set: AffectedSet, complexity: ComplexityClass, phases: Vec<SurgicalPhase>, regression_baseline: TestBaseline }
struct AffectedSet { direct_files: Vec<FilePath>, dependent_files: Vec<FilePath>, test_files: Vec<FilePath>, total_loc_estimate: usize, dependency_depth: usize }
enum ComplexityClass { TRIVIAL, SMALL, MEDIUM, LARGE }
struct SurgicalPhase { goal: String, steps: Vec<SurgicalStep>, regression_gate: RegressionGate }
struct SurgicalStep { grounding, pre_gate, task, tdd, deterministic_check, acceptance, integration, exit_criteria }  // 8 blocks, GROUNDING references DKG + existing code
struct TestBaseline { total_tests, passing, failing: usize, snapshot_hash: Blake2sHash }  // (note: actually blake3 per Anti-Stale-Training)
struct RegressionGate { test_scope: Vec<TestTarget>, must_pass: Vec<String>, baseline: TestBaseline, allow_new_failures: bool /* false default */ }
```

## Oracle mode routing

New CLI: `/research <question>` + `/ask <question>` alias. Enhanced fallback classifier (deterministic regex, no LLM):
- `HAS_PROJECT_REF` = message references file/function/module in DKG
- `HAS_ACTION_VERB` = implement/fix/add/create/modify/refactor/build/write
- `HAS_QUESTION` = `?` or how/what/why/when/does/is/can/compare/explain
- `NO_CODE_INTENT` = NOT "make it"/"change"/"update the code"

Routes:
- HAS_ACTION_VERB -> /task (or /fix)
- HAS_PROJECT_REF + HAS_QUESTION -> /explore
- HAS_QUESTION + NO_CODE_INTENT + !HAS_PROJECT_REF -> /research
- default -> /task

## Oracle FSM

`IDLE -> RESEARCHING -> IDLE` (new state or sub-state of PLANNING). No file edit, no bash (except curl via proxy). WebSearch + KnowledgeEngine + GRC. GRC budget 5/1000 per research topic.

## Research pipeline (7 phases)

- **Phase A — QUERY FORMULATION (LLM)**: user question -> 2-4 search queries. Structural rephrasing; queries logged in SSB.
- **Phase B — SEARCH EXECUTION (deterministic)**: WebSearch, collect URL+snippet+page, dedupe by URL.
- **Phase C — SOURCE TIER CLASSIFICATION (deterministic, no LLM)**:
  - TIER 1 AUTHORITATIVE: docs.python.org, doc.rust-lang.org, docs.oracle.com/javase, rfc-editor.org, w3.org/TR, ecma-international.org, arxiv.org, doi.org, dl.acm.org, ieee.org, postgresql.org/docs, docs.docker.com, kubernetes.io/docs, github.com/{official_org}/*
  - TIER 2 STRONG: developer.mozilla.org, learn.microsoft.com, wiki.archlinux.org, stackoverflow.com (answer score>20), serverfault.com (score>10), blog.rust-lang.org, dev.mysql.com, redis.io/docs, mongodb.com/docs, oreilly.com, manning.com
  - TIER 3 WEAK: personal blogs, Medium, Dev.to, Hashnode, tutorialspoint, geeksforgeeks, w3schools, SO score<20, random GitHub, YouTube transcripts, Reddit/HN
  - TIER 4 REJECTED (silent): SEO spam, AI content farms (domain pattern), paywalled, 403/404/captcha
  - Static Rust HashMap/phf. Unknown -> TIER 3 default. Loaded from `.elai_cc/config/source-tiers.toml`.
- **Phase D — FACT EXTRACTION (LLM, constrained)**: `ExtractedFact { claim, source_url, source_tier, exact_quote, page_section }`. No exact_quote = UNSUPPORTED (logged, not used).
- **Phase E — CROSS-VERIFICATION (deterministic)**:
  - VERIFIED: 2+ independent T1/T2 sources
  - SUPPORTED: 1 T1/T2
  - WEAK: only T3
  - UNSUPPORTED: no exact quotes
  Matching = set intersection (LLM semantic similarity OK, but TIER/COUNT deterministic).
- **Phase F — SYNTHESIS (LLM, constrained)**: receives only ExtractedFacts, never raw pages. Every sentence must cite a fact (regex check). Output format: ANSWER / EVIDENCE CHAIN / CONFIDENCE (HIGH/MEDIUM/LOW/UNKNOWN) / CAVEATS / UNANSWERED.
- **Phase G — POST-VERIFICATION (deterministic)**: citation check, tier consistency, "I don't know" check (if 0 VERIFIED/SUPPORTED -> output says so), hallucination guard (claims not in ExtractedFacts -> BLOCK). Re-synth max 2 retries, then escalate.

## 5 enforcement levels (what stops LLM trusting bad sources)

1. Source tier assigned by CODE, not LLM.
2. Exact quote requirement.
3. Cross-verification (2+ independent sources for VERIFIED).
4. Synthesis constraint (only sees ExtractedFacts).
5. Post-verification regex (uncited sentences removed).

## "I don't know" handling

0 VERIFIED/0 SUPPORTED -> "I don't know". Only WEAK -> "I found some information but cannot verify it" + WEAK-labeled evidence chain. User fallbacks: rephrase, `--domain=X`, `--seed=<url>`, `--include-weak`.

## Oracle types (Rust)

```rust
struct ResearchQuery { original_question: String, formulated_queries: Vec<String>, domain_hints: Vec<String>, seed_urls: Vec<Url>, include_weak: bool }
enum SourceTier { T1_AUTHORITATIVE, T2_STRONG, T3_WEAK, T4_REJECTED }
struct SourceClassification { url: Url, domain: String, tier: SourceTier, classification_method: ClassMethod /* EXACT_DOMAIN/PATTERN/DEFAULT */ }
struct ExtractedFact { id: FactId, claim: String, source_url: Url, source_tier: SourceTier, exact_quote: String, page_section: String, extraction_confidence: f32 }
enum VerificationLevel { VERIFIED, SUPPORTED, WEAK, UNSUPPORTED }
struct VerifiedClaim { claim: String, supporting_facts: Vec<FactId>, verification_level: VerificationLevel, independent_source_count: usize }
struct ResearchAnswer { question: String, answer_text: String, evidence_chain: Vec<VerifiedClaim>, confidence: AnswerConfidence, caveats: Vec<String>, unanswered_parts: Vec<String>, sources_consulted: usize, sources_rejected: usize }
enum AnswerConfidence { HIGH, MEDIUM, LOW, UNKNOWN }
```

## Integration with existing architecture

1. CLI: `/research` + `/ask` alias, enhanced classifier, `--surgical` / `--new` flags on `/task`.
2. FSM: RESEARCHING state (tools: WebSearch + KnowledgeEngine + GRC only; blocks edits/bash/git).
3. Pipeline factory: SurgicalDecomposer alongside ConceptDecomposer. Both produce executable plans for TreeNavigator (4E). SurgicalPlan -> shallow DecompositionTree (depth<=2).
4. Knowledge engine: Surgical DKG=PRIMARY grounding; Oracle DKG optional. Non-Rust projects use regex fallback (DKG Rust-only in Phase 1A).
5B. Language-parameterized enforcement: Surgical applies code-completeness-enforcement-design.txt; build/test/lint from ProjectProfile; module conventions language-specific (Rust mod.rs, Python __init__.py, TS index.ts, Go package dirs).
5. Research agent reuse: Oracle reuses FlashResearch (4D). Oracle stops after synthesis (no plan gen).
6. Source tier config: `.elai_cc/config/source-tiers.toml` ships with ~200 domains, reloadable without rebuild.

## Phase placement

- SURGICAL: depends on 1A (DKG), 3A (FSM), 4D (research partial). Suggested 4D-SURGICAL.
- ORACLE: depends on 4D only. Suggested 4D-ORACLE. Simpler: no code execution, no tests.

## Surgical context flow (7 steps, user request -> LLM prompt)

1. DKG GRAPH (SQLite, 0 tokens, already indexed) — "the MAP".
2. BLAST RADIUS via DKG TRAVERSAL (<10ms) -> AffectedSet.
3. PAGERANK REPO MAP (symbols in AffectedSet get 10x boost, binary-search fit to ~2-4K tokens, Slot 3).
4. TARGETED CODE RETRIEVAL (BM25 + vectors GraphCodeBERT/Granite + CatRAG graph-aware + A-RAG 55-87% token savings + cross-encoder reranker, fit to Slot 5).
5. CONTEXT ASSEMBLY (Phase 3E, 8 slots, <50ms, two cache breakpoints).
6. LLM receives 4K-12K tokens focused context.
7. DECOMPOSITION constrained by AffectedSet + regression gates.

## Context Telescope (5 layers, vertical/horizontal/structural)

### 1. Vertical (ancestry chain)

- LAYER 1 PROJECT VISION ~200 tokens (once at plan creation)
- LAYER 2 CONCEPT SUMMARY ~150 tokens per concept
- LAYER 3 PHASE GOAL ~100 tokens per phase
- LAYER 4 STEP CONTEXT ~100 tokens per step
- LAYER 5 BABY STEP TASK ~200-500 tokens (8 blocks)
- Total ancestry overhead: ~550-1050 tokens. Written ONCE during decomposition.

```rust
struct ConceptNode { summary: BoundedString<150>, /* other fields */ }
struct PhaseNode { summary: BoundedString<100>, /* other fields */ }
struct BoundedString<const N: usize>(String);
impl<const N: usize> BoundedString<N> { fn new(s: &str, tokenizer: &dyn TokenCounter) -> Result<Self, TooLong>; }
```

### 2. Horizontal (sibling/predecessor awareness, existing infrastructure)

- A. SSB HISTORY — file_overlap_score filter, A.2 sees "A.1 wrote validate_token() in auth/jwt.rs, tests passed."
- B. DKG INCREMENTAL UPDATES — A.1's code becomes grounding for A.2 via BM25/vector search.
- C. PRE_GATE CHECKS — deterministic: grep, AST check, test run.

### 3. Structural (full plan during decomposition)

PlanSkeleton (Phase 3G): TOML 2-4K tokens, all phases with input/output contracts. Always loaded in Slot 3 during PLANNING.

### 4. Surgical mode telescope (shallower, ~400-900 tokens)

- LAYER 1 PROJECT VISION (~200 from DKG concept annotations + README/Cargo.toml, auto-gen on first indexing)
- LAYER 2 CHANGE CONTEXT (~200, blast radius)
- LAYER 3 STEP TASK (~200-500)

### 5. Context assembly complete picture (greenfield, ~14330 tokens of 16K window)

Slot 1 330 / Slot 2 500 / Slot 3 1000 / Slot 4 2000 / Slot 5 6000 / Slot 6 1500 / Slot 7 1000 / Slot 8 2000.

### 6. Types

```rust
struct ContextTelescope {
  project_vision: BoundedString<200>,
  concept_summary: BoundedString<150>,
  phase_goal: BoundedString<100>,
  step_context: BoundedString<100>,
}
impl ContextTelescope {
  fn from_tree(tree: &DecompositionTree, baby_step_id: NodeId) -> Self;
  fn render(&self) -> String;
  fn token_estimate(&self) -> usize;
}
```

## Grounding enforcement

### 1. TechStackDecision as GATE

```rust
struct TechStackDecision {
  choice: String,                      // "blake3"
  alternatives_considered: Vec<String>,
  source: DecisionSource,              // RESEARCH | EXISTING_PROJECT | USER_SPECIFIED | FALLBACK
  evidence: Vec<ResearchRef>,
  rationale: String,
}
```

DecisionSource rules (orchestrator rejects):
- EXISTING_PROJECT: codebase uses it (DKG confirms). Default in surgical.
- USER_SPECIFIED: user said so.
- RESEARCH: FlashResearch run, sources + tier classified. REQUIRED for new tech.
- FALLBACK: research found nothing. ONLY for non-critical (logging format, test helper). BLOCKED for critical (crypto, auth, database, serialization).

REJECTED: no TechStackDecision / FALLBACK on critical / RESEARCH with 0 T1/T2 sources.

### 2. Research at every decomposition level

Decomposer output schema REQUIRES TechStackDecision for every tech reference. Orchestrator validates schema before accepting. Missing = schema rejection = retry.

### 3. Research cache + staleness prevention

Cache key: normalized topic (lowercase, sorted). TTL by volatility:
- STABLE (algorithms, data structures): 90 days
- MODERATE (frameworks, libraries): 30 days
- VOLATILE (APIs, cloud services, versions): 7 days
Volatility by source URL domain, not LLM.

### 4. Package verification (anti-hallucination)

PackageRef API_REGISTRY_CHECK: crates.io / npm / PyPI. Also verify API usage claims against docs.

## Post-edit feedback loop

### Per-edit checks (fire after every Edit/Write)

- T1 (<1ms) predicate DSL rules (rules.db). Example: `.unwrap()` in non-test -> WARN.
- T2 (<10ms) tree-sitter parse. Fail -> immediate error to LLM.
- T3 (<5s) `cargo check --package X` incremental. Fail -> compiler error to LLM.

Bash (test exec): FailureTaxonomy parse (Phase 4B: 9 categories) -> classified error to LLM.

### Baby step completion checks

- CHECK 1 TEST FLIP: baby step tests flip failing->passing. Max 3 retries.
- CHECK 2 REGRESSION: all existing tests in blast radius still pass. Break = BLOCK.
- CHECK 3 INTEGRATION WIRING (NEW):
  - Method A DKG reachability: path from `main()` to new symbol?
  - Method B grep for usage of new pub symbols.
  - Method C import chain (main.rs -> use/mod).
  - Not wired: if INTEGRATION block says so -> FAIL + add wiring; if separate baby step -> flag "pending integration" for wiring step's PRE_GATE.

Full timeline: CONTEXT ASSEMBLY -> PRE_GATE -> TESTING_FIRST -> EXECUTING (edits with PostToolUse per edit) -> COMPLETION CHECKS -> AUDITING -> ACCEPTANCE -> DKG re-index -> SSB record -> EXIT CRITERIA -> next baby step. Fail (3 retries max) -> BubbleUpHandler -> step level redecompose.

## Response to Gemini 3.1 Pro feedback (3 traps)

- TRAP 1 "tree-sitter brittle for data flow": correct — edges are HEURISTIC enrichment, not gates. Tracks: parameter, return, assignment, field access. Cannot track closures, dynamic dispatch, complex control flow, macros. Blast radius primary gate = call graph.
- TRAP 2 "Oracle I-don't-know loop": correct — "Found information from unverified sources only. No official documentation covers this yet."
- TRAP 3 "Test suite runtime": correct — per level: baby step `cargo check` only (<5s); step module tests (<30s); phase crate tests (<60s); concept workspace tests (<120s).

## Knowledge architecture assessment (3 gaps)

### What we do well

1. Semantic compression: CatRAG + A-RAG 55-87% token savings (arxiv 2507.16585).
2. Multi-modal: BM25 + dual vectors + reranker.
3. PageRank personalization (arxiv 2410.14684 RepoGraph).

### GAP 1 DATA FLOW TRACKING (critical, highest)

AIR/DTG (arxiv 2512.08492): SWE-bench Verified 87.1% vs 56.2% without DTG.
Schema: `dkg_data_flow(source_node_id, target_node_id, variable_name, flow_type IN ('PARAMETER','RETURN','ASSIGNMENT','FIELD_ACCESS'))`. Lightweight tree-sitter addition.

### GAP 2 GRAPH QUERIES AS AGENT TOOLS (medium)

codebadger (arxiv 2603.24837): Joern CPG MCP server, found zero-days in libtiff, patched libxml2 CVE.
Expose:
```
dkg_callers(symbol) -> Vec<DkgNode>
dkg_dependents(symbol) -> Vec<DkgNode>
dkg_data_flow(variable, direction) -> Vec<FlowPath>
dkg_subgraph(entry_point, depth) -> SubGraph
```

### GAP 3 LINE-LEVEL GRANULARITY (lower, optional)

RepoGraph arxiv 2410.14684. Feature-gated `cfg(feature="line_level")`.

### What to skip

Full CPG (AST+CFG+PDG) bloated; VCSG wrong domain; CLeVeR contrastive training unnecessary (dual-vector sufficient); LLMxCPG CPGQL vuln-specific.

### 3K enhancement — data-flow-aware retrieval

Query mentions variable/type -> follow data flow edges to touching functions, include even if BM25/vector low.

## Open questions + recommendations

1. RESEARCHING full state vs sub-state -> **Full state** (different tools + exit).
2. Oracle answer caching -> **Yes, tier-based TTL** (T1=30d, T2=7d, T3=1d). SQLite in `.elai_cc/`.
3. Surgical always DKG first -> **Yes** (<10ms). Skip decomposition only for TRIVIAL.
4. Default source-tiers.toml size -> **~150 curated domains** (T1 ~30, T2 ~50, T4 blocklist ~20, rest T3 default).
5. Oracle follow-ups -> **Yes, with constraints**. RESEARCHING->RESEARCHING, max 5 follow-ups per session. Context carries previous facts.
6. Max blast radius threshold:
   - AUTONOMOUS: >1000 LOC confirm, >5000 refuse
   - INTERACTIVE: >5000 warn, >10000 suggest breaking down; no hard refusal
   - Configurable `[surgical.max_blast_radius]`.

## Reasoning architecture integration

### Surgical + reasoning pipeline

- Layer 1 (60-70%): blast radius feeds root cause localization. DKG edge traversal resolves UNDEFINED_SYMBOL. TYPE_MISMATCH gets expected type from callee signatures. Fix templates reference AffectedSet patterns.
- Layer 2 (20-25%): Compressed Context Replay uses AffectedSet. More precise than greenfield's DKG similarity search.
- Layer 3: Decision trees language-parameterized via ProjectProfile.language.
- Layer 4 (AB-MCTS): branch point from blast radius. GG intrinsic signals score branches. MistakeBook forbidden_diffs.
- Layers 5-6 (cloud): receive surgical context — AffectedSet, TestBaseline, regression gate results.

REGRESSION GATE = primary verification signal for all reasoning layers.

### Oracle + reasoning pipeline

Oracle does NOT use reasoning pipeline (no code gen = no errors). Oracle source tier classification shares infrastructure with Layer 5 cloud routing (tiered model selection Haiku/Sonnet/Opus).

### Types affected

```rust
struct SurgicalStep { /* ... */, reasoning_budget: Option<ReasoningBudget> }
struct ReasoningBudget {
  max_ab_mcts_branches: usize,      // default 15, reduced for TRIVIAL
  cloud_escalation_tier: CloudTier, // default Haiku, Sonnet for LARGE
  micro_reflection_enabled: bool,   // false for TRIVIAL
}
```

### Context flow retry update

Step 6 (retry) — COMPRESSED CONTEXT REPLAY (context WIPED, fresh 1400 tokens: baby step 200 + MistakeBook 150 + relevant code from AffectedSet 500 + working example from DKG 300 + fix instruction L1/L2/L3/L5 100 + reserve 100). Previous conversation/failed output NEVER shown (Kimi-Dev principle).

Step 7 (repair gen params) — FixAudit temperatures. First attempt T=0.3, retries T=1.0 top-p=1.0.

Cross-references: reasoning-architecture-design.txt (6-layer pipeline), token-efficiency-design.txt section 5, code-completeness-enforcement-design.txt Layer 5 BubbleUp.

## External deps / papers

- tree-sitter, tantivy, SQLite, GraphCodeBERT + Granite dual embeddings, cross-encoder reranker, CatRAG, A-RAG, PageRank
- Papers: AIR/DTG 2512.08492, RepoGraph 2410.14684, codebadger 2603.24837, LLMxCPG 2507.16585, TiCoder 2404.10100, SWE-bench
- Models: Anthropic Haiku/Sonnet/Opus tiered; Claude Code comparison; Kimi-Dev principle
- Registries: crates.io, npm, PyPI
- Papers status: all verified via WebSearch as "verified"

## Status markers

- Document status: PROPOSAL.
- Section 1B current implementation gap: VERIFIED 2026-04-10, DKG not wired.

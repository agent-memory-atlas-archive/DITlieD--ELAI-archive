# Verification Report: token-efficiency-design.txt
Source: F:\.ELAI_workflow\docs\specs\guide\token-efficiency-design.txt
Date: 2026-05-17
Overall status: PARTIAL
Confidence: HIGH

## Summary
The design specifies 20+ mechanisms across benchmark harness, context budget adaptation, constrained decoding, retry economics, model swapping, and token reclamation. The context budget layer (ModelCapabilityProfile, SlotWeights, VramTier, effective ceiling, per-tier slot targets, REPAIR_CONTEXT, telescope with BoundedString, token reclamation, output formatting restoration, ShortCoder) is substantially implemented. The benchmark harness is partially implemented as a Python script with measured results in model_baseline.toml. Critical runtime mechanisms — structured ConstrainedDecodingConfig type, CRANE-style grammar-guided generation, structural retry strategy, best-of-N for baby steps, futility detection, and predictive preloading — are either absent or only partially stubbed. The ETC LogitsProcessor (entropy-trend, not CodeFast early stopping) is implemented in the sidecar but differs from the CodeFast mechanism described.

## Mechanisms verified DONE

- ModelCapabilityProfile struct with all fields from spec section 13 — evidence: crates/orchestrator/src/context_budget.rs:271-298
- VramTier enum (Vram8Gb/12Gb/16Gb/24Gb/48Gb/Cloud) — evidence: crates/core/src/vram_tier.rs:172, crates/providers/src/vram_tier.rs:45-50
- ModelTier enum (Micro/Small/Medium/Large/Cloud) — evidence: crates/orchestrator/src/context_budget.rs:2531
- SlotWeights struct (7-field proportional allocations) — evidence: crates/orchestrator/src/context_budget.rs:323-330
- ModelFeatures struct (supports_structured_output, supports_grammar, supports_stop_sequences, chunked_prefill, kv_compression) — evidence: crates/orchestrator/src/context_budget.rs:241
- Per-tier preset profiles: micro_7b_12gb(), micro_3b_12gb(), micro_8gb(), large_32b_48gb() — evidence: crates/orchestrator/src/context_budget.rs:308, 2563
- detect_model_capability_profile() wired at startup — evidence: crates/orchestrator/src/context_budget.rs:554, crates/orchestrator/src/task_runner.rs:2623
- effective_ceiling = min(in_vram_ceiling, attention_ceiling) — evidence: crates/orchestrator/src/context_budget.rs:282, 2606
- fixed_slot_targets [SystemPrompt, Rules, Telescope] per tier (e.g. [330, 400, 800] for 7B-12GB) — evidence: crates/orchestrator/src/context_budget.rs:331
- ContextTelescope type with BoundedString per layer (project_vision, concept_summary, phase_goal, step_context) — evidence: crates/core/src/telescope.rs:66-107, crates/orchestrator/src/telescope_caller.rs:128-180
- Telescope layers collapsed to budget via collapse_to_budget() — evidence: crates/orchestrator/src/telescope_caller.rs:168-180
- REPAIR_CONTEXT slot weights profile + ContextBudget::compute_repair_context() — evidence: crates/orchestrator/src/context_budget.rs:889, 2867
- invoke_repair_telescope() for repair-path context assembly — evidence: crates/orchestrator/src/task_runner.rs:8064
- ShortCoder (AST simplification for CodeContext compression) — evidence: crates/knowledge/src/shortcoder.rs, crates/orchestrator/src/shortcoder_hook.rs
- apply_token_reclamation() (path abbreviation, import collapse, doc-comment strip) — evidence: crates/orchestrator/src/context_budget.rs:1544, 3591
- abbreviate_path() function — evidence: crates/orchestrator/src/context_budget.rs:1544, 2961
- restore_output_formatting() via tree-sitter post-generation — evidence: crates/orchestrator/src/context_budget.rs:3090, crates/contracts/tests/output_postprocessor_tree_sitter_contract.rs:82
- stop_sequences propagated to vLLM and ollama providers — evidence: crates/providers/src/runtime/vllm.rs:174, crates/providers/src/runtime/ollama.rs:180
- json_schema_mode flag per runtime backend (vLLM=true, anthropic=false, etc.) — evidence: crates/providers/src/runtime/types.rs:157, vllm.rs:114
- BubbleUpPolicy with per-level retry counters (baby_step=3, step=2, phase=1, concept=1) — evidence: crates/orchestrator/src/pipeline_tree_navigator.rs:83-89
- BubbleUpEvent tracking and NavigatorTree execution — evidence: crates/orchestrator/src/pipeline_tree_navigator.rs:1219, 1772
- PerRoleRuntime (role-to-backend mapping, single-model default) — evidence: crates/orchestrator/src/per_role_runtime.rs:44
- PreloadManager + trigger_preload() on FSM state transitions — evidence: crates/orchestrator/src/preload.rs:21, crates/orchestrator/src/fsm.rs:664
- model_baseline.toml with measured attention_curve, retry_curve data — evidence: .elai_cc/metrics/model_baseline.toml (measurements 1+2 present)
- run_benchmark.py harness (Python, not Rust, at benches/model_capability/) — evidence: benches/model_capability/run_benchmark.py
- fixtures/tasks.json (50 coding tasks) — evidence: benches/model_capability/fixtures/tasks.json
- bench_config.toml — evidence: benches/model_capability/bench_config.toml
- FewShotExample type and few_shot_for_symbol() DKG lookup — evidence: crates/knowledge/src/fix_templates.rs:255, crates/orchestrator/src/ssb.rs:1415
- FixTemplateRegistry (code template patterns for known failure categories) — evidence: crates/knowledge/src/fix_templates.rs:59, crates/knowledge/src/tier3.rs:511
- FocusTunnel unified-diff output enforcement (model outputs diffs not full files) — evidence: crates/orchestrator/src/focus_tunnel/file_router.rs, crates/contracts/tests/focus_tunnel_e2_contract.rs:234
- ETC LogitsProcessor in sidecar (entropy-trend monitoring, prevents runaway generation) — evidence: python/elai_inference/server.py:952-980
- prefix_cache_hit optimization in sidecar (stateless inference with prefix KV reuse) — evidence: python/elai_inference/server.py:1912

## Mechanisms PARTIAL

- Benchmark harness (spec: Rust binary; actual: Python script)
  - exists: benches/model_capability/run_benchmark.py (Python, not Rust main.rs)
  - gap: design specified a Rust binary with tree-sitter parse validation; Python harness lacks tree-sitter validation. measurements 3, 4, 5 show 0% (fixture/validation bug noted in spec section 11). fixtures/templates.json and fixtures/padding.txt are missing (only tasks.json exists)

- Constrained decoding (spec: ConstrainedDecodingConfig struct with DecodingMode enum, CRANE-style adaptive grammar, EBNF grammars per language)
  - exists: json_schema_mode bool per backend (crates/providers/src/runtime/types.rs:157); ModelFeatures.supports_grammar bool (context_budget.rs:241); constrained_decoding_unavailable error constant in sidecar (server.py:117); a9_constrained_decoding test (tests/media_validator_adversarial.rs:474)
  - gap: no ConstrainedDecodingConfig struct or DecodingMode enum in codebase. CRANE adaptive grammar algorithm not implemented. EBNF grammar-guided decoding not wired to vLLM. supports_grammar is always false across all profiles tested (context_budget.rs:2737). No stop-sequence-per-output-type configuration per agent role

- Structural retry strategy (spec: RetryStrategy type; decompose->template->few_shot->split->best-of-3 sequence)
  - exists: BubbleUpPolicy with per-level max_retries (pipeline_tree_navigator.rs:170); recovery_recipes.rs with EscalationPolicy variants; FixTemplateRegistry for template retry
  - gap: no RetryStrategy struct with temperature_schedule, futility_same_error_threshold, futility_stagnation_threshold fields. No implemented sequence of DECOMPOSE->TEMPLATE->FEW_SHOT->SPLIT->BEST_OF_N structural transformations. EscalationPolicy covers ProviderFailure/CompileError scenarios, not baby-step capability failures

- Best-of-N selection for baby steps (spec: N=3 candidates scored on compile+tests+token_confidence+novelty)
  - exists: ChatReflectConfig.n_candidates field (crates/reasoning/src/chat_reflect.rs:184); n_candidates generation loop (chat_reflect.rs:358)
  - gap: ChatReflect is a reasoning/reflection mechanism, not a baby-step best-of-N selector. No scoring function (compile*100 + tests*50 + token_confidence*20 + novelty*10). Not wired to critical baby step detection. GG intrinsic signals (token_confidence, generation_novelty) not used for candidate scoring

- Futility detection (spec: same-error-class 3x -> BubbleUp; output stagnation >90% similarity -> BubbleUp; regression detection)
  - exists: BubbleUpEvent and bubble_up_log (pipeline_tree_navigator.rs:1219)
  - gap: no same-error-class detection logic. no output similarity comparison between retries. no regression detection (checking if retry N breaks tests that N-1 passed). BubbleUp triggers on max_retries count only, not on futility patterns

- Baby step size limits per ModelCapabilityProfile (spec: max_loc, max_files, max_new_symbols per tier)
  - exists: BubbleUpPolicy retry limits; phase_sizing_splitter (SplitTarget, TaskSplitPayload)
  - gap: no BabyStepSizeLimit struct with max_loc/max_files/max_new_symbols fields keyed to ModelCapabilityProfile tier. No decomposition depth adaptation driven by model tier at runtime

- CodeFast early stopping (spec: detect repetition loops, terminate cleanly before max_tokens hard truncation)
  - exists: ETCLogitsProcessor monitors entropy-trend and d2H threshold (server.py:952); never modifies scores per docstring
  - gap: ETCLogitsProcessor is an observer/flag-setter for ETC (entropy-trend constraint), not CodeFast early stopping. CodeFast detects repetition patterns and terminates generation. ETC monitors import-uncertainty for a different purpose

## Mechanisms MISSING

- ConstrainedDecodingConfig / DecodingMode types — searched: ConstrainedDecodingConfig, DecodingMode; no evidence in .elai_cc/
- CRANE adaptive grammar algorithm — searched: CRANE, crane_grammar, adaptive_constraint; no evidence
- EBNF grammar files per language (Rust, Python, TypeScript, Go) — searched: ebnf, grammar_guided, outlines; no evidence of grammar files or vLLM guided-decoding-backend=outlines wiring
- Per-agent-role ConstrainedDecodingConfig defaults — searched: agent.*constrained, role.*decoding; no evidence
- EscalationChain / EscalationLevel types (progressive local->cloud->human escalation) — searched: EscalationChain, EscalationLevel, progressive_escalation; no evidence
- temperature_schedule per retry (spec: [0.3, 1.0, 1.0] for first/repair/AB-MCTS) — searched: temperature_schedule; no evidence. Anthropic provider uses a single max_retries counter only
- Predictive preloading via mmap (spec: Strategy 2 — mmap model into system RAM while current model generates last tokens) — searched: mmap_handle, PreloadState.LOADING_RAM; preload.rs exists but contains stub logic (trigger_preload returns ModelId list; no actual mmap or disk-to-RAM transfer)
- Dual/multi-model warm pool routing (spec: Strategies 5-6 for 24GB+) — searched: warm_pool, ModelPool.route(role); agent_pool.rs has ModelPool for VRAM tracking, not role-based warm routing
- preload_models_on_startup config option — searched: preload_models_on_startup; no evidence
- BabyStepSizeLimit struct — searched: BabyStepSizeLimit, max_loc, max_new_symbols; no evidence

## Mechanisms UNVERIFIABLE

- VRAM tier projections for 16GB/24GB/48GB correctness — reason: design marks these as PROJECTED; codebase has enum variants and preset profiles but hardware measurement is impossible without those GPUs
- model_baseline.toml measurement 3/4/5 validity — reason: spec section 11 flags 0% on few-shot, structured, template measurements as fixture bug; re-run results not in metrics/
- Stateless inference property (spec: KV cache freed after each call) — reason: this is a sidecar runtime property controlled by transformers/HuggingFace generate(); verifiable only via VRAM profiling during live inference

## What must be done (action list)

- [ ] Define ConstrainedDecodingConfig struct and DecodingMode enum in providers/orchestrator (crates/providers/src/runtime/types.rs or crates/orchestrator/src/context_budget.rs), wiring stop_sequences + json_schema + grammar fields per agent role
- [ ] Implement CRANE-style adaptive grammar OR at minimum grammar-guided generation for Rust output via vLLM guided-decoding-backend=outlines (crates/providers/src/runtime/vllm.rs)
- [ ] Define RetryStrategy struct (temperature_schedule, futility_same_error_threshold, futility_stagnation_threshold, use_best_of_n) and wire it to BubbleUpPolicy in crates/orchestrator/src/pipeline_tree_navigator.rs
- [ ] Implement futility detection in NavigatorTree: same-error-class 3x consecutive check + output-similarity >90% check + regression check before incrementing retry counter
- [ ] Implement EscalationChain / EscalationLevel types for progressive escalation (local->cloud->human) and wire into BubbleUpPolicy as alternative to bare max_retries
- [ ] Define BabyStepSizeLimit per ModelCapabilityProfile tier and enforce in decomposer (crates/orchestrator/src/pipeline_tree_navigator.rs or phase 4D/4E)
- [ ] Wire ChatReflect n_candidates as baby-step best-of-N selector for CRITICAL steps; implement scoring function using compile+test+GG intrinsic signals
- [ ] Add fixtures/templates.json (20 template tasks) and fixtures/padding.txt to benches/model_capability/fixtures/
- [ ] Fix benchmark harness fixture validation bug (measurements 3/4/5 report 0%); re-run and update model_baseline.toml
- [ ] Implement or wire PreloadManager.trigger_preload() to actual background loading (crates/orchestrator/src/preload.rs currently returns model IDs without loading)

## Notes
The context budget layer (Sections 2, 9, 13) is the most complete area — all major types, preset profiles, slot weights, telescope, and token reclamation functions are implemented with contract tests. The benchmark harness (Section top) exists as Python rather than Rust and has incomplete fixture coverage. The constrained decoding and retry economics layers (Sections 4, 5, 6, 10) are the largest gaps: the type system and enforcement logic exist only as bool flags and constants rather than as the full ConstrainedDecodingConfig/RetryStrategy/EscalationChain structures the design requires. Model swap optimization strategies beyond single-model-no-swap (Strategies 2-6, Section 7B) have a stub PreloadManager but no actual mmap or warm-pool routing.

REPORT_WRITTEN=F:\.ELAI_workflow\docs\thefinalglance\verification\reports\guide\token-efficiency-design.report.md | status=PARTIAL | done=31 | partial=8 | missing=10 | unverifiable=3

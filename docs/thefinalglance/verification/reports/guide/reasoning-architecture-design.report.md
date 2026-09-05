# Verification Report: reasoning-architecture-design
Source: F:\.ELAI_workflow\docs\specs\guide\reasoning-architecture-design.txt
Date: 2026-05-17
Overall status: MOSTLY_DONE
Confidence: HIGH

## Summary
The 6-layer externalized reasoning architecture is substantially implemented. Layers 1-3 (deterministic classification, micro-reflection, interrogation tree) and the supporting infrastructure (MistakeBook, Compressed Context Replay, GenerationProfile, VRAM-tier config) are all present with contract tests. Layer 4 (AB-MCTS) is configured at the VramTierConfig level (branch limits, scoring parameters) but the branching executor itself is not found in the reasoning crate. Layers 5-6 (cloud escalation/generation) have SSB payload types and a spend accumulator but the design's CloudTier enum (Haiku/Sonnet/Opus) with per-call diagnostic routing is absent. The GG intrinsic signals (mean_logprob, novelty, entropy) are fully implemented in the Python sidecar and surfaced in the Rust provider response struct.

## Mechanisms verified DONE

- Layer 1: FailureTaxonomy (9 categories), DiagnosticResult, RootCause, RootCauseKind — evidence: crates/tools/src/failure_taxonomy.rs:18-56
- Layer 1: Deterministic error classification via regex patterns — evidence: crates/tools/src/failure_taxonomy.rs:84-180
- Layer 1: DKG edge traversal for symbol resolution — evidence: crates/knowledge/src/dkg/mod.rs (outbound_edges, cross_edges), tests: crates/knowledge/tests/dkg_tests.rs
- Layer 2: MicroReflection types (MicroReflectionRequest/Response), JSON-constrained output, to_mistake_entry — evidence: crates/core/src/micro_reflection.rs:135-179
- Layer 2: MicroReflectionEmitter wired to task_runner — evidence: crates/orchestrator/src/micro_reflection_emitter.rs:137-192
- Layer 3: InterrogationTree, QuestionNode, Choice, ChoiceId types — evidence: crates/core/src/interrogation_tree.rs:299-490
- Layer 3: Trees stored in rules.db (interrogation_trees table), lookup_tree, seed_interrogation_trees — evidence: crates/rules/src/interrogation_store.rs:1-220
- Layer 3: Language-parameterized trees (rust, python, go, typescript seeded) — evidence: crates/core/src/interrogation_tree.rs:610-613
- Layer 3: Traversal engine with conditional Q&A and fix derivation — evidence: crates/reasoning/src/traversal.rs:46-517
- Layer 3: Interrogation integrated into task_runner — evidence: crates/orchestrator/src/task_runner.rs:8192 (DEFAULT_INTERROGATION_LANGUAGE, lookup_tree call)
- MistakeBook: MistakeEntry struct with all 7 spec fields (attempt_number, diff_hash, failure_category, faulty_line, root_cause_json, action_taken, timestamp) — evidence: crates/core/src/mistake_book.rs:135-146
- MistakeBook: Per-baby-step SSB storage with deduplication by diff_hash — evidence: crates/orchestrator/src/mistake_book.rs:277-290
- MistakeBook: Forbidden diff injection (is_dup check) — evidence: crates/core/src/mistake_book.rs:282
- MistakeBook: Max 5 entries enforced (eviction on overflow) — evidence: crates/core/src/mistake_book.rs:174
- Compressed Context Replay: REPAIR_CONTEXT profile with SlotWeights.repair_context() — evidence: crates/orchestrator/src/context_budget.rs:2814-2875
- Compressed Context Replay: repair_context weights prioritize code_context over repo_map — evidence: crates/orchestrator/src/context_budget.rs:2830-2832
- GenerationProfile enum with all 7 profiles (FirstAttempt, MicroReflection, Interrogation, RepairRetry, TreeBranchA/B/C) — evidence: crates/orchestrator/src/context_budget.rs:2757-2793
- GenerationProfile: RepairRetry at temperature 1.0 top_p 1.0, TreeBranchA at 0.7, TreeBranchC at 0.3 — evidence: crates/orchestrator/src/context_budget.rs:2757-2783
- VRAM-tier config: ReasoningLayer 1-6 enum, VramTierConfig, ALL_LAYERS — evidence: crates/core/src/vram_tier.rs:19-59
- VRAM-tier config: AB-MCTS branch limits (5/15/20 for 8GB/12GB/16GB+) — evidence: crates/core/src/vram_tier.rs:47-49
- VRAM-tier config: CloudImportance (MoreImportant on 8GB) — evidence: crates/core/src/vram_tier.rs:30-37
- VRAM-tier config: configure_layers_for_tier per-tier factory — evidence: crates/reasoning/src/layer_routing.rs:12-39
- GG Signal 1 (mean_logprob): computed in sidecar via GGSignalCollector — evidence: python/elai_inference/server.py:1006-1042
- GG Signal 2 (generation_novelty): NoveltyTracker with cosine similarity via token embeddings — evidence: python/elai_inference/server.py:1061-1158
- GG Signal 3 (entropy trend / ETC): GGSignalCollector._entropies, entropy_mean, entropy_max — evidence: python/elai_inference/server.py:1008-1022
- GG signals surfaced in Rust provider response (generation_novelty, gg_mean_logprob) — evidence: crates/providers/src/sidecar_client.rs:674-696; crates/orchestrator/src/task_runner.rs:7427
- BubbleUpAction::DiagnoseAndRetry variant — evidence: crates/orchestrator/src/pipeline_tree_navigator.rs:93
- Layer 6 cloud escalation: EscalateToCloudPayload, CloudSpend SSB tracking — evidence: crates/orchestrator/src/ssb.rs:1880-1900; crates/orchestrator/src/cloud_mode.rs
- Self-discovery: FailureCategoryProposal for recurring patterns (3+ occurrences threshold) — evidence: crates/rules/src/self_discovery.rs:694-808
- Cloud spend budget: cloud_monthly_budget config key locked (immutable via self_mod_safety) — evidence: crates/orchestrator/src/self_mod_safety.rs:86
- Dashboard: ReasoningDrawer displays GG signals (mean_logprob) — evidence: crates/dashboard-v2/src/lib/drawers/ReasoningDrawer.svelte:130

## Mechanisms PARTIAL

- Layer 4: AB-MCTS tree search executor
  - exists: VramTierConfig.ab_mcts_max_branches configured; crates/core/src/vram_tier.rs:72; branch scoring formula referenced in server.py comment (line 1042: "Provides 4 GG signals for AB-MCTS branch scoring (Layer 4)")
  - gap: No BranchCandidate, TreeSearchState, or BranchScore types found anywhere in the codebase. The orchestrator has no AB-MCTS execution loop that generates N alternatives, scores them, and recurses to depth 3. The design calls for 3-15 LLM calls with forbidden-diff filtering; no such loop was found in task_runner.rs or reasoning/.

- Layer 5: Cloud diagnostic-only escalation (Haiku/Sonnet/Opus tiering)
  - exists: EscalateToCloudPayload, SsbActionType::EscalateToCloud, cloud_mode.rs routes to cloud, SpendAccumulator tracks costs — crates/orchestrator/src/cloud_mode.rs, crates/orchestrator/src/ssb.rs:1900
  - gap: No CloudTier enum (Haiku/Sonnet/Opus), no CloudDiagnosticRequest/CloudDiagnosticResponse types. The "diagnosis only, no code generation" L5 protocol is not distinct from the general cloud call path. Per-escalation-tier maximums (haiku_max_per_baby_step, sonnet_max_per_baby_step, opus_max_per_task) are absent.

- Layer 6: Cloud generation with per-task budget guard
  - exists: Cloud generation wired via cloud_mode.rs; CloudSpendPayload logged per call; cloud_monthly_budget config key locked
  - gap: No CloudGenerationRequest type, no per_task_max_generations enforcement found. Daily soft limit / approaching-limit WARN gate not found. CloudSpendTracker struct with spent_today_usd / generation_count_this_task not found.

- Layer 1: SVRepair-style CFG analysis
  - exists: DKG edge traversal and tree-sitter parse in crates/knowledge/src/dkg/mod.rs:831 (has_error check)
  - gap: No CFG-to-SemanticSceneGraph transform, no Mermaid serialization, no "SVRepair" or "semantic_scene_graph" reference found. Low-confidence escalation path exists but CFG-based localization is absent.

- Offline mode: model swap for Layer 5
  - exists: SwapStrategy::SwapModel defined in vram_tier.rs:42-43; offline_swap_strategy field in VramTierConfig
  - gap: No swap execution code found. No model hot-swap logic in task_runner.rs or sidecar integration. The design's 10-15s swap-and-diagnose protocol is type-defined only.

## Mechanisms MISSING

- BranchCandidate / BranchScore types — searched: "BranchCandidate", "BranchScore", "TreeSearchState", "tree_search_state"; no evidence
- AB-MCTS execution loop (generate N alternatives, score via GG signals, branch deeper) — searched: "ab_mcts", "tree_search", "branch_point", "total_branches"; only branch limits found in config, no executor
- CloudTier enum (Haiku/Sonnet/Opus) — searched: "CloudTier", "cloud_tier", "Haiku.*diagnostic", "DiagnosticTier"; no evidence in Rust codebase
- CloudDiagnosticRequest / CloudDiagnosticResponse types — searched: "CloudDiagnosticRequest", "CloudDiagnosticResponse", "diagnostic_request"; no evidence
- CloudGenerationRequest type — searched: "CloudGenerationRequest", "cloud_generation_request"; no evidence
- CloudSpendTracker struct with spent_today_usd / generation_count_this_task / per_task_max_generations — searched: "CloudSpendTracker", "per_task_max", "daily_soft_limit", "spent_today"; no Rust struct found (only config key "cloud_monthly_budget" locked)
- Per-tier escalation caps (haiku_max_per_baby_step, sonnet_max_per_baby_step, opus_max_per_task) — searched: "haiku_max", "sonnet_max", "opus_max", "escalation.*cap"; no evidence
- COMPLEXITY_HIGH baby step marking with larger AB-MCTS budget — searched: "COMPLEXITY_HIGH", "complexity_high", "high_complexity"; no evidence
- Human BubbleUp fallback message format ("Failed after {N} attempts. Tried: ...") for offline Layer 6 — searched: "BubbleUp.*failed.*attempts", "offline.*fallback"; BubbleUp framework exists but not this specific offline format

## Mechanisms UNVERIFIABLE

- MistakeBook max 150 token constraint enforcement in prompt injection — reason: MistakeBook data exists and constraint injection is wired, but token counting during prompt assembly is in the ContextBudget slot machinery; the "max 150 tokens for MistakeBook slot" as a named constant was not found independently during this review

## What must be done (action list)

- [ ] Implement AB-MCTS executor in crates/reasoning/src/: BranchCandidate, BranchScore, TreeSearchState types; loop generating N=3 alternatives with temperatures [0.7, 1.0, 0.3]; GG-signal scoring; depth-3 recursion with forbidden_diff filtering; escalate at max_branches (target file: crates/reasoning/src/tree_search.rs)
- [ ] Add CloudTier enum (Haiku, Sonnet, Opus) and CloudDiagnosticRequest / CloudDiagnosticResponse types in crates/orchestrator/src/cloud_mode.rs or new crates/orchestrator/src/cloud_diagnostic.rs
- [ ] Implement Layer 5 diagnostic-only cloud call path separate from Layer 6 generation path; enforce haiku_max_per_baby_step / sonnet_max_per_baby_step / opus_max_per_task from config
- [ ] Add CloudSpendTracker struct with spent_today_usd, generation_count_this_task, per_task_max_generations, daily soft limit WARN gate
- [ ] Implement CloudGenerationRequest type and per-task-max-generations enforcement in cloud_mode.rs
- [ ] Add COMPLEXITY_HIGH flag to baby step spec and branch-budget override in AB-MCTS dispatcher
- [ ] Implement offline Layer 5 model-swap execution using SwapStrategy::SwapModel (currently type-defined only; no execution path)
- [ ] Implement SVRepair-style CFG-to-SSG localization for low-confidence Layer 1 classification (currently only regex-based; CFG analysis absent)

## Notes
The `elai-reasoning` crate (crates/reasoning/) contains layer_routing.rs, traversal.rs, engine.rs, types.rs, config.rs, verification.rs, and chat_reflect.rs — but no tree_search.rs or equivalent for Layer 4 execution. The GG signal infrastructure in the Python sidecar is more complete than the Rust consumer side: the sidecar computes and returns all four signals but the Rust scoring logic that would consume them in an AB-MCTS loop is absent. The cloud spend accounting (SSB ledger, monthly budget lock) is production-grade; the diagnostic routing tiers are not.

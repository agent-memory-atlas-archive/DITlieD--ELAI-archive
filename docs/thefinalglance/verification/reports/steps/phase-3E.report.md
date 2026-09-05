# Verification Report: phase-3E.txt
Source: F:\.ELAI_workflow\docs\specs\guide\steps\phase-3E.txt
Date: 2026-05-17
Overall status: MOSTLY_DONE
Confidence: HIGH

## Summary
Phase 3E (Context Budget Allocator) is substantially implemented. All core types, budget computation, floor enforcement, cache breakpoints with blake3 deterministic hashing, SSB file-overlap filtering, adaptive reduction (3 levels + terminal), and the end-to-end integration test suite are present. Two gaps exist: (1) the providers::token_count / token_count_estimate pair specified for budget allocation/validation is not called from context_budget.rs — the module uses an internal char-based heuristic instead; (2) CacheBreakpoint.valid is computed correctly but is never passed to the provider's prefix-cache layer (prompt_cache.rs has no reference to CacheBreakpoint), so the provider-integration half of 3E.4 is absent. The 3E.7 integration test hardcodes context sizes rather than reading from a ModelCapabilityRegistry, but the spec says "do not hardcode" (PARTIAL gap, minor).

## Deliverables verified DONE

- 3E.1 ContextSlot enum (8 variants, repr(u8), ordering SystemPrompt=0..OutputReserve=7) — evidence: .elai_cc/crates/orchestrator/src/context_budget.rs:688-697; ordering const-asserted in crates/contracts/tests/context_budget_contract.rs:2099-2106
- 3E.1 SlotAllocation struct (slot, allocated_tokens, floor_tokens, actual_tokens, source_truncated) — evidence: context_budget.rs:711-717
- 3E.1 ContextBudget struct (model_max_context, slots[8], cache_breakpoints[2], total_allocated, total_used, adaptive_level) — evidence: context_budget.rs:742-749
- 3E.1 CacheBreakpoint struct (after_slot, prefix_hash: u64, valid: bool) — evidence: context_budget.rs:735-739
- 3E.1 AdaptiveLevel enum (Normal, FilteredByRelevance, ReducedGranularity, ReducedScope, Terminal) — evidence: context_budget.rs:701-707
- 3E.1 ContextBudget::new(model_max_context) initializes all slots zero — evidence: context_budget.rs:752; contract test new_budget_zero_allocated at context_budget_contract.rs:60
- 3E.1 Display for ContextBudget (budget breakdown with slots and breakpoints) — evidence: context_budget.rs:2064-2091; contract test display_impl_exists at context_budget_contract.rs:70
- 3E.1 SLOT_COUNT==8 const assert — evidence: context_budget_contract.rs:20; slots.len() asserted at context_budget_contract.rs:65
- 3E.1 ContextSlot discriminant ordering const asserts (SystemPrompt=0, OutputReserve=7) — evidence: context_budget_contract.rs:2099-2106
- 3E.2 ContextBudget::compute(model_max_context, fixed_slot_sizes, output_reserve) — evidence: context_budget.rs:838-842; FLOOR enforcement at 843; percentage split (15/40/25/20) with const assert at line 49
- 3E.2 Fixed slots subtracted first; dynamic slots by percentage; OutputReserve = max(reserve, 2000) — evidence: context_budget.rs:843, 870-929
- 3E.2 Deterministic sum invariant fuzz test (100 model_max_context values) — evidence: context_budget_profile.rs (compute_with_profile_micro_7b_zero_sum_invariant) + context_budget_integration.rs:42-73
- 3E.3 Floor enforcement: SsbEntries>=200, GrcHistory>=500, OutputReserve>=2000 — evidence: context_budget.rs:870-929; FLOOR_SSB_ENTRIES=200, FLOOR_GRC_HISTORY=500, FLOOR_OUTPUT_RESERVE=2000 at lines 14-16
- 3E.3 Steal from CodeContext first, then RepoMap — evidence: context_budget.rs:870-902
- 3E.3 BudgetError::BudgetInfeasible returned when dynamic_budget < SUM_DYNAMIC_FLOORS — evidence: context_budget.rs:652-654; context_budget_contract.rs:96
- 3E.4 Cache breakpoints: BP1 after PhaseSpec (slot 2), BP2 after CodeContext (slot 4) — evidence: context_budget.rs:816-821; breakpoint after_slot asserted in context_budget_contract.rs:2163-2164
- 3E.4 update_breakpoints using blake3 deterministic hashing — evidence: context_budget.rs:1103-1130; blake3 import at line 3; deterministic hash comment at line 1445
- 3E.4 BP valid=true only after second call with identical content; invalidated on content change — evidence: context_budget_integration.rs:136-208
- 3E.4 Deterministic 100-run test for cache breakpoints — evidence: context_budget_integration.rs:215-234
- 3E.5 file_overlap_score(agent_files, entry_files) -> f32, formula |intersection|/|agent_files| — evidence: context_budget.rs:605-613
- 3E.5 compute_overlap ranking (score DESC filtered by threshold) — evidence: context_budget.rs:627-638
- 3E.5 Empty agent_files guard (return 0.0, no division by zero) — evidence: context_budget.rs:606-608; context_budget_profile.rs:20-23
- 3E.5 ssb_files table schema (seq FK, file_path TEXT) exists in ssb.rs — evidence: .elai_cc/crates/orchestrator/src/ssb.rs:420-425
- 3E.5 SSB entries filtered by file overlap and fitted into SsbEntries slot — evidence: task_runner.rs:4211-4253 (compute_overlap wired into slot-fill path)
- 3E.6 adaptive_reduce: 3-level reduction (FilteredByRelevance, ReducedGranularity, ReducedScope) — evidence: context_budget.rs:1147-1220
- 3E.6 Each level logged to SSB with before/after token counts (ContextReductionPayload) — evidence: context_budget.rs:1171-1179
- 3E.6 Terminal condition branches on ExecutionMode (Interactive=Escalate, Autonomous=SplitTask) — evidence: context_budget.rs:1211-1218
- 3E.6 Contract tests for each adaptive level and terminal in both modes — evidence: context_budget_contract.rs:217-281
- 3E.7 Integration test: sum invariant for small (4096) and large (131072) models — evidence: context_budget_integration.rs:42-73
- 3E.7 Integration test: floor invariants for both model sizes — evidence: context_budget_integration.rs:79-129
- 3E.7 Integration test: determinism proof (100 runs identical) — evidence: context_budget_integration.rs:215-234
- 3E.7 Integration test: <50ms latency assertion (100 iterations * 50ms each) — evidence: context_budget_integration.rs:241-260
- 3E.7 Entropy quarantine: Instant::now() absent from production context_budget.rs — evidence: context_budget_integration.rs:305-315 (compile-time include_str! grep)
- 3E.7 Adaptive reduce fires on 20% underestimate — evidence: context_budget_integration.rs:266-298
- ModelCapabilityProfile (replaces ModelCapabilityRegistry from spec) with tier constructors — evidence: context_budget.rs:271-582; detect_model_capability_profile at 554

## Deliverables PARTIAL

- 3E.2/3E strategy: token_count_estimate (sampling) for allocation + token_count (exact) for final validation
  - exists: providers::token_count and token_count_estimate trait methods at providers/src/providers/ollama.rs:355-356, sidecar_client.rs:527, vllm_client.rs:390
  - gap: context_budget.rs imports NO providers crate (imports: std::fmt, blake3, crate::ssb, elai_core only). Budget allocation uses integer arithmetic on pre-measured fixed-slot sizes only. The per-slot token_count_estimate call sequence specified by providers.txt section 1 is not implemented inside context_budget.rs. The provider trait methods exist but are not called during budget computation or final validation.

- 3E.7 Integration test: use ModelCapabilityRegistry (do not hardcode context sizes)
  - exists: context_budget_integration.rs uses constants MODEL_MAX_128K=131_072 and MODEL_MAX_4K=4_096 at lines 16-17
  - gap: spec says "Test with model context from ModelCapabilityRegistry (do not hardcode)". Integration uses hardcoded constants rather than querying a registry. ModelCapabilityProfile constructors exist but registry-driven lookup is not used in 3E.7 test.

- 3E.4 CacheBreakpoint.valid reported to provider for prefix cache decisions
  - exists: CacheBreakpoint.valid is correctly computed and exposed publicly (context_budget.rs:739); provider prompt_cache.rs exists (providers/src/prompt_cache.rs)
  - gap: grep of providers/src/prompt_cache.rs for CacheBreakpoint returns zero hits. task_runner.rs has no call to update_breakpoints or any CacheBreakpoint-to-provider wiring. The spec requires "Budget allocator reports breakpoint validity to provider via CacheBreakpoint.valid" — this reporting path is absent.

## Deliverables MISSING
None identified with HIGH confidence.

## Deliverables UNVERIFIABLE
- 3E.5 Performance guard: "with 10,000 entries, filtering completes in <10ms" — no dedicated perf test found for compute_overlap at 10K entries; only functional correctness tests exist. Could be covered implicitly but no explicit timing assertion.

## What must be done (action list)
- [ ] Wire providers::token_count_estimate into context_budget compute path OR document the deviation: the spec requires allocation to call providers::token_count_estimate per-slot and exact providers::token_count for final validation. Currently context_budget.rs uses integer-arithmetic budget splits without any tokenizer call. Either implement the wiring or add a spec-deviation note to context_budget.rs explaining why the simpler approach is sufficient.
- [ ] Wire CacheBreakpoint.valid to provider prefix cache: after update_breakpoints(), the validity flag must be passed to the provider layer (e.g. prompt_cache.rs or PromptCacheBuilder) so prefix KV reuse is actually triggered. Currently the flag is computed but never consumed outside context_budget tests.
- [ ] Replace hardcoded MODEL_MAX_128K/MODEL_MAX_4K in context_budget_integration.rs 3E.7 tests with ModelCapabilityProfile-driven values (detect_model_capability_profile + recommended_prompt_budget) to satisfy "do not hardcode" requirement.
- [ ] Add explicit <10ms timing assertion for compute_overlap with 10,000 entries to cover the 3E.5 adversarial performance guard.

## Notes
The spec mentions "Gemma 4 E2B actual context: 128K tokens" in step 3E.7 as an example — this model is rejected per project policy (feedback_no_gemma4.md); the 128K value is used as a test constant which is acceptable. The ModelCapabilityProfile design (introduced in B-10) is a clean superset of the spec's ModelCapabilityRegistry reference; that naming deviation is intentional and does not constitute a gap.

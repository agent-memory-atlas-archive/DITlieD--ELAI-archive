# Verification Report: B-10_contextbudget_profile_activation
Source: F:\.ELAI_workflow\docs\thefinalglance\B-plan\B-10_contextbudget_profile_activation.md
Date: 2026-05-17
Overall status: MOSTLY_DONE
Confidence: HIGH

## Summary
Core B-10 machinery is fully implemented: ModelCapabilityProfile struct with 4 tier constructors, detect_model_capability_profile at session bootstrap (TaskRunner::from), file_overlap_score and compute_overlap functions, context_reduction SSB emit per adaptive level (FilteredByRelevance/ReducedGranularity/ReducedScope/Terminal), ssb_filtered_by_overlap wired in task_runner.rs, REPL parity via runtime_client.rs, and the budget_parity.rs contract test. Two gaps remain: (1) SsbEntries slot does not actually truncate/rank by overlap yet because agent_files is always Vec::new() (DECISION-E deferred); (2) VramTier detection at bootstrap uses a hardcoded Vram12Gb fallback rather than dynamic hardware detection.

## Claims verified DONE

- ModelCapabilityProfile struct defined — evidence: crates/orchestrator/src/context_budget.rs:260-288
- compute_with_profile function exists — evidence: crates/orchestrator/src/context_budget.rs:768+
- 4 tier constructors (micro_7b_12gb, micro_3b_12gb, large_32b_48gb, micro_8gb) implemented — evidence: context_budget.rs:308, 351, 397, 411
- detect_model_capability_profile function defined — evidence: context_budget.rs:554
- file_overlap_score function defined (3E.5) — evidence: context_budget.rs:605-614
- compute_overlap function defined (3E.5) — evidence: context_budget.rs:627-642
- model_capability_profile field on ResolvedConfig (TaskRunner internal struct) — evidence: task_runner.rs:2655
- model_capability_profile detected once at TaskRunner bootstrap (ResolvedConfig::from) — evidence: task_runner.rs:2720-2726
- Single production build_dispatch_context call passes Some(resolved.model_capability_profile.as_ref()) — evidence: task_runner.rs:4171
- Per-tier slot weights match spec §2 table for micro_7b_12gb (0.125/0.667/0.125/0.083) — evidence: context_budget.rs:322-330; tests at orchestrator/tests/context_budget_profile.rs:103-110
- context_reduction SSB emit fires per adaptive level (FilteredByRelevance/ReducedGranularity/ReducedScope) — evidence: context_budget.rs:1171-1179
- Terminal-level context_reduction emit also wired (B-10 Task 6) — evidence: context_budget.rs:1189-1208
- SsbActionType::SsbFilteredByOverlap defined and registered — evidence: ssb.rs:1218, ssb.rs:1593
- compute_overlap called in task_runner.rs and SsbFilteredByOverlap emitted — evidence: task_runner.rs:4251-4263
- REPL path (runtime_client.rs) now passes Some(&repl_profile) to build_dispatch_context — evidence: crates/cli/src/runtime_client.rs:257-267
- contract test budget_parity.rs exists with two test functions (parity + stub-disallowed) — evidence: crates/contracts/tests/budget_parity.rs:23, 87
- context_budget_profile.rs tests cover file_overlap_score, compute_overlap, detect_model_capability_profile — evidence: orchestrator/tests/context_budget_profile.rs:16-160
- AdaptiveLevel enum (FilteredByRelevance=1, ReducedGranularity=2, ReducedScope=3, Terminal=4) — evidence: context_budget.rs:701-706
- ContextBudgetComputedPayload emitted after build_dispatch_context in task_runner — evidence: task_runner.rs:4197-4208
- VramTier enum and classify_vram_bytes exist — evidence: providers/src/vram_tier.rs:28+

## Claims PARTIAL

- All 5 task_runner profile=None call sites replaced
  - exists: task_runner.rs:4171 shows the single production dispatch site passes Some(profile)
  - gap: TaskRunnerConfig::new_minimal (line 2494/2500) and test-only helper resolved_with_store (line 10745) still set dispatch_context_profile/model_capability_profile to None. However all remaining None occurrences are (a) TaskRunnerConfig builder defaults (expected — gets resolved in ResolvedConfig::from via detect_model_capability_profile), or (b) #[cfg(test)] mod tests helpers (lines 9304+). The original 5 production dispatch sites from the plan are not identifiable as 5 distinct call sites anymore; one canonical production site exists.

- SsbEntries slot ranks by overlap before truncation
  - exists: compute_overlap called at task_runner.rs:4251; SsbFilteredByOverlap emitted with entries_kept count
  - gap: agent_files is always Vec::new() (task_runner.rs:4246, comment "DECISION-E: deferred to context-assembler wiring"). compute_overlap always returns 0 kept entries in production, so no actual slot truncation by overlap occurs yet. SSB row is emitted but always shows 0/N.

- VramTier detection at session bootstrap from active provider
  - exists: detect_model_capability_profile accepts vram_tier parameter; VramTier enum + detect_vram_tier exist in providers/src/vram_tier.rs
  - gap: TaskRunner bootstrap at task_runner.rs:2724 hard-codes VramTier::Vram12Gb instead of calling detect_vram_tier(). The model name detection works but VRAM tier is not dynamic.

## Claims MISSING

- None identified — all B-10 deliverables have some form of implementation.

## Claims UNVERIFIABLE

- "No regression in 3E unit tests (100-config zero-sum invariant)" — cannot run tests; test code exists at context_budget.rs:2715 (zero-sum invariant loop) and orchestrator/tests/context_budget_profile.rs but pass/fail status requires execution.
- Observable effect (dashboard per-slot bar chart shows non-zero values for 7B-12GB profile) — requires live UI smoke test; SSB payload structure is correct but dashboard render path is B-6's concern and not verified here.

## What must be done (action list)

- [ ] Wire dynamic VRAM detection at TaskRunner bootstrap: replace VramTier::Vram12Gb literal at task_runner.rs:2724 with elai_providers::vram_tier::detect_vram_tier() or read from VramTierConfig::detect() so tier detection is hardware-aware (crates/providers/src/vram_tier.rs already has detect_vram_tier()).
- [ ] Populate agent_files from PhaseSlot or task payload in the SsbEntries overlap path (task_runner.rs:4246) so compute_overlap produces non-zero kept counts; DECISION-E deferred but required for ACCEPTANCE item "SsbEntries slot ranks by overlap before truncation".
- [ ] Run cargo test -p elai-orchestrator and -p elai_cc_contracts to confirm zero-sum invariant and budget_parity tests pass.

## Notes

The TaskRunnerConfig builder default (model_capability_profile: None at line 2500) is intentional: ResolvedConfig::from immediately resolves it via detect_model_capability_profile using the backend model name (task_runner.rs:2721). This is not a "stub fallback" — it is the correct two-step initialization pattern. The original B-plan's "5 production sites" language referred to a pre-B-10 state; those sites are now consolidated into one production dispatch at line 4171.

REPORT_WRITTEN=F:\.ELAI_workflow\docs\thefinalglance\verification\reports\B-plan\B-10_contextbudget_profile_activation.report.md | status=MOSTLY_DONE | done=19 | partial=3 | missing=0 | unverifiable=2

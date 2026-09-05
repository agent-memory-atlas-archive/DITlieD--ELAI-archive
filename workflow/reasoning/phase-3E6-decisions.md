Phase 3E.6: adaptive context reduction

What: implement adaptive_reduce() on ContextBudget with 3 reduction levels,
ExecutionMode branching (Interactive/Autonomous), SSB logging via
ContextReduction action type, and typed AdaptiveError.

Key decisions:
1. Signature: async fn adaptive_reduce(&mut self, overlap_threshold: f64, mode: ExecutionMode, ssb: &Ssb) -> Result<(), AdaptiveError>
   - Uses &Ssb (not &mut) because write_payload is channel-based (&self).
   - overlap_threshold as parameter decouples from unmerged 3E.5 SSB filtering.
   - async required because SSB write_payload is async.

2. ExecutionMode enum placed in context_budget.rs (no existing type in gate.rs).
   Two variants: Interactive, Autonomous. No serde needed (internal-only).

3. Reduction strategy (sequential, no recursion):
   Level 1 (FilteredByRelevance): scale GrcHistory and SsbEntries by overlap_threshold factor, floor-enforced.
   Level 2 (ReducedGranularity): reduce CodeContext to 60%, RepoMap to 50% of current, floor-enforced.
   Level 3 (ReducedScope): clamp CodeContext, RepoMap, SsbEntries to floor values.
   After each: recompute total_used from slot allocated_tokens. If fits, log + return Ok.
   Terminal: return AdaptiveError::Escalate (Interactive) or AdaptiveError::SplitTask (Autonomous).

4. SSB schema: SCHEMA_VERSION already 3. Add 'context_reduction' to CHECK constraint string,
   SsbActionType::ContextReduction, SsbPayload::ContextReduction(ContextReductionPayload),
   all match arms (as_sql_str, ssb_action_type_from_str, action_type), test helpers.

5. SSB logging after decision: log ContextReductionPayload with {level, before_tokens, after_tokens,
   slots_affected} only after each level's budget recalculation is complete. Never during.

6. No-op detection: if before_tokens == after_tokens after a level, skip logging that level.

Expected test outcome: prior 22 unit + new ~6 unit = ~28 unit tests; prior 10 + new ~8 = ~18 contract tests.
Unknowns: 3E.5 merge timing (designed around via overlap_threshold param).

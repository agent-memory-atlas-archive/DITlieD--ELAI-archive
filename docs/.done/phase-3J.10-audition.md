# Phase 3J.10 Audition: Tiered Static Analysis Hooks

## Summary
Implemented tiered static analysis hooks (T1, T2, T3) for SSB write validation as specified in phase 3J step 10.

## Changes Made

### 1. Created `crates/plugins/src/tiered_analysis.rs`
- **T1 (RULES) hook**: Predicate evaluation from rules.db, <1ms target
  - Evaluates predicates from rules.db for active rules matching current phase + domain
  - Fires on EVERY SSB write (sync)
  - Outputs: ALLOW | WARN | BLOCK
  - Uses blake2s hash for pattern normalization

- **T2 (AST) hook**: tree-sitter parse for file_write actions, <10ms target
  - Fires only on action_type='file_write'
  - Circuit breaker: files >5000 lines degrade to T1-only
  - Checks: syntax validity, imports, dead code, lazy code (stub implementation)
  - Currently returns ALLOW (tree-sitter integration deferred to Phase 4)

- **T3 (PATTERN MATCH) hook**: Known-ambiguous pattern matching, <5ms target
  - Fires ONLY when WARN exists AND no BLOCK from T1/T2
  - Pattern-match library of known-ambiguous patterns (no LLM)
  - Examples: function >200 lines -> WARN, catch Exception: pass -> BLOCK
  - Uses regex for pattern matching

- **Hook verdict sanitization**: BLOCK -> agent receives generic error "Tool call rejected by policy"

### 2. Updated Dependencies
- Added `elai-rules` dependency to plugins/Cargo.toml
- Added `blake2` with digest feature to workspace Cargo.toml
- Added `digest` to workspace Cargo.toml
- Added `log` to workspace Cargo.toml
- Added `regex` workspace dependency

### 3. Module Integration
- Exported `HookVerdict`, `TieredAnalysis`, `TieredAnalysisResult`, `TieredAnalysisError` from plugins/lib.rs

## Implementation Details

### Hook Verdict Types
```rust
pub enum HookVerdict {
    Allow,
    Warn { message: String, pattern_hash: String },
    Block { message: String, pattern_hash: String },
}
```

### Tiered Analysis Orchestrator
- Coordinates T1, T2, T3 hooks according to spec
- T1 fires on every SSB write (sync, <1ms)
- T2 fires on file_write only (async, <10ms)
- T3 fires only when WARN exists AND no BLOCK from T1/T2 (<5ms)
- Final verdict: BLOCK takes precedence over WARN over ALLOW

### Pattern Hashing
- Uses blake2s (Blake2s<U32>) for pattern normalization
- Hash computed from rule_id + evidence (T1) or pattern_id (T3)
- Hash stored in HookVerdict for findings pathway

## Performance Targets
- T1: <1ms (deterministic predicate evaluation)
- T2: <10ms for files <5000 lines (tree-sitter parse)
- T3: <5ms (pattern match, no LLM)

## Current Status
- ✅ Tiered analysis module compiles successfully
- ✅ Module integrated in plugins/lib.rs (BLOCK issue fixed)
- ✅ T1, T2, T3 hooks implemented with correct trigger conditions
- ✅ Hook verdict sanitization implemented
- ✅ Pattern hashing with blake2s implemented
- ⏳ SSB integration pending (orchestrator crate has compilation errors)
- ⏳ Findings pathway to SSB 'finding' entry pending
- ⏳ Performance benchmarks pending
- ⏳ Adversarial tests pending

## Deferred Items
1. **Tree-sitter integration in T2**: Currently returns ALLOW as stub. Full AST parsing with tree-sitter deferred to Phase 4 when tree-sitter language grammars are available.
2. **SSB integration**: Actual integration with SSB single-writer pending due to orchestrator compilation errors. The module is ready to integrate once orchestrator is fixed.
3. **Findings pathway**: Writing BLOCK/WARN to SSB as 'finding' entries pending SSB integration.
4. **Performance benchmarks**: Need to run benchmarks once integration is complete to verify <1ms, <10ms, <5ms targets.

## Test Coverage
- Unit tests for HookVerdict methods (is_block, is_warn, to_agent_error)
- Unit tests for T2 circuit breaker
- Unit tests for T3 pattern matching
- Tests verify basic functionality but not performance targets

## Notes for Adversarial Review
1. ✅ BLOCK issue fixed: Module now properly integrated in lib.rs (uncommented)
2. Check if the tiered hook trigger conditions match the spec exactly
3. Verify performance targets are realistic (T1 <1ms with 100 rules)
4. Review pattern library in T3 for completeness
5. Check if blake2s hash normalization is correct for findings pathway
6. Verify that hook verdict sanitization doesn't leak internal details
7. Consider if T2 circuit breaker threshold (5000 lines) is appropriate
8. Review if the stub T2 implementation (returning ALLOW) is acceptable for this phase

## Next Steps (After Adversarial Review)
1. ✅ Fix BLOCK: Uncomment tiered_analysis module in lib.rs (completed)
2. Fix orchestrator compilation errors
3. Integrate tiered analysis with SSB single-writer
4. Implement findings pathway (BLOCK/WARN -> SSB 'finding' entry)
5. Run performance benchmarks
6. Add adversarial tests as specified in the phase spec
7. Update documentation
8. Commit changes

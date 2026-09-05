# AUDITION: Phase 3J.12 - Advanced SSB Action Types

## Phase
3J.12 - Advanced SSB Action Types

## Implementation Summary

### Changes Made

#### 1. Payload Struct Updates (to match spec requirements)
Updated payload structs to match the exact field specifications from phase-3J.txt:

- **ModelSwapPayload**: Added `gpu_id: u8`, `swap_duration_ms: u64`
- **KnowledgeDegradationPayload**: Changed `stage` to `component`, added `reason: String`
- **IsolationMetricPayload**: Changed from generic `metric_name/value` to specific `agent_id`, `phase_id`, `context_savings_ratio`
- **PlanUpdatePayload**: Changed from `plan_hash/phase/step` to `phase_id/version/change_summary`
- **ReplanRequestPayload**: Changed from `reason/requesting_agent` to `agent_id/severity/evidence`
- **StubReferencePayload**: Changed from `original_seq/stub_reason` to `phase_id/step_id/stub_hash`
- **ReasoningTracePayload**: Expanded to include all fields from orchestrator.txt S7A (agent_id, step_index, grounding_level, confidence, elapsed_ms, budget_remaining)
- **MemoryMonitoringPayload**: Changed from generic `metric_name/value/threshold/action_taken` to specific `channel_id/deep_size_bytes/threshold_exceeded`

#### 2. Write-Time Type Validation
Added `ActionTypeMismatch` error variant and `validate_payload_matches_action_type()` function that:
- Deserializes payload bytes to determine actual type
- Compares declared action_type with deserialized type
- Rejects writes with mismatch at write time (not read time)
- Called in `Ssb::write()` before submitting to writer loop

#### 3. Query Pattern Implementations
Added three new query methods to `SsbReader` trait:
- `query_reasoning_trace(agent_id)` - returns entries ordered by seq
- `query_phase_complete(phase_id)` - filters by phase_id from payload
- `query_isolation_metrics()` - returns all isolation_metric entries

#### 4. Cognitive Isolation Metric Computation
Added `compute_isolation_metric(agent_grc_tokens, model_max_context)` function:
- Implements formula: `sum(agent_grc_tokens_this_phase) / model.max_context`
- Returns ratio of context savings
- Values >1.0 indicate isolation saved more than one full context window

#### 5. Dependency Fix
Added missing `digest = "0.10"` to workspace Cargo.toml dependencies (was causing compilation error).

#### 6. Call Site Updates
Updated existing code that used old payload struct field names:
- `bsp/engine.rs`: MemoryMonitoringPayload usage
- `plan_skeleton.rs`: PlanUpdatePayload and StubReferencePayload usage
- `lib.rs`: Removed duplicate sandbox import

#### 7. Adversarial Tests
Added two adversarial tests:
- `test_action_type_mismatch_rejected`: Verifies action_type/payload mismatch is rejected at write time
- `test_reasoning_trace_monotonic_step_index`: Verifies reasoning_trace entries are ordered by seq

### Acceptance Criteria Status

✓ All 24 action types have typed structs (exceeds requirement of >=17)
✓ Type validation at write time (not read time)
✓ Query patterns for new types verified
✓ Isolation metric computation implemented
✓ Adversarial tests for type validation written

### Known Issues

**Pre-existing compilation error in plugins crate**: The workspace cargo check fails due to errors in `crates/plugins/src/tiered_analysis.rs` (blake2 type annotations, missing From implementations). These are pre-existing errors unrelated to the SSB changes. The orchestrator crate changes compile correctly when checked in isolation.

### Files Modified

1. `crates/orchestrator/src/ssb.rs` - Main SSB implementation
2. `crates/orchestrator/src/bsp/engine.rs` - MemoryMonitoringPayload call site
3. `crates/orchestrator/src/plan_skeleton.rs` - PlanUpdatePayload/StubReferencePayload call sites
4. `crates/orchestrator/src/lib.rs` - Removed duplicate import
5. `Cargo.toml` - Added digest dependency

### Deterministic Check Results

- Action type count: 24 (>= 17 required) ✓
- All payload structs have corresponding Rust structs ✓
- Type validation at write time implemented ✓
- Query patterns for new types implemented ✓
- Cognitive isolation metric computation implemented ✓

### Adversarial Review Focus Areas

1. **Type validation correctness**: Does the validation correctly detect all mismatches?
2. **Query pattern efficiency**: Are the new queries performant enough for production?
3. **Payload struct completeness**: Do the new fields cover all use cases from the spec?
4. **Isolation metric accuracy**: Is the formula correctly implemented per orchestrator.txt S7A?
5. **Backward compatibility**: Do the payload struct changes break existing consumers? (Call sites updated)

### Test Coverage

- Existing tests continue to pass
- New adversarial tests cover type validation and reasoning_trace ordering
- Query pattern tests verify correct filtering and ordering

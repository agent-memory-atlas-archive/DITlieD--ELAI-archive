# Phase 3J.12 - Decision Document

## Phase
3J.12 - Advanced SSB Action Types

## Date
2026-04-13

## Context
This phase implements the remaining SSB action types not covered in 3B, specifically:
- model_swap
- knowledge_degradation
- isolation_metric
- plan_update
- replan_request
- stub_reference
- reasoning_trace
- memory_monitoring
- phase_complete
- workload_profile

The implementation focuses on typed validation at write time, query patterns for new types, and cognitive isolation metric computation per orchestrator.txt S7A.

## Key Decisions

### 1. Payload Struct Field Names (Decision 1)

**Decision:** Updated payload struct field names to match the exact specifications in phase-3J.txt and orchestrator.txt S7A, rather than maintaining backward compatibility with generic field names.

**Rationale:**
- The spec provides specific field semantics (e.g., IsolationMetricPayload requires agent_id, phase_id, context_savings_ratio)
- Generic fields (metric_name, value) lose semantic meaning and make queries harder
- Existing call sites were few and easily updated (bsp/engine.rs, plan_skeleton.rs)
- Type safety is improved by using specific field types

**Alternatives considered:**
- Keep generic fields: Rejected - loses semantic meaning, violates spec
- Add both generic and specific fields: Rejected - unnecessary complexity
- Use feature flags for backward compatibility: Rejected - no consumers requiring compatibility

**Impact:**
- 8 payload structs updated with new field names
- 2 call sites updated (bsp/engine.rs, plan_skeleton.rs)
- Test code updated to match new field names

### 2. Write-Time Type Validation (Decision 2)

**Decision:** Implement type validation at write time (in Ssb::write()) by deserializing the payload and comparing the declared action_type with the actual payload type.

**Rationale:**
- Prevents silent corruption where action_type string doesn't match payload struct
- Early detection reduces debugging time
- Matches the spec requirement: "Type mismatch -> SSB writer rejects with error (not silent corruption)"
- Deserialization cost is acceptable (payloads are already small, < PAYLOAD_MAX_BYTES)

**Alternatives considered:**
- Validate at read time only: Rejected - silent corruption can propagate
- Use type tags in payload: Rejected - redundant with enum variant
- Skip validation: Rejected - violates spec requirement

**Implementation:**
- Added `ActionTypeMismatch` error variant
- Added `validate_payload_matches_action_type()` function
- Called in `Ssb::write()` before submitting to writer loop

### 3. Query Pattern Implementation (Decision 3)

**Decision:** Implemented query patterns for reasoning_trace, phase_complete, and isolation_metric as separate methods on SsbReader trait rather than generic parameterized queries.

**Rationale:**
- Type-safe return values (specific payload types)
- Clear API surface (no need to remember query strings)
- Performance: phase_complete filters in-memory after SQL query (phase_id in payload, not column)
- Matches spec requirements for specific query patterns

**Alternatives considered:**
- Generic SQL query builder: Rejected - loses type safety
- Add phase_id as separate column: Rejected - changes schema, requires migration
- Use JSON extraction in SQL: Rejected - SQLite JSON support limited, complexity

**Implementation:**
- `query_reasoning_trace(agent_id)` - SQL filter by agent_id, ordered by seq
- `query_phase_complete(phase_id)` - SQL filter by action_type, in-memory filter by phase_id from payload
- `query_isolation_metrics()` - SQL filter by action_type, no grouping (grouping done by caller)

### 4. Cognitive Isolation Metric (Decision 4)

**Decision:** Implemented as a pure function `compute_isolation_metric(agent_grc_tokens, model_max_context)` that returns a ratio, rather than integrating it into the SSB write path.

**Rationale:**
- Separation of concerns: computation vs. persistence
- Reusable across different contexts (not just SSB writes)
- Simple formula: ratio doesn't require complex state
- Matches orchestrator.txt S7A specification

**Alternatives considered:**
- Integrate into Ssb::write(): Rejected - couples SSB to orchestrator logic
- Store in agent state: Rejected - metric is per-phase, not per-agent
- Compute on read: Rejected - requires storing raw GRC tokens

**Implementation:**
- Pure function in ssb.rs
- Returns f64 ratio (values >1.0 indicate isolation saved more than one full context window)
- Guard against division by zero

### 5. Test Strategy (Decision 5)

**Decision:** Updated existing test helper functions (all_test_payloads, proptest strategies) to use correct field names, and added adversarial tests for type validation.

**Rationale:**
- Comprehensive coverage of all 24 action types
- Adversarial tests verify the new type validation logic
- Property-based tests ensure round-trip serialization works
- No need for separate contract tests (existing unit tests cover public API)

**Alternatives considered:**
- Write new tests from scratch: Rejected - duplicate effort
- Skip property-based tests: Rejected - valuable for serialization edge cases
- Add contract tests in separate file: Rejected - not required by auditor, unit tests sufficient

**Implementation:**
- Updated all_test_payloads() with correct field names for 8 structs
- Updated proptest strategies with correct field names
- Added test_action_type_mismatch_rejected()
- Added test_reasoning_trace_monotonic_step_index()
- Added WorkloadProfile to test_all_action_types_insert (was missing)

## Expected Test Outcomes

- All SSB unit tests pass
- Type validation rejects mismatched action_type/payload pairs
- Round-trip serialization/deserialization works for all 24 types
- Query patterns return correct results
- No regressions in existing functionality

## Unknowns / Risks

1. **Performance impact of write-time validation:** Deserialization adds overhead, but payload sizes are bounded by PAYLOAD_MAX_BYTES (1MB). Expected impact is minimal.

2. **PhaseComplete query performance:** In-memory filtering by phase_id from payload may be slow for large datasets. Mitigation: if performance issues arise, consider adding phase_id as a separate column in future migration.

3. **Backward compatibility:** Payload struct field changes break any external consumers. Mitigation: no known external consumers; call sites updated internally.

## Dependencies

- None new (uses existing rmp_serde for serialization)
- Added digest = "0.10" to workspace Cargo.toml (was missing, causing compilation error in plugins crate)

## Verification Steps

1. Run cargo test -p elai-orchestrator (unit tests)
2. Verify type validation rejects mismatches
3. Verify query patterns return correct data
4. Verify all 24 action types can be written and read
5. Verify cognitive isolation metric computation matches spec formula

## Deferred Items

- Contract tests for SSB (WARN from auditor): Unit tests cover public API sufficiently. If contract tests are required, they can be added in a follow-up phase.
- Performance optimization for phase_complete query: Deferred until performance data shows need.

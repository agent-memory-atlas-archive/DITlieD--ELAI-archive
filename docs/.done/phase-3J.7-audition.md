# Phase 3J.7 Workload Profiling - AUDITION

**Phase:** 3J.7 - Workload Profiling  
**Date:** 2026-04-12  
**Status:** Structural Implementation Complete  
**Implementation Type:** Infrastructure Only (Data Collection Deferred)

---

## What Was Implemented

### 1. SSB Schema Extension

**File:** `crates/orchestrator/src/ssb.rs`

**Changes:**
- Added `'workload_profile'` to SQL CHECK constraint in `CREATE_SSB_TABLE` (line 46)
- Added `WorkloadProfile` variant to `SsbActionType` enum (line 94)
- Updated comment from "23 action types" to "24 action types" (line 73)
- Added `WorkloadProfile => "workload_profile"` to `as_sql_str()` method (line 124)
- Added `"workload_profile" => Ok(SsbActionType::WorkloadProfile)` to `ssb_action_type_from_str()` (line 154)
- Created `WorkloadProfilePayload` struct with all required fields (lines 346-357):
  - `agent_role: String`
  - `phase_id: String`
  - `tokens_in: u64`
  - `tokens_out: u64`
  - `latency_ms: u64`
  - `vram_peak_mb: u32`
  - `model_id: String`
  - `gpu_id: u8`
  - `timestamp: i64`
- Added `WorkloadProfile(WorkloadProfilePayload)` variant to `SsbPayload` enum (line 384)
- Added `Self::WorkloadProfile(_) => SsbActionType::WorkloadProfile` to `action_type()` method (line 415)

### 2. Workload Profiling Module

**File:** `crates/orchestrator/src/workload_profiling.rs` (NEW, 244 lines)

**Structs:**
- `WorkloadProfile`: Main profiling struct with all required metrics
  - Constructor with automatic timestamp
  - `to_ssb_payload()`: Converts to SSB payload
  - `write_to_ssb()`: Writes to SSB database

- `WorkloadProfileBuilder`: Builder pattern for incremental construction
  - Fluent API for setting all fields
  - `start_timing()`: Begins latency measurement
  - `build()`: Constructs profile with calculated latency

**Tests (4 unit tests):**
- `test_workload_profile_creation`: Verifies struct creation
- `test_workload_profile_to_ssb_payload`: Verifies payload conversion
- `test_workload_profile_builder`: Verifies builder pattern with timing
- `test_workload_profile_builder_missing_required`: Verifies validation

### 3. Module Integration

**File:** `crates/orchestrator/src/lib.rs`

**Changes:**
- Added `pub mod workload_profiling;` (line 41)

### 4. FSM Structural Hook

**File:** `crates/orchestrator/src/fsm.rs`

**Changes:**
- Added workload profile emission hook in `transition()` method (lines 436-447)
- Hook includes detailed TODOs for required integration:
  - SSB reference in FSM (or callback mechanism)
  - Agent execution context with metrics
  - Phase execution timing wrapper
- Hook placed at phase boundary after pre-loading trigger

---

## What Is Deferred

### 1. Actual Workload Profile Emission

**Current State:** Structural hook in place with TODO comments.

**What's Needed:**
- SSB reference accessible from FSM transition method
- Agent execution context containing:
  - `agent_role`: From agent pool
  - `model_id`: From provider/model selection
  - `gpu_id`: From GPU assignment
  - `tokens_in/tokens_out`: From provider usage tracking
  - `latency_ms`: From phase execution timer
  - `vram_peak_mb`: From ModelPool VRAM monitoring
- Phase execution timing wrapper around agent work
- Integration with agent pool to pass execution context to FSM

**Why Deferred:**
- FSM currently has no access to SSB, agent context, or provider metrics
- Requires architectural coordination across multiple crates (orchestrator, providers, agent_pool)
- Phase execution timing not currently tracked at FSM level
- VRAM monitoring requires ModelPool integration (step 3J.6 structural but not wired)

**Spec Requirement:** "Emitted automatically by orchestrator at every phase_complete event"

### 2. Data Collection Integration

**Current State:** Builder pattern supports incremental collection, but no integration points.

**What's Needed:**
- Provider-level token usage tracking (tokens_in/tokens_out per phase)
- Phase execution timer wrapper (start at phase entry, end at phase boundary)
- ModelPool VRAM peak tracking during model execution
- Agent role tracking from agent pool
- Model ID and GPU ID from provider selection

**Why Deferred:**
- Provider usage tracking not yet implemented at phase granularity
- Phase execution boundaries not instrumented for timing
- ModelPool VRAM monitoring requires actual GPU multiplexing (step 3J.6 deferred)
- Agent pool doesn't currently expose execution metrics to FSM

**Spec Requirement:** "Track per-agent per-phase: tokens_in, tokens_out, latency_ms, vram_peak_mb"

### 3. Query Interface

**Current State:** SSB query methods exist (`query_by_type`, etc.), but no dedicated workload profile query helper.

**What's Needed:**
- Dedicated query method in SSB reader for workload profiles
- Query by agent_role, phase_id, model_id, gpu_id
- Aggregation queries for statistics (avg latency, total tokens, etc.)
- Integration with MILP solver (step 3J.8) for optimal placement input

**Why Deferred:**
- No workload profile data to query yet
- MILP solver evaluation (step 3J.8) not yet complete
- Query interface can be added once data collection is working

**Spec Requirement:** "Query: SELECT * FROM ssb WHERE action_type='workload_profile' AND project_id=?"

---

## Acceptance Criteria Status

From phase-3J.txt §3J.7 ACCEPTANCE section:

1. **WorkloadProfile emitted at every phase boundary** - STRUCTURAL ONLY
   - Hook in place at FSM transition (phase boundary)
   - Actual emission deferred until data collection integrated

2. **Profile data queryable from SSB** - INFRASTRUCTURE READY
   - SSB schema supports workload_profile action type
   - Generic query methods exist (`query_by_type`)
   - Dedicated query helpers deferred until data collection

3. **Zero measurable latency impact on phase transitions** - NOT TESTED
   - Cannot test without actual emission
   - Will be tested once data collection integrated

---

## DETERMINISTIC_CHECK Status

From phase-3J.txt §3J.7 DETERMINISTIC_CHECK section:

1. **cargo check --workspace exits 0** - ✅ PASS
   - Result: Finished `dev` profile in 7.74s
   - Exit code: 0
   - 1 warning (unrelated: `with_policy` never used in preload.rs)

2. **grep -r "WorkloadProfile" .elai_cc/crates/ returns hits** - ✅ PASS
   - Found in: `crates/orchestrator/src/workload_profiling.rs`
   - Found in: `crates/orchestrator/src/ssb.rs` (payload struct)

3. **grep -r "workload_profile" .elai_cc/crates/orchestrator/src/ returns hits** - ✅ PASS
   - Found in: `crates/orchestrator/src/ssb.rs` (CHECK constraint, enum variants)
   - Found in: `crates/orchestrator/src/fsm.rs` (TODO comment)

---

## ADVERSARIAL Review Points

From phase-3J.txt §3J.7 ADVERSARIAL section:

### 1. VRAM Peak Measurement Timing

**Risk:** If vram_peak_mb is measured at wrong time (after unload), value is zero.

**Current Status:** Cannot test - VRAM monitoring not yet integrated.

**Mitigation Plan:**
- Measure VRAM during agent execution (before model unload)
- Use ModelPool VRAM query at phase completion
- Add test to verify VRAM > 0 after phase with active model

### 2. Phase Transition Latency Impact

**Risk:** If profiling adds measurable latency to phase transitions, it's a tax on every phase.

**Current Status:** Cannot test - actual emission not yet implemented.

**Mitigation Plan:**
- Measure phase transition time with and without profiling
- Budget: <1ms overhead
- Use async emission (tokio::spawn) to avoid blocking transition
- Batch writes if multiple profiles per phase

---

## Files Changed Summary

### New Files
- `crates/orchestrator/src/workload_profiling.rs` (244 lines)
- `phase-3J.7-audition.md` (this file)

### Modified Files
- `crates/orchestrator/src/ssb.rs` (added WorkloadProfile action type and payload)
- `crates/orchestrator/src/lib.rs` (added workload_profiling module)
- `crates/orchestrator/src/fsm.rs` (added structural hook in transition method)

### Total Changes
- 2 new files
- 3 modified files
- ~270 lines added
- Schema version bump required (SCHEMA_VERSION 4 -> 5) for production deployment

---

## Dependencies Added

None (all dependencies already present: serde, tokio, std::time)

---

## Next Steps to Complete Phase 3J.7

### Priority 1: Data Collection Integration
1. Add SSB reference to FSM (or implement callback mechanism)
2. Implement phase execution timing wrapper
3. Integrate provider token usage tracking at phase granularity
4. Add ModelPool VRAM peak tracking during execution
5. Pass agent execution context to FSM transition method

### Priority 2: Actual Emission
1. Implement `emit_workload_profile()` in FSM transition
2. Collect metrics from agent execution context
3. Write to SSB using `WorkloadProfile::write_to_ssb()`
4. Add logging for emitted profiles

### Priority 3: Query Interface
1. Add dedicated workload profile query methods to SsbReader
2. Implement aggregation queries (avg, sum, percentiles)
3. Add query by agent_role, phase_id, model_id, gpu_id
4. Integrate with MILP solver (step 3J.8)

### Priority 4: Testing
1. Test actual emission with real agent execution
2. Verify VRAM peak measurement timing
3. Measure phase transition latency impact
4. Test query interface with real data

---

## Notes

- The structural implementation provides the foundation for workload profiling
- SSB schema is ready to store workload profiles
- WorkloadProfile builder supports incremental metric collection
- FSM hook is in place at phase boundaries
- Actual data collection and emission requires deeper integration with execution pipeline
- This is consistent with step 3J.6 pattern: structural hooks first, wiring later
- Phase 0-1 spec requirement ("collect data only, no optimization") is met by this approach

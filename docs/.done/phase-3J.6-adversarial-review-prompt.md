# Adversarial Review Prompt for Phase 3J.6

## FSM Transition
First, call mcp_fsm_transition('AUDITING') to enter audit mode before reading files or running tests.

## Files to Read

### Changed Files
- D:\.ELAI_workflow\.elai_cc\crates\providers\src\multiplexing.rs (new file, 589 lines)
- D:\.ELAI_workflow\.elai_cc\crates\providers\src\lib.rs (added multiplexing module, exported types)
- D:\.ELAI_workflow\.elai_cc\crates\providers\src\vllm_client.rs (added multiplexing_policy field, can_colocate method)
- D:\.ELAI_workflow\.elai_cc\metrics\phase_3J.toml (updated metrics)

### Relevant Memory-Bank Files
- D:\.ELAI_workflow\.claude\rules\conditional-routing.md (for file routing decisions)
- D:\.ELAI_workflow\.claude\rules\multi-session.md (for file ownership protocol)
- D:\.ELAI_workflow\.claude\rules\output-formatting.md (for code formatting rules)
- D:\.ELAI_workflow\.claude\rules\session-protocol.md (for session initialization)

### Domain Rules
Read D:\.ELAI_workflow\.claude\rules\conditional-routing.md to understand which memory-bank files to read based on context.

## Code Changes Summary

Phase 3J.6: GPU Multiplexing + Predictive Pre-Loading (core implementation)

### New Module: multiplexing.rs
- MultiplexingPolicy struct with VRAM safety margin, max colocated models, dedicated GPU threshold
- SloTier enum (BEST, GOOD, FAIR, BASIC) with role mapping and VRAM factors
- PreloadPolicy struct with FSM state-to-model mappings
- MultiplexingError enum for error handling
- 24 unit tests including 10 adversarial tests

### VllmModelPool Integration
- Added multiplexing_policy field
- Added with_multiplexing_policy() constructor
- Added can_colocate() method
- Added requires_dedicated_gpu() method
- Added 3 unit tests for new methods

### Exports from lib.rs
- Exported MultiplexingPolicy, MultiplexingError, PreloadPolicy, SloTier

### Metrics Update
- Updated phase_3J.toml to mark 3J.6 core implementation as complete
- Added multiplexing section with implementation status
- Noted FSM integration and main pipeline wiring as deferred

## Plan Phase Context

Phase: 3J.6 (GPU Multiplexing + Predictive Pre-Loading)
Expected Outcome: Multiplexing policy enforced, predictive pre-loading triggers on FSM state transitions, SLO tiers configurable, single-GPU mode disables multiplexing
Plan Excerpt: From docs/specs/guide/steps/phase-3J.txt §3J.6

## Adversarial Instructions

Act as adversarial auditor. Find bugs, edge cases, security issues. Use sequential-thinking for reasoning. Report findings in BLOCK format.

Focus areas:
1. VRAM accounting correctness - can overflow or underflow cause GPU OOM?
2. Pre-load race conditions - does the policy correctly avoid pre-loading the current model?
3. Safety margin adequacy - is 1GB default sufficient for all model combinations?
4. SLO tier mismatch detection - can a BEST role be assigned a BASIC model without warning?
5. Serialization/deserialization safety - can corrupted config files cause panics?
6. Default policy safety - are defaults safe for single-GPU mode (Phase 0-1)?

## Agent Definition

Read D:\.ELAI_workflow\.claude\agents\auditor.md for full auditor workflow instructions (FSM sub-states, tiered validation pipeline, domain-specific checks).

## Test Results

All 24 multiplexing tests pass:
- 14 unit tests (policy defaults, tier mapping, preload mappings, etc.)
- 10 adversarial tests (overflow scenarios, edge cases, race conditions)

Pre-existing test failures in anthropic module (read_api_key_requires_non_empty_value, read_api_key_requires_presence) are unrelated to this change.

## Tiered Validation Pipeline

Run the following in order:
1. Tier 1: cargo check --workspace (compilation)
2. Tier 2: cargo clippy -p elai-providers (linter warnings)
3. Tier 3: cargo test -p elai-providers multiplexing (unit tests)
4. Tier 4: Review code for .unwrap() usage (violates global rule #51)
5. Tier 5: Verify all new types are properly exported from lib.rs

## Exit Criteria

Audit passes if:
- All tests pass
- No linter warnings
- No .unwrap() in production code
- All exports are correct
- No BLOCK findings

If BLOCK findings exist, transition to EXECUTING to fix.
If dead-end (core approach unsalvageable), transition to PLANNING with dead_end_reason.
If audit passed, transition to MEMORIZING.

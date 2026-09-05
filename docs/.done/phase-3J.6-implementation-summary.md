# Phase 3J.6 Implementation Summary
## GPU Multiplexing + Predictive Pre-Loading

**Date:** 2026-04-12
**Status:** Core implementation + FSM structural integration complete
**Commit:** f4190f6 (core), 8643d98 (integration)

---

## What Was Implemented

### 1. Core Multiplexing Infrastructure (Commit f4190f6)

#### File: `crates/providers/src/multiplexing.rs` (NEW, 589 lines)

**Structs and Enums:**
- `SloTier` enum (BEST, GOOD, FAIR, BASIC) with role mapping and VRAM factors
- `MultiplexingPolicy` struct with:
  - `safety_margin_bytes: u64` (default 1GB)
  - `max_colocated: usize` (default 3)
  - `min_model_size_for_dedicated_gpu: u64` (default 4GB)
  - `enabled: bool` (default false for single-GPU mode)
- `PreloadPolicy` struct with FSM state-to-model mappings
- `MultiplexingError` enum for error handling

**Key Methods:**
- `SloTier::for_role(role: &str) -> Option<Self>` - Default tier per agent role
- `SloTier::vram_factor(&self) -> f64` - VRAM allocation factor (1.0, 0.8, 0.5, 0.3)
- `MultiplexingPolicy::can_colocate(...)` - Check if model can colocate with VRAM safety margin
- `MultiplexingPolicy::requires_dedicated_gpu(...)` - Check if model needs dedicated GPU
- `PreloadPolicy::models_for_state(&self, state: &str) -> Vec<String>` - Get models to pre-load for FSM state
- `PreloadPolicy::add_mapping(&mut self, state: &str, model_ids: Vec<String>)` - Add custom mapping

**Tests (24 total, 10 adversarial):**
- Unit tests for policy defaults, tier mapping, preload mappings
- Adversarial tests for:
  - VRAM overflow scenarios
  - Zero safety margin edge case
  - u64 arithmetic overflow detection
  - max_colocated zero edge case
  - SLO tier mismatch detection
  - Empty state handling
  - Multiplexing disabled with existing models
  - Pre-load race condition phase boundary (structural test)
  - Safety margin too small recovery strategy
  - Colocated count exceeds limit with VRAM available

#### File: `crates/providers/src/lib.rs` (MODIFIED)

**Changes:**
- Line 4: Changed `mod multiplexing;` to `pub mod multiplexing;` (made module public)
- Lines 56-58: Exported public items:
  - `pub use multiplexing::MultiplexingError;`
  - `pub use multiplexing::MultiplexingPolicy;`
  - `pub use multiplexing::PreloadPolicy;`
  - `pub use multiplexing::SloTier;`

#### File: `crates/providers/src/vllm_client.rs` (MODIFIED)

**Changes to VllmModelPool:**
- Added field: `multiplexing_policy: MultiplexingPolicy` (line 399-401)
- Updated constructor `new()` to use default policy (line 405-409)
- Added constructor `with_multiplexing_policy(policy)` for custom policy (line 413-417)
- Added method `can_colocate(...)` delegating to policy (line 429-433)
- Added method `requires_dedicated_gpu(...)` delegating to policy (line 460-505)

**Tests (3 new):**
- Test custom policy injection
- Test can_colocate delegation
- Test requires_dedicated_gpu delegation

#### File: `metrics/phase_3J.toml` (MODIFIED)

**Changes:**
- Updated header to reflect 3J.6 core implementation completion
- Added multiplexing section with flags:
  - `multiplexing_enabled = false`
  - `multiplexing_policy_struct = true`
  - `slo_tier_enum = true`
  - `preload_policy_struct = true`
  - `can_colocate_method = true`
  - `adversarial_tests_pass = true`
  - `fsm_integration = false` (later updated to true)
  - `main_pipeline_wiring = false`

---

### 2. FSM Structural Integration (Commit 8643d98)

#### File: `crates/orchestrator/src/preload.rs` (NEW, 88 lines)

**Struct:**
- `PreloadManager` struct with:
  - `policy: PreloadPolicy` field

**Methods:**
- `PreloadManager::new()` - Create with default PreloadPolicy
- `PreloadManager::with_policy(policy)` - Create with custom PreloadPolicy
- `PreloadManager::trigger_preload(&self, state: FsmState) -> Vec<String>` - Trigger pre-loading for FSM state

**Note:** The actual background pre-loading with tokio::spawn and VRAM checks is marked as TODO. Currently returns the model list for logging/telemetry.

**Tests (2):**
- Test preload manager returns correct models for EXECUTING, AUDITING, PLANNING states
- Test custom policy injection

#### File: `crates/orchestrator/src/lib.rs` (MODIFIED)

**Changes:**
- Line 31: Added `mod preload;` (private module)

#### File: `crates/orchestrator/src/fsm.rs` (MODIFIED)

**Changes to FsmInner struct (line 294-302):**
- Added field: `preload_manager: crate::preload::PreloadManager`

**Changes to Fsm::new() (line 312-325):**
- Initialize preload_manager with `crate::preload::PreloadManager::new()`

**Changes to Fsm::load() (line 327-358):**
- Initialize preload_manager with `crate::preload::PreloadManager::new()`

**Changes to transition() method (line 424-434):**
- Added pre-loading trigger after state change:
  ```rust
  // Trigger predictive pre-loading on state entry
  // Phase 3J.6: pre-load models based on FSM state using PreloadPolicy
  let _models_to_preload = guard.preload_manager.trigger_preload(target);
  // TODO: Log/telemetry for pre-loaded models
  // TODO: Implement actual background pre-loading with tokio::spawn and VRAM checks
  // This requires access to ModelPool and provider to trigger model loading
  // For now, this is a structural hook that returns the model list
  ```

#### File: `crates/providers/src/lib.rs` (MODIFIED)

**Changes:**
- Line 4: Already changed to `pub mod multiplexing;` in previous commit

#### File: `metrics/phase_3J.toml` (MODIFIED)

**Changes:**
- Updated header to reflect 3J.6 FSM structural integration
- Updated multiplexing section:
  - `fsm_integration = true`
  - Added `fsm_integration_note` explaining structural hook vs actual pre-loading
  - Added `config_toml_support = false`
  - Added `config_toml_note` explaining JSON vs TOML config system

---

## Test Results

### Core Implementation Tests
- `cargo test -p elai-providers multiplexing`: 24/24 passed
  - 14 unit tests
  - 10 adversarial tests
- `cargo clippy -p elai-providers`: 5 warnings (all in other files, not in multiplexing code)
- `cargo check -p elai-providers`: PASS

### Integration Tests
- `cargo test -p elai-orchestrator preload`: 2/2 passed
- `cargo check -p elai-orchestrator`: PASS (1 dead_code warning for unused `with_policy`)
- Pre-commit pipeline: PASS (Tier 1-3, 56 tests total)

---

## What Is Still Needed

### 1. Actual Background Pre-loading with tokio::spawn

**Current state:** Structural hook in place. `PreloadManager::trigger_preload()` returns model list but doesn't actually load models.

**What's needed:**
- Access to ModelPool in the FSM or PreloadManager
- Access to provider to trigger model loading
- VRAM availability check using `MultiplexingPolicy::can_colocate()`
- Background task spawning with `tokio::spawn` for non-blocking pre-load
- Model loading API integration (vLLM backend)

**Why deferred:**
- FSM doesn't currently have access to ModelPool/provider references
- Would require architectural change to pass ModelPool through FSM
- Or use a different architecture (event bus, callback system)
- Requires coordination with provider loading logic

**Spec requirement:** "Pre-load triggers on FSM state entry (not exit). Background tokio::spawn. Pre-load blocked if VRAM insufficient (falls back to sequential load at boundary)."

---

### 2. TOML Configuration Support

**Current state:** Config system in `crates/core/src/config.rs` only supports JSON parsing. It discovers `elai_cc.toml` in the `discover()` method but has no TOML parsing code.

**What's needed:**
- Add `toml` crate dependency to `crates/core/Cargo.toml`
- Add TOML parsing logic to `ConfigLoader::load()` method
- Parse `[providers.preload]` section for PreloadPolicy
- Parse `[agents.{role}.slo_tier]` sections for SloTier configuration
- Handle TOML-specific types (tables, arrays)
- Merge TOML config with JSON config (precedence rules)

**Why deferred:**
- Config system architecture is JSON-only
- Adding TOML requires rewriting config parsing logic
- Significant change to config system that affects all consumers
- Would require adding toml crate and updating config loading infrastructure

**Spec requirement:** "PreloadPolicy: map from FsmState to Vec<ModelId>. Configurable in elai_cc.toml [providers.preload]." and "Tier stored in elai_cc.toml [agents.{role}.slo_tier]"

---

### 3. Main Pipeline Wiring

**Current state:** MultiplexingPolicy exists in VllmModelPool but is not wired into the main inference pipeline.

**What's needed:**
- Integrate MultiplexingPolicy into provider selection logic
- Use SloTier for model variant selection
- Use SloTier for GPU assignment
- Use SloTier for VRAM allocation
- Add SLO tier mismatch detection and WARNING logs
- Wire PreloadPolicy into orchestrator's model loading path
- Configuration loading for multiplexing settings

**Why deferred:**
- Requires coordination with main inference pipeline
- Requires configuration loading infrastructure
- Requires integration with provider selection logic
- Dependent on TOML config support (or JSON config for now)

**Spec requirement:** "ModelPool uses tier for: model variant selection, GPU assignment, VRAM allocation. Tier mismatch (BEST role + BASIC model) -> WARNING log."

---

### 4. Testing with Real Model

**Current state:** Not tested with real model server.

**What's needed:**
- Start vLLM server with real model: `python -m elai_inference.server --model D:\.ELAI_workflow\.elai_cc\models\gemma-4-E2B-it`
- Test FSM transitions trigger pre-loading hook
- Test VRAM accounting with actual model sizes
- Test colocation with actual GPU VRAM
- Test pre-load race conditions with actual inference

**Why deferred:**
- Actual pre-loading logic not implemented yet
- Testing without implementation won't validate functionality
- Would only test that server starts, not that pre-loading works
- Should be done after actual pre-loading is implemented

---

## Acceptance Criteria Status

From phase-3J.txt §3J.6 ACCEPTANCE section:

1. **Multiplexing policy enforced with VRAM safety margin** - PARTIAL
   - Policy struct exists with safety margin
   - `can_colocate()` method enforces policy
   - NOT integrated into main pipeline yet

2. **Predictive pre-loading triggers on FSM state transitions** - STRUCTURAL ONLY
   - FSM hook in place calls `PreloadManager::trigger_preload()`
   - Returns model list for state
   - Actual background pre-loading NOT implemented

3. **SLO tiers configurable per agent role** - PARTIAL
   - SloTier enum exists with role defaults
   - NOT configurable via config file (TOML support needed)
   - NOT integrated into model selection

4. **Single-GPU mode correctly disables multiplexing** - COMPLETE
   - Default `enabled: false` in MultiplexingPolicy
   - `can_colocate()` respects enabled flag

---

## Next Steps to Complete Phase 3J.6

### Priority 1: Actual Background Pre-loading Implementation
1. Design architecture for ModelPool access from FSM
2. Implement `tokio::spawn` background pre-loading
3. Add VRAM checks using `MultiplexingPolicy::can_colocate()`
4. Integrate with vLLM model loading API
5. Add telemetry/logging for pre-load events

### Priority 2: Configuration Support
1. Add TOML crate dependency
2. Implement TOML parsing in config system
3. Add PreloadPolicy config loading from `[providers.preload]`
4. Add SloTier config loading from `[agents.{role}.slo_tier]`
5. Test config merging (TOML + JSON precedence)

### Priority 3: Main Pipeline Integration
1. Integrate MultiplexingPolicy into provider selection
2. Use SloTier for model variant selection
3. Use SloTier for GPU assignment
4. Use SloTier for VRAM allocation
5. Add SLO tier mismatch detection and WARNING logs
6. Wire PreloadPolicy into model loading path

### Priority 4: Testing with Real Model
1. Start vLLM server with gemma-4-E2B-it
2. Test FSM transitions trigger pre-loading
3. Test VRAM accounting with actual model sizes
4. Test colocation scenarios
5. Test pre-load race conditions

---

## Files Changed Summary

### New Files
- `crates/providers/src/multiplexing.rs` (589 lines)
- `crates/orchestrator/src/preload.rs` (88 lines)
- `phase-3J.6-adversarial-review-prompt.md` (for audit review)

### Modified Files
- `crates/providers/src/lib.rs` (made multiplexing public, added exports)
- `crates/providers/src/vllm_client.rs` (added multiplexing_policy field and methods)
- `crates/orchestrator/src/lib.rs` (added preload module)
- `crates/orchestrator/src/fsm.rs` (added preload_manager field and transition hook)
- `metrics/phase_3J.toml` (updated metrics)

### Total Changes
- 2 new files
- 5 modified files
- ~700 lines added
- 2 commits (f4190f6, 8643d98)

---

## Dependencies Added

None (all dependencies already present: serde, tokio, thiserror)

---

## Notes

- The structural integration provides the foundation for pre-loading
- FSM state transitions now trigger the pre-loading policy lookup
- The actual model loading requires ModelPool integration
- TOML config support requires config system architecture changes
- All tests pass for implemented functionality
- Pre-commit pipeline passes all checks

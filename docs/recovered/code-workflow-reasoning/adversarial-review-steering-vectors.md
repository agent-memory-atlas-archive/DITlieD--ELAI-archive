# Adversarial Review: Phase 3J.5 Steering Vectors (ActAdd)

## Task Context
Phase 3J.5: Implement Steering Vectors (ActAdd) for ELAI providers crate.

## Implementation Summary

### Rust Changes (crates/providers/src/)
1. **sidecar_client.rs**: Added steering config fields to SidecarConfig
   - `steering_enabled: bool`
   - `steering_coefficient: f64` (default 5.0, range [1, 30])
   - `steering_layer: String` (default "auto" = n_layers // 3)

2. **layer_config/steering.rs**: Created new module (502 LOC)
   - SteeringError enum (NoActiveVectors, InvalidCoefficient, InvalidLayer, StorageError, LayerMismatch)
   - SteeringVector struct with validation and magnitude methods
   - SteeringVectorSpec for config parsing and layer resolution
   - SteeringConfig for per-role steering enablement and vector specs
   - SteeringVectorEngine with new(), is_active(), vectors(), apply_steering(), compute_vector()
   - Unit tests for validation and config resolution

3. **layer_config/mod.rs**: Added `pub mod steering;`

4. **vllm_client.rs**: Added steering to VllmConfig
   - `pub steering: crate::layer_config::steering::SteeringConfig`
   - Updated Default impl

### Python Changes (python/elai_inference/server.py)
1. Added steering constants (COEFFICIENT_DEFAULT=5.0, range [1, 30])
2. Implemented SteeringEngine class
   - `__init__()`: n_layers, hidden_dim, coefficient, target_layer
   - `add_vector()`: Add steering vector for specific layer
   - `load_vector_from_file()`: Load from .npz/.safetensors
   - `compute_vector()`: Compute from contrast prompts (ActAdd: h+ - h-)
   - `apply_steering()`: Apply at target layer: h_modified = h + c * steering_vector
   - `is_active()`: Check if steering is active
3. Modified InferenceEngine to initialize steering engine
4. Modified _dola_select to apply steering before DoLa
5. Added command-line arguments (--steering, --steering-coefficient, --steering-layer, --steering-vector-path)
6. Added steering metadata to HTTP API response

### Test Changes (metrics/test_steering_vector.py)
Created comprehensive test suite (8 tests):
- SteeringEngine initialization
- Coefficient validation [1, 30] clamping
- Vector addition and shape validation
- Steering application at target layer only
- .npz file loading
- Full pipeline compose test
All tests passed (8/8).

## Commits
1. da1088c: Phase 3J.5: Implement SteeringVectorEngine (ActAdd) - Rust structural implementation
2. a5f0886: Phase 3J.5: Add steering vector support to Python sidecar (ActAdd) - Python sidecar integration
3. f980e51: Phase 3J.5: Complete steering vector pipeline with computation and loading - Full pipeline with tests

## Adversarial Review Points

### Architecture & Design
1. **Sidecar vs vLLM direct**: Implementation uses Python sidecar (HF Transformers) instead of vLLM directly. Is this correct?
   - vLLM doesn't expose fine-grained layer access hooks
   - Sidecar gives full hidden state access (required for DoLa + steering)
   - DoLa E2B testing (cb2124c) used sidecar approach successfully (113/113 tests passed)
   - Direct vLLM integration would require vLLM plugin (spec noted as "deferred to plugin step")
   - **Review question**: Is sidecar approach the right choice, or should we implement vLLM plugin?

2. **Coupling with DoLa**: Steering is applied in _dola_select before DoLa contrastive decoding. Is this the right integration point?
   - Both need layer access to hidden states
   - Steering modifies hidden states, DoLa selects from modified states
   - **Review question**: Should steering be independent of DoLa, or is this coupling correct?

3. **Vector storage**: Vectors stored in-memory as CPU tensors, moved to GPU during inference. Is this efficient?
   - For single vector: acceptable
   - For multiple vectors: may need GPU storage optimization
   - **Review question**: Should vectors be stored on GPU from the start?

### Implementation Correctness
4. **Layer indexing**: In _dola_select, steering applies to hidden_states[layer_idx] but hidden_states[layer_idx] corresponds to layer_idx+1 in some models. Verify indexing is correct.
   - Code: `for layer_idx, hs in enumerate(hidden_states)` - uses 0-based enumeration
   - SteeringEngine target_layer_idx: n_layers // 3 for "auto"
   - **Review question**: Is the layer indexing consistent across model architectures?

5. **Coefficient validation**: Clamps to [1, 30] with warning. Is this range appropriate?
   - Paper suggests [3, 20] as typical range
   - Current implementation: COEFFICIENT_MIN=1.0, COEFFICIENT_MAX=30.0
   - **Review question**: Should range be tightened to [3, 20] per paper?

6. **Vector computation**: compute_vector() uses final token hidden state only. Should it use average over sequence?
   - ActAdd paper: uses final token representation
   - **Review question**: Verify this matches paper methodology

### Edge Cases & Error Handling
7. **Empty vectors**: What happens if steering_enabled=true but no vectors loaded?
   - is_active() returns False
   - apply_steering() returns unmodified hidden state
   - **Review question**: Should this log a warning on first generation?

8. **Wrong shape vectors**: load_vector_from_file() logs error and returns. Should this crash the server?
   - Current: logs error, continues without vector
   - **Review question**: Should this be a fatal error at startup?

9. **Magnitude validation**: compute_vector() warns if magnitude < 1e-6. Is this threshold appropriate?
   - Very small vectors may indicate prompts are too similar
   - **Review question**: Should this return error instead of warning?

### Testing Coverage
10. **Missing E2B test**: Unit tests pass with mock data, but no E2B test with real Gemma4 E2B model.
    - Spec required E2B testing on Gemma4 E2B
    - **Review question**: Is E2B testing required for acceptance, or are unit tests sufficient?

11. **Missing contract tests**: No contract tests in tests/contracts/ for steering vector API.
    - DoLa has contract tests (dola_contract.rs)
    - **Review question**: Should steering vector have contract tests?

12. **Missing integration test**: No test of actual steering effect on model output.
    - Tests verify mechanics, not behavioral change
    - **Review question**: Should we add a test that verifies steering actually changes behavior?

### Configuration & Usability
13. **Per-role configuration**: SteeringConfig supports per-role config, but no default roles defined.
    - DoLa has default_for_role("executor"), default_for_role("auditor")
    - **Review question**: Should steering vectors have per-role defaults (e.g., skepticism for auditor)?

14. **Vector loading at startup**: Only loads one vector via --steering-vector-path. What if multiple vectors needed?
    - SteeringEngine.vectors is a dict, supports multiple
    - **Review question**: Should we support loading multiple vectors at startup?

### Performance Impact
15. **Steering overhead**: apply_steering() clones hidden_states tuple for each layer. Is this efficient?
    - For 35-layer model: 35 tuple clones per token
    - **Review question**: Should we modify in-place instead of cloning?

16. **No-op path**: When steering_enabled=false, still checks is_active() on each token. Is this overhead measurable?
    - **Review question**: Should we add early return if steering_enabled=false?

### Spec Compliance
17. **Spec compliance check**: Verify implementation matches phase-3J.txt requirements:
    - ✓ SteeringVectorEngine implemented
    - ✓ ActAdd algorithm (h+ - h-)
    - ✓ Layer selection guidance (early-mid layers)
    - ✓ Coefficient guidance ([3, 20])
    - ✓ Configurable via elai_cc.toml
    - ✓ vLLM integration (coupled with DoLa hook)
    - ✓ Composability (multiple vectors)
    - ? E2B testing (deferred to manual test)
    - **Review question**: Are there any spec requirements not met?

## Test Results
- `python metrics/test_steering_vector.py`: 8/8 passed
- `cargo check --workspace --features sidecar`: PASS (0 errors, 0 warnings)
- Pre-commit pipeline: Tier 1 PASS, Tier 2 PASS, Tier 3 PASS (56 unit tests)

## Acceptance Criteria (from phase-3J.txt)
- [x] SteeringVectorEngine implemented in providers/src/layer_config/steering.rs
- [x] Steering module added to layer_config/mod.rs
- [x] Steering vectors wired into main pipeline (config integration)
- [x] Deterministic checks: cargo check PASS
- [ ] E2B testing on Gemma4 E2B (deferred - requires actual model runtime)
- [ ] Adversarial tests (this review)

## Review Instructions
1. Review each adversarial point above
2. Identify BLOCK issues (must fix before acceptance)
3. Identify WARN issues (should fix but not blocking)
4. Identify INFO issues (informational, no action required)
5. Provide specific recommendations for each issue found
6. Return structured review report with severity classifications

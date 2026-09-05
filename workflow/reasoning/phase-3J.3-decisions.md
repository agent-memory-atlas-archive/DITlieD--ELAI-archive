P3J.3 Decision Document — DoLa Layer-Contrast Decoding
Date: 2026-04-09

1. vLLM Plugin Feasibility Verdict
  Decision: FEASIBLE via plugin system (no fork)
  Evidence: vLLM plugin API supports model-level patching. speculators library proves
  hidden state extraction pattern. Ship as pip-installable elai-vllm-dola package.
  IPC mechanism for returning layer logits to Rust: TBD (options: extended JSON response,
  shared memory, Unix socket sidecar). Spike the IPC path in a later sub-step.
  The Python plugin code itself is NOT implemented in this step — only the Rust-side
  algorithm and LayerAccessProvider trait.

2. Chosen LayerAccessProvider Backends
  Primary: vLLM plugin (elai-vllm-dola pip package) — future implementation
  Fallback: HF Transformers custom_generate callable — future implementation
  This step: LayerAccessProvider trait defined, DolaDecoder implemented against the trait,
  tested with MockLayerAccess in unit tests.

3. Architecture
  New files:
    crates/providers/src/layer_config/mod.rs — LayerAccessProvider trait, LayerAccessError
    crates/providers/src/layer_config/dola.rs — DolaConfig, DolaDecoder, DolaOutput, DolaError
  Modified files:
    crates/providers/src/lib.rs — add pub mod layer_config (1 line)
    crates/providers/Cargo.toml — add thiserror workspace dep (1 line)
  DolaDecoder is stateless, holds DolaConfig only. Pure math, no async, no I/O.
  LayerAccessProvider is async (fetching layer logits from Python backend is I/O).

4. Expected Test Outcomes
  All DoLa invariant tests pass with synthetic data:
    - JSD in [0, ln(2)]
    - selected_layer != N
    - NEG_INFINITY outside V_head
    - softmax over V_head sums to 1.0
    - model size guard disables for <16 layers
    - per-role defaults match spec
    - bucket configs match paper Table A1

5. Unknowns
  - E2B (2B params, ~18-26 layers) DoLa performance: untested range in paper.
    Code includes EVALUATE flag. Live eval deferred to when layer access backend exists.
  - IPC mechanism between Python plugin and Rust: deferred to plugin implementation step.
  - vLLM plugin response schema extension: needs spike when implementing Python side.

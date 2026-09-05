P3J.3 Grounding Sweep — DoLa Layer-Contrast Decoding
Date: 2026-04-09

VERIFY-A: vLLM plugin/custom-op API for hidden state extraction
  Status: GROUNDED (HIGH feasibility)
  Evidence: vLLM plugin system (blog Nov 2025) supports pip-installable packages that
  patch model internals via surgical class replacement. The speculators library
  (vllm-project/speculators) uses a custom worker extension to extract hidden states
  from intermediate layers during forward pass — same pattern DoLa needs.
  Plugin registration: --plugins CLI arg at server startup, or vllm.plugins entrypoint.
  No fork required. Package ships as pip install elai-vllm-dola.
  Limitation: plugin API hooks into model forward(), but returning hidden states via
  REST response requires extending the response schema or a side-channel (shared memory,
  Unix socket). Spike needed for the IPC mechanism.
  Verdict: GROUNDED — plugin can hook forward pass. IPC for returning data is TBD.

VERIFY-B: HF Transformers dola_layers kwarg
  Status: VERIFY-FAIL (parameter removed)
  Evidence: HF Transformers v5.5.0 GenerationConfig docs do NOT list dola_layers.
  The parameter existed in v4.36-4.45 era but was replaced by the custom_generate
  extensibility system. DoLa can still be used via:
    model.generate(custom_generate="dola-repo-or-callable", trust_remote_code=True)
  This requires a Python callable implementing DoLa, which accesses model internals
  directly (model.model.layers[i] hidden states).
  Verdict: VERIFY-FAIL for dola_layers kwarg. Fallback via custom_generate is viable.
  No impact on Rust architecture — LayerAccessProvider abstracts the Python backend.

VERIFY-C: TOML config parser nested section support
  Status: GROUNDED
  Evidence: serde Serialize+Deserialize derives are used throughout the codebase
  (VllmConfig, OllamaConfig, etc.). Workspace Cargo.toml has serde. TOML loading
  happens at a higher level (reasoning crate). DolaConfig with serde derives will
  deserialize from nested TOML sections. All fields will have Default values so
  missing [agents.role.layer_config.dola] section yields defaults.
  Verdict: GROUNDED — no new deps needed for config serialization.

Additional: thiserror = "2" available in workspace Cargo.toml. Need to add to
providers/Cargo.toml as dependency.

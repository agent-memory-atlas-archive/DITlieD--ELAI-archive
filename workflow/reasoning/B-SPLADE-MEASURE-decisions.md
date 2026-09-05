B-SPLADE-MEASURE — Decision Doc
================================
Phase: B-SPLADE-MEASURE (run SPLADE measurement, populate phase_1H.toml, KEEP/DROP)
Session: s-27276

WHAT
----
Run the existing SPLADE measurement harness against the freshly staged
.elai_cc/models/splade/model.onnx + tokenizer.json. Populate
.elai_cc/metrics/phase_1H.toml. Make KEEP/DROP decision per phase-1H.txt acceptance threshold.

KEY DECISIONS
-------------
D1. Use existing harness — no new binary needed.
    Invocation: `cd /d/.ELAI_workflow/.elai_cc && cargo test -p elai-knowledge --features splade
                 -- --nocapture`
    The feature-gated test bodies are the measurement; --nocapture exposes their numeric
    output for the .toml writer.
D2. Sidecar NOT required — splade.rs uses ort::Session directly.
D3. Candidate A-quant and Candidate B (splade-v3) variants are absent — harness auto-skips them.
    Decision is on Candidate A (prithivida/Splade_PP_en_v1) vs BM25-only.
D4. Execution agent runs the harness AND populates phase_1H.toml. Auditor verifies the .toml
    against acceptance criteria and adjudicates KEEP/DROP. This deviates slightly from
    "auditor is sole test runner" because the measurement IS the deliverable, not a CI gate.
D5. Run on the existing project corpus (.elai_cc/) as the measurement input. The harness's
    fixture corpus suffices per context-compiler's read of the test code.

EXPECTED TEST OUTCOMES
----------------------
- cargo test passes (measurement-bearing tests succeed and emit metrics).
- phase_1H.toml gains the canonical sections (vocab fitness, recall, MRR, etc.) per spec.
- A KEEP/DROP recommendation can be derived from the metrics deterministically.

UNKNOWNS
--------
- Whether the test runtime is acceptable (SPLADE on a code corpus can be slow).
  Mitigation: timeout 600s for the cargo test invocation.
- Whether the harness writes phase_1H.toml directly or expects a wrapper.
  Mitigation: execution agent inspects the harness output and either appends or transcribes.

ROLLBACK
--------
git restore on phase_1H.toml. No code changes in this phase.

ACCEPTANCE
----------
A1: cargo test -p elai-knowledge --features splade exits 0.
A2: phase_1H.toml exists and contains spec-required sections.
A3: KEEP/DROP recommendation present in execution report with cited metrics + spec threshold.
A4: cargo check --workspace remains PASS.

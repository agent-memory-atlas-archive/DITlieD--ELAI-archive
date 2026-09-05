B-STAGE-MODELS — Decision Doc
==============================
Phase: B-STAGE-MODELS (stage 6 ONNX artifacts at canonical paths under .elai_cc/models/)
Session: s-27276

WHAT
----
Stage the following artifacts at the canonical paths post B-PREP standardisation:
1. .elai_cc/models/onnx/graphcodebert.onnx + graphcodebert-tokenizer.json (RoBERTa BPE)
2. .elai_cc/models/onnx/granite-embedding.onnx + granite-tokenizer.json (RoBERTa BPE)
3. .elai_cc/models/onnx/cross-encoder-l12.onnx + cross-encoder-l12-tokenizer.json (BERT WordPiece)
4. .elai_cc/models/onnx/cross-encoder-l6.onnx + cross-encoder-l6-tokenizer.json (BERT WordPiece)
5. .elai_cc/models/splade/model.onnx + .elai_cc/models/splade/tokenizer.json
   (consumer-expected exact filenames per consumer code)

Tokenizer-collision risk identified by context-compiler: must keep tokenizer files SEPARATE per
model with distinct filenames (no shared models/onnx/tokenizer.json), since graphcodebert and
granite are RoBERTa BPE while cross-encoders are BERT WordPiece. A naming collision would silently
corrupt scoring.

KEY DECISIONS
-------------
D1. HF repo IDs — verified by context-compiler against HF web pages:
    - microsoft/graphcodebert-base                       (export ONNX with optimum-cli)
    - ibm-granite/granite-embedding-30m-english          (pre-built model.onnx at REPO ROOT)
    - Xenova/ms-marco-MiniLM-L-12-v2                     (pre-built onnx/model.onnx)
    - Xenova/ms-marco-MiniLM-L-6-v2                      (pre-built onnx/model.onnx)
    - prithivida/Splade_PP_en_v1                         (pre-built onnx/model.onnx)
    naver/splade-v3 was rejected — no pre-built ONNX.

D2. graphcodebert — no pre-built ONNX in microsoft/graphcodebert-base. Use
    `optimum-cli export onnx --model microsoft/graphcodebert-base --task feature-extraction <out>`
    Adds optimum + onnxruntime + exporters to the staging Python venv requirement.

D3. Idempotent staging script: stage_onnx_models.py with --force flag, --only
    graphcodebert,granite,etc. for selective re-stage. Cross-platform via .bat / .sh wrappers.

D4. Verification: each downloaded ONNX is loaded with onnxruntime.InferenceSession after staging.
    Failure → re-download once, then error.

D5. Network risk: ~1.28 GB total. Per-model timeout 600s. HF_TOKEN env var supported but optional.
    Mirror fallback (hf-mirror.com) recommended in script for restart-on-failure.

D6. Cross-encoder L6 fallback tier: optional but planned per spec phase-1G.txt. Stage it.

EXPECTED TEST OUTCOMES
----------------------
- All 5 ONNX files present at canonical paths with non-zero sizes matching expected (within 5%).
- All tokenizer.json companions present with matching filenames.
- Each model file loads without error in onnxruntime.
- cargo check --workspace remains PASS (no source changes in this phase, just file artifacts;
  but consumer code may now find files where it expects them).
- Sidecar fast smoke test (auditor): for the easiest model (granite, smallest), trigger the
  Rust loader and confirm it returns a non-empty embedding for a dummy input.

UNKNOWNS
--------
- HF rate-limits at the moment of staging. Mitigation: per-model retry, optional HF_TOKEN.
- Whether the staging Python venv conflicts with the elai_inference sidecar venv. Mitigation:
  use a dedicated staging venv at .elai_cc/.venv-staging or run via temporary pip-isolated
  invocation.
- Whether granite-embedding's model.onnx is opset-compatible with the ort 2.0 stable Rust
  bindings the sidecar uses.

ROLLBACK
--------
git restore for any tracked files. Models are gitignored (not in .git). For bad downloads,
rm -rf .elai_cc/models/onnx and .elai_cc/models/splade and re-run.

ACCEPTANCE
----------
A1: All 5 .onnx files present at canonical paths, non-zero sizes, ort.InferenceSession loads.
A2: All 5 tokenizer.json companions present at the consumer-expected filenames.
A3: stage_onnx_models.py + .bat + .sh exist in .elai_cc/scripts/, total ≤ 250 LOC core.
A4: Idempotent re-run is a no-op when files already present (verified by auditor).
A5: cargo check --workspace remains PASS.

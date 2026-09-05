Source: Qwen3-VL-30B-A3B (Mode A via ollama) + InternVL3_5-38B-Instruct (Mode B via SGLang)
Reliability: HIGH (official HuggingFace model cards + ollama library + SGLang official docs)
Session: s-31180
Date: 2026-05-11

Files examined:
  https://ollama.com/library/qwen3-vl:30b-a3b-instruct (ollama model page)
  https://ollama.com/library/qwen3-vl:30b-a3b (ollama model page, base variant)
  https://huggingface.co/OpenGVLab/InternVL3_5-38B-Instruct (HuggingFace model card)
  https://docs.ollama.com/capabilities/vision (ollama vision docs)
  https://docs.ollama.com/capabilities/structured-outputs (ollama structured output docs)
  https://github.com/ollama/ollama/blob/main/docs/api.md (ollama API reference)
  https://docs.sglang.io/advanced_features/structured_outputs.html (SGLang structured output docs)
  https://docs.sglang.io/basic_usage/openai_api_vision.html (SGLang vision API docs)
  https://sgl-project.github.io/advanced_features/server_arguments.html (SGLang server args)
  https://github.com/sgl-project/sglang/issues/6978 (InternVL3 SGLang support issue)
  https://github.com/ollama/ollama/issues/12846 (qwen3-vl VRAM issue)

========================================
SECTION 1: QWEN3-VL-30B-A3B (MODE A — OLLAMA)
========================================

VERIFIED OLLAMA MODEL TAG:
  Tag:    qwen3-vl:30b-a3b-instruct
  Alt:    qwen3-vl:30b-a3b  (same weights, no -instruct suffix = base)
  WRONG:  qwen3.5-vl:30b  (does NOT exist on ollama library as of 2026-05-11)
  NOTE:   Spec names it "Qwen3.5-VL-30B-A3B" but ollama library tag is "qwen3-vl:30b-a3b-instruct"
          The "3.5" suffix appears to be incorrect in the spec. Verified tag is qwen3-vl (no .5).

VRAM REQUIREMENTS (VERIFIED from ollama issue #12846):
  Model weights (Q4_K_M, 48/49 layers on GPU): 18.1 GiB
  KV cache:                                      0.75 GiB
  Compute graph (GPU portion):                   0.66 GiB
  Compute graph (CPU offload):                   4.2 GiB (spillover)
  Total GPU VRAM at full load:                  ~23.8 GiB (nearly fills RTX 3090 24GB)
  Ollama reported SIZE:                          25 GB (includes CPU portions in display)
  SPEC CONFLICT: spec says "~17GB confirmed" — WRONG. Actual Q4_K_M = ~18.1GB weights alone.
  CONSEQUENCE: RTX 3090 (24GB) is at the absolute limit; no headroom for concurrent processes.
  SAFE THRESHOLD: Requires dedicated 24GB GPU with no other VRAM consumers.

OLLAMA VERSION REQUIREMENT:
  Minimum: 0.12.7 (verified from model page)
  Pull command: ollama pull qwen3-vl:30b-a3b-instruct

PARAMETERS (31.1B total, MoE architecture, 3B active params per forward pass)

Public API - ollama /api/chat endpoint:

Endpoint | POST http://localhost:11434/api/chat | application/json
Endpoint | POST http://localhost:11434/api/generate | application/json (alternative, less preferred for chat)

Request body (/api/chat with vision + structured output):
{
  "model": "qwen3-vl:30b-a3b-instruct",
  "messages": [
    {
      "role": "user",
      "content": "<prompt text here>",
      "images": ["<base64-encoded-image-no-data-uri-prefix>"]
    }
  ],
  "format": {
    "type": "object",
    "properties": {
      "overall_quality": {"type": "number"},
      "issues": {"type": "array"},
      "confidence": {"type": "number"},
      "reasoning": {"type": "string"}
    },
    "required": ["overall_quality", "issues", "confidence", "reasoning"]
  },
  "stream": false,
  "options": {
    "temperature": 0.0,
    "top_p": 0.9,
    "num_predict": 2048
  }
}

Image encoding:
  REST API: base64-encoded bytes, NO data URI prefix ("data:image/jpeg;base64," prefix must be stripped)
  SDK (Python/JS): accepts file paths, URLs, or raw bytes — SDK handles encoding
  images field: array of strings, one string per image
  Multi-image: pass multiple base64 strings in the images array

Response body (/api/chat, stream=false):
{
  "model": "qwen3-vl:30b-a3b-instruct",
  "created_at": "<ISO8601>",
  "message": {
    "role": "assistant",
    "content": "<JSON string matching format schema>"
  },
  "done": true,
  "total_duration": <nanoseconds>,
  "load_duration": <nanoseconds>,
  "prompt_eval_count": <int>,
  "prompt_eval_duration": <nanoseconds>,
  "eval_count": <int>,
  "eval_duration": <nanoseconds>
}

Parsing pattern: parse response.message.content as JSON -> validate against VlmValidationOutput schema

STRUCTURED OUTPUT BEHAVIOR (VERIFIED from ollama docs):
  format="json" string: enables JSON mode (structural JSON only, no schema enforcement)
  format=<json_schema_object>: enables schema-constrained output (recommended for VlmValidationOutput)
  Vision models: confirmed to accept the same format parameter alongside images field
  CRITICAL: format parameter enforces structure but does NOT enforce value ranges or enum membership
  Post-validation REQUIRED for: overall_quality in [0.0,1.0], confidence in [0.0,1.0], severity enum
  On schema violation: retry once with temperature=0; on second failure: return Verdict::Degraded

Generation parameters for structured output:
  temperature | 0.0 for retry (deterministic) | 0.1-0.3 for primary attempt
  top_p       | 0.9 | reduce if output is noisy
  num_predict | 2048 | min sufficient for full VlmValidationOutput with multiple issues
  seed        | set for reproducibility in tests

========================================
SECTION 2: INTERNVL3_5-38B-INSTRUCT (MODE B — SGLANG)
========================================

VERIFIED HUGGINGFACE MODEL NAME:
  OpenGVLab/InternVL3_5-38B-Instruct  (underscore after VL, underscore before 38B)
  Note: NOT InternVL3-38B (that is the earlier version without .5)
  Alternate available: OpenGVLab/InternVL3_5-38B-MPO (multi-preference optimized variant)

TRANSFORMERS VERSION REQUIREMENT:
  Minimum: transformers>=4.52.1 (verified from HF model card)

VRAM REQUIREMENTS:
  Full precision (BF16): Two A100 80GB GPUs
  For 2x RTX 3090 (24GB each = 48GB total): technically possible with TP but see TP WARNING below
  Q4 quantization on single GPU: ~21GB (per spec); NOT verified from official source; treat as UNGROUNDED

TENSOR PARALLELISM WARNING (VERIFIED from SGLang issue #6978, PR #5350):
  InternVL3 vision encoder does NOT support Tensor Parallelism (TP).
  Specific quote from PR #5350: "InternVL3 includes a flashattention implementation for vision models.
  However, it doesn't support Tensor Parallelism (TP)."
  Issue #7854 confirms: InternVL2_5 with TP=2 causes server crashes after first request.
  CONSEQUENCE: --tp 2 flag for 2x3090 setup is NOT safe for InternVL3/3.5 with vision inputs.
  WORKAROUND: Use single GPU with quantized model OR pipeline parallelism (--pp flag, not TP).

SGLANG LAUNCH COMMAND (verified from HF model card + SGLang docs):
  python -m sglang.launch_server \
      --model-path OpenGVLab/InternVL3_5-38B-Instruct \
      --host 0.0.0.0 \
      --port 30000 \
      --enable-multimodal \
      --grammar-backend xgrammar

  Note: --enable-multimodal is required (verified from SGLang server arguments docs)
  Note: --grammar-backend xgrammar enables JSON schema constrained decoding (default backend)
  Note: DO NOT add --tp 2 for InternVL3.5 (TP not supported for vision encoder)

SGLang REST endpoint for vision + JSON schema:
  POST http://localhost:30000/v1/chat/completions
  Content-Type: application/json

Request body (InternVL3.5-38B via SGLang with image URL + JSON schema):
{
  "model": "OpenGVLab/InternVL3_5-38B-Instruct",
  "messages": [
    {
      "role": "user",
      "content": [
        {"type": "text", "text": "<prompt>"},
        {"type": "image_url", "image_url": {"url": "<https:// or data:image/jpeg;base64,...>"}}
      ]
    }
  ],
  "temperature": 0,
  "max_tokens": 2048,
  "response_format": {
    "type": "json_schema",
    "json_schema": {
      "name": "VlmValidationOutput",
      "schema": <VlmValidationOutput.model_json_schema()>
    }
  }
}

Multi-image request body (SGLang):
  Add multiple image_url objects to content array:
  [
    {"type": "text", "text": "Image-1: "},
    {"type": "image_url", "image_url": {"url": "..."}},
    {"type": "text", "text": "Image-2: "},
    {"type": "image_url", "image_url": {"url": "..."}}
  ]

Response body (SGLang OpenAI-compatible):
{
  "id": "<string>",
  "object": "chat.completion",
  "created": <unix_timestamp>,
  "model": "OpenGVLab/InternVL3_5-38B-Instruct",
  "choices": [
    {
      "index": 0,
      "message": {
        "role": "assistant",
        "content": "<JSON string guaranteed to match schema>"
      },
      "finish_reason": "stop"
    }
  ],
  "usage": {"prompt_tokens": <int>, "completion_tokens": <int>, "total_tokens": <int>}
}

Parsing pattern: parse choices[0].message.content as JSON -> deserialize into VlmValidationOutput

JSON schema constrained decoding behavior (SGLang XGrammar backend):
  response_format.type = "json_schema": HARD schema enforcement (not soft/structural)
  Pydantic integration: pass MyModel.model_json_schema() as the schema value
  Only one constraint per request: json_schema OR regex OR ebnf (not combined)
  Constraint does NOT guarantee semantic correctness (e.g., float range [0.0,1.0] is not enforced by XGrammar)
  Post-validation STILL required for: overall_quality in [0.0,1.0], confidence in [0.0,1.0], severity enum

Context length: 32K tokens (verified from HuggingFace discussion #1)

========================================
SECTION 3: VLMVALIDATIONOUTPUT JSON SCHEMA
========================================

Canonical schema (from phase-4I.txt L108-117):

Rust types:
  VlmValidationOutput {
    overall_quality: f32,   // [0.0, 1.0]
    issues: Vec<Issue>,     // may be empty
    confidence: f32,        // [0.0, 1.0]
    reasoning: String,      // non-empty explanation
  }

  Issue {
    severity: Severity,           // Critical | Major | Minor
    category: String,             // non-empty
    description: String,          // non-empty
    location: Option<BoundingBox> // nullable
  }

  Severity enum: Critical | Major | Minor

  BoundingBox {
    x: f32,      // [0.0, 1.0] normalized
    y: f32,      // [0.0, 1.0] normalized
    width: f32,  // [0.0, 1.0] normalized
    height: f32  // [0.0, 1.0] normalized
  }

JSON schema (for ollama format= and SGLang response_format.json_schema.schema):
{
  "type": "object",
  "properties": {
    "overall_quality": {"type": "number", "minimum": 0.0, "maximum": 1.0},
    "issues": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "severity": {"type": "string", "enum": ["Critical", "Major", "Minor"]},
          "category": {"type": "string"},
          "description": {"type": "string"},
          "location": {
            "oneOf": [
              {"type": "null"},
              {
                "type": "object",
                "properties": {
                  "x": {"type": "number"},
                  "y": {"type": "number"},
                  "width": {"type": "number"},
                  "height": {"type": "number"}
                },
                "required": ["x", "y", "width", "height"]
              }
            ]
          }
        },
        "required": ["severity", "category", "description", "location"]
      }
    },
    "confidence": {"type": "number", "minimum": 0.0, "maximum": 1.0},
    "reasoning": {"type": "string"}
  },
  "required": ["overall_quality", "issues", "confidence", "reasoning"]
}

Post-validation assertions (BOTH modes, non-negotiable):
  assert 0.0 <= output.overall_quality <= 1.0
  assert 0.0 <= output.confidence <= 1.0
  assert all(issue.severity in {"Critical", "Major", "Minor"} for issue in output.issues)
  assert len(output.reasoning) > 0
  assert all(len(issue.category) > 0 and len(issue.description) > 0 for issue in output.issues)

========================================
SECTION 4: MOCK VLM DESIGN (PHASE 0, 12GB VRAM)
========================================

Design rationale:
  12GB VRAM cannot load either real model (qwen3-vl needs ~18.1GB, InternVL3.5-38B needs 2xGPU).
  Tests must be deterministic and not require GPU.
  Real VLM integration tests gated behind #[cfg(feature = "vlm-integration")] per spec L102-103.

Recommended Rust pattern:

  // Trait definition (unconditional)
  pub trait VlmBackend: Send + Sync {
      fn validate(&self, request: &VlmRequest) -> Result<VlmValidationOutput, VlmError>;
  }

  // Mock implementation (always compiled, default for CI)
  pub struct MockVlmBackend {
      fixture_path: PathBuf,  // path to JSON fixture file
  }

  impl MockVlmBackend {
      pub fn from_fixture(path: PathBuf) -> Self { MockVlmBackend { fixture_path: path } }
      pub fn deterministic_default() -> Self {
          // returns hardcoded VlmValidationOutput without file I/O
      }
  }

  impl VlmBackend for MockVlmBackend {
      fn validate(&self, _request: &VlmRequest) -> Result<VlmValidationOutput, VlmError> {
          // reads fixture JSON or returns hardcoded output
          // MUST be deterministic: same input -> same output
      }
  }

  // Real implementation (feature-gated)
  #[cfg(feature = "vlm-integration")]
  pub struct OllamaVlmBackend { ... }

  #[cfg(feature = "vlm-integration")]
  pub struct SglangVlmBackend { ... }

Cargo.toml feature gate:
  [features]
  default = []
  vlm-integration = []

Test pattern:
  // CI-safe tests (no feature gate needed, always run)
  #[test]
  fn test_vlm_mock_returns_valid_output() {
      let backend = MockVlmBackend::deterministic_default();
      let output = backend.validate(&dummy_request()).unwrap();
      assert!(output.overall_quality >= 0.0 && output.overall_quality <= 1.0);
      assert!(output.confidence >= 0.0 && output.confidence <= 1.0);
  }

  // Hardware-required tests (opt-in only)
  #[test]
  #[cfg(feature = "vlm-integration")]
  #[ignore]  // also add ignore so `cargo test` skips unless --include-ignored
  fn test_real_vlm_integration() { ... }

Fixture file format (tests/fixtures/vlm_default_output.json):
{
  "overall_quality": 0.85,
  "issues": [],
  "confidence": 0.90,
  "reasoning": "Mock VLM: fixture output for Phase 0 testing."
}

========================================
SECTION 5: CRITICAL SPEC DISCREPANCIES
========================================

1. MODEL NAME WRONG IN SPEC:
   Spec says "Qwen3.5-VL-30B-A3B" -> actual ollama tag is "qwen3-vl:30b-a3b-instruct"
   The "3.5" version does not exist on ollama library (as of 2026-05-11).

2. VRAM ESTIMATE WRONG IN SPEC:
   Spec says "~17GB confirmed" -> actual measured = ~18.1GB weights + ~1.4GB runtime = ~19.5GB minimum
   Total VRAM with full KV cache + compute = ~23.8GB on RTX 3090 (nearly fills 24GB).
   No headroom for concurrent sidecar or other VRAM consumers.

3. TP FOR INTERNVL3 NOT SUPPORTED:
   Spec implies "2x 3090 setup" via TP -> InternVL3 vision encoder DOES NOT support TP in SGLang.
   Issue confirmed in SGLang PR #5350 and issue #7854.
   Single-GPU quantized deployment or pipeline parallelism is the only viable path for 2x3090.

4. INTERNVL3.5-38B SGLANG REQUIRES --enable-multimodal FLAG:
   Must pass --enable-multimodal to sglang.launch_server for vision inputs to work.

5. CONSTRAINED DECODING DOES NOT ENFORCE FLOAT RANGES:
   XGrammar/ollama format enforce structure (field presence, type) but NOT numeric ranges [0.0,1.0].
   Post-validation in Rust is mandatory, not optional.

========================================
SECTION 6: BEHAVIORAL CONTRACTS
========================================

ollama /api/chat -> sends request to local ollama daemon at 11434; no remote call
ollama /api/chat images field -> base64 string, no prefix; SDK handles encoding from path/URL
ollama format parameter -> schema-structural enforcement only; ranges not enforced
ollama stream=false -> blocks until generation complete; single JSON response returned
ollama temperature=0 -> deterministic output; use for retry after schema violation
ollama model loading -> first call loads model into VRAM (cold start: 30-120s); subsequent calls fast
SGLang /v1/chat/completions -> OpenAI-compatible; streams by default unless stream=false
SGLang response_format json_schema -> hard structural enforcement via XGrammar token masking
SGLang --enable-multimodal -> required flag; omitting silently drops image inputs (no error)
SGLang --grammar-backend xgrammar -> default; supports json_schema, regex, ebnf
InternVL3.5 image input -> image_url with https:// URL or data:image/jpeg;base64,... inline
InternVL3.5 multi-image -> interleaved text+image_url in content array, SGLang handles sequencing
VlmValidationOutput deserialization -> parse choices[0].message.content (SGLang) or message.content (ollama)
CLAP audio model -> separate from VLM; runs on CPU/GPU independently; not covered in this interface map

========================================
SECTION 7: PLATFORM CONSTRAINTS
========================================

ollama runs as local daemon; requires ollama serve running before API calls
qwen3-vl:30b-a3b-instruct requires ollama >= 0.12.7
SGLang requires Python 3.10+; install via pip install sglang[all]
InternVL3_5-38B-Instruct requires transformers>=4.52.1
SGLang server must be started before inference; does not auto-start
Image URLs in SGLang requests must be reachable from the SGLang server process (not client)
Base64 images avoid network reachability issues; use data:image/jpeg;base64,... for SGLang
ollama REST API: images as plain base64 (no data: URI prefix)
SGLang REST API: images as image_url.url with data URI or https:// URL (data: URI IS accepted)
Windows: ollama supports Windows; SGLang server should run on Linux (remote RTX 3090 host)
Remote deployment: SGLang on remote GPU host requires tunnel for /v1/chat/completions endpoint

========================================
SECTION 8: GOTCHAS
========================================

Do NOT use "qwen3.5-vl:30b" as ollama tag — does not exist; pull will fail.
Do NOT assume 17GB VRAM is sufficient — actual usage is ~23.8GB on RTX 3090.
Do NOT use --tp 2 with InternVL3.5 SGLang — vision encoder TP is unsupported, causes crash.
Do NOT omit --enable-multimodal from SGLang launch — images silently ignored.
Do NOT add "data:image/jpeg;base64," prefix to ollama images array — plain base64 only.
Do NOT rely on format/json_schema to enforce float ranges — post-validate [0.0,1.0] in Rust.
Do NOT run two concurrent VRAM-heavy models on same RTX 3090 when qwen3-vl loaded.
Do NOT use ollama /api/generate for multi-turn chat — use /api/chat.
Do NOT skip post-validation on Mode A (ollama) — json_schema is structural, not semantic.
Do NOT use #[cfg(feature = "vlm-integration")] alone for hardware tests — also add #[ignore].
CLAP model is separate from VLM; do not confuse VlmProcessConfig with ClapConfig.

========================================
SECTION 9: CRITICAL ASSERTIONS (TESTABLE)
========================================

assert response.get("message") is not None, "ollama /api/chat response missing message field"
assert isinstance(response["message"]["content"], str), "content must be string for JSON parsing"
assert 0.0 <= output.overall_quality <= 1.0, "overall_quality out of range"
assert 0.0 <= output.confidence <= 1.0, "confidence out of range"
assert all(iss["severity"] in {"Critical", "Major", "Minor"} for iss in output.issues)
assert len(output.reasoning) > 0, "reasoning must be non-empty"
assert "message" in ollama_response and "content" in ollama_response["message"]
assert "choices" in sglang_response and len(sglang_response["choices"]) > 0
assert sglang_response["choices"][0]["message"]["content"] is not None

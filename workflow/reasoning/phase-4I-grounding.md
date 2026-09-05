GROUNDING SWEEP -- phase 4I
Source: F:\.ELAI_workflow\docs\thefinalglance\B-plan\B-24_phase_4I_media_validator.md
Date: 2026-05-11
FSM: PLANNING | Session: s-31180

SPEC REFERENCES
B-24 has NO formal "Spec references:" lines (spec_link_check.py: 0 found).
"## Reads" section (line 52) uses informal bullet list only.
phase-4I.txt         | GROUNDED      | file exists, read in full 624 lines -- all B-24 tasks traceable
media-validator.txt  | GROUNDED      | file exists, read in full 260 lines -- all B-24 tasks traceable
summary/integration-summary.md | VERIFY_NEEDED | relative path unresolved -- secondary only, acceptable
summary/phase-4-summary.md     | VERIFY_NEEDED | relative path unresolved -- secondary only, acceptable

CLAIM CLASSIFICATION
"Mode A: Qwen3.5-VL-30B-A3B via ollama ~17GB Q4"          | VERIFY      | media-validator.txt L63 says ~17GB. Caller reports measured ~19-23.8GB on RTX 3090. Discrepancy unresolved. See ACTIONS.
"ollama tag: qwen3.5-vl"                                   | MISMATCH    | media-validator.txt L20/L61 uses "Qwen3.5-VL-30B-A3B". Actual ollama tag is "qwen3-vl:30b-a3b-instruct". PRE_GATE must use real tag.
"Mode B: InternVL3.5-38B SGLang 2x 3090 tensor parallel"  | MISMATCH    | phase-4I.txt L99 prescribes TP. InternVL3 vision encoder does NOT support TP per SGLang issue #7854. Spec prescribes infeasible config.
"B-24 feedback loop max_iterations default 5"              | MISMATCH    | B-24 L96 says default 5. media-validator.txt section 7.3 step 7 and phase-4I.txt L501 both say default 3.
"Mode B: InternVL3.5-38B via SGLang only (RadixAttention)" | GROUNDED    | media-validator.txt L33-34: "MUST use SGLang"; phase-4I.txt L27-29
"Tier 1 <100ms deterministic"                              | GROUNDED    | media-validator.txt L39; phase-4I.txt L9
"Tier 2 ~2-5s advisory"                                    | GROUNDED    | media-validator.txt L48; phase-4I.txt L9
"SSIM/LPIPS via ONNX Runtime CPU EP only (not GPU)"        | GROUNDED    | phase-4I.txt L38-39 R-NEW-5, L255-257; ORT #4611 #22818 cited; ort-cpu-ep-interface.md pre-verified
"SSIM epsilon 1e-6 not bit-exact"                          | GROUNDED    | phase-4I.txt L256
"CLAP ~200M params ~0.4GB FP16 ~0.8-1.0GB FP32"           | GROUNDED    | media-validator.txt L53: exact match
"CLAP: Xenova/clap-htsat-unfused ONNX"                     | GROUNDED    | media-validator.txt L54: exact match
"CLAP input: raw PCM not FFmpeg-decoded"                   | GROUNDED    | media-validator.txt L55
"Qwen3.5-VL-30B-A3B MoE: all 30B stored in VRAM"          | GROUNDED    | media-validator.txt L62; phase-4I.txt L94-95
"Tier1 fail->Fail; Tier1+Tier2 disagree->HumanReview"      | GROUNDED    | media-validator.txt L50-51; phase-4I.txt L277-282
"VLM output constrained decoding JSON schema"              | GROUNDED    | phase-4I.txt L108-116; sglang-2312.07104-interface.md pre-verified
"Bounded concurrency max 4 Tier1 max 1 Tier2 queue 64"     | GROUNDED    | phase-4I.txt L295-298
"Lazy loading via ToolRegistry::register_lazy"             | GROUNDED    | phase-4I.txt L289-294
"MediaRenderAdapter trait 5 built-in adapters"             | GROUNDED    | media-validator.txt section 7.1-7.2 (L97-145)
"WebRenderAdapter Playwright headless CDP"                 | GROUNDED    | media-validator.txt L112-118; playwright-cdp-interface.md pre-verified
"BlenderRenderAdapter --background --python"               | GROUNDED    | media-validator.txt L123; blender-cli-interface.md pre-verified
"ComfyUIAdapter POST /prompt timeout 600s"                 | GROUNDED    | media-validator.txt L81-83, L145; phase-4I.txt L244
"FFmpeg key frame extraction animation adapter"            | GROUNDED    | media-validator.txt L129-134
"Convergence stop: issues unchanged between iterations"    | GROUNDED    | media-validator.txt section 7.3 step 7 L169-170; phase-4I.txt L500
"Max iterations default 3 (spec ground truth)"             | GROUNDED    | media-validator.txt L169; phase-4I.txt L501
"Design2Code/SeeAct screenshot-based evaluation"           | GROUNDED    | design2code-2403.03163-interface.md + seeact-2401.01614-interface.md pre-verified
"SSIM ImageMagick+image crate+ONNX"                        | GROUNDED    | ort-cpu-ep-interface.md + imagemagick-tesseract-interface.md pre-verified
"architecture-overview.txt section 9 line 283 STALE"       | GROUNDED    | phase-4I.txt L41-45 explicitly documents staleness; no action needed
"ollama >= 0.12.7 for Qwen3-VL"                           | UNGROUNDED  | phase-4I.txt L48 states this; zero source citation; not verified in any spec or paper
"Mode B single-GPU Q4 ~21GB fallback"                      | GROUNDED    | phase-4I.txt L99: "or single GPU at Q4 (~21GB)"

ACTIONS REQUIRED
- MISMATCH (highest priority): phase-4I.txt L99 prescribes "2x 3090 tensor parallel" for Mode B (InternVL3.5-38B). SGLang issue #7854 states InternVL3 vision encoder does NOT support tensor parallel. Spec prescribes infeasible config. Resolution: use single-GPU Q4 path (~21GB, already in spec as fallback at L99) or verify SGLang >= 0.4 resolved #7854. Ingest: grep sglang CHANGELOG/issues before EXECUTING.
- MISMATCH: B-24 L96 says feedback loop "default 5" iterations. media-validator.txt section 7.3 step 7 and phase-4I.txt L501 both say "default 3". Fix B-24 L96 before EXECUTING.
- MISMATCH: ollama model tag. Spec text "Qwen3.5-VL-30B-A3B" is the model name, not the ollama pull tag. Actual tag is "qwen3-vl:30b-a3b-instruct". PRE_GATE at 4I.5 must use `ollama pull qwen3-vl:30b-a3b-instruct`. Add correct tag to PRE_GATE before EXECUTING.
- VERIFY: Mode A VRAM ~17GB Q4_K_M (media-validator.txt L63). Caller reports measured ~19-23.8GB on RTX 3090. If real upper bound is ~23.8GB it still fits on a 24GB 3090 but with <1GB headroom -- marginal. Verify with `ollama ps` / `nvidia-smi` after loading. Update metrics/phase_4I.toml field vram_mode_a_gb with measured value before EXECUTING.
- UNGROUNDED: "ollama >= 0.12.7 for Qwen3-VL" (phase-4I.txt L48). No source. Verify: check github.com/ollama/ollama/releases for Qwen3-VL support version. Add citation to phase-4I.txt before EXECUTING.

VALIDATION WARNINGS
- B-24 has zero formal "Spec references:" lines -- spec_link_check.py returns 0/0/0. Informal Reads section only. All claims verified manually against primary spec files.
- summary/integration-summary.md and summary/phase-4-summary.md cited in B-24 Reads but paths unresolved (relative "summary/"). Secondary references only; primary specs verified directly.
- FSM state PLANNING consistent with pre-EXECUTING grounding sweep.

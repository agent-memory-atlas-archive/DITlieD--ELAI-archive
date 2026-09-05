# B-24: Phase 4I — Media Validator (conditional)
Objective: implement Phase 4I per spec — Tier 1 deterministic (CPU,
<100ms) + Tier 2 VLM advisory (GPU, ~2-5s). Mode A: Qwen3.5-VL-30B-A3B
single image. Mode B: InternVL3.5-38B SGLang sequences. CLAP for audio.
External tools: ImageMagick, FFmpeg, Tesseract, Blender CLI. Render-
Capture-Evaluate visual feedback loop.
Estimated effort: 2-3 sessions.
Pre-requisites: B-22 (autonomous mode for visual feedback iteration);
B-7 (PipelineSpec with media steps surfaces).
Parallel-safe with: B-23, B-25, B-26, B-27, B-28.

## Context

Per spec phase-4I.txt and integration/media-validator.txt
(10 sub-steps: 4I.1-4I.6 + 4I.7-4I.10 Render-Capture-Evaluate):

CONDITIONAL — only loaded when PipelineSpec has media-producing steps.
Lazy loading enforced.

Two-tier:
- Tier 1 deterministic CPU (<100ms): format, resolution, SSIM/LPIPS,
  vertex counts, sample rates. ImageMagick + Blender CLI + FFmpeg +
  Tesseract. SSIM/LPIPS via ONNX Runtime CPU EP only (GPU non-deterministic).
- Tier 2 VLM advisory (GPU ~2-5s): advisory only, Tier 1 has veto.
  Mode A: Qwen3.5-VL-30B-A3B via ollama (~17GB Q4). 
  Mode B: InternVL3.5-38B via SGLang only (~38GB 8-bit).
  Constrained decoding enforces VlmValidationOutput JSON schema.

CLAP for audio-text similarity (~200M params, ~0.4GB FP16, always-loaded
when audio active).

External tools: graceful degradation. Bounded concurrency: max 4 Tier 1,
max 1 Tier 2, queue depth 64.

Veto logic: Tier1 fail → Fail. Tier1 pass + Tier2 disagree →
HumanReview. VLM unavailable → Degraded.

Render-Capture-Evaluate (4I.7-4I.10):
- MediaRenderAdapter trait abstracts "render output to image(s)"
- Adapters per output type:
  - Playwright for web
  - Blender CLI for 3D
  - FFmpeg for animation
  - Direct file read for screenshots
  - ComfyUI for AI-generated
- Visual feedback loop: execute → render → VLM evaluates → structured
  feedback → executor iterates. Max iterations before escalation.
- Convergence check stops when issues unchanged between iterations.

≥28 tests (20 contract/functional + 8 adversarial). Lazy loading enforced.

## Reads

- F:\.ELAI_workflow\docs\specs\guide\steps\phase-4I.txt
- F:\.ELAI_workflow\docs\specs\integration\media-validator.txt
- summary/integration-summary.md (media-validator section)
- summary/phase-4-summary.md PHASE 4I

## Tasks

1. **Lazy module loading**: media_validator only instantiated when
   PipelineSpec has media steps. Add MediaSpec field to PipelineSpec
   (per 4D); on TaskRunner construction, check + instantiate.

2. **Tier 1 deterministic checks** (4I.1-4I.3):
   - Format/resolution checks via ImageMagick (identify)
   - SSIM/LPIPS via ONNX Runtime CPU EP (NOT GPU per spec)
   - 3D vertex/triangle counts via Blender CLI
   - Audio sample rate via FFmpeg
   - OCR text extraction via Tesseract (when expected_text present)
   - All <100ms

3. **Tier 2 VLM** (4I.4-4I.5):
   - Mode A: Qwen3.5-VL-30B-A3B via ollama. Constrained decoding for
     VlmValidationOutput JSON schema.
   - Mode B: InternVL3.5-38B via SGLang. Same schema.
   - Mode selection deterministic (single image vs sequence).
   - Bounded concurrency (max 1 Tier 2 in flight).

4. **CLAP audio-text** (always-loaded when audio active):
   - 200M params ~0.4GB FP16
   - Cosine similarity score audio embedding vs text embedding

5. **Veto logic** (4I.6): Tier1 fail veto Tier2; Tier1 pass + Tier2
   disagree → HumanReview SSB action; VLM unavailable → Degraded.

6. **MediaRenderAdapter trait** (4I.7):
   - render(spec) → Vec<RenderedImage>
   - 5 adapters: Playwright (web), BlenderCLI (3D), FFmpeg (animation),
     direct read (screenshots), ComfyUI (AI-generated)

7. **Visual feedback loop** (4I.8-4I.9):
   - execute → render → VLM evaluate → structured feedback → executor
     iterates
   - Max iterations (configurable, default 5) before escalation
   - Convergence stop: issues unchanged between iterations

8. **Dashboard surface**: MediaValidationPanel shows per-step Tier1
   results, Tier2 advisory + JSON output, render preview thumbnails,
   iteration count + last issues.

## WIRING_CONTRACT

produces_symbols:
  - MediaValidator + Tier1Validator + Tier2Validator
  - 5 MediaRenderAdapter impls
  - Visual feedback loop runner
  - SSB action_types: media_validation_started, media_tier1_complete,
    media_tier2_complete, render_complete, visual_feedback_iteration,
    media_human_review_required
consumes_symbols:
  - PipelineSpec with MediaSpec
  - dashboard MediaValidationPanel
reachable_from:
  - task_runner with media-producing PipelineSpec
stub_ban:
  - SSIM/LPIPS via GPU disallowed (CPU EP only per spec)
  - VLM Tier 2 vetoing Tier 1 disallowed (Tier 1 always wins)
  - Bounded concurrency violated disallowed
frontend_pair: MediaValidationPanel + render preview thumbnails
observable_effect:
  trigger: submit task that produces an image
  expect_ssb: media_tier1_complete + media_tier2_complete + render_complete
              within iteration budget
  expect_artifact: dashboard shows per-step thumbnails + Tier1/Tier2 results

## DETERMINISTIC_CHECK

```bash
# 1. MediaValidator lazy
rg "fn new_if_media_steps" .elai_cc/crates/orchestrator/src
# 2. ≥28 tests
cargo test --package elai-orchestrator media 2>&1 | grep "test result"
# Expected: ≥28 tests.
```

## ACCEPTANCE

- [ ] Lazy loading enforced (media_validator absent when no media steps)
- [ ] Tier 1 <100ms per check
- [ ] Tier 2 advisory only; Tier 1 has veto
- [ ] CLAP for audio
- [ ] 5 MediaRenderAdapter impls
- [ ] Visual feedback loop with convergence stop
- [ ] Bounded concurrency (max 4 Tier 1, max 1 Tier 2)
- [ ] HumanReview SSB action when Tier1+Tier2 disagree
- [ ] dashboard MediaValidationPanel renders thumbnails + per-step results
- [ ] 28+ tests pass

## Files touched

- ADD: crates/orchestrator/src/media_validator.rs
- ADD: crates/orchestrator/src/media_adapters/* (5 adapters)
- MODIFY: crates/orchestrator/src/ssb.rs (6 new action_types)
- MODIFY: crates/dashboard-v2/src-tauri/src/lib.rs (V2 — was V1
  crates/dashboard/src/commands.rs); ADD crates/dashboard-v2/src/lib/
  components/MediaValidationPanel.svelte (V2 has no equivalent yet —
  add as a new component consumed by SettingsWindow/MonitorWindow)
- ADD: deps onnx-runtime, image, etc. (with confirmation)
- ADD: scripts/media_models_setup.sh (download VLM if user opts in)
- CREATE: workflow/reasoning/B-24-decisions.md

## Next

B-25 (4J Empire Integration).

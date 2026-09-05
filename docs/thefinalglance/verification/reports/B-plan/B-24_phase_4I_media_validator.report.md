# Verification Report: B-24_phase_4I_media_validator.md
Source: F:\.ELAI_workflow\docs\thefinalglance\B-plan\B-24_phase_4I_media_validator.md
Date: 2026-05-17
Overall status: PARTIAL
Confidence: HIGH

## Summary
The core Rust backend for B-24 is shipped and structurally complete: the module
tree exists, all WIRING_CONTRACT symbols are present, the six SSB action types
are wired in ssb.rs, all 5 MediaRenderAdapter impls are present, bounded
concurrency constants are correct, and the dashboard Svelte panel exists. However,
CLAP audio and both Tier 2 VLM backends (OllamaTier2, SglangTier2) remain stubs
returning NotImplemented. The setup script media_models_setup.sh is absent.

## Claims verified DONE

- MediaValidator module exists — evidence: .elai_cc/crates/orchestrator/src/media_validator/mod.rs (entire module directory)
- new_if_media_steps lazy constructor present — evidence: .elai_cc/crates/orchestrator/src/media_validator/mod.rs:109
- new_if_media_steps_stub for testing present — evidence: .elai_cc/crates/orchestrator/src/media_validator/mod.rs:58
- new_if_media_steps wired into task_runner — evidence: .elai_cc/crates/orchestrator/src/task_runner.rs:4470 (comment "lazy, first task")
- MediaSpec field added to PipelineSpec / research_tree — evidence: .elai_cc/crates/orchestrator/src/research_tree.rs:620 (DEF_STRUCT MediaSpec), task_runner.rs:13840 (Vec::<MediaSpec>::new())
- Tier1Validator / Tier 1 deterministic checks in tier1.rs — evidence: .elai_cc/crates/orchestrator/src/media_validator/tier1.rs (ImageMagick, SSIM, audio sample-rate)
- SSIM present in Tier 1 via tier1.rs:660 (SSIM= format string) — CPU EP only per types comment
- MAX_TIER1_CONCURRENT=4, MAX_TIER2_CONCURRENT=1, VALIDATION_QUEUE_DEPTH=64 — evidence: .elai_cc/crates/orchestrator/src/media_validator/types.rs:63,66,69
- Semaphore bounded concurrency enforced — evidence: .elai_cc/crates/orchestrator/src/media_validator/mod.rs:138-146 (validation_queue_semaphore)
- MediaRenderAdapter trait defined — evidence: .elai_cc/crates/orchestrator/src/media_validator/render_adapters/mod.rs:84
- 5 MediaRenderAdapter impls present: WebRenderAdapter (Playwright), BlenderRenderAdapter (3D), AnimationRenderAdapter (FFmpeg), UiScreenshotAdapter (direct read), ComfyUIRenderAdapter (AI-generated) — evidence: render_adapters/web.rs:247, blender.rs:251, animation.rs:133, ui_screenshot.rs:43, comfyui.rs:92
- VlmBackend trait + SsbMediaSink present — evidence: .elai_cc/crates/orchestrator/src/media_validator/mod.rs:1334 (IMPL_TRAIT SsbMediaSink: MediaSsbSink)
- SsbMediaSink wired in task_runner — evidence: .elai_cc/crates/orchestrator/src/task_runner.rs:4470
- All 6 SSB action types present in ssb.rs — evidence: .elai_cc/crates/orchestrator/src/ssb.rs:1072-1077, 1251-1256 (media_validation_started, media_tier1_complete, media_tier2_complete, render_complete, visual_feedback_iteration, media_human_review_required)
- VisualFeedbackLoop in feedback_loop.rs with convergence check — evidence: .elai_cc/crates/orchestrator/src/media_validator/feedback_loop.rs:184 (run_with_sink), :276 (convergence_key)
- HumanReview SSB action when Tier1+Tier2 disagree — evidence: .elai_cc/crates/orchestrator/src/media_validator/mod.rs:399,518 (tier2_disagrees_with_tier1 branches), ssb.rs:762
- on_visual_feedback_iteration callback present — evidence: .elai_cc/crates/orchestrator/src/media_validator/mod.rs:257, :1393
- MediaValidationPanel.svelte present and mounted in MonitorWindow — evidence: .elai_cc/crates/dashboard-v2/src/lib/components/MediaValidationPanel.svelte:1; elai.map line 53316 (MonitorWindow imports MediaValidationPanel)
- Dashboard render_frames grid (thumbnails) present — evidence: MediaValidationPanel.svelte:255-270 (each entry.render_frames as frame, img with file:// src)
- Tier2 structured JSON output surfaced in dashboard — evidence: MediaValidationPanel.svelte:128 (entry.tier2.structured_output)
- ClapAudioModel type exists with cosine_similarity utility — evidence: .elai_cc/crates/orchestrator/src/media_validator/clap.rs:17,65
- OllamaTier2 (Mode A) and SglangTier2 (Mode B) structs and Tier2Validator impls defined — evidence: .elai_cc/crates/orchestrator/src/media_validator/tier2.rs:35,65
- 28+ tests present (actual count: 81 across 8 test files) — evidence: media_validator_contract.rs(13), media_validator_functional.rs(15), media_validator_adversarial.rs(14), media_validator_rce_contract.rs(15), media_validator_rce_functional.rs(7), media_validator_rce_integration.rs(3), media_validator_bench.rs(14) = 81 total
- Decision doc referenced (B-24-decisions.md) at multiple locations — evidence: .elai_cc/crates/orchestrator/src/media_validator/mod.rs:107, types.rs:488, tests/media_validator_adversarial.rs:239

## Claims PARTIAL

- ClapAudioModel for audio-text similarity (~200M params, ~0.4GB FP16, always-loaded)
  - exists: .elai_cc/crates/orchestrator/src/media_validator/clap.rs:17
  - gap: load_model(), embed_audio(), embed_text() all return NotImplemented; real ONNX inference deferred to Group C3. cosine_similarity math is real (clap.rs:65-84). Audio is not actually loaded/run.

- OllamaTier2 / SglangTier2 VLM backends (Mode A / Mode B)
  - exists: .elai_cc/crates/orchestrator/src/media_validator/tier2.rs:35,65
  - gap: both validate() impls return Err(Tier2Error::NotImplemented("... Group C5")). Constrained decoding, real HTTP calls to ollama/SGLang, and VlmValidationOutput JSON schema enforcement are not wired. Contract tests use MockVlmBackend only.

## Claims MISSING

- scripts/media_models_setup.sh (download VLM if user opts in) — searched: glob **/*media_models_setup* under .elai_cc; no file found

## Claims UNVERIFIABLE

- Tier 1 all-checks complete in <100ms — reason: no timing data in codebase; bench tests exist (media_validator_bench.rs) but results require running cargo test
- SSIM/LPIPS via ONNX Runtime CPU EP (NOT GPU) fully wired — reason: SSIM format string exists at tier1.rs:660 but ONNX session setup not confirmed from map snippets alone; clap.rs explicitly defers to Group C3 for ort::Session

## What must be done (action list)

- [ ] Implement ClapAudioModel::load_model, embed_audio, embed_text in .elai_cc/crates/orchestrator/src/media_validator/clap.rs (Group C3 scope)
- [ ] Implement OllamaTier2::validate HTTP calls to ollama /api/generate in .elai_cc/crates/orchestrator/src/media_validator/tier2.rs (Group C5 scope)
- [ ] Implement SglangTier2::validate HTTP calls to SGLang in .elai_cc/crates/orchestrator/src/media_validator/tier2.rs (Group C5 scope)
- [ ] Add constrained decoding for VlmValidationOutput JSON schema enforcement in Tier 2 (Group C5)
- [ ] Create .elai_cc/scripts/media_models_setup.sh for VLM model download opt-in

## Notes
The "5 adapter" breakdown differs slightly from the plan: plan names "direct file read for screenshots" — the impl is UiScreenshotAdapter (ui_screenshot.rs), which reads screenshots via direct file I/O, matching the intent. "Playwright for web" is WebRenderAdapter (web.rs). All 5 are correctly present.
The e2e cloud mode test file (media_validator_e2e_cloud_mode.rs) uses tokio::test attributes not matched by the #[test] pattern; its tests are counted in elai.map MACRO rows but not in the PowerShell #[test] grep — actual test count is >= 81.
OllamaTier2/SglangTier2 stubs are an explicit group-scoped deferral (C5), not forgotten — the plan's "Groups C1-C9" structure is internal B-24 sequencing not visible in the B-plan file itself.

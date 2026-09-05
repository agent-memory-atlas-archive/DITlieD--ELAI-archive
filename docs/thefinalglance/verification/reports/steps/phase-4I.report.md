# Verification Report: phase-4I
Source: F:\.ELAI_workflow\docs\specs\guide\steps\phase-4I.txt
Date: 2026-05-17
Overall status: MOSTLY_DONE
Confidence: HIGH

## Summary
Phase 4I (Media Validator) is substantially complete. All core types (4I.1), Tier-1 validators (4I.5), Tier-2 VLM config, all 5 render adapters (4I.7), VisualFeedbackLoop (4I.8/4I.9), and the metrics file (4I.6/4I.10) are implemented. Two adversarial test functions (a5 VLM sandbox escape, a7 ComfyUI timeout) remain #[ignore]'d pending Tier-C sandbox and Group C6 infrastructure. The metrics file stores 4I.10 RCE keys under [metrics.rce] sub-table rather than the spec-required flat key names (rce_web_render_p95_ms, etc.), creating a minor schema gap versus the DETERMINISTIC_CHECK grep expectation.

## Deliverables verified DONE

- MediaValidationRequest struct — evidence: crates/orchestrator/src/media_validator/types.rs:167
- MediaValidationResult struct — evidence: crates/orchestrator/src/media_validator/types.rs:292
- Tier1Result and Tier2Result structs — evidence: crates/orchestrator/src/media_validator/types.rs:213,220
- Verdict enum (Pass|Fail|HumanReview|Degraded) — evidence: crates/orchestrator/src/media_validator/types.rs:282
- Tier1Validator trait — evidence: crates/orchestrator/src/media_validator/tier1.rs:44
- ExternalToolWrapper trait with check_available() and execute() — evidence: crates/orchestrator/src/media_validator/external_tools.rs:65,71,75
- ImageMagickWrapper, FFprobeWrapper, TesseractWrapper, BlenderWrapper — evidence: crates/orchestrator/src/media_validator/external_tools.rs:183,227,270,313
- ToolCapabilityFlags bitflags with IMAGEMAGICK, FFPROBE, FFMPEG, TESSERACT, BLENDER, PLAYWRIGHT (6+ flags) — evidence: crates/orchestrator/src/media_validator/types.rs:313-319
- VlmProcessConfig struct — evidence: crates/orchestrator/src/media_validator/types.rs:403
- VlmValidationOutput struct — evidence: crates/orchestrator/src/media_validator/types.rs:230
- VlmMode (A|B) and VlmEngine (Ollama|Sglang) enums — evidence: crates/orchestrator/src/media_validator/types.rs:336,345
- Mode B + Ollama structurally impossible (no ModeB(OllamaConfig) arm) — evidence: crates/orchestrator/src/media_validator/types.rs:372,664
- ClapConfig struct — evidence: crates/orchestrator/src/media_validator/types.rs:533
- ClapAudioModel implementation — evidence: crates/orchestrator/src/media_validator/clap.rs:18,22
- ImageValidator, Model3DValidator, AudioValidator, UiScreenshotValidator (all impl Tier1Validator) — evidence: crates/orchestrator/src/media_validator/tier1.rs:306,426,508,607
- VlmProcessConfig hardware guard (HardwareInsufficient error) — evidence: crates/orchestrator/src/media_validator/types.rs:415,424,442,458,476
- MediaRenderAdapter trait — evidence: crates/orchestrator/src/media_validator/render_adapters/mod.rs:84
- AdapterRegistry struct — evidence: crates/orchestrator/src/media_validator/render_adapters/mod.rs:129
- AdapterRegistry::from_capabilities() — evidence: crates/orchestrator/src/media_validator/render_adapters/mod.rs:195
- WebRenderAdapter stub — evidence: crates/orchestrator/src/media_validator/render_adapters/web.rs:131
- BlenderRenderAdapter stub — evidence: crates/orchestrator/src/media_validator/render_adapters/blender.rs:120
- AnimationRenderAdapter stub — evidence: crates/orchestrator/src/media_validator/render_adapters/animation.rs:47
- UiScreenshotAdapter stub — evidence: crates/orchestrator/src/media_validator/render_adapters/ui_screenshot.rs:33
- ComfyUIRenderAdapter stub — evidence: crates/orchestrator/src/media_validator/render_adapters/comfyui.rs:47
- VisualFeedbackLoop struct — evidence: crates/orchestrator/src/media_validator/feedback_loop.rs:104
- CapturedFrame struct — evidence: crates/orchestrator/src/media_validator/render_adapters/types.rs:61
- new_if_media_steps lazy-load gate — evidence: crates/orchestrator/src/media_validator/mod.rs:109
- SsbMediaSink (SSB visual_feedback_iteration emission) — evidence: crates/orchestrator/src/media_validator/mod.rs:1262,1334
- Contract tests (media_validator_contract.rs: 13 tests) — evidence: crates/orchestrator/tests/media_validator_contract.rs
- Functional tests (media_validator_functional.rs) — evidence: crates/orchestrator/tests/media_validator_functional.rs
- Adversarial tests file (13 active + 2 #[ignore]'d) — evidence: crates/orchestrator/tests/media_validator_adversarial.rs
- RCE contract tests (15 tests) — evidence: crates/orchestrator/tests/media_validator_rce_contract.rs
- RCE functional tests (7 tests) — evidence: crates/orchestrator/tests/media_validator_rce_functional.rs
- RCE integration tests (3 tests) — evidence: crates/orchestrator/tests/media_validator_rce_integration.rs
- E2E cloud mode test — evidence: crates/orchestrator/tests/media_validator_e2e_cloud_mode.rs
- Benchmark harness — evidence: crates/orchestrator/tests/media_validator_bench.rs
- Metrics file exists with tier1_p95_ms, tier2_mode_a/b_p95_ms, gpu_swap fields, test_count=95 (>=28), adversarial_test_count=14 (>=8) — evidence: .elai_cc/metrics/phase_4I.toml:64-93
- RCE metrics collected (animation=59ms, ui=0ms, loop=984ms, convergence_rate=0.80) — evidence: .elai_cc/metrics/phase_4I.toml:174-214
- Tier 1 p95 breach documented exception (subprocess probe, fixed via OnceLock cache) — evidence: .elai_cc/metrics/phase_4I.toml:32-46,71-73

## Deliverables PARTIAL

- Adversarial scenario a5 (VLM sandbox escape) — #[ignore]'d
  - exists: crates/orchestrator/tests/media_validator_adversarial.rs (a5_vlm_sandbox_escape_blocked)
  - gap: requires Tier-C sandbox (B-22) + vlm-integration feature; test is present but skipped; metrics.toml adversarial_scenarios.vlm_sandbox_escape_blocked = false

- Adversarial scenario a7 (ComfyUI timeout cleanup) — conditionally #[ignore]'d
  - exists: crates/orchestrator/tests/media_validator_adversarial.rs (a7_comfyui_timeout_cleanup)
  - gap: metrics.toml adversarial_scenarios.comfyui_timeout_cleanup = false; test present but depends on Group C6 infrastructure; on Windows, real ComfyUI process spawn is constrained

- 4I.10 RCE metrics key naming — spec DETERMINISTIC_CHECK grep expects flat keys
  - exists: .elai_cc/metrics/phase_4I.toml under [metrics.rce] sub-table with keys web_render_p95_ms, blender_render_p95_ms, animation_render_p95_ms, loop_total_p95_ms, convergence_rate, avg_iterations, test_count
  - gap: spec L592-593 lists rce_web_render_p95_ms, rce_blender_render_p95_ms, rce_animation_render_p95_ms, rce_loop_total_p95_ms, rce_convergence_rate, rce_avg_iterations, rce_test_count as expected grep targets at top-level; actual keys are nested under [metrics.rce] without the rce_ prefix; values are present and correct but key path differs from spec expectation

- Web and Blender adapter render latency benchmarks — TOOL_MISSING on current host
  - exists: metrics.toml [metrics.rce] web_render_p95_ms = "skipped: TOOL_MISSING:playwright", blender_render_p95_ms = "skipped: TOOL_MISSING:blender"
  - gap: no real measured number; spec L600 accepts documented exception for CI; CI with Playwright/Blender would populate; values are sentinel strings, not numeric measurements

## Deliverables MISSING

- None identified. All structural deliverables are implemented.

## Deliverables UNVERIFIABLE

- Real VLM latency (Mode A / Mode B tier2_mode_a_p95_ms, tier2_mode_b_p95_ms): recorded as "null" sentinel — requires CUDA device with 24GB+ VRAM (Phase 0 host is 12GB). Spec accepts this with documented exception.
- GPU swap cost (gpu_swap_mode_a_ms, gpu_swap_mode_b_ms): same hardware constraint; recorded as "null".
- Phase 4I compile/test pass status: auditor role only; no test run performed per task scope.

## What must be done (action list)

- [ ] Rename or alias RCE metric keys in .elai_cc/metrics/phase_4I.toml [metrics] top-level to match spec L592-593 exactly: rce_web_render_p95_ms, rce_blender_render_p95_ms, rce_animation_render_p95_ms, rce_loop_total_p95_ms, rce_convergence_rate, rce_avg_iterations, rce_test_count (or confirm DETERMINISTIC_CHECK grep is updated to use the nested path metrics.rce.*)
- [ ] Resolve a5 (vlm_sandbox_escape_blocked): either implement Tier-C sandbox for VLM processes (B-22 dependency) or document as blocked-by-B22 in EXIT CRITERIA explicitly
- [ ] Resolve a7 (comfyui_timeout_cleanup): enable on non-Windows CI or document permanent skip rationale in EXIT CRITERIA

## Notes

- ToolCapabilityFlags has FFPROBE as a separate flag from FFMPEG (both present); spec mentions FFPROBE in check logic and FFMPEG in adapter wiring — both exist, no gap.
- The VlmEngine::Sglang (not SGLang) casing is consistent internally; Mode B type-safety is enforced via enum structure (no ModeB(OllamaConfig) constructor exists).
- Test count totals from metrics: 95 total (71 lib + 21 RCE + others), well above spec floors of 28 base + 15 RCE = 43.
- adversarial_test_count = 14 (spec requires >= 8); 2 of 8 spec scenarios are #[ignore]'d but the count still exceeds the floor due to additional adversarial tests added.
- SSB wiring via SsbMediaSink is confirmed present; visual_feedback_iteration action_type is emitted per feedback loop iteration per spec 4I.9.

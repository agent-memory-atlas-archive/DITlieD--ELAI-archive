B-24 Phase 4I Group C9 -- Final Integration Notes
====================================================

Scope
-----
This document is the reference card for B-24 (Phase 4I Media Validator)
final integration + cloud-mode operation. Generated as part of Group C9.
See workflow/reasoning/B-24-decisions.md (entries 93+) for design
rationales.

Cloud-mode invocation recipe for media validation
--------------------------------------------------

1. Set environment variables (Bash / PowerShell variants below).

   Bash:
     export ANTHROPIC_API_KEY=<your-key>
     # Cloud mode is the default; do NOT set ANTHROPIC_BASE_URL
     # (that variable is reserved for sidecar override).

   PowerShell:
     $env:ANTHROPIC_API_KEY = "<your-key>"
     # Cloud mode is the default; do NOT set $env:ANTHROPIC_BASE_URL

2. Run elai in cloud mode (cloud is the default routing -- the cloud
   mode router is in .elai_cc/crates/orchestrator/src/cloud_mode.rs):

     F:/.ELAI_workflow/.elai_cc/target/release/elai.exe \
       --model claude-sonnet-4-5-20250929 \
       "Render a UI screenshot of the dashboard MediaValidationPanel \
        and validate it via the Tier-1 image validator."

3. Verify the request hits the cloud:

     F:/.ELAI_workflow/.elai_cc/target/release/elai.exe doctor

   Expected:
     Doctor / Auth / Status = ok
     Doctor / Config / Status = ok

4. Watch the SSB events stream live (orchestrator/cli side):

     F:/.ELAI_workflow/.elai_cc/target/release/elai.exe \
       --resume latest /status

Required environment variables
------------------------------

  ANTHROPIC_API_KEY  -- required for cloud-mode AI calls. Read by the
                        cli during runtime construction and forwarded
                        to the orchestrator's CloudModeRouter.
                        Source: workflow/sessions/<id>/cloud-creds.toml
                        OR keyring (preferred); env var is the dev path.

  ANTHROPIC_BASE_URL -- DO NOT SET for cloud mode. Reserved for sidecar
                        override (local) per CLAUDE.md remote-sidecar block.

  ELAI_LOCAL_MAX_CONTEXT -- only meaningful for sidecar mode; cloud mode
                        uses the cloud_200k profile by default
                        (cloud_mode.rs DEFAULT_CLOUD_MODEL_ID).

How to verify the dashboard shows media events
----------------------------------------------

The dashboard subscriber (crates/dashboard-v2/src-tauri/src/media_validation.rs)
taps the same Arc<Ssb> the orchestrator's MediaValidator writes to. To
verify:

1. Open the dashboard:
     cd F:/.ELAI_workflow/.elai_cc/crates/dashboard-v2 && npm run tauri dev

2. Submit a media-producing task via the cli (or the dashboard's
   command palette). Examples that hit the media path:
     - "Render the chat orbit preview at 1280x720"
     - "Capture a UI screenshot of the ResearchTree panel"

3. Open the MediaValidationPanel in the dashboard (Explore window).
   The panel should populate with one row per validated artifact,
   each carrying:
     - media_type (image / audio / ui_screenshot / model3d)
     - file_path
     - tier1 status (pass + check_count + latency_ms)
     - tier2 status (advisory_pass + model_used + latency_ms)
     - final_verdict (Pass / Fail / HumanReview / Degraded)
     - render_frames (RCE adapters; only when visual_check=true)
     - feedback_iterations (RCE visual-feedback loop)

4. SSB action_types observed in the dashboard subscriber:
     media_validation_started
     media_tier1_complete
     media_tier2_complete
     render_complete
     visual_feedback_iteration
     media_human_review_required

Known limitations (deferred to follow-up B-N tasks)
---------------------------------------------------

* HumanReview decisions are NOT persistent across dashboard restart.
  The submit_media_validation_for_review Tauri command updates the
  in-memory MediaValidationState only; the persistent record is the
  orchestrator's next SSB row. Deferred per C8 audit decision 92
  follow-up.

* Mode A / Mode B real VLM (Qwen2.5-VL-72B / InternVL3-78B) is gated
  behind the orchestrator's `vlm-integration` Cargo feature. Default
  CI build ships MockVlmBackend only -- enabling vlm-integration
  requires CUDA + 24 GB+ VRAM and pulls in `ort` + `nvml-wrapper`.

* Tier C sandbox (subprocess isolation for the Tier-2 VLM process) is
  a B-22 dependency. Today the VLM either runs in-process (Mock) or
  via the user's local ollama/sglang daemon. Subprocess-sandboxed VLM
  arrives with B-22's autonomous-gate work.

* MediaValidator + SsbMediaSink construction happens at most once per
  TaskRunner lifetime (slot is `tokio::sync::Mutex<Option<...>>` on
  ResolvedConfig). Subsequent tasks reuse the handle; the lazy gate
  (spec L289-294) is honoured -- empty PipelineSpec.media skips
  construction entirely.

* The wire-up emits the MediaValidationStarted + MediaTier1Complete +
  MediaTier2Complete SSB rows for every pipeline::validate_with_sink
  call. RenderComplete and VisualFeedbackIteration are emitted by the
  RCE path (visual_check=true on QualitySpec); MediaHumanReviewRequired
  fires when Tier-1 passes and Tier-2 disagrees on a covered category.

Reproducer script outline (cloud-mode media smoke)
--------------------------------------------------

# 1) Confirm the binary boots without panic.
F:/.ELAI_workflow/.elai_cc/target/release/elai.exe --version
F:/.ELAI_workflow/.elai_cc/target/release/elai.exe --help | head -40
F:/.ELAI_workflow/.elai_cc/target/release/elai.exe doctor

# 2) Build + run the e2e mock test (NO network, NO API key).
cd F:/.ELAI_workflow/.elai_cc
cargo test -p elai-orchestrator --test media_validator_e2e_cloud_mode \
    -- --test-threads=1

# Expected: 2 passed, 1 ignored (cloud_mode_live).

# 3) (OPTIONAL) Run the live cloud-mode test.
#    Requires ANTHROPIC_API_KEY in the env. Will hit
#    api.anthropic.com:443 with a 16-token request to
#    claude-sonnet-4-5-20250929.
export ANTHROPIC_API_KEY=<your-key>   # or $env:ANTHROPIC_API_KEY in PowerShell
cargo test -p elai-orchestrator --test media_validator_e2e_cloud_mode \
    -- --ignored cloud_mode_live --nocapture

# Expected: "cloud_mode_live: SUCCESS - status=200 OK, model=claude-sonnet-4-5-..."

# 4) Verify no regression in the rest of the suite.
cargo test -p elai-orchestrator --lib media_validator -- --test-threads=1
cargo test -p elai-orchestrator --features media-validator-rce \
    --test media_validator_rce_contract \
    --test media_validator_rce_functional \
    --test media_validator_rce_integration -- --test-threads=1
cargo test -p elai-dashboard-v2 --lib -- --test-threads=1

# Expected: 71 lib + 11+7+3=21 RCE + 19 dashboard, all PASS.

Wire-up locations (canonical)
-----------------------------

* SsbMediaSink (production sink, Arc<Ssb>-backed):
    .elai_cc/crates/orchestrator/src/media_validator/mod.rs
    Class: SsbMediaSink. Constructor: SsbMediaSink::new(Arc<Ssb>, agent_id).
    Sync-to-async via tokio::runtime::Handle::try_current() + spawn.

* Lazy wire-up gate (decision 94):
    Function: media_validator::wire_media_validator_for_spec(
                  media_steps_count, capabilities, vlm_backend).
    Returns Option<MediaValidator>; None when media_steps_count == 0.

* Task-runner integration:
    .elai_cc/crates/orchestrator/src/task_runner.rs L3658
    (drive_task, immediately after spec_from_decomposition).
    Slots: ResolvedConfig.media_validator + ResolvedConfig.media_sink
    (both AsyncMutex<Option<Arc<...>>>, populated at most once per runner).

* Dashboard subscriber (consumer):
    .elai_cc/crates/dashboard-v2/src-tauri/src/media_validation.rs
    Function: spawn_subscriber(app, ssb_writer, media_state).
    Subscribes to ssb_writer.subscribe() and folds the 6 media_* rows
    into MediaValidationState.

Spec compliance summary (phase-4I.txt)
---------------------------------------

* L288-298 (Lazy loading): wire_media_validator_for_spec returns None
  when media_steps_count == 0. Pinned by
  wire_up_emits_six_media_ssb_events_through_real_ssb test.

* L613 (single RuntimeClient): SsbMediaSink wraps the same Arc<Ssb> the
  task_runner constructs once at startup. Pinned by
  sink_and_subscriber_share_the_same_ssb_handle test.

* L351 (Tier-1 < 100 ms budget): post-C5 Arc<ToolCapabilityFlags> cache
  collapses warm-path p95 to 0 ms; cold-start p99 = 1054 ms. See
  metrics/phase_4I.toml [metrics] tier1_p95_ms.

* L223-226 (bounded queue): per-process tokio::sync::Semaphore(64)
  guards pipeline::validate; QueueFull returned immediately when saturated.

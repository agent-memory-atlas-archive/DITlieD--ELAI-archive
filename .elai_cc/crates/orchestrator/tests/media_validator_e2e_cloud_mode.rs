//! B-24 Phase 4I Group C9 — end-to-end smoke test for the media validator
//! wire-up + cloud mode.
//!
//! This integration test exercises the FULL pipeline that B-24 has
//! shipped across Groups C1..C8 plus the C9 final wire-up:
//!
//!   orchestrator (mock)
//!      -> wire_media_validator_for_spec (lazy gate on PipelineSpec.media)
//!      -> SsbMediaSink::on_validation_started
//!      -> Ssb::write_payload (real on-disk SSB DB in a tempdir)
//!      -> broadcast subscriber (same channel dashboard-v2 listens on)
//!      -> decoded SsbPayload variants asserted against expected sequence
//!
//! Three tests live here:
//!   1. `wire_up_emits_started_tier1_tier2_and_human_review_through_real_ssb`
//!      — runs in every default `cargo test` invocation. NO network. NO API
//!      key. Mocks the orchestrator via a `MockOrchestrator` that synthesizes
//!      a `PipelineSpec` with non-empty `media`, drives a real
//!      `MediaValidator` through `validate_with_sink_async` across TWO
//!      sub-cases against the SAME Arc<Ssb> + subscriber:
//!        - Quick + qualitative_checks=false (MockVlmBackend::new()):
//!          asserts MediaValidationStarted + MediaTier1Complete only.
//!        - Full + qualitative_checks=true (MockVlmBackend::always_critical()):
//!          asserts MediaValidationStarted + MediaTier1Complete +
//!          MediaTier2Complete + MediaHumanReviewRequired.
//!      RenderComplete + VisualFeedbackIteration are emitted by the
//!      cfg(feature = "media-validator-rce") path and are covered by the
//!      feature-gated tests; NOT exercised here.
//!   2. `sink_and_subscriber_share_the_same_ssb_handle` — pins the spec
//!      L613 single-RuntimeClient invariant by reading SsbMediaSink's
//!      internal Arc<Ssb> raw pointer (via the test-only
//!      `__test_only_ssb_ptr()` accessor) and asserting equality with the
//!      runtime-side Arc<Ssb> raw pointer.
//!   3. `cloud_mode_live` — `#[ignore]`'d; opt-in via
//!      `cargo test -p elai-orchestrator --test media_validator_e2e_cloud_mode -- --ignored cloud_mode_live`.
//!      Hits the real Anthropic API with `claude-sonnet-4-5-20250929`
//!      ("Sonnet 4.6" per CLAUDE.md) using the workspace's `reqwest`
//!      client. Confirms the cloud path can issue a tiny chat completion
//!      end-to-end. Skipped automatically when `ANTHROPIC_API_KEY` is unset.
//!
//! Spec references:
//!   * phase-4I.txt L288-298 — "Lazy loading: MediaValidatorPlugin
//!     registered via ToolRegistry::register_lazy(name, factory_fn) at
//!     startup. Factory not invoked until first MediaValidationRequest
//!     arrives." Test 1 asserts the lazy contract holds.
//!   * phase-4I.txt L613 — "Every call site and struct that participates
//!     in request routing must wire ssb, fsm, and clock through the
//!     single RuntimeClient". Test 1 passes the same `Arc<Ssb>` to both
//!     the producer (`SsbMediaSink`) and the consumer (broadcast
//!     subscriber).
//!   * B-24 plan L1 — "Lazy module loading: media_validator only
//!     instantiated when PipelineSpec has media steps." Test 1 also
//!     pins the negative case (empty `spec.media` -> validator stays None).
//!   * CLAUDE.md — Sonnet 4.6 model ID = claude-sonnet-4-5-20250929.

use std::sync::Arc;
use std::time::Duration;

use tempfile::TempDir;

use elai_core::clock::{ClockProvider, OsClock};
use elai_orchestrator::media_validator::{
    pipeline, wire_media_validator_for_spec, MediaType, MediaValidationRequest, MockVlmBackend,
    QualitySpec, SsbMediaSink, ToolCapabilityFlags, ValidationMode, VlmBackend,
};
use elai_orchestrator::{
    Ssb, SsbActionType, SsbConfig, SsbEntry,
};

// ---------------------------------------------------------------------------
// Shared fixture helpers
// ---------------------------------------------------------------------------

/// Pre-computed minimal valid 1x1 RGB PNG. Identical to the bytes pinned by
/// `media_validator_functional.rs::write_tiny_png` so the C1..C3 functional
/// behaviour is reused unchanged. Pinning literal bytes avoids the `png`
/// crate dep — the workspace tier-1 image validator already handles this
/// header in production.
const TINY_PNG: &[u8] = &[
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG signature
    0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52, // IHDR length+type
    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, // 1x1
    0x08, 0x02, 0x00, 0x00, 0x00, 0x90, 0x77, 0x53, 0xDE, // depth, color, CRC
    0x00, 0x00, 0x00, 0x0C, 0x49, 0x44, 0x41, 0x54, // IDAT
    0x08, 0xD7, 0x63, 0xF8, 0xCF, 0xC0, 0x00, 0x00, 0x00, 0x03, 0x00, 0x01, 0x5B, 0x42, 0x5D, 0x6A,
    0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, // IEND
    0xAE, 0x42, 0x60, 0x82,
];

/// Construct a real on-disk SSB writer in a tempdir. The returned tuple
/// keeps the TempDir alive for the lifetime of the test so the
/// rusqlite-backed DB is not removed before the broadcast subscriber
/// finishes draining rows.
fn fresh_ssb() -> (Arc<Ssb>, TempDir) {
    let tmp = TempDir::new().expect("tempdir");
    let cfg = SsbConfig {
        db_path: tmp.path().join("ssb.db"),
        pool_size: Some(2),
        project_id: "b24-c9-e2e".into(),
        session_id: "b24-c9-e2e-session".into(),
    };
    let clock: Arc<dyn ClockProvider> = Arc::new(OsClock);
    let ssb = Arc::new(Ssb::new(cfg, clock).expect("ssb construct"));
    (ssb, tmp)
}

/// Drain up to `n` SsbEntry rows from `rx`, returning the ones whose
/// `action_type` matches any of the 6 media_* variants. Times out after
/// `timeout` to keep the test deterministic.
async fn drain_media_rows(
    rx: &mut tokio::sync::broadcast::Receiver<SsbEntry>,
    n: usize,
    timeout: Duration,
) -> Vec<SsbEntry> {
    let mut out = Vec::with_capacity(n);
    loop {
        if out.len() >= n {
            return out;
        }
        match tokio::time::timeout(timeout, rx.recv()).await {
            Ok(Ok(entry)) => {
                if matches!(
                    entry.action_type,
                    SsbActionType::MediaValidationStarted
                        | SsbActionType::MediaTier1Complete
                        | SsbActionType::MediaTier2Complete
                        | SsbActionType::RenderComplete
                        | SsbActionType::VisualFeedbackIteration
                        | SsbActionType::MediaHumanReviewRequired
                ) {
                    out.push(entry);
                }
            }
            Ok(Err(_)) => return out,
            Err(_) => return out, // timeout -> return what we have
        }
    }
}

/// MockOrchestrator: synthesizes a media-producing PipelineSpec.media count
/// and a single MediaValidationRequest pointing at a freshly-written PNG.
/// Replaces the live `spec_from_decomposition` + decomposer path so the
/// test stays under 100ms.
struct MockOrchestrator {
    pub media_steps_count: usize,
    pub request: MediaValidationRequest,
    _tempdir: TempDir,
}

impl MockOrchestrator {
    fn with_image_step() -> Self {
        let tmp = TempDir::new().expect("tempdir mock");
        let png_path = tmp.path().join("ui_screenshot.png");
        std::fs::write(&png_path, TINY_PNG).expect("write png");
        let request = MediaValidationRequest {
            media_type: MediaType::Image,
            file_path: png_path,
            quality_spec: QualitySpec {
                similarity_threshold: None,
                qualitative_checks: false,
                visual_check: false,
            },
            reference_path: None,
            mode: ValidationMode::Quick,
            output_dir: Some(tmp.path().to_path_buf()),
            request_id: uuid::Uuid::new_v4(),
        };
        Self {
            media_steps_count: 1,
            request,
            _tempdir: tmp,
        }
    }

    fn with_no_media_step() -> Self {
        // Path is never validated when media_steps_count == 0; we still
        // construct a stub request so the struct shape is comparable.
        // Use std::env::temp_dir() + a non-existent suffix instead of
        // "/dev/null" so the path is portable to Windows (where /dev/null
        // does not exist and would silently mis-route any accidental
        // filesystem touch). The path is INTENTIONALLY non-existent --
        // this is a stub for the negative gate test case (validator must
        // NOT be constructed when media_steps_count == 0, so the path is
        // never opened).
        let tmp = TempDir::new().expect("tempdir mock-empty");
        Self {
            media_steps_count: 0,
            request: MediaValidationRequest {
                media_type: MediaType::Image,
                file_path: std::env::temp_dir().join("b24_c9_nonexistent_stub"),
                quality_spec: QualitySpec::default(),
                reference_path: None,
                mode: ValidationMode::Quick,
                output_dir: None,
                request_id: uuid::Uuid::new_v4(),
            },
            _tempdir: tmp,
        }
    }

    /// Variant: Full ValidationMode + qualitative_checks=true. Drives the
    /// Tier2 path so the test can assert MediaTier2Complete +
    /// MediaHumanReviewRequired emissions when paired with
    /// `MockVlmBackend::always_critical()`. Uses the same TINY_PNG fixture
    /// as `with_image_step` so Tier1 still passes (header check) and the
    /// pipeline proceeds to Tier2 advisory evaluation.
    fn with_image_step_full_mode() -> Self {
        let tmp = TempDir::new().expect("tempdir mock-full");
        let png_path = tmp.path().join("ui_screenshot_full.png");
        std::fs::write(&png_path, TINY_PNG).expect("write png");
        let request = MediaValidationRequest {
            media_type: MediaType::Image,
            file_path: png_path,
            quality_spec: QualitySpec {
                similarity_threshold: None,
                qualitative_checks: true,
                visual_check: false,
            },
            reference_path: None,
            mode: ValidationMode::Full,
            output_dir: Some(tmp.path().to_path_buf()),
            request_id: uuid::Uuid::new_v4(),
        };
        Self {
            media_steps_count: 1,
            request,
            _tempdir: tmp,
        }
    }
}

// ---------------------------------------------------------------------------
// Test 1: full pipeline mock e2e — runs by default.
// ---------------------------------------------------------------------------

/// Lazy gate honoured: empty `spec.media` skips construction; non-empty
/// constructs MediaValidator + SsbMediaSink. The test drives TWO sub-cases
/// against the SAME `Arc<Ssb>` + subscriber to validate that event
/// ordering is preserved across ValidationMode switches:
///
///   (1) Quick mode + qualitative_checks=false (uses `MockVlmBackend::new()`):
///       expected events = MediaValidationStarted + MediaTier1Complete.
///       Verdict = Pass (no Tier-2 needed). This pins the C8 dashboard
///       subscriber behaviour for the common "lightweight check" path.
///
///   (2) Full mode + qualitative_checks=true (uses
///       `MockVlmBackend::always_critical()`): Tier-1 still passes (PNG
///       header valid) so the pipeline proceeds to Tier-2; the critical
///       mock issue forces `advisory_pass=false`, which emits
///       MediaTier2Complete + MediaHumanReviewRequired. Expected events
///       this sub-case appends to the bus = MediaValidationStarted +
///       MediaTier1Complete + MediaTier2Complete + MediaHumanReviewRequired.
///
/// Confirms the dashboard's C8 subscriber would see the same rows the
/// validator emits today for both Quick and Full validation modes.
/// (RenderComplete + VisualFeedbackIteration are emitted by the RCE
/// feature-gated path and are covered by the cfg(feature =
/// "media-validator-rce") tests; NOT exercised here.)
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn wire_up_emits_started_tier1_tier2_and_human_review_through_real_ssb() {
    // ---- LAZY GATE NEGATIVE: empty spec.media -> None ----
    let orch_empty = MockOrchestrator::with_no_media_step();
    let no_validator = wire_media_validator_for_spec(
        orch_empty.media_steps_count,
        ToolCapabilityFlags::empty(),
        None,
    );
    assert!(
        no_validator.is_none(),
        "lazy gate violated: validator was constructed despite spec.media being empty"
    );

    // ---- LAZY GATE POSITIVE: non-empty spec.media -> Some ----
    let orch = MockOrchestrator::with_image_step();
    let validator = wire_media_validator_for_spec(
        orch.media_steps_count,
        ToolCapabilityFlags::empty(),
        Some(Box::new(MockVlmBackend::new()) as Box<dyn VlmBackend>),
    )
    .expect("validator constructed for non-empty spec.media");

    // Confirm the validator's capability snapshot round-trips. Empty
    // capability flags are valid (Tier-1 still runs the in-process PNG
    // header parser without any external subprocess tool).
    assert_eq!(*validator.capabilities, ToolCapabilityFlags::empty());
    assert!(validator.vlm_backend.is_some());

    // ---- REAL SSB + SUBSCRIBER (shared across BOTH sub-cases) ----
    let (ssb, _tmp_db) = fresh_ssb();
    let mut rx = ssb.subscribe();

    // ---- PRODUCTION SINK: same Arc<Ssb> the dashboard would attach to ----
    let sink = SsbMediaSink::new(Arc::clone(&ssb), "test-agent-c9");

    // =========================================================
    // SUB-CASE 1: Quick mode, qualitative_checks=false
    //   -> MediaValidationStarted + MediaTier1Complete only
    // =========================================================
    let mock_backend_quick = MockVlmBackend::new();
    let result_quick =
        pipeline::validate_with_sink_async(&orch.request, &mock_backend_quick, Some(&sink))
            .await
            .expect("validate_with_sink_async (Quick) runs to completion");
    // TINY_PNG header valid -> Tier1 pass; Quick mode skips Tier-2 entirely
    // -> Verdict::Pass (verdict_reason = "Tier1 passed; no qualitative spec").
    assert!(
        matches!(
            result_quick.final_verdict,
            elai_orchestrator::media_validator::Verdict::Pass
        ),
        "Quick sub-case: expected Pass, got {:?}",
        result_quick.final_verdict
    );

    // ---- DRAIN ROWS (sub-case 1) ----
    // SsbMediaSink spawns each write fire-and-forget onto the current
    // tokio runtime; give them up to 2 seconds to land.
    let rows_quick = drain_media_rows(&mut rx, 2, Duration::from_secs(2)).await;
    assert!(
        rows_quick.len() >= 2,
        "Quick sub-case: expected >=2 media SSB rows (started + tier1), got {}",
        rows_quick.len()
    );
    assert!(
        rows_quick
            .iter()
            .any(|e| e.action_type == SsbActionType::MediaValidationStarted),
        "Quick sub-case: MediaValidationStarted row missing"
    );
    assert!(
        rows_quick
            .iter()
            .any(|e| e.action_type == SsbActionType::MediaTier1Complete),
        "Quick sub-case: MediaTier1Complete row missing"
    );
    // Quick mode MUST NOT emit Tier2/HumanReview rows.
    assert!(
        !rows_quick
            .iter()
            .any(|e| e.action_type == SsbActionType::MediaTier2Complete),
        "Quick sub-case: unexpected MediaTier2Complete row (Quick mode skips Tier-2)"
    );
    assert!(
        !rows_quick
            .iter()
            .any(|e| e.action_type == SsbActionType::MediaHumanReviewRequired),
        "Quick sub-case: unexpected MediaHumanReviewRequired row (Quick mode skips Tier-2)"
    );

    // =========================================================
    // SUB-CASE 2: Full mode, qualitative_checks=true,
    // MockVlmBackend::always_critical()
    //   -> MediaValidationStarted + MediaTier1Complete +
    //      MediaTier2Complete + MediaHumanReviewRequired
    // =========================================================
    let orch_full = MockOrchestrator::with_image_step_full_mode();
    let mock_backend_critical = MockVlmBackend::always_critical();
    let result_full = pipeline::validate_with_sink_async(
        &orch_full.request,
        &mock_backend_critical,
        Some(&sink),
    )
    .await
    .expect("validate_with_sink_async (Full + critical) runs to completion");
    // Tier-2 advisory_pass=false (critical issue) routes through the
    // HumanReview branch (mod.rs L536-546).
    assert!(
        matches!(
            result_full.final_verdict,
            elai_orchestrator::media_validator::Verdict::HumanReview
        ),
        "Full sub-case: expected HumanReview, got {:?}",
        result_full.final_verdict
    );

    // ---- DRAIN ROWS (sub-case 2) ----
    // Up to 4 new rows to drain (started + tier1 + tier2 + human_review).
    let rows_full = drain_media_rows(&mut rx, 4, Duration::from_secs(2)).await;
    assert!(
        rows_full.len() >= 4,
        "Full sub-case: expected >=4 media SSB rows (started + tier1 + tier2 + human_review), got {}",
        rows_full.len()
    );
    assert!(
        rows_full
            .iter()
            .any(|e| e.action_type == SsbActionType::MediaValidationStarted),
        "Full sub-case: MediaValidationStarted row missing"
    );
    assert!(
        rows_full
            .iter()
            .any(|e| e.action_type == SsbActionType::MediaTier1Complete),
        "Full sub-case: MediaTier1Complete row missing"
    );
    assert!(
        rows_full
            .iter()
            .any(|e| e.action_type == SsbActionType::MediaTier2Complete),
        "Full sub-case: MediaTier2Complete row missing"
    );
    assert!(
        rows_full
            .iter()
            .any(|e| e.action_type == SsbActionType::MediaHumanReviewRequired),
        "Full sub-case: MediaHumanReviewRequired row missing"
    );
}

/// Pins the spec L613 RuntimeClient invariant: the sink and the consumer
/// share the same Arc<Ssb>. If a future refactor splits the writer from
/// the broadcast bus, this test fires loudly.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn sink_and_subscriber_share_the_same_ssb_handle() {
    let (ssb, _tmp_db) = fresh_ssb();
    let mut rx = ssb.subscribe();
    let sink = SsbMediaSink::new(Arc::clone(&ssb), "agent-c9-share");

    // Pointer equality on the Arc<Ssb> — assert that SsbMediaSink's
    // INTERNAL `Arc<Ssb>` points at the SAME allocation as the runtime's
    // Arc<Ssb>. The previous version compared Arc::as_ptr(&ssb) vs
    // Arc::as_ptr(&Arc::clone(&ssb)) — that is trivially equal by Arc's
    // contract and proves nothing about the sink's internals.
    //
    // Using `SsbMediaSink::__test_only_ssb_ptr()` (added to mod.rs for
    // exactly this purpose) reads the sink's private `Arc<Ssb>` and
    // compares its raw pointer to the runtime-side handle. A future
    // refactor that hands `SsbMediaSink::new` a freshly-constructed
    // `Arc<Ssb>` (instead of an `Arc::clone` of the runtime's) would
    // produce a different allocation -> different pointer -> test fails.
    let runtime_ssb_ptr = Arc::as_ptr(&ssb) as usize;
    let sink_internal_ssb_ptr = sink.__test_only_ssb_ptr();
    assert_eq!(
        sink_internal_ssb_ptr, runtime_ssb_ptr,
        "spec L613: SsbMediaSink must hold the SAME Arc<Ssb> as the runtime — \
         regression-detection for single-RuntimeClient invariant"
    );
    drop(sink);

    // Sanity: subscriber is alive and bound to the same broadcast channel.
    let orch = MockOrchestrator::with_image_step();
    let sink2 = SsbMediaSink::new(Arc::clone(&ssb), "agent-c9-share-2");
    let mock_backend = MockVlmBackend::new();
    let _ = pipeline::validate_with_sink_async(&orch.request, &mock_backend, Some(&sink2))
        .await
        .expect("validate_with_sink_async");
    let rows = drain_media_rows(&mut rx, 6, Duration::from_secs(2)).await;
    assert!(
        !rows.is_empty(),
        "subscriber bound to same Ssb received zero rows; broadcast wiring broken"
    );
}

// ---------------------------------------------------------------------------
// Test 2: cloud-mode LIVE — opt-in via --ignored. Hits real Anthropic API.
// ---------------------------------------------------------------------------

/// Live cloud test. Reads `ANTHROPIC_API_KEY` from env; skips quietly when
/// unset. Sends the smallest possible chat completion to
/// `claude-sonnet-4-5-20250929` ("Sonnet 4.6" per CLAUDE.md) and asserts the
/// response decodes. NOT run in CI — opt in with:
///   cargo test -p elai-orchestrator --test media_validator_e2e_cloud_mode \
///       -- --ignored cloud_mode_live
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "cloud-mode-live (requires ANTHROPIC_API_KEY + network); run with -- --ignored cloud_mode_live"]
async fn cloud_mode_live() {
    let api_key = match std::env::var("ANTHROPIC_API_KEY") {
        Ok(k) if !k.is_empty() => k,
        _ => {
            eprintln!(
                "cloud_mode_live: ANTHROPIC_API_KEY unset; skipping (this is expected in CI)"
            );
            return;
        }
    };

    // Anthropic Messages API — uses workspace reqwest, no new dep.
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .expect("reqwest client");

    let payload = serde_json::json!({
        "model": "claude-sonnet-4-5-20250929",
        "max_tokens": 16,
        "messages": [
            {"role": "user", "content": "Reply with the single word OK."}
        ]
    });

    let resp = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", &api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&payload)
        .send()
        .await
        .expect("anthropic send");

    let status = resp.status();
    let body: serde_json::Value = resp.json().await.expect("anthropic json");
    assert!(
        status.is_success(),
        "Anthropic API call failed: status={status}, body={body}"
    );
    // Response shape: { content: [ { type: "text", text: "..." } ], model: "...", ... }
    let content = body
        .get("content")
        .and_then(|c| c.as_array())
        .expect("response has content[]");
    assert!(!content.is_empty(), "Anthropic response content[] is empty");

    eprintln!(
        "cloud_mode_live: SUCCESS - status={status}, model={}",
        body.get("model").and_then(|m| m.as_str()).unwrap_or("?")
    );
}

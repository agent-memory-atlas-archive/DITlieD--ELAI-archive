//! B-24 Phase 4I (Group C1): Tier-2 (VLM advisory) skeleton.
//!
//! Two backends are stubbed: `OllamaTier2` for Mode A (single-image, fast)
//! and `SglangTier2` for Mode B (sequence / multi-image). Real HTTP code
//! lands in Group C5; here we only need the trait surface and a working
//! `select_validator` so 4I.2 compiles and the C2 contract tests can
//! exercise the veto-logic branches via MockVlmBackend.

use super::mock_vlm::VlmBackend;
use super::types::{
    MediaType, MediaValidationRequest, OllamaConfig, SglangConfig, Tier2Error, Tier2Result,
    VlmMode,
};

/// Tier-2 advisory validator — async because the real impl POSTs to a
/// local VLM server. Native async fn in trait per CLAUDE.md anti-stale
/// rule (NO `async-trait` crate).
///
/// The trait is object-safe via Send + Sync bounds and a Pin<Box<Future>>
/// return — required because the dispatcher stores `Arc<dyn Tier2Validator>`.
pub trait Tier2Validator: Send + Sync {
    fn validate<'a>(
        &'a self,
        request: &'a MediaValidationRequest,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Tier2Result, Tier2Error>> + Send + 'a>,
    >;
}

// ---------------------------------------------------------------------------
// OllamaTier2 (Mode A)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct OllamaTier2 {
    pub config: OllamaConfig,
}

impl OllamaTier2 {
    pub fn new(config: OllamaConfig) -> Self {
        Self { config }
    }
}

impl Tier2Validator for OllamaTier2 {
    fn validate<'a>(
        &'a self,
        _request: &'a MediaValidationRequest,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Tier2Result, Tier2Error>> + Send + 'a>,
    > {
        Box::pin(async move {
            Err(Tier2Error::NotImplemented(
                "OllamaTier2::validate (Group C5)",
            ))
        })
    }
}

// ---------------------------------------------------------------------------
// SglangTier2 (Mode B)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct SglangTier2 {
    pub config: SglangConfig,
}

impl SglangTier2 {
    pub fn new(config: SglangConfig) -> Self {
        Self { config }
    }
}

impl Tier2Validator for SglangTier2 {
    fn validate<'a>(
        &'a self,
        _request: &'a MediaValidationRequest,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Tier2Result, Tier2Error>> + Send + 'a>,
    > {
        Box::pin(async move {
            Err(Tier2Error::NotImplemented(
                "SglangTier2::validate (Group C5)",
            ))
        })
    }
}

// ---------------------------------------------------------------------------
// MockVlmTier2 — wraps a VlmBackend so contract tests in C2 can exercise
// the veto-logic without a real VLM server.
// ---------------------------------------------------------------------------

pub struct MockTier2<B: VlmBackend> {
    pub backend: B,
}

impl<B: VlmBackend> MockTier2<B> {
    pub fn new(backend: B) -> Self {
        Self { backend }
    }
}

impl<B: VlmBackend + 'static> Tier2Validator for MockTier2<B> {
    fn validate<'a>(
        &'a self,
        request: &'a MediaValidationRequest,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Tier2Result, Tier2Error>> + Send + 'a>,
    > {
        Box::pin(async move {
            let advisory = self.backend.validate_advisory(&request.file_path);
            // Convert to JSON: serde_json on VlmValidationOutput is derived.
            let structured_output = serde_json::to_value(&advisory)
                .map_err(|e| Tier2Error::SchemaMismatch(e.to_string()))?;
            // advisory_pass = no Critical and overall_quality >= 0.5
            let any_critical = advisory
                .issues
                .iter()
                .any(|i| matches!(i.severity, super::types::Severity::Critical));
            let advisory_pass = !any_critical && advisory.overall_quality >= 0.5;
            Ok(Tier2Result {
                advisory_pass,
                structured_output,
                model_used: self.backend.model_id().to_owned(),
                latency_ms: 0,
            })
        })
    }
}

// ---------------------------------------------------------------------------
// Validator selection
// ---------------------------------------------------------------------------

/// Select Mode A or Mode B based on media type and whether the artifact
/// is a single image or a sequence (multi-frame video / animation /
/// multi-screenshot run). Heuristic per spec phase-4I.txt 4I.5:
///   * single image  -> Mode A (fast, low VRAM)
///   * sequence      -> Mode B (sequence reasoning)
///
/// `single_image` is determined upstream by inspecting the media artifact
/// (e.g. is it one PNG, or a directory of frames). The function is pure
/// so we can unit-test it without filesystem I/O.
pub fn select_validator(media_type: &MediaType, single_image: bool) -> VlmMode {
    match media_type {
        // Audio doesn't go through Tier 2 in the current spec; default to
        // Mode A so callers that pass it by mistake don't crash on .unwrap().
        // Group C5 will narrow this with a `Tier2Unsupported` variant.
        MediaType::Audio => VlmMode::A,
        MediaType::Image | MediaType::UiScreenshot if single_image => VlmMode::A,
        MediaType::Image | MediaType::UiScreenshot => VlmMode::B,
        MediaType::Model3D => VlmMode::B,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::media_validator::mock_vlm::MockVlmBackend;
    use crate::media_validator::types::{
        MediaValidationRequest, QualitySpec, ValidationMode,
    };
    use std::path::PathBuf;

    fn dummy_request(media_type: MediaType) -> MediaValidationRequest {
        MediaValidationRequest {
            media_type,
            file_path: PathBuf::from("test_fixture.png"),
            quality_spec: QualitySpec::default(),
            reference_path: None,
            mode: ValidationMode::Full,
            output_dir: None,
            request_id: uuid::Uuid::nil(),
        }
    }

    #[test]
    fn select_validator_single_image_picks_mode_a() {
        assert_eq!(select_validator(&MediaType::Image, true), VlmMode::A);
        assert_eq!(
            select_validator(&MediaType::UiScreenshot, true),
            VlmMode::A
        );
    }

    #[test]
    fn select_validator_sequence_picks_mode_b() {
        assert_eq!(select_validator(&MediaType::Image, false), VlmMode::B);
        assert_eq!(select_validator(&MediaType::Model3D, true), VlmMode::B);
    }

    #[test]
    fn mock_tier2_returns_pass_for_default_backend() {
        let mock = MockTier2::new(MockVlmBackend::new());
        let req = dummy_request(MediaType::Image);
        let fut = mock.validate(&req);
        // tiny block-on, std-only.
        let res = futures_block_on(fut);
        let r = res.expect("mock advisory");
        assert!(r.advisory_pass);
        assert_eq!(r.model_used, "mock-vlm");
    }

    #[test]
    fn mock_tier2_fails_when_critical() {
        let mock = MockTier2::new(MockVlmBackend::always_critical());
        let req = dummy_request(MediaType::Image);
        let res = futures_block_on(mock.validate(&req));
        let r = res.expect("advisory");
        assert!(!r.advisory_pass);
    }

    /// Single-future executor backed by a tokio current-thread runtime. The
    /// previous hand-rolled `Pending => continue` loop with a Noop waker only
    /// worked when futures resolved immediately; once Group C3 introduces
    /// `tokio::process::Command`, real async I/O will yield `Pending` and the
    /// spin loop would hang. Tokio is already a workspace dependency with the
    /// `rt` feature, so no new crate is required.
    fn futures_block_on<F: std::future::Future>(fut: F) -> F::Output {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("build current-thread tokio runtime for test")
            .block_on(fut)
    }
}

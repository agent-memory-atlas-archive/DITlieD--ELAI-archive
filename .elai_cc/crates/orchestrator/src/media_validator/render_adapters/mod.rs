//! B-24 Phase 4I Group C5 (4I.7 SKELETON): Render-Capture-Evaluate (RCE)
//! adapter trait + registry.
//!
//! Spec source: docs/specs/guide/steps/phase-4I.txt L380-432. The trait
//! is intentionally object-safe so the `AdapterRegistry` can store
//! `Box<dyn MediaRenderAdapter>` (spec adversarial L425-427). The
//! `Pin<Box<Future>>` shape matches `Tier1Validator` (see
//! crates/orchestrator/src/media_validator/tier1.rs L44-56) — same
//! rationale: opaque `impl Future` from `async fn` is not dyn-compatible,
//! and the `async-trait` crate is rejected per CLAUDE.md anti-stale rules.
//!
//! The adapter modules (web/blender/animation/ui_screenshot/comfyui) are
//! re-exported below so callers can `use elai_orchestrator::media_validator
//! ::render_adapters::*` without walking the submodule tree.

pub mod animation;
pub mod blender;
pub mod comfyui;
pub mod types;
pub mod ui_screenshot;
pub mod web;

use std::collections::HashMap;
use std::path::Path;
use std::pin::Pin;

use thiserror::Error;

pub use self::animation::AnimationRenderAdapter;
pub use self::blender::BlenderRenderAdapter;
pub use self::comfyui::ComfyUIRenderAdapter;
pub use self::types::{
    CameraAngle, CapturedFrame, CaptureMode, ConvergenceKey, FrameMetadata, Interaction,
    OutputType, RenderConfig, Viewport, VisualIssue,
};
pub use self::ui_screenshot::UiScreenshotAdapter;
pub use self::web::WebRenderAdapter;

// ---------------------------------------------------------------------------
// Adapter errors.
// ---------------------------------------------------------------------------

/// Per-adapter render failure. Distinct from `MediaValidatorError` so
/// callers can branch on render-stage failures without conflating them
/// with the Tier1/Tier2 validation errors. `NotImplemented` is the
/// SKELETON default: Group C5 ships type surfaces, Group C6 fills in
/// real Playwright + Blender CLI + FFmpeg + ComfyUI bodies.
#[derive(Debug, Error)]
pub enum RenderAdapterError {
    #[error("render adapter not implemented yet (Group C6): {0}")]
    NotImplemented(&'static str),

    #[error("required tool unavailable: {tool}")]
    ToolUnavailable { tool: &'static str },

    #[error("artifact path invalid or unreadable: {path:?}")]
    ArtifactPathInvalid { path: std::path::PathBuf },

    #[error("adapter returned malformed output: {0}")]
    MalformedOutput(String),

    #[error("render timed out after {millis} ms")]
    Timeout { millis: u64 },

    #[error("io: {0}")]
    Io(#[from] std::io::Error),

    #[error("duplicate adapter for output_type: {0:?}")]
    DuplicateAdapter(OutputType),
}

// ---------------------------------------------------------------------------
// MediaRenderAdapter trait.
// ---------------------------------------------------------------------------

/// Render-Capture-Evaluate adapter interface. One concrete impl per
/// `OutputType` per spec phase-4I.txt L380-385 + L396-401.
///
/// Object-safety: the trait MUST be dyn-compatible because the
/// `AdapterRegistry` stores `Box<dyn MediaRenderAdapter>`. The
/// `Pin<Box<dyn Future + Send + 'a>>` signature on `render()` keeps the
/// trait object-safe AND lets multi-thread tokio executors poll the
/// returned future.
pub trait MediaRenderAdapter: Send + Sync {
    /// True iff this adapter handles the given output type. Used by
    /// `AdapterRegistry::get_adapter` for the deterministic lookup
    /// (spec L382 + contract test `rce_c1_adapter_selection_deterministic`).
    fn can_handle(&self, output_type: &OutputType) -> bool;

    /// Render the artifact under `config` and emit one or more
    /// `CapturedFrame` entries. Returns `Err(NotImplemented)` for the
    /// SKELETON variants (Web / Blender / Animation / ComfyUI); the
    /// real bodies land in Group C6. `UiScreenshotAdapter::render`
    /// is the only REAL impl in Group C5.
    fn render<'a>(
        &'a self,
        artifact: &'a Path,
        config: &'a RenderConfig,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<Vec<CapturedFrame>, RenderAdapterError>> + Send + 'a>>;

    /// Adapter-specific cleanup (e.g. kill leaked Playwright browser
    /// processes, remove tempdir). Default impl is a no-op.
    /// Spec adversarial L490-491 ("If adapter cleanup() is not called
    /// after loop completion, Playwright browser processes leak").
    fn cleanup(&self) {}

    /// Human-readable adapter name, surfaced into
    /// `FrameMetadata.adapter_name` and the SSB
    /// `RenderCompletePayload.adapter_name`.
    fn name(&self) -> &'static str;
}

// ---------------------------------------------------------------------------
// AdapterRegistry.
// ---------------------------------------------------------------------------

/// Registry mapping `OutputType` to the (lazily-instantiated) adapter.
/// Built once at MediaValidator startup from `ToolCapabilityFlags`
/// (spec L406: "Built at startup from ToolCapabilityFlags").
///
/// Adversarial contract (spec L428-429): registering twice for the same
/// `OutputType` MUST be rejected — silent replacement would mask plugin
/// conflicts. We expose two registration entry points:
///   * `register` — last-write-wins (kept for backward compatibility
///     with the bare HashMap API). Returns the previous adapter.
///   * `register_unique` — errors on duplicate registration.
///
/// Tests live under `media_validator_rce_contract.rs`.
pub struct AdapterRegistry {
    adapters: HashMap<OutputType, Box<dyn MediaRenderAdapter>>,
}

impl Default for AdapterRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl AdapterRegistry {
    pub fn new() -> Self {
        Self {
            adapters: HashMap::new(),
        }
    }

    /// Look up the adapter for `output_type`. `None` when no adapter is
    /// registered (callers should treat this as a hard validation
    /// failure per spec L450 contract `rce_c3_no_output_type_rejected`).
    pub fn get_adapter(&self, output_type: &OutputType) -> Option<&dyn MediaRenderAdapter> {
        self.adapters.get(output_type).map(|b| b.as_ref())
    }

    /// Insert / replace the adapter for `output_type`. Returns the
    /// previous adapter when one existed. Use `register_unique` for
    /// plugin-safe registration.
    pub fn register(
        &mut self,
        output_type: OutputType,
        adapter: Box<dyn MediaRenderAdapter>,
    ) -> Option<Box<dyn MediaRenderAdapter>> {
        self.adapters.insert(output_type, adapter)
    }

    /// Like `register` but errors if `output_type` is already mapped.
    /// Spec adversarial L428-429.
    pub fn register_unique(
        &mut self,
        output_type: OutputType,
        adapter: Box<dyn MediaRenderAdapter>,
    ) -> Result<(), RenderAdapterError> {
        if self.adapters.contains_key(&output_type) {
            return Err(RenderAdapterError::DuplicateAdapter(output_type));
        }
        self.adapters.insert(output_type, adapter);
        Ok(())
    }

    /// Total registered adapters. Used by `MediaValidator::new` to log
    /// how many output types have adapter coverage.
    pub fn len(&self) -> usize {
        self.adapters.len()
    }

    /// True when no adapters are registered.
    pub fn is_empty(&self) -> bool {
        self.adapters.is_empty()
    }

    /// Build the default registry from a capability snapshot. Adapters
    /// are registered ONLY when their backing tool is available; this
    /// matches the spec L406 invariant.
    ///
    /// `UiScreenshotAdapter` is always registered because it requires
    /// no external tool (file read only).
    pub fn from_capabilities(caps: &super::types::ToolCapabilityFlags) -> Self {
        use super::types::ToolCapabilityFlags;
        let mut r = Self::new();
        // UiScreenshot: always available (file read).
        let _ = r.register_unique(
            OutputType::UiScreenshot,
            Box::new(UiScreenshotAdapter::new()),
        );
        if caps.contains(ToolCapabilityFlags::PLAYWRIGHT) {
            let _ = r.register_unique(OutputType::Web, Box::new(WebRenderAdapter::new()));
        }
        if caps.contains(ToolCapabilityFlags::BLENDER) {
            let _ = r.register_unique(
                OutputType::Model3D,
                Box::new(BlenderRenderAdapter::new()),
            );
        }
        if caps.contains(ToolCapabilityFlags::FFMPEG) {
            let _ = r.register_unique(
                OutputType::Animation,
                Box::new(AnimationRenderAdapter::new()),
            );
        }
        if caps.contains(ToolCapabilityFlags::COMFYUI) {
            let _ = r.register_unique(
                OutputType::AiGenerated,
                Box::new(ComfyUIRenderAdapter::new()),
            );
        }
        r
    }
}

// ---------------------------------------------------------------------------
// Unit tests — object-safety + registry invariants.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Compile-time check that the trait is object-safe.
    #[test]
    fn trait_is_object_safe() {
        let adapter: Box<dyn MediaRenderAdapter> = Box::new(UiScreenshotAdapter::new());
        assert_eq!(adapter.name(), "ui_screenshot");
    }

    #[test]
    fn registry_register_unique_rejects_duplicates() {
        let mut r = AdapterRegistry::new();
        r.register_unique(OutputType::UiScreenshot, Box::new(UiScreenshotAdapter::new()))
            .expect("first registration succeeds");
        let err = r
            .register_unique(OutputType::UiScreenshot, Box::new(UiScreenshotAdapter::new()))
            .expect_err("second registration should error");
        match err {
            RenderAdapterError::DuplicateAdapter(OutputType::UiScreenshot) => {}
            other => panic!("expected DuplicateAdapter, got {other:?}"),
        }
    }

    #[test]
    fn registry_register_returns_previous() {
        let mut r = AdapterRegistry::new();
        assert!(r
            .register(OutputType::UiScreenshot, Box::new(UiScreenshotAdapter::new()))
            .is_none());
        // last-write-wins: returns the previous adapter.
        assert!(r
            .register(OutputType::UiScreenshot, Box::new(UiScreenshotAdapter::new()))
            .is_some());
        assert_eq!(r.len(), 1);
    }

    #[test]
    fn get_adapter_returns_none_for_unregistered() {
        let r = AdapterRegistry::new();
        assert!(r.get_adapter(&OutputType::Web).is_none());
    }

    #[test]
    fn from_capabilities_always_registers_ui_screenshot() {
        use super::super::types::ToolCapabilityFlags;
        let r = AdapterRegistry::from_capabilities(&ToolCapabilityFlags::empty());
        // UiScreenshot needs no external tool — always present.
        assert!(r.get_adapter(&OutputType::UiScreenshot).is_some());
        assert!(r.get_adapter(&OutputType::Web).is_none());
        assert!(r.get_adapter(&OutputType::Model3D).is_none());
    }
}

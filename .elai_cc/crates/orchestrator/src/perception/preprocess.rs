//! J-1.T1 — preprocess spec + per-target-model sweet-spot edges.
//!
//! Source plan: `docs/thefinalglance/J-plan/J-1_perception_gateway.md`
//! (lines 25-26, 56-60). The sweet-spot table is EXTERNAL figure SRC-3 /
//! HD-1 — DESIGN RATIONALE ONLY; never shipped as a measured number in any
//! user-facing claim or metrics TOML. Every shipped number is ELAI-measured
//! (re-verify on model swaps; G-10 canary is the runtime sibling).
//!
//! The Rust side orchestrates + records; heavy image ops (resize, JPEG
//! encode, tile, thumbnail) land on the sidecar in T5 (legs 2/4,
//! sidecar-gated W-10). This module ships the spec + the skeleton `preprocess`
//! fn; T5 wires the live sidecar call (replacing the WIRE-DARK[J-1] marker
//! with a real production caller).

use serde::{Deserialize, Serialize};

/// Per-target-model longest-side sweet spot, in pixels. EXTERNAL SRC-3 /
/// HD-1 — design rationale only, never a measured claim. The values are
/// pinned here with a dated source line so a model swap re-verifies them
/// (plan open-question line 167-168; G-10 canary is the runtime sibling).
///
/// Source: `vlm-vision-research-summary.md` (SRC-3, 2026-06-10 ingest).
/// Re-verify on model swaps.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TargetModel {
    /// Anthropic Claude — 1568px longest side (SRC-3).
    Claude,
    /// OpenAI GPT-V — 2048px longest side (SRC-3).
    Gpt,
    /// Qwen-VL — 1280px longest side (SRC-3).
    QwenVl,
}

impl TargetModel {
    /// Return the longest-side sweet-spot edge in pixels for this target
    /// model. EXTERNAL SRC-3 / HD-1 — design rationale only; the J-1
    /// measurement fixtures (T7) record ELAI-measured deltas, not these
    /// pinned rationale values.
    // WIRE-DARK[J-1]
    #[must_use]
    pub fn sweet_spot_longest_side_px(self) -> u32 {
        match self {
            // EXTERNAL SRC-3 — design rationale, not a measured claim.
            TargetModel::Claude => 1568,
            // EXTERNAL SRC-3 — design rationale, not a measured claim.
            TargetModel::Gpt => 2048,
            // EXTERNAL SRC-3 — design rationale, not a measured claim.
            TargetModel::QwenVl => 1280,
        }
    }

    /// Default target model used by [`PreprocessSpec::default`]. Claude
    /// (1568px) — the active production provider today.
    #[must_use]
    pub fn default_model() -> Self {
        TargetModel::Claude
    }
}

/// Preprocess spec applied by the sidecar preprocess step (T5 wires the
/// live sidecar call). All fields are pure data; no image-processing dep
/// is introduced on the Rust side (plan stub_ban: "NOT a new
/// image-processing dep in Rust without approval").
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PreprocessSpec {
    /// Target model whose sweet-spot edge drives the longest-side resize.
    pub target_model: TargetModel,
    /// JPEG quality 85 — PNG->RGB JPEG q85 cuts image tokens ~60% with
    /// negligible quality loss (EXTERNAL SRC-3; ELAI-measured deltas land
    /// in T7 fixtures, not here).
    pub jpeg_q85: bool,
    /// Tile overlap fraction in `[0.0, 1.0]`. 0.0 = no tiling (single
    /// image); >0.0 = overlapping tiles + a global thumbnail for dense
    /// UIs. T5 wires the sidecar tiler; the Rust side records the spec.
    pub tile_overlap: f32,
    /// When true, attach a global thumbnail alongside the (optionally
    /// tiled) full-resolution image(s). Mirrors the SRC-3 dense-UI recipe.
    pub thumbnail: bool,
}

impl Default for PreprocessSpec {
    fn default() -> Self {
        Self {
            target_model: TargetModel::default_model(),
            jpeg_q85: true,
            tile_overlap: 0.0,
            thumbnail: false,
        }
    }
}

/// Outcome of the preprocess step. The Rust side records this; the sidecar
/// (T5) performs the actual image ops.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PreprocessOutcome {
    /// Preprocess applied successfully (resize + JPEG + optional tile /
    /// thumbnail). The conditioned image bytes are available in
    /// [`PreprocessResult::conditioned_bytes`].
    Preprocessed,
    /// No image present in the input — preprocess is a no-op.
    NoImagePresent,
    /// Preprocess failed (corrupt image, sidecar unavailable, etc.). The
    /// raw image forwards UNCONDITIONED with an explicit marker — never a
    /// silent downscale fallthrough (plan stub_ban).
    DegradedPassthrough,
}

/// Result of the preprocess step. Carries the conditioned bytes on
/// [`PreprocessOutcome::Preprocessed`]; the raw bytes on
/// [`PreprocessOutcome::DegradedPassthrough`]; empty on
/// [`PreprocessOutcome::NoImagePresent`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreprocessResult {
    pub outcome: PreprocessOutcome,
    /// Conditioned image bytes (Preprocessed) or raw bytes
    /// (DegradedPassthrough). Empty on NoImagePresent.
    pub conditioned_bytes: Vec<u8>,
    /// Longest-side pixel edge applied (Preprocessed) or 0 otherwise.
    pub longest_side_px: u32,
}

/// Perception-module error type. thiserror — never `Box<dyn Error>`
/// (anti-stale-training rule).
#[derive(Debug, thiserror::Error)]
pub enum PerceptionError {
    #[error("preprocess failed: {0}")]
    PreprocessFailed(String),
    #[error("scaffold failed: {0}")]
    ScaffoldFailed(String),
    #[error("router failed: {0}")]
    RouterFailed(String),
    #[error("no image present in the input")]
    NoImagePresent,
}

// WIRE-DARK[J-1]
/// Preprocess the given image bytes per `spec`. T1 skeleton — T5 wires the
/// live sidecar call (legs 2/4, sidecar-gated W-10). The skeleton returns
/// `DegradedPassthrough` on any non-empty input (never a silent fallthrough
/// — plan stub_ban) and `NoImagePresent` on empty input.
///
/// # Errors
/// Returns [`PerceptionError::PreprocessFailed`] only if the spec is
/// internally inconsistent (e.g. `tile_overlap` outside `[0.0, 1.0]`).
/// Otherwise the skeleton degrades to passthrough rather than panicking.
#[must_use]
pub fn preprocess(spec: &PreprocessSpec, image_bytes: &[u8]) -> Result<PreprocessResult, PerceptionError> {
    if !(0.0..=1.0).contains(&spec.tile_overlap) {
        return Err(PerceptionError::PreprocessFailed(format!(
            "tile_overlap must be in [0.0, 1.0]; got {}",
            spec.tile_overlap
        )));
    }
    if image_bytes.is_empty() {
        return Ok(PreprocessResult {
            outcome: PreprocessOutcome::NoImagePresent,
            conditioned_bytes: Vec::new(),
            longest_side_px: 0,
        });
    }
    // T1 skeleton — T5 wires the live sidecar resize + JPEG + tile +
    // thumbnail. Until then, degrade to passthrough (never a silent
    // downscale fallthrough — plan stub_ban).
    Ok(PreprocessResult {
        outcome: PreprocessOutcome::DegradedPassthrough,
        conditioned_bytes: image_bytes.to_vec(),
        longest_side_px: 0,
    })
}

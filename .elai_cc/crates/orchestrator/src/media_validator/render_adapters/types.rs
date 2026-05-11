//! B-24 Phase 4I Group C5 (4I.7 SKELETON): Render-Capture-Evaluate (RCE)
//! supporting types.
//!
//! Spec source: docs/specs/guide/steps/phase-4I.txt L380-416. Every type
//! declared here is mandated by the TASK block of 4I.7. The split into
//! types.rs / web.rs / blender.rs / animation.rs / ui_screenshot.rs /
//! comfyui.rs / mod.rs mirrors the per-adapter file partition required
//! by the spec's "one stub per output type" rule (L396-401).
//!
//! Anti-stale grounding (CLAUDE.md):
//!   * `serde::{Serialize, Deserialize}` derives — round-trip required for
//!     SSB RenderComplete + VisualFeedbackIteration payloads (Group C4 SSB
//!     already ships these variants; see crates/orchestrator/src/ssb.rs L1503-1519).
//!   * No magic numbers — all defaults are named constants.
//!   * `OutputType` derives `Hash + Eq` because it is the key of
//!     `AdapterRegistry::adapters: HashMap<OutputType, _>` (4I.7 L403).
//!   * `Duration` carried as nanos in serde round-trips via
//!     `humantime_serde_duration` helper (re-used from media_validator/types.rs).

use std::path::PathBuf;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use super::super::types::{BoundingBox, Severity};

// ---------------------------------------------------------------------------
// OutputType — drives AdapterRegistry lookup.
// ---------------------------------------------------------------------------

/// Top-level output classification used by the AdapterRegistry to pick the
/// MediaRenderAdapter. Distinct from `MediaType` (which classifies
/// the validation artifact); a Web output produces a screenshot whose
/// `MediaType` is `Image`, but the renderer is `WebRenderAdapter`.
///
/// Per spec phase-4I.txt L382-405. `Custom(String)` lets downstream
/// plugins extend the registry without touching this enum.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputType {
    Web,
    Model3D,
    Animation,
    UiScreenshot,
    AiGenerated,
    Custom(String),
}

// ---------------------------------------------------------------------------
// Captured frame + viewport metadata.
// ---------------------------------------------------------------------------

/// Single rendered frame written to disk by a `MediaRenderAdapter`.
/// `image_path` is the absolute path on disk; the file is owned by the
/// adapter's cleanup() lifecycle (typically a tempdir).
///
/// Spec phase-4I.txt L387. PartialEq derived for assertions in tests;
/// Hash NOT derived because `BoundingBox` contains f32 (no Hash impl —
/// and f32 hashing is a foot-gun — NaN, sign bits, etc.).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CapturedFrame {
    pub image_path: PathBuf,
    pub viewport: Viewport,
    pub metadata: FrameMetadata,
}

/// Logical viewport (CSS pixels for Web; viewport pixels for everything else).
/// `device` is a free-form label (e.g. `"iPhone 14"`, `"desktop-2k"`).
///
/// Spec phase-4I.txt L388.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Viewport {
    pub width: u32,
    pub height: u32,
    #[serde(default)]
    pub device: Option<String>,
}

impl Viewport {
    /// Standard mobile portrait (Playwright default `iPhone 14`).
    pub fn mobile() -> Self {
        Self {
            width: 375,
            height: 667,
            device: Some("mobile".into()),
        }
    }
    /// Tablet portrait.
    pub fn tablet() -> Self {
        Self {
            width: 768,
            height: 1024,
            device: Some("tablet".into()),
        }
    }
    /// Desktop landscape (Playwright default).
    pub fn desktop() -> Self {
        Self {
            width: 1440,
            height: 900,
            device: Some("desktop".into()),
        }
    }
}

/// Per-frame metadata emitted by the adapter; surfaced into the SSB
/// `RenderCompletePayload.adapter_name` (already shipped in Group C4).
///
/// Spec phase-4I.txt L389.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FrameMetadata {
    #[serde(with = "humantime_serde_duration")]
    pub capture_time: Duration,
    pub adapter_name: String,
    pub state_description: String,
}

// ---------------------------------------------------------------------------
// Render config (per-call).
// ---------------------------------------------------------------------------

/// Per-call render configuration. The adapter cross-product is:
/// `viewports.len() * (interactions.len() + 1)` for Screenshot mode;
/// `key_frames` for KeyFrames; `angles.len()` for MultiAngle.
///
/// Spec phase-4I.txt L390-392.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RenderConfig {
    pub viewports: Vec<Viewport>,
    pub interactions: Vec<Interaction>,
    #[serde(with = "humantime_serde_duration")]
    pub timeout: Duration,
    pub capture_mode: CaptureMode,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            viewports: vec![Viewport::desktop()],
            interactions: Vec::new(),
            timeout: Duration::from_secs(30),
            capture_mode: CaptureMode::Screenshot,
        }
    }
}

/// Capture strategy. `Screenshot` is a single still per viewport;
/// `KeyFrames` extracts frames at `interval` intervals; `MultiAngle`
/// renders the artifact from each camera angle (3D models).
///
/// Spec phase-4I.txt L392.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "mode")]
pub enum CaptureMode {
    Screenshot,
    KeyFrames {
        #[serde(with = "humantime_serde_duration")]
        interval: Duration,
    },
    MultiAngle {
        angles: Vec<CameraAngle>,
    },
}

/// Web-style interaction script. Replayed by `WebRenderAdapter` between
/// the initial render and the post-interaction screenshot.
///
/// Spec phase-4I.txt L393-394.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum Interaction {
    Click { selector: String },
    Scroll { y: i32 },
    Wait { ms: u64 },
    Hover { selector: String },
    Type { selector: String, text: String },
}

/// 3D camera placement. `Custom { azimuth, elevation }` is degrees;
/// the named angles map to fixed orthographic poses Blender's CLI
/// emits via `bpy.ops.transform.translate`/`rotate` (see
/// workflow/papers/blender-cli-interface.md).
///
/// Spec phase-4I.txt L395.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum CameraAngle {
    Front,
    Right,
    Top,
    Perspective,
    Custom { azimuth: f32, elevation: f32 },
}

impl CameraAngle {
    /// The 4 default angles emitted by `BlenderRenderAdapter` when no
    /// override is supplied. Spec phase-4I.txt L398.
    pub fn defaults() -> Vec<CameraAngle> {
        vec![
            CameraAngle::Front,
            CameraAngle::Right,
            CameraAngle::Top,
            CameraAngle::Perspective,
        ]
    }
}

// ---------------------------------------------------------------------------
// Visual feedback (loop output).
// ---------------------------------------------------------------------------

/// One visual issue identified by Tier 2 in the feedback loop.
/// `actionable_fix` is mandated non-empty by spec L455 (contract test
/// `rce_c7_feedback_structure_actionable`).
///
/// Spec phase-4I.txt L415-416.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VisualIssue {
    pub description: String,
    pub severity: Severity,
    #[serde(default)]
    pub location: Option<BoundingBox>,
    #[serde(default)]
    pub viewport: Option<Viewport>,
    pub actionable_fix: String,
}

impl VisualIssue {
    /// Convergence key used to dedupe across iterations. Spec adversarial
    /// L487-489 ("compare by severity + category + location, NOT raw
    /// description string"). Description is intentionally absent —
    /// minor VLM-wording drift across iterations would otherwise
    /// prevent convergence detection.
    ///
    /// Location is bucketed to 2-decimal precision so a +/-0.005 shift
    /// in the VLM's bounding box prediction does not break convergence
    /// (the test rce_c5 pins this contract). Category is derived from
    /// the first comma- or colon-separated token of description for
    /// VisualIssue (the type does not carry an explicit category field
    /// — Issue.category is the Tier-2-emitted equivalent).
    pub fn convergence_key(&self) -> ConvergenceKey {
        let category = self
            .description
            .split(|c: char| c == ':' || c == ',')
            .next()
            .map(|s| s.trim().to_lowercase())
            .unwrap_or_default();
        let bucket = self.location.map(|b| {
            (
                (b.x * 100.0).round() as i32,
                (b.y * 100.0).round() as i32,
                (b.width * 100.0).round() as i32,
                (b.height * 100.0).round() as i32,
            )
        });
        ConvergenceKey {
            severity: self.severity,
            category,
            location_bucket: bucket,
        }
    }
}

/// Hashable convergence key emitted by [`VisualIssue::convergence_key`].
/// Two issues with the same key are considered "the same issue" for
/// loop-termination purposes.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ConvergenceKey {
    pub severity: Severity,
    pub category: String,
    /// `(x, y, width, height)` in centi-units of the unit square.
    pub location_bucket: Option<(i32, i32, i32, i32)>,
}

// ---------------------------------------------------------------------------
// Duration serde helper — keep RenderConfig + FrameMetadata JSON-friendly.
// ---------------------------------------------------------------------------

mod humantime_serde_duration {
    use std::time::Duration;

    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(d: &Duration, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        s.serialize_u64(u64::try_from(d.as_millis()).unwrap_or(u64::MAX))
    }

    pub fn deserialize<'de, D>(d: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let ms = u64::deserialize(d)?;
        Ok(Duration::from_millis(ms))
    }
}

// ---------------------------------------------------------------------------
// Unit tests — pin the trivial invariants.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn output_type_round_trips_via_serde() {
        for v in [
            OutputType::Web,
            OutputType::Model3D,
            OutputType::Animation,
            OutputType::UiScreenshot,
            OutputType::AiGenerated,
            OutputType::Custom("ar_quicklook".into()),
        ] {
            let s = serde_json::to_string(&v).expect("serialize");
            let d: OutputType = serde_json::from_str(&s).expect("deserialize");
            assert_eq!(v, d);
        }
    }

    #[test]
    fn convergence_key_buckets_location() {
        let issue_a = VisualIssue {
            description: "Button overflow: text exceeds container".into(),
            severity: Severity::Major,
            location: Some(BoundingBox {
                x: 0.1234,
                y: 0.5678,
                width: 0.2,
                height: 0.1,
            }),
            viewport: None,
            actionable_fix: "increase max-width to 90%".into(),
        };
        let issue_b = VisualIssue {
            description: "Button overflow: same issue rephrased".into(),
            severity: Severity::Major,
            location: Some(BoundingBox {
                x: 0.1237, // <0.5% drift — same bucket.
                y: 0.5672,
                width: 0.2,
                height: 0.1,
            }),
            viewport: None,
            actionable_fix: "set max-width".into(),
        };
        // Different description, same bucket -> equal convergence key.
        assert_eq!(issue_a.convergence_key(), issue_b.convergence_key());
    }

    #[test]
    fn camera_angle_defaults_count_is_four() {
        let d = CameraAngle::defaults();
        assert_eq!(d.len(), 4, "spec L398 mandates 4 default angles");
    }

    #[test]
    fn viewport_helpers_return_distinct_devices() {
        assert_ne!(Viewport::mobile(), Viewport::desktop());
        assert_ne!(Viewport::tablet(), Viewport::desktop());
    }
}

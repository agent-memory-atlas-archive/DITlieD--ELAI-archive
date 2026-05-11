//! B-24 Phase 4I Group C5: `UiScreenshotAdapter` — pass-through adapter
//! for already-rendered screenshots.
//!
//! Spec phase-4I.txt L400 (`UiScreenshotAdapter {}` — no external tool
//! required) and L466-467 ("pass-through of existing image; no
//! rendering needed"). This is the ONE adapter that ships with a REAL
//! impl in Group C5 — the other 4 (Web/Blender/Animation/ComfyUI) are
//! NotImplemented skeletons until Group C6.
//!
//! Anti-stale grounding:
//!   * std::fs::metadata for size check (no tokio runtime required —
//!     adapter is called from sync + async contexts).
//!   * `std::time::Instant` for the capture_time stamp.
//!   * No magic numbers — `MAX_SCREENSHOT_BYTES` named const.

use std::path::Path;
use std::pin::Pin;
use std::time::{Duration, Instant};

use super::types::{CapturedFrame, FrameMetadata, OutputType, RenderConfig, Viewport};
use super::{MediaRenderAdapter, RenderAdapterError};

/// 25 MiB — comfortably above any practical UI screenshot (4K PNG ~
/// 8 MiB) but well below the validator's 50 MiB file cap.
const MAX_SCREENSHOT_BYTES: u64 = 25 * 1024 * 1024;

/// Pass-through adapter: validates that the artifact path exists and
/// is a readable PNG/JPEG, then emits a single `CapturedFrame` whose
/// `image_path` IS the artifact path (no copy).
///
/// Required-tool: NONE — file I/O only.
#[derive(Debug, Default)]
pub struct UiScreenshotAdapter {
    _private: (),
}

impl UiScreenshotAdapter {
    pub fn new() -> Self {
        Self { _private: () }
    }
}

impl MediaRenderAdapter for UiScreenshotAdapter {
    fn can_handle(&self, output_type: &OutputType) -> bool {
        matches!(output_type, OutputType::UiScreenshot)
    }

    fn render<'a>(
        &'a self,
        artifact: &'a Path,
        config: &'a RenderConfig,
    ) -> Pin<
        Box<
            dyn std::future::Future<Output = Result<Vec<CapturedFrame>, RenderAdapterError>>
                + Send
                + 'a,
        >,
    > {
        Box::pin(async move {
            let started = Instant::now();
            // 1. Path validity.
            let metadata = std::fs::metadata(artifact).map_err(|_| {
                RenderAdapterError::ArtifactPathInvalid {
                    path: artifact.to_path_buf(),
                }
            })?;
            // 2. Size guard (25 MiB).
            if metadata.len() > MAX_SCREENSHOT_BYTES {
                return Err(RenderAdapterError::MalformedOutput(format!(
                    "screenshot {} bytes exceeds cap {}",
                    metadata.len(),
                    MAX_SCREENSHOT_BYTES
                )));
            }
            // 3. Image-format sniff. PNG signature: 0x89 0x50 0x4E 0x47.
            //    JPEG: 0xFF 0xD8 0xFF. Anything else => malformed.
            let mut head = [0u8; 4];
            {
                use std::io::Read;
                let mut f = std::fs::File::open(artifact)?;
                let n = f.read(&mut head).unwrap_or(0);
                if n < 3 {
                    return Err(RenderAdapterError::MalformedOutput(format!(
                        "artifact too short ({n} bytes) for image header check"
                    )));
                }
            }
            let is_png = head == [0x89, 0x50, 0x4E, 0x47];
            let is_jpeg = head[0..3] == [0xFF, 0xD8, 0xFF];
            if !is_png && !is_jpeg {
                return Err(RenderAdapterError::MalformedOutput(format!(
                    "unrecognized image header: {:02X?}",
                    &head[..]
                )));
            }
            // 4. Viewport: first from config, default to desktop.
            let viewport = config
                .viewports
                .first()
                .cloned()
                .unwrap_or_else(Viewport::desktop);
            // 5. Emit one CapturedFrame.
            let frame = CapturedFrame {
                image_path: artifact.to_path_buf(),
                viewport,
                metadata: FrameMetadata {
                    capture_time: started.elapsed(),
                    adapter_name: self.name().to_string(),
                    state_description: if is_png {
                        "ui_screenshot_png_pass_through".into()
                    } else {
                        "ui_screenshot_jpeg_pass_through".into()
                    },
                },
            };
            Ok(vec![frame])
        })
    }

    fn name(&self) -> &'static str {
        "ui_screenshot"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // Pre-computed valid 1x1 RGB PNG (red pixel). Same bytes as the
    // bench fixture in media_validator_bench.rs.
    const TINY_PNG: &[u8] = &[
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x00, 0x00, 0x00, 0x90,
        0x77, 0x53, 0xDE, 0x00, 0x00, 0x00, 0x0C, 0x49, 0x44, 0x41, 0x54, 0x08, 0xD7, 0x63, 0xF8,
        0xCF, 0xC0, 0x00, 0x00, 0x00, 0x03, 0x00, 0x01, 0x5B, 0x42, 0x5D, 0x6A, 0x00, 0x00, 0x00,
        0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
    ];

    #[tokio::test]
    async fn renders_png_pass_through() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("ui.png");
        std::fs::write(&path, TINY_PNG).unwrap();
        let adapter = UiScreenshotAdapter::new();
        let cfg = RenderConfig::default();
        let frames = adapter.render(&path, &cfg).await.unwrap();
        assert_eq!(frames.len(), 1);
        assert_eq!(frames[0].image_path, path);
        assert_eq!(frames[0].metadata.adapter_name, "ui_screenshot");
        assert!(frames[0]
            .metadata
            .state_description
            .contains("png_pass_through"));
    }

    #[tokio::test]
    async fn rejects_missing_file() {
        let adapter = UiScreenshotAdapter::new();
        let cfg = RenderConfig::default();
        let err = adapter
            .render(Path::new("nonexistent.png"), &cfg)
            .await
            .unwrap_err();
        assert!(matches!(
            err,
            RenderAdapterError::ArtifactPathInvalid { .. }
        ));
    }

    #[tokio::test]
    async fn rejects_non_image_blob() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("garbage.bin");
        std::fs::write(&path, &[0x00, 0x01, 0x02, 0x03, 0x04, 0x05]).unwrap();
        let adapter = UiScreenshotAdapter::new();
        let cfg = RenderConfig::default();
        let err = adapter.render(&path, &cfg).await.unwrap_err();
        assert!(matches!(err, RenderAdapterError::MalformedOutput(_)));
    }

    #[test]
    fn can_handle_only_ui_screenshot() {
        let a = UiScreenshotAdapter::new();
        assert!(a.can_handle(&OutputType::UiScreenshot));
        assert!(!a.can_handle(&OutputType::Web));
        assert!(!a.can_handle(&OutputType::Model3D));
    }
}

/// Suppress unused-`Duration` lint in some build configurations.
#[allow(dead_code)]
fn _duration_marker(_: Duration) {}

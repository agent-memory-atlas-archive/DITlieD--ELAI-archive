//! B-24 Phase 4I Group C6 (4I.9 IMPLEMENT): `AnimationRenderAdapter` — real impl.
//!
//! Spec phase-4I.txt L399/464-465. Uses `ffmpeg -i <input> -vf
//! "select=not(mod(n,N))" -vsync vfr <out>/frame_%04d.png` to extract
//! frames at a fixed modulo. The `-vsync vfr` flag is MANDATORY when
//! `select` filter is used to avoid duplicated frames at fixed-fps
//! output (paper grounded; FFmpeg docs verified 2026-05).
//!
//! Anti-stale:
//!   * tokio::process::Command (no std::process in async).
//!   * tokio::time::timeout for kill on hang.
//!   * `-vsync vfr` paired with `select` filter (FFmpeg `-vsync` is the
//!     legacy alias still accepted in 7.x).
//!   * u64::try_from for ms narrowing.

use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::time::{Duration, Instant};

use tokio::process::Command;
use tracing::{debug, warn};

use super::types::{
    CaptureMode, CapturedFrame, FrameMetadata, OutputType, RenderConfig, Viewport,
};
use super::{MediaRenderAdapter, RenderAdapterError};

/// Default key-frame interval (1 second between keyframes). Real impl
/// derives the modulo from `RenderConfig.capture_mode = KeyFrames { interval }`.
pub const DEFAULT_KEY_FRAME_INTERVAL: Duration = Duration::from_secs(1);

/// Default modulo when CaptureMode is Screenshot (not KeyFrames). 30
/// = roughly 1 keyframe / second at 30fps. Documented constant rather
/// than magic number.
const DEFAULT_FRAME_MODULO: u32 = 30;

/// Default timeout when `RenderConfig.timeout == 0`.
const DEFAULT_ANIMATION_TIMEOUT_MS: u64 = 60_000;

/// FFmpeg command-line template documented for the spec audit. Variables
/// substituted via `cmd.arg(...)` rather than string-replace so paths
/// containing spaces work correctly.
pub const FFMPEG_KEYFRAME_CMD_TEMPLATE: &str =
    "ffmpeg -hide_banner -loglevel error -i {input} -vf select='not(mod(n\\,{modulo}))' -vsync vfr {out_dir}/frame_%04d.png";

#[derive(Debug)]
pub struct AnimationRenderAdapter {
    /// Absolute path to the `ffmpeg` binary. Empty -> resolved from
    /// `PATH` at render time.
    pub ffmpeg_path: PathBuf,
    pub key_frame_interval: Duration,
}

impl Default for AnimationRenderAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl AnimationRenderAdapter {
    pub fn new() -> Self {
        Self {
            ffmpeg_path: PathBuf::new(),
            key_frame_interval: DEFAULT_KEY_FRAME_INTERVAL,
        }
    }

    pub fn with_ffmpeg_path(ffmpeg_path: PathBuf) -> Self {
        Self {
            ffmpeg_path,
            key_frame_interval: DEFAULT_KEY_FRAME_INTERVAL,
        }
    }
}

fn ffmpeg_binary(adapter: &AnimationRenderAdapter) -> PathBuf {
    if adapter.ffmpeg_path.as_os_str().is_empty() {
        PathBuf::from("ffmpeg")
    } else {
        adapter.ffmpeg_path.clone()
    }
}

fn validate_artifact_extension(artifact: &Path) -> Result<(), RenderAdapterError> {
    let ext = artifact
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_lowercase());
    match ext.as_deref() {
        Some("mp4") | Some("mov") | Some("webm") | Some("mkv") | Some("avi") => Ok(()),
        _ => Err(RenderAdapterError::ArtifactPathInvalid {
            path: artifact.to_path_buf(),
        }),
    }
}

fn make_tempdir(prefix: &str) -> Result<PathBuf, RenderAdapterError> {
    let base = std::env::temp_dir();
    let id = uuid::Uuid::new_v4(); // entropy-allow: C-15 deferred — render-adapter Clock plumbing scheduled
    let dir = base.join(format!("{prefix}-{id}"));
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

fn remove_dir_safe(dir: &Path) -> std::io::Result<()> {
    if dir.exists() {
        std::fs::remove_dir_all(dir)
    } else {
        Ok(())
    }
}

/// Compute the frame modulo from the configured capture mode. KeyFrames
/// interval is interpreted assuming 30fps source video (paper grounded;
/// FFmpeg doesn't expose source fps via the simple select-modulo path).
fn modulo_from_mode(mode: &CaptureMode) -> u32 {
    match mode {
        CaptureMode::KeyFrames { interval } => {
            let secs = interval.as_secs_f64().max(0.0334); // 30fps floor
            let m = (secs * 30.0).round() as i64;
            if m < 1 {
                1
            } else if m > i64::from(u32::MAX) {
                u32::MAX
            } else {
                m as u32
            }
        }
        _ => DEFAULT_FRAME_MODULO,
    }
}

impl MediaRenderAdapter for AnimationRenderAdapter {
    fn can_handle(&self, output_type: &OutputType) -> bool {
        matches!(output_type, OutputType::Animation)
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
            let started = Instant::now(); // entropy-allow: C-15 deferred — render-adapter Clock plumbing scheduled
            validate_artifact_extension(artifact)?;
            let work_dir = make_tempdir("elai-anim-render")?;
            let modulo = modulo_from_mode(&config.capture_mode);
            // FFmpeg `select` filter: not(mod(n,M)) keeps frames whose
            // index n satisfies n % M == 0. `-vsync vfr` is MANDATORY to
            // avoid duplicated frames at fixed-fps output.
            let filter = format!("select=not(mod(n\\,{modulo}))");
            let out_pattern = work_dir.join("frame_%04d.png");

            let mut cmd = Command::new(ffmpeg_binary(self));
            cmd.arg("-hide_banner")
                .arg("-loglevel")
                .arg("error")
                .arg("-i")
                .arg(artifact)
                .arg("-vf")
                .arg(&filter)
                .arg("-vsync")
                .arg("vfr")
                .arg(&out_pattern)
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .kill_on_drop(true);

            let timeout_ms = if config.timeout.as_millis() == 0 {
                DEFAULT_ANIMATION_TIMEOUT_MS
            } else {
                u64::try_from(config.timeout.as_millis())
                    .unwrap_or(DEFAULT_ANIMATION_TIMEOUT_MS)
            };
            let timeout = Duration::from_millis(timeout_ms);
            debug!(
                adapter = "animation",
                artifact = %artifact.display(),
                modulo,
                timeout_ms,
                "spawning ffmpeg"
            );
            let output = match tokio::time::timeout(timeout, cmd.output()).await {
                Ok(Ok(out)) => out,
                Ok(Err(e)) => {
                    warn!(error = %e, "ffmpeg spawn failed");
                    let _ = remove_dir_safe(&work_dir);
                    return Err(RenderAdapterError::ToolUnavailable { tool: "ffmpeg" });
                }
                Err(_elapsed) => {
                    warn!(
                        adapter = "animation",
                        timeout_ms,
                        "AnimationRenderAdapter timed out; child killed"
                    );
                    let _ = remove_dir_safe(&work_dir);
                    return Err(RenderAdapterError::Timeout { millis: timeout_ms });
                }
            };
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
                let _ = remove_dir_safe(&work_dir);
                return Err(RenderAdapterError::MalformedOutput(format!(
                    "ffmpeg exit={:?}; stderr=`{stderr}`",
                    output.status.code()
                )));
            }

            // List produced frame_*.png files; sort lexicographically (the
            // %04d pattern is already monotonic).
            let mut entries: Vec<PathBuf> = std::fs::read_dir(&work_dir)?
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| {
                    p.extension()
                        .and_then(|s| s.to_str())
                        .map(|s| s.eq_ignore_ascii_case("png"))
                        .unwrap_or(false)
                })
                .filter(|p| {
                    p.file_name()
                        .and_then(|s| s.to_str())
                        .map(|s| s.starts_with("frame_"))
                        .unwrap_or(false)
                })
                .collect();
            entries.sort();

            if entries.is_empty() {
                let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
                let _ = remove_dir_safe(&work_dir);
                return Err(RenderAdapterError::MalformedOutput(format!(
                    "ffmpeg succeeded but produced 0 frames; stderr=`{stderr}`"
                )));
            }

            let viewport = config
                .viewports
                .first()
                .cloned()
                .unwrap_or_else(|| Viewport {
                    width: 1920,
                    height: 1080,
                    device: Some("video".into()),
                });

            let frames = entries
                .into_iter()
                .enumerate()
                .map(|(idx, p)| CapturedFrame {
                    image_path: p,
                    viewport: viewport.clone(),
                    metadata: FrameMetadata {
                        capture_time: started.elapsed(),
                        adapter_name: "animation".to_string(),
                        state_description: format!("animation_keyframe_{idx}"),
                    },
                })
                .collect::<Vec<_>>();
            Ok(frames)
        })
    }

    fn name(&self) -> &'static str {
        "animation"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_handle_only_animation() {
        let a = AnimationRenderAdapter::new();
        assert!(a.can_handle(&OutputType::Animation));
        assert!(!a.can_handle(&OutputType::Web));
    }

    #[test]
    fn ffmpeg_template_uses_vfr() {
        assert!(FFMPEG_KEYFRAME_CMD_TEMPLATE.contains("-vsync vfr"));
        assert!(FFMPEG_KEYFRAME_CMD_TEMPLATE.contains("select="));
    }

    #[test]
    fn default_key_frame_interval_is_one_second() {
        let a = AnimationRenderAdapter::new();
        assert_eq!(a.key_frame_interval, DEFAULT_KEY_FRAME_INTERVAL);
        assert_eq!(a.key_frame_interval.as_secs(), 1);
    }

    #[test]
    fn modulo_from_keyframes_assumes_30fps() {
        let m = modulo_from_mode(&CaptureMode::KeyFrames {
            interval: Duration::from_secs(2),
        });
        // 2s @ 30fps = 60
        assert_eq!(m, 60);
    }

    #[test]
    fn modulo_from_screenshot_is_default() {
        let m = modulo_from_mode(&CaptureMode::Screenshot);
        assert_eq!(m, DEFAULT_FRAME_MODULO);
    }

    #[test]
    fn modulo_from_tiny_interval_floors_at_1() {
        let m = modulo_from_mode(&CaptureMode::KeyFrames {
            interval: Duration::from_millis(1),
        });
        assert!(m >= 1, "modulo must never be 0 (FFmpeg div-by-zero)");
    }

    #[test]
    fn validate_artifact_extension_accepts_mp4() {
        assert!(validate_artifact_extension(Path::new("/tmp/v.mp4")).is_ok());
        assert!(validate_artifact_extension(Path::new("/tmp/v.mov")).is_ok());
        assert!(validate_artifact_extension(Path::new("/tmp/v.webm")).is_ok());
        assert!(validate_artifact_extension(Path::new("/tmp/v.txt")).is_err());
    }

    #[tokio::test]
    async fn render_returns_tool_unavailable_when_ffmpeg_missing() {
        let a = AnimationRenderAdapter::with_ffmpeg_path(PathBuf::from(
            "/nonexistent-ffmpeg-binary-elai",
        ));
        let cfg = RenderConfig::default();
        let dir = std::env::temp_dir();
        let mp4 = dir.join(format!("elai-mp4-stub-{}.mp4", uuid::Uuid::new_v4())); // entropy-allow: C-15 deferred — render-adapter Clock plumbing scheduled
        let _ = std::fs::write(&mp4, b"stub");
        let r = a.render(&mp4, &cfg).await;
        let _ = std::fs::remove_file(&mp4);
        assert!(
            r.is_err(),
            "missing ffmpeg binary must yield error, got {r:?}"
        );
    }
}

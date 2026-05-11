//! B-24 Phase 4I Group C6 (4I.9 IMPLEMENT): `BlenderRenderAdapter` — real impl.
//!
//! Spec phase-4I.txt L398/462-463. Uses the `blender --background --python <script>`
//! pattern (paper workflow/papers/blender-cli-interface.md). The Python
//! render script is embedded as `&'static str` and written to a
//! per-call temp directory before invocation. Four default angles
//! (Front / Right / Top / Perspective) match spec L398.
//!
//! Anti-stale (paper-grounded 2026):
//!   * `bpy.ops.wm.obj_import` is the Blender 4.0+ operator (the legacy
//!     `bpy.ops.import_scene.obj` was REMOVED in 4.0); we try the new
//!     name first then fall back for older Blender installs.
//!   * `blender --background --python <file>` is the documented headless
//!     entry. `--python-exit-code 1` makes Python errors propagate as
//!     non-zero exit. Trailing args after `--` are passed to the script
//!     via `sys.argv` (Blender truncates the leading flags).
//!   * tokio::process::Command (no std::process in async).
//!   * tokio::time::timeout for kill on hang.
//!   * u64::try_from for ms narrowing.

use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::time::{Duration, Instant};

use tokio::process::Command;
use tracing::{debug, warn};

use super::types::{CameraAngle, CapturedFrame, FrameMetadata, OutputType, RenderConfig, Viewport};
use super::{MediaRenderAdapter, RenderAdapterError};

/// Embedded Python render script. Reads artifact path + output dir
/// from environment vars (ELAI_OBJ_PATH / ELAI_OUT_DIR) so we don't
/// have to parse Blender's argv quirks. Renders 4 fixed orthographic
/// angles to PNG and prints `FRAME:<name>:<absolute-path>` per success.
pub const BLENDER_RENDER_SCRIPT: &str = r#"
import bpy
import os
import sys
import math

obj_path = os.environ.get("ELAI_OBJ_PATH")
out_dir = os.environ.get("ELAI_OUT_DIR", os.path.dirname(__file__) or ".")
if not obj_path:
    sys.stderr.write("ELAI_OBJ_PATH unset\n")
    sys.exit(1)

# Clear default scene.
bpy.ops.wm.read_factory_settings(use_empty=True)

# Import the artifact. Blender 4.0+ uses bpy.ops.wm.obj_import; pre-4.0
# fallback would be bpy.ops.import_scene.obj. .glb/.gltf use gltf_import.
ext = os.path.splitext(obj_path)[1].lower()
try:
    if ext in (".obj",):
        try:
            bpy.ops.wm.obj_import(filepath=obj_path)
        except AttributeError:
            bpy.ops.import_scene.obj(filepath=obj_path)
    elif ext in (".gltf", ".glb"):
        bpy.ops.import_scene.gltf(filepath=obj_path)
    elif ext in (".fbx",):
        bpy.ops.import_scene.fbx(filepath=obj_path)
    else:
        # Generic: try OBJ import operator first.
        try:
            bpy.ops.wm.obj_import(filepath=obj_path)
        except AttributeError:
            bpy.ops.import_scene.obj(filepath=obj_path)
except Exception as e:
    sys.stderr.write(f"import failed: {e}\n")
    sys.exit(2)

# Simple sun light at world origin.
bpy.ops.object.light_add(type='SUN', location=(0, 0, 5))

# 4 fixed angles (azimuth, elevation degrees). Spec L398.
angles = [
    ("front",       (0.0, 0.0)),
    ("right",       (90.0, 0.0)),
    ("top",         (0.0, 90.0)),
    ("perspective", (45.0, 35.0)),
]

# Workbench engine: fastest, no GPU required, deterministic output.
bpy.context.scene.render.engine = 'BLENDER_WORKBENCH'
bpy.context.scene.render.resolution_x = 512
bpy.context.scene.render.resolution_y = 512
bpy.context.scene.render.image_settings.file_format = 'PNG'

for name, (az, el) in angles:
    cam_data = bpy.data.cameras.new(name=f"cam_{name}")
    cam_obj = bpy.data.objects.new(name=f"cam_{name}", object_data=cam_data)
    bpy.context.scene.collection.objects.link(cam_obj)
    r = 5.0
    cam_obj.location = (
        r * math.cos(math.radians(el)) * math.sin(math.radians(az)),
        -r * math.cos(math.radians(el)) * math.cos(math.radians(az)),
        r * math.sin(math.radians(el)),
    )
    # Point camera at origin via Euler hint.
    cam_obj.rotation_euler = (math.radians(90 - el), 0, math.radians(az))
    bpy.context.scene.camera = cam_obj
    out_path = os.path.join(out_dir, f"render_{name}.png")
    bpy.context.scene.render.filepath = out_path
    bpy.ops.render.render(write_still=True)
    sys.stdout.write(f"FRAME:{name}:{out_path}\n")
    sys.stdout.flush()
"#;

const BLENDER_SCRIPT_FILENAME: &str = "elai_blender_render.py";
const DEFAULT_BLENDER_TIMEOUT_MS: u64 = 120_000;

/// Blender 3D renderer — REAL impl. Spawns `blender --background --python`
/// against the embedded script for 4 fixed camera angles per spec L398.
///
/// Audit WARN 1 fix (decision 74): the adapter tracks per-call tempdirs
/// in a `Mutex<Vec<PathBuf>>` and removes them on Drop. The captured
/// PNGs live inside those directories — consumers must read them
/// BEFORE dropping the adapter.
pub struct BlenderRenderAdapter {
    /// Absolute path to the `blender` binary. Empty -> resolved from
    /// `PATH` at render time.
    pub blender_path: PathBuf,
    /// Default camera angles (Front / Right / Top / Perspective).
    pub default_angles: Vec<CameraAngle>,
    /// TEST-ONLY override for the embedded Python script. Constructed
    /// exclusively via `__test_only_with_script_override` (audit BLOCK 4 —
    /// decision 73) so adversarial tests can inject malformed Python and
    /// exercise the error / timeout path. The constructor is
    /// `#[doc(hidden)]` + name-mangled so production callers cannot
    /// stumble into it; the field itself stays private to this module
    /// (no `pub` visibility from any production import path).
    python_script_override: Option<String>,
    /// Per-instance tempdir tracker (audit WARN 1 — decision 74).
    tempdirs: std::sync::Mutex<Vec<PathBuf>>,
}

impl std::fmt::Debug for BlenderRenderAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BlenderRenderAdapter")
            .field("blender_path", &self.blender_path)
            .field("default_angles_count", &self.default_angles.len())
            .field(
                "python_script_override.is_some",
                &self.python_script_override.is_some(),
            )
            .field(
                "tempdir_count",
                &self.tempdirs.lock().map(|g| g.len()).unwrap_or(0),
            )
            .finish()
    }
}

impl Drop for BlenderRenderAdapter {
    fn drop(&mut self) {
        if let Ok(mut guard) = self.tempdirs.lock() {
            for dir in guard.drain(..) {
                let _ = std::fs::remove_dir_all(&dir);
            }
        }
    }
}

impl Default for BlenderRenderAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl BlenderRenderAdapter {
    pub fn new() -> Self {
        Self {
            blender_path: PathBuf::new(),
            default_angles: CameraAngle::defaults(),
            python_script_override: None,
            tempdirs: std::sync::Mutex::new(Vec::new()),
        }
    }

    pub fn with_blender_path(blender_path: PathBuf) -> Self {
        Self {
            blender_path,
            default_angles: CameraAngle::defaults(),
            python_script_override: None,
            tempdirs: std::sync::Mutex::new(Vec::new()),
        }
    }

    /// TEST-ONLY entry point: build a `BlenderRenderAdapter` whose
    /// embedded Python script is replaced by `script`. Adversarial test
    /// `rce_a_blender_python_error_returns_within_2s` (audit BLOCK 4) injects
    /// `raise RuntimeError('test')` to exercise the Python-failure path
    /// without modifying the production const &str.
    ///
    /// `#[doc(hidden)]` + the explicit `__test_only_` prefix flag this as
    /// non-production API; no production code path should ever call it.
    #[doc(hidden)]
    pub fn __test_only_with_script_override(
        blender_path: PathBuf,
        script: String,
    ) -> Self {
        Self {
            blender_path,
            default_angles: CameraAngle::defaults(),
            python_script_override: Some(script),
            tempdirs: std::sync::Mutex::new(Vec::new()),
        }
    }
}

fn blender_binary(adapter: &BlenderRenderAdapter) -> PathBuf {
    if adapter.blender_path.as_os_str().is_empty() {
        PathBuf::from("blender")
    } else {
        adapter.blender_path.clone()
    }
}

fn validate_artifact_extension(artifact: &Path) -> Result<(), RenderAdapterError> {
    let ext = artifact
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_lowercase());
    match ext.as_deref() {
        Some("obj") | Some("gltf") | Some("glb") | Some("fbx") | Some("ply") | Some("stl") => {
            Ok(())
        }
        _ => Err(RenderAdapterError::ArtifactPathInvalid {
            path: artifact.to_path_buf(),
        }),
    }
}

fn make_tempdir(prefix: &str) -> Result<PathBuf, RenderAdapterError> {
    let base = std::env::temp_dir();
    let id = uuid::Uuid::new_v4();
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

impl MediaRenderAdapter for BlenderRenderAdapter {
    fn can_handle(&self, output_type: &OutputType) -> bool {
        matches!(output_type, OutputType::Model3D)
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
            validate_artifact_extension(artifact)?;

            // 1. Write the embedded Python script (or test-only override)
            // to a per-call tempdir. Production callers always get the
            // production const; `__test_only_with_script_override`
            // injects malformed Python in the BLOCK 4 adversarial path.
            let work_dir = make_tempdir("elai-blender-render")?;
            let script_path = work_dir.join(BLENDER_SCRIPT_FILENAME);
            let script_bytes: &[u8] = match &self.python_script_override {
                Some(s) => s.as_bytes(),
                None => BLENDER_RENDER_SCRIPT.as_bytes(),
            };
            std::fs::write(&script_path, script_bytes)?;

            // 2. Spawn blender headless. ELAI_OBJ_PATH/ELAI_OUT_DIR are
            //    read by the script. `--python-exit-code 1` propagates
            //    Python exceptions as non-zero exit (anti-stale check).
            let abs_artifact = std::fs::canonicalize(artifact)
                .unwrap_or_else(|_| artifact.to_path_buf());
            let mut cmd = Command::new(blender_binary(self));
            cmd.arg("--background")
                .arg("--python")
                .arg(&script_path)
                .arg("--python-exit-code")
                .arg("1")
                .env("ELAI_OBJ_PATH", &abs_artifact)
                .env("ELAI_OUT_DIR", &work_dir)
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .kill_on_drop(true);

            let timeout_ms = if config.timeout.as_millis() == 0 {
                DEFAULT_BLENDER_TIMEOUT_MS
            } else {
                u64::try_from(config.timeout.as_millis())
                    .unwrap_or(DEFAULT_BLENDER_TIMEOUT_MS)
            };
            let timeout = Duration::from_millis(timeout_ms);

            debug!(adapter = "blender", artifact = %abs_artifact.display(), timeout_ms, "spawning blender");

            let output = match tokio::time::timeout(timeout, cmd.output()).await {
                Ok(Ok(out)) => out,
                Ok(Err(e)) => {
                    warn!(error = %e, "blender spawn failed");
                    let _ = remove_dir_safe(&work_dir);
                    return Err(RenderAdapterError::ToolUnavailable { tool: "blender" });
                }
                Err(_elapsed) => {
                    warn!(
                        adapter = "blender",
                        timeout_ms,
                        "BlenderRenderAdapter timed out; child killed"
                    );
                    let _ = remove_dir_safe(&work_dir);
                    return Err(RenderAdapterError::Timeout { millis: timeout_ms });
                }
            };

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
                let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
                let _ = remove_dir_safe(&work_dir);
                return Err(RenderAdapterError::MalformedOutput(format!(
                    "blender exit={:?}; stdout=`{stdout}`; stderr=`{stderr}`",
                    output.status.code()
                )));
            }

            // 3. Parse stdout for `FRAME:<name>:<path>` lines.
            let stdout = String::from_utf8_lossy(&output.stdout);
            let mut frames: Vec<CapturedFrame> = Vec::new();
            let vp = config
                .viewports
                .first()
                .cloned()
                .unwrap_or_else(|| Viewport {
                    width: 512,
                    height: 512,
                    device: Some("blender-render".into()),
                });
            for line in stdout.lines() {
                let Some(rest) = line.strip_prefix("FRAME:") else {
                    continue;
                };
                let parts: Vec<&str> = rest.splitn(2, ':').collect();
                if parts.len() != 2 {
                    continue;
                }
                let name = parts[0].to_string();
                let png_path = PathBuf::from(parts[1]);
                frames.push(CapturedFrame {
                    image_path: png_path,
                    viewport: vp.clone(),
                    metadata: FrameMetadata {
                        capture_time: started.elapsed(),
                        adapter_name: self.name().to_string(),
                        state_description: format!("model3d_{name}"),
                    },
                });
            }

            if frames.is_empty() {
                let _ = remove_dir_safe(&work_dir);
                return Err(RenderAdapterError::MalformedOutput(format!(
                    "BlenderRenderAdapter produced 0 frames; stdout=`{stdout}`"
                )));
            }

            // Audit WARN 1 fix (decision 74): track tempdir for Drop /
            // cleanup() so the renders are not leaked.
            if let Ok(mut guard) = self.tempdirs.lock() {
                guard.push(work_dir);
            }
            Ok(frames)
        })
    }

    fn cleanup(&self) {
        // Drain tracked tempdirs and remove them. Idempotent.
        if let Ok(mut guard) = self.tempdirs.lock() {
            for dir in guard.drain(..) {
                let _ = std::fs::remove_dir_all(&dir);
            }
        }
    }

    fn name(&self) -> &'static str {
        "blender"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_handle_only_model3d() {
        let a = BlenderRenderAdapter::new();
        assert!(a.can_handle(&OutputType::Model3D));
        assert!(!a.can_handle(&OutputType::Web));
    }

    #[test]
    fn default_angles_are_four() {
        let a = BlenderRenderAdapter::new();
        assert_eq!(a.default_angles.len(), 4);
    }

    #[test]
    fn embedded_script_contains_modern_obj_import() {
        assert!(BLENDER_RENDER_SCRIPT.contains("bpy.ops.wm.obj_import"));
        assert!(BLENDER_RENDER_SCRIPT.contains("bpy.ops.import_scene.obj"));
        assert!(BLENDER_RENDER_SCRIPT.contains("BLENDER_WORKBENCH"));
    }

    #[test]
    fn validate_artifact_extension_accepts_obj() {
        assert!(validate_artifact_extension(Path::new("/tmp/m.obj")).is_ok());
        assert!(validate_artifact_extension(Path::new("/tmp/m.glb")).is_ok());
        assert!(validate_artifact_extension(Path::new("/tmp/m.gltf")).is_ok());
        assert!(validate_artifact_extension(Path::new("/tmp/m.fbx")).is_ok());
        assert!(validate_artifact_extension(Path::new("/tmp/foo.txt")).is_err());
    }

    #[tokio::test]
    async fn render_returns_tool_unavailable_when_blender_missing() {
        let a = BlenderRenderAdapter::with_blender_path(PathBuf::from(
            "/nonexistent-blender-binary-elai",
        ));
        let cfg = RenderConfig::default();
        let dir = std::env::temp_dir();
        let obj = dir.join(format!("elai-obj-stub-{}.obj", uuid::Uuid::new_v4()));
        let _ = std::fs::write(&obj, b"v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\n");
        let r = a.render(&obj, &cfg).await;
        let _ = std::fs::remove_file(&obj);
        assert!(
            r.is_err(),
            "missing blender binary must yield error, got {r:?}"
        );
    }
}

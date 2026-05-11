//! B-24 Phase 4I Group C6 (4I.9 IMPLEMENT): per-adapter functional tests.
//!
//! Spec phase-4I.txt L457-469. Tests are graceful-degradation:
//!   * Tool present (playwright / blender / ffmpeg) -> exercise the real
//!     subprocess and assert outputs.
//!   * Tool absent -> emit a TOOL_MISSING eprintln and return early
//!     WITHOUT failing. This matches spec L555-560 (the adversarial
//!     "if Playwright fails to launch ..." entries demand graceful
//!     degradation, NOT test-suite failure).
//!   * ComfyUI test ALWAYS runs: in-process mock HTTP server via
//!     `tokio::net::TcpListener`. No external tool needed.
//!
//! Test discipline (CLAUDE.md): `.unwrap()` permitted in test bodies;
//! deterministic; no live network.

use std::path::Path;

use tempfile::TempDir;

use elai_orchestrator::media_validator::render_adapters::{
    AnimationRenderAdapter, BlenderRenderAdapter, ComfyUIRenderAdapter, MediaRenderAdapter,
    RenderAdapterError, RenderConfig, UiScreenshotAdapter, Viewport, WebRenderAdapter,
};

// ---------------------------------------------------------------------------
// Helpers.
// ---------------------------------------------------------------------------

const TINY_PNG: &[u8] = &[
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x00, 0x00, 0x00, 0x90, 0x77, 0x53,
    0xDE, 0x00, 0x00, 0x00, 0x0C, 0x49, 0x44, 0x41, 0x54, 0x08, 0xD7, 0x63, 0xF8, 0xCF, 0xC0, 0x00,
    0x00, 0x00, 0x03, 0x00, 0x01, 0x5B, 0x42, 0x5D, 0x6A, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E,
    0x44, 0xAE, 0x42, 0x60, 0x82,
];

/// Returns true iff the named binary is on PATH. Uses `which.exe` on
/// Windows / `which` on Unix.
fn binary_on_path(name: &str) -> bool {
    let probe = if cfg!(target_os = "windows") {
        "where.exe"
    } else {
        "which"
    };
    let out = match std::process::Command::new(probe).arg(name).output() {
        Ok(o) => o,
        Err(_) => return false,
    };
    if !out.status.success() {
        return false;
    }
    let s = String::from_utf8_lossy(&out.stdout);
    s.lines().next().map(|l| !l.trim().is_empty()).unwrap_or(false)
}

// ---------------------------------------------------------------------------
// rce_f_web_render_three_viewports — graceful skip if node/playwright absent.
// ---------------------------------------------------------------------------

/// Spec L458-461. Real Playwright subprocess when available; graceful
/// skip when `node` OR the `playwright` npm package is missing.
///
/// Audit WARN 3 fix (decision 76): the previous probe only checked for
/// `node` on PATH, which is a weak signal — `node` could be installed
/// system-wide without `playwright`, in which case the helper script
/// would emit `ERR:playwright_not_installed` and the test would fail
/// with an "unexpected MalformedOutput" panic. The stronger probe
/// runs `npx playwright --version` and treats the test as skipped if
/// either the binary or the package is missing.
#[tokio::test]
async fn rce_f_web_render_three_viewports() {
    if !binary_on_path("node") {
        eprintln!("TOOL_MISSING:node — skipping rce_f_web_render_three_viewports");
        return;
    }
    // Stronger probe — actually try `npx playwright --version`.
    if !binary_on_path("npx") {
        eprintln!(
            "TOOL_MISSING:npx — playwright package probably absent; skipping"
        );
        return;
    }
    let probe = std::process::Command::new("npx")
        .args(["playwright", "--version"])
        .output();
    match probe {
        Ok(o) if o.status.success() => {}
        Ok(_) | Err(_) => {
            eprintln!(
                "TOOL_MISSING:playwright — `npx playwright --version` did not succeed; \
                 skipping rce_f_web_render_three_viewports"
            );
            return;
        }
    }
    let a = WebRenderAdapter::new();
    let cfg = RenderConfig {
        viewports: a.default_viewports.clone(),
        ..RenderConfig::default()
    };
    let dir = TempDir::new().unwrap();
    let html = dir.path().join("index.html");
    std::fs::write(&html, b"<html><body><h1>hi</h1></body></html>").unwrap();
    match a.render(&html, &cfg).await {
        Ok(frames) => {
            assert_eq!(frames.len(), 3, "one frame per viewport");
            let widths: Vec<u32> = frames.iter().map(|f| f.viewport.width).collect();
            assert_eq!(widths, vec![375, 768, 1440]);
            for f in &frames {
                assert!(f.image_path.exists(), "frame PNG must exist: {:?}", f.image_path);
            }
        }
        Err(RenderAdapterError::ToolUnavailable { .. }) => {
            eprintln!("TOOL_MISSING:playwright — skipping (npm playwright not installed)");
        }
        Err(RenderAdapterError::MalformedOutput(m)) => {
            // Helper printed ERR:playwright_not_installed -> graceful skip.
            if m.contains("playwright_not_installed") {
                eprintln!("TOOL_MISSING:playwright — {m}");
                return;
            }
            panic!("unexpected MalformedOutput: {m}");
        }
        Err(other) => panic!("unexpected render error: {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// rce_f_blender_four_angles — graceful skip if blender absent.
// ---------------------------------------------------------------------------

/// Spec L462-463. Real Blender CLI subprocess when available; graceful
/// skip when `blender` is missing (CI lacks Blender by default).
#[tokio::test]
async fn rce_f_blender_four_angles() {
    if !binary_on_path("blender") {
        eprintln!("TOOL_MISSING:blender — skipping rce_f_blender_four_angles");
        return;
    }
    let a = BlenderRenderAdapter::new();
    let dir = TempDir::new().unwrap();
    let obj = dir.path().join("cube.obj");
    std::fs::write(
        &obj,
        b"v -1 -1 -1\nv 1 -1 -1\nv 1 1 -1\nv -1 1 -1\nv -1 -1 1\nv 1 -1 1\nv 1 1 1\nv -1 1 1\nf 1 2 3 4\nf 5 6 7 8\nf 1 2 6 5\nf 2 3 7 6\nf 3 4 8 7\nf 4 1 5 8\n",
    )
    .unwrap();
    match a.render(&obj, &RenderConfig::default()).await {
        Ok(frames) => {
            assert_eq!(frames.len(), 4, "front + right + top + perspective");
        }
        Err(RenderAdapterError::ToolUnavailable { .. }) => {
            eprintln!("TOOL_MISSING:blender — graceful skip");
        }
        Err(other) => panic!("unexpected blender render error: {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// rce_f_animation_key_frames — synthesize a real .mp4 via ffmpeg lavfi.
// ---------------------------------------------------------------------------

/// Spec L464-465. Real FFmpeg subprocess. Synthesizes a 2s testsrc
/// video via `ffmpeg -f lavfi -i testsrc=duration=2:rate=30 ...` so the
/// test runs without external fixtures. Graceful skip when ffmpeg is
/// missing.
#[tokio::test]
async fn rce_f_animation_key_frames() {
    if !binary_on_path("ffmpeg") {
        eprintln!("TOOL_MISSING:ffmpeg — skipping rce_f_animation_key_frames");
        return;
    }
    let dir = TempDir::new().unwrap();
    let mp4 = dir.path().join("clip.mp4");
    // Synthesize a 2s, 30fps test video (60 frames).
    let synth = std::process::Command::new("ffmpeg")
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-y",
            "-f",
            "lavfi",
            "-i",
            "testsrc=duration=2:size=128x128:rate=30",
            "-pix_fmt",
            "yuv420p",
        ])
        .arg(&mp4)
        .output()
        .expect("ffmpeg synth spawn");
    assert!(
        synth.status.success(),
        "ffmpeg synth failed: stderr={}",
        String::from_utf8_lossy(&synth.stderr)
    );
    assert!(mp4.exists(), "synthesized mp4 must exist");
    let a = AnimationRenderAdapter::new();
    let cfg = RenderConfig {
        capture_mode: elai_orchestrator::media_validator::render_adapters::CaptureMode::KeyFrames {
            interval: std::time::Duration::from_secs(1),
        },
        ..RenderConfig::default()
    };
    match a.render(&mp4, &cfg).await {
        Ok(frames) => {
            // 2 second clip with modulo=30 -> ~3 frames (indices 0,30,60).
            // Allow 1..=4 to tolerate FFmpeg version variance.
            assert!(
                (1..=4).contains(&frames.len()),
                "expected 1..=4 keyframes, got {}: {frames:?}",
                frames.len()
            );
            for f in &frames {
                assert!(f.image_path.exists(), "frame PNG must exist");
            }
        }
        Err(RenderAdapterError::ToolUnavailable { .. }) => {
            eprintln!("TOOL_MISSING:ffmpeg — graceful skip");
        }
        Err(other) => panic!("unexpected animation render error: {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// rce_f_ui_screenshot_pass_through — REAL.
// ---------------------------------------------------------------------------

/// Spec L466-467.
#[tokio::test]
async fn rce_f_ui_screenshot_pass_through() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("ui.png");
    std::fs::write(&path, TINY_PNG).unwrap();

    let adapter = UiScreenshotAdapter::new();
    let cfg = RenderConfig::default();

    let frames = adapter.render(&path, &cfg).await.unwrap();
    assert_eq!(frames.len(), 1);
    let frame = &frames[0];
    assert_eq!(frame.image_path, path, "no copy — same path returned");
    assert_eq!(frame.metadata.adapter_name, "ui_screenshot");
    assert!(frame
        .metadata
        .state_description
        .contains("png_pass_through"));

    let frames2 = adapter.render(&path, &cfg).await.unwrap();
    assert_eq!(frames.len(), frames2.len());
    assert_eq!(frame.image_path, frames2[0].image_path);
    assert_eq!(frame.viewport, frames2[0].viewport);
}

/// Additional REAL test: rejects non-image bytes.
#[tokio::test]
async fn rce_f_ui_screenshot_rejects_non_image() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("garbage.bin");
    std::fs::write(&path, &[0xCA, 0xFE, 0xBA, 0xBE]).unwrap();
    let adapter = UiScreenshotAdapter::new();
    let cfg = RenderConfig::default();
    let err = adapter.render(&path, &cfg).await.unwrap_err();
    assert!(
        matches!(err, RenderAdapterError::MalformedOutput(_)),
        "non-PNG/JPEG must be rejected as MalformedOutput"
    );
}

// ---------------------------------------------------------------------------
// rce_f_comfyui_mock_workflow — in-process tokio TcpListener mock.
// ---------------------------------------------------------------------------

mod comfyui_mock {
    //! Hand-rolled minimal HTTP server. Implements POST /prompt,
    //! GET /history/{id}, GET /view to satisfy the ComfyUI client.
    //! No external mockito/wiremock dependency.
    use std::sync::Arc;

    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;
    use tokio::sync::oneshot;

    /// 1x1 red PNG, same bytes used by other tests.
    const TINY_PNG: &[u8] = &[
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x00, 0x00, 0x00, 0x90,
        0x77, 0x53, 0xDE, 0x00, 0x00, 0x00, 0x0C, 0x49, 0x44, 0x41, 0x54, 0x08, 0xD7, 0x63, 0xF8,
        0xCF, 0xC0, 0x00, 0x00, 0x00, 0x03, 0x00, 0x01, 0x5B, 0x42, 0x5D, 0x6A, 0x00, 0x00, 0x00,
        0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
    ];

    pub struct MockHandle {
        pub addr: std::net::SocketAddr,
        pub shutdown: oneshot::Sender<()>,
    }

    /// Spawn the mock. Returns a `MockHandle` whose `shutdown` field
    /// drops the listener on send. The handle's `addr` is a 127.0.0.1
    /// loopback with an OS-assigned port.
    pub async fn spawn() -> MockHandle {
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind 0");
        let addr = listener.local_addr().expect("local_addr");
        let (tx, mut rx) = oneshot::channel::<()>();
        let prompt_id = Arc::new("test-prompt-id-1".to_string());

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = &mut rx => break,
                    accepted = listener.accept() => {
                        let Ok((mut sock, _peer)) = accepted else { break; };
                        let prompt_id = prompt_id.clone();
                        tokio::spawn(async move {
                            let _ = handle_conn(&mut sock, &prompt_id).await;
                        });
                    }
                }
            }
        });

        MockHandle { addr, shutdown: tx }
    }

    async fn handle_conn(sock: &mut tokio::net::TcpStream, prompt_id: &str) -> std::io::Result<()> {
        // Read up to 64 KiB of request — enough for a tiny workflow.
        let mut buf = vec![0u8; 65_536];
        let mut total = 0;
        loop {
            let n = sock.read(&mut buf[total..]).await?;
            if n == 0 {
                break;
            }
            total += n;
            // Look for end-of-headers \r\n\r\n
            if buf[..total].windows(4).any(|w| w == b"\r\n\r\n") {
                break;
            }
            if total >= buf.len() {
                break;
            }
        }
        let text = String::from_utf8_lossy(&buf[..total]).to_string();
        let first_line = text.lines().next().unwrap_or("").to_string();
        let parts: Vec<&str> = first_line.split_whitespace().collect();
        if parts.len() < 2 {
            return write_simple(sock, 400, "Bad Request", b"bad").await;
        }
        let method = parts[0];
        let path = parts[1];

        if method == "POST" && path.starts_with("/prompt") {
            let body = format!(r#"{{"prompt_id":"{prompt_id}"}}"#);
            return write_simple(sock, 200, "OK", body.as_bytes()).await;
        }
        if method == "GET" && path.starts_with("/history/") {
            // First call returns empty {}; second returns completed outputs.
            // We track via a counter file in temp — simplest deterministic
            // approach for the single-test scenario is to ALWAYS return
            // completed (the client tolerates immediate completion).
            let body = format!(
                r#"{{"{prompt_id}":{{"outputs":{{"node1":{{"images":[{{"filename":"out.png","subfolder":"","type":"output"}}]}}}}}}}}"#
            );
            return write_simple(sock, 200, "OK", body.as_bytes()).await;
        }
        if method == "GET" && path.starts_with("/view") {
            // Serve a PNG byte payload.
            let headers = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: image/png\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                TINY_PNG.len()
            );
            sock.write_all(headers.as_bytes()).await?;
            sock.write_all(TINY_PNG).await?;
            sock.shutdown().await?;
            return Ok(());
        }
        write_simple(sock, 404, "Not Found", b"nope").await
    }

    async fn write_simple(
        sock: &mut tokio::net::TcpStream,
        status: u16,
        reason: &str,
        body: &[u8],
    ) -> std::io::Result<()> {
        let resp = format!(
            "HTTP/1.1 {status} {reason}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            body.len()
        );
        sock.write_all(resp.as_bytes()).await?;
        sock.write_all(body).await?;
        sock.shutdown().await?;
        Ok(())
    }
}

/// Spec L468-469. In-process mock HTTP server simulating the ComfyUI
/// /prompt + /history/{id} + /view surface. Always runs (no external
/// tool needed).
#[tokio::test]
async fn rce_f_comfyui_mock_workflow() {
    let mock = comfyui_mock::spawn().await;
    let addr = mock.addr;
    let url = format!("http://{addr}");

    let a = ComfyUIRenderAdapter::with_url(&url);
    let dir = TempDir::new().unwrap();
    let workflow = dir.path().join("workflow.json");
    std::fs::write(
        &workflow,
        br#"{"prompt":{"1":{"class_type":"Stub","inputs":{}}}}"#,
    )
    .unwrap();
    let frames = a
        .render(&workflow, &RenderConfig::default())
        .await
        .expect("mock render must succeed");
    assert!(!frames.is_empty(), "mock server returns at least 1 frame");
    let first = &frames[0];
    assert_eq!(first.metadata.adapter_name, "comfyui");
    assert!(first.image_path.exists(), "downloaded image must be on disk");
    // Cleanup.
    let _ = mock.shutdown.send(());
    // Yield once so the listener observes the shutdown signal.
    tokio::task::yield_now().await;
}

// ---------------------------------------------------------------------------
// Sanity: name() invariants for all adapters.
// ---------------------------------------------------------------------------

#[test]
fn rce_f_adapter_names_are_stable() {
    assert_eq!(WebRenderAdapter::new().name(), "web");
    assert_eq!(BlenderRenderAdapter::new().name(), "blender");
    assert_eq!(AnimationRenderAdapter::new().name(), "animation");
    assert_eq!(UiScreenshotAdapter::new().name(), "ui_screenshot");
    assert_eq!(ComfyUIRenderAdapter::new().name(), "comfyui");
}

#[allow(dead_code)]
fn _silence_unused(_: &Path, _: Viewport) {}

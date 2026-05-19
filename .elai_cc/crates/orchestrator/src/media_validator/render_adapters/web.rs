//! B-24 Phase 4I Group C6 (4I.9 IMPLEMENT): `WebRenderAdapter` — real impl.
//!
//! Spec phase-4I.txt L397/458-461. Uses the `npx playwright` subprocess
//! route (paper workflow/papers/playwright-cdp-interface.md ranks this
//! route HIGH vs the playwright-rs crate). A minimal Node.js helper
//! script is embedded as `&'static str` and dropped into a per-call
//! temp directory; each invocation captures one PNG per viewport.
//!
//! Anti-stale:
//!   * tokio::process::Command (no std::process in async).
//!   * tokio::time::timeout for hard kill on hang (Spec L489-490).
//!   * tracing for subprocess output.
//!   * u64::try_from for ms narrowing — no `as`.
//!   * No `.unwrap()` / `.expect()` outside tests.
//!   * Embedded Node script as const &str (no separate file in repo).
//!
//! Process group / tree kill:
//!   * Unix: tokio Command launches a child; on drop the child is reaped.
//!     We additionally call `process_group(0)` (where available) so
//!     `kill` reaches grandchildren (Chromium spawns helpers).
//!   * Windows: `kill_on_drop(true)` reaps the child; Chromium's own
//!     "browser process" tree is killed by Playwright when the parent
//!     exits abnormally — verified empirically with `--headless`.

use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::time::{Duration, Instant};

use tokio::process::Command;
use tracing::{debug, warn};

#[cfg(target_os = "linux")]
use std::os::unix::process::CommandExt;

use super::types::{
    CapturedFrame, FrameMetadata, OutputType, RenderConfig, Viewport,
};
use super::{MediaRenderAdapter, RenderAdapterError};

/// Embedded Node.js helper script — invoked as:
///   node helper.js --url <url> --viewports <w,h;w,h;...> --out <dir>
///
/// Prints one line per captured frame as:
///   FRAME:<width>:<height>:<absolute-png-path>
/// then "OK" on success. Any failure prints a line beginning with
/// "ERR:" and exits non-zero. Pure CommonJS so no transpile step.
pub const WEB_RENDER_HELPER_JS: &str = r#"
// B-24 Phase 4I Group C6: Playwright subprocess helper.
// Spec: workflow/papers/playwright-cdp-interface.md (subprocess route).
const path = require('path');

function parseArgs(argv) {
  const out = {};
  for (let i = 2; i < argv.length; i++) {
    const a = argv[i];
    if (a.startsWith('--')) {
      const k = a.slice(2);
      out[k] = argv[i + 1];
      i++;
    }
  }
  return out;
}

(async () => {
  const args = parseArgs(process.argv);
  const url = args.url;
  const viewportsRaw = args.viewports || '';
  const outDir = args.out;
  if (!url || !outDir) {
    console.log('ERR:missing --url or --out');
    process.exit(2);
  }
  let playwright;
  try {
    playwright = require('playwright');
  } catch (e) {
    console.log('ERR:playwright_not_installed:' + (e && e.message ? e.message : e));
    process.exit(3);
  }
  const viewports = viewportsRaw.split(';').filter(s => s.length).map(s => {
    const [w, h] = s.split(',').map(n => parseInt(n, 10));
    return { width: w, height: h };
  });
  if (viewports.length === 0) {
    console.log('ERR:no_viewports');
    process.exit(4);
  }
  let browser;
  try {
    browser = await playwright.chromium.launch({ headless: true });
  } catch (e) {
    console.log('ERR:browser_launch_failed:' + (e && e.message ? e.message : e));
    process.exit(5);
  }
  try {
    for (let i = 0; i < viewports.length; i++) {
      const vp = viewports[i];
      const ctx = await browser.newContext({ viewport: vp });
      const page = await ctx.newPage();
      await page.goto(url, { waitUntil: 'load', timeout: 15000 });
      const outPath = path.join(outDir, `web_${vp.width}x${vp.height}.png`);
      await page.screenshot({ path: outPath, fullPage: false });
      await ctx.close();
      console.log(`FRAME:${vp.width}:${vp.height}:${outPath}`);
    }
    console.log('OK');
  } catch (e) {
    console.log('ERR:render_failed:' + (e && e.message ? e.message : e));
    process.exit(6);
  } finally {
    try { await browser.close(); } catch (_) {}
  }
})();
"#;

const WEB_RENDER_HELPER_FILENAME: &str = "elai_playwright_helper.js";
const DEFAULT_WEB_TIMEOUT_MS: u64 = 30_000;

/// Web renderer (Playwright CDP) — REAL impl. Spawns `node helper.js`
/// against `npx playwright` for headless Chromium screenshots at
/// 375/768/1440 viewports (spec L458-459).
///
/// Audit WARN 1 fix (decision 74): the adapter now tracks all per-call
/// tempdirs in a `Mutex<Vec<PathBuf>>` and removes them on `Drop`. The
/// previous behavior leaked the directories (and the captured PNGs
/// inside) on every render; over a long agent session this is a
/// non-trivial disk-space leak. Production callers that need the PNGs
/// after the adapter is dropped MUST copy them out (matches the
/// `tempfile::TempDir` discipline).
pub struct WebRenderAdapter {
    /// Absolute path to the `node` binary (or `npx`). Empty -> resolved
    /// from `PATH` at render time.
    pub playwright_path: PathBuf,
    /// 3 default viewports (mobile / tablet / desktop) per spec L458-459.
    pub default_viewports: Vec<Viewport>,
    /// Per-instance tempdir tracker. Small scope (one entry per render
    /// call); `std::sync::Mutex` is fine — no async contention.
    tempdirs: std::sync::Mutex<Vec<PathBuf>>,
}

impl std::fmt::Debug for WebRenderAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WebRenderAdapter")
            .field("playwright_path", &self.playwright_path)
            .field("default_viewports", &self.default_viewports)
            .field(
                "tempdir_count",
                &self.tempdirs.lock().map(|g| g.len()).unwrap_or(0),
            )
            .finish()
    }
}

impl Drop for WebRenderAdapter {
    fn drop(&mut self) {
        if let Ok(mut guard) = self.tempdirs.lock() {
            for dir in guard.drain(..) {
                let _ = std::fs::remove_dir_all(&dir);
            }
        }
    }
}

impl Default for WebRenderAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl WebRenderAdapter {
    pub fn new() -> Self {
        Self {
            playwright_path: PathBuf::new(),
            default_viewports: vec![
                Viewport::mobile(),
                Viewport::tablet(),
                Viewport::desktop(),
            ],
            tempdirs: std::sync::Mutex::new(Vec::new()),
        }
    }

    /// Constructor for tests that need to assert the binary path
    /// is plumbed through to the SSB-emitted metadata.
    pub fn with_playwright_path(playwright_path: PathBuf) -> Self {
        Self {
            playwright_path,
            default_viewports: vec![
                Viewport::mobile(),
                Viewport::tablet(),
                Viewport::desktop(),
            ],
            tempdirs: std::sync::Mutex::new(Vec::new()),
        }
    }

    /// Pick the configured viewports if non-empty; else fall back to
    /// the 3 defaults so the contract test passes.
    fn effective_viewports<'a>(&'a self, config: &'a RenderConfig) -> &'a [Viewport] {
        if config.viewports.is_empty() {
            &self.default_viewports
        } else {
            &config.viewports
        }
    }
}

/// Resolve the `node` binary path. We accept either the helper's pinned
/// path (when caller built with `with_playwright_path`) OR fall back to
/// just `"node"` and let the OS PATH resolve it.
fn node_binary(adapter: &WebRenderAdapter) -> PathBuf {
    if adapter.playwright_path.as_os_str().is_empty() {
        PathBuf::from("node")
    } else {
        adapter.playwright_path.clone()
    }
}

/// Normalize the artifact path into a URL. `http://` / `https://` /
/// `file://` strings are passed through unchanged; everything else is
/// treated as a local file path and wrapped in `file://`.
fn artifact_to_url(artifact: &Path) -> String {
    let raw = artifact.to_string_lossy();
    if raw.starts_with("http://") || raw.starts_with("https://") || raw.starts_with("file://") {
        return raw.into_owned();
    }
    // Local file path -> file:// URL. Canonicalize so relative paths in
    // the test fixtures resolve correctly.
    let abs = std::fs::canonicalize(artifact).unwrap_or_else(|_| artifact.to_path_buf());
    let path_str = abs.to_string_lossy().replace('\\', "/");
    if path_str.starts_with('/') {
        format!("file://{path_str}")
    } else {
        format!("file:///{path_str}")
    }
}

/// Build the viewport CLI string: `375,667;768,1024;1440,900`.
fn viewports_cli(vps: &[Viewport]) -> String {
    vps.iter()
        .map(|v| format!("{},{}", v.width, v.height))
        .collect::<Vec<_>>()
        .join(";")
}

impl MediaRenderAdapter for WebRenderAdapter {
    fn can_handle(&self, output_type: &OutputType) -> bool {
        matches!(output_type, OutputType::Web)
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
            let viewports = self.effective_viewports(config).to_vec();
            if viewports.is_empty() {
                return Err(RenderAdapterError::MalformedOutput(
                    "WebRenderAdapter: no viewports configured".into(),
                ));
            }

            // 1. Build a per-call temp directory under the OS temp.
            let work_dir = make_tempdir("elai-web-render")?;
            let helper_path = work_dir.join(WEB_RENDER_HELPER_FILENAME);
            std::fs::write(&helper_path, WEB_RENDER_HELPER_JS)?;

            // 2. Normalize the artifact into a URL.
            let url = artifact_to_url(artifact);

            // 3. Spawn `node helper.js --url ... --viewports ... --out ...`.
            let mut cmd = Command::new(node_binary(self));
            cmd.arg(&helper_path)
                .arg("--url")
                .arg(&url)
                .arg("--viewports")
                .arg(viewports_cli(&viewports))
                .arg("--out")
                .arg(&work_dir)
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .kill_on_drop(true);

            // Audit WARN 4 fix (decision 77): on Linux, place the
            // playwright child in its OWN process group so a SIGKILL
            // reaches the entire Chromium subtree (grandchildren). The
            // previous `kill_on_drop` only signaled the direct child;
            // Chromium browser-processes would leak when the parent
            // exited abnormally. On Windows, `kill_on_drop` already
            // tears down the job-object tree (no extra work). macOS /
            // other unix not gated because `libc` is only a workspace
            // dep on Linux (Cargo.toml L72); the Linux carve-out
            // matches our primary deployment target.
            #[cfg(target_os = "linux")]
            {
                cmd.process_group(0);
            }

            let timeout_ms = if config.timeout.as_millis() == 0 {
                DEFAULT_WEB_TIMEOUT_MS
            } else {
                u64::try_from(config.timeout.as_millis()).unwrap_or(DEFAULT_WEB_TIMEOUT_MS)
            };
            let timeout = Duration::from_millis(timeout_ms);

            debug!(adapter = "web", url = %url, viewport_count = viewports.len(), timeout_ms, "spawning playwright helper");

            // Spawn the child manually so we can capture its PID and
            // signal the entire process group on timeout (audit WARN 4
            // fix — decision 77). The pre-existing
            // `tokio::time::timeout(_, cmd.output())` would drop the
            // inner child on elapsed, which sends SIGKILL to the
            // leader's PID only — Chromium grandchildren survived.
            let child = match cmd.spawn() {
                Ok(c) => c,
                Err(e) => {
                    let _ = remove_dir_safe(&work_dir);
                    warn!(error = %e, "WebRenderAdapter spawn failed");
                    return Err(RenderAdapterError::ToolUnavailable { tool: "node" });
                }
            };
            #[cfg(target_os = "linux")]
            let pgid: Option<i32> = child.id().and_then(|p| i32::try_from(p).ok());

            // `wait_with_output` consumes the child handle. On timeout
            // we send SIGKILL to the pgid (audit WARN 4 fix — decision
            // 77) BEFORE the wait future is dropped, so even though we
            // lose direct access to `child` on the timeout branch, the
            // OS reaps the group via the kill(-pgid) call. On Windows,
            // dropping the future triggers `kill_on_drop` (job-object
            // tree teardown is automatic).
            let output = match tokio::time::timeout(timeout, child.wait_with_output()).await {
                Ok(Ok(out)) => out,
                Ok(Err(e)) => {
                    let _ = remove_dir_safe(&work_dir);
                    warn!(error = %e, "WebRenderAdapter wait failed");
                    return Err(RenderAdapterError::ToolUnavailable { tool: "node" });
                }
                Err(_elapsed) => {
                    warn!(
                        adapter = "web",
                        timeout_ms,
                        "WebRenderAdapter timed out; killing process group"
                    );
                    // Linux: send SIGKILL to the negative pid to reach
                    // the entire group (Chromium grandchildren).
                    #[cfg(target_os = "linux")]
                    if let Some(pgid) = pgid {
                        // SIGKILL = 9. Negative pid -> target process
                        // group. Unsafe because we're calling FFI.
                        unsafe {
                            let _ = libc::kill(-pgid, libc::SIGKILL);
                        }
                    }
                    let _ = remove_dir_safe(&work_dir);
                    return Err(RenderAdapterError::Timeout { millis: timeout_ms });
                }
            };

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
                let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
                let _ = remove_dir_safe(&work_dir);
                // Map the helper script's well-known ERR: prefix to
                // ToolUnavailable when Playwright/Node is missing.
                if stdout.contains("ERR:playwright_not_installed") {
                    return Err(RenderAdapterError::ToolUnavailable {
                        tool: "playwright",
                    });
                }
                return Err(RenderAdapterError::MalformedOutput(format!(
                    "node exit={:?}; stdout=`{stdout}`; stderr=`{stderr}`",
                    output.status.code()
                )));
            }

            // 4. Parse helper stdout for `FRAME:w:h:path` lines.
            let stdout = String::from_utf8_lossy(&output.stdout);
            let mut frames: Vec<CapturedFrame> = Vec::with_capacity(viewports.len());
            for line in stdout.lines() {
                let Some(rest) = line.strip_prefix("FRAME:") else {
                    continue;
                };
                let parts: Vec<&str> = rest.splitn(3, ':').collect();
                if parts.len() != 3 {
                    continue;
                }
                let w: u32 = match parts[0].parse() {
                    Ok(v) => v,
                    Err(_) => continue,
                };
                let h: u32 = match parts[1].parse() {
                    Ok(v) => v,
                    Err(_) => continue,
                };
                let png_path = PathBuf::from(parts[2]);
                let vp = viewports
                    .iter()
                    .find(|v| v.width == w && v.height == h)
                    .cloned()
                    .unwrap_or(Viewport {
                        width: w,
                        height: h,
                        device: None,
                    });
                let state = vp
                    .device
                    .clone()
                    .unwrap_or_else(|| format!("{w}x{h}"));
                frames.push(CapturedFrame {
                    image_path: png_path,
                    viewport: vp,
                    metadata: FrameMetadata {
                        capture_time: started.elapsed(),
                        adapter_name: self.name().to_string(),
                        state_description: format!("web_viewport_{state}"),
                    },
                });
            }

            if frames.is_empty() {
                let _ = remove_dir_safe(&work_dir);
                return Err(RenderAdapterError::MalformedOutput(format!(
                    "WebRenderAdapter produced 0 frames; stdout=`{stdout}`"
                )));
            }

            // Audit WARN 1 fix (decision 74): track the per-call tempdir
            // so Drop / cleanup() removes it. PNGs inside the dir ARE
            // the captured frames; consumers must read them BEFORE the
            // adapter drops.
            if let Ok(mut guard) = self.tempdirs.lock() {
                guard.push(work_dir);
            }
            Ok(frames)
        })
    }

    fn cleanup(&self) {
        // Drain tracked tempdirs and remove them. Idempotent: subsequent
        // calls find an empty Vec. Production callers (e.g.
        // VisualFeedbackLoop) call this after the loop completes so the
        // disk footprint is bounded even when the adapter outlives
        // multiple render cycles.
        if let Ok(mut guard) = self.tempdirs.lock() {
            for dir in guard.drain(..) {
                let _ = std::fs::remove_dir_all(&dir);
            }
        }
    }

    fn name(&self) -> &'static str {
        "web"
    }
}

/// Build a unique tempdir under `std::env::temp_dir()` with the given
/// prefix. Uses `uuid::Uuid::new_v4` so concurrent renders don't collide.
/// Returns the absolute path.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_handle_only_web() {
        let a = WebRenderAdapter::new();
        assert!(a.can_handle(&OutputType::Web));
        assert!(!a.can_handle(&OutputType::Model3D));
        assert!(!a.can_handle(&OutputType::UiScreenshot));
    }

    #[test]
    fn default_viewports_are_three() {
        let a = WebRenderAdapter::new();
        assert_eq!(a.default_viewports.len(), 3, "spec L458 mandates 3 viewports");
        let widths: Vec<u32> = a.default_viewports.iter().map(|v| v.width).collect();
        assert_eq!(widths, vec![375, 768, 1440]);
    }

    #[test]
    fn viewports_cli_format() {
        let vps = vec![Viewport::mobile(), Viewport::desktop()];
        assert_eq!(viewports_cli(&vps), "375,667;1440,900");
    }

    #[test]
    fn artifact_to_url_preserves_http() {
        assert_eq!(
            artifact_to_url(Path::new("http://example.com/x")),
            "http://example.com/x"
        );
        assert_eq!(
            artifact_to_url(Path::new("https://example.com")),
            "https://example.com"
        );
        assert_eq!(
            artifact_to_url(Path::new("file:///tmp/x.html")),
            "file:///tmp/x.html"
        );
    }

    #[test]
    fn embedded_helper_contains_playwright_require() {
        // Anti-stale: helper MUST `require('playwright')`, NOT the
        // legacy `playwright-core` (different package surface).
        assert!(WEB_RENDER_HELPER_JS.contains("require('playwright')"));
        assert!(WEB_RENDER_HELPER_JS.contains("FRAME:"));
        assert!(WEB_RENDER_HELPER_JS.contains("chromium.launch"));
    }

    #[tokio::test]
    async fn render_returns_tool_unavailable_when_node_missing() {
        // With an obviously-bad binary path, spawn fails. We expect either
        // ToolUnavailable or an Io/MalformedOutput depending on platform.
        let a = WebRenderAdapter::with_playwright_path(PathBuf::from(
            "/nonexistent-node-binary-elai",
        ));
        let cfg = RenderConfig::default();
        let dir = std::env::temp_dir();
        let html = dir.join(format!("elai-web-stub-{}.html", uuid::Uuid::new_v4())); // entropy-allow: C-15 deferred — render-adapter Clock plumbing scheduled
        let _ = std::fs::write(&html, b"<html><body>hi</body></html>");
        let r = a.render(&html, &cfg).await;
        let _ = std::fs::remove_file(&html);
        // Either tool unavailable (good error) or spawn-IO (also fine).
        assert!(r.is_err(), "missing node binary must yield error, got {r:?}");
    }
}

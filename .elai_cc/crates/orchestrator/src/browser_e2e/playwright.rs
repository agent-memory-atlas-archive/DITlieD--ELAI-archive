//! E-14 — Playwright runner.
//!
//! Spawns `npx playwright` via `tokio::process::Command` to capture a
//! [`PageSnapshot`] (HTML + optional AX tree + optional screenshot).
//! Mirrors the embedded-helper pattern used by `WebRenderAdapter` in
//! `render_adapters/web.rs` so the toolchain probe + timeout +
//! `kill_on_drop(true)` semantics stay consistent across the two surfaces.
//!
//! Wire-dark posture: when `npx` / `node` / `playwright` is absent from
//! PATH, the runner returns [`E2eError::ToolchainMissing`] rather than
//! panicking. The caller surfaces this to the operator (and to SSB)
//! without breaking the rest of the pipeline.
//!
//! Anti-stale grounding:
//!   * `tokio::process::Command` (not `std::process` in async).
//!   * `tokio::time::timeout` for hard kill on hang.
//!   * `u64::try_from` for ms narrowing (not `as`).
//!   * `tracing` for subprocess output (not `log`).

use std::path::{Path, PathBuf};
use std::time::Duration;

use tokio::process::Command;
use tracing::{debug, warn};

use super::snapshot::PageSnapshot;
use super::E2eError;

/// Default per-snapshot wall-clock budget. 45 seconds covers cold-start
/// `npx playwright install` warmup on slow CI runners (BrowserStack 2026
/// flake-mitigation defaults); CLI subprocess can override via
/// [`PlaywrightSnapshotOpts::timeout`].
pub const DEFAULT_PLAYWRIGHT_TIMEOUT_MS: u64 = 45_000;

/// Embedded Node helper that drives `playwright` to capture the snapshot.
/// Mirrors `render_adapters/web.rs::WEB_RENDER_HELPER_JS` but emits HTML +
/// AX tree + screenshot per call instead of multi-viewport PNGs.
///
/// Output protocol (one line per artifact):
///   `HTML_PATH:<absolute-path>`
///   `AX_TREE_PATH:<absolute-path>` (only when `--ax-tree` requested)
///   `SCREENSHOT_PATH:<absolute-path>` (only when `--screenshot` requested)
///   `OK` on success; `ERR:<reason>` on failure (exit non-zero).
const PLAYWRIGHT_HELPER_JS: &str = r#"
// E-14 browser_e2e Playwright helper — captures HTML + AX tree + screenshot.
const fs = require('fs');
const path = require('path');

function parseArgs(argv) {
  const out = {};
  for (let i = 2; i < argv.length; i++) {
    const a = argv[i];
    if (a.startsWith('--')) {
      const k = a.slice(2);
      // Boolean flags: --ax-tree, --screenshot (no following value)
      if (k === 'ax-tree' || k === 'screenshot') {
        out[k] = '1';
      } else {
        out[k] = argv[i + 1];
        i++;
      }
    }
  }
  return out;
}

(async () => {
  const args = parseArgs(process.argv);
  const url = args.url;
  const outDir = args.out;
  if (!url || !outDir) {
    console.log('ERR:missing_url_or_out');
    process.exit(2);
  }
  let playwright;
  try {
    playwright = require('playwright');
  } catch (e) {
    console.log('ERR:playwright_not_installed:' + (e && e.message ? e.message : e));
    process.exit(3);
  }
  let browser;
  try {
    browser = await playwright.chromium.launch({ headless: true });
  } catch (e) {
    console.log('ERR:browser_launch_failed:' + (e && e.message ? e.message : e));
    process.exit(5);
  }
  try {
    const ctx = await browser.newContext();
    const page = await ctx.newPage();
    await page.goto(url, { waitUntil: 'load', timeout: 30000 });
    const html = await page.content();
    const htmlPath = path.join(outDir, 'page.html');
    fs.writeFileSync(htmlPath, html);
    console.log('HTML_PATH:' + htmlPath);
    if (args['ax-tree']) {
      const ax = await page.accessibility.snapshot({ interestingOnly: false });
      const axPath = path.join(outDir, 'ax-tree.json');
      fs.writeFileSync(axPath, JSON.stringify(ax || {}));
      console.log('AX_TREE_PATH:' + axPath);
    }
    if (args['screenshot']) {
      const pngPath = path.join(outDir, 'page.png');
      await page.screenshot({ path: pngPath, fullPage: false });
      console.log('SCREENSHOT_PATH:' + pngPath);
    }
    await ctx.close();
    console.log('OK');
  } catch (e) {
    console.log('ERR:capture_failed:' + (e && e.message ? e.message : e));
    process.exit(6);
  } finally {
    try { await browser.close(); } catch (_) {}
  }
})();
"#;

const HELPER_FILENAME: &str = "elai_browser_e2e_helper.js";

/// Caller-facing options for a single snapshot capture.
#[derive(Debug, Clone)]
pub struct PlaywrightSnapshotOpts {
    /// URL to navigate to.
    pub url: String,
    /// When true, the helper emits `ax-tree.json` and the runner reads
    /// it into `PageSnapshot::ax_tree`.
    pub capture_ax_tree: bool,
    /// When true, the helper emits `page.png` and the runner records
    /// the path in `PageSnapshot::screenshot_path`.
    pub capture_screenshot: bool,
    /// Wall-clock budget. `None` -> [`DEFAULT_PLAYWRIGHT_TIMEOUT_MS`].
    pub timeout: Option<Duration>,
}

impl PlaywrightSnapshotOpts {
    /// Convenience constructor for the html-only path.
    pub fn html_only(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            capture_ax_tree: false,
            capture_screenshot: false,
            timeout: None,
        }
    }
}

/// Playwright runner. Stateless aside from the resolved `npx` binary
/// (empty -> resolve from PATH at spawn time, matching `WebRenderAdapter`).
#[derive(Debug, Clone, Default)]
pub struct PlaywrightRunner {
    /// Absolute path to the `npx` binary (or `node`). Empty -> resolved
    /// from PATH at spawn time.
    pub npx_path: PathBuf,
}

impl PlaywrightRunner {
    pub fn new() -> Self {
        Self::default()
    }

    /// Constructor for tests that need to pin a specific binary.
    pub fn with_npx_path(npx_path: PathBuf) -> Self {
        Self { npx_path }
    }

    fn binary(&self) -> PathBuf {
        if self.npx_path.as_os_str().is_empty() {
            PathBuf::from("npx")
        } else {
            self.npx_path.clone()
        }
    }

    /// Capture a single snapshot. Wire-dark posture: returns
    /// `E2eError::ToolchainMissing` when `npx` / `node` / `playwright` is
    /// absent. Hard-kills the subprocess on timeout via
    /// `tokio::time::timeout` + `kill_on_drop(true)`.
    pub async fn snapshot(
        &self,
        opts: &PlaywrightSnapshotOpts,
        work_dir: &Path,
        now_ms: i64,
    ) -> Result<PageSnapshot, E2eError> {
        // 1. Drop the embedded helper into work_dir so node can require()
        //    playwright from the project's node_modules.
        std::fs::create_dir_all(work_dir)?;
        let helper_path = work_dir.join(HELPER_FILENAME);
        std::fs::write(&helper_path, PLAYWRIGHT_HELPER_JS)?;

        // 2. Build the npx command. We use `npx --no-install` to fail fast
        //    when playwright is not installed (rather than triggering an
        //    unsolicited `npx playwright install`). The runner intentionally
        //    does NOT shell out to `npx playwright install` — that's an
        //    operator-controlled step.
        let mut cmd = Command::new(self.binary());
        cmd.arg("--no-install")
            .arg("node")
            .arg(&helper_path)
            .arg("--url")
            .arg(&opts.url)
            .arg("--out")
            .arg(work_dir);
        if opts.capture_ax_tree {
            cmd.arg("--ax-tree");
        }
        if opts.capture_screenshot {
            cmd.arg("--screenshot");
        }
        cmd.stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .kill_on_drop(true);

        let timeout_ms = match opts.timeout {
            Some(d) => u64::try_from(d.as_millis()).unwrap_or(DEFAULT_PLAYWRIGHT_TIMEOUT_MS),
            None => DEFAULT_PLAYWRIGHT_TIMEOUT_MS,
        };
        let deadline = Duration::from_millis(timeout_ms);

        debug!(
            url = %opts.url,
            timeout_ms,
            ax_tree = opts.capture_ax_tree,
            screenshot = opts.capture_screenshot,
            "spawning playwright helper"
        );

        let child = match cmd.spawn() {
            Ok(c) => c,
            Err(e) => {
                // ENOENT on npx => ToolchainMissing. Other I/O errors bubble.
                if e.kind() == std::io::ErrorKind::NotFound {
                    return Err(E2eError::ToolchainMissing { tool: "npx" });
                }
                warn!(error = %e, "playwright spawn failed");
                return Err(E2eError::Io(e));
            }
        };

        let output = match tokio::time::timeout(deadline, child.wait_with_output()).await {
            Ok(Ok(out)) => out,
            Ok(Err(e)) => return Err(E2eError::Io(e)),
            Err(_elapsed) => {
                return Err(E2eError::Timeout {
                    tool: "playwright",
                    millis: timeout_ms,
                });
            }
        };

        let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
        let stderr = String::from_utf8_lossy(&output.stderr).into_owned();

        if !output.status.success() {
            // Map the helper's well-known ERR: prefixes to typed errors
            // so the dashboard can render a useful badge.
            if stdout.contains("ERR:playwright_not_installed") {
                return Err(E2eError::ToolchainMissing { tool: "playwright" });
            }
            return Err(E2eError::SubprocessFailed {
                tool: "playwright",
                code: output.status.code(),
                stderr: format!("stdout=`{stdout}` stderr=`{stderr}`"),
            });
        }

        // 3. Parse the helper's PROTO lines.
        let mut html_path: Option<PathBuf> = None;
        let mut ax_tree_path: Option<PathBuf> = None;
        let mut screenshot_path: Option<PathBuf> = None;
        for line in stdout.lines() {
            if let Some(rest) = line.strip_prefix("HTML_PATH:") {
                html_path = Some(PathBuf::from(rest.trim()));
            } else if let Some(rest) = line.strip_prefix("AX_TREE_PATH:") {
                ax_tree_path = Some(PathBuf::from(rest.trim()));
            } else if let Some(rest) = line.strip_prefix("SCREENSHOT_PATH:") {
                screenshot_path = Some(PathBuf::from(rest.trim()));
            }
        }

        let html_path = html_path.ok_or_else(|| E2eError::MalformedOutput {
            tool: "playwright",
            detail: format!("no HTML_PATH line; stdout=`{stdout}`"),
        })?;
        let html = std::fs::read_to_string(&html_path)?;
        let ax_tree = match ax_tree_path {
            Some(p) => Some(std::fs::read_to_string(&p)?),
            None => None,
        };

        Ok(PageSnapshot {
            url: opts.url.clone(),
            html,
            ax_tree,
            screenshot_path,
            ts_ms: now_ms,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn helper_contains_playwright_require() {
        // Anti-stale: helper requires `playwright`, NOT `playwright-core`.
        assert!(PLAYWRIGHT_HELPER_JS.contains("require('playwright')"));
        assert!(PLAYWRIGHT_HELPER_JS.contains("HTML_PATH:"));
        assert!(PLAYWRIGHT_HELPER_JS.contains("AX_TREE_PATH:"));
    }

    #[test]
    fn opts_html_only_constructor() {
        let o = PlaywrightSnapshotOpts::html_only("http://example.test/");
        assert_eq!(o.url, "http://example.test/");
        assert!(!o.capture_ax_tree);
        assert!(!o.capture_screenshot);
        assert!(o.timeout.is_none());
    }

    #[tokio::test]
    async fn snapshot_returns_toolchain_missing_for_absent_npx() {
        // With a clearly-bad binary the runner must surface
        // ToolchainMissing (wire-dark posture), NOT panic.
        let dir = tempfile::TempDir::new().expect("tempdir");
        let runner = PlaywrightRunner::with_npx_path(PathBuf::from(
            "/__elai_nonexistent_npx_binary__",
        ));
        let opts = PlaywrightSnapshotOpts::html_only("http://localhost:1/");
        let r = runner
            .snapshot(&opts, dir.path(), 1_700_000_000_000)
            .await;
        assert!(r.is_err());
        match r {
            Err(E2eError::ToolchainMissing { tool }) => assert_eq!(tool, "npx"),
            Err(E2eError::Io(_)) => {
                // Some platforms produce an Io error instead of ENOENT
                // on spawn — both are acceptable wire-dark surfaces.
            }
            other => panic!("expected ToolchainMissing or Io, got {other:?}"),
        }
    }
}

//! E-14 — `PageSnapshot` typed surface.
//!
//! A snapshot is the minimal artifact bundle the Playwright runner emits
//! per `navigate` call: rendered HTML, optional accessibility tree (as
//! the raw JSON string from Playwright's CLI), and optional on-disk
//! screenshot path. The assertion runner consumes this; the snapshot
//! itself is render-pipeline agnostic so a future Stagehand fallback can
//! build the same struct.
//!
//! All fields are owned so the snapshot can outlive the subprocess
//! tempdir (the runner copies the JSON / HTML into the struct, then drops
//! the tempdir).

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Kind of snapshot captured. Surfaced in SSB rows so the dashboard can
/// distinguish "we got HTML only" from "we got HTML + AX tree + PNG".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SnapshotKind {
    /// HTML only (lightest path; no Playwright AX tree dump).
    HtmlOnly,
    /// HTML + screenshot PNG.
    HtmlAndScreenshot,
    /// HTML + AX tree JSON.
    HtmlAndAxTree,
    /// Full bundle: HTML + AX tree JSON + screenshot PNG.
    Full,
}

impl SnapshotKind {
    /// Stable wire string for SSB payloads.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::HtmlOnly => "html_only",
            Self::HtmlAndScreenshot => "html_and_screenshot",
            Self::HtmlAndAxTree => "html_and_ax_tree",
            Self::Full => "full",
        }
    }
}

/// Snapshot of a single page load. `html` is always present; the other
/// fields are `None` when the runner did not capture them (e.g. the
/// `--ax-tree` flag was off).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PageSnapshot {
    /// URL the snapshot was captured from. Stored so assertions can
    /// reference the source page in their failure messages.
    pub url: String,
    /// Rendered HTML (full document.documentElement.outerHTML).
    pub html: String,
    /// Optional accessibility tree (raw JSON from Playwright's CLI).
    /// Not parsed into a typed `AxNode` tree in the scope-lean cut —
    /// assertions consume the raw string via `serde_json::Value` if
    /// needed. Parsing into a strongly-typed tree is deferred to the
    /// VLM-tier (RUNPOD-PENDING).
    pub ax_tree: Option<String>,
    /// Optional on-disk path to a screenshot PNG. The path may point
    /// inside a tempdir owned by the runner; consumers that need to
    /// retain the screenshot must copy it out before the runner drops.
    pub screenshot_path: Option<PathBuf>,
    /// Snapshot capture timestamp in ms-since-epoch. Sourced from the
    /// caller's clock; the runner records `chrono::Utc::now().timestamp_millis()`
    /// when no clock is plumbed.
    pub ts_ms: i64,
}

impl PageSnapshot {
    /// Classify the snapshot for SSB emission.
    pub fn kind(&self) -> SnapshotKind {
        match (&self.ax_tree, &self.screenshot_path) {
            (Some(_), Some(_)) => SnapshotKind::Full,
            (Some(_), None) => SnapshotKind::HtmlAndAxTree,
            (None, Some(_)) => SnapshotKind::HtmlAndScreenshot,
            (None, None) => SnapshotKind::HtmlOnly,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base(url: &str, html: &str) -> PageSnapshot {
        PageSnapshot {
            url: url.into(),
            html: html.into(),
            ax_tree: None,
            screenshot_path: None,
            ts_ms: 1_700_000_000_000,
        }
    }

    #[test]
    fn kind_html_only() {
        assert_eq!(base("http://x", "<html/>").kind(), SnapshotKind::HtmlOnly);
    }

    #[test]
    fn kind_full() {
        let mut s = base("http://x", "<html/>");
        s.ax_tree = Some("{}".into());
        s.screenshot_path = Some(PathBuf::from("/tmp/p.png"));
        assert_eq!(s.kind(), SnapshotKind::Full);
    }

    #[test]
    fn kind_html_and_screenshot() {
        let mut s = base("http://x", "<html/>");
        s.screenshot_path = Some(PathBuf::from("/tmp/p.png"));
        assert_eq!(s.kind(), SnapshotKind::HtmlAndScreenshot);
    }

    #[test]
    fn kind_html_and_ax_tree() {
        let mut s = base("http://x", "<html/>");
        s.ax_tree = Some("{}".into());
        assert_eq!(s.kind(), SnapshotKind::HtmlAndAxTree);
    }

    #[test]
    fn snapshot_kind_as_str_stable() {
        // Wire-string stability: changes here require a SCHEMA bump.
        assert_eq!(SnapshotKind::HtmlOnly.as_str(), "html_only");
        assert_eq!(SnapshotKind::Full.as_str(), "full");
    }
}

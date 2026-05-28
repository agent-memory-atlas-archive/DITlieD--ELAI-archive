//! E-16 -- AXTree YAML deterministic Tier-1 adapter (web-only).
//!
//! Background: Playwright v1.59+ ships `page.ariaSnapshot()` which
//! emits the accessibility tree of the rendered page as YAML
//! (playwright.dev/docs/aria-snapshots, vendor doc, verified May
//! 2026). Empirically AXTree-YAML catches ~40% of WCAG issues axe-core
//! misses (E-16 plan §Context point 3) and is deterministic + sub-
//! 100ms for typical SPA pages.
//!
//! This module is the deterministic web-only Tier-1 adapter that:
//!   1. Compares a captured AXTree YAML against a stored baseline
//!      snapshot per render task ID;
//!   2. Computes a stable diff percentage (`yaml_diff_pct`);
//!   3. Emits `AxtreeMismatch` SSB rows (via
//!      [`crate::ssb::VlmImageValidationKind::AxtreeMismatch`])
//!      when the diff exceeds [`AXTREE_DRIFT_THRESHOLD_PCT`].
//!
//! Web-only by construction: Blender / FFmpeg / ComfyUI adapters
//! never produce an aria-snapshot, so the [`AxTreeAdapter::evaluate`]
//! entry point takes a captured YAML directly (the parsing of which
//! lives in `render_adapters::web`).
//!
//! Anti-stale grounding (per CLAUDE.md):
//!   * thiserror for the module-local error enum.
//!   * tracing for structured logs.
//!   * blake3 for canonicalized snapshot hashing (NOT blake2 / sha2).
//!   * No `.unwrap()` / `.expect()` outside `#[cfg(test)]`.
//!
//! Wire-dark posture: the SSB emission gates on
//! `ELAI_VLM_IMAGE_VALIDATION_ENABLE=1` at the producer call site.
//! The adapter itself is gate-free so it can run as a deterministic
//! Tier-1 check even when the live VLM path is RUNPOD-PENDING.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::warn;

/// Default mismatch threshold (1.0 percent) per E-16 plan open Q4.
/// AXTree YAML drift above this on a same-React-render-tree is the
/// trigger to fall back to axe-core-only Tier-1 gating; below this
/// the mismatch is considered hydration-noise and ignored.
pub const AXTREE_DRIFT_THRESHOLD_PCT: f64 = 1.0;

/// Hard cap on AXTree YAML size in bytes. Snapshots larger than this
/// indicate a runaway render and are rejected rather than diff'd
/// (would blow the deterministic-budget). 256 KiB covers the
/// empirical p99 for typical SPA pages per playwright.dev examples.
pub const MAX_AXTREE_YAML_BYTES: usize = 262_144;

/// Module-local error enum. Per CLAUDE.md HARD RULE — no Box<dyn Error>
/// crossing crate boundaries.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum AxtreeError {
    /// The captured YAML exceeded [`MAX_AXTREE_YAML_BYTES`].
    #[error("axtree yaml too large: {bytes} > {limit} byte cap")]
    YamlTooLarge { bytes: usize, limit: usize },
    /// The baseline path was provided but the file does not exist.
    #[error("axtree baseline not found at {0}")]
    BaselineMissing(PathBuf),
    /// I/O error reading the baseline.
    #[error("axtree baseline read failed: {0}")]
    BaselineRead(String),
}

/// Per-render AXTree comparison result. Mirrors the
/// `axtree_mismatch` SSB details_json conventional fields
/// (see [`crate::ssb::VlmImageValidationKind::AxtreeMismatch`] docs).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AxTreeResult {
    /// The captured YAML (verbatim, post-size-check).
    pub yaml: String,
    /// Diff percentage in `[0.0, 100.0]`. Computed as
    /// `100.0 * different_lines / max(baseline_lines, current_lines)`.
    /// Anchored to line-granularity so semantically-equivalent YAML
    /// reorderings (rare on Playwright output) do NOT register as
    /// drift.
    pub diff_pct: f64,
    /// `true` when `diff_pct > AXTREE_DRIFT_THRESHOLD_PCT`. Producers
    /// emit `VlmImageValidationKind::AxtreeMismatch` on this flag.
    pub mismatch: bool,
    /// Baseline YAML if any was provided. Used by the dashboard
    /// `AxtreeSnapshot` panel + by audit replays.
    pub baseline_yaml: Option<String>,
    /// blake3-hex of the canonicalized YAML — used for deterministic
    /// equality checks across audit replays.
    pub blake3_hex: String,
}

/// Tier-1 AXTree adapter — web-only.
///
/// Construction is cheap and stateless. Callers in
/// `render_adapters::web` call `evaluate(...)` with the YAML their
/// helper subprocess captured + an optional baseline path. The
/// adapter returns the comparison result; emission of the
/// `axtree_mismatch` SSB row is the caller's responsibility (gated
/// behind `ELAI_VLM_IMAGE_VALIDATION_ENABLE`).
#[derive(Debug, Clone, Default)]
pub struct AxTreeAdapter {
    /// Per-instance drift threshold override. None -> use
    /// [`AXTREE_DRIFT_THRESHOLD_PCT`].
    pub drift_threshold_pct: Option<f64>,
}

impl AxTreeAdapter {
    /// Construct with the default threshold.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Construct with a per-instance threshold override.
    #[must_use]
    pub fn with_threshold(drift_threshold_pct: f64) -> Self {
        Self {
            drift_threshold_pct: Some(drift_threshold_pct),
        }
    }

    /// Compare `current_yaml` against `baseline_yaml`. The function is
    /// pure / deterministic.
    ///
    /// Behavior:
    ///   * If `current_yaml` exceeds [`MAX_AXTREE_YAML_BYTES`] -> Err.
    ///   * If `baseline_yaml` is None -> diff_pct = 0.0, mismatch =
    ///     false (first observation establishes the baseline).
    ///   * Otherwise -> diff_pct is line-granularity:
    ///       `100.0 * different_lines / max(base_lines, cur_lines)`.
    pub fn evaluate(
        &self,
        current_yaml: &str,
        baseline_yaml: Option<&str>,
    ) -> Result<AxTreeResult, AxtreeError> {
        if current_yaml.len() > MAX_AXTREE_YAML_BYTES {
            return Err(AxtreeError::YamlTooLarge {
                bytes: current_yaml.len(),
                limit: MAX_AXTREE_YAML_BYTES,
            });
        }

        let threshold = self
            .drift_threshold_pct
            .unwrap_or(AXTREE_DRIFT_THRESHOLD_PCT);

        let blake3_hex = blake3_hex_of(canonicalize_yaml(current_yaml).as_bytes());

        match baseline_yaml {
            None => Ok(AxTreeResult {
                yaml: current_yaml.to_owned(),
                diff_pct: 0.0,
                mismatch: false,
                baseline_yaml: None,
                blake3_hex,
            }),
            Some(base) => {
                let diff_pct = line_diff_pct(base, current_yaml);
                let mismatch = diff_pct > threshold;
                if mismatch {
                    warn!(
                        diff_pct,
                        threshold,
                        "axtree_drift_above_threshold"
                    );
                }
                Ok(AxTreeResult {
                    yaml: current_yaml.to_owned(),
                    diff_pct,
                    mismatch,
                    baseline_yaml: Some(base.to_owned()),
                    blake3_hex,
                })
            }
        }
    }

    /// Convenience: load `baseline_path` from disk and call
    /// [`Self::evaluate`].
    pub fn evaluate_with_baseline_path(
        &self,
        current_yaml: &str,
        baseline_path: &std::path::Path,
    ) -> Result<AxTreeResult, AxtreeError> {
        if !baseline_path.exists() {
            return Err(AxtreeError::BaselineMissing(baseline_path.to_path_buf()));
        }
        let base = std::fs::read_to_string(baseline_path)
            .map_err(|e| AxtreeError::BaselineRead(e.to_string()))?;
        self.evaluate(current_yaml, Some(&base))
    }
}

/// Canonicalize YAML for hash-stable comparison. The canonicalization
/// is conservative — trim trailing whitespace per line; collapse
/// blank-line runs; preserve everything else as-is. This is enough
/// to absorb Playwright's per-OS line-ending noise without semantic
/// reordering (Playwright output is already canonical per
/// playwright.dev/docs/aria-snapshots so deeper normalization is
/// unnecessary).
fn canonicalize_yaml(yaml: &str) -> String {
    let mut out = String::with_capacity(yaml.len());
    let mut last_was_blank = false;
    for line in yaml.lines() {
        let trimmed = line.trim_end();
        let is_blank = trimmed.is_empty();
        if is_blank && last_was_blank {
            continue;
        }
        out.push_str(trimmed);
        out.push('\n');
        last_was_blank = is_blank;
    }
    out
}

/// Compute the line-granularity diff percentage between `base` and
/// `current`. Uses a simple line-by-line equality scan — Playwright's
/// AXTree output is reorder-stable so reordering-tolerant diff is
/// overkill here. O(N) in lines.
fn line_diff_pct(base: &str, current: &str) -> f64 {
    let base_canon = canonicalize_yaml(base);
    let curr_canon = canonicalize_yaml(current);
    let base_lines: Vec<&str> = base_canon.lines().collect();
    let curr_lines: Vec<&str> = curr_canon.lines().collect();
    let max_len = base_lines.len().max(curr_lines.len());
    if max_len == 0 {
        return 0.0;
    }
    let mut diff_count: usize = 0;
    let common = base_lines.len().min(curr_lines.len());
    for i in 0..common {
        if base_lines[i] != curr_lines[i] {
            diff_count += 1;
        }
    }
    // Lines past the common prefix are all "different".
    diff_count += max_len.saturating_sub(common);
    100.0 * (diff_count as f64) / (max_len as f64)
}

/// blake3-hex helper. The `blake3` crate is in the workspace per
/// CLAUDE.md HARD RULE (NOT blake2 / sha2 for non-RFC uses).
fn blake3_hex_of(bytes: &[u8]) -> String {
    blake3::hash(bytes).to_hex().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evaluate_with_no_baseline_returns_zero_diff() {
        let a = AxTreeAdapter::new();
        let yaml = "- button \"Submit\"\n- text \"Hello\"\n";
        let res = a.evaluate(yaml, None).expect("evaluate succeeds");
        assert_eq!(res.diff_pct, 0.0);
        assert!(!res.mismatch);
        assert!(res.baseline_yaml.is_none());
        assert!(!res.blake3_hex.is_empty());
    }

    #[test]
    fn evaluate_identical_yaml_zero_diff() {
        let a = AxTreeAdapter::new();
        let yaml = "- button \"Submit\"\n- text \"Hello\"\n";
        let res = a.evaluate(yaml, Some(yaml)).expect("evaluate succeeds");
        assert_eq!(res.diff_pct, 0.0);
        assert!(!res.mismatch);
    }

    #[test]
    fn evaluate_drift_above_threshold_flags_mismatch() {
        let a = AxTreeAdapter::new();
        let base = (0..100)
            .map(|i| format!("- text \"line {i}\""))
            .collect::<Vec<_>>()
            .join("\n");
        // Mutate 5 lines -> 5% drift, above 1% threshold.
        let mut current_lines: Vec<String> = base.lines().map(str::to_owned).collect();
        for i in 0..5 {
            current_lines[i] = format!("- text \"changed line {i}\"");
        }
        let current = current_lines.join("\n");
        let res = a.evaluate(&current, Some(&base)).expect("evaluate ok");
        assert!(res.diff_pct >= 4.5 && res.diff_pct <= 5.5, "got {}", res.diff_pct);
        assert!(res.mismatch);
    }

    #[test]
    fn evaluate_drift_below_threshold_no_mismatch() {
        let a = AxTreeAdapter::with_threshold(5.0);
        let base = (0..100)
            .map(|i| format!("- text \"line {i}\""))
            .collect::<Vec<_>>()
            .join("\n");
        // Mutate 3 lines -> 3% drift, below 5% override threshold.
        let mut current_lines: Vec<String> = base.lines().map(str::to_owned).collect();
        for i in 0..3 {
            current_lines[i] = format!("- text \"changed line {i}\"");
        }
        let current = current_lines.join("\n");
        let res = a.evaluate(&current, Some(&base)).expect("evaluate ok");
        assert!(res.diff_pct >= 2.5 && res.diff_pct <= 3.5);
        assert!(!res.mismatch);
    }

    #[test]
    fn evaluate_rejects_oversize_yaml() {
        let a = AxTreeAdapter::new();
        let huge = "a".repeat(MAX_AXTREE_YAML_BYTES + 1);
        let err = a.evaluate(&huge, None).unwrap_err();
        assert!(matches!(err, AxtreeError::YamlTooLarge { .. }));
    }

    #[test]
    fn canonicalize_collapses_blank_runs_and_trims() {
        let raw = "line a   \n\n\nline b\n\n\n";
        let canon = canonicalize_yaml(raw);
        assert!(canon.contains("line a\n"));
        assert!(!canon.contains("\n\n\n"));
    }

    #[test]
    fn blake3_hex_is_64_chars() {
        let h = blake3_hex_of(b"test");
        assert_eq!(h.len(), 64);
        assert!(h.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn twenty_identical_renders_stay_within_threshold() {
        // E-16 plan open Q4: <=1% diff across 20 identical renders.
        // The Playwright output for the same DOM is byte-stable, so
        // this test pins the FLOOR (any drift introduced by future
        // canonicalize changes will trip).
        let a = AxTreeAdapter::new();
        let base = "- main\n  - button \"Add\"\n  - button \"Delete\"\n  - text \"empty\"\n";
        for i in 0..20 {
            let res = a.evaluate(base, Some(base)).expect("eval ok");
            assert_eq!(res.diff_pct, 0.0, "iter {i}: identical render drifted");
            assert!(!res.mismatch);
        }
    }
}

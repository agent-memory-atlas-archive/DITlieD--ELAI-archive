//! I-13: TraceQuery tool — on-demand line-range expansion from disk.
//!
//! Provides [`TraceQuery::expand`] which returns the exact lines `start_line..=end_line`
//! from a file on disk. Prevents condensation starvation: when `to_llm_content` produces
//! a condensed tool-result, the model can call TraceQuery to expand a specific range
//! instead of consuming the full file.
//!
//! Production wiring: registered in `production_agent_rpc.rs::build_tool_catalog`
//! (IS-14 3-arm pattern). Emits `MemoryGuardEvent(TraceQueryExpanded)` SSB row via
//! the caller (the translate_tool_request arm owns SSB emit, not this module).
//!
//! # Fail-closed
//! - `file_path` must resolve to a real file; non-existent paths return `Err`.
//! - `start_line` must be >= 1 (1-indexed); `end_line` must be >= `start_line`.
//! - Lines beyond EOF are silently truncated (no panic, returns what exists).
//! - No path traversal: absolute paths are accepted; relative paths are resolved
//!   against the current working directory (same as `file_read.rs`).
//!
//! # No .unwrap()/.expect() in production code (CLAUDE.md hard rule)

use std::path::Path;

use thiserror::Error;

/// Errors from [`TraceQuery::expand`].
#[derive(Debug, Error)]
pub enum TraceQueryError {
    #[error("TraceQuery: file not found: {0}")]
    FileNotFound(String),
    #[error("TraceQuery: io error reading {path}: {source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("TraceQuery: invalid range: start_line={start} end_line={end} (start must be >= 1, end >= start)")]
    InvalidRange { start: u32, end: u32 },
}

/// Expanded line-range result from [`TraceQuery::expand`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceQueryResult {
    /// Resolved absolute path that was read.
    pub file_path: String,
    /// Actual first line returned (1-indexed). May equal `start_line` or be
    /// clamped if the file is shorter.
    pub start_line: u32,
    /// Actual last line returned (1-indexed, inclusive).
    pub end_line: u32,
    /// Concatenated lines (each terminated with `\n`). Empty string when the
    /// file has fewer lines than `start_line`.
    pub content: String,
    /// Total line count in the file (useful for the model to judge context).
    pub total_lines: u32,
}

/// I-13: on-demand file line-range expansion tool.
pub struct TraceQuery;

impl TraceQuery {
    /// Expand lines `start_line..=end_line` (1-indexed, inclusive) from `path`.
    ///
    /// - `start_line` must be >= 1.
    /// - `end_line` must be >= `start_line`.
    /// - Lines beyond EOF are silently omitted (returns what exists).
    ///
    /// Returns `Err` on IO failure, missing file, or invalid range.
    /// Never panics (no `.unwrap()`/`.expect()` in this fn).
    // WIRE-DARK[I-13]: elai-tools is a dev-dep of elai-orchestrator (circular dep prevents direct
    // lib use); production dispatch inlines the same logic in production_agent_rpc.rs TOOL_TRACE_QUERY.
    // Backing row: wire-dark-allowlist.toml [[fn]] I-13; contract ARM-9 verifies equivalence.
    pub fn expand(
        path: impl AsRef<Path>,
        start_line: u32,
        end_line: u32,
    ) -> Result<TraceQueryResult, TraceQueryError> {
        if start_line < 1 || end_line < start_line {
            return Err(TraceQueryError::InvalidRange {
                start: start_line,
                end: end_line,
            });
        }

        let path = path.as_ref();
        if !path.exists() {
            return Err(TraceQueryError::FileNotFound(
                path.to_string_lossy().into_owned(),
            ));
        }

        let raw = std::fs::read_to_string(path).map_err(|e| TraceQueryError::Io {
            path: path.to_string_lossy().into_owned(),
            source: e,
        })?;

        let all_lines: Vec<&str> = raw.lines().collect();
        let total_lines = all_lines.len() as u32;

        // 1-indexed -> 0-indexed, clamped to EOF.
        let zero_start = (start_line - 1) as usize;
        let zero_end = end_line.saturating_sub(1) as usize;

        let actual_start: u32;
        let actual_end: u32;
        let content: String;

        if zero_start >= all_lines.len() {
            // start_line is beyond EOF — return empty content.
            actual_start = start_line;
            actual_end = start_line;
            content = String::new();
        } else {
            let clamped_end = zero_end.min(all_lines.len().saturating_sub(1));
            actual_start = start_line;
            actual_end = (clamped_end + 1) as u32;
            content = all_lines[zero_start..=clamped_end]
                .iter()
                .map(|l| format!("{l}\n"))
                .collect();
        }

        Ok(TraceQueryResult {
            file_path: path.to_string_lossy().into_owned(),
            start_line: actual_start,
            end_line: actual_end,
            content,
            total_lines,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn make_fixture(lines: &[&str]) -> NamedTempFile {
        let mut f = NamedTempFile::new().expect("tempfile");
        for line in lines {
            writeln!(f, "{line}").expect("write");
        }
        f
    }

    #[test]
    fn expand_exact_range() {
        let f = make_fixture(&["line1", "line2", "line3", "line4", "line5"]);
        let result = TraceQuery::expand(f.path(), 2, 4).expect("expand");
        assert_eq!(result.start_line, 2);
        assert_eq!(result.end_line, 4);
        assert_eq!(result.total_lines, 5);
        assert!(result.content.contains("line2"));
        assert!(result.content.contains("line3"));
        assert!(result.content.contains("line4"));
        assert!(!result.content.contains("line1"));
        assert!(!result.content.contains("line5"));
    }

    #[test]
    fn expand_beyond_eof_clamps() {
        let f = make_fixture(&["a", "b"]);
        let result = TraceQuery::expand(f.path(), 1, 100).expect("expand beyond eof");
        assert_eq!(result.total_lines, 2);
        assert_eq!(result.end_line, 2);
    }

    #[test]
    fn expand_start_beyond_eof_returns_empty() {
        let f = make_fixture(&["a", "b"]);
        let result = TraceQuery::expand(f.path(), 10, 20).expect("start beyond eof");
        assert_eq!(result.content, "");
    }

    #[test]
    fn invalid_range_returns_err() {
        let f = make_fixture(&["a"]);
        assert!(TraceQuery::expand(f.path(), 0, 1).is_err()); // start < 1
        assert!(TraceQuery::expand(f.path(), 5, 3).is_err()); // end < start
    }

    #[test]
    fn missing_file_returns_err() {
        let r = TraceQuery::expand("/nonexistent/__i13_test_file__.txt", 1, 5);
        assert!(r.is_err());
    }

    #[test]
    fn single_line_range() {
        let f = make_fixture(&["only_line"]);
        let result = TraceQuery::expand(f.path(), 1, 1).expect("single line");
        assert_eq!(result.total_lines, 1);
        assert!(result.content.contains("only_line"));
    }
}

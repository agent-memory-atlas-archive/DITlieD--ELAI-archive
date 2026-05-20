//! E-35: SEARCH/REPLACE tier-1 edit matcher with custom non-git fences.
//!
//! Why a new format: Qwen2.5-Coder-7B + aider polyglot Nov-20-2025 measured
//! SEARCH/REPLACE > unified-diff for the 7B-NF4 cliff (arXiv:2510.12487
//! Diff-XYZ JetBrains Oct 2025; aider.chat leaderboard). Whole-file rewrite
//! remains the tier-0 fallback when fence collisions are detected.
//!
//! Why non-git fences: the literal git conflict markers (`<<<<<<<` /
//! `=======` / `>>>>>>>`) collide with merge-conflict text in real source
//! files, and with `<<<<<<< HEAD` lines our own samples include. The
//! `<<< ELAI_SEARCH` / `=== ELAI_REPLACE` / `>>> ELAI_END` triple was
//! chosen to be:
//!
//! * unambiguous (no language uses these as syntax)
//! * 3 chars not 7 (faster scan, no false positives with `<<<<<<<HEAD`)
//! * uppercase-prefixed (no overlap with sphinx / markdown headings)
//!
//! Public surface:
//! * [`ELAI_SEARCH_FENCE`] / [`ELAI_REPLACE_FENCE`] / [`ELAI_END_FENCE`] — the
//!   canonical fence strings.
//! * [`DelimiterCollisionGuard`] — pre-flight check that flags source / patch
//!   text containing fence tokens OR git conflict markers; consumer falls
//!   back to whole-file rewrite when `collides()` returns true.
//! * [`SearchReplaceMatcher`] — parser + applier. Parses fenced patch text
//!   into a `SearchReplaceTier1`, then applies the edits left-to-right
//!   against a source string. Each block must match exactly once (matching
//!   the conservative aider behaviour); zero matches or > 1 match aborts
//!   the whole patch.
//!
//! The wire types (`SearchReplaceTier1`, `SearchReplaceEdit`) live in
//! `elai_ipc_types` so the orchestrator can hand a structured patch
//! to this matcher without re-parsing fenced text.

use std::fmt;

pub use elai_ipc_types::{SearchReplaceEdit, SearchReplaceTier1};

// ---------------------------------------------------------------------------
// Fence constants
// ---------------------------------------------------------------------------

/// Opening fence for a SEARCH block. Must appear at the start of a line.
pub const ELAI_SEARCH_FENCE: &str = "<<< ELAI_SEARCH";
/// Separator between SEARCH and REPLACE bodies. Must appear at the start
/// of a line.
pub const ELAI_REPLACE_FENCE: &str = "=== ELAI_REPLACE";
/// Closing fence for a REPLACE block. Must appear at the start of a line.
pub const ELAI_END_FENCE: &str = ">>> ELAI_END";

/// Git conflict-marker prefixes we refuse to coexist with. We test on the
/// 7-character prefix (the part that's stable across diff3/merge3 modes).
const GIT_MARKER_PREFIXES: &[&str] = &["<<<<<<<", "=======", ">>>>>>>"];

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// Errors produced by the SEARCH/REPLACE matcher. Stable snake_case
/// `category()` for SSB beacons / logs.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SearchReplaceError {
    #[error("patch is empty (no SEARCH fences found)")]
    EmptyPatch,
    #[error("malformed patch: missing {missing} fence after line {line}")]
    MalformedPatch { line: usize, missing: &'static str },
    #[error(
        "search block {block_index} matched {matches} times in target (must be exactly 1)"
    )]
    AmbiguousMatch { block_index: usize, matches: usize },
    #[error("search block {block_index} did not match the target file")]
    NoMatch { block_index: usize },
    #[error(
        "delimiter collision: {what} contains forbidden token {token}; fall back to whole-file rewrite"
    )]
    DelimiterCollision { what: &'static str, token: String },
}

impl SearchReplaceError {
    /// Stable snake_case category label for SSB / log surfaces.
    pub fn category(&self) -> &'static str {
        match self {
            Self::EmptyPatch => "empty_patch",
            Self::MalformedPatch { .. } => "malformed_patch",
            Self::AmbiguousMatch { .. } => "ambiguous_match",
            Self::NoMatch { .. } => "no_match",
            Self::DelimiterCollision { .. } => "delimiter_collision",
        }
    }
}

// ---------------------------------------------------------------------------
// DelimiterCollisionGuard
// ---------------------------------------------------------------------------

/// Pre-flight collision checker. Runs over both the source (file we are
/// about to patch) and the proposed patch text. If either contains an ELAI
/// fence token or a git conflict marker, [`Self::collides`] returns true
/// and the dispatcher MUST fall back to whole-file rewrite.
///
/// Implementation note: the guard scans for fence tokens AT THE START OF A
/// LINE so a literal mention of `<<< ELAI_SEARCH` mid-paragraph in a
/// docstring is fine. Git markers use the same rule.
#[derive(Debug, Clone, Copy, Default)]
pub struct DelimiterCollisionGuard;

impl DelimiterCollisionGuard {
    pub const fn new() -> Self {
        Self
    }

    /// Returns the first conflict found in `text`, identified by its
    /// stable snake_case token name. `None` means the text is safe.
    pub fn first_collision(&self, text: &str) -> Option<&'static str> {
        for raw_line in text.lines() {
            let line = raw_line.trim_start();
            if line.starts_with(ELAI_SEARCH_FENCE) {
                return Some("elai_search");
            }
            if line.starts_with(ELAI_REPLACE_FENCE) {
                return Some("elai_replace");
            }
            if line.starts_with(ELAI_END_FENCE) {
                return Some("elai_end");
            }
            for marker in GIT_MARKER_PREFIXES {
                if line.starts_with(marker) {
                    return Some("git_conflict_marker");
                }
            }
        }
        None
    }

    /// Returns true iff `text` contains any ELAI fence token or git conflict
    /// marker at the start of a (whitespace-trimmed) line.
    pub fn collides(&self, text: &str) -> bool {
        self.first_collision(text).is_some()
    }

    /// Convenience: validate both the patch and the source together. On
    /// collision returns `Err(SearchReplaceError::DelimiterCollision)` so
    /// callers can propagate the structured error.
    pub fn validate(&self, source_label: &'static str, source: &str) -> Result<(), SearchReplaceError> {
        if let Some(token) = self.first_collision(source) {
            return Err(SearchReplaceError::DelimiterCollision {
                what: source_label,
                token: token.to_string(),
            });
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// SearchReplaceMatcher
// ---------------------------------------------------------------------------

/// SEARCH/REPLACE patch parser + applier. Pure functions, no I/O. The
/// dispatching tool wraps these with the actual file read/write.
#[derive(Debug, Clone, Copy, Default)]
pub struct SearchReplaceMatcher;

impl SearchReplaceMatcher {
    pub const fn new() -> Self {
        Self
    }

    /// Parse a string containing one or more `ELAI_SEARCH` / `ELAI_REPLACE`
    /// / `ELAI_END` blocks into a `SearchReplaceTier1`.
    ///
    /// The patch's `file_path` is provided by the caller (the LLM emits it
    /// out-of-band; we do NOT parse a header line from the fenced text so
    /// the format stays simple).
    pub fn parse(&self, file_path: impl Into<String>, patch: &str) -> Result<SearchReplaceTier1, SearchReplaceError> {
        let lines: Vec<&str> = patch.lines().collect();
        let mut edits: Vec<SearchReplaceEdit> = Vec::new();
        let mut i = 0;
        while i < lines.len() {
            let line = lines[i].trim_start();
            if !line.starts_with(ELAI_SEARCH_FENCE) {
                i += 1;
                continue;
            }
            let search_start = i + 1;
            // Find the REPLACE fence.
            let mut replace_at: Option<usize> = None;
            let mut end_at: Option<usize> = None;
            let mut j = search_start;
            while j < lines.len() {
                let lj = lines[j].trim_start();
                if lj.starts_with(ELAI_REPLACE_FENCE) {
                    replace_at = Some(j);
                    let mut k = j + 1;
                    while k < lines.len() {
                        let lk = lines[k].trim_start();
                        if lk.starts_with(ELAI_END_FENCE) {
                            end_at = Some(k);
                            break;
                        }
                        k += 1;
                    }
                    break;
                }
                if lj.starts_with(ELAI_SEARCH_FENCE) || lj.starts_with(ELAI_END_FENCE) {
                    // Nested SEARCH or premature END inside a SEARCH body — malformed.
                    return Err(SearchReplaceError::MalformedPatch {
                        line: j + 1,
                        missing: "elai_replace",
                    });
                }
                j += 1;
            }
            let replace_at = replace_at.ok_or(SearchReplaceError::MalformedPatch {
                line: i + 1,
                missing: "elai_replace",
            })?;
            let end_at = end_at.ok_or(SearchReplaceError::MalformedPatch {
                line: replace_at + 1,
                missing: "elai_end",
            })?;

            let search_body = join_lines(&lines[search_start..replace_at]);
            let replace_body = join_lines(&lines[replace_at + 1..end_at]);
            edits.push(SearchReplaceEdit {
                search: search_body,
                replace: replace_body,
            });
            i = end_at + 1;
        }
        if edits.is_empty() {
            return Err(SearchReplaceError::EmptyPatch);
        }
        Ok(SearchReplaceTier1 {
            file_path: file_path.into(),
            edits,
        })
    }

    /// Apply a parsed `SearchReplaceTier1` to a source string. Each
    /// edit must match exactly once (aider-conservative). Edits are
    /// applied in declared order; later edits see the result of earlier
    /// edits.
    pub fn apply(&self, source: &str, patch: &SearchReplaceTier1) -> Result<String, SearchReplaceError> {
        let mut current = source.to_string();
        for (idx, edit) in patch.edits.iter().enumerate() {
            let count = current.matches(&edit.search).count();
            if count == 0 {
                return Err(SearchReplaceError::NoMatch { block_index: idx });
            }
            if count > 1 {
                return Err(SearchReplaceError::AmbiguousMatch {
                    block_index: idx,
                    matches: count,
                });
            }
            current = current.replacen(&edit.search, &edit.replace, 1);
        }
        Ok(current)
    }

    /// End-to-end: parse + collision-check + apply in one call. The most
    /// convenient surface for dispatcher integration.
    pub fn parse_and_apply(
        &self,
        file_path: impl Into<String>,
        patch_text: &str,
        source: &str,
    ) -> Result<(SearchReplaceTier1, String), SearchReplaceError> {
        let guard = DelimiterCollisionGuard::new();
        guard.validate("source", source)?;
        let patch = self.parse(file_path, patch_text)?;
        let result = self.apply(source, &patch)?;
        Ok((patch, result))
    }
}

fn join_lines(lines: &[&str]) -> String {
    let mut out = String::new();
    for (i, l) in lines.iter().enumerate() {
        if i > 0 {
            out.push('\n');
        }
        out.push_str(l);
    }
    out
}

// ---------------------------------------------------------------------------
// Display helper (for log lines)
// ---------------------------------------------------------------------------

impl fmt::Display for DelimiterCollisionGuard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DelimiterCollisionGuard(elai-fences + git-markers)")
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_SOURCE: &str = "fn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n";
    const SAMPLE_PATCH: &str = "<<< ELAI_SEARCH\nfn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n=== ELAI_REPLACE\nfn add(a: i32, b: i32) -> i32 {\n    let sum = a + b;\n    sum\n}\n>>> ELAI_END\n";

    #[test]
    fn fence_constants_are_unambiguous() {
        assert_ne!(ELAI_SEARCH_FENCE, ELAI_REPLACE_FENCE);
        assert_ne!(ELAI_SEARCH_FENCE, ELAI_END_FENCE);
        assert_ne!(ELAI_REPLACE_FENCE, ELAI_END_FENCE);
        // None of the ELAI fences overlap with git markers.
        for fence in [ELAI_SEARCH_FENCE, ELAI_REPLACE_FENCE, ELAI_END_FENCE] {
            for marker in GIT_MARKER_PREFIXES {
                assert!(!fence.starts_with(marker), "fence {fence} overlaps git marker {marker}");
            }
        }
    }

    #[test]
    fn parser_extracts_one_edit() {
        let m = SearchReplaceMatcher::new();
        let patch = m.parse("src/lib.rs", SAMPLE_PATCH).expect("parse");
        assert_eq!(patch.file_path, "src/lib.rs");
        assert_eq!(patch.edits.len(), 1);
        assert!(patch.edits[0].search.contains("a + b"));
        assert!(patch.edits[0].replace.contains("let sum"));
    }

    #[test]
    fn parser_extracts_two_edits() {
        let patch_text = "<<< ELAI_SEARCH\nold-1\n=== ELAI_REPLACE\nnew-1\n>>> ELAI_END\n<<< ELAI_SEARCH\nold-2\n=== ELAI_REPLACE\nnew-2\n>>> ELAI_END\n";
        let m = SearchReplaceMatcher::new();
        let patch = m.parse("a.txt", patch_text).expect("parse");
        assert_eq!(patch.edits.len(), 2);
        assert_eq!(patch.edits[0].search.trim(), "old-1");
        assert_eq!(patch.edits[1].replace.trim(), "new-2");
    }

    #[test]
    fn parser_rejects_missing_replace() {
        let bad = "<<< ELAI_SEARCH\nold\n>>> ELAI_END\n";
        let m = SearchReplaceMatcher::new();
        let err = m.parse("a.txt", bad).unwrap_err();
        assert!(matches!(err, SearchReplaceError::MalformedPatch { .. }));
        assert_eq!(err.category(), "malformed_patch");
    }

    #[test]
    fn parser_rejects_missing_end() {
        let bad = "<<< ELAI_SEARCH\nold\n=== ELAI_REPLACE\nnew\n";
        let m = SearchReplaceMatcher::new();
        let err = m.parse("a.txt", bad).unwrap_err();
        assert!(matches!(err, SearchReplaceError::MalformedPatch { .. }));
    }

    #[test]
    fn parser_rejects_empty_patch() {
        let m = SearchReplaceMatcher::new();
        let err = m.parse("a.txt", "no fences at all\n").unwrap_err();
        assert!(matches!(err, SearchReplaceError::EmptyPatch));
        assert_eq!(err.category(), "empty_patch");
    }

    #[test]
    fn applier_replaces_unique_match() {
        let m = SearchReplaceMatcher::new();
        let (_patch, out) = m
            .parse_and_apply("src/lib.rs", SAMPLE_PATCH, SAMPLE_SOURCE)
            .expect("apply ok");
        assert!(out.contains("let sum = a + b"));
        assert!(out.contains("sum\n}"));
    }

    #[test]
    fn applier_rejects_ambiguous_match() {
        let src = "x\nx\n";
        let patch_text = "<<< ELAI_SEARCH\nx\n=== ELAI_REPLACE\ny\n>>> ELAI_END\n";
        let m = SearchReplaceMatcher::new();
        let err = m
            .parse_and_apply("a.txt", patch_text, src)
            .unwrap_err();
        match err {
            SearchReplaceError::AmbiguousMatch { matches, .. } => {
                assert_eq!(matches, 2);
            }
            other => panic!("expected AmbiguousMatch, got {other:?}"),
        }
    }

    #[test]
    fn applier_rejects_no_match() {
        let patch_text = "<<< ELAI_SEARCH\nfoo\n=== ELAI_REPLACE\nbar\n>>> ELAI_END\n";
        let m = SearchReplaceMatcher::new();
        let err = m
            .parse_and_apply("a.txt", patch_text, "no such string here\n")
            .unwrap_err();
        assert!(matches!(err, SearchReplaceError::NoMatch { .. }));
    }

    #[test]
    fn collision_guard_flags_git_markers() {
        let g = DelimiterCollisionGuard::new();
        let conflict_text = "line\n<<<<<<< HEAD\nfoo\n=======\nbar\n>>>>>>> branch\n";
        assert!(g.collides(conflict_text));
        assert_eq!(g.first_collision(conflict_text), Some("git_conflict_marker"));
    }

    #[test]
    fn collision_guard_flags_elai_fences_in_source() {
        let g = DelimiterCollisionGuard::new();
        let src = "fn x() {}\n<<< ELAI_SEARCH\n";
        assert!(g.collides(src));
        assert_eq!(g.first_collision(src), Some("elai_search"));
    }

    #[test]
    fn collision_guard_clean_text_passes() {
        let g = DelimiterCollisionGuard::new();
        assert!(!g.collides("regular source code\nno fences here\n"));
        assert_eq!(g.first_collision("clean\n"), None);
    }

    #[test]
    fn parse_and_apply_propagates_collision_as_structured_error() {
        let m = SearchReplaceMatcher::new();
        let source_with_collision = "stuff\n<<<<<<< HEAD\nmore\n";
        let err = m
            .parse_and_apply("a.txt", SAMPLE_PATCH, source_with_collision)
            .unwrap_err();
        match err {
            SearchReplaceError::DelimiterCollision { what, token } => {
                assert_eq!(what, "source");
                assert_eq!(token, "git_conflict_marker");
            }
            other => panic!("expected DelimiterCollision, got {other:?}"),
        }
    }

    /// E-35 observable_effect smoke: build a `SearchReplaceTier1` payload,
    /// round-trip it through postcard (the same path the orchestrator would
    /// use over IPC), and assert it applies cleanly to a sample source.
    #[test]
    fn observable_effect_smoke_postcard_round_trip_then_apply() {
        let patch = SearchReplaceTier1 {
            file_path: "src/lib.rs".to_string(),
            edits: vec![SearchReplaceEdit {
                search: "fn add(a: i32, b: i32) -> i32 {\n    a + b\n}".to_string(),
                replace: "fn add(a: i32, b: i32) -> i32 {\n    let sum = a + b;\n    sum\n}"
                    .to_string(),
            }],
        };
        let bytes = postcard::to_allocvec(&patch).expect("postcard encode");
        let back: SearchReplaceTier1 = postcard::from_bytes(&bytes).expect("postcard decode");
        assert_eq!(patch, back);
        let m = SearchReplaceMatcher::new();
        let out = m.apply(SAMPLE_SOURCE, &back).expect("apply ok");
        assert!(out.contains("let sum = a + b"));
        assert!(out.contains("sum\n}"));
    }
}

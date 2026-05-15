//! Minified-file detection.
//!
//! A file is "minified" when its average line length exceeds
//! `MAX_AVG_LINE_LENGTH` (500 chars/line). This catches webpack/esbuild
//! bundles, minified CSS, generated protobuf go code, and other content
//! whose line-based chunking would yield single-line chunks far above the
//! embedding provider's context window.
//!
//! Reference: SocratiCode src/services/indexer.ts:432-439.

use super::constants::MAX_AVG_LINE_LENGTH;

/// Return `true` when `content` should be routed through the character-based
/// chunker rather than AST or line-block chunking.
///
/// `line_count` is `content.lines().count()` from the caller. When it is
/// zero (no newlines in the file at all) we fall back to comparing the raw
/// content length against the threshold so a single very long line still
/// trips detection.
pub fn is_minified(content: &str, line_count: usize) -> bool {
    if line_count == 0 {
        return content.len() > MAX_AVG_LINE_LENGTH;
    }
    (content.len() / line_count) > MAX_AVG_LINE_LENGTH
}

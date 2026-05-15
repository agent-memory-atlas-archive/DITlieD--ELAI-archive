//! Chunking thresholds — values mirror SocratiCode src/constants.ts:115,128.
//!
//! Rationale (from SocratiCode):
//!   * `MAX_AVG_LINE_LENGTH = 500` — files whose average line length exceeds
//!     this are treated as minified/bundled and routed to character-based
//!     chunking. 500 chars/line is well above any reasonable hand-written
//!     code line and well below the smallest minified bundle.
//!   * `MAX_CHUNK_CHARS = 2000` — universal hard cap on chunk content
//!     length, applied as the last step of every dispatch. Kept conservative
//!     so the embedding provider's own pre-truncation acts as defence-in-depth.

/// Threshold (chars/line) above which a file is treated as minified.
/// Source: SocratiCode src/constants.ts:115.
pub const MAX_AVG_LINE_LENGTH: usize = 500;

/// Universal upper bound on chunk content length, in bytes.
/// Source: SocratiCode src/constants.ts:128.
pub const MAX_CHUNK_CHARS: usize = 2000;

//! B-32 Step A — provider-boundary defensive cap tests (D4 / GAP-1).
//!
//! These tests cover the explicit truncation safety net at the
//! vector::indexer::index_file embed call site (vector/indexer.rs:~146-171).
//! The warn-emitting branch fires only on a regression — i.e. when an
//! oversized chunk slips past `apply_char_cap` at the dispatch layer.
//!
//! LIMITATION: the production warn site lives inside `VectorIndexer::index_file`
//! which only accepts chunks produced by `chunk_file(path)`. `chunk_file`
//! already applies `apply_char_cap` as its LAST step (per B-32 D1), so we
//! cannot feed an oversized chunk into the warn branch from a black-box
//! test without a production-code injection seam (e.g. a `#[cfg(test)]`
//! helper that bypasses dispatch). Adding such a seam is out of scope for
//! B-32 Step A. Instead these tests:
//!   1. exercise the underlying truncation primitive (`apply_char_cap` +
//!      `floor_char_boundary`) directly with the exact inputs the warn
//!      branch operates on,
//!   2. demonstrate a tracing capture harness (std-only via
//!      tracing-subscriber registry — no new deps) that future tests can
//!      reuse once the production seam exists,
//!   3. assert the cross-layer constant (`MAX_CHUNK_CHARS`) is stable.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use elai_knowledge::chunking::cap::{apply_char_cap, floor_char_boundary};
use elai_knowledge::chunking::constants::MAX_CHUNK_CHARS;
use elai_knowledge::bm25::{BM25Chunk, ChunkKind};
use tracing::Level;
use tracing_subscriber::layer::{Context, SubscriberExt};
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::Layer;

/// Std-only capture layer that appends every `tracing::warn!` event message
/// to a shared `String` buffer. Filters by event level so DEBUG/INFO noise
/// from dependencies does not pollute assertions.
struct WarnCaptureLayer {
    buf: Arc<Mutex<String>>,
}

impl<S> Layer<S> for WarnCaptureLayer
where
    S: tracing::Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_event(&self, event: &tracing::Event<'_>, _ctx: Context<'_, S>) {
        if event.metadata().level() != &Level::WARN {
            return;
        }
        let mut visitor = StringVisitor(String::new());
        event.record(&mut visitor);
        if let Ok(mut buf) = self.buf.lock() {
            buf.push_str(&visitor.0);
            buf.push('\n');
        }
    }
}

struct StringVisitor(String);
impl tracing::field::Visit for StringVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        use std::fmt::Write as _;
        let _ = write!(self.0, "{}={:?} ", field.name(), value);
    }
    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        use std::fmt::Write as _;
        let _ = write!(self.0, "{}={} ", field.name(), value);
    }
}

#[test]
fn test_provider_boundary_truncates_oversized_chunk() {
    // A chunk that somehow escapes the dispatch-level cap (LineBlock-kind,
    // not File-kind, oversized) must still be capped by the provider-boundary
    // defensive layer. The production site uses the same `floor_char_boundary`
    // + slice primitive that `apply_char_cap` calls — exercising
    // `apply_char_cap` here proves the truncation invariant the warn branch
    // depends on. See LIMITATION at top of file.
    let chunks = vec![BM25Chunk {
        path: PathBuf::from("escaped.rs"),
        content: "x".repeat(MAX_CHUNK_CHARS * 2),
        line_start: 1,
        line_end: 100,
        kind: ChunkKind::LineBlock,
    }];
    let capped = apply_char_cap(chunks);
    assert!(
        capped[0].content.len() <= MAX_CHUNK_CHARS,
        "provider-boundary cap must truncate to MAX_CHUNK_CHARS={MAX_CHUNK_CHARS}"
    );
    // Also verify the production-equivalent inline truncation produces an
    // identical-length result, since the warn branch slices via
    // `floor_char_boundary` directly rather than calling `apply_char_cap`.
    let raw = "x".repeat(MAX_CHUNK_CHARS * 2);
    let end = floor_char_boundary(&raw, MAX_CHUNK_CHARS);
    assert_eq!(end, MAX_CHUNK_CHARS, "ASCII content truncates exactly at the cap");
}

#[test]
fn test_provider_boundary_no_truncation_when_at_cap() {
    // A chunk at exactly MAX_CHUNK_CHARS must NOT be truncated (boundary
    // condition: the warn branch condition is `> MAX_CHUNK_CHARS`, not `>=`).
    let exactly_at_cap = "y".repeat(MAX_CHUNK_CHARS);
    let chunks = vec![BM25Chunk {
        path: PathBuf::from("at_cap.rs"),
        content: exactly_at_cap.clone(),
        line_start: 1,
        line_end: 1,
        kind: ChunkKind::Function,
    }];
    let capped = apply_char_cap(chunks);
    assert_eq!(
        capped[0].content.len(),
        MAX_CHUNK_CHARS,
        "chunk exactly at MAX_CHUNK_CHARS must not be truncated"
    );
    assert_eq!(capped[0].content, exactly_at_cap, "content must be unchanged");
}

#[test]
fn test_provider_boundary_constants_match_dispatch_constants() {
    // Defense-in-depth: the provider-boundary cap MUST reference the same
    // MAX_CHUNK_CHARS constant that the dispatch layer uses, otherwise the
    // two layers can diverge silently. This test asserts the constant is
    // stable and accessible via the canonical path.
    assert_eq!(
        MAX_CHUNK_CHARS, 2000,
        "MAX_CHUNK_CHARS must be 2000 (SocratiCode constants.ts:128)"
    );
}

#[test]
fn test_provider_boundary_floor_char_boundary_used_for_truncation() {
    // Emoji at boundary forces walk-back; assert the truncation does not
    // panic and produces valid UTF-8 (regression for gotchas.md:935).
    let pad = "a".repeat(MAX_CHUNK_CHARS - 2);
    let content = format!("{pad}🚀tail");
    let chunks = vec![BM25Chunk {
        path: PathBuf::from("emoji_escape.min.js"),
        content,
        line_start: 1,
        line_end: 1,
        kind: ChunkKind::Minified,
    }];
    let capped = apply_char_cap(chunks);
    let truncated = &capped[0].content;
    assert!(
        truncated.is_char_boundary(truncated.len()),
        "provider-boundary truncation must produce valid UTF-8"
    );
    assert!(truncated.len() <= MAX_CHUNK_CHARS);
    // Specifically: truncation must walk back from the multi-byte boundary.
    let walked = floor_char_boundary(
        &format!("{}🚀tail", "a".repeat(MAX_CHUNK_CHARS - 2)),
        MAX_CHUNK_CHARS,
    );
    assert!(
        walked <= MAX_CHUNK_CHARS && walked >= MAX_CHUNK_CHARS - 4,
        "floor_char_boundary should walk back by at most 4 bytes (one codepoint)"
    );
}

#[test]
fn test_tracing_capture_harness_records_warn_events() {
    // Harness self-test: prove the std-only WarnCaptureLayer wired through
    // `tracing_subscriber::registry` correctly captures `tracing::warn!`
    // events emitted with the production warn site's structured fields.
    // Once a `#[cfg(test)]` injection seam exists in vector::indexer,
    // future tests can reuse this harness to assert the actual call site
    // emits the regression-warn message.
    let buf: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
    let layer = WarnCaptureLayer { buf: Arc::clone(&buf) };
    let subscriber = tracing_subscriber::registry().with(layer);

    tracing::subscriber::with_default(subscriber, || {
        // Mirror the production warn-event fields from vector/indexer.rs:157-163.
        let fake_path = PathBuf::from("regression.rs");
        let oversized_len = MAX_CHUNK_CHARS * 2;
        tracing::warn!(
            target: "knowledge::vector::indexer",
            path = %fake_path.display(),
            chars = oversized_len,
            cap = MAX_CHUNK_CHARS,
            "MAX_CHUNK_CHARS regression — apply_char_cap missed a chunk; truncating at provider boundary"
        );
    });

    let captured = buf.lock().expect("buf lock").clone();
    assert!(
        captured.contains("MAX_CHUNK_CHARS regression"),
        "harness must capture warn message; got: {captured:?}"
    );
    assert!(
        captured.contains("regression.rs"),
        "harness must capture path field; got: {captured:?}"
    );
    assert!(
        captured.contains(&format!("cap={MAX_CHUNK_CHARS}")),
        "harness must capture cap field; got: {captured:?}"
    );
}

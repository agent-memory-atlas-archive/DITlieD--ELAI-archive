//! E-3 sub-phase 4B — handoff payload compression.
//!
//! When the orchestrator hands a rolling chat history from one
//! `RuntimeAdapter` (e.g. cloud Sonnet) to another (e.g. local Qwen3B
//! sidecar), the receiver typically has a smaller context window and
//! the older history is the least information-dense part of the
//! payload. `compress_handoff_payload` is a deterministic heuristic
//! pass that trims a `Vec<Message>` down to fit a target token budget
//! by:
//!
//!   1. Reserving a recency window (the last N messages verbatim).
//!   2. Summarising older messages by truncating each to a per-message
//!      character ceiling, prefixing a `[truncated <orig_bytes> bytes]`
//!      marker so the receiving model can tell the difference between
//!      a verbatim summary and a real short message.
//!   3. Dropping the oldest messages entirely if the budget is still
//!      exceeded, replacing them with a single `[handoff: <n> prior
//!      messages elided]` sentinel placed at the front of the result.
//!
//! Notes:
//!   * Token counting uses a 4-bytes-per-token heuristic (the
//!     conservative number Anthropic publishes for Claude tokenizers,
//!     reused here to avoid a tokenizer dependency in the hot path).
//!   * No LLM call is made — this is a pure utility per the CLAUDE.md
//!     "no LLM in the validation hot path" pattern.
//!   * Wired-but-no-caller in 4B: the orchestrator's `session_resume`
//!     path will pick this up in a later sub-phase. SSB emission via
//!     [`crate::ssb::SsbPayload::E3RuntimeHandoffEvent`] with
//!     `HandoffPayloadCompressed` is the observable surface.

use elai_providers::runtime::Message;

/// Compressed payload shape returned by [`compress_handoff_payload`].
#[derive(Debug, Clone, PartialEq)]
pub struct CompressedPayload {
    /// Final message list, ready to hand to the next adapter's
    /// `complete` / `stream`.
    pub messages: Vec<Message>,
    /// Total bytes across the original `Vec<Message>` content fields.
    pub original_bytes: usize,
    /// Total bytes across the compressed `messages` content fields.
    pub compressed_bytes: usize,
    /// Estimated original token count (4-bytes-per-token).
    pub original_tokens: usize,
    /// Estimated compressed token count (4-bytes-per-token).
    pub compressed_tokens: usize,
    /// Compression ratio (`original_tokens as f32 / compressed_tokens as f32`).
    /// Saturates at `f32::INFINITY` when the compressed token count is 0;
    /// callers should treat ratio > 100.0 as "near-total drop".
    pub ratio: f32,
}

impl CompressedPayload {
    /// Build a [`CompressedPayload`] from raw counts. Public so the
    /// `session_resume` caller can attach its own pre-computed counters
    /// when it knows them already.
    #[must_use]
    pub fn from_messages(messages: Vec<Message>, original_bytes: usize) -> Self {
        let compressed_bytes: usize = messages.iter().map(|m| m.content.len()).sum();
        let original_tokens = bytes_to_tokens(original_bytes);
        let compressed_tokens = bytes_to_tokens(compressed_bytes);
        let ratio = if compressed_tokens == 0 {
            f32::INFINITY
        } else {
            original_tokens as f32 / compressed_tokens as f32
        };
        Self {
            messages,
            original_bytes,
            compressed_bytes,
            original_tokens,
            compressed_tokens,
            ratio,
        }
    }
}

/// Number of recent messages preserved verbatim. Mirrors the
/// LLMLingua "keep tail" convention; tunable per future sub-phase but
/// hard-coded here to keep the API stable for the 4B wiring.
pub const VERBATIM_TAIL_MESSAGES: usize = 4;

/// Per-message character ceiling applied to older (non-tail) messages
/// when truncation is needed. 800 chars ~= 200 tokens.
pub const SUMMARY_CHAR_CEILING: usize = 800;

/// Compress a rolling chat history to fit within `target_tokens` using
/// the heuristic described in the module docs. Pure function; does not
/// emit SSB. Caller (orchestrator session_resume hook) emits
/// `E3RuntimeHandoffEvent::HandoffPayloadCompressed` with the returned
/// `original_tokens` / `compressed_tokens` / `ratio` triple.
///
/// Behaviour notes:
///   * `target_tokens == 0` returns an empty payload with a single
///     "all history elided" sentinel so the caller can still issue a
///     1-token canary on the receiving adapter.
///   * If `messages.len() <= VERBATIM_TAIL_MESSAGES` and the original
///     fits, the input is returned unchanged (no truncation marker).
#[must_use]
pub fn compress_handoff_payload(
    messages: &[Message],
    target_tokens: usize,
) -> CompressedPayload {
    let original_bytes: usize = messages.iter().map(|m| m.content.len()).sum();
    let original_tokens = bytes_to_tokens(original_bytes);

    // Cheap path: whole history fits already.
    if original_tokens <= target_tokens {
        return CompressedPayload::from_messages(messages.to_vec(), original_bytes);
    }

    // Hard floor: 0-budget request returns a single placeholder so the
    // receiving adapter has SOMETHING to anchor on but pays zero
    // history cost.
    if target_tokens == 0 {
        let placeholder = Message {
            role: elai_providers::runtime::MessageRole::User,
            content: format!(
                "[handoff: {} prior messages elided (target_tokens=0)]",
                messages.len()
            ),
        };
        return CompressedPayload::from_messages(vec![placeholder], original_bytes);
    }

    // Split into tail (preserved verbatim) and head (candidate for
    // truncation / drop).
    let tail_start = messages.len().saturating_sub(VERBATIM_TAIL_MESSAGES);
    let (head, tail) = messages.split_at(tail_start);

    // Step 1 — truncate head messages to the SUMMARY_CHAR_CEILING.
    let mut compressed: Vec<Message> = Vec::with_capacity(messages.len());
    for msg in head {
        if msg.content.len() <= SUMMARY_CHAR_CEILING {
            compressed.push(msg.clone());
        } else {
            let orig_len = msg.content.len();
            // Truncate on a safe UTF-8 boundary — find_char_boundary
            // is preferred when available, fall back to a conservative
            // 80-percent boundary scan otherwise.
            let truncate_at = floor_char_boundary(&msg.content, SUMMARY_CHAR_CEILING);
            let truncated_body = &msg.content[..truncate_at];
            let new_content = format!(
                "{truncated_body}\n[truncated {} bytes from original {} bytes]",
                orig_len - truncate_at,
                orig_len
            );
            compressed.push(Message {
                role: msg.role,
                content: new_content,
            });
        }
    }
    compressed.extend(tail.iter().cloned());

    // Step 2 — if still over budget, drop oldest messages until we
    // fit, recording the drop count in a sentinel at the head.
    let mut compressed_bytes: usize = compressed.iter().map(|m| m.content.len()).sum();
    let mut compressed_tokens = bytes_to_tokens(compressed_bytes);
    let mut dropped = 0usize;
    while compressed_tokens > target_tokens && compressed.len() > VERBATIM_TAIL_MESSAGES {
        // Always drop from the head — tail is sacrosanct.
        let removed = compressed.remove(0);
        compressed_bytes = compressed_bytes.saturating_sub(removed.content.len());
        compressed_tokens = bytes_to_tokens(compressed_bytes);
        dropped += 1;
    }
    if dropped > 0 {
        let sentinel = Message {
            role: elai_providers::runtime::MessageRole::User,
            content: format!("[handoff: {dropped} prior messages elided]"),
        };
        compressed.insert(0, sentinel);
    }

    CompressedPayload::from_messages(compressed, original_bytes)
}

/// Conservative byte-to-token estimate. The Anthropic-published
/// rule-of-thumb is ~3.5-4 chars per token for English. We round to 4
/// so the compression target is reached safely (slight over-shoot is
/// preferable to busting the receiver's context window).
#[must_use]
pub fn bytes_to_tokens(bytes: usize) -> usize {
    bytes.div_ceil(4)
}

/// Find the highest char-boundary `<= idx` so a slice up to that point
/// is valid UTF-8. Linear scan, but the byte distance covered is at
/// most 3 (UTF-8 code points are <=4 bytes), so this is O(1) practical.
fn floor_char_boundary(s: &str, idx: usize) -> usize {
    let mut i = idx.min(s.len());
    while i > 0 && !s.is_char_boundary(i) {
        i -= 1;
    }
    i
}

#[cfg(test)]
mod tests {
    use super::*;
    use elai_providers::runtime::{Message, MessageRole};

    fn msg(role: MessageRole, content: &str) -> Message {
        Message {
            role,
            content: content.to_owned(),
        }
    }

    #[test]
    fn under_budget_passes_through_unchanged() {
        let messages = vec![
            msg(MessageRole::User, "hello"),
            msg(MessageRole::Assistant, "hi"),
        ];
        let out = compress_handoff_payload(&messages, 1000);
        assert_eq!(out.messages, messages);
        assert!(out.ratio.is_finite());
        assert!(out.ratio <= 1.0_f32 + f32::EPSILON);
    }

    #[test]
    fn zero_budget_returns_single_placeholder() {
        let messages = vec![
            msg(MessageRole::User, "one"),
            msg(MessageRole::Assistant, "two"),
        ];
        let out = compress_handoff_payload(&messages, 0);
        assert_eq!(out.messages.len(), 1);
        assert!(out.messages[0].content.starts_with("[handoff: 2 prior"));
    }

    #[test]
    fn large_head_gets_truncated() {
        let big = "x".repeat(SUMMARY_CHAR_CEILING * 4);
        let mut messages = vec![msg(MessageRole::User, &big)];
        for i in 0..VERBATIM_TAIL_MESSAGES {
            messages.push(msg(MessageRole::Assistant, &format!("tail-{i}")));
        }
        let out = compress_handoff_payload(&messages, 200);
        // The first message must have been truncated AND a sentinel
        // may or may not appear depending on whether we still exceed
        // the budget — either way the size has dropped.
        assert!(out.compressed_bytes < big.len());
        assert!(out.ratio > 1.0);
    }

    #[test]
    fn tail_is_preserved() {
        // Build many tiny messages so the head can be dropped but the
        // tail must remain identical.
        let mut messages = Vec::new();
        for i in 0..20 {
            messages.push(msg(MessageRole::User, &format!("head-{i}")));
        }
        for i in 0..VERBATIM_TAIL_MESSAGES {
            messages.push(msg(MessageRole::Assistant, &format!("tail-{i}")));
        }
        let out = compress_handoff_payload(&messages, 10);
        let tail = &messages[messages.len() - VERBATIM_TAIL_MESSAGES..];
        // The last N messages in the compressed output must equal the
        // tail of the input.
        let out_tail = &out.messages[out.messages.len() - VERBATIM_TAIL_MESSAGES..];
        assert_eq!(out_tail, tail);
    }

    #[test]
    fn ratio_reported_for_compression() {
        let mut messages = Vec::new();
        for i in 0..40 {
            messages.push(msg(MessageRole::User, &"a".repeat(200 + i)));
        }
        for i in 0..VERBATIM_TAIL_MESSAGES {
            messages.push(msg(MessageRole::Assistant, &format!("tail-{i}")));
        }
        let out = compress_handoff_payload(&messages, 200);
        assert!(out.compressed_tokens <= 350); // budget + sentinel slack
        assert!(out.ratio > 5.0, "expected >=5x ratio, got {}", out.ratio);
    }
}

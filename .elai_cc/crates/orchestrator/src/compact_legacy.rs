//! L-16 rollback arm kept for one release behind ELAI_CONTEXT_PROJECTION_DISABLE.
//! The projection path in context_projection.rs is the default.
//! The rollback implementation remains unchanged.

use crate::compact::{
    extract_existing_compacted_summary, format_compact_summary, get_compact_continuation_message,
    merge_compact_summaries, should_compact, summarize_messages, CompactionConfig, CompactionResult,
};
use crate::session::{ContentBlock, ConversationMessage, MessageRole, Session};

pub(crate) fn compact_session_legacy_mutate(session: &Session, config: CompactionConfig) -> CompactionResult {
    if !should_compact(session, config) {
        return CompactionResult {
            summary: String::new(),
            formatted_summary: String::new(),
            compacted_session: session.clone(),
            removed_message_count: 0,
        };
    }

    let existing_summary = session
        .messages
        .first()
        .and_then(extract_existing_compacted_summary);
    let compacted_prefix_len = usize::from(existing_summary.is_some());

    // J-36 frozen-head: preserve leading non-summary System messages
    // (sys+rules+anchor head).
    let frozen_len = crate::context_policy::frozen_head_system_prefix_len(
        |i| {
            session.messages.get(i).is_some_and(|m| m.role == MessageRole::System)
        },
        |i| {
            session
                .messages
                .get(i)
                .and_then(extract_existing_compacted_summary)
                .is_some()
        },
        session.messages.len(),
    );
    let start = compacted_prefix_len.max(frozen_len);

    let keep_from = session
        .messages
        .len()
        .saturating_sub(config.preserve_recent_messages);
    let removed = &session.messages[start..keep_from.max(start)];
    let preserved = session.messages[keep_from.max(start)..].to_vec();
    let summary = merge_compact_summaries(
        existing_summary.as_deref(),
        &summarize_messages(removed, config.preserve_intermediate_numerics),
    );
    let formatted_summary = format_compact_summary(&summary);
    let continuation = get_compact_continuation_message(&summary, true, !preserved.is_empty());

    let mut compacted_messages: Vec<ConversationMessage> =
        session.messages[..frozen_len].to_vec();
    compacted_messages.push(ConversationMessage {
        role: MessageRole::System,
        blocks: vec![ContentBlock::Text { text: continuation }],
        usage: None,
    });
    compacted_messages.extend(preserved);

    let mut compacted_session = session.clone();
    compacted_session.messages = compacted_messages;
    compacted_session.record_compaction(summary.clone(), removed.len());

    CompactionResult {
        summary,
        formatted_summary,
        compacted_session,
        removed_message_count: removed.len(),
    }
}

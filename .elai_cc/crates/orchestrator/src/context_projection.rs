//! Pure provider-facing context projection over an immutable transcript view.
use crate::compact::{
    extract_existing_compacted_summary, format_compact_summary, get_compact_continuation_message,
    merge_compact_summaries, summarize_messages, CompactionConfig,
};
use crate::context_policy::ContextPolicy;
use crate::session::{ConversationMessage, MessageRole, Session};

/// CC-1: a read-only borrow of the append-only transcript.
pub struct TranscriptLog<'a> {
    messages: &'a [ConversationMessage],
}

impl<'a> TranscriptLog<'a> {
    pub fn from_session(session: &'a Session) -> Self { Self { messages: &session.messages } }
    pub fn messages(&self) -> &'a [ConversationMessage] { self.messages }
    pub fn len(&self) -> usize { self.messages.len() }
    pub fn is_empty(&self) -> bool { self.messages.is_empty() }
}

/// The frozen prefix lengths supplied by the production compaction consumer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FreezeState {
    pub frozen_head_len: usize,
    pub compacted_prefix_len: usize,
}

/// Persistent per-turn record used to decide whether the frozen prefix can be reused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FreezeEpoch {
    pub freeze: FreezeState,
    pub boundary_hash: [u8; 8],
    pub raw_log_len: usize,
    pub epoch: u32,
    pub reuse_count: u32,
    pub policy: ContextPolicy,
    pub projected_prefix: Vec<ConversationMessage>,
    pub tail_start: usize,
    /// The `SessionCompaction` generation this prefix was projected against.
    /// `0` means the session had never been compacted. A compaction increments
    /// it (`session.rs:222-230`), which invalidates the prefix.
    pub compaction_generation: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecomputeCause {
    FirstCall,
    TailOverflow,
    BoundaryMismatch,
    HistoryRewound,
    PolicyChanged,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FreezeDecision {
    Reuse,
    Recompute(RecomputeCause),
}

/// Fingerprint the role and textual payload of a boundary message.
#[must_use]
pub fn boundary_fingerprint(message: &ConversationMessage) -> [u8; 8] {
    let role = match message.role {
        MessageRole::System => 0u8,
        MessageRole::User => 1,
        MessageRole::Assistant => 2,
        MessageRole::Tool => 3,
    };
    let text = message.blocks.iter().filter_map(|block| match block {
        crate::session::ContentBlock::Text { text } => Some(text.as_str()),
        _ => None,
    }).collect::<String>();
    let mut hasher = blake3::Hasher::new();
    hasher.update(&[role]);
    hasher.update(&(text.len() as u64).to_le_bytes());
    hasher.update(text.as_bytes());
    let digest = hasher.finalize();
    let mut result = [0u8; 8];
    result.copy_from_slice(&digest.as_bytes()[..8]);
    result
}

#[must_use]
pub fn evaluate_freeze(
    state: Option<&FreezeEpoch>,
    log: &TranscriptLog<'_>,
    budget: &CompactionConfig,
    policy: ContextPolicy,
    compaction_generation: u32,
) -> FreezeDecision {
    let Some(state) = state else { return FreezeDecision::Recompute(RecomputeCause::FirstCall); };
    if log.len() < state.raw_log_len {
        return FreezeDecision::Recompute(RecomputeCause::HistoryRewound);
    }
    if policy != state.policy {
        return FreezeDecision::Recompute(RecomputeCause::PolicyChanged);
    }
    let index = state.freeze.frozen_head_len.saturating_sub(1);
    let Some(boundary) = log.messages().get(index) else {
        return FreezeDecision::Recompute(RecomputeCause::HistoryRewound);
    };
    if boundary_fingerprint(boundary) != state.boundary_hash {
        return FreezeDecision::Recompute(RecomputeCause::BoundaryMismatch);
    }
    if log.len() - state.raw_log_len > budget.preserve_recent_messages {
        return FreezeDecision::Recompute(RecomputeCause::TailOverflow);
    }
    // The compaction generation moved, so the frozen prefix is stale.
    if compaction_generation != state.compaction_generation {
        return FreezeDecision::Recompute(RecomputeCause::PolicyChanged);
    }
    FreezeDecision::Reuse
}

/// The provider-facing view, returned by value without mutating the transcript.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct View {
    pub messages: Vec<ConversationMessage>,
    pub summary: String,
    pub removed_message_count: usize,
    pub log_len: usize,
    pub tail_start: usize,
}

const ELAI_CONTEXT_PROJECTION_DISABLE_VAR: &str = "ELAI_CONTEXT_PROJECTION_DISABLE";

#[cfg(test)]
thread_local! {
    static PROJECTION_DISABLED_OVERRIDE: std::cell::Cell<Option<bool>> =
        std::cell::Cell::new(None);
}

#[cfg(test)]
pub(crate) fn set_projection_disabled_for_test(value: Option<bool>) {
    PROJECTION_DISABLED_OVERRIDE.with(|override_value| override_value.set(value));
}

/// Returns whether the legacy mutating rollback path is requested.
pub fn projection_disabled() -> bool {
    #[cfg(test)]
    if let Some(value) = PROJECTION_DISABLED_OVERRIDE.with(std::cell::Cell::get) {
        return value;
    }

    std::env::var(ELAI_CONTEXT_PROJECTION_DISABLE_VAR)
        .map(|value| value == "1")
        .unwrap_or(false)
}

/// Folds the immutable log into a deterministic provider-facing view.
///
/// The budget parameter is `CompactionConfig` in this slice and widens to
/// `ContextBudget` when the 3E allocator is threaded through; this slice
/// deliberately does not invent that threading. L-17/H-50 make `policy`
/// select the strategy set; today Local and Cloud fold identically.
#[must_use]
pub fn fold(
    log: &TranscriptLog<'_>,
    budget: &CompactionConfig,
    policy: ContextPolicy,
    freeze: &FreezeState,
) -> View {
    let _policy = policy;
    let _ = log.is_empty();
    let messages = log.messages();
    let existing_summary = messages.first().and_then(extract_existing_compacted_summary);
    let start = freeze.compacted_prefix_len.max(freeze.frozen_head_len);
    let keep_from = messages.len().saturating_sub(budget.preserve_recent_messages);
    let end = keep_from.max(start).min(messages.len());
    let removed = &messages[start..end];
    let preserved = messages[end..].to_vec();
    let summary = merge_compact_summaries(
        existing_summary.as_deref(),
        &summarize_messages(removed, budget.preserve_intermediate_numerics),
    );
    let continuation = get_compact_continuation_message(&summary, true, !preserved.is_empty());
    let mut projected = messages[..freeze.frozen_head_len.min(messages.len())].to_vec();
    projected.push(ConversationMessage {
        role: MessageRole::System,
        blocks: vec![crate::session::ContentBlock::Text { text: continuation }],
        usage: None,
    });
    projected.extend(preserved);
    View {
        messages: projected,
        summary,
        removed_message_count: removed.len(),
        log_len: log.len(),
        tail_start: end,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::Session;

    fn inputs() -> (Session, CompactionConfig, FreezeState) {
        let mut session = Session::new();
        session.messages.push(ConversationMessage {
            role: MessageRole::System,
            blocks: vec![crate::session::ContentBlock::Text { text: "head".to_string() }],
            usage: None,
        });
        session.messages.push(ConversationMessage {
            role: MessageRole::User,
            blocks: vec![crate::session::ContentBlock::Text { text: "old".to_string() }],
            usage: None,
        });
        session.messages.push(ConversationMessage {
            role: MessageRole::Assistant,
            blocks: vec![crate::session::ContentBlock::Text { text: "new".to_string() }],
            usage: None,
        });
        (session, CompactionConfig { preserve_recent_messages: 1, max_estimated_tokens: 1, preserve_intermediate_numerics: false }, FreezeState { frozen_head_len: 1, compacted_prefix_len: 0 })
    }

    #[test]
    fn evaluate_freeze_reports_history_rewound_when_the_log_shrinks_freeze() {
        let (session, budget, freeze) = inputs();
        let state = FreezeEpoch {
            freeze,
            boundary_hash: [0; 8],
            raw_log_len: session.messages.len() + 1,
            epoch: 1,
            reuse_count: 0,
            policy: ContextPolicy::Local,
            projected_prefix: Vec::new(),
            tail_start: 0,
            compaction_generation: 0,
        };
        let log = TranscriptLog::from_session(&session);
        assert_eq!(
            evaluate_freeze(Some(&state), &log, &budget, ContextPolicy::Local, 0),
            FreezeDecision::Recompute(RecomputeCause::HistoryRewound)
        );
    }

    #[test]
    fn evaluate_freeze_is_pure_freeze() {
        let (session, budget, freeze) = inputs();
        let log = TranscriptLog::from_session(&session);
        let state = FreezeEpoch {
            freeze,
            boundary_hash: boundary_fingerprint(&session.messages[0]),
            raw_log_len: log.len(),
            epoch: 1,
            reuse_count: 0,
            policy: ContextPolicy::Local,
            projected_prefix: Vec::new(),
            tail_start: 0,
            compaction_generation: 0,
        };
        let before = log.messages().len();
        let first = evaluate_freeze(Some(&state), &log, &budget, ContextPolicy::Local, 0);
        let second = evaluate_freeze(Some(&state), &log, &budget, ContextPolicy::Local, 0);
        assert_eq!(first, second);
        assert_eq!(log.messages().len(), before);
    }

    #[test]
    fn evaluate_freeze_recomputes_when_the_compaction_generation_moves_freeze() {
        let (session, budget, freeze) = inputs();
        let log = TranscriptLog::from_session(&session);
        let state = FreezeEpoch {
            freeze,
            boundary_hash: boundary_fingerprint(&session.messages[0]),
            raw_log_len: log.len(),
            epoch: 1,
            reuse_count: 0,
            policy: ContextPolicy::Local,
            projected_prefix: Vec::new(),
            tail_start: 0,
            compaction_generation: 1,
        };
        assert_eq!(
            evaluate_freeze(Some(&state), &log, &budget, ContextPolicy::Local, 1),
            FreezeDecision::Reuse
        );
        assert_eq!(
            evaluate_freeze(Some(&state), &log, &budget, ContextPolicy::Local, 2),
            FreezeDecision::Recompute(RecomputeCause::PolicyChanged)
        );
    }

    #[test]
    fn fold_is_pure_byte_identical() {
        let (session, budget, freeze) = inputs(); let log = TranscriptLog::from_session(&session);
        assert_eq!(fold(&log, &budget, ContextPolicy::Local, &freeze), fold(&log, &budget, ContextPolicy::Local, &freeze));
    }
    #[test]
    fn fold_never_shortens_the_log() {
        let (session, budget, freeze) = inputs(); let log = TranscriptLog::from_session(&session); let view = fold(&log, &budget, ContextPolicy::Local, &freeze);
        assert_eq!(view.log_len, log.len()); assert_eq!(log.messages().len(), session.messages.len());
    }
    #[test]
    fn fold_preserves_the_frozen_head() {
        let (session, budget, freeze) = inputs(); let log = TranscriptLog::from_session(&session); let view = fold(&log, &budget, ContextPolicy::Local, &freeze);
        assert_eq!(&view.messages[0], &log.messages()[0]);
    }
    #[test]
    fn fold_with_local_and_cloud_policy_agree_today() {
        let (session, budget, freeze) = inputs(); let log = TranscriptLog::from_session(&session);
        assert_eq!(fold(&log, &budget, ContextPolicy::Local, &freeze), fold(&log, &budget, ContextPolicy::Cloud, &freeze));
    }
}

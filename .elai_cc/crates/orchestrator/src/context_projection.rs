//! Pure provider-facing context projection over an immutable transcript view.
use crate::compact::{
    extract_existing_compacted_summary, format_compact_summary, get_compact_continuation_message,
    merge_compact_summaries, summarize_messages, CompactionConfig,
};
use crate::context_policy::{ContextPolicy, PressureLevel};
use crate::pristine_path_filter::MAX_CONSTRAINT_TOKENS;
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
    /// `Session::updated_at_ms` at the moment this prefix was projected, used
    /// to measure the idle gap against the provider cache TTL.
    pub computed_at_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecomputeCause {
    FirstCall,
    TailOverflow,
    BoundaryMismatch,
    HistoryRewound,
    PolicyChanged,
    /// The provider cache went cold because the session sat idle past its TTL.
    ColdGapTtl,
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
    now_ms: u64,
    ttl_ms: Option<u64>,
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
    // The provider cache went cold while the session sat idle, so reuse buys nothing.
    if let Some(ttl_ms) = ttl_ms {
        if now_ms.saturating_sub(state.computed_at_ms) > ttl_ms {
            return FreezeDecision::Recompute(RecomputeCause::ColdGapTtl);
        }
    }
    FreezeDecision::Reuse
}

/// Number of recall cards available at normal pressure.
pub const RECALL_BASE_CARDS: usize = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecallBudget {
    pub cards: usize,
    pub per_card_token_cap: usize,
    pub hints_only: bool,
}

#[must_use]
pub fn recall_budget(level: PressureLevel, base_cards: usize) -> RecallBudget {
    match level {
        PressureLevel::Normal => RecallBudget { cards: base_cards, per_card_token_cap: MAX_CONSTRAINT_TOKENS, hints_only: false },
        PressureLevel::Moderate => RecallBudget { cards: base_cards.div_ceil(2), per_card_token_cap: MAX_CONSTRAINT_TOKENS, hints_only: false },
        PressureLevel::High => RecallBudget { cards: base_cards.min(1), per_card_token_cap: MAX_CONSTRAINT_TOKENS / 4, hints_only: false },
        PressureLevel::Critical => RecallBudget { cards: base_cards.min(1), per_card_token_cap: 0, hints_only: true },
    }
}

/// The provider-facing view, returned by value without mutating the transcript.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct View {
    pub messages: Vec<ConversationMessage>,
    pub summary: String,
    pub removed_message_count: usize,
    pub log_len: usize,
    pub tail_start: usize,
    pub recalled_card_count: usize,
    /// L-19 slice 2 -- the paths whose folded content was paged back this turn,
    /// in card order. Empty on every turn that recalls nothing, which is what
    /// keeps the receipt silent instead of recording an empty recall.
    pub recalled_paths: Vec<String>,
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
        recalled_card_count: 0,
        recalled_paths: Vec::new(),
    }
}


/// Collect failed tool-result evidence from the folded span, preserving first occurrence order.
fn failed_path_evidence(messages: &[ConversationMessage]) -> Vec<(String, String)> {
    let mut evidence = Vec::new();
    for message in messages {
        for block in &message.blocks {
            let crate::session::ContentBlock::ToolResult { output, is_error: true, .. } = block else {
                continue;
            };
            let mut current = String::new();
            for ch in output.chars().chain(std::iter::once(' ')) {
                if ch.is_ascii_alphanumeric() || matches!(ch, '_' | '.' | '/' | '-') {
                    current.push(ch);
                } else {
                    if current.contains('/') && current.contains('.')
                        && !evidence.iter().any(|(path, _)| path == &current)
                    {
                        evidence.push((current.clone(), output.clone()));
                    }
                    current.clear();
                }
            }
        }
    }
    evidence
}

/// Append transient path-based recall cards to the projected tail.
pub fn append_recall_cards(
    view: &mut View,
    log: &TranscriptLog<'_>,
    freeze: &FreezeState,
    level: PressureLevel,
) -> usize {
    let start = freeze.compacted_prefix_len.max(freeze.frozen_head_len);
    let end = view.tail_start;
    if start >= end || end > log.len() {
        view.recalled_card_count = 0;
        view.recalled_paths.clear();
        return 0;
    }
    let folded = &log.messages()[start..end];
    let live_tail = &log.messages()[end..];
    let text = |message: &ConversationMessage| -> String {
        message.blocks.iter().filter_map(|block| match block {
            crate::session::ContentBlock::Text { text } => Some(text.as_str()),
            _ => None,
        }).collect::<Vec<_>>().join(" ")
    };
    let tokens = |messages: &[ConversationMessage]| -> Vec<String> {
        let mut found = Vec::new();
        let mut current = String::new();
        for message in messages {
            for ch in text(message).chars().chain(std::iter::once(' ')) {
                if ch.is_ascii_alphanumeric() || matches!(ch, '_' | '.' | '/' | '-') {
                    current.push(ch);
                } else {
                    if current.contains('/') && current.contains('.') && !found.contains(&current) {
                        found.push(current.clone());
                    }
                    current.clear();
                }
            }
        }
        found
    };
    let folded_tokens = tokens(folded);
    let tail_tokens = tokens(live_tail);
    let budget = recall_budget(level, RECALL_BASE_CARDS);
    let failed = failed_path_evidence(folded);
    let mut candidates = Vec::new();
    for (path, output) in failed {
        if tail_tokens.contains(&path) {
            candidates.push((path, Some(output)));
        }
    }
    for path in folded_tokens {
        if tail_tokens.contains(&path) && !candidates.iter().any(|(candidate, _)| candidate == &path) {
            candidates.push((path, None));
        }
    }
    let mut appended = 0;
    for (path, failed_output) in candidates.into_iter().take(budget.cards) {
        let failed = failed_output.is_some();
        let source = failed_output.unwrap_or_else(|| folded.iter().find_map(|message| {
            let body = text(message);
            body.contains(&path).then_some(body)
        }).unwrap_or_default());
        let card_text = if budget.hints_only {
            format!("[recall pointer] {path}")
        } else if failed {
            let words = source.split_whitespace().take(MAX_CONSTRAINT_TOKENS.min(budget.per_card_token_cap)).collect::<Vec<_>>().join(" ");
            format!("[recall avoid] {path} {words}")
        } else {
            let words = source.split_whitespace().take(budget.per_card_token_cap).collect::<Vec<_>>().join(" ");
            format!("[recall] {path} {words}")
        };
        view.messages.push(ConversationMessage {
            role: MessageRole::System,
            blocks: vec![crate::session::ContentBlock::Text { text: card_text }],
            usage: None,
        });
        appended += 1;
        view.recalled_paths.push(path);
    }
    view.recalled_card_count = appended;
    appended
}


/// L-19 slice 2: build the ambient page-in receipt for one projected view.
///
/// Returns `None` and records NOTHING when the turn appended no recall cards.
/// That silence is deliberate: most turns match no folded path, and a row
/// claiming zero recalled cards would record a recall that never happened.
pub fn recall_receipt_payload(
    view: &View,
    level: PressureLevel,
) -> Option<crate::ssb::SsbPayload> {
    if view.recalled_paths.is_empty() {
        return None;
    }
    let budget = recall_budget(level, RECALL_BASE_CARDS);
    let pressure_level = match level {
        PressureLevel::Normal => 0,
        PressureLevel::Moderate => 1,
        PressureLevel::High => 2,
        PressureLevel::Critical => 3,
    };
    Some(crate::ssb::SsbPayload::ContextRecalled(
        crate::ssb::ContextRecalledPayload {
            pressure_level,
            budget_cards: budget.cards,
            recalled_cards: view.recalled_paths.len(),
            per_card_token_cap: budget.per_card_token_cap,
            pointer_only: budget.hints_only,
            path_digests: view
                .recalled_paths
                .iter()
                .map(|path| *blake3::hash(path.as_bytes()).as_bytes())
                .collect(),
        },
    ))
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

    fn recall_inputs(tail_has_path: bool) -> (Session, CompactionConfig, FreezeState) {
        let mut session = Session::new();
        let entries = [
            (MessageRole::System, "head"),
            (MessageRole::User, "crates/orchestrator/src/session.rs zebrafish"),
            (MessageRole::Assistant, "middle"),
            (MessageRole::User, if tail_has_path { "tail crates/orchestrator/src/session.rs" } else { "tail only" }),
        ];
        for (role, text) in entries {
            session.messages.push(ConversationMessage { role, blocks: vec![crate::session::ContentBlock::Text { text: text.to_string() }], usage: None });
        }
        (session, CompactionConfig { preserve_recent_messages: 1, max_estimated_tokens: 100, preserve_intermediate_numerics: false }, FreezeState { frozen_head_len: 1, compacted_prefix_len: 0 })
    }


    fn failed_recall_inputs(is_error: bool, both: bool) -> (Session, CompactionConfig, FreezeState) {
        let mut session = Session::new();
        session.messages.push(ConversationMessage {
            role: MessageRole::System,
            blocks: vec![crate::session::ContentBlock::Text { text: "head".to_string() }],
            usage: None,
        });
        session.messages.push(ConversationMessage {
            role: MessageRole::Tool,
            blocks: vec![crate::session::ContentBlock::ToolResult {
                tool_use_id: "tool-1".to_string(),
                tool_name: "read".to_string(),
                output: "failed to open src/broken/thing.rs because the path is unavailable and must be avoided".to_string(),
                is_error,
            }],
            usage: None,
        });
        if both {
            session.messages.push(ConversationMessage {
                role: MessageRole::User,
                blocks: vec![crate::session::ContentBlock::Text { text: "also saw src/other/thing.rs".to_string() }],
                usage: None,
            });
        } else {
            session.messages.push(ConversationMessage {
                role: MessageRole::Assistant,
                blocks: vec![crate::session::ContentBlock::Text { text: "middle".to_string() }],
                usage: None,
            });
        }
        let tail = if both {
            "tail src/broken/thing.rs and src/other/thing.rs"
        } else {
            "tail src/broken/thing.rs"
        };
        session.messages.push(ConversationMessage {
            role: MessageRole::User,
            blocks: vec![crate::session::ContentBlock::Text { text: tail.to_string() }],
            usage: None,
        });
        (
            session,
            CompactionConfig { preserve_recent_messages: 1, max_estimated_tokens: 100, preserve_intermediate_numerics: false },
            FreezeState { frozen_head_len: 1, compacted_prefix_len: 0 },
        )
    }

    fn card_text(view: &View, index: usize) -> String {
        match &view.messages[index].blocks[0] {
            crate::session::ContentBlock::Text { text } => text.clone(),
            _ => panic!("expected text card"),
        }
    }

    #[test]
    fn l19_failed_path_recall_uses_contrastive_avoid_form() {
        let (session, budget, freeze) = failed_recall_inputs(true, false);
        let log = TranscriptLog::from_session(&session);
        let mut view = fold(&log, &budget, ContextPolicy::Local, &freeze);
        let before = view.messages.len();
        append_recall_cards(&mut view, &log, &freeze, PressureLevel::Normal);
        let card = card_text(&view, before);
        assert!(card.starts_with("[recall avoid] src/broken/thing.rs"));
        assert!(card.split_whitespace().skip(3).count() <= MAX_CONSTRAINT_TOKENS);
    }

    #[test]
    fn l19_successful_tool_path_does_not_use_avoid_form() {
        let (session, budget, freeze) = failed_recall_inputs(false, false);
        let log = TranscriptLog::from_session(&session);
        let mut view = fold(&log, &budget, ContextPolicy::Local, &freeze);
        append_recall_cards(&mut view, &log, &freeze, PressureLevel::Normal);
        assert!(!view.messages.iter().any(|message| match &message.blocks[0] {
            crate::session::ContentBlock::Text { text } => text.starts_with("[recall avoid]"),
            _ => false,
        }));
    }

    #[test]
    fn l19_failed_path_recall_is_pointer_only_under_critical_pressure() {
        let (session, budget, freeze) = failed_recall_inputs(true, false);
        let log = TranscriptLog::from_session(&session);
        let mut view = fold(&log, &budget, ContextPolicy::Local, &freeze);
        let before = view.messages.len();
        append_recall_cards(&mut view, &log, &freeze, PressureLevel::Critical);
        assert_eq!(card_text(&view, before), "[recall pointer] src/broken/thing.rs");
    }

    #[test]
    fn l19_failed_path_card_precedes_tier_zero_card() {
        let (session, budget, freeze) = failed_recall_inputs(true, true);
        let log = TranscriptLog::from_session(&session);
        let mut view = fold(&log, &budget, ContextPolicy::Local, &freeze);
        let before = view.messages.len();
        append_recall_cards(&mut view, &log, &freeze, PressureLevel::Normal);
        assert!(card_text(&view, before).starts_with("[recall avoid] src/broken/thing.rs"));
        assert!(card_text(&view, before + 1).starts_with("[recall] src/other/thing.rs"));
    }

    #[test]
    fn l19_recall_budget_ladder_scales_by_pressure_band() {
        assert_eq!(recall_budget(PressureLevel::Normal, 4), RecallBudget { cards: 4, per_card_token_cap: 30, hints_only: false });
        assert_eq!(recall_budget(PressureLevel::Moderate, 4), RecallBudget { cards: 2, per_card_token_cap: 30, hints_only: false });
        assert_eq!(recall_budget(PressureLevel::High, 4), RecallBudget { cards: 1, per_card_token_cap: 7, hints_only: false });
        assert_eq!(recall_budget(PressureLevel::Critical, 4), RecallBudget { cards: 1, per_card_token_cap: 0, hints_only: true });
    }

    #[test]
    fn l19_recall_budget_moderate_rounds_up_at_base_one() {
        assert_eq!(recall_budget(PressureLevel::Moderate, 1).cards, 1);
        assert_eq!(recall_budget(PressureLevel::Moderate, 0).cards, 0);
    }

    #[test]
    fn l19_recall_cards_ride_the_tail_and_leave_the_prefix_unchanged() {
        let (session, budget, freeze) = recall_inputs(true); let log = TranscriptLog::from_session(&session);
        let mut view = fold(&log, &budget, ContextPolicy::Local, &freeze); let before = view.messages.clone();
        let fingerprint = boundary_fingerprint(&session.messages[0]);
        let appended = append_recall_cards(&mut view, &log, &freeze, PressureLevel::Normal);
        assert!(appended > 0); assert_eq!(view.recalled_card_count, appended);
        assert_eq!(boundary_fingerprint(&session.messages[0]), fingerprint);
        assert_eq!(&view.messages[..before.len()], &before[..]);
        assert!(view.messages[before.len()..].iter().all(|m| m.role == MessageRole::System));
        assert!(view.messages[before.len()..].iter().any(|m| format!("{:?}", m.blocks).contains("crates/orchestrator/src/session.rs")));
    }

    #[test]
    fn l19_no_recall_when_no_folded_path_resurfaces() {
        let (session, budget, freeze) = recall_inputs(false); let log = TranscriptLog::from_session(&session);
        let mut view = fold(&log, &budget, ContextPolicy::Local, &freeze); let before = view.messages.clone();
        assert_eq!(append_recall_cards(&mut view, &log, &freeze, PressureLevel::Normal), 0);
        assert_eq!(view.recalled_card_count, 0); assert_eq!(view.messages, before);
    }

    #[test]
    fn l19_hints_only_card_under_critical_carries_no_folded_content() {
        let (session, budget, freeze) = recall_inputs(true); let log = TranscriptLog::from_session(&session);
        let mut view = fold(&log, &budget, ContextPolicy::Local, &freeze); let before = view.messages.len();
        assert_eq!(append_recall_cards(&mut view, &log, &freeze, PressureLevel::Critical), 1);
        let card = format!("{:?}", view.messages[before].blocks);
        assert!(card.contains("crates/orchestrator/src/session.rs")); assert!(!card.contains("zebrafish"));
    }

    #[test]
    fn l19_recall_receipt_is_silent_when_nothing_was_recalled() {
        let (session, budget, freeze) = recall_inputs(false);
        let log = TranscriptLog::from_session(&session);
        let mut view = fold(&log, &budget, ContextPolicy::Local, &freeze);
        let appended = append_recall_cards(&mut view, &log, &freeze, PressureLevel::Normal);
        assert_eq!(appended, 0);
        assert_eq!(recall_receipt_payload(&view, PressureLevel::Normal), None);
    }

    #[test]
    fn l19_recall_receipt_carries_the_band_budget_and_one_digest_per_card() {
        let (session, budget, freeze) = recall_inputs(true);
        let log = TranscriptLog::from_session(&session);
        let mut view = fold(&log, &budget, ContextPolicy::Local, &freeze);
        let appended = append_recall_cards(&mut view, &log, &freeze, PressureLevel::Normal);
        assert!(appended > 0);
        let Some(crate::ssb::SsbPayload::ContextRecalled(payload)) =
            recall_receipt_payload(&view, PressureLevel::Normal)
        else { panic!("expected ContextRecalled payload"); };
        assert_eq!(payload.pressure_level, 0);
        assert!(!payload.pointer_only);
        assert_eq!(payload.per_card_token_cap, MAX_CONSTRAINT_TOKENS);
        assert_eq!(payload.budget_cards, RECALL_BASE_CARDS);
        assert_eq!(payload.recalled_cards, view.recalled_card_count);
        assert_eq!(payload.path_digests.len(), payload.recalled_cards);
        assert_eq!(
            payload.path_digests[0],
            *blake3::hash(view.recalled_paths[0].as_bytes()).as_bytes()
        );
    }

    #[test]
    fn l19_recall_receipt_reports_pointer_only_at_critical() {
        let (session, budget, freeze) = recall_inputs(true);
        let log = TranscriptLog::from_session(&session);
        let mut view = fold(&log, &budget, ContextPolicy::Local, &freeze);
        append_recall_cards(&mut view, &log, &freeze, PressureLevel::Critical);
        let Some(crate::ssb::SsbPayload::ContextRecalled(payload)) =
            recall_receipt_payload(&view, PressureLevel::Critical)
        else { panic!("expected ContextRecalled payload"); };
        assert_eq!(payload.pressure_level, 3);
        assert!(payload.pointer_only);
        assert_eq!(payload.per_card_token_cap, 0);
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
            computed_at_ms: 0,
        };
        let log = TranscriptLog::from_session(&session);
        assert_eq!(
            evaluate_freeze(Some(&state), &log, &budget, ContextPolicy::Local, 0, 0, None),
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
            computed_at_ms: 0,
        };
        let before = log.messages().len();
        let first = evaluate_freeze(Some(&state), &log, &budget, ContextPolicy::Local, 0, 0, None);
        let second = evaluate_freeze(Some(&state), &log, &budget, ContextPolicy::Local, 0, 0, None);
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
            computed_at_ms: 0,
        };
        assert_eq!(
            evaluate_freeze(Some(&state), &log, &budget, ContextPolicy::Local, 1, 0, None),
            FreezeDecision::Reuse
        );
        assert_eq!(
            evaluate_freeze(Some(&state), &log, &budget, ContextPolicy::Local, 2, 0, None),
            FreezeDecision::Recompute(RecomputeCause::PolicyChanged)
        );
    }

    #[test]
    fn evaluate_freeze_recomputes_when_the_idle_gap_passes_the_ttl_freeze() {
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
            computed_at_ms: 1_000_000,
        };
        let ttl_ms = 300;
        assert_eq!(
            evaluate_freeze(Some(&state), &log, &budget, ContextPolicy::Local, 0, 1_000_000 + ttl_ms + 1, Some(ttl_ms)),
            FreezeDecision::Recompute(RecomputeCause::ColdGapTtl)
        );
        assert_eq!(
            evaluate_freeze(Some(&state), &log, &budget, ContextPolicy::Local, 0, 1_000_000 + ttl_ms, Some(ttl_ms)),
            FreezeDecision::Reuse
        );
        assert_eq!(
            evaluate_freeze(Some(&state), &log, &budget, ContextPolicy::Local, 0, 1_000_301, None),
            FreezeDecision::Reuse
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
